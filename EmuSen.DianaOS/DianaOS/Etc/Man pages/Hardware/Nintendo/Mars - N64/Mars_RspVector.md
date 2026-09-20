# Mars — the signal processor's vector unit

*Phase C's second slice, landed 2026-09-17. Coprocessor two on the RSP: thirty-two
128-bit registers, three flag registers, a 48-bit accumulator, two hidden registers, and
the sixty-four vector functions and twenty-four load and store formats that reach them.
`Rsp/Rsp.Vector.cs` (state, transfers, dispatch), `Rsp/Rsp.VectorMath.cs`,
`Rsp/Rsp.VectorMemory.cs` and `Rsp/Reciprocals.cs`. Tests in `MarsRspVectorTests`.*

---

## 0. Where the specification came from

Nothing on this page was read out of another emulator. It was read out of the hardware
test corpus, in two different forms that deserve to be kept apart:

- **Models.** For the loads, the stores, the flag-setting instructions, `VMACQ` and the
  reciprocals, the corpus does not store expected values; it computes them, with a small
  simulation of the part written beside each test. Those simulations are the
  specification those instructions were implemented from, and each is only as good as
  the claim that the corpus passes on hardware — which is its stated purpose, and the
  reason it is the oracle (`Mars_TestOracle.md`).
- **Tables.** For the plain multiplies, the rounding instructions and the accumulator
  overflow cases, the corpus stores eight-element expected results, with no model. The
  implementation was written from the arithmetic those tables imply and then checked
  against them — by hand for a few rows while writing, and by the corpus run for all.

The distinction matters in exactly one place, §6.1, where a model and a table in the same
corpus disagree, and the table was followed.

## 1. The unit, in outline

**Thirty-two registers of eight signed or unsigned 16-bit elements**, addressed as
`V0`–`V31`. Mars stores them as one flat array, register first, element second. A
register's sixteen bytes are big-endian within it: byte 0 is the high byte of element 0.
That byte order is what the loads, stores and transfers see.

**Three flag registers.** `VCO` is sixteen bits, carry in its low byte and a not-equal bit
per element in its high byte. `VCC` is sixteen bits, a less-or-equal bit per element low and
a greater-or-equal bit high. `VCE` is **eight** bits, one per element.

**A 48-bit accumulator per element**, which the multiplies write whole and most other
instructions write only the low sixteen bits of (§6).

**Two hidden registers** belonging to the reciprocal instructions (§10), which no
transfer can read.

The encoding: `COP2` (opcode 18) with bit 25 set is a vector operation, with the function
in bits 5:0, `vd` in 10:6, `vs` in 15:11, `vt` in 20:16 and the element selector in
24:21. With bit 25 clear it is a register transfer. `LWC2` and `SWC2` (opcodes 50 and 58)
carry a format in bits 15:11, an element in 10:7 and a seven-bit signed offset, which each
format scales by its own size.

## 2. The element selector, and reading before writing

**The selector shuffles `vt` and nothing else.** Element `i` of the operation pairs `vs[i]`
with one element of `vt` that the selector chooses:

| selector | element of `vt` paired with `i` |
| --- | --- |
| 0, 1 | `i` |
| 2, 3 (quarters) | `selector − 2 + (i & 6)` |
| 4–7 (halves) | `selector − 4 + (i & 4)` |
| 8–15 (one element) | `selector − 8`, for every `i` |

**Every source element is read before any destination element is written.** The corpus
tests `vd = vt`, `vd = vs` and `vt = vs` for most instructions, and with a selector the
difference is visible. `VOR V1, V1, V2[h0]` pairs elements 0–3 with `V1[0]`; written in
place, element 0 is overwritten first and elements 1–3 read the new value. With `V2` zero
the new value equals the old one and the mistake is invisible, which is how the unit test for
this rule was first written — it was rebuilt with a non-zero `V2[0]`, and an in-place `VOR`
now fails it. Mars copies `vs`, the selected `vt` and `vd` out to eight-element scratch spans
at the start of every operation and writes `vd` back once at the end, with no
per-instruction exceptions.

