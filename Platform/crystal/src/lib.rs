//! Crystal, the runtime beneath the frontends, in Rust. Version 1.0 of its frontend API is served by the C#
//! implementation of step B2 (`EmuSen.Crystal.Client` and `EmuSen/Crystal/`); this crate holds the API's types and
//! numbers exactly as `EmuSen.Crystal/include/emusen_crystal.h` declares them, so that step B7 implements the header
//! it was checked against. See EmuSen_Backend.md §3 and §14.

pub mod abi;
