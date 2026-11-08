//! A list of games as plain data between LibraryModel and GameModel (EmuSen_Qt.md §7.1): a JSON array, one object a
//! game, written from the library view's rows and read back the same way.

use crate::library::GameRow;
use serde_json::{json, Value};

/// The games as the JSON array GameModel reads.
pub fn to_json(games: &[GameRow]) -> String {
    let rows: Vec<Value> = games
        .iter()
        .map(|g| json!({ "path": g.path, "system": g.system, "title": g.title, "hue": g.hue, "favourite": g.favourite, "cover": g.cover, "hero": g.hero, "console": g.console, "facts": g.facts, "played": g.played_at }))
        .collect();
    Value::Array(rows).to_string()
}

/// The games less the one at `path`, as Home's shelf leaves out the suspended game; all of them for an empty path.
pub fn without(games: Vec<GameRow>, path: &str) -> Vec<GameRow> {
    games.into_iter().filter(|g| path.is_empty() || g.path != path).collect()
}

/// The games read from `to_json`'s text; none from text that is not such an array.
pub fn from_json(text: &str) -> Vec<GameRow> {
    let Ok(Value::Array(rows)) = serde_json::from_str::<Value>(text) else { return Vec::new() };
    let words = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).unwrap_or_default().to_owned();
    rows.iter()
        .map(|v| GameRow {
            path: words(v, "path"),
            system: words(v, "system"),
            title: words(v, "title"),
            hue: v.get("hue").and_then(Value::as_f64).unwrap_or_default(),
            favourite: v.get("favourite").and_then(Value::as_bool).unwrap_or_default(),
            cover: words(v, "cover"),
            hero: words(v, "hero"),
            console: words(v, "console"),
            facts: words(v, "facts"),
            played_at: v.get("played").and_then(Value::as_i64).unwrap_or_default(),
        })
        .collect()
}
