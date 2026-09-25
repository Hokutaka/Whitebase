# Whitebase Core API

[日本語](Core-API.ja.md) | English

`whitebase-core` is the Core layer that exposes registered Whitebase compute backends through a common Rust API.

> [!IMPORTANT]
> Whitebase is an experimental repository for learning and exploration. Operations, backends, and capabilities may change over time.

## Scope

This document covers the public API of the `whitebase-core` crate.

Core is responsible for:

- registering standard backends;
- enumerating backends;
- exposing capabilities and availability;
- executing one computation on a selected `BackendKind`;
- exposing common backend, operation, and error types.

Core is not responsible for:

- warmup;
- repeated timing;
- cross-backend comparison;
- benchmark report generation;
- scalar observation aggregation;
- HTTP, Tauri, or WASM transport;
- UI rendering.

Those responsibilities belong to `whitebase-runner`, `whitebase-interface`, the transport crates, and application layers.

## Crate

```toml
[dependencies]
whitebase-core = { path = "crates/compute/whitebase-core" }
```

The current crate version is `0.1.0`. It is intended for workspace-internal use and has `publish = false`.

Core does not expose Cerune-specific APIs or external toolchain concerns. From Core's point of view, Cerune VM is a normal `ComputeBackend`.

## API Summary

| API | Purpose |
| --- | --- |
| `Whitebase::new()` | Create a Core instance with the standard backends registered |
| `Whitebase::default()` | Equivalent to `Whitebase::new()` |
| `Whitebase::backends()` | Return information about all registered backends |
| `Whitebase::backend_info(kind)` | Return information about one backend |
| `Whitebase::add_f32(...)` | Element-wise addition of two `f32` arrays |
| `Whitebase::add_f64(...)` | Element-wise addition of two `f64` arrays |
| `Whitebase::add_scalar_f64(...)` | Add two scalar `f64` values |
| `Whitebase::sum_f64(...)` | Reduce an `f64` array to one sum |

## Basic Example

```rust
use whitebase_core::{BackendKind, Whitebase};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core = Whitebase::new();

    let lhs = [1.0_f64, 2.0, 3.0];
    let rhs = [10.0_f64, 20.0, 30.0];
    let mut output = [0.0_f64; 3];

    core.add_f64(
        BackendKind::RustScalar,
        &lhs,
        &rhs,
        &mut output,
    )?;

    assert_eq!(output, [11.0, 22.0, 33.0]);

    let sum = core.sum_f64(BackendKind::RustScalar, &output)?;
    assert_eq!(sum, 66.0);

    Ok(())
}
```

Core does not automatically select a backend. The caller supplies a `BackendKind`.

## Operations

Core operations are represented by `OperationKind`.

| `OperationKind` | Core API | Input | Output |
| --- | --- | --- | --- |
| `AddF32` | `add_f32` | `&[f32]`, `&[f32]`, `&mut [f32]` | `Result<(), ComputeError>` |
| `AddF64` | `add_f64` | `&[f64]`, `&[f64]`, `&mut [f64]` | `Result<(), ComputeError>` |
| `AddScalarF64` | `add_scalar_f64` | `f64`, `f64` | `Result<f64, ComputeError>` |
| `SumF64` | `sum_f64` | `&[f64]` | `Result<f64, ComputeError>` |

### `add_f32`

```rust
pub fn add_f32(
    &self,
    kind: BackendKind,
    lhs: &[f32],
    rhs: &[f32],
    output: &mut [f32],
) -> Result<(), ComputeError>
```

Computes `lhs[i] + rhs[i]` on the selected backend and writes the result to `output[i]`.

If the backend does not advertise `AddF32`, Core returns `OperationUnsupported`. If the backend is currently unavailable, Core returns `BackendUnavailable`.

Array length mismatches are reported by backend implementations as `LengthMismatch`.

### `add_f64`

```rust
pub fn add_f64(
    &self,
    kind: BackendKind,
    lhs: &[f64],
    rhs: &[f64],
    output: &mut [f64],
) -> Result<(), ComputeError>
```

Performs element-wise `f64` array addition on the selected backend.

### `add_scalar_f64`

