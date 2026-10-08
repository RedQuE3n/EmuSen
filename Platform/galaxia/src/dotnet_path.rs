//! .NET's `System.IO.Path` rules, reproduced so that a path built here is the string the C# built.
//!
//! Every function works on text, as .NET's do, and takes the platform's style as a value, so the Windows rules are
//! tested on any host. See EmuSen_RustPlatform.md §10.3 for what was measured against .NET and what is argued.

/// Which platform's separators and roots apply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    Unix,
    Windows,
}

impl Style {
    /// The style of the machine this was built for.
    pub const HOST: Style = if cfg!(windows) { Style::Windows } else { Style::Unix };

    /// `Path.DirectorySeparatorChar`.
    pub const fn separator(self) -> char {
        match self {
            Style::Unix => '/',
            Style::Windows => '\\',
        }
    }

    const fn is_separator(self, byte: u8) -> bool {
        match self {
            Style::Unix => byte == b'/',
            Style::Windows => byte == b'\\' || byte == b'/',
        }
    }
}

fn is_drive(byte: u8) -> bool {
    byte.is_ascii_alphabetic()
}

/// `\\?\` or `\\.\` and their forward-slash spellings; `\\?\` exactly is the extended form.
fn is_device(path: &[u8]) -> bool {
    path.starts_with(br"\\?\")
        || (path.len() >= 4
            && Style::Windows.is_separator(path[0])
            && Style::Windows.is_separator(path[1])
            && (path[2] == b'.' || path[2] == b'?')
            && Style::Windows.is_separator(path[3]))
}

fn is_device_unc(path: &[u8]) -> bool {
    path.len() >= 8 && is_device(path) && Style::Windows.is_separator(path[7]) && &path[4..7] == b"UNC"
}

/// `PathInternal.GetRootLength`: how much of the path names its root.
pub fn root_length(style: Style, path: &str) -> usize {
    let path = path.as_bytes();
    match style {
        Style::Unix => usize::from(path.first() == Some(&b'/')),
        Style::Windows => {
            let length = path.len();
            let device = is_device(path);
            let device_unc = device && is_device_unc(path);
            let mut i = 0;
            if (!device || device_unc) && length > 0 && style.is_separator(path[0]) {
                if device_unc || (length > 1 && style.is_separator(path[1])) {
                    i = if device_unc { 8 } else { 2 };
                    let mut separators = 2;
                    while i < length {
                        if style.is_separator(path[i]) {
                            separators -= 1;
                            if separators == 0 {
                                break;
                            }
                        }
                        i += 1;
                    }
                } else {
                    i = 1;
                }
            } else if device {
                i = 4;
                while i < length && !style.is_separator(path[i]) {
                    i += 1;
                }
                if i < length && i > 4 && style.is_separator(path[i]) {
                    i += 1;
                }
            } else if length >= 2 && path[1] == b':' && is_drive(path[0]) {
                i = 2;
                if length > 2 && style.is_separator(path[2]) {
                    i += 1;
                }
            }
            i
        }
    }
}

/// `Path.IsPathRooted`.
pub fn is_rooted(style: Style, path: &str) -> bool {
    let bytes = path.as_bytes();
    match style {
        Style::Unix => bytes.first() == Some(&b'/'),
        Style::Windows => {
            (!bytes.is_empty() && style.is_separator(bytes[0])) || (bytes.len() >= 2 && is_drive(bytes[0]) && bytes[1] == b':')
        }
    }
}

/// `Path.Combine` of two parts: an empty part is skipped, and a rooted second part stands alone.
pub fn combine(style: Style, first: &str, second: &str) -> String {
    if first.is_empty() {
        return second.to_string();
    }
    if second.is_empty() {
        return first.to_string();
    }
    if is_rooted(style, second) {
        return second.to_string();
    }
    let joined = style.is_separator(first.as_bytes()[first.len() - 1]) || style.is_separator(second.as_bytes()[0]);
    if joined { format!("{first}{second}") } else { format!("{first}{}{second}", style.separator()) }
}

/// `Path.Combine` of any number of parts, which is the two-part rule applied from the left.
pub fn combine_all(style: Style, parts: &[&str]) -> String {
    parts.iter().fold(String::new(), |path, part| combine(style, &path, part))
}

/// `Path.GetFileName`: what follows the last separator, or the root.
pub fn file_name(style: Style, path: &str) -> &str {
    let root = root_length(style, path);
    let bytes = path.as_bytes();
    let mut i = bytes.len();
    while i > 0 {
        i -= 1;
        if i < root || style.is_separator(bytes[i]) {
            return &path[i + 1..];
        }
    }
    path
}

/// `Path.GetFileNameWithoutExtension`: the file name up to its last period.
pub fn file_name_without_extension(style: Style, path: &str) -> &str {
    let name = file_name(style, path);
    match name.rfind('.') {
        Some(period) => &name[..period],
        None => name,
    }
}

/// `Path.ChangeExtension` with an extension given: the last period of the last part is where the old one began.
pub fn change_extension(style: Style, path: &str, extension: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let bytes = path.as_bytes();
    let mut stem = bytes.len();
    for i in (0..bytes.len()).rev() {
        if bytes[i] == b'.' {
            stem = i;
            break;
        }
        if style.is_separator(bytes[i]) {
            break;
        }
    }
    let stem = &path[..stem];
    if extension.starts_with('.') { format!("{stem}{extension}") } else { format!("{stem}.{extension}") }
}

/// `PathInternal.NormalizeDirectorySeparators`: runs of separators become one, of the platform's kind.
fn normalize_separators(style: Style, path: &str) -> String {
    let bytes = path.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut start = 0;
    // Windows keeps one doubled separator at the front, which is a UNC or device path's.
    if style == Style::Windows && !bytes.is_empty() && style.is_separator(bytes[0]) {
        out.push(b'\\');
        start = 1;
    }
    for i in start..bytes.len() {
        let mut byte = bytes[i];
        if style.is_separator(byte) {
            if i + 1 < bytes.len() && style.is_separator(bytes[i + 1]) {
                continue;
            }
            byte = style.separator() as u8;
        }
        out.push(byte);
    }
    String::from_utf8(out).expect("only separators were dropped or replaced")
}

/// `Path.GetDirectoryName`: none for an empty path or a root.
pub fn directory_name(style: Style, path: &str) -> Option<String> {
    // PathInternal.IsEffectivelyEmpty: empty everywhere, and all spaces on Windows.
    if path.is_empty() || (style == Style::Windows && path.bytes().all(|b| b == b' ')) {
        return None;
    }
    let bytes = path.as_bytes();
    let root = root_length(style, path);
    let mut end = bytes.len();
    if end <= root {
        return None;
    }
    while end > root {
        end -= 1;
        if style.is_separator(bytes[end]) {
            break;
        }
    }
    while end > root && style.is_separator(bytes[end - 1]) {
        end -= 1;
    }
    Some(normalize_separators(style, &path[..end]))
}

/// `Path.EndsInDirectorySeparator`.
pub fn ends_in_separator(style: Style, path: &str) -> bool {
    path.as_bytes().last().is_some_and(|&b| style.is_separator(b))
}

/// `Path.TrimEndingDirectorySeparator`: one separator off the end, unless the path is a root.
pub fn trim_ending_separator(style: Style, path: &str) -> &str {
    if ends_in_separator(style, path) && path.len() != root_length(style, path) { &path[..path.len() - 1] } else { path }
}

/// `PathInternal.RemoveRelativeSegments`: `//`, `/./` and `/../` taken out, the last by unwinding one part.
fn remove_relative_segments(style: Style, path: &str, root: usize) -> String {
    let bytes = path.as_bytes();
    let mut skip = root;
    if skip > 0 && style.is_separator(bytes[skip - 1]) {
        skip -= 1;
    }
    let mut out: Vec<u8> = bytes[..skip].to_vec();
    let length = bytes.len();
    let mut i = skip;
    while i < length {
        let mut byte = bytes[i];
        if style.is_separator(byte) && i + 1 < length {
            if style.is_separator(bytes[i + 1]) {
                i += 1;
                continue;
            }
            if (i + 2 == length || style.is_separator(bytes[i + 2])) && bytes[i + 1] == b'.' {
                i += 2;
                continue;
            }
            if i + 2 < length && (i + 3 == length || style.is_separator(bytes[i + 3])) && bytes[i + 1] == b'.' && bytes[i + 2] == b'.' {
                let mut unwound = false;
                let mut s = out.len();
                while s > skip {
                    s -= 1;
                    if style.is_separator(out[s]) {
                        // The part before a final ".." keeps its separator when it is the first, as .NET keeps "C:\" for "C:\tmp\..".
                        out.truncate(if i + 3 >= length && s == skip { s + 1 } else { s });
                        unwound = true;
                        break;
                    }
                }
                if !unwound {
                    out.truncate(skip);
                }
                i += 3;
                continue;
            }
        }
        if style.is_separator(byte) {
            byte = style.separator() as u8;
        }
        out.push(byte);
        i += 1;
    }
    String::from_utf8(out).expect("the path was cut only at separators")
}

/// `Path.GetFullPath` by .NET's Unix rule, with the working directory given: rooted, then its relative parts removed, links not followed.
pub fn full_path_from(style: Style, path: &str, working_directory: &str) -> String {
    let rooted = if is_rooted(style, path) { path.to_string() } else { combine(style, working_directory, path) };
    let collapsed = remove_relative_segments(style, &rooted, root_length(style, &rooted));
    if collapsed.is_empty() { style.separator().to_string() } else { collapsed }
}

/// `Path.GetFullPath` on this machine. Windows asks the system, as .NET does; elsewhere it is .NET's own rule.
pub fn full_path(path: &str) -> String {
    #[cfg(windows)]
    {
        match std::path::absolute(path) {
            Ok(absolute) => absolute.to_string_lossy().into_owned(),
            Err(_) => path.to_string(),
        }
    }
    #[cfg(not(windows))]
    {
        let working = std::env::current_dir().map(|d| d.to_string_lossy().into_owned()).unwrap_or_else(|_| "/".to_string());
        full_path_from(Style::Unix, path, &working)
    }
}

/// `char.IsWhiteSpace`, which is Unicode's White_Space and so Rust's own; U+001C to U+001F are not in it, whatever .NET's page says.
pub fn is_white_space(c: char) -> bool {
    c.is_whitespace()
}

/// `string.IsNullOrWhiteSpace`.
pub fn is_blank(text: Option<&str>) -> bool {
    text.is_none_or(|t| t.chars().all(is_white_space))
}

/// One character's upper case as .NET's ordinal comparison sees it: Unicode's simple mapping, one character to one, less the two that would turn a letter outside ASCII into one inside it.
pub(crate) fn simple_upper(c: char) -> char {
    match c {
        // The dotless i and the long s keep themselves, where Unicode sends them to I and S.
        '\u{131}' | '\u{17F}' => c,
        // Greek with a subscript iota: the standard library gives the two-letter full mapping, and the simple one is the capital beside it.
        '\u{1F80}'..='\u{1F87}' | '\u{1F90}'..='\u{1F97}' | '\u{1FA0}'..='\u{1FA7}' => char::from_u32(c as u32 + 8).unwrap_or(c),
        '\u{1FB3}' | '\u{1FC3}' | '\u{1FF3}' => char::from_u32(c as u32 + 9).unwrap_or(c),
        _ => {
            let mut upper = c.to_uppercase();
            match (upper.next(), upper.next()) {
                (Some(one), None) => one,
                // A character whose only upper case is several characters has no simple one.
                _ => c,
            }
        }
    }
}

/// `string.Equals(a, b, StringComparison.OrdinalIgnoreCase)`.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.chars().map(simple_upper).eq(b.chars().map(simple_upper))
}

