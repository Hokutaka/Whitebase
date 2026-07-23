use crate::OperationKind;

/// 計算バックエンドが提供する機能を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendCapabilities {
    /// `f32`配列加算を実行できるかどうか。
    pub add_f32: bool,

    /// 1命令または1ループ単位で処理する`f32`要素数の目安。
    ///
    /// Scalar実装では`1`、256-bit AVX実装では`8`です。
    pub vector_width_f32: usize,
}

impl BackendCapabilities {
    /// Scalarの`f32`配列加算能力を生成します。
    #[must_use]
    pub const fn scalar_add_f32() -> Self {
        Self {
            add_f32: true,
            vector_width_f32: 1,
        }
    }

    /// 256-bit AVXの`f32`配列加算能力を生成します。
    #[must_use]
    pub const fn avx_add_f32() -> Self {
        Self {
            add_f32: true,
            vector_width_f32: 8,
        }
    }

    /// 指定された演算をサポートするか返します。
    #[must_use]
    pub const fn supports(self, operation: OperationKind) -> bool {
        match operation {
            OperationKind::AddF32 => self.add_f32,
        }
    }
}
