#![deny(unsafe_op_in_unsafe_fn)]

use std::{error::Error, fmt};

use cerune_lang::{codegen::x86_64::Target, compile_to_native_object};

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

/// Cerune Native Object artifactの生成先です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeruneNativeTarget {
    X86_64PcWindowsMsvc,
    X86_64UnknownLinuxGnu,
}

impl CeruneNativeTarget {
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

/// Cerune Native Object artifactの生成エラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneNativeAdapterError {
    Compilation(String),
    UnsupportedHost,
    ObjcopyUnavailable,
    CompilerUnavailable,
    Io(String),
    AdaptationFailed(String),
    LinkFailed(String),
    LibraryLoad(String),
    SymbolLoad(String),
}

impl fmt::Display for CeruneNativeAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation(message) => {
                write!(
                    formatter,
                    "Cerune native object compilation failed: {message}"
                )
            }
            Self::UnsupportedHost => {
                write!(
                    formatter,
                    "Cerune native object execution is unsupported on this host"
                )
            }
            Self::ObjcopyUnavailable => {
                write!(
                    formatter,
                    "Cerune native object objcopy tool is unavailable"
                )
            }
            Self::CompilerUnavailable => {
                write!(
                    formatter,
                    "Cerune native object linker driver is unavailable"
                )
            }
            Self::Io(message) => {
                write!(formatter, "Cerune native object I/O failed: {message}")
            }
            Self::AdaptationFailed(message) => {
                write!(
                    formatter,
                    "Cerune native object adaptation failed: {message}"
                )
            }
            Self::LinkFailed(message) => {
                write!(formatter, "Cerune native object link failed: {message}")
            }
            Self::LibraryLoad(message) => {
                write!(formatter, "Cerune native library load failed: {message}")
            }
            Self::SymbolLoad(message) => {
                write!(formatter, "Cerune native symbol load failed: {message}")
            }
        }
    }
}

impl Error for CeruneNativeAdapterError {}

/// `f64 + f64 -> f64` のCerune関数をnative objectとして生成します。
pub fn emit_add_scalar_f64_object(
    target: CeruneNativeTarget,
) -> Result<Vec<u8>, CeruneNativeAdapterError> {
    compile_to_native_object(ADD_SCALAR_F64_SOURCE, target.cerune_target(), false)
        .map_err(|error| CeruneNativeAdapterError::Compilation(error.message().to_owned()))
}

/// Ceruneが直接生成したNative Objectを実行するAdapterです。
///
/// Native Object生成、symbol変換、共有ライブラリ構築、関数解決は
/// `new()`で完了し、演算呼び出し中には行いません。
pub struct CeruneNativeAdapter {
    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    native: native::NativeState,
}

impl fmt::Debug for CeruneNativeAdapter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CeruneNativeAdapter")
            .finish_non_exhaustive()
    }
}

