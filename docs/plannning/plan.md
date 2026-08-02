# 方向性とか

## 予定

1. Tauri Visualizationへの接続
2. Python Tools置き場の作成

やれればどっかにデモ。

## 拡張について

1. 演算経路  
四則演算。
行列積、ソート、文字列処理など特性の違う演算を追加する。


2. 比較  
Debug/Release、Scalar/SIMD/AVX512、複数コンパイラ（MSVC/Clang/GCC）を横断した同一演算の結果・速度マトリクスを自動生成

3. 可視化・レポートの強化  
Tauri/Http APIにJSON/CSV出力→
python(pandas/matplotlib)で集計分析を行う

4. 新しい実行環境の追加  
VMを用意。Linux環境でも動作するか確認したい。  
対比軸としては、GPU(wgpu/CUDA)やWASI(ブラウザ外Wasm実行)等、新しい呼び出し経路を作成する。


5. 表現　
```text　
0.1 + 0.2                    表現誤差　　
(1e16 + 1.0) - 1e16          情報落ち　　
(a + b) + c と a + (b + c)   非結合性　　
1.0 / 0.0                    無限大　　
0.0 / 0.0                    NaN　　
-0.0                         符号付きゼロ　　
極小値                       アンダーフロー　　
mul_add と a * b + c         FMA差　　
```

## Control Center

ないもの

- Doctor
- Tauri／Frontend開発サーバー起動
- 子プロセスツリーの安全な停止
- 無効ボタンの理由表示
- 増えたTask定義の整理
- READMEとスクリーンショット更新
- Linux実機での確認