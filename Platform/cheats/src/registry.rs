//! The cheats a session applies, the C# `CheatRegistry` (`man cheat`, EmuSen_Cheats.md): RAM pokes rewritten each
//! frame and ROM patches substituted for a read, one toggle a cheat whatever its writes, and a saved list read and
//! written. [`Applied`] is the registry's application at the frame's points, `CoreEngine.RefreshCheats`' division of
//! the pokes into those a core gates itself and those the host writes (EmuSen_Backend.md §29.4, §29.33).

use emusen_dotnet::json::{Text, Value};
use emusen_dotnet::text::{equals_ignore_case, is_blank, trim};
use emusen_systems::cheats::{Kind, Write, WriteType};

/// One cheat as the registry holds it, `CheatInfo`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cheat {
    pub id: i32,
    pub kind: Kind,
    pub writes: Vec<Write>,
    /// A ROM patch's single-byte compare; none is unconditional.
    pub compare: Option<u8>,
    pub description: String,
    pub enabled: bool,
}

/// One byte a ROM patch substitutes, and the cartridge byte it needs there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RomPatchByte {
    pub address: u32,
    pub value: u8,
    pub compare: Option<u8>,
}

#[derive(Clone, Debug)]
pub struct CheatRegistry {
    cheats: Vec<Cheat>,
    next_id: i32,
    master: bool,
    enabled_rom_patches: usize,
    version: i32,
}

impl Default for CheatRegistry {
    fn default() -> Self {
        CheatRegistry { cheats: Vec::new(), next_id: 1, master: true, enabled_rom_patches: 0, version: 0 }
    }
}

fn effective_width(w: &Write) -> i32 {
    w.effective_width()
}

fn first_address(w: &Write) -> i32 {
    let last = w.address.wrapping_add((w.effective_repeat_count() - 1).wrapping_mul(w.repeat_add_address));
    w.address.min(last)
}

fn last_address(w: &Write) -> i32 {
    let last = w.address.wrapping_add((w.effective_repeat_count() - 1).wrapping_mul(w.repeat_add_address));
    w.address.max(last).wrapping_add(effective_width(w) - 1)
}

fn value_at(w: &Write, repetition: i32) -> u32 {
    w.value.wrapping_add((repetition as u32).wrapping_mul(w.repeat_add_value))
}

fn address_at(w: &Write, repetition: i32) -> i32 {
    w.address.wrapping_add(repetition.wrapping_mul(w.repeat_add_address))
}

/// `TryResolveRepetition`: which pass of a write lands on `target`, and at which of its bytes.
fn resolve_repetition(w: &Write, target: i32) -> Option<(i32, i32)> {
    let width = effective_width(w);
    let repeats = w.effective_repeat_count();
    if w.repeat_add_address > 0 {
        let delta = target.wrapping_sub(w.address);
        if delta < 0 {
            return None;
        }
        let repetition = delta / w.repeat_add_address;
        if repetition >= repeats {
            return None;
        }
        let offset = delta - repetition * w.repeat_add_address;
        return (offset < width).then_some((repetition, offset));
    }
    (0..repeats).find_map(|i| {
        let delta = target.wrapping_sub(address_at(w, i));
        (delta >= 0 && delta < width).then_some((i, delta))
    })
}

fn read_wide(read: &mut dyn FnMut(&str, i32) -> u8, w: &Write, address: i32, width: i32) -> u32 {
    let mut value = 0u32;
    for b in 0..width {
        let shift = if w.big_endian { 8 * (width - 1 - b) } else { 8 * b };
        value |= (read(&w.space, address.wrapping_add(b)) as u32) << shift;
    }
    value
}

fn passes(w: &Write, read: &mut dyn FnMut(&str, i32) -> u8) -> bool {
    let width = effective_width(w);
    let mask = if width == 4 { u32::MAX } else { (1u32 << (8 * width)) - 1 };
    let (held, value) = (read_wide(read, w, w.address, width), w.value & mask);
    match w.kind {
        WriteType::IfEqual => held == value,
        WriteType::IfLess => held < value,
        WriteType::IfGreater => held > value,
        _ => held != value,
    }
}

