# Mars — floating point in software, and what the part refuses to compute

*Phase B's body, landed 2026-09-16. Arithmetic, square root, conversions, compares and
the coprocessor branches, on a software float rather than the host's.
`Cpu/Fpu/SoftFloat.cs`, `SoftFloatMath.cs`, `SoftFloatConvert.cs` and
`Cpu.Opcodes.Cop1Math.cs`, with `MarsFpuArithmeticTests` and the corpus.*

*The plan (`Mars_Gameplan.md` §4.2) required this start soft rather than on C#
`double`, on two arguments neither of which had been measured. §1 reports the
measurement. Both arguments hold, and the second is stronger than it was stated.*

---

## 1. The reversed NaN convention, confirmed and made precise

`Mars_Documentation.md` §2.1 recorded, marked **[community]**, that the VR4300's
quiet and signalling NaN patterns are the reverse of the modern convention. **It is
correct, and it is now [read] rather than [community]** — the corpus asserts it.

The confusing part, and the reason this needs stating carefully: the corpus's own
constants use the *modern* names. `QUIET_NAN_START_32` is `0x7FC00000`, the mantissa's
top bit set, exactly as IEEE 754-2008 defines a quiet NaN. What the corpus then
asserts about that pattern is the opposite of what the name implies, and it says so in
a comment of its own — *"which is the opposite of what their name implies"*.

Measured behaviour, for an arithmetic operand:

| Mantissa top bit | Modern name | What this part does |
|---|---|---|
| Set | quiet | **Raises Invalid Operation** and returns the default NaN |
| Clear | signalling | **Raises Unimplemented Operation** — it will not compute at all |

And the NaN it produces is **`0x7FBFFFFF`** (`0x7FF7FFFFFFFFFFFF` in double) — a
pattern a present-day host FPU calls *signalling*. A C# `double` would have produced
`0x7FF8000000000000` and been wrong in every NaN-returning operation, silently.

**Mars therefore names these classes for what they do**, not for what IEEE calls them:
`Nan` is the one that raises Invalid, `NanUnsupported` the one that is refused. Using
the standard names here would have imported the confusion permanently.

## 2. The representation

A number is unpacked into a sign, an unbiased exponent, and a 64-bit significand with
its leading one at bit 63. Every operation produces a 128-bit significand with the
leading one at bit 127, plus a sticky flag, and **rounds exactly once** (§4). The two
formats differ only in three constants.

The 128-bit intermediate is what makes the arithmetic straightforward rather than
clever: a double's 53×53 product is 106 bits and fits exactly, so multiplication never
needs a sticky bit at all.

## 3. What the part will not compute with

Two operand shapes are refused outright, with the unmaskable unimplemented-operation
exception, before any arithmetic happens:

- **A denormal**, either operand, always.
- **A NaN with its mantissa top bit clear** (§1).

**The denormal rule is the one that makes this tractable.** The VR4300 does not
compute with denormals; it declines. So the whole of gradual underflow — the part of
IEEE arithmetic that costs the most to implement and is hardest to get right — is not
needed. Mars's software float handles normals, zeros, infinities and NaNs, and
nothing else.

This is not a simplification Mars chose. It is what 32 separate corpus vectors assert
for `ADD.S` alone.

### 3.1 The result when there is no result

Invalid Operation returns the format's default NaN (§1). This is one value, not a
propagation of the operand's payload: an operand NaN's bits do not survive.

## 4. Rounding happens once, in one place

One function consults the rounding mode and one function checks the exponent range.
Round-to-nearest ties to even; the three directed modes increment when the discarded
part is non-zero and the sign agrees with the direction.

**Overflow is not always infinity.** Round-to-nearest gives infinity, round-to-zero
gives the largest finite value, and the directed modes give one or the other depending
on the sign. Getting this wrong produces a value that is merely enormous rather than
infinite, which no test of the common path would catch.

### 4.1 Underflow, which settles a question the survey left open

`Mars_Documentation.md` §2.1 recorded that rounding on underflow is **"disputed
between sources"** and "should be settled by test rather than by reading". It is
settled. With flush-to-zero set:

| Mode | A tiny negative result becomes |
|---|---|
| Nearest | `-0` |
| Zero | `-0` |
| Toward +∞ | `-0` |
| Toward −∞ | **`-1.1754944e-38`**, the smallest normal |

So the directed modes are taken **literally** even below the smallest normal: rounding
down a small negative number moves it away from zero, to the minimum normal. The
survey's report, which it marked unresolved, was right.

**And then there is this.** If either the underflow or the inexact exception is
*enabled*, the same operation raises **unimplemented operation** instead of underflow.
The corpus's own comment on the vectors reads *"works if exceptions are off, but
unimplemented if they are enabled (wow)"*. Mars implements it because it is measured,
not because it can be explained.

With flush-to-zero clear, Mars refuses. That combination is not exercised by the
corpus, which runs with flushing on, and the refusal is a guess.

## 5. How a result reaches the control word

The cause field is **replaced** by what this operation raised; the flag field
**accumulates**. If a raised cause has its enable set, the exception fires and the
destination register is not written.

### 5.1 Three operations that look like bit twiddling and are not

`ABS` and `NEG` clear or flip the sign bit — after classifying the operand exactly as
arithmetic does. A denormal makes `ABS.S` raise unimplemented; a NaN makes it raise
Invalid and return the default NaN rather than the operand with its sign cleared.
Implementing them as the one-line bit operations they appear to be fails 64 corpus
vectors.

`MOV` is the opposite: it classifies nothing, raises nothing, and copies the **whole
64-bit register** regardless of the format in its name.

### 5.2 A computed 32-bit result clears the rest of its register

`ADD.S` into a register leaves the upper 32 bits **zero**. `MTC1` and `LWC1` into the
same register leave them **untouched** (`Mars_Fpu.md` §2). Same register file, same
width, opposite rule, and nothing observes the difference until a program writes a
single and reads a double.

### 5.3 Half mode masks one index and not the others

In half mode (`Mars_Fpu.md` §2), coprocessor-1 arithmetic **ignores the low bit of the
first source index** and uses the low bit of the second source and of the destination
normally. The corpus states both halves as separate assertions — *"Lowest bit of fs
should be ignored"* and *"Lowest bit of ft should not be ignored"*.

This is not the half-select rule the moves use. A single-precision operand is the low
32 bits of the physical register named, not a half chosen by the index's low bit.

## 6. Conversions

Between the formats, and between floats and integers, through the same rounding
function. `ROUND`, `TRUNC`, `CEIL` and `FLOOR` are the four rounding modes named
explicitly rather than taken from the control word; `CVT` uses the control word.

### 6.1 Float to integer refuses rather than saturates

Infinity, either NaN, a denormal, or a value outside the destination's range all raise
unimplemented operation. There is no saturation and no wrapping.

### 6.2 Integer to float has a range too, and it is not the obvious one

`CVT.S.L` and `CVT.D.L` refuse a source outside **[−2⁵⁵, 2⁵⁵)**. Not 2⁶³, which is
what the source type would suggest, and not 2⁵³, which is what the single format's
precision would suggest. 2⁵⁵ is measured; no reading predicted it.

### 6.3 The 64-bit conversions stop at the double's mantissa

`CVT.L`, `ROUND.L`, `TRUNC.L`, `CEIL.L` and `FLOOR.L` refuse any source whose magnitude
is **2⁵³ or greater**, and raise unimplemented operation instead. Not 2⁶³, which is the
destination's range; not the source's range either. 9007199254740991 converts, and
9007199254740992 does not.

**The limit belongs to the unit, not to the format.** It is the same 2⁵³ from a single
source as from a double — `S → L` refuses at the same boundary, even though a single
carries only 24 bits of mantissa and reached that magnitude by scaling. So it is not
"the source cannot represent integers beyond here"; it is a fixed ceiling the conversion
hardware applies regardless of where the value came from. That makes it the third
distinct range rule in this section, and none of the three is the one the type
signatures suggest.

The 32-bit family keeps its own, smaller limit, which *is* the destination's range
(§6.1), and applies it after rounding — a source of 2147483647.6 converts under one
rounding mode and refuses under another, because rounding is what carries it across.
Whether the 2⁵³ ceiling is likewise applied after rounding is **not** established: every
source at that magnitude is already an exact integer, so no vector in the corpus can
separate the two. Mars applies it to the source exponent, before rounding, and that
choice is untested rather than measured.

