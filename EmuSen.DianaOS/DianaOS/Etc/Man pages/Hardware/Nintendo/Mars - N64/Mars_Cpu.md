# Mars — the VR4300's integer core

*Phase A's second slice, landed 2026-09-15. The instruction set begins here.
`EmuSen/Cores/Nintendo/Mars - N64/Cpu/`, tested by `MarsCpuTests` through the
`MipsAssembler` fixture. Two decisions in this page were taken deliberately in
advance rather than discovered: §4 and §5.*

---

## 1. Shape

A `switch` on the opcode field, with sub-switches for the instruction groups that
re-use it, and the opcode families in partial-class files beside it. That is what
Moon and Mercury do, it compiles to jump tables, and it reads in the order the
manual is written in.

Two alternatives were considered and rejected for now. A delegate table is more
uniform and costs an indirection this project has already measured and rejected the
equivalent of on Venus. A predecoded instruction cache is the fast option and the
wrong one at this stage: the N64 moves code into RDRAM and into the signal
processor's banks by DMA constantly, so a decoded cache needs invalidation, and
speed is Phase G's business (`Mars_Gameplan.md` §2.2).

## 2. Registers are 64-bit, always

Thirty-two general registers of 64 bits, with 32-bit operations sign-extending their
results into the full width — which is the whole of what MIPS III means. A shift or
an add on a register holding a 64-bit value operates on its low half and writes back
a sign-extended 32-bit answer.

**This is where Project64 took a shortcut and paid for it in its database.** `32bit`
is a per-game knob there, one of the columns counted in `Mars_References.md` §5.1.
Refusing that shortcut is the LLE decision restated at instruction level: a core that
models the registers the hardware has does not need to be told which games notice.

Register zero is wired to zero, and a write to it is dropped rather than stored.

## 3. Two program counters

`Pc` and `NextPc`. A step fetches at `Pc`, advances `Pc` to `NextPc` and `NextPc`
past it, and only then executes — so a branch writes `NextPc` and the instruction
already fetched behind it runs as its delay slot without any special case.

`InDelaySlot` records that the instruction being executed follows a taken branch,
which is what an exception needs in order to report the branch's address rather than
the slot's.

### 3.1 A likely branch not taken discards its slot

The branch-likely family nullifies the delay slot when the branch is not taken, which
in this model is one line: advance `Pc` past the slot before it is reached. Both
behaviours are tested against each other — the ordinary form runs its slot when
untaken, the likely form does not.

The linking branches have a separate trap worth naming: **the link happens whether or
not the branch is taken.** A test asserts it, because the natural reading of the
mnemonic is the opposite.

## 4. Exceptions are thrown

A single preallocated exception object, thrown where the fault is found, caught once
at the step boundary. The decision was taken before the code was written, against the
alternative of a pending-exception flag every access site checks.

**The argument is not brevity, it is partial state.** A load that faults must not
write its destination register; an overflowing add must not write its result.
Throwing before the write makes that true by construction rather than by every
opcode remembering to return early. Two tests assert exactly this — a trapping add
leaves its destination untouched, and a misaligned load leaves an earlier value in
place.

The cost is unwind overhead on a path the hardware corpus exercises deliberately and
hard. That is accepted for now and is a Phase G measurement rather than a Phase A
belief.

### 4.1 Raised, and now delivered

Vectoring landed in the slice after this one; §11 is what an exception does to the
machine. `LastException` survives as a diagnostic rather than a control signal — it
records the most recent raise and is never cleared, and execution continues into the
handler rather than stopping, which is what let the round-trip test in §11.3 exist.

## 5. Cycles: documented numbers only

One cycle per instruction, plus the multiply and divide stall counts the vendor
manual tabulates — 5, 8, 37 and 69 for the four forms. Nothing else is charged,
because nothing else is documented: memory latency, DMA duration, cache misses and
bus arbitration are absent from every source the survey found
(`Mars_Documentation.md` §5).

So `Count` derives from a clock that is partly principled and mostly a placeholder,
and the page says which is which rather than letting a plausible-looking number
imply a model. The corpus's timing category is what will replace the placeholder
half.

## 6. One way in to memory

Every access goes through one translate-then-read pair, so the TLB and later the
caches cannot be bypassed by an opcode that reaches the bus directly. Today the
translation handles the two direct-mapped segments and raises a TLB exception for
everything else, which is the correct behaviour for a machine whose TLB is empty.

Alignment is checked before translation, and the check raises before any access
happens (§4).

## 7. The merging loads and stores

