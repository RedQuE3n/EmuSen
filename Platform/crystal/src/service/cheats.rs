//! The cheat group: the C# `SessionService.CheatOp` and `CheatCodecs` over the session's cheat registry, its system's
//! codecs, the player's cheat lists and cheat database, and the provider with no game that answers session 0's ops a
//! game is not needed for (EmuSen_DianaSh.md §9.2, §13.11; EmuSen_Backend.md §14.12).

use emusen_cheats::registry::CheatRegistry;
use emusen_cheats::{cht, database::Database, prune};
use emusen_dotnet::json::Value;
use emusen_galaxia::models::{CHEAT_FILE_ID, CHEATS_CATEGORY, MODELS};
use emusen_galaxia::tree::{self as galaxia_tree, Directory, Tree};
use emusen_session_service::abi::{cheat as op, status};
use emusen_systems::cheats::{self as codecs, Codec, Kind, Write, WriteType};

use super::args::{self, Raw};
use super::machine::dm;
use super::{Answer, Call, Provider, READ_ONLY_WORDS, int32, no_such_id, not_supported, refused};
use crate::json::Json;
use crate::runtime::Runtime;

/// `CheatDatabaseInstaller.LibretroCheatsUrl`, what RetroArch's own "Update Cheats" pulls.
const LIBRETRO_CHEATS_URL: &str = "https://buildbot.libretro.com/assets/frontend/cheats.zip";

/// `CheatDatabaseInstaller.Attribution`, shown beside the database as its licence asks.
const ATTRIBUTION: &str = "Cheat data comes from the libretro cheat database (github.com/libretro/libretro-database),\nlicensed CC BY-SA 4.0. The codes themselves were collected from community sources,\nsubstantially GameHacking.org - credit belongs to their original authors.\nEmuSen ships none of this data; it is downloaded to your machine at your request.";

/// `CheatImport.DefaultSpaceName`.
const DEFAULT_SPACE: &str = "CpuBus";

/// The words of the `ArgumentOutOfRangeException` a codec threw indexing an empty list.
const RANGE_WORDS: &str = "Index was out of range. Must be non-negative and less than the size of the collection. (Parameter 'index')";

/// The ops a provider with no game answers, `GameFreeCheatOps`.
pub fn game_free(o: u32) -> bool {
    matches!(o, op::DB_SYSTEMS | op::DB_GAMES | op::DB_FIND | op::DB_STATUS | op::DB_PARSE | op::DB_PRUNE | op::DB_UPDATE | op::CHT_PARSE | op::FILES | op::FILE_READ)
}

/// What one cheat call reaches: the runtime, the provider, the registry (a fresh one where there is no game), the
/// system's codecs, the session's spaces, and whether the table is a read-only context's.
struct Cheats<'a> {
    rt: &'a Runtime,
    provider: &'a Provider,
    registry: &'a mut CheatRegistry,
    auto: Option<&'static Codec>,
    explicit: Option<&'static Codec>,
    spaces: Vec<String>,
    read_only: bool,
}

/// A system's codecs, by its pack's pair, `SystemCodecs`.
fn codecs_of(system: Option<&str>) -> (Option<&'static Codec>, Option<&'static Codec>) {
    let Some(pair) = system.and_then(|s| codecs::PAIRS.iter().find(|(id, _, _)| *id == s)) else { return (None, None) };
    (pair.1.and_then(codecs::codec), pair.2.and_then(codecs::codec))
}

/// The system a session's codecs are its pack's: the machine's, else its engine's first.
fn session_codecs(e: &crate::engine::Engine) -> (Option<&'static Codec>, Option<&'static Codec>) {
    let system = if e.machine.is_some() { Some(e.mi.system.clone()) } else { crate::info::systems(&e.info).first().map(|s| s.id.clone()) };
    codecs_of(system.as_deref())
}

fn kind_word(kind: Kind) -> &'static str {
    if kind == Kind::RomPatch { "rom" } else { "ram" }
}

fn type_word(kind: WriteType) -> &'static str {
    match kind {
        WriteType::Set => "Set",
        WriteType::Increase => "Increase",
        WriteType::Decrease => "Decrease",
        WriteType::IfEqual => "IfEqual",
        WriteType::IfNotEqual => "IfNotEqual",
        WriteType::IfLess => "IfLess",
        WriteType::IfGreater => "IfGreater",
    }
}

