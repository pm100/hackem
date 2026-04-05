use crate::debugger::debug_em::HackSystem;

pub struct CpuWindow {}

impl CpuWindow {
    pub fn new() -> Self {
        Self {}
    }

    pub fn ui(&self, ui: &mut egui::Ui, hacksys: &HackSystem) {
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
            ui.end_row();
            for i in 0..16 {
                ui.label(format!("R{}: ", i));
                ui.label(format!("0x{:04X}", hacksys.engine.ram[i]));
                ui.end_row();
            }
        });
    }
}
