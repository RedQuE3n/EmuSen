//! The consoles' cheat codecs, by codec id: whether a code is a codec's, its one byte, its compare byte and its writes,
//! each as the C# codecs answered and with their words where they refused (EmuSen_Cheats.md, EmuSen_CoreAPI.md §8.2:
//! "the codecs … keyed by system id"). A family's module decodes; this module names the codecs and pairs them per
//! system, and writes the answers as JSON for the facade.

use emusen_dotnet::json::{self, Value};

/// What a decoded code becomes: a write into a memory space, or a patch of the cartridge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    RamPoke,
    RomPatch,
}

/// A codec: its id, the name `cheat add` reports and its kind; a RAM codec writes into its system's `cheat_space`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Codec {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
}

/// Every codec, and each system's pair: the one a code is detected by, and the one asked for by name.
pub const CODECS: [Codec; 11] = [
    Codec { id: "nes.raw", name: "NES address:value", kind: Kind::RamPoke },
    Codec { id: "nes.gg", name: "Game Genie", kind: Kind::RomPatch },
    Codec { id: "snes.par", name: "Pro Action Replay", kind: Kind::RamPoke },
    Codec { id: "snes.gg", name: "Game Genie", kind: Kind::RomPatch },
    Codec { id: "md.ar", name: "Action Replay", kind: Kind::RamPoke },
    Codec { id: "md.gg", name: "Game Genie", kind: Kind::RomPatch },
    Codec { id: "psx.gs", name: "GameShark or Xploder", kind: Kind::RamPoke },
    Codec { id: "psx.xp", name: "Xploder", kind: Kind::RamPoke },
    Codec { id: "gb.gs", name: "GameShark", kind: Kind::RamPoke },
    Codec { id: "gb.gg", name: "Game Genie", kind: Kind::RomPatch },
    Codec { id: "n64.gs", name: "GameShark", kind: Kind::RamPoke },
];

/// Each system's codecs by id: detected, then explicit.
pub const PAIRS: [(&str, Option<&str>, Option<&str>); 9] = [
    ("nes", Some("nes.raw"), Some("nes.gg")),
    ("snes", Some("snes.par"), Some("snes.gg")),
    ("md", Some("md.ar"), Some("md.gg")),
    ("mcd", Some("md.ar"), Some("md.gg")),
    ("32x", Some("md.ar"), Some("md.gg")),
    ("psx", Some("psx.gs"), Some("psx.xp")),
    ("gb", Some("gb.gs"), Some("gb.gg")),
    ("gbc", Some("gb.gs"), Some("gb.gg")),
    ("n64", Some("n64.gs"), None),
];

pub fn codec(id: &str) -> Option<&'static Codec> {
    CODECS.iter().find(|c| c.id == id)
}

/// The space a codec's writes go into: its systems' `cheat_space`; none for a patch.
pub fn space(codec: &Codec) -> Option<&'static str> {
    if codec.kind == Kind::RomPatch {
        return None;
    }
    let system = PAIRS.iter().find(|(_, auto, explicit)| *auto == Some(codec.id) || *explicit == Some(codec.id))?.0;
    crate::find(system)?.cheat_space.as_deref()
}

/// What a write does to whatever is already at the address, `CheatWriteType`'s members in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteType {
    Set,
    Increase,
    Decrease,
    IfEqual,
    IfNotEqual,
    IfLess,
    IfGreater,
}

impl WriteType {
    pub fn is_test(self) -> bool {
        matches!(self, WriteType::IfEqual | WriteType::IfNotEqual | WriteType::IfLess | WriteType::IfGreater)
    }
}

/// One write of a cheat, field for field `CheatWrite`'s, unnormalised: a width or a repeat count is what the codec set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Write {
    pub space: String,
    pub address: i32,
    pub value: u32,
    pub width: i32,
    pub kind: WriteType,
    pub bit_position: Option<i32>,
    pub big_endian: bool,
    pub repeat_count: i32,
    pub repeat_add_address: i32,
    pub repeat_add_value: u32,
}

impl Write {
    /// `CheatWrite.Poke`: one plain write of `width` bytes.
    pub fn poke(space: &str, address: i32, value: u32, width: i32) -> Write {
        Write { space: space.to_string(), address, value, width, kind: WriteType::Set, bit_position: None, big_endian: false, repeat_count: 1, repeat_add_address: 0, repeat_add_value: 0 }
    }

