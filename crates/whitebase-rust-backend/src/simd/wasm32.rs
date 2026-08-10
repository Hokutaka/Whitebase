const F32_LANES: usize = 4;
const F64_LANES: usize = 2;

pub(super) const fn is_available() -> bool {
    true
}

#[target_feature(enable = "simd128")]
pub(super) unsafe fn add_f32(lhs: &[f32], rhs: &[f32], output: &mut [f32]) {
    use std::arch::wasm32::{f32x4_add, v128, v128_load, v128_store};

    let vectorized_len = lhs.len() / F32_LANES * F32_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = v128_load(lhs.as_ptr().add(index).cast::<v128>());
            let rhs_values = v128_load(rhs.as_ptr().add(index).cast::<v128>());

            let result = f32x4_add(lhs_values, rhs_values);

            v128_store(output.as_mut_ptr().add(index).cast::<v128>(), result);
        }

        index += F32_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[target_feature(enable = "simd128")]
pub(super) unsafe fn add_f64(lhs: &[f64], rhs: &[f64], output: &mut [f64]) {
    use std::arch::wasm32::{f64x2_add, v128, v128_load, v128_store};

    let vectorized_len = lhs.len() / F64_LANES * F64_LANES;
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let lhs_values = v128_load(lhs.as_ptr().add(index).cast::<v128>());
            let rhs_values = v128_load(rhs.as_ptr().add(index).cast::<v128>());

            let result = f64x2_add(lhs_values, rhs_values);

            v128_store(output.as_mut_ptr().add(index).cast::<v128>(), result);
        }

        index += F64_LANES;
    }

    while index < lhs.len() {
        output[index] = lhs[index] + rhs[index];
        index += 1;
    }
}

#[target_feature(enable = "simd128")]
pub(super) unsafe fn sum_f64(input: &[f64]) -> f64 {
    use std::arch::wasm32::{f64x2, f64x2_add, f64x2_extract_lane, v128, v128_load};

    let vectorized_len = input.len() / F64_LANES * F64_LANES;
    let mut accumulator = f64x2(0.0, 0.0);
    let mut index = 0;

    while index < vectorized_len {
        unsafe {
            let values = v128_load(input.as_ptr().add(index).cast::<v128>());

            accumulator = f64x2_add(accumulator, values);
        }

        index += F64_LANES;
    }

    let mut sum = f64x2_extract_lane::<0>(accumulator) + f64x2_extract_lane::<1>(accumulator);

    while index < input.len() {
        sum += input[index];
        index += 1;
    }

    sum
}
