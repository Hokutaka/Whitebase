//! WhitebaseのローカルHTTP APIサーバーです。

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use axum::{
    Json, Router,
    http::{HeaderValue, Method, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use whitebase_runner::{
    BackendRunResult, BackendRunStatus, BenchmarkOperation as RunnerBenchmarkOperation,
    BenchmarkPrecision as RunnerBenchmarkPrecision, BenchmarkReport as RunnerBenchmarkReport,
    BenchmarkRequest as RunnerBenchmarkRequest, F64Value, Runner, RunnerError,
    ScalarF64BackendObservation, ScalarF64ObservationReport, run_benchmark as run_runner_benchmark,
};

const SERVER_ADDRESS: &str = "127.0.0.1:1430";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let allowed_origins = [
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("http://127.0.0.1:1420"),
        HeaderValue::from_static("https://hokutaka.github.io"),
    ];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);

    let application = Router::new()
        .route("/api/health", get(health))
        .route(
            "/api/observations/add-scalar-f64",
            post(observe_add_scalar_f64),
        )
        .route("/api/benchmarks/run", post(run_benchmark))
        .route("/api/benchmarks/add-array", post(run_add_benchmark))
        .route(
            "/api/benchmarks/add-f32",
            post(run_legacy_add_f32_benchmark),
        )
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDRESS).await?;

    println!("[Whitebase Server] Listening on http://{SERVER_ADDRESS}");

    axum::serve(listener, application).await?;

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "whitebase-server",
    })
}

async fn observe_add_scalar_f64(
    Json(request): Json<ScalarF64Request>,
) -> Result<Json<ScalarF64ObservationDto>, ApiError> {
    let task = tokio::task::spawn_blocking(move || execute_scalar_f64_observation(request));

    let result = task.await.map_err(|error| {
        ApiError::internal(
            "scalar_f64_observation_task_failed",
            format!("scalar f64 observation task failed: {error}"),
        )
    })?;

    Ok(Json(result?))
}

fn execute_scalar_f64_observation(
    request: ScalarF64Request,
) -> Result<ScalarF64ObservationDto, ApiError> {
    Runner::new()
        .observe_add_scalar_f64(&request.lhs, &request.rhs)
        .map(Into::into)
        .map_err(map_scalar_f64_error)
}

fn map_scalar_f64_error(error: RunnerError) -> ApiError {
    let is_bad_request = matches!(
        &error,
        RunnerError::InvalidScalarF64Input { .. }
            | RunnerError::ScalarF64ReferenceOutOfRange { .. }
    );

    if is_bad_request {
        ApiError::bad_request("invalid_scalar_f64_request", error.to_string())
    } else {
        ApiError::internal("scalar_f64_observation_failed", error.to_string())
    }
}

async fn run_benchmark(
    Json(request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    run_benchmark_task(request).await.map(Json)
}

async fn run_add_benchmark(
    Json(mut request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    request.operation = BenchmarkOperation::AddArray;
    run_benchmark_task(request).await.map(Json)
}

async fn run_legacy_add_f32_benchmark(
    Json(request): Json<LegacyBenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    run_benchmark_task(BenchmarkRequest {
        operation: BenchmarkOperation::AddArray,
        precision: BenchmarkPrecision::F32,
        input_length: request.input_length,
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
    })
    .await
    .map(Json)
}

async fn run_benchmark_task(request: BenchmarkRequest) -> Result<BenchmarkReportDto, ApiError> {
    let task = tokio::task::spawn_blocking(move || execute_benchmark(request));

    task.await.map_err(|error| {
        ApiError::internal(
            "benchmark_task_failed",
            format!("benchmark task failed: {error}"),
        )
    })?
}

fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, ApiError> {
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
    .map_err(map_benchmark_error)
}

fn map_benchmark_error(error: RunnerError) -> ApiError {
    let is_bad_request = matches!(
        &error,
        RunnerError::ZeroInputLength
            | RunnerError::InputLengthTooLarge { .. }
            | RunnerError::WarmupIterationsTooLarge { .. }
            | RunnerError::MeasuredIterationsTooLarge { .. }
            | RunnerError::ZeroMeasuredIterations
            | RunnerError::SumF64RequiresF64
    );

    if is_bad_request {
        ApiError::bad_request("invalid_benchmark_request", error.to_string())
    } else {
        ApiError::internal("runner_failed", error.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ScalarF64Request {
    lhs: String,
    rhs: String,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScalarF64BackendResultDto {
    backend: String,
    result: F64ValueDto,
    matches_reference_bits: bool,
}

#[derive(Debug, Serialize)]
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum BenchmarkOperation {
    #[default]
    AddArray,
    SumF64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum BenchmarkPrecision {
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkRequest {
    #[serde(default)]
    operation: BenchmarkOperation,
    precision: BenchmarkPrecision,
    input_length: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyBenchmarkRequest {
    input_length: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkReportDto {
    operation: BenchmarkOperation,
    precision: BenchmarkPrecision,
    input_length: usize,
    reference_backend: String,
    warmup_iterations: usize,
    measured_iterations: usize,
    absolute_tolerance: f64,
    results: Vec<BackendResultDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BackendResultDto {
    backend: String,
    status: &'static str,

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
            BackendRunStatus::Completed { timing, comparison } => Self {
                backend,
                status: "completed",

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
                status: "unavailable",

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
                status: "failed",

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

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    fn internal(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code,
            message: message.into(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ApiErrorBody {
            code: self.code,
            message: self.message,
        };

        (self.status, Json(body)).into_response()
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorBody {
    code: &'static str,
    message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_f64_observation_generates_decimal_reference() {
        let report = execute_scalar_f64_observation(ScalarF64Request {
            lhs: "0.1".to_owned(),
            rhs: "0.2".to_owned(),
        })
        .expect("observation must succeed");

        assert_eq!(report.decimal_reference, "0.3");
        assert_eq!(report.reference.decimal, "0.29999999999999999");
        assert_eq!(report.reference.bits, "0x3fd3333333333333");
    }

    #[test]
    fn scalar_f64_observation_maps_invalid_input_to_bad_request() {
        let error = execute_scalar_f64_observation(ScalarF64Request {
            lhs: "not-a-number".to_owned(),
            rhs: "0.2".to_owned(),
        })
        .expect_err("invalid decimal must fail");

        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "invalid_scalar_f64_request");
    }

    #[test]
    fn f64_array_benchmark_uses_all_available_backends() {
        let report = execute_benchmark(BenchmarkRequest {
            operation: BenchmarkOperation::AddArray,
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

    #[test]
    fn sum_f64_benchmark_uses_all_available_backends() {
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
