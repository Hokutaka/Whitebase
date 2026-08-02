#![cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]

use whitebase_windows_gnu_adapter::{
    assembly_add_f32_avx, assembly_add_f32_scalar, assembly_add_f64_array_avx,
    assembly_add_f64_array_scalar, assembly_add_f64_scalar, cpp_add_f32_avx, cpp_add_f32_scalar,
    cpp_add_f64_array_avx, cpp_add_f64_array_scalar, cpp_add_f64_scalar, is_assembly_avx_available,
    is_available, is_cpp_avx_available,
};

#[test]
fn available_windows_gnu_backends_produce_expected_results() {
    if !is_available() {
        eprintln!("Windows GNU Native DLL is unavailable; skipping runtime smoke test");
        return;
    }

    let lhs_f32 = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let rhs_f32 = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
    let expected_f32 = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    let mut gcc_scalar_f32 = [0.0; 10];
    cpp_add_f32_scalar(&lhs_f32, &rhs_f32, &mut gcc_scalar_f32).unwrap();
    assert_eq!(gcc_scalar_f32, expected_f32);

    let mut nasm_scalar_f32 = [0.0; 10];
    assembly_add_f32_scalar(&lhs_f32, &rhs_f32, &mut nasm_scalar_f32).unwrap();
    assert_eq!(nasm_scalar_f32, expected_f32);

    if is_cpp_avx_available() {
        let mut gcc_avx_f32 = [0.0; 10];
        assert!(cpp_add_f32_avx(&lhs_f32, &rhs_f32, &mut gcc_avx_f32).unwrap());
        assert_eq!(gcc_avx_f32, expected_f32);
    }

    if is_assembly_avx_available() {
        let mut nasm_avx_f32 = [0.0; 10];
        assert!(assembly_add_f32_avx(&lhs_f32, &rhs_f32, &mut nasm_avx_f32).unwrap());
        assert_eq!(nasm_avx_f32, expected_f32);
    }

    let lhs_f64 = [0.1, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    let rhs_f64 = [0.2, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0];
    let expected_f64_bits = [
        0x3fd3_3333_3333_3334,
        11.0_f64.to_bits(),
        22.0_f64.to_bits(),
        33.0_f64.to_bits(),
        44.0_f64.to_bits(),
        55.0_f64.to_bits(),
        66.0_f64.to_bits(),
        77.0_f64.to_bits(),
        88.0_f64.to_bits(),
        99.0_f64.to_bits(),
    ];

    assert_eq!(
        cpp_add_f64_scalar(0.1, 0.2).unwrap().to_bits(),
        expected_f64_bits[0]
    );
    assert_eq!(
        assembly_add_f64_scalar(0.1, 0.2).unwrap().to_bits(),
        expected_f64_bits[0]
    );

    let mut gcc_scalar_f64 = [0.0; 10];
    cpp_add_f64_array_scalar(&lhs_f64, &rhs_f64, &mut gcc_scalar_f64).unwrap();
    assert_eq!(gcc_scalar_f64.map(f64::to_bits), expected_f64_bits);

    let mut nasm_scalar_f64 = [0.0; 10];
    assembly_add_f64_array_scalar(&lhs_f64, &rhs_f64, &mut nasm_scalar_f64).unwrap();
    assert_eq!(nasm_scalar_f64.map(f64::to_bits), expected_f64_bits);

    if is_cpp_avx_available() {
        let mut gcc_avx_f64 = [0.0; 10];
        assert!(cpp_add_f64_array_avx(&lhs_f64, &rhs_f64, &mut gcc_avx_f64).unwrap());
        assert_eq!(gcc_avx_f64.map(f64::to_bits), expected_f64_bits);
    }

    if is_assembly_avx_available() {
        let mut nasm_avx_f64 = [0.0; 10];
        assert!(assembly_add_f64_array_avx(&lhs_f64, &rhs_f64, &mut nasm_avx_f64).unwrap());
        assert_eq!(nasm_avx_f64.map(f64::to_bits), expected_f64_bits);
    }
}
