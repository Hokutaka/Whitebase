@echo off
chcp 65001 >nul
setlocal

rem scriptsディレクトリの1階層上をリポジトリルートとする
set "WHITEBASE_ROOT=%~dp0.."
for %%I in ("%WHITEBASE_ROOT%") do set "WHITEBASE_ROOT=%%~fI"

pushd "%WHITEBASE_ROOT%" || exit /b 1

set "APP_DIR=apps\whitebase-app"
set "WASM_PACKAGE=whitebase-wasm"
set "WASM_TARGET=wasm32-unknown-unknown"
set "WASM_DIR=crates\whitebase-wasm"
set "WASM_OUT_DIR=%APP_DIR%\src\wasm"

set "CPP_SOLUTION=%WHITEBASE_ROOT%\native\Whitebase.Cpp\Whitebase.Cpp.slnx"
set "CPP_EXE=%WHITEBASE_ROOT%\native\Whitebase.Cpp\x64\Debug\Whitebase.CppClient.exe"

set "CPP_SOLUTION_DIR=%WHITEBASE_ROOT%\native\Whitebase.Cpp"
set "ASM_PROJECT=%WHITEBASE_ROOT%\native\Whitebase.Cpp\Whitebase.AssemblyClient\Whitebase.AssemblyClient.vcxproj"
set "ASM_EXE=%WHITEBASE_ROOT%\native\Whitebase.Cpp\x64\Debug\Whitebase.AssemblyClient.exe"

set "CPP_BACKEND_PROJECT=%WHITEBASE_ROOT%\native\Whitebase.Cpp\Whitebase.CppBackend\Whitebase.CppBackend.vcxproj"
set "CPP_BACKEND_CLIENT_PROJECT=%WHITEBASE_ROOT%\native\Whitebase.Cpp\Whitebase.CppBackendClient\Whitebase.CppBackendClient.vcxproj"
set "CPP_BACKEND_EXE=%WHITEBASE_ROOT%\native\Whitebase.Cpp\x64\Debug\Whitebase.CppBackendClient.exe"
set "CPP_ADAPTER_PACKAGE=whitebase-cpp-adapter"

rem コマンド一覧
if "%~1"=="" goto help
if /i "%~1"=="help" goto help
if /i "%~1"=="-h" goto help
if /i "%~1"=="--help" goto help
if /i "%~1"=="setup" goto setup
if /i "%~1"=="test" goto test
if /i "%~1"=="fmt" goto fmt
if /i "%~1"=="lint" goto lint
if /i "%~1"=="check" goto check
if /i "%~1"=="wasm-check" goto wasm_check
if /i "%~1"=="cpp-check" goto run_cpp_check
if /i "%~1"=="cpp-backend-check" goto run_cpp_backend_check
if /i "%~1"=="cpp-adapter-check" goto run_cpp_adapter_check
if /i "%~1"=="asm-check" goto run_asm_check
if /i "%~1"=="wasm-build" goto wasm_build
if /i "%~1"=="c-api-build" goto run_c_api_build
if /i "%~1"=="cpp-build" goto run_cpp_build
if /i "%~1"=="cpp-backend-build" goto run_cpp_backend_build
if /i "%~1"=="asm-build" goto run_asm_build
if /i "%~1"=="tree" goto tree
if /i "%~1"=="diagram" goto diagram
if /i "%~1"=="dev" goto dev
if /i "%~1"=="web-dev" goto web_dev
if /i "%~1"=="web-build" goto web_build
if /i "%~1"=="tauri-build" goto tauri_build
if /i "%~1"=="clean" goto clean
goto unknown_command

:run_c_api_build
rem Rust C APIのビルド
call :c_api_build
if errorlevel 1 goto error
goto success

:run_cpp_build
rem C++クライアントのビルド
call :cpp_build
if errorlevel 1 goto error
goto success

:run_cpp_backend_build
rem C++計算バックエンドのビルド
call :cpp_backend_build
if errorlevel 1 goto error
goto success

:run_asm_build
rem Assemblyクライアントのビルド
call :asm_build
if errorlevel 1 goto error
goto success

:run_cpp_check
rem C++クライアントのチェック
call :cpp_check
if errorlevel 1 goto error
goto success

