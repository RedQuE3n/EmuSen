//! What a console runs before its game (Endou_OpenBoot.md): nothing on the Mark III and the SG-1000, which have no BIOS;
//! on the Master System and the Master System II, Endou's open boot program, built at compile time from `src/boot/`, or
//! the player's own image of that console's BIOS where the player chooses it (decided 2026-10-19).

/// The open boot program's 8 KiB image, assembled by `build.rs`.
pub const OPEN_BOOT: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/open_boot.bin"));

include!(concat!(env!("OUT_DIR"), "/open_boot_symbols.rs"));

/// The size of a BIOS image Endou maps: the consoles' 8 KiB ROM [R Sega's Mark III manual, "Memory enables"]. Larger
/// images, which carry a game, are not mapped at this stage (Endou_OpenBoot.md §5).
pub const BIOS_SIZE: usize = 0x2000;

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
