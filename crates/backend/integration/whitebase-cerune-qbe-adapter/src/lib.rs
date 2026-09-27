//! Cerune QBE artifactをWhitebaseの計算経路へ接続するためのアダプターです。

#![deny(unsafe_op_in_unsafe_fn)]

use std::{error::Error, fmt};

use cerune_lang::{codegen::qbe::Target, compile_to_qbe_with_target};

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use libloading::Library;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
use tempfile::TempDir;

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
const ADD_SCALAR_F64_SYMBOL: &[u8] = b"whitebase_cerune_add_scalar_f64\0";

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
type AddScalarF64Function = unsafe extern "C" fn(f64, f64) -> f64;

/// WhitebaseがCerune QBE生成時に選択するターゲットです。
///
/// 現在CeruneのQBE backendが対応しているのは
/// Linux x86-64 / GNU環境だけです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeruneQbeTarget {
    /// Linux x86-64 / GNU ABI。
    X86_64UnknownLinuxGnu,
}

impl CeruneQbeTarget {
    /// Cerune側のtarget tripleを返します。
    #[must_use]
    pub const fn triple(self) -> &'static str {
        match self {
            Self::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu",
        }
    }

    const fn cerune_target(self) -> Target {
        match self {
            Self::X86_64UnknownLinuxGnu => Target::X86_64UnknownLinuxGnu,
        }
    }
}

/// Cerune QBE adapterで発生したエラーです。
#[derive(Debug)]
pub enum CeruneQbeAdapterError {
    /// CeruneソースからQBE IRを生成できませんでした。
    Compilation { message: String },

    /// 現在のホストではQBE native実行を提供しません。
    UnsupportedHost {
        architecture: &'static str,
        operating_system: &'static str,
    },

    /// QBE実行ファイルを見つけられませんでした。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    QbeUnavailable,

    /// C compilerを見つけられませんでした。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    CompilerUnavailable,

    /// ファイルまたはプロセス操作に失敗しました。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    Io {
        operation: &'static str,
        source: io::Error,
    },

    /// QBEによるassembly生成に失敗しました。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    QbeFailed {
        program: String,
        status: String,
        stderr: String,
    },

    /// C compilerによる共有ライブラリ生成に失敗しました。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    CompilerFailed {
        program: String,
        status: String,
        stderr: String,
    },

    /// 生成した共有ライブラリを読み込めませんでした。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    LibraryLoad { message: String },

    /// Whitebase用symbolを解決できませんでした。
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    SymbolLoad { message: String },
}

