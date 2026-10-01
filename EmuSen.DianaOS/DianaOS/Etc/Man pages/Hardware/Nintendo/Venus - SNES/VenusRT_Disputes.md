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
- ~~Pinned by: `the_cpu_through_the_whole_suite`, which requires `e1 e 8669` to be the only failure of `e1.e.json` and
  the only (d,X) case of its kind.~~
- **Settled 2026-09-30, at stage 1 step 2, by a test ROM written for the purpose** (the second rung): gilyon's
  `cputest-full` v1.4, whose README lists among the "undocumented behavior" it tests that in emulation mode "when the
  low byte of D is nonzero, the (direct,X) addressing mode behaves strangely: the low byte of the indirect address is
  read from direct_addr+X+D without page wrapping (as expected). The high byte is read from direct_addr+X+D+1, but the
  +1 is done *with* wrapping within the page", and that "this behavior only applies to this addressing mode". Its test
  list (`tests-full.txt`) holds 26 tests of (d) and (d),Y and 26 of (d,X) in emulation mode with DL zero and the
  pointer at the page's last byte, and 12 of (d,X) with DL nonzero (D=$011A, X=$EE, $F7: low byte at $02FF, high at
  $0200). *Measured:* VenusRT with the conclusion above passed all 52 DL-zero tests and failed the 12 DL-nonzero
  ones, which it carried. Mesen passes the whole ROM; C# Venus fails it at test 0024, `adc ($EF,x)`, one of these.
- Conclusion, final: in emulation mode a (d,X) pointer's second byte is read within the page of its first byte,
  whatever DL is; a (d) or (d),Y pointer's second byte within the page when DL is zero and carried when it is not;
  [d], [d],Y and PEI carry. Measured by the ROM. The single-step suite's model carries (d,X) where the ROM wraps, so
  its emulation-mode (d,X) cases whose pointer starts at a page's last byte are named exceptions.
- Pinned by: `gilyons_cpu_tests_on_the_minimal_bus`, which requires `cputest-full` to pass with no failing test, and
  `the_cpu_through_the_whole_suite`, which requires those (d,X) cases, and only they, to fail.

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
- ~~Conclusion: the datasheet's list; argued. The 80 cases are named exceptions. Open.~~
- **Settled 2026-09-30 by gilyon's `cputest-full`** (the second rung). Its test 0277 runs `jsr ($FFFF,x)` in emulation
  mode at S=$0100 and expects the return address at $00FF and $0100 and S=$01FE: the datasheet's rule. Its test 03D9
  runs `plb` at S=$01FF and expects DBR from $0200; its README names this as undocumented, "despite the fact that the
  CPU manual doesn't list it". *Measured:* VenusRT on the datasheet's list passed 0277 and failed 03D9.
- Conclusion, final: in emulation mode JSR (a,X) addresses S in 16 bits, as the datasheet lists, and so does PLB,
  which the list omits; S is returned to page 1 after each. Measured by the ROM. The suite's 43 JSR (a,X) cases at
  S=$0100 are named exceptions; its PLB cases agree with the ROM and are no longer exceptions.
- Pinned by: `gilyons_cpu_tests_on_the_minimal_bus`, and `the_cpu_through_the_whole_suite`, which requires the 43
  cases, and only they, to fail in `fc.e` once D-2's pins are set aside.


Two readings of the oracles were made at stage 0 that a later reader might take for disputes. They are not: each is
about what a suite's file format means, settled by the suite's own data and README, and neither says anything about
the hardware. They are recorded in `VenusRT_Native.md` §2.3: the SingleStepTests 65816 cases of MVN and MVP stop at
100 cycles in the middle of a move, and the SPC700 suite records dummy reads of memory a case does not list without
their data.

