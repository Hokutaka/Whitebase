#![forbid(unsafe_code)]

use std::{error::Error, fmt};

use cerune_lang::compile_to_wat;

#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;

#[cfg(not(target_arch = "wasm32"))]
use wasmi::{Engine, F32, F64, Linker, Module, Store, TypedFunc};

#[cfg(not(target_arch = "wasm32"))]
const WHITEBASE_ADD_SCALAR_F64_EXPORT: &str = "whitebase_cerune_add_scalar_f64";

const ADD_SCALAR_F64_SOURCE: &str = r#"
fn add(lhs: f64, rhs: f64) -> f64 {
    return lhs + rhs;
}
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CeruneWatAdapterError {
    /// CeruneソースからWATを生成できませんでした。
    Compilation(String),

    /// Whitebase用WAT glueを生成できませんでした。
    HostGlue(String),

    /// Wasm runtimeを準備できませんでした。
    RuntimePreparation(String),

    /// Wasm関数の実行に失敗しました。
    RuntimeExecution(String),

    /// Wasm storeのロックを取得できませんでした。
    StoreUnavailable,
}

impl fmt::Display for CeruneWatAdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Compilation(message) => {
                write!(formatter, "Cerune WAT compilation failed: {message}")
            }

            Self::HostGlue(message) => {
                write!(formatter, "Cerune WAT host glue failed: {message}")
            }
            Self::RuntimePreparation(message) => {
                write!(
                    formatter,
                    "Cerune WAT runtime preparation failed: {message}"
                )
            }
            Self::RuntimeExecution(message) => {
                write!(formatter, "Cerune WAT runtime execution failed: {message}")
            }
            Self::StoreUnavailable => {
                write!(formatter, "Cerune WAT runtime store is unavailable")
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn add_whitebase_host_export(mut wat: String) -> Result<String, CeruneWatAdapterError> {
    let module_end = wat.rfind(')').ok_or_else(|| {
        CeruneWatAdapterError::HostGlue("WAT module closing delimiter was not found".to_owned())
    })?;

    wat.insert_str(
        module_end,
        concat!(
            "  (export \"whitebase_cerune_add_scalar_f64\" ",
            "(func $cerune_fn_add_0))\n"
        ),
    );

    Ok(wat)
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug)]
pub struct CeruneWatAdapter {
    store: Mutex<Store<()>>,
    add_scalar_f64: TypedFunc<(F64, F64), F64>,
}

#[cfg(not(target_arch = "wasm32"))]
impl CeruneWatAdapter {
    pub fn new() -> Result<Self, CeruneWatAdapterError> {
        let wat = emit_add_scalar_f64_wat()?;
        let wat = add_whitebase_host_export(wat)?;

        let engine = Engine::default();

        let module = Module::new(&engine, wat.as_bytes())
            .map_err(|error| CeruneWatAdapterError::RuntimePreparation(error.to_string()))?;

        let mut store = Store::new(&engine, ());
        let mut linker = Linker::new(&engine);

        linker
            .func_wrap("cerune", "print_i64", |_value: i64| {})
            .map_err(|error| CeruneWatAdapterError::RuntimePreparation(error.to_string()))?;

        linker
            .func_wrap("cerune", "print_f32", |_value: F32| {})
            .map_err(|error| CeruneWatAdapterError::RuntimePreparation(error.to_string()))?;

        linker
            .func_wrap("cerune", "print_f64", |_value: F64| {})
            .map_err(|error| CeruneWatAdapterError::RuntimePreparation(error.to_string()))?;

        let instance = linker
            .instantiate_and_start(&mut store, &module)
            .map_err(|error| CeruneWatAdapterError::RuntimePreparation(error.to_string()))?;

        let add_scalar_f64 = instance
            .get_typed_func::<(F64, F64), F64>(&store, WHITEBASE_ADD_SCALAR_F64_EXPORT)
            .map_err(|error| CeruneWatAdapterError::RuntimePreparation(error.to_string()))?;

        Ok(Self {
            store: Mutex::new(store),
            add_scalar_f64,
        })
    }

    pub fn add_scalar_f64(&self, lhs: f64, rhs: f64) -> Result<f64, CeruneWatAdapterError> {
        let mut store = self
            .store
            .lock()
            .map_err(|_| CeruneWatAdapterError::StoreUnavailable)?;

        let result = self
            .add_scalar_f64
            .call(&mut *store, (lhs.into(), rhs.into()))
            .map_err(|error| CeruneWatAdapterError::RuntimeExecution(error.to_string()))?;

        Ok(result.into())
    }
}

impl Error for CeruneWatAdapterError {}

/// `f64 + f64 -> f64` のCerune関数をWAT artifactとして生成します。
pub fn emit_add_scalar_f64_wat() -> Result<String, CeruneWatAdapterError> {
    compile_to_wat(ADD_SCALAR_F64_SOURCE)
        .map_err(|error| CeruneWatAdapterError::Compilation(error.message().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_add_scalar_f64_wat_artifact() {
        let wat = emit_add_scalar_f64_wat().unwrap();

        assert!(wat.contains(
            "(func $cerune_fn_add_0 \
             (param $cerune_lhs f64) \
             (param $cerune_rhs f64) \
             (result f64)"
        ));
        assert!(wat.contains("f64.add"));

        // Cerune currently exposes only its normal module entry point.
        assert!(wat.contains("(export \"main\" (func $main))"));

        // WAT artifacts expect the host to provide Cerune runtime imports.
        assert!(wat.contains(
            "(import \"cerune\" \"print_f64\" \
             (func $print_f64 (param f64)))"
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn adds_whitebase_export_to_wat_artifact() {
        let wat = emit_add_scalar_f64_wat().unwrap();
        let wat = add_whitebase_host_export(wat).unwrap();

        assert!(wat.contains(
            "(export \"whitebase_cerune_add_scalar_f64\" \
            (func $cerune_fn_add_0))"
        ));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn prepares_and_executes_add_scalar_f64_wat() {
        let adapter = CeruneWatAdapter::new().unwrap();

        let result = adapter.add_scalar_f64(0.1, 0.2).unwrap();

        assert_eq!(result.to_bits(), 0x3fd3_3333_3333_3334);
    }
}
