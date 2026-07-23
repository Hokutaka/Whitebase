//! Whitebaseの計算処理を提供するコアライブラリです。

mod error;

pub mod scalar;
pub mod simd;

pub use error::ArrayLengthError;

/// 2つの整数を加算します。
///
/// Whitebaseのライブラリ接続確認用となる最初の計算処理です。
#[must_use]
pub fn add(left: i32, right: i32) -> i32 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_two_numbers() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn adds_negative_numbers() {
        assert_eq!(add(-2, -3), -5);
    }
}
