//! The C layer over emusen-firmware, for the C# `FirmwareLibrary` and `FirmwareOverview`, their only
//! implementation since 2026-11-04 (EmuSen_Backend.md §15, §40.1); an install is the frontend API's `firmware_install`
//! since 2.0. Each call takes one JSON request; nothing is decided here.

use crate::serenity::guard;
use crate::{fail, given, put, report, status};
use serde_json::Value;
use emusen_firmware as firmware;

fn request(data: *const u8, len: usize) -> Result<Value, i32> {
    let text = unsafe { given(data, len) }?;
    serde_json::from_str::<Value>(text).map_err(|e| fail(status::PARSE, e.to_string()))
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
    guard(|s| s as i64, || {
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
    })
}

/// An image found: `{"request", "folders"}` in, `found_envelope` out; ABSENT when no folder holds it.
///
/// # Safety
/// As `emusen_firmware_folders`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_firmware_find(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    guard(|s| s as i64, || {
        thread_local! {
            static WAITING: crate::reply::Waiting = const { std::cell::RefCell::new(None) };
        }
        let req_text = match unsafe { given(request_data, request_len) } {
            Ok(t) => t,
            Err(s) => return s as i64,
        };
        let kept = WAITING.with_borrow_mut(|w| match w {
            Some((0, r, _)) if r.as_slice() == req_text.as_bytes() => w.take().map(|(_, _, a)| a),
            _ => None,
        });
        let bytes = match kept {
            Some(bytes) => bytes,
            None => {
                let make = || -> Result<Vec<u8>, i32> {
                    let r = serde_json::from_str::<Value>(req_text).map_err(|e| fail(status::PARSE, e.to_string()))?;
                    let wanted = parsed(firmware::request_from(r.get("request").unwrap_or(&Value::Null)))?;
                    let folders = parsed(firmware::folders_from(r.get("folders")))?;
                    firmware::find(&wanted, &folders).map(|f| firmware::found_envelope(&f)).ok_or_else(|| fail(status::ABSENT, format!("no folder holds {}", wanted.file)))
                };
                match make() {
                    Ok(bytes) => bytes,
                    Err(s) => return s as i64,
                }
            }
        };
        if out.is_null() || bytes.len() > len {
            let length = bytes.len();
            WAITING.with_borrow_mut(|w| *w = Some((0, req_text.as_bytes().to_vec(), bytes)));
            return length as i64;
        }
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
        bytes.len() as i64
    })
}

/// Where an image is, unread, and where a wrong-sized file of its name is: `{"request", "folders"}` in, `located_json` out.
///
/// # Safety
/// As `emusen_firmware_folders`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_firmware_locate(request_data: *const u8, request_len: usize, out: *mut u8, len: usize) -> i64 {
    guard(|s| s as i64, || {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_find_caches_between_length_query_and_fill() {
        let dir = std::env::temp_dir().join(format!("emusen-platform-fw-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("bios.bin");
        std::fs::write(&file, b"BIOSDATA").unwrap();
        let req = serde_json::json!({
            "request": { "file": "bios.bin", "size": 8 },
            "folders": [dir.to_string_lossy()]
        })
        .to_string();
        let n = unsafe { emusen_firmware_find(req.as_ptr(), req.len(), std::ptr::null_mut(), 0) };
        assert!(n > 0, "{n}");
        std::fs::remove_file(&file).unwrap();
        let mut buf = vec![0u8; n as usize];
        let m = unsafe { emusen_firmware_find(req.as_ptr(), req.len(), buf.as_mut_ptr(), buf.len()) };
        assert_eq!(m, n);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
