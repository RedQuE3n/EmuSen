//! psx-spx's "BIOS String Functions", with the refusals of null pointers and the documented behaviour of strpbrk,
//! strtok and strstr. Diamond_OpenKernel.md §4.8.

use crate::mem::{rd8, rd8s, wr8};

pub fn length(s: u32) -> u32 {
    let mut n = 0;
    while rd8(s + n) != 0 {
        n += 1;
    }
    n
}

fn contains(list: u32, c: u32) -> bool {
    let mut i = 0;
    loop {
        let l = rd8(list + i);
        if l == 0 {
            return false;
        }
        if l == c {
            return true;
        }
        i += 1;
    }
}

/// A(15h).
pub extern "C" fn strcat(dst: u32, src: u32) -> u32 {
    if dst == 0 || src == 0 {
        return 0;
    }
    strcpy(dst + length(dst), src);
    dst
}

/// A(16h): at most max(0,maxlen) characters appended, then 00h (psx-spx's "min" read as max, §4.8).
pub extern "C" fn strncat(dst: u32, src: u32, maxlen: u32) -> u32 {
    if dst == 0 || src == 0 {
        return 0;
    }
    let limit = (maxlen as i32).max(0) as u32;
    let end = dst + length(dst);
    let mut i = 0;
    while i < limit && rd8(src + i) != 0 {
        wr8(end + i, rd8(src + i));
        i += 1;
    }
    wr8(end + i, 0);
    dst
}

fn compare(a: u32, b: u32, max: u32) -> u32 {
    match (a, b) {
        (0, 0) => return 0,
        (0, _) => return u32::MAX,
        (_, 0) => return 1,
        _ => {}
    }
    let mut i = 0;
    while i < max {
        let (x, y) = (rd8s(a + i), rd8s(b + i));
        if x != y {
            return (x - y) as u32;
        }
        if x == 0 {
            break;
        }
        i += 1;
    }
    0
}

/// A(17h): the difference of the sign-extended bytes.
pub extern "C" fn strcmp(a: u32, b: u32) -> u32 {
    compare(a, b, u32::MAX)
}

/// A(18h).
pub extern "C" fn strncmp(a: u32, b: u32, maxlen: u32) -> u32 {
    compare(a, b, maxlen)
}

/// A(19h).
pub extern "C" fn strcpy(dst: u32, src: u32) -> u32 {
    if dst == 0 || src == 0 {
        return 0;
    }
    let mut i = 0;
    loop {
        let c = rd8(src + i);
        wr8(dst + i, c);
        if c == 0 {
            return dst;
        }
        i += 1;
    }
}

/// A(1Ah): no 00h when src is maxlen or longer, the rest padded with 00h when shorter.
pub extern "C" fn strncpy(dst: u32, src: u32, maxlen: u32) -> u32 {
    if dst == 0 || src == 0 {
        return 0;
    }
    let mut ended = false;
    for i in 0..maxlen {
        let c = if ended { 0 } else { rd8(src + i) };
        ended |= c == 0;
        wr8(dst + i, c);
    }
    dst
}

/// A(1Bh).
pub extern "C" fn strlen(src: u32) -> u32 {
    if src == 0 { 0 } else { length(src) }
}

/// A(1Ch) and A(1Eh): an address; char 00h finds the end.
pub extern "C" fn index(src: u32, c: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    let c = c & 0xFF;
    let mut i = 0;
    loop {
        let x = rd8(src + i);
        if x == c {
            return src + i;
        }
        if x == 0 {
            return 0;
        }
        i += 1;
    }
}

/// A(1Dh) and A(1Fh).
pub extern "C" fn rindex(src: u32, c: u32) -> u32 {
    if src == 0 {
        return 0;
    }
    let c = c & 0xFF;
    let mut found = 0;
    let mut i = 0;
    loop {
        let x = rd8(src + i);
        if x == c {
            found = src + i;
        }
        if x == 0 {
            return found;
        }
        i += 1;
    }
}

/// A(20h): with no occurrence, 0 only when src is empty and src otherwise, as documented.
pub extern "C" fn strpbrk(src: u32, list: u32) -> u32 {
    if src == 0 || list == 0 {
        return 0;
    }
    let mut i = 0;
    loop {
        let x = rd8(src + i);
        if x == 0 {
            return if rd8(src) == 0 { 0 } else { src };
        }
        if contains(list, x) {
            return src + i;
        }
        i += 1;
    }
}

fn span(src: u32, list: u32, inside: bool) -> u32 {
    if src == 0 || list == 0 {
        return 0;
    }
    let mut i = 0;
    loop {
        let x = rd8(src + i);
        if x == 0 || contains(list, x) != inside {
            return i;
        }
        i += 1;
    }
}

/// A(21h): the length of the leading run of characters in the list (POSIX; psx-spx's own reading in doubt, §4.8).
pub extern "C" fn strspn(src: u32, list: u32) -> u32 {
    span(src, list, true)
}

/// A(22h): the length of the leading run of characters not in the list.
pub extern "C" fn strcspn(src: u32, list: u32) -> u32 {
    span(src, list, false)
}

static mut TOKENS: [u8; 0x100] = [0; 0x100];
static mut NEXT: u32 = 0;

/// A(23h): the string copied to a 100h-byte buffer on the first call; following separators skipped only when the list
/// is one character, the rule both of psx-spx's examples follow.
pub extern "C" fn strtok(src: u32, list: u32) -> u32 {
    let buffer = &raw mut TOKENS as u32;
    if src != 0 {
        let mut i = 0;
        while i < 0xFF && rd8(src + i) != 0 {
            wr8(buffer + i, rd8(src + i));
            i += 1;
        }
        wr8(buffer + i, 0);
        unsafe { NEXT = buffer };
    }
    let start = unsafe { NEXT };
    if start == 0 || list == 0 || rd8(start) == 0 {
        unsafe { NEXT = 0 };
        return 0;
    }
    let mut i = start;
    while rd8(i) != 0 && !contains(list, rd8(i)) {
        i += 1;
    }
    if rd8(i) == 0 {
        unsafe { NEXT = i };
        return start;
    }
    wr8(i, 0);
    i += 1;
    if length(list) == 1 {
        while rd8(i) != 0 && contains(list, rd8(i)) {
            i += 1;
        }
    }
    unsafe { NEXT = i };
    start
}

/// A(24h): a failed partial match resumes after the mismatching character, so "aab" is not found in "aaab".
pub extern "C" fn strstr(s: u32, sub: u32) -> u32 {
    if s == 0 || sub == 0 {
        return 0;
    }
    if rd8(sub) == 0 {
        return s;
    }
    let mut i = s;
    loop {
        let start = i;
        let mut j = sub;
        loop {
            if rd8(j) == 0 {
                return start;
            }
            if rd8(i) == 0 {
                return 0;
            }
            if rd8(i) != rd8(j) {
                break;
            }
            i += 1;
            j += 1;
        }
        i += 1;
    }
}
