//! .NET's text files: `File.ReadAllText`'s decoding, its lines, its writes, and the words its exceptions carry.
//! See EmuSen_DianaSh.md §13.7.

use std::io::ErrorKind;

/// A file's bytes as `File.ReadAllText` gives them: a byte-order mark chooses UTF-8, UTF-16 or UTF-32 and is dropped, and what is not valid becomes U+FFFD.
pub fn decode(bytes: &[u8]) -> String {
    fn utf16(bytes: &[u8], unit: fn([u8; 2]) -> u16) -> String {
        let whole: Vec<u16> = bytes.as_chunks::<2>().0.iter().map(|pair| unit(*pair)).collect();
        let mut text: String = char::decode_utf16(whole).map(|c| c.unwrap_or('\u{FFFD}')).collect();
        if bytes.len() % 2 == 1 {
            text.push('\u{FFFD}');
        }
        text
    }
    fn utf32(bytes: &[u8], unit: fn([u8; 4]) -> u32) -> String {
        bytes.chunks(4).map(|four| <[u8; 4]>::try_from(four).ok().and_then(|four| char::from_u32(unit(four))).unwrap_or('\u{FFFD}')).collect()
    }
    match bytes {
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        [0xFF, 0xFE, 0, 0, rest @ ..] => utf32(rest, u32::from_le_bytes),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        [0, 0, 0xFE, 0xFF, rest @ ..] => utf32(rest, u32::from_be_bytes),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

/// The words .NET's file calls throw with, for the error a system call gave on `path`.
pub fn io_message(error: &std::io::Error, path: &str) -> String {
    match error.kind() {
        ErrorKind::NotFound => {
            let parent = std::path::Path::new(path).parent();
            if parent.is_some_and(|p| p.as_os_str().is_empty() || p.is_dir()) {
                format!("Could not find file '{path}'.")
            } else {
                format!("Could not find a part of the path '{path}'.")
            }
        }
        ErrorKind::PermissionDenied | ErrorKind::IsADirectory => format!("Access to the path '{path}' is denied."),
        ErrorKind::NotADirectory => format!("Could not find a part of the path '{path}'."),
        _ => error.to_string(),
    }
}

/// `File.ReadAllText`: a byte-order mark chooses the encoding.
pub fn read_all_text(path: &str) -> Result<String, String> {
    std::fs::read(path).map(|bytes| decode(&bytes)).map_err(|e| io_message(&e, path))
}

/// `File.ReadAllLines`: split at `\r\n`, `\n` or `\r`, a final terminator ending the last line rather than starting one.
pub fn read_all_lines(path: &str) -> Result<Vec<String>, String> {
    let text = read_all_text(path)?;
    let mut lines = Vec::new();
    let mut line = String::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                lines.push(std::mem::take(&mut line));
            }
            '\n' => lines.push(std::mem::take(&mut line)),
            _ => line.push(c),
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    Ok(lines)
}

/// `File.WriteAllText` (`append` false) and `File.AppendAllText`: UTF-8 with no byte-order mark.
pub fn write_all_text(path: &str, text: &str, append: bool) -> Result<(), String> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    if append {
        options.append(true).create(true);
    } else {
        options.write(true).create(true).truncate(true);
    }
    let mut file = options.open(path).map_err(|e| io_message(&e, path))?;
    file.write_all(text.as_bytes()).map_err(|e| io_message(&e, path))
}

/// `File.WriteAllBytes`: the file made or truncated, with .NET's words for a refusal.
pub fn write_all_bytes(path: &str, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(path, bytes).map_err(|e| io_message(&e, path))
}

/// `File.ReadAllBytes`, with .NET's words for a refusal.
pub fn read_all_bytes(path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(path).map_err(|e| io_message(&e, path))
}

/// `Environment.CurrentDirectory`.
pub fn current_directory() -> String {
    std::env::current_dir().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_end_at_any_terminator() {
        let dir = std::env::temp_dir().join(format!("dianash-lines-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.txt");
        std::fs::write(&file, "a\r\nb\rc\n\nd\n").unwrap();
        assert_eq!(read_all_lines(file.to_str().unwrap()).unwrap(), ["a", "b", "c", "", "d"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
