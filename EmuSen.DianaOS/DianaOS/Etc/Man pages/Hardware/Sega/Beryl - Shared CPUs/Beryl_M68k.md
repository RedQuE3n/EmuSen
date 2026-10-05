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
then. *Settled 2026-10-05 in §6: every class for SingleStepTests' reading except the overflow flags of the divisions
(D-9), which stay open. The referees also overturned SingleStepTests on four points (D-2, D-3, D-4, D-15) and refined
a reading its data could not test (D-5).*

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

*Opened 2026-10-05; §6 is the step.* It would take §3.3's classes one at a time, in the order of recourse of
`Nephrite_Plan.md` §1.3 (documents, then a test program, then the referees' 68000s, fx68k in the MegaCD core and
Nuked-MD's `68k.v`, each file logged), and end each with a rule in hardware terms:

1. Address errors: the abandoned cycle and the idle clocks before the stacking, the stacked PC's rule (§4.2), and
   whether a long postincrement advances before the fault.
2. The function code of PC-relative operand reads (program space by the PRM's Section 2; TomHarte's suite says data).
3. RTE's and RTR's read order.
4. CHK's and the divisions' internal delays.
5. ASR's C and X with a count at or above the operand's width.
6. LINK A7's stacked value.
7. The 733 ADD.l, SUB.l and ASL.b cases not yet read, and the rest of §4.2's readings, which rest on one suite.
8. Interrupt sampling within an instruction's last bus cycle (§5.1), if the Genesis's tests need it.


## 6. Stage 1, the dispute step: the two suites' disagreements (2026-10-05)

§5.7's classes, taken in its order and each settled in the order of recourse of `Nephrite_Plan.md` §1.3: the
documents, then a test program, then the referees' 68000s. No hardware is at hand to run a test program on, and none in
the corpus isolates any of these classes, so the second rung was empty for every class; where the documents left a
class open, the referees decided it. The referees were **run, not read**: each was compiled into a simulation and given
the suites' cases, and only its outputs were compared. The files read to build the benches are logged in §6.4. No
emulator source was opened.

This page holds the step's disputes log (`Nephrite_Plan.md` §1.4: a CPU dispute belongs to every core that uses the
crate), §6.3, in the entry shape of `VenusRT_Disputes.md`. The step itself changed no processor code; the rules it
ended with were implemented in the commit after it, which each entry's last line names.

### 6.1 The referees, run as black boxes

- **The simulator.** Verilator 5.046, from Fedora 44's packages (`verilator-5.046-3.fc44` and `gperftools-libs`),
  unpacked under `~/.cache/emusen/toolchains/verilator` rather than installed.
- **fx68k**, from the MegaCD MiSTer checkout (`~/Projects/megacd-mister-reference`, `a3a3da8`, 2026-07-30): `fx68k.sv`,
  `fx68kAlu.sv`, `uaddrPla.sv` and the microcode and nanocode tables. Two waivers: Verilator refuses a structure
  assigned both ways in one module (`BLKANDNBLK`), and the ALU's `unique case` checks fire on the reset state, so
  assertions are off at run time.
- **Nuked-MD's `m68kcpu`**, from the MegaDrive MiSTer checkout (`~/Projects/megadrive-mister-reference`, `4b11b2c`,
  2026-10-01): `68k.v` with `68k_ucode.txt` and `68k_ncode.txt`. A transcription of the NMOS 68000's die; its README
  calls it "Done, needs more testing & bugfixing".
- **The bench**, `EmuSen.WiseMan/Reference/rtl68k/tb68k.cpp`, built for either core by `build-referees.sh` into
  `~/.cache/emusen/probe/{fx68k,nuked68k}`. Memory answers at once (DTACK with AS), interrupts and the other inputs are
  held inactive. fx68k is given two of its clocks to each half of the 68000's clock, its phase enables alternating;
  Nuked-MD seven of its master clocks. After reset a boot program loads the case: the reset vectors point at code placed
  away from every byte the case names, which sets USP (`MOVEA.L`, `MOVE A0,USP`), loads D0-D7 and A0-A6 (`MOVEM.L` from
  a table) and runs `RTE` from a frame holding the case's SR and PC. RTE's two prefetches are answered with the case's
  own prefetch words, which TomHarte's cases do not hold in memory. The case starts as the second prefetch ends.
