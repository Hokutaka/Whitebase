use crate::ArrayLengthError;

/// 2つの`f32`配列を、要素ごとに加算します。
///
/// 計算結果は`output`へ書き込みます。
///
/// # Errors
///
/// `lhs`、`rhs`、`output`の長さが一致しない場合は
/// [`ArrayLengthError`]を返します。
pub fn add_f32(lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ArrayLengthError> {
    if lhs.len() != rhs.len() || lhs.len() != output.len() {
        return Err(ArrayLengthError::new(lhs.len(), rhs.len(), output.len()));
    }

    for index in 0..lhs.len() {
        output[index] = lhs[index] + rhs[index];
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_f32_arrays() {
        let lhs = [1.0, 2.0, 3.0];
        let rhs = [4.0, 5.0, 6.0];
        let mut output = [0.0; 3];

        add_f32(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(output, [5.0, 7.0, 9.0]);
    }

    #[test]
    fn accepts_empty_arrays() {
        let lhs: [f32; 0] = [];
        let rhs: [f32; 0] = [];
        let mut output: [f32; 0] = [];

        let result = add_f32(&lhs, &rhs, &mut output);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn adds_negative_and_fractional_values() {
        let lhs = [1.25, -2.0, 0.25];
        let rhs = [0.5, 0.5, -0.75];
        let mut output = [0.0; 3];

        add_f32(&lhs, &rhs, &mut output).unwrap();

        assert_eq!(output, [1.75, -1.5, -0.5]);
    }

    #[test]
    fn rejects_different_input_lengths() {
        let lhs = [1.0, 2.0];
        let rhs = [3.0];
        let mut output = [0.0; 2];

        let result = add_f32(&lhs, &rhs, &mut output);

        assert_eq!(result, Err(ArrayLengthError::new(2, 1, 2)));
    }

    #[test]
    fn does_not_modify_output_when_lengths_are_invalid() {
        let lhs = [1.0, 2.0];
        let rhs = [3.0, 4.0];
        let mut output = [10.0];

        let result = add_f32(&lhs, &rhs, &mut output);

        assert_eq!(result, Err(ArrayLengthError::new(2, 2, 1)));
        assert_eq!(output, [10.0]);
    }
}
