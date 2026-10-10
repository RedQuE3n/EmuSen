# Diamond (Sony PlayStation)

The PlayStation core, to be written in Rust from hardware documents under a clean-room protocol, on the core ABI v1
from its first commit (`EmuSen/Cores/Sony/Diamond - PlayStation/`, crate `diamond`). No code exists yet.

- [`Diamond_Plan.md`](Diamond_Plan.md) — the plan, written 2026-10-10 before any emulation: the machine in scope and
  how accurate each part must be (the R3000A with its cache and scratchpad, the GTE exact, DMA, timers, interrupts, a
  cycle-faithful software GPU, the SPU, the CD-ROM controller with CD-DA and XA, the MDEC, pads, cards and the
  multitap, PAL and NTSC), the disc formats, who may read what, the sources and where they are thin, the oracles by
  component, the black-box references and the FPGA referee, what a frontend gets, the open BIOS, the speed budget, the
  enhancements decided on 2026-10-10 (precise geometry and the rest) and their separable architecture, the stages with
  their gates, the shared MIPS and disc crates, and the questions for the tester.
- `Diamond_Native.md` — the build record, from stage 1.
- `Diamond_Disputes.md` — the disputes log, created with its first entry (D-0, the NTSC frame rate).

See `Man pages/EmuSen_Core_Naming_Scheme.md` for the naming scheme, and `Man pages/Hardware/README.md` for how this
documentation set is organized.