## 3. Transfers between the two halves

**`MTC2` and `MFC2` move sixteen bits at a byte position, not an element.** The element
field is a byte index from 0 to 15. `MTC2` writes the GPR's low sixteen bits to bytes `e`
and `e + 1`; `MFC2` reads them back and sign-extends.

**At byte 15 the two disagree.** `MTC2` writes only byte 15 and drops the low byte of the
value — there is no byte 16 and no wrap. `MFC2` at byte 15 reads byte 15 and then byte 0.
The corpus tests both, and the asymmetry is the whole content of both tests.

**`CFC2` and `CTC2` look only at the index's low two bits:** 0 is `VCO`, 1 is `VCC`, and
both 2 and 3 are `VCE`. So index 5 is `VCC` and index 30 is `VCE`. `CFC2` sign-extends from
sixteen bits, which means `VCE`, being eight, never reads back negative. A write keeps only
what the register has room for.

## 4. Loads

Each format has its own size, its own alignment and its own rule for what happens at the
register's end. By format:

| format | reads | into the register |
| --- | --- | --- |
| `LBV`, `LSV`, `LLV`, `LDV` | 1, 2, 4, 8 bytes from the address | from byte `e`, **stopping at byte 15** |
| `LQV` | from the address to the end of its 16-byte region | from byte `e`, stopping at byte 15 |
| `LRV` | from the start of the address's region up to the address | ending at byte 15, shifted up by `e`, the overflow dropped |
| `LPV`, `LUV` | eight bytes, rotated by the address's misalignment less `e` | each widened to an element, `<< 8` or `<< 7` |
| `LHV` | as `LUV`, stepping two bytes per element | each `<< 7` |
| `LFV` | bytes at a fixed pattern of offsets, one per element; with `e` of 0 or 8 that is four bytes each read twice | eight bytes of the widened result, from byte `e` |
| `LWV` | nothing | nothing — the format does not exist |
| `LTV` | sixteen bytes | **one element into each of eight registers** |

**A load never wraps inside the register.** A load that has more bytes than the register
has room for after `e` loads fewer bytes; it does not continue at byte 0. This is the
difference from stores (§5), and it is the rule most likely to be carried across wrongly
from one to the other.

**In memory, what reaches the end continues at the start.** The corpus's own comment names
`LSV`, `LLV` and `LDV` as the formats that can overflow and says the others stay inside their
region by aligning first. That holds for `LQV` and `LRV` — the quad load at `0xFF5` reads
eleven bytes and stops — but not for the rotated formats: `LPV`, `LUV`, `LHV`, `LFV` and `LTV`
read a sixteen-byte window from an 8-byte alignment, so from `0xFF8` the window runs to
`0x1007`, and the corpus's own model of *LPV (end of DMEM)* wraps there. Mars masks every byte
address to twelve bits, which gives both behaviours without a rule for either.

**`LTV` spreads across a register group.** `vt & ~7` names a group of eight registers, and
the load writes element `i` of register `group + ((e/2 + i) & 7)` — one element into each.
Which bytes it reads is rotated by bit 3 of the address as well as by `e`, and the corpus's
model is followed exactly, including that rotation.

**`LFV`'s pattern has one asymmetry that looks like a typo:** element 0 reads the byte at
`misalignment + e`, where every other element reads at `k − e`. It is in the corpus's model in
both of its tests, the selectors those tests use tell `+ e` from `− e` apart, and Mars passes
them with `+ e`. That makes it as certain as the rest of the corpus, and no more.

## 5. Stores

