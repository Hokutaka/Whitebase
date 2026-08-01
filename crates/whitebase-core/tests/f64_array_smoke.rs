use whitebase_core::{BackendKind, OperationKind, Whitebase};

#[test]
fn available_backends_add_f64_arrays() {
    let whitebase = Whitebase::new();

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

    for kind in [
        BackendKind::RustScalar,
        BackendKind::RustSimd,
        BackendKind::CppScalar,
        BackendKind::CppAvx,
        BackendKind::AssemblyScalar,
        BackendKind::AssemblyAvx,
    ] {
        let info = whitebase.backend_info(kind).unwrap();

        assert!(info.capabilities.supports(OperationKind::AddF64));

        if !info.available {
            continue;
        }

        let mut output = [0.0; 6];

        whitebase.add_f64(kind, &lhs, &rhs, &mut output).unwrap();

        let actual_bits = output.map(f64::to_bits);

        assert_eq!(
            actual_bits,
            expected_bits,
            "{} produced a different result",
            kind.display_name(),
        );
    }
}
