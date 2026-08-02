//! Whitebaseの統一計算APIを提供します。

#![forbid(unsafe_code)]

use whitebase_backend_bridge::{
    AssemblyAvxBackend, AssemblyScalarBackend, CppAvxBackend, CppScalarBackend, RustScalarBackend,
    RustSimdBackend,
};

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
use whitebase_backend_bridge::{
    WindowsGnuAssemblyAvxBackend, WindowsGnuAssemblyScalarBackend, WindowsGnuCppAvxBackend,
    WindowsGnuCppScalarBackend,
};
pub use whitebase_interface::{
    BackendCapabilities, BackendKind, ComputeBackend, ComputeError, OperationKind,
};

/// 計算バックエンドの状態を表します。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendInfo {
    /// バックエンドの種類。
    pub kind: BackendKind,

    /// バックエンドが提供する機能。
    pub capabilities: BackendCapabilities,

    /// 現在の環境で利用可能かどうか。
    pub available: bool,
}

/// Whitebaseの統一計算インターフェースです。
pub struct Whitebase {
    backends: Vec<Box<dyn ComputeBackend>>,
}

impl Whitebase {
    /// 標準バックエンドを登録したインスタンスを生成します。
    #[must_use]
    pub fn new() -> Self {
        Self {
            backends: standard_backends(),
        }
    }

    /// 登録されているバックエンドの情報を返します。
    #[must_use]
    pub fn backends(&self) -> Vec<BackendInfo> {
        self.backends
            .iter()
            .map(|backend| BackendInfo {
                kind: backend.kind(),
                capabilities: backend.capabilities(),
                available: backend.is_available(),
            })
            .collect()
    }

    /// 指定されたバックエンドの情報を返します。
    pub fn backend_info(&self, kind: BackendKind) -> Result<BackendInfo, ComputeError> {
        let backend = self.find_backend(kind)?;

        Ok(BackendInfo {
            kind: backend.kind(),
            capabilities: backend.capabilities(),
            available: backend.is_available(),
        })
    }

    /// 指定されたバックエンドで`f32`配列を加算します。
    pub fn add_f32(
        &self,
        kind: BackendKind,
        lhs: &[f32],
        rhs: &[f32],
        output: &mut [f32],
    ) -> Result<(), ComputeError> {
        let backend = self.find_backend(kind)?;

        if !backend.capabilities().supports(OperationKind::AddF32) {
            return Err(ComputeError::OperationUnsupported {
                backend: kind,
                operation: OperationKind::AddF32,
            });
        }

        if !backend.is_available() {
            return Err(ComputeError::BackendUnavailable { backend: kind });
        }

        backend.add_f32(lhs, rhs, output)
    }

    /// 指定されたバックエンドで`f64`配列を加算します。
    pub fn add_f64(
        &self,
        kind: BackendKind,
        lhs: &[f64],
        rhs: &[f64],
        output: &mut [f64],
    ) -> Result<(), ComputeError> {
        let backend = self.find_backend(kind)?;

        if !backend.capabilities().supports(OperationKind::AddF64) {
            return Err(ComputeError::OperationUnsupported {
                backend: kind,
                operation: OperationKind::AddF64,
            });
        }

        if !backend.is_available() {
            return Err(ComputeError::BackendUnavailable { backend: kind });
        }

        backend.add_f64(lhs, rhs, output)
    }

    /// 指定されたバックエンドで`f64`スカラー値を加算します。
    pub fn add_scalar_f64(
        &self,
        kind: BackendKind,
        lhs: f64,
        rhs: f64,
    ) -> Result<f64, ComputeError> {
        let backend = self.find_backend(kind)?;

        if !backend.capabilities().supports(OperationKind::AddScalarF64) {
            return Err(ComputeError::OperationUnsupported {
                backend: kind,
                operation: OperationKind::AddScalarF64,
            });
        }
        if !backend.is_available() {
            return Err(ComputeError::BackendUnavailable { backend: kind });
        }

        backend.add_scalar_f64(lhs, rhs)
    }

    fn find_backend(&self, kind: BackendKind) -> Result<&dyn ComputeBackend, ComputeError> {
        self.backends
            .iter()
            .find(|backend| backend.kind() == kind)
            .map(Box::as_ref)
            .ok_or(ComputeError::BackendNotRegistered { backend: kind })
    }
}

#[cfg(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc"))]
fn standard_backends() -> Vec<Box<dyn ComputeBackend>> {
    vec![
        Box::new(RustScalarBackend),
        Box::new(RustSimdBackend),
        Box::new(CppScalarBackend),
        Box::new(CppAvxBackend),
        Box::new(AssemblyScalarBackend),
        Box::new(AssemblyAvxBackend),
        Box::new(WindowsGnuCppScalarBackend),
        Box::new(WindowsGnuCppAvxBackend),
        Box::new(WindowsGnuAssemblyScalarBackend),
        Box::new(WindowsGnuAssemblyAvxBackend),
    ]
}

#[cfg(not(all(target_arch = "x86_64", target_os = "windows", target_env = "msvc")))]
fn standard_backends() -> Vec<Box<dyn ComputeBackend>> {
    vec![
        Box::new(RustScalarBackend),
        Box::new(RustSimdBackend),
        Box::new(CppScalarBackend),
        Box::new(CppAvxBackend),
        Box::new(AssemblyScalarBackend),
        Box::new(AssemblyAvxBackend),
    ]
}

impl Default for Whitebase {
    fn default() -> Self {
        Self::new()
    }
}

pub use whitebase_interface::{
    BackendCapabilities as Capabilities, BackendKind as Backend, ComputeError as Error,
};
