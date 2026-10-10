//! The text rules the preset and source readers lean on in C#: its regular expressions' classes, its number parsing,
//! and how it reads a file's lines. Each is .NET's rule as measured, not Rust's nearest. See EmuSen_RustPlatform.md §16.3.

use crate::word::is_word;
use emusen_galaxia::dotnet_path::{equals_ignore_case, is_white_space};

/// `\s`, and `char.IsWhiteSpace`, which measure the same in .NET 10 and are Unicode's White_Space.
pub fn is_space(c: char) -> bool {
    is_white_space(c)
}

/// What is left after a run of `\s`.
pub fn skip_spaces(text: &str) -> &str {
    text.trim_start_matches(is_space)
}

/// What is left after a run of `\s` at least one long.
pub fn spaces(text: &str) -> Option<&str> {
    let rest = skip_spaces(text);
    (rest.len() < text.len()).then_some(rest)
}

/// A run of `\S` at least one long, and what follows it.
pub fn word(text: &str) -> Option<(&str, &str)> {
    let end = text.find(is_space).unwrap_or(text.len());
    (end > 0).then(|| text.split_at(end))
}

/// `"([^"]*)"` at the start of the text: what is quoted, and what follows the closing quote.
pub fn quoted(text: &str) -> Option<(&str, &str)> {
    let inner = text.strip_prefix('"')?;
    let close = inner.find('"')?;
    Some((&inner[..close], &inner[close + 1..]))
}

/// `^\s*#\s*pragma\s+<keyword>\s+`: what follows, for a line that is this pragma.
pub fn pragma<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    let rest = skip_spaces(skip_spaces(line).strip_prefix('#')?).strip_prefix("pragma")?;
    spaces(spaces(rest)?.strip_prefix(keyword)?)
}

/// `\b` after a word that ends where `rest` begins: the next character is not one of `\w`.
pub fn word_ends(rest: &str) -> bool {
    !rest.chars().next().is_some_and(is_word)
}

/// The white space .NET's number parsing skips around a number: U+0009 to U+000D and the space.
fn is_number_space(c: char) -> bool {
    matches!(c, '\u{9}'..='\u{D}' | ' ')
}

/// A number's text taken apart as .NET's `NumberStyles.Float` reads it, or none when it is not one.
struct Number<'a> {
    negative: bool,
    whole: &'a str,
    fraction: &'a str,
    exponent: Option<(bool, &'a str)>,
}

fn digits(text: &str) -> (&str, &str) {
    text.split_at(text.find(|c: char| !c.is_ascii_digit()).unwrap_or(text.len()))
}

/// The number at the whole of `text`: white space, a sign, digits with at most one point when `decimal`, an exponent when
/// `exponent`, white space, then any number of NULs.
fn number(text: &str, decimal: bool, exponent: bool) -> Option<Number<'_>> {
    let mut rest = text.trim_start_matches(is_number_space);
    let negative = rest.starts_with('-');
    if negative || rest.starts_with('+') {
        rest = &rest[1..];
    }
    let (whole, after) = digits(rest);
    rest = after;
    let mut fraction = "";
    if decimal && let Some(after) = rest.strip_prefix('.') {
        (fraction, rest) = digits(after);
    }
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    let mut power = None;
    if exponent && let Some(after) = rest.strip_prefix(['e', 'E']) {
        let (minus, unsigned) = match after.strip_prefix('-') {
            Some(unsigned) => (true, unsigned),
            None => (false, after.strip_prefix('+').unwrap_or(after)),
        };
        // An `e` with no digits after it is not an exponent, and is left to fail as what follows the number.
        let (value, after) = digits(unsigned);
        if !value.is_empty() {
            power = Some((minus, value));
            rest = after;
        }
    }
    rest = rest.trim_start_matches(is_number_space).trim_start_matches('\0');
    rest.is_empty().then_some(Number { negative, whole, fraction, exponent: power })
}

/// .NET's `float.NaN`, which is the quiet NaN with its sign set.
pub const NAN: f32 = f32::from_bits(0xFFC0_0000);

/// `float.TryParse(text, NumberStyles.Float, CultureInfo.InvariantCulture, out value)`.
pub fn parse_float(text: &str) -> Option<f32> {
    if let Some(n) = number(text, true, true) {
        let mut plain = String::with_capacity(text.len() + 2);
        if n.negative {
            plain.push('-');
        }
        plain.push_str(if n.whole.is_empty() { "0" } else { n.whole });
        plain.push('.');
        plain.push_str(if n.fraction.is_empty() { "0" } else { n.fraction });
        if let Some((minus, value)) = n.exponent {
            plain.push('e');
            if minus {
                plain.push('-');
            }
            // More digits than any exponent that matters; Rust's own parser refuses nothing but length, so the value is kept in range.
            let value = value.trim_start_matches('0');
            plain.push_str(if value.len() > 9 { "999999999" } else if value.is_empty() { "0" } else { value });
        }
        return plain.parse::<f32>().ok();
    }
    // What is not a number may still be one of the three words, with any case and any white space around it.
    let word = text.trim_matches(is_white_space);
    if equals_ignore_case(word, "Infinity") {
        return Some(f32::INFINITY);
    }
    if equals_ignore_case(word, "-Infinity") {
        return Some(f32::NEG_INFINITY);
    }
    if equals_ignore_case(word, "NaN") {
        return Some(NAN);
    }
    if let Some(unsigned) = word.strip_prefix('+') {
        if equals_ignore_case(unsigned, "Infinity") {
            return Some(f32::INFINITY);
        }
        return equals_ignore_case(unsigned, "NaN").then_some(NAN);
    }
    word.strip_prefix('-').filter(|unsigned| equals_ignore_case(unsigned, "NaN")).map(|_| NAN)
}

