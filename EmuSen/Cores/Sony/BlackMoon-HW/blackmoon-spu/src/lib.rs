//! BlackMoon's SPU voice: what the PlayStation's SPU and each core of the PlayStation 2's SPU2 do alike, sample by
//! sample. SPU-ADPCM decoding, the Gaussian interpolation, the ADSR envelope and the volume sweeps, the noise generator,
//! pitch modulation, each voice's state, and the reverb's algorithm with its work area as a parameter. Written from
//! psx-spx's SPU chapter for Diamond (Diamond_SPU.md) and moved here for Sapphire (Sapphire_Plan.md §9.3); its record
//! is BlackMoon_SPU.md.
//!
//! Nothing here keeps time or knows a register's address: each core owns its register map, its mixing and routing,
//! its transfers, its external inputs and its output rate, and calls a voice once a sample. Sound RAM is the core's,
//! lent as a byte slice whose length is a power of two; addresses wrap within it.

#![no_std]
// The shifts are written as psx-spx writes them, the arithmetic moved unchanged from Diamond.
#![allow(clippy::precedence)]

#[cfg(test)]
extern crate std;

pub mod envelope;
pub mod noise;
pub mod reverb;
pub mod tables;
pub mod voice;

/// What sees each access the voices and the reverb make to sound RAM: the core's IRQ address comparison.
pub trait Touch {
    /// A halfword of sound RAM at byte `address` was read or written.
    fn touch(&mut self, address: u32);
}

/// No IRQ address watched.
impl Touch for () {
    #[inline]
    fn touch(&mut self, _: u32) {}
}

/// The byte mask of sound RAM lent as `ram`, whose length is a power of two.
#[inline]
pub(crate) fn mask(ram: &[u8]) -> usize {
    debug_assert!(ram.len().is_power_of_two());
    ram.len() - 1
}
