//! Whitebaseの共通インターフェースと各計算実装を接続します。
#![forbid(unsafe_code)]

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
mod assembly;

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
mod cpp;

mod rust;

#[cfg(not(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
)))]
mod unavailable;

use std::fmt;

use whitebase_interface::{BackendKind, ComputeError};

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
pub use assembly::{AssemblyAvxBackend, AssemblyScalarBackend};

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
pub use cpp::{CppAvxBackend, CppScalarBackend};

pub use rust::{RustScalarBackend, RustSimdBackend};

#[cfg(not(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
)))]
pub use unavailable::{AssemblyAvxBackend, AssemblyScalarBackend, CppAvxBackend, CppScalarBackend};

pub(crate) fn backend_failure(backend: BackendKind, error: impl fmt::Display) -> ComputeError {
    ComputeError::BackendFailure {
        backend,
        message: error.to_string(),
    }
}
