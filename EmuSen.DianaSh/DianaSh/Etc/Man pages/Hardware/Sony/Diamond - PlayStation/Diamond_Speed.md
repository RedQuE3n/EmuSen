# Diamond_Speed — the exact levers, stage 13

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
start). A run on battery of the current tree is owed; it needs the handheld unplugged.

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
