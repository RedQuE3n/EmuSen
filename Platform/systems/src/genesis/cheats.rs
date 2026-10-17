//! The Genesis family's Game Genie and Action Replay codes; the formats, their sources and the argued choices are
//! EmuSen_Cheats.md §9.

use crate::cheats::{Decoded, Error, Write, format_error, hex, parts};
use emusen_dotnet::text::{trim, upper_invariant};

const WRAM: &str = "WRAM";

/// The digits in the order of the values they stand for, 0 to 31.
const GENIE_DIGITS: &str = "ABCDEFGHJKLMNPRSTVWXYZ0123456789";
/// Each of the forty bits in the code's order: A-X are the address's bits from 23 down, a-p the word's from 15 down.
const GENIE_ORDER: &str = "ijklmnopIJKLMNOPABCDEFGHdefghabcQRSTUVWX";

/// `code.Trim().ToUpperInvariant()`, where only what can become ASCII matters to the patterns that follow.
fn upper_trimmed(code: &str) -> Vec<char> {
    trim(code).chars().map(upper_invariant).collect()
}

fn alnum(c: char) -> bool {
    c.is_ascii_uppercase() || c.is_ascii_digit()
}

/// `^[A-Z0-9]{4}-[A-Z0-9]{4}$` over the upper-cased code, every digit the Genie's.
pub fn is_game_genie(code: &str) -> bool {
    let t = upper_trimmed(code);
    t.len() == 9 && t[4] == '-' && t[..4].iter().chain(&t[5..]).all(|&c| alnum(c)) && t.iter().filter(|&&c| c != '-').all(|&c| GENIE_DIGITS.contains(c))
}

pub fn decode_game_genie(code: &str) -> Decoded<(i32, u16)> {
    let d: Vec<char> = upper_trimmed(code).into_iter().filter(|&c| c != '-').collect();
    if d.len() != 8 || !d.iter().all(|&c| GENIE_DIGITS.contains(c)) {
        return format_error(format!("'{code}' is not a Game Genie code (ABCD-EFGH in the Genie's digits, without I, O, Q or U)."));
    }
    let bits = d.iter().fold(0u64, |b, &c| (b << 5) | GENIE_DIGITS.find(c).unwrap_or(0) as u64);
    let (mut address, mut value) = (0u32, 0u32);
    for (i, label) in GENIE_ORDER.chars().enumerate() {
        if (bits >> (39 - i)) & 1 == 0 {
            continue;
        }
        if label.is_ascii_uppercase() {
            address |= 1 << (23 - (label as u32 - 'A' as u32));
        } else {
            value |= 1 << (15 - (label as u32 - 'a' as u32));
        }
    }
    Ok((address as i32, value as u16))
}

/// The inverse, for showing a patch as a code.
pub fn encode_game_genie(address: i32, value: u16) -> String {
    let mut bits = 0u64;
    for (i, label) in GENIE_ORDER.chars().enumerate() {
        let bit = if label.is_ascii_uppercase() { (address >> (23 - (label as i32 - 'A' as i32))) & 1 } else { ((value as i32) >> (15 - (label as i32 - 'a' as i32))) & 1 };
        bits |= (bit as u64 & 1) << (39 - i);
    }
    let d: Vec<char> = (0..8).map(|i| GENIE_DIGITS.as_bytes()[((bits >> (35 - 5 * i)) & 31) as usize] as char).collect();
    format!("{}-{}", d[..4].iter().collect::<String>(), d[4..].iter().collect::<String>())
}

fn is_hex_upper(c: char) -> bool {
    c.is_ascii_digit() || ('A'..='F').contains(&c)
}

fn hex_run(t: &[char]) -> bool {
    t.iter().all(|&c| is_hex_upper(c))
}

