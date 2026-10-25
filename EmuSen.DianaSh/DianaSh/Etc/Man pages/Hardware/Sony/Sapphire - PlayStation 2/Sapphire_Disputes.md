# Sapphire_Disputes — the disputes log

*Created 2026-10-23, at stage 0, with its first entry.* This is the log `Sapphire_Plan.md` §1.5 defines, in the shape
`Diamond_Disputes.md` keeps. An entry records a rule the documents and the test programs leave open or in
disagreement, the evidence taken, and the rule as implemented. It is the record of which sources a dispute step read,
so that the clean-room protocol of `Sapphire_Plan.md` §1.4 can be audited from outside. Disputes about the IOP's
processor are kept in `blackmoon-r3000`'s record (`../BlackMoon-HW/BlackMoon_R3000.md` §8), since they reach Diamond
too; disputes about the R5900 are kept in its crate's record once it exists.

## The rules this log keeps

- **A dispute step is separate.** Its only product is an entry. It ends with a rule stated in hardware terms, and a
  writer's step implements it and cites the entry. Nothing read in a dispute step is typed into code in that step.
- **The order of recourse is fixed:** the documents, then a test program written or run for the purpose, then the
  reference emulators run as black boxes, outputs only (`Sapphire_Plan.md` §1.6). No console is used, now or later,
  and no emulator's source, SDK source or FPGA RTL is read at any step.
- **The referees are weighed, not counted** (`Diamond_Referees.md` §3, adopted by `Sapphire_Plan.md` §1.5): a ruling
  needs agreement between lineages, or names the strongest single witness as such, its strength measured against the
  console outputs ps2autotests ships; a majority alone rules nothing.
- **No firmware is read.** A dispute step never opens, traces or disassembles the BIOS or any other firmware; a
  referee's log that shows BIOS output exposes its reader (`Sapphire_Plan.md` §1.4).
- **An entry is never deleted.** A conclusion later overturned is struck through and followed by the new one, with the
  evidence that overturned it.

## Entry shape

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what left it open or showed the disagreement>
- Documents read: <title, revision, section> for each
- Test program: <the program written or run, on what, and what it showed>
- Referee: <which referees ran which program, and what each showed> | <not consulted>
- Conclusion: <the rule as implemented, in prose>; argued | measured | open
- Depends on it: <what in the core or the framework changes with the answer>
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the commit that cites this entry>
```

## Entries

### D-0. The system clocks: whether the EE runs at 294.912 MHz, the bus and the GS at 147.456 MHz and the IOP at 36.864 MHz on every model, or at the 300, 150 and 37.5 MHz figures, or the 299 MHz of later models, that other sources give

- **Opened:** 2026-10-23, at stage 0, by the plan's reading of its sources (`Sapphire_Plan.md` §2.3).
- **Documents read** (online, 2026-10-23):
  - *ps2tek* at `1c91660`: "EE Architecture" gives the EE "Speed: 294.912 MHz", and the VU chapter the same figure;
    "EE Timers" gives the bus clock as "~147 MHz" (TN_MODE's clock setting); "CDVD Reads and Seeks" gives
    "IOP_CLOCK is ~36,864,000 Hz"; the BIOS's PlayStation compatibility notes say the IOP "is underclocked to 33.8688 MHz,
    the original speed of the PSX" in that mode. No GS clock is stated.
  - *Wikipedia, "PlayStation 2 technical specifications"*: the EE "clocked at 294.912 MHz (299 MHz on newer
    versions)", cited to Hennessy and Patterson and to Diefendorff and tagged as failing verification; the GS at
    147.456 MHz with no citation, and the GIF bus and video memory at 150 MHz citing Hennessy and Patterson; a fill rate
    of "16 × 147 Mpix/s = 2.352 gigapixel/s" in one place and "2400 megapixels/sec" citing the GS user's manual in
    another; the IOP at 33.8688 or 36.864 MHz, its memory at 37.5 MHz citing Sony's *Emotionally Charged Chip* article.
  - *Kunimatsu et al., "Vector Unit Architecture for Emotion Synthesis", IEEE Micro 20(2), 2000*, through its abstract
    as a search engine quotes it: "5.5 Gflops at an operation frequency of 300 MHz". The paper itself not yet read.
- **What the readings imply** [A, arithmetic on the figures above]:
  - 36.864 MHz is exactly 768 × 48,000 Hz, the SPU2's output rate, as the PlayStation's 33.8688 MHz is 768 × 44,100 Hz.
    294.912 MHz is 8 × 36.864 MHz and 147.456 MHz is 4 × 36.864 MHz. ps2tek's three figures are therefore one crystal
    family with an exact audio ratio, and 33.8688 MHz in the PlayStation mode is the other family.
  - 2,400 Mpixel/s at the GS's 16 pixels a clock is 150 MHz; at 147.456 MHz the same pipeline gives 2,359.3 Mpixel/s,
    which Wikipedia's own second figure (2.352) approximates. The manual's figure is consistent with a 150 MHz design
    target and with the EE's 300 MHz in Sony's paper: both are the published design, 1.7 % above ps2tek's.
  - 299 MHz is 8.11 × 36.864 MHz, not an integer multiple. If later models run the EE at 299 MHz, their EE and SPU2
    clocks are no longer one family, which would change the scheduler's ratios for those models; if the figure is a
    rounding or a design number, nothing changes.
  - 37.5 MHz for the IOP's memory is 150 / 4, the design family again, against ps2tek's processor at 36.864.
- **Test program:** none yet. **Evidence to take**: ps2autotests' `cpu/ee_cop0/performance` expected text, if it records
  cycle counts against a time base the console fixes (read at S2); a program of the project's that counts EE cycles
  (COP0 Count) and bus-clock timer ticks across a number of vertical blanks, run on the referees, which can only say
  what their authors chose; and any console log a public suite ships that counts cycles per frame, as ps1-tests' did for
  Diamond's D-0.
- **Referee:** not consulted.
- **Conclusion:** open. Sapphire's scheduler is written on ps2tek's family (294.912 / 147.456 / 36.864 MHz, the SPU2's
  48 kHz as 6,144 EE cycles), the ratio a constant of one module, so that a ruling for any model changes one place.
- **Depends on it:** the scheduler's ratios (`Sapphire_Plan.md` §5.1); the EE timers' rate on the bus clock; the
  frame's length in EE cycles and every speed figure of §5.12; the CRT filter's timing (§4.6), with the PCRTC's own
  video clock beside it; whether a model setting must carry a clock.
- **Pinned by:** to be written when the scheduler exists; S6 built the EE's interpreter, which counts its own cycles
  and holds no clock ratio (`Sapphire_Native.md` §16.11).
- **Implemented in:** —
- **Addendum, 2026-10-24 (stage S2)** [R, `Sapphire_References.md` §5.3]:
  - ps2autotests' `cpu/ee_cop0/performance.expected`, the console's output, tests the counters' register mapping and
    which modes count; it records no count against a time base and gives nothing for this entry.
  - K. Diefendorff, "Sony's Emotionally Charged Chip", Microprocessor Report 13(5), 1999-04-19, its Table 1 ("Source:
    SCE"): the GS at 150 MHz, the IOP at "33.8 MHz or 37.5 MHz selectable", the IOP-to-EE bus at 150 MB/s. A
    pre-release article, so the design family again (150 and 37.5 = 150 / 4), against ps2tek's 147.456 and 36.864.
    copetti.org's article (read 2026-10-24) gives the IOP at 37.5 MHz, citing its own sources.
  - The conclusion stays open; the evidence still to take is the project's own counting program on the referees (S4).
- **Addendum, 2026-10-25 (stage S11):** the scheduler exists. `clock.rs` holds the EE's 294.912 MHz, BUSCLK at two EE
  cycles and the television standards, the one place a ruling changes; the DMAC moves a qword a bus cycle and the timers
  count BUSCLK and its dividers on it (`Sapphire_Native.md` §20.3). Pinned by `a_burst_moves_a_qword_a_bus_cycle_and_cycle_stealing_releases_every_128_bytes`
  and `a_timer_is_evaluated_at_its_flag_and_not_before`. The referees' timers keep the dividers' ratios exactly
  (`s11-timers`), which says nothing of the clocks' absolute rates; the conclusion stays open.
- **Addendum, 2026-10-25 (stage S20):** the PCRTC runs on its own video clock (the *GS User's Manual*'s Figure 5-1), VCK
  at 54 MHz, held as the exact fraction 375/2048 of the EE's 294.912 MHz in `clock.rs` (D-200); a ruling on the EE's rate
  changes that fraction and nothing in `pcrtc`. Both referees make the NTSC field exactly 4,915,200 EE cycles, 1/60 s of
  294.912 MHz, which says only that each counts its field in its own EE cycles; the conclusion stays open.

### D-1. R5900, COP1: the EE's FPU has the single format alone with Sony's own operations, not the TX79's MIPS III FPU

- **Opened:** 2026-10-24, at stage S5, by the R5900's table (`EmuSen_Mips.md` §8.2), the TX79 manual and ps2tek
  disagreeing over 78 cells of COP1's tables and the OpCode table.
- **Documents read** (2026-10-24):
  - Toshiba, *TX79 Core Architecture*, rev. 2.0, 2001-04: D.4 "COP1 Instruction Encoding" gives rs = S, D, W and L,
    DMFC1 and DMTC1; the function table for S and D holds ROUND, TRUNC, CEIL and FLOOR to L and W, CVT.S, CVT.D, CVT.W,
    CVT.L and sixteen compares (C.F to C.NGT); W and L convert to S and D; A.5 holds LDC1 and SDC1 (35h, 3Dh). §10.5's
    tables list the same instructions as MIPS I to III. The manual describes Toshiba's derivative, not the EE (plan P4).
  - ps2tek at `1c91660`, "EE Instruction Decoding": COP1's rs table holds MFC1, CFC1, MTC1, CTC1, BC1, S and W only;
    "FPU.S" holds ADD.S to NEG.S, RSQRT.S (16h), ADDA.S, SUBA.S, MULA.S, MADD.S, MSUB.S, MADDA.S, MSUBA.S (18h–1Fh but
    1Bh), CVT.W (24h), MAX.S, MIN.S (28h, 29h), C.F, C.EQ, C.LT and C.LE (30h, 32h, 34h, 36h); "FPU.W" holds CVT.S alone;
    the OpCode table has `---` at 35h and 3Dh. "EE COP1 Floating Point Format": "COP1, along with the rest of the PS2,
    only supports single-precision floats".
  - The plan's own reading (`Sapphire_Plan.md` §1.1, the COP1 row): single precision only.
- **The two sides**: every TX79 cell for the D and L formats, DMFC1, DMTC1, LDC1, SDC1, the rounding conversions,
  CVT.D, CVT.L, CVT.D.W and the twelve compares other than F, EQ, OLT and OLE is reserved on ps2tek's EE (66 cells); the
  TX79 reserves the cells ps2tek gives to RSQRT, the accumulator forms, MAX and MIN (ten); and the compares at 34h and
  36h are named C.OLT and C.OLE by the TX79, C.LT and C.LE by ps2tek (two). With no NaN on the EE (ps2tek), ordered
  and unordered compares cannot be told apart by an operand, so the names may differ while the results agree [A].
- **Test program:** none yet. **Evidence to take** at S9: a program of the project's that executes each TX79-only
  encoding and records whether it is taken as a reserved instruction (or an unimplemented operation), run on the
  referees; `ee_fpu/*`'s expected texts for whatever they exercise.
- **Referee:** not consulted.
- **Conclusion:** ps2tek's tables are implemented: the TX79-only encodings decode as reserved, ps2tek's operations
  decode with the TX79's computational layout (fd at bits 10–6, fs at 15–11, ft at 20–16), the accumulator forms
  writing fs and ft [A]; argued.
- **Depends on it:** the R5900's decode and disassembly; S9's COP1; the open BIOS's instruction scan.
- **Pinned by:** `emusen-mips`'s `every_encoding_of_the_tx79_tables_decodes_disassembles_and_reassembles` (its
  differences list names D-1 for each cell) and `the_ee_s_cells_by_ps2tek`.
- **Implemented in:** stage S5's commit.
- **Addendum, 2026-10-24 (stage S6)** [R Sony, *EE Core Instruction Set Manual*, Version 5.0, October 2001, §7.1.1,
  §7.4.1, §7.4.3, §7.4.4; *EE Core User's Manual* §2.10.1]: Sony's own tables give COP1's rs as MFC1, CFC1, MTC1, CTC1,
  BC1, S and W only; the S table holds ADD to NEG, RSQRT, ADDA, SUBA, MULA, MADD, MSUB, MADDA, MSUBA, CVTW, MAX, MIN,
  C.F, C.EQ, C.LT and C.LE at the cells ps2tek gives; the W table CVTS alone; the OpCode table marks 35h and 3Dh
  "unsupported". **Conclusion: documented**, as implemented; the compares are named C.LT and C.LE.

### D-2. R5900, BC1: the EE has the likely forms BC1FL and BC1TL, which the TX79 reserves

- **Opened:** 2026-10-24, at stage S5.
- **Documents read:** the TX79 manual, D.4, BC1's rt table: BC1F (0), BC1T (1), every other cell `*`; ps2tek, "BC1":
  BC1F, BC1T, BC1FL, BC1TL (0–3). Both give BC0FL and BC0TL in COP0's BC0 table (C.2; ps2tek "BC0"), and ps2tek gives
  BC2FL and BC2TL.
- **Test program:** none yet; at S9, a program branching on each and recording whether the delay slot ran.
- **Referee:** not consulted.
- **Conclusion:** BC1FL and BC1TL decode, as ps2tek has them; argued from ps2tek alone and the TX79's own COP0 likely
  forms.
- **Depends on it:** the decode; S9.
- **Pinned by:** the gate test's differences (D-2) and `the_ee_s_cells_by_ps2tek`.
- **Implemented in:** stage S5's commit.
- **Addendum, 2026-10-24 (stage S6)** [R *EE Core Instruction Set Manual* §7.4.2, and its pages for BC1FL and BC1TL]:
  BC1's rt table gives BC1F, BC1T, BC1FL and BC1TL at 0 to 3. **Conclusion: documented**, as implemented.

### D-3. R5900: OpCode 12h is COP2, VU0's macro mode, and 36h and 3Eh are LQC2 and SQC2, which the TX79 reserves

- **Opened:** 2026-10-24, at stage S5.
- **Documents read:** the TX79 manual, A.5: 12h `*`, 36h and 3Eh η; ps2tek, "EE Instruction Decoding": COP2 at 12h with
  its own rs table (QMFC2, CFC2, QMTC2, CTC2, BC2, Special1 for every rs from 10h), LQC2 at 36h and SQC2 at 3Eh;
  Kunimatsu et al., Hot Chips 11 (1999): "Co-processor mode – MIPS COP2 instruction: 32-bit opcode", VU0 "available"
  in it, with COP2's layout drawn.
- **Conclusion:** COP2, LQC2 and SQC2 decode as ps2tek has them, COP2's operations by `vu.rs`'s table; argued, two
  sources agreeing (ps2tek and Sony's slides) against a manual of another product. The operand fields of QMFC2, QMTC2,
  CFC2, CTC2, LQC2 and SQC2 are the MIPS coprocessor moves' and loads' (rt, and rd or the base and offset) [A]; ps2tek
  names interlocking forms (".I") whose bit no source read gives.
- **Depends on it:** the decode; S12 (VU0 macro mode).
- **Pinned by:** the gate test's differences (D-3) and `the_ee_s_cells_by_ps2tek`.
- **Implemented in:** stage S5's commit.
- **Addendum, 2026-10-24 (stage S6)** [R *EE Core Instruction Set Manual* §7.1.1; *EE Core User's Manual* §2.10.2]:
  the OpCode table places COP2 at 12h and LQC2 and SQC2 ("COP2 Instructions") at 36h and 3Eh. **Conclusion:
  documented**, as implemented. The interlock bit of QMTC2.I and CTC2.I is read at S12 from the *VU User's Manual*
  §6.2 (`Sapphire_References.md` §1.1).

### D-4. VU: which half of a micro instruction runs ITOF, FTOI, ABS, NOP, MOVE, MR32, MTIR, MFIR and WAITQ

- **Opened:** 2026-10-24, at stage S5, by `vu.rs` (`EmuSen_Mips.md` §9.2).
- **Documents read:** Kunimatsu et al., Hot Chips 11: the upper unit's list (ADD/SUB, MUL, MADD/MSUB, MAX/MIN, outer
  product, clipping) and the lower unit's (DIV/SQRT/RSQRT, load/store, jump/branch, EFU, random unit; the integer ALU in
  the block diagram); ps2tek, "VU Instruction Format": the upper unit "mostly executes floating-point arithmetic", the
  lower "integer calculations, branches, and some specialized floating-point operations"; ps2tek's "FDIV Pipeline"
  puts WAITQ with DIV, SQRT and RSQRT.
- **The rule implemented** [A]: every operation the documents place lies in Special1's rows to 2Fh or Special2's rows
  0000–0101 if upper, and in Special1's row 110 or Special2's rows 0110 onwards if lower; the nine operations are
  placed by the same rows (ITOF, FTOI, ABS and NOP upper; MOVE, MR32, MTIR, MFIR, WAITQ lower).
- **Test program:** none yet; S13's microprograms placing each in the other half, run on the referees.
- **Referee:** not consulted.
- **Conclusion:** argued, by the tables' rows; open.
- **Depends on it:** the micro decode and disassembly; S13.
- **Pinned by:** `vu.rs`'s `every_encoding_of_the_documented_tables_reads_back_in_both_modes`.
- **Implemented in:** stage S5's commit.
- **Addendum, 2026-10-24 (stage S6)** [R *VU User's Manual*, Version 5.0, the contents of §4.2 "Upper Instruction
  Reference" and §4.3 "Lower Instruction Reference", and §4.1.2]: ABS, FTOI0–15, ITOF0–15 and NOP are upper
  instructions; MOVE (the example of the lower word's type 3), MR32, MTIR, MFIR and WAITQ lower. **Conclusion:
  documented**, as the rows' rule placed them. The micro words' own encodings are read at S13.

### D-5. VU: the field mask's bit order, x at bit 24 and w at bit 21

- **Opened:** 2026-10-24, at stage S5.
- **Documents read:** the Hot Chips slides draw dest as four bits (24–21) without naming them; ps2tek's MAC flag
  register orders each group x, y, z, w from the high bit (`|Ox|Oy|Oz|Ow|Ux|…|Zw|`), and its clip flags put x lowest.
- **Conclusion:** x at bit 24, by the MAC flags' order; argued, one analogy; open. It changes only how the mask is
  written, never which bits a word holds.
- **Depends on it:** the text of every masked VU operation; S12 and S13.
- **Pinned by:** `the_hot_chips_examples_and_the_flags` (`lqi.x`, a mask of bit 24 alone).
- **Implemented in:** stage S5's commit.
- **Addendum, 2026-10-24 (stage S6)** [R *VU User's Manual* §4.1.3, "dest field"]: "bits 56 through 53 for Upper
  instructions and bits 24 through 21 for Lower", bit 56 and bit 24 the x field, then y, z, w. **Conclusion:
  documented**, as implemented.

### D-6. R5900: a word divide by zero leaves LO = −1 (or +1 for a negative dividend, signed) and HI = the dividend, and a word instruction given an input that is not a word value computes on bits 31..0

- **Opened:** 2026-10-24, at stage S6, by the *EE Core Instruction Set Manual*'s Restrictions sections, which call
  both results undefined (DIV, DIVU, DIV1, DIVU1; ADD, ADDU, SUB, SUBU, the word shifts, MULT and the like).
- **Documents read:** the manual's pages above; the TX79 manual's DIV page ("No arithmetic exception occurs under any
  circumstances", the result left to software). Neither gives a value.
- **Test program:** `undefined` (`tests/s6/mod.rs`): DIV, DIVU, DIV1 and DIVU1 by zero over seven dividends, with LO
  and HI (LO1 and HI1) read back; ADDU, SUBU, SLL, SRA, SRLV and MULT with HI on four pairs that are not word values.
- **Referee:** PCSX2 2.8.2 (Americas BIOS) and Play! 0.72, measured 2026-10-24: all 168 such words equal on both, and
  equal to the reading above.
- **Conclusion:** the reading above, implemented; **measured**, two lineages agreeing. A program relying on it relies on
  undefined behaviour, which no console log yet confirms.
- **Depends on it:** `ee/ops.rs`'s `quotient_signed` and `quotient_unsigned`; the word instructions' use of bits 31..0.
- **Pinned by:** `the_undefined_inputs_give_the_interpreters_recorded_reading`.
- **Implemented in:** stage S6's commit.

### D-7. R5900: a trap's Cause.ExcCode is 13

- **Opened:** 2026-10-24, at stage S6: the *EE Core User's Manual* is not consistent with itself. Table 4-3 gives Trap
  13, as does the Cause register's table (§3.2, "01101 (13): Tr (Trap)"); §4.2.15's operation list says "Cause.ExcCode :
  12", the overflow's code.
- **Documents read:** the manual's §3.2, Table 4-3 and §4.2.15; the MIPS convention the TX79 manual follows (Tr = 13).
- **Test program:** `exc-trap`: every trap, register and immediate forms, true and false over operands whose signed and
  unsigned readings differ, through a handler recording Cause, EPC and Status.
- **Referee:** PCSX2 took every true trap and no false one, Cause.ExcCode 13, EPC the trap's, Status.EXL set: all 1,344
  words equal. Play! takes no trap through the vector (its high-level BIOS); not a witness.
- **Conclusion:** 13, implemented; the manual's two tables and PCSX2 against one line of §4.2.15; **argued**, one
  lineage (a strong witness here: it is exact on the whole program).
- **Depends on it:** `Exception::Trap`'s code; every handler that dispatches on Cause.
- **Pinned by:** `exceptions_record_cause_epc_badvaddr_and_status_as_chapter_4_says`.
- **Implemented in:** stage S6's commit.

### D-8. R5900: Status.CU*n* set to 1 makes coprocessor *n* usable

- **Opened:** 2026-10-24, at stage S6: the *EE Core User's Manual*'s Status table reads "CU … 0 Usable, 1 Unusable",
  while §4.2.14 says the exception occurs "when … the CUx bit of the Status register is 0 (unusable)" and §2.9 that
  COP0 instructions are enabled "when bit 28 (CU[0]) is set".
- **Test program:** `exc-unusable` (Status written zero, then MFC1 and QMFC2).
- **Referee:** neither PCSX2 nor Play! raises the unusable exception at all (8 of 20 words each differ, the same way);
  no witness.
- **Conclusion:** 1 = usable, §4.2.14 and §2.9 against the table's line; **argued**, documents only; open.
- **Depends on it:** the unusable checks in `ee/ops.rs`; S9 and S12 (COP1, COP2).
- **Pinned by:** `exceptions_record_cause_epc_badvaddr_and_status_as_chapter_4_says` (the `exc-unusable` class).
- **Implemented in:** stage S6's commit.

### D-9. R5900, SA: the register holds the byte count of the funnel shift, 0–15; MTSAB writes (rs ⊕ imm) mod 16, MTSAH twice (rs ⊕ imm) mod 8, MTSA rs mod 16, and MFSA reads the count

- **Opened:** 2026-10-24, at stage S6, by the `sa` program on the referees.
- **Documents read:** the *EE Core Instruction Set Manual*'s MTSAB ("SA ← (GPR[rs]3..0 XOR immediate3..0) × 8"),
  MTSAH (× 16), MTSA ("SA ← GPR[rs]63..0", undefined unless the value came from MFSA) and MFSA ("Since the value of the
  SA register is encoded in a special manner, the software cannot use the resulting value"). The manual writes a bit
  count, and declares what MFSA returns opaque.
- **Test program:** `sa`: MTSAB and MTSAH over 17 operands and six immediates, each read back by MFSA; MTSA of MFSA's
  value; `undefined`: MTSA of eight arbitrary values read back.
- **Referee:** PCSX2 and Play! agree on all 426 words: MFSA after MTSAB returns the byte count (MTSAB 1 → 1, not 8),
  after MTSAH twice the halfword count (MTSAH 1 → 2), MTSA of MFSA's value round-trips, and MTSA of an arbitrary value
  reads back modulo 16 (1Fh → Fh, 13h → 3, 10h → 0).
- **Conclusion:** the rule in the title, implemented; **measured**, two lineages agreeing, on what the manual leaves
  opaque. QFSRV's use of SA (a byte or a bit count) is S8's to grade against `ee_simd/*`.
- **Depends on it:** `ee/ops.rs`'s SA instructions; S8's QFSRV.
- **Pinned by:** `the_shift_amount_register_reads_back_as_written`, `the_undefined_inputs_give_the_interpreters_recorded_reading`.
- **Implemented in:** stage S6's commit.
- *Added 2026-10-25, at stage S8:* **QFSRV shifts by 8 × SA bits**, SA holding the byte count. `s8-misc` runs QFSRV after
  MTSAB of every count 0 to 15 over two operand pairs and after MTSAH of every count 0 to 7 with rs = rt (the page's
  rotation): all 200 record words equal on PCSX2 2.8.2 and on Play! 0.72 [M], and PCSX2 reproduces ps2autotests'
  `cpu/ee_simd/funnel` console text whole (`Sapphire_Native.md` §18.7). The open question this entry left is closed,
  **measured**; pinned by `plzcw_and_qfsrv_equal_the_manual`.

### D-10. R5900, LWR: at a byte other than 0, bits 63..32 of the destination are kept

- **Opened:** 2026-10-24, at stage S6, by the `memory` and `upper` programs on the referees.
- **Documents read:** the *EE Core Instruction Set Manual*'s LWR page: its Description ("Bytes of GPR[rt] that are not
  loaded are not changed. But if the sign bit (bit 31) is loaded, they are sign-extended"), its Operation section
  ("utemp ← GPR[rt]63..32 /* Otherwise, the high-order 4 bytes are not changed */") and its byte table; the TX79
  manual's LWR page, the same rule. Three statements in two documents agree.
- **Test program:** `memory` (LWR at every byte into a sentinel), `upper`.
- **Referee:** the two lineages disagree with each other and with the documents: PCSX2 clears bits 63..32, Play!
  sign-extends from bit 31 of the merged word (12 words of `memory` and 1 of `upper` each).
- **Conclusion:** the documents' rule, implemented; **argued**; open, no ruling by referees (they disagree).
- **Depends on it:** `ee/ops.rs`'s `lwr`; any 64-bit use of an LWR result.
- **Pinned by:** `every_load_and_store_at_every_byte_equals_the_manual`, `bits_127_to_64_survive_every_narrower_write`.
- **Implemented in:** stage S6's commit.

### D-11. R5900: DI in kernel mode clears Status.EIE

- **Opened:** 2026-10-24, at stage S6, by `exc-eidi` on the referees.
- **Documents read:** the *EE Core Instruction Set Manual*'s DI page (EIE ← 0 when EDI, EXL or ERL is set or KSU is
  00); the kit's *Restriction* (10) on DI, which concerns an interrupt arriving right after it.
- **Test program:** `exc-eidi`: Status written zero (kernel mode), EI, MFC0, DI, MFC0.
- **Referee:** Play! equal to the manual (EIE 1 after EI, 0 after DI); PCSX2 reads EIE still set after DI.
- **Conclusion:** the manual's rule, implemented; **argued**, documents and one lineage; open.
- **Depends on it:** `ee/ops.rs`'s `eidi`; S11's interrupts.
- **Pinned by:** `exceptions_record_cause_epc_badvaddr_and_status_as_chapter_4_says` (the `exc-eidi` class).
- **Implemented in:** stage S6's commit.

### D-12. R5900, COP0: the debug exception's code in Cause.EXC2 is 3

- **Opened:** 2026-10-25, at stage S7: the *EE Core User's Manual* is not consistent with itself. The Cause register's
  table (§3.2) gives EXC2 "011 (3) : Dbg (Debug)" and §4.2.4 "Cause.EXC2 = 3"; Table 4-3 gives the debug exception
  EXC2 4, a value the Cause table lists as reserved ("1xx (4-7): (Reserved)").
- **Documents read:** the manual's §3.2 (Cause), §4.1.3, §4.2.4 and Table 4-3; ps2tek at `1c91660`, "EE COP0 Exception
  Handling": "03h=Debug".
- **Test program:** `s7-debug` (`tests/s7/mod.rs`): an instruction address breakpoint and two data breakpoints through
  a handler at 80000100h recording Cause's EXC2 and BD2, ErrorEPC and BPC.
- **Referee:** PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, measured 2026-10-25: neither raises the debug exception (the instruction under each breakpoint ran and no handler record was written, `Sapphire_Native.md` §17.9); not witnesses. The rule rests on the manual alone.
- **Conclusion:** 3, implemented; two statements of the manual and ps2tek against one table cell that names a reserved
  value; **argued**.
- **Depends on it:** `Level2::Debug`'s code; every debug handler that dispatches on EXC2.
- **Pinned by:** `the_breakpoints_raise_the_debug_exception_as_chapter_9_says`, `level_2_exceptions_use_errorepc_bd2_and_the_dev_vectors`.
- **Implemented in:** stage S7's commit.

### D-13. R5900, COP0: the performance counter exception is taken when ERL is clear, CTE set and an OVFL bit set

- **Opened:** 2026-10-25, at stage S7: §7.2.4 writes the condition "STATUS.ERL && PCCR.CTE && (PCR0.OVFL ||
  PCR1.OVFL)", with ERL set; §7.2.1 says the counters do not count while "level 2 exception handlers are being
  executed", §4.1.3 that a level 2 handler holds the counter exception until it finishes, and §7.2.6's example sets
  ERL "to inhibit counting" before arming the counters and clears it by ERET to start them.
- **Documents read:** the *EE Core User's Manual* §4.1.3, §4.2.3, §7.2.1, §7.2.4, §7.2.6, §7.2.8.
- **Test program:** `s7-counters`' last part: PCR0 set to 7FFFFFF0h counting completed instructions in kernel mode with
  ERL clear, then forty NOPs; the handler at 80000080h records EXC2, Status.ERL and OVFL.
- **Referee:** PCSX2 2.8.2 and Play! 0.72, measured 2026-10-25: neither takes the counter exception; PCSX2's counters advance by the same amount whatever event and mode are selected, Play!'s do not advance; not witnesses. The rule rests on the manual alone.
- **Conclusion:** ERL clear, by three passages and the example against the expression's missing negation; **argued**.
  The exception is taken at the boundary before the next instruction; the manual allows it "a few instructions
  after the counter overflows" (§7.2.8), which the model does not reproduce (a named loss).
- **Depends on it:** `Ee::counter_pending`.
- **Pinned by:** `count_compare_and_the_performance_counters_equal_chapter_7`.
- **Implemented in:** stage S7's commit.

### D-14. R5900, COP0: BPC.BED set suppresses the debug exception

