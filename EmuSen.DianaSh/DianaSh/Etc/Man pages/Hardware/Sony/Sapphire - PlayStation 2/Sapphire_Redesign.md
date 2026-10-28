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
| **R0.3** The handheld's factor | the emulation thread's field time on the handheld on the charger against the desktop (on battery until battery runs were dropped, decided 2026-10-28), with one hot thread and the GPU busy at the redesign's expected load, after the clocks settle | **at most 2.0**; the tables are then redrawn at the measured factor |
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

**Its costs, named and measured.** The processor and GPU share the memory's bandwidth, and in MarsRT's runs the handheld's
memory clock fell to its lowest level after a few seconds (`Mars_Performance.md` §42.4); they share one power budget, so
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
then by eye on the first pass), and on the handheld **its speed** (mean and 95th percentile ms a field on the charger; battery runs dropped, decided 2026-10-28). One
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
| **R0** | **Five risk checks** (§2): R0.1 the busy-scene census, counting patches outside the repository; R0.2 the ring benchmark of Sapphire's emitter at Jak's live-block count, with the static form's bytes by a prototype emitter switch, on the desktop and the handheld; R0.3 the handheld's factor on the charger (battery runs dropped, decided 2026-10-28) with one hot thread (the GS's workers at one) and a GPU kernel (S44's `exact64` in a loop) busy at the expected load, after the clocks settle, reading the memory and fabric clocks; R0.4 the VU's adder and multiplier with MAC flags in vector lanes, graded and timed on both machines; R0.5 the memory types' 4 MB readback and uploads on the handheld | §2's pass marks; R0.2, R0.3 and R0.4 all pass at R0.1's design point, or the plan stops | S to M: about 1,500 lines, mostly R0.4's kernel and its tests; two handheld sessions | whether the plan is reachable, before 13,000 lines |
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
| **R10** | **The handheld scoreboard**, on the charger (battery runs dropped, decided 2026-10-28); any compromise of §4.8 taken only here, measured and named | the scoreboard's games at 13.4 ms mean and under 16.7 at the 95th percentile at R0.1's design point; every level each game reached kept | M | the first scoreboard |

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
   tester next has it free, under `systemd-run --user`; on the charger since battery runs were dropped (decided
   2026-10-28).

## 12. What was read for this proposal, and exposure

`Sapphire_Speed.md` §55 to §58; `Sapphire_Plan.md` §1.4, §5.1, §5.12 and §5.13; `Sapphire_Native.md` §44.1 to §44.6,
§44.8's block count and §44.9, §45.1 and §45.3, §52 (S44, from its branch's page); `Sapphire_Recompiler.md` §48.1, §48.4
to §48.6 and §48.15.1; `Sapphire_Scheduler.md` §54.1 and §54.9; `Mars_Recompiler.md` §9's ring; `Mars_Performance.md`
§42.4; the paused plan for the GIF's and VIF's exact skip (2026-10-28); `EmuSen_FastExact.md` from branch
`sim-reference`; the sealed review, the sealed review's audit, and the critic's review of the first version. No emulator's
source, no firmware, no game data. Nothing was built or run for this page.

## R0 results (measured 2026-10-28)

*Measured on the desktop (Ryzen 7 7700X) only; the handheld's session (R0.3, R0.5 and the handheld's runs of R0.2 and
R0.4) was held by the tester's decision of the same day, since R0.2 and R0.4 had already failed on the desktop. Disputes
D-1100 and D-1101. The instruments are committed and off by default: the bench's `--census FILE` (one row a field),
`examples/ring.rs`, `src/float_lanes.rs` behind the `lanes` feature with `examples/lanes.rs`, and `emusen-gpu`'s
`examples/memtypes.rs`. The static form's byte counter was a throwaway patch (a shadow compile of every block in three
forms), kept in the stage's scratch and reverted. Every timing ran alone at CPU weight 100. Exposure: the census and the
byte counter ran the player's BIOS (market americas) and Jak and Daxter, which makes this stage a player's-BIOS stage
(exposed, never a replacement writer). No firmware was listed, hashed or read, no program counter or code of the BIOS
or the game was logged, and no emulator's source was read.*

### The verdict

**Stop.** At R0.1's design point, R0.2 and R0.4 both fail on the desktop, and the handheld's factor (R0.3) cannot be
below 1, so it can only widen both gaps. By §2 the plan does not continue to R1 and returns to the tester with the
numbers below. R0.3 and R0.5 were not run.

| Check | Measured (desktop) | Pass mark | Result |
| --- | --- | --- | --- |
| R0.1 the design point | the 90th-percentile field: **4.92 M** EE instructions executed, **2.72 M** VU pairs (VU1 2.33 M) | tables hold at or below 3.99 M and 2.96 M | **redrawn**: 1.23 times the arena's instructions; the pairs below the mark's 2.96 M but 1.42 times the arena's 1.92 M on the same counter (D-1100) |
| R0.2 bytes | today's emitter **1,208**, C1 **924**, the static prototype **568** host bytes a guest instruction | at most 96 | **fail**, 5.9 times the mark |
| R0.2 the ring | **2.19 ns** a guest instruction at 15,549 blocks and 98 bytes; **56.0 ns** at 585 bytes | the EE's allowance at the design point: 0.92 ns (two threads, factor 2), 1.36 (factor 1.35) | **fail**, 2.4 and 1.6 times; 61 times at the prototype's bytes |
| R0.3 the handheld's factor | not run; the desktop's denominator is recorded below | at most 2.0 | **not run** |
| R0.4 the VU's lanes | bit-equal on every case graded; **1.06 ns** a lane addition, **0.86** a lane product | 0.5 ns on the desktop (1.0 on the handheld) | **fail**, 2.1 and 1.7 times |
| R0.5 unified memory | not run on the handheld | a cached device-local type reads 4 MB within 1 ms | **not run** |

### R0.1, the busy-scene census

The bench's census, with `--defaults` from the state at 6,000, wrote one row for each of 4,800 fields, 6,001 to 10,800.
The desktop reset itself at frame 10,570 of the first run (cause unknown: no OOM, MCE or GPU error in the log), so the
tail ran again from the state at 10,531. On the 39 fields the two runs share, the totals agree exactly: the EE's
instructions, the cycles, both VUs' pairs. The split between executed and passed moves by up to 1,104 instructions in 18
fields, since idle skipping's split depends on which blocks are compiled when a wait begins. The shared fields took 1.03
times their first time, so the slowdown the first run showed from about frame 10,380 (fields of 323 ms rising to 650 to
810) is the content's, not the machine's.

| Fields | EE executed, mean / p50 / **p90** / max (M) | passed (M instructions, M cycles) | VU pairs, mean / **p90** / max (M) | VU1 p90 | VU0 p90 |
| --- | --- | --- | --- | --- | --- |
| `jak-dock` (6,061–6,360) | 1.67 / 1.74 / 1.74 / 2.17 | 4.12, 3.61 | 0.61 / 0.86 / 1.11 | 0.72 | 0.31 |
| `jak-island` (7,561–7,860) | 1.93 / 1.93 / 1.99 / 3.56 | 3.83, 3.35 | 1.34 / 1.39 / 2.82 | 1.22 | 0.18 |
| `jak-arena` (9,061–9,360) | 3.99 / 4.03 / 4.92 / 5.98 | 1.71, 1.50 | 1.92 / 3.23 / 3.49 | 2.82 | 0.67 |
| **all 4,800 (6,001–10,800)** | 2.43 / 1.94 / **4.92** / 5.98 | 3.30, 2.89 | 1.25 / **2.72** / 3.49 | **2.33** | 0.37 |

The arena window's mean, 3.99 M instructions and 1.71 M passed, is §3.3's figure exactly. The 90th, 95th and 99th
percentiles of the instructions executed are all 4,920,320: in at least a tenth of the fields the EE passes nothing
and executes an instruction for nearly every one of the field's 4.92 M cycles. That is the most a field can ask of the
EE at this rate, and it is the design point. 856 of the 4,800 fields (17.8 %) are above the arena's mean instructions or
pairs.

**What the census does not cover.** One game. Jak's 4,800 fields are its opening drawn in its engine, not play, which
the core does not yet reach (D-900). Kingdom Hearts and Dark Cloud 2 are in the probe's disc folder with no press
script, state or window, and no other scoreboard game is installed for the probe. A design point drawn from one game's
cutscenes is the floor of the busy scenes, not their 90th percentile across a library (§11 Q2 below).