/// `int.TryParse(text, NumberStyles.Integer, CultureInfo.InvariantCulture, out value)`.
pub fn parse_int(text: &str) -> Option<i32> {
    let n = number(text, false, false)?;
    let digits = n.whole.trim_start_matches('0');
    if digits.len() > 10 {
        return None;
    }
    let magnitude: i64 = if digits.is_empty() { 0 } else { digits.parse().ok()? };
    i32::try_from(if n.negative { -magnitude } else { magnitude }).ok()
}

/// A file's lines as `File.ReadLines` gives them: decoded as `File.ReadAllText` decodes, cut at `\r\n`, `\r` or `\n`, with no empty line for a final break.
pub fn lines(bytes: &[u8]) -> Vec<String> {
    let text = emusen_galaxia::json::decode(bytes);
    let mut lines = Vec::new();
    let mut rest = text.as_str();
    while !rest.is_empty() {
        match rest.find(['\r', '\n']) {
            Some(at) => {
                lines.push(rest[..at].to_string());
                let after = &rest[at..];
                rest = &after[if after.starts_with("\r\n") { 2 } else { 1 }..];
            }
            None => {
                lines.push(rest.to_string());
                break;
            }
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(text: &str) -> Option<u32> {
        parse_float(text).map(f32::to_bits)
    }

    #[test]
    fn a_float_is_read_as_dotnet_measured_it() {
        for (text, expected) in [
            ("1", 0x3F80_0000u32),
            (" 1 ", 0x3F80_0000),
            ("+1", 0x3F80_0000),
            ("1\0\0", 0x3F80_0000),
            ("1 \0", 0x3F80_0000),
            ("\t1\u{B}", 0x3F80_0000),
            ("1.5", 0x3FC0_0000),
            (".5", 0x3F00_0000),
            ("5.", 0x40A0_0000),
            ("1e3", 0x447A_0000),
            ("1E-2", 0x3C23_D70A),
            ("Infinity", 0x7F80_0000),
            ("infinity", 0x7F80_0000),
            (" -INFINITY ", 0xFF80_0000),
            ("+Infinity", 0x7F80_0000),
            ("NaN", 0xFFC0_0000),
            ("-nan", 0xFFC0_0000),
            ("+NaN", 0xFFC0_0000),
            ("1e99999999999", 0x7F80_0000),
            ("-0", 0x8000_0000),
            ("1e-99999", 0),
            ("\u{A0}NaN\u{2003}", 0xFFC0_0000),
        ] {
            assert_eq!(bits(text), Some(expected), "{text:?}");
        }
        for text in [".", "1e", "1e+", "inf", "∞", "1,000", "0x10", "١", "1_0", "\u{A0}1", "1\u{A0}", "nan\0", "1.5f", "--1", "1.2.3", "", "1\0 "] {
            assert_eq!(bits(text), None, "{text:?}");
        }
    }

    #[test]
    fn an_integer_is_read_as_dotnet_measured_it() {
        for (text, expected) in [("1", 1), (" 1 ", 1), ("+1", 1), ("1\0\0", 1), ("\t1\u{B}", 1), ("-0", 0), ("2147483647", i32::MAX), ("-2147483648", i32::MIN), ("0000000000012", 12)] {
            assert_eq!(parse_int(text), Some(expected), "{text:?}");
        }
        for text in ["1.5", "1e3", "", "2147483648", "-2147483649", "١", "1_0", " ", "+", "99999999999999999999"] {
            assert_eq!(parse_int(text), None, "{text:?}");
        }
    }

    #[test]
    fn lines_break_at_cr_lf_and_crlf_and_nowhere_else() {
        assert_eq!(lines("a\r\nb\rc\n\nd\u{85}e\u{2028}f\n".as_bytes()), ["a", "b", "c", "", "d\u{85}e\u{2028}f"]);
        assert_eq!(lines(b"x\r"), ["x"]);
        assert_eq!(lines(b"\n"), [""]);
        assert!(lines(b"").is_empty());
        assert_eq!(lines(b"\xEF\xBB\xBFkey"), ["key"], "a byte-order mark is not part of the first line");
    }

    #[test]
    fn a_pragma_is_found_through_any_spacing_and_a_word_ends_only_at_a_boundary() {
        assert_eq!(pragma("  #  pragma   stage  vertex", "stage"), Some("vertex"));
        assert_eq!(pragma("#pragma stagevertex", "stage"), None);
        assert_eq!(pragma("#pragmastage vertex", "stage"), None);
        assert_eq!(pragma("x #pragma stage vertex", "stage"), None);
        assert!(word_ends("") && word_ends(" x") && word_ends("-") && word_ends("😀"));
        assert!(!word_ends("x") && !word_ends("_") && !word_ends("é") && !word_ends("9"));
        assert_eq!(quoted("\"a b\" c"), Some(("a b", " c")));
        assert_eq!(quoted("\"\""), Some(("", "")));
        assert_eq!(quoted("\"open"), None);
        assert_eq!(word("abc  d"), Some(("abc", "  d")));
        assert_eq!(word(" abc"), None);
    }
}
