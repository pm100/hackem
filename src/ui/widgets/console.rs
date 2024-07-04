use egui::{debug_text::print, Id, Key, Modifiers};

use crate::ui::app::{AppMessage, MyWidget, UpdateMessage, UpdateType};

pub(crate) struct ConsoleWindow {
    text: String,
    new_line: bool,
    command_history: Vec<String>,
    history_cursor: usize,
    prompt: bool,
}

impl ConsoleWindow {
    pub fn new() -> Self {
        Self {
            text: ">> ".to_string(),
            new_line: false,
            command_history: Vec::new(),
            history_cursor: usize::MAX,
            prompt: false,
        }
    }
    pub fn prompt(&mut self) {
        self.prompt = true;
        self.text.push_str("\n>> ");
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        // ui.label("Console");
        //  ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            let widget = egui::TextEdit::multiline(&mut self.text)
                .font(egui::TextStyle::Monospace) // for cursor height
                .code_editor()
                .desired_rows(10)
                .lock_focus(true)
                .desired_width(f32::INFINITY)
                //.cursor_at_end(self.new_line)
                .id(Id::new("console_text"))
                .show(ui);

            let cursor = widget.state.cursor.char_range().unwrap();
            let last_off = self.text.rfind('\n').unwrap_or(0);
            if cursor.primary.index < last_off {
                self.new_line = true;
            }
            println!("Cursor: {:?}", cursor);
            let text_edit_id = widget.response.id;
            if self.new_line {
                if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                    let ccursor = egui::text::CCursor::new(self.text.char_indices().count());
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::one(ccursor)));
                    state.store(ui.ctx(), text_edit_id);
                    //   ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id)); // give focus back to the [`TextEdit`].
                }
                // self.delta = 0;
                self.new_line = false;
            }
        });
    }
}

impl MyWidget for ConsoleWindow {
    fn draw(&mut self, ctx: &egui::Context, open: &mut bool) {
        egui::Window::new(self.name())
            .id(Id::new(self.name()))
            .open(open)
            .default_height(500.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
    fn update(&mut self, msg: crate::ui::app::UpdateMessage) {
        match msg.message {
            UpdateType::Text(output) => {
                self.text.push_str("\n");
                self.text.push_str(output.as_str());
                self.text.push_str("\n>> ");
                self.new_line = true;
            }
            _ => {}
        }
    }
    fn name(&self) -> &'static str {
        "Console Window"
    }
    fn keyboard_peek(&mut self, ctx: &egui::Context) -> Option<AppMessage> {
        //  println!("Console focus: peek");

        let key_list = vec![
            (Modifiers::NONE, Key::ArrowDown),
            (Modifiers::NONE, Key::ArrowUp),
            (Modifiers::NONE, Key::Enter),
        ];
        let mut matched_key = usize::MAX;
        ctx.input_mut(|inp| {
            for (idx, key) in key_list.iter().enumerate() {
                if inp.consume_key(key.0, key.1) {
                    matched_key = idx;
                    break;
                }
            }
        });
        if matched_key != usize::MAX {
            // println!("Console focus: key: {:?}", key_list[matched_key]);

            match key_list[matched_key] {
                (Modifiers::NONE, Key::ArrowDown) => {
                    //self.delta = 1;
                    let lines = self.text.lines();
                    let last = lines.last().unwrap_or("").strip_prefix(">> ").unwrap_or("");
                    self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
                    //  self.text.push_str("\n>> ");
                    if self.history_cursor < self.command_history.len() - 1 {
                        self.history_cursor += 1;
                    }
                    self.text
                        .push_str(self.command_history[self.history_cursor].as_str());
                    return None;
                }
                (Modifiers::NONE, Key::ArrowUp) => {
                    println!("Console focus: ArrowUp");
                    //self.delta = 1;
                    if self.command_history.is_empty() {
                        return None;
                    }
                    self.history_cursor = self.history_cursor.saturating_sub(1);
                    let lines = self.text.lines();
                    let last = lines.last().unwrap_or("").strip_prefix(">> ").unwrap_or("");
                    self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
                    //  self.text.push_str("\n>> ");
                    self.text
                        .push_str(self.command_history[self.history_cursor].as_str());

                    return None;
                }
                (Modifiers::NONE, Key::Enter) => {
                    println!("Console focus: Enter");
                    let lines = self.text.lines();
                    let last = lines
                        .last()
                        .unwrap_or("")
                        .strip_prefix(">> ")
                        .unwrap_or("")
                        .to_string();
                    println!("command: {}", last);
                    self.command_history.push(last.clone());
                    self.history_cursor = self.command_history.len() - 1;
                    // self.text.push_str("\n>> ");
                    self.new_line = true;
                    return Some(AppMessage::ConsoleCommand(last));
                }
                _ => {}
            }
        };
        None
    }
    fn id(&self) -> Id {
        Id::new("console_text")
    }
}
