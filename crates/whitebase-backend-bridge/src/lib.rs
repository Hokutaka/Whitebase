//! Whitebaseの共通インターフェースと各計算実装を接続します。

#![forbid(unsafe_code)]

mod assembly;
mod cpp;
mod rust;

use std::fmt;

use whitebase_interface::{BackendKind, ComputeError};

pub use assembly::{AssemblyAvxBackend, AssemblyScalarBackend};

pub use cpp::{CppAvxBackend, CppScalarBackend};

pub use rust::{RustScalarBackend, RustSimdBackend};

pub(crate) fn backend_failure(backend: BackendKind, error: impl fmt::Display) -> ComputeError {
    ComputeError::BackendFailure {
        backend,
        message: error.to_string(),
    }
}
