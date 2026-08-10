# Whitebase

学習・実験用リポジトリです。
APIや構成は頻繁に変わる予定です。

## 目的

仕組みを、小さい単位で実装し、観察・記録・比較・可視化することです。
例えば

- ScalarとSIMD
- Rust/C++/Assembly
- C ABIとFFI
- Debug / Releaseビルド
- TauriのIPC
- HTTP + JSON
- WebAssembly
- デスクトップUIとブラウザUI

## 動作確認方法

Whitebaseは、以下の3つの方法で動作を確認できます。

| 実行方法 | Server | Whitebaseの呼び出し経路 |
| --- | ---: | --- |
| Tauriアプリ | 不要 | Tauri → Tauri Commands → Interface → Runner / Core |
| GitHub Pages | 任意 | Browser → WASM → Interface → Runner / Core、またはloopback接続が利用可能な場合は Browser → HTTP → Whitebase Server → Interface → Runner / Core |
| ローカルWebアプリ | 任意 | Browser → WASM → Interface → Runner / Core、または Browser → HTTP → Whitebase Server → Interface → Runner / Core |

Browser環境では、アプリ起動時に実行経路を判定します。

1. Tauri環境の場合はTauri Commandsを使用します。
2. Browser環境ではWhitebase ServerのHealth APIを確認します。
3. Whitebase ServerのHealth APIへ接続できた場合はHTTP APIを使用します。
4. Serverへ接続できない場合はWebAssemblyを使用します。

選択された実行経路は、そのセッション中は変更されません。
Serverの起動・停止後に実行経路を切り替える場合は、ページを再読み込みしてください。

### Tauriアプリ

Whitebase Appを起動します。

```powershell
npm --prefix apps\whitebase-app run tauri dev
```

またはWhitebase Operationsを使用します。

```powershell
scripts\ops.bat dev
```

TauriアプリはWhitebase Serverを必要としません。

計算処理はTauri CommandsからRunnerとWhitebase Coreを経由して、
利用可能なNative backendへルーティングされます。

```text
Tauri
→ Tauri Commands
→ Runner
→ Whitebase Core
→ Native backend
```

### GitHub Pages

GitHub Pages上のWhitebase Appは、起動時にWhitebase ServerのHealth APIを確認し、
接続可否に応じて実行経路を自動的に選択します。

ブラウザがloopback HTTP接続を許可し、Whitebase Serverが利用可能な場合は、
HTTP APIを使用します。ブラウザのセキュリティ設定や権限によってServerへ
接続できない場合は、WebAssemblyへフォールバックします。

```text
Browser
→ HTTP
→ Whitebase Server
→ Runner
→ Whitebase Core
→ Native backend
```

Whitebase Serverが利用できない、または接続できない場合は、
WebAssemblyを使用します。

```text
Browser
→ WebAssembly
→ Runner
→ Whitebase Core
→ Rust backend
```

WebAssembly環境では、Rust ScalarおよびRust SIMDバックエンドを利用できます。

Rust SIMDバックエンドは、wasm32環境ではWebAssembly SIMD128を使用します。

### ローカルWebアプリ

ローカルのフロントエンド開発サーバーを起動します。

```powershell
npm --prefix apps\whitebase-app run dev
```

または、WebAssemblyの開発用ビルドを含めて起動します。

```powershell
scripts\ops.bat web-dev
```

Whitebase Serverを起動していない場合は、WebAssembly経路が使用されます。

Whitebase Server経路を確認する場合は、別のターミナルでServerを起動します。

```powershell
cargo run -p whitebase-server
```

その後、ローカルWebアプリを再読み込みします。

Serverが利用可能であれば、HTTP API経路が選択されます。

### WebAssemblyのビルド構成

開発用WebAssemblyとRelease用WebAssemblyは、同じ
`apps/whitebase-app/src/wasm`へ生成されますが、ビルド構成が異なります。

開発時はDebug構成のWebAssemblyを使用します。

```powershell
scripts\ops.bat web-dev
```

ブラウザ用のRelease成果物を生成する場合は、以下を使用します。

```powershell
scripts\ops.bat wasm-build
```

WebフロントエンドをReleaseビルドする場合は、
Release構成のWebAssemblyを生成してからフロントエンドをビルドします。

```powershell
scripts\ops.bat web-build
```

### ビルド構成による違い

開発環境とRelease環境では、次の項目に差が出る可能性があります。

- Rustの最適化レベル
- WebAssemblyの最適化レベル
- C++およびAssemblyのDebug / Releaseライブラリ
- SIMD処理の性能
- ベンチマーク結果
- 実行ファイルやバンドルの出力先

特にベンチマーク結果は、DebugビルドとReleaseビルドで大きく異なります。
性能を比較する場合は、Release構成での結果を基準としてください。

計算結果の正当性は、ビルド構成にかかわらず各バックエンド間で
検証されることを前提とします。

## アーキテクチャ

TauriではIPCを使用し、通常のブラウザでは起動時に
HTTP APIまたはWebAssemblyの実行経路を選択します。

どのApplication Interfaceからも`whitebase-interface`を共通の境界として利用します。

`whitebase-interface`は、Pure ComputeではWhitebase Coreを直接利用し、
比較・計測・観測などのApplied ComputeではRunnerを利用します。

HTTP、Tauri、WebAssemblyはWhitebase内部のCore / Runnerを直接公開せず、
それぞれの実行環境に合わせて`whitebase-interface`を公開します。

![モジュール構成図](/docs/diagrams/structure/architecture.svg)

## 構成図

### モジュール構成図

矢印は、モジュール間の依存・利用関係または計算結果の受け渡し方向を示します。  
破線の枠線で示したモジュールは、将来的な追加を予定しています。

![モジュール構成図](/docs/diagrams/structure//module.svg)

### 利用構成図

ライブラリの利用方法の想定です。  
`whitebase-core`を計算処理の中心とし、利用する環境に応じた境界を通して呼び出します。  

![利用構成図](/docs/diagrams/structure/usage.svg)

## 実装済みのベンチマークについて
```text
入力生成
→ 参照バックエンドをウォームアップ
→ 複数回実行して時間を計測
→ 最小・最大・平均・合計時間を集計
→ 参照結果との誤差を比較
→ Tauri・ブラウザへレポートを返す
```

※ベンチマーク結果はビルド構成、CPU、キャッシュ、メモリ帯域、OSのスケジューリングなどに影響されます。性能比較ではDebugではなくRelease構成を推奨します。

### 時間計測の扱い

Whitebase Runnerは、各measured iterationを個別に計測します。

1回でも計測時間が`Duration::ZERO`になった場合、その値を
`0 ns`の実行時間として扱いません。

この状態は、演算が現在のタイマー分解能より短く、
単発実行時間を正しく観測できなかったことを意味するため、
Timingを`TooFastToMeasure`として報告します。

`TooFastToMeasure`はBackendの実行失敗ではありません。

- Backend Status: `Completed`
- Result comparison: 通常どおり実行
- Timing: `TooFastToMeasure`
- Mean / Minimum / Maximum / Total: 報告しない
- Fastest / Speedup: 計算対象外

Whitebaseは、タイマー分解能未満の実行時間を推定値へ置き換えず、
観測不能であることをそのまま結果として扱います。

### Core API

Rust API、演算、バックエンド、エラーの詳細については、
[Core API](api/Core-API.ja.md)を参照してください。

### HTTP API

Whitebase ServerはローカルHTTP/JSON APIを公開しています。

エンドポイントやリクエスト・レスポンスの詳細については、
[HTTP API](api/HTTP-API.ja.md)を参照してください。


