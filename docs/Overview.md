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

## Primer Boundary

### Current Implementation

Whitebase connects built-in Rust, C++, and Assembly operations to a common compute API and compares their results and execution times. It does not yet accept Primer source or artifacts for building and execution.

- [`ComputeBackend`](../crates/backend/whitebase-backend-contract/src/backend.rs) defines calls to known operations such as array addition.
- [`BackendCapabilities`](../crates/backend/whitebase-backend-contract/src/capabilities.rs) describes support for those operations and their processing widths. `is_available()` reports availability in the current environment. Neither represents external-compiler support or permission to execute.
- [`Runner`](../crates/compute/whitebase-runner/src/runner.rs) repeats, measures, and compares Core operations. It is not a runner that builds arbitrary source through external compilers.
- [`F64Value`](../crates/compute/whitebase-runner/src/report.rs) retains values and bits received directly from operations. These observations differ from an external program's standard output.

### Integration Responsibilities

The following describes integration boundaries, not APIs that are already available.

| Owner | Responsibility |
| --- | --- |
| Primer | Syntax, types, operation semantics, diagnostics, Primer IR, artifacts for each output route, and VM execution |
| Whitebase-side experiment processing | External-tool and target selection, builds, separate-process execution, recording conditions, comparison, and measurement |
| Consumer execution environment | Authorization, isolation of untrusted code, and resource limits |

Use Primer's public CLI and artifacts rather than its internal Rust IR. Do not force generated programs into the existing `ComputeBackend` contract or consider integration complete merely by adding a `BackendKind`. Choose the placement of experiment processing once its inputs and outputs are concrete. Keep external-process launching outside Core.

Regression tests checking Primer's language rules serve a different purpose from user-configured experiments. Primer tests that execute emitted code do not necessarily duplicate Whitebase's role.

### Support and Comparison Results

Integration distinguishes generation support, build readiness, execution readiness, and permission to execute. Finding a tool does not guarantee success; record preflight checks separately from actual stage results. Distinguish unsupported routes, missing tools, policy denial, generation/build/execution failure, and timeout. Retain unexecutable routes with their reasons. Existing operation capabilities and execution statuses are not substitutes for this detail.

Whitebase having its own Linux Assembly implementation does not change Primer's `emit-asm` target: Windows x86-64, even when generated in WSL. Running C or LLVM artifacts on Linux requires separate checks of external tools and libraries. Adding Linux Direct ASM is a different task.

Existing `primer run` and `emit-*` commands can initially support exit-status and standard-output comparison. Matching displayed results does not establish equality of all internal bits, and two nonzero exits need not have the same cause. Check known expected results as well as the VM, and state comparison conditions such as newline normalization or numeric tolerance. Do not compare timings that include process startup and output with the existing Runner's compute-only timings as though they measured the same work.

### Recording and Authorization

Experiments retain input and artifact identity, Primer build identity, output route, target, external-tool versions and options, stage results, and comparison conditions. `primer --version` reports the package version. To distinguish development builds sharing a version, also record the verified commit or executable hash. A machine-readable supported-route listing is not implemented; integration can initially use the CLI and documentation of a verified version.

Observation data and artifacts must not carry executable commands or authority. Design tool invocation around trusted, user-selected executables with separate arguments, and require appropriate isolation for untrusted code. Timeouts and separate processes alone are not a security sandbox. Bound captured output and do not unconditionally record source text, paths, or entire environments.

This clarification adds no external-command execution rights to HTTP, Tauri, or browsers. Design new public schemas or bitwise observation formats when concrete consumers require them. Primer language features such as strings do not need to wait for integration to be completed.

See Primer's [Output routes and targets](https://github.com/Hokutaka/Primer/blob/master/docs/design/targets.en.md) and [Compiler architecture](https://github.com/Hokutaka/Primer/blob/master/docs/design/architecture.en.md) for its outputs and responsibilities.

## How to Verify Operation

Whitebase can be tested in the following three ways.

| Runtime | Server | Whitebase call path |
| --- | ---: | --- |
| Tauri app | Not required | Tauri → Tauri Commands → Interface → Runner / Core |
| GitHub Pages | Optional | Browser → WASM → Interface → Runner / Core, or when loopback access is available: Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |
| Local web app | Optional | Browser → WASM → Interface → Runner / Core, or Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |

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

Computation is routed from Tauri Commands through `whitebase-interface`,
which uses Runner or Whitebase Core depending on the operation,
to the available native backends.

```text
Tauri
→ Tauri Commands
→ whitebase-interface
→ Runner / Whitebase Core
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
→ whitebase-http-api
→ whitebase-interface
→ Runner / Whitebase Core
→ Native backend
```

If Whitebase Server is unavailable or cannot be reached, WebAssembly is used.

```text
Browser
→ WebAssembly
→ whitebase-interface
→ Runner / Whitebase Core
→ Rust backend
```

In the WebAssembly environment, Rust Scalar and Rust SIMD backends are available.

The current Whitebase WebAssembly artifact requires a runtime with WebAssembly
SIMD128 support.

On wasm32, the Rust SIMD backend uses WebAssembly SIMD128.
A baseline artifact for runtimes without SIMD128 support is not currently provided.

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

Each Application Interface uses `whitebase-interface` as the shared
Application Boundary.

`whitebase-interface` is transport-independent. It uses Whitebase Core
directly for Pure Compute and Runner for Applied Compute such as comparison,
measurement, and observation.

For HTTP, `whitebase-http-api` handles HTTP / JSON / Axum-specific behavior,
while `whitebase-server` acts only as the Host that starts the Interface Adapter.

Tauri Commands and WebAssembly also avoid exposing Whitebase Core / Runner
directly and use `whitebase-interface` for their respective execution
environments.

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

### Timing Measurement

Whitebase Runner measures each measured iteration individually.

If any measured iteration produces `Duration::ZERO`, that value is not treated
as an actual execution time of `0 ns`.

This means the operation completed faster than the current timer resolution can
reliably observe as an individual execution. In that case, Runner reports the
timing as `TooFastToMeasure`.

`TooFastToMeasure` does not mean that backend execution failed.

- Backend status: `Completed`
- Result comparison: still performed normally
- Timing: `TooFastToMeasure`
- Mean / Minimum / Maximum / Total: not reported
- Fastest / Speedup: excluded from calculation

Whitebase does not replace timing values below timer resolution with estimated
values. The inability to observe the individual execution time is reported as
part of the benchmark result.

### Core API

See [Core API](api/Core-API.md) for the Rust API, operations, backends, and errors.

### HTTP API

`whitebase-http-api` provides the local HTTP / JSON Interface Adapter.

`whitebase-server` hosts that adapter and handles server startup.

See [HTTP API](api/HTTP-API.md) for endpoint and request/response details.
