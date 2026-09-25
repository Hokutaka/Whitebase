# Whitebase Core API

[English](HTTP-API.en.md) | **日本語**

`whitebase-core` は、Whitebase に登録された計算 Backend を共通の Rust API から利用するための Core 層です。

> [!IMPORTANT]
> Whitebase は学習・実験用のリポジトリです。Operation、Backend、Capability は今後変更される可能性があります。

## 対象

この文書は `whitebase-core` crate が公開する API を対象とします。

Core の責務は次のとおりです。

- 標準 Backend の登録
- Backend の列挙
- Capability と利用可否の公開
- `BackendKind` を指定した1回の計算
- Backend / Operation / Error の共通型の公開

次の処理は Core の責務ではありません。

- Warmup
- 反復計測
- Backend 間の結果比較
- Benchmark Report の生成
- Scalar observation の集約
- HTTP / Tauri / WASM などの transport
- UI 表示

これらは `whitebase-runner`、`whitebase-interface`、各 transport / application 層が担当します。

## Crate

```toml
[dependencies]
whitebase-core = { path = "crates/compute/whitebase-core" }
```

現在の crate version は `0.1.0` で、workspace 内部利用を前提として `publish = false` です。

Core は Cerune 固有 API や外部 toolchain を直接公開しません。Cerune VM も Core から見ると通常の `ComputeBackend` です。

## API 一覧

| API | 用途 |
| --- | --- |
| `Whitebase::new()` | 標準 Backend を登録した Core instance を生成 |
| `Whitebase::default()` | `Whitebase::new()` と同等 |
| `Whitebase::backends()` | 登録されている全 Backend の情報を取得 |
| `Whitebase::backend_info(kind)` | 指定 Backend の情報を取得 |
| `Whitebase::add_f32(...)` | `f32` 配列を要素ごとに加算 |
| `Whitebase::add_f64(...)` | `f64` 配列を要素ごとに加算 |
| `Whitebase::add_scalar_f64(...)` | 2つの `f64` scalar を加算 |
| `Whitebase::sum_f64(...)` | `f64` 配列を1つの値へ合計 |

## 基本例

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

Core は Backend を自動選択しません。呼び出し側が `BackendKind` を指定します。

## Operation

Core が扱う Operation は `OperationKind` で表現されます。

| `OperationKind` | Core API | 入力 | 出力 |
| --- | --- | --- | --- |
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

Backend が `AddF32` を Capability として持たない場合は `OperationUnsupported`、現在の環境で利用できない場合は `BackendUnavailable` になります。

配列長の不一致は Backend 実装から `LengthMismatch` として返されます。

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

Native Scalar Backend に加え、`CeruneVm` がこの Operation を公開します。AVX / SIMD Backend は `AddScalarF64` を Capability として持ちません。

### `sum_f64`

```rust
pub fn sum_f64(
    &self,
    kind: BackendKind,
    input: &[f64],
) -> Result<f64, ComputeError>
```

指定 Backend で `f64` 配列を1つの値へ reduce します。

空配列の合計は Backend 契約上 `0.0` です。

## Backend

### `BackendKind`

現在定義されている Backend は次のとおりです。

| `BackendKind` | 表示名 | 実装 |
| --- | --- | --- |
| `RustScalar` | `Rust Scalar` | Rust Scalar |
| `RustSimd` | `Rust SIMD` | Rust SIMD |
| `CeruneVm` | `Cerune VM` | Cerune VM |
| `CppScalar` | `C++ Scalar` | C++ Scalar |
| `CppAvx` | `C++ AVX` | C++ AVX |
| `AssemblyScalar` | `Assembly Scalar` | Assembly Scalar |
| `AssemblyAvx` | `Assembly AVX` | Assembly AVX |
| `WindowsGnuCppScalar` | `Windows GCC Scalar` | Windows GNU / GCC Scalar |
| `WindowsGnuCppAvx` | `Windows GCC AVX` | Windows GNU / GCC AVX |
| `WindowsGnuAssemblyScalar` | `Windows NASM Scalar` | Windows GNU / NASM Scalar |
| `WindowsGnuAssemblyAvx` | `Windows NASM AVX` | Windows GNU / NASM AVX |

