#![deny(unsafe_op_in_unsafe_fn)]

use std::{error::Error, fmt};

use cerune_lang::{codegen::x86_64::Target, compile_to_asm_with_target};

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
use libloading::Library;

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
use tempfile::TempDir;

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
const WHITEBASE_ADD_SCALAR_F64_SYMBOL: &[u8] = b"whitebase_cerune_add_scalar_f64";

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

/// Cerune ASM artifactの生成先です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeruneAsmTarget {
    X86_64PcWindowsMsvc,
    X86_64UnknownLinuxGnu,
}

impl CeruneAsmTarget {
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

/// Cerune ASM artifactの生成エラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneAsmAdapterError {
    Compilation(String),
    UnsupportedHost,
    CompilerUnavailable,
    Io(String),
    CompilerFailed(String),
    LibraryLoad(String),
    SymbolLoad(String),
}

impl fmt::Display for CeruneAsmAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation(message) => {
                write!(formatter, "Cerune ASM compilation failed: {message}")
            }
            Self::UnsupportedHost => {
                write!(
                    formatter,
                    "Cerune ASM execution is unsupported on this host"
                )
            }
            Self::CompilerUnavailable => {
                write!(formatter, "Cerune ASM compiler is unavailable")
            }
            Self::Io(message) => {
                write!(formatter, "Cerune ASM I/O failed: {message}")
            }
            Self::CompilerFailed(message) => {
                write!(formatter, "Cerune ASM compiler failed: {message}")
            }
            Self::LibraryLoad(message) => {
                write!(formatter, "Cerune ASM library load failed: {message}")
            }
            Self::SymbolLoad(message) => {
                write!(formatter, "Cerune ASM symbol load failed: {message}")
            }
        }
    }
}

impl Error for CeruneAsmAdapterError {}

/// `f64 + f64 -> f64` のCerune関数をASM artifactとして生成します。
pub fn emit_add_scalar_f64_asm(target: CeruneAsmTarget) -> Result<String, CeruneAsmAdapterError> {
    compile_to_asm_with_target(ADD_SCALAR_F64_SOURCE, target.cerune_target())
        .map_err(|error| CeruneAsmAdapterError::Compilation(error.message().to_owned()))
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn add_whitebase_wrapper(mut assembly: String) -> String {
    assembly.push_str(concat!(
        "\n.text\n",
        ".globl whitebase_cerune_add_scalar_f64\n",
        ".p2align 4\n",
        "whitebase_cerune_add_scalar_f64:\n",
        "  jmp cerune_fn_add_0\n",
    ));

    assembly
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
type AddScalarF64Function = unsafe extern "C" fn(f64, f64) -> f64;

#[derive(Debug)]
pub struct CeruneAsmAdapter {
    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    add_scalar_f64: AddScalarF64Function,

    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    _library: Library,

    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    _workspace: TempDir,
}

impl CeruneAsmAdapter {
    pub fn new() -> Result<Self, CeruneAsmAdapterError> {
        #[cfg(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        ))]
        {
            prepare_native()
        }

        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        )))]
        {
            Err(CeruneAsmAdapterError::UnsupportedHost)
        }
    }

    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, CeruneAsmAdapterError> {
        #[cfg(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        ))]
        {
            // SAFETY:
            // `new()`で対象ホスト用共有ライブラリから、
            // `f64, f64 -> f64` のC ABI symbolとして解決済みです。
            Ok(unsafe { (self.add_scalar_f64)(lhs, rhs) })
        }

        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        )))]
        {
            let _ = (lhs, rhs);
            Err(CeruneAsmAdapterError::UnsupportedHost)
        }
    }
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn prepare_native() -> Result<CeruneAsmAdapter, CeruneAsmAdapterError> {
    let target = host_target();
    let assembly = emit_add_scalar_f64_asm(target)?;
    let assembly = add_whitebase_wrapper(assembly);

    let workspace =
        tempfile::tempdir().map_err(|error| CeruneAsmAdapterError::Io(error.to_string()))?;

    let source = workspace.path().join("cerune.s");
    let library_path = workspace.path().join(shared_library_name());

    fs::write(&source, assembly).map_err(|error| CeruneAsmAdapterError::Io(error.to_string()))?;

    compile_shared_library(&source, &library_path)?;

    // SAFETY:
    // この直前にWhitebase自身が生成した共有ライブラリを読み込みます。
    let library = unsafe { Library::new(&library_path) }
        .map_err(|error| CeruneAsmAdapterError::LibraryLoad(error.to_string()))?;

    // SAFETY:
    // Whitebaseが同一artifactへ追加した固定名のC ABI wrapperです。
    let add_scalar_f64 = unsafe {
        let symbol: libloading::Symbol<AddScalarF64Function> = library
            .get(WHITEBASE_ADD_SCALAR_F64_SYMBOL)
            .map_err(|error| CeruneAsmAdapterError::SymbolLoad(error.to_string()))?;

        *symbol
    };

    Ok(CeruneAsmAdapter {
        add_scalar_f64,
        _library: library,
        _workspace: workspace,
    })
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
const fn host_target() -> CeruneAsmTarget {
    CeruneAsmTarget::X86_64PcWindowsMsvc
}

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
const fn host_target() -> CeruneAsmTarget {
    CeruneAsmTarget::X86_64UnknownLinuxGnu
}

#[cfg(target_os = "windows")]
fn shared_library_name() -> &'static str {
    "whitebase_cerune_asm.dll"
}

