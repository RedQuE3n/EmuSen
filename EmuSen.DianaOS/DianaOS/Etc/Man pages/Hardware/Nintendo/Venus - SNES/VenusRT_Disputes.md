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
- Added the same day: undisbeliever's `reset-position-test` exists to answer this question. Its reset handler
  latches the counters as its first instruction and prints them. Mesen prints OPHCT `$0035`, VenusRT `$0015`, and
  VenusRT at 128 clocks prints `$0035`, the same 32 dots. Its source records no console result. It is the cheaper
  of the two ROMs that would settle the entry: `$0015` on a console for a start at clock 0, `$0035` for Mesen's.
- Measured again 2026-10-01, after D-9 moved `$2137`'s latch four clocks earlier. VenusRT at clock 0 now prints
  Sour's first row as `$00AC` and `reset-position-test`'s OPHCT as `$0014`. No offset reproduces all four of Sour's
  rows any longer: 132 clocks gives the first three and `reset-position-test`'s `$0035`, 130 the last three. The
  128 of 2026-09-30 is retired; it agreed with Mesen only while VenusRT latched a dot late. The two latch points,
  Mesen's and VenusRT's, now differ in a way four rows cannot separate from the start's phase. Unchanged: the rule
  and what would settle it, now `$0014` for a start at clock 0.

### D-7. PPU: master brightness N scales a colour component c to c×(N+1)/16, rounded down
- Opened: 2026-09-30, at stage 3 step 1, by PeterLemon's `RedSpace9BitHDMA` at frame 300. The ROM writes a backdrop
  colour and a brightness for every line by HDMA; the pairs come from its own `Gradient.py`, which was run to list
  them. VenusRT's red component equals the formula on 219 of 224 lines; Mesen's is one lower on 59 of them (colour
  26 at brightness 12: the formula gives 21.125, so 21; Mesen shows 20), and never higher. The other five lines
  are the table's lines at brightness 0, black in both engines, as fullsnes's "0=Screen Black" has it.
- Documents read: fullsnes, "PPU Registers, INIDISP": "N=1..15: Brightness*(N+1)/16"; anomie's register document,
  `$2100`: "F=max, 0=off", no formula.
- Test ROM: `RedSpace9BitHDMA`, as above. Its folder holds two PNG screenshots, 8-bit colour, one named for colour
  emulation; the README names emulators as the way to run the ROMs, so neither image is taken as a console's
  output.
- Referee: not read. Mesen's source: none.
- Conclusion: fullsnes's formula, rounded down; argued from the one document that gives a formula. Mesen's lower
  values are an observation with no document behind them. Open: a console capture of this ROM, or of any ROM that
  pairs one colour with several brightnesses, would settle it.
- Pinned by: nothing yet. The picture comparison of `VenusRT_Native.md` §15 reproduces it.

### D-8. NMI: the CPU's /NMI falls at HC=6 of the first V-Blank line, four clocks after `$4210` bit 7 sets at HC=2
- Opened: 2026-09-30, at stage 3 step 1. With a picture, 28 of the corpus's standing ROMs turn out to grade
  themselves through the backdrop, colour 0 blue (`$7C00`) for a pass and red (`$001F`) for a failure, the protocol
  of byuu's `snestest_082506` and the `blobs` and `nmi_irq` folders. Mesen passes all 28; VenusRT passes 11 and
  fails 17, at a power-on phase of 0 clocks and of 128 alike (D-6 is not the cause). Each keeps its test number in
  SRAM at `$700000`. `snestest_082506/test_nmi`, and its copies in `blobs` and `nmi_irq/nmi_pf`, stop at test 1.
- Documents read: fullsnes, "SNES Timing H/V Events": "H=0.5, V=225 set NMI flag". anomie's timing document,
  "Interrupts": the timer "will set its NMI output low at H=0.5", and "the actual check occurs just before the final
  CPU cycle of the instruction". VenusRT sets the flag and the CPU's edge together at HC=0, and samples the edge at
  the start of each cycle, so its check is anomie's and its timing of the edge is neither document's.
- Test ROM: `test_nmi.asm`'s header, written for the purpose (byuu, 2006-07-25), states as results: `$4210` bit 7
  sets at HC=2; NMI "goes low at HC=6", so a final cycle that begins at HC=4 does not see it and one that begins at
  HC=6 or later does; reading `$4210` at HC=2 or 4 leaves the bit set for the next read, at HC=6 or later clears
  it; a `$4200` write that disables NMI prevents it only if its bus cycle begins by V=224, HC=1362. Test 1 runs SEC
  with its final cycle at V=225, HC=4 and expects the NMI after the next instruction; VenusRT takes it after SEC
  (the handler finds `$EA`, NOP, at its return address, where the test wants `$18`, CLC).
- Referee: not read. Mesen's source: none.
- Conclusion: open, and expected to be settled by the test ROM, which is the order of recourse's second source and
  more precise than the two documents: the flag at HC=2 as fullsnes and anomie say, the CPU's line at HC=6. Not
  implemented in this step, which is the PPU's: it changes the interrupt timing of stage 2, and the 17 ROMs and the
  CPU trace against Mesen are its measure. The other sixteen's first failures were not read: `irq` at test `$16`,
  `test_irq`, `demo_irqtest` and `demo_irq` at 6, `test_irqb` at 7, both `test_hdma` at 2, `test_hdmatiming` at
  `$52`, `nmi` at `$1F`; `test_dma` and `test_hdmasync` leave 0, and the two `test_dmavalid` write `$AA` there and
  use the address for something else.
