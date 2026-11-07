# Diamond_Speed — the exact levers, stage 13 and the second pass

*Written 2026-10-12, at the plan's stage 13 (`Diamond_Plan.md` §5.12, §7 row 13, §8's G6), with the tester's go of
2026-10-11.* The plan names four levers, each to be bit-identical to the plain path, in an order: the rasteriser on
worker threads, a decoded-instruction cache, idle skipping, a recompiler. This page records the bench the stage was
measured on, the profile taken before any lever was chosen, the two levers built (idle skipping first, against the
plan's order, and the first tier of the decoded-instruction cache), how each is known to be exact, what each bought
on the desktop, under the laptop's proxy and on the handheld, and what was not built and why. The build record's
entry is `Diamond_Native.md` §21.

The marks are the plan's: **[M]** measured, on the date given, with the tool named; **[R]** read from a source; **[A]**
argued; **[C]** a choice.

## 1. The machines and the target

- **The desktop**: Ryzen 7 7700X, Fedora 44, Rust nightly 2026-10-01. Other work ran on it throughout, so every
  figure here is from runs interleaved build by build on one core (`taskset -c 11`), and is reported as the fastest
  and the median of the rounds.
- **The laptop's proxy**: the desktop under `systemd-run --user --scope -p CPUQuota=33%`, the plan's proxy for the
  i5-10310U (§5.12).
- **The handheld**: the Legion Go S, a Ryzen Z1 Extreme with 16 threads, SteamOS, glibc 2.43, governor `powersave`,
  platform profile `custom`; measured on battery (`BAT0` discharging, 10 to 8 per cent), as G6 asks.
- **The target** [C, §5.12]: every bench game at full speed at the 33 per cent quota with a fifth of the frame left,
  13.4 ms of an NTSC frame's 16.7; the plan derived a desktop mean of about 3.8 ms from VenusRT's quota factor of 3.3
  to 3.5, and G6 adds full speed on the handheld on battery.

## 2. The bench

`examples/bench.rs` boots an image and times each frame's `advance` on the emulation thread, and nothing else: the
digests it prints (FNV-1a of every frame's picture, of the sound's samples and of the final state, and with `--every
N` the state every N frames) are taken outside the timed span. A disc or a PS-EXE runs on the open kernel with no
firmware; a `.s` source is assembled and run with no kernel. `--plain` turns the levers off, `--verify N` checks
every Nth skip against the interpreter (§4.4), `--armed` runs the debugger's frame with the kit's C15 tables.

**The bench set** is the seven copied discs in the probe's `discs/` folder, and no others: Ape Escape, Castlevania:
Symphony of the Night, Crash Bandicoot, Rayman, Silent Hill, Spyro the Dragon and Tekken 3, each booted on the open
kernel to 3,600 frames (a minute of NTSC), the mean taken over all of them. The project's programs and ps1-tests'
release are the exactness bench (§4.5, §5.3), not the speed bench: they finish in a few frames and then wait.

**The link's placement** [M, 2026-10-12, the readwatch step's study, its logs in the probe's `readwatch/logs/`]: the
same machine code moved 1.8 to 10.7 per cent a frame between two links of the default build from where the linker put
the functions, and with every function aligned to 64 bytes (`RUSTFLAGS="-C llvm-args=-align-all-functions=6"`) two
links agreed within 0.1 to 0.9 per cent. So every lever here was measured in both links, built from the same source by
the probe's `build.sh`, and a gain under 10 per cent in the default link alone is not claimed. The aligned link is a
measuring instrument; whether the shipped build should use it is proposed in §8.2, not decided.

## 3. The profile before any lever

[M, 2026-10-11, `rtsample.py` (the speed tooling's ptrace sampler for native examples) at 2 ms on the bench before the
stage, Spyro the Dragon, Ape Escape and Tekken 3, 6,700 samples each.] The emulation thread is the R3000A's
interpreter. Of the samples outside the bench's own digests:

| Part | Spyro the Dragon | Ape Escape | Tekken 3 |
|---|---|---|---|
| The step, its completion, the fetch, the address's decode, the word's decode, loads and stores | 80% | 84% | 77% |
| The GPU: pixels, the rasteriser, the texture cache, the display read | 10% | 3% | 11% |
| The SPU | 3% | 5% | 4% |

Inside the interpreter, the fetch decoded the address through the whole map on every instruction (`decode` and
`ram_banks` 8 to 10 per cent of the thread) and decoded the word again (`emusen_mips::decode`, 3.5 per cent).

**Where the instructions went.** A scratch tool kept out of the tree counted each PC's
instructions and cycles over 2,400 frames:

| Game | Instructions a frame | The waits, and their share of the instructions |
|---|---|---|
| Ape Escape | 238,769 | the libraries' VSync with its timeout on the stack, a turn of 14 instructions: 42% |
| Castlevania: SotN | 245,762 | the same VSync loop, 23%, and a poll through a call with a register timeout, 25% |
| Crash Bandicoot | 175,876 | the VSync loop, 31% |
| Rayman | 244,900 | a byte polled in RAM and a halfword polled through a call: 43% |
| Silent Hill | 198,720 | the VSync loop, 41% |
| Spyro the Dragon | 258,352 | DMA channel 3's CHCR polled for its start bit, 35% |
| Tekken 3 | 199,412 | none the size of these |

A wait costs the interpreter what any instruction costs, so between a quarter and a half of six games' time was spent
interpreting loops whose outcome was already decided by the next event. The plan ordered the levers by what it
expected to cost (§5.12); the profile ordered them by what they would buy, and the stage took idle skipping first and
the decoded-instruction cache second. The threaded rasteriser could buy at most the GPU's 3 to 11 per cent, and none
of it under the quota (§7.1).

## 4. Lever 3, built first: idle skipping

### 4.1 What a skip is

At a loop's head the machine is in a state from which, until the next event, every turn does the same thing: the same
instructions, the same accesses, the same time, the registers back where they were, memory unchanged but for at most
one value that moves by a fixed step. If that is proven, the turns between now and the event can be passed by adding
their time to the clock and their steps to the one moving value, and the machine is where the interpreter would have
left it. The skip never crosses the next event: every device that changes between events changes there, and an event
is where the interpreter would see it. The debugger's frame never skips.

### 4.2 The proof (`src/idle.rs`, `prove` and `walk`)

A head is the target of a taken backward branch within 1 KiB, met once the branch's delay slot has run. At a head,
`walk` runs one turn over the words the I-cache holds (every word must hit, since a hit is what the turn will fetch),
each instruction as `blackmoon-r3000`'s step runs it: the operands read before the previous load lands, the load
cancelled by a second load of the same register, a branch taking effect after its slot. Every value is a constant, the
concrete value the processor holds, except one, the **induction**: a register or a memory word, written `X + a` for its
value at the turn's start plus a constant.

The walk refuses, and so the head is not skipped, when the turn would:

- fetch a word the I-cache does not hold, or from KSEG1, or pass the kernel's two seams (the shell's entry while a
  program waits to be side-loaded, the A0h and B0h entries while putchar is tapped);
- read a device other than I_STAT, I_MASK and the DMA registers, whose reads are pure and which only events and the
  processor's own stores change; a timer, GPUSTAT, the SPU and the CD-ROM controller all change with time or on read;
- store anywhere but RAM or the scratchpad, or store a changing value to more than one word;
- use the multiplier, a coprocessor, an exception, LWL, LWR, SWL or SWR, or the induction in anything but an addition
  of a constant, a move, or a comparison for equality in BEQ or BNE;
- take an interrupt at the head, have the debug unit armed, the cache isolated, or the MTC0 view of D-17 pending.

A first walk with every value a constant finds what a turn changes: if nothing that the turn reads before it writes
it, and no memory word, changes, the loop is a pure poll and needs no second walk. If exactly one register read before
written, or one memory word, changes, a second walk takes it as the induction; its step `δ` is what the turn's end
holds, `X + δ`. Registers the turn writes before it reads them and leaves as `X + a` (a copy of a memory induction in
a register, as the libraries' VSync keeps in `v0`) move with it; every other register must end as it began. Each BEQ
or BNE on the induction is a test `X + a == c`, true or false at the first turn; for `δ = ±1` the turn at which its
answer changes is `(c − x − a)·δ`, and the least of them is the number of turns the path is proven for.

### 4.3 The window and the skip (`Idle::arrive`, `plan`, `Skip::apply`)

A proof is not a measure of time. The machine keeps a **window** from the arrival at which the proof was made, and
four turns later compares: the processor's registers (the movers less four steps), HI/LO's readiness, the induction
moved by exactly four steps, and the bus's timing, the clock, the end of the processor's last bus access and the write
queue's four entries, each moved by one span or not at all. Four turns, because a turn that queues one store rewrites
one of the queue's four entries, so only after four are all of them a span later. Every one of the bus's timing rules
is a comparison of times or a time plus a constant, so a state shifted in time behaves as it did shifted; with the
data the same and nothing between events able to change it, the next four turns are the last four, a span later, and
so on. If an event ran during the window (`Board::events_run` moved) or an exception was taken, the window is dropped.

The skip passes `n` whole periods, the most that end before the next event (`now + n·span < next_event`) and within
the proof's turns (`4 + 4n ≤ turns`): the clock, the end of the last bus access and the queue's entries move by `n`
spans where they moved, the induction and its movers by `4n` steps. The rest of the loop, up to and across the event,
is interpreted.

A head whose proof fails, or whose window meets an event or does not repeat, waits twice as many arrivals as before its
next attempt, up to 4,095, in a table of 1,024 heads. The plain loop watches for heads for 512 steps in every 4,608,
a window or a skip renewing the watch: a wait is thousands of turns long, so it is met within a turn or two of its
start, and Tekken 3, which waits in no loop the proof accepts, pays for the watch in one step in nine. Before the
watch was rationed, the skipper cost Tekken 3 4 to 5 per cent [M].

### 4.4 How it is known to be exact

1. **The verifier.** `Idle::verify = N` (the bench's `--verify N`) checks every Nth skip as it is made: a copy of the
   processor and the board runs the skip's turns instruction by instruction through the plain step, must meet no
   event and no exception on the way, and must reach the state the skip leaves, the whole state byte for byte with the
   next event and the event count. A difference panics with the state's first differing field.
2. **`programs/idle.s`**, the loop forms games wait in, each recording what it leaves, timer 2 stamping each: eight
   vblank waits with the libraries' stack timeout under a vblank handler; register timeouts that expire after 9 to
   13, 500 and 1,001 turns, either side of the period; the same on the stack; timeouts counting up to 9 to 13 and 777,
   each turn ending on a store; a DMA6 and a DMA2 transfer waited for on their CHCR; a wait on timer 2 itself, never
   skipped; and three waits on I_STAT whose turns grow the turn after timer 2's interrupt is flagged, the flag carried
   in the scratchpad so that nothing in the registers or the bus's timing shows the event.
3. **The tests** (`tests/programs.rs`, stage 13's block; `src/idle.rs`): `idle.s`, six of the project's timing
   programs, the CD read program, eight of ps1-tests' release on the open kernel (opt-in) and a copied disc for 600
   frames (opt-in, `EMUSEN_DIAMOND_DISC`) each give the same digests of picture, sound and state with the skipper and
   without, every skip verified; every debugger table armed gives the plain run's digests on `idle.s`, the plain run now
   being the skipper's; the walk lands loads as the processor does; a timeout is proven to the turn it expires,
   counting either way, in a register or memory; and the refusals of §4.2.
4. **The games.** The seven bench discs, 3,600 frames each with every sixteenth skip verified, give the same picture,
   sound and state digests (the state every 600 frames) with the skipper as without it [M, 2026-10-11 and 2026-10-12].

**Mutants** [M, `mutants.py` in the probe's `s13/`, each applied, the crate's tests run, the file restored]: caught,
the write queue left where it was; a skip allowed to reach the event; the path's bound without the window's four
turns; the movers or the memory induction not moved; the end of the last bus access not moved; a second load not
cancelling the first; timer 2 read as a constant; a timeout's turn one late, counting up and down; windows kept across
events; and a copy of the induction not taken as a mover, caught by the test that names each proof's movers (the
mutant would only cost speed, since the window's comparison then refuses the skip). Survived, two checks the proof
makes redundant and kept as defence in depth [C]: the movers' move compared at the fourth arrival (the proof implies
it), and the window's registers compared (the proof makes the turn a fixed point, and a register the turn writes before
reading it is rewritten every turn alike).

### 4.5 What it skips

[M, 2026-10-12, the handheld's run of the head build on the current tree (§6), 3,600 frames each.]

| Game | Skips | Turns passed | Share of the clock passed |
|---|---|---|---|
| Ape Escape | 23,162 | 25,852,544 | 38.4% |
| Castlevania: SotN | 35,050 | 12,333,308 | 17.3% |
| Crash Bandicoot | 39,192 | 10,782,264 | 14.4% |
| Rayman | 53,063 | 66,389,812 | 38.3% |
| Silent Hill | 153,519 | 15,730,124 | 25.6% |
| Spyro the Dragon | 11,755 | 13,563,064 | 25.2% |
| Tekken 3 | 9,684 | 6,150,076 | 10.0% |

The day before's tree passed 72 per cent of Ape Escape's and Castlevania's clocks, which were then stuck at their
intros; with D-29's and D-45's merges they play on, and wait less. Spyro the Dragon's share fell from 36.7 to 25.2
per cent with D-29, which changed when DMA channel 3 ends: its CHCR wait of §3 is gone from its profile. Tekken 3's
movies brought it waits the skipper takes, 10 per cent of its clock.

## 5. Lever 2, its first tier: the decoded-instruction cache

### 5.1 The design

The processor fetches cached code from the I-cache only, and the I-cache changes only by a refill and an isolated
store: a store to RAM leaves a cached line as it was, on the console as here (`Diamond_Native.md` §11.4). So a word
decoded as it enters the cache is the word that runs, and no store can make it stale. `ICache` keeps each word's
`emusen_mips::Op` beside it, decoded at `fill` and at an isolated store, and decoded again from the words after a
state load (the decoded words are not state). `blackmoon-r3000`'s `Bus` gains `fetch_decoded`, a method whose default
is `fetch` and `decode`, which the step now calls (`BlackMoon_R3000.md` §2); Diamond's bus answers a fetch in KUSEG's
first 512 MiB or KSEG0 that falls in RAM or in the BIOS's window, with the cache on and a hit, from the decoded word,
charging the hit's cycle, and every other fetch as before. The plan's "invalidated by stores to code" is therefore
needed for uncached code only, and uncached code is decoded at each fetch, as it is fetched at each fetch.

Three changes beside it: memory control keeps the RAM window with its register, since every access asked for it;
the debug unit's checks return at once while its enable bit is clear; and a data access to RAM is answered before the
map is decoded, with the same wait for the bus and the write queue, the same stall and the word read whole, as
`decode`, `read_at` and `write_at` make it. An instruction's completion is inlined into each arm of the step.

### 5.2 Why this is the first tier

The plan's lever speaks of blocks whose fetch costs are their lines' hits and misses. This tier caches the decode and
keeps the step whole, one instruction at a time; a block executor (runs of decoded instructions with the boundary's
work done once per run where the timing model allows) is the second tier, priced in §7.2 and not built.

### 5.3 How it is known to be exact

`a_decoded_fetch_is_a_fetch_and_its_decoding` holds `fetch_decoded` to `fetch` and `decode`, word, time and fault,
over every segment, both RAM sizes, the BIOS's window shrunk below an address, the cache off and on, misses, hits and
an isolated store into a cached line, and then with the map changed under cached lines (the BIOS's window shrunk, RAM
to 2 MiB, the cache switched off): a line's tag outlives the map it was filled under, and none of those may still hit.
`each_word_is_decoded_as_it_enters` covers a refill, an isolated store and a state's words. The r3000 suite's 55,000
cases pass through the default method; the crate's 154 unit and 67 program tests and the bench's digests are unchanged.
Mutants, all caught: the BIOS's window not asked, the cache's switch not asked, KUSEG past 512 MiB taken as RAM, a
refill or an isolated store not decoded, a state's words not decoded, the RAM window not kept.

## 6. Measured

All on the tree of 2026-10-12 after WiseMan's c27bbfbf (D-29, D-41, D-45 merged), whose games reach further than
the day before's: Silent Hill, Castlevania and Tekken 3 now play their movies and Ape Escape its polygon intro, so the
waits of §3 are a smaller part of their frames than §3's table and every figure measured before these merges
(Ape Escape and Castlevania near 1 ms a frame) mostly timed games stuck waiting. *Before* is WiseMan c27bbfbf with the
bench added; *after* is the branch's head, the same tree with the levers. Every pair of runs gave the same picture,
sound and state digests (§6.5). Milliseconds of the emulation thread a frame, the mean over 3,600 frames.

### 6.1 The desktop

[M, 2026-10-12, `cmp.py` in the probe's `s13/`: three rounds, the builds interleaved, fastest and median of the means.]

| Game | Before, default link | Before, aligned | After, default link | After, aligned | After ÷ before (aligned medians) |
|---|---|---|---|---|---|
| Ape Escape | 4.812 / 4.884 | 4.944 / 4.974 | 3.041 / 3.041 | 3.012 / 3.028 | 0.61 |
| Castlevania: SotN | 5.038 / 5.516 | 5.155 / 5.156 | 3.737 / 3.745 | 3.704 / 3.721 | 0.72 |
| Crash Bandicoot | 4.069 / 4.206 | 4.091 / 4.173 | 3.041 / 3.049 | 2.951 / 3.003 | 0.72 |
| Rayman | 4.290 / 4.380 | 4.435 / 4.443 | 2.327 / 2.364 | 2.285 / 2.309 | 0.52 |
| Silent Hill | 4.074 / 4.091 | 4.170 / 4.188 | 2.852 / 2.868 | 2.784 / 2.794 | 0.67 |
| Spyro the Dragon | 5.174 / 5.190 | 5.272 / 5.283 | 3.501 / 3.511 | 3.466 / 3.472 | 0.66 |
| Tekken 3 | 4.165 / 4.212 | 4.214 / 4.220 | 3.173 / 3.184 | 3.132 / 3.148 | 0.75 |

Every game is now under the plan's desktop figure of 3.8 ms, Castlevania by the least (3.72). The factors are each
game's share of waiting plus the decoded cache's 15 to 25 per cent of what is left.

### 6.2 The laptop's proxy, the 33 per cent quota

[M, 2026-10-12, the same builds aligned, two rounds under `systemd-run --user --scope -p CPUQuota=33%`.]

| Game | Before | After | Within 13.4 ms |
|---|---|---|---|
| Ape Escape | 15.127 / 16.980 | 9.398 / 10.203 | after only |
| Castlevania: SotN | 15.477 / 15.557 | 11.154 / 11.186 | after only |
| Crash Bandicoot | 12.379 / 12.457 | 8.973 / 9.029 | both |
| Rayman | 13.291 / 13.350 | 7.104 / 7.119 | both, before by 0.05 |
| Silent Hill | 12.391 / 12.806 | 8.513 / 8.576 | both |
| Spyro the Dragon | 15.949 / 16.237 | 10.407 / 10.670 | after only |
| Tekken 3 | 12.914 / 12.930 | 9.376 / 9.653 | both |

The quota's factor over the desktop's aligned medians was 2.9 to 3.4 after the levers, inside VenusRT's 3.3 to 3.5 at
its top and below it elsewhere, so the desktop figure of §5.12 was a conservative stand-in for this one.

### 6.3 The handheld

[M, 2026-10-12, `bench.sh` in a scratch folder on the handheld, run by `systemd-run --user` with no Mistress running,
the default link of each build, one run each, the bench's discs copied there for the run and removed after.]

| Game | Before, charger | After, charger | After, p95 | After, battery (the tree before the merges) |
|---|---|---|---|---|
| Ape Escape | 5.285 | 3.335 | 6.792 | 1.343 |
| Castlevania: SotN | 5.504 | 4.125 | 5.663 | 1.166 |
| Crash Bandicoot | 4.512 | 3.242 | 5.106 | 3.384 |
| Rayman | 4.666 | 2.572 | 4.300 | 2.746 |
| Silent Hill | 4.400 | 3.115 | 4.186 | 2.354 |
| Spyro the Dragon | 5.599 | 3.823 | 5.653 | 3.983 |
| Tekken 3 | 4.536 | 3.506 | 4.374 | 4.540 |

The charger rows are the current tree (BAT0 charging, 41 to 43 per cent). The battery rows (BAT0 discharging, 10 to 8
per cent) were taken before WiseMan's D-29 and D-45 merges, the build of 2026-10-12's first handheld run, when Ape
Escape and Castlevania still waited at their intros; they are kept apart and are not the current tree's. On that run
the build before the levers took 4.20 to 5.69 ms on battery. The worst frame of either build on either supply was
11.0 ms; the slowest mean, 5.7 ms, is a third of an NTSC frame. **The handheld's emulation thread was at full speed
before this stage and is at about a fifth of a frame after it**: the slowness the tester reported playing stage 11's
build on the handheld does not come from the emulation thread's cost, which this bench measures, and is to be looked
for in what surrounds it there (the frontend's loop and presentation, the disc's opening, the games that do not
start). A run on battery of the current tree was owed until 2026-10-28, when runs on battery were dropped.

### 6.4 G6

| G6's terms | Result |
|---|---|
| §5.12's budget at the 33 per cent quota on the bench set | **Met**: every game 7.1 to 11.2 ms against 13.4 (§6.2) |
| The desktop's stand-in, about 3.8 ms | **Met**: 2.3 to 3.7 ms (§6.1) |
| Full speed on the Legion Go S on battery | **Met on the earlier tree** (1.2 to 4.5 ms on battery) and on the current tree on the charger (2.6 to 4.1); the current tree's battery run is owed (§6.3) |

### 6.5 Exactness on the games

Each of the seven, 3,600 frames, with the levers and without, gave the same digests of every frame's picture, of the
sound and of the final state, on the desktop in both links and on the handheld, the handheld's digests equal to the
desktop's. On the desktop each was run again with every sixteenth skip verified against the interpreter (§4.4) and the
state's digest every 600 frames, and none differed (`final-verify.txt` beside the bench).

## 7. What was not built, and why

### 7.1 Lever 1: the rasteriser on worker threads

The GPU is 3 to 11 per cent of the emulation thread in the profile (§3), and about 10 per cent of Tekken 3's after the two levers, and
under the quota a worker's time is charged to the same 33 per cent, so the proxy would show the threads' cost of
handing over commands and nothing of their gain; on the handheld the emulation thread already leaves three quarters
of the frame (§6.3). Built, it would also have to take the display's line-by-line read of VRAM with it, since the scan
reads VRAM at each line's time, and it would cross the GPU module, where `diamond-gpu4` was working at the same time.
Not built: the budget is met without it (§6.4). It remains the plan's lever for when an enhancement, a higher internal
resolution, puts the GPU on the critical path.

### 7.2 Lever 2's second tier, and lever 4, the recompiler

After the first tier the step is still about 10 ns an instruction on Tekken 3, which waits in nothing, and the step's
own boundary (the interrupt check, the delays, the completion) is most of it. A block executor would do that work once
per run of instructions between events, and a recompiler on MarsRT's Cranelift path would remove the dispatch; both
are exact only with the timing model inside them, fetch by fetch, and both are larger than this stage's other parts
together. Neither was needed for G6 (§6.4); both are recorded as the levers left.

## 8. Proposals, not built

### 8.1 The speed settings that are not exact (`effect: accuracy`)

The plan's §5.12 lists three, to be offered only after the exact levers; none is built here, and each would be a
setting in the core's settings category with its cost stated:

- **The CPU's timing model reduced to a fixed cost an instruction.** It would let the step drop the I-cache's tag
  compare, the write queue and the stall arithmetic, and is the largest of the three; games that time themselves with
  the root counters against instruction counts (D-4's `access-time` and amidog's timing loops) would run at other
  speeds, and the timers' records would no longer equal the console's.
- **The GPU's drawing time made instant.** Little speed by itself, since drawing is done at once already and only its
  time is charged; it changes GPUSTAT's busy bits and the DMA's pacing (P5 predicts at least one bench game behaves
  differently).
- **The CD-ROM controller's delays shortened.** Loading, not frame time: fewer frames spent waiting on a sector,
  which the idle skipper already makes cheap; it changes every game's timing from the first read.

### 8.2 The shipped link

The aligned link measured within 3.4 per cent of the default link on every bench game (§6.1): faster by 1 to 3 per
cent on all seven with the levers, slower by 0.5 to 3.4 per cent on all seven without them, so whether it helps is
itself a matter of placement, and it buys no speed; what it buys is a figure that does not move with the next
unrelated change. Proposed for the tester's decision [C]: keep the default link for what ships, and use the
aligned link for every measurement of a lever, as this stage did.

## 9. Retired predictions

- **P2** (*a heavy 3D scene misses the budget with the interpreter alone and meets it with the threaded rasteriser
  and the decoded-instruction cache, before the recompiler*): the first half held at the quota for three games before
  the stage (Ape Escape 16.9 ms, Spyro the Dragon 18.7, Tekken 3 16.3 against 13.4); the second half named the wrong
  levers. The budget was met by idle skipping and the decoded cache's first tier, with no threads: the cost the plan
  priced as the interpreter's was in good part the interpreter waiting.
- **P12** (*the handheld runs the accurate core at full speed on battery after stage 13*): held, and held before it:
  the emulation thread took 4.2 to 5.7 ms a frame on the handheld's battery before the stage and 1.2 to 4.5 after (§6.3).

## 10. Reading log

Read for this stage: `Diamond_Plan.md` §5.1, §5.2, §5.12, §7, §8 and §13; `Diamond_Native.md` §11.4, §11.7 and §19;
`Diamond_Debugger.md`; `BlackMoon_R3000.md` §2 and §5; `Mars_Native.md` §5.2 (RunIdle), §6.12 (the signal
processor's decoded table) and its sampler's notes; `Mars_Recompiler.md` §15 (the idle loop); `VenusRT_Native.md`'s
quota measurements; and the project's own code: MarsRT's `cpu/idle.rs`, Diamond's bus, machine, I-cache, memory
control, debugger and state, `blackmoon-r3000`'s step and coprocessor 0, and `emusen-mips`'s decoder. No emulator's
source was read; the player's BIOS was not opened, copied or run; the open kernel ran as the bench's kernel, its image
untouched.

---

# The second pass (2026-10-12)

*Written 2026-10-12, with the tester's request that the work return to speed after G6.* Sections 11 to 18 record
the pass: the base re-recorded, the profile, the two levers built (runs of steps with the boundary's work held, and
the rasteriser painting a row at a time), how each is known to be exact, what they bought on the desktop and under the
quota, what was measured on the handheld and what is owed there, the frontend's share of a frame, and what was not
built. The build record's entry is
`Diamond_Native.md` §22; the processor's side is `BlackMoon_R3000.md` §13. The marks are those of §1.

## 11. The base, and why it moved

Three trees were measured. **c6c3a39e** is the tree the pass started on: D-46 (a triangle's time by its rows), the
open kernel's exception path in RAM and the game fixes had merged since §6, and the games reach further than §6's
tree did, so its figures are not §6's. **414cac36** is WiseMan after the open kernel's image c1a002e8 (LoadExec
masking I_MASK, synchronous CD operations masking IRQ2): the games' timing changed with the kernel, so every digest
and every figure in §14 is taken against a base built from 414cac36 itself, `b2` in the probe's `speed2/`. The levers
were built on c6c3a39e and merged onto 414cac36; on both trees the levers' digests equal the base's.

On c6c3a39e the desktop's aligned medians were Ape Escape 3.23 ms, Castlevania 4.40, Crash 3.48, Rayman 2.52, Silent
Hill 3.01, Spyro 3.21 and Tekken 3 3.42 [M, `cmp.py`, three rounds]: Castlevania had grown from §6's 3.72 to 4.40,
over the plan's 3.8, because it now plays where it had waited (the skipper passes 7.7 per cent of its clock, against
§4.5's 17.3). On the handheld's battery (BAT0 discharging, 96 to 95 per cent), the run §6.3 owed, the same tree took
3.48, 4.70, 3.69, 2.65, 3.19, 3.37 and 3.56 ms [M, `hh-base-bat.txt`]; its digests equalled the desktop's.

## 12. The profile

[M, 2026-10-12, `rtsample.py` at 2 ms on c6c3a39e's base, about 6,000 to 8,000 samples a game; `report.py` and
`split.py` in the probe's `speed2/` group the samples by their inline chains.]

| Part | Castlevania | Spyro the Dragon | Tekken 3 |
|---|---|---|---|
| The CPU's step and the bus | 52.6% | 46.4% | 66.9% |
| The GPU (pixels, texels, the texture cache, the display read) | 32.9% | 32.2% | 6.2% |
| The SPU | 4.6% | 5.6% | 6.1% |
| The MDEC | 1.1% | 0.2% | 8.5% |
| The bench's own digests (outside the timed span) | 6.4% | 11.1% | 5.4% |

The GPU had grown from §3's 3 to 11 per cent to a third of the two slowest games: Castlevania's play draws 155,000
texels and 111,000 pixels a frame, 122,000 of them in five 8-bit modulated rectangles, its backgrounds; Spyro draws
470 primitives a frame, most of them dithered triangles of 13 to 32 pixels a row [M, a counting build kept out of the
tree]. The plan's §7.1 is therefore no longer a lever of 3 to 11 per cent.

**Where a step's time went.** A count of the host's own events (`pstat.py`, `perf_event_open` on the bench) gave
Tekken 3 4.0 instructions a cycle and 1.1 branch misses a thousand instructions: the step was not stalling, it was
long. A nine-instruction loop run in the I-cache with the levers off (`alu.s`) cost 5.9 ms a frame, 10.6 ns a guest
instruction; stepped under gdb one host instruction at a time it took 1,665 host instructions a turn, 185 a guest
instruction [M]. By source line they were: the boundary (IP2, the interrupt check with the MTC0 view, the observer,
the PC break, the fetchable check) 43; the fetch, the I-cache's segment tests, tag and valid bit, 31; building the
step's `Run` value and calling `execute`, 24; `execute` itself 53, of which the field decoding was 9; the completion
6; the frame loop's per-step call and the kernel's seams about 24. §7.2 had named the boundary as most of the step;
it was less than a quarter. The larger part was the shape of the step: one value carrying the instruction's operands,
delays and results through every arm, which the compiler kept in memory.

## 13. The levers

### 13.1 Runs of steps, and the common instructions' path

`blackmoon-r3000` gains `R3000::run(bus, limit)` and `Bus::running`, a default method answering no
(`BlackMoon_R3000.md` §2, §13). A run is the loop it replaces, `step` until a step is not an instruction, the limit,
or `running`'s no, with the boundary's work held while it cannot change: after a full step that leaves the boundary
quiet (the debug unit off, no MTC0 view pending, no interrupt the next check would take), the steps that follow make
only the line's comparison, the fetchable check, the fetch and the instruction, until the line moves or an
instruction that can change SR, Cause or DCIC runs. Inside the step, the instructions that compute, branch, jump,
load or store a byte, halfword or word take a path with their operands as parameters, and complete as `finish`
completes one; the rest keep `Run::execute`. In the games 98 to 99 per cent of the instructions take the new path
(Spyro 92.7, its GTE transfers and commands being the rest) [M, a counting build].

Diamond's bus answers `running` while its clock is short of the next event, which is exactly the condition under
which `step_on` runs no events after a step. `Machine::steps` makes a run where the kernel's two seams have nothing
to do (no PS-EXE waiting at the shell's entry, no tapped putchar), runs the last step's events after it, and counts
steps as the idle skipper's watch counts them, so that the skipper watches the same steps with runs as without
(§4.3). `Machine::runs` holds the switch (`--stepped` turns it off in the bench), the verifier and the counts; none of
it is state. The debugger's frame never runs. A run averages 87 to 715 steps in the bench games.

The I-cache fetch's segment test is one mask: bits 30 and 29 clear are KUSEG's first 512 MiB and KSEG0, which are the
segments `cached_hit` admits.

### 13.2 The rasteriser by rows

`Painter::row` paints a row's pixels from left to right, each texel fetched and each pixel written before the next,
the same order as before: the paint's settings are read once a row, the textured or the flat loop is chosen once, a
pixel's VRAM is read only where it is blended or the mask is checked (the old path read it always and used it only
then), a texture cache miss is filled out of line, and the counts the drawing time is charged by are summed in
locals. Triangles and rectangles call it with the per-pixel colour and texture coordinate their walks give; lines
keep `pixel`, which is a row of one. Per pixel on the desktop [M, `gpubench.rs`, a scratch tool beside the probe's
copy, the same pixels drawn by both builds, digest F4C33AB282682E69 for both]:

| Primitive | Before, ns a pixel | After |
|---|---|---|
| Rectangle, flat | 3.45 | 1.44 |
| Rectangle, flat, blended | 4.61 | 3.28 |
| Rectangle, 8-bit, modulated | 8.80 | 7.20 |
| Rectangle, 4-bit, blended, mask checked | 8.62 | 6.68 |
| Triangle, flat | 5.00 | 2.94 |
| Triangle, shaded, dithered | 5.54 | 3.57 |
| Triangle, 4-bit, shaded, dithered | 10.26 | 9.58 |
| Triangle, 4-bit, shaded, blended | 13.25 | 12.36 |

A loop specialised for the 8-bit modulated rectangle, Castlevania's backgrounds, measured 5.4 ns a pixel against the
row's 6.8; it would be a second copy of the pixel's rules, and was not kept [C].

### 13.3 How they are known to be exact

1. **The verifier.** `Runs::verify = N` (the bench's `--verify-runs N`) checks every Nth run as it is made: a copy
   takes the run's count of steps one at a time through `step`, must meet an event or a step that is not an
   instruction only at the last, must end in the same step, and must reach the run's state byte for byte with the
   next event and the events' count (`Machine::verified_run`).
2. **The crate.** The r3000 suite's 55,000 cases pass through the new path; `a_run_is_its_steps_on_random_programs`
   holds 19,801 runs over 400 random programs, interrupted, faulting, in user mode, isolated and with the debug unit
   armed, to a copy stepped one step at a time; `a_runs_boundary_is_the_steps` names the five boundaries a run must
   still see (`BlackMoon_R3000.md` §13).
3. **The machine's tests** (`tests/programs.rs`, the block *Speed, the second pass*): the project's nine programs, the
   CD read and ps1-tests' eight programs on the open kernel give the digests of their steps stepped in runs, with the
   skipper off and on, every 7th (251st for ps1-tests) run and every skip verified; a disc named by
   `EMUSEN_DIAMOND_DISC` the same over 600 frames; and a kernel of the test's own whose putchar is tapped is stepped
   one at a time, its calls met. `a_rows_texels_blend_by_their_own_bit_15_and_the_mask_keeps_its_pixels` holds what
   no test held, that a semi-transparent texture blends only its texels with bit 15 set and the mask keeps its pixels;
   it passes on the rasteriser before the change and after.
4. **The games.** The seven bench discs, 3,600 frames each, give the base's picture, sound and state digests every 600
   frames with runs, with runs and the rows, with every 97th run and 16th skip verified, and stepped one at a time, on
   both trees [M, `dig-b2.txt`, `dig-h2.txt`, `dig-h4.txt`, `dig-h2-stepped.txt` beside the bench].
5. **Mutants** [M, `mutants-r3000.py`, `mutants-diamond.py`, `mutants-gpu.py`, each applied, the tests run, the file
   restored]. In the crate, 24, of which 23 caught: the line not watched in a run, the PC not checked, the bus's no,
   the limit and an exception not ending the quiet steps, MTC0 and RFE not ending them (each alone), quiet with the
   debug unit on, with a view pending, with an interrupt due, the line taken as low, a second load not cancelling the
   first, the link one instruction early, a branch not landing the pending load, LB not sign-extended, SUB's overflow
   not raised, a faulting load or store completing, JAL not told to the observer, a store's datum read after the load
   lands, R0 written, a store's address error not raised. The survivor makes the quiet check stricter (a masked
   request then ends the quiet steps), which costs speed and changes no step. In Diamond, all 7 caught: a run going on
   at the event's cycle, a run never ending for an event, the last step's events not run, runs made while a program
   waits to be side-loaded or on a tapped kernel, KSEG1 or KUSEG past 512 MiB taken as cached. In the rasteriser, all
   6 caught, the two the existing tests missed (an opaque texel blended, the mask checked against nothing) by the new
   test.

## 14. Measured

Milliseconds of the emulation thread a frame, the mean over 3,600 frames, on 414cac36's tree: *base* is WiseMan
414cac36 (`b2`), *runs* the branch with runs alone (`h2`), *both* the branch's head with runs and rows (`h4`).

### 14.1 The desktop

[M, 2026-10-12, `cmp.py`: three rounds, the builds interleaved on one core, fastest and median of the means.]

| Game | Base, default | Base, aligned | Runs, default | Runs, aligned | Both, default | Both, aligned | Both ÷ base (aligned medians) |
|---|---|---|---|---|---|---|---|
| Ape Escape | 3.297 / 3.297 | 3.118 / 3.123 | 2.438 / 2.441 | 2.415 / 2.416 | 2.302 / 2.302 | 2.302 / 2.302 | 0.74 |
| Castlevania: SotN | 4.453 / 4.464 | 4.309 / 4.312 | 3.284 / 3.300 | 3.263 / 3.263 | 2.956 / 2.973 | 2.926 / 2.926 | 0.68 |
| Crash Bandicoot | 3.531 / 3.537 | 3.376 / 3.378 | 2.601 / 2.614 | 2.561 / 2.574 | 2.472 / 2.479 | 2.445 / 2.453 | 0.73 |
| Rayman | 2.543 / 2.550 | 2.375 / 2.390 | 1.775 / 1.797 | 1.760 / 1.769 | 1.803 / 1.805 | 1.763 / 1.775 | 0.74 |
| Silent Hill | 3.045 / 3.078 | 2.967 / 2.973 | 2.357 / 2.361 | 2.326 / 2.334 | 2.230 / 2.230 | 2.181 / 2.183 | 0.73 |
| Spyro the Dragon | 3.257 / 3.263 | 3.142 / 3.145 | 2.595 / 2.597 | 2.573 / 2.573 | 2.417 / 2.420 | 2.408 / 2.413 | 0.77 |
| Tekken 3 | 3.441 / 3.476 | 3.289 / 3.294 | 2.407 / 2.412 | 2.396 / 2.398 | 2.417 / 2.433 | 2.367 / 2.386 | 0.72 |

Runs bought 18 to 27 per cent of every game; the rows another 5 to 10 per cent where a game draws (Castlevania 10,
Spyro 6) and nothing where it does not (Rayman, Tekken 3). Every gain exceeds §2's 10 per cent in both links. Every
game is now at 1.8 to 3.0 ms against the plan's 3.8; Castlevania, over it before the pass, has the most margin it has
had.

### 14.2 The laptop's proxy, the 33 per cent quota

[M, 2026-10-12, the aligned builds, two rounds under `systemd-run --user --scope -p CPUQuota=33%`.]

| Game | Base | Both | Within 13.4 ms |
|---|---|---|---|
| Ape Escape | 10.313 / 10.331 | 7.626 / 7.655 | both |
| Castlevania: SotN | 13.929 / 14.165 | 9.855 / 9.876 | both only |
| Crash Bandicoot | 11.175 / 11.335 | 7.990 / 8.091 | both |
| Rayman | 7.842 / 7.897 | 5.751 / 5.783 | both |
| Silent Hill | 9.948 / 9.954 | 7.312 / 7.386 | both |
| Spyro the Dragon | 10.544 / 10.616 | 7.924 / 9.545 | both |
| Tekken 3 | 10.587 / 10.753 | 7.532 / 7.719 | both |

The base tree had lost G6's first term: Castlevania took 13.9 to 14.2 ms under the quota against 13.4. With the levers
every game leaves 3.5 to 7.6 ms of the frame. Spyro's second round (9.5) is the quota's own noise, as its first
(7.9) and the desktop's figures show.

### 14.3 The handheld

[M, 2026-10-12, `bench.sh` beside the bench's discs on the handheld, run by `systemd-run --user` with no Mistress
running, the default link, one run.] Only the base of c6c3a39e was measured there, on battery (BAT0 discharging, 96 to
95 per cent), which is the run §6.3 owed:

| Game | Base of c6c3a39e, battery | p95 | Worst frame |
|---|---|---|---|
| Ape Escape | 3.481 | 7.562 | 9.461 |
| Castlevania: SotN | 4.697 | 5.618 | 7.796 |
| Crash Bandicoot | 3.690 | 6.601 | 8.343 |
| Rayman | 2.650 | 5.032 | 6.630 |
| Silent Hill | 3.191 | 4.648 | 6.840 |
| Spyro the Dragon | 3.374 | 7.156 | 9.065 |
| Tekken 3 | 3.557 | 4.292 | 6.336 |

Its digests equalled the desktop's for that tree. The slowest mean, 4.7 ms, is 28 per cent of an NTSC frame, and the
worst frame of any game 9.5 ms: **the emulation thread was at full speed on battery before this pass.** The handheld
left the network before the levers were measured on it, and stayed away for the rest of the day. **Owed**: the base of
414cac36 and the branch's head, each on the charger and on battery. From the desktop's ratios (0.68 to 0.77, §14.1)
the head is expected at 1.9 to 3.3 ms there [A]; that is an expectation to be retired by the run, not a result. The
bench's disc copies and the one bench build are still in `~/diamond-bench/` on the handheld and are to be removed
when the runs are done.

## 15. What was not built, and why

- **§7.1, the rasteriser on worker threads.** The GPU is now a third of the two slowest games on the desktop, so the
  lever is worth more than §7.1 found; but under the quota a worker is charged to the same 33 per cent, and the
  display's line-by-line read of VRAM would make the emulation thread wait on the worker at every line drawn behind.
  With every game at 3.0 ms or less on the desktop and inside the quota's budget by 3.5 ms, it is not needed; it
  remains the lever for a higher internal resolution.
- **§7.2's block executor in the full sense** (decoded runs with the delays resolved per block, one tag check a line)
  and **the recompiler.** The second tier the profile named was the step's shape and its boundary, and runs took most
  of what those cost: a guest instruction is now about 119 host instructions where it was 185 (§12). What remains is
  the dispatch on the decoded operation, the per-fetch tag compare and the delays' bookkeeping, which a block form
  would remove only by keeping a second form of each instruction's rules.
- **The 8-bit rectangle's own loop** (§13.2): 20 per cent of that primitive's pixels, at the price of a second copy of
  the pixel's rules.
- **The inexact settings of §8.1** stay proposals.

## 16. The frontend's share

The handheld's Mistress logs a line each second (`general.log`, `[fps]`): the frames run, the core's time (*run*),
the frames offered to the render thread and shown, the copy and the draw, and the loop's time outside the core
(requests, audio, the hand-off, sleep and the rest).

**On the desktop, headless** [M, 2026-10-12, WiseMan's opt-in
`The_frame_loop_reports_its_pace_on_a_named_disc`, Castlevania for 30 s on the branch's head]: 59.8 frames a second
offered every second after the first; run 1.0 to 3.7 ms; requests 0.01, audio 0.04, hand-off 0.05 ms; the rest of the
16.7 ms frame asleep; content-paced at 59.83 Hz. Nothing outside the core costs a frame. The headless render timer
draws in software at 29.9 frames a second (draw 6.3 ms); it is the test host's renderer, not a player's, and the
handheld's own logs show the GPU path drawing every frame offered (shown 59.8, draw 0.53 to 0.58 ms).

**On the handheld** [R, its logs of 2026-10-12, read, not changed]: in the games launched from 00:53 to 13:00 the loop offers
59.3 to 60.3 frames a second with run 3.8 to 6.0 ms and the rest of the frame asleep; the seconds below 58 frames
are a launch's first and last (loading, the window closing). Three launches, 08:58, 08:59 and 08:59, ran every
second at 15.0 frames a second with run 0.6 to 4.1 ms and the scheduler's reason "speed not 100%": Mistress's speed
was Slow Motion, 25 per cent, which is a quarter of 59.83. The launch before them (08:56) ended with the same reason
in its last line. The speed is the pad menu's row "Speed < 100% >", which left and right step through Slow Motion,
Normal and Fast Forward, and it is kept by the window from one game to the next. **The slowness the tester reported
playing on the handheld in those launches was the speed setting, not the emulation and not the loop.** Whether a game
should start at Normal, or the menu's row should take a confirmation, is a question for the tester; it is not a
defect with a test, and nothing was changed.

## 17. Predictions retired

- **§6.3's** *the slowness ... is to be looked for in what surrounds it there*: held, and found in neither the loop
  nor the presentation but in the speed setting (§16).
- **§7.2's** *the step's own boundary ... is most of it* [A]: did not hold. The boundary was 43 of 185 host
  instructions a guest instruction; the step's shape was the larger cost (§12).
- **§7.1's** *the GPU is 3 to 11 per cent*: true of §3's tree, retired by the games' progress: a third of Castlevania
  and Spyro on c6c3a39e.
- **P12** (*the handheld runs the accurate core at full speed on battery after stage 13*): held on c6c3a39e's base on
  battery (2.7 to 4.7 ms, §14.3), the run §6.3 owed. The levers' own run there is owed.

## 18. Reading log

Read for this pass: this page's §1 to §10; `Diamond_Plan.md` §5.5 and §5.12; `Diamond_Native.md` §21;
`BlackMoon_R3000.md` §2 and §5; and the project's own code: `blackmoon-r3000`'s step, Diamond's machine, bus, I-cache,
idle skipper, GPU (the painter, the rasteriser, the texture cache, the display read), Mistress's frame loop, its
scheduler and speed controller, and WiseMan's PlayStation library tests. No emulator's source was read; the player's
BIOS was not opened, copied or run; the open kernel ran as the bench's kernel, its image untouched.

