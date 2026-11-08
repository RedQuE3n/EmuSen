//! A failure as the API reports it: a status with the words last_error gave on the same thread, or a library refused
//! at load, or JSON this client cannot read.

use crate::raw;

pub use emusen_c_calls::Error;

/// This crate's result.
pub type Result<T> = std::result::Result<T, Error>;

/// A failure the API returned, named.
pub(crate) fn status(status: i32, words: String) -> Error {
    Error::Status { status, name: status_name(status), words }
}

/// True for EMUSEN_SVC_ABSENT, an answer rather than a failure.
pub(crate) fn absent(e: &Error) -> bool {
    e.status() == Some(raw::EMUSEN_SVC_ABSENT)
}

/// A status's name as the headers spell it.
pub fn status_name(status: i32) -> &'static str {
    match status {
        raw::EMUSEN_SVC_NULL => "EMUSEN_SVC_NULL",
        raw::EMUSEN_SVC_BAD_STRING => "EMUSEN_SVC_BAD_STRING",
        raw::EMUSEN_SVC_NO_SESSION => "EMUSEN_SVC_NO_SESSION",
        raw::EMUSEN_SVC_NO_SUCH_SPACE => "EMUSEN_SVC_NO_SUCH_SPACE",
        raw::EMUSEN_SVC_READ_ONLY => "EMUSEN_SVC_READ_ONLY",
        raw::EMUSEN_SVC_NOT_SUPPORTED => "EMUSEN_SVC_NOT_SUPPORTED",
        raw::EMUSEN_SVC_BAD_ARGUMENT => "EMUSEN_SVC_BAD_ARGUMENT",
        raw::EMUSEN_SVC_NO_SUCH_ID => "EMUSEN_SVC_NO_SUCH_ID",
        raw::EMUSEN_SVC_PARSE => "EMUSEN_SVC_PARSE",
        raw::EMUSEN_SVC_ABSENT => "EMUSEN_SVC_ABSENT",
        raw::EMUSEN_SVC_FAULT => "EMUSEN_SVC_FAULT",
        raw::EMUSEN_CRYSTAL_NO_ENGINE => "EMUSEN_CRYSTAL_NO_ENGINE",
        raw::EMUSEN_CRYSTAL_FIRMWARE => "EMUSEN_CRYSTAL_FIRMWARE",
        raw::EMUSEN_CRYSTAL_BUSY => "EMUSEN_CRYSTAL_BUSY",
        raw::EMUSEN_CRYSTAL_BAD_STRUCT => "EMUSEN_CRYSTAL_BAD_STRUCT",
        raw::EMUSEN_CRYSTAL_IO => "EMUSEN_CRYSTAL_IO",
        raw::EMUSEN_CRYSTAL_STATE_REFUSED => "EMUSEN_CRYSTAL_STATE_REFUSED",
        raw::EMUSEN_CRYSTAL_NOT_APPROVED => "EMUSEN_CRYSTAL_NOT_APPROVED",
        _ => "an unknown status",
    }
}
