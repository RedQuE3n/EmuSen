# EmuSen_Mips — the MIPS instruction set as data

*Started 2026-10-11, at Diamond's stage 2.* `emusen-mips`, `EmuSen/Cores/Shared/emusen-mips/`: the fields of a MIPS
instruction word, the decode of MIPS I as the R3000 decodes it, the disassembler with a table per coprocessor, and an
assembler for test programs. Planned in `Diamond_Plan.md` §9.1 and decided with its §11 Q10 on 2026-10-11. It has no
dependency, and **no execution**: each processor crate keeps its own semantics, because the R3000A's and the VR4300's
differ in width, delay slots, coprocessor 0, caches and exceptions (the plan's §9.1 table), and an interpreter shared
"with flags for the differences" would be two interpreters in one file.

The marks are `Diamond_Plan.md`'s: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from a source;
**[A]** argued; **[C]** a choice.

## 1. Who uses it, and who owns what

- **`blackmoon-r3000`** decodes every instruction it executes with `decode`, and gives `emusen-mips` its coprocessor
  0's table and its dialect (`BlackMoon_R3000.md` §2). **`blackmoon-gte`** gives the GTE's table, its register names
  from psx-spx and its 22 commands. Diamond joins the two when it claims `DEBUG_DISASSEMBLE` (its stage 12).
- **One owner per fact.** The opcode map, the mnemonics, the operand order and the general registers' conventional
  names are this crate's. A coprocessor's register and command names belong to the crate that owns the coprocessor,
  so this crate defines only the table's type: the GTE's names were already `blackmoon-gte`'s, and coprocessor 0's
  names differ between the R3000A (BPC, TAR, DCIC…) and an R4000-style processor (Index, EntryHi, Count…).
- **MarsRT** may adopt the disassembler when it gains `DEBUG_DISASSEMBLE`, a change for MarsRT's own owner to decide
  and measure (the plan's §9.1). Nothing in MarsRT was read or changed for this crate.
- **Sapphire** (the PlayStation 2) takes the R5900's table (§8) and the vector units' (§9) from its stage S5 on: for its
  test programs now, and later for its interpreter's decode, its debugger and the open BIOS's scan of its EE image
  (`Sapphire_Plan.md` §5.9, §9.1).

## 2. The decode

`src/decode.rs`. `decode(word) -> Op`, with `Word` giving the fields (op, rs, rt, rd, sa, funct, the immediate
zero- or sign-extended, the jump target, SYSCALL's code, the coprocessor's number and COPz's 25-bit command).

### 2.1 The map

The opcode map of IDT's *R30xx Family Software Reference Manual*, Appendix A, which agrees with psx-spx's "CPU Opcode
Encoding" tables [R]: 58 operations, and nine coprocessor forms (MFCz, CFCz, MTCz, CTCz, BCzF, BCzT, COPz, LWCz, SWCz)
for each of the four coprocessors. Every encoding psx-spx marks "N/A", in the primary and the SPECIAL table, is
`Op::Reserved`, which the R3000A turns into the reserved instruction exception. `Op::canonical` is each operation's word
with its operands zero, the assembler's starting point; `Op::operands` lists its operands in Appendix A's order.

### 2.2 The R3000's partial decode

The R3000 decodes fewer bits than the architecture assigns, and the decode follows it:

- **Unused fields are ignored**: SPECIAL's sa or rs where an instruction does not use them, LUI's rs, BLEZ's and
  BGTZ's rt, SYSCALL's and BREAK's code. psx-spx: the unused bits "should be usually zero, but do not necessarily
  trigger exceptions if set to nonzero values" [R]; the r3000 suite's random fields agree in every case [M].
- **REGIMM (BcondZ)**: bit 16 selects BGEZ over BLTZ, and the branch links only when rt's bits 20–17 are `1000`;
  nocash's single-file psx-spx lists the other rt values as "undocumented dupes" of BLTZ and BGEZ [R]. The suite's
  BCondZ file uses all 32 rt values and agrees in every case [M]. A MIPS II or III decode, where rt 2, 3, 8–14 and
  18–19 are other instructions, is a different decode (§2.4).
- **The coprocessor formats**: bit 25 makes COPz whatever bits 24–21 hold; otherwise rs 0, 2, 4 and 6 are MFCz,
  CFCz, MTCz and CTCz, rs 8 is BCzF or BCzT by rt's bit 0 alone [A: the R3000 has only the two coprocessor branches,
  and nothing says it decodes rt's other bits], and the other rs values are reserved. ps1-tests' `cpu/cop` runs `COP0`
  and `COP2` with bits 24–21 all set on a console without an exception, consistent with bit 25 deciding [R].

### 2.3 The extension point per coprocessor

The decode names a coprocessor instruction by its form and number (`Op::Cop(2)`, `Op::Mfc(0)`); what a command means
is its coprocessor's. The extension is data, `Coprocessor`: the names of its 32 data and 32 control registers (empty
for an unnamed one, written `$n`), and its commands, each a name, the bits of the 25-bit command that select it, and
its canonical 25 bits. `Dialect` gives a processor's four coprocessor slots. A coprocessor is added by writing its
table in its own crate; the decode and the disassembler do not change.

### 2.4 What later processors add

MIPS III (the VR4300) and the RSP extend the map: 64-bit operations, the likely branches, traps, LL/SC, 64-bit loads
and stores; the RSP's vector unit on coprocessor 2 with its own load and store forms. They are to be a second decode
function beside `decode`, the same `Op` gaining their operations, and the RSP's vector unit a `Coprocessor` table, so
that an R3000A's decode stays exactly MIPS I [C]. *Built since, each as a table of its own rather than an extension of
`Op`*: the VR4300's (§6), the RSP's (§7), the R5900's (§8) and the PlayStation 2's vector units' (§9).

## 3. The disassembler

`src/disasm.rs`. `disassemble(word, pc, dialect)` returns a value that is `Display`, so nothing is allocated unless
the caller allocates. The text:

- **Appendix A's mnemonics, lower case, and its operand order** ("ADD rd, rs, rt", "SLL rd, rt, sa", "LW rt,
  offset(base)", "MFCz rt, rd"…), the coprocessor's number in place of z. A test transcribes every Appendix A format
  line, gives each field a value that tells it apart, and checks the text (§5).
- **General registers** by their conventional names (IDT Table 2.1; `fp` for 30, psx-spx's name, the assembler also
  taking `s8`). **Coprocessor registers** by the dialect's table, `$n` without one.
- **Numbers**: a sign-extended immediate and a load's offset as signed hex (`-0x10`), a zero-extended immediate as
  hex, a shift amount in decimal, a branch's or jump's target as the absolute address it reaches from `pc` (counted
  from `pc + 4`, as the manual defines it), SYSCALL's and BREAK's code only when it is not zero.
- **Synthetic forms**: only `nop` for the zero word, and `jalr rs` when rd is 31, both Appendix A's own. Nothing else
  synthetic is written, so the text is always one machine instruction.
- **A coprocessor command** by the table's name: the name alone when the 25 bits are its canonical ones, the name and
  the 25 bits otherwise (`nclip 0x0140006`, the form psx-spx reports a game using); `copz 0x…` when no command
  matches. A reserved word is `.word 0x…`.

## 4. The assembler

`src/asm.rs`, a development tool for test programs and, later, the open BIOS's entry (`Diamond_Plan.md` §5.9).
`assemble_line(text, pc, dialect)` reads one instruction as the disassembler writes it; `assemble(source, origin,
dialect)` reads a program, one instruction or `.word` to a line, with `name:` labels for branch and jump targets and
`#` or `;` comments. It checks every operand's range, a branch's reach and alignment, and a jump's 256 MiB region, and
names the line of an error. It writes only what Appendix A's formats name, `nop` and `.word`; pseudo-instructions
(`li`, `la`, `move`, `b`) are not provided [C: a test program then says exactly which instructions it runs].

## 5. How it is tested

On its own [M, `cargo test --release`, 2026-10-11, 10 tests]: every operation, each coprocessor's four times, decodes
from its canonical word with 64 sets of random operands; the primary and SPECIAL maps reserve exactly psx-spx's N/A
entries; REGIMM's 32 rt values and the coprocessor formats decode as §2.2 says; every Appendix A format is written in
its order; and every operation's disassembly, with 256 sets of random operands and PCs, assembles back to the same
word. In `blackmoon-r3000` [M]: **every one of the r3000 suite's 55,000 opcodes decodes to its file's instruction**, and
its disassembly assembles to a word that agrees with it on every field the instruction uses and is canonical elsewhere,
and disassembles to the same text. In `blackmoon-gte` [M]: each of the 22 commands is written by its name and reads
back.

## 6. MIPS III in the VR4300's spelling (2026-10-17)

*Built with the backend's step B3, part (d) (`EmuSen_Backend.md` §13.5), as the tester decided on 2026-10-17.*
`emusen-mips` owns the instruction set for both of the project's MIPS processors (`EmuSen_Backend.md` F8). The VR4300's
disassembly, which was the MarsRT shim's C# `Vr4300Disassembler` (`Mars_Disassembler.md`), is `src/vr4300.rs`: MIPS III
with the VR4300's coprocessor 0 and FPU, written in that part's own spelling. The two spellings differ in every
respect a reader notices, register 30 (`s8` against `fp`), folded forms (`beqz`, `b`, `move`, `neg`, `not`), hexadecimal
case and `.word` for what the part does not decode, and each console's debugger output and golden corpus hold its own.
So the spelling is a field of `Dialect` (`Spelling::AppendixA`, the default, and `Spelling::Vr4300`), and
`Dialect::VR4300` writes `vr4300::decode`'s text; neither golden changes [C].

The R3000's decode (`decode`), which `blackmoon-r3000` executes by, is untouched: MIPS III's instructions are reserved
instructions on that part, so the VR4300's table is a decode of its own rather than an extension of `Op` [A]. Its
`decode` also gives the address a word names where the word alone gives it (a branch, a jump, a load or store from the
zero register), for the debugger's references. The C# remains, moved to `Crystal/Systems/N64/`, as the failover the
parity class holds this to.


## 7. The N64's signal processor (2026-10-18)

*Built with step B4 for MarsRT (`EmuSen_Backend.md` §18.7, step 4), the item §13.6 there left for it.* The RSP's
disassembly, which was the MarsRT shim's C# `RspDisassembler` (`Mars_Disassembler.md` §5), is `src/rsp.rs`: the scalar
unit's MIPS subset, `mfc0` and `mtc0` over the sixteen registers the RSP reaches (the SP's and the DP's, by their
libultra names), and coprocessor 2, the vector unit, with the SGI names for its operations, element selectors and
loads and stores. It writes the VR4300's spelling and returns `vr4300::Instruction`, so the two N64 processors' text
and references have one shape; every address it names is an offset into IMEM (a branch or jump, wrapped at 4 KiB) or
DMEM (a load or store from register zero), never a CPU address [M: the C#'s `ImemMask` and `DmemMask`].

It is a table of its own beside `vr4300.rs` rather than a coprocessor table on `Dialect`: the RSP's primary opcodes
are a subset of MIPS I's with different meanings in the LWC2 and SWC2 slots, and its branches compute IMEM offsets
where the VR4300's compute virtual addresses, so sharing the decode would make one branch rule two [A].

The N64 pack reaches it as `n64::disasm::decode_rsp`, and the platform library's `emusen_systems_disassemble` as the
processor `rsp` (interface 19). The C# moved to `Crystal/Systems/N64/RspDisassembler.cs`, its `Decode` forwarding to the
library and its `DecodeManaged` the failover, held by `DebugCodeParityTests.Every_rsp_word_disassembles_alike` over
every primary opcode crossed with every function, element and register pattern the decode reads, and 200,000 random
words, at IMEM addresses that wrap.

## 8. The R5900 (2026-10-24)

*Built at Sapphire's stage S5 (`Sapphire_Plan.md` §7 and §9.1), as the tester decided with that plan's §11 Q10 on
2026-10-24.* `src/r5900.rs`: the PlayStation 2's EE core, decoded and written as a table of its own. The marks are the
plan's; the sources are the documents pinned at its stage S2 (`Sapphire_References.md` §1): Toshiba's *TX79 Core
Architecture*, rev. 2.0 (2001-04), Appendices A to D, and ps2tek at `1c91660`, "EE Instruction Decoding", "EE COP0
Registers" and "EE COP1". No emulator's, assembler's or SDK's source and no Sony manual was read.

### 8.1 Why a table of its own

The R5900's map differs from MIPS III in both directions [R the TX79 manual, A.5 and §3.3]: it adds the MMI opcode
space, the second multiplier's instructions (MULT1, DIV1, MADD1…), the three-operand MULT and MADD, MFSA and MTSA, LQ
and SQ, and lacks DMULT, DMULTU, DDIV, DDIVU, LL, LLD, SC and SCD, whose cells the manual marks η ("currently not
supported … Reserved Instruction exception"). Folded into the VR4300's decode, one opcode would carry two rules, the
argument `rsp.rs` was made by (§7) [A]. Nothing in `decode.rs`, `vr4300.rs` or `rsp.rs` changed: their text over
3,020,480 words (every primary opcode with every function, five register patterns each, and three million random words
at varied addresses) hashes the same before and after [M, a throwaway program over `disassemble`, `vr4300::decode` and
`rsp::decode`, 2026-10-24].

### 8.2 The map

The manual's opcode tables are A.5 (the OpCode field, SPECIAL's function, REGIMM's rt), B.5 (MMI's function and the
MMI0 to MMI3 tables by bits 10–6), C.2 (COP0's rs, MF0 and MT0 with rd = Debug or Perf, BC0's rt, C0's function) and
D.4 (COP1's rs, BC1's rt, the function with rs = S or D, and with rs = W or L). `r5900.rs` holds one Rust table for each,
indexed by the same field, so the source reads cell for cell against the manual: 290 mnemonics.

**Where the EE differs from the TX79, ps2tek's tables are followed** and each difference is a dispute
(`Sapphire_Plan.md` P6, `Sapphire_Disputes.md`):

- **D-1, COP1.** The TX79's FPU is MIPS III's, with the D and L formats, DMFC1, DMTC1, LDC1, SDC1, the rounding
  conversions and sixteen compare conditions [R D.4, §10.5]. ps2tek's EE has single precision only ("COP1, along with
  the rest of the PS2, only supports single-precision floats", "EE COP1 Floating Point Format") and its own operations:
  RSQRT.S, ADDA.S, SUBA.S, MULA.S, MADD.S, MSUB.S, MADDA.S, MSUBA.S, MAX.S, MIN.S, four compares, CVT.W.S and CVT.S.W
  [R "FPU.S", "FPU.W"]. Its compares at 34h and 36h are named C.LT and C.LE where the TX79 names those cells C.OLT and
  C.OLE; ps2tek's names are written.
- **D-2, BC1FL and BC1TL.** ps2tek's BC1 table has the likely forms; the TX79's has BC1F and BC1T alone.
- **D-3, COP2, LQC2 and SQC2.** The TX79 reserves the OpCode cells 12h, 36h and 3Eh; ps2tek gives them to COP2, LQC2
  and SQC2, and Sony's Hot Chips slides describe the "co-processor mode: MIPS COP2 instruction" [R]. COP2's operations
  are VU0's, written by §9's table with a `v`.

What ps2tek leaves out and the TX79 gives is taken from the TX79 [C]: the C790's own COP0 moves (MFBPC to MTDVBM through
rd = Debug, MFPS, MFPC, MTPS and MTPC through rd = Perf, by C.2's two small tables), SYNC.P (stype's top bit, A's
SYNC.stype), PMFHL's five formats and PMTHL's one (B's PMFHL.fmt and PMTHL.fmt; the other fmt values are reserved
here, the manual naming none), and every instruction's operand order (each page's Format line). The EE's COP0 register
names are the TX79's Table 4-1, which ps2tek's list repeats ("PRId" as the TX79 writes it).

**Fields the format fixes at zero are not decoded** [C]: SPECIAL's sa where an instruction has none, BREAK's code
read as a code, a COP0 move's low bits outside Debug and Perf, and the like. The VR4300's decode does the same, and no
document says whether the C790 raises a reserved instruction exception for them; S6's interpreter is where it would
matter, and ps2autotests' `cpu/ee/*` there.

### 8.3 What it returns

`operation(word) -> Option<Op>`: `Op::Cpu(name, form)` or `Op::Vu0(&vu::Operation)`, `None` where the EE reserves the
encoding; an interpreter and the open BIOS's scan ("refuses any instruction the R5900 lacks", `Sapphire_Plan.md` §5.9)
both read it. `decode(word, pc)` returns the VR4300's `Instruction` shape (mnemonic, operands, and the address the word
names: a branch's or jump's target, or a load's or store's address when its base is the zero register), so that the
N64's processors and the EE give a debugger one shape. `disassemble(word, pc, &Dialect::R5900)` writes the same text.

