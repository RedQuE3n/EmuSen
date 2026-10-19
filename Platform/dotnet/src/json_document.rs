//! JSON as `JsonDocument.Parse(string)` reads it with its default options: no comments, no trailing comma, at most 64
//! deep, and System.Text.Json's words for every refusal, `LineNumber` and `BytePositionInLine` included. The scraper
//! reads ScreenScraper's answers by it, and an answer's refusal is kept in `media.db`, so the words are the record.
//! Each rule was measured against .NET 10 on 2026-10-18 (EmuSen_Backend.md §30.1); the tests below are the measurements.

use crate::json::{Text, Value};

const DEPTH: usize = 64;

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
    line: usize,
    line_start: usize,
    depth: usize,
}

/// A byte as System.Text.Json names it in a message: itself when printable ASCII, else `0x` and two hexadecimal digits.
fn printable(byte: u8) -> String {
    if (0x20..0x7F).contains(&byte) { (byte as char).to_string() } else { format!("0x{byte:02X}") }
}

const DEPTH_AT_END: &str = "Expected depth to be zero at the end of the JSON payload. There is an open JSON object or array that should be closed.";
const DIGIT_AT_END: &str = "Expected a digit ('0'-'9'), but instead reached end of data.";
const STRING_AT_END: &str = "Expected end of string, but instead reached end of data.";

