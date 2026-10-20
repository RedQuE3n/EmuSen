//! Endou, the Sega Master System core in Rust, with the Mark III and the SG-1000's cartridges, on the core ABI v1.
//! Written from hardware documents under Endou_Plan.md §1.4's clean-room protocol; the build record is Endou_Native.md.
//!
//! Stage 8: the machine with the Master System's and the SG-1000's pictures and sound, the BIOS and the debugger.
//! `media` reads the image and `board` chooses its board; `cart` maps it, `io` is the I/O chip, `bus` the Z80's memory
//! and port map with the clock, the BIOS's slot and Beryl's VDP, `sound` Beryl's PSG resampled to the output rate,
//! `model` the consoles' differences, `boot` the open boot program or the player's BIOS, and `machine` runs Beryl's Z80
//! a frame at a time and colours the VDP's picture. `debug` is the debugger's view of it, `state` the format ENDO, and
//! `v1` the core ABI's glue.

pub mod board;
pub mod boot;
pub mod bus;
pub mod cart;
pub mod debug;
pub mod io;
pub mod machine;
pub mod media;
pub mod model;
#[cfg(test)]
mod openboot;
#[cfg(test)]
mod programs;
pub mod sound;
pub mod state;
pub mod v1;
