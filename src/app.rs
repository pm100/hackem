use crate::StopReason;
use crate::{hacksys::HackSystem, key_lookup::lookup_key};
use egui::{vec2, Color32, InputState, Key, Painter, Pos2, Rect, Sense, TextureHandle, Ui, Vec2};

use web_time::{Duration, Instant};
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
//#[derive(serde::Deserialize, serde::Serialize)]
//#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HackEmulator {
    //  #[serde(skip)]
    pub(crate) hacksys: HackSystem,
    // #[serde(skip)]
    running: bool,
    //#[serde(skip)]
    texture: TextureHandle,
    pixels: Vec<Color32>,
    draw_count: usize,
    last_draw_time: Instant,
    start: Instant,
    elapsed: Duration,
    text: String,
}

const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;

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

        let app = Self {
            texture: cc.egui_ctx.load_texture(
                "noise",
                egui::ColorImage::example(),
                egui::TextureOptions::NEAREST,
            ),
            hacksys: HackSystem::new(),
            running: false,
            pixels: vec![egui::Color32::BLACK; SCREEN_WIDTH * SCREEN_HEIGHT],
            draw_count: 0,
            last_draw_time: Instant::now(),
            start: Instant::now(),
            elapsed: Duration::from_secs(0),
            text: String::new(),
        };

        app
    }

    // fn draw_pixel(area: &Rect, painter: &Painter, x: i32, y: i32) {
    //     let rect_points =
    //         egui::Rect::from_min_size(Pos2::new(x as f32, y as f32), Vec2::splat(1 as f32));
    //     painter.rect_filled(rect_points, 0.0, egui::Color32::BLACK);
    // }

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
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        //  eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui
        ctx.input(|inp| {
            // log::trace!("{:?}", inp);
            if inp.keys_down.len() > 0 {
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
            if let Some(ev) = inp.events.iter().next() {
                if let egui::Event::Text(text) = ev {
                    println!("{:?}", text);
                    unsafe {
                        CURRENT_KEY = text.chars().next().unwrap() as u8;
                    }
                }
            }
        });
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                #[cfg(target_arch = "wasm32")]
                self.wasm_update_and_menu(ui);
                #[cfg(not(target_arch = "wasm32"))]
                ui.menu_button("File", |ui| {
                    if ui.button("Load Binary").clicked() {
                        if let Some(path) = rfd::FileDialog::new().pick_file() {
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
                self.hacksys.execute_instructions(Duration::ZERO);
            }

            if ui.button("Run").clicked() {
                self.build_texture();

                self.running = !self.running;
            }
        });

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
            });
        egui::Window::new("CPU")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_cpu_state(ui));
        egui::Window::new("RAM")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_ram(ui));

        if self.running {
            let stop = self.hacksys.execute_instructions(Duration::from_millis(50));
            // println!("{:?}", stop);
            match stop {
                StopReason::SysHalt | StopReason::HardLoop => {
                    self.running = false;
                    self.elapsed = Instant::now() - self.start;
                }
                StopReason::ScreenUpdate(addr) => {
                    // self.update_pixels(addr);

                    //ctx.request_repaint();
                }
                StopReason::Count => {
                    ctx.request_repaint();
                }
                _ => {}
            }
        };
    }
}