:run_cpp_backend_check
rem C++計算バックエンドのチェック
call :cpp_backend_check
if errorlevel 1 goto error
goto success

:run_cpp_adapter_check
rem RustからC++計算バックエンドへの接続確認
call :cpp_adapter_check
if errorlevel 1 goto error
goto success

:run_asm_check
rem Assemblyクライアントのチェック
call :asm_check
if errorlevel 1 goto error
goto success

:setup
rem セットアップ処理
echo [Whitebase] Installing frontend dependencies...
call npm --prefix "%APP_DIR%" install
if errorlevel 1 goto error

echo.
echo [Whitebase] Installing WebAssembly compilation target...
rustup target add %WASM_TARGET%
if errorlevel 1 goto error

echo.
where wasm-pack >nul 2>&1
if errorlevel 1 (
    echo [Whitebase] Installing wasm-pack...
    cargo install wasm-pack
    if errorlevel 1 goto error
) else (
    echo [Whitebase] wasm-pack is already installed.
)

echo.
echo [Whitebase] Setup completed.
goto success

:test
rem Rustワークスペースのテスト
call :prepare_native_dependencies
if errorlevel 1 goto error

echo.
echo [Whitebase] Running Rust workspace tests...
cargo test --workspace
if errorlevel 1 goto error
goto success

:fmt
rem Rustソースコードのフォーマット
echo [Whitebase] Formatting Rust sources...
cargo fmt --all
if errorlevel 1 goto error
goto success

:lint
rem Clippyによる静的解析
call :prepare_native_dependencies
if errorlevel 1 goto error

echo.
echo [Whitebase] Running Clippy...
cargo clippy --workspace --all-targets -- -D warnings
if errorlevel 1 goto error
goto success

echo.
echo [Whitebase] Running Clippy...
cargo clippy --workspace --all-targets -- -D warnings
if errorlevel 1 goto error
goto success

:check
rem 総合チェック
echo [Whitebase] Checking Rust formatting...
cargo fmt --all -- --check || goto error

call :prepare_native_dependencies
if errorlevel 1 goto error

echo.
echo [Whitebase] Running Rust static analysis...
cargo clippy --workspace --all-targets -- -D warnings || goto error

echo.
echo [Whitebase] Running Rust workspace tests...
cargo test --workspace || goto error

echo.
echo [Whitebase] Checking WebAssembly crate...
cargo check -p %WASM_PACKAGE% --target %WASM_TARGET% || goto error

echo.
echo [Whitebase] Building frontend...
call npm --prefix "%APP_DIR%" run build || goto error

call :cpp_check
if errorlevel 1 goto error

call :cpp_backend_check
if errorlevel 1 goto error

call :asm_check
if errorlevel 1 goto error

echo.
echo [Whitebase] All checks passed.
goto success

:wasm_check
rem WebAssemblyクレートのコンパイル確認
echo [Whitebase] Checking WebAssembly crate...
cargo check -p %WASM_PACKAGE% --target %WASM_TARGET%
if errorlevel 1 goto error
goto success

:cpp_check
rem C++からRust C ABIへの接続確認
echo.
echo [Whitebase] Checking C++ to Rust C ABI connection...

call :cpp_build
if errorlevel 1 exit /b 1

if not exist "%CPP_EXE%" (
    echo [Whitebase] ERROR: C++ smoke test executable was not found.
    echo [Whitebase] Expected: %CPP_EXE%
    exit /b 1
)

"%CPP_EXE%"

if errorlevel 1 (
    echo [Whitebase] ERROR: C++ smoke test failed.
    exit /b 1
)

echo [Whitebase] C++ smoke test passed.
exit /b 0

:cpp_backend_check
rem C++計算バックエンドの動作確認
echo.
echo [Whitebase] Checking C++ computation backend...

call :cpp_backend_client_build
if errorlevel 1 exit /b 1

if not exist "%CPP_BACKEND_EXE%" (
    echo [Whitebase] ERROR: C++ backend smoke test executable was not found.
    echo [Whitebase] Expected: %CPP_BACKEND_EXE%
    exit /b 1
)

"%CPP_BACKEND_EXE%"

if errorlevel 1 (
    echo [Whitebase] ERROR: C++ backend smoke test failed.
    exit /b 1
)

