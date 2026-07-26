use std::{
    io::{BufRead, BufReader},
    path::PathBuf,
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
        Box::new(|_creation_context| Ok(Box::new(ControlCenterApp::default()))),
    )
}

#[derive(Clone, Copy)]
enum Task {
    CheckControlCenter,
    CheckFormat,
    CheckClippy,
    CheckWorkspace,
    BuildWorkspace,
    BuildControlCenterRelease,
    TestWorkspace,
    RunServer,
}

impl Task {
    fn label(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Check Control Center",
            Self::CheckFormat => "Check Format",
            Self::CheckClippy => "Check Clippy",
            Self::CheckWorkspace => "Check Workspace",
            Self::BuildWorkspace => "Build Workspace",
            Self::BuildControlCenterRelease => "Build Control Center Release",
            Self::TestWorkspace => "Test Workspace",
            Self::RunServer => "Run Server",
        }
    }

    fn running_message(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Checking Control Center...",
            Self::CheckWorkspace => "Checking Workspace...",
            Self::BuildWorkspace => "Building Workspace...",
            Self::BuildControlCenterRelease => "Building Control Center Release...",
            Self::TestWorkspace => "Testing Workspace...",
            Self::CheckFormat => "Checking Format...",
            Self::CheckClippy => "Checking  with Clippy...",
            Self::RunServer => "Running Whitebase Server...",
        }
    }

    fn success_message(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Control Center check completed successfully",
            Self::CheckWorkspace => "Workspace check completed successfully",
            Self::BuildWorkspace => "Workspace build completed successfully",
            Self::BuildControlCenterRelease => {
                "Control Center Release build completed successfully"
            }
            Self::TestWorkspace => "Workspace tests completed successfully",
            Self::CheckFormat => "Format check completed successfully",
            Self::CheckClippy => "Clippy check completed successfully",
            Self::RunServer => "Whitebase Server exited successfully",
        }
    }

    fn command(self) -> Command {
        let mut command = Command::new("cargo");
        command.current_dir(repository_root());

        match self {
            Self::CheckControlCenter => {
                command.args(["check", "-p", "whitebase-control-center"]);
            }
            Self::CheckWorkspace => {
                command.args(["check", "--workspace"]);
            }
            Self::BuildWorkspace => {
                command.args(["build", "--workspace"]);
            }
            Self::TestWorkspace => {
                command.args(["test", "--workspace"]);
            }
            Self::CheckFormat => {
                command.args(["fmt", "--check"]);
            }
            Self::CheckClippy => {
                command.args(["clippy", "--workspace", "--all-targets"]);
            }
            Self::RunServer => {
                command.args(["run", "-p", "whitebase-server"]);
            }
            Self::BuildControlCenterRelease => {
                command.args(["build", "-p", "whitebase-control-center", "--release"]);
            }
        }

        command
    }
}

enum WorkerEvent {
    Log(String),
    Finished { success: bool, message: String },
    Stopped { message: String },
}

struct ControlCenterApp {
    status: String,
    log: String,
    active_task: Option<Task>,
    event_receiver: Option<Receiver<WorkerEvent>>,
    stop_sender: Option<mpsc::Sender<()>>,
}

impl Default for ControlCenterApp {
    fn default() -> Self {
        Self {
            status: "Idle".to_owned(),
            log: "No output yet.".to_owned(),
            active_task: None,
            event_receiver: None,
            stop_sender: None,
        }
    }
}

impl eframe::App for ControlCenterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_events();

        let is_running = self.active_task.is_some();

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
                for task in [
                    Task::CheckControlCenter,
                    Task::CheckFormat,
                    Task::CheckClippy,
                    Task::CheckWorkspace,
                    Task::TestWorkspace,
                ] {
                    let button = egui::Button::new(task.label());

                    if ui.add_enabled(!is_running, button).clicked() {
                        self.start_task(task);
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Build").strong());

            ui.horizontal_wrapped(|ui| {
                for task in [Task::BuildWorkspace, Task::BuildControlCenterRelease] {
                    let button = egui::Button::new(task.label());

                    if ui.add_enabled(!is_running, button).clicked() {
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
        let (sender, receiver) = mpsc::channel();
        let (stop_sender, stop_receiver) = mpsc::channel();

        self.status = task.running_message().to_owned();
        self.log = format!("Running task: {}\n", task.label());
        self.active_task = Some(task);
        self.event_receiver = Some(receiver);
        self.stop_sender = Some(stop_sender);

        thread::spawn(move || {
            // ここから下は既存処理
            let started_at = Instant::now();
            let mut command = task.command();

            let mut child = match command
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
            {
                Ok(child) => child,
                Err(error) => {
                    let _ = sender.send(WorkerEvent::Finished {
                        success: false,
                        message: format!("Failed to start Cargo: {error}"),
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
                                if sender.send(WorkerEvent::Log(line)).is_err() {
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
                    message: format!("Failed while waiting for Cargo: {} ({elapsed:.2?})", error),
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

        if let Some(receiver) = self.event_receiver.as_ref() {
            loop {
                match receiver.try_recv() {
                    Ok(WorkerEvent::Log(line)) => {
                        self.log.push_str(&line);
                        self.log.push('\n');
                    }

                    Ok(WorkerEvent::Finished { success, message }) => {
                        self.status = message;
                        self.active_task = None;
                        clear_receiver = true;
                        clear_stop_sender = true;

                        if !success {
                            self.log.push_str("\nProcess finished with an error.\n");
                        }

                        break;
                    }

                    Ok(WorkerEvent::Stopped { message }) => {
                        self.status = message;
                        self.active_task = None;
                        clear_receiver = true;
                        clear_stop_sender = true;
                        self.log.push_str("\nTask stopped by user.\n");
                        break;
                    }

                    Err(TryRecvError::Empty) => break,

                    Err(TryRecvError::Disconnected) => {
                        self.status = "Worker disconnected unexpectedly".to_owned();
                        self.active_task = None;
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
    }
}
