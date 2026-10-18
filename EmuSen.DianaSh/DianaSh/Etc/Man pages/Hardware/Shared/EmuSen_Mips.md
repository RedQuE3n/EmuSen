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
that an R3000A's decode stays exactly MIPS I [C]. None of it is built.

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

