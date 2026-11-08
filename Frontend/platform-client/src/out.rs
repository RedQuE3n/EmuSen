//! emusen-c-calls' idioms with these headers' status names and the platform's ABSENT.

use crate::error::{Result, status_name};
use crate::raw::EMUSEN_PLATFORM_ABSENT;

pub(crate) use emusen_c_calls::{json, string, text, to_json, words};

/// A negative return as a failure with `words`, read at once on this thread.
pub(crate) fn check(n: i64, words: impl FnOnce() -> String) -> Result<i64> {
    emusen_c_calls::check(n, status_name, words)
}

/// The length-query copy-out, a status failing with `words`; EMUSEN_PLATFORM_ABSENT as none.
pub(crate) fn copy_out(call: impl FnMut(*mut u8, usize) -> i64, words: impl FnOnce() -> String) -> Result<Option<Vec<u8>>> {
    emusen_c_calls::read(call, Some(EMUSEN_PLATFORM_ABSENT), status_name, words)
}

/// The copy-out of a header with no ABSENT (the library's, the scrape table's).
pub(crate) fn answer(call: impl FnMut(*mut u8, usize) -> i64, words: impl FnOnce() -> String) -> Result<Vec<u8>> {
    emusen_c_calls::read(call, None, status_name, words).map(Option::unwrap_or_default)
}
