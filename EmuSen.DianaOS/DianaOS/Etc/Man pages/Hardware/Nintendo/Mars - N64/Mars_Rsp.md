# Mars — the signal processor's scalar half

*Phase C's first slice, landed 2026-09-17. The RSP as a processor: its program counter,
the scalar instruction set, the interface registers it is driven through, and the clock
it shares with the main CPU. `Rsp/Rsp.cs`, with the interface in `Memory/SpInterface.cs`.
Tests in `MarsRspTests`. The vector unit, which landed the same day as the next slice, is
`Mars_RspVector.md`.*

---

## 1. What the RSP is, for the purposes of this slice

A second MIPS processor inside the RCP, with 4KB of instruction memory and 4KB of data
memory and nothing else. Its scalar half is MIPS I with most of the hard parts removed:
**no exceptions, no alignment, no 64-bit words, no multiply or divide, no branch-likely,
no TLB and no caches.** What it has instead is a vector coprocessor on coprocessor 2, which
is the reason the processor exists and has its own page (`Mars_RspVector.md`).

The removals are what make this half small, and they are also where it differs from the
main CPU in ways that are easy to carry over by accident. §3 and §4 are both of that kind.

## 2. The program counter is an offset into instruction memory

Twelve bits, with the low two always clear, so every counter value is `& 0xFFC`. A write
of `0xFFFFFFFF` to the register reads back `0xFFC`, and a counter that runs off the end of
instruction memory wraps to its start rather than leaving it. It lives at `0x04080000`, a
page away from the other interface registers.

Mars models it with the same two-counter delay-slot arrangement as the main CPU
(`Mars_Cpu.md` §3), with both counters masked at every step.

### 2.1 Three delay-slot behaviours the corpus pins

- **A break in a delay slot still lets the branch decide the counter.** A taken branch
  with a `BREAK` in its slot halts with the counter at the branch target; an untaken one
  halts at the instruction after the slot. That falls out of the two-counter model without
  a special case, which is some evidence the model is the right shape.
- **A jump-and-link whose link register is its own target register reads the target
  first.** `JALR $at, $at` must jump to the old value of `$at` and then overwrite it with
  the return address. Writing the link first and then reading the target — which is the
  order the operation's name suggests — jumps to the return address instead. Mars had it
  that way round; the corpus caught it, and a test now reproduces the corpus's case.
- **The link of a branch-and-link is written whether or not the branch is taken.**

## 3. Nothing traps, so the signed and unsigned forms agree

`ADD` and `ADDU` are the same instruction here, as are `SUB` and `SUBU`, and `ADDI` and
`ADDIU`. With no exception mechanism there is nothing for an overflow to do, and the
corpus's vector `0x12345678 + 0xFFFFEDCB` wraps to `0x12344443` under both encodings.

This is the first of the two places where carrying the main CPU's code across would have
been wrong in a way no test on the main CPU could catch, because on the main CPU the
distinction is real.

## 4. A data address is twelve bits, and alignment does not exist

Every load and store reaches data memory only, at `address & 0xFFF`. Everything above
those twelve bits is discarded rather than faulting, so `0x7FFD` and `0x1FFD` both name
`0xFFD`.

**An unaligned access is ordinary.** A `LW` at `0x001` reads the four bytes at `0x001`
through `0x004`, big-endian, and a `LW` at `0xFFD` reads `0xFFD`, `0xFFE`, `0xFFF` and then
wraps to `0x000`. Mars implements every data access as a byte loop with the mask applied
to each byte, which makes both properties structural rather than special cases. The
corpus's load table is reproduced in `MarsRspTests` for the offsets that exercise each:
aligned, one byte in, three bytes in, straddling the top, and wrapping.

This is the second place the main CPU's code would have been wrong: there, an unaligned
`LW` is an address error (`Mars_Cpu.md` §6).

## 5. The interface registers, from both sides

Eight registers at `0x04040000` — the two DMA addresses, the two DMA lengths, status, the
two DMA progress flags and the semaphore — and the counter at `0x04080000`. Phase A built
the DMA half of these for the boot handoff (`Mars_Memory.md` §6); this slice adds the
processor behind them.

**The RSP reaches the same registers through its own coprocessor zero**, by index: 0–7
are these eight, and 8–15 are the display processor's command registers. So `MFC0` and
`MTC0` on the RSP are not a separate mechanism but a second address for the same
hardware, and Mars routes them through the bus rather than duplicating the registers.

Status is assembled on read rather than stored: the processor owns halt and break, and
the interface owns single-step, interrupt-on-break and the eight signals.

Since Phase G the break instruction raises the SP interrupt itself, when interrupt-on-break
is set and the break bit was down (`Mars_Performance.md` §15). That second condition, and
Mars's halting after each instruction in single-step, are both disputed by the FPGA core,
Project64 and mupen64plus, which raise on every break and ignore single-step; §15 there
records the evidence, and neither rule is pinned by a test until it is decided.

### 5.1 A write names a request per field, and naming both halves is no request

