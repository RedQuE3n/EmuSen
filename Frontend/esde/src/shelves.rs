//! The shelves' text (EmuSen_Qt.md §4.9): one shelf per list with its facts and its games, the one schema qt-models
//! writes and esde-qt's view reads. Borrowed where written and owned where read; a key either side does not name is an
//! error, never a field read as absent.

use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// A text field of the shelves: borrowed from the library where written, owned where read.
pub type Text<'a> = Cow<'a, str>;

/// One shelf as a list shows it, its games written as `Entry` and read as `GameRow`; the keys in the order a JSON
/// object's sorted map gave them.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ShelfText<'a, G = GameRow<'a>> {
    pub collection_id: Option<i64>,
    /// None where a hand-made text leaves the view to count the games itself.
    pub counts: Option<CountsText>,
    pub favourites_on_top: bool,
    pub folder: Text<'a>,
    pub full_name: Text<'a>,
    pub games: Vec<G>,
    pub key: Text<'a>,
    /// "regular", "auto" or "custom".
    pub kind: Text<'a>,
    pub letters: Cow<'a, [(String, usize)]>,
    pub name: Text<'a>,
    pub sort: Text<'a>,
    pub stars: bool,
    pub system: Text<'a>,
    pub theme: Text<'a>,
}

/// A list's counts, as the help and the game count show them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CountsText {
    pub favorites: usize,
    pub filtered: bool,
    pub games: usize,
    pub in_folder: bool,
    pub shown: usize,
}

/// A game's row, or a folder entry's. A collection's member is written by its file, system and sort name alone
/// (`Entry::Member`), its row the one its console's shelf carries, so its `name` is empty.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GameRow<'a> {
    pub alt_emulator: bool,
    pub broken: bool,
    pub completed: bool,
    pub controller: Option<Text<'a>>,
    pub custom_sort_name: Option<Text<'a>>,
    /// A console's row leaves it out, asked on demand; a folder entry carries its own where it has one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<Text<'a>>,
    pub developer: Option<Text<'a>>,
    pub emulator: Option<Text<'a>>,
    pub esde_full_name: Text<'a>,
    pub esde_system: Text<'a>,
    pub face: Option<Text<'a>>,
    pub favourite: bool,
    pub file: Text<'a>,
    pub folder: bool,
    pub folder_link: Option<Text<'a>>,
    pub folder_path: Text<'a>,
    pub genre: Option<Text<'a>>,
    pub hidden: bool,
    pub hide_metadata: bool,
    /// Written only where true: the library marks no game yet, and a hand-made text may.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub in_collection: bool,
    pub is_collection: bool,
    pub kid_game: bool,
    /// Seconds since 1970; 0 for never.
    pub last_played: i64,
    pub manual: bool,
    pub name: Text<'a>,
    pub not_counted: bool,
    /// As games.db keeps it, signed.
    pub play_count: i64,
    /// Seconds, as games.db keeps them, signed.
    pub play_time: i64,
    pub players: Option<Text<'a>>,
    pub publisher: Option<Text<'a>>,
    /// 0 to 1.
    pub rating: Option<f64>,
    /// The metadata's `yyyy-MM-dd`.
    pub release_date: Option<Text<'a>>,
    pub sort_name: Option<Text<'a>>,
}

/// An entry as written: a whole row, or a collection's member by its file, system and sort name alone, which reads
/// back as a row with no name.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Entry<'a> {
    Row(GameRow<'a>),
    Member { esde_system: Text<'a>, file: Text<'a>, sort_name: Option<Text<'a>> },
}

impl GameRow<'_> {
    /// Whether this entry is a whole row, not a collection's member written by its file alone.
    pub fn is_whole(&self) -> bool {
        !self.name.is_empty()
    }
}