- **What is recorded.** Every bus cycle from the case's start: its clock, read or write, the function code, the address
  with A0 from the data strobes, and the value on the strobed lane. A cycle with AS and neither data strobe is recorded
  as abandoned. TAS's read-modify-write cycle holds AS from its read through its write, so it is recorded as two cycles,
  split where R/W turns. A second run plants a probe at the case's final PC, `MOVEM.L D0-D7/A0-A7,abs.L` and
  `MOVE SR,abs.L`, which stores the registers and the status register. The probe's first prefetch, a strobed read of its
  third word, marks where the instruction ended, which gives the instruction's length. Where the final PC is the case's
  PC plus two, the next opcode is already in the queue when the case starts, so the probe's first word is put there too.
  A case that a trace follows is compared on the bus only, and so is STOP, which waits for an interrupt that never
  comes; an address error clears T, so its cases are probed.
- **The comparison**, `referee.py`. The timelines are compared cycle by cycle in clocks from the instruction's start. A
  suite's abandoned cycle (`re`, `we`) is compared by kind, function code and word address. The length is compared, and
  so are D0-D7, A0-A6, the active A7 and SR. The inactive stack pointer and the prefetch queue are not seen. `suite`
  grades one suite's cases, `disputes` the cases a listing names (with Beryl's own result beside each), `divzero` the
  flags of division by zero from every condition code. The listings come from `singlestep.rs`
  (`EMUSEN_BERYL_DISAGREEMENTS`, `EMUSEN_BERYL_MISSES`).

### 6.2 The controls (measured 2026-10-05)

- **fx68k over every SingleStepTests case** agrees with the suite on 310,649 of 317,500: on the bus and the length in
  315,094, and on the registers in 179,712 of the 184,157 cases the probe reached. The 6,851 it does not agree on are
  those of D-2, D-3, D-4 and D-15 (§6.3), in each of which Nuked-MD agrees with fx68k. Every other reading §4.2 took
  from this suite alone is therefore confirmed by the referee.
- **Nuked-MD over 200 cases of each SingleStepTests file** (25,400) agrees with the suite on 24,803. Of the 597 it
  does not agree on, 516 are the ruled cases above, where it agrees with fx68k; 78 are divisions' overflows; 3 are a
  rotate with a zero count. Over all 55,606 address-error cases it gives the same bus cycles, stacked values and
  registers as fx68k, once its complemented access address (below) is allowed.
- **Nuked-MD's three differences**, found by these controls, two of them defects. It stacks a group 0 frame's access
  address complemented, both words: $5C32 where fx68k and SingleStepTests stack $A3CD. The User's Manual's frame (Figure
  6-7) holds the access address itself, so `referee.py --complement` accepts a word written as the complement of the
  reference's, and nothing else. It clears C after ROXL.L or ROXR.L with a zero count and X set, where the Programmer's
  Reference Manual sets C to X. On a division's overflow it sets Z and clears N, where fx68k and SingleStepTests set N
  and clear Z; that is D-9's split, which stays open.
- **Bench artifacts**, which the conclusions do not rest on. One TomHarte case (MOVEM.l (d16,PC)) reads its operands
  from the final PC, where the probe was planted, so its registers there are the probe's words. Four faults of the bench
  were found by the first full run over SingleStepTests and corrected before the final runs: TAS recorded as one cycle
  (2,108 cases); backward branches whose probe overwrote the branch itself in the queue (79); traced cases whose trace
  happened to reach the probe (11); and an end marker that matched an abandoned cycle at the probe's third word (3).

### 6.3 The disputes log

#### D-1. Address error: the abandoned cycle runs on the bus as AS without either data strobe, four clocks, and eight internal clocks follow before the stacking
- Opened: 2026-10-04, by §3.3's address-error class: 178,087 TomHarte cases record no abandoned cycle and four idle
  clocks before the stacking.
- Documents read: User's Manual §6.3.10 ("The bus cycle is aborted"), which does not say whether AS is asserted or how
  long the processor waits.
