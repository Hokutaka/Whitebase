use whitebase_core::BackendKind;
use whitebase_runner::Runner;

#[test]
fn observes_scalar_f64_values_and_bits() {
    let report = Runner::new()
        .run_add_scalar_f64(BackendKind::RustScalar, 0.1, 0.2)
        .unwrap();

    assert_eq!(report.backend, BackendKind::RustScalar);
    assert_eq!(report.lhs.bits, 0x3fb9_9999_9999_999a);
    assert_eq!(report.rhs.bits, 0x3fc9_9999_9999_999a);
    assert_eq!(report.result.bits, 0x3fd3_3333_3333_3334);
    assert_eq!(0.3_f64.to_bits(), 0x3fd3_3333_3333_3333);
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn observes_cpp_scalar_f64_result() {
    let report = Runner::new()
        .run_add_scalar_f64(BackendKind::CppScalar, 0.1, 0.2)
        .unwrap();

    assert_eq!(report.backend, BackendKind::CppScalar);
    assert_eq!(report.result.bits, 0x3fd3_3333_3333_3334);
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn observes_assembly_scalar_f64_result() {
    let report = Runner::new()
        .run_add_scalar_f64(BackendKind::AssemblyScalar, 0.1, 0.2)
        .unwrap();

    assert_eq!(report.backend, BackendKind::AssemblyScalar);
    assert_eq!(report.result.bits, 0x3fd3_3333_3333_3334);
}
