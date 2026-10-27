//! VenusRT's state errors over the shared codec. The format is VenusRT's own, "VNRT" (VenusRT_Plan.md §5.6).

pub use emusen_native::{StateReader, StateWriter, Truncated};

/// "VNRT" as the bytes a state starts with.
pub const STATE_MAGIC: u32 = u32::from_le_bytes(*b"VNRT");
/// C# Venus's magic, "SNES": a state it made is refused as foreign, and the host names the engine.
pub const VENUS_MAGIC: u32 = 0x5345_4E53;
/// The words a refused C# Venus state is given, naming the engine that made it.
pub const VENUS_STATE_WORDS: &str = "it was saved by Venus (C#), the C# SNES engine, whose states VenusRT cannot read";
pub const STATE_VERSION: i32 = 22;
/// A state written under the other NEC DSP engine than the one this machine runs (VenusRT_DspHle.md §2.2).
pub const STATUS_OTHER_DSP_ENGINE: i32 = -11;

emusen_native::state_error! {
    pub enum StateError {
        DspEngine { saved: u8, running: u8 } => STATUS_OTHER_DSP_ENGINE,
    }
}

pub type StateResult<T = ()> = Result<T, StateError>;
