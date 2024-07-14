use crate::{
    debugger::{debug_em::HackSystem, shell::Shell},
    emulator::engine::StopReason,
    ui::key_lookup::lookup_key,
};

use egui::Id;

use thiserror::Error;
use web_time::Duration;

use super::widgets::{
    console::{ConsoleEvent, ConsoleWindow},
    cpu::CpuWindow,
    files::FilesWindow,
    screen::ScreenWindow,
};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HackEgui {
    //  #[serde(skip)]
    pub(crate) hacksys: HackSystem,
    // #[serde(skip)]
    running: bool,
    //#[serde(skip)]
    elapsed: Duration,
    text: String,
    console: String,
    // pub windows: Vec<Box<dyn AppWindow>>,
    console_window: ConsoleWindow,
    screen_window: ScreenWindow,
    cpu_window: CpuWindow,
    files_window: FilesWindow,
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

// pub enum AppMessage {
//     LoadBinary(String),
//     ConsoleCommand(String),
//     Quit,
//     None,
// }
pub struct UpdateMessage {
    pub message: UpdateType,
    //pub widget: Id,
}
pub enum UpdateType {
    Text(String),
}

impl HackEgui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        let mut ret = Self {
            hacksys: HackSystem::new(),
            running: false,
            elapsed: Duration::from_secs(0),
            text: String::new(),
            console: String::new(),

            console_window: ConsoleWindow::new(">> "),
            screen_window: ScreenWindow::new(),
            cpu_window: CpuWindow::new(),
            files_window: FilesWindow::new(),

            shell: Shell::new(),
        };
        ret.once(&cc.egui_ctx);
        ret
    }
    fn save_history(&self) {
        if cfg!(target_arch = "wasm32") {
        } else {
            if let Some(home) = dirs::home_dir() {
                let history_path = home.join(".hackem_history");

                let history = self.console_window.get_history();
                std::fs::write(history_path, history.join("\n")).unwrap();
            }
        }
    }
    fn once(&mut self, ctx: &egui::Context) {
        // boot up stuff
        if cfg!(target_arch = "wasm32") {
        } else {
            if let Some(home) = dirs::home_dir() {
                let history_path = home.join(".hackem_history");
                if let Ok(history) = std::fs::read_to_string(history_path) {
                    self.console_window.load_history(history.lines());
                }
            }
        }
        egui_extras::install_image_loaders(ctx);
    }
}

impl eframe::App for HackEgui {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //  eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let mut open = true;

        // draw all our windows

        let console_response = self.console_window.draw(ctx, &mut open, &self.hacksys);
        self.screen_window.draw(ctx, &mut open, &self.hacksys);
        self.cpu_window.draw(ctx, &mut open, &self.hacksys);
        self.files_window.draw(ctx, &mut open, &self.hacksys);

        if let ConsoleEvent::Command(cmd) = console_response {
            // get shell to execute the command
            if let Ok(response) = self.shell.execute_message(&cmd, &mut self.hacksys) {
                // some output in response
                self.console_window.sync_response(&response);
            }
        }

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

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
                            let _ignore_for_now = self.hacksys.engine.load_file(&bin);
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
            });
            if ui.button("Step").clicked() {
                let _ = self.hacksys.engine.execute_instructions(Duration::ZERO);
            }

            if ui.button("Run").clicked() {
                self.running = !self.running;
            }
        });

        egui::Window::new("ui debug")
            .default_height(500.0)
            .show(ctx, |ui| ctx.inspection_ui(ui));

        // run instructions until we hit a stop condition
        // - time out after the supliied number of ms draw next screen frame)
        // - Sys.halt or hard loop detected
        // - an error occurs
        if self.running {
            let stop = self
                .hacksys
                .engine
                .execute_instructions(Duration::from_millis(50));
            // println!("{:?}", stop);
            match stop {
                Ok(reason) => match reason {
                    StopReason::SysHalt => {
                        self.running = false;
                        self.console_window.async_message("SysHalt hit");
                    }
                    StopReason::HardLoop => {
                        self.running = false;
                    }

                    StopReason::RefreshUI => {
                        ctx.request_repaint();
                    }
                    _ => {
                        self.running = false;
                    }
                },
                Err(err) => {
                    self.running = false;
                    log::error!("Error: {}", err);
                }
            }
        };
    }
}
