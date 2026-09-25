# Whitebase

日本語 | [English](Overview.en.md)

Whitebase は、計算処理・実行経路・インターフェースの違いを、小さい単位で実装し、観察・記録・比較・可視化するための学習・実験用リポジトリです。

API や構成は、実験対象の追加に合わせて変更される可能性があります。

## 目的

Whitebase の目的は、異なる実装や実行環境を同じ観測基盤へ接続し、その違いを確認できる形にすることです。

例えば、次のような対象を扱います。

- Scalar と SIMD
- Rust / C++ / Assembly
- Cerune VM
- C ABI と FFI
- Debug / Release ビルド
- Tauri IPC
- HTTP + JSON
- WebAssembly
- デスクトップ UI とブラウザ UI
- Backend ごとの Capability、利用可否、実行結果
- IEEE 754 の値・ビット表現の観測

Whitebase は、すべての実装を1つに隠蔽することよりも、経路ごとの差を確認できることを重視します。

## Ceruneとの境界

### 現在できること

Whitebase は、Rust・C++・Assembly の計算 Backend に加え、Cerune VM を共通の計算基盤へ接続できます。

現在の Cerune 統合では、Cerune のソースを bytecode へコンパイルし、関数を解決したうえで Cerune VM から `AddScalarF64` を実行します。

```text
Cerune Source
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
    ↓
Whitebase Runner
```

[`ComputeBackend`](../crates/backend/whitebase-backend-contract/src/backend.rs) は、Whitebase の計算 Backend が満たす共通契約です。

[`BackendCapabilities`](../crates/backend/whitebase-backend-contract/src/capabilities.rs) は、各 Backend がどの Operation を提供するかと、配列処理幅の目安を表します。Capability と現在の実行環境での利用可否は別に扱います。

[`Runner`](../crates/compute/whitebase-runner/src/runner.rs) は、Core の演算を利用して、計測・比較・観測を構成します。

[`F64Value`](../crates/compute/whitebase-runner/src/report.rs) は、`f64` の値と IEEE 754 ビット表現をまとめて保持します。

Cerune VM も Core から見ると通常の `ComputeBackend` であり、Cerune 固有の bytecode、VM value、function handle を Core API へ公開しません。

### 連携時の役割分担

Cerune と Whitebase の責務は分離します。

| 担当 | 役割 |
| --- | --- |
| Cerune | 構文・型・演算の意味、診断、Cerune IR、bytecode / VM、各種 Emitter |
| Whitebase Backend Integration | Cerune VM や生成 Artifact を Whitebase Backend として接続するための準備・適合処理 |
| Whitebase Core | Backend の登録、Capability 確認、利用可否確認、dispatch |
| Whitebase Runner | 計測、比較、観測、Backend ごとの実行状態の集約 |
| 利用側の実行環境 | 実行許可、外部 toolchain の提供、必要に応じた隔離や資源制限 |

現在実装されているのは Cerune VM 経路です。

Cerune が生成する C / LLVM / QBE / WAT / Assembly / Native Object などを Whitebase から実行する Artifact 経路は、将来の拡張として分離して扱います。

外部 compiler、assembler、linker、Wasm runtime などの探索・起動・availability 判定を Cerune 自体の責務にはしません。また、それらを `whitebase-core` へ直接持ち込みません。

Cerune 統合の詳細は [Cerune Integration](Layer/Cerune-Integration.md) を参照してください。

### 対応状況と比較結果

Whitebase では、次の状態を区別します。

- Operation を Capability として提供しているか
- Backend が現在の環境で利用可能か
- 実際の処理が成功したか
- 実行結果が参照結果と一致したか
- 複数 Backend の結果が互いに一致したか

Benchmark では Backend ごとに `Completed / Unavailable / Failed` を保持します。

Scalar `f64` observation でも同じ考え方を使い、1つの Backend が利用不可または実行失敗になっても、他の Backend の観測を継続します。

