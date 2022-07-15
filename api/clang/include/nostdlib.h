#include <stddef.h>

void abort() {
	__builtin_unreachable();
}

int puts_n(const char *str, size_t len);

/* TODO:
void* malloc(size_t amount);
void* realloc(void *ptr, size_t size);
void* calloc(size_t num, size_t size);
void free(void* mem);
*/

size_t strlen(const char *str) {
    const char *s;
    for (s = str; *s; ++s);
    return (s - str);
}

void* memcpy(void* dest, const void* src, size_t count) {
    for (size_t i = 0; i < count; ++i) {
        ((char*) dest)[i] = ((const char*) src)[i];
    }
    return dest;
}

void* memset(void * dest, int value, size_t count) {
    for (size_t i = 0; i < count; ++i) {
        ((unsigned char*) dest)[i] = (unsigned char) value;
    }
    return dest;
}

const int line_buffer_size = 100;
char line_buffer[line_buffer_size];
int line_buffer_pos = 0;
void _putchar(char ch) {
	if (ch == '\n') {
		line_buffer[line_buffer_pos] = 0;
		puts_n(line_buffer, line_buffer_pos);
		line_buffer_pos = 0;
	} else if (line_buffer_pos < line_buffer_size - 1) {
		line_buffer[line_buffer_pos++] = ch;
	}

	if (line_buffer_pos == line_buffer_size - 1) {
		_putchar('\n');
	}
}

#include "printf.h"