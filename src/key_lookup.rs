use egui::{InputState, Key};

use crate::app::CURRENT_KEY;

pub(crate) fn lookup_key(input: &InputState) {
    match input.keys_down.iter().next().unwrap() {
        Key::A => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'A' as u8;
            } else {
                CURRENT_KEY = 'a' as u8;
            }
        },
        Key::B => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'B' as u8;
            } else {
                CURRENT_KEY = 'b' as u8;
            }
        },
        Key::C => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'C' as u8;
            } else {
                CURRENT_KEY = 'c' as u8;
            }
        },
        Key::D => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'D' as u8;
            } else {
                CURRENT_KEY = 'd' as u8;
            }
        },
        Key::E => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'E' as u8;
            } else {
                CURRENT_KEY = 'e' as u8;
            }
        },
        Key::F => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'F' as u8;
            } else {
                CURRENT_KEY = 'f' as u8;
            }
        },
        Key::G => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'G' as u8;
            } else {
                CURRENT_KEY = 'g' as u8;
            }
        },
        Key::H => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'H' as u8;
            } else {
                CURRENT_KEY = 'h' as u8;
            }
        },
        Key::I => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'I' as u8;
            } else {
                CURRENT_KEY = 'i' as u8;
            }
        },
        Key::J => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'J' as u8;
            } else {
                CURRENT_KEY = 'j' as u8;
            }
        },
        Key::K => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'K' as u8;
            } else {
                CURRENT_KEY = 'k' as u8;
            }
        },
        Key::L => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'L' as u8;
            } else {
                CURRENT_KEY = 'l' as u8;
            }
        },
        Key::M => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'M' as u8;
            } else {
                CURRENT_KEY = 'm' as u8;
            }
        },
        Key::N => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'N' as u8;
            } else {
                CURRENT_KEY = 'n' as u8;
            }
        },
        Key::O => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'O' as u8;
            } else {
                CURRENT_KEY = 'o' as u8;
            }
        },
        Key::P => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'P' as u8;
            } else {
                CURRENT_KEY = 'p' as u8;
            }
        },
        Key::Q => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'Q' as u8;
            } else {
                CURRENT_KEY = 'q' as u8;
            }
        },
        Key::R => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'R' as u8;
            } else {
                CURRENT_KEY = 'r' as u8;
            }
        },
        Key::S => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'S' as u8;
            } else {
                CURRENT_KEY = 's' as u8;
            }
        },
        Key::T => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'T' as u8;
            } else {
                CURRENT_KEY = 't' as u8;
            }
        },
        Key::U => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'U' as u8;
            } else {
                CURRENT_KEY = 'u' as u8;
            }
        },
        Key::V => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'V' as u8;
            } else {
                CURRENT_KEY = 'v' as u8;
            }
        },
        Key::W => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'W' as u8;
            } else {
                CURRENT_KEY = 'w' as u8;
            }
        },
        Key::X => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'X' as u8;
            } else {
                CURRENT_KEY = 'x' as u8;
            }
        },
        Key::Y => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'Y' as u8;
            } else {
                CURRENT_KEY = 'y' as u8;
            }
        },
        Key::Z => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = 'Z' as u8;
            } else {
                CURRENT_KEY = 'z' as u8;
            }
        },
        Key::Num0 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = ')' as u8;
            } else {
                CURRENT_KEY = '0' as u8;
            }
        },
        Key::Num1 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '!' as u8;
            } else {
                CURRENT_KEY = '1' as u8;
            }
        },
        Key::Num2 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '@' as u8;
            } else {
                CURRENT_KEY = '2' as u8;
            }
        },
        Key::Num3 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '#' as u8;
            } else {
                CURRENT_KEY = '3' as u8;
            }
        },
        Key::Num4 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '$' as u8;
            } else {
                CURRENT_KEY = '4' as u8;
            }
        },
        Key::Num5 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '%' as u8;
            } else {
                CURRENT_KEY = '5' as u8;
            }
        },
        Key::Num6 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '^' as u8;
            } else {
                CURRENT_KEY = '6' as u8;
            }
        },
        Key::Num7 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '&' as u8;
            } else {
                CURRENT_KEY = '7' as u8;
            }
        },
        Key::Num8 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '*' as u8;
            } else {
                CURRENT_KEY = '8' as u8;
            }
        },
        Key::Num9 => unsafe {
            if input.modifiers.shift {
                CURRENT_KEY = '(' as u8;
            } else {
                CURRENT_KEY = '9' as u8;
            }
        },
        Key::Space => unsafe {
            CURRENT_KEY = ' ' as u8;
        },
        Key::Enter => unsafe {
            CURRENT_KEY = 128 as u8;
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