- **Opened:** 2026-10-25, at stage S7: the *EE Core User's Manual* names bit 15 "BED, Breakpoint Exception Disable"
  (§3.2's BPC table) and says "When the bit is 1, a debug exception does not occur even though the breakpoint
  conditions are met" (§9.2.1); the three sample programs of §9.3.2 to §9.3.4 set it with the comment "BED = 1:
  debugging exception occurs", and Figures 9-3 and 9-4 read the same way in the extracted text.
- **Documents read:** the manual's §3.2 (BPC), §9.2.1, §9.2.5, §9.3.2 to §9.3.4.
- **Test program:** `s7-debug`: an instruction breakpoint with BED clear (taken) and with BED set (not taken, the IAB
  flag set).
- **Referee:** PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, measured 2026-10-25: neither raises the debug exception (the instruction under each breakpoint ran and no handler record was written, `Sapphire_Native.md` §17.9); not witnesses. The rule rests on the manual alone.
- **Conclusion:** set suppresses, by the register's name and its definition; **argued**. The samples are the weaker
  witnesses: §9.3.2's also labels bit 20 "IUE = 1: enabled in supervisor mode", which is DSE, so they carry errors of
  their own.
- **Depends on it:** `Ee::instruction_break`, `Ee::data_break`.
- **Pinned by:** `the_breakpoints_raise_the_debug_exception_as_chapter_9_says`.
- **Implemented in:** stage S7's commit.

### D-15. R5900, COP0: Int0 drives Cause.IP2 and Status.IM2, Int1 drives IP3 and IM3

- **Opened:** 2026-10-25, at stage S7: the *EE Core User's Manual*'s Status text says "IM[7] (bit 15) and IM[3:2] (bit
  11:10) correspond to the internal timer interrupt and Int[1:0] signals, respectively" (IM3 to Int1, IM2 to Int0),
  while its Cause table reads "IP[3] 11 Set when the Int[0] interrupt is pending" and "IP[2] 10 … Int[1]".
- **Documents read:** the manual's §3.2 (Status, Cause), §4.2.5; ps2tek at `1c91660`, "EE COP0 Exception Handling":
  Status bit 10 "INT0 enable (INTC)", bit 11 "INT1 enable (DMAC)", Cause bit 10 "INTC interrupt pending", bit 11
  "DMAC interrupt pending".
- **Test program:** none can tell them apart until S11 connects the INTC and the DMAC to the lines.
- **Referee:** not consulted.
- **Conclusion:** Int0 on bit 10 and Int1 on bit 11 in both registers, the Status text and ps2tek against the Cause
  table's line; **argued**; open until S11's programs raise each line.
- **Depends on it:** `cop0::cause::IP2` and `IP3`; S11's INTC and DMAC.
- **Pinned by:** `an_interrupt_waits_for_ie_eie_and_im7_and_vectors_to_200h` (Int0 on IP2).
- **Implemented in:** stage S7's commit.
- **Addendum, 2026-10-25 (stage S11).** `s11-intc` raises each line through the INTC and the DMAC: its handler logs
  Cause. PCSX2 2.8.2 sets bit 10 (IP2) for a timer's compare through the INTC and bit 11 (IP3) for the DMAC's end of
  transfer, equal to the interpreter; Play! 0.72 takes no interrupt through the vector. **Conclusion: measured** (one
  lineage, agreeing with the Status text and ps2tek); closed. Pinned also by `the_intc_latches_masks_and_drives_int0_and_the_dmac_int1`.

### D-16. R5900, data cache: a refill takes the LRF way even when the other way is invalid

- **Opened:** 2026-10-25, at stage S7: the *EE Core User's Manual* §6.3.1 gives Table 6-4 (the refill way is the XOR of
  the two LRF bits, and the refill flips that way's bit) and makes an exception for the instruction cache alone ("when
  one of the ways is Invalid in the Instruction Cache, regardless of the state of the LRF bits, the data is refilled
  into the Invalidated way"); ps2tek's "EE COP0 Caches" puts the invalid way first for both caches.
- **Documents read:** the manual's §6.3.1, §6.3.7; ps2tek at `1c91660`, "EE COP0 Caches".
- **Test program:** `s7-dcache`: after DHIN invalidates way 0 of set 0 with its LRF bit at 1, and way 1 is invalid with
  LRF 0, the next miss refills way 1 (1 XOR 0); later refills follow the table, read back by CACHE DXLTG.
- **Referee:** PCSX2 2.8.2 and Play! 0.72, measured 2026-10-25: neither models the data cache (PCSX2 with its default settings; CACHE DXLTG leaves TagLo as written on both), and accesses through the program's own TLB entries do not reach the mapped memory on either; not witnesses. The rule rests on the manual alone.
- **Conclusion:** the manual's rule, the invalid-way exception for the instruction cache only; **argued**, Sony's
  manual over a source of unstated origin (P6).
- **Depends on it:** `DCache::victim`, `ICache::victim`.
- **Pinned by:** `the_data_cache_writes_back_replaces_and_locks_as_chapter_6_says`.
- **Implemented in:** stage S7's commit.

### D-17. R5900, COP0: writing Compare clears Cause.IP7

- **Opened:** 2026-10-25, at stage S7, by `s7-counters` on the referees.
- **Documents read:** the *EE Core User's Manual* §3.2, Compare: "As a side effect, writing to the Compare register
  clears the timer interrupt"; ps2tek at `1c91660`, "EE COP0 Timer": "Writing to Compare acknowledges and clears the
  pending interrupt".
- **Test program:** `s7-counters`: Compare set 400 cycles ahead with IE, EIE and IM7 set; the handler at 80000200h
  records Cause's IP7 and ExcCode, writes Compare, and records IP7 again.
- **Referee:** PCSX2 2.8.2, measured 2026-10-25: the interrupt taken with IP7 set and ExcCode 0, as the manual says, and
  IP7 still set after Compare is written. Play! 0.72 never took the interrupt in the program's 1,048,576 turns of its
  loop; not a witness.
- **Conclusion:** cleared, two documents against one lineage; **argued**.
- **Depends on it:** `Ee::write_cop0`; every timer handler that returns with IE set.
- **Pinned by:** `count_compare_and_the_performance_counters_equal_chapter_7`.
- **Implemented in:** stage S7's commit.

### D-18. R5900, COP0: PCCR keeps only its defined fields; the bits the manual draws as 0 read 0

- **Opened:** 2026-10-25, at stage S7, by `s7-counters` on the referees.
- **Documents read:** the *EE Core User's Manual* §3.2 (PCCR) and §7.1.1: CTE at 31, EVENT1 at 19..15, U1, S1, K1 and
  EXL1 at 14..11, EVENT0 at 9..5, U0, S0, K0 and EXL0 at 4..1, the rest drawn 0; ps2autotests' `cpu/ee_cop0/performance`
  writes 000FFBFEh, every defined bit but CTE (its console record does not print a read-back).
- **Test program:** `s7-counters`: MTPS of 7FFFFFFFh, MFPS.
- **Referee:** Play! 0.72 reads back 000FFBFEh, as the manual's fields give; PCSX2 2.8.2 reads back 7FFFFFFFh. Both
  agree that MTPS with a reg field other than 0 writes nothing and that MFPC and MTPC take an even field as PCR0 and an
  odd one as PCR1, as the console record says.
- **Conclusion:** the defined fields alone (800FFBFEh writable), the manual and one lineage against the other;
  **argued**.
- **Depends on it:** `cop0::perf::WRITABLE`.
- **Pinned by:** `count_compare_and_the_performance_counters_equal_chapter_7`.
- **Implemented in:** stage S7's commit.

### D-19. R5900, TLB: TLBR of a scratchpad entry returns S and reads both C fields as 2

- **Opened:** 2026-10-25, at stage S7, by `s7-spr` on the referees.
- **Documents read:** the *EE Core User's Manual* §5.2.5 ("The PFN and C field values are disregarded. (When read, the
  C field value is 2, which indicates uncached mode.)"), §3.2's PageMask ("When reading the TLB entry that corresponds
  to scratchpad RAM … the values of the PageMask register are undefined"); the *EE Core Instruction Set Manual*'s TLBR
  Operation (EntryLo0 ← TLB[63..33] || G, which carries S at bit 31).
- **Test program:** `s7-spr`: an S entry written with C 3 and read back by TLBR.
- **Referee:** PCSX2 2.8.2 returns EntryLo0 C01Fh, without S and with C 3 as written, and EntryLo1 1Fh; Play! 0.72
  returns another entry's words for the index (its TLBR differs throughout `s7-tlb`, where PCSX2's equals the manual on
  every other field). Neither maps the scratchpad through the program's entries.
- **Conclusion:** S returned and C read as 2 in both halves, the manual alone; **argued**; open. PageMask is returned as
  written, the manual leaving it undefined.
- **Depends on it:** `Tlb::read`.
- **Pinned by:** `the_scratchpad_is_reached_through_s_entries_alone`.
- **Implemented in:** stage S7's commit.

### D-20. R5900, PDIVBW: each remainder is sign-extended from bit 15 into its word of HI

- **Opened:** 2026-10-25, at stage S8: the *EE Core Instruction Set Manual*'s PDIVBW page is not consistent with itself.
  Its Description says the four remainders "are zero-extended and stored in the words … in the HI register"; its
  Operation section writes "HI31..0 ← (r015)16 || r015..0" and its figure "sign ext(A0 MOD B0)".
- **Documents read:** the PDIVBW page (Description, Restrictions, Operation, figure, Programming Notes).
- **Test program:** `s8-divide`: PDIVBW over five dividend quadwords by seven divisors in GPR[rt]15..0 (1, −1, 32,767,
  −32,768, 3, −3, 256) with other bits above them, among them negative dividends whose remainders are negative.
- **Referee:** PCSX2 2.8.2 and Play! 0.72, measured 2026-10-25: all 280 words of PDIVBW equal to the Operation
  section's reading on both. ps2autotests' console record `cpu/ee_simd/muldiv.expected` shows negative remainders
  sign-extended ("pdivbw C_P_S8_A, C_P_S8_B: H: fffffffffffff755 …"), and PCSX2 reproduces that text whole.
- **Conclusion:** sign extension, the Operation section and the figure against the Description; **measured**, two
  lineages and the console's record agreeing. For every divisor but zero the remainder's magnitude is below 32,768,
  so the sign extension of r15..0 is the remainder's own 32 bits [A]; the interpreter writes those, which also gives
  D-22's zero divisor.
- **Depends on it:** `ee/mmi.rs`'s `pdivbw`.
- **Pinned by:** `parallel_divides_equal_the_manual`.
- **Implemented in:** stage S8's commit.

### D-21. R5900, PHMADH and PHMSBH: the words of HI and LO the figures call undefined hold the odd product beside each sum, complemented for PHMSBH

- **Opened:** 2026-10-25, at stage S8, by `s8-undefined` on the referees.
- **Documents read:** the PHMADH and PHMSBH pages. The Operation sections write LO31..0, HI31..0, LO95..64 and HI95..64
  and nothing else of HI and LO; the figures mark HI and LO 63..32 and 127..96 "Undefined".
- **Test program:** `s8-undefined`: fourteen operand pairs for each, HI and LO preset by MTHI, MTHI1, MTLO and MTLO1,
  every word of HI and LO read back.
- **Referee:** PCSX2 2.8.2 (measured 2026-10-25): for PHMADH, LO63..32 = A1×B1, HI63..32 = A3×B3, LO127..96 = A5×B5 and
  HI127..96 = A7×B7, the product of each pair's high halfwords; for PHMSBH the bitwise complement of each. Play! 0.72
  writes zero in all eight words. **ps2autotests' console record** `cpu/ee_simd/muldiv.expected` decides: "phmadh -1, -1:
  … H: 0000000100000002", where every product is 1 and every sum 2, the undefined word holding 1; "phmsbh 0, 0: … H:
  ffffffff00000000", the complement of a zero product; "phmsbh -2147483648, -1: … L: ffff7fff00008000", the complement
  of 0x8000 beside 0x8000. PCSX2 reproduces the console text whole.
- **Conclusion:** product 2j+1 in the word beside sum j, complemented for PHMSBH; **measured**, the console's record
  with one lineage agreeing and the other against. The first reading built, the words left as they were, matched
  neither referee and is overturned.
- **Depends on it:** `ee/mmi.rs`'s `horizontal`.
- **Pinned by:** `the_undefined_multimedia_inputs_give_the_interpreters_recorded_reading`.
- **Implemented in:** stage S8's commit.

### D-22. R5900, the multimedia instructions' undefined inputs: a zero divisor as D-6 word by word, operands that are not word values computed on words 31..0 and 95..64, and PSRLH and PSRAH by sa3..0

- **Opened:** 2026-10-25, at stage S8, by the Restrictions sections: a zero divisor is undefined for PDIVW, PDIVUW and
  PDIVBW; operands that are not sign-extended words (bits 127..95 and 63..31 each equal) for PMULTW, PMULTUW, PMADDW,
  PMADDUW, PMSUBW, PDIVW and PDIVUW; and "The value of sa must be within the range from 0 to 15. Otherwise, the result
  is undefined" for PSRLH and PSRAH, whose Operation sections take s ← sa3..0 nonetheless.
- **Test program:** `s8-undefined`: each divide by zero in one doubleword or word beside a defined one, over six
  dividends; each word multiply and divide on operands with bits set in words 1 and 3; PSRLH and PSRAH by 16, 17, 23
  and 31.
- **Referee:** PCSX2 2.8.2 equal on all of it (measured 2026-10-25). Play! 0.72 equal on PDIVW and PDIVBW by zero,
  PMULTW, PMULTUW and PDIVW on such operands, and both shifts; it does not execute PDIVUW, PMADDW, PMADDUW or PMSUBW at all
  (`Sapphire_Native.md` §18.6). The console's record agrees for PDIVBW by zero: "pdivbw 53, 0: H: 0000000000000035 …
  L: ffffffffffffffff …", "pdivbw -2147483648, 0: H: ffffffff80000000 … L: 0000000100000001 …", LO ±1 and HI the whole
  dividend.
- **Conclusion:** the rules in the title; **measured**, two lineages agreeing (and the console for PDIVBW), PCSX2 alone
  for PDIVUW, PMADDW, PMADDUW and PMSUBW. A program relying on them relies on undefined behaviour.
- **Depends on it:** `ee/mmi.rs`'s divides and word multiplies, `psrlh`, `psrah`.
- **Pinned by:** `the_undefined_multimedia_inputs_give_the_interpreters_recorded_reading`.
- **Implemented in:** stage S8's commit.

### D-23. R5900, the saturating and clamping multimedia instructions: the bounds are the lane's signed or unsigned range, as the Descriptions give them

- **Opened:** 2026-10-25, at stage S8: the Operation sections of the saturating instructions cannot be read literally.
  PADDSB's compares the unsigned sum of two bytes with 0x7F ("if ((GPR[rs]7..0 + GPR[rt]7..0) > 0x7F) then GPR[rd]7..0 ←
  0x7F"), which taken as written saturates −1 + 1 (0xFF + 0x01) to 0x7F; PADDSW's second condition, "0x100000000 <= …
  < 0x80000000", is an empty range; PMFHL.SH's clamp compares a word with 0xFFFF8000 and PMFHL.SLW's a doubleword with
  0xFFFFFFFF80000000, which unsigned would clamp every positive value. The Descriptions say "Arithmetic results beyond
  the range of a signed 8-bit integer are saturated", and the figures "Clamp to signed halfword", "Clamp to Signed Word".
- **Test program:** `s8-arith` (every saturating add and subtract over every pair of each width's edges), `s8-hilo`
  (PMFHL.SH and PMFHL.SLW over HI and LO at every clamp's edge).
- **Referee:** PCSX2 2.8.2 and Play! 0.72 equal on every saturating add and subtract; PCSX2 equal on PMFHL.SH and
  PMFHL.SLW, while Play! does not execute PMFHL.SLW and clamps 24 of PMFHL.SH's words otherwise (a word of 1 and a word
  of FFFF7FFFh, below −32,768, each read back as FFFFh). PCSX2 reproduces the console's `arithmetic` and `muldiv` texts whole.
- **Conclusion:** the Descriptions' ranges; **measured**, two lineages on the adds and subtracts, PCSX2 and the console's
  record on PMFHL.
- **Depends on it:** `ee/mmi.rs`'s `clamp_signed`, `pmfhl_sh`, `pmfhl_slw`.
- **Pinned by:** `parallel_add_and_subtract_wrap_and_saturate_as_the_manual_says`, `moves_to_and_from_hi_and_lo_equal_the_manual`.
- **Implemented in:** stage S8's commit.

### D-24. R5900, SQRT.S: the operand is FPR[ft], the fs field fixed at zero

- **Opened:** 2026-10-25, at stage S9: `emusen-mips` decoded SQRT.S with the TX79's computational layout (D-1), its
  operand in fs, while Sony's page puts it in ft.
- **Documents read:** *EE Core Instruction Set Manual*, Version 5.0, SQRT.S's page: the operation code shows ft at bits
  20..16 and 00000 at 15..11, the Format is `SQRT.S fd, ft`, the Operation `FPR[fd] ← SQRT(FPR[ft])`. RSQRT.S's page
  keeps fs and ft (`FPR[fs] / SQRT(FPR[ft])`). The TX79 manual's SQRT takes fs.
- **Test program:** `s9-roots`: SQRT.S of every input class in f2 (ft) with π in f1 (fs).
- **Referee:** PCSX2 2.8.2 and Play! 0.72, measured 2026-10-25: both give each root of ft, never π's root.
- **Conclusion:** ft, as Sony's page writes; **measured**, the document and two lineages agreeing.
- **Depends on it:** `emusen-mips`'s form `FdFt` (decode, disassembly, assembly); `ee/fpu.rs`'s `sqrt_s`.
- **Pinned by:** `square_root_and_reciprocal_square_root_equal_the_manual`; `emusen-mips`'s round-trip test, through the
  form's operand bits.
- **Implemented in:** stage S9's commit.

### D-25. EE FPU, addition: the smaller operand is aligned to the larger's 24 bits and one more, what falls below is lost, and the sum is truncated

- **Opened:** 2026-10-25, at stage S9, by the last bit after rounding that ps2tek calls "not well understood" (plan §2.3).
- **Documents read:** *EE Core User's Manual* §8.7: "the FPU does not use the Guard, Round and Sticky bits during
  rounding. Also, since 'Rounding toward 0' does not require full value prior to rounding, unlike the definition of
  IEEE 754, the results may differ from the IEEE 754 Rounding to 0. This difference is usually restricted to the least
  significant bit only"; §8.8 the same. *VU User's Manual* §2.2: "A 24-bit calculation including hidden bits is
  performed, and the result is truncated." ps2tek "EE COP1 Floating Point Format": the last bit "may vary after a round,
  and the details on this are not well understood".
- **Test program:** ps2autotests' `cpu/ee_fpu/arithmetic` console text (a console's output, read alone): `add CF_MIN,
  CF_ONE: ffffffff` and `sub CF_MAX, CF_ONE: 7fffffff`, where IEEE 754's rounding towards zero, with its sticky bit,
  gives FFFFFFFEh and 7FFFFFFEh: the small operand vanishes before the sum is formed. Every one of the text's 144 add,
  adda, sub and suba lines is equal under alignment with 0, 1, 2 or 3 bits kept below the larger operand's 24
  [M, `examples/s9.rs texts` and a model search]: the text does not decide how many. The project's `s9-add`, 2,048 cases.
- **Referee:** measured 2026-10-25 on the 800 cases of `s9-add` whose operands and result are normal and below the
  exponent 254: PCSX2 2.8.2 equal to one bit kept on all 800 (none, 754; two, 770; IEEE 754 towards zero, 524); Play!
  0.72 equal to IEEE 754 towards zero on all 800, which the console's two lines above exclude. PCSX2's results are not
  its host's rounding towards zero, so its one bit is a choice of its own, and its weight elsewhere on the FPU is weak
  (`Sapphire_Native.md` §19.6); here it is the only witness that separates the readings.
- **Conclusion:** one bit kept, PCSX2 named as the single witness; **measured** for the absence of a sticky bit (the
  console), **argued** for the one bit. The documents' literal reading, no bit at all, differs from it only where an
  operand loses bits to the alignment and the sum then shifts left one place (1.0 − 0.1 gives 3F666668h with none,
  3F666667h with one); a console text with such a case would settle it.
- **Depends on it:** `float::add` (`GUARD`), every addition the VUs will make (S12, S13).
- **Pinned by:** `add_and_subtract_over_every_pair_of_input_classes_equal_the_manual`,
  `the_format_reproduces_the_console_lines_that_decide_its_readings`.
- **Implemented in:** stage S9's commit.

### D-26. EE FPU, multiplication: the product is summed from a radix-4 Booth recoding of FPR[ft] with its lowest column left out, so it is not commutative in its last bit

- **Opened:** 2026-10-25, at stage S9, by the console text below.
- **Documents read:** *EE Core User's Manual* §1.3.3, the COP1 pipeline's X stage: "For significand, the Booth
  function/Wallace multiplication is performed"; §8.7 and §8.8 as D-25.
