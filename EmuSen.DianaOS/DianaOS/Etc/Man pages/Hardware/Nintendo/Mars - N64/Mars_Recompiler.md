# Mars — the recompiler: blocks between the interpreter's checks

*Phase G, 2026-09-18. The code is `Cpu/Blocks/` (the shape, the cache, the emitter) and `Cpu/Core/Cpu.Blocks.cs` (the
dispatcher); the tests are `EmuSen.WiseMan/Cores/MarsBlockTests.cs`; the measurement that preceded it is §7, and the
switch build behind that measurement is kept with the speed tooling outside the repository. The rule it works under is
`Mars_Performance.md` §0: nothing a game can observe may change, and the grade is the probe's whole save state, frame by
frame.*

---

## 0. What it is, and the claim it makes

The interpreter runs one instruction a step: it checks for an interrupt, fetches, decodes, executes, ticks the devices,
checks the timer. `Mars_Performance.md` §20 measured that the fetch and decode are about a quarter of a step and the
checks around the instruction about a third, and that a cache of decoded instructions recovers little of either,
because its lookup and its dispatch cost most of what decoding does. What recovers both is code that *is* the
instruction — a run of instructions compiled into one method, with the checks made once for the run where they cannot
change and at the exact instruction where they can.

The claim is narrow and it is the whole design: **a block leaves the machine in the state the interpreter would have
left it after the same instructions, at every point the machine can be observed.** Not "the same output"; the same
registers, the same devices, the same cycle count, the same save state. Every mechanism below is there to make one
observation point exact, and §5 is how Debug builds prove it on every test that runs a frame.

What it is not: a register allocator, a block chainer, or an optimiser. Every block calls the interpreter's own
opcode methods for anything that can fault, branch, or touch a device, passing the constant word; only the arithmetic
that can neither fault nor touch anything outside the register file is emitted inline. The speed comes from removing
the fetch, the decode and the per-instruction checks, not from doing the instruction's work differently. That was
deliberate: the interpreter's opcodes are what the corpus graded, and a block that calls them cannot get an instruction
wrong on its own.

## 1. Where blocks apply

A block is compiled for code in RDRAM reached through a kernel direct address (KSEG0 or KSEG1), in kernel mode. That is
where every game runs after IPL3 has copied it in, and where the corpus runs. Everything else — user and supervisor
mode, code in the signal processor's memories (IPL3 itself, for a few thousand instructions at boot), code the TLB
maps, an unaligned program counter, an instruction that is the delay slot of a branch already taken — runs through the
interpreter, one instruction at a time, from the same dispatcher (§3.1). Nothing is compiled speculatively and nothing
is compiled twice for the two direct segments: the block is keyed by the physical word address and takes its virtual
addresses from the program counter it was entered with, so the same code reached through KSEG0 and KSEG1 is one block.

## 2. A block

A block is a run of consecutive words starting at one address, with the bytes it was compiled from kept beside the
compiled method.

### 2.1 Its shape

The shape is decided by scanning forward from the start:

- **A branch or jump ends the block after its delay slot.** All of them: the four relative forms and their likely
  forms, REGIMM's eight, `J`, `JAL`, `JR`, `JALR`, and the coprocessor-1 branch. The slot is part of the block, since
  what the slot does after a branch is what the interpreter's two-program-counter model does (`Mars_Cpu.md` §3), and a
  block reproduces that model with the same two fields. (§10 measured going on past a conditional branch that fell
  through, and did not keep it.)
- **A COP0 write, a TLB instruction or `ERET` ends the block after itself**, because each can change the privilege
  mode, the timer, or the interrupt check's inputs, and the next instruction must be dispatched with those seen afresh.
  A COP0 read does not end it.
- **`SYSCALL`, `BREAK`, and a word the interpreter would refuse as reserved end the block after themselves**, because
  nothing after them in the block can run.
- **Sixty-four instructions end it**, a cap on what one compilation costs, never between a branch and its slot.
- **The end of memory ends it.**

### 2.2 Refused shapes

A branch whose delay slot is itself a branch, a jump, or one of the enders above is not compiled. The interpreter's
behaviour there is defined (the slot's branch simply wins, as `Mars_Cpu.md` §3 has it), but the block would have to
reproduce a branch taken inside a slot with the fields half-updated, and no game does this. If the branch is the first
word, the entry is marked refused and runs through the interpreter every time; otherwise the block is cut short before
the branch, and the branch gets its own refused entry.

### 2.3 The cache, the threshold, and validation on entry

Entries live in a two-level table by physical word address, a page of a thousand and twenty-four entries allocated when
a page first holds one, so that a game's four or eight megabytes cost nothing until code runs there. An entry begins
cold: its shape is known but nothing is compiled. Each dispatch at its address runs it through the interpreter along its
own straight line — no further than the shape, and no further than the instruction where the program counter left that
line — and counts the run. **The sixty-fourth run hands it to the compiler** (§2.4), and the entry keeps running through
the interpreter until the compiled code is published. Cold code — a boot's straight line, an error path — is never
compiled at all, and the threshold is what §7's sweep chose.

**Every entry to a compiled block first compares the block's bytes with memory.** This is the invalidation design, and
it is worth being explicit about why it is the whole of it. The alternative — flagging pages that hold code and having
every writer of RDRAM consult the flags — is exact only if every writer is found: the processor's stores, the three DMA
engines, the display processor's spans, the serial interface's transfer, the cheat patcher, the debugger's poke, a
loaded state. Any writer missed is a block executing code that is no longer there, which is exactly the class of
divergence `Mars_Gameplan.md` §2.2 warned this phase about, and it is silent. A comparison on entry needs no inventory:
whoever wrote the memory, the block sees it before it runs. It costs one vectorised compare of at most 260 bytes a
block, a fraction of a nanosecond an instruction at the block lengths games have. A block whose bytes have changed is
dropped and its entry begins cold again; a game that reloads an overlay into the same address — Ocarina of Time does
this constantly — gets the new code compiled once it is hot, and never the old.

**The shape is taken again at the threshold.** *Added 2026-09-19, after a defect.* The paragraph above was true of the
words and not of the shape. An entry's shape — its length, where its branch and slot fall, whether it is refused — was
computed from the words at its first run and kept, while its image was taken from the words at its sixty-fourth. A block
still cold when an overlay was loaded over it kept the old shape and was compiled from the new words with it: a branch
the new code has in the middle of what had been a straight line was emitted as if it ended nothing, and after its slot
the block ran on through instructions the branch had skipped. The comparison on entry cannot see it, because it compares
memory with the image, and the image is the new words. A test demonstrated it before the change — a routine called
twenty times, replaced by one whose first instruction branches over three adds, and called a hundred times more: the
interpreter never ran the adds and the blocks ran them fifty-seven times. At the threshold the dispatcher now shapes the
words again, and a shape that differs replaces the entry and starts its count over. Length and refusal are the whole of
what the emitter takes from a shape; the kind of every instruction it decodes from the image, so a shape that matches in
both is a shape the image can be compiled with. The defect was reached from a game, not from the lab: Ocarina of Time's
opening diverged between a run that carried a history of blocks and one started from a saved state with none (§13).

What a comparison on entry does not cover is memory that changes *while the block runs*. §4 is that.

### 2.4 Compiled on another thread

Compiling a block means the JIT compiling a `DynamicMethod` at full optimisation, and §7 measured that at about a
millisecond a block — three orders of magnitude more than emitting its IL, and the whole of what the first version of
this page's subject lost in Wave Race. A millisecond on the emulation thread is a visible hitch, and a scene that brings
a few hundred new blocks would stutter for as long as they compiled. So the processor's thread never compiles. When a
block reaches the threshold it takes a copy of its bytes, and the block, its cache and the copy go on a queue that one
background thread drains: the thread emits the method, creates the delegate, forces the JIT with
`RuntimeHelpers.PrepareDelegate` — measured to move the whole cost off the first call, §7 — and publishes the delegate
with a volatile write. Until then the processor's thread keeps interpreting the block, which it was doing anyway.

What makes this safe to reason about is that nothing the compiler's thread touches is shared with the running machine:
it compiles from the block's own copy of the bytes, taken on the processor's thread before the block was queued, and
the processor's thread compares that same copy with memory before every entry, so a block whose code changed while it
was in the queue is discarded on arrival like any other. The counts of what was compiled and what it cost are
interlocked; nothing else crosses. The dispatcher reads the published reference once per dispatch: a block that arrives
between one dispatch and the next is run at the next, after the comparison, never in the dispatch that first found it
missing — the first version read it twice and could have run a block once uncompared, in the window between the two
reads, which no test could have shown and the review found. The result of emulation cannot depend on when the code arrives, because interpreted
and compiled are the same machine by construction — §6 has a test that runs a block both before and after the thread
publishes it and compares the state at the end — and `Cpu.CompileInBackground` turns the thread off for the tests that
count what was compiled.

## 3. The dispatcher

`Cpu.StepBlock` runs one block, or one instruction where no block can (§3.1). `MarsCore` calls it from the same quiet
loop that `Mars_Performance.md` §13 introduced — the loop with no debugger in it — and only there: when anything is
armed that has to see every instruction, the frame runs through `Cpu.Step` as before, and `MarsCore.UseBlocks` turns
the blocks off outright, which is what the differential tests use.

### 3.1 What runs through the interpreter

Before looking up a block, the dispatcher steps the interpreter once instead if a branch is pending (the instruction
about to run is a delay slot and belongs to the block that branched), the mode is not kernel, the program counter is
outside the direct segments, the address is not in RDRAM, or it is misaligned. A cold or refused entry runs its
straight line through the interpreter as §2.3 says. In every case the instruction executes exactly as it would have with
no blocks at all, because it is the same code.

### 3.2 The entry and the exits

A block begins as a step begins: `CurrentPc` is the program counter, `InDelaySlot` is false (a pending branch never
reaches here), the interrupt check runs if `_recheck` is set or the MI line has changed since it last ran, and an
interrupt taken there is the same exception the interpreter would take before the same instruction. Then the compiled
method runs.

Inside it, each instruction advances the cycle count, the instruction count, and the signal processor when it is not
halted, in the order the interpreter's tick does. What the interpreter checks after every instruction — is a device
event due, is the timer due — a block checks with a single compare against a **stop cycle** computed on entry: the
earlier of the bus's next event, the timer's due cycle, and the frame's cycle cap. That one number is loop-invariant
inside a block by construction: the next event changes only when a device register is written (a store through the bus,
which ends the block, §4) or when the events run (which happens only at an exit); the timer's due cycle changes only on
a COP0 write (which ends the block) or when the timer fires (an exit); the cap is the frame's. When an instruction ends
on or past the stop, the block exits after it, and the exit does what the interpreter's step would have done at that
point: runs the events if due, fires the timer if due, settles the count. The event therefore runs after exactly the
instruction it would have run after, and the next block's entry check sees whatever it raised.

The other exits are the shape's ends (the slot after a branch, an ender), a likely branch not taken (the interpreter has
already moved the program counter past the slot; the block returns and the dispatcher continues from there), a store
that may have touched code or a device (§4), and — while the signal processor runs — an RSP step that wrote memory or
raised the MI line (§4). Every exit leaves `Pc`, `NextPc`, `CurrentPc` and `InDelaySlot` as the interpreter would have
left them after the same instruction, so that a save state written between blocks is the state the interpreter writes.

