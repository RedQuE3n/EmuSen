//! `nano`, the shell's one shipped program (EmuSen_DianaSh.md §8.1): a full-screen editor that owns the terminal
//! while it runs, as a process does on every system. It refuses as the C# command refused, in its words: no path, a
//! path above the sandbox's root (`DIANASH_ROOT`, which the shell gives every program, §7.1), or no terminal. Its keys
//! are the C# editor's: arrows, Home, End, Page Up and Down, Enter, Backspace, Delete, ^O to save and ^X to leave.

#[allow(dead_code, unused_imports)]
mod terminal;

use emusen_dianash::sandbox::Sandbox;
use std::io::Write;
use std::process::ExitCode;

/// A refusal on standard error, status 1, as the C# command's `Fail`.
fn refuse(words: &str) -> ExitCode {
    eprintln!("{words}");
    ExitCode::from(1)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return refuse("nano: usage: nano <path>");
    }
    let root = std::env::var("DIANASH_ROOT").ok();
    let sandbox = Sandbox { root };
    let (inside, path) = sandbox.resolve(&args[1]);
    if !inside {
        return refuse(&format!("nano: '{}' is outside the project sandbox ({})", args[1], sandbox.root()));
    }
    if !terminal::is_terminal() || !terminal::output_is_terminal() {
        return refuse("nano: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)");
    }
    let Some(raw) = terminal::Raw::enter() else {
        return refuse("nano: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)");
    };
    let mut editor = Editor::open(&path, &args[1]);
    let saved = editor.run();
    drop(raw);
    print!("\x1b[2J\x1b[H");
    let _ = std::io::stdout().flush();
    if saved {
        println!("nano: saved {} line(s) to {}", editor.lines.len(), args[1]);
    } else {
        println!("nano: exited {} without saving", args[1]);
    }
    ExitCode::SUCCESS
}

enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Backspace,
    Delete,
    Enter,
    Escape,
    Save,
    Exit,
    Char(char),
    Other,
}

fn read_key() -> Option<Key> {
    let byte = terminal::read_byte()?;
    Some(match byte {
        b'\r' | b'\n' => Key::Enter,
        0x7F | 0x08 => Key::Backspace,
        0x0F => Key::Save,
        0x18 => Key::Exit,
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
                (b'~', "5") => Key::PageUp,
                (b'~', "6") => Key::PageDown,
                _ => Key::Other,
            };
        }
        parameters.push(b as char);
    }
}

struct Editor {
    path: String,
    shown: String,
    lines: Vec<Vec<char>>,
    row: usize,
    col: usize,
    top: usize,
    modified: bool,
}

/// The window's size, with the C# editor's floor and its fallback where the terminal does not say.
fn size() -> (usize, usize) {
    terminal::size().map_or((80, 24), |(w, h)| (w.max(20), h.max(5)))
}

fn pad_or_truncate(text: &str, width: usize) -> String {
    let mut chars: Vec<char> = text.chars().take(width).collect();
    chars.resize(width, ' ');
    chars.into_iter().collect()
}

impl Editor {
    fn open(path: &str, shown: &str) -> Editor {
        let mut lines: Vec<Vec<char>> = if std::path::Path::new(path).is_file() {
            emusen_dotnet::file::read_all_lines(path).unwrap_or_default().into_iter().map(|l| l.chars().collect()).collect()
        } else {
            Vec::new()
        };
        if lines.is_empty() {
            lines.push(Vec::new());
        }
        Editor { path: path.to_string(), shown: shown.to_string(), lines, row: 0, col: 0, top: 0, modified: false }
    }

    /// Edits until ^X; whether anything was saved.
    fn run(&mut self) -> bool {
        print!("\x1b[2J");
        self.draw();
        let mut saved = false;
        while let Some(key) = read_key() {
            match key {
                Key::Exit => {
                    if self.modified {
                        match self.ask_save() {
                            None => {
                                self.draw();
                                continue;
                            }
                            Some(true) => {
                                self.save();
                                saved = true;
                            }
                            Some(false) => {}
                        }
                    }
                    break;
                }
                Key::Save => {
                    self.save();
                    saved = true;
                }
                other => self.edit(other),
            }
            self.draw();
        }
        saved
    }

