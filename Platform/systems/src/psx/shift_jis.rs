//! A memory card's title as the C# module read it: code page 932 decoded as .NET decodes it, then normalised to NFKC
//! (`string.Normalize(NormalizationForm.FormKC)`), so full-width letters read as their ASCII and half-width kana as
//! full-width. The normalisation is complete for what code page 932 decodes to, and only for that: the tables are the
//! decompositions and compositions of that repertoire (`cp932.rs`). See Hardware/Shared/EmuSen_MediumFormats.md §4.

use super::cp932::{COMPOSED, DECOMPOSED, PAIRS};

/// What .NET gives for a byte or pair it has no character for.
const UNMAPPED: char = '\u{30FB}';

fn is_lead(b: u8) -> bool {
    matches!(b, 0x81..=0x9F | 0xE0..=0xFC)
}

/// Bytes decoded as .NET's code page 932: a lead byte takes the next byte whatever it is, a pair it has no character
/// for and a lead at the end reading as U+30FB; A0h and FDh-FFh read as the private characters Windows gives them.
pub fn decode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        let c = if is_lead(b) {
            i += 1;
            match bytes.get(i) {
                Some(&t) => {
                    let code = (b as u16) << 8 | t as u16;
                    PAIRS.binary_search_by_key(&code, |&(k, _)| k).map_or(UNMAPPED, |at| char::from_u32(PAIRS[at].1 as u32).unwrap_or(UNMAPPED))
                }
                None => UNMAPPED,
            }
        } else {
            match b {
                0x00..=0x80 => b as char,
                0xA0 => '\u{F8F0}',
                0xA1..=0xDF => char::from_u32(0xFF61 + (b - 0xA1) as u32).unwrap_or(UNMAPPED),
                0xFD..=0xFF => char::from_u32(0xF8F1 + (b - 0xFD) as u32).unwrap_or(UNMAPPED),
                _ => UNMAPPED,
            }
        };
        out.push(c);
        i += 1;
    }
    out
}

/// A combining mark's canonical combining class; 0 for a starter. The marks are the eight the repertoire's
/// decompositions hold, with their classes from Unicode's UnicodeData.txt.
fn class(c: char) -> u8 {
    match c {
        '\u{0301}' | '\u{0304}' | '\u{0306}' | '\u{0308}' | '\u{030A}' => 230,
        '\u{0338}' => 1,
        '\u{3099}' | '\u{309A}' => 8,
        _ => 0,
    }
}

fn composed(starter: char, mark: char) -> Option<char> {
    let key = (starter as u32, mark as u32);
    COMPOSED
        .binary_search_by_key(&key, |&(s, m, _)| (s as u32, m as u32))
        .ok()
        .and_then(|at| char::from_u32(COMPOSED[at].2 as u32))
}

/// NFKC over text code page 932 decodes to: each character's compatibility decomposition, the marks put in canonical
/// order, then canonical composition.
pub fn nfkc(text: &str) -> String {
    let mut chars: Vec<char> = Vec::with_capacity(text.len());
    for c in text.chars() {
        let key = c as u32;
        match DECOMPOSED.binary_search_by_key(&key, |&(k, _)| k as u32) {
            Ok(at) => chars.extend(DECOMPOSED[at].1.chars()),
            Err(_) => chars.push(c),
        }
    }
    // Canonical ordering: each run of marks sorted by class, keeping the order of equal classes.
    let mut start = 0;
    while start < chars.len() {
        if class(chars[start]) == 0 {
            start += 1;
            continue;
        }
        let end = (start..chars.len()).find(|&i| class(chars[i]) == 0).unwrap_or(chars.len());
        chars[start..end].sort_by_key(|&c| class(c));
        start = end;
    }
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut starter: Option<usize> = None;
    for c in chars {
        let cc = class(c);
        if let Some(s) = starter {
            let last = out.len() - 1;
            let open = last == s || (class(out[last]) != 0 && class(out[last]) < cc);
            if open && let Some(both) = composed(out[s], c) {
                out[s] = both;
                continue;
            }
        }
        if cc == 0 {
            starter = Some(out.len());
        }
        out.push(c);
    }
    out.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pairs_and_singles_decode_as_dotnet_decodes_them() {
        assert_eq!(decode(b"\x82\x60\x81\x40A"), "Ａ\u{3000}A");
        assert_eq!(decode(b"\x81\x20A"), "\u{30FB}A", "an unmapped pair takes its trail byte with it");
        assert_eq!(decode(b"A\x81"), "A\u{30FB}");
        assert_eq!(decode(b"\x80\xA0\xB6\xDE\xFF"), "\u{80}\u{F8F0}ｶﾞ\u{F8F3}");
        assert_eq!(decode(b"\xF0\x40"), "\u{E000}");
    }

    #[test]
    fn full_width_letters_and_half_width_kana_normalise() {
        assert_eq!(nfkc("ＤＲＡＣＵＬＡ　Ｘ"), "DRACULA X");
        assert_eq!(nfkc("ｶﾞｲﾄﾞ"), "ガイド");
        assert_eq!(nfkc("ガ"), "ガ");
        assert_eq!(nfkc("゛"), " \u{3099}");
        assert_eq!(nfkc("Ёﾞ"), "Ё\u{3099}");
        assert_eq!(nfkc("①㈱"), "1(株)");
        assert_eq!(nfkc("≠"), "≠");
    }
}
