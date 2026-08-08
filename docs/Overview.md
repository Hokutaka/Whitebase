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

## How to Verify Operation

Whitebase can be tested in the following three ways.

| Execution method | Server | Core call path |
| --- | ---: | --- |
| Tauri app | Not required | Tauri → Tauri Commands → Runner → Core |
| GitHub Pages | Optional | Browser → WASM → Runner → Core, or, when loopback access is available, Browser → HTTP → Whitebase Server → Runner → Core |
| Local web app | Optional | Browser → WASM → Runner → Core, or Browser → HTTP → Whitebase Server → Runner → Core |

In browser environments, the execution route is selected when the application starts.

1. In a Tauri environment, Tauri Commands are used.
2. In a browser environment, Whitebase checks the Whitebase Server health API.
3. If the Whitebase Server Health API can be reached, the HTTP API is used.
4. If the Server cannot be reached, WebAssembly is used.

The selected execution route does not change during the current session.
Reload the page after starting or stopping the Server to detect the route again.

### Tauri App

Start Whitebase App.

```powershell
npm --prefix apps\whitebase-app run tauri dev
```

Alternatively, use Whitebase Operations.

```powershell
scripts\ops.bat dev
```

The Tauri app does not require Whitebase Server.

Computation is routed from Tauri Commands through Runner and Whitebase Core
to the available native backends.

```text
Tauri
→ Tauri Commands
→ Runner
→ Whitebase Core
→ Native backend
```

### GitHub Pages

Whitebase App on GitHub Pages checks the Whitebase Server Health API at startup
and automatically selects an execution route.

If the browser permits loopback HTTP access and the Whitebase Server Health API
can be reached, the HTTP API is used. If the Server cannot be reached because of
browser security settings or permissions, the application falls back to WebAssembly.

```text
Browser
→ HTTP
→ Whitebase Server
→ Runner
→ Whitebase Core
→ Native backend
```

If Whitebase Server is unavailable or cannot be reached, WebAssembly is used.

```text
Browser
→ WebAssembly
→ Runner
→ Whitebase Core
→ Rust backend
```

In the WebAssembly environment, Rust Scalar and Rust SIMD backends are available.

On wasm32, the Rust SIMD backend uses WebAssembly SIMD128.

### Local Web App

Start the local frontend development server.

```powershell
npm --prefix apps\whitebase-app run dev
```

Alternatively, start it together with a development WebAssembly build.

```powershell
scripts\ops.bat web-dev
```

If Whitebase Server is not running, the WebAssembly route is used.

To test the Whitebase Server route, start the Server in another terminal.

```powershell
cargo run -p whitebase-server
```

Then reload the local web app.

If the Server is available, the HTTP API route is selected.

### WebAssembly Build Configuration

Development and Release WebAssembly builds are generated into the same
`apps/whitebase-app/src/wasm` directory, but use different build profiles.

For development, use the Debug WebAssembly build.

```powershell
scripts\ops.bat web-dev
```

To generate browser-compatible Release WebAssembly artifacts, use:

```powershell
scripts\ops.bat wasm-build
```

To build the web frontend for Release, Whitebase first generates the Release
WebAssembly artifacts and then builds the frontend.

```powershell
scripts\ops.bat web-build
```

### Differences Between Build Configurations

Debug and Release builds may differ in the following areas:

- Rust optimization level
- WebAssembly optimization level
- C++ and Assembly Debug / Release libraries
- SIMD performance
- Benchmark results
- Executable and bundle output locations

Benchmark results can differ significantly between Debug and Release builds.
Use Release builds as the baseline when comparing performance.

Computation correctness is expected to be validated across backends regardless
of the build configuration.


## Architecture

Tauri uses IPC, while regular web browsers select either the local HTTP API
or WebAssembly at startup.

These execution routes share the same Runner and Core layers.

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

See [Core API](api/Core-API.md) for the Rust API, operations, backends, and errors.
### HTTP API

Whitebase Server exposes a local HTTP/JSON API.

See [HTTP API](api/HTTP-API.md) for endpoint and request/response details.