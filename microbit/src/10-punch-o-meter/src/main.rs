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
    hal::{prelude::*, Timer},
    display::blocking::Display,
};

#[cfg(feature = "v2")]
use microbit::{
    hal::twim,
    pac::twim0::frequency::FREQUENCY_A,
    hal::{prelude::*, Timer},
    display::blocking::Display,
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

    
    let mut timer_led = Timer::new(board.TIMER1);
    let mut display = Display::new(board.display_pins);
    const ROW_COUNT:usize = 5;
    const COL_COUNT:usize = 5;
    let mut image_matrix = [[0;COL_COUNT];ROW_COUNT];
    rprintln!("Display initialised");

    loop {
        // By default, the app is not "observing" the acceleration of the board.
        // When a significant X acceleration is detected (i.e. the acceleration goes above some threshold), the app should start a new measurement.
        // During that measurement interval, the app should keep track of the maximum acceleration observed
        // After the measurement interval ends, the app must report the maximum acceleration observed. You can report the value using the rprintln! macro.

        let mut x_acc = sensor.accel_data().unwrap().x.abs();
        if x_acc >= 1000 {
            rprintln!("Start measurement");
            display.clear();
            timer.start(1000000_u32); // documentation says frequency is 1 MHz and argument is cycle count. so... 1e6 would be 1 second
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
            // light up the led matrix for a second to show results
            // max is 16, we have 25 leds. I want to always light up at 
            // least one to show it's working, but that's not an issue
            // since this only runs if we have at least 1 g acceleration
            let mut score = max_acc*25/16/1000;
            rprintln!("Score: {}",score);
            'outer: for row in 0..5 {
                for col in 0..5 {
                    if score > 0 {
                        image_matrix[row][col] = 1;
                        score-=1;
                    } else {
                        break 'outer;
                    }
                }
            }
            display.show(&mut timer_led, image_matrix,2000_u32);
            image_matrix = [[0;COL_COUNT];ROW_COUNT];
        }
    }
}
