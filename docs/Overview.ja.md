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

## Primerとの境界

### 現在できること

Whitebaseは、組み込みのRust・C++・Assembly演算を共通の計算APIへ接続し、結果や実行時間を比較します。Primerのソースや生成物を受け取ってビルド・実行する機能は、まだありません。

- [`ComputeBackend`](../crates/backend/whitebase-backend-contract/src/backend.rs)は、配列加算などの決まった演算を呼び出す契約です。
- [`BackendCapabilities`](../crates/backend/whitebase-backend-contract/src/capabilities.rs)は、その演算への対応と処理幅を表します。`is_available()`は現在の環境での利用可否です。外部コンパイラの対応状況や実行権限ではありません。
- [`Runner`](../crates/compute/whitebase-runner/src/runner.rs)は、Coreの演算を反復して計測・比較します。任意のソースを外部コンパイラでビルドするRunnerではありません。
- [`F64Value`](../crates/compute/whitebase-runner/src/report.rs)は、演算から直接受け取った値とビット表現を保持します。外部プログラムの標準出力から得た観測とは区別します。

### 連携時の役割分担

以下は連携時に守る境界であり、既に利用できるAPIを示すものではありません。

| 担当 | 役割 |
| --- | --- |
| Primer | 構文・型・演算の意味、診断、Primer IR、各出力経路の生成物、VM実行 |
| Whitebase側の実験処理 | 外部ツールとターゲットの選択、ビルド、別プロセスでの実行、条件の記録、比較・計測 |
| 利用側の実行環境 | 実行許可、未信頼コードの隔離、資源制限 |

Primerの公開CLIと成果物を接点にし、内部のRust IRへ依存しません。生成プログラムを既存の`ComputeBackend`へ無理に当てはめたり、`BackendKind`へ項目を追加するだけで接続済みと扱ったりしません。必要な実験処理の配置は、その入出力が具体化してから決めます。Coreへ外部プロセスの起動責務を持ち込まない方針です。

Primerの言語規則を検証する回帰テストと、利用者が条件を選ぶ実験は目的が異なります。Primer側に生成コードの実行テストがあっても、Whitebaseの役割との重複とは限りません。

### 対応状況と比較結果

連携時は「生成できる」「ビルドできる」「実行できる」「実行してよい」を分けます。ツールが見つかったことだけで成功を保証せず、事前確認と実際の処理結果を別々に記録します。未対応、ツール不足、権限による拒否、生成・ビルド・実行の失敗、時間切れを区別し、実行できなかった経路も理由付きで残します。既存の演算用capabilityや実行状態を、この詳細情報の代わりにはしません。

Whitebase自身にLinux向けAssembly実装があっても、Primerの`emit-asm`はWindows x86-64向けです。WSLから生成してもターゲットは変わりません。LinuxでC・LLVMの生成物を実行できるかは外部ツールとライブラリの条件で別途確認します。Linux向けDirect ASMの追加とは別の作業です。

まず既存の`primer run`と`emit-*`で、終了状態と標準出力を比較できます。ただし、表示の一致は数値の内部ビットすべての一致ではなく、非ゼロ終了同士でも同じ理由の停止とは限りません。VMのみを正解とせず既知の期待値でも確認し、改行の正規化や許容誤差などの比較条件を明記します。プロセス起動・表示を含む時間を、既存Runnerの演算だけの時間と同じ尺度で比較しません。

### 記録と実行許可

実験では入力と生成物の識別情報、Primerのビルド識別情報、出力経路、ターゲット、外部ツールの版とオプション、段階ごとの結果、比較条件を保持します。`primer --version`でパッケージの版を取得できます。同じ版番号の開発ビルドを区別する場合は、検証したコミットや実行ファイルのハッシュも記録します。対応出力先の機械可読な一覧は未実装であり、当面は検証した版のCLIと文書をもとに接続できます。

観測データや生成物にコマンドや権限を持たせません。利用者が選んだ信頼済みのツールを、引数を分離して呼び出す設計とし、未信頼コードには適切な隔離が必要です。時間制限や別プロセス化だけを安全な隔離とはみなしません。出力サイズを制限し、ソース本文・パス・環境変数全体を無条件に記録しないようにします。

この整理によってHTTP・Tauri・ブラウザに外部コマンドの実行権限は追加しません。新しい公開スキーマやビット単位の観測形式は、実際の利用箇所が必要とした時点で設計します。stringなどPrimerの言語機能を、その完成まで待たせる必要はありません。

Primer側の出力先と責務は、[出力経路とターゲット](https://github.com/Hokutaka/Primer/blob/master/docs/design/targets.ja.md)と[コンパイラ設計](https://github.com/Hokutaka/Primer/blob/master/docs/design/architecture.ja.md)を参照してください。

## 動作確認方法

Whitebaseは、以下の3つの方法で動作を確認できます。

| 実行方法 | Server | Whitebaseの呼び出し経路 |
| --- | ---: | --- |
| Tauriアプリ | 不要 | Tauri → Tauri Commands → Interface → Runner / Core |
| GitHub Pages | 任意 | Browser → WASM → Interface → Runner / Core、またはloopback接続が利用可能な場合は Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |
| ローカルWebアプリ | 任意 | Browser → WASM → Interface → Runner / Core、または Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |

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

計算処理はTauri Commandsから`whitebase-interface`を経由し、
処理内容に応じてRunnerまたはWhitebase Coreを利用して、
利用可能なNative backendへルーティングされます。

```text
Tauri WebView
→ Tauri IPC / invoke()
→ Tauri Host (src-tauri)
→ whitebase-tauri-api
→ whitebase-interface
→ Runner / Whitebase Core
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
→ whitebase-http-api
→ whitebase-interface
→ Runner / Whitebase Core
→ Native backend
```

Whitebase Serverが利用できない、または接続できない場合は、
WebAssemblyを使用します。

```text
Browser
→ WebAssembly
→ whitebase-interface
→ Runner / Whitebase Core
→ Rust backend
```

WebAssembly環境では、Rust ScalarおよびRust SIMDバックエンドを利用できます。

現在のWhitebase WebAssembly artifactはWebAssembly SIMD128対応runtimeを
実行要件とします。

Rust SIMDバックエンドは、wasm32環境ではWebAssembly SIMD128を使用します。
SIMD128非対応runtime向けのbaseline artifactは現在提供していません。

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

各Application Interfaceは`whitebase-interface`を共通のApplication Boundaryとして利用します。

`whitebase-interface`はtransportに依存せず、
Pure ComputeではWhitebase Coreを直接利用し、
比較・計測・観測などのApplied ComputeではRunnerを利用します。

HTTPでは`whitebase-http-api`がHTTP / JSON / Axum固有処理を担当し、
`whitebase-server`はそのInterface Adapterを起動するHostとして扱います。

Tauri CommandsとWebAssemblyもWhitebase内部のCore / Runnerを直接公開せず、
それぞれの実行環境に合わせて`whitebase-interface`を利用します。

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

`whitebase-http-api`はローカルHTTP / JSON向けのInterface Adapterを提供します。

`whitebase-server`はそのAdapterを起動するHostとして扱います。

エンドポイント、Request / Responseの詳細については、
[HTTP API](api/HTTP-API.ja.md)を参照してください。


