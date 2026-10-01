//! VenusRT's state errors over the shared codec. The format is VenusRT's own, "VNRT" (VenusRT_Plan.md §5.6).

use emusen_native::ffi::status;
pub use emusen_native::{StateReader, StateWriter, Truncated};

/// "VNRT" as the bytes a state starts with.
pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"VNRT");
/// C# Venus's magic, "SNES": a state it made is refused as foreign, and the host names the engine.
pub const VENUS_MAGIC: u32 = 0x5345_4E53;
pub const STATE_VERSION: i32 = 6;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateError {
    Truncated { at: usize, wanted: usize },
    Foreign(u32),
    Version(i32),
    BufferTooSmall { needed: usize },
}

impl emusen_native::ffi::Status for StateError {
    fn status(&self) -> i32 {
        match self {
            StateError::Truncated { .. } => status::TRUNCATED,
            StateError::Foreign(_) => status::FOREIGN,
            StateError::Version(_) => status::VERSION,
            StateError::BufferTooSmall { .. } => status::BUFFER_TOO_SMALL,
        }
    }
}

impl From<Truncated> for StateError {
    fn from(t: Truncated) -> Self {
        StateError::Truncated { at: t.at, wanted: t.wanted }
    }
}

pub type StateResult<T = ()> = Result<T, StateError>;
