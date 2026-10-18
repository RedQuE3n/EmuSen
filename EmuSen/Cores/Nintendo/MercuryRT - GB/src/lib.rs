//! MercuryRT, the Game Boy and Game Boy Color core in Rust, called through the core ABI v1 (EmuSen_CoreAPI.md). See Mercury_Native.md.

pub mod apu;
pub mod cpu;
pub mod debug;
pub mod ffi;
pub mod machine;
pub mod memory;
#[cfg(test)]
mod naming;
pub mod ppu;
pub mod state;

/// A field C# marks `[SkipInState]`; see emusen-native.
pub use emusen_native::Skip;