impl CeruneNativeAdapter {
    pub fn new() -> Result<Self, CeruneNativeAdapterError> {
        #[cfg(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        ))]
        {
            Ok(Self {
                native: native::prepare_native()?,
            })
        }

        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        )))]
        {
            Err(CeruneNativeAdapterError::UnsupportedHost)
        }
    }

    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, CeruneNativeAdapterError> {
        #[cfg(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        ))]
        {
            Ok(self.native.add_scalar_f64(lhs, rhs))
        }

        #[cfg(not(any(
            all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
            all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
        )))]
        {
            let _ = (lhs, rhs);

            Err(CeruneNativeAdapterError::UnsupportedHost)
        }
    }
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
mod native {
    use super::{CeruneNativeAdapterError, CeruneNativeTarget, emit_add_scalar_f64_object};

    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
    };

    use libloading::Library;
    use tempfile::TempDir;

    const CERUNE_ADD_SCALAR_F64_SYMBOL: &str = "cerune_fn_add_0";

    const WHITEBASE_ADD_SCALAR_F64_SYMBOL_NAME: &str = "whitebase_cerune_add_scalar_f64";

    const WHITEBASE_ADD_SCALAR_F64_SYMBOL: &[u8] = b"whitebase_cerune_add_scalar_f64";

    type AddScalarF64Function = unsafe extern "C" fn(f64, f64) -> f64;

    pub(super) struct NativeState {
        add_scalar_f64: AddScalarF64Function,
        _library: Library,
        _workspace: TempDir,
    }

    impl NativeState {
        pub(super) fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> f64 {
            // SAFETY:
            // `prepare_native()`でWhitebase所有の固定C ABI名として
            // 共有ライブラリから解決済みです。
            unsafe { (self.add_scalar_f64)(lhs, rhs) }
        }
    }

    pub(super) fn prepare_native() -> Result<NativeState, CeruneNativeAdapterError> {
        let object = emit_add_scalar_f64_object(host_target())?;

        let workspace =
            tempfile::tempdir().map_err(|error| CeruneNativeAdapterError::Io(error.to_string()))?;

        let input = workspace.path().join(object_name());
        let adapted = workspace.path().join(adapted_object_name());
        let library_path = workspace.path().join(shared_library_name());

        fs::write(&input, object)
            .map_err(|error| CeruneNativeAdapterError::Io(error.to_string()))?;

        #[cfg(target_os = "windows")]
        adapt_windows_coff_symbol(&input, &adapted)?;

        #[cfg(target_os = "linux")]
        {
            let globalized = workspace.path().join(global_object_name());

            adapt_linux_object_symbols(&input, &globalized, &adapted)?;
        }

        link_shared_library(&adapted, &library_path)?;

        // SAFETY:
        // Whitebaseがこの処理中に生成・linkした共有ライブラリです。
        let library = unsafe { Library::new(&library_path) }
            .map_err(|error| CeruneNativeAdapterError::LibraryLoad(error.to_string()))?;

        // SAFETY:
        // objcopyでglobal化し、Whitebase固定名へrenameした
        // `f64, f64 -> f64` のC ABI関数です。
        let add_scalar_f64 = unsafe {
            let symbol: libloading::Symbol<AddScalarF64Function> = library
                .get(WHITEBASE_ADD_SCALAR_F64_SYMBOL)
                .map_err(|error| CeruneNativeAdapterError::SymbolLoad(error.to_string()))?;

            *symbol
        };

        Ok(NativeState {
            add_scalar_f64,
            _library: library,
            _workspace: workspace,
        })
    }

    #[cfg(target_os = "windows")]
    fn adapt_windows_coff_symbol(
        input: &Path,
        output: &Path,
    ) -> Result<(), CeruneNativeAdapterError> {
        const COFF_HEADER_SIZE: usize = 20;
        const COFF_SYMBOL_SIZE: usize = 18;
        const IMAGE_SYM_CLASS_EXTERNAL: u8 = 2;

        let mut object =
            fs::read(input).map_err(|error| CeruneNativeAdapterError::Io(error.to_string()))?;

        if object.len() < COFF_HEADER_SIZE {
            return Err(CeruneNativeAdapterError::AdaptationFailed(
                "COFF header is truncated".to_owned(),
            ));
        }

        let symbol_table_offset = read_coff_u32(&object, 8)? as usize;

        let symbol_count = read_coff_u32(&object, 12)? as usize;

        let symbol_table_size = symbol_count.checked_mul(COFF_SYMBOL_SIZE).ok_or_else(|| {
            CeruneNativeAdapterError::AdaptationFailed("COFF symbol table size overflow".to_owned())
        })?;

        let string_table_offset = symbol_table_offset
            .checked_add(symbol_table_size)
            .ok_or_else(|| {
                CeruneNativeAdapterError::AdaptationFailed(
                    "COFF string table offset overflow".to_owned(),
                )
            })?;

        if string_table_offset
            .checked_add(4)
            .is_none_or(|end| end > object.len())
        {
            return Err(CeruneNativeAdapterError::AdaptationFailed(
                "COFF string table is missing".to_owned(),
            ));
        }

        let string_table_length = read_coff_u32(&object, string_table_offset)?;

        let string_table_end = string_table_offset
            .checked_add(string_table_length as usize)
            .ok_or_else(|| {
                CeruneNativeAdapterError::AdaptationFailed(
                    "COFF string table size overflow".to_owned(),
                )
            })?;

        if string_table_end > object.len() {
            return Err(CeruneNativeAdapterError::AdaptationFailed(
                "COFF string table is truncated".to_owned(),
            ));
        }

        let mut symbol_index = 0usize;
        let mut target_entry = None;

        while symbol_index < symbol_count {
            let entry = symbol_table_offset
                .checked_add(symbol_index.checked_mul(COFF_SYMBOL_SIZE).ok_or_else(|| {
                    CeruneNativeAdapterError::AdaptationFailed(
                        "COFF symbol offset overflow".to_owned(),
                    )
                })?)
                .ok_or_else(|| {
                    CeruneNativeAdapterError::AdaptationFailed(
                        "COFF symbol offset overflow".to_owned(),
                    )
                })?;

            if entry + COFF_SYMBOL_SIZE > object.len() {
                return Err(CeruneNativeAdapterError::AdaptationFailed(
                    "COFF symbol table is truncated".to_owned(),
                ));
            }

            let name = coff_symbol_name(&object, entry, string_table_offset, string_table_end)?;

            if name == CERUNE_ADD_SCALAR_F64_SYMBOL {
                target_entry = Some(entry);
                break;
            }

            let auxiliary_count = object[entry + COFF_SYMBOL_SIZE - 1] as usize;

            symbol_index = symbol_index
                .checked_add(1 + auxiliary_count)
                .ok_or_else(|| {
                    CeruneNativeAdapterError::AdaptationFailed(
                        "COFF symbol index overflow".to_owned(),
                    )
                })?;
        }

        let entry = target_entry.ok_or_else(|| {
            CeruneNativeAdapterError::AdaptationFailed(format!(
                "COFF symbol `{CERUNE_ADD_SCALAR_F64_SYMBOL}` was not found"
            ))
        })?;

        let new_name = WHITEBASE_ADD_SCALAR_F64_SYMBOL_NAME.as_bytes();

        let new_name_offset = string_table_length;

        let new_string_table_length = string_table_length
            .checked_add(u32::try_from(new_name.len() + 1).map_err(|_| {
                CeruneNativeAdapterError::AdaptationFailed(
                    "COFF symbol name is too large".to_owned(),
                )
            })?)
            .ok_or_else(|| {
                CeruneNativeAdapterError::AdaptationFailed(
                    "COFF string table length overflow".to_owned(),
                )
            })?;

        // COFFの8-byte name fieldをlong-name参照へ変更します。
        object[entry..entry + 4].fill(0);

        object[entry + 4..entry + 8].copy_from_slice(&new_name_offset.to_le_bytes());

        // Cerune local symbol (STATIC=3) をexternal symbolへ変更します。
        object[entry + 16] = IMAGE_SYM_CLASS_EXTERNAL;

        object.splice(
            string_table_end..string_table_end,
            new_name.iter().copied().chain(std::iter::once(0)),
        );

        object[string_table_offset..string_table_offset + 4]
            .copy_from_slice(&new_string_table_length.to_le_bytes());

        fs::write(output, object).map_err(|error| CeruneNativeAdapterError::Io(error.to_string()))
    }

    #[cfg(target_os = "linux")]
    fn adapt_linux_object_symbols(
        input: &Path,
        globalized: &Path,
        output: &Path,
    ) -> Result<(), CeruneNativeAdapterError> {
        let objcopy = find_objcopy()?;

        let globalize = Command::new(&objcopy)
            .arg(format!("--globalize-symbol={CERUNE_ADD_SCALAR_F64_SYMBOL}"))
            .arg(input)
            .arg(globalized)
            .output()
            .map_err(|error| CeruneNativeAdapterError::Io(error.to_string()))?;

        if !globalize.status.success() {
            return Err(CeruneNativeAdapterError::AdaptationFailed(
                String::from_utf8_lossy(&globalize.stderr).into_owned(),
            ));
        }

        let redefine = Command::new(&objcopy)
        .arg(format!(
            "--redefine-sym={CERUNE_ADD_SCALAR_F64_SYMBOL}={WHITEBASE_ADD_SCALAR_F64_SYMBOL_NAME}"
        ))
        .arg(globalized)
        .arg(output)
        .output()
        .map_err(|error| {
            CeruneNativeAdapterError::Io(error.to_string())
        })?;

        if !redefine.status.success() {
            return Err(CeruneNativeAdapterError::AdaptationFailed(
                String::from_utf8_lossy(&redefine.stderr).into_owned(),
            ));
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn read_coff_u32(bytes: &[u8], offset: usize) -> Result<u32, CeruneNativeAdapterError> {
        let value = bytes.get(offset..offset + 4).ok_or_else(|| {
            CeruneNativeAdapterError::AdaptationFailed("COFF field is truncated".to_owned())
        })?;

        Ok(u32::from_le_bytes([value[0], value[1], value[2], value[3]]))
    }

    #[cfg(target_os = "windows")]
    fn coff_symbol_name(
        object: &[u8],
        entry: usize,
        string_table_offset: usize,
        string_table_end: usize,
    ) -> Result<String, CeruneNativeAdapterError> {
        let name_field = object.get(entry..entry + 8).ok_or_else(|| {
            CeruneNativeAdapterError::AdaptationFailed("COFF symbol name is truncated".to_owned())
        })?;

        if name_field[..4] == [0, 0, 0, 0] {
            let offset =
                u32::from_le_bytes([name_field[4], name_field[5], name_field[6], name_field[7]])
                    as usize;

            if offset < 4 {
                return Err(CeruneNativeAdapterError::AdaptationFailed(
                    "COFF long symbol name has an invalid offset".to_owned(),
                ));
            }

            let start = string_table_offset.checked_add(offset).ok_or_else(|| {
                CeruneNativeAdapterError::AdaptationFailed(
                    "COFF symbol name offset overflow".to_owned(),
                )
            })?;

            if start >= string_table_end {
                return Err(CeruneNativeAdapterError::AdaptationFailed(
                    "COFF symbol name points outside the string table".to_owned(),
                ));
            }

            let bytes = &object[start..string_table_end];

            let end = bytes.iter().position(|byte| *byte == 0).ok_or_else(|| {
                CeruneNativeAdapterError::AdaptationFailed(
                    "COFF long symbol name is not terminated".to_owned(),
                )
            })?;

            return std::str::from_utf8(&bytes[..end])
                .map(str::to_owned)
                .map_err(|error| CeruneNativeAdapterError::AdaptationFailed(error.to_string()));
        }

        let end = name_field
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(name_field.len());

        std::str::from_utf8(&name_field[..end])
            .map(str::to_owned)
            .map_err(|error| CeruneNativeAdapterError::AdaptationFailed(error.to_string()))
    }

    #[cfg(target_os = "linux")]
    fn objcopy_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(path) = env::var_os("WHITEBASE_CERUNE_NATIVE_OBJCOPY") {
            candidates.push(path.into());
        }

        if let Some(path) = env::var_os("OBJCOPY") {
            candidates.push(path.into());
        }

        candidates.push(PathBuf::from("llvm-objcopy"));
        candidates.push(PathBuf::from("objcopy"));

        candidates
    }

    #[cfg(target_os = "linux")]
    fn find_objcopy() -> Result<PathBuf, CeruneNativeAdapterError> {
        objcopy_candidates()
            .into_iter()
            .find(|tool| {
                Command::new(tool)
                    .arg("--version")
                    .output()
                    .is_ok_and(|output| output.status.success())
            })
            .ok_or(CeruneNativeAdapterError::ObjcopyUnavailable)
    }

    fn compiler_candidates() -> Vec<PathBuf> {
        let mut candidates = Vec::new();

        if let Some(path) = env::var_os("WHITEBASE_CERUNE_NATIVE_CC") {
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

    fn find_compiler() -> Result<PathBuf, CeruneNativeAdapterError> {
        compiler_candidates()
            .into_iter()
            .find(|compiler| {
                Command::new(compiler)
                    .arg("--version")
                    .output()
                    .is_ok_and(|output| output.status.success())
            })
            .ok_or(CeruneNativeAdapterError::CompilerUnavailable)
    }

    fn link_shared_library(object: &Path, output: &Path) -> Result<(), CeruneNativeAdapterError> {
        let compiler = find_compiler()?;

        let mut command = Command::new(compiler);

        #[cfg(target_os = "windows")]
        {
            command
                .arg("--target=x86_64-pc-windows-msvc")
                .arg("-shared")
                .arg("-nostdlib")
                .arg("-fuse-ld=lld")
                .arg("-Wl,/noentry")
                .arg(format!(
                    "-Wl,/export:{WHITEBASE_ADD_SCALAR_F64_SYMBOL_NAME}"
                ));
        }

        #[cfg(target_os = "linux")]
        {
            command.arg("-shared").arg("-nostdlib");
        }

        let result = command
            .arg(object)
            .arg("-o")
            .arg(output)
            .output()
            .map_err(|error| CeruneNativeAdapterError::Io(error.to_string()))?;

        if !result.status.success() {
            return Err(CeruneNativeAdapterError::LinkFailed(
                String::from_utf8_lossy(&result.stderr).into_owned(),
            ));
        }

        Ok(())
    }

    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
    const fn host_target() -> CeruneNativeTarget {
        CeruneNativeTarget::X86_64PcWindowsMsvc
    }

    #[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
    const fn host_target() -> CeruneNativeTarget {
        CeruneNativeTarget::X86_64UnknownLinuxGnu
    }

    #[cfg(target_os = "windows")]
    fn object_name() -> &'static str {
        "cerune.obj"
    }

    #[cfg(target_os = "linux")]
    fn object_name() -> &'static str {
        "cerune.o"
    }

    #[cfg(target_os = "linux")]
    fn global_object_name() -> &'static str {
        "cerune-global.o"
    }

    #[cfg(target_os = "windows")]
    fn adapted_object_name() -> &'static str {
        "cerune-whitebase.obj"
    }

    #[cfg(target_os = "linux")]
    fn adapted_object_name() -> &'static str {
        "cerune-whitebase.o"
    }

    #[cfg(target_os = "windows")]
    fn shared_library_name() -> &'static str {
        "whitebase_cerune_native.dll"
    }

    #[cfg(target_os = "linux")]
    fn shared_library_name() -> &'static str {
        "libwhitebase_cerune_native.so"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CERUNE_ADD_SYMBOL: &[u8] = b"cerune_fn_add_0";

    fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }

    #[test]
    fn reports_target_triples() {
        assert_eq!(
            CeruneNativeTarget::X86_64PcWindowsMsvc.triple(),
            "x86_64-pc-windows-msvc"
        );

        assert_eq!(
            CeruneNativeTarget::X86_64UnknownLinuxGnu.triple(),
            "x86_64-unknown-linux-gnu"
        );
    }

    #[test]
    fn emits_windows_coff_object() {
        let object = emit_add_scalar_f64_object(CeruneNativeTarget::X86_64PcWindowsMsvc).unwrap();

        assert!(object.len() > 100);

        // AMD64 COFF machine type 0x8664, little endian.
        assert_eq!(&object[..2], &[0x64, 0x86]);

        assert!(contains_bytes(&object, CERUNE_ADD_SYMBOL,));
    }

    #[test]
    fn emits_linux_elf_object() {
        let object = emit_add_scalar_f64_object(CeruneNativeTarget::X86_64UnknownLinuxGnu).unwrap();

        assert!(object.len() > 64);

        assert_eq!(&object[..7], b"\x7fELF\x02\x01\x01");

        assert!(contains_bytes(&object, CERUNE_ADD_SYMBOL,));
    }

    #[test]
    fn emits_deterministic_native_objects() {
        for target in [
            CeruneNativeTarget::X86_64PcWindowsMsvc,
            CeruneNativeTarget::X86_64UnknownLinuxGnu,
        ] {
            let first = emit_add_scalar_f64_object(target).unwrap();

            let second = emit_add_scalar_f64_object(target).unwrap();

            assert_eq!(first, second);
        }
    }

    #[cfg(any(
        all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
        all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
    ))]
    #[test]
    fn prepares_and_executes_add_scalar_f64_native_object() {
        let adapter = CeruneNativeAdapter::new().unwrap();

        let result = adapter.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }
}
