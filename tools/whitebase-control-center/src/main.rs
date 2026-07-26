use std::{
    process::Command,
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

struct CheckResult {
    status: String,
    log: String,
}

struct ControlCenterApp {
    status: String,
    log: String,
    running: bool,
    result_receiver: Option<Receiver<CheckResult>>,
}

impl Default for ControlCenterApp {
    fn default() -> Self {
        Self {
            status: "Idle".to_owned(),
            log: "No output yet.".to_owned(),
            running: false,
            result_receiver: None,
        }
    }
}

impl eframe::App for ControlCenterApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.receive_result();

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Whitebase Control Center");
            ui.separator();

            ui.label(format!(
                "Platform: {} / {}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ));

            ui.add_space(12.0);

            let button = egui::Button::new("Check Control Center");

            if ui.add_enabled(!self.running, button).clicked() {
                self.start_check();
            }

            ui.label(format!("Status: {}", self.status));

            if self.running {
                ui.spinner();
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
    fn start_check(&mut self) {
        let (sender, receiver) = mpsc::channel();

        self.status = "Checking...".to_owned();
        self.log = "Running cargo check...\n".to_owned();
        self.running = true;
        self.result_receiver = Some(receiver);

        thread::spawn(move || {
            let result = Command::new("cargo")
                .args(["check", "-p", "whitebase-control-center"])
                .output();

            let check_result = match result {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);

                    let mut log = String::new();

                    if !stdout.is_empty() {
                        log.push_str(&stdout);
                    }

                    if !stderr.is_empty() {
                        log.push_str(&stderr);
                    }

                    let status = if output.status.success() {
                        "Check completed successfully".to_owned()
                    } else {
                        format!("Check failed with {}", output.status)
                    };

                    CheckResult { status, log }
                }
                Err(error) => CheckResult {
                    status: "Failed to start Cargo".to_owned(),
                    log: error.to_string(),
                },
            };

            let _ = sender.send(check_result);
        });
    }

    fn receive_result(&mut self) {
        let result = self.result_receiver.as_ref().map(Receiver::try_recv);

        match result {
            Some(Ok(result)) => {
                self.status = result.status;
                self.log = result.log;
                self.running = false;
                self.result_receiver = None;
            }
            Some(Err(TryRecvError::Disconnected)) => {
                self.status = "Worker disconnected unexpectedly".to_owned();
                self.running = false;
                self.result_receiver = None;
            }
            Some(Err(TryRecvError::Empty)) | None => {}
        }
    }
}
