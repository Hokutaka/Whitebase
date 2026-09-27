use whitebase_backend_contract::{
    BackendCapabilities, BackendKind, ComputeBackend, ComputeError, OperationKind,
};

use whitebase_cerune_vm_adapter::{CeruneVmAdapter, CeruneVmAdapterError};

#[cfg(not(target_arch = "wasm32"))]
use whitebase_cerune_c_adapter::{CeruneCAdapter, CeruneCAdapterError};

#[cfg(not(target_arch = "wasm32"))]
use whitebase_cerune_llvm_adapter::{CeruneLlvmAdapter, CeruneLlvmAdapterError};

#[cfg(not(target_arch = "wasm32"))]
use whitebase_cerune_qbe_adapter::{CeruneQbeAdapter, CeruneQbeAdapterError};

#[cfg(not(target_arch = "wasm32"))]
use whitebase_cerune_wat_adapter::{CeruneWatAdapter, CeruneWatAdapterError};

use crate::backend_failure;

/// Cerune VMによる計算バックエンドです。
#[derive(Debug, Clone)]
pub struct CeruneVmBackend {
    adapter: Result<CeruneVmAdapter, CeruneVmAdapterError>,
}

impl CeruneVmBackend {
    /// Cerune VMの実行対象を準備します。
    ///
    /// コンパイルと関数解決はここで完了し、演算呼び出し中には行いません。
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: CeruneVmAdapter::new(),
        }
    }
}

impl Default for CeruneVmBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Cerune C artifactによる計算バックエンドです。
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct CeruneCBackend {
    adapter: Result<CeruneCAdapter, CeruneCAdapterError>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CeruneCBackend {
    /// Cerune C artifactの実行対象を準備します。
    ///
    /// C生成、native compilerによる共有ライブラリ生成、symbol解決は
    /// ここで完了し、演算呼び出し中には行いません。
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: CeruneCAdapter::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for CeruneCBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ComputeBackend for CeruneCBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CeruneC
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::empty().with_add_scalar_f64()
    }

    fn is_available(&self) -> bool {
        self.adapter.is_ok()
    }

    fn add_f32(&self, _lhs: &[f32], _rhs: &[f32], _output: &mut [f32]) -> Result<(), ComputeError> {
        Err(ComputeError::OperationUnsupported {
            backend: self.kind(),
            operation: OperationKind::AddF32,
        })
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        let adapter = self
            .adapter
            .as_ref()
            .map_err(|error| backend_failure(self.kind(), error))?;

        Ok(adapter.add_scalar_f64(lhs, rhs))
    }
}

/// Cerune LLVM artifactによる計算バックエンドです。
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct CeruneLlvmBackend {
    adapter: Result<CeruneLlvmAdapter, CeruneLlvmAdapterError>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CeruneLlvmBackend {
    /// Cerune LLVM artifactの実行対象を準備します。
    ///
    /// LLVM生成、Clang/LLDによる共有ライブラリ生成、symbol解決は
    /// ここで完了し、演算呼び出し中には行いません。
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: CeruneLlvmAdapter::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for CeruneLlvmBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ComputeBackend for CeruneLlvmBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CeruneLlvm
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::empty().with_add_scalar_f64()
    }

    fn is_available(&self) -> bool {
        self.adapter.is_ok()
    }

    fn add_f32(&self, _lhs: &[f32], _rhs: &[f32], _output: &mut [f32]) -> Result<(), ComputeError> {
        Err(ComputeError::OperationUnsupported {
            backend: self.kind(),
            operation: OperationKind::AddF32,
        })
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        let adapter = self
            .adapter
            .as_ref()
            .map_err(|error| backend_failure(self.kind(), error))?;

        Ok(adapter.add_scalar_f64(lhs, rhs))
    }
}

/// Cerune QBE artifactによる計算バックエンドです。
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct CeruneQbeBackend {
    adapter: Result<CeruneQbeAdapter, CeruneQbeAdapterError>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CeruneQbeBackend {
    /// Cerune QBE artifactの実行対象を準備します。
    ///
    /// QBE生成、native共有ライブラリ生成、関数解決はここで完了し、
    /// 演算呼び出し中には行いません。
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: CeruneQbeAdapter::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for CeruneQbeBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ComputeBackend for CeruneQbeBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CeruneQbe
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::empty().with_add_scalar_f64()
    }

    fn is_available(&self) -> bool {
        self.adapter.is_ok()
    }

    fn add_f32(&self, _lhs: &[f32], _rhs: &[f32], _output: &mut [f32]) -> Result<(), ComputeError> {
        Err(ComputeError::OperationUnsupported {
            backend: self.kind(),
            operation: OperationKind::AddF32,
        })
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        let adapter = self
            .adapter
            .as_ref()
            .map_err(|error| backend_failure(self.kind(), error))?;

        Ok(adapter.add_scalar_f64(lhs, rhs))
    }
}