echo [Whitebase] C++ backend smoke test passed.
exit /b 0

:cpp_adapter_check
rem RustからC++計算バックエンドへの接続確認
echo.
echo [Whitebase] Checking Rust to C++ backend adapter...

call :cpp_backend_build
if errorlevel 1 exit /b 1

cargo test -p %CPP_ADAPTER_PACKAGE% -- --nocapture
if errorlevel 1 (
    echo [Whitebase] ERROR: C++ adapter smoke test failed.
    exit /b 1
)

echo [Whitebase] C++ adapter smoke test passed.
exit /b 0

:asm_check
rem C++からAssemblyへの接続確認
echo.
echo [Whitebase] Checking C++ to Assembly connection...

call :asm_build
if errorlevel 1 exit /b 1

if not exist "%ASM_EXE%" (
    echo [Whitebase] ERROR: Assembly smoke test executable was not found.
    echo [Whitebase] Expected: %ASM_EXE%
    exit /b 1
)

"%ASM_EXE%"

if errorlevel 1 (
    echo [Whitebase] ERROR: Assembly smoke test failed.
    exit /b 1
)

echo [Whitebase] Assembly smoke test passed.
exit /b 0

:wasm_build
rem WebAssemblyのブラウザ用成果物を生成
echo [Whitebase] Building WebAssembly package...
wasm-pack build "%WASM_DIR%" ^
    --target web ^
    --dev ^
    --out-dir "%CD%\%WASM_OUT_DIR%"
if errorlevel 1 goto error
goto success

:c_api_build
rem Rust C APIのビルド
echo.
echo [Whitebase] Building Rust C API...

cargo build -p whitebase-c-api
if errorlevel 1 (
    echo [Whitebase] ERROR: Rust C API build failed.
    exit /b 1
)

echo [Whitebase] Rust C API build completed.
exit /b 0

:cpp_build
rem C++クライアントのビルド
echo.
echo [Whitebase] Building C++ smoke test...

call :c_api_build
if errorlevel 1 exit /b 1

call :find_msbuild
if errorlevel 1 exit /b 1

if not exist "%CPP_SOLUTION%" (
    echo [Whitebase] ERROR: C++ solution was not found.
    echo [Whitebase] Expected: %CPP_SOLUTION%
    exit /b 1
)

"%MSBUILD%" "%CPP_SOLUTION%" ^
    /t:Build ^
    /m ^
    /p:Configuration=Debug ^
    /p:Platform=x64 ^
    /v:minimal

if errorlevel 1 (
    echo [Whitebase] ERROR: C++ build failed.
    exit /b 1
)

echo [Whitebase] C++ build completed.
exit /b 0

:cpp_backend_build
rem C++計算バックエンドのビルド
echo.
echo [Whitebase] Building C++ computation backend...

call :find_msbuild
if errorlevel 1 exit /b 1

if not exist "%CPP_BACKEND_PROJECT%" (
    echo [Whitebase] ERROR: C++ backend project was not found.
    echo [Whitebase] Expected: %CPP_BACKEND_PROJECT%
    exit /b 1
)

"%MSBUILD%" "%CPP_BACKEND_PROJECT%" ^
    /t:Build ^
    /m ^
    /p:Configuration=Debug ^
    /p:Platform=x64 ^
    /p:SolutionDir=%CPP_SOLUTION_DIR%\ ^
    /v:minimal

if errorlevel 1 (
    echo [Whitebase] ERROR: C++ backend build failed.
    exit /b 1
)

echo [Whitebase] C++ backend build completed.
exit /b 0

:cpp_backend_client_build
rem C++計算バックエンドのスモークテストクライアントをビルド
echo.
echo [Whitebase] Building C++ backend smoke test...

call :find_msbuild
if errorlevel 1 exit /b 1

if not exist "%CPP_BACKEND_CLIENT_PROJECT%" (
    echo [Whitebase] ERROR: C++ backend client project was not found.
    echo [Whitebase] Expected: %CPP_BACKEND_CLIENT_PROJECT%
    exit /b 1
)

