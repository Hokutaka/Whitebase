use wasm_bindgen::prelude::*;

/// 2つの整数を加算します。
///
/// JavaScriptからWhitebase Coreを呼び出すためのWasm境界です。
#[wasm_bindgen]
#[must_use]
pub fn add(left: i32, right: i32) -> i32 {
    whitebase_rust_backend::add(left, right)
}
