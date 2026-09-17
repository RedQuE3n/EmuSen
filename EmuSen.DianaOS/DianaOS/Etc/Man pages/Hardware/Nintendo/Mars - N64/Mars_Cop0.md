# Mars — coprocessor zero is not thirty-two words of storage

*Written 2026-09-16, from the hardware corpus's COP0, LL/SC and address-error
sections. `Mars_Cpu.md` §9 described COP0 as "the register file is otherwise plain
words", and said so accurately: it was. Almost every register turned out to be
narrower than a word, partly read-only, partly written by hardware, constant, or not
storage at all. `Cpu/Core/Cop0Registers.cs`, with `MarsCop0RegisterTests` and
`MarsCpuFaultRegisterTests` in `EmuSen.WiseMan`.*

*Every mask and value here is a hardware measurement taken from the corpus's own
assertions, not a reading of a manual.*

---

## 1. What this slice closed

With it, **every CPU, COP0, TLB, exception, LL/SC and address-error test the corpus
runs passes.** What remains failing is caches, which Mars does not model and which
cannot pass (`Mars_Cpu.md` §13), and cartridge DMA, which is Phase E. The tally moved
174 → 138.

That is the condition `Mars_Gameplan.md` §4.1 set for Phase A being finished, reached
two phases later than planned and by a route the plan did not anticipate: not by
writing more of Phase A, but by building enough of Phase B that the oracle could
speak at all.

> **Qualified, 2026-09-16.** The sentence in bold above says *the corpus runs*, and
> that was doing more work than it looked like: the run stopped a third of the way
> through the test list. When it was made to finish, forty-four further COP0 tests
> appeared and all of them failed (§10). The claim was not false, but it was a
> statement about an instrument's output rather than about the hardware, and it read
> as the second. `Mars_Corpus.md` §2 is where that distinction now lives.

## 2. The write masks

| Register | What a write keeps |
|---|---|
| `Index` | `0x8000003F` — six bits of index and the probe-failure bit |
| `Context` | bits 63–23; the rest belong to the fault (§7) |
| `Wired` | `0x3F` |
| `BadVAddr` | nothing — read-only |
| `Status` | `0xFFF7FFFF` — bit 19 is not writable, and there is no upper word |
| `PRId` | nothing — constant `0x00000B22` (§3) |
| `Config` | `(value & 0x7F00800F) | 0x00066460` (§4) |
| `LLAddr` | `0xFFFFFFFF` |
| `XContext` | bits 63–33; the rest belong to the fault (§7) |
| `PErr` | `0xFF` |
| `CacheErr` | nothing — always zero |
| `TagLo` | `0xFFFFFFFF` |
| `TagHi` | nothing — always zero |
| `EPC`, `ErrorEPC` | everything — sixty-four bits, no mask at all |

Storing plain words made all of these read back wrong, and **none of it is visible
until something reads a register back**. A handler that does a read-modify-write on
one is the usual way a game finds out (§4 is exactly that case).

### 2.1 A 32-bit move carries all sixty-four bits

`MTC0` delivers the **whole** source register; the destination's own width and mask
decide what survives. It is not a 32-bit value sign-extended, which is what Mars did
before and what the instruction's name suggests.

The corpus proves this for `Context`, `XContext` and `EPC` — writing a 64-bit register
through `MTC0` lands all of it. Mars generalises from those three to every register,
because the alternative is a per-register rule with no evidence behind it; for the
32-bit registers the two readings are indistinguishable anyway, since the mask
discards the upper word either way.

## 3. The processor identifier is a constant, and not everyone's is the same

`PRId` reads `0x00000B22` and ignores writes. The corpus notes that a survey of real
consoles found **one** reporting `0x00000B10` instead, and accepts either. Mars
answers `0x00000B22`, the majority value, as a choice rather than a fact — if a title
is ever found that behaves differently on the two, this is the line to revisit.

## 4. Config is half constant, and the constant half is why the corpus's first test failed

`Config` keeps `0x7F00800F` of what is written and forces `0x00066460` into the rest.

The corpus's very first assertion — the one printed at the top of every report since
the oracle started talking — was `Initial COP0 Config: a=0x6E463 b=0x7006E463`. The
difference is `0x70000000`, which lies inside the *writable* mask, so no amount of
masking would have fixed it.