`LWL`/`LWR`, `LDL`/`LDR` and their four store counterparts, added in their own slice
because `Mars_Gameplan.md` §5 names them as a classic source of subtly wrong values
— the kind that survive casual review because the common case looks right.

Each accesses the *aligned* word or doubleword containing its address, and merges:
a left load takes the addressed byte and everything above it into the high end of
the register and keeps the register's remaining low bytes; a right load does the
reverse. The stores do the same to memory. Used in pairs they read or write a value
that straddles an alignment boundary, which is the idiom they exist for and which
has its own test at both widths.

**These addresses are legal.** An ordinary load at the same address faults (§6);
these do not, and a test asserts it.

### 7.1 A near-miss worth recording: the corpus's tables are reverse-endian

The hardware corpus contains expected-value tables for exactly these instructions,
which is precisely the oracle this project prefers to prose. They are in its
**reverse-endian** test, where the byte roles are mirrored — its left-load table at
offset zero shows a single byte loaded, which is the opposite of what the
architecture specifies in normal mode.

Lifting them would have produced an implementation that was confidently, uniformly
wrong, and every test would have agreed with it. The expectations here are derived by
hand from the architecture's definition instead, and the corpus's own normal-mode
verdict is what will eventually confirm or refute them.

The general form of this is worth keeping: **an oracle's answer is only an answer to
the question it was asking.** The tables were real, hardware-derived and correct —
for a mode we were not in.

### 7.2 Two of these tests were wrong before they were right

The first run failed one case, and the fault was the test: `LUI` sign-extends, so a
register preloaded with it holds `0xFFFFFFFF` in its upper half rather than the
pattern byte, and a right-double-load keeps exactly that half. The expectation had
assumed a pattern that the setup never put there. Fixed by preloading the register
through a real doubleword load, which is also a better test — the kept bytes are now
visible as the pattern at both ends.

Removing the merge from the left load reddens three of its four cases, so the
merging is pinned rather than incidental.

### 7.3 The corpus's normal-mode verdict, and the rule the tables could not see

§7.1 ends by saying the hand-derived expectations would eventually be confirmed or
refuted by the corpus's own normal-mode tests. Those tests now run, and they refute one:

**`LWR` does not sign-extend a partial merge; `LWL` always does.** Both instructions
produce a 32-bit value and write it to a 64-bit register, and the obvious reading — the
one Mars implemented — is that the same rule governs both, because everything else that
computes a 32-bit result on this part sign-extends bit 31 into the upper half. It does
not. Where `LWR` takes fewer than four bytes, the register's upper 32 bits are left
exactly as they were; only a `LWR` that happens to take a whole word sign-extends.
`LWL` sign-extends in all four cases, including the three that merge.

The corpus's vectors start from `0xFEDCBA98_76543210` and read a doubleword of
`0x0123456789ABCDEF`, so the two behaviours are separable:

| offset | `LWL` gives | `LWR` gives |
| --- | --- | --- |
| 0 | `0x00000000_01234567` | `0xFEDCBA98_76543201` |
| 1 | `0x00000000_23456710` | `0xFEDCBA98_76540123` |
| 2 | `0x00000000_45673210` | `0xFEDCBA98_76012345` |
| 3 | `0x00000000_67543210` | `0x00000000_01234567` |

**Why the existing tests could not have caught it.** §7.2's fix preloads the register
through `LUI`, which sign-extends, so the register's upper half was already `0xFFFFFFFF`
— indistinguishable from the sign extension of every merged value the test produced.
The tests were not wrong; they were *blind*, in a way that only a register whose upper
half is neither zero nor all ones can expose. The corpus's vectors are chosen to have
exactly that property, which is the difference between a table that exercises an
instruction and one that discriminates between two implementations of it.

Both tables are now in `MarsCpuUnalignedTests`, and the four hand-derived rows from §7.1
stay beside them: they were right, and keeping them records that the reverse-endian
trap was avoided rather than merely survived.

## 8. What is not implemented yet

- ~~The TLB~~ — landed, `Mars_Tlb.md`. Three of five segments now translate, and the
  refill-versus-invalid distinction §11.1 could not previously express is carried on
  the exception itself.
- ~~**COP1**, which is Phase B.~~ — landed, `Mars_Fpu.md` and `Mars_FpuMath.md`.
- **Software interrupts**, the two bits a program raises itself, are storage with
  nothing behind them.
- **Every interrupt source except the counter and the peripheral interface.** The
  aggregator has six inputs and two are wired (§12).
- ~~**Supervisor and user mode.**~~ — landed, `Mars_Privilege.md`.
- ~~**Reverse-endian addressing.**~~ — landed, `Mars_ReverseEndian.md`.

