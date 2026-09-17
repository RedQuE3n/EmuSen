# Mars — the coprocessor register files, and the day the oracle started talking

*Phase B's first slice, landed 2026-09-16. It moves bits into and out of coprocessor
1 and never interprets them as numbers: no arithmetic, no rounding, no conversions.
`Cpu/Core/Fpu.cs`, `Cpu/Opcodes/Cpu.Opcodes.Cop1.cs` and `Cpu.Opcodes.Cop2.cs`, with
`MarsFpuRegisterFileTests`, `MarsFpuAccessTests` and `MarsCorpusTests` in
`EmuSen.WiseMan`.*

*§9 is the part worth reading first if you only read one section: with this slice in
place the hardware corpus begins printing verdicts, and it has opinions.*

---

## 1. Why the register file is a slice of its own

The boundary is "bits, not numbers". Everything here is decided by the instruction's
sub-opcode field alone; nothing needs the function field, which is where arithmetic
lives. That is not a tidy-looking split invented afterwards — it is the line the
decode itself draws, and it means this slice can be complete rather than partial.

It is also where the corpus was stopped. `Mars_Boot.md` §4 recorded that the first
instruction the ROM reached after its own bootcode was a coprocessor-1 control move,
configuring the FPU before anything else happened. Nothing downstream of that could
be graded until the control registers existed.

## 2. One register file, two shapes

`Status` bit 26 selects between them, and the difference is not cosmetic.

**Full mode (bit set)** is thirty-two independent 64-bit registers. An index names one
of them directly.

**Half mode (bit clear)** is sixteen 64-bit registers addressed as thirty-two 32-bit
halves. A **wide** access — `DMFC1`, `DMTC1`, `LDC1`, `SDC1` — drops the index's low
bit, so `$5` and `$4` are the same register and the odd one cannot be reached at all.
A **word** access — `MFC1`, `MTC1`, `LWC1`, `SWC1` — uses the low bit to pick a half,
with **odd meaning the upper half**.

**The trap that makes this worth a section.** A word write is a half write: it must
leave the other half of the register standing. An implementation that treats `MTC1` as
"store a 32-bit value, zero or sign-extend the rest" is wrong in *both* modes and
wrong silently, because nothing observes the discarded half until a program writes two
halves and reads back a double. Nine tests pin the mode behaviour, and the four that
die first under mutation are the ones asserting the surviving half.

**Provenance.** Every expected value in §2 is taken from the corpus's own hardware
assertions (`src/tests/cop1/full_vs_half_mode.rs`), which are measured on silicon.
That source is MIT-licensed and is the oracle rather than a reference emulator, so it
is read as a specification without the constraint `Mars_References.md` §2 imposes on
the GPL codebases.

## 3. Usability is answered before the decode

`Status` bit 29 says whether coprocessor 1 may be used at all. When it is clear,
**every** coprocessor-1 instruction raises Coprocessor Unusable — including encodings
that are reserved, which would otherwise raise a floating-point fault (§5), and
including the four load and store opcodes, which look like ordinary memory
instructions and are not.

The ordering is the claim: usability first, decode second. An implementation that
decodes first reports the wrong exception for a reserved word on a disabled
coprocessor, which is a difference no program notices until a handler does.

### 3.1 The Cause register names which coprocessor asked

Bits 28–29 of `Cause` carry the coprocessor number, and a handler has no other way to
learn it. Mars writes the field on **every** exception, with zero for every fault that
names no coprocessor — measured behaviour, not a convenience: the corpus deliberately
presets the field to 2 and then confirms an unrelated fault clears it back to 0.

Exceptions now carry the number on themselves rather than the raising site poking
`Cause` before throwing, so the field cannot be left stale by a path that forgets.

## 4. Two control registers, and the twelve bits that are not storage

`FCR0` reads `0x00000A00` and ignores every write. `FCR31` is the control and status
word: rounding mode, five sticky flags, five enables, five maskable causes plus the
unimplemented-operation cause, the condition bit and the flush-to-zero bit.

### 4.1 The writable mask

`0x0183FFFF`. Bits 18–22 and 25–31 read back as zero however they are written. Mars
masks on write rather than on read, so the stored word is always a legal one.

### 4.2 The starting value is zero, and that is all anyone will vouch for

`FCR31` is zero out of a hard reset and retains its previous contents across a soft
one. This is recorded because the corpus **commented out its own assertion** about the
value — its note says it has no fixed value — and an emulator that seeds a plausible
default instead of zero would be inventing a fact the oracle explicitly declines to
state. Mars leaves it zero and the HLE boot does not touch it.

