//! Cerune VMをWhitebaseの計算経路から利用するためのアダプターです。

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use cerune_lang::{
    compile_to_bytecode,
    embedding::{HostValue, ResolvedFunction, invoke_function, resolve_function},
};

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

const ADD_SCALAR_F64_FUNCTION: &str = "add";

/// Cerune VM経路の準備または実行中に発生したエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneVmAdapterError {
    /// Ceruneソースからbytecodeを生成できませんでした。
    Compilation { message: String },

    /// 生成済みbytecodeから対象関数を解決できませんでした。
    FunctionResolution { message: String },

    /// Cerune VMで関数を実行できませんでした。
    Invocation { message: String },

    /// `f64`を返すべき関数が値を返しませんでした。
    MissingReturnValue,

    /// 現在のadapterでは扱えない戻り値が返されました。
    UnsupportedReturnValue,
}

impl fmt::Display for CeruneVmAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation { message } => {
                write!(formatter, "failed to compile Cerune source: {message}")
            }
            Self::FunctionResolution { message } => {
                write!(formatter, "failed to resolve Cerune function: {message}")
            }
            Self::Invocation { message } => {
                write!(formatter, "failed to invoke Cerune function: {message}")
            }
            Self::MissingReturnValue => {
                write!(formatter, "Cerune function did not return a value")
            }
            Self::UnsupportedReturnValue => {
                write!(formatter, "Cerune function returned an unsupported value")
            }
        }
    }
}

impl Error for CeruneVmAdapterError {}

/// `AddScalarF64`用に準備済みのCerune VM実行対象です。
///
/// 生成と関数解決は構築時に完了するため、`add_scalar_f64`の呼び出しでは
/// 同じ`ResolvedFunction`を繰り返し利用します。
#[derive(Debug, Clone)]
pub struct CeruneVmAdapter {
    add_scalar_f64: ResolvedFunction,
}

impl CeruneVmAdapter {
    /// Ceruneソースをbytecodeへ変換し、実行対象関数を解決します。
    ///
    /// # Errors
    ///
    /// Ceruneソースのコンパイルまたは関数解決に失敗した場合は
    /// [`CeruneVmAdapterError`]を返します。
    pub fn new() -> Result<Self, CeruneVmAdapterError> {
        let bytecode = compile_to_bytecode(ADD_SCALAR_F64_SOURCE).map_err(|error| {
            CeruneVmAdapterError::Compilation {
                message: error.message().to_owned(),
            }
        })?;

        let add_scalar_f64 = resolve_function(Arc::new(bytecode), ADD_SCALAR_F64_FUNCTION)
            .map_err(|error| CeruneVmAdapterError::FunctionResolution {
                message: format!("{error:?}"),
            })?;

        Ok(Self { add_scalar_f64 })
    }

    /// Cerune VMで2つの`f64`スカラー値を加算します。
    ///
    /// 構築時に解決済みの関数を利用し、この呼び出しでは再コンパイルしません。
    ///
    /// # Errors
    ///
    /// VM実行に失敗した場合、または期待した`f64`戻り値を取得できない場合は
    /// [`CeruneVmAdapterError`]を返します。
    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, CeruneVmAdapterError> {
        let execution = invoke_function(
            &self.add_scalar_f64,
            &[HostValue::F64(lhs), HostValue::F64(rhs)],
        )
        .map_err(|error| CeruneVmAdapterError::Invocation {
            message: format!("{error:?}"),
        })?;

        match execution.return_value() {
            Some(HostValue::F64(value)) => Ok(*value),
            Some(_) => Err(CeruneVmAdapterError::UnsupportedReturnValue),
            None => Err(CeruneVmAdapterError::MissingReturnValue),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_f64_scalars_with_cerune_vm() {
        let adapter = CeruneVmAdapter::new().unwrap();

        let result = adapter.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[test]
    fn reuses_prepared_function() {
        let adapter = CeruneVmAdapter::new().unwrap();

        assert_eq!(adapter.add_scalar_f64(1.0, 2.0).unwrap(), 3.0);
        assert_eq!(adapter.add_scalar_f64(10.5, 20.25).unwrap(), 30.75);
    }
}
