use crate::{
    debugger::{debug_em::HackSystem, shell::Shell},
    emulator::engine::StopReason,
};

use egui_console::{ConsoleBuilder, ConsoleEvent, ConsoleWindow};
use thiserror::Error;
use web_time::Duration;

use super::widgets::{
    code::CodeWindow,
    cpu::CpuWindow,
    data::DataWindow,
    screen::ScreenWindow,
};

pub struct HackEgui {
    pub(crate) hacksys: HackSystem,
    running: bool,
    console_window: ConsoleWindow,
    console_window_open: bool,
    screen_window: ScreenWindow,
    cpu_window: CpuWindow,
    code_window: CodeWindow,
    data_window1: DataWindow,
    data_window2: DataWindow,
    shell: Shell,
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Invalid RAM read address {0}")]
    InvalidReadAddress(u16),
    #[error("Invalid RAM write address {0}")]
    InvalidWriteAddress(u16),
    #[error("Invalid instruction address {0}")]
    InvalidPC(u16),
}

impl HackEgui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let ret = Self {
            hacksys: HackSystem::new(),
            running: false,
            console_window: ConsoleBuilder::new()
                .prompt(">> ")
                .history_size(20)
                .tab_quote_character('\"')
                .build(),
            console_window_open: true,
            screen_window: ScreenWindow::new(),
            cpu_window: CpuWindow::new(),
            code_window: CodeWindow::new(),
            data_window1: DataWindow::new("Data 1"),
            data_window2: DataWindow::new("Data 2"),
            shell: Shell::new(),
        };
        ret
    }

    fn save_history(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(home) = dirs::home_dir() {
            let history_path = home.join(".hackem_history");
            let history: Vec<String> = self.console_window.get_history().into_iter().collect();
            let _ = std::fs::write(history_path, history.join("\n"));
        }
    }

    fn console_write(&mut self, msg: &str) {
        self.console_window.write(msg);
        self.console_window.prompt();
    }
}

impl eframe::App for HackEgui {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut open = true;

        // Draw non-console windows
        self.screen_window.draw(ctx, &mut open, &self.hacksys);
        self.cpu_window.draw(ctx, &mut open, &self.hacksys);
        self.code_window.draw(ctx, &mut open, &mut self.hacksys);
        self.data_window1.draw(ctx, &mut open, &self.hacksys);
        self.data_window2.draw(ctx, &mut open, &self.hacksys);

        // Console window using egui_console
        let mut console_response: ConsoleEvent = ConsoleEvent::None;
        egui::Window::new("Console")
            .default_height(500.0)
            .resizable(true)
            .open(&mut self.console_window_open)
            .show(ctx, |ui| {
                console_response = self.console_window.draw(ui);
            });

        if let ConsoleEvent::Command(cmd) = console_response {
            if let Ok(response) = self.shell.execute_message(&cmd, &mut self.hacksys) {
                match response.as_str() {
                    "__go__" => {
                        self.running = true;
                    }
                    "__quit__" => {
                        self.save_history();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    _ => {
                        if !response.is_empty() {
                            self.console_write(&response);
                        } else {
                            self.console_window.prompt();
                        }
                    }
                }
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                #[cfg(target_arch = "wasm32")]
                self.wasm_update_and_menu(ui);
                #[cfg(not(target_arch = "wasm32"))]
                ui.menu_button("File", |ui| {
                    if ui.button("Load Binary").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new().add_filter("x", &["hx"]).pick_file()
                        {
                            let bin = std::fs::read_to_string(path).unwrap();
                            let _ignore = self.hacksys.engine.load_file(&bin);
                        }
                        ui.close_menu();
                    }

                    if ui.button("Quit").clicked() {
                        self.save_history();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                egui::widgets::global_dark_light_mode_buttons(ui);

                if ui.button("Console").clicked() {
                    self.console_window_open = true;
                }
            });

            ui.add_enabled_ui(!self.running, |ui| {
                if ui.button("Step").clicked() {
                    let _ = self.hacksys.engine.execute_instructions(Duration::ZERO);
                }
            });

            let run_label = if self.running { "Pause" } else { "Run" };
            if ui.button(run_label).clicked() {
                self.running = !self.running;
            }
        });

        if self.running {
            let pc = self.hacksys.engine.pc;
            let stop = self
                .hacksys
                .engine
                .execute_instructions(Duration::from_millis(50));
            match stop {
                Ok(reason) => match reason {
                    StopReason::SysHalt => {
                        self.running = false;
                        self.console_write("SysHalt");
                    }
                    StopReason::HardLoop => {
                        self.running = false;
                        self.console_write(&format!("Hard loop at 0x{:04X}", pc));
                    }
                    StopReason::BreakPoint => {
                        self.running = false;
                        self.console_write(&format!(
                            "Breakpoint hit at 0x{:04X}",
                            self.hacksys.engine.pc
                        ));
                    }
                    StopReason::WatchPoint => {
                        self.running = false;
                        let addr = self.hacksys.engine.triggered_watchpoint.unwrap_or(0);
                        self.console_write(&format!("Watchpoint hit at 0x{:04X}", addr));
                    }
                    StopReason::RefreshUI => {
                        ctx.request_repaint();
                    }
                },
                Err(err) => {
                    self.running = false;
                    self.console_write(&format!("Error: {}", err));
                }
            }
        }
    }
}

