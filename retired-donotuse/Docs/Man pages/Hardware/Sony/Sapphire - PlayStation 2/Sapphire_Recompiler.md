# Sapphire: the EE recompiler's record, from S37

The build record of Sapphire's EE recompiler from stage S37 on, continuing `Sapphire_Native.md`, which grew past the size
a page in the tree may have. Its sections keep that page's numbering: a section number with no page named is
`Sapphire_Native.md`'s (§44 is S36's recompiler, §47 S33's idle skipping), or this page's own from §48 (§48.15, the
loads' and stores' exits shared, continues it). The plan is
`Sapphire_Plan.md` (§5.12, §5.13, §7's S36 to S39 rows); disputes, were any raised, would be in `Sapphire_Disputes.md`.

## 48. Stage S37, the EE recompiler: loads and stores, with the RDRAM, scratchpad and data cache fast paths (2026-10-26)

*Numbered §48, after S36's §44, the stages beside it (S40's §45, the GS's fast paths' §46, S33's §47, S43's §50) taking
the numbers around it; its disputes would be numbered from D-720, and it raised none: every oracle below agreed with the
interpreter, and the stage reads nothing of the hardware the interpreter does not already implement. Built on WiseMan
`079129f7f` and `86b6a565d`, rebased onto `26888b88e` (S33), `63f380138` (D-860) and `4bfd76232` (S43), each oracle run
again on the last.*

### 48.1 In one paragraph

The plan's §7 row S37, on the tester's go of 2026-10-26 (speed first, the PlayStation 2 ahead of other work), under the
rule decided the same day (cycle accuracy where it is realistic, byte accuracy the floor) and with this stage held to
exactness. S36's blocks ended at every load and store, so the EE's memory path stayed the interpreter's. **A block now
holds the loads and stores of every width** (LB to LQ, SB to SQ, the unaligned LWL, LWR, LDL, LDR, SWL, SWR, SDL and SDR)
and PREF, each lowered inline: its address translated through a **translation cache** of 4 KiB pages keyed by the TLB's
contents, the mode, the ASID and the configuration (§48.3), then **the scratchpad and RDRAM reached in host memory
directly, a cached page through the data cache's own tags and lines on a hit, and an uncached load with its bus wait**
(§48.4). Everything else (a misaligned address, a register, the boot ROM, an accelerated load, a page that would fault, a
data cache miss) leaves the block before the instruction, and the interpreter runs it, raising exactly what it raises.
**The board's replay at a block's exit now covers its accesses** (§48.5): the DMAC is the only unit that reads or writes
memory or the scratchpad, so a block runs no access after an instruction whose cycles reach the board's next event, and
the replay stops before the DMAC would run between two of a block's accesses; a rollback then undoes the block's stores
from a log, or, past its last access, keeps the state after it. A block runs to the first line the instruction cache does
not hold, and is shaped past it from RDRAM, which longer blocks needed (§48.4). **The gate is met** (§48.7): the
verifier after every instruction and every block, 240 random programs now with loads and stores, eight mechanism tests
new to this stage beside S36's fourteen, the S6 and S7 programs, ps2autotests 52 of 55 on both tiers with every text
identical, the OSDSYS boot and Jak and Daxter's program equal to the interpreter in every picture, the sound and the
state, and from a state in an empty cache, on every base the stage was rebased onto. Speed in §48.8: Jak and Daxter's
loading 1.4 to 2 times S36's on every measurement, the boot and its language screen within the noise, and the memory card
screen 0.94 of S36's on the last base, its cause measured and named (code compiled while the screen runs) and part of it
recovered by shorter blocks, not taken. C2's timing part is not built: a miss leaves the block (§48.6).

### 48.2 What was built

