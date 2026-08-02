//! C++計算バックエンドをRustから利用するためのアダプターです。

#![cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]

use whitebase_rust_backend::ArrayLengthError;

unsafe extern "C" {
    fn whitebase_cpp_add_f32_scalar(
        lhs: *const f32,
        rhs: *const f32,
        output: *mut f32,
        length: usize,
    );

    fn whitebase_cpp_add_f64_array_scalar(
        lhs: *const f64,
        rhs: *const f64,
        output: *mut f64,
        length: usize,
    );

    fn whitebase_cpp_add_f64_scalar(lhs: f64, rhs: f64) -> f64;
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
unsafe extern "C" {
    fn whitebase_cpp_is_avx_available() -> i32;

    fn whitebase_cpp_add_f32_avx(
        lhs: *const f32,
        rhs: *const f32,
        output: *mut f32,
        length: usize,
    ) -> i32;

    fn whitebase_cpp_add_f64_array_avx(
        lhs: *const f64,
        rhs: *const f64,
        output: *mut f64,
        length: usize,
    ) -> i32;
}

/// C++ Scalarバックエンドで2つの`f32`配列を加算します。
///
/// # Errors
///
/// 入力配列と出力配列の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f32_scalar(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<(), ArrayLengthError> {
    validate_lengths(lhs, rhs, output)?;

    // SAFETY:
    // 各ポインターは有効なスライスから取得しており、
    // 事前にすべての長さが一致することを確認しています。
    unsafe {
        whitebase_cpp_add_f32_scalar(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len());
    }

    Ok(())
}

/// C++ Scalarバックエンドで2つの`f64`配列を加算します。
///
/// # Errors
///
/// 入力配列と出力配列の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f64_array_scalar(
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<(), ArrayLengthError> {
    validate_lengths(lhs, rhs, output)?;

    // SAFETY:
    // 各ポインターは有効なスライスから取得しており、
    // 事前にすべての長さが一致することを確認しています。
    unsafe {
        whitebase_cpp_add_f64_array_scalar(
            lhs.as_ptr(),
            rhs.as_ptr(),
            output.as_mut_ptr(),
            lhs.len(),
        );
    }

    Ok(())
}

/// C++ Scalarバックエンドで2つの`f64`値を加算します。
#[must_use]
pub fn add_f64_scalar(lhs: f64, rhs: f64) -> f64 {
    // SAFETY:
    // 値渡しの`f64`を受け取り、値渡しの`f64`を返すC ABI関数です。
    unsafe { whitebase_cpp_add_f64_scalar(lhs, rhs) }
}

/// C++ AVXバックエンドが現在の環境で利用可能か返します。
///
/// Linux版は現在Scalar実装のみのため、常に`false`を返します。
#[must_use]
pub fn is_avx_available() -> bool {
    platform_is_avx_available()
}

/// C++ AVXバックエンドで2つの`f32`配列を加算します。
///
/// 戻り値が`true`ならAVX処理が実行されています。AVXを利用できない
/// 環境では`false`を返し、出力は変更されません。
///
/// # Errors
///
/// 入力配列と出力配列の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f32_avx(lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<bool, ArrayLengthError> {
    validate_lengths(lhs, rhs, output)?;

    Ok(platform_add_f32_avx(lhs, rhs, output))
}

/// C++ AVXバックエンドで2つの`f64`配列を加算します。
///
/// 戻り値が`true`ならAVX処理が実行されています。AVXを利用できない
/// 環境では`false`を返し、出力は変更されません。
///
/// # Errors
///
/// 入力配列と出力配列の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f64_array_avx(
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<bool, ArrayLengthError> {
    validate_lengths(lhs, rhs, output)?;

    Ok(platform_add_f64_array_avx(lhs, rhs, output))
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn platform_is_avx_available() -> bool {
    // SAFETY:
    // 引数を取らず、CPUとOSの対応状況を確認するだけの関数です。
    unsafe { whitebase_cpp_is_avx_available() != 0 }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
const fn platform_is_avx_available() -> bool {
    false
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn platform_add_f32_avx(lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> bool {
    // SAFETY:
    // 各ポインターは有効なスライスから取得しており、
    // 呼び出し前にすべての長さが一致することを確認しています。
    unsafe {
        whitebase_cpp_add_f32_avx(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len()) != 0
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
fn platform_add_f32_avx(_lhs: &[f32], _rhs: &[f32], _output: &mut [f32]) -> bool {
    false
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn platform_add_f64_array_avx(lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> bool {
    // SAFETY:
    // 各ポインターは有効なスライスから取得しており、
    // 呼び出し前にすべての長さが一致することを確認しています。
    unsafe {
        whitebase_cpp_add_f64_array_avx(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len())
            != 0
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
fn platform_add_f64_array_avx(_lhs: &[f64], _rhs: &[f64], _output: &mut [f64]) -> bool {
    false
}

fn validate_lengths<T>(lhs: &[T], rhs: &[T], output: &[T]) -> Result<(), ArrayLengthError> {
    if lhs.len() != rhs.len() || lhs.len() != output.len() {
        return Err(ArrayLengthError::new(lhs.len(), rhs.len(), output.len()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_f64_scalars_with_ieee_754_rounding() {
        let result = add_f64_scalar(0.1, 0.2);

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }

    #[test]
    fn adds_f64_arrays_with_ieee_754_rounding() {
        let lhs = [0.1, 1.0, 2.0, 3.0, 4.0, 5.0];
        let rhs = [0.2, 10.0, 20.0, 30.0, 40.0, 50.0];
        let mut output = [0.0; 6];

        add_f64_array_scalar(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(output[0].to_bits(), 0x3fd3_3333_3333_3334);
        assert_eq!(output[1..], [11.0, 22.0, 33.0, 44.0, 55.0]);
    }

    #[test]
    fn rejects_different_lengths() {
        let lhs = [1.0, 2.0];
        let rhs = [3.0];
        let mut output = [0.0; 2];

        assert_eq!(
            add_f32_scalar(&lhs, &rhs, &mut output),
            Err(ArrayLengthError::new(2, 1, 2))
        );
    }
}
