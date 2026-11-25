# SilverMillennium Hardware (the shared hardware of the Nintendo cores)

Not a console core: the chips more than one Nintendo core uses, one crate each in
`EmuSen/Cores/Nintendo/SilverMillennium-HW/`, so that each can be used alone. *Decided 2026-10-23 by the tester:
crates shared between the Nintendo cores live in SilverMillennium Hardware, prefixed `silvermillennium-<chip>`, as
Beryl Hardware holds the Sega cores' and BlackMoon Hardware the Sony cores' (`EmuSen_Core_Naming_Scheme.md` §2).*

## The crates

- `silvermillennium-sm83`: the SM83's instruction table and disassembler, `no_std`, with no execution and no
  dependencies. MercuryRT's `DEBUG_DISASSEMBLE` and the Game Boy pack in `emusen-systems` both use it, so the processor
  has one decoder (`Mercury_Debug.md` §4; `EmuSen_Backend.md` §17.2 D10 and Q3). It was `Cores/Shared/emusen-sm83`,
  built in B4, and moved here unchanged on 2026-10-23. Only its name and folder changed, so its tests and their counts
  are those of B4's record.

## What stays elsewhere

- `emusen-mips` stays in `Cores/Shared/` because the Sony cores share it (`../../Shared/EmuSen_Mips.md`).
- `emusen-6502` stays in `Cores/Shared/`. The 6502 is MOS Technology's processor, and its likelier second user is
  another manufacturer's machine. Today MoonRT and the NES pack use it.
- Code that one core alone uses stays in that core. A crate moves here when a second Nintendo core needs it.

## Expected next occupants

These are expectations, not decisions. Each moves here only when its second user is built:
- the Game Boy's cartridge boards and its picture and sound units, if the Super Game Boy (VenusRT), the Transfer Pak
  (MarsRT) or the Game Boy Advance's Game Boy mode (Jupiter) is built;
- the ARM7TDMI, for Jupiter and Luna;
- the GameCube's and the Wii's processors, for Uranus and Neptune.
