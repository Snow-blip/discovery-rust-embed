#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

use cortex_m::register::basepri::write;
use embedded_hal_nb::serial;
use core::fmt::Write;
use heapless::Vec;
use core::str::from_utf8;

#[cfg(feature = "v1")]
use microbit::{
    hal::twi,
    pac::twi0::frequency::FREQUENCY_A,
    hal::prelude::*,
    hal::uart,
    hal::uart::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
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

use lsm303agr::{
    AccelOutputDataRate, Lsm303agr,
};

#[entry]
fn main() -> ! {
    // This is necessary
    rtt_init_print!();

    // take ownership of the board
    let board = microbit::Board::take().unwrap();

    // create i2c bus?
    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    rprintln!("i2c bus created");

    // setup serial communication through uart/uarte
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
    rprintln!("Serial communication created");

    // setup sensor connection
    // requires resetting the microbit
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    rprintln!("Sensor connected");

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
        let data = sensor.accel_data().unwrap();
        // serial print
        write!(serial,"Acceleration: x {} y {} z {}", data.x, data.y, data.z).unwrap();
        nb::block!(serial.flush()).unwrap()
    }
}