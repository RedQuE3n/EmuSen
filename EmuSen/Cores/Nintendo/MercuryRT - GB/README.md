# MercuryRT (Game Boy / Game Boy Color)

Native Rust Game Boy and Game Boy Color emulation core (`mercuryrt`, `cdylib` + `rlib`), shipping on Core ABI v1.

- **Hardware covered:** DMG, CGB colour mode, and DMG-on-CGB compatibility mode (automatic CGB boot compatibility palettes); M-cycle Sharp SM83 CPU, 160×144 LCD/PPU with STAT edge detection, 4-channel DIV-clocked stereo APU, OAM DMA and CGB HDMA/double speed, MBC1 (including MBC1M)/MBC2/MBC3 (RTC)/MBC5 (rumble) boards, and debugger hooks (using `silvermillennium-sm83` for disassembly).
- **Hardware manuals:** [`Mercury_Hardware.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_Hardware.md), [`Mercury_CgbAndModels.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/Mercury%20-%20GB-GBC/Mercury_CgbAndModels.md).
- **Naming:** Inner Senshi codename — see [`Overview_and_Architecture.md#core-and-component-naming-scheme`](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).
