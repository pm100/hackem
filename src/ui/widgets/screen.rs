use egui::{ColorImage, Id, Sense, TextureHandle, TextureOptions, Ui, Vec2};

use crate::{debugger::debug_em::HackSystem, ui::key_lookup::lookup_key};
const SCREEN_WIDTH: usize = 512;
const SCREEN_HEIGHT: usize = 256;

pub(crate) static mut CURRENT_KEY: u8 = 0;
pub struct ScreenWindow {
    paint_id: Id,
    texture: Option<TextureHandle>,
}
impl ScreenWindow {
    pub fn new() -> Self {
        Self {
            paint_id: Id::new("Screen"),
            texture: None,
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

        let (fg, bg) = if ui.style().visuals.dark_mode {
            (egui::Color32::WHITE, egui::Color32::BLACK)
        } else {
            (egui::Color32::BLACK, egui::Color32::WHITE)
        };

        // Build screen image from RAM
        let screen = &hacksys.engine.ram[0x4000..0x6000];
        let mut pixels = vec![bg; SCREEN_WIDTH * SCREEN_HEIGHT];
        for row in 0..SCREEN_HEIGHT {
            for col in 0..SCREEN_WIDTH / 16 {
                let word = screen[row * (SCREEN_WIDTH / 16) + col];
                if word != 0 {
                    for i in 0..16u16 {
                        if word & (1 << i) != 0 {
                            pixels[row * SCREEN_WIDTH + col * 16 + i as usize] = fg;
                        }
                    }
                }
            }
        }

        let image = ColorImage::new([SCREEN_WIDTH, SCREEN_HEIGHT], pixels);

        // Update or create the texture
        let opts = TextureOptions::NEAREST;
        match &mut self.texture {
            Some(tex) => tex.set(image, opts),
            None => {
                self.texture = Some(ctx.load_texture("hack_screen", image, opts));
            }
        }

        if let Some(tex) = &self.texture {
            let size = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
            let response = ui.add(
                egui::Image::new((tex.id(), size))
                    .fit_to_exact_size(size)
                    .sense(Sense::click()),
            );
            self.paint_id = response.id;
            if response.clicked() {
                response.request_focus();
            }
        }
    }
}
