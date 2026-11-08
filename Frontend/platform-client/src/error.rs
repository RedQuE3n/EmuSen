//! A failure as the platform library reports it, emusen-c-calls' error with these headers' status names; the words are
//! the platform's, the library handle's or the scrape table's last_error, read on the same thread.

use crate::raw;

pub use emusen_c_calls::Error;

/// This crate's result.
pub type Result<T> = std::result::Result<T, Error>;

/// A failure the library returned, named.
pub(crate) fn status(status: i32, words: String) -> Error {
    Error::Status { status, name: status_name(status), words }
}

/// True for EMUSEN_PLATFORM_ABSENT, an answer rather than a failure.
pub(crate) fn absent(e: &Error) -> bool {
    e.status() == Some(raw::EMUSEN_PLATFORM_ABSENT)
}

/// A status's name as the headers spell it; the bands the library and the scraper share with the platform are named once.
pub fn status_name(status: i32) -> &'static str {
    match status {
        raw::EMUSEN_PLATFORM_NULL => "EMUSEN_PLATFORM_NULL",
        raw::EMUSEN_PLATFORM_BAD_STRING => "EMUSEN_PLATFORM_BAD_STRING",
        raw::EMUSEN_PLATFORM_NOT_SUPPORTED => "EMUSEN_PLATFORM_NOT_SUPPORTED",
        raw::EMUSEN_PLATFORM_IO => "EMUSEN_PLATFORM_IO",
        raw::EMUSEN_PLATFORM_PARSE => "EMUSEN_PLATFORM_PARSE",
        raw::EMUSEN_PLATFORM_NEWER_FILE => "EMUSEN_PLATFORM_NEWER_FILE",
        raw::EMUSEN_PLATFORM_DEVICE => "EMUSEN_PLATFORM_DEVICE",
        raw::EMUSEN_PLATFORM_ABSENT => "EMUSEN_PLATFORM_ABSENT",
        raw::EMUSEN_PLATFORM_NOT_FOUND => "EMUSEN_PLATFORM_NOT_FOUND",
        raw::EMUSEN_PLATFORM_ACCESS => "EMUSEN_PLATFORM_ACCESS",
        raw::EMUSEN_PLATFORM_BAD_ARGUMENT => "EMUSEN_PLATFORM_BAD_ARGUMENT",
        raw::EMUSEN_PLATFORM_REFUSED => "EMUSEN_PLATFORM_REFUSED",
        raw::EMUSEN_LIBRARY_NEWER_FILE => "EMUSEN_LIBRARY_NEWER_FILE",
        raw::EMUSEN_LIBRARY_DATABASE => "EMUSEN_LIBRARY_DATABASE",
        raw::EMUSEN_LIBRARY_NO_SQLITE => "EMUSEN_LIBRARY_NO_SQLITE",
        raw::EMUSEN_LIBRARY_BAD_REQUEST => "EMUSEN_LIBRARY_BAD_REQUEST",
        raw::EMUSEN_LIBRARY_CANCELLED => "EMUSEN_LIBRARY_CANCELLED",
        raw::EMUSEN_LIBRARY_BUSY => "EMUSEN_LIBRARY_BUSY",
        raw::EMUSEN_LIBRARY_IO => "EMUSEN_LIBRARY_IO",
        raw::EMUSEN_SCRAPE_NEWER_FILE => "EMUSEN_SCRAPE_NEWER_FILE",
        raw::EMUSEN_SCRAPE_DATABASE => "EMUSEN_SCRAPE_DATABASE",
        raw::EMUSEN_SCRAPE_NO_SQLITE => "EMUSEN_SCRAPE_NO_SQLITE",
        raw::EMUSEN_SCRAPE_BAD_REQUEST => "EMUSEN_SCRAPE_BAD_REQUEST",
        raw::EMUSEN_SCRAPE_BUSY => "EMUSEN_SCRAPE_BUSY",
        raw::EMUSEN_SCRAPE_IO => "EMUSEN_SCRAPE_IO",
        raw::EMUSEN_SCRAPE_NO_LIBRARY => "EMUSEN_SCRAPE_NO_LIBRARY",
        raw::EMUSEN_SCRAPE_STORE_IN_USE => "EMUSEN_SCRAPE_STORE_IN_USE",
        _ => "an unknown status",
    }
}
