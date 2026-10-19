//! The settings a client reads and changes, the C# `ManagedRuntime`'s settings: a console's rows (the engine's choice,
//! the engine's own settings, the save media's), each value in layers, game over console over the row's default, a
//! change written to `consoles.json` or the game's own file and applied to the open sessions it reaches; and SR10's
//! one copy of games.db's engine choices (EmuSen_Backend.md §3.6, §14.12, §29.19).

use emusen_core_host::{Discovered, Discovery, Trust};
use emusen_dotnet::text::equals_ignore_case;
use emusen_galaxia::dotnet_path::{self as path, Style};
use emusen_galaxia::tree::{Directory, Tree};
use emusen_session_service::abi::status as svc;

use crate::abi;
use crate::catalog::{self, Catalog, ENGINE_KEY};
use crate::json::Json;
use crate::machine_info::SettingDesc;
use crate::media::SaveMedium;
use crate::runtime::{Refused, Runtime};
use crate::session::Session;

/// How a row's note is found.
#[derive(Clone)]
enum Note {
    /// No note: the row writes none.
    None,
    /// The running engine's note for the key, or null.
    Engine(Option<String>),
    /// A save medium's slot after the first, which says when an earlier slot names the same file.
    SameName { medium: Box<SaveMedium>, slot: usize },
}

/// One row, `CoreSetting`.
#[derive(Clone)]
pub struct Row {
    pub key: String,
    label: String,
    hint: String,
    kind: &'static str,
    pub default: String,
    min: i64,
    max: i64,
    pub choices: Vec<String>,
    choice_labels: Vec<String>,
    group: Option<String>,
    note: Note,
    shown_with: Option<(String, String)>,
}

/// An engine's own setting as a row, `AsCoreSetting`: none for a kind no row shows.
fn row_of(s: &SettingDesc, note: Note) -> Option<Row> {
    let kind = match s.kind.as_str() {
        "switch" => "switch",
        "count" => "count",
        "choice" => "choice",
        _ => return None,
    };
    let (min, max) = if kind == "count" { (s.min, s.max) } else { (0, 0) };
    let choices = if kind == "choice" { s.choices.clone() } else { Vec::new() };
    let labels = if kind == "choice" && s.choice_labels.len() == s.choices.len() { s.choice_labels.clone() } else { Vec::new() };
    Some(Row {
        key: s.key.clone(),
        label: s.label.clone(),
        hint: s.cost.as_ref().map_or_else(|| s.help.clone(), |c| format!("{} {c}", s.help)),
        kind,
        default: s.default.clone(),
        min,
        max,
        choices,
        choice_labels: labels,
        group: (s.effect == "enhancement").then(|| "Enhancements".to_string()),
        note,
        shown_with: None,
    })
}

/// A library's settings a window lists, `CoreSidecarSettings`.
fn sidecar_rows(core: &Discovered, trust: &Trust) -> Vec<Row> {
    let Ok(lib) = core.open(trust) else { return Vec::new() };
    let Ok(text) = lib.settings_text() else { return Vec::new() };
    crate::machine_info::settings(&text).iter().filter(|s| !s.hidden).filter_map(|s| row_of(s, Note::None)).collect()
}

/// The discovered engines that run a console's systems, `EnginesOfConsole`.
fn engines_of_console<'d>(discovery: &'d Discovery, console: &str) -> Vec<&'d Discovered> {
    let mut ids: Vec<String> = catalog::system_ids_for(console).into_iter().map(str::to_string).collect();
    for p in emusen_systems::table().unwrap_or(&[]) {
        if equals_ignore_case(&p.console, console) {
            ids.push(p.id.clone());
        }
    }
    discovery.found().iter().filter(|c| c.sidecar.info.systems.iter().any(|s| ids.contains(&s.id) || equals_ignore_case(&s.id, console))).collect()
}