**The budget redrawn at the design point** (§3.5's allocation; the VU's counts the census's own, D-1100), desktop ns an
item at a handheld factor of 1.35 / 2:

| | One thread | Two threads |
| --- | --- | --- |
| an EE instruction executed, everything in (4.92 M) | 0.75 / 0.51 | **1.36 / 0.92** |
| a VU pair, both units (2.72 M) | 0.98 / 0.66 | — |
| a VU1 pair (2.33 M) | — | 2.86 / 1.93 |
| a VU0 pair (0.37 M) | — | 3.00 / 2.03 |

### R0.2, the emitter's footprint and the ring

**The bytes** (the throwaway counter: each block the dispatcher compiles is also lowered and compiled three more times
outside the module, as today's emitter, with C1's fixed cost, and as the static prototype, and their sizes summed; the
arena window from the state at 9,000, its 360 fields): 31,770 blocks lowered of 267,094 guest instructions, 8.41 a
block.

| Form | Host bytes a guest instruction |
| --- | --- |
| today's emitter | **1,207.8** |
| C1, a fixed cycle an instruction (`EMUSEN_SAPPHIRE_C1`) | 923.8 |
| **the static prototype**: C1, with each instruction's issue charge reduced to a constant offset and the per-step replay store and Random's update removed | **567.9** |

The prototype is a lower bound on the static form's code, which also needs the quiescent test at each region's start (a
few compares, estimated at 30 to 40 bytes a block) and Random and Count computed at its exits. **It is 5.9 times the
mark of 96.** Today's emitter is 3.2 times §44.8's 380 bytes, which was measured at S36 and S37, before the loads' and
stores' exits, the rollbacks and the rewind (D-1101).

**The ring** (`examples/ring.rs`): Cranelift blocks chained in one random cycle through every block, each entered as
§4.2's chained jump would enter it: its horizon compare, a compare of its instruction-cache line's generation, its guest
instructions, its cycles added, a tail call to the next. Blocks hold 3 or 4 guest instructions, 3.6 on average, as
§44.8's Jak did. Each guest instruction is a run of synthetic loads, operations and stores on a register file in memory,
as many as make the compiled code the bytes asked for, measured on the code. Two rounds at 15,549 blocks and 98 bytes
agree within 0.5 %; the medians of five laps:

| Blocks live | Code | 98 bytes: ns an entry / an instruction | 390 bytes: ns an entry / an instruction |
| --- | --- | --- | --- |
| 1 | — | 3.25 / 0.81 | 15.3 / 3.82 |
| 500 | 0.18 MB at 98 bytes | 3.99 / 1.11 | 16.3 / 4.55 |
| 5,000 | 1.76 MB | 6.26 / 1.74 | 20.6 / 5.72 |
| **15,549** | **5.47 MB** (21.8 at 390) | **7.86 / 2.19** | **51.4 / 14.3** |
| 15,549 at the prototype's 585 bytes | 32.7 MB | 201.5 ns an entry, **56.0** an instruction | |

**Why the ring's 98 bytes and the prototype's 568 differ.** They measure different things. The ring's blocks are the
fastest code a block of that size could be. They carry only the chained entry's two compares, the instructions' own
work and the exit, with no exit path writing state back, no translation-cache probe, no data-cache tag compare, no undo
log and no helper call. The ring answers "if the static form were 96 bytes, would its footprint fit the allowance?". The
prototype is today's emitter with the issue model taken out, so it keeps every load's and store's inline page probe and
tag compare, their slow paths, and the side exits that each write back the block's registers and the model, and those
dominate its bytes. Its count includes the cold exit blocks, which Cranelift lays after the straight line and which are
fetched only when taken. The hot path's share was not separated (the compiled code gives no block boundaries without
Cranelift's `machine_code_cfg_info` flag, which the counter did not set), so the prototype's fetched footprint is below
568 by an unmeasured amount.

**Against the mark.** Even at 98 bytes the ring fails. At Jak's live-block count it costs 2.19 ns a guest instruction:
2.4 times the 0.92 allowed with two threads at a factor of 2, and 1.6 times the 1.36 at 1.35. Its footprint alone, 2.19
less the 0.81 of one block live, is 1.38 ns, more than either allowance before any instruction's real work. At the
prototype's bytes the code no longer fits the desktop's 32 MB last-level cache and the cost is 56 ns.

### R0.4, the VU's floats in vector lanes

`float_lanes` computes `float::add`, `sub`, `mul` and `fused` for four lanes in AVX2's 128-bit integer lanes, with the
MAC word in the VU's layout and the field mask. The adder aligns by a per-lane shift, which gives 0 for a count of 32 or
more, so no test of the distance is needed. It finds the sum's leading bit by converting the sum, with the bit below its
leading one cleared, to a float and reading the exponent. That conversion is exact under any rounding mode, since a
value whose second-highest bit is clear cannot round into the next binade. The multiplier forms four 48-bit products in
64-bit lanes less §56.4's loss. Nothing goes through the host's float arithmetic, and MXCSR is never read or changed.

**Equality** (`cargo test --release --features lanes`), against `float` lane by lane with the reference's MAC bits as
`vu::ops` sets them, the field mask random or full:

| Test | Cases |
| --- | --- |
| every pair of a reduced set: fields 0–5, 100–154 (every alignment distance 0 to 54) and 250–255, sixteen fractions at the significand's edges, both signs; each pair beside three others, against an accumulator from the set | 4,596,736 vectors, each graded as add, sub, mul and both multiply-adds |
| §56.5's classes | 4,000,000 vectors |
| random (ignored, 61 s) | 1,000,000,000 lanes |

All equal. The redesign asked for a reduced *format* graded exhaustively. `float` is not parameterised by its format,
so there is no reduced reference to grade against, and a reduced *set* of operands graded exhaustively in pairs takes
its place.

**Speed** (`examples/lanes.rs`: 1,024 vectors of operands drawn in §56.2's shares, 4,096 times over, the results stored;
the median of seven; two rounds):

| ns a lane operation | Lanes, round 1 / 2 | `float` lane by lane | Mark (desktop) |
| --- | --- | --- | --- |
| add | **1.072 / 1.058** | 1.78 | 0.5 |
| mul | **0.863 / 0.862** | 1.56 | 0.5 |
| multiply-add (two lane operations) | 2.290 / 2.275 | 3.03 | 1.0 |
| multiply-add chained through the accumulator | 2.234 / 2.204 | — | — |

The kernel is 1.3 to 1.8 times faster than `float` and 1.7 to 2.1 times slower than the mark. A lane addition is about
65 vector instructions for four lanes, so it is bound by its instruction count, not by latency: the chained
multiply-add costs what the independent one does.

**Seeded faults**, each caught by both tests: the leading-bit conversion without its clearing (F1), the loss table's
row for an even multiplicand and y ≡ 2 (mod 4) set to 0 (F2c), the MAC word's lanes unreversed (F3), and the
accumulator's field-255 rule removed (F4). Two first attempts at F2 survived and are **equivalent mutants**, which the
argument shows: rows 3 (y ≡ 1, x odd) and 5 (y ≡ 2, x odd) of the loss table can never change a result. With x odd the
exact product P is odd in row 3 and ≡ 2 (mod 4) in row 5, so neither P − 1 nor P − 2 crosses a multiple of 2^23, and
the truncated significand is the same with the loss or without it. **Only the even multiplicand's rows (4 and 6) reach
a result**, a sharper statement of §56.4 than it makes.

### R0.3 and R0.5: not run

R0.3's denominator is recorded: the arena window with one hot thread (`--blocks --idle --vu0 --vu1 --gs-workers 1`)
takes **819.8 and 818.0 ms** a field on the desktop (medians 722.2 and 727.2), two rounds. The handheld's run of the
same binary, the state at 9,000 and S44's `exact64` in a loop is ready and waits for the tester. `memtypes` (R0.5) runs
on llvmpipe, a single software type that reads 4 MB in 0.20 ms. That is a check of the probe and not evidence.

### Could anything close the gaps? (estimates, not measurements)

- **The EE's footprint (R0.2).** Longer chained blocks would not close it. The ring has no dispatcher already, and its
  cost grows with the bytes live, not with the entries: from 0.81 ns at one block to 1.11, 1.74 and 2.19 as the code
  grows from nothing to 0.18, 1.76 and 5.47 MB. Longer blocks with the same bytes move the same lines. The ring met the
  1.36 allowance only with the code within the desktop's L2 (500 blocks, 0.18 MB, 1.11 ns), and never met the 0.92. For
  Jak's 56,000 live guest instructions to stay within about 1 MB, the code would have to fall to about 18 bytes a guest
  instruction. No emitter the project has measured comes near that (Mars's 80 to 100, the prototype's 568). The one way
  the ring could be too pessimistic is locality. It visits every block uniformly, while a field may re-run a small
  working set: 4.92 M instructions over a live set of about 56,000 is about 88 runs of each instruction a field. R2's
  census of block entries a field would measure the working set. It is the only number that could reverse R0.2, and
  R0.2's mark on bytes would still fail.
- **The VU's lanes (R0.4).** Three changes to the lane form are visible, each an estimate:
  - Specialise the common case: 73 to 89 % of the VU's additions are two normal operands (§56.2). A path for those alone,
    leaving zeros, saturation and field 255 to a slower path, saves about 15 of the addition's 65 instructions.
  - Elide the MAC word where §4.3's analysis finds it unread: about 6 more.
  - Process two independent upper operations in AVX2's 256-bit lanes: about half the cost a lane, where a microprogram
    has two independent FMAC operations in a row.

  The first two together give an estimated 0.7 to 0.8 ns a lane addition on the desktop, still above 0.5. With the third,
  about 0.35 to 0.45, within the mark only for code that supplies operations in pairs. Whether microprograms do is a
  count R2 could take. The handheld's processor is not assumed to have AVX-512.

### Disputes raised

- **D-1100. VU pairs a field: §3.3's 2.53 M VU1 and 0.42 M VU0 on the arena against the bench's 1.56 M and 0.36 M.**
  The census's counter (`vpu.pairs_run`, which the bench's `vu` line has always printed) gives the same counts with
  both VUs interpreted and with both compiled (30 fields from 9,000, equal field by field), and the Oct-26 runs of the
  same window printed 1.56 M and 0.36 M. §58.4's figures came from that stage's own runs at N = 1 and were not
  re-derived here. Until they are, R0.1's pairs are compared with the arena on the census's own counter. **Open.**
- **D-1101. The EE emitter's bytes: §44.8's 380 a guest instruction against 1,208 measured.** §44.8 measured S36's and
  S37's blocks (3.6 instructions a block). Today's blocks average 8.41 and carry the loads' and stores' exits, rollbacks
  and rewind. Both counts are total code from the compiled size. The redesign's §4.2 and §3.5 used 380. **Settled by
  measurement: 1,208 for today's emitter on the arena.**

### Questions for the tester

1. **[decision] The stop.** R0.2 and R0.4 fail at the design point, so by §2 the plan stops before R1. The choices are
   §2's: compromises that change more than timing (C3 on the VU, or C1), or a narrower target than every field at full
   speed on the handheld. Recommended: before choosing, two cheap measurements that could move the verdict. (a) R2's
   working-set census of block entries a field, the only number that could make the ring's uniform walk too pessimistic.
   (b) The lane form specialised and in pairs (above), a day's kernel work, to settle whether R0.4's gap is the form's or
   the method's. If both stay short, accept the stop.
2. **[decision] The handheld session.** Recommended: run it once anyway. R0.3's factor and R0.5's memory type decide any
   narrower target too, and the binary, the state and the GPU load are ready.
3. **[decision] The scoreboard's games (§11 Q2), for a design point that is more than one game's opening.**
   Recommended as §11 Q2: press scripts and states for Kingdom Hearts and Dark Cloud 2 from their discs in the probe's
   folder, and the PlayStation 2 games installed on the handheld.

### Addendum: the two follow-up measurements (measured 2026-10-28)

*Decided 2026-10-28 by the tester: measure the two numbers that could move the stop (R0 results, Q1) before any
verdict on the target, and give them to the critic. The scoreboard is Jak's opening, Kingdom Hearts, Dark Cloud 2 and
the PlayStation 2 games installed on the handheld. Battery runs on the handheld are dropped, all of them: R0.3 measures
the handheld on the charger. The critic's estimate of a factor of about 2 on battery stays only as a recorded risk,
never measured. Measured figures and estimates are kept apart below; every estimate says so.*

**Verdict: unchanged, stop.** Neither follow-up moves R0.2 or R0.4 within its mark at the design point.

| Check | Measured (desktop) | Mark | Moves the stop? |
| --- | --- | --- | --- |
| R0.2, the ring at a field's working set | **2.00 ns** a guest instruction at 96 bytes (two rounds, 2.01 / 2.00) | 0.92 (two threads, factor 2) and 1.36 (factor 1.35) | **no**: 2.2 and 1.5 times the marks, against 2.4 and 1.6 at all 15,549 blocks |
| R0.4, two operations at once in eight lanes | **0.46 ns** a lane addition, **0.39** a lane product; a multiply-add 1.29 a lane (0.64 a lane operation) | 0.5 | **no**: under the mark for an addition or a product alone, over it for the multiply-add, the VUs' most common operation (§56.2). Only 60 % of the adder's and multiplier's operations can be paired |
| R0.1 on Kingdom Hearts and Dark Cloud 2 | not run | — | — |

**Kingdom Hearts and Dark Cloud 2 (C): skipped.** Both boot from the probe's images, but neither reaches gameplay on the
core. Dark Cloud 2 holds a grey shape after its memory-card message (`Sapphire_Native.md` §39.19), and Kingdom Hearts
has no press script past its logo (D-464). Getting them to gameplay is core work, not part of R0.

#### R0.2 again: the working set a field