### 4.3 The registers that do not exist

Only 0 and 31 are implemented. Mars returns zero for the rest. **This is an assumption
and not a measurement** — the corpus tests indices 0 and 31 and no others — and it is
recorded here so that a later disagreement is understood as filling a gap rather than
as a regression.

### 4.4 A write to the control word can raise the exception itself

If a write leaves a maskable cause bit set beside its matching enable bit, the
floating-point exception fires immediately, with the faulting address pointing at the
`CTC1` itself. This is how the corpus reaches the exception path without doing any
arithmetic, and Mars implements it in that order: **the write lands first, and the
control word keeps exactly what the program wrote.** It is not rewritten on the way
out, which distinguishes it from the operation-raised case in §5.

**~~Untested, and stated as such:~~ Corrected, 2026-09-16.** Whether setting the
*unimplemented* cause bit through `CTC1` also fires was recorded here as untested,
with Mars not firing on it, and the note said "if that is wrong, this paragraph is
where the correction goes". **It was wrong.** The corpus has a test named *Fire
unimplemented exception through CTC1*, which the run did not reach until the following
slice; it fires. The unimplemented cause has no enable of its own, and setting it by
any route — an operation or a hand-written control word — raises immediately.

## 5. Reserved here is a floating-point fault, not a reserved instruction

Twenty-one of the thirty-two coprocessor-1 sub-opcodes are reserved. On the integer
side a reserved encoding raises Reserved Instruction; **here it does not.** The unit
decodes the word, refuses it, and raises a floating-point exception with the
unimplemented-operation cause set.

The distinction matters because it says where the refusal happened: the CPU understood
the instruction well enough to hand it to the coprocessor, and the coprocessor is what
declined. The same applies to reserved function codes inside the arithmetic formats
and to the undefined branch conditions, neither of which this slice reaches yet.

On the way out the maskable causes are cleared and the unimplemented cause is set
alone, while the **sticky flags are left untouched** — the flags accumulate across
operations and an exception is not an operation that clears them. The unimplemented
cause has no enable bit and fires regardless of the enable word.

## 6. What stops the machine loudly, and why it is not an exception

Coprocessor-1 arithmetic, conversions and branches are real operations that Mars has
not built. They throw `NotImplementedException`, not a `CpuException`.

This is deliberate. A `CpuException` is *emulated hardware behaviour*; using one for
"not built yet" would make an unimplemented instruction indistinguishable from an
instruction the hardware genuinely refuses, and the difference is exactly what a
half-finished coprocessor makes easy to lose. The step loop does not catch it, so it
escapes to the caller and stops the run where it happened.

**This is scaffolding and it comes out when Phase B finishes.** While it stands, one
test asserts it by name so that the boundary is a stated position rather than a
surprise.

## 7. The four transfers

`LWC1`, `LDC1`, `SWC1` and `SDC1` use the same half-select rules as the moves (§2) and
the same alignment and translation path as the integer loads and stores, faulting
before anything is written. They are coprocessor instructions and are gated by §3,
which is easy to miss because their opcodes sit among the ordinary memory ones.

## 8. Coprocessor two, which is one latch

**Why it is in a coprocessor-1 page.** Because a coprocessor-1 test needs it: the
corpus's helper for the `CTC1` exception case (§4.4) first fires a *coprocessor-2*
unusable fault in order to preset the Cause field of §3.1. Without coprocessor 2 the
corpus cannot set up several of the tests that grade this slice. It is the same
mechanism, and separating it would have meant shipping a slice its own oracle could
not fully exercise.

The VR4300's coprocessor 2 is not a register file. **Every index names the same 64-bit
latch**: writing through `$5` and reading through `$6` or `$31` returns the same value.
`MTC2` and `DMTC2` both store the whole 64-bit source register; `MFC2` reads the low
half sign-extended and `DMFC2` reads all of it. `DCFC2` and `DCTC2` are reserved and
raise Reserved Instruction with the coprocessor field set to 2, which is the one place
Mars writes a non-zero coprocessor number for something that is not an unusable fault.

### 8.1 What is deliberately not modelled

- **`CFC2` and `CTC2`** do not raise when the coprocessor is usable, and what they
  read or write is measured nowhere. Mars reads zero and drops writes.
