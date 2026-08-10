use std::error::Error;
use std::fmt;

use whitebase_core::ComputeError;
use whitebase_runner::RunnerError;

#[derive(Debug)]
pub enum InterfaceError {
    InvalidRequest { code: &'static str, message: String },
    Internal { code: &'static str, message: String },
}

impl InterfaceError {
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidRequest { code, .. } | Self::Internal { code, .. } => code,
        }
    }

    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::InvalidRequest { message, .. } | Self::Internal { message, .. } => message,
        }
    }

    #[must_use]
    pub fn is_invalid_request(&self) -> bool {
        matches!(self, Self::InvalidRequest { .. })
    }
}

impl fmt::Display for InterfaceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl Error for InterfaceError {}

pub(crate) fn benchmark_error(error: RunnerError) -> InterfaceError {
    let invalid_request = match &error {
        RunnerError::ZeroInputLength
        | RunnerError::InputLengthTooLarge { .. }
        | RunnerError::WarmupIterationsTooLarge { .. }
        | RunnerError::MeasuredIterationsTooLarge { .. }
        | RunnerError::ZeroMeasuredIterations
        | RunnerError::BenchmarkWorkloadTooLarge { .. }
        | RunnerError::SumF64RequiresF64 => true,

        RunnerError::NoBackends
        | RunnerError::InvalidAbsoluteTolerance { .. }
        | RunnerError::ReferenceBackendUnavailable { .. }
        | RunnerError::InvalidScalarF64Input { .. }
        | RunnerError::ScalarF64ReferenceOutOfRange { .. }
        | RunnerError::Compute { .. } => false,
    };

    if invalid_request {
        InterfaceError::InvalidRequest {
            code: "invalid_benchmark_request",
            message: error.to_string(),
        }
    } else {
        InterfaceError::Internal {
            code: "runner_failed",
            message: error.to_string(),
        }
    }
}

pub(crate) fn scalar_f64_error(error: RunnerError) -> InterfaceError {
    let invalid_request = match &error {
        RunnerError::InvalidScalarF64Input { .. }
        | RunnerError::ScalarF64ReferenceOutOfRange { .. } => true,

        RunnerError::NoBackends
        | RunnerError::ZeroMeasuredIterations
        | RunnerError::InvalidAbsoluteTolerance { .. }
        | RunnerError::ReferenceBackendUnavailable { .. }
        | RunnerError::Compute { .. }
        | RunnerError::ZeroInputLength
        | RunnerError::InputLengthTooLarge { .. }
        | RunnerError::WarmupIterationsTooLarge { .. }
        | RunnerError::MeasuredIterationsTooLarge { .. }
        | RunnerError::SumF64RequiresF64
        | RunnerError::BenchmarkWorkloadTooLarge { .. } => false,
    };

    if invalid_request {
        InterfaceError::InvalidRequest {
            code: "invalid_scalar_f64_request",
            message: error.to_string(),
        }
    } else {
        InterfaceError::Internal {
            code: "scalar_f64_observation_failed",
            message: error.to_string(),
        }
    }
}

pub(crate) fn compute_error(error: ComputeError) -> InterfaceError {
    match &error {
        ComputeError::LengthMismatch { .. } => InterfaceError::InvalidRequest {
            code: "invalid_compute_request",
            message: error.to_string(),
        },

        ComputeError::BackendNotRegistered { .. }
        | ComputeError::BackendUnavailable { .. }
        | ComputeError::OperationUnsupported { .. }
        | ComputeError::BackendFailure { .. } => InterfaceError::Internal {
            code: "compute_failed",
            message: error.to_string(),
        },
    }
}
