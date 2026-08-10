use whitebase_core::Whitebase;

use crate::error::{InterfaceError, compute_error};

pub use whitebase_core::BackendKind;

/// Whitebase CoreのPure ComputeをApplication Boundaryから公開します。
pub struct ComputeInterface {
    whitebase: Whitebase,
}

impl ComputeInterface {
    #[must_use]
    pub fn new() -> Self {
        Self {
            whitebase: Whitebase::new(),
        }
    }

    pub fn add_scalar_f64(
        &self,
        backend: BackendKind,
        lhs: f64,
        rhs: f64,
    ) -> Result<f64, InterfaceError> {
        self.whitebase
            .add_scalar_f64(backend, lhs, rhs)
            .map_err(compute_error)
    }

    pub fn add_f32(
        &self,
        backend: BackendKind,
        lhs: &[f32],
        rhs: &[f32],
    ) -> Result<Vec<f32>, InterfaceError> {
        let mut output = vec![0.0; lhs.len()];

        self.whitebase
            .add_f32(backend, lhs, rhs, &mut output)
            .map_err(compute_error)?;

        Ok(output)
    }

    pub fn add_f64(
        &self,
        backend: BackendKind,
        lhs: &[f64],
        rhs: &[f64],
    ) -> Result<Vec<f64>, InterfaceError> {
        let mut output = vec![0.0; lhs.len()];

        self.whitebase
            .add_f64(backend, lhs, rhs, &mut output)
            .map_err(compute_error)?;

        Ok(output)
    }

    pub fn sum_f64(&self, backend: BackendKind, input: &[f64]) -> Result<f64, InterfaceError> {
        self.whitebase
            .sum_f64(backend, input)
            .map_err(compute_error)
    }
}

impl Default for ComputeInterface {
    fn default() -> Self {
        Self::new()
    }
}