| format | writes | from the register |
| --- | --- | --- |
| `SBV`, `SSV`, `SLV`, `SDV` | 1, 2, 4, 8 bytes at the address | from byte `e`, **wrapping to byte 0** |
| `SQV` | from the address to the end of its region | from byte `e`, wrapping |
| `SRV` | from the start of the region up to the address | the register's tail, rotated by `e` |
| `SPV`, `SUV` | eight bytes | each element narrowed, `>> 8` for one half of the register and `>> 7` for the other |
| `SHV` | eight bytes at every other position of the region | a byte pair read across element boundaries, `>> 7` |
| `SFV` | four bytes at every fourth position | four elements `>> 7`, **or four zeroes** |
| `SWV` | sixteen bytes, rotated by misalignment | the whole register |
| `STV` | sixteen bytes | **two bytes from each of eight registers** |

**A store wraps inside the register; a load does not.** `SDV` from byte 12 stores bytes
12–15 and then 0–3. The unit test that pins this pairs it with the `LDV` that does not
wrap, so either rule broken alone fails it.

**`SPV` and `SUV` swap which half gets which shift.** The shift is chosen by bit 3 of
`e + i`, not by `i`, so a selector moves the boundary.

**`SFV` stores something for eight selectors only** — 0, 1, 4, 5, 8, 11, 12 and 15 — each
starting at its own element and continuing within that half of the register. Every other
selector stores four zero bytes. The table is the corpus's, and Mars keeps it as a table;
no rule was found that generates it, and none was invented.

**`STV` is `LTV`'s inverse and is not symmetric with it.** The source register for each
byte depends on the address's alignment as well as on `e`, and at addresses whose bit 3 is
set the bytes land in a rotated order. The corpus's model was followed as written.

## 6. The accumulator

**Forty-eight bits per element, and it wraps.** Mars stores each element's accumulator
masked to 48 bits and sign-extends it whenever it is read as a number. Adding past the top
wraps negative, and the corpus tests exactly that for every accumulating multiply (§6.1).

**Who writes what:**

- The multiplies (§7) and the rounding instructions (§9) write all 48 bits.
- Almost every other function writes **only the low sixteen bits**, with the value it wrote
  to the destination — or, for addition, subtraction and `VABS`, a value that was not
  clamped (§8). The high thirty-two bits are left alone, so an addition after a multiply
  leaves a hybrid.
- `VSAR` writes nothing to it. Selectors 8, 9 and 10 read the high, middle and low sixteen
  bits into `vd`; every other selector reads zero.
- `VNOP` and `VNULL` write nothing at all (§11).

### 6.1 Two specifications in one corpus, and the one that was followed

The corpus's stress tests (which the corpus does not run by default) carry a simulator for
each accumulating multiply. **Four of the six — `VMACU`, `VMADL`, `VMADM` and `VMADN` —
compute the clamp on the unwrapped 64-bit sum.** The other two, `VMACF` and `VMADH`, truncate
bits 47:16 to a 32-bit integer before clamping, which amounts to wrapping. The corpus's
ordinary tests include an *accumulator itself overflowed* case for all six, and **those require
the clamp to read the accumulator after it has wrapped**: `VMADL` driven past
`0x7FFF_FFFF_FFFF` must produce `0x8000_0000_3FEF` in the accumulator and `0` in the
destination, the negative clamp, where its stress simulator's arithmetic would produce
`0xFFFF`.

The two cannot both be right, but they only disagree when a sum leaves 48 bits, and the four
simulators that do not wrap never see one: each stress batch starts from a cleared accumulator
and runs thirty-two steps, and none of those four families adds more than 2³¹ in a step. (`VMADH`
can leave 48 bits inside a batch — each step can add 2⁴⁶ — and its simulator is one of the two
that wraps.) So the disagreement is a gap in simulators that were never exercised where they
are wrong, not a contradiction in what hardware was observed to do. Mars follows the overflow
tests: the accumulator wraps first, and every clamp reads what it wrapped to. The unit test for
this reproduces the corpus's `VMADL` case exactly.

## 7. The multiplies

Every multiply is a 16×16 product with a signedness per operand, added to either a
constant or the existing accumulator, with a clamp that depends on the family. Writing it
that way collapses fourteen instructions into five families, which is how Mars implements
them — the plain and accumulating forms of a family differ only in what they add the
product to.

