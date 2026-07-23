//! Whitebaseの計算バックエンドに共通する契約を定義します.

#![forbid(unsafe_code)]

mod backend;
mod capabilities;
mod error;
mod operation;

pub use backend::{BackendKind, ComputeBackend};
pub use capabilities::BackendCapabilities;
pub use error::ComputeError;
pub use operation::OperationKind;

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyScalarBackend;

    impl ComputeBackend for DummyScalarBackend {
        fn kind(&self) -> BackendKind {
            BackendKind::RustScalar
        }

        fn capabilities(&self) -> BackendCapabilities {
            BackendCapabilities::scalar_add_f32()
        }

        fn is_available(&self) -> bool {
            true
        }

        fn add_f32(
            &self,
            lhs: &[f32],
            rhs: &[f32],
            output: &mut [f32],
        ) -> Result<(), ComputeError> {
            ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

            for ((lhs_value, rhs_value), output_value) in lhs.iter().zip(rhs).zip(output) {
                *output_value = lhs_value + rhs_value;
            }

            Ok(())
        }
    }

    #[test]
    fn backend_can_be_used_as_trait_object() {
        let backend: Box<dyn ComputeBackend> = Box::new(DummyScalarBackend);

        let lhs = [1.0, 2.0, 3.0];
        let rhs = [10.0, 20.0, 30.0];
        let mut output = [0.0; 3];

        backend.add_f32(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(backend.kind(), BackendKind::RustScalar);
        assert_eq!(output, [11.0, 22.0, 33.0]);
    }

    #[test]
    fn reports_length_mismatch() {
        let backend = DummyScalarBackend;

        let lhs = [1.0, 2.0];
        let rhs = [3.0];
        let mut output = [0.0; 2];

        assert_eq!(
            backend.add_f32(&lhs, &rhs, &mut output),
            Err(ComputeError::LengthMismatch {
                lhs_len: 2,
                rhs_len: 1,
                output_len: 2,
            })
        );
    }

    #[test]
    fn capabilities_report_supported_operation() {
        let capabilities = BackendCapabilities::avx_add_f32();

        assert!(capabilities.supports(OperationKind::AddF32));
        assert_eq!(capabilities.vector_width_f32, 8);
    }
}
