use whitebase_interface::{BackendCapabilities, BackendKind, ComputeBackend, ComputeError};

use crate::backend_failure;

/// RustによるScalar計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct RustScalarBackend;

impl ComputeBackend for RustScalarBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::RustScalar
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::scalar_add_f32()
    }

    fn is_available(&self) -> bool {
        true
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        whitebase_rust_backend::scalar::add_f32(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

/// RustによるAVX計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct RustSimdBackend;

impl ComputeBackend for RustSimdBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::RustSimd
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::avx_add_f32()
    }

    fn is_available(&self) -> bool {
        whitebase_rust_backend::simd::is_avx_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        whitebase_rust_backend::simd::add_f32(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}
