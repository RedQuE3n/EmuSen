//! Nephrite, the Sega Genesis / Mega Drive core in Rust, with the Sega CD and the 32X as attachments of the same
//! machine, on the core ABI v1. Written from hardware documents under Nephrite_Plan.md §1's clean-room protocol; the
//! build record is Nephrite_Native.md.
//!
//! Stage 3: the Genesis's buses, cartridges, I/O, the scheduler and the interrupts, with the CPUs from the Beryl
//! crates; the picture is stage 4's.

pub mod cart;
pub mod genesis;
pub mod io;
pub mod machine;
pub mod media;
#[cfg(test)]
mod programs;
pub mod state;
pub mod v1;
pub mod vdp;
