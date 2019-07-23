use core::{slice, mem};
use vcell::VolatileCell;
use crate::target::UsbAccessType;
use usb_device::{Result, UsbError};

pub struct EndpointBuffer(&'static mut [VolatileCell<u32>]);

impl EndpointBuffer {
    pub fn new(buffer: &'static mut [u32]) -> Self {
        Self(unsafe { mem::transmute(buffer) })
    }

    #[inline(always)]
    fn read_word(&self, index: usize) -> u16 {
        (self.0[index].get() & 0xffff) as u16
    }

    #[inline(always)]
    fn write_word(&self, index: usize, value: u16) {
        self.0[index].set(value as UsbAccessType);
    }

    pub fn read(&self, mut buf: &mut [u8]) {
        let mut index = 0;

        while buf.len() >= 2 {
            let word = self.read_word(index);

            buf[0] = (word & 0xff) as u8;
            buf[1] = (word >> 8) as u8;

            index += 1;

            buf = &mut buf[2..];
        }

        if buf.len() > 0 {
            buf[0] = (self.read_word(index) & 0xff) as u8;
        }
    }

    pub fn write(&self, mut buf: &[u8]) {
        let mut index = 0;

        while buf.len() >= 2 {
            let value: u16 = buf[0] as u16 | ((buf[1] as u16) << 8);
            self.write_word(index, value);
            index += 1;

            buf = &buf[2..];
        }

        if buf.len() > 0 {
            self.write_word(index, buf[0] as u16);
        }
    }

    pub fn capacity(&self) -> usize {
        self.0.len() << 1
    }
}

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

    pub fn allocate_buffer(&mut self, size: usize) -> Result<EndpointBuffer> {
        assert!(size < 1024);

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
}
