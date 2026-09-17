# Mars — the corpus run, and what its tally is a measurement of

*Landed 2026-09-16. This page owns the hardware corpus as a running instrument: how far
it gets, what stops it, what its numbers mean and what they do not. The protocol is
`Mars_TestOracle.md`; the first time it spoke is `Mars_Fpu.md` §9. The harness is
`MarsCorpusTests` in `EmuSen.WiseMan`.*

---

## 1. Where it stands

**The run completes.** The corpus reaches its own teardown and prints its own verdict:

```
Finished in 3.09s. Base: Failed 163 of 4637 tests (96% success rate)
```

That is 1,040 test groups and 4,637 assertions, every one of them attempted, **and no
failing group left that tests the CPU, or the RSP as a processor** (§3). It is the
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
| the seventeen CPU groups | 1,040 | 319 of 4,637 assertions |
| the RSP's vector unit | 1,040 | 163 of 4,637 assertions |

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

The 87 distinct test groups that fail in the complete run, counted by name — a group that
fails for thirty-three values is one row — against the corpus's own 163, which counts
assertions.

| | groups | why |
| --- | --- | --- |
| caches, all four families | 45 | Mars models no caches; these cannot pass (`Mars_Cpu.md` §13) |
| cartridge memory and writes | 19 | Phase E — the PI and cartridge DMA |
| PIF RAM, MI, RDRAM registers | 13 | Phase E |
| RDP status and registers | 6 | Phase D |
| the main CPU's sub-word access to RSP memory | 4 | bus, not processor (`Mars_Rsp.md` §8) |

**No group testing the RSP as a processor remains either.** Two rows still carry the RSP's
name without being about it: the four `spmem` groups test how the main CPU's sub-word loads
and stores reach the RSP's memories, and one of the six RDP groups is called *RSP STATUS* and
tests the display processor freezing (`Mars_Rsp.md` §8). The 155 vector groups cleared in one
slice, and the five rows here are the same groups, one for one, that the census before it
listed (§10). Of what is left, only the `spmem` row is work inside a phase this plan has
reached.

> **Retired 2026-09-17: the census that preceded this one**, which is kept for its first row
> and for a count that was wrong. It opened *"The 259 distinct test groups that fail in the
> complete run"* and tabled 242. The 259 was the count before the seventeen CPU groups were
> cleared; the table was brought up to date and the sentence above it was not. The unit
> change it announced still holds — groups by name, not failure lines — and it read:
>
> | | groups | why |
> | --- | --- | --- |
> | the RSP's vector unit | 155 | Phase C, next slice (`Mars_Rsp.md` §6) |
> | caches, all four families | 45 | Mars models no caches; these cannot pass (`Mars_Cpu.md` §13) |
> | cartridge memory and writes | 19 | Phase E — the PI and cartridge DMA |
> | PIF RAM, MI, RDRAM registers | 13 | Phase E |
> | RDP status and registers | 6 | Phase D |
> | the main CPU's sub-word access to RSP memory | 4 | bus, not processor (`Mars_Rsp.md` §8) |

**No CPU group remains, and this time the run is complete**, so the statement §2 had to
retract can be made about the run it describes. It is still a claim about what this corpus
asks rather than about the CPU in general; `Mars_Tlb.md` §7.4 and §7.5 name two rules Mars
implements on the architecture's word rather than on a measurement, and those are the
places a different corpus would look first.

> **Retired 2026-09-17, the same day it was written: the list of CPU groups.** It is kept
> because each bullet diagnosed its group from the failure messages alone, before any code
> was read, and all four diagnoses held — but they were incomplete, and the incompleteness
> is worth seeing. Fixing them cleared sixteen of the seventeen groups; the seventeenth
> still failed on fourteen of its thirty-three rows, for a reason none of the bullets
> named (§9).
>
> **Seventeen CPU groups, and what the failures already say about them:**

> - **The extended TLB refill vector.** With 64-bit addressing on, a refill vectors to
>   `0x80000080`, not `0x80000000`. Mars always uses the 32-bit one. That single defect is
>   the whole of *LW TLB Miss or Address Exception (64 bit addressing mode)* — thirty-three
>   failure lines from one cause — and both *address not sign extended (64 bit)* groups.
> - **The region bits in a TLB match.** *Expect TLB miss on R mismatch* fails for
>   twenty-eight values: bits 63:62 of an address must participate in matching an entry,
>   and Mars's comparison masks them away with everything above bit 31.
> - **Write masks on the TLB's own registers.** `EntryLo0` and `EntryLo1` keep thirty bits,
>   not thirty-two; `PageMask` keeps only valid mask bits; `EntryHi` has its own mask; and
>   `Random` ignores writes.
> - **`Config`'s clock-ratio bits.** Bits 30:28 are read-only ones, and `Mars_Cop0.md` §4
>   listed them as writable.
>
> The first bullet's "that single defect is the whole of" was the claim that did not hold.

