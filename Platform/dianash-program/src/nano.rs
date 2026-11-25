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
    let mut editor = match Editor::open(&path, &args[1]) {
        Ok(editor) => editor,
        Err(words) => return refuse(&format!("nano: {words}")),
    };
    let Some(raw) = terminal::Raw::enter() else {
        return refuse("nano: needs a real interactive terminal (stdin/stdout is redirected, or this console has none)");
    };
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
    status: Option<String>,
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
    fn open(path: &str, shown: &str) -> Result<Editor, String> {
        let mut lines: Vec<Vec<char>> = match std::fs::read_to_string(path) {
            Ok(text) => text.lines().map(|l| l.chars().collect()).collect(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(format!("{shown}: {e}")),
        };
        if lines.is_empty() {
            lines.push(Vec::new());
        }
        Ok(Editor { path: path.to_string(), shown: shown.to_string(), lines, row: 0, col: 0, top: 0, modified: false, status: None })
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
                                if self.save().is_err() {
                                    self.draw();
                                    continue;
                                }
                                saved = true;
                            }
                            Some(false) => {}
                        }
                    }
                    break;
                }
                Key::Save => {
                    if self.save().is_ok() {
                        saved = true;
                    }
                }
                other => {
                    self.status = None;
                    self.edit(other);
                }
            }
            self.draw();
        }
        saved
    }

    /// Each line ended by `\n`, as the shell's files are on every system (§12.1).
    fn save(&mut self) -> Result<(), String> {
        let text: String = self.lines.iter().map(|l| l.iter().collect::<String>() + "\n").collect();
        match std::fs::write(&self.path, text) {
            Ok(()) => {
                self.modified = false;
                self.status = Some(format!("Saved {} line(s) to {}", self.lines.len(), self.shown));
                Ok(())
            }
            Err(e) => {
                let words = format!("Save failed: {}: {e}", self.shown);
                self.status = Some(words.clone());
                Err(words)
            }
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
        let footer = self.status.as_deref().unwrap_or("^O Save    ^X Exit    Arrows/PgUp/PgDn move    Enter/Backspace/Delete edit");
        screen += &format!("\x1b[{height};1H{}", pad_or_truncate(footer, width));
        screen += &format!("\x1b[{};{}H", self.row - self.top + 2, self.col.min(width - 1) + 1);
        print!("{screen}");
        let _ = std::io::stdout().flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn scratch(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("emusen-nano-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn open_allows_missing_file_but_rejects_directories_and_invalid_utf8() {
        let dir = scratch("open");
        let missing = dir.join("new.txt");
        let editor = Editor::open(missing.to_str().unwrap(), "new.txt").unwrap();
        assert_eq!(editor.lines, vec![Vec::<char>::new()]);

        assert!(Editor::open(dir.to_str().unwrap(), "dir").is_err());

        let bad_utf8 = dir.join("binary.bin");
        std::fs::write(&bad_utf8, [0xFF, 0xFE, 0x00]).unwrap();
        let err = Editor::open(bad_utf8.to_str().unwrap(), "binary.bin").err().unwrap();
        assert!(err.contains("binary.bin"), "{err}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[cfg(unix)]
    #[test]
    fn open_rejects_unreadable_file() {
        use std::os::unix::fs::PermissionsExt;
        let dir = scratch("unreadable");
        let file = dir.join("secret.txt");
        std::fs::write(&file, "hello\n").unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();
        let readable_as_root = std::fs::read_to_string(&file).is_ok();
        if !readable_as_root {
            assert!(Editor::open(file.to_str().unwrap(), "secret.txt").is_err());
        }
        let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_surfaces_write_errors_and_preserves_modified_flag() {
        let dir = scratch("save");
        let ok_path = dir.join("ok.txt");
        let mut ok_editor = Editor::open(ok_path.to_str().unwrap(), "ok.txt").unwrap();
        ok_editor.lines[0] = "hello".chars().collect();
        ok_editor.modified = true;
        assert!(ok_editor.save().is_ok());
        assert!(!ok_editor.modified);
        assert_eq!(std::fs::read_to_string(&ok_path).unwrap(), "hello\n");

        let blocked = dir.join("missing-parent").join("out.txt");
        let mut fail_editor = Editor::open(blocked.to_str().unwrap(), "out.txt").unwrap();
        fail_editor.lines[0] = "unsaved".chars().collect();
        fail_editor.modified = true;
        let err = fail_editor.save().unwrap_err();
        assert!(err.contains("Save failed") && err.contains("out.txt"), "{err}");
        assert!(fail_editor.modified, "failed save must keep modified flag set");
        assert_eq!(fail_editor.status.as_deref(), Some(err.as_str()));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