/// `a.EndsWith(suffix, StringComparison.OrdinalIgnoreCase)`.
pub fn ends_with_ignore_case(a: &str, suffix: &str) -> bool {
    let (a, suffix): (Vec<char>, Vec<char>) = (a.chars().map(simple_upper).collect(), suffix.chars().map(simple_upper).collect());
    a.ends_with(&suffix)
}

#[cfg(test)]
mod tests {
    use super::Style::{Unix, Windows};
    use super::*;

    #[test]
    fn combine_skips_empty_parts_and_restarts_at_a_rooted_one() {
        assert_eq!(combine(Unix, "/a", "b"), "/a/b");
        assert_eq!(combine(Unix, "/a/", "b"), "/a/b");
        assert_eq!(combine(Unix, "", "b"), "b");
        assert_eq!(combine(Unix, "/a", ""), "/a");
        assert_eq!(combine(Unix, "/a", "/b"), "/b");
        assert_eq!(combine_all(Unix, &["/r", "EmuSen.DianaOS", "DianaOS", "Usr", "Home"]), "/r/EmuSen.DianaOS/DianaOS/Usr/Home");
        assert_eq!(combine(Windows, r"C:\a", "b"), r"C:\a\b");
        assert_eq!(combine(Windows, "C:", "b"), r"C:\b");
        assert_eq!(combine(Windows, r"C:\a", r"D:\b"), r"D:\b");
        assert_eq!(combine(Windows, r"C:\a", r"\b"), r"\b");
        assert_eq!(combine(Windows, "a/", "b"), "a/b");
    }