/// `Writes(w, writes)`.
fn writes_json(j: &mut Json, writes: &[Write]) {
    j.begin_array();
    for x in writes {
        j.begin_object().str("space", &x.space).u64("address", x.address as u32 as u64).u64("value", x.value as u64).int("width", x.width as i64).str("type", type_word(x.kind));
        match x.bit_position {
            Some(b) => j.int("bit", b as i64),
            None => j.null("bit"),
        };
        j.bool("big_endian", x.big_endian).int("repeat", x.repeat_count as i64).int("repeat_address", x.repeat_add_address as i64).u64("repeat_value", x.repeat_add_value as u64).end_object();
    }
    j.end_array();
}

/// `ParseWrite`: a write as JSON names it, checked as the C# checks it.
fn parse_write(x: &Value) -> Answer<Write> {
    let mut kind = WriteType::Set;
    if let Some(name) = args::optional_string(x, "type")? {
        kind = emusen_cheats::registry::parse_write_type(Some(&name)).ok_or_else(|| refused(status::BAD_ARGUMENT, format!("no cheat write type '{name}'")))?;
    }
    let value = args::int(x, "value")?;
    let repeat_value = args::optional_int(x, "repeat_value")?.unwrap_or(0);
    if !(0..=u32::MAX as i64).contains(&value) || !(0..=u32::MAX as i64).contains(&repeat_value) {
        return Err(refused(status::BAD_ARGUMENT, "a write's value is 32 bits"));
    }
    let bit = args::optional_int(x, "bit")?;
    let space = args::optional_string(x, "space")?.unwrap_or_default();
    let address = args::address(x, "address")?;
    let width = int32(args::optional_int(x, "width")?.unwrap_or(1), "width")?;
    let bit_position = bit.map(|b| int32(b, "bit")).transpose()?;
    let big_endian = args::optional_bool(x, "big_endian")?.unwrap_or(false);
    let repeat_count = int32(args::optional_int(x, "repeat")?.unwrap_or(1), "repeat")?;
    let repeat_add_address = int32(args::optional_int(x, "repeat_address")?.unwrap_or(0), "repeat_address")?;
    Ok(Write { space, address, value: value as u32, width, kind, bit_position, big_endian, repeat_count, repeat_add_address, repeat_add_value: repeat_value as u32 })
}

/// `CompareOf`: a compare byte, none where none is given.
fn compare_of(a: &Value) -> Answer<Option<u8>> {
    match args::optional_int(a, "compare")? {
        Some(c) if !(0..=0xFF).contains(&c) => Err(refused(status::BAD_ARGUMENT, "a compare is one byte")),
        c => Ok(c.map(|c| c as u8)),
    }
}

fn space_of(spaces: &[String], name: Option<&str>) -> Answer<String> {
    let Some(name) = name else { return Err(refused(status::BAD_ARGUMENT, "a memory space must be named")) };
    spaces
        .iter()
        .find(|s| emusen_dotnet::text::equals_ignore_case(s, name))
        .cloned()
        .ok_or_else(|| refused(status::NO_SUCH_SPACE, format!("No memory space named '{name}'. Available: {}", spaces.join(", "))))
}

/// A list's file of the `cheats` category, by the name the player typed.
fn file_name(name: &str) -> String {
    format!("{name}.json")
}

/// `ValidName`.
fn valid_name(name: String) -> Answer<String> {
    if emusen_galaxia::cheats::is_valid_name(emusen_galaxia::dotnet_path::Style::HOST, Some(&name)) { Ok(name) } else { Err(refused(status::BAD_ARGUMENT, format!("'{name}' is not a usable file name."))) }
}

/// A registry's list as the bytes `JsonSerializer.SerializeToUtf8Bytes(file, ConfigJson.Options)` writes.
fn file_text(cheats: &CheatRegistry) -> Answer<String> {
    let mut canonical = String::new();
    emusen_dotnet::json::write_compact(&mut canonical, &cheats.to_file());
    let model = &MODELS[CHEAT_FILE_ID as usize];
    match emusen_galaxia::config::bind_text(model, &canonical, false).map_err(|w| refused(status::FAULT, w))? {
        Some(bound) => emusen_galaxia::model::format(model.schema, &bound).map_err(|w| refused(status::FAULT, w)),
        None => Ok("null".to_string()),
    }
}

