use whitebase_core::BackendKind;

/// Runnerによる演算実行と計測の設定です。
#[derive(Debug, Clone, PartialEq)]
pub struct RunnerConfig {
    /// 実行対象のバックエンド。
    pub backends: Vec<BackendKind>,

    /// 結果比較の基準にするバックエンド。
    pub reference_backend: BackendKind,

    /// 計測前に実行するウォームアップ回数。
    pub warmup_iterations: usize,

    /// 計測対象として実行する回数。
    pub measured_iterations: usize,

    /// `f32`結果比較に使用する絶対誤差の許容値。
    pub absolute_tolerance: f32,

    /// `f64`結果比較に使用する絶対誤差の許容値。
    pub absolute_tolerance_f64: f64,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            backends: default_backends(),
            reference_backend: BackendKind::RustScalar,
            warmup_iterations: 3,
            measured_iterations: 10,
            absolute_tolerance: 1.0e-6,
            absolute_tolerance_f64: 1.0e-12,
        }
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn default_backends() -> Vec<BackendKind> {
    vec![
        BackendKind::RustScalar,
        BackendKind::RustSimd,
        BackendKind::CppScalar,
        BackendKind::CppAvx,
        BackendKind::AssemblyScalar,
        BackendKind::AssemblyAvx,
        BackendKind::WindowsGnuCppScalar,
        BackendKind::WindowsGnuCppAvx,
        BackendKind::WindowsGnuAssemblyScalar,
        BackendKind::WindowsGnuAssemblyAvx,
    ]
}

#[cfg(not(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc")))]
fn default_backends() -> Vec<BackendKind> {
    vec![
        BackendKind::RustScalar,
        BackendKind::RustSimd,
        BackendKind::CppScalar,
        BackendKind::CppAvx,
        BackendKind::AssemblyScalar,
        BackendKind::AssemblyAvx,
    ]
}
