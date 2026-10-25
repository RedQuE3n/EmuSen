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
