#![no_main]
#![no_std]

use cortex_m::register::basepri::write;
use cortex_m_rt::entry;
use embedded_hal_nb::serial;
use core::fmt::Write;
use heapless::String;
use heapless::Vec;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;

#[cfg(feature = "v1")]
use microbit::{
    hal::prelude::*,
    hal::uart,
    hal::uart::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

#[cfg(feature = "v1")]
use embedded_io::Read;
#[cfg(feature = "v2")]
use embedded_hal_nb::serial::Read;

#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let mut serial = {
        uart::Uart::new(
            board.UART0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        )
    };

    #[cfg(feature = "v2")]
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 32> = Vec::new();

    loop {
        buffer.clear();
        let mut eol= false;
        while !eol {
            let read_byte=nb::block!(serial.read()).unwrap();
            if read_byte == 13 {
                eol=true;
            } else {
                match buffer.push(read_byte) {
                    Ok(_) => (),
                    Err(failed_byte) => {
                        write!(serial,"Buffer full, could not write {} to internal buffer. Clearing buffer.\n\r",failed_byte).unwrap();
                        buffer.clear();
                    },
                }
            }
        }
        write!(serial,"{}\n\r",buffer.iter().map(|&b| b as char).collect::<String<32>>());

        // TODO Send back the reversed string
    }
}