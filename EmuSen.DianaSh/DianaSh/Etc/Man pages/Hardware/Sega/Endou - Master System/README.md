# Endou (Sega Master System, with the Mark III and the SG-1000's cartridges)

The Master System core (`EmuSen/Cores/Sega/Endou - Master System/`, crate `endou` once built), to be written from
hardware documents under a clean-room protocol, on the core ABI v1 from its first commit. *Decided 2026-10-19:* the
Master System is the next new console, ahead of the PlayStation 2; building starts on the tester's go. *Decided
2026-10-19:* the plan's questions as recommended, with no real hardware used for any dispute; the go for stage 1.

- [`Endou_Plan.md`](Endou_Plan.md) — the plan, written 2026-10-19 before any emulation: the machines in scope and what
  is in or out and why, the Game Gear as a follow-on and the forms it could take, what comes from Beryl and what Beryl
  gains (the VDP family as a shared crate, its line composition shared with Nephrite's mode 4), what a frontend gets,
  the design decisions (the master clock, the VDP's granularity, sound, the open boot program, cartridge boards, the
  state, settings, the speed budget), the oracles by component, the stages with their gates and sizes, and the
  questions, decided 2026-10-19.
- [`Endou_Native.md`](Endou_Native.md) — the build record, from stage 1 (2026-10-19): the stub crate `endou` on the
  core ABI v1, marked in development; the systems `sms` and `sg1000` as data; `SyntheticSmsRom`; the kit's gate; the
  five references' costs; the seeded faults; and the questions before stage 2.
- [`Endou_References.md`](Endou_References.md) — the documents, test programs, references and referees with their URL,
  version, licence and SHA-256, as stage 1 fetched and pinned them, and the corrections to what stage 0 recalled.
- [`Endou_Disputes.md`](Endou_Disputes.md) — the disputes log, opened with D-0 (the PSG's noise register by console),
  D-1 (the Master System's colour levels, open) and D-2 (the YM2413's operator ROMs, open).

The CPU and the PSG are documented with their crates in [`../Beryl-HW/`](../Beryl-HW/) (Beryl Hardware). The Game
Gear's pages, when its stages start, go in [`../Jadeite - Game Gear/`](../Jadeite%20-%20Game%20Gear/).

The core was first reserved as Endymion, a name now held by the SDL crate; see `Man pages/EmuSen_Core_Naming_Scheme.md`
§3 and §10.
