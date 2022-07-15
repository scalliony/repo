/// Called at boot-time (optional)
fn main() {
    println!("Starting");
}

/// Called at each tick (required)
#[no_mangle]
pub extern "C" fn tick() {
    println!("Ticking");
    scalliony_api::motor::right();
}
