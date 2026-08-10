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
use whitebase_interface::{
    InterfaceError,
    benchmark::{
        BenchmarkOperation, BenchmarkPrecision, BenchmarkReportDto,
        BenchmarkRequest as InterfaceBenchmarkRequest,
        execute_benchmark as execute_interface_benchmark,
    },
    scalar_f64::{
        ScalarF64ObservationDto, ScalarF64Request,
        execute_scalar_f64_observation as execute_interface_scalar_f64_observation,
    },
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
    execute_interface_scalar_f64_observation(request).map_err(Into::into)
}

async fn run_benchmark(
    Json(request): Json<HttpBenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    run_benchmark_task(request.into()).await.map(Json)
}

async fn run_add_benchmark(
    Json(mut request): Json<HttpBenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    request.operation = BenchmarkOperation::AddArray;

    run_benchmark_task(request.into()).await.map(Json)
}

async fn run_legacy_add_f32_benchmark(
    Json(request): Json<LegacyBenchmarkRequest>,
) -> Result<Json<BenchmarkReportDto>, ApiError> {
    run_benchmark_task(InterfaceBenchmarkRequest {
        operation: BenchmarkOperation::AddArray,
        precision: BenchmarkPrecision::F32,
        input_length: request.input_length,
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
    })
    .await
    .map(Json)
}

async fn run_benchmark_task(
    request: InterfaceBenchmarkRequest,
) -> Result<BenchmarkReportDto, ApiError> {
    let task = tokio::task::spawn_blocking(move || execute_benchmark(request));

    task.await.map_err(|error| {
        ApiError::internal(
            "benchmark_task_failed",
            format!("benchmark task failed: {error}"),
        )
    })?
}

fn execute_benchmark(request: InterfaceBenchmarkRequest) -> Result<BenchmarkReportDto, ApiError> {
    execute_interface_benchmark(request).map_err(Into::into)
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpBenchmarkRequest {
    #[serde(default = "default_benchmark_operation")]
    operation: BenchmarkOperation,
    precision: BenchmarkPrecision,
    input_length: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
}

fn default_benchmark_operation() -> BenchmarkOperation {
    BenchmarkOperation::AddArray
}

impl From<HttpBenchmarkRequest> for InterfaceBenchmarkRequest {
    fn from(request: HttpBenchmarkRequest) -> Self {
        Self {
            operation: request.operation,
            precision: request.precision,
            input_length: request.input_length,
            warmup_iterations: request.warmup_iterations,
            measured_iterations: request.measured_iterations,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyBenchmarkRequest {
    input_length: usize,
    warmup_iterations: usize,
    measured_iterations: usize,
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

impl From<InterfaceError> for ApiError {
    fn from(error: InterfaceError) -> Self {
        if error.is_invalid_request() {
            Self::bad_request(error.code(), error.to_string())
        } else {
            Self::internal(error.code(), error.to_string())
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
        let report = execute_benchmark(InterfaceBenchmarkRequest {
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
        let report = execute_benchmark(InterfaceBenchmarkRequest {
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
