# Sapphire (Sony PlayStation 2)

The pages of Sapphire, EmuSen's PlayStation 2 core, planned 2026-10-23, its stage S1 built 2026-10-24. The core will live
in `EmuSen/Cores/Sony/Sapphire - PlayStation 2/`; the name is fixed by `EmuSen_Core_Naming_Scheme.md` §4.

- **`Sapphire_Plan.md`** — the stage 0 plan (2026-10-23): the clean-room protocol and the exposure rule carried over from
  Diamond; every unit with its source and oracle (the EE and its MMI, COP0, the FPU, VU0 and VU1, VIF, GIF, the GS,
  IPU, DMAC, INTC, timers, SIF, the IOP, SPU2, CDVD, SIO2) and what is left out; PS1 discs routed to Diamond; the
  player's BIOS first and the open replacement as a separate track; the speed budget and the recompilers; the GS's
  exact software reference and its Vulkan path; the shared crates; the stages, gates, questions and predictions.
- **`Sapphire_Native.md`** — the build record, from stage S1 (2026-10-24): the stub crate on the core ABI v1, marked in
  development (an EE program by its ELF header, a disc by `SYSTEM.CNF`'s `BOOT2`, the three BIOS entries, the `SPH2`
  state); the system `ps2` with its recogniser and the PlayStation's `BOOT2` exclusion, root files read as far as a
  DVD's volume places them; the CRT row owed while the console is offered to no player; the fixtures, the kit's gate
  and the seeded faults.
- **`Sapphire_Disputes.md`** — the disputes log, opened with D-0, the system clocks.

The shared Sony hardware Sapphire uses (the IOP's R3000A) is in `../BlackMoon-HW/`; the shared crates it extends
(`emusen-mips`, `emusen-disc`, `emusen-gpu`) are in `../../Shared/`. See `Man pages/Hardware/README.md` for how this
documentation set is organised.