```rust
pub fn add_scalar_f64(
    &self,
    kind: BackendKind,
    lhs: f64,
    rhs: f64,
) -> Result<f64, ComputeError>
```

Adds two scalar `f64` values using the selected backend.

In addition to native scalar backends, `CeruneVm` exposes this operation. AVX / SIMD backends do not advertise `AddScalarF64`.

### `sum_f64`

```rust
pub fn sum_f64(
    &self,
    kind: BackendKind,
    input: &[f64],
) -> Result<f64, ComputeError>
```

Reduces an `f64` array to one sum using the selected backend.

The backend contract defines the sum of an empty array as `0.0`.

## Backends

### `BackendKind`

The currently defined backend kinds are:

| `BackendKind` | Display name | Implementation |
| --- | --- | --- |
| `RustScalar` | `Rust Scalar` | Rust Scalar |
| `RustSimd` | `Rust SIMD` | Rust SIMD |
| `CeruneVm` | `Cerune VM` | Cerune VM |
| `CppScalar` | `C++ Scalar` | C++ Scalar |
| `CppAvx` | `C++ AVX` | C++ AVX |
| `AssemblyScalar` | `Assembly Scalar` | Assembly Scalar |
| `AssemblyAvx` | `Assembly AVX` | Assembly AVX |
| `WindowsGnuCppScalar` | `Windows GCC Scalar` | Windows GNU / GCC Scalar |
| `WindowsGnuCppAvx` | `Windows GCC AVX` | Windows GNU / GCC AVX |
| `WindowsGnuAssemblyScalar` | `Windows NASM Scalar` | Windows GNU / NASM Scalar |
| `WindowsGnuAssemblyAvx` | `Windows NASM AVX` | Windows GNU / NASM AVX |

### Standard Registration

On normal targets, `Whitebase::new()` registers:

```text
Rust Scalar
Rust SIMD
Cerune VM
C++ Scalar
C++ AVX
Assembly Scalar
Assembly AVX
```

On `x86_64-pc-windows-msvc`, it additionally registers:

```text
Windows GCC Scalar
Windows GCC AVX
Windows NASM Scalar
Windows NASM AVX
```

Being registered is different from being available in the current execution environment.

## Capabilities

`BackendCapabilities` describes what a backend can execute.

### Operation Support

The current capability model by backend class is:

| Backend class | `AddF32` | `AddF64` | `AddScalarF64` | `SumF64` |
| --- | :---: | :---: | :---: | :---: |
| Native Scalar | ✓ | ✓ | ✓ | ✓ |
| AVX / SIMD | ✓ | ✓ | — | ✓ |
| Cerune VM | — | — | ✓ | — |

`CeruneVm` currently supports only `AddScalarF64`.

Capabilities indicate whether a backend provides an operation. Runtime availability is reported separately through `available`.

### Vector Width

`BackendCapabilities` also exposes array processing width.

| Backend class / target | `vector_width_f32` | `vector_width_f64` |
| --- | ---: | ---: |
| Native Scalar | `1` | `1` |
| Rust SIMD (`aarch64` / `wasm32`) | `4` | `2` |
| 256-bit AVX / SIMD | `8` | `4` |
| Cerune VM | `0` | `0` |

`0` means that the backend does not expose a processing width for that array type. Cerune VM currently exposes only scalar `f64` addition, so both vector widths are `0`.

### Checking Capabilities

```rust
use whitebase_core::{OperationKind, Whitebase};

let core = Whitebase::new();

for backend in core.backends() {
    if backend.available
        && backend
            .capabilities
            .supports(OperationKind::AddScalarF64)
    {
        println!(
            "{} supports AddScalarF64",
            backend.kind.display_name()
        );
    }
}
```

Main fields exposed by `BackendCapabilities`:

| Field | Type | Meaning |
| --- | --- | --- |
| `add_f32` | `bool` | Supports `AddF32` |
| `add_f64` | `bool` | Supports `AddF64` |
| `add_scalar_f64` | `bool` | Supports `AddScalarF64` |
| `sum_f64` | `bool` | Supports `SumF64` |
| `vector_width_f32` | `usize` | Approximate `f32` array processing width |
| `vector_width_f64` | `usize` | Approximate `f64` array processing width |

`supports(OperationKind)` can also be used to query support by operation.

## Backend Information API

