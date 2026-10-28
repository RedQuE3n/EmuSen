# Sapphire: the redesign proposal (2026-10-28, revised the same day)

*A design proposal; nothing in it is built. Decided 2026-10-28 by the tester: the redesign replaces the lever list of
`Sapphire_Speed.md` §58.7, it is reviewed by a critic, and the tester approves it before any stage starts. The first
version (commit `c515ce2e4`) was reviewed the same day with the verdict "revise"; this version answers the review (§9),
records the tester's three answers to it and the decision on unified memory (below), adds the risk checks of R0, and
budgets at a handheld factor of 2 beside 1.35. Its sections are this page's own (§1 to §12); a section number with a page
named is that page's, and one with no page named (§44, §57.4) is the build record's numbering shared by
`Sapphire_Native.md`, `Sapphire_Recompiler.md`, `Sapphire_Scheduler.md` and `Sapphire_Speed.md`. Its stages are R0 to
R10, its predictions P-R1 to P-R12, and the compromises keep the numbers of `Sapphire_Plan.md` §5.13 (C1 to C11), the new
ones C12 and C13.*

*Sapphire's goals, in the tester's words of 2026-10-28: play more games than the other PlayStation 2 emulators, follow
the project's Unix philosophy, and be "bit accurate rather than cycle", with compromises where they have to be; the first
scoreboard is the PlayStation 2 games on the handheld, playing correctly at full speed there; no fixes that only work for
one game; it works out of the box, with additional options.*

**Decided 2026-10-28 by the tester**, answering the review's questions and one of the tester's own:

1. **Exact is measured against a cycle-ordered reference model**, with the goldens re-recorded once, a timing-only
   change. Today's core, which settles several couplings at the end of each EE instruction's step (§4.0), is no longer
   the reference.
2. **Busy, EE-saturated scenes like the arena are in scope.** Whether the emulation is split over a second thread (a
   deterministic split at fixed quanta of emulated time, timing-only) is decided **before R3**.
3. **R1's gate is strict**: digest equality with the reference at small N; failing it stops the plan.
4. **Unified memory is the primary case and a discrete GPU the special case**: most handhelds and laptops share memory
   between the processor and the GPU, as every Apple Silicon Mac does. The GS's device path is designed for unified
   memory first (§4.6).
5. **Two faces of one core, for every core** (`EmuSen_TwoFaces.md`): a cycle face that is the reference, built first and
   never bent for speed, and a byte face with the same results and approximated timing, sharing semantics, state and the
   save-state format, the face a compile-time parameter chosen at a game's start. Sapphire's cycle face is the
   cycle-ordered reference of decision 1; **this proposal builds Sapphire's byte face against it, unit by unit** (§4.10),
   and once the byte face is accepted it is the default and the cycle face the "Accurate" option.

*Sources, and how far each is trusted: the measurements are Sapphire's own (§55 to §58, and the stages they cite). The
sealed review (a report of 2026-10-28 by a reader of other PlayStation 2 emulators' sources, sealed so that its reader
alone saw them) and the sealed review's audit (a second reading of the same sources, which corrects it) are theory about
how others work, not measurements of Sapphire, and their counts of per-game entries are rough; they are used for hints and
for the classes of games that break, never for code. The critic's review is a reading of this page's first version and of
the project's own records. `EmuSen_FastExact.md` (branch `sim-reference`) is a literature reference whose citations were
written without access to the sources and are unverified; its ideas are used and its references are not cited here as
checked. No emulator's source was read for this proposal, and the clean-room rules of `Sapphire_Plan.md` §1.4 apply to
every stage below.*

## 1. In one paragraph

§58 measured the design's own cost: with the board's background work batched and every observation kept exact, Jak and
Daxter runs at 207, 302 and 488 ms a field on the desktop, and the handheld needs 16.68 ms. The shortfall on the
handheld is 17 to 40 times at a factor of 1.35 over the desktop and 25 to 59 times at 2 (§3.4), spread over every unit,
and each unit's cost is overhead per instruction, per pair and per observation more than work. **The proposal is a board
run at events, with the EE in long compiled blocks between them**: every unit is visited only at its own events or where
another observes it; the board lags the EE only up to a horizon below which no line or observed value can change (§4.1);
the EE compiles every instruction class into chained blocks whose timing is folded where it is static (§4.2); the VUs run
long stretches of a microprogram between observations (§4.3); the GIF and VIF run at their events and DMA and the FIFOs
on one timeline (§4.4); the IOP runs at its own instruction times (§4.5); the GS draws on the device, through memory the
processor and GPU share (§4.6). In the terms of `EmuSen_TwoFaces.md`, this is Sapphire's byte face, built unit by unit against its cycle face, the
cycle-ordered reference (§4.0, §4.10), and most of its pieces are exact against it. **The busy scenes do not fit
one thread at a factor of 2** (§3.5: the arena would need 0.63 ns an EE instruction and 0.61 a VU pair), so the
proposal also designs a deterministic split over two threads at fixed quanta of emulated time (§4.7), timing-only, to be
decided before R3. **Nothing is built before R0**, five cheap checks with pass marks (§2, §8) that settle whether the
plan's per-item costs are reachable at all.

## 2. What the plan commits to, and what decides at R0 whether to continue

**Committed now**: R0 only, the five checks below, and no code that stays. **Committed if R0 passes**: R1 (the
cycle-ordered reference and the strict test of batching against it) and R2 (the instruments). **Committed if R1 passes and
the tester decides the split**: R3 to R10, whose goal is R0's design point, the busy field the scoreboard's games actually
run (R0.1), at full speed on the handheld, exact against the reference except for the split if it is taken. Nothing in this
plan commits to a game's full speed before R0 has shown the per-item costs within reach.

| Check | What it measures | Pass mark |
| --- | --- | --- |
| **R0.1** Busy scenes | EE instructions executed and passed, and VU pairs, a field, in gameplay windows of every scoreboard game available | defines the design point, the 90th-percentile field across the games; the plan's tables hold if it is at or below the arena's 3.99 M instructions and 2.96 M pairs, and are redrawn at it before R1 otherwise |
| **R0.2** The emitter's footprint | a ring of compiled blocks at Jak's live-block count, ns an entry and an instruction; host bytes a guest instruction, today's emitter and the static form | at the design point and a factor of 2, ns an executed guest instruction, everything in, at most the EE's allowance (§3.5: 1.13 on the arena split over two threads), and **at most 96 bytes of host code a guest instruction** |
| **R0.3** The handheld's factor | the emulation thread's field time on the handheld on battery against the desktop, with one hot thread and the GPU busy at the redesign's expected load, after the clocks settle | **at most 2.0**; the tables are then redrawn at the measured factor |
| **R0.4** The VU's floats in vector lanes | a kernel of the console's adder and multiplier with MAC flags in host vector lanes, exact integer arithmetic, graded bit-equal and timed | equal on every case graded, and **at most 1.0 ns a lane operation on the handheld** (0.5 on the desktop) |
| **R0.5** Unified memory | on the handheld, 4 MB written by the GPU and read by the processor after a fence, for each memory type; uploads written in place | a host-visible, device-local, processor-cached type reads 4 MB in at most 1 ms after the fence; if none exists, the staging path's cost is recorded and the census's count of local-to-host transfers (R2) decides whether it matters |

**What decides.** If R0.2, R0.3 and R0.4 all pass at R0.1's design point, the plan continues to R1. If any fails, the
plan stops and returns to the tester with the numbers and the gap; the choices then are compromises that change more than
timing (C3 on the VU, or a C1 the audit's conditions allow) or a target narrower than every field at full speed on the
handheld, and neither is this proposal's to make. R0.5 does not stop the plan; it chooses the device path's memory
(§4.6).

## 3. The problem in numbers

### 3.1 The target

A bench frame is one field, and every field's cycles must be run in real time whatever rate the game draws at (§58.2):
**16.68 ms a field on the handheld**, the Legion Go S. The plan's target leaves a fifth spare (a mean of 13.4 ms, no field
over 16.7 ms at the 95th percentile, `Sapphire_Plan.md` §5.12). **The handheld's factor** over the desktop's thread was
argued at 1.3 to 1.4 (`Sapphire_Plan.md` §5.12); the critic points out that on this handheld the memory and fabric clocks
fell to 400 and 400 MHz after two to four seconds of a run and lengthened MarsRT's emulation thread's field by about half
(`Mars_Performance.md` §42.4), and that the redesign's load (one hot thread and a busy GPU sharing the chip's power) is
unlike today's. This page budgets at both:

| | Factor 1.35 | Factor 2 |
| --- | --- | --- |
| The handheld's 13.4 ms target, in the desktop's time | 9.93 ms | 6.70 ms |
| The handheld's 16.68 ms ceiling, in the desktop's time | 12.36 ms | 8.34 ms |

### 3.2 The floors (§58.4, synced N = 64, desktop, ms a field)

