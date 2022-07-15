#include <stdint.h>

struct i64_32_t {
    int64_t a;
    int32_t b;
};

void _io_log(const char *s, int32_t l) __attribute__((
    __import_module__("io"),
    __import_name__("log")));

void _motor_rotate(int32_t left) __attribute__((
    __import_module__("motor"),
    __import_name__("rotate")
));
void _motor_move(int32_t dist) __attribute__((
    __import_module__("motor"),
    __import_name__("move")
));

struct i64_32_t _sensors_contact() __attribute__((
    __import_module__("sensors"),
    __import_name__("contact_s")
));
