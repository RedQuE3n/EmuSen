# Diamond (Sony PlayStation)

The PlayStation core, written in Rust from hardware documents under a clean-room protocol, on the core ABI v1 from
its first commit (`EmuSen/Cores/Sony/Diamond - PlayStation/`, crate `diamond`), and marked in development until its
gate. At stage 1 (2026-10-11) it recognises a PS-EXE or a disc and shows a black picture; nothing is emulated yet. Its
processors are the BlackMoon crates (`../BlackMoon-HW/README.md`).

- [`Diamond_Plan.md`](Diamond_Plan.md) — the plan, written 2026-10-10 before any emulation: the machine in scope and
  how accurate each part must be (the R3000A with its cache and scratchpad, the GTE exact, DMA, timers, interrupts, a
  cycle-faithful software GPU, the SPU, the CD-ROM controller with CD-DA and XA, the MDEC, pads, cards and the
  multitap, PAL and NTSC), the disc formats, who may read what, the sources and where they are thin, the oracles by
  component, the black-box references and the FPGA referee, what a frontend gets, the open BIOS, the speed budget, the
  enhancements decided on 2026-10-10 (precise geometry and the rest) and their separable architecture, the stages with
  their gates, the shared MIPS and disc crates, and the questions for the tester.
- [`Diamond_Native.md`](Diamond_Native.md) — the build record, from stage 1 (2026-10-11): the stub crate on the core
  ABI v1 (what an image is, read from its bytes; the provisional rates; the settings, firmware, spaces and state), the
  system pack `psx` and which facts it owns, the BlackMoon crates, the registration in development, WiseMan's PS-EXE
  and synthetic disc builders (ISO 9660, raw sectors with ECMA-130's EDC and parity, a CD-DA track, a cue sheet), and
  the gate: the kit's C1–C15 on a synthetic program and disc, and the r3000 replay controls.
- `Diamond_Disputes.md` — the disputes log, created with its first entry (D-0, the NTSC frame rate).

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the naming scheme, and `Man pages/Hardware/README.md` for how this
documentation set is organized.
