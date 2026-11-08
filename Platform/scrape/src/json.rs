//! Requests read and answers' values written for `emusen_scrape.h`'s calls, both by serde_json. See EmuSen_Backend.md
//! §30.3 and EmuSen_RustPlatform.md §47.13.

use crate::Error;
use serde_json::Value;

pub fn parse(text: &str) -> Result<Value, Error> {
    let value: Value = serde_json::from_str(text).map_err(|e| Error::BadRequest(format!("The request is not JSON: {e}")))?;
    if !value.is_object() {
        return Err(Error::BadRequest("The request is not a JSON object.".into()));
    }
    Ok(value)
}

fn missing(name: &str) -> Error {
    Error::BadRequest(format!("The request has no {name}."))
}

fn wrong(name: &str, what: &str) -> Error {
    Error::BadRequest(format!("The request's {name} is not {what}."))
}

pub fn text(r: &Value, name: &str) -> Result<String, Error> {
    match r.get(name) {
        Some(Value::String(t)) => Ok(t.clone()),
        Some(_) => Err(wrong(name, "text")),
        None => Err(missing(name)),
    }
}

pub fn text_or_none(r: &Value, name: &str) -> Result<Option<String>, Error> {
    match r.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => text(r, name).map(Some),
    }
}

pub fn flag(r: &Value, name: &str, default: bool) -> Result<bool, Error> {
    match r.get(name) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Bool(b)) => Ok(*b),
        Some(_) => Err(wrong(name, "true or false")),
    }
}

pub fn integer(r: &Value, name: &str) -> Result<i64, Error> {
    match r.get(name) {
        Some(v) => v.as_i64().ok_or_else(|| wrong(name, "a whole number")),
        None => Err(missing(name)),
    }
}

pub fn integer_or_none(r: &Value, name: &str) -> Result<Option<i64>, Error> {
    match r.get(name) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => integer(r, name).map(Some),
    }
}

pub fn strings(r: &Value, name: &str) -> Result<Vec<String>, Error> {
    match r.get(name) {
        Some(Value::Array(items)) => items
            .iter()
            .map(|v| v.as_str().map(str::to_string).ok_or_else(|| wrong(name, "a list of texts")))
            .collect(),
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(_) => Err(wrong(name, "a list of texts")),
    }
}

/// A text as a JSON string.
pub fn quoted(text: &str) -> String {
    Value::from(text).to_string()
}

pub fn text_or_null(out: &mut String, text: Option<&str>) {
    out.push_str(&Value::from(text).to_string());
}

pub fn number_or_null(out: &mut String, n: Option<i64>) {
    out.push_str(&Value::from(n).to_string());
}

/// A float as the shortest text that reads back as the same float, or null.
pub fn float_or_null(out: &mut String, n: Option<f32>) {
    out.push_str(&n.filter(|f| f.is_finite()).map_or(Value::Null, Value::from).to_string());
}

pub fn strings_into(out: &mut String, items: impl IntoIterator<Item = impl AsRef<str>>) {
    out.push_str(&Value::Array(items.into_iter().map(|s| Value::from(s.as_ref())).collect()).to_string());
}
