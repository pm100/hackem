use egui::{Color32, Id, RichText, ScrollArea};

use crate::debugger::debug_em::HackSystem;

pub struct CodeWindow {
    id: Id,
    /// How many instructions to show
    display_count: u16,
}

impl CodeWindow {
    pub fn new() -> Self {
        Self {
            id: Id::new("code_window"),
            display_count: 32,
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, hacksys: &mut HackSystem) {
        let pc = hacksys.engine.pc;

        // Center the view around PC: show display_count/2 instructions before and after.
        let half = self.display_count / 2;
        let start = pc.saturating_sub(half);

        let rows = hacksys.engine.disassemble_range(start, self.display_count);

        ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("code_grid")
                .num_columns(5)
                .spacing([4.0, 2.0])
                .striped(true)
                .show(ui, |ui| {
                    for (addr, raw, mnemonic) in &rows {
                        let at_pc = *addr == pc;
                        let has_bp = hacksys.engine.break_points.contains_key(addr);

                        // PC indicator
                        let pc_label = if at_pc { "▶" } else { " " };
                        ui.label(RichText::new(pc_label).color(Color32::YELLOW));

                        // Breakpoint indicator — click to toggle
                        let bp_text = if has_bp {
                            RichText::new("●").color(Color32::RED)
                        } else {
                            RichText::new("○").color(Color32::DARK_GRAY)
                        };
                        if ui.small_button(bp_text).clicked() {
                            if has_bp {
                                hacksys.engine.remove_breakpoint(*addr);
                            } else {
                                hacksys.engine.add_breakpoint(*addr);
                            }
                        }

                        // Address
                        let addr_text =
                            RichText::new(format!("{:04X}", addr))
                                .monospace()
                                .color(if at_pc {
                                    Color32::YELLOW
                                } else {
                                    Color32::GRAY
                                });
                        ui.label(addr_text);

                        // Raw hex
                        ui.label(
                            RichText::new(format!("{:04X}", raw))
                                .monospace()
                                .color(Color32::DARK_GRAY),
                        );

                        // Mnemonic
                        let mn_color = if at_pc {
                            Color32::WHITE
                        } else {
                            Color32::LIGHT_GRAY
                        };
                        ui.label(RichText::new(mnemonic).monospace().color(mn_color));

                        ui.end_row();
                    }
                });
        });
    }

    pub fn draw(&mut self, ctx: &egui::Context, open: &mut bool, hacksys: &mut HackSystem) {
        egui::Window::new("Code")
            .id(self.id)
            .open(open)
            .default_width(350.0)
            .default_height(500.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Lines:");
                    ui.add(
                        egui::DragValue::new(&mut self.display_count)
                            .range(8..=128)
                            .speed(1.0),
                    );
                });
                ui.separator();
                self.ui(ui, hacksys);
            });
    }
}
