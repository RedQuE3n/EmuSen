# Sapphire: the redesign proposal (2026-10-28)

*A design proposal; nothing in it is built. Decided 2026-10-28 by the tester: the redesign replaces the lever list of
`Sapphire_Speed.md` §58.7, it is reviewed by a critic, and the tester approves it before any stage starts. Its sections
are this page's own (§1 to §10); a section number with a page named is that page's, and one with no page named
(§44, §57.4) is the build record's numbering shared by `Sapphire_Native.md`, `Sapphire_Recompiler.md`,
`Sapphire_Scheduler.md` and `Sapphire_Speed.md`. Its stages are R1 to R10, its predictions P-R1 to P-R12, and the
compromises keep the numbers of `Sapphire_Plan.md` §5.13 (C1 to C11), the new ones C12 and C13.*

*Sapphire's goals, in the tester's words of the same day: play more games than the other PlayStation 2 emulators, follow
the project's Unix philosophy, and be "bit accurate rather than cycle", with compromises where they have to be; the first
scoreboard is the PlayStation 2 games on the handheld, playing correctly at full speed there; no fixes that only work for
one game; it works out of the box, with additional options.*

*Sources, and how far each is trusted: the measurements are Sapphire's own (§55 to §58, and the stages they cite). The
sealed review (a report written on 2026-10-28 by a reader of other PlayStation 2 emulators' sources, sealed so that its
reader alone saw them) and the sealed review's audit (a second reading of the same sources, which corrects it) are theory
about how others work, not measurements of Sapphire, and their counts of per-game entries are rough; they are used for
hints and for the classes of games that break, never for code. `EmuSen_FastExact.md` (branch `sim-reference`) is a
literature reference whose citations were written without access to the sources and are unverified; its ideas are used
and its references are not cited here as checked. No emulator's source was read for this proposal, and the
clean-room rules of `Sapphire_Plan.md` §1.4 apply to every stage below.*

## 1. In one paragraph

§58 measured the design's own cost: with the board's background work batched and every observation kept exact, Jak and
Daxter runs at 207, 302 and 488 ms a field on the desktop, and the handheld needs 16.68 ms, which is 12.36 ms of the
desktop's time at the handheld's factor of 1.35. The shortfall, 17 to 40 times on the handheld (§2.3), is spread over
every unit, and each unit's cost is overhead per instruction, per pair and per observation rather than work. **The
proposal is a board run at events, with the EE in long compiled blocks between them**: every unit is visited only at
its own events or where another unit observes it (§3.1); the board lags the EE only up to a horizon below which no
interrupt line can change and no value the EE reads can differ, so that batching stays exact by construction rather than
by a tuned N; the EE compiles every instruction class into chained blocks whose timing is folded at compile time where
it is static and summed at run time where it is not, with memory through a page table and the caches' models kept
(§3.2); VU1 runs whole stretches of a microprogram between observations, with VIF1's writes and PATH1's reads of its
memory interleaved inside its run (§3.3); the GIF and VIF run at their own events and the DMA and FIFOs on one timeline
(§3.4); the IOP runs at its own instruction times, at most a block behind (§3.5); and the GS draws on the device (§3.6).
**Every piece of that is exact against today's core**, and is verified by digests equal to today's; the compromises are
named separately (§3.7) and taken only where a measurement shows they are needed. **The weak point is the arena**: the
budget asks of it 0.93 ns an EE instruction and 0.90 ns a VU pair on the desktop, below anything the project has
measured (§2.4), so the proposal commits to the dock and the island on one emulation thread and leaves the arena to be
decided by R6's and R7's numbers (§7, §9). The first stage finds what stalls Jak at N ≥ 512 (§7, R1), since the exactness
claim of §3.1 stands or falls with it.

## 2. The problem in numbers

### 2.1 The target

A bench frame is one field, and every field's cycles must be run in real time whatever rate the game draws at (§58.2):
**16.68 ms a field on the handheld**, the Legion Go S, whose thread `Sapphire_Plan.md` §5.12 argues to be 1.3 to 1.4
times slower than the desktop's; this page takes 1.35, so **12.36 ms of the desktop's time**. The plan's target leaves a
fifth of the field spare (a mean of 13.4 ms on the handheld, no field over 16.7 ms at the 95th percentile), which is
**9.93 ms of the desktop's time**. The budgets below are drawn to the 13.4 ms target, with the 16.68 ms ceiling given
beside them. The factor 1.35 is itself unmeasured on Sapphire (§58.8; R2 measures it).

### 2.2 The floors (§58.4, synced N = 64, desktop, ms a field)

Grouped by the units of §3. The EE's float operations are counted with the VUs, since 99 % of `sapphire::float`'s calls
come from them (§56.7). The remainder is what §58.4's classes leave unattributed (on the island, the EE's compiled code
among it).

| ms a field, desktop | `jak-dock` (210.5) | `jak-island` (~300) | `jak-arena` (490.1) |
| --- | --- | --- | --- |
| EE: compiled code, interpreter, dispatcher and exits, timing model | 108.7 | 103.5 | 206.9 |
| VU0 and VU1 with the floats | 35.3 | 84.8 | 141.4 |
| GIF and VIF | 16.2 | 26.6 | 38.8 |
| GS on the emulation thread | 15.4 | 31.0 | 31.2 |
| Scheduler, board and idle skipping's own work | 21.2 | 36.0 | 51.0 |
| IOP, its units and the SPU2 | 6.7 | 7.9 | 8.9 |
| Remainder | 7.0 | 10.2 | 11.9 |
| GS workers, busy, each of four (another thread) | 90 to 98 | 34 to 46 | 35 to 48 |

The work behind those costs, a field, at N = 1 (§58.4): EE instructions executed 1,665, 1,930 and 3,986 thousand (and 4.1,
3.8 and 1.7 million passed by idle skipping, not run); VU pairs 571, 1,343 and 2,955 thousand (VU1 340, 1,220 and 2,534
thousand of them).

### 2.3 The budget

