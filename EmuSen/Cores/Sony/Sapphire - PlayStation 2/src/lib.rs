//! Sapphire, the Sony PlayStation 2 core in Rust, on the core ABI v1. Written from hardware documents under
//! Sapphire_Plan.md §1.4's clean-room protocol; the build record is Sapphire_Native.md.
//!
//! Stage S1: a stub. The image is read (`media`): an EE program as an ELF, or a disc whose SYSTEM.CNF names a BOOT2
//! program; the machine (`machine`) holds the memories, a frame counter, a black picture and silence, and nothing is
//! emulated. `state` is the format SPH2, and `v1` the core ABI's glue.

pub mod bios;
pub mod machine;
pub mod media;
pub mod state;
pub mod v1;
