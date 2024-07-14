use egui::{Id, Pos2, Sense, Ui, Vec2};

use crate::{debugger::debug_em::HackSystem, ui::app::AppMessage};
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;
pub struct ScreenWindow {
    // hacksys: &'hack HackSystem,
    id: Id,
}
impl ScreenWindow {
    pub fn new() -> Self {
        Self {
            id: Id::new("Screen"),
        }
    }

    fn ui(&mut self, ui: &mut Ui, hacksys: &HackSystem) {
        let color = if ui.style().visuals.dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        let draw_area_size =
            Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32) + Vec2::splat(1.0);
        let sense = Sense::click(); // Sense::focusable_noninteractive();
        let (response, painter) = ui.allocate_painter(draw_area_size, sense);

        let top_left = Pos2::new(response.rect.min.x * 1.0, response.rect.min.y * 1.0).ceil();
        // response.request_focus();
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

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        hacksys: &HackSystem,
    ) -> Option<AppMessage> {
        egui::Window::new(self.name())
            .id(self.id)
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| {
                self.ui(ui, hacksys);
            });
        ctx.memory_mut(|mem| mem.interested_in_focus(self.id));

        None
    }

    fn name(&self) -> &'static str {
        "Screen"
    }
}