**This list is not a census.** The conditional traps were missing for nine slices and
are not in it (§15), because an omission nobody has noticed cannot appear on a list of
noticed omissions. What the corpus reaches is the census; this is the part of it
written down in advance.

## 9. Coprocessor zero, minimally

Enough to move values in and out, with `Count` read from the machine clock rather
than from storage — the register file is otherwise plain words. `Compare` is stored
and nothing compares against it yet.

**The one behaviour here that is not a placeholder** is the emulator-extension opcode
range: coprocessor-zero operations with a function code at or above `0x20` are
ignored rather than refused. The hardware corpus calls one of them *unconditionally*
in its first instructions, so refusing them traps before it can print anything
(`Mars_TestOracle.md` §2.3). A test asserts both halves — that range ignored, and the
range below it still refused, so this is not a blanket amnesty.

## 10. Tests read as assembly

`MipsAssembler` emits instruction words, so a test is a short program rather than a
table of hex. A program is assembled, loaded, and run for a fixed number of steps.

**Two of the first tests written were wrong, and the implementation was right.** One
expected a shift to discard a value that MIPS sign-extends back in; the other
asserted that writing `Count` and reading it two instructions later returns the
written value, which would mean the clock had stopped. Both were corrected against
the instruction semantics rather than by adjusting the core, and they are recorded
here because "the test was wrong" is the outcome this project's tests exist to make
visible.

## 11. What an exception does

In order: save the return address, record the cause, raise the exception level, and
jump to a handler.

The return address is **the faulting instruction itself**, not the instruction after
it — a handler that fixes the cause and returns must re-run what failed. A fault in a
delay slot saves the *branch* instead and sets a flag saying so, because returning to
a delay slot alone would execute it without the jump that gave it meaning. Both have
tests.

This is where `CurrentPc` earns its place. The step loop advances `Pc` past the
instruction before executing it, so that branches can compute targets relative to
the delay slot — which means `Pc` is the wrong answer for "where did this fault
happen". `CurrentPc` is the right one, and an exception that used `Pc` would return
one instruction too far, every time, invisibly.

### 11.1 Two doors

A TLB refill arriving from ordinary execution gets its own vector; everything else,
including a refill that happens *while already handling an exception*, uses the
general one. That distinction is the hardware's way of making the common case — a
page that simply is not mapped yet — cheap, while keeping nested faults survivable.

A second flag in `Status` moves both vectors into the boot address space, which is
what the machine uses before RAM is trustworthy. All three paths are tested.

Mars has no TLB, so every mapped-segment access faults as a refill. That is the
correct behaviour for a machine whose TLB is empty rather than a placeholder.

### 11.2 A fault inside a handler keeps the first return address

The saved address and the delay-slot flag are written only when the exception level
was clear. Overwriting them would destroy the outer handler's way home, which is the
difference between a nested fault being survivable and being fatal.

### 11.3 The round trip

The return instruction restores the program counter from the saved address and drops
the exception level, with no delay slot of its own. A test runs the whole circuit: a
system call faults, a handler at the vector records that it ran, steps the saved
address past the faulting instruction, returns — and the instruction after the fault
then executes normally.

## 12. Interrupts

Two sources reach this core today. The RCP's aggregator drives one line and is
**level-triggered** — read afresh every step, so clearing the device that raised it
lowers the CPU's line with no further action. The counter drives the other and is
**latched** — once raised it stays raised until a handler writes the comparison
value, which is how the hardware makes acknowledgement explicit.

Three gates stand between a raised line and an exception: the global enable bit, the
per-line mask, and the exception level. All three have a test, and the third is the
one that matters most — without it a handler would be interrupted by the very line it
was entered to service, forever.

The check happens **before the instruction is fetched**, so the saved address is the
instruction that has not run yet rather than one that half did. An interrupt arriving
when the next instruction is a delay slot saves the branch, the same way a
synchronous fault there does (§11).

### 12.1 Hardware compares for equality; this clock cannot

The real counter increments by one at a fixed rate and raises its line on the cycle
where it **equals** the comparison value. Mars charges documented stall counts (§5),
so a single instruction can advance the counter by more than one — a multiply moves it
three at once — and an equality test would step straight over the comparison value and
never fire.

So the question asked here is *did the comparison value fall inside the interval this
instruction covered*, with the wrap handled as two ranges rather than one. Replacing
it with the equality test hardware performs reddens exactly the test written for it.

