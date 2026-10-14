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
  *Stage 13 of Diamond (2026-10-11)* added **`fetch_decoded`**, a default method giving `fetch`'s word with
  `emusen_mips::decode` of it, which the step now calls in place of the two. A bus that keeps decoded words may answer
  from them, provided what it answers is the fetch's word, time and fault and the word's decoding: Diamond's bus
  answers an I-cache hit from the word decoded at its refill (`Diamond_Speed.md` §5). It is an addition with a default,
  so a bus that does not give it is unchanged, and the suite's 55,000 cases pass through the default [M].
  *Diamond's second speed pass (2026-10-12)* added **`running`**, a default method answering false, and with it
  **`R3000::run(bus, limit)`**: up to `limit` steps, each as `step` makes it, ending after the first that is not an
  instruction or after which `running` says no; the steps taken and the last one's result. A bus answers true while
  nothing it must do between two steps is due (Diamond's: while the clock is short of the next event). The default
  makes a run one step, so a bus that does not give it is unchanged (§13).
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
  observer stopped it before the instruction ran, nothing having run. *Since 2026-10-11* the observer is also told
  each byte a load read, and **`load_ahead`** names the bytes the next instruction would load, for a debugger that
  stops in front of a load (§12).
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
valid read and then drifts, which it does not explain; open]. *Revised 2026-10-11 by the referee step of §10*: still
0 and no exception, now a choice the referees leave standing rather than one they confirm: none of five follows
psx-spx's description, and they answer in three ways. MTC0 writes BPC, BDA, BDAM, BPCM; DCIC's R/W bits
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
  LWC reads its word and discards it [A: the processor addresses memory for a coprocessor load], ~~SWC writes nothing
  [C: no coprocessor drives the data]~~ *since 2026-10-11* SWC0, SWC1 and SWC3 store a word of zero on the bus (D-21:
  the store is a console's, by amidog's expectations; its value is one referee's); ps1-tests' `cpu/cop` shows that COP1
  and COP3 raise nothing with CU1 or CU3 set and CpU without [R].

### 6.3 Cache isolation and the debug unit

- **IsC** sends every load and store to `isolated_read` and `isolated_write`, never to memory, and the observer sees no
  stored byte (the IDT manual's SR, IsC: "makes all loads and stores access only the data cache, and never memory";
  psx-spx: on the PlayStation, the I-cache by the BIU's setting). Fetches are unaffected.
- **The debug unit** (psx-spx "COP0 - Debug Registers"): with DCIC's DE and PCE set and KD or UD for the current mode,
  a PC matching BPC under BPCM sets DB and PC; with DE, DAE and DR or DW, a load or store address matching BDA under
  BDAM sets DB, DA and R or W. With TR set either becomes an exception, ExcCode 9, at `80000040h` (or `BFC00140h`),
  taken before the fetch or before the access [A: psx-spx says where it jumps, not when in the instruction].
  Without TR only the status bits are set. *Revised 2026-10-11 by D-20*: KD or UD is chosen by the address compared,
  KD when its bit 31 is set and UD when it is clear, not by the mode; the data address compared is the one the access
  puts on the bus, LWL's and SWL's aligned word; the comparison comes first and sets the status bits even where an
  address error then follows, the address error takes precedence over the break, and the break over a bus error. Trace (TE) and the jump redirection bits 12–13 are not modelled; psx-spx
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
ps1-tests' programs. *Since D-20's and D-21's second steps (2026-10-11)* the file holds 24 cases, among them the
debug unit's mode by the address, its bus address for LWL and SWL, its comparison before the address error, and
SWC0's, SWC1's and SWC3's store.

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

### D-17. An MTC0 to SR reaches the interrupt check one instruction late: the instruction after it runs before an interrupt it enables, and may still be interrupted after one it disables

- **Opened:** 2026-10-11, by amidog's `psxtest_cpx` INTR on Diamond with the pad (`Diamond_Pad.md` §5.1): eight cases
  in which `mtc0 cause` sets a software interrupt, a NOP follows and `mtc0 sr` enables it, want EPC at the second
  instruction after the MTC0 where Diamond gave the first.
- **Documents read:** the IDT R30xx manual, chapter 13, "Bitfields in CPU control registers": for the R30xx family
  "the enable won't allow an interrupt to affect ... the following two instructions; it can happen before the third";
  psx-spx is silent.
- **Test program:** `psxtest_cpx` 1.1, its cases disassembled from RAM (`mtc0 a1, cause; nop; mtc0 a0, sr; nop`, and
  the same with `mflo v0` after the MTC0, whose result the case checks): the console's wanted EPC is the MTC0's address
  plus 8, one instruction later than Diamond's first build and one earlier than the IDT figure.
- **Referee:** not consulted. **Witnesses:** none.
- **Conclusion:** the interrupt check at the boundary after an MTC0 to SR sees SR and Cause's software bits as they
  were before it, so one instruction runs; the PlayStation's CPU differs from the IDT family's two. A disabling MTC0
  takes effect as late by the same rule, which no case measures [C]. An MTC0 to Cause is seen at once, as before: the
  cases write Cause two instructions ahead and do not measure it. Measured by a console's expectations, at the second
  rung.
- **Depends on it:** the kernel's critical sections; the state block, 209 bytes with the held values.
- **Pinned by:** `an_interrupt_enabled_by_mtc0_sr_waits_one_instruction`.
- **Implemented in:** `blackmoon-r3000: an MTC0 to SR reaches the interrupt check an instruction late, TAR on a branch
  not taken, CE zero on DBE and the debug unit's breaks`.

### D-18. An exception in a delay slot sets TAR on every branch: the target when taken, the instruction after the slot when not

- **Opened:** 2026-10-11, by `psxtest_cpx` SLOT: eight cases of an LW address error in the slot of a branch not taken
  want TAR at the branch's address plus 8, where Diamond left it.
- **Documents read:** psx-spx `6d7d1bc`, "cop0r6 - TAR": "When an exception occurs in the delay slot of a jump or
  branch (cop0r13.31=1), and the branch is to be taken ... this register is updated"; "Cause" bit 30.
- **Test program:** `psxtest_cpx` 1.1 SLOT: BD set, BT clear, CE 3, and TAR the fall-through on the console.
- **Referee:** not consulted. **Witnesses:** none.
- **Conclusion:** with BD set, TAR is written whether or not the branch is taken, with the address execution would
  have continued at; psx-spx's sentence describes only the taken case. Measured by a console's expectations.
- **Depends on it:** a handler that resumes through TAR.
- **Pinned by:** `an_exception_in_a_not_taken_branchs_slot_sets_tar_to_the_fall_through`.
- **Implemented in:** the same commit as D-17.

### D-19. CE is the faulting instruction's opcode bits 26-27 but on a bus error on a load or store and on a break of the debug unit, where it is zero

- **Opened:** 2026-10-11, by `psxtest_cpx` BREAK (and stage 2's open item, §10): some 35,000 of its first run's
  39,146 failing cases differ from the console only in CE, on DBE from loads and stores to KSEG2 and on the debug
  unit's data breaks; SLOT's address errors keep the opcode's bits on the console.
- **Documents read:** psx-spx "Cause" bits 28-29 ("the coprocessor number if the exception occurred because of a
  coprocessor instruction"); the IDT manual, Cause, CE "undefined" for other exceptions.
- **Test program:** `psxtest_cpx` 1.1, BREAK and SLOT; and Diamond's own `exceptions.s` of stage 3, whose twelve
  exceptions include no DBE.
- **Referee:** not consulted. **Witnesses:** at stage 3 Beetle PSX wrote the opcode's bits on the eleven exceptions it
  took, none a DBE (§10).
- **Conclusion:** DBE and the debug unit's Bp write CE 0; every other exception the opcode's bits. Every faulting
  access in the cases is followed by a NOP, so "CE from the instruction after" fits them equally; Diamond takes the
  simpler rule until a case tells them apart. Measured by a console's expectations; the alternative open.
- **Depends on it:** nothing in the kernel; a handler reading CE.
- **Pinned by:** `a_bus_error_and_a_debug_break_leave_ce_zero`.
- **Implemented in:** the same commit as D-17.

### D-20. The debug unit's breaks: KD or UD by the compared address's bit 31, the address the access puts on the bus, the comparison before the address error; ~~open~~ settled by the test's own model

- **Opened:** 2026-10-11, by the 7,680 cases of `psxtest_cpx` BREAK that remain after D-17 to D-19 (`Diamond_Pad.md`
  §5.1), every load and store form, every LWL/LWR/SWL/SWR byte offset, and the PC break.
- **Documents read:** psx-spx "COP0 - Debug Registers" (DCIC's bits, KD bit 29 "Break in kernel mode", UD bit 30
  "Break in user mode", TR bit 31, the vector 80000040h).
- **Test program:** `psxtest_cpx` 1.1 BREAK, its TTY's got and wanted lines grouped by field. Three groups, in kernel
  mode throughout (SR F0000000h):
  1. a data or PC break whose access also meets no device (KSEG2): Diamond breaks to 80000040h, or with TR clear sets
     DCIC's status bits, where the console takes DBE (IBE for the PC) at 80000080h and sets none (858 and 798 cases);
  2. matching by mode: Diamond breaks where DCIC has KD and not UD and the console does not (326), and the console
     breaks where DCIC has UD and not KD and Diamond does not (418), as if the console's kernel-mode accesses answered
     to UD;
  3. the console setting status bits, or breaking, where Diamond sets none or takes DBE (2,082 and 822), the `_8_8_1`
     and `_8_4_4` cases, whose address relation the TTY does not print;
  and the 2,376 cases of D-21.
- **Referee:** not consulted. **Witnesses:** none.
- **Conclusion:** open. The cases' parameters (the address, BDA and BDAM of each) are not printed, so the rules cannot
  be stated from the TTY alone; the next step is a program of the project's own that sets each register combination
  and records what is taken, run against the console's expectations through this test.
- **Depends on it:** debuggers on the console, cheat devices; no game is known to use the unit.
- **Pinned by:** nothing yet. **Implemented in:** nothing yet.
- **The second step, 2026-10-11: the test's own model read.** The cases' parameters are not printed, but the program
  computes each case's wanted record in its own code, which was disassembled from RAM while it ran (the program is
  packed; its code was read only as Diamond's memory held it). The routine that runs one case is given DCIC's value
  with the status bits it expects on a match, the instruction (one of 33 forms: LB, LBU, LH, LHU, LW, LWL and LWR at
  each byte offset, LWC0 to LWC3; SB, SH, SW, SWL and SWR at each offset, SWC0 to SWC3; and the PC), its kind (load, store or PC), its width, the address,
  BDA and BDAM, BPC and BPCM; a hook on its entry in a copy of the probe's runner, never in the tree, listed all
  112,320 calls [M]. Its rules, in the order the code applies them:
  1. *Match.* PC: PCE, and `((pc XOR BPC) AND BPCM) = 0`. Load: DAE and DR; store: DAE and DW; and `((a XOR BDA) AND
     BDAM) = 0` with `a` the effective address for every form but LWL and SWL, whose `a` is the effective address with
     its low two bits cleared (the table the code jumps through for the sixteen LWL/LWR/SWL/SWR forms adds the byte
     offset to the base and clears the low bits only for LWL and SWL).
  2. *Mode.* A match counts when DE and **KD are set for an address whose bit 31 is set, or DE and UD for one whose bit
     31 is clear**: the code tests the address's top bit, not SR; every case runs in kernel mode (SR F0000000h), so a
     kernel-mode store to 001F0000h or 401F0000h answers to UD. A counted match sets the status bits.
  3. *Address error.* A word access with either low bit set, a halfword access with bit 0 set, or a misaligned PC,
     takes AdEL or AdES with BadVaddr and the opcode's CE (D-19), and not the break; the status bits set by 2 stay set.
  4. *Break.* A counted match with TR set takes ExcCode 9 at 80000040h, CE 0, whatever the address meets.
  5. *Bus error.* An address whose top three bits are not 000, 100 or 101 (so 2xxxxxxxh to 7xxxxxxxh and KSEG2) takes
     DBE (IBE for the PC) at 80000080h, CE 0, with the status bits of 2.
  SWC0, SWC1 and SWC3 are stores in all five steps (D-21), LWC0 to LWC3 loads.
- **Evidence weighed** [A]. The model is the test author's, and the TTY's wanted values are its output; they are taken
  as a console's expectations as D-17 to D-19 took the other groups' (`Diamond_Pad.md` §5), the same rung.
  It stays open where the cases do not reach: user mode (where every kernel address faults first, so only rule 2's UD
  half can show), the trace bit, and LWL's and SWL's own address with a mask that keeps the low bits; the mode rule is
  stated by the code but measured only in kernel mode. psx-spx's "Break in kernel mode" and "Break in user mode" read
  naturally as SR's mode, which the test contradicts for kernel-mode accesses to user addresses. **Referee:** not
  consulted: the test waits for Start, which the driver does not press, and the rules are the test's own.
- **Test program:** `psxtest_cpx` 1.1 BREAK, run with Start at frame 400 for 12,000 frames on the open kernel, its
  TTY drained frame by frame (the TTY space holds 64 KiB, which BREAK's 7,680 failing cases overflow). Before: 7,680
  cases failing. With all four rules (this entry's three and D-21's store): **0**, and BREAK's verdict OK. Reverting
  one rule at a time [M]: the mode by SR, 5,052 cases fail; SWCz making no access, 2,436; the address error before the
  comparison, 1,264; LWL and SWL compared at the effective address, 100. The rest of the program's groups, and
  `psxtest_cpu` 1.3 (ALL DONE, every column OK or N/A, `Result: 00000101`), unchanged.
- **Conclusion, the second step:** the five rules above, measured by a console's expectations; open for user mode and
  the trace bit.
- **Pinned by, since the second step:** `the_debug_unit_answers_to_kd_or_ud_by_the_address_segment`,
  `lwl_and_swl_match_their_aligned_word_and_lwr_and_swr_their_address`,
  `a_misaligned_access_that_matches_sets_the_status_bits_and_takes_the_address_error` (each failing before the
  change), the two earlier cases rewritten for UD on a user address, and WiseMan's
  `Amidogs_cpx_test_runs_from_its_menu_and_passes_every_group`.
- **Implemented in, since the second step:** `blackmoon-r3000: the debug unit answers to KD or UD by the address, on
  the bus address, before the address error (D-20); SWC0, SWC1 and SWC3 store (D-21)`.

### D-21. SWC0, SWC1 and SWC3 with their CU bit set make a store on the bus, which can bus-error; what they store is not known, and Diamond stores zero

- **Opened:** 2026-10-11, by `psxtest_cpx` BREAK: 2,376 cases of SWC0, SWC1 and SWC3 to an address that meets no
  device, CU set, want DBE where Diamond's SWCz with z ≠ 2 makes no access (D-3 settled only when CpU is raised).
- **Documents read:** psx-spx "COP0 - Misc" and D-3's.
- **Test program:** as opened.
- **Referee:** not consulted. **Witnesses:** none.
- **Conclusion:** open. The store happens; its value cannot be observed on an address with no device, and modelling
  it would write an unknown value to RAM on any other address. A case storing to RAM would settle it.
- **Depends on it:** nothing known.
- **Pinned by:** nothing yet. **Implemented in:** nothing yet.
- **The second step, 2026-10-11.** *What the console's expectations say* (D-20's model): SWC0, SWC1 and SWC3 are
  stores to the debug unit (DW, the effective address) and to the bus (DBE where no device answers, at 401F0000h,
  601F0000h and C01F0000h alike). They do not say what is stored: no BREAK case reads memory after a store.
  *Test program:* `programs/swcz.s`, with CU0 to CU3 set: SWC0 of PRId (amidog's form), of SR and of an absent
  register, SWC1 and SWC3 of two registers, each to its own word filled with 5A5A5A5Ah, with the GPR of the same number
  holding a known value and the data bus last carrying a known word (a load before, or a store before, or two NOPs);
  SWC2 of the GTE's VXY0 as the control; and SWC3, SW and SWC0 to C0000000h, where no device answers, recording the
  CAUSE taken.
  *Referee* (`Diamond_Referees.md` §4.12) [M]:

  | | SWCz to RAM (10 cases) | SWC2 | SWC3, SW, SWC0 to C0000000h |
  |---|---|---|---|
  | DuckStation | 0 in all ten | 12345678h | DBE, DBE, DBE |
  | Beetle PSX | unchanged | 12345678h | none, none, none |
  | SwanStation | unchanged | 12345678h | DBE with CE 3, none, none |
  | PCSX-ReARMed | unchanged | 12345678h | DBE, none, DBE |
  | PCSX-Redux | stops before writing its done word, as it stops at an address error (§4.11 of that page) | — | — |
  | Diamond before | unchanged | 12345678h | none, DBE, none |
  | Diamond after | 0 in all ten | 12345678h | DBE, DBE, DBE |

  *Weighed* [A]. Only DuckStation makes the store a console makes by amidog's expectations, and only it takes DBE on
  the plain SW to KSEG2 that every case of the test expects; the three that leave RAM unchanged make no store at all
  (Beetle PSX takes no bus error even for SW), so they model no store rather than a store of the old contents. None
  stores the GPR, PRId or the bus's last word.
- **Conclusion, the second step:** the store is made, with the CU bit's check of D-3 and the debug unit's comparison of
  D-20, and can bus-error: measured by a console's expectations. **Its value is DuckStation's, zero: the strongest
  single witness, since it alone models the store; open.** A store to RAM on a console would settle it; none is used.
- **Pinned by, since the second step:** `swc0_swc1_and_swc3_store_on_the_bus` (failing before the change) and
  Diamond's `swcz_stores_zero_on_the_bus`.
- **Implemented in, since the second step:** the same commit as D-20's second step.

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

*The referee step on the garbage registers, 2026-10-11* (`Diamond_Referees.md` §4.12). `cop0-garbage.s` was given
its own exception vector, which counts what it takes, and ten more cases: a garbage read straight after a register
of a known value (BPC 11111111h, BDA 22222222h, BPCM 33333333h, BDAM 44444444h, SR, DCIC), after one NOP and eight,
after a loop of 1,000, and after an MTC0. psx-spx predicts the known values there. The referees [M]: **DuckStation**
raises a reserved-instruction exception on every one of the 28 reads (CAUSE 28h), so the registers keep what they held;
**Beetle PSX** takes none and leaves the register unchanged (its pattern of stage 3 was the program's own earlier
register values: here 08004088h, the vector's word, in t1 throughout); **SwanStation, PCSX-ReARMed and PCSX-Redux**
read 0 everywhere and take none; none reads a known value after its register. SingleStepTests' r3000 suite has no
coprocessor-0 file. Ruled [A]: **no exception**, as psx-spx says and three lineages agree (PCSX, Mednafen, and
SwanStation's state of the DuckStation lineage), DuckStation alone against; **the value stays 0** [C], the reading two
lineages share, while psx-spx's own account, which names no rule and no referee reproduces, is not modelled. Open on
the hardware. Pinned by `the_garbage_registers_read_zero` (30 words: 28 reads, no exception).

*Answers with the pad, 2026-10-11* (`Diamond_Pad.md` §5). **CE**: answered by `psxtest_cpx`, the opcode's bits on an
address error, zero on DBE and on the debug unit's breaks (D-19). **A second load cancelling the first**: answered by
`psxtest_cpu`, which reads the register in the second load's delay slot at 441 places and passes; a build in which the
first load lands fails 265 of its cases. **The garbage registers**: still open, since neither amidog test reads them.
`psxtest_cpx` also found D-17 (MTC0's interrupt latency) and D-18 (TAR), fixed, and leaves D-20 and D-21 open.

The items below stand as written at stage 2.

- Where the I-cache and scratchpad live (§2); the isolated calls' answers.
- D-2's measurement, and §5.5's unmodelled points.
- The garbage registers' values; SWC0's and LWC0's bus behaviour; the trace break. *2026-10-11*: SWC0's is D-21's
  second step; LWC0 to LWC3 load their word, as amidog's DBE cases have it; the garbage registers below.
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

## 12. Loads for a debugger (2026-10-11)

*Built for the core ABI's minor 1.2, read watches and read data breakpoints (`EmuSen_CoreAPI_Extensions.md` §13,
`Diamond_Debugger.md` §9).* Two additions, both outside the plain step.

- **`Observer::read`**, `emusen-native`'s, with an empty default: each byte of a load once the bus has answered it,
  with the processor's space and the loading instruction's address, as `wrote` reports a store. Every load reaches it
  from the one place the crate reads data (`Run::read`): LB, LBU, LH, LHU, LW, LWL, LWR, and LWCz with CUz set (LWC2's
  word goes to the GTE; LWC0's, LWC1's and LWC3's is read and dropped, §10). LWL and LWR report the bytes of the word
  they reach, from `address & !3` up to the address and from the address up to the word's end, D-1's one access of up
  to three lanes. **Not reported**: a load that met no device (DBE), as a faulting store's bytes are not; a load
  while IsC isolates the cache, which reaches the cache and not memory (§6.3); and fetches. With `Unobserved` the
  loop's body is empty and the plain step compiles to the same instructions as before; Diamond's frame cost, before and
  after with the link's placement held fixed, shows no change (`Diamond_Debugger.md` §9.5).
- **`R3000::load_ahead(word)`**: for a debugger in front of the instruction `word` at the PC, the first address and
  the count of the bytes it would load if it ran now, nothing changed. It applies the step's own rules in advance
  (§5.1, §5.3, §6.2, §6.3): none if the fetch would fault, if the debug unit's PC break would trap, with IsC set, for
  an instruction that loads nothing, for LWCz with CUz clear, or for an address error or a data break that would
  trap; the address from rs as the instruction reads it, before a pending load lands. The debug unit is asked on a
  copy of the registers, so its status bits are not set by asking. What it cannot know is whether the bus will
  answer: a load to no device is named, and then takes DBE without reading. It takes the word as an argument because
  the processor never peeks its bus; the caller supplies what a fetch would return (Diamond's `Board::peek_code`,
  which answers from the I-cache where the fetch would hit).
- **Tested** (`every_load_reports_the_bytes_it_read_and_load_ahead_names_them_first`, the file's 25th case) [M]: LWL
  at 10Bh and LWR at 109h report 108h–10Bh and 109h–10Bh with their values and their instruction's address; then each
  of the eight loads, in five modes (kernel; IsC; user mode at a KSEG0 address; CU2 clear; a bus with no device at the
  address) at four offsets. In the 128 cases where the bus answers, `load_ahead` named exactly the bytes the step then
  reported, the misaligned and the refused loads included (both none); in the 32 to no device the step reported
  nothing, and `load_ahead` had named the 22 of them that were aligned, the one case only the read can tell. The
  suite's 55,000 cases are unchanged.

## 13. Runs, and the common instructions' path (2026-10-12)

*Built for Diamond's second speed pass (`Diamond_Speed.md` §12).* Two changes, the first an addition to the
interface (§2), the second inside the step; neither changes what a step does.

**`R3000::run`.** Its promise is the loop it replaces: `step` until a step is not an instruction, the limit is
reached, or `running` answers no. Inside it the boundary's work is done once and then held while it cannot change.
After a full step, the boundary is *quiet* when the debug unit's DE is clear, no MTC0 view of D-17 is pending, and the
interrupt check would not be taken with SR and Cause as they stand. The check's inputs are SR, Cause, the view and the
external line; SR, Cause and DCIC change only by MTC0, RFE and exception entry, the view only by MTC0, and the line
only on the bus. So the steps that follow a quiet boundary skip the interrupt check, the observer (a run has none) and
the PC break (DE is clear, and `pc_break` then changes nothing), and keep IP2 as it is, which is what the step would
write while the line holds. Each of them still asks the line and the fetchable check; a line that moved, a PC that is
not a word's or is kernel space in user mode, an instruction that ran MTC0 or RFE, or a fetch that met no device sends
the run back to a full step, which then takes the interrupt, the address error or the exception as the step does.
D-17 is kept by the order: the MTC0 ends the quiet steps, the full step after it uses the view, and the boundary after
that is judged quiet or not on the SR the MTC0 wrote.

**The common instructions.** The step's instruction path was one `Run` value carrying the instruction's operands,
delays and results through every arm; the compiler kept it in memory, and a guest instruction cost about 185 host
instructions on the desktop [M, a stepped trace of a nine-instruction loop under gdb]. The instructions that only
compute, the branches and jumps, and LB, LBU, LH, LHU, LW, SB, SH and SW now take a path of their own (`common` in
`exec.rs`) with their operands as parameters, and complete as `finish` completes one: the previous load lands unless
the instruction loads the same register, then the result, R0 kept zero, a load left for the next. Their exceptions
(ADD's, ADDI's and SUB's overflow, address errors, data breaks, a data access that met no device) enter through the
same `exception`. The checks of a data access, the read and the write are one set of functions that `Run` calls too,
so each rule has one owner; `Run::execute` keeps the rest (the multiplier, LWL, LWR, SWL, SWR, the coprocessors,
SYSCALL, BREAK, the reserved encodings). With the run, the loop costs about 119 host instructions an instruction.

**How it is known.** The suite's 55,000 cases pass through the new path [M]. `a_run_is_its_steps_on_random_programs`
runs 400 random programs of every kind of instruction, with handlers at the four vectors, an interrupt line that rises
and falls with the clock, SR written with interrupts on and off and in user mode with the cache isolated and BEV set,
DCIC armed on a PC and on data, loads and stores unaligned or to no device, as runs of random limit against a bus
whose `running` ends at a random cycle, and holds the processor, the clock, the writes, the isolated accesses and the
GTE's traffic to those of a copy stepped one step at a time: 19,801 runs, the longest 300 steps, 4,297 interrupts, and
every exception code but IBE met [M]. `a_runs_boundary_is_the_steps` names five boundaries a run must still see: a line
rising mid-run, an interrupt MTC0 enables one instruction late, a PC break MTC0 arms, a jump to an address not a
word's, and kernel space reached in user mode. Twenty-four mutants were applied one at a time: twenty-three were
caught. The survivor, the quiet check's interrupt test without SR's mask bits, makes the check stricter: a masked
request then sends a run back to full steps, which costs speed and changes no step, so nothing can catch it.


## 14. The moves, for the enhancement layer (2026-10-13)

*Built at Diamond's stage E1 (`Diamond_Enhancements.md`, `Diamond_Plan.md` §6.2).* The precision side-channel needs to
follow a projected vertex from the GTE through the processor's registers and memory to the GPU. The processor is the
only part that knows which register a value went to or came from; the bus sees an address and a value, never the
register. So the processor reports, and its console interprets.

**The interface.** `Bus::MOVES`, an associated constant false by default, and `Bus::moved(Move)`, a default method
with an empty body: additions with defaults as `bus_error`, `fetch_decoded` and `running` were (§2), so a bus that does
not give them is unchanged. Every report is made under `if B::MOVES`, so the step compiled for a bus that leaves the
constant false holds no trace of the reports, their arguments included; a bus that sets it true is told. `Move` names
seven moves (eight since version 2, below), `at` the instruction's address, each a load once it has read and a store before it writes (*since stage
E4*, below):

| Move | Instructions | Fields |
|---|---|---|
| `Load` | LB, LBU, LH, LHU, LW, LWL, LWR | the register, the bus address and size of the read, the value the register will hold after the next instruction (extended, or merged for LWL and LWR) |
| `Store` | SB, SH, SW, SWL, SWR | the register, the bus address and size of the write, the bytes written |
| `FromCop2` | MFC2 | the register, coprocessor 2's data register, the value |
| `ToCop2` | MTC2 | the register, coprocessor 2's data register, the value |
| `LoadCop2` | LWC2 | coprocessor 2's data register, the address, the value |
| `StoreCop2` | SWC2 | coprocessor 2's data register, the address, the value |
| `Copy` | ADDU, ADD, SUBU, SUB, OR, XOR with R0 as the second operand; ADDU, ADD, OR, XOR with R0 as the first; ADDIU, ADDI, ORI, XORI of 0; SLL, SRL, SRA by 0 | the register given the value, the register it came from, the value |
| `Shift` (*version 2*) | SLL, SRL, SRA by a constant of 1 to 31 | the register given the value, the register it came from, the kind (`Shift::Left`, `Right` or `Arithmetic`), the amount, the value shifted |

Not told: a load that met no device (it ends in DBE; a store is told before it can know, below), one the isolated cache took (it reached no memory), any
move whose register is R0 (it holds nothing, so a NOP, `SLL R0, R0, 0`, is not a copy), CFC2 and CTC2 (the control
registers hold matrices, vectors and offsets, never a projected vertex), and an instruction that computes anything
else. A load is told when it is made, though its register is written one instruction later; a console that checks
the value it tracks against the value it later meets is not misled by the delay slot's use of the old value
(`Diamond_Enhancements.md` §3.2).

**A store is told before it writes** (*since Diamond's stage E4, 2026-10-13*, `Diamond_Enhancements.md` §21.4). A
device the store reaches may act on the value inside the write: Diamond's GPU starts a command the moment its last word
arrives with the FIFO empty, and its layer must know which register that word came from before the command draws.
Told after the write, as at E1, a triangle sent to GP0 by the processor's stores was shown to reach the layer with no
register known (`precision`'s `a_projected_triangle_is_precise_by_the_processor_and_by_dma` fails on the old order).
The value is known before the write; whether the write meets a bus error is not, so a store that faults is told too,
which a console that checks the value it tracks at the use cannot be misled by. Loads stay told after the read, their
value being the read's.

**Why the bus, and not a second observer.** `README.md` beside this page had said the layer's half would be "a second
generic parameter". The layer must see the processor's moves, the GTE's projections and the GPU's receipts in one
order, and the GTE and the GPU are reached through the bus, inside the console's board. A second parameter beside the
bus would have to be the same object the board holds, borrowed twice. Told to the bus, the moves reach the console's
board, and a console that wants them steps through a bus of its own that forwards them; Diamond's is `LayeredBus`
(`Diamond_Enhancements.md` §2.3).

**Version 2: a shift by a constant** (*2026-10-14*, `Diamond_Enhancements.md` §54). Version 1 is the seven moves of the
table but the last, with the store told before it writes; version 2 adds `Move::Shift` and changes none of the seven. A shift by a constant
gives a register another's bits moved, which a console following a value may want to follow as it follows a copy: Spyro
the Dragon keeps each projected vertex shifted left five bits in another register, with flags in the bits cleared, and
shifts it back before the GPU takes it. It is told from the arm of the computing instructions that tells a copy, under
`if B::MOVES`, every shift by a constant whether in place or into another register, so that the report is a class of
instructions and not one console's policy; a shift by 0 stays a copy, one into R0 is told nothing, and SLLV, SRLV and
SRAV, shifts by a register's amount, are not told (no vertex's path read in a game shifts that way). `Move` is matched
exhaustively by its consumers, so a variant is a change to each: within the project Diamond's precision layer and test
observer and this crate's test bus, all changed with it.

**How it is known.** `each_move_is_told_with_its_values` runs a program of every move against the test bus and holds
the moves told to a list of nineteen, field for field, including LWL and LWR merged across a pending load and SWL and
SWR's bytes; a load that met no device, a store and a load to the isolated cache, and a NOP tell nothing. Since version
2 a second program holds four shifts by a constant (left, right, arithmetic by 31, and one in place) told with their
kinds, amounts and values, a shift by 0 told as a copy, and a shift into R0 and an SLLV told nothing; the random
programs told 18,156 moves over the same runs [M, 2026-10-14].
`a_run_is_its_steps_on_random_programs` (§13) now records the moves on both buses and holds a run's equal to its
steps', 15,479 moves over the 19,801 runs [M, 2026-10-13]. The suite's 55,000 cases pass unchanged [M]. Diamond's
goldens hold the processor's results unchanged with the moves compiled in, and again with a layer told them
(`Diamond_Enhancements.md` §7).

**Why a constant, measured.** The first build reported through `moved` alone, trusting the empty default to vanish.
It did, as calls: no call remained in the step for Diamond's board. But the instructions around the reports were
compiled otherwise, a register move added to each arm of the computing instructions and a shift sunk into its arms,
and `exec::run` for the board was 95 bytes longer [M, the two links' `nm` sizes and their disassembly compared]. On
the aligned links of `Diamond_Speed.md` §2, the processor's change alone cost Crash Bandicoot 0.7 per cent and Tekken
3 1.4 per cent a frame [M, `Diamond_Enhancements.md` §9]. Under `if B::MOVES`, `exec::run` and `Run::execute` for the
board are the base's to the byte in size, and the cost went with them. The prediction recorded when the reports were
written, that an empty default compiles to nothing, held for the calls and failed for the code around them.
