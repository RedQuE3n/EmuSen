# VenusRT (Super Nintendo Entertainment System)

Native Rust SNES / Super Famicom emulation core (`venusrt`, `cdylib` + `rlib`), shipping on Core ABI v1.

- **Hardware covered:** NTSC and PAL Ricoh 5A22 (WDC 65816) with DMA/HDMA and math unit, LoROM/HiROM/ExHiROM cartridges, S-PPU (Modes 0–7, EXTBG, windows, color math, hi-res/interlace), S-SMP/SPC700 and S-DSP audio, enhancement coprocessors (SA-1, SuperFX/GSU, OBC1, NEC µPD77C25/µPD96050 LLE), and built-in open HLE replacements for DSP-1/1B, DSP-2, DSP-4, and ST010.
- **Hardware manuals:** [`Venus_Hardware.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_Hardware.md), [`Venus_Coprocessors.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_Coprocessors.md), [`Venus_DspHle.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Venus%20-%20SNES/Venus_DspHle.md).
- **Naming:** Inner Senshi codename — see [`Overview_and_Architecture.md#core-and-component-naming-scheme`](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).
