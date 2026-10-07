//! Nephrite, the Sega Genesis / Mega Drive core in Rust, with the Sega CD and the 32X as attachments of the same
//! machine, on the core ABI v1. Written from hardware documents under Nephrite_Plan.md §1's clean-room protocol; the
//! build record is Nephrite_Native.md.
//!
//! Stage 4, step 4: the Genesis's buses, cartridges, I/O, the scheduler and the interrupts, with the CPUs from the
//! Beryl crates, the VDP's ports, FIFO and DMA on the slot schedule, and its picture in modes 5 and 4, interlaced and
//! in PAL, mode 5's lines drawn in spans up to each write. Stage 5, step 1: the PSG (Beryl's SN76489) and the
//! YM2612's ports, timers, busy flag and DAC, mixed and resampled to 48 kHz. Step 2: the FM operators, then held to
//! the board's samples, with the reset line the YM2612 shares with the Z80; the LFO, SSG-EG, CSM, the timers and the
//! test register, measured on the board.

pub mod cart;
pub mod debugger;
pub mod eeprom;
pub mod fifo_records;
pub mod fm;
pub mod fm_tables;
pub mod genesis;
pub mod io;
#[allow(dead_code)]
mod lfo_tables;
pub mod machine;
pub mod render;
pub mod sound;
#[cfg(test)]
mod sounds;
pub mod media;
#[cfg(test)]
mod board_bus;
#[cfg(test)]
mod board_fm;
#[cfg(test)]
mod board_chip;
#[cfg(test)]
mod board_reads;
#[cfg(test)]
mod board_rows;
#[cfg(test)]
pub(crate) mod pictures;
#[cfg(test)]
mod programs;
pub mod state;
pub mod v1;
pub mod vdp;
#[cfg(test)]
mod voices;
pub mod ym2612;