Status is written as pairs of bits — clear halt and set halt, clear interrupt and set
interrupt, and so on through the eight signals. **A write with both bits of a pair set
leaves that field unchanged.** It is not a clear followed by a set, nor a set followed by
a clear; it is nothing.

Mars had applied the clear and then the set, so set won. Eight corpus signal tests, the
interrupt-on-break test and two register-access tests all failed on it, and several of
them failed with values that looked like an unrelated defect — a status register reading
back with every signal raised — because the corpus writes many pairs at once and expects
most of them to be inert.

The corpus tests this rule for **every pair Mars applies it to**: the signals,
interrupt-on-break, the interrupt, and halt. Halt is the interesting one, because
clearing halt starts the processor: the corpus writes both halt bits with a program
loaded and checks the counter never moved, then clears halt alone and checks that it did.
Mars had applied the rule to all four before finding that out, on the reading that one
set-and-reset mechanism serves every field; the reading was right, and it is now a
measurement rather than a reading.

Clearing halt starts the processor at whatever the counter register holds, and setting it
stops the processor where it is — the counter is not reset, and a halted program resumes
from where it stopped.

## 6. An encoding this half does not own does nothing

The RSP raises no exceptions, so there is nothing a reserved or unimplemented encoding
could do except nothing. Every opcode outside the scalar set and the vector unit is a
no-op.

> **Superseded 2026-09-17 by the vector unit (`Mars_RspVector.md`).** This section recorded
> that `COP2`, `LWC2` and `SWC2` executed as no-ops while the unit was unbuilt, and why that
> was chosen over stopping the machine. The reasoning is kept because it was borne out: it
> read
>
> *That is a deliberate choice against the alternative this project used for the FPU
> (`Mars_Fpu.md` §6), where an unbuilt instruction stopped the machine loudly. There, the
> scaffold was worth its cost because nothing past the first unbuilt instruction could run.
> Here the opposite holds: the vector tests are a self-contained block, the corpus's
> programs report results through data memory whether or not the vector instructions did
> anything, and a loud stop would have cut the run short at exactly the moment it first
> became able to reach the end. The vector tests fail, individually and honestly, and
> everything after them runs.*
>
> The complete runs that followed are what located the seventeen CPU groups
> (`Mars_Corpus.md` §9), and the census they produced attributed exactly 155 groups to the
> unit — which is the number the unit then cleared.
>
> **One consequence the section did not foresee:** the vector unit's first unit tests were
> written against the no-op build to show they failed, and four cases passed anyway, because
> a no-op load leaves a register at the zero a test expected (`Mars_Corpus.md` §10). A
> silent no-op makes a test's failure depend on it not expecting the value nothing produces.

## 7. The clock

The RSP steps from the machine's one clock (`Mars_Memory.md` §3): each tick of the bus
executes one RSP instruction while the processor is running. **One instruction per tick is
a placeholder**, not a measurement. The two processors are clocked at different rates on
hardware, and timing accuracy is Phase G's (`Mars_Gameplan.md` §2.2).

What the placeholder does establish is that the two processors run in parallel rather than
one to completion before the other, which the corpus tests directly — the RSP raises a
signal, waits in a loop for the CPU to raise one back, and records whether it timed out.
That test passes.

## 8. What the corpus says

**Eighty-eight of the 248 RSP and SP test groups pass**: every scalar instruction, all
branches and jumps including the three delay-slot cases in §2.1, `BREAK` in all three of
its forms, the counter wrap, the counter masking, the parallel-running test, and every
interface register test. Of the 160 that fail, all but six are the vector unit. The six are
four `spmem` tests about how the *main CPU's* sub-word loads and stores reach the RSP's
memories — a bus question rather than a processor one — and one `RSP STATUS` test that is
in fact about the display processor freezing, which is Phase D.

**The larger result of this slice is not in that count.** With the RSP able to halt, the
corpus run got past the point where it had waited since Phase A — and ran straight into a
defect in how Mars was *running the corpus*, which is `Mars_Corpus.md` §8.

> **Update 2026-09-17, after the display processor's interface (`Mars_Rdp.md`): all 248 pass.**
> The last one was the display processor's after all, and cleared with the rest of its module.
>
> **Update 2026-09-17, after the main CPU's access to RSP memory (`Mars_Memory.md` §2.4):
> 247 of the 248 pass.** The last is *RSP STATUS: start-valid*, which lives in the corpus's
> `rdp` module at its RDP level and fails on *"RDP was told to freeze, but it didn't"*. No test
> of the RSP fails.
>
> **Update 2026-09-17, after the vector unit: 243 of the 248 pass.** The five that fail are
> the four `spmem` groups and the `RSP STATUS` group named above. That also corrects this
> section's arithmetic, which said *"all but six"* and then named five: 160 failing groups
> less 155 in the vector unit is five, and five is what the run now shows.

## 9. What this slice does not do

- **Single-step as more than a status bit.** It is stored and reported, and it halts the
  processor after each instruction; nothing is known about how hardware interacts it with
  a break or a delay slot.
- **Timing.** §7.

## 10. What the references do about stepping, and why Mars is not changing it yet

