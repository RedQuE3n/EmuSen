//! MarsRT's state: the shared codec (`emusen-native`) and the refusals only Mars's format has. See Mars_Native.md §5.1.

use emusen_native::ffi::status;
pub use emusen_native::{State, StateReader, StateWriter, Truncated};

/// Why a state could not be read or written; each has a status code for the C ABI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    NotAMarsState(u32),
    Version(i32),
    RdramSize(i32),
    PendingOverflow(i32),
    BufferTooSmall { needed: usize },
    PendingInState(usize),
}

impl StateError {
    pub fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::NotAMarsState(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::RdramSize(_) => -12,
            StateError::PendingOverflow(_) => -13,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
            StateError::PendingInState(_) => -14,
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

pub type StateResult<T = ()> = Result<T, StateError>;

/// A zeroed array on the heap, too large to build on the stack.
pub fn boxed<T: Copy, const N: usize>(value: T) -> Box<[T; N]> {
    vec![value; N].into_boxed_slice().try_into().unwrap_or_else(|_| unreachable!())
}
