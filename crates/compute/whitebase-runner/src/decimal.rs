use crate::RunnerError;

const MAX_DECIMAL_INPUT_LENGTH: usize = 256;
const MAX_EXPANDED_DIGITS: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactDecimal {
    negative: bool,
    digits: String,
    scale: usize,
}

impl ExactDecimal {
    pub(crate) fn parse(name: &'static str, input: &str) -> Result<Self, RunnerError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(invalid_input(name, input, "value must not be empty"));
        }

        if trimmed.len() > MAX_DECIMAL_INPUT_LENGTH {
            return Err(invalid_input(
                name,
                input,
                format!("value must not exceed {MAX_DECIMAL_INPUT_LENGTH} characters"),
            ));
        }

        let (negative, unsigned) = match trimmed.as_bytes().first() {
            Some(b'+') => (false, &trimmed[1..]),
            Some(b'-') => (true, &trimmed[1..]),
            _ => (false, trimmed),
        };

        if unsigned.is_empty() {
            return Err(invalid_input(name, input, "value must contain digits"));
        }

        let mut exponent_marker = None;

        for (index, character) in unsigned.char_indices() {
            if matches!(character, 'e' | 'E') && exponent_marker.replace(index).is_some() {
                return Err(invalid_input(
                    name,
                    input,
                    "multiple exponent markers are not supported",
                ));
            }
        }

        let (mantissa, exponent) = match exponent_marker {
            Some(index) => {
                let exponent_text = &unsigned[index + 1..];

                if exponent_text.is_empty() {
                    return Err(invalid_input(name, input, "exponent must contain digits"));
                }

                let exponent = exponent_text.parse::<i32>().map_err(|_| {
                    invalid_input(name, input, "exponent must be a signed 32-bit integer")
                })?;

                (&unsigned[..index], exponent)
            }
            None => (unsigned, 0),
        };

        let mut digits = String::with_capacity(mantissa.len());
        let mut seen_decimal_point = false;
        let mut fractional_digits = 0_usize;

        for character in mantissa.chars() {
            match character {
                '0'..='9' => {
                    digits.push(character);

                    if seen_decimal_point {
                        fractional_digits += 1;
                    }
                }
                '.' if !seen_decimal_point => {
                    seen_decimal_point = true;
                }
                '.' => {
                    return Err(invalid_input(
                        name,
                        input,
                        "value must contain at most one decimal point",
                    ));
                }
                _ => {
                    return Err(invalid_input(
                        name,
                        input,
                        "value must use decimal digits, an optional point, and an optional exponent",
                    ));
                }
            }
        }

        if digits.is_empty() {
            return Err(invalid_input(name, input, "value must contain digits"));
        }

        let first_non_zero = digits.find(|character| character != '0');

        let Some(first_non_zero) = first_non_zero else {
            return Ok(Self::zero());
        };

        digits.replace_range(..first_non_zero, "");

        let scale = i64::try_from(fractional_digits).expect("decimal input length is bounded")
            - i64::from(exponent);

        let mut decimal = if scale < 0 {
            let trailing_zeros = usize::try_from(-scale).map_err(|_| {
                invalid_input(name, input, "expanded decimal representation is too large")
            })?;

            ensure_expanded_length(name, input, digits.len() + trailing_zeros)?;
            digits.extend(std::iter::repeat_n('0', trailing_zeros));

            Self {
                negative,
                digits,
                scale: 0,
            }
        } else {
            let scale = usize::try_from(scale).map_err(|_| {
                invalid_input(name, input, "expanded decimal representation is too large")
            })?;

            ensure_expanded_length(name, input, digits.len().max(scale))?;

            Self {
                negative,
                digits,
                scale,
            }
        };

        decimal.normalize();

        Ok(decimal)
    }

    pub(crate) fn add(&self, rhs: &Self) -> Self {
        let common_scale = self.scale.max(rhs.scale);
        let lhs_digits = aligned_digits(self, common_scale);
        let rhs_digits = aligned_digits(rhs, common_scale);

        let (negative, digits) = if self.negative == rhs.negative {
            (self.negative, add_magnitudes(&lhs_digits, &rhs_digits))
        } else {
            match compare_magnitudes(&lhs_digits, &rhs_digits) {
                std::cmp::Ordering::Greater => {
                    (self.negative, subtract_magnitudes(&lhs_digits, &rhs_digits))
                }
                std::cmp::Ordering::Less => {
                    (rhs.negative, subtract_magnitudes(&rhs_digits, &lhs_digits))
                }
                std::cmp::Ordering::Equal => return Self::zero(),
            }
        };

        let mut result = Self {
            negative,
            digits,
            scale: common_scale,
        };

        result.normalize();
        result
    }

    pub(crate) fn to_canonical_string(&self) -> String {
        if self.digits == "0" {
            return "0".to_owned();
        }

        let sign = if self.negative { "-" } else { "" };

        if self.scale == 0 {
            let digits = &self.digits;
            return format!("{sign}{digits}");
        }

        if self.digits.len() > self.scale {
            let integer_length = self.digits.len() - self.scale;
            let (integer, fraction) = self.digits.split_at(integer_length);
            return format!("{sign}{integer}.{fraction}");
        }

        let leading_zeros = "0".repeat(self.scale - self.digits.len());
        format!("{sign}0.{leading_zeros}{}", self.digits)
    }

    fn zero() -> Self {
        Self {
            negative: false,
            digits: "0".to_owned(),
            scale: 0,
        }
    }

    fn normalize(&mut self) {
        let first_non_zero = self.digits.find(|character| character != '0');

        match first_non_zero {
            Some(index) if index > 0 => {
                self.digits.replace_range(..index, "");
            }
            None => {
                *self = Self::zero();
                return;
            }
            _ => {}
        }

        while self.scale > 0 && self.digits.ends_with('0') {
            self.digits.pop();
            self.scale -= 1;
        }

        if self.digits.is_empty() {
            *self = Self::zero();
        }
    }
}

