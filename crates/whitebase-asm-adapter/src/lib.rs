//! Assembly計算バックエンドをRustから利用するためのAdapterです。

use whitebase_rust_backend::ArrayLengthError;

unsafe extern "C" {
    fn whitebase_asm_add_f32_scalar(
        lhs: *const f32,
        rhs: *const f32,
        output: *mut f32,
        length: usize,
    );

    fn whitebase_asm_add_f32_avx(lhs: *const f32, rhs: *const f32, output: *mut f32, length: usize);
}

/// Assembly Scalarバックエンドで`f32`配列を加算します。
pub fn add_f32_scalar(
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<(), ArrayLengthError> {
    validate_lengths(lhs, rhs, output)?;

    // SAFETY:
    // 各ポインターは有効なスライスから取得しており、
    // すべての配列長が一致することを事前に確認しています。
    unsafe {
        whitebase_asm_add_f32_scalar(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len());
    }

    Ok(())
}

/// 現在のCPUとOSでAVXを利用できるか返します。
#[must_use]
pub fn is_avx_available() -> bool {
    std::arch::is_x86_feature_detected!("avx")
}

/// Assembly AVXバックエンドで`f32`配列を加算します。
///
/// AVXを利用できない環境では`false`を返し、出力を変更しません。
pub fn add_f32_avx(lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<bool, ArrayLengthError> {
    validate_lengths(lhs, rhs, output)?;

    if !is_avx_available() {
        return Ok(false);
    }

    // SAFETY:
    // AVXの利用可能性と配列長を確認済みです。
    // 各ポインターは呼び出し中有効です。
    unsafe {
        whitebase_asm_add_f32_avx(lhs.as_ptr(), rhs.as_ptr(), output.as_mut_ptr(), lhs.len());
    }

    Ok(true)
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
