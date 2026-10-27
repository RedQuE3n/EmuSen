# Sapphire: the scheduler's record, from §54

The build record of Sapphire's board and scheduler from §54 on, continuing `Sapphire_Native.md`, which is within a few
kilobytes of the size a page in the tree may have. Its sections keep that page's numbering: a section number with no
page named is `Sapphire_Native.md`'s (§20 is S11's scheduler, §43 S32's bench and profile, §44 S36's recompiler, §47
S33's idle skipping, §53 S34's decoded cache), `Sapphire_Recompiler.md`'s from §48, or this page's own from §54. The plan
is `Sapphire_Plan.md` (§5.1, §5.12, §5.13); disputes, were any raised, would be in `Sapphire_Disputes.md` from D-980.

## 54. The board brought along at events, not after every EE instruction (2026-10-26)

*S32's lever #9 (§43.8), named again by S33 (§47.12) and S34 (§53.6), taken on the tester's decision of 2026-10-26 that
speed comes first and exact levers first. Built on WiseMan `f6c51d5b9` (S37's loads and stores in blocks, S34's IOP
decoded cache, S33's idle skipping, S43's GS workers and D-860 beneath). It raised no dispute: it reads nothing of the
hardware the interpreter does not already implement, and every claim below is the interpreter's own behaviour, kept.*

### 54.1 In one paragraph

After every EE instruction the board is brought to its time (`System::step`'s `run_to`, the replay's `run_through`
after a block, `drive_iop` inside a skip). S32's profile priced that bringing along at 16 to 17 % of the frame on the
interpreter, and S33's and S34's at about a fifth of Jak and Daxter's loading frame on the recompiled tier, most of it
the IOP called at every EE time and the units asked whether they were idle. **Each unit's next event is now answered
where it is known, and a call that would do nothing is not made** (§54.3 to §54.6): the IOP's next instruction time
decides whether the IOP is called at all, and its service with nothing due moves only its sync time; between two events
the replay is shown the lines once and finds the interrupt check from Count's request alone; a VIF found idle keeps
that answer, and both VUs found idle keep it and owe their micro modes the clock moves, until something reaches them;
the IOP's console hook stops the IOP's run only at the hook, through a breakpoint `blackmoon-r3000`'s bus may now name.
**Nothing a program can see moves**: the boot (760 frames) and Jak and Daxter (900) give S34's digests of every picture,
the sound and the state on both EE tiers; S33's verifier holds over 30 frames of each; states saved between any two steps
load and run alike; ps2autotests' 78 texts are byte for byte S34's; five seeded faults are caught by programs written for
them (§54.7). **Speed** (§54.8): Jak and Daxter's loading window from 47.3 to 43.2 ms a frame (1.09 times, 21.1 to 23.1 frames a second); the windows bound by the GS's workers within the runs' spread. **What was
not built** (§54.9): bringing the IOP along only at SIF registers and SIF DMA, as `Sapphire_Plan.md` §5.1 drew it, is
not exact on this machine, and why is recorded.

### 54.2 What was built

| File | What |
| --- | --- |
| `src/iop/mod.rs` | `run_to` and `catch_up` answer a call at a time the IOP has reached inline, the run out of line (`run_on`); `run_watched` runs on between the hook's boundaries (§54.6) |
| `src/iop/bus.rs` | `service`'s quiet path (§54.3); the bus's `breakpoint`, not state, and `stop_before` |
| `src/sif/mod.rs`, `src/spu2/mod.rs` | `wake_iop_pending`, `wake_pending` and `due`: the questions `service` asks without taking anything |
| `src/board.rs` | `Times`, what a run of times shows the replay, with `quiet` for the times between events (§54.4); `Through`; the VUs' rest ended in `run_vifs` (§54.5) |
| `src/blocks.rs` | the replay's call site only: `Seen`, its `Times`, in place of the closure; the emitter and the dispatcher untouched |
| `src/vif/mod.rs` | `rest`, not state: `idle` known until `run_to`, `write`, `request`, `put`, `get` or a state load |
| `src/vu/mod.rs`, `src/vu/micro.rs`, `src/vu/pipe.rs` | the VPU's `rest`, not state: the clock move owed while both units stay idle, `wake` making it, `Micro::write_state_resting` writing a block as it stands once made |
| `blackmoon-r3000` (`lib.rs`, `exec.rs`, `boundary.rs`) | `Bus::stop_before(pc)`, by default false: `R3000::run` ends in front of the step at that PC; a test of it |
| `tests/scheduler.rs`, `tests/s54/mod.rs` | the step's programs and tests (§54.7) |

