//! Eclipse `#21` Cheats backend: grouped list, inspector, local database card, browser, and code editor.

use crate::card::Row as CardRow;
use crate::cheats::List;
use crate::clients::ClientSource;
use crate::game_model::cover_url;
use crate::library::{console_name, title_of, LibrarySource, COVER};
use crate::question::confirm;
use crate::settings_shown::planet_of;
use crate::tools::{eclipse_help_entry, eclipse_trail, Effect};
use emusen_crystal_client::shapes::CheatCodecs;
use emusen_kit_core::input::UiButton;
use emusen_platform_client::cheats::CheatsDatabase;
use emusen_platform_client::Platform;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Keyboard erase icon SVG path reused cleanly by `EclipseKeyboard`.
const ERASE_ICON: &str = "M20 6H9l-6 6 6 6h11a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2zM17 10l-4 4M13 10l4 4";
const FILTER_SPECS: [(&str, &str); 4] = [("all", "All"), ("database", "From database"), ("yours", "Yours"), ("active", "Active")];
const SORT_CAPTION: &str = "Active first, then A\u{2013}Z";

pub const ACT_TOGGLE: &str = "toggle";
pub const ACT_MASTER: &str = "master";
pub const ACT_MORE: &str = "more";
pub const ACT_BROWSE: &str = "browse";
pub const ACT_ADD: &str = "add";
pub const ACT_UPDATE_DB: &str = "updateDb";
pub const ACT_EDIT_ROW: &str = "cheatEdit";
pub const ACT_REMOVE_ROW: &str = "cheatRemove";
const ASK_REMOVE_PREFIX: &str = "removeCheat:";

/// One entry parsed out of a game's matching `.cht` database file.
#[derive(Clone, Debug, PartialEq)]
pub struct ChtEntry {
    pub description: String,
    pub raw_code: String,
    pub badge: String,
    pub is_live_ram: bool,
    pub display_code: String,
    pub decoded: String,
    pub entry: Value,
}

/// Complete Eclipse data for one cheat-capable library game.
#[derive(Clone, Debug, PartialEq)]
pub struct EclipseGame {
    pub path: String,
    pub title: String,
    pub console_id: String,
    pub console_name: String,
    pub system_badge: String,
    pub list_console: String,
    pub core_name: String,
    pub planet: i32,
    pub cover: String,
    pub codecs_summary: String,
    pub default_space: String,
    pub auto_codec: Option<String>,
    pub explicit_codec: Option<String>,
    pub db_file: Option<(String, String, String)>,
    pub cht_entries: Vec<ChtEntry>,
    pub cht_loaded: bool,
    pub list: Option<List>,
    pub list_path_display: String,
}

/// Global local cheat database snapshot read off the GUI thread.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EclipseRead {
    pub directory: String,
    pub exists: bool,
    pub attribution: String,
    pub files: usize,
    pub newest: Option<i64>,
    pub codecs: CheatCodecs,
    pub games: Vec<EclipseGame>,
}

/// Focus region on the main `#21` Cheats surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Zone {
    Master = 0,
    Filters = 1,
    Rows = 2,
    DbBrowse = 3,
    DbUpdate = 4,
    AddCard = 5,
}

impl Zone {
    pub fn from_code_pub(code: i32) -> Self {
        Self::from_code(code)
    }

    fn from_code(code: i32) -> Self {
        match code {
            0 => Zone::Master,
            1 => Zone::Filters,
            3 => Zone::DbBrowse,
            4 => Zone::DbUpdate,
            5 => Zone::AddCard,
            _ => Zone::Rows,
        }
    }
}

#[derive(Clone, Debug)]
struct ResolvedCheat {
    orig_index: usize,
    name: String,
    desc: String,
    on: bool,
    is_live_ram: bool,
    badge: String,
    state_tag: String,
    code: String,
    decoded: String,
    from_db: bool,
    writes: Vec<NormWrite>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct NormWrite {
    space: String,
    address: u32,
    value: u32,
    width: i32,
}

/// Interactive state for Eclipse `#21` Cheats.
pub struct State {
    pub read: Option<EclipseRead>,
    pub requested_game: String,
    pub session_game: String,
    pub master_on: bool,
    pub selected_game: usize,
    pub selected_game_path: String,
    pub active_filter: String,
    pub filter_cursor: usize,
    pub zone: Zone,
    pub focused_orig_index: Option<usize>,
    pub browse_open: bool,
    pub browse_cursor: usize,
    pub editor_open: bool,
    pub editor_edit_orig: Option<usize>,
    pub editor_field: usize,
    pub editor_typing: bool,
    pub editor_desc: String,
    pub editor_code: String,
    pub editor_error: String,
    pub status_note: String,
    pub cheats_dirty_for: Option<String>,
    pub master_dirty: Option<bool>,
}

impl Default for State {
    fn default() -> Self {
        State {
            read: None,
            requested_game: String::new(),
            session_game: String::new(),
            master_on: true,
            selected_game: 0,
            selected_game_path: String::new(),
            active_filter: "all".to_owned(),
            filter_cursor: 0,
            zone: Zone::Rows,
            focused_orig_index: None,
            browse_open: false,
            browse_cursor: 0,
            editor_open: false,
            editor_edit_orig: None,
            editor_field: 0,
            editor_typing: false,
            editor_desc: String::new(),
            editor_code: String::new(),
            editor_error: String::new(),
            status_note: String::new(),
            cheats_dirty_for: None,
            master_dirty: None,
        }
    }
}

/// Reads library cheat games for Eclipse `#21`, filtering to `requested_game` when provided and deferring `.cht` decoding in overview mode.
pub fn read_eclipse(source: &ClientSource, found: &super::cheats::Found) -> Result<EclipseRead, String> {
    read_eclipse_for(source, found, None)
}

/// Reads library cheat games for Eclipse `#21`, optionally filtered to a single `requested_game`.
pub fn read_eclipse_for(source: &ClientSource, found: &super::cheats::Found, requested_game: Option<&str>) -> Result<EclipseRead, String> {
    let target = requested_game.map(str::trim).filter(|s| !s.is_empty());
    let codecs = source.runtime.cheat_codec_list().unwrap_or_default();
    let systems = source.systems().unwrap_or_default();
    let all_games = source.games().unwrap_or_default();
    let candidates: Vec<_> = all_games
        .into_iter()
        .filter_map(|game| {
            if target.is_some_and(|req| game.path != req) {
                return None;
            }
            let system = systems.iter().find(|s| Some(&s.id) == game.system.as_ref()).filter(|s| !s.cheat_systems.is_empty())?.clone();
            Some((game, system))
        })
        .collect();
    if candidates.is_empty() {
        return Ok(EclipseRead {
            directory: found.directory.clone(),
            exists: found.exists,
            attribution: found.attribution.clone(),
            files: found.files,
            newest: found.newest,
            codecs,
            games: Vec::new(),
        });
    }
    let query_games: Vec<_> = candidates.iter().map(|(g, _)| g.clone()).collect();
    #[cfg(test)]
    test_counters::record_query(query_games.len());
    let engines: HashMap<String, _> = source.engines(&systems).unwrap_or_default().into_iter().collect();
    let names: HashMap<String, String> = source.names(&query_games).unwrap_or_default().into_iter().collect();
    let covers: HashMap<String, String> = source.media(&systems, &query_games, &[COVER]).unwrap_or_default().into_iter().collect();

    let db = if found.exists { CheatsDatabase::open(&source.platform, &found.directory).ok() } else { None };
    let db_entries = db.as_ref().and_then(|d| d.entries().ok()).unwrap_or_default();

    let mut out_games = Vec::with_capacity(candidates.len());
    for (game, system) in &candidates {
        let stem = Path::new(&game.path).file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let title = names.get(&game.path).filter(|t| !t.is_empty()).cloned().unwrap_or_else(|| title_of(&game.path));
        let db_file = db.as_ref().and_then(|d| find_db_match(d, &db_entries, &stem, &system.cheat_systems));
        let (cht_entries, cht_loaded) = if target.is_some() {
            let loaded = db_file
                .as_ref()
                .map(|(_, _, cht_path)| load_cht_entries(&source.platform, &source.runtime, &system.id, cht_path, &codecs))
                .unwrap_or_default();
            (loaded, true)
        } else {
            (Vec::new(), false)
        };
        let list = List::of(&source.platform, &system.console, &game.path).ok().flatten();
        let list_path_display = list.as_ref().map(|l| format!("cheats/{}.json", l.name)).unwrap_or_else(|| format!("cheats/{stem}.json"));
        let engine = engines.get(&system.id);
        let core_name = engine.map(|e| e.name.clone()).filter(|n| !n.is_empty()).unwrap_or_else(|| "Unassigned".to_owned());
        let planet = engine.map(|e| planet_of(&e.name, e.libretro, !e.name.is_empty())).unwrap_or(-2);
        let pair = codecs.systems.iter().find(|p| p.id == system.id);
        let auto_codec = pair.and_then(|p| p.auto.clone());
        let explicit_codec = pair.and_then(|p| p.explicit.clone());
        let default_space = auto_codec
            .as_deref()
            .and_then(|id| codecs.codecs.iter().find(|c| c.id == id))
            .and_then(|c| c.space.clone())
            .unwrap_or_else(|| "CpuBus".to_owned());
        let codecs_summary = build_codecs_summary(&codecs, auto_codec.as_deref(), explicit_codec.as_deref());
        let cover = covers.get(&game.path).map(|f| cover_url(f)).unwrap_or_default();

        out_games.push(EclipseGame {
            path: game.path.clone(),
            title,
            console_id: system.id.clone(),
            console_name: console_name(system),
            system_badge: system.id.to_uppercase(),
            list_console: system.console.clone(),
            core_name,
            planet,
            cover,
            codecs_summary,
            default_space,
            auto_codec,
            explicit_codec,
            db_file,
            cht_entries,
            cht_loaded,
            list,
            list_path_display,
        });
    }

    // Put games with active/existing cheats or DB matches first, sorted by title for deterministic overview selection.
    out_games.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()).then_with(|| a.path.cmp(&b.path)));

    Ok(EclipseRead {
        directory: found.directory.clone(),
        exists: found.exists,
        attribution: found.attribution.clone(),
        files: found.files,
        newest: found.newest,
        codecs,
        games: out_games,
    })
}

