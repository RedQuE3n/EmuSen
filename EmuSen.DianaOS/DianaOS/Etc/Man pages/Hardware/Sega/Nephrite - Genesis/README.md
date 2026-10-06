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

The CPUs and the PSG are documented with their crates in [`../Beryl-HW/`](../Beryl-HW/) (Beryl Hardware).
