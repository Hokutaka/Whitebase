//! Windows GNU Native DLLをRustから利用するためのAdapterです。

#![cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]

use std::{
    env,
    error::Error,
    ffi::{CStr, c_char, c_void},
    fmt,
    mem::{size_of, transmute_copy},
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
    sync::OnceLock,
};

const DLL_NAME: &str = "whitebase_windows_gnu_native.dll";
const DLL_PATH_ENVIRONMENT_VARIABLE: &str = "WHITEBASE_WINDOWS_GNU_NATIVE_DLL";

type Module = *mut c_void;
type FarProc = Option<unsafe extern "system" fn() -> isize>;

type AddF32Array = unsafe extern "C" fn(*const f32, *const f32, *mut f32, usize);
type AddF64Array = unsafe extern "C" fn(*const f64, *const f64, *mut f64, usize);
type AddF64Scalar = unsafe extern "C" fn(f64, f64) -> f64;
type SumF64Scalar = unsafe extern "C" fn(*const f64, usize) -> f64;
type IsAvxAvailable = unsafe extern "C" fn() -> i32;
type AddF32ArrayAvx = unsafe extern "C" fn(*const f32, *const f32, *mut f32, usize) -> i32;
type AddF64ArrayAvx = unsafe extern "C" fn(*const f64, *const f64, *mut f64, usize) -> i32;
type SumF64Avx = unsafe extern "C" fn(*const f64, usize, *mut f64) -> i32;

#[link(name = "kernel32")]
unsafe extern "system" {
    #[link_name = "LoadLibraryW"]
    fn load_library_w(file_name: *const u16) -> Module;

    #[link_name = "FreeLibrary"]
    fn free_library(module: Module) -> i32;

    #[link_name = "GetProcAddress"]
    fn get_proc_address(module: Module, procedure_name: *const c_char) -> FarProc;
}

/// Windows GNU Native Adapterで発生するエラーです。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdapterError {
    /// 入力配列と出力配列の長さが一致しません。
    ArrayLengthMismatch {
        lhs_length: usize,
        rhs_length: usize,
        output_length: usize,
    },

    /// DLLまたは必要なエクスポート関数を読み込めませんでした。
    NativeLibraryUnavailable { message: String },
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArrayLengthMismatch {
                lhs_length,
                rhs_length,
                output_length,
            } => write!(
                formatter,
                "array length mismatch: lhs={lhs_length}, rhs={rhs_length}, output={output_length}",
            ),
            Self::NativeLibraryUnavailable { message } => formatter.write_str(message),
        }
    }
}

impl Error for AdapterError {}

struct ModuleHandle(usize);

impl ModuleHandle {
    fn load(path: &Path) -> Result<Self, AdapterError> {
        let wide_path = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();

        // SAFETY:
        // `wide_path`はNUL終端されたUTF-16文字列であり、呼び出し中有効です。
        let module = unsafe { load_library_w(wide_path.as_ptr()) };

        if module.is_null() {
            return Err(AdapterError::NativeLibraryUnavailable {
                message: format!(
                    "failed to load {}: {}",
                    path.display(),
                    std::io::Error::last_os_error(),
                ),
            });
        }

        Ok(Self(module as usize))
    }

    fn as_raw(&self) -> Module {
        self.0 as Module
    }
}

impl Drop for ModuleHandle {
    fn drop(&mut self) {
        // SAFETY:
        // ハンドルは`LoadLibraryW`が返したもので、このDropで一度だけ解放します。
        unsafe {
            free_library(self.as_raw());
        }
    }
}

struct NativeApi {
    _module: ModuleHandle,
    cpp_add_f32_scalar: AddF32Array,
    cpp_add_f64_array_scalar: AddF64Array,
    cpp_add_f64_scalar: AddF64Scalar,
    cpp_sum_f64_scalar: SumF64Scalar,
    cpp_is_avx_available: IsAvxAvailable,
    cpp_add_f32_avx: AddF32ArrayAvx,
    cpp_add_f64_array_avx: AddF64ArrayAvx,
    cpp_sum_f64_avx: SumF64Avx,
    assembly_add_f32_scalar: AddF32Array,
    assembly_add_f64_array_scalar: AddF64Array,
    assembly_add_f64_scalar: AddF64Scalar,
    assembly_sum_f64_scalar: SumF64Scalar,
    assembly_add_f32_avx: AddF32ArrayAvx,
    assembly_add_f64_array_avx: AddF64ArrayAvx,
    assembly_sum_f64_avx: SumF64Avx,
}

