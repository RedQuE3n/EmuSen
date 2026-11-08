//! The file names of the databases under Galaxia's roots, each spelled here once and read by every component that
//! opens one, since Galaxia owns every file name (EmuSen_Surfaces.md §6.2).

/// The library's games and their records, in the library folder.
pub const GAMES_DB: &str = "games.db";
/// What the C# front end recorded of files it wrote or fetched; the runtime copies its state rows once (D8).
pub const RECORDS_DB: &str = "records.db";
/// The runtime's record of each save state it wrote.
pub const STATES_DB: &str = "states.db";
/// The scraper's media and its records.
pub const MEDIA_DB: &str = "media.db";
/// The installed big-picture themes.
pub const THEMES_DB: &str = "themes.db";

/// The files Galaxia names a front end by number (`EMUSEN_GALAXIA_FILE_*`): the settings file and the stores.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Settings,
    Games,
    States,
    Records,
    Media,
    Themes,
}

impl File {
    pub const ALL: [File; 6] = [File::Settings, File::Games, File::States, File::Records, File::Media, File::Themes];

    pub fn from_u32(which: u32) -> Option<File> {
        File::ALL.get(which as usize).copied()
    }

    /// The file's path in `tree`: the settings in the config directory, games.db, states.db and records.db in the library
    /// folder, media.db in the media folder and themes.db in the themes folder.
    pub fn path(self, tree: &crate::tree::Tree) -> std::path::PathBuf {
        use crate::tree::Directory;
        let (folder, name) = match self {
            File::Settings => return tree.config_path(None, crate::models::APP_SETTINGS_FILE),
            File::Games => (Directory::Library, GAMES_DB),
            File::States => (Directory::Library, STATES_DB),
            File::Records => (Directory::Library, RECORDS_DB),
            File::Media => (Directory::Media, MEDIA_DB),
            File::Themes => (Directory::Themes, THEMES_DB),
        };
        tree.directory(folder).unwrap_or_default().join(name)
    }
}
