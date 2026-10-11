# Diamond (Sony PlayStation)

The PlayStation core, written in Rust from hardware documents under a clean-room protocol, on the core ABI v1 from
its first commit (`EmuSen/Cores/Sony/Diamond - PlayStation/`, crate `diamond`), and marked in development until its
gate. At stage 1 (2026-10-11) it recognises a PS-EXE or a disc and shows a black picture; at stage 2 (the same day)
its R3000A was built and passes the r3000 single-step suite; at stage 3 the bus, its timing and the devices beside
the CPU run it, on the open kernel or a player's own BIOS, and ps1-tests' CPU, DMA and timer programs pass by their
TTY. Its processors are the BlackMoon crates (`../BlackMoon-HW/README.md`).

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
  the gate: the kit's C1–C15 on a synthetic program and disc, and the r3000 replay controls; and stage 2 (§10): the
  R3000A and the shared `emusen-mips`, 55,000 of 55,000 single-step cases with one dispute naming cases, and P1
  retired; and stage 3's machine (§11): the map, every access timed by memory control and the console's figures, the
  I-cache, the interrupts, the root counters, the seven DMA channels, the video timing on D-0's clocks, the stubs,
  PS-EXE loading, the kernel's seam, the TTY reader, the project's own programs with the references as witnesses, and
  the gate's results.
- [`Diamond_OpenKernel.md`](Diamond_OpenKernel.md) — the open BIOS kernel's first phase, written 2026-10-11 at stage
  3 under the replacement writers' rules: the image and the RAM it uses, the reset path and the hand-over at 80030000h
  with what a machine must provide (the TTY at the DUART's 1F802023h), the exception entry, the priority chains and the
  hook, the syscalls, the control blocks, heaps, events, threads, the memory, string and file calls with the rule for
  documented bugs, the call table of 576 entries with each one's status, the pinned toolchain and the reproducible
  image, the harness on blackmoon-r3000 and what it proved, the accuracy cost, the choices, the questions and the
  reading log.
- [`Diamond_References.md`](Diamond_References.md) — the oracles, documents and reference cores pinned at stage 1
  (2026-10-11): the test programs with their console logs and pictures, the documents and their exclusions, the probe on
  the three libretro cores (only PCSX-ReARMed may run without a BIOS; Beetle PSX and SwanStation boot OpenBIOS and are
  refused), their costs and spaces, the referee's simulator, and the search for an AccuracyCoin-like suite.
- [`Diamond_Disputes.md`](Diamond_Disputes.md) — the disputes log, created 2026-10-11 with its first entry, D-0: the
  NTSC console's video clock and line length, and so its frame rate; from stage 3, D-4 (memory control's access
  times), D-5 (the DMA's gaps and a store's drain) and D-6 (timer sync mode 2).

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the naming scheme, and `Man pages/Hardware/README.md` for how this
documentation set is organized.