/// A save medium's rows: its mode, and a row for each slot, `MemoryCards.RowsFor`.
fn medium_rows(tree: &Tree, m: &SaveMedium) -> Vec<Row> {
    let noun = emusen_savemedia::in_text(&m.noun);
    let mut names: Vec<String> = (0..crate::media::NAMES_OFFERED).map(|i| m.default_name(i)).collect();
    for n in crate::media::medium_names(tree, &m.console, m.is_console(), &m.extension) {
        if crate::media::is_name(&n) && !names.iter().any(|x| equals_ignore_case(x, &n)) {
            names.push(n);
        }
    }
    let mut rows = vec![Row {
        key: m.mode_key(),
        label: m.title.clone(),
        hint: format!(
            "Per game: each game keeps its own, as before. Shared: one {noun} in each slot, used by every game, as a console with it always there. Named: a drawer of them, each a file of the save folder, and you choose which is in each slot. A save state never rolls a shared or named {noun} back. Changing this moves, copies and deletes nothing: each game's own are used again when Per game is chosen again. Takes effect when a game is next loaded."
        ),
        kind: "choice",
        default: m.default_mode().to_string(),
        min: 0,
        max: 0,
        choices: vec![crate::media::PER_GAME.to_string(), crate::media::SHARED.to_string(), crate::media::NAMED.to_string()],
        choice_labels: vec!["Per game".to_string(), "Shared".to_string(), "Named".to_string()],
        group: None,
        note: Note::None,
        shown_with: None,
    }];
    for slot in 0..m.slots.len() {
        rows.push(Row {
            key: m.slot_key(slot),
            label: format!("{}: {}", m.title, m.slot_label(slot)),
            hint: format!("Which {noun} is here under Named; a name with no file yet is a new, blank one, which the game formats. Takes effect when a game is next loaded."),
            kind: "choice",
            default: m.default_name(slot),
            min: 0,
            max: 0,
            choices: names.clone(),
            choice_labels: Vec::new(),
            group: None,
            note: if slot > 0 { Note::SameName { medium: Box::new(m.clone()), slot } } else { Note::None },
            shown_with: Some((m.mode_key(), crate::media::NAMED.to_string())),
        });
    }
    rows
}

/// A console's rows, the engine's choice first, `ManagedRuntime.Schema`.
pub fn schema(rt: &Runtime, catalog: &Catalog, discovery: &Discovery, tree: &Tree, console: &str, engine: Option<&str>, session: Option<&Session>) -> Vec<Row> {
    let mut rows = Vec::new();
    let engines = engines_of_console(discovery, console);
    if let Some(first) = engines.first() {
        rows.push(Row {
            key: ENGINE_KEY.to_string(),
            label: "Engine".to_string(),
            hint: "Which implementation runs the console. Takes effect when a game is next loaded.".to_string(),
            kind: "choice",
            default: first.engine_name().to_string(),
            min: 0,
            max: 0,
            choices: engines.iter().map(|e| e.engine_name().to_string()).collect(),
            choice_labels: Vec::new(),
            group: None,
            note: Note::None,
            shown_with: None,
        });
    }
    let media: Vec<Row> = crate::media::media_for(catalog, console).iter().flat_map(|m| medium_rows(tree, m)).collect();
    if let Some(s) = session
        && engine.is_none()
    {
        let e = s.engine.lock().unwrap_or_else(|p| p.into_inner());
        for d in e.settings.iter().filter(|d| !d.hidden) {
            let note = e.notes.iter().find(|(k, _)| *k == d.key).map(|(_, v)| v.clone());
            rows.extend(row_of(d, Note::Engine(note)));
        }
        rows.extend(media);
        return rows;
    }
    let named = engine.and_then(|n| engines.iter().find(|e| e.engine_name() == n)).copied();
    let mut own = match named {
        Some(core) => sidecar_rows(core, rt.trust()),
        None => {
            let core = catalog::discovered_for(discovery, console).or_else(|| catalog::sole_engine(console).and_then(|e| discovery.by_engine_name(e)));
            core.map(|c| sidecar_rows(c, rt.trust())).unwrap_or_default()
        }
    };
    if own.is_empty()
        && named.is_none()
        && let Some(sole) = engines.first()
    {
        own = sidecar_rows(sole, rt.trust());
    }
    rows.extend(own);
    rows.extend(media);
    rows
}

