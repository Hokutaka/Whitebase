use std::sync::OnceLock;

use crate::error::{InterfaceError, scalar_f64_error};
use serde::{Deserialize, Serialize};
use whitebase_runner::{F64Value, Runner, ScalarF64BackendObservation, ScalarF64ObservationReport};

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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScalarF64BackendResultDto {
    pub backend: String,
    pub result: F64ValueDto,
    pub matches_reference_bits: bool,
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
    fn from(result: ScalarF64BackendObservation) -> Self {
        Self {
            backend: result.backend.display_name().to_owned(),
            result: result.result.into(),
            matches_reference_bits: result.matches_reference_bits,
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