impl Reader<'_> {
    fn fail<T>(&self, what: impl AsRef<str>, at: usize) -> Result<T, String> {
        Err(format!("{} LineNumber: {} | BytePositionInLine: {}.", what.as_ref(), self.line, at - self.line_start))
    }

    fn end(&self) -> bool {
        self.at >= self.b.len()
    }

    fn skip_white_space(&mut self) {
        while let Some(&c) = self.b.get(self.at) {
            match c {
                b'\n' => {
                    self.at += 1;
                    self.line += 1;
                    self.line_start = self.at;
                }
                b' ' | b'\t' | b'\r' => self.at += 1,
                _ => break,
            }
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        match self.b[self.at] {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => Ok(Value::String(self.string()?)),
            b't' => self.literal(b"true", Value::Bool(true)),
            b'f' => self.literal(b"false", Value::Bool(false)),
            b'n' => self.literal(b"null", Value::Null),
            b'-' | b'0'..=b'9' => self.number(),
            c => self.fail(format!("'{}' is an invalid start of a value.", printable(c)), self.at),
        }
    }

    fn literal(&mut self, word: &[u8], value: Value) -> Result<Value, String> {
        let rest = &self.b[self.at..];
        if rest.starts_with(word) {
            self.at += word.len();
            return Ok(value);
        }
        let mismatch = (1..word.len()).find(|&i| rest.get(i) != Some(&word[i])).unwrap_or(word.len());
        let what = format!("'{}' is an invalid JSON literal. Expected the literal '{}'.", String::from_utf8_lossy(rest), String::from_utf8_lossy(word));
        self.fail(what, self.at + mismatch)
    }

    fn digits(&mut self) {
        while matches!(self.b.get(self.at), Some(b'0'..=b'9')) {
            self.at += 1;
        }
    }

    /// A digit must come next, after the sign or point or exponent named.
    fn digit_after(&mut self, after: &str) -> Result<(), String> {
        match self.b.get(self.at) {
            None => self.fail(DIGIT_AT_END, self.at),
            Some(b'0'..=b'9') => Ok(()),
            Some(&c) => self.fail(format!("'{}' is invalid within a number, immediately after {after}. Expected a digit ('0'-'9').", printable(c)), self.at),
        }
    }

    fn number(&mut self) -> Result<Value, String> {
        let start = self.at;
        if self.b[self.at] == b'-' {
            self.at += 1;
            self.digit_after("a sign character ('+' or '-')")?;
        }
        if self.b[self.at] == b'0' {
            self.at += 1;
            if let Some(&c @ b'0'..=b'9') = self.b.get(self.at) {
                return self.fail(format!("Invalid leading zero before '{}'.", printable(c)), self.at);
            }
        } else {
            self.digits();
        }
        if self.b.get(self.at) == Some(&b'.') {
            self.at += 1;
            self.digit_after("a decimal point ('.')")?;
            self.digits();
        }
        if matches!(self.b.get(self.at), Some(b'e' | b'E')) {
            self.at += 1;
            if matches!(self.b.get(self.at), Some(b'+' | b'-')) {
                self.at += 1;
            }
            // .NET names the sign here whether or not one was written.
            self.digit_after("a sign character ('+' or '-')")?;
            self.digits();
        }
        match self.b.get(self.at) {
            None if self.depth > 0 => {
                return self.fail(format!("'{}' is an invalid end of a number. Expected a delimiter.", printable(self.b[self.at - 1])), self.at);
            }
            Some(&c) if !matches!(c, b',' | b'}' | b']' | b' ' | b'\n' | b'\r' | b'\t' | b'/') => {
                return self.fail(format!("'{}' is an invalid end of a number. Expected a delimiter.", printable(c)), self.at);
            }
            _ => {}
        }
        Ok(Value::Number(String::from_utf8_lossy(&self.b[start..self.at]).into_owned()))
    }

    fn string(&mut self) -> Result<Text, String> {
        let start = self.at + 1;
        self.at += 1;
        loop {
            let Some(&c) = self.b.get(self.at) else { return self.fail(STRING_AT_END, self.b.len()) };
            match c {
                b'"' => break,
                b'\\' => {
                    self.at += 1;
                    let Some(&e) = self.b.get(self.at) else { return self.fail(STRING_AT_END, self.b.len()) };
                    match e {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => self.at += 1,
                        b'u' => {
                            for k in 1..=4 {
                                let Some(&h) = self.b.get(self.at + k) else { return self.fail(STRING_AT_END, self.b.len()) };
                                if !h.is_ascii_hexdigit() {
                                    let what = format!("'{}' is not a hex digit following '\\u' within a JSON string. The string should be correctly escaped.", printable(h));
                                    return self.fail(what, self.at + k);
                                }
                            }
                            self.at += 5;
                        }
                        _ => {
                            let what = format!("'{}' is an invalid escapable character within a JSON string. The string should be correctly escaped.", printable(e));
                            return self.fail(what, self.at);
                        }
                    }
                }
                0..=0x1F => return self.fail(format!("'{}' is invalid within a JSON string. The string should be correctly escaped.", printable(c)), self.at),
                _ => self.at += 1,
            }
        }
        let raw = &self.b[start..self.at];
        self.at += 1;
        Ok(unescape(raw))
    }

    fn enter(&mut self, what: &str) -> Result<(), String> {
        if self.depth >= DEPTH {
            return self.fail(format!("The maximum configured depth of {DEPTH} has been exceeded. Cannot read next JSON {what}."), self.at);
        }
        self.depth += 1;
        self.at += 1;
        Ok(())
    }

    /// After a comma: white space, then the end is refused at the comma when nothing followed it, else where the space ended.
    fn after_comma(&mut self) -> Result<(), String> {
        let comma = self.at;
        self.at += 1;
        let spaced = self.at;
        self.skip_white_space();
        if self.end() {
            let at = if self.at == spaced { comma } else { self.at };
            return self.fail("Expected start of a property name or value, but instead reached end of data.", at);
        }
        Ok(())
    }

    /// After a value inside a container: a comma (true), the container's own closer (false), or a refusal.
    fn after_value(&mut self, closer: u8) -> Result<bool, String> {
        self.skip_white_space();
        match self.b.get(self.at) {
            None => self.fail(DEPTH_AT_END, self.at),
            Some(b',') => {
                self.after_comma()?;
                Ok(true)
            }
            Some(&c) if c == closer => Ok(false),
            Some(&c @ (b'}' | b']')) => self.fail(format!("'{}' is invalid without a matching open.", printable(c)), self.at),
            Some(&c) => self.fail(format!("'{}' is invalid after a value. Expected either ',', '}}', or ']'.", printable(c)), self.at),
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.enter("object")?;
        let mut members = Vec::new();
        self.skip_white_space();
        match self.b.get(self.at) {
            None => return self.fail(DEPTH_AT_END, self.at),
            Some(b'}') => {}
            Some(_) => loop {
                match self.b[self.at] {
                    b'"' => {}
                    b'}' if !members.is_empty() => {
                        return self.fail("The JSON object contains a trailing comma at the end which is not supported in this mode. Change the reader options.", self.at);
                    }
                    c => return self.fail(format!("'{}' is an invalid start of a property name. Expected a '\"'.", printable(c)), self.at),
                }
                let name = self.string()?;
                self.skip_white_space();
                match self.b.get(self.at) {
                    None => return self.fail("Expected a value, but instead reached end of data.", self.at),
                    Some(b':') => self.at += 1,
                    Some(&c) => return self.fail(format!("'{}' is invalid after a property name. Expected a ':'.", printable(c)), self.at),
                }
                self.skip_white_space();
                if self.end() {
                    return self.fail(DEPTH_AT_END, self.at);
                }
                members.push((name, self.value()?));
                if !self.after_value(b'}')? {
                    break;
                }
            },
        }
        self.at += 1;
        self.depth -= 1;
        Ok(Value::Object(members))
    }

    fn array(&mut self) -> Result<Value, String> {
        self.enter("array")?;
        let mut items = Vec::new();
        self.skip_white_space();
        match self.b.get(self.at) {
            None => return self.fail(DEPTH_AT_END, self.at),
            Some(b']') => {}
            Some(_) => loop {
                if self.b[self.at] == b']' && !items.is_empty() {
                    return self.fail("The JSON array contains a trailing comma at the end which is not supported in this mode. Change the reader options.", self.at);
                }
                items.push(self.value()?);
                if !self.after_value(b']')? {
                    break;
                }
            },
        }
        self.at += 1;
        self.depth -= 1;
        Ok(Value::Array(items))
    }
}

/// A string's text between its quotes, its escapes read. A lone surrogate's escape gives U+FFFD and marks the text
/// broken, with the words `JsonElement.GetString` refuses the first one in: a high surrogate with no `\u` straight
/// after it is incomplete; a low one first, or a high one followed by an escape that is not a low one, names that value.
fn unescape(raw: &[u8]) -> Text {
    let mut out: Vec<u8> = Vec::with_capacity(raw.len());
    let mut refusal: Option<String> = None;
    let hex = |at: usize| -> Option<u32> { u32::from_str_radix(std::str::from_utf8(raw.get(at..at + 4)?).ok()?, 16).ok() };
    let invalid = |value: u32| format!("Cannot read invalid UTF-16 JSON text as string. Invalid surrogate value: '0x{value:02X}'.");
    let mut i = 0;
    while i < raw.len() {
        if raw[i] != b'\\' {
            out.push(raw[i]);
            i += 1;
            continue;
        }
        let e = raw[i + 1];
        i += 2;
        match e {
            b'b' => out.push(0x08),
            b'f' => out.push(0x0C),
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b't' => out.push(b'\t'),
            b'u' => {
                let unit = hex(i).expect("the reader checked four hexadecimal digits");
                i += 4;
                let scalar = match unit {
                    0xD800..=0xDBFF => {
                        let next = (raw.get(i..i + 2) == Some(b"\\u")).then(|| hex(i + 2)).flatten();
                        match next {
                            Some(low @ 0xDC00..=0xDFFF) => {
                                i += 6;
                                char::from_u32(0x10000 + ((unit - 0xD800) << 10) + (low - 0xDC00))
                            }
                            Some(other) => {
                                refusal.get_or_insert_with(|| invalid(other));
                                None
                            }
                            None => {
                                refusal.get_or_insert_with(|| "Cannot read incomplete UTF-16 JSON text as string with missing low surrogate.".to_string());
                                None
                            }
                        }
                    }
                    0xDC00..=0xDFFF => {
                        refusal.get_or_insert_with(|| invalid(unit));
                        None
                    }
                    _ => char::from_u32(unit),
                };
                match scalar {
                    Some(c) => out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes()),
                    None => out.extend_from_slice("\u{FFFD}".as_bytes()),
                }
            }
            other => out.push(other),
        }
    }
    Text { text: String::from_utf8(out).expect("the text was UTF-8 and was cut only at ASCII"), broken: refusal.is_some(), refusal }
}

