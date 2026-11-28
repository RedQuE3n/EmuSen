//! `emusen.cheat`: `cheat`, every cheat mechanism under one command, written from its C# class to print what it
//! printed. Decoding is the codecs', the cheat lists' files and the database are the session's, and the module
//! formats (EmuSen_DianaSh.md §9.2); a .cht file is read and written through the call's resolve. See §8.1 and §13.11.

use dianash_module::{Answer, Call, Manifest, Module};
use serde_json::{json, Value};
use emusen_session_service::abi::{self, cheat, status};
use emusen_session_service::space::find;
use emusen_session_service::words::{hex_digits, parse_hex, parse_value_bits, split_common_flags, split_flags_before_tail};
use emusen_session_service::{object, quoted, Record, Refusal, Session};

pub const USAGE: &str = concat!(
    "  cheat add <code> [description]    decode a code, guessing whether it's Pro\n",
    "                                     Action Replay/Game Wizard or Game Genie from\n",
    "                                     its formatting (see `gg` if it guesses wrong -\n",
    "                                     both formats use the same 8 hex-digit\n",
    "                                     characters, so this is a best-effort guess,\n",
    "                                     not a guarantee)\n",
    "  cheat poke <space> <addr> <value> [description] [-w 1|2|4] [--le|--be] [--dec|--float] [--json]\n",
    "                                     add a raw RAM-poke cheat directly, bypassing\n",
    "                                     code decoding (e.g. an address you already\n",
    "                                     found with `search`)\n",
    "  cheat from-freeze <id> [--keep|--remove] [--json] [description]\n",
    "                                     graduate an active freeze into a persisted\n",
    "                                     custom cheat\n",
    "  cheat gg <code> [description]     decode a real SNES Game Genie code (8\n",
    "                                     characters) and add it as a ROM patch, enabled\n",
    "                                     - use this instead of `add` when its guess is\n",
    "                                     wrong for a code formatted unconventionally\n",
    "  cheat rompatch <addr> <value> [<compare>|-] [description]\n",
    "                                     add a Game Genie-style ROM-read intercept\n",
    "                                     directly, bypassing code decoding: substitute\n",
    "                                     <value> for reads of cartridge address <addr>.\n",
    "                                     <compare> gates the patch to only apply while\n",
    "                                     the real byte there equals it (pass - to skip,\n",
    "                                     an unconditional patch - what every real SNES\n",
    "                                     Game Genie code decodes to; unlike NES/Genesis,\n",
    "                                     SNES Game Genie codes never carry a compare)\n",
    "  cheat list [--json]                list every cheat with its ID, kind, and state\n",
    "  cheat master [on|off]              one switch over every cheat at once, without\n",
    "                                     touching any cheat's own enabled state (no\n",
    "                                     argument reports which way it is set)\n",
    "  cheat enable <id>                  turn a cheat back on\n",
    "  cheat disable <id>                 turn a cheat off without removing it\n",
    "  cheat remove <id>                  remove a cheat entirely\n",
    "  cheat clear                        remove every cheat\n",
    "  cheat save <name>                  write every cheat to\n",
    "                                     /etc/EmuSen/cheats/<name>.json\n",
    "  cheat load <name>                  add every cheat from that file to whatever\n",
    "                                     is already loaded (`cheat clear` first to\n",
    "                                     replace rather than merge)\n",
    "  cheat files                        list the saved cheat files\n",
    "  cheat import <path.cht>            import RetroArch/libretro .cht cheats, added\n",
    "                                     disabled so nothing turns on by surprise\n",
    "  cheat export <path.cht>            write RAM-poke cheats out as a .cht file\n",
    "  cheat db [status]                  where the cheat database is, and what is in it\n",
    "  cheat db find <game>               search the database for a game\n",
    "  cheat db load <game>               import that game's cheats, disabled\n",
    "  cheat db update                    download the libretro cheat database (CC BY-SA 4.0)\n",
    "  cheat db prune [--apply]           delete the systems no core in this build can use"
);

/// The module: one value, no state of its own.
pub struct Cheat;

pub static MODULE: Cheat = Cheat;

type Run = Result<Answer, String>;

/// One codec as `cheat_codecs` names it.
struct Codec {
    name: String,
    space: Option<String>,
}

fn codec(value: Option<&Value>) -> Option<Codec> {
    value.map(|c| Codec { name: c.text("name"), space: c.optional_text("space") })
}

/// A cheat op's record.
fn op(session: &Session, op: u32, args: &str) -> Result<Value, Refusal> {
    let (bytes, _) = session.cheat_op(op, args.as_bytes())?;
    serde_json::from_str::<Value>(&String::from_utf8_lossy(&bytes)).map_err(|e| Refusal::new(status::FAULT, e.to_string()))
}

/// A change's value: an id, a count or 0.
fn change(session: &Session, op: u32, args: &str) -> Result<i64, Refusal> {
    Ok(session.cheat_op(op, args.as_bytes())?.1)
}

fn compact(value: &Value) -> String {
    let mut out = String::new();
    out.push_str(&serde_json::to_string(value).expect("a value is JSON"));
    out
}