### 標準登録

通常の環境では、`Whitebase::new()` が次を登録します。

```text
Rust Scalar
Rust SIMD
Cerune VM
C++ Scalar
C++ AVX
Assembly Scalar
Assembly AVX
```

`x86_64-pc-windows-msvc` では、さらに次を登録します。

```text
Windows GCC Scalar
Windows GCC AVX
Windows NASM Scalar
Windows NASM AVX
```

登録されていることと、現在の実行環境で利用可能であることは別です。

## Capability

`BackendCapabilities` は Backend が何を実行できるかを表します。

### Operation 対応

現在の Backend class ごとの Capability は次のとおりです。

| Backend class | `AddF32` | `AddF64` | `AddScalarF64` | `SumF64` |
| --- | :---: | :---: | :---: | :---: |
| Native Scalar | ✓ | ✓ | ✓ | ✓ |
| AVX / SIMD | ✓ | ✓ | — | ✓ |
| Cerune VM | — | — | ✓ | — |

`CeruneVm` は現在 `AddScalarF64` 専用です。

Capability は「その Backend がその Operation を提供するか」を表します。実際に現在の環境で実行できるかどうかは `available` で別に確認します。

### Vector width

`BackendCapabilities` は配列処理の幅も公開します。

| Backend class / target | `vector_width_f32` | `vector_width_f64` |
| --- | ---: | ---: |
| Native Scalar | `1` | `1` |
| Rust SIMD (`aarch64` / `wasm32`) | `4` | `2` |
| 256-bit AVX / SIMD | `8` | `4` |
| Cerune VM | `0` | `0` |

`0` は、その配列型の処理幅を持たないことを表します。Cerune VM は scalar `f64` 加算のみを提供するため、現在の vector width は両方 `0` です。

### Capability の確認

```rust
use whitebase_core::{OperationKind, Whitebase};

let core = Whitebase::new();

for backend in core.backends() {
    if backend.available
        && backend
            .capabilities
            .supports(OperationKind::AddScalarF64)
    {
        println!(
            "{} supports AddScalarF64",
            backend.kind.display_name()
        );
    }
}
```

`BackendCapabilities` の主な Field:

| Field | 型 | 意味 |
| --- | --- | --- |
| `add_f32` | `bool` | `AddF32` 対応 |
| `add_f64` | `bool` | `AddF64` 対応 |
| `add_scalar_f64` | `bool` | `AddScalarF64` 対応 |
| `sum_f64` | `bool` | `SumF64` 対応 |
| `vector_width_f32` | `usize` | `f32` 配列処理幅の目安 |
| `vector_width_f64` | `usize` | `f64` 配列処理幅の目安 |

`supports(OperationKind)` で Operation 単位の確認もできます。

## Backend 情報 API

### `backends`

```rust
pub fn backends(&self) -> Vec<BackendInfo>
```

登録されている全 Backend の `BackendInfo` を返します。

```rust
pub struct BackendInfo {
    pub kind: BackendKind,
    pub capabilities: BackendCapabilities,
    pub available: bool,
}
```

- `kind`: Backend の識別子
- `capabilities`: Backend が提供する Operation
- `available`: 現在の実行環境で利用できるか

### `backend_info`

```rust
pub fn backend_info(
    &self,
    kind: BackendKind,
) -> Result<BackendInfo, ComputeError>
```

指定 Backend の情報を返します。

Core に登録されていない `BackendKind` を指定した場合は `BackendNotRegistered` になります。

## 実行前の判定順序

Core の各 Operation は概ね次の順序で判定します。

```text
Backend registered?
      ↓ yes
Operation supported?
      ↓ yes
Backend available?
      ↓ yes
invoke backend
```

したがって、主な Error は次の対応になります。

