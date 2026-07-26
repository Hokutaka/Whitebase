//! Whitebaseの共通インターフェースと各計算実装を接続します。
#![forbid(unsafe_code)]

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
mod assembly;

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
mod cpp;

mod rust;

#[cfg(not(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc")))]
mod unavailable;

use std::fmt;

use whitebase_interface::{BackendKind, ComputeError};

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
pub use assembly::{AssemblyAvxBackend, AssemblyScalarBackend};

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
pub use cpp::{CppAvxBackend, CppScalarBackend};

pub use rust::{RustScalarBackend, RustSimdBackend};

#[cfg(not(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc")))]
pub use unavailable::{AssemblyAvxBackend, AssemblyScalarBackend, CppAvxBackend, CppScalarBackend};

pub(crate) fn backend_failure(backend: BackendKind, error: impl fmt::Display) -> ComputeError {
    ComputeError::BackendFailure {
        backend,
        message: error.to_string(),
    }
}