/// `LooksLikeGameGenieFormat`: a separator after four hex digits; none at all is Pro Action Replay's form.
fn looks_like_game_genie(code: &str) -> bool {
    for (i, c) in code.chars().enumerate() {
        if !c.is_ascii_hexdigit() {
            return i == 4;
        }
    }
    false
}

/// `CheatWrite` as `FormatWrite` prints it, from the service's record of one.
fn format_write(w: &Value, rom: bool) -> String {
    let address = w.address("address");
    let mut place = if rom { format!("0x{address:06X}") } else { format!("{} 0x{address:X}", w.text("space")) };
    let kind = w.text("type");
    let operator = match kind.as_str() {
        "Increase" => "+=",
        "Decrease" => "-=",
        "IfEqual" => "==",
        "IfNotEqual" => "!=",
        "IfLess" => "<",
        "IfGreater" => ">",
        _ => "=",
    };
    if matches!(kind.as_str(), "IfEqual" | "IfNotEqual" | "IfLess" | "IfGreater") {
        place = format!("if {place}");
    }
    let value = w.int("value") as u32;
    if let Some(bit) = w.optional_int("bit") {
        return format!("{place} bit{bit} {operator} {}", value & 1);
    }
    let width = match w.int("width") {
        2 => 2,
        4 => 4,
        _ => 1,
    };
    let repeat = w.int("repeat").max(1);
    let shown = format!("0x{value:0digits$X}", digits = width * 2);
    let size = if width > 1 { format!(" ({width}-byte{})", if w.flag("big_endian") { ", big-endian" } else { "" }) } else { String::new() };
    let steps = if repeat > 1 {
        let add_value = w.int("repeat_value") as u32;
        format!(" x{repeat} step 0x{:X}{}", w.int("repeat_address") as i32, if add_value != 0 { format!("/+0x{add_value:X}") } else { String::new() })
    } else {
        String::new()
    };
    format!("{place} {operator} {shown}{size}{steps}")
}

fn format_cheat(c: &Value) -> String {
    let state = if c.flag("enabled") { "on " } else { "off" };
    let rom = c.text("kind") == "rom";
    let kind = if rom { "ROM" } else { "RAM" };
    let compare = c.optional_int("compare").map_or(String::new(), |v| format!(" if==0x{v:02X}"));
    let writes = c.list("writes");
    let description = c.text("description");
    if writes.len() == 1 {
        return format!("  #{}: [{state}] {kind}  {}{compare}  {description}", c.int("id"), format_write(&writes[0], rom));
    }
    let mut lines = vec![format!("  #{}: [{state}] {kind}  {} writes{compare}  {description}", c.int("id"), writes.len())];
    lines.extend(writes.iter().map(|w| format!("        {}", format_write(w, rom))));
    lines.join("\n")
}

/// `CheatPrunePlan.Human`: bytes in the largest unit that holds them.
fn human(bytes: i64) -> String {
    if bytes >= 1 << 30 {
        format!("{:.1}G", bytes as f64 / (1u64 << 30) as f64)
    } else if bytes >= 1 << 20 {
        format!("{:.0}M", bytes as f64 / (1u64 << 20) as f64)
    } else if bytes >= 1 << 10 {
        format!("{:.0}K", bytes as f64 / (1u64 << 10) as f64)
    } else {
        format!("{bytes}B")
    }
}

/// `Path.Combine`: a separator between, unless the first already ends in one.
fn combine(directory: &str, name: &str) -> String {
    let separator = std::path::MAIN_SEPARATOR;
    if directory.ends_with(separator) || directory.ends_with('/') { format!("{directory}{name}") } else { format!("{directory}{separator}{name}") }
}

/// The cheats a parse found, each added disabled; what the registry refused counts as neither loaded nor skipped.
fn add_parsed(session: &Session, parsed: &Value) -> Result<i64, Refusal> {
    let mut loaded = 0;
    for cheat in parsed.list("cheats") {
        let args = object(&[
            ("kind", quoted(&cheat.text("kind"))),
            ("writes", compact(cheat.field("writes").unwrap_or(&Value::Array(Vec::new())))),
            ("description", quoted(&cheat.text("description"))),
            ("enabled", "false".to_string()),
        ]);
        match change(session, cheat::ADD, &args) {
            Ok(_) => loaded += 1,
            Err(refusal) if refusal.is(status::BAD_ARGUMENT) => {}
            Err(refusal) => return Err(refusal),
        }
    }
    Ok(loaded)
}

/// A code decoded unchecked, as the C# command decoded it: the codec's own words where it refuses.
fn decode(session: &Session, code: &str, which: &str) -> Result<Value, String> {
    Ok(op(session, cheat::DECODE, &object(&[("code", quoted(code)), ("codec", quoted(which)), ("unchecked", "true".to_string())]))?)
}

