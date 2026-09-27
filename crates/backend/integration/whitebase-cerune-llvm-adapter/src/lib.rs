//! Cerune LLVM artifactをWhitebaseの計算経路へ接続するためのアダプターです。

#![deny(unsafe_op_in_unsafe_fn)]

use std::{error::Error, fmt};

use cerune_lang::{codegen::llvm::Target, compile_to_llvm_with_target};

#[cfg(not(target_arch = "wasm32"))]
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(not(target_arch = "wasm32"))]
use libloading::Library;

#[cfg(not(target_arch = "wasm32"))]
use tempfile::TempDir;

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

#[cfg(not(target_arch = "wasm32"))]
const ADD_SCALAR_F64_SYMBOL: &[u8] = b"whitebase_cerune_add_scalar_f64\0";

#[cfg(not(target_arch = "wasm32"))]
type AddScalarF64Function = unsafe extern "C" fn(f64, f64) -> f64;

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

/// Cerune LLVM artifactの生成・準備中に発生したエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneLlvmAdapterError {
    /// CeruneソースからLLVM IRを生成できませんでした。
    Compilation { message: String },

    /// 現在のホストには対応するCerune LLVM targetがありません。
    #[cfg(not(target_arch = "wasm32"))]
    UnsupportedHost {
        arch: &'static str,
        os: &'static str,
    },

    /// 利用可能なClangが見つかりませんでした。
    #[cfg(not(target_arch = "wasm32"))]
    CompilerUnavailable { attempts: Vec<String> },

    /// 一時artifactの読み書きに失敗しました。
    #[cfg(not(target_arch = "wasm32"))]
    Io {
        operation: &'static str,
        message: String,
    },

    /// Clangによる共有ライブラリ生成に失敗しました。
    #[cfg(not(target_arch = "wasm32"))]
    CompilerFailed {
        compiler: String,
        status: Option<i32>,
        stderr: String,
    },

    /// 生成した共有ライブラリをロードできませんでした。
    #[cfg(not(target_arch = "wasm32"))]
    LibraryLoad { message: String },

    /// Whitebase用wrapper symbolを解決できませんでした。
    #[cfg(not(target_arch = "wasm32"))]
    SymbolLoad { message: String },
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

            #[cfg(not(target_arch = "wasm32"))]
            Self::UnsupportedHost { arch, os } => {
                write!(formatter, "unsupported Cerune LLVM host: {arch}-{os}")
            }

            #[cfg(not(target_arch = "wasm32"))]
            Self::CompilerUnavailable { attempts } => {
                write!(
                    formatter,
                    "Clang is unavailable; attempted: {}",
                    attempts.join(", ")
                )
            }

            #[cfg(not(target_arch = "wasm32"))]
            Self::Io { operation, message } => {
                write!(formatter, "{operation} failed: {message}")
            }

            #[cfg(not(target_arch = "wasm32"))]
            Self::CompilerFailed {
                compiler,
                status,
                stderr,
            } => {
                write!(
                    formatter,
                    "Clang failed ({compiler}, status {status:?}): {stderr}"
                )
            }

            #[cfg(not(target_arch = "wasm32"))]
            Self::LibraryLoad { message } => {
                write!(formatter, "failed to load Cerune LLVM library: {message}")
            }

            #[cfg(not(target_arch = "wasm32"))]
            Self::SymbolLoad { message } => {
                write!(formatter, "failed to load Cerune LLVM symbol: {message}")
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

/// Cerune LLVM artifactをnative共有ライブラリとして準備したアダプターです。
///
/// LLVM IR生成、Clangによる共有ライブラリ生成、symbol解決は
/// [`CeruneLlvmAdapter::new`] で完了します。
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct CeruneLlvmAdapter {
    add_scalar_f64: AddScalarF64Function,
    _library: Library,
    _workspace: TempDir,
}

#[cfg(not(target_arch = "wasm32"))]
impl CeruneLlvmAdapter {
    /// ホスト向けLLVM artifactを生成し、共有ライブラリとして準備します。
    ///
    /// # Errors
    ///
    /// LLVM生成、Clang起動、共有ライブラリ生成・ロード、
    /// symbol解決のいずれかに失敗した場合は
    /// [`CeruneLlvmAdapterError`]を返します。
    pub fn new() -> Result<Self, CeruneLlvmAdapterError> {
        let target = host_target()?;

        let workspace = tempfile::Builder::new()
            .prefix("whitebase-cerune-llvm-")
            .tempdir()
            .map_err(|error| io_error("create temporary workspace", error))?;

        let llvm_path = workspace.path().join("cerune_add.ll");
        let library_path = workspace.path().join(shared_library_filename());

        let mut llvm = emit_add_scalar_f64_llvm(target)?;
        llvm.push('\n');
        llvm.push_str(host_glue(target));

        // std::fs::writeはUTF-8 BOMを付与しません。
        fs::write(&llvm_path, llvm).map_err(|error| io_error("write LLVM artifact", error))?;

        compile_shared_library(target, &llvm_path, &library_path)?;

        let library = unsafe { Library::new(&library_path) }.map_err(|error| {
            CeruneLlvmAdapterError::LibraryLoad {
                message: error.to_string(),
            }
        })?;

        let add_scalar_f64 = unsafe {
            let symbol = library
                .get::<AddScalarF64Function>(ADD_SCALAR_F64_SYMBOL)
                .map_err(|error| CeruneLlvmAdapterError::SymbolLoad {
                    message: error.to_string(),
                })?;

            *symbol
        };

        Ok(Self {
            add_scalar_f64,
            _library: library,
            _workspace: workspace,
        })
    }

    /// 準備済みLLVM native関数で`f64`スカラー加算を実行します。
    #[must_use]
    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> f64 {
        unsafe { (self.add_scalar_f64)(lhs, rhs) }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn host_target() -> Result<CeruneLlvmTarget, CeruneLlvmAdapterError> {
    #[cfg(all(target_arch = "x86_64", target_os = "windows"))]
    {
        return Ok(CeruneLlvmTarget::X86_64PcWindowsMsvc);
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    {
        return Ok(CeruneLlvmTarget::X86_64UnknownLinuxGnu);
    }

    #[allow(unreachable_code)]
    Err(CeruneLlvmAdapterError::UnsupportedHost {
        arch: env::consts::ARCH,
        os: env::consts::OS,
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn host_glue(target: CeruneLlvmTarget) -> &'static str {
    match target {
        CeruneLlvmTarget::X86_64PcWindowsMsvc => {
            r#"
@_fltused = global i32 0

define dllexport double @whitebase_cerune_add_scalar_f64(double %lhs, double %rhs) {
entry:
  %result = call double @cerune.fn.add.0(double %lhs, double %rhs)
  ret double %result
}
"#
        }

        CeruneLlvmTarget::X86_64UnknownLinuxGnu => {
            r#"
define double @whitebase_cerune_add_scalar_f64(double %lhs, double %rhs) {
entry:
  %result = call double @cerune.fn.add.0(double %lhs, double %rhs)
  ret double %result
}
"#
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn compile_shared_library(
    target: CeruneLlvmTarget,
    llvm_path: &Path,
    library_path: &Path,
) -> Result<(), CeruneLlvmAdapterError> {
    let mut attempts = Vec::new();

    for compiler in clang_candidates() {
        let compiler_name = compiler.display().to_string();
        attempts.push(compiler_name.clone());

        let mut command = Command::new(&compiler);

        command.arg(format!("--target={}", target.triple())).args([
            "-O2",
            "-shared",
            "-Wno-override-module",
        ]);

        match target {
            CeruneLlvmTarget::X86_64PcWindowsMsvc => {
                command.args(["-nostdlib", "-fuse-ld=lld", "-Wl,/noentry"]);
            }

            CeruneLlvmTarget::X86_64UnknownLinuxGnu => {
                command.arg("-fPIC");
            }
        }

        command.arg(llvm_path).arg("-o").arg(library_path);

        match command.output() {
            Ok(output) if output.status.success() => return Ok(()),

            Ok(output) => {
                return Err(CeruneLlvmAdapterError::CompilerFailed {
                    compiler: compiler_name,
                    status: output.status.code(),
                    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
                });
            }

            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,

            Err(error) => {
                return Err(io_error("launch Clang", error));
            }
        }
    }

    Err(CeruneLlvmAdapterError::CompilerUnavailable { attempts })
}

#[cfg(not(target_arch = "wasm32"))]
fn clang_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = env::var_os("WHITEBASE_LLVM_CLANG") {
        candidates.push(path.into());
    }

    if let Some(path) = env::var_os("CLANG") {
        candidates.push(path.into());
    }

    #[cfg(target_os = "windows")]
    candidates.push(PathBuf::from(r"C:\Program Files\LLVM\bin\clang.exe"));

    #[cfg(target_os = "windows")]
    candidates.push(PathBuf::from("clang.exe"));

    #[cfg(not(target_os = "windows"))]
    candidates.push(PathBuf::from("clang"));

    candidates
}

#[cfg(target_os = "windows")]
fn shared_library_filename() -> &'static str {
    "whitebase_cerune_llvm.dll"
}

#[cfg(all(not(target_os = "windows"), not(target_arch = "wasm32")))]
fn shared_library_filename() -> &'static str {
    "libwhitebase_cerune_llvm.so"
}

#[cfg(not(target_arch = "wasm32"))]
fn io_error(operation: &'static str, error: io::Error) -> CeruneLlvmAdapterError {
    CeruneLlvmAdapterError::Io {
        operation,
        message: error.to_string(),
    }
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

    #[cfg(all(
        not(target_arch = "wasm32"),
        target_arch = "x86_64",
        any(target_os = "windows", target_os = "linux")
    ))]
    #[test]
    fn prepares_and_executes_add_scalar_f64() {
        let adapter = match CeruneLlvmAdapter::new() {
            Ok(adapter) => adapter,
            Err(CeruneLlvmAdapterError::CompilerUnavailable { .. }) => return,
            Err(error) => panic!("failed to prepare Cerune LLVM adapter: {error}"),
        };

        let result = adapter.add_scalar_f64(0.1, 0.2);

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[cfg(all(
        not(target_arch = "wasm32"),
        target_arch = "x86_64",
        any(target_os = "windows", target_os = "linux")
    ))]
    #[test]
    fn reuses_prepared_add_scalar_f64_function() {
        let adapter = match CeruneLlvmAdapter::new() {
            Ok(adapter) => adapter,
            Err(CeruneLlvmAdapterError::CompilerUnavailable { .. }) => return,
            Err(error) => panic!("failed to prepare Cerune LLVM adapter: {error}"),
        };

        assert_eq!(
            adapter.add_scalar_f64(1.25, 2.5).to_bits(),
            3.75_f64.to_bits()
        );

        assert_eq!(
            adapter.add_scalar_f64(-4.0, 1.5).to_bits(),
            (-2.5_f64).to_bits()
        );
    }
}