## 7. Compares follow different rules from arithmetic

`Mars_Documentation.md` §2.1 recorded that the vendor manual's main text *excludes
compare instructions* from the denormal and NaN rules. Confirmed, and the difference
is total:

- **A denormal is compared by value.** No exception, no refusal — the number is read
  as the denormal it is. Mars's unpacking normalises subnormals for exactly this path
  while arithmetic still refuses them (§3).
- **The mantissa-top-bit-clear NaN raises nothing** in an unordered compare, where
  arithmetic refuses it.
- **The mantissa-top-bit-set NaN raises Invalid**, even for the conditions whose names
  say they do not signal.
- The conditions with bit 3 set raise Invalid for **either** NaN.

The condition's low three bits select which of less-than, equal and unordered make it
true, which is ordinary MIPS. Everything above it is not.

### 7.1 The branches

Four encodings — on the condition true or false, each with a likely form. The rest of
the sub-opcode's space is reserved and raises unimplemented. They use the same
delay-slot machinery as the integer branches, including `Mars_Cop0.md` §9's rule that
an untaken ordinary branch still has a delay slot.

## 8. Square root, and a bug worth recording

Digit by digit on a 128-bit radicand, which gives an exact remainder and so an exact
sticky bit. The exponent is forced even before the root is taken.

**The first version was wrong and passed its first tests.** The digit trial value is
`4R + 1`, not `2R + 1` — the new digit contributes `d(4R + d)` — and with `2R + 1` the
square roots of exact powers of four come out **right**. `sqrt(16) = 4` and
`sqrt(1) = 1` both passed while `sqrt(2)` returned 1.65. A test suite of tidy values
would have shipped it.

## 9. What the oracle said when this slice landed

**561 tests started, 138 failed**, and every failure was a cache test or a cartridge
DMA test — the two groups Mars does not model and Phase E owns. Every COP1 test the
run reached passed: the register file, the moves, all four arithmetic operations in
both precisions, square root, the conversions, all sixteen compares, and the branches.

**The scaffold is gone.** `Mars_Fpu.md` §6 introduced a `NotImplementedException` to
stop the machine loudly at an instruction Mars had not built, and a test asserting
where that happened. There is nothing left for it to catch, and the test now asserts
the inverse: that the corpus reaches no such instruction.

The current state of the run is no longer an FPU topic and lives in `Mars_Corpus.md`.

### 9.1 Where the run was cut, and a prediction that was wrong

*Retired 2026-09-16. Kept because the reasoning was sound and the conclusion was not.*

This section read, in full:

> The run is truncated at its instruction budget **inside the 64-bit conversion tests**,
> and that is deliberate: with a budget six times larger the run reaches an exception
> storm in that same test and aborts, which means something in `CVT.L`/`ROUND.L`/
> `TRUNC.L`/`CEIL.L`/`FLOOR.L` is wrong in a way the corpus's handler cannot recover
> from. The ordinary cases are right — 4.5 → 4, 5.5 → 6, 4.4 → 5 rounding up — so it is
> an edge case rather than the mechanism.

Every observation in it was correct. The inference — that the storm was *caused by* a
defect in those five instructions — was not. The storm was `TEQ`, which Mars had never
implemented, reached through the corpus's own divide-by-zero guard while it formatted
the conversion failures into messages (`Mars_Cpu.md` §15.2).

There *was* a conversion defect, and the corpus states it outright: the 2⁵³ ceiling in
§6.3. It accounts for seven of the failures and none of the storm. Finding it required
reading the corpus's expectation tables, which is what should have happened first;
predicting its shape from where the run stopped produced a plausible sentence and no
progress.

**The general form, since this is the second time on this page.** §8 records a square
root bug that every tidy test value passed. This records a diagnosis that every
available observation supported. Both have the same cause: a small, self-consistent
body of evidence will confirm a wrong answer as readily as a right one, and the way out
is a measurement that could distinguish them — here, one instrumented run that printed
the offending instruction word.

## 10. Two commercial cartridges