Grouped by the units of §4. The EE's float operations are counted with the VUs, since 99 % of `sapphire::float`'s calls
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
| GS workers, busy, each of four (other threads) | 90 to 98 | 34 to 46 | 35 to 48 |

### 3.3 Light scenes and busy ones

A field is 4.92 million EE cycles. On the dock and the island the EE executes 1.67 and 1.93 million instructions and
passes about 4 million cycles in wait loops: it idles about 70 % of the field. **On the arena it executes 3.99 million and
passes only 1.7**: the EE is saturated, and VU1 runs 2.53 million pairs (VU0 0.42). The critic's first objection is that
busy gameplay will look like the arena across much of the library, and that a plan committed to the light scenes repeats
the lever list's pattern at a larger scale. That is an inference from one game, and R0.1 measures it; the tester's
decision 2 puts busy scenes in scope either way.

### 3.4 The budget on one thread

The handheld's ms a field each unit may spend, drawn to the 13.4 ms target, against today's cost on the handheld and the
factor by which each must fall, at a handheld factor of 1.35 and of 2. The allocation is a design choice made from the
work each unit has to do (§3.5), not derived; another allocation moves the factors between rows but not their product.

| Unit | Budget, handheld ms | `jak-dock` ×1.35 / ×2 | `jak-island` | `jak-arena` |
| --- | --- | --- | --- | --- |
| EE | **5.0** | **29× / 43×** | 28× / 41× | **56× / 83×** |
| VU0 and VU1 with the floats | **3.6** | 13× / 20× | 32× / 47× | **53× / 79×** |
| GIF and VIF | **1.2** | 18× / 27× | 30× / 44× | 44× / 65× |
| GS on the emulation thread | **1.2** | 17× / 26× | 35× / 52× | 35× / 52× |
| Scheduler, board, idle skipping | **0.7** | 41× / 61× | 69× / 103× | 98× / 146× |
| IOP, its units, SPU2 | **1.2** | 8× / 11× | 9× / 13× | 10× / 15× |
| Remainder | **0.5** | 19× / 28× | 28× / 41× | 32× / 48× |
| **The emulation thread** | **13.4** | **21× / 31×** | **30× / 45×** | **49× / 73×** |

### 3.5 What the budget asks of each item of work

