use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use eframe::egui;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Control Center must be inside tools")
        .parent()
        .expect("tools must be inside the repository")
        .to_path_buf()
}

fn strip_ansi_escape_sequences(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if character != '\u{1b}' {
            output.push(character);
            continue;
        }

        if !matches!(chars.peek(), Some(&'[')) {
            continue;
        }

        chars.next();

        for code_character in chars.by_ref() {
            if ('@'..='~').contains(&code_character) {
                break;
            }
        }
    }
    output
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([720.0, 480.0])
            .with_min_inner_size([520.0, 360.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Whitebase Control Center",
        options,
        Box::new(|creation_context| {
            egui_system_fonts::add_with_region(
                &creation_context.egui_ctx,
                egui_system_fonts::FontRegion::Japanese,
                egui_system_fonts::FontStyle::Sans,
            );

            Ok(Box::new(ControlCenterApp::default()))
        }),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Task {
    CheckControlCenter,
    CheckFormat,
    CheckClippy,
    CheckWorkspace,
    CheckWasm,
    CheckLinuxNative,
    CheckWindowsGnuNative,
    CheckCppClient,
    CheckCppBackend,
    CheckCppAdapter,
    CheckAssembly,
    BuildWorkspace,
    BuildWorkspaceRelease,
    InstallFrontendDependencies,
    BuildFrontend,
    BuildWasm,
    BuildWasmRelease,
    BuildLinuxNative,
    BuildLinuxNativeRelease,
    BuildWindowsGnuNative,
    BuildWindowsGnuNativeRelease,
    BuildWindowsNativeRelease,
    BuildCApi,
    BuildCApiRelease,
    BuildCppClient,
    BuildAssemblyClient,
    BuildTauriRelease,
    BuildControlCenterRelease,
    TestWorkspace,
    RunServer,
}

const CHECK_ALL_TASKS: &[Task] = &[
    Task::CheckFormat,
    // LinuxではRustのAdapterを検査する前に、GCC/NASMのライブラリを準備する。
    // 非対応OSではTaskSequence::check_all()が除外する。
    Task::CheckLinuxNative,
    // Windows GNU NativeのGCC/NASM Scalarバックエンドを検査する。
    Task::CheckWindowsGnuNative,
    // RustのAdapterを検査する前に、Windowsのネイティブライブラリを準備する。
    // 非対応OSではTaskSequence::check_all()がこれらを除外する。
    Task::CheckCppBackend,
    Task::CheckAssembly,
    Task::CheckClippy,
    Task::TestWorkspace,
    Task::CheckWasm,
    // Frontend buildが参照するTypeScript/Viteをlockfileどおりに準備する。
    Task::InstallFrontendDependencies,
    Task::BuildFrontend,
    Task::CheckCppClient,
    Task::CheckCppAdapter,
];

const BUILD_ALL_TASKS: &[Task] = &[
    // LinuxではWorkspace内のAdapterをリンクする前にネイティブ成果物を準備する。
    Task::BuildLinuxNative,
    // Build欄の個別タスクをすべて順番に実行する。
    // WindowsではWorkspace内のAdapterをリンクする前にネイティブ成果物を準備する。
    Task::BuildCApi,
    Task::BuildCppClient,
    Task::BuildAssemblyClient,
    Task::BuildWindowsGnuNative,
    Task::BuildWorkspace,
    Task::BuildWasm,
    // Frontend buildが参照するTypeScript/Viteをlockfileどおりに準備する。
    Task::InstallFrontendDependencies,
    // Wasm生成物を含んだ状態でFrontendを組み立てる。
    Task::BuildFrontend,
    Task::BuildControlCenterRelease,
];

const RELEASE_ALL_TASKS: &[Task] = &[
    // C API自身が必要なMSVC/MASM Releaseライブラリを先に準備してからRust側をビルドする。
    // その後のWindows Native Release全体ビルドではC++ ClientがC APIのimport libraryを参照できる。
    // Linuxでは非対応タスクとしてTaskSequence::release_all()が除外する。
    Task::BuildCApiRelease,
    Task::BuildWindowsNativeRelease,
    // Windows GNU NativeのGCC/NASM成果物をRelease構成で準備する。
    Task::BuildWindowsGnuNativeRelease,
    // LinuxではRustのRelease成果物をリンクする前にGCC/NASMを準備する。
    // Windowsでは非対応タスクとして除外する。
    Task::BuildLinuxNativeRelease,
    // 実行中のControl Center本体を除いたWorkspaceをReleaseでビルドする。
    Task::BuildWorkspaceRelease,
    // TauriのbeforeBuildCommandが参照するWasm成果物をReleaseで生成する。
    Task::BuildWasmRelease,
    // Tauri CLI/TypeScript/Viteをlockfileどおりに準備する。
    Task::InstallFrontendDependencies,
    // Tauri ReleaseはFrontendのProductionビルドとBundle生成を含む。
    Task::BuildTauriRelease,
    // 実行中のControl CenterがDebug版など、安全に上書きできる場合だけ実行する。
    Task::BuildControlCenterRelease,
];

#[derive(Clone, Copy)]
struct CommandSpec {
    program: &'static str,
    args: &'static [&'static str],
}

impl CommandSpec {
    fn into_command(self, working_directory: &Path) -> Command {
        let mut command = Command::new(self.program);
        command.args(self.args);
        command.current_dir(working_directory);
        command
    }

    fn display(self) -> String {
        std::iter::once(self.program)
            .chain(self.args.iter().copied())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Task {
    fn label(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Check Control Center",
            Self::CheckFormat => "Check Format",
            Self::CheckClippy => "Check Clippy",
            Self::CheckWorkspace => "Check Workspace",
            Self::CheckWasm => "Check Wasm",
            Self::CheckLinuxNative => "Check Linux Native",
            Self::CheckWindowsGnuNative => "Check Windows GNU Native",
            Self::CheckCppClient => "Check C++ Client",
            Self::CheckCppBackend => "Check C++ Backend",
            Self::CheckCppAdapter => "Check C++ Adapter",
            Self::CheckAssembly => "Check Assembly",
            Self::BuildWorkspace => "Build Workspace",
            Self::BuildWorkspaceRelease => "Build Workspace Release",
            Self::InstallFrontendDependencies => "Install Frontend Dependencies",
            Self::BuildFrontend => "Build Frontend",
            Self::BuildWasm => "Build Wasm",
            Self::BuildWasmRelease => "Build Wasm Release",
            Self::BuildLinuxNative => "Build Linux Native",
            Self::BuildLinuxNativeRelease => "Build Linux Native Release",
            Self::BuildWindowsGnuNative => "Build Windows GNU Native",
            Self::BuildWindowsGnuNativeRelease => "Build Windows GNU Native Release",
            Self::BuildWindowsNativeRelease => "Build Windows Native Release",
            Self::BuildCApi => "Build C API",
            Self::BuildCApiRelease => "Build C API Release",
            Self::BuildCppClient => "Build C++ Client",
            Self::BuildAssemblyClient => "Build Assembly Client",
            Self::BuildTauriRelease => "Build Tauri Release",
            Self::BuildControlCenterRelease => "Build Control Center Release",
            Self::TestWorkspace => "Test Workspace",
            Self::RunServer => "Run Server",
        }
    }

    fn running_message(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Checking Control Center...",
            Self::CheckFormat => "Checking Format...",
            Self::CheckClippy => "Checking with Clippy...",
            Self::CheckWorkspace => "Checking Workspace...",
            Self::CheckWasm => "Checking Wasm...",
            Self::CheckLinuxNative => "Checking Linux Native...",
            Self::CheckWindowsGnuNative => "Checking Windows GNU Native...",
            Self::CheckCppClient => "Checking C++ Client...",
            Self::CheckCppBackend => "Checking C++ Backend...",
            Self::CheckCppAdapter => "Checking C++ Adapter...",
            Self::CheckAssembly => "Checking Assembly...",
            Self::BuildWorkspace => "Building Workspace...",
            Self::BuildWorkspaceRelease => "Building Workspace Release...",
            Self::InstallFrontendDependencies => "Installing Frontend Dependencies...",
            Self::BuildFrontend => "Building Frontend...",
            Self::BuildWasm => "Building Wasm...",
            Self::BuildWasmRelease => "Building Wasm Release...",
            Self::BuildLinuxNative => "Building Linux Native...",
            Self::BuildLinuxNativeRelease => "Building Linux Native Release...",
            Self::BuildWindowsGnuNative => "Building Windows GNU Native...",
            Self::BuildWindowsGnuNativeRelease => "Building Windows GNU Native Release...",
            Self::BuildWindowsNativeRelease => "Building Windows Native Release...",
            Self::BuildCApi => "Building C API...",
            Self::BuildCApiRelease => "Building C API Release...",
            Self::BuildCppClient => "Building C++ Client...",
            Self::BuildAssemblyClient => "Building Assembly Client...",
            Self::BuildTauriRelease => "Building Tauri Release...",
            Self::BuildControlCenterRelease => "Building Control Center Release...",
            Self::TestWorkspace => "Testing Workspace...",
            Self::RunServer => "Running Whitebase Server...",
        }
    }

    fn success_message(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Control Center check completed successfully",
            Self::CheckFormat => "Format check completed successfully",
            Self::CheckClippy => "Clippy check completed successfully",
            Self::CheckWorkspace => "Workspace check completed successfully",
            Self::CheckWasm => "Wasm check completed successfully",
            Self::CheckLinuxNative => "Linux Native check completed successfully",
            Self::CheckWindowsGnuNative => "Windows GNU Native check completed successfully",
            Self::CheckCppClient => "C++ Client check completed successfully",
            Self::CheckCppBackend => "C++ Backend check completed successfully",
            Self::CheckCppAdapter => "C++ Adapter check completed successfully",
            Self::CheckAssembly => "Assembly check completed successfully",
            Self::BuildWorkspace => "Workspace build completed successfully",
            Self::BuildWorkspaceRelease => "Workspace Release build completed successfully",
            Self::InstallFrontendDependencies => "Frontend dependencies installed successfully",
            Self::BuildFrontend => "Frontend build completed successfully",
            Self::BuildWasm => "Wasm build completed successfully",
            Self::BuildWasmRelease => "Wasm Release build completed successfully",
            Self::BuildLinuxNative => "Linux Native build completed successfully",
            Self::BuildLinuxNativeRelease => "Linux Native Release build completed successfully",
            Self::BuildWindowsGnuNative => "Windows GNU Native build completed successfully",
            Self::BuildWindowsGnuNativeRelease => {
                "Windows GNU Native Release build completed successfully"
            }
            Self::BuildWindowsNativeRelease => {
                "Windows Native Release build completed successfully"
            }
            Self::BuildCApi => "C API build completed successfully",
            Self::BuildCApiRelease => "C API Release build completed successfully",
            Self::BuildCppClient => "C++ Client build completed successfully",
            Self::BuildAssemblyClient => "Assembly Client build completed successfully",
            Self::BuildTauriRelease => "Tauri Release build completed successfully",
            Self::BuildControlCenterRelease => {
                "Control Center Release build completed successfully"
            }
            Self::TestWorkspace => "Workspace tests completed successfully",
            Self::RunServer => "Whitebase Server exited successfully",
        }
    }

    fn command_spec(self) -> CommandSpec {
        match self {
            Self::CheckControlCenter => CommandSpec {
                program: "cargo",
                args: &["check", "-p", "whitebase-control-center"],
            },
            Self::CheckFormat => CommandSpec {
                program: "cargo",
                args: &["fmt", "--all", "--", "--check"],
            },
            Self::CheckClippy => CommandSpec {
                program: "cargo",
                args: &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ],
            },
            Self::CheckWorkspace => CommandSpec {
                program: "cargo",
                args: &["check", "--workspace"],
            },
            Self::CheckWasm => CommandSpec {
                program: "cargo",
                args: &[
                    "check",
                    "-p",
                    "whitebase-wasm",
                    "--target",
                    "wasm32-unknown-unknown",
                ],
            },
            Self::CheckLinuxNative => CommandSpec {
                program: "bash",
                args: &["scripts/linux-native.sh", "check"],
            },
            Self::CheckWindowsGnuNative => CommandSpec {
                program: "powershell.exe",
                args: &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "scripts\\windows-gnu-native.ps1",
                    "check",
                ],
            },
            Self::CheckCppClient => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "cpp-check"],
            },
            Self::CheckCppBackend => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "cpp-backend-check"],
            },
            Self::CheckAssembly => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "asm-check"],
            },
            Self::BuildWorkspace => CommandSpec {
                program: "cargo",
                // 実行中のControl Center本体はWindowsがロックするため、
                // Workspaceの他パッケージを先にビルドする。
                // Control Centerは専用のReleaseタスクで別にビルドする。
                args: &[
                    "build",
                    "--workspace",
                    "--exclude",
                    "whitebase-control-center",
                ],
            },
            Self::BuildWorkspaceRelease => CommandSpec {
                program: "cargo",
                args: &[
                    "build",
                    "--workspace",
                    "--release",
                    "--locked",
                    "--exclude",
                    "whitebase-control-center",
                ],
            },
            Self::InstallFrontendDependencies => CommandSpec {
                program: if cfg!(windows) { "npm.cmd" } else { "npm" },
                args: &[
                    "--prefix",
                    "apps/whitebase-app",
                    "ci",
                    "--prefer-offline",
                    "--no-audit",
                    "--no-fund",
                ],
            },
            Self::BuildFrontend => CommandSpec {
                program: if cfg!(windows) { "npm.cmd" } else { "npm" },
                args: &["--prefix", "apps/whitebase-app", "run", "build"],
            },
            Self::BuildWasm => CommandSpec {
                program: "wasm-pack",
                args: &[
                    "build",
                    "--target",
                    "web",
                    "--dev",
                    "--out-dir",
                    "../../apps/whitebase-app/src/wasm",
                ],
            },
            Self::BuildWasmRelease => CommandSpec {
                program: "wasm-pack",
                args: &[
                    "build",
                    "--target",
                    "web",
                    "--release",
                    "--out-dir",
                    "../../apps/whitebase-app/src/wasm",
                ],
            },
            Self::BuildLinuxNative => CommandSpec {
                program: "bash",
                args: &["scripts/linux-native.sh", "build"],
            },
            Self::BuildLinuxNativeRelease => CommandSpec {
                program: "bash",
                args: &["scripts/linux-native.sh", "release"],
            },
            Self::BuildWindowsGnuNative => CommandSpec {
                program: "powershell.exe",
                args: &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "scripts\\windows-gnu-native.ps1",
                    "build",
                ],
            },
            Self::BuildWindowsGnuNativeRelease => CommandSpec {
                program: "powershell.exe",
                args: &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                    "scripts\\windows-gnu-native.ps1",
                    "release",
                ],
            },
            Self::BuildWindowsNativeRelease => CommandSpec {
                program: "powershell.exe",
                args: &[
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    r#"$ErrorActionPreference = 'Stop'; $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'; if (-not (Test-Path $vswhere)) { throw "vswhere.exe was not found: $vswhere" }; $installationPath = & $vswhere -latest -products * -requires Microsoft.Component.MSBuild -property installationPath; if (-not $installationPath) { throw 'Visual Studio with MSBuild was not found.' }; $msbuild = Join-Path $installationPath 'MSBuild\Current\Bin\MSBuild.exe'; & $msbuild 'native\Whitebase.Cpp\Whitebase.Cpp.slnx' /t:Build /m /p:Configuration=Release /p:Platform=x64 /v:minimal; if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }"#,
                ],
            },
            Self::BuildCApi => {
                if cfg!(windows) {
                    CommandSpec {
                        program: "cmd.exe",
                        args: &["/C", "scripts\\ops.bat", "c-api-build"],
                    }
                } else {
                    CommandSpec {
                        program: "cargo",
                        args: &["build", "-p", "whitebase-c-api"],
                    }
                }
            }
            Self::BuildCApiRelease => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "c-api-release-build"],
            },
            Self::BuildCppClient => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "cpp-build"],
            },
            Self::BuildAssemblyClient => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "asm-build"],
            },
            Self::CheckCppAdapter => CommandSpec {
                program: "cmd.exe",
                args: &["/C", "scripts\\ops.bat", "cpp-adapter-check"],
            },
            Self::BuildTauriRelease => CommandSpec {
                program: if cfg!(windows) { "npm.cmd" } else { "npm" },
                args: &[
                    "--prefix",
                    "apps/whitebase-app",
                    "run",
                    "tauri",
                    "--",
                    "build",
                ],
            },
            Self::BuildControlCenterRelease => CommandSpec {
                program: "cargo",
                args: &[
                    "build",
                    "--release",
                    "--locked",
                    "-p",
                    "whitebase-control-center",
                ],
            },
            Self::TestWorkspace => CommandSpec {
                program: "cargo",
                args: &["test", "--workspace"],
            },
            Self::RunServer => CommandSpec {
                program: "cargo",
                args: &["run", "-p", "whitebase-server"],
            },
        }
    }

    fn ready_message(self, line: &str) -> Option<&'static str> {
        match self {
            Self::RunServer if line.contains("[Whitebase Server] Listening on ") => {
                Some("Whitebase Server is ready")
            }
            _ => None,
        }
    }

    fn working_directory(self) -> PathBuf {
        match self {
            Self::BuildWasm | Self::BuildWasmRelease => {
                repository_root().join("crates").join("whitebase-wasm")
            }

            _ => repository_root(),
        }
    }

    fn is_supported(self) -> bool {
        if matches!(
            self,
            Self::CheckLinuxNative | Self::BuildLinuxNative | Self::BuildLinuxNativeRelease
        ) {
            return cfg!(all(target_os = "linux", target_arch = "x86_64"));
        }

        if matches!(
            self,
            Self::CheckWindowsGnuNative
                | Self::BuildWindowsGnuNative
                | Self::BuildWindowsGnuNativeRelease
        ) {
            return cfg!(all(target_os = "windows", target_arch = "x86_64"));
        }

        if matches!(
            self,
            Self::CheckCppClient
                | Self::CheckCppAdapter
                | Self::CheckCppBackend
                | Self::CheckAssembly
                | Self::BuildCppClient
                | Self::BuildAssemblyClient
                | Self::BuildWindowsNativeRelease
                | Self::BuildCApiRelease
        ) {
            return cfg!(target_os = "windows");
        }

        if matches!(self, Self::BuildControlCenterRelease) {
            // Windowsでは実行中のexeを上書きできない。
            // Debug版からRelease版を作る場合だけ安全に実行できる。
            return !cfg!(target_os = "windows") || cfg!(debug_assertions);
        }

        true
    }
}

