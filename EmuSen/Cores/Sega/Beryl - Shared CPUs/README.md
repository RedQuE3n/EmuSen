# Beryl (the shared CPUs of the Sega cores)

Not a console: one Rust crate per processor, so that each can be used alone.

- `beryl-m68k` — the Motorola 68000: Nephrite's Genesis and Sega CD, and later any core with a 68000.
- `beryl-z80` — the Zilog Z80: Nephrite's sound CPU, and later Endou (Master System) and Jadeite (Game Gear).
- `beryl-sh2` — the Hitachi SH-2 (SH7604): Nephrite's 32X, and later Zoisite (Saturn).

Named for Queen Beryl, who commands the Shitennou (`EmuSen_Core_Naming_Scheme.md` §3). The interface and each crate's
record are in `Man pages/Hardware/Sega/Beryl - Shared CPUs/`.
