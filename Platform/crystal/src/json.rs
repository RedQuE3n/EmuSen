//! The API's JSON as the C# implementation writes it with a default `Utf8JsonWriter`: compact, strings escaped by
//! .NET's default encoder and doubles in .NET's shortest form, so that an answer of the Rust runtime is the C#'s to the
//! byte (EmuSen_Backend.md §29.2).

use emusen_dotnet::json::{escape_into, format_f64};

/// A writer of one JSON document, its commas placed as values follow one another.
#[derive(Default)]
pub struct Json {
    out: String,
    first: Vec<bool>,
}

impl Json {
    pub fn new() -> Json {
        Json::default()
    }

    fn comma(&mut self) {
        if let Some(first) = self.first.last_mut() {
            if !*first {
                self.out.push(',');
            }
            *first = false;
        }
    }

    fn name(&mut self, key: &str) -> &mut Json {
        self.comma();
        self.out.push('"');
        escape_into(&mut self.out, key);
        self.out.push_str("\":");
        if let Some(first) = self.first.last_mut() {
            *first = true;
        }
        self
    }

    fn open(&mut self, c: char) -> &mut Json {
        self.comma();
        self.out.push(c);
        self.first.push(true);
        self
    }

    pub fn begin_object(&mut self) -> &mut Json {
        self.open('{')
    }

    pub fn end_object(&mut self) -> &mut Json {
        self.first.pop();
        self.out.push('}');
        self
    }

    pub fn begin_array(&mut self) -> &mut Json {
        self.open('[')
    }

    pub fn end_array(&mut self) -> &mut Json {
        self.first.pop();
        self.out.push(']');
        self
    }

    pub fn key_object(&mut self, key: &str) -> &mut Json {
        self.name(key);
        self.begin_object()
    }

    pub fn key_array(&mut self, key: &str) -> &mut Json {
        self.name(key);
        self.begin_array()
    }

    pub fn str_value(&mut self, value: &str) -> &mut Json {
        self.comma();
        self.out.push('"');
        escape_into(&mut self.out, value);
        self.out.push('"');
        self
    }

    pub fn int_value(&mut self, value: i64) -> &mut Json {
        self.comma();
        self.out.push_str(&value.to_string());
        self
    }

    pub fn null_value(&mut self) -> &mut Json {
        self.comma();
        self.out.push_str("null");
        self
    }

    pub fn str(&mut self, key: &str, value: &str) -> &mut Json {
        self.name(key);
        self.str_value(value)
    }

    pub fn str_or_null(&mut self, key: &str, value: Option<&str>) -> &mut Json {
        match value {
            Some(v) => self.str(key, v),
            None => self.null(key),
        }
    }

    pub fn null(&mut self, key: &str) -> &mut Json {
        self.name(key);
        self.null_value()
    }

    pub fn int(&mut self, key: &str, value: i64) -> &mut Json {
        self.name(key);
        self.int_value(value)
    }

    pub fn bool(&mut self, key: &str, value: bool) -> &mut Json {
        self.name(key);
        self.comma();
        self.out.push_str(if value { "true" } else { "false" });
        self
    }

    /// A double as .NET writes it; .NET refuses a value JSON has no number for, which no answer here holds.
    pub fn f64(&mut self, key: &str, value: f64) -> &mut Json {
        self.name(key);
        self.comma();
        self.out.push_str(&format_f64(value).unwrap_or_else(|| "0".to_string()));
        self
    }

    pub fn strs<S: AsRef<str>>(&mut self, key: &str, values: impl IntoIterator<Item = S>) -> &mut Json {
        self.key_array(key);
        for v in values {
            self.str_value(v.as_ref());
        }
        self.end_array()
    }

    pub fn finish(self) -> String {
        self.out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_writes_what_a_default_utf8jsonwriter_writes() {
        let mut j = Json::new();
        j.begin_object().str("a", "<é>\n").int("b", -3).f64("c", 1.365385).f64("d", 2.0).strs("e", ["x", "y"]).null("f").key_array("g");
        j.begin_object().bool("h", true).end_object().begin_object().end_object().end_array().end_object();
        assert_eq!(j.finish(), concat!(r#"{"a":""#, "\\u003C\\u00E9\\u003E\\n", r#"","b":-3,"c":1.365385,"d":2,"e":["x","y"],"f":null,"g":[{"h":true},{}]}"#));
    }
}
