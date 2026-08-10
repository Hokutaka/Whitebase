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

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn observes_available_windows_gnu_scalar_f64_results() {
    let runner = Runner::new();

    for backend in [
        BackendKind::WindowsGnuCppScalar,
        BackendKind::WindowsGnuAssemblyScalar,
    ] {
        let report = match runner.run_add_scalar_f64(backend, 0.1, 0.2) {
            Ok(report) => report,
            Err(whitebase_runner::RunnerError::Compute {
                error: whitebase_core::ComputeError::BackendUnavailable { .. },
            }) => continue,
            Err(error) => panic!("{} failed: {error}", backend.display_name()),
        };

        assert_eq!(report.backend, backend);
        assert_eq!(report.result.bits, 0x3fd3_3333_3333_3334);
    }
}
