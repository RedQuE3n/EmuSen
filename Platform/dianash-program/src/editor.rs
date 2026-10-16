//! The line editor, written to the C# `ConsoleLineReader`'s behaviour (§16 Q12): history on Up and Down, Left,
//! Right, Home, End, Backspace and Delete, Escape to clear, anchored past the prompt, with its known gap in
//! wrapping. Two keys the .NET runtime answered outside the reader are answered here: Ctrl-C discards the line and
//! Ctrl-D on an empty one ends input. See EmuSen_DianaSh.md §12.3 and §13.7.

use crate::terminal::{self, Raw};
use std::io::{self, BufRead, Write};

enum Key {
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Up,
    Down,
    Escape,
    Interrupt,
    EndOfInput,
    Char(char),
    Other,
}

/// A line, or none at the end of input. Not at a terminal, a line is read as it comes.
pub fn read_line(prompt_width: usize, history: &[String]) -> Option<String> {
    if !terminal::is_terminal() {
        let mut line = String::new();
        return match io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line.trim_end_matches(['\n', '\r']).to_string()),
        };
    }
    let Some(_raw) = Raw::enter() else {
        let mut line = String::new();
        return io::stdin().lock().read_line(&mut line).ok().filter(|n| *n > 0).map(|_| line.trim_end_matches(['\n', '\r']).to_string());
    };
    Editor { start: prompt_width, buffer: Vec::new(), cursor: 0 }.run(history)
}

struct Editor {
    /// The column the input starts at, past the prompt.
    start: usize,
    buffer: Vec<char>,
    cursor: usize,
}

impl Editor {
    fn run(mut self, history: &[String]) -> Option<String> {
        // None while editing a fresh line; `pending` is what Down past the newest restores.
        let mut index: Option<usize> = None;
        let mut pending = String::new();
        loop {
            match read_key()? {
                Key::Enter => {
                    print!("\r\n");
                    flush();
                    return Some(self.buffer.iter().collect());
                }
                Key::Backspace if self.cursor > 0 => {
                    let previous = self.buffer.len();
                    self.buffer.remove(self.cursor - 1);
                    self.cursor -= 1;
                    self.redraw(previous);
                }
                Key::Delete if self.cursor < self.buffer.len() => {
                    let previous = self.buffer.len();
                    self.buffer.remove(self.cursor);
                    self.redraw(previous);
                }
                Key::Left if self.cursor > 0 => {
                    self.cursor -= 1;
                    self.place();
                }
                Key::Right if self.cursor < self.buffer.len() => {
                    self.cursor += 1;
                    self.place();
                }
                Key::Home => {
                    self.cursor = 0;
                    self.place();
                }
                Key::End => {
                    self.cursor = self.buffer.len();
                    self.place();
                }
                Key::Up if !history.is_empty() => {
                    match index {
                        None => {
                            pending = self.buffer.iter().collect();
                            index = Some(history.len() - 1);
                        }
                        Some(i) if i > 0 => index = Some(i - 1),
                        _ => {}
                    }
                    self.set(&history[index.unwrap_or(0)]);
                }
                Key::Down => match index {
                    None => {}
                    Some(i) if i + 1 < history.len() => {
                        index = Some(i + 1);
                        self.set(&history[i + 1]);
                    }
                    Some(_) => {
                        index = None;
                        let restored = pending.clone();
                        self.set(&restored);
                    }
                },
                Key::Escape => {
                    index = None;
                    self.set("");
                }
                Key::Interrupt => {
                    print!("^C\r\n");
                    flush();
                    return Some(String::new());
                }
                Key::EndOfInput if self.buffer.is_empty() => {
                    print!("\r\n");
                    flush();
                    return None;
                }
                Key::Char(c) => {
                    let previous = self.buffer.len();
                    self.buffer.insert(self.cursor, c);
                    self.cursor += 1;
                    self.redraw(previous);
                }
                _ => {}
            }
        }
    }

    fn place(&self) {
        print!("\r\x1b[{}G", self.start + self.cursor + 1);
        flush();
    }

    fn redraw(&self, previous: usize) {
        let text: String = self.buffer.iter().collect();
        print!("\r\x1b[{}G{text}", self.start + 1);
        if self.buffer.len() < previous {
            print!("{}", " ".repeat(previous - self.buffer.len()));
        }
        self.place();
    }

    fn set(&mut self, text: &str) {
        let previous = self.buffer.len();
        self.buffer = text.chars().collect();
        self.cursor = self.buffer.len();
        self.redraw(previous);
    }
}

fn flush() {
    let _ = io::stdout().flush();
}

/// One key from the terminal's bytes: UTF-8 text, control bytes, and the CSI and SS3 sequences of the arrows, Home, End and Delete.
fn read_key() -> Option<Key> {
    let byte = terminal::read_byte()?;
    Some(match byte {
        b'\r' | b'\n' => Key::Enter,
        0x7F | 0x08 => Key::Backspace,
        0x03 => Key::Interrupt,
        0x04 => Key::EndOfInput,
        0x1B => {
            if !terminal::byte_waiting(50) {
                return Some(Key::Escape);
            }
            match terminal::read_byte()? {
                b'[' => csi(),
                b'O' => match terminal::read_byte()? {
                    b'H' => Key::Home,
                    b'F' => Key::End,
                    _ => Key::Other,
                },
                _ => Key::Other,
            }
        }
        b if b < 0x20 => Key::Other,
        b if b < 0x80 => Key::Char(b as char),
        lead => {
            let extra = if lead >= 0xF0 { 3 } else if lead >= 0xE0 { 2 } else { 1 };
            let mut bytes = vec![lead];
            for _ in 0..extra {
                bytes.push(terminal::read_byte()?);
            }
            match std::str::from_utf8(&bytes).ok().and_then(|s| s.chars().next()) {
                Some(c) if !c.is_control() => Key::Char(c),
                _ => Key::Other,
            }
        }
    })
}

fn csi() -> Key {
    let mut parameters = String::new();
    loop {
        let Some(b) = terminal::read_byte() else { return Key::Other };
        if (0x40..=0x7E).contains(&b) {
            return match (b, parameters.as_str()) {
                (b'A', _) => Key::Up,
                (b'B', _) => Key::Down,
                (b'C', _) => Key::Right,
                (b'D', _) => Key::Left,
                (b'H', _) => Key::Home,
                (b'F', _) => Key::End,
                (b'~', "1" | "7") => Key::Home,
                (b'~', "4" | "8") => Key::End,
                (b'~', "3") => Key::Delete,
                _ => Key::Other,
            };
        }
        parameters.push(b as char);
    }
}
