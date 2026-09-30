//! A small JSON reader for the single-step suites' files (SingleStepTests), so that no core's tests need a
//! dependency to read them. It reads the whole of RFC 8259 but keeps integers apart from other numbers.
//! See VenusRT_Native.md §2.3.

/// One JSON value; an object keeps its members in file order.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

/// Where the text stopped being JSON, as a byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Error {
    pub at: usize,
}

impl Value {
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match *self {
            Value::Int(n) => Some(n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
}

/// The one value `text` holds, with nothing but whitespace around it.
pub fn parse(text: &[u8]) -> Result<Value, Error> {
    let mut p = Parser { s: text, i: 0 };
    let v = p.value(0)?;
    p.ws();
    if p.i != text.len() {
        return Err(Error { at: p.i });
    }
    Ok(v)
}

const MAX_DEPTH: usize = 128;

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn err<T>(&self) -> Result<T, Error> {
        Err(Error { at: self.i })
    }

    fn ws(&mut self) {
        while let Some(b' ' | b'\t' | b'\n' | b'\r') = self.s.get(self.i) {
            self.i += 1;
        }
    }

    fn eat(&mut self, word: &[u8]) -> Result<(), Error> {
        if self.s[self.i..].starts_with(word) {
            self.i += word.len();
            Ok(())
        } else {
            self.err()
        }
    }

    fn value(&mut self, depth: usize) -> Result<Value, Error> {
        if depth > MAX_DEPTH {
            return self.err();
        }
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                let mut members = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b'}') {
                    self.i += 1;
                    return Ok(Value::Object(members));
                }
                loop {
                    self.ws();
                    if self.s.get(self.i) != Some(&b'"') {
                        return self.err();
                    }
                    let key = self.string()?;
                    self.ws();
                    self.eat(b":")?;
                    members.push((key, self.value(depth + 1)?));
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b'}') => {
                            self.i += 1;
                            return Ok(Value::Object(members));
                        }
                        _ => return self.err(),
                    }
                }
            }
            Some(b'[') => {
                self.i += 1;
                let mut items = Vec::new();
                self.ws();
                if self.s.get(self.i) == Some(&b']') {
                    self.i += 1;
                    return Ok(Value::Array(items));
                }
                loop {
                    items.push(self.value(depth + 1)?);
                    self.ws();
                    match self.s.get(self.i) {
                        Some(b',') => self.i += 1,
                        Some(b']') => {
                            self.i += 1;
                            return Ok(Value::Array(items));
                        }
                        _ => return self.err(),
                    }
                }
            }
            Some(b'"') => Ok(Value::Str(self.string()?)),
            Some(b't') => self.eat(b"true").map(|()| Value::Bool(true)),
            Some(b'f') => self.eat(b"false").map(|()| Value::Bool(false)),
            Some(b'n') => self.eat(b"null").map(|()| Value::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => self.err(),
        }
    }

    fn number(&mut self) -> Result<Value, Error> {
        let start = self.i;
        if self.s.get(self.i) == Some(&b'-') {
            self.i += 1;
        }
        let digits = |p: &mut Self| {
            let from = p.i;
            while let Some(b'0'..=b'9') = p.s.get(p.i) {
                p.i += 1;
            }
            p.i - from
        };
        let int_digits = digits(self);
        if int_digits == 0 || (int_digits > 1 && self.s[self.i - int_digits] == b'0') {
            return self.err();
        }
        let mut integral = true;
        if self.s.get(self.i) == Some(&b'.') {
            self.i += 1;
            integral = false;
            if digits(self) == 0 {
                return self.err();
            }
        }
        if let Some(b'e' | b'E') = self.s.get(self.i) {
            self.i += 1;
            integral = false;
            if let Some(b'+' | b'-') = self.s.get(self.i) {
                self.i += 1;
            }
            if digits(self) == 0 {
                return self.err();
            }
        }
        let text = std::str::from_utf8(&self.s[start..self.i]).expect("ASCII");
        if integral {
            if let Ok(n) = text.parse::<i64>() {
                return Ok(Value::Int(n));
            }
        }
        text.parse::<f64>().map(Value::Float).or(Err(Error { at: start }))
    }

    fn hex4(&mut self) -> Result<u32, Error> {
        let digits = self.s.get(self.i..self.i + 4).ok_or(Error { at: self.i })?;
        let text = std::str::from_utf8(digits).map_err(|_| Error { at: self.i })?;
        let v = u32::from_str_radix(text, 16).map_err(|_| Error { at: self.i })?;
        self.i += 4;
        Ok(v)
    }

    fn string(&mut self) -> Result<String, Error> {
        self.i += 1;
        let mut out = Vec::new();
        loop {
            match self.s.get(self.i) {
                None => return self.err(),
                Some(b'"') => {
                    self.i += 1;
                    return String::from_utf8(out).or(Err(Error { at: self.i }));
                }
                Some(b'\\') => {
                    self.i += 1;
                    let c = match self.s.get(self.i) {
                        Some(b'"') => '"',
                        Some(b'\\') => '\\',
                        Some(b'/') => '/',
                        Some(b'b') => '\u{8}',
                        Some(b'f') => '\u{c}',
                        Some(b'n') => '\n',
                        Some(b'r') => '\r',
                        Some(b't') => '\t',
                        Some(b'u') => {
                            self.i += 1;
                            let hi = self.hex4()?;
                            self.i -= 1;
                            let code = if (0xD800..0xDC00).contains(&hi) {
                                self.i += 1;
                                self.eat(b"\\u")?;
                                let lo = self.hex4()?;
                                self.i -= 1;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return self.err();
                                }
                                0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                            } else {
                                hi
                            };
                            char::from_u32(code).ok_or(Error { at: self.i })?
                        }
                        _ => return self.err(),
                    };
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
                    self.i += 1;
                }
                Some(&b) if b < 0x20 => return self.err(),
                Some(&b) => {
                    out.push(b);
                    self.i += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_step_case_reads_as_written() {
        let v = parse(br#"{"name": "a9 n 1", "initial": {"pc": 64793, "ram": [[6487322, 251]]}, "cycles": [[6487321, null, "dp-r-mx-"]]}"#).unwrap();
        assert_eq!(v.get("name").and_then(Value::as_str), Some("a9 n 1"));
        assert_eq!(v.get("initial").and_then(|i| i.get("pc")).and_then(Value::as_i64), Some(64793));
        let cycle = &v.get("cycles").unwrap().as_array().unwrap()[0];
        assert!(cycle.as_array().unwrap()[1].is_null());
        assert_eq!(cycle.as_array().unwrap()[2].as_str(), Some("dp-r-mx-"));
    }

    #[test]
    fn the_rest_of_the_grammar_reads_and_malformed_text_is_refused() {
        assert_eq!(parse(b" [true,false,null,-0,1.5e2,\"\\u00e9\\ud83d\\ude00\\n\"] ").unwrap(),
            Value::Array(vec![Value::Bool(true), Value::Bool(false), Value::Null, Value::Int(0), Value::Float(150.0), Value::Str("\u{e9}\u{1F600}\n".into())]));
        assert_eq!(parse(b"{}").unwrap(), Value::Object(vec![]));
        for bad in [&b"[1,]"[..], b"{\"a\" 1}", b"01", b"[1] x", b"\"\x01\"", b"tru", b"", b"1.", b"\"\\ud800x\""] {
            assert!(parse(bad).is_err(), "{:?}", std::str::from_utf8(bad));
        }
    }
}
