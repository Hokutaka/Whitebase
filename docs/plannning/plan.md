# 方向性とか

## 予定

1. Tauri Visualizationへの接続
2. Python Tools置き場の作成
3. `0.1+0.2`追加をして、機能追加フローを作成

やれればどっかにデモ。

## 拡張について

1. 演算経路  
行列積、ソート、文字列処理など特性の違う演算を追加する。

2. 比較  
Debug/Release、Scalar/SIMD/AVX512、複数コンパイラ（MSVC/Clang/GCC）を横断した同一演算の結果・速度マトリクスを自動生成

3. 可視化・レポートの強化  
Tauri/Http APIにJSON/CSV出力→
python(pandas/matplotlib)で集計分析を行う

4. 新しい実行環境の追加  
VMを用意。Linux環境でも動作するか確認したい。  
対比軸としては、GPU(wgpu/CUDA)やWASI(ブラウザ外Wasm実行)等、新しい呼び出し経路を作成する。
