use std::sync::OnceLock;

use wasm_bindgen::prelude::*;
use whitebase_core::{BackendKind, ComputeError, Whitebase};
use whitebase_interface::{
    benchmark::{BenchmarkOperation, BenchmarkPrecision, BenchmarkRequest, execute_benchmark},
    scalar_f64::{ScalarF64Request, execute_scalar_f64_observation},
};

static WHITEBASE: OnceLock<Whitebase> = OnceLock::new();

fn whitebase() -> &'static Whitebase {
    WHITEBASE.get_or_init(Whitebase::new)
}

fn js_error(error: ComputeError) -> JsValue {
    JsValue::from_str(&error.to_string())
}

/// 2つの`f64`スカラー値を加算します。
#[wasm_bindgen]
pub fn add_scalar_f64(left: f64, right: f64) -> Result<f64, JsValue> {
    whitebase()
        .add_scalar_f64(BackendKind::RustScalar, left, right)
        .map_err(js_error)
}

/// 2つの`f32`配列を要素ごとに加算します。
#[wasm_bindgen]
pub fn add_f32(lhs: Box<[f32]>, rhs: Box<[f32]>) -> Result<Box<[f32]>, JsValue> {
    let mut output = vec![0.0; lhs.len()];

    whitebase()
        .add_f32(BackendKind::RustScalar, &lhs, &rhs, &mut output)
        .map_err(js_error)?;

    Ok(output.into_boxed_slice())
}

/// 2つの`f64`配列を要素ごとに加算します。
#[wasm_bindgen]
pub fn add_f64(lhs: Box<[f64]>, rhs: Box<[f64]>) -> Result<Box<[f64]>, JsValue> {
    let mut output = vec![0.0; lhs.len()];

    whitebase()
        .add_f64(BackendKind::RustScalar, &lhs, &rhs, &mut output)
        .map_err(js_error)?;

    Ok(output.into_boxed_slice())
}

/// `f64`配列の要素を合計します。
#[wasm_bindgen]
pub fn sum_f64(input: Box<[f64]>) -> Result<f64, JsValue> {
    whitebase()
        .sum_f64(BackendKind::RustScalar, &input)
        .map_err(js_error)
}

#[wasm_bindgen]
pub fn observe_add_scalar_f64(lhs: &str, rhs: &str) -> Result<JsValue, JsValue> {
    let report = execute_scalar_f64_observation(ScalarF64Request {
        lhs: lhs.to_owned(),
        rhs: rhs.to_owned(),
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    serde_wasm_bindgen::to_value(&report).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn run_benchmark(
    operation: &str,
    precision: &str,
    input_length: u32,
    warmup_iterations: u32,
    measured_iterations: u32,
) -> Result<JsValue, JsValue> {
    let operation = match operation {
        "add-array" => BenchmarkOperation::AddArray,
        "sum-f64" => BenchmarkOperation::SumF64,
        value => {
            return Err(JsValue::from_str(&format!(
                "unsupported benchmark operation: {value}"
            )));
        }
    };

    let precision = match precision {
        "f32" => BenchmarkPrecision::F32,
        "f64" => BenchmarkPrecision::F64,
        value => {
            return Err(JsValue::from_str(&format!(
                "unsupported benchmark precision: {value}"
            )));
        }
    };

    let report = execute_benchmark(BenchmarkRequest {
        operation,
        precision,
        input_length: input_length as usize,
        warmup_iterations: warmup_iterations as usize,
        measured_iterations: measured_iterations as usize,
    })
    .map_err(|error| JsValue::from_str(&error.to_string()))?;

    serde_wasm_bindgen::to_value(&report).map_err(|error| JsValue::from_str(&error.to_string()))
}