*2026-09-19, Phase G. A study, and a decision not to act on it.* §7 calls one RSP instruction per bus tick a
placeholder. A survey of every reference this project keeps says something sharper than that: **no reference
interleaves the two processors at instruction granularity, and none needs to.**

| | when the RSP is handed control | how much it runs |
| --- | --- | --- |
| mupen64plus-core | a CPU write to SP_STATUS clearing halt | the whole task (`doRspCycles(0xffffffff)`) |
| cxd4 | the same | the whole task, an unbounded `for(;;)` |
| paraLLEl-RSP | the same | the whole task, JIT regions of up to 128 instructions |
| Project64 | the same, plus a timer every 0x200 CPU cycles | the whole task (`ExecuteOps(-1, -1)`) |
| Mars | every CPU instruction | **one instruction** |

**What they buy back the lost concurrency with** is a small set of yield heuristics, and the survey names every one:
a per-destination-register counter on `MFC0` of SP_STATUS that force-halts a spin after sixteen turns; a yield on the
first read of SP_SEMAPHORE; the same counter on the display processor's busy registers; a forced exit when a DMA
writes instruction memory; and a "the task stopped without BREAK" path that clears halt again and raises the
interrupt so the CPU learns the task yielded. paraLLEl removed the semaphore yield once for accuracy's sake and
immediately broke four games, then restored it.

**One hardware test depends on it.** The corpus's `ParallelRunning` has the RSP raise a signal and spin up to ten
thousand times for a reply, then *records how far it got* in data memory; a run-to-completion RSP writes zero there
and fails. Mars passes it by actually interleaving. The same corpus's `SemaphoreRegisterRSPOnly` proves real hardware
reads the semaphore five times without yielding, which the two references' semaphore hack violates — so their
compatibility is bought with a known inaccuracy that Mars does not have.

**The structural advice, which is sound and is not being taken.** Mars's inner loop tests the halt flag twice per
instruction (the interface's loop condition and `Step`'s own guard), tests the single-step flag and the coverage hook
a third and fourth time, and keeps the counter as two fields so every instruction pays two loads and two stores.
cxd4 keeps one program counter in a register, increments it *before* dispatch so it already names the delay slot, and
reaches the slot with a `goto` on taken branches only; it writes the memory-mapped counter once, at exit, and checks
the halt flag only in the two arms that can set it, with the comment *"only BREAK and COP0 set this"* as the
argument. All of that is bit-identical by construction and attacks the loop directly.

**Why it is not being done now.** `Mars_Performance.md` §32's fresh profile, taken after the display processor and
the scan-out moved to their own threads, puts this loop at **1.4 to 2.4 per cent** of an emulation thread that is
**idle 65 per cent of the time**. A change that made the loop free would shorten no frame, because the frame does not
end when the emulation thread finishes. The advice is recorded here so that it is not re-derived, and it becomes
worth building the moment the emulation thread is the bound again — which §32 says it is not.

**One accuracy finding worth separating from the speed question.** The corpus's `ClockCPUvsRSP` pins the CPU and the
RCP at three to two, and the MiSTer clock tree confirms it: 93.75 MHz against 62.5 MHz. Mars runs one RSP
instruction per CPU instruction, which is one to one. That is §7's placeholder stated as a number, and it belongs to
the phase's timing work rather than to this section.

**What the hardware guarantees, for whenever the slice does change.** The MiSTer register-transfer code offers
exactly one ordering guarantee: each register access is one indivisible bus transaction, with the RSP core winning
arbitration, and halt takes effect on the following cycle. Anything finer is undefined — a CPU write to instruction
or data memory that collides with the running RSP's own access in the same cycle raises an error flag rather than
having a defined result. So any slice boundary is legal provided it does not fall inside a register access, and a
yield must leave the program counter exact, clear halt again, and raise the interrupt.

**Whether any reference runs the signal processor on a thread of its own, read on 2026-09-19.** One does, optionally:
Project64's "RSP multi-threaded" setting runs a low-level task on a second thread in slices of a hundred instructions,
and the processor reads data memory, instruction memory and the status register with no lock, fence or wait between
them, so the ordering is whatever the host's memory system gives. It is off by default and no game entry turns it on.
CEN64 runs the RCP and the VR4300 on two threads that rendezvous on a condition variable every slice of a few thousand
cycles, and its own history records accuracy given up for it. simple64 and gopher64 run the task in bursts on the
emulation thread and back-date its cost by scheduling the interrupt the burst's cycles later. None of this moves the
decision above: a thread for the signal processor buys concurrency the machine's thread does not need while it is idle
two thirds of the time, at the price of an ordering no reference has made exact.

### 10.1 The loop, once the thread was the bound

*2026-09-20.* §10 declined its own advice while the emulation thread was idle. `Mars_Rdp.md` §2.8 made that thread
the bound, and an interrupt-based sample of it (`Mars_Performance.md` §36) put the per-instruction loop at eleven per
cent of the thread and the signal processor as a whole at half of it. Two of §10's items are taken, both exact by
construction, and the rest are recorded as still open.