/// Decodes `.cht` entries only for the first selected overview game off the GUI thread.
pub fn ensure_initial_cht(source: &ClientSource, read: &mut EclipseRead) {
    if read.games.is_empty() {
        return;
    }
    let idx = read
        .games
        .iter()
        .position(|g| g.list.as_ref().is_some_and(|l| !l.entries().is_empty()))
        .or_else(|| read.games.iter().position(|g| g.db_file.is_some()))
        .unwrap_or(0);
    load_game_cht(source, &read.codecs, &mut read.games[idx]);
}

fn load_game_cht(source: &ClientSource, codecs: &CheatCodecs, game: &mut EclipseGame) {
    if game.cht_loaded {
        return;
    }
    if let Some((_, _, cht_path)) = game.db_file.as_ref() {
        game.cht_entries = load_cht_entries(&source.platform, &source.runtime, &game.console_id, cht_path, codecs);
    }
    game.cht_loaded = true;
}

fn find_db_match(db: &CheatsDatabase, entries: &[(String, String, String)], stem: &str, cheat_systems: &[String]) -> Option<(String, String, String)> {
    let places = db.query(&json!({ "op": "find", "text": stem, "limit": entries.len().max(1) })).ok()?;
    let ours = |e: &&(String, String, String)| cheat_systems.iter().any(|c| c.eq_ignore_ascii_case(&e.0));
    places
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_u64)
        .filter_map(|at| entries.get(at as usize))
        .find(ours)
        .cloned()
}

fn build_codecs_summary(codecs: &CheatCodecs, auto_id: Option<&str>, explicit_id: Option<&str>) -> String {
    let mut parts = Vec::new();
    for id in [auto_id, explicit_id].into_iter().flatten() {
        if let Some(c) = codecs.codecs.iter().find(|c| c.id == id) {
            parts.push(format!("{} \u{00B7} {}", c.id, c.name));
        }
    }
    if parts.is_empty() { "System codecs".to_owned() } else { parts.join(" / ") }
}

fn load_cht_entries(platform: &Platform, runtime: &emusen_crystal_client::Runtime, system_id: &str, cht_path: &str, codecs: &CheatCodecs) -> Vec<ChtEntry> {
    #[cfg(test)]
    test_counters::record_cht_decode(cht_path);
    let raw_codes = std::fs::read_to_string(cht_path).map(|text| parse_cht_raw_codes(&text)).unwrap_or_default();
    let Ok(parsed) = platform.cheats_parse(&json!({ "path": cht_path, "system": system_id })) else {
        return Vec::new();
    };
    let explicit_codec = codecs.systems.iter().find(|s| s.id == system_id).and_then(|s| s.explicit.as_deref());
    let Some(entries) = parsed["file"]["Cheats"].as_array() else { return Vec::new() };
    entries
        .iter()
        .map(|entry| {
            let description = entry["Description"].as_str().unwrap_or_default().to_owned();
            let raw_code = raw_codes.get(&description.to_lowercase()).cloned().unwrap_or_default();
            let is_patch = entry["Kind"].as_str().is_some_and(|k| k.eq_ignore_ascii_case("RomPatch"));
            let writes = parse_writes(entry);
            let encoded_gg = if is_patch && raw_code.is_empty() {
                writes.first().and_then(|w| explicit_codec.and_then(|c| runtime.cheat_encode(c, w.address as i32, w.value).ok().flatten()))
            } else {
                None
            };
            let display_code = if !raw_code.is_empty() { raw_code.clone() } else { encoded_gg.unwrap_or_else(|| format_entry_code(entry, system_id, &writes)) };
            let decoded = format_decoded_line(entry, system_id, &writes);
            ChtEntry {
                description,
                raw_code,
                badge: if is_patch { "ROM Patch".to_owned() } else { "Live RAM".to_owned() },
                is_live_ram: !is_patch,
                display_code,
                decoded,
                entry: entry.clone(),
            }
        })
        .collect()
}

fn parse_cht_raw_codes(text: &str) -> HashMap<String, String> {
    let mut fields: HashMap<String, String> = HashMap::new();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else { continue };
        let val = v.trim();
        let unquoted = val.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(val);
        fields.insert(k.trim().to_ascii_lowercase(), unquoted.to_owned());
    }
    let count = fields.get("cheats").and_then(|v| v.parse::<usize>().ok()).unwrap_or(0);
    let mut out = HashMap::new();
    for i in 0..count {
        let desc = fields.get(&format!("cheat{i}_desc")).map(String::as_str).unwrap_or("");
        let code = fields.get(&format!("cheat{i}_code")).map(String::as_str).unwrap_or("");
        if !desc.is_empty() && !code.is_empty() {
            out.insert(desc.to_ascii_lowercase(), code.to_owned());
        }
    }
    out
}

fn parse_hex_u32(val: &Value) -> u32 {
    match val {
        Value::String(s) => {
            let trimmed = s.trim().trim_start_matches("0x").trim_start_matches("0X").trim_start_matches('$');
            u32::from_str_radix(trimmed, 16).unwrap_or(0)
        }
        Value::Number(n) => n.as_u64().unwrap_or(0) as u32,
        _ => 0,
    }
}

fn parse_writes(entry: &Value) -> Vec<NormWrite> {
    let mut out = Vec::new();
    if let Some(arr) = entry["Writes"].as_array() {
        for w in arr {
            let address = parse_hex_u32(&w["Address"]);
            let value = parse_hex_u32(&w["Value"]);
            let space = w["Space"].as_str().unwrap_or_default().to_owned();
            let width = w["Width"].as_i64().unwrap_or(1).clamp(1, 4) as i32;
            out.push(NormWrite { space, address, value, width });
        }
    }
    if out.is_empty() && (entry.get("Address").is_some() || entry.get("Value").is_some()) {
        out.push(NormWrite {
            space: entry["Space"].as_str().unwrap_or_default().to_owned(),
            address: parse_hex_u32(&entry["Address"]),
            value: parse_hex_u32(&entry["Value"]),
            width: 1,
        });
    }
    out
}

fn display_address(w: &NormWrite, system_id: &str) -> (String, u32) {
    if system_id == "snes" && w.space.eq_ignore_ascii_case("WRAM") && w.address < 0x02_0000 {
        ("WRAM".to_owned(), 0x7E_0000 + w.address)
    } else if system_id == "snes" && w.space.eq_ignore_ascii_case("CpuBus") && (0x7E_0000..=0x7F_FFFF).contains(&w.address) {
        ("WRAM".to_owned(), w.address)
    } else {
        let space = if w.space.is_empty() { "ROM".to_owned() } else { w.space.clone() };
        (space, w.address)
    }
}

fn format_entry_code(entry: &Value, system_id: &str, writes: &[NormWrite]) -> String {
    let is_patch = entry["Kind"].as_str().is_some_and(|k| k.eq_ignore_ascii_case("RomPatch"));
    let Some(w) = writes.first() else { return "\u{2014}".to_owned() };
    let val_digits = (w.width as usize * 2).max(2);
    if is_patch {
        return format!("${:06X}:{:0val_digits$X}", w.address, w.value);
    }
    let (_, addr) = display_address(w, system_id);
    let addr_digits = if addr > 0xFFFF { 6 } else { 4 };
    if writes.len() > 1 {
        format!("{:0addr_digits$X}:{:0val_digits$X} +{}", addr, w.value, writes.len() - 1)
    } else {
        format!("{:0addr_digits$X}:{:0val_digits$X}", addr, w.value)
    }
}

fn primary_write_summary(entry: &Value, system_id: &str, writes: &[NormWrite]) -> Option<(bool, String, String, String)> {
    let is_patch = entry["Kind"].as_str().is_some_and(|k| k.eq_ignore_ascii_case("RomPatch"));
    let w = writes.first()?;
    let val_digits = (w.width as usize * 2).max(2);
    let (space, addr) = display_address(w, system_id);
    let addr_digits = if addr > 0xFFFF { 6 } else { 4 };
    Some((is_patch, space, format!("{addr:0addr_digits$X}"), format!("{:0val_digits$X}", w.value)))
}

fn format_decoded_line(entry: &Value, system_id: &str, writes: &[NormWrite]) -> String {
    let Some((is_patch, space, addr_hex, val_hex)) = primary_write_summary(entry, system_id, writes) else { return String::new() };
    if is_patch {
        match entry["Compare"].as_str().filter(|s| !s.is_empty()) {
            Some(c) => format!("ROM ${addr_hex} \u{2190} 0x{val_hex} if 0x{c}"),
            None => format!("ROM ${addr_hex} \u{2190} 0x{val_hex} (patch)"),
        }
    } else {
        format!("{space} ${addr_hex} \u{2190} 0x{val_hex} / frame")
    }
}

