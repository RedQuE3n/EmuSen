//! MoonRT's state: the shared codec (`emusen-native`) and the refusals only Moon's format has. See Moon_Native.md §3.1.

use emusen_native::ffi::status;
pub use emusen_native::{State, StateReader, StateWriter, Truncated};

/// Why a state could not be read or written; each has a status code for the C ABI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    NotAMoonState(u32),
    Version(i32),
    BufferTooSmall { needed: usize },
    /// A state of one console, by NES 2.0's timing number, loaded into a machine of another (Moon_PAL.md §7).
    OtherModel { saved: u8, running: u8 },
}

/// A state saved on the NTSC NES, the PAL NES or the Dendy, loaded into a machine of another of them.
pub const STATUS_OTHER_MODEL: i32 = -12;

impl StateError {
    pub fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::NotAMoonState(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
            StateError::OtherModel { .. } => STATUS_OTHER_MODEL,
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
