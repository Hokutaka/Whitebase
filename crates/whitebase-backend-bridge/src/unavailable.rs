use whitebase_interface::{BackendCapabilities, BackendKind, ComputeBackend, ComputeError};

macro_rules! define_unavailable_backend {
    ($name:ident, $kind:expr, $capabilities:expr) => {
        #[derive(Debug, Clone, Copy, Default)]
        pub struct $name;

        impl ComputeBackend for $name {
            fn kind(&self) -> BackendKind {
                $kind
            }

            fn capabilities(&self) -> BackendCapabilities {
                $capabilities
            }

            fn is_available(&self) -> bool {
                false
            }

            fn add_f32(
                &self,
                lhs: &[f32],
                rhs: &[f32],
                output: &mut [f32],
            ) -> Result<(), ComputeError> {
                ComputeError::validate_lengths(lhs.len(), rhs.len(), output.len())?;

                Err(ComputeError::BackendUnavailable {
                    backend: self.kind(),
                })
            }
        }
    };
}

define_unavailable_backend!(
    CppScalarBackend,
    BackendKind::CppScalar,
    BackendCapabilities::scalar_add_f32().with_add_scalar_f64()
);

define_unavailable_backend!(
    CppAvxBackend,
    BackendKind::CppAvx,
    BackendCapabilities::avx_add_f32()
);

define_unavailable_backend!(
    AssemblyScalarBackend,
    BackendKind::AssemblyScalar,
    BackendCapabilities::scalar_add_f32()
);

define_unavailable_backend!(
    AssemblyAvxBackend,
    BackendKind::AssemblyAvx,
    BackendCapabilities::avx_add_f32()
);
