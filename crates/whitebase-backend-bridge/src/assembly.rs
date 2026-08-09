use crate::backend_failure;

use whitebase_backend_contract::{BackendCapabilities, BackendKind, ComputeBackend, ComputeError};

/// AssemblyによるScalar計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct AssemblyScalarBackend;

impl ComputeBackend for AssemblyScalarBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::AssemblyScalar
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::scalar_add_f32()
            .with_add_f64(1)
            .with_add_scalar_f64()
            .with_sum_f64()
    }

    fn is_available(&self) -> bool {
        true
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        whitebase_asm_adapter::add_f32_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }

    fn add_f64(&self, lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        whitebase_asm_adapter::add_f64_array_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        Ok(whitebase_asm_adapter::add_f64_scalar(lhs, rhs))
    }

    fn sum_f64(&self, input: &[f64]) -> Result<f64, ComputeError> {
        Ok(whitebase_asm_adapter::sum_f64_scalar(input))
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
        BackendCapabilities::simd_add_f32(8)
            .with_add_f64(4)
            .with_sum_f64()
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

    fn add_f64(&self, lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        let executed = whitebase_asm_adapter::add_f64_array_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        Ok(())
    }

    fn sum_f64(&self, input: &[f64]) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(ComputeError::BackendUnavailable {
                backend: self.kind(),
            });
        }

        whitebase_asm_adapter::sum_f64_avx(input).ok_or(ComputeError::BackendUnavailable {
            backend: self.kind(),
        })
    }
}
