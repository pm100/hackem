use egui::{Id, Pos2, Sense, Ui, Vec2};

use crate::{
    debugger::debug_em::HackSystem,
    ui::app::{AppWindow},
};
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;
pub struct ScreenWindow {
    // hacksys: &'hack HackSystem,
}
impl ScreenWindow {
    pub fn new() -> Self {
        Self {}
    }

    fn draw(&mut self, ui: &mut Ui, hacksys: &HackSystem) {
        let color = if ui.style().visuals.dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        let draw_area_size =
            Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32) + Vec2::splat(1.0);
        let (response, painter) = ui.allocate_painter(draw_area_size, Sense::hover());

        let top_left = Pos2::new(response.rect.min.x * 1.0, response.rect.min.y * 1.0).ceil();
        let screen = &hacksys.engine.ram[0x4000..0x6000];
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

impl AppWindow for ScreenWindow {
    fn draw(&mut self, ctx: &egui::Context, open: &mut bool, hacksys: &HackSystem) {
        egui::Window::new(self.name())
            .id(Id::new(self.name()))
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| {
                self.draw(ui, hacksys);
            });
    }

    fn name(&self) -> &'static str {
        "Screen"
    }
}