**The cause was a missing reset value.** The cartridge's boot code does a
read-modify-write of `Config`: it reads, changes the bits it cares about, and writes
back. On hardware the read returns the reset value, which carries `0x70000000`. Mars
seeded `Config` with `0x0006E463` and so the boot code wrote those bits away. Seeding
`0x7006E463` reproduces the hardware answer exactly.

This is worth stating because of what it says about the class of bug: **a register
whose reset value is wrong is invisible until software reads it before writing it**,
and read-modify-write is the normal way to touch a configuration register. The masks
in §2 have the same property.

## 5. Seven registers that are not registers

Numbers **7, 21, 22, 23, 24, 25 and 31** do not exist. They are not zero, and they are
not storage: they read back **the last value any COP0 write put on the bus**, whatever
register that write was aimed at.

So writing to one and reading it straight back returns what was written — which is
what makes it look like storage — but writing to *any other* COP0 register in between
changes the answer. The corpus tests exactly that, with several values, and says in
its own comment that it is checking the emulator is not cheating.

Mars models it as one latch, written by every COP0 write before the destination is
considered. That is a guess at the mechanism from the outside; what is measured is
the behaviour, and the latch is the smallest thing that produces it.

## 6. The linked load records a physical address that nothing compares

Two findings, both counter to how the pair reads:

- **`LLAddr` holds the physical address shifted right by four**, not the virtual
  address. Mars stored the virtual one.
- **`SC` gates on the link bit alone and never compares `LLAddr`.** The corpus proves
  this with two virtual aliases of one physical page: the `LL` and the `SC` resolve to
  *different physical lines* and the `SC` still succeeds. `LLAddr` is observational —
  something a handler can read, and nothing the hardware acts on.

`ERET` breaks the link but **leaves `LLAddr` standing**, so an `SC` after a return
fails and stores nothing while the address stays readable. Mars cleared the link on
exception entry already; clearing it on exception *exit* was missing.

## 7. Three registers a fault fills in, from one address

`BadVAddr`, `Context` and `XContext` are all written from the faulting address, on
**every** address-related exception — address errors included, not only TLB failures,
which is all Mars did before.

- `Context` bits 22–4 take address bits 31–13; bits 3–0 are cleared; bits 63–23 are
  left to software.
- `XContext` bits 30–4 take address bits 39–13, bits 32–31 take address bits 63–62,
  bits 3–0 are cleared, and bits 63–33 are left to software.

**The clearing of the low four bits is the part that had to be measured.** The
software-write path in §2 *preserves* the whole low field of `Context`; the fault path
overwrites it, zeros included. Preserving it in both places is the obvious
implementation and produces a value that is wrong only in its bottom nibble — which is
below page granularity and so invisible to any handler that uses the register for what
it is for.

## 8. An address has to be the sign extension of its own bit 31

Outside 64-bit addressing, an address whose upper word is not the sign extension of
bit 31 is not an address: `0x00000000_80001234` raises an address error where
`0xFFFFFFFF_80001234` loads. Mars decoded the low word and ignored the rest, so it
accepted both.

The check is gated on `Status` bit 7, kernel-mode extended addressing. User and
supervisor modes have their own enables and **are not modelled** — Mars has no
privilege level, so every access is kernel.

### 8.1 What accepting the address does not mean

With 64-bit addressing enabled the wide address is accepted, and then **Mars puts it
in the wrong place**: segment decoding reads the low word only, so
`0x00000000_80001234` is treated as `KSEG0` rather than as the 64-bit user segment it
names. A test records this rather than asserting the behaviour is right.

Nothing in the corpus grades it, because its address-error tests run in 32-bit mode.
It is a real gap and it is written down here so that it is found deliberately rather
than rediscovered.

## 9. The delay slot behind a branch that was not taken

An ordinary branch has a delay slot **whether or not it is taken**, and a fault in
that slot must save the branch's address and set the delay-slot flag in `Cause`. Mars
set the flag only when the branch was taken, because the same variable was doing two
jobs: "a branch is pending" and "the next instruction is a delay slot". They are not
the same claim.

A branch-*likely* that is not taken is the genuine exception: its slot is nullified
and never executes, so there is nothing to be in a delay slot. Both halves have a
test, and the untaken-ordinary case is the one that was wrong.