- Pinned by: nothing yet.
- Read again 2026-10-01, before the code, at stage 3 step 2. The header's results are a console's (the header states
  them as the console's behaviour, and the ROM grades itself against them), and they are taken as the document for this rule. The header
  names times in two frames, and the test bodies say which: an interrupt check is placed at the *start* of the
  instruction's last bus cycle (tests 1 and 2: SEC's last cycle starting at HC=4 sees no NMI, at HC=6 sees it), but a
  read of `$4210` is placed two clocks *into* its six-clock cycle (tests 15 to 18: the read cycles start at
  V=224 HC=1362 and at V=225 HC=0, 2 and 4, and the header calls the last three "HC=2", "HC=4" and "HC>=6"). Writes
  to `$4200` take effect at the end of their cycle (tests 19 to 26: a disabling write whose cycle ends at HC=4
  prevents the NMI, one that ends at HC=6 does not). VenusRT applies a read's and a write's effect at the end of its
  cycle, so in its own frame the header's rules are one time, HC=6 of line 225:
  - at HC=6 the `$4210` flag sets and, if `$4200` bit 7 is set, /NMI falls; a cycle that ends exactly at HC=6 sees
    both, and a write ending there applies after them;
  - a read of `$4210` whose cycle ends before HC=10 of line 225 returns the flag without clearing it (the header's
    "HC=2 or HC=4");
  - the check stays where it was, at the start of the instruction's last cycle.
  The rule, in hardware terms: the flag is visible from HC=2 and the line falls at HC=6, and a read before HC=6 does
  not clear the flag; argued from the header's console results, and implemented in the end-of-cycle frame above.
- Implemented 2026-10-01 (with D-9, which its seek needed, and a one-cycle hold for an edge a `$4200` write makes,
  the header's version-1.1 case: the check at the very next cycle's start does not see it). **Measured:** all three
  copies of `test_nmi` (`snestest_082506`, `blobs`, `nmi_irq/nmi_pf`) pass every test, 1 to 27, on VenusRT; before,
  they stopped at test 1. Of the 25 ROMs that grade by the backdrop, VenusRT now passes 12 and fails 13 (it
  passed 8 of them before: these four are the difference); the 13 are IRQ, HDMA and DMA tests (D-10 and later steps). gilyon's two
  CPU ROMs still pass on the machine. **Settled**, measured on the test ROM written for it.
- Pinned by: `the_nmi_flag_sets_at_hc_6_of_line_225_and_an_early_read_leaves_it` in `scpu.rs`, and the three
  `test_nmi` rows of `VenusRtBaseline.tsv`.

### D-9. PPU: a read of `$2137` latches the counters four master clocks before its bus cycle ends
- Opened: 2026-10-01, at stage 3 step 2, by `test_nmi` after D-8's rule went in. Its test 1 and 2 SECs were meant
  to begin their last cycle at V=224 HC=1360 and 1362 (the header's cycle lists); on VenusRT they began at 1356 and
  1358, four clocks early. The ROM reaches those points through `libclock`'s `seek_frame`, which latches the counters
  with `$2137` twice and subtracts the dot it reads from a fixed count, so a latch one dot late lands the seek one
  dot early. VenusRT latched at the end of the read's cycle, the frame in which it applies every read and write.
- Documents read: anomie's timing document, its preface: the counter "may also be latched by writing 0 to $4201
  bit 7: this will latch 1 dot later than if the same memory access cycle were reading $2137". fullsnes, "OPHCT",
  gives no point within the cycle.
- Test ROM: `test_nmi` and its `libclock`, as above. With the read's latch four clocks before the cycle's end, the
  SECs begin at 1360 and 1362 and `nmi_irq/nmi_pf/test_nmi` passes. The WRIO latch stays at the end of its write's
  cycle, which with this rule is anomie's "1 dot later".
- Referee: not read. Mesen's source: none.
- Conclusion: a `$2137` read latches the dot of its cycle's end less four clocks; argued from anomie's relative
  statement and measured on the test ROM. It moves every latched H counter by one dot, so D-6's figures are read
  again after it. Only `$2137` is moved; whether other reads sample early, as D-8's `$4210` reads do in the same
  frame, is not settled by this entry.
- Pinned by: `test_nmi` in the corpus, once the runner reads the backdrop.
- Measured 2026-10-01: with the latch moved, `test_nmi`'s SECs begin where the header says, and the ROM passes
  (D-8). Settled for `$2137`, measured.

### D-10. IRQ: `$4211`'s flag sets at HTIME×4+14 (10 for HTIME 0), the CPU's /IRQ falls four clocks later
- Opened: 2026-10-01, at stage 3 step 2, by `test_irq` (byuu, 2005-10-01) stopping at its test 1, and the other
  IRQ tests among D-8's 17.
- Documents read: anomie's timing document, "Interrupts": `$4211` bit 7 "gets set 1374 master cycles after dot 0.0
  of the previous scanline" for H=0, "otherwise ... 14+H*4", which is VenusRT's `irq_point`, used until now for the
  flag and the line alike. fullsnes, "SNES Timing H/V Events": "H=HTIME+3.5 H-IRQ", "H=2.5 V-IRQ", in dots.
- Test ROM: `test_irq`'s header, the same author's console results in the form of `test_nmi`'s: "test when IRQ
  trigger occurs: V=VTIME,H=(HTIME)?(HTIME*4+18):(14)"; "test when $4211.d7 is set: (HTIME*4+14):(10)"; reading
  `$4211` at that point or two clocks after leaves the flag set, any later clears it; a `$4200` write prevents the
  IRQ only if it is earlier than the trigger. Its tests 1 and 2 place SEC's last cycle at V=225 HC=12 (no IRQ
  after SEC) and HC=14 (IRQ after SEC).
- Referee: not read. Mesen's source: none.
- Conclusion: the shape of D-8. The flag sets at anomie's point and the line falls four clocks after it; a read
  whose sampling point is within four clocks of the flag's setting does not clear it. In VenusRT's end-of-cycle
  frame, where a read's sampling point is its end less four: one event at `irq_point` + 4 sets the flag and the
  line, and a `$4211` read whose cycle ends less than eight clocks after `irq_point` leaves it set. Argued from
  the header's console results.
- Pinned by: `test_irq` in the corpus, once the runner reads the backdrop.
- Implemented 2026-10-01, with one more piece the header requires: the IRQ the check saw is taken though the
  instruction's final cycle disables the line (its tests 9 and 10: a disabling `$4200` write whose cycle begins at
  the trigger point does not stop the IRQ). VenusRT had also required the flag to be set still at the instruction's
  end, a remnant of stage 2's first check. **Measured:** `blobs/test_irq` passes all its tests; it stopped at test
  1. `test_irq4209` passes as before. `test_irq4200`, `test_irqb`, `irq`, `demo_irq` and `demo_irqtest` still fail,
  at tests not read. Settled for the trigger and flag points, measured; open for those five.

