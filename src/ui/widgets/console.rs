use egui::{Align, Context, Event, Id, Key, Modifiers, Ui};

use crate::{
    debugger::debug_em::HackSystem,
    ui::app::{AppMessage, AppWindow, UpdateType},
};

pub(crate) struct ConsoleWindow {
    text: String,
    new_line: bool,
    command_history: Vec<String>,
    history_cursor: usize,
    prompt: bool,
    // last_cursor: usize,
    force_cursor: Option<usize>,
}

impl ConsoleWindow {
    pub fn new() -> Self {
        Self {
            text: ">> ".to_string(),
            new_line: false,
            command_history: Vec::new(),
            history_cursor: usize::MAX,
            prompt: false,
            //  last_cursor: usize::MAX,
            force_cursor: None,
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
            ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                let widget = egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    // .desired_rows(10)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    //.cursor_at_end(self.new_line)
                    .id(Id::new("console_text"));
                let output = widget.show(ui);
                let mut new_cursor = None;
                // self.last_cursor = usize::MAX;
                if let Some(cursor) = output.state.cursor.char_range() {
                    let last_off = self.text.rfind('\n').unwrap_or(0);
                    // println!("last_off: {:?} cursor {}", last_off, cursor.primary.index);
                    //  self.last_cursor = cursor.primary.index;
                    if cursor.primary.index < last_off {
                        new_cursor = Some(egui::text::CCursorRange::one(egui::text::CCursor::new(
                            self.text.char_indices().count(),
                        )));
                    }
                }

                //println!("Cursor: {:?}", cursor);
                let text_edit_id = output.response.id;
                if self.new_line {
                    new_cursor = Some(egui::text::CCursorRange::one(egui::text::CCursor::new(
                        self.text.char_indices().count(),
                    )));
                    self.new_line = false;
                }
                if new_cursor.is_some() {
                    if let Some(mut state) = egui::TextEdit::load_state(ui.ctx(), text_edit_id) {
                        println!("set cursor: {:?}", new_cursor);
                        state.cursor.set_char_range(new_cursor);
                        //   self.last_cursor = new_cursor.unwrap().primary.index;
                        state.store(ui.ctx(), text_edit_id);
                        //   ui.ctx().memory_mut(|mem| mem.request_focus(text_edit_id)); // give focus back to the [`TextEdit`].
                    }
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                }
                output.response
            });
            // self.delta = 0;
            //self.new_line = false;
        });
    }
    pub fn count_and_not_consume_key(
        ctx: &Context,
        modifiers: Modifiers,
        logical_key: Key,
    ) -> usize {
        let mut count = 0usize;
        ctx.input(|input| {
            for event in &input.events {
                let is_match = matches!(
                    event,
                    Event::Key {
                        key: ev_key,
                        modifiers: ev_mods,
                        pressed: true,
                        ..
                    } if *ev_key == logical_key && ev_mods.matches_logically(modifiers)
                );

                count += is_match as usize;
            }
        });
        count
    }
    fn consume_key(ctx: &Context, modifiers: Modifiers, logical_key: Key) {
        ctx.input_mut(|inp| inp.consume_key(modifiers, logical_key));
    }
}

impl AppWindow for ConsoleWindow {
    fn draw(&mut self, ctx: &egui::Context, open: &mut bool, _hacksys: &HackSystem) {
        egui::Window::new(self.name())
            .id(Id::new(self.name()))
            .open(open)
            .vscroll(false)
            .default_height(50.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
    fn update(&mut self, msg: crate::ui::app::UpdateMessage, _hacksys: &HackSystem) {
        match msg.message {
            UpdateType::Text(output) => {
                self.text.push('\n');
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

    fn keyboard_peek(&mut self, ctx: &egui::Context, _hacksys: &HackSystem) -> Option<AppMessage> {
        //  println!("Console focus: peek");
        let cursor =
            if let Some(state) = egui::TextEdit::load_state(ctx, Id::new("console_text")) {
                println!("got state {:?}", state.cursor.char_range());
                state.cursor.char_range().unwrap().primary.index
            } else {
                0
            };
        let key_list = [(Modifiers::NONE, Key::ArrowDown),
            (Modifiers::NONE, Key::ArrowUp),
            (Modifiers::NONE, Key::Enter),
            (Modifiers::NONE, Key::ArrowLeft),
            (Modifiers::NONE, Key::Backspace)];
        let mut matched_key = usize::MAX;
        for (idx, key) in key_list.iter().enumerate() {
            if Self::count_and_not_consume_key(ctx, key.0, key.1) > 0 {
                matched_key = idx;
                break;
            }
        }

        if matched_key != usize::MAX {
            // println!("Console focus: key: {:?}", key_list[matched_key]);
            let mut eatit = false;
            let return_value = match key_list[matched_key] {
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
                    eatit = true;
                    None
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
                    eatit = true;
                    None
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
                    eatit = true;
                    Some(AppMessage::ConsoleCommand(last))
                }
                (Modifiers::NONE, Key::ArrowLeft) | (Modifiers::NONE, Key::Backspace) => {
                    let last_off = self.text.rfind('\n').unwrap_or(usize::MAX);
                    let last_off = if last_off == usize::MAX {
                        -1
                    } else {
                        last_off as isize
                    };
                    println!("last_off: {:?} cursor {}", last_off, cursor);
                    if cursor < (last_off + 5) as usize {
                        //   self.force_cursor = Some(last_off + 4);
                        eatit = true;
                    }

                    None
                }
                _ => None,
            };
            if eatit {
                let key = key_list[matched_key];
                Self::consume_key(ctx, key.0, key.1);
            }
            return return_value;
        };
        None
    }
    fn id(&self) -> Id {
        Id::new("console_text")
    }
}
