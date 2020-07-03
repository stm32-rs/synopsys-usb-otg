#![allow(dead_code)]
use core::{slice, mem};
use core::marker::PhantomData;
use vcell::VolatileCell;
use crate::UsbPeripheral;
use crate::target::fifo_read_into;
use usb_device::{Result, UsbError};

#[derive(Eq, PartialEq)]
pub enum EndpointBufferState {
    Empty,
    DataOut,
    DataSetup,
}

pub struct EndpointBuffer {
    buffer: &'static mut [VolatileCell<u32>],
    data_size: u16,
    has_data: bool,
    is_setup: bool,
}

impl EndpointBuffer {
    pub fn new(buffer: &'static mut [u32]) -> Self {
        Self {
            buffer: unsafe { mem::transmute(buffer) },
            data_size: 0,
            has_data: false,
            is_setup: false
        }
    }

    pub fn read_packet(&mut self, mut buf: &mut [u8]) -> Result<usize> {
        if !self.has_data {
            return Err(UsbError::WouldBlock)
        }

        let data_size = self.data_size as usize;

        if buf.len() < data_size {
            return Err(UsbError::BufferOverflow);
        }

        let mut index = 0;
        let mut current_size = data_size;
        while current_size >= 4 {
            let word = self.buffer[index].get();
            index += 1;

            let bytes = word.to_ne_bytes();
            buf[..4].copy_from_slice(&bytes);
            buf = &mut buf[4..];

            current_size -= 4;
        }
        if current_size > 0 {
            let word = self.buffer[index].get();
            let bytes = word.to_ne_bytes();
            buf[..current_size].copy_from_slice(&bytes[..current_size]);
        }

        self.has_data = false;

        Ok(data_size)
    }

    pub fn fill_from_fifo(&mut self, data_size: u16, is_setup: bool) -> Result<()> {
        if self.has_data {
            return Err(UsbError::WouldBlock);
        }

        if data_size as usize > self.capacity() {
            return Err(UsbError::BufferOverflow);
        }

        let words = (data_size as usize + 3) / 4;
        fifo_read_into(&self.buffer[..words]);

        self.is_setup = is_setup;
        self.data_size = data_size;
        self.has_data = true;

        Ok(())
    }

    pub fn state(&self) -> EndpointBufferState {
        if self.has_data {
            if self.is_setup {
                EndpointBufferState::DataSetup
            } else {
                EndpointBufferState::DataOut
            }
        } else {
            EndpointBufferState::Empty
        }
    }

    pub fn capacity(&self) -> usize {
        self.buffer.len() * 4
    }
}

impl Default for EndpointBuffer {
    fn default() -> Self {
        EndpointBuffer::new(&mut [])
    }
}


pub struct EndpointMemoryAllocator<USB> {
    next_free_offset: usize,
    max_size_words: usize,
    memory: &'static mut [u32],
    tx_fifo_size_words: [u16; 8],
    _marker: PhantomData<USB>,
}

impl<USB: UsbPeripheral> EndpointMemoryAllocator<USB> {
    pub fn new(memory: &'static mut [u32]) -> Self {
        Self {
            next_free_offset: 0,
            max_size_words: 0,
            memory,
            tx_fifo_size_words: [0; 8],
            _marker: PhantomData
        }
    }

    pub fn allocate_rx_buffer(&mut self, size: usize) -> Result<EndpointBuffer> {
        let size_words = (size + 3) / 4;

        let offset = self.next_free_offset;
        if offset + size_words > self.memory.len() {
            return Err(UsbError::EndpointMemoryOverflow);
        }

        self.next_free_offset += size_words;
        self.max_size_words = core::cmp::max(self.max_size_words, size_words);

        let buffer = unsafe {
            let ptr = self.memory.as_mut_ptr().offset(offset as isize);
            slice::from_raw_parts_mut(ptr, size_words)
        };
        Ok(EndpointBuffer::new(buffer))
    }

    pub fn allocate_tx_buffer(&mut self, ep_number: u8, size: usize) -> Result<()> {
        let ep_number = ep_number as usize;
        assert!(ep_number < self.tx_fifo_size_words.len());

        if self.tx_fifo_size_words[ep_number] != 0 {
            return Err(UsbError::InvalidEndpoint)
        }

        let mut used = self.total_rx_buffer_size_words() as usize + 30;
        for sz in &self.tx_fifo_size_words {
            used += core::cmp::max(*sz as usize, 16);
        }
        used -= 16;

        let size_words = core::cmp::max((size + 3) / 4, 16);
        if (used + size_words) > USB::FIFO_DEPTH_WORDS {
            return Err(UsbError::EndpointMemoryOverflow);
        }

        self.tx_fifo_size_words[ep_number] = size_words as u16;

        Ok(())
    }

    /// Returns the size of memory allocated for OUT endpoints in words
    pub fn total_rx_buffer_size_words(&self) -> u16 {
        self.next_free_offset as u16
    }

    pub fn tx_fifo_size_words(&self, ep_number: usize) -> u16 {
        self.tx_fifo_size_words[ep_number]
    }

    pub fn max_buffer_size_words(&self) -> usize {
        self.max_size_words
    }
}
