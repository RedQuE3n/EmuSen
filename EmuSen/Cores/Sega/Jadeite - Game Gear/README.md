# Jadeite (Sega Game Gear)

No standalone crate lives in this directory. Sega Game Gear emulation is implemented inside [`endou`](../Endou%20-%20Master%20System/README.md) (`src/gg.rs`, `src/ggio.rs`, `src/eeprom.rs`, and `src/boot/gg.s`), sharing `beryl-z80`, `beryl-sms-vdp` (`Variant::GG`, `315-5378`), `beryl-sms-colours`, and `beryl-sn76489` with the Master System core. This folder preserves the Game Gear's place under the Sega cores directory layout.

## Documentation

- [Endou Game Gear Hardware Reference](../../../../Docs/Man%20pages/Hardware/Sega/Endou%20-%20Master%20System/Endou_GameGear.md)
- [Endou Master System Hardware Reference](../../../../Docs/Man%20pages/Hardware/Sega/Endou%20-%20Master%20System/Endou_Hardware.md)
- [Core and Component Naming Scheme](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme)