| family | plain / accumulating | product | added to the accumulator as | destination |
| --- | --- | --- | --- | --- |
| fraction | `VMULF` / `VMACF` | `vs × vt`, signed × signed | `product << 1` | signed clamp of bits 47:16 |
| unsigned fraction | `VMULU` / `VMACU` | signed × signed | `product << 1` | 0 if negative, `0xFFFF` if ≥ 2³¹, else bits 31:16 |
| low | `VMUDL` / `VMADL` | unsigned × unsigned | `product >> 16` | bits 15:0, **if** bits 47:16 fit sixteen signed bits, else 0 or `0xFFFF` |
| middle | `VMUDM` / `VMADM` | signed `vs` × unsigned `vt` | `product` | signed clamp of bits 47:16 |
| normal | `VMUDN` / `VMADN` | unsigned `vs` × signed `vt` | `product` | as *low* |
| high | `VMUDH` / `VMADH` | signed × signed | `product << 16` | signed clamp of bits 47:16 |

**The plain fraction multiplies add `0x8000` and the accumulating ones do not.** The plain
form starts from `0x8000`, a rounding constant; the accumulating form starts from what was
there. For the other families the plain form starts from zero. `VMULF`'s one overflow,
`0x8000 × 0x8000`, is the only input that makes its clamp do anything.

**`VMULQ` is the odd one.** The product is shifted up sixteen, biased by thirty-one (as
`0x1F_0000`) if negative, and the destination is the signed clamp of bits 47:17 with its low
four bits cleared — so a positive overflow writes `0x7FF0`, not `0x7FFF`.

## 8. Addition, comparison, clipping, and the flags

In the table, *carry* is `VCO`'s low bit for the element and *not-equal* its high bit.

| instruction | destination | flags |
| --- | --- | --- |
| `VADD`, `VSUB` | signed clamp of `vs ± vt ± carry` | `VCO` cleared |
| `VABS` | `vt`, `−vt` or 0 by the sign of `vs`; `−0x8000` clamps to `0x7FFF` | none |
| `VADDC` | `vs + vt`, unsigned, wrapping | `VCO` = carries, high byte cleared |
| `VSUBC` | `vs − vt`, unsigned, wrapping | `VCO` = borrow low, not-equal high |
| `VLT`, `VGE` | the chosen one of `vs`, `vt` | `VCC` low = chosen, high cleared; `VCO` cleared |
| `VEQ` | `vt`, always | as `VLT` |
| `VNE` | `vs`, always | as `VLT` |
| `VCL` | `vs`, `vt` or `−vt` | `VCC` updated where not-equal was clear; `VCO`, `VCE` cleared |
| `VCH` | `vs`, `vt` or `−vt` | all three written |
| `VCR` | `vs`, `vt` or `~vt` | `VCC` written; `VCO`, `VCE` cleared |
| `VMRG` | `vs` where `VCC`'s low bit is set, else `vt` | `VCO` cleared |
| `VAND` … `VNXOR` | the bitwise result | none |

**The accumulator's low third gets the unclamped value** for `VADD` and `VSUB`, and the
unclamped negation for `VABS`. For everything else in this table it gets what was written to
the destination.

**The comparisons read whatever `VCO` holds.** `VLT` counts an equal pair as less when both of
that element's carry and not-equal bits are set; `VGE` counts it as greater-or-equal unless
both are; `VEQ` refuses an equal pair whose not-equal bit is set, and `VNE` accepts any pair
whose not-equal bit is set. A reading, not something the corpus tests: flags left by a
`VSUBC` on the low halves of two 32-bit values would let these compare the high halves and
decide the whole.

**`VCL` is the second half of a clip whose first half is `VCH`.** It reads the flags `VCH`
left: where the signs differed it decides less-or-equal from the sum and `VCE`; where they did
not it decides greater-or-equal from an unsigned comparison; and wherever `VCH` recorded
not-equal, it keeps the comparison `VCH` made instead of deciding a new one. It then clears
`VCO` and `VCE`. The corpus runs four `VCO` values against four `VCC`-and-`VCE` pairs for each
of the sixteen selectors, 256 programs for this one instruction.