### D-4. Cartridge header: Batman: Revenge of the Joker (U) is a LoROM cartridge, though its only well-formed header is at the HiROM place
- Opened: 2026-09-30, at stage 2 step 1, by the header check over the player's library (826 SNES files, read by header
  only, each copied to scratch). VenusRT chose HiROM: at $00FFC0 the file has a readable title, map mode $31, a
  complement and checksum that agree, and a checksum ($FDBC) equal to the sum of the whole file; at $007FC0 it has 64
  zero bytes. Mesen, through the probe's log of its own loader, chose LoROM with map mode $00.
- Documents read: fullsnes, "SNES Cartridge ROM Header" and "SNES Memory Map". Both say where a header is for each
  map and what its fields mean; neither gives a rule that picks LoROM for a file whose LoROM header is empty.
- Test ROM: the game itself, run as a black box. *Measured:* with the HiROM choice VenusRT never leaves its first
  loop ($00:8024) and its WRAM at frame 30 is empty; with the HiROM-place header blanked, so that its scorer takes
  LoROM, its WRAM at frame 30 equals Mesen's in 131,069 of 131,072 bytes. Mesen draws mode 7 by frame 600.
- Referee: not read. Mesen's source: none; its outputs only.
- Conclusion: the cartridge is LoROM; measured. VenusRT's scorer is wrong on this file, and how to fix it without
  a list of titles is not settled: the rule owed is one that weighs a candidate by where its reset handler leads,
  checked against all 826 files before it replaces the present scorer. Open in that sense; no code changes here.
- ~~Pinned by: nothing yet. The library check of `VenusRT_Native.md` §12.3 is the test that will.~~
- **The rule, 2026-09-30, at stage 2 step 3.** *Measured first, over all 826 library files (scratch copies):* each
  candidate header's reset handler was run for 4,000 instructions under its own map, counting its writes to the I/O
  registers and whether it ran into BRK, COP, STP or unmapped memory. Of the candidates the fields reject, 819 of 826
  crash, and of those they choose, 4. The write count alone separates nothing (337 chosen handlers write fewer than 8
  registers), and a rule that added evidence to the score would turn Street Fighter Alpha 2 to HiROM, whose rejected
  candidate writes 81 registers. Batman is the one file where the chosen handler does nothing (1 write, no crash)
  and another's works (9 writes, no crash).
- Conclusion, final for this file: when the fields' choice has a handler that neither crashes nor writes more than
  one I/O register in 4,000 instructions, and another candidate's handler does not crash and writes at least 4, the
  other is taken. Measured: over the library it changes Batman alone, to LoROM, as Mesen has it and as the game
  runs. The three constants are calibrated on these 826 files and on nothing else; a file outside them that breaks
  the rule reopens this entry.
- Pinned by: `the_reset_handler_decides_when_the_fields_choose_a_handler_that_does_nothing` in `cart.rs`, on a
  synthetic image of Batman's shape, and the library check of `VenusRT_Native.md` §14.5.

### D-5. Cartridge map: map mode $x2 without an S-DD1, which Mesen calls ExLoROM
- Opened: 2026-09-30, by the same check: Street Fighter Alpha 2 (U), 4 MiB with an S-DD1, and two dumps of Test Drive
  II: The Duel (U), 1 MiB with no coprocessor, all with map mode $32. Both engines read the same header; VenusRT
  maps them as LoROM and Mesen's loader names them ExLoROM.
- Documents read: fullsnes, "ROM Speed and Map Mode" (mode 2 is "LoROM/32K Banks + S-DD1", "mappable") and "SNES
  Memory Map" (BigLoROM and SpecialLoROM, which map further LoROM banks above 2 MiB).
- Test ROM: none. Below 2 MiB the two maps decode banks $00-$3F and $80-$BF alike (argued), so a 1 MiB cartridge can
  differ only where it reads banks $40-$7D or $C0-$FF; nothing run here touched them.
- Referee: not read. Mesen's source: none.
- Conclusion: open. The S-DD1 is outside parity (`VenusRT_Plan.md` §4.2). Test Drive II is settled when the goldens of
  stage 6 run it against Mesen.

