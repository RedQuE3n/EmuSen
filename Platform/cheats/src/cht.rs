//! RetroArch's `.cht` format, the C# `ChtFile`: a list of `key = value` lines read into cheats, each an explicit
//! write or a code a console's codec decodes, and cheats written back the same way (`man cheat`, EmuSen_Cheats.md §8).
//! The codecs are emusen-systems', named by id; a code no codec takes, type 0 or a malformed number is skipped.

use emusen_dotnet::text::{equals_ignore_case, fold, is_blank, trim};
use emusen_systems::cheats::{self, Answer, Error, Kind, Write, WriteType};
use std::collections::BTreeSet;
use std::collections::HashMap;

/// RetroArch's `memory_search_size`: 0 to 2 are sub-byte, 3 to 5 are 1, 2 and 4 bytes.
const SIZE_BYTE: i32 = 3;
const SIZE_WORD: i32 = 4;
const SIZE_DWORD: i32 = 5;

/// RetroArch's `cheat_type`; 0 means the entry exists but does nothing.
const TYPE_DISABLED: i32 = 0;
const TYPE_SET: i32 = 1;
const TYPE_INCREASE: i32 = 2;
const TYPE_DECREASE: i32 = 3;

/// One cheat read out of a file: its words, whether the file enabled it, its writes, and whether it patches the cartridge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cheat {
    pub description: String,
    pub enabled: bool,
    pub writes: Vec<Write>,
    pub kind: Kind,
}

/// A file's cheats, and how many entries were there but unusable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Parsed {
    pub cheats: Vec<Cheat>,
    pub skipped: usize,
}

/// What the C# let escape a read: a codec's `ArgumentOutOfRangeException`, which it did not catch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutOfRange;

/// The fields of a file, by key without regard to case; the last of a key wins.
struct Fields(HashMap<String, String>);

impl Fields {
    /// `ReadFields`: blank lines, `#` and `//` lines and lines with no key are left out; a value in quotes loses them.
    fn read(text: &str) -> Fields {
        let mut fields = HashMap::new();
        for raw in text.split('\n') {
            let line = trim(raw);
            if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
                continue;
            }
            let Some(equals) = line.find('=').filter(|&at| at > 0) else { continue };
            let key = trim(&line[..equals]);
            let mut value = trim(&line[equals + 1..]);
            if value.encode_utf16().count() >= 2 && value.starts_with('"') && value.ends_with('"') {
                value = &value[1..value.len() - 1];
            }
            fields.insert(fold(key), value.to_string());
        }
        Fields(fields)
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(&fold(key)).map(String::as_str)
    }

    fn has(&self, key: &str) -> bool {
        self.0.contains_key(&fold(key))
    }

    /// `Long`: `long.TryParse` with the invariant culture, or the fallback.
    fn long(&self, key: &str, fallback: i64) -> i64 {
        self.get(key).and_then(emusen_dotnet::number::parse_i64).unwrap_or(fallback)
    }

    fn int(&self, key: &str, fallback: i32) -> i32 {
        self.long(key, fallback as i64) as i32
    }

    fn is_true(&self, key: &str) -> bool {
        self.get(key).is_some_and(|v| equals_ignore_case(v, "true"))
    }

    /// The entries whose fields are there, by number: every other entry below the count reads nothing and is skipped.
    fn entries(&self) -> BTreeSet<u64> {
        let mut found = BTreeSet::new();
        for key in self.0.keys() {
            let Some(rest) = key.strip_prefix("CHEAT") else { continue };
            let Some((digits, _)) = rest.split_once('_') else { continue };
            if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) || (digits.len() > 1 && digits.starts_with('0')) {
                continue;
            }
            if let Ok(n) = digits.parse::<u64>() {
                found.insert(n);
            }
        }
        found
    }
}

fn answer(codec: &str, code: &str) -> Answer {
    cheats::decode(codec, code).expect("a codec named by its id")
}

/// `code.Split('+', RemoveEmptyEntries | TrimEntries)`.
fn tokens(code: &str) -> Vec<&str> {
    code.split('+').map(trim).filter(|t| !t.is_empty()).collect()
}

/// A codec's answer as the C# caller saw it: a value, a `FormatException` it catches, or a range fault it lets escape.
fn checked<T>(result: Result<T, Error>) -> Result<Option<T>, OutOfRange> {
    match result {
        Ok(v) => Ok(Some(v)),
        Err(Error::Format(_)) => Ok(None),
        Err(Error::Range) => Err(OutOfRange),
    }
}

/// `ChtFile.Parse`, with each codec named by its emusen-systems id.
pub fn parse(text: &str, codec: Option<&str>, space: &str, patch: Option<&str>) -> Result<Parsed, OutOfRange> {
    let fields = Fields::read(text);
    let Some(count) = fields.get("cheats").and_then(emusen_dotnet::number::parse_i32) else { return Ok(Parsed::default()) };
    let mut parsed = Parsed::default();
    if count <= 0 {
        return Ok(parsed);
    }
    let mut read = 0usize;
    for index in fields.entries().into_iter().take_while(|&i| i < count as u64) {
        read += 1;
        match read_cheat(&fields, index, codec, space, patch)? {
            Some(cheat) => parsed.cheats.push(cheat),
            None => parsed.skipped += 1,
        }
    }
    parsed.skipped += count as usize - read;
    Ok(parsed)
}