/// `AddPatchWrites` with the explicit codec, or its single patch: the ROM cheat a Game Genie-style code adds.
fn add_patch(session: &Session, explicit: &Codec, code: &str, description: &str, detected: bool) -> Run {
    let decoded = decode(session, code, "explicit")?;
    if let Some(writes) = decoded.field("writes") {
        let args = object(&[("kind", quoted("rom")), ("writes", compact(writes)), ("description", quoted(description))]);
        let id = change(session, cheat::ADD, &args)?;
        let list = writes.items();
        let what = if list.len() == 1 { format_write(&list[0], true) } else { format!("{} writes", list.len()) };
        return Ok(Answer::ok(format!("Cheat #{id} added (detected {} format): ROM {what} ({description})", explicit.name)));
    }
    let (address, value) = (decoded.address("address"), decoded.int("value"));
    let compare = decoded.optional_int("compare");
    let args = object(&[
        ("address", (address as u32).to_string()),
        ("value", value.to_string()),
        ("compare", compare.map_or("null".to_string(), |c| c.to_string())),
        ("description", quoted(description)),
    ]);
    let id = change(session, cheat::ROM_PATCH, &args)?;
    let condition = compare.map_or(String::new(), |c| format!(" if==0x{c:02X}"));
    let detected = if detected { format!(" (detected {} format)", explicit.name) } else { String::new() };
    Ok(Answer::ok(format!("Cheat #{id} added{detected}: ROM 0x{address:06X} = 0x{value:02X}{condition} ({description})")))
}

