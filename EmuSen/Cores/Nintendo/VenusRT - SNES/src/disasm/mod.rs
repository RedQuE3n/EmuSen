//! The disassemblers of the four instruction sets a cartridge can run, each written from its document: the 65C816
//! from WDC's datasheet (the S-CPU and the SA-1), the SPC700, the GSU and the µPD77C25 from fullsnes. They decode
//! bytes read through a closure and know nothing of the machine. See VenusRT_Native.md §36.

pub mod gsu;
pub mod spc700;
pub mod upd77c25;
pub mod w65816;

pub use emusen_native::core::{Instruction, Reference};

/// `count` bytes from `at`, read through `read`, as `step` moves the address.
pub(crate) fn bytes(read: &dyn Fn(u32) -> u8, at: u32, count: u32, step: impl Fn(u32, u32) -> u32) -> Vec<u8> {
    (0..count).map(|i| read(step(at, i))).collect()
}
