//! File names against the patterns psx-spx's "B(42h) - firstfile2" describes: "?" one character, "*" the rest (in
//! firstfile2 and nextfile only), and its documented bug, that a "?" meeting a name's end ends the comparison.

use crate::mem::{rd8, wr8};

fn fold(c: u32, insensitive: bool) -> u32 {
    if insensitive { crate::convert::toupper(c) & 0xFF } else { c }
}

/// Whether the name at `name` (ended by 0, or by `end` when it is not 0) matches the pattern at `pattern`.
pub fn matches(pattern: u32, name: u32, end: u32, star: bool, insensitive: bool) -> bool {
    let mut i = 0;
    loop {
        let p = rd8(pattern + i);
        let mut n = rd8(name + i);
        if end != 0 && n == end {
            n = 0;
        }
        match p {
            0 => return n == 0,
            0x2A if star => return true,
            0x3F if n == 0 => return true,
            0x3F => {}
            _ if fold(p, insensitive) != fold(n, insensitive) => return false,
            _ => {}
        }
        i += 1;
    }
}

/// The path past any leading backslashes.
pub fn strip(path: u32) -> u32 {
    let mut p = path;
    while rd8(p) == b'\\' as u32 {
        p += 1;
    }
    p
}

/// At most `max` bytes of a name copied to `dst`, the rest of `size` bytes zero.
pub fn copy(dst: u32, src: u32, max: u32, size: u32) {
    let mut ended = false;
    for i in 0..size {
        let c = if ended || i >= max { 0 } else { rd8(src + i) };
        ended |= c == 0;
        wr8(dst + i, c);
    }
}

/// Whether the string contains `c`.
pub fn contains(s: u32, c: u32) -> bool {
    let mut i = 0;
    loop {
        match rd8(s + i) {
            0 => return false,
            x if x == c => return true,
            _ => i += 1,
        }
    }
}
