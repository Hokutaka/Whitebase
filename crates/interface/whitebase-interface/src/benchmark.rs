use crate::error::{InterfaceError, benchmark_error};
use serde::{Deserialize, Serialize};
use whitebase_runner::{
    BackendRunResult, BackendRunStatus, BenchmarkOperation as RunnerBenchmarkOperation,
    BenchmarkPrecision as RunnerBenchmarkPrecision, BenchmarkReport as RunnerBenchmarkReport,
    BenchmarkRequest as RunnerBenchmarkRequest, TimingMeasurement,
    run_benchmark as run_runner_benchmark,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkOperation {
    #[default]
    AddArray,
    SumF64,
}

impl BenchmarkOperation {
    pub fn parse_wire(value: &str) -> Result<Self, String> {
        let deserializer = serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value);

        Self::deserialize(deserializer)
            .map_err(|_| format!("unsupported benchmark operation: {value}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkPrecision {
    F32,
    F64,
}

impl BenchmarkPrecision {
    pub fn parse_wire(value: &str) -> Result<Self, String> {
        let deserializer = serde::de::value::StrDeserializer::<serde::de::value::Error>::new(value);

        Self::deserialize(deserializer)
            .map_err(|_| format!("unsupported benchmark precision: {value}"))
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRequest {
    pub operation: BenchmarkOperation,
    pub precision: BenchmarkPrecision,
    pub input_length: usize,
    pub warmup_iterations: usize,
    pub measured_iterations: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkReportDto {
    pub operation: BenchmarkOperation,
    pub precision: BenchmarkPrecision,
    pub input_length: usize,
    pub reference_backend: String,
    pub warmup_iterations: usize,
    pub measured_iterations: usize,
    pub absolute_tolerance: f64,
    pub results: Vec<BackendResultDto>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackendResultStatus {
    Completed,
    Unavailable,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TimingStatus {
    Measured,
    TooFastToMeasure,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendResultDto {
    pub backend: String,
    pub status: BackendResultStatus,
    pub timing_status: Option<TimingStatus>,

    pub iterations: Option<usize>,
    pub total_nanoseconds: Option<f64>,
    pub minimum_nanoseconds: Option<f64>,
    pub maximum_nanoseconds: Option<f64>,
    pub mean_nanoseconds: Option<f64>,

    pub matches_reference: Option<bool>,
    pub mismatch_count: Option<usize>,
    pub maximum_absolute_error: Option<f64>,

    pub error: Option<String>,
}

pub fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, InterfaceError> {
    run_runner_benchmark(RunnerBenchmarkRequest {
        operation: match request.operation {
            BenchmarkOperation::AddArray => RunnerBenchmarkOperation::AddArray,
            BenchmarkOperation::SumF64 => RunnerBenchmarkOperation::SumF64,
        },
        precision: match request.precision {
            BenchmarkPrecision::F32 => RunnerBenchmarkPrecision::F32,
            BenchmarkPrecision::F64 => RunnerBenchmarkPrecision::F64,
        },
        input_length: request.input_length,
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
    })
    .map(Into::into)
    .map_err(benchmark_error)
}

impl From<RunnerBenchmarkReport> for BenchmarkReportDto {
    fn from(report: RunnerBenchmarkReport) -> Self {
        Self {
            operation: match report.operation {
                RunnerBenchmarkOperation::AddArray => BenchmarkOperation::AddArray,
                RunnerBenchmarkOperation::SumF64 => BenchmarkOperation::SumF64,
            },
            precision: match report.precision {
                RunnerBenchmarkPrecision::F32 => BenchmarkPrecision::F32,
                RunnerBenchmarkPrecision::F64 => BenchmarkPrecision::F64,
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
                        Some(TimingStatus::Measured),
                        Some(timing.iterations),
                        Some(timing.total_nanoseconds as f64),
                        Some(timing.minimum_nanoseconds as f64),
                        Some(timing.maximum_nanoseconds as f64),
                        Some(timing.mean_nanoseconds),
                    ),

                    TimingMeasurement::TooFastToMeasure => (
                        Some(TimingStatus::TooFastToMeasure),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ),
                };

                Self {
                    backend,
                    status: BackendResultStatus::Completed,
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
                status: BackendResultStatus::Unavailable,
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
                status: BackendResultStatus::Failed,
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