struct TaskSequence {
    label: &'static str,
    success_message: &'static str,
    pending_tasks: VecDeque<Task>,
}

impl TaskSequence {
    fn from_supported_tasks(
        label: &'static str,
        success_message: &'static str,
        tasks: &[Task],
    ) -> Self {
        let pending_tasks = tasks
            .iter()
            .copied()
            .filter(|task| task.is_supported())
            .collect();

        Self {
            label,
            success_message,
            pending_tasks,
        }
    }

    fn check_all() -> Self {
        Self::from_supported_tasks(
            "Check All",
            "Whitebase check completed successfully",
            CHECK_ALL_TASKS,
        )
    }

    fn build_all() -> Self {
        Self::from_supported_tasks(
            "Build All",
            "Whitebase build completed successfully",
            BUILD_ALL_TASKS,
        )
    }

    fn release_all() -> Self {
        Self::from_supported_tasks(
            "Release All",
            "Whitebase Release build completed successfully",
            RELEASE_ALL_TASKS,
        )
    }
}

enum WorkerEvent {
    Log(String),
    Ready { message: String },
    Finished { success: bool, message: String },
    Stopped { message: String },
}

struct ControlCenterApp {
    status: String,
    log: String,
    active_task: Option<Task>,
    active_sequence: Option<TaskSequence>,
    event_receiver: Option<Receiver<WorkerEvent>>,
    stop_sender: Option<mpsc::Sender<()>>,
}