fn invalid_input(name: &'static str, value: &str, reason: impl Into<String>) -> RunnerError {
    RunnerError::InvalidScalarF64Input {
        name,
        value: value.to_owned(),
        reason: reason.into(),
    }
}

fn ensure_expanded_length(
    name: &'static str,
    input: &str,
    length: usize,
) -> Result<(), RunnerError> {
    if length > MAX_EXPANDED_DIGITS {
        return Err(invalid_input(
            name,
            input,
            format!("expanded decimal representation must not exceed {MAX_EXPANDED_DIGITS} digits"),
        ));
    }

    Ok(())
}

fn aligned_digits(value: &ExactDecimal, common_scale: usize) -> String {
    let mut digits = value.digits.clone();
    digits.extend(std::iter::repeat_n('0', common_scale - value.scale));
    digits
}

fn compare_magnitudes(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    lhs.len().cmp(&rhs.len()).then_with(|| lhs.cmp(rhs))
}

fn add_magnitudes(lhs: &str, rhs: &str) -> String {
    let lhs = lhs.as_bytes();
    let rhs = rhs.as_bytes();
    let mut lhs_index = lhs.len();
    let mut rhs_index = rhs.len();
    let mut carry = 0_u8;
    let mut reversed = Vec::with_capacity(lhs.len().max(rhs.len()) + 1);

    while lhs_index > 0 || rhs_index > 0 || carry > 0 {
        let lhs_digit = if lhs_index > 0 {
            lhs_index -= 1;
            lhs[lhs_index] - b'0'
        } else {
            0
        };

        let rhs_digit = if rhs_index > 0 {
            rhs_index -= 1;
            rhs[rhs_index] - b'0'
        } else {
            0
        };

        let sum = lhs_digit + rhs_digit + carry;
        reversed.push(b'0' + (sum % 10));
        carry = sum / 10;
    }

    reversed.reverse();
    String::from_utf8(reversed).expect("decimal arithmetic only emits ASCII digits")
}

fn subtract_magnitudes(larger: &str, smaller: &str) -> String {
    let larger = larger.as_bytes();
    let smaller = smaller.as_bytes();
    let mut larger_index = larger.len();
    let mut smaller_index = smaller.len();
    let mut borrow = 0_i8;
    let mut reversed = Vec::with_capacity(larger.len());

    while larger_index > 0 {
        larger_index -= 1;

        let mut digit =
            i8::try_from(larger[larger_index] - b'0').expect("decimal digit fits into i8") - borrow;

        let smaller_digit = if smaller_index > 0 {
            smaller_index -= 1;
            i8::try_from(smaller[smaller_index] - b'0').expect("decimal digit fits into i8")
        } else {
            0
        };

        if digit < smaller_digit {
            digit += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }

        let result_digit =
            u8::try_from(digit - smaller_digit).expect("subtraction result is a decimal digit");
        reversed.push(b'0' + result_digit);
    }

    while reversed.len() > 1 && reversed.last() == Some(&b'0') {
        reversed.pop();
    }

    reversed.reverse();
    String::from_utf8(reversed).expect("decimal arithmetic only emits ASCII digits")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(input: &str) -> ExactDecimal {
        ExactDecimal::parse("value", input).expect("test decimal must parse")
    }

    #[test]
    fn adds_decimal_fractions_exactly() {
        let result = parse("0.1").add(&parse("0.2"));

        assert_eq!(result.to_canonical_string(), "0.3");
    }

    #[test]
    fn adds_values_with_signs_and_different_scales() {
        let result = parse("-12.75").add(&parse("2.5"));

        assert_eq!(result.to_canonical_string(), "-10.25");
    }

    #[test]
    fn accepts_scientific_notation() {
        let result = parse("1.25e2").add(&parse("-2.5E1"));

        assert_eq!(result.to_canonical_string(), "100");
    }

    #[test]
    fn normalizes_decimal_zeroes() {
        assert_eq!(parse("000.3000").to_canonical_string(), "0.3");
        assert_eq!(parse("-0.000").to_canonical_string(), "0");
    }

    #[test]
    fn rejects_non_decimal_syntax() {
        let error = ExactDecimal::parse("lhs", "0x10").expect_err("hex must be rejected");

        assert!(matches!(
            error,
            RunnerError::InvalidScalarF64Input { name: "lhs", .. }
        ));
    }
}
