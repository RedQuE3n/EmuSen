//! JSON as System.Text.Json reads and writes it under `ConfigJson.Options`, reproduced so that a config file means
//! the same thing to both implementations and is written as the same bytes.
//!
//! The reader accepts what .NET's reader accepts with comments skipped and trailing commas allowed, and nothing else;
//! the writer escapes and lays out numbers as .NET's does. Each rule was measured against .NET 10 and is held by the
//! parity class; EmuSen_RustPlatform.md §5.2 and §11.3 list them.

use std::fmt;

/// Text from a JSON string, with whether an escape in it was not valid UTF-16. .NET refuses such a string only when it is used, so the flag travels with it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Text {
    pub text: String,
    pub broken: bool,
}

impl Text {
    pub fn new(text: impl Into<String>) -> Text {
        Text { text: text.into(), broken: false }
    }
}

/// One JSON value. An object keeps its members in file order, duplicates included; a number keeps its token.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(String),
    String(Text),
    Array(Vec<Value>),
    Object(Vec<(Text, Value)>),
}

impl Value {
    pub fn string(text: impl Into<String>) -> Value {
        Value::String(Text::new(text))
    }

    pub fn number(token: impl ToString) -> Value {
        Value::Number(token.to_string())
    }

    /// A member of an object by its exact name; the last, as a later member replaces an earlier one.
    pub fn get(&self, name: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members.iter().rev().find(|(key, _)| key.text == name).map(|(_, value)| value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(text) => Some(&text.text),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(value) => Some(*value),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// Where the text stopped being JSON, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub what: &'static str,
    pub line: usize,
    pub byte: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at line {}, byte {}.", self.what, self.line, self.byte)
    }
}

/// The deepest nesting .NET reads by default.
pub const MAX_DEPTH: usize = 64;

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
    depth: usize,
}

