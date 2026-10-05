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

### 4.3 What stage 1 still owed

Interrupts, the reset exception, trace mode, a check of the decoder against TomHarte's instruction map, the
disassembler, the state block and the processor's own cost, none of which the single-step suites exercise. Step 2
built them (§5).

### 4.4 Provenance

Read to write this step: the Programmer's Reference Manual and the User's Manual (Motorola), Yacht.txt, the BCD
verifier's README and expected-results model, and the two suites' data and READMEs. No emulator source was opened, the
suites' generators included.

## 5. Stage 1, step 2: between instructions, the decoder, the state and the cost (2026-10-05)

### 5.1 The order of a step

A step is one instruction or one exception, decided at the instruction boundary in this order:

1. **A trace owed by the previous instruction** (vector 9). Trace is group 1's highest priority, so it is taken before
   a pending interrupt. Recording it as owed, rather than taking it inside the traced instruction's step, keeps a step
   one instruction long, which is also what the single-step suites assume: many of their cases start with T set and
   end before any trace.
2. **A pending interrupt.** The User's Manual (§6.3.2): requests "are made pending" and "are detected between
   instruction executions"; one above the mask is taken; level 7 cannot be masked and "is generated each time the
   interrupt request level changes from some lower level to level 7", and "may still be caused by the level comparison
   if the request level is a 7 and the processor priority is set to a lower level by an instruction". The processor
   samples IPL once per boundary and keeps the level it last saw, which the state carries, so that a rise to 7
   interrupts even at mask 7 and a level 7 held into its handler does not interrupt again until the mask is lowered.
   *Argued, not measured:* when within an instruction a change of IPL becomes pending, relative to its last bus cycle,
   is below this granularity; the VDP's interrupt timing tests at stage 4 are the oracle that will say whether it
   matters.
3. **STOP** idles four clocks a step until an interrupt (or a trace) ends it. The PC stacked when an interrupt ends
   STOP is the instruction after it, four bytes on; the suite's STOP cases leave the PC on STOP itself, as SingleStepTests
   records it.
4. **The instruction**, with the observer's stop first.

### 5.2 The sequences (Yacht.txt's tables, with the two clocks the suite showed between the handler's prefetches)

| Exception | Bus activity | Clocks |
|---|---|---|
| Interrupt | 6 idle; PC low stacked; the acknowledge cycle (FC 7), its clocks the bus's; 4 idle; SR, PC high; vector; prefetch, 2 idle, prefetch | 44 with a four-clock acknowledge |
| Reset | 14 idle; SSP and PC from 0-7 in supervisor program space; prefetch, 2 idle, prefetch | 40 |
| Trace | 4 idle; PC low, SR, PC high; vector 9; prefetch, 2 idle, prefetch | 34 |

These are the User's Manual's totals (44, 40 and 34 clocks), which Yacht.txt notes it falls two clocks short of in
each; the two clocks between the handler's two prefetches, measured for every other exception in the suite (§4),
account for the difference in all three. The acknowledge cycle is the bus's to time, because on the Genesis it is an
autovector whose length follows the E clock. An interrupt sets the mask to its level, enters supervisor mode and clears
T; a device's vector is used when the bus returns one, the autovector (24 plus the level, the User's Manual's "$18 plus the
interrupt level") otherwise. The spurious interrupt (a bus error during the acknowledge) and the uninitialised vector
(15) are the bus's to return as vectors; the Genesis needs neither. Trace follows an
instruction that completes and the group 2 exceptions (TRAP, TRAPV, CHK, division by zero), whose handler's first
instruction is then the one stacked; it does not follow an illegal instruction, a privilege violation or an address
error. Reset sets S, clears T and sets the mask to 7, leaving the condition codes.

`src/boundary.rs` pins each against a recording bus: the cycle order and the totals above, a masked level, a device
vector, level 7's rise and its comparison once an instruction lowers the mask, STOP ended by an interrupt, reset, trace after an ordinary instruction and after TRAP, and none
after ILLEGAL. **These rules are argued from the manuals and Yacht.txt; no single-step oracle covers them**, and the
Genesis's own interrupt tests at stage 3 and 4 are their test.

