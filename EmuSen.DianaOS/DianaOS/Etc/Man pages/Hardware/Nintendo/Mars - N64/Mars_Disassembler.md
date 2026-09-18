# Mars — the two disassemblers

*Not a slice: an instrument, landed 2026-09-18. Two pure decoders — one word and its address in, text and a static
reference out — for the VR4300 (`Cpu/Disassembler/Vr4300Disassembler.cs`, which also holds the shared
`MarsInstruction`) and for the RSP (`Rsp/RspDisassembler.cs`). Tests in `MarsDisassemblerTests`. Neither is wired into
`MarsDebugTarget` yet; that is a separate change (§9).*

---

## 0. What grades this

A disassembler has no hardware behaviour to measure. What it can be wrong about is narrower and more checkable: which
bits it reads for each field, which encodings it names, what it calls them, and the arithmetic that turns an offset
into an address. The page keeps four kinds of evidence apart, because they are evidence for different things:

- **For the VR4300, capstone** (the Python binding, 5.0.7), an independent MIPS decoder built from LLVM's tables, run in
  its MIPS64 big-endian mode against a generated set of about two hundred thousand encodings (§6). It is evidence for
  field extraction, operand order, the mnemonic each MIPS III encoding carries, and branch and jump arithmetic. **It is
  not evidence for which encodings the VR4300 refuses**: capstone decodes a superset — MIPS IV and V, MIPS32/64
  releases 1, 2 and 6, MSA, the DSP extension and even Cavium's Octeon instructions — so "capstone names it" never means "the VR4300
  executes it". Nor is it independent of the MIPS manuals: a misreading the manuals invite would be shared, and would
  show up as agreement.
- **For the RSP there is no comparable oracle.** Capstone has no RSP vector ISA. What exists instead, in order of
  weight (§7): the hardware corpus's own RSP assembler, which produced the encoding of every RSP test the corpus runs on
  a console; Mars's RSP interpreter, which those tests grade (243 of 248 groups, `Mars_Rsp.md` §8); cxd4's decode;
  Project64's RSP disassembler; and parallel-rsp's debug disassembler. None of them was run: each was read, field by
  field, against the decoder.
- **Mars's own interpreter** decides which encodings get a mnemonic at all (§3). That makes the disassembler a
  description of what Mars executes, which is what a debugger listing is for, and it means the disassembler inherits
  every decode decision the interpreter's pages record rather than taking its own.
- **The unit tests** pin the text: literal words to literal strings, never strings built from the decoder's tables
  (§8). The breakage round shows that each rule on this page has a named test that fails without it.

## 1. The contract

```csharp
public readonly record struct MarsInstruction(string Mnemonic, string Operands, StaticReferenceKind? Kind, uint Target);
public static class Vr4300Disassembler { public static MarsInstruction Decode(uint word, uint address); }
public static class RspDisassembler    { public static MarsInstruction Decode(uint word, uint address); }
```

- **`address` is the instruction's own address**: the 32-bit virtual address for the VR4300, and the IMEM offset for
  the RSP. The RSP decoder masks what it is given to twelve bits, so a bus address such as `0x04001FF8` decodes as
  offset `0xFF8`.
- **Nothing throws.** An encoding with no mnemonic (§3) comes back as `.word` with the whole word in eight hex digits,
  `Kind` null and `Target` zero. The tests sweep 786,432 words through both decoders to hold that.
- **`Kind` and `Target`** are §4.

## 2. Notation

One spelling for each kind of operand, chosen once and used everywhere:

| operand | spelling | example |
| --- | --- | --- |
| mnemonic | lowercase | `daddiu` |
| general register | o32 name, no `$` — the names `MarsDebugTarget` already shows | `sp`, `s8`, `ra` |
| register 30 | `s8`, never `fp` | `sllv s8, t4, gp` |
| float register | `f0`–`f31` | `add.s f3, f1, f2` |
| float control register | `fcr0`–`fcr31` | `ctc1 at, fcr31` |
| VR4300 COP0 register | the VR4300 manual's name; an unused index as `$n` | `mtc0 zero, Status`, `mfc0 at, $7` |
| VR4300 COP2 register | `$n`, since the part has one latch and no names (`Mars_Fpu.md` §8) | `mfc2 t9, $22` |
| immediate | hex, `0x` and upper-case digits; signed where the instruction sign-extends it | `addiu sp, sp, -0x18`, `andi a1, a0, 0x8010` |
| memory operand | `offset(base)`, with the offset always written, even `0x0` | `lw a1, 0x0(a0)` |
| branch or jump target | the absolute address: eight digits on the VR4300, three on the RSP | `beq a0, a1, 0x80000414`, `j 0x0FC` |
| separator | comma and one space | |