### D-11. OAM: the internal address reloads at the start of line 225 outside forced blank, and when forced blank ends during that line
- Opened: 2026-10-01, at stage 3 step 2, by reading the two documents for sprite priority, which takes the first
  sprite from the internal address. They disagree. fullsnes, "OAMADDL/OAMADDH": the reload occurs "at begin of line
  225/240, but only if not in Forced Blank mode", and "also when deactivating forced blank anytime during the first
  scanline of vblank". anomie's register document, `$2102`/`$2103` and "SPRITES": "The reload also occurs on a 1->0
  transition of $2100.7", with no line named; its timing document, "OAM RESET", reports byuu as seeing "the reset
  occurs on any 1->0" transition as well.
- Test ROM: none in the corpus isolates it.
- Referee: not read. Mesen's source: none.
- Conclusion: fullsnes's rule, as stage 3 step 1 built it; argued only from its being the narrower statement and the
  one built first. Open: a ROM that ends forced blank mid-frame after writing OAM, and shows the first sprite's
  priority, would settle it.
- Pinned by: `oam_and_cgram_latch_their_low_bytes_and_the_counters_latch_on_2137` (the line-225 reload).

### D-12. OAM: a write during active display lands where the internal address points
- Opened: 2026-10-01, at stage 3 step 2, by the step's scope. fullsnes, "OAMADDL/OAMADDH": "During rendering, the
  PPU is destroying the Address register (using it internally for whatever purposes)"; its "PPU Memory Accesses"
  notes that Mario Kart uses forced blank to change OAM mid-screen. anomie: the address "is invalidated during the
  rendering of a scanline; this invalidation is deterministic, but we do not know how or when the value is
  determined", and a write in H-blank "CAN" happen but "the actual OAM byte written will probably not be what you
  expect".
- Test ROM: none in the corpus.
- Referee: not read. Mesen's source: none.
- Conclusion: not modelled. Both documents say the address is replaced during rendering and neither says by what, so
  any rule would be invented; VenusRT writes where the address points, as in forced blank, and names it a known
  difference. The priority rotation reads the same address (anomie: the first sprite comes from the address "not
  affected by OAM Address Invalidation"), which is therefore right only while nothing invalidates it.
- Pinned by: nothing.

### D-13. Colour math: the front-most sub-screen pixel is used whatever its priority against the main pixel's
- Opened: 2026-10-01, at stage 3 step 3, by reading the documents before the colour-math code. fullsnes, "SNES PPU
  Color-Math": math occurs "only if the front-most Sub Screen pixel has same or higher (XXX or is it same or lower
  -- or is it ANY priority?) priority than the Main Screen pixel", the question its own. anomie's register
  document, "RENDERING THE SCREEN", "Color Math": "Add the corresponding subscreen pixel, or the fixed color if
  it's the subscreen backdrop", and its three-step rendering list, with no priority condition.
- Test ROM: none in the corpus isolates it.
- Referee: not read. Mesen's source: none.
- Conclusion: anomie's, any priority; argued from fullsnes's sentence being a question and anomie's procedure being
  stated. Open until a ROM puts a sub-screen pixel behind the main pixel's priority with math on.
- Pinned by: nothing yet.

### D-14. HDMA, indirect mode: a line count of zero still loads the pointer's high byte, clears its low byte, and advances the table by two
- Opened: 2026-10-01, at stage 3 step 3, by `snestest_082506/test_hdma` (byuu, 2005-10-21) failing at its test 1, one
  of the 13 backdrop failures. The test runs an HDMA init on an indirect channel whose table begins `00 AA BB`, and
  expects DASxL `$00`, DASxH `$AA`, A2AxW the table start plus 2 and NTRLx `$00`. VenusRT leaves DASx as the program
  wrote it ($5555) and A2AxW at the start plus 1: it loads the pointer only for a non-zero count.
- Documents read: fullsnes, "HDMA Table Formats" (a `00h` count terminates the channel; nothing on the pointer) and
  `43x5h`-`43x7h`; anomie's timing document, "HDMA" (24 master clocks for an indirect channel's init, 16 to load a new
  indirect address; nothing on a terminator). Neither covers the case.
- Test ROM: `test_hdma`'s test 1 and its comment, the author's console results: after loading NTRLx, "if the value is
  zero, then it will load the next byte from the table into DASxH and clear DASxL. A2AxW is equal to A1TxW plus
  two", and "it won't actually read from the table three times -- only two reads actually occur, as determined by
  checking OPHCT".
- Referee: not read. Mesen's source: none.
- Conclusion: the test's rule, in every table load (the init and a line's reload alike, since the documents describe
  one load); the second read costs 8 master clocks, the one read it is. Argued from the test ROM's console results.
- Pinned by: `test_hdma` in the corpus, once it passes its test 1.
- Implemented 2026-10-01. **Measured:** at the failure the five registers are now the test's ($00, $AA, $02, $00,
  $00), where before DASx kept $5555. The test still stops at test 1, on its next check: OPHCT latched just after the
  init reads `$3A` where the test wants `$38`, two dots late. The test's own comment says the init's timing differs
  between console revisions ("1/1/1 and 2/1/3 SNES units") and seeks a DMA phase to cope; the init's cost was not
  changed to fit one number. Settled for the registers; the init's timing is open.

### D-15. Mosaic: the first row of blocks starts at the top of the picture, and a size change waits for the current block to end
- Opened: 2026-10-01, at stage 3 step 4, by the two documents before the mosaic code. fullsnes, "MOSAIC": "Vertically,
  the first block is located on the top of the TV screen. When changing the mosaic size mid-frame, the hardware does
  first finish current block (using the old vertical size) before applying the new vertical size", implemented as an
  index within the block subtracted from the vertical scroll. anomie's register document, `$2106`: the first row of
  squares is on the "starting scanline", which "if this register is set during the frame ... is the current
  scanline", with his own note "XXX: It seems that writing the same value to this register does not reset the
  'starting scanline', but which changes do reset it?".
