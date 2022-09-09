default:
  @just --list --unsorted

# Run client
play *args: (client "run -F offline" args)
# Exec task on client
client *args:
  @just --justfile {{justfile_directory()}}/client/justfile {{args}}

build-hello-wasi:
  cargo +nightly build -p hello-wasi --release --target wasm32-wasi -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort
build-wasm +args:
  cargo build --release --target wasm32-unknown-unknown -p {{args}}