| Module | What changed | Lines |
| --- | --- | --- |
| `src/ee/jit/shape.rs` | the memory class: 21 loads and stores and PREF compiled like plain instructions, in a line or a delay slot; the word's class memoised | +65 |
| `src/ee/jit/emit.rs` | the access (§48.4): translation, the data cache's hit, the scratchpad, RDRAM, the uncached load's bus wait in the issue model; the unaligned merges; the undo log; the exits' guard and the state after each access written back on a rollback past it; the exits at a line the cache does not hold; GPRs' upper halves held for LQ and SQ; the exits laid out cold | +697, 1,841 |
| `src/ee/jit/mod.rs` | the context's translation cache, its generation, the board's next event, the guard, the undo log, the line limit, the counts; `Options::memory` and `blocking`; the entry refused while a data breakpoint is enabled; `held_words` and `cached_words` past a missing line | +138 |
| `src/blocks.rs` | the translation cache's fill and generations; the rollbacks of §48.5; the line limit at the entry; the block rest; the traffic and steps counted; the data cache and the scratchpad in the verifier's comparison | +305 |
| `src/board.rs` | `run_to` as its body and its last act (`run_to_held`, `take_wake`, S33's `quick_wake` on the quick path), the same calls; `run_through` guarded (§48.5); `Board::finish`, `next_event` public | +87 |
| `src/ee/memory.rs`, `mod.rs`, `ops.rs`, `timing.rs` | `Ee::data_page`, the translation a data access makes, with no effect; the TLB's stamp, new at every write and load; the bus's free cycle's place | +44 |
| `examples/bench.rs`, `examples/s33.rs` | the dispatcher's counts printed with `--blocks` and by `s33 time` | +11 |
| `tests/blocks.rs` | the random programs' loads and stores; eight mechanism tests (§48.7) | +424 |

`emusen-jit` is unchanged (its interface 1.0.0 served blocks with helper calls, joins and cold blocks as it served S36's),
so MarsRT was not in the blast radius. The whole-machine oracles use `examples/s36.rs` (`state`), `examples/s33.rs`
(`verify`, `time`, with idle skipping on), S32's bench (digests) and `examples/s10.rs`; `EMUSEN_SAPPHIRE_MEMORY=0` shapes
blocks as S36 did, for measurement, and changes no result.

### 48.3 The translation cache

A load or store's address is translated by the interpreter's `translate` and `data_mode` on every access: the segments,
the TLB's 48 entries (behind a one-entry memo), the page's cache mode and Config.DCE. S32 measured that path at 8 to 21 %
of a frame (§43.5). The blocks keep its answers per 4 KiB page instead, in a direct-mapped table of 4,096 entries in the
dispatcher (`jit::Page`: a tag, the page's first host byte, and its meaning: nothing fast, the scratchpad, a cached
physical page with its frame number, or RDRAM uncached; and whether a store may be made, the TLB's D bit). The code
compares the entry's tag with `gen1 | vaddr & !0xFFF`, at the entry `jit::page_index` gives (the page number's low twelve
bits mixed with its top eight, so that kuseg, kseg0, kseg1 and the TLB's uncached and accelerated windows onto the same
memory do not take one entry in turn: before the mix, the DMAC test's three windows refilled 3,416 times in 40,000
steps, after it 4); a page not found calls the fill (`blocks::fill`), which runs
`Ee::data_page`, the interpreter's translation without its one effect (the TLB-miss event, which counts only while CTE
is set, and blocks are never entered then), and caches whatever it finds, a page that would fault included, so that a
polled register costs no second call.

**What keys it.** A page's meaning depends on the TLB's entries, EntryHi's ASID, the operating mode as a data access sees
it (user, supervisor, kernel, or kernel at level 2 where kuseg is unmapped), Config.K0 and DCE, and where RDRAM lies. The
TLB carries a stamp (`Ee::tlb_stamp`), drawn from a process-wide counter at every TLBWI and TLBWR, every state loaded and
every core made, so no two contents of any TLB share one; the dispatcher compares the key before each block and moves
the table to a generation of its own when it differs. A generation is a 32-bit number in each tag, so moving is
immediate and the table is cleared only when the number wraps. **The last four keys keep their generations**: the first
version made a new generation at every change, and the OSDSYS boot's interrupt handlers (kernel mode) and the programs
around them (user mode) moved it 130,000 times a frame, refilling every page each time (3.9 million generations and 5.8
million fills over 30 frames under the verifier); with the keys remembered the same 30 frames made 21 generations and
8,944 fills, and the boot's frame lost the cost of the refills. Nothing inside a block can change the key: a block holds no
COP0 instruction and takes no exception.

### 48.4 The emitter's access

Each load, store or PREF is, in order: **the board's next event** (below, §48.5); the effective address, bits 31..0 of
base plus offset; **alignment** (a natural access whose low bits are set leaves, and the interpreter raises the address
error; LQ and SQ drop their low four bits; the unaligned forms take the aligned unit); **the page** (§48.3); then by the
page's meaning:

- **A cached page** (C 3, or K0 3, with DCE set): the set is virtual address bits 11..6 and the two ways' tags are read
  from the core's own `DCache` and compared with the page's frame; a hit in way 0 is taken first, as `DCache::hit` takes
  it. The access is made in the line itself, which is where the interpreter's would be made: a store dirties the line
  unless it is locked (§6.3.7) and never reaches RDRAM. A miss leaves the block, and the interpreter's access refills the
  line, writes the victim back and charges both; a store without the D bit leaves, to raise TLB Modified.
- **The scratchpad** (an S entry): the 16 KiB in host memory, loads and stores.
- **RDRAM past the caches** (kseg1, C 2, the TLB's uncached and accelerated pages, or DCE clear), below 32 MiB: made in
  RDRAM itself and counted into the board's traffic as `Bus::read` and `Bus::write` count them. A load waits for the bus
  as `read` charges it (`timing::UNCACHED`, two cycles): the issue model's data wait, which every other access a block
  makes leaves at 0, becomes a run-time value for the load (the bus's free cycle `mem_free` read and set, Config.NBE's
  blocking stall folded in as an option of the block), so the timing stays `Timing::issue`'s. An accelerated page's load
  leaves (the UCAB holds its own line), and so do the unaligned stores, since the interpreter reads the unit over the bus
  first.
- **Anything else** (a register at 10000000h and up, the boot ROM, an unpopulated or extended address, K0 0's
  write-through, an unmapped or invalid page, an address the mode may not reach): the block leaves before it.

Every way of leaving is one exit before the instruction, the core at that instruction, with nothing of it done, laid
out after the block's straight line (Cranelift's cold blocks); each reason is counted (`Stats::left`: at the event,
misaligned, the page, a miss, an accelerated load). **A block whose first instruction's access leaves at once**, as a
loop polling a register does at every turn, would cost its entry's checks for nothing at every turn: after four such
entries in a row the dispatcher leaves the next 64 to the interpreter without them, as it passes over a refused block.
The unaligned forms merge as `ops` does, the offset in the unit a run-time shift (LWR's and LDR's whole-unit case selected, since a shift by the unit's width
wraps in the host). **Timing**: a hit, the scratchpad and a store past the caches are what §2.12's table charges, the load
and store rows with no data wait, exactly as the interpreter charges them, so the issue model of §44.4 stands unchanged
with the memory instructions' rows made constants, and an uncached load's wait the one run-time value; every other access
that waits for memory (a refill, a write-back, the UCAB's line) is the interpreter's.

**The caches' model kept exact.** The data cache is the interpreter's own storage, read and written in place, its tags
compared as `hit` compares them, a store's D bit set as `write_merge` sets it, nothing refilled or replaced by a block.
Any store disables the UCAB, as `write_merge` does first (the block's write-back clears its valid bit, as SYNC.L's
did at S36). The instruction cache is untouched by a store, so code written into a compiled block's lines is seen
exactly when the interpreter sees it: after CACHE invalidates the line, when the next entry finds the line missing or
different (§44.3); a block that stores over its own words runs on with the words the cache holds, as the fetches would.
**Data breakpoints**: a block is not entered while BPC's DRE or DWE is set, so the interpreter judges every access.

**A block and the instruction cache.** S36 entered a block only when the cache held every line of it, and shaped a
new block only from lines it held, so that a block was what the interpreter's fetches would read (§44.3). Blocks that run
through loads and stores are longer, span more lines, and so were refused far more often: on Jak and Daxter's memory card
screen, with idle skipping, 2.76 million entries in 500 frames were refused for a line the cache did not hold and 1.72
million new blocks waited for one, against 0.88 and 0.65 million with S36's shapes, and the interpreter ran 159 million
steps there against 116 (measured 2026-10-26, frames 400 to 900 of the same binary, `EMUSEN_SAPPHIRE_MEMORY` 1 and 0).
Two changes, both exact, give the block back: **a block runs to the first line the cache does not hold** (the entry
counts the words the cache holds, `Ee::held_words`, and the code leaves at the first instruction of each later line when
`Context::limit` says so, an exit made only where a line begins outside a delay slot); and **a block is shaped past a line
the cache does not hold from RDRAM**, as the line's refill would read it now (`cached_words`), since the entry compares
every line it runs with the cache and shapes the block again if a refilled line differs. On the same window the refusals
fell to 1.61 million in the whole run, the waits to none, and the window's time matched S36's shapes (112.6 against 112.3
ms a frame). A word's class (plain, branch, memory, other) is memoised by word on the thread that asks, since
the dispatcher shapes a block again at every entry that waits for a line, and the decode and the name's lookup showed in
the profile of that screen.

**Registers**: LQ writes and SQ reads all 128 bits, so the block holds GPRs' upper halves too, loaded when first named
and written back when written; every other load writes the low 64 bits, keeping the upper, as `Ee::set` does.

### 48.5 The board, the DMAC and rollbacks

S36's replay (§44.5) was exact because nothing a block computed was read by the board and nothing the board did in
between was read by the block. Loads and stores break the first half of that unless they are ordered with the one unit
that reaches memory: **the DMAC** (every other unit, the VIFs, the GIF, the GS, the VUs, the IPU and the IOP, is fed
by the DMAC or keeps memory of its own; the board's `run_to` hands the scratchpad to `step_dmac` alone). In the
interpreter the DMAC's steps during `run_to(t_i)`, the board brought to instruction i's cycles, see the stores of
instructions 0 to i and none after, and a load of instruction k > i sees what they wrote. So a block is exact if **no
DMAC step falls in `run_to(t_i)` for any i before its last access**, and two mechanisms make it so:

- **The board's next event at the entry.** `run_to(t)` runs no event before `Board::next_event`, and while the GIF, the
  VIFs and the VUs make no new request the DMAC steps only at an event or when the IOP's side wakes it through the SIF.
  The dispatcher hands the next event to the block (`Context::horizon`); before each access the block compares the
  cycles of the instruction before with it and leaves if they reach it, so its accesses all precede the event's time.
- **The guarded replay.** Each exit tells the replay the index of its last access (`Context::guard`), and
  `Board::run_through` stops before running a time within it where an event has come due after all (a busy unit's
  request moved the event earlier), or before taking a wake the IOP's side has just asked for there. To stop before the
  wake, `run_to` is now its body and its last act (`run_to_held`, `take_wake`, the latter S33's `quick_wake` on the quick
  path), which together make the same calls in the same order as before; the stop keeps the wake for `Board::finish`,
  which takes it as the interrupted call would have.

**Rollbacks.** A replay that stops (for an interrupt as at S36, or for the DMAC) leaves fewer instructions standing than
the block ran, and its accesses must not be run twice over a DMAC step. Two cases, decided by where the stop falls:

- **Short of the block's last access**: no DMAC step has run within the block, so memory holds only the block's own
  stores. Each store logs the 16 bytes of its quadword and the 8 of its line's tag before it writes (`Context::log`, two
  entries a store, at offsets fixed when the block is compiled); the dispatcher undoes them last first, and the
  interpreter runs the instructions that stand from the block's start, with the lines each saw, then the held wake. A
  load run again reads what it first read, a store writes what it first wrote, and a hit is still a hit.
- **Past the block's last access**: the DMAC may have run after it, so nothing is undone; the code writes back the
  state after that access (kept for each access, its write-back emitted once, as a cold block the exits past it reach),
  and the interpreter runs the register-only instructions from there.

**A whole-block guard was tried and dropped** (measured 2026-10-26): keeping the DMAC out of every one of a block's
times once it had made an access removes the second case and its write-backs, but turns every DMAC step in a block's
tail into a rollback. On Jak and Daxter's first 400 frames it made 311,129 rollbacks against 23,465, took the code
from 94 to 83 MB, and in three interleaved rounds OSDSYS's language screen ran at 0.76 times S36's speed (medians 307.7
against 234.2 ms a frame) where the guard to the last access had run at 1.09 times; the guard to the last access is the
one kept.

### 48.6 C2's timing part, and the switch

C2 (`Sapphire_Plan.md` §5.13) would model the caches' coherency without their contents' timing. Its timing part could be
an emitter switch only where the emitter charges a miss's time, and **at S37 it charges none: a miss leaves the block
and the interpreter refills the line** (a refill inside a block would need the undo log to hold the victim line and
RDRAM's 64 bytes). So C2's timing part belongs to the interpreter's memory path, not to S37's emitter, and is not built
here; the misses it would act on are counted (§48.8: 15 million over Jak and Daxter's 900 frames, 2.7 million over the
boot's 760). **`EMUSEN_SAPPHIRE_MEMORY=0`** (`Blocks::memory`) shapes blocks as S36 did, every load and store left to
the interpreter; it changes no result and exists for measurement.

### 48.7 The gate (measured 2026-10-26)

On the desktop beside other stages' work; the player's Americas BIOS `ps2-0230a-20080220.bin` executed through the
referees' link to it, never read; ps2autotests from `~/.cache/emusen/probe/sapphire/ps2autotests/`, Jak and Daxter read
in place from `~/.cache/emusen/probe/sapphire/discs/`. Every whole-machine oracle ran on the first base and again on the
last (`4bfd76232`, with idle skipping, D-860's interrupt delay and the GS's workers beneath); the figures are the last's.

| Oracle (`Mars_Native.md` §5.8.5) | Run | Result |
| --- | --- | --- |
| 1. The interpreter stepped beside the blocks, compared after every instruction | `the_verifier_steps_beside_every_compiled_instruction_and_finds_nothing`, `random_programs_under_the_verifier_find_nothing` (20 random programs, `Verify::Steps` and `Verify::Blocks`, 10,000 steps each), `loads_and_stores_beside_the_dmac…` under `Verify::Steps`; the comparison now holds the data cache's tags and lines and the scratchpad | held |
| | `s33 verify boot` and `s33 verify disc` (the recompiler with idle skipping, the interpreter stepped through every block and every skipped turn), 30 frames each, quiet | held: 220,863,488 and 231,997,440 steps compared, the whole states equal at the end |
| 2. Random programs, seeded | seeds 0 to 159 and 1000 to 1079 as at S36, 24,000 steps each, the generator now writing a load, store or PREF of every width in a fifth of its segments' slots: through gp cached over three data caches' span, through s7 past the caches (kseg1) over the same memory, in the scratchpad through sp, and now and then misaligned, unmapped (a TLB refill, the handler now at 80000000h too), a timer's count or the same memory through kseg0; compared every 400 steps, whole at the end | equal at every comparison and whole: 3,770,517 of 3,836,815 and 1,873,339 of 1,918,580 steps compiled; 1,431 and 739 rollbacks, 25 and 14 past the last access; loads and stores left at the event 4 and 6 times, misaligned 113 and 8, on the page 1,378 and 707, missing the cache 391 and 238 |
| 3. The mechanisms one at a time | S36's fourteen tests, now running loads and stores compiled, D-780's test of narrow stores to registers, and eight new, below | 23 of 23 |
| 4. ps2autotests' 55 EE programs | `s10` on each tier | 52 of 55 on both, the same three as §36.5 (`cpu/ee_fpu/sqrt`, `dma/vif/intstall`, `vu/lower/efu`); all 55 texts identical byte for byte between the tiers, on every base |
| | the project's memory programs | the S6 programs (every load and store at every byte of a quadword, `memory`, and the address errors of `exceptions`) and the S7 programs (`s7-tlb`, `s7-spr`, `s7-dcache`, `s7-icache`, `s7-counters`, `s7-debug`, `s7-buserror`) equal to the interpreter and to the manuals' records on the blocks' machine |
| 5. The bench workloads, frame by frame | `examples/bench.rs` interpreted and with `--blocks`: the digests of every picture, of the sound and of the final state | the OSDSYS boot, 760 frames: `50F06A02F1B76860`, `2EE7B3D10DF3C401`, state `2051C6BB42228C53` on both; Jak and Daxter, 900 frames: `7EEC08D4D07DB621`, `20BB66B5E91347C9`, state `83E7D9CAFA075D14` on both (the pictures and sound S32's own, §43.2; the states' digests follow the state's layout on each base, equal between the tiers on each); the same in all 24 timed runs of §48.8 |
| 6. A state loaded into an empty cache | `a_state_saved_between_blocks_loads_into_an_empty_cache_and_runs_alike` (three random programs, now with memory); `s36 state` on the boot (state at frame 400, 100 frames on) and Jak and Daxter (300, 100) | equal throughout: 0 pictures, 0 sound, 0 states differing on both |

The mechanisms new at S37, each a program of the project's own:

| Test | What it holds |
| --- | --- |
| `every_load_and_store_runs_compiled_on_every_region` | every load, store and PREF at every byte its alignment allows, through kuseg, kseg0, kseg1, the TLB's uncached and accelerated pages and the scratchpad, forty passes; every one of the 22 compiled |
| `the_stage_s7_memory_programs_leave_the_interpreters_machine` | the seven S7 programs on the blocks' machine, equal to the interpreter and to their records |
| `loads_and_stores_beside_the_dmac_leave_the_interpreters_machine` | the DMAC moving RDRAM to the scratchpad and back, cycle stealing in 128-byte units, while a loop reads and writes the same bytes through the cache, past it and in the scratchpad; accesses left at the DMAC's events; also under `Verify::Steps` |
| `a_tlb_write_or_a_mode_change_is_seen_by_the_next_access` | a page at 04000000h of TLB entry 40 mapped, remapped, made clean, made invalid and its ASID changed between accesses; the scratch area through kseg0 in kernel mode, then in user mode, where it raises; 30 TLB Modified, 60 TLB Invalid or refill and 587 address errors taken alike |
| `the_board_stops_a_guarded_run_before_the_dmac_runs` | a transfer due at once: the guarded run stands before the event's time, the unguarded one runs every time |
| `a_rollback_leaves_memory_as_the_instructions_that_stand_left_it` | a loop of stores to a cached line and the scratchpad, Count's interrupt falling inside its blocks, a handler reading what was stored last |
| `a_store_into_a_clean_line_dirties_it_and_reaches_memory_when_it_is_replaced` | a line refilled by a load, a block's store into it, the line replaced by two others of its set and RDRAM read past the caches |
| `a_block_runs_to_the_first_line_the_cache_does_not_hold` | a loop over four lines invalidating its third at every pass: 168 entries ran to that line |

The S6 programs' test shapes its blocks from the lines the cache holds (`Blocks::wait_lines` off, code run once), as at
S36; every one of the 121 mnemonics a block compiles is compiled by them.

### 48.8 Speed (measured 2026-10-26)

**The method.** Every timed run alone among this stage's own jobs, in a scope at CPU weight 100, while other stages'
builds and timed series went on beside at their own weights; the load average (one, five and fifteen minutes) read
before and after each run, given beside each figure, since S32 found load to cost these runs far more than its count
suggests (§43.5). Each frame's `advance` timed; the windows are the OSDSYS boot's frames 0 to 690 and its language
screen (690 to 760), Jak and Daxter's loading (8 to 400) and its memory card screen (400 to 900). The S36 tier is each
base's own build; S37's is this branch's, or the same binary with `EMUSEN_SAPPHIRE_MEMORY=0`, which shapes blocks as S36
did. Every timed run gave the interpreter's digests (§48.7, oracle 5).

**On the first base** (`86b6a565d`, no idle skipping, S32's bench with `--blocks`), three rounds each, the order of the
builds alternated; medians of three (ms a frame) with the spread of the three, one-minute loads of 4.7 to 21.0:

| Window | S36 | S37, first design | ratio | S36 | S37, whole-block guard | ratio |
| --- | --- | --- | --- | --- | --- | --- |
| The boot, 0-760 | 451.4 (26 %) | 492.6 (22 %) | 0.92 | 482.4 (10 %) | 417.0 (26 %) | 1.16 |
| Its language screen, 690-760 | 261.0 (17 %) | 239.9 (9 %) | **1.09** | 234.2 (29 %) | 307.7 (47 %) | **0.76** |
| Jak and Daxter loading, 8-400 | 217.1 (1 %) | 159.3 (32 %) | **1.36** | 294.3 (49 %) | 150.2 (14 %) | **1.96** |
| Its memory card screen, 400-900 | 706.9 (45 %) | 803.5 (64 %) | **0.88** | 911.1 (36 %) | 805.7 (16 %) | **1.13** |

The first design is §48.4 and §48.5 without the uncached loads, the page index's mix, the cold exits, the block rest and
the line limit; the second added the first three with the whole-block guard of §48.5, which the language screen then
showed to cost more than it saved. Over these runs 97 % of the boot's steps ran compiled (S36: 95 %) and 90 % of Jak and
Daxter's (S36: 80 %); the interpreter's steps fell from 372 to 208 million and from 1,140 to 577 million.

**On top of S33** (`26888b88e`, idle skipping on, S33's runner `s33 time … recompiler on`), one interleaved round, loads
2.1 to 6.8: the boot 0-690 189.9 to 176.2 ms a frame (1.08), the language screen 132.4 to 130.8 (1.01), Jak and Daxter's
loading 69.0 to 48.9 (1.41), its memory card screen 105.0 to 111.3 (**0.94**).

**Why the memory card screen was slower, first cause: the instruction cache's lines.** The same binary with and without
loads and stores in blocks, on that screen (frames 400 to 900, loads near 6): 103.1 against 98.5 ms; the dispatcher's
counts over the window showed entries refused for a line the cache did not hold (2.76 against 0.88 million) and new
blocks waiting for one (1.72 against 0.65 million), and the interpreter running 159 against 116 million steps. The line
limit and the shaping past a missing line (§48.4) took the window to 112.6 against 112.3 ms in the same pairing.

**On the last base** (`4bfd76232`: S33, D-860's interrupt delay and S43's GS workers beneath, the workers at their
default count, so these numbers include them), S33's runner. One interleaved round against WiseMan's own build at loads
of 10 to 14, with this stage's S6 test running beside the last two runs: the boot 0-690 223.1 against S37's 250.9, the
language screen 196.4 against 172.2, Jak and Daxter's loading 76.1 against 63.9, its memory card screen 111.6 against
132.5 (its 95th percentile 183 ms against 126, a run disturbed by load). Then, alone and at loads of 3.4 to 8.0, the
same binary with and without loads and stores in blocks, two interleaved pairs (with / without, ms a frame):

| Window | with, pairs 1 and 2 | without | ratio of means |
| --- | --- | --- | --- |
| The boot, 0-690 | 229.1, 179.7 | 204.1, 198.2 | 0.98 |
| Its language screen, 690-760 | 138.1, 132.1 | 142.7, 132.6 | 1.02 |
| Jak and Daxter loading, 8-400 | 51.3, 51.7 | 76.0, 83.4 | **1.55** |
| Its memory card screen, 400-900 | 114.2, 123.2 | 108.0, 115.7 | **0.94** |

**Second cause, on the memory card screen: code made while it runs.** Over its 500 frames the screen enters its code at
new places all along: with loads and stores in blocks 16,646 new blocks were compiled there, 202 MB of code, against
8,964 and 13 MB without; each new block is interpreted for its first 64 entries and while it waits behind the others in
the compiler's queue, and the interpreter ran 171 against 116 million steps of the window, the blocks 54 against 109
million. A block is larger because each load or store carries its own exit, with the write-back of everything the block
holds, and the write-back of the state after it for a rollback past it (§48.5); and there are more blocks, because every
place an access or a line sends a block out is a new entry. **Shorter blocks recover part of it** (`EMUSEN_SAPPHIRE_LONGEST`,
the same pairing, loads 3.4 to 5.3): at 16 instructions the screen ran at 108.3 and 106.6 ms against 104.4 and 102.4
without (0.96), at 32 at 112.3, at 64 at 110.4, Jak and Daxter's loading unchanged (49.2 and 48.6 at 16, 49.5 at 64);
not enough to take, and the boot not measured at 16, so the length stays 64. What would recover it is fewer bytes an
access: an exit shared by the accesses of a block, or no separate write-back for the rollback past the last access, each
a change to the emitter's exits rather than to its accesses; recorded as open (§48.12).

**What the blocks left to the interpreter** (S37's counts, the first base, Jak and Daxter's 900 frames): 752 million loads
and stores left on a page the code does not serve before uncached loads were compiled, 389 million after (a device
register polled, about 780,000 a frame on the memory card screen, which idle skipping (S33) passes when it can prove the
wait), 8 to 15 million data cache misses, 4.2 to 4.9 million at the board's next event, none misaligned; the guard's stops
30,990 in 900 frames, 3.67 million with the whole-block guard.

### 48.9 Seeded faults (2026-10-26)

Each made alone on a copy of the crate built apart (on the tree rebased onto `26888b88e`), ten tests of `tests/blocks.rs`
run (the random programs, the eight mechanisms of §48.7, the S6 and S7 programs, the state test), the source restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | a rollback leaves the block's stores in memory | 1: the rollback beside the handler's reads |
| F2 | a store hitting a cached line leaves its D bit clear | 2: the store into a clean line, the TLB and mode |
| F3 | the DMAC unguarded: no check of the next event and no guard in the replay | 2: beside the DMAC, every region |
| F4 | the replay's guard ignored | 1: the guarded board |
| F5 | the translation cache's key without the mode | 2: the TLB and mode, the S6 programs |
| F6 | LWR at a word's first byte keeps the register's upper half | 2: the random programs, the S6 programs |
| F7 | an uncached load without its bus wait | 4: the store into a clean line, every region, the random programs, the S7 programs |
| F8 | the two ways of a data cache hit swapped | 9 of 10 |

F1 and F2 were first run before their tests existed and survived the random programs, the regions and the S7 programs: a
store a rollback leaves behind is written again with the same value when the instructions run again, and a line whose D
bit is lost is seen only when it is replaced after a block's store before any interpreter store dirtied it. Their tests
were written for them, as MarsRT's survivors taught (`Mars_Native.md` §5.8.6), and caught them.

### 48.10 Exposure, and what was observed

Running the player's BIOS on Sapphire makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and never a
replacement writer**. The BIOS image was executed by the machine, given by the path of the referees' link to it, and
never opened, hashed, diffed, disassembled or dumped by any tool of this stage, and the BIOS folder was not listed; the
recompiler compiled BIOS code as it compiles any guest code, and no block made from a BIOS address, no program counter
inside the boot ROM or in BIOS code in RAM, and no memory holding BIOS code or data was printed, logged or looked at; the
verifier on the BIOS ran quiet, reporting pass or fail and the count of steps. What was printed of BIOS runs: counts
(steps, blocks, rollbacks, fills, generations, exits by reason, bytes of code), equality of pictures, sound and states,
digests, and times. Two profiles of Jak and Daxter's memory card screen (`rtsample.py`, §43.4's sampler) were named
through Sapphire's own symbols only; the compiled blocks' samples were left unnamed (no perf map was written), and no
guest address was printed. No SIF traffic was traced. No emulator, SDK or loader source was read.

### 48.11 Where the plan was wrong, or left open

- **§5.12's "loads and stores through a fast path for RDRAM and the scratchpad"**: on the EE most of RDRAM is reached
  through the data cache, whose lines, not RDRAM, hold a cached page's bytes, so the fast path a cached page can take is
  the cache's own hit (§48.4); RDRAM itself is reached directly only past the caches.
- **The board's catch-up was assumed independent of a block's work** (§44.5); with memory it is not, and the DMAC's
  ordering is what the replay now guards (§48.5). Where the guard reaches decided the language screen's speed.
- **S36's rule for the instruction cache** (a block entered only with every line held, shaped only from lines held) was
  right for blocks that ended at the first load or store and cost the longer blocks their entries (§48.4).
- **Code size**: a memory instruction's exit and its state's write-back are emitted once each, so a block of loads and
  stores is several times S36's (8.5 KB a block on Jak and Daxter at first, against S36's 1.4); shaped past missing lines,
  far fewer blocks are made (5,993 over Jak and Daxter's 900 frames with idle skipping, 63 MB, against 23,881 and 238 MB),
  well inside the 256 MiB budget.

### 48.12 Questions this stage leaves

1. **Q1 [routine], loads and stores in the default tier.** They are part of the recompiler, `effect: exact`, with
   `EMUSEN_SAPPHIRE_MEMORY=0` for measurement only. *Recommended*: keep them on wherever the recompiler is chosen; no
   setting of their own.
2. **Q2 [routine], C2's timing part.** Not built: a miss leaves the block (§48.6). *Recommended*: weigh it with the
   interpreter's memory path, and a refill inside a block (the undo log holding the victim line), when a profile puts the
   misses (15 million over Jak and Daxter's 900 frames) above the noise.
3. **Q3 [routine], an exit's size.** Each load or store carries its own exit and two write-backs (§48.5), which costs the
   memory card screen 6 % (§48.8). *Recommended*: a next step, before S38 adds more code a block: exits shared by a
   block's accesses, measured on that screen and the language screen. *Decided 2026-10-26 (routine) and built: §48.15.*
4. **Q4 [routine], the polled registers.** The largest reason a load leaves its block is a page it cannot serve, a
   device register polled (§48.8). *Recommended*: leave them to idle skipping (S33), which passes such waits whole.

### 48.13 What was read for this stage

`Sapphire_Plan.md` §1.4, §5.2, §5.12 with its amendment, §5.13 with the amendment of 2026-10-26, §7's S36 to S39 rows and
their amendment, §9.6; `EmuSen_Jit.md`; `Mars_Native.md` §5.8; this page's §16, §17, §30.83, §42, §43, §44 and §47 (for
the rebases); S33's `src/idle.rs` where it meets the board and the dispatcher; the interpreter's
`ee/memory.rs`, `cache.rs`, `tlb.rs`, `ops.rs`'s loads and stores, `ee/mod.rs`'s step, the board's `run_to`,
`run_through` and `step_dmac`, the DMAC's channels in `dmac.rs`; S32's `examples/bench.rs`; Cranelift 0.136's
`InstBuilder`, `FunctionBuilder` (its cold blocks) and `MemFlagsData`. No emulator, SDK or loader source, no article, and no firmware.

### 48.14 What this stage does not cover

- A data cache miss's refill, an uncached load and the UCAB inside a block (each leaves to the interpreter; §48.6).
- MMI as vector code (S38), COP1 and COP2's macro forms (S39), COP0, SYSCALL, BREAK and CACHE: each still ends a block.
- Chaining blocks, loops inside a block, and sharing the exits' write-backs (the last built at §48.15).
- The handheld: no measurement was made there.

## 48 (continued)

### 48.15 The loads' and stores' exits shared (2026-10-26)

*S37's Q3 (§48.12), decided 2026-10-26 (routine): shrink the code a block's loads and stores emit before S38 adds more
code to a block. Built on WiseMan `9c754f2c4` (S37 merged). Its disputes would be numbered from D-960; it raised none:
every oracle below agreed with the interpreter, and the step reads nothing of the hardware the interpreter does not
already implement.*

#### 48.15.1 In one paragraph

At S37 every load or store carried its own exit: a call of the replay, a write-back of everything the block held for
the case where every step stood, and the branches to the rollbacks; and, for the rollback past it, a second write-back,
of the state after it (§48.5). **Now a load or store that leaves at most two steps after the load or store before it
writes back nothing of its own**: one helper call (`Context::leave`) counts the reason, sets the guard and replays the
board through the steps run, and the state after the access before it is written back by that access's own block, the
one S37 already emitted for the rollback past it; the dispatcher then has the interpreter run the steps from there to
the access that left, as it does after any rollback past the last access. One write-back an access instead of two, and
the replay's call and its branches made once in Rust instead of in every exit. An access further than two steps from
the one before it keeps an exit of its own, as at S37, because running more steps again cost Jak and Daxter's loading
more than the smaller code saved (§48.15.4). The machine the dispatcher hands back is the interpreter's at the access,
exactly as before: the gate (§48.15.7) holds on every oracle, and ten seeded faults are caught (§48.15.8). **The code
is 28 % smaller an access** on the project's own blocks, 12 % smaller on the boot's and the loading's windows and 15 to
28 % on the card screen's; **the memory card screen ran at 1.03 to 1.04 times S37's speed, but not back to
memory-in-blocks off** (0.96 to 0.985 of it); Jak and
Daxter's loading is kept. The remaining cost is not in the exits: the publishing wait follows the compiler's throughput,
and most of a block's bytes are the issue model computed at run time (§48.15.6).

#### 48.15.2 What was built

| Module | What changed |
| --- | --- |
| `src/ee/jit/emit.rs` | a load or store's slow path: `leave` (the helper's call, then the state block of the access before it, or the shared return of `ROLLBACK` with nothing written when the replay stopped short of it or there is none) when at most `REWIND_MOST` (2) steps stand since that access; S37's exit otherwise, and when nothing ran or the verifier wrote back every step |
| `src/ee/jit/mod.rs` | `Context::leave`; `leave_word` and `left_by`, the one owner of the word the exit hands over (the steps run, the steps after the access before it, the undo entries written, the reason) |
| `src/blocks.rs` | `leave`: the reason counted, the guard set, the replay made without setting the lines when every step stands; the dispatcher's rollback branch takes it (`Run::rewound`); `Stats::rewinds`, `rewound_steps` and `queued` (entries that found their block's code still with the compiler); `Blocks::compile_nanos`; `EMUSEN_SAPPHIRE_WAIT=1`, for measurement |
| `examples/s33.rs` | `time` prints for each window the blocks and bytes compiled, the steps interpreted and compiled, the entries that waited for the compiler, and the compiler's seconds |
| `tests/blocks.rs` | `a_load_or_store_leaving_after_plain_instructions_leaves_the_interpreters_machine` |

`emusen-jit` is unchanged, so MarsRT is outside the blast radius.

#### 48.15.3 Why it is exact

The exit before access k, at instruction j, used to write back the state at j. It now writes back the state after access
k-1 (the instruction at `base`), which the code already held for S37's rollback past k-1, and the interpreter runs the
instructions from `base` to j. Those instructions make no load or store (k-1 is the last before k), so running them
again reads nothing a DMAC step could have changed and writes nothing the undo log must know; they read the registers
and the issue model the state block wrote, which are what the interpreter's machine held after k-1. The board is not
run twice: the replay has brought it through the times of every step to j, and the dispatcher's interpreter steps find
it there (`run_to` of a time already reached does nothing), the lines set before each step from what the replay saw, as
after an interrupt's rollback. The replay must not set the lines and Cause.IP2 and IP3 as an exit where every step stands
does, since the steps run again would see the last step's lines; `leave` replays with that part left out, and the
rollback branch then sets them exactly as for a rollback stopped at j. When the replay stops short of `base` (an
interrupt, or the DMAC within the guard), nothing is written and the stores are undone, as at every exit since S37.
With no access before k, `base` is 0 and the same branch undoes nothing and runs the steps from the block's start. The
guard is the one S37's exit set (the last access's index), so the DMAC's order against the block's accesses is
unchanged (§48.5). The interrupt delay of D-860 (`Sapphire_Native.md` §30.78) is untouched: the replay stops before any step at which an interrupt is
due, so no step run again makes a recognition, and the recognition stays the interpreter's next step's.

#### 48.15.4 The limit of two steps (measured 2026-10-26)

The first version shared the exit of every load or store that left once its block had run a step. On Jak and Daxter's
900 frames it made 11.2 million rewinds and ran 39.1 million steps again in the interpreter (3.5 a rewind), and the
loading window ran at 48.7 ms a frame against S37's 46.5 (§48.15.5, series A). The gap between two accesses is known when
the block is compiled, so an access more than `REWIND_MOST` steps from the one before it keeps S37's exit. In one
interleaved round of two pairs on Jak and Daxter (loads 1.5 to 4.7), the limit of two took the loading window from 49.7
to 47.5 ms a frame and left the memory card screen as it was (106.3 against 106.8), the steps run again falling to 5.0
million in 6.3 million rewinds; the code grew by 3 to 5 % (the card screen's window 115 to 117 MB against 111 to 113,
the loading's 39.6 against 37.7). The limit is the one kept.

#### 48.15.5 The code and the speed (measured 2026-10-26)

**The code an access emits**, on the project's own block (an ADDU, n accesses, an ADDU), compiled by the same lowering and
measured as Cranelift's code size (bytes), S37 against this step:

| n | loads | | loads, each with an ADDU | | stores, each with an ADDIU | |
| --- | --- | --- | --- | --- | --- | --- |
| 8 | 14,192 | 12,496 | 22,688 | 17,056 | 22,288 | 16,992 |
| 16 | 27,048 | 24,488 | 45,264 | 32,888 | 45,304 | 32,888 |

An access with a plain instruction beside it costs 1,974 bytes against 2,748 (28 % less); back-to-back loads gain less
(9 %), since S37 already emitted no state block for an access with nothing between it and the one before.

**The windows**, S33's runner (`s33 time … recompiler on`, idle skipping on, the GS's workers at their default count),
each binary built from its tree with the release profile, and memory-in-blocks off being this step's binary with
`EMUSEN_SAPPHIRE_MEMORY=0`. Code compiled in each window (blocks, MB), steps the interpreter took (millions) and entries
that found their block's code still with the compiler (millions), from series B; every round of a binary gave the same
counts within a few per cent, except on the card screen (below):

| Window | S37 | this step | memory off |
| --- | --- | --- | --- |
| The boot, 0-690 | 14,778, 128.5; 211; 68 | 14,771, 112.9; 200; 58 | 9,821, 16.6; 346; 5.8 |
| Its language screen, 690-760 | 136, 0.6; 4.8; 0 | 136, 0.5; 4.8; 0 | 116, 0.2; 10.1; 0 |
| Jak and Daxter loading, 8-400 | 4,877, 45.2; 70; 31 | 4,877, 39.6; 67; 29 | 6,404, 8.9; 214; 1.9 |
| Its memory card screen, 400-900 | 12,855 to 14,905, 137 to 184; 155 to 170; 77 to 91 | 12,855 to 13,781, 117 to 132; 158; 80 | 8,964, 13.0; 116; 2.1 |

**The card screen's code is bimodal**: in one run of S37 and one of this step (and in no run with memory off) the window
compiled only 1,116 and 1,504 blocks, 17.6 and 16.3 MB, with 104 million entries waiting; the other seven runs of the two
compiled 12,500 to 14,900. Which happens depends on when the screen's first blocks are published, which depends on the
compiler thread's timing, not on any result: every run's pictures, sound and state were the interpreter's.

**The speed.** Each frame's `advance` timed, one run at a time at CPU weight 100 beside other stages' work, the order of
the binaries alternated within each round, the one-minute load before and after each run recorded. Medians of three
rounds (ms a frame), with the spread of the three:

| Window | S37 | this step | memory off | this step against S37 | against memory off |
| --- | --- | --- | --- | --- | --- |
| *Series A*, shared exits without the limit, loads 1.2 to 4.0 | | | | | |
| The boot, 0-690 | 173.2 (2 %) | 173.3 (2 %) | 189.7 (1 %) | 1.00 | 1.09 |
| Its language screen, 690-760 | 123.3 (3 %) | 124.2 (1 %) | 129.2 (1 %) | 0.99 | 1.04 |
| Jak and Daxter loading, 8-400 | 46.5 (1 %) | 48.7 (7 %) | 70.9 (2 %) | 0.95 | 1.46 |
| Its memory card screen, 400-900 | 105.8 (2 %) | 102.2 (2 %) | 100.7 (1 %) | **1.04** | **0.985** |
| *Series B*, the step as committed, loads 3.5 to 10.2 | | | | | |
| The boot, 0-690 | 185.3 (15 %) | 180.3 (23 %) | 202.0 (11 %) | 1.03 | 1.12 |
| Its language screen, 690-760 | 127.5 (9 %) | 130.0 (6 %) | 140.0 (3 %) | 0.98 | 1.08 |
| Jak and Daxter loading, 8-400 | 52.0 (17 %) | 50.0 (20 %) | 74.9 (11 %) | 1.04 | 1.50 |
| Its memory card screen, 400-900 | 114.2 (30 %) | 111.3 (8 %) | 106.8 (10 %) | **1.03** | **0.96** |

A preliminary round on Jak and Daxter (shared exits without the limit, loads 4.0 to 6.6) gave the card screen 112.0,
109.8 and 106.5 ms and the loading 51.8, 51.7 and 74.4 (S37, this step, memory off). Series B ran under twice series A's
load and its spreads are wide; on the card screen the two series and the preliminary round agree on the order (memory
off, then this step, then S37), and on this step's distance from memory off, 1.5 to 4.5 ms a frame. The goal (the card
screen at least as fast as memory off, the loading kept) is half met: the loading is kept, the card screen not.

#### 48.15.6 The publishing wait, and where the bytes are (measured 2026-10-26)

**The wait is the compiler's throughput.** With memory in blocks the card screen's window waits for the compiler about
40 times as often as without (80 against 2 million entries), and the interpreter takes 158 against 116 million steps of
it; the compiler is busy 32 to 36 s of the window's 55 (4 s without). `EMUSEN_SAPPHIRE_WAIT=1` waits for each block's code
at its hand-over, so that nothing runs interpreted behind the queue: in two runs the card screen took 188.2 and 184.1 ms a
frame and the loading 102.6 and 100.0, the compiler's 45 and 22 s now inside the frames, 15,108 blocks and 169 MB
compiled on the card screen. Compiling a block's first version sooner does not pay while compiling costs this much: the
compiler makes about 3.7 MB of code a second, so the wait falls only with the bytes.

**Where a block's bytes are.** Cranelift's listing of the project's block of two loads, each with an ADDU, shows about 50
host instructions for an access's fast path, 70 for a write-back, and **85 for each instruction's issue model**, loads and
plain instructions alike: the pairing state a block enters with (`open`, the last category and pipe, `last_writes`) is
loaded, not known, and once it is a run-time value the model's selects and maxima stay run-time values to the block's
end (§44.4's folding acts only on what the block itself fixes). The exits were the part an access added beyond that,
and this step removed most of it; what remains of a memory block's size is the model, paid by every instruction of a
block whose code is 4.5 (the boot) to 6.3 (the card screen) times as large as with memory off.

**The next step, measured and not built**: make the issue model's code smaller, by specialising a block on the pairing
state at its entry (a guard on the few fields, the model then folded from constants), or by a first tier of S36's shapes
(memory off: 13 MB on the card screen against 117 to 132) recompiled with memory in blocks once a block is hot. Either is
a change to S36's emitter or to the dispatcher's tiers, not to the exits (§48.15.10, Q2).

#### 48.15.7 The gate (measured 2026-10-26)

On the desktop beside other stages' work; the player's Americas BIOS executed through the referees' link to it, never
read; ps2autotests from `~/.cache/emusen/probe/sapphire/ps2autotests/`, Jak and Daxter read in place from
`~/.cache/emusen/probe/sapphire/discs/`. Every whole-machine oracle ran on the shared exits without the limit and again on
the step as committed; the figures are the latter's, and both agreed throughout.

| Oracle | Run | Result |
| --- | --- | --- |
| 1. The verifier | `the_verifier_steps_beside_every_compiled_instruction_and_finds_nothing`, `random_programs_under_the_verifier_find_nothing` (`Verify::Steps` and `Verify::Blocks`); `s33 verify boot` and `s33 verify disc`, 30 frames each, quiet | held: 220,863,488 and 231,997,440 steps compared, the whole states equal |
| 2. Random programs | seeds 0 to 159 and 1000 to 1079, 24,000 steps each, compared every 400 and whole at the end | equal: 3,790,606 and 1,883,354 steps compiled, 1,431 and 739 rollbacks as at S37, 975 and 767 rewinds |
| 3. The mechanisms | the 24 tests S36, S37, D-780 and D-860 left, and the new test | 25 of 25 |
| 4. ps2autotests | `s10` on each tier | 52 of 55 on both, the same three as §36.5; all 55 texts identical byte for byte between the tiers |
| | the S6 and S7 memory programs | equal to the interpreter and to their records on the blocks' machine |
| 5. The bench workloads | `examples/bench.rs` interpreted, with `--blocks`, and with `--blocks --idle` | the boot's 760 frames `50F06A02F1B76860`, `2EE7B3D10DF3C401`, state `2051C6BB42228C53`; Jak and Daxter's 900 frames `7EEC08D4D07DB621`, `20BB66B5E91347C9`, state `83E7D9CAFA075D14`; the same on every tier, and in every timed run |
| 6. A state into an empty cache | `a_state_saved_between_blocks_loads_into_an_empty_cache_and_runs_alike`; `s36 state` on the boot (400, 100) and Jak and Daxter (300, 100) | equal: 0 pictures, 0 sound, 0 states differing |

**The new test**, `a_load_or_store_leaving_after_plain_instructions_leaves_the_interpreters_machine`: a loop whose second
access is followed two steps later by a load of a timer's count, which leaves at every pass, then by a load that misses
now and then and one misaligned at every fourth pass, each further from the access before it; 20,000 steps beside the
interpreter, compared every 100, and under `Verify::Blocks`: 890 rewinds, 1,359 steps run again, and every way of
leaving taken.

#### 48.15.8 Seeded faults (2026-10-26)

Each made alone on a copy of the crate built apart (the step as committed), the 25 tests of `tests/blocks.rs` run, the
source restored. F1 to F8 are S37's (§48.9), made again on this code (F3 narrower: the check of the board's next event
removed, the guard kept, which F4 removes); F9 and F10 are this step's.

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | a rollback leaves the block's stores in memory | 1: the rollback beside the handler's reads |
| F2 | a store hitting a cached line leaves its D bit clear | 6 |
| F3 | the board's next event not checked before an access | 1: beside the DMAC |
| F4 | the replay's guard ignored | 1: the guarded board |
| F5 | the translation cache's key without the mode | 2: the TLB and mode, the S6 programs |
| F6 | LWR at a word's first byte keeps the register's upper half | 1: the S6 programs |
| F7 | an uncached load without its bus wait | 6, among them the random programs |
| F8 | the two ways of a data cache hit swapped | 13 |
| F9 | a load or store's exit restores the state after the access two before it, not the one before | 8, among them the new test |
| F10 | the exit's replay sets the lines and leaves `Context::steps` as an exit where every step stands does | 10 |

F9 first survived the new test: its load of the timer's count followed the block's first access, whose line was
replaced by the loop's two conflicting ones, so that it left at once and the block began after it; the access two before
and the access before were then the same. The loop's first load was moved to a line of its own and F9 is caught there.

#### 48.15.9 Exposure, and what was observed

Running the player's BIOS on Sapphire makes this writer a writer of a player's-BIOS stage (`Sapphire_Plan.md` §1.4):
**exposed, and never a replacement writer**. The BIOS image was executed by the machine through the path of the
referees' link to it, and never opened, hashed, diffed, disassembled or dumped, and the BIOS folder was not listed. What
was printed of BIOS runs: counts (steps, blocks, bytes, rewinds, rollbacks, exits by reason, entries waiting, the
compiler's seconds), the verifier's pass and its count of steps, digests, equality of pictures, sound and states, and
times. One mistake in this stage's own build printed Cranelift's refusals of blocks it had lowered wrongly (a value used
where it was not defined), each naming the block's start address; those addresses were not looked at further, and the
code that caused them was replaced before any measurement. The disassembly of §48.15.6 is of the project's own block,
assembled for the purpose. No SIF traffic was traced. No emulator, SDK or loader source was read.

#### 48.15.10 Questions this step leaves

1. **Q1 [routine], the limit.** `REWIND_MOST` is 2, measured against no limit (§48.15.4). *Recommended*: keep it; measure
   it again if S38's vector code changes how far apart a block's accesses fall.
2. **Q2 [decision], the card screen's remaining cost.** It is the compiler's throughput against the bytes a block of
   loads and stores now holds, most of them the issue model computed at run time (§48.15.6). *Recommended*: a next step
   before S38, specialising a block on its entry's pairing state so that the model folds, measured on the card screen,
   the loading and the boot's language screen; the first tier of S36's shapes as the alternative if the guard on the
   entry's state refuses too many entries.
3. **Q3 [routine], the card screen's two modes.** Which one a run takes depends on the compiler thread's timing
   (§48.15.5). *Recommended*: record the blocks compiled beside every timing of that screen, as here, and compare only
   runs of the same mode.

#### 48.15.11 What was read for this step

This page's §48; `Sapphire_Native.md` §30.78 to §30.85, §44 and §47; `EmuSen_Jit.md`; the dispatcher (`blocks.rs`),
the emitter (`emit.rs`, `mod.rs`), `Board::run_through`, `Ee::interrupt_now` and S33's runner; Cranelift 0.136's
`FunctionBuilder`, jump tables and `Context::set_disasm`. No emulator, SDK or loader source, no article, and no firmware.
