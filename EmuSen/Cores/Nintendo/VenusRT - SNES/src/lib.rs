//! VenusRT, the SNES core in Rust, called through the common native interface (EmuSen_NativeCores.md).
//! Written from hardware documents under VenusRT_Plan.md §1's clean-room protocol; the build record is VenusRT_Native.md.
//!
//! Stage 0: the machine is a stub with the memories and the state format, so that the runners and the build have an
//! engine to drive. No part of the console is emulated yet.

pub mod ffi;
pub mod machine;
pub mod state;

#[cfg(test)]
mod singlestep;
