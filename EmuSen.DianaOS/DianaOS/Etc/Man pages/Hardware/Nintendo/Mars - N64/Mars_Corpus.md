# Mars — the corpus run, and what its tally is a measurement of

*Landed 2026-09-16. This page owns the hardware corpus as a running instrument: how far
it gets, what stops it, what its numbers mean and what they do not. The protocol is
`Mars_TestOracle.md`; the first time it spoke is `Mars_Fpu.md` §9. The harness is
`MarsCorpusTests` in `EmuSen.WiseMan`.*

---

## 1. Where it stands

**The run completes.** The corpus reaches its own teardown and prints its own verdict:

```
Finished in 2.50s. Base: Failed 395 of 4637 tests (91% success rate)
```

That is 1,040 test groups and 4,637 assertions, every one of them attempted. It is the
first complete run of the corpus this project has made, and the ratchet now asserts that
line (§5). It took two changes in the slice that produced it: the RSP learned to halt
(`Mars_Rsp.md`), which let the run past the wait it had sat in since Phase A, and the
harness was given the memory the corpus assumes, which let it past everything after
(§8).

Every tally before that, each taken at the end of a slice, counted test *groups* started
and failed rather than the corpus's own assertions, because no run had yet reached the
line that reports those:

| | started | failed |
| --- | --- | --- |
| coprocessor register files | 521 | 202 |
| arithmetic operand widths | 521 | 174 |
| coprocessor zero | 521 | 138 |
| software floating point | 561 | 138 |
| letting the run finish | 720 | 171 |
| privilege modes | 720 | 167 |
| reverse-endian | 720 | 160 |
| the RSP's scalar half, and a complete run | 1,040 | 395 of 4,637 assertions |

The fourth row is the shape to want: forty tests that had never run before ran, and all
forty passed.

**The failure count went up twice, and both times that was the point.** The fifth row ran
159 tests that had never run and 33 of them failed; the last ran 320 more groups and the
count of failing groups rose from 160 to 395. A tally is only comparable with itself when
the run reaches the same place, which is why every row above asserted both halves — and
why the last row stops needing to, because a run that reaches the end reaches the same
place every time.

## 2. What "the corpus passes" means, and what it does not

The tally is the output of an instrument, and an instrument that stops early reports on
the part of the world it reached. That sounds obvious written down, and it was not
obvious in use: `Mars_Cop0.md` §1 recorded that **every** COP0 test the corpus ran
passed, which was true, and which read as a statement about coprocessor zero. When the
run was made to go further, forty-four more COP0 tests appeared and every one of them
failed.

So: a claim of the form *"every X test passes"* is worth exactly as much as the answer
to *"and where did the run stop?"* — and that second question was not being asked. The
tally in §1 is now written as a pair for that reason, and the budget's comment in the
harness names where the run ends rather than how long it takes.

**This page's own claim, stated to the same standard.** As of this slice the run stops
in `RSP BREAK`, spinning on a four-instruction wait loop. Everything before that point
in the corpus's test list has been attempted. Everything after it has not.

> **This page then made exactly the mistake it describes, two slices later.** §3 said,
> in bold, that there was *"no CPU-level failure left anywhere in the run"* — and
> qualified it carefully as a claim about what the corpus asks rather than a claim that
> the CPU was finished. The qualification was right and not sufficient. The run that
> claim described still stopped before the corpus's TLB section, and when the run
> completed, seventeen failing CPU test groups appeared, accounting for seventy-six
> failure lines between them. Every one of them is CPU work.
>
> The qualification guarded against the wrong reading — "the CPU is finished" — and not
> against the one that mattered — "the run is complete". The only claim of this form that
> survives is one made about a run that reached the corpus's own summary line, and §1 is
> now the first such claim on this page.

## 3. The census of what still fails

The 259 distinct test groups that fail in the complete run. **The unit changed here**:
every earlier table on this page counted failure *lines*, one per failing value, because
that was what the truncated runs could count. A group that fails for thirty-three values is
one row below, and the corpus's own 395 counts something else again — assertions.

| | groups | why |
| --- | --- | --- |
| the RSP's vector unit | 155 | Phase C, next slice (`Mars_Rsp.md` §6) |
| caches, all four families | 45 | Mars models no caches; these cannot pass (`Mars_Cpu.md` §13) |
| cartridge memory and writes | 19 | Phase E — the PI and cartridge DMA |
| **TLB and COP0 registers** | 13 | **CPU** — see below |
| PIF RAM, MI, RDRAM registers | 13 | Phase E |
| RDP status and registers | 6 | Phase D |
| the main CPU's sub-word access to RSP memory | 4 | bus, not processor (`Mars_Rsp.md` §8) |
| **64-bit addressing** | 4 | **CPU** — see below |