"%MSBUILD%" "%CPP_BACKEND_CLIENT_PROJECT%" ^
    /t:Build ^
    /m ^
    /p:Configuration=Debug ^
    /p:Platform=x64 ^
    /p:SolutionDir=%CPP_SOLUTION_DIR%\ ^
    /v:minimal

if errorlevel 1 (
    echo [Whitebase] ERROR: C++ backend client build failed.
    exit /b 1
)

echo [Whitebase] C++ backend smoke test build completed.
exit /b 0

:asm_build
rem Assemblyクライアントのビルド
echo.
echo [Whitebase] Building Assembly smoke test...

call :find_msbuild
if errorlevel 1 exit /b 1

if not exist "%ASM_PROJECT%" (
    echo [Whitebase] ERROR: Assembly client project was not found.
    echo [Whitebase] Expected: %ASM_PROJECT%
    exit /b 1
)

"%MSBUILD%" "%ASM_PROJECT%" ^
    /t:Build ^
    /m ^
    /p:Configuration=Debug ^
    /p:Platform=x64 ^
    /p:SolutionDir=%CPP_SOLUTION_DIR%\ ^
    /v:minimal

if errorlevel 1 (
    echo [Whitebase] ERROR: Assembly build failed.
    exit /b 1
)

echo [Whitebase] Assembly build completed.
exit /b 0

:tree
rem Git管理対象からリポジトリツリーを生成
echo [Whitebase] Generating repository tree...
call "%~dp0export-tree.bat"
if errorlevel 1 goto error
goto success

:diagram
rem Mermaidの構成図を更新
echo [Whitebase] Updating diagrams...
call npm run diagram
if errorlevel 1 goto error
goto success

:dev
rem Tauriアプリケーションを開発モードで起動
call :prepare_native_dependencies
if errorlevel 1 goto error

echo [Whitebase] Starting Tauri development environment...
call npm --prefix "%APP_DIR%" run tauri -- dev
if errorlevel 1 goto error
goto success

:web_dev
rem WebAssemblyを開発用ビルドしてWeb開発サーバーを起動
echo [Whitebase] Building WebAssembly package for development...
wasm-pack build "%WASM_DIR%" ^
    --target web ^
    --dev ^
    --out-dir "%CD%\%WASM_OUT_DIR%"
if errorlevel 1 goto error

echo.
echo [Whitebase] Starting Web development environment...
call npm --prefix "%APP_DIR%" run dev
if errorlevel 1 goto error
goto success

:web_build
rem Webフロントエンドをビルド
echo [Whitebase] Building frontend...
call npm --prefix "%APP_DIR%" run build
if errorlevel 1 goto error
goto success

:tauri_build
rem Tauriデスクトップアプリケーションをビルド
call :prepare_native_dependencies
if errorlevel 1 goto error

echo [Whitebase] Building Tauri application...
call npm --prefix "%APP_DIR%" run tauri -- build
if errorlevel 1 goto error
goto success

:clean
rem 生成されたビルド成果物を削除
echo [Whitebase] Cleaning Rust build artifacts...
cargo clean
if errorlevel 1 goto error

if exist "%APP_DIR%\dist" (
    echo [Whitebase] Removing frontend build artifacts...
    rmdir /s /q "%APP_DIR%\dist"
)

echo.
echo [Whitebase] Clean completed.
goto success

:unknown_command
rem 未知のコマンド
echo [Whitebase] Unknown command: %~1
echo.
call :show_help
goto invalid_command

:help
rem ヘルプ表示
call :show_help
goto success

