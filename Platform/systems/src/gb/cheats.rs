//! The Game Boy's two code formats: the Game Genie, a ROM-read intercept, and the GameShark, a RAM poke
//! (Mercury_Cheats.md §2, §3).

use crate::cheats::{Decoded, format_error, hex, hex_value, is_hex};

/// The code's hex digits, upper case, where there are as many as the format takes.
fn digits(code: &str, lengths: &[usize]) -> Option<Vec<char>> {
    let hex: Vec<char> = code.chars().filter(|&c| is_hex(c)).map(|c| c.to_ascii_uppercase()).collect();
    lengths.contains(&hex.len()).then_some(hex)
}

fn genie_digits(code: &str) -> Decoded<Vec<char>> {
    match digits(code, &[6, 9]) {
        Some(d) => Ok(d),
        None => format_error(format!("'{code}' isn't a valid Game Boy Game Genie code - expected 6 or 9 hex digits (AAA-AAA-AAA).")),
    }
}

pub fn genie_can_decode(code: &str) -> bool {
    digits(code, &[6, 9]).is_some()
}

/// The address is stored with its top nibble last and inverted, which is the whole cipher.
pub fn genie_decode(code: &str) -> Decoded<(i32, u8)> {
    let d = genie_digits(code)?;
    let n = |i: usize| hex_value(d[i]);
    Ok((((n(5) ^ 0x0F) << 12 | n(2) << 8 | n(3) << 4 | n(4)) as i32, (n(0) << 4 | n(1)) as u8))
}

/// A nine-digit code's compare byte: the sixth and ninth digits, rotated and masked; none for six digits.
pub fn genie_compare(code: &str) -> Decoded<Option<u8>> {
    let d = genie_digits(code)?;
    if d.len() != 9 {
        return Ok(None);
    }
    let packed = (hex_value(d[6]) << 4 | hex_value(d[8])) as u8;
    Ok(Some(packed.rotate_right(2) ^ 0xBA))
}

pub fn shark_can_decode(code: &str) -> bool {
    digits(code, &[8]).is_some()
}

/// The address is little-endian in the code, so its halves come out swapped.
pub fn shark_decode(code: &str) -> Decoded<(i32, u8)> {
    let Some(d) = digits(code, &[8]) else {
        return format_error(format!("'{code}' isn't a valid Game Boy GameShark code - expected 8 hex digits (TTVVAAAA)."));
    };
    let text: String = d.iter().collect();
    Ok(((hex(&text[6..8]) << 8 | hex(&text[4..6])) as i32, hex(&text[2..4]) as u8))
}
