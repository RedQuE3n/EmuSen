# Mercury (Game Boy / Game Boy Color)

Started 2026-08-04. **One core covers both**, DMG first with colour as an additive mode later — the
"two separate cores?" question this file used to leave open is settled in `Mercury_Core.md` §1.

Built so far: the full SM83 instruction set (unprefixed and `$CB`), interrupts with the EI delay and
the HALT bug, the DIV/TIMA timer, the joypad, OAM DMA, five cartridge boards (no-MBC, MBC1, MBC2,
MBC3 with RTC, MBC5) and save states.

Not built: the PPU, the APU, and an `IDebugTarget` — which is why this core is deliberately not
registered in `CoreCatalog`/`CoreFactory` yet. See `Man pages/Hardware/Nintendo/Mercury - GB-GBC/`.
