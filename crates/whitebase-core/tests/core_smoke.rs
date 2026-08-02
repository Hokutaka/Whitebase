use whitebase_core::Whitebase;

#[test]
fn every_available_backend_produces_the_same_result() {
    let whitebase = Whitebase::new();

    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    let expected = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    let backends = whitebase.backends();

    for info in backends {
        let kind = info.kind;

        println!("{} available: {}", kind.display_name(), info.available,);

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
        10
    } else {
        6
    };

    assert_eq!(whitebase.backends().len(), expected);
}
