//! Debug interface based on the UART hooked up to ST-LINK

use core::fmt::{self, Write};
use stm32f4xx_hal::nb::block;
use cortex_m::interrupt;
use stm32f4xx_hal::{
    serial::{Serial, Tx},
    time::Bps,
    stm32::USART3,
    prelude::*
};
use stm32f4xx_hal::gpio::{gpiod::{PD8, PD9}, Alternate, AF7};
use stm32f4xx_hal::rcc::Clocks;
use stm32f4xx_hal::serial::config::Config;


static mut STDOUT: Option<SerialWrapper> = None;

type TxPin = PD8<Alternate<AF7>>;
type RxPin = PD9<Alternate<AF7>>;

struct SerialWrapper(Tx<USART3>);

impl Write for SerialWrapper {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.as_bytes() {
            if *byte == '\n' as u8 {
                let res = block!(self.0.write('\r' as u8));

                if res.is_err() {
                    return Err(::core::fmt::Error);
                }
            }

            let res = block!(self.0.write(*byte));

            if res.is_err() {
                return Err(::core::fmt::Error);
            }
        }
        Ok(())
    }
}


/// Configures stdout
pub fn configure<X, Y>(
    uart: USART3, tx: PD8<X>, rx: PD9<Y>,
    baudrate: Bps, clocks: Clocks
) {
    let config = Config {
        baudrate,
        ..Config::default()
    };
    let tx = tx.into_alternate_af7();
    let rx = rx.into_alternate_af7();
    let serial = Serial::usart3(uart, (tx, rx), config, clocks).unwrap();
    let (tx, _) = serial.split();

    interrupt::free(|_| {
        unsafe {
            STDOUT.replace(SerialWrapper(tx));
        }
    })
}

/// Writes string to stdout
pub fn write_str(s: &str) {
    interrupt::free(|_| unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = stdout.write_str(s);
        }
    })
}

/// Writes formatted string to stdout
pub fn write_fmt(args: fmt::Arguments) {
    interrupt::free(|_| unsafe {
        if let Some(stdout) = STDOUT.as_mut() {
            let _ = stdout.write_fmt(args);
        }
    })
}

/// Macro for printing to the serial standard output
#[macro_export]
macro_rules! sprint {
    ($s:expr) => {
        $crate::debug::write_str($s)
    };
    ($($tt:tt)*) => {
        $crate::debug::write_fmt(format_args!($($tt)*))
    };
}

/// Macro for printing to the serial standard output, with a newline.
#[macro_export]
macro_rules! sprintln {
    () => {
        $crate::debug::write_str("\n")
    };
    ($s:expr) => {
        $crate::debug::write_str(concat!($s, "\n"))
    };
    ($s:expr, $($tt:tt)*) => {
        $crate::debug::write_fmt(format_args!(concat!($s, "\n"), $($tt)*))
    };
}