`allBackendsMatch` は、実行に成功した Backend の結果ビットだけを比較します。成功した Backend が1つもない場合は `false` です。

`matchesReferenceBits` は別の指標です。各 Backend の結果ビットが、入力された10進文字列を正確に加算したあと `f64` へ丸めた参照値と一致するかを表します。

そのため、例えば `0.1 + 0.2` では、

- 実行に成功した Backend 同士は同じビットを返す
- その一方で exact decimal reference とは1 bit 異なる

という状態を観測できます。

将来 Artifact 経路を追加するときも、「生成できる」「ビルドできる」「ロードできる」「実行できる」「結果が一致する」を同一の状態として扱いません。

### 記録と実行許可

Cerune VM 経路では、コンパイルと関数解決を演算呼び出しの前に完了させます。

```text
Preparation

source
  ↓
compile to bytecode
  ↓
resolve function

Measurement / Observation

arguments
  ↓
VM invocation
  ↓
result
```

これにより、通常の Backend 演算時間へ Cerune のコンパイル時間を混ぜません。

将来 Artifact 経路を追加する場合も、可能な限り build / link / load を Preparation として分離し、実際の演算呼び出しと区別します。

コンパイル時間や link 時間そのものを測定する場合は、通常の演算 Benchmark とは別の観測対象として扱います。

外部 toolchain を利用する経路を追加しても、観測データや生成物そのものへコマンド実行権限を持たせません。HTTP、Tauri、Browser などの公開 Interface に、必要性のない外部コマンド実行機能を追加しない方針です。

## 動作確認方法

Whitebase は、主に次の3つの方法で動作を確認できます。

| 実行方法 | Server | Whitebase の呼び出し経路 |
| --- | ---: | --- |
| Tauri アプリ | 不要 | Tauri → Tauri Commands → Interface → Runner / Core |
| GitHub Pages | 任意 | Browser → WASM → Interface → Runner / Core、または Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |
| ローカル Web アプリ | 任意 | Browser → WASM → Interface → Runner / Core、または Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |

Browser 環境では、アプリ起動時に実行経路を判定します。

1. Tauri 環境では Tauri Commands を使用します。
2. 通常の Browser 環境では Whitebase Server の Health API を確認します。
3. Health API へ接続できた場合は HTTP API を使用します。
4. Server へ接続できない場合は WebAssembly を初期化して使用します。

選択された実行経路は、そのページセッション中は固定されます。

Server の起動・停止後に実行経路を切り替える場合は、ページを再読み込みしてください。

### Tauriアプリ

Whitebase App を起動します。

```powershell
npm --prefix apps\whitebase-app run tauri dev
```

または Whitebase Operations を使用します。

```powershell
scripts\ops.bat dev
```

Tauri アプリは Whitebase Server を必要としません。

計算処理は Tauri Commands から `whitebase-interface` を経由し、処理内容に応じて Runner または Whitebase Core を利用します。

```text
Tauri WebView
  ↓
Tauri IPC / invoke()
  ↓
Tauri Host (src-tauri)
  ↓
whitebase-tauri-api
  ↓
whitebase-interface
  ↓
Runner / Whitebase Core
  ↓
Backend
```

Native 環境では、現在の Platform と Backend の availability に応じて Rust、C++、Assembly、Cerune VM などが利用されます。

### GitHub Pages

GitHub Pages 上の Whitebase App は、起動時に Whitebase Server の Health API を確認し、接続可否に応じて実行経路を選択します。

Browser が loopback HTTP 接続を許可し、Whitebase Server が利用可能な場合は HTTP API を使用します。

```text
Browser
  ↓
HTTP
  ↓
Whitebase Server
  ↓
whitebase-http-api
  ↓
whitebase-interface
  ↓
Runner / Whitebase Core
  ↓
Backend
```

Server へ接続できない場合は WebAssembly を使用します。

```text
Browser
  ↓
WebAssembly
  ↓
whitebase-interface
  ↓
Runner / Whitebase Core
  ↓
WASM で利用可能な Backend
```

