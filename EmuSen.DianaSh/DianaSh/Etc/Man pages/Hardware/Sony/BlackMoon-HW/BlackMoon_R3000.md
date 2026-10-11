# BlackMoon_R3000 — the R3000A's record

*Started 2026-10-11, at Diamond's stage 1.* `blackmoon-r3000`, the R3000A, the MIPS I processor of the PlayStation's
CPU, and later the PlayStation 2's I/O processor (Sapphire). Planned in `Diamond_Plan.md` §9.1 and §7's stage 2; the
common interface is `README.md` beside this page. A dispute about the processor is kept here (§8), not in
`Diamond_Disputes.md`, since it belongs to every core that uses the crate (`Diamond_Plan.md` §1.4).

The marks are the plan's: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44, Rust 1.98.1) with
the tool named; **[R]** read from a source, named; **[A]** argued, with no measurement behind it; **[C]** a choice.

## 1. Stage 1 (2026-10-11)

The bus trait, the registers with both delay slots, the state block and the step's signature (`NotBuilt` while
`BUILT` was false), and the single-step harness with its controls. No instruction was implemented. Written from IDT's
*R30xx Family Software Reference Manual* and psx-spx's "CPU Specifications" (pinned by `Diamond_Plan.md` §2.1) as
names and layout only; no emulator's source and no firmware was read.

## 2. The interface

- **`Bus`**: `fetch` (one instruction word); `read` and `write` by `Size`; `isolated_read` and `isolated_write`, which
  a load or store reaches instead while SR's IsC isolates the cache; `idle` (cycles inside the processor: the HI/LO
  interlock); `cycles`, the bus's clock, which the processor only reads; `interrupt` (the external request as
  Cause's IP2 sees it); and coprocessor 2's `cop2_data`, `set_cop2_data`, `cop2_control`, `set_cop2_control` and
  `cop2_command` (README, "A coprocessor"). Stage 2 added the isolated calls, the clock and the three-byte size.
  *Stage 3 (2026-10-11)* added **`bus_error`**, a default method answering false: after a fetch the processor asks it
  and takes IBE (ExcCode 6), after a load or store and takes DBE (7), EPC at the instruction, a faulting load landing
  nothing and a faulting store's bytes not reported to the observer (psx-spx "Memory Exceptions": "Bus Error ------>
  Unused Memory Regions"). The suite's flat bus never faults, so its 55,000 cases are unchanged [M].
- **`Size`**: `Byte`, `Half`, `Tribyte` and `Word`. A read's value is in the low bits and anything above them is
  ignored. A write carries the register as the processor drives it, the datum in the low `size` bytes: a byte or
  halfword store passes the whole register, which psx-spx's "Caution - 8/16-bit writes to certain IO registers" says
  reaches the bus, and which the suite records (§3.1). `Tribyte` is LWL's, LWR's, SWL's or SWR's three bytes within one
  word, one access by the manuals (D-1, §8). How the byte lanes meet a device is the bus's.
- **What the bus is given, and what the processor keeps.** The trait says the bus meets what the processor does not
  answer itself, and leaves open where the 4 KiB I-cache, the 1 KiB scratchpad and the cache control register at
  `FFFE0130h` sit [C, open]: inside the crate, configured by its core, or in Diamond's bus. The isolated calls serve
  either answer: the processor says that an access is isolated, and whoever holds the cache answers it. The
  PlayStation 2's I/O processor is said to differ in its caches [A, unread], so the choice is made at Diamond's stage
  3 with the timing model of `Diamond_Plan.md` §5.2, when the cost of each is known, and recorded here. *Decided at
  stage 3*: **in Diamond's bus** (`Diamond_Native.md` §11.4), the refill's cost being the region's and the scratchpad
  a region of the map; the crate keeps no cache.
- **`Registers`**: R0–R31, HI, LO, the PC, coprocessor 0's SR, CAUSE, EPC, BadVaddr, TAR, DCIC, BPC, BDA, BPCM and BDAM
  (psx-spx "COP0 Register Summary"), and the two delays the R3000A makes architectural: **`BranchDelay`** (whether the
  next instruction is a delay slot, whether the branch is taken, its target) and **`LoadDelay`** (the register a load
  writes after the next instruction, if any, and its value). PRId is a constant of the chip, `R3000::prid`, the
  PlayStation's `0x00000002` by default (psx-spx "cop0r15"), which another core sets for its own chip.