## 9. Rounding

**`VRNDP` and `VRNDN` add `vt` to the accumulator when the accumulator is non-negative and
negative respectively**, then write the signed clamp of bits 47:16. `vt` is shifted left
sixteen first **if `vs`'s register number is odd**. The contents of `vs` are never read; the
corpus fills it with pseudo-random data and runs every even and every odd register to show it.

**`VMACQ` ignores `vs` and `vt` entirely** — the corpus uses pseudo-random register numbers and
selectors to show that too. If bit 21 of the accumulator is clear, it moves the accumulator
`0x20_0000` toward zero, unless the accumulator is non-negative and below 2²², which it leaves
alone; the destination is then the signed
clamp of bits 47:17 with the low four bits cleared, as for `VMULQ`. The corpus's model for
this was followed as written; its reason is not given and none is offered here.

## 10. The reciprocals

**Six instructions share one lookup, two tables and two hidden registers.** `VRCP` and `VRSQ`
take a 16-bit input, sign-extend it, and write the low half of a 32-bit reciprocal (or
reciprocal square root) to `vd`. `VRCPH`/`VRSQH` and `VRCPL`/`VRSQL` split a 32-bit input and
result across two instructions. For all six:

- **the input is `vt[e & 7]`**, the raw register, not the selector-shuffled one;
- **the output goes to `vd[vs & 7]`**, and every other element of `vd` is left alone;
- **the accumulator's low third gets the whole selector-shuffled `vt`**, not the result.

`VMOV` belongs with them for the last two rules: it copies one element, `vt` shuffled, at
position `vs & 7`, and puts the whole shuffled `vt` in the accumulator.

**The hidden registers.** A *high* instruction writes the last result's upper word to `vd`,
stores its input as the upper half of the next 32-bit input, and **sets a flag saying that it
has**. A *low* instruction uses that stored upper half only if the flag is set — otherwise it
sign-extends its own 16-bit input — computes, writes the lower word, records the upper word,
and clears the flag. `VRCP` and `VRSQ` clear the flag too. The two reciprocal families share
all of this: a `VRCPH` sets up a `VRSQL`.

The flag is a separate bit and not a special value of the stored input; the corpus proves it
by trying all 65,536 values. **The flag persists across RSP runs**, which is why one of the
corpus's tests expects `0xFFFF` from a `VRCPL` that its own program never set up: the test
before it did.

**The tables are generated, not transcribed.** 512 entries each. The generation rules were
read from the corpus's test file, which states it ported them from ares under ares's
permissive licence; Mars's code was written from those formulas, not from either file, which
is the discipline `Mars_References.md` §2 set in advance for exactly these tables. The
corpus's two *verify table* tests read every entry back out of the part.

### 10.1 Two things the lookup does that nothing explains

- **Negative inputs above `0xFFFF_8000` are taken one lower before being inverted.** The
  corpus's model says *"Why? No idea"*, and Mars implements it without an explanation
  either. `0xFFFF_8000` itself is a special case whose result is `0xFFFF_0000`.
- **A normalising shift of thirty-two is taken modulo thirty-two.** The corpus's model uses a
  wrapping shift, which in its language masks the shift count; C#'s shift operator masks it
  the same way, so the behaviour carries across without a special case. It happens only for
  an input magnitude of 1.

## 11. Encodings that do nothing, and encodings that zero

**`VNOP` (55) and `VNULL` (63) do nothing at all** — not to `vd`, not to the flags, not to the
accumulator.

**Every other undocumented function zeroes `vd` and writes `vs + vt` to the accumulator's low
third.** That is nineteen functions: `VSUT`, `VADDB`, `VSUBB`, `VACCB`, `VSUCB`, `VSAD`,
`VSAC`, `VSUM`, `VEXTT`, `VEXTQ`, `VEXTN`, `VINST`, `VINSQ`, `VINSN` and the unnamed 30, 31,
46, 47 and 59. The corpus tests every one. Mars makes this the dispatch's default, so a
function is a no-operation only by being named as one.