An exception thrown inside the block — a fault, a trap, an interrupt taken at entry — is caught by the dispatcher
exactly where the interpreter catches its own: `LastException` is recorded, the exception is entered, one cycle is
ticked. The instructions before the fault have already advanced the counts; the faulting one has not, as in the
interpreter, where a faulting step never reaches its increment.

### 3.3 What the emitted code does for each instruction

| kind | how it executes | before it | after it |
| --- | --- | --- | --- |
| **pure** — shifts, the non-trapping adds and subtracts, the logic ops, the compares, the immediates, `LUI`, `MFHI`/`MFLO`/`MTHI`/`MTLO`, `SYNC` | inline IL on the register array, register zero never written | nothing | tick, stop compare |
| **call** — loads, the trapping arithmetic, the traps, `CACHE`, the coprocessor moves and arithmetic, `LL`/`LLD` | `Execute(word)`, the interpreter's own switch | `CurrentPc` | tick, stop compare |
| **multiply, divide** | `Execute(word)`, then `_extraCycles` cleared | `CurrentPc` | tick charged the vendor's stall, stop compare |
| **store** — every aligned, unaligned, conditional and coprocessor-1 form | the interpreter's own method, returning where it landed | `CurrentPc` | tick, the store check (§4), stop compare |
| **branch, jump** | `Execute(word)` | `CurrentPc`, `Pc`, `NextPc` | tick; exit if the slot was nullified; else the slot, which sets `InDelaySlot` from the pending flag and moves the counters as the step does |
| **ender** — COP0 write, TLB, `ERET`, `SYSCALL`, `BREAK`, reserved | `Execute(word)` | `CurrentPc`, `Pc`, `NextPc` | tick, exit |

Every exit of every instruction is a two-instruction stub placed after the block's hot path, naming the instruction it
exits from; one epilogue per block stores the counters from that index and calls the step's tail. `CurrentPc` is stored
before anything that can raise, and once more in the epilogue so that a state saved between blocks carries the address
of the last instruction run, as the interpreter's does. `Pc` and `NextPc` are stored before the
branch family and the enders, which read them, and at every exit; the pure and call kinds never need them mid-block,
because nothing they call reads them and a fault reads only `CurrentPc`. The delay slot after a `JR` through `ra` runs
the return observer the interpreter runs there, with the interpreter's own condition, so `bt` sees the same stack.

### 3.4 A loop that stays in its block

The first measurement of a game (§7) found Super Mario 64's title screen running two instructions a block: the idle
thread's `b .; nop`, which the shape rules make a block of exactly a branch and its slot, and which paid a whole dispatch
for every iteration. So when the delay slot of a block's last branch leaves the program counter on the block's own
first address, the block does not return. It does instead, inline, what the dispatcher would have done at re-entry —
and nothing else, because everything else the dispatcher checks is invariant here: the branch is consumed, the mode is
kernel, the address is the one it was entered at, the memory has not changed under it or a store or an RSP step would
have exited, and the interrupt check's inputs have not changed for the same reason. It clears the delay-slot flag, sets
the current address, compares the cycle count with the stop, and jumps to the top. Loops whose head is not the block's
start get there by themselves: the first pass exits to the head, the head becomes a block of its own, and from then on
the loop is that block. Debug builds run the verifier at the loop's back edge as at every other instruction (§5).

## 4. Memory that changes while a block runs

Three things can change memory, or what is due, between one instruction of a block and the next, and each has its own
exit.

**The processor's own stores.** Every store method now returns the physical address it wrote. The direct path to RDRAM
(`Mars_Performance.md` §17) returns where it wrote; the path through the bus — a device, the cartridge's latch, the
signal processor's memories, an RDRAM store the MI repeats or a watcher is reporting — returns a value beyond any memory.
After the store the block exits if that value is beyond memory (a device write may have rescheduled the VI or AI,
raised or cleared an interrupt, started a DMA, or started the signal processor; the next entry sees all of it) or if it
is within eight bytes before the block's own start and before its end — the block may have rewritten a word it has
already compiled, and the next dispatch will compare and find out. The eight bytes are slack for a doubleword store
landing just under the start; a spurious exit costs a dispatch and nothing else. A conditional store that did not land
returns address zero, which lies inside no block but the one at the exception vectors, where it costs the same spurious
exit. The unaligned and coprocessor-1 stores write through the bus but return their real address, since neither the
MI's repeat nor the reporting path applies to them (`Mars_Memory.md` §8.4 and §16 of the performance page say why).

**The signal processor.** It runs in step with the processor, inside the block, and can DMA into RDRAM, feed the display
processor (which writes RDRAM and raises the DP line), and raise the SP line on its break. The bus counts every write to
memory that is not the processor's own direct store — its `Write32`, the serial interface's transfer, each display list
the display processor takes, the cheat patcher, the debugger's poke, a loaded state — and after each RSP step a block
compares the count and the MI line with what it saw at entry, exiting on either. That is one load and one compare each,
paid only while the signal processor is not halted.

**Between blocks**, the comparison on entry (§2.3) covers every writer without naming any.

## 5. What Debug builds prove

The interpreter's own skipped-check verifier (`Mars_Performance.md` §10) throws on the first instruction after a COP0
write that bypassed `Cpu.Cop0Written`. A block extends it: in Debug builds the emitted code calls `VerifyBlockStep`
after every instruction it runs on from, which throws if `_recheck` is set or the MI line differs from what the last
check saw — that is, if the interpreter would have acted before the next instruction and the block did not exit — and
then runs the interpreter's verifier and the mode check. The whole test suite is a Debug build, so every frame of every
game the suite runs, and the corpus, runs with this on. It is the mechanism by which the argument of §3.2 — that the
stop cycle is invariant inside a block — is checked rather than believed, and the mutation round in §6 shows it firing.

## 6. Tests

`MarsBlockTests` compares the machine after the same instructions two ways: `Cpu.Run` and `Cpu.RunBlocks`, then the
processor's and the bus's serialised state, byte for byte. That is the probe's grade at unit-test size. The cases are
the mechanisms above, each one an observation point:

- a hot loop compiled, with the threshold checked one run short and one run on;
- a block published by the compiler's thread while the loop runs, and the state after it arrives;
- the VI's and AI's events landing inside a block; the timer's interrupt; the signal processor's break interrupt while
  a block runs — each taken on the same instruction as the interpreter;
- a fault in the middle of a block, with the handler's view of `EPC` and the faulting address;
- a block storing into its own words, seen by the next pass; code replaced under a compiled block, compiled again from
  the new words; a store the MI repeats, landing on the block from eight bytes before it; the signal processor's DMA
  rewriting a running block's code — the last two written for the mutants of §6.1 that escaped the first round;
- a likely branch not taken; a branch in a delay slot, left to the interpreter; `JAL` and `JR` reaching the call-stack
  observers the same number of times;
