use whitebase_interface::{BackendCapabilities, BackendKind, ComputeBackend, ComputeError};

use crate::backend_failure;

fn unavailable(backend: BackendKind) -> ComputeError {
    ComputeError::BackendUnavailable { backend }
}

/// Windows GNU環境のGCCによるScalar計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsGnuCppScalarBackend;

impl ComputeBackend for WindowsGnuCppScalarBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::WindowsGnuCppScalar
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::scalar_add_f32()
            .with_add_f64(1)
            .with_add_scalar_f64()
            .with_sum_f64()
    }

    fn is_available(&self) -> bool {
        whitebase_windows_gnu_adapter::is_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::cpp_add_f32_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }

    fn add_f64(&self, lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::cpp_add_f64_array_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::cpp_add_f64_scalar(lhs, rhs)
            .map_err(|error| backend_failure(self.kind(), error))
    }
    fn sum_f64(&self, input: &[f64]) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::cpp_sum_f64_scalar(input)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

/// Windows GNU環境のGCCによるAVX計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsGnuCppAvxBackend;

impl ComputeBackend for WindowsGnuCppAvxBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::WindowsGnuCppAvx
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::simd_add_f32(8)
            .with_add_f64(4)
            .with_sum_f64()
    }

    fn is_available(&self) -> bool {
        whitebase_windows_gnu_adapter::is_cpp_avx_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        let executed = whitebase_windows_gnu_adapter::cpp_add_f32_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(unavailable(self.kind()));
        }

        Ok(())
    }

    fn add_f64(&self, lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        let executed = whitebase_windows_gnu_adapter::cpp_add_f64_array_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(unavailable(self.kind()));
        }

        Ok(())
    }
    fn sum_f64(&self, input: &[f64]) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        let result = whitebase_windows_gnu_adapter::cpp_sum_f64_avx(input)
            .map_err(|error| backend_failure(self.kind(), error))?;

        result.ok_or_else(|| unavailable(self.kind()))
    }
}

/// Windows GNU環境のNASMによるScalar計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsGnuAssemblyScalarBackend;

impl ComputeBackend for WindowsGnuAssemblyScalarBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::WindowsGnuAssemblyScalar
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::scalar_add_f32()
            .with_add_f64(1)
            .with_add_scalar_f64()
            .with_sum_f64()
    }

    fn is_available(&self) -> bool {
        whitebase_windows_gnu_adapter::is_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::assembly_add_f32_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }

    fn add_f64(&self, lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::assembly_add_f64_array_scalar(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::assembly_add_f64_scalar(lhs, rhs)
            .map_err(|error| backend_failure(self.kind(), error))
    }
    fn sum_f64(&self, input: &[f64]) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        whitebase_windows_gnu_adapter::assembly_sum_f64_scalar(input)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

/// Windows GNU環境のNASMによるAVX計算バックエンドです。
#[derive(Debug, Clone, Copy, Default)]
pub struct WindowsGnuAssemblyAvxBackend;

impl ComputeBackend for WindowsGnuAssemblyAvxBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::WindowsGnuAssemblyAvx
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::simd_add_f32(8)
            .with_add_f64(4)
            .with_sum_f64()
    }

    fn is_available(&self) -> bool {
        whitebase_windows_gnu_adapter::is_assembly_avx_available()
    }

    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        let executed = whitebase_windows_gnu_adapter::assembly_add_f32_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(unavailable(self.kind()));
        }

        Ok(())
    }

    fn add_f64(&self, lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ComputeError> {
        ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        let executed = whitebase_windows_gnu_adapter::assembly_add_f64_array_avx(lhs, rhs, output)
            .map_err(|error| backend_failure(self.kind(), error))?;

        if !executed {
            return Err(unavailable(self.kind()));
        }

        Ok(())
    }
    fn sum_f64(&self, input: &[f64]) -> Result<f64, ComputeError> {
        if !self.is_available() {
            return Err(unavailable(self.kind()));
        }

        let result = whitebase_windows_gnu_adapter::assembly_sum_f64_avx(input)
            .map_err(|error| backend_failure(self.kind(), error))?;

        result.ok_or_else(|| unavailable(self.kind()))
    }
}