**Three of those tests carry a warning Mars does not model.** For `VSUM`, 30 and 31 the corpus
pads with three `NOP`s, and says that with fewer the test fails on hardware because *"one of
the previous multiplications will still be able to write to the accumulator"*. That is a
pipeline effect: a multiply's accumulator write lands some instructions after the multiply.
Mars executes every instruction to completion before the next, so it cannot reproduce the
failure, and nothing in the corpus requires it to.

## 12. What the corpus says

**All 155 vector groups pass**, in the first complete run after the unit was built, and the
corpus's tally moved from 319 failed assertions to 163. The run's census and the diff that
shows nothing else moved are `Mars_Corpus.md` §10. With this slice no RSP test in the corpus
fails except the four `spmem` groups, which test the main CPU's access to the RSP's memories
rather than the RSP (`Mars_Rsp.md` §8).

**What that does and does not establish.** For the parts implemented from a model (§0), a
pass means Mars agrees with the model at the inputs the corpus chose, and the models are the
corpus's claim about hardware rather than a measurement Mars made. For the parts implemented
from tables it means agreement at those rows. Neither covers the pipeline (§11), and §6.1 is
one place the corpus contradicted itself; a different corpus would look there first.

**One prediction made while writing this slice was wrong.** A hand check put one element of
the `VRNDP` overflow table `0x8000_0000` away from the model, and the slice began expecting to
record that group as an unexplained residual. It passed; the hand arithmetic was wrong, and a
simulation of the model reproduces the corpus's table exactly. `Mars_Corpus.md` §10 has the
detail.

## 13. What this slice does not do

- **The pipeline.** §11's delayed accumulator write, and anything else that depends on the
  RSP's instructions overlapping. No corpus test requires it; microcode that relied on it
  would be wrong on Mars.
- **The stress tests.** The corpus's own stress tests are not in its default run and take
  hours on hardware. §6.1 is the one place their simulators were read, and was not followed.
- **Performance.** Every vector operation copies three eight-element spans and writes one
  back. That is the simple way to get §2 right, and Phase G owns whether it is fast enough.

## 14. The same arithmetic, eight elements at a time

*2026-09-19, Phase G.* Every function in §6 to §11 is written as a loop over the eight elements, and
`Mars_Performance.md` §26 measured the signal processor at 27 to 36 per cent of a frame of gameplay, the largest
share left on the emulation thread once the display processor moved off it (`Mars_Rdp.md` §2.6). The unit is a
vector unit; the host has 256-bit vectors; the loops are there because the specification is written element by
element, not because the arithmetic needs them. `Rsp.VectorSimd.cs` is the same arithmetic in host vectors, and
`Rsp.VectorMath.cs` stays exactly as it was, as the reference the vector form is checked against
(`Rsp.UseSimd`, on where `Vector256.IsHardwareAccelerated`).

**What the layout had to become.** The register file is already eight sixteen-bit elements laid out register-first,
so a register is one 128-bit load and one store, and the element selector of §2 is one byte shuffle from a table of
the sixteen patterns — the same table the references reach for (`Mars_References.md`: cxd4 keeps it as `ei[]` for
its non-vector path and computes it inline otherwise; paraLLEl-RSP keeps sixteen `pshufb` keys). **The accumulator
could not stay as it was and did.** Forty-eight bits per element is eight 64-bit lanes, which is two 256-bit
vectors, and Mars stores them in exactly that order already, so the array the save state carries
(`Mars_SaveStates.md`) is loaded as two vectors and stored back as two. The references instead transpose the
accumulator into three vectors of eight sixteen-bit lanes — high, middle and low — and carry between them by hand,
because SSE2 has no 64-bit compare and no 64-bit shift-with-sign; with AVX2 the wide lanes are available and the
transposition buys nothing that the save state's layout does not already give. That is the one place this slice
departs from every reference read for it, and the reason is the host's instruction set, not the hardware's.

