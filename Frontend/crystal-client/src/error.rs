//! A failure as the API reports it: a status with the words last_error gave on the same thread, or a library refused
//! at load, or JSON this client cannot read.

use crate::raw;

/// What a call of this crate can fail with.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("The frontend API's library at {path} could not be loaded: {words}")]
    Load { path: String, words: String },
    #[error(
        "The frontend API's library at {path} is version {major}.{minor}; this program needs {want_major}.{want_minor} or a later minor of {want_major}."
    )]
    Version { path: String, major: u32, minor: u32, want_major: u32, want_minor: u32 },
    #[error("{words} ({})", status_name(*status))]
    Status { status: i32, words: String },
    #[error("{call} answered JSON this program cannot read: {words}")]
    Shape { call: &'static str, words: String },
}

impl Error {
    /// The status, for a failure the API returned.
    pub fn status(&self) -> Option<i32> {
        match self {
            Error::Status { status, .. } => Some(*status),
            _ => None,
        }
    }

    /// True for EMUSEN_SVC_ABSENT, an answer rather than a failure.
    pub fn is_absent(&self) -> bool {
        self.status() == Some(raw::EMUSEN_SVC_ABSENT)
    }
}

/// This crate's result.
pub type Result<T> = std::result::Result<T, Error>;

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