- the multiply and divide stalls; a store to the VI mid-loop, with the rescheduled field kept;
- and two `MarsCore` cases, five frames of a synthetic program and three frames ending on the cycle cap, saved and
  compared as whole states with `UseBlocks` on and off;
- from the rounds of §10 and §12, kept as differential checks: a loop with a branch inside it, stopped once on a
  fallen-through branch's slot, and the likely-branch form; two compiled blocks reaching each other, frames ending
  between them, one block rewriting another's first word, the timer and the devices' events landing between blocks, a
  fault on a block's first instruction, a register written by a call into the interpreter; and the dispatcher's own
  address rules — a misaligned target, a target outside the direct segments, a branch to its own delay slot (§12.4);
- a routine replaced under a block that was not yet hot, compiled from the new words with the old shape (§2.3, §13).

The synthetic image's program has to be carried into RDRAM by a stub, because the boot runs an image's first kilobytes
from the signal processor's memory, where no block applies — which is also why the existing core-level tests run the
interpreter and are not evidence about blocks. The commercial-ROM tests and the corpus are: they run from RDRAM.

`Cpu.RunBlocks(n)` runs exactly `n` instructions by giving each block a cycle cap of the instructions that remain: an
instruction costs at least a cycle, so a block that must stop at the cap has run at most that many. Its first version
gave no cap and hung on the first test whose program ends in an idle loop, once loops stayed in their block (§3.4) —
the frame loop always passes the frame's cap, so the hang was the test API's alone, and it is recorded here because the
loop-back is exactly the kind of change that turns a harmless omission into one that is not.

### 6.1 The mutation round

Thirty-three mutants of the dispatcher, the emitter, the shape and the store return, each run against every Mars test
(`mutate_blocks.py`, with the workbench scripts outside the repository). Twenty-eight were caught. Four of those were
caught only by §5's verifier — the entry check removed, the RSP raising the line ignored — which is the verifier doing
the job it was written for: proving a skipped check would have acted where no test happened to look. Two were caught by
a hang rather than a failure: with the frame's cap out of the stop cycle, a frame whose VI is never programmed never
ends, and the runner now counts a run past ten minutes as caught and kills it.

Five escaped the first round and were tests that did not yet exist rather than mechanisms that did not matter: the
bus-path store's sentinel (a store the MI repeats, landing on the block from before its slack), the RSP's writes ending
a block (its DMA over a running block), another privilege mode left to the interpreter (a kseg0 fetch in user mode
faults; a block would have run it), a frame ending inside a cold run (a 123-cycle field), the count settled by a fault
that ends a block, and the inline arithmetic's widths (`SRA` shifting the whole register, `ADDIU` sign-extending,
`SLTIU` unsigned, register zero never written). Each has its test now (§6), and the round run again caught every one
of them with the test written for it.

Three escaped and are recorded as what they are. *A branch in a delay slot compiled rather than refused* is equivalent:
the slot calls the interpreter's own branch with the fields exactly as the interpreter has them and the block ends
right after it, so the refusal is a guard against a shape whose enders would not be equivalent, not a correctness
condition for branches. *An interpreted run that does not stop where its straight line does* is equivalent too — every
step of it is the interpreter — and exists to keep entries at block starts, not for exactness. *The published code read
twice a dispatch* is the race of §2.4, which no test can time.

## 7. Measured

**Before the emitter was written**, the shape was measured the way §20 measured the cache: the three loops of the
tickbench — a three-instruction ALU loop, a five-instruction load–add–store loop, a twelve-instruction loop — were
compiled by hand into exactly the methods §3.3 describes, dispatched by exactly §3.2's dispatcher, and run beside the
interpreter for sixty million instructions each with the VI and AI running. The state was identical after every run —
registers, cycles, VI fields, AI samples — and the time, best of five, three runs:

| ns an instruction | interpreter | hand-compiled block |
| --- | --- | --- |
| three instructions (add, branch, slot) | 5.43 – 6.42 | 2.05 |
| five (load, add, store, branch, slot) | 6.60 | 1.80 |
| twelve (ten arithmetic, branch, slot) | 5.77 | 1.14 |

The per-instruction work in a block is about a nanosecond; the rest is the dispatch — the lookup, the comparison, the
entry check, the delegate — amortised over the block's length, which is why the twelve-instruction loop is under half
the three-instruction one. The interpreter's first row varied between runs the way `Mars_Performance.md` §12 records;
the block rows did not.

**The first game measurement, and what it cost to read.** The emitter's first version, compiling on the processor's
thread after eight runs, was run for three hundred frames of two games in a scratch harness that counts what the blocks
do. Super Mario 64 came out *slower* than the interpreter, and the counters said why in one number: 2.1 instructions a
block entry, the idle loop of §3.4. With the loop-back that game rose to 51 fps against 44. Wave Race then came out at
31 against 46, with 44 instructions an entry and 6,006 blocks compiled, and the sampled profile put 63 per cent of the
thread in the dispatcher's own frame with the blocks' methods at a fifth. Two explanations were tried and each was
ruled out by a measurement before the third was found:

- *The blocks' code was too large for the instruction cache.* Each instruction carried its own exit sequence; one
  epilogue per block with two-instruction stubs shrank the emitted IL — and the JIT's summary showed the machine code
  shrank by four per cent, from 1,487 to 1,430 bytes a block, because the JIT had already placed the cold paths out of
  line. The change is kept, since it costs nothing, and it was not the cause.
- *The RSP's stores were ending blocks.* Had the RSP written its data memory through the bus, every store would have
  moved the write count and ended the running block an instruction later for as long as a display list ran. It does
  not; its stores go to the array directly.
- *The JIT.* `CreateDelegate` does not compile a dynamic method; the first call does, from inside the dispatcher's
  frame, which is where the profiler had put the time. Timing the first call instead of the emission gave 5.8 seconds
  of a 10.3-second run — 0.96 ms a block — and a sweep of the threshold confirmed it end to end:

| threshold | blocks compiled | compiling | Wave Race, 300 frames | instructions in blocks |
| --- | --- | --- | --- | --- |
| 8 | 6,006 | 5.8 s | 29.1 fps | 99.8% |
| 64 | 2,873 | 2.6 s | 42.3 fps | 99.3% |
| 512 | 915 | 0.8 s | 57.6 fps | 98.2% |
| 4,096 | 238 | 0.2 s | 62.4 fps | 95.3% |
| the interpreter | — | — | 46.3 fps | — |

A scratch test then showed `RuntimeHelpers.PrepareDelegate` performs the whole compilation — 1.4 ms for a
two-hundred-instruction dynamic method, and a first call of nothing afterwards — which is what §2.4 is built on. With
the compiler on its own thread the threshold stops mattering to the frame rate (60 to 65 fps for Wave Race and Super
Mario 64 at 8, 64 and 512 alike) and matters only to how much of the game runs compiled; 64 keeps the queue short and
the coverage above 96 per cent in both.

**The games, timed the phase's way** (`Mars_Performance.md` §12: interleaved builds, medians of three rounds of 600
frames from boot, against the commit before):

| | before | with blocks | |
| --- | --- | --- | --- |
| Ocarina of Time (PAL) | 31.3 fps | 39.6 fps | +27%, 79% of the console |
| Super Mario 64 (PAL) | 36.8 fps | 49.3 fps | +34%, 99% of the console |
| Wave Race 64 (NTSC) | 36.4 fps | 45.6 fps | +25%, 76% of the console |

Every frame's whole state matched the probe's baseline in all three, in a Debug build with §5's verifier on. The 600
frames include compiling every block the games reached; in the scratch harness's quick runs the same games at 300
frames showed 62, 63 and 49 fps against 47, 45 and 37 interpreted, the difference from the table being the harness's
single runs and shorter horizon rather than anything the blocks do.

**What compilation costs, stated.** About a millisecond of one core per block, off the emulation thread; a few
thousand blocks in a game's first minutes, so a few seconds of a spare core, and about two kilobytes of machine code
each. The cost the emulation thread still pays is the interpreted runs of a hot block between its sixty-fourth run and
the arrival of its code, which the sweep bounds at a few per cent of instructions in the first three hundred frames.

## 8. What it does not do, and what is unmeasured

- **No register allocation, no chaining, no extended blocks.** Registers live in the array between instructions; a
  block ends at every branch, taken or not, except the loop-back of §3.4; a block reaches the next through the
  dispatcher. Each is a known lever and each was left until this one is measured in play. Each was then measured —
  §10, §11, §12 — and none is kept; §12.3 says what the three established together.
- **Only what the interpreter's opcodes do.** Loads, stores, coprocessor work and anything that can fault call the
  interpreter's own methods; the inline set is the arithmetic that cannot. A defect in an opcode is the same defect
  either way, which is the point.
- **Only kernel mode, only direct addresses, only RDRAM.** §1. The corpus's user-mode and TLB-mapped cases run
  interpreted and are unchanged by this page.
- **The debugger's poke from another thread** reaches a running block no sooner than the interpreter's next fetch would
  have seen it, and possibly later by the rest of that block; the comparison on entry catches it at the next. The
  interpreter's own torn read in that race is not made worse, and neither is made right.
- **A conditional store that did not land reports address zero** (§4), which costs a spurious exit in a block at the
  exception vectors and nothing anywhere else.
