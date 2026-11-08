//! The C layers' two shared halves: a caller's text read, and an answer made once and copied by the length-query idiom - see EmuSen_RustPlatform.md §47.25.

use std::cell::RefCell;
use std::thread::LocalKey;

/// A layer's answer that did not fit, by the call and the request that made it, taken by the same call asked again.
pub(crate) type Waiting = RefCell<Option<(u8, Vec<u8>, Vec<u8>)>>;

/// The statuses every header gives a null text with a length and one that is not UTF-8.
const NULL: i32 = -1;
const BAD_STRING: i32 = -5;

/// The text a caller passed; an empty one may come as a null pointer with no length. `fail` records why not.
pub(crate) unsafe fn text<'a>(data: *const u8, len: usize, fail: impl FnOnce(i32, &str) -> i32) -> Result<&'a str, i32> {
    if data.is_null() {
        return if len == 0 { Ok("") } else { Err(fail(NULL, "a text with a length is null")) };
    }
    let bytes = if len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    std::str::from_utf8(bytes).map_err(|_| fail(BAD_STRING, "the text is not UTF-8"))
}

/// An answer made once: copied when it fits, else kept in the layer's slot for the same call asked again.
pub(crate) unsafe fn answer<E>(waiting: &'static LocalKey<Waiting>, call: u8, request: &str, out: *mut u8, len: usize, make: impl FnOnce() -> Result<String, E>, failed: impl FnOnce(E) -> i32) -> i64 {
    let kept = waiting.with_borrow_mut(|w| match w {
        Some((c, r, _)) if *c == call && r.as_slice() == request.as_bytes() => w.take().map(|(_, _, a)| a),
        _ => None,
    });
    let bytes = match kept {
        Some(bytes) => bytes,
        None => match make() {
            Ok(text) => text.into_bytes(),
            Err(e) => return failed(e) as i64,
        },
    };
    if out.is_null() || bytes.len() > len {
        let length = bytes.len();
        waiting.with_borrow_mut(|w| *w = Some((call, request.as_bytes().to_vec(), bytes)));
        return length as i64;
    }
    unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    bytes.len() as i64
}
