#include "raw.h"
#include "nostdlib.h"
#include <stdbool.h>

struct str_t {
    const char *data;
    uint32_t size;
};
/// Write string to logs
inline void io_log(const char *str) {
    _io_log(str, strlen(str));
}
inline void io_log_n(const struct str_t *str) {
    _io_log(str->data, str->size);
}

inline int puts(const char* str) {
    io_log(str);
    return 0;
}
inline int puts_n(const char* str, size_t len) {
    _io_log(str, len);
    return 0;
}


inline void motor_rotate(bool left) {
    _motor_rotate(left ? -1 : 1);
}
inline void motor_rotate_left() {
    motor_rotate(true);
}
inline void motor_rotate_right() {
    motor_rotate(false);
}

/// Move forward of dist cells
/// Direction depends of current rotation
/// Actual movement is delayed
inline void motor_move(uint16_t dist) {
    _motor_move(dist);
}

enum entity_type: int32_t {
    Unexpected = -1,
    Rock, Bot, Building
};
struct entity_t {
    int64_t id;
    enum entity_type type;
};
enum rotation: int32_t {
    Up = 0, Right, Down, Left
};

/// Check of returned entity is valid
inline bool is_valid_entity(struct entity_t e) {
    return e.id > 0 && e.type != Unexpected;
}

/// Check for entity just in front (depending of rotation)
/// Returns valid entity if something is in contact
/// Must check validity with bool is_valid_entity(struct entity_t)
inline struct entity_t sensors_contact() {
    struct i64_32_t res = _sensors_contact();
    return *(struct entity_t*)&res;
}
