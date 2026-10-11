# BlackMoon_R3000 — the R3000A's record

*Started 2026-10-11, at Diamond's stage 1.* `blackmoon-r3000`, the R3000A, the MIPS I processor of the PlayStation's
CPU, and later the PlayStation 2's I/O processor (Sapphire). Planned in `Diamond_Plan.md` §9.1 and §7's stage 2; the
common interface is `README.md` beside this page. A dispute about the processor is kept here, not in
`Diamond_Disputes.md`, since it belongs to every core that uses the crate (`Diamond_Plan.md` §1.4).

## 1. Stage 1 (2026-10-11)

The bus trait, the registers with both delay slots, the state block and the step's signature (`NotBuilt` while
`BUILT` is false), and the single-step harness with its controls. No instruction is implemented. Written from IDT's
*R30xx Family Software Reference Manual* and psx-spx's "CPU Specifications" (pinned by `Diamond_Plan.md` §2.1) as
names and layout only; no emulator's source and no firmware was read.

## 2. The interface

- **`Bus`**: `fetch` (one instruction word), `read` and `write` by `Size` (byte, halfword, word; the value in the low
  bits), `idle` (cycles inside the processor: the HI/LO interlock, a coprocessor 2 interlock), `interrupt` (the
  external request as Cause's IP2 sees it), and coprocessor 2's `cop2_data`, `set_cop2_data`, `cop2_control`,
  `set_cop2_control` and `cop2_command` (README, "A coprocessor").
- **What the bus is given, and what the processor keeps.** The trait says the bus meets what the processor does not
  answer itself, and leaves open where the 4 KiB I-cache, the 1 KiB scratchpad and the cache control register at
  `FFFE0130h` sit [C, open]: inside the crate, configured by its core, or in Diamond's bus. The PlayStation 2's I/O
  processor is said to differ in its caches [A, unread], so the choice is made at Diamond's stage 3 with the timing
  model of `Diamond_Plan.md` §5.2, when the cost of each is known, and recorded here.
- **`Registers`**: R0–R31, HI, LO, the PC, coprocessor 0's SR, CAUSE, EPC, BadVaddr, TAR, DCIC, BPC, BDA, BPCM and BDAM
  (psx-spx "COP0 Register Summary"; PRID is a constant and is not held), and the two delays the R3000A makes
  architectural: **`BranchDelay`** (whether the next instruction is a delay slot, whether the branch is taken, its
  target) and **`LoadDelay`** (the register a load writes after the next instruction, if any, and its value).
- **`DEBUG_REGISTERS`**: the 45 names a debugger lists, in `debug_registers`' order: the general registers by their
  conventional names (psx-spx "CPU Registers": `zero`, `at`, `v0`… `fp`, `ra`), `hi`, `lo`, `pc`, then coprocessor 0's
  ten. Diamond's machine info lists them for processor 0 from stage 1.

## 3. The oracle

### 3.1 The single-step harness

`src/singlestep.rs`, over SingleStepTests' r3000 suite at `51e2274` (MIT; 55 files of 1,000 cases, 55,000 in all),
found through `EMUSEN_BLACKMOON_CORPUS` as `r3000/v1/*.json`. The suite ships its cases as `*.json.bin`; its own
`transcode_json.py` writes the JSON the harness reads, as its README directs. **Read:** the README and that script,
which is the data format's definition; its generator, a fork of an emulator's interpreter, was neither fetched nor
read (`Diamond_Plan.md` §1.3). Provenance and every file's MD5 are in `~/.cache/emusen/probe/diamond/PROVENANCE.txt`.

**What a case holds**, read from the data on 2026-10-11 [M, a scan of all 55,000 cases]:

- `initial` and `final`: the 32 registers, `hi`, `lo`, `EPC`, `TAR`, `CAUSE`, `PC`, and `delay` with `branch`
  (`slot`, `take`, `target`) and `load` (`target`, the register, −1 for none; `val`). The README's example prints the
  two delays' names exchanged in its `initial`; the data and the script name them as above, and the example's own
  `final` agrees with them.
- `opcode` and `opcode_addr`: in every case the instruction at the initial PC, and the first cycle is its fetch.
- `cycles`: one to three, each `actions` (1 read, 2 write, 4 fetch; no other value occurs), `sz` (1, 2 or 4), `addr`
  and `val`, every value within 32 bits. There is no memory list: what a read returns is the cycle's `val`. The
  shapes are a fetch alone (most cases), a fetch and one read or write, and LWL/LWR's fetch with a halfword and a byte
  read.
- Exceptions occur although the README says they are disabled: all of SYSCALL's and BREAK's cases end at `80000080h`
  with CAUSE and EPC changed, and so do the halfword and word loads and stores whose address is misaligned, after the
  fetch alone (LW 768, LH 486, LHU 491, SW 732 and SH 507 cases, the last in the file `SHL.json`; in each file exactly
  the cases with a misaligned address). The harness grades what the suite records; the exception model is stage 2's
  to write from the manuals.

**The bus** answers each fetch and read with the value of the first unanswered recorded cycle of the same kind,
address and width, and 0 for any other, and records every access as a cycle. A case is graded on the recorded state
(`Recorded`: exactly the fields above, so registers the suite does not record, SR among them, are not compared) and on
the cycle list, kind, width, address and value, in order.

**The controls (measured 2026-10-11, `cargo test --release`, the desktop).**

- *Positive:* a fake processor that performs each case's own cycles through the bus, in order, and ends in the case's
  own final state: **55,000 of 55,000** pass, the whole suite read and graded in 0.1 s on four threads.
- *Negative:* the same replay with HI's lowest bit flipped in its final state, and with its last cycle dropped, over
  every case: **0 of 55,000** pass for each. Every case has at least its fetch, so dropping a cycle is always seen.
- *The unbuilt processor* fails the inline case; its run over the suite is skipped while `BUILT` is false.

### 3.2 The independent checks

The suite is generated from an emulator's interpreter, so it is a primary oracle with independent checks beside it
(`Diamond_Plan.md` §11 Q8): amidog's `psxtest_cpu` and `psxtest_cpx`, ps1-tests' `cpu/*`, each disagreement a
dispute here. They need the bus and the kernel, so they run from Diamond's stage 3.

## 4. The state block

`impl State for R3000`: `R` (32 words), `Hi`, `Lo`, `Pc`, `Cop0` (ten words in the order of §2), then `Branch.Slot`,
`Branch.Taken`, `Branch.Target`, `Load.Pending`, `Load.Register`, `Load.Value`: 192 bytes, the layout pinned by
`the_layout_is_pinned_and_a_block_round_trips`. A short block is refused and changes nothing. Diamond's state carries
it as `Cpu`.

## 5. What stage 2 owes

The instruction set with both delay slots, COP0 and RFE, the exceptions and interrupts, MULT/DIV with their latency
through `idle`, the observed step, the disassembler and assembler through `emusen-mips` (`Diamond_Plan.md` §9.1), and
the suite's every case on registers, delays and cycles, but cases named in a dispute entry (§7's stage 2, G1).
