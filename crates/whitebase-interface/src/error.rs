use std::error::Error;
use std::fmt;

use crate::{BackendKind, OperationKind};

/// 計算バックエンドの実行時に発生するエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComputeError {
    /// 入力配列または出力配列の長さが一致しません。
    LengthMismatch {
        lhs_len: usize,
        rhs_len: usize,
        output_len: usize,
    },

    /// バックエンドを現在の環境で利用できません。
    BackendUnavailable { backend: BackendKind },

    /// バックエンドが指定された演算をサポートしていません。
    OperationUnsupported {
        backend: BackendKind,
        operation: OperationKind,
    },

    /// バックエンド内部で処理に失敗しました。
    BackendFailure {
        backend: BackendKind,
        message: String,
    },

    /// 指定されたバックエンドが登録されていません。
    BackendNotRegistered { backend: BackendKind },
}

impl ComputeError {
    /// 配列長の一致を検証します。
    pub fn validate_lengths(lhs_len: usize, rhs_len: usize, output_len: usize) -> Result<(), Self> {
        if lhs_len == rhs_len && lhs_len == output_len {
            return Ok(());
        }

        Err(Self::LengthMismatch {
            lhs_len,
            rhs_len,
            output_len,
        })
    }
}

impl fmt::Display for ComputeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthMismatch {
                lhs_len,
                rhs_len,
                output_len,
            } => write!(
                formatter,
                "array lengths do not match: \
                 lhs={lhs_len}, rhs={rhs_len}, output={output_len}"
            ),

            Self::BackendUnavailable { backend } => {
                write!(formatter, "{} is not available", backend.display_name())
            }

            Self::OperationUnsupported { backend, operation } => write!(
                formatter,
                "{} does not support {operation:?}",
                backend.display_name()
            ),

            Self::BackendFailure { backend, message } => {
                write!(formatter, "{} failed: {message}", backend.display_name())
            }

            Self::BackendNotRegistered { backend } => {
                write!(formatter, "{} is not registered", backend.display_name())
            }
        }
    }
}

impl Error for ComputeError {}
