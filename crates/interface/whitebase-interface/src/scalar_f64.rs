use std::sync::OnceLock;

use crate::error::{InterfaceError, scalar_f64_error};
use serde::{Deserialize, Serialize};
use whitebase_runner::{
    F64Value, Runner, ScalarF64BackendObservation, ScalarF64BackendStatus,
    ScalarF64ObservationReport,
};

static RUNNER: OnceLock<Runner> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64Request {
    pub lhs: String,
    pub rhs: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct F64ValueDto {
    pub value: f64,
    pub decimal: String,
    pub bits: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScalarF64BackendResultStatus {
    Completed,
    Unavailable,
    Failed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64BackendResultDto {
    pub backend: String,
    pub status: ScalarF64BackendResultStatus,
    pub result: Option<F64ValueDto>,
    pub matches_reference_bits: Option<bool>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64ObservationDto {
    pub lhs_input: String,
    pub rhs_input: String,
    pub lhs: F64ValueDto,
    pub rhs: F64ValueDto,
    pub decimal_reference: String,
    pub reference: F64ValueDto,
    pub results: Vec<ScalarF64BackendResultDto>,
    pub all_backends_match: bool,
}

pub fn execute_scalar_f64_observation(
    request: ScalarF64Request,
) -> Result<ScalarF64ObservationDto, InterfaceError> {
    RUNNER
        .get_or_init(Runner::new)
        .observe_add_scalar_f64(&request.lhs, &request.rhs)
        .map(Into::into)
        .map_err(scalar_f64_error)
}

impl From<ScalarF64ObservationReport> for ScalarF64ObservationDto {
    fn from(report: ScalarF64ObservationReport) -> Self {
        Self {
            lhs_input: report.lhs_input,
            rhs_input: report.rhs_input,
            lhs: report.lhs.into(),
            rhs: report.rhs.into(),
            decimal_reference: report.decimal_reference,
            reference: report.reference.into(),
            results: report.results.into_iter().map(Into::into).collect(),
            all_backends_match: report.all_backends_match,
        }
    }
}

impl From<ScalarF64BackendObservation> for ScalarF64BackendResultDto {
    fn from(observation: ScalarF64BackendObservation) -> Self {
        let backend = observation.backend.display_name().to_owned();

        match observation.status {
            ScalarF64BackendStatus::Completed {
                result,
                matches_reference_bits,
            } => Self {
                backend,
                status: ScalarF64BackendResultStatus::Completed,
                result: Some(result.into()),
                matches_reference_bits: Some(matches_reference_bits),
                error: None,
            },

            ScalarF64BackendStatus::Unavailable => Self {
                backend,
                status: ScalarF64BackendResultStatus::Unavailable,
                result: None,
                matches_reference_bits: None,
                error: None,
            },

            ScalarF64BackendStatus::Failed { error } => Self {
                backend,
                status: ScalarF64BackendResultStatus::Failed,
                result: None,
                matches_reference_bits: None,
                error: Some(error.to_string()),
            },
        }
    }
}

impl From<F64Value> for F64ValueDto {
    fn from(value: F64Value) -> Self {
        Self {
            value: value.value,
            decimal: format!("{:.17}", value.value),
            bits: format!("0x{:016x}", value.bits),
        }
    }
}
