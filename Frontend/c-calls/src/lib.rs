//! How a client calls an EmuSen C header (emusen_crystal.h, emusen_session.h, emusen_platform.h, emusen_library.h and
//! the service tables), written once for both client crates (EmuSen_Qt.md §1.1.1): the library opened by its path with
//! its version read before anything else, text passed as a pointer and a length, the length-query copy-out in which a
//! call returns the whole length and writes only when the buffer holds all of it, a negative return as a status with the
//! words its last_error gave on the same thread, and a JSON answer read into its record.

use serde::de::DeserializeOwned;
use std::path::Path;

/// What a call through a header can fail with.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("The library at {path} could not be loaded: {words}")]
    Load { path: String, words: String },
    #[error("{words}")]
    Refused { path: String, words: String },
    #[error("{words} ({name})")]
    Status { status: i32, name: &'static str, words: String },
    #[error("{call} answered JSON this program cannot read: {words}")]
    Shape { call: &'static str, words: String },
}

impl Error {
    /// The status, for a failure the library returned.
    pub fn status(&self) -> Option<i32> {
        match self {
            Error::Status { status, .. } => Some(*status),
            _ => None,
        }
    }
}

/// A header's status names, as its client spells them.
pub type Names = fn(i32) -> &'static str;

/// The library at `path` opened, and the version its `symbol` (a NUL-ended name) answers, read before any other symbol.
pub fn open(path: &Path, symbol: &[u8]) -> Result<(libloading::Library, u32), Error> {
    let load = |e: libloading::Error| Error::Load { path: path.display().to_string(), words: e.to_string() };
    let lib = unsafe { libloading::Library::new(path) }.map_err(load)?;
    let version = unsafe { lib.get::<unsafe extern "C" fn() -> u32>(symbol) }.map_err(load)?;
    let version = unsafe { version() };
    Ok((lib, version))
}

/// A negative return as a failure with `words`, read at once on this thread.
pub fn check(n: i64, names: Names, words: impl FnOnce() -> String) -> Result<i64, Error> {
    if n < 0 { Err(Error::Status { status: n as i32, name: names(n as i32), words: words() }) } else { Ok(n) }
}

/// The copy-out, a status failing with `words`; the status `absent` names, an answer rather than a failure, as none.
pub fn read(
    call: impl FnMut(*mut u8, usize) -> i64,
    absent: Option<i32>,
    names: Names,
    words: impl FnOnce() -> String,
) -> Result<Option<Vec<u8>>, Error> {
    match copy_out(call) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(n) if Some(n) == absent.map(i64::from) => Ok(None),
        Err(n) => Err(Error::Status { status: n as i32, name: names(n as i32), words: words() }),
    }
}

/// The words a last_error copies out, empty where it has none.
pub fn words(call: impl FnMut(*mut u8, usize) -> i64) -> String {
    copy_out(call).map(|b| String::from_utf8_lossy(&b).into_owned()).unwrap_or_default()
}

/// An answer read into its record.
pub fn json<T: DeserializeOwned>(call: &'static str, bytes: &[u8]) -> Result<T, Error> {
    serde_json::from_slice(bytes).map_err(|e| Error::Shape { call, words: e.to_string() })
}

/// An answer as text.
pub fn string(call: &'static str, bytes: Vec<u8>) -> Result<String, Error> {
    String::from_utf8(bytes).map_err(|e| Error::Shape { call, words: e.to_string() })
}

/// A record as the JSON text a call takes.
pub fn to_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value).expect("a record serialises")
}

/// Text as the headers take it; none is the absent string, a null pointer with a zero length.
pub fn text(s: Option<&str>) -> (*const u8, usize) {
    s.map_or((std::ptr::null(), 0), |s| (s.as_ptr(), s.len()))
}

/// The length asked with a null buffer, then the copy, asked again when the answer grew between the two calls.
pub fn copy_out(mut call: impl FnMut(*mut u8, usize) -> i64) -> Result<Vec<u8>, i64> {
    let mut n = call(std::ptr::null_mut(), 0);
    loop {
        if n < 0 {
            return Err(n);
        }
        let mut buf = vec![0u8; n as usize];
        let m = call(buf.as_mut_ptr(), buf.len());
        if m >= 0 && m as usize <= buf.len() {
            buf.truncate(m as usize);
            return Ok(buf);
        }
        n = m;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A call answering `answers` in turn, each written only when the buffer holds it.
    fn answering(answers: &[&'static [u8]]) -> impl FnMut(*mut u8, usize) -> i64 {
        let mut at = 0;
        let answers = answers.to_vec();
        move |out, len| {
            let a = answers[at.min(answers.len() - 1)];
            at += 1;
            if !out.is_null() && len >= a.len() {
                unsafe { std::ptr::copy_nonoverlapping(a.as_ptr(), out, a.len()) };
            }
            a.len() as i64
        }
    }

    #[test]
    fn an_answer_is_asked_its_length_then_copied() {
        assert_eq!(copy_out(answering(&[b"[1,2]"])).unwrap(), b"[1,2]");
        assert_eq!(copy_out(answering(&[b""])).unwrap(), b"");
    }

    #[test]
    fn an_answer_that_grew_between_the_two_calls_is_asked_again() {
        assert_eq!(copy_out(answering(&[b"[1]", b"[1,2,3]", b"[1,2,3]"])).unwrap(), b"[1,2,3]");
    }

    #[test]
    fn a_status_is_handed_back() {
        assert_eq!(copy_out(|_, _| -1107), Err(-1107));
        let mut first = true;
        assert_eq!(copy_out(move |_, _| if std::mem::take(&mut first) { 4 } else { -1108 }), Err(-1108));
    }

    #[test]
    fn none_is_the_absent_string() {
        assert_eq!(text(None), (std::ptr::null(), 0));
        let (p, n) = text(Some("abc"));
        assert!(!p.is_null() && n == 3);
    }
}
