//! Cerune C artifactをWhitebaseの計算経路へ接続するためのアダプターです。

#![forbid(unsafe_code)]

use std::error::Error;
use std::fmt;

use cerune_lang::compile_to_c;

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

/// Cerune C artifactの生成中に発生したエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneCAdapterError {
    /// CeruneソースからC artifactを生成できませんでした。
    Compilation { message: String },
}

impl fmt::Display for CeruneCAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation { message } => {
                write!(formatter, "failed to compile Cerune source to C: {message}")
            }
        }
    }
}

impl Error for CeruneCAdapterError {}

/// `AddScalarF64`用のCerune C artifactを生成します。
///
/// # Errors
///
/// CeruneソースからCコードを生成できない場合は
/// [`CeruneCAdapterError`]を返します。
pub fn emit_add_scalar_f64_c() -> Result<String, CeruneCAdapterError> {
    compile_to_c(ADD_SCALAR_F64_SOURCE).map_err(|error| CeruneCAdapterError::Compilation {
        message: error.message().to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_add_scalar_f64_c_artifact() {
        let c = emit_add_scalar_f64_c().unwrap();

        assert!(c.contains("double cerune_fn_add_0("));
        assert!(c.contains("return"));
    }

    #[test]
    fn emitted_artifact_contains_program_entry_point() {
        let c = emit_add_scalar_f64_c().unwrap();

        assert!(c.contains("int main(void)"));
    }
}
