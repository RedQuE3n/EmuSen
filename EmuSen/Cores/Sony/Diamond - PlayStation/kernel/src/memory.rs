//! psx-spx's "BIOS Memory Fill/Copy/Compare": byte by byte, with the refusals and the two documented bugs (memmove's
//! backward copy of len+1 bytes, memcmp's difference after the mismatch). Diamond_OpenKernel.md §4.8.

use crate::mem::{rd8, wr8};

const LIMIT: u32 = 0x7FFF_FFFF;

fn forward(dst: u32, src: u32, len: u32) {
    for i in 0..len {
        wr8(dst.wrapping_add(i), rd8(src.wrapping_add(i)));
    }
}

/// A(2Ah).
pub extern "C" fn memcpy(dst: u32, src: u32, len: u32) -> u32 {
    if dst != 0 && len <= LIMIT {
        forward(dst, src, len);
    }
    dst
}

/// A(2Bh): 0 when len is 0 or too large.
pub extern "C" fn memset(dst: u32, byte: u32, len: u32) -> u32 {
    if dst == 0 || len == 0 || len > LIMIT {
        return 0;
    }
    for i in 0..len {
        wr8(dst.wrapping_add(i), byte);
    }
    dst
}

/// A(2Ch): backwards only when src < dst and dst >= src+len, and then len+1 bytes.
pub extern "C" fn memmove(dst: u32, src: u32, len: u32) -> u32 {
    if dst == 0 || len > LIMIT {
        return dst;
    }
    if src < dst && dst >= src.wrapping_add(len) {
        let mut i = len;
        loop {
            wr8(dst.wrapping_add(i), rd8(src.wrapping_add(i)));
            if i == 0 {
                break;
            }
            i -= 1;
        }
    } else {
        forward(dst, src, len);
    }
    dst
}

/// A(2Dh) and A(29h): the difference of the bytes after the first mismatch.
pub extern "C" fn memcmp(a: u32, b: u32, len: u32) -> u32 {
    if a == 0 || b == 0 {
        return 0;
    }
    for i in 0..len {
        if rd8(a.wrapping_add(i)) != rd8(b.wrapping_add(i)) {
            let n = i.wrapping_add(1);
            return (rd8(a.wrapping_add(n)) as i32 - rd8(b.wrapping_add(n)) as i32) as u32;
        }
    }
    0
}

/// A(2Eh).
pub extern "C" fn memchr(src: u32, byte: u32, len: u32) -> u32 {
    if src == 0 || len > LIMIT {
        return 0;
    }
    (0..len).map(|i| src.wrapping_add(i)).find(|a| rd8(*a) == byte & 0xFF).unwrap_or(0)
}

/// A(27h): memcpy with src first, refusing a null src, returning src.
pub extern "C" fn bcopy(src: u32, dst: u32, len: u32) -> u32 {
    if src != 0 && len <= LIMIT {
        forward(dst, src, len);
    }
    src
}

/// A(28h).
pub extern "C" fn bzero(dst: u32, len: u32) -> u32 {
    memset(dst, 0, len)
}
