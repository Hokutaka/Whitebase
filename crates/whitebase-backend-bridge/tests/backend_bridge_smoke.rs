use whitebase_backend_bridge::{
    AssemblyAvxBackend, AssemblyScalarBackend, CppAvxBackend, CppScalarBackend, RustScalarBackend,
    RustSimdBackend,
};
use whitebase_interface::ComputeBackend;

#[test]
fn available_backends_produce_the_same_result() {
    let backends: Vec<Box<dyn ComputeBackend>> = vec![
        Box::new(RustScalarBackend),
        Box::new(RustSimdBackend),
        Box::new(CppScalarBackend),
        Box::new(CppAvxBackend),
        Box::new(AssemblyScalarBackend),
        Box::new(AssemblyAvxBackend),
    ];

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
