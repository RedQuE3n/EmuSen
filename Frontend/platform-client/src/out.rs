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

/// The room an action's answer is given: two paths of PATH_MAX with every byte escaped fit it.
pub(crate) const ANSWER_ROOM: usize = 64 * 1024;

/// An action's answer from one call into `room` bytes: a call that changes something (a prune, a moved game's manifest
/// rewritten) would run twice under the length query, so it is asked once; an answer longer than the room is a failure
/// whose words say the change was made.
pub(crate) fn once(call: impl FnOnce(*mut u8, usize) -> i64, room: usize, words: impl FnOnce() -> String) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; room];
    let n = check(call(buf.as_mut_ptr(), buf.len()), words)? as usize;
    if n > room {
        let said = format!("The answer of {n} bytes did not fit the {room} bytes given; the change was made.");
        return Err(crate::error::status(crate::raw::EMUSEN_PLATFORM_IO, said));
    }
    buf.truncate(n);
    Ok(buf)
}