- **`R3000`**: the registers, `hilo_ready` (the bus cycle at which the multiplier's result can be read, §5.5), PRId,
  and the processor number and store space the observer is told. **`step`** is `step_observed` with `Unobserved`;
  it returns `Instruction`, `Exception(code)` with the Cause code (0 for an interrupt), or `Observed(reasons)` when the
  observer stopped it before the instruction ran, nothing having run.
- **`DEBUG_REGISTERS`**: the 45 names a debugger lists, in `debug_registers`' order: the general registers by their
  conventional names, `hi`, `lo`, `pc`, then coprocessor 0's ten. A test holds the first 32 equal to `emusen-mips`'s
  `GPR_NAMES` and the last ten equal to `COP0`'s names for those registers.
- **`COP0`** and **`dialect`**: coprocessor 0's table for `emusen-mips` (its registers' names and its commands, RFE and
  the four TLB commands the PlayStation reserves), and the R3000A's dialect, coprocessor 0 by that table and
  coprocessor 2 by the table its core gives, the GTE's on the PlayStation (`blackmoon-gte`'s `COPROCESSOR`). The two
  crates stay independent of each other; the core joins them.

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
  and `val`, every value within 32 bits. There is no memory list: what a read returns is the cycle's `val`, whose
  bits above the access's width are random and are not the datum (an LB's result is the low byte of its read's `val`
  sign-extended). A write's `val` is the register as driven: SB and SH record the whole register, SWL the register
  shifted right to its datum. The shapes are a fetch alone (most cases), a fetch and one read or write, and the
  three-byte LWL, LWR, SWL and SWR as a fetch, a halfword and a byte (D-1).
- **The unused fields hold random bits**: SPECIAL's sa or rs where the instruction does not use them, LUI's rs, BLEZ's
  and BGTZ's rt, SYSCALL's and BREAK's code, in 960 to 1,000 cases of each file concerned; and BCondZ's rt takes all 32
  values. The suite is therefore also a test of the R3000's partial decode (`EmuSen_Mips.md` §2.2).
- **Exceptions occur although the README says they are disabled**: 5,468 cases end at `80000080h` with CAUSE and EPC
  changed: all of SYSCALL's and BREAK's, ADD's 248 and SUB's 236 overflows, and the halfword and word loads and stores
  whose address is misaligned, after the fetch alone (LW 768, LH 486, LHU 491, SW 732 and SH 507 cases, the last in
  the file `SHL.json`). 3,377 cases begin in a delay slot, all of a taken branch; 13,621 begin with a load pending.

**The bus** answers each fetch and read with the value of the first unanswered recorded cycle of the same kind,
address and width, and 0 for any other, and records every access as a cycle. It counts a cycle for each access and
for each idle cycle, and holds the interrupt line where the case's initial CAUSE has IP2, so that Cause's IP2, which
follows the line, keeps the value the suite gave it (the suite does not record SR, which the harness leaves zero, so no
interrupt is enabled). A case is graded on the recorded state (`Recorded`: exactly the fields above) and on the cycle
list, kind, width, address and value, in order; a case a dispute names is graded in the dispute's form (§8).

**The controls (measured 2026-10-11, `cargo test --release`, the desktop).**

- *Positive:* a fake processor that performs each case's own cycles through the bus, in order, and ends in the case's
  own final state: **55,000 of 55,000** pass. Since stage 2 it performs the cycles in the dispute's form where a case
  is named, so that it controls the harness the processor is graded by.
- *Negative:* the same replay with HI's lowest bit flipped in its final state, and with its last cycle dropped, over
  every case: **0 of 55,000** pass for each.
- *Mutations of the processor* [M, by hand on 2026-10-11, each reverted]: ADDU subtracting fails 940 ADDU cases; a
  pending load that lands although the instruction loads the same register fails 36 load cases; LWL and LWR merging
  with the register instead of the pending load fails 7; an exception writing CE as zero fails 2,984 cases of LH,
  LHU, LW, SH and SW. The harness sees each.

### 3.2 The independent checks

The suite is generated from an emulator's interpreter, so it is a primary oracle with independent checks beside it
(`Diamond_Plan.md` §11 Q8): amidog's `psxtest_cpu` and `psxtest_cpx`, ps1-tests' `cpu/*`, each disagreement a
dispute here. They need the bus and the kernel, so they run from Diamond's stage 3. ps1-tests' `cpu/cop` was read at
stage 2 for its source and its console log, which settled D-3 (§8).

## 4. The state block

`impl State for R3000`: `R` (32 words), `Hi`, `Lo`, `Pc`, `Cop0` (ten words in the order of §2), then `Branch.Slot`,
`Branch.Taken`, `Branch.Target`, `Load.Pending`, `Load.Register`, `Load.Value`, and since stage 2 `HiLoReady`, a u64:
**200 bytes**, the layout pinned by `the_layout_is_pinned_and_a_block_round_trips`. A short block is refused and
changes nothing. Diamond's state carries it as `Cpu`; no player holds a Diamond state, so its version 1 was redefined
rather than raised (`Diamond_Native.md` §2.7). `HiLoReady` is an absolute time on the bus's clock, so it is valid only
beside the bus's own clock in the same state, which Diamond's bus will write from stage 3.

