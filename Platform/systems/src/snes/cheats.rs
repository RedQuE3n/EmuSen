//! The SNES's Pro Action Replay and Game Genie codes, as fullsnes's "SNES Cart Cheat Devices - Code Formats" gives them
//! (EmuSen_CoreAPI.md §20).

use crate::cheats::{Decoded, Write, format_error, hex, is_hex, parts};
use emusen_dotnet::text::{is_white_space, trim, upper_invariant};

/// The space the work RAM's codes write into, beside the CPU bus the codec names.
const WRAM: &str = "WRAM";
const CPU_BUS: &str = "CpuBus";

/// Spaces, dashes and colons between digits are how codes are printed, not part of them.
fn digits(code: &str) -> String {
    code.chars().filter(|&c| !is_white_space(c) && c != '-' && c != ':').collect()
}

fn all_hex(s: &str) -> bool {
    !s.is_empty() && s.chars().all(is_hex)
}

/// The device's "do nothing" padding code.
const DO_NOTHING: u32 = 0x7E00_0000;

/// Eight hex digits printed as one word, less the pre-boot codes and the device's own prefixes.
pub fn is_pro_action_replay(code: &str) -> bool {
    let d = trim(code);
    if d.chars().count() != 8 || !all_hex(d) {
        return false;
    }
    let word = hex(d);
    !matches!(word >> 24, 0xFE | 0xFF) && word != 0xDEAD_C0DE && word >> 16 != 0xC0DE
}

pub fn decode_pro_action_replay(code: &str) -> Decoded<(i32, u8)> {
    let d = digits(code);
    if d.chars().count() != 8 || !all_hex(&d) {
        return format_error(format!("'{code}' is not eight hex digits (a Pro Action Replay code is AAAAAADD)."));
    }
    let word = hex(&d);
    Ok(((word >> 8) as i32, word as u8))
}

/// Each part a write: the work RAM's at 7E0000-7FFFFF, the CPU bus's otherwise; the padding code left out.
pub fn pro_action_replay_writes(code: &str) -> Decoded<Vec<Write>> {
    let mut writes = Vec::new();
    for part in parts(code) {
        let (address, value) = decode_pro_action_replay(part)?;
        if ((address as u32) << 8 | value as u32) == DO_NOTHING {
            continue;
        }
        let wram = (0x7E_0000..=0x7F_FFFF).contains(&address);
        let mut w = Write::poke(if wram { WRAM } else { CPU_BUS }, if wram { address - 0x7E_0000 } else { address }, value as u32, 1);
        w.repeat_count = 0;
        writes.push(w);
    }
    Ok(writes)
}

/// Genie digits in the order of the hex digits they stand for.
const GENIE_DIGITS: &str = "DF4709156BC8A23E";
/// Bit 23 first: where each address bit sits in the code's six address digits, and where it belongs in the address.
const GENIE_ORDER: &str = "ijklqrstopabcduvwxefghmn";
const ADDRESS_ORDER: &str = "abcdefghijklmnopqrstuvwx";

/// The printed form, four digits, a dash, four digits, so that eight bare digits stay the Action Replay's.
pub fn is_game_genie(code: &str) -> bool {
    let t: Vec<char> = trim(code).chars().collect();
    t.len() == 9 && t[4] == '-' && t[..4].iter().all(|&c| is_hex(c)) && t[5..].iter().all(|&c| is_hex(c))
}

fn genie_position(label: char) -> u32 {
    GENIE_ORDER.find(label).unwrap_or(0) as u32
}

pub fn decode_game_genie(code: &str) -> Decoded<(i32, u8)> {
    let d: Vec<char> = digits(code).chars().map(upper_invariant).collect();
    if d.len() != 8 || !d.iter().all(|&c| GENIE_DIGITS.contains(c)) {
        return format_error(format!("'{code}' is not a Game Genie code (DDAA-AAAA in Genie digits)."));
    }
    let word = d.iter().fold(0u32, |w, &c| (w << 4) | GENIE_DIGITS.find(c).unwrap_or(0) as u32);
    let genie = word & 0xFF_FFFF;
    let mut address = 0u32;
    for (i, label) in ADDRESS_ORDER.chars().enumerate() {
        if (genie >> (23 - genie_position(label))) & 1 != 0 {
            address |= 1 << (23 - i);
        }
    }
    Ok((address as i32, (word >> 24) as u8))
}

/// The inverse, for showing a patch as a code.
pub fn encode_game_genie(address: i32, value: u8) -> String {
    let mut genie = 0u32;
    for (i, label) in ADDRESS_ORDER.chars().enumerate() {
        if ((address as u32) >> (23 - i)) & 1 != 0 {
            genie |= 1 << (23 - genie_position(label));
        }
    }
    let word = (value as u32) << 24 | genie;
    let d: Vec<char> = (0..8).map(|i| GENIE_DIGITS.as_bytes()[((word >> (28 - 4 * i)) & 0xF) as usize] as char).collect();
    format!("{}-{}", d[..4].iter().collect::<String>(), d[4..].iter().collect::<String>())
}
