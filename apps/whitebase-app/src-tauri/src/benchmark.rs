use serde::{Deserialize, Serialize};
use whitebase_runner::{
    run_benchmark as run_runner_benchmark, BackendRunResult, BackendRunStatus,
    BenchmarkOperation as RunnerBenchmarkOperation, BenchmarkPrecision as RunnerBenchmarkPrecision,
    BenchmarkReport as RunnerBenchmarkReport, BenchmarkRequest as RunnerBenchmarkRequest,
    TimingMeasurement,
};

/// ベンチマークで実行する演算です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum BenchmarkOperation {
    AddArray,
    SumF64,
}

/// ベンチマークで使用する浮動小数点精度です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum BenchmarkPrecision {
    F32,
    F64,
}

/// フロントエンドから受け取るベンチマーク設定です。
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkRequest {
    /// 実行する演算。
    pub operation: BenchmarkOperation,

    /// 使用する浮動小数点精度。
    pub precision: BenchmarkPrecision,

    /// 配列の要素数。
    pub input_length: usize,

    /// 計測前のウォームアップ回数。
    pub warmup_iterations: usize,

    /// 計測回数。
    pub measured_iterations: usize,
}

/// フロントエンドへ返すベンチマークレポートです。
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

/// 1バックエンド分の表示用結果です。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendResultDto {
    pub backend: String,
    pub status: String,
    pub timing_status: Option<String>,

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

/// 選択された演算をバックグラウンドでベンチマークします。
#[tauri::command]
pub async fn run_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, String> {
    tauri::async_runtime::spawn_blocking(move || execute_benchmark(request))
        .await
        .map_err(|error| format!("benchmark task failed: {error}"))?
}

fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, String> {
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
    .map_err(|error| error.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f64_add_benchmark_reports_selected_operation_and_precision() {
        let report = execute_benchmark(BenchmarkRequest {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F64,
            input_length: 17,
            warmup_iterations: 1,
            measured_iterations: 2,
        })
        .expect("f64 add benchmark must succeed");

        assert_eq!(report.operation, BenchmarkOperation::AddArray);
        assert_eq!(report.precision, BenchmarkPrecision::F64);
        assert_eq!(report.input_length, 17);
        assert!(report.results.iter().all(|result| {
            result.status == "unavailable" || result.matches_reference == Some(true)
        }));
    }

    #[test]
    fn sum_f64_benchmark_uses_reduction_runner() {
        let report = execute_benchmark(BenchmarkRequest {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F64,
            input_length: 17,
            warmup_iterations: 1,
            measured_iterations: 2,
        })
        .expect("sum f64 benchmark must succeed");

        assert_eq!(report.operation, BenchmarkOperation::SumF64);
        assert_eq!(report.precision, BenchmarkPrecision::F64);
        assert_eq!(report.input_length, 17);
        assert!(report.results.iter().all(|result| {
            result.status == "unavailable" || result.matches_reference == Some(true)
        }));
    }
}
