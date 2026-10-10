//! `emusen_platform`: the one library the C# programs load, and the C layer over each platform crate.
//!
//! Nothing is decided here. Each export converts its arguments, calls the crate that owns the rule, and converts the
//! answer; `Platform/include/emusen_platform.h` is the interface's specification and EmuSen_RustPlatform.md §3 its
//! rules: the host calls and the library returns, results are copied out by the length-query idiom, statuses are
//! signed, and nothing unwinds.

use std::cell::RefCell;

pub mod endymion;
pub mod endymion_devices;
pub mod galaxia;
pub mod galaxia_config;
pub mod serenity;

/// The interface's one number, matched exactly by the host that was built against it.
pub const ABI_VERSION: u32 = 5;

/// Every status the interface returns. Zero and up is success: a count or a length.
pub mod status {
    /// A null pointer where one was required.
    pub const NULL: i32 = -1;
    /// Text that is not UTF-8.
    pub const BAD_STRING: i32 = -5;
    /// An operation this build does not have.
    pub const NOT_SUPPORTED: i32 = -256;
    /// Reading or writing failed, for a reason that is none of the ones below.
    pub const IO: i32 = -1024;
    /// Text that is not the document it should be: not JSON as .NET reads it, or not what its model allows.
    pub const PARSE: i32 = -1025;
    /// What was asked for does not exist, where that is an answer and not a failure.
    pub const ABSENT: i32 = -1030;
    /// A file or directory that had to exist does not.
    pub const NOT_FOUND: i32 = -1031;
    /// The system refused access.
    pub const ACCESS: i32 = -1032;
    /// A kind, an operation or a number the call does not know.
    pub const BAD_ARGUMENT: i32 = -1033;
}

thread_local! {
    static LAST_ERROR: RefCell<String> = const { RefCell::new(String::new()) };
    static DIAGNOSTICS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

/// Records why a call failed, for `emusen_platform_last_error` on the same thread, and returns its status.
pub(crate) fn fail(status: i32, words: impl Into<String>) -> i32 {
    LAST_ERROR.with_borrow_mut(|last| *last = words.into());
    status
}

/// The status for an I/O error, with its words recorded.
pub(crate) fn fail_io(error: &std::io::Error) -> i32 {
    let status = match error.kind() {
        std::io::ErrorKind::NotFound => status::NOT_FOUND,
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::IsADirectory => status::ACCESS,
        _ => status::IO,
    };
    fail(status, error.to_string())
}

/// Queues what the C# reports through `ConfigDiagnostics`: the host drains it after the call and reports each message itself.
pub(crate) fn report(message: String) {
    DIAGNOSTICS.with_borrow_mut(|queue| queue.push(message));
}

pub(crate) fn diagnostics_waiting() -> usize {
    DIAGNOSTICS.with_borrow(Vec::len)
}

/// Drops the messages queued since `waiting`, for a call whose answer did not fit and will be asked again.
pub(crate) fn diagnostics_rewind(waiting: usize) {
    DIAGNOSTICS.with_borrow_mut(|queue| queue.truncate(waiting));
}

/// The text a caller passed: none for a null pointer, a status for bytes that are not UTF-8.
///
/// # Safety
/// `data` must be valid for `len` bytes, or null.
pub(crate) unsafe fn text<'a>(data: *const u8, len: usize) -> Result<Option<&'a str>, i32> {
    if data.is_null() {
        return Ok(None);
    }
    let bytes = if len == 0 { &[][..] } else { unsafe { std::slice::from_raw_parts(data, len) } };
    std::str::from_utf8(bytes).map(Some).map_err(|_| fail(status::BAD_STRING, "the text is not UTF-8"))
}

/// The text a caller had to pass.
///
/// # Safety
/// As `text`.
pub(crate) unsafe fn required<'a>(data: *const u8, len: usize) -> Result<&'a str, i32> {
    unsafe { text(data, len) }?.ok_or_else(|| fail(status::NULL, "a required text is null"))
}

/// The length-query idiom: the whole length, with `bytes` copied only when `out` holds all of them.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
pub(crate) unsafe fn put(bytes: &[u8], out: *mut u8, len: usize) -> i64 {
    if !out.is_null() && bytes.len() <= len {
        unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, bytes.len()) };
    }
    bytes.len() as i64
}

/// The interface's version, which the host matches exactly.
#[unsafe(no_mangle)]
pub extern "C" fn emusen_platform_abi_version() -> u32 {
    ABI_VERSION
}

/// Where a panic is written before the process aborts; the first path given stands.
///
/// # Safety
/// `path` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_platform_set_crash_log(path: *const u8, len: usize) -> i32 {
    let path = match unsafe { required(path, len) } {
        Ok(path) => path,
        Err(status) => return status,
    };
    match std::ffi::CString::new(path) {
        Ok(path) => unsafe { emusen_native::abi::install_crash_log("emusen_platform", path.as_ptr()) },
        Err(_) => fail(status::BAD_STRING, "the path holds a NUL"),
    }
}

/// The words for the last failing call on the calling thread.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_platform_last_error(out: *mut u8, len: usize) -> i64 {
    LAST_ERROR.with_borrow(|last| unsafe { put(last.as_bytes(), out, len) })
}

