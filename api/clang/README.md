# C/C++

Just simple C and C++ bindings using some nice features of clang

## Content

- [lib.c](./lib.c): Stupid traveler C
- [lib.cpp](./lib.cpp): Stupid traveler C++
- include/
  - [api.h](./include/api.h): Scalliony API C bindning
  - [api.hpp](./include/api.h): Scalliony API C++ bindning
  - [raw.h](./include/raw.h): Scalliony API extern functions
  - [nostdlib.h](./include/nostdlib.hpp): A minimal libc
  - [nostdlib.hpp](./include/nostdlib.hpp): A minimal std
  - [print.h](./include/printf.h): Tiny *prints implementations for embedded systems

## Setup

1. Copy this folder
2. Install clang and wasm-ld
3. Run `./make.sh lib.c # or lib.cpp`
4. Upload `out.wasm`

# No libc

Scalliony does not provide libc but just a minimal part of it, so your code can not use neither c nor c++ dynamic standard libraries
