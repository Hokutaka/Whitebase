# Whitebase Core API

`whitebase-core` は、Whitebase の計算バックエンドを統一した Rust API として公開します。

> [!IMPORTANT]
> Whitebase は学習・実験用のリポジトリです。Operation や Backend の追加に伴い、Core API は変更される可能性があります。

## 対象

この文書は `whitebase-core` crate が公開する API を対象とします。

- Backend の列挙と利用可否・Capability の確認
- Backend を指定した計算処理
- Core が公開する共通型とエラー

時間計測、Warmup、Backend 間の結果比較、Benchmark Report の生成は Core ではなく `whitebase-runner` の責務です。

## Crate

```toml
[dependencies]
whitebase-core = { path = "crates/whitebase-core" }
```

現在の crate version は `0.1.0` で、workspace 内部利用を前提として `publish = false` です。

## API 一覧

| API | 用途 |
|---|---|
| `Whitebase::new()` | 標準 Backend を登録した Core インスタンスを生成 |
| `Whitebase::default()` | `Whitebase::new()` と同等 |
| `Whitebase::backends()` | 登録されている全 Backend の情報を取得 |
| `Whitebase::backend_info(kind)` | 指定 Backend の情報を取得 |
| `Whitebase::add_f32(...)` | `f32` 配列を要素ごとに加算 |
| `Whitebase::add_f64(...)` | `f64` 配列を要素ごとに加算 |
| `Whitebase::add_scalar_f64(...)` | 2つの `f64` scalar を加算 |
| `Whitebase::sum_f64(...)` | `f64` 配列を1つの値へ合計 |

## 基本的な利用例

```rust
use whitebase_core::{BackendKind, Whitebase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core = Whitebase::new();

    let lhs = [1.0_f64, 2.0, 3.0];
    let rhs = [10.0_f64, 20.0, 30.0];
    let mut output = [0.0_f64; 3];

    core.add_f64(
        BackendKind::RustScalar,
        &lhs,
        &rhs,
        &mut output,
    )?;

    assert_eq!(output, [11.0, 22.0, 33.0]);

    let sum = core.sum_f64(BackendKind::RustScalar, &output)?;
    assert_eq!(sum, 66.0);

    Ok(())
}
```

## Operation 一覧

Core が公開する Operation は `OperationKind` で表現されます。

| `OperationKind` | Core API | 入力 | 出力 |
|---|---|---|---|
| `AddF32` | `add_f32` | `&[f32]`, `&[f32]`, `&mut [f32]` | `Result<(), ComputeError>` |
| `AddF64` | `add_f64` | `&[f64]`, `&[f64]`, `&mut [f64]` | `Result<(), ComputeError>` |
| `AddScalarF64` | `add_scalar_f64` | `f64`, `f64` | `Result<f64, ComputeError>` |
| `SumF64` | `sum_f64` | `&[f64]` | `Result<f64, ComputeError>` |

### `add_f32`

```rust
pub fn add_f32(
    &self,
    kind: BackendKind,
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<(), ComputeError>
```

指定 Backend で `lhs[i] + rhs[i]` を計算し、`output[i]` に書き込みます。

`lhs`、`rhs`、`output` の長さが一致しない場合は `ComputeError::LengthMismatch` になります。

### `add_f64`

```rust
pub fn add_f64(
    &self,
    kind: BackendKind,
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<(), ComputeError>
```

指定 Backend で `f64` 配列を要素ごとに加算します。

`lhs`、`rhs`、`output` の長さが一致しない場合は `ComputeError::LengthMismatch` になります。

### `add_scalar_f64`

```rust
pub fn add_scalar_f64(
    &self,
    kind: BackendKind,
    lhs: f64,
    rhs: f64,
) -> Result<f64, ComputeError>
```

指定 Backend で2つの `f64` scalar を加算します。

現在、Scalar Backend がこの Operation を公開します。AVX/SIMD Backend は `OperationUnsupported` になります。

### `sum_f64`

```rust
pub fn sum_f64(
    &self,
    kind: BackendKind,
    input: &[f64],
) -> Result<f64, ComputeError>
```

指定 Backend で `f64` 配列を1つの値へ reduce します。

空配列の合計は `0.0` です。

## Backend

### `BackendKind`

現在定義されている Backend は次のとおりです。

| `BackendKind` | 表示名 | 実装 |
|---|---|---|
| `RustScalar` | `Rust Scalar` | Rust Scalar |
| `RustSimd` | `Rust SIMD` | Rust AVX/SIMD |
| `CppScalar` | `C++ Scalar` | C++ Scalar |
| `CppAvx` | `C++ AVX` | C++ AVX |
| `AssemblyScalar` | `Assembly Scalar` | Assembly Scalar |
| `AssemblyAvx` | `Assembly AVX` | Assembly AVX |
| `WindowsGnuCppScalar` | `Windows GCC Scalar` | Windows GNU / GCC Scalar |
| `WindowsGnuCppAvx` | `Windows GCC AVX` | Windows GNU / GCC AVX |
| `WindowsGnuAssemblyScalar` | `Windows NASM Scalar` | Windows GNU / NASM Scalar |
| `WindowsGnuAssemblyAvx` | `Windows NASM AVX` | Windows GNU / NASM AVX |

Windows GNU Backend は `x86_64-pc-windows-msvc` で Core に追加登録されます。

その他の標準 Backend は Rust Scalar/SIMD、C++ Scalar/AVX、Assembly Scalar/AVX です。実際に利用できるかどうかは Platform、CPU、Native library の状態に依存します。

### Operation 対応

Capability 上の現在の対応は次のとおりです。

