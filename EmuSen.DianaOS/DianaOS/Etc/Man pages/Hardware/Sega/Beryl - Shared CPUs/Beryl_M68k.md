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

### 3.3 Where the two suites disagree

TomHarte's suite and SingleStepTests' disagree with each other on whole classes of case. Measured 2026-10-04 by
`the_second_suite_disagreements` (opt-in through `EMUSEN_BERYL_DISAGREEMENTS`), with the processor of §4 passing every
case of SingleStepTests' suite, each failing case of TomHarte's put in the first class that holds:

| Class | Cases | What differs |
|---|---|---|
| Address errors | 178,087 | TomHarte's suite records no abandoned cycle and four idle clocks before the stacking where SingleStepTests' records the abandoned cycle and eight; it advances a long postincrement before the fault; its stacked PC follows another rule |
| Function code only | 4,362 | TomHarte's suite reads the PC-relative modes' operands in data space; SingleStepTests' in program space, as the Programmer's Reference Manual's Section 2 classifies them ("these accesses classify as program references") |
| RTE, RTR | 8,049 | TomHarte's suite reads the stacked PC's high word before the status word |
| CHK, DIVS, DIVU | 9,949 | Internal delays |
| ASR | 3,736 | With a count at or above the operand's width, TomHarte's suite clears C and X where the bits shifted out are the sign |
| LINK A7 | 1,005 | TomHarte's suite stacks A7 less four; SingleStepTests' the original A7 |
| ADD.l, SUB.l, ASL.b | 733 | Not yet read |

**Decided for stage 1 (argued):** SingleStepTests' suite is the oracle that gates (G1), because it was generated from
a model of the 68000's own microcode, and TomHarte's from an implementation written by hand. Each class above is a
candidate disputes entry, to be settled by the documents and then by the referees' 68000s (fx68k in the MegaCD core,
Nuked-MD's microcoded `68k.v`) in logged dispute steps; none has been read. TomHarte's suite stays a measurement until
then.

## 4. Stage 1, step 1: the whole instruction set (2026-10-04)

### 4.1 What it built

`src/exec.rs`: every 68000 instruction, the group 1 and 2 exceptions (illegal, line A and F, privilege, TRAP, TRAPV,
CHK, division by zero), group 0 processing for address errors with its seven-word frame, STOP, RESET's pulse, and the
observer calls (the instruction boundary, each stored byte, calls, exceptions and returns). Semantics are the
Programmer's Reference Manual's; each instruction's bus cycles and internal delays are Yacht.txt's (§2), in the order it
gives them, through the bus of §2. Where the suite showed another order or another delay, the suite's was taken, and
§4.2 lists each such reading.

**Measured 2026-10-04, SingleStepTests' 68000 suite: 317,500 of 317,500 cases, registers, memory and every
transaction.** The first build passed 259,410 (81.7%); the readings of §4.2 took it to all. TomHarte's suite: 794,139
of 1,000,060, its failures the classes of §3.3. Both suites together run in 5.7 s of wall time on four threads,
JSON parsing included.

### 4.2 Readings taken from the suite

Each is a rule the suite's data shows and the documents leave open, stated in hardware terms. They are measured
against SingleStepTests' suite only, which §3.3's disagreements make a single source; each is therefore also a
candidate for a dispute step.

- **The PC a group 0 frame stacks.** The opcode's address plus two, advanced by two for each absolute address or
  immediate word fetched and, for a word or byte operand, by two at a predecrement; the displacements of `(d16,An)`,
  `(d8,An,Xn)` and the PC-relative modes do not advance it. MOVE stacks the prefetch address as its destination phase
  begins, plus two for an absolute long destination; MOVEM the prefetch address as its transfers begin; JSR its return
  address; BSR its target; DBcc, CMPM, ADDX and SUBX to memory, and UNLK, the prefetch address.
- **The access address** in the frame is the whole 32-bit address, its top byte included; the bus sees 24 bits.
- **The instruction register** in the frame, and in the status word's top eleven bits, is the opcode, except for MOVE
  of a byte or word to `-(An)`, whose store follows the final prefetch, which stacks the next opcode.
- **Postincrement and predecrement on a fault.** A long `(An)+` operand advances its register only after both words
  are read; MOVE's `(An)+` destination only after its store, whatever the size; MOVE.L's `-(An)` destination only after
  both words are stored; ADDX.L and SUBX.L adjust each register only after its operand's two reads; CMPM advances its
  source before each read (two at a time for a long) and its destination after.
- **MOVE.L's flags when its first store faults:** with an `(An)` or `(An)+` destination, untouched from a register or
  immediate source and taken from the low word from a memory source; N and Z set and V and C untouched for a register
  source to `(d16,An)` or `(d8,An,Xn)`; from the low word for an absolute long destination from memory; otherwise set
  in full. Byte and word moves set them in full before the store.
- **CHK.** Without a trap: six idle clocks, then the prefetch. With one: eight idle clocks, or ten when the register is
  negative and the bound less the register, as a 16-bit difference, is not negative. N is the register's sign; Z, V and
  C are cleared, Z set for a zero register.
- **DBcc** writes its decremented counter only after the branch's prefetch succeeds.
- **TRAPV's trap** enters supervisor mode before its prefetch, which therefore reads supervisor program space.
- **ORI, ANDI and EORI to CCR or SR, and MOVE to CCR or SR,** refill the prefetch queue from the word after the
  instruction: the word already in IRC is read again.
- **DIVU and DIVS on overflow** set N and V and clear Z and C. Their timing is Yacht.txt's flowcharts read as rules
  per quotient bit, the formulation also published as Jorge Cwik's analysis of the 68000's division timing.
- **ABCD, SBCD and NBCD** follow the BCD verifier's reference model (`flamewing/68k-bcd-verifier`, its README and its
  expected-results source, read as a test program's statement of what it expects): the binary carries of each nibble
  and the decimal carries choose the correction, and the undefined V and N follow from it.
- **LINK A7** stacks the original A7.
- **UNLK** reads before it moves the stack pointer, so a fault leaves A7 alone.
- **BTST** with an immediate destination idles two clocks after its prefetch, as with a data register.

### 4.3 What stage 1 still owes

Interrupts (IPL sampling at the instruction boundary, the acknowledge cycle, autovectors and the spurious vector) and
the reset exception; trace mode; a check of the decoder against TomHarte's instruction map (`map/68000.official.json`);
the disassembler for `DEBUG_DISASSEMBLE`; the registers as a `StateWriter` block; and the processor's own cost on the
desktop. None is exercised by the single-step suites, which begin and end at instruction boundaries with no interrupt
pending.

### 4.4 Provenance

Read to write this step: the Programmer's Reference Manual and the User's Manual (Motorola), Yacht.txt, the BCD
verifier's README and expected-results model, and the two suites' data and READMEs. No emulator source was opened, the
suites' generators included.

