use serde::{Deserialize, Serialize};
use whitebase_runner::{
    AddF32Report, AddF64Report, BackendRunResult, BackendRunStatus, Runner, RunnerConfig,
};

const MAX_INPUT_LENGTH: usize = 10_000_000;
const MAX_ITERATIONS: usize = 10_000;

/// 配列ベンチマークで使用する浮動小数点精度です。
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

/// 選択された精度で配列加算をバックグラウンド実行します。
#[tauri::command]
pub async fn run_add_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, String> {
    tauri::async_runtime::spawn_blocking(move || run_benchmark(request))
        .await
        .map_err(|error| format!("benchmark task failed: {error}"))?
}

fn run_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, String> {
    validate_request(request)?;

    let config = RunnerConfig {
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
        ..RunnerConfig::default()
    };

    let runner = Runner::new();

    match request.precision {
        BenchmarkPrecision::F32 => {
            let lhs = create_lhs_f32(request.input_length);
            let rhs = create_rhs_f32(request.input_length);

            runner
                .run_add_f32(&lhs, &rhs, &config)
                .map(Into::into)
                .map_err(|error| error.to_string())
        }

        BenchmarkPrecision::F64 => {
            let lhs = create_lhs_f64(request.input_length);
            let rhs = create_rhs_f64(request.input_length);

            runner
                .run_add_f64(&lhs, &rhs, &config)
                .map(Into::into)
                .map_err(|error| error.to_string())
        }
    }
}

fn validate_request(request: BenchmarkRequest) -> Result<(), String> {
    if request.input_length == 0 {
        return Err("input length must be greater than zero".to_owned());
    }

    if request.input_length > MAX_INPUT_LENGTH {
        return Err(format!("input length must not exceed {MAX_INPUT_LENGTH}"));
    }

    if request.measured_iterations == 0 {
        return Err("measured iterations must be greater than zero".to_owned());
    }

    if request.warmup_iterations > MAX_ITERATIONS || request.measured_iterations > MAX_ITERATIONS {
        return Err(format!("iteration count must not exceed {MAX_ITERATIONS}"));
    }

    Ok(())
}

fn create_lhs_f32(length: usize) -> Vec<f32> {
    (0..length)
        .map(|index| {
            let value = (index % 1024) as f32;
            value * 0.25 - 128.0
        })
        .collect()
}

fn create_rhs_f32(length: usize) -> Vec<f32> {
    (0..length)
        .map(|index| {
            let value = (index % 512) as f32;
            value * 0.5 + 1.0
        })
        .collect()
}

fn create_lhs_f64(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            let value = (index % 1024) as f64;
            value * 0.25 - 128.0
        })
        .collect()
}

fn create_rhs_f64(length: usize) -> Vec<f64> {
    (0..length)
        .map(|index| {
            let value = (index % 512) as f64;
            value * 0.5 + 1.0
        })
        .collect()
}

impl From<AddF32Report> for BenchmarkReportDto {
    fn from(report: AddF32Report) -> Self {
        Self {
            precision: BenchmarkPrecision::F32,
            input_length: report.input_length,
            reference_backend: report.reference_backend.display_name().to_owned(),
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: f64::from(report.absolute_tolerance),
            results: report
                .results
                .into_iter()
                .map(BackendResultDto::from)
                .collect(),
        }
    }
}

impl From<AddF64Report> for BenchmarkReportDto {
    fn from(report: AddF64Report) -> Self {
        Self {
            precision: BenchmarkPrecision::F64,
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
            BackendRunStatus::Completed { timing, comparison } => Self {
                backend,
                status: "completed".to_owned(),

                iterations: Some(timing.iterations),
                total_nanoseconds: Some(timing.total_nanoseconds as f64),
                minimum_nanoseconds: Some(timing.minimum_nanoseconds as f64),
                maximum_nanoseconds: Some(timing.maximum_nanoseconds as f64),
                mean_nanoseconds: Some(timing.mean_nanoseconds),

                matches_reference: Some(comparison.matches_reference),
                mismatch_count: Some(comparison.mismatch_count),
                maximum_absolute_error: comparison
                    .maximum_absolute_error
                    .is_finite()
                    .then_some(comparison.maximum_absolute_error),

                error: None,
            },

            BackendRunStatus::Unavailable => Self {
                backend,
                status: "unavailable".to_owned(),

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
    fn f64_benchmark_reports_selected_precision() {
        let report = run_benchmark(BenchmarkRequest {
            precision: BenchmarkPrecision::F64,
            input_length: 17,
            warmup_iterations: 1,
            measured_iterations: 2,
        })
        .expect("f64 benchmark must succeed");

        assert_eq!(report.precision, BenchmarkPrecision::F64);
        assert_eq!(report.input_length, 17);
        assert!(report.results.iter().all(|result| {
            result.status == "unavailable" || result.matches_reference == Some(true)
        }));
    }
}
