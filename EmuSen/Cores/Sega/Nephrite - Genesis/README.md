# Nephrite (Sega Genesis / Mega Drive, Sega CD, and 32X)

Rust Core ABI v1 emulation core (`nephrite`) for the Sega Genesis / Mega Drive (`md`), Sega CD / Mega-CD (`mcd`), and Sega 32X (`32x`, stub marked in-development).

## Architecture and Implemented Hardware

- **Shared Chip Crates (`Beryl-HW`)**: `beryl-m68k` (Genesis main CPU and Sega CD sub-CPU), `beryl-z80` (Genesis sound CPU), `beryl-sn76489` (Genesis PSG), `beryl-fm-tables` (YM2612 operator ROMs), `beryl-sms-vdp` (Genesis mode 4 composition), and `beryl-sh2` (32X SH-2 stub, `BUILT = false`).
- **Genesis / Mega Drive (`src/genesis/`)**: Master-clock-synchronized 68000 and Z80 buses (refresh waits, Z80 bank window, TMSS, locked `TAS`), slot-timed VDP with 4-word FIFO/DMA and span-based Mode 5 + Mode 4 rendering, YM2612/YM3438 FM synthesis with measured LFO vibrato and Model 1/Model 2 output circuits, SN76489 PSG, Sega SSF mapper, 21 I²C EEPROM boards, Sonic & Knuckles lock-on, 3/6-button pads, Team Player, 4 Way Play, and J-Cart.
- **Sega CD / Mega-CD (`src/mcd/`, feature `mcd`)**: Isolated behind `src/interface.rs` (`Attachment` trait). Implements the 12.5 MHz sub-68000, gate array, 2M/1M Word RAM, rotation/scaling graphics ASIC, Sanyo LC8951 CDC, CDD drive controller + CD-DA fader/de-emphasis (`emusen-disc`), Ricoh RF5C164 8-channel PCM, Mode 1 (`mcd_beneath`) and Backup RAM Cartridge support, plus a compile-time assembled 128 KiB open Sega CD BIOS (`src/mcd/bios/`, `build.rs`) with `_CDBIOS` and `_BURAM` backup RAM filesystem.
- **Sega 32X (`src/s32x/`, feature `s32x`)**: Stub implementing `Attachment` with reserved memory spaces (`SDRAM`, `FRAMEBUFFER`, `PALETTE`), firmware metadata, and `development: true` until `beryl-sh2` and 32X hardware are implemented.
- **Core Services**: Versioned `"NPHR"` save states (`src/state.rs`), debugger integration (`src/genesis/debugger.rs`), Game Genie ROM patches, and boundary verification tests (`src/boundary.rs`).

## Documentation

- [Nephrite Genesis / Mega Drive Hardware Reference](../../../../Docs/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_Genesis.md)
- [Nephrite Sega CD / Mega-CD Reference](../../../../Docs/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_SegaCD.md)
- [Nephrite Sega 32X Stub Reference](../../../../Docs/Man%20pages/Hardware/Sega/Nephrite%20-%20Genesis/Nephrite_32X.md)
- [Beryl-HW Shared Chip Crates](../../../../Docs/Man%20pages/Hardware/Sega/Beryl-HW/README.md)
- [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme)