**Seventeen CPU groups, and what the failures already say about them:**

- **The extended TLB refill vector.** With 64-bit addressing on, a refill vectors to
  `0x80000080`, not `0x80000000`. Mars always uses the 32-bit one. That single defect is the
  whole of *LW TLB Miss or Address Exception (64 bit addressing mode)* — thirty-three
  failure lines from one cause — and both *address not sign extended (64 bit)* groups.
- **The region bits in a TLB match.** *Expect TLB miss on R mismatch* fails for
  twenty-eight values: bits 63:62 of an address must participate in matching an entry, and
  Mars's comparison masks them away with everything above bit 31 (`Mars_Tlb.md` §6).
- **Write masks on the TLB's own registers.** `EntryLo0` and `EntryLo1` keep thirty bits,
  not thirty-two; `PageMask` keeps only valid mask bits; `EntryHi` has its own mask; and
  `Random` ignores writes. The same kind of work as `Mars_Cop0.md` §2, on registers that
  slice did not reach.
- **`Config`'s clock-ratio bits.** Written with `0x8000`, hardware reads back `0x7006E460`
  and Mars reads `0x0006E460`: bits 30:28 are read-only ones, and `Mars_Cop0.md` §4 lists
  them as writable. That is a claim on a page which described itself as settled, refuted by
  a test the run had never reached — the same shape as the §2 note above, one level down.

These are the next CPU-level work the instrument asks for. Each is small; together they are
most of what stands between Mars's CPU and a corpus section with no failures in it.

> **Retired 2026-09-17: the census that preceded this one.** It listed 160 failures and
> said *"Every group is now a later phase or a decision. There is no CPU-level failure
> left anywhere in the run."* It was accurate about the run it described, and §2 records
> why that was not enough.

## 4. The budget, and why it is now a safety net

The harness runs until the corpus prints its summary, checking every million
instructions, and the budget of 400 million is only there to stop a run that never gets
there. **The run ends itself.** That is a better arrangement than any budget, because a
budget has to be chosen past a stopping point that moves every time Mars improves, and it
is cheaper, because the run stops the moment it is done rather than idling to a number.

For the record of how that number moved: it was 60 million while the run was cut inside
the 64-bit conversions (`Mars_FpuMath.md` §9.1), 190 million while it stopped in
`RSP BREAK`, and briefly 900 million while this slice looked for where a completed run
ended — which is how the storm in §8 was found.

A complete run costs about twenty seconds in a debug build, and the suite pays it twice.
That is a real cost on every test run and it is accepted deliberately: this is the only
instrument in the project that grades Mars against silicon, and a graded measurement
that is skipped is not a measurement.

**Why instructions and not wall time or frames.** The count is deterministic across
machines and across builds, so the tally is reproducible; anything derived from the host
clock would make the ratchet flap.

## 5. The ratchet

`MarsCorpusTests` asserts **the corpus's own summary line**: `Failed 395 of 4637 tests`.
That replaced a started-and-failed pair derived by counting markers in the verdict text,
and it is a stronger assertion in two ways. It counts what the corpus counts, which is
assertions rather than groups. And it cannot be satisfied by a run that did not finish —
the corpus prints that line only from its teardown, so a truncated run fails the ratchet
with *"the run did not reach the end"* rather than with a smaller number that looks like
progress. That was checked by mutation: running the corpus without the memory it assumes
fails the assertion with exactly that message.

The earlier forms are kept here because each was right for the run it measured. A floor
was tried first and **thrown away after it failed to catch two deliberate mutations**
(`Mars_Fpu.md` §9.1). The pair replaced it, and was sensitive at the scale that mattered —
the `LWR` fix moved two tests and the tally moved from 173 to 171. The summary replaces the
pair because the run it measured no longer exists.

The assertion is meant to be edited as defects are fixed, and any change to it should be
accompanied by an explanation of which tests moved and why.

The whole verdict text is written to `mars-corpus-verdicts.txt` beside the test assembly
on every run, and the assertion message names the path. A tally says something moved;
only the report says what.

## 6. What this slice fixed, and what each was worth

Four defects, in descending order of what they cleared:

- **The COP0 decode boundary** (`Mars_Cop0.md` §10) — 44 tests. Reserved function codes
  in the `CO` space are no-ops, not Reserved Instruction; three sub-opcodes are
  recognised and idle.
- **The conditional traps** (`Mars_Cpu.md` §15) — no tests directly, and it is the
  reason the other three could be counted at all. Twelve instructions that had never
  been implemented, found as an exception storm 64 million instructions into the run.
- **The 64-bit conversion ceiling** (`Mars_FpuMath.md` §6.3) — 7 tests.
- **`LWR`'s partial merge** (`Mars_Cpu.md` §7.3) — 2 tests, and it refutes an
  expectation that had been derived by hand from the architecture and had stood, tested
  and green, for eight slices.

