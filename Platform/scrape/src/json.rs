//! Requests read and answers written for `emusen_scrape.h`'s calls. See EmuSen_Backend.md §30.3.

use crate::Error;
use emusen_dotnet::json::{self as dj, Value};

pub fn parse(text: &str) -> Result<Value, Error> {
    let value = dj::parse(text).map_err(|e| Error::BadRequest(format!("The request is not JSON: {e}")))?;
    if !matches!(value, Value::Object(_)) {
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
        Some(Value::String(t)) if !t.broken => Ok(t.text.clone()),
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
        Some(Value::Number(token)) => token.parse::<i64>().map_err(|_| wrong(name, "a whole number")),
        Some(_) => Err(wrong(name, "a whole number")),
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
            .map(|v| match v {
                Value::String(t) if !t.broken => Ok(t.text.clone()),
                _ => Err(wrong(name, "a list of texts")),
            })
            .collect(),
        None | Some(Value::Null) => Ok(Vec::new()),
        Some(_) => Err(wrong(name, "a list of texts")),
    }
}

pub fn quoted(text: &str) -> String {
    dj::quoted(text)
}

pub fn text_or_null(out: &mut String, text: Option<&str>) {
    match text {
        Some(t) => out.push_str(&dj::quoted(t)),
        None => out.push_str("null"),
    }
}

pub fn number_or_null(out: &mut String, n: Option<i64>) {
    match n {
        Some(n) => out.push_str(&n.to_string()),
        None => out.push_str("null"),
    }
}

/// A float as the shortest text that reads back as the same float, or null.
pub fn float_or_null(out: &mut String, n: Option<f32>) {
    match n {
        Some(f) if f.is_finite() => out.push_str(&format!("{f:?}")),
        _ => out.push_str("null"),
    }
}

pub fn strings_into(out: &mut String, items: impl IntoIterator<Item = impl AsRef<str>>) {
    out.push('[');
    for (i, s) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&dj::quoted(s.as_ref()));
    }
    out.push(']');
}
