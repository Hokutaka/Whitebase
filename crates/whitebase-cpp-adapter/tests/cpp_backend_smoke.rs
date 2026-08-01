#![cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]

use whitebase_cpp_adapter::{
    add_f32_avx, add_f32_scalar, add_f64_array_avx, add_f64_array_scalar, add_f64_scalar,
    is_avx_available,
};

#[test]
fn cpp_backend_f32_array_smoke_test() {
    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    let expected = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    let mut scalar_output = [0.0; 10];

    add_f32_scalar(&lhs, &rhs, &mut scalar_output).unwrap();

    assert_eq!(scalar_output, expected);

    println!("C++ AVX available: {}", is_avx_available());

    let mut avx_output = [0.0; 10];

    let avx_executed = add_f32_avx(&lhs, &rhs, &mut avx_output).unwrap();

    if avx_executed {
        assert_eq!(avx_output, expected);
        assert_eq!(avx_output, scalar_output);
    }
}

#[test]
fn cpp_backend_f64_scalar_smoke_test() {
    let result = add_f64_scalar(0.1, 0.2);

    assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    assert_eq!(0.3_f64.to_bits(), 0x3fd3_3333_3333_3333);
}

#[test]
fn cpp_backend_f64_array_smoke_test() {
    let lhs = [0.1, 1.0, 2.0, 3.0, 4.0, 5.0];
    let rhs = [0.2, 10.0, 20.0, 30.0, 40.0, 50.0];
    let expected_bits = [
        0x3fd3_3333_3333_3334,
        11.0_f64.to_bits(),
        22.0_f64.to_bits(),
        33.0_f64.to_bits(),
        44.0_f64.to_bits(),
        55.0_f64.to_bits(),
    ];

    let mut scalar_output = [0.0; 6];
    add_f64_array_scalar(&lhs, &rhs, &mut scalar_output).unwrap();

    assert_eq!(scalar_output.map(f64::to_bits), expected_bits);

    let mut avx_output = [0.0; 6];
    let avx_executed = add_f64_array_avx(&lhs, &rhs, &mut avx_output).unwrap();

    if avx_executed {
        assert_eq!(avx_output.map(f64::to_bits), expected_bits);
        assert_eq!(avx_output, scalar_output);
    }
}