- Test program: none.
- Referee: fx68k and Nuked-MD, run (§6.1). Both run the faulting cycle with AS and no data strobe and begin the
  stacking's first write twelve clocks after that cycle begins, as SingleStepTests records it, in all 55,606 of its
  address-error cases. In TomHarte's address-error class, fx68k matched TomHarte's bus on none of the 178,087
  cases.
- Emulator source: none.
- Conclusion: SingleStepTests' form; TomHarte's suite is wrong for this class. The `Bus::address_error` hook stays a
  four-clock cycle that transfers nothing; its description, which said AS was not asserted, is corrected. Measured.
- Pinned by: `the_processor_against_the_suites`.
- Implemented in: stage 1 step 1, unchanged; the description, the commit after this step's.

#### D-2. Address error: the stacked PC of DBcc, of JSR through a displacement or an index, and of MOVEM to registers through an index
- Opened: 2026-10-05, by the controls: fx68k and Nuked-MD both stack another PC than SingleStepTests in these forms.
- Documents read: User's Manual §6.2: the PC stacked for an address error "is unpredictable and may be incremented from
  the address of the instruction that caused the error". The documents leave it open.
- Test program: none.
- Referee: both cores, identical on every case. DBcc taking its branch to an odd target stacks the opcode's address plus
  two, not plus four (632 SingleStepTests cases). JSR with `(d16,An)`, `(d8,An,Xn)`, `(d16,PC)` or `(d8,PC,Xn)` stacks
  the opcode's address plus two, as JMP does, not its return address (762); with `(An)` and the absolute modes it
  stacks its return address, as SingleStepTests has it. MOVEM to registers through `(d8,An,Xn)` or `(d8,PC,Xn)` stacks
  four less than through the other modes (306).
