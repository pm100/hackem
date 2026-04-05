use egui::{Id, Pos2, Sense, Ui, Vec2};

use crate::{debugger::debug_em::HackSystem, ui::key_lookup::lookup_key};
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;

pub(crate) static mut CURRENT_KEY: u8 = 0;
pub struct ScreenWindow {
    paint_id: Id,
}
impl ScreenWindow {
    pub fn new() -> Self {
        Self {
            paint_id: Id::new("Screen"),
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, hacksys: &HackSystem) {
        let ctx = ui.ctx().clone();
        // Handle keyboard input when screen is focused
        if ctx.memory(|mem| mem.has_focus(self.paint_id)) {
            ctx.input(|inp| {
                if !inp.keys_down.is_empty() {
                    lookup_key(inp);
                } else {
                    unsafe {
                        CURRENT_KEY = 0;
                    }
                }
                if let Some(egui::Event::Text(text)) = inp.events.first() {
                    unsafe {
                        CURRENT_KEY = text.chars().next().unwrap() as u8;
                    }
                }
            });
        }

        let color = if ui.style().visuals.dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        let draw_area_size =
            Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32) + Vec2::splat(1.0);
        let sense = Sense::click();
        let (response, painter) = ui.allocate_painter(draw_area_size, sense);
        self.paint_id = response.id;
        if response.clicked() {
            response.request_focus();
        }

        let top_left = Pos2::new(response.rect.min.x, response.rect.min.y).ceil();
        let screen = &hacksys.engine.ram[0x4000..0x6000];
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH / 16 {
                let pixels = screen[row * SCREEN_WIDTH / 16 + col];
                if pixels != 0 {
                    let y = top_left.y + (row as f32);
                    for i in 0..16 {
                        if pixels & (1 << i) != 0 {
                            let x = top_left.x + ((col * 16 + i) as f32);
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