**What this costs, stated rather than discovered:** the interrupt is raised at the end
of the instruction that crossed the value, not at the cycle that reached it. A long
instruction can therefore delay it — up to sixty-nine cycles for the worst divide —
and nothing is checked mid-instruction. That is a real divergence from hardware and it
is a consequence of the cycle model rather than of this mechanism; when §5's
placeholder half is replaced by measured costs, this inherits the improvement.

The rebase on a write matters for the same reason. Writing either the count or the
comparison value resets the interval's starting point, so a value that the counter has
already passed does not fire immediately — it waits for the wrap, which is what
hardware does.

## 13. Two instructions that are already finished when they are issued

`CACHE` and `SYNC` are accepted and do nothing. Both were added because the corpus
executes them before it prints a word, and both are recorded here rather than in a
code comment because "does nothing" is a claim about the machine, not about the code.

**Neither is a stub.** `CACHE` operates on caches, and Mars models none, so every
cache line is already in the state the instruction asks for. `SYNC` orders memory
accesses against each other, and Mars executes strictly in order through a single bus
with no write buffer and no reordering, so the ordering it demands is the only
ordering available. In both cases the instruction's postcondition holds on entry.

**What this does not cover.** The corpus's cache sections cannot pass against a core
with no caches, and they should not — the failure is the accurate report. `CACHE`
being a no-op is what lets the *rest* of the corpus run; it is not a claim that cache
behaviour is emulated. If cache emulation is ever built, this section is the thing to
delete rather than to amend.

**Where they were found.** Not from a manual read in advance: the real ROM stopped on
each in turn, four thousand instructions apart, and each stop was diagnosed from the
faulting word. That is the pattern this phase has repeated — the corpus finds the
omission, and the omission turns out to be a decision rather than an oversight.

## 14. Two operations that do not read the operand width they appear to

*Added 2026-09-16, after the hardware corpus began reporting (`Mars_Fpu.md` §9). Both
of these were written from the instruction set's shape — "a 32-bit instruction reads
32-bit operands" — and both are wrong. Neither would have been found by reading, and
neither would ever have been found by a test this project wrote for itself, because
the wrong answer is the one an implementer expects.*

### 14.1 SRA and SRAV shift the whole register

`SRA` shifts the **full 64-bit register** arithmetically, takes the low 32 bits of
that result, and sign-extends those into the destination. It does not truncate first.

For `SRA $rd, 0x0123456789ABCDEF, 4` hardware produces `0x00000000789ABCDE`. Truncating
first gives `0xFFFFFFFFF89ABCDE` — a different value *and a different sign*, which is
the kind of divergence a game accumulates rather than crashes on.

`SRL` is unaffected and its tests passed throughout, so this is a property of the
arithmetic shift and not a general rule about 32-bit shifts. `SRAV` behaves
identically, with the count still taken from the low five bits of `rs`.

**A note on what the four hardware vectors can and cannot settle.** For any shift
amount in 0–31, the low 32 bits of a 64-bit shift are the same whether the shift is
arithmetic or logical — the bits duplicated from the sign only ever land in the upper
half. So the measurements pin *that the whole register is read* and say nothing about
which 64-bit shift is performed. Mars uses the arithmetic one; if that is ever shown
to matter, no test here would have caught the difference.

### 14.2 MULT reads thirty-five bits of its second operand

Not thirty-two, and not sixty-four. `MULT` takes `rs` as a full 64-bit signed value
and `rt` as a **35-bit signed value** — bits 34 down to 0, with bit 34 as the sign —
multiplies them into 64 bits, then splits that into `LO` (bits 31–0) and `HI` (bits
63–32), sign-extending each half.

**The consequence is that `MULT` is not commutative.** `MULT(0xEEFFFFFFFF, 0x10)` and
`MULT(0x10, 0xEEFFFFFFFF)` give different answers on silicon, because only the second
operand is narrowed. That is the observation that made the earlier guesses fail: any
rule symmetric in its operands is refuted by a single swapped pair.

Three vectors make the width unmistakable, all with `rs = 1`:

| `rt` | `HI:LO` | why |
|---|---|---|
| `0x0000000200000000` | `2 : 0` | 2³³ fits in 35 bits, positive |
| `0x0000000400000000` | `-4 : 0` | 2³⁴ **is** the sign bit, so it reads negative |
| `0x0000000800000000` | `0 : 0` | 2³⁵ is outside the field and truncates to nothing |

A 32-bit reading gives zero for all three; a 64-bit reading gives a positive result
for all three. Only the 35-bit reading produces the middle row.

