//! MarsRT, the N64 core in Rust, on the core ABI v1, and the C# Mars's native components, called through a C ABI. See Mars_Native.md.

pub mod cpu;
pub mod ffi;
pub mod machine;
pub mod memory;
#[cfg(test)]
mod naming;
pub mod rdp;
pub mod rom;
pub mod rsp;
pub mod state;
#[cfg(test)]
mod tests;
pub mod vi;

/// A field C# marks `[SkipInState]`; see emusen-native.
pub use emusen_native::Skip;
