# MoonRT (Nintendo Entertainment System)

Native Rust NES emulation core (`moonrt`, `cdylib` + `rlib`), shipping on Core ABI v1.

- **Hardware covered:** NTSC NES (`2A03`/`2C02`), PAL NES (`2A07`/`2C07`), and Dendy (`6527P`/`6538`); cycle-exact 6502 bus with OAM/DMC DMA halts, per-dot 2C02 timing and sprite-0 hit, 5-channel APU with non-linear DAC, 16 iNES/NES 2.0 mappers, `nes.ppu` raw pre-palette output, and debugger hooks (using `emusen-6502` for disassembly).
- **Hardware manuals:** [`Moon_Hardware.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Moon%20-%20NES/Moon_Hardware.md), [`Moon_PAL.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Moon%20-%20NES/Moon_PAL.md).
- **Naming:** Inner Senshi codename — see [`Overview_and_Architecture.md#core-and-component-naming-scheme`](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).
