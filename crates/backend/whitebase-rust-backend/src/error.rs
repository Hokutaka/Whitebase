use core::fmt;

/// 配列演算に渡された配列の長さが一致しない場合のエラーです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrayLengthError {
    pub lhs_len: usize,
    pub rhs_len: usize,
    pub output_len: usize,
}

impl ArrayLengthError {
    pub const fn new(lhs_len: usize, rhs_len: usize, output_len: usize) -> Self {
        Self {
            lhs_len,
            rhs_len,
            output_len,
        }
    }
}

impl fmt::Display for ArrayLengthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "array length mismatch: lhs={}, rhs={}, output={}",
            self.lhs_len, self.rhs_len, self.output_len
        )
    }
}

impl std::error::Error for ArrayLengthError {}