The rules the table does not show:

- **Sign follows the instruction, not the field.** `addi`, `addiu`, `slti`, `sltiu`, `daddi`, `daddiu`, every load and
  store offset, and the six trap-immediate instructions print a signed value; `andi`, `ori`, `xori` and `lui` print the
  sixteen bits as they are. `sltiu` is signed on purpose: its immediate is sign-extended and then compared unsigned, and
  the printed value is the one the comparison uses. The trap immediates are the same case (`Mars_Cpu.md` §15.1).
- **A shift amount is the field's value.** `dsll32 v0, a1, 0x1F` shifts by 63, and prints the 31 the word holds, as
  every MIPS assembler does.
- **A software code field is printed only when it is non-zero**: the twenty-bit code of `syscall` and `break`, and the
  ten-bit code of the six register traps. `break`'s code is **one twenty-bit number**, as the VR4300 manual defines the
  field, not the ten-and-ten split GNU and capstone print (`break 0xf, 0x225` there is `break 0x3E25` here).

### 2.1 Pseudo-instructions

**The alias set is LLVM's, which is capstone's, less three: `ssnop`, `ehb`, and `jr` for `jalr zero`.** Every alias
kept drops an operand that is the zero register and names what is left:

| alias | encoding |
| --- | --- |
| `nop` | the word `0x00000000` only |
| `move rd, rs` | `addu`, `or` or `daddu` with `rt` = `zero` |
| `neg`, `negu` | `sub`, `subu` with `rs` = `zero` |
| `not rd, rs` | `nor` with `rt` = `zero` |
| `b` | `beq zero, zero` |
| `beqz`, `bnez` | `beq`, `bne` with `rt` = `zero` (so `bne zero, zero` is `bnez zero`, never taken) |
| `bal` | `bgezal zero` |

Not used: **`ssnop` and `ehb`**, which are MIPS32's names for `sll zero, zero, 1` and `3` — on the VR4300 those are
ordinary shifts into register zero, and naming them after a pipeline hazard the part does not have would mislead;
**`jr` for `jalr zero, rs`**, because it renames the word to a different encoding; GNU's **`li`**, which capstone does
not use either; and **no alias for the branch-likely forms**, which capstone does not alias. `dsubu rd, zero, rt` stays
as it is for the same reason: capstone has no `dnegu`.

The set was chosen to match the oracle, so that what remains of a disagreement is substantive. The one thing an alias
loses is which of `move`'s three encodings was used; the debugger prints the word beside the text, which settles it.

## 3. What decides a mnemonic

**A word gets a mnemonic when the architecture names its encoding and Mars's interpreter executes it as that
instruction. Everything else is `.word`.** The fields that select an instruction — the opcode, the function, the `rs`
or `rt` sub-opcode, the float format — are read exactly as the interpreter reads them; **fields the interpreter does
not read are not checked either.** So `sll t3, s7, 0x13` is what the word `0x03575CC0` says, although its `rs` field
is non-zero and capstone refuses it: the interpreter executes it as that shift. The places this leniency matters, all
of them the interpreter's own:

- the shifts ignore `rs`; the three-register forms ignore the shift field; `jr`, `mfhi` and the multiplies ignore the
  register fields they do not name; `sync` ignores its whole operand space;
- the COP0 moves ignore bits 10:0, where MIPS32 put a select field;
- **the COP0 `CO` half ignores bits 24:6 entirely**, so `0x43FFFFD8` is `eret` (`Mars_Cop0.md` §10.1);
- the float compares ignore bits 10:6, where MIPS IV put a condition-code number;
- the unary float operations ignore `ft`.

Refused encodings print as `.word` whatever the refusal is: Reserved Instruction for an unknown opcode, function or
sub-opcode; the FPU's unimplemented-operation refusal for an undefined function, for a same-format conversion
(`cvt.s.s`, `cvt.d.d`, `Mars_FpuMath.md` §6), for any arithmetic in the W and L formats, and for a `BC1` selector above
three (`Mars_FpuMath.md` §7.1).

`CFC0`, `CTC0` and the four `BC0` branches are named although they do nothing: the corpus measured that the part
decodes them and refuses the sub-opcodes around them (`Mars_Cop0.md` §10.2), so they are instructions, if idle ones.

### 3.1 Where `.word` and the interpreter part company

The rule says "names the encoding *and* executes it". Six groups of encodings are accepted by the interpreter — run or
silently ignored, but not refused — and have no architectural name, so they print as `.word`; a reader of a listing
should know what they do anyway:

| encodings | what the interpreter does | why `.word` |
| --- | --- | --- |
| VR4300 COP0 `CO` functions other than the five and `0x10` | nothing (`Mars_Cop0.md` §10) | reserved; no name |
| VR4300 `BC0` with `rt` above three | nothing | MIPS III defines four selectors |
| RSP `REGIMM` with `rt` outside 0, 1, 16, 17 | **branches**: bit 4 links, bit 0 chooses `≥ 0` over `< 0` | no reference names them |
| RSP vector functions 30, 31, 46, 47, 59 | zero `vd`, sum into the accumulator (`Mars_RspVector.md` §11) | the references disagree on a name (§7.2) |
| RSP `LWC2` format 10 | nothing | the corpus: "LWV doesn't exist - it does nothing" |
| every other RSP encoding outside the scalar set and the vector unit | nothing (`Mars_Rsp.md` §6) | not the RSP's |

**The `REGIMM` row is a finding about the interpreter, not about the disassembler.** `Rsp.ExecuteRegImm` branches on
every `rt` by its bits, so `0x04820004` branches like `bltz`. `Mars_Rsp.md` §6 says an encoding the RSP does not own
does nothing, and this one does something. Nothing is known to test it: the corpus's assembler emits only the four
defined selectors, and no reference decodes the others. Which of the page and the code is right is unmeasured; the
disassembler does not pretend to know, and says `.word`.

The other direction has one family: **architectural names the rule withholds because the interpreter refuses the
encoding.** MIPS III defines `LWC2`, `LDC2`, `SWC2` and `SDC2`, and the coprocessor-2 branches `BC2F`, `BC2T`, `BC2FL`
and `BC2TL`; Mars raises Reserved Instruction for all eight — the loads and stores deliberately, while nothing says
what the part does (`Mars_Fpu.md` §8.1), and the branches because this part's coprocessor 2 is one latch with no
condition (§8). They print as `.word` for as long as the interpreter refuses them.

## 4. Static references

**`Call` is every control transfer whose target is in the word**: `j`, `jal`, every conditional branch including the
likely and linking forms, and the `BC0` and `BC1` branches. That is wider than "a call", and deliberately: the `Kind`
feeds `callers`, whose contract is "static call/jump references", and the other cores classify the same way — Mercury's
SM83 counts `JP` and `JR` and their conditional forms, Moon and Venus count `JMP`. `jr` and `jalr` name nothing,
because their target is in a register.

- **A branch counts from its delay slot**: `address + 4 + (sign-extended offset << 2)`, wrapped at thirty-two bits.
  That is what the interpreter does — `BranchIf` adds to `Pc`, which already holds the delay slot's address when the
  branch executes (`Cpu.Step`).
- **A jump keeps the top four bits of its delay slot**, not its own: `((address + 4) & 0xF0000000) | (index << 2)`.
  The two differ only for a jump in the last word of a 256MB region — `j` at `0x8FFFFFFC` goes to `0x90000000`, not
  `0x80000000` — and that is the case the interpreter's `JumpTarget` gets right by reading `Pc`, and the case a test
  pins.
- **On the RSP both wrap inside IMEM**: a branch is `(address + 4 + (offset << 2)) & 0xFFC`, a jump `(index << 2) &
  0xFFC`, as `Rsp.BranchIf` and `Rsp.Jump` compute them.

**`Read` and `Write` are loads and stores whose base register is `zero`**, the only case a single word fixes the
address. The target is the sign-extended offset as a 32-bit address on the VR4300, and the offset masked to twelve bits
on the RSP — for a vector transfer, after scaling (§5.3). `cache` names nothing: it reaches no data.

**What this does and does not buy.** On the RSP it is useful: microcode addresses DMEM from `zero` constantly, so a
static scan for writers of DMEM `0x3F0` finds the `sqv`s that store there. On the VR4300 it is correct and rarely
useful: an address formed from `zero` lies in `0x00000000`–`0x00007FFF` or `0xFFFF8000`–`0xFFFFFFFF`, which is mapped
space no game touches this way. Real code builds addresses with `lui` and an offset, and following that pair is analysis across words, which a
decoder of one word cannot do and does not try to.

## 5. The RSP notation

The scalar half is spelled exactly as the VR4300 is, with three-digit targets. `lwu` is named: the interpreter executes
it as a word load, and the corpus's assembler has it.

### 5.1 The element selector

**The SGI spelling**: nothing for the whole register, `[0q]`/`[1q]` for the quarters, `[0h]`–`[3h]` for the halves,
`[0]`–`[7]` for one element broadcast.

