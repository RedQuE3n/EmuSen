//! A JSON writer for the descriptors: compact, members in call order, strings escaped per RFC 8259.

/// One document under construction; `begin`/`end` nest, and commas are placed for the caller.
#[derive(Default)]
pub struct Json {
    out: String,
    /// Per open container: whether it already holds a value.
    filled: Vec<bool>,
}

impl Json {
    pub fn new() -> Json {
        Json::default()
    }

    pub fn finish(self) -> String {
        debug_assert!(self.filled.is_empty(), "an unclosed container");
        self.out
    }

    fn sep(&mut self) {
        if let Some(f) = self.filled.last_mut() {
            if *f {
                self.out.push(',');
            }
            *f = true;
        }
    }

    pub fn key(&mut self, k: &str) -> &mut Json {
        self.sep();
        self.string(k);
        self.out.push(':');
        // The value that follows must not add a comma of its own.
        if let Some(f) = self.filled.last_mut() {
            *f = false;
        }
        self
    }

    fn after_value(&mut self) {
        if let Some(f) = self.filled.last_mut() {
            *f = true;
        }
    }

    pub fn begin_object(&mut self) -> &mut Json {
        self.sep();
        self.out.push('{');
        self.filled.push(false);
        self
    }

    pub fn end_object(&mut self) -> &mut Json {
        self.filled.pop();
        self.out.push('}');
        self.after_value();
        self
    }

    pub fn begin_array(&mut self) -> &mut Json {
        self.sep();
        self.out.push('[');
        self.filled.push(false);
        self
    }

    pub fn end_array(&mut self) -> &mut Json {
        self.filled.pop();
        self.out.push(']');
        self.after_value();
        self
    }

    pub fn str(&mut self, v: &str) -> &mut Json {
        self.sep();
        self.string(v);
        self
    }

    pub fn int(&mut self, v: i64) -> &mut Json {
        self.sep();
        self.out.push_str(&v.to_string());
        self
    }

    pub fn uint(&mut self, v: u64) -> &mut Json {
        self.sep();
        self.out.push_str(&v.to_string());
        self
    }

    pub fn bool(&mut self, v: bool) -> &mut Json {
        self.sep();
        self.out.push_str(if v { "true" } else { "false" });
        self
    }

    pub fn null(&mut self) -> &mut Json {
        self.sep();
        self.out.push_str("null");
        self
    }

    pub fn opt_str(&mut self, v: Option<&str>) -> &mut Json {
        match v {
            Some(s) => self.str(s),
            None => self.null(),
        }
    }

    /// A member `"k": "v"`.
    pub fn field_str(&mut self, k: &str, v: &str) -> &mut Json {
        self.key(k).str(v)
    }

    pub fn field_int(&mut self, k: &str, v: i64) -> &mut Json {
        self.key(k).int(v)
    }

    pub fn field_uint(&mut self, k: &str, v: u64) -> &mut Json {
        self.key(k).uint(v)
    }

    pub fn field_bool(&mut self, k: &str, v: bool) -> &mut Json {
        self.key(k).bool(v)
    }

    pub fn field_strs<S: AsRef<str>>(&mut self, k: &str, items: &[S]) -> &mut Json {
        self.key(k).begin_array();
        for s in items {
            self.str(s.as_ref());
        }
        self.end_array()
    }

    fn string(&mut self, s: &str) {
        self.out.push('"');
        for c in s.chars() {
            match c {
                '"' => self.out.push_str("\\\""),
                '\\' => self.out.push_str("\\\\"),
                '\n' => self.out.push_str("\\n"),
                '\r' => self.out.push_str("\\r"),
                '\t' => self.out.push_str("\\t"),
                c if (c as u32) < 0x20 => self.out.push_str(&format!("\\u{:04x}", c as u32)),
                c => self.out.push(c),
            }
        }
        self.out.push('"');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commas_nest_and_strings_escape() {
        let mut j = Json::new();
        j.begin_object().field_str("a", "x\"\\\n\u{1}é").key("b").begin_array().int(-1).uint(2).bool(true).null().begin_object().end_object().end_array().field_strs("c", &["p", "q"]).end_object();
        let text = j.finish();
        assert_eq!(text, r#"{"a":"x\"\\\n\u0001é","b":[-1,2,true,null,{}],"c":["p","q"]}"#);
        assert!(crate::json::parse(text.as_bytes()).is_ok());
    }
}