### `backends`

```rust
pub fn backends(&self) -> Vec<BackendInfo>
```

Returns `BackendInfo` for every registered backend.

```rust
pub struct BackendInfo {
    pub kind: BackendKind,
    pub capabilities: BackendCapabilities,
    pub available: bool,
}
```

- `kind`: backend identifier;
- `capabilities`: operations provided by the backend;
- `available`: whether the backend can currently execute.

### `backend_info`

```rust
pub fn backend_info(
    &self,
    kind: BackendKind,
) -> Result<BackendInfo, ComputeError>
```

Returns information about one backend.

If that `BackendKind` is not registered in Core, the call returns `BackendNotRegistered`.

## Execution Checks

Core operation methods follow this general sequence:

```text
Backend registered?
      ↓ yes
Operation supported?
      ↓ yes
Backend available?
      ↓ yes
invoke backend
```

The corresponding errors are:

```text
not registered
  → BackendNotRegistered

registered, operation unsupported
  → OperationUnsupported

registered, supported, unavailable
  → BackendUnavailable

backend invocation failed
  → BackendFailure
```

Core does not silently fall back to a different backend when one backend is unavailable or fails.

## Cerune VM Backend

`CeruneVm` is a backend adapted from Cerune VM at the L3 Backend Integration layer.

```text
Cerune source
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
```

Its current capability is `AddScalarF64` only.

Cerune-specific bytecode, VM values, and function handles are not exposed through Core. From the Core API, `CeruneVm` behaves like any other `ComputeBackend`.

Cerune compilation and function resolution are separated from the operation invocation and are performed while preparing the execution target.

See [Cerune Integration](../Layer/Cerune-Integration.md) for the wider integration boundary.

## Errors

Core computation APIs return `ComputeError`.

| Variant | Condition |
| --- | --- |
| `LengthMismatch` | Array input/output lengths do not match |
| `BackendUnavailable` | The backend is registered and supports the operation but cannot run in the current environment |
| `OperationUnsupported` | The backend does not advertise the requested operation |
| `BackendFailure` | Backend or adapter execution failed |
| `BackendNotRegistered` | The requested backend is not registered in Core |

Backend-specific errors may be mapped to `BackendFailure`.

## Public Types

`whitebase-core` publicly exposes:

| Type | Description |
| --- | --- |
| `Whitebase` | Unified compute API |
| `BackendInfo` | Backend capabilities and availability |
| `BackendKind` | Backend identifier |
| `BackendCapabilities` | Operation support and processing width |
| `OperationKind` | Operation identifier |
| `ComputeBackend` | Common backend trait |
| `ComputeError` | Core / backend computation error |

The following aliases are also exported:

| Alias | Original type |
| --- | --- |
| `Backend` | `BackendKind` |
| `Capabilities` | `BackendCapabilities` |
| `Error` | `ComputeError` |

## Core and Runner Responsibilities

Core executes one computation on one selected backend.

```text
Caller
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Backend implementation / adapter
```

Runner builds timing, comparison, and observation behavior on top of Core.

```text
Caller
  ↓
Whitebase Runner
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Backend implementation / adapter
```

For example, per-backend `Completed / Unavailable / Failed` aggregation for scalar `f64` observation and cross-backend result comparison belong to Runner, not to the Core API return types.

## HTTP API Mapping

The HTTP API uses the same compute system, but it is not a one-to-one transport wrapper around Core.

| Concept | Core API | HTTP API |
| --- | --- | --- |
| `f32` array addition | `Whitebase::add_f32` | Benchmark `add-array` + `precision: "f32"` |
| `f64` array addition | `Whitebase::add_f64` | Benchmark `add-array` + `precision: "f64"` |
| Scalar `f64` addition | `Whitebase::add_scalar_f64` | `POST /api/observations/add-scalar-f64` |
| `f64` array sum | `Whitebase::sum_f64` | Benchmark `sum-f64` + `precision: "f64"` |

The HTTP benchmark API generates inputs on the server and uses Runner for warmup, timing, and comparison.

Scalar observation uses Runner to traverse multiple backends and produce a report containing per-backend status and IEEE 754 observation data.

See [HTTP API](HTTP-API.en.md) for endpoint, request, and response details.
