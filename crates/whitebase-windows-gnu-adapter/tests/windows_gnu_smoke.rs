#![cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]

use whitebase_windows_gnu_adapter::{
    assembly_add_f32_avx, assembly_add_f32_scalar, cpp_add_f32_avx, cpp_add_f32_scalar,
    is_assembly_avx_available, is_available, is_cpp_avx_available,
};

#[test]
fn available_windows_gnu_backends_produce_expected_results() {
    if !is_available() {
        eprintln!("Windows GNU Native DLL is unavailable; skipping runtime smoke test");
        return;
    }

    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
    let expected = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    let mut gcc_scalar = [0.0; 10];
    cpp_add_f32_scalar(&lhs, &rhs, &mut gcc_scalar).unwrap();
    assert_eq!(gcc_scalar, expected);

    let mut nasm_scalar = [0.0; 10];
    assembly_add_f32_scalar(&lhs, &rhs, &mut nasm_scalar).unwrap();
    assert_eq!(nasm_scalar, expected);

    if is_cpp_avx_available() {
        let mut gcc_avx = [0.0; 10];
        assert!(cpp_add_f32_avx(&lhs, &rhs, &mut gcc_avx).unwrap());
        assert_eq!(gcc_avx, expected);
    }

    if is_assembly_avx_available() {
        let mut nasm_avx = [0.0; 10];
        assert!(assembly_add_f32_avx(&lhs, &rhs, &mut nasm_avx).unwrap());
        assert_eq!(nasm_avx, expected);
    }
}
