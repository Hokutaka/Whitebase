//! Cerune C artifactをWhitebaseの計算経路へ接続するためのアダプターです。

#![deny(unsafe_op_in_unsafe_fn)]

use std::{
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
};

use cerune_lang::compile_to_c;
use libloading::Library;
use tempfile::TempDir;

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

const ADD_SCALAR_F64_SYMBOL: &[u8] = b"whitebase_cerune_add_scalar_f64\0";

const ADD_SCALAR_F64_WRAPPER: &str = r#"

#ifdef _WIN32
#define WHITEBASE_EXPORT __declspec(dllexport)
#else
#define WHITEBASE_EXPORT __attribute__((visibility("default")))
#endif

WHITEBASE_EXPORT double whitebase_cerune_add_scalar_f64(double lhs, double rhs) {
    return cerune_fn_add_0(lhs, rhs);
}
"#;

type AddScalarF64Function = unsafe extern "C" fn(f64, f64) -> f64;

/// Cerune C経路の準備または実行中に発生したエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneCAdapterError {
    /// CeruneソースからC artifactを生成できませんでした。
    Compilation { message: String },

    /// 利用可能なC compilerを見つけられませんでした。
    CompilerUnavailable { attempts: Vec<String> },

    /// 一時的なsourceまたはbuild artifactを作成できませんでした。
    Io { message: String },

    /// C compilerによる共有ライブラリ生成に失敗しました。
    CompilerFailed {
        compiler: String,
        status: Option<i32>,
        stderr: String,
    },

    /// 生成した共有ライブラリを読み込めませんでした。
    LibraryLoad { message: String },

    /// 共有ライブラリから必要な関数を解決できませんでした。
    SymbolLoad { message: String },
}

impl fmt::Display for CeruneCAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation { message } => {
                write!(formatter, "failed to compile Cerune source to C: {message}")
            }
            Self::CompilerUnavailable { attempts } => write!(
                formatter,
                "C compiler is unavailable. Attempts: {}",
                attempts.join(", ")
            ),
            Self::Io { message } => formatter.write_str(message),
            Self::CompilerFailed {
                compiler,
                status,
                stderr,
            } => write!(
                formatter,
                "C compiler failed: compiler={compiler}, status={status:?}, stderr={stderr}"
            ),
            Self::LibraryLoad { message } => {
                write!(formatter, "failed to load Cerune C library: {message}")
            }
            Self::SymbolLoad { message } => {
                write!(formatter, "failed to load Cerune C symbol: {message}")
            }
        }
    }
}

impl Error for CeruneCAdapterError {}

/// `AddScalarF64`用に準備済みのCerune C実行対象です.
///
/// CeruneからのC生成、C compilerによる共有ライブラリ生成、symbol解決は
/// 構築時に完了します。
///
/// `add_scalar_f64`では準備済み関数だけを呼び出します。
pub struct CeruneCAdapter {
    add_scalar_f64: AddScalarF64Function,

    // function pointerより先にLibraryが破棄されないよう、
    // AdapterがLibrary自体を所有します。
    _library: Library,

    // Libraryが破棄された後に一時build directoryを削除します。
    _workspace: TempDir,
}

impl fmt::Debug for CeruneCAdapter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CeruneCAdapter")
            .finish_non_exhaustive()
    }
}

