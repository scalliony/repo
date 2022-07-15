#include "api.h"

/// Called at boot-time (optional)
void _start() {
    io_log("Starting");
}

/// Called at each tick (required)
void tick() {
    struct entity_t front = sensors_contact();
    if (is_valid_entity(front)) {
        motor_rotate_left();
    } else {
        //Formatting string at runtime can be expensive
        //printf("Collide %lld", front.id);
    }
    motor_move(3);
}