impl<'a> Reader<'a> {
    fn fail<T>(&self, what: &'static str) -> Result<T, Error> {
        let before = &self.bytes[..self.at.min(self.bytes.len())];
        let line = before.iter().filter(|&&b| b == b'\n').count();
        let line_start = before.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
        Err(Error { what, line: line + 1, byte: before.len() - line_start + 1 })
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn skip_white_space(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    /// White space and comments, which .NET skips everywhere but between a name and its colon.
    fn skip_filler(&mut self) -> Result<(), Error> {
        loop {
            self.skip_white_space();
            if self.peek() != Some(b'/') {
                return Ok(());
            }
            match self.bytes.get(self.at + 1) {
                Some(b'/') => {
                    self.at += 2;
                    while let Some(byte) = self.peek() {
                        if byte == b'\n' || byte == b'\r' {
                            break;
                        }
                        // A line or paragraph separator inside a one-line comment is refused, as .NET refuses it.
                        if self.bytes[self.at..].starts_with(&[0xE2, 0x80, 0xA8]) || self.bytes[self.at..].starts_with(&[0xE2, 0x80, 0xA9]) {
                            return self.fail("A line separator inside a comment");
                        }
                        self.at += 1;
                    }
                }
                Some(b'*') => {
                    let Some(end) = self.bytes[self.at + 2..].windows(2).position(|pair| pair == b"*/") else {
                        return self.fail("A comment that never ends");
                    };
                    self.at += 2 + end + 2;
                }
                _ => return self.fail("A stray '/'"),
            }
        }
    }

    fn value(&mut self) -> Result<Value, Error> {
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => Ok(Value::String(self.string()?)),
            Some(b't') => self.literal(b"true", Value::Bool(true)),
            Some(b'f') => self.literal(b"false", Value::Bool(false)),
            Some(b'n') => self.literal(b"null", Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => self.fail("Not the start of a value"),
            None => self.fail("The text ended where a value was expected"),
        }
    }

    fn literal(&mut self, word: &[u8], value: Value) -> Result<Value, Error> {
        if !self.bytes[self.at..].starts_with(word) {
            return self.fail("Not a value");
        }
        self.at += word.len();
        Ok(value)
    }

    fn digits(&mut self) -> usize {
        let start = self.at;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.at += 1;
        }
        self.at - start
    }

    fn number(&mut self) -> Result<Value, Error> {
        let start = self.at;
        if self.peek() == Some(b'-') {
            self.at += 1;
        }
        match self.peek() {
            Some(b'0') => self.at += 1,
            Some(b'1'..=b'9') => {
                self.digits();
            }
            _ => return self.fail("A number with no digits"),
        }
        if self.peek() == Some(b'.') {
            self.at += 1;
            if self.digits() == 0 {
                return self.fail("A number with no digits after its point");
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.at += 1;
            }
            if self.digits() == 0 {
                return self.fail("A number with no digits in its exponent");
            }
        }
        // What may follow a number: the end, white space, a separator, a closing bracket or a comment.
        if !matches!(self.peek(), None | Some(b',' | b'}' | b']' | b' ' | b'\t' | b'\n' | b'\r' | b'/')) {
            return self.fail("A number that runs into something else");
        }
        Ok(Value::Number(String::from_utf8_lossy(&self.bytes[start..self.at]).into_owned()))
    }

    fn hex4(&mut self) -> Result<u32, Error> {
        let Some(digits) = self.bytes.get(self.at..self.at + 4) else {
            return self.fail("An escape cut short");
        };
        let mut unit = 0u32;
        for &digit in digits {
            let Some(value) = (digit as char).to_digit(16) else {
                return self.fail("An escape that is not four hexadecimal digits");
            };
            unit = unit * 16 + value;
        }
        self.at += 4;
        Ok(unit)
    }

    fn string(&mut self) -> Result<Text, Error> {
        self.at += 1;
        let mut out: Vec<u8> = Vec::new();
        let mut broken = false;
        loop {
            let Some(byte) = self.peek() else {
                return self.fail("A string that never ends");
            };
            self.at += 1;
            match byte {
                b'"' => break,
                0..=0x1F => {
                    self.at -= 1;
                    return self.fail("A control character inside a string");
                }
                b'\\' => {
                    let Some(escape) = self.peek() else {
                        return self.fail("A string that never ends");
                    };
                    self.at += 1;
                    match escape {
                        b'"' | b'\\' | b'/' => out.push(escape),
                        b'b' => out.push(0x08),
                        b'f' => out.push(0x0C),
                        b'n' => out.push(b'\n'),
                        b'r' => out.push(b'\r'),
                        b't' => out.push(b'\t'),
                        b'u' => {
                            let unit = self.hex4()?;
                            let scalar = match unit {
                                0xD800..=0xDBFF => {
                                    // A high surrogate counts only with the low one written straight after it.
                                    let after = self.at;
                                    let low = if self.bytes[self.at..].starts_with(b"\\u") {
                                        self.at += 2;
                                        self.hex4().ok().filter(|low| (0xDC00..=0xDFFF).contains(low))
                                    } else {
                                        None
                                    };
                                    match low {
                                        Some(low) => char::from_u32(0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00)),
                                        None => {
                                            self.at = after;
                                            None
                                        }
                                    }
                                }
                                0xDC00..=0xDFFF => None,
                                _ => char::from_u32(unit),
                            };
                            match scalar {
                                Some(scalar) => out.extend_from_slice(scalar.encode_utf8(&mut [0; 4]).as_bytes()),
                                None => {
                                    broken = true;
                                    out.extend_from_slice("\u{FFFD}".as_bytes());
                                }
                            }
                        }
                        _ => {
                            self.at -= 1;
                            return self.fail("An escape that JSON does not have");
                        }
                    }
                }
                _ => out.push(byte),
            }
        }
        // The input was a str and only whole characters were copied or appended.
        Ok(Text { text: String::from_utf8(out).expect("the text was UTF-8 and was cut only at ASCII"), broken })
    }

    fn enter(&mut self) -> Result<(), Error> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return self.fail("Nested more than 64 deep");
        }
        self.at += 1;
        Ok(())
    }

    fn array(&mut self) -> Result<Value, Error> {
        self.enter()?;
        let mut items = Vec::new();
        loop {
            self.skip_filler()?;
            if self.peek() == Some(b']') {
                break;
            }
            items.push(self.value()?);
            self.skip_filler()?;
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b']') => break,
                _ => return self.fail("Neither ',' nor ']' after an array's value"),
            }
        }
        self.at += 1;
        self.depth -= 1;
        Ok(Value::Array(items))
    }

    fn object(&mut self) -> Result<Value, Error> {
        self.enter()?;
        let mut members = Vec::new();
        loop {
            self.skip_filler()?;
            match self.peek() {
                Some(b'}') => break,
                Some(b'"') => {}
                _ => return self.fail("Neither a name nor '}' in an object"),
            }
            let name = self.string()?;
            self.skip_white_space();
            if self.peek() != Some(b':') {
                return self.fail("No ':' after a name");
            }
            self.at += 1;
            self.skip_filler()?;
            members.push((name, self.value()?));
            self.skip_filler()?;
            match self.peek() {
                Some(b',') => self.at += 1,
                Some(b'}') => break,
                _ => return self.fail("Neither ',' nor '}' after an object's value"),
            }
        }
        self.at += 1;
        self.depth -= 1;
        Ok(Value::Object(members))
    }
}