The handheld's ms a field each unit may spend, drawn to the 13.4 ms target and summing to it, against today's cost on the
handheld (the desktop's × 1.35) and the factor by which each must fall. The allocation is a design choice, made from the
work each unit has to do (§2.4), not derived; another allocation moves the factors between rows but not their product.

| Unit | Budget, handheld ms | `jak-dock`: today → factor | `jak-island` | `jak-arena` |
| --- | --- | --- | --- | --- |
| EE | **5.0** | 146.7 → **29×** | 139.7 → **28×** | 279.3 → **56×** |
| VU0 and VU1 with the floats | **3.6** | 47.7 → **13×** | 114.5 → **32×** | 190.9 → **53×** |
| GIF and VIF | **1.2** | 21.9 → **18×** | 35.9 → **30×** | 52.4 → **44×** |
| GS on the emulation thread | **1.2** | 20.8 → **17×** | 41.9 → **35×** | 42.1 → **35×** |
| Scheduler, board, idle skipping | **0.7** | 28.6 → **41×** | 48.6 → **69×** | 68.9 → **98×** |
| IOP, its units, SPU2 | **1.2** | 9.0 → **8×** | 10.7 → **9×** | 12.0 → **10×** |
| Remainder | **0.5** | 9.5 → **19×** | 13.8 → **28×** | 16.1 → **32×** |
| **The emulation thread** | **13.4** (ceiling 16.68) | 284.2 → **21×** (17× to the ceiling) | 405.0 → **30×** (24×) | 661.6 → **49×** (40×) |
| GS drawing, on the device | within the field, beside the thread (§3.6) | four workers × 90–98 ms of the desktop | 4 × 34–46 | 4 × 35–48 |

### 2.4 What the budget asks of each item of work

Dividing each budget by the work of §2.2 gives the cost an item may have on the desktop:

| Desktop, at the 13.4 ms target (at the 16.68 ms ceiling) | `jak-dock` | `jak-island` | `jak-arena` | Best measured in the project |
| --- | --- | --- | --- | --- |
| ns an EE instruction executed, everything in | 2.22 (2.77) | 1.92 (2.39) | **0.93 (1.16)** | Mars's compiled instruction 1.1–2.1 (`Mars_Recompiler.md` §7); Sapphire's 9.2 today in the code alone, 69 with its exits (§4) |
| ns a VU pair, both units, floats in | 4.67 (5.81) | 1.99 (2.47) | **0.90 (1.12)** | MarsRT's RSP about 3 a step (`Mars_Native.md`); Sapphire's 41 to 63 today, of which the work alone 16 to 32 (§3.3) |

**What this says before any design.** The dock fits with room on both rows and the island at costs near Mars's best; the
arena asks of both rows less than anything measured anywhere in the project, at once. Either per-item costs fall below
1 ns (possible for the EE on the reasoning of §3.2, not shown for the VU), or the arena's VU work leaves the emulation
thread, which §3.3 shows cannot be done exactly and deterministically on this board without a compromise. This proposal
therefore states the arena as unproven and makes it the decision of R7 (§7, §9 Q5), rather than hide it in an
allocation that only looks balanced.

## 3. The design, unit by unit

### 3.0 The exactness argument every unit rests on

`EmuSen_FastExact.md` §1 states the sufficient condition for a batched model to give the same results as a stepped one:
every interaction between units is one of an enumerated set of observation points; at each, the observed unit is first
brought to the observer's time; ties at equal times are broken by the same fixed rule in both models; and an output that
changes level (an interrupt line) takes effect at the time it changes, not the time it is next looked at. §58.6 measured
what happens when the last part is not kept: the synced build kept every observation exact and the geometry right, but
from N = 512 the game stalled, and its candidates are the interrupt lines, VU1's end and XGKICK as VIF1 sees them, and
idle skipping's waits (§58.6 (3)). The sealed review's audit says the same of the most compatible emulator it read: coarse
in throughput but tight at its couplings, taking interrupts within a few cycles, breaking the IOP out when an EE event is
near, and running VU0 to the exact cycle for interlocked COP2 moves. **So the design is not "batch to N"; it is "batch to
a horizon"**: the board may lag the EE only up to the earliest cycle at which any unit could change a line or a value the
EE reads, and every unit says how early that could be (§3.1). Exactness is then a property of the enumeration and the
horizon, and the digests of §6.1 test both.

"Exact" means identical to today's core, the stepped board with the documented issue model (§44, `Sapphire_Plan.md`
§5.2). It does not mean identical to the console in timing, which nobody has measured; today's model is the reference.

### 3.1 The scheduler: batched synchronisation with exact observations

**What changes.** `Board::run_to` after every EE instruction (or its replay at a block's exit, §44.5) is replaced by an
event loop. Each unit owns its next event as a time in EE cycles in a flat array of 20 to 30 entries with a cached
minimum, recomputed only when an entry changes (`EmuSen_FastExact.md` §7's "next event" form; the board's units already
report their next events since S11, `Sapphire_Plan.md` §5.1). The loop runs the earliest unit to its event, and only
that unit, in the fixed order of today's `run_to` for equal times. **The horizon** handed to the EE is the minimum of:

- every unit's next event (the PCRTC's lines and fields, the timers' targets, the DMAC's slices and ends, the SPU2's
  samples, the CDVD's sector times, the IOP's next instruction, §3.5);
- every running unit's **lookahead**, the earliest cycle at which it could raise an interrupt line or change a value the
  EE reads without the EE first observing it: for a running VU, the shortest path in pairs from its PC to a pair with the
  E, T or D bit or an XGKICK, which the compiler computes per entry point of a microprogram (§3.3); for a VIF, its next
  VIFcode with the interrupt bit or a MARK, read from its FIFO; for the GIF, the end of the packet in transfer (SIGNAL,
  FINISH and LABEL are only at packet boundaries); for the DMAC, its next slice.

**What the EE does with it.** A compiled block is entered only when its static cost from the entry fits below the horizon
(one compare, §3.2); otherwise it runs a checked form that compares at each instruction boundary, or the interpreter
steps. The EE's observations (§3.4's table) call a helper that runs the event loop to the access's exact cycle, which is
the block's start plus the instruction's offset plus what run-time costs have added (§3.2), then performs the access. If
the access moves the horizon earlier (a DMA kick, a VIF start), the helper says so and the block leaves after the
instruction.

**Why it reaches its budget.** Today's board costs 20 to 51 ms a field because it is stepped 1.05 to 3.76 million
times a field (§57.2's census), and most steps find nothing (§57.2: 81 to 85 % of VU1's calls run nothing; the GIF's and VIF's shares are nearly all checks, §3.4).
At events, the count is the events' count: on the order of 1,000 to 4,000 XGKICKs (§57.2), the VIFcodes, the DMAC's
slices, 262 lines and the IOP's runs. **P-R1**: the board and scheduler at R5 under 0.52 ms a field on the desktop (0.7 on
the handheld) on all three windows, on the reasoning that 50,000 to 100,000 events at 5 to 10 ns is 0.25 to 1 ms; the
census of R2 counts the events, and if they exceed 100,000 a field the row is refuted before R5 is built.

**What stays exact.** Every unit's results and the times at which it is observed and raises its lines, by §3.0. Ties
keep today's order.

**What does not, and is named.** The board's own bookkeeping (each unit's `synced` and `horizon` clocks, the SPU2's
per-call sample, §54.9 (3)) is state that SPH2 writes, and it will differ: the state digest is no longer an oracle across
the change, so §6.1 defines a canonical digest of what a program can read.

### 3.2 The EE: long compiled blocks, a cost per block, memory through a page table, self-modifying code

