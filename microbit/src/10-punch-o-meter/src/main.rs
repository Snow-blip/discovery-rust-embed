#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;

#[cfg(feature = "v1")]
use microbit::{
    hal::twi,
    pac::twi0::frequency::FREQUENCY_A,
    hal::{prelude::*, Timer}
};

#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
    hal::{prelude::*, Timer}
};

use lsm303agr::{
    AccelOutputDataRate, Lsm303agr, AccelScale
};

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    // create i2c bus?
    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    rprintln!("i2c bus created");

    // setup sensor connection
    // requires resetting the microbit
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_accel_scale(AccelScale::G16);
    rprintln!("Sensor connected");

    let mut timer = Timer::new(board.TIMER0).into_periodic();

    loop {
        // By default, the app is not "observing" the acceleration of the board.
        // When a significant X acceleration is detected (i.e. the acceleration goes above some threshold), the app should start a new measurement.
        // During that measurement interval, the app should keep track of the maximum acceleration observed
        // After the measurement interval ends, the app must report the maximum acceleration observed. You can report the value using the rprintln! macro.

        let mut x_acc = sensor.accel_data().unwrap().x.abs();
        if (x_acc >= 1000) {
            rprintln!("Start measurement");
            timer.start(1000000_u32); // documentation says frequency is 1 MHz and argument is cycle count. so... 1e6 would be 1 second?
            let mut max_acc = x_acc;
            loop {
                match timer.wait() {
                    Ok(()) => {
                        rprintln!("Max acceleration is {} mg.",max_acc);
                        break;
                    },
                    Err(_) => {
                        x_acc = sensor.accel_data().unwrap().x.abs();
                        if x_acc>max_acc {
                            max_acc = x_acc;
                        }
                    },
                }
            }
        }
    }
}