- **The interpreter's return-observer quirk is kept**: `_returnAfterSlot` set by a `jr ra` whose slot faults survives
  until the next delay slot anywhere. A block reproduces it because it runs the same condition at every slot.
- ~~**3D gameplay is still unmeasured.** Every number here comes from boots, title screens and an attract race, as every
  number in `Mars_Performance.md` does; a scene with more code hot at once compiles more blocks, and the queue's latency
  in such a scene is a number nobody has.~~ Measured on 2026-09-19 from states saved in play (`Mars_Performance.md`
  §26): 6,600, 4,700 and 3,400 blocks compiled in the first 600 frames of a scene, 5.4, 4.9 and 2.8 seconds of the
  compiler's thread, 99.8 per cent of instructions in blocks once compiled, and the processor a fifth of the frame.

## 9. The next levers, measured before any was built

*2026-09-19.* §8 named three: registers kept in locals across a block, blocks extended past untaken branches, and
chaining between blocks. Each was priced on the committed recompiler before a line of any was written, the way §7
priced the recompiler itself.

**Where the block time goes.** A sampled profile of 600 frames, the emulation thread only:

| | Wave Race | Ocarina of Time |
| --- | --- | --- |
| the dispatcher's own frame (lookup, comparison, entry, the call) | 9.3% | 9.3% |
| the emitted code | 5.0% | 7.7% |
| `Execute` and what it calls (the call-outs) | 6.0% | 3.2% |
| interpreted runs and fallbacks (`Interpret`, `Step`) | 6.6% | 5.4% |
| the RSP (`Sp.Step` and the processor) | 29% | 15% |
| the VI | 15% | 30% |
| the RDP | 20% | 25% |

The blocks' own code is the smallest of these. The dispatcher is the largest block-related share, and the counters put
it at 18 to 19 million entries in 600 frames of each game — about 70 ns an entry.

**What an entry costs, and why.** A ring of blocks each ending in a jump to the next, all compiled (`ringbench`),
prices an entry with a given number of blocks live:

| blocks live | instructions a block | ns an entry |
| --- | --- | --- |
| 1 (the loop stays in its block) | 8 | 9 |
| 50 | 8 | 26 |
| 500 | 8 | 43 |
| 5,000 | 2 | 61 |
| 5,000 | 8 | 100 |
| 5,000 | 32 | 345 |
| 5,000 | 64 | 1,133 |
| 200 | 32 | 169 |

