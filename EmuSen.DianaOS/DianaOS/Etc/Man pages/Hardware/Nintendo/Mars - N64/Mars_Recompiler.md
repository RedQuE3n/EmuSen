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
  compared as whole states with `UseBlocks` on and off.

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
  dispatcher. Each is a known lever and each was left until this one is measured in play.
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
- **3D gameplay is still unmeasured.** Every number here comes from boots, title screens and an attract race, as every
  number in `Mars_Performance.md` does; a scene with more code hot at once compiles more blocks, and the queue's latency
  in such a scene is a number nobody has.

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