    /// Each line ended by `\n`, as the shell's files are on every system (§12.1).
    fn save(&mut self) {
        let text: String = self.lines.iter().map(|l| l.iter().collect::<String>() + "\n").collect();
        if std::fs::write(&self.path, text).is_ok() {
            self.modified = false;
        }
    }

    /// y, n, or Escape to stay.
    fn ask_save(&self) -> Option<bool> {
        let (width, height) = size();
        print!("\x1b[{height};1H{}", pad_or_truncate("Save modified buffer? (y/n, Esc to cancel)", width));
        let _ = std::io::stdout().flush();
        loop {
            match read_key()? {
                Key::Char('y' | 'Y') => return Some(true),
                Key::Char('n' | 'N') => return Some(false),
                Key::Escape => return None,
                _ => {}
            }
        }
    }

    fn edit(&mut self, key: Key) {
        let body = size().1.saturating_sub(2).max(1);
        let length = self.lines[self.row].len();
        match key {
            Key::Up if self.row > 0 => {
                self.row -= 1;
                self.col = self.col.min(self.lines[self.row].len());
            }
            Key::Down if self.row + 1 < self.lines.len() => {
                self.row += 1;
                self.col = self.col.min(self.lines[self.row].len());
            }
            Key::Left => {
                if self.col > 0 {
                    self.col -= 1;
                } else if self.row > 0 {
                    self.row -= 1;
                    self.col = self.lines[self.row].len();
                }
            }
            Key::Right => {
                if self.col < length {
                    self.col += 1;
                } else if self.row + 1 < self.lines.len() {
                    self.row += 1;
                    self.col = 0;
                }
            }
            Key::Home => self.col = 0,
            Key::End => self.col = length,
            Key::PageUp => {
                self.row = self.row.saturating_sub(body);
                self.col = self.col.min(self.lines[self.row].len());
            }
            Key::PageDown => {
                self.row = (self.row + body).min(self.lines.len() - 1);
                self.col = self.col.min(self.lines[self.row].len());
            }
            Key::Backspace => {
                if self.col > 0 {
                    self.lines[self.row].remove(self.col - 1);
                    self.col -= 1;
                    self.modified = true;
                } else if self.row > 0 {
                    let line = self.lines.remove(self.row);
                    self.row -= 1;
                    self.col = self.lines[self.row].len();
                    self.lines[self.row].extend(line);
                    self.modified = true;
                }
            }
            Key::Delete => {
                if self.col < length {
                    self.lines[self.row].remove(self.col);
                    self.modified = true;
                } else if self.row + 1 < self.lines.len() {
                    let next = self.lines.remove(self.row + 1);
                    self.lines[self.row].extend(next);
                    self.modified = true;
                }
            }
            Key::Enter => {
                let rest = self.lines[self.row].split_off(self.col);
                self.lines.insert(self.row + 1, rest);
                self.row += 1;
                self.col = 0;
                self.modified = true;
            }
            Key::Char(c) => {
                self.lines[self.row].insert(self.col, c);
                self.col += 1;
                self.modified = true;
            }
            _ => {}
        }
    }

    fn draw(&mut self) {
        let (width, height) = size();
        let body = height.saturating_sub(2).max(1);
        if self.row < self.top {
            self.top = self.row;
        } else if self.row >= self.top + body {
            self.top = self.row + 1 - body;
        }
        let mut screen = String::from("\x1b[H");
        screen += &pad_or_truncate(&format!(" DianaSh nano  {}{} ", self.shown, if self.modified { " *" } else { "" }), width);
        for i in 0..body {
            let line: String = self.lines.get(self.top + i).map(|l| l.iter().collect()).unwrap_or_default();
            screen += &format!("\x1b[{};1H{}", i + 2, pad_or_truncate(&line, width));
        }
        screen += &format!("\x1b[{height};1H{}", pad_or_truncate("^O Save    ^X Exit    Arrows/PgUp/PgDn move    Enter/Backspace/Delete edit", width));
        screen += &format!("\x1b[{};{}H", self.row - self.top + 2, self.col.min(width - 1) + 1);
        print!("{screen}");
        let _ = std::io::stdout().flush();
    }
}