impl NativeApi {
    fn load() -> Result<Self, AdapterError> {
        let mut errors = Vec::new();

        for path in dll_candidates() {
            match Self::load_from(&path) {
                Ok(api) => return Ok(api),
                Err(error) => errors.push(error.to_string()),
            }
        }

        Err(AdapterError::NativeLibraryUnavailable {
            message: format!(
                "{DLL_NAME} is unavailable. Set {DLL_PATH_ENVIRONMENT_VARIABLE} to an explicit DLL path if needed. Attempts: {}",
                errors.join(" | "),
            ),
        })
    }

    fn load_from(path: &Path) -> Result<Self, AdapterError> {
        let module = ModuleHandle::load(path)?;

        // SAFETY:
        // 各シンボル名と関数型はCヘッダーおよびDEFファイルの契約と一致しています。
        unsafe {
            Ok(Self {
                cpp_add_f32_scalar: load_symbol(&module, c"whitebase_gnu_cpp_add_f32_scalar")?,
                cpp_add_f64_array_scalar: load_symbol(
                    &module,
                    c"whitebase_gnu_cpp_add_f64_array_scalar",
                )?,
                cpp_add_f64_scalar: load_symbol(&module, c"whitebase_gnu_cpp_add_f64_scalar")?,
                cpp_sum_f64_scalar: load_symbol(&module, c"whitebase_gnu_cpp_sum_f64_scalar")?,
                cpp_is_avx_available: load_symbol(&module, c"whitebase_gnu_cpp_is_avx_available")?,
                cpp_add_f32_avx: load_symbol(&module, c"whitebase_gnu_cpp_add_f32_avx")?,
                cpp_add_f64_array_avx: load_symbol(
                    &module,
                    c"whitebase_gnu_cpp_add_f64_array_avx",
                )?,
                cpp_sum_f64_avx: load_symbol(&module, c"whitebase_gnu_cpp_sum_f64_avx")?,
                assembly_add_f32_scalar: load_symbol(&module, c"whitebase_gnu_asm_add_f32_scalar")?,
                assembly_add_f64_array_scalar: load_symbol(
                    &module,
                    c"whitebase_gnu_asm_add_f64_array_scalar",
                )?,
                assembly_add_f64_scalar: load_symbol(&module, c"whitebase_gnu_asm_add_f64_scalar")?,
                assembly_sum_f64_scalar: load_symbol(&module, c"whitebase_gnu_asm_sum_f64_scalar")?,
                assembly_add_f32_avx: load_symbol(&module, c"whitebase_gnu_asm_add_f32_avx")?,
                assembly_add_f64_array_avx: load_symbol(
                    &module,
                    c"whitebase_gnu_asm_add_f64_array_avx",
                )?,
                assembly_sum_f64_avx: load_symbol(&module, c"whitebase_gnu_asm_sum_f64_avx")?,
                _module: module,
            })
        }
    }
}

static NATIVE_API: OnceLock<Result<NativeApi, AdapterError>> = OnceLock::new();

fn native_api() -> Result<&'static NativeApi, AdapterError> {
    NATIVE_API
        .get_or_init(NativeApi::load)
        .as_ref()
        .map_err(Clone::clone)
}

unsafe fn load_symbol<T: Copy>(
    module: &ModuleHandle,
    name: &'static CStr,
) -> Result<T, AdapterError> {
    debug_assert_eq!(size_of::<T>(), size_of::<FarProc>());

    // SAFETY:
    // モジュールハンドルは有効で、`name`はNUL終端された静的文字列です。
    let address = unsafe { get_proc_address(module.as_raw(), name.as_ptr()) };

    let Some(address) = address else {
        return Err(AdapterError::NativeLibraryUnavailable {
            message: format!(
                "failed to load symbol {}: {}",
                name.to_string_lossy(),
                std::io::Error::last_os_error(),
            ),
        });
    };

    // SAFETY:
    // 呼び出し元がシンボルのC ABI関数型を指定し、サイズ一致も確認しています。
    Ok(unsafe { transmute_copy::<FarProc, T>(&Some(address)) })
}