/// Reads one JSON document as .NET reads a config file: comments skipped, a comma allowed before a closing bracket, at most 64 deep.
pub fn parse(text: &str) -> Result<Value, Error> {
    let mut reader = Reader { bytes: text.as_bytes(), at: 0, depth: 0 };
    reader.skip_filler()?;
    let value = reader.value()?;
    reader.skip_filler()?;
    if reader.at != reader.bytes.len() {
        return reader.fail("Something after the document's end");
    }
    Ok(value)
}

/// A string between its quotes as .NET's default encoder writes it: ASCII alone, and of that not the characters HTML gives a meaning.
pub fn escape_into(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '"' | '&' | '\'' | '+' | '<' | '>' | '`' => out.push_str(&format!("\\u{:04X}", c as u32)),
            ' '..='~' => out.push(c),
            _ => {
                for unit in c.encode_utf16(&mut [0; 2]) {
                    out.push_str(&format!("\\u{unit:04X}"));
                }
            }
        }
    }
}

/// A string with its quotes, as .NET writes it.
pub fn quoted(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    escape_into(&mut out, text);
    out.push('"');
    out
}

/// .NET's general format for a floating-point number from its shortest digits: `digits` without a point, the first worth 10^`exponent`.
fn dotnet_general(negative: bool, digits: &str, exponent: i32, round_trip_digits: usize) -> String {
    let count = digits.len() as i32;
    // Where the point falls, counted from before the first digit.
    let scale = exponent + 1;
    let mut out = String::from(if negative { "-" } else { "" });
    if scale > count.max(round_trip_digits as i32) || scale < -3 {
        out.push_str(&digits[..1]);
        if count > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        out.push_str(&format!("E{}{:02}", if exponent < 0 { '-' } else { '+' }, exponent.abs()));
    } else if scale <= 0 {
        out.push_str("0.");
        out.push_str(&"0".repeat(-scale as usize));
        out.push_str(digits);
    } else if scale >= count {
        out.push_str(digits);
        out.push_str(&"0".repeat((scale - count) as usize));
    } else {
        out.push_str(&digits[..scale as usize]);
        out.push('.');
        out.push_str(&digits[scale as usize..]);
    }
    out
}

/// A number in Rust's scientific form as its digits and the power of ten of the first.
fn digits_of(scientific: &str) -> (String, i32) {
    let (mantissa, exponent) = scientific.split_once('e').expect("Rust's {:e} always has an exponent");
    (mantissa.replace('.', ""), exponent.parse().expect("Rust's exponent is an integer"))
}

/// The shortest digits that read back as the number, as .NET chooses them. `shortest` is Rust's own choice and `rounded` the number to a given count of digits after the first, exactly.
///
/// The two agree except where the number lies exactly half way between two candidates of the shortest length: Rust takes the upper and .NET the one whose last digit is even.
fn shortest(shortest: &str, rounded: impl Fn(usize) -> String, most_digits: usize) -> (String, i32) {
    let (digits, exponent) = digits_of(shortest);
    // One more digit, rounded: only a number that then ends in 5 can be a tie, and nine in ten do not.
    if !digits_of(&rounded(digits.len())).0.ends_with('5') {
        return (digits, exponent);
    }
    let (exact, exact_exponent) = digits_of(&rounded(most_digits));
    let exact = exact.trim_end_matches('0');
    let lower = &exact[..exact.len().min(digits.len())];
    let tie = exact.len() == digits.len() + 1 && exact.ends_with('5');
    if tie && lower.as_bytes()[lower.len() - 1] % 2 == 0 { (lower.to_string(), exact_exponent) } else { (digits, exponent) }
}

/// A `double` as .NET writes it into JSON; none for a value JSON has no number for, which .NET refuses to write.
pub fn format_f64(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    if value == 0.0 {
        return Some(if value.is_sign_negative() { "-0" } else { "0" }.to_string());
    }
    let value_abs = value.abs();
    // A double's digits end within 767 of the first, so that many are all of them.
    let (digits, exponent) = shortest(&format!("{value_abs:e}"), |after| format!("{value_abs:.after$e}"), 770);
    Some(dotnet_general(value < 0.0, &digits, exponent, 17))
}