Dividing each budget by the work of §3.3 gives the cost an item may have on the desktop. **One thread** uses §3.4's
allocation. **Two threads** (§4.7) gives thread A (the EE, VU0, the DMAC and the board, the IOP side) 9.0 ms for the
EE, 1.5 for VU0, 0.7 for the board, 1.2 for the IOP side and 0.5 for the remainder, and thread B (VIF1, VU1, the GIF and
its arbitration, the GS's parser) 9.0 for VU1, 2.0 for VIF1 and the GIF, 1.4 for the parser and 0.5 for its quantum
barriers, each with 0.5 spare.

| Desktop ns, factor 1.35 / factor 2 | `jak-dock` | `jak-island` | `jak-arena` |
| --- | --- | --- | --- |
| One thread: an EE instruction executed, everything in | 2.22 / 1.50 | 1.92 / 1.30 | **0.93 / 0.63** |
| One thread: a VU pair, both units, floats in | 4.67 / 3.15 | 1.99 / 1.34 | **0.90 / 0.61** |
| Two threads: an EE instruction | 4.00 / 2.70 | 3.45 / 2.33 | **1.67 / 1.13** |
| Two threads: a VU1 pair | 19.6 / 13.2 | 5.46 / 3.69 | **2.63 / 1.78** |
| Two threads: a VU0 pair | 4.81 / 3.25 | 9.03 / 6.10 | 2.64 / 1.78 |

**What the project has measured, for comparison.** Mars's recompiler: 1.1 to 2.1 ns a compiled instruction on
hand-written loops of 3 to 12 instructions with one block live (`Mars_Recompiler.md` §7), but about 70 ns an entry in
games, bound by memory: about 50 ns for the block's metadata brought in cold and 6 to 25 ns an instruction of code
streamed in, with 3,000 to 6,000 blocks live and 80 to 100 bytes of host code a guest instruction (`Mars_Recompiler.md`
§9's ring). Sapphire today: 9.2 ns a compiled instruction in the code alone and about 69 all in (§5); Jak's 15,549
blocks in 21.1 MB (§44.8), about 380 bytes a guest instruction at 3.6 instructions a block. MarsRT's RSP: about 3 ns a
step, integer only (`Mars_Native.md`). The VU's floats: §56.3 proves no host path for the adder, nor for the sum in every
multiply-add.

**What the numbers say.** The critic is right that the first version's commitment to the island contradicted its own
evidence: the island's 1.99 ns a VU pair at 1.35 is below MarsRT's integer-only RSP, and the EE's 1.92 rested on Mars's
hot-loop figure. At a factor of 2, **one thread reaches no window on any measured basis**; two threads bring the light
windows into the range of measured costs on the EE (2.33 to 2.70 ns) and leave the arena at 1.13 ns an EE instruction
and 1.78 a VU1 pair, both still below anything measured. Hence R0: the emitter's footprint (R0.2) and the VU's lanes
(R0.4) are the two numbers that decide whether the arena is reachable, and the handheld's factor (R0.3) decides which
column applies.

## 4. The design, unit by unit

### 4.0 Exactness, and the reference it is measured against

`EmuSen_FastExact.md` §1 states the sufficient condition for a batched model to give the same results as a stepped one:
every interaction between units is one of an enumerated set of observation points; at each, the observed unit is first
brought to the observer's time; ties at equal times are broken by one fixed rule; and an output that changes level (an
interrupt line) takes effect at the time it changes, not when it is next looked at. §58.6 measured what happens when the
last part is not kept: the synced build kept every observation exact and the geometry right, but from N = 512 the game
stalled. The sealed review's audit says the most compatible emulator it read is coarse in throughput but tight at its
couplings: interrupts within a few cycles, the IOP broken out when an EE event is near, VU0 at the exact cycle for
interlocked COP2 moves. **So the design batches to a horizon, not to an N** (§4.1).

**The reference.** The critic's second objection: an event loop probably cannot equal today's core, because today's
board settles several couplings at the boundaries of EE instruction steps, in a fixed unit order, which no enumeration of
observation points reproduces. In `run_to_held` the GIF and the VIFs run, then VU1 is given `path1.set_free` from the
GIF's estimate at the step's time, then the VUs catch up, their interrupts are latched at the step's end, and only then
are the kicks handed to the GIF; `Bus::read` of VU memory observes at `self.now`, the end of the previous instruction's
step; and the DMAC's wake from the IOP's side falls at the end of the EE instruction during which it happened (§44.5).
§58.3's digest drift at synced N = 64, with every observation exact, is consistent with that. **Decided 2026-10-28 by the
tester (decision 1)**: the reference becomes a **cycle-ordered model**, today's stepped board with each of those couplings
moved to its own cycle (§8, R1), its goldens re-recorded once, a timing-only change. That model is **Sapphire's cycle
face** (`EmuSen_TwoFaces.md`): never bent for speed, still graded by ps2autotests, the documents and the black-box
referees, and the player's "Accurate" option once the byte face is accepted. "Exact" in this page means identical to it. It does not mean identical to the console in
timing, which nobody has measured.

### 4.1 The scheduler: batched synchronisation with exact observations

**What changes.** `Board::run_to` after every EE instruction (or its replay at a block's exit, §44.5) is replaced by an
event loop. Each unit owns its next event as a time in EE cycles in a flat array of 20 to 30 entries with a cached
minimum, recomputed only when an entry changes (`EmuSen_FastExact.md` §7); the board's units already report their next
events (`Sapphire_Plan.md` §5.1). The loop runs the earliest unit to its event in the reference's tie order. **The
horizon** handed to the EE is the minimum of:

- every unit's next event (the PCRTC's lines and fields, the timers' targets, the DMAC's slices and ends, the SPU2's
  samples, the CDVD's sector times);
- every running unit's **lookahead**, the earliest cycle at which it could raise an interrupt line or change a value the
  EE reads without the EE first observing it: for a running VU, the fewest pairs from its PC to a pair with the E, T or D
  bit or an XGKICK, which the compiler computes for each pair of a microprogram (§4.3) and R2 computes offline from Jak's
  microprograms first, so that the horizon's length is known before R5 is built; for a VIF, its next VIFcode with the
  interrupt bit or a MARK, read from its FIFO; for the GIF, the end of the packet in transfer; for the DMAC, its next
  slice, including the slices that wait for a FIFO's room.

The IOP is not in the horizon: its effects on the EE's side are handled as §4.5 says.

**What the EE does with it.** A compiled block is entered only when its static cost fits below the horizon (one compare,
§4.2); otherwise it runs a checked form that compares at each instruction boundary, or the interpreter steps. The EE's
observations (§4.4's table) call a helper that runs the event loop to the access's exact cycle, then performs the access.
**After every helper the horizon is compared again** (the critic's fifth objection), since a run-time cost or an access
(a DMA kick, a VIF start) can move the block past it; the block leaves after that instruction when it does.

**Why it reaches its budget.** Today's board costs 21 to 51 ms a field because it is stepped 1.05 to 3.76 million times
a field (§57.2's census), and most steps find nothing (§57.2: 81 to 85 % of VU1's calls run nothing; the GIF's and VIF's
shares are nearly all checks, §4.4). At events, the count is the events' count: 1,000 to 4,000 XGKICKs (§57.2), the
VIFcodes, the DMAC's slices, 262 lines and the IOP's runs. **P-R1**: the board and scheduler at R5 under 0.52 ms a field
on the desktop on all three windows, on the reasoning that 50,000 to 100,000 events at 5 to 10 ns is 0.25 to 1 ms; R2's
census counts the events, and over 100,000 a field refutes the row before R5 is built.

**What stays exact**, against the reference: every unit's results and the times at which it is observed and raises its
lines, by §4.0. **What does not**: the board's own bookkeeping (each unit's `synced` and `horizon` clocks, the SPU2's
per-call sample, §54.9 (3)) is state that SPH2 writes today. Under the rule of two faces a save state holds machine state
only and loads under either face, so SPH2's next version is written at a quiet point (§4.10) without that bookkeeping,
which each face rebuilds after a load; the state digest is replaced by a canonical digest (§7.1).

### 4.2 The EE: long compiled blocks, a cost per block, memory through a page table, self-modifying code

**Coverage.** Blocks compile every instruction class: the integer and branch instructions of S36, the loads and stores of
S37, **the MMI's 128-bit operations as Cranelift vector operations, COP1 and COP2's macro instructions** (the float
operations inline where §4.3's lane forms are proven, called where not), COP0, SYNC, the cache instructions. A block ends
only at a control transfer it cannot follow, an instruction that raises, the end of its page, or a line the instruction
cache does not hold. **P-R2**: at R3 at least 95 % of executed instructions are compiled on the three windows (29 to 46 %
today, §58.4).

**Slow paths return into the block.** A data cache miss, an uncached load's bus wait, an access to a register or a
watched page, a COP2 interlock: each is a helper call from which the code continues. Exits remain for exceptions,
interrupts and the horizon.

**Chaining.** A block's exits to known targets jump to the target's code once it exists, each jump preceded by the
target's horizon compare; an indirect jump goes through a small table of recent targets. Chaining needs a patch step in
`emusen-jit`'s W^X batches, an optional capability of that crate (§6) that MarsRT need not take. Chaining removes the
dispatcher and its cold metadata from the steady state; it does not remove the cost of host code streamed in cold, which
is why the footprint is a pass mark of R0.2.

**The footprint.** Today's 380 bytes a guest instruction is mostly the issue model computed at run time (§48.15.6), which
the static form below removes; Mars's emitter, without it, made 80 to 100. The static form's target is **at most 96
bytes**, so that Jak's live code (about 56,000 guest instructions in 15,549 blocks today) is about 5.4 MB, a third of the
handheld's last-level cache, half the desktop's being 32 MB [A, the parts' published cache sizes]. R0.2 measures both the
bytes and the ring's cost at that size.

**The timing: static where it is static, ended where it is not.** The issue model is a chain of maximums over the
instruction sequence and the state at entry: the previous instruction's category and pipe (Table 1-3's partner rule), the
ready cycles of registers in flight (a load's latency, the multipliers' and dividers' busy cycles, HI1 and LO1), and the
bus's free cycle. So:

- **A static region** is a run of instructions whose issue cycles the compiler folds into constants for a *quiescent*
  start: every resource the region reads is ready by its first cycle and the previous category has no partner effect on
  its first instruction. The region's start tests that with a few compares on the fields concerned. Count, Random and the
  counters are written at the exit from the totals (as §44.4 already does for Count). Pairing parity at entry may split
  the entries (the critic's assumption 3); a block whose entries come in both parities gets a variant for each, which R2's
  census of distinct entry states per block prices.
- **A slow point ends the region.** A data cache miss, an uncached load's wait or a COP2 interlock adds cycles that do not
  shift every later instruction by the same amount when something is in flight: a miss of d cycles under an outstanding
  DIV or MULT overlaps the divider's latency, so later instructions move by less than d (the critic's fifth objection,
  which is right: the first version's "added to every later offset" was not exact). The region after a slow point is
  therefore a new static region with its own quiescent test at that point: where nothing is in flight the test passes and
  the folded offsets apply from there, exactly; where something is, the block continues in its dynamic form (today's
  emitter's, §44.4, compiled lazily) to its end. The seeded fault "a miss under an outstanding DIV" is R4's.
- **C1, a cost per block (a compromise, §4.8)**, only if R4 measures the static form short of its budget. **C1 must keep
  two things exact, by the sealed review's audit:** the issue timing between a COP2 macro instruction and a later read of
  its flags (games read COP2 flags in macro mode a few cycles after the operation), so COP2 macro instructions and the
  instructions up to their flag reads keep the model; and FMV's behaviour, which §7.4 tests before any C1 step.

**P-R3**: on the three windows at R4, at least 90 % of the instructions executed in compiled code run in static regions;
R2's census counts it on today's code, so the prediction is settled before R4 is built.

**Memory through a page table.** Today's translation cache of 4 KiB pages keyed by the TLB, the mode, the ASID and the
configuration (§48.3) becomes a direct table indexed by the guest's virtual page for the current mode and ASID: 2^20
entries of eight bytes, each a host base and a kind (cached, scratchpad, uncached RDRAM, accelerated, watched, slow),
rebuilt entry by entry on TLBWI, TLBWR and the mode's and Config's writes. The sealed review and its audit both favour an
inline software table over fault-and-patch under Cranelift. The table is indexed by the guest's pages and assumes nothing
of the host's (§4.9). An access is a shift, a load, a compare on the kind and the access, before the data cache's own.

**The caches stay modelled, exactly, on the default path.** A cached access compares the data cache's two tags of its
set as §48.4 does and is made in the line; a miss is a helper that refills and writes back exactly. The audit corrects the
review here: the data cache class is large (the other emulators' instant-DMA workarounds, notes on about 26 serials that a
game works on hardware only because of the cache, the BIOS's font drawing), so **C2's coherency part is not taken and not
offered**. The cost is two tag compares an access, about 1 ns on the 431 to 789 thousand accesses a field. **The
instruction cache stays modelled** too, since it is what makes self-modifying code exact.

**Self-modifying code.** A store never reaches the instruction cache (§17.3, §44.3), so a block is what the interpreter's
fetches would read as long as its lines hold the words it was compiled from. Proposed: **a generation count for each of
the instruction cache's lines**, raised by every refill and CACHE invalidation; a block records its lines' (slot,
generation) pairs and compares them at its entry and at each line it crosses. A refill whose words equal the old ones
keeps the blocks after a comparison on the slow path. No store is slowed for code. Code run uncached (kseg1 or an
uncached page) has no cache to isolate it: those pages carry a version stamp, raised by every store and DMA write into
them and checked at entry, with a count that sends a page that keeps changing to the interpreter and decays back (C12).

**The DMA hazard.** The audit widens the review's case: EE stores into RAM a channel is reading, DMA writes into RAM the
EE reads, and scratchpad DMA in both directions (one series of games' scratchpad errors is the largest group of notes on
the other emulators' DMA timing switch). With the caches modelled, a cached hit never touches RDRAM, so the EE's RDRAM
traffic is its uncached accesses, refills and write-backs. **While a channel is active, the pages it reads or writes are
watched in the page table** (for a chain, the current tag's page and the current transfer's range, updated at the DMAC's
events; the scratchpad's one entry while an SPR channel runs), and an access to a watched page brings the board along
first. Watched pages do not cover a destination the board has not yet learned: SIF0's next destination comes from a tag
the IOP supplies (the critic's assumption 2), so §4.5 keeps a check for that case.

### 4.3 The VUs: whole microprograms once the GIF runs at its own events

**What changes.** Today VU1 is brought to the EE's time after every EE instruction, 1.2 to 1.3 pairs a call (§57.2),
because the GIF's arbitration at every board step reads PATH1 from VU1's memory and VIF1 writes VU1's memory while it runs
(§57.4). With the board at events the first reason goes: VU1 is run at its observations only (the EE's accesses to its
registers and memories, VPU-STAT, FBRST, VIF1's MSCAL, FLUSH, FLUSHE and FLUSHA, the horizon, a state), as one call of
compiled code between them. The second is kept: **the other units' accesses to VU1's memory are interleaved inside VU1's
run at their cycles.** VU1's code holds the earliest cycle of any pending foreign access (VIF1's next write, the GIF's
next PATH1 read) in a register and compares it at each pair; when a pair would issue past it, a helper makes the access
and returns the next. XGKICK hands the kick to the GIF at the kick's cycle, the order within a cycle the reference's.
(The first version said "PATH1's place as the hardware reserves it"; the critic is right that this was loose, and the
audit's correction 4 says reservation at the program's start is the other emulators' approximation. The kick's cycle is
the reference's rule.)

**The lookahead** (§4.1): for each pair, the fewest pairs to an E, T or D bit or an XGKICK.

**VU0.** Macro instructions run in the EE's blocks with their issue timing (§4.2). Micro mode runs whole between the EE's
observations of VU0, and the interlocked moves (QMFC2.I, CQMFC2.I, CFC2 and CTC2 with the interlock, VCALLMS while busy)
bring VU0 to the EE's exact cycle (the review's pointer 7).

**The floats in vector lanes.** §56 proved fast paths one call at a time, and §56.3 proves no host path for the adder or
for the multiply-add's sum: 1.7 million (island) to 2.7 million (arena) lane additions a field. The critic's sixth
objection is that the VU budget therefore has no basis even on the island; it is right. The proposal is an **exact
integer form in host vector lanes**: the adder (the operands aligned by the console's guard rule, the truncating sum, the
normalisation and saturation) and the multiplier (§56.4's closed form) computed for four lanes at once in integer
arithmetic, with the MAC flags, never through the host's float rounding. R0.4 builds it as a kernel, grades it bit-equal
against `sapphire::float` (exhaustive on a reduced format, §56.5's sets on the full one) and times it; its pass mark of
1.0 ns a lane operation on the handheld is what VU1's 1.78 ns a pair (two threads, factor 2, desktop: 3.6 ns on the
handheld) allows for about one addition and one multiplication a pair.

**Flags.** Written only where a pair of the same microprogram can read them before they are overwritten, the analysis
exact within one program; always at the program's end and at every observation. Never across programs (C4 stays refused).

**Program identity.** By the audit's hint 7, games that patch a microprogram's immediates between runs recompile
constantly under today's validation (§45.4); the identity check is to exclude those words and the code to load them at
run time. R2 counts the recompiles first.

**Why it reaches its budget**, if R0.4 passes: of VU micro mode's 15.7 to 20.2 % (§57.7), the catch-up and dispatch of 2
to 9 million calls a field go with whole runs; the work, 32, 26 and 16 ns a pair today (8.0, 8.7 and 6.8 % of 228, 408
and 702 ms over 571, 1,343 and 2,955 thousand pairs), is mostly the float calls and their frames, which the lane form
replaces. **P-R5**: at R6, a VU1 pair at most 3.7 ns on the island and 1.8 on the arena (desktop), floats in.

### 4.4 The board: observation points; the GIF and VIF at events; DMA and the FIFOs on one timeline

**The observation points**, enumerated; nothing else crosses from one side to the other.

| Observer → observed | Points | What the observed side is brought to |
| --- | --- | --- |
| EE → any unit | an access to 1000_0000h–1000_FFFFh and the units' FIFOs, the GS's privileged registers (1200_0000h), VU memories through the window (1100_0000h), the SIF's registers, IOP RAM through 1C00_0000h | the board, to the access's cycle |
| EE → VU0 | COP2's interlocked moves, VCALLMS while busy, CFC2 of VU0's status, BC2F and BC2T | VU0, to the instruction's cycle |
| EE ↔ DMAC | a refill, a write-back, an uncached access or a scratchpad access to a watched page (§4.2) | the board, to the access's cycle |
| any unit → EE | an interrupt line changing (INTC, the DMAC's lines, COP0's timer) | the horizon: the EE does not pass it (§4.1) |
| VIF1, GIF → VU1 | VIF1's writes into VU1's memory, PATH1's reads of it | interleaved in VU1's run at their cycles (§4.3) |
| VU1 → GIF | XGKICK | the kick at its cycle |
| VIF1 → VU1 | MSCAL, MSCNT, FLUSH, FLUSHE, FLUSHA | VU1 to VIF1's time |
| DMAC ↔ VIF, GIF, IPU, SIF, SPR | a FIFO's room, a qword moved | one timeline, below |
| IOP ↔ EE side | the SIF's FIFOs and registers, the IOP's wake of the EE's DMAC | §4.5 |
| PCRTC → GS | CSR's field and blanking bits | the PCRTC's timing (the audit, correction 9) |

**The GIF at its own events.** The GIF is visited when something arrives (a kick, PATH2 or PATH3 data, a mask change, a
write to CTRL or MODE, a BUSDIR change) and at its own next event (a packet's end; inside an IMAGE transfer in interrupt
mode, the slices). The exact skip of the GIF and VIF paused on 2026-10-28 before its build is the first step of R5: skip
`Gif::run_to` when nothing has arrived since its last run and the time is before its resume, skip an idle VIF's `run_to`,
run unpacks in a tight loop, and keep a run at every step where a kick arrives under PATH3's slice mode. On today's island
profile the GIF's checks that find nothing are about 6 % of the thread and the packing work 0.5 %.

**The GS's observable state from the packets.** SIGNAL, FINISH and LABEL, and CSR's bits that follow from them, are
answered by the GIF's parser on the processor when it parses the packet, never by waiting for the rasteriser (the
review's pointer 2), with the audit's two fixes: CSR's field and blanking bits come from the PCRTC's timing, and a second
SIGNAL before the first is acknowledged stalls the GIF, flow control the parser models. Only a local-to-host transfer and
the field's picture wait for the drawing; R2 counts local-to-host transfers a field (the critic's assumption 4), since
each serialises the processor and the GPU.

**DMA and the FIFOs on one timeline.** A channel's data moves at its slices into the destination's FIFO, and a channel
ends when its last qword is accepted, not after an estimate; completion, room and the consumers' progress are one sequence
of events. The audit corrects the review's claim that refusing a write to a busy channel is the hardware's rule: the other
emulator's option finishes the busy transfer and then accepts the write for most channels and refuses only the IPU's,
and its authors call the rule unconfirmed; **a dispute candidate**, opened if a program shows it.

**P-R6**: at R5 the GIF and VIF together at most 0.9 ms a field on the desktop on all three windows (16 to 39 today, nearly
all per-step checks).

### 4.5 The IOP and the SPU2: in batches

§54.9 found three reasons the IOP stands at every EE instruction's time. (2), the EE's reads of the SIF's registers and IOP
RAM, become observation points: a helper brings the IOP to the EE's cycle before the read. (3) is the bookkeeping named in
§4.1. (1), the wake of the EE's DMAC, is kept: **the IOP runs at its own instruction times, at most one EE block behind**;
in the reference the wake takes effect at the IOP store's cycle (§4.0), and with the block's offsets known the EE
instruction that cycle falls in is computable. Most of the time the wake's DMAC steps land after the block's last RDRAM
access and nothing conflicts. **The case the watched pages cannot cover** (the critic's assumption 2): a SIF0 transfer
whose destination tag the IOP supplied inside the block just run, which writes a page the block then read uncached or
refilled from. The block's RDRAM-touching accesses are already logged (today's undo log, §48.5); when a DMAC step lands
inside the last block's span, its destination range is compared with the log, and on a conflict the block is rolled back
and run again as today's rollbacks do. Exact; its cost is the conflicts' count, which a test program written for it (SIF0
waiting in chain mode while the EE refills from its target) and R2's census measure.

**Not proposed: the IOP batched by a slack (C8, C7).** The audit corrects the review's "routinely thousands of cycles"
(at most about 3,000 EE cycles elsewhere, tens when either side has an interrupt due) and adds that the documented boot
race depends on the two processors' relative speed, which argues against any IOP slack during boot.

**The IOP's cost.** 6.7 to 8.9 ms today, 1 to 2 % of the frame (§57.5) but 8 to 15 times its 1.2 ms budget once the rest
falls. Batching removes the calls; the interpretation stays, about 10 ns an instruction with its bus timing.
`Sapphire_Plan.md` §5.12 measured 348,000 to 570,000 IOP instructions a frame on the menus before idle skipping (the
critic's assumption 1); the first version's 90,000 threshold is not met by that unless idle skipping passes most of them.
R2's census counts executed against passed in Jak's windows; above 90,000 executed, R7 builds a block tier for
`blackmoon-r3000` (exact, shared with Diamond).

### 4.6 The GS: the device path, designed for unified memory first

**What changes.** The drawing leaves the processor: S44's mirror of the 4 MiB local memory on a lent device (§52), then
S45's compute rasteriser held bit-exact and S46's picture on the device, as `Sapphire_Plan.md` §5.4 and §52.11 Q1
recommend: tiles each owned by one workgroup, primitives applied in submission order (`EmuSen_FastExact.md` §12's exact
choice; the sealed review describes one other emulator taking the native layout with per-pixel order). The processor
keeps the parser and the register model (§4.4) and appends primitives and register changes to the command stream of §6;
the primitives' setup moves to the device.

**Unified memory first** (decision 4). One memory abstraction for the GS's local memory, with the memory type chosen per
device:

- **On unified memory** (the handheld, most laptops, every Apple Silicon Mac): a host-visible, device-local type that the
  processor caches. A local-to-host transfer waits for the GPU's fence and then reads the bytes in place, with no copy;
  uploads (host-to-local transfers, CLUTs, the command stream) are written in place; the processor's fallbacks for rare
  GS operations (§52.6.1's cases handed back) work on the same memory.
- **On a discrete GPU** (the desktop's RX 6800, the special case): device-local memory, with staging buffers for uploads
  and readbacks.

Neither changes a result: the bytes are the same, only where they live and how they are reached differs.

**Its costs, named and measured.** The processor and GPU share the memory's bandwidth, and on battery the handheld's
memory clock falls to its lowest level after a few seconds (`Mars_Performance.md` §42.4); they share one power budget, so
a busy GPU may lower the emulation thread's boost clock; and host-visible types that the processor does not cache are slow
to read, so a readback through one would cost far more than its bytes suggest. The first two are in R0.3's factor, which is
measured with the GPU busy at the redesign's expected load; the third is R0.5, which times a 4 MB GPU write then processor
read after a fence for each memory type the handheld's driver offers. **The desktop's Radeon 610M** is a unified-memory
proxy for building and testing the path; it gives no speed numbers.

**The throughput it must have.** The device draws field n while the processor runs field n + 1, so it has about 14 ms of
the handheld's GPU a field for what four of the desktop's threads draw in 90 to 98 ms each on the dock (about 380
thread-ms), and 34 to 48 ms each on the island and the arena. P-S44b (§52.6.4) predicts the exact arithmetic alone at 0.3
to 0.6 ns a pixel on the RX 6800. The handheld's GPU has about a quarter of the RX 6800's compute-unit cycles a second at
its top clock [A, from the parts' published counts and clocks], but the critic notes it stayed at its 800 MHz floor unless
kept saturated (`Mars_Performance.md` §42.4, §42.13), which would make it nearer a thirteenth. The device path keeps the
GPU busy, which the governor rewards, but that is an expectation: R8 measures it. Jak's pixels a field in the 3D windows
are unknown (§52.6.3 counted only the menus); R2 counts them.

**The processor's GS work.** 15 to 31 ms today, including the setup and the workers' job building. **P-R7**: with setup on
the device, parsing and recording at most 0.9 ms a field on the desktop, on the 62 to 250 thousand qwords a field the
transfers move.

### 4.7 The second thread: a deterministic split at fixed quanta of emulated time

The arena is in scope (decision 2), and §3.5 shows one thread cannot carry it at a factor of 2. The EE's row cannot be
divided, so the split is of the rest. **Thread A** runs the EE, VU0, VIF0, the DMAC, the INTC, the timers, the PCRTC, the
IPU and the IOP side, with today's board form for them. **Thread B** runs VIF1, VU1, the GIF and its path arbitration, and
the GS's parser. Both run in **quanta of Q emulated EE cycles**: in quantum k each thread runs its units from kQ to
(k + 1)Q, and everything that crosses between them in quantum k is delivered at the start of quantum k + 1:

- from A to B: the DMAC's qwords for VIF1 and for PATH3, copied at the DMAC's slice times on A (RDRAM is read only on A,
  so the copy is A's and deterministic); the EE's writes to B's registers and memories;
- from B to A: the FIFOs' room, VIF1's and the GIF's interrupt lines, SIGNAL, FINISH and LABEL's interrupts, VU1's busy
  state and its T and D interrupts, and the register values the EE reads, each as B left it at the end of the previous
  quantum.

**Determinism holds by construction**: what each thread sees depends on the quantum's number, never on how far the other
thread has got, which answers the audit's objection to the one VU1 thread it describes (results depending on the host's
scheduling, failing `Sapphire_Plan.md` §5.13's first condition). **The cost** is two barriers a quantum, about 63 ns a
handoff on the desktop (§57.4): at Q = 2,048, 2,400 quanta a field and about 0.3 ms of each thread's 0.5 ms allowance.
**What it changes** is when each coupling across the split is seen, by up to 2Q: a timing-only compromise, C7 in its
deterministic form. The classes it can break are the audit's: games that must see VU1 busy or PATH3 overlap it (76 serials
need VU1 not run to completion elsewhere, 67 need it off its thread). Q is a setting, the same for every game; Q = 0 is the
byte face on one thread, digest-equal to the cycle face. A state is taken at a quantum boundary, the queues in transit part of it, so a state
loads under any Q.

**What it changes in the plan**: VIF1, the GIF and PATH arbitration have a different owner, so R5's board must be built
with that boundary from the start (B's event loop and the crossing queues), even if the split is first run with Q = 0.
That is why the tester decides it before R3. **Recommended** (§11 Q4): design R5 for the split, decide its default from
R0's numbers.

### 4.8 The compromises

Every item of §4.1 to §4.7 is exact against the reference unless listed here. Timing-only compromises change when
something happens and no byte a game reads where the game does not depend on that timing; byte-changing ones change a
value a game reads or a pixel or sample the player gets (`Sapphire_Plan.md` §5.13).

| # | Compromise | Kind | Default | When |
| --- | --- | --- | --- | --- |
| C7 (split) | Thread B (VIF1, VU1, the GIF, the parser) beside thread A at fixed quanta, every crossing at a quantum boundary (§4.7) | timing-only, deterministic | **decided before R3** (§11 Q4) | built at R7, its boundary at R5 |
| C1 | A cost per block instead of the issue model, COP2 macro instructions and their flag reads kept on the model | timing-only | **off**; proposed only if R4's static form misses its budget, after FMV is measured | R4 |
| C8, C7 (IOP) | The IOP run with a bounded slack | timing-only | **not proposed** (§4.5) | never during boot |
| C2 (coherency) | The data cache's coherency dropped | byte-changing | **not offered** | — |
| C3 | Floats on the host's arithmetic with clamping | byte-changing | off, an option | unchanged |
| C4 | VU flags across programs | byte-changing | refused (§45.9) | — |
| C5 | The host's rasterisation and blending | byte-changing | off, an option | S46 at the earliest |
| C6 | Render targets on the device with guessed reinterpretations | byte-changing | not proposed | — |
| C9 | The CDVD's delays shortened, short reads and seeks only, long seeks kept (the audit's hint 5) | a loading screen's length | off, an option | with the scoreboard's loading |
| C12 | Uncached code pages that keep changing sent to the interpreter | none (speed only) | on | R4 |
| C13 | The picture presented a field late on the device | none in emulation | decided at S46 | S46 |

**Out of the box, beside them**: deinterlacing chosen from the registers with an override, and the emulated field rate
locked to a 60 Hz panel with the sound resampled (the audit's hint 8). Both are the frontend's and change no result.

### 4.9 Portability the design must not close off

Constraints, not work now: the Mac's Vulkan work is paused (2026-10-16), and the platform scope is Linux first.

- **Host pages of 16 KiB** (Apple Silicon, ARM Linux). The EE's page table is indexed by the guest's 4 KiB pages and holds
  host addresses, so it needs nothing of the host's page size; nothing in the design maps guest pages onto host pages.
  Anything that protects or maps host memory (the code cache's W^X batches, a guard page) works in the host's page size,
  read at run time, never assumed to be 4 KiB.
- **aarch64 and W^X.** The recompilers emit through Cranelift, which targets aarch64. On macOS, JIT pages are MAP_JIT
  with write protection toggled per thread; `emusen-jit`'s W^X batches, and the chaining patch step (§4.2), go through one
  place that can switch between that and Linux's mprotect, so neither the EE's emitter nor the VU's assumes either.
- **Tile-based GPUs** (Apple's, most Android devices'). The compute rasteriser orders pixels by construction, one
  workgroup owning a tile and applying primitives in order, so it needs from such a GPU only compute shaders, storage
  buffers of 4 MiB and, for `exact64`, 64-bit integers (`exact32` otherwise, §52.6.1). It needs neither fragment shader
  interlock nor Metal's raster order groups. Those would matter only for a graphics-pipeline path (C5), which would need
  `VK_EXT_fragment_shader_interlock` where offered or raster order groups through Metal, and has no exact fallback
  without them.

### 4.10 Sapphire's two faces: what each unit shares, and what differs

Under `EmuSen_TwoFaces.md`, the redesign is not a second core: each unit's semantics are written once and its schedule
and execution strategy are the face. The machine is generic over a face for each unit (`Machine<Ee, Vu, Board>`, each
`Cycle` or `Byte`), so a build compiles each face's hot paths alone, no instruction tests a face at run time, and the
differential tests can put one unit on its byte form while the rest stay on the cycle face. A build the player gets has
one face for the whole machine, chosen when the game starts.

| Unit | Shared by both faces (the semantics layer) | Cycle face | Byte face |
| --- | --- | --- | --- |
| EE | `emusen-mips`'s decode; `ee::ops`' effects; the issue model's rules; COP0, the TLB and both caches' models; `sapphire::float` | the interpreter, each instruction timed by the issue model and the board run to its cycle | chained compiled blocks over every class; static regions; the page table; line generations (§4.2) |
| VU0, VU1 | the decode, `vu::ops`' effects, the pipelines' hazard rules, the flags' landing rules, `sapphire::float` | `Pipe` stepped to each observer's cycle | whole runs between observations, the foreign-access agenda, lane forms, flags within a program (§4.3) |
| The board | each unit's register semantics, FIFOs, the DMAC's tags and channels, the INTC's lines, the clock's ratios (`clock.rs`) | every event in cycle order, each coupling at its own cycle (R1 (a)) | the event loop batched to the horizon; the split at fixed quanta if taken (§4.1, §4.7) |
| VIF, GIF | the VIFcodes, unpacks, the GIF's tags, paths and arbitration rules, PATH3's masking | stepped with the board | at their own events, VIF1's writes and PATH1's reads interleaved in VU1's run (§4.4) |
| GS | the register file, the privileged registers, CSR, SIGNAL, FINISH and LABEL, the local memory's layout, the pixel pipeline's arithmetic (`gs::exact`) | the software reference drawing as the packets arrive | the parser and the command stream, the drawing on the device (§4.6) |
| IOP side | `blackmoon-r3000`, `blackmoon-spu`, the CDVD, SIO2, the SIF's semantics | brought to every event in cycle order | at its own instruction times, a block behind, with §4.5's SIF0 check |

**Per-unit differential tests.** Each byte form is first run with every other unit on the cycle face, and its canonical
digests compared with the all-cycle machine's at each field's end: the EE's blocks on the cycle board (R3 and R4 do this,
since the replay keeps the cycle board's calls), the event board with the cycle EE interpreting, VU1's whole runs on the
cycle board's observation points. Where one unit's byte form needs another's (VU1's whole runs need a board that does
not step it every instruction), the pair is tested together and named.

**Safe points.** A quiet point is a field's end with every unit caught up, no crossing in a queue of the split and no DMA
slice half moved; states are written there, and a change of face happens only through a state or at such a point.

**Acceptance.** Where the byte face is digest-equal to the cycle face (R3 to R6, the IOP, the GS's parser), the rule's
acceptance holds trivially. Where it moves timing (the split, any compromise of §4.8), §7.1 applies the rule's test.

## 5. Why the recompiled tier is slower than the interpreter today

§58.4: 234.1 and 242.5 ms against the interpreter tier's 229.6 on the dock, 413.7 and 433.8 against 403.3 on the island,
714.6 and 744.1 against 692.9 on the arena. §58.5's figures explain it: a compiled instruction's code costs 7.4 to 9.2 ns,
the dispatcher, exits and replay about 60 ns a compiled step (§58.7), and an interpreted instruction about 40 ns before
the dispatcher's share. **A compiled instruction therefore costs about 69 ns all in against about 40 for an interpreted
one** (the 60 ns includes the replay's board work, which the interpreter tier pays in the board's own row; the shares are
sampled at synced N = 64 and the counts are N = 1's). The likely causes, each from a measurement:

| # | Cause | Evidence | Removed by |
| --- | --- | --- | --- |
| 1 | **The replay keeps the board per instruction**: a block moves the board's calls to its exit and makes them in order there, the IOP caught up at each time (§44.5, §48.5) | "EE: dispatcher, exits and the replay's per-step work" 41.5 / 44.3 / 81.9 ms (§58.4), against compiled code 6.4 / — / 10.0 | the board at events (§4.1) |
| 2 | **Short blocks and constant switching between tiers**: every MMI, COP1 and COP2 instruction ends a block; 44 to 65 % of loads and stores leave; 28 to 65 % of the rest run interpreted between an exit and the next entry; each switch pays the lookup, the entry checks (§44.3) and the state through memory | 54 to 71 % of instructions interpreted on the recompiled tier (§58.4) | full coverage, slow paths that return (§4.2) |
| 3 | **The issue model computed at run time in the code**, most of a block's bytes (§48.15.6) | the timing model 13.8 / 12.1 / 26.8 ms; 380 bytes a guest instruction | the static regions (§4.2) |
| 4 | **Cold code**: 15,549 blocks in 21.1 MB on Jak (§44.8), past the last-level cache's share a thread gets | Mars's ring: 6 to 25 ns an instruction streamed in cold (`Mars_Recompiler.md` §9) | the static form's footprint (R0.2) |
| 5 | **Rollbacks**: a replay stopped by an interrupt or the DMAC undoes the block and runs it again (§48.5) | 23,465 rollbacks over Jak's first 400 frames | the horizon; watched pages; §4.5's narrow check |
| 6 | **The dispatcher between every pair of blocks**, with cold metadata | inside row 1 (§55 to §57 count the dispatcher and the replay together, 5.6 to 10.9 %); Mars's about 50 ns an entry cold | chaining (§4.2) |

How the 60 ns divides between causes 1, 2, 4 and 6 is not settled; R2's census counts entries, block lengths and the
replay's calls, and R3's gate is the recompiled tier beating the interpreter tier.

## 6. Modularity: the components and their surfaces after the redesign

Each component has one job; what crosses a boundary is listed, and nothing else does.

| Component | Its one job | Surface (version) | What crosses it |
| --- | --- | --- | --- |
| The core ABI | the frontend's view of Sapphire | core ABI v1, unchanged | frames, sound, input, states, settings, the lent device |
| `sapphire::face` | which face each unit is compiled for (§4.10) | a compile-time parameter of `Machine`, one per unit for the differential tests, one for the whole machine in a build the player gets | nothing at run time |
| `sapphire::board` | which unit runs next, to when; the horizon; with the split, one loop a thread and the crossing queues | `Unit`: `next_event`, `lookahead`, `run_to`; versioned with SPH2 | times in EE cycles; the queues' qwords, writes and lines at quantum boundaries |
| `sapphire::ee` (interpreter, `ee::jit`) | the R5900 and its caches | `Ee::run(horizon) -> Exit`; I/O by `Bus::access(addr, cycle)`; the page table it owns | cycles, accesses, exits |
| `sapphire::vu` (both units, `vu::jit`) | VU0 and VU1 | `Micro`: `run_to`, `lookahead`, the foreign-access agenda (§4.3) | cycles, kicks, accesses |
| `sapphire::vif`, `gif`, `dmac`, `ipu`, `intc`, `timers`, `pcrtc` | each its unit | `Unit` | FIFO qwords, lines, events |
| `sapphire::gs::parse` | the GS's registers, privileged registers, CSR, SIGNAL, FINISH, LABEL | the command stream, plain data (versioned, below) | the stream out; CSR's timing bits from the PCRTC in |
| `sapphire::gs::draw` (`gs::device::Path`) | drawing: the software reference or the device | consumes the stream; local-to-host reads answered; the memory type chosen per device (§4.6) | the stream in, pixels and readbacks out |
| `sapphire::float` | the console's floats, exactly | functions, their proven fast paths, and the lane forms (§4.3) | values |
| The IOP side: `blackmoon-r3000`, `blackmoon-spu`, the CDVD, SIO2 | the second computer | the crates' surfaces, unchanged; the SIF as a `Unit` | SIF qwords, registers, the wake |
| `emusen-jit` | a recompiler's thread, publication, W^X, budget, verifier | its crate surface, **a minor version adding chaining's patch step as an optional capability**, its W^X through one switchable place (§4.9) | compiled functions, patches |
| `emusen-gpu` | one Vulkan compute device | its crate surface; the lend's features word (§52.11 Q2); memory types queried, not assumed | buffers, programs, submissions |

**The command stream is the new surface to version first**: the GS's parser and its drawing meet only there, and it is
the format of the audit's hint 1 (§7.3): the GS's registers and local memory at the start, then the parser's output with
field marks and the PCRTC's state at each field (merge and feedback output depend on the previous field's). With the split
it is also what thread B hands the device.

**Tools.** Every tool the stages add is a DianaSh command written in Rust, plain text or data in and out: the scoreboard
(`ps2score`, §7.4), the canonical digest (§7.1, a field of the bench's output), the census (R2), and the GS stream's record
and replay (`gsreplay`, §7.3). The R0 kernels are the bench's examples. The names are proposals.

## 7. Verification

### 7.1 Each stage proves its results unchanged

**The reference** is the cycle-ordered model of R1 (§4.0), its goldens re-recorded once there. Its own evidence, since it
moves timing against today's core: ps2autotests with no new failure (52 of 55 EE texts with the same three failing, 23 of
23 IOP), every timing test of §58.6's list examined, the boot's and Jak's pictures by eye at their anchors, and Jak's
DISPFB counts in their ranges.

**Every later exact stage** (R3 to R6, the exact parts of R7) is graded against the reference: the boot's 760 frames and
Jak's 900, the three windows' 300; ps2autotests' 78 texts byte for byte the reference's; the verifier stepping the
interpreter beside every block; random programs as at S36 and S37; picture and sound digests equal frame by frame; and a
**canonical digest** in place of the state digest: at the field's end, the board caught up, a digest of everything a
program can read (RDRAM, the scratchpad, both caches' lines and tags, every processor's registers, VU memories, the GS's
local memory and registers, IOP RAM, SPU2 RAM, the units' architectural registers and FIFO contents), excluding the board's
own clocks (`EmuSen_FastExact.md` §10's canonical serialisation). R2 shows it equal across tiers on the reference before
any stage depends on it.

**Where timing moves** (the split, or a compromise of §4.8), digests change, and the byte face is held to
`EmuSen_TwoFaces.md` §5's acceptance: every test program's output identical to the cycle face's; the canonical state
identical at each field's end or the next input poll, aligned for the timing shift; every scoreboard game's frames and
sound in the same sequence within a stated phase shift (proposed for the split: 2Q cycles, at most one field); anything
else a named exception, never fixed for one game. The GS streams of §7.3 support the picture comparison. **Per unit**,
the differential test runs one unit on its byte form with the rest on the cycle face (§4.10). With the split, every run at a given Q is also
run twice and on both machines, its digests equal each time, which is the determinism test.

### 7.2 Seeded faults

Three to seven a stage, aimed at its exactness argument: an observation point left out of §4.4's table, a horizon one
cycle long, a tie broken the other way, a line generation not raised on a refill, a watched page not watched, a quiescent
test that passes a busy multiplier, **a data cache miss under an outstanding DIV**, a SIF0 write into a page the block read
uncached, a crossing delivered a quantum early.

### 7.3 GS streams, recorded and replayed

The audit's first hint: a corpus of recorded GS input (the start state and the stream of §6, with field marks), replayed
headless, each field's picture digest compared with the software reference. It grades the device path and any of C5 field
by field without the whole machine, and separates a GS defect from a CPU-side timing change. The corpus comes from
Sapphire's own runs of the tester's games, kept out of the repository with the discs; synthetic streams go in the tests.

### 7.4 The compatibility scoreboard

**Each of the handheld's PlayStation 2 games, run headless** (the probe's discs under
`~/.cache/emusen/probe/sapphire/discs/`, never off the NAS), at three levels: **boots** (the game's own first picture),
**reaches gameplay** (a press script to a gameplay anchor), **looks right** (pictures at anchors against the reference,
then by eye on the first pass), and on the handheld **its speed** (mean and 95th percentile ms a field on battery). One
plain-text row a game from `ps2score`. The audit's hint 2: no other emulator runs its library automatically; the games are
best chosen to cover the classes that break elsewhere (DMA completion, PATH1 and PATH3, the VIF and GIF FIFOs, VU0's macro
flags, floats, data cache coherency, FMV, the CDVD). **FMV is in every stage's check from R3**, as the audit's test of the
EE's timing. R0.1 runs on whatever part of the scoreboard exists then. The list is the tester's (§11 Q2).

## 8. Stages, in order

Sizes are in lines of code and steps of the kind §55 to §58 were; "buys" is the prediction it is gated on. R0 builds
nothing that stays except, if it passes, R0.4's kernel (which becomes R6's lane form). R8 and R9 need the tester present
for the GPU.

| Stage | What | Gate (measurable) | Size | Buys |
| --- | --- | --- | --- | --- |
| **R0** | **Five risk checks** (§2): R0.1 the busy-scene census, counting patches outside the repository; R0.2 the ring benchmark of Sapphire's emitter at Jak's live-block count, with the static form's bytes by a prototype emitter switch, on the desktop and the handheld; R0.3 the handheld's factor on battery with one hot thread (the GS's workers at one) and a GPU kernel (S44's `exact64` in a loop) busy at the expected load, after the clocks settle, reading the memory and fabric clocks; R0.4 the VU's adder and multiplier with MAC flags in vector lanes, graded and timed on both machines; R0.5 the memory types' 4 MB readback and uploads on the handheld | §2's pass marks; R0.2, R0.3 and R0.4 all pass at R0.1's design point, or the plan stops | S to M: about 1,500 lines, mostly R0.4's kernel and its tests; two handheld sessions | whether the plan is reachable, before 13,000 lines |
| **R1** | **The cycle-ordered reference, then batching tested strictly against it.** (a) Today's board with each step-quantised coupling moved to its cycle: VU memory read by `Bus::read` at the access's cycle; the DMAC's wake at the IOP store's cycle; VU interrupts at the T and D bits' cycles; the kick handed to the GIF and `set_free`'s estimate at the kick's cycle; the VIF's and GIF's latches at theirs; any other the code reading finds; goldens re-recorded once (§7.1). (b) Throwaway batched builds on (a) with **every candidate fixed** (the lines at their cycles, VU1's end and XGKICK at VIF1's time, the DMAC's wakes for FIFO room, the IOP's and SIF's wakes, idle skipping passing only cycles the board has run) at N = 2, 4, 16 and 64, then each candidate removed in turn | **Strict**: canonical, picture and sound digests of (b) equal to (a)'s at N = 2, 4 and 16 on the boot, Jak's 900 and the three windows, and ps2autotests' texts equal; **failing stops the plan**. N = 64 and the removals recorded, a difference at 64 alone naming a horizon rule R5 must have | M: (a) about 600 lines that stay, (b) throwaway | the reference; the test of §4.0's claim, on interacting causes rather than one at a time |
| **R2** | **The instruments**: the canonical digest; a census build counting a field the EE's observations by class, block entries and lengths, static-region coverage and distinct entry states per block, the replay's calls, watched-page accesses, SIF0 conflicts, the board's events, VIF1 writes into a running VU1, VU recompiles, IOP instructions executed and passed, local-to-host transfers, Jak's pixels drawn; **VU1's lookahead computed offline** from Jak's microprograms; `ps2score` on the desktop | the digest equal across tiers on the reference; P-R1, P-R3 and §4.5's IOP count settled; the horizon's distribution recorded | M: about 900 lines | numbers for the decision and for R3 to R7 |
| **Decision** | **The split** (§4.7, §11 Q4), the tester's, on R0's and R2's numbers | before R3 | — | R5's ownership fixed |
| **R3** | **The EE recompiler I: coverage.** MMI in vector operations, COP1 and COP2 macro in blocks, loads and stores kept, slow paths returning, the data cache's refill as a helper; the replay kept, so timing is the reference's | digests and the verifier equal to the reference; at least 95 % compiled (P-R2); **the recompiled tier faster than the interpreter tier** on all three windows, two rounds | L: about 3,500 lines | the interpreter's 47 / 47 / 88 ms under 10 / 10 / 20; the windows at about 170–195 / 330–370 / 560–620 (P-R8) |
| **R4** | **The EE recompiler II: blocks.** The page table, chaining, line generations and uncached stamps, static regions with their quiescent tests ended at slow points, the dynamic form for the rest | digests equal; R0.2's ns an instruction at the measured footprint met on the windows' blocks; P-R3 | L: about 2,500 lines | the timing model's 14–27 ms under 2 |
| **R5** | **The board at events**, built with the split's boundary if decided: the GIF's and VIF's exact skip first, then the event loop, the horizon with lookahead, observations at exact offsets with the horizon re-checked after each helper, watched pages and §4.5's SIF0 check, the replay retired, the IOP at its own times, DMA and FIFOs on one timeline, the parser answering SIGNAL, FINISH, LABEL and CSR | canonical digests equal to the reference at Q = 0 on the boot, Jak and the windows, N = 64 included; ps2autotests unchanged; P-R1 and P-R6 | XL: about 4,500 lines | the board's 21 / 36 / 51 ms, the GIF's and VIF's 16 / 27 / 39 and most of the replay: the windows at about 70 / 140 / 205 ms (P-R9) |
| **R6** | **The VUs whole.** Runs between observations, the foreign-access agenda, the lookahead, flags within a program, R0.4's lane forms, immediates out of the identity if R2 found recompiles | the verifier beside every pair; digests equal; P-R5 | L: about 2,500 lines | the VUs' 35 / 85 / 141 ms to about 3 / 6 / 10; the windows at about 40 / 65 / 75 (P-R10) |
| **R7** | **The second thread and the rest of the thread.** The split's thread B at Q > 0 if decided; the parser to parse and record only (P-R7); the IOP's block tier if R2 asks for it | Q = 0 equal to the reference; at the chosen Q, determinism (§7.1) and the scoreboard at every level it reached at Q = 0; each thread under its §3.5 allowance at the measured factor (P-R11) | L: about 2,500 lines | the arena's VU1 off thread A |
| **R8** | **The GPU window** (the tester present): P-S44b on the RX 6800 and the handheld, the device's ns a pixel on Jak's profile, the GPU's clock under the device path's load; the unified-memory path built on the 610M | P-S44b retired on both; the handheld's clock and ns a pixel recorded | S: about 40 minutes a device | whether S45 stays bit-exact on the device |
| **R9** | **The GS on the device**, S45 and S46, unified memory first (§4.6) | pictures bit-equal to the software reference on the boot, Jak and the GS streams; the device at most 14 ms a field on the handheld on the windows | XL | the GS workers' 90–98 ms off the processor |
| **R10** | **The handheld scoreboard**, on battery; any compromise of §4.8 taken only here, measured and named | the scoreboard's games at 13.4 ms mean and under 16.7 at the 95th percentile at R0.1's design point; every level each game reached kept | M | the first scoreboard |

**P-R12**: if R0 passes, R3 to R7 put every window within the handheld's budget on both threads at the measured factor;
if R0.2 or R0.4 passes only at a factor of 1.35, the arena is named short at a factor of 2 with its number.

**Order.** R0, then R1, then R2 and the decision; R3, R4 and R5 in order; R6 after R5; R8 whenever the tester and the
machine are free, before R9.

## 9. The review, answered

| Objection | Answer | Where |
| --- | --- | --- |
| 1. The committed scope is the light scenes | Accepted. Busy scenes are in scope (decision 2); R0.1 measures how common they are; the split is designed now (§4.7) and decided before R3 | §2, §3.3, §4.7 |
| 2. "Exact against today's core" is not reachable by an event loop | Accepted. The reference becomes a cycle-ordered model (decision 1), built at R1 with the step-quantised couplings named | §4.0, R1 |
| 3. The EE budget rests on a hot-loop microbenchmark | Accepted on the method: R0.2 is Mars §9's ring at Jak's live-block count. **Partly disputed on the projection**: Mars's 70 ns an entry is a dispatcher visited at every entry with cold metadata, which chaining removes, and most of today's 380 bytes is the run-time issue model, which the static form removes; so R0.2 must time the static form's bytes, not today's | §3.5, §4.2, R0.2 |
| 4. The handheld factor is nearer 2 | Accepted: budgeted at 2 (§3.4, §3.5); R0.3 measures it under the redesign's load mix | §3.1, R0.3 |
| 5. Run-time terms are not exact | Accepted. Static regions end at slow points; the horizon is re-checked after every helper; the seeded fault is R4's | §4.1, §4.2, §7.2 |
| 6. The VU budget has no basis | Accepted. The lane form is an exact integer form, R0.4 grades and times it, and a pass mark gates the plan | §4.3, R0.4 |
| 7. R1 cannot decide what it claims | Accepted: every candidate fixed and removed in turn, the DMAC's room wakes and the IOP's and SIF's added, digest equality the pass, the escape clause gone, VU lookahead offline in R2 | R1, R2 |
| Assumption: IOP under 90,000 a field | Open; §5.12's 348,000 to 570,000 before idle skipping make it doubtful; R2 counts it, R7 has the block tier | §4.5 |
| Assumption: watched pages cover SIF0 | They do not; §4.5's check against the block's access log covers it, with a test program | §4.5, §7.2 |
| Assumption: 90 % quiescent | Open; entry parity may halve it; variants by parity, R2 counts distinct entry states | §4.2 |
| Assumption: local-to-host is rare | Open; R2 counts it, R0.5 prices it on unified memory | §4.4, §4.6 |

**Where this page disagrees with the review.** Two points. First, objection 1's "busy gameplay in most of the library
will look like the arena" rests on one game; this page measures it (R0.1) rather than assume it either way, and the
decision does not depend on the answer, since the tester put busy scenes in scope. Second, objection 1 also says C1
"removes only timing, which the static form already claims to remove": the static form removes the run-time model only in
quiescent regions, and C1 would also remove the dynamic form, the region breaks at slow points and the parity variants, so
C1 is still a lever on the EE's row (a timing-only one, kept for the case R4 fails), though not one that closes the
arena alone. The rest of the review is taken as written.

## 10. Risks, unknowns and weak points

| # | Risk or unknown | The cheapest check | When |
| --- | --- | --- | --- |
| 1 | The handheld's factor exceeds 2 under the redesign's load | R0.3 | R0 |
| 2 | The emitter's footprint makes cold code dominate | R0.2 | R0 |
| 3 | The VU's lane form misses 1 ns a lane | R0.4 | R0 |
| 4 | Busy fields are busier than the arena | R0.1 | R0 |
| 5 | Unified memory has no processor-cached device type on the handheld's driver | R0.5 | R0 |
| 6 | Batching cannot equal the cycle-ordered reference | R1's gate | R1 |
| 7 | The horizon is short (frequent XGKICKs or T bits, short DMA slices) | R2's offline lookahead and the census | R2 |
| 8 | The IOP's interpretation exceeds its budget | R2's count | R2 |
| 9 | SIF0 conflicts are frequent enough to cost | a test program, then R2's count | R2 |
| 10 | Few quiescent regions | R2's count of entry states | R2 |
| 11 | The split breaks scoreboard games at any Q that pays | R7's sweep of Q against the scoreboard | R7 |
| 12 | The GS's device throughput on the handheld, at whatever clock its governor gives | R8 | R8 |
| 13 | Chaining in `emusen-jit` disturbs MarsRT | MarsRT's tests in the blast radius of R4 | R4 |
| 14 | Disputes the redesign reaches: the busy-channel write (the audit's correction 6); a TLB entry mapping nothing, which the audit says one demo relies on | programs written for each, then the referees as black boxes | when a program shows it |
| 15 | The scoreboard is three games today | the tester's list | before R0.1 |

**The weak points, stated before the critic does again.**

1. **The arena is not shown reachable on two threads at a factor of 2**: 1.13 ns an EE instruction and 1.78 a VU1 pair
   are below anything measured; R0 decides, and the plan stops if it fails.
2. **The budget tables are allocations**; only the thread's product (21 to 73 times) is a fact.
3. **The split is a compromise**, deterministic but timing-only on the couplings most known to break games elsewhere.
4. **Exactness rests on enumeration**; one coupling missing from §4.4 is a silent difference wherever the digests do not
   look.
5. **"Exact" is against a reference model**, now re-recorded once; if the model is wrong somewhere, the redesign keeps it
   wrong, exactly.
6. **The sealed review and its audit are someone's reading of others' code**; the classes they name choose tests, not
   designs' evidence.
7. **R5 cannot be split much further**: only its first step, the GIF's and VIF's skip, is separable.
8. **R0's kernels are not the system**: a ring and a lane kernel bound the costs from below; the census's design point
   and the stages' own gates are where they are met or not.

## 11. Questions for the tester

1. **[decision] Approve R0, and the plan as revised, with R0's stop rule** (§2): continue to R1 only if R0.2, R0.3 and R0.4
   pass at R0.1's design point. Recommended: yes.
2. **[decision] The scoreboard's games**, needed for R0.1. Recommended: the PlayStation 2 games installed on the handheld,
   plus Jak and Daxter, Kingdom Hearts and Dark Cloud 2, copied from the NAS into the probe's disc folder; and on the
   audit's offer, a reader of the sealed sources may pass on which titles stress each coupling class (§7.4), titles only,
   recorded as exposure.
3. **[decision] C1.** Recommended: not now; only if R4 measures the static form short, in §4.2's form, after FMV is
   measured.
4. **[decision] The split, before R3** (decision 2, §4.7). Recommended: design R5 with the split's boundary whatever is
   decided, since it costs little at Q = 0; make the split the default only if R0's numbers say one thread cannot carry
   R0.1's design point at the measured factor, at the smallest Q that pays, the same for every game; the cycle face stays
   the reference and the Accurate option.
5. **[decision] §5.13's second condition and "no fixes for one game".** Recommended: amend "narrowed or turned off for the
   unit or game involved" to "for the unit involved, for every game".
6. **[routine] The handheld for R0.3, R0.2's and R0.4's handheld runs, and R0.5.** Recommended: one session when the
   tester next has it free, on battery, under `systemd-run --user`.

## 12. What was read for this proposal, and exposure

`Sapphire_Speed.md` §55 to §58; `Sapphire_Plan.md` §1.4, §5.1, §5.12 and §5.13; `Sapphire_Native.md` §44.1 to §44.6,
§44.8's block count and §44.9, §45.1 and §45.3, §52 (S44, from its branch's page); `Sapphire_Recompiler.md` §48.1, §48.4
to §48.6 and §48.15.1; `Sapphire_Scheduler.md` §54.1 and §54.9; `Mars_Recompiler.md` §9's ring; `Mars_Performance.md`
§42.4; the paused plan for the GIF's and VIF's exact skip (2026-10-28); `EmuSen_FastExact.md` from branch
`sim-reference`; the sealed review, the sealed review's audit, and the critic's review of the first version. No emulator's
source, no firmware, no game data. Nothing was built or run for this page.
