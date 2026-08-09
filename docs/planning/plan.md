# Plan

今後の方向性と、実装予定・拡張候補をまとめる。

Whitebaseでは、一度に多くの機能を追加するのではなく、
現在の構成を整理しながら、Operation・Backend・実行環境を段階的に拡張する。

## 直近の予定

1. Layer構成の整理
2. Tauri Visualizationへの接続
3. Python Tools置き場の作成

## 1. Layer構成の整理

現在のWhitebaseの責務と接続関係を整理する。

- Layer Overviewの作成
- Backend Contractの整理
- Backend Implementationの整理
- Backend Integrationの整理
- Core / Pure Computeの整理
- Runner / Applied Computeの整理
- HTTP / Tauri / WASM Interfaceの整理
- C APIの位置づけ整理
- Operations Planeの整理
- READMEからLayer Overviewへの導線追加

実際の実装と設計意図がずれない状態を先に作る。

## 2. Visualization

現在取得できる実行結果・比較結果を、
TauriおよびBrowser UIから確認できるようにする。

- Backendごとの実行結果
- Benchmark結果
- 計測時間
- 結果比較
- 数値表現の観察結果
- 使用された実行経路

## 3. Python Tools

Whitebaseから出力した結果を、
外部から集計・可視化するためのToolsを用意する。

- JSON出力
- CSV出力
- pandasによる集計・比較
- matplotlibによるグラフ生成
- 実行環境ごとの結果レポート

## 4. Operationの拡張

現在のAdd中心の構成を段階的に拡張する。

最初に四則演算を対象とする。

- Add
- Sub
- Mul
- Div

以下の形について整理する。

- scalar + scalar
- array + array
- array + scalar
- f32
- f64

一度にすべて実装するのではなく、
Operationを1つずつ追加しながら既存のContract / Core / Runner / Backend構成を確認する。

## 5. 数値表現の観察

IEEE 754に由来する代表的な挙動を、
実装・Backend・実行方式ごとに観察・比較できるようにする。

```text
0.1 + 0.2                    表現誤差
(1e16 + 1.0) - 1e16          情報落ち
近い値同士の減算             桁落ち
(a + b) + c と a + (b + c)   非結合性
1.0 / 0.0                    Infinity
0.0 / 0.0                    NaN
-0.0                         符号付きゼロ
極小値                       Underflow
Subnormal値　　　　　　　　　Subnormal
最大値付近の演算             Overflow
mul_add と a * b + c         FMAによる差
```

将来的には以下も比較する。

- Scalar / SIMDによる演算順序の差
- Reduction順序による結果の差
- Debug / Releaseによる差
- Compiler / optimizationによる差

## 6. 比較軸の拡張

同一Operationについて、比較可能な条件を増やす。

- Debug / Release
- Scalar / SIMD
- MSVC / GCC
- Windows / Linux

既存環境で比較軸を整理してから、
新しい命令セットやToolchainを検討する。

## 7. ワークロードの追加

四則演算の構成が安定した後、
性質の異なるOperationを少数追加する。

候補:

- 行列積
- ソート
- 文字列処理

目的は機能数を増やすことではなく、
現在のWhitebaseの構成が異なる処理にも適用できるか確認すること。

## 8. Control Center

現在不足しているもの:

- Doctor機能
- TauriおよびFrontend開発サーバーの起動
- 子プロセスツリーの安全な停止
- 無効化されたボタンの理由表示
- Task定義の整理
- READMEとスクリーンショットの更新
- Linux実機での動作確認
