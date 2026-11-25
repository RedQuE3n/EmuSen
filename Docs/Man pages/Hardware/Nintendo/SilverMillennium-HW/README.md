# SilverMillennium Hardware (`EmuSen/Cores/Nintendo/SilverMillennium-HW`)

Shared chip-level Rust crates used by more than one Nintendo core or shared between a Nintendo core and `Platform/systems`. Each chip lives in its own crate prefixed `silvermillennium-<chip>` so it can be built and tested standalone.

For EmuSen's Sailor Moon naming convention (Silver Millennium for Nintendo shared hardware, alongside Beryl Hardware for Sega and BlackMoon Hardware for Sony), see [`Overview_and_Architecture.md#core-and-component-naming-scheme`](../../../../EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).

---

## Live Crates

### `silvermillennium-sm83`
- **Path:** `EmuSen/Cores/Nintendo/SilverMillennium-HW/silvermillennium-sm83` (`silvermillennium_sm83`, v0.1.0, `#![no_std]`, zero dependencies).
- **Role:** Single-owner SM83 (Game Boy / Game Boy Color) instruction table and disassembler, shared by `MercuryRT`'s `DEBUG_DISASSEMBLE` and `Platform/systems`'s Game Boy disassembler. Contains no CPU execution.
- **Public API (`src/lib.rs`):**
  - `ILLEGAL = "???"`, `CB_LENGTH = 2`, `Reference` (`Call`, `Read`, `Write`).
  - `length_of(opcode: u8) -> usize`: Instruction byte length (`1`–`3`, `2` for `0xCB` prefix).
  - `mnemonic_of(opcode: u8, suffix: u8) -> String`: Uppercase mnemonic (`0xCB` prefix resolves `suffix` bit/rotate/shift operations).
  - `format_operand(opcode: u8, address: i32, low: u8, high: u8) -> String`: Formats immediate, direct, indirect, high-RAM (`$FF00+n`), signed relative jump (`address + 2 + offset`), and `RST` operands.
  - `classify(opcode: u8, operand: i32, address: i32) -> Option<(Reference, i32)>`: Static target classification for debugger cross-references.

---

## Placement Boundaries

- **Cross-manufacturer ISAs stay in `EmuSen/Cores/Shared/`:** `emusen-mips` (shared by Nintendo N64 and Sony PS1/PS2) and `emusen-6502` (MOS 6502, shared across non-Nintendo hardware) live under [`Hardware/Shared/CpuTables.md`](../../Shared/CpuTables.md).
- **Single-core code stays inside its core:** Hardware moves into `SilverMillennium-HW/` only when a second Nintendo core requires it (for example, shared Game Boy PPU/APU/MBC crates if Super Game Boy, Transfer Pak, or GBA DMG mode is built; ARM7TDMI for GBA/DS; PowerPC/Gecko/Broadway for GameCube/Wii).
