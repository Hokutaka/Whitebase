use std::ffi::c_int;

/// C ABI経由で2つの整数を加算します.
///
/// # ABI
///
/// この関数はC互換ABIで公開され、CやC++などの
/// 外部言語から呼び出すことを想定しています。
#[unsafe(no_mangle)]
pub extern "C" fn whitebase_add(left: c_int, right: c_int) -> c_int {
    whitebase_rust_backend::add(left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_through_c_abi() {
        assert_eq!(whitebase_add(2, 3), 5);
    }

    #[test]
    fn adds_negative_numbers_through_c_abi() {
        assert_eq!(whitebase_add(-2, -3), -5);
    }
}
