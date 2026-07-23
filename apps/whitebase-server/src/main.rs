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
use whitebase_runner::{AddF32Report, BackendRunResult, BackendRunStatus, Runner, RunnerConfig};

const SERVER_ADDRESS: &str = "127.0.0.1:1430";
const MAX_INPUT_LENGTH: usize = 10_000_000;
const MAX_ITERATIONS: usize = 10_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let allowed_origins = [
        HeaderValue::from_static("http://localhost:1420"),
        HeaderValue::from_static("http://127.0.0.1:1420"),
    ];

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE]);

    let application = Router::new()
        .route("/api/health", get(health))
        .route("/api/benchmarks/add-f32", post(run_add_f32_benchmark))
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(SERVER_ADDRESS).await?;

    println!(
        "[Whitebase Server] Listening on \
         http://{SERVER_ADDRESS}"
    );

    axum::serve(listener, application).await?;

    Ok(())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "whitebase-server",
    })
}

async fn run_add_f32_benchmark(
    Json(request): Json<BenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    let task = tokio::task::spawn_blocking(move || execute_benchmark(request));

    let result = task.await.map_err(|error| {
        ApiError::internal(
            "benchmark_task_failed",
            format!("benchmark task failed: {error}"),
        )
    })?;

    let report = result?;

    Ok(Json(report))
}

fn execute_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, ApiError> {
    validate_request(request)?;

    let lhs = create_lhs(request.input_length);
    let rhs = create_rhs(request.input_length);

    let config = RunnerConfig {
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
        ..RunnerConfig::default()
    };

    let report = Runner::new()
        .run_add_f32(&lhs, &rhs, &config)
        .map_err(|error| ApiError::internal("runner_failed", error.to_string()))?;

    Ok(report.into())
}

fn validate_request(request: BenchmarkRequest) -> Result<(), ApiError> {
    if request.input_length == 0 {
        return Err(ApiError::bad_request(
            "input_length_zero",
            "input length must be greater than zero",
        ));
    }

    if request.input_length > MAX_INPUT_LENGTH {
        return Err(ApiError::bad_request(
            "input_length_too_large",
            format!(
                "input length must not exceed \
                 {MAX_INPUT_LENGTH}"
            ),
        ));
    }

    if request.measured_iterations == 0 {
        return Err(ApiError::bad_request(
            "measured_iterations_zero",
            "measured iterations must be greater than zero",
        ));
    }

    if request.warmup_iterations > MAX_ITERATIONS {
        return Err(ApiError::bad_request(
            "warmup_iterations_too_large",
            format!(
                "warmup iterations must not exceed \
                 {MAX_ITERATIONS}"
            ),
        ));
    }

    if request.measured_iterations > MAX_ITERATIONS {
        return Err(ApiError::bad_request(
            "measured_iterations_too_large",
            format!(
                "measured iterations must not exceed \
                 {MAX_ITERATIONS}"
            ),
        ));
    }

    Ok(())
}

fn create_lhs(length: usize) -> Vec<f32> {
    (0..length)
        .map(|index| {
            let value = (index % 1024) as f32;
            value * 0.25 - 128.0
        })
        .collect()
}

fn create_rhs(length: usize) -> Vec<f32> {
    (0..length)
        .map(|index| {
            let value = (index % 512) as f32;
            value * 0.5 + 1.0
        })
        .collect()
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkRequest {
    input_length: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BenchmarkReportDto {
    input_length: usize,
    reference_backend: String,
    warmup_iterations: usize,
    measured_iterations: usize,
    absolute_tolerance: f32,
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

impl From<AddF32Report> for BenchmarkReportDto {
    fn from(report: AddF32Report) -> Self {
        Self {
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
                let maximum_absolute_error = comparison.maximum_absolute_error;

                Self {
                    backend,
                    status: "completed",

                    iterations: Some(timing.iterations),
                    total_nanoseconds: Some(timing.total_nanoseconds as f64),
                    minimum_nanoseconds: Some(timing.minimum_nanoseconds as f64),
                    maximum_nanoseconds: Some(timing.maximum_nanoseconds as f64),
                    mean_nanoseconds: Some(timing.mean_nanoseconds),

                    matches_reference: Some(comparison.matches_reference),
                    mismatch_count: Some(comparison.mismatch_count),
                    maximum_absolute_error: maximum_absolute_error
                        .is_finite()
                        .then_some(f64::from(maximum_absolute_error)),

                    error: None,
                }
            }

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
