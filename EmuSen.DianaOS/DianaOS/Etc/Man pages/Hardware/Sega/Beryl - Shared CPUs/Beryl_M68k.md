# Beryl_M68k — the 68000's record

*Started 2026-10-04, at stage 0.* `beryl-m68k`, the Motorola 68000 shared by Nephrite's Genesis and Sega CD and by any
later core with a 68000. Planned in `Nephrite_Plan.md` §8 and §6's stage 1; the interface every Beryl crate keeps is
`README.md` beside this page. Registers as in `Nephrite_Native.md`: **measured** on the desktop (Ryzen 7 7700X, Fedora
44, Rust 1.98.1), **argued**, **predicted**.

## 1. Stage 0 (2026-10-04)

The crate's skeleton and its oracle: the bus trait, the programmer's model and the step's signature, which answers
`NotBuilt` while `BUILT` is false; and the single-step harness with its controls. No instruction is implemented.

## 2. The interface

- **`Access`**: a 24-bit address that keeps A0 (the byte select of a byte cycle), the width (`Byte`, `Word`), the
  function code FC2–FC0 (constants in `function`), and `locked` for TAS's read and write, between which AS stays
  asserted. The Genesis's bus does not complete TAS's write, so the bus, not the processor, decides what a locked write
  does.
- **`Bus`**: `read` and `write` (four clocks each, plus the waits the bus adds), `idle` (clocks without the bus, in the
  manual's order among the cycles), `interrupt_level` (IPL2–IPL0 as sampled), `acknowledge` (the vector, or `None` for
  the autovector), `address_error` (a word cycle at an odd address abandoned without AS: by default four idle clocks)
  and `reset_devices` (the RESET instruction's pulse).
- **`Registers`**: D0–D7, A0–A7 with A7 the current mode's stack pointer and `other_sp` the other's, SR, PC, and the two
  prefetch words (IRD and IRC in the user's manual).
- **`M68000::step`** is `step_observed` with `Unobserved`; `step_observed` is generic over `emusen-native`'s
  `Observer`. A step is one instruction or one exception (`Step`).

## 3. The oracle

### 3.1 The single-step harness

`src/singlestep.rs`, unit tests over two suites found through `EMUSEN_BERYL_CORPUS`
(`~/.cache/emusen/probe/nephrite/singlestep`): SingleStepTests' `m68000/v1` (127 files × 2,500 cases, generated from
MAME's microcoded 68000; its README names TAS's timing and TRAPV as unverified) and TomHarte's `680x0/68000/v1` (124
files × 8,065). Without the corpus the corpus tests pass unrun.

**The bus** is a flat 16 MiB holding only what a case lists, recording every transaction in the form the processor's
`Bus` sees: a byte cycle's address with A0 and its value as the byte. Idle runs are merged on both sides before
comparing, since one idle period may be reported in parts. Each case is graded three ways, counted apart: the registers
(D, A, USP, SSP, SR, PC and the prefetch), the memory the case lists, and the transaction list, compared whole; an
address-error cycle is compared by kind, function code, word address and width, not by value, since nothing on the bus
sees it.

**The suites' forms, read from their data and READMEs** (statements about the files, not the hardware, so none is a
dispute):

1. **SingleStepTests drops A0 and names the lanes.** A transaction is `[kind, clocks, fc, address, width, value, UDS,
   LDS]` with 1 asserted; a byte cycle's lane gives A0 (LDS alone: odd).
2. **Its data bus is the real bus.** A byte on the upper lane (UDS alone) has its value in the upper eight bits of the
   recorded word, `0xAB00` for `0xAB`; a lower-lane byte is the low eight bits. Read the other way, the replay control
   failed 29,003 of 317,500 cases, every one a byte cycle on the upper lane (**measured**).
3. **Its PC is MAME's next-prefetch address**, the opcode's address plus four; TomHarte's is the opcode's address.
   Both are loaded and compared as the suite gives them, and stage 1's processor reads each suite's convention.
4. **TAS.** SingleStepTests records TAS as a read, two idle clocks and a write; TomHarte's records one `t` transaction of
   ten clocks carrying the written value. The harness expands `t` into the same three, its read's value not compared.
5. **Address errors.** SingleStepTests records the abandoned cycle as `re` or `we`; TomHarte's suite has none.

**The controls (measured 2026-10-04).**

- *Positive:* a replay processor that performs each case's own transactions through the harness's bus and ends in the
  case's final registers. A replay pass shows that the harness loads, answers and records as the suites' environments
  did. **317,500 of 317,500** SingleStepTests cases and **1,000,060 of 1,000,060** TomHarte cases pass, in 4.4 s of wall
  time on four threads for both.
- *Negative:* two spoiled replays over the first 200 cases of every eighth file, each of which must pass nothing: the
  carry flipped in the final SR (fails on registers) and the last transaction dropped (fails on transactions). **0 of
  3,200** pass, for each spoiling and each suite.
- *The unbuilt processor* fails the inline case, as it must.

### 3.2 What the suites do not cover

The Genesis's bus (its waits, the Z80's window, DMA's hold), interrupts arriving mid-instruction, and anything over
more than one instruction. Those are the console test programs' (`Nephrite_Plan.md` §3.2): the BCD verifier, the opcode
sizes and the illegal-instruction test, and Yacht.txt for the order of cycles.
