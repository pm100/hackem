#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{collections::BTreeMap, path::Path};

use eframe::egui;
use egui::{Id, Ui, WidgetText};
use egui_dock::{DockArea, DockState, Style, TabViewer};

use crate::{
    debugger::debug_em::HackSystem,
    ui::app::{AppWindow, UpdateType},
};

/// We identify tabs by the title of the file we are editing.
type Title = String;

struct Buffers {
    buffers: BTreeMap<Title, String>,
}

impl TabViewer for Buffers {
    type Tab = Title;

    fn title(&mut self, title: &mut Title) -> WidgetText {
        WidgetText::from(&*title)
    }

    fn ui(&mut self, ui: &mut Ui, title: &mut Title) {
        let text = self.buffers.entry(title.clone()).or_default();
        egui::TextEdit::multiline(text)
            .desired_width(f32::INFINITY)
            .font(egui::TextStyle::Monospace)
            .code_editor()
            .show(ui);
    }
}

pub struct FilesWindow {
    buffers: Buffers,
    tree: DockState<String>,
}

impl Default for FilesWindow {
    fn default() -> Self {
        let mut buffers = BTreeMap::default();
        buffers.insert(
            "file1".to_owned(),
            std::fs::read_to_string(Path::new("c:\\work\\hackem\\cargo.toml")).unwrap(),
        );
        // buffers.insert("LICENSE".to_owned(), include_str!("../LICENSE").to_owned());
        // buffers.insert(
        //     "README.md".to_owned(),
        //     include_str!("../README.md").to_owned(),
        // );

        let tree = DockState::new(vec!["file1".to_owned()]);

        Self {
            buffers: Buffers { buffers },
            tree,
        }
    }
}
impl AppWindow for FilesWindow {
    fn draw(&mut self, ctx: &egui::Context, open: &mut bool, _hacksys: &HackSystem) {
        egui::Window::new(self.name())
            .id(Id::new(self.name()))
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }

    fn name(&self) -> &'static str {
        "Files"
    }
    fn update(&mut self, msg: crate::ui::app::UpdateMessage, _hacksys: &HackSystem) {
        match msg.message {
            UpdateType::Text(_output) => {
                // self.text.push_str("\n");
                // self.text.push_str(output.as_str());
                // self.text.push_str("\n>> ");
                // self.new_line = true;
            } //_ => {}
        }
    }
}
impl FilesWindow {
    pub fn new() -> Self {
        Self::default()
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::SidePanel::left("documents").show_inside(ui, |ui| {
            //});
            for title in self.buffers.buffers.keys() {
                let tab_location = self.tree.find_tab(title);
                let is_open = tab_location.is_some();
                if ui.selectable_label(is_open, title).clicked() {
                    if let Some(tab_location) = tab_location {
                        self.tree.set_active_tab(tab_location);
                    } else {
                        // Open the file for editing:
                        self.tree.push_to_focused_leaf(title.clone());
                    }
                }
            }
        });

        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ui.ctx().style().as_ref()))
            .show_inside(ui, &mut self.buffers);
    }
    // fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    //     egui::Window::new("xxxx")
    //         .default_height(500.0)
    //         .show(ctx, |ui| {
    //             egui::SidePanel::left("documents").show_inside(ui, |ui| {
    //                 //});
    //                 for title in self.buffers.buffers.keys() {
    //                     let tab_location = self.tree.find_tab(title);
    //                     let is_open = tab_location.is_some();
    //                     if ui.selectable_label(is_open, title).clicked() {
    //                         if let Some(tab_location) = tab_location {
    //                             self.tree.set_active_tab(tab_location);
    //                         } else {
    //                             // Open the file for editing:
    //                             self.tree.push_to_focused_leaf(title.clone());
    //                         }
    //                     }
    //                 }
    //             });

    //             DockArea::new(&mut self.tree)
    //                 .style(Style::from_egui(ctx.style().as_ref()))
    //                 .show_inside(ui, &mut self.buffers);
    //         });
    // }
}
