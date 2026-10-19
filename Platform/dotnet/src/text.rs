//! .NET's string rules: casing, ordinal comparison ignoring case, lengths counted in UTF-16 code units, white space
//! and line endings. See EmuSen_RustPlatform.md §11.3.

use std::cmp::Ordering;

/// `char.ToLowerInvariant`: Unicode's one-to-one lower case, with the dotted capital I left as it is.
pub fn lower_invariant(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_lowercase();
    }
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
    match char::from_u32(unit as u32).map(simple_upper) {
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
    compare_units(&a, &b)
}

/// `text.Contains(value, StringComparison.OrdinalIgnoreCase)`: some run of the text's code units equal to the value's by the same rule; an empty value is in every text.
pub fn contains_ignore_case(text: &str, value: &str) -> bool {
    let (text, value): (Vec<u16>, Vec<u16>) = (text.encode_utf16().collect(), value.encode_utf16().collect());
    value.is_empty() || text.windows(value.len()).any(|run| compare_units(run, &value) == Ordering::Equal)
}

fn compare_units(a: &[u16], b: &[u16]) -> Ordering {
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
                let upper = |scalar: u32| char::from_u32(scalar).map_or(scalar, |c| simple_upper(c) as u32);
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

/// `char.IsWhiteSpace`, which is Unicode's White_Space and so Rust's own; U+001C to U+001F are not in it, whatever .NET's page says.
pub fn is_white_space(c: char) -> bool {
    c.is_whitespace()
}

/// `string.IsNullOrWhiteSpace`.
pub fn is_blank(text: Option<&str>) -> bool {
    text.is_none_or(|t| t.chars().all(is_white_space))
}

/// One character's upper case as .NET's ordinal comparison sees it: Unicode's simple mapping, one character to one, less the two that would turn a letter outside ASCII into one inside it.
pub fn simple_upper(c: char) -> char {
    if c.is_ascii() {
        return c.to_ascii_uppercase();
    }
    match c {
        // The dotless i and the long s keep themselves, where Unicode sends them to I and S.
        '\u{131}' | '\u{17F}' => c,
        // Greek with a subscript iota: the standard library gives the two-letter full mapping, and the simple one is the capital beside it.
        '\u{1F80}'..='\u{1F87}' | '\u{1F90}'..='\u{1F97}' | '\u{1FA0}'..='\u{1FA7}' => char::from_u32(c as u32 + 8).unwrap_or(c),
        '\u{1FB3}' | '\u{1FC3}' | '\u{1FF3}' => char::from_u32(c as u32 + 9).unwrap_or(c),
        _ => {
            let mut upper = c.to_uppercase();
            match (upper.next(), upper.next()) {
                (Some(one), None) => one,
                // A character whose only upper case is several characters has no simple one.
                _ => c,
            }
        }
    }
}

/// `char.ToUpperInvariant`: Unicode's simple mapping, one character to one, except the dotless i, which keeps itself; the
/// long s goes to S (measured on .NET 10.0.112, 2026-10-17), where `simple_upper` keeps it.
pub fn upper_invariant(c: char) -> char {
    if c == '\u{17F}' { 'S' } else { simple_upper(c) }
}

/// `string.Equals(a, b, StringComparison.OrdinalIgnoreCase)`.
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    // ASCII's ordinal case is ASCII's own, and it is how nearly every name the shell compares is spelt.
    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }
    a.chars().map(simple_upper).eq(b.chars().map(simple_upper))
}

/// `a.EndsWith(suffix, StringComparison.OrdinalIgnoreCase)`.
pub fn ends_with_ignore_case(a: &str, suffix: &str) -> bool {
    let (a, suffix): (Vec<char>, Vec<char>) = (a.chars().map(simple_upper).collect(), suffix.chars().map(simple_upper).collect());
    a.ends_with(&suffix)
}

/// `string.CompareOrdinal`: UTF-16 code units compared as numbers, which orders a character outside the Basic
/// Multilingual Plane before U+E000 to U+FFFF, where UTF-8's byte order puts it after.
pub fn compare_ordinal(a: &str, b: &str) -> Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

/// `char.IsLetter`. A character outside the Basic Multilingual Plane is two surrogates to .NET, and neither is a
/// letter; inside it, Unicode's Alphabetic stands in for the letter categories, which it also counts the letter
/// numbers and a few marks among.
pub fn is_letter(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphabetic();
    }
    (c as u32) <= 0xFFFF && c.is_alphabetic()
}

/// `char.IsLetterOrDigit`, from .NET's own ranges: a character outside the Basic Multilingual Plane is two surrogates to
/// .NET, and neither is a letter or a digit.
pub fn is_letter_or_digit(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_alphanumeric();
    }
    let unit = c as u32;
    unit <= 0xFFFF && crate::letter_or_digit::LETTER_OR_DIGIT.binary_search_by(|&(low, high)| {
        if (high as u32) < unit { std::cmp::Ordering::Less } else if (low as u32) > unit { std::cmp::Ordering::Greater } else { std::cmp::Ordering::Equal }
    }).is_ok()
}

/// The runs of Unicode's decimal digits above ASCII where .NET 10.0.12's `char.IsDigit` holds, generated from it on
/// 2026-10-19; each is ten long.
static DIGIT_ZEROS: [u16; 36] = [
    0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6, 0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090,
    0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80, 0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0, 0xFF10,
];

