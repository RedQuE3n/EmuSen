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
    let at = game_settings_path(tree, system, game);
    let Ok(Some(bytes)) = emusen_galaxia::atomic::try_read(&at) else { return Vec::new() };
    let text = emusen_dotnet::file::decode(&bytes);
    let none = |why: String| {
        crate::error_log::warning("settings", &format!("{at} is not a game's settings, and is read as none: {why}"));
        Vec::new()
    };
    match emusen_dotnet::json::parse(&text) {
        Ok(emusen_dotnet::json::Value::Object(members)) => {
            let mut out: Vec<(String, String)> = Vec::new();
            for (k, v) in members {
                let Some(v) = v.as_str() else { return none(format!("the value of \"{}\" is not a string.", k.text)) };
                match out.iter_mut().find(|(key, _)| *key == k.text) {
                    Some(slot) => slot.1 = v.to_string(),
                    None => out.push((k.text, v.to_string())),
                }
            }
            out
        }
        Ok(emusen_dotnet::json::Value::Null) => Vec::new(),
        Ok(_) => none("the document is not an object.".to_string()),
        Err(e) => none(e.to_string()),
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

#[cfg(test)]
mod tests {
    use emusen_galaxia::tree::{Override, tree, tree_mut};

    #[test]
    fn a_game_file_that_is_not_a_map_of_strings_is_read_as_none_and_said_in_the_error_log() {
        let root = std::env::temp_dir().join(format!("crystal-settings-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let (config, logs) = (root.join("Config"), root.join("Logs"));
        std::fs::create_dir_all(config.join("games").join("nes")).unwrap();
        std::fs::write(config.join("appsettings.json"), format!("{{\"LogDirectory\":{:?}}}", logs.to_string_lossy())).unwrap();
        std::fs::write(config.join("games").join("nes").join("Bad.json"), "{\"Engine\":5}").unwrap();
        std::fs::write(config.join("games").join("nes").join("Good.json"), "{\"Engine\":\"MoonRT (Rust)\"}").unwrap();
        {
            let mut t = tree_mut();
            t.set_override(Override::Config, Some(config.to_string_lossy().into_owned()));
            t.set_override(Override::Data, Some(root.join("Home").to_string_lossy().into_owned()));
        }
        let t = tree().clone();
        assert!(super::game_values(&t, Some("nes"), "/roms/Bad.nes").is_empty());
        assert_eq!(super::game_values(&t, Some("nes"), "/roms/Good.nes"), vec![("Engine".to_string(), "MoonRT (Rust)".to_string())]);
        let written: String = std::fs::read_dir(&logs).unwrap().flatten().map(|e| std::fs::read_to_string(e.path()).unwrap()).collect();
        assert!(written.contains("WARN [settings]") && written.contains("Bad.json is not a game's settings, and is read as none: the value of \"Engine\" is not a string."), "{written}");
        assert!(!written.contains("Good.json"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
