# Whitebase

[日本語](Overview.ja.md) | English

Whitebase is a learning and experimentation repository for implementing compute paths and interfaces in small pieces, then observing, recording, comparing, and visualizing their differences.

Its APIs and structure may change as new experiments and backends are added.

## Purpose

Whitebase aims to connect different implementations and execution environments to the same observation framework without hiding the differences that make them interesting.

Examples include:

- Scalar and SIMD
- Rust / C++ / Assembly
- Cerune VM
- C ABI and FFI
- Debug / Release builds
- Tauri IPC
- HTTP + JSON
- WebAssembly
- Desktop and browser UIs
- Per-backend capabilities, availability, and execution results
- IEEE 754 value and bit-level observation

Whitebase favors making execution paths visible and comparable rather than flattening every implementation behind one opaque abstraction.

## Boundary with Cerune

### What is available today

Whitebase connects Rust, C++, and Assembly compute backends to a common compute API, and it now also integrates Cerune VM as a backend.

The current Cerune path compiles Cerune source to bytecode, resolves the target function, and invokes `AddScalarF64` through Cerune VM.

```text
Cerune Source
    ↓
compile to bytecode
    ↓
resolve function
    ↓
Cerune VM
    ↓
whitebase-cerune-vm-adapter
    ↓
whitebase-backend-bridge
    ↓
Whitebase Core
    ↓
Whitebase Runner
```

[`ComputeBackend`](../crates/backend/whitebase-backend-contract/src/backend.rs) defines the common contract implemented by Whitebase compute backends.

[`BackendCapabilities`](../crates/backend/whitebase-backend-contract/src/capabilities.rs) describes which operations a backend provides and its approximate array processing width. Capability support and runtime availability are treated separately.

[`Runner`](../crates/compute/whitebase-runner/src/runner.rs) builds timing, comparison, and observation behavior on top of Core operations.

[`F64Value`](../crates/compute/whitebase-runner/src/report.rs) stores an `f64` value together with its IEEE 754 bit representation.

From Core's point of view, Cerune VM is a normal `ComputeBackend`. Cerune-specific bytecode, VM values, and function handles are not exposed through the Core API.

### Responsibilities

Cerune and Whitebase keep their responsibilities separate.

| Owner | Responsibility |
| --- | --- |
| Cerune | Syntax, types, operation semantics, diagnostics, Cerune IR, bytecode / VM, and emitters |
| Whitebase Backend Integration | Preparation and adaptation required to expose Cerune VM or emitted artifacts as Whitebase backends |
| Whitebase Core | Backend registration, capability checks, availability checks, and dispatch |
| Whitebase Runner | Timing, comparison, observation, and aggregation of per-backend execution states |
| Execution environment | Execution permission, external toolchains, isolation, and resource limits where required |

The Cerune VM route is implemented today.

Artifact routes for C, LLVM, QBE, WAT, Assembly, Native Object, and other Cerune outputs are future extensions and are kept separate from the VM route.

Discovery, invocation, and availability checks for external compilers, assemblers, linkers, or Wasm runtimes are not Cerune responsibilities. They also do not belong directly in `whitebase-core`.

See [Cerune Integration](Layer/Cerune-Integration.md) for the detailed integration boundary.

### Support status and comparison results

Whitebase distinguishes several different questions:

- Does a backend advertise the requested operation?
- Is that backend available in the current environment?
- Did execution succeed?
- Does the result match the reference?
- Do successful backends agree with each other?

Benchmark reports retain `Completed / Unavailable / Failed` for each backend.

Scalar `f64` observation follows the same model. One backend being unavailable or failing does not prevent the remaining backends from being observed.

`allBackendsMatch` compares only the result bits from backends that completed successfully. It is `false` when no backend completed.

`matchesReferenceBits` is a different metric. It tells whether one backend's result bits match the `f64` value produced by adding the input decimal strings exactly and then rounding that exact decimal result to `f64`.

For example, with `0.1 + 0.2`, Whitebase can observe a case where:

- every completed backend returns the same binary `f64` result; while
- that result differs by one bit from the exact-decimal reference rounded to `f64`.

Future artifact routes will likewise distinguish "can emit", "can build", "can load", "can execute", and "produced a matching result" instead of collapsing them into one status.

### Recording and execution permission

For the Cerune VM route, compilation and function resolution are completed before the operation invocation.

```text
Preparation

source
  ↓
compile to bytecode
  ↓
resolve function

Measurement / Observation

arguments
  ↓
VM invocation
  ↓
result
```

This keeps Cerune compilation time out of the normal backend operation timing.

Future artifact routes should follow the same principle where practical: build, link, and load belong to preparation, while the callable operation is measured separately.