fn run(call: &Call, parts: &[String]) -> Run {
    let session = Session::from_service(call.service(abi::SERVICE, abi::MAJOR))?;
    session.info()?;
    if parts.len() < 2 {
        return Ok(Answer::ok("Usage: cheat add|poke|from-freeze|gg|rompatch|list|master|enable|disable|remove|clear|save|load|files|import|export ..."));
    }
    let sub = parts[1].to_lowercase();
    let codecs = session.cheat_codecs()?;
    let (auto, explicit) = (codec(codecs.field("auto")), codec(codecs.field("explicit")));
    let rest = |from: usize| parts[from..].join(" ");
    match sub.as_str() {
        "add" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: cheat add <code> [description]"));
            }
            let code = &parts[2];
            let description = if parts.len() > 3 { rest(3) } else { code.clone() };
            let prefers_explicit = match (&explicit, &auto) {
                (None, _) | (_, None) => explicit.is_some(),
                _ => {
                    let claims = op(&session, cheat::CLAIMS, &object(&[("code", quoted(code))]))?;
                    let (e, a) = (claims.flag("explicit"), claims.flag("auto"));
                    if e != a { e } else { looks_like_game_genie(code) }
                }
            };
            if prefers_explicit {
                let Some(explicit) = &explicit else { return Ok(Answer::ok("No Game Genie-style cheat codec is registered for this target.")) };
                return add_patch(&session, explicit, code, &description, true);
            }
            let Some(auto) = &auto else { return Ok(Answer::ok("No cheat codec is registered for this target.")) };
            let decoded = decode(&session, code, "auto")?;
            if let Some(writes) = decoded.field("writes") {
                let args = object(&[("kind", quoted("ram")), ("writes", compact(writes)), ("description", quoted(&description))]);
                let id = change(&session, cheat::ADD, &args)?;
                let list = writes.items();
                let what = if list.len() == 1 { format_write(&list[0], false) } else { format!("{} writes", list.len()) };
                return Ok(Answer::ok(format!("Cheat #{id} added (detected {} format): {what} ({description})", auto.name)));
            }
            let (address, value) = (decoded.address("address"), decoded.int("value"));
            let space = auto.space.clone().unwrap_or_else(|| "CpuBus".to_string());
            let id = add_poke(&session, &space, address, value, &description)?;
            Ok(Answer::ok(format!("Cheat #{id} added (detected {} format): {space} 0x{address:06X} = 0x{value:02X} ({description})", auto.name)))
        }
        "poke" => {
            let (pos, _, desc_tokens, flags) = split_flags_before_tail(&parts[2..], 3, &[])?;
            if pos.len() < 3 {
                return Ok(Answer::ok("Usage: cheat poke <space> <addr> <value> [-w 1|2|4] [--le|--be] [--dec|--float] [--json] [description]"));
            }
            let space = find(&session, &pos[0])?;
            let address = parse_hex(&pos[1])?;
            let width = flags.width.unwrap_or(if flags.float { 4 } else { 1 });
            let big_endian = flags.endian.unwrap_or(space.byte_order).is_big();
            let decimal = flags.decimal.unwrap_or(flags.float);
            let value = if width == 1 && !flags.float && !decimal && flags.width.is_none() {
                u32::from(parse_hex(&pos[2])? as u8)
            } else {
                parse_value_bits(&pos[2], width, decimal, flags.float)?
            };
            let description = if desc_tokens.is_empty() { format!("{} 0x{address:X}", space.name) } else { desc_tokens.join(" ") };
            let id = add_poke_wide(&session, &space.name, address, i64::from(value), width, big_endian, &description)?;
            if flags.json {
                let native_code = native_ram_code(&session, &space.name, address as u32, value, width as u8, big_endian);
                let digits = hex_digits(space.size);
                return Ok(Answer::ok(json!({
                    "id": id,
                    "name": description,
                    "space": space.name,
                    "address": address as u32,
                    "addr_hex": format!("{:0digits$X}", address as u32),
                    "value": value,
                    "width": width,
                    "big_endian": big_endian,
                    "native_code": native_code,
                }).to_string()));
            }
            if width == 1 && !flags.float && flags.width.is_none() {
                Ok(Answer::ok(format!("Cheat #{id} added: {} 0x{address:X} = 0x{value:02X} ({description})", pos[0])))
            } else {
                let digits = (width as usize) * 2;
                let order = if big_endian { "BE" } else { "LE" };
                Ok(Answer::ok(format!("Cheat #{id} added: {} 0x{address:X} = 0x{value:0digits$X} ({width}-byte {order}) ({description})", space.name)))
            }
        }
        "from-freeze" => {
            let (pos, extra, desc_tokens, flags) = split_flags_before_tail(&parts[2..], 1, &["--keep", "--remove"])?;
            if pos.is_empty() {
                return Ok(Answer::ok("Usage: cheat from-freeze <id> [--keep|--remove] [--json] [description]"));
            }
            let remove = extra.iter().any(|t| t == "--remove");
            let freeze_id = parse_hex(&pos[0])?;
            let freezes = session.freeze_list()?;
            let Some(found) = freezes.items().iter().find(|f| f.int("id") == i64::from(freeze_id)) else {
                return Ok(Answer::fail(format!("No frozen address #{freeze_id}.")));
            };
            let space_name = found.text("space");
            let address = found.address("address");
            let value = found.int("value") as u32;
            let width = found.optional_int("width").unwrap_or(1) as i32;
            let big_endian = found.flag("big_endian");
            let description = if desc_tokens.is_empty() { format!("{space_name} 0x{address:X}") } else { desc_tokens.join(" ") };
            let id = add_poke_wide(&session, &space_name, address, i64::from(value), width, big_endian, &description)?;
            if remove {
                session.freeze_remove(freeze_id as u32)?;
            }
            let native_code = native_ram_code(&session, &space_name, address as u32, value, width as u8, big_endian);
            if flags.json {
                return Ok(Answer::ok(json!({
                    "id": id,
                    "name": description,
                    "space": space_name,
                    "address": address as u32,
                    "value": value,
                    "width": width,
                    "big_endian": big_endian,
                    "native_code": native_code,
                }).to_string()));
            }
            let digits = (width.max(1) as usize) * 2;
            Ok(Answer::ok(format!("Cheat #{id} added from freeze #{freeze_id}: {space_name} 0x{address:X} = 0x{value:0digits$X} ({description})")))
        }
        "gg" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: cheat gg <code> [description]"));
            }
            let Some(explicit) = &explicit else { return Ok(Answer::ok("No Game Genie-style cheat codec is registered for this target.")) };
            let description = if parts.len() > 3 { rest(3) } else { parts[2].clone() };
            add_patch(&session, explicit, &parts[2], &description, false)
        }
        "rompatch" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: cheat rompatch <addr> <value> [<compare>|-] [description]"));
            }
            let address = parse_hex(&parts[2])?;
            let value = parse_hex(&parts[3])? as u8;
            let mut compare = None;
            let mut from = 4;
            if parts.len() > 4 {
                if parts[4] != "-" {
                    compare = Some(parse_hex(&parts[4])? as u8);
                }
                from = 5;
            }
            let description = if parts.len() > from { rest(from) } else { format!("ROM 0x{address:06X}") };
            let args = object(&[
                ("address", (address as u32).to_string()),
                ("value", value.to_string()),
                ("compare", compare.map_or("null".to_string(), |c| c.to_string())),
                ("description", quoted(&description)),
            ]);
            let id = change(&session, cheat::ROM_PATCH, &args)?;
            let condition = compare.map_or(String::new(), |c| format!(" if==0x{c:02X}"));
            Ok(Answer::ok(format!("Cheat #{id} added: ROM 0x{address:06X} = 0x{value:02X}{condition} ({description})")))
        }
        "list" => {
            let (_, flags) = split_common_flags(&parts[2..])?;
            let list = op(&session, cheat::LIST, "{}")?;
            let cheats = list.list("cheats");
            if flags.json {
                let rows: Vec<Value> = cheats
                    .iter()
                    .enumerate()
                    .map(|(index, c)| {
                        json!({
                            "index": index,
                            "id": c.int("id"),
                            "name": c.text("description"),
                            "enabled": c.flag("enabled"),
                            "kind": c.text("kind"),
                            "writes": c.field("writes").cloned().unwrap_or_else(|| Value::Array(Vec::new())),
                        })
                    })
                    .collect();
                return Ok(Answer::ok(Value::Array(rows).to_string()));
            }
            if cheats.is_empty() {
                return Ok(Answer::ok("No cheats added."));
            }
            let mut lines: Vec<String> = cheats.iter().map(format_cheat).collect();
            if !list.flag("master") {
                lines.insert(0, "Cheats are OFF at the master switch (`cheat master on`).".to_string());
            }
            Ok(Answer::ok(lines.join("\n")))
        }
        "master" => {
            let state = |on: bool| if on { "ON" } else { "OFF" };
            if parts.len() < 3 {
                return Ok(Answer::ok(format!("Cheats are {} at the master switch.", state(op(&session, cheat::MASTER, "{}")?.flag("enabled")))));
            }
            let want = parts[2].to_lowercase();
            if want != "on" && want != "off" {
                return Ok(Answer::ok("Usage: cheat master [on|off]"));
            }
            change(&session, cheat::MASTER, &object(&[("enabled", (want == "on").to_string())]))?;
            Ok(Answer::ok(format!("Cheats are now {} at the master switch.", state(op(&session, cheat::MASTER, "{}")?.flag("enabled")))))
        }
        "enable" | "disable" | "remove" => {
            if parts.len() < 3 {
                return Ok(Answer::ok(format!("Usage: cheat {sub} <id>")));
            }
            let id = parse_hex(&parts[2])?;
            let which = match sub.as_str() {
                "enable" => cheat::ENABLE,
                "disable" => cheat::DISABLE,
                _ => cheat::REMOVE,
            };
            Ok(Answer::ok(match change(&session, which, &object(&[("id", id.to_string())])) {
                Ok(_) => format!("Cheat #{} {sub}d.", parts[2]),
                Err(refusal) if refusal.is(status::NO_SUCH_ID) => format!("No cheat #{} found.", parts[2]),
                Err(refusal) => return Err(refusal.words),
            }))
        }
        "clear" => {
            change(&session, cheat::CLEAR, "{}")?;
            Ok(Answer::ok("All cheats removed."))
        }
        "save" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: cheat save <name>"));
            }
            let name = &parts[2];
            let path = combine(&op(&session, cheat::FILES, "{}")?.text("directory"), &format!("{name}.json"));
            Ok(Answer::ok(match change(&session, cheat::FILE_WRITE, &object(&[("name", quoted(name))])) {
                Ok(saved) => format!("Saved {saved} cheat(s) to {path}"),
                Err(refusal) if refusal.is(status::BAD_ARGUMENT) => format!("cheat save: '{name}' is not a usable file name."),
                Err(refusal) if refusal.is(status::FAULT) => format!("cheat save: couldn't write {path}"),
                Err(refusal) => return Err(refusal.words),
            }))
        }
        "load" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: cheat load <name>"));
            }
            let name = &parts[2];
            let read = match op(&session, cheat::FILE_READ, &object(&[("name", quoted(name))])) {
                Ok(read) => read,
                Err(refusal) if refusal.is(status::BAD_ARGUMENT) => return Ok(Answer::ok(format!("cheat load: '{name}' is not a usable file name."))),
                Err(refusal) => return Err(refusal.words),
            };
            let path = read.text("path");
            let Some(file) = read.field("file") else { return Ok(Answer::ok(format!("cheat load: no readable cheat file at {path}"))) };
            let entries = match file {
                Value::Object(members) => members.iter().find(|(k, _)| k.eq_ignore_ascii_case("cheats")).map_or(0, |(_, v)| v.items().len()),
                _ => 0,
            } as i64;
            let added = session.cheat_op(cheat::IMPORT, compact(file).as_bytes())?.1;
            let skipped = entries - added;
            let note = if skipped > 0 { format!(" ({skipped} unparseable entr{} skipped)", if skipped == 1 { "y" } else { "ies" }) } else { String::new() };
            Ok(Answer::ok(format!("Loaded {added} cheat(s) from {path}{note}")))
        }
        "import" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: cheat import <path.cht>"));
            }
            let arg = rest(2);
            let Ok(path) = call.resolve(&arg) else { return Ok(Answer::ok(format!("cheat import: '{arg}' is outside the sandbox."))) };
            if !std::path::Path::new(&path).is_file() {
                return Ok(Answer::ok(format!("cheat import: no file at {path}")));
            }
            let imported = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path)).map_err(|e| Refusal::new(status::FAULT, e)).and_then(|text| {
                let parsed = op(&session, cheat::CHT_PARSE, &object(&[("text", quoted(&text))]))?;
                Ok((add_parsed(&session, &parsed)?, parsed.int("skipped")))
            });
            let (loaded, skipped) = match imported {
                Ok(counts) => counts,
                Err(refusal) => return Ok(Answer::ok(format!("cheat import: {}", refusal.words))),
            };
            let name = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
            let note = if skipped > 0 { format!(", {skipped} skipped") } else { String::new() };
            Ok(Answer::ok(format!("Imported {loaded} cheat(s) from {name}{note} - all disabled, `cheat enable <id>` to turn one on.")))
        }
        "export" => {
            if parts.len() < 3 {
                return Ok(Answer::ok("Usage: cheat export <path.cht>"));
            }
            let arg = rest(2);
            let Ok(path) = call.resolve(&arg) else { return Ok(Answer::ok(format!("cheat export: '{arg}' is outside the sandbox."))) };
            let export = op(&session, cheat::CHT_EXPORT, "{}")?;
            if let Some(parent) = std::path::Path::new(&path).parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    return Ok(Answer::ok(format!("cheat export: {}: {}", parent.to_string_lossy(), e)));
                }
            }
            if let Err(words) = std::fs::write(&path, export.text("text")).map_err(|e| format!("{path}: {e}")) {
                return Ok(Answer::ok(format!("cheat export: {words}")));
            }
            let mut note = String::new();
            if export.int("rom_patches") > 0 {
                note += &format!(" ({} ROM patch(es) skipped - .cht has no equivalent)", export.int("rom_patches"));
            }
            if export.int("conditional") > 0 {
                note += &format!(" ({} conditional cheat(s) skipped - one .cht entry cannot hold a test and the write it guards)", export.int("conditional"));
            }
            Ok(Answer::ok(format!("Exported {} cheat(s) to {path}{note}", export.int("exported"))))
        }
        "db" => database(&session, parts),
        "files" => {
            let files = op(&session, cheat::FILES, "{}")?;
            let directory = files.text("directory");
            let names = files.list("names");
            if names.is_empty() {
                return Ok(Answer::ok(format!("No cheat files in {directory}")));
            }
            let mut lines = vec![directory];
            lines.extend(names.iter().filter_map(Value::as_str).map(|n| format!("  {n}")));
            Ok(Answer::ok(lines.join("\n")))
        }
        _ => {
            let subcommands = ["add", "poke", "from-freeze", "gg", "rompatch", "list", "master", "enable", "disable", "remove", "clear", "save", "load", "files", "import", "export", "db"];
            Ok(Answer::ok(format!("Unknown 'cheat' subcommand '{sub}'.{} Try {}.", emusen_suggest::hint(&sub, &subcommands, 2), subcommands.join("/"))))
        }
    }
}

