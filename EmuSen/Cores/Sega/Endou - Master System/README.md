# Endou (Sega Master System, Mark III, SG-1000, and Game Gear)

Rust Core ABI v1 emulation core (`endou`) for the Sega Master System (`sms`), Mark III, SG-1000 (`sg1000`), and Game Gear (`gg`, currently flagged in-development in the system descriptor).

## Features and Architecture

- **CPUs & Devices (`Beryl-HW`)**: `beryl-z80` main CPU, `beryl-sms-vdp` (`315-5124`, `315-5246`, `315-5378`, `TMS9918A`), `beryl-sms-colours`, `beryl-sn76489` (Sega integrated and discrete variants with Game Gear stereo), and `beryl-fm-tables`.
- **Console Models (`src/model.rs`)**: Master System I, Master System II, Japanese Master System (built-in YM2413), Mark III, Mark III + FM Sound Unit, SG-1000, and export/Japanese Game Gear.
- **Audio (`src/sound.rs`, `src/opll.rs`)**: SN76489 PSG and YM2413 (OPLL) FM synthesis with port `$F2` routing, resampled to 48 kHz stereo via `emusen-native`'s step synthesizer.
- **Cartridges (`src/board.rs`, `src/cart.rs`, `src/eeprom.rs`)**: Plain, SG-1000 RAM, Sega, Codemasters, and Game Gear `93C46B` serial EEPROM boards, plus battery saves and Game Genie patches.
- **Game Gear (`src/gg.rs`, `src/ggio.rs`)**: Native 160×144 12-bit LCD window, Master System compatibility 160×146 subpixel/vertical LCD shrink, ports `$00`–`$06`, stereo routing, and TH H-counter latch.
- **Open Boot ROMs (`src/boot/`, `build.rs`)**: Compile-time assembled 8 KiB export/Japanese Master System and 1 KiB Game Gear open boot ROMs (using `beryl-z80` `asm` and `beryl-font`), with optional per-console player BIOS override.
- **Save States & Debugging (`src/state.rs`, `src/debug.rs`)**: Versioned `"ENDO"` save states and Z80 debugger integration (memory spaces, watches, breakpoints, disassembly).

## Documentation

- [Endou Hardware Reference (Master System, Mark III, SG-1000)](../../../../Docs/Man%20pages/Hardware/Sega/Endou%20-%20Master%20System/Endou_Hardware.md)
- [Endou Game Gear Hardware Reference](../../../../Docs/Man%20pages/Hardware/Sega/Endou%20-%20Master%20System/Endou_GameGear.md)
- [Beryl-HW Shared Chip Crates](../../../../Docs/Man%20pages/Hardware/Sega/Beryl-HW/README.md)
- [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme)
