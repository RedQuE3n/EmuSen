//! Sapphire, the Sony PlayStation 2 core in Rust, on the core ABI v1. Written from hardware documents under
//! Sapphire_Plan.md §1.4's clean-room protocol; the build record is Sapphire_Native.md.
//!
//! Stage S1: the image is read (`media`): an EE program as an ELF, or a disc whose SYSTEM.CNF names a BOOT2 program;
//! `state` is the format SPH2, and `v1` the core ABI's glue. Since S24 the machine (`machine`) is the whole console. Stage S6: `ee`, the EE core's integer
//! interpreter and its timing model; S7: COP0, the TLB, the caches, the physical memory map, the counters and the
//! breakpoints; S8: the multimedia instructions; S9: COP1, on `float`, the PlayStation 2's single-precision format in
//! software; run by the project's programs and not yet by the machine (S10 loads programs). S11: the DMAC (`dmac`),
//! the INTC (`intc`), the EE timers (`timer`) and a blanking stand-in, replaced by the PCRTC at S20, on one clock
//! (`clock`), with the core in `board::System`. S12: the vector units (`vu`), VU0 in macro mode through COP2, micro
//! mode an interface for S13.
//! S21: the IOP (`iop`), `blackmoon-r3000` on its own bus, a unit of the board.
//! S16: the GS (`gs`), its registers, its local memory in every format's layout and its transfers, on the board.
//! S13: micro mode for both units (`vu::pipe`), with VU1's EFU (`vu::efu`).
//! S23: the CDVD drive controller (`cdvd`) at its register interface, on `emusen-disc`'s CDs and DVDs.
//! S20: the PCRTC (`pcrtc`), the video timing in place of S11's blanking stand-in, and the picture.
//! S22: the SIF (`sif`), its registers and the FIFOs joining the two DMACs' SIF channels.
//! S17: the GS draws every primitive, flat and Gouraud, with scissoring, FBMSK and the depth test.
//! S14: VIF0 and VIF1 (`vif`) on the DMAC's channels 0 and 1, writing the VUs' memories and starting micro mode.
//! S18: texture mapping (`gs::texture`: the CLUT buffer, LOD, mipmaps, filters, wrap modes, TEXA) and fog.
//! S25: SIO2 (`sio2`), the DualShock 2, the memory card and the multitap behind it, on the IOP's bus.
//! S15: the GIF (`gif`), its three paths, their arbitration and PATH3's masking, driving the GS's host interface.
//! S26: the SPU2 (`spu2`), two cores of `blackmoon-spu`'s voices on the IOP's bus and its DMA channels 4 and 7.
//! S27: the IPU (`ipu`), MPEG-2 macroblock decoding as H.262 defines it and its colour conversion, on channels 3 and 4.
//! S24: the console assembled (`machine`): the player's BIOS in the boot ROM for both processors, the IOP reset by the
//! SIF, the drive joined to the IOP's bus, the EEPROM a console medium, the pads, the picture and the sound handed on,
//! and the hardware's own reports of what no unit answers (`reports`).
//! S30: every unit's state in `state`'s format SPH2, written beside each unit and composed by `board::System`.
//! S31: the debugger (`debugger`), four processors over the core ABI's debug interface, with probes beside the IOP and
//! the VUs and the EE observed around its untouched step (`ee::observe`).
//! S33: idle skipping (`idle`), the EE's waits passed up to the board's next event with the IOP brought along.

pub mod bios;
pub mod blocks;
pub mod board;
pub mod cdvd;
pub mod clock;
pub mod debugger;
pub mod dmac;
pub mod ee;
pub mod float;
#[cfg(feature = "lanes")]
pub mod float_lanes;
pub mod gif;
pub mod gs;
pub mod idle;
pub mod intc;
pub mod iop;
pub mod ipu;
pub mod machine;
pub mod mch;
pub mod media;
pub mod pcrtc;
pub mod reports;
pub mod sif;
pub mod sio2;
pub mod spu2;
pub mod state;
pub mod timer;
pub mod vu;
pub mod v1;
pub mod vif;
