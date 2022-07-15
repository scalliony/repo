# Scalliony API samples

Various ways to create scalliony compatible programs.

See README.md in each folders

## Languages

- [C/C++](./clang)
- [Rust](./rust)
- [AssemblyScript](./assemblyscript): A subset of TypeScript
- [WAT](./wat): Plain text assembler

## Tips

Wasm files can be optimized using [binaryen](https://github.com/webassembly/binaryen)'s `wasm-opt -O raw.wasm -o opt.wasm`

Wasm files can be converted back to *readable* [WAT](https://webassembly.github.io/spec/core/text/index.html) using [wabt](https://github.com/WebAssembly/wabt)'s `wasm2wat`

## API

The bot can interface with [WASI](https://wasi.dev/) (The WebAssembly System Interface) and a set of custom functions. Full description is available as [api.json](./api.json).
<!-- TODO: auto generate doc -->

A valid Bot must export a `void tick()` function called at every 'Game tick'

### Multi-value return

WebAssembly [multi-value proposal](https://github.com/WebAssembly/multi-value) defines a way to return a tuple of mixed type values from function calls.
This feature is fully supported by [wasmtime](https://wasmtime.dev/), the WASM runtime used by Scalliony. Sadly any compiler is well supported for now.

As fallback, *C-like* functions `_s` are temporary available and return original struct value at a user provider pointer address.

## License

All snippets are distributed under the MIT license to facilitate cooperation and knowledge sharing.
However, use with respect to contributors and end-users is strongly advised.