/// `SettingFolders.Cheats`: the player's database folder, else the data folder's `Cheats`.
fn database_folder(tree: &Tree) -> String {
    let model = &MODELS[emusen_galaxia::models::APP_SETTINGS_ID as usize];
    let (settings, _) = emusen_galaxia::config::load_or_new(tree, model, None, emusen_galaxia::models::APP_SETTINGS_FILE);
    match settings.get("CheatDatabaseDirectory").and_then(|v| v.as_str()) {
        Some(named) if !emusen_dotnet::text::is_blank(Some(named)) => named.to_string(),
        _ => tree.directory(Directory::Cheats).unwrap_or_default(),
    }
}

/// `Parsed(parsed, game)`: a `.cht` file's cheats as CHT_PARSE and DB_PARSE answer them.
fn parsed_json(parsed: &cht::Parsed, game: Option<&str>) -> Vec<u8> {
    let mut j = Json::new();
    j.begin_object().str_or_null("game", game).key_array("cheats");
    for c in &parsed.cheats {
        j.begin_object().str("kind", kind_word(c.kind)).str("description", &c.description).bool("enabled", c.enabled);
        j.key("writes");
        writes_json(&mut j, &c.writes);
        j.end_object();
    }
    j.end_array().int("skipped", parsed.skipped as i64).end_object();
    j.finish().into_bytes()
}

/// A `.cht` text read with the session's codecs, the C# `ChtFile.Parse`; a codec's range fault escapes as a fault.
fn read_cht(cc: &Cheats, text: &str) -> Answer<cht::Parsed> {
    let space = cc.auto.and_then(codecs::space).unwrap_or(DEFAULT_SPACE);
    cht::parse(text, cc.auto.map(|c| c.id), space, cc.explicit.map(|c| c.id)).map_err(|_| refused(status::FAULT, RANGE_WORDS))
}

fn codec_error(e: &codecs::Error) -> String {
    match e {
        codecs::Error::Format(words) => words.clone(),
        codecs::Error::Range => RANGE_WORDS.to_string(),
    }
}

/// `Decode`: a code through the codec asked for, checked first unless asked not to be.
fn decode(cc: &Cheats, code: &str, which: &str, unchecked: bool) -> Answer<Vec<u8>> {
    let codec = match which {
        "auto" => cc.auto.ok_or_else(|| not_supported("No cheat codec is registered for this target."))?,
        "explicit" => cc.explicit.ok_or_else(|| not_supported("No Game Genie-style cheat codec is registered for this target."))?,
        other => return Err(not_supported(format!("no codec '{other}'"))),
    };
    let answer = codecs::decode(codec.id, code).ok_or_else(|| not_supported(format!("no codec '{which}'")))?;
    if !unchecked && !answer.can_decode {
        return Err(refused(status::PARSE, format!("'{code}' is not a {} code.", codec.name)));
    }
    let parse = |e: &codecs::Error| refused(status::PARSE, codec_error(e));
    let fault = |e: &codecs::Error| refused(status::FAULT, codec_error(e));
    let (mut decoded, mut compare, writes) = ((0, 0u8), None, None::<Vec<Write>>);
    let writes = if unchecked {
        let writes = answer.writes.as_ref().map_err(parse)?.clone();
        if writes.is_none() {
            decoded = *answer.decode.as_ref().map_err(parse)?;
            compare = *answer.compare.as_ref().map_err(parse)?;
        }
        writes
    } else {
        decoded = *answer.decode.as_ref().map_err(parse)?;
        compare = *answer.compare.as_ref().map_err(fault)?;
        let _ = writes;
        answer.writes.as_ref().map_err(fault)?.clone()
    };
    let mut j = Json::new();
    j.begin_object().str("codec", codec.name).str("kind", kind_word(codec.kind)).str_or_null("space", codecs::space(codec)).u64("address", decoded.0 as u32 as u64).int("value", decoded.1 as i64);
    match compare {
        Some(c) => j.int("compare", c as i64),
        None => j.null("compare"),
    };
    match &writes {
        None => {
            j.null("writes");
        }
        Some(list) => {
            j.key("writes");
            writes_json(&mut j, list);
        }
    }
    j.end_object();
    Ok(j.finish().into_bytes())
}