    /// `CheatWrite.Patch`: a cartridge write, its space empty.
    pub fn patch(address: i32, value: u32, width: i32) -> Write {
        Write::poke("", address, value, width)
    }

    pub fn effective_width(&self) -> i32 {
        if self.width == 2 || self.width == 4 { self.width } else { 1 }
    }

    pub fn effective_repeat_count(&self) -> i32 {
        self.repeat_count.max(1)
    }

    /// `CheatWrite.ByteAt`: byte `offset` of `value` in the write's byte order.
    pub fn byte_at(&self, value: u32, offset: i32) -> u8 {
        let width = self.effective_width();
        let shift = if self.big_endian { 8 * (width - 1 - offset) } else { 8 * offset };
        (value >> shift) as u8
    }
}

/// Why a decode gave no answer: the C# codec's `FormatException` and its words, or the `ArgumentOutOfRangeException`
/// it threw indexing an empty list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Format(String),
    Range,
}

pub type Decoded<T> = Result<T, Error>;

pub(crate) fn format_error<T>(words: String) -> Decoded<T> {
    Err(Error::Format(words))
}

/// A code's every answer from one codec, as `ICheatCodeCodec` gives them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub can_decode: bool,
    pub decode: Decoded<(i32, u8)>,
    pub compare: Decoded<Option<u8>>,
    pub writes: Decoded<Option<Vec<Write>>>,
}

/// A code's answers from the codec `id`; none for a codec no family has.
pub fn decode(id: &str, code: &str) -> Option<Answer> {
    use crate::{gb, genesis, n64, nes, psx, snes};
    Some(match codec(id)?.id {
        "nes.raw" => Answer { can_decode: nes::cheats::raw_can_decode(code), decode: nes::cheats::raw_decode(code), compare: Ok(None), writes: Ok(None) },
        "nes.gg" => Answer {
            can_decode: nes::cheats::genie_can_decode(code),
            decode: nes::cheats::genie_decode(code).map(|(a, v, _)| (a, v)),
            compare: nes::cheats::genie_decode(code).map(|(_, _, c)| c),
            writes: Ok(None),
        },
        "snes.par" => Answer {
            can_decode: snes::cheats::is_pro_action_replay(code),
            decode: snes::cheats::decode_pro_action_replay(code),
            compare: Ok(None),
            writes: snes::cheats::pro_action_replay_writes(code).map(Some),
        },
        "snes.gg" => Answer { can_decode: snes::cheats::is_game_genie(code), decode: snes::cheats::decode_game_genie(code), compare: Ok(None), writes: Ok(None) },
        "md.ar" => Answer {
            can_decode: genesis::cheats::is_action_replay(code),
            decode: genesis::cheats::decode_action_replay(code),
            compare: Ok(None),
            writes: genesis::cheats::action_replay_writes(code).map(Some),
        },
        "md.gg" => Answer {
            can_decode: genesis::cheats::is_patch(code),
            decode: genesis::cheats::decode_patch(code),
            compare: Ok(None),
            writes: genesis::cheats::patch_writes(code).map(Some),
        },
        "psx.gs" => {
            let writes = psx::cheats::either_writes(code);
            Answer { can_decode: psx::cheats::is_either(code), decode: writes.clone().and_then(|w| psx::cheats::one_byte(&w, code)), compare: Ok(None), writes: writes.map(Some) }
        }
        "psx.xp" => {
            let writes = psx::cheats::xploder_writes(code);
            Answer { can_decode: psx::cheats::is_xploder(code), decode: writes.clone().and_then(|w| psx::cheats::one_byte(&w, code)), compare: Ok(None), writes: writes.map(Some) }
        }
        "gb.gs" => Answer { can_decode: gb::cheats::shark_can_decode(code), decode: gb::cheats::shark_decode(code), compare: Ok(None), writes: Ok(None) },
        "gb.gg" => Answer { can_decode: gb::cheats::genie_can_decode(code), decode: gb::cheats::genie_decode(code), compare: gb::cheats::genie_compare(code), writes: Ok(None) },
        _ => {
            let writes = n64::cheats::decode_writes(code);
            Answer { can_decode: n64::cheats::can_decode(code), decode: writes.clone().and_then(|w| n64::cheats::one_byte(&w, code)), compare: Ok(None), writes: writes.map(Some) }
        }
    })
}

