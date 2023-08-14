//! Futures-based test app. Blinks a light. While a button is pressed, the
//! blinking is suspended. Note that golf2 doesn't implement the LED or button
//! syscall drivers, so these are all just GPIO calls.

#![no_std]
#![allow(internal_features)]  // TODO
#![feature(lang_items, naked_functions)]

use common::*;

mod alarm;
mod app;
mod gpio;
mod task;
mod tock_static;

fn main() {
    alarm::init();
    gpio::start();
    app::APP.start();

    loop {
        syscalls::yieldk();
    }
}
