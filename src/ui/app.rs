use crate::{
    debugger::{debug_em::HackSystem, shell::Shell},
    emulator::engine::StopReason,
    ui::key_lookup::lookup_key,
};

use egui::Id;

use thiserror::Error;
use web_time::Duration;

use super::widgets::{
    console::ConsoleWindow, cpu::CpuWindow, files::FilesWindow, screen::ScreenWindow,
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
    pub windows: Vec<Box<dyn AppWindow>>,
    init: bool,
    //  shell: Shell,
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

pub enum AppMessage {
    LoadBinary(String),
    ConsoleCommand(String),
    Quit,
    None,
}
pub struct UpdateMessage {
    pub message: UpdateType,
    pub widget: Id,
}
pub enum UpdateType {
    Text(String),
}
pub trait AppWindow {
    fn draw(&mut self, ctx: &egui::Context, open: &mut bool, hacksys: &HackSystem);
    fn update(&mut self, _msg: UpdateMessage, _hacksys: &HackSystem) {}
    fn name(&self) -> &'static str;
    fn keyboard_peek(&mut self, _ctx: &egui::Context, _hacksys: &HackSystem) -> Option<AppMessage> {
        None
    }
    fn id(&self) -> Id {
        Id::new(self.name())
    }
}
// ascii value of current key

pub(crate) static mut CURRENT_KEY: u8 = 0;
impl HackEgui {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Self {
            hacksys: HackSystem::new(),
            running: false,
            elapsed: Duration::from_secs(0),
            text: String::new(),
            console: String::new(),
            windows: vec![
                Box::new(ConsoleWindow::new()),
                Box::new(ScreenWindow::new()),
                Box::new(CpuWindow::new()),
                Box::new(FilesWindow::new()),
            ],
            init: true,
            // shell: Shell::new(),
        }
    }

    // fn draw_ram(&mut self, ui: &mut Ui) {
    //     let screen = self.hacksys.get_screen_ram();
    //     egui::Grid::new("RAM").num_columns(2).show(ui, |ui| {
    //         for i in 0..100 {
    //             ui.label(format!("0x{:04X}", i));
    //             ui.label(format!("0x{:04X}", screen[i as usize]));
    //             ui.end_row();
    //         }
    //     });
    // }
}

impl eframe::App for HackEgui {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        //  eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui_extras::install_image_loaders(ctx);
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        let mut open = true;
        let mut app_msg = AppMessage::None;
        if !ctx.wants_keyboard_input() {
            ctx.input(|inp| {
                // log::trace!("{:?}", inp);
                if !inp.keys_down.is_empty() {
                    println!(
                        "{:?} {:?} {} ",
                        inp.keys_down,
                        inp.modifiers,
                        inp.keys_down.len()
                    );
                    lookup_key(inp);
                    println!("key: {}", unsafe { CURRENT_KEY as char });
                } else {
                    unsafe {
                        CURRENT_KEY = 0;
                    }
                }
                if let Some(egui::Event::Text(text)) = inp.events.first() {
                    // if let egui::Event::Text(text) = ev {
                    println!("Text {:?}", text);
                    unsafe {
                        CURRENT_KEY = text.chars().next().unwrap() as u8;
                    }
                    //}
                }
            });
        } else {
            // pass events to out windows, see if they want to do anything with them
            // if they do, we will get a message back
            if let Some(id) = ctx.memory(|mem| mem.focused()) {
                for window in self.windows.iter_mut() {
                    if id == window.id() {
                        if let Some(msg) = window.keyboard_peek(ctx, &self.hacksys) {
                            app_msg = msg;
                        }
                        break;
                    }
                }
            }
        }

        // execute the message we got
        match app_msg {
            AppMessage::LoadBinary(bin) => {
                let _ = self.hacksys.engine.load_file(&bin);
            }
            AppMessage::ConsoleCommand(cmd) => {
                let id = self.windows[0].id();
                let cw = &mut self.windows[0];
                if let Ok(response) = Shell::execute_message(&cmd, &mut self.hacksys) {
                    cw.update(
                        UpdateMessage {
                            message: UpdateType::Text(response),
                            widget: id,
                        },
                        &self.hacksys,
                    );
                }
            }
            AppMessage::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            AppMessage::None => {}
        }

        // draw our widget windows

        for window in self.windows.iter_mut() {
            window.draw(ctx, &mut open, &self.hacksys);
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

        egui::Window::new("xxxx")
            .default_height(500.0)
            .show(ctx, |ui| ctx.inspection_ui(ui));
        // egui::Window::new("screen2")
        //     .default_height(500.0)
        //     .show(ctx, |ui| {
        //         //let mut string = "".to_string();
        //         ui.add(
        //             egui::TextEdit::singleline(&mut self.text).hint_text("Write something here"),
        //         );
        //         ui.end_row();
        //         let egui_icon = egui::include_image!("../../data/fast-forward.svg");
        //         if ui
        //             .add(egui::Button::image_and_text(egui_icon, "Run"))
        //             .clicked()
        //         {
        //             //*boolean = !*boolean;
        //         }
        //     });

        // egui::Window::new("RAM")
        //     .default_height(500.0)
        //     .show(ctx, |ui| self.draw_ram(ui));

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
                    }
                    StopReason::HardLoop => {
                        self.running = false;
                    }

                    StopReason::RefreshUI => {
                        ctx.request_repaint();
                    }
                    _ => {}
                },
                Err(err) => {
                    self.running = false;
                    log::error!("Error: {}", err);
                }
            }
        };
    }
}