fn apply_write(w: &Write, read: &mut dyn FnMut(&str, i32) -> u8, write: &mut dyn FnMut(&str, i32, u8)) {
    let width = effective_width(w);
    for i in 0..w.effective_repeat_count() {
        let address = address_at(w, i);
        let value = value_at(w, i);
        if let Some(bit) = w.bit_position {
            let mask = 1u8 << (bit & 7);
            let current = read(&w.space, address);
            write(&w.space, address, if value & 1 != 0 { current | mask } else { current & !mask });
            continue;
        }
        let target = match w.kind {
            WriteType::Increase => read_wide(read, w, address, width).wrapping_add(value),
            WriteType::Decrease => read_wide(read, w, address, width).wrapping_sub(value),
            _ => value,
        };
        for b in 0..width {
            write(&w.space, address.wrapping_add(b), w.byte_at(target, b));
        }
    }
}

impl CheatRegistry {
    pub fn new() -> CheatRegistry {
        CheatRegistry::default()
    }

    /// The one switch over every cheat, runtime-only and never saved.
    pub fn master_enabled(&self) -> bool {
        self.master
    }

    pub fn set_master_enabled(&mut self, on: bool) {
        self.master = on;
        self.recount();
    }

    /// Moved by every change, so the application knows when to send the patch table and pokes again.
    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn enabled_rom_patches(&self) -> usize {
        self.enabled_rom_patches
    }

    fn recount(&mut self) {
        self.enabled_rom_patches = if self.master { self.cheats.iter().filter(|c| c.kind == Kind::RomPatch && c.enabled).count() } else { 0 };
        self.version = self.version.wrapping_add(1);
    }

    pub fn add_ram_poke(&mut self, space: &str, address: i32, value: u8, description: &str, enabled: bool) -> Result<i32, String> {
        self.add(Kind::RamPoke, vec![Write::poke(space, address, value as u32, 1)], None, description, enabled)
    }

    pub fn add_rom_patch(&mut self, address: i32, value: u8, compare: Option<u8>, description: &str, enabled: bool) -> Result<i32, String> {
        self.add(Kind::RomPatch, vec![Write::patch(address, value as u32, 1)], compare, description, enabled)
    }

    /// `AddCheat`: refused, with `ArgumentException`'s message, for no write and for the combinations that cannot be
    /// honestly implemented.
    pub fn add(&mut self, kind: Kind, writes: Vec<Write>, compare: Option<u8>, description: &str, enabled: bool) -> Result<i32, String> {
        if writes.is_empty() {
            return Err("A cheat needs at least one write. (Parameter 'writes')".to_string());
        }
        if kind == Kind::RomPatch {
            if writes.iter().any(|w| w.kind != WriteType::Set) {
                return Err("A ROM patch can only set a value - increase/decrease needs somewhere to accumulate, and a ROM read substitution has none.".to_string());
            }
            if compare.is_some() && writes.iter().any(|w| effective_width(w) != 1) {
                return Err("A compare byte only applies to a single-byte ROM patch.".to_string());
            }
        }
        if writes.iter().any(|w| w.kind.is_test() && (w.bit_position.is_some() || w.effective_repeat_count() > 1)) {
            return Err("A test compares one value at one address - a bit position or a repeat run on it has no meaning.".to_string());
        }
        let id = self.next_id;
        self.next_id += 1;
        self.cheats.push(Cheat { id, kind, writes, compare, description: description.to_string(), enabled });
        self.recount();
        Ok(id)
    }

    pub fn remove(&mut self, id: i32) -> bool {
        let before = self.cheats.len();
        self.cheats.retain(|c| c.id != id);
        if self.cheats.len() == before {
            return false;
        }
        self.recount();
        true
    }

