use std::error::Error;
use std::fmt;

use whitebase_core::{BackendKind, ComputeError};

/// Runnerの設定または実行時に発生するエラーです。
#[derive(Debug, Clone, PartialEq)]
pub enum RunnerError {
    /// 実行対象のバックエンドが指定されていません。
    NoBackends,

    /// 計測回数が0です。
    ZeroMeasuredIterations,

    /// 絶対誤差の許容値が不正です。
    InvalidAbsoluteTolerance { value: f64 },

    /// 参照バックエンドを現在の環境で利用できません。
    ReferenceBackendUnavailable { backend: BackendKind },

    /// `f64`スカラー観測用の10進入力が不正です。
    InvalidScalarF64Input {
        name: &'static str,
        value: String,
        reason: String,
    },

    /// 正確な10進参照値を有限の`f64`へ変換できません。
    ScalarF64ReferenceOutOfRange { value: String },

    /// Coreによる演算実行に失敗しました。
    Compute { error: ComputeError },

    /// ベンチマークの入力長が0です。
    ZeroInputLength,

    /// ベンチマークの入力長が上限を超えています。
    InputLengthTooLarge { maximum: usize },

    /// ウォームアップ回数が上限を超えています。
    WarmupIterationsTooLarge { maximum: usize },

    /// 計測回数が上限を超えています。
    MeasuredIterationsTooLarge { maximum: usize },

    /// `SumF64`ベンチマークに`F32`精度が指定されています。
    SumF64RequiresF64,

    /// ベンチマークの総処理量が上限を超えています。
    BenchmarkWorkloadTooLarge { maximum: usize },
}

impl fmt::Display for RunnerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBackends => {
                write!(formatter, "no backends were selected")
            }

            Self::ZeroMeasuredIterations => {
                write!(formatter, "measured iterations must be greater than zero")
            }

            Self::InvalidAbsoluteTolerance { value } => {
                write!(
                    formatter,
                    "absolute tolerance must be a finite, \
                     non-negative value: {value}"
                )
            }

            Self::ReferenceBackendUnavailable { backend } => {
                write!(
                    formatter,
                    "reference backend is unavailable: {}",
                    backend.display_name()
                )
            }

            Self::InvalidScalarF64Input {
                name,
                value,
                reason,
            } => {
                write!(formatter, "invalid {name} value `{value}`: {reason}")
            }

            Self::ScalarF64ReferenceOutOfRange { value } => {
                write!(
                    formatter,
                    "decimal reference is outside the finite f64 range: {value}"
                )
            }

            Self::Compute { error } => {
                write!(formatter, "compute operation failed: {error}")
            }

            Self::ZeroInputLength => {
                write!(formatter, "input length must be greater than zero")
            }

            Self::InputLengthTooLarge { maximum } => {
                write!(formatter, "input length must not exceed {maximum}")
            }

            Self::WarmupIterationsTooLarge { maximum } => {
                write!(formatter, "warmup iterations must not exceed {maximum}")
            }

            Self::MeasuredIterationsTooLarge { maximum } => {
                write!(formatter, "measured iterations must not exceed {maximum}")
            }

            Self::SumF64RequiresF64 => {
                write!(formatter, "sum-f64 benchmark requires f64 precision")
            }

            Self::BenchmarkWorkloadTooLarge { maximum } => {
                write!(
                    formatter,
                    "benchmark workload must not exceed {maximum} element-iterations"
                )
            }
        }
    }
}

impl Error for RunnerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Compute { error } => Some(error),
            _ => None,
        }
    }
}

impl From<ComputeError> for RunnerError {
    fn from(error: ComputeError) -> Self {
        Self::Compute { error }
    }
}
