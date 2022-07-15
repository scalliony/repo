#include "nostdlib.h"

[[noreturn]] inline void abort2() { __builtin_unreachable(); }
void *operator new(size_t) { abort2(); }
void *operator new[](size_t) { abort2(); }
void operator delete(void *) noexcept { abort(); }
void operator delete[](void *) noexcept { abort(); }
void *__cxa_pure_virtual = nullptr;

namespace std {

class string_view {
private:
    const char* dt;
    size_t len;

public:
    string_view(const char* ref, size_t size) noexcept: dt(ref), len(size) { }
    string_view(const char* ref) noexcept: string_view(ref, strlen(ref)) { }

    constexpr const char *c_str() const noexcept { return dt; }
    constexpr size_t size() const noexcept { return len; }
};


template <typename T>
class optional {
private:
    struct none_t {};
    union {
        none_t none;
        T value;
    };

    bool has_data = false;

public:
    constexpr optional() noexcept: none(), has_data(false) { }
    constexpr optional(const T& v): value(v), has_data(true) { }

    constexpr bool has_value() const { return has_data; }

    constexpr explicit operator bool() const noexcept { return has_data; }

    constexpr T &get() & { return value; }
    constexpr const T &get() const & { return value; }

    constexpr T &operator*() & { return value; }
    constexpr const T &operator*() const & { return value; }
};

}