use whitebase_rust_backend::scalar::add_f64;

#[test]
fn adds_f64_scalars_with_ieee_754_rounding() {
    let result = add_f64(0.1, 0.2);

    assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    assert_eq!(0.3_f64.to_bits(), 0x3fd3_3333_3333_3333);
    assert_ne!(result, 0.3);
}
