#![allow(dead_code)]
#![allow(unused_imports)]
//! Target-specific definitions

use vcell::VolatileCell;

// Export HAL
pub use stm32f4xx_hal as hal;


// USB PAC reexports
pub use hal::stm32::OTG_FS_GLOBAL as OTG_GLOBAL;
pub use hal::stm32::OTG_FS_DEVICE as OTG_DEVICE;
pub use hal::stm32::OTG_FS_PWRCLK as OTG_PWRCLK;

use crate::ral::{otg_global, otg_device, otg_pwrclk, otg_fifo};

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

/// Enables USB peripheral
pub fn apb_usb_enable() {
    cortex_m::interrupt::free(|_| {
        let rcc = unsafe { (&*hal::stm32::RCC::ptr()) };
        rcc.ahb2enr.modify(|_, w| w.otgfsen().set_bit());
    });
}

// Workaround: cortex-m contains pre-compiled __delay function
// on which rust-lld fails with "unrecognized reloc 103"
// https://github.com/rust-embedded/cortex-m/issues/125
#[cfg(feature = "delay_workaround")]
pub fn delay(n: u32) {
    #[inline(never)]
    fn __delay(mut n: u32) {
        while n > 0 {
            n -= 1;
            cortex_m::asm::nop();
        }
    }

    __delay(n / 4 + 1);
}
#[cfg(not(feature = "delay_workaround"))]
pub use cortex_m::asm::delay;


/// Wrapper around device-specific peripheral that provides unified register interface
pub struct UsbRegisters {
    pub global: otg_global::Instance,
    pub device: otg_device::Instance,
    pub pwrclk: otg_pwrclk::Instance,
}

unsafe impl Send for UsbRegisters {}

impl UsbRegisters {
    pub fn new(_global: OTG_GLOBAL, _device: OTG_DEVICE, _pwrclk: OTG_PWRCLK) -> Self {
        Self {
            global: unsafe { otg_global::OTG_GLOBAL::steal() },
            device: unsafe { otg_device::OTG_DEVICE::steal() },
            pwrclk: unsafe { otg_pwrclk::OTG_PWRCLK::steal() },
        }
    }
}



pub trait UsbPins: Send { }


pub mod usb_pins {
    use super::hal::gpio::{AF10, Alternate};
    use super::hal::gpio::gpioa::{PA11, PA12};

    pub type UsbPinsType = (
        PA11<Alternate<AF10>>,
        PA12<Alternate<AF10>>
    );
    impl super::UsbPins for UsbPinsType {}
}