/// `AddRamPoke`: one write in the space as the command was given it.
fn add_poke(session: &Session, space: &str, address: i32, value: i64, description: &str) -> Result<i64, Refusal> {
    add_poke_wide(session, space, address, value, 1, false, description)
}

fn add_poke_wide(session: &Session, space: &str, address: i32, value: i64, width: i32, big_endian: bool, description: &str) -> Result<i64, Refusal> {
    let write = object(&[
        ("space", quoted(space)),
        ("address", (address as u32).to_string()),
        ("value", value.to_string()),
        ("width", width.to_string()),
        ("big_endian", big_endian.to_string()),
    ]);
    change(session, cheat::ADD, &object(&[("kind", quoted("ram")), ("writes", format!("[{write}]")), ("description", quoted(description)), ("enabled", "true".to_string())]))
}

fn native_ram_code(session: &Session, space: &str, address: u32, value: u32, width: u8, big_endian: bool) -> Option<String> {
    let system = session.system().ok()?;
    let (auto, _) = emusen_systems::cheats::pair(&system);
    let codec = auto?;
    let expected_space = emusen_systems::cheats::space(codec);
    let matches_space = expected_space.is_some_and(|s| s.eq_ignore_ascii_case(space))
        || (codec.id == "snes.par" && space.eq_ignore_ascii_case("WRAM"));
    if !matches_space {
        return None;
    }
    if width > 1 && big_endian != matches!(codec.id, "md.ar" | "n64.gs") {
        return None;
    }
    emusen_systems::cheats::encode_ram(codec.id, address, value, width)
}