### 5.3 Two decoder defects, found by the instruction map

`decodemap::every_opcode_against_the_map` runs the processor on each of the 65,536 opcodes in supervisor mode and asks
whether it takes an illegal or line A/F exception, against the map's "None". *Measured before the fix:* 13 opcodes
disagreed, all decoded by the processor and absent from the map: CMPI with a PC-relative destination (`0C3A`, `0C3B`,
`0C7A`, `0C7B`, `0CBA`, `0CBB`), which the Programmer's Reference Manual's CMPI page marks as not applying to the
MC68000 ("PC relative addressing modes do not apply to MC68000"), and `4E78`-`4E7F`, mode 7 of the miscellaneous group, which the processor
read as `4E70`-`4E77`. Neither is in either single-step suite, which test valid opcodes only. Both now take the
illegal-instruction exception; *measured after:* 0 of 65,536 disagree, and both suites still pass (§4).

### 5.4 The state block

`State` for `M68000` (`src/state.rs`): D0-D7, A0-A7, the other stack pointer, SR, PC, the two prefetch words, STOP and
the halt, the last sampled IPL and an owed trace with its PC, 86 bytes. The layout is pinned by a test, and a short
block is refused with the processor unchanged. A core writes it as a group of its own state.

### 5.5 The disassembler

`src/disasm.rs`: the Programmer's Reference Manual's syntax (`MOVE.W #$1234,-$10(A5)`, `MOVEM.L D0-D1/A0-A1,-(A7)`,
`$00FF0000.L`, targets as addresses), with `DC.W` for an opcode the 68000 does not decode, and the static reference
the words alone give: JSR and BSR targets as calls, absolute and PC-relative operands as reads, an absolute destination
of a store as a write. **Measured:** the map test also renders every opcode in the map's own notation (its names for
the special forms, its mode words, the quick operands' numbers) and compares: 0 of 65,536 differ. A second test runs
every instruction that falls through, 40,242 of them, and finds the disassembler's length equal to the words the
processor consumed in every case. Golden lines pin the syntax.

### 5.6 The processor's own cost (measured 2026-10-05)

`examples/cost.rs`: a loop of ordinary instructions (a long copy with postincrement, ADD, LSL, MULU, CMP, a branch,
ADDQ, JSR and RTS, DBRA) on a flat RAM bus that only counts clocks, 50 million instructions, median of three runs, on
the desktop (Ryzen 7 7700X) at CPU weight 20, load average 0.9: **16.3 ns an instruction**, 14.2 clocks an instruction,
873 million emulated clocks a second, 114 times the Genesis's 7.67 MHz. A frame's worth of 68000 work (128,006
clocks) takes **0.147 ms**, and the Sega CD's sub-CPU at 12.5 MHz would add about 0.24 ms. Two runs gave the same
figures to the third digit. The bus here is the cheapest possible, so these are the processor's cost alone; Nephrite's
bus with its map and waits will add to it, and `Nephrite_Plan.md` §5.5's budget of 1.5 ms for the Genesis leaves the
68000 about a tenth of it (argued).

### 5.7 What the dispute step would cover

Not opened. It would take §3.3's classes one at a time, in the order of recourse of `Nephrite_Plan.md` §1.3
(documents, then a test program, then the referees' 68000s, fx68k in the MegaCD core and Nuked-MD's `68k.v`, each file
logged), and end each with a rule in hardware terms:

1. Address errors: the abandoned cycle and the idle clocks before the stacking, the stacked PC's rule (§4.2), and
   whether a long postincrement advances before the fault.
2. The function code of PC-relative operand reads (program space by the PRM's Section 2; TomHarte's suite says data).
3. RTE's and RTR's read order.
4. CHK's and the divisions' internal delays.
5. ASR's C and X with a count at or above the operand's width.
6. LINK A7's stacked value.
7. The 733 ADD.l, SUB.l and ASL.b cases not yet read, and the rest of §4.2's readings, which rest on one suite.
8. Interrupt sampling within an instruction's last bus cycle (§5.1), if the Genesis's tests need it.

