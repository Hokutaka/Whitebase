use whitebase_rust_backend::scalar::add_f64_array;

#[test]
fn scalar_add_f64_array_smoke_test() {
    let lhs = [0.1, 1.0, 2.0, 3.0, 4.0];
    let rhs = [0.2, 10.0, 20.0, 30.0, 40.0];
    let mut output = [0.0; 5];

    add_f64_array(&lhs, &rhs, &mut output).unwrap();

    assert_eq!(output[0].to_bits(), 0x3fd3_3333_3333_3334);
    assert_eq!(output[1..], [11.0, 22.0, 33.0, 44.0]);
}
