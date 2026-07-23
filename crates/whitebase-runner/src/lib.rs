//! Whitebaseの演算実行、計測、結果比較を提供します。

#![forbid(unsafe_code)]

mod config;
mod error;
mod report;
mod runner;

pub use config::RunnerConfig;
pub use error::RunnerError;
pub use report::{
    AddF32Report, BackendRunResult, BackendRunStatus, ComparisonSummary, TimingSummary,
};
pub use runner::Runner;
