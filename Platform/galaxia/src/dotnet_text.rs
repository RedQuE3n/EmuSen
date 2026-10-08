//! .NET's string rules that are counted in UTF-16 code units or decided by its own casing, reproduced for the
//! places where an answer shown to the player or an order on screen depends on them. See EmuSen_RustPlatform.md §11.3.

use std::cmp::Ordering;

/// `char.ToLowerInvariant`: Unicode's one-to-one lower case, with the dotted capital I left as it is.
pub fn lower_invariant(c: char) -> char {
    let mut lower = c.to_lowercase();
    match (lower.next(), lower.next()) {
        (Some(one), None) => one,
        _ => c,
    }
}

/// One UTF-16 code unit lowered as `char.ToLowerInvariant` lowers a `char`: half of a surrogate pair is left alone.
pub fn lower_unit(unit: u16) -> u16 {
    match char::from_u32(unit as u32).map(lower_invariant) {
        Some(lower) if (lower as u32) <= 0xFFFF => lower as u16,
        _ => unit,
    }
}

/// A string's `Length`: its UTF-16 code units.
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

fn upper_unit(unit: u16) -> u16 {
    match char::from_u32(unit as u32).map(crate::dotnet_path::simple_upper) {
        Some(upper) if (upper as u32) <= 0xFFFF => upper as u16,
        _ => unit,
    }
}

fn is_high(unit: u16) -> bool {
    (0xD800..=0xDBFF).contains(&unit)
}

fn is_low(unit: u16) -> bool {
    (0xDC00..=0xDFFF).contains(&unit)
}

/// `StringComparer.OrdinalIgnoreCase.Compare`: UTF-16 code units by their upper case, a surrogate pair as one character by its own, and a pair after any single unit.
pub fn compare_ignore_case(a: &str, b: &str) -> Ordering {
    let (a, b): (Vec<u16>, Vec<u16>) = (a.encode_utf16().collect(), b.encode_utf16().collect());
    let length = a.len().min(b.len());
    let mut i = 0;
    while i < length {
        let pair_a = is_high(a[i]) && i + 1 < a.len() && is_low(a[i + 1]);
        let pair_b = is_high(b[i]) && i + 1 < b.len() && is_low(b[i + 1]);
        match (pair_a, pair_b) {
            (false, false) => {
                if upper_unit(a[i]) != upper_unit(b[i]) {
                    return upper_unit(a[i]).cmp(&upper_unit(b[i]));
                }
                i += 1;
            }
            (false, true) => return Ordering::Less,
            (true, false) => return Ordering::Greater,
            (true, true) => {
                let scalar = |high: u16, low: u16| 0x10000 + (((high as u32) - 0xD800) << 10) + ((low as u32) - 0xDC00);
                let upper = |scalar: u32| char::from_u32(scalar).map_or(scalar, |c| crate::dotnet_path::simple_upper(c) as u32);
                let (upper_a, upper_b) = (upper(scalar(a[i], a[i + 1])), upper(scalar(b[i], b[i + 1])));
                if upper_a != upper_b {
                    return upper_a.cmp(&upper_b);
                }
                i += 2;
            }
        }
    }
    a.len().cmp(&b.len())
}

/// `string.ReplaceLineEndings`: every newline sequence .NET knows becomes `with`, a carriage return and line feed together counting once.
pub fn replace_line_endings(text: &str, with: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                out.push_str(with);
            }
            '\n' | '\u{C}' | '\u{85}' | '\u{2028}' | '\u{2029}' => out.push_str(with),
            _ => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_case_is_one_to_one() {
        assert_eq!(lower_invariant('A'), 'a');
        assert_eq!(lower_invariant('\u{130}'), '\u{130}');
        assert_eq!(lower_invariant('\u{3A3}'), '\u{3C3}');
        assert_eq!(lower_invariant('\u{212A}'), 'k');
        assert_eq!(lower_invariant('\u{1E9E}'), '\u{DF}');
        assert_eq!(lower_invariant('\u{1C5}'), '\u{1C6}');
        assert_eq!(lower_unit(0xD83D), 0xD83D);
        assert_eq!(utf16_len("a😀é"), 4);
    }

    #[test]
    fn names_are_ordered_as_dotnet_orders_them_without_case() {
        let order = |a: &str, b: &str| compare_ignore_case(a, b);
        assert_eq!(order("a", "B"), Ordering::Less);
        assert_eq!(order("_", "a"), Ordering::Greater);
        assert_eq!(order("[", "a"), Ordering::Greater);
        assert_eq!(order("abc", "ABC"), Ordering::Equal);
        assert_eq!(order("ab", "abc"), Ordering::Less);
        assert_eq!(order("é", "Éz"), Ordering::Less);
        assert_eq!(order("😀", "\u{E000}"), Ordering::Greater);
        assert_eq!(order("\u{E000}", "😀"), Ordering::Less);
        assert_eq!(order("é", "É"), Ordering::Equal);
        // Past the first character outside ASCII the order is still by upper case, not by the characters as written.
        assert_eq!(order("éA~", "ßÉ"), Ordering::Less);
        assert_eq!(order("日h", "日["), Ordering::Less);
        assert_eq!(order("ıza", "ı_"), Ordering::Less);
        assert_eq!(order("\u{10428}9", "\u{10400}a"), Ordering::Less);
    }

    #[test]
    fn every_newline_dotnet_knows_is_replaced_and_a_pair_counts_once() {
        assert_eq!(replace_line_endings("a\r\nb\nc\rd\u{C}e\u{85}f\u{2028}g\u{2029}h\u{B}i", "|"), "a|b|c|d|e|f|g|h\u{B}i");
        assert_eq!(replace_line_endings("\r\r\n\n", "|"), "|||");
    }
}
