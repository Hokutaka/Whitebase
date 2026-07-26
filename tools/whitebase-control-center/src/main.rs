use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use eframe::egui;

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
}

impl Task {
    fn label(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Check Control Center",
        }
    }

    fn running_message(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Checking Control Center...",
        }
    }

    fn success_message(self) -> &'static str {
        match self {
            Self::CheckControlCenter => "Check completed successfully",
        }
    }

    fn command(self) -> Command {
        match self {
            Self::CheckControlCenter => {
                let mut command = Command::new("cargo");
                command.args(["check", "-p", "whitebase-control-center"]);
                command
            }
        }
    }
}

enum WorkerEvent {
    Log(String),
    Finished { success: bool, message: String },
}

struct ControlCenterApp {
    status: String,
    log: String,
    running: bool,
    event_receiver: Option<Receiver<WorkerEvent>>,
}

impl Default for ControlCenterApp {
    fn default() -> Self {
        Self {
            status: "Idle".to_owned(),
            log: "No output yet.".to_owned(),
            running: false,
            event_receiver: None,
        }
    }
}

impl eframe::App for ControlCenterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_events();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Whitebase Control Center");
            ui.separator();

            ui.label(format!(
                "Platform: {} / {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ));

            ui.add_space(12.0);

            let task = Task::CheckControlCenter;
            let button = egui::Button::new(task.label());

            if ui.add_enabled(!self.running, button).clicked() {
                self.start_task(task);
            }

            ui.label(format!("Status: {}", self.status));

            if self.running {
                ui.spinner();

                // ワーカースレッドから届いたログを定期的に受け取るため、
                // 実行中は画面を再描画する。
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            }

            ui.add_space(12.0);
            ui.separator();
            ui.heading("Log");

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.log)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(12)
                            .interactive(false),
                    );
                });
        });
    }
}

impl ControlCenterApp {
    fn start_task(&mut self, task: Task) {
        let (sender, receiver) = mpsc::channel();

        self.status = task.running_message().to_owned();
        self.log.clear();
        self.running = true;
        self.event_receiver = Some(receiver);

        thread::spawn(move || {
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

            let exit_status = child.wait();

            if let Some(thread) = stdout_thread {
                let _ = thread.join();
            }

            if let Some(thread) = stderr_thread {
                let _ = thread.join();
            }

            let event = match exit_status {
                Ok(status) if status.success() => WorkerEvent::Finished {
                    success: true,
                    message: task.success_message().to_owned(),
                },
                Ok(status) => WorkerEvent::Finished {
                    success: false,
                    message: format!("{} failed with {status}", task.label()),
                },
                Err(error) => WorkerEvent::Finished {
                    success: false,
                    message: format!("Failed while waiting for Cargo: {error}"),
                },
            };

            let _ = sender.send(event);
        });
    }

    fn receive_events(&mut self) {
        let mut clear_receiver = false;

        if let Some(receiver) = self.event_receiver.as_ref() {
            loop {
                match receiver.try_recv() {
                    Ok(WorkerEvent::Log(line)) => {
                        self.log.push_str(&line);
                        self.log.push('\n');
                    }
                    Ok(WorkerEvent::Finished { success, message }) => {
                        self.status = message;
                        self.running = false;
                        clear_receiver = true;

                        if !success {
                            self.log.push_str("\nProcess finished with an error.\n");
                        }

                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        self.status = "Worker disconnected unexpectedly".to_owned();
                        self.running = false;
                        clear_receiver = true;
                        break;
                    }
                }
            }
        }

        if clear_receiver {
            self.event_receiver = None;
        }
    }
}
