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
  five references' costs; the seeded faults; and the questions before stage 2, decided 2026-10-19. Stage 2 (§9,
  2026-10-19): the bus, the boards and their table, the I/O chip and the models, the scheduler, the state's version 2,
  the test programs, ZEXDOC and ZEXALL on the machine, the library survey, the referees, the gate and the seeded faults.
  Stage 3 (§10, 2026-10-19): the VDP's mode 4 wired to the machine, FluBBa's VDP test passing whole, the pictures
  against the references, the state's version 3, the kit, the seeded faults, and the questions before stage 4. Stage 4
  (§11, 2026-10-19): the TMS9918's modes, the TMS9918A and the SG-1000 as a modelled console with the Mark III as its
  second model, the 315-5124's boosted blue, Nephrite moved onto the shared composition, the library's SG-1000 games
  against the references, SMS_MiSTer under GHDL, the seeded faults, and the questions before stage 5, decided 2026-10-19.
  Stage 5 (§12, 2026-10-19): the PSG mixed and resampled in both variants, port `$F2` with no FM unit, the voices,
  sound test programs and library games against the references' audio, no allocation on the audio path, the state's
  version 5, the seeded faults, SMS_MiSTer's dispute step weighed and left, and the questions before stage 6.
- [`Endou_References.md`](Endou_References.md) — the documents, test programs, references and referees with their URL,
  version, licence and SHA-256, as stage 1 fetched and pinned them, the corrections to what stage 0 recalled, and GHDL
  (stage 4).
- [`Endou_Disputes.md`](Endou_Disputes.md) — the disputes log, opened with D-0 (the PSG's noise register by console),
  D-1 (the Master System's colour levels, open) and D-2 (the YM2413's operator ROMs, open); stage 2 added D-3 (the
  Mark III's TH), D-4 (port `$3E` and the undriven reads) and D-5 (Codemasters' slots), each argued; stage 3 added D-6
  (the 315-5124's zoom), D-7 (the fine scroll's first pixels), D-8 (the access slots, open) and D-9 (the line's
  timing, measured by FluBBa's test); stage 4 settled D-1 by the tester's decision and added D-10 (four sprites a line
  in the TMS9918's modes), D-11 (the TMS9918A's colours) and D-12 (its 4K mode), each argued; stage 5 measured D-0 on
  the output and added D-13 (the audio control port with no FM unit), argued.

The CPU, its assembler, the PSG and the VDP are documented with their crates in [`../Beryl-HW/`](../Beryl-HW/) (Beryl Hardware). The Game
Gear's pages, when its stages start, go in [`../Jadeite - Game Gear/`](../Jadeite%20-%20Game%20Gear/).

The core was first reserved as Endymion, a name now held by the SDL crate; see `Man pages/EmuSen_Core_Naming_Scheme.md`
§3 and §10.