**How the pieces fall out.** A multiply is one 32-bit lane multiply: every product in §7 is a sixteen-by-sixteen
product with a signedness per operand, and all four combinations fit a signed 32-bit lane except unsigned by
unsigned, whose bits are right and are read back unsigned. The products widen to the accumulator's 64-bit lanes,
where the shift and the add of the family happen, the sum wraps to 48 bits as §6.1 requires, and the clamp of the
family is a minimum and a maximum. The three clamps of §7 keep their three shapes: the signed clamp of bits 47:16,
the unsigned one, and the low clamp that keeps a low word only while the word above it fits sixteen signed bits.
The flags are eight lane masks while the arithmetic runs and are packed to `VCO`, `VCC` and `VCE` by taking the
lanes' sign bits, which is the one idea taken from the references unchanged. The comparisons, the clips and the
merge of §8 become selects. The reciprocals of §10 stay scalar, a table lookup on one element, as they are in all
three references.

**Reading before writing, which §2 is about, is free here.** Both sources and the destination are loaded as whole
vectors before anything is computed and the destination is stored once at the end, so the aliasing rule holds by
construction rather than by copying to scratch spans.

### 14.1 What it was worth

Interleaved from the gameplay states of `Mars_Performance.md` §26, order rotated, three rounds of 600 frames each,
second halves, with the scan-out and the display processor's list already on their own threads (§27, §28 of that
page) so that what is measured is the emulation thread's own work:

| from the state, second 300 frames | element by element | eight at a time | of the console |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 47.6 fps | 49.5 | 99% |
| Wave Race 64 (NTSC, 60) | 50.7 | **60.1** | 100% |
| Super Mario 64, in the castle (PAL, 50) | 59.7 | **72.9** | 146% |

Medians of three; the two columns' rounds do not overlap in any row. **+4, +19 and +22 per cent**, and the order of
the three follows the profile that motivated the work: §26 put the signal processor at 27 per cent of a frame of
Ocarina of Time and 35 to 36 per cent of the other two, and Ocarina of Time's frame is bounded by its display
processor's list and its scan-out rather than by this. Wave Race 64 reaches its console's sixty from this change and
no other.

### 14.2 What the vector form is checked against

The element-by-element unit is the oracle. `MarsRspVectorSimdTests` runs **every one of the 64 function codes with
every one of the 16 selectors**, six rounds each, 6,144 cases: both paths are given the same thirty-two registers,
the same eight accumulators, the same three flag registers, and one instruction through the processor's own fetch
and dispatch; then all four registers the instruction can touch, all eight accumulators and all three flag
registers are compared. Operands are drawn two times in three at random and one in three from the eight values the
clamps and carries turn on — zero, one, `0x7FFF`, `0x8000`, `0x8001`, `0xFFFF`, `0x4000`, `0xC000` — and the
accumulators are seeded across all 48 bits. One round of every case names the same register as source and
destination, which is where §2's rule shows. The double-precision reciprocals are run as the pairs they are, the
high half then the low, over all sixty-four corner combinations, since the high half leaves state for the low one.

**The test was checked by mutation**, three of them, each reverted: the plain fraction multiply's rounding constant
changed from `0x8000` to `0x4000`, the low clamp's upper bound moved by one, and the element selector's last lane
pointed at the first. Each was caught — the first two by the sweep, the third by the sweep and the aliasing case
both. The corpus's 155 vector groups pass through the vector path as they do through the other (§12), which is a
weaker statement than the sweep and worth having anyway because it is the hardware's own test rather than Mars's.

## 15. The accumulator in three sixteen-bit thirds, and what the measurement said it was worth

*2026-09-20.* §14 kept the accumulator as the array a state carries, eight 48-bit values in 64-bit lanes, and every
operation of the eight-lane unit loaded it as two 256-bit vectors, sign-extended, added, wrapped and stored. Almost
every operation touches it, since every one that writes a register also writes its low third. The eight-lane unit
now keeps it as three vectors of sixteen-bit lanes, the high, middle and low thirds, and computes in them.