**The census** (the bench's `--working FILE`, the `census` feature): for each field, the compiled blocks entered, their
guest instructions, entries and steps, and the fewest blocks holding nine tenths of the entries (the hot set). Jak's
4,800 fields from 6,000 to 10,800, `--defaults`, from the state at 6,000, under a watch comparing each 100 fields'
time with the first census's for the same frames (0.97 to 1.08 times throughout). Two earlier attempts ended early.
One was stopped by a watch rule that tripped on Jak's own swings, and in one the sizes the perf map gave were corrupt
(D-1102). A third was killed from outside the run at frame 7,313, its watch healthy to the end.

| A field | p50 | p90 | max |
| --- | --- | --- | --- |
| compiled blocks entered | 4,072 | 10,200 | 12,887 |
| their guest instructions | 32,846 | 83,077 | 105,734 |
| hot blocks (nine tenths of the entries) | 1,077 | 1,926 | 2,790 |
| their guest instructions | 4,778 | 9,450 | 17,571 |
| steps an entry | 2.87 | 3.01 | — |
| steps in compiled code | 0.87 M | 1.48 M | 3.70 M |

The arena window alone has p90 fields of 11,366 blocks and 91,879 guest instructions, 2,168 blocks hot. A field
enters about two thirds as many blocks as §44.8's 15,549 live, and nine tenths of its entries fall on a fifth of them.
**What the census does not cover:** only today's compiled code. That is 0.87 M to 1.48 M steps of the 1.94 M to 4.92 M
a field executes (R0.1). The interpreted remainder, which P-R2 would compile, has code of its own that no count here
includes, so the working set measured is a lower bound.

**The ring at that working set** (`ring mix`): the p90 field by blocks entered, 2,048 hot blocks in a random ring
turned 36 times to each pass through a random chain of the 8,152 cold ones. That makes the hot blocks nine tenths of
the entries, as measured. Each entry runs 2.87 guest instructions, every block's place in memory is random, and the
rest is as R0.2's ring. The medians of five laps, two rounds:

| Ring | Code | ns an entry | ns a guest instruction |
| --- | --- | --- | --- |
| one block, 3 instructions an entry, 96 bytes | — | 2.26 | 0.75 |
| the 2,048 hot blocks alone, 96 bytes | 0.56 MB | 4.03 | 1.41 |
| **hot and cold, 96 bytes** | 2.82 MB | **5.77 / 5.74** | **2.01 / 2.00** |
| hot and cold, 96 bytes for each of a block's 8.13 static instructions (281 an executed one) | 8.26 MB | 15.0 / 15.0 | 5.23 / 5.21 |
| hot and cold, the static prototype's bytes (586) | 17.2 MB | 30.7 / 30.2 | 10.7 / 10.5 |

The working set takes the ring from 2.19 ns to 2.00 at 96 bytes, and the hot set alone would take it to 1.41. Neither
meets 0.92. The hot set alone is within 4 % of the 1.36 allowed at a factor of 1.35, but the cold entries are a tenth
of the field's, and they carry the cost. **What dominates now is the entry**: today's blocks run 2.87 instructions an
entry, so even one hot block costs 0.75 ns an instruction, most of it the entry's compares, its counters and the call.

**Could longer blocks close it?** (estimates, not measurements) The redesign's blocks end only at transfers they
cannot follow (§4.2), so an entry would carry more instructions than today's 2.87, which leave at loads, stores and
side exits. If the entry's fixed cost of about 1.5 ns (the one-block ring's 2.26 ns less its three instructions' work)
were spread over 10 instructions, the hot-and-cold ring would come to about 1.1 to 1.3 ns an instruction at 96 bytes.
That is within 1.36 and still above 0.92. Two things stand against it. The 96 bytes are the mark, not a measured form:
the prototype measures 568. And the interpreted code the census does not count would join the working set. **R0.2
stays failed.**

#### R0.4 again: the lane form specialised and paired

`float_lanes` is now one generic form over AVX2's four lanes and eight, of which eight are two independent four-lane
operations. Four forms are timed:

- **the full form:** results and MAC words;
- **`*_bits`:** without the MAC word, for operations whose flags no later pair reads;
- **`*_fast`:** a short path for vectors with no zero operand and no exponent overflow or underflow, with any other
  vector sent to the full form;
- **`*2`:** two operations at once.

Every form is graded bit-equal, results and MAC words with the field masks, against `float` lane by lane: 4,596,736
reduced-set vectors, 4,000,000 class vectors and the billion-lane random test, each now also checking the paired, flag-free
and short forms. Seeded faults, each caught by both tests:

- F5: the second operation's field mask read from the first's;
- F6: the product's odd lanes blended wrongly;
- F7: the short path's underflow check dropped.

| ns a lane operation (three rounds; seven or nine laps each) | Four lanes | Eight lanes, two operations |
| --- | --- | --- |
| add, full | 1.09–1.25 (the first kernel's 1.06) | **0.456–0.472** |
| mul, full | 0.95–1.09 (the first kernel's 0.86) | **0.383–0.393** |
| multiply-add, full (two lane operations) | 2.34–2.68 | 1.279–1.294 |
| add, without the MAC word | 0.817–0.822 | 0.453–0.468 |
| mul, without the MAC word | 0.697–0.700 | 0.385–0.395 |
| add, the short path | 1.244–1.248 | — |

The generic form's four lanes are slower than the first kernel's: its even-and-odd product and the mask built per
call cost about 10 % on the adder and about 20 % on the multiplier. The first kernel's figures stand for four lanes.
The short path is no faster than the full form. §56.2's shares put a zero or would-be denormal in one VU addition
operand in 10 to 25 %, so most four-lane vectors hold such a lane and take the full form after the short one. Dropping
the MAC word saves a quarter. **Pairing halves the cost a lane**: an addition or a product alone comes under 0.5.

**How often two operations are there to pair** (a counting patch on VU1's interpreter, kept in the stage's scratch
and reverted; the arena and island windows, 360 fields each from their states). An adder or multiplier upper
operation (ADD, SUB, MUL, MADD, MSUB in every form, OPMULA, OPMSUB) can run with the next pair's upper operation if
that one is also one of them and reads no lane of a VF register, nor ACC, that the first writes. Pairs are taken
greedily along the order of execution, a pairing undone when the next pair run is not the next in memory. The count
is static over distinct pair positions (by address and both words) and dynamic over the pairs run:

| | `jak-arena` | `jak-island` |
| --- | --- | --- |
| VU1 pairs run | 587.1 M | 434.8 M |
| of which adder or multiplier upper operations | 318.8 M (54.3 %) | 238.9 M (54.9 %) |
| **those run two at a time** | **192.9 M (60.5 %)** | **143.9 M (60.2 %)** |
| distinct pair positions; adder or multiplier; pairable with the next | 7,181; 3,725; 1,723 (46.3 %) | 5,354; 2,718; 1,272 (46.8 %) |

**What the two give together** (an estimate from measured parts). With 60 % of the adder's and multiplier's
operations paired and 40 % single on the first kernel, a lane addition averages about 0.6 × 0.46 + 0.4 × 1.06 =
0.70 ns and a lane product about 0.6 × 0.39 + 0.4 × 0.86 = 0.58 ns. A multiply-add, at 0.64 a lane operation paired,
is above the mark even when paired. MAC words elided where unread would take some 10 to 25 % off, by a share not yet
counted. **R0.4 stays failed at the desktop's 0.5**, by about 1.2 to 1.4 times instead of 1.7 to 2.1. The handheld's
mark of 1.0 depends on R0.3's factor, which is not measured.

#### The perf map and D-1101

No R0 figure came from the perf map. D-1101's 1,208 bytes a guest instruction came from the throwaway counter's own
compiles (Cranelift's `code_info().total_size` for each shadow-compiled block), and its check from emusen-jit's byte
counter. The census now counts guest instructions. The two-writer race is D-1102, with its proposed fix.

### Second addendum: the plateau, the marks at §3.1's definition, and instructions an entry (measured 2026-10-28)

*Decided 2026-10-28 by the tester:*
- *The marks use §3.1's definition: a mean of 13.4 ms a field and the 95th-percentile field under 16.7 ms on the
  handheld, not the 90th-percentile field against the mean.*
- *Heavy fields that stay short of it are a named limitation, the tier kept.*

*The critic's review of the census found the plateau below. The handheld's kernel runs (step 1) did not take place:
EmuSen was running on the handheld, and closing it was refused by the permission system, so the handheld's own ns are
still owed and the charger's assumed factor of 1.35 stands in for them.*

| Check | Measured (desktop) | Mark at §3.1, two threads, factor 1.35 | Result |
| --- | --- | --- | --- |
| R0.1 the design point | the plateau excluded: EE executed mean **2.19 M**, p95 **4.08 M**; VU1 mean 1.14 M, p95 2.77 M | — | **redrawn** (below) |
| R0.2 ns, at Jak's working set and 96 bytes | **1.40–1.47 ns** a guest instruction with §4.2's 14.5 instructions an entry; 2.00 with today's 2.87 | **2.04** (2.75 on the handheld) | **pass** with either; 1.69 for the arena's own fields, which only §4.2's entries meet |
| R0.2 bytes | the static prototype **568** bytes a guest instruction; at that size the ring costs 13.2–13.7 ns | at most 96 | **fail** |
| R0.3 the factor | not run | at most 2.0 | **not run** |
| R0.4 lanes | four lanes: 1.06 (add), 0.86 (mul); two operations at once: 0.46, 0.39; a multiply-add 1.17 a lane operation in four lanes, 0.64 paired | **0.83** (1.125 on the handheld) | four lanes **fail**; paired **pass**; the 60/40 mix of pairable and single operations, an estimate: add 0.70 and mul 0.58 pass, the multiply-add about 0.85 at the mark |

**Verdict: still stop by §2's rule**, but on two narrow points. (1) R0.2's byte mark: the ns pass only if the static
form comes to about 96 bytes a guest instruction, and the one prototype measures 568. (2) R0.4 is under the mark
paired but not alone, and the mix is an estimate. R0.3 is not measured. What would settle each is in the last
paragraph.

#### The plateau, proven

- **Which fields.** 420 of the 4,800 fields, 82 of the arena window's 300, execute exactly 4,920,320 or 4,919,296 EE
  instructions with no VU pair, in runs of 20 to 23 fields separated by one or two (8,381–8,471, 9,176–9,268,
  9,513–9,673, 10,439–10,530, 10,728–10,800). The critic counted 428 and 83; the difference is the join's.
- **What runs** (a counting patch on the EE's step, in the stage's scratch and reverted, counting transfers by class
  and keeping no address; the interpreter tier with idle skipping on, from the state at 9,000, fields 9,177 to 9,197).
  Each field makes 702,903 taken conditional branches, one every seven instructions, and no other transfer. With blocks
  on, the working-set census shows the same loop as two hot blocks of five or six guest instructions (272 of the 420
  fields), entered about 446,000 times a field.
- **What it reads** (the bench's `--registers`, the same fields). The EE reads **10009000h, D1_CHCR**, the control
  register of the DMAC's channel 1 (VIF1), 702,875 times a field: once a turn. Nothing else the EE reads comes close;
  the IOP's busiest register is read 333 times a field. No VU pair runs, so VIF1 is not feeding VU1 while the EE waits
  for the channel to end.
- **Why the skipper misses it.** Idle skipping refuses a turn that reads a unit's register at 10000000h
  (`Refusal::Device`). The counts show about 172 refusals a field on that kind and about 702,000 arrivals a field
  waiting behind its backoff. Every turn is stepped, and the EE executes an instruction every cycle of the field.
- **The observations those fields make:** 702,875 EE register reads a field. That is **seven times P-R1's bound of
  100,000 events a field**, so the plateau, as the console runs it, refutes P-R1 for these fields unless the loop is
  passed rather than observed. Passing it would need the skipper to treat a DMAC register as changed only at the
  DMAC's events. That is not done here: idle skipping is unchanged.
- **Whether the console does the same is open (D-1103):** a VIF1 transfer that does not end for 23 fields while
  neither VU runs may be the game's own wait, or a stall like D-465's.

#### The design point and the marks, redrawn at §3.1's definition

The plateau is excluded from the design point and named. Its fields keep their tier: the poll is exact, and on today's
skipper they cost a full field of EE instructions. With the plateau excluded, Jak's 4,380 other fields of the opening:

| A field | mean | p90 | p95 |
| --- | --- | --- | --- |
| EE instructions executed | 2.187 M | 3.618 M | **4.083 M** |
| VU1 pairs | 1.144 M | 2.345 M | **2.771 M** |
| VU0 pairs | 0.230 M | 0.375 M | 0.521 M |
| both VUs' pairs | 1.375 M | 2.721 M | 3.159 M |

The arena window's 218 other fields: EE mean 3.635 M, p95 4.919 M; VU1 mean 2.146 M, p95 2.850 M.

**The marks.** §3.5's two-thread allocation gives the EE 9.0 ms and VU1 9.0 ms of the handheld's 13.4 ms mean. At the
95th-percentile field the same shares of the 16.7 ms ceiling are 11.2 ms each. An item's allowance is the smaller of
the mean budget over the mean count and the p95 budget over the p95 count:

| Allowance (handheld ns; desktop at 1.35) | Jak's opening | the arena's fields |
| --- | --- | --- |
| an EE instruction | min(4.12, **2.75**) → **2.04** | min(2.48, **2.28**) → 1.69 |
| a VU1 pair | min(7.87, **4.05**) → 3.00 | min(4.19, **3.94**) → 2.92 |
| a lane operation (R0.4's 1.0 at 3.6 ns a pair, scaled) | **1.125** → **0.83** | 1.09 → 0.81 |
| one thread, an EE instruction (5.0 ms mean, 6.2 at p95) | min(2.29, 1.53) → 1.13 | — |

On one thread no measured EE figure meets 1.13. The split (§4.7) is what lets the EE's marks pass.

#### Instructions an entry under §4.2's rule

The same counting patch on the arena's fields 9,061 to 9,175, interpreted, gives per field:
- 4.053 M instructions;
- 251,942 taken conditional branches;
- 395 J and JAL;
- 21,211 JR and JALR;
- 6,282 other non-sequential steps: exceptions, ERET and the skipped slots of likely branches.

That is **14.5 instructions a transfer**. A block that ends only at a transfer it cannot follow (§4.2) runs at most that
many an entry; page ends and lines the cache does not hold shorten it, and were not counted. Today's blocks run 2.87
an entry because loads and stores leave them.

**The ring at that length** (`ring mix`, desktop, two rounds): Jak's p90 working set of 83,077 guest instructions in
blocks of 14.5, 652 hot and 5,078 cold, 70 hot laps to each cold pass (nine tenths of the entries hot). At 99 bytes
(8.23 MB of code) it costs 20.4–21.4 ns an entry and **1.40–1.47 ns a guest instruction**. At the prototype's 587
bytes (48.8 MB) it costs 13.2–13.7 ns.

#### An AVX-512VL lane form (proposed, not built; estimates)

The handheld's processor (a Ryzen Z1 Extreme) and the desktop's both report AVX-512VL. Three of its instructions
replace sequences in today's adder:
- `vplzcntd` gives the leading bit directly, replacing the five-instruction conversion trick;
- `vpternlogd` merges the sign, field and mask logic three operations at a time;
- mask registers replace `blendv` and the zero handling.

The estimate is a quarter to a third fewer instructions for an addition. That would bring the four-lane full form to
about 0.75 to 0.8 ns a lane addition on the desktop, just under the 0.83 mark, and the paired 256-bit form
to about 0.35. 512-bit registers would hold four operations at once, which few microprograms supply in a row (60 %
pair even two). Worth building only if R0.4's mix is to be measured, not estimated.

#### What would settle the remaining points

- **R0.2's bytes:** an emitter prototype of §4.2's static form (the page table, chaining, no replay and no undo log)
  measured on the same counter, against 96 bytes.
- **R0.4's mix:** the paired kernel run on recorded microprograms, rather than the 60/40 estimate.
- **R0.3 and both kernels' handheld ns:** the charger session, once EmuSen there is closed.

### Third addendum: both kernels on the handheld (measured 2026-10-28)

*EmuSen on the handheld was closed at the tester's request (SIGTERM, exited in 2 s) and was not restarted. The handheld
was on the charger (ACAD online = 1 before and after) and the GPU was not used. The same binaries ran as on the desktop,
two rounds each under `systemd-run --user`, and were removed afterwards. Clocks sampled every 0.5 s through the runs:
- the memory clock's active level moved between 400, 800, 937 and 1,200 MHz, mostly 800 and 937;
- the fabric clock between 400, 937 and 1,200 MHz (21 of 91 samples marked no level);
- the fastest core ran at 4.8 to 5.0 GHz.*

| Kernel | Handheld, round 1 / 2 | Desktop | Ratio |
| --- | --- | --- | --- |
| ring, working set, today's 2.87 instructions an entry, 96 bytes | 2.236 / 2.371 ns a guest instruction | 2.01 / 2.00 | 1.12–1.18 |
| ring, working set, §4.2's 14.5 an entry, 99 bytes | 2.430 / 2.247 | 1.474 / 1.404 | 1.60–1.65 |
| ring, the same at the prototype's 587 bytes | 22.66 / 20.53 | 13.67 / 13.22 | 1.55–1.66 |
| ring, one block | 0.765 / 0.746 | 0.754 | 1.0 |
| lanes, four, add | 1.184 / 1.229 ns a lane | 1.09–1.25 | ~1.05 |
| lanes, four, mul | 1.030 / 1.070 | 0.95–1.09 | ~1.05 |
| lanes, four, multiply-add (two lane operations) | 2.509 / 2.632 | 2.34–2.68 | ~1.05 |
| lanes, paired, add | 0.504 / 0.522 | 0.456–0.472 | 1.10 |
| lanes, paired, mul | 0.434 / 0.442 | 0.383–0.393 | 1.12 |
| lanes, paired, multiply-add | 1.409 / 1.446 | 1.279–1.294 | 1.10 |

The kernels' own factors are 1.0 to 1.2 where the code stays in the cache and about 1.6 where §4.2's longer blocks
stream 8 MB of code. They measure kernels, not the emulation thread under a busy GPU, so they do not replace R0.3.

**The marks against the handheld's own ns** (§3.1's definition, two threads, the plateau excluded; marks from the
second addendum, no factor applied):

| Check | Handheld | Mark (handheld) | Result |
| --- | --- | --- | --- |
| R0.2 ns, Jak's opening | 2.24–2.37 (today's entries), 2.25–2.43 (§4.2's) | 2.75 | **pass** |
| R0.2 ns, the arena's fields | the same | 2.28 | **at the mark**: one round each side of it |
| R0.2 bytes | the prototype's 568 | 96 | **fail** |
| R0.4, four lanes | add 1.18–1.23, mul 1.03–1.07, multiply-add 1.25–1.32 a lane operation | 1.125 | add and multiply-add **fail**, mul **passes** |
| R0.4, paired | add 0.50–0.52, mul 0.43–0.44, multiply-add 0.70–0.72 a lane operation | 1.125 | **pass** |
| R0.3 | not run | 2.0 | **not run** |

### Fourth addendum: the lanes on VU1's own stream, and the static form's hot bytes (measured 2026-10-28)

*Decided 2026-10-28 by the tester ("yes to both"): replace the 60/40 estimate with VU1's own stream, and measure a
static-form emitter prototype's bytes, hot and cold apart. Both are measurements; nothing in the default build changes.*

| Check | Desktop | Handheld (charger) | Mark (desktop / handheld) | Result |
| --- | --- | --- | --- | --- |
| R0.4 on VU1's stream, pairs of one kind in eight lanes | **1.28–1.31** ns a lane operation written | **1.43–1.47** | 0.83 / 1.125 | **fail**, 1.55 and 1.30 times |
| the same, every operation in four lanes | 1.37–1.40 | 1.50–1.57 | 0.83 / 1.125 | fail |
| R0.2 bytes, the static prototype's hot straight line | **273.5** host bytes a guest instruction (cold exits 272.6 more) | — | 96 | **fail**, 2.85 times |
| R0.2 ns, the ring at those hot bytes (283), §4.2's 14.5 an entry | **4.95–5.03** ns a guest instruction | **10.1–10.3** | 2.04 / 2.75 | **fail**, 2.4 and 3.7 times |
| the same with today's 2.87 an entry | 5.20 | 6.49 | 2.04 / 2.75 | fail |

**Verdict: stop.** Measured on VU1's own operations, R0.4 fails on both machines. Measured on the hot straight line
alone, R0.2's bytes fail, and the ring at the bytes reached fails on both machines. Nothing is left estimated in either
verdict. What remains estimated is only how a different design than the one measured might do: the AVX-512VL form
(the second addendum) and an emitter further from today's than this prototype.

#### R0.4 on VU1's own stream

**The count** (a throwaway counter on VU1's interpreter, in the stage's scratch and reverted, keeping no address, word
or operand: only how many operations of each kind and lanes ran, and how many pairs the microcode allows). The rule
is the earlier addendum's: an adder or multiplier operation pairs with the next pair's if that one reads nothing the
first writes. Pairs are taken greedily along the execution. The arena and island windows, 360 fields each, from their
states:

| | `jak-arena` | `jak-island` |
| --- | --- | --- |
| adder and multiplier operations | 318.8 M | 238.9 M |
| additions / products / multiply-adds | 23.6 / 39.9 / 36.5 % | 19.3 / 42.0 / 38.7 % |
| run in pairs | 60.5 % | 60.2 % |
| **run in pairs of one kind** (what the eight-lane kernel takes) | **18.7 %** | **22.8 %** |
| lanes written an operation | 4.95 (a multiply-add counted twice) | 4.94 |

Most pairs join an addition and a product, which the paired kernel, one operation over eight lanes, cannot take
together. Multiply-adds seldom pair: each reads the ACC the one before it wrote.

**The timing** (`lanes mix`): a million operations in the counted shares, the lanes written as counted, operands in
§56.2's classes, each kind in runs as compiled code would hold them. A pair of one kind runs in eight lanes and
everything else in four. The ns are over the lanes written, a multiply-add counting twice; over the lanes computed
they are about a tenth lower. Two rounds each window:
- desktop: **1.28–1.31** ns paired where it can, 1.37–1.40 all in four lanes;
- handheld: **1.43–1.47** and 1.50–1.57.

A shuffled order, which adds the host's mispredicted dispatch between kinds, costs 1.76–1.79 on the desktop. The
stream is costlier per lane than any kernel alone for three reasons. The multiply-add is its largest share and its
dearest operation. Only a fifth of the operations pair in one kind. And a masked operation computes four lanes to
write fewer.

#### R0.2: the static prototype's bytes, hot and cold

**The prototype** (the throwaway byte counter of the R0 results, carried further): C1's constant issue charges, no
per-step replay store, no per-step update of Random, and now no undo log on the stores. Each block the dispatcher
compiles is compiled again outside the module in three forms. Every IR instruction is tagged by whether its block is
cold, and the hot straight line is measured from the code's own layout: Cranelift places every cold block after the
hot ones, so the hot region ends where the first cold-tagged range starts. The arena window, 360 fields from the state
at 9,000: 29,997 blocks of 270,601 guest instructions, 9.02 a block. A guest instruction costs:

| Form | Hot straight line | Cold exits | Total |
| --- | --- | --- | --- |
| today's emitter | 543.8 bytes | 705.2 | 1,249.0 |
| C1 | 348.9 | 597.5 | 946.4 |
| **the static prototype** | **273.5** | 272.6 | 546.1 |

The tagging is clean: in the prototype one byte a guest instruction of hot-tagged code falls past the first cold range;
today's emitter has 23. **The fetched part is 273.5 bytes, 2.85 times the mark of 96.** The remaining hot bytes are
the instructions' own work plus each load's and store's inline translation probe, data-cache tag compare and dirty-bit
update.

**The ring at the bytes reached** (283 bytes a guest instruction, Jak's p90 working set):
- with §4.2's 14.5 instructions an entry: 4.95–5.03 ns a guest instruction on the desktop (23.5 MB of code), 10.1–10.3
  on the handheld;
- with today's 2.87 an entry: 5.20 and 6.49.

All fail 2.04 and 2.75. The handheld's longer-block figure is the worse of the two because 23.5 MB of code exceeds its
last-level cache.

#### Handheld and desktop, every kernel of R0 side by side

| Kernel | Desktop | Handheld (charger) |
| --- | --- | --- |
| ring, working set, today's entries, 96 bytes | 2.00–2.01 ns a guest instruction | 2.24–2.37 |
| ring, working set, §4.2's entries, 99 bytes | 1.40–1.47 | 2.25–2.43 |
| ring, working set, §4.2's entries, 283 bytes | 4.95–5.03 | 10.1–10.3 |
| lanes, four, add / mul / multiply-add a lane | 1.09–1.25 / 0.95–1.09 / 2.34–2.68 | 1.18–1.23 / 1.03–1.07 / 2.51–2.63 |
| lanes, paired, add / mul / multiply-add a lane | 0.46–0.47 / 0.38–0.39 / 1.28–1.29 | 0.50–0.52 / 0.43–0.44 / 1.41–1.45 |
| lanes on VU1's stream, a lane operation written | 1.28–1.31 | 1.43–1.47 |

## R0 conclusion (2026-10-28): the stop is accepted

*Decided 2026-10-28 by the tester, before the last round was run: the exit rule.*
- *Split the static prototype's hot bytes by purpose, and say whether an exact path to about 96 bytes a guest
  instruction exists. If none does, R0.2 is closed as failed.*
- *Build the AVX-512VL lane form, AVX2 kept as the fallback, and time it on VU1's recorded mix on the handheld. If it
  misses 1.125 ns a lane operation, R0.4 is closed as failed.*
- *If either closes, the stop is accepted.*

*The same day the tester allowed the handheld's GPU for tests, so R0.3 and R0.5 ran there in full, on the charger.*

**R0.4 closed as failed, so by the rule the stop is accepted.** By §2 the plan does not go on to R1. The choices are
the tester's:
- compromises that change more than timing (C3 on the VU, or C1);
- a target narrower than every field at full speed on the handheld.

### Every check, at the end

| Check | Measured | Mark | Result |
| --- | --- | --- | --- |
| R0.1 the design point | the plateau (D-1103) excluded: EE p95 4.08 M, VU1 p95 2.77 M a field | — | redrawn (second addendum) |
| R0.2 ns, at 96 bytes | handheld 2.24–2.43 ns a guest instruction | 2.75 | pass |
| R0.2 bytes | the static prototype's hot straight line, 274.5 a guest instruction (arena), 248.5 (dock) | 96 | not met by the prototype; an exact path to about 96 is plausible by estimate (below), so not closed by the rule |
| R0.3 the handheld's factor | **1.19–1.20** (978.8 and 983.4 ms a field against the desktop's 819.8 and 818.0) | at most 2.0 | **pass** |
| R0.4 the lanes on VU1's own mix, AVX-512VL | handheld **1.48–1.51** ns a lane operation written (AVX2 1.39–1.45) | 1.125 | **fail: closed** |
| R0.5 unified memory | no device-local, cached, host-visible type; GPU-written host-cached memory reads 4 MB in **0.147–0.154 ms** after the fence | 1 ms | met through host-cached system memory |

### R0.2's bytes, by purpose

**The method.** The static prototype is the fourth addendum's. Every IR instruction is tagged with the emitter's
purpose for it, the hot straight line's bytes are summed by tag, and the cold exits are left out. Cranelift's optimiser
drops the source locations of the pure operations it re-creates, so a share of the bytes is untagged. Separately, the
pure IR operations of each purpose are counted before optimisation, and the untagged bytes are apportioned in those
proportions. **That apportioning is an estimate**; the tagged bytes are measured. Aggregate counts only.

The arena window: 31,416 blocks of 264,275 guest instructions, of which **39.7 % are loads and stores**, 51.6 % plain
and 8.7 % branches. The dock window: 248.5 hot bytes, the same order of purposes.

| Purpose (arena) | Tagged hot bytes a guest instruction | With the untagged share (estimate) | Smallest exact form | Its bytes a guest instruction (estimate) |
| --- | --- | --- | --- | --- |
| **cache tags and dispatch** | 80.5 | **about 130** (about 330 a memory instruction) | an inline fast path for a hit in either way of the set (a tag compare a way, the line's address, the dirty bit for a store); the scratchpad, uncached and accelerated pages, a miss, a locked line and every counter out of line | about 24 (about 60 a memory instruction) |
| exits' write-back | 46.6 | about 55 | write back only the registers the block changed, and Count, Random and the cycle total from the block's constants; the issue model's fields not at all in a static region | about 6 |
| entry | 25.6 | about 29 | a shared entry stub: the quiescent test's few compares and the two memory-context loads, not the twenty fields of today's model | about 5 |
| translation | 17.2 | about 28 | §4.2's page table: a shift, a load and a compare of the kind | about 8 |
| address and alignment | 5.1 | about 11 | the base, the offset, the alignment check | about 4 |
| horizon and line checks | 4.3 | about 7 | one compare a load or store against the horizon | about 3 |
| data access | 4.3 | about 6 | the load or store itself | about 6 |
| the instruction's work | 1.3 | about 6 | as now | about 6 |
| timing | 0.2 | about 4 | constants folded at the exit | about 1 |
| guest registers | 3.6 | 3.6 | as now | about 4 |
| exception checks | 0.0 | 0.0 | as now | 0 |
| **in all** | 274.5 (untagged 91.6) | 280 | | **about 66** |

**The verdict on the bytes.** By these estimates an exact path to about 96 bytes exists: about 66 bytes a guest
instruction. Nearly all of the saving comes from three levers:
- **the cache path:** about 130 down to about 24;
- **the exits' write-back:** about 55 down to about 6;
- **the entry:** about 29 down to about 5.

None of the three changes a result. The estimate is not a measurement, and the prototypes that would measure it were
not built, because R0.4 closed first. R0.2 is therefore **not closed as failed**. It stays unproven either way, and the
cache path's 330 bytes a memory instruction is its leading lever if the work resumes.

### R0.4: the AVX-512VL form

`float_lanes::wide` is the same generic form with two of its operations taken from AVX-512VL: the leading bit from
`vplzcntd` and `a | (b & c)` from `vpternlogd`. The AVX2 form stays the fallback, chosen by `available512` at run time.
It is bit-equal to `float`, with flags and field masks, in four lanes and paired, under the same tests: 4.6 M
reduced-set vectors, 4 M class vectors and the billion-lane run (305 s). Two seeded faults, each caught by both tests:
- W1: the leading bit one off;
- W2: the merge's truth table wrong.

**Timed on VU1's recorded mix** (`lanes mix FILE 7 wide`), two rounds each window, with pairs of one kind in eight
lanes:

| | AVX2 | AVX-512VL |
| --- | --- | --- |
| desktop (another job of up to 8 GB running alongside) | 1.34–1.37 | 1.39–1.44 |
| **handheld** (charger) | 1.39–1.45 | **1.48–1.51** |

The AVX-512VL form is about 5 % slower, not faster. The two instructions save little against the adder's and
multiply-add's length, and the AVX-512 encodings cost the rest. **Against the handheld's 1.125, both forms fail**, by
1.24 to 1.34 times. The binding pattern is the multiply-add chain through ACC, 37–39 % of VU1's operations, which
neither pairing nor wider registers reach. **R0.4 is closed as failed.**

**Revised 2026-10-28 (D-1104).** The multiplier's dropped columns are settled at 15, which made this section's loss
table, §56.4's for one column, wrong; `float_lanes` now takes the loss as `float::product` does, bit-exact under the
same three suites, and costs more: 1.81–2.12 ns a lane operation on the desktop and 1.92–1.96 on the handheld for the
AVX2 form, 1.63–1.69 and 1.79–1.85 for the AVX-512VL form (`Sapphire_Speed.md` §59.6). The figures above are for
*k* = 1. An exact kernel on the host's floats, built beside it, reaches 1.66–1.70 on the handheld and also fails
(§59.7).

### R0.3: the handheld's factor (measured on the charger)

The same bench binary and the state at 9,000 as the desktop's denominator ran the arena window with one hot thread
(`--blocks --idle --vu0 --vu1 --gs-workers 1`). S44's exact64 kernel ran in a loop on the handheld's GPU beside it,
started 60 s ahead so that the clocks settled. Two rounds, sampled by `mon.py`:
- **978.8 and 983.4 ms a field** (medians 871.3 and 876.6) against the desktop's 819.8 and 818.0: **a factor of 1.19
  to 1.20**;
- the picture, sound and state digests equal to the desktop's;
- the GPU 56 to 57 % busy at 800 MHz throughout, memory and fabric clocks at 937 MHz throughout;
- package power 31 W (p90 37 W);
- the emulation thread's core at 4.47 to 4.49 GHz.

The factor is well under the mark of 2.0, and under the 1.35 the tables were drawn at.

### R0.5: unified memory (measured on the charger)

`memtypes` on the handheld's GPU (RADV PHOENIX), 4 MB a type, nine rounds:

| Types | Read after the fence | Written in place |
| --- | --- | --- |
| host-visible, coherent, **cached** (system memory: types 5 and 16) | **0.147–0.154 ms** | 0.056–0.057 ms |
| device-local and host-visible, uncached (types 3, 4 and 15) | 74.6–75.3 ms | 0.118–0.125 ms |
| host-visible, uncached (system memory: types 2 and 14) | 74.1–75.9 ms | 0.109 ms |
| device-local only (type 0), staged through type 5 | copy and read 0.303 ms | — |

No type is device-local, host-visible and cached together. The mark is met another way: the GPU writes into
host-cached system memory, which the processor reads at once. That is §4.6's device path for readback on this machine,
and a staged copy from device-local memory costs 0.30 ms for 4 MB. The protected types read back wrong words, as
protected memory must, and are not candidates.

### How many fields would make full speed on the handheld, at the measured costs (an estimate from R0's data)

*An analysis of data already measured; nothing was run for it.*

**The model.**
- **Per-field counts:** the census's per-field counts of Jak's opening, the plateau's 420 fields excluded (D-1103).
  Each field's two threads are costed as §4.7 splits them.
- **Thread A:** the EE's instructions executed at the ring's handheld ns a guest instruction, and VU0's pairs at the
  lanes' cost, plus §3.5's allocations for the board (0.7 ms), the IOP's side (1.2), the remainder (0.5) and the spare
  (0.5).
- **Thread B:** VU1's pairs at the lanes' cost, plus §3.5's allocations for VIF1 and the GIF (2.0), the parser (1.4),
  the quantum barriers (0.5) and the spare (0.5).
- **The lanes' cost:** 2.70 lane operations written a pair (VU1's own mix: 0.54 adder or multiplier operations a pair,
  4.95 lanes each) at 1.42 ns, the AVX2 form's handheld figure on that mix.
- **The field's time** is the slower thread's.
- **The kernels' ns are the handheld's own**, so no factor is applied. R0.3's measured 1.2 says the handheld's
  emulation thread is no slower than that against the desktop.

**What the model leaves out, so that it is optimistic:**
- every unit cost only by the items R0 measured or by §3.5's allocations;
- VU1's work beyond its floats (the integer and lower operations, the pipeline, the dispatch), which is not counted;
- the EE's helpers and its interpreted fallbacks;
- the rule that a unit's allocation is met;
- the ring's own synthetic work standing in for the instructions' real work.

| Window (fields) | EE at the prototype's 274 bytes, §4.2's entries (10.2 ns) | at 274 bytes, today's entries (6.5 ns) | **at the exact forms' ~66 bytes (an estimate; the ring at 96, 2.34 ns, as its bound)** |
| --- | --- | --- | --- |
| `jak-dock` (300) | 0 % of fields; mean 20.8, p95 23.0 ms | 97.0 %; mean 14.7, p95 16.0 | **100 %; mean 7.8, p95 8.2: full speed** |
| `jak-island` (300) | 0 %; 23.1, 34.6 | 93.3 %; 15.9, 23.3 | **100 %; 9.1, 12.3: full speed** |
| `jak-arena`'s fields (218) | 0 %; 41.9, 53.1 | 0 %; 28.4, 34.9 | **99.1 %; mean 13.8, p95 15.3: the p95 met, the mean 3 % over** |
| Jak's opening (4,380) | 1.7 %; 26.1, 45.4 | 59.9 %; 18.0, 30.2 | **99.9 %; 9.5, 15.2: full speed** |

A window is at full speed by §3.1 when its mean is at most 13.4 ms and its 95th-percentile field under 16.7 ms. The
percentages are the fields each under 16.7 ms.

**What binds.** In every field that misses, in every form, it is **thread A, the EE**. VU1's thread B averages 5.8,
9.1 and 12.6 ms on the three windows and never binds, but it holds only VU1's floats. At today's bytes the EE's fields
miss by a median of 1.29 to 2.67 times (up to 4.0) with §4.2's entries, and by 1.04 to 1.80 times with today's. At the
exact forms' bytes the five fields that miss are the arena's heaviest, by 1.15 to 1.18 times.

**What it says.** In this model the EE's code size decides the result. At the static prototype's bytes the handheld
misses on every window. At the exact forms the estimate puts nearly every field of Jak's opening under 16.7 ms, the
arena's mean 3 % over. The estimate rests on unbuilt forms and on the optimistic model above. It does not reopen R0.4,
closed by the exit rule, but it names the EE's bytes as the lever that matters most if the target is reconsidered.

### What the R0 stage leaves

The instruments, all off by default:
- the bench's `--census` and `--working`, the latter behind the `census` feature;
- `examples/ring.rs`;
- `float_lanes` with its four-lane, paired and AVX-512VL forms behind the `lanes` feature, and `examples/lanes.rs`
  with its mix mode;
- `emusen-gpu`'s `examples/memtypes.rs`.

The throwaway patches stay in the stage's scratch:
- the static-form byte counter and its purpose tags;
- VU1's pairing counter;
- the EE's transfer counter.

Disputes:
- D-1100, the VU pair counts, open;
- D-1101, the emitter's bytes, settled;
- D-1102, the perf map, a defect candidate;
- D-1103, the D1_CHCR poll, taken as core work on its own branch.

Exposure: the player's-BIOS writer of this stage ran the player's BIOS and Jak and Daxter, both on the desktop and on
the handheld. No firmware was listed, hashed or read, and no program counter, code word or operand was logged.

## The emitter's exact forms: the data cache's path (measured 2026-10-28)

*Decided 2026-10-28 by the tester: prototype R0.2's largest lever, the data cache's path, then the exits' write-back
and a shared entry stub, one at a time, as measurement prototypes for the byte face's emitter (EmuSen_TwoFaces.md). Each
must stay exact. The work stops early if a lever comes in at less than half its estimated saving.*

**The stop rule was met after the first lever.** The cache path is exact, but it saves **37.0 hot bytes a guest
instruction on the arena** (271.9 to 234.9) against an estimated saving of about 106 (about 130 down to about 24), so
the exits' write-back and the entry stub were not built. The disassembly shows where the remaining bytes are: the
spills and reloads of the entry's values, which the exits hold live (below). That is the evidence for the next lever.

### The instrument

The `forms` feature (off by default; without it `jit/forms_off.rs` compiles every lever out, so the default build's
code is unchanged):
- `EMUSEN_SAPPHIRE_FORMS=cache` turns the lever on for the blocks the dispatcher runs, so the bench's digests and
  `tests/blocks.rs` grade it;
- `EMUSEN_SAPPHIRE_FORMS=count` counts each compiled load and store that stays in its block, by its page's kind;
- `EMUSEN_SAPPHIRE_FORMS_BYTES=1` is R0.2's shadow compile, now kept in the tree: each block the dispatcher compiles is
  compiled again outside the module as today's emitter, C1, the static prototype, the static prototype with the
  levers, and today's with the levers. The hot straight line is counted by purpose, by R0's method;
- `EMUSEN_SAPPHIRE_FORMS_DUMP=FILE` writes the compiled code of the first eight blocks with three or more loads and
  stores, in the two static forms.

### The form

- **Dispatch:** the page's kind and, for a store, its writable bit tested as one masked compare.
- **A hit in either way:** each way's tag read as one little-endian word, its frame and V kept by a mask and compared
  with the page's frame and V set. Way 0 is taken first, as `DCache::hit` finds it. The line's address and its tag's
  address follow with no multiply.
- **The scratchpad** goes inline too, as a second fast path. Counted on the dock window with the lever
  (`EMUSEN_SAPPHIRE_FORMS=cache,count`, 300 fields), the compiled loads and stores that stayed in their blocks were
  35.3 M to the scratchpad, 45.4 M cached hits and 6,307 uncached. The scratchpad is common, and RDRAM past the caches
  is not.
- **Out of line:** everything else, in the existing path laid cold. That covers RDRAM past the caches, the accelerated
  pages, a page not writable, a miss and anything the interpreter takes. The counts of RDRAM's traffic past the caches
  (`Context::uncached`, `uncached_loads`) moved into that path, because a hit adds zero to them.
- **The store's D bit** is set as before, unless the line is locked.

### Exactness (measured)

| Check | Today's path | With the lever |
| --- | --- | --- |
| the arena window, 300 fields from the state at 9,000 | picture B77DC314C6AE837C, sound BCA8F00F4743A955, state 4ED52755B09F207E (R0's runs) | the same |
| the dock window, 300 fields from the state at 6,000 | 43D839015A0E4DA1, 2C225C73DB4D3F9D, E220B19BD26D8EBC (R0's runs) | the same |
| the dock's first 60 fields | 5A2BDD3521E043D7, D5F15509CCDD1709, 66581EBBF2A092D5 | the same |
| `tests/blocks.rs`, test-checked profile | 25 of 25 | 25 of 25 |

**Seeded faults**, each seeded into the lever alone, then reverted:

| Fault | Caught by |
| --- | --- |
| the wrong way: way 0's hit takes way 1's line | the 60 fields' picture and state |
| a miss taken as a hit in way 1 | the 60 fields' picture and state |
| V ignored: an invalid line with the frame hits | the 60 fields' picture and state |
| the D bit set on way 0's tag whichever way hit | the 60 fields' picture and state |
| a store to a cached page the TLB has not made writable run inline | not the 60 fields (no such store in them); `a_tlb_write_or_a_mode_change_is_seen_by_the_next_access` |
| an uncached load's bus wait dropped | not the 60 fields; six tests of `tests/blocks.rs`, among them `every_load_and_store_runs_compiled_on_every_region` and `the_stage_s7_memory_programs_leave_the_interpreters_machine` |

### The bytes (measured)

Host bytes a guest instruction; the hot straight line is the part laid before the first cold block.

| Window | Form | Hot | Cold | Total | Cache tags and dispatch, tagged (a memory instruction) |
| --- | --- | --- | --- | --- | --- |
| arena (23,251 blocks, 212,366 instructions, 39.5 % memory) | the static prototype | 271.9 | 272.0 | 544.0 | 80.3 (203) |
| | **with the cache path** | **234.9** | 306.4 | 541.2 | **53.2 (135)** |
| | today's emitter | 538.6 | 703.5 | 1,242.1 | — |
| | today's, with the cache path | 502.7 | 743.9 | 1,246.6 | 56.1 |
| dock (5,705 blocks, 62,319 instructions, 37.0 % memory) | the static prototype | 243.8 | 280.7 | 524.4 | 75.5 |
| | **with the cache path** | **209.6** | 313.9 | 523.5 | **49.4** |
| | today's emitter | 511.4 | 872.1 | 1,383.5 | 80.6 |
| | today's, with the cache path | 478.5 | 911.4 | 1,389.9 | 51.5 |

- **The saving:** 37.0 hot bytes a guest instruction on the arena and 34.2 on the dock (94 and 92 a memory
  instruction). The tagged cache bytes fell by 27 and 26, and the untagged ones by the rest. The estimate was about 106.
  The cold bytes rose by about 34, the moved paths.
- **A branchy way selection**, tried on the dock's first 60 fields (way 0 compared and taken first, way 1 only on its
  miss), took the tagged cache bytes from 54.3 to 47.4, but the untagged bytes rose from 84.2 to 92.2. The hot total was
  the same (252.9 against 253.5), so the select form was kept.
- **Why the estimate missed:** it priced the cache path's own instructions. About 135 bytes a memory instruction remain
  under that tag:
  - the two kinds' tests;
  - both ways' masked compares;
  - the line's address;
  - the scratchpad's path;
  - the D bit;
  - the moves of the three values the paths join with;

  and below them the reloads of values the register allocator has spilled.

### The ring at the bytes reached (measured)

R0's `ring` binary (`ring mix`, Jak's p90 working set), asked for 235 bytes. Its code came to 243.0 and 243.3 bytes a
guest instruction. Two rounds; the handheld on the charger (ACAD online 1 before and after), CPU only, scratch removed
afterwards.

| Entries | Desktop | Handheld | R0 at 283 bytes, desktop / handheld | Mark (handheld): Jak's opening / the arena's fields |
| --- | --- | --- | --- | --- |
| today's 2.87 instructions an entry | 4.60–4.74 ns a guest instruction | 5.56–5.61 | 5.20 / 6.49 | 2.75 / 2.28 |
| §4.2's 14.5 | 3.99–4.40 | 9.00–9.27 | 4.95–5.03 / 10.1–10.3 | 2.75 / 2.28 |

Both fail by about two to four times. The ring is still far from the mark of about 96 bytes, at which R0 measured 2.24
to 2.43 on the handheld.

### The next lever's evidence: the entry's values held to every exit (measured from the disassembly; the saving estimated)

*Analysis only; nothing was built or run for it. The figures are from `EMUSEN_SAPPHIRE_FORMS_DUMP` on the dock's first
60 fields: the first eight blocks with three or more loads and stores, 58 guest instructions, in the static prototype
with the cache path. Blocks that heavy in loads and stores are not the average. Instructions are sized by their x86-64
encodings (REX, opcode, ModRM, SIB for a stack address, disp8 or disp32). The hot blocks are those printed before the
first cold one, since Cranelift emits cold blocks last.*

**(a) What is held.** The entry loads about twenty values, the same in every block:
- **the issue model:** `last`, `floor`, the two halves of `last_writes`, and the five byte fields `open`,
  `group_ends`, `paired`, `last_category` and `last_pipe`;
- **the instructions issued**, Count, Compare, Random, the cycles Count last saw, and the issue group;
- **the memory context:** the translation cache's address and generation, the horizon, and the data cache's lines;
- **the context pointer itself**, which arrives in a caller-saved register.

Guest registers are loaded where an instruction first reads them, not at the entry.

**(b) Why they live to every exit.**
- **Every exit writes back the whole model.** `write_back` stores each of the model's fields, Random and the group at
  every exit, whether or not the block changed them. In the static prototype only `last` changes, so the other values
  stored are the entry's, and each must stay live to the block's last exit.
- **A block has many exits:** each load's and store's way out, the line checks, and the final exit.
- **Calls cross those live ranges.** The final exit, which is hot, calls the replay helper before its write-back.
  Cold paths call the translation cache's fill and `leave`. The System V convention lets a call clobber nine
  general-purpose registers, and leaves five callee-saved ones, which the block's own pointers fill. Cranelift's
  allocator therefore spills these values at the entry and reloads them at each use. It does the same for the guest
  registers and addresses that are live across the final exit's call.

**(c) What they cost, measured.** In the hot code of those blocks:
- **spills:** 186, 18.8 bytes a guest instruction;
- **reloads:** 323, 33.3 bytes;
- **the entry's loads:** 157, 19.7 bytes;
- **the prologue:** the frame and five saved registers, about 30 bytes a block.

Spills and reloads together are **52.1 bytes a guest instruction**, about a fifth of the hot code. With the entry's
loads they are 71.8.

The reloads by the slot read:
- 110 are of the model, the instructions issued, the cycles seen and the group (41 of them the five byte fields, 21
  `last`);
- 26 are of the data cache's lines and 14 of the horizon;
- 34 are of the context pointer;
- 99 are of values computed in the block.

**(d) The smallest exact change, with its saving estimated.**
- **Write back only what changed**, the rule the guest registers already follow: a model field, Random or the group is
  stored at an exit only when the block's value is not the very value the entry loaded. Unchanged means the memory
  already holds it, unless a helper wrote it in between. The replay, `leave` and the fill are not expected to write
  the EE's issue model, and that would have to be checked before relying on it. In the static prototype this makes
  ten of the entry's loads dead (every model field but `last`, Random and the group), so their spills and reloads go
  with them.
  - **Estimated saving:** about 40 hot bytes a guest instruction. That is their share of the entry's loads (about 10),
    of the spills (about 11) and of the reloads (about 9), and the final exit's ten stores (about 11).
  - **Where it applies:** in today's full-model emitter most of the model changes at every instruction, so it saves
    much less there.
- **The count of instructions issued** as one add to memory at the exit, in place of a load at the entry kept live to
  the exit: a few bytes more.
- **Not exact as it stands:** writing back before the replay call, so that nothing is live across it, would leave a
  rollback (kept < steps) with the EE's registers already overwritten. It needs an undo of its own.
- **To be checked:** a calling convention for the helpers that preserves more registers would keep the context pointer
  and the memory context in registers, worth about 5 to 8 bytes by the reload counts. Whether this Cranelift version
  offers one for indirect calls was not checked.

**What it says.** The bytes left are mostly not the cache path's or the translation's own instructions. They are the
cost of state the block holds from its entry for exits that may never be taken. An exit form that writes back only what
changed attacks the entry, the write-back and a fifth of the untagged bytes at once. That is the lever to build next if
the work resumes.

### Tests run

- `tests/blocks.rs`, test-checked profile: 25 of 25 on today's path; 25 of 25 with the lever, on its final form; the
  two faults' runs as above.
- The bench: the arena, the dock and the dock's first 60 fields, with and without the lever, digests as above.

## Option 1: Cranelift kept, the code handed to it reshaped (measured 2026-10-28)

*Decided 2026-10-28 by the tester: keep Cranelift, reshape the code the emitter hands it in three steps, each a form
behind the `forms` feature (the default build unchanged), each measured before the next, and find how close exact
compiled code comes to about 96 hot bytes a guest instruction. The steps: (1) write back only what the block changed;
(2) hold nothing across a helper call; (3) Cranelift's own features: cold blocks, a pinned register, a cheaper calling
convention. The stop rule: stop after a step that saves less than half its estimate or cannot stay exact, or when the
arena reaches about 96. These are byte-face emitter forms (`EmuSen_TwoFaces.md`); exactness is the requirement.*

**The result.** Step 1 met its estimate: about 31 to 34 hot bytes a guest instruction against about 40. Step 2 saved
7.7 to 8.0 against the 35.6 bytes of spills and reloads it targeted, so **the stop rule was met after step 2**. Step 3
had been built in the same tree and was measured in the same runs, and it saved 2.6 to 2.7 against an earlier estimate
of 5 to 8. The forms end at **180.6 hot bytes a guest instruction on the arena and 180.9 on the dock**, 1.9 times the
mark of 96. Every form is exact on every digest. On the handheld the ring at those bytes runs 3.93 to 4.00 ns a guest
instruction with today's entries, against marks of 2.75 for Jak's opening and 2.28 for the arena.

### The forms

`EMUSEN_SAPPHIRE_FORMS` takes the levers, comma-separated:
- `changed` (step 1);
- `reload` (step 2);
- `preserve` (step 3);
- `pinned` (step 3), for the shadow count only;
- `watch`, an instrument.

`EMUSEN_SAPPHIRE_FORMS_BYTES=1` now also compiles the static prototype with each step's levers in turn (`static+cache`,
`static+changed`, `static+reload`, `static+preserve`, `static+reload+pinned`, `static+preserve+pinned`). Every step can
then be compared on the same blocks in one run.

- **`changed`, write back only what changed.** An exit stores a model field, Random or the group only when the value the
  block holds for it is not the very value the entry loaded (the same SSA value, `emit.rs`'s `same`). Otherwise memory
  already holds it, and the entry's load goes dead. The guest registers already followed this rule. In today's
  full-model emitter every issue computes every field anew, so the rule stores nothing less there. In the static
  prototype only `last` changes, so ten entry loads and their stores go.
- **`reload`, nothing held across a helper call.** At the write-back, the instructions issued, Count, Compare and the
  cycles Count saw are read after the replay, not loaded at the entry and held to every exit. The translation cache's
  address and generation, the horizon and the data cache's line pointer are read at each load or store. Cranelift's
  alias analysis merges loads with no store or call between them, so after a call they are read again rather than
  spilled. In the static prototype `last` is the entry's plus a constant, read again where it is used (`Held::last_k`).
  It is off under the verifier, whose write-back after every instruction would make those memory values the last
  write-back's rather than the entry's.
- **`preserve`, a calling convention that clobbers nothing.** Cranelift 0.136.4 (the version in `Cargo.lock`) offers
  `CallConv::PreserveAll`: a call clobbers no register, the callee saves what it uses, and it may return no value
  (`cranelift-codegen` `isa/x64/abi.rs`, `get_regs_clobbered_by_call` and `compute_frame_layout`). The Rust helpers
  cannot be written in that convention. `forms::trampolines` compiles, once a process, one Cranelift function a helper
  (replay, `leave`, fill) in `preserve_all`, which calls the Rust helper and stores its answer in `Context::answer`. The
  dispatcher puts the trampolines in the context's helper slots, and the code reads the answer after the call. The
  verifier's hook keeps the System V call.
- **`pinned`, the context pointer in a reserved register.** Cranelift's shared flag `enable_pinned_reg` reserves r15 on
  x86-64 from allocation, with `get_pinned_reg` and `set_pinned_reg` to use it. With the flag, r15 is no longer
  callee-saved, so a block saves the caller's r15 at its entry and restores it at every return. The flag belongs to the
  module's ISA, which `emusen-jit` builds, so the form is compiled only by the shadow, against an ISA rebuilt from the
  module's flags with the pinned register on. It never runs.
- **Cold blocks** were already in use for every exit, rollback, miss and fill. Cranelift lays the cold blocks after the
  straight line, and the hot-tagged bytes laid past the first cold block are 1.0 a guest instruction, so nothing rare
  is left on the straight line to move.
- **`watch`**: every call of the replay, `leave`, the fill and the hook is checked to leave alone the issue model, COP0
  (but Cause's IP2 and IP3), the cycles Count saw, the group, the guest's registers and place, and the memory context.
  It reports the calls and those that changed anything (`forms watch`).

### What the helpers can write (from the code)

A compiled block calls four helpers, all in `src/blocks.rs`. Each reaches the machine only through `Run::sys`, and what
it borrows mutably decides what it can change:

| Helper | When it runs | What it can write | Why the skipped and reloaded fields are not among them |
| --- | --- | --- | --- |
| `replay` | every exit that ran a step, before the write-back | the board (`Board::run_through`, handed `&mut ee.spram` and nothing else of the core); `ee.int0`, `int1` and `cpcond0`; Cause's IP2 and IP3; the context's `steps` and `lines`; the dispatcher's `Run` | the board holds no reference to the core, so only the scratchpad can change through it; Cause is never skipped, and the write-back already reads it after the call |
| `leave` | a load or store leaving its block after a step | the context's `left`, `guard`, `base`, `logged` and `steps`; then `replay_steps` with `rewind`, which returns before the lines and Cause are touched | as `replay` |
| `fill` | a translation-cache miss (cold) | one entry of `Context::pages`, and `Run::fills` | it reads the core only through `Ee::data_page(&self)`; an unmapped page or a TLB miss comes back as `None` and is cached as kind `NONE`, with no exception raised and no COP0 register written |
| `hook` | the verifier only (`Options::verify`) | the board through `run_to`; the lines; Cause's IP2 and IP3 (`interrupt_pending`) | the verifier's machine is read through `&System`; `reload` is off under the verifier, and `changed` holds there because the write-back before each hook call stores every field the block has changed |

**So no helper, on any path, writes the fields the forms skip or read again.** That covers:
- the issue model, Random, Wired and the group;
- Count, Compare, the cycles Count saw and the instructions issued;
- the guest's registers and place;
- the translation cache's address and generation, the horizon, and the data cache's line pointer.

The types show it: during a call, the only mutable routes into `Ee` are `spram`, the three lines and Cause. **No rare
path needs accounting for.** None of the helpers raises an exception, refills the TLB or writes COP0 beyond those two
bits. Had one done so, the form would have stored the field after any call to that helper, whatever its value; none
needed it.

**Measured:** the watch saw 119,145,125 replays, 7,879,387 leaves and 91,971 fills on the arena, and 55,108,463,
1,949,174 and 54,237 on the dock, with no change in any of them. This is evidence beside the argument, not the proof.

**Rollback stays exact.** A rollback returns before any write-back, or through the write-back of the state after the
last load or store (`partial`). That write-back obeys the same rule: on the path to it, no write-back ran before, so
memory still holds every entry value. A field's held value never returns to the entry's SSA value once changed, since
every issue makes a new value or a constant.

### Exactness (measured)

The digests before the forms are this tree's own. They differ from the previous section's because D-1104 moved the VU's
products, which changes the picture and the state.

| Check | No form (`base`) | Step 1: `cache,changed` | Step 2: `+reload` | Step 3: `+preserve` |
| --- | --- | --- | --- | --- |
| the dock's first 60 fields (picture, sound, state) | 0F8671F57AB63C25, D5F15509CCDD1709, 818361BFE5D2DBF9 | the same | the same | the same |
| the dock, 300 fields from the state at 6,000 | 830E3FEFEE1BE6B1, 2C225C73DB4D3F9D, 9CF1BED1AA64EAEE | the same | (graded with step 3) | the same |
| the arena, 300 fields from the state at 9,000 | 611B58F7CFE67E8B, BCA8F00F4743A955, 8B710CA05FDDF3A0 | the same | (graded with step 3) | the same |
| `tests/blocks.rs`, test-checked profile, every lever on and `watch` | — | — | — | 25 of 25 |

**Seeded faults**, each built alone into the Release bench, run on the dock's first 60 fields with its step's levers,
and reverted:

| Step | Fault | Caught by |
| --- | --- | --- |
| 1 | a changed field taken as unchanged: `floor` compared with itself, so it is not written back | the state (EF5F76A3666814CF) |
| 1 | the premise broken: the replay writes Random between the entry and the exit | the watch (17,152,565 of 18,190,414 replays and 746,533 leaves changed); the state (D109ABD4C87E29BA), through the rollbacks that return with nothing written back |
| 2 | Count read again from Compare's register after the replay | the state (3B0986B5246E7E4E) |
| 2 | the horizon read again from the translation cache's pointer, so no load or store ever leaves at the board's next event | not the 60 fields; `tests/blocks.rs`'s `loads_and_stores_beside_the_dmac_leave_the_interpreters_machine` (24 of 25) |
| 3 | the trampolines not installed: the code calls the System V helpers as though they preserved every register | a segmentation fault in the first field |
| 3 | the answer read from `Context::steps` in place of `Context::answer` | the run stalled: no field past 6,000 in 139 s, where 16 s is the norm |

### The bytes (measured)

Host bytes a guest instruction on the hot straight line, by R0's method. All forms of a row come from one run, on the
same blocks: the arena, 13,471 blocks of 128,851 guest instructions (37.7 % memory); the dock, 4,697 blocks of 41,665
(37.5 %). Which blocks a run compiles depends on its timing, so the static prototype comes out at 257.9 here against the
previous section's 271.9. Compare within a row.

| Form | Arena hot | its saving | Arena cold | Dock hot | its saving |
| --- | --- | --- | --- | --- | --- |
| the static prototype | 257.9 | | 271.0 | 260.3 | |
| + the cache path (the previous section) | 222.2 | 35.7 | 304.2 | 225.2 | 35.1 |
| **step 1**, + `changed` | **191.0** | **31.2** | 232.0 | **191.5** | **33.7** |
| **step 2**, + `reload` | **183.3** | **7.7** | 224.8 | **183.5** | **8.0** |
| **step 3**, + `preserve` | **180.6** | **2.7** | 222.0 | **180.9** | **2.6** |
| step 2 with `pinned` in place of `preserve` | 184.5 | −1.2 against step 2 | 218.4 | 184.4 | −0.9 |
| step 3 with `pinned` too | 179.5 | 1.1 against step 3 | 217.8 | 179.5 | 1.4 |

Each step's first measurement had its own run. Step 1 alone, on 22,069 arena blocks, took the arena from 234.9 (the
previous section's cache path) to 200.9, and the dock from 209.6 to 182.9.

**By purpose at step 3 (arena; untagged bytes as measured, not apportioned):**

| Purpose | Static prototype | Step 3 |
| --- | --- | --- |
| cache tags and dispatch | 77.0 | 52.8 |
| untagged | 84.4 | 61.2 |
| exits' write-back | 40.4 | 27.6 |
| translation | 16.3 | 17.5 |
| horizon and line checks | 4.2 | 9.6 |
| address and alignment | 4.3 | 4.2 |
| guest registers | 3.3 | 2.7 |
| data access | 4.2 | 2.3 |
| entry | 22.4 | 1.5 |
| the instruction's work | 1.3 | 1.2 |
| timing | 0.2 | 0.1 |

**The spills, from the disassembly** (`EMUSEN_SAPPHIRE_FORMS_DUMP`; the first eight blocks with three or more loads and
stores on the dock's first 60 fields, 53 guest instructions, sized as the previous section sized them):

| Bytes a guest instruction | Static prototype | Step 1 | Step 3 |
| --- | --- | --- | --- |
| spills | 22.4 | 10.5 | 0.6 |
| reloads | 45.2 | 25.1 | 4.6 |
| the entry's loads | 21.8 | 10.4 | 3.2 |
| host instructions on the hot line, in all | 3,585 | 2,739 | 2,546 |

**Why step 2 missed.** The spills and reloads went, nearly all of them: 35.6 bytes a guest instruction to 5.2 on those
blocks. The hot code fell far less, because each reload from the stack became a load of the same size from the core or
the context. The value still has to come from memory after a call. Only the spill stores, and the entry's loads with
them, were saved. The horizon check shows it: under `reload` each load or store reads `last` and the horizon again, and
that tag rose from 4.2 to 9.6. Step 3's convention mattered little for the same reason. By step 2 almost nothing was
held across the hot call, so there was little for `preserve_all` to keep.

### The ring at the bytes reached (measured)

R0's `ring mix` (Jak's p90 working set), asked for sizes that land near each step's bytes. The handheld ran on the
charger (ACAD online 1 before and after), CPU only, through `systemd-run --user`, with its scratch removed afterwards.
The ring binary needs GLIBC_2.34 at most. ns a guest instruction, two rounds:

| Step (arena hot bytes) | Ring code | Desktop, today's 2.87 an entry | Desktop, §4.2's 14.5 | Handheld, today's | Handheld, §4.2's |
| --- | --- | --- | --- | --- | --- |
| the cache path (234.9) | 243.0 / 243.3 | 4.60–4.74 | 3.99–4.40 | 5.56–5.61 | 9.00–9.27 |
| step 1 (191.0) | 191.8 / 194.2 | 3.79–3.85 | 3.26–3.28 | 4.26–4.30 | 6.31–6.45 |
| step 2 (183.3) | 185.5 / 185.7 | 3.64–3.66 | 2.98–3.03 | 4.10–4.13 | 6.26–6.28 |
| step 3 (180.6) | 178.5 / 181.0 | 3.51–3.54 | 3.03–3.04 | 3.93–4.00 | 5.88–6.12 |
| the mark, handheld | | | | **2.75** (Jak's opening) / **2.28** (the arena's fields) | |

The cache path's row is the previous section's. At step 3 the handheld is 1.4 to 1.5 times the mark for Jak's opening,
and 1.7 to 1.8 times the arena's, with today's entries. R0 measured 2.24 to 2.43 there at 96 bytes.

### What is left, and whose limit it is (an assessment)

**Measured.** The forms reach 180.6 hot bytes a guest instruction with every result exact. The three steps together
saved 41.6 bytes from the cache path's 222.2. What remains is no longer state held for exits. On the eight heavy blocks
the hot line is 48.0 host instructions a guest instruction. The tags place the
bytes in the cache path and its dispatch (52.8), the exits' write-back (27.6), the translation (17.5) and the horizon
checks (9.6), with 61.2 untagged.

**The design's share.** Most of those bytes are the exact model's own work at every load and store:
- the translation cache's hash, tag load and compare, and the page's host address and kind;
- the kind's dispatch;
- both ways' tags and the scratchpad's second fast path;
- the D bit;
- the horizon compare.

A block's final exit also stores its place, Count, Cause, the instructions issued and the replay's guard, base and log
count. None of that is Cranelift's. A form that saves it changes the design:
- one translation a block for each base register and page;
- one tag check a line rather than an access;
- an exit stub shared between blocks, fed by a table of each block's constants.

Those are estimates for a redesign, not levers of this emitter.

**Cranelift's share, counted on the disassembly of the heavy blocks** (`overhead.py`; counts measured, bytes estimated
from x86-64 encodings). In each guest instruction's hot code:
- **register-to-register moves:** 6.7, about 20 bytes, from the allocator's splits and its fixed argument registers (the
  context pointer moved into `rdi` before each use as a base);
- **blocks holding only a jump:** 0.8, about 3 bytes, the edges of the fill's miss and the cache path's joins;
- **operands taken from the constant pool:** 2.1, about 6 bytes over a 32-bit immediate, for masks such as 0xFFFFF000
  and 0xFFFFFFFF;
- **block parameters zeroed with an `uninit` and an `xor`:** 1.1, about 3 bytes;
- **conditions materialised with `setcc` before a test:** 1.4, about 8 bytes with their `or` and `test`.

That is about 40 bytes a guest instruction on these memory-heavy blocks, and plausibly 25 to 30 on the average block,
by the share of memory instructions.

**What a purpose-built emitter would plausibly save (an estimate).** An emitter in the manner of Wasmtime's Winch would
write x86-64 directly through Cranelift's assembler crate (`cranelift-assembler-x64`, in the lock file). It would keep a
fixed assignment of the context, the core, the memory context and the guest registers the block names, and lay each
fast path as a straight sequence that branches on flags. The overheads above are its natural saving: about 25 to 30
bytes a guest instruction, which would take 180.6 to about 150 to 155. The remaining distance to 96 is the design's per
access and per exit work. An emitter's quality does not reach it.

**So option 1 cannot get near 96.** Of the 84.6 bytes between step 3 and the mark:
- about 25 to 30 are Cranelift's code generation (estimated);
- the rest, about 55 to 60, is the exact design's per-access and per-exit work (by the tags, measured).

Reaching about 96 would need both a purpose-built emitter and the design changes above. Each is an estimate until it is
built, and the second changes what a block is, not how it is compiled.

### Tests run

- **The bench, before the forms and with them:** on the dock's first 60 fields, `base`, step 1, step 2 and step 3; on
  the dock and the arena, `base`, step 1 and step 3. Digests as above.
- **`tests/blocks.rs`, test-checked profile:** 25 of 25 with `cache,changed,reload,preserve,watch`; 24 of 25 with the
  horizon fault, as above.
- **The six seeded faults** as above, each on its own Release build, and reverted.
- **The ring:** two rounds at three sizes on the desktop, and on the handheld on the charger.

## The exact design's work a block: one translation a page, one tag check a line, a shared exit (2026-10-28)

*Decided 2026-10-28 by the tester: after option 1, cut the per-access and per-exit work its assessment measured as the
larger part of what remains, in three levers behind the `forms` feature (the default build unchanged), measured one at
a time in this order: (1) one translation a page a block; (2) one cache-tag check a line a block; (3) one shared exit
stub. The stop rule: each lever's saving estimated in writing before it is built; stop after a lever that saves less
than half its estimate or cannot be proven exact, or when the arena reaches about 96. These are byte-face emitter forms
(`EmuSen_TwoFaces.md`); exactness is the requirement.*

### The estimates, set before each lever was built

The base is option 1's step 3 on the arena: 180.6 hot bytes a guest instruction, 37.7 % of guest instructions loads and
stores, so about 479 hot bytes a load or store, of which the translation's tag holds 17.5 a guest instruction (about 46
a load or store) and the cache path and its dispatch 52.8 (about 140).

**Lever 1, one translation a page.** Assumptions: about 45 % of a block's loads and stores have an earlier one in the
same block on the same base register unchanged (stack frames, a structure's fields); each such access drops the
translation (about 46 bytes, and about 10 of the untagged bytes with it) and pays a guard of about 14 (an exclusive or, a
compare and a branch to the full path laid cold). **Estimated saving: about 7 hot bytes a guest instruction** (0.377 ×
0.45 × 42), a range of 5 to 10.
