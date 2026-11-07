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
