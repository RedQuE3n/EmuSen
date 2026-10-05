//! Nephrite, the Sega Genesis / Mega Drive core in Rust, with the Sega CD and the 32X as attachments of the same
//! machine, on the core ABI v1. Written from hardware documents under Nephrite_Plan.md §1's clean-room protocol; the
//! build record is Nephrite_Native.md.
//!
//! Stage 0: the image is recognised and reported, the memories exist, the picture is blank. The CPUs come from the
//! Beryl crates (`beryl-m68k`, `beryl-z80`, `beryl-sh2`) as they are built.

pub mod machine;
pub mod media;
pub mod state;
pub mod v1;
