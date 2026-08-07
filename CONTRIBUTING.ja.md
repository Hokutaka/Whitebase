# Whitebaseへのコントリビューション

English: [CONTRIBUTING.md](./CONTRIBUTING.md)

Whitebaseへの貢献に興味を持っていただきありがとうございます。

## 開発モデル

Whitebaseでは、`dev`を統合開発ブランチ、`main`を安定ブランチとして運用します。

```text
作業ブランチ
    ↓ Pull Request
dev
    ↓ Pull Request
main
```

機能追加、修正、ドキュメント、CIなどの作業ブランチは、最新の`dev`から作成してください。

`main`または`dev`上で直接開発しないでください。

## 作業の開始

作業ブランチを作る前に`dev`を最新化します。

```shell
git switch dev
git pull --ff-only origin dev
git switch -c feature/example
```

ブランチ名は変更内容が分かるものを使用してください。例:

- `feature/...` — 機能追加
- `fix/...` — バグ修正
- `docs/...` — ドキュメント更新
- `ci/...` — CI / Workflow更新

## 開発環境

使用するRustツールチェーンは`rust-toolchain.toml`で管理しています。

### Windows

リポジトリのルートディレクトリから、Windows共通のセットアップと統合チェックを実行できます。

```powershell
.\scripts\ops.bat setup
.\scripts\ops.bat check
```

Windows GNU Nativeを開発する場合は、MSYS2 UCRT64環境とMinGW-w64 GCC、CMake、Ninja、NASMも必要です。

### Linux x86_64

LinuxのGCC/NASM Nativeバックエンドは以下で確認できます。

```bash
./scripts/linux-native.sh check
```

各コマンドの詳細は[Whitebase Operations](./docs/tools/Whitebase%20Operations.md)を参照してください。

## Rustワークスペースの確認

Pull Requestを作成する前に、変更内容に対応する検査を実行してください。基本的なRustチェックは以下です。

```shell
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

OS固有の変更では、可能な限り対象OS上でも動作確認してください。

## Control Center

Whitebase Control Centerから、現在のプラットフォームで利用可能な検査、ビルド、Release操作をまとめて実行できます。

```shell
cargo run -p whitebase-control-center
```

主な一括操作:

- `Check All` — 現在のプラットフォームで利用可能な検査を順番に実行します。
- `Build All` — 現在のプラットフォーム向け開発用成果物をビルドします。
- `Release All` — 現在のプラットフォーム向けRelease成果物をビルドします。

## コミット前の確認

コミット前に作業ツリーを確認してください。

```shell
git status
git diff --check
git diff --stat
```

秘密鍵、アクセストークン、個人情報、一時的なパッチやZIP、不要な生成物をコミットしないでください。

## Pull Request

通常の作業ブランチは`dev`向けにPull Requestを作成します。

Pull Requestテンプレートに変更内容と動作確認結果を記載してください。OS固有の変更では、確認したOSを明記してください。

リポジトリのRulesetで要求されているCI、Dependency Review、Code Scanningを通し、レビュー上の会話を解決してからマージしてください。

作業ブランチのマージにはSquash mergeを推奨します。

## `dev`から`main`への昇格

複数の変更を`dev`で統合・確認した後、`dev`から`main`へのPull Requestを作成します。

`main`は安定ブランチとして扱います。Release成果物やGitHub Pagesは、リポジトリのCI設定に従って`main`から生成・更新されます。

## セキュリティ

セキュリティ上の問題を公開Issueで報告しないでください。

[SECURITY.md](./SECURITY.md)に記載された非公開の報告手順を使用してください。