    pub fn set_enabled(&mut self, id: i32, enabled: bool) -> bool {
        let Some(c) = self.cheats.iter_mut().find(|c| c.id == id) else { return false };
        c.enabled = enabled;
        self.recount();
        true
    }

    pub fn clear(&mut self) {
        self.cheats.clear();
        self.recount();
    }

    pub fn cheats(&self) -> &[Cheat] {
        &self.cheats
    }

    /// `ApplyAll`: every enabled RAM poke written, a failed test holding until the next write that is not a test.
    pub fn apply_all(&self, read: &mut dyn FnMut(&str, i32) -> u8, write: &mut dyn FnMut(&str, i32, u8)) {
        if !self.master {
            return;
        }
        for c in &self.cheats {
            if c.kind != Kind::RamPoke || !c.enabled {
                continue;
            }
            let mut skip = false;
            for w in &c.writes {
                if w.kind.is_test() {
                    if !skip && !passes(w, read) {
                        skip = true;
                    }
                    continue;
                }
                if skip {
                    skip = false;
                    continue;
                }
                apply_write(w, read, write);
            }
        }
    }

    /// `TryPatchRom`: the first enabled patch over `address` whose compare holds.
    pub fn try_patch_rom(&self, address: u32, original: u8) -> Option<u8> {
        if self.enabled_rom_patches == 0 {
            return None;
        }
        let target = address as i32;
        for c in &self.cheats {
            if c.kind != Kind::RomPatch || !c.enabled || c.compare.is_some_and(|x| x != original) {
                continue;
            }
            for w in &c.writes {
                if target < first_address(w) || target > last_address(w) {
                    continue;
                }
                if let Some((repetition, offset)) = resolve_repetition(w, target) {
                    return Some(w.byte_at(value_at(w, repetition), offset));
                }
            }
        }
        None
    }

    /// `ResolveRomPatches`: every byte `try_patch_rom` would substitute below `limit`, in the order it tries them.
    pub fn resolve_rom_patches(&self, limit: i64) -> Vec<RomPatchByte> {
        let mut bytes = Vec::new();
        if self.enabled_rom_patches == 0 {
            return bytes;
        }
        for c in &self.cheats {
            if c.kind != Kind::RomPatch || !c.enabled {
                continue;
            }
            for w in &c.writes {
                let mut seen = std::collections::HashSet::new();
                for i in 0..w.effective_repeat_count() {
                    for b in 0..effective_width(w) {
                        let target = address_at(w, i).wrapping_add(b);
                        if target < 0 || target as i64 >= limit || !seen.insert(target) {
                            continue;
                        }
                        if let Some((repetition, offset)) = resolve_repetition(w, target) {
                            bytes.push(RomPatchByte { address: target as u32, value: w.byte_at(value_at(w, repetition), offset), compare: c.compare });
                        }
                    }
                }
            }
        }
        bytes
    }

    /// `ToCheatFile`: the list as `cheats/<game>.json` holds it, the canonical JSON Galaxia's model binds.
    pub fn to_file(&self) -> Value {
        let text = |s: &str| Value::string(s);
        let entries = self
            .cheats
            .iter()
            .map(|c| {
                let rom = c.kind == Kind::RomPatch;
                let writes = c
                    .writes
                    .iter()
                    .map(|w| {
                        object(vec![
                            ("Space", text(&w.space)),
                            ("Address", text(&hex(w.address as u32, if rom { 6 } else { 1 }))),
                            ("Value", text(&hex(w.value, (effective_width(w) * 2) as usize))),
                            ("Width", Value::number(effective_width(w))),
                            ("Type", text(type_name(w.kind))),
                            ("BitPosition", w.bit_position.map_or(Value::Null, Value::number)),
                            ("BigEndian", Value::Bool(w.big_endian)),
                            ("RepeatCount", Value::number(w.effective_repeat_count())),
                            ("RepeatAddAddress", text(&hex(w.repeat_add_address as u32, 1))),
                            ("RepeatAddValue", text(&hex(w.repeat_add_value, 1))),
                        ])
                    })
                    .collect();
                object(vec![
                    ("Kind", text(if rom { "RomPatch" } else { "RamPoke" })),
                    ("Writes", Value::Array(writes)),
                    ("Compare", c.compare.map_or(Value::Null, |x| text(&format!("{x:02X}")))),
                    ("Description", text(&c.description)),
                    ("Enabled", Value::Bool(c.enabled)),
                    ("Space", Value::Null),
                    ("Address", Value::Null),
                    ("Value", Value::Null),
                ])
            })
            .collect();
        object(vec![("Cheats", Value::Array(entries))])
    }