### D-6. Power-on: where in line 0 the CPU's first instruction begins
- Opened: 2026-09-30, at stage 2 step 3, by the probe's CPU trace of four games against VenusRT's own, instruction by
  instruction. Registers and each instruction's master clocks agree for the first 1,994 instructions of Super Mario
  World, and the clocks' running total is identical (50,222 at record 1,990); but Mesen's 40-clock refresh falls in
  an instruction about 175 clocks earlier than VenusRT's, line after line, and Super Metroid's 44th instruction reads
  $4212 with the H-blank flag set in Mesen and clear in VenusRT.
- Documents read: fullsnes, "SNES Timing H/V Events": "H=0, V=0, F=0: SNES starts at this time after /RESET", and
  the refresh at H=133.5; anomie's timing document: the refresh "begins at 538 cycles into the first scanline of the
  first frame". The datasheet's reset is seven cycles.
- Test ROM: none. Both observations are explained by one cause, Mesen's CPU beginning its first instruction 130 to
  180 clocks later in the line than a reset that starts at H=0, or by a refresh placed earlier; the CPU cannot tell
  the two apart, and the PPU's H-counter latch of stage 3 can.
- Referee: not read. Mesen's source: none.
- Conclusion, as it stood at stage 2: fullsnes's start at H=0, V=0 and the documents' refresh point; argued. Open
  until stage 3's H-counter.
- Measured 2026-09-30, at stage 3 step 1, with `$2137`, `$213C` and `$213D` built. Sour's `timing_test` latches the
  counters four times before it uses HDMA and prints each as H and V. VenusRT, whose reset sequence begins at clock
  0 of line 0, prints `$00AD.$0001`, `$0041.$0002`, `$00C0.$0004` and `$0050.$00DB`; Mesen prints `$00CD.$0001`,
  `$0061.$0002`, `$00E0.$0004` and `$0070.$00DB`. Every row is 32 dots, 128 master clocks, later in Mesen. The
  `power_phase` example moves VenusRT's power-on position by a number of clocks; of 0, 120, 124 to 132, 136
  and 140, 128 alone reproduces all four of Mesen's rows; 127 and 129 each miss two or more. The prediction of stage 2, one cause and a size of 130 to 180 clocks,
  holds in kind and is retired in size: the cause is the CPU's start, 128 clocks, and the refresh needs no moving.
  Stage 2's two observations, the refresh's instruction and Super Metroid's `$4212`, were not measured again at the
  new position.
- What the documents and the test ROMs say of the number: nothing. fullsnes puts the counters at H=0, V=0 "after
  /RESET" and does not say how long after that the CPU's first cycle is; anomie's timing document credits the
  observation "that the SNES returns to a known timing position on reset" and does not give the position. Sour's
  `timing_test` carries no expected values (its README calls it work in progress, and the row is printed, not
  graded), `test_dmatiming` latches in its NMI handler after `libclock` has sought a known dot, as every test of
  `snestest_082506` does, and so cannot see where the machine began. Run at 0 and at 128 for 1,200 frames, the
  text 304 images leave in VRAM differs in Sour's three images and in five of PeterLemon's SPC700 images, which
  need an APU that is still a stand-in, and in no self-graded test: the corpus holds no hardware verdict on the phase.
- Referee: not read. Mesen's source: none.
- Conclusion: unchanged, and still argued: the reset sequence begins at H=0, V=0. Mesen's 128 clocks is an
  observation with one sample and no document behind it; adopting it would make four printed rows equal and prove
  nothing. Open. It would be settled by `timing_test`'s first row on a console: `$00AD` for a start at clock 0,
  `$00CD` for a start 128 clocks later, and anything else for neither. Until then it bounds every frame-exact
  differential with Mesen by what 128 clocks of phase change, and `power_phase` measures that for a ROM.
- Pinned by: nothing; `power_phase` (an example of the crate) reproduces the measurement.

