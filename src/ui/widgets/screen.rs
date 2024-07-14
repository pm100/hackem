use egui::{Id, Pos2, Sense, Ui, Vec2};

use crate::{debugger::debug_em::HackSystem, ui::key_lookup::lookup_key};
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;

pub(crate) static mut CURRENT_KEY: u8 = 0;
pub struct ScreenWindow {
    // hacksys: &'hack HackSystem,
    id: Id,
    paint_id: Id,
}
impl ScreenWindow {
    pub fn new() -> Self {
        Self {
            id: Id::new("Screen"),
            paint_id: Id::new("Screen"),
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
        self.paint_id = response.id;
        if response.clicked() {
            response.request_focus();
            println!("xccc{:?}", response.id);
        }

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

    pub fn draw(&mut self, ctx: &egui::Context, open: &mut bool, hacksys: &HackSystem) {
        if ctx.memory(|mem| {
            // println!("{:?} {:?}", mem.focused(), self.paint_id);
            mem.has_focus(self.paint_id)
        }) {
            ctx.input(|inp| {
                // println!("xx{:?}", inp);
                if !inp.keys_down.is_empty() {
                    println!(
                        "{:?} {:?} {} ",
                        inp.keys_down,
                        inp.modifiers,
                        inp.keys_down.len()
                    );
                    lookup_key(inp);
                    println!("xkey: {}", unsafe { CURRENT_KEY as char });
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
        };
        egui::Window::new(self.name())
            .id(self.id)
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| {
                self.ui(ui, hacksys);
            });
    }

    fn name(&self) -> &'static str {
        "Screen"
    }
}