**Coverage.** Blocks compile every instruction class: the integer and branch instructions of S36, the loads and stores of
S37, **the MMI's 128-bit operations as Cranelift vector operations, COP1 and COP2's macro instructions** (the float
operations inline where §3.3's fast paths are proven, called where not), COP0, SYNC, the cache instructions. A block ends
only at a control transfer it cannot follow, an instruction that raises, the end of its page, or a line the instruction
cache does not hold (below). Today 29 to 46 % of executed instructions are compiled (§58.4) and every MMI, COP1 and COP2
instruction ends a block; **P-R2**: at R3 at least 95 % of executed instructions are compiled on the three windows.

**Slow paths return into the block.** A data cache miss, an uncached load's bus wait, an access to a register, a
watched page (below) or a COP2 interlock is a helper call from which the code continues, not an exit to the interpreter.
Exits remain only for exceptions, interrupts and a horizon moved earlier.

**Chaining.** A block's exits to known targets jump to the target's code once it exists; an indirect jump goes through a
small table of recent targets (`EmuSen_FastExact.md` §2). The dispatcher is visited when a target is not yet compiled. A
chained jump is preceded by the horizon compare of the target, so an event still interrupts a chain. `emusen-jit`'s
publication by one atomic pointer and its W^X batches stay; chaining needs a patch step in a W^X batch, which is an
optional capability of `emusen-jit` (§5) that MarsRT need not take.

**The timing: static where it is static, summed where it is not.** Today each block computes `Timing::plan` and `commit`
for every instruction at run time, which is most of a block's bytes (§48.15.6) and 13.8 to 26.8 ms a field (§58.4). The
issue model is a function of the instruction sequence and of the state at the block's entry: the previous instruction's
category and pipe (Table 1-3's partner rule), the ready cycles of registers still in flight (a load's latency, the
multipliers' and dividers' busy cycles, HI1 and LO1), and the bus's free cycle. So:

- **The static form, exact (the default proposed):** the compiler folds the whole block's issue cycles into constants for
  a *quiescent* entry, one at which every resource the block reads in its first instructions is already ready and the
  previous category has no partner effect on the first instruction; the entry tests that with a handful of compares on the
  few fields concerned (the compiler knows which). Each instruction's cycle is then the block's start plus a constant
  offset; Count, Random and the counters are written at the exit from the total (as §44.4 already does for Count).
- **Run-time terms, exact:** where timing depends on what happens inside the block (a data cache miss and its refill and
  write-back, an uncached load's wait on `mem_free`, a COP2 interlock waiting for VU0), the helper returns the cycles
  added and the block keeps them in one register added to every later offset. The sealed review's audit is explicit that
  in-block offsets hold only where timing is static and that misses and entry-dependent stalls must leave or be summed at
  run time; this is the second.
- **A non-quiescent entry** runs the block's dynamic form, today's emitter (§44.4), compiled lazily on first need.
- **C1, a cost per block (the compromise, §3.7):** only if R4 measures the quiescent fraction too low or the static form
  short of its budget. Under C1 the block's cost is a fixed sum and the model's pairing and latencies are not applied.
  **C1 must keep two things exact, by the audit's evidence:** the issue timing between a COP2 macro instruction and a
  later read of its flags (games read COP2 flags in macro mode a few cycles after the operation, and those reads depend on
  the EE's issue timing), so COP2 macro instructions and the instructions up to their flag reads keep the model; and the
  FMV path's behaviour, which §6.4 tests before any C1 step (the audit: other emulators' per-game cycle-rate entries are
  partly for FMV hangs and sound drift, cured in the IPU's FIFO and DMA handshakes, not decode delays).

**P-R3**: on the three windows at R4, at least 90 % of block entries are quiescent; the census of R2 counts it on today's
code (the entry state is in the issue model's fields), so the prediction is settled before R4 is built.

**Memory through a page table.** Today's translation cache of 4 KiB pages keyed by the TLB's contents, the mode, the ASID
and the configuration (§48.3) becomes a direct table indexed by the virtual page for the current mode and ASID: 2^20
entries of eight bytes, each a host base and a kind (cached, scratchpad, uncached RDRAM, accelerated, slow), rebuilt
entry by entry on TLBWI, TLBWR and the mode's and Config's writes, and swapped on an ASID change. The sealed review and its
audit both favour an inline software table over fault-and-patch under Cranelift, and so does this proposal: it keeps
every rule in one place, and rewriting emitted code is not something `emusen-jit` does. An access is a shift, a load, a
compare on the kind and the access; about five host instructions before the data cache's own.

**The caches stay modelled, exactly, on the default path.** A cached access compares the data cache's two tags of its set
as §48.4 does, and is made in the line; a miss is a helper that refills and writes back exactly. The sealed review's
audit corrects the review here: the data cache class is large (the other emulators' instant-DMA workarounds, notes on
about 26 serials that a game works on hardware only because of the cache, even the BIOS's font drawing), so **C2's
coherency part is not taken, and not offered as a default** (§3.7). The cost is two tag compares an access, about 1 ns on
the 431 to 789 thousand accesses a field: 0.4 to 0.8 ms of the EE's 3.7 ms desktop budget. **The instruction cache stays
modelled** too, since it is what makes self-modifying code exact (below).

**Self-modifying code.** A store never reaches the instruction cache (§17.3, §44.3), so a compiled block is exactly what the
interpreter's fetches would read as long as the lines it spans still hold the words it was compiled from. Today the entry
compares every line of the block with the cache's words. Proposed: **a generation count for each of the instruction
cache's lines**, raised by every refill and every CACHE invalidation; a block records its lines' (slot, generation)
pairs and compares them at its entry and at each line it crosses (one load and one compare a line). A refill whose words
equal the old ones keeps the blocks valid after a comparison on the slow path. No page needs write protection and no
store is slowed for code, because the hardware's incoherence already isolates the code. Code run uncached (kseg1 or an
uncached page in RDRAM) has no cache to isolate it: those pages carry a version stamp in the page table, raised by every
store and DMA write into them and checked at a block's entry, with a per-page count that sends a page that keeps changing
to the interpreter and decays back (`EmuSen_FastExact.md` §4's hysteresis). The ROM cannot be written.

**The DMA hazard.** The DMAC is the only unit that reads or writes RDRAM and the scratchpad behind the EE (§48.5). The
sealed review's audit widens the review's case: it is not only EE stores into RAM a DMA channel is reading, but also DMA
writes into RAM the EE reads, and scratchpad DMA in both directions (one series of games' scratchpad errors is the largest
group of notes on the other emulators' DMA timing switch). With the caches modelled, a cached hit never touches RDRAM, so
the EE's RDRAM traffic is its uncached accesses and its refills and write-backs, all of which are either an inline
uncached access or a helper. Proposed: **while a channel is active, the pages it is reading or writing are marked
watched in the page table** (for a chain, the current tag's page and the current transfer's range, updated at the
DMAC's events; the scratchpad's one entry while an SPR channel runs); an access to a watched page takes the slow path,
which brings the board to the access's cycle first. Exact by §3.0, and today's guard and rollbacks (§48.5) are no longer
needed, since no access runs ahead of a DMAC step it could conflict with.

**Why it reaches its budget.** 3.7 ms of the desktop over 1,665 to 3,986 thousand instructions is 2.22 to 0.93 ns each
(§2.4). The integer instructions with registers in host registers are three to five host instructions; an access with
the page table and the data cache's tags about twelve; a block's fixed costs (the horizon compare, the quiescent test, the
line generations, the chained jump) about ten, over blocks that will be longer than today's once nothing but control
transfers ends them. Mars's recompiler measured 1.1 to 2.1 ns a compiled instruction while still calling the
interpreter's handlers for everything but simple instructions (`Mars_Recompiler.md` §7). **P-R4**: the EE's whole cost at
R4 is at most 2.2 ns an executed instruction on the dock and the island (desktop); the arena's 0.93 is not predicted.

### 3.3 The VUs: whole microprograms once the GIF runs at its own events

**What changes.** Today VU1 is brought to the EE's time after every EE instruction, 1.2 to 1.3 pairs a call (§57.2),
because the GIF's arbitration at every board step reads PATH1 from VU1's memory and VIF1 writes VU1's memory while it
runs (§57.4). With the board at events (§3.1) the first reason goes: VU1 is run only at its observations, which are the
EE's accesses to its registers and memories (and VPU-STAT, FBRST), VIF1's waits (MSCAL, FLUSH, FLUSHE, FLUSHA), the
horizon, and a state. **Between observations VU1 runs as one call of compiled code**, hundreds to thousands of pairs.
The second reason is kept exactly rather than removed: **the other units' accesses to VU1's memory are interleaved inside
VU1's run at their cycles.** VU1's code holds the earliest cycle of any pending foreign access (VIF1's next write, PATH1's
next read by the GIF) in a register and compares it at each pair; when a pair would issue past it, a helper makes the
access (VIF1's unpack step, the GIF's qword) and returns the next. The order of every read and write in VU1's memory is
then today's, without the board being stepped, at one compare a pair and a call for each of the 35 to 151 thousand VIF1
writes a field into a running VU1 (§57.2).

**The lookahead** (§3.1) is computed when a microprogram is compiled: for each pair, the fewest pairs to an E, T or D bit
or an XGKICK. A VU1 that has 500 pairs to its next such point lets the EE run 500 cycles without looking at it.

**VU0.** Macro instructions run in the EE's blocks with their issue timing (§3.2). Micro mode (VCALLMS) runs whole
between the EE's observations of VU0, and the interlocked moves (QMFC2.I, CQMFC2.I, CFC2 and CTC2 with the interlock,
VCALLMS while busy) bring VU0 to the EE's exact cycle, as the audit says the most compatible emulator does; that is the
sealed review's pointer 7, exact and cheap.

**The floats on the pair's lanes.** §56 proved fast paths for `sapphire::float`'s operations one call at a time. A pair's
upper operation is up to four lanes of the same operation; proposed is a vector form of each proven fast path, the four
lanes computed in the host's double or wider lanes with a filter (operands and result in the classes the proof covers),
the call to the exact function kept for a lane the filter refuses. Each vector form is proven as §56.5 proved the scalar
ones (exhaustive on a reduced format, stratified on the full one), and its equality evidence recorded before it is the
default.

**Flags.** The MAC, status and clip flags are written only where a pair of the same microprogram can read them before
they are overwritten, the analysis exact within one program; they are always written at the program's end and at every
observation. The sealed review's pointer 11 holds: never across programs (C4 stays refused, §45.9).

**Program identity.** Microprograms are cached by start and validated through micro memory's lending count (§45.4). By
the audit's hint 7, games that patch a microprogram's immediates between runs (the I-bit constant, the integer
immediates) recompile constantly; the identity check is to exclude those words and the code to load them at run time.
Exact; it costs those words their constant folding. R2 counts the recompiles before R6 decides it.

**Why it reaches its budget.** §57.7: of VU micro mode's 15.7 to 20.2 %, the work is 6.8 to 8.7 % and the rest is the
catch-up and dispatch of 2 to 9 million calls a field, which whole runs remove. The work itself is not small: 8.0, 8.7
and 6.8 % of 228, 408 and 702 ms over 571, 1,343 and 2,955 thousand pairs is 32, 26 and 16 ns a pair, mostly the float
calls and their frames, so the work must fall 7 to 18 times too, which is what the vector fast paths and the inline
plan are for. **P-R5**: at R6, VU0 and VU1 together at most 2.0 ns a pair on the island and
4.7 on the dock (desktop), floats in. The arena's 0.90 is not predicted; §9 Q5 takes it.

**Why VU1 is not on a host thread.** §57.4 showed an exact trailing thread hands over at every board step and costs as
much as VU1's whole work. The audit adds the stronger reason: the one working VU1 thread the review describes makes
what the EE sees depend on how far the host's thread has got, which fails `Sapphire_Plan.md` §5.13's first condition
(results deterministic, units synchronised at fixed points of emulated time). A thread could be offered only with every
handoff pinned to emulated time: VU1's side running in fixed quanta of emulated cycles with the board's couplings to it
delivered at quantum boundaries. That is C7 for the VUs, timing-only, and is listed in §3.7 as an option that R7 may
propose for the arena with its measured cost; the audit's hint 4 gives the measured alternative the others use, VU entry
in larger quanta on the same thread, which the whole runs above already are, exactly.

### 3.4 The board: observation points; the GIF and VIF at events; DMA and the FIFOs on one timeline

**The observation points**, enumerated; nothing else crosses from one side to the other.

| Observer → observed | Points | What the observed side must be brought to |
| --- | --- | --- |
| EE → any unit | an access to 1000_0000h–1000_FFFFh and the units' FIFOs, the GS's privileged registers (1200_0000h), VU memories through the window (1100_0000h), the SIF's registers, IOP RAM through 1C00_0000h | the board to the access's cycle |
| EE → VU0 | COP2's interlocked moves, VCALLMS while busy, CFC2 of VU0's status, BC2F/BC2T | VU0 to the instruction's cycle |
| EE ↔ DMAC | a cached refill or write-back, an uncached access, or a scratchpad access to a watched page (§3.2) | the board to the access's cycle |
| any unit → EE | an interrupt line changing (INTC, the DMAC's lines, COP0's timer) | the EE may not pass the change: the horizon (§3.1) |
| VIF1, GIF → VU1 | VIF1's writes into VU1's memory, PATH1's reads of it | interleaved in VU1's run at their cycles (§3.3) |
| VU1 → GIF | XGKICK | the kick handed to the GIF at its cycle; PATH1's place among the paths as the hardware reserves it |
| VIF1 → VU1 | MSCAL, MSCNT, FLUSH, FLUSHE, FLUSHA | VU1 to VIF1's time; VIF1 waits for VU1's end or PATH1's |
| DMAC ↔ VIF, GIF, IPU, SIF, SPR | a FIFO's room, a qword moved | on one timeline, below |
| IOP ↔ EE side | SIF's FIFOs and registers, the IOP's wake of the EE's DMAC | §3.5 |
| PCRTC → GS | CSR's field and blanking bits | the PCRTC's own timing (the audit, correction 9) |

**The GIF at its own events.** The GIF is visited when something arrives (a kick, PATH2 or PATH3 data, a mask change, a
write to CTRL or MODE, a BUSDIR change) and at its own next event (a packet's end; inside an IMAGE transfer in the
interrupt-mode slices), never at a board step that brings it nothing. This is the exact skip of the GIF and VIF plan
paused on 2026-10-28 before its build, kept as the first step of R5 since it stands on today's board: skip `Gif::run_to`
when nothing has arrived since its last run and the time is before its resume; skip an idle VIF's `run_to` by its idle
clock move; run unpacks in a tight loop while their times fall before the target; keep one run at every step where a kick
arrives under PATH3's slice mode, whose interrupt depends on that. On today's island profile the GIF's per-step checks
that find nothing are about 6 % of the thread and the packing work 0.5 %.

**The GS's observable state from the packets.** SIGNAL, FINISH and LABEL, and CSR's bits that follow from them, are
answered by the GIF's parser on the emulation thread when it parses the packet, never by waiting for the rasteriser
(the sealed review's pointer 2), with the audit's two fixes: CSR's field and blanking bits come from the PCRTC's timing,
and **a second SIGNAL before the first is acknowledged stalls the GIF**, flow control the parser models. Only a
local-to-host transfer and the picture at the field waits for the drawing.

**DMA and the FIFOs on one timeline.** A channel's data moves at its slices into the destination's FIFO, and a channel ends
when its last qword is accepted by the destination, not after an estimate: completion, the FIFOs' room and the
consumers' progress are one sequence of events. The sealed review's audit corrects the review's claim that refusing a
write to a busy channel is the hardware's rule: the other emulator's option finishes the busy transfer and then accepts
the write for most channels, refuses only the IPU's, and its authors call the rule unconfirmed; **that is a dispute
candidate**, opened at R5 if a program shows it, not a design input. Where today's DMAC already moves data and raises
completion on one timeline, nothing changes; R2's census lists any channel where it does not.

**Why it reaches its budget.** GIF and VIF today are 16 to 39 ms, nearly all per-step checks (the paused plan's split:
packing work 0.5 % and unpacking 0.2 % of the island's thread). **P-R6**: at R5 the GIF and VIF together at most 0.9 ms a
field on the desktop on all three windows.

### 3.5 The IOP and the SPU2: in batches

**What §54.9 found.** The IOP stands at every EE instruction's time because (1) its store that starts a SIF channel wakes
the EE's DMAC at the board's time after the EE instruction during which it happened, (2) the EE reads the SIF's
registers and IOP RAM at instruction granularity, and (3) its per-call clocks are state.

**What changes.** (2) becomes an observation point (§3.4): a helper brings the IOP to the EE's cycle before the read. (1)
is kept exactly: with the block's offsets known (§3.2), the EE instruction during which a given IOP time falls is
computable, so the IOP runs **at its own instruction times as a unit of the event loop**, at most one EE block behind,
and a wake that falls inside the block just run is placed at that instruction's end; if the DMAC then has a step inside
the block before an access it conflicts with, the watched pages of §3.2 have already brought the board along at that
access, so no rollback is needed. (3) is the bookkeeping named in §3.1. The SPU2's samples are events every 6,144 EE
cycles and need nothing else; its DMA and address-match interrupt are the DMAC's and the INTC's (the audit: the SPU2
tolerates slack apart from those).

**Not proposed: the IOP batched by a slack (C8, C7).** The sealed review called the EE and IOP "routinely skewed by
thousands of cycles" elsewhere; the audit corrects it (at most about 3,000 EE cycles, falling to tens when either side has
an interrupt due) and adds that the documented boot race (a game's own module resetting the disc driver while a load is
in flight) depends on the two processors' relative speed and was cured elsewhere by changing the EE's speed. That argues
against any IOP slack during boot, and the exact form above costs no more.

**Why it reaches its budget.** The IOP and its units are 6.2 to 8.9 ms today, 1 to 2 % of the frame (§57.5, the audit's
point 12), but 8 to 10 times their 1.2 ms budget once everything else falls. Batching removes the calls; the IOP's
interpretation remains, about 10 ns an instruction with its bus timing. **Unknown**: how many IOP instructions Jak's
windows execute after idle skipping. If R2's census finds more than about 90,000 a field, R7 proposes a block tier for
`blackmoon-r3000` (exact, shared with Diamond, which then benefits too) rather than a slack.

### 3.6 The GS: the device path, and what its throughput must be

**What changes.** The drawing leaves the processor: S44's mirror (the 4 MiB local memory on a lent device in `gs::local`'s
layout, the transfers on it, §52), then S45's compute rasteriser held bit-exact and S46's picture on the device, as
`Sapphire_Plan.md` §5.4 and §52.11 Q1 recommend: a compute rasteriser with tiles owned by one workgroup and primitives
applied in order, which the sealed review describes as the road one other emulator takes (native layout on the GPU, per-
pixel order) and `EmuSen_FastExact.md` §12 as the exact choice. The emulation thread keeps the parser and register model
(§3.4) and appends primitives and register changes to a command stream; the primitives' setup moves to the device.

**What stays exact.** Every pixel: the rasteriser is graded against the software reference, which stays in the tree as
the reference path, and §52.6.2 found no case the device answered that differed from the processor.

**The throughput it must have.** The device draws field n while the emulation thread runs field n + 1, so it has about
one field, 16.68 ms, less its submission and the picture's hand-off: **at most about 14 ms of the handheld's GPU a
field** for what four of the desktop's processor threads draw in 90 to 98 ms each on the dock (about 380 thread-ms),
and 34 to 48 ms each on the island and the arena. P-S44b (§52.6.4) predicts the exact arithmetic alone at 0.3 to 0.6 ns a
pixel on the RX 6800; the handheld's GPU has about a quarter of the RX 6800's compute-unit cycles a second at full clock
[A, from the parts' published compute-unit counts and clocks; not measured], so 1.2 to 2.4 ns a pixel for the
arithmetic, before coverage, texel fetch through the page tables and the ordered pixel pipeline. **Unknown**: Jak's pixels
a field in the 3D windows (§52.6.3 counted only the menus), which R2 counts. If the dock draws 3 million pixels a field,
the arithmetic alone is 3.6 to 7.2 ms of the handheld's 14: within, but not by a margin that leaves the rest free. R8
retires P-S44b on both devices before S45 starts.

**The emulation thread's GS work.** 15 to 31 ms today, which includes the setup and the workers' job building. **P-R7**:
with setup on the device, parsing and recording at most 0.9 ms a field on the desktop (the 1.2 ms budget), on the 1 to 4
MB a field the transfers move (62 to 250 thousand qwords at 3 to 10 ns). R2 splits today's 15 to 31 ms first.

### 3.7 The compromises

Every item of §3.1 to §3.6 is exact against today's core unless listed here. Timing-only compromises change when
something happens and no byte a game reads where the game does not depend on that timing; byte-changing ones change a
value a game reads or a pixel or sample the player gets (`Sapphire_Plan.md` §5.13).

| # | Compromise | Kind | Default | When it is proposed |
| --- | --- | --- | --- | --- |
| C1 | A cost per block instead of the issue model, COP2 macro instructions and their flag reads kept on the model | timing-only | **off**; proposed as default only if R4's static form misses its budget, after FMV is measured (§6.4) | R4, with §9 Q3 |
| C7 (VUs) | VU1's side run in fixed quanta of emulated cycles on its own thread, every handoff at a quantum boundary | timing-only (VU1's busy state, its T and D interrupts and PATH1's SIGNAL, FINISH and LABEL reach the EE up to a quantum late) | **off**; an option at most | R7, only for the arena, with §9 Q5 |
| C8, C7 (IOP) | The IOP run with a bounded slack | timing-only | **not proposed** (§3.5) | never during boot |
| C2 (coherency) | The data cache's coherency dropped | byte-changing | **not offered** (the audit's correction 1) | — |
| C3 | Floats on the host's arithmetic with clamping | byte-changing | off, an option | unchanged |
| C4 | VU flags across programs | byte-changing | refused (§45.9) | — |
| C5 | The host's rasterisation and blending | byte-changing | off, an option | S46 at the earliest |
| C6 | Render targets on the device with guessed reinterpretations | byte-changing | not proposed: the mirror is exact | — |
| C9 | The CDVD's delays shortened, **short reads and seeks only, long seeks kept** (the audit's hint 5: games issue DMA during long seeks) | changes a loading screen's length | off, an option | when the scoreboard reaches loading |
| C12 | Code pages run uncached sent to the interpreter when they keep changing | none (speed only; the interpreter is exact) | on | R4 |
| C13 | The picture presented a field late on the device | none in emulation (presentation latency) | decided at S46 | S46 |

**Out of the box, beside the compromises**: deinterlacing chosen from the registers (field or frame mode, whether
consecutive fields' display start differs by a line, a game that interlaces itself) with an override, and the emulated
field rate locked to a 60 Hz panel with the sound resampled, since 59.94 Hz drifts a frame about every 17 s (the audit's
hint 8). Both are the frontend's and the display's, change no result, and belong to the CRT and display work rather
than to these stages.

## 4. Why the recompiled tier is slower than the interpreter today

§58.4: 234.1 and 242.5 ms against the interpreter tier's 229.6 on the dock, 413.7 and 433.8 against 403.3 on the island,
714.6 and 744.1 against 692.9 on the arena. The figures that explain it are §58.5's: a compiled instruction's code costs
7.4 to 9.2 ns, the dispatcher, exits and replay about 60 ns a compiled step (§58.7), and an interpreted instruction about
40 ns before the dispatcher's share. **A compiled instruction therefore costs about 69 ns all in against about 40 for an
interpreted one** (with the caveat that the 60 ns includes the replay's board work, which the interpreter tier pays in
the board's own row, and that the shares are sampled at synced N = 64 while the counts are N = 1's). The likely causes,
each from a measurement, and what removes it:

| # | Cause | Evidence | Removed by |
| --- | --- | --- | --- |
| 1 | **The replay keeps the board per instruction.** A block moves the board's per-step calls to its exit and makes them there, in order, with the IOP caught up at each time (§44.5, §48.5); it saves the calls' dispatch, not their work. | "EE: dispatcher, exits and the replay's per-step work" 41.5 / 44.3 / 81.9 ms (§58.4), against compiled code 6.4 / — / 10.0 | the board at events (§3.1): no replay |
| 2 | **Short blocks and constant switching between tiers.** Every MMI, COP1 and COP2 instruction ends a block; 44 to 65 % of loads and stores leave (a miss, a register, an event, a misalignment); 28 to 65 % of the rest run in the interpreter between a block's exit and the next entry (§58.4). Each switch pays the dispatcher's lookup and its entry checks (§44.3: not in a delay slot, CTE, BPC, the interrupt check, every line of the block compared with the cache's words), and the registers and the issue model's fields through memory. | 54 to 71 % of instructions interpreted on the recompiled tier | full coverage and slow paths that return (§3.2) |
| 3 | **The issue model computed at run time inside the code.** Most of a block's bytes are the issue model (§48.15.6); its fields are loaded at entry and stored at exit. | the timing model 13.8 / 12.1 / 26.8 ms; 9.2 ns a compiled instruction against Mars's 1.1 to 2.1 | the static form (§3.2) |
| 4 | **Rollbacks.** A replay that stops for an interrupt or the DMAC undoes a block's stores and runs its instructions again in the interpreter (§48.5). | 23,465 rollbacks over Jak's first 400 frames with the guard kept (§48.5) | the horizon, which never lets a block run past an event, and watched pages (§3.2) |
| 5 | **The dispatcher between every pair of blocks.** No chaining. | inside row 1's figure (§55 to §57 count the dispatcher and the replay together, 5.6 to 10.9 % of the thread) | chaining (§3.2) |
| 6 | **Checks per access.** Each load and store compares the cycles with the board's next event (§48.4). | (in rows 1 and 3) | the horizon compare once, at the entry |

What the table does not settle: how the 60 ns divides between causes 1, 2 and 5. R2's census counts block entries, the
mean block length and the replay's calls, and R3 is gated on the recompiled tier beating the interpreter tier (§7).

## 5. Modularity: the components and their surfaces after the redesign

Each component has one job; what crosses a boundary is listed, and nothing else does: no unit reads another's fields,
and none holds another's pointer except through the surfaces below.

| Component | Its one job | Surface (version) | What crosses it |
| --- | --- | --- | --- |
| The core ABI | the frontend's view of Sapphire | core ABI v1, unchanged | frames, sound, input, states, settings, the lent device |
| `sapphire::board` (the event loop) | which unit runs next, to when; the horizon | `Unit`: `next_event`, `lookahead`, `run_to`; the board's own module surface, versioned with SPH2 | times in EE cycles only |
| `sapphire::ee` (interpreter, `ee::jit`) | the R5900 and its caches | `Ee::run(horizon) -> Exit`; I/O through `Bus::access(addr, cycle)`; the page table it owns | cycles, accesses, exits; the horizon in |
| `sapphire::vu` (both units, `vu::jit`) | VU0 and VU1 | `Micro`: `run_to`, `lookahead`, the foreign-access agenda (§3.3), the memories by `Bus` | cycles, kicks, accesses |
| `sapphire::vif`, `gif`, `dmac`, `ipu`, `intc`, `timers`, `pcrtc` | each its unit | `Unit` | FIFO qwords, lines, events |
| `sapphire::gs::parse` | the GS's registers, privileged registers, CSR, SIGNAL/FINISH/LABEL | a command stream of register writes and transfers, plain data (versioned, below) | the stream out; CSR's timing bits from the PCRTC in |
| `sapphire::gs::draw` (`gs::device::Path`) | drawing: the software reference or the device | consumes the command stream; local-to-host reads answered | the stream in, pixels and readbacks out |
| `sapphire::float` | the console's floats, exactly | functions and their proven fast paths, scalar and vector | values |
| The IOP side: `blackmoon-r3000`, `blackmoon-spu`, the CDVD, SIO2 | the second computer | `blackmoon-r3000`'s and `blackmoon-spu`'s crate surfaces (unchanged); the SIF as a `Unit` | SIF qwords, registers, the wake |
| `emusen-jit` | a recompiler's thread, publication, W^X, budget, verifier | its crate surface, **a minor version adding block chaining's patch step as an optional capability**; MarsRT does not take it | compiled functions, patches |
| `emusen-gpu` | one Vulkan compute device | its crate surface; the lend's features word of §52.11 Q2 when S46 needs it | buffers, programs, submissions |

**The command stream is the new surface**, and the one worth versioning first: the GS's parser and its drawing meet only
there. It is the format of the audit's hint 1 (GS record and replay, §6.3), so one definition serves the device path,
the software path, the record and the replay: the GS's registers and local memory at the start, then the parser's output
with field marks and the PCRTC's state at each field (merge and feedback output depend on the previous field's). It is
plain data, so a DianaSh command can read and write it.

**Tools.** Every tool the stages add is a DianaSh command written in Rust, plain text or plain data in and out: the
scoreboard (`ps2score`, §6.4), the canonical digest (§6.1, a field of the bench's existing output), the census (R2), and
the GS stream's record and replay (`gsreplay`, §6.3). The names are proposals.

## 6. Verification

### 6.1 Each stage proves its results unchanged

**Where timing is unchanged** (R3, R4, R5, R6 and R7's exact forms; everything in §3 not in §3.7), the oracle is today's
core: the boot's 760 frames and Jak's 900, the three windows' 300, with the interpreter tier and the stepped board as the
reference; ps2autotests' 78 texts byte for byte today's; the verifier (§44.6) stepping the interpreter beside every
block; random programs as at S36 and S37. Picture and sound digests equal frame by frame. **The state digest is replaced
by a canonical digest** for the board stages: a digest at the field's end, the board caught up, of everything a program
can read (RDRAM, the scratchpad, both caches' lines and tags, every processor's registers, VU memories, the GS's local
memory and registers, IOP RAM, SPU2 RAM, the units' architectural registers and FIFO contents), excluding the board's
own clocks (§3.1). R2 adds it beside SPH2's and shows it equal on today's core across tiers before any stage depends on it
(`EmuSen_FastExact.md` §10's canonical serialisation).

**Where timing moves** (a compromise of §3.7 taken), digests are expected to change, and the evidence is: ps2autotests
with no new failure and every timing test (§58.6's list) examined; the scoreboard (§6.4) at every level a game reached
before; pictures at the games' own anchors (a script reaching a game state, not a frame number) compared with the
reference path; and the GS streams of §6.3, which keep the GS's goldens valid when the CPU's timing shifts frames.

**The reference path stays.** The stepped board and the issue model remain a setting (`sync`, `stepped` or `events`;
`ee_timing`, `model`, `static` or `fixed`), so every compromise has its exact value one setting away, as §5.13's third
condition asks, and so the exact stages are graded against the same tree.

### 6.2 Seeded faults

Each stage seeds three to seven faults aimed at its exactness argument: an observation point left out of §3.4's table, a
horizon one cycle long, a tie broken the other way, a line generation not raised on a refill, a watched page not
watched, a quiescent test that passes a busy multiplier. Each must be caught by a named test.

### 6.3 GS streams, recorded and replayed

The audit's first hint: a corpus of recorded GS input (the start state and the stream of §5, with field marks), replayed
headless, each field's picture digest compared with the software reference. It grades the device path (S45, S46) and any
of C5 field by field without the whole machine, and it separates a GS defect from a CPU-side timing change. The corpus
comes from Sapphire's own runs of the tester's games, kept out of the repository with the discs; synthetic streams from
`SyntheticRom`-style builders go in the tests.

### 6.4 The compatibility scoreboard

The scoreboard is the first measure of the goals: **each of the handheld's PlayStation 2 games, run headless** (the
probe's discs under `~/.cache/emusen/probe/sapphire/discs/`, never off the NAS), at three levels: **boots** (the game's
own first picture), **reaches gameplay** (a press script to a gameplay anchor, as Jak's windows have), **looks right**
(pictures at anchors against the reference path, then by eye on the first pass), and, on the handheld, **its speed** (mean
and 95th percentile ms a field on battery). One row a game in plain text, from `ps2score`. The audit's hint 2: no other
emulator runs its library automatically, so this is Sapphire's to build; and the games are best chosen to cover the
classes that break elsewhere: DMA completion, PATH1 and PATH3, the VIF and GIF FIFOs, VU0's macro flags, floats, data
cache coherency, FMV and the CDVD. **FMV is in every stage's check from R3**, since it is the audit's test of the EE's
timing (§3.2). **The list of games is the tester's** (§9 Q2).

## 7. Stages, in order

Sizes are in lines of code and in steps of the kind §55 to §58 were; "buys" is the prediction it is gated on. Stages R1 to
R7 are exact (but R1 builds nothing that stays) and run on the desktop; R8 and R9 need the tester present for the GPU.

| Stage | What | Gate (measurable) | Size | Expected to buy |
| --- | --- | --- | --- | --- |
| **R1** | **What stalls Jak at N ≥ 512**, on §58's synced patch kept outside the repository: three builds, each removing one candidate: (a) the interrupt lines exact (the board brought along before any line could change: the horizon of §3.1 approximated by the units' events and a VU's or VIF's run); (b) VU1's end and XGKICK as VIF1 sees them brought to VIF1's time; (c) idle skipping off. Also: why synced N = 64 already drifts in digests (§58.3) | a named cause: one build whose Jak changes DISPFB within 5 % of N = 1's count at N = 4,096; **and** either a build at N = 64 whose picture digests equal N = 1's, or the named reason none can be | S: two to three throwaway builds, one step | the horizon's rules (§3.1); a falsification test of §3.0's claim before anything is built on it |
| **R2** | **The instruments**: the canonical digest (§6.1); a census build counting, a field, the EE's observations by class, block entries and their mean length, quiescent entries, the replay's calls, watched-page accesses, the events at the board, VIF1 writes into a running VU1, VU recompiles, IOP instructions executed, Jak's pixels drawn; `ps2score`'s first form on the desktop; the bench on the handheld once, for the factor | the digest equal across both tiers and N = 1 runs on the boot, Jak and the windows; the census's numbers recorded; P-R1, P-R3 and §3.5's IOP count settled; the handheld's factor measured | M: about 800 lines | numbers for R3 to R7, and three predictions settled before their stages |
| **R3** | **The EE recompiler I: coverage.** MMI as vector operations, COP1 and COP2 macro in blocks, every load and store kept in the block, slow paths returning, the data cache's refill as a helper. Today's replay kept, so timing is unchanged | digests and the verifier equal to today's; at least 95 % of executed instructions compiled (P-R2); **the recompiled tier faster than the interpreter tier on all three windows**, two rounds | L: about 3,500 lines of emitter | the interpreter's 47 / 47 / 88 ms to under 10 / 10 / 20; with the dispatcher's exits fewer, the shipped windows 234 / 414 / 715 → about 170–195 / 330–370 / 560–620 (P-R8) |
| **R4** | **The EE recompiler II: blocks.** The page table, chaining with the horizon compare, the line generations and uncached code's stamps, the static timing with the quiescent test and run-time terms, the dynamic form for the rest | digests equal; the EE's code at most 2.2 ns an executed instruction on the dock and island once R5 removes the replay (P-R4, measured on the integer loop and the windows' blocks with the replay's time subtracted); the quiescent fraction as P-R3 said | L: about 2,500 lines | the timing model's 14–27 ms to under 2; the dispatcher's lookup gone from the steady state |
| **R5** | **The board at events.** First the GIF's and VIF's exact skip (§3.4) on today's board; then the event loop, the horizon with lookahead, observations at exact offsets, the watched pages, the replay and rollbacks retired, the IOP at its own instruction times, the DMA and FIFOs on one timeline, the GS's parser answering SIGNAL, FINISH, LABEL and CSR | canonical digests equal to today's on the boot, Jak and the windows; ps2autotests' texts unchanged; Jak's DISPFB counts equal (the stall gone by construction); the board and scheduler under 0.52 ms (P-R1) and the GIF and VIF under 0.9 ms (P-R6) a field | XL: about 4,000 lines, the largest | the board's 21 / 36 / 51 ms, the GIF's and VIF's 16 / 27 / 39 and most of the dispatcher's row: the windows at about 70 / 140 / 205 ms (P-R9), the VUs, the GS's thread side and the IOP then leading |
| **R6** | **The VUs whole.** VU1 and VU0 run between observations, the foreign-access agenda, the lookahead table, flags within a program, vector float fast paths proven, immediates out of the identity if R2 found recompiles | the verifier beside every pair; digests equal; P-R5 | L: about 2,500 lines and the proofs | the VUs' 35 / 85 / 141 ms to under 2.7 / 2.7 / 3–6 (the arena's estimated, not predicted); the windows at about 40 / 60 / 70 (P-R10), the GS's thread side, the IOP and the unattributed remainder then leading |
| **R7** | **The rest of the thread, and the arena's decision.** The GS's thread side to parse and record only (P-R7); the IOP's block tier on `blackmoon-r3000` if R2's count asks for it; then the three windows profiled again, and if the arena is short, C7 for the VUs designed with its handoff count and cost for the tester | digests equal for the exact parts; the dock and the island under 9.93 ms a field on the desktop with the GS drawn elsewhere (P-R11); the arena's shortfall measured and named | M to L | the dock and island at the target on the desktop; the arena's number |
| **R8** | **The GPU window** (the tester present): P-S44b on the RX 6800 and on the handheld; the device's ns a pixel on Jak's profile | P-S44b retired on both; the handheld's ns a pixel recorded | S: one window of about 40 minutes a device | whether S45 is bit-exact on the device as recommended |
| **R9** | **The GS on the device**, S45 and S46 as planned (the tester present for the GPU runs) | pictures bit-equal to the software reference on the boot, Jak and the GS streams (§6.3); the device at most 14 ms a field on the handheld on the windows | XL (S45 and S46's own sizes) | the GS workers' 90–98 ms off the processor |
| **R10** | **The handheld scoreboard.** Every scoreboard game on the handheld on battery; any compromise of §3.7 taken only here and only with its measurement, each named in machine info | the scoreboard's games at 13.4 ms mean and under 16.7 at the 95th percentile; every level each game reached kept | M | the first scoreboard |

**P-R12**: R3 to R6 together put the dock and the island within the handheld's budget on the emulation thread
(P-R11 on the desktop × 1.35), and the arena between 1.2 and 2 times over it.

**What can run beside what.** R1 and R2 first, in that order. R3, R4 and R5 in order, since each one's gate uses the one
before. R8 can go whenever the tester and the machine are free for it, before R9. R6 after R5.

## 8. Risks and unknowns

| # | Risk or unknown | The cheapest check that settles it | When |
| --- | --- | --- | --- |
| 1 | The handheld's factor of 1.35 is argued, not measured | the bench's three windows on the handheld once, today's build | R2 |
| 2 | The horizon is short (VU1 with frequent XGKICKs or T bits, DMA in short slices), so the board steps nearly as often as now | the census: the distribution of horizon lengths a field, from R1's builds | R1, R2 |
| 3 | §3.0's exactness fails because some coupling is not in §3.4's table (the synced N = 64 build already drifts in digests) | R1's gate: N = 64 with digests equal to N = 1, or the reason | R1 |
| 4 | Today's stepped board has artefacts (orders within a cycle, the wake's rounding) that an event loop cannot reproduce cheaply, so R5 cannot be digest-equal | R5's first step bisects the first differing field of each window; each artefact is reproduced, or named as the stepped model's and the tester asked | R5 |
| 5 | The quiescent fraction is low, so the static timing rarely applies | the census on today's code (the entry state is in the issue model's fields) | R2 |
| 6 | The EE's per-instruction cost does not reach 2.2 ns | a prototype of the static form on the project's integer loop, timed against §44.8's 80.6 ms | R4's first step |
| 7 | The vector float fast paths cannot cover the multiplier's closed form (§56.4) | the closed form in host vector lanes, exhaustive on a reduced format, ns a lane measured | R6's first step |
| 8 | The GS's device throughput on the handheld | S44's `exact64` on the handheld | R8 |
| 9 | Jak's 3D draws more pixels a field than the device can take within 14 ms | the census's pixel count | R2 |
| 10 | The IOP's interpretation alone exceeds its 1.2 ms budget | the census's IOP instructions after idle skipping | R2 |
| 11 | Chaining in `emusen-jit` disturbs MarsRT | the capability optional, MarsRT's tests in the blast radius of the R4 change | R4 |
| 12 | Full coverage grows the code past the budget, or the compiler's throughput stalls entries (as at the memory card screen, §48.15) | the census of bytes compiled and entries that waited, at R3's gate | R3 |
| 13 | Uncached code that keeps changing thrashes | the census's stamps raised a field on the scoreboard's games | R4 |
| 14 | A dispute the redesign reaches: the busy-channel write (the audit's correction 6); a TLB entry mapping nothing, which the audit says one demo relies on | programs written for each, then the referees as black boxes | when a program shows it |
| 15 | The scoreboard is three games today | the tester's list (§9 Q2) | R2 |

**The proposal's weak points, stated before the critic does.**

1. **The arena is not shown reachable**, on one thread or two (§2.4); everything about it after R6 is a number to be
   measured, not a design.
2. **The budget table is an allocation, not a derivation.** Its factors are honest only as a product: the thread's 21, 30
   and 49 times.
3. **Every unit cost the design reaches for is below anything Sapphire has measured**; for the EE the support is Mars's
   1.1 to 2.1 ns, from a different processor and a less complete recompiler; for the VU, MarsRT's RSP at about 3 ns, which
   is above the island's 2.0 budget.
4. **Exactness rests on enumeration.** One coupling missing from §3.4 is a silent difference wherever the digests do not
   look; the digests look at three games' windows, the boot and ps2autotests. The scoreboard widens that, but it is not
   a proof.
5. **"Exact" is against today's model, not the console.** If today's model is wrong somewhere, the redesign keeps it
   wrong, exactly.
6. **The sealed review and its audit are someone's reading of others' code**, with rough counts; the classes they name are
   used to choose tests, not as evidence that a design works.
7. **The largest stage, R5, cannot be split much further**: the event loop, the horizon and the retirement of the replay
   only pay together, and only R5's first step (the GIF's and VIF's skip) is separable.
8. **Two compromises the tester may be asked for (C1, C7 for the VUs)** are not ruled out by this design; it only
   arranges that they are asked for last, with numbers.

## 9. Questions for the tester

1. **[decision] Approve the redesign as the plan replacing §58.7's lever list, with R1 as its first stage.** Recommended:
   yes, on the condition that R1's gate (§7) is passed before R3 starts, since it tests the exactness claim everything
   after rests on. If R1 finds that batching cannot keep the digests at N = 64, the proposal comes back revised.
2. **[decision] The scoreboard's games.** Recommended: the PlayStation 2 games installed on the handheld, plus Jak and
   Daxter, Kingdom Hearts and Dark Cloud 2, copied from the NAS into the probe's disc folder; and, if the tester allows
   it, the audit's offer: whether a reader of the sealed sources may pass on which titles stress each coupling class of
   §6.4 (titles are not code, but they come from another emulator's table, so it is the tester's ruling under §1.4).
   Recommended there: yes for titles only, recorded as exposure, nothing else from the table.
3. **[decision] C1 (a cost per block).** Recommended: not now. R4 builds the exact static form first; C1 comes to the
   tester only if R4 measures that form short of its budget, and then in the form of §3.2 (COP2 macro instructions and
   their flag reads kept on the model) and only after FMV is measured.
4. **[decision] The reference path kept.** Recommended: the stepped board and the issue model stay in the tree as settings
   (`sync = stepped`, `ee_timing = model`) for as long as the redesign is graded against them, at the cost of maintaining
   two paths.
5. **[decision] The arena, if R7 measures it short.** Recommended: decide then, on R7's numbers; the option then is C7 for
   the VUs with every handoff pinned to emulated time (§3.3), offered as an option, not the default, and never per game.
6. **[decision] §5.13's second condition and the "no fixes for one game" goal.** The condition says a timing-only
   compromise that changes a bench game's bytes is "narrowed or turned off for the unit or game involved". Recommended:
   amend it to "for the unit involved, for every game": a compromise that breaks a scoreboard game is narrowed for all
   games or turned off, never set per game.
7. **[routine] The handheld for R2's one bench run** (§8 risk 1). Recommended: when the tester next has it free, today's
   build, the three windows on battery under `systemd-run --user`.

## 10. What was read for this proposal, and exposure

`Sapphire_Speed.md` §55 to §58; `Sapphire_Plan.md` §1.4, §5.1, §5.12 and §5.13; `Sapphire_Native.md` §44.1 to §44.6 and
§44.9, §45.1 and §45.3, §52 (S44, from its branch's page); `Sapphire_Recompiler.md` §48.1, §48.4 to §48.6 and §48.15.1;
`Sapphire_Scheduler.md` §54.1 and §54.9; the paused plan for the GIF's and VIF's exact skip (2026-10-28, kept in the
scratch of the step that wrote §58); `EmuSen_FastExact.md` from branch `sim-reference`; the sealed review and the sealed
review's audit. No emulator's source, no firmware, no game data. Nothing was built or run for this page.
