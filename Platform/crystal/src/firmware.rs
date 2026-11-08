//! Firmware as the runtime asks for it: what a game needs on its engine and the library does not hold, a request
//! installed, and every system's overview, over `emusen-firmware`'s store and the cores' own declarations, as the C#
//! `EmulatorSession.MissingFirmwareFor`, `FirmwareLibrary` and `FirmwareOverview` give them (EmuSen_Backend.md §29.2).

use emusen_core_host::Discovered;
use emusen_core_host::entry::{FirmwareRole, firmware_role, system_folder};
use emusen_firmware::Request;
use emusen_galaxia::tree::{Directory, Tree};
use emusen_native::json::Value;

use crate::catalog::{self, Catalog};
use crate::json::Json;
use crate::runtime::Runtime;

/// The console setting of which firmware a machine is given, read where it is located (EmuSen_Cores.md §3.12).
pub const PREFERENCE_KEY: &str = "FirmwarePreference";

/// `FirmwarePreference`: the player's own where found else the replacement, the replacement always, or the player's own only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Preference {
    #[default]
    Auto,
    Replacement,
    Own,
}

impl Preference {
    /// The row's choices, in the order a window lists them, and their labels.
    pub const CHOICES: [&str; 3] = ["auto", "replacement", "own"];
    pub const LABELS: [&str; 3] = ["Automatic", "Open replacement", "Own firmware"];

    /// A stored value; anything but the two others is auto.
    pub fn of(value: Option<&str>) -> Preference {
        match value {
            Some("replacement") => Preference::Replacement,
            Some("own") => Preference::Own,
            _ => Preference::Auto,
        }
    }
}

/// One firmware entry of core info, `CoreFirmware`.
#[derive(Clone, Debug)]
pub struct Entry {
    pub which: u32,
    pub name: String,
    pub label: String,
    pub size: i64,
    pub required: bool,
    pub parts: Vec<Vec<String>>,
    pub replacement: Option<(String, Option<String>)>,
    pub md5: Vec<String>,
    /// Core ABI 1.19's declared need, empty where the core gives none.
    pub need: String,
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array().unwrap_or(&[]).iter().filter_map(Value::as_str).map(str::to_string).collect()
}

impl Entry {
    pub fn read(f: &Value) -> Entry {
        let s = |k: &str| f.get(k).and_then(Value::as_str).unwrap_or("").to_string();
        Entry {
            which: f.get("which").and_then(Value::as_i64).unwrap_or(0) as u32,
            name: s("name"),
            label: s("label"),
            size: f.get("size").and_then(Value::as_i64).unwrap_or(0),
            required: matches!(f.get("required"), Some(Value::Bool(true))),
            parts: f.get("parts").and_then(Value::as_array).unwrap_or(&[]).iter().map(strs).collect(),
            replacement: f.get("replacement").filter(|r| matches!(r, Value::Object(_))).map(|r| {
                (r.get("effect").and_then(Value::as_str).unwrap_or("none").to_string(), r.get("cost").and_then(Value::as_str).map(str::to_string))
            }),
            md5: f.get("md5").map(strs).unwrap_or_default(),
            need: s("need"),
        }
    }

    /// Whether the core runs an open replacement in the file's place: exact or at a cost, not "none".
    pub fn has_replacement(&self) -> bool {
        self.replacement.as_ref().is_some_and(|(effect, _)| effect != "none")
    }

    /// The store's request for the entry, `CoreEngine.Request`'s fields the store reads.
    pub fn request(&self) -> Request {
        Request { file: self.name.clone(), size: self.size as i32, alternates: Vec::new(), parts: self.parts.clone(), md5: self.md5.clone() }
    }
}

/// A list of entries, the core's `firmware_for` answer or a system's `firmware`.
pub fn entries(list: &Value) -> Vec<Entry> {
    list.as_array().unwrap_or(&[]).iter().map(Entry::read).collect()
}

/// Each system of an info with its firmware entries.
pub fn systems_of(info: &Value) -> Vec<(String, String, bool, Vec<Entry>)> {
    info.get("systems")
        .and_then(Value::as_array)
        .unwrap_or(&[])
        .iter()
        .map(|s| {
            (
                s.get("id").and_then(Value::as_str).unwrap_or("").to_string(),
                s.get("name").and_then(Value::as_str).unwrap_or("").to_string(),
                matches!(s.get("development"), Some(Value::Bool(true))),
                s.get("firmware").map(entries).unwrap_or_default(),
            )
        })
        .collect()
}

