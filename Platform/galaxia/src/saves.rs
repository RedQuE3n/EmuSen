//! What a ROM's save files are called, and where: the C# `SaveLibrary`. See EmuSen_Galaxia.md §5.

use std::borrow::Cow;
use std::path::{Path, PathBuf};
use crate::tree::Tree;

pub const SRAM_EXTENSION: &str = ".srm";
pub const STATE_EXTENSION: &str = ".state";
/// The slot that keeps the plain `<rom>.state` name.
pub const DEFAULT_STATE_SLOT: i32 = 1;

/// The name a console started empty goes by, `SaveLibrary.ConsoleSessionName`.
pub fn console_session_name(console: &str) -> String {
    format!("{console} (no disc)")
}


fn stem(rom_path: &str) -> Cow<'_, str> {
    Path::new(rom_path).file_stem().unwrap_or_default().to_string_lossy()
}

/// The states' directory: the caller's, unless it is absent or blank. A parameter and never a settings read; see §5.1.
fn state_directory(tree: &Tree, directory_override: Option<&str>) -> PathBuf {
    match directory_override {
        Some(directory) if !directory.trim().is_empty() => PathBuf::from(directory),
        _ => tree.save_states(),
    }
}

/// `Saves/<console>/<stem>.srm`.
pub fn sram_path(tree: &Tree, rom_path: &str, console: &str) -> PathBuf {
    tree.saves().join(console).join(format!("{}{SRAM_EXTENSION}", stem(rom_path)))
}

/// `Saves/<stem>.srm`, where saves went before each console had a folder; read to copy them across, never written.
pub fn flat_sram_path(tree: &Tree, rom_path: &str) -> PathBuf {
    tree.saves().join(format!("{}{SRAM_EXTENSION}", stem(rom_path)))
}

/// `<stem>.state` for slot 1 and `<stem>.slotN.state` for the rest.
pub fn state_path(tree: &Tree, rom_path: &str, slot: i32, directory_override: Option<&str>) -> PathBuf {
    let suffix = if slot == DEFAULT_STATE_SLOT { String::new() } else { format!(".slot{slot}") };
    state_directory(tree, directory_override).join(format!("{}{suffix}{STATE_EXTENSION}", stem(rom_path)))
}

/// `<stem>.resume.state`, a name no slot can take.
pub fn resume_state_path(tree: &Tree, rom_path: &str, directory_override: Option<&str>) -> PathBuf {
    state_directory(tree, directory_override).join(format!("{}.resume{STATE_EXTENSION}", stem(rom_path)))
}

/// The name a package's saves and states go by: its folder's, less the extension, since the folder is named after the
/// game's file when it is made and does not change when the game moves (EmuSen_GamePackages.md §3.2, §14).
pub fn package_stem(root: impl AsRef<Path>) -> String {
    root.as_ref().file_stem().unwrap_or_default().to_string_lossy().into_owned()
}

/// The name a package's files go by: its own, and for a game opened with its patch list applied, `.patched-<key>` after
/// it, `key` being `package::save_key`'s, so that a hack never reaches the original's saves (EmuSen_GamePackages.md §5.5).
pub fn package_name(root: impl AsRef<Path>, key: Option<&str>) -> String {
    match key.filter(|k| !k.is_empty()) {
        Some(key) => format!("{}.patched-{key}", package_stem(root)),
        None => package_stem(root),
    }
}

/// `<package>/saves/<name>.srm`: a packaged game's battery save, whatever its console or the file it opened from.
pub fn package_sram_path(root: impl AsRef<Path>, key: Option<&str>) -> PathBuf {
    let root = root.as_ref();
    root.join(crate::package::SAVES_FOLDER).join(format!("{}{SRAM_EXTENSION}", package_name(root, key)))
}

/// `<package>/states/<name>.state` for slot 1 and `.slotN.state` for the rest; the state folder setting gives way to it.
pub fn package_state_path(root: impl AsRef<Path>, slot: i32, key: Option<&str>) -> PathBuf {
    let root = root.as_ref();
    let suffix = if slot == DEFAULT_STATE_SLOT { String::new() } else { format!(".slot{slot}") };
    root.join(crate::package::STATES_FOLDER).join(format!("{}{suffix}{STATE_EXTENSION}", package_name(root, key)))
}

/// `<package>/states/<name>.resume.state`.
pub fn package_resume_state_path(root: impl AsRef<Path>, key: Option<&str>) -> PathBuf {
    let root = root.as_ref();
    root.join(crate::package::STATES_FOLDER).join(format!("{}.resume{STATE_EXTENSION}", package_name(root, key)))
}

