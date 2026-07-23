use whitebase_asm_adapter::{add_f32_avx, add_f32_scalar, is_avx_available};

#[test]
fn assembly_f32_array_smoke_test() {
    let lhs = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    let rhs = [10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

    let expected = [11.0, 22.0, 33.0, 44.0, 55.0, 66.0, 77.0, 88.0, 99.0, 110.0];

    let mut scalar_output = [0.0; 10];

    add_f32_scalar(&lhs, &rhs, &mut scalar_output).unwrap();

    assert_eq!(scalar_output, expected);

    println!("Assembly AVX available: {}", is_avx_available());

    let mut avx_output = [0.0; 10];

    let avx_executed = add_f32_avx(&lhs, &rhs, &mut avx_output).unwrap();

    if avx_executed {
        assert_eq!(avx_output, expected);
        assert_eq!(avx_output, scalar_output);
    }
}
