default:
  @just --list --unsorted

# Run client
play *ARGS:
  @just play-dev -r {{ARGS}}
# Run client (dev)
play-dev *ARGS:
  cargo run -p scalliony-client {{ARGS}}
# Build client
build *ARGS: 
  @just build-dev -r {{ARGS}}
# Build client (dev)
build-dev *ARGS:
  cargo build -p scalliony-client {{ARGS}}

# Run client (wasm) in browser
play-web *ARGS: copy-index
  @test -f client/dist/bundle.js || just bundle-js  
  @just build-web {{ARGS}}
  basic-http-server client/dist
# Build client (wasm)
build-web *ARGS: 
  @just build --target wasm32-unknown-unknown {{ARGS}}
  wasm-opt -Os --strip target/wasm32-unknown-unknown/release/scalliony-client.wasm -o client/dist/scalliony.wasm
# Build client (wasm, no backtrace)
build-web-abort:
  cargo +nightly build -p scalliony-client -r --target wasm32-unknown-unknown -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort
  @cp target/wasm32-unknown-unknown/release/scalliony-client.wasm client/dist/scalliony.wasm
# Build client (wasm, dev)
build-web-dev: 
  @just build-dev --target wasm32-unknown-unknown
  @cp target/wasm32-unknown-unknown/release/scalliony-client.wasm -o client/dist/scalliony.wasm

# Run client (.exe, dev) usefull in wsl
play-win-dev *ARGS:
  @just play-dev --target x86_64-pc-windows-gnu {{ARGS}}

build-hello-wasi:
  cargo +nightly build -p hello-wasi --release --target wasm32-wasi -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort
build-wasm +ARGS:
  cargo build --release --target wasm32-unknown-unknown -p {{ARGS}}

@copy-index:
  mkdir -p client/dist
  cp client/src/index.html client/dist
bundle-js:
  wget -qO- https://raw.githubusercontent.com/not-fl3/miniquad/master/js/gl.js > client/dist/bundle.js
  wget -qO- https://raw.githubusercontent.com/not-fl3/sapp-jsutils/master/js/sapp_jsutils.js >> client/dist/bundle.js
  wget -qO- https://raw.githubusercontent.com/not-fl3/quad-net/master/js/quad-net.js >> client/dist/bundle.js
  wget -qO- https://raw.githubusercontent.com/not-fl3/quad-snd/master/js/audio.js >> client/dist/bundle.js
  npx uglify-js client/dist/bundle.js -c -m -o client/dist/bundle.js
