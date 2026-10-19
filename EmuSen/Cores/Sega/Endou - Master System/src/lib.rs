//! Endou, the Sega Master System core in Rust, with the Mark III and the SG-1000's cartridges, on the core ABI v1.
//! Written from hardware documents under Endou_Plan.md §1.4's clean-room protocol; the build record is Endou_Native.md.
//!
//! Stage 1: a stub. The image is read (`media`), the console it is for is chosen, and the machine (`machine`) holds
//! the memories, Beryl's Z80 and SN76489 at reset, a frame counter, a black picture and silence; nothing is emulated.
//! `state` is the format ENDO, and `v1` the core ABI's glue.

pub mod machine;
pub mod media;
pub mod state;
pub mod v1;