/// `char.IsDigit`: a decimal digit of any script; outside the Basic Multilingual Plane a character is two surrogates to
/// .NET, and neither is a digit.
pub fn is_digit(c: char) -> bool {
    if c.is_ascii() {
        return c.is_ascii_digit();
    }
    let unit = c as u32;
    DIGIT_ZEROS.iter().any(|&zero| unit >= zero as u32 && unit < zero as u32 + 10)
}

/// `string.Trim()`.
pub fn trim(text: &str) -> &str {
    text.trim_matches(is_white_space)
}

/// `string.TrimEnd()`.
pub fn trim_end(text: &str) -> &str {
    text.trim_end_matches(is_white_space)
}

/// `string.TrimStart()`.
pub fn trim_start(text: &str) -> &str {
    text.trim_start_matches(is_white_space)
}

/// `string.ToLowerInvariant()`.
pub fn to_lower(text: &str) -> String {
    if text.is_ascii() {
        return text.to_ascii_lowercase();
    }
    text.chars().map(lower_invariant).collect()
}

/// `string.ToUpperInvariant()`.
pub fn to_upper(text: &str) -> String {
    if text.is_ascii() {
        return text.to_ascii_uppercase();
    }
    text.chars().map(simple_upper).collect()
}

/// `text.StartsWith(prefix, OrdinalIgnoreCase)`.
pub fn starts_with_ignore_case(text: &str, prefix: &str) -> bool {
    if text.is_ascii() && prefix.is_ascii() {
        return text.len() >= prefix.len() && text.as_bytes()[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes());
    }
    let mut text = text.chars().map(simple_upper);
    prefix.chars().map(simple_upper).all(|c| text.next() == Some(c))
}

/// A key under which `OrdinalIgnoreCase` finds a name.
pub fn fold(text: &str) -> String {
    text.chars().map(simple_upper).collect()
}

/// `{value,-width}`: padded on the right to `width` UTF-16 units.
pub fn pad_right(text: &str, width: usize) -> String {
    let mut out = text.to_string();
    for _ in utf16_len(text)..width {
        out.push(' ');
    }
    out
}

/// `{value,width}` and `PadLeft(width)`.
pub fn pad_left(text: &str, width: usize) -> String {
    let mut out = String::new();
    for _ in utf16_len(text)..width {
        out.push(' ');
    }
    out.push_str(text);
    out
}

#[cfg(test)]
mod tests {

    #[test]
    fn digits_are_dotnets() {
        assert!(is_digit('7') && is_digit('\u{663}') && is_digit('\u{FF19}') && is_digit('\u{1C59}'));
        assert!(!is_digit('a') && !is_digit('\u{B2}') && !is_digit('\u{2163}') && !is_digit('\u{FF1A}') && !is_digit('\u{1D7CE}'));
    }

    #[test]
    fn letters_and_digits_and_upper_case_are_dotnets() {
        assert!(is_letter_or_digit('\u{663}') && is_letter_or_digit('\u{FF11}') && is_letter_or_digit('\u{E9}'));
        assert!(!is_letter_or_digit('\u{B2}') && !is_letter_or_digit('\u{2163}') && !is_letter_or_digit('\u{301}') && !is_letter_or_digit('\u{1F600}'));
        assert_eq!([upper_invariant('a'), upper_invariant('\u{17F}'), upper_invariant('\u{131}'), upper_invariant('\u{DF}')], ['A', 'S', '\u{131}', '\u{DF}']);
    }
    use super::*;

    #[test]
    fn the_ascii_paths_answer_as_the_general_ones() {
        for c in (0u8..128).map(char::from) {
            let mut upper = c.to_uppercase();
            assert_eq!(simple_upper(c), upper.next().unwrap(), "{c:?}");
            assert_eq!(lower_invariant(c), c.to_lowercase().next().unwrap(), "{c:?}");
        }
        assert!(equals_ignore_case("Grep", "gREP"));
        assert!(!equals_ignore_case("k", "\u{212A}"));
        assert!(!equals_ignore_case("grep", "grep "));
        assert!(starts_with_ignore_case("BP3", "bp"));
        assert!(!starts_with_ignore_case("b", "bp"));
        assert_eq!(to_lower("SoRT"), "sort");
    }

    #[test]
    fn a_text_holds_another_whatever_its_case() {
        assert!(contains_ignore_case("AMD Radeon RX 6800 (RADV NAVI21)", "radv"));
        assert!(contains_ignore_case("AMD Radeon RX 6800 (RADV NAVI21)", "6800"));
        assert!(contains_ignore_case("llvmpipe (LLVM 20.1, 256 bits)", "LLVMPIPE"));
        assert!(contains_ignore_case("anything", ""));
        assert!(contains_ignore_case("", ""));
        assert!(!contains_ignore_case("", "a"));
        assert!(!contains_ignore_case("Radeon", "Radeons"));
        assert!(contains_ignore_case("caf\u{C9}", "\u{E9}"));
        // The long s is not an s to an ordinal comparison, as it is not to .NET's.
        assert!(!contains_ignore_case("\u{17F}", "s"));
    }

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