fn entries_json(db: &Database, places: &[usize]) -> Vec<u8> {
    let mut j = Json::new();
    j.begin_array();
    for &i in places {
        let e = &db.entries[i];
        j.begin_object().str("system", &e.system).str("game", &e.game).str("path", &e.path).end_object();
    }
    j.end_array();
    j.finish().into_bytes()
}

/// `Prune(apply)`: the plan, applied where asked and where it can be.
fn prune_json(cc: &Cheats, apply: bool) -> Vec<u8> {
    let db = Database::scan(&database_folder(&galaxia_tree::tree()));
    let systems = cc.rt.supported_cheat_systems();
    let plan = prune::plan(&db, &systems);
    let (pruned, failed) = if apply && plan.reason.is_none() { prune::apply(&db, &plan.removing) } else { (0, Vec::new()) };
    let mut j = Json::new();
    j.begin_object().bool("can_apply", plan.reason.is_none()).str_or_null("reason", plan.reason.as_deref()).key_array("removing");
    for r in &plan.removing {
        j.begin_object().str("system", &r.system).int("files", r.files as i64).int("bytes", r.bytes as i64).end_object();
    }
    j.end_array().strs("keeping", &plan.keeping).int("pruned", pruned as i64).strs("failed", &failed).end_object();
    j.finish().into_bytes()
}

/// The database downloaded and installed, `CheatDatabaseInstaller.Fetch` and `Install`.
fn update_json(cc: &Cheats) -> Answer<Vec<u8>> {
    let directory = database_folder(&galaxia_tree::tree());
    let fetch = cc.rt.fetch.as_ref().ok_or_else(|| refused(status::FAULT, "this runtime downloads nothing"))?;
    let zip = fetch(LIBRETRO_CHEATS_URL).map_err(|w| refused(status::FAULT, w))?;
    let installed = emusen_cheats::install::install(&zip, &directory).map_err(|r| match r {
        emusen_cheats::install::Refusal::Unsupported(w) => refused(status::FAULT, w),
        emusen_cheats::install::Refusal::Io(e) => refused(status::FAULT, e.to_string()),
    })?;
    let mut j = Json::new();
    j.begin_object().int("installed", installed.installed as i64).str("directory", &directory).str("attribution", ATTRIBUTION).end_object();
    Ok(j.finish().into_bytes())
}

/// `Pending(op, apply)`: a change whose record a short buffer cannot take is kept until one can.
fn pending(provider: &Provider, o: u32, apply: impl FnOnce() -> Answer<Vec<u8>>) -> Answer<Vec<u8>> {
    let mut kept = provider.pending.lock().unwrap_or_else(|p| p.into_inner());
    if let Some((_, record)) = kept.iter().find(|(k, _)| *k == o) {
        return Ok(record.clone());
    }
    let record = apply()?;
    kept.push((o, record.clone()));
    Ok(record)
}