## 9. The seventeen CPU groups

*Landed 2026-09-17. `Mars_Tlb.md` §7 has the rules; this section is how the slice went.*

**395 → 319**, and the seventy-six assertions that moved are exactly the seventy-six failure
lines the seventeen groups held — so nothing else moved in either direction. Every group was
written as a failing test in the corpus's own vectors before any code changed, and the
changes were, in order of what they cleared: the extended refill vector and a sixty-four bit
TLB match; what an entry keeps of the registers written to it, which is less than the
registers keep and normalised differently; the registers' own masks; `Random` as a real
counter; `Config`; and the end of the kernel's mapped region, which is two gigabytes short of
the pattern the other regions follow.

**The one diagnosis that was wrong** was the confident one. The census said a single defect,
the refill vector, was *the whole* of the largest group. The first round of fixes cleared
nineteen of that group's thirty-three rows. The other fourteen were address errors, not
misses — the group's name says *TLB Miss or Address Exception* — and they failed on
`EntryHi`, which an address error fills in just as a miss does and which Mars wrote only
beside the TLB lookup (`Mars_Tlb.md` §7.6).

**Those fourteen failures were in the verdicts the census was written from.** So was the
end of the kernel's mapped region: four of the rows said *"Expected exception AdEL but got
Ok(TLBL)"* at addresses from `0xC00000FF_80000000` up. The diagnosis was drawn from the first
four failure lines of a thirty-three-line group, all of which happened to be miss rows, and
the remaining twenty-nine were not read. The census therefore named one cause for a group
that had three.

The narrower lesson is not about §2's truncated runs at all, because this run was complete:
a group's name is a summary of its rows, and a diagnosis of the group is only as good as the
fraction of its rows that were read. An earlier draft of this section said the address-error
rows had only become visible once the vector was fixed. That was wrong, and was checked and
corrected before it was committed — it was the same mistake a second time, drawing a
conclusion about the rows from the ones in front of it.

**Two of the tests written for this slice were wrong before they were right, both in the same
way.** They used page zero to provoke a TLB miss, and an empty TLB entry matches page zero
— as an *invalid* entry, which takes the general vector rather than a refill. The corpus
avoids this by clearing its TLB with distinct page numbers; Mars's tests now avoid page zero
and say why.

> **Retired 2026-09-17: the census that preceded this one.** It listed 160 failures and
> said *"Every group is now a later phase or a decision. There is no CPU-level failure
> left anywhere in the run."* It was accurate about the run it described, and §2 records
> why that was not enough.

## 10. The vector unit

*Landed 2026-09-17. `Mars_RspVector.md` has the rules; this section is what the run said.*

**319 → 163, and 242 failing groups → 87.** The 156 assertions that moved are exactly the 156
failure lines the 155 vector groups held; a diff of the failing group names before and after
is 155 removals and **no additions**; and every other failure line in the report is
unchanged to the character, values included. Every one of the 155 passed in the first complete run after the unit was built. There
was no second round.

That is the opposite of §9, where the census's diagnosis named one cause for a group that
had three, and the difference is worth stating precisely rather than as luck. §9's groups
were diagnosed from the failure messages; these were implemented from the test source,
which for most of the unit carries a model of the part rather than a list of values
(`Mars_RspVector.md` §0). A model covers every row of its group; four lines of a message
do not.

**One suspected disagreement was an arithmetic error, and it was the author's.** Before the
run, a hand check of the `VRNDP` accumulator-overflow table appeared to put one element
`0x8000_0000` away from the model, and the slice was begun expecting that group to fail and
to need recording as an unexplained residual. It passed. Simulating the model
instead of multiplying by hand gives the corpus's values exactly: the hand calculation had
carried wrongly in `0x7FFF_0000 × 32769`. It is recorded because the prediction was stated
before the measurement and the measurement refuted it.

**The run is longer, and still ends itself.** Three of the vector tests run an RSP program
65,536 times each. A complete run now takes about thirty seconds of wall time under the
harness — the time before this slice was not measured, so no ratio is given — and it still
reaches the corpus's summary line, which it could not do had the 400-million instruction
budget (§4) run out first.

**The unit's tests were checked for bite.** Of the twenty-nine cases written before the unit
existed, four passed against the unbuilt unit — all four from one theory, a zero-result test
that passed because the load that should have made the register non-zero did not happen
either. It now asserts the
load. A second, the aliasing test, was found while writing the man page not to distinguish
a correct implementation from one that writes in place; it was rebuilt, and then checked by
mutation: an in-place `VOR` fails it.

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

`MarsCorpusTests` asserts **the corpus's own summary line**, `Failed N of 4637 tests`, with
the `N` of the latest row of §1's table.
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
