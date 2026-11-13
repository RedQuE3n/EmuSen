//! The player's edits a front end reads or writes by name: ES-DE's gamelist fields, which the scraper's metadata.rs
//! lists whole (`FIELDS`), and the value a flag holds when it is set. One owner here for the names the program uses.

/// The game left out of every library view until Settings lists hidden games again.
pub const HIDDEN: &str = "hidden";
/// The name the player gave the game.
pub const NAME: &str = "name";
/// The engine the game runs on, where the player chose one.
pub const ALT_EMULATOR: &str = "altemulator";
/// A flag that is set, as ES-DE's gamelists and games.db store it.
pub const YES: &str = "1";

/// Whether the flag `field` is set in a game's edits, `{field: value}`.
pub fn flag(edits: &serde_json::Value, field: &str) -> bool {
    edits.get(field).and_then(serde_json::Value::as_str) == Some(YES)
}
