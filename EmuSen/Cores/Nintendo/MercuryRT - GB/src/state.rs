//! MercuryRT's state: the shared codec (`emusen-native`) and the refusals only Mercury's format has. See Mercury_Native.md §3.1.

pub use emusen_native::{State, StateReader, StateWriter, StringError, Truncated};

emusen_native::state_error! {
    /// Why a state could not be read or written; each has a status code for the C ABI.
    pub enum StateError {
        BadStringLength { at: usize } => emusen_native::ffi::status::BAD_STRING,
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
