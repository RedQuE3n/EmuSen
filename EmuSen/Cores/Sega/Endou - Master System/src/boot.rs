//! What a console runs before its game (Endou_OpenBoot.md, Jadeite_OpenBoot.md): nothing on the Mark III and the
//! SG-1000, which have no BIOS; on the Master Systems, the Japanese one among them, and the Game Gear, Endou's open boot
//! program for the console, built at compile time from `src/boot/`, or the player's own image of that console's BIOS
//! or boot ROM where the player chooses it (decided 2026-10-19).

/// The open boot program's 8 KiB image, assembled by `build.rs`: the export Master Systems', and the Japanese Master
/// System's, which checks no header (Endou_Native.md §16.4).
pub const OPEN_BOOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/open_boot.bin"));
pub const OPEN_BOOT_JAPAN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/open_boot_japan.bin"));

/// The Game Gear's open boot program, 1 KiB (Jadeite_OpenBoot.md §3).
pub const OPEN_BOOT_GG: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/open_boot_gg.bin"));

/// The open boot program a console runs.
pub fn open_boot(model: crate::model::Model) -> &'static [u8] {
    if model.is_game_gear() {
        OPEN_BOOT_GG
    } else if model.checks_export_header() {
        OPEN_BOOT
    } else {
        OPEN_BOOT_JAPAN
    }
}

include!(concat!(env!("OUT_DIR"), "/open_boot_symbols.rs"));

/// The size of a BIOS image Endou maps: the consoles' 8 KiB ROM [R Sega's Mark III manual, "Memory enables"]. Larger
/// images, which carry a game, are not mapped at this stage (Endou_OpenBoot.md §5).
pub const BIOS_SIZE: usize = 0x2000;

/// The Game Gear's boot ROM, 1 KiB at $0000-$03FF [R MacDonald's `smstech` §4].
pub const GG_BOOT_SIZE: usize = 0x400;

/// The size of the image a console's BIOS slot takes.
pub fn bios_size(model: crate::model::Model) -> usize {
    if model.is_game_gear() { GG_BOOT_SIZE } else { BIOS_SIZE }
}

/// Port $3E at reset with a BIOS: the BIOS, the work RAM and the I/O chip on, every slot off (Endou_OpenBoot.md §2, U1).
pub const RESET_MEMORY_CONTROL: u8 = 0xE3;

/// The program a machine starts with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Boot {
    /// No BIOS: the cartridge from reset, port $3E as a BIOS leaves it for a cartridge.
    None,
    /// Endou's open boot program.
    Open,
    /// The player's own image.
    Player,
}

impl Boot {
    pub fn byte(self) -> u8 {
        match self {
            Boot::None => 0,
            Boot::Open => 1,
            Boot::Player => 2,
        }
    }
}
