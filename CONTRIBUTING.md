# Contributing to Whitebase

Whitebaseへの貢献に興味を持っていただき、ありがとうございます。

このドキュメントでは、開発環境の準備、変更の確認、IssueやPull Requestの作成方法を説明します。

## Development Status

Whitebaseは現在開発中です。

仕様やディレクトリ構成、公開APIが変更される可能性があります。大きな変更を始める前に、Issueで方針を確認してください。

## Requirements

主な開発環境は以下です。

- Rust toolchain
- Cargo
- Git
- Node.js
- npm
- WindowsまたはLinux

必要なRustバージョンは、リポジトリ内の`rust-toolchain.toml`で管理されています。

## Repository Setup

リポジトリを取得します。

```shell
git clone https://github.com/Hokutaka/Whitebase.git
cd Whitebase
```

### Windows

Windowsでは、リポジトリ直下の運用スクリプトを使用できます。

```cmd
scripts\ops.bat setup
scripts\ops.bat check
```

### Linux

Linuxでは、必要なネイティブライブラリをインストールしたうえでCargoコマンドを実行してください。

```shell
cargo check --workspace
cargo test --workspace
```

## Whitebase Control Center

Debug版を起動します。

```shell
cargo run -p whitebase-control-center
```

Release版をビルドします。

```shell
cargo build --release -p whitebase-control-center
```

生成される実行ファイルは以下です。

### Windows

```text
target/release/whitebase-control-center.exe
```

### Linux

```text
target/release/whitebase-control-center
```

## Before Submitting Changes

Pull Requestを作成する前に、可能な範囲で以下を実行してください。

```shell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Control Centerを変更した場合は、Debug版とRelease版の両方を確認してください。

```shell
cargo run -p whitebase-control-center
cargo build --release -p whitebase-control-center
```

OS固有の変更を行った場合は、対象OSでの動作確認結果をPull Requestへ記載してください。

## Commit Guidelines

コミットは、ひとつの目的ごとに小さくまとめてください。

コミットメッセージは、変更内容が分かる簡潔な文章にします。

例：

```text
Control CenterにReleaseビルドタスクを追加
CIでLinuxワークスペースを検証
サーバー停止時のプロセス処理を修正
```

生成物、秘密鍵、アクセストークン、個人情報はコミットしないでください。

## Reporting Bugs

不具合を報告する場合は、GitHub IssuesのBug reportフォームを使用してください。

可能な範囲で以下を含めてください。

- 不具合の概要
- 再現手順
- 期待する動作
- 実際の動作
- OSとバージョン
- Whitebaseのバージョンまたはコミット
- 関連するログ

セキュリティ上の問題は公開Issueへ投稿せず、`SECURITY.md`の手順に従ってください。

## Feature Requests

新機能や改善案は、GitHub IssuesのFeature requestフォームを使用してください。

実装方法だけでなく、解決したい問題や想定する利用方法も記載してください。

## Pull Requests

Pull Requestでは、テンプレートに沿って以下を記載してください。

- 変更の概要
- 主な変更内容
- 動作確認結果
- 確認したOS
- 関連Issue
- スクリーンショットやログ
- 今後の課題

CIが失敗している場合は、原因と対応方針を記載してください。

## Code Review

レビューでは、主に以下を確認します。

- 変更の目的が明確か
- 既存機能を壊していないか
- WindowsとLinuxへの影響
- テストや確認手順が十分か
- 不要な依存関係が増えていないか
- セキュリティ上の問題がないか

## License

コードを投稿することで、その変更をWhitebaseのライセンス条件のもとで提供することに同意したものとみなします。