/// False only when every system lists its firmware and none of it is required, `CanRequireFirmware`.
pub fn can_require(info: &Value) -> bool {
    let systems = systems_of(info);
    systems.is_empty() || systems.iter().any(|(_, _, _, f)| f.is_empty() || f.iter().any(|e| e.required))
}

/// Whether any of a console's engines declares, for one of its systems, a file with an open replacement.
pub fn replacement_offered(infos: &[&Value], systems: &[String]) -> bool {
    infos.iter().any(|info| systems_of(info).iter().any(|(id, _, _, list)| systems.contains(id) && list.iter().any(Entry::has_replacement)))
}

/// The firmware store, `FirmwareLibrary.Directory`.
pub fn store(tree: &Tree) -> String {
    tree.directory(Directory::Firmware).unwrap_or_default()
}

/// The folders searched, first to last, `FirmwareLibrary.Folders`.
pub fn folders(tree: &Tree) -> Vec<String> {
    let models = &emusen_galaxia::models::MODELS[emusen_galaxia::models::APP_SETTINGS_ID as usize];
    let (settings, _) = emusen_galaxia::config::load_or_new(tree, models, None, emusen_galaxia::models::APP_SETTINGS_FILE);
    let chosen = settings.get("FirmwareDirectory").and_then(|v| v.as_str()).map(str::to_string);
    let home = emusen_galaxia::tree::user_home();
    emusen_firmware::folders(chosen.as_deref(), &home, &store(tree)).0
}

/// One request as `firmware_needed` lists it.
pub fn request_json(j: &mut Json, core: &str, e: &Entry) {
    j.begin_object()
        .str("core", core)
        .str("chip", &e.label)
        .str("file", &e.name)
        .int("size", e.size as i32 as i64)
        .str("purpose", if e.required { "required" } else { "optional" })
        .strs("alternate_names", Vec::<String>::new())
        .key_array("parts");
    for form in &e.parts {
        j.begin_array();
        for p in form {
            j.str_value(p);
        }
        j.end_array();
    }
    j.end_array().bool("required", e.required);
    match &e.replacement {
        Some((effect, cost)) => {
            j.key_object("replacement").str("effect", effect).str_or_null("cost", cost.as_deref()).end_object();
        }
        None => {
            j.null("replacement");
        }
    }
    j.end_object();
}

/// The required firmware a file needs on its engine that no folder holds; none for a file that opens nowhere.
pub fn needed(rt: &Runtime, tree: &Tree, at: &str, engine: Option<&str>) -> (String, Vec<Entry>) {
    let discovery = rt.discovery();
    let catalog = Catalog::new(&discovery);
    let chosen = engine.map(str::to_string).or_else(|| {
        let (row, _) = crate::route::route(&catalog, &discovery, at);
        let system = crate::route::system_of_file(&catalog, &discovery, at);
        crate::settings::engine_for(tree, at, row.map(|r| catalog.rows[r].console()), system.as_deref(), emusen_galaxia::claims::root_for(tree, at).as_deref())
    });
    let _ = emusen_galaxia::consoles::load(tree);
    let Ok(Some(core)) = crate::route::v1_engine(&catalog, &discovery, at, chosen.as_deref()) else { return (String::new(), Vec::new()) };
    let Ok(lib) = core.open(&rt.trust()) else { return (String::new(), Vec::new()) };
    if !can_require(&core.sidecar.info_json) {
        return (core.sidecar.info.name.clone(), Vec::new());
    }
    let is_image = |e: &str| crate::route::is_image_extension(&catalog, &discovery, e);
    let Ok(set) = emusen_galaxia::disc_set::DiscSet::of(tree, at, &is_image) else { return (String::new(), Vec::new()) };
    let Ok(disc) = set.open(tree, 0, &is_image) else { return (String::new(), Vec::new()) };
    let Ok(text) = lib.firmware_for(disc.image.bytes()) else { return (String::new(), Vec::new()) };
    let list = emusen_native::json::parse(text.as_bytes()).map(|v| entries(&v)).unwrap_or_default();
    let searched = folders(tree);
    let missing = list
        .into_iter()
        .filter(|e| e.required)
        .filter_map(|e| match core.entry.as_ref().and_then(|entry| firmware_role(&entry.stem, &e.name).map(|role| (entry, role))) {
            Some((_, FirmwareRole::Carried)) => None,
            Some((entry, FirmwareRole::Console(system))) => {
                let own = system_folder(&entry.folder, &entry.stem).join(&e.name);
                let held = std::fs::read_dir(&own).is_ok_and(|mut d| d.any(|f| f.is_ok_and(|f| f.path().is_file())));
                let wanted = console_firmware(&discovery, system);
                let found = wanted.iter().any(|w| emusen_firmware::locate(&w.request(), &searched).is_some_and(|l| l.whole));
                (!held && !found).then(|| asked_for_folder(&e, &wanted))
            }
            None => emusen_firmware::locate(&e.request(), &searched).is_none().then_some(e),
        })
        .collect();
    (core.sidecar.info.name.clone(), missing)
}