- **`LWC2`, `LDC2`, `SWC2`, `SDC2`** are left raising Reserved Instruction. The corpus
  has a test for them which its own author marks as incomplete and describes as
  "quite unclear how things actually work". Guessing at a behaviour whose measurement
  the oracle itself disclaims is how a core acquires a wrong answer that no test will
  ever challenge. It stays unimplemented until there is something to implement it
  *from*.

## 9. What the oracle said the first time it spoke

Before this slice the corpus executed 5,683 instructions and printed nothing. After
it, the ROM runs **4,651,016 instructions and emits about 50KB of text**, of which the
first lines are:

```
Heap range: ffffffff80219520 to ffffffff80700000
Running StartupTest...
Test 'StartupTest' failed: a == b expected, but a=0x6e463 b=0x7006e463. Initial COP0 Config
```

**521 tests started, 202 of them failed** — 174 after the slice that followed. That is the first graded measurement of
this core against real silicon, and it costs 413 milliseconds, which is why it is a
permanent test rather than an occasional exercise.

Two instructions had to be accepted along the way, four thousand instructions apart,
and both turned out to be decisions rather than omissions: `CACHE` and `SYNC`
(`Mars_Cpu.md` §13).

The failures group into four kinds:

- **Caches** — a large share. Mars models no caches and these cannot pass; the failure
  is the accurate report (`Mars_Cpu.md` §13).
- **Cartridge DMA edge cases** — misaligned and page-crossing transfers. Phase E.
- **COP0 registers** — `Config`, `TagLo`, `Wired`, `XContext` and the unused registers
  all have read-only or masked bits that Mars stores plainly. The very first failure
  is one of these.
- **Genuine CPU defects**, which is §9.2.

### 9.1 The tally is a ratchet

`MarsCorpusTests` asserts the exact pair — 521 started, and the failure count of the
day — rather than a floor. A floor was written first and **thrown away after it failed to catch two
deliberate mutations**: making `CTC1` a no-op, and giving the coprocessor-unusable
fault the wrong coprocessor number. Neither halts the ROM, so a "did it get far
enough" assertion sails past both; the failure count moved 202 → 203 and 202 → 206.

The number is deterministic across runs. It is meant to be edited **downward** as
defects are fixed, and any upward movement is a regression.

### 9.2 Two integer defects found here and deliberately not fixed here

**`SRA` and `SRAV` shift the full 64-bit register, not the low 32 bits.** Mars takes
the low word, shifts it as a signed 32-bit value and sign-extends the result.
Hardware arithmetic-shifts the whole 64-bit register, takes the low 32 bits of *that*,
and sign-extends. For `SRA $rd, 0x0123456789ABCDEF, 4` hardware gives `0x789ABCDE`
where Mars gives `0xFFFFFFFFF89ABCDE`. `SRL` is not affected and its tests pass, so
this is a property of the arithmetic shift and not of 32-bit shifts generally.

**`MULT` does not use only the low 32 bits of its operands either, and is not
commutative.** `MULT(0xEEFFFFFFFF, 0x10)` and `MULT(0x10, 0xEEFFFFFFFF)` produce
*different* results on hardware. The full rule has **not** been determined here: two
candidate readings were tested against the reported pairs and both failed on the
swapped case.

**They are recorded rather than repaired because they are the same question.** Fixing
`SRA` on two data points while `MULT` remains unexplained would risk exactly the
failure `Mars_Cpu.md` §7.1 already records — an implementation that satisfies the
evidence in front of it and is uniformly wrong. The next slice reads the corpus's
arithmetic tests directly and settles the 64-bit operand question for all three
together.

> **Settled, 2026-09-16, in the slice that followed.** `Mars_Cpu.md` §14. The caution
> was warranted: neither guess attempted here was right, and the actual rule —
> `MULT` reads *thirty-five* bits of its second operand — is one no amount of fitting
> to the reported pairs would have produced. The tally moved 202 → 174.

## 10. What this slice does not do

- **No arithmetic, no conversions, no compares, no branches.** §6 is what happens when
  one is reached.
- **No software floating point yet.** The decision that Phase B starts soft rather than
  on host `double` (`Mars_Gameplan.md` §4.2, and two independent arguments in
  `Mars_References.md` §5.2 and `Mars_Documentation.md` §2.1) is untouched by this
  slice, which stores bit patterns and never reads them as values.
- **No cycle costs.** Coprocessor transfers charge one cycle like everything else
  (`Mars_Cpu.md` §5).
