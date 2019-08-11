//! USB peripheral driver for STM32 microcontrollers.
//!
//! This also serves as the reference implementation and example repository for the `usb-device`
//! crate for now.

#![no_std]

#[cfg(all(feature = "fs", feature = "hs"))]
compile_error!("choose only one USB mode");

#[cfg(not(any(feature = "fs", feature ="hs")))]
compile_error!("select USB mode feature (fs/hs)");

mod endpoint;
mod endpoint_memory;

mod target;

/// USB peripheral driver.
pub mod bus;

pub use crate::bus::UsbBus;
pub use crate::target::usb_pins::UsbPinsType;
pub type UsbBusType = UsbBus<UsbPinsType>;

mod ral;
