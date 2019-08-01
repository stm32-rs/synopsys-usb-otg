#![allow(dead_code)]
#![allow(unused_imports)]
//! Target-specific definitions

// Export HAL
pub use stm32f4xx_hal as hal;


// USB PAC reexports
pub use hal::stm32::OTG_FS_GLOBAL as USB;
pub use hal::stm32::OTG_FS_GLOBAL;
pub use hal::stm32::OTG_FS_DEVICE;
pub use hal::stm32::OTG_FS_PWRCLK;

use stm32ral::{read_reg, write_reg, modify_reg, reset_reg};
use stm32ral::{otg_fs_global, otg_fs_device, otg_fs_pwrclk};

pub fn init_core(instance: &otg_fs_global::Instance) {
    modify_reg!(otg_fs_global, instance, FS_GAHBCFG,
        GINT: 1, // Unmask the interrupt assertion to the application
        TXFELVL: 0, // TXFE interrupt indicates that the IN Endpoint TxFIFO is half empty
        PTXFELVL: 0 // PTXFE interrupt indicates that the Periodic TxFIFO is half empty
    );

    modify_reg!(otg_fs_global, instance, FS_GUSBCFG,
        HNPCAP: 0, // HNP capability is not enabled
        SRPCAP: 0, // SRP capability is not enabled
        TOCAL: 0, // ??? FS timeout calibration
        TRDT: 0xF, // ??? USB turnaround time
        FDMOD: 1 // Force device mode
    );

    modify_reg!(otg_fs_global, instance, FS_GINTMSK,
        OTGINT: 1, // OTG interrupt mask - unmasked
        MMISM: 1 // Mode mismatch interrupt mask - unmasked
    );
}

pub fn init_device(global: &otg_fs_global::Instance, device: &otg_fs_device::Instance) {
    modify_reg!(otg_fs_device, device, FS_DCFG,
        DSPD: 0b11, // Device speed: Full speed
        NZLSOHSK: 1 // Send a STALL handshake on a nonzero-length status OUT transaction and
                    // do not send the received OUT packet to the application
    );

    modify_reg!(otg_fs_global, global, FS_GINTMSK,
        USBRST: 1,
        ENUMDNEM: 1,
        ESUSPM: 1,
        USBSUSPM: 1,
        SOFM: 1
    );

    modify_reg!(otg_fs_global, global, FS_GCCFG, VBUSBSEN: 1);

    // Next: wait for USB reset
}

// Use bundled register definitions instead of device-specific ones
// This should work because register definitions from newer chips seem to be
// compatible with definitions for older ones.
pub use crate::pac::usb;


// TODO: remove these
pub type UsbAccessType = u32;
pub const EP_MEM_ADDR: usize = 0x4000_6000;
pub const EP_MEM_SIZE: usize = 1024;

pub const OTG_FS_BASE: usize = 0x5000_0000;
pub const FIFO_OFFSET: usize = 0x1000;
pub const FIFO_SIZE: usize = 0x1000;


pub const NUM_ENDPOINTS: usize = 8;

#[inline(always)]
fn fifo_ptr(channel: usize) -> &'static vcell::VolatileCell<u32> {
    assert!(channel <= 15);
    let address = OTG_FS_BASE + FIFO_OFFSET + channel * FIFO_SIZE;
    unsafe { &*(address as *const vcell::VolatileCell<u32>) }
}

pub fn fifo_write(channel: impl Into<usize>, mut buf: &[u8]) {
    let fifo = fifo_ptr(channel.into());

    while buf.len() >= 4 {
        let mut u32_bytes = [0u8; 4];
        u32_bytes.copy_from_slice(&buf[..4]);
        buf = &buf[4..];
        fifo.set(u32::from_ne_bytes(u32_bytes));
    }
    if buf.len() > 0 {
        let mut u32_bytes = [0u8; 4];
        u32_bytes[..buf.len()].copy_from_slice(buf);
        fifo.set(u32::from_ne_bytes(u32_bytes));
    }
}

pub fn fifo_read(mut buf: &mut [u8]) {
    let fifo = fifo_ptr(0);

    while buf.len() >= 4 {
        let word = fifo.get();
        let bytes = word.to_ne_bytes();
        buf[..4].copy_from_slice(&bytes);
        buf = &mut buf[4..];
    }
    if buf.len() > 0 {
        let word = fifo.get();
        let bytes = word.to_ne_bytes();
        buf.copy_from_slice(&bytes[..buf.len()]);
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
    pub global: otg_fs_global::Instance,
    pub device: otg_fs_device::Instance,
    pub pwrclk: otg_fs_pwrclk::Instance,
}

unsafe impl Send for UsbRegisters {}

impl UsbRegisters {
    pub fn new(_usb: USB) -> Self {
        Self {
            global: unsafe { otg_fs_global::OTG_FS_GLOBAL::steal() },
            device: unsafe { otg_fs_device::OTG_FS_DEVICE::steal() },
            pwrclk: unsafe { otg_fs_pwrclk::OTG_FS_PWRCLK::steal() },
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
