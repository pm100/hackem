use egui::{InputState, Key};

use super::widgets::screen::CURRENT_KEY;

pub(crate) fn lookup_key(input: &InputState) {
    match input.keys_down.iter().next().unwrap() {
        Key::A => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'A';
            } else {
                CURRENT_KEY = b'a';
            }
        },
        Key::B => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'B';
            } else {
                CURRENT_KEY = b'b';
            }
        },
        Key::C => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'C';
            } else {
                CURRENT_KEY = b'c';
            }
        },
        Key::D => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'D';
            } else {
                CURRENT_KEY = b'd';
            }
        },
        Key::E => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'E';
            } else {
                CURRENT_KEY = b'e';
            }
        },
        Key::F => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'F';
            } else {
                CURRENT_KEY = b'f';
            }
        },
        Key::G => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'G';
            } else {
                CURRENT_KEY = b'g';
            }
        },
        Key::H => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'H';
            } else {
                CURRENT_KEY = b'h';
            }
        },
        Key::I => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'I';
            } else {
                CURRENT_KEY = b'i';
            }
        },
        Key::J => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'J';
            } else {
                CURRENT_KEY = b'j';
            }
        },
        Key::K => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'K';
            } else {
                CURRENT_KEY = b'k';
            }
        },
        Key::L => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'L';
            } else {
                CURRENT_KEY = b'l';
            }
        },
        Key::M => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'M';
            } else {
                CURRENT_KEY = b'm';
            }
        },
        Key::N => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'N';
            } else {
                CURRENT_KEY = b'n';
            }
        },
        Key::O => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'O';
            } else {
                CURRENT_KEY = b'o';
            }
        },
        Key::P => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'P';
            } else {
                CURRENT_KEY = b'p';
            }
        },
        Key::Q => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'Q';
            } else {
                CURRENT_KEY = b'q';
            }
        },
        Key::R => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'R';
            } else {
                CURRENT_KEY = b'r';
            }
        },
        Key::S => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'S';
            } else {
                CURRENT_KEY = b's';
            }
        },
        Key::T => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'T';
            } else {
                CURRENT_KEY = b't';
            }
        },
        Key::U => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'U';
            } else {
                CURRENT_KEY = b'u';
            }
        },
        Key::V => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'V';
            } else {
                CURRENT_KEY = b'v';
            }
        },
        Key::W => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'W';
            } else {
                CURRENT_KEY = b'w';
            }
        },
        Key::X => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'X';
            } else {
                CURRENT_KEY = b'x';
            }
        },
        Key::Y => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'Y';
            } else {
                CURRENT_KEY = b'y';
            }
        },
        Key::Z => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'Z';
            } else {
                CURRENT_KEY = b'z';
            }
        },
        Key::Num0 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b')';
            } else {
                CURRENT_KEY = b'0';
            }
        },
        Key::Num1 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'!';
            } else {
                CURRENT_KEY = b'1';
            }
        },
        Key::Num2 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'@';
            } else {
                CURRENT_KEY = b'2';
            }
        },
        Key::Num3 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'#';
            } else {
                CURRENT_KEY = b'3';
            }
        },
        Key::Num4 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'$';
            } else {
                CURRENT_KEY = b'4';
            }
        },
        Key::Num5 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'%';
            } else {
                CURRENT_KEY = b'5';
            }
        },
        Key::Num6 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'^';
            } else {
                CURRENT_KEY = b'6';
            }
        },
        Key::Num7 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'&';
            } else {
                CURRENT_KEY = b'7';
            }
        },
        Key::Num8 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'*';
            } else {
                CURRENT_KEY = b'8';
            }
        },
        Key::Num9 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = b'(';
            } else {
                CURRENT_KEY = b'9';
            }
        },
        Key::Space => unsafe {
            CURRENT_KEY = b' ';
        },
        Key::Enter => unsafe {
            CURRENT_KEY = 128_u8;
        },
        Key::Backspace => unsafe {
            CURRENT_KEY = 129;
        },
        Key::Escape => unsafe {
            CURRENT_KEY = 140;
        },
        Key::ArrowLeft => unsafe {
            CURRENT_KEY = 130;
        },
        Key::ArrowRight => unsafe {
            CURRENT_KEY = 132;
        },
        Key::ArrowUp => unsafe {
            CURRENT_KEY = 131;
        },
        Key::ArrowDown => unsafe {
            CURRENT_KEY = 133;
        },
        Key::Home => unsafe {
            CURRENT_KEY = 134;
        },
        Key::End => unsafe {
            CURRENT_KEY = 135;
        },
        Key::PageUp => unsafe {
            CURRENT_KEY = 136;
        },
        Key::PageDown => unsafe {
            CURRENT_KEY = 137;
        },
        Key::Insert => unsafe {
            CURRENT_KEY = 138;
        },
        Key::Delete => unsafe {
            CURRENT_KEY = 139;
        },
        Key::F1 => unsafe {
            CURRENT_KEY = 141;
        },
        Key::F2 => unsafe {
            CURRENT_KEY = 142;
        },
        Key::F3 => unsafe {
            CURRENT_KEY = 143;
        },
        Key::F4 => unsafe {
            CURRENT_KEY = 144;
        },
        Key::F5 => unsafe {
            CURRENT_KEY = 145;
        },
        Key::F6 => unsafe {
            CURRENT_KEY = 146;
        },
        Key::F7 => unsafe {
            CURRENT_KEY = 147;
        },
        Key::F8 => unsafe {
            CURRENT_KEY = 148;
        },
        Key::F9 => unsafe {
            CURRENT_KEY = 149;
        },
        Key::F10 => unsafe {
            CURRENT_KEY = 150;
        },
        Key::F11 => unsafe {
            CURRENT_KEY = 151;
        },
        Key::F12 => unsafe {
            CURRENT_KEY = 152;
        },
        Key::Comma => unsafe {
            CURRENT_KEY = 44;
        },
        Key::Period => unsafe {
            CURRENT_KEY = 46;
        },
        Key::Slash => unsafe {
            CURRENT_KEY = 47;
        },

        Key::Semicolon => unsafe {
            CURRENT_KEY = 59;
        },
        Key::Colon => unsafe {
            CURRENT_KEY = 58;
        },
        Key::Pipe => unsafe {
            CURRENT_KEY = 124;
        },
        Key::OpenBracket => unsafe {
            CURRENT_KEY = 91;
        },
        Key::CloseBracket => unsafe {
            CURRENT_KEY = 93;
        },
        Key::Backslash => unsafe {
            CURRENT_KEY = 92;
        },
        Key::Minus => unsafe {
            CURRENT_KEY = 45;
        },
        Key::Equals => unsafe {
            CURRENT_KEY = 61;
        },
        Key::Questionmark => unsafe {
            CURRENT_KEY = 63;
        },
        Key::Backtick => unsafe {
            CURRENT_KEY = 96;
        },

        _ => {}
    }
}