| selector | 0, 1 | 2 | 3 | 4–7 | 8–15 |
| --- | --- | --- | --- | --- | --- |
| suffix | none | `[0q]` | `[1q]` | `[0h]`–`[3h]` | `[0]`–`[7]` |

The other candidate was the raw number, `v3[e]`, which parallel-rsp prints. The SGI form was chosen because it names
the pairing the selector makes rather than a code the reader must look up, and because it is what the references that
print a selector at all agree on: Project64 prints exactly these spellings, and the corpus's assembler names its
selector values `All`, `All1`, `Q0`, `Q1`, `H0`–`H3` and `_0`–`_7`. **The cost is that selectors 0 and 1 print alike.**
They mean the same thing in the interpreter's `Selected`, in cxd4's element table and in the corpus's
`EFFECTIVE_INDEX`, so nothing a listing reader needs is lost; the word column holds the difference.

`vsar` uses the same suffix, which reads naturally: `vsar v1, v2, v3[0]` is selector 8, the accumulator's high third,
and the corpus's assembler names 8, 9 and 10 `High`, `Mid` and `Low`.

### 5.2 Operand forms

- **The computational functions print `vd, vs, vt[e]`**, including the ones that do not read every field — `vrndp` and
  `vrndn` read `vs` only for its parity, and `vmacq` and `vsar` read neither source (`Mars_RspVector.md` §6, §9). The form
  is the encoding's; what each function does with it is that page's business.
- **The single-lane functions print `vd[de], vt[e]`**: `vrcp`, `vrcpl`, `vrcph`, `vrsq`, `vrsql`, `vrsqh` and `vmov`,
  with `de` the low three bits of the `vs` field, which is where the destination lane lives. With `e` from 8 to 15 —
  which is what an assembler emits for `vt[k]` — `vrcp v1[5], v2[3]` reads correctly as lane 3 in, lane 5 out. With
  `e` below 8 the interpreter still takes the source lane from `e & 7` but shuffles `vt` by the whole selector for the
  accumulator, and the suffix shows the shuffle; **this is the one place the notation does not say which lane the
  scalar comes from.**
- **`vnop` and `vnull` print no operands**, whatever their fields hold.

### 5.3 Transfers

- **A vector load or store prints `vt[e], offset(base)`**, where `e` is the byte index 0–15 the interpreter reads from
  bits 10:7, and `offset` is the **byte offset after scaling** — the seven-bit field sign-extended and shifted by the
  format's size — not the raw field:

  | format | `bv` | `sv` | `lv` | `dv` | `qv` | `rv` | `pv` | `uv` | `hv` | `fv` | `wv` | `tv` |
  | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
  | shift | 0 | 1 | 2 | 3 | 4 | 4 | 3 | 3 | 4 | 4 | 4 | 4 |

  So `0xE801203F` is `sqv v1[0], 0x3F0(zero)` and `0xE8012040` is `sqv v1[0], -0x400(zero)`. Every reference that
  scales at all agrees on this table (§7.1 — parallel-rsp prints the field unscaled), and it is the one the
  interpreter's `TransferScale` holds.
- **`mfc2` and `mtc2` print `rt, vN[e]`** with the same byte index, which is what they move (`Mars_RspVector.md` §3).
- **`cfc2` and `ctc2` name the flag register**: `vco`, `vcc`, and `vce` for both 2 and 3, by the two bits the
  interpreter reads.
- **`mfc0` and `mtc0` name the interface register by libultra's name** — `SP_MEM_ADDR` through `SP_SEMAPHORE`, then
  `DPC_START` through `DPC_TMEM` — by the four bits the interpreter reads (`Mars_Rsp.md` §5), so index 20 prints as
  `SP_STATUS`.

## 6. The capstone differential

**Method.** A Python script outside the repository generates three sets of encodings, decodes each with capstone (MIPS64,
big-endian) and with Mars (through a scratch console program over the built `EmuSen.dll`), normalises both, and assigns
every disagreement a class from an explicit table; a disagreement the table does not cover prints as unclassified, and
the run is not accepted until there are none.

- **canonical** — every form Mars names (219 of them, each float condition and format separately), 400 instances
  each, operand fields random and the fields the architecture requires zero set to zero;
- **dirty** — the same forms with those must-be-zero fields random and non-zero, 100 each for the 138 forms that have
  any;
- **random** — 100,000 uniformly random words.

Addresses are random, weighted towards region and 32-bit boundaries so the wrap cases are exercised. Each word is
first matched to the form it instantiates by its fixed bits, independently of either decoder's output, which is what
lets the script prove rather than assume that a capstone refusal is about a field Mars ignores.

**Normalisation.** Beyond `$`, case and number base, seven rules — each counted over the first seed's agreements, so it
is visible how much of the agreement each one carries:

