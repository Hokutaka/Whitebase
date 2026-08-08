use crate::ArrayLengthError;

#[cfg(target_arch = "x86_64")]
const AVX_F32_LANES: usize = 8;

#[cfg(target_arch = "x86_64")]
const AVX_F64_LANES: usize = 4;

#[cfg(target_arch = "wasm32")]
const WASM_F32_LANES: usize = 4;

#[cfg(target_arch = "wasm32")]
const WASM_F64_LANES: usize = 2;

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
        is_avx_available()
    }

    #[cfg(target_arch = "wasm32")]
    {
        true
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "wasm32")))]
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
            add_f32_wasm_simd(lhs, rhs, output);
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if is_avx_available() {
                // SAFETY:
                // 実行前にAVX対応を確認しており、
                // 配列の長さも検証済みです。
                unsafe {
                    add_f32_avx(lhs, rhs, output);
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
        // SAFETY:
        // WASM版はsimd128を有効化した専用関数内で実行します。
        unsafe {
            add_f64_wasm_simd(lhs, rhs, output);
        }

        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if is_avx_available() {
                // SAFETY:
                // 実行前にAVX対応を確認しています。
                unsafe {
                    add_f64_avx(lhs, rhs, output);
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
        // SAFETY:
        // WASM版はsimd128を有効化した専用関数内で実行します。
        unsafe { sum_f64_wasm_simd(input) }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        #[cfg(target_arch = "x86_64")]
        {
            if is_avx_available() {
                // SAFETY:
                // 実行前にAVX対応を確認しています。
                unsafe {
                    return sum_f64_avx(input);
                }
            }
        }

        crate::scalar::sum_f64(input)
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn sum_f64_avx(input: &[f64]) -> f64 {
    use std::arch::x86_64::{_mm256_add_pd, _mm256_loadu_pd, _mm256_setzero_pd, _mm256_storeu_pd};

    let vectorized_len = input.len() / AVX_F64_LANES * AVX_F64_LANES;
    let mut accumulator = _mm256_setzero_pd();
    let mut index = 0;

    while index < vectorized_len {
        // SAFETY:
        // vectorized_lenは4要素単位に切り下げられているため、
        // indexから4要素分の読み込みがinputの範囲内に収まります。
        unsafe {
            let values = _mm256_loadu_pd(input.as_ptr().add(index));
            accumulator = _mm256_add_pd(accumulator, values);
        }
        index += AVX_F64_LANES;
    }

    let mut lanes = [0.0; AVX_F64_LANES];
    // SAFETY:
    // lanesは4要素のf64配列であり、storeuはアラインメントを要求しません。
    unsafe {
        _mm256_storeu_pd(lanes.as_mut_ptr(), accumulator);
    }

    let mut sum = lanes[0] + lanes[1] + lanes[2] + lanes[3];

    while index < input.len() {
        sum += input[index];
        index += 1;
    }

    sum
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

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn add_f64_avx(lhs: &[f64], rhs: &[f64], output: &mut [f64]) {
    use std::arch::x86_64::{_mm256_add_pd, _mm256_loadu_pd, _mm256_storeu_pd};

    let vectorized_len = lhs.len() / AVX_F64_LANES * AVX_F64_LANES;

    let mut index = 0;

    while index < vectorized_len {
        // SAFETY:
        // vectorized_lenは4要素単位に切り下げられているため、
        // indexから4要素分の読み書きが各スライスの範囲内に収まります。
        // loadu/storeuはメモリアドレスのアラインメントを要求しません。
        unsafe {
            let lhs_values = _mm256_loadu_pd(lhs.as_ptr().add(index));
            let rhs_values = _mm256_loadu_pd(rhs.as_ptr().add(index));

            let result = _mm256_add_pd(lhs_values, rhs_values);

            _mm256_storeu_pd(output.as_mut_ptr().add(index), result);
        }

        index += AVX_F64_LANES;
    }

    // 4要素に満たなかった末尾部分をScalarで処理します。
    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
unsafe fn add_f32_wasm_simd(lhs: &[f32], rhs: &[f32], output: &mut [f32]) {
    use std::arch::wasm32::{f32x4_add, v128, v128_load, v128_store};

    let vectorized_len = lhs.len() / WASM_F32_LANES * WASM_F32_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = v128_load(lhs.as_ptr().add(index).cast::<v128>());
            let rhs_values = v128_load(rhs.as_ptr().add(index).cast::<v128>());

            let result = f32x4_add(lhs_values, rhs_values);

            v128_store(output.as_mut_ptr().add(index).cast::<v128>(), result);
        }

        index += WASM_F32_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
unsafe fn add_f64_wasm_simd(lhs: &[f64], rhs: &[f64], output: &mut [f64]) {
    use std::arch::wasm32::{f64x2_add, v128, v128_load, v128_store};

    let vectorized_len = lhs.len() / WASM_F64_LANES * WASM_F64_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = v128_load(lhs.as_ptr().add(index).cast::<v128>());
            let rhs_values = v128_load(rhs.as_ptr().add(index).cast::<v128>());

            let result = f64x2_add(lhs_values, rhs_values);

            v128_store(output.as_mut_ptr().add(index).cast::<v128>(), result);
        }

        index += WASM_F64_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[cfg(target_arch = "wasm32")]
#[target_feature(enable = "simd128")]
unsafe fn sum_f64_wasm_simd(input: &[f64]) -> f64 {
    use std::arch::wasm32::{f64x2, f64x2_add, f64x2_extract_lane, v128, v128_load};

    let vectorized_len = input.len() / WASM_F64_LANES * WASM_F64_LANES;

    let mut accumulator = f64x2(0.0, 0.0);
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let values = v128_load(input.as_ptr().add(index).cast::<v128>());

            accumulator = f64x2_add(accumulator, values);
        }

        index += WASM_F64_LANES;
    }

    let mut sum = f64x2_extract_lane::<0>(accumulator) + f64x2_extract_lane::<1>(accumulator);

    while index < input.len() {
        sum += input[index];
        index += 1;
    }

    sum
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
            add_f32_avx(&lhs, &rhs, &mut output);
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
            add_f64_avx(&lhs, &rhs, &mut output);
        }

        assert_eq!(output, [3.0; 4]);
    }
}
