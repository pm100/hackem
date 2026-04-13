use crate::{
    debugger::{debug_em::HackSystem, shell::Shell},
    emulator::engine::StopReason,
};

use egui_console::{ConsoleBuilder, ConsoleEvent, ConsoleWindow};
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use thiserror::Error;
use web_time::Duration;

use super::widgets::{code::CodeWindow, cpu::CpuWindow, data::DataWindow, screen::ScreenWindow};

#[derive(Debug, Clone, PartialEq)]
pub enum AppTab {
    Console,
    Code,
    Cpu,
    Data1,
    Data2,
    Screen,
}

impl std::fmt::Display for AppTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppTab::Console => write!(f, "Console"),
            AppTab::Code => write!(f, "Code"),
            AppTab::Cpu => write!(f, "CPU"),
            AppTab::Data1 => write!(f, "Data 1"),
            AppTab::Data2 => write!(f, "Data 2"),
            AppTab::Screen => write!(f, "Screen"),
        }
    }
}

struct AppTabViewer<'a> {
    hacksys: &'a mut HackSystem,
    console_window: &'a mut ConsoleWindow,
    console_response: &'a mut ConsoleEvent,
    code_window: &'a mut CodeWindow,
    cpu_window: &'a mut CpuWindow,
    data_window1: &'a mut DataWindow,
    data_window2: &'a mut DataWindow,
    screen_window: &'a mut ScreenWindow,
    
}

impl<'a> TabViewer for AppTabViewer<'a> {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        format!("{}", tab).into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            AppTab::Console => {
                *self.console_response = self.console_window.draw(ui);
            }
            AppTab::Code => {
                self.code_window.ui(ui, self.hacksys);
            }
            AppTab::Cpu => {
                self.cpu_window.ui(ui, self.hacksys);
            }
            AppTab::Data1 => {
                self.data_window1.ui(ui, self.hacksys);
            }
            AppTab::Data2 => {
                self.data_window2.ui(ui, self.hacksys);
            }
            AppTab::Screen => {
                self.screen_window.ui(ui, self.hacksys);
            }
        }
    }
}

pub struct HackEgui {
    pub(crate) hacksys: HackSystem,
    running: bool,
    console_window: ConsoleWindow,
    screen_window: ScreenWindow,
    cpu_window: CpuWindow,
    code_window: CodeWindow,
    data_window1: DataWindow,
    data_window2: DataWindow,
    shell: Shell,
    dock_state: DockState<AppTab>,
    puts_buffer:String
}

#[derive(Debug, Error, PartialEq)]
pub enum RuntimeError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Invalid RAM read address {0}")]
    InvalidReadAddress(u16),
    #[error("Invalid RAM write address {0}")]
    InvalidWriteAddress(u16),
    #[error("Invalid instruction address {0}")]
    InvalidPC(u16),
}

impl HackEgui {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Build initial dock layout:
        //   Left 60% = Code
        //   Right 40% top = [CPU, Data1, Data2, Screen] tabs
        //   Right 40% bottom = Console
        let mut dock_state = DockState::new(vec![AppTab::Code]);
        let surface = dock_state.main_surface_mut();
        let [_left, right] = surface.split_right(NodeIndex::root(), 0.6, vec![AppTab::Cpu]);
        surface.push_to_focused_leaf(AppTab::Data1);
        surface.push_to_focused_leaf(AppTab::Data2);
        surface.push_to_focused_leaf(AppTab::Screen);
        let [_top, _bottom] = surface.split_below(right, 0.5, vec![AppTab::Console]);

        let mut console_window = ConsoleBuilder::new()
            .prompt(">> ")
            .history_size(50)
            .tab_quote_character('\"')
            .build();

        // Load persisted history
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(home) = dirs::home_dir() {
            let history_path = home.join(".hackem_history");
            if let Ok(contents) = std::fs::read_to_string(&history_path) {
                // store so lifetime covers the load_history call
                console_window.load_history(contents.lines());
            }
        }

