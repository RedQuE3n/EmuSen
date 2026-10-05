# Beryl_SH2 — the SH-2's record

*Started 2026-10-04, at stage 0.* `beryl-sh2`, the Hitachi SH-2 as the SH7604 (with its cache and on-chip modules),
shared by Nephrite's 32X and, later, Zoisite's Saturn. Planned in `Nephrite_Plan.md` §8 and §6's stage 11; the common
interface is `README.md` beside this page.

## 1. Stage 0 (2026-10-04)

The bus trait, the registers and the step's signature (`NotBuilt` while `BUILT` is false), with a test that the
unbuilt processor refuses to step. No instruction is implemented.

## 2. The interface

- **`Bus`**: the external accesses that the processor's own cache and on-chip modules do not answer, by width (`Byte`,
  `Word`, `Long`), `idle` (cycles inside the processor: stalls, the multiplier and divider, cache hits),
  `interrupt_level` (IRL3–IRL0) and `acknowledge` (an external vector, or the auto-vector).
- **What stays inside the crate**: the 4 KiB four-way cache, and the SH7604's DMAC, FRT, WDT, DIVU, INTC, SCI and BSC,
  since they are on the chip and every machine with an SH7604 has them; the core gives the bus only what leaves the chip.
- **`Registers`**: R0–R15, SR, GBR, VBR, MACH, MACL, PR, PC. `Sh2::new(slave)` marks the slave of a pair.

## 3. The oracle

**No SH-2 single-step suite exists** (searched 2026-10-04: SingleStepTests has an SH-4 suite, generated from another
emulator's interpreter, and nothing for the SH-2). The plan, for stage 11:

- **Test programs from the manual.** The SH-1/SH-2 Programming Manual gives each instruction's operation as pseudocode.
  Programs written with the crate's assembler run each instruction over chosen and random operands on a flat bus and
  compare with the pseudocode's results, transcribed as expected values by a second writer from the manual.
- **The SH-4 suite, in part.** The SH-4's integer instructions that the SH-2 shares are a second opinion on results
  (registers and memory, not timing); the instructions whose behaviour differs (MAC's saturation, the delay slots'
  illegal instructions, anything touching the SH-4's banks or FPU) are excluded by a list read from the two manuals.
- **On the console.** 32X homebrew and the 240p 32X suite against PicoDrive through the probe; the S32X_MiSTer RTL in
  dispute steps only.