/// The picture beside a state: the same name with `.png`.
pub fn picture_path(state_path: impl AsRef<Path>) -> PathBuf {
    state_path.as_ref().with_extension("png")
}

/// Where a package keeps a state's record: beside the state, the same name with `.json`, for a state in a package's
/// `states` folder; none for any other state, whose record is states.db's (EmuSen_GamePackages.md §18.1).
pub fn package_record_path(state_path: impl AsRef<Path>) -> Option<PathBuf> {
    let full = crate::path_key::key(state_path);
    let folder = full.parent()?;
    if folder.file_name()? != crate::package::STATES_FOLDER {
        return None;
    }
    let root = folder.parent()?;
    crate::files::file_exists(root.join(crate::package::FILE_NAME)).then(|| full.with_extension("json"))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn tree() -> Tree {
        Tree { root: "/r".into(), ..Tree::default() }
    }

    #[test]
    fn a_save_is_named_for_the_rom_and_filed_by_console() {
        assert_eq!(sram_path(&tree(), "/roms/SNES/USA/Zelda (U).sfc", "SNES"), Path::new("/r/home/Saves/SNES/Zelda (U).srm"));
        assert_eq!(flat_sram_path(&tree(), "/roms/SNES/USA/Zelda (U).sfc"), Path::new("/r/home/Saves/Zelda (U).srm"));
    }

    #[test]
    fn slot_one_is_the_unsuffixed_name() {
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", 1, None), Path::new("/r/home/Saves/Save States/a.b.state"));
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", 2, None), Path::new("/r/home/Saves/Save States/a.b.slot2.state"));
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", 0, None), Path::new("/r/home/Saves/Save States/a.b.slot0.state"));
        assert_eq!(state_path(&tree(), "/roms/a.b.nes", -3, None), Path::new("/r/home/Saves/Save States/a.b.slot-3.state"));
    }

    #[test]
    fn a_blank_override_falls_back_and_the_roms_folder_never_leaks() {
        assert_eq!(state_path(&tree(), "/roms/a.nes", 1, Some("  ")), Path::new("/r/home/Saves/Save States/a.state"));
        assert_eq!(state_path(&tree(), "/roms/a.nes", 1, Some("")), Path::new("/r/home/Saves/Save States/a.state"));
        assert_eq!(state_path(&tree(), "/roms/a.nes", 1, Some("/elsewhere")), Path::new("/elsewhere/a.state"));
        assert_eq!(resume_state_path(&tree(), "/roms/a.nes", None), Path::new("/r/home/Saves/Save States/a.resume.state"));
        assert_eq!(resume_state_path(&tree(), "/roms/a.nes", Some("/elsewhere")), Path::new("/elsewhere/a.resume.state"));
    }

    #[test]
    fn a_packaged_game_keeps_its_saves_and_states_in_its_package_by_its_own_name() {
        assert_eq!(package_sram_path("/p/SNES/Zelda (U).sfc", None), Path::new("/p/SNES/Zelda (U).sfc/saves/Zelda (U).srm"));
        assert_eq!(package_state_path("/p/a.b.nes/", 1, None), Path::new("/p/a.b.nes/states/a.b.state"));
        assert_eq!(package_state_path("/p/a.b.nes", 3, Some("")), Path::new("/p/a.b.nes/states/a.b.slot3.state"));
        assert_eq!(package_resume_state_path("/p/g.nes", None), Path::new("/p/g.nes/states/g.resume.state"));
    }

    #[test]
    fn a_patched_game_keeps_its_saves_and_states_apart_by_its_lists_key() {
        assert_eq!(package_sram_path("/p/SNES/Zelda (U).sfc", Some("0a1b2c3d")), Path::new("/p/SNES/Zelda (U).sfc/saves/Zelda (U).patched-0a1b2c3d.srm"));
        assert_eq!(package_state_path("/p/a.b.nes", 1, Some("0a1b2c3d")), Path::new("/p/a.b.nes/states/a.b.patched-0a1b2c3d.state"));
        assert_eq!(package_state_path("/p/a.b.nes", 3, Some("0a1b2c3d")), Path::new("/p/a.b.nes/states/a.b.patched-0a1b2c3d.slot3.state"));
        assert_eq!(package_resume_state_path("/p/g.nes", Some("0a1b2c3d")), Path::new("/p/g.nes/states/g.patched-0a1b2c3d.resume.state"));
    }

    #[test]
    fn a_states_picture_takes_its_name() {
        assert_eq!(picture_path("/s/a.slot2.state"), Path::new("/s/a.slot2.png"));
        assert_eq!(picture_path("/s/a.resume.state"), Path::new("/s/a.resume.png"));
    }
}