fn describe_writes(entry: &Value, system_id: &str, writes: &[NormWrite]) -> String {
    let Some((is_patch, space, addr_hex, val_hex)) = primary_write_summary(entry, system_id, writes) else { return String::new() };
    if is_patch {
        format!("Patches cartridge byte ${addr_hex} to 0x{val_hex}.")
    } else if writes.len() > 1 {
        format!("Writes {} {space} addresses each frame, starting at ${addr_hex}.", writes.len())
    } else {
        format!("Writes 0x{val_hex} to {space} ${addr_hex} each frame.")
    }
}

fn write_sig(entry: &Value, writes: &[NormWrite]) -> (bool, Vec<(String, u32, u32, i32)>) {
    let is_patch = entry["Kind"].as_str().is_some_and(|k| k.eq_ignore_ascii_case("RomPatch"));
    let sig = writes.iter().map(|w| (w.space.to_ascii_lowercase(), w.address, w.value, w.width)).collect();
    (is_patch, sig)
}

fn resolve_game_cheats(game: &EclipseGame) -> Vec<ResolvedCheat> {
    let mut db_by_sig: BTreeMap<(bool, Vec<(String, u32, u32, i32)>), &ChtEntry> = BTreeMap::new();
    let mut db_by_desc: HashMap<String, &ChtEntry> = HashMap::new();
    for cht in &game.cht_entries {
        let cht_writes = parse_writes(&cht.entry);
        db_by_sig.insert(write_sig(&cht.entry, &cht_writes), cht);
        db_by_desc.insert(cht.description.trim().to_ascii_lowercase(), cht);
    }
    let Some(list) = game.list.as_ref() else { return Vec::new() };
    list.entries()
        .iter()
        .enumerate()
        .map(|(orig_index, entry)| {
            let name = entry["Description"].as_str().unwrap_or_default().to_owned();
            let on = entry["Enabled"].as_bool() == Some(true);
            let is_patch = entry["Kind"].as_str().is_some_and(|k| k.eq_ignore_ascii_case("RomPatch"));
            let writes = parse_writes(entry);
            let sig = write_sig(entry, &writes);
            let matched = db_by_sig.get(&sig).copied().or_else(|| db_by_desc.get(&name.trim().to_ascii_lowercase()).copied());
            let from_db = matched.is_some();
            let code = matched.map(|c| c.display_code.clone()).filter(|c| !c.is_empty()).unwrap_or_else(|| format_entry_code(entry, &game.console_id, &writes));
            let decoded = format_decoded_line(entry, &game.console_id, &writes);
            let desc = describe_writes(entry, &game.console_id, &writes);
            let state_tag = format!("{} \u{00B7} {}", if on { "ON" } else { "OFF" }, if is_patch { "ROM PATCH" } else { "RAM POKE" });
            ResolvedCheat {
                orig_index,
                name,
                desc,
                on,
                is_live_ram: !is_patch,
                badge: if is_patch { "ROM Patch".to_owned() } else { "Live RAM".to_owned() },
                state_tag,
                code,
                decoded,
                from_db,
                writes,
            }
        })
        .collect()
}

fn short_attribution(raw: &str, exists: bool) -> String {
    if !exists {
        return "Not installed".to_owned();
    }
    if raw.contains("CC BY-SA 4.0") && raw.to_ascii_lowercase().contains("libretro") {
        "CC BY-SA 4.0 \u{00B7} libretro".to_owned()
    } else if raw.contains("MIT") && raw.to_ascii_lowercase().contains("libretro") {
        "MIT \u{00B7} libretro".to_owned()
    } else if raw.is_empty() {
        "Local DB".to_owned()
    } else {
        raw.lines().next().unwrap_or("Local DB").trim().chars().take(36).collect()
    }
}

/// Validates and parses user-entered description and code into a Galaxia `CheatFileEntry`.
pub fn parse_user_cheat(platform: &Platform, game: &EclipseGame, desc: &str, raw_code: &str) -> Result<Value, String> {
    let desc = desc.trim();
    if desc.is_empty() {
        return Err("Enter a description for the cheat.".to_owned());
    }
    let code = raw_code.trim();
    if code.is_empty() {
        return Err("Enter a cheat code.".to_owned());
    }
    let try_parse = |candidate: &str| -> Option<Value> {
        let text = format!("cheats = 1\ncheat0_desc = \"{}\"\ncheat0_code = \"{candidate}\"\ncheat0_enable = true\n", desc.replace('"', "\\\""));
        let parsed = platform.cheats_parse(&json!({ "text": text, "system": game.console_id })).ok()?;
        parsed["file"]["Cheats"].as_array().and_then(|a| a.first()).cloned()
    };
    if let Some(entry) = try_parse(code) {
        return Ok(entry);
    }
    // Strip colons/spaces across `+`-separated parts for codecs such as `snes.par` that accept 8 bare hex digits (`7E00F2:04` -> `7E00F204`).
    if code.contains(':') {
        let compact: String = code
            .split('+')
            .map(|p| p.chars().filter(|c| !c.is_whitespace() && *c != ':').collect::<String>())
            .collect::<Vec<_>>()
            .join("+");
        if let Some(entry) = try_parse(&compact) {
            return Ok(entry);
        }
        // Fallback explicit hex `ADDR:VALUE` write on systems that expose a cheat write space.
        if let Some(entry) = parse_explicit_colon_code(game, desc, code) {
            return Ok(entry);
        }
    }
    Err(format!("'{code}' is not a valid cheat code for {}.", game.console_name))
}

fn parse_explicit_colon_code(game: &EclipseGame, desc: &str, code: &str) -> Option<Value> {
    let mut writes = Vec::new();
    for part in code.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        let (addr_s, val_s) = part.split_once(':')?;
        let addr_s = addr_s.trim().trim_start_matches("0x").trim_start_matches("0X").trim_start_matches('$');
        let val_s = val_s.trim().trim_start_matches("0x").trim_start_matches("0X");
        if addr_s.is_empty() || val_s.is_empty() || addr_s.len() > 8 || val_s.len() > 8 {
            return None;
        }
        if !addr_s.chars().all(|c| c.is_ascii_hexdigit()) || !val_s.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let addr = u32::from_str_radix(addr_s, 16).ok()?;
        let val = u32::from_str_radix(val_s, 16).ok()?;
        let width_digits = if val > 0xFFFF { 8 } else if val > 0xFF { 4 } else { 2 };
        let width = width_digits / 2;
        writes.push(json!({
            "Space": game.default_space,
            "Address": format!("{addr:06X}"),
            "Value": format!("{val:0width_digits$X}"),
            "Width": width,
            "Type": "Set",
        }));
    }
    (!writes.is_empty()).then(|| json!({ "Kind": "RamPoke", "Description": desc, "Enabled": true, "Writes": writes }))
}

impl State {
    pub fn set_target(&mut self, game: &str, session_game: &str, master_on: bool) {
        let changed_req = self.requested_game != game;
        self.requested_game = game.to_owned();
        self.session_game = session_game.to_owned();
        self.master_on = master_on;
        if changed_req {
            self.apply_requested_game();
        }
    }

    pub fn take_read(&mut self, read: EclipseRead) {
        self.read = Some(read);
        self.apply_requested_game();
        self.clamp_focus();
    }

    pub fn ensure_current_cht(&mut self, source: &ClientSource) {
        let Some(read) = self.read.as_mut() else { return };
        if let Some(game) = read.games.get_mut(self.selected_game) {
            load_game_cht(source, &read.codecs, game);
        }
    }

    pub fn needs_refresh(&self) -> bool {
        !self.requested_game.is_empty() && self.read.as_ref().is_some_and(|r| !r.games.iter().any(|g| g.path == self.requested_game))
    }

    fn apply_requested_game(&mut self) {
        let Some(read) = self.read.as_ref() else { return };
        if read.games.is_empty() {
            self.selected_game = 0;
            self.selected_game_path.clear();
            self.focused_orig_index = None;
            return;
        }
        let next_idx = if !self.requested_game.is_empty() {
            read.games.iter().position(|g| g.path == self.requested_game).unwrap_or(0)
        } else if !self.selected_game_path.is_empty() {
            read.games.iter().position(|g| g.path == self.selected_game_path).unwrap_or(0)
        } else {
            read.games
                .iter()
                .position(|g| g.list.as_ref().is_some_and(|l| !l.entries().is_empty()))
                .or_else(|| read.games.iter().position(|g| g.db_file.is_some()))
                .unwrap_or(0)
        };
        self.selected_game_path = read.games[next_idx].path.clone();
        if next_idx != self.selected_game {
            self.selected_game = next_idx;
            self.focused_orig_index = None;
            self.browse_open = false;
            self.editor_open = false;
        }
    }

    fn current_game(&self) -> Option<&EclipseGame> {
        self.read.as_ref()?.games.get(self.selected_game)
    }

    fn current_game_mut(&mut self) -> Option<&mut EclipseGame> {
        self.read.as_mut()?.games.get_mut(self.selected_game)
    }