/// One op: a record to copy out, or a value, `CheatOp`.
fn run(cc: &mut Cheats, o: u32, bytes: &[u8]) -> Answer<(Option<Vec<u8>>, i64)> {
    if o > op::DB_UPDATE {
        return Err(not_supported(format!("no cheat op {o}")));
    }
    let a = args::parse(bytes)?;
    let changes = matches!(o, op::ADD | op::POKE | op::ROM_PATCH | op::ENABLE | op::DISABLE | op::REMOVE | op::CLEAR | op::IMPORT)
        || (o == op::MASTER && args::optional_bool(&a, "enabled")?.is_some())
        || (o == op::DB_PRUNE && args::optional_bool(&a, "apply")? == Some(true))
        || o == op::DB_UPDATE;
    if cc.read_only && changes {
        return Err(refused(status::READ_ONLY, READ_ONLY_WORDS));
    }
    let record = |bytes: Vec<u8>| Ok((Some(bytes), 0));
    match o {
        op::LIST => {
            let mut j = Json::new();
            j.begin_object().bool("master", cc.registry.master_enabled()).key_array("cheats");
            for c in cc.registry.cheats() {
                j.begin_object().int("id", c.id as i64).str("kind", kind_word(c.kind)).str("description", &c.description).bool("enabled", c.enabled);
                match c.compare {
                    Some(v) => j.int("compare", v as i64),
                    None => j.null("compare"),
                };
                j.key("writes");
                writes_json(&mut j, &c.writes);
                j.end_object();
            }
            j.end_array().end_object();
            record(j.finish().into_bytes())
        }
        op::ADD => {
            let kind = match args::string(&a, "kind")?.as_str() {
                "ram" => Kind::RamPoke,
                "rom" => Kind::RomPatch,
                other => return Err(not_supported(format!("no cheat kind '{other}'"))),
            };
            let Some(Value::Array(items)) = a.get("writes") else { return Err(refused(status::BAD_ARGUMENT, "a cheat has an array of writes")) };
            let list = items.iter().map(parse_write).collect::<Answer<Vec<Write>>>()?;
            let compare = compare_of(&a)?;
            let description = args::optional_string(&a, "description")?.unwrap_or_default();
            let enabled = args::optional_bool(&a, "enabled")?.unwrap_or(true);
            cc.registry.add(kind, list, compare, &description, enabled).map(|id| (None, id as i64)).map_err(|w| refused(status::BAD_ARGUMENT, w))
        }
        op::POKE => {
            let value = args::int(&a, "value")?;
            if !(0..=0xFF).contains(&value) {
                return Err(refused(status::BAD_ARGUMENT, "a poke's value is one byte"));
            }
            let name = args::string(&a, "space")?;
            let space = space_of(&cc.spaces, Some(&name))?;
            let address = args::address(&a, "address")?;
            let description = args::optional_string(&a, "description")?.unwrap_or_default();
            let enabled = args::optional_bool(&a, "enabled")?.unwrap_or(true);
            cc.registry.add_ram_poke(&space, address, value as u8, &description, enabled).map(|id| (None, id as i64)).map_err(|w| refused(status::FAULT, w))
        }
        op::ROM_PATCH => {
            let value = args::int(&a, "value")?;
            if !(0..=0xFF).contains(&value) {
                return Err(refused(status::BAD_ARGUMENT, "a patch's value is one byte"));
            }
            let address = args::address(&a, "address")?;
            let compare = compare_of(&a)?;
            let description = args::optional_string(&a, "description")?.unwrap_or_default();
            let enabled = args::optional_bool(&a, "enabled")?.unwrap_or(true);
            cc.registry.add_rom_patch(address, value as u8, compare, &description, enabled).map(|id| (None, id as i64)).map_err(|w| refused(status::FAULT, w))
        }
        op::ENABLE | op::DISABLE => {
            let id = args::int(&a, "id")?;
            if !cc.registry.set_enabled(int32(id, "id")?, o == op::ENABLE) {
                return Err(no_such_id("cheat", id));
            }
            Ok((None, 0))
        }
        op::REMOVE => {
            let id = args::int(&a, "id")?;
            if !cc.registry.remove(int32(id, "id")?) {
                return Err(no_such_id("cheat", id));
            }
            Ok((None, 0))
        }
        op::CLEAR => {
            cc.registry.clear();
            Ok((None, 0))
        }
        op::MASTER => match args::optional_bool(&a, "enabled")? {
            Some(on) => {
                cc.registry.set_master_enabled(on);
                Ok((None, 0))
            }
            None => {
                let mut j = Json::new();
                j.begin_object().bool("enabled", cc.registry.master_enabled()).end_object();
                record(j.finish().into_bytes())
            }
        },
        op::EXPORT => record(file_text(cc.registry)?.into_bytes()),
        op::IMPORT => {
            let not_a_file = |w: String| refused(status::PARSE, format!("That is not a cheat file: {w}"));
            let text = std::str::from_utf8(bytes).map_err(|_| not_a_file("the text is not UTF-8".to_string()))?;
            if let Some(words) = super::cheat_file::refusal(text) {
                return Err(not_a_file(words));
            }
            let file = emusen_galaxia::config::bind_text(&MODELS[CHEAT_FILE_ID as usize], text, false).map_err(not_a_file)?.ok_or_else(|| not_a_file("null".to_string()))?;
            super::cheat_file::load(cc.registry, &file).map(|n| (None, n as i64)).map_err(|w| refused(status::FAULT, w))
        }
        op::DECODE => {
            let code = args::string(&a, "code")?;
            let which = args::optional_string(&a, "codec")?.unwrap_or_else(|| "auto".to_string());
            let unchecked = args::optional_bool(&a, "unchecked")?.unwrap_or(false);
            record(decode(cc, &code, &which, unchecked)?)
        }
        op::DB_SYSTEMS => {
            let db = Database::scan(&database_folder(&galaxia_tree::tree()));
            let mut j = Json::new();
            j.begin_array();
            for (system, count) in db.systems() {
                j.begin_object().str("system", &system).int("count", count as i64).end_object();
            }
            j.end_array();
            record(j.finish().into_bytes())
        }
        op::DB_GAMES => {
            let db = Database::scan(&database_folder(&galaxia_tree::tree()));
            let system = args::string(&a, "system")?;
            let filter = args::optional_string(&a, "filter")?;
            record(entries_json(&db, &db.games(&system, filter.as_deref())))
        }
        op::DB_FIND => {
            let db = Database::scan(&database_folder(&galaxia_tree::tree()));
            let text = args::string(&a, "text")?;
            let limit = int32(args::optional_int(&a, "limit")?.unwrap_or(20), "limit")?;
            record(entries_json(&db, &db.find(&text, limit)))
        }
        op::CLAIMS => {
            let code = args::string(&a, "code")?;
            let claims = |c: Option<&'static Codec>| c.map(|c| codecs::decode(c.id, &code).is_some_and(|a| a.can_decode));
            let mut j = Json::new();
            j.begin_object();
            for (key, codec) in [("auto", cc.auto), ("explicit", cc.explicit)] {
                match claims(codec) {
                    Some(b) => j.bool(key, b),
                    None => j.null(key),
                };
            }
            j.end_object();
            record(j.finish().into_bytes())
        }
        op::CHT_PARSE => {
            let text = args::string(&a, "text")?;
            record(parsed_json(&read_cht(cc, &text)?, None))
        }
        op::CHT_EXPORT => {
            let all = cc.registry.cheats();
            let rom_patches = all.iter().filter(|c| c.kind != Kind::RamPoke).count();
            let conditional = all.iter().filter(|c| c.kind == Kind::RamPoke && c.writes.iter().any(|w| w.kind.is_test())).count();
            let exportable: Vec<cht::Cheat> = all
                .iter()
                .filter(|c| c.kind == Kind::RamPoke && !c.writes.iter().any(|w| w.kind.is_test()))
                .map(|c| cht::Cheat { description: c.description.clone(), enabled: c.enabled, writes: c.writes.clone(), kind: Kind::RamPoke })
                .collect();
            let text = cht::write(&exportable);
            let mut j = Json::new();
            j.begin_object().str("text", &text).int("exported", exportable.len() as i64).int("rom_patches", rom_patches as i64).int("conditional", conditional as i64).end_object();
            record(j.finish().into_bytes())
        }
        op::FILES => {
            let tree = galaxia_tree::tree();
            let mut j = Json::new();
            j.begin_object().str("directory", &emusen_galaxia::cheats::directory(&tree)).strs("names", emusen_galaxia::cheats::names(&tree)).end_object();
            record(j.finish().into_bytes())
        }
        op::FILE_READ => {
            let name = valid_name(args::string(&a, "name")?)?;
            let tree = galaxia_tree::tree();
            let file = file_name(&name);
            let model = &MODELS[CHEAT_FILE_ID as usize];
            let loaded = match emusen_galaxia::config::load(&tree, model, Some(CHEATS_CATEGORY), &file, false) {
                Ok(emusen_galaxia::config::Loaded::Document(d)) => emusen_galaxia::model::format(model.schema, &d).ok().and_then(|t| emusen_dotnet::json::parse(&t).ok()),
                _ => None,
            };
            let mut j = Json::new();
            j.begin_object().str("path", &tree.config_path(Some(CHEATS_CATEGORY), &file));
            match &loaded {
                Some(v) => j.key_value("file", v),
                None => j.null("file"),
            };
            j.end_object();
            record(j.finish().into_bytes())
        }
        op::FILE_WRITE => {
            let name = valid_name(args::string(&a, "name")?)?;
            let tree = galaxia_tree::tree();
            let file = file_name(&name);
            let mut canonical = String::new();
            emusen_dotnet::json::write_compact(&mut canonical, &cc.registry.to_file());
            if emusen_galaxia::config::save(&tree, &MODELS[CHEAT_FILE_ID as usize], Some(CHEATS_CATEGORY), &file, Some(&canonical)).is_err() {
                return Err(refused(status::FAULT, format!("couldn't write {}", tree.config_path(Some(CHEATS_CATEGORY), &file))));
            }
            Ok((None, cc.registry.cheats().len() as i64))
        }
        op::DB_STATUS => {
            let directory = database_folder(&galaxia_tree::tree());
            let mut j = Json::new();
            j.begin_object().str("directory", &directory).bool("exists", galaxia_tree::directory_exists(&directory)).str("attribution", ATTRIBUTION).end_object();
            record(j.finish().into_bytes())
        }
        op::DB_PARSE => {
            let db = Database::scan(&database_folder(&galaxia_tree::tree()));
            let game = args::string(&a, "game")?;
            let Some(&best) = db.find(&game, 1).first() else { return record(parsed_json(&cht::Parsed::default(), None)) };
            let entry = &db.entries[best];
            let text = std::fs::read(&entry.path).map_err(|e| refused(status::FAULT, emusen_dotnet::file::io_message(&e, &entry.path)))?;
            let parsed = read_cht(cc, &emusen_dotnet::file::decode(&text))?;
            record(parsed_json(&parsed, Some(&entry.game)))
        }
        op::DB_PRUNE => {
            if args::optional_bool(&a, "apply")? == Some(true) {
                let provider = cc.provider;
                record(pending(provider, o, || Ok(prune_json(cc, true)))?)
            } else {
                record(prune_json(cc, false))
            }
        }
        _ => {
            let provider = cc.provider;
            record(pending(provider, o, || update_json(cc))?)
        }
    }
}

