//! emusen-c-calls' idioms with this header's status names and its ABSENT.

use crate::error::{Result, status_name};
use crate::raw::EMUSEN_SVC_ABSENT;

pub(crate) use emusen_c_calls::{json, string, text, to_json};

/// A negative return as a failure with `words`, read at once on this thread.
pub(crate) fn check(n: i64, words: impl FnOnce() -> String) -> Result<i64> {
    emusen_c_calls::check(n, status_name, words)
}

/// The length-query copy-out, a status failing with `words`.
pub(crate) fn copy_out(call: impl FnMut(*mut u8, usize) -> i64, words: impl FnOnce() -> String) -> Result<Vec<u8>> {
    emusen_c_calls::read(call, None, status_name, words).map(Option::unwrap_or_default)
}

/// As `copy_out`, with EMUSEN_SVC_ABSENT as none.
pub(crate) fn copy_out_absent(call: impl FnMut(*mut u8, usize) -> i64, words: impl FnOnce() -> String) -> Result<Option<Vec<u8>>> {
    emusen_c_calls::read(call, Some(EMUSEN_SVC_ABSENT), status_name, words)
}
