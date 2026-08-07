# Whitebase Core API

`whitebase-core` exposes Whitebase compute backends through a unified Rust API.

> [!IMPORTANT]
> Whitebase is a repository for learning and experimentation. The Core API may change as operations and backends are added.

## Scope

This document covers the public API exposed by the `whitebase-core` crate.

- Enumerating backends and inspecting availability/capabilities
- Running computations on a selected backend
- Common types and errors exposed by Core

Timing, warmup, cross-backend comparison, and benchmark report generation belong to `whitebase-runner`, not Core.

## Crate

```toml
[dependencies]
whitebase-core = { path = "crates/whitebase-core" }
```

The current crate version is `0.1.0`. It is intended for workspace-internal use and has `publish = false`.

## API Summary

| API | Purpose |
|---|---|
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

## Operations

Core operations are represented by `OperationKind`.

| `OperationKind` | Core API | Input | Output |
|---|---|---|---|
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

If `lhs`, `rhs`, and `output` do not have the same length, the call returns `ComputeError::LengthMismatch`.

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

If `lhs`, `rhs`, and `output` do not have the same length, the call returns `ComputeError::LengthMismatch`.

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

Currently, scalar backends expose this operation. AVX/SIMD backends return `OperationUnsupported`.

### `sum_f64`

```rust
pub fn sum_f64(
    &self,
    kind: BackendKind,
    input: &[f64],
) -> Result<f64, ComputeError>
```

Reduces an `f64` array to one sum using the selected backend.

The sum of an empty array is `0.0`.

## Backends

### `BackendKind`

The following backend kinds are currently defined.

| `BackendKind` | Display name | Implementation |
|---|---|---|
| `RustScalar` | `Rust Scalar` | Rust Scalar |
| `RustSimd` | `Rust SIMD` | Rust AVX/SIMD |
| `CppScalar` | `C++ Scalar` | C++ Scalar |
| `CppAvx` | `C++ AVX` | C++ AVX |
| `AssemblyScalar` | `Assembly Scalar` | Assembly Scalar |
| `AssemblyAvx` | `Assembly AVX` | Assembly AVX |
| `WindowsGnuCppScalar` | `Windows GCC Scalar` | Windows GNU / GCC Scalar |
| `WindowsGnuCppAvx` | `Windows GCC AVX` | Windows GNU / GCC AVX |
| `WindowsGnuAssemblyScalar` | `Windows NASM Scalar` | Windows GNU / NASM Scalar |
| `WindowsGnuAssemblyAvx` | `Windows NASM AVX` | Windows GNU / NASM AVX |

The Windows GNU backends are additionally registered by Core on `x86_64-pc-windows-msvc`.

The other standard backends are Rust Scalar/SIMD, C++ Scalar/AVX, and Assembly Scalar/AVX. Actual availability depends on the platform, CPU, and native-library state.

### Operation Support

The current capability-level operation support is:

| Backend class | `AddF32` | `AddF64` | `AddScalarF64` | `SumF64` |
|---|:---:|:---:|:---:|:---:|
| Scalar | ✓ | ✓ | ✓ | ✓ |
| AVX / SIMD | ✓ | ✓ | — | ✓ |

AVX/SIMD availability also depends on the execution environment, including CPU AVX support.

### Vector Width

`BackendCapabilities` exposes approximate processing width in addition to operation support.

| Implementation | `vector_width_f32` | `vector_width_f64` |
|---|---:|---:|
| Scalar | `1` | `1` when `f64` is supported |
| 256-bit AVX | `8` | `4` when `f64` is supported |

## Backend Information API

### `backends`

```rust
pub fn backends(&self) -> Vec<BackendInfo>
```

Returns `BackendInfo` for every backend registered in Core.

```rust
pub struct BackendInfo {
    pub kind: BackendKind,
    pub capabilities: BackendCapabilities,
    pub available: bool,
}
```

`available` indicates whether the backend can be used in the current execution environment.

### `backend_info`

```rust
pub fn backend_info(
    &self,
    kind: BackendKind,
) -> Result<BackendInfo, ComputeError>
```