/// The plan's lines: what would go, biggest first, and what stays.
fn prune_listing(plan: &Value) -> (String, Vec<String>, String) {
    let mut removing: Vec<&Value> = plan.list("removing").iter().collect();
    removing.sort_by_key(|s| std::cmp::Reverse(s.int("files")));
    let files: i64 = plan.list("removing").iter().map(|s| s.int("files")).sum();
    let bytes: i64 = plan.list("removing").iter().map(|s| s.int("bytes")).sum();
    let header = format!("{} system(s), {files} file(s), {}", plan.list("removing").len(), human(bytes));
    let listing = removing.iter().map(|s| format!("  {}  ({} file(s), {})", s.text("system"), s.int("files"), human(s.int("bytes")))).collect();
    let keeping: Vec<String> = plan.list("keeping").iter().filter_map(Value::as_str).map(str::to_string).collect();
    (header, listing, format!("Keeping: {}", keeping.join(", ")))
}

fn database(session: &Session, parts: &[String]) -> Run {
    let sub = parts.get(2).map(|s| s.to_lowercase()).unwrap_or_else(|| "status".to_string());
    let rest = |from: usize| parts[from..].join(" ");
    match sub.as_str() {
        "status" => {
            let status = op(session, cheat::DB_STATUS, "{}")?;
            let directory = status.text("directory");
            if !status.flag("exists") {
                return Ok(Answer::ok(format!(
                    "No cheat database at {directory}\nRun `cheat db update` to download one, or point AppSettings.CheatDatabaseDirectory at an existing RetroArch cheats folder."
                )));
            }
            let systems = op(session, cheat::DB_SYSTEMS, "{}")?;
            let total: i64 = systems.items().iter().map(|s| s.int("count")).sum();
            if total == 0 {
                return Ok(Answer::ok(format!("Cheat database at {directory} is empty.")));
            }
            let mut lines = vec![format!("{total} cheat file(s) in {directory}")];
            lines.extend(systems.items().iter().map(|s| format!("  {}  ({})", s.text("system"), s.int("count"))));
            Ok(Answer::ok(lines.join("\n")))
        }
        "find" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: cheat db find <game>"));
            }
            let matches = op(session, cheat::DB_FIND, &object(&[("text", quoted(&rest(3))), ("limit", "20".to_string())]))?;
            if matches.items().is_empty() {
                return Ok(Answer::ok("No matching cheat file."));
            }
            Ok(Answer::ok(matches.items().iter().map(|m| format!("  {}   [{}]", m.text("game"), m.text("system"))).collect::<Vec<_>>().join("\n")))
        }
        "load" => {
            if parts.len() < 4 {
                return Ok(Answer::ok("Usage: cheat db load <game>"));
            }
            let parsed = match op(session, cheat::DB_PARSE, &object(&[("game", quoted(&rest(3)))])) {
                Ok(parsed) => parsed,
                Err(refusal) => return Ok(Answer::ok(format!("cheat db load: {}", refusal.words))),
            };
            let Some(game) = parsed.optional_text("game") else { return Ok(Answer::ok("No matching cheat file - try `cheat db find` first.")) };
            let loaded = match add_parsed(session, &parsed) {
                Ok(loaded) => loaded,
                Err(refusal) => return Ok(Answer::ok(format!("cheat db load: {}", refusal.words))),
            };
            let skipped = parsed.int("skipped");
            let note = if skipped > 0 { format!(", {skipped} skipped") } else { String::new() };
            Ok(Answer::ok(format!("Loaded {loaded} cheat(s) for {game}{note} - all disabled, `cheat enable <id>` to turn one on.")))
        }
        "update" => {
            let result = match op(session, cheat::DB_UPDATE, "{}") {
                Ok(result) => result,
                Err(refusal) => return Ok(Answer::ok(format!("cheat db update: {}", refusal.words))),
            };
            let plan = op(session, cheat::DB_PRUNE, "{}")?;
            let offer = if plan.flag("can_apply") {
                let removing = plan.list("removing").len();
                let files: i64 = plan.list("removing").iter().map(|s| s.int("files")).sum();
                let bytes: i64 = plan.list("removing").iter().map(|s| s.int("bytes")).sum();
                format!(
                    "\n\n{removing} of {} system(s) have no core in this build ({files} file(s), {}). `cheat db prune` lists them.",
                    removing + plan.list("keeping").len(),
                    human(bytes)
                )
            } else {
                String::new()
            };
            Ok(Answer::ok(format!("Installed {} cheat file(s) into {}{offer}\n\n{}", result.int("installed"), result.text("directory"), result.text("attribution"))))
        }
        "prune" => {
            let plan = op(session, cheat::DB_PRUNE, "{}")?;
            if !plan.flag("can_apply") {
                return Ok(Answer::ok(format!("Nothing to prune - {}", plan.text("reason"))));
            }
            let (header, listing, kept) = prune_listing(&plan);
            if !parts.iter().skip(3).any(|p| p.eq_ignore_ascii_case("--apply")) {
                let mut lines = vec![format!("Would delete {header}:")];
                lines.extend(listing);
                lines.push(kept);
                lines.push("\nRe-run as `cheat db prune --apply` to delete them.".to_string());
                return Ok(Answer::ok(lines.join("\n")));
            }
            let applied = op(session, cheat::DB_PRUNE, "{\"apply\":true}")?;
            let failed: Vec<String> = applied.list("failed").iter().filter_map(Value::as_str).map(str::to_string).collect();
            let trouble = if failed.is_empty() { String::new() } else { format!("\nCouldn't remove: {}", failed.join("; ")) };
            let (_, _, kept) = prune_listing(&applied);
            Ok(Answer::ok(format!("Deleted {} system(s), {}.\n{kept}{trouble}", applied.int("pruned"), header.split_once(", ").map_or(header.as_str(), |(_, rest)| rest))))
        }
        _ => {
            let subcommands = ["status", "find", "load", "update", "prune"];
            Ok(Answer::ok(format!("Unknown 'cheat db' subcommand '{sub}'.{} Try {}.", emusen_suggest::hint(&sub, &subcommands, 2), subcommands.join("/"))))
        }
    }
}

