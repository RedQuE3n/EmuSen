# VenusRT_Disputes — the disputes log

*Created 2026-09-30, at stage 0, before any VenusRT rule was written.* This is the log `VenusRT_Plan.md` §1.4 defines.
An entry records one disagreement between VenusRT and an oracle that the documents did not settle at once, the
evidence that settled it, and the rule as implemented. It is the only record of which sources a dispute step read,
so that the clean-room protocol of §1.2 can be audited from outside.

## The rules this log keeps

- **A dispute step is separate.** It changes no code. It ends with a rule stated in hardware terms, and a later
  writer's step implements that sentence and cites the entry (`VenusRT_Plan.md` §1.2).
- **The order of recourse is fixed:** the documents, then a test ROM written for the purpose, then the SNES_MiSTer
  RTL, and only then Mesen's source. An entry that cites Mesen's source says why the three before it were not enough.
- **Nothing read in a dispute step is typed into code in that step.** Files read are listed by path and line; nothing
  is transcribed here either, only the conclusion in prose.
- **An entry is never deleted.** A conclusion that is later overturned is struck through and followed by the new one,
  with the evidence that overturned it.
- **The weight of the referee** is `Venus_Referee.md` §0's table, subsystem by subsystem. Only §0 of that page is
  read; its later sections describe Venus's internals.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what showed the disagreement: ROM, frame, space, the two values>
