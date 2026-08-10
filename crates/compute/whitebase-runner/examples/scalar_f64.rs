use std::error::Error;

use whitebase_core::{BackendKind, ComputeError};
use whitebase_runner::{Runner, RunnerError};

fn main() -> Result<(), Box<dyn Error>> {
    let backends = scalar_backends();

    let runner = Runner::new();
    let literal = 0.3_f64;

    for backend in backends {
        let report = match runner.run_add_scalar_f64(backend, 0.1, 0.2) {
            Ok(report) => report,
            Err(RunnerError::Compute {
                error: ComputeError::BackendUnavailable { .. },
            }) => continue,
            Err(error) => return Err(Box::new(error)),
        };

        let backend_label = backend.display_name();

        println!("backend:  {backend_label}");
        println!("lhs:      {:.17}", report.lhs.value);
        println!("rhs:      {:.17}", report.rhs.value);
        println!("result:   {:.17}", report.result.value);
        println!("literal:  {:.17}", literal);
        println!("bits:     0x{:016x}", report.result.bits);
        println!();
    }

    Ok(())
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn scalar_backends() -> [BackendKind; 5] {
    [
        BackendKind::RustScalar,
        BackendKind::CppScalar,
        BackendKind::AssemblyScalar,
        BackendKind::WindowsGnuCppScalar,
        BackendKind::WindowsGnuAssemblyScalar,
    ]
}

#[cfg(all(target_arch = "x86_64", target_os = "linux", target_env = "gnu"))]
fn scalar_backends() -> [BackendKind; 3] {
    [
        BackendKind::RustScalar,
        BackendKind::CppScalar,
        BackendKind::AssemblyScalar,
    ]
}

#[cfg(not(any(
    all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"),
    all(target_arch = "x86_64", target_os = "linux", target_env = "gnu")
)))]
fn scalar_backends() -> [BackendKind; 1] {
    [BackendKind::RustScalar]
}
