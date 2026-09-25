[![CI](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml)
[![CodeQL](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql)

**日本語** | [English](README.en.md)

学習・実験用リポジトリです。

# Whitebase
Whitebaseは、基盤となる実装を作りながら試すための実験場所です。

Whitebase Coreを中心としたモノレポとして構成されています。

このリポジトリでは、利用者向けの機能を実行結果・グラフ・その他の可視化に絞っています。
具体的なアプリケーションや用途固有の実装は、別のプロジェクトとして扱います。

現在は、Rust、C++、Assembly、Ceruneを使った`f32` / `f64`の小さな計算処理を中心に、Scalar、SIMD、VMなど異なる実行経路を実装・比較しています。

実験用プロジェクトのため、利用は自己責任でお願いします。

内容については、以下を確認してください。

## Documentation

- [詳しい説明](/docs/Overview.ja.md)
- [プロジェクト構成](docs/project-tree.md)
- [レイヤーの説明](docs/Layer/Layer-Overview.md)

### Tools

| 名称 | 実装内容 |
| --- | --- |
| [Whitebase Operations](docs/tools/Whitebase%20Operations.md) | Windows Batch / Linux Native Shell |
| [Whitebase Control Center](docs/tools/Whitebase%20Control%20Center.md) | Rust + egui / Windows and Linux |
| [Whitebase Control Panel](docs/tools/Whitebase%20Control%20Panel.md) | C# + WPF / Windows |

## License

MIT LICENSE