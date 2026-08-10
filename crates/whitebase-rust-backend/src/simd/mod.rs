use crate::ArrayLengthError;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "wasm32")]
mod wasm32;

#[cfg(target_arch = "x86_64")]
mod x86_64;

/// 現在の実行環境でAVXが利用できるかを返します。
#[must_use]
pub fn is_avx_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        std::arch::is_x86_feature_detected!("avx")
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        false
    }
}

/// 現在の実行環境でSIMD実装が利用できるかを返します。
#[must_use]
pub fn is_simd_available() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        x86_64::is_avx_available()
    }

    #[cfg(target_arch = "wasm32")]
    {
        wasm32::is_available()
    }

    #[cfg(target_arch = "aarch64")]
    {
        aarch64::is_neon_available()
    }

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "wasm32",
        target_arch = "aarch64"
    )))]
    {
        false
    }
}

/// 2つの`f32`配列を、SIMDを利用して要素ごとに加算します。
///
/// x86_64環境でAVXが利用できる場合はAVX実装を使用します。
/// AVXを利用できない環境ではScalar実装へフォールバックします。
///
/// # Errors
///
/// `lhs`、`rhs`、`output`の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f32(lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ArrayLengthError> {
    if lhs.len() != rhs.len() || lhs.len() != output.len() {
        return Err(ArrayLengthError::new(lhs.len(), rhs.len(), output.len()));
    }

    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY:
        // WASM版はsimd128を有効化した専用関数内で実行します。
        unsafe {
            wasm32::add_f32(lhs, rhs, output);
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if x86_64::is_avx_available() {
                // SAFETY:
                // 実行前にAVX対応を確認しており、
                // 配列長も検証済みです。
                unsafe {
                    x86_64::add_f32(lhs, rhs, output);
                }

                return Ok(());
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if aarch64::is_neon_available() {
                // SAFETY:
                // 実行前にNEON対応を確認しており、
                // 配列長も検証済みです。
                unsafe {
                    aarch64::add_f32(lhs, rhs, output);
                }

                return Ok(());
            }
        }

        crate::scalar::add_f32(lhs, rhs, output)
    }
}

/// 2つの`f64`配列を、SIMDを利用して要素ごとに加算します。
///
/// x86_64環境でAVXが利用できる場合はAVX実装を使用します。
/// AVXを利用できない環境ではScalar実装へフォールバックします。
///
/// # Errors
///
/// `lhs`、`rhs`、`output`の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f64_array(lhs: &[f64], rhs: &[f64], output: &mut [f64]) -> Result<(), ArrayLengthError> {
    if lhs.len() != rhs.len() || lhs.len() != output.len() {
        return Err(ArrayLengthError::new(lhs.len(), rhs.len(), output.len()));
    }

    #[cfg(target_arch = "wasm32")]
    {
        unsafe {
            wasm32::add_f64(lhs, rhs, output);
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if x86_64::is_avx_available() {
                unsafe {
                    x86_64::add_f64(lhs, rhs, output);
                }

                return Ok(());
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if aarch64::is_neon_available() {
                unsafe {
                    aarch64::add_f64(lhs, rhs, output);
                }

                return Ok(());
            }
        }

        crate::scalar::add_f64_array(lhs, rhs, output)
    }
}

