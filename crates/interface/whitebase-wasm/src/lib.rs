use std::sync::OnceLock;

use wasm_bindgen::prelude::*;
use whitebase_core::{BackendKind, ComputeError, Whitebase};

use serde::Serialize;
use whitebase_runner::{
    BackendRunResult, BackendRunStatus, BenchmarkOperation as RunnerBenchmarkOperation,
    BenchmarkPrecision as RunnerBenchmarkPrecision, BenchmarkReport as RunnerBenchmarkReport,
    BenchmarkRequest as RunnerBenchmarkRequest, F64Value, Runner, ScalarF64BackendObservation,
    ScalarF64ObservationReport, TimingMeasurement, run_benchmark as run_runner_benchmark,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkReportDto {
    operation: String,
    precision: String,
    input_length: usize,
    reference_backend: String,
    warmup_iterations: usize,
    measured_iterations: usize,
    absolute_tolerance: f64,
    results: Vec<BackendResultDto>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendResultDto {
    backend: String,
    status: String,
    timing_status: Option<String>,

    iterations: Option<usize>,
    total_nanoseconds: Option<f64>,
    minimum_nanoseconds: Option<f64>,
    maximum_nanoseconds: Option<f64>,
    mean_nanoseconds: Option<f64>,

    matches_reference: Option<bool>,
    mismatch_count: Option<usize>,
    maximum_absolute_error: Option<f64>,

    error: Option<String>,
}

static WHITEBASE: OnceLock<Whitebase> = OnceLock::new();

fn whitebase() -> &'static Whitebase {
    WHITEBASE.get_or_init(Whitebase::new)
}

fn js_error(error: ComputeError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// 2つの`f64`スカラー値を加算します。
#[wasm_bindgen]
pub fn add_scalar_f64(left: f64, right: f64) -> Result<f64, JsValue> {
    whitebase()
        .add_scalar_f64(BackendKind::RustScalar, left, right)
        .map_err(js_error)
}

/// 2つの`f32`配列を要素ごとに加算します。
#[wasm_bindgen]
pub fn add_f32(lhs: Box<[f32]>, rhs: Box<[f32]>) -> Result<Box<[f32]>, JsValue> {
    let mut output = vec![0.0; lhs.len()];

    whitebase()
        .add_f32(BackendKind::RustScalar, &lhs, &rhs, &mut output)
        .map_err(js_error)?;

    Ok(output.into_boxed_slice())
}

/// 2つの`f64`配列を要素ごとに加算します。
#[wasm_bindgen]
pub fn add_f64(lhs: Box<[f64]>, rhs: Box<[f64]>) -> Result<Box<[f64]>, JsValue> {
    let mut output = vec![0.0; lhs.len()];

    whitebase()
        .add_f64(BackendKind::RustScalar, &lhs, &rhs, &mut output)
        .map_err(js_error)?;

    Ok(output.into_boxed_slice())
}

/// `f64`配列の要素を合計します。
#[wasm_bindgen]
pub fn sum_f64(input: Box<[f64]>) -> Result<f64, JsValue> {
    whitebase()
        .sum_f64(BackendKind::RustScalar, &input)
        .map_err(js_error)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScalarF64ObservationDto {
    lhs_input: String,
    rhs_input: String,
    lhs: F64ValueDto,
    rhs: F64ValueDto,
    decimal_reference: String,
    reference: F64ValueDto,
    results: Vec<ScalarF64BackendResultDto>,
    all_backends_match: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScalarF64BackendResultDto {
    backend: String,
    result: F64ValueDto,
    matches_reference_bits: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct F64ValueDto {
    value: f64,
    decimal: String,
    bits: String,
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

#[wasm_bindgen]
pub fn observe_add_scalar_f64(lhs: &str, rhs: &str) -> Result<JsValue, JsValue> {
    let report = Runner::new()
        .observe_add_scalar_f64(lhs, rhs)
        .map_err(|error| JsValue::from_str(&error.to_string()))?;

    serde_wasm_bindgen::to_value(&ScalarF64ObservationDto::from(report))
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

impl From<RunnerBenchmarkReport> for BenchmarkReportDto {
    fn from(report: RunnerBenchmarkReport) -> Self {
        Self {
            operation: match report.operation {
                RunnerBenchmarkOperation::AddArray => "add-array".to_owned(),
                RunnerBenchmarkOperation::SumF64 => "sum-f64".to_owned(),
            },
            precision: match report.precision {
                RunnerBenchmarkPrecision::F32 => "f32".to_owned(),
                RunnerBenchmarkPrecision::F64 => "f64".to_owned(),
            },
            input_length: report.input_length,
            reference_backend: report.reference_backend.display_name().to_owned(),
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: report.absolute_tolerance,
            results: report
                .results
                .into_iter()
                .map(BackendResultDto::from)
                .collect(),
        }
    }
}

impl From<BackendRunResult> for BackendResultDto {
    fn from(result: BackendRunResult) -> Self {
        let backend = result.backend.display_name().to_owned();

        match result.status {
            BackendRunStatus::Completed { timing, comparison } => {
                let (
                    timing_status,
                    iterations,
                    total_nanoseconds,
                    minimum_nanoseconds,
                    maximum_nanoseconds,
                    mean_nanoseconds,
                ) = match timing {
                    TimingMeasurement::Measured(timing) => (
                        Some("measured".to_owned()),
                        Some(timing.iterations),
                        Some(timing.total_nanoseconds as f64),
                        Some(timing.minimum_nanoseconds as f64),
                        Some(timing.maximum_nanoseconds as f64),
                        Some(timing.mean_nanoseconds),
                    ),

                    TimingMeasurement::TooFastToMeasure => (
                        Some("too-fast-to-measure".to_owned()),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                };

                Self {
                    backend,
                    status: "completed".to_owned(),
                    timing_status,

                    iterations,
                    total_nanoseconds,
                    minimum_nanoseconds,
                    maximum_nanoseconds,
                    mean_nanoseconds,

                    matches_reference: Some(comparison.matches_reference),
                    mismatch_count: Some(comparison.mismatch_count),
                    maximum_absolute_error: comparison
                        .maximum_absolute_error
                        .is_finite()
                        .then_some(comparison.maximum_absolute_error),

                    error: None,
                }
            }

            BackendRunStatus::Unavailable => Self {
                backend,
                status: "unavailable".to_owned(),
                timing_status: None,

                iterations: None,
                total_nanoseconds: None,
                minimum_nanoseconds: None,
                maximum_nanoseconds: None,
                mean_nanoseconds: None,

                matches_reference: None,
                mismatch_count: None,
                maximum_absolute_error: None,

                error: None,
            },

            BackendRunStatus::Failed { error } => Self {
                backend,
                status: "failed".to_owned(),
                timing_status: None,

                iterations: None,
                total_nanoseconds: None,
                minimum_nanoseconds: None,
                maximum_nanoseconds: None,
                mean_nanoseconds: None,

                matches_reference: None,
                mismatch_count: None,
                maximum_absolute_error: None,

                error: Some(error.to_string()),
            },
        }
    }
}

#[wasm_bindgen]
pub fn run_benchmark(
    operation: &str,
    precision: &str,
    input_length: u32,
    warmup_iterations: u32,
    measured_iterations: u32,
) -> Result<JsValue, JsValue> {
    let operation = match operation {
        "add-array" => RunnerBenchmarkOperation::AddArray,
        "sum-f64" => RunnerBenchmarkOperation::SumF64,
        value => {
            return Err(JsValue::from_str(&format!(
                "unsupported benchmark operation: {value}"
            )));
        }
    };

    let precision = match precision {
        "f32" => RunnerBenchmarkPrecision::F32,
        "f64" => RunnerBenchmarkPrecision::F64,
        value => {
            return Err(JsValue::from_str(&format!(
                "unsupported benchmark precision: {value}"
            )));
        }
    };

    let report = run_runner_benchmark(RunnerBenchmarkRequest {
        operation,
        precision,
        input_length: input_length as usize,
        warmup_iterations: warmup_iterations as usize,
        measured_iterations: measured_iterations as usize,
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    serde_wasm_bindgen::to_value(&BenchmarkReportDto::from(report))
        .map_err(|error| JsValue::from_str(&error.to_string()))
}
