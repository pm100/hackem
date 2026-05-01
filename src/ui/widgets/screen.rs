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

    pub fn ui(&mut self, ui: &mut Ui, hacksys: &mut HackSystem) {
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

        // Upload texture to GPU only when the engine has written new screen pixels.
        if hacksys.engine.screen_dirty || self.texture.is_none() {
            let image = ColorImage::new(
                [SCREEN_WIDTH, SCREEN_HEIGHT],
                hacksys.engine.screen_pixels.clone(),
            );
            let opts = TextureOptions::NEAREST;
            match &mut self.texture {
                Some(tex) => tex.set(image, opts),
                None => {
                    self.texture = Some(ctx.load_texture("hack_screen", image, opts));
                }
            }
            hacksys.engine.screen_dirty = false;
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
