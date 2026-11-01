//! Since 1.16: what a create or an insert gives beside its bytes, the image's path (`IMAGE_PATH`) and the names of
//! named battery files, read by the core during that call through [`image_path`] and [`file_name`]
//! (EmuSen_LibretroHost.md §2, E1 and E3).

use std::cell::RefCell;
use std::ffi::{CStr, c_char};

use super::sys::flags;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Request {
    pub image_path: Option<String>,
    pub names: Vec<(u32, String)>,
}

thread_local! {
    static CURRENT: RefCell<Option<Request>> = const { RefCell::new(None) };
}

/// The image's file during the create or insert in flight, where the host gave one to a core claiming `IMAGE_PATH`.
pub fn image_path() -> Option<String> {
    CURRENT.with(|c| c.borrow().as_ref().and_then(|r| r.image_path.clone()))
}

/// The name of named battery file `which` during the create or insert in flight.
pub fn file_name(which: u32) -> Option<String> {
    CURRENT.with(|c| c.borrow().as_ref().and_then(|r| r.names.iter().find(|n| n.0 == which).map(|n| n.1.clone())))
}

/// Every named file's number and name during the call in flight.
pub fn file_names() -> Vec<(u32, String)> {
    CURRENT.with(|c| c.borrow().as_ref().map(|r| r.names.clone()).unwrap_or_default())
}

/// `f` with `request` readable, then nothing.
pub fn with<R>(request: Request, f: impl FnOnce() -> R) -> R {
    let previous = CURRENT.with(|c| c.replace(Some(request)));
    let r = f();
    CURRENT.with(|c| *c.borrow_mut() = previous);
    r
}

/// Whether a named file's name is one a host may store: UTF-8, relative, `/`-separated, no part empty, `.` or `..`, no
/// backslash and no NUL, at most 255 bytes.
pub fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 255 && !name.contains(['\\', '\0', ':']) && name.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

/// A C string the host passed, as text, or why not.
pub unsafe fn text(p: *const c_char) -> Option<Result<String, ()>> {
    (!p.is_null()).then(|| unsafe { CStr::from_ptr(p) }.to_str().map(str::to_owned).map_err(|_| ()))
}

/// Whether `which` is in the named range.
pub fn is_named(which: u32) -> bool {
    (flags::FILE_NAMED..flags::FILE_MEDIA).contains(&which)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_that_escape_or_hide_are_refused() {
        for bad in ["", "/abs", "a/../b", "..", "a//b", "a/./b", "a\\b", "c:x", "a/"] {
            assert!(!valid_name(bad), "{bad:?}");
        }
        for good in ["card.bin", "memcards/Mcd001.ps2", "a.b/c-d_e"] {
            assert!(valid_name(good), "{good:?}");
        }
    }

    #[test]
    fn the_request_is_readable_only_in_its_call() {
        let r = Request { image_path: Some("/g/game.iso".into()), names: vec![(flags::FILE_NAMED, "card.bin".into())] };
        with(r, || {
            assert_eq!(image_path().as_deref(), Some("/g/game.iso"));
            assert_eq!(file_name(flags::FILE_NAMED).as_deref(), Some("card.bin"));
        });
        assert_eq!(image_path(), None);
    }
}
