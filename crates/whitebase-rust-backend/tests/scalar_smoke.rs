use whitebase_rust_backend::scalar::add_f32;

#[test]
fn scalar_add_f32_smoke_test() {
    let lhs = [1.0, 2.0, 3.0, 4.0];
    let rhs = [10.0, 20.0, 30.0, 40.0];
    let mut output = [0.0; 4];

    add_f32(&lhs, &rhs, &mut output).unwrap();

    assert_eq!(output, [11.0, 22.0, 33.0, 44.0]);
}