### 8.4 The spelling

**Appendix A's**, the R3000's (§3) [C]: the IOP beside the EE is written so, and one console's two processors read
alike. Lower-case mnemonics, the general registers by their conventional names with `fp` for 30 (ps2tek's "EE
Registers" table gives the same names), signed hexadecimal immediates and offsets, the shift amount in decimal, a
branch's or jump's target as the address it reaches, `nop` for the zero word and `.word` for a reserved one. What is
the R5900's own:

- The three-operand multiplies (MULT, MULTU, MADD, MADDU and their pipeline-1 forms) are always written with rd, the
  C790's Format line; the assembler also takes the two-operand MIPS form, rd zero.
- `sync` and `sync.p`; `pmfhl.lw`, `.uw`, `.slw`, `.lh`, `.sh`; `pmthl.lw`; `mfps rt, reg` with the counter's number.
- COP0's registers by name, a reserved one `$n`; the FPU's `fN` and `fcrN`; the vector registers in COP2's moves and
  loads as `vfN`; VU0's control registers in CFC2 and CTC2 as `$n`, since no source read here numbers them.
- CACHE's operation and PREF's hint as numbers, as the VR4300's spelling writes CACHE's; Table C-1's names are not used.

**Left out**: ps2tek names "QMTC2.I/CTC2.I" (the interlocking forms, "VU Instruction Format") without giving the bit
that selects them, so QMFC2, QMTC2, CFC2 and CTC2 are decoded without it, and a word with any of their low eleven bits
set reads as the plain move.

### 8.5 The assembler

`assemble` and `assemble_line` with `Dialect::R5900` read what `decode` writes, with `asm.rs`'s labels, comments and
`.word`; `r5900::encode` builds each instruction from its table's canonical word and its form's fields, and refuses
any text whose word decodes to another mnemonic (`mfc0 t0, Debug` is `mfbpc`'s encoding, so it is refused). A VU0
operation is read by §9's `vu::encode` behind the COP2 bits.

### 8.6 How it is tested

[M, `cargo test --release`, 2026-10-24.] **The gate, `every_encoding_of_the_tx79_tables_decodes_disassembles_and_reassembles`**:
every cell of the manual's tables transcribed into the test (806 cells: 261 named, 462 reserved, 83 where the EE's
table differs), not read from `r5900.rs`. A named cell decodes to the manual's mnemonic and, with its operand fields set
at random 64 times at random addresses, disassembles and reassembles to the same word; a reserved cell (`*`, `η`, `ϕ`)
decodes to nothing; and each of the 83 cells of the differences above decodes as ps2tek's table reads, its dispute
named in the test. `the_ee_s_cells_by_ps2tek`: ps2tek's own primary rows 6 and 7, COP1, BC1, BC2, FPU.S, FPU.W and COP2
tables cell for cell, and every VU0 operation under each of COP2's sixteen rs values reading back. `the_spelling_is_appendix_a_s`:
words built by hand from the manuals' encoding pictures, one or two of each kind of operand, and two refusals.

## 9. The vector units (2026-10-24)

*Built at the same stage.* `src/vu.rs`: the operations VU0 and VU1 share, read in micro mode as a 64-bit pair and in
macro mode as the EE's COP2 word.

### 9.1 One table, two modes

Sony's Hot Chips 11 slides (Kunimatsu et al., 1999, "Instruction opcode to support 2 modes") draw the three layouts
side by side [R]: the **upper word** of a VLIW instruction is I, E, M, D, T (bits 31–27), two zero bits, then dest (4
bits), ft, fs, fd and a 6-bit opcode, MADDbc as the example (`0010--`); the **lower word** is `1000000` ("Lower OP.",
bits 31–25), then dest, ft, is and an 11-bit opcode, LQI as the example (`01101 1111 00`); and the **COP2 word** is
`010010`, a 1 (co), then the same 25 bits, VMADDbc and VLQI as its examples, "A VLIW instruction includes two COP2
instructions". ps2tek gives the 64-bit pair's order ("the lower 32-bit word is the lower instruction and the upper word
is the upper instruction"), the five flags' meanings, and the two macro tables, "COP2 Special1" by bits 5–0 and
"COP2 Special2" by bits 10–6 and 1–0 where bits 5–2 are 1111 [R]. So `vu.rs` holds those two tables once, each
operation with its micro name (the slides' "MADD bc"), and macro mode writes it with ps2tek's `v` ("VMADDx"). The
slides' two examples sit in ps2tek's tables where their bits say (MADDx at 08h; LQI at row 0110, column 4), so the
layouts and the tables agree [A].

ps2tek's tables carry two printing slips, read as the obvious names: "VSUbi" (VSUBi) and "CALLMSR" without its V [A].

### 9.2 Which half runs an operation

The slides list the upper unit's operations (add and subtract, multiply, multiply-add and -subtract, maximum and
minimum, the outer product, clipping) and the lower unit's (divide, square root and reciprocal square root, loads and
stores, jumps and branches, the EFU, the random unit, and in the block diagram the integer ALU) [R]. Every operation so
placed is in Special1's rows up to 2Fh or Special2's rows 0000–0101 if upper, and in Special1's row 110 or Special2's
rows 0110 onwards if lower, so the table puts each operation by its row [A], and Special1's VCALLMS and VCALLMSR, which
call a microprogram, in macro mode alone. **The rule places nine operations the documents do not**: ITOF and FTOI (four
each), ABS and NOP upper, and MOVE, MR32, MTIR, MFIR and WAITQ lower. `Sapphire_Disputes.md` D-4.

