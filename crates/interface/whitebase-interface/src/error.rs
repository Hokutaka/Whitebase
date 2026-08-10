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
    let invalid_request = matches!(
        &error,
        RunnerError::ZeroInputLength
            | RunnerError::InputLengthTooLarge { .. }
            | RunnerError::WarmupIterationsTooLarge { .. }
            | RunnerError::MeasuredIterationsTooLarge { .. }
            | RunnerError::ZeroMeasuredIterations
            | RunnerError::BenchmarkWorkloadTooLarge { .. }
            | RunnerError::SumF64RequiresF64
    );

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
    let invalid_request = matches!(
        &error,
        RunnerError::InvalidScalarF64Input { .. }
            | RunnerError::ScalarF64ReferenceOutOfRange { .. }
    );

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
    InterfaceError::Internal {
        code: "compute_failed",
        message: error.to_string(),
    }
}
