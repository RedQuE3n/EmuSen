# Beryl Hardware (the shared hardware of the Sega cores)

Not a console: one Rust crate per chip that more than one Sega core can use, so that each can be used alone. The
first were the processors.

- `beryl-m68k` — the Motorola 68000: Nephrite's Genesis and Sega CD, and later any core with a 68000. Its assembler, a
  development tool behind the `asm` feature, builds the open firmware at compile time (`Beryl_M68k.md` §8).
- `beryl-z80` — the Zilog Z80: Nephrite's sound CPU and Endou's (Master System) main CPU, later Jadeite's (Game Gear).
  Its assembler, behind the `asm` feature, builds Endou's test programs (`Beryl_Z80.md` §8).
- `beryl-sh2` — the Hitachi SH-2 (SH7604): Nephrite's 32X, and later Zoisite (Saturn).
- `beryl-sn76489` — the Texas Instruments SN76489 and Sega's copy of it in the VDP: the Genesis's PSG, and later
  Endou's and Jadeite's (with the Game Gear's stereo register).
- `beryl-sms-vdp` — the Master System's VDP family (315-5124, 315-5246, 315-5378) in mode 4: Endou's picture, and at
  its stage 4 the TMS9918A's and the Genesis's mode 4 composition (`Beryl_SmsVdp.md`).
- `beryl-sms-colours` — the Master System VDP family's colours as RGBA, which Endou's picture and the `sms` pack's
  palette view both read; no dependencies (`Beryl_SmsVdp.md` §10).
- `beryl-fm-tables` — the log-sine and exponent ROMs of the Yamaha FM operator: Endou's YM2413 and Nephrite's YM2612
  read one copy (`Beryl_FmTables.md`).
- `beryl-asm` — the front half the processors' assemblers share (expressions, labels, constants, conditionals,
  includes); a development tool, linked by no core's library.
- `beryl-font` — the open firmware's own five-by-seven font: Nephrite's open Sega CD BIOS and Endou's open boot program,
  each `build.rs` making its console's tiles from the glyphs (`Beryl_Font.md`).

Named for Queen Beryl, who commands the Shitennou (`EmuSen_Core_Naming_Scheme.md` §3). The interface and each crate's
record are in `Man pages/Hardware/Sega/Beryl-HW/`.
