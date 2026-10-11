//! psx-spx's "BIOS Number/String/Character Conversion": the digits, the prefixes 0b, 0x and o that override the base
//! (atoi's octal prefix is a leading 0), and the documented treatment of signs and blanks. Diamond_OpenKernel.md §4.8.

use crate::mem::{rd8, wr};

fn digit(c: u32) -> Option<u32> {
    match c as u8 {
        b'0'..=b'9' => Some(c - b'0' as u32),
        b'A'..=b'Z' => Some(c - b'A' as u32 + 10),
        b'a'..=b'z' => Some(c - b'a' as u32 + 10),
        _ => None,
    }
}

/// A(0Ah).
pub extern "C" fn todigit(c: u32) -> u32 {
    digit(c & 0xFF).unwrap_or(9_999_999)
}

/// A(25h).
pub extern "C" fn toupper(c: u32) -> u32 {
    let c = c & 0xFF;
    if (b'a' as u32..=b'z' as u32).contains(&c) { c - 0x20 } else { c }
}

/// A(26h).
pub extern "C" fn tolower(c: u32) -> u32 {
    let c = c & 0xFF;
    if (b'A' as u32..=b'Z' as u32).contains(&c) { c + 0x20 } else { c }
}

/// A(0Eh) and A(0Fh).
pub extern "C" fn abs(v: u32) -> u32 {
    (v as i32).wrapping_abs() as u32
}

#[derive(Clone, Copy, PartialEq)]
enum Prefixes {
    /// strtol's: 0b, 0x and o.
    Letter,
    /// atoi's: 0b, 0x and a leading 0.
    Zero,
}

/// The number at `src`, and the address after it.
fn parse(src: u32, base: u32, signed: bool, prefixes: Prefixes) -> (u32, u32) {
    let mut p = src;
    while matches!(rd8(p), 0x09..=0x0D | 0x20) {
        p += 1;
    }
    let negative = signed && rd8(p) == b'-' as u32;
    if negative {
        p += 1;
    }
    let mut base = if (2..=36).contains(&base) { base } else { 10 };
    let (c0, c1) = (rd8(p) as u8, rd8(p + 1) as u8);
    match (c0, c1.to_ascii_lowercase()) {
        (b'0', b'b') => {
            base = 2;
            p += 2;
        }
        (b'0', b'x') => {
            base = 16;
            p += 2;
        }
        (b'o' | b'O', _) if prefixes == Prefixes::Letter => {
            base = 8;
            p += 1;
        }
        (b'0', _) if prefixes == Prefixes::Zero => {
            base = 8;
            p += 1;
        }
        _ => {}
    }
    let mut value: u32 = 0;
    while let Some(d) = digit(rd8(p)).filter(|d| *d < base) {
        value = value.wrapping_mul(base).wrapping_add(d);
        p += 1;
    }
    (if negative { value.wrapping_neg() } else { value }, p)
}

/// A(0Dh).
pub extern "C" fn strtol(src: u32, src_end: u32, base: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    let (v, end) = parse(src, base, true, Prefixes::Letter);
    if src_end != 0 {
        wr(src_end, end);
    }
    v
}

/// A(0Ch).
pub extern "C" fn strtoul(src: u32, src_end: u32, base: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    let (v, end) = parse(src, base, false, Prefixes::Letter);
    if src_end != 0 {
        wr(src_end, end);
    }
    v
}

/// A(10h) and A(11h).
pub extern "C" fn atoi(src: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    parse(src, 10, true, Prefixes::Zero).0
}

/// A(12h): the end address returned, the value stored.
pub extern "C" fn atob(src: u32, num_dst: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    let (v, end) = parse(src, 10, true, Prefixes::Letter);
    wr(num_dst, v);
    end
}
