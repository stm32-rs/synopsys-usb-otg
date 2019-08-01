#![no_std]
#![no_main]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use stm32f4xx_hal::{prelude::*, stm32};

use stm32f429_usbd_fs::{UsbBus, sprintln};
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

    let gpioa = dp.GPIOA.split();

    let usb_dm = gpioa.pa11.into_alternate_af10();
    let usb_dp = gpioa.pa12.into_alternate_af10();

    let usb_bus = UsbBus::new(dp.OTG_FS_GLOBAL, (usb_dm, usb_dp), unsafe { &mut EP_MEMORY });

    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x5824, 0x27dd))
        .manufacturer("Fake company")
        .product("Serial port")
        .serial_number("TEST")
        .device_class(0)
        .build();

    loop {
        if usb_dev.poll(&mut []) {
        }
    }
}
