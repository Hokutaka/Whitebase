use std::collections::HashSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use whitebase_core::{BackendKind, ComputeError, Whitebase};

use crate::{
    AddF32Report, AddScalarF64Report, BackendRunResult, BackendRunStatus, ComparisonSummary,
    F64Value, RunnerConfig, RunnerError, ScalarF64BackendObservation, ScalarF64ObservationReport,
    TimingSummary, decimal::ExactDecimal,
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

    /// 指定されたバックエンドで`f64`スカラー加算を実行し、
    /// 入力と結果のビット表現を返します。
    pub fn run_add_scalar_f64(
        &self,
        backend: BackendKind,
        lhs: f64,
        rhs: f64,
    ) -> Result<AddScalarF64Report, RunnerError> {
        let result = self.whitebase.add_scalar_f64(backend, lhs, rhs)?;

        Ok(AddScalarF64Report {
            backend,
            lhs: F64Value::new(lhs),
            rhs: F64Value::new(rhs),
            result: F64Value::new(result),
        })
    }

    /// 入力文字列を10進数として正確に加算しつつ、
    /// 対応する全スカラーバックエンドの`f64`結果を横断して観測します。
    pub fn observe_add_scalar_f64(
        &self,
        lhs_input: &str,
        rhs_input: &str,
    ) -> Result<ScalarF64ObservationReport, RunnerError> {
        let lhs_decimal = ExactDecimal::parse("lhs", lhs_input)?;
        let rhs_decimal = ExactDecimal::parse("rhs", rhs_input)?;

        let lhs = parse_finite_f64("lhs", lhs_input)?;
        let rhs = parse_finite_f64("rhs", rhs_input)?;

        let decimal_reference = lhs_decimal.add(&rhs_decimal).to_canonical_string();
        let reference_value = decimal_reference.parse::<f64>().map_err(|_| {
            RunnerError::ScalarF64ReferenceOutOfRange {
                value: decimal_reference.clone(),
            }
        })?;

        if !reference_value.is_finite() {
            return Err(RunnerError::ScalarF64ReferenceOutOfRange {
                value: decimal_reference,
            });
        }

        let reference = F64Value::new(reference_value);
        let mut results = Vec::new();

        for backend in scalar_f64_backends() {
            let info = self.whitebase.backend_info(backend)?;

            if !info.available {
                continue;
            }

            let report = self.run_add_scalar_f64(backend, lhs, rhs)?;

            results.push(ScalarF64BackendObservation {
                backend,
                matches_reference_bits: report.result.bits == reference.bits,
                result: report.result,
            });
        }

        let first_result_bits = results.first().map(|result| result.result.bits);
        let all_backends_match = first_result_bits
            .is_some_and(|bits| results.iter().all(|result| result.result.bits == bits));

        Ok(ScalarF64ObservationReport {
            lhs_input: lhs_input.trim().to_owned(),
            rhs_input: rhs_input.trim().to_owned(),
            lhs: F64Value::new(lhs),
            rhs: F64Value::new(rhs),
            decimal_reference,
            reference,
            results,
            all_backends_match,
        })
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

fn scalar_f64_backends() -> [BackendKind; 3] {
    [
        BackendKind::RustScalar,
        BackendKind::CppScalar,
        BackendKind::AssemblyScalar,
    ]
}

fn parse_finite_f64(name: &'static str, input: &str) -> Result<f64, RunnerError> {
    let value = input
        .trim()
        .parse::<f64>()
        .map_err(|_| RunnerError::InvalidScalarF64Input {
            name,
            value: input.to_owned(),
            reason: "value must be parseable as f64".to_owned(),
        })?;

    if !value.is_finite() {
        return Err(RunnerError::InvalidScalarF64Input {
            name,
            value: input.to_owned(),
            reason: "value must be finite".to_owned(),
        });
    }

    Ok(value)
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

    #[test]
    fn observes_exact_decimal_reference_for_point_one_plus_point_two() {
        let report = Runner::new()
            .observe_add_scalar_f64("0.1", "0.2")
            .expect("scalar f64 observation must succeed");

        assert_eq!(report.decimal_reference, "0.3");
        assert_eq!(report.reference.bits, 0x3fd3_3333_3333_3333);
        assert!(!report.results.is_empty());
        assert!(report.all_backends_match);
        assert!(report.results.iter().all(|result| {
            result.result.bits == 0x3fd3_3333_3333_3334 && !result.matches_reference_bits
        }));
    }

    #[test]
    fn scalar_observation_rejects_non_finite_inputs() {
        let error = Runner::new()
            .observe_add_scalar_f64("NaN", "0.2")
            .expect_err("NaN must be rejected");

        assert!(matches!(
            error,
            RunnerError::InvalidScalarF64Input { name: "lhs", .. }
        ));
    }
}
