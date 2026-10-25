//! What a ROM's save files are called, and where: the C# `SaveLibrary`. See EmuSen_Galaxia.md §5.

use crate::dotnet_path::{self as path, Style};
use crate::tree::Tree;

pub const SRAM_EXTENSION: &str = ".srm";
pub const STATE_EXTENSION: &str = ".state";
/// The slot that keeps the plain `<rom>.state` name.
pub const DEFAULT_STATE_SLOT: i32 = 1;

/// The name a console started empty goes by, `SaveLibrary.ConsoleSessionName`.
pub fn console_session_name(console: &str) -> String {
    format!("{console} (no disc)")
}

const STYLE: Style = Style::HOST;

fn stem(rom_path: &str) -> &str {
    path::file_name_without_extension(STYLE, rom_path)
}

/// The states' directory: the caller's, unless it is absent or blank. A parameter and never a settings read; see §5.1.
fn state_directory(tree: &Tree, directory_override: Option<&str>) -> String {
    match directory_override {
        Some(directory) if !path::is_blank(Some(directory)) => directory.to_string(),
        _ => tree.save_states(),
    }
}

/// `Saves/<console>/<stem>.srm`.
pub fn sram_path(tree: &Tree, rom_path: &str, console: &str) -> String {
    path::combine_all(STYLE, &[&tree.saves(), console, &format!("{}{SRAM_EXTENSION}", stem(rom_path))])
}

/// `Saves/<stem>.srm`, where saves went before each console had a folder; read to copy them across, never written.
pub fn flat_sram_path(tree: &Tree, rom_path: &str) -> String {
    path::combine(STYLE, &tree.saves(), &format!("{}{SRAM_EXTENSION}", stem(rom_path)))
}

/// `<stem>.state` for slot 1 and `<stem>.slotN.state` for the rest.
pub fn state_path(tree: &Tree, rom_path: &str, slot: i32, directory_override: Option<&str>) -> String {
    let suffix = if slot == DEFAULT_STATE_SLOT { String::new() } else { format!(".slot{slot}") };
    path::combine(STYLE, &state_directory(tree, directory_override), &format!("{}{suffix}{STATE_EXTENSION}", stem(rom_path)))
}

/// `<stem>.resume.state`, a name no slot can take.
pub fn resume_state_path(tree: &Tree, rom_path: &str, directory_override: Option<&str>) -> String {
    path::combine(STYLE, &state_directory(tree, directory_override), &format!("{}.resume{STATE_EXTENSION}", stem(rom_path)))
}

/// The name a package's saves and states go by: its folder's, less the extension, since the folder is named after the
/// game's file when it is made and does not change when the game moves (EmuSen_GamePackages.md §3.2, §14).
pub fn package_stem(root: &str) -> &str {
    path::file_name_without_extension(STYLE, path::trim_ending_separator(STYLE, root))
}

/// The name a package's files go by: its own, and for a game opened with its patch list applied, `.patched-<key>` after
/// it, `key` being `package::save_key`'s, so that a hack never reaches the original's saves (EmuSen_GamePackages.md §5.5).
pub fn package_name(root: &str, key: Option<&str>) -> String {
    match key.filter(|k| !k.is_empty()) {
        Some(key) => format!("{}.patched-{key}", package_stem(root)),
        None => package_stem(root).to_string(),
    }
}

/// `<package>/saves/<name>.srm`: a packaged game's battery save, whatever its console or the file it opened from.
pub fn package_sram_path(root: &str, key: Option<&str>) -> String {
    path::combine_all(STYLE, &[root, crate::package::SAVES_FOLDER, &format!("{}{SRAM_EXTENSION}", package_name(root, key))])
}

/// `<package>/states/<name>.state` for slot 1 and `.slotN.state` for the rest; the state folder setting gives way to it.
pub fn package_state_path(root: &str, slot: i32, key: Option<&str>) -> String {
    let suffix = if slot == DEFAULT_STATE_SLOT { String::new() } else { format!(".slot{slot}") };
    path::combine_all(STYLE, &[root, crate::package::STATES_FOLDER, &format!("{}{suffix}{STATE_EXTENSION}", package_name(root, key))])
}

