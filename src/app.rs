use crate::hacksys::HackSystem;
use crate::StopReason;
use egui::{vec2, Color32, Painter, Pos2, Rect, Sense, TextureHandle, Ui, Vec2};

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
}

const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;

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
            //   request_state: Rc::new(Cell::new(RequestState::Idle)),
            // request_data: Rc::new(RefCell::new(String::new())),
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
    fn draw_word(&self, word: u16) -> [Color32; 16] {
        let mut result = [Color32::BLACK; 16];
        for i in 0..16 {
            if word & (1 << i) != 0 {
                result[i] = egui::Color32::WHITE;
            }
        }
        result
    }

    fn update_pixels(&mut self, addr: u16) {
        let screen = self.hacksys.get_screen_ram();
        let addr = addr - 0x4000;
        let pix_word = screen[addr as usize];

        let row = addr as usize / (SCREEN_WIDTH / 16);
        let col = addr as usize % (SCREEN_WIDTH / 16);
        let word = self.draw_word(pix_word);
        //   println!("row: {} col: {} ", row, col);
        //self.pixels[(row * SCREEN_WIDTH) + col * 16..row * SCREEN_WIDTH + col * 16 + 16]
        //  .copy_from_slice(&word);
        let img = egui::ColorImage {
            size: [16, 1],
            pixels: word.to_vec(),
        };
        self.texture
            .set_partial([col * 16, row], img, egui::TextureOptions::LINEAR);
    }
    fn build_texture(&mut self) {
        self.texture.set(
            egui::ColorImage {
                size: [SCREEN_WIDTH, SCREEN_HEIGHT],
                pixels: self.pixels.clone(),
            },
            egui::TextureOptions::NEAREST,
        );
    }
    fn draw_screen2(&mut self, ui: &mut Ui) {
        let now = Instant::now();
        if now - self.last_draw_time > std::time::Duration::from_millis(100) {
            self.last_draw_time = now;
            //  self.build_texture();
        }
        let size = self.texture.size_vec2();
        let sized_texture = egui::load::SizedTexture::new(&self.texture, size);
        ui.add(egui::Image::new(sized_texture).fit_to_exact_size(size));
    }
    fn draw_screen(&mut self, ui: &mut Ui) {
        let color = if ui.style().visuals.dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        // println!("draw_screen {}", self.screen_ram[0]);
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

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
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
                self.hacksys.execute_instructions(1);
            }

            if ui.button("Run").clicked() {
                self.build_texture();

                self.running = !self.running;
            }
        });

        // egui::Window::new("screen")
        //     .default_height(500.0)
        //     .show(ctx, |ui| self.draw_screen(ui));
        egui::Window::new("screen2")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_screen2(ui));
        egui::Window::new("CPU")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_cpu_state(ui));
        egui::Window::new("RAM")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_ram(ui));

        if self.running {
            let stop = self.hacksys.execute_instructions(1000_000);
            //  println!("{:?}", stop);
            match stop {
                StopReason::SysHalt | StopReason::HardLoop => {
                    self.running = false;
                    self.elapsed = Instant::now() - self.start;
                }
                StopReason::ScreenUpdate(addr) => {
                    self.update_pixels(addr);

                    ctx.request_repaint();
                }
                StopReason::Count => {
                    ctx.request_repaint();
                }
                _ => {}
            }
        };
    }
}
