# emusen-singlestep: the shared half of the single-step CPU harnesses

Written 2026-10-27. The crate is `EmuSen/Cores/Shared/emusen-singlestep`. It is a development-only crate: the CPU
crates take it as a `dev-dependency`, only their `#[cfg(test)]` harness modules use it, and it never ships.

## 1. Why it exists

SingleStepTests publishes one JSON file per opcode, each a list of cases: an initial state, a final state and the bus
cycles between them. Six harnesses run these suites: `beryl-z80`, `beryl-m68k`, `blackmoon-r3000` and VenusRT's
`spc700` and `w65816` (the 65816 and the SPC700 share a module). A duplication audit on 2026-10-27 found the loading,
the parallel run, the tally and the report copied across them, about 1,900 tokens over six files. A fact held six times
can be fixed in five of them, so it is held once.

## 2. What the crate holds

- `suite_files`, `read_cases`, `int` and `ram`: a suite's folder listed in order, a file parsed into cases, the
  integer and the `[address, value]` pairs a case names (the address masked by the caller).
- `Outcome`: the three checks of one case, registers, memory and cycles, with the first difference reported.
  `Outcome::from_checks` takes each check's difference (None when it held) and reports registers first, then memory,
  then cycles. `capped` marks a case whose cycle list ends at the suite's cap, and `explained` a failure with a
  documented cause (the 68000's address errors), counted apart as `unexplained`.
- `first_mismatch`: the index where two cycle lists first differ, or where the shorter ends, so a missing or extra
  cycle fails as a difference does.
- `FileReport`, `run_cases`, `run_suite`, `total`, `summary` and `print_misses`: the cases of a file graded and tallied,
  the files run on a few threads and reported in file order.

## 3. What stays in each CPU

The register set, the bus that records the cycles and the form a cycle takes differ by processor and stay in the CPU's
harness: the Z80's T-states and port transactions, the 68000's two suites and its merged byte cycles, the R3000A's
dispute D-1 (`BlackMoon_R3000.md` §8), and the 65816's cycle cap and replay. Each CPU supplies a grading function from
a case to an `Outcome`; the crate does not know what a register is. VenusRT keeps its own `EMUSEN_VENUSRT_THREADS`
setting and the two older harnesses keep their `replay` controls and spoilings, which are tests of the CPU's own bus.

## 4. Measured on 2026-10-27

Every CPU's results were unchanged by the move: the Z80 passed 1,604,000 of 1,604,000 cases, the 68000 310,649 of
317,500 on the primary suite and 794,139 of 1,000,060 on the second (the ruled misses of `Beryl_M68k.md`), the R3000A
55,000 of 55,000, and VenusRT's SPC700 256,000 and 65816 5,120,000 replay cases (39,989 capped) as before.

Three faults were seeded in the crate, each reverted: a registers check read from the memory difference, a length
mismatch of the cycle lists ignored, and the first case of every file skipped. The crate's own tests caught all three,
and the ignored length mismatch was also caught by the R3000A's replay controls.
