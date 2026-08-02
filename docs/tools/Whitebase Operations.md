### Windows用開発コマンド

Windows環境では、開発・テスト・ビルド用の操作を
`scripts/ops.bat`から実行できます。

リポジトリのルートディレクトリで、以下の形式で実行します。

```powershell
.\scripts\ops.bat <command>
```

| コマンド | 説明 | Description |
| --- | --- | --- |
| `setup` | npm依存関係、WebAssemblyコンパイルターゲット、wasm-packをセットアップします。 | Set up npm dependencies, the WebAssembly compilation target, and wasm-pack. |
| `test` | Rustワークスペース全体のテストを実行します。 | Run Rust workspace tests. |
| `fmt` | Rustソースコードをフォーマットします。 | Format Rust source code. |
| `lint` | Clippyによる静的解析を実行します。 | Run static analysis with Clippy. |
| `check` | フォーマット確認、静的解析、テスト、Wasm、フロントエンド、Rust C ABI、C++バックエンド、RustからC++へのAdapter、Assembly経路を総合検査します。 | Run formatting, linting, tests, WebAssembly, frontend, Rust C ABI, C++ backend, Rust-to-C++ adapter, and Assembly integration checks. |
| `wasm-check` | WebAssemblyクレートのコンパイルを確認します。 | Check that the WebAssembly crate compiles. |
| `cpp-check` | C++からRust C ABIを呼び出せることを確認します。 | Check the C++ to Rust C ABI connection. |
| `cpp-backend-check` | C++計算バックエンドのScalarおよびAVX配列演算を確認します。 | Check the scalar and AVX array operations of the C++ computation backend. |
| `cpp-adapter-check` | RustからC++計算バックエンドを呼び出せることを確認します。 | Check the Rust to C++ computation backend adapter. |
| `asm-check` | C++からAssembly関数を呼び出せることを確認します。 | Check the C++ to Assembly connection. |
| `tree` | リポジトリツリーを表示し、ドキュメントを更新します。 | Display the repository tree and update its documentation. |
| `diagram` | Mermaidの構成図をSVGへ変換して更新します。 | Generate and update SVG diagrams from Mermaid sources. |
| `dev` | Tauriアプリケーションを開発モードで起動します。 | Start the Tauri application in development mode. |
| `web-dev` | WebAssemblyを開発用にビルドし、Web開発サーバーを起動します。 | Build WebAssembly for development and start the Web development server. |
| `wasm-build` | WebAssemblyのブラウザ用成果物を生成します。 | Build browser-compatible WebAssembly artifacts. |
| `c-api-build` | Rust C APIのDLLとインポートライブラリをビルドします。 | Build the Rust C API DLL and import library. |
| `cpp-build` | C++スモークテストクライアントをビルドします。 | Build the C++ smoke test client. |
| `cpp-backend-build` | C++計算バックエンドの静的ライブラリをビルドします。 | Build the C++ computation backend static library. |
| `asm-build` | Assemblyライブラリとスモークテストクライアントをビルドします。 | Build the Assembly library and smoke test client. |
| `web-build` | フロントエンドをビルドします。 | Build the frontend. |
| `tauri-build` | Tauriデスクトップアプリケーションをビルドします。 | Build the Tauri desktop application. |
| `clean` | 生成されたビルド成果物を削除します。 | Remove generated build artifacts. |

### Linux Native用開発コマンド

Linux x86_64環境では、GCCおよびNASMで実装された
Linux Nativeバックエンドのビルド・テスト操作を
`scripts/linux-native.sh`から実行できます。

リポジトリのルートディレクトリで、以下の形式で実行します。

```bash
./scripts/linux-native.sh <command>
````

| コマンド | 説明 | Description |
| --- | --- | --- |
| `build` | Linux NativeバックエンドをDebug構成で設定し、ビルドします。| Configure and build the Linux Native backends in the Debug configuration. |
| `check` | Linux NativeバックエンドをDebug構成でビルドし、CTestおよびNativeスモークテストを実行します。 | Build the Linux Native backends in the Debug configuration and run CTest and the native smoke test. |
| `release`  | Linux NativeバックエンドをRelease構成でビルドし、CTestおよびNativeスモークテストを実行します。 | Build the Linux Native backends in the Release configuration and run CTest and the native smoke test. |
| `clean` | `native/Whitebase.Linux/build`以下のLinux Nativeビルド成果物を削除します。 | Remove the Linux Native build outputs under `native/Whitebase.Linux/build`. |
| `help`, `-h`, `--help` | 使用方法と利用可能なコマンドを表示します。 | Display usage information and the available commands. |

