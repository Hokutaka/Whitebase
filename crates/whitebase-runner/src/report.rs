use whitebase_core::{BackendKind, ComputeError};

/// 反復実行の計測結果です。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingSummary {
    /// 計測回数。
    pub iterations: usize,

    /// 合計実行時間。
    pub total_nanoseconds: u128,

    /// 最短実行時間。
    pub minimum_nanoseconds: u128,

    /// 最長実行時間。
    pub maximum_nanoseconds: u128,

    /// 平均実行時間。
    pub mean_nanoseconds: f64,
}

/// 参照結果との比較結果です。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ComparisonSummary {
    /// 全要素が許容誤差内で一致したかどうか。
    pub matches_reference: bool,

    /// 一致しなかった要素数。
    pub mismatch_count: usize,

    /// 検出された最大絶対誤差。
    pub maximum_absolute_error: f32,
}

/// 各バックエンドの実行状態です。
#[derive(Debug, Clone, PartialEq)]
pub enum BackendRunStatus {
    /// 計測と結果比較に成功しました。
    Completed {
        timing: TimingSummary,
        comparison: ComparisonSummary,
    },

    /// 現在の環境では利用できません。
    Unavailable,

    /// バックエンドの実行に失敗しました。
    Failed { error: ComputeError },
}

/// 1つのバックエンドに対するRunnerの結果です。
#[derive(Debug, Clone, PartialEq)]
pub struct BackendRunResult {
    /// 実行対象のバックエンド。
    pub backend: BackendKind,

    /// 実行結果。
    pub status: BackendRunStatus,
}

/// `f32`配列加算の計測・比較レポートです。
#[derive(Debug, Clone, PartialEq)]
pub struct AddF32Report {
    /// 入力配列の要素数。
    pub input_length: usize,

    /// 比較基準に使用したバックエンド。
    pub reference_backend: BackendKind,

    /// ウォームアップ回数。
    pub warmup_iterations: usize,

    /// 計測回数。
    pub measured_iterations: usize,

    /// 結果比較時の絶対誤差許容値。
    pub absolute_tolerance: f32,

    /// 各バックエンドの実行結果。
    pub results: Vec<BackendRunResult>,
}
