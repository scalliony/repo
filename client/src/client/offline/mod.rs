#[cfg_attr(not(feature = "threaded"), path = "pulled.rs")]
#[cfg_attr(all(feature = "threaded", target_arch = "wasm32"), path = "worker.rs")]
#[cfg_attr(all(feature = "threaded", not(target_arch = "wasm32")), path = "thread.rs")]
mod inner;
pub use inner::Client;
