# Rust

Just a simple rust binding

Use the [rust-template](https://github.com/scalliony/rust-template) for a basic ready to go AI

## Content

- [api](./api): Scalliony API Rust bindning
- [hello](./hello/src/lib.rs): Simple `Hello world`
- [explorer](./explorer/src/lib.rs): Stupid traveler

## Shrink size

Executable file size is a big part of the bot startup cost.

Always use `--release`, it won't run with `RUST_BACKTRACE=1` anyway. Keep binary small with `[profile.release] lto = "thin"`.

Using `wasm32-unknown-unknown` target is the preferred choice but if you release need full std, you can use `wasm32-wasi` with [hello-wasi](./hello-wasi) project. By default backtrace will be huge, it can be disabled with `cargo +nightly build -p hello-wasi --release --target wasm32-wasi -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort`
