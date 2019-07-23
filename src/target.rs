#![allow(dead_code)]
#![allow(unused_imports)]
//! Target-specific definitions

// Export HAL
pub use stm32f4xx_hal as hal;


// USB PAC reexports
pub use hal::stm32::OTG_FS_GLOBAL as USB;
use stm32ral::{otg_fs_global, otg_fs_device, otg_fs_pwrclk};

// Use bundled register definitions instead of device-specific ones
// This should work because register definitions from newer chips seem to be
// compatible with definitions for older ones.
pub use crate::pac::usb;


// TODO: remove these
pub type UsbAccessType = u32;
pub const EP_MEM_ADDR: usize = 0x4000_6000;
pub const EP_MEM_SIZE: usize = 1024;


pub const NUM_ENDPOINTS: usize = 8;


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
    pub global: otg_fs_global::Instance,
    pub device: otg_fs_device::Instance,
}

unsafe impl Send for UsbRegisters {}

impl UsbRegisters {
    pub fn new(_usb: USB) -> Self {
        Self {
            global: unsafe { otg_fs_global::OTG_FS_GLOBAL::steal() },
            device: unsafe { otg_fs_device::OTG_FS_DEVICE::steal() },
        }
    }

    pub fn ep_register(index: u8) -> &'static usb::EPR {
        let usb_ptr = USB::ptr() as *const usb::RegisterBlock;
        unsafe { &(*usb_ptr).epr[index as usize] }
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