/// The firmware the console's own engines declare for a system, each file once: what the player's store is searched
/// for when a hosted core's folder of console firmware is filled (EmuSen_LibretroHost.md §10s).
pub fn console_firmware(discovery: &emusen_core_host::Discovery, system: &str) -> Vec<Entry> {
    let mut found: Vec<Entry> = Vec::new();
    for c in discovery.found().iter().filter(|c| c.entry.is_none() && !c.sidecar.development) {
        for (id, _, _, list) in systems_of(&c.sidecar.info_json) {
            if id == system {
                found.extend(list.into_iter().filter(|e| e.size > 0 && !found.iter().any(|f| f.name == e.name)).collect::<Vec<_>>());
            }
        }
    }
    found
}

/// A folder of console firmware none of whose files the store holds, asked for as one file the player can give: the
/// first declared name and size, found by the content of any of them, under the folder's own words.
fn asked_for_folder(folder: &Entry, wanted: &[Entry]) -> Entry {
    let Some(first) = wanted.first() else { return folder.clone() };
    let mut md5: Vec<String> = Vec::new();
    for w in wanted {
        md5.extend(w.md5.iter().filter(|m| !md5.contains(m)).cloned().collect::<Vec<_>>());
    }
    Entry { which: folder.which, name: first.name.clone(), label: folder.label.clone(), size: first.size, required: true, parts: vec![vec![first.name.clone()]], replacement: None, md5, need: folder.need.clone() }
}

/// For a hosted libretro core: each folder of console firmware in its system folder given links, read-only, to the
/// store's files its console's engines declare, by their declared names; a file already there is left as it is. The
/// names linked.
pub fn link_player_firmware(core: &Discovered, discovery: &emusen_core_host::Discovery, tree: &Tree) -> Vec<String> {
    let Some(entry) = &core.entry else { return Vec::new() };
    let searched = folders(tree);
    let mut linked = Vec::new();
    for (_, path, role) in emusen_core_host::entry::FIRMWARE_ROLES.iter().filter(|(stem, _, _)| *stem == entry.stem) {
        let FirmwareRole::Console(system) = role else { continue };
        let own = system_folder(&entry.folder, &entry.stem).join(path);
        for w in console_firmware(discovery, system) {
            let Some(at) = emusen_firmware::locate(&w.request(), &searched).filter(|l| l.whole) else { continue };
            let source = std::path::Path::new(&at.folder).join(&at.names[0]);
            let link = own.join(&w.name);
            match std::fs::symlink_metadata(&link) {
                Ok(m) if !m.file_type().is_symlink() => continue,
                Ok(_) if std::fs::read_link(&link).is_ok_and(|t| t == source) => continue,
                Ok(_) => {
                    let _ = std::fs::remove_file(&link);
                }
                Err(_) => {}
            }
            #[cfg(unix)]
            if std::fs::create_dir_all(&own).is_ok() && std::os::unix::fs::symlink(&source, &link).is_ok() {
                linked.push(w.name.clone());
            }
        }
    }
    linked
}

pub fn needed_json(rt: &Runtime, at: &str, engine: Option<&str>) -> String {
    let tree = emusen_galaxia::tree::tree();
    let (core, missing) = needed(rt, &tree, at, engine);
    let mut j = Json::new();
    j.begin_array();
    for e in &missing {
        request_json(&mut j, &core, e);
    }
    j.end_array();
    j.finish()
}