/// `SchemaJson`: each row with its note as the console's stored values stand.
pub fn schema_json(rows: &[Row], tree: &Tree, console: Option<&str>) -> String {
    let stored = emusen_galaxia::consoles::load(tree);
    let value = |key: &str| -> String {
        console.and_then(|c| stored.value(c, key)).map(str::to_string).or_else(|| rows.iter().find(|r| r.key == key).map(|r| r.default.clone())).unwrap_or_default()
    };
    let mut j = Json::new();
    j.begin_array();
    for r in rows {
        j.begin_object().str("key", &r.key).str("label", &r.label).str("hint", &r.hint).str("kind", r.kind).str("default", &r.default).int("min", r.min).int("max", r.max);
        j.key_array("choices");
        for c in &r.choices {
            j.str_value(c);
        }
        j.end_array().key_array("choice_labels");
        for c in &r.choice_labels {
            j.str_value(c);
        }
        j.end_array().str_or_null("group", r.group.as_deref());
        match &r.note {
            Note::None => {}
            Note::Engine(note) => {
                j.str_or_null("note", note.as_deref());
            }
            Note::SameName { medium, slot } => {
                let same = value(&medium.mode_key()) == crate::media::NAMED && (0..*slot).any(|s| equals_ignore_case(&value(&medium.slot_key(s)), &value(&medium.slot_key(*slot))));
                let words = same.then(|| format!("The same {} cannot be in two slots: this slot keeps the game's own while they match.", emusen_savemedia::in_text(&medium.noun)));
                j.str_or_null("note", words.as_deref());
            }
        }
        match &r.shown_with {
            Some((k, v)) => {
                j.key_object("shown_with").str("key", k).str("value", v).end_object();
            }
            None => {
                j.null("shown_with");
            }
        }
        j.end_object();
    }
    j.end_array();
    j.finish()
}

/// Why a value is not one the row takes, or none.
fn refusal(row: &Row, value: &str) -> Option<String> {
    match row.kind {
        "switch" if !matches!(value.trim().to_ascii_lowercase().as_str(), "true" | "false") => Some(format!("{} is a switch, true or false, not \"{value}\"", row.key)),
        "count" if emusen_dotnet::number::parse_i32(value).is_none_or(|n| (n as i64) < row.min || (n as i64) > row.max) => {
            Some(format!("{} is a count from {} to {}, not \"{value}\"", row.key, row.min, row.max))
        }
        "choice" if !row.choices.is_empty() && !row.choices.iter().any(|c| c == value) => Some(format!("{} is one of {}, not \"{value}\"", row.key, row.choices.join(", "))),
        _ => None,
    }
}

/// A value as the layers give it, the game's own over the console's; none where neither holds one, `Layered`.
fn layered(tree: &Tree, console: &str, system: Option<&str>, game: Option<&str>, key: &str) -> Option<String> {
    if let Some(game) = game
        && let Some((_, own)) = crate::settings::game_values(tree, system, game).into_iter().find(|(k, _)| k == key)
    {
        return if key == ENGINE_KEY { crate::settings::current(Some(own)) } else { Some(own) };
    }
    let stored = emusen_galaxia::consoles::load(tree).value(console, key).map(str::to_string);
    if key == ENGINE_KEY { crate::settings::current(stored) } else { stored }
}

/// A session's own system, its machine's.
fn system_of(s: &Session) -> Option<String> {
    let e = s.engine.lock().unwrap_or_else(|p| p.into_inner());
    e.machine.as_ref().map(|_| e.mi.system.clone())
}

/// A session's stored value of a key, game over console, `GetSettingOf`.
pub fn setting_of(tree: &Tree, s: &Session) -> impl Fn(&str) -> Option<String> {
    let (console, system, game) = (s.console.clone(), system_of(s), s.game_key());
    let tree = tree.clone();
    move |key| layered(&tree, &console, system.as_deref(), Some(&game), key)
}

