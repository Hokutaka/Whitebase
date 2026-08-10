const AVX_F32_LANES: usize = 8;
const AVX_F64_LANES: usize = 4;

pub(super) fn is_avx_available() -> bool {
    std::arch::is_x86_feature_detected!("avx")
}

#[target_feature(enable = "avx")]
pub(super) unsafe fn sum_f64(input: &[f64]) -> f64 {
    use std::arch::x86_64::{_mm256_add_pd, _mm256_loadu_pd, _mm256_setzero_pd, _mm256_storeu_pd};

    let vectorized_len = input.len() / AVX_F64_LANES * AVX_F64_LANES;
    let mut accumulator = _mm256_setzero_pd();
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let values = _mm256_loadu_pd(input.as_ptr().add(index));
            accumulator = _mm256_add_pd(accumulator, values);
        }

        index += AVX_F64_LANES;
    }

    let mut lanes = [0.0; AVX_F64_LANES];

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

#[target_feature(enable = "avx")]
pub(super) unsafe fn add_f32(lhs: &[f32], rhs: &[f32], output: &mut [f32]) {
    use std::arch::x86_64::{_mm256_add_ps, _mm256_loadu_ps, _mm256_storeu_ps};

    let vectorized_len = lhs.len() / AVX_F32_LANES * AVX_F32_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = _mm256_loadu_ps(lhs.as_ptr().add(index));
            let rhs_values = _mm256_loadu_ps(rhs.as_ptr().add(index));

            let result = _mm256_add_ps(lhs_values, rhs_values);

            _mm256_storeu_ps(output.as_mut_ptr().add(index), result);
        }

        index += AVX_F32_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[target_feature(enable = "avx")]
pub(super) unsafe fn add_f64(lhs: &[f64], rhs: &[f64], output: &mut [f64]) {
    use std::arch::x86_64::{_mm256_add_pd, _mm256_loadu_pd, _mm256_storeu_pd};

    let vectorized_len = lhs.len() / AVX_F64_LANES * AVX_F64_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = _mm256_loadu_pd(lhs.as_ptr().add(index));
            let rhs_values = _mm256_loadu_pd(rhs.as_ptr().add(index));

            let result = _mm256_add_pd(lhs_values, rhs_values);

            _mm256_storeu_pd(output.as_mut_ptr().add(index), result);
        }

        index += AVX_F64_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}
