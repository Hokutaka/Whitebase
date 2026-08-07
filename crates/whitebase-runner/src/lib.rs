//! Whitebaseの演算実行、計測、結果比較を提供します。

#![forbid(unsafe_code)]

mod config;
mod decimal;
mod error;
mod report;
mod runner;

pub use config::RunnerConfig;
pub use error::RunnerError;
pub use report::{
    AddF32Report, AddF64Report, AddScalarF64Report, BackendRunResult, BackendRunStatus,
    ComparisonSummary, F64Value, ScalarF64BackendObservation, ScalarF64ObservationReport,
    SumF64Report, TimingSummary,
};
pub use runner::Runner;