/// The record copied out when the buffer holds it, and a kept change's record then let go, `TakePending`.
fn answered(provider: &Provider, o: u32, result: (Option<Vec<u8>>, i64), room: Option<usize>) -> (Option<Vec<u8>>, i64) {
    if let (Some(record), Some(room)) = (&result.0, room)
        && room >= record.len()
    {
        provider.pending.lock().unwrap_or_else(|p| p.into_inner()).retain(|(k, _)| *k != o);
    }
    result
}

/// A session's cheat op, under its gate; `room` is the caller's buffer, none for a length query.
pub fn cheat_op(c: &Call, o: u32, raw: Raw, room: Option<usize>) -> Answer<(Option<Vec<u8>>, i64)> {
    let result = c.gated(|e, _| {
        let bytes = raw.bytes()?;
        c.target()?;
        let (auto, explicit) = session_codecs(e);
        let (d, m) = dm(e);
        let spaces = m.map(|m| d.spaces(m).into_iter().map(|s| s.name).collect()).unwrap_or_default();
        let mut cc = Cheats { rt: c.rt, provider: &c.s.provider, registry: &mut d.cheats, auto, explicit, spaces, read_only: c.read_only };
        run(&mut cc, o, bytes)
    })?;
    Ok(answered(&c.s.provider, o, result, room))
}