- **Test program:** ps2autotests' `cpu/ee_fpu/muldiv` console text: `mul CF_MAX_MANTISSA, CF_ONE: 3fffffff` but `mul
  CF_ONE, CF_MAX_MANTISSA: 3ffffffe`, and `mul CF_ONE, CF_MAX: 7ffffffe` against `mul CF_MAX, CF_ONE: 7fffffff`; the
  same in `mula` and `madd`. The exact product truncated gives 3FFFFFFFh for both orders. A multiplier that recodes ft
  as radix-4 Booth digits (each partial product a multiple of fs, a negative one as its ones' complement and a carry-in
  of 1) and leaves out every column below column *k* reproduces all 72 mul and mula lines and all 236 multiply-add
  lines for every *k* from 1 to 22, and fails for 0 (the exact product) and from 23 on [M, model search]. With fs = 1.0
  the recoding of ft's low bits leaves a ones' complement row whose low bits and carry-in are lost, one unit below the
  product, which borrows through the zeros of fs × 2²³.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 give the exact product truncated, commutative, on both orders (`s9-muldiv`,
  70 in-range cases each against the interpreter); the console's lines above exclude that.
- **Conclusion:** Booth recoding of ft, *k* = 1, the reading nearest the documented truncation (the product is exact
  or one or two units of its lowest column below it); **measured** for the recoding of ft and its loss, **open** for
  *k*, which the texts bound to 1–22 and nothing available narrows.
- **Depends on it:** `float::booth` (`DROPPED_COLUMNS`), MUL.S, MULA.S and the multiply-adds; the VUs' FMAC if it is the
  same multiplier (S12).
- **Pinned by:** `multiply_and_divide_over_every_pair_of_input_classes_equal_the_manual`,
  `the_format_reproduces_the_console_lines_that_decide_its_readings`, `the_booth_sum_is_the_exact_product_or_just_below_it`.
- **Implemented in:** stage S9's commit.

### D-27. EE FPU, DIV.S and SQRT.S round to the nearest, not towards zero

- **Opened:** 2026-10-25, at stage S9, by the console texts below.
- **Documents read:** *EE Core User's Manual* §8.7 ("The FPU only supports 'Rounding towards 0'"), Table 8-6; the
  DIV.S and SQRT.S pages, which name no rounding.
- **Test program:** ps2autotests' console texts: `div CF_ONE, CF_MAX_MANTISSA: 3f000001`, where the quotient
  1/(2 − 2⁻²³) lies half a unit and a little above 3F000000h; `sqrt CF_MAX_MANTISSA: 3fb504f3`, `sqrt 7fffffff:
  5fb504f3`, `sqrt CF_GARBAGE2: 4f152108`, each where truncation gives one less. All 36 div lines and all 21 sqrt lines
  are equal under rounding to the nearest; truncation fails one div line and six sqrt lines. Neither operation can
  meet a tie (`Sapphire_Native.md` §19.3), so the nearest is one rule.
- **Referee:** PCSX2 2.8.2 equal to the nearest on every in-range quotient and root of `s9-muldiv` and `s9-roots`;
  Play! 0.72 truncates (121 quotients and 3 roots one unit below).
- **Conclusion:** the nearest, against §8.7's words; **measured**, the console's texts with one lineage agreeing.
- **Depends on it:** `float::div`, `float::sqrt`, `float::nearest`; RSQRT.S through D-28.
- **Pinned by:** `multiply_and_divide_over_every_pair_of_input_classes_equal_the_manual`,
  `square_root_and_reciprocal_square_root_equal_the_manual`, `the_format_reproduces_the_console_lines_that_decide_its_readings`.
- **Implemented in:** stage S9's commit.

### D-28. EE FPU, RSQRT.S: FPR[fs] over SQRT.S's root of FPR[ft], the quotient truncated, signed as FPR[fs]; D for a zero ft, I for a negative one

- **Opened:** 2026-10-25, at stage S9.
- **Documents read:** RSQRT.S's page: `FPR[fs] / SQRT(FPR[ft])`, "When the value of FPR[ft] is 0, Flag D and Flag SD
  are set", "When the value of FPR[ft] is negative, Flag I and Flag SI are set and the value of FPR[fs] divided by
  SQRT(ABS(FPR[ft])) is stored". *EE Core User's Manual* Table 8-5: `(+0)/SQRT(+0)` 7FFFFFFFh with I, `(+0)/SQRT(-0)`
  FFFFFFFFh with I.
- **Test program:** ps2autotests' `cpu/ee_fpu/sqrt` console text, 39 rsqrt lines. The sign is fs's alone: `rsqrt
  00000000, 80000000: 7fffffff` and `rsqrt 80000000, 00000000: ffffffff`, against Table 8-5's FFFFFFFFh for the
  first. Of the readings tried [M, model search]: the exact quotient rounded to the nearest, 29 lines; truncated, 37; the
  root rounded to the nearest and the quotient to the nearest, 32; **the root rounded to the nearest (SQRT.S's) and the
  quotient truncated, 38**; the root kept to 24 to 33 bits by truncation or rounding with either quotient rounding, at
  most 38; a reciprocal root times fs, at most 35. No single rounding threshold on the quotient fits both `rsqrt 3f800000,
  41c80000: 3e4ccccd` (a fraction of 0.8 rounded up) and `rsqrt 7fffffff, 7fffffff: 5fb504f2` (0.70 against the rounded
  root, rounded down). The one line not reproduced is `rsqrt 3f800000, 41c80000`, 1/√25, where the console gives
  3E4CCCCDh and the reading 3E4CCCCCh. The flags are not in the text.
- **Referee:** PCSX2 2.8.2 rounds 45 in-range quotients of `s9-roots` one unit above the reading, and gives `rsqrt
  CF_ONE, CF_MAX_MANTISSA` 3F3504F4h where the console gives 3F3504F3h; for ft = −0 it raises I and D together. Play!
  0.72 differs in 1,067 words of the program.
- **Conclusion:** the reading in the title, the page's flags; **open**: 38 of 39 console lines, the 39th named as a loss.
- **Depends on it:** `float::rsqrt`, `ee/fpu.rs`'s `rsqrt_s`.
- **Pinned by:** `square_root_and_reciprocal_square_root_equal_the_manual`, `the_format_reproduces_the_console_lines_that_decide_its_readings`.
- **Implemented in:** stage S9's commit.

### D-29. EE FPU, SQRT.S of −0 gives +0

- **Opened:** 2026-10-25, at stage S9.
- **Documents read:** SQRT.S's page: "If the value of FPR[ft] is –0, the result is –0."
- **Test program:** ps2autotests' `cpu/ee_fpu/sqrt` console text: `sqrt 80000000/-0.00: 00000000/+0.00`, and `sqrt
  CF_NEGZERO: 00000000`.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 give +0 (`s9-roots`).
- **Conclusion:** +0 for either zero, the would-be denormals among them, against the page; **measured**, the console
  and two lineages. I is raised for a negative value, never for −0 or a negative would-be denormal (the page's
  "FPR[ft] < 0" read on values; PCSX2 raises it on the sign bit, Play! never).
- **Depends on it:** `float::sqrt`, `ee/fpu.rs`'s `sqrt_s`.
- **Pinned by:** `square_root_and_reciprocal_square_root_equal_the_manual`, `the_format_reproduces_the_console_lines_that_decide_its_readings`.
- **Implemented in:** stage S9's commit.

### D-30. EE FPU, the multiply-adds: a zero product adds by ADD.S's sign rule; a product with exponent field 255 is a number; MSUB's overflow takes −TEMP's sign; an accumulator with exponent field 255 is kept, with O

- **Opened:** 2026-10-25, at stage S9: MADD.S's, MSUB.S's, MADDA.S's and MSUBA.S's Operation sections cannot be read
  literally against the console's text.
- **Documents read:** the four pages' pseudocode: `if (TEMP.exp = 0x00) then FPR[fd] ← ACC`; `if (TEMP.exp >= 0xFF) then
  FPR[fd] ← maxmin(TEMP.sign)`; `if (ACC.exp >= 0xFF) … FPR[fd] ← ACC … FCR31.O ← 1`; `if (TEMP.exp = plusminuszero)
  then FCR31.SU ← 1`.
- **Test program:** ps2autotests' `cpu/ee_fpu/muldiv` console text, 236 multiply-add lines: `madd (80000000)
  80000000, 80000000: 00000000` (−0 + +0 = +0, where `FPR[fd] ← ACC` gives −0); `msub (80000000) 00000000, 80000000:
  00000000`; `madd (CF_ZERO) CF_MAX_EXP, CF_ONE: 7f800001` (a product with exponent field 255 passed through, where
  `TEMP.exp >= 0xFF` read on the field gives 7FFFFFFFh); `msub (00000000) 7fffffff, 7fffffff: ffffffff` (the overflow's
  sign is −TEMP's, where `maxmin(TEMP.sign)` gives 7FFFFFFFh); `madd (ffffffff) 7fffffff, 7fffffff: 7fffffff` (the
  product's overflow wins over the sum). All 236 lines are reproduced by: product overflow, maxmin of ±TEMP; otherwise an
  accumulator with exponent field 255 kept; otherwise ACC ± TEMP by ADD.S. The texts print no flags.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 clamp exponent 255 and multiply exactly (`s9-acc`), so they answer for none of
  these rules; both set neither O nor SU from a multiply-add.
- **Conclusion:** the rules in the title; the accumulator branch and the flags as the pseudocode writes them (O and SO
  with the kept accumulator, SU alone for a product that underflows, O and U otherwise from the sum, D-32's clearing);
  **measured** for the values, **argued** for the flags.
- **Depends on it:** `float::fused`, `ee/fpu.rs`'s `fused`.
- **Pinned by:** `the_accumulator_instructions_equal_the_manual`, `the_format_reproduces_the_console_lines_that_decide_its_readings`.
- **Implemented in:** stage S9's commit.

### D-31. EE FPU, MAX.S and MIN.S compare the bits as sign and magnitude: −0 below +0, a would-be denormal kept and ordered above zero

- **Opened:** 2026-10-25, at stage S9.
- **Documents read:** MAX.S's and MIN.S's Operation sections: `if (FPR[fs] >= FPR[ft]) then FPR[fd] ← FPR[fs]`, which,
  with the compares' "+0 = −0", returns fs for max(−0, +0). *EE Core User's Manual* Table 8-5: Max(−0, +0) = +0,
  Min(+0, −0) = −0.
- **Test program:** ps2autotests' `cpu/ee_fpu/arithmetic` console text: `max 80000000, 00000000: 00000000`, `min
  00000000, 80000000: 80000000`, `min CF_MIN_EXP, CF_ONE: 00000001`, `max CF_GARBAGE1, CF_GARBAGE2: 00001337`.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 both return fs for max(+0, −0) and flush the would-be denormals
  (`s9-misc`, 75 words each), against the console.
- **Conclusion:** the title's order, Table 8-5 and the console against the Operation sections and both lineages;
  **measured**. The compares themselves read every zero, the would-be denormals included, as equal (the console's `eq
  00000000, 00000001: true`).
- **Depends on it:** `float::max`, `float::min`, `float::raw_order`.
- **Pinned by:** `moves_conversions_picks_compares_loads_stores_and_branches_equal_the_manual`.
- **Implemented in:** stage S9's commit.

### D-32. EE FPU, FCR31's flags: an instruction writes the cause flags its page names, cleared where their condition is absent, sets their sticky flags, and keeps every other flag; DIV.S raises no O or U, CVT.W.S no I, ABS.S, NEG.S, MAX.S and MIN.S clear O and U

- **Opened:** 2026-10-25, at stage S9.
- **Documents read:** chapter 5's opening: "Flags that are not indicated in the instructions are not changed". *EE Core
  User's Manual* Table 8-2: each cause flag "is set to 1 when … Otherwise, the bit is cleared to 0"; the sticky flags
  "are never set to 0 unless software explicitly clears them". The pages: ADD.S, SUB.S, MUL.S and the accumulator forms
  name O, U, SO, SU; DIV.S names I, D, SI, SD, though its Description says an overflow gives ±maximum; SQRT.S names I,
  SI and `Flag D ← 0`; RSQRT.S I, D, SI, SD; ABS.S, NEG.S, MAX.S and MIN.S `Flag O ← 0`, `Flag U ← 0`; MOV.S, the
  conversions and C.cond.S none but C. CVT.W.S's page clamps with no flag; Table 8-6 says "I bit (SI bit) is set" for a
  conversion's overflow, Table 8-4 "None".
- **Test program:** `s9-add`, `s9-muldiv`, `s9-roots` (FCR31 after every case from a cleared FCR31); `s9-misc` (ABS.S,
  NEG.S, MAX.S, MIN.S, MOV.S and the conversions from an FCR31 with O, U, SO and SU set, the compares with C set
  opposite to their answer and O and SI set). ps2autotests' `cpu/ee_fpu/fcr` console text, from a cleared FCR31:
  sqrt(−1) 01020041h (I, SI), 0/0 01020041h, 1/0 01010021h (D, SD), an overflow 01008011h (O, SO); an underflow
  01000001h, none set, by an operation the text does not name.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 both keep O and U through ABS.S, NEG.S, MAX.S and MIN.S (`s9-misc`, 352 words
  each), raise neither O nor U from any arithmetic, and Play! no D or I from DIV.S: their agreement is the absence of
  flag modelling, not evidence about it. The compares' C and the other flags kept: equal on both.
- **Conclusion:** the title; the pages and Table 8-2, CVT.W.S's page over Table 8-6; **argued**. PCSX2 raises I for
  SQRT.S and RSQRT.S of −0 (D-29).
- **Depends on it:** `ee/fpu.rs`'s `exponent_flags`, `divide_flags`, `unary`, `pick`, `compare`.
- **Pinned by:** the S9 programs' FCR31 words.
- **Implemented in:** stage S9's commit.

### D-33. EE FPU, the control registers: FCR0 reads 00002E30h; FCR31 holds bits 24 and 0 at 1 and lets CTC1 write 0083C078h; CFC1 and CTC1 select FCR31 by bit 4 of the register number and FCR0 otherwise

- **Opened:** 2026-10-25, at stage S9.
- **Documents read:** *EE Core User's Manual* §8.3: FCR0's implementation number 2Eh and its revision "Revision
  Number"; Figure 8-5, FCR31 with bit 24 drawn as 1 and bits 2..0 as 001; §8.7 ("the two least significant bits … are
  hardwired to '01'"); CTC1's page: "This operation is only defined when fs is 0 or 31".
- **Test program:** ps2autotests' `cpu/ee_fpu/fcr` and `transfer` console texts: FCR0 00002E30h; FCR1 to FCR15 read
  00002E30h and FCR16 to FCR30 read FCR31's value; CTC1 of all ones to FCR31 reads back 0183C079h; writes to the
  enables, the rounding mode, FS and the other condition bits leave them as they were. `s9-misc`: CFC1 of all 32
  numbers, CTC1 of all ones, of zero to FCR0 and of a mixed pattern.
- **Referee:** PCSX2 2.8.2 (recompiler) reproduces the `fcr` lines on the registers and `transfer` whole, and equals
  `s9-misc`'s moves; its interpreter reads FCR0 as 00002E00h and FCR1 to FCR31 as 0, and keeps every bit CTC1 writes.
  Play! 0.72 equals `s9-misc`'s moves.
- **Conclusion:** the title, the console's record over the manual's unstated revision; CTC1 to a number other than 0
  or 31 is undefined and taken by the same bit 4 [A]; **measured**.
- **Depends on it:** `ee/fpu.rs`'s `FCR0`, `FCR31_FIXED`, `FCR31_WRITABLE`, `cfc1`, `ctc1`.
- **Pinned by:** `moves_conversions_picks_compares_loads_stores_and_branches_equal_the_manual`.
- **Implemented in:** stage S9's commit.

### D-34. DMAC, stall control: the drain holds a normal-mode or refs packet while Dd_MADR + 8 qwords > D_STADR, every held request setting SIS; the source copies its MADR to D_STADR after normal-mode data and cnts packets only

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** *EE User's Manual* (EEUM) §5.3.2: "the values of Ds_MADR are copied to D_STADR in succession
  after the end of transfer. The packets targeted for stall control in the source channel are the transfer data in the
  Normal Mode and the packet data of ID=cnts in the Destination Chain Mode"; the condition "Dd_MADR.ADDR + 8 >
  D_STADR.ADDR", whose unit the text does not give and Figure 5-7 labels "8 qwords or more"; §5.4, SIS "Set to 1 when
  stall condition is met. (DMA does not stop.)"; the "Normal to Normal" example, in which STD is cleared by the
  source's end-of-transfer handler. ps2tek gives no stall rule.
- **Test program:** `s11-stall` (fromSPR the source, VIF1 the drain, both normal): held at once; released 8 qwords as
  D_STADR moves 12 on; finished; released by STD cleared. `s11-chain`: D_STADR after a destination cnt (unchanged) and
  cnts (moved).
- **Referee:** PCSX2 2.8.2 never holds the drain and copies MADR to D_STADR after every destination packet, cnt and end
  included; Play! 0.72 neither holds nor moves D_STADR.
- **Conclusion:** the title; the "8" as qwords by Figure 5-7; that a held drain sets SIS at each request, so a write
  clearing SIS while the stall lasts is undone at the next arbitration, is **argued**; the rest the manual's words.
  Neither lineage is a witness. Open for a console text or a game that stalls a drain.
- **Depends on it:** `dmac.rs`'s `stalled` and the STADR copy in `run`.
- **Pinned by:** `stall_control_holds_and_releases_the_drain`, `source_and_destination_chains_walk_their_tags_as_chapter_5_says`.
- **Implemented in:** stage S11's commit.

### D-35. DMAC, source chain: after a cnt packet TADR takes the address after its data; an end tag, and a ret with nothing pushed, leave TADR at the tag

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §5.2.3: for cnt "updates Dn_TADR following the transfer"; the tables give end and refe "Next
  tag address (None)" and ret with nothing pushed ends the chain; ps2tek "DMAC Chain Mode" lists TADR for each ID and
  none for end.
- **Test program:** `s11-callret` (a ret ending the chain), `s11-mfifo` (an end tag at ring offset F0h).
  ps2autotests' `dma/dmac/tagintr` console text ("TADR offset: 00000020" after a chain ending in end) agrees with either
  reading, depending on where its end tag lies, which only its source would say.
- **Referee:** PCSX2 2.8.2 leaves TADR at the end and the ret tags, and reproduces `tagintr`'s console text whole
  (`Sapphire_Native.md` §20.8); Play! 0.72 gives a TADR of neither reading on the ring and never ends `s11-callret`.
- **Conclusion:** the title. ~~TADR past the end or ret tag, as after cnt, read by analogy with cnt~~ (S11's first
  implementation), retired the same day on PCSX2's evidence, the manual being silent. **Measured** (one lineage,
  the lineage that reproduces every console text of the stage).
- **Depends on it:** `dmac.rs`'s `tadr_follows`.
- **Pinned by:** `call_and_ret_push_and_pop_the_address_stack`, `the_mfifo_drain_holds_while_empty_and_wraps_with_the_ring`.
- **Implemented in:** stage S11's commit.

### D-36. DMAC, TTE on toSPR: the tag's whole quadword is written to the scratchpad before its packet

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §5.2.3: "Whether to transfer the tag itself with the data or not can be specified by the
  TTE flag … The tag itself is transferred before the DMA transfer activated by the tag"; §5.9, TTE "Transfers
  DMAtag"; ps2tek: "When Dn_CHCR.TTE is on, bits 64-127 are transferred BEFORE QWC" (what a VIF takes, read here as the
  VIF's concern).
- **Test program:** `s11-chain`: an end tag with its upper words 11111111h, 22222222h, 33333333h and one data qword,
  toSPR with TTE.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 both write the tag's four words at SADR and the data after, SADR moving 20h.
- **Conclusion:** the title; on the other channels the whole quadword is handed to the peripheral marked as a tag, its
  stage deciding what the peripheral takes. **Measured** (two lineages).
- **Depends on it:** `dmac.rs`'s `read_tag`; S14's VIF.
- **Pinned by:** `source_and_destination_chains_walk_their_tags_as_chapter_5_says`.
- **Implemented in:** stage S11's commit.

### D-37. DMAC, undefined tags: call and ret on a channel without an address stack, call with two addresses pushed, and a destination tag ID other than cnt, cnts and end end the transfer with CIS, CHCR keeping the tag's bits, TADR at the tag and nothing moved

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §5.2.3 and §5.6, call and ret "Effective only on the VIF0, VIF1, and GIF channels"; §5.4,
  "Overflow with call stack … Transfer processing ends without transferring the call packet, and the address of the call
  tag is maintained in Dn_TADR", and "Undefined tag … In Source Chain Mode, the read tag address is maintained in
  Dn_TADR"; §5.9, ASR0 and ASR1 on channels 0 to 2 alone.
- **Test program:** `s11-chain` (call on toSPR; destination ID 3 on fromSPR), `s11-callret` (call at ASP 2 on VIF0).
- **Referee:** PCSX2 2.8.2 moves the call's packet on toSPR and then ends with TADR at the tag and CHCR keeping it; at
  ASP 2 it moves the call's packet too (MADR advanced); on an undefined destination tag it does not end but runs on
  through the scratchpad as tags. Play! 0.72 performs the call on toSPR (ASP 1, TADR at the called tag's successor) and
  mishandles the destination tag.
- **Conclusion:** the title, "effective only" read as undefined on the other channels and §5.4's words for the rest;
  **argued**; the lineages disagree with each other and with it. Open.
- **Depends on it:** `dmac.rs`'s `read_tag`.
- **Pinned by:** `source_and_destination_chains_walk_their_tags_as_chapter_5_says`, `call_and_ret_push_and_pop_the_address_stack`.
- **Implemented in:** stage S11's commit.

### D-38. EE timers: ZRET returns the count to 0 at the tick that reaches COMP; gate modes 1 to 3 count before their first edge; writing MODE leaves COUNT

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §4.2, ZRET "The counter is cleared to 0 when the counter value is equal to the reference
  value"; GATM "Resets and starts counting at the gate signal's rising edge" (and falling, and both); ps2tek "EE Timers":
  "Clear counter when it reaches compare value", "Reset counter when gate goes from low to high".
- **Test program:** `s11-timers`: COUNT sampled 600 times on BUSCLK/16 with COMP 8 and ZRET, its largest value
  recorded; gate mode 1 on V-blank read twice before its first edge; COUNT read just after MODE's write.
- **Referee:** PCSX2 2.8.2: the largest value 7, counting before the first edge, COUNT kept. Play! 0.72: no ZRET (the
  count reached 8Dh), counting before the first edge, COUNT kept.
- **Conclusion:** the title. ~~COMP read for one tick before 0~~ (S11's first implementation), retired the same day on
  PCSX2's evidence and the manual's literal "cleared … when … equal". **Measured**: ZRET on one lineage, the other two
  readings on both.
- **Depends on it:** `timer.rs`'s `advance` and `next_event`.
- **Pinned by:** `the_timers_count_compare_overflow_and_gate_as_chapter_4_says`, `zret_returns_to_zero_at_the_tick_that_reaches_comp`.
- **Implemented in:** stage S11's commit.

### D-39. INTC and DMAC: INT0 and INT1 are levels, so a request latched while masked is taken once unmasked, and one pending while Status.IE is clear is taken when IE is set

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §3.1: "the AND between the I_STAT register and the I_MASK register is obtained per bit; if any
  bit is 1, an INT0 interrupt is asserted"; §5.8, INT1's formula; *EE Core User's Manual* §3.2 and §4.2.5 (an interrupt
  taken while IP AND IM and IE and EIE, EXL and ERL clear).
- **Test program:** `s11-intc`: a timer's compare latched with its I_MASK bit clear, then unmasked; both lines raised
  with IE clear, then IE set.
- **Referee:** PCSX2 2.8.2 takes an interrupt only when the request rises with its mask and IE already set: neither the
  unmasked request nor the pending pair is taken. Play! 0.72 takes none through the vector.
- **Conclusion:** the title, the manual's level formulas; **argued**; PCSX2 departs. Open; the kernel tests at S10 run
  the BIOS's handlers over these lines.
- **Depends on it:** `board.rs`'s `lines`, `ee/mod.rs`'s `interrupt_pending`.
- **Pinned by:** `the_intc_latches_masks_and_drives_int0_and_the_dmac_int1`.
- **Implemented in:** stage S11's commit.

### D-40. DMAC, register widths: D_SQWC keeps every bit written and a transfer uses TQWC's and SQWC's low eight bits; QWC keeps 16 bits

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §5.8, D_SQWC drawn with SQWC at bits 7..0, TQWC at 23..16 and the rest "−"; §5.9, QWC at
  15..0.
- **Test program:** `s11-spr` (D_SQWC written with all ones and read back; QWC with 12345h); `s11-interleave` (TQWC
  written as 257 and SQWC as 256). ps2autotests' `dma/spr/interleave` console text: "transfer 257" moves blocks of 1 and
  "skip 256" skips none.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 both read D_SQWC back as written; PCSX2 keeps 16 bits of QWC, Play! 17.
- **Conclusion:** the title; **measured** (D_SQWC on two lineages and the console text, QWC on PCSX2 and the manual).
- **Depends on it:** `dmac.rs`'s `write`.
- **Pinned by:** `the_scratchpad_channels_and_the_common_registers_equal_chapter_5`, `interleave_moves_blocks_and_skips_as_the_console_text_records`.
- **Implemented in:** stage S11's commit.

### D-41. DMAC, priority control: D_PCR.PCE with a channel's CDEn clear holds the channel, and a tag's PCE field of 10 or 11 clears or sets D_PCR.PCE

- **Opened:** 2026-10-25, at stage S11.
- **Documents read:** EEUM §5.3.3; §5.6 (PCE 10 "Priority setting disabled (D_PCR.PCE = 0)", 11 "enabled"); §5.8, the
  condition "D_CTRL.DMAE && (!D_PCR.PCE || D_PCR.CDEn) && Dn_CHCR.STR".
- **Test program:** `s11-spr` (toSPR held with PCE and CDE9 clear, released with CDE9 set); `s11-chain` (a cnt tag with
  PCE 11, a refe with 10, D_PCR read after each).
- **Referee:** neither PCSX2 2.8.2 nor Play! 0.72 holds the channel or changes D_PCR from a tag.
- **Conclusion:** the title, the manual's formula; **argued**; neither lineage is a witness.
- **Depends on it:** `dmac.rs`'s `arbitrate` and `read_tag`.
- **Pinned by:** `the_scratchpad_channels_and_the_common_registers_equal_chapter_5`, `source_and_destination_chains_walk_their_tags_as_chapter_5_says`.
- **Implemented in:** stage S11's commit.

### D-42. The blanking signals until the PCRTC: a field of 262 lines of 9,370 bus cycles with V-blank on 22, H-blank at each line's start (NTSC)

- **Opened:** 2026-10-25, at stage S11, whose timers and INTC need H-blank and V-blank before S20 builds them.
- **Documents read:** ps2tek "EE Timers": "NTSC: 262 scanlines per frame (VBOFF: 240 | VBON: 22)", "9370 BUSCLK cycles
  per scanline", PAL 312 (286 | 26) and 9436. No document gives H-blank's width (the *GS User's Manual* searched).
- **Test program:** `s11-timers` (a timer on H-blank counted between two V-blank starts), `s11-intc` (the cycles from
  VB_ON to VB_OFF).
- **Referee:** Play! 0.72 counts 262 H-blanks; PCSX2 2.8.2 counts 525 between two V-blank starts and holds V-blank
  for 430,658 EE cycles, about 23 lines, against the stand-in's 22 (412,280 cycles).
- **Conclusion:** ps2tek's figures, with Play!; the H-blank's width of 1,600 bus cycles a choice; **open** until S20,
  where the PCRTC's programmed timing replaces the stand-in and PCSX2's 525 is to be explained (interlace's two fields,
  or an H-blank counted twice a line).
- **Depends on it:** `clock.rs`'s standards and `HBLANK`, `blank.rs`; the timers on H-blank and their gates; the INTC's
  V-blank sources.
- **Pinned by:** `the_timers_count_compare_overflow_and_gate_as_chapter_4_says`, `the_intc_latches_masks_and_drives_int0_and_the_dmac_int1`,
  `the_board_holds_no_h_blank_event_unless_a_timer_needs_it`.
- **Implemented in:** stage S11's commit.
- **Addendum, 2026-10-25 (stage S20):** the stand-in is replaced by the PCRTC's timing (D-200 to D-202). PCSX2's 525 is
  measured: it raises H-blank and HSINT every 4,681 BUSCLK, twice NTSC's line rate, so a 1/60 s field holds a frame's 525
  of them (D-201). Sapphire's field is now 263 lines non-interlaced, 262.5 interlaced, of 3,432 VCK; V-blank stays 22
  lines. **Closed.**

### D-43. VU float: the vector units' FMAC and divider compute as the FPU does, the adder without a sticky bit and one bit kept, the multiplier on a Booth recoding of the second operand, quotients and roots to the nearest

- **Opened:** 2026-10-25, at stage S12, by the question S9 left (`Sapphire_Native.md` §19.9 Q2, Q3): whether the VU's
  arithmetic shares the FPU's measured adder, multiplier and divider (D-25 to D-28).
- **Documents read:** *VU User's Manual* §2.2: "A 24-bit calculation including hidden bits is performed, and the result
  is truncated. The rounding-off operation in IEEE 754 is performed in the 0 direction, so the values for the least
  significant bit may vary", the same words the *EE Core User's Manual* §8.7 and §8.8 use for the FPU, whose console
  texts then showed no sticky bit in the adder and rounding to the nearest in DIV.S and SQRT.S (D-25, D-27); §2.4, the
  same differences from IEEE 754 as CORE §8.8; every multiply page's Remarks (§4.2: MUL, MULA, MADD, MSUB, OPMULA and
  their forms): "There is an operation error of 1 bit in multiplication, so the value multiplied by 1 may not be the same
  as the original value. By using VF[fs] as a multiplicand, the results of multiplication with 1 are guaranteed to be
  accurate", which is D-26's multiplier, fs × ft exact when ft is 1.0 and not when fs is, from the VU's own manual.
- **Test program:** `s12-add`, `s12-mul`, `s12-fused` and `s12-fdiv` (`Sapphire_Native.md` §21.4), every ordered pair of
  S9's 32 classes in four lanes. No console text of ps2autotests prints a VU sum, product or quotient that separates the
  readings: `vu/lower/fdivdelay` divides exact powers of two, `vu/lower/random` and `cpu/vu0_macro/*` print none.
- **Referee:** measured 2026-10-25, PCSX2 2.8.2 (Americas BIOS) and Play! 0.72: on every lane whose operands have
  exponent fields 1 to 253 and whose result is in range, 834 sums, 766 differences, 718 products, 342 quotients and 17
  roots, **both equal IEEE 754's rounding towards zero**, with a sticky bit, an exact commutative product and truncated
  quotients and roots, and neither differs from it anywhere [M, `examples/s12.rs explain` and a model of IEEE 754 towards
  zero on exact rationals]. That is what a host's floating point in that mode gives [A]; PCSX2's FPU, by contrast, rounds
  DIV.S to the nearest as the console does (D-27), so its VU path is not modelled on the console's.
- **Conclusion:** the VU shares `sapphire::float`'s adder, multiplier, divider and root, D-25 to D-28's readings, with
  the FPU; **argued**, the two manuals' identical words and the VU manual's own statement of the uneven multiplier
  against two lineages that both show their host's arithmetic. 562 sums and differences, 76 products, 89 quotients and 3
  roots of the programs separate the readings, so a console text of VU arithmetic decides it: S13's `vu/games/triace`
  (a game's microprogram, its twelve results printed on a console) is the first candidate.
- **Depends on it:** every VU FMAC and divider result; C3's classes (`Sapphire_Plan.md` §5.13) for the VU1 recompiler.
- **Pinned by:** `add_and_subtract_over_every_pair_of_input_classes_equal_the_manual`,
  `multiply_over_every_pair_of_input_classes_equals_the_manual`, `the_multiply_adds_and_the_outer_product_equal_the_manual`,
  `divide_square_root_and_reciprocal_root_equal_the_manual`.
- **Implemented in:** stage S12's commit.
- **Revisited 2026-10-25, at stage S13** (`Sapphire_Native.md` §23.8): `vu/games/triace`'s microprogram is generated at
  run time and its twelve values cannot be computed on Sapphire before S10 runs the program; PCSX2, on IEEE 754 towards
  zero, misses one of them (42546665h for the console's 42546666h). In `vu/lower/efu` the nano code on this module's
  multiplier and divider gives the console's last bit where IEEE 754 towards zero does not (EATAN of 1.0, 3F490FDAh;
  ERCPR of 3FFFFFFFh, 3F000001h), evidence from the EFU's datapath beside the argument. The conclusion stands,
  **argued**; `triace` on Sapphire at S10 decides it.

### D-44. VU flags: each lane in the field mask sets Z for a zero result, S for its sign bit (a clamped −0 or −MAX included), U and O for the exponent's underflow and overflow; a lane out of the mask clears its four; the status flag collects them; a multiply-add's sticky flags also take its product's

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** *VU User's Manual* §3.3.1 ("The flags of ALUs which do not operate are cleared to 0"; "When an
  overflow or underflow occurs, the Z and S flags are set according to the clamped results (+MAX/-MAX/+0/-0)"), whose
  S definition ("set to 0 when the calculation results are positive or 0") leaves −0 open; §3.3.6's table, an underflow's
  S column "X", read with the sentence before as S from the clamped −0's sign; §2.3 (overflow ±MAX with O, underflow ±0
  with U and Z); §3.3.2 ("Addition and subtraction results are reflected in the Z/ZS/S/SS/U/US/O/OS flags, and
  multiplication results are reflected only in the ZS/SS/US/OS flags") against §3.3.6's MADD and MSUB tables, whose
  rows with the accumulator at ±MAX or an overflowing sum write the product into ZS and SS ("b", "c") and whose first
  row writes "X" for ZS: taken as §3.3.2's general rule, the product's Z, S, U and O into the sticky flags; the MSUB
  table's "b" in rows where the result is positive read as the product's sign as it is subtracted.
- **Test program:** every FMAC case of `s12-add`, `s12-mul` and `s12-fused` records the status flag and the MAC flags;
  `s12-add`'s sixteen field masks, the empty one among them.
- **Referee:** measured 2026-10-25: on lanes with no exponent field of 0 or 255 among the operands, PCSX2 2.8.2 and Play!
  0.72 agree with the module on Z and S everywhere, and **neither ever raises U or O**, in the MAC flags or the status
  flag, for a lane that overflows or underflows (44 cases with an underflow, 9 with an overflow, 4 with a product's US);
  the FPU's referees omitted O and U the same way (D-30).
- **Conclusion:** the title; **argued** from the manual, neither lineage a witness for U, O or the sticky product.
- **Depends on it:** `vu/ops.rs`'s `mac_bits` and `set_flags`; `float::Fused::product`.
- **Pinned by:** the four arithmetic program tests.
- **Implemented in:** stage S12's commit.

### D-45. VU0's control registers: CCR[2,18] keeps 24 bits, CCR[2,19] reads 00002E30h, CCR[2,23], 25 and 30 read 0, CCR[2,24] keeps 0C0Ch, CCR[2,27] and 31 keep 16 bits, FBRST keeps its enables but a reset clears them; CFC2 writes bits 63..0 and keeps 127..64

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** *VU User's Manual* §5.1.3's register pictures and table: the status flag's twelve bits, writes to
  the low six ignored; the MAC flags read-only; the clipping flag's 24 bits; R's 23; CCR[2,19], 23, 24, 25 and 30
  "reserved"; TPC read-only; CMSAR0's 16 bits; FBRST's FB, RS, DE and TE per unit, FB and RS reading 0 and DE and TE
  "Current setting"; VPU-STAT read-only; CMSAR1 written only ("-/w"); the reset initialising the control registers
  (§1.3.1, "all 0" in the table). CFC2's page: "the sign-extended 32-bit data of CCR[2,id]".
- **Test program:** ps2autotests' `cpu/vu0_macro/transfer` console text (a console's output, read alone): `ctc2 -> vi16:
  00000fc0`, `vi17: 00000000`, `vi18: 00ffffff`, `vi19: 00002e30`, `vi20: 007fffff`, `vi21` and `vi22: ffffffff`, `vi23:
  00000000`, `vi24: 00000c0c`, `vi25: 00000000`, `vi27: 0000ffff`, `vi28: 00000000`, `vi29: 00000000`, `vi31: 0000ffff`,
  and `cfc2: 12345678 12345678 ffffffff ffffffff`. The test's source is not read, so what each line wrote is inferred:
  PCSX2 2.8.2 reads back FFFFFFFFh for every register it stores whole (vi18, 19, 23, 24, 25, 27) on the same program
  [M, `ps2referee.py tty`], so **each line writes all ones and reads the register back**. Read so, the console's lines
  give each register's kept bits; 0C0Ch is FBRST's D and T enables' positions, and FBRST itself reads 0 after all ones
  because its RS bits reset both units. The CFC2 line prints a 128-bit register, w first: bits 127..64 kept, bits 63..0
  the sign-extended −1. `s12-transfer` writes all ones and then zero to every register but TPC (read-only) and CMSAR1
  (whose write starts VU1, S13).
- **Referee:** PCSX2 2.8.2 reproduces the console text but the eight lines vi18, 19, 23, 24, 25, 27, 28 (0C0Ch, its
  enables kept through the reset) and 31 (0); `s12-transfer` the same, and PCSX2 keeps VU0's flags through FBRST's reset
  (D-52). Play! 0.72 reads CCR[2,19], 24 and 27 as 0, keeps 00C1h of the status flag's all ones, and keeps the flags
through the reset too.
- **Conclusion:** the title, CCR[2,19] read-only, CCR[2,24] a register of its own keeping 0C0Ch, CMSAR1 readable;
  **measured**, the console's text, the reading of what it wrote confirmed by PCSX2's read-back.
- **Depends on it:** `vu/mod.rs`'s `Vpu::ccr` and `set_ccr`, `CCR19`; `ee/cop2.rs`'s `cfc2`.
- **Pinned by:** `the_transfers_branches_and_exceptions_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-46. VU divider flags: DIV writes I for 0/0 and D for another zero divisor; SQRT writes I for a negative value and clears D; RSQRT writes I for a negative value and D for a zero root under a non-zero dividend, so 0/√0 raises neither; −0 is not negative

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** *VU User's Manual* §3.3.2: I "when a 0/0 calculation is executed by the DIV instruction or a root
  calculation of negative number is executed by the SQRT/RSQRT instruction", D "when 0 division (except 0/0) is performed
  by the DIV/RSQRT instruction, and set to 0 when the SQRT instruction is executed regardless of the results"; §3.3.5's
  table, whose SQRT row ("-X----" under the status flag, "0X----" under the sticky flags) reads as §3.3.2 says only with
  its two groups of columns exchanged; SQRT's page (DS, IS, D and I all "X"); §2.3 ("Note: D flag is not set for 0/0").
  The FPU's RSQRT.S page raises D for any zero ft (D-28), so the units part on 0/√0.
- **Test program:** `s12-fdiv`: VDIV and VRSQRT over every ordered pair of the classes, VSQRT of every class, the status
  flag after each.
- **Referee:** measured 2026-10-25. PCSX2 2.8.2 clears the status flag's Z on every divide (514 VDIV and 1,024 VRSQRT
  cases where the module keeps it), raises I for VRSQRT of 0/√0 and for VSQRT of −0 (the sign bit, as its FPU did, D-29).
  Play! 0.72 raises D and not I for VDIV's 0/0, never sets DS, raises D for 0/√0, and raises no I for a negative root
  (69 cases).
- **Conclusion:** the title, §3.3.2's words; **argued**, the three answers for 0/√0 (none, I, D) disagreeing.
- **Depends on it:** `vu/ops.rs`'s `div`, `sqrt`, `rsqrt`, `divide_flags`.
- **Pinned by:** `divide_square_root_and_reciprocal_root_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-47. VU MAX and MINI compare as MAX.S and MIN.S do, the bits as sign and magnitude

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** MAX's and MINI's pages: `if (VF[fs]x > VF[ft]x) then VF[fd]x = VF[fs]x else VF[fd]x = VF[ft]x`,
  no flag changed; the FPU's MAX.S reads `>=` and keeps fs on a tie, which on sign-and-magnitude order can differ only
  for identical bits.
- **Test program:** `s12-misc`, VMAX and VMINI with their broadcast and I forms over every pair of S9's twelve.
- **Referee:** PCSX2 2.8.2 equal on all; Play! 0.72 gives 80000000h in one lane where the module gives +0.
- **Conclusion:** D-31's order shared; **measured**, PCSX2 a witness here and Play! not.
- **Depends on it:** `float::max`, `float::min` through `vu/ops.rs`'s `pick`.
- **Pinned by:** `the_flagless_instructions_and_clip_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-48. The random unit: RNEXT shifts R left one place, bit 22 XOR bit 4 entering at bit 0, the M series of x²³ + x⁵ + 1

- **Opened:** 2026-10-25, at stage S12: RNEXT's page says "Random numbers are generated in M series" and §1.1.2 that the
  unit's mantissa comes from an M series of a chosen type, and no document gives the series.
- **Documents read:** RINIT, RGET, RNEXT and RXOR's pages (§4.3): R's 23 bits, RGET's value 0 01111111 then R.
- **Test program:** ps2autotests' `vu/lower/random` console text (a console's output): RINIT and RXOR print R as the
  low 23 bits of 01234567h, 89ABCDEFh, FEDCBA98h and 76543210h and their XORs; RNEXT prints four steps from each of
  eight seeds. Every step shifts R left with one new bit at bit 0, so only the feedback is open. Over the seventeen steps
  whose R is known (two all-ones seeds' steps, a zero seed, the twelve steps after the first) the feedback is the XOR of
  bits (4, 22), (8, 22), (4, 6), (6, 8) or (18, 20) among the pairs [M, a search over every set of up to four taps], and
  only (4, 22) and (8, 22) involve bit 22, as a 23-bit maximal series must. The four first steps, whose seeds the text
  names only as `CVF_GARBAGE`, are each one of the four RINIT values above if those are the seeds [A: the same lanes,
  and each first step equals its value shifted]; with them, (8, 22) fails 01234567h's step and (4, 22) holds on all
  twenty-one. x²³ + x⁵ + 1 is a primitive trinomial, so the series is maximal [A].
- **Referee:** PCSX2 2.8.2 and Play! 0.72 equal the module on every RINIT, RGET, RXOR and RNEXT word of `s12-int`.
- **Conclusion:** the title; **measured**, the console's text with two lineages agreeing, the last step of the inference
  argued.
- **Depends on it:** `vu/ops.rs`'s `next_r`.
- **Pinned by:** `the_manual_s_examples_and_the_console_lines_hold`, `the_integer_unit_memory_window_and_random_unit_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-49. VU CLIP compares as the FPU's compares do, every zero equal and exponent 255 a number, and the flag keeps 24 bits

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** CLIP's page: `CF = CF << 6`, then `VF[fs]x > +|VF[ft]w|` and `VF[fs]x < -|VF[ft]w|` for x, y and z;
  its example's before and after pictures, whose oldest judgement leaves at bit 24; §5.1.3's 24-bit register.
- **Test program:** `s12-misc`, CLIP over vectors of the classes against w of the classes, the flag read after each
  second judgement.
- **Referee:** PCSX2 2.8.2 equal on every judgement; Play! 0.72 keeps more than 24 bits and sets +x where −x is due.
- **Conclusion:** the title; **measured** against PCSX2 alone; `vu/upper/clip`'s console text (S13) is to be read against
  it once its constants are known from a run.
- **Measured 2026-10-25, at stage S13:** PCSX2 2.8.2 reproduces `vu/upper/clip`'s 32 console lines; their "Basic test"
  judgements (15h, 2Ah, 19h, 26h; zeros equal) and "Multi test" (each judgement seen four cycles after its CLIP) are
  reproduced on the interpreter by `the_clip_console_text_s_multi_test_holds` (`Sapphire_Native.md` §23.8).
- **Depends on it:** `vu/ops.rs`'s `clip`.
- **Pinned by:** `the_flagless_instructions_and_clip_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-50. VU0's data addresses: a qword index with bit 10 set (4000h on) reaches VU1's registers, each at its own width; below it VU0's 4 KiB repeat

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** *VU User's Manual* §1.1.5 ("VU1 registers are mapped to addresses 0x4000 to 0x43ff in VU0");
  §5.5.3 to §5.5.5, each register's qword and its width ("all 0" above VI's 16 bits, the flags' and R's widths, MAC and
  TPC read-only); the *EE User's Manual*'s map, VU0's memories each imaged over 16 KiB. Nothing says what VU0's own
  instructions reach from 1000h to 3FFFh.
- **Test program:** `s12-int`: VLQI at qword 105h, VSQI and VLQI of VU1's VF05 at 405h and of VI03 at 423h.
- **Referee:** both lineages read 105h as qword 5. PCSX2 2.8.2 keeps all 128 bits written to VI03's qword; Play! 0.72
  reads the window as zeros.
- **Conclusion:** the title; **argued** for the mirror (both lineages agree) and from §5.5 for the widths, which neither
  models.
- **Depends on it:** `vu/ops.rs`'s `load_q`, `store_q`; `Vu::window_read`, `window_write`, `mem_q`.
- **Pinned by:** `the_integer_unit_memory_window_and_random_unit_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-51. Q in macro mode: a divide's quotient reaches a macro instruction issued 7 cycles after VDIV or VSQRT and 13 after VRSQRT, and a transfer one cycle later; before that the instruction reads the old Q

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** *VU User's Manual* §5.4.1 (VDIV: `I Q R A T D1 … D6 F`; a macro instruction reads at T, a transfer
  at the A slot); §5.4.7 ("The VWAITQ instruction stalls until the VDIV/VSQRT/VRSQRT instruction executed most recently
  completes"; "if the Q register is referred to following an instruction such as VDIV, calculation results are not
  guaranteed"); §3.4.5's Figure 3-8 ("data dependency is not checked regarding the Q register, so the Q register value
  previously obtained is used here") and Figure 3-7 (a reader whose T meets F sees the quotient).
- **Test program:** `s12-fdiv`'s last five cases: Q set to 1.0, VDIV of 3/2, then 0, 1, 5, 6 or 7 VNOPs and VADDq.x.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 give 1.5 after every count: the quotient lands at once in both.
- **Conclusion:** the title, VWAITQ issuing as the quotient lands with nothing beside it; **argued** from the figures;
  neither lineage models the delay.
- **Depends on it:** `Vpu::q_pending` and `settle_q`; `ee/cop2.rs`'s `vu_op`; the timing model's `alone`.
- **Pinned by:** `divide_square_root_and_reciprocal_root_equal_the_manual`, `the_macro_timing_follows_section_5_4`.
- **Implemented in:** stage S12's commit.
- **Measured 2026-10-25, at stage S13, in micro mode** (`Sapphire_Native.md` §23.8): ps2autotests' `vu/lower/fdivdelay`
  console text reads the old Q six times after DIV and SQRT and twelve after RSQRT, then the quotient: Q reaches the T
  stage 7 and 13 cycles after the divide, the figures' F stage. PCSX2 2.8.2 reproduces all 350 lines;
  `the_fdivdelay_console_text_s_single_delayed_lines_hold` reproduces the three "single delayed" cases.

### D-52. FBRST's reset clears the unit's status, MAC and clipping flags

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** *VU User's Manual* §1.3.1 ("On reset, the control register is initialized"), §5.1.3's initial
  values (the three flags "all 0").
- **Test program:** `s12-transfer`, the flags read after FBRST is written with all ones.
- **Referee:** PCSX2 2.8.2 keeps the status flag's Z and the MAC flags through the reset; Play! 0.72 not separable (it
  stores the register whole).
- **Conclusion:** the title; **argued**.
- **Depends on it:** `Vpu::write_fbrst`.
- **Pinned by:** `the_transfers_branches_and_exceptions_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-53. A field mask of zero writes no lane, and an FMAC instruction with it clears the MAC flags

- **Opened:** 2026-10-25, at stage S12.
- **Documents read:** every page's Operation (`if (x ⊆ dest) then …`); §3.3.1, the flags of ALUs that do not operate
  cleared.
- **Test program:** `s12-add` (VADD with each of the sixteen masks), `s12-misc` (VMOVE and VMR32 with the empty mask).
- **Referee:** PCSX2 2.8.2 writes all four lanes for VMR32 with the empty mask, and agrees elsewhere; Play! 0.72 agrees.
- **Conclusion:** the title; **argued**, one lineage agreeing.
- **Depends on it:** `vu/ops.rs`'s lane loops.
- **Pinned by:** `add_and_subtract_over_every_pair_of_input_classes_equal_the_manual`, `the_flagless_instructions_and_clip_equal_the_manual`.
- **Implemented in:** stage S12's commit.

### D-54. QMTC2.I and CTC2.I proceed once the microprogram reaches a pair with the M bit set

- **Opened:** 2026-10-25, at stage S12, for S13.
- **Documents read:** *VU User's Manual* §4.1.3 ("The QMTC2 / CTC2 instruction is executed without interlocking when the
  M bit is set to 1"); §5.4.6 ("stalls occur and continue until a microinstruction, which sets the M bit to 1, is
  executed"); CTC2's page in §6.2: "until a micro instruction with the M bit set to 0 is executed".
- **Test program:** none yet; S13's microprograms with QMTC2.I against pairs with M clear and set.
- **Referee:** not consulted.
- **Conclusion:** M set releases, two passages against one; **argued**. The interface (`Micro::release`) carries it.
- **Depends on it:** `vu/micro.rs`'s `release`; S13.
- **Pinned by:** `vcallms_starts_micro_mode_and_the_ee_waits` (on the stand-in).
- **Implemented in:** stage S12's commit.

*D-55 to D-59 are unused: stage S12, built beside S21, ended at D-54, and S21's entries begin at D-60.*

### D-60. IOP, PRId: 1Fh, inside the range ps2tek's BIOS chapter leaves (at least 0Fh, below 59h)

- **Opened:** 2026-10-25, at stage S21, whose plan (`Sapphire_Plan.md` §9.2 and the stage's row) read ps2tek as
  requiring a PRId of at least 59h.
- **Documents read:** ps2tek at `1c91660`, "BIOS Boot Process" (lines 4620-4621): "The BIOS checks which CPU is
  executing by checking COP0.PRid (register 15). If PRid >= 59h, the EE boot code is executed. Else, the IOP boot code
  is executed"; "BIOS PlayStation Compatibility" (line 5095): "RESET will jump to TBIN if COP0.PRid is less than 0Fh or
  if I/O register 1F801570.3 is set". So the IOP's PRId is below 59h (59h and above is the EE's path) and, to boot the
  PlayStation 2's IOP path rather than TBIN, at least 0Fh. No document gives the value. psx-spx "cop0r15": the
  PlayStation's CXD8606CQ reads 2.
- **Test program:** `s21-map` records PRId and whether it lies in 0Fh to 58h.
- **Referee:** PCSX2 2.8.2 reads **1Fh**, inside the range. Play! 0.72 cannot run an IOP program (§22.9 of the build
  record).
- **Conclusion:** 1Fh, the one value the strongest single witness gives, inside the documents' range. **Measured** on one
  lineage; the plan's "at least 59h" was a misreading of the first sentence, corrected in `Sapphire_Plan.md` §9.2.
- **Depends on it:** `iop::PRID`, which `Iop::new` sets on `blackmoon-r3000`'s `R3000::prid`; S24's boot path.
- **Pinned by:** `the_memory_map_scratchpad_and_bus_errors_equal_the_documents` (the range word graded, the value
  against `iop::PRID`).
- **Implemented in:** stage S21's commit.

### D-61. IOP memory map: RAM's 2 MiB mirrored through 8 MiB; the unused regions (past 8 MiB, the scratchpad in KSEG1, the gap after it, 1E000000h) a bus error on a load, and a fetch from the scratchpad an instruction bus error

- **Opened:** 2026-10-25, at stage S21.
- **Documents read:** ps2tek "Memory Map": the IOP's RAM "2 MB" at 0 "(same as on PSX)", I/O at 1F800000h, the SPU2 at
  1F900000h, the BIOS at 1FC00000h, the cache control at FFFE0000h; nothing on mirrors or errors. psx-spx at `6d7d1bc`,
  "Memory Mirrors": "2MB RAM can be mirrored to the first 8MB (strangely, enabled by default)", "The Scratchpad is
  mirrored only in KUSEG and KSEG0, but not in KSEG1"; "Memory Exceptions": "Bus Error ------> Unused Memory Regions
  (including Gaps in I/O Region)"; "Scratchpad": "NOT executable. Attempts to jump to this region will cause a bus error
  on the first instruction fetch".
- **Test program:** `s21-map`: a word read through its three mirrors and written through the last; loads at 00800000h,
  BF800000h, 9F800400h and 9E000000h, each logged by the program's handler; `s21-fetch`: a jump to 9F800000h.
- **Referee:** PCSX2 2.8.2 mirrors RAM as psx-spx does (four words equal) and raises no bus error at any of the four
  loads; on the fetch from the scratchpad it stops altogether (no done word from the EE either), so `s21-fetch` runs on
  Sapphire only.
- **Conclusion:** psx-spx's PlayStation rules, which ps2tek's "same as on PSX" points to; the mirrors **measured**, the
  bus errors **argued**, PCSX2 not a witness for them. Which I/O addresses within 1F801000h-1F80FFFFh meet no device on
  the IOP is unknown; they read zero rather than fault [C], so that S22 to S26 add their devices without moving a
  fault.
- **Depends on it:** `iop::bus::decode`.
- **Pinned by:** `the_memory_map_scratchpad_and_bus_errors_equal_the_documents`, `a_fetch_from_the_scratchpad_is_an_instruction_bus_error`.
- **Implemented in:** stage S21's commit.

### D-62. IOP I-cache and cache control: 4 KiB, direct-mapped, psx-spx's PlayStation rules (stale code until a flush, a refill from the word to the line's end, the isolated cache taking stores and loads); BCC's bits 6 and 10 read zero

- **Opened:** 2026-10-25, at stage S21, by `Sapphire_References.md` §5.1: no public document gives the IOP's cache, and
  Sony told Microprocessor Report of "some minor enhancements to the cache".
- **Documents read:** psx-spx "i-Cache" (256 lines of four words, the tag, the fill behaviour, "Store instructions …
  do not affect the i-cache"), "FFFE0130h - BCC" (the bits, TAG mode's stores and loads); ps2tek "Memory Map" (the cache
  control in KSEG2, nothing more); Microprocessor Report, 1999-04-19, page 4 and Table 1.
- **Test program:** `s21-cache`, run from KSEG1 so that only its routines are cached: a routine rewritten through KSEG1
  and called through KSEG0 before and after a flush; two routines 4 KiB apart (one line) and two 2 KiB apart; a line
  entered at word 2 and then at word 0; a store and a load with SR's IsC; a loop timed cached, uncached and with IS1
  clear. An 8 KiB or two-way cache would show at the 4 KiB pair.
- **Referee:** PCSX2 2.8.2 models no IOP I-cache: rewritten code runs at once (three words differ), so neither the size
  nor the fill can be seen; its isolated store reaches neither memory nor anything a load then reads (the load returns
  memory); BCC reads 14000200h whatever is written.
- **Conclusion:** psx-spx's rules at 4 KiB, **argued**; **open** for the enhancement, which only a lineage modelling the
  IOP's cache, or a game at S28 that depends on it, can show.
- **Depends on it:** `iop::icache`, `iop::bus`'s fetch.
- **Pinned by:** `the_icache_holds_stale_code_until_flushed_and_conflicts_at_4_kib`, `a_miss_fills_to_the_lines_end_and_iblksz_0_fills_two_words`.
- **Implemented in:** stage S21's commit.

### D-63. IOP access times: a cache hit 1 cycle, an uncached fetch or load from RAM 1 + 4, an I/O register's load 1 + 2, the scratchpad and stores nothing beyond the instruction

- **Opened:** 2026-10-25, at stage S21.
- **Documents read:** none gives the IOP's; psx-spx "DMA Channels" ("CPU DRAM access takes 1 opcode cycle plus 6
  waitstates"); the PlayStation's figures measured on its console (`Diamond_Native.md` §11.3: RAM 4 cycles' stall,
  internal registers 2), taken as the IOP's [A].
- **Test program:** `s21-cache`'s three timed loops (1,000 passes of three instructions): Sapphire 3,039, 15,047 and
  15,047 IOP cycles; graded only by the documents' shape (cached at least twice as fast; IS1 clear as uncached).
- **Referee:** PCSX2 2.8.2: 3,023, 3,024 and 3,029, no wait states anywhere.
- **Conclusion:** the PlayStation's measured figures, **argued**; no write queue (a store costs nothing extra) and the
  boot ROM's 6 cycles a placeholder [C]; S32's bench measures what they cost a game.
- **Depends on it:** `iop::bus`'s `RAM_STALL`, `IO_STALL`, `ROM_STALL`.
- **Pinned by:** `the_icache_holds_stale_code_until_flushed_and_conflicts_at_4_kib`, `the_iop_keeps_its_ratio_to_the_ee_clock`.
- **Implemented in:** stage S21's commit.

### D-64. IOP timers, the request bit and ZRET: with ZRET the count runs to the target inclusive before 0; a repeated pulse leaves bit 10 set; one-shot clears it and requests nothing more until the mode or the target is written; toggle inverts it, requesting as it falls

- **Opened:** 2026-10-25, at stage S21.
- **Documents read:** psx-spx "Timers": ZRET "the counter increments up to (including) the selected target value, and
  does then restart at 0000h"; "Normally, Pulse mode should be used (INTF is permanently set, except for a few clock
  cycles when an IRQ occurs)"; one-shot "suppresses any further IRQs until a new write to the Mode register"; toggle
  "IRQ4/5/6 are triggered only each 2nd time, ie. when INTF changes from 1 to 0". ps2tek "IOP Timers": bit 6 "if unset,
  bit 10 is set to 0 after interrupt occurs"; "This raises an IRQ in I_STAT if both mode.4 and mode.10 are enabled";
  "If mode.7 (LEVL) is not set, writes to target set mode.10 to 1"; "Writes to mode reset the count to zero and set bit
  10 to 1. Reads from mode clear the two raised interrupt flags."
- **Test program:** `s21-timers`: timer 5 sampled 400 times with ZRET at 9 on the clock over 256; a repeated pulse's
  mode after its request; a one-shot's mode and I_STAT after a further 16,000 cycles, then the target written; a toggle's
  bit after each target.
- **Referee:** PCSX2 2.8.2: the largest count 8 (the target never read, as the EE timers' D-38); bit 10 clear after a
  repeated pulse; bit 10 left set after a one-shot, which requests again; the target's re-arming and the toggle equal.
- **Conclusion:** the documents' readings, which agree with each other; **argued**, PCSX2 departing on three. psx-spx's
  holds ("it will stay at 0000h for 2 clock cycles" after a write or a reset) are not modelled [A], a named loss until a
  lineage measures the IOP's.
- **Depends on it:** `iop::timer`'s `distances`, `event`, `write`.
- **Pinned by:** `the_timers_widths_clocks_flags_requests_and_gates_equal_the_documents`,
  `zret_counts_through_the_target_and_a_repeated_pulse_requests_each_time`, `one_shot_clears_the_request_bit_and_toggle_requests_every_second_time`.
- **Implemented in:** stage S21's commit.

### D-65. IOP INTC: the controller's line is the processor's Int0, Cause's IP2 (bit 10), enabled by SR's IM2, as on the PlayStation; ps2tek's "COP0.Cause:8" is not bit 8

- **Opened:** 2026-10-25, at stage S21, by ps2tek's wording against psx-spx's and the crate's.
- **Documents read:** ps2tek "IOP Interrupts": "If I_CTRL && (I_STAT & I_MASK), then COP0.Cause:8 is set. When
  COP0.Status:8 is also set when this occurs, COP0.Cause.Excode is set to 00h and the IOP jumps to 80000080h"; psx-spx
  "Interrupts" and "cop0r13": the PlayStation's controller on Cause bit 10; `BlackMoon_R3000.md` §2 (`Bus::interrupt`,
  "as Cause's IP2 sees it").
- **Test program:** `s21-intc`: Cause read with a request unmasked and SR's IEc clear; an interrupt with SR 401h (IEc,
  IM2) and with 1 (IEc alone), each over a window of a hundred loop passes; a probe beside it with SR 101h (IEc, IM0).
- **Referee:** PCSX2 2.8.2 takes the interrupt with IM2 and not with IM0 or IEc alone, a few instructions after the
  enable (its test is not at every instruction, so a four-instruction window saw none); it shows no IP bit in Cause at
  any time, the logged Cause being 0.
- **Conclusion:** IP2 and IM2, **measured** (the enabling bit on PCSX2, the PlayStation's wiring in psx-spx); ps2tek's
  "Cause:8" read as the IP field's start, not bit 8. That Cause shows IP2 while the interrupt is disabled is the
  processor's (`BlackMoon_R3000.md` D-22), where PCSX2 departs.
- **Depends on it:** `iop::intc::Intc::line` through `Bus::interrupt`; S24's BIOS.
- **Pinned by:** `the_intc_masks_latches_and_drives_ip2`.
- **Implemented in:** stage S21's commit.

### D-66. IOP DMAC registers: MADR and TADR keep 24 bits, CHCR the bits ps2tek names (OTC's three, its step always backward), DICR its documented bits; the trigger bit clears as a transfer begins; DMACEN clear holds every transfer; DMACINTEN's bit 0 keeps the flags from the master flag and bit 1 the master flag from the INTC

- **Opened:** 2026-10-25, at stage S21.
- **Documents read:** ps2tek "IOP DMA" (Dn_MADR and Dn_TADR "24-31 Unused, zero"; Dn_CHCR's bit list; DMACEN "0=All
  transfers disabled"; DMACINTEN "When 0, all channel interrupts disabled. Master interrupt flag is 0 in all cases but bus
  error interrupts", "When 1, DMA interrupts disabled - IRQ 3 is never sent to INTC"); psx-spx "DMA Channels" ("Bit 28 is
  automatically cleared upon BEGIN of the transfer", "D6_CHCR has only three read/write-able bits: 24,28,30 … bit 1 is
  always 1", DICR's bits and its master flag "recalculated on every write").
- **Test program:** `s21-dma`: every register written with all ones and read; OTC's CHCR after a transfer; a transfer
  started with DMACEN clear, then with the channel's enable clear; the master flag and I_STAT with DMACINTEN 0 and 3.
- **Referee:** PCSX2 2.8.2 keeps every bit of CHCR, MADR, TADR and DICR written (OTC's CHCR too, without its forced bit
  1); leaves bit 28 set after a transfer; runs a transfer with DMACEN clear; raises the master flag with DMACINTEN 0 and
  IRQ 3 with its bit 1 set. OTC's list, MADR and BCR kept, the flags, the master flag's IRQ 3, the bus error flag and the
  channel enable's hold equal.
- **Conclusion:** the documents' readings; **argued**, PCSX2 a witness only for the list and the flags.
- **Depends on it:** `iop::dma`'s `write`, `ready`, `master_now`, `update`.
- **Pinned by:** `otc_and_the_dmac_registers_equal_the_documents`.
- **Implemented in:** stage S21's commit.

### D-67. IOP timers' clocks and gates: timer 0's external clock is the 13.5 MHz pixel clock; timers 1 and 3 count H-blank, 262 a field on the stand-in; timer 0's gate is H-blank, 1's and 3's V-blank, 2's, 4's and 5's a stop in modes 0 and 3; gate mode 2 resets at both edges

- **Opened:** 2026-10-25, at stage S21.
- **Documents read:** ps2tek "IOP Timers" (bit 8: "Timer 0: pixel clock (13.5 MHz regardless of screen mode)", "Timer
  1/3: HBLANK"; the prescalers; nothing on the gates of timers 3 to 5); psx-spx "Timers" (the synchronisation modes of
  timers 0 to 2; timer 2's "0 or 3 = Stop counter at current value"); `Diamond_Disputes.md` D-6 (mode 2 reads zero
  outside the blank on the PlayStation's console log); D-42 (the stand-in's field).
- **Test program:** `s21-timers`: each prescaler against the IOP's clock; timer 0 on its external clock against it
  (4,096 : 1,500); timers 1 and 3 on H-blank between two V-blank starts; timer 2 in gate modes 0, 3 and 1; timer 1's
  V-blank gate in modes 0 and 3; timer 0's H-blank gate in mode 0; timers 3 to 5's gates recorded ungraded.
- **Referee:** PCSX2 2.8.2: the four prescalers equal; timer 0's external clock not 13.5 MHz; 525 H-blanks a field
  (as on the EE timers, D-42); timer 2 counting in gate mode 3; no H-blank gate on timer 0; timer 1's V-blank gate
  equal; timers 3 to 5's gates as Sapphire's reading (timer 3 counting outside V-blank, 4 stopped, 5 counting).
- **Conclusion:** ps2tek's clocks and psx-spx's gates, timers 3 to 5's gates by analogy [A], with PCSX2's agreement on
  them the only evidence; **argued** but the prescalers and timer 1's gate, **measured**; the H-blank count waits for S20
  as D-42 does.
- **Depends on it:** `iop::timer`'s `clock`, `gate`, `counting`, `edges`; `clock::PIXEL`.
- **Pinned by:** `the_timers_widths_clocks_flags_requests_and_gates_equal_the_documents`, `the_prescalers_and_the_pixel_clock`.
- **Implemented in:** stage S21's commit.

### D-68. IOP DMAC, the readings no document gives: a word a cycle on every channel with the processor held through a unit; a cycle's gap between blocks and nodes; a chain's IRQ tag sets the channel's flag and the chain goes on; a chain to memory takes its tags from the peripheral; a list ends at an address with bit 23 set; an address past 8 MiB a bus error ending the transfer

- **Opened:** 2026-10-25, at stage S21.
- **Documents read:** ps2tek "IOP DMA" (the chain tag's format, "raises IQE interrupt in DICR2 when all words in this tag
  are transferred", "a tag interrupt can still happen even when the transfer interrupt is disabled", the IOP's chain
  mode used by SIF0 and SIF1); psx-spx "DMA Channels" (the transfer rates, "CPU Operation during DMA", the linked list's
  end, the bus error flag); Microprocessor Report's "4× PlayStation DMA" (no figure per channel).
- **Test program:** `s21-dma-modes`, on Sapphire only, since its channels reach peripherals a referee has (slice,
  linked list, both chains, a requested burst and a chopped one, into the stand-ins); the unit tests of `tests/iop.rs`.
- **Referee:** not consulted; no referee's IOP channel but OTC can be driven without a peripheral's own state.
- **Conclusion:** the title, each [A] or [C]; **argued**, S22 (SIF0, SIF1 and SIF2's tags), S23, S25 and S26 measure
  what their peripherals need. *Amended 2026-10-25 at S22*: a chain to memory takes from the peripheral a quadword of
  four words, its first two the tag, whatever CHCR's bit 8 (D-143), ~~two words, four with bit 8~~; and a chain's
  unit moves only as many words as its peripheral gives or takes, the packet going on in later units (S22's FIFOs).
  The IRQ tag going on is kept against PCSX2, which ends the transfer there (D-144).
- **Depends on it:** `iop::dma`'s `unit`, `WORD`, `GAP`, `LIMIT`.
- **Pinned by:** `each_dma_mode_reaches_its_stand_in`, `a_burst_holds_the_bus_and_chopping_opens_windows`,
  `priorities_order_the_channels_and_a_channel_without_dreq_waits`.
- **Implemented in:** stage S21's commit.
- **Built 2026-10-25, at stage S13:** `Micro::release` answers the cycle after the first pair with M set, §5.4.6's
  figure's; pinned by `the_m_bit_releases_the_interlocked_transfers` and `the_ee_waits_for_micro_mode_as_section_5_4_draws_it`.
  The referees are not compared (the programs' records cannot show a transfer's cycle).

### D-80. VU micro mode: a conditional branch, JR or JALR reads an integer register written by the integer unit in the pair just before as it was before the write; one pair between, or a flag check as the writer, gives the new value

- **Opened:** 2026-10-25, at stage S13 (numbered from D-80, the numbers below set aside for stage S21, built beside it).
- **Documents read:** *VU User's Manual* §3.4.8: "a one-instruction slot is necessary between a branch condition setting
  instruction and a condition branching instruction. Flag check instructions (FCAND, FCEQ, FCGET, FCOR, FMAND, FMEQ,
  FMOR, FSAND, FSEQ, and FSOR) are an exception, and a conditional branch instruction can be placed immediately after
  them"; every conditional branch page's Remarks, the same; §3.4.7 (integer results bypassed to the next instruction's
  T stage, which the branch's need for a slot shows does not reach the branch unit). Nothing says what a branch without
  the slot reads. JR and JALR are not named; they read the same integer registers in the same unit [A].
- **Test program:** `s13-visible`: IADDIU VI01 ← 0 with VI01 1 before, then IBNE VI01, VI00 at once, after one pair, and
  after FSAND; `tests/vu_micro.rs`'s `a_branch_reads_an_integer_written_just_before_as_it_was`.
- **Referee:** measured 2026-10-25: PCSX2 2.8.2 and Play! 0.72 both take the branch right after the IADDIU (the old 1)
  and not after a pair between or after FSAND, as implemented.
- **Conclusion:** the title; **argued** from §3.4.8, two lineages agreeing.
- **Depends on it:** `vu/pipe.rs`'s `branch_vi` and `vi_last`.
- **Pinned by:** `a_branch_reads_an_integer_written_just_before_as_it_was`, `what_each_pair_sees_equals_the_manual_s_pipeline`.
- **Implemented in:** stage S13's commit.

### D-81. VU micro mode: the I bit writes I at its pair's T stage for the next pair; its own pair's upper instruction reads I as it was

- **Opened:** 2026-10-25, at stage S13.
- **Documents read:** *VU User's Manual* §3.1.4 ("the content of the Lower OP field is written to the I register at the
  T stage of the instruction … It is used by the next instruction to be executed, such as ADDi/MULi"), Figure 3-1 (the
  LOI pair's upper is a MUL that reads no I). Whether an upper instruction reading I in the I bit's own pair sees the
  constant is not said; both read at T.
- **Test program:** `s13-visible`: I ← 1.0, then `[i] addi.x vf2, vf0, i | 4.0` and `addi.x vf3, vf0, i`.
- **Referee:** measured 2026-10-25: PCSX2 2.8.2 and Play! 0.72 both give 1.0 and then 4.0.
- **Conclusion:** the title; **argued**, two lineages agreeing.
- **Depends on it:** `vu/pipe.rs`'s `execute` (I written after the upper instruction runs).
- **Pinned by:** `the_i_bit_loads_i_for_the_next_pair`, `what_each_pair_sees_equals_the_manual_s_pipeline`.
- **Implemented in:** stage S13's commit.

### D-82. TPC holds the address a microprogram stopped at in doublewords, as CMSAR0 and VCALLMSR take it; CMSAR1 takes bytes

- **Opened:** 2026-10-25, at stage S13.
- **Documents read:** *VU User's Manual* §5.1.3: TPC "indicates the PC … at which the micro subroutine has stopped",
  its unit not given; CMSAR0's VCALLMSR page: "Specify the address divided by 8"; CMSAR1: "a 16-bit address (an address
  in byte units in Micro Mem)"; §7.3.1: "read the TPC … transfer the value to CMSAR0 … VCALLMSR", and for VU1 "transfer
  the value to CMSAR1", which with CMSAR1's bytes contradicts the first; branches and JALR's link count doublewords
  (§4.3, the `vu/lower/branch` console text's link 4 two pairs on).
- **Test program:** `s13-visible`: an E bit at pair 2 of a VU0 microprogram, TPC read by CFC2 after it.
- **Referee:** measured 2026-10-25: PCSX2 2.8.2 and Play! 0.72 both read 4.
- **Conclusion:** the title; **argued**, two lineages agreeing; §7.3.1's VU1 restart read as needing TPC times 8.
- **Depends on it:** `vu/pipe.rs` (`tpc`), `Vpu::set_ccr`'s CMSAR1.
- **Pinned by:** `figure_3_19_the_e_bit_and_its_delay_slot`, `what_each_pair_sees_equals_the_manual_s_pipeline`.
- **Implemented in:** stage S13's commit.

### D-83. VU micro mode: a divide's I and D reach the status flag with its quotient, at the F stage

- **Opened:** 2026-10-25, at stage S13.
- **Documents read:** §3.4.4 times the FMAC's flags (the S stage); §3.4.5's FDIV pipeline writes back at F; DIV's,
  SQRT's and RSQRT's pages give the flags but no stage. Macro mode (S12) writes them as the instruction runs, which no
  figure there separates.
- **Test program:** `s13-divider`: FSAND of I and D in the pair after WAITQ.
- **Referee:** not separable: both referees' divider flags differ for other reasons (D-46).
- **Conclusion:** the title; **argued**, the flags as a result of the divider, landing with its other result.
- **Depends on it:** `Landing::flags`, `Vu::settle`.
- **Pinned by:** `the_divider_in_micro_mode_equals_the_manual`.
- **Implemented in:** stage S13's commit.

### D-84. The EFU computes as §7.2's nano code on the format's arithmetic, its accumulator adding each clamped product as ADD adds; its own last bits are a loss

- **Opened:** 2026-10-25, at stage S13.
- **Documents read:** *VU User's Manual* §7.2 (each function as pseudo microinstructions "for reference purpose": MUL,
  MULA, MADDA, MADD, ADD, SUB, DIV, SQRT, RSQRT on the constants of §4.3's pages, in order); §4.3's EFU pages (the
  approximations, the ranges they are valid in, no flags); §7.4's latencies.
- **Test program:** ps2autotests' `vu/lower/efu` console text, 208 values of 13 functions over 16 named inputs, whose
  values its own lines give (`Sapphire_Native.md` §23.8); `s13-efu` over the same inputs. With D-30's multiply-add (an
  accumulator of exponent field 255 kept) the nano code equals 181 of the 208; adding the clamped product as ADD adds
  equals 193, every overflowing input among them (ESUM of 7F800000h gives 7FFFFFFFh, ESIN of 7FFFFFFFh 7FD76AA2h,
  EATANxy of zero 7FC90FD7h), and the other 15 are within two in the last place [M].
- **Referee:** measured 2026-10-25: PCSX2 2.8.2 equals 93 of the console's 208 values, Play! 0.72 88; both compute on the
  host (NaNs, infinities).
- **Conclusion:** the title; **measured** against the console's text for the accumulator, the last bits of 15 values a
  named loss.
- **Depends on it:** `vu/efu.rs`.
- **Pinned by:** `the_efu_against_the_console_s_text`, `the_efu_equals_the_nano_code`.
- **Implemented in:** stage S13's commit.

### D-85. VU1 started by CMSAR1 runs its first pair 3 cycles after the CTC2, as VCALLMS's

- **Opened:** 2026-10-25, at stage S13.
- **Documents read:** §5.4.5's figures (VCALLMS: I Q R A D, the first pair's M, then T); §5.1.3 and §5.5 (CMSAR1 starts
  VU1 when written) with no figure.
- **Test program:** none separates it; the EE sees VU1 only through BC2F, BC2T and VPU-STAT.
- **Referee:** not consulted.
- **Conclusion:** the title; **argued**, a named loss.
- **Depends on it:** `Vpu::set_ccr` (`pipe::START`).
- **Pinned by:** none.
- **Implemented in:** stage S13's commit.

*D-100 on are stage S16's, built beside S21 and S13 (whose entries begin at D-80).*

### D-100. GS privileged registers: a read of a write-only or reserved address returns CSR

- **Opened:** 2026-10-25, at stage S16 (numbered from D-100 beside S21's and S13's entries): the *GS User's Manual*
  marks every privileged register but CSR and SIGLBLID "(w)" and says nothing of a read.
- **Documents read:** *GS User's Manual* §7.2 (each register's "(w)" or "(r/w)"); *EE User's Manual* §2.2.3 (the map,
  its "(Reserved) GS-dependent" ranges); ps2tek "I/O Maps" and "GS Register List" (no read rule).
- **Test program:** `s16-regs`: SMODE1, SMODE2, SRFSH, SYNCH1, SYNCH2 and SYNCV read; PMODE, DISPFB1, DISPLAY1,
  DISPFB2, DISPLAY2, EXTBUF, EXTDATA and BGCOLOR written 0123456789ABCDEFh and read; IMR and BUSDIR written and read;
  GS addresses 0Fh, 3Fh, 42h, 43h, 45h, 49h and 7Fh read.
- **Referee:** PCSX2 2.8.2: every one reads CSR's value (551B6000h at the time: ID 55h, REV 1Bh, FIFO empty, FIELD set).
  Play! 0.72: CCCCCCCCCCCCCCCCh from every register and 0 from 49h and 7Fh, a fill rather than any register's value.
- **Conclusion:** CSR; **measured**, PCSX2 the only witness that returns a register (Play!'s fill is no reading of
  hardware).
- **Depends on it:** `gs::Gs::read_privileged`; a program polling a write-only register.
- **Pinned by:** `the_registers_csr_imr_and_the_signals_behave_as_chapters_6_and_7_say`,
  `the_privileged_registers_answer_their_images_and_above_them_a_bus_error`.
- **Implemented in:** stage S16's commit.

### D-101. CSR's REV and ID read 1Bh and 55h

- **Opened:** 2026-10-25, at stage S16.
- **Documents read:** *GS User's Manual* §7.2: REV (bits 23..16) "Revision No. of the GS" and ID (bits 31..24) "ID of the
  GS", no values; ps2tek gives none.
- **Test program:** `s16-regs` records CSR's bits 31..16, ungraded.
- **Referee:** PCSX2 2.8.2: ID 55h, REV 1Bh. Play! 0.72: ID 00h, REV 07h.
- **Conclusion:** PCSX2's; **argued**, the lineage whose CSR models the manual's other fields (the events, FIFO, the
  reset) against one that models none of them (D-100's table); open until a console text prints CSR.
- **Depends on it:** `gs::csr::REV`, `gs::csr::ID`; a program that identifies its GS.
- **Pinned by:** `the_privileged_registers_answer_their_images_and_above_them_a_bus_error`.
- **Implemented in:** stage S16's commit.

### D-102. GS privileged registers: EE addresses that differ in bits 10 and 11, and in bits 13 to 15, reach the same register for reads and writes; 12010000h to 13FFFFFFh is a bus error

- **Opened:** 2026-10-25, at stage S16.
- **Documents read:** *EE User's Manual* §2.2.3: "addresses [12] and [9:4] of the CPU bus are output to GA[6:0]"; its
  table's "Image of 0x1200_0000 - 0x1200_03f0" for 12000400h to 120007F0h and "Image of 0x1200_0000 - 0x1200_07f0" for
  12000800h to 12000FF0h, the same for the 12001000h bank; 12002000h to 1200FFF0h "(Reserved)"; 12010000h to 13FFFFF0h
  "(Reserved) Bus error".
- **Test program:** `s16-regs`: SIGLBLID written at 12001080h and read at 12001480h, 12001880h, 12001C80h, 12003080h
  and 1200F080h; written at 12001880h and read at 12001080h; CSR read at 12001400h.
- **Referee:** PCSX2 2.8.2 writes SIGLBLID through 12001880h but reads CSR at every image (D-100's rule at an address it
  does not take for SIGLBLID). Play! 0.72 neither writes nor reads through an image (reads 0).
- **Conclusion:** images for reads and writes. The images of bits 10 and 11 by the manual's table, **documented**, against
  both lineages; those of bits 13 to 15 by the sentence on GA[6:0], **argued**, the table calling the range reserved; the
  bus error as the table gives it.
- **Depends on it:** `gs::privileged::ga`, `Board::error`.
- **Pinned by:** `the_privileged_registers_answer_their_images_and_above_them_a_bus_error`,
  `the_registers_csr_imr_and_the_signals_behave_as_chapters_6_and_7_say`.
- **Implemented in:** stage S16's commit.

### D-103. The GS's interrupt is one line, any event whose IMR bit is clear, which the INTC latches at its rise: an event while it is high gives no request, and a stalled SIGNAL taken when the first's flag is cleared gives none until IMR's mask is set and cleared

- **Opened:** 2026-10-25, at stage S16.
- **Documents read:** *GS User's Manual* §6.1 ("it asserts interrupt signals to the host when signals are generated, it
  can control them individually with the Interrupt Mask Register"); §6.2.1's note: "the interrupt to the host
  corresponding to the write operation to the second SIGNAL register does not occur. The interrupt corresponding to the
  second SIGNAL occurs by clearing the SIGNAL flag corresponding to the first SIGNAL, and setting the SIGMSK flag of the
  IMR register to 1 and then to 0"; the kit's *Restriction*, "Restrictions on GS (2) GS Interrupt Handler": "while the
  interrupt handler is in the process for the interrupt from the GS, if additional interrupt from the GS occurs, the EE
  may not be able to detect it"; *EE User's Manual* §3.1 (I_STAT takes a request at its edge).
- **Test program:** `s16-regs`: SIGNAL masked then unmasked; a second SIGNAL with a LABEL and a FINISH behind it; the
  first's flag cleared; SIGMSK set and cleared; FINISH unmasked and raised while SIGNAL holds the line; each followed by
  I_STAT's bit 0, read and cleared.
- **Referee:** PCSX2 2.8.2 stalls the second SIGNAL and holds the LABEL and FINISH behind it as §6.2.1 says, but raises
  the INTC's request at every unmasked event and every mask cleared over a set flag: the stalled SIGNAL taken at the clear,
  FINISH unmasked under SIGNAL and FINISH raised under SIGNAL each give one (three entries). Play! 0.72 models neither
  the stall nor SIGNAL's mask, so is no witness.
- **Conclusion:** the documents' line; **argued**, two passages against one lineage.
- **Depends on it:** `Gs::line`, `Gs::take_rise`, `Board::sync_gs`; a game whose GS handler leaves IMR unmasked.
- **Pinned by:** `the_gs_interrupt_is_one_line_the_intc_latches_at_its_rise`,
  `the_registers_csr_imr_and_the_signals_behave_as_chapters_6_and_7_say`.
- **Implemented in:** stage S16's commit.

### D-104. A PSMT8 or PSMT4 buffer an odd number of units wide: the pages across are its width over the page's, rounded down

- **Opened:** 2026-10-25, at stage S16.
- **Documents read:** *GS User's Manual* §2.6 (a texture buffer's width "must be a multiple of 128 texels when PSMT8 or
  PSMT4 is specified"), §8.2 (pages in order along the buffer's rows), §7.1's TBW and MIPTBP notes ("It is set to 1 if
  the width is less than 64"). No rule for an odd width.
- **Test program:** `s16-places`: PSMT8 16×128 written at base 200h with width 1, then the raw words of pages 10h and
  11h read back, each filled first; under this reading rows 64 to 127 fall on page 10h again and page 11h is untouched.
- **Referee:** PCSX2 2.8.2 and Play! 0.72 both equal the reading.
- **Conclusion:** rounded down; **measured**, both lineages.
- **Depends on it:** `gs::local::address`; S18's textures one unit wide in PSMT8 or PSMT4.
- **Pinned by:** `transfers_take_their_places_wrap_restart_and_copy_as_chapter_4_says`.
- **Implemented in:** stage S16's commit.

### D-105. A depth format from a base pointer inside a page: its blocks follow the page-start order counted on from the base, as for the colour formats

- **Opened:** 2026-10-25, at stage S16.
- **Documents read:** *GS User's Manual* §8.5: "The block arrangement order conforms to the one obtained when the start
  of a page is pointed to. The block pointed to and the ones following configure a one-page rectangle", with two PSMCT32
  examples; §8.3's depth figures, each its colour figure with block bits 3 and 4 inverted (block XOR 18h).
- **Test program:** `s16-formats-z`: PSMZ32 and PSMZ16 written from block 40h (a page's start) and read raw as PSMCT32;
  then each depth format written as one page-rectangle from block 1, read raw and in its own format.
- **Referee:** PCSX2 2.8.2: from a page's start equal; from block 1 the depth formats' blocks 7, 15, 23 and 31 land
  elsewhere and blocks 24 and 32 keep their old contents, as if the inversion were applied after the base is added
  ((base + colour block) XOR 18h); its round trips equal, so the difference shows only to another format. Play! 0.72
  moves no depth format through the transfers, so is no witness.
- **Conclusion:** §8.5's order; **argued**, one lineage dissenting where the inversion and the carry from the base meet.
  A game that draws a depth buffer, or textures from one, at a base inside a page would decide it.
- **Depends on it:** `gs::psm::block_number`, `gs::local::address` for PSMZ32, PSMZ24, PSMZ16 and PSMZ16S; S17's depth
  buffers (ZBP is page-aligned, so drawing cannot reach the case), S18's textures from a depth buffer.
- **Pinned by:** `every_format_round_trips_and_lies_in_memory_as_the_manual_says`, `every_formats_page_falls_as_chapter_8s_figures_show`.
- **Implemented in:** stage S16's commit.

### D-106. A transfer's coordinates wrap at 2048 in X and in Y, both directions

- **Opened:** 2026-10-25, at stage S16.
- **Documents read:** *GS User's Manual* §4.1.3: "X = (SAX + RRX) % 2048, Y = (SAY + RRY) % 2048"; ps2tek "GS
  Transfers" the same.
- **Test program:** `s16-places`: PSMCT32 16×4 written from (2040, 2046), read back from there and at (0, 0).
- **Referee:** Play! 0.72: equal. PCSX2 2.8.2: Y wraps, X does not; host to local writes the wrapped pixels elsewhere and
  local to host reads them as 0.
- **Conclusion:** the manual's; **measured**, with Play!.
- **Depends on it:** `gs::transfer`.
- **Pinned by:** `transfers_take_their_places_wrap_restart_and_copy_as_chapter_4_says`.
- **Implemented in:** stage S16's commit.

### D-107. A local-to-local transfer whose rectangles overlap copies pixel by pixel in TRXPOS's order

- **Opened:** 2026-10-25, at stage S16, for S19, whose gate names "local-to-local overlap".
- **Documents read:** *GS User's Manual* §4.1.2 and figure 4-1 (the four orders, "enabled only when transmitting from
  local buffer to local buffer"); nothing on overlap.
- **Test program:** `s16-places`: PSMCT32 8×2 copied one pixel to the right in order 0 (left to right, top to bottom).
- **Referee:** PCSX2 2.8.2 copies as if the source were read whole first (a shift by one); Play! 0.72 performs no
  local-to-local transfer.
- **Conclusion:** pixel by pixel in the order, so order 0 smears the first pixel along the row; **argued**: an order the
  program chooses can matter only if the copy runs in it, and order 2 is the one that shifts right. **Open** for S19,
  whose programs copy in each order at several overlaps.
- **Depends on it:** `gs::transfer::copy`.
- **Pinned by:** `transfers_take_their_places_wrap_restart_and_copy_as_chapter_4_says`.
- **Implemented in:** stage S16's commit.
### D-120. CDVD seeks: contiguous below 16 sectors on a DVD and 8 on a CD at a block's time a sector, fast below 14,764 and 4,371 at 30 ms, full otherwise at 100 ms; a stopped drive spins up for 333 ms first, and Stop takes 166.67 ms

- **Opened:** 2026-10-25, at stage S23, by ps2tek's "Unknown what delta is needed for a contiguous read" and its quotation
  of an emulator's bound (excluded as evidence, `DOCS.txt` X.3).
- **Documents read:** ps2tek at `1c91660`, "CDVD Reads and Seeks" ("Contiguous read: When the seek delta is very small or
  zero, seek time = block_timing * delta"; "Fast seek: When seek delta < 14764 for DVD reads and < 4371 for CD reads,
  seek time = ~30 ms"; "Full seek: Seek time = ~100 ms"; "it takes 333 ms for the drive to spin and finish the seek";
  the formula), "CDVD N Commands" (Stop: "Seems to have a 166 ms delay?").
- **Test program:** `ncmd_*` (`Sapphire_Native.md` §25.3): N command Seek sent from the EE to the drive's registers on a
  synthetic DVD and CD, each timed by COP0 Count until the N status's busy bit clears; deltas 0, 8, 15, 16, 960, 13,000,
  14,763, 14,764, 14,765 and the disc's length on the DVD; 4, 7, 8, 12, 3,370, 4,370, 4,372 on the CD; Stop, then a
  seek; Standby; Pause.
- **Referee:** PCSX2 2.8.2 (its lineage alone reaches the drive from the EE; Play! 0.72 answers CCh at every register):
  a fast seek 30.000 ms below 14,764 sectors on the DVD and 4,371 on the CD and a full seek 100.000 ms at and past them,
  exactly ps2tek's bounds; a delta of 15 (DVD) or 7 (CD) sectors 0.168 ms and of 16 or 8 a fast seek, the same either
  way; Stop 166.667 ms and the seek after it 333.33 ms longer; Pause 10 EE cycles. After Standby a short seek took 8.84 ms
  (DVD) and 10.78 ms (CD) until the first fast seek, and Standby itself 9.93 and 14.28 ms.
- **Conclusion:** ps2tek's formula and classes with the contiguous bounds PCSX2 shows (16 and 8, measured); the time of a
  contiguous move is ps2tek's block_timing × delta rather than PCSX2's constant, the document before the single lineage;
  Standby is a seek to sector 0; Pause and NOP complete at once (`timing::PROMPT`). **Measured** for the bounds and the
  classes, **argued** for the contiguous time.
- **Depends on it:** `cdvd/timing.rs`; every load's timing.
- **Pinned by:** `the_formula_and_the_classes_are_ps2teks`, `seeks_take_ps2teks_times_and_raise_the_interrupt_at_their_end`.
- **Implemented in:** stage S23's commit.

### D-121. CDVD ReadDvd: each 2,064-byte block is the sector's ECMA-267 Data Frame, its sector number counted from (030000) at the start of each layer

- **Opened:** 2026-10-25, at stage S23: `Sapphire_Plan.md` §5.6 has DVD reads report "what ECMA-267's sector header
  gives", and ps2tek describes another header.
- **Documents read:** ECMA-267 §16 and §25 (an Opposite Track Path disk numbers layer 1 from layer 0's last inverted);
  ps2tek "CDVD N Commands", ReadDvd: "0 Volume number + 0x20; 1 3 Sector number - volume start + 0x30000, in big-endian;
  4 8 ? (all zeroes); 12 2048 Raw sector data; 2060 4 ? (all zeroes)".
- **Test program:** none that reaches the data: a read waits for the IOP's DMA channel 3 to take each block, and an EE
  program cannot drive it (on PCSX2 ReadDvd stays busy, N status 8Dh, for the whole bounded wait).
- **Referee:** not reachable.
- **Conclusion:** the fields where ps2tek and ECMA-267 agree in place (the ID with the layer in bit 0 of byte 0, then the
  number) are ECMA-267's, with the IED, CPR_MAI and EDC it defines, ps2tek's "?" being unknown rather than zero; the
  number is ps2tek's, from (030000) at each layer's start, since ps2tek describes what the drive hands on and ECMA-267
  what the disc holds; so Sapphire reads a DVD with `TrackPath::Parallel`. **Argued**; open until a program on the IOP
  (S21, S24) reads a block.
- **Depends on it:** `Cdvd::load`, ReadDvd's blocks.
- **Pinned by:** `a_dvd_reads_sector_for_sector_with_the_layer_break_placed`.
- **Implemented in:** stage S23's commit.

### D-122. CDVD ReadDvd: byte 0's 20h is set on a dual-layer disc and clear on a single-layer one

- **Opened:** 2026-10-25, at stage S23.
- **Documents read:** ECMA-267 §16.1 (b29: "ONE if the reflectivity is 40 % max."), §12.6 (Types A and B 45 % to 85 %,
  Types C and D 18 % to 30 %); ps2tek's "Volume number + 0x20" for every disc.
- **Test program, referee:** as D-121, not reachable.
- **Conclusion:** ECMA-267's: b29 by the reflectivity, which the layer count gives; ps2tek's constant 20h matches a
  dual-layer disc's. **Argued**; open with D-121.
- **Depends on it:** `ecma267::sector_information`, `Dvd::read`.
- **Pinned by:** `a_dual_layer_images_second_volume_places_the_layer_break`, `a_dvd_reads_sector_for_sector_with_the_layer_break_placed`.
- **Implemented in:** stage S23's commit.

### D-123. CDVD seeks on a dual-layer disc: the distance is taken across the radius, layer 1's sectors over the layer 0 sectors they lie on, and a change of layer costs at least a fast seek

- **Opened:** 2026-10-25, at stage S23.
- **Documents read:** ECMA-267 §25 and figure 26 (in OTP mode layer 1 runs outward-in from layer 0's outer end), §23
  ("a Middle Zone in each layer to allow the read-out beam to move from Layer 0 to Layer 1"); ps2tek, which gives seeks
  by "delta" and nothing of layers.
- **Test program:** `ncmd_dual`: seeks across a synthetic dual-layer DVD's break (600,000), 599,990 to 600,010 and
  back, and along layer 1.
- **Referee:** PCSX2 2.8.2 times a seek by the logical difference alone: 599,990 to 600,010 (20 sectors) a fast seek,
  600,030 to 599,980 the same, 1,000,000 to 10 a full seek; no cost for the layer.
- **Conclusion:** the title, with the PS2's dual-layer discs taken as OTP: a seek from layer 0's end to layer 1's start
  moves no distance but refocuses, and a seek from sector 0 to layer 1's inner end is a short radial move. **Argued**
  from the disc's geometry; the single lineage does not model layers.
- **Depends on it:** `timing::radius`, `timing::seek`.
- **Pinned by:** `a_dual_layer_seek_is_measured_across_the_radius`, `a_dual_layer_seek_is_timed_across_the_radius`.
- **Implemented in:** stage S23's commit.

### D-124. CDVD MechaconVersion (S command 03h, subcommand 00h) answers the four bytes 03h 06h 02h 00h

- **Opened:** 2026-10-25, at stage S23, as `Sapphire_Plan.md` §5.6 requires: "chosen at S23 by a dispute step from the
  referees' behaviour, never copied from the quotation".
- **Documents read:** ps2tek "CDVD S Commands": four result bytes, "Unknown what the bytes mean. Newer versions of
  CDVDMAN do checks on them of some sort." Its next line, an emulator's value, is excluded as evidence (`DOCS.txt` X.3)
  and was not used.
- **Test program:** `reach` (`Sapphire_Native.md` §25.3): S command 03h sent from the EE with parameter 00h, the status
  and the result bytes read until the status says none is left.
- **Referee:** PCSX2 2.8.2, LRPS2 and PCEE2 (one lineage, three builds) answer 03h 06h 02h 00h; Play! 0.72 reaches no
  drive register from the EE.
- **Conclusion:** the referees' answer, the strongest and only witness; its meaning is not known, so the value is kept
  until S24's BIOS shows whether CDVDMAN accepts it. **Measured** on one lineage; open.
- **Depends on it:** `cdvd::VERSION`; S24.
- **Pinned by:** `the_s_commands_answer_as_the_documents_and_the_record_say`.
- **Implemented in:** stage S23's commit.

### D-125. CDVD S commands: one the drive does not implement answers the one byte 80h, an unknown subcommand of 03h 81h; ForbidDVD answers 05h; the tray request 06h opens with 00h and closes with 01h, answering 00h, and 80h when the tray is already so; OpenConfig takes mode, area and count; the writing commands answer 00h

- **Opened:** 2026-10-25, at stage S23: ps2tek lists eight S commands and the parameters or results of few.
- **Documents read:** ps2tek "CDVD S Commands" (03h:00h, 05h, 08h, 09h, 15h, 40h, 41h, 43h; ForbidDVD's quoted value is
  excluded as evidence) and CDVDMAN's exports (`sceCdTrayReq`, `sceCdOpenConfig(block, mode, block_count, result)`,
  `sceCdWriteConfig`), which name the calls but not their command numbers or byte order.
- **Test program:** `ssweep` (every S command 00h–FFh but 0Fh, sent with no parameters, twice, and on single- and
  dual-layer DVDs), `s_config`, `s_tray` and `s_rtc` (§25.3).
- **Referee:** PCSX2 2.8.2: 202 of the 255 commands answer one byte 80h; 03h with 01h or 02h answers 81h; 0Fh ends the
  run (the console's power, by its effect); 15h answers 05h; 06h with 00h opens the tray (drive status 01h) and with 01h
  closes it (00h), each answering 00h, and 80h for 02h or when already open; OpenConfig's first parameter is the mode (1
  writes, 0 reads), its second the area and its third the count: area 0 and area 1 kept apart, a count's second block
  blank, reading past it zeros; WriteRTC, OpenConfig, WriteConfig, CloseConfig and UpdateStickyFlags answer 00h; ReadRTC
  reads back what WriteRTC wrote. Fifty-three commands answer otherwise (02h with eleven bytes, 36h with fifteen holding a
  region text, 84h and 85h with thirteen bytes of what reads as noise, and others); the answers did not differ between a
  single- and a dual-layer disc.
- **Conclusion:** the referee's answers for the eight commands ps2tek names and for the tray request, whose call
  CDVDMAN exports; every other command answered as absent (80h), MagicGate's and SECRMAN's among them, as §5.6 requires
  of the keys, and the region and model queries too, since no document describes them. **Measured** on one lineage.
- **Depends on it:** `Cdvd::scmd`, `cdvd::nvm`.
- **Pinned by:** `the_s_commands_answer_as_the_documents_and_the_record_say`, `the_tray_opens_closes_and_the_disc_is_detected_as_it_spins_up`, `areas_are_written_and_read_back_by_sessions`.
- **Implemented in:** stage S23's commit.

### D-126. CDVD N command 07h reads a CD's audio sectors whole; a DVD read of a CD and an audio read of a DVD end at once with error 14h; a CD read of a DVD gives its Main Data

- **Opened:** 2026-10-25, at stage S23: the plan's S23 row names CD-DA reads, and ps2tek lists no N command for them.
- **Documents read:** ps2tek "CDVD N Commands" (00h–06h, 08h, 09h); CDVDMAN's export `sceCdReadCDDA`.
- **Test program:** `nsweep` (N commands 00h–1Fh, eleven zero parameters, on a synthetic CD and DVD).
- **Referee:** PCSX2 2.8.2: 07h runs on the CD and ends with error 14h on the DVD; 08h ends with error 14h on the CD;
  06h runs on both; every command given more parameters than it takes ends with error 22h.
- **Conclusion:** 07h is the CD-DA read, its blocks the 2,352-byte raw sectors, timed at ps2tek's CD speed; the medium
  checks and the error value as measured. The parameter count is not checked (a named loss). **Measured** on one
  lineage for the number and the checks, **argued** for the block.
- **Depends on it:** `Cdvd::start`.
- **Pinned by:** `a_cd_reads_its_block_sizes_and_its_audio`.
- **Implemented in:** stage S23's commit.

### D-127. A dual-layer DVD image's layer 1 begins at its first volume's Volume Space Size, where that is a multiple of 16 and a second primary volume descriptor lies 16 sectors past it

- **Opened:** 2026-10-25, at stage S23 (`Sapphire_Plan.md` §9.4: "taken from where the image's second layer begins, by
  the rule ECMA-267 and the PlayStation 2's discs give … a dispute candidate"; `Sapphire_Native.md` §14.8 Q2).
- **Documents read:** ECMA-119 §8.4.8; ECMA-267 §25 and §26.5.1 (bytes 13–15, layer 0's last sector, in the lead-in an
  image does not hold); ps2tek's CDVDMAN exports `sceCdReadDvdDualInfo` and `sceCdLayerSearchFile`.
- **Test program:** the S sweep and the seeks on synthetic single- and dual-layer DVDs (D-123, D-125).
- **Referee:** PCSX2 2.8.2 reports disk type 14h for both and the same S answers; its seeks ignore layers. Nothing it
  shows through the EE tells where it puts a break. Play! not reachable.
- **Conclusion:** the title (`EmuSen_Disc.md` §14.2); S4's builder corrected to it (layer 0's descriptor naming layer 0's
  sectors, layer 1's its own). The one real disc at hand is single-layer and opens as one layer. **Argued**; open until
  a dual-layer disc of the tester's (Q8's God of War) or a program on the IOP shows it.
- **Depends on it:** `Dvd::find_layer_break`; ReadDvd's layer bit and numbering; seeks.
- **Pinned by:** `a_dual_layer_images_second_volume_places_the_layer_break`, `an_image_whose_volumes_place_no_break_is_one_layer`.
- **Implemented in:** stage S23's commit.

### D-128. CDVD registers at rest: the N status 40h, the drive status 0Ah with a disc and 00h without; after the tray closes the drive status 00h, the disk type 01h for half the spin-up and its detecting value for the rest

- **Opened:** 2026-10-25, at stage S23.
- **Documents read:** ps2tek "CDVD I/O Ports" (the N status's bits, bit 2 "DEV9 device connected", bit 3 unknown; the
  drive status's bits; the disk types 01h–04h "Detecting").
- **Test program:** `reach` with no disc, `s_tray`.
- **Referee:** PCSX2 2.8.2: N status 4Ch (bits 2 and 3 set as well as ready), drive status 0Ah with no disc in it as with
  one, disk type 00h without a disc and 12h or 14h with one; 00h right after the tray closes.
- **Conclusion:** bit 2 clear, since the plan leaves DEV9 out (§11 Q12); bit 3 clear, unknown; the drive status 00h with
  no disc, nothing spinning; the closing as PCSX2 shows it; the detecting values' timing a choice [C], since no witness
  shows it. **Argued**.
- **Depends on it:** `Cdvd::new`, `Cdvd::tray`.
- **Pinned by:** `at_power_on_the_registers_report_the_disc`, `the_tray_opens_closes_and_the_disc_is_detected_as_it_spins_up`.
- **Implemented in:** stage S23's commit.

### D-140. SIF, the FIFOs: each of SIF0, SIF1 and SIF2 has a FIFO of 32 words (128 bytes); a channel that would overfill or underrun it holds until the other side's channel moves words

- **Opened:** 2026-10-25, at stage S22.
- **Documents read:** *EE User's Manual* (EEUM) §1.2's table of built-in devices: "SIF 32-bit (address/data
  multiplex), 128-byte FIFO embedded"; *EE Overview* §2.6: "The DMA controllers (DMACs) for the IOP and EE operate in
  cooperation through the bidirectional FIFO (SFIFO) in the SIF", Figure 2-13 placing a "pack/unpack" stage between the
  IOP's 32-bit side and the EE's 128-bit side; EEUM §5.3.3: "the SIF0, SIF1, and SIF2 share the FIFO and might cause a
  deadlock"; ps2tek "SIF DMA": "a set of fifos (First In First Out) located in the SIF, refered to as SIF0 and SIF1",
  "if a SIF0/SIF1 transfer is started on either component and the fifo is empty the transfer continues running until
  the respective fifo starts filling with data".
- **Test program:** `s22-sif1`, the EE's SIF1 of sixteen quadwords started with the IOP's channel stopped, QWC read
  after a delay (eight left, STR set); `s22-sif0`, the IOP's SIF0 of 66 words started with the EE's channel stopped,
  MADR read after a delay (30 data words moved beside the two words of the EE's tag); the unit test
  `a_full_fifo_holds_the_ees_channel_until_the_iop_takes_words`.
- **Referee:** PCSX2 2.8.2 holds neither side: the EE's sixteen quadwords and the IOP's 66 words all leave memory with
  the other side's channel stopped, and are delivered whole once it starts (measured 2026-10-25).
- **Conclusion:** the title. The size is the manual's; one FIFO a channel, of that size, is **argued**: the manual's
  "share the FIFO" is read as the warning it gives (a held channel can deadlock the others), which a FIFO a channel
  does not reproduce, and ps2tek names a FIFO for each of SIF0 and SIF1. The EE moves whole quadwords, four words with
  the first in bits 31..0, and asks for room or words in quadwords; the IOP moves words. PCSX2 is no witness on the
  depth.
- **Depends on it:** `sif::FIFO_WORDS`, `EePorts::request`, `IopPorts::request`.
- **Pinned by:** `sif1_carries_the_ees_packets_to_the_iop_in_every_mode`, `sif0_carries_the_iops_packets_to_the_ee_with_the_ees_tags`,
  `a_full_fifo_holds_the_ees_channel_until_the_iop_takes_words`, `a_quadword_is_four_words_low_first_and_a_full_fifo_takes_no_more`.
- **Implemented in:** stage S22's commit.

### D-141. SIF registers: MSCOM is written by the EE and SMCOM by the IOP, each read by both; MSFLG is set by the EE's writes and cleared by the IOP's, SMFLG the reverse; CTRL reads F0000102h on the EE and F0000002h on the IOP, and the EE's write with bit 18 set raises the IOP's interrupt 1; BD6 is a word both sides write and read; nothing drives the EE's INT_SBUS

- **Opened:** 2026-10-25, at stage S22.
- **Documents read:** ps2tek "SIF Registers": MSCOM "Only writable by EE", SMCOM "Only writable by IOP", MSFLG "IOP
  writes mask", SMFLG "EE writes mask", "If the EE sends 20000h to MSFLG, the IOP can read this and clear MSFLG by
  writing 20000h to it. The opposite is true for SMFLG"; CTRL "1 Always 1? 8 Always 1 for EE, 0 for IOP? 28-31 Always
  0xF? Other Unknown", "Very little is known about this register"; BD6 "Nothing is known about this register"; ps2tek's
  IOP interrupt table (IRQ1 "GPU (used in PSX mode)"); EEUM §3.1 (INT_SBUS, "Detection of an interrupt from a peripheral
  on SBUS", on a falling edge) and chapter 4's Tn_HOLD ("The value of Tn_COUNT is copied when an SBUS interrupt
  occurs"); EEUM §2.2.1 lists only SB_SMFLG of the SIF's registers. No document says what raises INT_SBUS or what a
  write to CTRL does.
- **Test program:** two probes, recorded and not graded, run on PCSX2 only: `s22-probe-iop` (each bit of CTRL written
  by the IOP, with the EE's I_STAT, timer 0's HOLD and CTRL after it; SMFLG and SMCOM written by the IOP; BD6 written by
  each side) and `s22-probe-ee` (each bit of CTRL written by the EE, with the IOP's I_STAT and CTRL). Their first words,
  the registers as the referee's firmware left them, are not used. Then `s22-registers`, graded.
- **Referee:** PCSX2 2.8.2 (measured 2026-10-25): the EE reads CTRL as F0000102h throughout, its own writes changing
  nothing it reads; the IOP reads F0000002h, with bit 8 as the EE last wrote it; the IOP's writes of bits 4 to 7 set
  them in both sides' views, its bit 5 setting bit 13 too; the EE's write of bit 18 sets the IOP's I_STAT bit 1, and of
  bit 19 the IOP stops answering and its RAM reads zero (read as a reset of the IOP); no write of the IOP's (CTRL's 32
  bits, SMFLG, SMCOM) set the EE's I_STAT bit 1, and HOLD stayed 0; BD6 read back what the EE wrote until the IOP wrote
  it, after which both sides read 0. On `s22-registers` PCSX2 differs on 3 of 28 words: it takes the EE's write to SMCOM,
  and reads the IOP's CTRL bit 8 as set after the EE wrote it.
- **Conclusion:** the title. The flags and COMs are ps2tek's words, PCSX2 agreeing but on the EE's SMCOM write, against
  which the document is kept; CTRL's reads are ps2tek's documented bits, PCSX2's further bits not modelled; the IOP's
  interrupt from the EE's bit 18 is the single witness's reading, the only driver the IOP's interrupt 1 has in
  PlayStation 2 mode; the IOP's reset by bit 19 is not modelled (the IOP's boot is S24's); BD6 as plain storage is a
  choice [C]. No witness drives INT_SBUS or HOLD, so neither is driven. **Argued**, bit 18 **measured** on one lineage.
- **Depends on it:** `sif::Sif`'s reads and writes, `sif::CTRL_EE`, `CTRL_IOP`, `CTRL_IOP_INTERRUPT`, `IOP_SOURCE`;
  `IopBus::sif_write_from_ee`.
- **Pinned by:** `the_mailbox_registers_pass_values_and_flags_both_ways`, `the_flags_set_on_one_side_and_clear_on_the_other`.
- **Implemented in:** stage S22's commit.

### D-142. SIF0's tags: the EE's destination tag is the two words the IOP sends after its own tag with CHCR's bit 8, the tag's bits 127..64 not crossing; without bit 8 the data alone crosses and TADR steps over two words

- **Opened:** 2026-10-25, at stage S22.
- **Documents read:** ps2tek "IOP DMA": "If Dn_CHCR.8 is set, 2 words after the tag in memory (TADR + 8) will be
  transferred first. This is used by SIF0 to transfer a DMAtag to the EE's DMAC", "TADR will be incremented by the size
  of the tag, which is either 2 or 4 words"; *EE Overview* §3.1.2: the destination chain tag is 128 bits, bits 127..64
  "(Arbitrary)", and Figure 3-3's "Destination Chain DMA from SIF"; §2.6: "The EE-DMAC reads the packet from the SIF,
  interprets the first word as a tag".
- **Test program:** `s22-sif0`: two IOP tags with bit 8 carrying a cnt and an end tag into an EE destination chain, and
  64 words behind an end tag of sixteen quadwords; `s22-sif0-normal`: an IOP tag without bit 8, two marked words after
  it, into the EE's normal mode.
- **Referee:** PCSX2 2.8.2 places every word of `s22-sif0`'s chains where the two-word reading puts them (a four-word
  tag would shift the data by two words). Without bit 8 it sends the two words after the tag anyway and steps TADR by
  16: the EE's buffer reads EEEE0002h, EEEE0003h, 63000000h, 63000001h, then 63000000h to 63000003h, 9 of 20 words
  differing (measured 2026-10-25).
- **Conclusion:** the title. The two-word EE tag **measured** on one lineage and argued from ps2tek's sentence (the
  quadword's other half never crosses, so nothing of the packet's data is lost to it); bit 8's choice is ps2tek's,
  against PCSX2, whose second quadword repeating two words is no hardware rule anyone has written.
- **Depends on it:** `EePorts::tag_ready`, `get_tag`; `dmac::Peripherals`'s two methods of that name; `iop::dma`'s chain.
- **Pinned by:** `sif0_carries_the_iops_packets_to_the_ee_with_the_ees_tags`.
- **Implemented in:** stage S22's commit.

### D-143. SIF1's tags on the IOP: a chain into memory takes a quadword of four words from the FIFO as its tag, the first two the IOP's tag, whatever CHCR's bit 8

- **Opened:** 2026-10-25, at stage S22, by D-68, which left the tags of a chain into memory to this stage.
- **Documents read:** ps2tek "IOP DMA" (the tag's two words; bit 8 described for a chain from memory only); *EE
  Overview* §2.6 and Figure 2-14 (the SIF's packets drawn with their tags in 128-bit rows); EEUM §5.1 (SIF1 a source
  chain, moving quadwords).
- **Test program:** `s22-sif1`, its first version with a two-word IOP tag before ten words in three quadwords, its
  second with each EE packet's first quadword the IOP's tag and two marked words; both with bit 8 clear and set.
- **Referee:** PCSX2 2.8.2, on the first version, took the first four words as the tag whether bit 8 was set or not, so
  the data landed two words late; on the second, every word equal (measured 2026-10-25).
- **Conclusion:** the title, **measured** on one lineage and argued: the EE's packets are whole quadwords, so a tag of
  two words would leave every later packet half a quadword out of step. The last two words are dropped. ~~Two words,
  four with bit 8~~ (S21's chain to memory, written before the FIFO existed).
- **Depends on it:** `iop::dma`'s `chain_tag_words` and chain.
- **Pinned by:** `sif1_carries_the_ees_packets_to_the_iop_in_every_mode`, `each_dma_mode_reaches_its_stand_in`.
- **Implemented in:** stage S22's commit.

### D-144. IOP chains, the IRQ tag: a tag with IRQ and without the end bit sets the channel's DICR2 flag and the chain goes on (D-68 kept); PCSX2 ends the transfer there

- **Opened:** 2026-10-25, at stage S22.
- **Documents read:** ps2tek "IOP DMA": bit 30 "IRQ - raises IQE interrupt in DICR2 when all words in this tag are
  transferred", bit 31 "End of transfer, raises transfer completion interrupt", "This works independently of the
  channel's interrupt mask"; "DICR.31 = DICR.15 | (DICR.23 && (DICR.24-30 || DICR2.24-29)".
- **Test program:** the last part of `s22-sif1` (an EE chain of three packets, the IOP's second tag with IRQ) and of
  `s22-sif0` (two IOP tags, the first with IRQ), each last so that a stop leaves nothing behind for a later part.
- **Referee:** PCSX2 2.8.2 sets the DICR2 flag and raises interrupt 3, and ends the IOP's transfer at the IRQ tag on
  both channels: the third SIF1 packet is never taken and the second SIF0 packet never sent, the EE's destination chain
  left waiting (14 words of the two programs). Its DICR bit 31 reads 0 with a DICR2 flag set and the master enable on
  (one word) (measured 2026-10-25).
- **Conclusion:** D-68's reading kept, the title: ps2tek gives the IRQ and end bits separate effects, and its formula
  puts DICR2's flags in bit 31. PCSX2 against, its stop indistinguishable from the document's reading to a program
  that always sets both bits. **Argued**; S24's BIOS shows which its modules rely on.
- **Depends on it:** `iop::dma`'s chain.
- **Pinned by:** `sif1_carries_the_ees_packets_to_the_iop_in_every_mode`, `sif0_carries_the_iops_packets_to_the_ee_with_the_ees_tags`.
- **Implemented in:** stage S22's commit.

### D-145. SIF2: the EE's channel 7 and the IOP's channel 2 meet in one FIFO of 32 words, each side's direction its own channel's DIR, the EE in normal mode, the IOP in any mode

- **Opened:** 2026-10-25, at stage S22.
- **Documents read:** EEUM §5.1: SIF2 "both", priority group B, slice, no chain; §5.9: DIR "Effective only on VIF1
  (ch-1) and SIF2 (ch-7) channels"; ps2tek: "SIF2 (bidirectional, used for PSX mode and debugging)" on the EE, "SIF2
  (EE<->IOP, GPU in PSX mode)" on the IOP.
- **Test program:** `s22-sif2`: two quadwords from the EE into the IOP's slice of two blocks of four, then eight words
  from the IOP's slice into the EE's normal mode.
- **Referee:** PCSX2 2.8.2 moves nothing on either channel, both left running (26 of 32 words differing; measured
  2026-10-25). No lineage is a witness.
- **Conclusion:** the title, on the documents alone; **argued**.
- **Depends on it:** `sif::EePorts` (SIF2's direction from the EE channel's CHCR.DIR), `IopPorts`.
- **Pinned by:** `sif2_carries_data_both_ways`.
- **Implemented in:** stage S22's commit.

### D-160. GS, Gouraud shading: a pixel's colour is the vertices' colours interpolated exactly at its centre and rounded down

- **Opened:** 2026-10-25, at stage S17.
- **Documents read:** *GS User's Manual* §3.1 ("The RGBA value, Z value, texture value and Fog value for each pixel are
  calculated from the gradient obtained in the preprocessing stage"), §3.2.8, §2.4.4 (pixels centred where the window
  coordinates are whole), RGBAQ's page (8-bit channels). No precision is given for the gradient or the DDA's
  accumulator, and no rounding.
- **Test program:** `s17-gouraud` (a Gouraud triangle over a whole PSMCT32 page from colours at both ends of their
  range, the same into PSMCT24 and PSMCT16, Gouraud lines in every direction), and the Gouraud primitives of
  `s17-triangles`, `s17-lines`, `s17-formats`, `s17-depth-32`, `s17-depth-16` and `s17-kick`.
- **Referee:** PCSX2 2.8.2 floors as the reading does but departs by one: on the page-sized triangle 141 of its 4,208
  channel values, every one of them within 1/16 of a whole number (74 a step low just above one, 67 a step high just
  below), and so on the other programs' Gouraud pixels (their colour differences are all of one step, in PSMCT16 one
  step of a 5-bit channel). No fixed-point DDA of 8 to 24 fractional bits, anchored at the origin or at each row, and no
  single-precision plane reproduces PCSX2's values (the nearest leaves 48 differing), so the departures are taken as its
  arithmetic's precision [A]. Play! 0.72 is no witness (`Sapphire_Referees.md`, S17's note).
- **Conclusion:** exact barycentric weights at the centre, rounded down; **argued**. The documents give the rule and not
  the precision; an exact reference is the one every faster implementation can be measured against, and PCSX2's
  departures are a step either way of it, not a different rounding. A second lineage's pictures (Play! through the
  probe's reserved OpenGL context, `Sapphire_Plan.md` §10) would weigh it; a game's gradient would not decide it.
- **Depends on it:** `gs::raster::triangle`, `gs::raster::line`; S18's texture coordinates and fog, interpolated the same
  way; the device path's exactness (S44 to S46, C5).
- **Pinned by:** `gouraud_colour_and_z_are_interpolated_exactly_and_rounded_down`, `generated_primitives_draw_as_the_model_draws_them`.
- **Implemented in:** stage S17's commit.

### D-161. GS, lines: an end of a line on the edge of a pixel's diamond lies in no pixel's area

- **Opened:** 2026-10-25, at stage S17.
- **Documents read:** *GS User's Manual* §3.2.9 and figures 3-4 and 3-5: "By defining the area for each pixel as shown
  with small gray squares in Figure 3-5, the pixels located in the areas where a line goes through are drawn. However, a
  pixel located at the endpoint of a line is not drawn", the start drawn; the areas are diamonds, |dx| + |dy| < 1/2
  about each centre, touching at their corners. Whether a diamond's edges belong to it is not said.
- **Test program:** `s17-lines`: 32 lines, among them ends at (+5, −3) sixteenths from a centre; its line strip and
  meeting lines, drawn with GREATER over Z already written at their Z so that a pixel drawn twice keeps its first
  colour, with vertices at (+4, −4) and (+4, +4); `s17-gouraud`'s radiating lines, whose starts lie at (−2, +6), (−8,
  0) and (0, −8) from a centre.
- **Referee:** PCSX2 2.8.2 agrees with the reading at every interior pixel of every line and at the ends that lie inside
  a diamond or in the spaces between. Of the ends on an edge, it gives the upper-right edge to the diamond (four
  places, six ends: three starts drawn and three ends' columns left out, where the reading leaves out the start's
  column and draws the end's), and treats starts on a lower-left edge, at a left corner and at a top corner, and a
  vertex on a lower-right edge, as the reading does. Play! 0.72 is no witness.
- **Conclusion:** an end on an edge lies in no diamond: the start's pixel is then the first column crossed and the end's
  column is drawn when the line crosses it; **argued**: the figure draws closed shapes touching only at corners, and
  giving the edges to the diamond as PCSX2 does for one of the four would need a rule no document states. A crossing of
  a column exactly at a corner shared by two diamonds takes the lower one for an x-major line and the right one for a
  y-major one, as a half rounds up; PCSX2 agrees at every such crossing the programs make.
- **Depends on it:** `gs::raster::line`'s end test; nothing else.
- **Pinned by:** `lines_draw_the_diamonds_they_pass_through_but_their_end`, `generated_primitives_draw_as_the_model_draws_them`.
- **Implemented in:** stage S17's commit.
- **Amended 2026-10-25, at stage S18:** `s18-uv`'s textured line from (2.5, 0.5) to (3.75, 31.25) ends at (−1/4, +1/4)
  from pixel (4, 31)'s centre, on its diamond's lower-left edge; PCSX2 2.8.2 leaves that pixel undrawn, giving the edge to
  the diamond, where the reading draws the end's row. A second edge PCSX2 gives to its diamond; the conclusion stands.

### D-162. GS, XYOFFSET: it is subtracted at the drawing kick, the offset of the drawing's context, from every vertex of the primitive

- **Opened:** 2026-10-25, at stage S17.
- **Documents read:** *GS User's Manual* §2.4.3 (window = primitive − offset, the offset in XYOFFSET), §3.3.1 (XYOFFSET
  among the drawing environment's registers), §3.3.2's second example (PRMODE's CTXT set to 1 before one vertex of a
  strip, "Drawing 2nd Triangle with Context 2", and back to 0, "Drawing 3rd Triangle with Context 1"); ps2tek "GS Vertex
  Attributes": "When a vertex kick occurs, XYOFFSET is subtracted from the vertex's X and Y coordinates."
- **Test program:** `s17-kick`: a strip whose fourth vertex is kicked under context 2 by PRMODE, with XYOFFSET_2 differing
  from XYOFFSET_1 by (−5.1875, 2.5625); context 2 by PRIM's CTXT, and by PRIM's while PRMODECONT selects PRMODE.
- **Referee:** PCSX2 2.8.2 equals the drawing-kick reading in every pixel (but one-step Gouraud values, D-160); under
  ps2tek's reading the strip's triangles and the context-2 picture differ from it in 205 words of the two pictures and
  the Z buffer.
- **Conclusion:** at the drawing kick, every vertex by the drawing's context; **measured**, PCSX2 with the manual's
  example, whose third triangle (vertices 2 to 4, one kicked under context 2) is drawn as an ordinary context-1 triangle
  only under this reading. ps2tek's sentence is the same where XYOFFSET does not change inside a primitive.
- **Depends on it:** `gs::draw::Gs::vertex_kick`.
- **Pinned by:** `the_vertex_queue_prmode_and_the_contexts_follow_section_3_2`.
- **Implemented in:** stage S17's commit.

### D-163. GS, the depth test: a pixel's Z beyond its Z buffer format's range is clamped to the format's largest value, for the test and the write

- **Opened:** 2026-10-25, at stage S17.
- **Documents read:** *GS User's Manual* §2.3.2 and §2.5.3 (Z32, Z24, Z16), §3.7.4, XYZ2's page (Z 32 bits), XYZF2's
  (24 bits). Nothing on a Z wider than the buffer's format.
- **Test program:** `s17-zrange`: Z = 01234567h written with ALWAYS into PSMZ24 and PSMZ16, a Gouraud triangle with Z
  from 0 to FFFFFFFFh beside it, and a sprite at 00FFFFFFh tested with GREATER across both.
- **Referee:** PCSX2 2.8.2: equal, every word of both pictures and both Z buffers.
- **Conclusion:** clamped; **measured**, one lineage. Keeping the format's low bits instead would wrap a far surface
  near.
- **Depends on it:** `gs::pixel::z_max`, `gs::pixel::draw`.
- **Pinned by:** `z_beyond_a_formats_range_is_clamped`.
- **Implemented in:** stage S17's commit.

### D-164. GS, Z interpolation: a pixel's Z is the vertices' Z interpolated exactly at its centre and rounded down, in all 32 bits

- **Opened:** 2026-10-25, at stage S17.
- **Documents read:** as D-160; Z is "calculated from the gradient" with no precision given.
- **Test program:** `s17-gouraud` (a quad with Z from 10000000h to F0000000h read back from PSMZ32), the depth programs'
  Gouraud-Z triangles in every Z format.
- **Referee:** PCSX2 2.8.2 departs in the low bits: by one or two in 1,037 of the quad's 2,048 words, by up to 203 in
  869 PSMZ32 words of `s17-depth-32` (about 2⁻²⁴ of the range, a single-precision float's [A]), by one in PSMZ24 and
  PSMZ16. Every pass and fail of every depth test is the same in both, including the equalities GEQUAL and GREATER
  divide.
- **Conclusion:** exact, rounded down; **argued**, as D-160.
- **Depends on it:** `gs::raster`; the device path's exactness.
- **Pinned by:** `gouraud_colour_and_z_are_interpolated_exactly_and_rounded_down`, `every_depth_test_holds_in_psmz32_and_psmz24`.
- **Implemented in:** stage S17's commit.

### D-165. GS, Gouraud lines: a pixel whose centre lies beyond an end takes the colour the line's gradient gives there, held to 0 to 255, and Z likewise to its 32 bits

- **Opened:** 2026-10-25, at stage S17.
- **Documents read:** as D-160; §3.2.9 draws the start's pixel, whose centre may lie before the start along the major
  axis.
- **Test program:** `s17-lines`' line 20, from (3, 2) to (5, 4) sixteenths from a centre, which draws that centre's
  pixel 1.5 of its own lengths before its start.
- **Referee:** PCSX2 2.8.2: the colour continued along the line and saturated (EFh, 16h, 64h, 07h to 63h, 8Ah, 70h, 1Bh
  gives FFh, 00h, 52h, 00h), not the start's colour.
- **Conclusion:** continued and held to the channel's range; **measured**, one lineage; a DDA started at the first pixel's
  centre gives it.
- **Depends on it:** `gs::raster::line`.
- **Pinned by:** `lines_draw_the_diamonds_they_pass_through_but_their_end`.
- **Implemented in:** stage S17's commit.

*D-180 on are stage S14's, built beside S22 (from D-140), S17 (from D-160) and S20 (from D-200).*

### D-180. UNPACK's fields the manual leaves indeterminate: V2 writes X and Y again into Z and W; V3 writes 0 into W

- **Opened:** 2026-10-25, at stage S14, by §6.3.4's figures ("Indeterminate" in V2's Z and W and V3's W) and §6.3.7's
  example 4, which shows the indeterminate fields written and added to Row unless masked.
- **Documents read:** *EE User's Manual* (Version 5.0) §6.3.4, §6.3.6, §6.3.7; ps2tek "VIF UNPACK" ("Indetermin.").
- **Test program:** ps2autotests' `dma/vif/unpack` text, a console's (`Sapphire_References.md` §2): every V2 line prints
  X, Y, X, Y; every V3 line prints 0 in W; one vector each, over a memory whose earlier contents the text does not show.
  `s14-unpack`: three vectors of each format over a memory filled with a pattern first.
- **Referee:** PCSX2 2.8.2 equals the console's lines and `s14-unpack`'s V2 fields; in V3's W it writes the element after
  the vector in 8 of each VIF's 15 vectors and 0 in the others, by no rule the alignments show. Play! 0.72 writes 0 in V2's Z and W
  (against the console) and in V3's W.
- **Conclusion:** V2: Z = X and W = Y, **measured** (the console's text, PCSX2 beside it). V3: W written 0, the console's
  single vector and Play!, **argued**; PCSX2's partial reading of the next element is recorded, not followed.
- **Depends on it:** `vif::unpack::decode`.
- **Pinned by:** `the_unpack_console_text_holds`, `every_unpack_format_equals_section_6_3_4`.
- **Implemented in:** stage S14's commit.

### D-181. CYCLE with WL 0: one block that never ends, every write taking a vector into the next qword, the write cycles counting 1 to 4 and staying at 4

- **Opened:** 2026-10-25, at stage S14: §6.3.5's formulas divide by WL.
- **Documents read:** §6.3.5, the CYCLE register's page, the UNPACK page's length formula.
- **Test program:** ps2autotests' `dma/vif/stcycl` text: every line with WL 0, whatever CL, shows eight consecutive
  qwords written in cycles 1, 2, 3, 4, 4, 4, 4, 4 (its masks fill from Col, so it shows where and in which cycle, not
  what is read); `dma/vif/unpack`'s "No data" lines show an UNPACK waiting for data (VPS 1) after a reset, which leaves
  CYCLE 0 if the program set nothing. `s14-cycle` with CL 1 and CL 0 under WL 0.
- **Referee:** PCSX2 equals the `stcycl` lines; in `s14-cycle` it takes more words than four V4-16 vectors (the MARK
  after them read as data). Play! takes fewer.
- **Conclusion:** WL 0 compares below every CL, so the write is a skipping one whose block never closes: each write takes
  a vector, at ADDR + k, its cycle min(k, 3); the reading that WL 0 counts as 256 (filling, CL vectors read) fits the
  `stcycl` text but would complete the "No data" UNPACK at once after a reset. **Argued**.
- **Depends on it:** `vif::unpack::Cycle`.
- **Pinned by:** `the_stcycl_console_text_holds`, `skipping_and_filling_writes_equal_section_6_3_5`.
- **Implemented in:** stage S14's commit.

### D-182. MODE 3: the input written unadded and kept in Row

- **Opened:** 2026-10-25, at stage S14: VIFn_MODE's page calls 11 "Undefined".
- **Documents read:** §6.3.7, VIFn_MODE's page.
- **Test program:** ps2autotests' `dma/vif/stmod` text: "mode 3 (no mask)" writes 0 to 15 and leaves Row at the last
  vector, C, D, E, F; with the diagonal mask the Row fields the mask writes keep their values (Row 1000h written in the
  first cycle's X, then the X, Y, Z of earlier vectors). Reproduced on both VIFs (`the_stmod_console_text_holds`) and in
  `s14-mask`.
- **Referee:** PCSX2 equal; Play! leaves Row unchanged and writes it.
- **Conclusion:** the title, for fields written as is; **measured** (the console's text, PCSX2 beside it).
- **Depends on it:** `vif::unpack::write`.
- **Pinned by:** `the_stmod_console_text_holds`, `the_mask_and_the_addition_modes_equal_sections_6_3_6_and_6_3_7`.
- **Implemented in:** stage S14's commit.

### D-183. A filling write decompresses the vector at the stream's pointer without taking it, and its 0 specifiers write that vector through MODE; words that have not arrived read 0

- **Opened:** 2026-10-25, at stage S14: §6.3.5 says only that "the data read is suspended" and the fields filled "according
  to the specification of the MASK register"; a 0 specifier on a filling write has no input.
- **Documents read:** §6.3.5, §6.3.6, §6.3.7 ("Processing to add the Row register value is performed only when the m bit
  of the CMD field is 0 or the mask specifier of the corresponding field is 0").
- **Test program:** `s14-cycle`: V3-32 under CL 2 and WL 4 without the m bit, so three filling writes have only 0
  specifiers, one of them after the last vector, where the stream holds a MARK.
- **Referee:** PCSX2 writes in each of the three the vector its next read takes (and in the last the MARK's word and the
  NOPs after it); Play! writes 0.
- **Conclusion:** the title, the decoder reading at a pointer that does not advance; the stronger witness (PCSX2 equals
  the VIF's console texts but where §28.7 of the build record says) and one mechanism for the read and the fill.
  **Argued**, PCSX2 the witness.
- **Depends on it:** `vif::unpack::Unpack::next`, `Vif::peek`.
- **Pinned by:** `skipping_and_filling_writes_equal_section_6_3_5`.
- **Implemented in:** stage S14's commit.

### D-184. VIF1's double buffer: OFFSET sets TOPS to BASE and clears DBF; the MSCAL family copies TOPS to TOP, reverses DBF and sets TOPS to the buffer DBF then names (0 BASE, 1 BASE + OFFSET), all 10 bits wide

- **Opened:** 2026-10-25, at stage S14: the manual's pages disagree. OFFSET's page, §6.3.8 and VIF1_TOPS's page set TOPS
  at OFFSET; VIF1_OFST's page says "the VIF1_BASE register value is transferred to the VIF1_TOP register". VIF1_TOPS's
  page ("if the DBF flag ... is 0, the VIF1_BASE value is set again ... Since the DBF flag value is reversed at this
  time") leaves open whether DBF is read before or after its reversal.
- **Documents read:** §6.3.8, the OFFSET, BASE and MSCAL pages, VIF1_BASE's, VIF1_OFST's, VIF1_TOP's, VIF1_TOPS's and
  VIF1_STAT's (DBF: "0 TOPS=BASE, 1 TOPS=BASE+OFFSET") pages; ps2tek "VIF Commands".
- **Test program:** `s14-dbuf`: BASE and OFFSET, then MSCAL, MSCAL, MSCNT and OFFSET, each microprogram storing XTOP and
  XITOP at the qword TOP names, UNPACKs with FLG between; BASE 3F0h with OFFSET 20h.
- **Referee:** PCSX2 equal on every register and memory word but TOPS after BASE 3F0h and OFFSET 20h, which it keeps as
  410h (its UNPACK lands at the same qword, VU1's memory wrapping); Play! reads every double-buffer register as 0.
- **Conclusion:** the title: TOP is left by OFFSET (the majority of the pages, and PCSX2), DBF read after its reversal
  (STAT's field definition); **measured** for both (PCSX2), **argued** for TOPS's width (the register's page).
- **Depends on it:** `Vif::fetch` (OFFSET), `Vif::wait` (the MSCAL family).
- **Pinned by:** `vif1_s_double_buffer_equals_section_6_3_8`.
- **Implemented in:** stage S14's commit.

### D-185. NUM keeps 8 bits on VIF0 and 16 on VIF1

- **Opened:** 2026-10-25, at stage S14: both NUM pages draw bits 7:0, but VIF1's counts DIRECT's IMMEDIATE, up to 65,536.
- **Documents read:** VIF0_NUM's and VIF1_NUM's pages.
- **Test program:** ps2autotests' `dma/vif/unpack` text: "No data UNPACK(NUM: 256)" prints NUM 0 on VIF0 and 256 on VIF1.
- **Referee:** PCSX2 prints 256 on both (the one line of the text it misses); Play! 0 on VIF1.
- **Conclusion:** VIF0 8 bits, **measured** (the console); VIF1 more than 8, **measured**, 16 **argued** (DIRECT's count).
- **Depends on it:** `Vif::num_bits`.
- **Pinned by:** `the_unpack_console_text_holds`, `every_unpack_format_equals_section_6_3_4`.
- **Implemented in:** stage S14's commit.

### D-186. STAT's VPS and FQC: VPS 0 idle, 1 a command waiting for its data, 2 a VIFcode held by a stall (the i bit's, an undefined code's) or a command waiting on the VU or the GIF, 3 data in hand; FQC the FIFO's qwords without the one being taken apart

- **Opened:** 2026-10-25, at stage S14: VPS's four values name states the manual does not tie to the commands.
- **Documents read:** VIFn_STAT's pages.
- **Test program:** ps2autotests' `dma/vif/intstall` text: after a VIFcode with the i bit, STAT reads 00000C02 (VPS 2,
  VIS, INT) with FQC 0 and the DMA ended; `dma/vif/unpack`: an UNPACK with no data reads VPS 1. `s14-regs`.
- **Referee:** PCSX2 counts the qword being taken apart in FQC (01000C02 for the console's 00000C02) and reports VPS 3
  for VIF1's dataless UNPACK written through the FIFO; Play! reads 0 in most states.
- **Conclusion:** the title; **measured** for VPS 1 and 2 and FQC (the console's texts), **argued** for the waits.
- **Depends on it:** `Vif::status`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`, `the_unpack_console_text_holds`.
- **Implemented in:** stage S14's commit.

### D-187. FBRST.RST empties the FIFO, drops the VIFcode in progress and clears STAT (but DBF and FDR), NUM, CODE and ERR; CYCLE, MODE, MASK, Row, Col, ITOPS, ITOP, MARK and the double buffer's registers are kept

- **Opened:** 2026-10-25, at stage S14: "Resets the whole VIF0 (including VIF0-FIFO)".
- **Documents read:** VIFn_FBRST's and VIFn_STAT's pages.
- **Test program:** ps2autotests' `dma/vif/intstall` text: the STMASK case, after the MARK case set MRK, reads STAT without
  MRK. `s14-regs`: every register read after RST.
- **Referee:** PCSX2 and Play! keep CYCLE, MODE, MASK, Row, Col, ITOPS and MARK; PCSX2 clears ERR and CODE and keeps MRK;
  Play! keeps ERR.
- **Conclusion:** the title: the registers both lineages keep, kept; MRK cleared as the console's text shows if its
  program resets between cases (its source is not read); ERR as PCSX2, the stronger witness. **Argued**.
- **Depends on it:** `Vif::fbrst`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`.
- **Implemented in:** stage S14's commit.

### D-188. Undefined VIFcodes: with ERR.ME1 clear, ER1, INT_VIF and a stall at the code, STC going on after it; with ME1 set, a NOP. VIF1's own codes are undefined on VIF0, MSCALF not; an UNPACK with vl 3 and vn other than 3 is undefined

- **Opened:** 2026-10-25, at stage S14: §6.3.2's list marks OFFSET, BASE, MSKPATH3, FLUSH, FLUSHA, MSCALF, DIRECT and
  DIRECTHL "VIF1 only", but MSCALF's own page gives "VIF0/VIF1"; §6.3.4's table has no format for 63h, 67h and 6Bh.
- **Documents read:** §6.3.2, §6.3.4, §6.4's pages, VIFn_ERR's and VIFn_STAT's pages; §3.1's INTC table ("VIF0's
  detection of VIFcode with an interrupt bit or an exception").
- **Test program:** `s14-regs`: 08h, OFFSET on VIF0 and 63h on both, with ME1 clear and set.
- **Referee:** PCSX2 sets no ER1, raises no interrupt and runs the MARK after each; Play! sets ER1 and raises the
  interrupt but does not stall, and STC leaves ER1 set.
- **Conclusion:** the title, MSCALF by its page; **argued** (the documents), Play! a partial witness for ER1 and the
  interrupt.
- **Depends on it:** `Vif::fetch`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`.
- **Implemented in:** stage S14's commit.

### D-189. The i bit: INT and the interrupt as its VIFcode ends; the next VIFcode but after MARK is fetched and held with VIS until STC; ERR.MII ignores the bit wholly; the DMA goes on filling the FIFO meanwhile

- **Opened:** 2026-10-25, at stage S14: "the processing of the following VIFcode (excluding the VIFcode MARK) stalls"
  reads two ways; MARK's own page says "No stall".
- **Documents read:** §6.3.2 and every page of §6.4, VIFn_ERR's MII, VIFn_STAT's VIS and INT, VIFn_FBRST's STC.
- **Test program:** ps2autotests' `dma/vif/intstall` text, three lines a VIF: NOP with the i bit leaves MASK unchanged and
  STAT 00000C02, MARK with it changes MASK and reads 00000840 (INT, MRK, no stall), STMASK with it changes MASK and
  stalls; the text prints no DMA warning, so the DMA ended. Read as one-qword packets (their source is not read), the
  shape `s14-regs` sends, with a four-qword packet whose later qwords wait in the FIFO.
- **Referee:** in the text's own program PCSX2 holds the DMA (its warnings: "DMA transfer timed out"), counts the qword
  in FQC and, on VIF1, leaves MASK unchanged in MARK's and STMASK's cases; in `s14-regs` it equals every MASK and holds
  the four-qword packet's DMA. Play! raises no interrupt for the i bit and runs nothing held after STC.
- **Conclusion:** the title; **measured** for which codes stall (the console's text, the MARK reading MARK's page gives),
  **argued** for MII and for the FIFO.
- **Depends on it:** `Vif::complete`, `Vif::fetch`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`.
- **Implemented in:** stage S14's commit.

### D-190. FBRST.FBK stalls at once and FBRST.STP after the VIFcode in process (at once when none is), each holding the FIFO's data until STC

- **Opened:** 2026-10-25, at stage S14.
- **Documents read:** VIFn_FBRST's pages ("Causes a ForceBreak ... (Stall occurrence)", "Stops after end of VIFcode in
  process"), VIFn_STAT's VSS and VFS.
- **Test program:** `s14-regs`: FBK while an UNPACK waits for data, then its data; STP while idle, then a MARK; STP while
  an UNPACK waits, then its data and a MARK.
- **Referee:** PCSX2 sets VFS and VSS but goes on in each case; Play! does not stop.
- **Conclusion:** the title, the pages' words; **argued**.
- **Depends on it:** `Vif::fbrst`, `Vif::step`, `Vif::fetch`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`.
- **Implemented in:** stage S14's commit.

### D-191. MARK keeps 16 bits, the EE's write as the VIFcode's

- **Opened:** 2026-10-25, at stage S14.
- **Documents read:** VIFn_MARK's page (MARK 15:0).
- **Test program:** `s14-regs`: the EE writes 155AAh.
- **Referee:** PCSX2 and Play! both read 155AAh back.
- **Conclusion:** 16 bits, the register's page before two lineages that do not mask a write; **argued**.
- **Depends on it:** `Vif::write`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`.
- **Implemented in:** stage S14's commit.

### D-192. ITOPS and ITOP keep 10 bits on both VIFs, and XITOP reads ITOP whole

- **Opened:** 2026-10-25, at stage S14: VU0's 256 qwords need 8.
- **Documents read:** VIFn_ITOP's and VIFn_ITOPS's pages (9:0 on both), XITOP's page.
- **Test program:** `s14-micro`: ITOP 2ABh, then MSCAL of a microprogram storing XITOP.
- **Referee:** PCSX2 keeps 2ABh in VIF0's ITOPS but reads ABh from ITOP and from XITOP; Play! reads its registers as 0
  and XITOP as 2ABh.
- **Conclusion:** the pages' 10 bits; **argued**.
- **Depends on it:** `Vif::fetch` (ITOP), `Vif::wait`.
- **Pinned by:** `mpg_and_the_mscal_family_start_and_wait_for_the_vus`.
- **Implemented in:** stage S14's commit.

### D-193. A chain with TTE hands a VIF its tags' upper 64 bits as stream words; ER0 is not raised

- **Opened:** 2026-10-25, at stage S14: §6.3.1's figure places VIFcodes beside the DMAtag; VIFn_STAT's ER0 "cannot be used
  appropriately due to a failure in determining DMAtag Mismatch error".
- **Documents read:** §6.3.1, §5.2.3's Figure 5-3, VIFn_ERR's ME0.
- **Test program:** `s14-regs`: cnt and end tags carrying MARK and STMOD in their upper halves.
- **Referee:** PCSX2 and Play! equal.
- **Conclusion:** the title; **measured** for TTE, ER0 a named loss (the manual's own advice is to mask it).
- **Depends on it:** `Vif::word`.
- **Pinned by:** `the_registers_the_interrupt_stall_fbrst_and_the_error_masks`.
- **Implemented in:** stage S14's commit.

### D-194. The VIF's time: BUSCLK, one bus cycle a VIFcode and one a write (an UNPACK qword, an MPG instruction, a DIRECT qword); a microprogram the MSCAL family starts issues its first pair 3 EE cycles after; a wait on the VU ends the cycle after its last pair's T stage

- **Opened:** 2026-10-25, at stage S14: no document times the VIF.
- **Documents read:** chapter 6 whole; the *VU User's Manual*'s §5.4.5 (VCALLMS's start).
- **Test program:** `the_dmac_waits_on_a_full_fifo`: an S-32 UNPACK of 256 words, 65 qwords by DMA into VIF1's
  16-qword FIFO, ends at 402 EE cycles, the VIF's pace (392 for the 49 qwords it must take first) and not the DMAC's
  130 [M].
- **Referee:** not consulted: neither lineage's timing is a witness for the console's.
- **Conclusion:** the title; **argued**, a named loss.
- **Depends on it:** `Vif::step`, `Vif::wait`; every transfer's timing.
- **Pinned by:** `the_dmac_waits_on_a_full_fifo`.
- **Implemented in:** stage S14's commit.

### D-200. The PCRTC's NTSC and PAL timing: a line of 858 and 864 pixels of 13.5 MHz (3,432 and 3,456 VCK of 54 MHz); 525 and 625 lines a frame interlaced, a field half of it; 263 and 314 lines a non-interlaced field; the video clock its own, not BUSCLK's

- **Opened:** 2026-10-25, at stage S20, by D-42 and by the referees' fields (`s20-timing`).
- **Documents read:** *GS User's Manual* §5.1.1 (Figure 5-1: the sync generator fed by its own "Clock NTSC/PAL" through a
  PLL), §5.1.2's table (NTSC 59.940 Hz interlaced and "(59.82)" non-interlaced at 15.734 kHz; PAL 50.000 and "(49.76)" at
  15.625 kHz; DTV 480p 59.94 Hz at 31.469 kHz, 1080i 60.00 Hz at 33.750 kHz), §5.2.1 ("the standard video clock VCK which
  defines the PCRTC operation timing", "set by the kernel service"), §5.2.2's MAGH table (640 pixels at ×4, 512 at ×5,
  320 at ×8, 256 at ×10: 2,560 VCK across); ps2tek "EE Timers" (262 lines of 9,370 BUSCLK, PAL 312 of 9,436) and "IOP
  Timers" (timer 0's 13.5 MHz pixel clock "regardless of screen mode").
- **Arithmetic** [A]: 858 pixels of 13.5 MHz make NTSC's 15,734.27 Hz line; 262.5 lines give 59.940 Hz and 263 give
  59.826 Hz, the table's two figures to their printed digits; PAL's 864 pixels make 15,625 Hz exactly, 312.5 lines
  50.000 Hz and 314 lines 49.761 Hz. VCK at four pixels, 54 MHz, puts the MAGH table's 2,560 VCK at 47.4 µs of a
  52.6 µs active line, and 294.912 MHz × 375 / 2048 = 54 MHz exactly, so every edge falls at a fixed fraction of the EE's
  clock. 480p at 1,716 VCK and 525 lines, and 1080i at 2,200 VCK of 74.25 MHz and 1,125 lines, reproduce their rows too.
  ps2tek's 262 × 9,370 BUSCLK is a 60.065 Hz field, matching neither row.
- **Test program:** `s20-timing`, in four settings of SMODE2 (as the firmware left it, INT 0, INT 1 with FFMD 0 and 1):
  the fields' lengths by COP0 Count, timer 0 on H-blank, timers 1 and 2 on BUSCLK/256, the second gated off during
  H-blank, the H-syncs by CSR's HSINT, CSR's FIELD at each edge.
- **Referee:** PCSX2 2.8.2 (Americas BIOS): every field 4,915,200 EE cycles, exactly 1/60 s, in all four settings, so
  SMODE2's INT changes nothing; Play! 0.72: every field 4,915,200 ± 5 likewise. The two lineages agree with each other
  and with no document.
- **Conclusion:** the documents' table read at 13.5 MHz, as the title; **argued**. The referees' agreement on a round
  1/60 s, unchanged by INT, is the two emulators' own figure where Sony's table gives another, so it does not rule (the
  order of recourse puts the documents first). The figures the table does not give are choices, each named in
  `Sapphire_Native.md` §29.3: a non-interlaced field's extra half line, the 4× of VCK, 1080i's 74.25 MHz.
- **Depends on it:** `pcrtc::timing`'s modes and `clock::VCK_54`, `VCK_74`; every timer on H-blank, the INTC's V-blank
  sources, the IOP's timers; the machine's frame rate (60,000/1,001 interlaced, as S1 chose); the CRT filter's timing
  (`Sapphire_Plan.md` §4.6).
- **Pinned by:** `the_four_rates_of_the_manuals_table_follow_from_13_5_mhz`, `the_timing_program_sees_the_documents_fields`.
- **Implemented in:** stage S20's commit.

### D-201. H-blank reaches the timers once a line; its width is the line less BT.601's 720 pixels, 552 VCK in NTSC and 576 in PAL

- **Opened:** 2026-10-25, at stage S20, by D-42's question: PCSX2 counts 525 H-blanks between two V-blank starts.
- **Documents read:** ps2tek "EE Timers" ("9370 BUSCLK cycles per scanline", H-blank a clock and a gate of the timers);
  the *GS User's Manual* (no width given); ITU-R BT.601's 720 active pixels of 858 and 864, the 13.5 MHz family's own
  (general knowledge of the standard, [A]).
- **Test program:** `s20-timing`'s H-blank ticks a field, H-syncs a field, the Count at six successive H-syncs and six
  successive H-blank ticks, and timer 2's share of BUSCLK/256 with H-blank gating it off.
- **Referee:** **PCSX2** raises both H-blank and HSINT every 9,362–9,364 EE cycles (4,681 BUSCLK), twice NTSC's line
  rate, 525 of them in its 1/60 s field, in every setting of SMODE2; its H-blank is high for 16.3 % of that period. So its
  525 is a frame's lines counted in one field: H-blank at 31.47 kHz, not a line counted twice by the timer. **Play!**
  raises them every 19,200 EE cycles with irregular gaps, 262 or 263 ticks a field, and does not gate the timer on
  H-blank at all.
- **Conclusion:** one H-blank a line, as ps2tek's "per scanline" and Play! have it, at the documents' line (D-200), so a
  field counts 263, or 262 and 263 alternately when interlaced; its width BT.601's blanking, 16.1 % of the line, a choice
  [C] within 0.2 % of PCSX2's share. **Argued**; D-42's 525 is explained as measured.
- **Depends on it:** `pcrtc::timing`'s `hblank` and the edges; the EE and IOP timers clocked or gated by H-blank.
- **Pinned by:** `the_timing_program_sees_the_documents_fields`, `edges_fall_once_each_and_a_change_waits_for_the_field`.
- **Implemented in:** stage S20's commit.

### D-202. V-blank is high for 22 lines (NTSC) or 26 (PAL) from a field's start; the V-sync, which raises VSINT and changes CSR's FIELD, falls 4 lines in

- **Opened:** 2026-10-25, at stage S20.
- **Documents read:** ps2tek "EE Timers" ("VBOFF: 240 | VBON: 22", PAL "286 | 26"); *GS User's Manual* §7.2 (VSINT,
  "Vsync interrupt"; NFIELD "updated by sampling at VSync"; FIELD "Field Displayed Currently"); *EE User's Manual* §3.1
  (VB_ON and VB_OFF at V-blank's start and end). No document places the V-sync inside V-blank.
- **Test program:** `s20-timing`: Count from VB_ON to VB_OFF, and one polling pass from VB_ON timing VSINT and FIELD's
  change.
- **Referee:** PCSX2: V-blank 430,665 EE cycles (23 NTSC lines); VSINT and FIELD both 74,894 cycles, 4.0 lines, after
  VB_ON. Play!: V-blank 489,600 cycles (26.1 lines); VSINT at VB_ON; FIELD after VB_OFF. Both change FIELD inside each
  field's V-blank, not at its start.
- **Conclusion:** ps2tek's 22 and 26 lines; the V-sync 4 lines into the field, PCSX2 the stronger single witness (its
  timing is modelled to the half line where Play!'s moves in steps of 4,800 cycles); FIELD changes at the V-sync and
  VB_ON is the field's start [C]. **Argued**, the V-sync's place **measured** on one lineage.
- **Depends on it:** `pcrtc::timing::Mode`'s `vblank`, `vsync`; the board's VB_ON, VB_OFF, VSINT and CSR's FIELD.
- **Pinned by:** `hsint_and_vsint_follow_the_pcrtcs_timing`, `the_timing_program_sees_the_documents_fields`.
- **Implemented in:** stage S20's commit.

### D-203. The merge circuit rounds to the nearest: OUT1 = (Rs·A + Rd·(FFh − A) + 7Fh) / FFh

- **Opened:** 2026-10-25, at stage S20.
- **Documents read:** *GS User's Manual* §5.2.3: "Ro = Rs x A + Rd x (0xff – A) … The above results are multiplied by
  256(0x100) / 255(0xff)", with no rounding stated.
- **Test program:** `s20-alp` (ALP 60h over BGCOLOR) and `s20-alpha` (circuit 1's alpha doubled over PSMCT16S).
- **Referee:** PCEE2 (PCSX2's lineage, its software renderer's picture through the probe): of `s20-alp`'s 1,535
  colours, 1,448 are produced from the program's own pixels by rounding to the nearest (in each of several equivalent forms),
  217 by truncating the quotient and 204 by the ×257 approximation; the other 87 by no blend of one pixel, PCEE2 having
  scaled that picture to 48 × 64 (§29.8). Play!'s picture cannot be taken (S3's decision).
- **Conclusion:** to the nearest, PCSX2's lineage the single witness; the manual's sentence read as a scale, not a
  truncation. **Measured** on one lineage.
- **Depends on it:** `pcrtc::merge`.
- **Pinned by:** `the_display_programs_pictures_equal_the_manuals` (`s20-alpha`, `s20-alp`).
- **Implemented in:** stage S20's commit.

### D-204. CSR's NFIELD reads 0; FIELD reads the field begun at the last V-sync, 0 even, alternating in both scan modes

- **Opened:** 2026-10-25, at stage S20.
- **Documents read:** *GS User's Manual* §7.2: NFIELD "Output Value of NFIELD Output (Output value is updated by
  sampling at VSync.)"; FIELD 0 EVEN, 1 ODD, "(In Non-Interlace Mode) 0 Equivalent to EVEN 1 Equivalent to ODD".
- **Test program:** `s20-timing`'s CSR bits 12 and 13 at every V-blank start and end in the four settings.
- **Referee:** PCSX2 and Play! both read NFIELD 0 throughout, and FIELD alternating field by field in every setting.
- **Conclusion:** NFIELD 0, the two lineages agreeing where the document gives no value (an active-low copy of the
  field signal would be another reading, untested); FIELD alternating with each field in both modes, the even field
  the one whose lines start on the frame's H-syncs and fill the even rasters [C]. **Measured** for NFIELD and FIELD's
  alternation, argued for which field is even.
- **Depends on it:** `gs::Gs::csr`, `Gs::vsync`, `pcrtc::timing::Timing::raster`.
- **Pinned by:** `the_even_field_by_csr_reads_the_even_rasters`, `the_timing_program_sees_the_documents_fields`.
- **Implemented in:** stage S20's commit.

### D-205. The read circuits read each raster at its line's time, so a register changed in the middle of a field shows from the next line

- **Opened:** 2026-10-25, at stage S20.
- **Documents read:** *GS User's Manual* §5.2.2: the circuit "reads the specified rectangular area in the frame buffer
  adjusting to CRT scanning"; `Sapphire_Plan.md` §5.4.
- **Test program:** `s20-midframe`: DISPFB1 set to buffer A at each V-sync and to buffer B forty H-syncs later.
- **Referee:** PCEE2 shows buffer B on every row: PCSX2's lineage reads the display once a frame with the registers as
  they then stand.
- **Conclusion:** at each line's start, as the manual's sentence and the plan have it; the one lineage that can show it
  departs. Within a line the change takes effect from the next line (a named loss, §29.6). **Argued**.
- **Depends on it:** `pcrtc::Pcrtc::read_due`, the board's line events.
- **Pinned by:** `a_register_changed_mid_field_shows_from_the_next_line`.
- **Implemented in:** stage S20's commit.

### D-206. Where circuit 1 gives nothing (outside its rectangle, or EN1 clear), OUT1 is what it would blend over: OUT2, or BGCOLOR with SLBG

- **Opened:** 2026-10-25, at stage S20.
- **Documents read:** *GS User's Manual* §5.2.3: OUT2 is circuit 2 inside its rectangle and BGCOLOR outside it or with
  the circuit off; nothing is said of circuit 1 outside its rectangle.
- **Test program:** `s20-pair` (the circuits side by side), `s20-ct32` (BGCOLOR around circuit 1).
- **Referee:** PCEE2's `s20-pair` picture holds exactly the 5,017 colours Sapphire's does, circuit 2's pixels among them
  where circuit 1 is absent; its picture is framed differently (138 × 104 against 154 × 56), so only the colours are
  compared.
- **Conclusion:** as the title, circuit 1's alpha taken as 0 there [A], PCSX2's lineage agreeing. **Measured** on one
  lineage.
- **Depends on it:** `pcrtc::merge`.
- **Pinned by:** `the_display_programs_pictures_equal_the_manuals`.
- **Implemented in:** stage S20's commit.

### D-220. SIO2, the ports: SEND3's two port bits select one of four ports, 0 and 1 the controller connectors and 2 and 3 the card slots; a peripheral byte that nothing on its port answers reads HiZ

*D-220 to D-230 were opened at stage S25, part A (`Sapphire_Native.md` §30), numbered after the stages built beside it
(S17 from D-160, S14 from D-180, S20 from D-200).*

- **Opened:** 2026-10-25, at stage S25, by ps2tek's two-bit port field and psx-spx's note on the PlayStation 2's ports.
- **Documents read:** ps2tek "SIO2 Registers" (SEND3 bits 0-1 "Port"; SEND1/SEND2 "Port1/2 Control?", four of each by
  the address rule); psx-spx "PS2 multitap" ("plugs into both the controller and memory card ports (which are not
  wired in parallel on the PS2)").
- **Test program:** `s25-probe` (a controller read and a card probe on each of the four ports); `s25-card`'s last
  exchanges (the slot 2 card on port 3, a controller's command on port 3).
- **Referee:** PCSX2 2.8.2 answers by the port's low bit: the controller in port 1 answers on ports 0 and 2, the card in
  slot 1 on ports 0 and 2, and so for port 3 (measured 2026-10-25). One lineage, against the four ports.
- **Conclusion:** the title, **argued** from the two documents: four port selects for four connectors not wired
  together. PCSX2's routing accepts either reading; S24's BIOS (PADMAN, MCMAN) shows which ports it uses, and a BIOS
  that sent cards' commands on ports 0 and 1 would show this reading wrong at once.
- **Depends on it:** `sio2::Sio2::start`'s routing.
- **Pinned by:** `the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes`, `the_dualshock_2_answers_identification_configuration_and_modes_as_documented`.
- **Implemented in:** stage S25 part A's commit.

### D-221. SIO2, RECV1: 1100h after a queue whose peripherals all answered; after one that met no device, 2D100h when the missing peripheral's byte is a controller's (01h) and 1D100h otherwise

- **Opened:** 2026-10-25, at stage S25, by ps2tek's value failing its own rule.
- **Documents read:** ps2tek "SIO2_RECV1": "(RECV1 & F000h) == 1000h after a memory card command, the memory card is
  connected"; "(RECV1 & 2000h) == 0 after a pad command, the pad is connected"; "Known value for a disconnected
  peripheral is 1D100h. Known value for a connected peripheral is 1100h". 1D100h has bit 13 clear, so by the second rule
  it reads as a connected pad.
- **Test program:** `s25-probe`, `s25-pad` (port 1 empty), `s25-tap-*` (the multitap's byte with no tap).
- **Referee:** PCSX2 2.8.2: 1100h for every answered command; 2D100h for a controller's command to an empty port;
  1D100h for the multitap's byte 21h with no tap (measured 2026-10-25). RECV2 reads 0000000Fh, RECV3 0, and the
  register at 1F808280h 1 after a queue.
- **Conclusion:** the title, **measured** on one lineage and argued: both values meet both of ps2tek's rules, and
  ps2tek's 1D100h stands for the non-controller case. Which bits a queue of several commands leaves is not documented;
  Sapphire leaves the last missing peripheral's [C].
- **Depends on it:** `sio2::{CONNECTED, NO_PAD, NO_DEVICE}`.
- **Pinned by:** `the_dualshock_2_answers_identification_configuration_and_modes_as_documented`, `the_authentication_commands_get_no_answer`.
- **Implemented in:** stage S25 part A's commit.

### D-222. The memory card, the reply's place: the card answers byte k of a command (k counted from the peripheral byte 81h) with byte k - 2 of ps2tek's reply list, and HiZ (FFh) for the peripheral and command bytes; a command sent two bytes longer than ps2tek's command list receives the whole reply

- **Opened:** 2026-10-25, at stage S25, by ps2tek's lists, which give a command's bytes and its reply's but not where
  the one meets the other.
- **Documents read:** ps2tek "SIO2 PS2 Memcards" (every command's "Command", "Reply" and "Length"; 26h's reply "2B + 2
  + 2 + 4 + 1 + Terminator" eleven bytes for a command of eleven); ps2tek "SIO2 Controller Commands" ("the reply comes
  immediately after the appropriate byte has been sent"); psx-spx "Controllers and Memory Cards" (a card drives
  nothing until it has its address byte: HiZ).
- **Test program:** `s25-probe` and `s25-probe-write`, every reply read with eight bytes more; `s25-card`.
- **Referee:** PCSX2 2.8.2 places ps2tek's list from byte 2 for every command but the reads and writes (11h, 12h, 21h to
  23h, 26h, 27h, 28h, BFh, F3h), its last byte (the terminator) pushed into FIFOOUT one past the transfer's end; it
  answers 00h, not FFh, to the peripheral and command bytes; for 42h and 43h it gives 00h, 2Bh at bytes 2 and 3 where
  the list puts 2Bh and the terminator, the data from byte 4 as the list does, and for a short read the checksum and
  terminator straight after the data rather than at the list's fixed places (measured 2026-10-25; of `s25-card`'s 848
  graded words, 19 differ in the first two bytes alone, 40 in the reads' and writes' first bytes, 5 in short reads).
- **Conclusion:** the title, **argued** from the documents and the one lineage's placement of the data: the reading in
  which every list fits its command's length and a reply byte depends only on the bytes before it. A byte past what is
  clocked is never received, so the programs send two bytes more than ps2tek's command lists.
- **Depends on it:** `sio2::card::MemoryCard::exchange`.
- **Pinned by:** `the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes`.
- **Implemented in:** stage S25 part A's commit.

### D-223. The DualShock 2, 43h outside configuration mode: it answers as 42h does, without the pressure bytes, before entering configuration mode

- **Opened:** 2026-10-25, at stage S25.
- **Documents read:** ps2tek "SIO2 Controller Commands" ("Reply: header reply (with mode byte) + same as Read Data
  (but without pressure values in DS2 mode)"); psx-spx "Normal Mode - Command 43h" ("the response is same as for
  command 42h (button data) (and analog inputs when in analog mode)").
- **Test program:** `s25-pad`, 43h from digital and from analogue mode with every reply byte enabled.
- **Referee:** PCSX2 2.8.2 answers the mode byte and 00h for every following byte (two words, measured 2026-10-25).
- **Conclusion:** the title, on both documents; **argued**. PCSX2 against.
- **Depends on it:** `sio2::pad::DualShock2::exchange`.
- **Pinned by:** `the_dualshock_2_answers_identification_configuration_and_modes_as_documented`.
- **Implemented in:** stage S25 part A's commit.

### D-224. The DualShock 2, configuration replies psx-spx gives and PCSX2 does not: 46h with an index past 1 answers zeros; 48h with index 0 or 1 answers 01h at byte 7 (counted from 0); 44h with a mode past 1 answers FFh at byte 5; 40h keeps the value written and answers the old one, FFh for an index past 0Bh; byte 8 of 40h's and 4Fh's reply is not graded

- **Opened:** 2026-10-25, at stage S25.
- **Documents read:** psx-spx "Config Mode - Command 46h", "48h", "44h" ("Dualshock2 sets Err=FFh upon Led=02h..FFh"),
  "40h" ("Set NEW Val, array[Idx]=Val"; "Old Val (or FFh when Idx>0Bh)"), and its note "Some Dualshock2 Config Mode
  commands do occasionally send 00h, 5Ah, or FFh as last (9th) reply byte"; ps2tek gives none of these.
- **Test program:** `s25-pad`.
- **Referee:** PCSX2 2.8.2: 46h index 2 answers index 1's bytes; 48h answers zeros; 44h with mode 2 answers zeros; 40h
  answers 02h whatever was written and for index 0Ch; the ninth byte of 4Fh and 40h is 5Ah (six words, measured
  2026-10-25).
- **Conclusion:** the title, psx-spx's, the one document; **argued**. PCSX2 against on each.
- **Depends on it:** `sio2::pad::DualShock2::exchange`.
- **Pinned by:** `the_dualshock_2_answers_identification_configuration_and_modes_as_documented`.
- **Implemented in:** stage S25 part A's commit.

### D-225. The DualShock 2, 41h: in analogue mode FFh FFh 03h 00h 00h 5Ah, the bytes 4Fh can enable, whatever 4Fh last set; in digital mode zeros

- **Opened:** 2026-10-25, at stage S25, by the two documents' disagreement.
- **Documents read:** psx-spx "Command 41h": "Reply HiZ F3h 5Ah FFh FFh 03h 00h 00h 00h", "a constant bitmask
  indicating which reply bytes can be enabled"; ps2tek "41h - Query Masked Mode": "header reply + 3 mask bytes + 00 +
  00 + 5A", "used to find out what buttons are included in poll responses", zeros in digital mode.
- **Test program:** `s25-pad`: 41h in digital mode, after 44h's analogue mode (reply bytes 3Fh), after 4Fh (3FFFFh).
- **Referee:** PCSX2 2.8.2: zeros in digital mode; FFh FFh 03h 00h 00h 5Ah after 44h and after 4Fh alike (measured
  2026-10-25; a first version of the program graded ps2tek's current mask, 3Fh, and differed there).
- **Conclusion:** the title, **measured** on one lineage: psx-spx's constant for the three bytes, ps2tek's zeros in
  digital mode and its last byte 5Ah.
- **Depends on it:** `sio2::pad::DualShock2::exchange`.
- **Pinned by:** `the_dualshock_2_answers_identification_configuration_and_modes_as_documented`.
- **Implemented in:** stage S25 part A's commit.

### D-226. The card's error correction: each parity of Ridge's 20-bit code is stored complemented, the unused bits clear, so that an erased chunk's code is 77h 7Fh 7Fh

- **Opened:** 2026-10-25, at stage S25, by Ridge's silence on the stored form and a referee's spare bytes.
- **Documents read:** Ross Ridge, "PlayStation 2 Memory Card File System", "Error Correction Code" (the column parities
  in the first byte, even groups low and odd high; the line parities' even and odd groups in the second and third; four
  codes in the spare area's first twelve bytes); nothing on whether a parity is stored as such or complemented.
- **Test program:** `s25-probe` and `s25-probe-write`, pages read with their spare bytes from a folder card.
- **Referee:** PCSX2 2.8.2's folder card, which keeps no spare bytes of its own, answers 77h 7Fh 7Fh four times and
  FFh FFh FFh FFh for every page's spare area, whatever was written (measured 2026-10-25): the code of an erased chunk
  (FFh bytes) in the complemented form, and the code an all-zero chunk has too. Its file card stores the spare bytes as
  written, as a flash does.
- **Conclusion:** the title, **measured** for the erased chunk on one lineage and argued for the rest: an erased page
  then carries a code that checks, which the plain form would not. The bits' order inside each byte is Ridge's
  description read in the natural order, unwitnessed until part B reads a card the BIOS formatted.
- **Depends on it:** `emusen_ps2_card::ecc`.
- **Pinned by:** `an_erased_chunks_code_reads_77h_7fh_7fh_and_one_wrong_bit_is_put_right`.
- **Implemented in:** stage S25 part A's commit.

### D-227. The DualShock 2's sticks: the reply's bytes 6 to 9 are the right stick's X and Y, then the left stick's X and Y

- **Opened:** 2026-10-25, at stage S25.
- **Documents read:** psx-spx "Standard Controllers" (halfword 2 RightJoyX, RightJoyY; halfword 3 LeftJoyX, LeftJoyY)
  and its 4Fh table in the same order; ps2tek "Input Format" lists "right y, right x, left y, right x", which names the
  right stick's X twice.
- **Test program:** `s25-pressure` grades the sticks centred; the order is checked by `v1`'s `set_axis` test.
- **Referee:** not a witness: PCSX2 given no input centres every stick at 7Fh.
- **Conclusion:** psx-spx's order, ps2tek's list taken as a slip; **argued**.
- **Depends on it:** `sio2::pad::Input`, `v1::set_axis`.
- **Pinned by:** `the_two_cards_are_slots_of_ps2_card_and_a_card_file_of_the_wrong_size_is_refused`.
- **Implemented in:** stage S25 part A's commit.

### D-228. The multitap: its peripheral byte is 21h, answered on both ports of its slot; 12h and 13h answer FFh 80h 5Ah 04h 00h 5Ah, 21h and 22h FFh 80h 5Ah 00h 00h 00h 5Ah; which slot 21h and 22h select is Sapphire's choice

- **Opened:** 2026-10-25, at stage S25, by the two documents' different peripheral bytes and the absence of any
  document of the commands.
- **Documents read:** ps2tek "SIO2 Registers" ("21h - Multitap", "61h - Infrared"); psx-spx "PS2 multitap"
  ("accepting switching commands with address 61h", the protocol otherwise unknown).
- **Test program:** `s25-tap-21-0`, `s25-tap-21-2`, `s25-tap-61-0` (every command byte 00h to 2Fh and five more under
  each peripheral byte, with and without a multitap), `s25-tap-slots` (each selection followed by a controller read
  and a card probe), `s25-tap`.
- **Referee:** PCSX2 2.8.2 with a multitap in port 1: 21h answers 12h, 13h, 21h and 22h as the title says, on port 0
  and port 2, and nothing else; 61h answers nothing; no selection changes what the controller and card ports then
  answer (measured 2026-10-25).
- **Conclusion:** ps2tek's byte, the one witness's frame, **measured**; the selection (the byte after the command's
  00h, 0 to 3, routing the controller port's traffic for 21h and the card port's for 22h, any other value none) is a
  choice [C] with no witness, the multitap's slots past A reachable through it only.
- **Depends on it:** `sio2::tap`, `sio2::Slot`.
- **Pinned by:** `a_multitap_answers_its_frame_and_routes_the_slot_it_selects`.
- **Implemented in:** stage S25 part A's commit.

### D-229. The memory card, three readings: 27h answers the terminator it replaces; 28h's last byte is 55h whatever the terminator; 82h erases the whole erase block holding the address 21h gave

- **Opened:** 2026-10-25, at stage S25.
- **Documents read:** ps2tek "27h - Set Terminator" ("Reply: XX + 2B + Old terminator"), "28h - Get Terminator" ("2B +
  Terminator + 55"), "82h - Erase Block" ("Erases data at the address given by command 21h ... (512 + 16) * 16 bytes");
  Toshiba's TC58128AFT data sheet (a block erase takes a block's address, the page's bits ignored); Ridge "NAND Flash
  Memory Basics".
- **Test program:** `s25-card`: the terminator set to AAh and back; a block erased by its first page and later by its
  last.
- **Referee:** PCSX2 2.8.2 answers the new terminator to 27h (two words) and the terminator, not 55h, as 28h's last
  byte (one word); an erase from page 47 leaves pages 32 and 33 as they were, so it erases from the address given rather
  than the block holding it (273 words of the last two page reads) (measured 2026-10-25).
- **Conclusion:** the title, ps2tek's and the flash's, **argued**; PCSX2 against on each. A BIOS that erases only by a
  block's first page cannot tell the erase readings apart.
- **Depends on it:** `sio2::card::MemoryCard::exchange`.
- **Pinned by:** `the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes`, `a_write_clears_bits_and_an_erase_sets_its_whole_block`.
- **Implemented in:** stage S25 part A's commit.

### D-230. SIO2, SEND3's length: bits 18 to 26 hold a command's length counting its peripheral byte

- **Opened:** 2026-10-25, at stage S25, by ps2tek's field being too narrow for its own commands.
- **Documents read:** ps2tek "SIO2_SEND3": "8-16 Fake command length", "18-24 Real command length (counting peripheral
  byte)"; ps2tek "43h - Read Data": a command of 132 bytes after the peripheral byte, more than seven bits hold.
- **Test program:** `s25-card`, every read and write a command of 134 bytes.
- **Referee:** PCSX2 2.8.2 sends and answers all 134 bytes (measured 2026-10-25), so it reads at least eight bits.
- **Conclusion:** nine bits, as wide as the fake length's field, **argued**; the bits above 24 are otherwise unused.
- **Depends on it:** `sio2::LENGTH_MAX`.
- **Pinned by:** `the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes`.
- **Implemented in:** stage S25 part A's commit.

### D-240. GS, the texture function: with TCC RGB every function keeps the fragment's alpha, DECAL included

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** *GS User's Manual* §3.4.9's table, whose DECAL row gives "Av = At" under TCC=RGB as under RGBA;
  §3.4.3: "The TCC field determines whether to use the A value of the texture during the texture function execution";
  TEX0's page (TCC 0 "RGB", 1 "RGBA"); ps2tek "GS Textures": DECAL with RGB "Av = Av", the fragment's.
- **Test program:** `s18-tfx`: each of the four functions with TCC 0 and 1 over a PSMCT32 texture of generated alpha
  and 32 flat fragment colours whose alpha runs from 0 to FFh.
- **Referee:** PCSX2 2.8.2: every word of all eight pages equal to the reading, DECAL with TCC RGB keeping the
  fragment's alpha.
- **Conclusion:** the fragment's alpha; **measured**, one lineage, with §3.4.3's definition of TCC and ps2tek, against
  the table's DECAL entry, which would make TCC mean nothing for DECAL.
- **Depends on it:** `gs::texture::Sampler::function`.
- **Pinned by:** `every_texture_function_with_tcc_follows_the_table_of_3_4_9`.
- **Implemented in:** stage S18's commit.

### D-241. GS, the LOD: a LOD of 0 or less takes MMAG at level 0, above 0 MMIN

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** *GS User's Manual* §3.4.12: "When the calculated LOD value is 0.0 or less, the level 0 texture is
  used, and the filter specified in the MMAG field ... When the LOD value is more than 0.0 ... MMIN"; TEX1's page: MMAG
  "Filter when Texture is Expanded (LOD < 0)", MMIN "when Texture is Reduced (LOD >= 0)". The two disagree at 0.
- **Test program:** `s18-filter`'s fourth page: an 8×8 texture magnified with LCM 1 and K = 0, −1/16 and +1/16, MMAG
  nearest and MMIN linear, and K = 0 with MMAG linear; its fifth page: LODs of 7/16, 8/16, 15/16 and 1 with mipmapping.
- **Referee:** PCSX2 2.8.2 takes MMAG at 0, as §3.4.12 says; it also takes MMAG at +1/16, where both documents say MMIN;
  at 7/16 it blends levels 0 and 1 (MMIN 3).
- **Conclusion:** MMAG at 0 or less; **measured** at 0, one lineage with §3.4.12. PCSX2's MMAG at 1/16 departs from both
  documents and is taken as its own threshold [A].
- **Depends on it:** `gs::texture::Sampler::sample`.
- **Pinned by:** `bilinear_lod_and_the_mipmap_filters_work_in_sixteenths`.
- **Implemented in:** stage S18's commit.

### D-242. GS, the LOD's arithmetic: (log2(1/|Q|) × 2^L + K) exact, floored to sixteenths; with STQ the pixel's interpolated Q, with UV the drawing kick's vertex's; levels blended by the fraction in sixteenths

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** TEX1's page: LOD = (log2(1/|Q|) << L) + K, K int1:7:4; §3.4.12; §3.4.4's table ("MIPMAP (Q Linear
  Interpolation)": possible with STQ, impossible with UV); §3.4.12: "When LCM is set to 0, LOD is calculated from the Q
  value even if the texture coordinates are given by UV". No precision is given for the logarithm or the blend.
- **Test program:** `s18-filter`: four quarters of constant Q (1, 0.5, 0.3, and 0.3 with L = 2 and K = −2) under MMIN 2
  and 3; a quad in perspective, Q from 1 to 1/8, under MMIN 3 and 5; a UV sprite with Q 0.25 and a UV triangle whose
  last vertex has Q 1 and the others 0.25.
- **Referee:** PCSX2 2.8.2: equal where the LOD is a whole level; with Q 0.3 its blends fit a LOD of 1 + 188/256 and,
  with L = 2, 4 + 242/256, the exact values held to eight fractional bits, where the reading holds four (27/16 and 79/16);
  per pixel in perspective it departs in 2,021 of 2,048 words, and eight-bit logarithms of the exactly interpolated Q
  reproduce only 294 of them, so its per-pixel Q or LOD is computed otherwise [A]; with UV it draws level 0 for the
  sprite whose Q gives level 2, LOD = K, against the manual's sentence, and equals the reading on the triangle, where the
  last vertex's Q gives LOD 0.
- **Conclusion:** the LOD exact, floored to K's fixed point; Q per pixel with STQ, the drawing kick's vertex's with UV;
  **argued**: K's format is the one fixed point the documents give the LOD, and §3.4.12 says Q is used with UV. PCSX2's
  eight bits are its arithmetic's.
- **Depends on it:** `gs::texture::Setup`, `gs::texture::lod_of`, `gs::exact::log2_scaled`.
- **Pinned by:** `bilinear_lod_and_the_mipmap_filters_work_in_sixteenths`, `generated_textured_primitives_draw_as_the_model_draws_them`.
- **Implemented in:** stage S18's commit.

### D-243. GS, the CLUT's load: only a TEX0 or TEX2 whose PSM is indexed loads, 16 or 256 entries, under CLD's codes against CBP0 and CBP1; codes 6 and 7 load nothing

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.7's "Load Operation" ("CLUT data is loaded ... when the TEX0 or TEX2 register is accessed"),
  its CLD table and TEX0's and TEX2's pages; §2.7.1 (16 entries for IDTEX4, 256 for IDTEX8). Nothing says what a
  non-indexed PSM loads, nor what CLD 6 and 7 do.
- **Test program:** `s18-clut`: a PSMCT32 TEX0 with CLD 1 and another CBP between a load and a draw with CLD 0; CLD 6;
  CLD 2, 0, 4, 4, 3, 5, 5 and 4 with the CLUT's memory rewritten between draws.
- **Referee:** PCSX2 2.8.2: every word equal; the PSMCT32 TEX0 loads nothing, CLD 6 loads nothing, and CBP0 and CBP1
  are kept apart.
- **Conclusion:** as stated; **measured**, one lineage.
- **Depends on it:** `gs::texture::Clut::load`.
- **Pinned by:** `the_clut_buffer_loads_as_cld_says_and_keeps_what_it_loaded`.
- **Implemented in:** stage S18's commit.

### D-244. GS, the CLUT buffer's addressing: entries at CSA × 16 wrap within 256 places of 32 bits or 512 of 16; CSA applies in CSM2; a CPSM TEX0's page does not list reads as PSMCT32

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.7's tables (CSA 0 to 15 for PSMCT32, 0 to 31 for the 16-bit formats; a 32-bit entry's halves
  at k and 256 + k), TEX0's page ("In CSM2 mode, CSA=0 must be set"; CPSM 0000, 0010, 1010), §2.7.4.
- **Test program:** `s18-clut`: PSMT4 with a PSMCT32 CLUT at CSA 20, PSMT8 with a PSMCT16 CLUT at CSA 24, CSM2 with CSA
  3, CPSM 0001; a PSMCT16 CLUT at CSA 20 over a PSMCT32 one read afterwards.
- **Referee:** PCSX2 2.8.2: every word equal.
- **Conclusion:** as stated; **measured**, one lineage.
- **Depends on it:** `gs::texture::Clut`, `gs::texture::clut_format`.
- **Pinned by:** `the_clut_buffer_loads_as_cld_says_and_keeps_what_it_loaded`.
- **Implemented in:** stage S18's commit.

### D-245. GS, wrap modes: REGION_CLAMP takes MIN and then MAX, so MAX wins when MIN exceeds it; at level n both region modes shift their parameters right by n

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.5's formulas ("u < MINU: u' = MINU ... u > MAXU: u' = MAXU") and its sentence on levels for
  REGION_CLAMP ("MINU >>n, MINV >>n, MAXU >>n, and MAXV >>n"); nothing for REGION_REPEAT at levels or for MIN above MAX.
- **Test program:** `s18-clamp`: REGION_CLAMP with MINU 9, MAXU 5, MINV 12, MAXV 6; REGION_CLAMP and REGION_REPEAT at
  level 1.
- **Referee:** PCSX2 2.8.2: every word equal to MIN then MAX (2,048 of 2,048, where MIN-first-and-stop gives 372), and to
  the shifted parameters at level 1 in both modes.
- **Conclusion:** as stated; **measured**, one lineage, the formulas applied in their order.
- **Depends on it:** `gs::texture::Wrap::apply`.
- **Pinned by:** `every_wrap_mode_acts_on_whole_texels_at_each_level`.
- **Implemented in:** stage S18's commit.

### D-246. GS, MTBA: levels 1 to 3 follow each other, each the level's texels' bits rounded up to whole blocks, its width the level's texels over 64 and at least 1, written into MIPTBP1

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.11's "Automatic Calculation of Base Pointer" and its PSMCT16 512×512 example (7200, 9248,
  9760, 9888, widths 8, 4, 2, 1); TEX1's page (MTBA "Base address of TBP1 - TBP3 is automatically set"). No formula, and
  nothing for a buffer wider than the texture or a level smaller than a block.
- **Test program:** `s18-filter`'s last two pages: 64×64 PSMCT32 with TBW 1, levels 1 to 3 by K, then MTBA cleared and
  level 1 read through MIPTBP1; the same with TBW 4; 32×32 PSMT8, whose level 3 is a quarter of a block.
- **Referee:** PCSX2 2.8.2: every word equal.
- **Conclusion:** as stated; **measured**, one lineage, and the manual's example.
- **Depends on it:** `gs::texture::auto_miptbp1`.
- **Pinned by:** `bilinear_lod_and_the_mipmap_filters_work_in_sixteenths`.
- **Implemented in:** stage S18's commit.

### D-247. GS, texture coordinates: S, T and Q exact, interpolated exactly, divided exactly and floored to sixteenths of a texel; UV interpolated exactly and floored

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.10 (S, T and Q interpolated, then s = S/Q), §2.6.2 (texel coordinates in sixteenths, a texel's
  centre at .5), ST's and RGBAQ's pages (float 32), UV's (int 0:10:4); ps2tek "GS Textures": S and T's "lower 8 bits are
  rounded down to zero", a statement with no source. No precision is given for the interpolation or the division.
- **Test program:** `s18-stq` and `s18-uv`, and every textured primitive of the other programs.
- **Referee:** PCSX2 2.8.2: equal but at 112 words of `s18-stq` and 4 of `s18-uv`, every one a coordinate exactly on a
  whole texel or above it by 1.5 × 10^-8 to 3.6 × 10^-7 of a texel, where it takes the texel below.
- **Conclusion:** exact; **argued**, as D-160: the documents give the arithmetic, not its precision, and PCSX2's
  departures lie within single precision's reach of a boundary. Zero exponent fields as zero and the largest as an
  ordinary exponent [A]; ps2tek's dropped bits are not followed, having no source.
- **Depends on it:** `gs::exact`, `gs::texture::Setup`.
- **Pinned by:** `st_and_q_divide_exactly_after_interpolation`, `uv_coordinates_are_interpolated_exactly_on_every_primitive`.
- **Implemented in:** stage S18's commit.

### D-248. GS, fog: each colour (F × C + (100h − F) × FOGCOL) >> 8, so that F 0 gives FOGCOL itself

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.5: "R = F * Rv + ( 0xff – F )* Rfc ... However, A*B = ( A x B) >> 8"; FOG's page: "When the fog
  value is 0, the fog effect is at the maximum (the drawing color becomes the same as the distant color)"; ps2tek "GS
  Fog": the same two products, and "a value of 0xFF results in no change", which its own formula does not give.
- **Test program:** `s18-fog`'s first page: sixteen flat sprites at F 0, 1, 40h, 7Fh, 80h, FEh, FFh and 33h.
- **Referee:** PCSX2 2.8.2: all sixteen equal to (F × C + (100h − F) × FOGCOL) >> 8, none to §3.5's two products, nine to
  division by FFh.
- **Conclusion:** as stated; **measured**, one lineage with FOG's page; it is the alpha blend's form with FOGCOL as the
  destination, as ps2tek observes of fog.
- **Depends on it:** `gs::texture::fog`.
- **Pinned by:** `fog_blends_fogcol_by_the_interpolated_coefficient`.
- **Implemented in:** stage S18's commit.

### D-249. GS, bilinear filtering: the four texels weighted by α and β in sixteenths and the sum rounded down once

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.8's formula, R = (1 − α)(1 − β)Ra + α(1 − β)Rb + (1 − α)βRc + αβRd, α and β "the decimal parts"
  of s × USIZE − 0.5 and t × VSIZE − 0.5; §2.6.2's sixteenths. No rounding is given.
- **Test program:** `s18-filter`'s magnification pages (every sixteenth across, REPEAT and CLAMP, PSMCT16 and PSMT8),
  `s18-clamp`'s bilinear pages.
- **Referee:** PCSX2 2.8.2 departs by one step in 1,462, 435, 1,068 and 344 words of the filter's pages and 869, 167, 53
  and 870 of the clamp's; interpolating across, flooring, then down, flooring again, reproduces every word of the six
  pages decomposed (12,288 of 12,288), where the single rounding agrees with 586 to 1,995 of each page's 2,048.
- **Conclusion:** the formula with one rounding; **argued**, as D-160: PCSX2's two stages are its arithmetic; the
  documents' formula has none between its terms.
- **Depends on it:** `gs::texture::Sampler::level`.
- **Pinned by:** `bilinear_lod_and_the_mipmap_filters_work_in_sixteenths`, `every_wrap_mode_acts_on_whole_texels_at_each_level`.
- **Implemented in:** stage S18's commit.

### D-250. GS, blending between mip levels: every channel of levels m and m + 1 mixed by the LOD's fraction and rounded down once

- **Opened:** 2026-10-25, at stage S18.
- **Documents read:** §3.4.12's MMIN table ("Linear interpolation by LOD" for MMIN 3, trilinear for 5). No precision.
- **Test program:** `s18-filter`: MMIN 3 at K = 1.25 and 1.9375 and 7/16, MMIN 5 at 2.625 and 15/16, over a 64×64
  mipmap of seven levels of generated texels.
- **Referee:** PCSX2 2.8.2 blends red and green by the reading's weights; in pixel columns 2 and 3 of every four it
  leaves blue and alpha at level m, unblended. Blending by the fraction, its two-stage bilinear (D-249), and that column
  pattern reproduce every word of the five quarters tried (2,560 of 2,560).
- **Conclusion:** every channel blended; **argued**: no document makes a channel or a column's blend depend on the
  pixel's place, so the pattern is taken as a defect of the referee's renderer.
- **Depends on it:** `gs::texture::Sampler::sample`.
- **Pinned by:** `bilinear_lod_and_the_mipmap_filters_work_in_sixteenths`.
- **Implemented in:** stage S18's commit.

### D-280. GIF: a path holds the GIF from its packet's first GIFtag to its EOP primitive's end, though its data stops meanwhile; the other paths wait

- **Opened:** 2026-10-25, at stage S15: §7.1.2 checks the other paths "whenever transfer of a GS packet in a path ends",
  and says nothing of a path whose data stops mid-packet (a DMA that ends before the EOP primitive).
- **Documents read:** *EE User's Manual* (Version 5.0) §7.1.2, §7.1.3, §7.2.1 ("Transfer processing is performed in
  units of GS packets"), DIRECTHL's page; ps2tek "Graphics Interface (GIF)" ("Only one PATH may run at a time … when
  one PATH stops, the highest priority queued PATH will begin transfer").
- **Test program:** `s15-arbitration`: PATH3 sends a packet's first primitive (EOP clear) and its DMA ends; then a DIRECT,
  an XGKICK, or both; then the packet's rest. `s15-intermittent`'s continuous and DIRECTHL cases against a transfer in
  flight.
- **Referee:** PCSX2 2.8.2 lets DIRECT and XGKICK reach the GS before the rest of PATH3's packet, GIF_STAT reading idle
  between. Play! 0.72 holds DIRECT until PATH3's packet ends, in every case that shows it, and delivers XGKICK at once.
- **Conclusion:** as the title: the manual's unit of transfer is the packet, and Play!'s lineage holds PATH2 as it says;
  PCSX2's switch at a DMA's end is recorded, not followed. **Argued**, with one lineage agreeing for PATH2 and none for
  PATH1.
- **Depends on it:** `gif::Gif::run_to`, `path2`.
- **Pinned by:** `the_paths_are_arbitrated_at_packet_ends_and_path3_masked_and_interrupted_as_section_7_1_says`.
- **Implemented in:** stage S15's commit.

### D-281. GIF, PACKED: the descriptors 0Ch (XYZF3) and 0Dh (XYZ3) write the qword's lower 64 bits, unpacked, as TEX0 and CLAMP do

- **Opened:** 2026-10-25, at stage S15, by PCSX2's record of `s15-packed`.
- **Documents read:** *EE User's Manual* §7.2.3's table ("0x0c XYZF3 … Lower 64 bits", "0x0d XYZ3 … Lower 64 bits"),
  §7.3.2 (packing formats drawn only for 00h to 05h, 0Ah, 0Eh, 0Fh); ps2tek "GIF Data Formats" ("All registers not
  specified below output the lower 64-bits"; 04h and 05h alone given the XYZ packing).
- **Test program:** `s15-packed`: two sprites whose first vertex is a register value through 0Dh and 0Ch, the second a
  packed XYZ2.
- **Referee:** PCSX2 packs 0Ch and 0Dh as XYZF and XYZ (Y from bits 47..32, so the raw value's Z gives Y 0 and the sprite
  runs from the page's top); every other descriptor equal. Play! shows no drawing (`Sapphire_Native.md` §27.7).
- **Conclusion:** the lower 64 bits, as both documents give them. **Argued**, the documents against one lineage.
- **Depends on it:** `gif::packing::packed`.
- **Pinned by:** `packed_mode_writes_each_descriptor_as_section_7_3_draws_it`, `every_packing_conversion_writes_what_section_7_3_draws`.
- **Implemented in:** stage S15's commit.

### D-282. GIF_STAT: P1Q a kick waiting, P2Q PATH2 refused and waiting, P3Q PATH3's data or interrupted transfer waiting while it does not hold the GIF; OPH and APATH the path holding the GIF, or whose last output is still being sent; the registers §7.6 makes readable only under PSE answered at any time

- **Opened:** 2026-10-25, at stage S15: §7.6 names the fields without saying when a held path counts as outputting, and
  marks GIF_TAG0 to GIF_TAG3, GIF_CNT, P3CNT and P3TAG "accessible only when stopped temporarily by the PSE flag".
- **Documents read:** §7.6's pages; ps2tek "GIF I/O".
- **Test program:** `s15-arbitration` and `s15-regs`: GIF_STAT at every step; the tag registers and GIF_CNT under PSE
  with a packet held after its third data qword (LOOPCNT 2, REGCNT 2 by "decremented when starting processing of the
  NREGSth register descriptor"); P3CNT 24 and P3TAG 8020h after an interruption.
- **Referee:** PCSX2 never sets P1Q, P2Q or P3Q, shows OPH and APATH only while data moves (and OPH while BUSDIR is
  reversed and after it), and reads 0 from the tag and count registers under PSE. Play! reads E00h or 200h whatever the
  state.
- **Conclusion:** as the title, by the fields' names. **Argued**; neither lineage models them.
- **Depends on it:** `gif::Gif::status`, `read`.
- **Pinned by:** `the_gif_registers_pause_reset_and_report_as_section_7_6_says`, the arbitration program.
- **Implemented in:** stage S15's commit.

### D-283. DMAC channel 2: a DMAtag sent by TTE is not GIF data and the GIF drops it

- **Opened:** 2026-10-25, at stage S15: TTE sends the tag's quadword before its packet (D-36), and a VIF takes its upper
  64 bits as VIFcodes (D-193); the GIF's data is GIFtags and their data, which a DMAtag is not.
- **Documents read:** *EE User's Manual* §5.2.3, Dn_CHCR's TTE, §7.2.1 ("For PATH2 and PATH3, however, the VIFcode and
  DMATag are put in front of the GS packet"); ps2tek "DMAC Chain Mode".
- **Test program:** `s15-paths`: a packet by chain DMA with TTE, its tags' upper 64 bits zero; were the tag taken as a
  GIFtag, its QWC would be read as NLOOP and the packet lost.
- **Referee:** PCSX2 draws the region and Play! receives its LABEL: both drop the tag.
- **Conclusion:** dropped. **Measured** on both lineages.
- **Depends on it:** `gif::Gif::put`.
- **Pinned by:** `every_path_carries_the_same_packets_to_the_gs`.
- **Implemented in:** stage S15's commit.

### D-284. GIF_CTRL.RST clears the paths' packets, the FIFO, the waiting kicks, PATH3's interruption, the tag and count registers and GIF_MODE, and keeps MSKPATH3's mask

- **Opened:** 2026-10-25, at stage S15: RST returns "the internal register" to its initial value and GIF_STAT's initial
  value is 0, while §7.1.4 says the mask set by the VIF "can be cancelled only by the VIF".
- **Documents read:** GIF_CTRL's, GIF_MODE's and GIF_STAT's pages; §7.1.4.
- **Test program:** `s15-regs`: RST with a PATH3 packet held mid-way (PATH2 then goes at once), and with GIF_MODE 5 and
  MSKPATH3 set (GIF_STAT then M3P alone).
- **Referee:** PCSX2 lets PATH2 go and clears GIF_MODE's bits, and clears M3P too. Play! does not end the packet.
- **Conclusion:** as the title, §7.1.4's sentence deciding M3P. **Argued** against one lineage.
- **Depends on it:** `gif::Gif::reset`.
- **Pinned by:** `the_gif_registers_pause_reset_and_report_as_section_7_6_says`.
- **Implemented in:** stage S15's commit.

### D-285. The intermittent mode: the slice is 8 qwords of the IMAGE primitive's data counted from its start; a PATH1 or DIRECT request present when PATH3 reaches a slice's end, or arriving while it waits there, takes the GIF; DIRECTHL never does, and waits behind an interrupted PATH3 at the next arbitration

- **Opened:** 2026-10-25, at stage S15: §7.1.3 checks other requests "whenever 1 slice (8 qwords) is transferred",
  without saying where slices are counted from or what a request arriving at a waiting boundary meets.
- **Documents read:** §7.1.3, GIF_MODE's and GIF_STAT's IMT, P3CNT's and P3TAG's pages, DIRECTHL's page.
- **Test program:** `s15-arbitration`: an IMAGE packet held after 8 data qwords placed on an 8-qword boundary of memory, so
  that a slice counted from the primitive and one counted from memory agree, then DIRECT, DIRECTHL or XGKICK, in each
  mode. `s15-intermittent`: the same against a transfer of 30,720 qwords in flight.
- **Referee:** PCSX2 lets DIRECT overtake the running transfer in the intermittent mode (the LABEL while D2_CHCR.STR is
  set); its continuous and DIRECTHL cases finish the DMA first and decide nothing; it lets every path pass a held packet
  (D-280). Play! has no intermittent mode (DIRECT waits), and holds DIRECTHL.
- **Conclusion:** as the title. **Argued**, PCSX2 a witness for DIRECT's overtaking.
- **Depends on it:** `gif::Gif::at_slice`, `interrupt`, `choose`.
- **Pinned by:** the arbitration and intermittent programs.
- **Implemented in:** stage S15's commit.

### D-286. The GIF's time: a bus cycle a GIFtag, a PACKED qword and each 64-bit output of REGLIST and IMAGE; a bus cycle at each arbitration; a DMA unit's qwords arriving a bus cycle apart; XGKICK told an estimated end

- **Opened:** 2026-10-25, at stage S15: no document times the GIF beyond its idle cycles.
- **Documents read:** §7.1's figure (128-bit inputs at BUSCLK, a 64-bit output), §7.1.2, §7.3.1, §7.4.1, §7.5.1 (the idle
  cycles); *GS User's Manual* §1.1.2 (host transfers at most 1.2 GB/s).
- **Test program:** `the_fifo_holds_sixteen_qwords_and_the_gif_paces_the_dma`: 200 qwords take 400 EE cycles as PACKED,
  738 as IMAGE.
- **Referee:** not consulted for timing; PCSX2's GIF DMA of 30,720 qwords finishes within a few hundred EE instructions,
  far faster than any rate the documents allow.
- **Conclusion:** as the title, each a named loss (`Sapphire_Native.md` §33.4, §33.9). **Argued**.
- **Depends on it:** `gif::packing::Packet::take`'s cycles, `gif::Gif::put`, `gif::Path1Port`.
- **Pinned by:** the FIFO's test, `an_xgkick_is_told_its_packets_end_and_a_second_starts_behind_it`.
- **Implemented in:** stage S15's commit.

### D-287. While BUSDIR reverses the host interface the GIF sends nothing, its paths waiting; the GS's stalled SIGNAL does not stop the GIF

- **Opened:** 2026-10-25, at stage S15: §7.1.5 has the GIF set its direction from BUSDIR, and S16's GS drops a general
  register write while reversed and holds every write behind a stalled SIGNAL, unbounded.
- **Documents read:** §7.1.5; *GS User's Manual* §4.2.2 ("The general-purpose port register cannot be accessed while the
  FIFO is in the reverse direction"), §6.2.1, CSR's FIFO field.
- **Test program:** S16's programs through PATH3, whose readbacks reverse the bus after the FINISH handshake.
- **Referee:** PCSX2 sets OPH while BUSDIR is reversed and keeps it after (`s15-regs`).
- **Conclusion:** as the title; the GS's own FIFO depth is not documented, so the GIF does not wait on CSR.FIFO.
  **Argued**.
- **Depends on it:** `gif::Gif::run_to`, `path2`.
- **Pinned by:** `s16s_programs_run_whole_through_path3_and_vif1`.
- **Implemented in:** stage S15's commit.

### D-288. PATH1 reads its packet from VU1's memory as it sends it, the address wrapping at 1,024 qwords

- **Opened:** 2026-10-25, at stage S15: §7.1.1 has XGKICK start the transfer from VU1's memory, not copy it, and GIF_CNT's
  VUADDR is 10 bits.
- **Documents read:** §7.1.1, GIF_CNT's page; *VU User's Manual* XGKICK's page and §3.4.9.
- **Test program:** `s15-paths`: a packet of 10 qwords kicked at qword 1,020.
- **Referee:** PCSX2 draws it and Play! receives its LABEL: both wrap.
- **Conclusion:** as the title. **Measured** on both lineages for the wrap; the reading at the time of sending argued.
- **Depends on it:** `gif::Gif::run_to`.
- **Pinned by:** `every_path_carries_the_same_packets_to_the_gs`.
- **Implemented in:** stage S15's commit.

### D-289. PATH3's masks are read at the arbitration: a packet in progress when M3R or MSKPATH3 is set finishes, and the next waits, its data kept in the FIFO

- **Opened:** 2026-10-25, at stage S15: GIF_MODE's page has M3R "enabled after the end of transmission", MSKPATH3's page
  "applies to the next data block or later", and ps2tek has the mask apply "once GIF DMA ends its transfer".
- **Documents read:** §7.1.4, GIF_MODE's page, MSKPATH3's page; ps2tek "GIF I/O", "GIF PATH3 Masking".
- **Test program:** `s15-arbitration`: M3R set while PATH3's packet is held mid-way after its DMA ended, then the
  packet's rest, then another packet.
- **Referee:** PCSX2 and Play! both finish the packet and hold the next until M3R is cleared.
- **Conclusion:** as the title: ps2tek's DMA-end reading would have held the rest, which both lineages send. **Measured**
  on both lineages.
- **Depends on it:** `gif::Gif::choose`.
- **Pinned by:** the arbitration program.
- **Implemented in:** stage S15's commit.

### D-290. PCSX2's second DIRECT (S14): a referee defect, not a protocol rule

- **Opened:** 2026-10-25, at stage S14 (`Sapphire_Native.md` §28.8): PCSX2 takes `s14-direct`'s first DIRECT and never
  its second.
- **Documents read:** §7.1.2, §7.1.4; DIRECT's page.
- **Test program:** `s15-direct-a` to `-d` (`Sapphire_Native.md` §33.8): two DIRECTs with no GIF reset, with one, with
  the second stream DIRECT alone, and with a PATH3 packet between.
- **Referee:** PCSX2 hangs in (a) and (c): GIF_STAT 0 at the start, E00h (APATH PATH3) after the first DIRECT with no
  PATH3 data and channel 2 stopped, then E80h; it takes both in (b) and (d). Play! takes both in every form.
- **Conclusion:** PCSX2, entered by fast boot, holds a PATH3 request that waits through an idle GIF and is granted after
  a PATH2 packet with nothing to send; under §7.1.2 an idle GIF grants a request at once and PATH3 has no data, so the
  hang has no reading in the documents. The programs reset the GIF first. **Measured**.
- **Depends on it:** nothing in the core; every S15 program's opening.
- **Pinned by:** `a_second_direct_reaches_the_gs_in_every_form_of_the_probe`.
- **Implemented in:** stage S15's commit.

### D-291. Q is the GIF's own register: ST's packed form sets it and writes only S and T, RGBAQ's packed form gives it, and every GIFtag sets it to 1.0

- **Opened:** 2026-10-25, at stage S15: §7.3.2 keeps Q "in the internal register of the GIF" and outputs it with the
  next RGBAQ, initialised "whenever the GIFtag is read"; ps2tek's "STQ" writes "ST register and Q component of RGBAQ",
  its RGBA leaves Q unchanged, and "The GS Q register is initialized to 1.0f when reading a GIFtag".
- **Documents read:** §7.3.2's ST and RGBAQ; ps2tek "GIFtags", "GIF Data Formats".
- **Test program:** none can see Q before texturing (S18); `every_packing_conversion_writes_what_section_7_3_draws`
  checks the writes the manual's reading gives.
- **Referee:** not consulted.
- **Conclusion:** the manual's reading. **Open** until S18's programs draw with Q.
- **Depends on it:** `gif::packing::Packet::take`.
- **Pinned by:** `every_packing_conversion_writes_what_section_7_3_draws`.
- **Implemented in:** stage S15's commit.