**dest's bit order** is not drawn: bit 24 is taken as x and bit 21 as w, as ps2tek's MAC flag register puts x highest
(`Ox|Oy|Oz|Ow…`) [A]. `Sapphire_Disputes.md` D-5.

### 9.3 What the documents do not give

- **The micro-only lower instructions**: those with an opcode of their own in bits 31–25 rather than the `1000000`
  class, which ps2tek's pipeline section and VIF chapter name (WAITP, XITOP, the EFU's ESADD to EATAN) and which the
  slides' lower list implies (loads and stores of the LQ and SQ kind, branches, integer immediates): no source read
  here gives their encodings, so such a lower word is written `.word`. S13 (VU micro mode) needs them.
- **Operand layouts** beyond the slides' two: an operation whose fields are the slides' upper layout with an fd, fs and
  ft (the FMAC family, and its accumulator and q and i forms) is written with them; every other operation is written
  `Raw`, its name alone when its 25 bits are its canonical ones, otherwise its name and the 25 bits in hexadecimal, as
  §3 writes a coprocessor command (`div 0x0102bbc`). This makes no claim about which field an undocumented operation
  reads, and reads back exactly.
- **VU1's own operations** (the EFU, XGKICK): undocumented encodings, as above; the one table serves both units.

### 9.4 The spelling and the assembler