/// The game's values written whole, indented as `JsonSerializer` writes a dictionary.
fn write_game_values(tree: &Tree, system: Option<&str>, game: &str, values: &[(String, String)]) -> Result<(), Refused> {
    let Some(system) = system else {
        return Err((svc::NOT_SUPPORTED, format!("{} runs as no system of the table, so it has no file of its own settings", path::file_name(Style::HOST, game))));
    };
    let at = crate::settings::game_settings_path(tree, system, game);
    if let Some(folder) = path::directory_name(Style::HOST, &at) {
        let _ = std::fs::create_dir_all(folder);
    }
    let mut text = String::new();
    if values.is_empty() {
        text.push_str("{}");
    } else {
        text.push('{');
        for (i, (k, v)) in values.iter().enumerate() {
            text.push_str(if i == 0 { "\n  " } else { ",\n  " });
            text.push_str(&emusen_dotnet::json::quoted(k));
            text.push_str(": ");
            text.push_str(&emusen_dotnet::json::quoted(v));
        }
        text.push_str("\n}");
    }
    if emusen_galaxia::atomic::write(&at, text.as_bytes()).is_err() {
        return Err((abi::status::IO, format!("{at} could not be written")));
    }
    Ok(())
}

impl Runtime {
    fn settings_target(&self, session: u32, scope: u32, console: Option<&str>) -> Result<(String, Option<std::sync::Arc<Session>>), Refused> {
        if scope != abi::SCOPE_CONSOLE && scope != abi::SCOPE_GAME {
            return Err((svc::NOT_SUPPORTED, format!("no settings scope {scope} in 1.{}", abi::MINOR)));
        }
        if session == 0 {
            if scope == abi::SCOPE_GAME {
                return Err((svc::BAD_ARGUMENT, "a game's settings are a session's: name the session".to_string()));
            }
            return console.map(|c| (c.to_string(), None)).ok_or_else(|| (svc::BAD_ARGUMENT, "with no session, the console is named".to_string()));
        }
        let s = self.sessions.get(session)?;
        if let Some(c) = console
            && !equals_ignore_case(c, &s.console)
        {
            return Err((svc::BAD_ARGUMENT, format!("session {} runs the {}, not the {c}", s.id, s.console)));
        }
        Ok((s.console.clone(), Some(s)))
    }

    /// `settings_schema`'s JSON.
    pub fn settings_schema_json(&self, session: u32, console: Option<&str>, engine: Option<&str>) -> Result<String, Refused> {
        let s = if session != 0 { Some(self.sessions.get(session)?) } else { None };
        let name = match (console, &s) {
            (Some(c), _) => c.to_string(),
            (None, Some(s)) => s.console.clone(),
            (None, None) => return Err((svc::BAD_ARGUMENT, "with no session, the console is named".to_string())),
        };
        let tree = emusen_galaxia::tree::tree();
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let rows = schema(self, &catalog, &discovery, &tree, &name, engine, s.as_deref());
        Ok(schema_json(&rows, &tree, Some(&name)))
    }

    pub fn settings_get(&self, session: u32, scope: u32, console: Option<&str>, key: &str) -> Result<String, Refused> {
        let (name, s) = self.settings_target(session, scope, console)?;
        let tree = emusen_galaxia::tree::tree();
        let stored = match (&s, scope) {
            (Some(s), abi::SCOPE_GAME) => setting_of(&tree, s)(key),
            _ => layered(&tree, &name, None, None, key),
        };
        if let Some(v) = stored {
            return Ok(v);
        }
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        schema(self, &catalog, &discovery, &tree, &name, None, s.as_deref())
            .into_iter()
            .find(|r| r.key == key)
            .map(|r| r.default)
            .ok_or_else(|| (svc::ABSENT, format!("{key} has no value and no default")))
    }