Removing the comparison on entry changed 42 ns to 35 with 500 blocks and nothing with 5,000. So the cost is not the
dispatcher's instructions and not the validation: it is memory. About fifty nanoseconds an entry is the block's
metadata brought in cold, and the rest grows with the target's code — six to ten nanoseconds an instruction, up to
twenty-five when 5,000 blocks of sixty-four no longer fit the last cache. The emitted code is 80 to 100 bytes of
machine code an emulated instruction (the JIT's summary: about 1,430 bytes a block), and a block that is not in the
instruction cache streams all of it in before running any of it at a nanosecond an instruction. Games have three to six
thousand blocks live; their entries average 48 to 59 instructions and cost about what the ring's 5,000 rows cost.

**What follows an entry.** Counting, in a switch build, where the next dispatch after a compiled block landed:

| | fall-through past an untaken branch | the ending branch's static target | elsewhere |
| --- | --- | --- | --- |
| Ocarina of Time | 26.5% | 64.8% | 8.7% |
| Super Mario 64 | 24.2% | 57.1% | 18.6% |
| Wave Race | 22.5% | 60.2% | 17.3% |

Extension removes the first column's entries outright — the instructions join the block. Chaining spares the second
column the lookup, the delegate and the entry check, though not the target's comparison or its cold code. Together they
reach about 85 per cent of entries.

**Registers in locals**, hand-written against the emitter on the two ALU loops, identical state after sixty million
instructions: 1.36 against 1.38 ns an instruction on the three-instruction loop (1 per cent) and 0.77 against 0.96 on
the twelve-instruction one (20 per cent). On its own it buys a fifth of the emitted code's 5 to 8 per cent — one to
one and a half per cent of a frame — with the writebacks a call-out forces still to be paid for. What it also buys,
which the hand variant shows and the ring explains, is bytes: a register in a machine register is a four-byte add where
the array is twenty.

**The dynamic instruction mix is not where the time is.** Half the instructions the games run are `nop` and the branch
of an idle loop (pure 49 to 50 per cent, branch 46 to 48, call-outs 1.5 to 4.2, stores about 1) and the idle loop costs
nothing since §3.4. The time is in the other half's entries and the code they stream.

**The order this sets.** Extension first: the smallest change, a quarter of the entries, and the groundwork for longer
straight runs. Chaining second, for the sixty per cent. Then density — the per-instruction tick and check batched over
a run of pure instructions where the RSP is halted and no stop falls inside it, with the run's registers in locals —
which the ring names as the largest lever of all and §8 did not name, since it was not visible until an entry had a
price. Each step is graded as every step of the phase has been: the probe's whole state, then interleaved builds.
Beyond the blocks, the RSP is the largest single cost in Wave Race and the VI's scan-out in Ocarina of Time, and
neither is this page's.

## 10. Blocks extended past a branch that is not taken: measured, and not kept

*2026-09-19, the first of §9's levers.* A conditional branch ends its block after the slot whatever it did, so a loop
body with an `if` in it is two or three blocks, and §9's census found a quarter of all entries following a branch that
had fallen through. The shape scan was made to run on past a conditional branch's slot to the next jump, ender or the
cap; after the slot the emitted code compared `Pc` with the next instruction's address and, on a fall-through, lowered
the delay-slot flag and went on through the usual stop compare; a likely branch that had nullified its slot jumped over
the slot's code, with the stop compare moved ahead of that jump so an event due after the branch still ended the block
there; a taken branch exited or looped as before.

**What it measured.** Over 600 frames: entries fell by a quarter, as the census said they would — 19.0 to 14.5 million
in Ocarina of Time, 18.8 to 13.8 in Super Mario 64, 18.4 to 14.3 in Wave Race, with 77, 79 and 62 instructions an entry
against 59, 59 and 48. Blocks fell in number and grew in size: 2,724 bytes of machine code a block against 1,430, and
the compiler's thread spent 40 per cent longer. Interleaved, three rounds of 600 frames against the commit before:
39.2 → 39.0 fps, 48.1 → 47.8, 44.5 → 44.9. Nothing, in all three games.

**Why, as §9 already said.** An entry's cost is mostly the target's code streamed in cold, and grows with it. Removing
the entries that fell through to the next instruction removed the cheapest ones — their code was adjacent to what had
just run — while every entry that remained now brought in a block nearly twice the size. The saving and the cost were
the same quantity, moved.

**The defect the probe found on the way.** The first version lowered `InDelaySlot` on the fall-through path *before*
the stop compare. A frame that ends on that slot — an event due right after it — then saved the flag down where the
interpreter, whose step had just run a delay slot, saves it up. Nothing behaves differently, the verifier has nothing to
see, and the probe's whole-state hash caught it on frame 342 of Super Mario 64: one byte. A diff harness confirmed the
byte was the flag. The lesson stays though the change does not: it is the second time this page has needed the whole
state rather than the output (§3.3's `CurrentPc` was the first), and the reason §0 asks for it. Two tests from the work
are kept as differential checks — a loop with a branch inside it, stopped once exactly on the slot of a branch that fell
through, and the same with a likely branch — and they pass against the emitter as it was.

**Not kept, and what reopens it.** By the rule of `Mars_Performance.md` §6 a change that measures nothing is not
carried. The trade it lost is a trade on code size, and §9's third lever — density — changes the size side of it;
extension is to be measured again once a block's instructions cost fewer bytes. The switch build and its timings are
with the speed tooling outside the repository.

## 11. Density: the block's registers and counters in locals — measured, and not kept

*2026-09-19, the third of §9's levers, built second.* §9's ring priced a block entry by the bytes of code it streamed in
— 80 to 100 of machine code an emulated instruction — and §10 showed that moving entries around without changing the
bytes changed nothing. This change changes the bytes.

**What moves into locals.** Every general register an instruction of the block names, the cycle count and the
instruction count. The prologue loads them once — the registers eagerly, so that every local is valid from the first
instruction on — and reads the RSP's halted flag once, since the RSP cannot start without a store to its interface,
which ends the block. A pure instruction is then an operation between locals: the JIT keeps them in machine registers,
and an add that was a bounds-checked array load, an add and a bounds-checked store is an add. The tick is an add to a
local; the stop compare is a compare between locals; the halted test is a test of a local.

**Where the array and the fields are brought up to date.** The claim of §0 is about what can be observed, and the
places the interpreter's state can be observed inside a block are exactly these:

- *Before any call into the interpreter* — a load, a store, a branch, a coprocessor instruction, a trap — the cycle
  and instruction counts are written to their fields and every register whose local differs from the array is written
  back. The callee may read any of them; an exception out of it reaches the dispatcher's handler with the locals gone,
  and the array and the fields must already hold what the instructions before the fault left. The dirty set is known at
  compile time along the straight line, so a call costs a store for each register changed since the last, and no more.
- *After a call that writes a register* — a load's target, a link register, a trapping add's destination, a
  coprocessor move's — that register's local is reloaded from the array, so every local is valid again. Which register
  a call writes is decided by the opcode, in one table.
- *At every exit*, the epilogue writes the counters and every local back, changed or not; a value that matches the
  array is written to it harmlessly, and the shared epilogue needs no per-exit knowledge of what changed.
- *At the loop's back edge* (§3.4), the dirty registers are written back so that the top of the block, which the
  prologue reaches with nothing dirty, is reached the same way from below.

The halted flag is the one local with a subtlety of its own, and the RSP test found it on the first run: a store to
the signal processor's interface starts it, and the interpreter's tick after that very store steps it once. The block
exits after such a store, but not before ticking it, so the flag is re-read after every store — the only kind of
instruction that can start the RSP inside a block. The symptom was the RSP's program counter one instruction behind
after 65,000 of the CPU's, everything else equal.

A trim tried on the way is worth recording because two tests refused it within a second: the instruction count is read
inside a block only by coprocessor-zero instructions, so it was written back only before those. But a fault out of a
load or a store never reaches the epilogue, and the count the interpreter leaves after a fault is the count of the
instructions completed before it — which the field did not hold. Both counters go back before every call, for the
same reason the dirty registers do.

Nothing else inside a block reads the array or the counters: the RSP step reads neither (checked, since the RSP runs
inside the block), the return observer reads neither, and the Debug verifier compares the count with a bias derived
from the same field, so a stale field agrees with itself.

**What it does not change.** `Pc`, `NextPc`, `CurrentPc` and the delay-slot flag stay fields, since a call may read any
of them and they are set only where §3.3 says. `Hi` and `Lo` stay fields; the multiply and divide are calls. The slot
preamble, the store check, the stubs and the loop-back are as they were. A register the block does not name has no
local and is read and written in the array as before, which is how the emitter stays correct if the register table
under-counts.

**What it did to the bytes, which was not what §9 expected.** The JIT's summary on Super Mario 64's first hundred
frames: 1,810 bytes of machine code a block against 1,430 before — *more*, not less. A pure instruction did shrink, to
an operation between machine registers with a register add for the tick and a register compare for the stop. But every
call into the interpreter now carries the writeback of the two counters and of whatever registers changed since the
last call, and a reload after it, and real code is a load or a store every few instructions; the JIT also gives a
method with a dozen long-lived locals a frame it did not need before. The ring (§9) priced the result the way it prices
any code growth: an entry with 500 blocks live went from 43 to 68 ns, with 5,000 from 100 to 153, and a
thirty-two-instruction block with 5,000 live from 345 to 793. By §9's model alone this change should have lost.

**Measured.** Interleaved on a quiet machine, three rounds of 600 frames against the commit before: 39.7 → 38.1 fps
in Ocarina of Time, 48.8 → 46.4 in Super Mario 64, 45.4 → 44.3 in Wave Race — a loss of 4.0, 4.9 and 2.4 per cent, in
every round of every game. A first run taken while another program held seven of the machine's cores showed the same
direction at a fifth less speed on both sides, and was discarded as a measurement though not as a hint. Exactness held
throughout: the probe's whole state on every frame of the three games, the Mars suite, the block tests, all with the
verifier on.

**Not kept.** The ring's price of the bytes was the outcome, as the paragraph above said before the run. What the
change established is that the writeback a call-out forces is not a small tax: real code calls into the interpreter
every few instructions, and a scheme that keeps registers in machine registers pays for each call in bytes it cannot
amortise. This change kept the tick and the stop compare per instruction, in locals; §9's description of density also
batched them over a run of pure instructions, which this change did not do because the stop must fall after exactly
the instruction it falls after — a batched run needs a guard that the whole run fits under the stop and a second,
per-instruction copy for when it does not, which the ring prices as more bytes again. That shape is unmeasured (§12.3).
The patch and its measurements are with the speed tooling outside the repository.

## 12. Chaining: a block hands on to the next — measured, and not kept

*2026-09-19, the second of §9's levers, built third.* §9's census found six entries in ten following the ending
branch's static target and a further quarter falling through to the next instruction; §9's ring found an entry costing
about fifty nanoseconds before the target's code was touched, and §10 found that moving entries about did not pay.
This change leaves the entries where they are and takes the dispatcher out of most of them.

**The mechanism.** A block's method now takes the block itself beside the processor, and returns a block or nothing.
Its epilogue, after the step's tail (§3.2), calls `Cpu.Chain` with the block that is ending. `Chain` does what the
dispatcher does at an entry, in the dispatcher's order, and returns the block that may run next — or null, in which case
the method returns null and the dispatcher is back where it was. The dispatcher runs whatever is returned in a loop
inside the same `try` the first block ran in. No block calls another: the stack is one frame deep whatever the chain's
length, the fault handler is the dispatcher's whatever block faults, and the JIT is not asked for a tail call it may
turn into a helper.

`Chain` refuses, and hands the decision back, when:

- a branch is pending — the block stopped between a branch and its slot, and §3.1's rule stands;
- the VI's field is not the one the frame began in, or the cycle count has reached the frame's cap — the events the
  epilogue just ran may have ended the frame, and the frame loop must see it;
- the interrupt check's inputs have changed — `_recheck` is set or the MI line differs from what the check last saw —
  so that the dispatcher takes the interrupt exactly as it does today, on the same instruction;
- the mode is not kernel, the address is outside the direct segments or misaligned, or beyond RDRAM — §3.1's conditions,
  unchanged;
- the target has no block yet, or its block is not compiled — so that the dispatcher counts its runs and it becomes hot
  as any block does;
- the target's words are not the ones it was compiled from — §2.3's comparison, which chaining does not spare.

Otherwise it does the entry's own work — the current address, the delay-slot flag lowered, the counters, the Debug
build's mode check — and returns the target.

**The remembered target.** Each block keeps the compiled block it last handed on to, and trusts it only while its
address is the one asked for. A jump through a register whose target varies looks up the table again — the cost the
dispatcher paid, no more. A remembered block whose comparison fails is forgotten, so that the block the dispatcher
places in its stead is found at the next hand-over rather than the stale one retried for ever; a remembered block whose
comparison succeeds after the dispatcher has replaced it is run, which is correct — its code is the code its words say
— and merely leaves the newer block cold. A target that is not compiled is not remembered.

**What it spares, and what it does not.** Spared: the table lookup (two dependent loads in a structure that does not
stay in the first-level cache), the dispatcher's own checks and stores, its return and its call. Not spared: the
comparison of the target's words, the delegate's invocation (the dispatcher's loop still calls through it), and the
target's code brought in cold, which §9 named as the larger part of an entry's cost. §9's estimate of what chaining
could buy was made on that account and stands: this is a change to the smaller part.

**The argument for exactness** is §3.2's. Every field the interpreter's next step could read is set by the epilogue
before `Chain` runs, and `Chain` sets exactly what the dispatcher's entry sets, under exactly the conditions the
dispatcher would have entered; where any condition differs it does nothing and the dispatcher decides. A fault in the
third block of a chain is caught by the frame that caught faults before. `RunBlocks(n)`'s exactness (§6) holds because
no block is handed on to at or past the cap.

**Tests**, each written before the round of §12.1 and each a differential comparison of the whole state:

- two compiled blocks jumping to each other, with the counter of hand-overs proving the mechanism ran, not merely the
  result;
- a two-line field, so that frames end at hand-overs, saved and compared frame by frame as a whole state;
- one block rewriting the first word of the block it jumps to: the hand-over compares and refuses, the dispatcher
  discards, and the sum the rewritten instruction accumulates is the interpreter's;
- the timer's interrupt and the VI's and AI's events landing at hand-overs, over a hundred thousand and three and a
  half million instructions;
- a jump to a misaligned address inside a compiled block's words, which faults in the interpreter and which a hand-over
  that did not check would have run from the word below;
- a fault on the first instruction of a block reached by a hand-over, whose address only the hand-over can have set.

### 12.1 The mutation round

Fifteen mutants of `Chain`, of the epilogue's call to it and of the dispatcher's loop, each also run with the verifier
off: twenty-eight runs against every Mars test, fifteen caught. Caught: the frame's end ignored (four tests, two of them
§12's); the cap ignored (seven); the interrupt check's flag ignored (eight, all by the verifier; three with it off, the
two timer tests among them); the MI line ignored (the RSP break test and Wave Race's picture, both by the verifier);
the comparison removed (the rewriting test alone, which is what it was written for); the current address not set (the
first-instruction fault test alone, likewise); the delay-slot flag left up (eight); and the epilogue's call replaced by
a null (the five tests that asserted the counter, which is what a counter in a test is for).

Escaped, and each is recorded as what it is:

- *The pending-branch rule.* Equivalent in every test and not in general: it matters only when a block starts at a
  delay slot's address — a branch whose target is another branch's slot — and the stop falls between that branch and
  its slot. No test had that shape. The test that would — a branch to its own slot under the VI's and AI's events,
  so that a block starts at a slot's address — was written afterwards (§12.4); it catches the dispatcher's own
  pending-branch mutant beside five older tests.
- *The mode, the segment, the alignment and the memory range.* Masked, in every test written, by the recheck flag:
  the `ERET` that changes the mode or returns to the misaligned address is a COP0 write, the flag is up, and the
  hand-over goes back to the dispatcher before those checks are reached. A hot `jr` from a compiled block to a
  misaligned address, to one beyond memory (where the mutant indexes past the table and throws), or into a segment the
  TLB maps would have needed three further tests. §12.4 says which of the three the dispatcher itself turned out to
  need.
- *The remembered target trusted at another address.* Masked by the comparison, which is made at the asked-for
  address: a stale block's words differ from the words there unless they are identical, and identical words run
  identically except for the store check's compile-time bounds. Equivalent for every test and nearly so in general.
- *The dispatcher's loop removed.* Equivalent by construction — the hand-over's entry work is redone by the
  dispatcher, and the counter counts a hand-over that did not run — and the loop's absence is a slower dispatcher, not
  a different machine.

### 12.2 Measured

**How much of the dispatch it removed.** Over 600 frames, entries handed on without a dispatch: 18.1 of 18.9 million
in Ocarina of Time (95.5 per cent), 17.7 of 18.7 in Super Mario 64 (94.5), 17.1 of 18.4 in Wave Race (93.0). More than
§9's static-target column, because the remembered target looks up again when its address differs, so most of the
"elsewhere" column joins. Instructions an entry unchanged: 58.8, 58.6, 48.3.

**What an entry then cost.** §9's ring, base against chain, interleaved on a quiet machine (the base rows reproduce
§9's table):

| blocks live | instructions a block | ns an entry, dispatched | ns an entry, handed on |
| --- | --- | --- | --- |
| 500 | 8 | 43, 44 | 47, 49 |
| 5,000 | 8 | 99, 100 | 99, 98 |
| 5,000 | 32 | 380, 372 | 362, 373 |

Nothing at 5,000 blocks, where the entries the games make live; four nanoseconds *dearer* at 500, where the table is
warm and the lookup it spares cost less than the call that spares it. §9's sentence stands verbatim: the cost is not the
dispatcher's instructions and not the validation, it is memory.

**In play.** Interleaved, three rounds of 600 frames: 39.7 → 39.0 fps in Ocarina of Time (−1.8 per cent, each of the
three rounds below each of the base's), 48.8 → 48.8 in Super Mario 64, 45.4 → 45.6 in Wave Race. Nothing, with a small
real cost in the game with the most entries. Exactness held: the probe's whole state identical on every frame of all
three games with the verifier on, the Mars suite at 3,389.

**Not kept.** The seven tests are, as §10 kept two: they are the suite's only multi-block programs — two compiled
blocks reaching each other, frames ending between them, one block rewriting another, the timer and the devices'
events landing between them, a misaligned jump target, a fault on a block's first instruction — and each passes
against the dispatcher as it was, which is what they now check. The patch is with the speed tooling.

### 12.3 What the three levers established

§9 priced them before any was built and said where the time was: the target's code brought in cold, and the
comparison against it. §10 moved a quarter of the entries into their predecessors and gained nothing; §11 changed the
bytes and lost; §12 took the dispatcher out of nineteen entries in twenty and gained nothing. The recompiler's
remaining cost — five to eight per cent of a frame in its own code and nine in the entries (§9) — is memory traffic in
proportion to the bytes emitted per instruction, eighty to a hundred, and a lever that does not lower that number does
not move it.

What would lower it is emitting less per instruction. The tick, the RSP's halted test and the stop compare are most of
an instruction's bytes, and they are per instruction because an event must run after exactly the instruction it is due
after. A shape that hoists them over a run of pure instructions behind one guard — the whole run fits under the stop
and the RSP is halted — and keeps the per-instruction copy for when the guard fails is the one idea in this page's line
that is not measured, and the ring's rows say what it must beat: its own second copy. Beyond the processor, the RSP is
29 per cent of Wave Race's frame and the VI 30 per cent of Ocarina of Time's (§9), both larger than anything left here,
and both other pages'.

### 12.4 The dispatcher's address checks, tested afterwards

The four checks §12.1 found masked in the hand-over are the dispatcher's own (§3.1), so the question of whether the
suite tests them stood after the change was reverted. Three mutants of the dispatcher's entry condition — the
alignment test removed, the segment test removed, the memory-range test removed — were added to the standing round and
run against the suite as it was:

- *Misaligned*: caught by §12's misaligned-target test alone. Without the check the dispatcher indexes the table by the
  address's word, finds the compiled block below it, fails its comparison two bytes out, shapes a block from misaligned
  words in its place and, sixty-four faults later, compiles and runs them.
- *Beyond memory*: caught by thirty-one tests, because the boot runs from the signal processor's memory, whose physical
  addresses lie beyond RDRAM, and the mutant indexes past the page table on the first instruction.
- *Outside the direct segments*: **escaped every test in the suite.** A jump into kuseg or kseg2 from RDRAM code was a
  shape no test had. Its test now: a compiled loop ending in a `jr` to `0x2010`, which takes the TLB refill vector —
  the program's own first word, where the count restarts and the loop runs again — while the dispatcher without the
  check would have looked the address up by its low bits and run RDRAM as code. A first draft jumped to `0x10` and
  never restarted: a zeroed TLB has entries whose page is zero, so an address in the first eight kilobytes of kuseg hits
  an entry that is present and invalid, takes the general vector rather than the refill's, and an `ERET` there returns
  to the same fault. Recorded because the difference between "no entry" and "an invalid entry" is the kind a fetch
  test needs to know before it is written.

With the two tests the round catches all three, and the pending-branch mutant with six tests where it had five. The
standing round has thirty-six sites.

## 13. The first gameplay states, and a defect the probe could not reach

*2026-09-19.* Every exactness claim above rests on the golden probe — boot, title screens and an attract race — and on
programs written for the tests. The first states saved in play were three from Mistress: Super Mario 64 on the castle
grounds, Ocarina of Time in the opening after a new file, Wave Race 64 on the watercraft screen before warm-up. A
harness runs the interpreter and the blocks in lockstep from a saved state and compares the whole state every frame,
and a second one steps the interpreter to the first fault and walks libultra's thread list; both are with the tooling
outside the repository.

**What diverged.** From the Ocarina of Time state the two machines were identical for 3,733 frames and differed on the
3,734th: the same instruction count and cycle, a different program counter, a different state. Played on with blocks,
the opening's "Navi…" message stopped after its first word and the game drew the same frame for ever; played on with
the interpreter, the message ran to its end. The cause was §2.3's: a shape kept from words an overlay had since
replaced. With the dispatcher shaping again at the threshold the two machines were identical for all 4,300 frames, and
the counter showed eighteen entries reshaped in that stretch of the game alone. The probe's three games, which never
reload code under a cold block, were identical before and after.

**Why a state found it and a probe did not.** A run from a saved state starts with no blocks, and a run that has played
there carries every entry shaped along the way — so the two ran different compiled code from the same machine, and the
first sign of the defect was that a checkpoint saved mid-run and loaded again did not replay what the run itself did,
though a save and load are exact (checked: a machine reloaded at frame 150 matched the straight run for 250 frames).
That asymmetry is worth keeping as a test of its own: any divergence between a run with a history and one without is
the block cache's.

**What it does not explain.** ~~The interpreter, and now the blocks with it, draw the whole of that message over a flat
grey screen where the Great Deku Tree should stand. That is not the recompiler's — both machines draw it — and it is
open.~~ *Closed the same day: the grey is the game's, not a defect.* The frame's commands, replayed from the RAM at its
start through angrylion and through Mars, drew the same 76,800 pixels; cut before its last three fill rectangles, the
same commands show the Deku Tree and Navi. The last rectangle is the scene transition's fade, grey at alpha 255, and
the transition is held in its wait mode until the cutscene sets a control byte in the save context — which its
transition command does, after the message box. Pressed through, the message closes and the tree fades in. The one
difference the replay did find is its own item: cut before those fills, 1,168 of the 76,800 pixels differ between Mars
and angrylion, hidden in that frame by the fade and not yet located. The earlier failures from the same games (a faulted graph thread in Ocarina of Time, a display list run past its
buffer in Wave Race) were in states saved by a frontend that raced the emulation thread (`EmuSen_Settings_Reference.md`
§4.21a) and cannot be reasoned about from those states.

## 14. A census of what blocks hand the interpreter, and the six branches compiled inline

*2026-09-20.* §7's attributions, and §9 to §12's, came from the runtime's sample profiler, which
`Mars_Performance.md` §36 shows cannot attribute time on the emulation thread: it stops a thread at its next safepoint
and charges the sample there. Their interleaved timings stand; the picture of *where* the interpreted share went does
not. So the question was asked again with a counter instead of a sampler: a count, by opcode, of every word that
reaches the interpreter's switch, kept when `EMUSEN_MARS_CENSUS` is set and folded away by the compiler otherwise,
printed by the play harness.

**One opcode.** From the gameplay states, four rasteriser workers, 300 frames:

| of the words reaching the switch | Wave Race 64 | Ocarina of Time |
| --- | --- | --- |
| BEQ | 86.9% | 81.5% |
| LW | 2.4% | 3.4% |
| LWC1 | 1.6% | 2.1% |
| BNE | 1.2% | 0.9% |
| the floating-point multiply | 0.9% | 1.3% |
| everything else | under 0.6% each | under 0.9% each |
| words a frame | 744,000 | 944,000 |

That is not a distribution of a program's branches; it is the idle loop. Both games spend most of their processor's
time in libultra's idle thread, a branch to itself with a nothing in its slot, which §3.4 keeps inside its block but
§3.3 still hands to the interpreter for the compare: a call, the switch, and `BranchIf`, seven hundred thousand
times a frame. Every load, every store and every floating-point operation put together is a tenth of that. The
recompiler's interpreted share, measured honestly, is one instruction.

**What is compiled now.** The six plain conditional branches — BEQ, BNE, BLEZ, BGTZ, BLTZ and BGEZ, not the likely
forms and not the linking ones — are emitted inline: the two registers loaded and compared, or one against zero
signed, the target stored into `NextPc` when taken, and the pending flag set either way. That is what the interpreter's
`BranchIf` does for them and nothing else: the target is a constant of the block, `Pc` plus the offset, and the
untaken path leaves the `NextPc` the block already stored before the branch. Everything around the branch — the
addresses stored before it, the tick, the slot's handling, the exit if the slot was nullified — is unchanged, and the
likely forms and the links still go through the interpreter, since they nullify a slot or write a register and an
observer.

**How it is known to be exact.** `The_inlined_branches_leave_the_machine_the_interpreter_leaves_taken_and_not` runs
all six over a counter that changes sign, each taken and not taken a hundred times, into an idle loop, against the
interpreter, whole state compared; with BEQ's condition flipped in the emitter it fails at once. The interpreter and
the blocks run in lockstep from the four gameplay states (`h-play`, 200 to 400 frames each, whole state compared every
frame) and stay identical; the probe's 1,800 frames are identical to the baseline with the verifier on. The
measurement is `Mars_Performance.md` §36.2.

## 15. The idle loop, run without its block

*2026-09-20.* `Mars_Performance.md` §38's census found one block of two instructions, an unconditional branch to
itself over a no-operation, to be 82 to 93 per cent of every instruction executed: the operating system's idle
thread. `BlockShape` now recognises it when a block is shaped — `b` to itself as `0x1000FFFF`, or a `j` whose target
is its own address, with a zero word in the slot — and records the cycles of the branch and of the whole turn on the
block. When the dispatcher reaches such a block with compiled code (so the words have just been compared with the
image, as for any block), `RunIdle` runs it instead of the code.

**The argument for exactness.** The loop changes nothing but the time: each instruction adds its cycles to the bus
and one to the instruction count, steps the signal processor if it is running, and leaves when the bus reports a
write or the interrupt line differs from what the CPU last saw, or when the cycles reach the block's stop, the
least of the next event, the timer and the caller's cap. `RunIdle` does exactly those things in that order, one
instruction at a time, and differs in two places only. While the signal processor is halted nothing can happen
before the stop, so it adds whole turns at once — all but the last, which it runs singly, so the exit falls on the
instruction it would have. And while the processor runs and a cycle is a step and an instruction (both instructions
cost one), it lets the processor run compiled blocks (`Mars_Rsp.md` §11) and adds as many cycles and instructions
as steps were run; the parity of that count says which of the two instructions was the last. On leaving it writes
what the block's own exits write: after the branch, the counters past it with the branch pending into the slot;
after the slot, the counters at the loop's head and the slot flag set; then `AfterInstruction`, as the epilogue does.

**The proof.** `MarsIdleTests` runs a machine whose CPU idles in RDRAM while the signal processor runs a counted
loop of scalar, vector, store and control-register work that raises its interrupt half way, with the CPU's
interrupts on and a handler that stores the count register it was entered at; with a leading no-operation the whole
program lands on the other instruction of the pair. The state after seven frames must equal the state with both
paths off, and the test refuses to pass unless idle turns were passed and more than ten thousand steps ran in
blocks. Four mutants were tried against it: the exit after the branch leaving no branch pending, the parity of a
block run ignored, a control-register move not ending a block, and idle turns passed up to the stop itself. The
first version of the test caught one of the four; it caught all four only once the interrupt came from a block
that had run thousands of times (a status write that runs once is never compiled), once the handler recorded when
it was entered (a handler of no-operations ends in the same state two cycles later), and once the padding moved
every step and not only the break. The probe's 4,800 frames are identical with both paths on.

`EMUSEN_MARS_NOIDLESKIP=1` turns it off, for measurement. What it does not cover: an idle loop of another shape —
a wait on a memory word, a counted delay — is a block like any other.

## 16. The loads compiled inline

*2026-09-20.* §14's census, taken again from Ocarina of Time's field with the idle loop set aside
(`Mars_Performance.md` §38), put 52 per cent of the busy instructions through the interpreter's switch. A third of
those are integer loads, the load of a word alone 21 per cent; the loads into the coprocessor are 14 per cent and
the word moves to and from it six; its arithmetic is 17; the rest are the register jumps and links, the likely
branches and the coprocessor's branch.

**What is compiled.** The seven aligned integer loads into a register other than zero, the two loads into the
coprocessor, and the word moved to it and from it. Each is the interpreter's own fast case and nothing more, tested
in the interpreter's order: the address is the base plus the offset; it is a direct kernel address (`address −
0xFFFFFFFF80000000` below `0x40000000`, unsigned, and a block only ever runs in kernel mode, where the byte order is
never reversed); it is aligned; its physical address is inside RDRAM, whose length the block is compiled for; and
the page is not one the display processor is writing (`_dpWriteMarks`, the same word the interpreter tests). Then
the bytes are read big-endian straight from the array and extended as the instruction says. For the coprocessor the
status register's usable bit is tested first, as `RequireCop1` tests it, and the value goes through the
interpreter's own `WriteFpuWord`, `WriteFpuWide` and `ReadFpuWord`, so the half and full register modes are theirs.
**Any test that fails goes to the call the block made before**, `Execute` with the word, which raises what it
raises, waits for what it waits for and reaches the devices. Nothing the fast case skips has a side effect.

**What it bought.** One build, the loads inlined or called (`EMUSEN_MARS_NOINLINELOADS=1`), four rounds of 1,200
frames interleaved, the scan-out off, medians: Ocarina of Time's mean frame 9.10 to 8.61 milliseconds and its
ninetieth percentile, the drawing frames, **20.87 to 19.88, under the slot of twenty for the first time**; Super
Mario 64 5.12 to 4.93; Wave Race 7.49 to 7.36. Five per cent, where a third of the switch's traffic was removed:
the interpreter's fast case was already short, and what was saved is the call, the switch and the decode. The state
hash after 900 frames is unchanged in all three games.

**The proof.** `The_inlined_loads_leave_the_machine_the_interpreter_leaves` runs every one of them in a loop beside
the interpreter — bytes with their top bit set, both signs of offset, a load into register zero, a load from a
device, which takes the slow call — refuses to pass unless the loop compiled, and in its second case takes the
coprocessor away half way, so a block that has run a hundred times finds it unusable. 
`An_inlined_load_that_is_not_aligned_faults_as_the_interpreter_faults` moves the address by one at the 128th turn.
Four mutants of the emitter were tried and all four caught: the signed half word read unsigned, alignment not
tested, the usable bit not tested, the word from the coprocessor not sign-extended. The first form of both tests
passed without compiling anything — the fault came on the first turn, before any block existed — which is why they
now assert that one did.

**What is not compiled, and why.** Stores already call their handler directly and return where they landed, for
the block's check of its own words. The coprocessor's arithmetic is software floating point, exact by construction;
a host-float fast case would have to prove the inexact flag bit for bit and is its own piece of work. The register
jump and the link call the debugger's call-stack observers. The likely branches annul their slot. Each is a few per
cent of the switch. After this the largest single item on the CPU's side is the dispatcher's entry, which §9 and
§12 measured as memory and declined to chase, and the largest on the thread is the signal processor's vector unit
(`Mars_Performance.md` §38).

## 17. Blocks behind the TLB

*2026-09-20.* *GoldenEye* ran at a third of full speed, thirty to fifty-five milliseconds a frame, with every one of
its 1.87 million instructions a frame counted as busy and none passed as idle. The pacing harness run from boot
(`pacebench` with `-` for the state) put 0.2 per cent of its instructions in blocks, and the program counter at
every frame's end in `0x70xxxxxx` or `0x7Fxxxxxx`: the game runs from mapped memory, and §2's dispatcher took a
block only from a direct kernel address, so the whole game went through the interpreter, its idle loop included.

**What changed.** The dispatcher translates a mapped address itself: the segment must be a mapped one in 32-bit
addressing, the TLB must hold a valid entry for it, and the page's frame is remembered for the next fetch from the
same page. Anything else — no entry, an invalid half, 64-bit addressing, a user or supervisor mode — is the
interpreter's step, which raises the refill or the address error exactly as before; a block never raises a fetch
fault. The block is found and proved by its physical address as ever (§2.3), and its code names no address of its
own — the entry is read from the program counter, branches are relative, and a jump's target is formed by the
interpreter from the counter — so one compiled block serves every address its frame is mapped at.

**The page is the limit.** Inside a page, virtual and physical addresses run together; past its end the next
virtual word may be in any frame. A block shaped from a mapped address is cut at the end of its four-kilobyte page
(the smallest, so safe for every page size), and a block already in the cache that crosses its page, having been
shaped from a direct address, is left to the interpreter when reached through the TLB.

**The remembered page is forgotten** whenever coprocessor 0 is written (`Cop0Written`, which a state's load also
calls) and whenever a TLB entry is written. The second looked redundant: GoldenEye's state hash after 1,620 frames
was the same with it removed, because the game loads its entry registers immediately before each write and those
loads already forget. It is kept for the handler that writes an entry from registers loaded earlier, and
`A_page_remapped_by_a_tlb_write_alone_is_fetched_from_its_new_frame` is that case. The test's first form passed
with the forgetting removed: a stale frame only matters once the block at it is *compiled*, since a cold block is
run by the interpreter, which translates for itself; the loop in the first frame now runs two hundred turns.

**The idle loop anywhere.** §15 recognised the loop by a branch to itself or a jump to its own physical address.
The jump names an address and is only trusted at a direct one; the branch names none, and is now recognised
wherever it is mapped, which is where GoldenEye's is (`0x70000710`).

**Measured**, 1,500 frames from boot with Start pressed three times, one build with the mapped blocks off and on
(`EMUSEN_MARS_NOMAPPEDBLOCKS=1`): the mean frame 36.8 to 13.1 milliseconds, the median 36.8 to 9.2, 99.3 per cent of
instructions in blocks and 87 per cent of all instructions passed as idle turns. The state hash is identical in
both modes. `A_mapped_loop_across_two_pages_leaves_the_machine_the_interpreter_leaves` runs a loop across two pages
whose frames are not neighbours, with a decoy after the first frame that adds to another register; a block not held
to its page runs the decoy, and that mutant and one that drops the page offset are both caught. The probe's two
games, neither of which maps its code, are identical to their baselines. Seen: the game reaches its mission
select, the Dam's briefing and the level's opening.

**What it does not cover.** Loads and stores through the TLB still call the interpreter (§16 inlines only direct
addresses), and a mapped block that would cross its page is interpreted rather than split.

## 18. A round on the compiler: what the census named, what each was worth, and two things bounded before they were built

*2026-09-20.* With the loads inline (§16) and mapped code in blocks (§17), the census was taken again, on Ocarina
of Time in play and on GoldenEye into its first level. A quarter of busy instructions still reach the
interpreter's switch in both: 26 and 27 per cent. Of that traffic the jumps and links are 18 per cent, the likely
branches 12 to 17, the coprocessor's arithmetic 27 in Ocarina of Time and 12 in GoldenEye, its conversions and
branch another 10 or so; the rest are instructions the interpreter runs for blocks still cold.

**The jumps, the links and the likely branches compiled.** A jump leaves its target in the next counter — the
slot's address with its low twenty-eight bits replaced — and a branch pending, as `Branch` does; a link writes the
address after the slot first; a register jump reads its target before the link is written, since they may be one
register. While a debugger's call stack is listening, which is the only thing `JumpAndLink` and `JumpRegister` do
besides that, the interpreter's call is kept: the link forms test `CallObserver` and a jump through `ra` tests
`ReturnObserver`, at run time. A likely branch compares as its plain twin (the opcode less its likely bit, the
register-immediate pair two on) and, not taken, annuls its slot as `NullifyDelaySlot` does — the counters past the
slot and nothing pending — which sends the block out by the exit it already had. The state hash after 1,200 frames
is unchanged in three games. Three mutants: a link one word short and a likely branch that runs its slot were
caught by tests already there; a register jump reading its target after its link was written was caught by
nothing, and `A_register_call_whose_link_is_its_own_register_leaves_the_machine_the_interpreter_leaves` is the test
written for it. **What it was worth was inside the noise**, and the arithmetic says why: nine per cent of busy
instructions made about five nanoseconds cheaper is a third of a millisecond in a drawing frame of twenty.

**Single-precision arithmetic on the host** (`Mars_FpuMath.md` §11) is the change that showed: with both changes on
against both off, four rounds interleaved, Ocarina of Time's mean frame 9.25 to 9.00 milliseconds and its drawing
frames 21.49 to 20.84; Super Mario 64 one per cent.

**Bounded first and not built: batching the counters.** Every compiled instruction adds its cycles to the bus and
one to the instruction count in memory, and tests the stop. The pending counts are compile-time constants along a
straight line, so they could be kept out of memory and flushed before anything that can observe them; the design
is long and every exit of a block takes part in it. A prototype that moves both counters once at a block's head —
inexact, and so an upper bound — was switched on in the same build: **1.7 per cent of Ocarina of Time's mean frame
and 0.6 of Super Mario 64's.** Not built.

**Where a busy instruction's time is, after all this.** Blocks average 6.9 instructions in Ocarina of Time and 5.0
in GoldenEye, so a drawing frame enters a block a hundred thousand times. By an interrupt sample the dispatcher's
entry is 8.5 per cent of the thread, the largest single item on the CPU's side and about what all the block bodies
together cost; §9 and §12 measured extension, density and chaining against exactly this and kept none, finding the
cost to be memory and not dispatch, and nothing here contradicts them. Software floating point was the other 8.5,
and is what §11 of `Mars_FpuMath.md` went after. What this round says about method is what the day's other rounds
said: a census counts instructions and a sample counts time, and only a bound measured in the games says what a
change is worth before it is built.

## 19. The dispatcher again: the bytes read rather than reasoned about

*2026-09-20.* §9 to §12 established that an entry's cost is memory — the emitted code streamed in cold, eighty to
a hundred bytes an emulated instruction — and that a lever which does not lower that number does not move it. Two
levers since had tried to lower it and raised it instead (§11), and one prototype had bounded the counters' share
at under two per cent (§18). What none of them had done is look at the bytes.

**The size now.** From the runtime's perf map (`runs/blockbytes.sh`, Ocarina of Time in play, 600 frames): 6,224
blocks, 11.9 megabytes of machine code, a mean of 1,911 bytes a block and a median of 1,332, against blocks that
average seven instructions an entry. No cache near the processor holds that.

**First guess, and wrong.** Every register access in a block is an array access with a bounds check, three to an
arithmetic instruction, some thirteen bytes each. Blocks now take the registers, RDRAM and the write marks by a
reference to their first element, made once in the prologue, and address them without a check — a register's index
is five bits of the instruction, and a physical address has just been compared with RDRAM's length where it is
used. The code shrank by three per cent, 1,940 to 1,876 bytes a block, and the time by nothing measurable. It is
kept because it is smaller and no less exact (`EMUSEN_MARS_CHECKEDBLOCKS=1` restores the checks), but the bounds
checks were never the bulk.

**What the disassembly showed** (`DOTNET_JitDisasm=mars_block_…`, which the release runtime honours). Of an
instruction's hundred and forty-five bytes, about ninety were one thing: the compiler had *inlined `RspRan` after
every instruction* — the count, the single-step test, the call of the processor's step, the compare of the bus's
write count and the compare of the interrupt line — a path taken only while the signal processor runs beside busy
code, a fifth of its steps (`Mars_Rsp.md` §12), and jumped over on every other instruction. The instruction itself
with its tick was about fifty. `RspRan` had been small enough to inline since it was written, and had grown under
§36.1 and §15's work without anyone looking at where it went.

**The change is an attribute**: `RspRan` is not inlined, and the block carries a call. Blocks fell from 1,876 to
1,331 bytes in the mean and 1,307 to 943 in the median, 12.3 megabytes to 9.0. Interleaved against the commit
before, four rounds of 1,200 frames, the scan-out off: Ocarina of Time's mean frame 8.37 to 8.21 milliseconds and
Super Mario 64's 4.81 to 4.72, two per cent each; GoldenEye, whose blocks are shortest at five instructions an
entry, a median frame of 7.25 to 6.63 and a ninetieth percentile of 19.34 to 16.62. The state hash is the same in
both builds in all three, the Mars suite passes and the probe's two games are identical.

**Tried after it and not kept:** the call sites moved to the end of the method beside the exit stubs, so that the
path with the processor halted would run straight on. Neither the bytes nor the time moved, and the emitter is
simpler without it.

**What an instruction is now**, read from the same block: the operation some seventeen bytes, the two counters
fourteen, the halted test seven, the call's site twenty-six, the stop compare seventeen — about eighty, of which
the operation is a fifth. §12.3's unmeasured shape, the tick and the tests hoisted over a run behind one guard,
is still the only thing that would change that, and §18's prototype bounded the counters' part of it at under two
per cent; the tests' part is not bounded.

**The method, recorded because it would have saved a day.** §9's model was right and its number was measured, and
for a day the bytes were reasoned about from the emitter's source. The JIT's listing for one block answered in a
minute what the source could not: the largest thing in a block was not anything the emitter emits.

