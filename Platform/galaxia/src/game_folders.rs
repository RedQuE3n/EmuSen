//! Where a game sits below its console's folder, as ES-DE's `ROMs/<system>/<folder>/<file>`: the folders between the
//! console folder (the ROM folder's first level) and the file. The scraper stores a game's media under it, so it is the
//! store's layout rule; the C# `GameFolders.Of` is its twin until state 3. See EmuSen_Settings_Reference.md §4.67 and
//! EmuSen_Backend.md §30.1.

use crate::path_key;
use std::path::Path;

/// The game's folder, '/'-separated; "" at the console folder's top, outside the ROM folder, or with no ROM folder.
pub fn of(rom_directory: Option<&str>, path: &str) -> String {
    of_from(rom_directory, path, &std::env::current_dir().unwrap_or_default())
}

/// As [`of`], with the working directory given.
pub fn of_from(rom_directory: Option<&str>, path: &str, working_directory: &Path) -> String {
    let Some(roms) = rom_directory.filter(|r| !r.trim().is_empty()) else { return String::new() };
    let Some(directory) = Path::new(path).parent().filter(|d| !d.as_os_str().is_empty()) else { return String::new() };
    let root = path_key::key_from(Path::new(roms), working_directory);
    let dir = path_key::key_from(directory, working_directory);
    let Ok(rest) = dir.strip_prefix(&root) else { return String::new() };
    rest.components().skip(1).map(|part| part.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}

#[cfg(all(test, unix))]
mod tests {
    use super::of_from;

    fn of(roms: Option<&str>, path: &str) -> String {
        of_from(roms, path, std::path::Path::new("/work"))
    }
    #[test]
    fn the_folders_below_the_console_folder_and_nothing_outside() {
        assert_eq!(of(Some("/r"), "/r/snes/RPG/Square/ff.sfc"), "RPG/Square");
        assert_eq!(of(Some("/r/"), "/r/snes/RPG/ff.sfc"), "RPG");
        assert_eq!(of(Some("/r"), "/r/snes/ff.sfc"), "");
        assert_eq!(of(Some("/r"), "/r/ff.sfc"), "");
        assert_eq!(of(Some("/r"), "/other/snes/RPG/ff.sfc"), "");
        assert_eq!(of(Some("/r"), "/rx/snes/RPG/ff.sfc"), "");
        assert_eq!(of(Some("/r"), "/r/snes/RPG/../Act/x.sfc"), "Act");
        assert_eq!(of(Some("roms"), "/work/roms/nes/A/x.nes"), "A");
        assert_eq!(of(Some("/"), "/snes/A/x.sfc"), "A");
        assert_eq!(of(Some(" "), "/r/snes/A/x.sfc"), "");
        assert_eq!(of(None, "/r/snes/A/x.sfc"), "");
        assert_eq!(of(Some("/r"), "x.sfc"), "");
        assert_eq!(of(Some("/R"), "/r/snes/A/x.sfc"), "");
    }
}
