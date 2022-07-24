pub use scalliony_engine::*;

pub fn run() -> InterfaceRef {
    let paused = std::env::var("GAME_PAUSED")
        .map_or(false, |v| v.parse().unwrap_or_else(|_| v.parse::<u8>().unwrap() != 0));

    scalliony_engine::run(paused)
}