No dependency was added, and nothing of the state changed: `SPH2`'s layout is unchanged, every field added being a
cache or a rest that a state load clears. `blackmoon-r3000`'s change is an addition to its interface with a default
that a bus which does not name a breakpoint compiles away, so Diamond's code is the same; its tests pass (§54.7).

### 54.3 The IOP's next instruction, and its service with nothing due

**The calls.** `Iop::run_to(t)` runs the IOP while its clock is below `t`; S33 counted that a call at a time the IOP has
already reached does nothing (§47.4), which is most calls, since an IOP instruction takes at least eight EE cycles and an
EE instruction about one. S34's sampler put 1.84 % of all samples on Jak and Daxter's loading, about 4 % of the frame's
work, in `Iop::run_to` itself, and the disassembly showed nearly all of it in the function's entry and exit around that
empty test. The test is now made where the call is (`#[inline]`): `held || now · 8 < t`, the IOP's next instruction
time (`Iop::next_event`'s own figure) against the time; the run is the same function as before, called only then.
Exact because the call that is not made is one whose loop would not have run: it changed nothing, its debugger flag
`halts` being set and cleared around nothing.

**The service.** Before each run of instructions `IopBus::service` brings the units to the IOP's clock (`run_units`) and
holds the processor while the DMAC has the bus. With no event due (`next` past the clock), no wake from the EE's side
(SIF) or the SPU2's AutoDMA pending, no SPU2 sample due and the DMAC not holding the bus, `run_units` does nothing but
move `synced` to the clock: its wake test takes nothing, its event loop does not run, and `sync_spu2` makes no sample and
takes no wake. The quiet path makes exactly that move.

### 54.4 The lines between events

At a block's exit the replay brings the board through each instruction's time and is shown the lines after each, from
which it decides whether the interpreter would take an interrupt before the next instruction (§44.5). On the quiet path
(no event by the last time, the GIF, the VIFs and the VUs idle, no wake pending) the lines cannot change until the IOP's
side wakes the DMAC, so they are read once; S34's profile put the per-time closure at 1.6 % of the samples. Now
`Times::quiet` is told the lines once for all those times: it fills the lines the replay keeps, and finds the first time
at which an interrupt is due, which with the lines standing depends only on whether Count has reached Compare by then
(IP7). The answer with IP7 and without it is computed once; if neither is due no time stops the run, if the answer
without is due the first time does, and otherwise the first time at which Count's request has risen. The board brings
the IOP through the times up to that one, stopping at a wake as before, and calls `each` at the stop, so that the replay
keeps what it kept: the lines of every time run and whether the stop was an interrupt.

### 54.5 The VIFs and the VUs resting

`run_to`'s quick path, `run_through`'s quiet test and S33's segments ask the GIF, both VIFs and both VUs whether they are
idle at every time; for the VUs the question is two calls through `Box<dyn Micro>`, and S34's profile put 1.1 % of the
samples in `Pipe::idle_to` and `Pipe::idle`, 0.5 % more in the VIFs' tests. The answers change only when something
reaches the unit, so they are kept:

- **A VIF** found idle sets `rest`, cleared by every call that may change what `idle` reads (`run_to`, `write`,
  `request`, `put`, `get`) and by a state load; `idle` answers `rest` first. Its clock move (`time` to `t`) is made at
  each time as before: it is one comparison.
- **The VPU** found idle by `idle_to(t)` (both micro modes idle, each moved to `t`) sets `rest = t`. While it rests,
  `idle_to` answers yes and raises the rest to `t`, with no call into either micro mode: the moves are owed. `wake` makes
  them, `idle_to(rest)` on each, and ends the rest. The board wakes the VPU in `run_vifs`, which every COP2 instruction
  (`Bus::vpu`), every access to VU memory or a VIF, and every event passes through before anything reaches the units;
  `Vpu::start` wakes it too, for a caller outside the board. The state written while the units rest writes each micro
  mode's clock as the owed move would leave it (`write_state_resting`: `now.max(rest)`).

Exact because a micro mode's idle answer depends only on its own state and the units' Q and P pipelines, which only a
call through the board changes, and its clock's moves are `max`es: one move to the latest time leaves what the moves at
every time would, and nothing reads the clock of an idle unit but the state, which writes it moved. **The seeded fault
F2** (§54.7) is the wake in `run_vifs` left out: COP2's DIV then leaves VU0's quotient in flight past the time the
quick path would have landed it.

### 54.6 The console hook

While the IOP's console hook is armed (S10, `iop::tty`), S10 ran the IOP one instruction at a time, showing each
boundary to the hook (§36.3), because the hook acts in front of an instruction, at IOMAN's `write`, which no bus access
announces. `blackmoon-r3000`'s bus may now name a PC a run stops in front of (`Bus::stop_before`, checked where
`running` is, after each step that completed an instruction); a run that starts there runs its step. `run_watched` now
runs the IOP on with the hook as the bus's breakpoint while it is not at the hook, and at the hook takes the boundary as
before (the device's answer, or the call kept once its first instruction has run). Every other boundary did nothing in
S10's loop: the answer is refused, `at` leaves no call pending and `stepped` takes none. With the debugger's probe fitted
the loop steps as before. **Measured on the workloads, this changes little** (§54.8): the board brings the IOP to each EE
time, so a run is about one instruction long whatever the hook does; it matters where the EE's instructions are long
(a divide waited for, a skipped wait), which the console program of §54.7 makes so that its fault is seen.

### 54.7 The gate (measured 2026-10-26)

On the desktop beside other stages' work; the player's Americas BIOS `ps2-0230a-20080220.bin` executed through the
referees' link to it, never read; Jak and Daxter read in place from `~/.cache/emusen/probe/sapphire/discs/`;
ps2autotests from `~/.cache/emusen/probe/sapphire/ps2autotests/`. "Before" is WiseMan `f6c51d5b9`, built apart.

Every whole-machine oracle ran on the first base (`f6c51d5b9`) and again on the tree rebased onto `f986199a2` (the
EE recompiler's shared exits, which change the replay's call site beside this step's), with that tree as "before"; the
figures are the same on both.

| Oracle | Run | Result |
| --- | --- | --- |
| The bench's digests, both EE tiers, before and after | `examples/bench.rs`, the boot (760 frames) and Jak and Daxter (900), idle skipping and the GS workers on, interpreted and with `--blocks` | all eight runs on each base: `50F06A02F1B76860`, `2EE7B3D10DF3C401`, state `2051C6BB42228C53` and `7EEC08D4D07DB621`, `20BB66B5E91347C9`, state `83E7D9CAFA075D14`, S34's own (§53.3) |
| S33's verifier | `s33 verify`, 30 frames of each, the interpreter stepped after every block and through every skipped turn | held: 220,863,488 and 231,997,440 steps compared, the whole states equal at the end |
| States between events | `s33 state`: the boot's state at frame 400 and Jak and Daxter's at 300 loaded into a second machine, 100 frames on beside a plain one | 0 pictures, 0 sound, 0 states differing on both |
| States between any two steps | `alike_when_reloaded` (`tests/scheduler.rs`): the step's programs beside a machine loaded from its own state every 37, 97 or 499 steps | equal at every load |
| ps2autotests | `examples/s10.rs` on the 55 EE programs and the 23 IOP programs as CDs (§40.4), one process at a time | **52 of 55 and 23 of 23**, the same three failing as §36.5; all 78 texts byte for byte S34's |
| The project's programs | the crate's suite, 478 tests in 33 binaries, and on the rebased tree `blocks` (25), `scheduler` (5), `idle` (7), `s34` (4), `cop0` (21), `interpreter` (23) and the library (109) | passed |
| `blackmoon-r3000` | its library's 39 tests with `a_run_ends_in_front_of_the_buss_breakpoint` | passed |
| The blast radius (`blast.sh`) | Diamond's crates and WiseMan's `Diamond` and `SapphireTests` classes | passed: `blackmoon-r3000` 39, Diamond 301, its `precision` 8 and `v1` 5; WiseMan 80 of 80 (lavapipe) |

**The programs** (`tests/s54/mod.rs`), each written to make one of the calls the step does not make visible:

- `lockstep`: the IOP writes a count into SMCOM as fast as it can and the EE reads SMCOM 512 times after delays of 0 to 7
  turns, keeping each value with Count. The EE reads the register with the IOP where the board left it after the
  instruction before, so a time at which the IOP was not brought along shows as a value one store early.
- `sif_kick`: S33's paired program (§47.8) with the delays between the EE's SIF1 kicks varied: the IOP re-arms its SIF1
  channel for each packet and counts its polls until the packet lands.
- `vpu`: 48 rounds of a macro DIV whose quotient lands during a delay of 3 to 10 turns, read back by CFC2 with Count, VU0's
  microprogram by VCALLMS and VU1's by CMSAR1 after delays, VPU-STAT read while VU1 may run.
- `console`: the IOP registers an export table named `ioman` whose entry 7 is a `write` of its own, from its cache, and
  writes "Hi!" to standard output through it after delays; the EE divides and waits for each quotient, so that the board
  brings the IOP through several instructions at a time. Run three frames with the device answering and not.

Their records and the digests of their states after a fixed count of steps (and the console's calls and text) were
taken on the tree before the step and are the tests' expectations; `alike_when_reloaded` runs a machine beside one
loaded from its own state every 37, 97 or 499 steps, which keeps no rest, and compares the two whole at each load.

**Seeded faults**, each made alone, the crate's test named run, the source restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | an event computed one cycle late: the IOP's next instruction time taken one EE cycle late in `catch_up` | `lockstep`'s records and `sif_kick`'s counts (2 of 5 in `tests/scheduler.rs`) |
| F2 | a unit register store not treated as an observation: the VUs' rest not ended where the board reaches them (`run_vifs`), so a COP2 DIV's store into VU0's Q pipeline leaves the rest standing | `vpu`, the machine loaded every 37 steps differing after 74 (the end state equal: the quotient lands when CFC2 asks) |
| F3 | a SIF kick not bringing the IOP along: `service`'s quiet path taken with the EE side's wake pending | `sif_kick`'s counts |
| F4 | Count's interrupt found one cycle late in the replay (`Seen::ip7`) | 6 of 24 in `tests/blocks.rs`, the random programs with Count's and the timers' interrupts among them |
| F5 | the hook's breakpoint never met (`stop_before` always false) | `console`'s calls and text |

First forms of two faults were caught by nothing, and why is recorded. F2 survived three forms (the wake left out of
`Board::vpu`, then of `run_vifs` and `run_gif` too) while the VPU's own methods also woke it: COP2's DIV calls
`settle_q` before it writes the Q pipeline, so the rest always ended; the wake was then made in one place, `run_vifs`,
with `Vpu::start` the one method keeping one, and the fault in that place is caught. F5 survived on a console program
whose IOP ran uncached beside an EE in a two-instruction loop, where every run of the IOP was one instruction long and
every boundary a run's first.

### 54.8 Speed (measured 2026-10-26)

**The method**: `examples/s33.rs time … recompiler on`, the core's defaults (the recompiler, idle skipping, the GS's
workers automatic), each frame's `advance` timed and the windows' means taken; three rounds, each running both workloads
on the tree before (WiseMan `f986199a2`) and after (this step rebased onto it), the order of the trees alternated between
rounds; every run alone among this step's jobs in a capped scope at CPU weight 100 while other stages' work went on
beside it, the one-minute load average 2.5 to 8.1 around the runs.

| Window | Before, rounds 1–3 | median (spread) | After, rounds 1–3 | median (spread) | Ratio | Frames a second |
| --- | --- | --- | --- | --- | --- | --- |
| The OSDSYS boot, frames 0–690 | 177.1, 176.2, 174.6 | 176.2 (1.4 %) | 175.4, 180.3, 169.4 | 175.4 (6.2 %) | 1.00 | 5.68 → 5.70 |
| Its language screen, 690–760 | 128.8, 124.2, 125.1 | 125.1 (3.7 %) | 128.3, 127.6, 123.5 | 127.6 (3.8 %) | 0.98 | 7.99 → 7.84 |
| Jak and Daxter loading, 8–400 | 49.2, 46.6, 47.3 | 47.3 (5.5 %) | 46.1, 43.2, 42.3 | 43.2 (8.8 %) | **1.09** | 21.1 → 23.1 |
| Its memory card screen, 400–900 | 127.7, 105.1, 106.5 | 106.5 (21 %) | 113.2, 107.4, 106.5 | 107.4 (6.2 %) | 0.99 | 9.39 → 9.31 |

On Jak and Daxter's loading the ratio holds in every round (1.07, 1.08, 1.12). The other windows are bound by the GS's
workers (§53.6: the language screen waits on them for 57 % of the emulation thread's samples) or by the GS's arithmetic,
where the board's calls were a small part; their differences are inside the rounds' spread. **`jak-dock`**, the gameplay
workload's window (`Sapphire_Speed.md` §51; frames 6,061 to 6,360 from the state at 6,000, `--defaults`), one round on
WiseMan `3d56d5a79` and this step rebased onto it: 347.9 against 357.1 ms a frame (2.87 and 2.80 frames a second), the
median frames 341.7 and 341.9, so no change outside the noise of one pair on a window bound by the GS's workers; its
picture, sound and state digests equal (`43D839015A0E4DA1`, `2C225C73DB4D3F9D`, `E220B19BD26D8EBC`).

**The board's share** (S32's sampler, §43.4, on Jak and Daxter's loading, frames 100 to 400, before on `f6c51d5b9` and
after on this step's first build; shares of the samples that did not stop the thread in the sampler's futex wait, §53.6):
the scheduler and synchronisation from 8.5 to 5.2 %, the IOP brought to the EE's time (`Iop::run_to`'s own samples) from
4.0 % to none, the VU micro modes' idle checks from 2.8 to 0.8 %. The IOP's own work (its interpreter, bus and units)
rose from 8.7 to 10.3 % of a smaller frame: the same 381,541 instructions a frame, most of them still run one call at a
time (§54.9). What remains of the board on that window is the replay's entry (1.3 %, mostly loading COP0's and the
core's fields), `run_through` (1.4 %) and the VIFs' idle answers in the quick path (1.4 %).

### 54.9 What was not built, and why: the IOP brought along only at the SIF

`Sapphire_Plan.md` §5.1 drew the IOP as a unit "brought to the present only where another can observe it: ... the IOP at
SIF registers and SIF DMA", and this step's brief asked that the board and the IOP be brought along only at their events
or where the EE does something another unit can see. For the IOP that is not exact here, for three reasons found in the
code:

1. **The IOP wakes the EE's DMAC from any instruction.** A store of the IOP's to a DMA channel's register starts the
   channel at once (D-361, `write_io`'s `wake_from(now)`), the run stops after it, and the service that follows moves a
   word into a SIF FIFO, which wakes the EE's DMAC at the board's time *after the EE instruction during which it
   happened* (§44.5). That time is part of the DMA's timing, and the EE's next instructions may read what the DMAC then
   writes. An IOP run behind the EE cannot know where that wake would have fallen; an IOP run ahead would read SIF
   registers, IOP RAM through the EE's window and the drive before the EE wrote them.
2. **The EE reads the IOP's side at its instruction's granularity**: SIF registers and IOP RAM through its window are read
   with the IOP where the board left it after the instruction before, without a call (`Board::read`), so the IOP must
   stand at every EE instruction's time where such a read may come, which is every one.
3. **The IOP's per-call fields are state.** `synced` and `horizon` (`IopBus`'s clocks) and the SPU2's sample are set by
   each call's last service, so even a batched run that met every wake exactly would leave different bytes in `SPH2`
   unless its last call were the plain run's; reconstructing that call from the boundaries near the end was worked out
   and judged not worth its risk for the gain left after §54.3, about 1 to 2 % of Jak and Daxter's loading frame.

So the IOP is still brought to every EE time, and what was taken is the cost of the calls that do nothing (§54.3).
Within one block's replay the IOP runs two to four instructions, so batching it across a block's times would save at
most the service and run entry of those few. The plan's §5.1 is amended to say so.

### 54.10 Exposure, and what was observed

Running the player's BIOS and the game made the step's writer an exposed writer of a player's-BIOS stage, never a
replacement writer (`Sapphire_Plan.md` §1.4). No firmware was opened, hashed, listed or dumped, and nothing printed or
kept holds a program counter, an instruction or memory of the BIOS or the game: counts, cycles, digests, times and the
samples of Sapphire's own host functions. No emulator's source was read, and no article.

### 54.11 Where the plan was wrong, or left open

- §5.1's catch-up of the IOP at the SIF alone (§54.9).
- S32's lever #9 expected 1.13 / 1.14 / 1.05 on the interpreter's path before S33, S34, S36 and S37; on the recompiled
  tier after them it is 1.09 on Jak and Daxter's loading and within the noise on the windows the GS's workers bound,
  since S33, S36 and S37 took most of what it addressed: the skipped turns' calls, and the calls inside blocks.
- The recompiler's dispatcher spends 1.5 % of the samples on Jak and Daxter's loading in `Blocks::translation`'s key
  comparison at every entry, and the replay's entry 1.3 % loading COP0 and the core's fields; neither is the board's,
  and both are named for the recompiler's steps.

### 54.12 Questions this step leaves

1. **Q1 [routine], the pinned records.** `tests/scheduler.rs` holds digests of whole states taken before the step; a later
   stage that changes the state's layout or the machine's timing on purpose re-takes them, as `version-5.layout` is
   re-pinned. *Recommended*: keep them; they are what makes this step's claim of no change a test.
2. **Q2 [routine], the IOP across a block's times** (§54.9, reason 3). *Recommended*: not taken; revisit only if a
   workload shows the IOP's service and run entry above 3 % of a frame.

### 54.13 What was read for this step

`Sapphire_Plan.md` §1.4 and §5.1; `Sapphire_Native.md` §30.78 to §30.85, §43.6 to §43.8, §44.5, §47 and §53;
`Sapphire_Recompiler.md` §48.5; `BlackMoon_R3000.md` §2 and §13; the core's `board.rs`, `blocks.rs`, `machine.rs`,
`idle.rs`, `iop/*.rs`, `sif`, `spu2`, `vif`, `vu` and `ee/cop2.rs`, and `blackmoon-r3000`'s run loop.