    /// `LoadFrom`: a saved list's cheats added, one bad entry skipped and the rest loaded; how many of each.
    pub fn load(&mut self, file: &Value) -> (usize, usize) {
        let (mut loaded, mut skipped) = (0, 0);
        let entries = match file.get("Cheats") {
            Some(Value::Array(e)) => e.as_slice(),
            _ => &[],
        };
        for e in entries {
            let Some((kind, writes, compare)) = read_entry(e) else {
                skipped += 1;
                continue;
            };
            let description = e.get("Description").and_then(Value::as_str).unwrap_or("");
            let enabled = e.get("Enabled").and_then(Value::as_bool).unwrap_or(true);
            match self.add(kind, writes, compare, description, enabled) {
                Ok(_) => loaded += 1,
                Err(_) => skipped += 1,
            }
        }
        (loaded, skipped)
    }
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (Text::new(k), v)).collect())
}

/// `CheatFileWrite.Hex`: `value.ToString("X" + digits)`.
fn hex(value: u32, digits: usize) -> String {
    format!("{value:0digits$X}")
}

fn type_name(kind: WriteType) -> &'static str {
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

/// `TryParseWriteType`: absent or blank is `Set`; otherwise a member's name or number, ignoring case, as
/// `Enum.TryParse` takes it.
pub fn parse_write_type(text: Option<&str>) -> Option<WriteType> {
    let Some(t) = text.filter(|t| !is_blank(Some(t))) else { return Some(WriteType::Set) };
    let t = trim(t);
    const ALL: [WriteType; 7] = [WriteType::Set, WriteType::Increase, WriteType::Decrease, WriteType::IfEqual, WriteType::IfNotEqual, WriteType::IfLess, WriteType::IfGreater];
    if let Some(k) = ALL.iter().find(|k| equals_ignore_case(type_name(**k), t)) {
        return Some(*k);
    }
    // Enum.TryParse takes a number too, any int; one outside the members is kept by .NET as its value, which no
    // write type has, and the write then acts as a plain set of its value.
    emusen_dotnet::number::parse_i32(t).map(|n| ALL.get(n as usize).copied().unwrap_or(WriteType::Set))
}

/// `CheatFileWrite.TryHex`: hex text with an optional 0x or $, as an unsigned 32-bit number.
fn try_hex(text: Option<&str>) -> Option<u32> {
    let t = text.filter(|t| !is_blank(Some(t)))?;
    let t = trim(t);
    let stripped = if t.len() >= 2 && t[..2].eq_ignore_ascii_case("0x") {
        &t[2..]
    } else if let Some(rest) = t.strip_prefix('$') {
        rest
    } else {
        t
    };
    hex_number(stripped, 8)
}

/// `uint.TryParse(text, NumberStyles.HexNumber)` and its byte form: white space around, leading zeros allowed.
fn hex_number(text: &str, digits: usize) -> Option<u32> {
    let d = trim(text);
    if d.is_empty() || !d.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let significant = d.trim_start_matches('0');
    if significant.len() > digits {
        return None;
    }
    u32::from_str_radix(if significant.is_empty() { "0" } else { significant }, 16).ok()
}

