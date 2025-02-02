#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::{prelude::*, Timer},
};

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let show_time = 100_u32;
    let dark_time = 100_u32;
    let mut display = Display::new(board.display_pins);
    let light_it_all = [
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
        [1, 1, 1, 1, 1],
    ];
    // Show light_it_all for 1000ms just because
    display.show(&mut timer, light_it_all, 1000_u32);
    // clear the display
    display.clear();
    timer.delay_ms(1000_u32);

    const ROW_COUNT:usize = 5;
    const COL_COUNT:usize = 5;
    let mut image_matrix = [[0;COL_COUNT];ROW_COUNT];
    let light_sequence = [[0,0],[0,1],[0,2],[0,3],[0,4],[1,4],[2,4],[3,4],[4,4],[4,3],[4,2],[4,1],[4,0],[3,0],[2,0],[1,0]];

    loop {
        for [row,col] in light_sequence {
            // Show the current matrix
            image_matrix[row][col] = 1;
            display.show(&mut timer, image_matrix, show_time);
            // clear the matrix and display again
            image_matrix[row][col] = 0;
            display.clear();
            timer.delay_ms(dark_time);
        }  
    }
}