    pub fn settings_set(&self, session: u32, scope: u32, console: Option<&str>, key: &str, value: Option<&str>) -> Result<(), Refused> {
        let (name, s) = self.settings_target(session, scope, console)?;
        let tree = emusen_galaxia::tree::tree();
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let row = schema(self, &catalog, &discovery, &tree, &name, None, s.as_deref())
            .into_iter()
            .find(|r| r.key == key)
            .ok_or_else(|| (svc::NO_SUCH_ID, format!("the {name} has no setting \"{key}\"")))?;
        if let Some(v) = value
            && let Some(why) = refusal(&row, v)
        {
            return Err((svc::BAD_ARGUMENT, why));
        }
        if scope == abi::SCOPE_GAME {
            let s = s.as_ref().expect("a game's scope names a session");
            let system = system_of(s);
            let mut values = crate::settings::game_values(&tree, system.as_deref(), &s.game_key());
            match value {
                None => values.retain(|(k, _)| k != key),
                Some(v) => match values.iter_mut().find(|(k, _)| k == key) {
                    Some(slot) => slot.1 = v.to_string(),
                    None => values.push((key.to_string(), v.to_string())),
                },
            }
            write_game_values(&tree, system.as_deref(), &s.game_key(), &values)?;
        } else {
            let mut stored = emusen_galaxia::consoles::load(&tree);
            match value {
                None => stored.forget(&name, key),
                Some(v) => stored.set_value(&name, key, v),
            }
            stored.save(&tree);
        }
        drop(tree);
        for open in self.sessions.all() {
            if equals_ignore_case(&open.console, &name) && (scope == abi::SCOPE_CONSOLE || s.as_ref().is_some_and(|s| s.id == open.id)) {
                self.apply_to(&open, key);
            }
        }
        let scope_word = if scope == abi::SCOPE_GAME { "game" } else { "console" };
        self.events.raise(s.as_ref().map_or(0, |s| s.id), abi::EVENT_SETTINGS, "settings", |j| {
            j.str("scope", scope_word).str("console", &name).str("key", key).str_or_null("value", value);
        });
        Ok(())
    }

    /// The value the layers now give applied to an open session, `ApplySetting`.
    fn apply_to(&self, s: &Session, key: &str) {
        let value = setting_of(&emusen_galaxia::tree::tree(), s)(key);
        {
            let e = s.engine.lock().unwrap_or_else(|p| p.into_inner());
            let Some(setting) = e.listed().into_iter().find(|d| d.key == key) else { return };
            if e.value(key) == Some(value.as_deref().unwrap_or(&setting.default)) {
                return;
            }
        }
        let _ = self.with_ctx(|ctx| {
            s.change(|e, _| {
                e.apply(ctx, key, value.as_deref());
                Ok(())
            })
        });
    }

    /// The console a file opens as and its system, `ConsoleForRom` and `SystemOfFile`.
    fn console_and_system(&self, at: &str) -> (Option<String>, Option<String>) {
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let row = crate::route::route(&catalog, &discovery, at).0.map(|r| catalog.rows[r].console().to_string());
        (row, crate::route::system_of_file(&catalog, &discovery, at))
    }

    /// A game's own value with no session, `GameSetting`.
    pub fn game_setting_get(&self, at: &str, key: &str) -> Result<String, Refused> {
        let (console, system) = self.console_and_system(at);
        let tree = emusen_galaxia::tree::tree();
        let own = crate::settings::game_values(&tree, system.as_deref(), at).into_iter().find(|(k, _)| k == key).map(|(_, v)| v);
        let own = own.ok_or_else(|| (svc::ABSENT, "the game holds no value of its own for that key".to_string()))?;
        Ok(if key == ENGINE_KEY && console.is_some() { crate::settings::current(Some(own.clone())).unwrap_or(own) } else { own })
    }