fn read_cheat(fields: &Fields, index: u64, codec: Option<&str>, space: &str, patch: Option<&str>) -> Result<Option<Cheat>, OutOfRange> {
    let prefix = format!("cheat{index}_");
    let at = |name: &str| format!("{prefix}{name}");
    let description = fields.get(&at("desc")).map_or_else(|| format!("cheat{index}"), str::to_string);
    let enabled = fields.is_true(&at("enable"));
    let code = fields.get(&at("code"));
    let explicit = fields.has(&at("address"));
    let is_patch = !explicit && claims_patch(code, codec, patch);
    let writes = if explicit {
        read_explicit(fields, &prefix, space)
    } else if is_patch {
        read_patch(code.expect("a claimed patch has a code"), patch.expect("a claimed patch has a codec"))?
    } else {
        read_code(code, codec, space)?
    };
    Ok(writes.filter(|w| !w.is_empty()).map(|writes| Cheat { description, enabled, writes, kind: if is_patch { Kind::RomPatch } else { Kind::RamPoke } }))
}

/// Every number in these fields is decimal, unlike the code string's hex.
fn read_explicit(fields: &Fields, prefix: &str, space: &str) -> Option<Vec<Write>> {
    let at = |name: &str| format!("{prefix}{name}");
    let cheat_type = fields.int(&at("cheat_type"), TYPE_SET);
    if cheat_type == TYPE_DISABLED {
        return None;
    }
    let kind = match cheat_type {
        TYPE_INCREASE => WriteType::Increase,
        TYPE_DECREASE => WriteType::Decrease,
        _ => WriteType::Set,
    };
    let address = fields.long(&at("address"), -1);
    if address < 0 {
        return None;
    }
    let search_size = fields.int(&at("memory_search_size"), SIZE_BYTE);
    let width = match search_size {
        SIZE_WORD => 2,
        SIZE_DWORD => 4,
        _ => 1,
    };
    let bit_position = (search_size < SIZE_BYTE).then(|| fields.int(&at("address_bit_position"), 0));
    Some(vec![Write {
        space: space.to_string(),
        address: address as i32,
        value: fields.long(&at("value"), 0) as u32,
        width,
        kind,
        bit_position,
        big_endian: fields.is_true(&at("big_endian")),
        repeat_count: fields.int(&at("repeat_count"), 1),
        repeat_add_address: fields.int(&at("repeat_add_to_address"), 0),
        repeat_add_value: fields.long(&at("repeat_add_to_value"), 0) as u32,
    }])
}

/// A code is a patch when only the patch codec claims it, the rule `cheat add` follows (EmuSen_Cheats.md §8).
fn claims_patch(code: Option<&str>, codec: Option<&str>, patch: Option<&str>) -> bool {
    let (Some(code), Some(patch)) = (code, patch) else { return false };
    if is_blank(Some(code)) || cheats::codec(patch).is_none_or(|c| c.kind != Kind::RomPatch) {
        return false;
    }
    if codec.is_some_and(|c| answer(c, code).can_decode) {
        return false;
    }
    answer(patch, code).can_decode || tokens(code).iter().all(|t| answer(patch, t).can_decode)
}

/// A patch with a compare value has nowhere to keep it here, so it is skipped rather than made wrong.
fn read_patch(code: &str, patch: &str) -> Result<Option<Vec<Write>>, OutOfRange> {
    match checked(answer(patch, code).writes)? {
        None => return Ok(None),
        Some(Some(whole)) => return Ok(Some(whole)),
        Some(None) => {}
    }
    let mut writes = Vec::new();
    for token in tokens(code) {
        let a = answer(patch, token);
        match checked(a.compare)? {
            None | Some(Some(_)) => return Ok(None),
            Some(None) => {}
        }
        let Some((address, value)) = checked(a.decode)? else { return Ok(None) };
        writes.push(Write::patch(address, value as u32, 1));
    }
    Ok(Some(writes))
}

/// The console's own form: one or more codes joined with `+`, the whole first, since one of a codec's codes can span a `+`.
fn read_code(code: Option<&str>, codec: Option<&str>, space: &str) -> Result<Option<Vec<Write>>, OutOfRange> {
    let (Some(code), Some(codec)) = (code, codec) else { return Ok(None) };
    if is_blank(Some(code)) {
        return Ok(None);
    }
    match checked(answer(codec, code).writes)? {
        None => return Ok(None),
        Some(Some(whole)) => return Ok(Some(whole)),
        Some(None) => {}
    }
    let mut writes = Vec::new();
    for token in tokens(code) {
        let a = answer(codec, token);
        if !a.can_decode {
            return Ok(None);
        }
        let Some((address, value)) = checked(a.decode)? else { return Ok(None) };
        writes.push(Write::poke(space, address, value as u32, 1));
    }
    Ok(Some(writes))
}

