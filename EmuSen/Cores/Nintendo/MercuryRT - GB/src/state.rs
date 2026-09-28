//! MercuryRT's state: the shared codec (`emusen-native`) and the refusals only Mercury's format has. See Mercury_Native.md §3.1.

use emusen_native::ffi::status;
pub use emusen_native::{State, StateReader, StateWriter, StringError, Truncated};

/// Why a state could not be read or written; each has a status code for the C ABI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    NotAMercuryState(u32),
    Version(i32),
    BadStringLength { at: usize },
    BufferTooSmall { needed: usize },
}

impl StateError {
    pub fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::NotAMercuryState(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::BadStringLength { .. } => status::BAD_STRING,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl emusen_native::ffi::Status for StateError {
    fn status(&self) -> i32 {
        StateError::status(self)
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

impl From<StringError> for StateError {
    fn from(e: StringError) -> Self {
        match e {
            StringError::Truncated(t) => t.into(),
            StringError::BadLength { at } => StateError::BadStringLength { at },
        }
    }
}

pub type StateResult<T = ()> = Result<T, StateError>;
