//! A folder setting's value after the rules of EmuSen_Config_Reference.md §7.7: the Rust half of the C#
//! `SettingFolders.Checked` and `FolderValue`, which the platform's file services read (EmuSen_Backend.md §15). Blank is
//! unset, `~` and `~/` are the home, a relative path or a file is refused, and a folder that does not exist is said.

use crate::files::{directory_exists, file_exists};
use crate::path_key;
use std::path::{Path, PathBuf};

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

/// `~` alone, or followed by a separator, is the player's home; `~name` is not expanded and so stays relative.
fn expanded(value: &str, home: &Path) -> PathBuf {
    let windows_tilde = cfg!(windows) && value.starts_with("~\\");
    if value != "~" && !value.starts_with("~/") && !windows_tilde {
        return PathBuf::from(value);
    }
    match value.get(2..).unwrap_or("") {
        "" => home.to_path_buf(),
        rest => home.join(rest),
    }
}

/// `SettingFolders.Checked`, with the player's home passed in by the host.
pub fn checked(value: Option<&str>, home: impl AsRef<Path>) -> FolderValue {
    let Some(named) = value.filter(|v| !v.trim().is_empty()) else {
        return FolderValue { state: FolderState::Unset, path: None, written: value.unwrap_or("").to_string() };
    };
    let written = named.trim().to_string();
    let expanded = expanded(&written, home.as_ref());
    if !expanded.is_absolute() {
        return FolderValue { state: FolderState::Relative, path: None, written };
    }
    let full = path_key::key_text(&expanded);
    if file_exists(&full) {
        return FolderValue { state: FolderState::File, path: None, written };
    }
    let state = if directory_exists(&full) { FolderState::Usable } else { FolderState::Missing };
    FolderValue { state, path: Some(full), written }
}

/// `SettingFolders.SameFolder`: equal once full, part by part; by case on Linux alone.
pub fn same_folder(a: &str, b: &str) -> bool {
    let (a, b) = (path_key::key(a), path_key::key(b));
    if cfg!(target_os = "linux") {
        return a == b;
    }
    let parts = |p: &Path| p.components().map(|c| c.as_os_str().to_string_lossy().to_uppercase()).collect::<Vec<_>>();
    parts(&a) == parts(&b)
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
        assert_eq!((v.state, v.path.as_deref().map(Path::new), v.written.as_str()), (FolderState::Usable, Some(dir.path()), "~"));
        let v = checked(Some("~/absent"), dir.path());
        assert_eq!(v.state, FolderState::Missing);
        assert_eq!(v.problem().unwrap(), format!("The folder {} does not exist.", dir.join("absent").display()));
    }

    #[test]
    fn a_file_is_refused_and_paths_are_made_full() {
        let dir = TempDir::new("setting-folders-file");
        std::fs::write(dir.join("f"), b"x").unwrap();
        let v = checked(Some(&dir.join("f").to_string_lossy()), "/");
        assert_eq!((v.state, v.problem().unwrap()), (FolderState::File, format!("\"{}\" is a file, not a folder, so it is not used.", dir.join("f").display())));
        let v = checked(Some(&format!("{}/./sub/..", dir.path().display())), "/");
        assert_eq!((v.state, v.path.as_deref().map(Path::new)), (FolderState::Usable, Some(dir.path())));
        assert!(same_folder(&format!("{}/", dir.path().display()), &dir.path().to_string_lossy()));
    }
}
