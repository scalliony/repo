#include "api.hpp"

/// Called at boot-time (optional)
extern "C" void _start() {
    io::log("Starting");
}

/// Called at each tick (required)
extern "C" void tick() {
    if (const auto front = sensors::contact()) {
        motor::rotate_left();
    }
    motor::move(3);
}