//! VenusRT, the SNES core in Rust, called through the common native interface (EmuSen_NativeCores.md).
//! Written from hardware documents under VenusRT_Plan.md §1's clean-room protocol; the build record is VenusRT_Native.md.
//!
//! Built so far: the 65816 (`cpu`), the cartridge (`cart`) and the S-CPU's bus and master clock (`bus`); the PPU, the
//! APU and the S-CPU's other devices come with later stages.

pub mod apu;
pub mod cpu;
pub mod bus;
pub mod cart;
pub mod chips;
pub mod ffi;
pub mod machine;
pub mod ppu;
pub mod scpu;
pub mod state;

#[cfg(test)]
mod cputest;
#[cfg(test)]
mod singlestep;
#[doc(hidden)]
pub mod speedtest;
