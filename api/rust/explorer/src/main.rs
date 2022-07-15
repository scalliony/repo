#![no_main]
use scalliony_api::*;

/// Called at boot-time (optional)
#[no_mangle]
pub extern "C" fn _start() {
    io::log("Explorer");
}

/// Called at each tick (required)
#[no_mangle]
pub extern "C" fn tick() {
    if sensors::contact().is_some() {
        motor::rotate_left();
    }
    motor::go_forward(3);
}