    #[test]
    fn file_names_and_extensions() {
        assert_eq!(file_name(Unix, "/a/b.c"), "b.c");
        assert_eq!(file_name(Unix, "/a/b/"), "");
        assert_eq!(file_name(Unix, "b"), "b");
        assert_eq!(file_name_without_extension(Unix, "/roms/Game (USA) (Rev 1).sfc"), "Game (USA) (Rev 1)");
        assert_eq!(file_name_without_extension(Unix, "a.tar.gz"), "a.tar");
        assert_eq!(file_name_without_extension(Unix, ".hidden"), "");
        assert_eq!(file_name_without_extension(Unix, "name."), "name");
        assert_eq!(file_name_without_extension(Unix, r"C:\roms\a.sfc"), r"C:\roms\a");
        assert_eq!(file_name_without_extension(Windows, r"C:\roms\a.sfc"), "a");
        assert_eq!(file_name(Windows, "C:a.sfc"), "a.sfc");
        assert_eq!(change_extension(Unix, "/s/a.state", ".png"), "/s/a.png");
        assert_eq!(change_extension(Unix, "/s.d/a", ".png"), "/s.d/a.png");
        assert_eq!(change_extension(Unix, "/s/a.slot2.state", "png"), "/s/a.slot2.png");
        assert_eq!(change_extension(Unix, "/s/", ".png"), "/s/.png");
        assert_eq!(change_extension(Unix, "", ".png"), "");
    }

