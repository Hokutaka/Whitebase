use crate::CommandError;
use whitebase_interface::benchmark::{BenchmarkReportDto, BenchmarkRequest, execute_benchmark};

/// 選択された演算をバックグラウンドでベンチマークします。
#[tauri::command]
pub async fn run_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReportDto, CommandError> {
    tauri::async_runtime::spawn_blocking(move || execute_benchmark(request).map_err(Into::into))
        .await
        .map_err(|error| {
            CommandError::internal(
                "benchmark_task_failed",
                format!("benchmark task failed: {error}"),
            )
        })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use whitebase_interface::benchmark::{
        BackendResultStatus, BenchmarkOperation, BenchmarkPrecision,
    };

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
            result.status == BackendResultStatus::Unavailable
                || result.matches_reference == Some(true)
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
            result.status == BackendResultStatus::Unavailable
                || result.matches_reference == Some(true)
        }));
    }
}