- Test ROM: none in the corpus writes `$2106` mid-frame; PeterLemon's `MosaicMode3` sets it once.
- Referee: not read. Mesen's source: none.
- Conclusion: fullsnes's, a counter that starts at the first picture line and takes a new size when it wraps;
  argued from its describing a mechanism where anomie records a question. The two agree for a size set in V-Blank.
  Open for mid-frame changes.
- Pinned by: `mosaic_repeats_the_blocks_first_pixel_and_first_line` in `ppu.rs`, for the agreed case.
- Measured 2026-10-01, on the agreed case: the 240p suite's intro fades by mosaic, one size a frame, written in
  V-Blank. Mesen runs it ten frames after VenusRT (the sound CPU's stand-in answers at once), and with that offset
  its frames 60, 64 and 68 equal VenusRT's 50, 54 and 58 in every pixel, at sizes 5, 9 and 13. An experiment that
  counted the rows from line 0 instead of the first picture line matched none. Mid-frame changes stay open.

### D-16. Offset-per-tile: visible tile T of the background takes its offsets from visible tile T-1 of BG3, and keeps the low three bits of its own scroll
- Opened: 2026-10-01, at stage 3 step 4. fullsnes's section is "Under construction (see Anomie's docs for now)".
  anomie's register document, "Mode 2", gives a per-pixel formula headed "Hopefully these calculations are right":
  the BG3 entry at `((X-8)&~7)+(BG3HOFS&~7)`, and a replaced `HOFS = (HOFS&7) | ((X&~7) + (Hval&~7))`; and then prose:
  "number the visible tiles in BGn from 0-32, and the 'visible' tiles in BG3 the same way. BGn tile 0 is offset as
  normal, then for 1<=T<33 BGn tile T gets the offset data from BG3 tile T-1. It doesn't matter whether or not the
  tiles actually align". When the background's scroll is not a multiple of 8 a visible tile straddles two values of
  `X&~7`, so the formula read per pixel splits one tile between two columns, and the prose does not.
- Test ROM: `snestest_082506/test_opt` grades HDMA and register behaviour, not the picture (it passed before any
  offset-per-tile existed). No ROM in the corpus draws with offset-per-tile; the bench games that use it (Yoshi's
  Island) do not reach it yet.
- Referee: not read. Mesen's source: none.
- Conclusion: the prose, per visible tile: with T the pixel's visible tile, `(X + (BGnHOFS&7)) / 8`, tile 0 uses the
  registers; tile T reads BG3's map at column `(T-1)*8 + (BG3HOFS&~7)`, rows BG3VOFS and BG3VOFS+8 (mode 4: one
  entry, bit 15 choosing which it is); a valid horizontal value replaces the scroll with `(Hval&~7) | (BGnHOFS&7)`,
  a valid vertical value replaces BGnVOFS whole. Argued; **no oracle**, so it is built and unmeasured.
- Pinned by: `offset_per_tile_takes_each_visible_tiles_scroll_from_bg3` in `ppu.rs`, which pins the reading, not
  the hardware.
- **An oracle found, 2026-10-01, at stage 3 step 5.** lidnariq's `ppubusact` (higan collection) changes BGMODE by
  HDMA every 32 lines, modes 0 to 6, and its README says "HDMA changes BG3VOFS to get offset-per-tile to be
  visible". At frame 3600 its mode 2 and mode 4 bands equal Mesen's picture in every pixel; built without
  offset-per-tile, the same bands differ on 3,294 and 3,300 pixels. So the ROM exercises the rule, and this reading
  agrees with Mesen's output there (Mesen's output is an observation, not a hardware result; the ROM's author posted
  bus traces, not a picture). Mode 6's band: 2,295 differing half-pixels without offset-per-tile, 86 with it, in
  two small clusters; not read further. **Measured against Mesen for modes 2 and 4; open for mode 6's 86.**

### D-17. DMA: an HDMA run or init on a channel ends a general DMA on that channel where it stands
- Opened: 2026-10-01, at stage 3 step 4, by `snestest_082506/test_dma` (byuu, 2006-07-27) failing at its test 1. The
  test starts a 128-byte DMA on a channel that HDMA is also enabled on, so that the line's HDMA point falls inside
  it, and expects the count left non-zero; VenusRT runs the DMA to its end ($4305 = 0).
- Documents read: anomie's timing document, "HDMA": "HDMA takes priority over DMA", and nothing on what becomes of
  the DMA. fullsnes, "SNES DMA and HDMA Notes": nothing on the overlap.
- Test ROM: `test_dma`'s four tests and their comments, the author's console results. Tests 1 and 2: "if HDMA run
  [init] occurs on the same channel as an active DMA channel, the HDMA run [init] will kill the DMA transfer,
  leaving $43x5 != 0", with "$4305 = ~#$37 on hardware" and "~#$41". Tests 3 and 4, DMA on channels 0 and 1 with
  HDMA on both: "HDMA run will kill DMA on both channels, 0 *and* 1; not just 0", with the registers read from a
  console: channel 0's address advanced by `$47` and its count `$39` left, channel 1's address and count untouched;
  for the init, channel 0's A2AxW is its A1TxW as the DMA left it, plus one.
- Referee: not read. Mesen's source: none.
- Conclusion: when HDMA's init or a line's run takes its channels, a general DMA in progress on one of them stops
  with its address and count as they are, and a pending DMA on another of them does not start. The DMA's address
  and count are in the registers as each byte moves, so the init reloads the table address from what the DMA
  reached. The tests enable HDMA on every channel the DMA uses, so they do not say what happens to a DMA channel
  HDMA does not take; it is left running, the narrower reading, argued and open.