| rule | agreements it was needed for |
| --- | --- |
| register 30: capstone's `$fp` is `s8` | 6,683 |
| a coprocessor register compared by index (capstone spells COP0 and COP2 registers as general-register names, and appends a select) | 4,001 |
| the divides: capstone prints the assembler's three-operand form, `div $zero, $t1, $ra` | 1,600 |
| the trap immediates: capstone prints the raw sixteen bits, Mars the sign-extended value; compared modulo 2¹⁶ | 1,357 |
| branch targets: capstone carries past thirty-two bits, printing `0x10000b53c` or a negative number where the target wraps; compared modulo 2³² | 790 |
| `break`: capstone splits the code ten and ten | 420 |
| a zero memory offset, which capstone omits: `($a0)` | 0 — one access in 65,536 has one, and none was drawn |

The coprocessor rule compares indices, so it cannot see a wrong *name*; the names are the unit tests' job (§8), which
check all thirty-two COP0 indices against the manual.

**Result.** Two seeds, 201,400 encodings each (87,600 canonical, 13,800 dirty, 100,000 random), and **no disagreement
the classification table does not account for** in either:

| class | seed 20260918 | seed 7 |
| --- | --- | --- |
| **A** agree after normalisation | 154,375 | 154,132 |
| **A** both refuse | 10,746 | 10,796 |
| **N4** capstone spells `jalr zero, rs` as `jr rs` | 10 | 13 |
| **S0** capstone refuses: a must-be-zero field Mars does not read is non-zero | 22,720 | 22,852 |
| **S1** capstone refuses: COP0 `CO` bits 24:6 non-zero (`Mars_Cop0.md` §10.1) | 566 | 573 |
| **S2** capstone prints a MIPS32 COP select (bits 2:0) | 6 | 6 |
| **S4** capstone prints a MIPS32 `sync` stype | 120 | 117 |
| **S5** capstone re-reads a right shift's must-be-zero bit as a MIPS32r2 rotate (`rotr`, `drotrv`, …) | 15 | 28 |
| **S6** capstone re-reads `mult`'s bits 12:11 as a DSP accumulator | 1 | 2 |
| **I0** capstone decodes an instruction from outside MIPS III on an encoding the VR4300 reserves | 3,273 | 3,321 |
| **I1** capstone decodes MIPS IV `pref` on opcode `0x33` | 1,532 | 1,555 |
| **I2** capstone refuses `cfc0`/`ctc0`, which MIPS32/64 reserve and the VR4300 decodes | 800 | 800 |
| **I3** capstone reads a MIPS IV condition code in `bc1` where the VR4300 refuses `rt` > 3 | 55 | 40 |
| **I4** the same in `bc0` | 38 | 31 |
| **I5** capstone decodes MIPS III's `bc2f`/`bc2t`/`bc2fl`/`bc2tl`, which Mars refuses | 51 | 51 |
| **D1** capstone decodes Octeon's `bbit0`/`bbit032`/`bbit1`/`bbit132` on the `lwc2`/`ldc2`/`swc2`/`sdc2` opcodes | 6,292 | 6,283 |
| **D2** capstone has no `cfc2`/`ctc2` | 800 | 800 |

On the canonical set alone, every one of the 87,600 encodings agrees except the forms capstone cannot decode at all
(`cfc0`, `ctc0`, `cfc2`, `ctc2`: 1,600) and the ten `jalr zero` spellings.

**Every class, and what it is:**

- **A** — agreement. It covers every other form Mars names: all of MIPS III's integer set including the doubleword,
  unaligned, linked and trap instructions, the COP0 moves and TLB operations, every COP1 move, branch, arithmetic,
  conversion and compare in each format the part has, and the COP1 loads and stores.
- **N** — notation only. `jr` for `jalr zero, rs` is capstone's alias; Mars keeps the encoding's own name (§2.1).
- **S** — strictness. Capstone refuses an encoding, or re-reads it as a later instruction, because a field the VR4300
  requires zero is not; the interpreter does not read that field, so it executes the instruction the other fields
  name, and so does the disassembler (§3). **In every one of these words the script found the form and the non-zero
  must-be-zero bits itself**, from the form table and not from either decoder, so none is a disagreement explained
  after the fact.
