use crate::{BackendCapabilities, ComputeError};

/// Whitebaseで利用できる計算バックエンドの種類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BackendKind {
    /// RustによるScalar実装。
    RustScalar,

    /// RustによるSIMD実装。
    RustSimd,

    /// C++によるScalar実装。
    CppScalar,

    /// C++によるAVX実装。
    CppAvx,

    /// AssemblyによるScalar実装。
    AssemblyScalar,

    /// AssemblyによるAVX実装。
    AssemblyAvx,
}

impl BackendKind {
    /// 表示用の安定した名前を返します。
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::RustScalar => "Rust Scalar",
            Self::RustSimd => "Rust SIMD",
            Self::CppScalar => "C++ Scalar",
            Self::CppAvx => "C++ AVX",
            Self::AssemblyScalar => "Assembly Scalar",
            Self::AssemblyAvx => "Assembly AVX",
        }
    }
}

/// 計算バックエンドが満たす共通インターフェースです。
///
/// 時間計測、結果保存、バックエンド間比較は担当しません。
/// それらはRunner層が担当します。
pub trait ComputeBackend: Send + Sync {
    /// バックエンドの種類を返します。
    fn kind(&self) -> BackendKind;

    /// バックエンドの能力を返します。
    fn capabilities(&self) -> BackendCapabilities;

    /// 現在の実行環境で利用可能か返します。
    fn is_available(&self) -> bool;

    /// 2つの`f32`配列を要素ごとに加算します。
    fn add_f32(&self, lhs: &[f32], rhs: &[f32], output: &mut [f32]) -> Result<(), ComputeError>;
}
