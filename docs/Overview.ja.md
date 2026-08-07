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
- Http + JSON
- WebAssembly
- デスクトップUIとブラウザUI

## 動作確認方法

Whitebaseは、以下の3つの方法で動作を確認できます。

| 実行方法 | Server | Coreの呼び出し経路 |
|---|---:|---|
| Tauriアプリ | 不要 | Tauri → Rust → Whitebase Core |
| GitHub Pages | 必要 | Browser → HTTP → Whitebase Server → Core |
| ローカルWebアプリ | 必要 | Browser → HTTP → Whitebase Server → Core |

### Tauriアプリ

Whitebase Appを起動します。

TauriアプリはWhitebase Serverを必要とせず、アプリ内のRustコードから
Whitebase Coreを直接呼び出します。

### GitHub Pages + Whitebase Server

Whitebase Serverを起動した状態で、GitHub Pages上のWhitebase Appを開きます。

GitHub Pagesは静的なフロントエンドのみを提供し、計算処理は
Whitebase ServerのHTTP APIを通して実行されます。

### ローカルWebアプリ + Whitebase Server

Whitebase Serverと、ローカルのフロントエンド開発サーバーを起動します。

```powershell
cargo run -p whitebase-server
npm --prefix apps/whitebase-app run dev
```

表示されたローカルURLをブラウザで開きます。

### ビルド構成による違い

開発環境とRelease環境では、次の項目に差が出る可能性があります。

- Rustの最適化レベル
- C++およびAssemblyのDebug / Releaseライブラリ
- SIMD処理の性能
- ベンチマーク結果
- 実行ファイルやバンドルの出力先

特にベンチマーク結果は、DebugビルドとReleaseビルドで大きく異なります。
計算結果の正当性は、ビルド構成にかかわらず各バックエンド間で
検証されることを前提とします。

## アーキテクチャ
TauriではIPC、通常のブラウザではHTTP APIを通して、  
同じRunnerとCoreを利用します。

![モジュール構成図](/docs/diagrams/structure/architecture.svg)

## 構成図

### モジュール構成図

矢印は、モジュール間の依存・利用関係または計算結果の受け渡し方向を示します。  
破線のモジュールは、将来的な追加を予定しています。

![モジュール構成図](/docs/diagrams/structure//module.svg)

### 利用構成図

ライブラリの利用方法の想定です。  
`whitebase-core`を計算処理の中心とし、利用する環境に応じた境界を通して呼び出します。  

![利用構成図](/docs/diagrams/structure/usage.svg)

## 実装済みのベンチマークについて
```text
入力生成→参照BackEndをウォームアップ→複数回実行して時間計測→最小・最大。平均・合計時間の集計→参照結果との誤差を比較→Tauri・ブラウザへレポートを返す
```

※ベンチマーク結果はビルド構成、CPU、キャッシュ、メモリ帯域、OSのスケジューリングなどに影響されます。性能比較ではDebugではなくRelease構成を推奨します。

### Core API

See [Core API](api/Core-API.ja.md) for the Rust API, operations, backends, and errors.


### HTTP API

`Whitebase Server`は`local HTTP/JSON API`を公開しています

See [HTTP API](api/HTTP-API.ja.md) for endpoint and request/response details.