impl Module for Cheat {
    fn manifest(&self) -> &'static str {
        static MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        MANIFEST.get_or_init(|| {
            Manifest { id: "emusen.cheat", version: env!("CARGO_PKG_VERSION"), services: &[(abi::SERVICE, abi::MAJOR, 1, false)], commands: &[("cheat", false, USAGE)], aliases: &[], jobs: &[], settings: &[], approvals: false }.json()
        })
    }

    fn run(&self, call: &Call) -> i32 {
        let args = call.args();
        call.answer(run(call, &args).unwrap_or_else(Answer::fault))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_as_the_pruner_prints_them() {
        assert_eq!(human(76), "76B");
        assert_eq!(human(1536), "2K");
        assert_eq!(human(5 << 20), "5M");
        assert_eq!(human(3 << 29), "1.5G");
        assert!(looks_like_game_genie("DD62-6DAD"));
        assert!(!looks_like_game_genie("7E0019:01"));
        assert!(!looks_like_game_genie("DD626DAD"));
    }

    #[test]
    fn poke_wide_from_freeze_and_list_json() {
        use emusen_dianash::shell::{Config, Shell, Submitted};
        use emusen_session_service::abi::{self, emusen_session_v1};
        use std::cell::RefCell;
        use std::ffi::c_void;

        struct FakeCheatCtx {
            cheats: RefCell<Vec<Value>>,
            freezes: RefCell<Vec<(u32, String, u32, u32, i32, bool)>>,
        }

        unsafe extern "C" fn fake_info(_c: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
            unsafe { abi::copy_answer(br#"{"id":1,"engine":"Fake","system":"n64","byte_order":"big","capabilities":["freezes","cheats"]}"#, o, l) }
        }

        unsafe extern "C" fn fake_spaces(_c: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
            unsafe { abi::copy_answer(br#"[{"name":"RDRAM","size":4096,"writable":true,"side_effects":false,"byte_order":"big"}]"#, o, l) }
        }

        unsafe extern "C" fn fake_codecs(_c: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
            unsafe { abi::copy_answer(br#"{"auto":{"name":"GameShark","kind":"RamPoke","space":"RDRAM"},"explicit":null,"systems":["n64"]}"#, o, l) }
        }

        unsafe extern "C" fn fake_freeze_list(c: *mut c_void, _s: u32, o: *mut u8, l: usize) -> i64 {
            let f = unsafe { &*(c as *const FakeCheatCtx) };
            let arr: Vec<Value> = f
                .freezes
                .borrow()
                .iter()
                .map(|(id, sp, addr, val, w, be)| json!({"id": id, "space": sp, "address": addr, "value": val, "width": w, "big_endian": be, "blocked": 1}))
                .collect();
            unsafe { abi::copy_answer(Value::Array(arr).to_string().as_bytes(), o, l) }
        }

        unsafe extern "C" fn fake_freeze_remove(c: *mut c_void, _s: u32, id: u32) -> i32 {
            let f = unsafe { &*(c as *const FakeCheatCtx) };
            f.freezes.borrow_mut().retain(|x| x.0 != id);
            0
        }

        unsafe extern "C" fn fake_cheat_op(c: *mut c_void, _s: u32, op_code: u32, args: *const u8, args_len: usize, o: *mut u8, l: usize) -> i64 {
            let f = unsafe { &*(c as *const FakeCheatCtx) };
            let text = unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(args, args_len)) };
            match op_code {
                abi::cheat::ADD => {
                    let mut entry: Value = serde_json::from_str(text).unwrap();
                    let mut list = f.cheats.borrow_mut();
                    let id = (list.len() + 1) as i64;
                    entry["id"] = json!(id);
                    entry["enabled"] = json!(true);
                    list.push(entry);
                    id
                }
                abi::cheat::LIST => {
                    let doc = json!({"master": true, "cheats": *f.cheats.borrow()}).to_string();
                    unsafe { abi::copy_answer(doc.as_bytes(), o, l) }
                }
                _ => -1103,
            }
        }

        let mut ctx = FakeCheatCtx {
            cheats: RefCell::new(Vec::new()),
            freezes: RefCell::new(vec![(7, "RDRAM".to_string(), 0x1002, 0x03E7, 2, true)]),
        };
        let table = emusen_session_v1 {
            ctx: (&mut ctx as *mut FakeCheatCtx).cast(),
            info: Some(fake_info),
            spaces: Some(fake_spaces),
            freeze_remove: Some(fake_freeze_remove),
            freeze_list: Some(fake_freeze_list),
            cheat_op: Some(fake_cheat_op),
            cheat_codecs: Some(fake_codecs),
            ..abi::blank_table()
        };
        let mut sh = Shell::new(Config { sessions: false, ..Config::default() });
        sh.add_module(&crate::MODULE).unwrap();
        sh.set_service_table(abi::SERVICE, abi::MAJOR, abi::MINOR, (&table as *const emusen_session_v1).cast()).unwrap();

        let mut run = |line: &str| -> String {
            assert_eq!(sh.submit(0, line).unwrap(), Submitted::Done);
            sh.take_output()
        };

        let legacy = run("cheat poke RDRAM 1004 FF Infinite ammo -w 2 --json");
        assert!(legacy.contains("Infinite ammo -w 2 --json"), "{legacy}");

        let wide_le: Value = serde_json::from_str(&run("cheat poke RDRAM 1008 999 -w 2 --le --dec --json LE HP")).unwrap();
        assert_eq!(wide_le["big_endian"], false);
        assert_eq!(wide_le["native_code"], Value::Null);

        let wide: Value = serde_json::from_str(&run("cheat poke RDRAM 1006 999 -w 2 --be --dec --json Max HP")).unwrap();
        assert_eq!(wide["name"], "Max HP");
        assert_eq!(wide["width"], 2);
        assert_eq!(wide["big_endian"], true);
        assert_eq!(wide["native_code"], "81001006 03E7");

        let from_fz: Value = serde_json::from_str(&run("cheat from-freeze 7 --remove --json Infinite gold")).unwrap();
        assert_eq!(from_fz["name"], "Infinite gold");
        assert_eq!(from_fz["value"], 999);
        assert_eq!(from_fz["native_code"], "81001002 03E7");
        assert!(ctx.freezes.borrow().is_empty());

        let listed: Value = serde_json::from_str(&run("cheat list --json")).unwrap();
        assert_eq!(listed.as_array().unwrap().len(), 4);
        assert_eq!(listed[1]["writes"][0]["big_endian"], false);
        assert_eq!(listed[2]["writes"][0]["width"], 2);
        assert_eq!(listed[2]["writes"][0]["big_endian"], true);
    }
}
