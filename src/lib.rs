//! llamastash library — TUI + daemon for managing local llama.cpp
//! servers. The binary at `src/main.rs` is a thin wrapper around the
//! modules exposed here.

#![warn(rust_2018_idioms)]
#![deny(clippy::shadow_unrelated)]

pub mod backend;
pub mod banner;
pub mod config;
pub mod daemon;
pub mod discovery;
pub mod gguf;
pub mod gpu;
pub mod ipc;
pub mod launch;
pub mod proxy;
#[cfg(any(test, feature = "test-fixtures"))]
#[doc(hidden)]
pub mod test_support;
pub mod theme;
pub mod tui;
pub mod util;

#[cfg(any(test, feature = "test-fixtures"))]
pub mod test_lock {
  use std::sync::{Mutex, MutexGuard, OnceLock};

  pub fn serialize() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK
      .get_or_init(|| Mutex::new(()))
      .lock()
      .unwrap_or_else(|poison| poison.into_inner())
  }
}
