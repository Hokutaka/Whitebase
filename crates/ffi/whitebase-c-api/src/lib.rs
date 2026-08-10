//! Whitebase CoreをC ABIとして公開します。

use std::mem::{align_of, size_of};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr::NonNull;
use std::slice;
use std::sync::OnceLock;

use whitebase_core::{BackendKind, ComputeError, OperationKind, Whitebase};

const STATUS_OK: i32 = 0;
const STATUS_INVALID_ARGUMENT: i32 = 1;
const STATUS_UNKNOWN_BACKEND: i32 = 2;
const STATUS_BACKEND_NOT_REGISTERED: i32 = 3;
const STATUS_BACKEND_UNAVAILABLE: i32 = 4;
const STATUS_OPERATION_UNSUPPORTED: i32 = 5;
const STATUS_BACKEND_FAILURE: i32 = 6;
const STATUS_INTERNAL_PANIC: i32 = 7;
const STATUS_UNKNOWN_OPERATION: i32 = 8;

static WHITEBASE: OnceLock<Whitebase> = OnceLock::new();

fn whitebase() -> &'static Whitebase {
    WHITEBASE.get_or_init(Whitebase::new)
}

fn backend_from_id(id: u32) -> Result<BackendKind, i32> {
    match id {
        0 => Ok(BackendKind::RustScalar),
        1 => Ok(BackendKind::RustSimd),
        2 => Ok(BackendKind::CppScalar),
        3 => Ok(BackendKind::CppAvx),
        4 => Ok(BackendKind::AssemblyScalar),
        5 => Ok(BackendKind::AssemblyAvx),
        6 => Ok(BackendKind::WindowsGnuCppScalar),
        7 => Ok(BackendKind::WindowsGnuCppAvx),
        8 => Ok(BackendKind::WindowsGnuAssemblyScalar),
        9 => Ok(BackendKind::WindowsGnuAssemblyAvx),
        _ => Err(STATUS_UNKNOWN_BACKEND),
    }
}

fn operation_from_id(id: u32) -> Result<OperationKind, i32> {
    match id {
        0 => Ok(OperationKind::AddF32),
        1 => Ok(OperationKind::AddF64),
        2 => Ok(OperationKind::AddScalarF64),
        3 => Ok(OperationKind::SumF64),
        _ => Err(STATUS_UNKNOWN_OPERATION),
    }
}

fn status_from_compute_error(error: ComputeError) -> i32 {
    match error {
        ComputeError::LengthMismatch { .. } => STATUS_INVALID_ARGUMENT,
        ComputeError::BackendNotRegistered { .. } => STATUS_BACKEND_NOT_REGISTERED,
        ComputeError::BackendUnavailable { .. } => STATUS_BACKEND_UNAVAILABLE,
        ComputeError::OperationUnsupported { .. } => STATUS_OPERATION_UNSUPPORTED,
        ComputeError::BackendFailure { .. } => STATUS_BACKEND_FAILURE,
    }
}

fn ffi_status(operation: impl FnOnce() -> Result<(), i32>) -> i32 {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(Ok(())) => STATUS_OK,
        Ok(Err(status)) => status,
        Err(_) => STATUS_INTERNAL_PANIC,
    }
}

fn validate_region<T>(address: usize, length: usize) -> Result<(), i32> {
    if !address.is_multiple_of(align_of::<T>()) {
        return Err(STATUS_INVALID_ARGUMENT);
    }

    let maximum_length = (isize::MAX as usize) / size_of::<T>();
    if length > maximum_length {
        return Err(STATUS_INVALID_ARGUMENT);
    }

    Ok(())
}

unsafe fn input_slice<'a, T>(pointer: *const T, length: usize) -> Result<&'a [T], i32> {
    if length == 0 {
        // SAFETY: dangling is non-null and aligned; a zero-length slice does not access memory.
        return Ok(unsafe { slice::from_raw_parts(NonNull::<T>::dangling().as_ptr(), 0) });
    }
    if pointer.is_null() {
        return Err(STATUS_INVALID_ARGUMENT);
    }

    validate_region::<T>(pointer as usize, length)?;

    // SAFETY: the C caller promises that `pointer` references `length` readable T values.
    Ok(unsafe { slice::from_raw_parts(pointer, length) })
}