        Self {
            hacksys: HackSystem::new(),
            running: false,
            console_window,
            screen_window: ScreenWindow::new(),
            cpu_window: CpuWindow::new(),
            code_window: CodeWindow::new(),
            data_window1: DataWindow::new("Data 1"),
            data_window2: DataWindow::new("Data 2"),
            shell: Shell::new(),
            dock_state,
            puts_buffer:String::new()
        }
    }

    fn save_history(&self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(home) = dirs::home_dir() {
            let history_path = home.join(".hackem_history");
            let history: Vec<String> = self.console_window.get_history().into_iter().collect();
            let _ = std::fs::write(history_path, history.join("\n"));
        }
    }

    fn console_write(&mut self, msg: &str) {
        self.console_window.write(msg);
        self.console_window.prompt();
    }
}

impl eframe::App for HackEgui {
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {}

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                ui.menu_button("File", |ui| {
                    if ui.button("Load Binary").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Hack binary", &["hackem", "hack", "hx"])
                            .pick_file()
                        {
                            let file_name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            match std::fs::read_to_string(&path) {
                                Err(e) => self
                                    .console_write(&format!("Error reading {}: {}", file_name, e)),
                                Ok(bin) => match self.hacksys.engine.load_file(&bin) {
                                    Err(e) => self.console_write(&format!("Load error: {}", e)),
                                    Ok(()) => self.console_write(&format!(
                                        "Loaded {}  ROM: {} words  RAM: {} words",
                                        file_name,
                                        self.hacksys.engine.rom_words_loaded,
                                        self.hacksys.engine.ram_words_loaded
                                    )),
                                },
                            }
                        }
                        ui.close();
                    }

                    if ui.button("Quit").clicked() {
                        self.save_history();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                egui::global_theme_preference_buttons(ui);
            });
        });

        let mut console_response = ConsoleEvent::None;

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut viewer = AppTabViewer {
                hacksys: &mut self.hacksys,
                console_window: &mut self.console_window,
                console_response: &mut console_response,
                code_window: &mut self.code_window,
                cpu_window: &mut self.cpu_window,
                data_window1: &mut self.data_window1,
                data_window2: &mut self.data_window2,
                screen_window: &mut self.screen_window,
            };
            DockArea::new(&mut self.dock_state)
                .style(Style::from_egui(ui.style()))
                .show_inside(ui, &mut viewer);
        });

        if let ConsoleEvent::Command(cmd) = console_response {
            if let Ok(response) = self.shell.execute_message(&cmd, &mut self.hacksys) {
                match response.as_str() {
                    "__go__" => {
                        self.running = true;
                    }
                    "__quit__" => {
                        self.save_history();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    _ => {
                        if !response.is_empty() {
                            self.console_write(&response);
                        } else {
                            self.console_window.prompt();
                        }
                    }
                }
            }
        }

        if self.running {
            let pc = self.hacksys.engine.pc;
            let stop = self
                .hacksys
                .engine
                .execute_instructions(Duration::from_millis(50));
            match stop {
                Ok(reason) => match reason {
                    StopReason::SysHalt => {
                        self.running = false;
                        self.console_write("SysHalt");
                    }
                    StopReason::HardLoop => {
                        self.running = false;
                        self.console_write(&format!("Hard loop at 0x{:04X}", pc));
                    }
                    StopReason::BreakPoint => {
                        self.running = false;
                        self.console_write(&format!(
                            "Breakpoint hit at 0x{:04X}",
                            self.hacksys.engine.pc
                        ));
                    }
                    StopReason::WatchPoint => {
                        self.running = false;
                        let addr = self.hacksys.engine.triggered_watchpoint.unwrap_or(0);
                        self.console_write(&format!("Watchpoint hit at 0x{:04X}", addr));
                    }
                    StopReason::RefreshUI => {
                        ctx.request_repaint();
                        if self.hacksys.engine.ram[32767] != 0{
                            let ch = self.hacksys.engine.ram[32767] as u8;
                            self.hacksys.engine.ram[32767] = 0;
                            if ch == 13 || ch == 10{
                                self.console_write(&self.puts_buffer.clone());
                                self.puts_buffer.clear();
                            }
                            else{
                                self.puts_buffer.push(ch as char);
                            }
                        }
                    }
                },
                Err(err) => {
                    self.running = false;
                    self.console_write(&format!("Error: {}", err));
                }
            }
        }
    }
}
