use whitebase_core::BackendKind;

use crate::{
    AddF32Report, AddF64Report, BackendRunResult, Runner, RunnerConfig, RunnerError, SumF64Report,
};

pub const MAX_INPUT_LENGTH: usize = 10_000_000;
pub const MAX_ITERATIONS: usize = 10_000;
pub const MAX_TOTAL_ELEMENT_ITERATIONS: usize = 1_000_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkOperation {
    AddArray,
    SumF64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchmarkPrecision {
    F32,
    F64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BenchmarkRequest {
    pub operation: BenchmarkOperation,
    pub precision: BenchmarkPrecision,
    pub input_length: usize,
    pub warmup_iterations: usize,
    pub measured_iterations: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkReport {
    pub operation: BenchmarkOperation,
    pub precision: BenchmarkPrecision,
    pub input_length: usize,
    pub reference_backend: BackendKind,
    pub warmup_iterations: usize,
    pub measured_iterations: usize,
    pub absolute_tolerance: f64,
    pub results: Vec<BackendRunResult>,
}

pub fn run_benchmark(request: BenchmarkRequest) -> Result<BenchmarkReport, RunnerError> {
    validate_request(request)?;

    let config = RunnerConfig {
        warmup_iterations: request.warmup_iterations,
        measured_iterations: request.measured_iterations,
        ..RunnerConfig::default()
    };

    let runner = Runner::new();

    match request.operation {
        BenchmarkOperation::AddArray => match request.precision {
            BenchmarkPrecision::F32 => {
                let lhs = create_lhs_f32(request.input_length);
                let rhs = create_rhs_f32(request.input_length);

                runner
                    .run_add_f32(&lhs, &rhs, &config)
                    .map(BenchmarkReport::from)
            }
            BenchmarkPrecision::F64 => {
                let lhs = create_lhs_f64(request.input_length);
                let rhs = create_rhs_f64(request.input_length);

                runner
                    .run_add_f64(&lhs, &rhs, &config)
                    .map(BenchmarkReport::from)
            }
        },

        BenchmarkOperation::SumF64 => {
            if request.precision != BenchmarkPrecision::F64 {
                return Err(RunnerError::SumF64RequiresF64);
            }

            let input = create_lhs_f64(request.input_length);

            runner
                .run_sum_f64(&input, &config)
                .map(BenchmarkReport::from)
        }
    }
}

fn validate_request(request: BenchmarkRequest) -> Result<(), RunnerError> {
    if request.input_length == 0 {
        return Err(RunnerError::ZeroInputLength);
    }

    if request.input_length > MAX_INPUT_LENGTH {
        return Err(RunnerError::InputLengthTooLarge {
            maximum: MAX_INPUT_LENGTH,
        });
    }

    if request.warmup_iterations > MAX_ITERATIONS {
        return Err(RunnerError::WarmupIterationsTooLarge {
            maximum: MAX_ITERATIONS,
        });
    }

    if request.measured_iterations > MAX_ITERATIONS {
        return Err(RunnerError::MeasuredIterationsTooLarge {
            maximum: MAX_ITERATIONS,
        });
    }

    if request.measured_iterations == 0 {
        return Err(RunnerError::ZeroMeasuredIterations);
    }

    let total_iterations = request
        .warmup_iterations
        .checked_add(request.measured_iterations)
        .ok_or(RunnerError::BenchmarkWorkloadTooLarge {
            maximum: MAX_TOTAL_ELEMENT_ITERATIONS,
        })?;

    let total_element_iterations = request.input_length.checked_mul(total_iterations).ok_or(
        RunnerError::BenchmarkWorkloadTooLarge {
            maximum: MAX_TOTAL_ELEMENT_ITERATIONS,
        },
    )?;

    if total_element_iterations > MAX_TOTAL_ELEMENT_ITERATIONS {
        return Err(RunnerError::BenchmarkWorkloadTooLarge {
            maximum: MAX_TOTAL_ELEMENT_ITERATIONS,
        });
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

impl From<AddF32Report> for BenchmarkReport {
    fn from(report: AddF32Report) -> Self {
        Self {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F32,
            input_length: report.input_length,
            reference_backend: report.reference_backend,
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: f64::from(report.absolute_tolerance),
            results: report.results,
        }
    }
}

impl From<AddF64Report> for BenchmarkReport {
    fn from(report: AddF64Report) -> Self {
        Self {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F64,
            input_length: report.input_length,
            reference_backend: report.reference_backend,
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: report.absolute_tolerance,
            results: report.results,
        }
    }
}

impl From<SumF64Report> for BenchmarkReport {
    fn from(report: SumF64Report) -> Self {
        Self {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F64,
            input_length: report.input_length,
            reference_backend: report.reference_backend,
            warmup_iterations: report.warmup_iterations,
            measured_iterations: report.measured_iterations,
            absolute_tolerance: report.absolute_tolerance,
            results: report.results,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_input_length() {
        let request = BenchmarkRequest {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F32,
            input_length: 0,
            warmup_iterations: 0,
            measured_iterations: 1,
        };

        assert_eq!(run_benchmark(request), Err(RunnerError::ZeroInputLength));
    }

    #[test]
    fn rejects_zero_measured_iterations() {
        let request = BenchmarkRequest {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F32,
            input_length: 1,
            warmup_iterations: 0,
            measured_iterations: 0,
        };

        assert_eq!(
            run_benchmark(request),
            Err(RunnerError::ZeroMeasuredIterations)
        );
    }

    #[test]
    fn rejects_sum_f64_with_f32_precision() {
        let request = BenchmarkRequest {
            operation: BenchmarkOperation::SumF64,
            precision: BenchmarkPrecision::F32,
            input_length: 1,
            warmup_iterations: 0,
            measured_iterations: 1,
        };

        assert_eq!(run_benchmark(request), Err(RunnerError::SumF64RequiresF64));
    }

    #[test]
    fn rejects_excessive_total_benchmark_workload() {
        let request = BenchmarkRequest {
            operation: BenchmarkOperation::AddArray,
            precision: BenchmarkPrecision::F32,
            input_length: MAX_INPUT_LENGTH,
            warmup_iterations: 100,
            measured_iterations: 1,
        };

        assert_eq!(
            run_benchmark(request),
            Err(RunnerError::BenchmarkWorkloadTooLarge {
                maximum: MAX_TOTAL_ELEMENT_ITERATIONS,
            })
        );
    }
}
