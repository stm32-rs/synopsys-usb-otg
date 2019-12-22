[![crates.io](https://img.shields.io/crates/d/synopsys-usb-otg.svg)](https://crates.io/crates/synopsys-usb-otg)
[![crates.io](https://img.shields.io/crates/v/synopsys-usb-otg.svg)](https://crates.io/crates/synopsys-usb-otg)
[![Build Status](https://travis-ci.com/stm32-rs/synopsys-usb-otg.svg?branch=master)](https://travis-ci.com/stm32-rs/synopsys-usb-otg)

# `synopsys-usb-otg`

> [usb-device](https://github.com/mvirkkunen/usb-device) implementation for Synopsys USB OTG IP cores.

This repository is a fork for the [mvirkkunen/stm32f103xx-usb](https://github.com/mvirkkunen/stm32f103xx-usb) crate.

## Supported microcontrollers

* `STM32F429xx` (OTG_FS and OTG_HS in FS mode)
* `STM32F401xx`
* And others...


## Usage

This driver is intended for use through a device hal library.
Such hal library should implement `UsbPeripheral` for the corresponding USB peripheral object.
This trait declares all the peripheral properties that may vary from one device family to the other.
Additionally, hal should pass `fs` of `hs` feature to the `synopsys-usb-otg` library to
define a peripheral type:
* `fs` - for FullSpeed peripherals
* `hs` - for HighSpeed peripherals (only FS mode with internal PHY is supported)

Only one peripheral type can be selected at the moment.

## Examples

See the [usb-otg-workspace](https://github.com/Disasm/usb-otg-workspace) repo for different device-specific examples.
