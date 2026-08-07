use crate::OperationKind;

/// 計算バックエンドが提供する機能を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// `f32`配列加算を実行できるかどうか。
    pub add_f32: bool,

    /// `f64`配列加算を実行できるかどうか。
    pub add_f64: bool,

    /// `f64`スカラー加算を実行できるかどうか。
    pub add_scalar_f64: bool,

    /// `f64`配列の合計を計算できるかどうか。
    pub sum_f64: bool,

    /// 1命令または1ループ単位で処理する`f32`要素数の目安。
    ///
    /// Scalar実装では`1`、256-bit AVX実装では`8`です。
    pub vector_width_f32: usize,

    /// 1命令または1ループ単位で処理する`f64`要素数の目安。
    ///
    /// 未対応では`0`、Scalar実装では`1`、256-bit AVX実装では`4`です。
    pub vector_width_f64: usize,
}

impl BackendCapabilities {
    /// Scalarの`f32`配列加算能力を生成します。
    #[must_use]
    pub const fn scalar_add_f32() -> Self {
        Self {
            add_f32: true,
            add_f64: false,
            add_scalar_f64: false,
            sum_f64: false,
            vector_width_f32: 1,
            vector_width_f64: 0,
        }
    }

    /// 256-bit AVXの`f32`配列加算能力を生成します。
    #[must_use]
    pub const fn avx_add_f32() -> Self {
        Self {
            add_f32: true,
            add_f64: false,
            add_scalar_f64: false,
            sum_f64: false,
            vector_width_f32: 8,
            vector_width_f64: 0,
        }
    }

    /// `f64`配列加算能力を追加します。
    #[must_use]
    pub const fn with_add_f64(mut self, vector_width_f64: usize) -> Self {
        self.add_f64 = true;
        self.vector_width_f64 = vector_width_f64;
        self
    }

    /// `f64`スカラー加算能力を追加します。
    #[must_use]
    pub const fn with_add_scalar_f64(mut self) -> Self {
        self.add_scalar_f64 = true;
        self
    }

    /// `f64`配列合計能力を追加します。
    #[must_use]
    pub const fn with_sum_f64(mut self) -> Self {
        self.sum_f64 = true;
        self
    }

    /// 指定された演算をサポートするか返します。
    #[must_use]
    pub const fn supports(self, operation: OperationKind) -> bool {
        match operation {
            OperationKind::AddF32 => self.add_f32,
            OperationKind::AddF64 => self.add_f64,
            OperationKind::AddScalarF64 => self.add_scalar_f64,
            OperationKind::SumF64 => self.sum_f64,
        }
    }
}