WASM 経路でも Core / Runner の構造は維持されます。どの Backend が実行対象になるかは、Operation の Capability と wasm32 環境での availability によって決まります。

Rust SIMD Backend は wasm32 では WebAssembly SIMD128 を使用します。現在の Whitebase WebAssembly artifact は SIMD128 対応 runtime を実行要件とし、SIMD128 非対応 runtime 向けの baseline artifact は提供していません。

### ローカルWebアプリ

ローカルのフロントエンド開発 Server を起動します。

```powershell
npm --prefix apps\whitebase-app run dev
```

または、WebAssembly の開発用ビルドを含めて起動します。

```powershell
scripts\ops.bat web-dev
```

Whitebase Server を起動していない場合は WebAssembly 経路が使用されます。

Whitebase Server 経路を確認する場合は、別のターミナルで Server を起動します。

```powershell
cargo run -p whitebase-server
```

その後、ローカル Web アプリを再読み込みします。

Server の Health API へ接続できれば HTTP API 経路が選択されます。

### WebAssemblyのビルド構成

開発用 WebAssembly と Release 用 WebAssembly は、同じ `apps/whitebase-app/src/wasm` へ生成されますが、ビルド構成が異なります。

開発時は Debug 構成の WebAssembly を使用します。

```powershell
scripts\ops.bat web-dev
```

Browser 用の Release 成果物を生成する場合は、次を使用します。

```powershell
scripts\ops.bat wasm-build
```

Web frontend を Release build する場合は、Release 構成の WebAssembly を生成してから frontend を build します。

```powershell
scripts\ops.bat web-build
```

### ビルド構成による違い

開発環境と Release 環境では、次の項目に差が出る可能性があります。

- Rust の最適化レベル
- WebAssembly の最適化レベル
- C++ / Assembly の Debug / Release library
- SIMD 処理の性能
- Benchmark 結果
- 実行ファイルや bundle の出力先

特に Benchmark 結果は Debug build と Release build で大きく異なる可能性があります。

性能を比較する場合は、同じ Platform・同じ build profile・同じ入力条件で比較してください。性能比較の基準には Release 構成を推奨します。

計算結果の正当性は、build profile にかかわらず Backend 間で検証できることを前提とします。

## アーキテクチャ

Whitebase は、責務ごとに Backend Contract、Backend Implementation、Backend Integration、Core、Runner、Interface、Presentation を分離しています。

主な Layer は次のとおりです。

| Layer | 役割 | 主な Component |
| --- | --- | --- |
| L1 Backend Contract | Backend 共通契約 | `whitebase-backend-contract` |
| L2 Backend Implementation | 実計算 | Rust / C++ / Assembly |
| L3 Backend Integration | Adapter と Backend 統合 | C++ / Assembly / Windows GNU / Cerune VM adapters、`whitebase-backend-bridge` |
| L4 Core | Pure Compute | `whitebase-core` |
| L5 Runner | Applied Compute | `whitebase-runner` |
| L6 Interface | Application Boundary / Interface Adapter | `whitebase-interface`、HTTP、Tauri、WASM |
| L7 Presentation | User Facing | Whitebase App / Browser UI |

Tauri では IPC を使用し、通常の Browser では起動時に HTTP API または WebAssembly の実行経路を選択します。

各 Application Interface は `whitebase-interface` を共通の Application Boundary として利用します。

`whitebase-interface` は transport に依存せず、Pure Compute を公開する場合は Whitebase Core、比較・計測・観測などの Applied Compute を公開する場合は Runner を利用します。

HTTP では `whitebase-http-api` が HTTP / JSON / Axum 固有処理を担当し、`whitebase-server` はその Interface Adapter を起動する Host として扱います。

Tauri Commands と WebAssembly も Whitebase 内部の Core / Runner を直接公開せず、それぞれの実行環境に合わせて `whitebase-interface` を利用します。

FFI Boundary と Operations Plane は、この7 Layerとは別に扱います。