If compilation or linking time becomes an experiment of its own, it should be measured as a separate subject rather than mixed with normal operation benchmarks.

Adding external-toolchain routes does not mean giving observations or generated artifacts command-execution authority. Whitebase also avoids adding unnecessary external-command execution to public HTTP, Tauri, or browser-facing interfaces.

## How to run Whitebase

Whitebase can primarily be exercised in three ways.

| Execution method | Server | Whitebase call path |
| --- | ---: | --- |
| Tauri app | Not required | Tauri → Tauri Commands → Interface → Runner / Core |
| GitHub Pages | Optional | Browser → WASM → Interface → Runner / Core, or Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |
| Local web app | Optional | Browser → WASM → Interface → Runner / Core, or Browser → HTTP → Whitebase Server → HTTP API Adapter → Interface → Runner / Core |

In a browser environment, the execution route is selected during application startup.

1. Tauri uses Tauri Commands.
2. A normal browser probes the Whitebase Server Health API.
3. If the Health API is reachable, the HTTP API route is used.
4. If the server is unavailable, the application initializes WebAssembly and uses the WASM route.

The selected route is kept for the lifetime of that page session.

Reload the page after starting or stopping the server if you want the browser to select a different route.

### Tauri app

Start Whitebase App with:

```powershell
npm --prefix apps\whitebase-app run tauri dev
```

or through Whitebase Operations:

```powershell
scripts\ops.bat dev
```

The Tauri application does not require Whitebase Server.

Compute requests go through Tauri Commands and `whitebase-interface`, then use Runner or Whitebase Core depending on the operation.

```text
Tauri WebView
  ↓
Tauri IPC / invoke()
  ↓
Tauri Host (src-tauri)
  ↓
whitebase-tauri-api
  ↓
whitebase-interface
  ↓
Runner / Whitebase Core
  ↓
Backend
```

In native environments, Rust, C++, Assembly, Cerune VM, and other registered backends are used according to platform support and runtime availability.

### GitHub Pages

Whitebase App on GitHub Pages probes the Whitebase Server Health API during startup and selects its execution route based on the result.

If the browser permits loopback HTTP access and Whitebase Server is reachable, the HTTP API route is used.

```text
Browser
  ↓
HTTP
  ↓
Whitebase Server
  ↓
whitebase-http-api
  ↓
whitebase-interface
  ↓
Runner / Whitebase Core
  ↓
Backend
```

If the server cannot be reached, the application uses WebAssembly.

```text
Browser
  ↓
WebAssembly
  ↓
whitebase-interface
  ↓
Runner / Whitebase Core
  ↓
Backends available on WASM
```

The Core / Runner structure is preserved on the WASM route. Which backends participate depends on operation capabilities and availability in the wasm32 environment.

Rust SIMD uses WebAssembly SIMD128 on wasm32. The current Whitebase WebAssembly artifact requires a SIMD128-capable runtime; a baseline artifact for runtimes without SIMD128 is not currently provided.

### Local web app

Start the local frontend development server with:

```powershell
npm --prefix apps\whitebase-app run dev
```

or start it together with a development WebAssembly build:

```powershell
scripts\ops.bat web-dev
```

When Whitebase Server is not running, the application uses the WebAssembly route.

To exercise the server route, start Whitebase Server in another terminal:

```powershell
cargo run -p whitebase-server
```

Then reload the local web application.

If the Health API is reachable, the HTTP API route is selected.

### WebAssembly build profiles

Development and Release WebAssembly artifacts are both generated under `apps/whitebase-app/src/wasm`, but they use different build profiles.

For development:

```powershell
scripts\ops.bat web-dev
```

For a Release WebAssembly artifact:

```powershell
scripts\ops.bat wasm-build
```

For a Release web frontend, build the Release WebAssembly package first and then build the frontend:

```powershell
scripts\ops.bat web-build
```

### Differences between build profiles

Development and Release environments may differ in:

- Rust optimization level
- WebAssembly optimization level
- Debug / Release C++ and Assembly libraries
- SIMD performance
- benchmark results
- output locations for executables and bundles

Benchmark results can differ substantially between Debug and Release builds.

Performance comparisons should use the same platform, build profile, and input conditions. Release builds are recommended for performance-oriented comparisons.

Computation correctness is expected to remain verifiable across backends regardless of build profile.

## Architecture

Whitebase separates Backend Contract, Backend Implementation, Backend Integration, Core, Runner, Interface, and Presentation responsibilities.

The main layers are:

| Layer | Role | Main components |
| --- | --- | --- |
| L1 Backend Contract | Common backend contract | `whitebase-backend-contract` |
| L2 Backend Implementation | Compute implementation | Rust / C++ / Assembly |
| L3 Backend Integration | Adapters and backend integration | C++ / Assembly / Windows GNU / Cerune VM adapters, `whitebase-backend-bridge` |
| L4 Core | Pure Compute | `whitebase-core` |
| L5 Runner | Applied Compute | `whitebase-runner` |
| L6 Interface | Application Boundary / Interface Adapter | `whitebase-interface`, HTTP, Tauri, WASM |
| L7 Presentation | User Facing | Whitebase App / Browser UI |

Tauri uses IPC, while a normal browser selects either HTTP API or WebAssembly during startup.

Every application-facing adapter uses `whitebase-interface` as the shared application boundary.

`whitebase-interface` is transport-independent. It uses Whitebase Core for Pure Compute and Runner for Applied Compute such as timing, comparison, and observation.

For HTTP, `whitebase-http-api` owns HTTP / JSON / Axum-specific behavior, while `whitebase-server` acts as the host that starts the adapter.

Tauri Commands and WebAssembly also use `whitebase-interface` rather than exposing Core or Runner directly to their respective environments.

The FFI Boundary and Operations Plane are kept outside the seven main layers.

- `whitebase-c-api`: C ABI for native consumers
- Control Center / scripts / CI: operations for build, test, run, release, and environment checks

See [Layer Overview](Layer/Layer-Overview.md) for the detailed layer model.

![Architecture](/docs/diagrams/structure/architecture.svg)

## Diagrams

### Module diagram

Arrows represent dependency or usage relationships between modules, or the direction in which computation results are passed.

Modules shown with dashed borders are planned future additions.

![Module diagram](/docs/diagrams/structure/module.svg)

### Usage diagram

`whitebase-core` is the center of Pure Compute and is accessed through the appropriate Interface or FFI Boundary for each environment.

When timing, comparison, or observation is required, the call path goes through `whitebase-runner`.

![Usage diagram](/docs/diagrams/structure/usage.svg)

## Implemented benchmarks

Whitebase Runner benchmarks follow this general flow:

```text
generate input
  ↓
prepare reference backend
  ↓
warmup
  ↓
run measured iterations
  ↓
aggregate min / max / mean / total time
  ↓
compare with the reference result
  ↓
collect per-backend state into a report
  ↓
return through the Interface to Tauri / Browser
```

Benchmark backend results use `Completed / Unavailable / Failed`.

Benchmark results are affected by build profile, CPU, cache state, memory bandwidth, OS scheduling, and other environmental factors.

Release builds are recommended for performance comparisons.

### Timing behavior

Whitebase Runner measures each measured iteration independently.

If any measured iteration produces `Duration::ZERO`, Whitebase does not interpret that value as a real `0 ns` execution time.

Instead, it means the operation completed below the effective timer resolution, so timing is reported as `TooFastToMeasure`.

`TooFastToMeasure` is not a backend execution failure.

- Backend Status: `Completed`
- Result comparison: still performed normally
- Timing: `TooFastToMeasure`
- Mean / Minimum / Maximum / Total: not reported
- Fastest / Speedup: excluded

Whitebase does not replace sub-resolution measurements with estimated timing values. It records that the duration could not be measured accurately.

### Scalar f64 observation

Scalar `f64` observation is separate from benchmarking. It observes the relationship between decimal input strings and the `f64` result returned by each backend.

```text
decimal input
  ↓
exact decimal reference
  ↓
reference rounded to f64
  ↓
observe each AddScalarF64 backend
  ↓
compare value / decimal / IEEE 754 bits
```

Each backend has one of three states:

- `Completed`: includes the result and `matchesReferenceBits`
- `Unavailable`: cannot run in the current environment
- `Failed`: backend execution failed

An unavailable or failed backend does not stop observation of the remaining backends.

`allBackendsMatch` compares result bits among successful backends. `matchesReferenceBits` compares one backend's result against the `f64` reference derived from the exact decimal result.

They intentionally answer different questions.

### Core API

See [Core API](api/Core-API.en.md) for the Rust API, operations, backends, capabilities, and errors.

### HTTP API

`whitebase-http-api` provides the local HTTP / JSON Interface Adapter.

The main endpoints are:

- `GET /api/health`
- `POST /api/observations/add-scalar-f64`
- `POST /api/benchmarks/run`
- `POST /api/benchmarks/add-array`
- `POST /api/benchmarks/add-f32` (compatibility endpoint)

`whitebase-server` acts as the host that starts the adapter.

See [HTTP API](api/HTTP-API.en.md) for endpoint, request, and response details.
