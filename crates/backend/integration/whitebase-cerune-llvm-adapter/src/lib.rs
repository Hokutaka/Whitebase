//! Cerune LLVM artifactをWhitebaseの計算経路へ接続するためのアダプターです。

#![forbid(unsafe_code)]

use std::{error::Error, fmt};

use cerune_lang::{codegen::llvm::Target, compile_to_llvm_with_target};

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

/// WhitebaseがCerune LLVM生成時に選択するターゲットです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeruneLlvmTarget {
    /// Windows x86-64 / MSVC ABI。
    X86_64PcWindowsMsvc,

    /// Linux x86-64 / GNU ABI。
    X86_64UnknownLinuxGnu,
}

impl CeruneLlvmTarget {
    /// LLVM target tripleを返します。
    #[must_use]
    pub const fn triple(self) -> &'static str {
        match self {
            Self::X86_64PcWindowsMsvc => "x86_64-pc-windows-msvc",
            Self::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
        }
    }

    const fn cerune_target(self) -> Target {
        match self {
            Self::X86_64PcWindowsMsvc => Target::X86_64PcWindowsMsvc,
            Self::X86_64UnknownLinuxGnu => Target::X86_64UnknownLinuxGnu,
        }
    }
}

/// Cerune LLVM artifactの生成中に発生したエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneLlvmAdapterError {
    /// CeruneソースからLLVM IRを生成できませんでした。
    Compilation { message: String },
}

impl fmt::Display for CeruneLlvmAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation { message } => {
                write!(
                    formatter,
                    "failed to compile Cerune source to LLVM: {message}"
                )
            }
        }
    }
}

impl Error for CeruneLlvmAdapterError {}

/// `AddScalarF64`用のCerune LLVM artifactを生成します。
///
/// # Errors
///
/// CeruneソースからLLVM IRを生成できない場合は
/// [`CeruneLlvmAdapterError`]を返します。
pub fn emit_add_scalar_f64_llvm(
    target: CeruneLlvmTarget,
) -> Result<String, CeruneLlvmAdapterError> {
    compile_to_llvm_with_target(ADD_SCALAR_F64_SOURCE, Some(target.cerune_target())).map_err(
        |error| CeruneLlvmAdapterError::Compilation {
            message: error.message().to_owned(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_windows_add_scalar_f64_llvm_artifact() {
        let llvm = emit_add_scalar_f64_llvm(CeruneLlvmTarget::X86_64PcWindowsMsvc).unwrap();

        assert!(llvm.starts_with("target triple = \"x86_64-pc-windows-msvc\""));
        assert!(llvm.contains("define double @cerune.fn.add.0("));
        assert!(llvm.contains("fadd double"));
    }

    #[test]
    fn emits_linux_add_scalar_f64_llvm_artifact() {
        let llvm = emit_add_scalar_f64_llvm(CeruneLlvmTarget::X86_64UnknownLinuxGnu).unwrap();

        assert!(llvm.starts_with("target triple = \"x86_64-unknown-linux-gnu\""));
        assert!(llvm.contains("define double @cerune.fn.add.0("));
        assert!(llvm.contains("fadd double"));
    }
}
