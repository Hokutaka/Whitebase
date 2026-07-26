# Contributing to Whitebase

Whitebaseへの貢献ありがとうございます。

## Development

必要なRustバージョンは`rust-toolchain.toml`で管理しています。

### Windows

```cmd
scripts\ops.bat setup
scripts\ops.bat check
```

### Rust workspace

```shell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Control Center

```shell
cargo run -p whitebase-control-center
cargo build --release -p whitebase-control-center
```

## Pull Requests

変更内容と動作確認結果を記載してください。

OS固有の変更では、確認したOSも記載してください。

コミットには秘密鍵、アクセストークン、個人情報、不要な生成物を含めないでください。

セキュリティ上の問題は、公開Issueではなく`SECURITY.md`の手順で報告してください。