- Documents read: <title, section> for each
- Test ROM: <the ROM written or run to settle it, its result on VenusRT and on Mesen>
- Referee: <RTL files and lines read, and the weight of that subsystem per Venus_Referee.md §0>
- Mesen's source: <none> | <commit, each file path read, and why the three above did not settle it>
- Conclusion: <the rule as implemented, in prose>; argued | measured
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the later commit that cites this entry>
```

## Entries

Stage 0 wrote no emulation rule and opened none. The entries begin with stage 1.

### D-1. 65C816, emulation mode with DL zero: ~~a direct-page pointer's second byte is read at the first byte's address plus one, carried across the page~~ a (d), (d,X) or (d),Y pointer's second byte stays within the direct page; [d], [d],Y and PEI carry
- Opened: 2026-09-30, at stage 1, by SingleStepTests 65816 case `e1 e 8669` (SBC (d,x), E=1, D=$F400, DO=$B0, X=$4F).
  The index addition wrapped within the page to $00:F4FF, as every other emulation-mode case requires; the pointer's
  high byte was then read at $00:F500. VenusRT, wrapping it within the page, read $00:F400.
- Documents read: WDC, W65C816S datasheet (March 13, 2024), Table 5-7, rows 11, 12 and 13, which give the pointer's
  second address as D+DO+X+1 and D+DO+1 with no wrap; anomie, "SNES OpenBus & Wrapping" (romhacking.net document 194,
  revision 1126), "Instruction Wrapping", which says that word reads in emulation mode with DL zero wrap within the
  page "theoretically" and that the (d) and (d,X) address loads follow the same rule, untested.
- Test ROM: the single-step suite above, one case. No ROM in the corpus isolates the rule, and none was written; the
  suite holds no case of the same crossing for (d) or (d),y (a search of all of it, 2026-09-30).
- Referee: not read.
- Mesen's source: none.
- Conclusion: the pointer's second byte is never page-wrapped; only the index addition of d,X and (d,X) is, with DL
  zero in emulation mode. Measured for (d,X) by the one case; argued for (d) and (d),Y from the datasheet, against
  anomie's untested note. The datasheet is the primary document and the only test agrees with it, so the dispute is
  settled at the first rung for (d,X). For (d) and (d),Y it stays open until a test ROM or the referee is read; a
  later step that finds a game or ROM depending on it reopens this entry.
- Pinned by: `the_cpu_through_the_whole_suite` (case `e1 e 8669`).
- Implemented in: the commit after this entry's, "VenusRT stage 1: the 65816's data instructions".
- **Reopened 2026-09-30, at stage 1 step 2, and the conclusion above overturned.** A document not read before: the
  same datasheet's §7.2, "Direct Addressing": in emulation mode with DL zero "the direct addressing range is 000000
  to 0000FF, except for [Direct] and [Direct],Y addressing modes and the PEI instruction which will increment from
  0000FE or 0000FF into the Stack area", and the same for a nonzero DH. That names the exceptions, and (d), (d,X) and
  (d),Y are not among them. It also takes away the first conclusion's ground: Table 5-7 writes D+DO+X for d,X, which
  every emulation-mode case of the suite shows wrapping, so the table's formulas do not encode emulation wrapping and
  say nothing either way. anomie's note agrees with §7.2. *Measured:* the suite's one PEI case of the same crossing
  (`d4 e 232`, D=$0C00, DO=$FF) carries to $0D00, as §7.2 says PEI does; being §7.2's named exception, it does not
  bear on (d) or (d),Y.
- New conclusion: (d), (d,X) and (d),Y read a pointer's second byte within the direct page, in emulation mode with
  DL zero; [d], [d],Y and PEI carry. Argued from two documents. The single-step case `e1 e 8669` disagrees for (d,X)
  and is a named exception: one model's output against the chip's datasheet and anomie, with no hardware result
  either way. Open for a dispute step with the referee (`Venus_Referee.md` §0 weighs its 65C816 as real support,
  transcribed from the same datasheet, so its agreement would be correlated with §7.2's).
- Pinned by: `the_cpu_through_the_whole_suite`, which requires `e1 e 8669` to be the only failure of `e1.e.json` and
  the only (d,X) case of its kind.

### D-2. 65C816: JMP (a,X) and JSR (a,X) read their pointer as a program address, VPA high and VDA low
- Opened: 2026-09-30, at stage 1 step 2, by every case of SingleStepTests `7c` and `fc`, 40,000: the suite gives the
  two pointer reads VDA high and VPA low.
- Documents read: the W65C816S datasheet, Table 5-7, rows 2a and 2b: "PBR,AA+X, New PCL", VDA 0, VPA 1, and the same
  for the high byte. §2.26 defines VPA as a valid program address. The pointer is read from the program bank, PBR,
  not from DBR as (a,X)'s data modes would.
- Test ROM: none. On the SNES both pin pairs select memory, so the difference is not observable through the console's
  bus (argued: stage 2's bus treats VDA or VPA as an access); only a logic analyser on the chip could settle it.
- Referee: not read. Mesen's source: none.
- Conclusion: VPA, as the datasheet's table gives it; argued. The 40,000 cases are named exceptions on these two
  cycles' VDA and VPA only. Open, and of no consequence to a game.
- Pinned by: `the_cpu_through_the_whole_suite`, which grades `7c` and `fc` again with only those two pins of the
  pointer reads exchanged, and requires every case to pass that way.

### D-3. 65C816, emulation mode: JSR (a,X) addresses S in 16 bits; PLB stays in page 1
- Opened: 2026-09-30, at stage 1 step 2, by SingleStepTests `fc.e` and `ab.e`. At S=$0100 the suite's JSR (a,X)
  pushes to $0100 and then $01FF (43 cases); at S=$01FF its PLB reads $0200 (37 cases). Both are the reverse of the
  datasheet's list.
- Documents read: the W65C816S datasheet, §7.1, "Stack Addressing": "The following opcodes and addressing modes will
  increment or decrement beyond this range when accessing two or three bytes: JSL, JSR (a,x), PEA, PEI, PER, PHD,
  PLD, RTL". PLB is not there, and PLB accesses one byte. anomie's document 194 repeats the list from the datasheet
  and confirms on hardware PEA, PLD, d,S and (d,S),Y only. The suite agrees with the list on every other opcode in it
  (JSL, PEA, PEI, PER, PHD, PLD, RTL) at the page edge.
- Test ROM: none. A ROM that runs JSR (a,X) at S=$0100 and PLB at S=$01FF in emulation mode on a console would
  settle both; writing one is owed if a game is found to depend on either.
- Referee: not read. Mesen's source: none.
- Conclusion: the datasheet's list; argued. The 80 cases are named exceptions. Open.
- Pinned by: `the_cpu_through_the_whole_suite`, which requires the failures of `fc.e` (besides D-2's pins) and of
  `ab.e` to be exactly the cases at those stack edges.


Two readings of the oracles were made at stage 0 that a later reader might take for disputes. They are not: each is
about what a suite's file format means, settled by the suite's own data and README, and neither says anything about
the hardware. They are recorded in `VenusRT_Native.md` §2.3: the SingleStepTests 65816 cases of MVN and MVP stop at
100 cycles in the middle of a move, and the SPC700 suite records dummy reads of memory a case does not list without
their data.