    /// A game's own value set or forgotten with no session, `SetGameSetting`.
    pub fn game_setting_set(&self, at: &str, key: &str, value: Option<&str>) -> Result<(), Refused> {
        let (row_console, system) = self.console_and_system(at);
        let console = row_console.or_else(|| system.as_deref().map(emusen_dotnet::text::to_upper)).ok_or_else(|| {
            (svc::NOT_SUPPORTED, format!("{} runs as no system of the table, so it has no file of its own settings", path::file_name(Style::HOST, at)))
        })?;
        let tree = emusen_galaxia::tree::tree();
        let discovery = self.discovery();
        let catalog = Catalog::new(&discovery);
        let row = schema(self, &catalog, &discovery, &tree, &console, None, None)
            .into_iter()
            .find(|r| r.key == key)
            .ok_or_else(|| (svc::NO_SUCH_ID, format!("the {console} has no setting \"{key}\"")))?;
        if let Some(v) = value
            && let Some(why) = refusal(&row, v)
        {
            return Err((svc::BAD_ARGUMENT, why));
        }
        let mut values = crate::settings::game_values(&tree, system.as_deref(), at);
        match value {
            None => values.retain(|(k, _)| k != key),
            Some(v) => match values.iter_mut().find(|(k, _)| k == key) {
                Some(slot) => slot.1 = v.to_string(),
                None => values.push((key.to_string(), v.to_string())),
            },
        }
        write_game_values(&tree, system.as_deref(), at, &values)?;
        drop(tree);
        let full = path::full_path(at);
        for open in self.sessions.all() {
            if open.game_path.as_deref().is_some_and(|p| path::full_path(p) == full) {
                self.apply_to(&open, key);
            }
        }
        self.events.raise(0, abi::EVENT_SETTINGS, "settings", |j| {
            j.str("scope", "game").str("console", &console).str("key", key).str_or_null("value", value);
        });
        Ok(())
    }

    /// SR10's one migration: each game's "altemulator" in games.db copied to its own file as "Engine", never over a
    /// value already there, `CopyEngineChoices`; how many were copied.
    pub fn copy_engine_choices(&self) -> usize {
        let Some(sqlite) = self.sqlite() else { return 0 };
        let tree = emusen_galaxia::tree::tree();
        let library = tree.directory(Directory::Library).unwrap_or_default();
        let config = tree.directory(Directory::Config).unwrap_or_default();
        let games_db = path::combine(Style::HOST, &library, "games.db");
        let marker = path::combine_all(Style::HOST, &[&config, "games", ".engine-choices-copied"]);
        if !emusen_galaxia::dotnet_files::file_exists(&games_db) || emusen_galaxia::dotnet_files::file_exists(&marker) {
            return 0;
        }
        let mut choices: Vec<(String, String)> = Vec::new();
        let read = emusen_sqlite::Connection::open_read_only(sqlite, &games_db)
            .and_then(|db| db.query("SELECT path, value FROM game_edit WHERE field = 'altemulator'", &[], |row| choices.push((row.text(0), row.text(1)))).map(|_| ()));
        if let Err(e) = read {
            crate::error_log::warning("settings", &format!("games.db's engine choices could not be read, and none was copied: {}", e.message));
            return 0;
        }
        drop(tree);
        let mut copied = 0;
        for (game, engine) in choices {
            let (row_console, system) = self.console_and_system(&game);
            let Some(system) = system.filter(|_| !engine.is_empty()) else { continue };
            let console = row_console.unwrap_or_else(|| emusen_dotnet::text::to_upper(&system));
            let tree = emusen_galaxia::tree::tree();
            let discovery = self.discovery();
            let catalog = Catalog::new(&discovery);
            let offered = schema(self, &catalog, &discovery, &tree, &console, None, None).into_iter().find(|r| r.key == ENGINE_KEY).is_some_and(|r| r.choices.contains(&engine));
            if !offered {
                continue;
            }
            let mut values = crate::settings::game_values(&tree, Some(&system), &game);
            if values.iter().any(|(k, _)| k == ENGINE_KEY) {
                continue;
            }
            values.push((ENGINE_KEY.to_string(), engine));
            if write_game_values(&tree, Some(&system), &game, &values).is_ok() {
                copied += 1;
            }
        }
        let tree = emusen_galaxia::tree::tree();
        let _ = std::fs::create_dir_all(path::directory_name(Style::HOST, &marker).unwrap_or_default());
        let day = emusen_dotnet::date::local_text(emusen_galaxia::clock::local_ticks_now(), "yyyy-MM-dd");
        let _ = std::fs::write(&marker, format!("{copied} engine choice(s) copied from {games_db} on {day}\n"));
        drop(tree);
        copied
    }
}
