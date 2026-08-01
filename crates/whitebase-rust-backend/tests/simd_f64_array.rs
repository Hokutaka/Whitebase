use whitebase_rust_backend::{
    scalar::add_f64_array as add_f64_scalar,
    simd::{add_f64_array as add_f64_simd, is_avx_available},
};

#[test]
fn simd_add_f64_array_matches_scalar() {
    println!("AVX available: {}", is_avx_available());

    let lhs: Vec<f64> = (0..21).map(|value| f64::from(value) * 0.1).collect();
    let rhs: Vec<f64> = (0..21).map(|value| f64::from(value) * 0.2).collect();

    let mut scalar_output = vec![0.0; lhs.len()];
    let mut simd_output = vec![0.0; lhs.len()];

    add_f64_scalar(&lhs, &rhs, &mut scalar_output).unwrap();
    add_f64_simd(&lhs, &rhs, &mut simd_output).unwrap();

    assert_eq!(simd_output, scalar_output);
}
