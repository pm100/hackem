use crate::{
    emulator::{hacksys::HackSystem, lib::StopReason},
    ui::key_lookup::lookup_key,
};

use egui::{Id, Key, Modifiers, Pos2, Sense, Ui, Vec2};

use thiserror::Error;
use web_time::Duration;

use super::widgets::console::ConsoleWindow;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HackEmulator {
    //  #[serde(skip)]
    pub(crate) hacksys: HackSystem,
    // #[serde(skip)]
    running: bool,
    //#[serde(skip)]
    elapsed: Duration,
    text: String,
    console: String,
    windows: Vec<Box<dyn MyWidget>>,
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
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;
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
pub trait MyWidget {
    fn draw(&mut self, ctx: &egui::Context, open: &mut bool);
    fn update(&mut self, msg: UpdateMessage) {}
    fn name(&self) -> &'static str;
    fn keyboard_peek(&mut self, ctx: &egui::Context) -> Option<AppMessage> {
        None
    }
    fn id(&self) -> Id {
        Id::new(self.name())
    }
}
// ascii value of current key

pub(crate) static mut CURRENT_KEY: u8 = 0;
impl HackEmulator {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
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
            windows: vec![Box::new(ConsoleWindow::new())],
        }
    }

    fn draw_ram(&mut self, ui: &mut Ui) {
        let screen = self.hacksys.get_screen_ram();
        egui::Grid::new("RAM").num_columns(2).show(ui, |ui| {
            for i in 0..100 {
                ui.label(format!("0x{:04X}", i));
                ui.label(format!("0x{:04X}", screen[i as usize]));
                ui.end_row();
            }
        });
    }

    fn draw_cpu_state(&mut self, ui: &mut Ui) {
        let (pc, a, d) = self.hacksys.get_registers();
        egui::Grid::new("CPU Status").num_columns(2).show(ui, |ui| {
            ui.label("PC: ");
            ui.label(format!("0x{:04X}", pc));
            ui.end_row();
            ui.label("A: ");
            ui.label(format!("0x{:04X}", a));
            ui.end_row();
            ui.label("D: ");
            ui.label(format!("0x{:04X}", d));
            ui.end_row();
            ui.label("Speed: ");
            let speed = self.hacksys.get_speed();
            if speed > 0.0 {
                ui.label(format!("{}", speed));
            } else {
                ui.label("Stopped");
            }
            // ui.label(format!("0x{:04X}", d));
            ui.end_row();
            ui.label("Elapsed: ");
            ui.label(format!("{}", self.elapsed.as_secs()));
            ui.end_row();
        });
    }

    fn draw_screen(&mut self, ui: &mut Ui) {
        let color = if ui.style().visuals.dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        let draw_area_size =
            Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32) + Vec2::splat(1.0);
        let (response, painter) = ui.allocate_painter(draw_area_size, Sense::hover());

        let top_left = Pos2::new(response.rect.min.x * 1.0, response.rect.min.y * 1.0).ceil();
        let screen = self.hacksys.get_screen_ram();
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH / 16 {
                let pixels = screen[row * SCREEN_WIDTH / 16 + col];

                if pixels != 0 {
                    // println!("pixels: {:04x} r{} c{}", pixels, row, col);
                    let y = top_left.y + ((row) as f32);

                    for i in 0..16 {
                        if pixels & (1 << i) != 0 {
                            let x = top_left.x + (((col * 16) + i) as f32);
                            // println!("x: {} y: {}", x, y);
                            let rect_points =
                                egui::Rect::from_min_size(Pos2::new(x, y), Vec2::splat(1.0));
                            painter.rect_filled(rect_points, 0.0, color);
                        }
                    }
                }
            }
        }
    }
}

impl eframe::App for HackEmulator {
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
            if let Some(id) = ctx.memory(|mem| mem.focused()) {
                // println!(
                //     "Focused: {:?} {:?}",
                //     id,
                //     Id::new("Console Window".to_string())
                // );
                for window in self.windows.iter_mut() {
                    if id == window.id() {
                        if let Some(msg) = window.keyboard_peek(ctx) {
                            app_msg = msg;
                        }
                        break;
                    }
                }
                // if id == Id::new("Console Window".to_string()) {
                //     if ctx.input_mut(|inp| inp.consume_key(Modifiers::NONE, Key::ArrowDown)) {
                //         println!("Console focus: down");
                //     }
                //     if ctx.input_mut(|inp| inp.consume_key(Modifiers::NONE, Key::ArrowUp)) {
                //         println!("Console focus: Up");
                //     }
                // }
            }
        }
        match app_msg {
            AppMessage::LoadBinary(bin) => {
                self.hacksys.load_file(&bin);
            }
            AppMessage::ConsoleCommand(cmd) => {
                let id = self.windows[0].id();
                self.windows[0].update(UpdateMessage {
                    message: UpdateType::Text(cmd),
                    widget: id,
                });
            }
            AppMessage::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            AppMessage::None => {}
        }
        self.windows[0].draw(ctx, &mut open);
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
                            self.hacksys.load_file(&bin);
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
                let _ = self.hacksys.execute_instructions(Duration::ZERO);
            }

            if ui.button("Run").clicked() {
                self.running = !self.running;
            }
        });
        // egui::Window::new("console")
        //     .default_height(500.0)
        //     .show(ctx, |ui| {
        //         egui::ScrollArea::vertical().show(ui, |ui| {
        //             ui.add(
        //                 egui::TextEdit::multiline(&mut self.console)
        //                     .id(Id::new("consolexx".to_string()))
        //                     .font(egui::TextStyle::Monospace) // for cursor height
        //                     .code_editor()
        //                     .desired_rows(10)
        //                     .lock_focus(true)
        //                     .desired_width(f32::INFINITY),
        //                 // .layouter(&mut layouter),
        //             );
        //         });
        //     });

        egui::Window::new("screen")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_screen(ui));
        egui::Window::new("xxxx")
            .default_height(500.0)
            .show(ctx, |ui| ctx.inspection_ui(ui));
        egui::Window::new("screen2")
            .default_height(500.0)
            .show(ctx, |ui| {
                //let mut string = "".to_string();
                ui.add(
                    egui::TextEdit::singleline(&mut self.text).hint_text("Write something here"),
                );
                ui.end_row();
                let egui_icon = egui::include_image!("../../data/fast-forward.svg");
                if ui
                    .add(egui::Button::image_and_text(egui_icon, "Run"))
                    .clicked()
                {
                    //*boolean = !*boolean;
                }
            });
        egui::Window::new("CPU")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_cpu_state(ui));
        egui::Window::new("RAM")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_ram(ui));

        // run instructions until we hit a stop condition
        // - time out after the supliied number of ms draw next screen frame)
        // - Sys.halt or hard loop detected
        // - an error occurs
        if self.running {
            let stop = self.hacksys.execute_instructions(Duration::from_millis(50));
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
                },
                Err(err) => {
                    self.running = false;
                    log::error!("Error: {}", err);
                }
            }
        };
    }
}