```text
not registered
  → BackendNotRegistered

registered, operation unsupported
  → OperationUnsupported

registered, supported, unavailable
  → BackendUnavailable

backend invocation failed
  → BackendFailure
```

Core は unavailable / failed な Backend を別 Backend へ自動 fallback しません。

## Cerune VM Backend

`CeruneVm` は L3 Backend Integration で Cerune VM を `ComputeBackend` へ適合させた Backend です。

```text
Cerune source
    ↓
compile to bytecode
    ↓
resolve function
    ↓
Cerune VM
    ↓
whitebase-cerune-vm-adapter
    ↓
whitebase-backend-bridge
    ↓
Whitebase Core
```

現在の Capability は `AddScalarF64` のみです。

Cerune 固有の bytecode、VM value、function handle は Core API に公開されません。Core から見ると `CeruneVm` は他の Backend と同じ `ComputeBackend` です。

Cerune の compile / function resolution は演算呼び出しそのものとは分離され、実行対象の準備時に行われます。

Cerune 統合全体の責務境界については [Cerune Integration](../Layer/Cerune-Integration.md) を参照してください。

## Error

Core の計算 API は `ComputeError` を返します。

| Variant | 条件 |
| --- | --- |
| `LengthMismatch` | 配列演算の入力・出力長が一致しない |
| `BackendUnavailable` | Backend は登録され、Operation に対応しているが現在の環境では利用できない |
| `OperationUnsupported` | Backend が指定 Operation を Capability として持たない |
| `BackendFailure` | Backend / adapter 内部の実行に失敗 |
| `BackendNotRegistered` | 指定 Backend が Core に登録されていない |

Core は Backend 内部 Error を必要に応じて `BackendFailure` へ写像します。

## 公開型

`whitebase-core` は次の型を公開します。

| 型 | 説明 |
| --- | --- |
| `Whitebase` | 統一計算 API |
| `BackendInfo` | Backend の Capability と利用可否 |
| `BackendKind` | Backend 識別子 |
| `BackendCapabilities` | Backend の Operation 対応と処理幅 |
| `OperationKind` | Operation 識別子 |
| `ComputeBackend` | Backend 実装が満たす共通 trait |
| `ComputeError` | Core / Backend の計算 Error |

次の alias も公開されます。

| Alias | 元の型 |
| --- | --- |
| `Backend` | `BackendKind` |
| `Capabilities` | `BackendCapabilities` |
| `Error` | `ComputeError` |

## Core と Runner

Core は「指定 Backend で1回の計算を実行する」層です。

```text
Caller
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Backend implementation / adapter
```

Runner は Core の上で、計測・比較・観測を組み立てます。

```text
Caller
  ↓
Whitebase Runner
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Backend implementation / adapter
```

たとえば scalar `f64` observation の `Completed / Unavailable / Failed` の集約や Backend 間の結果比較は Runner の責務であり、Core API の戻り値ではありません。

## HTTP API との対応

HTTP API と Core API は同じ計算系を利用しますが、単純な1対1 wrapper ではありません。

| 概念 | Core API | HTTP API |
| --- | --- | --- |
| `f32` 配列加算 | `Whitebase::add_f32` | Benchmark `add-array` + `precision: "f32"` |
| `f64` 配列加算 | `Whitebase::add_f64` | Benchmark `add-array` + `precision: "f64"` |
| Scalar `f64` 加算 | `Whitebase::add_scalar_f64` | `POST /api/observations/add-scalar-f64` |
| `f64` 配列合計 | `Whitebase::sum_f64` | Benchmark `sum-f64` + `precision: "f64"` |

HTTP Benchmark API は Server 側で入力を生成し、Runner を通じて Warmup、計測、比較を行います。

Scalar observation は Runner が複数 Backend を横断し、Backend 単位の状態と IEEE 754 の観測結果を Report にまとめます。

Endpoint、Request、Response の詳細は [HTTP API](HTTP-API.ja.md) を参照してください。
