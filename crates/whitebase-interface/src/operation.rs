/// Whitebaseが提供する演算の種類です。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationKind {
    /// 2つの`f32`配列を要素ごとに加算します。
    AddF32,

    /// 2つの`f64`配列を要素ごとに加算します。
    AddF64,

    /// 2つの`f64`スカラー値を加算します。
    AddScalarF64,

    /// `f64`配列の要素を合計します。
    SumF64,
}