- **I** — ISA scope. Capstone decodes MIPS IV, V, MIPS32/64 releases 1, 2 and 6, MIPS16e's `jalx`, MSA, the DSP
  extension, Octeon, and MIPS I/II's coprocessor-3 branches on encodings the VR4300 reserves; or refuses the two MIPS
  III COP0 transfers that MIPS32 dropped. By sub-architecture, for the first seed's I0: MIPS16e 1,485, MSA 1,061,
  MIPS IV 196, Octeon 151, MIPS64r2 130, MIPS32r2 98, DSP 66, COP3 branches 44, MIPS32 33, MIPS32r6 and MIPS64r6 7,
  MIPS V 2. **I5 is different in kind**: the coprocessor-2 branches are MIPS III instructions, and capstone is right
  to name them; Mars refuses them because this part's coprocessor 2 is one latch with no condition to branch on
  (`Mars_Fpu.md` §8), and the disassembler follows the interpreter (§3.1). That was first classified with I0, as a
  later architecture's instruction, and corrected on reading the class's mnemonic list.
- **D** — capstone defects, for this purpose. **D1**: in its generic MIPS64 mode capstone decodes Cavium Octeon's reuse
  of opcodes `0x32`, `0x36`, `0x3A` and `0x3E`, which MIPS64 itself assigns to `lwc2`, `ldc2`, `swc2` and `sdc2`; there
  is no mode that turns it off. (Mars prints `.word` there for its own reason, §3.1, so even a correct capstone would
  disagree.) **D2**: capstone has no `cfc2` or `ctc2` at all, though MIPS III and MIPS32/64 both define them.

**What the result says.** No disagreement is Mars's. **The Kind and Target of every agreeing word — 154,375 and
154,132 — matched what capstone's own operands imply**: the printed target of every branch and jump, and the offset of
every access based on `zero`.

**What it does not say.** The decoder was written with capstone's probe output already in view for the aliases (§2.1)
and the operand forms, so agreement on spelling is by construction and proves nothing. Field extraction, operand order,
sign extension and target arithmetic were not tuned to it, and those are what the run grades.

**Predictions that did not survive:**

- ~~"`CS_MODE_MIPS3` is the mode that matches the VR4300."~~ With `MIPS3` alone capstone decodes nothing — not even
  `0x00000000` — and with `MIPS3 | MIPS64` it behaves exactly as `MIPS64`, Octeon and MSA included. There is no
  MIPS III mode to restrict it to; §0's caveat about the superset is the consequence.
- ~~"Capstone has no `dmult`."~~ The first probe word for `dmult`, `0x0085101C`, came back undecoded, and was read as a
  gap. It has `rd` = 2, a field `dmult` requires zero; the canonical `0x0085001C` decodes. That misreading is why the
  script identifies each word's form and dirty bits before a refusal may be blamed on either side.

## 7. The RSP references

### 7.1 Field decoding

| question | corpus assembler | Mars's interpreter | cxd4 | Project64 | parallel-rsp |
| --- | --- | --- | --- | --- | --- |
| transfer offset scale, by format 0–11 | 0 1 2 3 4 4 3 3 4 4 4 4 (the byte offset it is given must be a multiple, and is shifted) | same (`TransferScale`) | same (`1*offset` … `16*offset`) | same | **none**: prints the raw field |
| transfer element | bits 10:7 | bits 10:7 | bits 10:7 | bits 10:7 (`del`) | bits 10:7 |
| seven-bit offset sign | signed, −64 to 63 | signed | signed | signed | signed |
| `mfc2`/`mtc2` element | bits 10:7 | bits 10:7 | bits 10:7 (`vd >> 1`) | bits 10:7 (`sa >> 1`) | bits 10:7 |
| `cfc2`/`ctc2` index | names 0, 1, 2; any index through `write_cfc2_any_index` | `& 3`, 2 and 3 both `VCE` | — | `% 4`, printed as a number | printed as a GPR name |
| `mfc0`/`mtc0` index | names 0–9, 11, 12 | `& 0xF` | `% 16` | all five bits; 16 and above "Unknown Register" | printed as a GPR name |
| vector fields | `vd` 10:6, `vs` 15:11, `vt` 20:16, `e` 24:21 | same | same | same | same |
| single-lane destination | the `vs` field | `vs & 7` | — | `vs & 7` | — |
| branch target | — | `& 0xFFC` from the delay slot | — | from the delay slot, `& 0x1FFC` (IMEM at `0x1000`) | `& 0xFFC` from the delay slot |
| jump target | — | `(index << 2) & 0xFFC` | — | `(index << 2) & 0x1FFC` | `(index & 0x3FF) << 2` |

Every row agrees with the decoder where the reference answers at all, except for parallel-rsp's unscaled offsets,
Project64's five-bit COP0 index and Project64's `0x1000`-based targets, all three classified in §7.3.

### 7.2 Names

- **Function 29 is `vsar`.** The corpus's assembler, parallel-rsp and `Mars_RspVector.md` call it `VSAR`; Project64 calls
  it `VSAW`. Mars follows the corpus.