/// `<package>/states/<name>.resume.state`.
pub fn package_resume_state_path(root: &str, key: Option<&str>) -> String {
    path::combine_all(STYLE, &[root, crate::package::STATES_FOLDER, &format!("{}.resume{STATE_EXTENSION}", package_name(root, key))])
}

/// The picture beside a state: the same name with `.png`.
pub fn picture_path(state_path: &str) -> String {
    path::change_extension(STYLE, state_path, ".png")
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn tree() -> Tree {
        Tree { root: "/r".to_string(), ..Tree::default() }
    }

    #[test]
    fn a_save_is_named_for_the_rom_and_filed_by_console() {
        assert_eq!(sram_path(&tree(), "/roms/SNES/USA/Zelda (U).sfc", "SNES"), "/r/home/Saves/SNES/Zelda (U).srm");
        assert_eq!(flat_sram_path(&tree(), "/roms/SNES/USA/Zelda (U).sfc"), "/r/home/Saves/Zelda (U).srm");
    }

    #[test]
    fn slot_one_is_the_unsuffixed_name() {
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", 1, None), "/r/home/Saves/Save States/a.b.state");
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", 2, None), "/r/home/Saves/Save States/a.b.slot2.state");
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", 0, None), "/r/home/Saves/Save States/a.b.slot0.state");
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", -3, None), "/r/home/Saves/Save States/a.b.slot-3.state");
    }

    #[test]
    fn a_blank_override_falls_back_and_the_roms_folder_never_leaks() {
        assert_eq!(state_path(&tree(), "/roms/a.nes", 1, Some("  ")), "/r/home/Saves/Save States/a.state");
        assert_eq!(state_path(&tree(), "/roms/a.nes", 1, Some("")), "/r/home/Saves/Save States/a.state");
        assert_eq!(state_path(&tree(), "/roms/a.nes", 1, Some("/elsewhere")), "/elsewhere/a.state");
        assert_eq!(resume_state_path(&tree(), "/roms/a.nes", None), "/r/home/Saves/Save States/a.resume.state");
        assert_eq!(resume_state_path(&tree(), "/roms/a.nes", Some("/elsewhere")), "/elsewhere/a.resume.state");
    }

    #[test]
    fn a_packaged_game_keeps_its_saves_and_states_in_its_package_by_its_own_name() {
        assert_eq!(package_sram_path("/p/SNES/Zelda (U).sfc", None), "/p/SNES/Zelda (U).sfc/saves/Zelda (U).srm");
        assert_eq!(package_state_path("/p/a.b.nes/", 1, None), "/p/a.b.nes/states/a.b.state");
        assert_eq!(package_state_path("/p/a.b.nes", 3, Some("")), "/p/a.b.nes/states/a.b.slot3.state");
        assert_eq!(package_resume_state_path("/p/g.nes", None), "/p/g.nes/states/g.resume.state");
    }

    #[test]
    fn a_patched_game_keeps_its_saves_and_states_apart_by_its_lists_key() {
        assert_eq!(package_sram_path("/p/SNES/Zelda (U).sfc", Some("0a1b2c3d")), "/p/SNES/Zelda (U).sfc/saves/Zelda (U).patched-0a1b2c3d.srm");
        assert_eq!(package_state_path("/p/a.b.nes", 1, Some("0a1b2c3d")), "/p/a.b.nes/states/a.b.patched-0a1b2c3d.state");
        assert_eq!(package_state_path("/p/a.b.nes", 3, Some("0a1b2c3d")), "/p/a.b.nes/states/a.b.patched-0a1b2c3d.slot3.state");
        assert_eq!(package_resume_state_path("/p/g.nes", Some("0a1b2c3d")), "/p/g.nes/states/g.patched-0a1b2c3d.resume.state");
    }

    #[test]
    fn a_states_picture_takes_its_name() {
        assert_eq!(picture_path("/s/a.slot2.state"), "/s/a.slot2.png");
        assert_eq!(picture_path("/s/a.resume.state"), "/s/a.resume.png");
    }
}
