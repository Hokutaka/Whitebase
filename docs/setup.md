# セットアップ

各種設定です。

## Rust/Tauri

### バージョンの確認

```powershell
rustc --version
cargo --version
rustup show active-toolchain

node --version
npm --version
```

現在の開発環境は以下です。
- rustc 1.94.0 (4a4ef493e 2026-03-02)
- cargo 1.94.0 (85eff7c80 2026-01-15)
- stable-x86_64-pc-windows-msvc (default)
- v20.12.2
- 10.5.0


### Whitebase-coreの作成

```cargo
cargo new --lib crates/whitebase-core --vcs none
```

## Tauriアプリの作成

- 操作
```Powershell
New-Item -ItemType Directory -Force apps
cd apps
npm create tauri-app@latest
```

- 設定内容
```
✔ Project name · whitebase-app
✔ Package name · whitebase-app
✔ Identifier · com.hokutaka.Whitebase
✔ Choose which language to use for your frontend · TypeScript / JavaScript - (pnpm, yarn, npm, deno, bun)
✔ Choose your package manager · npm
✔ Choose your UI template · Vanilla
✔ Choose your UI flavor · TypeScript
```

## npm install

`/app/whitebase-app`で以下の操作を行い、起動を確認。

```
npm install
npm run tauri dev
```

## ライブラリのテスト

```powershell
cargo test -p whitebase-core
```

以下が表示されれば成功。

```powershell
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## ワークスペース全体のテスト

```powershell
cargo test --workspace
```

## mermaid

ルートで以下をインストール。

```npm
npm install --save-dev @mermaid-js/mermaid-cli
```

## Operationsツール準備

`scripts/ops.bat`を叩くための管理ツール。

- Visual Studio 2026
- C# / .NET 10.0
- WPF(Windows Presentation Foundation)

### 手順

1. Visual Studioで「新しいプロジェクトの作成」
2. テンプレートからWPFアプリケーションを選択  
   ※WPFアプリケーション（.NET Framework）ではなく、.NET版
3. プロジェクト名を入力  
`Whitebase.ControlPanel`
4. 場所を指定  
`Whitebase\tools\whitebase-control-panel`
5. ソリューション名も指定  
`Whitebase.ControlPanel`
6. 「ソリューションとプロジェクトを同じディレクトリに配置する」はオフ  
以下の構成になる  
    ```text
    tools/
    └─ whitebase-control-panel/
    ├─ Whitebase.ControlPanel.sln
    └─ Whitebase.ControlPanel/
        ├─ Whitebase.ControlPanel.csproj
        ├─ App.xaml
        ├─ App.xaml.cs
        ├─ MainWindow.xaml
        └─ MainWindow.xaml.cs
    ```

## Wasmの用意

1. ライブラリクレート `crates/whitebase-wasm` を追加
    ```powershell
    cargo new --lib crates/whitebase-wasm --vcs none
    ```


    crates/whitebase-core/Cargo.toml
    ```toml
    [package]
    name = "whitebase-core"
    version = "0.1.0"
    edition = "2024"

    [dependencies]
    ```

2. ワークスペースに追加する
    ```toml
    [workspace]
    resolver = "3"
    members = [
        "crates/whitebase-core",
        "crates/whitebase-wasm", // ←wasmを追加
        "apps/whitebase-app/src-tauri",
    ]
    ```

3. `whitebase-wasm`の設定

    ```toml
    [package]
    name = "whitebase-wasm"
    version = "0.1.0"
    edition = "2024"
    publish = false

    [lib]
    crate-type = ["cdylib", "rlib"]

    [dependencies]
    whitebase-core = { path = "../whitebase-core" }
    wasm-bindgen = "0.2"
    ```

4. 役割はこうなります

    ```text
    whitebase-wasm
    ├── whitebase-coreを呼ぶ
    └── JavaScriptへ公開する
    ```

5. 通常環境での確認とWasmコンパイル確認  
    5.1. ワークスペースとして認識されているか確認。  
    ```powershell
    cargo metadata --no-deps
    ```  

    5.2. テストして確認
    ```powershell
    .\scripts\ops.bat test
    ```  
  
    5.3. Wasm向けコンパイルを確認
    ```powershell
    rustup target add wasm32-unknown-unknown
    ```  
  
    5.4. Wasm境界だけを指定して確認
    ```powershell
    cargo check -p whitebase-wasm --target wasm32-unknown-unknown
    ```  

6. `wasm-pack`を入れる
    ```powershell
    cargo install wasm-pack
    ```

7. 依存関係の確認  
    `winget`でnode.js更新

    ```powershell
    winget upgrade --id OpenJS.NodeJS.20 --exact
    ```

## RustコアをABIで公開してC++から呼ぶ準備
やることはこれ
```text
C++アプリ
   ↓ C ABI
whitebase-c-api
   ↓ Rust
whitebase-core
```

1. C API Cratesを作成
```
cargo new --lib crates/whitebase-c-api
```

2. ルートの`Cargo.tml`に追加

```toml
    "crates/whitebase-c-api",
```

3. `whitebase-c-api/Cargo.toml`をこうする

```toml
[package]
name = "whitebase-c-api"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
crate-type = ["cdylib"]

[dependencies]
whitebase-core = { path = "../whitebase-core" }
```