/// The calling thread's waiting diagnostics, each ended by a NUL, removed once a buffer has taken them all.
///
/// # Safety
/// `out` must be valid for `len` bytes, or null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn emusen_platform_diagnostics(out: *mut u8, len: usize) -> i64 {
    DIAGNOSTICS.with_borrow_mut(|queue| {
        let joined = join_nul(queue);
        let length = unsafe { put(&joined, out, len) };
        if !out.is_null() && joined.len() <= len {
            queue.clear();
        }
        length
    })
}

/// Texts as one buffer, each followed by a NUL, which no path or message holds.
pub(crate) fn join_nul(texts: &[String]) -> Vec<u8> {
    let mut joined = Vec::with_capacity(texts.iter().map(|t| t.len() + 1).sum());
    for text in texts {
        joined.extend_from_slice(text.as_bytes());
        joined.push(0);
    }
    joined
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_result_is_copied_only_when_it_all_fits() {
        let mut out = [0xAAu8; 4];
        assert_eq!(unsafe { put(b"abcde", out.as_mut_ptr(), 4) }, 5);
        assert_eq!(out, [0xAA; 4]);
        assert_eq!(unsafe { put(b"abcd", out.as_mut_ptr(), 4) }, 4);
        assert_eq!(&out, b"abcd");
        assert_eq!(unsafe { put(b"abc", std::ptr::null_mut(), 0) }, 3);
    }

    #[test]
    fn text_tells_null_from_empty_and_refuses_what_is_not_utf8() {
        assert_eq!(unsafe { text(std::ptr::null(), 0) }, Ok(None));
        assert_eq!(unsafe { text(b"".as_ptr(), 0) }, Ok(Some("")));
        assert_eq!(unsafe { text(b"caf\xC3\xA9".as_ptr(), 5) }, Ok(Some("café")));
        assert_eq!(unsafe { text(b"\xFF".as_ptr(), 1) }, Err(status::BAD_STRING));
        assert_eq!(unsafe { required(std::ptr::null(), 0) }, Err(status::NULL));
        let mut words = [0u8; 64];
        let length = unsafe { emusen_platform_last_error(words.as_mut_ptr(), words.len()) } as usize;
        assert_eq!(&words[..length], b"a required text is null");
    }

    #[test]
    fn diagnostics_wait_until_a_buffer_takes_them_all() {
        report("first".to_string());
        report("second".to_string());
        assert_eq!(unsafe { emusen_platform_diagnostics(std::ptr::null_mut(), 0) }, 13);
        let mut small = [0u8; 5];
        assert_eq!(unsafe { emusen_platform_diagnostics(small.as_mut_ptr(), small.len()) }, 13);
        let mut out = [0u8; 13];
        assert_eq!(unsafe { emusen_platform_diagnostics(out.as_mut_ptr(), out.len()) }, 13);
        assert_eq!(&out, b"first\0second\0");
        assert_eq!(unsafe { emusen_platform_diagnostics(out.as_mut_ptr(), out.len()) }, 0);
    }

    /// The names after `extern "C" fn` in a source file.
    fn exports_in(source: &str) -> Vec<String> {
        source
            .split("extern \"C\" fn ")
            .skip(1)
            .map(|rest| rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect::<String>())
            .filter(|name| name.starts_with("emusen_"))
            .collect()
    }

    /// The names the header declares: an identifier beginning `emusen_` and followed by an opening parenthesis.
    fn declared_in(header: &str) -> Vec<String> {
        let code: String = header.split("/*").map(|part| part.split_once("*/").map_or(part, |(_, code)| code)).collect();
        let mut names = Vec::new();
        for (at, _) in code.match_indices("emusen_") {
            let name: String = code[at..].chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '_').collect();
            if code[at + name.len()..].starts_with('(') {
                names.push(name);
            }
        }
        names
    }

    #[test]
    fn the_header_declares_exactly_what_the_library_exports() {
        let mut exported = exports_in(include_str!("lib.rs"));
        exported.extend(exports_in(include_str!("galaxia.rs")));
        exported.extend(exports_in(include_str!("galaxia_config.rs")));
        exported.extend(exports_in(include_str!("endymion.rs")));
        exported.extend(exports_in(include_str!("endymion_devices.rs")));
        exported.extend(exports_in(include_str!("serenity.rs")));
        exported.sort();
        let mut declared = declared_in(include_str!("../../include/emusen_platform.h"));
        declared.sort();
        assert!(exported.len() >= 4, "{exported:?}");
        assert_eq!(declared, exported);
    }

    #[test]
    fn the_header_carries_the_version_and_every_status() {
        let header = include_str!("../../include/emusen_platform.h");
        assert!(header.contains(&format!("#define EMUSEN_PLATFORM_ABI_VERSION {ABI_VERSION}u")));
        for (name, value) in [
            ("EMUSEN_PLATFORM_NULL", status::NULL),
            ("EMUSEN_PLATFORM_BAD_STRING", status::BAD_STRING),
            ("EMUSEN_PLATFORM_NOT_SUPPORTED", status::NOT_SUPPORTED),
            ("EMUSEN_PLATFORM_IO", status::IO),
            ("EMUSEN_PLATFORM_PARSE", status::PARSE),
            ("EMUSEN_PLATFORM_ABSENT", status::ABSENT),
            ("EMUSEN_PLATFORM_NOT_FOUND", status::NOT_FOUND),
            ("EMUSEN_PLATFORM_ACCESS", status::ACCESS),
            ("EMUSEN_PLATFORM_BAD_ARGUMENT", status::BAD_ARGUMENT),
        ] {
            assert!(header.contains(&format!("#define {name} ({value})")), "{name}");
        }
    }
}
