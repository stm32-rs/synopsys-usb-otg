//! CDC-ACM serial port example using polling in a busy loop.
#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32f429_usbd_fs::{UsbBus, sprintln};
use stm32f4xx_hal::{prelude::*, stm32};
use usb_device::prelude::*;

static mut EP_MEMORY: [u32; 1024] = [0; 1024];

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().unwrap();

    let rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .sysclk(48.mhz())
        .pclk1(24.mhz())
        .require_pll48clk()
        .freeze();

    let gpiod = dp.GPIOD.split();
    stm32f429_usbd_fs::debug::configure(dp.USART3, gpiod.pd8, gpiod.pd9, 115_200.bps(), clocks);

    sprintln!("==========================");

    let gpiob = dp.GPIOB.split();
    let mut led = gpiob.pb7.into_push_pull_output();
    led.set_low(); // Turn off


    let gpioa = dp.GPIOA.split();

    let usb_dm = gpioa.pa11.into_alternate_af10();
    let usb_dp = gpioa.pa12.into_alternate_af10();

    let usb_bus = UsbBus::new((dp.OTG_FS_GLOBAL, dp.OTG_FS_DEVICE, dp.OTG_FS_PWRCLK), (usb_dm, usb_dp), unsafe { &mut EP_MEMORY });

    let mut serial = usbd_serial::SerialPort::new(&usb_bus);

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x5824, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(usbd_serial::USB_CLASS_CDC)
        .build();

    loop {
        if !usb_dev.poll(&mut [&mut serial]) {
            continue;
        }

        let mut buf = [0u8; 64];

        match serial.read(&mut buf) {
            Ok(count) if count > 0 => {
                led.set_high(); // Turn on

                // Echo back in upper case
                for c in buf[0..count].iter_mut() {
                    if 0x61 <= *c && *c <= 0x7a {
                        *c &= !0x20;
                    }
                }

                let mut write_offset = 0;
                while write_offset < count {
                    match serial.write(&buf[write_offset..count]) {
                        Ok(len) if len > 0 => {
                            write_offset += len;
                        },
                        _ => {},
                    }
                }
            }
            _ => {}
        }

        led.set_low(); // Turn off
    }
}