#[cfg(target_os = "linux")]
fn shared_library_name() -> &'static str {
    "libwhitebase_cerune_asm.so"
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn compiler_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = env::var_os("WHITEBASE_CERUNE_ASM_CC") {
        candidates.push(path.into());
    }

    if let Some(path) = env::var_os("CC") {
        candidates.push(path.into());
    }

    #[cfg(target_os = "windows")]
    {
        candidates.push(PathBuf::from("clang"));
        candidates.push(PathBuf::from(r"C:\Program Files\LLVM\bin\clang.exe"));
    }

    #[cfg(target_os = "linux")]
    {
        candidates.push(PathBuf::from("cc"));
        candidates.push(PathBuf::from("clang"));
        candidates.push(PathBuf::from("gcc"));
    }

    candidates
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn find_compiler() -> Result<PathBuf, CeruneAsmAdapterError> {
    compiler_candidates()
        .into_iter()
        .find(|compiler| {
            Command::new(compiler)
                .arg("--version")
                .output()
                .is_ok_and(|output| output.status.success())
        })
        .ok_or(CeruneAsmAdapterError::CompilerUnavailable)
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
fn compile_shared_library(source: &Path, output: &Path) -> Result<(), CeruneAsmAdapterError> {
    let compiler = find_compiler()?;

    let mut command = Command::new(&compiler);

    #[cfg(target_os = "windows")]
    {
        command
            .arg("--target=x86_64-pc-windows-msvc")
            .arg("-shared")
            .arg("-nostdlib")
            .arg("-fuse-ld=lld")
            .arg("-Wl,/noentry")
            .arg("-Wl,/export:whitebase_cerune_add_scalar_f64");
    }

    #[cfg(target_os = "linux")]
    {
        command.arg("-shared").arg("-fPIC");
    }

    let result = command
        .arg(source)
        .arg("-o")
        .arg(output)
        .output()
        .map_err(|error| CeruneAsmAdapterError::Io(error.to_string()))?;

    if !result.status.success() {
        return Err(CeruneAsmAdapterError::CompilerFailed(
            String::from_utf8_lossy(&result.stderr).into_owned(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_target_triples() {
        assert_eq!(
            CeruneAsmTarget::X86_64PcWindowsMsvc.triple(),
            "x86_64-pc-windows-msvc"
        );
        assert_eq!(
            CeruneAsmTarget::X86_64UnknownLinuxGnu.triple(),
            "x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn emits_windows_add_scalar_f64_asm() {
        let asm = emit_add_scalar_f64_asm(CeruneAsmTarget::X86_64PcWindowsMsvc).unwrap();

        assert!(asm.contains("cerune_fn_add_0:"));
        assert!(asm.contains("addsd %xmm1, %xmm0"));
        assert!(asm.contains(".globl main"));
    }

    #[test]
    fn emits_linux_add_scalar_f64_asm() {
        let asm = emit_add_scalar_f64_asm(CeruneAsmTarget::X86_64UnknownLinuxGnu).unwrap();

        assert!(asm.contains("cerune_fn_add_0:"));
        assert!(asm.contains("addsd %xmm1, %xmm0"));
        assert!(asm.contains(".globl main"));
        assert!(asm.contains(".note.GNU-stack"));
    }

    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    #[test]
    fn adds_whitebase_wrapper_to_asm() {
        let asm = emit_add_scalar_f64_asm(CeruneAsmTarget::X86_64PcWindowsMsvc).unwrap();

        let asm = add_whitebase_wrapper(asm);

        assert!(asm.contains(".globl whitebase_cerune_add_scalar_f64"));
        assert!(asm.contains("whitebase_cerune_add_scalar_f64:"));
        assert!(asm.contains("jmp cerune_fn_add_0"));
    }

    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    #[test]
    fn prepares_and_executes_add_scalar_f64_asm() {
        let adapter = CeruneAsmAdapter::new().unwrap();

        let result = adapter.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }
}
