//! What every Rust core builds from: its state, its audio queue, and the common native interface. The state is the C# `StateSerializer`'s byte format, so a core's state is a C# core's, byte for byte.
//!
//! - [`StateWriter`] writes a state, counts one, or lists its layout; [`StateReader`] reads one back.
//! - [`State`] is one C# class or struct, walked in the C# ordinal order; its `Error` is the core's own.
//! - [`naming`] is the rule that ties each Rust field to the C# name written beside it.
//! - [`Skip`] is a field C# does not serialize; [`ffi`] is the status range and the state exports of a core's C ABI.
//! - [`SampleQueue`] is C#'s `EmuSen.Common.SampleQueue`, a core's undrained audio with its drop-oldest limit.
//! - [`abi`] is the common native interface: the `NativeCore` trait and `native_exports!`, the fixed `emusen_native_*`
//!   names, the version, the capability bits and the interface's status codes (EmuSen_NativeCores.md §3).
//!
//! Nothing here knows a console: magics, versions and a core's own refusals stay in the core. See EmuSen_RustState.md.

pub mod abi;
pub mod ffi;
pub mod naming;
mod reader;
pub mod samples;
mod skip;
mod writer;

pub use reader::StateReader;
pub use samples::SampleQueue;
pub use skip::Skip;
pub use writer::StateWriter;

/// The input ended before a field: `wanted` bytes at `at`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Truncated {
    pub at: usize,
    pub wanted: usize,
}

/// Why `BinaryReader.ReadString` would have refused a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringError {
    Truncated(Truncated),
    /// A length prefix of more than five bytes, or a negative count.
    BadLength { at: usize },
}

impl From<Truncated> for StringError {
    fn from(t: Truncated) -> Self {
        StringError::Truncated(t)
    }
}

/// One C# class or struct: its serialized fields, written and read in the C# ordinal order.
pub trait State {
    /// The core's error type; a short input converts into it.
    type Error: From<Truncated>;
    fn write_state(&self, w: &mut StateWriter);
    fn read_state(&mut self, r: &mut StateReader) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests;
