use whitebase_core::{BackendKind, ComputeError, OperationKind, Whitebase};

#[test]
fn rust_scalar_adds_f64_values() {
    let whitebase = Whitebase::new();

    let result = whitebase
        .add_scalar_f64(BackendKind::RustScalar, 0.1, 0.2)
        .unwrap();

    assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    assert_eq!(0.3_f64.to_bits(), 0x3fd3_3333_3333_3333);
}

#[test]
fn unsupported_backend_reports_scalar_f64_operation() {
    let whitebase = Whitebase::new();

    assert_eq!(
        whitebase.add_scalar_f64(BackendKind::RustSimd, 0.1, 0.2),
        Err(ComputeError::OperationUnsupported {
            backend: BackendKind::RustSimd,
            operation: OperationKind::AddScalarF64,
        })
    );
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
#[test]
fn cpp_scalar_adds_f64_values() {
    let whitebase = Whitebase::new();

    let result = whitebase
        .add_scalar_f64(BackendKind::CppScalar, 0.1, 0.2)
        .unwrap();

    assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
#[test]
fn assembly_scalar_adds_f64_values() {
    let whitebase = Whitebase::new();

    let result = whitebase
        .add_scalar_f64(BackendKind::AssemblyScalar, 0.1, 0.2)
        .unwrap();

    assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn available_windows_gnu_scalar_backends_add_f64_values() {
    let whitebase = Whitebase::new();

    for backend in [
        BackendKind::WindowsGnuCppScalar,
        BackendKind::WindowsGnuAssemblyScalar,
    ] {
        if !whitebase.backend_info(backend).unwrap().available {
            continue;
        }

        let result = whitebase.add_scalar_f64(backend, 0.1, 0.2).unwrap();
        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }
}