/// `TryReadEntry`: a saved cheat's writes and compare, or none for an entry that is broken.
fn read_entry(e: &Value) -> Option<(Kind, Vec<Write>, Option<u8>)> {
    let compare = match e.get("Compare").and_then(Value::as_str) {
        None => None,
        Some(c) if is_blank(Some(c)) || trim(c) == "-" => None,
        Some(c) => {
            let t = trim(c);
            let stripped = if t.len() >= 2 && t[..2].eq_ignore_ascii_case("0x") { &t[2..] } else { t.strip_prefix('$').unwrap_or(t) };
            Some(hex_number(stripped, 2)? as u8)
        }
    };
    let kind = if e.get("Kind").and_then(Value::as_str).is_some_and(|k| equals_ignore_case(k, "RomPatch")) { Kind::RomPatch } else { Kind::RamPoke };
    let listed: Vec<Value> = match e.get("Writes") {
        Some(Value::Array(w)) if !w.is_empty() => w.clone(),
        _ => match e.get("Address").and_then(Value::as_str) {
            None => Vec::new(),
            Some(address) => vec![object(vec![
                ("Space", Value::string(e.get("Space").and_then(Value::as_str).unwrap_or(""))),
                ("Address", Value::string(address)),
                ("Value", Value::string(e.get("Value").and_then(Value::as_str).unwrap_or("00"))),
            ])],
        },
    };
    if listed.is_empty() {
        return None;
    }
    let mut writes = Vec::new();
    for w in &listed {
        let s = |name: &str, fallback: &'static str| -> Option<String> {
            match w.get(name) {
                Some(Value::String(t)) => Some(t.text.clone()),
                Some(Value::Null) => None,
                _ => Some(fallback.to_string()),
            }
        };
        let int = |name: &str, fallback: i64| -> i64 { number(w.get(name)).unwrap_or(fallback) };
        let address = try_hex(s("Address", "0").as_deref())?;
        let value = try_hex(s("Value", "00").as_deref())?;
        let repeat_address = s("RepeatAddAddress", "0");
        let repeat_value = s("RepeatAddValue", "0");
        let repeat_address = if is_blank(repeat_address.as_deref()) { 0 } else { try_hex(repeat_address.as_deref())? };
        let repeat_value = if is_blank(repeat_value.as_deref()) { 0 } else { try_hex(repeat_value.as_deref())? };
        let kind = parse_write_type(s("Type", "Set").as_deref())?;
        writes.push(Write {
            space: s("Space", "").unwrap_or_default(),
            address: address as i32,
            value,
            width: int("Width", 1) as i32,
            kind,
            bit_position: match w.get("BitPosition") {
                Some(Value::Number(_)) => number(w.get("BitPosition")).map(|b| b as i32),
                _ => None,
            },
            big_endian: w.get("BigEndian").and_then(Value::as_bool).unwrap_or(false),
            repeat_count: int("RepeatCount", 1) as i32,
            repeat_add_address: repeat_address as i32,
            repeat_add_value: repeat_value,
        });
    }
    Some((kind, writes, compare))
}

/// The registry's application, `CoreEngine.RefreshCheats`: the ROM patches within the machine's range as triples,
/// each simple enabled poke as a quad the core gates itself, and the rest kept for the host to write at the frame's end.
#[derive(Default)]
pub struct Applied {
    /// The registry's version last sent; none sends at the next refresh.
    sent: Option<i32>,
    /// The pokes the host writes, the whole registry where the core takes no pokes.
    pub host: CheatRegistry,
    /// Whether the host's registry is the session's own, as when the core claims no pokes.
    pub host_is_all: bool,
    pub triples: Vec<u32>,
    pub quads: Vec<u32>,
}

/// What a refresh asks of the machine: the patch table, the pokes, or nothing.
#[derive(Debug, PartialEq, Eq)]
pub enum Send<'a> {
    Nothing,
    /// Triples to `set_rom_patches` where the machine has a patch range, quads to `set_cheat_pokes` where it takes them.
    Tables { triples: Option<&'a [u32]>, quads: Option<&'a [u32]> },
}

impl Applied {
    /// `_cheatVersion = -1`: the next refresh sends again.
    pub fn stale(&mut self) {
        self.sent = None;
    }

