# Beryl Hardware (the shared hardware of the Sega cores)

Not a console core: the chips more than one Sega core uses, one crate each in `EmuSen/Cores/Sega/Beryl-HW/`, so that
each can be used alone. The processors came first, planned in `Nephrite_Plan.md` §8 since Nephrite is their first
user; each crate's own record is beside this page. *Decided 2026-10-05: crates shared between the Sega cores live in
Beryl, whatever the chip; the folder, once named for the CPUs alone, is `Beryl-HW` (Beryl Hardware).* The interface
below is the processors'; a crate of another kind keeps the contract its own section gives.

- [`Beryl_M68k.md`](Beryl_M68k.md) — `beryl-m68k`, the Motorola 68000, graded by SingleStepTests' 68000 suite and
  TomHarte's 680x0 tests, their disagreements refereed by fx68k and Nuked-MD's 68000 run as black boxes (its §6).
- [`Beryl_Z80.md`](Beryl_Z80.md) — `beryl-z80`, the Zilog Z80, graded by SingleStepTests' Z80 suite and ZEXALL.
- [`Beryl_SH2.md`](Beryl_SH2.md) — `beryl-sh2`, the Hitachi SH-2 (SH7604), for which no single-step suite exists.

## The interface every crate keeps

- **A bus trait in the processor's own terms**, which the core implements: the 68000's cycles with their function
  code, the Z80's machine cycles with the opcode fetch's refresh address and I/O, the SH-2's external accesses by width;
  each with the processor's interrupt lines and acknowledge cycle.
- **The bus owns the clock.** The processor reports every access and every internal cycle through the bus, which
  advances its own time and adds its wait states; the processor keeps no clock. One implementation then serves a flat
  test bus, a console's bus with its waits and a second instance on a coprocessor's bus alike. The reason is the
  scheduler of `Nephrite_Plan.md` §5.1: every access advances the master clock, so devices can be caught up exactly at
  the access that couples them.
- **The debugger through `emusen-native`'s `Observer`** (`debug.rs`, added 2026-10-04). Each crate has `step`, which
  is `step_observed` with `Unobserved`, so the plain step is compiled with no hook at all; the observed step is
  compiled again with the shared `Hooks` and reports the instruction boundary (breakpoints, stepping, coverage on its
  processor), each stored byte, and calls, exceptions and returns. Processor 0's breakpoints are the hooks'; another
  processor's are its core's to keep, as `Hooks` already arranges.
- **State, disassembly and an assembler.** Each processor's registers are a block of its core's state written with
  `StateWriter`; each crate decodes its instructions for `DEBUG_DISASSEMBLE` and carries an assembler as a development
  tool, for its test programs and for the firmware replacements `Nephrite_Plan.md` §5.6 writes in assembly.
- **The single-step oracle lives in the crate**, as unit tests that find the corpus through `EMUSEN_BERYL_CORPUS`
  (`~/.cache/emusen/probe/nephrite/singlestep` on the desktop) and pass unrun without it, or while the crate's `BUILT`
  is false.