/// `f64`配列の要素をSIMDを利用して合計します。
///
/// x86_64環境でAVXが利用できる場合はAVX実装を使用します。
/// AVXを利用できない環境ではScalar実装へフォールバックします。
/// 空配列の合計は`0.0`です。
#[must_use]
pub fn sum_f64(input: &[f64]) -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { wasm32::sum_f64(input) }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if x86_64::is_avx_available() {
                unsafe {
                    return x86_64::sum_f64(input);
                }
            }
        }

        #[cfg(target_arch = "aarch64")]
        {
            if aarch64::is_neon_available() {
                unsafe {
                    return aarch64::sum_f64(input);
                }
            }
        }

        crate::scalar::sum_f64(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_vector_and_tail_elements() {
        let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        let mut output = [0.0; 10];

        add_f32(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(
            output,
            [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0,]
        );
    }

    #[test]
    fn adds_f64_vector_and_tail_elements() {
        let lhs = [0.1, 1.0, 2.0, 3.0, 4.0, 5.0];
        let rhs = [0.2, 10.0, 20.0, 30.0, 40.0, 50.0];
        let mut output = [0.0; 6];

        add_f64_array(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(output[0].to_bits(), 0x3fd3_3333_3333_3334);
        assert_eq!(output[1..], [11.0, 22.0, 33.0, 44.0, 55.0]);
    }

    #[test]
    fn adds_arrays_shorter_than_one_vector() {
        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut output = [0.0; 3];

        add_f32(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(output, [5.0, 7.0, 9.0]);
    }

    #[test]
    fn matches_scalar_reference() {
        let lhs: Vec<f32> = (0..37).map(|value| value as f32).collect();

        let rhs: Vec<f32> = (0..37).map(|value| value as f32 * 0.5).collect();

        let mut scalar_output = vec![0.0; lhs.len()];
        let mut simd_output = vec![0.0; lhs.len()];

        crate::scalar::add_f32(&lhs, &rhs, &mut scalar_output).unwrap();

        add_f32(&lhs, &rhs, &mut simd_output).unwrap();

        assert_eq!(simd_output, scalar_output);
    }

    #[test]
    fn f64_matches_scalar_reference() {
        let lhs: Vec<f64> = (0..19).map(|value| f64::from(value) * 0.1).collect();
        let rhs: Vec<f64> = (0..19).map(|value| f64::from(value) * 0.2).collect();

        let mut scalar_output = vec![0.0; lhs.len()];
        let mut simd_output = vec![0.0; lhs.len()];

        crate::scalar::add_f64_array(&lhs, &rhs, &mut scalar_output).unwrap();
        add_f64_array(&lhs, &rhs, &mut simd_output).unwrap();

        assert_eq!(simd_output, scalar_output);
    }

    #[test]
    fn sums_f64_vector_and_tail_elements() {
        let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

        assert_eq!(sum_f64(&input), 55.0);
        assert_eq!(sum_f64(&[]), 0.0);
    }

    #[test]
    fn f64_sum_matches_scalar_reference_for_exact_values() {
        let input: Vec<f64> = (1..=19).map(f64::from).collect();

        assert_eq!(sum_f64(&input), crate::scalar::sum_f64(&input));
    }

    #[test]
    fn rejects_different_lengths() {
        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut output = [10.0; 2];

        let result = add_f32(&lhs, &rhs, &mut output);

        assert_eq!(result, Err(ArrayLengthError::new(3, 3, 2)));
        assert_eq!(output, [10.0; 2]);
    }

    #[test]
    fn rejects_different_f64_lengths() {
        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0];
        let mut output = [10.0; 3];

        let result = add_f64_array(&lhs, &rhs, &mut output);

        assert_eq!(result, Err(ArrayLengthError::new(3, 2, 3)));
        assert_eq!(output, [10.0; 3]);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn avx_implementation_works_when_available() {
        if !is_avx_available() {
            return;
        }

        let lhs = [1.0; 8];
        let rhs = [2.0; 8];
        let mut output = [0.0; 8];

        // SAFETY:
        // このテストでは事前にAVX対応を確認しています。
        unsafe {
            x86_64::add_f32(&lhs, &rhs, &mut output);
        }

        assert_eq!(output, [3.0; 8]);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn f64_avx_implementation_works_when_available() {
        if !is_avx_available() {
            return;
        }

        let lhs = [1.0; 4];
        let rhs = [2.0; 4];
        let mut output = [0.0; 4];

        // SAFETY:
        // このテストでは事前にAVX対応を確認しています。
        unsafe {
            x86_64::add_f64(&lhs, &rhs, &mut output);
        }

        assert_eq!(output, [3.0; 4]);
    }
}
