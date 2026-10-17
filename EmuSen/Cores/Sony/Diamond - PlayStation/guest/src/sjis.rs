//! Shift-JIS as the console's software meets it, read from psx-spx's "Shift-JIS Character Set": the full-width Latin
//! letters, digits and common punctuation of JIS X 0208's first rows as their ASCII forms, shared by the kernel's
//! Krom2RawAdd and the shell's save titles. Diamond_OpenKernel_Phase2.md §25.1, Diamond_Shell.md §3.1.

/// The full-width code's ASCII form among the letters, digits and punctuation of JIS X 0208's first rows ("Shift-JIS
/// Character Set"); the card manager's titles use the same.
pub const fn ascii(code: u32) -> Option<u8> {
    let (a, b) = (code >> 8, code & 0xFF);
    Some(match (a, b) {
        (0x82, 0x4F..=0x58) => (b - 0x4F) as u8 + b'0',
        (0x82, 0x60..=0x79) => (b - 0x60) as u8 + b'A',
        (0x82, 0x81..=0x9A) => (b - 0x81) as u8 + b'a',
        (0x81, 0x40) => b' ',
        (0x81, 0x43) => b',',
        (0x81, 0x44) => b'.',
        (0x81, 0x46) => b':',
        (0x81, 0x47) => b';',
        (0x81, 0x48) => b'?',
        (0x81, 0x49) => b'!',
        (0x81, 0x5B..=0x5D | 0x7C) => b'-',
        (0x81, 0x5E) => b'/',
        (0x81, 0x60) => b'~',
        (0x81, 0x62) => b'|',
        (0x81, 0x65 | 0x66) => b'\'',
        (0x81, 0x67 | 0x68) => b'"',
        (0x81, 0x69) => b'(',
        (0x81, 0x6A) => b')',
        (0x81, 0x6D) => b'[',
        (0x81, 0x6E) => b']',
        (0x81, 0x7B) => b'+',
        (0x81, 0x81) => b'=',
        (0x81, 0x83) => b'<',
        (0x81, 0x84) => b'>',
        (0x81, 0x90) => b'$',
        (0x81, 0x93) => b'%',
        (0x81, 0x94) => b'#',
        (0x81, 0x95) => b'&',
        (0x81, 0x96) => b'*',
        (0x81, 0x97) => b'@',
        _ => return None,
    })
}

/// Whether `a` begins a two-byte Shift-JIS code.
pub const fn lead(a: u8) -> bool {
    matches!(a, 0x81..=0x9F | 0xE0..=0xEF)
}