/// `ChtFile.Write`: one entry a cheat, its first write in the fields and any more riding the code string as RetroArch's do.
pub fn write(cheats: &[Cheat]) -> String {
    let mut out = format!("cheats = {}\n", cheats.len());
    for (i, cheat) in cheats.iter().enumerate() {
        let none = Write { space: String::new(), address: 0, value: 0, width: 0, kind: WriteType::Set, bit_position: None, big_endian: false, repeat_count: 0, repeat_add_address: 0, repeat_add_value: 0 };
        let first = cheat.writes.first().unwrap_or(&none);
        let p = format!("cheat{i}_");
        let cheat_type = match first.kind {
            WriteType::Increase => TYPE_INCREASE,
            WriteType::Decrease => TYPE_DECREASE,
            _ => TYPE_SET,
        };
        let size = if first.bit_position.is_some() {
            0
        } else {
            match first.effective_width() {
                2 => SIZE_WORD,
                4 => SIZE_DWORD,
                _ => SIZE_BYTE,
            }
        };
        out.push('\n');
        out.push_str(&format!("{p}desc = \"{}\"\n", cheat.description.replace('"', "'")));
        out.push_str(&format!("{p}handler = 1\n"));
        out.push_str(&format!("{p}enable = {}\n", if cheat.enabled { "true" } else { "false" }));
        out.push_str(&format!("{p}cheat_type = {cheat_type}\n"));
        out.push_str(&format!("{p}memory_search_size = {size}\n"));
        out.push_str(&format!("{p}address = {}\n", first.address));
        out.push_str(&format!("{p}value = {}\n", first.value));
        out.push_str(&format!("{p}address_bit_position = {}\n", first.bit_position.unwrap_or(0)));
        out.push_str(&format!("{p}big_endian = {}\n", if first.big_endian { "true" } else { "false" }));
        out.push_str(&format!("{p}repeat_count = {}\n", first.effective_repeat_count()));
        out.push_str(&format!("{p}repeat_add_to_address = {}\n", first.repeat_add_address));
        out.push_str(&format!("{p}repeat_add_to_value = {}\n", first.repeat_add_value));
        if cheat.writes.len() > 1 {
            let codes: Vec<String> = cheat.writes.iter().map(|w| format!("{:06X}{:02X}", w.address, w.value)).collect();
            out.push_str(&format!("{p}code = \"{}\"\n", codes.join("+")));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_entries_read_their_decimal_fields_and_a_disabled_type_is_skipped() {
        let text = "cheats = 3\n\ncheat0_desc = \"Lives\"\ncheat0_enable = TRUE\ncheat0_address = 1234\ncheat0_value = 99\ncheat0_memory_search_size = 4\n\
                    CHEAT1_ADDRESS = 5\ncheat1_cheat_type = 0\n# a comment\ncheat2_desc = \"none\"\n";
        let parsed = parse(text, None, "CpuBus", None).unwrap();
        assert_eq!(parsed.skipped, 2);
        let c = &parsed.cheats[0];
        assert_eq!((c.description.as_str(), c.enabled, c.writes[0].address, c.writes[0].value, c.writes[0].width), ("Lives", true, 1234, 99, 2));
    }

    #[test]
    fn a_huge_count_reads_only_the_entries_that_are_there() {
        let parsed = parse("cheats = 2147483647\ncheat7_address = 1\n", None, "CpuBus", None).unwrap();
        assert_eq!((parsed.cheats.len(), parsed.skipped), (1, 2147483646));
        assert_eq!(parse("cheats = -3\ncheat0_address = 1\n", None, "x", None).unwrap(), Parsed::default());
        assert_eq!(parse("cheats = 2\ncheat02_address = 1\ncheat2_address = 1\n", None, "x", None).unwrap().cheats.len(), 0);
    }

    #[test]
    fn codes_decode_through_the_consoles_codecs_and_a_patch_is_told_apart() {
        let text = "cheats = 2\ncheat0_code = \"7E0DBE09\"\ncheat1_code = \"DD62-6DAD\"\n";
        let parsed = parse(text, Some("snes.par"), "CpuBus", Some("snes.gg")).unwrap();
        assert_eq!(parsed.cheats.len(), 2);
        assert_eq!((parsed.cheats[0].kind, parsed.cheats[0].writes[0].address, parsed.cheats[0].writes[0].value), (Kind::RamPoke, 0x0DBE, 9));
        assert_eq!(parsed.cheats[1].kind, Kind::RomPatch);
    }

    #[test]
    fn written_cheats_carry_extra_writes_in_the_code() {
        let cheats = vec![Cheat { description: "A \"q\"".into(), enabled: true, writes: vec![Write::poke("CpuBus", 0x10, 1, 1), Write::poke("CpuBus", -1, 0x1FF, 1)], kind: Kind::RamPoke }];
        let text = write(&cheats);
        assert!(text.starts_with("cheats = 1\n\ncheat0_desc = \"A 'q'\"\ncheat0_handler = 1\n"), "{text}");
        assert!(text.ends_with("cheat0_code = \"00001001+FFFFFFFF1FF\"\n"), "{text}");
    }
}