**The arithmetic.** A product is one 32-bit multiply in eight lanes as before, narrowed into its low and high
words. What is added is 48 bits made from those: for the fraction multiplies twice the product — the low word
doubled, the high word doubled with the low word's top bit carried in, and above them the product's sign, which is
right even for the one product past thirty-one bits, the most negative value squared, because its sign is taken
before the doubling; for the middle and normal multiplies the product sign-extended; for the high multiply the
product one third up; for the low multiply the unsigned product's upper word alone. `Add48` adds three lanes of
sixteen and carries: a sum below its addend carried, which is an unsigned compare; the carry into the middle third
can itself carry, when the middle sum was all ones, and that second carry is a second compare. There is no fourth
lane, which is the wrap. The clamps read the thirds directly: the middle third is the signed result while the high
third equals the middle's sign extension, and otherwise the nearer end by the high third's sign; the unsigned clamp
is nothing below zero and all ones once the upper thirds pass the largest positive word; the low clamp keeps the
low third under the same fit; the quarter multiply's result, the accumulator down seventeen bits, fits only while
the high third is all zeros or all ones. The two rounds and the accumulating quarter add a masked constant or the
sign-extended operand through the same `Add48`.

**The array stays the state.** The format is unchanged and so is every state on disk. The thirds are the truth
while the eight-lane unit runs and the array is brought up to date only when something asks: `WidenAccumulator`
before a state is written and at the top of the element-by-element unit, and `AccumulatorWritten` after a state is
read or a test fills the array. The cost is a branch an operation.

**The proof.** §14's 6,144 cases, every function and selector against the element-by-element unit with the clamps'
boundary values, pass unchanged. They step one instruction at a time from an array, so a carry kept wrongly would
be rewritten each case; `Chains_of_accumulator_operations_agree_with_the_unit_they_are_built_from` runs 4,000 chains
of ten operations from the accumulator's family over random registers without touching the array between. Four
mutants were caught by both: the carry of a carry dropped, the doubled product's sign taken after the doubling, the
quarter clamp fitting only a zero high third, and the array never brought up to date.

**What it was worth: almost nothing, and the reason is the finding.** Two interleaved rounds of the games gave five
per cent and then one, which is the noise. A benchmark of single operations (`rspbench`: fifteen of one instruction
and a jump, through the processor's own fetch and dispatch) showed why: before the change a multiply-accumulate
cost 7.3 nanoseconds and a bitwise AND 5.4. The arithmetic was a quarter of an operation; three quarters was
getting to it — the step, the first dispatch, the coprocessor's test, the vector function's entry, its three field
decodes, its element shuffle through a table, and its switch of sixty-four. `Mars_Performance.md` §38.1 had read
the vector function's fifteen per cent of the thread as arithmetic because its helpers were inlined into it; that
reading was wrong, and is retired there. The narrow accumulator is kept because it is exact, proven and no slower,
and because once the dispatch was removed (`Mars_Rsp.md` §12) the arithmetic became what is left.

**Two things found on the way, both kept.** The element selection was a shuffle with a mask from a table, and
selections zero and one are the register as it stands, so they skip it, and the rest use the native shuffle, whose
indices are all in range. And the quad load and store, the commonest vector memory operations, moved their sixteen
bytes one at a time through two helpers, 15 and 12 nanoseconds: when the whole register moves and the bytes do not
wrap, which is whenever the address is aligned, they are now one vector load, one exchange of each pair of bytes
and one store, 3.1 and 2.3. `A_quad_load_and_store_move_the_bytes_their_definition_moves_up_to_the_end_of_memory`
checks every address in the last forty-eight bytes of data memory at four elements against the definition written
out byte by byte. One mutant of it, the bound loosened by a byte, survives and is equivalent: a quad load moves
sixteen bytes only from an aligned address, and the last aligned address does not wrap.
