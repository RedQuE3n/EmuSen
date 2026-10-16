//! Nephrite's open Sega CD BIOS, assembled by build.rs from `bios/*.s` (Nephrite_OpenBios.md): the boot ROM a disc
//! runs on when the player gives none.

/// The 128 KiB boot ROM.
pub const OPEN_BIOS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/open_bios.bin"));