- `whitebase-c-api`: Native consumer 向け C ABI
- Control Center / scripts / CI: build、test、run、release、環境確認などの Operations

詳細は [Layer Overview](Layer/Layer-Overview.md) を参照してください。

![モジュール構成図](/docs/diagrams/structure/architecture.svg)

## 構成図

### モジュール構成図

矢印は、モジュール間の依存・利用関係または計算結果の受け渡し方向を示します。

破線の枠線で示したモジュールは、将来的な追加を予定しています。

![モジュール構成図](/docs/diagrams/structure/module.svg)

### 利用構成図

`whitebase-core` を Pure Compute の中心とし、利用する環境に応じた Interface または FFI Boundary を通して呼び出します。

比較・計測・観測が必要な場合は `whitebase-runner` を経由します。

![利用構成図](/docs/diagrams/structure/usage.svg)

## 実装済みのベンチマークについて

Whitebase Runner の Benchmark は、概ね次の流れで処理します。

```text
入力生成
  ↓
参照 Backend を準備
  ↓
Warmup
  ↓
複数回実行して時間を計測
  ↓
最小・最大・平均・合計時間を集計
  ↓
参照結果との誤差を比較
  ↓
Backend ごとの状態を Report にまとめる
  ↓
Interface を通して Tauri / Browser へ返す
```

Benchmark の Backend result は、`Completed / Unavailable / Failed` の状態を持ちます。

Benchmark 結果は、build profile、CPU、cache、memory bandwidth、OS scheduling などに影響されます。

性能比較では Debug ではなく Release 構成を推奨します。

### 時間計測の扱い

Whitebase Runner は、各 measured iteration を個別に計測します。

1回でも計測時間が `Duration::ZERO` になった場合、その値を `0 ns` の実行時間として扱いません。

この状態は、演算が現在の timer resolution より短く、単発実行時間を正しく観測できなかったことを意味するため、Timing を `TooFastToMeasure` として報告します。

`TooFastToMeasure` は Backend の実行失敗ではありません。

- Backend Status: `Completed`
- Result comparison: 通常どおり実行
- Timing: `TooFastToMeasure`
- Mean / Minimum / Maximum / Total: 報告しない
- Fastest / Speedup: 計算対象外

Whitebase は、timer resolution 未満の実行時間を推定値へ置き換えず、観測不能であることをそのまま結果として扱います。

### Scalar f64 observation

Scalar `f64` observation は、Benchmark とは別に、入力された10進文字列と各 Backend の `f64` 加算結果を観測する機能です。

```text
decimal input
  ↓
exact decimal reference
  ↓
reference rounded to f64
  ↓
各 AddScalarF64 Backend を観測
  ↓
value / decimal / IEEE 754 bits を比較
```

Backend ごとの状態は次の3種類です。

- `Completed`: 結果と `matchesReferenceBits` を返す
- `Unavailable`: 現在の環境で利用できない
- `Failed`: Backend の実行に失敗した

1つの Backend が `Unavailable` または `Failed` でも、他の Backend の観測は継続します。

`allBackendsMatch` は成功した Backend 同士の結果ビット比較、`matchesReferenceBits` は exact decimal reference から作った `f64` 参照値との比較です。

この2つは同じ意味ではありません。

### Core API

Rust API、Operation、Backend、Capability、Error の詳細については、[Core API](api/Core-API.ja.md) を参照してください。

### HTTP API

`whitebase-http-api` は、ローカル HTTP / JSON 向けの Interface Adapter を提供します。

現在の主な Endpoint は次のとおりです。

- `GET /api/health`
- `POST /api/observations/add-scalar-f64`
- `POST /api/benchmarks/run`
- `POST /api/benchmarks/add-array`
- `POST /api/benchmarks/add-f32`（互換用）

`whitebase-server` は、その Adapter を起動する Host として扱います。

Endpoint、Request / Response の詳細については、[HTTP API](api/HTTP-API.ja.md) を参照してください。