*The halt tested once, and the loop counting down.* The interface's step was a counted loop testing the halt flag
each turn, calling a step that tested it again, then testing the single-step flag; nearly every call is for one
instruction from a block's tick, which has already tested the halt. The halt is now tested once before a do-while
that counts the cycles down, the step it calls (`StepOne`) does not test it, and the single-step flag is still read
after every instruction, because the processor can set it on itself through its own status write.

*The fetch without the span's check.* The program counter is masked to the four kilobytes of instruction memory
before every fetch, so the span's bounds check tested what the mask had already made true; the fetch now reads the
word through an unchecked reference and swaps its bytes, which is the same instruction sequence less the compare.

*Not taken, still.* One program counter in place of the pair: the branch code and the state format both carry
`NextPc`, and the saving is two stores an instruction against a change that reaches every branch. The dispatch
itself — a switch on the opcode, then one on the function — is the shape cxd4 has, and nothing here is cheaper.

*Taken the same day, once §10.1's first two measured the least of §36.1's three:* the block's tick now steps the
processor itself for the one instruction a tick nearly always is, reading the single-step flag once and skipping the
interface's loop; and the register file is read and written through an unchecked reference, since every index is a
five-bit field of the word or the constant 31, which is what the array's check would have tested. Interleaved at four
workers, three rounds of 600 frames, second halves: Wave Race 64 83.2 to 90.0 fps, Super Mario 64 90.7 to 96.9 in the
castle and 96.3 to 102.1 outside it, the rounds disjoint; Ocarina of Time 78.8 to 77.5 with the rounds overlapping,
which is the state where the signal processor's share is smallest. Kept on three states' evidence
(`Mars_Performance.md` §36.2). What is still open is what §10 named first: one program counter, and a dispatch that
does not return to a loop between instructions.

## 11. Straight lines compiled, and run whole while the CPU idles

*2026-09-20.* §10 kept the rule that the signal processor takes one step for each cycle of the CPU, and this does
not change it: the steps are the same steps in the same order at the same cycles. What changes is who runs them.
`Mars_Performance.md` §38 found that every step is taken while the CPU is in its idle loop, and
`Mars_Recompiler.md` §15 runs that loop natively; from there, when a cycle is a step, the processor is asked to
`RunBlocks` with a budget of the cycles left before the CPU's stop.