fn dll_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = env::var_os(DLL_PATH_ENVIRONMENT_VARIABLE) {
        push_unique_path(&mut candidates, absolute_path(PathBuf::from(path)));
    }

    if let Ok(executable) = env::current_exe()
        && let Some(directory) = executable.parent()
    {
        push_unique_path(&mut candidates, directory.join(DLL_NAME));
    }

    let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("adapter crate must be inside the repository");
    let native_build_root = repository_root
        .join("native")
        .join("Whitebase.Windows.Gnu")
        .join("build");

    let profiles = if cfg!(debug_assertions) {
        ["Debug", "Release"]
    } else {
        ["Release", "Debug"]
    };

    for profile in profiles {
        push_unique_path(
            &mut candidates,
            native_build_root.join(profile).join(DLL_NAME),
        );
    }

    candidates
}

fn absolute_path(path: PathBuf) -> PathBuf {
    if path.is_absolute() {
        path
    } else {
        env::current_dir().map_or(path.clone(), |directory| directory.join(path))
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn validate_lengths<T>(lhs: &[T], rhs: &[T], output: &[T]) -> Result<(), AdapterError> {
    if lhs.len() != rhs.len() || lhs.len() != output.len() {
        return Err(AdapterError::ArrayLengthMismatch {
            lhs_length: lhs.len(),
            rhs_length: rhs.len(),
            output_length: output.len(),
        });
    }

    Ok(())
}

/// Windows GNU Native DLLを読み込めるか返します。
#[must_use]
pub fn is_available() -> bool {
    native_api().is_ok()
}

/// GCC AVXバックエンドを利用できるか返します。
#[must_use]
pub fn is_cpp_avx_available() -> bool {
    let Ok(api) = native_api() else {
        return false;
    };

    // SAFETY:
    // 引数を取らず、CPUとOSのAVX対応状況を返すC ABI関数です。
    unsafe { (api.cpp_is_avx_available)() != 0 }
}

/// NASM AVXバックエンドを利用できるか返します。
#[must_use]
pub fn is_assembly_avx_available() -> bool {
    is_available() && std::arch::is_x86_feature_detected!("avx")
}

/// GCC Scalarバックエンドで`f32`配列を加算します。
pub fn cpp_add_f32_scalar(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<(), AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、各ポインターは呼び出し中有効です。
    unsafe {
        (api.cpp_add_f32_scalar)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len());
    }

    Ok(())
}

/// GCC Scalarバックエンドで`f64`配列を加算します。
pub fn cpp_add_f64_array_scalar(
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<(), AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、各ポインターは呼び出し中有効です。
    unsafe {
        (api.cpp_add_f64_array_scalar)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len());
    }

    Ok(())
}

/// GCC Scalarバックエンドで2つの`f64`値を加算します。
pub fn cpp_add_f64_scalar(lhs: f64, rhs: f64) -> Result<f64, AdapterError> {
    let api = native_api()?;

    // SAFETY:
    // 値渡しの`f64`を受け取り、値渡しの`f64`を返すC ABI関数です。
    Ok(unsafe { (api.cpp_add_f64_scalar)(lhs, rhs) })
}

/// GCC Scalarバックエンドで`f64`配列の要素を合計します。
pub fn cpp_sum_f64_scalar(input: &[f64]) -> Result<f64, AdapterError> {
    let api = native_api()?;

    // SAFETY:
    // `input`は呼び出し中有効で、Native側は指定された長さの範囲だけを読み取ります。
    Ok(unsafe { (api.cpp_sum_f64_scalar)(input.as_ptr(), input.len()) })
}

/// GCC AVXバックエンドで`f32`配列を加算します。
pub fn cpp_add_f32_avx(lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<bool, AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、各ポインターは呼び出し中有効です。
    let executed = unsafe {
        (api.cpp_add_f32_avx)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len())
    };

    Ok(executed != 0)
}

/// GCC AVXバックエンドで`f64`配列を加算します。
pub fn cpp_add_f64_array_avx(
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<bool, AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、各ポインターは呼び出し中有効です。
    let executed = unsafe {
        (api.cpp_add_f64_array_avx)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len())
    };

    Ok(executed != 0)
}