/// Cerune WAT artifactによる計算バックエンドです。
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct CeruneWatBackend {
    adapter: Result<CeruneWatAdapter, CeruneWatAdapterError>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CeruneWatBackend {
    /// Cerune WAT artifactの実行対象を準備します。
    ///
    /// WAT生成、Wasm module準備、関数解決はここで完了し、
    /// 演算呼び出し中には行いません。
    #[must_use]
    pub fn new() -> Self {
        Self {
            adapter: CeruneWatAdapter::new(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for CeruneWatBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ComputeBackend for CeruneWatBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CeruneWat
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::empty().with_add_scalar_f64()
    }

    fn is_available(&self) -> bool {
        self.adapter.is_ok()
    }

    fn add_f32(&self, _lhs: &[f32], _rhs: &[f32], _output: &mut [f32]) -> Result<(), ComputeError> {
        Err(ComputeError::OperationUnsupported {
            backend: self.kind(),
            operation: OperationKind::AddF32,
        })
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        let adapter = self
            .adapter
            .as_ref()
            .map_err(|error| backend_failure(self.kind(), error))?;

        adapter
            .add_scalar_f64(lhs, rhs)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

impl ComputeBackend for CeruneVmBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CeruneVm
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::empty().with_add_scalar_f64()
    }

    fn is_available(&self) -> bool {
        self.adapter.is_ok()
    }

    fn add_f32(&self, _lhs: &[f32], _rhs: &[f32], _output: &mut [f32]) -> Result<(), ComputeError> {
        Err(ComputeError::OperationUnsupported {
            backend: self.kind(),
            operation: OperationKind::AddF32,
        })
    }

    fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, ComputeError> {
        let adapter = self
            .adapter
            .as_ref()
            .map_err(|error| backend_failure(self.kind(), error))?;

        adapter
            .add_scalar_f64(lhs, rhs)
            .map_err(|error| backend_failure(self.kind(), error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_cerune_vm_capabilities() {
        let backend = CeruneVmBackend::new();
        let capabilities = backend.capabilities();

        assert!(backend.is_available());
        assert!(capabilities.supports(OperationKind::AddScalarF64));
        assert!(!capabilities.supports(OperationKind::AddF32));
        assert!(!capabilities.supports(OperationKind::AddF64));
        assert!(!capabilities.supports(OperationKind::SumF64));
    }

    #[test]
    fn adds_f64_scalars_through_compute_backend() {
        let backend = CeruneVmBackend::new();

        let result = backend.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn reports_cerune_c_capabilities() {
        let backend = CeruneCBackend::new();
        let capabilities = backend.capabilities();

        assert!(backend.is_available());
        assert!(capabilities.supports(OperationKind::AddScalarF64));
        assert!(!capabilities.supports(OperationKind::AddF32));
        assert!(!capabilities.supports(OperationKind::AddF64));
        assert!(!capabilities.supports(OperationKind::SumF64));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn adds_f64_scalars_through_c_compute_backend() {
        let backend = CeruneCBackend::new();

        let result = backend.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn reports_cerune_llvm_capabilities() {
        let backend = CeruneLlvmBackend::new();
        let capabilities = backend.capabilities();

        assert!(backend.is_available());
        assert!(capabilities.supports(OperationKind::AddScalarF64));
        assert!(!capabilities.supports(OperationKind::AddF32));
        assert!(!capabilities.supports(OperationKind::AddF64));
        assert!(!capabilities.supports(OperationKind::SumF64));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn adds_f64_scalars_through_llvm_compute_backend() {
        let backend = CeruneLlvmBackend::new();

        let result = backend.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn reports_cerune_qbe_capabilities() {
        let backend = CeruneQbeBackend::new();
        let capabilities = backend.capabilities();

        assert!(capabilities.supports(OperationKind::AddScalarF64));
        assert!(!capabilities.supports(OperationKind::AddF32));
        assert!(!capabilities.supports(OperationKind::AddF64));
        assert!(!capabilities.supports(OperationKind::SumF64));
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    #[test]
    fn adds_f64_scalars_through_cerune_qbe_backend() {
        let backend = CeruneQbeBackend::new();

        if !backend.is_available() {
            return;
        }

        let result = backend.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn reports_cerune_wat_capabilities() {
        let backend = CeruneWatBackend::new();
        let capabilities = backend.capabilities();

        assert!(backend.is_available());
        assert!(capabilities.supports(OperationKind::AddScalarF64));
        assert!(!capabilities.supports(OperationKind::AddF32));
        assert!(!capabilities.supports(OperationKind::AddF64));
        assert!(!capabilities.supports(OperationKind::SumF64));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn adds_f64_scalars_through_cerune_wat_backend() {
        let backend = CeruneWatBackend::new();

        let result = backend.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }
}