/// The address, the value and its width in bytes, from `AAAAAA:V…`, `AAAAA AVVVV` or the bare eight or ten digits.
pub fn try_parse_action_replay(code: &str) -> Option<(i32, u32, i32)> {
    let t = upper_trimmed(code);
    let text = |s: &[char]| s.iter().collect::<String>();
    let (a, v): (String, String) = if t.len() >= 8 && t[6] == ':' && hex_run(&t[..6]) && matches!(t.len() - 7, 1..=4 | 8) && hex_run(&t[7..]) {
        (text(&t[..6]), text(&t[7..]))
    } else if t.len() == 11 && hex_run(&t[..5]) && matches!(t[5], ' ' | '-') && hex_run(&t[6..]) {
        (text(&t[..5]) + &text(&t[6..7]), text(&t[7..]))
    } else if matches!(t.len(), 8 | 10) && hex_run(&t) {
        (text(&t[..6]), text(&t[6..]))
    } else {
        return None;
    };
    let width = if v.len() <= 2 { 1 } else if v.len() <= 4 { 2 } else { 4 };
    Some((hex(&a) as i32, hex(&v), width))
}

pub fn is_ram(address: i32) -> bool {
    (0xE0_0000..=0xFF_FFFF).contains(&address)
}

pub fn is_cartridge(address: i32) -> bool {
    (0..=0x3F_FFFF).contains(&address)
}

fn for_ram(part: &str) -> bool {
    try_parse_action_replay(part).is_some_and(|(a, _, _)| is_ram(a))
}

fn for_cartridge(part: &str) -> bool {
    try_parse_action_replay(part).is_some_and(|(a, _, _)| is_cartridge(a))
}

/// Action Replay codes for the main RAM, which the RAM's mirrors from $E00000 all reach.
pub fn is_action_replay(code: &str) -> bool {
    let p = parts(code);
    !p.is_empty() && p.iter().all(|p| for_ram(p))
}

pub fn action_replay_writes(code: &str) -> Decoded<Vec<Write>> {
    parts(code)
        .into_iter()
        .map(|p| match try_parse_action_replay(p) {
            Some((address, value, width)) if is_ram(address) => {
                let mut w = Write::poke(WRAM, address & 0xFFFF, value, width);
                w.big_endian = true;
                Ok(w)
            }
            _ => format_error(format!("'{p}' is not an Action Replay code for the main RAM (AAAAAA:VVVV at $E00000-$FFFFFF).")),
        })
        .collect()
}

/// The first write's address and its first byte; the C# indexed an empty list for a code of no parts.
fn first_byte(writes: Decoded<Vec<Write>>) -> Decoded<(i32, u8)> {
    let writes = writes?;
    let w = writes.first().ok_or(Error::Range)?;
    Ok((w.address, w.byte_at(w.value, 0)))
}

pub fn decode_action_replay(code: &str) -> Decoded<(i32, u8)> {
    first_byte(action_replay_writes(code))
}

/// Each Game Genie code is the word on the 16-bit bus at its address, which has no line for the address's lowest bit.
pub fn patch_writes(code: &str) -> Decoded<Vec<Write>> {
    parts(code)
        .into_iter()
        .map(|p| {
            let mut w = if is_game_genie(p) {
                let (address, value) = decode_game_genie(p)?;
                Write::patch(address & 0x3F_FFFE, value as u32, 2)
            } else if let Some((a, v, width)) = try_parse_action_replay(p).filter(|&(a, _, _)| is_cartridge(a)) {
                Write::patch(a, v, width)
            } else {
                return format_error(format!("'{p}' is not a Game Genie code or an Action Replay code for the cartridge."));
            };
            w.big_endian = true;
            Ok(w)
        })
        .collect()
}

pub fn is_patch(code: &str) -> bool {
    let p = parts(code);
    !p.is_empty() && p.iter().all(|p| is_game_genie(p) || for_cartridge(p))
}

pub fn decode_patch(code: &str) -> Decoded<(i32, u8)> {
    first_byte(patch_writes(code))
}
