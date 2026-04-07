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

    pub fn ui(&mut self, ui: &mut egui::Ui, hacksys: &mut HackSystem) {
        let pc = hacksys.engine.pc;
        let dark = ui.visuals().dark_mode;

        let addr_color = ui.visuals().weak_text_color();
        let raw_color = ui.visuals().weak_text_color();
        let normal_color = ui.visuals().text_color();
        let pc_accent = if dark {
            Color32::from_rgb(255, 220, 80)
        } else {
            Color32::from_rgb(160, 100, 0)
        };

        // Center the view around PC: show display_count/2 instructions before and after.
        let half = self.display_count / 2;
        let start = pc.saturating_sub(half);

        let rows = hacksys.engine.disassemble_range(start, self.display_count);

        ui.horizontal(|ui| {
            ui.label("Lines:");
            ui.add(
                egui::DragValue::new(&mut self.display_count)
                    .range(8..=128)
                    .speed(1.0),
            );
        });
        ui.separator();

        ScrollArea::vertical().id_salt(self.id).show(ui, |ui| {
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
                        ui.label(RichText::new(pc_label).color(pc_accent));

                        // Breakpoint indicator — click to toggle
                        let bp_text = if has_bp {
                            RichText::new("●").color(Color32::RED)
                        } else {
                            RichText::new("○").color(ui.visuals().weak_text_color())
                        };
                        if ui.small_button(bp_text).clicked() {
                            if has_bp {
                                hacksys.engine.remove_breakpoint(*addr);
                            } else {
                                hacksys.engine.add_breakpoint(*addr);
                            }
                        }

                        // Address
                        let addr_text = RichText::new(format!("{:04X}", addr))
                            .monospace()
                            .color(if at_pc { pc_accent } else { addr_color });
                        ui.label(addr_text);

                        // Raw hex
                        ui.label(
                            RichText::new(format!("{:04X}", raw))
                                .monospace()
                                .color(raw_color),
                        );

                        // Mnemonic
                        let mn_color = if at_pc { pc_accent } else { normal_color };
                        ui.label(RichText::new(mnemonic).monospace().color(mn_color));

                        ui.end_row();
                    }
                });
        });
    }
}
