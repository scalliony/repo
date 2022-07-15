#include "raw.h"
#include "nostdlib.hpp"

namespace io {
    /// Write string to logs
    inline void log(const std::string_view &str) {
        _io_log(str.c_str(), str.size());
    }
}

namespace motor {
    inline void rotate(bool left) {
        _motor_rotate(left ? -1 : 1);
    }
    inline void rotate_left() {
        rotate(true);
    }
    inline void rotate_right() {
        rotate(false);
    }

    /// Move forward of dist cells
    /// Direction depends of current rotation
    /// Actual movement is delayed
    inline void move(uint16_t dist) {
        _motor_move(dist);
    }
}


enum class entity_type: int32_t {
    Rock, Bot, Building
};
struct entity_t {
    int64_t id;
    entity_type type;

    /// Check and return entity if valid
    static inline std::optional<entity_t> Of(i64_32_t raw) {
        if (raw.a > 0 && raw.b >= 0)
            return entity_t{raw.a, static_cast<entity_type>(raw.b)};
        else
            return {};
    }
};
enum class rotation: int32_t {
    Up = 0, Right, Down, Left
};

namespace sensors {
    /// Check for entity just in front (depending of rotation)
    /// Returns entity if something is in contact
    inline std::optional<entity_t> contact() {
        return entity_t::Of(_sensors_contact());
    }
}