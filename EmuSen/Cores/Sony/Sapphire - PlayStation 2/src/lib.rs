//! Sapphire, the Sony PlayStation 2 core in Rust, on the core ABI v1. Written from hardware documents under
//! Sapphire_Plan.md §1.4's clean-room protocol; the build record is Sapphire_Native.md.
//!
//! Stage S1: a stub. The image is read (`media`): an EE program as an ELF, or a disc whose SYSTEM.CNF names a BOOT2
//! program; the machine (`machine`) holds the memories, a frame counter, a black picture and silence, and nothing is
//! emulated. `state` is the format SPH2, and `v1` the core ABI's glue. Stage S6: `ee`, the EE core's integer
//! interpreter and its timing model; S7: COP0, the TLB, the caches, the physical memory map, the counters and the
//! breakpoints; S8: the multimedia instructions; S9: COP1, on `float`, the PlayStation 2's single-precision format in
//! software; run by the project's programs and not yet by the machine (S10 loads programs). S11: the DMAC (`dmac`),
//! the INTC (`intc`), the EE timers (`timer`) and the blanking stand-in (`blank`) on one clock (`clock`), with the
//! core in `board::System`.

pub mod bios;
pub mod blank;
pub mod board;
pub mod clock;
pub mod dmac;
pub mod ee;
pub mod float;
pub mod intc;
pub mod machine;
pub mod media;
pub mod state;
pub mod timer;
pub mod vu;
pub mod v1;
