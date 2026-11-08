//! The API's JSON answers, built member by member as a `serde_json::Value` and written by serde_json: the builder keeps
//! the order members are given in and writes nothing itself. See EmuSen_RustPlatform.md §47.12.

use serde_json::{Map, Value};

/// One answer under construction: the containers still open, each with the name it will take in its parent.
#[derive(Default)]
pub struct Json {
    open: Vec<(Option<String>, Value)>,
    name: Option<String>,
    done: Option<Value>,
}

impl Json {
    pub fn new() -> Json {
        Json::default()
    }

    /// A finished value placed in the container open now, under the name given last, or as the answer itself.
    fn put(&mut self, value: Value) -> &mut Json {
        let name = self.name.take();
        match self.open.last_mut() {
            Some((_, Value::Object(members))) => {
                members.insert(name.unwrap_or_default(), value);
            }
            Some((_, Value::Array(items))) => items.push(value),
            _ => self.done = Some(value),
        }
        self
    }

    fn begin(&mut self, container: Value) -> &mut Json {
        let name = self.name.take();
        self.open.push((name, container));
        self
    }

    fn end(&mut self) -> &mut Json {
        if let Some((name, container)) = self.open.pop() {
            self.name = name;
            self.put(container);
        }
        self
    }

    pub fn begin_object(&mut self) -> &mut Json {
        self.begin(Value::Object(Map::new()))
    }

    pub fn end_object(&mut self) -> &mut Json {
        self.end()
    }

    pub fn begin_array(&mut self) -> &mut Json {
        self.begin(Value::Array(Vec::new()))
    }

    pub fn end_array(&mut self) -> &mut Json {
        self.end()
    }

    /// A member's name, its value given next.
    pub fn key(&mut self, key: &str) -> &mut Json {
        self.name = Some(key.to_string());
        self
    }

    pub fn key_object(&mut self, key: &str) -> &mut Json {
        self.key(key).begin_object()
    }

    pub fn key_array(&mut self, key: &str) -> &mut Json {
        self.key(key).begin_array()
    }

    pub fn str_value(&mut self, value: &str) -> &mut Json {
        self.put(Value::from(value))
    }

    pub fn int_value(&mut self, value: i64) -> &mut Json {
        self.put(Value::from(value))
    }

    pub fn u64_value(&mut self, value: u64) -> &mut Json {
        self.put(Value::from(value))
    }

    /// A double; one JSON has no number for is written as `null`.
    pub fn f64_value(&mut self, value: f64) -> &mut Json {
        self.put(Value::from(value))
    }

    pub fn null_value(&mut self) -> &mut Json {
        self.put(Value::Null)
    }

    /// A parsed document of the old reader's, written as it holds it.
    pub fn key_value(&mut self, key: &str, v: &Value) -> &mut Json {
        self.key(key).put(v.clone())
    }

    pub fn u64(&mut self, key: &str, value: u64) -> &mut Json {
        self.key(key).u64_value(value)
    }

    pub fn str(&mut self, key: &str, value: &str) -> &mut Json {
        self.key(key).str_value(value)
    }

    pub fn str_or_null(&mut self, key: &str, value: Option<&str>) -> &mut Json {
        self.key(key).put(value.map_or(Value::Null, Value::from))
    }

    pub fn null(&mut self, key: &str) -> &mut Json {
        self.key(key).null_value()
    }

    pub fn int(&mut self, key: &str, value: i64) -> &mut Json {
        self.key(key).int_value(value)
    }

    pub fn bool(&mut self, key: &str, value: bool) -> &mut Json {
        self.key(key).put(Value::Bool(value))
    }

    pub fn f64(&mut self, key: &str, value: f64) -> &mut Json {
        self.key(key).f64_value(value)
    }

    pub fn strs<S: AsRef<str>>(&mut self, key: &str, values: impl IntoIterator<Item = S>) -> &mut Json {
        self.key(key).put(Value::Array(values.into_iter().map(|v| Value::from(v.as_ref())).collect()))
    }

    /// The answer as compact JSON.
    pub fn finish(self) -> String {
        self.done.map_or_else(String::new, |v| v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_keep_their_order_and_serde_writes_the_text() {
        let mut j = Json::new();
        j.begin_object().str("a", "<é>\n").int("b", -3).f64("c", 1.365385).f64("d", 2.0).strs("e", ["x", "y"]).null("f").key_array("g");
        j.begin_object().bool("h", true).end_object().begin_object().end_object().end_array().f64("i", f64::NAN).end_object();
        assert_eq!(j.finish(), r#"{"a":"<é>\n","b":-3,"c":1.365385,"d":2.0,"e":["x","y"],"f":null,"g":[{"h":true},{}],"i":null}"#);
        let mut list = Json::new();
        list.begin_array().int_value(1).str_value("two").end_array();
        assert_eq!(list.finish(), r#"[1,"two"]"#);
    }
}
