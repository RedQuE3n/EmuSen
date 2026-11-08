//! The headers' common idioms, written once: text in as a pointer and a length, the length-query copy-out, a status
//! read with its words, and a JSON answer read into its record.

use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

/// Text as the API takes it; none is the absent string.
pub(crate) fn text(s: Option<&str>) -> (*const u8, usize) {
    s.map_or((std::ptr::null(), 0), |s| (s.as_ptr(), s.len()))
}

/// A negative return as a failure with `words`, read at once on this thread.
pub(crate) fn check(n: i64, words: impl FnOnce() -> String) -> Result<i64> {
    if n < 0 { Err(Error::Status { status: n as i32, words: words() }) } else { Ok(n) }
}

/// The length-query idiom: asks the length, then copies, and asks again where the answer grew between the two calls.
pub(crate) fn copy_out(mut call: impl FnMut(*mut u8, usize) -> i64, words: impl Fn() -> String) -> Result<Vec<u8>> {
    let mut n = check(call(std::ptr::null_mut(), 0), &words)?;
    loop {
        let mut buf = vec![0u8; n as usize];
        let m = check(call(buf.as_mut_ptr(), buf.len()), &words)?;
        if m as usize <= buf.len() {
            buf.truncate(m as usize);
            return Ok(buf);
        }
        n = m;
    }
}

/// As `copy_out`, with EMUSEN_SVC_ABSENT as none.
pub(crate) fn copy_out_absent(call: impl FnMut(*mut u8, usize) -> i64, words: impl Fn() -> String) -> Result<Option<Vec<u8>>> {
    match copy_out(call, words) {
        Err(e) if e.is_absent() => Ok(None),
        other => other.map(Some),
    }
}

/// An answer as text.
pub(crate) fn string(call: &'static str, bytes: Vec<u8>) -> Result<String> {
    String::from_utf8(bytes).map_err(|e| Error::Shape { call, words: e.to_string() })
}

/// An answer read into its record.
pub(crate) fn json<T: DeserializeOwned>(call: &'static str, bytes: &[u8]) -> Result<T> {
    serde_json::from_slice(bytes).map_err(|e| Error::Shape { call, words: e.to_string() })
}

/// A record as the JSON text a call takes.
pub(crate) fn to_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("a record serialises")
}