    fn sorted_groups(&self) -> (Vec<ResolvedCheat>, Vec<ResolvedCheat>, Vec<usize>, usize, usize, usize, usize) {
        let Some(game) = self.current_game() else {
            return (Vec::new(), Vec::new(), Vec::new(), 0, 0, 0, 0);
        };
        let all = resolve_game_cheats(game);
        let total = all.len();
        let db_count = all.iter().filter(|c| c.from_db).count();
        let yours_count = all.iter().filter(|c| !c.from_db).count();
        let active_count = all.iter().filter(|c| c.on).count();

        let matches_filter = |c: &ResolvedCheat| match self.active_filter.as_str() {
            "database" => c.from_db,
            "yours" => !c.from_db,
            "active" => c.on,
            _ => true,
        };
        let sort_group = |mut items: Vec<ResolvedCheat>| {
            items.sort_by(|a, b| b.on.cmp(&a.on).then_with(|| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase())).then_with(|| a.orig_index.cmp(&b.orig_index)));
            items
        };
        let db_group = sort_group(all.iter().filter(|c| c.from_db && matches_filter(c)).cloned().collect());
        let yours_group = sort_group(all.iter().filter(|c| !c.from_db && matches_filter(c)).cloned().collect());
        let order: Vec<usize> = db_group.iter().chain(yours_group.iter()).map(|c| c.orig_index).collect();
        (db_group, yours_group, order, total, db_count, yours_count, active_count)
    }

    fn clamp_focus(&mut self) {
        let (_, _, order, _, _, _, _) = self.sorted_groups();
        if let Some(cur) = self.focused_orig_index {
            if !order.contains(&cur) {
                self.focused_orig_index = order.first().copied();
            }
        } else {
            self.focused_orig_index = order.first().copied();
        }
        if order.is_empty() && self.zone == Zone::Rows {
            self.zone = Zone::AddCard;
        }
    }

    pub fn choose_filter(&mut self, key: &str) {
        if let Some(pos) = FILTER_SPECS.iter().position(|(k, _)| *k == key) {
            self.active_filter = key.to_owned();
            self.filter_cursor = pos;
            if self.sorted_groups().2.is_empty() {
                self.zone = Zone::Filters;
            }
            self.clamp_focus();
        }
    }

    pub fn step_filter(&mut self, delta: i32) {
        let idx = (FILTER_SPECS.iter().position(|(k, _)| *k == self.active_filter).unwrap_or(0) as i32 + delta).rem_euclid(FILTER_SPECS.len() as i32) as usize;
        let key = FILTER_SPECS[idx].0;
        self.choose_filter(key);
    }

    pub fn step_game(&mut self, delta: i32) {
        let len = self.read.as_ref().map_or(0, |r| r.games.len());
        if len <= 1 {
            return;
        }
        self.selected_game = (self.selected_game as i32 + delta).rem_euclid(len as i32) as usize;
        if let Some(p) = self.read.as_ref().and_then(|r| r.games.get(self.selected_game)).map(|g| g.path.clone()) {
            self.selected_game_path = p;
        }
        self.focused_orig_index = None;
        self.browse_open = false;
        self.editor_open = false;
        self.clamp_focus();
    }

    pub fn focus_cheat_key(&mut self, key: &str) {
        if let Ok(idx) = key.parse::<usize>() {
            self.focused_orig_index = Some(idx);
            self.zone = Zone::Rows;
            self.clamp_focus();
        }
    }

    pub fn toggle_cheat(&mut self, platform: &Platform, orig_index: usize) {
        let Some(game) = self.current_game_mut() else { return };
        let path = game.path.clone();
        let Some(list) = game.list.as_mut() else { return };
        let current = list.entries().get(orig_index).and_then(|e| e["Enabled"].as_bool()) == Some(true);
        if list.set(orig_index, !current) {
            if let Err(why) = list.save(platform) {
                self.status_note = format!("Could not save cheats: {why}");
            } else {
                self.status_note.clear();
                self.cheats_dirty_for = Some(path);
            }
        }
        self.focused_orig_index = Some(orig_index);
        self.clamp_focus();
    }

    pub fn toggle_master(&mut self) {
        self.master_on = !self.master_on;
        self.master_dirty = Some(self.master_on);
    }

    pub fn open_browse(&mut self) {
        let has_cht = self.current_game().is_some_and(|g| !g.cht_entries.is_empty());
        if has_cht {
            self.browse_open = true;
            self.browse_cursor = 0;
        }
    }

    pub fn open_add_editor(&mut self) {
        self.editor_open = true;
        self.editor_edit_orig = None;
        self.editor_field = 0;
        self.editor_typing = false;
        self.editor_desc.clear();
        self.editor_code.clear();
        self.editor_error.clear();
    }

    pub fn open_edit_editor(&mut self, orig_index: usize) {
        let Some(game) = self.current_game() else { return };
        let resolved = resolve_game_cheats(game);
        let Some(cheat) = resolved.iter().find(|c| c.orig_index == orig_index) else { return };
        let edit_code = if cheat.from_db || cheat.writes.len() <= 1 {
            cheat.code.clone()
        } else {
            let entry = game.list.as_ref().and_then(|l| l.entries().get(orig_index));
            cheat
                .writes
                .iter()
                .map(|w| entry.map_or_else(|| cheat.code.clone(), |e| format_entry_code(e, &game.console_id, std::slice::from_ref(w))))
                .collect::<Vec<_>>()
                .join(" + ")
        };
        self.editor_open = true;
        self.editor_edit_orig = Some(orig_index);
        self.editor_field = 0;
        self.editor_typing = false;
        self.editor_desc = cheat.name.clone();
        self.editor_code = edit_code;
        self.editor_error.clear();
    }

    pub fn set_field(&mut self, field: i32, text: &str) {
        match field {
            0 => self.editor_desc = text.to_owned(),
            1 => self.editor_code = text.to_owned(),
            _ => {}
        }
        self.editor_error.clear();
    }

    pub fn submit_editor(&mut self, platform: &Platform) {
        let Some(game_clone) = self.current_game().cloned() else { return };
        let entry = match parse_user_cheat(platform, &game_clone, &self.editor_desc, &self.editor_code) {
            Ok(e) => e,
            Err(why) => {
                self.editor_error = why;
                return;
            }
        };
        let edit_orig = self.editor_edit_orig;
        let Some(game) = self.current_game_mut() else { return };
        let mut list = match game.list.take() {
            Some(l) => l,
            None => match List::of(platform, &game.list_console, &game.path) {
                Ok(Some(l)) => l,
                _ => {
                    self.editor_error = format!("{} cannot store a cheat file.", game.title);
                    return;
                }
            },
        };
        let arr = match list.file.get_mut("Cheats").and_then(Value::as_array_mut) {
            Some(a) => a,
            None => {
                list.file = json!({ "Cheats": [] });
                list.file.get_mut("Cheats").and_then(Value::as_array_mut).unwrap()
            }
        };
        let target_idx = match edit_orig {
            Some(idx) if idx < arr.len() => {
                let keep_on = arr[idx]["Enabled"].as_bool() == Some(true);
                let mut updated = entry;
                if let Some(obj) = updated.as_object_mut() {
                    obj.insert("Enabled".into(), Value::Bool(keep_on));
                }
                arr[idx] = updated;
                idx
            }
            _ => {
                arr.push(entry);
                arr.len() - 1
            }
        };
        let saved = list.save(platform);
        let path = game.path.clone();
        game.list = Some(list);
        match saved {
            Ok(()) => {
                self.editor_open = false;
                self.editor_typing = false;
                self.editor_error.clear();
                self.focused_orig_index = Some(target_idx);
                self.zone = Zone::Rows;
                self.cheats_dirty_for = Some(path);
                self.clamp_focus();
            }
            Err(why) => self.editor_error = format!("Could not save cheats: {why}"),
        }
    }

    pub fn import_cht_entry(&mut self, platform: &Platform, cht_idx: usize) {
        let Some(game) = self.current_game_mut() else { return };
        let Some(cht) = game.cht_entries.get(cht_idx).cloned() else { return };
        let resolved = resolve_game_cheats(game);
        let cht_writes = parse_writes(&cht.entry);
        let cht_sig = write_sig(&cht.entry, &cht_writes);
        if let Some(existing) = resolved.iter().find(|r| write_sig(&game.list.as_ref().unwrap().entries()[r.orig_index], &r.writes) == cht_sig || r.name.eq_ignore_ascii_case(&cht.description)) {
            let idx = existing.orig_index;
            self.toggle_cheat(platform, idx);
            return;
        }
        let mut list = match game.list.take() {
            Some(l) => l,
            None => match List::of(platform, &game.list_console, &game.path) {
                Ok(Some(l)) => l,
                _ => return,
            },
        };
        let mut to_add = cht.entry;
        if let Some(obj) = to_add.as_object_mut() {
            obj.insert("Enabled".into(), Value::Bool(true));
        }
        if let Some(arr) = list.file.get_mut("Cheats").and_then(Value::as_array_mut) {
            arr.push(to_add);
        } else {
            list.file = json!({ "Cheats": [to_add] });
        }
        let new_idx = list.entries().len().saturating_sub(1);
        let saved = list.save(platform);
        let path = game.path.clone();
        game.list = Some(list);
        if saved.is_ok() {
            self.focused_orig_index = Some(new_idx);
            self.cheats_dirty_for = Some(path);
            self.clamp_focus();
        }
    }

    pub fn remove_cheat(&mut self, platform: &Platform, orig_index: usize) {
        let Some(game) = self.current_game_mut() else { return };
        let path = game.path.clone();
        let Some(list) = game.list.as_mut() else { return };
        let Some(arr) = list.file.get_mut("Cheats").and_then(Value::as_array_mut) else { return };
        if orig_index >= arr.len() {
            return;
        }
        arr.remove(orig_index);
        if let Err(why) = list.save(platform) {
            self.status_note = format!("Could not save cheats: {why}");
        } else {
            self.status_note.clear();
            self.focused_orig_index = None;
            self.cheats_dirty_for = Some(path);
            self.clamp_focus();
        }
    }

    pub fn navigate(&mut self, dir: &str) -> Effect {
        if self.browse_open {
            let n = self.current_game().map_or(0, |g| g.cht_entries.len());
            match dir {
                "up" if self.browse_cursor > 0 => self.browse_cursor -= 1,
                "down" if self.browse_cursor + 1 < n => self.browse_cursor += 1,
                _ => {}
            }
            return Effect::Done;
        }
        if self.editor_open {
            if !self.editor_typing {
                match dir {
                    "up" if self.editor_field > 0 => self.editor_field -= 1,
                    "down" if self.editor_field < 3 => self.editor_field += 1,
                    "left" if self.editor_field == 3 => self.editor_field = 2,
                    "right" if self.editor_field == 2 => self.editor_field = 3,
                    _ => {}
                }
            }
            return Effect::Done;
        }
        let (_, _, order, _, _, _, _) = self.sorted_groups();
        let has_browse = self.current_game().is_some_and(|g| !g.cht_entries.is_empty());
        match dir {
            "up" => match self.zone {
                Zone::Master => {}
                Zone::Filters => self.zone = Zone::Master,
                Zone::Rows => {
                    let pos = self.focused_orig_index.and_then(|k| order.iter().position(|&x| x == k)).unwrap_or(0);
                    if pos > 0 {
                        self.focused_orig_index = Some(order[pos - 1]);
                    } else {
                        self.zone = Zone::Filters;
                    }
                }
                Zone::DbBrowse => self.zone = Zone::Master,
                Zone::DbUpdate => self.zone = if has_browse { Zone::DbBrowse } else { Zone::Master },
                Zone::AddCard => self.zone = Zone::DbUpdate,
            },
            "down" => match self.zone {
                Zone::Master => self.zone = Zone::Filters,
                Zone::Filters => {
                    if let Some(first) = order.first().copied() {
                        self.focused_orig_index = Some(first);
                        self.zone = Zone::Rows;
                    } else {
                        self.zone = if has_browse { Zone::DbBrowse } else { Zone::AddCard };
                    }
                }
                Zone::Rows => {
                    let pos = self.focused_orig_index.and_then(|k| order.iter().position(|&x| x == k)).unwrap_or(0);
                    if pos + 1 < order.len() {
                        self.focused_orig_index = Some(order[pos + 1]);
                    }
                }
                Zone::DbBrowse => self.zone = Zone::DbUpdate,
                Zone::DbUpdate => self.zone = Zone::AddCard,
                Zone::AddCard => {}
            },
            "left" => match self.zone {
                Zone::Filters => {
                    if self.filter_cursor > 0 {
                        self.filter_cursor -= 1;
                    }
                }
                Zone::DbBrowse | Zone::DbUpdate | Zone::AddCard => {
                    self.zone = if order.is_empty() { Zone::Filters } else { Zone::Rows };
                }
                Zone::Master => self.zone = Zone::Filters,
                Zone::Rows => {}
            },
            "right" => match self.zone {
                Zone::Filters => {
                    if self.filter_cursor + 1 < FILTER_SPECS.len() {
                        self.filter_cursor += 1;
                    } else {
                        self.zone = if has_browse { Zone::DbBrowse } else { Zone::DbUpdate };
                    }
                }
                Zone::Rows => self.zone = if has_browse { Zone::DbBrowse } else { Zone::DbUpdate },
                Zone::DbBrowse | Zone::DbUpdate | Zone::AddCard => {}
                Zone::Master => {}
            },
            _ => {}
        }
        Effect::Done
    }

    pub fn eclipse_act(&mut self, platform: &Platform, action: &str, key: Option<&str>, downloading: bool) -> (Effect, bool) {
        if action == "closeOverlay" {
            if self.editor_typing {
                self.editor_typing = false;
            } else if self.editor_open {
                self.editor_open = false;
                self.editor_error.clear();
            } else if self.browse_open {
                self.browse_open = false;
            }
            return (Effect::Done, false);
        }
        if self.browse_open {
            match action {
                ACT_TOGGLE | "activate" => {
                    let idx = key.and_then(|k| k.parse::<usize>().ok()).unwrap_or(self.browse_cursor);
                    self.browse_cursor = idx;
                    self.import_cht_entry(platform, idx);
                }
                _ => {}
            }
            return (Effect::Done, false);
        }
        if self.editor_open {
            match action {
                "focusField" => {
                    if let Some(f) = key.and_then(|k| k.parse::<usize>().ok()) {
                        self.editor_field = f.min(3);
                        self.editor_typing = self.editor_field <= 1;
                    }
                }
                "focusField0" => {
                    self.editor_field = 0;
                    self.editor_typing = true;
                }
                "focusField1" => {
                    self.editor_field = 1;
                    self.editor_typing = true;
                }
                "stopTyping" => self.editor_typing = false,
                "saveEditor" => self.submit_editor(platform),
                "cancelEditor" => {
                    self.editor_open = false;
                    self.editor_typing = false;
                    self.editor_error.clear();
                }
                ACT_TOGGLE | "activate" => match self.editor_field {
                    0 | 1 => self.editor_typing = !self.editor_typing,
                    2 => self.submit_editor(platform),
                    _ => {
                        self.editor_open = false;
                        self.editor_typing = false;
                        self.editor_error.clear();
                    }
                },
                _ => {}
            }
            return (Effect::Done, false);
        }

        let mut db_toggle = false;
        let eff = match action {
            ACT_MASTER => {
                self.toggle_master();
                Effect::Done
            }
            ACT_BROWSE => {
                self.open_browse();
                Effect::Done
            }
            ACT_ADD => {
                self.open_add_editor();
                Effect::Done
            }
            ACT_UPDATE_DB => {
                db_toggle = true;
                Effect::Done
            }
            "prevFilter" => {
                self.step_filter(-1);
                Effect::Done
            }
            "nextFilter" => {
                self.step_filter(1);
                Effect::Done
            }
            "prevGame" => {
                self.step_game(-1);
                Effect::Done
            }
            "nextGame" => {
                self.step_game(1);
                Effect::Done
            }
            "chooseFilter" => {
                if let Some(k) = key {
                    self.choose_filter(k);
                }
                Effect::Done
            }
            ACT_MORE => {
                let target = key.and_then(|k| k.parse::<usize>().ok()).or(self.focused_orig_index);
                if let Some(idx) = target {
                    self.focused_orig_index = Some(idx);
                    let rows = vec![CardRow::new(ACT_EDIT_ROW, "Edit code\u{2026}"), CardRow::new(ACT_REMOVE_ROW, "Remove from list\u{2026}")];
                    Effect::Page("Cheat options".to_owned(), rows)
                } else {
                    Effect::Done
                }
            }
            ACT_EDIT_ROW => {
                if let Some(idx) = key.and_then(|k| k.parse::<usize>().ok()).or(self.focused_orig_index) {
                    self.open_edit_editor(idx);
                }
                Effect::Done
            }
            ACT_REMOVE_ROW => {
                if let Some(idx) = key.and_then(|k| k.parse::<usize>().ok()).or(self.focused_orig_index) {
                    let name = resolve_game_cheats_name(self.current_game(), idx);
                    Effect::Ask(confirm("Remove this cheat?", &[&name, "This deletes it from the game's cheat file."]), format!("{ASK_REMOVE_PREFIX}{idx}"))
                } else {
                    Effect::Done
                }
            }
            ACT_TOGGLE | "activate" => {
                if let Some(k) = key {
                    if let Ok(idx) = k.parse::<usize>() {
                        self.toggle_cheat(platform, idx);
                        return (Effect::Done, false);
                    }
                }
                match self.zone {
                    Zone::Master => self.toggle_master(),
                    Zone::Filters => {
                        let fk = FILTER_SPECS[self.filter_cursor.min(FILTER_SPECS.len() - 1)].0;
                        self.choose_filter(fk);
                    }
                    Zone::Rows => {
                        if let Some(idx) = self.focused_orig_index {
                            self.toggle_cheat(platform, idx);
                        }
                    }
                    Zone::DbBrowse => self.open_browse(),
                    Zone::DbUpdate => {
                        let _ = downloading;
                        db_toggle = true;
                    }
                    Zone::AddCard => self.open_add_editor(),
                }
                Effect::Done
            }
            _ => Effect::Done,
        };
        (eff, db_toggle)
    }

    pub fn answer(&mut self, asked: &str, yes: bool, platform: &Platform) {
        if let Some(rest) = asked.strip_prefix(ASK_REMOVE_PREFIX) {
            if yes {
                if let Ok(idx) = rest.parse::<usize>() {
                    self.remove_cheat(platform, idx);
                }
            }
        }
    }

    pub fn projection(&self, downloading: bool) -> Value {
        let Some(read) = self.read.as_ref() else {
            return json!({
                "ready": false,
                "title": "Cheats",
                "trail": eclipse_trail(&["Tools", "Cheats"]),
                "eraseIcon": ERASE_ICON,
                "footerHelp": [],
            });
        };

        let Some(game) = self.current_game() else {
            let db_copy = if !read.exists {
                format!("No cheat database found at {}.", read.directory)
            } else {
                format!("Database holds {} files in {}.", grouped(read.files), read.directory)
            };
            return json!({
                "ready": true,
                "hasGame": false,
                "trail": eclipse_trail(&["Tools", "Cheats"]),
                "gameTitle": "No cheat-capable games in library",
                "systemBadge": "",
                "cover": "",
                "coreText": "",
                "planet": -2,
                "listPathText": "",
                "dbMatchText": "",
                "activeSummary": "0 of 0 active",
                "appliedSub": "Add a supported ROM to manage cheats",
                "masterOn": self.master_on,
                "masterLabel": if self.master_on { "Cheats on" } else { "Cheats off" },
                "masterFocused": false,
                "filters": [],
                "sortCaption": SORT_CAPTION,
                "groups": [],
                "inspector": { "empty": true, "eyebrow": "FOCUSED CHEAT", "name": "No game selected", "stateTag": "", "stateActive": false, "code": "", "decoded": "", "sentence": "", "facts": [] },
                "dbCard": { "title": "Local cheat database", "attributionTag": short_attribution(&read.attribution, read.exists), "meterPct": 0.0, "copy": db_copy, "showBrowse": false, "browseLabel": "", "browseFocused": false, "updateLabel": if downloading { "Stop download" } else if read.exists { "Update database\u{2026}" } else { "Get database\u{2026}" }, "updateFocused": true, "downloading": downloading },
                "addCard": { "pill": "VIEW", "title": "Add a cheat code\u{2026}", "copy": "Open a supported game to add codes.", "focused": false },
                "positionCounter": "0 of 0",
                "overlayOpen": false,
                "browseOpen": false,
                "editorOpen": false,
                "percentSuffix": "%",
                "actionLines": "",
                "eraseIcon": ERASE_ICON,
                "footerHelp": [
                    { "button": UiButton::Back.index() as i32, "label": "Back", "enabled": true }
                ],
            });
        };

        let (db_group, yours_group, order, total, db_count, yours_count, active_count) = self.sorted_groups();
        let all_resolved = resolve_game_cheats(game);
        let focused_orig = self.focused_orig_index.filter(|k| order.contains(k)).or_else(|| order.first().copied());
        let focused_pos = focused_orig.and_then(|k| order.iter().position(|&x| x == k));
        let is_live_session = !self.session_game.is_empty() && self.session_game == game.path;

        let trail = eclipse_trail(&["Consoles", &game.console_name, &game.title, "Cheats"]);
        let db_match_text = match &game.db_file {
            Some((_, name, _)) => format!("Database match: {name}.cht"),
            None => "Database: No match".to_owned(),
        };
        let active_summary = format!("{active_count} of {total} active");
        let applied_sub = if !self.master_on {
            "Master switch is off".to_owned()
        } else if is_live_session {
            "Applied live to session".to_owned()
        } else {
            "Applied on next launch".to_owned()
        };
        let filters: Vec<Value> = FILTER_SPECS
            .iter()
            .enumerate()
            .map(|(i, (k, label))| {
                let count = match *k {
                    "database" => db_count,
                    "yours" => yours_count,
                    "active" => active_count,
                    _ => total,
                };
                json!({
                    "key": k,
                    "label": label,
                    "count": count.to_string(),
                    "active": *k == self.active_filter,
                    "focused": self.zone == Zone::Filters && self.filter_cursor == i,
                    "objectName": format!("filterPill.{k}"),
                    "labelObjectName": format!("filterWords.{k}"),
                })
            })
            .collect();

        let mut vis_idx = 0usize;
        let mut to_row_json = |c: &ResolvedCheat| {
            let cur = vis_idx;
            vis_idx += 1;
            json!({
                "key": c.orig_index.to_string(),
                "origIndex": c.orig_index,
                "visualIndex": cur,
                "name": c.name,
                "desc": c.desc,
                "on": c.on,
                "isLiveRam": c.is_live_ram,
                "badge": c.badge,
                "code": c.code,
                "focused": self.zone == Zone::Rows && focused_orig == Some(c.orig_index),
                "objectName": format!("cheatRow.{cur}"),
                "bgObjectName": format!("cheatRowBg.{cur}"),
                "ringObjectName": format!("cheatRowRing.{cur}"),
            })
        };

        let mut groups = Vec::new();
        if !db_group.is_empty() {
            let rows: Vec<Value> = db_group.iter().map(&mut to_row_json).collect();
            groups.push(json!({ "title": format!("From the database \u{00B7} {}", db_group.len()), "rows": rows, "objectName": "groupLabel.database" }));
        }
        if !yours_group.is_empty() {
            let rows: Vec<Value> = yours_group.iter().map(&mut to_row_json).collect();
            groups.push(json!({ "title": format!("Yours \u{00B7} {}", yours_group.len()), "rows": rows, "objectName": "groupLabel.yours" }));
        }

        let focused_cheat = focused_orig.and_then(|idx| all_resolved.iter().find(|c| c.orig_index == idx));
        let inspector = match focused_cheat {
            Some(c) => {
                let takes_effect = if !self.master_on {
                    "Paused while master switch is off"
                } else if is_live_session {
                    "Immediately in running session"
                } else {
                    "Next launch"
                };
                let origin = if c.from_db {
                    game.db_file.as_ref().map(|(_, f, _)| format!("{f}.cht")).unwrap_or_else(|| "Database".to_owned())
                } else {
                    "Entered by you".to_owned()
                };
                let conflicts = find_conflicts(c, &all_resolved);
                let mut facts = vec![
                    json!({ "label": "Takes effect", "value": takes_effect, "mono": false, "planet": -3 }),
                    json!({ "label": "Origin", "value": origin, "mono": c.from_db, "planet": -3 }),
                    json!({ "label": "Core codec", "value": game.codecs_summary, "mono": false, "planet": game.planet }),
                ];
                if !conflicts.is_empty() {
                    facts.push(json!({ "label": "Not with", "value": conflicts.join(", "), "mono": false, "planet": -3 }));
                }
                json!({
                    "empty": false,
                    "eyebrow": "FOCUSED CHEAT",
                    "name": c.name,
                    "stateTag": c.state_tag,
                    "stateActive": c.on,
                    "code": c.code,
                    "decoded": c.decoded,
                    "sentence": c.desc,
                    "facts": facts,
                })
            }
            None => json!({
                "empty": true,
                "eyebrow": "FOCUSED CHEAT",
                "name": if total == 0 { "No cheats in this game's list" } else { "No cheats match this filter" },
                "stateTag": "",
                "stateActive": false,
                "code": "\u{2014}",
                "decoded": "",
                "sentence": if total == 0 {
                    "Add a custom code with VIEW or import cheats from a matching database file."
                } else {
                    "Choose another filter tab with LB or RB."
                },
                "facts": [
                    { "label": "Core codec", "value": game.codecs_summary, "mono": false, "planet": game.planet }
                ],
            }),
        };

        let db_added = all_resolved.iter().filter(|c| c.from_db).count();
        let cht_total = game.cht_entries.len();
        let meter_pct = if cht_total > 0 { (db_added as f64 / cht_total as f64).min(1.0) } else { 0.0 };
        let newest_words = read.newest.map(format_epoch_date).unwrap_or_default();
        let db_copy = if !read.exists {
            format!("No cheat database in {}. Download it to browse .cht files offline.", read.directory)
        } else if let Some((_, file_name, _)) = &game.db_file {
            if newest_words.is_empty() {
                format!("{db_added} of {cht_total} codes from {file_name}.cht are in your list. Database holds {} files.", grouped(read.files))
            } else {
                format!("{db_added} of {cht_total} codes from {file_name}.cht are in your list. Database holds {} files \u{00B7} newest {newest_words}.", grouped(read.files))
            }
        } else if newest_words.is_empty() {
            format!("No .cht match for this game. Database holds {} files.", grouped(read.files))
        } else {
            format!("No .cht match for this game. Database holds {} files \u{00B7} newest {newest_words}.", grouped(read.files))
        };

        let browse_rows: Vec<Value> = if self.browse_open {
            game.cht_entries
                .iter()
                .enumerate()
                .map(|(i, cht)| {
                    let cht_writes = parse_writes(&cht.entry);
                    let sig = write_sig(&cht.entry, &cht_writes);
                    let added = all_resolved.iter().any(|r| {
                        game.list.as_ref().and_then(|l| l.entries().get(r.orig_index)).is_some_and(|e| write_sig(e, &r.writes) == sig)
                            || r.name.eq_ignore_ascii_case(&cht.description)
                    });
                    json!({
                        "index": i,
                        "indexKey": i.to_string(),
                        "name": cht.description,
                        "code": cht.display_code,
                        "badge": cht.badge,
                        "isLiveRam": cht.is_live_ram,
                        "added": added,
                        "statusWords": if added { "In list" } else { "Add" },
                        "focused": self.browse_cursor == i,
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        let mut help = Vec::new();
        if self.browse_open {
            eclipse_help_entry(&mut help, UiButton::Accept, None, "Add or switch");
            eclipse_help_entry(&mut help, UiButton::Back, None, "Close");
        } else if self.editor_open {
            if self.editor_typing {
                eclipse_help_entry(&mut help, UiButton::Accept, None, "Type");
                eclipse_help_entry(&mut help, UiButton::Back, None, "Delete");
                eclipse_help_entry(&mut help, UiButton::Screensaver, None, "Done");
            } else {
                eclipse_help_entry(&mut help, UiButton::Accept, None, if self.editor_field <= 1 { "Edit field" } else { "Confirm" });
                eclipse_help_entry(&mut help, UiButton::Back, None, "Cancel");
            }
        } else {
            let a_label = match self.zone {
                Zone::Master => if self.master_on { "Turn off all" } else { "Turn on all" },
                Zone::Filters => "Apply filter",
                Zone::Rows => focused_cheat.map_or("Toggle", |c| if c.on { "Turn off" } else { "Turn on" }),
                Zone::DbBrowse => "Browse file",
                Zone::DbUpdate => if downloading { "Stop" } else { "Update" },
                Zone::AddCard => "Add code",
            };
            eclipse_help_entry(&mut help, UiButton::Accept, None, a_label);
            if self.zone == Zone::Rows && focused_cheat.is_some() {
                eclipse_help_entry(&mut help, UiButton::Screensaver, None, "Edit or remove\u{2026}");
            }
            if cht_total > 0 {
                eclipse_help_entry(&mut help, UiButton::Search, None, "Browse file");
            }
            eclipse_help_entry(&mut help, UiButton::Options, None, "Add code");
            eclipse_help_entry(&mut help, UiButton::PageUp, Some(UiButton::PageDown), "Filter");
            eclipse_help_entry(&mut help, UiButton::Back, None, "Back");
        }

        let position_counter = if order.is_empty() {
            "0 of 0".to_owned()
        } else {
            format!("{} of {}", focused_pos.unwrap_or(0) + 1, order.len())
        };
        let action_lines = if self.browse_open || (self.editor_open && !self.editor_typing) {
            format!("closeOverlay {:?}\n", UiButton::Back)
        } else if self.editor_typing {
            format!("stopTyping {:?}\n", UiButton::Back)
        } else {
            format!(
                "{ACT_MASTER} {:?}\nprevFilter {:?}\nnextFilter {:?}\nprevGame {:?}\nnextGame {:?}\n{ACT_BROWSE} {:?}\n{ACT_ADD} {:?}\n{ACT_MORE} {:?}\n",
                UiButton::Menu,
                UiButton::PageUp,
                UiButton::PageDown,
                UiButton::First,
                UiButton::Last,
                UiButton::Search,
                UiButton::Options,
                UiButton::Screensaver,
            )
        };

        let focused_row_y = if let Some(pos) = focused_pos {
            let db_len = db_group.len();
            let mut y = 8i64;
            if db_len > 0 {
                y += 28;
                if pos < db_len {
                    y += pos as i64 * 79;
                } else {
                    y += db_len as i64 * 79 - 9 + 18 + 28 + (pos - db_len) as i64 * 79;
                }
            } else if !yours_group.is_empty() {
                y += 28 + pos as i64 * 79;
            }
            y
        } else {
            0
        };

        let db_card = json!({
            "title": "Local cheat database",
            "attributionTag": short_attribution(&read.attribution, read.exists),
            "meterPct": meter_pct,
            "copy": db_copy,
            "showBrowse": cht_total > 0,
            "browseLabel": format!("Browse {cht_total} in file\u{2026}"),
            "browseFocused": self.zone == Zone::DbBrowse,
            "updateLabel": if downloading { "Stop download" } else if read.exists { "Update database\u{2026}" } else { "Get database\u{2026}" },
            "updateFocused": self.zone == Zone::DbUpdate,
            "downloading": downloading,
            "statusNote": self.status_note,
        });
        let add_card = json!({
            "pill": "VIEW",
            "title": "Add a cheat code\u{2026}",
            "copy": "Type a Game Genie, Action Replay, or raw address:value code.",
            "focused": self.zone == Zone::AddCard,
        });
        let mut out = json!({
            "ready": true,
            "hasGame": true,
            "percentSuffix": "%",
            "focusedRowY": focused_row_y,
            "actionLines": action_lines,
            "trail": trail,
            "gameTitle": game.title,
            "systemBadge": game.system_badge,
            "cover": game.cover,
            "coreText": format!("Core: {}", game.core_name),
            "planet": game.planet,
            "listPathText": format!("List: {}", game.list_path_display),
            "dbMatchText": db_match_text,
            "dotSep": "\u{00B7}",
            "multiGame": read.games.len() > 1,
            "activeSummary": active_summary,
            "appliedSub": applied_sub,
            "masterHint": "MENU",
            "masterOn": self.master_on,
            "masterLabel": if self.master_on { "Cheats on" } else { "Cheats off" },
            "masterFocused": self.zone == Zone::Master,
            "filters": filters,
            "sortCaption": SORT_CAPTION,
            "groups": groups,
            "emptyStateWords": if total == 0 { "No cheats in this game's list yet." } else { "No cheats match this filter." },
            "inspector": inspector,
            "dbCard": db_card,
            "addCard": add_card,
            "positionCounter": position_counter,
        });
        let zone_code = match self.zone {
            Zone::Master => 0i32,
            Zone::Filters => 1,
            Zone::Rows => 2,
            Zone::DbBrowse => 3,
            Zone::DbUpdate => 4,
            Zone::AddCard => 5,
        };
        let obj = out.as_object_mut().expect("object");
        obj.insert("zoneCode".into(), Value::from(zone_code));
        obj.insert("focusedOrigIndex".into(), Value::from(focused_orig.map_or(-1i32, |i| i as i32)));
        obj.insert("focusedVisualIndex".into(), Value::from(focused_pos.map_or(-1i32, |i| i as i32)));
        obj.insert("browseCursor".into(), Value::from(self.browse_cursor as i32));
        obj.insert("overlayOpen".into(), Value::Bool(self.browse_open || self.editor_open));
        obj.insert("browseOpen".into(), Value::Bool(self.browse_open));
        obj.insert("browseTitle".into(), Value::String(game.db_file.as_ref().map(|(_, n, _)| format!("{n}.cht")).unwrap_or_else(|| "Database file".to_owned())));
        obj.insert("browseSubtitle".into(), Value::String(format!("{cht_total} codes in file")));
        obj.insert("browseRows".into(), Value::Array(browse_rows));
        obj.insert("editorOpen".into(), Value::Bool(self.editor_open));
        obj.insert("editorTitle".into(), Value::String(if self.editor_edit_orig.is_some() { "Edit cheat code" } else { "Add a cheat code" }.to_owned()));
        obj.insert("editorSubtitle".into(), Value::String(format!("Saved to {} for {}", game.list_path_display, game.console_name)));
        obj.insert("editorField".into(), Value::from(self.editor_field));
        obj.insert("editorTyping".into(), Value::Bool(self.editor_typing));
        obj.insert("editorDesc".into(), Value::String(self.editor_desc.clone()));
        obj.insert("editorCode".into(), Value::String(self.editor_code.clone()));
        obj.insert("editorError".into(), Value::String(self.editor_error.clone()));
        obj.insert("editorDescLabel".into(), Value::String("Description".to_owned()));
        obj.insert("editorDescHint".into(), Value::String("e.g. Infinite lives".to_owned()));
        obj.insert("editorCodeLabel".into(), Value::String("Cheat code".to_owned()));
        obj.insert("editorCodeHint".into(), Value::String("e.g. 7E00F2:04 or DD62-6DAD".to_owned()));
        obj.insert("editorSaveLabel".into(), Value::String(if self.editor_edit_orig.is_some() { "Save changes" } else { "Add to list" }.to_owned()));
        obj.insert("editorCancelLabel".into(), Value::String("Cancel".to_owned()));
        obj.insert("eraseIcon".into(), Value::String(ERASE_ICON.to_owned()));
        obj.insert("footerHelp".into(), Value::Array(help));
        out
    }
}

fn find_conflicts(focused: &ResolvedCheat, all: &[ResolvedCheat]) -> Vec<String> {
    let mut names = Vec::new();
    for other in all {
        if other.orig_index == focused.orig_index {
            continue;
        }
        let collides = focused.writes.iter().any(|fw| {
            other
                .writes
                .iter()
                .any(|ow| fw.address == ow.address && fw.space.eq_ignore_ascii_case(&ow.space))
        });
        if collides && !names.iter().any(|n| n == &other.name) {
            names.push(other.name.clone());
        }
    }
    names
}

fn resolve_game_cheats_name(game: Option<&EclipseGame>, orig_idx: usize) -> String {
    game.and_then(|g| g.list.as_ref())
        .and_then(|l| l.entries().get(orig_idx))
        .and_then(|e| e["Description"].as_str())
        .unwrap_or("This cheat")
        .to_owned()
}

fn grouped(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn format_epoch_date(secs: i64) -> String {
    // Civil calendar conversion from Unix seconds (Howard Hinnant algorithm) in UTC.
    let days = secs.div_euclid(86_400) + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = mp + if mp < 10 { 3 } else { -9 };
    if m <= 2 {
        y += 1;
    }
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let mon = MONTHS.get((m - 1).clamp(0, 11) as usize).copied().unwrap_or("Jan");
    format!("{d} {mon} {y}")
}

#[cfg(test)]
mod test_counters {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    static QUERIED_GAMES: AtomicUsize = AtomicUsize::new(0);
    static DECODED_CHTS: Mutex<Vec<String>> = Mutex::new(Vec::new());

    pub fn reset() {
        QUERIED_GAMES.store(0, Ordering::SeqCst);
        DECODED_CHTS.lock().unwrap().clear();
    }

    pub fn record_query(n: usize) {
        QUERIED_GAMES.fetch_add(n, Ordering::SeqCst);
    }

    pub fn record_cht_decode(path: &str) {
        DECODED_CHTS.lock().unwrap().push(path.to_owned());
    }

    pub fn snapshot() -> (usize, Vec<String>) {
        (QUERIED_GAMES.load(Ordering::SeqCst), DECODED_CHTS.lock().unwrap().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_game() -> EclipseGame {
        EclipseGame {
            path: "/roms/snes/Chrono Trigger (USA).sfc".to_owned(),
            title: "Chrono Trigger".to_owned(),
            console_id: "snes".to_owned(),
            console_name: "Super Nintendo".to_owned(),
            system_badge: "SNES".to_owned(),
            list_console: "snes".to_owned(),
            core_name: "Venus".to_owned(),
            planet: 1,
            cover: String::new(),
            codecs_summary: "snes.par \u{00B7} Pro Action Replay / snes.gg \u{00B7} Game Genie".to_owned(),
            default_space: "CpuBus".to_owned(),
            auto_codec: Some("snes.par".to_owned()),
            explicit_codec: Some("snes.gg".to_owned()),
            db_file: Some(("Nintendo - Super Nintendo Entertainment System".to_owned(), "Chrono Trigger (USA)".to_owned(), "/db/ct.cht".to_owned())),
            cht_entries: vec![
                ChtEntry {
                    description: "Infinite lives".to_owned(),
                    raw_code: "7E00F2:04".to_owned(),
                    badge: "Live RAM".to_owned(),
                    is_live_ram: true,
                    display_code: "7E00F2:04".to_owned(),
                    decoded: "WRAM $7E00F2 \u{2190} 0x04 / frame".to_owned(),
                    entry: json!({
                        "Kind": "RamPoke",
                        "Description": "Infinite lives",
                        "Enabled": true,
                        "Writes": [{ "Space": "CpuBus", "Address": "7E00F2", "Value": "04", "Width": 1, "Type": "Set" }]
                    }),
                },
                ChtEntry {
                    description: "Walk through walls".to_owned(),
                    raw_code: "7E0110:01".to_owned(),
                    badge: "Live RAM".to_owned(),
                    is_live_ram: true,
                    display_code: "7E0110:01".to_owned(),
                    decoded: "WRAM $7E0110 \u{2190} 0x01 / frame".to_owned(),
                    entry: json!({
                        "Kind": "RamPoke",
                        "Description": "Walk through walls",
                        "Enabled": false,
                        "Writes": [{ "Space": "CpuBus", "Address": "7E0110", "Value": "01", "Width": 1, "Type": "Set" }]
                    }),
                },
            ],
            list: Some(List {
                name: "Chrono Trigger".to_owned(),
                category: "cheats".to_owned(),
                file: json!({
                    "Cheats": [
                        { "Kind": "RamPoke", "Description": "Start disabled", "Enabled": false, "Writes": [{ "Space": "CpuBus", "Address": "7E00F0", "Value": "01", "Width": 1, "Type": "Set" }] },
                        { "Kind": "RamPoke", "Description": "Infinite lives", "Enabled": true, "Writes": [{ "Space": "CpuBus", "Address": "7E00F2", "Value": "04", "Width": 1, "Type": "Set" }] },
                        { "Kind": "RamPoke", "Description": "Fast dash", "Enabled": false, "Writes": [{ "Space": "CpuBus", "Address": "7E00A4", "Value": "02", "Width": 1, "Type": "Set" }] },
                        { "Kind": "RamPoke", "Description": "Double speed", "Enabled": true, "Writes": [{ "Space": "CpuBus", "Address": "7E00A4", "Value": "04", "Width": 1, "Type": "Set" }] }
                    ]
                }),
            }),
            cht_loaded: true,
            list_path_display: "cheats/Chrono Trigger.json".to_owned(),
        }
    }

    fn state_with_sample() -> State {
        let mut s = State::default();
        s.take_read(EclipseRead {
            directory: "/db".to_owned(),
            exists: true,
            attribution: "Cheat data comes from the libretro cheat database, licensed CC BY-SA 4.0.".to_owned(),
            files: 19412,
            newest: Some(1_763_600_000),
            codecs: CheatCodecs::default(),
            games: vec![sample_game()],
        });
        s
    }

    #[test]
    fn groups_and_sorting_put_active_first_and_separate_database_from_yours() {
        let s = state_with_sample();
        let v = s.projection(false);
        let groups = v["groups"].as_array().unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0]["title"], "From the database \u{00B7} 1");
        assert_eq!(groups[1]["title"], "Yours \u{00B7} 3");
        let yours_rows = groups[1]["rows"].as_array().unwrap();
        assert_eq!(yours_rows[0]["name"], "Double speed");
        assert_eq!(yours_rows[0]["on"], true);
        assert_eq!(yours_rows[1]["name"], "Fast dash");
        assert_eq!(yours_rows[2]["name"], "Start disabled");
    }

    #[test]
    fn inspector_omits_not_with_when_no_address_conflict_exists() {
        let mut s = state_with_sample();
        s.focus_cheat_key("1");
        let v = s.projection(false);
        let facts = v["inspector"]["facts"].as_array().unwrap();
        assert!(!facts.iter().any(|f| f["label"] == "Not with"));

        s.focus_cheat_key("2");
        let v2 = s.projection(false);
        let facts2 = v2["inspector"]["facts"].as_array().unwrap();
        let conflict = facts2.iter().find(|f| f["label"] == "Not with").expect("conflict row expected");
        assert_eq!(conflict["value"], "Double speed");
    }

    #[test]
    fn filters_step_through_all_database_yours_active() {
        let mut s = state_with_sample();
        s.step_filter(1);
        assert_eq!(s.active_filter, "database");
        assert_eq!(s.projection(false)["groups"].as_array().unwrap().len(), 1);
        s.step_filter(1);
        assert_eq!(s.active_filter, "yours");
        assert_eq!(s.projection(false)["groups"].as_array().unwrap()[0]["rows"].as_array().unwrap().len(), 3);
        s.step_filter(1);
        assert_eq!(s.active_filter, "active");
        assert_eq!(s.projection(false)["positionCounter"], "2 of 2");
    }

    #[test]
    fn parse_cht_raw_codes_extracts_descriptions_and_codes() {
        let parsed = parse_cht_raw_codes("cheats = 2\ncheat0_desc = \"Infinite lives\"\ncheat0_code = \"7E00F2:04\"\ncheat1_desc = \"All keys\"\ncheat1_code = \"DD62-6DAD\"\n");
        assert_eq!(parsed.get("infinite lives").map(String::as_str), Some("7E00F2:04"));
        assert_eq!(parsed.get("all keys").map(String::as_str), Some("DD62-6DAD"));
    }

    #[test]
    fn explicit_colon_fallback_accepts_hex_pair_and_rejects_garbage() {
        let game = sample_game();
        let ok = parse_explicit_colon_code(&game, "Test cheat", "7E00F2:04");
        assert!(ok.is_some());
        assert!(parse_explicit_colon_code(&game, "Bad", "ZZZZ:04").is_none());
        assert!(parse_explicit_colon_code(&game, "Bad", "notacode").is_none());
    }

    #[test]
    fn single_game_cheat_open_skips_unrelated_cht_files_and_overview_lazy_loads() {
        use emusen_client_fixture::cheats::{CHT, SNES};
        use emusen_client_fixture::{library, World};
        use emusen_platform_client::galaxia::consts::MODEL_APP_SETTINGS;
        use emusen_platform_client::Platform;

        let world = World::new("models-cheats-eclipse-single-game");
        let target_rom = world.dir("Roms/snes").join("Alpha Quest.sfc");
        let other_rom = world.dir("Roms/snes").join("Beta Voyage.sfc");
        std::fs::write(&target_rom, vec![0u8; 4096]).unwrap();
        std::fs::write(&other_rom, vec![0u8; 4096]).unwrap();

        let platform = Platform::load(library()).unwrap();
        let mut settings = platform.model_new(MODEL_APP_SETTINGS).unwrap();
        settings["RomDirectory"] = json!(world.dir("Roms").to_str().unwrap());
        settings["OpenEmuFallback"] = json!(false);
        platform.model_save(MODEL_APP_SETTINGS, None, "appsettings.json", &settings).unwrap();

        let status = emusen_qt_models_db_dir(&platform);
        let db_dir = Path::new(&status).join(SNES);
        std::fs::create_dir_all(&db_dir).unwrap();
        let target_cht = db_dir.join("Alpha Quest.cht");
        let other_cht = db_dir.join("Beta Voyage.cht");
        std::fs::write(&target_cht, CHT).unwrap();
        std::fs::write(&other_cht, CHT).unwrap();

        let source = crate::clients::source_for(library().to_str().unwrap(), "").unwrap();
        let found = crate::tools::cheats::read(&source).unwrap();
        assert!(found.exists);

        test_counters::reset();
        let target_str = target_rom.to_string_lossy().into_owned();
        let single = read_eclipse_for(&source, &found, Some(&target_str)).unwrap();
        let (queried, decoded) = test_counters::snapshot();
        assert_eq!(single.games.len(), 1);
        assert_eq!(single.games[0].path, target_str);
        assert!(single.games[0].cht_loaded);
        assert_eq!(single.games[0].cht_entries.len(), 2);
        assert_eq!(queried, 1);
        assert_eq!(decoded, vec![target_cht.to_string_lossy().into_owned()]);

        test_counters::reset();
        let overview = read_eclipse_for(&source, &found, None).unwrap();
        let (queried_all, decoded_none) = test_counters::snapshot();
        assert_eq!(overview.games.len(), 2);
        assert!(overview.games.iter().all(|g| !g.cht_loaded && g.cht_entries.is_empty()));
        assert_eq!(queried_all, 2);
        assert!(decoded_none.is_empty(), "overview mode must defer .cht decoding");

        let mut state = State::default();
        state.take_read(overview);
        state.ensure_current_cht(&source);
        let (_, decoded_one) = test_counters::snapshot();
        assert_eq!(decoded_one.len(), 1);
        assert_eq!(decoded_one[0], target_cht.to_string_lossy().into_owned());
    }

    fn emusen_qt_models_db_dir(platform: &Platform) -> String {
        let source = crate::clients::source_for(platform.path().to_str().unwrap(), "").unwrap();
        let found = crate::tools::cheats::read(&source).unwrap();
        found.directory
    }
}