fn object(members: Vec<(&str, Value)>) -> Value {
    Value::Object(members.into_iter().map(|(k, v)| (json::Text::new(k), v)).collect())
}

fn error(e: &Error) -> Value {
    match e {
        Error::Format(words) => object(vec![("format", Value::string(words.as_str()))]),
        Error::Range => object(vec![("range", Value::Bool(true))]),
    }
}

fn write_json(w: &Write) -> Value {
    object(vec![
        ("space", Value::string(w.space.as_str())),
        ("address", Value::number(w.address)),
        ("value", Value::number(w.value)),
        ("width", Value::number(w.width)),
        ("type", Value::number(w.kind as i32)),
        ("bit", w.bit_position.map_or(Value::Null, Value::number)),
        ("big_endian", Value::Bool(w.big_endian)),
        ("repeat", Value::number(w.repeat_count)),
        ("repeat_address", Value::number(w.repeat_add_address)),
        ("repeat_value", Value::number(w.repeat_add_value)),
    ])
}

/// An answer as JSON: `can`, then `decode` (`address` and `value`), `compare` (a byte or null) and `writes` (a list or
/// null), each of the three an object with `format` (the words) or `range` where it refused.
pub fn answer_json(a: &Answer) -> String {
    let decode = match &a.decode {
        Ok((address, value)) => object(vec![("address", Value::number(*address)), ("value", Value::number(*value))]),
        Err(e) => error(e),
    };
    let compare = match &a.compare {
        Ok(c) => c.map_or(Value::Null, Value::number),
        Err(e) => error(e),
    };
    let writes = match &a.writes {
        Ok(Some(list)) => Value::Array(list.iter().map(write_json).collect()),
        Ok(None) => Value::Null,
        Err(e) => error(e),
    };
    let mut out = String::new();
    json::write_compact(&mut out, &object(vec![("can", Value::Bool(a.can_decode)), ("decode", decode), ("compare", compare), ("writes", writes)]));
    out
}

/// Every codec and every system's pair as JSON: `codecs` (each `id`, `name`, `kind`, `space`) and `systems` (each `id`,
/// `auto`, `explicit`).
pub fn codecs_json() -> String {
    let codecs = CODECS
        .iter()
        .map(|c| {
            object(vec![
                ("id", Value::string(c.id)),
                ("name", Value::string(c.name)),
                ("kind", Value::string(if c.kind == Kind::RamPoke { "RamPoke" } else { "RomPatch" })),
                ("space", space(c).map_or(Value::Null, Value::string)),
            ])
        })
        .collect();
    let systems = PAIRS
        .iter()
        .map(|(id, auto, explicit)| {
            object(vec![
                ("id", Value::string(*id)),
                ("auto", auto.map_or(Value::Null, Value::string)),
                ("explicit", explicit.map_or(Value::Null, Value::string)),
            ])
        })
        .collect();
    let mut out = String::new();
    json::write_compact(&mut out, &object(vec![("codecs", Value::Array(codecs)), ("systems", Value::Array(systems))]));
    out
}

/// Hex digits as `Uri.IsHexDigit` takes them: ASCII only.
pub(crate) fn is_hex(c: char) -> bool {
    c.is_ascii_hexdigit()
}

/// `Convert.ToInt32(digit.ToString(), 16)` for one hex digit.
pub(crate) fn hex_value(c: char) -> u32 {
    c.to_digit(16).unwrap_or(0)
}

/// `uint.Parse(text, NumberStyles.HexNumber)` over hex digits alone, at most eight.
pub(crate) fn hex(text: &str) -> u32 {
    text.chars().fold(0u32, |n, c| n.wrapping_shl(4) | hex_value(c))
}

/// `code.Split('+', RemoveEmptyEntries | TrimEntries)`.
pub(crate) fn parts(code: &str) -> Vec<&str> {
    code.split('+').map(emusen_dotnet::text::trim).filter(|p| !p.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pair_names_codecs_there_are_for_systems_of_the_table() {
        for (system, auto, explicit) in PAIRS {
            assert!(crate::find(system).is_some(), "{system}");
            for id in [auto, explicit].into_iter().flatten() {
                assert!(codec(id).is_some(), "{id}");
            }
        }
        assert_eq!(space(codec("snes.par").unwrap()), Some("CpuBus"));
        assert_eq!(space(codec("snes.gg").unwrap()), None);
    }
}
