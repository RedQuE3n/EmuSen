//! The C layer over emusen-firmware, for the C# `FirmwareLibrary` and `FirmwareOverview` behind
//! `EMUSEN_SERVICES_NATIVE` (EmuSen_Backend.md §15). Each call takes one JSON request; nothing is decided here.

use crate::{fail, given, put, report, status};
use emusen_dotnet::json::{self, Value};
use emusen_firmware as firmware;

fn request(data: *const u8, len: usize) -> Result<Value, i32> {
    let text = unsafe { given(data, len) }?;
    json::parse(text).map_err(|e| fail(status::PARSE, e.to_string()))
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a str, i32> {
    value.get(name).and_then(Value::as_str).ok_or_else(|| fail(status::PARSE, format!("the request has no {name}")))
}

fn parsed<T>(result: Result<T, String>) -> Result<T, i32> {
    result.map_err(|words| fail(status::PARSE, words))
}

/// The folders searched and the setting's value: `{"setting", "home", "store"}` in, `folders_json` out. A refused or
/// absent value is queued once for the host's diagnostics.
///
/// # Safety
/// `request` must be valid for `request_len` bytes; `out` for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_firmware_folders(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<String, i32> {
        let r = request(request_data, request_len)?;
        let (list, value) = firmware::folders(r.get("setting").and_then(Value::as_str), field(&r, "home")?, field(&r, "store")?);
        if let Some(line) = firmware::report_once(&value) {
            report(line);
        }
        Ok(firmware::folders_json(&list, &value))
    };
    match answer() {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(s) => s as i64,
    }
}

/// An image found: `{"request", "folders"}` in, `found_envelope` out; ABSENT when no folder holds it.
///
/// # Safety
/// As `emusen_firmware_folders`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_firmware_find(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<Vec<u8>, i32> {
        let r = request(request_data, request_len)?;
        let wanted = parsed(firmware::request_from(r.get("request").unwrap_or(&Value::Null)))?;
        let folders = parsed(firmware::folders_from(r.get("folders")))?;
        firmware::find(&wanted, &folders).map(|f| firmware::found_envelope(&f)).ok_or_else(|| fail(status::ABSENT, format!("no folder holds {}", wanted.file)))
    };
    match answer() {
        Ok(bytes) => unsafe { put(&bytes, out, len) },
        Err(s) => s as i64,
    }
}

/// Where an image is, unread, and where a wrong-sized file of its name is: `{"request", "folders"}` in, `located_json` out.
///
/// # Safety
/// As `emusen_firmware_folders`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_firmware_locate(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    let answer = || -> Result<String, i32> {
        let r = request(request_data, request_len)?;
        let wanted = parsed(firmware::request_from(r.get("request").unwrap_or(&Value::Null)))?;
        let folders = parsed(firmware::folders_from(r.get("folders")))?;
        let located = firmware::locate(&wanted, &folders);
        let wrong = if located.is_none() { firmware::wrong_size_in(&wanted, &folders) } else { None };
        Ok(firmware::located_json(located.as_ref(), wrong.as_deref()))
    };
    match answer() {
        Ok(text) => unsafe { put(text.as_bytes(), out, len) },
        Err(s) => s as i64,
    }
}

/// An install into the store: `{"request", "source", "store"}` in; 1 installed, 0 refused for its size, IO with the
/// words when the store or the file could not be written.
///
/// # Safety
/// `request` must be valid for `request_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_firmware_install(request_data: *const u8, request_len: usize) -> i32 {
    let answer = || -> Result<bool, i32> {
        let r = request(request_data, request_len)?;
        let wanted = parsed(firmware::request_from(r.get("request").unwrap_or(&Value::Null)))?;
        firmware::install(&wanted, field(&r, "source")?, field(&r, "store")?).map_err(|words| fail(status::IO, words))
    };
    match answer() {
        Ok(done) => i32::from(done),
        Err(s) => s,
    }
}