| Backend 種別 | `AddF32` | `AddF64` | `AddScalarF64` | `SumF64` |
|---|:---:|:---:|:---:|:---:|
| Scalar | ✓ | ✓ | ✓ | ✓ |
| AVX / SIMD | ✓ | ✓ | — | ✓ |

AVX/SIMD Backend の利用可否は CPU の AVX 対応など実行環境にも依存します。

### Vector width

`BackendCapabilities` は Operation 対応に加え、処理幅の目安を公開します。

| 実装 | `vector_width_f32` | `vector_width_f64` |
|---|---:|---:|
| Scalar | `1` | `1` (`f64`対応時) |
| 256-bit AVX | `8` | `4` (`f64`対応時) |

## Backend 情報 API

### `backends`

```rust
pub fn backends(&self) -> Vec<BackendInfo>
```

Core に登録されている Backend 全件について `BackendInfo` を返します。

```rust
pub struct BackendInfo {
    pub kind: BackendKind,
    pub capabilities: BackendCapabilities,
    pub available: bool,
}
```

`available` は現在の実行環境で Backend を利用できるかを表します。

### `backend_info`

```rust
pub fn backend_info(
    &self,
    kind: BackendKind,
) -> Result<BackendInfo, ComputeError>
```

指定 Backend の情報を返します。

Core に登録されていない Backend を指定した場合は `ComputeError::BackendNotRegistered` になります。

### Capability の確認

```rust
use whitebase_core::{OperationKind, Whitebase};

let core = Whitebase::new();

for backend in core.backends() {
    if backend.available
        && backend.capabilities.supports(OperationKind::SumF64)
    {
        println!("{} supports SumF64", backend.kind.display_name());
    }
}
```

`BackendCapabilities` が公開する主な Field:

| Field | 型 | 意味 |
|---|---|---|
| `add_f32` | `bool` | `AddF32` 対応 |
| `add_f64` | `bool` | `AddF64` 対応 |
| `add_scalar_f64` | `bool` | `AddScalarF64` 対応 |
| `sum_f64` | `bool` | `SumF64` 対応 |
| `vector_width_f32` | `usize` | `f32` の処理幅の目安 |
| `vector_width_f64` | `usize` | `f64` の処理幅の目安 |

`supports(OperationKind)` で Operation 単位の対応確認もできます。

## Error

Core の計算 API は `ComputeError` を返します。

| Variant | 条件 |
|---|---|
| `LengthMismatch` | 配列演算の入力・出力長が一致しない |
| `BackendUnavailable` | Backend は登録されているが現在の環境では利用できない |
| `OperationUnsupported` | Backend が指定 Operation をサポートしていない |
| `BackendFailure` | Backend 内部または Native adapter の処理に失敗 |
| `BackendNotRegistered` | 指定された Backend が Core に登録されていない |

### `LengthMismatch`

```rust
ComputeError::LengthMismatch {
    lhs_len,
    rhs_len,
    output_len,
}
```

`add_f32` と `add_f64` では3つの配列長が一致している必要があります。

### Backend の選択

Core は指定された `BackendKind` を自動的に別 Backend へ fallback しません。

そのため利用側では、必要に応じて事前に

1. `backend_info()` または `backends()` で登録状態を確認
2. `capabilities.supports(...)` で Operation 対応を確認
3. `available` で実行環境上の利用可否を確認

してから Operation を呼び出せます。

## 公開型

`whitebase-core` は次の型を公開します。

| 型 | 説明 |
|---|---|
| `Whitebase` | 統一計算 API |
| `BackendInfo` | Backend の Capability と利用可否 |
| `BackendKind` | Backend 識別子 |
| `BackendCapabilities` | Backend の Operation 対応と処理幅 |
| `OperationKind` | Operation 識別子 |
| `ComputeBackend` | Backend 実装が満たす共通 trait |
| `ComputeError` | Core/Backend の計算 Error |

次の alias も公開されます。

| Alias | 元の型 |
|---|---|
| `Backend` | `BackendKind` |
| `Capabilities` | `BackendCapabilities` |
| `Error` | `ComputeError` |

## Core と Runner の責務

Core は「指定 Backend で1回の計算を行う」ための API です。

```text
Caller
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Rust / C++ / Assembly
```

Benchmark や観察用途では Runner が Core を利用し、Warmup、複数回計測、参照 Backend との比較、Report の生成を追加します。

```text
Caller
  ↓
Whitebase Runner
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Rust / C++ / Assembly
```

HTTP API の Benchmark Endpoint などは Runner を経由します。Core API は HTTP/JSON のような transport を持たず、Rust 内から直接利用する計算 API です。

## HTTP API との対応

HTTP API と Core API は同じ計算系を利用しますが、1対1の transport wrapper ではありません。

| 概念 | Core API | HTTP API |
|---|---|---|
| `f32` 配列加算 | `Whitebase::add_f32` | Benchmark `add-array` + `precision: "f32"` |
| `f64` 配列加算 | `Whitebase::add_f64` | Benchmark `add-array` + `precision: "f64"` |
| Scalar `f64` 加算 | `Whitebase::add_scalar_f64` | `POST /api/observations/add-scalar-f64` |
| `f64` 配列合計 | `Whitebase::sum_f64` | Benchmark `sum-f64` + `precision: "f64"` |

HTTP Benchmark API は入力配列を Server 側で生成し、Runner による Warmup・複数回計測・Backend 比較を行います。Core API は利用側から配列を直接受け取り、指定 Backend の演算を1回実行します。

See [HTTP API](HTTP-API.ja.md) for HTTP endpoint and request/response details.