/// A system's firmware as the player has it, `firmware_status`: each file its engines declare, once, with whether it is
/// needed and found by `locate` as the overview finds it; the native engines' declarations, else the hosted cores'. None
/// for a system neither the table nor an engine names (EmuSen_Cores.md §3.11).
pub fn status_json(rt: &Runtime, system: &str) -> Option<String> {
    let discovery = rt.discovery();
    let declared = |native: bool| -> Vec<(Entry, bool)> {
        let mut found: Vec<(Entry, bool)> = Vec::new();
        for core in discovery.found().iter().filter(|c| c.entry.is_none() == native) {
            for s in core.sidecar.info_json.get("systems").and_then(Value::as_array).unwrap_or(&[]) {
                if s.get("id").and_then(Value::as_str) != Some(system) {
                    continue;
                }
                for f in s.get("firmware").and_then(Value::as_array).unwrap_or(&[]) {
                    let (e, needed) = (Entry::read(f), emusen_core_host::listing::firmware_needed(f));
                    match found.iter_mut().find(|(have, _)| have.name == e.name) {
                        Some(have) => have.1 |= needed,
                        None if e.size > 0 => found.push((e, needed)),
                        None => {}
                    }
                }
            }
        }
        found
    };
    let lists = |native: bool| discovery.found().iter().any(|c| c.entry.is_none() == native && systems_of(&c.sidecar.info_json).iter().any(|(id, _, _, _)| id == system));
    if emusen_systems::find(system).is_none() && !lists(true) && !lists(false) {
        return None;
    }
    let wanted = if lists(true) { declared(true) } else { declared(false) };
    let searched = folders(&emusen_galaxia::tree::tree());
    let mut j = Json::new();
    j.begin_object().str("system", system).key_array("firmware");
    for (e, needed) in &wanted {
        let status = match emusen_firmware::locate(&e.request(), &searched) {
            Some(_) => "present",
            None if *needed => "missing",
            None => "not_needed",
        };
        j.begin_object().str("label", &e.label).str("file", &e.name).bool("required", *needed).str("status", status).str("need", &need(e)).end_object();
    }
    j.end_array().end_object();
    Some(j.finish())
}

/// The need a declared file meets, `firmware_status`'s `need` (frontend API 1.11): the id its engine declares (core ABI
/// 1.19), so that alternatives share it whatever their sizes, else one of its own from its name (EmuSen_Cores.md §3.13).
pub fn need(e: &Entry) -> String {
    if e.need.is_empty() { format!("file:{}", e.name) } else { e.need.clone() }
}

/// A discovered engine a player is offered and whose library is there to run.
fn shipped<'d>(core: Option<&'d Discovered>) -> Option<&'d Discovered> {
    core.filter(|c| !c.sidecar.development && c.sidecar.library_path.is_file())
}

/// `{"folder","folders","systems":[…]}`, `FirmwareOverview`.
pub fn overview_json(rt: &Runtime) -> String {
    let tree = emusen_galaxia::tree::tree();
    let stored = emusen_galaxia::consoles::load(&tree);
    let discovery = rt.discovery();
    let catalog = Catalog::new(&discovery);
    let searched = folders(&tree);
    let store = store(&tree);
    let role = |folder: Option<&str>| folder.map(|f| if f == store { "EmuSen's folder" } else { "your folder" });
    let item = |j: &mut Json, e: &Entry| {
        let request = e.request();
        let found = emusen_firmware::locate(&request, &searched);
        let wrong = if found.is_none() { emusen_firmware::wrong_size_in(&request, &searched) } else { None };
        let (effect, cost) = match &e.replacement {
            Some((effect, cost)) => (Some(effect.as_str()), cost.as_deref()),
            None => (None, None),
        };
        j.begin_object()
            .str("label", &e.label)
            .str("file", &e.name)
            .int("size", e.size as i32 as i64)
            .bool("present", found.is_some())
            .bool("wrong_size", wrong.is_some())
            .str_or_null("effect", effect)
            .str_or_null("cost", cost)
            .str_or_null("found_in", role(found.as_ref().map(|f| f.folder.as_str())))
            .str_or_null("wrong_size_in", role(wrong.as_deref()))
            .end_object();
    };
    let mut j = Json::new();
    j.begin_object().str("folder", &store).strs("folders", &searched).key_array("systems");
    let mut names: Vec<String> = Vec::new();
    let shelves = catalog.shelves();
    for r in catalog.consoles_in_release_order() {
        let row = &catalog.rows[r];
        if row.discovered {
            continue;
        }
        let name = shelves.iter().filter(|s| s.row == r).map(|s| s.system.esde_full_name.as_str()).collect::<Vec<_>>().join(" and ");
        let console = row.console();
        let chosen = crate::settings::current(stored.value(console, catalog::ENGINE_KEY).map(str::to_string)).or_else(|| catalog::sole_engine(console).map(str::to_string));
        j.begin_object().str("name", &name);
        match shipped(chosen.as_deref().and_then(|e| discovery.by_engine_name(e))) {
            Some(v1) => {
                j.str("engine", v1.engine_name()).null("note").key_array("items");
                let ids = catalog::system_ids_for(console);
                let mut seen: Vec<String> = Vec::new();
                for (_, _, _, list) in systems_of(&v1.sidecar.info_json).into_iter().filter(|(id, _, _, _)| ids.contains(&id.as_str())) {
                    for e in list {
                        if seen.contains(&e.name) {
                            continue;
                        }
                        seen.push(e.name.clone());
                        item(&mut j, &e);
                    }
                }
                j.end_array();
            }
            None => {
                j.str("engine", &row.display_name).null("note").key_array("items").end_array();
            }
        }
        j.end_object();
        names.push(name);
    }
    for core in discovery.found() {
        if shipped(Some(core)).is_none() {
            continue;
        }
        for (id, sname, development, list) in systems_of(&core.sidecar.info_json) {
            if development || catalog::hand_console_of(&id).is_some() || names.contains(&sname) {
                continue;
            }
            let note = emusen_systems::find(&id).and_then(|s| s.firmware_note.as_deref());
            j.begin_object().str("name", &sname).str("engine", core.engine_name()).str_or_null("note", note).key_array("items");
            for e in &list {
                item(&mut j, e);
            }
            j.end_array().end_object();
            names.push(sname);
        }
    }
    j.end_array().end_object();
    j.finish()
}

