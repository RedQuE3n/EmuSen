//! The saved cheat lists as a set of files: which there are, and what one may be called. The static half of the C#
//! `CheatFile`, and `GameCheats`' naming of a game's own list; a list's own contents are the `CheatFile` model. See
//! EmuSen_Config_Reference.md §3.4 and §3.4b.

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

/// A derived name with each character a file name cannot hold made `_`: it was not typed, so it is mended and not refused.
pub fn sanitized(style: Style, text: &str) -> String {
    text.chars().map(|c| if invalid_in_a_file_name(style, c) { '_' } else { c }).collect()
}

/// Where a frontend keeps one game's own list: a config file of `category`, named `<name>.json`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameList {
    /// The ROM's file name without its extension, sanitized, as the list is shown to the player.
    pub name: String,
    /// `cheats/<console>`, or the flat `cheats` for a console that gives no folder.
    pub category: String,
}

impl GameList {
    /// The list's file name in its category.
    pub fn file_name(&self) -> String {
        format!("{}.json", self.name)
    }
}

/// `GameCheats.For`: none for a path that gives no name a list may have; a console blank or no folder's name falls back to the flat folder.
pub fn game_list(style: Style, console: Option<&str>, rom_path: Option<&str>) -> Option<GameList> {
    let name = sanitized(style, path::file_name_without_extension(style, rom_path.filter(|p| !p.is_empty())?));
    if !is_valid_name(style, Some(&name)) {
        return None;
    }
    let folder = console.filter(|c| !is_blank(Some(c))).map(|c| sanitized(style, c)).filter(|f| is_valid_name(style, Some(f)));
    let category = folder.map_or_else(|| CHEATS_CATEGORY.to_string(), |f| path::combine(style, CHEATS_CATEGORY, &f));
    Some(GameList { name, category })
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

    fn list(style: Style, console: Option<&str>, rom_path: &str) -> Option<(String, String)> {
        game_list(style, console, Some(rom_path)).map(|list| (list.name, list.category))
    }

    fn some(name: &str, category: &str) -> Option<(String, String)> {
        Some((name.to_string(), category.to_string()))
    }

    #[test]
    fn a_games_list_is_its_rom_stem_in_its_consoles_folder() {
        assert_eq!(list(Style::Unix, Some("SNES"), "/roms/Gamma.smc"), some("Gamma", "cheats/SNES"));
        assert_eq!(game_list(Style::Unix, Some("SNES"), Some("Gamma.smc")).unwrap().file_name(), "Gamma.json");
        assert_eq!(list(Style::Windows, Some("SNES"), r"C:\roms\Gamma.smc"), some("Gamma", r"cheats\SNES"));
        assert_eq!(list(Style::Unix, Some("SNES"), r"C:\roms\Gamma.smc"), some(r"C:\roms\Gamma", "cheats/SNES"));
        // No extension, a dotted name, a final period: as Path.GetFileNameWithoutExtension, the last period only.
        assert_eq!(list(Style::Unix, Some("NES"), "/roms/Gamma"), some("Gamma", "cheats/NES"));
        assert_eq!(list(Style::Unix, Some("NES"), "/roms/Super.Mario.Bros.nes"), some("Super.Mario.Bros", "cheats/NES"));
        assert_eq!(list(Style::Unix, Some("NES"), "/roms.d/name."), some("name", "cheats/NES"));
        assert_eq!(list(Style::Unix, Some("Game Boy"), "é 日本 😀.gb"), some("é 日本 😀", "cheats/Game Boy"));
    }

    #[test]
    fn what_a_file_name_cannot_hold_is_mended_by_platform() {
        assert_eq!(list(Style::Unix, Some("a/b"), "x\0y:z?.smc"), some("x_y:z?", "cheats/a_b"));
        assert_eq!(list(Style::Unix, Some("a\0b"), "x.y.smc"), some("x.y", "cheats/a_b"));
        assert_eq!(list(Style::Windows, Some("Sega: CD"), "/roms/Mario|Luigi? <1>.md"), some("Mario_Luigi_ _1_", r"cheats\Sega_ CD"));
        assert_eq!(list(Style::Windows, Some("a\u{1F}b\"\\c"), "x*y.md"), some("x_y", r"cheats\a_b__c"));
        assert_eq!(sanitized(Style::Unix, "a\\b:c"), "a\\b:c");
        assert_eq!(sanitized(Style::Windows, "a\\b:c"), "a_b_c");
    }

    #[test]
    fn a_console_that_gives_no_folder_falls_back_to_the_flat_one() {
        for console in [None, Some(""), Some("  "), Some("\t\u{2003}"), Some("."), Some("..")] {
            assert_eq!(list(Style::Unix, console, "/roms/Gamma.smc"), some("Gamma", "cheats"), "{console:?}");
        }
        // Mended first and judged after: a console of one bad character is a folder of one underscore.
        assert_eq!(list(Style::Unix, Some("/"), "Gamma.smc"), some("Gamma", "cheats/_"));
        assert_eq!(list(Style::Unix, Some(" SNES "), "Gamma.smc"), some("Gamma", "cheats/ SNES "));
    }

    #[test]
    fn a_path_that_gives_no_name_gives_no_list() {
        assert_eq!(game_list(Style::Unix, Some("SNES"), None), None);
        for rom_path in ["", "/roms/", "/roms/.smc", ".hidden", "/roms/   .smc", "..", "...", "/roms/.."] {
            assert_eq!(list(Style::Unix, Some("SNES"), rom_path), None, "{rom_path:?}");
        }
        assert_eq!(list(Style::Windows, Some("SNES"), r"C:\roms\"), None);
        assert_eq!(list(Style::Windows, Some("SNES"), "C:"), None);
        // Only "." and ".." are refused: four periods leave three, which is a name.
        assert_eq!(list(Style::Unix, Some("SNES"), "...."), some("...", "cheats/SNES"));
    }
}
