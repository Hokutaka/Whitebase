//! WhitebaseのTauri IPC Interface Adapterです。

#![forbid(unsafe_code)]

pub mod benchmark;
mod error;
pub mod scalar_f64;

pub use error::CommandError;