/// A whole number as .NET's `N0` writes it in the invariant culture: thousands parted by commas.
pub fn n0(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    if value < 0 { format!("-{out}") } else { out }
}

/// A refusal: its status and words.
pub type Refused = (i32, String);

/// A request as `firmware_needed` lists it, read back and installed from `source` into the store,
/// `ManagedRuntime.InstallFirmware`; a failure's line said through `say`.
pub fn install(tree: &Tree, request: &str, source: &str, say: &dyn Fn(String)) -> Result<(), Refused> {
    use emusen_session_service::abi::status as svc;
    let bad = |w: String| (svc::BAD_ARGUMENT, w);
    let doc = if request.is_empty() { emusen_dotnet::json::parse("{}") } else { emusen_dotnet::json::parse(request) }.map_err(|e| bad(format!("the argument is not JSON: {e}")))?;
    let size = match doc.get("size") {
        Some(emusen_dotnet::json::Value::Number(n)) => n.parse::<i32>().ok(),
        _ => None,
    }
    .ok_or_else(|| bad("the request's \"size\" must be given".to_string()))?;
    let text = |k: &str| doc.get(k).and_then(|v| v.as_str()).map(str::to_string).ok_or_else(|| bad(format!("\"{k}\" must be given as a string")));
    let (_core, _chip, file) = (text("core")?, text("chip")?, text("file")?);
    let alternates = match doc.get("alternate_names") {
        Some(emusen_dotnet::json::Value::Array(items)) => items.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect(),
        _ => Vec::new(),
    };
    if !emusen_galaxia::files::file_exists(source) {
        return Err((crate::abi::status::IO, format!("{source} does not exist")));
    }
    let length = std::fs::metadata(source).map(|m| m.len() as i64).unwrap_or(0);
    if length != size as i64 {
        let name = emusen_galaxia::dotnet_path::file_name(emusen_galaxia::dotnet_path::Style::HOST, source);
        return Err(bad(format!("{name} is {} bytes, and {file} is {}", n0(length), n0(size as i64))));
    }
    let store = store(tree);
    let wanted = Request { file: file.clone(), size, alternates, parts: Vec::new(), md5: Vec::new() };
    let written = match emusen_firmware::install(&wanted, source, &store) {
        Ok(done) => done,
        Err(words) => {
            say(format!("[Firmware] Could not install {file}: {words}"));
            false
        }
    };
    if written { Ok(()) } else { Err((crate::abi::status::IO, format!("{file} could not be written to {store}"))) }
}

#[cfg(test)]
mod need_tests {
    use super::*;

    fn entry(json: &str) -> Entry {
        Entry::read(&emusen_native::json::parse(json.as_bytes()).unwrap())
    }

    #[test]
    fn alternatives_share_the_need_their_engine_declares_whatever_their_sizes_and_the_rest_meet_their_own() {
        let small = entry(r#"{"which":2,"name":"a.bin","label":"A","size":8,"required":false,"need":"bios"}"#);
        let large = entry(r#"{"which":3,"name":"b.bin","label":"B","size":16,"required":false,"need":"bios"}"#);
        let chip = entry(r#"{"which":2,"name":"dsp1.rom","label":"DSP-1","size":8,"required":false}"#);
        let other = entry(r#"{"which":2,"name":"dsp2.rom","label":"DSP-2","size":8,"required":false}"#);
        assert_eq!(need(&small), need(&large), "two sizes, one need");
        assert_ne!(need(&chip), need(&other), "the same size and slot, and still two needs");
        assert_ne!(need(&chip), need(&small));
    }
}
