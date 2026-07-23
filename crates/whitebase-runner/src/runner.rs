use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use whitebase_core::{BackendKind, ComputeError, Whitebase};

use crate::{
    AddF32Report, BackendRunResult, BackendRunStatus, ComparisonSummary, RunnerConfig, RunnerError,
    TimingSummary,
};

/// Whitebase Coreを利用して演算の反復実行、計測、比較を行います。
pub struct Runner {
    whitebase: Whitebase,
}

impl Runner {
    /// 標準構成のWhitebase Coreを使うRunnerを生成します。
    #[must_use]
    pub fn new() -> Self {
        Self {
            whitebase: Whitebase::new(),
        }
    }

    /// 指定されたバックエンドで`f32`配列加算を実行し、
    /// 計測結果と比較結果を返します。
    pub fn run_add_f32(
        &self,
        lhs: &[f32],
        rhs: &[f32],
        config: &RunnerConfig,
    ) -> Result<AddF32Report, RunnerError> {
        validate_config(config)?;

        ComputeError::validate_lengths(lhs.len(), rhs.len(), lhs.len())?;

        let reference_info = self.whitebase.backend_info(config.reference_backend)?;

        if !reference_info.available {
            return Err(RunnerError::ReferenceBackendUnavailable {
                backend: config.reference_backend,
            });
        }

        let mut reference_output = vec![0.0; lhs.len()];

        self.whitebase
            .add_f32(config.reference_backend, lhs, rhs, &mut reference_output)?;

        let backends = unique_backends(&config.backends);

        if backends.is_empty() {
            return Err(RunnerError::NoBackends);
        }

        let mut results = Vec::with_capacity(backends.len());

        for backend in backends {
            results.push(self.run_backend(backend, lhs, rhs, &reference_output, config));
        }

        Ok(AddF32Report {
            input_length: lhs.len(),
            reference_backend: config.reference_backend,
            warmup_iterations: config.warmup_iterations,
            measured_iterations: config.measured_iterations,
            absolute_tolerance: config.absolute_tolerance,
            results,
        })
    }

    fn run_backend(
        &self,
        backend: BackendKind,
        lhs: &[f32],
        rhs: &[f32],
        reference_output: &[f32],
        config: &RunnerConfig,
    ) -> BackendRunResult {
        let info = match self.whitebase.backend_info(backend) {
            Ok(info) => info,

            Err(error) => {
                return BackendRunResult {
                    backend,
                    status: BackendRunStatus::Failed { error },
                };
            }
        };

        if !info.available {
            return BackendRunResult {
                backend,
                status: BackendRunStatus::Unavailable,
            };
        }

        let mut output = vec![0.0; lhs.len()];

        for _ in 0..config.warmup_iterations {
            let result = self.whitebase.add_f32(
                backend,
                black_box(lhs),
                black_box(rhs),
                black_box(output.as_mut_slice()),
            );

            if let Err(error) = result {
                return BackendRunResult {
                    backend,
                    status: BackendRunStatus::Failed { error },
                };
            }
        }

        let mut durations = Vec::with_capacity(config.measured_iterations);

        for _ in 0..config.measured_iterations {
            let started_at = Instant::now();

            let result = self.whitebase.add_f32(
                backend,
                black_box(lhs),
                black_box(rhs),
                black_box(output.as_mut_slice()),
            );

            let elapsed = started_at.elapsed();

            if let Err(error) = result {
                return BackendRunResult {
                    backend,
                    status: BackendRunStatus::Failed { error },
                };
            }

            durations.push(elapsed);
        }

        let timing = summarize_timings(&durations);

        let comparison = compare_outputs(&output, reference_output, config.absolute_tolerance);

        BackendRunResult {
            backend,
            status: BackendRunStatus::Completed { timing, comparison },
        }
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}

fn validate_config(config: &RunnerConfig) -> Result<(), RunnerError> {
    if config.backends.is_empty() {
        return Err(RunnerError::NoBackends);
    }

    if config.measured_iterations == 0 {
        return Err(RunnerError::ZeroMeasuredIterations);
    }

    if !config.absolute_tolerance.is_finite() || config.absolute_tolerance < 0.0 {
        return Err(RunnerError::InvalidAbsoluteTolerance {
            value: config.absolute_tolerance,
        });
    }

    Ok(())
}

fn unique_backends(backends: &[BackendKind]) -> Vec<BackendKind> {
    let mut seen = HashSet::new();

    backends
        .iter()
        .copied()
        .filter(|backend| seen.insert(*backend))
        .collect()
}

fn summarize_timings(durations: &[Duration]) -> TimingSummary {
    let total = durations.iter().copied().sum::<Duration>();

    let minimum = durations
        .iter()
        .copied()
        .min()
        .expect("measured iterations are validated");

    let maximum = durations
        .iter()
        .copied()
        .max()
        .expect("measured iterations are validated");

    let total_nanoseconds = total.as_nanos();

    TimingSummary {
        iterations: durations.len(),
        total_nanoseconds,
        minimum_nanoseconds: minimum.as_nanos(),
        maximum_nanoseconds: maximum.as_nanos(),
        mean_nanoseconds: total_nanoseconds as f64 / durations.len() as f64,
    }
}

fn compare_outputs(actual: &[f32], reference: &[f32], tolerance: f32) -> ComparisonSummary {
    let mut mismatch_count = 0;
    let mut maximum_absolute_error = 0.0_f32;

    for (&actual_value, &reference_value) in actual.iter().zip(reference) {
        let exact_match = actual_value.to_bits() == reference_value.to_bits();

        let both_nan = actual_value.is_nan() && reference_value.is_nan();

        let absolute_error = if exact_match || both_nan {
            0.0
        } else if actual_value.is_finite() && reference_value.is_finite() {
            (actual_value - reference_value).abs()
        } else {
            f32::INFINITY
        };

        maximum_absolute_error = maximum_absolute_error.max(absolute_error);

        let within_tolerance =
            actual_value.is_finite() && reference_value.is_finite() && absolute_error <= tolerance;

        if !exact_match && !both_nan && !within_tolerance {
            mismatch_count += 1;
        }
    }

    ComparisonSummary {
        matches_reference: mismatch_count == 0,
        mismatch_count,
        maximum_absolute_error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_duplicate_backends_without_reordering() {
        let backends = unique_backends(&[
            BackendKind::RustScalar,
            BackendKind::CppScalar,
            BackendKind::RustScalar,
        ]);

        assert_eq!(
            backends,
            vec![BackendKind::RustScalar, BackendKind::CppScalar,]
        );
    }

    #[test]
    fn detects_result_mismatches() {
        let actual = [1.0, 2.5, 3.0];
        let reference = [1.0, 2.0, 3.0];

        let comparison = compare_outputs(&actual, &reference, 0.01);

        assert!(!comparison.matches_reference);
        assert_eq!(comparison.mismatch_count, 1);
        assert_eq!(comparison.maximum_absolute_error, 0.5);
    }
}