unsafe fn output_slice<'a, T>(pointer: *mut T, length: usize) -> Result<&'a mut [T], i32> {
    if length == 0 {
        // SAFETY: dangling is non-null and aligned; a zero-length slice does not access memory.
        return Ok(unsafe { slice::from_raw_parts_mut(NonNull::<T>::dangling().as_ptr(), 0) });
    }
    if pointer.is_null() {
        return Err(STATUS_INVALID_ARGUMENT);
    }

    validate_region::<T>(pointer as usize, length)?;

    // SAFETY: the C caller promises that `pointer` references `length` writable T values
    // and that no overlapping mutable access exists for the duration of this call.
    Ok(unsafe { slice::from_raw_parts_mut(pointer, length) })
}

unsafe fn write_scalar<T>(pointer: *mut T, value: T) -> Result<(), i32> {
    if pointer.is_null() || !(pointer as usize).is_multiple_of(align_of::<T>()) {
        return Err(STATUS_INVALID_ARGUMENT);
    }

    // SAFETY: the C caller promises that `pointer` references writable storage for one T.
    unsafe { pointer.write(value) };
    Ok(())
}

/// 指定したバックエンドが現在の環境で利用可能か返します。
///
/// # Safety
///
/// `available`は書き込み可能な`i32` 1要素を指す必要があります。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whitebase_backend_is_available(backend: u32, available: *mut i32) -> i32 {
    ffi_status(|| {
        let kind = backend_from_id(backend)?;
        let info = whitebase()
            .backend_info(kind)
            .map_err(status_from_compute_error)?;

        // SAFETY: validity is part of this exported function's C ABI contract.
        unsafe { write_scalar(available, if info.available { 1 } else { 0 }) }
    })
}

/// 指定したバックエンドがOperationをサポートするか返します。
///
/// # Safety
///
/// `supported`は書き込み可能な`i32` 1要素を指す必要があります。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whitebase_backend_supports(
    backend: u32,
    operation: u32,
    supported: *mut i32,
) -> i32 {
    ffi_status(|| {
        let kind = backend_from_id(backend)?;
        let operation = operation_from_id(operation)?;
        let info = whitebase()
            .backend_info(kind)
            .map_err(status_from_compute_error)?;

        // SAFETY: validity is part of this exported function's C ABI contract.
        unsafe {
            write_scalar(
                supported,
                if info.capabilities.supports(operation) {
                    1
                } else {
                    0
                },
            )
        }
    })
}

/// 指定したバックエンドで`f32`配列を要素ごとに加算します。
///
/// # Safety
///
/// `lhs`と`rhs`はそれぞれ`length`要素を読み取り可能で、`output`は`length`要素を
/// 書き込み可能である必要があります。
/// `output`が指す領域は`lhs`および`rhs`が指す領域と重なってはいけません。
/// `length == 0`の場合はnull pointerを許容します。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whitebase_add_f32(
    backend: u32,
    lhs: *const f32,
    rhs: *const f32,
    output: *mut f32,
    length: usize,
) -> i32 {
    ffi_status(|| {
        let kind = backend_from_id(backend)?;

        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let lhs = unsafe { input_slice(lhs, length)? };
        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let rhs = unsafe { input_slice(rhs, length)? };
        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let output = unsafe { output_slice(output, length)? };

        whitebase()
            .add_f32(kind, lhs, rhs, output)
            .map_err(status_from_compute_error)
    })
}

/// 指定したバックエンドで`f64`配列を要素ごとに加算します。
///
/// # Safety
///
/// `lhs`と`rhs`はそれぞれ`length`要素を読み取り可能で、`output`は`length`要素を
/// 書き込み可能である必要があります。`length == 0`の場合はnull pointerを許容します。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whitebase_add_f64(
    backend: u32,
    lhs: *const f64,
    rhs: *const f64,
    output: *mut f64,
    length: usize,
) -> i32 {
    ffi_status(|| {
        let kind = backend_from_id(backend)?;

        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let lhs = unsafe { input_slice(lhs, length)? };
        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let rhs = unsafe { input_slice(rhs, length)? };
        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let output = unsafe { output_slice(output, length)? };

        whitebase()
            .add_f64(kind, lhs, rhs, output)
            .map_err(status_from_compute_error)
    })
}

