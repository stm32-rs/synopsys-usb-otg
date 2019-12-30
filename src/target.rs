#![allow(dead_code)]
#![allow(unused_imports)]
//! Target-specific definitions

use vcell::VolatileCell;
use core::marker::PhantomData;

#[cfg(feature = "cortex-m")]
pub use cortex_m::interrupt;
#[cfg(feature = "riscv")]
pub use riscv::interrupt;

use crate::ral::{otg_global, otg_device, otg_pwrclk, otg_fifo};
use crate::UsbPeripheral;

pub fn fifo_write(channel: impl Into<usize>, mut buf: &[u8]) {
    let fifo = otg_fifo::instance(channel.into());

    while buf.len() >= 4 {
        let mut u32_bytes = [0u8; 4];
        u32_bytes.copy_from_slice(&buf[..4]);
        buf = &buf[4..];
        fifo.write(u32::from_ne_bytes(u32_bytes));
    }
    if buf.len() > 0 {
        let mut u32_bytes = [0u8; 4];
        u32_bytes[..buf.len()].copy_from_slice(buf);
        fifo.write(u32::from_ne_bytes(u32_bytes));
    }
}

pub fn fifo_read(mut buf: &mut [u8]) {
    let fifo = otg_fifo::instance(0);

    while buf.len() >= 4 {
        let word = fifo.read();
        let bytes = word.to_ne_bytes();
        buf[..4].copy_from_slice(&bytes);
        buf = &mut buf[4..];
    }
    if buf.len() > 0 {
        let word = fifo.read();
        let bytes = word.to_ne_bytes();
        buf.copy_from_slice(&bytes[..buf.len()]);
    }
}

pub fn fifo_read_into(buf: &[VolatileCell<u32>]) {
    let fifo = otg_fifo::instance(0);

    for p in buf {
        let word = fifo.read();
        p.set(word);
    }
}

/// Wrapper around device-specific peripheral that provides unified register interface
pub struct UsbRegisters<USB> {
    pub global: otg_global::Instance,
    pub device: otg_device::Instance,
    pub pwrclk: otg_pwrclk::Instance,
    _marker: PhantomData<USB>,
}

unsafe impl<USB> Send for UsbRegisters<USB> {}

impl<USB: UsbPeripheral> UsbRegisters<USB> {
    pub fn new() -> Self {
        Self {
            global: unsafe { otg_global::OTG_GLOBAL::steal() },
            device: unsafe { otg_device::OTG_DEVICE::steal() },
            pwrclk: unsafe { otg_pwrclk::OTG_PWRCLK::steal() },
            _marker: PhantomData,
        }
    }
}
