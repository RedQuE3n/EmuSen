//! MoonRT's state: the shared codec (`emusen-native`) and the refusals only Moon's format has. See Moon_Native.md §3.1.

pub use emusen_native::{State, StateReader, StateWriter, Truncated};

/// A state saved on the NTSC NES, the PAL NES or the Dendy, loaded into a machine of another of them.
pub const STATUS_OTHER_MODEL: i32 = -12;

emusen_native::state_error! {
    /// Why a state could not be read or written; each has a status code for the C ABI.
    pub enum StateError {
        /// A state of one console, by NES 2.0's timing number, loaded into a machine of another (Moon_PAL.md §7).
        OtherModel { saved: u8, running: u8 } => STATUS_OTHER_MODEL,
    }
}

pub type StateResult<T = ()> = Result<T, StateError>;
