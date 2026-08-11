/// バックエンドを実行しているCPU / VMアーキテクチャです。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionArchitecture {
    X86_64,
    Aarch64,
    Wasm32,
    Other,
}

impl ExecutionArchitecture {
    #[must_use]
    pub const fn current() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::X86_64
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self::Aarch64
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::Wasm32
        }

        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "wasm32"
        )))]
        {
            Self::Other
        }
    }

    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "AArch64",
            Self::Wasm32 => "WASM32",
            Self::Other => "Other",
        }
    }
}

/// バックエンドが実際の計算に使用する実行方式です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Scalar,
    Avx,
    Neon,
    WasmSimd128,
}

impl ExecutionMode {
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Scalar => "Scalar",
            Self::Avx => "AVX",
            Self::Neon => "NEON",
            Self::WasmSimd128 => "SIMD128",
        }
    }
}

/// 現在の実行環境でバックエンドが利用する計算経路です。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendExecutionProfile {
    pub architecture: ExecutionArchitecture,
    pub mode: ExecutionMode,
    pub vector_bits: Option<usize>,
}

impl BackendExecutionProfile {
    #[must_use]
    pub const fn new(
        architecture: ExecutionArchitecture,
        mode: ExecutionMode,
        vector_bits: Option<usize>,
    ) -> Self {
        Self {
            architecture,
            mode,
            vector_bits,
        }
    }
}
