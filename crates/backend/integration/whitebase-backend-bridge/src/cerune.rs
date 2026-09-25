use whitebase_backend_contract::{
    BackendCapabilities, BackendKind, ComputeBackend, ComputeError, OperationKind,
};
use whitebase_cerune_vm_adapter::{CeruneVmAdapter, CeruneVmAdapterError};

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
}
