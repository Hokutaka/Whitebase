const F32_LANES: usize = 4;
const F64_LANES: usize = 2;

pub(super) fn is_neon_available() -> bool {
    std::arch::is_aarch64_feature_detected!("neon")
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn add_f32(lhs: &[f32], rhs: &[f32], output: &mut [f32]) {
    use std::arch::aarch64::{vaddq_f32, vld1q_f32, vst1q_f32};

    let vectorized_len = lhs.len() / F32_LANES * F32_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = vld1q_f32(lhs.as_ptr().add(index));
            let rhs_values = vld1q_f32(rhs.as_ptr().add(index));

            let result = vaddq_f32(lhs_values, rhs_values);

            vst1q_f32(output.as_mut_ptr().add(index), result);
        }

        index += F32_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn add_f64(lhs: &[f64], rhs: &[f64], output: &mut [f64]) {
    use std::arch::aarch64::{vaddq_f64, vld1q_f64, vst1q_f64};

    let vectorized_len = lhs.len() / F64_LANES * F64_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = vld1q_f64(lhs.as_ptr().add(index));
            let rhs_values = vld1q_f64(rhs.as_ptr().add(index));

            let result = vaddq_f64(lhs_values, rhs_values);

            vst1q_f64(output.as_mut_ptr().add(index), result);
        }

        index += F64_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[target_feature(enable = "neon")]
pub(super) unsafe fn sum_f64(input: &[f64]) -> f64 {
    use std::arch::aarch64::{vaddq_f64, vdupq_n_f64, vld1q_f64, vst1q_f64};

    let vectorized_len = input.len() / F64_LANES * F64_LANES;
    let mut accumulator = vdupq_n_f64(0.0);
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let values = vld1q_f64(input.as_ptr().add(index));
            accumulator = vaddq_f64(accumulator, values);
        }

        index += F64_LANES;
    }

    let mut lanes = [0.0; F64_LANES];

    unsafe {
        vst1q_f64(lanes.as_mut_ptr(), accumulator);
    }

    let mut sum = lanes[0] + lanes[1];

    while index < input.len() {
        sum += input[index];
        index += 1;
    }

    sum
}