impl Default for ControlCenterApp {
    fn default() -> Self {
        Self {
            status: "Idle".to_owned(),
            log: "No output yet.".to_owned(),
            active_task: None,
            active_sequence: None,
            event_receiver: None,
            stop_sender: None,
        }
    }
}

impl eframe::App for ControlCenterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_events();

        let is_running = self.active_task.is_some() || self.active_sequence.is_some();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Whitebase Control Center");
            ui.separator();

            ui.label(format!("Version: {}", env!("CARGO_PKG_VERSION")));

            let build_profile = if cfg!(debug_assertions) {
                "Debug"
            } else {
                "Release"
            };

            ui.label(format!("Build: {build_profile}"));

            ui.label(format!(
                "Platform: {} / {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ));

            ui.add_space(12.0);

            ui.label(egui::RichText::new("Checks").strong());

            ui.horizontal_wrapped(|ui| {
                let check_all_button = egui::Button::new("Check All");

                if ui.add_enabled(!is_running, check_all_button).clicked() {
                    self.start_check_all();
                }
                for task in [
                    Task::CheckControlCenter,
                    Task::CheckFormat,
                    Task::CheckLinuxNative,
                    Task::CheckWindowsGnuNative,
                    Task::CheckClippy,
                    Task::CheckWorkspace,
                    Task::CheckWasm,
                    Task::CheckCppClient,
                    Task::CheckCppBackend,
                    Task::CheckCppAdapter,
                    Task::CheckAssembly,
                    Task::TestWorkspace,
                ] {
                    let button = egui::Button::new(task.label());

                    let is_enabled = !is_running && task.is_supported();

                    if ui.add_enabled(is_enabled, button).clicked() {
                        self.start_task(task);
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Build").strong());

            ui.horizontal_wrapped(|ui| {
                let build_all_button = egui::Button::new("Build All");

                if ui.add_enabled(!is_running, build_all_button).clicked() {
                    self.start_build_all();
                }

                for task in [
                    Task::BuildLinuxNative,
                    Task::BuildWindowsGnuNative,
                    Task::BuildWorkspace,
                    Task::BuildFrontend,
                    Task::BuildWasm,
                    Task::BuildCApi,
                    Task::BuildCppClient,
                    Task::BuildAssemblyClient,
                ] {
                    let button = egui::Button::new(task.label());

                    let is_enabled = !is_running && task.is_supported();

                    if ui.add_enabled(is_enabled, button).clicked() {
                        self.start_task(task);
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Release").strong());

            ui.horizontal_wrapped(|ui| {
                let release_all_button = egui::Button::new("Release All");

                if ui.add_enabled(!is_running, release_all_button).clicked() {
                    self.start_release_all();
                }

                for task in [
                    Task::BuildCApiRelease,
                    Task::BuildWindowsNativeRelease,
                    Task::BuildWindowsGnuNativeRelease,
                    Task::BuildLinuxNativeRelease,
                    Task::BuildWorkspaceRelease,
                    Task::BuildWasmRelease,
                    Task::BuildTauriRelease,
                    Task::BuildControlCenterRelease,
                ] {
                    let button = egui::Button::new(task.label());
                    let is_enabled = !is_running && task.is_supported();

                    if ui.add_enabled(is_enabled, button).clicked() {
                        self.start_task(task);
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Run").strong());

            ui.horizontal_wrapped(|ui| {
                let task = Task::RunServer;
                let button = egui::Button::new(task.label());

                if ui.add_enabled(!is_running, button).clicked() {
                    self.start_task(task);
                }
            });

            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Status: {}", self.status));

                if let Some(task) = self.active_task {
                    ui.label(format!("Task: {}", task.label()));
                }

                if is_running {
                    ui.spinner();
                }

                let stop_button = egui::Button::new("Stop");

                if ui
                    .add_enabled(is_running && self.stop_sender.is_some(), stop_button)
                    .clicked()
                {
                    self.request_stop();
                }
            });

            if is_running {
                // ワーカースレッドから届いたログを定期的に受け取るため、
                // 実行中は画面を再描画する。
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            }

            ui.add_space(12.0);
            ui.separator();
            ui.heading("Log");

            ui.horizontal(|ui| {
                if ui.button("Copy Log").clicked() {
                    ui.ctx().copy_text(self.log.clone());
                }

                if ui.button("Clear Log").clicked() {
                    self.log.clear();
                }
            });

            let log_height = ui.available_height();

            egui::ScrollArea::both()
                .id_salt("control-center-log")
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .max_height(log_height)
                .show(ui, |ui| {
                    ui.add(
                        egui::Label::new(
                            egui::RichText::new(&self.log).text_style(egui::TextStyle::Monospace),
                        )
                        .selectable(true)
                        .extend(),
                    );
                });
        });
    }
}

impl ControlCenterApp {
    fn start_task(&mut self, task: Task) {
        self.active_sequence = None;
        self.start_task_internal(task, true);
    }

    fn start_check_all(&mut self) {
        self.start_sequence(TaskSequence::check_all());
    }

    fn start_build_all(&mut self) {
        self.start_sequence(TaskSequence::build_all());
    }

    fn start_release_all(&mut self) {
        self.start_sequence(TaskSequence::release_all());
    }

    fn start_sequence(&mut self, mut sequence: TaskSequence) {
        let next_task = sequence.pending_tasks.pop_front();

        self.status = format!("Running {}...", sequence.label);
        self.log = format!("Running sequence: {}\n", sequence.label);
        self.active_sequence = Some(sequence);

        if let Some(task) = next_task {
            self.start_task_internal(task, false);
        } else if let Some(sequence) = self.active_sequence.take() {
            self.status = sequence.success_message.to_owned();
        }
    }

    fn start_task_internal(&mut self, task: Task, replace_log: bool) {
        let (sender, receiver) = mpsc::channel();
        let (stop_sender, stop_receiver) = mpsc::channel();

        let command_spec = task.command_spec();
        let working_directory = task.working_directory();

        self.status = task.running_message().to_owned();

        let task_header = format!(
            "Running task: {}\nCommand: {}\nWorking directory: {}\n",
            task.label(),
            command_spec.display(),
            working_directory.display()
        );

        if replace_log {
            self.log = task_header;
        } else {
            if !self.log.is_empty() && !self.log.ends_with('\n') {
                self.log.push('\n');
            }

            if !self.log.is_empty() {
                self.log.push('\n');
            }

            self.log.push_str(&task_header);
        }

        self.active_task = Some(task);
        self.event_receiver = Some(receiver);
        self.stop_sender = Some(stop_sender);

        thread::spawn(move || {
            let started_at = Instant::now();
            let mut command = command_spec.into_command(&working_directory);

            let mut child = match command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Finished {
                        success: false,
                        message: format!("Failed to start command: {error}"),
                    });
                    return;
                }
            };

            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let stdout_thread = stdout.map(|stdout| {
                let sender = sender.clone();

                thread::spawn(move || {
                    let reader = BufReader::new(stdout);

                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                let ready_message = task.ready_message(&line);

                                if sender.send(WorkerEvent::Log(line)).is_err() {
                                    break;
                                }

                                if let Some(message) = ready_message
                                    && sender
                                        .send(WorkerEvent::Ready {
                                            message: message.to_owned(),
                                        })
                                        .is_err()
                                {
                                    break;
                                }
                            }
                            Err(error) => {
                                let _ = sender.send(WorkerEvent::Log(format!(
                                    "Failed to read stdout: {error}"
                                )));
                                break;
                            }
                        }
                    }
                })
            });

            let stderr_thread = stderr.map(|stderr| {
                let sender = sender.clone();

                thread::spawn(move || {
                    let reader = BufReader::new(stderr);

                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                if sender.send(WorkerEvent::Log(line)).is_err() {
                                    break;
                                }
                            }
                            Err(error) => {
                                let _ = sender.send(WorkerEvent::Log(format!(
                                    "Failed to read stderr: {error}"
                                )));
                                break;
                            }
                        }
                    }
                })
            });

            let exit_status = loop {
                match stop_receiver.try_recv() {
                    Ok(()) => {
                        let kill_result = child.kill();
                        let wait_result = child.wait();
                        let elapsed = started_at.elapsed();

                        let message = match (kill_result, wait_result) {
                            (Ok(()), Ok(_)) => {
                                format!("{} stopped ({elapsed:.2?})", task.label())
                            }
                            (Err(error), _) => {
                                format!(
                                    "Failed to stop {}: {} ({elapsed:.2?})",
                                    task.label(),
                                    error
                                )
                            }
                            (Ok(()), Err(error)) => {
                                format!(
                                    "{} stopped, but waiting failed: {} ({elapsed:.2?})",
                                    task.label(),
                                    error
                                )
                            }
                        };

                        let _ = sender.send(WorkerEvent::Stopped { message });
                        return;
                    }

                    Err(TryRecvError::Disconnected) => {
                        let _ = child.kill();
                        let _ = child.wait();
                        return;
                    }

                    Err(TryRecvError::Empty) => {}
                }

                match child.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Ok(None) => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(error) => break Err(error),
                }
            };

            if let Some(thread) = stdout_thread {
                let _ = thread.join();
            }

            if let Some(thread) = stderr_thread {
                let _ = thread.join();
            }

            let elapsed = started_at.elapsed();

            let event = match exit_status {
                Ok(status) if status.success() => WorkerEvent::Finished {
                    success: true,
                    message: format!("{} ({elapsed:.2?})", task.success_message()),
                },

                Ok(status) => WorkerEvent::Finished {
                    success: false,
                    message: format!("{} failed with {} ({elapsed:.2?})", task.label(), status),
                },

                Err(error) => WorkerEvent::Finished {
                    success: false,
                    message: format!(
                        "Failed while waiting for command: {} ({elapsed:.2?})",
                        error
                    ),
                },
            };

            let _ = sender.send(event);
        });
    }

    fn request_stop(&mut self) {
        let Some(sender) = self.stop_sender.take() else {
            return;
        };

        self.status = "Stopping...".to_owned();

        if sender.send(()).is_err() {
            self.status = "Failed to send stop request".to_owned();
        }
    }

    fn receive_events(&mut self) {
        let mut clear_receiver = false;
        let mut clear_stop_sender = false;
        let mut next_sequence_task = None;
        let mut completed_sequence_message = None;

        if let Some(receiver) = self.event_receiver.as_ref() {
            loop {
                match receiver.try_recv() {
                    Ok(WorkerEvent::Log(line)) => {
                        let clean_line = strip_ansi_escape_sequences(&line);

                        self.log.push_str(&clean_line);
                        self.log.push('\n');
                    }
                    Ok(WorkerEvent::Ready { message }) => {
                        self.status = message;
                    }

                    Ok(WorkerEvent::Finished { success, message }) => {
                        self.active_task = None;
                        clear_receiver = true;
                        clear_stop_sender = true;

                        if success {
                            if let Some(sequence) = self.active_sequence.as_mut() {
                                self.log.push('\n');
                                self.log.push_str(&message);
                                self.log.push('\n');

                                match sequence.pending_tasks.pop_front() {
                                    Some(task) => next_sequence_task = Some(task),
                                    None => {
                                        completed_sequence_message =
                                            Some(sequence.success_message.to_owned());
                                    }
                                }
                            } else {
                                self.status = message;
                            }
                        } else {
                            self.status = message;
                            self.active_sequence = None;
                            self.log.push_str("\nProcess finished with an error.\n");
                        }

                        break;
                    }

                    Ok(WorkerEvent::Stopped { message }) => {
                        self.status = message;
                        self.active_task = None;
                        self.active_sequence = None;
                        clear_receiver = true;
                        clear_stop_sender = true;
                        self.log.push_str("\nTask stopped by user.\n");
                        break;
                    }

                    Err(TryRecvError::Empty) => break,

                    Err(TryRecvError::Disconnected) => {
                        self.status = "Worker disconnected unexpectedly".to_owned();
                        self.active_task = None;
                        self.active_sequence = None;
                        clear_receiver = true;
                        clear_stop_sender = true;
                        break;
                    }
                }
            }
        }

        if clear_receiver {
            self.event_receiver = None;
        }

        if clear_stop_sender {
            self.stop_sender = None;
        }

        if let Some(message) = completed_sequence_message {
            self.log.push('\n');
            self.log.push_str(&message);
            self.log.push('\n');
            self.status = message;
            self.active_sequence = None;
        }

        if let Some(task) = next_sequence_task {
            self.start_task_internal(task, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_command_checks_all_workspace_packages() {
        let spec = Task::CheckFormat.command_spec();

        assert_eq!(spec.program, "cargo");
        assert_eq!(spec.args, &["fmt", "--all", "--", "--check"]);
    }

    #[test]
    fn clippy_command_treats_warnings_as_errors() {
        let spec = Task::CheckClippy.command_spec();

        assert_eq!(spec.program, "cargo");
        assert_eq!(
            spec.args,
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ]
        );
    }

    #[test]
    fn frontend_dependencies_use_clean_lockfile_install() {
        let spec = Task::InstallFrontendDependencies.command_spec();
        let expected_program = if cfg!(windows) { "npm.cmd" } else { "npm" };

        assert_eq!(spec.program, expected_program);
        assert_eq!(
            spec.args,
            &[
                "--prefix",
                "apps/whitebase-app",
                "ci",
                "--prefer-offline",
                "--no-audit",
                "--no-fund",
            ]
        );
    }

    #[test]
    fn frontend_build_uses_the_platform_npm_launcher() {
        let spec = Task::BuildFrontend.command_spec();
        let expected_program = if cfg!(windows) { "npm.cmd" } else { "npm" };

        assert_eq!(spec.program, expected_program);
        assert_eq!(
            spec.args,
            &["--prefix", "apps/whitebase-app", "run", "build"]
        );
    }

    #[test]
    fn command_display_is_copyable_from_the_log() {
        let spec = Task::CheckWorkspace.command_spec();

        assert_eq!(spec.display(), "cargo check --workspace");
    }

    #[test]
    fn only_the_server_task_reports_server_readiness() {
        let line = "[Whitebase Server] Listening on http://127.0.0.1:1430";

        assert_eq!(
            Task::RunServer.ready_message(line),
            Some("Whitebase Server is ready")
        );
        assert_eq!(Task::CheckWorkspace.ready_message(line), None);
    }

    #[test]
    fn ansi_escape_sequences_are_removed_from_log_lines() {
        let input = "\x1b[36mvite v6.4.3 \x1b[32mbuilding for production...\x1b[39m";

        let output = strip_ansi_escape_sequences(input);

        assert_eq!(output, "vite v6.4.3 building for production...");
    }

    #[test]
    fn plain_log_lines_are_not_changed() {
        let input = "6 modules transformed.";

        let output = strip_ansi_escape_sequences(input);

        assert_eq!(output, input);
    }

    #[test]
    fn wasm_check_uses_wasm_target() {
        let spec = Task::CheckWasm.command_spec();

        assert_eq!(spec.program, "cargo");
        assert_eq!(
            spec.args,
            [
                "check",
                "-p",
                "whitebase-wasm",
                "--target",
                "wasm32-unknown-unknown",
            ]
        );
    }

    #[test]
    fn c_api_build_uses_platform_appropriate_route() {
        let spec = Task::BuildCApi.command_spec();

        if cfg!(windows) {
            assert_eq!(spec.program, "cmd.exe");
            assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "c-api-build"]);
        } else {
            assert_eq!(spec.program, "cargo");
            assert_eq!(spec.args, ["build", "-p", "whitebase-c-api"]);
        }
    }

    #[test]
    fn c_api_release_build_uses_ops_script() {
        let spec = Task::BuildCApiRelease.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "c-api-release-build"]);
    }

    #[test]
    fn workspace_build_excludes_the_running_control_center() {
        let spec = Task::BuildWorkspace.command_spec();

        assert_eq!(spec.program, "cargo");
        assert_eq!(
            spec.args,
            [
                "build",
                "--workspace",
                "--exclude",
                "whitebase-control-center",
            ]
        );
    }

    #[test]
    fn control_center_release_build_is_safe_for_the_current_platform_and_profile() {
        let expected = !cfg!(target_os = "windows") || cfg!(debug_assertions);

        assert_eq!(Task::BuildControlCenterRelease.is_supported(), expected);
    }

    #[test]
    fn wasm_build_uses_wasm_pack() {
        let spec = Task::BuildWasm.command_spec();

        assert_eq!(spec.program, "wasm-pack");
        assert_eq!(
            spec.args,
            [
                "build",
                "--target",
                "web",
                "--dev",
                "--out-dir",
                "../../apps/whitebase-app/src/wasm",
            ]
        );
    }

    #[test]
    fn wasm_build_runs_inside_wasm_crate() {
        let working_directory = Task::BuildWasm.working_directory();

        assert_eq!(
            working_directory,
            repository_root().join("crates").join("whitebase-wasm")
        );
    }

    #[test]
    fn cpp_client_build_uses_ops_script() {
        let spec = Task::BuildCppClient.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "cpp-build"]);
    }

    #[test]
    fn cpp_client_support_matches_platform() {
        assert_eq!(
            Task::BuildCppClient.is_supported(),
            cfg!(target_os = "windows")
        );
    }

    #[test]
    fn cpp_client_check_uses_ops_script() {
        let spec = Task::CheckCppClient.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "cpp-check"]);
    }

    #[test]
    fn cpp_client_tasks_support_matches_platform() {
        let expected = cfg!(target_os = "windows");

        assert_eq!(Task::CheckCppClient.is_supported(), expected);
        assert_eq!(Task::CheckCppAdapter.is_supported(), expected);
        assert_eq!(Task::CheckAssembly.is_supported(), expected);
        assert_eq!(Task::CheckCppBackend.is_supported(), expected);
        assert_eq!(Task::BuildCppClient.is_supported(), expected);
        assert_eq!(Task::BuildAssemblyClient.is_supported(), expected);
    }

    #[test]
    fn cpp_backend_check_uses_ops_script() {
        let spec = Task::CheckCppBackend.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "cpp-backend-check"]);
    }

    #[test]
    fn cpp_adapter_check_uses_ops_script() {
        let spec = Task::CheckCppAdapter.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "cpp-adapter-check"]);
    }

    #[test]
    fn assembly_check_uses_ops_script() {
        let spec = Task::CheckAssembly.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "asm-check"]);
    }

    #[test]
    fn assembly_client_build_uses_ops_script() {
        let spec = Task::BuildAssemblyClient.command_spec();

        assert_eq!(spec.program, "cmd.exe");
        assert_eq!(spec.args, ["/C", "scripts\\ops.bat", "asm-build"]);
    }

    #[test]
    fn linux_native_tasks_use_linux_native_script() {
        let check = Task::CheckLinuxNative.command_spec();
        let build = Task::BuildLinuxNative.command_spec();
        let release = Task::BuildLinuxNativeRelease.command_spec();

        assert_eq!(check.program, "bash");
        assert_eq!(check.args, ["scripts/linux-native.sh", "check"]);
        assert_eq!(build.program, "bash");
        assert_eq!(build.args, ["scripts/linux-native.sh", "build"]);
        assert_eq!(release.program, "bash");
        assert_eq!(release.args, ["scripts/linux-native.sh", "release"]);
    }

    #[test]
    fn linux_native_task_support_matches_platform() {
        let expected = cfg!(all(target_os = "linux", target_arch = "x86_64"));

        assert_eq!(Task::CheckLinuxNative.is_supported(), expected);
        assert_eq!(Task::BuildLinuxNative.is_supported(), expected);
        assert_eq!(Task::BuildLinuxNativeRelease.is_supported(), expected);
    }

    #[test]
    fn windows_gnu_native_tasks_use_windows_gnu_script() {
        let check = Task::CheckWindowsGnuNative.command_spec();
        let build = Task::BuildWindowsGnuNative.command_spec();
        let release = Task::BuildWindowsGnuNativeRelease.command_spec();

        for spec in [check, build, release] {
            assert_eq!(spec.program, "powershell.exe");
            assert!(spec.args.contains(&"-NoProfile"));
            assert!(spec.args.contains(&"-ExecutionPolicy"));
            assert!(spec.args.contains(&"Bypass"));
            assert!(spec.args.contains(&"-File"));
            assert!(spec.args.contains(&"scripts\\windows-gnu-native.ps1"));
        }

        assert_eq!(check.args.last(), Some(&"check"));
        assert_eq!(build.args.last(), Some(&"build"));
        assert_eq!(release.args.last(), Some(&"release"));
    }

    #[test]
    fn windows_gnu_native_task_support_matches_platform() {
        let expected = cfg!(all(target_os = "windows", target_arch = "x86_64"));

        assert_eq!(Task::CheckWindowsGnuNative.is_supported(), expected);
        assert_eq!(Task::BuildWindowsGnuNative.is_supported(), expected);
        assert_eq!(Task::BuildWindowsGnuNativeRelease.is_supported(), expected);
    }

    #[test]
    fn check_all_sequence_contains_expected_supported_tasks() {
        let sequence = TaskSequence::check_all();
        let expected = CHECK_ALL_TASKS
            .iter()
            .copied()
            .filter(|task| task.is_supported())
            .collect::<VecDeque<_>>();

        assert_eq!(sequence.label, "Check All");
        assert_eq!(
            sequence.success_message,
            "Whitebase check completed successfully"
        );
        assert_eq!(sequence.pending_tasks, expected);
    }

    #[test]
    fn build_all_sequence_contains_expected_supported_tasks() {
        let sequence = TaskSequence::build_all();
        let expected = BUILD_ALL_TASKS
            .iter()
            .copied()
            .filter(|task| task.is_supported())
            .collect::<VecDeque<_>>();

        assert_eq!(sequence.label, "Build All");
        assert_eq!(
            sequence.success_message,
            "Whitebase build completed successfully"
        );
        assert_eq!(sequence.pending_tasks, expected);
    }

    #[test]
    fn workspace_release_build_excludes_the_running_control_center() {
        let spec = Task::BuildWorkspaceRelease.command_spec();

        assert_eq!(spec.program, "cargo");
        assert_eq!(
            spec.args,
            [
                "build",
                "--workspace",
                "--release",
                "--locked",
                "--exclude",
                "whitebase-control-center",
            ]
        );
    }

    #[test]
    fn wasm_release_build_uses_release_profile() {
        let spec = Task::BuildWasmRelease.command_spec();

        assert_eq!(spec.program, "wasm-pack");
        assert_eq!(
            spec.args,
            [
                "build",
                "--target",
                "web",
                "--release",
                "--out-dir",
                "../../apps/whitebase-app/src/wasm",
            ]
        );
        assert_eq!(
            Task::BuildWasmRelease.working_directory(),
            repository_root().join("crates").join("whitebase-wasm")
        );
    }

    #[test]
    fn tauri_release_build_uses_the_platform_npm_launcher() {
        let spec = Task::BuildTauriRelease.command_spec();
        let expected_program = if cfg!(windows) { "npm.cmd" } else { "npm" };

        assert_eq!(spec.program, expected_program);
        assert_eq!(
            spec.args,
            [
                "--prefix",
                "apps/whitebase-app",
                "run",
                "tauri",
                "--",
                "build",
            ]
        );
    }

    #[test]
    fn windows_native_release_builds_the_release_solution() {
        let spec = Task::BuildWindowsNativeRelease.command_spec();

        assert_eq!(spec.program, "powershell.exe");
        assert!(spec.args.contains(&"-Command"));
        assert!(
            spec.args
                .iter()
                .any(|argument| argument.contains("/p:Configuration=Release"))
        );
        assert!(
            spec.args
                .iter()
                .any(|argument| argument.contains("Whitebase.Cpp.slnx"))
        );
    }

    #[test]
    fn windows_release_tasks_support_matches_platform() {
        let expected = cfg!(target_os = "windows");

        assert_eq!(Task::BuildCApiRelease.is_supported(), expected);
        assert_eq!(Task::BuildWindowsNativeRelease.is_supported(), expected);
    }

    #[test]
    fn frontend_dependencies_precede_frontend_consumers_in_sequences() {
        fn index(tasks: &[Task], expected: Task) -> usize {
            tasks
                .iter()
                .position(|task| *task == expected)
                .expect("task must be present")
        }

        assert!(
            index(CHECK_ALL_TASKS, Task::InstallFrontendDependencies)
                < index(CHECK_ALL_TASKS, Task::BuildFrontend)
        );
        assert!(
            index(BUILD_ALL_TASKS, Task::InstallFrontendDependencies)
                < index(BUILD_ALL_TASKS, Task::BuildFrontend)
        );
        assert!(
            index(RELEASE_ALL_TASKS, Task::InstallFrontendDependencies)
                < index(RELEASE_ALL_TASKS, Task::BuildTauriRelease)
        );
    }

    #[test]
    fn release_all_sequence_contains_expected_supported_tasks() {
        let sequence = TaskSequence::release_all();
        let expected = RELEASE_ALL_TASKS
            .iter()
            .copied()
            .filter(|task| task.is_supported())
            .collect::<VecDeque<_>>();

        assert_eq!(sequence.label, "Release All");
        assert_eq!(
            sequence.success_message,
            "Whitebase Release build completed successfully"
        );
        assert_eq!(sequence.pending_tasks, expected);
    }
}
