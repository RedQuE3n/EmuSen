//! Settings in layers, game over console over the schema's default: a console's in `consoles.json`, a game's in a file
//! of its own, `/etc/EmuSen/games/<system>/<file stem>.json` (EmuSen_Backend.md §3.6, §14.7, SR10, SR12).

use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::{Directory, Tree};

/// A retired engine's name a player's settings may still hold, and the engine that runs instead (`RetiredEngines`).
const RETIRED: [(&str, &str); 4] = [("Mars (C#)", "MarsRT (Rust)"), ("Mercury (C#)", "MercuryRT (Rust)"), ("Moon (C#)", "MoonRT (Rust)"), ("Venus (C#)", "VenusRT (Rust)")];

/// The retired names already logged, once a process.
static RETIRED_LOGGED: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());

/// A stored engine's name as the engine that runs: a retired one's replacement, logged once a process, any other as
/// it is (`Current`).
pub fn current(stored: Option<String>) -> Option<String> {
    let s = stored?;
    let Some((old, new)) = RETIRED.iter().find(|(old, _)| *old == s) else { return Some(s) };
    let first = {
        let mut logged = RETIRED_LOGGED.lock().unwrap_or_else(|p| p.into_inner());
        let first = !logged.contains(old);
        if first {
            logged.push(old);
        }
        first
    };
    if first {
        crate::error_log::warning("engine", &format!("{old} was retired; {new} runs its games instead."));
    }
    Some(new.to_string())
}

/// Where a game's own settings are kept.
pub fn game_settings_path(tree: &Tree, system: &str, game: &str) -> String {
    let config = tree.directory(Directory::Config).unwrap_or_default();
    let stem = path::file_name_without_extension(Style::HOST, game);
    path::combine_all(Style::HOST, &[&config, "games", system, &format!("{stem}.json")])
}

/// A game's own values, in the file's order; none where it has no file or one that is not a map of strings.
pub fn game_values(tree: &Tree, system: Option<&str>, game: &str) -> Vec<(String, String)> {
    let Some(system) = system else { return Vec::new() };
    let Ok(Some(bytes)) = emusen_galaxia::atomic::try_read(&game_settings_path(tree, system, game)) else { return Vec::new() };
    let text = emusen_dotnet::file::decode(&bytes);
    match emusen_dotnet::json::parse(&text) {
        Ok(emusen_dotnet::json::Value::Object(members)) => {
            let mut out: Vec<(String, String)> = Vec::new();
            for (k, v) in members {
                let Some(v) = v.as_str() else { return Vec::new() };
                match out.iter_mut().find(|(key, _)| *key == k.text) {
                    Some(slot) => slot.1 = v.to_string(),
                    None => out.push((k.text, v.to_string())),
                }
            }
            out
        }
        _ => Vec::new(),
    }
}

/// The engine a game opens on, `ManagedRuntime.EngineFor`: the game's own choice, else its console's.
pub fn engine_for(tree: &Tree, game: &str, row_console: Option<&str>, system: Option<&str>) -> Option<String> {
    let own = game_values(tree, system, game).into_iter().find(|(k, _)| k == crate::catalog::ENGINE_KEY).map(|(_, v)| v);
    if let Some(own) = own {
        return if row_console.is_none() { Some(own) } else { current(Some(own)) };
    }
    let console = row_console?;
    current(emusen_galaxia::consoles::load(tree).value(console, crate::catalog::ENGINE_KEY).map(str::to_string))
}
