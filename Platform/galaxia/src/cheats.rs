//! The saved cheat lists as a set of files: which there are, and what one may be called. The static half of the C#
//! `CheatFile`; a list's own contents are the `CheatFile` model. See EmuSen_Config_Reference.md §3.4.

use crate::dotnet_path::{self as path, Style, equals_ignore_case, is_blank};
use crate::dotnet_text::compare_ignore_case;
use crate::models::CHEATS_CATEGORY;
use crate::tree::Tree;

/// Whether this platform's file names match a pattern without regard to case, as .NET's enumeration does on Windows and macOS.
const PATTERNS_IGNORE_CASE: bool = cfg!(any(windows, target_os = "macos"));

/// Whether a file name ends as a pattern `*<suffix>` asks.
pub(crate) fn ends_as(name: &str, suffix: &str) -> bool {
    match name.len().checked_sub(suffix.len()).filter(|&at| name.is_char_boundary(at)) {
        Some(at) if PATTERNS_IGNORE_CASE => equals_ignore_case(&name[at..], suffix),
        Some(at) => &name[at..] == suffix,
        None => false,
    }
}

/// The files directly in a folder, by name: what is not a folder, a link judged by what it names and a dangling one counted, as .NET lists them.
pub(crate) fn files_in(directory: &str) -> std::io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in std::fs::read_dir(directory)? {
        let entry = entry?;
        if !std::fs::metadata(entry.path()).is_ok_and(|m| m.is_dir()) {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    Ok(names)
}

/// `CheatFile.DirectoryPath`.
pub fn directory(tree: &Tree) -> String {
    path::combine(Style::HOST, &tree.config(), CHEATS_CATEGORY)
}

/// `CheatFile.ListNames`: the saved lists by name, in .NET's caseless order; none when the folder cannot be listed.
pub fn names(tree: &Tree) -> Vec<String> {
    let Ok(files) = files_in(&directory(tree)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = files.iter().filter(|file| ends_as(file, ".json")).map(|file| path::file_name_without_extension(Style::HOST, file).to_string()).filter(|name| !name.is_empty()).collect();
    names.sort_by(|a, b| compare_ignore_case(a, b));
    names
}

/// The characters a file name may not hold on a platform, as `Path.GetInvalidFileNameChars` lists them.
fn invalid_in_a_file_name(style: Style, c: char) -> bool {
    match style {
        Style::Unix => c == '\0' || c == '/',
        Style::Windows => (c as u32) < 32 || matches!(c, '"' | '<' | '>' | '|' | ':' | '*' | '?' | '\\' | '/'),
    }
}

/// `CheatFile.IsValidName`: a name the player typed is checked and never rewritten, since rewriting it would save to a file they did not name.
pub fn is_valid_name(style: Style, name: Option<&str>) -> bool {
    match name {
        Some(name) => !is_blank(Some(name)) && !name.chars().any(|c| invalid_in_a_file_name(style, c)) && name != "." && name != "..",
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::TempDir;
    use crate::tree::Override;

    #[test]
    fn the_lists_are_the_json_files_in_the_folder_in_caseless_order() {
        let temp = TempDir::new("cheats");
        let mut tree = Tree::default();
        tree.set_override(Override::Config, Some(temp.path().to_string()));
        assert_eq!(names(&tree), Vec::<String>::new());
        std::fs::create_dir_all(temp.join("cheats/folder.json")).unwrap();
        for file in ["zelda.json", "Mario.json", "alttp.json", ".json", "notes.txt", "a.b.json", "_x.json", "é.json"] {
            std::fs::write(temp.join(&format!("cheats/{file}")), "{}").unwrap();
        }
        // An underscore sorts after the letters: .NET compares ASCII by its upper case, and '_' follows 'Z'.
        assert_eq!(names(&tree), ["a.b", "alttp", "Mario", "zelda", "_x", "é"]);
        assert_eq!(directory(&tree), temp.join("cheats"));
    }

    #[test]
    fn a_name_is_refused_for_what_a_file_name_cannot_hold() {
        for name in ["Zelda", "a b", "é", "a.b", "...", "con", "a\\b", "a:b"] {
            assert!(is_valid_name(Style::Unix, Some(name)), "{name}");
        }
        for name in ["", "  ", ".", "..", "a/b", "a\0b"] {
            assert!(!is_valid_name(Style::Unix, Some(name)), "{name}");
        }
        assert!(!is_valid_name(Style::Unix, None));
        for name in ["a\\b", "a:b", "a*b", "a?b", "a\"b", "a<b", "a|b", "a\u{1F}b"] {
            assert!(!is_valid_name(Style::Windows, Some(name)), "{name}");
        }
        assert!(is_valid_name(Style::Windows, Some("Zelda (U)")));
    }
}