- Pinned by: `test_dma` in the corpus.
- Implemented 2026-10-01. **Measured:** `test_dma` passes all four tests (blue backdrop); it failed at test 1.
  After test 4 channel 1's twelve registers equal the console's as the test's comment lists them; channel 0 stopped
  four bytes earlier than the console's ($4302 `$3F` and $4305 `$41` against `$43` and `$3D`), which is the DMA's
  start within the line, set by the test's own NOPs, and inside what the test accepts. Settled for the channels HDMA
  takes; open for a DMA channel it does not.

### D-18. DMA between WRAM and its own port `$2180`: nothing reaches the port, and a read of it writes `$00`
- Opened: 2026-10-01, at stage 3 step 4, by the two `test_dmavalid` copies (byuu, 2008-03-03) failing. VenusRT moved
  the bytes both ways.
- Documents read: fullsnes, "WMDATA", "DMA Notes": "WRAM-to-WRAM DMA isn't possible (neither in A-Bus to B-Bus
  direction, nor vice-versa)", the chip being "unable to process both at once". It does not say what is left behind.
- Test ROM: `test_dmavalid`'s tests 2 and 3 and their comments, a console's results. WRAM to `$2180`: the port's
  address is not incremented, the write does not occur, the DMA's address and count move as usual and its time is
  spent. `$2180` to WRAM: the port's address is not incremented, and "DMA write did occur, but wrote unknown value
  (not MDR ...)", the byte the console showed being `$00`; the test accepts anything but the old value.
- Referee: not read. Mesen's source: none.
- Conclusion: fullsnes's rule with the test's details; in the port-to-WRAM direction the byte written is `$00`, the
  one value a console is recorded as showing. Argued for that value, measured for the rest.
- Pinned by: the two `test_dmavalid` rows of the corpus.
- Implemented 2026-10-01. **Measured:** both copies pass (blue backdrop). The eight bytes test 2 stores are the
  console's as its comment lists them, `3F 55 55 00 14 7E 00 00`, and the V counter latched after the DMA is the
  console's `$36`; the H counter is `$D4` against its `$CE`, inside the test's four lines of latitude.

### D-19. Hi-res: colour math reaches the sub screen's half-pixels through the main pixel before them
- Opened: 2026-10-01, at stage 3 step 5, by the documents before the hi-res code. fullsnes, "BGMODE": "Mode 5/6 don't
  support screen addition/subtraction"; its "Hires and Pseudo 3-Layer Math" says COLDATA's addition applies to both
  screens' half-pixels. anomie's register document, "Color Math": "In hires modes, color math is applied to the
  visible subscreen pixels as well ... look at the previous main-screen pixel ... If no math was applied to that
  pixel, don't math this subscreen pixel either. If the fixed color was added/subtracted, add/subtract the fixed
  color. And if a pixel from the subscreen was added/subtracted, add/subtract that main-screen pixel (the original
  value before math). What happens to the subscreen pixel at the left edge of the screen is unknown"; and for the
  colour window, "we use the previous main-screen pixel to determine whether the color window effect should be
  applied to a subscreen pixel".
- Test ROM: PeterLemon's four `HiColor64PerTileRowPseudoHiRes` ROMs use pseudo-hi-res with the sub screen and
  colour math; they are the measure, through Mesen's picture.
- Referee: not read. Mesen's source: none.
- Conclusion: anomie's rule, which is a procedure where fullsnes has a sentence; the fixed colour's case is common to
  both. The left edge's sub half-pixel is taken as unmathed and unclipped, his unknown. Argued; the four ROMs'
  result is recorded when measured.
