# SilverMillennium Hardware

Shared chip-level Rust crates used across Nintendo cores and `Platform/systems`. Not a console core: each chip lives in its own standalone crate.

- `silvermillennium-sm83` — `#![no_std]` SM83 instruction table and disassembler (no execution), shared by `MercuryRT`'s `DEBUG_DISASSEMBLE` and `Platform/systems`'s Game Boy disassembler.

Named for the Silver Millennium (the Moon Kingdom of the Sailor Guardians), alongside Beryl Hardware (Sega) and BlackMoon Hardware (Sony) — see [`Overview_and_Architecture.md#core-and-component-naming-scheme`](../../../../Docs/EmuSen%20Manual/Overview_and_Architecture.md#core-and-component-naming-scheme).
For full crate contracts and placement rules, see [`Docs/Man pages/Hardware/Nintendo/SilverMillennium-HW/README.md`](../../../../Docs/Man%20pages/Hardware/Nintendo/SilverMillennium-HW/README.md).