    /// `RefreshCheats`: nothing when the version has not moved; else the tables, with `space_of` naming a space's id.
    pub fn refresh<'a>(&'a mut self, cheats: &CheatRegistry, patch_range: Option<(i64, i64)>, takes_pokes: bool, space_of: &dyn Fn(&str) -> Option<u32>) -> Send<'a> {
        let version = cheats.version();
        if self.sent == Some(version) {
            return Send::Nothing;
        }
        self.sent = Some(version);
        self.triples.clear();
        let has_range = patch_range.is_some();
        if let Some((low, high)) = patch_range {
            for patch in cheats.resolve_rom_patches(high + 1) {
                if (patch.address as i64) < low {
                    continue;
                }
                self.triples.extend([patch.address, patch.value as u32, patch.compare.map_or(u32::MAX, u32::from)]);
            }
        }
        if !takes_pokes {
            self.host_is_all = true;
            return Send::Tables { triples: has_range.then_some(&self.triples[..]), quads: None };
        }
        self.host_is_all = false;
        self.quads.clear();
        let mut host = CheatRegistry::new();
        host.set_master_enabled(cheats.master_enabled());
        for c in cheats.cheats() {
            if c.kind != Kind::RamPoke || !c.enabled {
                continue;
            }
            let spaces: Vec<Option<u32>> = c.writes.iter().map(|w| space_of(&w.space)).collect();
            let simple = cheats.master_enabled()
                && c.writes.iter().zip(&spaces).all(|(w, s)| s.is_some() && w.kind == WriteType::Set && effective_width(w) == 1 && w.bit_position.is_none() && w.repeat_count <= 1);
            if simple {
                for (w, s) in c.writes.iter().zip(&spaces) {
                    self.quads.extend([s.unwrap_or(0), w.address as u32, w.value & 0xFF, c.compare.map_or(u32::MAX, u32::from)]);
                }
            } else {
                let _ = host.add(c.kind, c.writes.clone(), c.compare, &c.description, true);
            }
        }
        self.host = host;
        Send::Tables { triples: has_range.then_some(&self.triples[..]), quads: Some(&self.quads[..]) }
    }

    /// The registry the host writes at the frame's end: the session's own where the core takes no pokes.
    pub fn host<'a>(&'a self, cheats: &'a CheatRegistry) -> &'a CheatRegistry {
        if self.host_is_all { cheats } else { &self.host }
    }
}

fn number(v: Option<&Value>) -> Option<i64> {
    match v {
        Some(Value::Number(n)) => n.parse().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_simple_poke_goes_to_the_core_and_the_rest_stay_the_hosts() {
        let mut c = CheatRegistry::new();
        c.add_ram_poke("RAM", 4, 9, "simple", true).unwrap();
        c.add(Kind::RamPoke, vec![Write { width: 2, ..Write::poke("RAM", 6, 0x102, 2) }], None, "wide", true).unwrap();
        c.add_ram_poke("NOPE", 1, 1, "no space", true).unwrap();
        c.add_rom_patch(0x10, 0xEA, Some(0x20), "patch", true).unwrap();
        let mut applied = Applied::default();
        let space_of = |s: &str| (s == "RAM").then_some(0u32);
        assert_eq!(applied.refresh(&c, Some((0, 0xFF)), true, &space_of), Send::Tables { triples: Some(&[0x10, 0xEA, 0x20]), quads: Some(&[0, 4, 9, u32::MAX]) });
        assert_eq!(applied.host(&c).cheats().iter().map(|x| x.description.as_str()).collect::<Vec<_>>(), ["wide", "no space"]);
        assert_eq!(applied.refresh(&c, Some((0, 0xFF)), true, &space_of), Send::Nothing);
        applied.stale();
        assert!(matches!(applied.refresh(&c, None, false, &space_of), Send::Tables { triples: None, quads: None }));
        assert_eq!(applied.host(&c).cheats().len(), 4);
    }
}
