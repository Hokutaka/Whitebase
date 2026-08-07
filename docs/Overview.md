# Whitebase

Whitebase is a repository for learning and experimentation.

Its APIs and project structure are expected to change frequently.

## Purpose

The purpose of Whitebase is to implement systems in small units so they can be observed, documented, compared, and visualized.

Examples include:

- Scalar and SIMD implementations
- Rust, C++, and Assembly
- C ABI and FFI
- Debug and Release builds
- Tauri IPC
- HTTP and JSON
- WebAssembly
- Desktop and browser-based user interfaces

## Running Whitebase

Whitebase can be tested in the following three ways.

| Method | Server required | Core invocation path |
|---|---:|---|
| Tauri application | No | Tauri → Rust → Whitebase Core |
| GitHub Pages | Yes | Browser → HTTP → Whitebase Server → Core |
| Local web application | Yes | Browser → HTTP → Whitebase Server → Core |

### Tauri Application

Start the Whitebase App.

The Tauri application does not require Whitebase Server. It calls Whitebase Core directly from the Rust code embedded in the application.

### GitHub Pages + Whitebase Server

Start Whitebase Server, then open the Whitebase App hosted on GitHub Pages.

GitHub Pages provides only the static frontend. Computation is performed through the Whitebase Server HTTP API.

### Local Web Application + Whitebase Server

Start Whitebase Server and the local frontend development server.

```powershell
cargo run -p whitebase-server
npm --prefix apps/whitebase-app run dev
```

Open the local URL displayed by the development server in your browser.

### Differences Between Build Configurations

Development and Release environments may differ in the following areas:

- Rust optimization levels
- Debug and Release versions of the C++ and Assembly libraries
- SIMD performance
- Benchmark results
- Output locations for executables and application bundles

Benchmark results in particular may differ significantly between Debug and Release builds.

Calculation results are expected to be validated across backends regardless of the selected build configuration.

## Architecture

Tauri uses IPC, while regular web browsers use the HTTP API.

Both paths use the same Runner and Core.

![Architecture diagram](/docs/diagrams/structure/architecture.svg)

## Diagrams

### Module Diagram

The arrows indicate dependencies, usage relationships between modules, or the direction in which calculation results are passed.

Modules with dashed outlines are planned for future implementation.

![Module diagram](/docs/diagrams/structure/module.svg)

### Usage Diagram

This diagram shows the intended ways to use the libraries.

`whitebase-core` acts as the center of computation and is called through the appropriate boundary for each target environment.

![Usage diagram](/docs/diagrams/structure/usage.svg)

## Implemented Benchmarks

```text
Generate input
→ Warm up the reference backend
→ Execute multiple measured iterations
→ Calculate minimum, maximum, average, and total execution times
→ Compare errors against the reference result
→ Return the report to Tauri or the browser
```

Benchmark results are affected by the build configuration, CPU, cache, memory bandwidth, operating-system scheduling, and other environmental factors.

Release builds are recommended instead of Debug builds when comparing performance.

### Core API

Rust API、演算、バックエンド、エラーについては [Core API](api/Core-API.md) を参照してください。

### HTTP API

Whitebase Server exposes a local HTTP/JSON API.

See [HTTP API](api/HTTP-API.md) for endpoint and request/response details.