A pair is written `upper | lower`: the upper's set flags first in brackets, in the order I, E, M, D, T (`[e] nop |
move`); with the I bit, the lower is the constant in hexadecimal (`[i] addi vf0, vf0, i | 0x3f800000`); an operation
`name.dest vfD, vfS, vfT` (`maddx.xyzw vf1, vf2, vf3`), the accumulator `acc`, the q and i registers `q` and `i`; an
upper word that is no upper operation, or has bits 26–25 set, is `.word` whole, flags included; a lower word that is not
in the `1000000` class or not a lower operation is `.word`. `vu::assemble_pair` and `vu::assemble` read that text back
(a program one pair to a line, `#` and `;` comments, no labels while no branch is decodable); `vu::encode` reads one
operation, which `r5900::encode` uses for COP2.

### 9.5 How it is tested

[M, 2026-10-24.] `every_encoding_of_the_documented_tables_reads_back_in_both_modes`: ps2tek's two tables transcribed
cell for cell (60 cells of Special1 below its four 1111 cells, all 128 of Special2): each of the 120 named cells decodes
to its name and reads back 64 times with random fields and flags, in macro mode and in micro mode in its own half; each
empty cell decodes to nothing. `the_hot_chips_examples_and_the_flags`: the slides' VMADDx and LQI words, the flags, the
I bit's constant, and the refusals (an integer add in the upper half, a field mask out of order).
