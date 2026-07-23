use crate::ArrayLengthError;

#[cfg(target_arch = "x86_64")]
const AVX_F32_LANES: usize = 8;

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

    #[cfg(target_arch = "x86_64")]
    {
        if is_avx_available() {
            // SAFETY:
            // 実行前にAVX対応を確認しており、配列の長さも検証済みです。
            unsafe {
                add_f32_avx(lhs, rhs, output);
            }

            return Ok(());
        }
    }

    crate::scalar::add_f32(lhs, rhs, output)
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn add_f32_avx(lhs: &[f32], rhs: &[f32], output: &mut [f32]) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_loadu_ps, _mm256_storeu_ps};

    let vectorized_len = lhs.len() / AVX_F32_LANES * AVX_F32_LANES;

    let mut index = 0;

    while index < vectorized_len {
        // SAFETY:
        // vectorized_lenは8要素単位に切り下げられているため、
        // indexから8要素分の読み書きが各スライスの範囲内に収まります。
        // loadu/storeuはメモリアドレスのアラインメントを要求しません。
        unsafe {
            let lhs_values = _mm256_loadu_ps(lhs.as_ptr().add(index));
            let rhs_values = _mm256_loadu_ps(rhs.as_ptr().add(index));

            let result = _mm256_add_ps(lhs_values, rhs_values);

            _mm256_storeu_ps(output.as_mut_ptr().add(index), result);
        }

        index += AVX_F32_LANES;
    }

    // 8要素に満たなかった末尾部分をScalarで処理します。
    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
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
    fn rejects_different_lengths() {
        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut output = [10.0; 2];

        let result = add_f32(&lhs, &rhs, &mut output);

        assert_eq!(result, Err(ArrayLengthError::new(3, 3, 2)));
        assert_eq!(output, [10.0; 2]);
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
            add_f32_avx(&lhs, &rhs, &mut output);
        }

        assert_eq!(output, [3.0; 8]);
    }
}