/// One JSON document as `JsonDocument.Parse` reads it, or its `JsonReaderException`'s message.
pub fn parse(text: &str) -> Result<Value, String> {
    let mut reader = Reader { b: text.as_bytes(), at: 0, line: 0, line_start: 0, depth: 0 };
    reader.skip_white_space();
    if reader.end() {
        return reader.fail("The input does not contain any JSON tokens. Expected the input to start with a valid JSON token, when isFinalBlock is true.", reader.at);
    }
    let value = reader.value()?;
    reader.skip_white_space();
    if let Some(&c) = reader.b.get(reader.at) {
        return reader.fail(format!("'{}' is invalid after a single JSON value. Expected end of data.", printable(c)), reader.at);
    }
    Ok(value)
}

/// One JSON value as `JsonDocument.ParseValue(ref reader)` reads it from a fresh reader over the whole text: as `parse`,
/// except that what follows the value is never read, so it is not refused.
pub fn parse_value(text: &str) -> Result<Value, String> {
    let mut reader = Reader { b: text.as_bytes(), at: 0, line: 0, line_start: 0, depth: 0 };
    reader.skip_white_space();
    if reader.end() {
        return reader.fail("The input does not contain any JSON tokens. Expected the input to start with a valid JSON token, when isFinalBlock is true.", reader.at);
    }
    reader.value()
}

#[cfg(test)]
mod tests;
