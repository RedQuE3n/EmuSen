# Beryl-HW (Shared Sega Hardware Crates)

Standalone Rust crates implementing processors, peripheral chips, and shared hardware tables used across EmuSen's Sega console cores.

## Crates

- `beryl-m68k` — Motorola 68000 CPU (Genesis main CPU, Sega CD sub-CPU) with disassembler and optional `asm` feature.
- `beryl-z80` — Zilog Z80 CPU (Genesis sound CPU, Master System / Game Gear main CPU) with disassembler and optional `asm` feature.
- `beryl-sh2` — Hitachi SH-2 (`SH7604`) CPU contract stub (`BUILT = false`) for the 32X and future Saturn cores.
- `beryl-asm` — Zero-dependency assembler front end shared by `beryl-m68k` and `beryl-z80` behind their `asm` features (build/dev tool only).
- `beryl-sms-vdp` — Master System VDP family (`315-5124`, `315-5246`, `315-5378`), `TMS9918A`, and Genesis mode 4 line composition.
- `beryl-sms-colours` — Zero-dependency RGBA colour tables for Master System, Game Gear, and TMS9918A palettes.
- `beryl-sn76489` — TI SN76489 / Sega integrated PSG with Game Gear stereo routing.
- `beryl-fm-tables` — Zero-dependency 256-entry Yamaha FM `LOGSIN` and `EXP` operator ROM tables shared by YM2413 and YM2612.
- `beryl-font` — Zero-dependency 47-glyph 5×7 font used at build time by Sega open firmware (`build.rs` only).

## Documentation

- [Beryl-HW Architecture & Contracts](../../../../Docs/Man%20pages/Hardware/Sega/Beryl-HW/README.md)
- [Beryl CPUs and Assemblers](../../../../Docs/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_Cpus.md)
- [Beryl Shared Devices and Data Crates](../../../../Docs/Man%20pages/Hardware/Sega/Beryl-HW/Beryl_Devices.md)
- [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme)
