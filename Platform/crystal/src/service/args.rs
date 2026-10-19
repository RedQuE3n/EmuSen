//! What a call is handed: text and bytes as the header passes them, read only when the C# provider reads them, and the
//! JSON arguments read as `SessionJson` reads them, `JsonDocument.ParseValue` and each accessor's words.

use emusen_dotnet::json::Value;
use emusen_session_service::abi::status;

use super::{Answer, refused};

/// Text or bytes as passed in, a pointer and a length, read when the call reads it.
#[derive(Clone, Copy)]
pub struct Raw<'a> {
    data: *const u8,
    len: usize,
    _life: std::marker::PhantomData<&'a [u8]>,
}

impl<'a> Raw<'a> {
    /// # Safety
    /// `data` null or valid for `len` bytes for `'a`.
    pub unsafe fn new(data: *const u8, len: usize) -> Raw<'a> {
        Raw { data, len, _life: std::marker::PhantomData }
    }

    pub fn none() -> Raw<'a> {
        Raw { data: std::ptr::null(), len: 0, _life: std::marker::PhantomData }
    }

    pub fn from_slice(bytes: &'a [u8]) -> Raw<'a> {
        Raw { data: bytes.as_ptr(), len: bytes.len(), _life: std::marker::PhantomData }
    }

    /// The text, the absent string for a null pointer with no length, `SessionExports.Text`.
    pub fn text(&self) -> Answer<Option<&'a str>> {
        if self.data.is_null() {
            return if self.len == 0 { Ok(None) } else { Err(refused(status::NULL, "a null string with a length")) };
        }
        let bytes = if self.len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(self.data, self.len) } };
        std::str::from_utf8(bytes).map(Some).map_err(|_| refused(status::BAD_STRING, "text that is not UTF-8"))
    }

    /// The bytes, none for a null pointer with no length, `SessionExports.Bytes`.
    pub fn bytes(&self) -> Answer<&'a [u8]> {
        if self.data.is_null() {
            return if self.len == 0 { Ok(&[]) } else { Err(refused(status::NULL, "a null buffer with a length")) };
        }
        Ok(if self.len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(self.data, self.len) } })
    }
}

/// `SessionJson.Parse`: the argument's one JSON value, `{}` for none.
pub fn parse(utf8: &[u8]) -> Answer<Value> {
    if utf8.is_empty() {
        return Ok(Value::Object(Vec::new()));
    }
    let text = match std::str::from_utf8(utf8) {
        Ok(t) => t,
        Err(e) => return Err(refused(status::BAD_ARGUMENT, format!("the argument is not JSON: '0x{:02X}' is an invalid start of a value. LineNumber: 0 | BytePositionInLine: {}.", utf8[e.valid_up_to()], e.valid_up_to()))),
    };
    emusen_dotnet::json_document::parse_value(text).map_err(|why| refused(status::BAD_ARGUMENT, format!("the argument is not JSON: {why}")))
}

/// A member that is there and not null.
fn member<'v>(args: &'v Value, key: &str) -> Option<&'v Value> {
    match args {
        Value::Object(_) => args.get(key).filter(|v| !v.is_null()),
        _ => None,
    }
}

/// `TryGetInt64`: a number written as a whole number that fits.
fn int64(token: &str) -> Option<i64> {
    let digits = token.strip_prefix('-').unwrap_or(token);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    token.parse().ok()
}

pub fn optional_int(args: &Value, key: &str) -> Answer<Option<i64>> {
    let Some(v) = member(args, key) else { return Ok(None) };
    match v {
        Value::Number(token) => int64(token).map(Some).ok_or_else(|| refused(status::BAD_ARGUMENT, format!("\"{key}\" is not an integer"))),
        _ => Err(refused(status::BAD_ARGUMENT, format!("\"{key}\" is not an integer"))),
    }
}

pub fn int(args: &Value, key: &str) -> Answer<i64> {
    optional_int(args, key)?.ok_or_else(|| refused(status::BAD_ARGUMENT, format!("the argument has no \"{key}\"")))
}

/// `GetString`'s refusal of a string holding a lone surrogate, which the provider's catch answers as a fault.
fn string_of(text: &emusen_dotnet::json::Text) -> Answer<String> {
    match &text.refusal {
        Some(why) if text.broken => Err(refused(status::FAULT, why.clone())),
        _ => Ok(text.text.clone()),
    }
}

pub fn optional_string(args: &Value, key: &str) -> Answer<Option<String>> {
    let Some(v) = member(args, key) else { return Ok(None) };
    match v {
        Value::String(t) => string_of(t).map(Some),
        _ => Err(refused(status::BAD_ARGUMENT, format!("\"{key}\" is not a string"))),
    }
}

pub fn string(args: &Value, key: &str) -> Answer<String> {
    optional_string(args, key)?.ok_or_else(|| refused(status::BAD_ARGUMENT, format!("the argument has no \"{key}\"")))
}

pub fn optional_bool(args: &Value, key: &str) -> Answer<Option<bool>> {
    let Some(v) = member(args, key) else { return Ok(None) };
    match v {
        Value::Bool(b) => Ok(Some(*b)),
        _ => Err(refused(status::BAD_ARGUMENT, format!("\"{key}\" is not true or false"))),
    }
}

/// `AddressOf`: a 32-bit pattern, written signed or unsigned.
fn address_of(value: i64, key: &str) -> Answer<i32> {
    if (i32::MIN as i64..=u32::MAX as i64).contains(&value) { Ok(value as u32 as i32) } else { Err(refused(status::BAD_ARGUMENT, format!("\"{key}\" {value} is not a 32-bit address"))) }
}

pub fn address(args: &Value, key: &str) -> Answer<i32> {
    address_of(int(args, key)?, key)
}

pub fn optional_address(args: &Value, key: &str) -> Answer<Option<i32>> {
    optional_int(args, key)?.map(|v| address_of(v, key)).transpose()
}

/// `JsonValueKind`'s name for a value, as `GetString`'s refusal names it.
pub fn kind_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "Null",
        Value::Bool(true) => "True",
        Value::Bool(false) => "False",
        Value::Number(_) => "Number",
        Value::String(_) => "String",
        Value::Array(_) => "Array",
        Value::Object(_) => "Object",
    }
}

/// `JsonElement.GetString` on an array's element: its text, none for null, and a fault for another kind.
pub fn element_string(v: &Value) -> Answer<Option<String>> {
    match v {
        Value::Null => Ok(None),
        Value::String(t) => string_of(t).map(Some),
        other => Err(refused(status::FAULT, format!("The requested operation requires an element of type 'String', but the target element has type '{}'.", kind_name(other)))),
    }
}