/// GCC AVXバックエンドで`f64`配列の要素を合計します。
///
/// AVXを利用できない場合は`None`を返します。
pub fn cpp_sum_f64_avx(input: &[f64]) -> Result<Option<f64>, AdapterError> {
    let api = native_api()?;
    let mut output = 0.0;

    // SAFETY:
    // `input`と`output`は呼び出し中有効で、Native側は指定された長さの範囲だけを読み取ります。
    let executed = unsafe { (api.cpp_sum_f64_avx)(input.as_ptr(), input.len(), &mut output) };

    Ok((executed != 0).then_some(output))
}

/// NASM Scalarバックエンドで`f32`配列を加算します。
pub fn assembly_add_f32_scalar(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<(), AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、各ポインターは呼び出し中有効です。
    unsafe {
        (api.assembly_add_f32_scalar)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len());
    }

    Ok(())
}

/// NASM Scalarバックエンドで`f64`配列を加算します。
pub fn assembly_add_f64_array_scalar(
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<(), AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、各ポインターは呼び出し中有効です。
    unsafe {
        (api.assembly_add_f64_array_scalar)(
            lhs.as_ptr(),
            rhs.as_ptr(),
            output.as_mut_ptr(),
            lhs.len(),
        );
    }

    Ok(())
}

/// NASM Scalarバックエンドで2つの`f64`値を加算します。
pub fn assembly_add_f64_scalar(lhs: f64, rhs: f64) -> Result<f64, AdapterError> {
    let api = native_api()?;

    // SAFETY:
    // 値渡しの`f64`を受け取り、値渡しの`f64`を返すC ABI関数です。
    Ok(unsafe { (api.assembly_add_f64_scalar)(lhs, rhs) })
}

/// NASM Scalarバックエンドで`f64`配列の要素を合計します。
pub fn assembly_sum_f64_scalar(input: &[f64]) -> Result<f64, AdapterError> {
    let api = native_api()?;

    // SAFETY:
    // `input`は呼び出し中有効で、Native側は指定された長さの範囲だけを読み取ります。
    Ok(unsafe { (api.assembly_sum_f64_scalar)(input.as_ptr(), input.len()) })
}

/// NASM AVXバックエンドで`f32`配列を加算します。
pub fn assembly_add_f32_avx(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<bool, AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、NASM側もAVXの利用可能性を確認します。
    let executed = unsafe {
        (api.assembly_add_f32_avx)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len())
    };

    Ok(executed != 0)
}

/// NASM AVXバックエンドで`f64`配列を加算します。
pub fn assembly_add_f64_array_avx(
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<bool, AdapterError> {
    validate_lengths(lhs, rhs, output)?;
    let api = native_api()?;

    // SAFETY:
    // 配列長は一致し、NASM側もAVXの利用可能性を確認します。
    let executed = unsafe {
        (api.assembly_add_f64_array_avx)(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len())
    };

    Ok(executed != 0)
}

/// NASM AVXバックエンドで`f64`配列の要素を合計します。
///
/// AVXを利用できない場合は`None`を返します。
pub fn assembly_sum_f64_avx(input: &[f64]) -> Result<Option<f64>, AdapterError> {
    let api = native_api()?;
    let mut output = 0.0;

    // SAFETY:
    // `input`と`output`は呼び出し中有効で、NASM側もAVXの利用可能性を確認します。
    let executed = unsafe { (api.assembly_sum_f64_avx)(input.as_ptr(), input.len(), &mut output) };

    Ok((executed != 0).then_some(output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_different_lengths_before_loading_the_dll() {
        let lhs = [1.0, 2.0];
        let rhs = [3.0];
        let mut output = [0.0; 2];

        assert_eq!(
            cpp_add_f32_scalar(&lhs, &rhs, &mut output),
            Err(AdapterError::ArrayLengthMismatch {
                lhs_length: 2,
                rhs_length: 1,
                output_length: 2,
            })
        );
    }

    #[test]
    fn repository_candidates_include_both_build_profiles() {
        let candidates = dll_candidates();

        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("adapter crate must be inside the repository");

        let native_build_root = repository_root
            .join("native")
            .join("Whitebase.Windows.Gnu")
            .join("build");

        assert!(candidates.contains(&native_build_root.join("Debug").join(DLL_NAME)));

        assert!(candidates.contains(&native_build_root.join("Release").join(DLL_NAME)));
    }
}
