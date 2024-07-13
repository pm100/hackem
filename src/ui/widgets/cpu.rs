use egui::Id;

use crate::{debugger::debug_em::HackSystem, ui::app::AppMessage};

pub struct CpuWindow {}

impl CpuWindow {
    pub fn new() -> Self {
        Self {}
    }
    fn ui(&self, ui: &mut egui::Ui, hacksys: &HackSystem) {
        // let (pc, a, d) = self.hacksys.get_registers(
        let (pc, a, d) = hacksys.engine.get_registers();
        egui::Grid::new("CPU Status").num_columns(2).show(ui, |ui| {
            ui.label("PC: ");
            ui.label(format!("0x{:04X}", pc));
            ui.end_row();
            ui.label("A: ");
            ui.label(format!("0x{:04X}", a));
            ui.end_row();
            ui.label("D: ");
            ui.label(format!("0x{:04X}", d));
            ui.end_row();
            ui.label("Speed: ");
            let speed = hacksys.engine.speed;
            if speed > 0.0 {
                ui.label(format!("{}", speed));
            } else {
                ui.label("Stopped");
            }
            // ui.label(format!("0x{:04X}", d));
            ui.end_row();
            ui.label("Elapsed: ");
            //  ui.label(format!("{}", hacksys.elapsed.as_secs()));
            ui.end_row();
        });
    }

    pub fn name(&self) -> &'static str {
        "Cpu"
    }

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        hacksys: &HackSystem,
    ) -> Option<AppMessage> {
        egui::Window::new(self.name())
            .id(Id::new(self.name()))
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| {
                self.ui(ui, hacksys);
            });
        None
    }
}
