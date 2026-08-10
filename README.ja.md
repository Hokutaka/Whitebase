[![CI](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/main.yml)
[![CodeQL](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql/badge.svg?branch=main)](https://github.com/Hokutaka/Whitebase/actions/workflows/github-code-scanning/codeql)

[English](README.md) | **日本語**

学習・実験用リポジトリです。

# Whitebase
Whitebaseは、さまざまな基礎実装を作り、試すための場所です。

Coreを中心としたモノレポ構成を採用しています。
利用側の機能は、実行結果や図表の表示にとどめます。
具体的な応用や用途に特化した実装は、別のプロジェクトへ分ける方針です。

今はRust・C++・Assemblyでf32, f64の配列演算をしています。
試すなら自己責任でお願いします。

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