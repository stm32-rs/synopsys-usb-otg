#![allow(dead_code)]
use core::{slice, mem};
use vcell::VolatileCell;
use crate::target::{UsbAccessType, fifo_read_into};
use usb_device::{Result, UsbError};

#[derive(Default)]
pub struct EndpointBuffer(&'static mut [VolatileCell<u32>]);

impl EndpointBuffer {
    pub fn new(buffer: &'static mut [u32]) -> Self {
        Self(unsafe { mem::transmute(buffer) })
    }

    pub fn read_packet(&self, mut buf: &mut [u8]) -> Result<()> {
        let data_size = if let Some(size) = self.data_size() {
            size
        } else {
            return Err(UsbError::WouldBlock)
        };

        if buf.len() < data_size {
            return Err(UsbError::BufferOverflow);
        }

        let mut index = 1;
        while data_size >= 4 {
            let word = self.0[index].get();
            index += 1;

            let bytes = word.to_ne_bytes();
            buf[..4].copy_from_slice(&bytes);
            buf = &mut buf[4..];
        }
        if data_size > 0 {
            let word = self.0[index].get();
            let bytes = word.to_ne_bytes();
            buf[..data_size].copy_from_slice(&bytes[..data_size]);
        }

        self.set_data_size(None);

        Ok(())
    }

    pub fn fill_from_fifo(&self, data_size: usize) -> Result<()> {
        if self.data_size().is_some() {
            return Err(UsbError::WouldBlock);
        }

        if data_size > self.capacity() {
            return Err(UsbError::BufferOverflow);
        }

        let words = (data_size + 3) / 4;
        fifo_read_into(&self.0[1..1 + words]);

        self.set_data_size(Some(data_size));

        Ok(())
    }

    fn set_data_size(&self, size: Option<usize>) {
        let value = size.map(|size| 0x8000_0000 | (size as u32)).unwrap_or(0);
        self.0[0].set(value);
    }

    pub fn data_size(&self) -> Option<usize> {
        let v = self.0[0].get();
        if v & 0x8000_0000 != 0 {
            Some((v & 0x7fff_ffff) as usize)
        } else {
            None
        }
    }

    pub fn capacity(&self) -> usize {
        (self.0.len() - 1) * 4
    }
}

unsafe impl Sync for EndpointBuffer {}

#[repr(C)]
pub struct BufferDescriptor {
    pub addr_tx: VolatileCell<UsbAccessType>,
    pub count_tx: VolatileCell<UsbAccessType>,
    pub addr_rx: VolatileCell<UsbAccessType>,
    pub count_rx: VolatileCell<UsbAccessType>,
}

pub struct EndpointMemoryAllocator {
    next_free_offset: usize,
    memory: &'static mut [u32],
}

impl EndpointMemoryAllocator {
    pub fn new(memory: &'static mut [u32]) -> Self {
        Self {
            next_free_offset: 0,
            memory
        }
    }

    pub fn allocate_rx_buffer(&mut self, size: usize) -> Result<EndpointBuffer> {
        let size_words = (size + 3) / 4;

        let offset = self.next_free_offset;
        if offset + size_words > self.memory.len() {
            return Err(UsbError::EndpointMemoryOverflow);
        }

        self.next_free_offset += size_words;

        let buffer = unsafe {
            let ptr = self.memory.as_mut_ptr().offset(offset as isize);
            slice::from_raw_parts_mut(ptr, size_words)
        };
        Ok(EndpointBuffer::new(buffer))
    }

    pub fn total_rx_buffer_size(&self) -> usize {
        self.next_free_offset
    }
}
