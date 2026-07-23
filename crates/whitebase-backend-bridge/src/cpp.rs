use whitebase_interface::{BackendCapabilities, BackendKind, ComputeBackend, ComputeError};

use crate::backend_failure;

/// C++によるScalar計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct CppScalarBackend;

impl ComputeBackend for CppScalarBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CppScalar
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::scalar_add_f32()
    }

    fn is_available(&self) -> bool {
        true
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        whitebase_cpp_adapter::add_f32_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

/// C++によるAVX計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct CppAvxBackend;

impl ComputeBackend for CppAvxBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CppAvx
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::avx_add_f32()
    }

    fn is_available(&self) -> bool {
        whitebase_cpp_adapter::is_avx_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        let executed = whitebase_cpp_adapter::add_f32_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        Ok(())
    }
}