**The trap family is the one worth drawing a conclusion from.** It cost thirty lines and
cleared nothing by itself; what it bought was 159 additional tests reaching the
instrument. Three of the four defects in this list were invisible until it landed, and
two of them are in subsystems that had been declared finished.

## 7. The three slices after it

Four tests, and a fifth thing that does not show in the tally. The VR4300's three modes
landed (`Mars_Privilege.md`): the mode derivation, the forty-five row address map, the
coprocessor-zero gate and the doubleword-instruction restriction. 171 → 167.

**The number understates it.** The address map is the first part of Mars where the same
address means different things to different code, and the four tests that moved are the
corpus checking a table of forty-five rows that Mars now reproduces exactly. It also
closed a gap `Mars_Cop0.md` §8.1 had recorded, in writing, as something to find
deliberately later — and the route that found it was this page's §3 census, not anyone
remembering the note.

**Then reverse-endian**, seven tests, 167 → 160 (`Mars_ReverseEndian.md`). Two of the
seven fell to the feature itself; the other five fell to a defect in the **TLB** that the
feature's test harness exposed — a page-size error that had survived four slices because
it is invisible unless a pair's two halves name unrelated frames, or two pairs sit next to
each other (`Mars_Tlb.md` §1.1).

That is the second time in three slices that the useful thing was not the feature but what
building it made reachable. The trap family bought 159 tests; reverse-endian bought a TLB
bug that nothing else in the corpus had caught and that every operating system would
have.

**Then the RSP's scalar half** (`Mars_Rsp.md`), and the third time. Eighty-eight RSP and
SP groups pass, and two defects inside the RSP itself were found and fixed — a status
write naming both halves of a field, and a jump-and-link whose target is its own link
register. But the RSP learning to halt is what let the run past its oldest wait, and what
it reached was §8, and what that led to was the first complete run.

## 8. The corpus assumes an Expansion Pak, and the harness did not provide one

With the RSP able to halt, the run passed the wait it had sat in since Phase A and
reached the corpus's TLB section — and then, inside *TLB: Execute code mapped in 4k page*,
it reached an exception storm and aborted, about 229 million instructions in.

**It was not a CPU defect.** The measurement that settled it: the test maps a 4K page onto
the physical address `0x700000`, writes a three-instruction function into it, and jumps
there. At the moment of the jump the TLB entry was present, valid and correct — and the
physical memory under it read as zeros, so execution ran through the page as `NOP`s and
off its end into the invalid half of the pair, faulting on every fetch.

`0x700000` is seven megabytes. The corpus defines its heap as ending there, which is a
fixed address inside the **8MB Expansion Pak**. `MarsCorpusTests` had constructed the bus
with Mars's default of 4MB, and Mars reads above installed RDRAM as zero and drops writes to
it — deliberately, because the corpus's own bootstrap depends on that behaviour to tell a
4MB machine from an 8MB one (`Mars_Memory.md` §2.1). The store and the fetch were both
correct. There was nothing there to store into.

Giving the corpus the memory it assumes let the run complete, and turned an aborted run of
995 groups into the complete run in §1. The ratchet now depends on it, and running the
corpus without the Expansion Pak fails the ratchet with *"the run did not reach the end"*.

### 8.1 How long this had been true, and why nothing showed it

Every run since the first one had used 4MB. Most of the corpus never touches memory above
four megabytes, and the parts that did were far enough into the test list that no run had
reached them. So the harness had been measuring the corpus against the wrong machine the
whole time, and it was invisible for exactly as long as the run was short.

That is the same shape as §2, in a different place. §2 is about a claim that silently
depends on how far the run got; this is about **a configuration** that silently depends on
it. The TLB page-size defect was the same shape again (`Mars_Tlb.md` §1.1): invisible to
every case the existing tests built, and exposed the first time a test built something
ordinary that they had not.

### 8.2 How it was found, recorded because the route was not straight

The diagnosis took six instrumented runs, and four of them were wrong about something.
One logged exceptions inside a guard that skipped every step without one, and so recorded
nothing. One armed itself on the verdict text only every 65,536 steps, and so armed after
the event. One dumped the TLB at the *first* arrival at the target address, which was an
earlier test, and showed an empty TLB that looked like the explanation and was not. The
run that settled it dumped the state at the *last* arrival instead, and showed the entry
present and the memory empty in the same frame.

The lesson is narrower than "instrument carefully". Each wrong probe produced a plausible
answer rather than no answer, and the plausible answers pointed three different ways: at
a missing exception, at a cleared TLB, and at a store that never happened. The one thing
they had in common was that none of them had looked at the entry and the memory at the
same instant.