- Pinned by: nothing yet.
- Measured 2026-10-01, at stage 3 step 5, on PeterLemon's four pseudo-hi-res ROMs with the sub screen added and
  halved (CGADSUB `$61`). The rule as built, anomie's sentence read literally (the sub half-pixel mathed with "that
  main-screen pixel (the original value before math)"), differs from Mesen's picture on 2,691 to 27,099 of each
  ROM's half-pixels. A variant tried and not kept: the sub half-pixel mathed with the main pixel before it *after*
  its math, and the left edge's taken as mathed with black, equals Mesen's picture in every half-pixel of all four.
  The two readings differ only in that operand. anomie's words favour the first; the sentence can be parsed for the
  second ("the original value before math" read as the sub pixel's own). The built rule is kept, being the
  documents', and the entry is **open**: it is the first case in which the documents give a rule and Mesen's
  output disagrees with it, and the next source in the order of recourse is the SNES_MiSTer RTL, in a dispute step
  of its own.
- **Referee, 2026-10-01, at stage 3 step 6**, a dispute step that changed no code. Read: `Venus_Referee.md` §0 only
  (the PPU is "real support", from anomie's and fullsnes's register documents, apart from two mode 7 quirks), then
  SNES_MiSTer `rtl/PPU.vhd` at `c61bfd4`: a search of the file for its hi-res and math signals, and lines 2403-2480,
  the colour-math process. Nothing else of the RTL was opened. What it does, in prose: each dot has a sub-screen
  phase and then a main-screen phase; the colour fetched at each phase is kept as "the previous colour" for the
  next; the math enable, the halving and the clip mask are taken only in the main phase. So at the sub phase of
  dot x the colour mathed with the sub pixel is the main pixel of dot x-1 as fetched from CGRAM, before its math,
  under dot x-1's enable, halving and clip; with the fixed colour where that pixel used it. The previous colour is
  cleared at the line's last dot, so the left edge's sub half-pixel is mathed with black under whatever the
  enables last were.
- **Conclusion: settled for the rule as built.** The referee agrees with anomie's sentence read literally, so the
  documents and the referee agree and Mesen's picture is the one that differs: the four pseudo-hi-res ROMs stay
  different from Mesen on 2,691 to 27,099 half-pixels, and those differences are recorded as Mesen's, not VenusRT's.
  The variant of the measurement above is not adopted. The left edge is open between "unmathed" (built) and the
  referee's "mathed with black under the last enables"; it moves one column of four ROMs and is left as built.
- Pinned by: `the_sub_half_pixel_takes_colour_0_and_the_math_of_the_main_pixel_before` in `ppu.rs`.

### D-20. IRQ: a point past its line's end is not carried over the short line's end or a frame's
- Opened: 2026-10-01, at stage 3 step 5, by `nmi_irq/demo_irq` (and `blobs/demo_irqtest`) failing at test 6's first
  check, with interlace now built. With HTIME 339 the flag's point, anomie's 14+4×339 and the long dots, is past the
  line's 1,364 clocks, and VenusRT raises the IRQ in the next line for the line before; it did so across the short
  line and across the frame's end as well.
- Documents read: anomie's timing document, "Interrupts": "no IRQ will trigger for dot 153 on the short scanline in
  non-interlace mode, and no IRQ will trigger for dot 153 on the last scanline of any frame"; nothing on dot 339.
  fullsnes, "Long and Short Scanlines": the short line has 340 dots of four clocks.
- Test ROM: `demo_irq.asm`'s test 6 and its comment, the author's console results, as lists. Unlatchable: V=240,
  H=339 without interlace on the frame with the field flag set; V=261, H=339 without interlace; V=262 without
  interlace; H=340; V=263 with interlace; V=262 with interlace on the frame with the flag set. Latchable: V=240,
  H=339 with interlace on the flagged frame; V=262 with interlace; H=339 on an ordinary line; V=261 without
  interlace.
- Referee: not read. Mesen's source: none.
- Conclusion: the test's lists. A point past its line's end raises its IRQ in the next line, except after the short
  line and after a frame's last line, where it is lost; the line counts are interlace's (263 lines on frames with
  the flag clear). anomie's two dot-153 exclusions stay beside it. Argued from the test ROM's console results.
- Pinned by: `demo_irq` in the corpus.
- Implemented 2026-10-01, after interlace's line count. **Measured:** `demo_irq` and `blobs/demo_irqtest` pass all
  six tests (blue backdrop); they stopped at test 6's first check. Settled, measured on the test ROM.

### D-21. IRQ: enabling V-IRQ on its line, after its point, raises it at once
- Opened: 2026-10-01, at stage 3 step 5, by `blobs/test_irq4200` failing. The ROM (lost source, disassembled by
  Jonas Quinn) sets VTIME 1 and HTIME 0, then HTIME `$152`, waits into the line, and for ten combinations writes
  `$4200` four times with a disabling write between, recording each IRQ taken; it compares the record with a table.
- Documents read: anomie's timing document, "Interrupts": V-IRQ at "V=VTIME, H=~2.5", and that "when enabling IRQs,
  the IRQ output will go low even if the enable write occurs at the exact cycle when the IRQ is scheduled to
  trigger". fullsnes, "H/V Events": "H=2.5, V=VTIME V-IRQ". Neither says what an enable later in the line does.
- Test ROM: the table. With either HTIME: writing `$20` (V-IRQ) raises an IRQ every time it is written after a
  disable, four times for `$20,$20,$20,$20` and twice wherever it is two of the four; writing `$10` (H-IRQ) or `$30`
  (HV-IRQ) past their point raises none.
- Referee: not read. Mesen's source: none.
- Conclusion: the V comparator alone is a level across its line: a `$4200` write that selects V-IRQ from another
  setting while V=VTIME, past the line's point, sets the flag at that write. H-IRQ and HV-IRQ stay points. Argued
  from the test's table; the table does not say what a `$20` written over `$20` does, and it is left raising nothing.
- Pinned by: `test_irq4200` in the corpus.
- Implemented 2026-10-01. **Measured:** `test_irq4200` passes (blue backdrop), its record equal to the table; the
  other backdrop-graded ROMs are unchanged by it. Settled for the table's cases.

### D-22. SPC700: the order of each instruction's bus cycles where anomie's cycle document leaves it open
- Opened: 2026-10-02, at stage 4 step 1, before the SPC700's code, by reading the two sources side by side.
  anomie's SPC700 cycle document (romhacking.net document 198, revision 1126) gives every addressing mode's cycles
  in order, marking each as "Verified by blargg", "This should be accurate", or open ("2 and 3 could be swapped",
  "Cycles 2-5 could be rearranged", "Or is it Data-IO-IO or IO-IO-Data?", "Order of reading new addr and pushing
  old addr may be wrong", "WTF with all the IO cycles?"); its IO cycles have no address ("??"). fullsnes gives each
  opcode's total cycles and its dummy reads in prose ("Most of the Memory Store opcodes are implemented like ALU
  opcodes (ie. as RMW opcodes, issuing a dummy read ...)"), and no order. SingleStepTests' SPC700 suite records,
  for each of 256,000 cases, every cycle with its address and kind; it shows orders anomie leaves open (POP reading
  its byte last, RET popping after its IO cycles, CALL fetching its target before pushing), and gives its IO cycles
  as either a read of the next program byte or a wait with no address.
- Test ROM: the suite is the measure, and the plan's oracle for the SPC700 (`VenusRT_Plan.md` §6, stage 4).
- Referee: not read. Mesen's source: none.
- Conclusion: where anomie marks a cycle verified, his order is built, and where the suite departs from it the
  case is recorded as a failure, not adopted; where he leaves the order open, the suite's order is built, as the
  only source that states one. The IO cycles' form (a read of the next byte, or a wait) is the suite's. The
  results are recorded by group in `VenusRT_Native.md` §21.
- Pinned by: the SingleStepTests SPC700 run with cycle lists.

### D-23. SPC700: DIV YA,X divides bit-serially, which gives its results when the quotient does not fit in a byte
- Opened: 2026-10-02, at stage 4 step 1, by the SPC700 suite: 760 of the 1,000 DIV cases fail, every one with a
  quotient above 255 or X zero; VenusRT computed the documented case and left the rest.
- Documents read: fullsnes, "SPC700 CPU ALU Commands": "DIV YA,X ... A=YA/X, Y=YA MOD X ... NV..H.Z.", 12 cycles,
  nothing on overflow or on what H means; anomie's cycle document: DIV's cycles only. No document covers it.
- Test ROM: SingleStepTests' DIV cases give the results but not a rule; no ROM in the corpus isolates DIV.
- Referee: `Venus_Referee.md` §0 rates the SPC700 "real support" and names its divider "a 9-iteration bit-serial
  divider, not bsnes's closed form". Read: SNES_MiSTer `rtl/SPC700/MulDiv.vhd` whole (the multiply and divide
  unit), and the nine rows of `rtl/SPC700/MCode.vhd` for opcode 9E; nothing else. What it does, in prose: a
  17-bit register starts as 0, Y, A; each of nine steps rotates it left by one, the bit rotated in inverted when
  the rotated value is at least X shifted left by nine, and then subtracts X shifted left by nine when the new low
  bit is set. A is the register's low eight bits, Y its top eight; V is its bit 8, N its bit 7, Z its low byte's
  zero; H is set when Y's low nibble is at least X's.
- Conclusion: the referee's divider, to be measured on the suite's 1,000 DIV cases. Argued from the referee until
  measured.
- Pinned by: the SPC700 suite's DIV file.
- Implemented 2026-10-02. **Measured:** all 1,000 DIV cases pass with their cycles, and with them the whole suite,
  256,000 of 256,000. Settled, measured: the referee's divider and the suite's recorded results agree on every case.

### D-24. 65816: how many times an interrupt taken while executing from WMDATA ($2180) reads it
- Opened: 2026-10-02, at stage 4 step 1, by KungFuFurby's `test_irqb` once the sound CPU answers its ports: its
  first four cases agree with the ROM's expectations, and the fifth (`jmp $217F`, a CLC read from the APU port and
  the next fetch from $2180) leaves the IRQ handler's record one byte early: the ROM expects WMDATA's address to have
  advanced twice before the handler, and VenusRT advances it once. Mesen passes; C# Venus is not graded on it.
