### Whitebase Control Center

![Whitebase Control Center](../images/whitebase-control-center.png)

`Whitebase Control Center`は、開発・検証用のCargoタスクをGUIから実行するための、Windows / Linux対応Rustネイティブアプリケーションです。

ワークスペースのチェック、フォーマット確認、Clippy、テスト、ビルド、Whitebase Serverの起動・停止などを実行できます。
標準出力、実行状態、処理時間は画面上で確認できます。

Debug版を起動します。

```powershell
cargo run -p whitebase-control-center
```

Release版をビルドします。

```powershell
cargo build --release -p whitebase-control-center
```

生成される実行ファイルは以下です。

```text
Windows: target/release/whitebase-control-center.exe
Linux:   target/release/whitebase-control-center
```

Windows専用の`Whitebase Control Panel`とは別に、OSをまたいだ共通の開発操作を提供します。
内部では`scripts/ops.bat`を呼び出しているため、コマンドラインと同じ処理を実行できます。  
標準出力、終了コード、処理時間は画面上で確認できます。

`scripts/ops.bat`のコマンド及び、`Whitebase Control Panel`については以下を参考にしてください。
- [Whitebase Operations](../tools/Whitebase%20Operations.md)
- [Whitebase Control Panel](../tools/Whitebase%20Control%20Panel.md)