Returns information about one backend.

If the backend is not registered in Core, the call returns `ComputeError::BackendNotRegistered`.

### Checking Capabilities

```rust
use whitebase_core::{OperationKind, Whitebase};

let core = Whitebase::new();

for backend in core.backends() {
    if backend.available
        && backend.capabilities.supports(OperationKind::SumF64)
    {
        println!("{} supports SumF64", backend.kind.display_name());
    }
}
```

Main fields exposed by `BackendCapabilities`:

| Field | Type | Meaning |
|---|---|---|
| `add_f32` | `bool` | Supports `AddF32` |
| `add_f64` | `bool` | Supports `AddF64` |
| `add_scalar_f64` | `bool` | Supports `AddScalarF64` |
| `sum_f64` | `bool` | Supports `SumF64` |
| `vector_width_f32` | `usize` | Approximate `f32` processing width |
| `vector_width_f64` | `usize` | Approximate `f64` processing width |

`supports(OperationKind)` can also be used to test support by operation.

## Errors

Core computation APIs return `ComputeError`.

| Variant | Condition |
|---|---|
| `LengthMismatch` | Array input/output lengths do not match |
| `BackendUnavailable` | Backend is registered but unavailable in the current environment |
| `OperationUnsupported` | Backend does not support the requested operation |
| `BackendFailure` | Backend-internal or native-adapter execution failed |
| `BackendNotRegistered` | Requested backend is not registered in Core |

### `LengthMismatch`

```rust
ComputeError::LengthMismatch {
    lhs_len,
    rhs_len,
    output_len,
}
```

`add_f32` and `add_f64` require all three arrays to have the same length.

### Selecting a Backend

Core does not automatically fall back from the requested `BackendKind` to another backend.

Callers may therefore check, as needed:

1. Registration using `backend_info()` or `backends()`
2. Operation support using `capabilities.supports(...)`
3. Runtime availability using `available`

before executing an operation.

## Public Types

`whitebase-core` exposes the following types.

| Type | Description |
|---|---|
| `Whitebase` | Unified computation API |
| `BackendInfo` | Backend capabilities and runtime availability |
| `BackendKind` | Backend identifier |
| `BackendCapabilities` | Operation support and processing width |
| `OperationKind` | Operation identifier |
| `ComputeBackend` | Common trait implemented by computation backends |
| `ComputeError` | Core/backend computation error |

The following aliases are also exported.

| Alias | Original type |
|---|---|
| `Backend` | `BackendKind` |
| `Capabilities` | `BackendCapabilities` |
| `Error` | `ComputeError` |

## Core and Runner Responsibilities

Core is the API for performing one computation on a selected backend.

```text
Caller
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Rust / C++ / Assembly
```

For benchmarking and observation use cases, Runner uses Core and adds warmup, repeated measurement, reference-backend comparison, and report generation.

```text
Caller
  ↓
Whitebase Runner
  ↓
Whitebase Core
  ↓
Backend Bridge
  ↓
Rust / C++ / Assembly
```

HTTP benchmark endpoints use Runner. Core itself has no HTTP/JSON transport and is intended to be called directly from Rust.

## Mapping to the HTTP API

The HTTP API and Core API use the same computation stack, but the HTTP API is not a one-to-one transport wrapper around Core.

| Concept | Core API | HTTP API |
|---|---|---|
| `f32` array addition | `Whitebase::add_f32` | Benchmark `add-array` + `precision: "f32"` |
| `f64` array addition | `Whitebase::add_f64` | Benchmark `add-array` + `precision: "f64"` |
| Scalar `f64` addition | `Whitebase::add_scalar_f64` | `POST /api/observations/add-scalar-f64` |
| `f64` array sum | `Whitebase::sum_f64` | Benchmark `sum-f64` + `precision: "f64"` |

The HTTP Benchmark API generates input arrays on the server and uses Runner for warmup, repeated measurements, and cross-backend comparison. Core receives arrays directly from the caller and performs one operation using the selected backend.

See [HTTP API](HTTP-API.md) for HTTP endpoint and request/response details.
