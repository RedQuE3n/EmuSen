# Nephrite (Sega Genesis / Mega Drive, with the Sega CD and the 32X)

The Genesis core, with the Sega CD and the 32X as attachments of one machine (`EmuSen/Cores/Sega/Nephrite - Genesis/`,
crate `nephrite`). Written from hardware documents under a clean-room protocol, on the core ABI v1 from its first commit.

- [`Nephrite_Plan.md`](Nephrite_Plan.md) — the plan, written 2026-10-04 before any emulation: the hardware in scope,
  who may read what, the sources and where they are thin, the oracles by component, what a frontend gets, the design
  decisions (the master clock, the VDP's granularity, the attachments' clocking, the speed budget, the open replacements
  for the Sega CD BIOS and the 32X's boot ROMs, disc images and the ABI), the stages with their effort, the gates, the
  Beryl CPU crates and the questions for the tester.
- [`Nephrite_Native.md`](Nephrite_Native.md) — the build record, from stage 0.
- `Nephrite_Disputes.md` — the disputes log, created with its first entry.
- [`Nephrite_OperatorTables.md`](Nephrite_OperatorTables.md) — the YM2612 operator's log-sine and exponent ROM tables:
  their sources, the two sampling offsets, and the open question of whether the OPN2 shares the OPL2's ROMs.
- [`Nephrite_LfoTables.md`](Nephrite_LfoTables.md) — the YM2612's vibrato: the offset the LFO adds to a channel's
  frequency number for each PMS and LFO step, measured on the board's YM3438 for every frequency number, with the LFO's
  step timing at each rate.
- [`Nephrite_Attachments.md`](Nephrite_Attachments.md) — the attachment interface, designed 2026-10-15 before its code
  and built the same day: the coupling between the Genesis and the Sega CD counted by kind, the three traits an
  attachment implements and what the Genesis offers it, the dispatch that keeps the cartridge Genesis's hot path free,
  the module layout inside the one `nephrite` crate with what its visibility holds and the two checks for what it
  cannot (separate crates decided not now), the forms weighed, the migration in eight commits with the state kept
  byte-identical, the questions for the tester as decided, and the build's record (§11), each design section followed
  by what was built where it differed.
- [`Nephrite_OpenBios.md`](Nephrite_OpenBios.md) — stage 10's plan for the open Sega CD BIOS, written 2026-10-15
  before its code: the five parts by Sega's manuals, cited by section; what the manuals leave unsaid (the function
  numbers, the sub CPU's jump table slots, the backup RAM's format, the main CPU's routines), the steps with their
  oracles, the accuracy cost and the questions for the tester.

The CPUs and the PSG are documented with their crates in [`../Beryl-HW/`](../Beryl-HW/) (Beryl Hardware).