- Documents read: the W65C816S datasheet's Table 5-7, the interrupt sequence (an opcode read of the program
  counter, then an internal cycle); fullsnes on WMDATA; neither says what the S-CPU's bus does on an internal cycle.
- Test ROM: `test_irqb` itself, and its disassembly (`jonasquinn-test-roms/blobs/disassembly/test_irqb.asm`). One
  reading was tried and is rejected by measurement: making the interrupt's internal cycle a second read of the
  program counter moves case 1's latched H from 7 to 9 (cases 1 to 4 execute from $2137, whose read latches the
  counters), so that cycle does not read the bus.
- Referee: not read. Mesen's source: none.
- Conclusion: open. Not a sound-unit rule; left for a CPU-side step with the probe's CPU trace of the fifth case.
- Pinned by: `test_irqb`, case 5.

### D-25. S-SMP: a timer is reset when its CONTROL bit goes from 0 to 1, and a cleared bit only stops it
- Opened: 2026-10-02, at stage 4 step 1, by blargg's `spc_smp`: every test passes up to "Timers/random timer0
  enable", which fails (code 02), with the timers built as fullsnes reads literally.
- Documents read: fullsnes, "00F1h - CONTROL": "0-2 Timer 0-2 Enable (0=Disable, set TnOUT=0 & reload divider,
  1=Enable)". Read literally, a write of 0 clears TnOUT and the stage, and that is what VenusRT built. The text does
  not say whether the reset happens at the write of 0 or at the next enabling write. No other fetched document
  covers it.
- Test ROM: `spc_smp`, `spc_timer`, blargg 2010 `test_timer_stop` and gilyon `spctest`. With the reset moved to the
  write that sets a cleared bit, and a cleared bit only stopping the count (TnOUT kept, still cleared by a read),
  `spc_smp` passes every test ("PASSED TESTS"), and `spc_timer`, `test_timer_stop` and `spctest` still pass. Measured
  2026-10-02 in a trial build, before this entry's code.
- Referee: not read. Mesen's source: none.
- Conclusion: the rule above; measured on the four ROMs.
- Pinned by: `spc_smp`'s "random timer0 enable", and a unit test in `apu/smp.rs`.

### D-26. S-SMP: what blargg's `test_timer_stop2` stops, which the documents' TEST bits do not explain
- Opened: 2026-10-02, at stage 4 step 1, by blargg 2010 `test_timer_stop2`: VenusRT prints 00 and fails, Mesen
  prints 04 and passes. `test_timer_stop` passes on both. Neither has a source.
- Documents read: fullsnes, "00F0h - TEST": bit 0 "Timer-Enable (0=Normal, 1=Timers don't work)", bit 3
  "Timer-Disable (0=Timers don't work, 1=Normal)"; "00F1h - CONTROL" as in D-25. anomie's S-DSP document places
  the timers' first-stage ticks in the DSP's 32-cycle sample loop and notes that "frobbing the SPC700 TEST register
  can change this syncronization". Neither says which stage a TEST bit stops.
- Test ROM: the ROM itself; its SPC700 program is not disassembled yet.
- Referee: `Venus_Referee.md` §0 rates the SMP "real support". Read 2026-10-02: SNES_MiSTer `rtl/SMP.vhd` lines
  210-360, the process holding the I/O registers' writes and the timers, and nothing else. What it does, in prose:
  TEST bits 0 and 3 gate only the second stage (the count toward TnDIV) of all three timers; the first-stage
  prescalers, one shared by timers 0 and 1 and one for timer 2, run on every SPC700 cycle whatever TEST and CONTROL
  hold, and are never reset by them; CONTROL's write clears a timer's count and TnOUT only where its bit goes from 0
  to 1 (as D-25 measured); and the prescalers' step per cycle depends on TEST bits 4 to 7, so that the 8 kHz and
  64 kHz ticks come every 128 and 16 cycles only at TEST's default. VenusRT matches the first three. The fourth is
  the only difference the read found, and it belongs to D-27.
- Conclusion: open, argued to be D-27: no rule in the read process separates `test_timer_stop` from
  `test_timer_stop2` other than TEST's speed bits, so the ROM is to be measured again once D-27 is built.
- Pinned by: `test_timer_stop2`.