/// Session 0's cheat op: the provider with no game, for the ops that need none.
pub fn cheat_op_without_game(rt: &Runtime, o: u32, raw: Raw, room: Option<usize>) -> Answer<(Option<Vec<u8>>, i64)> {
    if !game_free(o) {
        return Err(refused(status::NO_SESSION, format!("cheat op {o} is a game's: name its session")));
    }
    let bytes = raw.bytes()?;
    let mut fresh = CheatRegistry::new();
    let mut cc = Cheats { rt, provider: &rt.no_game, registry: &mut fresh, auto: None, explicit: None, spaces: Vec::new(), read_only: false };
    let result = run(&mut cc, o, bytes)?;
    Ok(answered(&rt.no_game, o, result, room))
}

fn codec_json(j: &mut Json, key: &str, codec: Option<&'static Codec>) {
    match codec {
        None => {
            j.null(key);
        }
        Some(c) => {
            j.key_object(key).str("name", c.name).str("kind", kind_word(c.kind)).str_or_null("space", codecs::space(c)).end_object();
        }
    }
}

/// `{"auto","explicit","systems"}`: the session's codecs and the systems the database is pruned to keep.
pub fn cheat_codecs(c: &Call) -> Answer<Vec<u8>> {
    c.target()?;
    c.read(|e, _| {
        let (auto, explicit) = session_codecs(e);
        let mut j = Json::new();
        j.begin_object();
        codec_json(&mut j, "auto", auto);
        codec_json(&mut j, "explicit", explicit);
        j.strs("systems", c.rt.supported_cheat_systems()).end_object();
        Ok(j.finish().into_bytes())
    })
}