Super Mario 64 and Wave Race 64 both run **twenty million instructions without a
single fault**. Neither is playable and neither is meant to be: both finish booting and
settle into a loop waiting for a video interrupt that no part of Mars can raise yet.
Mario's loop tightens as it gives up on more of the machine — thirty-two instructions
after two million, and two instructions (`BEQ $0, $0, -1` and its delay slot) by
twenty million. Wave Race spins in an operating-system critical section, reading and
writing `Status` around a message-queue wait.

**What this is evidence of, and what it is not.** It is evidence that the integer
core, COP0, the TLB, exceptions and the FPU execute real commercial code — hundreds of
thousands of distinct instruction paths written by people who never saw this emulator
— without hitting a defect that faults. It is **no** evidence about the RCP, about
timing, or about anything either game draws, because neither has drawn anything.

Two tests keep the position, skipping when the cartridges are absent: neither faults
with anything but a timer interrupt, and Mario reaches a small wait loop. The
cartridges are not in the repository and never will be.

> **Update 2026-09-17: the video interrupt is now measured, not inferred.** "Waiting for a
> video interrupt" above was read off the shape of the loop. Raising that interrupt from the
> test harness starts both games' audio microcode, and raising the serial interrupt as well
> gets Wave Race to its first graphics task (`Mars_Microcode.md` §2). Both games had unmasked
> the video interrupt and programmed its line before settling, so the inference was right.

## 11. Single-precision add, subtract and multiply on the host, where its answer is provably the unit's

*2026-09-20.* The software unit is exact by construction and costs what that costs: an interrupt sample of Ocarina
of Time in play put it at 8.5 per cent of the emulation thread, a multiply, an add and the rounding they share
most of it, and the census put the three single-precision operations at a fifth of what still reaches the
interpreter (`Mars_Recompiler.md` §18). `HostSingle.TryCompute` answers those three on the host **only where the
host's answer can be proved to be the unit's**, and declines everything else.

**The argument.** A single has twenty-four significant bits. The product of two is at most forty-eight, which a
double holds whole, so `(double)a * (double)b` is the exact product; the sum or difference of two singles whose
exponents differ by no more than twenty-eight is likewise whole in a double's fifty-three. The host's conversion of
that exact double to a single rounds to nearest, ties to even, which is the unit's rounding in the only mode the
path accepts, and because the double was exact there is one rounding and not two. The result is inexact exactly
when converting it back does not give the double. So for such operands the bits and the inexact flag are the
unit's, and no other flag can arise.

**What is declined**, and left to the unit with its flags and its refusals: any operand that is not a normal
number — a zero, a denormal, an infinity, a NaN — since zeros have sign rules and the rest raise or are refused;
any rounding mode but nearest; a sum whose exponents are more than twenty-eight apart; an exact result smaller
than the smallest normal number, which covers tininess by either definition and the zero of a cancelling sum; and a
result that rounds to infinity. Double precision is not attempted: its products are not whole in anything the
host offers.

**Delivered the same way.** The host's bits and flag go to `Deliver` as the unit's would: the cause bits replaced,
the enables consulted, the sticky flags added, the register written. Nothing about the status register is decided
twice.

**The proof.** `MarsHostSingleTests` compares `TryCompute` with the unit (`HostSingle.Reference`) for each
operation over 1.5 million random bit patterns, every exponent against every exponent within thirty-two of it with
the mantissas that sit on rounding boundaries and random ones in all four sign pairs, and 400,000 products aimed at
both ends of the range, and insists that the host answered more than a million cases, found more than 100,000
inexact and declined more than 100,000. Four mutants of the guards were all caught: sums allowed past the
exponents a double holds whole, tiny results not declined, overflow not declined, the inexact flag never raised.
The state hash after 1,200 frames is the same with the path off and on in Ocarina of Time, Super Mario 64 and
GoldenEye, and the probe's two games are identical to their baselines. `EMUSEN_MARS_NOHOSTFLOAT=1` turns it off.

**What it was worth** is `Mars_Recompiler.md` §18: about three per cent of Ocarina of Time's frame, the game that
leans on the unit most. Division, the square root, the conversions and the compares are still the unit's.