- Emulator source: none.
- Conclusion: as the referees; SingleStepTests' suite is wrong for these 1,700 cases. Measured.
- Pinned by: `the_processor_against_the_suites` (each file's ruled misses).
- Implemented in: the commit after this step's.

#### D-3. Address error: a postincrement is not applied before its operand's access, so a faulting `(An)+` leaves An unchanged
- Opened: 2026-10-05, by the controls: both cores leave a word `(An)+` register unadvanced on a fault, where both suites
  advance it. §4.2's reading already held this for a long operand and for MOVE's destination.
- Documents read: none says.
- Test program: none.
- Referee: both cores, every case. It holds for every instruction with a word `(An)+` operand, source or destination,
  and for CMPM's source, which §4.2 advanced before each read.
- Emulator source: none.
- Conclusion: as the referees. Both suites are wrong here: 4,445 SingleStepTests cases, and TomHarte's equivalents,
  which already fail on D-1. Measured.
- Pinned by: `the_processor_against_the_suites`.
- Implemented in: the commit after this step's.

#### D-4. Address error: the status word's I/N bit, for a MOVE's word store to `-(An)` made after the final prefetch, is set when the step after the next opcode's fetch is an exception
- Opened: 2026-10-05, by the controls: 83 SingleStepTests cases where both cores set I/N and the suite clears it.
- Documents read: User's Manual Figure 6-7: I/N is "Instruction=0, Not=1", without a rule for this point.
- Test program: none.
- Referee: both cores. The store follows the fetch of the next opcode (§4.2), and I/N is set exactly when what follows
  that fetch is an exception: a pending trace, or a next opcode that is illegal, line A or F, or privileged in user mode
  (by the instruction map). Measured on all 452 such faults in both suites: 452 agree.
- Emulator source: none.
- Conclusion: as the referees; SingleStepTests' suite is wrong for these 83 cases. Measured.
- Pinned by: `the_processor_against_the_suites`.
- Implemented in: the commit after this step's.

#### D-5. Address error: MOVE.L from a register to `(d16,An)` or `(d8,An,Xn)`, its store faulting, sets N from bit 31 and clears Z if the high word is non-zero, leaving Z otherwise, and V, C and X
- Opened: 2026-10-05, by TomHarte's cases, whose source register is often A7 = $00000800: both cores leave Z as it was
  where §4.2's reading set Z from the whole long.
- Documents read: none says.
- Test program: none.
- Referee: both cores; the rule fits all 468 such faults in both suites. SingleStepTests' random registers almost never
  have a zero high word, so the suite could not tell this from §4.2's reading.
- Emulator source: none.
- Conclusion: the flags are those of the high word's half of a long's flag computation, applied to the flags already
  held. Measured; for a zero long, argued from the same rule (no case).
- Pinned by: `the_processor_against_the_suites` (TomHarte's suite, measured).
- Implemented in: the commit after this step's.

#### D-6. The PC-relative modes read their operands in program space
- Opened: 2026-10-04, §3.3: 4,362 TomHarte cases read them in data space.
- Documents read: Programmer's Reference Manual Section 2: "these accesses classify as program references". Settled
  here, at the first rung.
- Referee, as corroboration: fx68k and Nuked-MD read in program space in all 4,362.
- Conclusion: program space, as SingleStepTests; TomHarte's suite is wrong for this class.
- Implemented in: stage 1 step 1, unchanged.

#### D-7. RTE reads the status word, then the PC's high word, then its low word; RTR the condition codes' word first
- Opened: 2026-10-04, §3.3: 8,049 TomHarte cases read the PC's high word first.
- Documents read: the Programmer's Reference Manual's operation lines name the status register first, which describes
  the result rather than the order of the cycles.
- Referee: both cores read the status word first in all 8,049 cases.
- Conclusion: SingleStepTests' order; TomHarte's suite is wrong for this class. Measured.
- Implemented in: stage 1 step 1, unchanged.

#### D-8. CHK's and the divisions' internal clocks, and CHK's flags, are §4.2's
- Opened: 2026-10-04, §3.3: 9,949 TomHarte cases.
- Documents read: Yacht.txt and the division timing analysis of §4.2, whose rules Beryl follows.
- Referee: both cores give Beryl's bus and length in every CHK, DIVU and DIVS case of the class, and Beryl's CHK flags
  in all 321 where TomHarte's differ.
- Conclusion: as §4.2; TomHarte's suite is wrong on these clocks. Measured.
- Implemented in: stage 1 step 1, unchanged.

#### D-9. OPEN. N and Z after a division's overflow
- Opened: 2026-10-04, within §3.3's division class: 3,609 TomHarte cases differ only in these flags.
- Documents read: the Programmer's Reference Manual calls both undefined on overflow.
- Test program: none.
- Referee: split. fx68k sets N and clears Z, as SingleStepTests; Nuked-MD sets Z and clears N; TomHarte's suite
  agrees with neither consistently.
- Conclusion: none. Beryl keeps SingleStepTests' reading, which gates, and the entry stays open for a test program run
  on a 68000; no Genesis game is known to read these flags.

#### D-10. Division by zero leaves X, clears V and C, and sets N and Z from the dividend's high word for DIVU; for DIVS, sets Z and clears N
- Opened: 2026-10-05, by TomHarte's one division by zero (DIVU.json case 5745); SingleStepTests has none.
- Documents read: the Programmer's Reference Manual: N, Z and V undefined, C cleared.
- Referee: `referee.py divzero`, DIVU and DIVS by zero from each of the 32 condition codes and eight dividends. Both
  cores give the same table: the result does not depend on the previous N, Z, V or C. For DIVU, N is the dividend's
  bit 31 and Z is set when its high word is zero; for DIVS, Z is set and N cleared. Beryl had kept N and Z.
- Conclusion: as the referees. TomHarte's case also stacks the opcode's address where both cores stack it plus four,
  so the case still fails. Measured.
- Pinned by: the referee's table, rerun by `referee.py divzero`; no suite case.
- Implemented in: the commit after this step's.

#### D-11. ASR with a count at or above the operand's width sets C and X to the sign bit
- Opened: 2026-10-04, §3.3: 3,736 TomHarte cases clear them.
- Documents read: the Programmer's Reference Manual: C and X are "set according to the last bit shifted out of the
  operand", and an arithmetic shift right fills with the sign, so the last bit out is the sign. Settled here.
- Referee, as corroboration: both cores in all 3,736.
- Conclusion: SingleStepTests'; TomHarte's suite is wrong for this class.
- Implemented in: stage 1 step 1, unchanged.

#### D-12. LINK A7 stacks A7's value before the decrement
- Opened: 2026-10-04, §3.3: 1,005 TomHarte cases stack A7 less four.
- Documents read: the Programmer's Reference Manual's operation, "SP - 4 → SP; An → (SP)", read in order, would stack
  the decremented value. It describes the general case and does not settle An = A7.
- Referee: both cores stack the original value in all 1,005.
- Conclusion: SingleStepTests'; TomHarte's suite is wrong for this class. Measured.
- Implemented in: stage 1 step 1, unchanged.

#### D-13. ADDQ.L and SUBQ.L to an address register take eight clocks
- Opened: 2026-10-05, from the 731 ADD.l and SUB.l cases of §3.3, all `ADDQ.L` or `SUBQ.L` to An, which TomHarte's
  suite times at six clocks.
- Documents read: the User's Manual's Table 8-5, "8(1/0)". Settled here.
- Referee, as corroboration: both cores end the instruction at eight clocks in all 731.
- Conclusion: eight; TomHarte's suite is wrong for this form.
- Implemented in: stage 1 step 1, unchanged.

#### D-14. ASL.B changes only the low byte
- Opened: 2026-10-05, from §3.3's two ASL.b cases (ASL.b.json 1582 and 1760). TomHarte's suite gives D2 a final value
  unrelated to its initial one: `ASL.B #2,D2` on $CDFB7FBE ends at $2E5E4304, where the shift gives $CDFB7FF8.
- Documents read: the Programmer's Reference Manual's ASL. Settled here.
- Referee, as corroboration: both cores give $CDFB7FF8.
- Conclusion: a defect in those two cases of TomHarte's data.
- Implemented in: stage 1 step 1, unchanged.

#### D-15. TRAPV's prefetch is made before the trap, in the mode TRAPV ran in
- Opened: 2026-10-05, by the full control over SingleStepTests: in all 623 cases where TRAPV traps in user mode, both
  cores make its prefetch in user program space (function code 2), where the suite, and §4.2's reading taken from it,
  has supervisor program space (6). SingleStepTests' README names TRAPV as unverified. TomHarte's TRAPV cases all run in
  supervisor mode, where the two readings coincide.
- Documents read: the Programmer's Reference Manual's TRAPV, which does not say.
- Test program: none.
- Referee: both cores, all 623.
- Conclusion: the instruction's own prefetch comes first, and supervisor mode is entered with the exception that follows
  it; SingleStepTests' suite is wrong for these 623 cases. Measured.
- Pinned by: `the_processor_against_the_suites`.
- Implemented in: the commit after this step's.

#### Not opened: interrupt sampling within an instruction's last bus cycle (§5.7, item 8)
The single-step suites have no interrupt case, and §5.1's rule is argued. The Genesis's interrupt timing tests at
stages 3 and 4 open it if they need it.

### 6.4 Files read in this step

| File | What was read | What for | Class it settled |
|---|---|---|---|
| `megacd-mister-reference/rtl/FX68K/fx68k.txt` | Whole: the core's own description of its clock enables, reset and power-up inputs | The bench's clocking and reset | None by reading; every class by running (D-1 to D-14) |
| `megacd-mister-reference/rtl/FX68K/fx68k.sv` | The port declarations of `fx68k` and `fx68kTop`; and lines 283, 728, 775, 1174 and 1175, which Verilator's diagnostics quoted | The bench's pins; the build's waivers | None by reading |
| `megadrive-mister-reference/rtl/nuked-md/68k.v` | The `m68kcpu` port list (lines 26-56) and its `$readmemb` lines (1106-1109) | The bench's pins; which tables to copy | None by reading; every class by running |
| `megadrive-mister-reference/rtl/nuked-md/md_board.v` | The `m68kcpu` instance (lines 520-570) | Which clock drives MCLK and CLK | None |
| `megadrive-mister-reference/rtl/nuked-md/README.md`, `Progress.md` | Whole | The 68000's status | None |

No other RTL file was opened, and no logic of either core was read. Documents read in this step: the User's Manual
§6.2, §6.3.10, Figure 6-7 and Table 8-5, and the Programmer's Reference Manual's Section 2 and its pages for ASR,
ASL, LINK, RTE, DIVU, DIVS, ROXL, ROXR and TRAPV.