## 5. Stage 2: the instructions and the step (2026-10-11)

`src/exec.rs`, from IDT's manual (Appendix A for each instruction's operation; Chapters 3 and 4 for coprocessor 0 and
exceptions; Chapter 13's Table 13.1 for the latencies) and psx-spx's "CPU Specifications". The decode is
`emusen-mips`'s (`EmuSen_Mips.md` §2); the execution is this crate's alone, and MarsRT's VR4300 was not read or
copied (`Diamond_Plan.md` §9.1: execution is not shared).

### 5.1 The step

One call is one instruction, or the exception or interrupt taken in its place, in this order:

1. Cause's IP2 is set from the bus's interrupt line, which the IDT manual says it follows ("These bits follow the CPU
   inputs").
2. **An interrupt**, if SR's IEc is set and an IM bit meets an IP bit (the two software bits included). The interrupted
   instruction is fetched, and if it is a GTE command (`COP2 imm25`) with CU2 set, the command is passed to the bus
   before the exception is taken, EPC still pointing at it: psx-spx's "Interrupts vs GTE Commands", which several games
   depend on [R]. That the fetch happens for every interrupt, not only for a GTE command, is [A]: the instruction is in
   the pipeline when the interrupt is recognised, and the GTE rule requires it to have been fetched.
3. The observer's `before`, which may stop the step with nothing run.
4. A fetch address error: a misaligned PC, or kernel space in user mode, raises AdEL with BadVaddr and EPC at the PC,
   and nothing is fetched. A jump to a misaligned address therefore faults at the target, not at the jump (psx-spx,
   "jumps and branches").
5. The debug unit's PC breakpoint (§6.3), before the fetch.
6. The fetch, and the instruction.

### 5.2 The branch delay

A taken branch or jump leaves `BranchDelay { slot, taken, target }`; the next step executes the delay slot and then
continues at the target. A conditional branch records its target whether taken or not, as the suite does. **A
branch's target is counted from the address of the instruction that follows it**, which is the manual's definition
("the instruction 'offset' words away from the 'delay slot' instruction after the branch"), and so is a jump's upper
four bits and a link's return address (the delay slot's address plus four). Where the branch is itself in a delay
slot, the instruction that follows it is the earlier branch's target, and the suite agrees in all of its such cases
[M: the 65 BEQ, 73 BNE, 64 BCondZ and 60 BLEZ cases in a slot count from the earlier target; 118 J and JAL cases take
the target's upper bits; JALR in a slot links the earlier target plus four]. BLTZAL and BGEZAL link whether or not
they branch, comparing rs before linking (psx-spx).

### 5.3 The load delay

The load's value is written after the next instruction has read its operands. The rules for the corners:

- **An instruction's own write to the register a load is pending for wins**: the load lands first and the
  instruction's result over it. The pipeline argues for it [A: the later instruction writes back a cycle later], and
  the suite agrees in all of its such cases [M].
- **A load of the register a load is pending for cancels the first**, which never lands; the register keeps its old
  value until the second lands. The manuals do not say; the suite does it in every case [M]. Whether the hardware's
  bypass could hand the first value to the instruction after the second load is not known to this page [A, open]; the
  ps1-tests and amidog programs at stage 3 are where it could show.
- **LWL and LWR merge with the pending value** of their register, so that the pair runs with no delay between them,
  as both the IDT manual ("special hardware allows the lwl, lwr pair to be used in consecutive instructions") and
  psx-spx say; a load after the pair is delayed as any other.
- **An exception lets the pending load land** before its entry, as psx-spx says of an interrupt ("the load would
  complete during IRQ handling"), and the suite does in every exception case with a load pending [M].
- A load to R0 is recorded as pending and lands as nothing, as the suite records it in its 181 such cases [M].
- MFC0, MFC2 and CFC2 go through the same delay (psx-spx "Caution - Load Delay", both sections).

### 5.4 Exceptions

Entry (IDT manual Chapter 4; psx-spx "COP0 - Exception Handling"): the pending load lands; Cause's ExcCode, CE, BT and
BD are written and its other bits kept, IP and the software bits among them; EPC is the faulting instruction, or the
branch before it with BD set when it is in a delay slot; TAR takes the branch's target when BT is set (psx-spx
"cop0r6"); the mode stack is pushed (KUc and IEc to KUp and IEp, those to KUo and IEo, the current pair cleared); the
branch and load delays are cleared; and execution continues at `80000080h`, or `BFC00180h` with BEV set. Address
errors set BadVaddr, and only they (psx-spx "cop0r8"). The vectors of the debug unit are `80000040h` and `BFC00140h`.

**CE is written on every exception from the instruction's bits 27–26** [R the suite; A]. The IDT manual defines CE
only for a coprocessor-unusable exception ("this field has the co-processor number from that instruction") and
psx-spx likewise; neither says what other exceptions leave there. The suite writes the instruction's bits 27–26 on
every exception (1 for LH, LHU and SH, 3 for LW and SW, 0 for SPECIAL), which is what a CE field loaded from those bits
unconditionally would show, and is consistent with both documents. It is followed as the only evidence; a fetch
address error, which has no instruction, writes 0.

What raises what: ADD, ADDI and SUB on signed overflow (Ov), leaving the destination unchanged; LH, LHU, SH misaligned
to two, LW, SW, LWC2 and SWC2 to four, and any data access to kernel space in user mode (AdEL, AdES); SYSCALL (Sys);
BREAK (Bp); an encoding psx-spx marks N/A (RI); and the coprocessor rules of §6.2. Integer division never traps; its
results by zero and for `−80000000h / −1` are psx-spx's table, which the suite agrees with in all 2,000 DIV and DIVU
cases [M].

### 5.5 The timing model

The processor keeps no clock; every cycle it causes reaches the bus as a call, and the bus counts them with its own
wait states (`README.md`). What the processor reports:

- **One fetch per instruction**, through `fetch`; whether it hits the I-cache, misses it or is uncached is the bus's.
- **Each data access** through `read` or `write`, after the fetch, at its width; LWL, LWR, SWL and SWR as one access of
  one to four bytes (D-1). Whether a store waits for the write buffer is the bus's.
- **No cycle for a branch** (the delay slot is the branch's cost), **none for the load delay** (it is architectural,
  not a stall), and none of its own for an exception: the handler's first fetch is the next cycle [A].
- **The multiplier's interlock**, the only cycles the processor itself spends. MULT, MULTU, DIV and DIVU write HI and
  LO at once and note `hilo_ready`, the bus's clock after the instruction's fetch plus the operation's latency; MFHI
  and MFLO, after their own fetch, call `idle` for the difference while the clock is short of it. The latencies are
  psx-spx's "Multiply/divide" table, measured on the PlayStation [R]: MULTU 6 cycles for rs below `800h`, 9 below
  `100000h`, 13 otherwise; MULT the same by the magnitude of a signed rs; DIV and DIVU 36. The IDT manual gives the
  R30xx family 12 and 35 (Chapter 2), or a result 11 and 35 instructions late (Table 13.1); D-2 (§8) records the
  disagreement. With this reading, MFLO straight after a short MULTU idles five cycles, and after five or more
  one-cycle instructions none [M, `the_multiplier_holds_mfhi_and_mflo_by_rs`]; psx-spx's own example ("one can
  insert up to six (cached) ALU opcodes ... without additional slowdown") fits it and fits one more cycle of latency
  equally, so the boundary is the reading's and stage 3 measures it (§9).
- **Coprocessor 2's interlock is the core's**: the GTE returns a command's cycles and the core holds the next
  coprocessor 2 access through the bus's calls (`README.md`, "A coprocessor"). The processor does not count it.
  *Since Diamond's stage 5 (2026-10-11)*: Diamond holds all five calls, the writes included, and the processor was not
  changed for it (`BlackMoon_GTE.md` §2, §6, D-13).

Not modelled, each [C, open until a test program shows it]: a second MULT or DIV issued while one is busy restarts the
unit rather than waiting; MTHI and MTLO while it is busy write at once; psx-spx's note that HI and LO should not be
touched "in the next 2 cycles or so" after an access, which it calls not understood; the store delay on SR's CU2
("cop2 isn't actually enabled until after 2 clock cycles or so").

## 6. Coprocessor 0 and the coprocessors

`src/cop0.rs`; `src/exec.rs`'s `coprocessor`.

### 6.1 The registers

MFC0 reads BPC, BDA, TAR, DCIC, BadVaddr, BDAM, BPCM, SR, CAUSE, EPC and PRId; registers 0–2, 4 and 10 raise RI
(psx-spx "cop0r0..r2, cop0r4, cop0r10"); 16–31 read 0 [C: psx-spx calls them "garbage" whose value follows the last
valid read and then drifts, which it does not explain; open]. MTC0 writes BPC, BDA, BDAM, BPCM; DCIC's R/W bits
(`FF80F03Fh`, psx-spx's table); SR's mode stack, IM, IsC, SwC, PZ, BEV, RE and the four CU bits (`F247FF3Fh`), its
status and zero bits left; CAUSE's two software bits only; and ignores TAR, BadVaddr, EPC, PRId and the absent
registers. CFC0 and CTC0 raise RI (psx-spx: the control registers "aren't used in any MIPS processors"). RFE pops the
mode stack, KUo and IEo left as they were. SwC is held and has no effect, as psx-spx reports of the hardware.

### 6.2 Usability

- Coprocessor 0's MFC0, MTC0, CFC0, CTC0 and COP0 run in kernel mode, or in user mode with CU0 set; otherwise CpU
  (IDT manual, CU0). MFC0 of a garbage register runs in user mode regardless (psx-spx "cop0r16-r31").
- COP0's commands by bits 0–5: `10h` is RFE; TLBR, TLBWI, TLBWR and TLBP raise RI (psx-spx "cop0cmd"); any other
  does nothing, as ps1-tests' `cpu/cop` shows on a console for `COP0 1E00000h` ("cop0_invalid", no exception) [R its
  `psx.log`].
- **LWCz, SWCz and BCzF/BCzT check CUz in every mode**, coprocessor 0 included: D-3 (§8).
- Coprocessor 2 runs with CU2 set, its transfers and commands through the bus. BC2F is always taken and BC2T never,
  the GTE's flag reading false (psx-spx "GTE Operation": "jump never ... jump always"); BC0x, BC1x and BC3x the same
  with their CU bit set [A].
- Coprocessors 1 and 3, absent on the PlayStation, run with their CU bit set and do nothing: MFC and CFC load 0 [C],
  LWC reads its word and discards it [A: the processor addresses memory for a coprocessor load], SWC writes nothing
  [C: no coprocessor drives the data]; ps1-tests' `cpu/cop` shows that COP1 and COP3 raise nothing with CU1 or CU3 set
  and CpU without [R].

### 6.3 Cache isolation and the debug unit

- **IsC** sends every load and store to `isolated_read` and `isolated_write`, never to memory, and the observer sees no
  stored byte (the IDT manual's SR, IsC: "makes all loads and stores access only the data cache, and never memory";
  psx-spx: on the PlayStation, the I-cache by the BIU's setting). Fetches are unaffected.
- **The debug unit** (psx-spx "COP0 - Debug Registers"): with DCIC's DE and PCE set and KD or UD for the current mode,
  a PC matching BPC under BPCM sets DB and PC; with DE, DAE and DR or DW, a load or store address matching BDA under
  BDAM sets DB, DA and R or W. With TR set either becomes an exception, ExcCode 9, at `80000040h` (or `BFC00140h`),
  taken before the fetch or before the access [A: psx-spx says where it jumps, not when in the instruction].
  Without TR only the status bits are set. Trace (TE) and the jump redirection bits 12–13 are not modelled; psx-spx
  describes the first in one line and calls the second "more or less unusable".

## 7. The project's own cases

`src/boundary.rs`, 17 cases against a 1 MiB RAM bus that counts cycles, each program written with `emusen-mips`'s
assembler: an interrupt at the boundary with the stack pushed, the pending load landed and CE from the interrupted
instruction; a masked or disabled interrupt not taken while IP2 follows the line; a software interrupt from Cause and
an interrupt in a delay slot (BD, BT, TAR); the GTE command meeting an interrupt; RFE; BEV; address errors on a
fetch, a load and a store in user mode; reserved and unusable instructions with their CE; coprocessor 0 in user mode;
MFC0 through the load delay and MTC0's writable bits; BC2F and BC2T; an overflow in a delay slot; the multiplier's
interlock by rs and DIV's; cache isolation; the debug unit's PC and data breakpoints; the load delay's corners in
sequence; and the observer (its stop, each stored byte with its space and the storing instruction's address, a call,
a return, an exception's entry as `kind::BRK` and an interrupt's as `kind::IRQ`). They grade the readings of §5 and
§6 against themselves, not against the hardware: the hardware's word on them is stage 3's, through amidog's and
ps1-tests' programs.

## 8. The disputes log

In the shape of `Diamond_Disputes.md`. Each entry's evidence and rule; the processor implements the rule.

### D-1. LWL, LWR, SWL and SWR moving three bytes make one access of three byte lanes, not a halfword and a byte

- **Opened:** 2026-10-11, at stage 2, by the harness: the suite records each three-byte form as two cycles, a
  halfword and a byte at the next address (LWL and SWL at offset 2: the halfword at the word's address, then the byte at
  +2; LWR and SWR at offset 1: the byte at +1, then the halfword at +2).
- **Documents read:** IDT, *R30xx Family Software Reference Manual*, rev. 1.0, Chapter 2, "Unaligned loads and
  stores": LWL and LWR "load 1, 2, 3 or 4 bytes from within one aligned 4-byte (word) location". psx-spx `6d7d1bc`,
  "Unaligned Load/Store (Details)": "The CPU has four separate byte-access signals, so, within a 32bit location, it
  can transfer all fragments of Rt at once (including for odd 24bit amounts)." Settled here, at the first rung: both
  documents give one access; the suite's two cycles are its generator's bus interface, which has no three-byte width.
- **Test program:** none needed.
- **Referee:** not consulted. **Witnesses:** none.
- **Conclusion:** one access of `Size::Tribyte` at the lowest of the three bytes' addresses, its datum the three bytes.
  The harness grades the 1,022 cases concerned (LWL 260, LWR 265, SWL 235, SWR 262) with the suite's two cycles joined
  into that access, its value the two values' data joined, and compares a three-byte access by its three bytes; every
  other field, the registers included, is compared as recorded. Argued from the documents.
- **Depends on it:** Diamond's bus (stage 3) decides what a three-byte access does to each region; RAM takes the three
  bytes.
- **Pinned by:** `the_processor_against_the_suite`, and `d1_cases_differ_from_the_suite_only_in_the_split_access`,
  which shows that every one of the 1,022 fails as recorded and passes joined.
- **Implemented in:** `blackmoon-r3000: the R3000A, every case of the r3000 suite passing, one dispute named`.

### D-2. The multiplier's latencies: psx-spx's PlayStation table, not the IDT family's figures

- **Opened:** 2026-10-11, at stage 2, by the documents disagreeing.
- **Documents read:** IDT, *R30xx Family Software Reference Manual*, Chapter 2, "Integer multiply unit and
  registers": "In the R30xx family, multiply operations take 12 clocks and division takes 35." IDT, *MIPS
  Microprocessor Family Software Reference Manual*, 1996, Chapter 13, Table 13.1: `mult, multu (R30xx)` a result 11
  late, `div, divu (R30xx)` 35. psx-spx `6d7d1bc`, "Multiply/divide": MULTU 6, 9 or 13 cycles by rs, MULT the same by
  rs's magnitude, DIV and DIVU 36 "no matter of rs and rt values". LSI's *CW33000 MIPS Embedded Processor User's
  Manual* (1992), searched for the multiplier's timing: none given.
- **The argument:** the PlayStation's CPU is an LSI core, not one of IDT's parts, so IDT's figures describe another
  implementation of the same architecture; psx-spx's table is the only one stated for the PlayStation, and its
  dependence on rs is the signature of an early-out multiplier, which IDT's fixed figure is not. psx-spx's table has
  one defect of its own: its MULT ranges end at `FFFFF801h` and `FFF00001h`, each overlapping the next range's start;
  the symmetric reading by the magnitude of rs, `FFF00000h..FFFFF7FFh` for 9 cycles and `80000000h..FFEFFFFFh` for 13,
  is taken.
- **Test program:** ~~none yet.~~ *Stage 3, 2026-10-11*: `programs/multiply.s` in Diamond's crate, assembled with
  `emusen-mips`: each case times MULT, MULTU, DIV or DIVU of its operands and MFLO at once, against MFLO alone, by
  timer 2 on the system clock from the I-cache with interrupts off, keeping a second pass; a second table times MULTU
  of a small rs, N NOPs and MFLO against the NOPs and MFLO, N = 0..15. Results in RAM (`Diamond_Native.md` §11.13).
  - *On Diamond* [M]: MULTU 6 cycles for rs 0 and 7FFh, 9 for 800h and FFFFFh, 13 for 100000h and FFFFFFFFh; MULT 6
    for 7FFh and F800h (−2048), 9 for 800h, FFFFF7FFh, FFFFFh and FFF00000h, 13 for 100000h, FFEFFFFFh and 80000000h;
    MULTU of 1 by FFFFFFFFh 6 and MULT of 3 by 80000000h 6 (rt does not count); DIV and DIVU 36 for every case, by
    zero and 80000000h / −1 included. The NOP table: 6, 5, 4, 3, 2, 1, then 1 from N = 5 (the multiply's own cycle).
    This is the reading of §5.5 reproduced, as it must be; it says nothing of the hardware.
  - *Beetle PSX on the player's BIOS* [M, RAM only]: every case 2 cycles more (8, 11, 15; 38 for a division); the NOP
    table 8, 7, 6, 5, 4, 3, then 1 from N = 6. *SwanStation*: 1 for every case and every N, no latency modelled.
    *PCSX-ReARMed*: 0 everywhere, its timer not advancing within the measurement. The witnesses disagree among
    themselves; the one that models a latency agrees with psx-spx's dependence on rs and differs from §5.5's boundary
    by two cycles, which is the size of this reading's uncertainty about where the latency is counted from.
- **Conclusion:** psx-spx's table, with the boundary of §5.5 (the result readable when the bus's clock after MFLO's
  fetch reaches MULT's fetch plus the latency). **Open**: argued, and witnessed only by emulators that disagree; ~~it
  stays open until `multiply.s` runs on the tester's console (`Diamond_Plan.md` §11 Q16), whose two tables decide both
  the latencies and the boundary~~ (decided 2026-10-11: no console is used for any dispute).
- **The referee step, 2026-10-11** (`Diamond_Referees.md` §4.2) [M]: `multiply.s` on DuckStation and PCSX-Redux as
  well. DuckStation reads 5, 8, 12 and 35, MFLO free after four instructions; Beetle PSX 8, 11, 15 and 38, free after
  six, a stall costing it two cycles more than the wait; PCSX-Redux 8, 11, 15 and 38 in its own clock, where a NOP
  costs two; SwanStation 1 throughout on its interpreter too; PCSX-ReARMed's interpreter 7 to 16 with no classes by
  rs. **Confirmed by agreement between three lineages** (DuckStation, Mednafen, PCSX): the table's shape, the class
  steps of 3 and 4 cycles by rs or its magnitude, rt irrelevant, and division 30 above the smallest class whatever its
  operands. **Still open**: the offset (where the latency is counted from) and the boundary, on which the referees
  differ by up to three cycles and no output ranks them; Diamond keeps §5.5's.
- **Depends on it:** the cycle at which MFHI and MFLO complete, so every timing measurement downstream of a multiply.
- **Pinned by:** `the_multiplier_holds_mfhi_and_mflo_by_rs`; Diamond's `the_multiplier_takes_psx_spxs_cycles`.
- **Implemented in:** the same commit as D-1.

### D-3. LWC0 and SWC0 (and every LWCz, SWCz and BCz) raise CpU only when CUz is clear, in kernel mode too; psx-spx's "always CpU" does not hold

- **Opened:** 2026-10-11, at stage 2: psx-spx ("jf/jt cop0flg ... mov [mem],cop0reg / mov cop0reg,[mem]") says these
  coprocessor 0 forms are "Not supported by the CPU. Trying to execute these opcodes causes a Coprocessor Unusable
  Exception", while ps1-tests' `cpu/cop` expects no exception from SWC0 with CU0 set.
- **Documents read:** psx-spx `6d7d1bc`, "COP0 - Misc"; the IDT manual's SR, CU0 ("The CPU control instructions encoded
  as 'co-processor 0' type are always usable in kernel mode, regardless of the setting of this bit"), which speaks of
  the control instructions and not of loads, stores or branches.
- **Test program:** ps1-tests `cpu/cop` at `f727802` (`main.cpp`, read for what it expects, and `psx.log`, its output on
  a console): `testSwc0Disabled` (CU0 clear, kernel mode, as the kernel runs PSn00bSDK programs) expects an exception
  and `testSwc0Enabled` (CU0 set) expects none; the console passes both, and the same pairs for SWC2, SWC3, COP1 and
  COP3. Not run here; its console log is the evidence.
- **Conclusion:** LWCz, SWCz, BCzF and BCzT raise CpU exactly when SR's CUz is clear, whatever the mode; psx-spx's
  statement describes them with CU0 clear. What SWC0 stores, and whether LWC0 reads, the log does not say (§6.2).
  Measured by a console, at the second rung.
- **Depends on it:** nothing in the kernel's usual paths; a program probing coprocessor 0.
- **Pinned by:** `reserved_and_unusable_instructions`.
- **Implemented in:** the same commit as D-1.

## 9. The gate (G1's first half)

`Diamond_Plan.md` §7's stage 2 row and §8's G1: SingleStepTests r3000, every case, on registers, HI/LO, the PC, both
delay states and every cycle's fetch, read and write, but cases named in a disputes entry. **Measured 2026-10-11,
`cargo test --release` with `EMUSEN_BLACKMOON_CORPUS`: 55,000 of 55,000**, 1,022 of them in D-1's form, in 0.4 s on
four threads.

| Group | Files | Cases | Passed | Of which in a dispute's form | Notes |
|---|---|---|---|---|---|
| ALU, registers | ADD, ADDU, SUB, SUBU, AND, OR, XOR, NOR, SLT, SLTU | 10,000 | 10,000 | 0 | 484 overflow traps |
| ALU, immediates | ADDI, ADDIU, SLTI, SLTIU, ANDI, ORI, XORI, LUI | 8,000 | 8,000 | 0 | no ADDI overflow occurs in the suite |
| Shifts | SLL, SRL, SRA, SLLV, SRLV, SRAV | 6,000 | 6,000 | 0 | |
| Multiply and divide | MULT, MULTU, DIV, DIVU, MFHI, MFLO, MTHI, MTLO | 8,000 | 8,000 | 0 | results only; the suite records no idle cycle |
| Branches and jumps | BEQ, BNE, BLEZ, BGTZ, BCondZ, J, JAL, JR, JALR | 9,000 | 9,000 | 0 | BCondZ's 32 rt values |
| Loads | LB, LBU, LH, LHU, LW, LWL, LWR | 7,000 | 7,000 | 525 (D-1) | 1,745 address errors |
| Stores | SB, SH, SW, SWL, SWR | 5,000 | 5,000 | 497 (D-1) | 1,239 address errors |
| System | SYSCALL, BREAK | 2,000 | 2,000 | 0 | 129 in a delay slot |
| **All** | 55 | **55,000** | **55,000** | **1,022** | |

What the gate does not cover, by the suite's construction: interrupts, coprocessor 0 beyond what an exception writes,
the coprocessors, SR (never recorded), BadVaddr (never recorded), user mode, the multiplier's timing, and any
instruction the suite has no file for (the coprocessor instructions, the reserved encodings). §7's cases hold the
readings for those against themselves; amidog's CPU and CPX tests and ps1-tests at stage 3 are their oracle. **P1**
(`Diamond_Plan.md` §13) is retired in `Diamond_Native.md` §10.

## 10. Open, and owed to stage 3

*Stage 3's answers, 2026-10-11* (`Diamond_Native.md` §11.13 and §11.14). Each item was checked by a program of the
project's own run on Diamond and on the three reference cores, their RAM read; amidog's CPU and CPX tests, the
independent check, report on screen and wait for the GPU (stage 4), and ps1-tests' `cpu/*` cover none of the three:

- **CE on every exception**: `exceptions.s` takes twelve exceptions (Sys, Bp, Ov from ADD and ADDI, AdEL from LW and
  LH, AdES from SH and SW, RI, CpU from SWC3, MFC2 and LWC2). Diamond writes the opcode's low two bits as CE on every
  one; Beetle PSX does the same on the eleven it takes (it raises no CpU for LWC2 with CU2 clear, against D-3's
  reading); SwanStation takes five (no address error on the misaligned accesses, no CpU for the three coprocessor
  forms); PCSX-ReARMed raises RI for ADD's overflow and stops. The one witness that takes the exceptions agrees on CE;
  still open on the hardware.
- **A second load cancelling the first**: `loads.s`. Diamond, Beetle PSX and SwanStation agree on all eight results:
  the first load never lands (the register keeps 55h), the instruction's own write wins, LWL and LWR merge with the
  pending value, and MFC0 through the delay cancels a pending load of its register the same way; PCSX-ReARMed lets
  the first load land (11111111h). Two of three witnesses with the suite; the bypass question of §5.3 is answered for
  the instruction after the second load (it sees the old value) on every witness that cancels.
- **The garbage registers 16–31**: `cop0-garbage.s`. Diamond reads 0; SwanStation and PCSX-ReARMed 0; Beetle PSX a
  pattern of period eight (10800h, 0, 304h, C1h, 304h, 1F800000h, 8, 4) repeated from 16 to 31, and 10800h after a
  delay and after EPC, which matches neither psx-spx's "follows the last valid read" nor zero. Open: no document gives
  values, and the witnesses disagree.

*Stage 4's answers, 2026-10-11* (`Diamond_GPU.md` §10). With a screen, amidog's tests run: **`psxtest_cpu` 1.3**
reaches "ALL DONE" with every test's exception, flag, value and timing columns OK or N/A, among them its "MEM DLY" and
"MEM ADV" groups, the "delayed and advanced load" tests its README names, for every load instruction, and its "EXC"
group (RFE, BREAK, SYSCALL). The screen gives a verdict per instruction, not the cases, so whether a second load of a
pending register is among them is not shown; the load delay's corners stand confirmed by an independent program as far
as its verdicts reach. **`psxtest_cpx` 1.1** draws its menu and runs only when Start is pressed, so CE on every
exception and the garbage registers wait on the pad (stage 6).

The items below stand as written at stage 2.

- Where the I-cache and scratchpad live (§2); the isolated calls' answers.
- D-2's measurement, and §5.5's unmodelled points.
- The garbage registers' values; SWC0's and LWC0's bus behaviour; the trace break.
- The load-after-load corner of §5.3 against amidog's "delayed and advanced load" tests, which its README names.
- The timing of a fetch on an interrupt (§5.1) against ps1-tests' `access-time` once the bus counts it.

## 11. What was read for stage 2

IDT's *R30xx Family Software Reference Manual* rev. 1.0 (Chapters 2–4, Appendix A); IDT's *MIPS Microprocessor
Family Software Reference Manual* (1996), Chapter 13's Table 13.1; LSI's *CW33000 User's Manual* (1992), Chapter 1 and
a search for the multiplier's timing; psx-spx `6d7d1bc`, "CPU Specifications" in full and "Geometry Transformation
Engine (GTE)" for the command encoding and summary; nocash's single-file psx-spx for the REGIMM duplicates; the
r3000 suite's README, `transcode_json.py` and data; ps1-tests' `cpu/cop` source and console log; amidog's
`psxtest_cpu.txt` and `psxtest_cpx.txt`. No emulator's source, no firmware and no firmware listing, no PSX_MiSTer
file, and no MarsRT code.