`MULTU` is different again and was already right: it truncates **both** operands to 32
bits unsigned. `DMULT` and `DMULTU` take both as full 64-bit values. The four
instructions therefore use three different operand widths between them, which is why
"the multiply family" is not a thing that can be implemented once.

### 14.3 How they were found, and why the rule came from a comment

Neither was found by inspection. The corpus began reporting verdicts in the slice
before this one, and named both in its output with the expected and actual values
beside each other.

The `MULT` rule itself was **not** derived from those failures. Two candidate readings
were fitted to the reported pairs and both survived several vectors before failing on
a swapped one — the exact shape of a wrong answer that looks confirmed.
`Mars_Fpu.md` §9.2 records the decision to stop guessing and go and read, and the rule
turned out to be stated outright in a comment at the top of the corpus's own multiply
test. It was then checked mechanically against all 63 multiply vectors in the corpus —
`MULT`, `MULTU`, `DMULT`, `DMULTU` — with no mismatches, before any code was written.

**What this cost, recorded because it is the point.** The evidence needed to *state*
the rule had been sitting in the oracle's source the whole time; what was missing was
the instrument to notice the rule was needed. Twenty-eight corpus failures cleared
from two changes of a few lines each.

### 14.4 What is still not verified

The divide family's edge cases — division by zero and the single overflowing case —
pass against the corpus and were written in advance from the architecture manual.
Those are the only arithmetic behaviours here confirmed by measurement rather than
merely unrefuted; everything else in §14 rests on the corpus's vectors, which is a
stronger footing than the rest of this page enjoys.

## 15. The conditional traps, and the instruction that was never missed

Twelve instructions: `TGE`, `TGEU`, `TLT`, `TLTU`, `TEQ` and `TNE` in the `SPECIAL`
space, and `TGEI`, `TGEIU`, `TLTI`, `TLTIU`, `TEQI` and `TNEI` in `REGIMM`. Each
compares two 64-bit values and raises a Trap exception if the comparison holds; if it
does not hold the instruction costs nothing and execution continues. They are in
`Cpu.Opcodes.Trap.cs`, and the whole family is thirty lines.

**They had been missing since the dispatch table was written**, and nothing noticed for
nine slices. The gameplan does not list them, the documentation survey did not flag
them, and no test asked for them. `Mars_Cpu.md` §8 listed what was not implemented yet
and did not mention them either — an omission is invisible to a list of known
omissions.

### 15.1 The immediate sign-extends even where the comparison does not

`TLTIU` and `TGEIU` sign-extend their 16-bit immediate to 64 bits and *then* compare
unsigned. The two halves of that sentence pull in opposite directions and it would be
easy to write either half alone: an immediate of `-2` becomes
`0xFFFFFFFF_FFFFFFFE`, so `TLTIU $2, -2` traps for almost every value including zero,
and does not trap for `0xFFFFFFFF_FFFFFFFF`. The corpus's table pins all of it.

The register forms have the matching property in the other direction: the comparison is
over all 64 bits, so a pair like `0x00000095_00000096` and `0x00000096_00000095`
orders the opposite way from how their low words order. Mars's `SLT` already read the
whole register, so this was never in doubt — but it is the property that makes the
corpus's vectors worth keeping verbatim rather than paraphrasing.

### 15.2 How the gap was found, and why it presented as something else

It was found as an **exception storm inside the floating-point conversion tests**, and
`Mars_FpuMath.md` §9.1 predicted, in writing, that the cause was an edge case in
`CVT.L`/`ROUND.L`/`TRUNC.L`/`CEIL.L`/`FLOOR.L`. That prediction was wrong, and the way
it was wrong is worth keeping.

The corpus is written in Rust, and rustc emits a `TEQ` after every integer division as
its divide-by-zero guard. The word Mars refused was `0x03E001F4` — `TEQ $ra, $zero` —
sitting immediately after a `DIVU`, in the corpus's own code for formatting a 64-bit
integer into a failure message. The conversions are the first test in the run whose
failures print a value large enough to take that path. So the storm appeared exactly
where the conversions were, was caused by the conversions failing, and had nothing to
do with how they failed: Mars raised Reserved Instruction, the corpus's handler could
not attribute it, and the recovery path divided again, and again.

**The diagnostic that settled it took one measurement.** Rather than reading the
conversion code, the run was instrumented to record the program counter and exception
code of every fault and to print the instruction word at any site raising Reserved
Instruction. One site, one word, one decode. The lesson is not that the prediction was
careless — it was the only reading the evidence then supported — but that *where* a
fault surfaces in a test corpus is evidence about the corpus's control flow, not about
the subject under test.
