# Mars — the corpus run, and what its tally is a measurement of

*Landed 2026-09-16. This page owns the hardware corpus as a running instrument: how far
it gets, what stops it, what its numbers mean and what they do not. The protocol is
`Mars_TestOracle.md`; the first time it spoke is `Mars_Fpu.md` §9. The harness is
`MarsCorpusTests` in `EmuSen.WiseMan`.*

---

## 1. Where it stands

**720 tests started, 171 failed.** The run executes about 177 million instructions and
emits about 88KB of verdicts before it stops, which it does inside the RSP tests,
waiting for a signal from hardware Phase C has not built.

Every tally so far, each taken at the end of a slice:

| | started | failed |
| --- | --- | --- |
| coprocessor register files | 521 | 202 |
| arithmetic operand widths | 521 | 174 |
| coprocessor zero | 521 | 138 |
| software floating point | 561 | 138 |
| this slice | 720 | 171 |

The fourth row is the shape to want: forty tests that had never run before ran, and all
forty passed.

**The failure count went up, and that is the point.** 159 tests ran here that had never
run before, and 33 of them fail. A tally is only comparable with itself when the run
reaches the same place, and this is the first slice where it does not — which is why
both halves of the pair are asserted and not just the second.

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

## 3. The census of what still fails

All 171, grouped, and every group is either a subsystem Mars does not have or a
deliberate decision:

| | count | why |
| --- | --- | --- |
| cartridge memory and writes | 92 | Phase E — the PI and cartridge DMA |
| caches, all four families | 46 | Mars models no caches; these cannot pass (`Mars_Cpu.md` §13) |
| privilege and reverse-endian user mode | 11 | not built: supervisor and user modes |
| PIF RAM, MI, RDRAM registers | 14 | Phase E |
| RDP status and registers | 5 | Phase D |
| RSP status and program counter | 3 | Phase C |

**There is one group here that is neither a later phase nor a decision.** The eleven
privilege tests are CPU work: the VR4300 has kernel, supervisor and user modes, with
different legal address ranges and different rules about which instructions are
available, and Mars runs everything as kernel. That is the next CPU-level thing the
corpus is asking for, and it is a slice of its own rather than a tail-end fix.

## 4. The budget, and why it is a number of instructions

`VerdictBudget` is 190 million instructions — comfortably past where the run stops, and
chosen so that the stall is inside the budget rather than the budget being what ends the
run. The distinction matters: a budget that cuts the run makes the tally a function of
the budget, and the previous slice's tally was exactly that (`Mars_FpuMath.md` §9.1).

It costs about twelve seconds per run in a debug build, and the suite pays it twice —
once for the tally and once for the assertion that nothing unimplemented is reached.
That is a real cost on every test run and it is accepted deliberately: this is the only
instrument in the project that grades Mars against silicon, and a graded measurement
that is skipped is not a measurement.

**Why instructions and not wall time or frames.** The count is deterministic across
machines and across builds, so the tally is reproducible; anything derived from the host
clock would make the ratchet flap.

## 5. The ratchet

`MarsCorpusTests` asserts the exact pair — started and failed — rather than a floor.
A floor was tried first and **thrown away after it failed to catch two deliberate
mutations** (`Mars_Fpu.md` §9.1). The assertion is meant to be edited as defects are
fixed, and any change to it should be accompanied by an explanation of which tests moved
and why.

It is sensitive at the scale that matters: the `LWR` fix in this slice moved two tests,
and the tally moved from 173 to 171.

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
