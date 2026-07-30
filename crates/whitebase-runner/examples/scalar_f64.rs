use std::error::Error;

use whitebase_core::BackendKind;
use whitebase_runner::Runner;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
    let backends = [
        BackendKind::RustScalar,
        BackendKind::CppScalar,
        BackendKind::AssemblyScalar,
    ];

    #[cfg(not(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc")))]
    let backends = [BackendKind::RustScalar];

    let runner = Runner::new();
    let literal = 0.3_f64;

    for backend in backends {
        let report = runner.run_add_scalar_f64(backend, 0.1, 0.2)?;

        let backend_label = match backend {
            BackendKind::RustScalar => "Rust Scalar",
            BackendKind::CppScalar => "C++ Scalar",
            BackendKind::AssemblyScalar => "Assembly Scalar",
            _ => unreachable!("scalar_f64 example selected an unexpected backend"),
        };

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
