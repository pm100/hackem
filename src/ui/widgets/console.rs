use std::str::Lines;

use egui::{
    text::CCursorRange, Align, Context, Event, EventFilter, Id, Key, Modifiers, TextEdit, Ui,
};

use crate::{
    debugger::debug_em::HackSystem,
    ui::app::{AppMessage, UpdateMessage, UpdateType},
};

pub(crate) struct ConsoleWindow {
    text: String,
    new_line: bool,
    command_history: Vec<String>,
    history_cursor: isize,
    prompt: String,
    prompt_len: usize,
    id: Id,
    search_mode: Option<usize>,
    save_prompt: String,
    search_partial: String,
}

impl ConsoleWindow {
    pub fn new(prompt: &str) -> Self {
        Self {
            text: prompt.to_string(),
            new_line: false,
            command_history: Vec::new(),
            history_cursor: -1,
            prompt: prompt.to_string(),
            prompt_len: prompt.chars().count(),
            search_mode: None,
            id: Id::new("console_text"),
            save_prompt: prompt.to_string(),
            search_partial: "".to_string(),
        }
    }

    pub fn load_history(&mut self, history: Lines<'_>) {
        self.command_history = history.into_iter().map(|s| s.to_string()).collect();
        self.history_cursor = self.command_history.len() as isize - 1;
    }
    pub fn get_history(&self) -> Vec<String> {
        self.command_history.clone()
    }
    pub fn set_prompt(&mut self, prompt: &str) {
        self.prompt = prompt.to_string();
    }
    fn cursor_at_end(&self) -> CCursorRange {
        egui::text::CCursorRange::one(egui::text::CCursor::new(self.text.chars().count()))
    }
    fn ui(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_sized(ui.available_size(), |ui: &mut Ui| {
                let widget = egui::TextEdit::multiline(&mut self.text)
                    .font(egui::TextStyle::Monospace)
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .id(self.id);
                let output = widget.show(ui);
                let mut new_cursor = None;

                // if there was mouse movement, scrolling etc
                // cursor might not be on the last line

                if let Some(cursor) = output.state.cursor.char_range() {
                    let last_off = self.text.rfind('\n').unwrap_or(0);
                    if cursor.primary.index < last_off + self.prompt_len {
                        new_cursor = Some(self.cursor_at_end());
                    }
                }

                // we need a new line (user pressed enter)
                if self.new_line {
                    new_cursor = Some(self.cursor_at_end());
                    self.new_line = false;
                }

                let text_edit_id = output.response.id;
                if new_cursor.is_some() {
                    if let Some(mut state) = TextEdit::load_state(ui.ctx(), text_edit_id) {
                        state.cursor.set_char_range(new_cursor);
                        state.store(ui.ctx(), text_edit_id);
                    }
                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                }
                output.response
            });
        });
    }
    fn get_last_line(&self) -> &str {
        self.text
            .lines()
            .last()
            .unwrap_or("")
            .strip_prefix(&self.prompt)
            .unwrap_or("")
    }

    fn consume_key(ctx: &Context, modifiers: Modifiers, logical_key: Key) {
        ctx.input_mut(|inp| inp.consume_key(modifiers, logical_key));
    }

    fn handle_key(
        &mut self,
        key: &Key,
        modifiers: Modifiers,
        cursor: usize,
    ) -> (bool, Option<String>) {
        let mut eatit = false;
        println!("Console focus: handle key: {:?} {:?}", modifiers, key);
        let return_value = match (modifiers, key) {
            (Modifiers::NONE, Key::ArrowDown) => {
                let last = self.get_last_line();
                self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
                if self.history_cursor < self.command_history.len() as isize - 1 {
                    self.history_cursor += 1;

                    self.text
                        .push_str(self.command_history[self.history_cursor as usize].as_str());
                }
                eatit = true;
                None
            }
            (Modifiers::NONE, Key::ArrowUp) => {
                if self.command_history.is_empty() {
                    return (true, None);
                }
                if self.history_cursor >= 0 {
                    // self.history_cursor = self.history_cursor.saturating_sub(1);
                    let last = self.get_last_line();
                    self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
                    self.text
                        .push_str(self.command_history[self.history_cursor as usize].as_str());
                    self.history_cursor -= 1;
                }
                eatit = true;
                None
            }
            (Modifiers::NONE, Key::Enter) => {
                let last = self.get_last_line().to_string();
                println!("command: {}", last);
                self.history_cursor = self.command_history.len() as isize;
                self.command_history.push(last.clone());

                self.new_line = true;
                eatit = true;
                Some(last)
            }
            (Modifiers::NONE, Key::ArrowLeft) | (Modifiers::NONE, Key::Backspace) => {
                let last_off = self.text.rfind('\n').unwrap_or(usize::MAX);
                let last_off = if last_off == usize::MAX {
                    -1
                } else {
                    last_off as isize
                };

                if cursor < (last_off + self.prompt_len as isize + 2) as usize {
                    eatit = true;
                }
                None
            }
            (Modifiers::NONE, Key::Escape) => {
                eatit = true;
                self.search_mode = None;
                self.prompt = self.save_prompt.clone();
                let last_off = self.text.rfind('\n').unwrap_or(0);
                self.text.truncate(last_off);
                self.draw_prompt();

                None
            }
            (
                Modifiers {
                    alt: false,
                    ctrl: true,
                    shift: false,
                    mac_cmd: false,
                    command: true,
                },
                Key::R,
            ) => {
                eatit = true;
                self.search_mode = Some(self.command_history.len());
                self.prompt = "(reverse-i-search): ".to_string();
                let last_off = self.text.rfind('\n').unwrap_or(0);
                self.text.truncate(last_off);
                self.draw_prompt();
                None
            }
            _ => None,
        };

        (eatit, return_value)
    }
    fn draw_prompt(&mut self) {
        if self.text.len() > 0 && !self.text.ends_with('\n') {
            self.text.push('\n');
        }
        self.text.push_str(&self.prompt);
    }
    fn search_history(&mut self, partial: &str) {
        let mut found = None;
        for (i, cmd) in self.command_history.iter().enumerate() {
            if cmd.contains(partial) {
                found = Some(i);
                break;
            }
        }
        if let Some(i) = found {
            self.history_cursor = i as isize;
            let last = self.get_last_line();
            self.text = self.text.strip_suffix(last).unwrap_or("").to_string();
            self.text
                .push_str(self.command_history[self.history_cursor as usize].as_str());
        }
    }
    fn handle_kb(&mut self, ctx: &egui::Context, _hacksys: &HackSystem) -> AppMessage {
        // process all the key events in the queue
        // if they are meaningful to the console then use them and consume them
        // otherwise pass along to the textedit widget

        // current cursor position

        let cursor = if let Some(state) = egui::TextEdit::load_state(ctx, self.id) {
            state.cursor.char_range().unwrap().primary.index
        } else {
            0
        };

        // a list of keys to consume

        let mut kill_list = vec![];
        let mut command = None;
        ctx.input(|input| {
            for event in &input.events {
                if let Event::Key {
                    key,
                    physical_key: _,
                    pressed,
                    modifiers,
                    repeat: _,
                } = event
                {
                    if *pressed {
                        let (kill, msg) = self.handle_key(key, *modifiers, cursor);
                        if kill {
                            kill_list.push((*modifiers, *key));
                        }
                        command = msg;
                        // if the user pressed enter we are done
                        if command.is_some() {
                            break;
                        }
                    }
                }
            }
        });

        // consume the keys we didnt use
        for (modifiers, key) in kill_list {
            println!("Console focus: consume key: {:?} {:?}", modifiers, key);
            Self::consume_key(ctx, modifiers, key);
        }

        if let Some(command) = command {
            return AppMessage::ConsoleCommand(command);
        }
        AppMessage::None
    }

    pub fn draw(
        &mut self,
        ctx: &egui::Context,
        open: &mut bool,
        hacksys: &HackSystem,
    ) -> AppMessage {
        let msg = if let Some(id) = ctx.memory(|mem| mem.focused()) {
            if id == Id::new("console_text") {
                self.handle_kb(ctx, hacksys)
            } else {
                AppMessage::None
            }
        } else {
            {
                AppMessage::None
            }
        };

        egui::Window::new(self.name())
            .id(Id::new(self.name()))
            .open(open)
            .vscroll(false)
            .default_height(50.0)
            .show(ctx, |ui| {
                self.ui(ui);
            });
        if self.search_mode.is_some() {
            // this is all so that we get the escape key (to exit search)
            let event_filter = EventFilter {
                escape: true,
                horizontal_arrows: true,
                vertical_arrows: true,
                tab: false,
                //..Default::default()
            };
            if ctx.memory(|mem| mem.has_focus(self.id)) {
                ctx.memory_mut(|mem| mem.set_focus_lock_filter(self.id, event_filter));
            }
        }
        msg
    }
    pub fn update(&mut self, msg: crate::ui::app::UpdateMessage) {
        if let UpdateType::Text(output) = msg.message {
            self.text
                .push_str(&format!("\n{}\n{}", output, self.prompt));
            self.new_line = true;
        }
    }
    fn name(&self) -> &'static str {
        "Console Window"
    }

    fn id(&self) -> Id {
        self.id
    }
}
