//! Nephrite, the Sega Genesis / Mega Drive core in Rust, with the Sega CD and the 32X as attachments of the same
//! machine, on the core ABI v1. Written from hardware documents under Nephrite_Plan.md §1's clean-room protocol; the
//! build record is Nephrite_Native.md.
//!
//! Stage 4, step 4: the Genesis's buses, cartridges, I/O, the scheduler and the interrupts, with the CPUs from the
//! Beryl crates, the VDP's ports, FIFO and DMA on the slot schedule, and its picture in modes 5 and 4, interlaced and
//! in PAL, mode 5's lines drawn in spans up to each write; the sound is to come.

pub mod cart;
pub mod eeprom;
pub mod fifo_records;
pub mod genesis;
pub mod io;
pub mod machine;
pub mod render;
pub mod media;
#[cfg(test)]
mod board_rows;
#[cfg(test)]
mod pictures;
#[cfg(test)]
mod programs;
pub mod state;
pub mod v1;
pub mod vdp;
