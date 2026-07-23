//! C++計算バックエンドをRustから利用するためのアダプターです。

use whitebase_rust_backend::ArrayLengthError;

unsafe extern "C" {
    fn whitebase_cpp_add_f32_scalar(
        lhs: *const f32,
        rhs: *const f32,
        output: *mut f32,
        length: usize,
    );

    fn whitebase_cpp_is_avx_available() -> i32;

    fn whitebase_cpp_add_f32_avx(
        lhs: *const f32,
        rhs: *const f32,
        output: *mut f32,
        length: usize,
    ) -> i32;
}

/// C++ Scalarバックエンドで2つの`f32`配列を加算します.
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

/// C++ AVXバックエンドが現在の環境で利用可能か返します。
#[must_use]
pub fn is_avx_available() -> bool {
    // SAFETY:
    // 引数を取らず、CPUとOSの対応状況を確認するだけの関数です。
    unsafe { whitebase_cpp_is_avx_available() != 0 }
}

/// C++ AVXバックエンドで2つの`f32`配列を加算します.
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

    // SAFETY:
    // 各ポインターは有効なスライスから取得しており、
    // 事前にすべての長さが一致することを確認しています。
    let executed = unsafe {
        whitebase_cpp_add_f32_avx(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len())
    };

    Ok(executed != 0)
}

fn validate_lengths(lhs: &[f32], rhs: &[f32], output: &[f32]) -> Result<(), ArrayLengthError> {
    if lhs.len() != rhs.len() || lhs.len() != output.len() {
        return Err(ArrayLengthError::new(lhs.len(), rhs.len(), output.len()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