- **The fourteen undocumented functions that have a name in both the corpus and Project64 are named**: `vsut`,
  `vaddb`, `vsubb`, `vaccb`, `vsucb`, `vsad`, `vsac`, `vsum`, `vextt`, `vextq`, `vextn`, `vinst`, `vinsq`, `vinsn`.
  Project64 prints them as "Reserved (VSUT)" and so on; the corpus's assembler writes them by these names.
- **Functions 30, 31, 46, 47 and 59 have no name both agree on**: the corpus calls them `V30`, `V31`, `V46`, `V47`,
  `V59`; Project64 calls the first two `VACC` and `VSUC` and the rest `V056`, `V057`, `V073`, which are their octal
  numbers. Mars prints `.word` (§3.1). A spelling like the corpus's would have read as a vector register.
- **Load format 10 has no name.** Project64 prints `LWV` and the corpus's assembler has a `write_lwv`, used by the one
  test that checks it does nothing; cxd4 and parallel-rsp leave the slot empty. The store of format 10 is `swv` in all
  but parallel-rsp.

### 7.3 Disagreements, classified

| reference | disagreement | classification |
| --- | --- | --- |
| Project64 | branch and jump targets in a `0x1000`-based IMEM, and `J`'s bit 12 kept | notation: the same IMEM word; this decoder's contract (§1) is the offset |
| Project64 | immediates and offsets printed as unsigned sixteen bits | notation |
| Project64 | `cfc2 rt, 1` rather than `cfc2 rt, vcc` | notation |
| Project64 | `mfc0` index 16 and above named "Unknown Register" | **unresolved**: Mars's interpreter and cxd4 read four bits and would reach `SP_*`; nothing measured says what the part does |
| Project64 | `VMULU` prints its `vs` as a bare number: `"$v%d, %d, $v%d%s"` | Project64 defect (its format string) |
| Project64 | `VRNDP`/`VRNDN` print `vs & 1` as a register, `$v0` or `$v1` | notation: both readings name the field; Mars prints the field whole (§5.2) |
| Project64 | the single-lane source printed as `e & 7`; `VRCPH`, `VRSQH` and `VMOV` add the selector suffix as well | notation: the same fields; §5.2 says what Mars's form shows |
| Project64 | `B` for `bgez zero`, `BEQZ` when either register is zero, `JALR` always with two operands | notation |
| parallel-rsp | a vector transfer's offset printed as the raw seven-bit field, unscaled | parallel-rsp defect for a reader: the number is not the byte offset the transfer uses |
| parallel-rsp | `bgezal` printed as `bltzal` | parallel-rsp defect (copied case) |
| parallel-rsp | three-register operations printed `rd, rt, rs` | parallel-rsp defect: `subu` and `slt` read backwards |
| parallel-rsp | `mtc0` prints its operands swapped, and COP0 and vector registers as GPR names | parallel-rsp defect |
| parallel-rsp | `lfv` and `swv` missing, printed as `nop`; `vrndp`, `vmulq`, `vrndn`, `vmacq` missing | parallel-rsp gaps — it is a JIT's debug aid, and says so |
| parallel-rsp | `add`/`addu` both `addu`, `sub`/`subu` both `subu`, any write to `zero` printed `nop` | deliberate in parallel-rsp: identical semantics on the RSP |

**The cross-check against Mars's interpreter** was field by field against `Rsp.cs`, `Rsp.Vector.cs` and
`Rsp.VectorMemory.cs`: the same bit positions, the same scale table, the same four-bit COP0 and two-bit control index,
the same wrap. It is the strongest local evidence of what each field means, because those files pass the corpus; it is
also not independent of the decoder, which was written from them.

## 8. Tests and the breakage round

`MarsDisassemblerTests` holds 550 cases in 27 tests: every VR4300 form with a literal expected line (many of them words
the differential also graded), the alias table including the forms that must *not* alias, every COP0 name and every float
condition by index, branch arithmetic across zero, across `0xFFFFFFFC` and at a region's last word, the RSP's wraps,
every RSP interface name and vector function by index, the selector spellings, each transfer format's scale at the
largest and most negative offsets, the `Kind`/`Target` of both decoders, the `.word` space, and a sweep of 786,432
words through both decoders that must not throw.

**The breakage round**: each rule broken alone in the source, the tests run, the source restored from a copy. The
first round broke thirty-six rules and every one was caught:

| broken rule | caught by |
| --- | --- |
| signed immediates lose their sign extension | `An_integer_instruction_reads_as_its_listing_does` |
| a branch counts from itself, not its delay slot | five tests, 38 cases |
| a branch offset is not sign-extended | the three branch tests |
| a jump's region is its own, not its delay slot's | `A_jump_keeps_the_top_four_bits_of_its_delay_slot` |
| register 30 spelled `fp` | three listing tests |
| `EntryLo0` and `EntryLo1` swapped | `Every_coprocessor_zero_index_has_its_manual_name` |
| the D format printed as S | the COP1 listing test and the `.word` test |
| a same-format conversion no longer refused | `An_encoding_the_interpreter_does_not_execute_is_a_word` |
| the trap immediates printed unsigned | `A_branch_and_a_register_immediate_read_as_their_listing_does` |
| `move` keyed on `rs` rather than `rt` | `A_pseudo_instruction_is_used_only_where_the_alias_table_says` |
| a static address keyed on `rt` rather than the base | `An_access_based_on_zero_names_its_address` |
| `bc1` accepting a MIPS IV condition code | the `.word` test |
| the `break` code narrowed to ten bits | the integer listing test |
| the COP0 `CO` half decoded only with bits 24:21 clear | `A_coprocessor_zero_instruction_names_its_register` |
| compare conditions 10 and 11 swapped | `Every_compare_condition_has_its_name` |
| `jalr`'s short form keyed on `rd` = 0 | the integer listing test |
| the logical immediates sign-extended | the integer listing test |
| branches no longer call references | `A_branch_or_jump_names_its_target_as_a_call` |
| `lwc2` decoded although the interpreter refuses it | the `.word` test |
| the RSP's `lrv` scaled by 8 | `An_rsp_vector_transfer_prints_its_scaled_byte_offset` |
| the RSP's `lhv`/`shv` scaled by 8 | the same |
| the seven-bit offset not sign-extended | the same, and the DMEM address test |
| an RSP branch not wrapped inside IMEM | `An_rsp_branch_or_jump_lands_inside_instruction_memory` |
| the quarter selector off by one | `An_rsp_vector_operation_spells_its_selector_the_sgi_way` |
| selector 1 read as a single element | the same |
| `mfc2`/`mtc2`'s element read from bit 6 | `An_rsp_transfer_names_its_register_the_way_the_interpreter_reads_it` |
| a transfer's element read from bit 6 | the scaled-offset test |
| the RSP COP0 index read as three bits | `Every_rsp_coprocessor_zero_index_has_its_interface_name` |
| the `cfc2` index read as one bit | the transfer test |
| a single-lane destination taken from `vd` | the selector test |
| `lwv` named | `An_rsp_encoding_the_processor_does_not_own_is_a_word` |
| RSP jump targets in Project64's `0x1000`-based space | the IMEM test |
| `vsar` one slot along in the name table | three tests, including `Every_rsp_vector_function_has_its_reference_name` |
| the DMEM target not masked to twelve bits | `An_rsp_access_based_on_zero_names_its_dmem_address` |
| `vnull` given operands | the selector test |
| RSP `REGIMM` decoded by bit pattern, as the interpreter executes it | the RSP `.word` test |

**A round with no survivors is a reason to suspect the round**, so a second one aimed at rules the first had not
reached. Six more were caught (a widened trap code, `negu` keyed on `rt`, a zero code printed as `0x0`, a data word
without its leading zeros, a COP0 move reading its register from `rt`, `bc0` left unnamed). **Four survived**, and all
four were the same oversight: the RSP decoder repeats the VR4300's alias rules and `break` code field rather than
sharing them, and only the VR4300's copies were tested: the RSP's `neg` alias removed, its `not` keyed on `rs`, its
`move`-through-`or` keyed on `rs`, and its `break` code narrowed to ten bits.
`An_rsp_alias_or_code_follows_the_same_rules_as_the_vr4300s` was added for them, and all four now fail it. Forty-six
rules broken in all; every one fails a named test.

## 9. What is not verified

- **The decoders are not wired in.** `MarsDebugTarget.Disassemble` and `ClassifyStaticReference` still return nothing;
  the change that uses these belongs to whoever owns that file.
- **No real code was disassembled.** Neither decoder has been run over a commercial ROM or microcode and read; the
  RSP side in particular has only its references and its tests.
- **The RSP `REGIMM` encodings outside the four** (§3.1): the interpreter branches, the disassembler says `.word`, and
  hardware has not been asked.
- **The RSP COP0 index above 15** (§7.3): two implementations read four bits, one disassembler refuses to name them,
  and no measurement exists.
- **The VR4300's `LWC2` and `BC2` families** stay `.word` for as long as the interpreter refuses them (§3.1).
- **A shared misreading of the MIPS manuals** by Mars and capstone would show as agreement (§0).
