use crate::backend_failure;

use whitebase_interface::{BackendCapabilities, BackendKind, ComputeBackend, ComputeError};

/// AssemblyによるScalar計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct AssemblyScalarBackend;

impl ComputeBackend for AssemblyScalarBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::AssemblyScalar
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::scalar_add_f32()
    }

    fn is_available(&self) -> bool {
        true
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        whitebase_asm_adapter::add_f32_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

/// AssemblyによるAVX計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct AssemblyAvxBackend;

impl ComputeBackend for AssemblyAvxBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::AssemblyAvx
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::avx_add_f32()
    }

    fn is_available(&self) -> bool {
        whitebase_asm_adapter::is_avx_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        let executed = whitebase_asm_adapter::add_f32_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        Ok(())
    }
}
