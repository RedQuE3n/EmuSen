//! Where a game sits below its console's folder, as ES-DE's `ROMs/<system>/<folder>/<file>`: the folders between the
//! console folder (the ROM folder's first level) and the file. The scraper stores a game's media under it, so it is the
//! store's layout rule; the C# `GameFolders.Of` is its twin until state 3. See EmuSen_Settings_Reference.md §4.67 and
//! EmuSen_Backend.md §30.1.

use crate::dotnet_path::{self, Style};

/// The game's folder, '/'-separated; "" at the console folder's top, outside the ROM folder, or with no ROM folder.
pub fn of(rom_directory: Option<&str>, path: &str) -> String {
    of_from(Style::HOST, rom_directory, path, &std::env::current_dir().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default())
}

/// As [`of`], with the style and the working directory given.
pub fn of_from(style: Style, rom_directory: Option<&str>, path: &str, working_directory: &str) -> String {
    let Some(roms) = rom_directory.filter(|r| !emusen_dotnet::text::is_blank(Some(r))) else { return String::new() };
    let Some(directory) = dotnet_path::directory_name(style, path).filter(|d| !d.is_empty()) else { return String::new() };
    let root = dotnet_path::full_path_from(style, roms, working_directory);
    let dir = dotnet_path::full_path_from(style, &directory, working_directory);
    let separator = style.separator();
    let root = root.trim_end_matches(separator);
    let rest = if dir.trim_end_matches(separator) == root {
        return String::new();
    } else if let Some(rest) = dir.strip_prefix(root).and_then(|r| r.strip_prefix(separator)) {
        rest
    } else {
        return String::new();
    };
    let parts: Vec<&str> = rest.split(separator).filter(|p| !p.is_empty()).collect();
    parts.iter().skip(1).copied().collect::<Vec<_>>().join("/")
}

#[cfg(test)]
mod tests {
    use super::of_from;
    use crate::dotnet_path::Style::Unix;

    fn of(roms: Option<&str>, path: &str) -> String {
        of_from(Unix, roms, path, "/work")
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