/// A `float` as .NET writes it into JSON, from the float's own shortest digits.
pub fn format_f32(value: f32) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    if value == 0.0 {
        return Some(if value.is_sign_negative() { "-0" } else { "0" }.to_string());
    }
    let value_abs = value.abs();
    let (digits, exponent) = shortest(&format!("{value_abs:e}"), |after| format!("{value_abs:.after$e}"), 120);
    Some(dotnet_general(value < 0.0, &digits, exponent, 9))
}

/// A value as compact JSON any reader takes: numbers by their tokens, strings escaped no further than JSON requires.
pub fn write_compact(out: &mut String, value: &Value) {
    fn text(out: &mut String, text: &str) {
        out.push('"');
        for c in text.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\u{0}'..='\u{1F}' => out.push_str(&format!("\\u{:04X}", c as u32)),
                _ => out.push(c),
            }
        }
        out.push('"');
    }
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(token) => out.push_str(token),
        Value::String(value) => text(out, &value.text),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_compact(out, item);
            }
            out.push(']');
        }
        Value::Object(members) => {
            out.push('{');
            for (i, (name, member)) in members.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                text(out, &name.text);
                out.push(':');
                write_compact(out, member);
            }
            out.push('}');
        }
    }
}

/// A file's bytes as `File.ReadAllText` gives them: a byte-order mark chooses UTF-8, UTF-16 or UTF-32 and is dropped, and what is not valid becomes U+FFFD.
pub fn decode(bytes: &[u8]) -> String {
    fn utf16(bytes: &[u8], unit: fn([u8; 2]) -> u16) -> String {
        let whole: Vec<u16> = bytes.as_chunks::<2>().0.iter().map(|pair| unit(*pair)).collect();
        let mut text: String = char::decode_utf16(whole).map(|c| c.unwrap_or('\u{FFFD}')).collect();
        if bytes.len() % 2 == 1 {
            text.push('\u{FFFD}');
        }
        text
    }
    fn utf32(bytes: &[u8], unit: fn([u8; 4]) -> u32) -> String {
        bytes.chunks(4).map(|four| <[u8; 4]>::try_from(four).ok().and_then(|four| char::from_u32(unit(four))).unwrap_or('\u{FFFD}')).collect()
    }
    match bytes {
        [0xFE, 0xFF, rest @ ..] => utf16(rest, u16::from_be_bytes),
        [0xFF, 0xFE, 0, 0, rest @ ..] => utf32(rest, u32::from_le_bytes),
        [0xFF, 0xFE, rest @ ..] => utf16(rest, u16::from_le_bytes),
        [0xEF, 0xBB, 0xBF, rest @ ..] => String::from_utf8_lossy(rest).into_owned(),
        [0, 0, 0xFE, 0xFF, rest @ ..] => utf32(rest, u32::from_be_bytes),
        _ => String::from_utf8_lossy(bytes).into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn object(members: &[(&str, Value)]) -> Value {
        Value::Object(members.iter().map(|(name, value)| (Text::new(*name), value.clone())).collect())
    }

    #[test]
    fn a_document_is_read_with_its_order_its_duplicates_and_its_number_tokens() {
        let value = parse(r#" {"b": 1.50, "a": [true, null, "x"], "b": -0} "#).unwrap();
        assert_eq!(value, object(&[("b", Value::number("1.50")), ("a", Value::Array(vec![Value::Bool(true), Value::Null, Value::string("x")])), ("b", Value::number("-0"))]));
        assert_eq!(value.get("b"), Some(&Value::number("-0")));
    }

    #[test]
    fn comments_go_anywhere_but_between_a_name_and_its_colon() {
        for text in [
            r#"{/*c*/"I":1}"#,
            r#"{"I":/*c*/1}"#,
            r#"{"I":1/*c*/}"#,
            r#"{"I":1,/*c*/}"#,
            r#"{"I":1/*c*/,}"#,
            r#"[/*c*/1/*c*/,/*c*/2,/*c*/]"#,
            "{\"I\"://c\n1}",
            r#"{"I":1}/*c*//*d*/ //e"#,
            "/**/null",
            "{\"I\":1//\r}",
            r#"{"I":1/* */}"#,
        ] {
            assert!(parse(text).is_ok(), "{text}");
        }
        for text in [r#"{"I"/*c*/:1}"#, "{\"I\"//c\n:1}", r#"{"I":1}/*c"#, r#"{"I":1/}"#, r#"{"I":1/*/}"#, "//", "/**/", "{ // c", "{\"I\":1//c\u{2028}\n}"] {
            assert!(parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn a_comma_may_end_a_list_and_nothing_else_is_forgiven() {
        for text in [r#"{"I":1,}"#, "[1,2,]", "{}", "[]", " null ", "{\"I\" :\t1 , \"L\"\n:\r2}"] {
            assert!(parse(text).is_ok(), "{text}");
        }
        for text in [
            "{,}", r#"{"I":1,,}"#, "[,]", "[1,,2]", "", "  ", "nul", "{} x", r#"{"I":1 "S":"x"}"#, r#"{"I" 1}"#, "{I:1}", "{'I':1}", r#"{"I":1"#, r#"{"S":"abc"#, "[1 2]", "[]{}",
            "{\"I\":1}\u{A0}", "{\"I\":1}\u{C}", "\u{FEFF}{}", "{\"I\":1\u{B}}",
        ] {
            assert!(parse(text).is_err(), "{text}");
        }
    }

    #[test]
    fn numbers_follow_json_and_not_what_a_person_might_type() {
        for token in ["0", "-0", "1", "10", "1.5", "-0.0", "1e2", "1E5", "1.5E+2", "0e0", "1e999", "10000000000000000000000"] {
            assert_eq!(parse(token), Ok(Value::number(token)), "{token}");
        }
        for token in ["01", "-01", "1.", ".5", "-.5", "+1", "-", "1e", "1e+", "1.e2", "00", "0.", "1x"] {
            assert!(parse(token).is_err(), "{token}");
        }
    }

    #[test]
    fn strings_are_unescaped_and_a_broken_surrogate_is_noted_not_refused() {
        let read = |text: &str| parse(text).map(|value| match value {
            Value::String(text) => (text.text, text.broken),
            _ => panic!("not a string"),
        });
        assert_eq!(read(r#""a\"b\\c\/d\b\f\n\r\t""#), Ok(("a\"b\\c/d\u{8}\u{C}\n\r\t".to_string(), false)));
        assert_eq!(read(r#""\u00e9\u00C9\ud83d\ude00""#), Ok(("éÉ😀".to_string(), false)));
        assert_eq!(read("\"\u{7F}\u{80}é\""), Ok(("\u{7F}\u{80}é".to_string(), false)));
        for broken in [r#""\ud800""#, r#""\ude00""#, r#""\ud83dx""#, r#""\ud83d\n""#, r#""\ud83d\ud83d""#] {
            assert!(read(broken).unwrap().1, "{broken}");
        }
        assert_eq!(read(r#""\ud83d\n""#).unwrap().0, "\u{FFFD}\n");
        for refused in ["\"a\tb\"", "\"a\nb\"", r#""\x""#, r#""\U00e9""#, r#""\u00E""#, r#""\uZZZZ""#, "\"\\"] {
            assert!(read(refused).is_err(), "{refused}");
        }
    }

    #[test]
    fn sixty_four_deep_is_read_and_sixty_five_is_not() {
        let nested = |depth: usize| format!("{}{}", "[".repeat(depth), "]".repeat(depth));
        assert!(parse(&nested(64)).is_ok());
        assert!(parse(&nested(65)).is_err());
        assert!(parse(&format!("{{\"U\":{}}}", nested(63))).is_ok());
        assert!(parse(&format!("{{\"U\":{}}}", nested(64))).is_err());
    }

    #[test]
    fn an_error_says_where() {
        let error = parse("{\n  \"a\": 1,\n  \"b\": nope\n}").unwrap_err();
        assert_eq!((error.line, error.byte), (3, 8));
        assert_eq!(error.to_string(), "Not a value at line 3, byte 8.");
    }

    #[test]
    fn strings_are_escaped_as_dotnet_escapes_them() {
        assert_eq!(quoted("é<>&'+\"\\`\t\u{1}/€😀"), r#""\u00E9\u003C\u003E\u0026\u0027\u002B\u0022\\\u0060\t\u0001/\u20AC\uD83D\uDE00""#);
        assert_eq!(quoted("\u{8}\u{C}\n\r\t\u{7F}\u{0}\u{1F} /~\u{80}\u{2028}\u{FFFD}"), r#""\b\f\n\r\t\u007F\u0000\u001F /~\u0080\u2028\uFFFD""#);
    }

    // The ties are written with every digit they have, which is one more than the shortest that reads back.
    #[allow(clippy::excessive_precision)]
    #[test]
    fn doubles_are_laid_out_as_dotnet_lays_them_out() {
        let cases: [(f64, &str); 29] = [
            (1e14, "100000000000000"),
            (1e15, "1000000000000000"),
            (1e16, "10000000000000000"),
            (1e17, "1E+17"),
            (123456789012345678.0, "1.2345678901234568E+17"),
            (1234567890123456.0, "1234567890123456"),
            (0.0001, "0.0001"),
            (0.00001, "1E-05"),
            (0.00012345, "0.00012345"),
            (1.5e-5, "1.5E-05"),
            (5e-324, "5E-324"),
            (f64::MAX, "1.7976931348623157E+308"),
            (100.0, "100"),
            (0.1, "0.1"),
            (123.456, "123.456"),
            (1e21, "1E+21"),
            (12345678901234567890.0, "1.2345678901234567E+19"),
            (9007199254740993.0, "9007199254740992"),
            (0.1 + 0.2, "0.30000000000000004"),
            (-1e15, "-1000000000000000"),
            (4.94e-322, "4.94E-322"),
            (0.0, "0"),
            (-0.0, "-0"),
            (1.0, "1"),
            // Half way between two candidates of the shortest length: the even one, where Rust alone would take the upper.
            (1186865448985138.25, "1186865448985138.2"),
            (1186865448985138.75, "1186865448985138.8"),
            (96572437558383.125, "96572437558383.12"),
            (96572437558383.375, "96572437558383.38"),
            (4503599627370495.5, "4503599627370495.5"),
        ];
        for (value, text) in cases {
            assert_eq!(format_f64(value).as_deref(), Some(text));
        }
        assert_eq!(format_f64(f64::INFINITY), None);
        assert_eq!(format_f64(f64::NAN), None);
    }

    #[allow(clippy::excessive_precision)]
    #[test]
    fn floats_keep_their_own_width() {
        let cases: [(f32, &str); 22] = [
            (1e6, "1000000"),
            (1e7, "10000000"),
            (1e8, "100000000"),
            (1234567.0, "1234567"),
            (12345678.0, "12345678"),
            (123456789.0, "123456790"),
            (0.0001, "0.0001"),
            (0.00001, "1E-05"),
            (1.5e-5, "1.5E-05"),
            (0.8, "0.8"),
            (16777216.0, "16777216"),
            (f32::MAX, "3.4028235E+38"),
            (1e-45, "1E-45"),
            (0.1, "0.1"),
            (100.0, "100"),
            (1e9, "1E+09"),
            (0.3, "0.3"),
            (1482412.25, "1482412.2"),
            (1482412.75, "1482412.8"),
            (520306.125, "520306.12"),
            (520306.875, "520306.88"),
            (8388607.5, "8388607.5"),
        ];
        for (value, text) in cases {
            assert_eq!(format_f32(value).as_deref(), Some(text));
        }
        assert_eq!(format_f32(f32::NEG_INFINITY), None);
    }

    #[test]
    fn a_files_bytes_are_decoded_by_their_mark() {
        assert_eq!(decode(b"{}"), "{}");
        assert_eq!(decode(b"\xEF\xBB\xBF{\"a\":\"\xFF\"}"), "{\"a\":\"\u{FFFD}\"}");
        assert_eq!(decode(b"\xFF\xFE{\0}\0"), "{}");
        assert_eq!(decode(b"\xFE\xFF\0{\0}"), "{}");
        assert_eq!(decode(b"\xFF\xFE\0\0{\0\0\0}\0\0\0"), "{}");
        assert_eq!(decode(b"\0\0\xFE\xFF\0\0\0{\0\0\0}"), "{}");
        assert_eq!(decode(b"\xFF\xFE{\0}"), "{\u{FFFD}");
        assert_eq!(decode(b"\xFF\xFE\x00\xD8x\0"), "\u{FFFD}x");
        assert_eq!(decode(b""), "");
    }

    #[test]
    fn the_compact_form_reads_back_as_itself() {
        let value = parse(r#"{"a":[1.50,-0,1e999],"b\u0000\"\\":"é\n","c":{"d":null,"e":true,"f":false}}"#).unwrap();
        let mut text = String::new();
        write_compact(&mut text, &value);
        assert_eq!(text, "{\"a\":[1.50,-0,1e999],\"b\\u0000\\\"\\\\\":\"é\\u000A\",\"c\":{\"d\":null,\"e\":true,\"f\":false}}");
        assert_eq!(parse(&text), Ok(value));
    }
}