/// 指定したバックエンドで2つの`f64`スカラー値を加算します。
///
/// # Safety
///
/// `output`は書き込み可能な`f64` 1要素を指す必要があります。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whitebase_add_scalar_f64(
    backend: u32,
    lhs: f64,
    rhs: f64,
    output: *mut f64,
) -> i32 {
    ffi_status(|| {
        let kind = backend_from_id(backend)?;
        let result = whitebase()
            .add_scalar_f64(kind, lhs, rhs)
            .map_err(status_from_compute_error)?;

        // SAFETY: validity is part of this exported function's C ABI contract.
        unsafe { write_scalar(output, result) }
    })
}

/// 指定したバックエンドで`f64`配列の要素を合計します。
///
/// # Safety
///
/// `input`は`length`要素を読み取り可能で、`output`は書き込み可能な`f64` 1要素を
/// 指す必要があります。
/// `output`が指す領域は`input`が指す領域と重なってはいけません。
/// `length == 0`の場合、`input`はnull pointerでも構いません。
#[unsafe(no_mangle)]
pub unsafe extern "C" fn whitebase_sum_f64(
    backend: u32,
    input: *const f64,
    length: usize,
    output: *mut f64,
) -> i32 {
    ffi_status(|| {
        let kind = backend_from_id(backend)?;

        // SAFETY: pointer validity is part of this exported function's C ABI contract.
        let input = unsafe { input_slice(input, length)? };
        let result = whitebase()
            .sum_f64(kind, input)
            .map_err(status_from_compute_error)?;

        // SAFETY: validity is part of this exported function's C ABI contract.
        unsafe { write_scalar(output, result) }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backend_is_available(backend: u32) -> bool {
        let mut available = 0;

        // SAFETY: availableは書き込み可能なi32 1要素を指しています。
        let status = unsafe { whitebase_backend_is_available(backend, &raw mut available) };

        assert_eq!(status, STATUS_OK);

        available != 0
    }

    #[test]
    fn rust_scalar_sum_runs_through_c_api_and_core() {
        let input = [1.0, 2.0, 3.0, 4.0];
        let mut output = -1.0;

        // SAFETY: all pointers refer to valid storage for the supplied lengths.
        let status = unsafe { whitebase_sum_f64(0, input.as_ptr(), input.len(), &raw mut output) };

        assert_eq!(status, STATUS_OK);
        assert_eq!(output, 10.0);
    }

    #[test]
    fn cpp_scalar_sum_runs_through_c_api_and_core() {
        if !backend_is_available(2) {
            return;
        }

        let input = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut output = -1.0;

        // SAFETY: all pointers refer to valid storage for the supplied lengths.
        let status = unsafe { whitebase_sum_f64(2, input.as_ptr(), input.len(), &raw mut output) };

        assert_eq!(status, STATUS_OK);
        assert_eq!(output, 15.0);
    }

    #[test]
    fn assembly_scalar_sum_runs_through_c_api_and_core() {
        if !backend_is_available(4) {
            return;
        }

        let input = [1.0, 2.0, 3.0, 4.0, 5.0];
        let mut output = -1.0;

        // SAFETY: inputとoutputは指定した長さに対して有効な領域を指しています。
        let status = unsafe { whitebase_sum_f64(4, input.as_ptr(), input.len(), &raw mut output) };

        assert_eq!(status, STATUS_OK);
        assert_eq!(output, 15.0);
    }

    #[test]
    fn rejects_unknown_backend_without_writing_output() {
        let input = [1.0];
        let mut output = -1234.0;

        // SAFETY: inputとoutputは指定した長さに対して有効な領域を指しています。
        let status =
            unsafe { whitebase_sum_f64(u32::MAX, input.as_ptr(), input.len(), &raw mut output) };

        assert_eq!(status, STATUS_UNKNOWN_BACKEND);
        assert_eq!(output, -1234.0);
    }

    #[test]
    fn reports_sum_capability_through_core() {
        let mut supported = 0;

        // SAFETY: `supported` points to writable storage.
        let status = unsafe { whitebase_backend_supports(4, 3, &raw mut supported) };

        assert_eq!(status, STATUS_OK);
        assert_eq!(supported, 1);
    }
}
