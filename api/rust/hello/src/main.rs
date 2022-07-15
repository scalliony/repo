#![no_main]
use scalliony_api::*;

/// Called at boot-time (optional)
#[no_mangle]
pub extern "C" fn _start() {
    io::log("Starting");
}

/// Called at each tick (required)
#[no_mangle]
pub extern "C" fn tick() {
    io::log("Ticking");
}
