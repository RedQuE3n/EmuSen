//! The NES's two code formats: the plain address:value form and Galoob's Game Genie (Moon_Cheats.md §2).

use emusen_dotnet::text::upper_invariant;
use crate::cheats::{Decoded, format_error, hex, is_hex};

/// The plain form: exactly six hex digits once anything else is stripped.
fn raw_digits(code: &str) -> Option<String> {
    let hex: String = code.chars().filter(|&c| is_hex(c)).collect();
    (hex.len() == 6).then_some(hex)
}

pub fn raw_can_decode(code: &str) -> bool {
    raw_digits(code).is_some()
}

pub fn raw_decode(code: &str) -> Decoded<(i32, u8)> {
    match raw_digits(code) {
        Some(d) => Ok((hex(&d[..4]) as i32, hex(&d[4..]) as u8)),
        None => format_error(format!("'{code}' isn't a valid NES address:value code - expected 6 hex digits (4-digit address + 2-digit value).")),
    }
}

/// A letter's index is the nibble it decodes to.
const ALPHABET: &str = "APZLGITYEOXUKSVN";
/// Which bit of the assembled code each output bit comes from, most significant first.
const ADDRESS_BITS: [u32; 15] = [14, 13, 12, 19, 22, 21, 20, 7, 10, 9, 8, 15, 18, 17, 16];
const VALUE_BITS: [u32; 8] = [3, 6, 5, 4, 23, 2, 1, 0];
const COMPARE_BITS: [u32; 8] = [27, 30, 29, 28, 23, 26, 25, 24];

/// The letters of the alphabet in the code, upper case, where there are six or eight.
fn genie_letters(code: &str) -> Option<Vec<u32>> {
    let letters: Vec<u32> = code.chars().filter_map(|c| ALPHABET.find(upper_invariant(c)).map(|i| i as u32)).collect();
    matches!(letters.len(), 6 | 8).then_some(letters)
}

pub fn genie_can_decode(code: &str) -> bool {
    genie_letters(code).is_some()
}

fn gather(raw: u32, bits: &[u32]) -> u32 {
    bits.iter().fold(0, |r, &b| (r << 1) | ((raw >> b) & 1))
}

/// The address, the value and, for the eight-letter form, the compare byte.
pub fn genie_decode(code: &str) -> Decoded<(i32, u8, Option<u8>)> {
    let Some(letters) = genie_letters(code) else {
        return format_error(format!("'{code}' isn't a valid NES Game Genie code - expected 6 or 8 letters from the alphabet {ALPHABET}."));
    };
    let raw = letters.iter().enumerate().fold(0u32, |r, (i, &n)| r | (n << (i * 4)));
    let mut value_bits = VALUE_BITS;
    let mut compare = None;
    if letters.len() == 8 {
        value_bits[4] = 31;
        compare = Some(gather(raw, &COMPARE_BITS) as u8);
    }
    Ok((gather(raw, &ADDRESS_BITS) as i32 + 0x8000, gather(raw, &value_bits) as u8, compare))
}