    #[test]
    fn directory_names() {
        assert_eq!(directory_name(Unix, "/a/b").as_deref(), Some("/a"));
        assert_eq!(directory_name(Unix, "/a").as_deref(), Some("/"));
        assert_eq!(directory_name(Unix, "/"), None);
        assert_eq!(directory_name(Unix, ""), None);
        assert_eq!(directory_name(Unix, "a").as_deref(), Some(""));
        assert_eq!(directory_name(Unix, "a/b/").as_deref(), Some("a/b"));
        assert_eq!(directory_name(Unix, "/a//b//c").as_deref(), Some("/a/b"));
        assert_eq!(directory_name(Windows, r"C:\a\b").as_deref(), Some(r"C:\a"));
        assert_eq!(directory_name(Windows, r"C:\a").as_deref(), Some(r"C:\"));
        assert_eq!(directory_name(Windows, r"C:\"), None);
        assert_eq!(directory_name(Windows, "C:/a/b").as_deref(), Some(r"C:\a"));
        assert_eq!(directory_name(Windows, r"\\server\share\a").as_deref(), Some(r"\\server\share"));
        assert_eq!(directory_name(Windows, r"\\server\share"), None);
    }

    #[test]
    fn roots() {
        assert_eq!(root_length(Unix, "/a"), 1);
        assert_eq!(root_length(Unix, "a"), 0);
        assert_eq!(root_length(Windows, r"C:\a"), 3);
        assert_eq!(root_length(Windows, "C:a"), 2);
        assert_eq!(root_length(Windows, r"\a"), 1);
        assert_eq!(root_length(Windows, r"\\server\share\a"), 14);
        assert_eq!(root_length(Windows, r"\\?\C:\a"), 7);
        assert_eq!(root_length(Windows, r"\\?\UNC\server\share\a"), 20);
        assert_eq!(trim_ending_separator(Unix, "/a/"), "/a");
        assert_eq!(trim_ending_separator(Unix, "/a//"), "/a/");
        assert_eq!(trim_ending_separator(Unix, "/"), "/");
        assert_eq!(trim_ending_separator(Windows, r"C:\"), r"C:\");
        assert_eq!(trim_ending_separator(Windows, r"C:\a\"), r"C:\a");
    }

    #[test]
    fn full_paths_lose_their_relative_parts_and_keep_a_trailing_separator() {
        assert_eq!(full_path_from(Unix, "/a/./b/../c", "/w"), "/a/c");
        assert_eq!(full_path_from(Unix, "/a/b/", "/w"), "/a/b/");
        assert_eq!(full_path_from(Unix, "/a/b/.", "/w"), "/a/b");
        assert_eq!(full_path_from(Unix, "/a/b/..", "/w"), "/a");
        assert_eq!(full_path_from(Unix, "/a/..", "/w"), "/");
        assert_eq!(full_path_from(Unix, "/..", "/w"), "/");
        assert_eq!(full_path_from(Unix, "/../a", "/w"), "/a");
        assert_eq!(full_path_from(Unix, "//a///b", "/w"), "/a/b");
        assert_eq!(full_path_from(Unix, "x/../y", "/w"), "/w/y");
        assert_eq!(full_path_from(Unix, "..", "/w/v"), "/w");
        assert_eq!(full_path_from(Unix, "/a/...", "/w"), "/a/...");
        assert_eq!(full_path_from(Unix, "/a/.b/..c", "/w"), "/a/.b/..c");
        assert_eq!(full_path_from(Windows, r"C:\a\.\b\..\c", r"C:\w"), r"C:\a\c");
        assert_eq!(full_path_from(Windows, "C:/a/b/..", r"C:\w"), r"C:\a");
        assert_eq!(full_path_from(Windows, r"C:\tmp\..", r"C:\w"), r"C:\");
        assert_eq!(full_path_from(Windows, r"x\y", r"C:\w"), r"C:\w\x\y");
        assert_eq!(full_path_from(Windows, r"C:a\..", r"C:\w"), "C:");
        assert_eq!(full_path_from(Windows, r"C:a\..\..\b", r"C:\w"), r"C:\b");
    }

    #[test]
    fn blanks_and_case() {
        assert!(is_blank(None));
        assert!(is_blank(Some("")));
        assert!(is_blank(Some(" \t\u{85}\u{A0}\u{2028}\u{3000}")));
        assert!(!is_blank(Some("\u{1C}")));
        assert!(!is_blank(Some("\u{180E}")));
        assert!(!is_blank(Some(" a ")));
        assert!(equals_ignore_case("MacOS", "macos"));
        assert!(equals_ignore_case("/Home/É", "/home/é"));
        assert!(!equals_ignore_case("straße", "STRASSE"));
        assert!(!equals_ignore_case("ı", "I"));
        assert!(!equals_ignore_case("ſ", "S"));
        assert!(equals_ignore_case("\u{1F80}", "\u{1F88}"));
        assert!(equals_ignore_case("\u{1FF3}", "\u{1FFC}"));
        assert!(!equals_ignore_case("k", "\u{212A}"));
        assert!(ends_with_ignore_case("/Applications/EmuSen.APP", ".app"));
        assert!(!ends_with_ignore_case("app", ".app"));
    }
}
