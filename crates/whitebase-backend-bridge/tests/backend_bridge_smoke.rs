use whitebase_backend_bridge::{
    AssemblyAvxBackend, AssemblyScalarBackend, CppAvxBackend, CppScalarBackend, RustScalarBackend,
    RustSimdBackend,
};
use whitebase_backend_contract::ComputeBackend;

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
use whitebase_backend_bridge::{
    WindowsGnuAssemblyAvxBackend, WindowsGnuAssemblyScalarBackend, WindowsGnuCppAvxBackend,
    WindowsGnuCppScalarBackend,
};

#[test]
fn available_backends_produce_the_same_result() {
    let backends = standard_backends();

    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    let expected = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    for backend in backends {
        println!(
            "{} available: {}",
            backend.kind().display_name(),
            backend.is_available(),
        );

        if !backend.is_available() {
            continue;
        }

        let mut output = [0.0; 10];

        backend.add_f32(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(
            output,
            expected,
            "{} produced a different result",
            backend.kind().display_name(),
        );
    }
}

#[test]
fn unavailable_backend_reports_an_error() {
    let backend = CppAvxBackend;

    if backend.is_available() {
        return;
    }

    let lhs = [1.0];
    let rhs = [2.0];
    let mut output = [0.0];

    assert!(backend.add_f32(&lhs, &rhs, &mut output).is_err());
}

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
#[test]
fn linux_uses_native_backends() {
    assert!(CppScalarBackend.is_available());
    assert!(AssemblyScalarBackend.is_available());

    let avx_available = std::arch::is_x86_feature_detected!("avx");

    assert_eq!(CppAvxBackend.is_available(), avx_available);
    assert_eq!(AssemblyAvxBackend.is_available(), avx_available);
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn standard_backends() -> Vec<Box<dyn ComputeBackend>> {
    vec![
        Box::new(RustScalarBackend),
        Box::new(RustSimdBackend),
        Box::new(CppScalarBackend),
        Box::new(CppAvxBackend),
        Box::new(AssemblyScalarBackend),
        Box::new(AssemblyAvxBackend),
        Box::new(WindowsGnuCppScalarBackend),
        Box::new(WindowsGnuCppAvxBackend),
        Box::new(WindowsGnuAssemblyScalarBackend),
        Box::new(WindowsGnuAssemblyAvxBackend),
    ]
}

#[cfg(not(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc")))]
fn standard_backends() -> Vec<Box<dyn ComputeBackend>> {
    vec![
        Box::new(RustScalarBackend),
        Box::new(RustSimdBackend),
        Box::new(CppScalarBackend),
        Box::new(CppAvxBackend),
        Box::new(AssemblyScalarBackend),
        Box::new(AssemblyAvxBackend),
    ]
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn windows_gnu_backends_share_the_dll_availability_contract() {
    let gcc_scalar = WindowsGnuCppScalarBackend;
    let nasm_scalar = WindowsGnuAssemblyScalarBackend;
    let gcc_avx = WindowsGnuCppAvxBackend;
    let nasm_avx = WindowsGnuAssemblyAvxBackend;

    assert_eq!(gcc_scalar.is_available(), nasm_scalar.is_available());

    if !gcc_scalar.is_available() {
        assert!(!gcc_avx.is_available());
        assert!(!nasm_avx.is_available());
        return;
    }

    let avx_available = std::arch::is_x86_feature_detected!("avx");

    assert_eq!(gcc_avx.is_available(), avx_available);
    assert_eq!(nasm_avx.is_available(), avx_available);
}
