//! The .NET rules the C# shell's words and choices rest on, reproduced so that the golden corpus reads the same
//! through either shell: trimming, `int.TryParse`, casing, padding by UTF-16 length, the text file calls and the
//! words of their exceptions. See EmuSen_DianaSh.md §13.7.

use emusen_galaxia::dotnet_path::{equals_ignore_case, is_white_space, simple_upper};
use emusen_galaxia::dotnet_text::{lower_invariant, utf16_len};
use std::io::ErrorKind;

/// `char.IsLetter`. A character outside the Basic Multilingual Plane is two surrogates to .NET, and neither is a letter.
pub fn is_letter(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphabetic();
    }
    (c as u32) <= 0xFFFF && c.is_alphabetic()
}

/// `char.IsLetterOrDigit`, as `is_letter` approximates it outside ASCII.
pub fn is_letter_or_digit(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphanumeric();
    }
    (c as u32) <= 0xFFFF && (c.is_alphabetic() || c.is_numeric())
}

/// `string.Trim()`.
pub fn trim(text: &str) -> &str {
    text.trim_matches(is_white_space)
}

/// `string.TrimStart()`.
pub fn trim_start(text: &str) -> &str {
    text.trim_start_matches(is_white_space)
}

/// `string.ToLowerInvariant()`.
pub fn to_lower(text: &str) -> String {
    text.chars().map(lower_invariant).collect()
}

/// `string.Equals(a, b, OrdinalIgnoreCase)`.
pub fn same(a: &str, b: &str) -> bool {
    equals_ignore_case(a, b)
}

/// `text.StartsWith(prefix, OrdinalIgnoreCase)`.
pub fn starts_with_ignore_case(text: &str, prefix: &str) -> bool {
    let mut text = text.chars().map(simple_upper);
    prefix.chars().map(simple_upper).all(|c| text.next() == Some(c))
}

/// A key under which `OrdinalIgnoreCase` finds a name.
pub fn fold(text: &str) -> String {
    text.chars().map(simple_upper).collect()
}

/// `{value,-width}`: padded on the right to `width` UTF-16 units.
pub fn pad_right(text: &str, width: usize) -> String {
    let mut out = text.to_string();
    for _ in utf16_len(text)..width {
        out.push(' ');
    }
    out
}

/// `PadLeft(width)`.
pub fn pad_left(text: &str, width: usize) -> String {
    let mut out = String::new();
    for _ in utf16_len(text)..width {
        out.push(' ');
    }
    out.push_str(text);
    out
}

/// `NumberStyles.Integer`'s white space: tab to carriage return, and the space.
fn number_white(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{D}' | ' ')
}

/// `long.TryParse(text)`: white space around, one sign, ASCII digits, no overflow.
pub fn parse_i64(text: &str) -> Option<i64> {
    let body = text.trim_matches(number_white);
    let (negative, digits) = match body.chars().next() {
        Some('-') => (true, &body[1..]),
        Some('+') => (false, &body[1..]),
        _ => (false, body),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut value: i128 = 0;
    for b in digits.bytes() {
        value = value * 10 + i128::from(b - b'0');
        if value > i128::from(i64::MAX) + 1 {
            return None;
        }
    }
    let value = if negative { -value } else { value };
    i64::try_from(value).ok()
}

/// `int.TryParse(text)`.
pub fn parse_i32(text: &str) -> Option<i32> {
    parse_i64(text).and_then(|v| i32::try_from(v).ok())
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
    std::fs::read(path).map(|bytes| emusen_galaxia::json::decode(&bytes)).map_err(|e| io_message(&e, path))
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

/// `Environment.CurrentDirectory`.
pub fn current_directory() -> String {
    std::env::current_dir().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_parse_as_dotnet_parses_them() {
        assert_eq!(parse_i32(" 12 "), Some(12));
        assert_eq!(parse_i32("+7"), Some(7));
        assert_eq!(parse_i32("-0"), Some(0));
        assert_eq!(parse_i32("2147483648"), None);
        assert_eq!(parse_i32("-2147483648"), Some(i32::MIN));
        assert_eq!(parse_i32("1_0"), None);
        assert_eq!(parse_i32(""), None);
        assert_eq!(parse_i32("-"), None);
        assert_eq!(parse_i32("٣"), None);
        assert_eq!(parse_i64("-9223372036854775808"), Some(i64::MIN));
        assert_eq!(parse_i64("9223372036854775808"), None);
    }

    #[test]
    fn padding_counts_utf16_units() {
        assert_eq!(pad_right("ab", 4), "ab  ");
        assert_eq!(pad_right("abcdef", 4), "abcdef");
        assert_eq!(pad_right("😀", 3), "😀 ");
        assert_eq!(pad_left("7", 3), "  7");
    }

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