impl CeruneCAdapter {
    /// Cerune C経路の`AddScalarF64`実行対象を準備します。
    ///
    /// # Errors
    ///
    /// C生成、compiler探索、共有ライブラリ生成、loadまたはsymbol解決に
    /// 失敗した場合は[`CeruneCAdapterError`]を返します。
    pub fn new() -> Result<Self, CeruneCAdapterError> {
        let c = emit_add_scalar_f64_c()?;
        let compiler = find_c_compiler()?;

        let workspace = tempfile::Builder::new()
            .prefix("whitebase-cerune-c-")
            .tempdir()
            .map_err(|error| CeruneCAdapterError::Io {
                message: format!("failed to create Cerune C build directory: {error}"),
            })?;

        let source_path = workspace.path().join("add_scalar_f64.c");
        let library_path = shared_library_path(workspace.path());

        let mut source = c;
        source.push_str(ADD_SCALAR_F64_WRAPPER);

        fs::write(&source_path, source).map_err(|error| CeruneCAdapterError::Io {
            message: format!(
                "failed to write Cerune C source {}: {error}",
                source_path.display()
            ),
        })?;

        compile_shared_library(&compiler, &source_path, &library_path)?;

        // SAFETY:
        // `library_path`は直前にWhitebase自身がC compilerで生成した
        // native shared libraryです。
        let library = unsafe { Library::new(&library_path) }.map_err(|error| {
            CeruneCAdapterError::LibraryLoad {
                message: error.to_string(),
            }
        })?;

        // SAFETY:
        // wrapper側で`whitebase_cerune_add_scalar_f64`を
        // `double(double, double)`のC ABIとして定義しています。
        let add_scalar_f64 = unsafe {
            let symbol = library
                .get::<AddScalarF64Function>(ADD_SCALAR_F64_SYMBOL)
                .map_err(|error| CeruneCAdapterError::SymbolLoad {
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

    /// 準備済みCerune C関数で2つの`f64`値を加算します。
    #[must_use]
    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> f64 {
        // SAFETY:
        // function pointerは構築時にWhitebase wrapperのC ABI symbolから
        // 解決済みで、LibraryもAdapterが保持しています。
        unsafe { (self.add_scalar_f64)(lhs, rhs) }
    }
}

/// `AddScalarF64`用のCerune C artifactを生成します.
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

fn find_c_compiler() -> Result<PathBuf, CeruneCAdapterError> {
    let mut candidates = Vec::new();

    if let Some(cc) = env::var_os("CC") {
        candidates.push(PathBuf::from(cc));
    }

    if cfg!(windows) {
        if let Some(ucrt64_bin) = env::var_os("WHITEBASE_UCRT64_BIN") {
            candidates.push(PathBuf::from(ucrt64_bin).join("gcc.exe"));
        }

        candidates.push(PathBuf::from(r"C:\msys64\ucrt64\bin\gcc.exe"));
        candidates.push(PathBuf::from("gcc.exe"));
    } else {
        candidates.push(PathBuf::from("cc"));
        candidates.push(PathBuf::from("gcc"));
        candidates.push(PathBuf::from("clang"));
    }

    let mut attempts = Vec::new();

    for candidate in candidates {
        let display = candidate.display().to_string();
        attempts.push(display);

        let available = Command::new(&candidate)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());

        if available {
            return Ok(candidate);
        }
    }

    Err(CeruneCAdapterError::CompilerUnavailable { attempts })
}

fn compile_shared_library(
    compiler: &Path,
    source: &Path,
    output: &Path,
) -> Result<(), CeruneCAdapterError> {
    let mut command = Command::new(compiler);

    command.arg("-O2").arg("-shared");

    if !cfg!(windows) {
        command.arg("-fPIC");
    }

    let result = command
        .arg("-o")
        .arg(output)
        .arg(source)
        .output()
        .map_err(|error| CeruneCAdapterError::Io {
            message: format!(
                "failed to launch C compiler {}: {error}",
                compiler.display()
            ),
        })?;

    if !result.status.success() {
        return Err(CeruneCAdapterError::CompilerFailed {
            compiler: compiler.display().to_string(),
            status: result.status.code(),
            stderr: String::from_utf8_lossy(&result.stderr).trim().to_owned(),
        });
    }

    Ok(())
}

fn shared_library_path(directory: &Path) -> PathBuf {
    if cfg!(windows) {
        directory.join("whitebase_cerune_c.dll")
    } else if cfg!(target_os = "macos") {
        directory.join("libwhitebase_cerune_c.dylib")
    } else {
        directory.join("libwhitebase_cerune_c.so")
    }
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

    #[test]
    fn prepares_and_executes_add_scalar_f64() {
        let adapter = CeruneCAdapter::new().unwrap();

        let result = adapter.add_scalar_f64(0.1, 0.2);

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[test]
    fn reuses_prepared_c_function() {
        let adapter = CeruneCAdapter::new().unwrap();

        assert_eq!(adapter.add_scalar_f64(1.0, 2.0), 3.0);
        assert_eq!(adapter.add_scalar_f64(10.5, 20.25), 30.75);
    }
}
