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
    CheckCppClient,
    CheckCppBackend,
    CheckCppAdapter,
    CheckAssembly,
    BuildWorkspace,
    BuildFrontend,
    BuildWasm,
    BuildCApi,
    BuildCppClient,
    BuildAssemblyClient,
    BuildControlCenterRelease,
    TestWorkspace,
    RunServer,
}

const CHECK_ALL_TASKS: &[Task] = &[
    Task::CheckFormat,
    // RustのAdapterを検査する前に、Windowsのネイティブライブラリを準備する。
    // 非対応OSではTaskSequence::check_all()がこれらを除外する。
    Task::CheckCppBackend,
    Task::CheckAssembly,
    Task::CheckClippy,
    Task::TestWorkspace,
    Task::CheckWasm,
    Task::BuildFrontend,
    Task::CheckCppClient,
    Task::CheckCppAdapter,
];

const BUILD_ALL_TASKS: &[Task] = &[
    // Build欄の個別タスクをすべて順番に実行する。
    // WindowsではWorkspace内のAdapterをリンクする前にネイティブ成果物を準備する。
    Task::BuildCApi,
    Task::BuildCppClient,
    Task::BuildAssemblyClient,
    Task::BuildWorkspace,
    Task::BuildWasm,
    // Wasm生成物を含んだ状態でFrontendを組み立てる。
    Task::BuildFrontend,
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
            Self::CheckCppClient => "Check C++ Client",
            Self::CheckCppBackend => "Check C++ Backend",
            Self::CheckCppAdapter => "Check C++ Adapter",
            Self::CheckAssembly => "Check Assembly",
            Self::BuildWorkspace => "Build Workspace",
            Self::BuildFrontend => "Build Frontend",
            Self::BuildWasm => "Build Wasm",
            Self::BuildCApi => "Build C API",
            Self::BuildCppClient => "Build C++ Client",
            Self::BuildAssemblyClient => "Build Assembly Client",
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
            Self::CheckCppClient => "Checking C++ Client...",
            Self::CheckCppBackend => "Checking C++ Backend...",
            Self::CheckCppAdapter => "Checking C++ Adapter...",
            Self::CheckAssembly => "Checking Assembly...",
            Self::BuildWorkspace => "Building Workspace...",
            Self::BuildFrontend => "Building Frontend...",
            Self::BuildWasm => "Building Wasm...",
            Self::BuildCApi => "Building C API...",
            Self::BuildCppClient => "Building C++ Client...",
            Self::BuildAssemblyClient => "Building Assembly Client...",
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
            Self::CheckCppClient => "C++ Client check completed successfully",
            Self::CheckCppBackend => "C++ Backend check completed successfully",
            Self::CheckCppAdapter => "C++ Adapter check completed successfully",
            Self::CheckAssembly => "Assembly check completed successfully",
            Self::BuildWorkspace => "Workspace build completed successfully",
            Self::BuildFrontend => "Frontend build completed successfully",
            Self::BuildWasm => "Wasm build completed successfully",
            Self::BuildCApi => "C API build completed successfully",
            Self::BuildCppClient => "C++ Client build completed successfully",
            Self::BuildAssemblyClient => "Assembly Client build completed successfully",
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
            Self::BuildCApi => CommandSpec {
                program: "cargo",
                args: &["build", "-p", "whitebase-c-api"],
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
            Self::BuildControlCenterRelease => CommandSpec {
                program: "cargo",
                args: &["build", "--release", "-p", "whitebase-control-center"],
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
            Self::BuildWasm => repository_root().join("crates").join("whitebase-wasm"),

            _ => repository_root(),
        }
    }

    fn is_supported(self) -> bool {
        if matches!(
            self,
            Self::CheckCppClient
                | Self::CheckCppAdapter
                | Self::CheckCppBackend
                | Self::CheckAssembly
                | Self::BuildCppClient
                | Self::BuildAssemblyClient
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
                    Task::BuildWorkspace,
                    Task::BuildFrontend,
                    Task::BuildWasm,
                    Task::BuildCApi,
                    Task::BuildCppClient,
                    Task::BuildAssemblyClient,
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
    fn c_api_build_uses_c_api_package() {
        let spec = Task::BuildCApi.command_spec();

        assert_eq!(spec.program, "cargo");
        assert_eq!(spec.args, ["build", "-p", "whitebase-c-api"]);
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
}
