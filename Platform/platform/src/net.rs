//! The C layer over emusen-net, the platform's HTTP client, for the C# `CheatDatabaseInstaller.Fetch` behind
//! `EMUSEN_SERVICES_NATIVE` (EmuSen_Backend.md §16). A download is a handle running on a thread of the library's own;
//! the host drains its progress and waits for its end, and nothing calls the host back.

use std::time::Duration;

use crate::{fail, given, put, status};
use emusen_dotnet::json::{self, Value};
use emusen_net::{Download, Failure, Request};

fn request(data: *const u8, len: usize) -> Result<Request, i32> {
    let r = json::parse(unsafe { given(data, len) }?).map_err(|e| fail(status::PARSE, e.to_string()))?;
    let url = r.get("url").and_then(Value::as_str).ok_or_else(|| fail(status::PARSE, "url is not text"))?.to_string();
    let timeout = match r.get("timeout_ms") {
        None | Some(Value::Null) => Duration::MAX,
        Some(Value::Number(n)) => Duration::from_millis(n.parse::<u64>().map_err(|_| fail(status::PARSE, "timeout_ms is not a whole number of milliseconds"))?),
        _ => return Err(fail(status::PARSE, "timeout_ms is not a number")),
    };
    let test_roots = match r.get("test_roots") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::String(pem)) => emusen_net::pem_certificates(&pem.text).ok_or_else(|| fail(status::PARSE, "test_roots holds a certificate that does not read"))?,
        _ => return Err(fail(status::PARSE, "test_roots is not text")),
    };
    Ok(Request { url, timeout, test_roots })
}

/// A download started: `{"url", "timeout_ms", "test_roots"}` in, a handle out, freed with `emusen_net_download_free`;
/// null for a request that is not one, with the words in `emusen_platform_last_error`.
///
/// # Safety
/// `request` must be valid for `request_len` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_start(request_data: *const u8, request_len: usize) -> *mut Download {
    match request(request_data, request_len) {
        Ok(r) => Box::into_raw(Box::new(Download::start(r))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// # Safety
/// `d` must be a handle `emusen_net_download_start` gave and not yet freed, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_free(d: *mut Download) {
    if !d.is_null() {
        drop(unsafe { Box::from_raw(d) });
    }
}

fn handle<'a>(d: *const Download) -> Result<&'a Download, i32> {
    if d.is_null() { Err(fail(status::NULL, "no download")) } else { Ok(unsafe { &*d }) }
}

/// Where the download stands (0 running, 1 done, 2 failed, 3 cancelled), the body's bytes so far and its length, -1 until known.
///
/// # Safety
/// `d` must be a live handle; `received` and `total` valid for one value each, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_progress(d: *const Download, received: *mut u64, total: *mut i64) -> i32 {
    let d = match handle(d) {
        Ok(d) => d,
        Err(s) => return s,
    };
    let (got, length, state) = d.progress();
    if !received.is_null() {
        unsafe { *received = got };
    }
    if !total.is_null() {
        unsafe { *total = length.map_or(-1, |t| t as i64) };
    }
    state as i32
}

/// Waits up to `timeout_ms` for the download's end, and says where it stands.
///
/// # Safety
/// `d` must be a live handle.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_wait(d: *const Download, timeout_ms: u32) -> i32 {
    match handle(d) {
        Ok(d) => d.wait(Duration::from_millis(timeout_ms as u64)) as i32,
        Err(s) => s,
    }
}

/// Ends the download for the host now: it stands cancelled, and its fetch stops at its next read.
///
/// # Safety
/// `d` must be a live handle, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_cancel(d: *const Download) {
    if let Ok(d) = handle(d) {
        d.cancel();
    }
}

/// A finished download's body, by the length-query idiom; ABSENT while it runs or when it failed.
///
/// # Safety
/// `d` must be a live handle; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_body(d: *const Download, out: *mut u8, len: usize) -> i64 {
    let d = match handle(d) {
        Ok(d) => d,
        Err(s) => return s as i64,
    };
    match d.with_outcome(|o| o.as_ref().ok().map(|body| unsafe { put(body, out, len) })) {
        Some(Some(n)) => n,
        _ => fail(status::ABSENT, "the download has no body") as i64,
    }
}

fn failure_json(f: &Failure) -> String {
    let words = |t: &Option<String>| t.as_deref().map_or(Value::Null, Value::string);
    let value = Value::Object(
        [
            ("kind", Value::string(f.kind.name())),
            ("message", Value::string(f.message.as_str())),
            ("inner", words(&f.inner)),
            ("status", f.status.map_or(Value::Null, Value::number)),
        ]
        .into_iter()
        .map(|(k, v)| (json::Text::new(k), v))
        .collect(),
    );
    let mut out = String::new();
    json::write_compact(&mut out, &value);
    out
}

/// Why a download that ended gave no body: `{"kind", "message", "inner", "status"}`, by the length-query idiom; ABSENT
/// while it runs or when it gave one.
///
/// # Safety
/// `d` must be a live handle; `out` valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_net_download_failure(d: *const Download, out: *mut u8, len: usize) -> i64 {
    let d = match handle(d) {
        Ok(d) => d,
        Err(s) => return s as i64,
    };
    match d.with_outcome(|o| o.as_ref().err().map(failure_json)) {
        Some(Some(text)) => unsafe { put(text.as_bytes(), out, len) },
        _ => fail(status::ABSENT, "the download has no failure") as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use emusen_net::State;

    #[test]
    fn a_request_that_is_not_one_gives_no_handle() {
        for text in ["{}", r#"{"url": 3}"#, r#"{"url": "https://x/", "timeout_ms": -1}"#, r#"{"url": "https://x/", "test_roots": "-----BEGIN CERTIFICATE-----\n!!\n-----END CERTIFICATE-----\n"}"#] {
            assert!(unsafe { emusen_net_download_start(text.as_ptr(), text.len()) }.is_null(), "{text}");
        }
    }

    #[test]
    fn a_refused_url_fails_with_its_words_and_has_no_body() {
        let text = r#"{"url": "ftp://example.org/", "timeout_ms": 1000}"#;
        let d = unsafe { emusen_net_download_start(text.as_ptr(), text.len()) };
        assert_eq!(unsafe { emusen_net_download_wait(d, 5000) }, State::Failed as i32);
        let (mut received, mut total) = (7u64, 7i64);
        assert_eq!(unsafe { emusen_net_download_progress(d, &mut received, &mut total) }, 2);
        assert_eq!((received, total), (0, -1));
        assert_eq!(unsafe { emusen_net_download_body(d, std::ptr::null_mut(), 0) }, status::ABSENT as i64);
        let n = unsafe { emusen_net_download_failure(d, std::ptr::null_mut(), 0) } as usize;
        let mut out = vec![0u8; n];
        assert_eq!(unsafe { emusen_net_download_failure(d, out.as_mut_ptr(), n) } as usize, n);
        assert_eq!(String::from_utf8(out).unwrap(), r#"{"kind":"scheme","message":"The 'ftp' scheme is not supported.","inner":null,"status":null}"#);
        unsafe { emusen_net_download_free(d) };
    }
}
