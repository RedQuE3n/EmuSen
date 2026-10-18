//! A folder setting's value after the rules of EmuSen_Config_Reference.md §7.7: the Rust half of the C#
//! `SettingFolders.Checked` and `FolderValue`, which the platform's file services read (EmuSen_Backend.md §15). Blank is
//! unset, `~` and `~/` are the home, a relative path or a file is refused, and a folder that does not exist is said.

use crate::dotnet_path::{self as path, Style, is_blank};
use crate::tree::{directory_exists, file_exists};

/// How a checked folder setting stands, `FolderState`'s members in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FolderState {
    Unset,
    Usable,
    Missing,
    Relative,
    File,
}

impl FolderState {
    /// The member's name as the C# enumeration spells it.
    pub fn name(self) -> &'static str {
        match self {
            FolderState::Unset => "Unset",
            FolderState::Usable => "Usable",
            FolderState::Missing => "Missing",
            FolderState::Relative => "Relative",
            FolderState::File => "File",
        }
    }
}

/// `FolderValue`: the full path a value names when it names one, how it stands, and what was written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FolderValue {
    pub state: FolderState,
    pub path: Option<String>,
    pub written: String,
}

impl FolderValue {
    /// Whether readers use the path: a usable folder, or one that is not there yet and so holds nothing.
    pub fn searched(&self) -> bool {
        matches!(self.state, FolderState::Usable | FolderState::Missing)
    }

    /// The sentence a page or a log says about a value that is not simply usable, `FolderValue.Problem`'s words.
    pub fn problem(&self) -> Option<String> {
        match self.state {
            FolderState::Relative => Some(format!("\"{}\" is not a full path, so it is not used. Type a full path, or one starting with ~.", self.written)),
            FolderState::File => Some(format!("\"{}\" is a file, not a folder, so it is not used.", self.written)),
            FolderState::Missing => Some(format!("The folder {} does not exist.", self.path.as_deref().unwrap_or(""))),
            _ => None,
        }
    }
}

/// `Path.IsPathFullyQualified`: rooted with no part left to a current drive or directory.
pub fn is_fully_qualified(style: Style, value: &str) -> bool {
    let b = value.as_bytes();
    match style {
        Style::Unix => b.first() == Some(&b'/'),
        Style::Windows => {
            let separator = |c: u8| c == b'\\' || c == b'/';
            (b.len() >= 2 && separator(b[0]) && separator(b[1])) || (b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && separator(b[2]))
        }
    }
}

/// `Path.Join` of two parts: an empty part is the other, and one separator between them.
pub fn join(style: Style, first: &str, second: &str) -> String {
    if first.is_empty() {
        return second.to_string();
    }
    if second.is_empty() {
        return first.to_string();
    }
    if path::ends_in_separator(style, first) || second.starts_with(style.separator()) || (style == Style::Windows && second.starts_with('/')) {
        format!("{first}{second}")
    } else {
        format!("{first}{}{second}", style.separator())
    }
}

/// `~` alone, or followed by a separator, is the player's home; `~name` is not expanded and so stays relative.
fn expanded(value: &str, home: &str) -> String {
    let windows_tilde = cfg!(windows) && value.starts_with("~\\");
    if value != "~" && !value.starts_with("~/") && !windows_tilde {
        return value.to_string();
    }
    if value.len() == 1 { home.to_string() } else { join(Style::HOST, home, &value[2..]) }
}

/// `SettingFolders.Checked`, with the home `Environment.GetFolderPath(UserProfile)` gives passed in by the host.
pub fn checked(value: Option<&str>, home: &str) -> FolderValue {
    let Some(named) = value.filter(|v| !is_blank(Some(v))) else {
        return FolderValue { state: FolderState::Unset, path: None, written: value.unwrap_or("").to_string() };
    };
    let written = emusen_dotnet::text::trim(named).to_string();
    let expanded = expanded(&written, home);
    if !is_fully_qualified(Style::HOST, &expanded) {
        return FolderValue { state: FolderState::Relative, path: None, written };
    }
    let full = path::full_path(&expanded);
    if file_exists(&full) {
        return FolderValue { state: FolderState::File, path: None, written };
    }
    let state = if directory_exists(&full) { FolderState::Usable } else { FolderState::Missing };
    FolderValue { state, path: Some(full), written }
}

/// `SettingFolders.SameFolder`: equal once full and without a trailing separator; by case on Linux alone.
pub fn same_folder(a: &str, b: &str) -> bool {
    let (a, b) = (path::full_path(a), path::full_path(b));
    let (a, b) = (path::trim_ending_separator(Style::HOST, &a), path::trim_ending_separator(Style::HOST, &b));
    if cfg!(target_os = "linux") { a == b } else { path::equals_ignore_case(a, b) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;

    #[test]
    fn blank_is_unset_and_keeps_what_was_written() {
        for blank in [None, Some(""), Some("   "), Some("\t")] {
            let v = checked(blank, "/home/p");
            assert_eq!((v.state, v.path, v.written.as_str()), (FolderState::Unset, None, blank.unwrap_or("")));
        }
    }

    #[test]
    fn relative_and_tilde_names_are_refused_and_tilde_alone_is_home() {
        assert_eq!(checked(Some("BIOS"), "/home/p").state, FolderState::Relative);
        assert_eq!(checked(Some("~name/x"), "/home/p").state, FolderState::Relative);
        let dir = TempDir::new("setting-folders");
        let v = checked(Some(" ~ "), dir.path());
        assert_eq!((v.state, v.path.as_deref(), v.written.as_str()), (FolderState::Usable, Some(dir.path()), "~"));
        let v = checked(Some("~/absent"), dir.path());
        assert_eq!(v.state, FolderState::Missing);
        assert_eq!(v.problem().unwrap(), format!("The folder {} does not exist.", dir.join("absent")));
    }

    #[test]
    fn a_file_is_refused_and_paths_are_made_full() {
        let dir = TempDir::new("setting-folders-file");
        std::fs::write(dir.join("f"), b"x").unwrap();
        let v = checked(Some(&dir.join("f")), "/");
        assert_eq!((v.state, v.problem().unwrap()), (FolderState::File, format!("\"{}\" is a file, not a folder, so it is not used.", dir.join("f"))));
        let v = checked(Some(&format!("{}/./sub/..", dir.path())), "/");
        assert_eq!((v.state, v.path.as_deref()), (FolderState::Usable, Some(dir.path())));
        assert!(same_folder(&format!("{}/", dir.path()), dir.path()));
    }

    #[test]
    fn join_puts_one_separator_between_parts() {
        assert_eq!(join(Style::Unix, "/h", "x"), "/h/x");
        assert_eq!(join(Style::Unix, "/h/", "x"), "/h/x");
        assert_eq!(join(Style::Unix, "/h", ""), "/h");
        assert_eq!(join(Style::Unix, "", "x"), "x");
    }
}
