use egui::{Color32, Id, RichText, ScrollArea};

use crate::debugger::debug_em::HackSystem;

pub struct DataWindow {
    id: Id,
    title: String,
    start_addr_text: String,
    start_addr: u16,
    row_count: u16,
}

impl DataWindow {
    pub fn new(title: &str) -> Self {
        Self {
            id: Id::new(title),
            title: title.to_string(),
            start_addr_text: "0".to_string(),
            start_addr: 0,
            row_count: 16,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, hacksys: &HackSystem) {
        ui.horizontal(|ui| {
            ui.label("Address:");
            let resp = ui.add(
                egui::TextEdit::singleline(&mut self.start_addr_text)
                    .desired_width(60.0)
                    .hint_text("0x0000"),
            );
            if resp.lost_focus() {
                let s = self.start_addr_text.trim();
                let parsed = if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("$"))
                {
                    u16::from_str_radix(hex, 16).ok()
                } else {
                    s.parse::<u16>().ok()
                };
                if let Some(addr) = parsed {
                    self.start_addr = addr;
                }
            }

            ui.label("Rows:");
            ui.add(
                egui::DragValue::new(&mut self.row_count)
                    .range(1..=64)
                    .speed(1.0),
            );
        });
        ui.separator();

        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new(format!("{}_grid", self.title))
                .num_columns(9) // addr + 8 values
                .spacing([4.0, 2.0])
                .striped(true)
                .show(ui, |ui| {
                    for row in 0..self.row_count {
                        let base = self.start_addr.wrapping_add(row * 8);
                        ui.label(
                            RichText::new(format!("{:04X}:", base))
                                .monospace()
                                .color(Color32::GRAY),
                        );
                        for col in 0..8u16 {
                            let addr = base.wrapping_add(col);
                            if addr as usize >= hacksys.engine.ram.len() {
                                ui.label("----");
                            } else {
                                let val = hacksys.engine.ram[addr as usize];
                                ui.label(
                                    RichText::new(format!("{:04X}", val))
                                        .monospace()
                                        .color(Color32::LIGHT_GRAY),
                                )
                                .on_hover_text(format!("{} ({})", val, val as i16));
                            }
                        }
                        ui.end_row();
                    }
                });
        });
    }

    pub fn draw(&mut self, ctx: &egui::Context, open: &mut bool, hacksys: &HackSystem) {
        let title = self.title.clone();
        egui::Window::new(title)
            .id(self.id)
            .open(open)
            .default_width(380.0)
            .default_height(300.0)
            .show(ctx, |ui| {
                self.ui(ui, hacksys);
            });
    }
}
