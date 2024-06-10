use egui::{vec2, Painter, Pos2, Rect, Sense, Ui, Vec2};
use engine::HackEngine;
/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HackEmulator {
    screen_ram: Vec<u16>,
    #[serde(skip)]
    engine: HackEngine<'static>,
}
const SCREEN_WIDTH: usize = 256;
const SCREEN_HEIGHT: usize = 256;
impl Default for HackEmulator {
    fn default() -> Self {
        Self {
            screen_ram: vec![0; SCREEN_WIDTH / 16 * SCREEN_HEIGHT],
            engine: HackEngine::new(),
        }
    }
}

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

        let mut app = Self::default();
        app.screen_ram[0] = 0xFFFF;
        app.screen_ram[20] = 0x1;
        app.screen_ram[600] = 0x5555;
        println!("new {}", app.screen_ram[0]);
        app
    }

    fn draw_pixel(area: &Rect, painter: &Painter, x: i32, y: i32) {
        let rect_points =
            egui::Rect::from_min_size(Pos2::new(x as f32, y as f32), Vec2::splat(2 as f32));
        painter.rect_filled(rect_points, 0.0, egui::Color32::BLACK);
    }

    fn draw_cpu_state(&mut self, ui: &mut Ui) {
        egui::Grid::new("CPU Status").num_columns(2).show(ui, |ui| {
            ui.label("PC: ");
            ui.label(format!("0x{:04X}", self.engine.pc));
            ui.end_row();
            ui.label("A: ");
            ui.label(format!("0x{:04X}", self.engine.a));
            ui.end_row();
            ui.label("D: ");
            ui.label(format!("0x{:04X}", self.engine.d));
            ui.end_row();
        });
    }

    fn draw_screen(&mut self, ui: &mut Ui) {
        let color = if ui.style().visuals.dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        println!("draw_screen {}", self.screen_ram[0]);
        let draw_area_size =
            Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32) + Vec2::splat(2.0);
        let (response, painter) = ui.allocate_painter(draw_area_size, Sense::hover());

        let top_left = Pos2::new(response.rect.min.x * 1.0, response.rect.min.y * 1.0).ceil();
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH / 16 {
                let pixels = self.screen_ram[row * SCREEN_WIDTH / 16 + col];

                if pixels != 0 {
                    println!("pixels: {:x}", pixels);
                    let y = top_left.y + ((row) as f32);

                    for i in 0..16 {
                        if pixels & (1 << i) != 0 {
                            let x = top_left.x + ((col + i) as f32);
                            let rect_points =
                                egui::Rect::from_min_size(Pos2::new(x, y), Vec2::splat(2.0));
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
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Load Binary").clicked() {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                self.engine.load_file(path.to_str().unwrap());
                            }
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_dark_light_mode_buttons(ui);
            });
            if ui.button("Step").clicked() {
                self.engine.execute_instruction();
            }
        });

        egui::Window::new("screen")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_screen(ui));
        egui::Window::new("CPU")
            .default_height(500.0)
            .show(ctx, |ui| self.draw_cpu_state(ui));
        // egui::TopBottomPanel::top("bottom_panel").show(ctx, |ui| {
        //     ui.horizontal(|ui| {
        //         ui.label("This is the bottom panel.");
        //         ui.label("You can put some widgets here.");
        //         egui::SidePanel::left("side_panel3")
        //             .default_width(200.0)
        //             .show_inside(ui, |ui| {
        //                 ui.heading("Side Panel3");

        //                 ui.label("This is a sidxxe panel.");
        //                 ui.label("You can put some side widgets here.");
        //             });
        //         egui::SidePanel::left("side_panel4")
        //             .default_width(200.0)
        //             .show_inside(ui, |ui| {
        //                 //  ui.heading("Side Panel4");

        //                 ui.label("This is a sidxxe panel.");
        //                 ui.label("You can put some side widgets here.");
        //             });
        //     });
        // });
        // egui::SidePanel::left("side_panel")
        //     .default_width(200.0)
        //     .show(ctx, |ui| {
        //         ui.heading("Side Panel");

        //         ui.label("This is a side panel.");
        //         ui.label("You can put some side widgets here.");
        //     });
        // egui::SidePanel::left("side_panel2")
        //     .default_width(200.0)
        //     .show(ctx, |ui| {
        //         ui.heading("Side Panel2");

        //         ui.label("This is a side panel2.");
        //         ui.label("You can put some side widgets here.");
        //     });
        // egui::CentralPanel::default().show(ctx, |ui| {
        //     self.draw_screen(ui);
        //     //self.draw_test(ui);
        // });
    }
}