**A block** is a straight line of instruction memory: up to a branch and its slot, or up to and including a move to
or from the control registers or a break, at most thirty-two instructions and never across the end of the memory.
It is compiled (`Reflection.Emit`, off the machine's thread, run interpreted until published) to what `StepOne`
does for each instruction — the two counters moved past it, as constants except in a slot, where they are taken
from the branch as the step takes them — and then a call of the instruction's own handler with its word as a
constant: `ExecuteSpecial`, `ExecuteRegImm`, `ExecuteCop0`, `ExecuteCop2`, the two vector memory handlers, or
`Execute` for the rest. No handler was changed or copied; the block removes the fetch, the first decode and the
step's bookkeeping, and nothing else.

**Why the events end a block.** Between two steps the CPU's side tests two things, a write through the bus and the
interrupt line. Only two kinds of instruction can change either: a control-register move (a DMA, a status write, a
display-processor register) and a break. Ending a block on them, and returning at once when a block so ended has
run, means the test is made after the same step it was always made after. A branch whose slot is an event or a
branch is left to the interpreter, so the shape never has to reason about it.

**Several blocks an address.** The graphics and the sound microcode are loaded over each other every field, so the
same address holds different code from one task to the next. Blocks are found by address and then by comparing
their words with memory, four to an address, the least run giving way; nothing is invalidated when instruction
memory is written, because every entry compares. A block is compiled after three runs.

**What it bought and what it did not** is §38's table: a quarter to a third of the emulation thread in Super Mario
64 and Wave Race, less in Ocarina of Time, with 74 per cent of steps run in blocks. The rest are runs shorter than
a block near the CPU's stop, branches with awkward slots, and blocks not yet compiled. The per-step cost fell less
than the samples of the fetch and dispatch promised: the time is inside the vector handlers, whose own dispatch,
element shuffle and register arrays a block does not touch. Compiling those with their operands constant is the
next step and is not begun. `EMUSEN_MARS_NORSPBLOCKS=1` turns the blocks off; `MarsIdleTests` is the proof they
change nothing (`Mars_Recompiler.md` §15).

### 11.1 Two compiler threads, not a task a block (2026-09-22)

**The defect: boots stalled for seconds.** Run from boot with pacebench, whose 120 warm-up frames are not measured, Super Mario 64 had a single frame of **1,657 ms**, and others of 583, 554 and 242 ms. Ocarina of Time had a single frame of **2,294 ms**. Afterwards both settled to 7 to 9 ms a frame.

A thread-time trace of the first sixty frames showed what the emulation thread was doing: it spent 3.9 of its 5.7 s in `MarsCore.JoinPresentation`, waiting on the frame's deferred presentation job. That job runs on the thread pool. The pool had grown to **27 threads on 16 cores**, each spending 1.5 to 3.9 s in `Rsp.Compile` and `RuntimeHelpers.PrepareDelegate`. Each RSP block had been queued with its own `Task.Run`. A boot shapes about 1,700 blocks within seconds, and the pool, seeing its workers blocked, kept injecting threads for them. The presentation job queued behind all of them.

The CPU's recompiler never did this. It has always compiled on one dedicated thread fed by a channel (`BlockCompiler`, `Mars_Recompiler.md` §2.4).

**The change.** RSP blocks now go into one channel, drained by dedicated background threads named "Mars RSP block compiler". The thread pool is left to presentation and the RDP. How many compiler threads was chosen by measurement, over 1,200 frames from boot:

| Compiler threads | SM64 worst frame | OoT worst frame | SM64 frames 900-1199 | SM64 RSP steps in blocks |
|---|---|---|---|---|
| a task a block (before) | 1,657 ms | 2,294 ms | 8.47 ms | 77.5% |
| 1 | 66 ms | 25 ms | 9.37 ms | 49.7% |
| **2** | 72 ms | 24 ms | 8.58 ms | 66.3% |
| 4 | 108 ms | 30 ms | 8.71 ms | 73.5% |
| 8 | 123 ms | 37 ms | 8.52 ms | 76.3% |

One thread removes the stall, but it compiles slowly enough that a fifth of the blocks are still waiting at twenty seconds, so the steady frame costs about a millisecond more. Four or more threads compile as fast as the pool did, but they compete with the emulation thread and the RDP workers, and the long frames return. **Two was chosen.** It compiles every block the pool compiled in the same twenty seconds (1,719 and 1,546), with the steady frame within noise of the old build.

`EMUSEN_MARS_RSPCOMPILERS=n` overrides the count, for measurement.

*Superseded the next day (§11.2).* Two compiler threads were a remedy for a compile that was itself the defect. Blocks now compile in two tiers: one thread compiles the cheap first tier, and one folds the hot blocks. `EMUSEN_MARS_RSPCOMPILERS` now sets the number of folding threads. The measurements above stand as the record of the pool defect and of what thread count alone could do.

**The prediction that did not hold.** Before the table was measured, the expectation was that compile threads could only help, and that the fix would be a matter of taking them off the pool. The rows for four and eight threads retire that: the compiler competes for the same cores the machine runs on. The right number is a property of the core count and of the emulation thread's load. Two is right for this 16-core machine, and it is unmeasured on the low-end x86-64 laptop the project also targets.

**Confirmed over three interleaved rounds** (base and new, both games):
- Mario's worst frame went from 1,666, 1,684 and 1,669 ms to 102, 103 and 105 ms.
- Ocarina's went from 2,285, 2,283 and 2,269 ms to 23, 28 and 35 ms.
- The steady frames were unchanged: Mario 8.68/8.57/8.47 ms against 8.50/8.50/8.43 ms; Ocarina 7.01/6.91/7.09 ms against 7.06/6.99/7.06 ms.
- Mario has more frames over their interval, 26 to 29 against 18 to 20, but the worst is 83 ms late instead of 1.65 s. The single stall used to swallow the lateness of the frames behind it.

**Exactness.** A block runs interpreted until its code is published, so the change moves only *when* code arrives. The state checksum after 1,200 frames is identical in every row above: `0B629FEAC97BA032` for SM64 and `51196CF259709DF9` for OoT.

**What is left, and why it is not this.** Mario's remaining long frames, 36 to 51 after the warm-up, are the emulation thread interpreting RSP work before its blocks exist: `Rsp.StepOne` and the SP's DMA transfers, not waiting. Early in a process that interpreter is also first-tier JIT code. A ReadyToRun build of pacebench, which is how Mistress now publishes (`EmuSen_Settings_Reference.md` §4.42), brings Mario's worst frame to 55 to 67 ms and Ocarina's to 21 to 33 ms, with one to three late frames in Ocarina's twenty seconds.

**Boot itself is not slow.** Measured from the very first frame (`WARMUP=0`):
- Loading the cartridge takes 13 ms for SM64 and 24 ms for OoT.
- The first frame takes 55 to 66 ms.
- Mario's first five frames are 30 to 76 ms each while the boot code is interpreted.

That is about a quarter of a second in total, spent on a black screen. What the frontend adds between choosing a game and the first frame (the resume prompt, a state read, the audio device) is not measured here.

**Test:** `MarsIdleTests.Blocks_compiled_in_the_background_are_compiled_off_the_pool_and_change_nothing`. With background compiling on, it asserts that blocks were compiled, that none was compiled on a pool thread (`Rsp.PoolCompiles`), and that the state equals the single-stepped one. A mutant that restores `Task.Run` fails it.

### 11.2 Two tiers: a block compiled cheaply at once, and folded when hot (2026-09-22)

**The report.** On a Legion Go S (Ryzen Z1 Extreme, SteamOS, RADV), Ocarina of Time's intro was "very slow" after the file-select screen. The case was a state the player made there, with A pressed to load a save. It was run headless on the device over SSH with pacebench built ReadyToRun, from that state, with the device's settings (GPU on, 8 RDP workers).

**The build on the device predated §11.1.** It showed a single frame of **5,233 ms** and 58.5% of full speed over fifteen seconds: the pool defect of §11.1, worse on a handheld's slower cores.

With §11.1 the stall went, to 71 ms worst and 93.2% of full speed. But only **24%** of the RSP's steps ran in blocks after fifteen seconds, against 80% on the old build. Another thread count did not help: four compiled more and made the early frames worse, eight worse again.

**The measurement that located it.** The time from shaping a block to publishing it was **29 to 34 ms a block** on the device, and 19 to 23 ms on the desktop's 16-core Ryzen. It did not vary with thread count or with the compile threshold (3, 30 and 200 runs were tried). With two threads that is about 1,100 blocks in fifteen seconds, whatever the scheduling.

The cost is §12's design working as intended. Every instruction is a call to a general handler with the word as a constant. The JIT inlines the handler's decode switch and folds it to the one instruction, and §14 splits the block into methods of four so that the inliner's budget is not exhausted. The code produced is fast. The price is the JIT importing and optimising the whole decode tree once per instruction.

**The same calls, uninlined.** When every handler call goes through a `NoInlining` wrapper, the JIT compiles only the calls:

| Blocks | Compile a block | Legion Go S steady frame | Desktop steady frame (SM64) |
|---|---|---|---|
| folded (§12) | 29 ms | 3.40 ms | 8.84 ms |
| uninlined | **0.38 ms** | **3.21 ms** | 9.37 ms |

Uninlined blocks compile 55 to 76 times faster. On the handheld they are not even slower, because far more of them exist in time. On the desktop they are about 5% slower once everything is compiled. §12's gain is real where the machine has time to buy it, and a loss where it has not.

**The change: two tiers.**
- A block that has run three times is compiled uninlined on the "Mars RSP block compiler" thread. It is published within about half a millisecond.
- A block that has then run `FoldAfter` times is compiled again, folded, on a separate "Mars RSP block folder" thread, and its code is swapped in with a volatile write.

Both tiers call the same handlers in the same order, so they compute the same state. Only the inlining differs.

`FoldAfter` was chosen by measurement:

| FoldAfter | Blocks folded (device) | Device steady frame | Desktop SM64 steady frame | Desktop SM64 worst frame |
|---|---|---|---|---|
| 500 | 551 | 2.96 ms | not measured | not measured |
| 2,000 | 267 | 2.88 ms | 9.23 / 8.94 ms | 30 / 31 ms |
| **10,000** | 99 | **2.64 ms** | **8.86 / 8.82 ms** | 29 / 30 ms |
| never | 0 | 3.56 ms | 9.32 / 9.36 ms | 33 / 30 ms |

At 10,000 the desktop's steady frame equals the folded-only build's (8.84 and 8.81 ms). Its worst frame falls from about 104 ms to about 29 ms, and its mean over 1,200 frames from 8.7 to 7.7 ms. On the device, 99 folded blocks give the best steady frame measured. `EMUSEN_MARS_RSPFOLDAFTER=n` overrides the threshold.

**Why the threshold is high.** Folding is worth doing only for the blocks that carry the time. At 10,000 runs a hundred to two hundred blocks qualify, and folding them costs 2.5 to 4 s of one background thread spread over the first twenty seconds. Lower thresholds fold hundreds more blocks that barely run, and on a handheld those compiles compete for the same power budget as the emulation thread. The number is specific to libultra's microcode mix in these two games, and is unmeasured elsewhere.

**Exactness.** The state checksum after fifteen seconds on the device is `093C9C669DA764B2` in every configuration: the old build, §11.1, both tiers and every threshold. On the desktop over 1,200 frames it is `0B629FEAC97BA032` (SM64) and `51196CF259709DF9` (OoT), unchanged.

**What is left on the device, and why it is not the RSP.** With every block compiled within a second, the first 300 frames after pressing A still average 9 to 10 ms, against 2.6 to 2.8 ms later, with 60 to 80 frames over their interval (worst 40 to 70 ms late). The emulated work in those frames is about twice the steady workload while the scene loads and the CPU's recompiler compiles its own blocks (40 to 100 in the worst frames). Some frames also wait 18 to 64 ms for the RDP.

GPU on or off, and four or eight RDP workers, change it within noise: 9.1 to 10.0 ms mean, 62 to 83 late frames. The device has no profiler that can see managed code, so this was not traced further. It is the next question.

**Tests.** `MarsIdleTests.Either_tier_of_block_and_the_move_between_them_leave_the_state_the_single_steps_leave` runs `FoldAfter` = 1, 50 and never, and compares each with the single-stepped state. A mutant that routes ordinary instructions to the wrong decoder in the first tier fails all three.

A mutant that routes vector loads to the store handler **survives**: the test's RSP program has no vector loads or stores. The first tier's load and store wrappers are therefore covered only by the checksums of the game runs above.

## 12. The handlers folded into the blocks

*2026-09-20.* §11's blocks called each instruction's handler with its word as a constant, and bought less than the
samples promised. `Mars_RspVector.md` §15's benchmark of single operations said why: a vector operation cost about
seven nanoseconds whatever it computed, a bitwise AND 5.4, and through a block only 0.6 less, because the block
removed the fetch and the first dispatch and left the coprocessor's test, the vector function's entry, three field
decodes, the element shuffle and a switch of sixty-four.

**The change is an attribute.** The handlers — the scalar switch, the special and register-immediate groups, the
coprocessor's moves, the vector function, the vector loads and stores — are marked for aggressive inlining, and a
block calls the eight-lane vector function directly (behind a test that that unit is the one in use, since a test
can switch units). The compiler inlines the handler into the block with the word a constant, and folds: the fields
become constants, both switches become their one case, the selection's test for the whole register is decided, the
accumulate and signedness flags are decided. No handler was rewritten and nothing was copied; an instruction in a
block and the same instruction interpreted are still one piece of source.

**Measured by operation, through blocks, nanoseconds a step, before and after:** a multiply-accumulate 6.7 to 3.2;
an add 6.1 to 2.7; a bitwise AND 4.8 to 0.95; an add-immediate 2.5 to 2.0; a load word 4.2 to 3.5; a shift 1.8 to
1.1; the quad load 14.6 to 3.1 and the quad store 11.9 to 2.3 (the last two are §15's fast case, not the inlining).

**Measured in the games, it is two to three per cent**, five rounds interleaved against a build of the commit
before, medians of the mean frame: Ocarina of Time 8.34 to 8.20 milliseconds, Wave Race 7.04 to 6.80, Super Mario
64 4.80 to 4.71, the state hash after 1,200 frames identical in every run of both builds. An interrupt sample put
the vector unit at 16 per cent of the thread where it had been 26.

**Why so little of it reached the games, and a claim corrected.** Counting how each run of blocks ends, and how
many steps are taken singly, in 900 frames of Ocarina of Time: 338 million steps in blocks, 24 million stepped
singly inside the idle loop (after a branch's slot, a shape the compiler refuses, a budget too short, an event) —
and **99 million, a fifth of all the processor's steps, taken outside the idle loop altogether**, one call from
the CPU's tick for every instruction the CPU runs while the processor is running. `Mars_Performance.md` §38 said
every step is taken inside the idle loop; its census counted a block's instructions by whether the processor was
running when the block *ended*, and the blocks that run beside it are the operating system's own, short and
frequent, which that count missed. The conclusion drawn from it, that the processor has no CPU work to overlap
with, stands for four fifths of its steps and not for the last fifth.

**What that fifth would take.** Those steps cannot simply be run ahead or owed: the CPU can read the processor's
memory and its program counter at any instruction, and the processor's events must land on the instruction they
always landed on. Running ahead to the next event would be exact only with a way to undo it when the CPU looks,
and a copy of the processor's state at every block entry costs more than it saves. It is the next problem on this
thread and is not begun.

## 13. The fifth of the steps taken beside the CPU's own code: two routes measured, neither kept

*2026-09-20.* §12 left 99 million steps in 900 frames of Ocarina of Time, a fifth of the processor's work, taken
one at a time from the CPU's tick while the CPU runs real code. Two ways of making them cheaper were tried the same
day. Both were measured before anything was kept, and neither is.

**Route one: every instruction slot compiled, so a single step is its handler with the word folded in.** This keeps
the interleave exactly as it is — one step a cycle, in the same order, so it is exact by construction, and the state
hash after 1,200 frames was identical in all three games — and takes §12's folding to the steps a block cannot
cover. A table of compiled one-instruction methods, four to a slot found by the word, the step moving its own
counters and then calling the method. *It made the games slower*: four rounds interleaved against the commit
before, Ocarina of Time's mean frame 8.29 to 8.80 milliseconds, Wave Race 6.91 to 7.30, Super Mario 64 level. A
benchmark of single steps put the fixed cost of the compiled step at seven to eight nanoseconds whatever the
operation — the table's compare, the delegate, its shuffle thunk and an indirect call whose target changes every
step — against an interpreted add-immediate at 3.6 and a vector AND at 5.3. Folding pays inside a block because
sixteen instructions share one call; a step at a time, the call costs more than the two switches it removes. One
thing from it was worth knowing: inlining the handlers into the *interpreter's* dispatch as well makes that one
method enormous and slow, so §12's attribute must stay off the path the interpreter takes. It does: the
interpreter's `Execute` is not inlined into the step, and only emitted code names the handlers directly. *(Corrected
2026-09-21, §14: the disassembly of the step, `Cpu.RspRan`, shows `Execute` inlined into it, with the vector unit
behind it, in 5,061 bytes; what this sentence describes is what §14's cold paths do for the blocks.)*

**Route two: the processor run ahead of the CPU to its next event, and the CPU's ticks paid from the steps already
run.** The pure instructions between two events touch nothing the CPU can see unless it reads the processor's
memory, its program counter or writes its status, so the processor could run them early in blocks and the tick
become a subtraction. To be exact it needs the events never run early (blocks ending before them, not after), a
record of the processor's registers and the parts of data memory it has written since the last event, and an undo
that restores them and steps forward again whenever the CPU touches the processor while steps are owed — at a
state, a debugger's read, a status write. Events come about every fifty steps in the graphics microcode, so the
record is taken that often. Before any of that was built, a prototype with no record and no undo — inexact, and
flattering, since it pays none of those costs — was switched on in the same build: Ocarina of Time's mean frame
fell 7.5 per cent, **and its drawing frames two**, 18.78 to 18.34 milliseconds at the ninetieth percentile. The
steps taken beside the CPU's own code are mostly the sound task's, in the light fields, which cost four
milliseconds against a slot of twenty. The frames that are over their slot are the drawing frames, and this route
gives them a third of a millisecond at best, before its overhead. (Wave Race ran 15 per cent *slower* under the
prototype, which says only that a game whose events land at the wrong cycles does different work; it is why an
inexact prototype can bound a gain and cannot measure one.)

**Why neither is kept.** The first loses. The second, at its ceiling, improves the frames that need it by two per
cent, and costs the most delicate machinery on this thread: an undo that has to be right at every place the CPU can
look at the processor. The exactness contract is the project's first rule and this would put the most weight on it
for the least return of anything measured this phase. If the light fields ever matter — a slower host, where four
milliseconds is not nothing — the prototype's number is here to start from.

**What the drawing frames are made of now** is not this processor so much as the CPU: by an interrupt sample with
the scan-out off, the CPU's interpreted instructions 17 per cent of the thread, its blocks 13, the dispatcher's
entry 8 and software floating point 7, against the vector unit's 16 and the scalar half's 14. The next gains for
Ocarina of Time's drawing frames are on that side: the coprocessor's arithmetic, a sixth of what reaches the
interpreter's switch, and the register jumps, links and likely branches (`Mars_Recompiler.md` §16).

## 14. The folding that the inliner undid, and the byte loads taken whole (2026-09-21)

**Measured first, by the processor's own counters and a census** (600 frames of each of the three play states,
instructions retired on the emulation thread by `perf_event_open`): 81, 73 and 81 per cent of the processor's steps
already run in blocks in Super Mario 64, Ocarina of Time and Wave Race. But in those blocks **45, 53 and 46 per cent
of the vector operations executed, and a third of the vector loads and stores, were still full calls** into the
vector unit rather than §12's folded code. In the hottest block, of nineteen vector operations eight were folded and
every one after them a call. The compiler stops inlining a method at about five hundred inlinees, and §12's folding
inlines a handler, and everything the handler inlines, for every instruction of a block. Two things spent that
allowance early: each vector operation's path for when the eight-lane unit is not the one in use inlined the whole
unit a second time, and `TransferOperands` was never inlined, so a vector load or store kept its format switch. The
sampler had also been filing the interpreted steps under the CPU: counted where they run, the processor is about
42 per cent of Mario's emulation thread, not 28.

**The changes, all exact by construction:**

- A block longer than four instructions is compiled as methods of four, each with the compiler's allowance of its
  own, called in order from the block's method. A branch whose slot begins the next method carries its flag across.
- The fallback when the eight lanes are not in use goes through a method that is never inlined.
- `TransferOperands` is inlined, and its scale is a nibble of a constant rather than an entry of an array, so a
  block's word folds the format away.
- The scalar half's small helpers (the register reads and writes, the address, the data reads and writes, the
  branches, jumps and links) are marked for inlining.
- **The byte, short, word and double loads and stores take whole lanes** when the element is even, the bytes do not
  run past the register and the address does not wrap: one unaligned read, each pair of bytes exchanged, one write,
  where the loop took each byte with two loads, a bounds check and a read-modify-write of half a lane. It is taken
  on a little-endian host only, as the quad fast case is.

Of the operations still called rather than folded, 45.5 per cent fell to 5.0 in Mario's blocks. Instructions retired
on the emulation thread fell 17.9, 11.1 and 16.6 per cent in the three games, and the blocks' machine code grew from
2.56 to 2.89 MB.

**The time is less than the instructions**, as it should be read. `pacebench`, flat out, eaed28d against this, five
rounds alternated, medians, every state hash the same: Mario at one ran at 354 per cent of full speed against 340,
Wave Race at 285 against 259, Ocarina at 256 against 253. Ocarina's frames are bound by its CPU and the interface's
waits more than by this processor. The golden probe was identical.

**Tested.** `The_small_loads_and_stores_move_the_bytes_their_definition_moves_at_every_element` runs the four small
formats, loading and storing, at every element and at the start, the middle and the end of data memory, against the
definition byte by byte; three mutants (no exchange of the bytes, odd elements taken whole, a wrapping address taken
whole) each fail it. The chunking, the cold path and the inlining change which code the compiler emits and nothing
it computes, and the state hash over 600 frames of each game and the whole Mars suite hold them.

**Measured and not kept:** marking the nineteen vector arithmetic helpers for inlining as well made the count worse,
since it spent the allowance again; moving the program counter once a block instead of once an instruction, and
delegates closed over nothing to avoid the shuffle thunk, each changed the count by under half a per cent; routing
the interpreter's own vector cases through cold wrappers changed it by under one.

**What this does not cover.** A fifth of the steps are still taken one at a time beside the CPU's own code (§13), and
the entry to each block still compares its image against instruction memory.