:show_help
echo Whitebase Operations / Whitebase 操作コマンド
echo.
echo Usage / 使用方法:
echo   .\scripts\ops.bat ^<command^>
echo.
echo Commands / コマンド:
echo.
echo   setup
echo     npm依存関係、WebAssemblyコンパイルターゲット、wasm-packをセットアップします。
echo     Set up npm dependencies, the WebAssembly compilation target, and wasm-pack.
echo.
echo   test
echo     Rustワークスペース全体のテストを実行します。
echo     Run Rust workspace tests.
echo.
echo   fmt
echo     Rustソースコードをフォーマットします。
echo     Format Rust source code.
echo.
echo   lint
echo     Clippyによる静的解析を実行します。
echo     Run static analysis with Clippy.
echo.
echo   check
echo     フォーマット確認、静的解析、テスト、Wasm、フロントエンド、C++バックエンド、Adapter、Assembly経路を総合検査します。
echo     Run formatting, linting, tests, WebAssembly, frontend, C++ backend, adapter, and Assembly integration checks.
echo.
echo   wasm-check
echo     WebAssemblyクレートのコンパイルを確認します。
echo     Check that the WebAssembly crate compiles.
echo.
echo   cpp-check
echo     C++からRust C ABIを呼び出せることを確認します。
echo     Check the C++ to Rust C ABI connection.
echo.
echo   cpp-backend-check
echo     C++計算バックエンドのScalar版とAVX版を確認します。
echo     Check the C++ computation backend Scalar and AVX implementations.
echo.
echo   cpp-adapter-check
echo     RustからC++計算バックエンドを呼び出せることを確認します。
echo     Check the Rust to C++ computation backend adapter.
echo.
echo   asm-check
echo     C++からAssembly関数を呼び出せることを確認します。
echo     Check the C++ to Assembly connection.
echo.
echo   tree
echo     リポジトリツリーのドキュメントを生成します。
echo     Generate the repository tree document.
echo.
echo   diagram
echo     Mermaidの構成図を更新します。
echo     Update Mermaid diagrams.
echo.
echo   dev
echo     Tauriアプリケーションを開発モードで起動します。
echo     Start the Tauri application in development mode.
echo.
echo   web-dev
echo     WebAssemblyを開発用ビルドしてWeb開発サーバーを起動します。
echo     Build WebAssembly for development and start the Web development server.
echo.
echo   wasm-build
echo     WebAssemblyのブラウザ用成果物を生成します。
echo     Build browser-compatible WebAssembly artifacts.
echo.
echo   c-api-build
echo     Rust C APIのDLLとインポートライブラリをビルドします。
echo     Build the Rust C API DLL and import library.
echo.
echo   cpp-build
echo     C++スモークテストクライアントをビルドします。
echo     Build the C++ smoke test client.
echo.
echo   cpp-backend-build
echo     C++計算バックエンドの静的ライブラリをビルドします。
echo     Build the C++ computation backend static library.
echo.
echo   asm-build
echo     Assemblyライブラリとスモークテストクライアントをビルドします。
echo     Build the Assembly library and smoke test client.
echo.
echo   web-build
echo     フロントエンドをビルドします。
echo     Build the frontend.
echo.
echo   tauri-build
echo     Tauriデスクトップアプリケーションをビルドします。
echo     Build the Tauri desktop application.
echo.
echo   clean
echo     生成されたビルド成果物を削除します。
echo     Remove generated build artifacts.
exit /b 0

:find_msbuild
rem Visual StudioのMSBuildを探す
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
set "VS_INSTALL="
set "MSBUILD="

if not exist "%VSWHERE%" (
    echo [Whitebase] ERROR: vswhere.exe was not found.
    echo [Whitebase] Visual Studio Installer could not be located.
    exit /b 1
)

for /f "usebackq delims=" %%I in (`"%VSWHERE%" -latest -products * -requires Microsoft.Component.MSBuild -property installationPath`) do (
    set "VS_INSTALL=%%I"
)

if not defined VS_INSTALL (
    echo [Whitebase] ERROR: Visual Studio with MSBuild was not found.
    exit /b 1
)

set "MSBUILD=%VS_INSTALL%\MSBuild\Current\Bin\MSBuild.exe"

if not exist "%MSBUILD%" (
    echo [Whitebase] ERROR: MSBuild.exe was not found.
    echo [Whitebase] Expected: %MSBUILD%
    exit /b 1
)

exit /b 0

:prepare_native_dependencies
rem Rust Adapterがリンクするネイティブライブラリを準備
echo.
echo [Whitebase] Preparing native libraries for Rust adapters...

call :cpp_backend_build
if errorlevel 1 exit /b 1

call :asm_build
if errorlevel 1 exit /b 1

echo [Whitebase] Native libraries are ready.
exit /b 0


:error
rem エラー処理
echo.
echo [Whitebase] Operation failed.
popd
exit /b 1

:invalid_command
rem 無効なコマンド
popd
exit /b 2

:success
rem 正常終了
popd
exit /b 0