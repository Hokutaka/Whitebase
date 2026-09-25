use whitebase_core::{OperationKind, Whitebase};

#[test]
fn every_available_backend_supporting_add_f32_produces_the_same_result() {
    let whitebase = Whitebase::new();

    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    let expected = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    let backends = whitebase.backends();

    for info in backends {
        let kind = info.kind;

        println!("{} available: {}", kind.display_name(), info.available,);

        if !info.capabilities.supports(OperationKind::AddF32) {
            continue;
        }

        if !info.available {
            continue;
        }

        let mut output = [0.0; 10];

        whitebase.add_f32(kind, &lhs, &rhs, &mut output).unwrap();

        assert_eq!(
            output,
            expected,
            "{} produced a different result",
            kind.display_name(),
        );
    }
}

#[test]
fn reports_all_standard_backends() {
    let whitebase = Whitebase::new();

    let expected = if cfg!(all(
        target_arch = "x86_64",
        target_os = "windows",
        target_env = "msvc"
    )) {
        11
    } else {
        7
    };

    assert_eq!(whitebase.backends().len(), expected);
}

#[test]
fn cerune_vm_adds_f64_scalars() {
    use whitebase_core::BackendKind;

    let whitebase = Whitebase::new();

    let info = whitebase.backend_info(BackendKind::CeruneVm).unwrap();

    assert!(info.available);
    assert!(info.capabilities.supports(OperationKind::AddScalarF64));
    assert!(!info.capabilities.supports(OperationKind::AddF32));
    assert!(!info.capabilities.supports(OperationKind::AddF64));
    assert!(!info.capabilities.supports(OperationKind::SumF64));

    let result = whitebase
        .add_scalar_f64(BackendKind::CeruneVm, 0.1, 0.2)
        .unwrap();

    assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
}

#[test]
fn rust_backends_sum_f64_values() {
    use whitebase_core::BackendKind;

    let whitebase = Whitebase::new();
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    assert_eq!(
        whitebase.sum_f64(BackendKind::RustScalar, &input).unwrap(),
        55.0
    );

    let simd_info = whitebase.backend_info(BackendKind::RustSimd).unwrap();
    if simd_info.available {
        assert_eq!(
            whitebase.sum_f64(BackendKind::RustSimd, &input).unwrap(),
            55.0
        );
    }
}

#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
#[test]
fn cpp_backends_sum_f64_values() {
    use whitebase_core::BackendKind;

    let whitebase = Whitebase::new();
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    assert_eq!(
        whitebase.sum_f64(BackendKind::CppScalar, &input).unwrap(),
        55.0
    );

    let avx_info = whitebase.backend_info(BackendKind::CppAvx).unwrap();
    if avx_info.available {
        assert_eq!(
            whitebase.sum_f64(BackendKind::CppAvx, &input).unwrap(),
            55.0
        );
    }
}
#[cfg(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
))]
#[test]
fn assembly_backends_sum_f64_values() {
    use whitebase_core::BackendKind;

    let whitebase = Whitebase::new();
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    assert_eq!(
        whitebase
            .sum_f64(BackendKind::AssemblyScalar, &input)
            .unwrap(),
        55.0
    );

    let avx_info = whitebase.backend_info(BackendKind::AssemblyAvx).unwrap();
    if avx_info.available {
        assert_eq!(
            whitebase.sum_f64(BackendKind::AssemblyAvx, &input).unwrap(),
            55.0
        );
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn windows_gnu_cpp_backends_sum_f64_values() {
    use whitebase_core::BackendKind;

    let whitebase = Whitebase::new();
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let scalar_info = whitebase
        .backend_info(BackendKind::WindowsGnuCppScalar)
        .unwrap();
    if scalar_info.available {
        assert_eq!(
            whitebase
                .sum_f64(BackendKind::WindowsGnuCppScalar, &input)
                .unwrap(),
            55.0
        );
    }

    let avx_info = whitebase
        .backend_info(BackendKind::WindowsGnuCppAvx)
        .unwrap();
    if avx_info.available {
        assert_eq!(
            whitebase
                .sum_f64(BackendKind::WindowsGnuCppAvx, &input)
                .unwrap(),
            55.0
        );
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
#[test]
fn windows_gnu_assembly_backends_sum_f64_values() {
    use whitebase_core::BackendKind;

    let whitebase = Whitebase::new();
    let input = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let scalar_info = whitebase
        .backend_info(BackendKind::WindowsGnuAssemblyScalar)
        .unwrap();
    if scalar_info.available {
        assert_eq!(
            whitebase
                .sum_f64(BackendKind::WindowsGnuAssemblyScalar, &input)
                .unwrap(),
            55.0
        );
    }

    let avx_info = whitebase
        .backend_info(BackendKind::WindowsGnuAssemblyAvx)
        .unwrap();
    if avx_info.available {
        assert_eq!(
            whitebase
                .sum_f64(BackendKind::WindowsGnuAssemblyAvx, &input)
                .unwrap(),
            55.0
        );
    }
}