impl fmt::Display for CeruneQbeAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation { message } => {
                write!(
                    formatter,
                    "failed to compile Cerune source to QBE: {message}"
                )
            }
            Self::UnsupportedHost {
                architecture,
                operating_system,
            } => {
                write!(
                    formatter,
                    "Cerune QBE native execution is unsupported on \
                     {architecture}-{operating_system}"
                )
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::QbeUnavailable => {
                write!(formatter, "QBE executable is unavailable")
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::CompilerUnavailable => {
                write!(formatter, "C compiler is unavailable")
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::Io { operation, source } => {
                write!(formatter, "{operation} failed: {source}")
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::QbeFailed {
                program,
                status,
                stderr,
            } => {
                write!(formatter, "QBE failed ({program}, {status}): {stderr}")
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::CompilerFailed {
                program,
                status,
                stderr,
            } => {
                write!(
                    formatter,
                    "C compiler failed ({program}, {status}): {stderr}"
                )
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::LibraryLoad { message } => {
                write!(formatter, "failed to load QBE shared library: {message}")
            }

            #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
            Self::SymbolLoad { message } => {
                write!(formatter, "failed to load QBE symbol: {message}")
            }
        }
    }
}

impl Error for CeruneQbeAdapterError {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// `AddScalarF64`用のCerune QBE artifactを生成します。
///
/// # Errors
///
/// CeruneソースからQBE IRを生成できない場合は
/// [`CeruneQbeAdapterError`]を返します。
pub fn emit_add_scalar_f64_qbe(target: CeruneQbeTarget) -> Result<String, CeruneQbeAdapterError> {
    compile_to_qbe_with_target(ADD_SCALAR_F64_SOURCE, Some(target.cerune_target())).map_err(
        |error| CeruneQbeAdapterError::Compilation {
            message: error.message().to_owned(),
        },
    )
}

/// Cerune QBE artifactをnative共有ライブラリとして実行します。
#[derive(Debug)]
pub struct CeruneQbeAdapter {
    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    add_scalar_f64: AddScalarF64Function,

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    _library: Library,

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    _workspace: TempDir,
}

impl CeruneQbeAdapter {
    /// QBE artifact生成、assembly生成、共有ライブラリ生成、
    /// symbol解決までを完了したadapterを生成します。
    ///
    /// # Errors
    ///
    /// QBE native実行に必要なtoolchainを準備できない場合は
    /// [`CeruneQbeAdapterError`]を返します。
    pub fn new() -> Result<Self, CeruneQbeAdapterError> {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            Self::prepare_linux()
        }

        #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
        {
            Err(CeruneQbeAdapterError::UnsupportedHost {
                architecture: std::env::consts::ARCH,
                operating_system: std::env::consts::OS,
            })
        }
    }

    /// 準備済みのQBE native関数で`f64`スカラー値を加算します。
    #[must_use]
    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> f64 {
        #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
        {
            // SAFETY:
            // `new`でWhitebase所有の固定symbolを解決し、
            // libraryとworkspaceをadapterの生存期間中保持しています。
            unsafe { (self.add_scalar_f64)(lhs, rhs) }
        }

        #[cfg(not(all(target_arch = "x86_64", target_os = "linux")))]
        {
            let _ = (lhs, rhs);
            unreachable!("CeruneQbeAdapter cannot be constructed on an unsupported host");
        }
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    fn prepare_linux() -> Result<Self, CeruneQbeAdapterError> {
        let workspace = tempfile::tempdir().map_err(|source| CeruneQbeAdapterError::Io {
            operation: "create QBE temporary directory",
            source,
        })?;

        let qbe_path = workspace.path().join("add_scalar_f64.ssa");
        let assembly_path = workspace.path().join("add_scalar_f64.s");
        let library_path = workspace.path().join("libwhitebase_cerune_qbe.so");

        let mut qbe = emit_add_scalar_f64_qbe(CeruneQbeTarget::X86_64UnknownLinuxGnu)?;

        qbe.push_str(qbe_host_glue());

        fs::write(&qbe_path, qbe).map_err(|source| CeruneQbeAdapterError::Io {
            operation: "write QBE artifact",
            source,
        })?;

        compile_qbe_to_assembly(&qbe_path, &assembly_path)?;
        compile_shared_library(&assembly_path, &library_path)?;

        // SAFETY:
        // `library_path` is the shared object generated immediately above,
        // and `Library` is retained by the adapter.
        let library = unsafe { Library::new(&library_path) }.map_err(|error| {
            CeruneQbeAdapterError::LibraryLoad {
                message: error.to_string(),
            }
        })?;

        // SAFETY:
        // The symbol is emitted by the fixed Whitebase-owned QBE wrapper
        // with the exact C ABI signature declared by AddScalarF64Function.
        let add_scalar_f64 = unsafe {
            let symbol = library
                .get::<AddScalarF64Function>(ADD_SCALAR_F64_SYMBOL)
                .map_err(|error| CeruneQbeAdapterError::SymbolLoad {
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
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn qbe_host_glue() -> &'static str {
    r#"

export function d $whitebase_cerune_add_scalar_f64(d %lhs, d %rhs) {
@start
  %result =d call $cerune_fn_add_0(d %lhs, d %rhs)
  ret %result
}
"#
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn compile_qbe_to_assembly(input: &Path, output: &Path) -> Result<(), CeruneQbeAdapterError> {
    for program in qbe_candidates() {
        let result = Command::new(&program)
            .arg("-t")
            .arg("amd64_sysv")
            .arg("-o")
            .arg(output)
            .arg(input)
            .output();

        match result {
            Ok(result) if result.status.success() => return Ok(()),
            Ok(result) => {
                return Err(CeruneQbeAdapterError::QbeFailed {
                    program: program.display().to_string(),
                    status: result.status.to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr).trim().to_owned(),
                });
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(CeruneQbeAdapterError::Io {
                    operation: "launch QBE",
                    source,
                });
            }
        }
    }

    Err(CeruneQbeAdapterError::QbeUnavailable)
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn compile_shared_library(input: &Path, output: &Path) -> Result<(), CeruneQbeAdapterError> {
    for program in compiler_candidates() {
        let result = Command::new(&program)
            .arg("-shared")
            .arg("-fPIC")
            .arg(input)
            .arg("-o")
            .arg(output)
            .output();

        match result {
            Ok(result) if result.status.success() => return Ok(()),
            Ok(result) => {
                return Err(CeruneQbeAdapterError::CompilerFailed {
                    program: program.display().to_string(),
                    status: result.status.to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr).trim().to_owned(),
                });
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(source) => {
                return Err(CeruneQbeAdapterError::Io {
                    operation: "launch C compiler",
                    source,
                });
            }
        }
    }

    Err(CeruneQbeAdapterError::CompilerUnavailable)
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn qbe_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(program) = env::var_os("WHITEBASE_QBE") {
        candidates.push(PathBuf::from(program));
    }

    if let Some(program) = env::var_os("QBE") {
        candidates.push(PathBuf::from(program));
    }

    candidates.push(PathBuf::from("qbe"));

    candidates
}

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
fn compiler_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(program) = env::var_os("CC") {
        candidates.push(PathBuf::from(program));
    }

    candidates.push(PathBuf::from("cc"));
    candidates.push(PathBuf::from("gcc"));
    candidates.push(PathBuf::from("clang"));

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_linux_add_scalar_f64_qbe_artifact() {
        let qbe = emit_add_scalar_f64_qbe(CeruneQbeTarget::X86_64UnknownLinuxGnu).unwrap();

        assert!(qbe.starts_with("# target: x86_64-unknown-linux-gnu (qbe -t amd64_sysv)"));
        assert!(qbe.contains("function d $cerune_fn_add_0(d %arg0, d %arg1)"));
        assert!(qbe.contains("=d add"));
        assert!(qbe.contains("export function w $main()"));
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    #[test]
    fn prepares_and_executes_qbe_add_scalar_f64() {
        let adapter = match CeruneQbeAdapter::new() {
            Ok(adapter) => adapter,
            Err(CeruneQbeAdapterError::QbeUnavailable)
            | Err(CeruneQbeAdapterError::CompilerUnavailable) => return,
            Err(error) => panic!("failed to prepare QBE adapter: {error}"),
        };

        let result = adapter.add_scalar_f64(0.1, 0.2);

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux"))]
    #[test]
    fn reuses_prepared_qbe_function() {
        let adapter = match CeruneQbeAdapter::new() {
            Ok(adapter) => adapter,
            Err(CeruneQbeAdapterError::QbeUnavailable)
            | Err(CeruneQbeAdapterError::CompilerUnavailable) => return,
            Err(error) => panic!("failed to prepare QBE adapter: {error}"),
        };

        assert_eq!(adapter.add_scalar_f64(1.25, 2.5), 3.75);
        assert_eq!(adapter.add_scalar_f64(-4.0, 1.5), -2.5);
    }
}