**How it surfaced.** As `ExceptPC points to wrong instruction` — the corpus read the
word at the saved address and found the wrong opcode. Nothing about the failure said
"delay slot"; the route from that message to this cause was reading the test's own
assembly and noticing that its branch compares a register with itself.

## 10. The decode boundary: where coprocessor zero refuses and where it ignores

Coprocessor zero's instruction space has two halves, and they answer a reserved
encoding differently.

**With the `CO` bit set** — opcode `0x10`, `rs = 0b10000`, the low six bits selecting a
TLB or exception operation — exactly five function codes are defined: `TLBR` (1),
`TLBWI` (2), `TLBWR` (6), `TLBP` (8) and `ERET` (24). **Every other code in `0x00`–`0x3F`
is a silent no-op.** It raises nothing, writes nothing, and the instruction after it
runs normally.

**Without it**, the `rs` field selects a coprocessor transfer, and a reserved value
there *does* raise Reserved Instruction. `0x03`, `0x07` and `0x09`–`0x0F` all refuse.

So the same word shape refuses in one half of the space and is ignored in the other.
Mars previously refused in both, except for function codes `0x20` and above, which were
allowed through for an unrelated reason — see §10.3.

### 10.1 One reserved function code out of fifty-nine still traps

Function code `0x10` is the R3000's `RFE`, which the R4000 dropped in favour of `ERET`.
It is the only reserved code in the `CO` space that raises Reserved Instruction. There
is no principle available here that predicts which one it would be; it is a measurement,
and it is implemented as a measurement — a single named case beside the five defined
ones, rather than a range.

Nothing between the sub-opcode and the function field is decoded at all. The twenty
bits in between can hold anything: a reserved code stays a no-op, `0x10` still traps,
and `TLBP` still executes.

### 10.2 Three sub-opcodes that are decoded and idle

`CFC0` (`rs = 2`), `CTC0` (`rs = 6`) and `BC0` (`rs = 8`) raise nothing. The two control
moves are the generic coprocessor form, and coprocessor zero has no control registers
for them to move; `BC0` is a coprocessor-condition branch whose condition never holds.
All three are inert.

They matter because they are what makes the boundary in §10's second paragraph precise.
Without them the rule would read "reserved `rs` traps"; with them it reads "these three
are recognised, and the values around them are not" — which is the difference between a
range check and a decode.

**What is not established here.** `BC0` is a branch, and whether the instruction behind
it occupies a delay slot is not answered by any evidence Mars has. The corpus's test
places a `nop` there and asserts only that no exception is raised. Mars treats `BC0` as
inert with no delay slot, which is the minimum the evidence supports and is recorded
here as the assumption it is.

### 10.3 A right answer that had been reached for the wrong reason

Function codes `0x20` and above were already ignored, because the corpus calls one of
them unconditionally at startup as an emulator-detection hook and Mars had to get past
it (`Mars_Cpu.md` §9). The behaviour was right and the justification was narrow: it was
"this particular range is an emulator extension", not "the reserved `CO` space is
inert". The corpus's sweep now establishes the general rule, and the special case for
the extension range is gone.

A test asserting the opposite for `0x1F` — that a reserved function *below* that range
still refuses — had stood since that slice and was **retired here**, because it was
false. It is worth naming rather than quietly deleting: it was a claim derived from
where a boundary had been drawn for a different purpose, and it passed for nine slices
because nothing had yet asked hardware.

### 10.4 What this was worth

Forty-four corpus failures, from a change of about ten lines. The three tests behind
them — the sweep of the whole `CO` space, the sweep of the reserved `rs` values, and the
operand-bit sweep — are among the most thorough in the corpus, and the section of its
source that defines them reads as a specification of the boundary rather than as a list
of cases. `MarsCop0DecodeTests` mirrors it: the reserved-function sweep is generated
from the same five defined codes and one exception, so it cannot drift out of step with
the rule it is asserting.

**None of this was reachable before.** `Mars_Cop0.md` §1 claimed that every COP0 test the
corpus ran passed. That was true of the run, and the run stopped before these tests —
which is the difference between a statement about an instrument's output and a statement
about the hardware. See `Mars_Corpus.md` §2.
