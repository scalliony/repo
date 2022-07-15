pub use anyhow::{anyhow as err_str, Result};
pub mod spec;

#[cfg_attr(target_arch = "wasm32", path = "wasm/mod.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "other/mod.rs")]
mod os;
pub use os::*;