### D-27. S-SMP: TEST bits 4 to 7 add waitstates to the SPC700's cycles
- Opened: 2026-10-02, at stage 4 step 1, by blargg 2010 `test_timer_speed` and `test_timer_speed2`: they write TEST
  with bits 4 to 7 set and count timer ticks against a loop; VenusRT ignores those bits and prints 2731 for every
  setting, Mesen prints 2731, 1639, 910, 482, 2049, 1366, 819, 456, 820, 683, 512 and 342 and passes.
- Documents read: fullsnes, "00F0h - TEST": bits 4-5 "Waitstates on RAM Access (0..3 = 0/1/4/9 cycles)", bits 6-7
  "Waitstates on I/O and ROM Access (0..3 = 0/1/4/9 cycles)", internal cycles timed as one or the other, and the
  table "SPC700 Waitstates on Internal Cycles", giving per opcode how many internal cycles take the I/O timing,
  with two more for a conditional branch taken. The ROMs' folder holds blargg's `notes.txt`, a table of the same
  counts with the ratios between them.
- Test ROM: the two ROMs, and `test_timer_speed3` (ungraded) for the same counts.
- Referee: not read for this rule. D-26's read of `rtl/SMP.vhd` lines 210-360 found that the timers' prescalers
  step by an amount TEST bits 4 to 7 set, which fullsnes does not state; the SPC700's own waitstates were not read.
  Mesen's source: none.
- Conclusion: open; fullsnes's rule is to be built and measured on the three ROMs. A setting the software "should
  never change" (fullsnes), so no game is expected to depend on it.
- Pinned by: `test_timer_speed`, `test_timer_speed2`.

### D-28. S-DSP: the Gaussian interpolation's rounding, where fullsnes and anomie give different formulas
- Opened: 2026-10-02, at stage 4 step 2, before the S-DSP's code, by reading the two sources side by side.
  fullsnes ("SNES APU DSP BRR Pitch", "4-Point Gaussian Interpolation") multiplies each of the four 15-bit samples
  by its coefficient and shifts right by 10, sums the first three with no overflow handling, adds the fourth with
  16-bit saturation, and shifts the 16-bit result right by one. anomie's S-DSP document (romhacking.net 191,
  revision 1212, "PITCH ADJUSTMENTS") shifts each product right by 11, wraps the first three's sum to 15 bits, and
  clamps the fourth's addition to 15 bits. Both use the same 512-entry table, given in both. The two differ in the
  low bit of most results, and in where the sum wraps.
- Test ROM: blargg's `spc_dsp6`, which grades the DSP by its own measurements; the audio against Mesen's through
  the probe is the second, coarser, oracle.
- Referee: not read. Mesen's source: none.
- Conclusion: open. fullsnes's formula is built first, since it states the partial overflow handling explicitly
  and anomie's comment ("the above 3 wrap at 15 bits") is a simplification of the same structure at half the
  scale; `spc_dsp6` is to decide, and the other formula is tried if it fails on an interpolation case.
- Pinned by: `spc_dsp6`.

### D-29. S-DSP: what a key-on does to the envelope and its hidden value over the start-up samples
- Opened: 2026-10-02, at stage 4 step 2, by blargg's `spc_dsp6`: every test before it passes, and "Envelope/hidden
  env 0 at kon" fails, printing 0000 six times, then 0020, then 0008 three times (checksum 179DD951). The test keys
  voice 0 twice ten samples apart with GAIN at $80 (linear decrease, rate 0), and sets GAIN to $FF (bent increase,
  every sample) for one sample at an offset moved by six SPC700 cycles each round; each value is the envelope's
  step that sample gave. VenusRT gives +32 only when the bent sample is the start-up's fifth, where the hidden value
  is still the key-on's 0, and +8 after it, where the linear decrease has left the hidden value negative.
- Documents read: anomie's S-DSP document, "BRR DECODING" (the five start-up samples, "#0 ... the envelope is set to
  0 and enters the Attack state, and is not updated for the next several samples"; "#5 = Envelope updating
  begins"), the register section on VxADSR/VxGAIN (the new value saved before clamping for the bent increase) and
  KON/KOFF (the internal KON, cleared at cycle 29 and loaded with KOFF at cycle 30 every other sample); fullsnes,
  "KON/KOFF Notes" and "Gain Notes". Neither says what the hidden value is during the start-up samples, or on which
  of the two polls a KON written at cycle 30 is taken.
- Test ROM: `spc_dsp6`, with its DSP register accesses logged around the case (the test synchronises by writing
  ENVX and reading it back until the DSP overwrites it). Two readings were tried and rejected by measurement: a
  key-on that keeps the hidden value (every step +8), and a rate-0 setting that leaves the envelope's phase and
  hidden value untouched (fails the earlier "attack->decay during gain").
- Referee: `Venus_Referee.md` §0 rates the S-DSP's formulas weak (they follow an emulator's, the key-on delay
  among them) and its rate counter real. Read 2026-10-02: SNES_MiSTer `rtl/DSP.vhd` lines 925-1000 (the voice's
  envelope stage: start-up and key-on), 1075-1090 (the KON clear and the KON/KOFF load), 1108-1120 (where its
  every-other-sample flag turns) and 1185-1270 (the envelope's update), and nothing else. What it does, in prose: a
  key-on and a key-off act only on the samples the poll belongs to, every other one; the start-up samples hold the
  envelope and the bent-increase memory at zero; the bent increase's memory is a flag set from each new value
  (12-bit, at 0x600 or more, or out of range), whether or not the counter applies the value; and the decay-to-sustain
  test compares bits 8-10 of the new value. Its ENVX is taken from the envelope before the sample's update.
- Conclusion: settled by measurement for two rules: KON and KOFF act on the poll's samples only, the poll made
  before voice 0's envelope step in cycle 30; and ENVX shows the envelope applied to the sample, before that
  sample's update (the test list's "Order/envx uses prev env" names it). With both, "hidden env 0 at kon" and the
  eight tests after it pass (measured 2026-10-02); the hidden-value reading already built agrees with the
  referee's. Sustain is now matched on bits 8-10 of the new value, as the referee and anomie's "upper 3 bits of E"
  read it.
- Pinned by: `spc_dsp6`, "hidden env 0 at kon" to "kon clears independent".

