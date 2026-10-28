# Sapphire: the speed record of the games' workloads, from §51

The record of Sapphire's speed measured on the games themselves, from the gameplay workload on, continuing
`Sapphire_Native.md`, which grew past the size a page in the tree may have, as `Sapphire_Recompiler.md` does for the EE
recompiler. Its sections keep that page's numbering: a section number with no page named is `Sapphire_Native.md`'s (§43
is S32's bench and profile, §50 S43's GS workers), `Sapphire_Recompiler.md`'s from §48, or this page's own from §51. The
plan is `Sapphire_Plan.md` (§5.12, §5.13, phase G of §7); the disputes are in `Sapphire_Disputes.md`.

## 51. A gameplay workload: Jak and Daxter toward play, timed and profiled, and the levers ranked again (2026-10-26)

*Numbered §51 and its disputes from D-900, the stages beside it (S37's §48 and S34's §53) taking the numbers around
it; measured on WiseMan `4bfd76232` (S43), before S37 and S34 merged. Decided 2026-10-26 by the tester: speed first, and the decisive measurement is a game in play.*

### 51.1 In one paragraph

Every measurement before this section was of a boot, a menu or a loading screen. **Jak and Daxter does not reach play
on Sapphire**: run from power-on on the Americas BIOS by a press script, it passes its memory card message, title and
new game into an opening of about 9,000 frames that no button skips, draws it whole to frame 11,700, and from about
12,900 shows four stale pictures in turn while the GS holds its writes behind a SIGNAL (**D-900**, open; neither S43's
workers nor S33's skipping makes it). So the workload is the nearest to play the core reaches: **the opening, drawn in the
game's engine** (characters, scenery and effects through VU1 and the GS, as play draws them), three windows from saved
states, one command each in the bench. **On the default path at CPU weight 100: the dock at night 361.1 ms a frame
(2.77 frames a second), Misty Island from the sea 564.1 (1.77), its arena 1,337.4 (0.75)**, against the menus' 49 to
72 ms (§50.7): the in-engine 3D is 5 to 27 times slower than every screen measured so far, and 22 to 80 times short of
the NTSC frame. The profile explains it unit by unit (§51.6): **the EE's floats in software (6 to 10 %), VU micro mode
(18 to 24 %), the GIF and the VIF (12 to 14 %), the GS's emulation-thread waits for CLUT reads on the dock (18 %), a hash
map keyed by name on every interpreted VU pair and COP2 instruction (4 to 5 %), and on the arena VU1's compiler joined
on the emulation thread at its code budget (frames of 2 to 50 s, 16 to 29 % of the window)**; the EE's recompiler
compiles only 34 to 37 % of the instructions run and the interpreter tier is as fast. The levers are ranked again
(§51.8): cheap exact fixes first, then S37 to S39 together; with every exact lever listed, the windows would still be 12
to 38 times short. The handheld's columns are owed.

### 51.2 What was built

- **The pad's input as a script** (`examples/common/script.rs`), shared by the runner and the bench so that the run that
  reaches a window and the bench's timing of it hold the same buttons and sticks at the same frames: `press <frame>
  <bits> [frames]` as S28 had it, overlapping presses now held together, and **`stick <frame> <axis> <value> [frames]`**
  (port 1's left X, left Y, right X, right Y, -1.0 to 1.0, down positive; the later begun line holds an axis, an axis no
  line holds is centred). `tests/script.rs` holds the rules.
- **The runner** (`examples/s28.rs`) reads a game's script with `script=<file>` before its control file, keeping the
  presses apart from a run's pictures and states, and drives the sticks through the core's `set_axis`. A save at an
  instant where a queue is past its slots (§38.2) no longer stops the run: the runner names the unit and saves at the
  first later frame whose state fits (§51.3).
- **Jak and Daxter's script** (`examples/scripts/jak-and-daxter.txt`): power-on to the opening, each press commented.
- **The bench** (`examples/bench.rs`) gains the workload **`game <image>`**: the disc in the drive at power-on and the
  BIOS booting it, with a DualShock 2 in each port and the console's erased card in slot 1 as a host's create makes the
  console, so that the runner's states are the bench's machine; `--nvm`, `--state`, `--script` and `--save-at N FILE`.
  Its frames and `--from` are the machine's own count, so that a run from a state times the frames a run from power-on
  would. **`--window NAME`** names one of `WINDOWS` (`jak-dock`, `jak-island`, `jak-arena`), the script built into the
  binary; **`--defaults`** is the levers as a player has them (`--blocks --idle --gs-workers auto` and **`--vu1`**,
  VU1's recompiler, which `Machine::new` does not choose by itself, so that S43's series ran VU1 interpreted). Over the
  window the bench now prints the recompiler's compiled and interpreted steps, the cycles and steps idle skipping passed,
  and VU1's recompiler counts (pairs compiled and interpreted, microprograms compiled and found changed, the budget's
  flushes). One command times a window:

      bench game <bios> americas <Jak's image> --window jak-arena --state <the state at frame 9000> --defaults

  the state saved once with `--save-at 9000 FILE` on the same window from power-on (about two hours at today's speed),
  or by the runner with the script.

### 51.3 Toward play: what the script reaches (measured 2026-10-26)

The runner from power-on, the Americas `0230a` BIOS by the link the step was given, Sapphire's configured EEPROM
(§39.2), the console's erased card in slot 1, the player's defaults (recompilers, idle skipping, four GS workers):

- **Frames 0 to 3,400**, S28's presses (§39.4): Cross at the memory card message (1,100), Start at the title (2,700),
  Cross on NEW GAME (2,820), Cross at the second memory card message (2,960); the opening from about 3,300.
- **The opening cannot be skipped**: Start at 3,410 pauses it ("PAUSE" drawn) and Start at 3,510 resumes it; Square,
  Cross, Triangle and Circle at 3,530 to 3,620 change nothing on screen. These six presses are in the script, so every
  later frame number is that of a run that made them.
- **The opening drawn**: the credits over a nebula, the village at night, the boat and the dock (6,000), Misty Island
  from the sea (7,500), the arena where the Lurkers gather (9,000), the precursor silo and the dark eco (10,800 to
  11,700). By eye nothing is missing or misdrawn, but at one cut near 10,500 to 10,650 (a half-drawn sky, then a picture
  ghosted and combed), which may be the game's own transition; no referee was run on the opening.
- **It stops (D-900)**: from 11,775 every picture is ghosted and combed, and from about 12,900 to 15,100, where the run
  was ended, the display cycles four stale pictures while the GS holds more than 64 writes behind a SIGNAL. Triage from
  the state at 10,531: one GS thread and idle skipping off each give the same pictures byte for byte to 12,000, so
  neither lever makes it. `Sapphire_Disputes.md` D-900 holds the observations; the pictures are in the step's scratch.
- **The state's bounds** (§38.2) were met in a game for the first time: no state could be saved at frames 10,500 to
  10,530 (the GIF's queues past their slots) nor from 13,500 to the end (the GS's held writes), which a player would see
  as a refused save and a gap in rewind. The runner saves at the first frame that fits.
- **Determinism**: the run resumed from the state at 9,000 gave pictures equal byte for byte to the uninterrupted run's
  at 9,300, 9,600 and 9,900; the bench from the state at 6,000 gave the runner's picture at 6,300; and each window's
  rounds gave one set of digests (picture, sound and state), on the recompiler tier and on the interpreter tier alike.

"Jak running, the camera turning" was therefore not reached; the windows are the opening's, chosen as Diamond chose its
bench (`Diamond_Speed.md` §2): what the game itself draws, not the project's programs. A first series, run with the
bench's `game` workload plugging port 2 empty while the runner's console had a pad there, drew different pictures from
the runner's and was discarded; the numbers below are the second series, on the runner's console.

### 51.4 The windows

| Window | The state | Timed | What is drawn |
| --- | --- | --- | --- |
| `jak-dock` | frame 6,000 | 6,061 to 6,360 | the village's dock at night, Jak and Daxter at the boat |
| `jak-island` | 7,500 | 7,561 to 7,860 | Misty Island from the sea, then the shore and its bones |
| `jak-arena` | 9,000 | 9,061 to 9,360 | the arena, the Lurkers' gathering with particles, the characters close |

Each is timed after 60 frames that refill the recompilers' caches from the state. No stick is held: the opening takes no
input.

### 51.5 Speed (measured 2026-10-26)

**The method**: the bench with `--defaults`, each frame's `advance` timed, the window's mean; three rounds of the three
windows, their order rotated; every run alone among this step's jobs in a capped scope at **CPU weight 100**, other
stages' work beside it at weight 20; the one-minute load average before and after each run.

| Window | Mean ms a frame, rounds | Median | Frames a second | Spread | Median frame; p95 | Load (one minute) |
| --- | --- | --- | --- | --- | --- | --- |
| `jak-dock` | 361.1, 352.6, 403.5 | **361.1** | **2.77** | 14 % | 347 to 394; 412 to 455 | 5.1 to 8.8 |
| `jak-island` | 609.3, 564.1, 563.2 | **564.1** | **1.77** | 8 % | 534 to 570; 841 to 1,032 | 5.1 to 9.1 |
| `jak-arena` | 1,382.7, 1,132.7, 1,337.4 | **1,337.4** | **0.75** | 22 % | 812 to 1,050; 1,440 to 2,072 | 5.9 to 12.3 |

The dock's third round ran at a load of 7 to 9. **The arena holds 4 to 18 frames of 2 to 50 seconds each**, 55 to
122 s of each round's 340 to 415 s; without them its mean is 963 to 1,072 ms, and its spread is theirs, their length
being the compiler thread's progress (§51.6), while the emulated result is the same in every round. The dock holds one
such frame in one round, the island none.

**The tester's "about one frame a second"** is confirmed for the later opening: the runner, at CPU weight 100 and loads of
4 to 12, ran frames 9,000 to 12,000 at 0.7 to 1.9 frames a second (its log every 300 frames). The menus run 13.9 to 20.3
(§50.7): **the in-engine 3D is 5 to 27 times slower than every screen measured before**, 22 to 80 times short of
16.68 ms.

**What a frame holds** (the bench's counts a frame over the window, the same in every round):

| | `jak-dock` | `jak-island` | `jak-arena` | Jak's memory card screen (§43.6, §47.11) |
| --- | --- | --- | --- | --- |
| EE instructions | 5.79 million at 0.85 cycles | 5.76 million at 0.85 | 5.70 million at 0.86 | 5.58 million at 0.88 |
| of which passed by idle skipping | 4.12 million (73.3 % of the cycles) | 3.83 million (68.1 %) | 1.71 million (30.5 %) | 91.3 % of the cycles |
| run: compiled / interpreted | 0.61 / 1.05 million (36.8 % compiled) | 0.69 / 1.24 (35.6 %) | 1.33 / 2.65 (33.5 %) | |
| IOP instructions | 355,830 | 377,891 | 348,228 | 348,298 |
| VU0 pairs (interpreted, S41 not switched on) | 247,432 | 123,392 | 359,683 | 41,886 |
| VU1 pairs: compiled / interpreted | 347,579 / 17,519 | 894,400 / 325,100 | 1,168,926 / 390,282 | 24,604 in all |
| VU1 microprograms compiled, found changed | 0.36, 0.62 | 2.13, 3.80 | 3.21, 5.57 | |
| VU1's budget flushed (300 frames) | 1 | 6 | 8 | |
| GS primitives, shared by the workers | about 6,900 | about 36,500 | about 28,900 | |
| the emulation thread's waits for the workers | about 48 ms (CLUT reads) | under 1 ms | about 4 ms | none |

**On the tree with S37 and S34** (WiseMan `f986199a2`, merged after the series; one round of each window, loads 1.9 to
5.8): the same digests in every window, S37 being exact; the recompiler compiles 46.2, 41.5 and 36.5 % of the steps run
(36.8, 35.6 and 33.5 before), and the means without the long frames are 338, 581 and 942 ms against 347 to 398, 534 to
609 and 963 to 1,072 in the series: within the series' spread. The island had two long frames here (57 s), its VU1
budget's join finding a long queue this time. S37 alone does not move the 3D; §51.8's ranking stands on this tree.

VU1 is the 3D's unit: 1.2 to 1.6 million pairs a frame on the island and the arena, the range §5.12 priced (1 to 3
million), against 25,000 on the memory card screen; VU0's micro mode runs 0.1 to 0.4 million pairs, interpreted; and on
the arena idle skipping passes under a third of the EE's cycles.

### 51.6 Where the frame goes (measured 2026-10-26)

**The method**: S32's sampler (§43.4) at 4 ms on **every thread** of one run of each window (`THREAD=all`), from the
window's first progress line to its end, at loads of 2 to 12; the emulation thread's samples classed by S32's rules,
with the recompiler's dispatcher and exit replay (`blocks.rs`, `ee/jit`) under the EE, idle skipping apart, and code
with no symbol (the compiled blocks and microprograms, whose perf map was not written because it names guest addresses)
as "compiled code". Each share is multiplied by the timed median (§43.7). Only host functions were named. A copy of the
sampler was changed to stop when the program's threads have exited (its first form waited out its whole time on one).

| The emulation thread | `jak-dock` (22,842 samples) | `jak-island` (34,224) | `jak-arena` (54,689) |
| --- | --- | --- | --- |
| **EE**, all of it | 30.4 % (110 ms) | 31.4 % (177 ms) | 29.9 % (400 ms) |
| of which **COP1's floats in software** (`float.rs` under the FPU) | **6.4 % (23 ms)** | **10.4 % (59 ms)** | **8.1 % (108 ms)** |
| of which the recompiler's dispatcher and exit replay | 7.3 % | 5.9 % | 5.1 % |
| of which the timing model, translation, fetch, data, decode, step | 16.6 % | 15.1 % | 16.8 % |
| **Compiled code** (EE blocks and VU1 microprograms) | 3.5 % (13 ms) | 4.8 % (27 ms) | 3.8 % (50 ms) |
| **VU micro mode** (VU0 interpreted, VU1's interpreted pairs, the float operations VU1's code calls, its dispatcher) | **17.9 % (65 ms)** | **24.3 % (137 ms)** | **21.2 % (283 ms)** |
| **GIF** | 6.7 % (24 ms) | 7.5 % (42 ms) | 6.7 % (89 ms) |
| **VIF** | 5.3 % (19 ms) | 6.4 % (36 ms) | 6.0 % (80 ms) |
| **GS** on the emulation thread | **18.7 % (68 ms)**, 17.7 % publishing jobs and waiting for CLUT reads | 7.0 % (39 ms) | 2.9 % (39 ms) |
| **A hash map keyed by name** (`RandomState`, SipHash) | 4.8 % (17 ms) | 4.3 % (24 ms) | 3.9 % (52 ms) |
| **Blocked in a system call** | 1.6 % | about 1 % | **9.7 % (129 ms)**: VU1's compiler joined (below) |
| **Idle skipping's own work** | 0.9 % | 2.2 % | 5.5 % (74 ms) |
| Scheduler and synchronisation | 4.2 % | 4.4 % | 4.4 % |
| EE timers and INTC, DMAC, IOP, PCRTC, SPU2, SIF, IPU | 5.5 % | 6.3 % | 5.6 % |

| Other threads, share of samples not parked | `jak-dock` | `jak-island` | `jak-arena` |
| --- | --- | --- | --- |
| GS workers (four) | 34 to 37 % each | 37 to 65 % | 16 to 32 % |
| VU1's compiler thread | 7 % | **61 %** | **53 %** |
| the EE's compiler thread | under 1 % | 1 % | under 1 % |

What the tables say, unit by unit:

1. **The EE's floats.** Every COP1 operation is computed by `sapphire::float`'s software arithmetic (§5.3's exactness);
   the opening runs 160,000 to 210,000 COP1 instructions a frame (§51.7), and their arithmetic is 6 to 10 % of the frame,
   more than the recompiler's own work.
2. **VU micro mode.** VU0's micro mode is interpreted (S41's tier built, not switched on), and VU1 interprets 5 to 27 %
   of its pairs while its compiler catches up: on the island and the arena it compiles 2 to 3.5 microprograms a frame and
   finds 3.8 to 6.3 changed, so the cache of eight programs a start that held on the frames S40 measured (§45.7) is
   overrun, and its compiler thread is busy 53 to 61 % of the time.
3. **VU1's compiler joined at its budget.** The arena's long frames are one call: the stacks of the emulation thread
   during two of them (`eu-stack`, 30 samples, host functions only) show 23 of 30 in `vu::jit::run`'s budget check,
   which drops the `emusen-jit` compiler and **joins its thread while it compiles the microprograms queued**; VU1's 64
   MiB budget flushed 8 times in the arena's 300 frames and 6 times in the island's, and where the queue was long the
   emulation thread waited it out. 16 to 29 % of the arena's time; the island's flushes found short queues.
4. **The name map.** `vu::ops::entry` finds an operation's table entry through a `HashMap<&str, usize>` on its name, and
   is called for every pair the VU interpreter steps and for each COP2 macro instruction's issue (`ee::cop2`, in the same
   stacks): SipHash on a name, 4 to 5 % of every window. The index carried in the decoded entry would do it exactly.
5. **The GIF and the VIF**, 12 to 14 % together: the unpacks and PATH transfers that feed 7,000 to 36,000 primitives a frame.
6. **The GS** costs the emulation thread little where it does not read: on the dock about 48 ms a frame waiting for CLUT
   reads behind the workers (2,968 waits in a run of 360 frames, §50's `clut` count), elsewhere under 5 ms. The workers
   are not the bound: each is parked half the time or more.
7. **The EE's recompiler buys nothing here.** Its blocks end at every load, store, MMI, COP1 and COP2 instruction, 38 to
   58 % of the instructions run (§51.7), so 63 to 66 % run interpreted, and the interpreter tier timed beside the counts
   was as fast (dock 391.4 against 350.3 ms, island 559.2 against 566.4, arena 1,087.9 against 1,109.8; single rounds, the
   digests equal). S36's 1.49 on Jak's start (§44.8) was integer code; the 3D's EE code is not.
8. **Idle skipping** passes 68 to 73 % of the EE's cycles on the dock and the island but 30.5 % on the arena, where its own
   proofs and refusals cost 5.5 % of the frame.

### 51.7 The EE's instruction mix (measured 2026-10-26)

Counts only, by the instruction word's major opcode, in a copy of `Ee::step` built apart and not committed (as §46.2
counted `gs::exact`); no address, word or register was kept. On the interpreter tier every instruction run passes
`Ee::step`; on the recompiler tier only the interpreted ones do, so the last column is what ends the blocks. A frame,
over each window:

| Class | `jak-dock` | `jak-island` | `jak-arena` | Interpreted on the recompiler tier (dock / island / arena) |
| --- | --- | --- | --- | --- |
| MMI (opcode 1Ch) | 240,863 (14.5 %) | 233,041 (12.1 %) | 306,368 (7.7 %) | all |
| COP1, LWC1 and SWC1 | 159,564 (9.6 %) | 170,925 (8.9 %) | 209,479 (5.3 %) | all |
| COP2 macro operations | 77,088 (4.6 %) | 96,150 (5.0 %) | 108,038 (2.7 %) | all |
| COP2 moves and branches, LQC2 and SQC2 | 139,049 (8.3 %) | 102,668 (5.3 %) | 167,739 (4.2 %) | all |
| Loads (LB to LD, LQ, LDL and LDR) | 223,305 (13.4 %) | 245,051 (12.7 %) | 546,535 (13.7 %) | all |
| Stores (SB to SD, SQ, SDL and SDR) | 118,711 (7.1 %) | 111,486 (5.8 %) | 157,085 (3.9 %) | all |
| Branches and jumps | 140,652 (8.4 %) | 224,905 (11.7 %) | 695,124 (17.4 %) | 50,928 / 115,724 / 421,291 |
| The rest (ALU, shifts, multiplies, SYNC, COP0) | 566,693 (34.0 %) | 745,945 (38.6 %) | 1,795,805 (45.1 %) | 43,573 / 168,184 / 735,943 |
| All run | 1,665,925 | 1,930,170 | 3,986,173 | 1,052,686 / 1,242,836 / 2,652,166 |

S37's loads and stores are 18 to 21 % of the instructions run, S38's MMI 8 to 15 %, S39's COP1 and COP2 12 to 23 %;
together 38 to 58 %, and every one ends a block today. The arena's EE runs more integer code and branches, which idle
skipping does not pass.

### 51.8 The levers and compromises, ranked again for the 3D

As §43.8: the share of each window's frame a lever addresses (dock / island / arena), the bound 1 / (1 − s), the
expectation under its assumption, the work in stages, its kind, and the gain per stage. The shares are of the default
path (S33, S36, S40 and S43 in).

| # | Lever or compromise | Kind | Share (%) | Bound | Expected (assumption) | Work | Gain a stage |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | *New:* **VU1's compiler not joined at its budget** (the old module left to finish on its thread, or its queue dropped) | exact | 0 / 0 / 16–29 | 1.00 / 1.00 / 1.2–1.4 | 1.00 / 1.00 / **1.25** (the joins gone) | 0.25 | 1.0 on the arena |
| 2 | *New:* **the name map out of the VU's decode** (`vu::ops::entry` by index) | exact | 4.8 / 4.3 / 3.9 | 1.04–1.05 | **1.04 to 1.05** (nine tenths gone) | 0.1 | 0.4 |
| 3 | **VU1's microprogram cache sized for the 3D** (more programs a start, cheaper checks of changed words), with #1 | exact | VU1's interpreted pairs: about 0.5 / 7 / 5 | 1.01 / 1.07 / 1.05 | 1.00 / 1.03 / 1.02 (halved) | 0.25 | 0.1 |
| 4 | **VU0's recompiler switched on** (S41, built and graded) | exact | VU0's interpreted pairs: about 13 / 4 / 6 | 1.15 / 1.04 / 1.06 | 1.07 / 1.02 / 1.03 (half, S40's ratio) | 0.25 | 0.16 |
| 5 | *New:* **CLUT reads without waiting for the workers** (§50.10 Q2) | exact | 17.7 / 4.0 / 1.7 | 1.22 / 1.04 / 1.02 | 1.15 / 1.04 / 1.02 | 0.5 | 0.3 on the dock |
| 6 | **The EE recompiler's remaining stages, S37 to S39 together** (loads and stores, MMI, COP1 and COP2 compiled, the floats still called) | exact | the EE less its floats, and its compiled code: 27.5 / 25.8 / 25.6 | 1.38 / 1.35 / 1.34 | 1.16 / 1.15 / 1.15 (that cost halved, blocks unbroken) | 3 | 0.05 |
| 7 | *New:* **the FPU's floats on the host where proven equal** (an exact fast path, as §46 did for the GS) | exact where proven | 6.4 / 10.4 / 8.1 | 1.07 / 1.12 / 1.09 | 1.05 / 1.08 / 1.06 (three quarters of the classes) | 1 | 0.06 |
| 8 | **C3**, the floats on the host's arithmetic with clamping, the FPU's and the VU float calls | byte-changing | about 13 / 17 / 14 | 1.15 / 1.20 / 1.16 | 1.11 / 1.15 / 1.12 | 1.5 | 0.08; needs the tester's acceptance |
| 9 | **VU1 and the IOP on threads** (S42) | exact | VU1's work on the emulation thread with its code and calls: about 8 / 15 / 12; the IOP 1 | 1.09 / 1.18 / 1.14 | 1.05 / 1.10 / 1.08 (its waits taking a third) | 1 | 0.08 |
| 10 | *New:* **the GIF's and the VIF's paths** (unpacks and PATH3 qwords in runs) | exact | 12.0 / 13.9 / 12.7 | 1.14 / 1.16 / 1.15 | 1.06 / 1.07 / 1.07 (half) | 1 | 0.07 |
| 11 | *New:* **idle skipping's own checks** on the arena's loops (its refusals and proofs) | exact | 0.9 / 2.2 / 5.5 | 1.01 / 1.02 / 1.06 | 1.00 / 1.01 / 1.03 (half) | 0.5 | 0.04 on the arena |
| 12 | **The board at events** (§43.8 #9, §47.12) | exact | the scheduler's: 4.2 / 4.4 / 4.4 | 1.05 | 1.03 | 1 | 0.03 |
| 13 | **C1**, the EE a fixed cost an instruction | timing-only | the timing model: 3.7 / 3.7 / 5.0 | 1.04–1.05 | 1.03–1.04 | 0.5 | 0.07; refused at §44.9 |
| 14 | **The GS on the device** (S44 to S46) | exact (C5, C6 apart) | the GS on the emulation thread: 18.7 / 7.0 / 2.9, most of the dock's #5's | 1.01–1.03 after #5 | 1.01 to 1.03; the workers' cores freed on the handheld | 3 | 0.01 |
| 15 | **C7**, bounded slack | timing-only | the scheduler's, and S42's waits | 1.05 | 1.03 | 0.5 | 0.06 |
| — | **C4** (refused, §45.9), **C8**, **C9**, **C11** | — | under 1 each, or none of the frame | — | — | — | — |

**Compared with the menus' ranking** (§43.8): there the GS's exact arithmetic, idle skipping and the GS's threads led,
and all three are taken; on the 3D none of them is a large share any more, though idle skipping still passes 30 to 73 %
of the EE's cycles, which unpassed would be the largest cost of all. What leads now was not on §43.8's list or was
ranked low: VU1's budget join, the VU's name map and VU0's tier, which the menus barely ran, the CLUT reads, and the EE's
floats, which S32 found at 0.3 % of a menu. The EE recompiler's later stages rank low per stage of work but are the
largest exact lever left in absolute terms. **Together, every exact lever listed (#1 to #7 and #9 to #12) would bring the
dock from 361 ms to about 200, the island from 564 to about 325 and the arena from 1,337 to about 640**: still 12 to 38
times short of 16.68 ms. No exact lever on the list closes the gap; it lies in every unit's cost per instruction and per
pair, an order of magnitude, as P17 predicted.

### 51.9 The handheld (owed)

Not run: the tester has said handheld runs come later. Owed: the three windows' rounds on the Legion Go S, on charger and
on battery, the bench built on the desktop and copied with the three states (the script is in the binary):
`bench game <bios> americas <image> --window jak-arena --state <state> --defaults` under `systemd-run --user`, and one
sampled run of a window. §43.9's and §50.10's handheld columns stay owed beside them.

### 51.10 Tests and seeded faults (2026-10-26)

**Tests** [M]: `tests/script.rs` **3 of 3**, new (`a_press_holds_its_buttons_from_its_frame_for_six_frames_or_its_count`,
`presses_that_overlap_are_held_together`, `a_stick_holds_its_value_and_the_later_begun_line_holds_an_axis`). The runner,
the bench and the test build without a warning and clippy finds nothing in them. No unit changed: the blast radius is the two examples, the shared script and its
test. **Seeded faults**, each made in place, the test run, the source restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | overlapping presses: the first line alone held | `presses_that_overlap_are_held_together` |
| F2 | a press held one frame past its count | `a_press_holds_its_buttons_from_its_frame_for_six_frames_or_its_count`, `presses_that_overlap_are_held_together` |
| F3 | the earlier begun stick line holding an axis | `a_stick_holds_its_value_and_the_later_begun_line_holds_an_axis` |

The bench's `game` workload was checked against the runner (the same picture at frame 6,300 from the same state,
§51.3), which is how the first series' empty port 2 was found.

### 51.11 Exposure, and what was observed

Running the player's BIOS and a game makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and never a
replacement writer.** The BIOS was handed to the machine by the link the step was given (market americas); it was never
opened, hashed, diffed, listed, disassembled or dumped, and no firmware folder's names were listed. Observed: pictures of
the game, the bench's times, digests and counts, the GS's and the GIF's queue lengths against their slots, the display
registers the runner prints, the EE's instructions counted by class (counts only, in a build apart), and the host's
instruction pointer and stacks in Sapphire's own code. No program counter, instruction or memory of the BIOS or the game
was printed, logged or read; the recompilers' perf map, which names blocks by guest address, was left off, the compiled
code's samples counted unnamed, and no log held the recompilers' refusal lines, which print a block's address. The disc
was read in place from the probe's folder. No emulator's source was read.

### 51.12 Named limits of the measurement

- **No play.** The windows are the opening's: the same engine and units as play, without the player's camera or input,
  possibly lighter or heavier than the village in play. D-900 stands between them and play.
- **One sampled run a window**, at 4 ms on every thread, which slows the run; shares only are used (§43.7).
- **The VU split** is inferred: the sampler cannot tell VU0's interpreted pairs from VU1's, or the float calls of VU1's
  code from VU0's, without stacks; #3, #4 and #9 split the VU micro share by the pairs' counts.
- **The interpreter-tier timings** (§51.6, 7) are single rounds beside the counting build, not a three-round series.
- **Load**: other stages' work ran beside every round, at loads up to 12; the arena's spread is its long frames'.

### 51.13 Where the plan was wrong, or left open

- **§5.12's VU1 row** (1 to 3 million pairs a frame) holds for the 3D (1.2 to 1.6 million); its cost does not: compiled at
  61 to 64 ns a pair with its floats called (§45.7), 75 to 100 ms a frame, against §5.12's 1.5 to 12 ms.
- **VU0** was priced at 0 to 15 ms; the 3D runs 0.1 to 0.4 million VU0 pairs a frame, interpreted.
- **§5.13's C3** was expected to matter at VU1's recompiler; it matters at the EE's FPU first.
- **The state's fixed slots** (§38.2: "no program of the project's comes near any bound") are met by a game: the GIF's at
  one cut, the GS's held writes through D-900.
- **P12, P17 and P19**: on these numbers the desktop does not meet §5.12's target with the exact levers (P19 and P12's
  first half look lost), and the exact levers leave every 3D window 12 to 38 times short (P17); they are retired at S47
  by their own measurements, not here.

### 51.14 Questions this step leaves

1. **[decision] The next stage.** *Recommended*: one exact step of the cheap levers, #1 VU1's budget join, #2 the
   name map, #3 VU1's cache, #4 VU0's tier and #5 the CLUT reads (about 1.15 to 1.35 times on the 3D windows, under one
   and a half stages, each graded as S40 and S43 were); then **D-900**, so that play can be measured; then S37 to S39 as
   one push, since the EE's recompiler buys nothing on the 3D until all three are in.
2. **[decision] The EE's floats.** *Recommended*: measure an exact fast path (#7) before C3: the FPU's operations and
   operand classes where the host's arithmetic with chopped rounding is proven equal to `sapphire::float`'s, as §46 did
   for the GS, and C3 only for what the proof leaves.
3. **[routine] D-900.** *Recommended*: a dispute step from the state at 10,531: the referee's pictures on the same script,
   the GS's CSR and IMR and the count of SIGNALs at the first stalled frame, and the window on the EE's and VU1's
   interpreters.
4. **[routine] The state's slots.** *Recommended*: raise the GIF's and the GS's held-write slots, or write those queues
   at their length, so that a player's save and rewind are not refused in a game (`SPH2` version 6); after D-900 shows
   whether the GS's backlog is a defect.
5. **[routine] The handheld's columns** (§51.9). *Recommended*: with S43's and S32's, when handheld runs are allowed.

### 51.15 What was read for this step

`Sapphire_Plan.md` §5.12, §5.13, §7's S32 and S47 rows and phase G, §13's P12, P17 and P19; `Sapphire_Native.md`'s §38.2, §39
(S28's runner, anchors and presses), §43 (S32's bench, profile and ranking), §44 to §47 and §50; `Diamond_Speed.md` §1 to
§3; `Sapphire_Disputes.md`'s D-800 to D-860 for the form. The bench's, the runner's, `vu::jit`'s, `vu::ops`'s,
`gs::workers`' and `emusen-jit`'s sources in the crate. No article, no emulator source, and no firmware.

## 52. The VU decode without the name map (2026-10-27)

§51's profile put the VU operation lookup at about 4 % of Jak's 3D frame: `vu::ops::entry` found an operation's `TABLE`
entry through a `HashMap` keyed on its name, hashed on every interpreted VU pair and every COP2 instruction. The lookup is
now constant time and hashes nothing. The decoder's three tables (`SPECIAL1`, `SPECIAL2`, `LOWER`) are statics, so an
operation's address gives its slot among their 320; a list of `TABLE` indices, one per slot, is built once from the names
and read thereafter. `entry`, `of_word` and `of_lower` keep their signatures, and the decoder in `emusen-mips` is unchanged.

**Measured on 2026-10-27**: the bench's `jak-arena` window from the state at frame 9,000 with `--defaults`, at CPU weight 100,
two rounds each, alternating before and after:

| Build | Mean ms a frame, rounds | Median ms a frame, rounds | Wall s |
|---|---|---|---|
| before | 1,211.7, 1,234.7 | 803.5, 819.9 | 363.5, 370.4 |
| after | 1,014.2, 1,171.6 | 759.2, 770.2 | 304.3, 351.5 |

The medians fall by 5.5 % and 6.1 %, a little more than the 4 % share, the rest being the machine's drift; the means move
with the spread of the window's slow frames (§51.5). The picture, sound and state digests (`B77DC314C6AE837C`,
`BCA8F00F4743A955`, `4ED52755B09F207E`) are equal in all four rounds.

**Proof**: a unit test maps every decoder operation to the entry found by name. The tests `vu0` (12), `vu_micro` (14),
`micro` (25) and the VU1 recompiler's oracles in `vu_jit` (16, recompiled equal to the interpreter) pass. Three faults
were seeded and reverted: an entry running another's function (`add` running `sub`) fails four of `vu0`'s tests; one
operation given another's index (`sub` as `add`) fails the unit test; the micro-only operations left unmapped fail the
unit test.

## 55. The vector units' and the GS's pure overhead in the 3D (2026-10-27)

*Numbered §55 after S34's §53 and §54; disputes would start at D-1020, and the step raised none. Decided 2026-10-27 by the
tester: work is chosen by the overhead it consumes, largest first; this step takes the overhead §51.6 measured in Jak and
Daxter's 3D that buys nothing for the emulated machine: VU1's compiler joined at its budget, its microprogram cache
overrun, VU0 interpreted where its compiled tier was built and graded (§45), and the emulation thread waiting for the GS's
workers over the CLUT. §52 took the name map out of the VU's decode beforehand; its tree is this step's "before".*

### 55.1 In one paragraph

Four exact changes, each measured on its own: **VU1's compiler is retired at its code budget instead of joined**
(`emusen-jit`'s `over_budget` lets the thread go, which refuses what is queued and frees its module as it exits); **the
microprograms kept for a start are 32, dropped least recently run, and the budget is 128 MiB**, sized from the working set
the three windows measure (24 to 31 MiB of code, 35 to 42 VU1 microprograms, at most 11 for one start, against the eight
kept oldest-first before); **VU0's micro mode runs on the same recompiler by default**, a setting `vu0` beside `vu1`; and
**a CLUT slot is reused only after the workers that read it have drawn**, one of 256 slots rather than 16, which is what
the dock's 45 ms a frame were (§55.4: not the CLUT's load, which never waited there). The windows' pictures, sound and
states are those of the tree before, on both EE tiers wherever measured, and VU0's code ran under S40's verifier over the
dock's 300 frames without a difference. **At CPU weight 100, two rounds each: the dock from 314.1 to 254.4 ms a frame
(3.18 to 3.93 frames a second, 1.23 times), Misty Island from 545.2 to 470.6 (1.83 to 2.13, 1.16 times), the arena from
990.0 to 818.9 (1.01 to 1.22, 1.21 times)**; the arena's and the island's frames of 12 to 18 s are gone (the longest
now 1.2 s), and the menus are unchanged. The sampled profile after (§55.6) puts the EE first (its software floats 8 to
10 % and the recompiler's dispatch 5 to 9 %), then VU micro mode's calls, which are now the calls into code rather than
interpretation, then the GIF and the VIF.

### 55.2 What was built

| Where | What |
| --- | --- |
| `emusen-jit/src/lib.rs` | `Compiler::retire`: the thread's stop set and its handle let go, never joined; a stopped thread refuses (publishes `FAILED` for) every job still queued, checked between the blocks of a batch too, so that a drop or a retirement compiles nothing more than the block in hand; `Jit::over_budget` retires the compiler. `EmuSen_Jit.md` §3.4 |
| `vu/jit/mod.rs` | the variants a start keeps, looked up most recent first and moved to the back when found, the least recently run dropped; `Settings::variants` (32) and `Settings::budget_mib` (128), `SAPPHIRE_VU_VARIANTS` and `SAPPHIRE_VU_BUDGET_MIB` for measurement; the budget checked where a program is handed to the compiler rather than on every run; `Counts::kept` and `widest`, the programs held and the most for a start |
| `gs/workers.rs` | 256 CLUT slots; the last job each worker reads a slot in, so that a slot's reuse waits for those workers only, up to those jobs |
| `v1.rs` | the setting `vu0` (*Performance*, `effect: exact`, the recompiler by default) and `vu0_recompiler_chosen`, read as `vu1` is |
| `examples/bench.rs` | `--vu0`, in `--defaults`; either unit left interpreted under `SAPPHIRE_VU0=interpreter` or `SAPPHIRE_VU1=interpreter`; the tier's settings from the environment; each unit's recompiler counts with the code and programs held |
| WiseMan `SapphireTests` | the settings' keys with `vu0` |

### 55.3 VU1's compiler at its budget, and the cache sized for the 3D

**What the join cost.** The budget check at the top of `vu::jit::run` dropped the module when its code passed 64 MiB, and
the drop joined the compiler's thread, which first compiled every microprogram still queued: on the arena, 4 to 18 frames
of 2 to 50 s a round (§51.5). Nothing needed the join: the dispatcher forgets every address the module published before
it runs again, so the thread can finish its block and free the module by itself. Retired that way, with the old cache
(eight a start, 64 MiB) kept to measure the change alone, the arena ran at 840.9 ms a frame with its longest frame 1.19 s
(before: 990.0, frames of 16.5 to 18.1 s) and the island at 486.4 with 1.01 s (545.2, 12.5 s); single rounds.

**Why the budget was reached at all.** The cache kept eight programs a start and dropped the oldest compiled, and Jak and
Daxter uploads more than eight to some starts in turn: with eight, a cycle of nine finds none of them, every upload
compiles again, and 2.1 to 3.1 programs a frame of 0.4 to 0.8 MiB each filled 64 MiB every 50 frames. **The working set**,
measured with 64 kept a start and a budget of 4 GiB, after each window's 60 frames of warming:

| Window | VU1 programs held | most for a start | code | compiled in the 300 frames | VU0 programs, code |
| --- | --- | --- | --- | --- | --- |
| `jak-dock` | 40 | 11 | 26.8 MiB | 0.02 a frame | 22, 0.6 MiB |
| `jak-island` | 35 | 9 | 24.1 MiB | 0.03 a frame | 21, 0.5 MiB |
| `jak-arena` | 42 | 11 | 30.8 MiB | 0.00 a frame | 40, 0.9 MiB |

So **32 kept a start, the least recently run dropped**, is three times the widest start measured, and **128 MiB** holds
four such windows' programs before a flush, which now costs a recompilation and no wait. Looking up most recent first
keeps the cost of a changed memory where it was: the running program is found first. With these, VU1 interprets 0.8 to
8.2 thousand pairs a frame on the island and the arena, against 180 and 309 thousand with the old cache, and compiles
nothing in the windows; the arena's mean moves from 840.9 to 818.9 ms (with VU0's tier, below, which the old-cache run
also had), the island's from 486.4 to 470.6.

**The check itself.** The sampled profile after (§55.7) found the budget's check at the top of every `run` at 0.8 to 1.2 %
of the emulation thread (an `Option`, an `Arc` and an atomic load, nine times a pair): the bytes of code change only when
a program is compiled, so the check moved to the one place a program is handed to the compiler, before it is, which keeps
the module at most the programs still in flight past the budget. One round on the build with the check moved, after the series: the dock 250.6, the island 468.3 and the arena
803.8 ms a frame (the series' after: 254.4, 470.6 and 818.9), the digests equal.

### 55.4 The CLUT's wait was its slots, not its load

§51.6 put about 48 ms of the dock's frame in "publishing jobs and waiting for CLUT reads", and the plan for this step
read that as the CLUT's load (TEX0 with CLD) waiting for the workers. **Measured on the dock, the load waited 2 times in
360 frames for 0.1 ms; the 45 ms a frame (2,885 waits, 16.4 s) were `Cause::Clut`**, the GS's thread waiting to reuse one
of the sixteen CLUT slots: the dock changes its CLUT 105 times a frame, so a slot came round again every sixteen
changes, and its reuse waited for every worker to pass the last primitive that had read it. The load itself already
waits only for the workers whose jobs change the lanes it reads (§50.3.5), exactly as asked; on the island and the arena
it costs 1.1 to 1.5 ms a frame, the real order of a CLUT drawn or uploaded just before, and is left as it is.

**The change**: 256 slots (a slot is 1 KiB and 8 bytes), and for each slot the last job each worker reads it in: every
worker for a primitive shared by rows, the one worker for a primitive drawn alone. A slot's reuse waits for those
workers, up to those jobs, and for no other. That is exact by §50.3's argument: a worker reads a slot only while running
a job that names it and gives it rows, and its count stored with Release after the job orders that read before the
GS's thread's Acquire and the copy over the slot. On the dock the CLUT's waits went from 45.5 ms a frame to none; the
ring's backpressure now appears instead (5,480 waits, 2.7 ms a frame), the workers being busy 85 to 94 ms a frame. On the
arena, 2.9 ms a frame of slot waits went too.

### 55.5 VU0's micro mode on the recompiler

S40 built the tier for both units and graded VU0 by the same oracles (§45.5: 80 random programs, every operation, the
mechanisms); §45.6 left it off because the frames S40 measured ran almost none of VU0's micro mode. The 3D runs 0.12 to
0.36 million VU0 pairs a frame (§51.5). Switching it on needed nothing more than the setting: VU0's memory operations
reach VU1 through the window by the helpers S40 wrote, and the stall memo stays VU1's (§45.3). With it on, every VU0
pair of the three windows but 1 to 98 a frame runs compiled, from 22 to 40 programs and under 1 MiB of code. **Its
share, measured by the bench left with `SAPPHIRE_VU0=interpreter` on the same build (single rounds)**: the dock 265.6 to
254.4 ms (4.2 %), the island 478.1 to 470.6 (1.6 %), the arena 842.7 to 818.9 (2.8 %). The setting is `vu0`, the
recompiler by default, as `vu1` is (decided 2026-10-26 by the tester for VU1, §45.6; this step applies the same rule to
the unit the same oracles graded).

### 55.6 Speed (measured 2026-10-27)

**The method**: the bench's three windows with `--defaults`, from the states at frames 6,000, 7,500 and 9,000; **two
rounds** (two rather than three to save time, decided 2026-10-27), before and after alternating window by
window (before 1, after 1, before 2, after 2); every run alone at CPU weight 100, nothing else of the project running, the
one-minute load 0.9 to 3.7. **Before** is WiseMan `8b1b27d54` (S43, S33, S36, S37, S34, §52), VU1 recompiled and VU0
interpreted as its `--defaults` had it; **after** is this step, VU0 recompiled.

| Window | Before: means (median ms) | After: means (median ms) | Before → after, median of means | Frames a second | Longest frame |
| --- | --- | --- | --- | --- | --- |
| `jak-dock` | 322.4, 305.7 (306.5, 295.4) | 256.0, 252.8 (247.7, 245.5) | **314.1 → 254.4, 1.23×** | 3.18 → 3.93 | 1.4–2.2 s → 0.42 s |
| `jak-island` | 545.1, 545.3 (486.2, 484.7) | 470.6, 470.5 (461.9, 462.5) | **545.2 → 470.6, 1.16×** | 1.83 → 2.13 | 12.5 s → 0.97 s |
| `jak-arena` | 997.6, 982.4 (746.8, 750.2) | 821.9, 815.9 (711.5, 696.6) | **990.0 → 818.9, 1.21×** | 1.01 → 1.22 | 16.5–18.1 s → 1.16 s |

The rounds agree within 6 % before (the dock) and 1 % after. Against §51.5's series on the older tree (361.1, 564.1 and
1,337.4 ms) the windows are 1.42, 1.20 and 1.63 times faster with §52, S37 and S34 between.

**The menus and the loading**, two rounds a build on the recompiler tier, one on the interpreter tier, from power-on:

| Workload | Before | After |
| --- | --- | --- |
| the boot's language screen (frames 690–760) | 44.5, 44.4 ms | 44.9, 44.5 |
| the boot, frames 0–760 | 68.6, 68.6 | 68.4, 68.3 |
| Jak's start and loading by the script (frames 8–400) | 85.3, 85.4 | 84.8, 84.5 |
| Jak, frames 0–900 | 60.3, 59.9 | 59.9, 59.7 |

Unchanged, as they must be: they run almost no VU micro mode and change their CLUT rarely.

**Each piece's share** (single rounds on the after build, each piece taken away alone; the frames' means):

| Piece | `jak-dock` | `jak-island` | `jak-arena` | Evidence |
| --- | --- | --- | --- | --- |
| 1. VU1's compiler retired, not joined | none (one long frame a round) | 545.2 → 486.4 (11 %) | 990.0 → 840.9 (15 %) | the old cache kept, the join gone; longest frames 1.0 and 1.2 s |
| 3. the cache sized (32 a start, LRU, 128 MiB) | under 1 % | 486.4 → 470.6 (3 %) | 840.9 → 818.9 (3 %) | VU1's interpreted pairs 180k → 0.8k, 309k → 8k a frame; flushes 2 and 6 → 0 |
| 4. VU0 recompiled | 265.6 → 254.4 (4 %) | 478.1 → 470.6 (2 %) | 842.7 → 818.9 (3 %) | VU0 interpreted on the same build |
| 5. the CLUT's slots | 45.5 ms a frame of waits → none (with the ring's 2.7 back), about 15 % | none to speak of | 2.9 ms a frame → none | the GS's wait counts |

### 55.7 Where the frame goes now (measured 2026-10-27)

**The method**: §51.6's: the sampler at 4 ms on every thread of one run of each window from its first progress line, the
emulation thread's samples classed by S32's rules (`units2.py` in the step's scratch), host functions only.

| The emulation thread | `jak-dock` (13,512 samples) | `jak-island` (26,610) | `jak-arena` (41,661) | §51.6 before |
| --- | --- | --- | --- | --- |
| **EE**, all of it | 39.3 % (100 ms) | 34.2 % (161 ms) | 36.3 % (297 ms) | 30 to 31 % |
| of which COP1's floats in software | **9.9 % (25 ms)** | **12.3 % (58 ms)** | **10.3 % (84 ms)** | 6 to 10 % |
| of which the recompiler's dispatcher and exit replay | 9.1 % | 6.1 % | 5.6 % | 5 to 7 % |
| of which the timing model | 5.1 % | 4.3 % | 6.3 % | |
| **VU micro mode** (the dispatcher in `vu::jit`, `Pipe`'s `run_to` and `finish`, the float operations compiled code calls) | **17.8 % (45 ms)** | **20.9 % (98 ms)** | **19.8 % (162 ms)** | 18 to 24 % |
| of which `vu::jit`'s dispatcher, and `Pipe` | 6.0 and 5.2 % | 7.5 and 7.2 % | 6.7 and 7.2 % | |
| **Compiled code** (EE blocks, VU0 and VU1 microprograms) | 7.1 % (18 ms) | 7.2 % (34 ms) | 5.9 % (48 ms) | 3.5 to 4.8 % |
| **GIF** | 9.2 % (23 ms) | 8.1 % (38 ms) | 8.2 % (67 ms) | 6.7 to 7.5 % |
| **VIF** | 7.1 % (18 ms) | 7.2 % (34 ms) | 7.3 % (60 ms) | 5.3 to 6.4 % |
| **GS** on the emulation thread | **3.7 % (9 ms)** | 7.3 % (34 ms) | 3.2 % (26 ms) | 18.7 / 7.0 / 2.9 % |
| **Idle skipping's own work** | 1.2 % | 2.5 % | 6.9 % (56 ms) | 0.9 / 2.2 / 5.5 % |
| Scheduler and synchronisation | 4.9 % | 4.2 % | 4.6 % | 4.2 to 4.4 % |
| EE timers and INTC, DMAC, IOP, PCRTC and the rest | 9.4 % | 8.4 % | 7.7 % | |
| In a system call (the workers' unparking at a job's publication; no join) | 1.4 % | 1.2 % | 0.6 % | 1.6 / 1 / **9.7 %** |

The shares are of a shorter frame, so a unit whose cost did not change takes a larger share: the EE's floats are 25, 58
and 84 ms a frame, as before (23, 59, 108 in §51.6 on the older tree). The VU's 45 to 162 ms are no longer interpretation:
both units' pairs run compiled (all but 0.8 to 8 thousand a frame), and what remains is the cost of reaching the code nine
times a pair (§45.3, §45.11): the dispatcher's checks and the frame it builds, `Pipe::run_to` and `finish` after every EE
instruction, and the float operations called. The GS on the dock fell from 68 to 9 ms. No sample of the emulation thread
waited for a compiler; the budget's check itself was 0.8 to 1.2 % and has moved since (§55.3).

**The new top five** (by the emulation thread's share, the three windows together): **1. the EE's floats in software** (9.9 to 12.3 %, §51.8 #7 and C3); **2. VU micro mode's call path**
(17.8 to 20.9 % with its float calls; 12 to 15 % the dispatcher and `Pipe`'s synchronisation alone, S42's lever, §51.8
#9); **3. the GIF** (8.1 to 9.2 %); **4. the VIF** (7.1 to 7.3 %), 3 and 4 together §51.8 #10; **5. the EE recompiler's
dispatcher and exit replay** (5.6 to 9.1 %, the blocks ending at every load, store, MMI and COP instruction S38 and S39
would compile, §51.7). On the arena idle skipping's own work (6.9 %) comes next.

### 55.8 Tests and seeded faults (2026-10-27)

**Tests** [M, Release]: `emusen-jit` **11 of 11** (new: `past_its_budget_the_compiler_is_retired_without_waiting_for_its_queue`);
`tests/vu_jit.rs` **19 of 19** (new: `an_upload_over_a_compiled_vu0_program_is_run_as_uploaded`,
`past_the_code_budget_every_program_is_compiled_anew`, `the_programs_kept_for_a_start_drop_the_least_recently_run`; S40's
oracles, VU1 and VU0 recompiled against the interpreter, unchanged and passing); `tests/threads.rs` **11 of 11** (new:
`a_clut_slot_is_reused_only_after_the_worker_drawing_with_it_is_done`, 300 sprites over three CLUTs with a worker held,
compared with one thread); `v1`'s `the_vu0_setting_chooses_its_tier_and_the_recompiler_is_the_default`; the blast
radius: Sapphire's crate **489 of 489**, MarsRT's **507 of 507** (it shares `emusen-jit`), WiseMan's `MarsRt` and `SapphireTests`
**269 of 269** on lavapipe. **The bench's gate**: the three windows' picture, sound and state digests equal in all four
rounds and in every single-round run of §55.6 (`43D839015A0E4DA1`, `D9F0AF56FD3DEF28`, `B77DC314C6AE837C` the pictures);
the boot's 760 frames (`50F06A02…`), Jak by the disc's fast boot to 900 (`7EEC08D4…`) and Jak by the BIOS and the script to 900 (`FFB0FCE9…`) equal before and after on both EE tiers; **S40's verifier** (oracle 1,
`SAPPHIRE_VU_VERIFY=1`) on the dock's 300 frames with both units recompiled: no difference, 247,430 VU0 and 364,653 VU1
pairs a frame compiled. ps2autotests: **52 of 55 and 23 of 23** with both units recompiled (`SAPPHIRE_VU0` and `SAPPHIRE_VU1` set for `s10`, whose machine is made directly), the same three failing as §36.5 (`cpu/ee_fpu/sqrt`, `dma/vif/intstall`, `vu/lower/efu`).

**Seeded faults**, each made in place, the tests run, the source restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | a CLUT load not waiting for the worker that writes its source (`Mem::read` under `Cause::ClutLoad` settles nothing) | `generated_streams_leave_the_memory_and_every_read_back_as_one_thread_does_at_every_worker_count`, `a_state_saved_with_primitives_in_flight_loads_and_continues_as_one_thread_does` |
| F2 | a VU0 program resumed without its words checked (a stale microprogram run) | `random_microprograms_on_vu0_leave_the_units_as_the_interpreter_does` |
| F3 | the programs kept after the budget dropped their code (compiled code run after its eviction), seeded at the check's first place and again at its second | `past_the_code_budget_every_program_is_compiled_anew`, both times |
| F4 | a CLUT slot reused without waiting for the worker reading it | `a_clut_slot_is_reused_only_after_the_worker_drawing_with_it_is_done` |
| F5 | the join put back at the budget | `past_its_budget_the_compiler_is_retired_without_waiting_for_its_queue` |
| F6 | the programs a start keeps dropped oldest compiled again, not least recently run | `the_programs_kept_for_a_start_drop_the_least_recently_run` |

F1's own test, `a_clut_load_from_a_page_being_drawn_waits_and_loads_what_was_drawn`, passed under the fault: its held
worker had finished by the load. The generated streams find it.

### 55.9 Exposure

Running the player's BIOS and Jak and Daxter makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and
never a replacement writer.** The BIOS was handed to the machine by the link the step was given (market americas); it was
never opened, hashed, diffed, listed, disassembled or dumped, and no firmware folder was listed. Observed: the bench's
times, digests and counts (microprograms compiled, held and kept a start, code bytes, the GS's waits by cause), and the
host's instruction pointers in Sapphire's own code; no program counter, microprogram word or memory of the BIOS or the
game was printed or logged, and the perf map stayed off. No emulator's source was read.

### 55.10 Named limits

- **Two rounds a window**, and the pieces' shares single rounds: a piece's share under 3 % is within one round's spread.
- **The working set is the opening's three windows**; play may hold more programs a start. The counts the bench prints
  (`held`, `at most … for a start`) show it on any later window.
- **The budget's flush is cheaper, not free**: past 128 MiB every program is compiled again, and VU1 interprets until
  its code returns.
- **The handheld's columns** remain owed with §51.9's.

### 55.11 Questions this step leaves

1. **[routine] VU0's default.** *Recommended*: the recompiler, as set here, on the same oracles as VU1's (§55.5, §55.8);
   the setting `vu0` keeps the interpreter one choice away.
2. **[decision] The next step by overhead.** *Recommended*: the EE's floats by an exact fast path (§51.8 #7, §51.14 Q2),
   the largest single share left (8 to 10 %); then the VU's call cost (S42, VU1 running ahead to its observation points),
   now that its pairs run compiled and the calls into them are what remains of "VU micro".
3. **[routine] The CLUT's load** costs 1.1 to 1.5 ms a frame on the island and the arena, the real order of a CLUT
   drawn just before; finer lanes (blocks rather than pages) would shorten it. *Recommended*: leave it until the GS on
   the device (S44) changes where the CLUT lives.

### 55.12 What was read for this step

This page's §51 and §52; `Sapphire_Native.md` §45 and §50; `EmuSen_Jit.md` §3 and §5. The code of `emusen-jit`,
`vu::jit`, `gs::workers`, `gs::texture`'s CLUT load, `v1` and the bench. No article, no emulator source, no firmware.

## 56. The PlayStation 2's floats by exact fast paths (2026-10-27)

*Numbered §56 after §55; disputes would start at D-1040, and the step raised none. Decided 2026-10-27 by the tester:
work is chosen by the overhead it consumes, largest first; §55.7 put "the EE's floats in software" first, at 9.9 to
12.3 % of the emulation thread. This step is §51.8's lever #7 (§51.14 Q2): the operations and operand classes where the
host's arithmetic gives `sapphire::float`'s bits are taken there, as §46 did for the GS, and nothing else. C3 (§51.8 #8),
the floats on the host with clamping, changes bytes and is not part of it.*

### 56.1 In one paragraph

A census of every call into `sapphire::float` on the three windows (§56.2) found that **the share §55.7 gave the EE was
the vector units'**: COP1 makes 0.5 to 1.2 % of the calls, VU0 and VU1 the rest, and the profile's rule put any sample in
`float.rs` without a VU frame in its inline chain on the EE, which a call from compiled microcode never has. Most of the
cost was one function, the Booth multiplier of D-26, a loop of thirteen rows run 1.3 to 3.6 million times a frame by the
VUs' multiplies and multiply-adds. **Three changes, each proven bit for bit (§56.3, §56.4)**: the product is the exact
integer product less a loss of 0, 1 or 2 read from three bits, which is what the Booth loop sums for every pair of
significands; DIV and SQRT take the host's single-precision division and root, correctly rounded to the nearest as the
console's are (D-27), wherever both operands have an exponent field of 1 to 254 and the quotient lands at a field of
2 to 254; the rest stays as it was. **No rounding mode is set anywhere**: where the console truncates (the adder, the
multiplier, RSQRT's quotient) the step keeps integers, and where it rounds to the nearest it uses the host's default
rounding, which is the same. The checks live in `float.rs`, so the EE's two tiers and both VUs' tiers take them alike.
**At CPU weight 100, two rounds each: the dock from 251.4 to 235.3 ms a frame (3.98 to 4.25 frames a second, 1.07
times), Misty Island from 470.1 to 423.9 (2.13 to 2.36, 1.11 times), the arena from 811.4 to 726.1 (1.23 to 1.38, 1.12
times)**; every picture, sound and state digest equal on the windows, the boot's 760 frames and Jak's 900 on both EE
tiers. The floats fall from 9.9 / 12.3 / 10.3 % of the emulation thread to 2.1 / 2.8 / 2.1 %.

### 56.2 The census: who calls, which operations, which classes (measured 2026-10-27)

A counting build (a census module and one line in the bench, kept in the step's scratch, not committed) counted each
call by caller (COP1, or a VU: VU0's macro and micro modes and VU1), operation and the class of each operand: **Z** zero,
**D** a would-be denormal (field 0, fraction not 0), **N** a field of 1 to 254, **M** a field of 255; and of the result:
an exponent overflow or underflow. "RZ differs" counts where IEEE 754's rounding towards zero on the exact result would
give other bits than the console's rule (§56.3). The windows' 360 frames each (60 warming, 300 timed); calls a frame:

| Caller, operation | `jak-dock` | `jak-island` | `jak-arena` | N op N | a zero | a would-be denormal | field 255 | overflow / underflow | RZ differs |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| VU add and subtract | 284 k | 408 k | 864 k | 73–89 % | 7–13 % | 3.5–14.4 % | under 0.07 % | 0.02 % / none | **10.8–12.5 %** |
| VU multiply | 608 k | 1,176 k | 1,810 k | 77–88 % | 12–23 % | 0.4–0.6 % | under 0.23 % | under 0.08 % / 0.02 % | **3.3–6.2 %** |
| VU multiply-add (the product summed) | 650 k | 1,293 k | 1,804 k | 58–82 % | 17–40 % | 0.5–2.4 % | under 0.02 % | 0.03 % / 0.0001 % | — |
| VU DIV | 15 k | 67 k | 67 k | 99.1–99.9 % | 0.1–0.9 % | 0.03 % | 0.02 % | — | — |
| VU RSQRT | 10 k | 11 k | 32 k | 98.9–99.7 % | 0.3–1.0 % | — | 0.06–0.14 % | — | — |
| VU SQRT | 0.7 k | 0.1 k | 0.5 k | 99 % | 0.4–1.5 % | — | — | — | — |
| COP1 ADD.S, SUB.S, ADDA.S, SUBA.S | 3.7 k | 2.6 k | 9.4 k | 88–94 % | 6–12 % | 0.01 % | — | — | **10.7–15.8 %** |
| COP1 MUL.S, MULA.S | 4.6 k | 2.7 k | 9.7 k | 84–89 % | 11–16 % | — | — | none / 0.005 % | **0.6–2.9 %** |
| COP1 DIV.S | 2.8 k | 1.9 k | 8.6 k | 96–99.5 % | 0.5–3.8 % | — | — | — | — |
| COP1 the multiply-adds, SQRT.S | 0.3 k | 0.3 k | 0.4 k | 78–99 % | 1–23 % | — | — | — | — |

So **the multiplier carries the share**: 1.26, 2.47 and 3.61 million Booth sums a frame from the VUs, against 5 to
10 thousand from COP1. Would-be denormals and field 255 occur, in the VUs' data above all (a 14 % share of the dock's
VU additions has a would-be denormal operand), so no fast path may assume them away. RZ differs in 3 to 16 % of the
additions and products, far more than random significands would give (the product's case is about 2 in 2^23 for them):
the games' numbers have short significands, 1.0, 0.5 and the like, whose exact products end in zeros that the Booth
loss reaches.

### 56.3 Each operation and class: what the host's arithmetic gives

[A; R the *EE Core User's Manual* §8.1.1, §8.7, §8.8 and Table 8-4 as §19 read them; IEEE 754-2019 §4.3.1 (rounding
to the nearest, ties to even) and §5.4.1 (the arithmetic operations, each correctly rounded: the exact result rounded
once in the destination's format)]. "The host" is the x86-64 SSE unit as Rust leaves it: rounding to the nearest, no
flush to zero, no denormals-are-zero; nothing in the project changes MXCSR (searched 2026-10-27).

| Operation | Classes taken on the fast path | Proven equal because | Left to the integers, and why |
| --- | --- | --- | --- |
| **MUL** (MUL.S, MULA.S; the VUs' MUL, MULA, OPMULA and the products of every multiply-add, OPMSUB) | every pair: the product is integer arithmetic (§56.4) | the closed form is the Booth sum, for every pair of 24-bit significands (§56.4); the exponent, normalisation and `finish` are unchanged | nothing. The host's multiply rounded towards zero would give other bits in 0.6 to 6.2 % of the windows' products (§56.2), where the Booth loss crosses a 24-bit boundary, so it is not used |
| **DIV** (DIV.S; the VUs' DIV) | both operands N, the host's quotient at a field of 2 to 254 | the console rounds the quotient to the nearest (D-27) and §19.3 shows no tie can occur; IEEE 754's division rounds the exact quotient to the nearest of the same 24-bit format, so the bits agree wherever both formats read the operands alike (N) and the result is a normal number in both | a zero, a would-be denormal or field 255 operand (the console's rules for them are not IEEE 754's); a quotient at field 1 (the host may round up into it from below 2^-126 at denormal precision where the console flushes to zero), 0 or 255 (the host's denormal and infinity; the console's underflow to ±0 and exponent 255). 0.1 to 3.8 % of the windows' quotients |
| **SQRT** (SQRT.S; the VUs' SQRT; RSQRT's root) | the operand N | the console rounds the root to the nearest (D-27), no tie possible (§19.3); a root of a normal number is normal; the sign is dropped as the console drops it (D-29) | zero (the root of either zero is +0), a would-be denormal, field 255 |
| **RSQRT** | its root, as SQRT | as SQRT | the quotient: the console truncates it (D-28's reading), which the host can do only with the rounding mode changed or in a wider format; 10 to 32 thousand a frame are not worth it |
| **ADD, SUB** (and the sum of every multiply-add) | none | — | the console aligns the smaller operand to the larger's 24 bits and one more and drops the rest (D-25). For equal signs that is IEEE 754's rounding towards zero of the exact sum; for opposite signs it is the larger less the smaller rounded *down*, which truncates to one unit more than the exact difference truncated wherever bits were dropped: 10.7 to 15.8 % of the windows' additions. A host form must take the exponents' difference and mask the dropped bits, which is the integer path's own work; after this step the adder is 0.4 to 0.6 % of the thread |
| MAX, MIN, the compares, ABS, NEG, MOV, the conversions | — | — | already a few integer operations on the bits |

**The rounding mode.** A fast path could set the host's rounding towards zero in one place, or avoid that by an exact
formulation; this one avoids it. Every operation the console rounds to the nearest goes to the host as it
stands, and every one it truncates stays on integers. Changing MXCSR would touch state the compiler assumes fixed (Rust
and LLVM fold and reorder floating-point operations on the assumption of rounding to the nearest) and that the GS's
code, Cranelift's compiled code and the bench's own timing share on the emulation thread; and truncation alone would
still not give the console's adder or multiplier.

**Per caller.** `add`, `mul`, `div`, `sqrt`, `rsqrt` and `fused` return the same `Outcome`, `Quotient` or `Fused`, bits
and flags, for every input (§56.5). Every flag each caller raises is a function of those: **the EE's FPU** its O, U and
SU (with SO and SU sticky) from the outcome and the multiply-add's product, I and D (SI, SD) from the quotient
(§19.2); **the VUs' upper instructions** their MAC flags (O, U, S, Z) and the status register's sticky flags from the
outcome and, for the multiply-adds, the product's own overflow and underflow (§21.3); **the VUs' lower instructions**
DIV's, SQRT's and RSQRT's I and D status flags from the quotient. CLIP's judgement (`order`) and the clamping of
conversions are not touched. So the proof of each function covers every caller, on both tiers of each unit.

**Left for the VUs.** Their floats are now 2.1 to 2.8 % of the thread with COP1's; what remains of "VU micro" is the
calls themselves, one function pointer a lane through `arith` and `lanes`, and the dispatcher and `Pipe` (§56.7).

### 56.4 The Booth sum in closed form

`booth(x, y)` (D-26) recodes `y`, shifted left one bit, as thirteen radix-4 digits from bits 2i + 2, 2i + 1, 2i; a
digit selects 0, x or 2x, negative ones as the ones' complement with a carry-in of 1, and the row is added at column 2i
with every column below `DROPPED_COLUMNS` (1) cleared. **Argument** [A]:

1. Without the clearing, the rows sum to x × y exactly: radix-4 Booth recoding is exact for any `y` below 2^24 with
   thirteen digits (the top digit reads `y`'s bit 23 and two zeros, so it is 0 or +1), and a negative row's
   complement plus its carry is the negated multiple modulo 2^64.
2. Clearing column 0 changes only the row at i = 0: a row at i ≥ 2 is shifted left by i and has no bit in column 0, and
   its carry 1 << i lies above column 0 and is kept. The row at i = 0 loses its bit 0, and its carry, which would land
   in column 0, is dropped.
3. That row's digit reads `y`'s bits 1 and 0 and a zero: y & 3 = 0 gives 0 (nothing lost); 1 gives +x (loses x & 1);
   2 gives −2x, whose complement is odd (loses 1 and the carry, 2); 3 gives −x, whose complement's bit 0 is 1 − (x & 1)
   (loses that and the carry, 2 − (x & 1)).
4. The sum is therefore x × y − loss, loss in {0, 1, 2}, and since x × y ≥ 2^46 the 48-bit mask the loop applies is
   never reached.

`product(x, y)` computes it as `x * y - (0x1222_1000 >> (((y & 3) << 1 | x & 1) << 2) & 3)`, the eight losses as
nibbles; a constant assertion ties it to `DROPPED_COLUMNS == 1`, so that a different reading of D-26 fails to build
rather than computing the wrong product. `booth` stays in the tests as the reference.

### 56.5 The equality evidence (2026-10-27)

`float`'s tests, in release; the classes drawn are zero, would-be denormals, field 255, fields 1 to 4 and 251 to 254,
fields within 32 of 127 and anything else, with a zero, all-ones or random fraction, either sign.

| Test | What it compares | Cases |
| --- | --- | --- |
| `the_closed_form_product_is_the_booth_sum_for_every_multiplier` | `product` against `booth`: six multiplicands (the smallest and largest, odd and even, two random) with **every** 24-bit multiplier, then random pairs | 50,331,648 and 2,000,000 |
| `a_billion_products_equal_the_booth_sum` (ignored) | the same, random | **1,000,000,000** |
| `products_and_multiply_adds_keep_their_outcomes_and_flags_over_every_class` | `mul` and `fused` (adding and subtracting) against their bodies on `booth`, the whole outcome with its overflow and underflow, over every class; asserts more than 10,000 overflows and underflows met | 4,000,000 each |
| `the_host_quotient_equals_the_integer_one_wherever_it_is_taken` | `div` against `div_soft`, the whole quotient with I and D, over every class (asserting more than a million taken by the host); and dividends at fields 1, 2, 3, 127, 128, 253, 254 and 255 by divisors at fields 1, 126, 127, 128 and 253, each side of a result field's edge | 4,000,000; 40,000,000 |
| `a_billion_quotients_equal_the_integer_ones` (ignored) | the same, random | **1,000,000,000** |
| `the_host_root_equals_the_integer_one_for_every_operand_of_two_binades_and_every_class` | `sqrt` against `sqrt_soft` for every fraction at fields 126 and 127, either sign (a root's bits at any field are those of the binade of the same parity, scaled), then random over every class | 33,554,432 and 4,000,000 |
| `every_root_equals_the_integer_one` (ignored) | every 32-bit pattern | **4,294,967,296**, all |

The ignored tests ran once, in 53 s together. S9's lib tests (`the_booth_sum_is_the_exact_product_or_just_below_it`,
the ties) pass unchanged.

### 56.6 Speed (measured 2026-10-27)

**The method**: §55.6's: the bench's three windows with `--defaults` from the states at 6,000, 7,500 and 9,000; two
rounds, before and after alternating window by window; every run alone at CPU weight 100, the one-minute load 0.9 to
4.3. **Before** is WiseMan `87265dcf4` (§55 merged; its bench built from `c0675c294`), **after** this step.

| Window | Before: means (median ms) | After: means (median ms) | Before → after, median of means | Frames a second | Longest frame |
| --- | --- | --- | --- | --- | --- |
| `jak-dock` | 251.1, 251.7 (244.1, 246.8) | 234.0, 236.6 (225.1, 228.7) | **251.4 → 235.3, 1.07×** | 3.98 → 4.25 | 0.41 → 0.37 s |
| `jak-island` | 469.1, 471.1 (459.8, 458.5) | 417.8, 429.9 (413.7, 415.9) | **470.1 → 423.9, 1.11×** | 2.13 → 2.36 | 0.97 → 0.88 s |
| `jak-arena` | 808.1, 814.6 (701.2, 698.5) | 724.0, 728.1 (654.8, 652.4) | **811.4 → 726.1, 1.12×** | 1.23 → 1.38 | 1.17 → 0.99 s |

The rounds agree within 3 %. §51.8 expected 1.05 / 1.08 / 1.06 from three quarters of the classes; the gain is larger
because the VUs' calls, which §51.6 counted as VU micro rather than as floats, go through the same functions.

**The menus and the loading**, from power-on (the means of the frames named, ms; two rounds on the recompiler tier, one on
the interpreter tier):

| Workload | Before | After |
| --- | --- | --- |
| the boot's language screen (frames 690–760) | 44.8, 44.9 | 44.6, 49.0 |
| the boot, frames 0–760 | 68.1, 69.2; interpreter 91.4 | 68.7, 70.5; interpreter 92.4 |
| Jak's start and loading by the script (frames 8–400) | 85.3, 85.4; interpreter 132.6 | 85.4, 86.3; interpreter 131.1 |
| Jak, frames 0–900 | 60.2, 60.2; interpreter 95.9 | 60.5, 60.7; interpreter 94.8 |

Unchanged within the rounds' spread (the 49.0 is one round's), as expected: the menus run few floats.

### 56.7 Where the frame goes now (measured 2026-10-27)

**The method**: §55.7's sampler at 4 ms on every thread of one run of each window, the emulation thread's samples classed
by §55.7's rules with one change: a sample in `float.rs` is counted as `sapphire::float` with every caller, since the
census (§56.2) shows that the rule that put it on the EE was wrong for 99 % of the calls.

| The emulation thread | `jak-dock` (12,525 samples) | `jak-island` (23,659) | `jak-arena` (37,648) | §55.7 |
| --- | --- | --- | --- | --- |
| **`sapphire::float`**, every caller | **2.1 %** | **2.8 %** | **2.1 %** | 9.9 / 12.3 / 10.3 % (as "the EE's floats") |
| of which the adder, the multiplier and the multiply-add (with `finish`) | 0.7, 0.6, 0.4 % | 1.0, 0.7, 0.6 % | | |
| **EE**, all of it | 32.3 % | 24.7 % | 28.3 % | 34 to 39 % with the floats |
| of which the recompiler's dispatcher and exit replay | 10.2 % | 7.3 % | 6.1 % | 5.6 to 9.1 % |
| of which the timing model | 5.8 % | 4.8 % | 6.6 % | 4.3 to 6.3 % |
| **VU micro mode** (`vu::jit`, `Pipe`, `vu::ops`) | 18.4 % | 22.7 % | 21.0 % | 17.8 to 20.9 % with its float calls |
| of which `vu::jit` and `Pipe` | 5.6 and 5.4 % | 7.0 and 7.8 % | 6.4 and 8.0 % | |
| **GIF** | 9.1 % | 8.8 % | 9.2 % | 8.1 to 9.2 % |
| **VIF** | 8.3 % | 7.7 % | 8.3 % | 7.1 to 7.3 % |
| **Compiled code** | 7.4 % | 8.0 % | 6.4 % | 5.9 to 7.2 % |
| **Idle skipping's own work** | 1.2 % | 2.9 % | 7.5 % | 1.2 / 2.5 / 6.9 % |
| Scheduler and synchronisation | 5.4 % | 4.3 % | 4.9 % | 4.2 to 4.9 % |
| GS on the emulation thread | 5.0 % | 8.7 % | 3.5 % | 3.2 to 7.3 % |

**The new top five** (by the emulation thread's share, the three windows together): **1. VU micro mode's call path**,
18.4 to 22.7 % (the dispatcher and `Pipe`'s synchronisation 11.0 to 14.8 % of it; S42's lever, §51.8 #9); **2. the EE
recompiler's dispatcher and exit replay**, 6.1 to 10.2 % (S37 to S39, §51.8 #6); **3. the GIF**, 8.8 to 9.2 %; **4. the
VIF**, 7.7 to 8.3 % (3 and 4 together §51.8 #10); **5. the EE's timing model**, 4.8 to 6.6 % (C1's share, refused at
§44.9). On the arena idle skipping's own work (7.5 %) comes before the fifth.

### 56.8 Tests and seeded faults (2026-10-27)

**Tests** [M, release]: `float`'s 6 of 6 and the 3 ignored (§56.5); the blast radius: Sapphire's crate **493 of 493** (4 new), WiseMan's
`SapphireTests` **5 of 5** on lavapipe. **The bench's gate**: the three windows' picture, sound and state digests
(`43D839015A0E4DA1`, `2C225C73DB4D3F9D`, `E220B19BD26D8EBC`; `D9F0AF56FD3DEF28`, `4F27F29527E1B781`, `F074C69B5F66E65E`;
`B77DC314C6AE837C`, `BCA8F00F4743A955`, `4ED52755B09F207E`) equal in all four rounds of each build and in the census
build; the boot's 760 frames (`50F06A02F1B76860`, `2EE7B3D10DF3C401`, `2051C6BB42228C53`) and Jak by the BIOS and the
script to 900 (`FFB0FCE98BD7C164`, `7CF6B8FA4F6C2701`, `0DCD2B18F0881625`) equal before and after on both EE tiers.
ps2autotests: **52 of 55 and 23 of 23** with both units recompiled, the same three failing as §36.5
(`cpu/ee_fpu/sqrt`, the RSQRT.S line D-28 names; `dma/vif/intstall`; `vu/lower/efu`).

**Seeded faults**, each made in `float.rs`, the lib tests and `tests/fpu.rs`, `vu0.rs` and `vu_micro.rs` run, the
source restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | a would-be denormal operand taken on the host (the class check reading field 0 as a number) | `the_host_quotient_equals_the_integer_one_wherever_it_is_taken`, `the_host_root_equals_…`; the EE's `square_root_and_reciprocal_square_root_equal_the_manual`, `multiply_and_divide_over_every_pair_of_input_classes_equal_the_manual`; VU0's `divide_square_root_and_reciprocal_root_equal_the_manual` |
| F2 | an overflowing quotient taken on the host (field 255, the host's infinity, accepted) | `the_host_quotient_equals_…`; the EE's `multiply_and_divide_…`; VU0's `divide_square_root_and_reciprocal_root_…` |
| F3 | the rounding left at the nearest: the product of two N operands taken from the host's multiply as it rounds | `products_and_multiply_adds_keep_their_outcomes_and_flags_over_every_class`; the EE's `the_format_reproduces_the_console_lines_that_decide_its_readings`, `multiply_and_divide_…`, `the_accumulator_instructions_equal_the_manual`; VU0's `multiply_over_every_pair_of_input_classes_equals_the_manual`, `the_multiply_adds_and_the_outer_product_equal_the_manual`; `vu::efu`'s `the_series_land_where_the_code_puts_them` |
| F4 | the EE's flags not set on the fast path: the product's underflow dropped (U, SU) | `products_and_multiply_adds_…`; the EE's `multiply_and_divide_…`, `the_accumulator_instructions_…`; VU0's `multiply_over_every_pair_…`, `the_multiply_adds_…` |
| F5 | a VU flag not set on the fast path: the product's overflow dropped (the MAC's O and the status's sticky O) | `products_and_multiply_adds_…`; VU0's `multiply_over_every_pair_of_input_classes_equals_the_manual`, `the_multiply_adds_and_the_outer_product_equal_the_manual`; the EE's `the_format_reproduces_…`, `multiply_and_divide_…`, `the_accumulator_instructions_…` |

The EE's and VU0's tests that caught them are S9's and S12's programs graded against the manuals transcribed apart from
`float.rs` (§19.4, §21.4); `vu_micro.rs` caught none, its programs comparing the tiers with each other.

### 56.9 Exposure

Running the player's BIOS and Jak and Daxter makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and
never a replacement writer.** The BIOS was handed to the machine by the link the step was given (market americas); it was
never opened, hashed, diffed, listed, disassembled or dumped, and no firmware folder was listed. Observed: the bench's
times, digests and counts, the census's counts of calls by operation and operand class, and the host's instruction
pointers in Sapphire's own code; no program counter, operand value, microprogram word or memory of the BIOS or the game
was printed or logged. No emulator's source was read.

### 56.10 Named limits

- **Two rounds a window.** The gains (7 to 12 %) are well outside the rounds' spread (3 %).
- **The census is the opening's three windows.** Other scenes may divide or take roots more; the fast paths' share of
  the work moves with them, their results do not.
- **The host's default floating-point environment is assumed**: rounding to the nearest, no flush to zero. Rust
  guarantees it on every target the project builds for; code that changed MXCSR on the emulation thread would break
  DIV's and SQRT's fast paths, and §56.3 records that nothing does.
- **RSQRT's quotient and the adder stay on integers** (§56.3), as does every operation on a zero, a would-be denormal
  or field 255.
- **The handheld's columns** remain owed with §51.9's.

### 56.11 Questions this step leaves

1. **[routine] The profile's attribution.** *Recommended*: keep §56.7's rule (a sample in `float.rs` is the floats',
   every caller), and split it by the census when a step needs the callers; the old rule's "EE floats" was the VUs'.
2. **[decision] The next step by overhead.** *Recommended*: the VU's call path (S42, §51.8 #9), 18 to 23 % of the
   thread and now the largest single share; the EE recompiler's remaining stages (#6) after it.
3. **[routine] C3** (§51.8 #8). *Recommended*: retire it from the ranking: the floats it would replace are 2.1 to
   2.8 % of the thread, so its bound is now about 1.03 against a byte-changing cost.

### 56.12 What was read for this step

This page's §51 and §55; `Sapphire_Native.md` §19 (S9, the FPU and `sapphire::float`) and §46 (the GS's exact fast
paths); `float.rs` and `ee/fpu.rs`, `vu/ops.rs`'s float calls. IEEE 754-2019 §4.3.1 and §5.4.1 for correct rounding,
and the *EE Core User's Manual* §8 as §19 cites it. No emulator source, no firmware.

## 57. The VU's call path: what threads would cost, and the calls answered at the unit's due cycle (2026-10-27)

*Numbered §57 after §56; disputes would start at D-1060, and the step raised none. Decided 2026-10-27 by the tester:
work is chosen by the overhead it consumes, and S42 (the plan's §5.12 lever 3, §51.8 #9) follows the EE's floats. §56.7
put VU micro mode's call path first, at 18.4 to 22.7 % of the emulation thread. The step was to profile the path, then
put VU1 (and the IOP, if its share warranted it) on a thread of its own, exact by construction. The profile and a count
of the observations (§57.2, §57.4) show that an exact thread cannot pay for itself on this board, so VU1 stays on the
emulation thread and the question goes to the tester (§57.11 Q1); what was built is the exact part of the lever that the
profile asked for, the calls that cannot run a pair answered without running the unit (§57.3).*

### 57.1 In one paragraph

The board brings both VUs to the EE's time after every instruction it does not pass (§45.3), and the "synchronisation"
§55.7 named is those calls, not locks: `Pipe` has none. **The profile split** (§57.2): of the 18.5 / 22.8 / 21.0 % the
call path takes, **7.6 / 10.6 / 11.2 % is the catch-up itself** (calls, the landing of flags and results, the checks
that find nothing to run), 2.6 / 3.4 / 3.1 % finding and entering compiled code, and 7.7 / 8.5 / 6.7 % the work (the
float operations compiled code calls and their helpers; the compiled code's own share is not separable from the EE's
blocks). **A count of the calls** (§57.2) found that 81 to 85 % of VU1's calls and 80 to 95 % of VU0's run nothing and
land nothing. **A thread does not pay** (§57.4): every board step at which VU1 runs (0.49, 1.52 and 2.20 million a frame)
observes VU1, because the GIF's arbitration at the step's end takes the XGKICKs up to its time and reads PATH1's packet
from VU1's memory, and VU1's XGKICK reads the GIF's estimate the board gives it at that step; a VU1 thread that trails
the EE therefore hands over at each such step, 63 ns a round trip on this machine, which is 31 / 96 / 139 ms a frame,
72 to 100 % of what the whole of VU micro mode costs now; and VU1 cannot lead the EE, because VIF1 writes VU1's data
memory while VU1 runs 35 to 151 thousand times a frame (80 to 90 % of its writes), each an input VU1 would have to stand
at. **What was built** (§57.3): `Pipe::run_to` answers at once below the unit's due cycle, the earliest of its next
pair's T stage (with the recompiler's planned stall) and its first landing, moving only its clock; compiled code reads
the earliest landing from its frame instead of calling for it. **At CPU weight 100: the dock from 234.0 to 227.8 ms a
frame (4.27 to 4.39 frames a second, 1.03 times), Misty Island from 420.3 to 407.5 (2.38 to 2.45, 1.03 times), the arena
from 728.1 to 701.6 (1.37 to 1.43, 1.04 times)**; every picture, sound and state digest equal on the windows, the boot's
760 frames and Jak's 900 on both EE tiers. The catch-up falls to 3.5 / 5.2 / 4.9 % of the thread. The IOP is 1.1 to
2.1 % and stays where it is (§57.5).

### 57.2 The call path, split (measured 2026-10-27)

**The profile** is §56.7's samples of the emulation thread, read again by the innermost frame in Sapphire's VU modules
(`callsplit.py` in the step's scratch): **catch-up** is `Vpu::catch_up` and `run_to`, `Pipe::run_to`, `run_pairs`,
`land`, `drain` and `settle` outside a helper, the idle checks, `stat`, `ready`, and the recompiler's checks at a call's
entry (`vu::jit::run` and the lines of `Tier::run` before its loop); **dispatch** is `Tier::run`'s loop, `code_for`,
`covers`, `valid`, `published`, `Frame::new` and `pending`; **work** is `vu::ops`, `vu::efu`, `sapphire::float` under
a VU frame, the helpers compiled code calls (`h_upper`, `h_lower`, `h_push`, `h_land`, `h_external`) with what they call,
and the register accessors.

| Share of the emulation thread | `jak-dock` | `jak-island` | `jak-arena` |
| --- | --- | --- | --- |
| VU micro mode, all of it (with its float calls) | 18.5 % | 22.8 % | 21.0 % |
| catch-up | **7.6 %** | **10.6 %** | **11.2 %** |
| dispatch | 2.6 % | 3.4 % | 3.1 % |
| work (compiled code apart) | 7.7 % | 8.5 % | 6.7 % |

**The census**: a counting build (counters at the calls, the board's steps, the kicks and the writes into VU1's memory,
kept in the step's scratch, not committed) over each window's 360 frames, with the lever of §57.3 in, its digests
those of §56.8. A frame's counts:

| Per frame | `jak-dock` | `jak-island` | `jak-arena` |
| --- | --- | --- | --- |
| board steps that bring the VUs along | 1.05 M | 1.87 M | 3.76 M |
| of which with VU1 running | 0.49 M (46 %) | 1.52 M (81 %) | 2.20 M (59 %) |
| VU1's `run_to` calls (the steps, VIF1's `ready` and its writes) | 2.04 M | 4.89 M | 8.78 M |
| of which nothing to run or land (answered by §57.3) | 1.73 M (85 %) | 3.97 M (81 %) | 7.49 M (85 %) |
| of which ran a pair; pairs a running call | 0.31 M; 1.19 | 0.92 M; 1.33 | 1.28 M; 1.22 |
| VIF1's polls of VU1's state (`Vpu::ready`) | 0.95 M | 2.91 M | 4.84 M |
| VU0's calls; nothing to do | 1.06 M; 80 % | 1.88 M; 95 % | 3.77 M; 92 % |
| VU1's starts; XGKICKs | 235; 1,085 | 2,043; 3,701 | 1,592; 3,545 |
| VIF1's writes into VU1's data memory; while VU1 runs | 39.0 k; 35.1 k (90 %) | 104.5 k; 83.5 k (80 %) | 175.9 k; 151.1 k (86 %) |
| the EE's stores into VU memory; BC2F and BC2T | 0; 0 | 0; 0 | 0; 0 |

**Which the lever addresses.** The catch-up is the largest third, and nine tenths of it is calls that find nothing
(§45.3 counted nine calls a pair on the opening; here 5.6 to 6.7 calls a pair on VU1 alone). A thread would move the work
(7 to 9 %) and VU1's part of the compiled code off the emulation thread, and turn the catch-up into handoffs between
threads (§57.4). Dispatch is the smallest part and does not dominate; the step took one helper call out of it (§57.3).

### 57.3 What was built: the call answered at the unit's due cycle

- **`Pipe::due`** (`src/vu/pipe.rs`): the earliest cycle by which `run_to` could change more than the unit's clock: the
  first flag write, quotient or EFU result in flight (`vu::jit::pending`, the minimum `drain` and `Vu::settle` test) and,
  while the unit runs, its next pair's earliest T stage, `next_t` and `kick_wait` and, on the recompiler, the T stage it
  planned for the pair it stopped before (`Tier::stalled_until`, §45.3's memo, valid while the PC and the micro memory's
  lending count are those it was planned with).
- **`Pipe::run_to`** answers a call below that cycle by moving the unit's clock and nothing else.
- **The frame carries the earliest landing** (`Frame::pending`), computed by the dispatcher as it builds the frame;
  compiled code loads it where it called `h_pending` at its entry, a helper call less for each of the 0.3 to 1.3 million
  entries a frame. `h_pending` is gone.

**Why it is exact** [A]. `Pipe::plan` starts every pair's T stage at `next_t.max(kick_wait)` and only raises it, so no
pair issues by `t` below that cycle, and the recompiler's dispatcher returns at once for the same condition and, below
the memo's planned cycle, at its second check (§45.3), each before touching anything; `drain` lands only from the front
of the flag queue while its cycle is at most `t`, and `settle` only a quotient or EFU result whose cycle is at most `t`
(the second EFU result waits for the first); the debugger's probe and the trace are asked only for a pair that runs.
So below `due` the full call changes nothing but `now`, which the answer moves. The due cycle is read from the unit's
state at each call, never cached, so no write elsewhere (a start, a state's load, FBRST, VU0's early Q through CTC2) can
leave it stale. The frame's landing is `pending` of the state the code is entered with, which `h_pending` returned at
the same instant. The counts the bench prints are unchanged; the bench clock's count of calls (§45.7) no longer counts
the answered ones.

### 57.4 Why VU1 is not on a thread: the observation points on this board

The plan's claim (§5.1, §5.12 lever 3) was that VU1 is observed only at a VIF1 or EE access to its memories or
registers, at XGKICK's packet and at the E bit, and that a thread synchronised there would be exact and would wait only
where another unit could see it. Exactness holds; the waiting does not, for three reasons found on this board:

1. **The GIF observes VU1 at every board step at which VU1 runs.** `Board::run_to` brings VU1 to the step's time, then
   the GIF: `run_gif` takes every XGKICK up to that time and runs PATH1 from VU1's data memory as it stands. A kick handed
   to the GIF a step late starts a step late, and the GIF's wake of the DMAC and its interrupts follow it, so the GIF
   cannot be run without VU1 at the step's time. There is no lookahead: a kick's cycle is known only by running VU1 to it.
2. **VU1 observes the GIF at every XGKICK.** The end of a PATH1 transfer VU1 is told comes from the GIF's estimate the
   board hands the port at the step (`Path1::set_free`, §33.4), so VU1 cannot pass a kick before the EE's side reaches
   the step that contains it.
3. **VU1's inputs arrive while it runs.** VIF1 unpacks into VU1's data memory while VU1 runs, 80 to 90 % of its writes
   (§57.2's census), each brought to the write's cycle (`Vif::store`'s `run_unit`). A VU1 thread running ahead of the EE
   would have to roll back past each such write, or stop at the EE's time, which is trailing.

**A trailing thread**, which runs VU1 to each step's time after the EE's side posts it, is exact by construction (a
queue of the board's calls, VU1's state read only when the queue is empty) and waits at each of the 0.49, 1.52 and 2.20
million steps a frame at which VU1 runs. **The handoff measured** (a two-thread round trip on atomics, both threads
spinning, `pingpong` in the step's scratch, three runs): **63 ns** on this machine (Ryzen 7 7700X, one CCD). That is
**31, 96 and 139 ms a frame** of waiting alone, against 43, 96 and 152 ms for the whole of VU micro mode, both units,
work and catch-up, on §56's frames (18.5 / 22.8 / 21.0 % of 235 / 424 / 726 ms). The best a trailing thread could do is
break even on the island and lose on the dock; it was not built, since building it would only confirm the arithmetic.

**What would make a thread pay** is fewer observations, and each way is outside this step:

- **C7** (§5.13), synchronisation at a bounded slack: timing-only, the exact cycle at which the GIF sees a kick moved
  within the quantum; it needs the tester's acceptance, and this step was to be exact.
- **Speculation with rollback**: VU1 runs ahead and is rolled back to the cycle of any input that arrives behind it.
  Exact, but each of the 35 to 151 thousand VIF1 writes a frame into a running VU1 would roll back unless the writes are
  tracked against what VU1 read and wrote since, a check in every compiled load and store; its state (32 KiB of memory,
  the registers, the pipelines' state, the PATH1 port) would be copied at each point it may return to.
- **A board at events for the GIF** (§51.8 #12, `Sapphire_Scheduler.md` §54): the GIF run only where it is observed,
  with its next event known, so that VU1's kicks are needed at the GIF's events rather than at every EE instruction.
  That is the precondition for an exact thread worth having, and it is a stage of its own.

### 57.5 The IOP

1.1 to 2.1 % of the emulation thread on the windows (§56.7's samples and this step's), with the SPU2 0.2 to 0.4 %: the
bound of a thread for it is 1.01 to 1.02, before its handoffs, and it is observed at every EE instruction's time
(`Sapphire_Scheduler.md` §54.9: it wakes the EE's DMAC from any instruction that starts a SIF channel). Named and left.

### 57.6 Speed (measured 2026-10-27)

**The method**: §56.6's windows with `--defaults` from the states at 6,000, 7,500 and 9,000, every run alone at CPU
weight 100 on a machine otherwise idle, the one-minute load 1.0 to 3.8 (2.4 to 3.8 for the windows). **Before** is §56's
"after" build (`5d054b083`, landed on WiseMan as `45f77f6c0`): its two rounds from §56.6 and a third today, taken because
the island's first two "after" rounds were 5 % apart; **after** is this step, three rounds on the island and two on the
others.

| Window | Before: means (median ms) | After: means (median ms) | Before → after, median of means | Frames a second | Longest frame |
| --- | --- | --- | --- | --- | --- |
| `jak-dock` | 234.0, 236.6, 231.8 (225.1, 228.7, 222.7) | 226.9, 228.7 (218.6, 219.4) | **234.0 → 227.8, 1.03×** | 4.27 → 4.39 | 0.37 → 0.36 s |
| `jak-island` | 417.8, 429.9, 420.3 (413.7, 415.9, 411.5) | 407.5, 429.8, 406.0 (399.2, 422.2, 402.3) | **420.3 → 407.5, 1.03×** | 2.38 → 2.45 | 0.88 → 0.84 s |
| `jak-arena` | 724.0, 728.1, 735.2 (654.8, 652.4, 677.6) | 706.0, 697.1 (620.5, 615.7) | **728.1 → 701.6, 1.04×** | 1.37 → 1.43 | 0.99 → 0.98 s |

The gain is about 3 %, the size of the rounds' spread on the island (one round of each build there sits 2 to 3 % above
the other two), and outside it on the dock and the arena. The catch-up's share fell by 4 to 6 points of the thread
(§57.7), the frames by 3 to 4 %: the answered calls still cost the call through `Box<dyn Micro>` and the reads of the
unit's state.

**The menus and the loading**, from power-on (the means of the frames named, ms; two rounds on the recompiler tier, one
on the interpreter tier, and the interpreter tier's Jak run again with both builds because its first after-run read
104.1 under load):

| Workload | Before (§56.6's after) | After |
| --- | --- | --- |
| the boot's language screen (frames 690–760) | 44.6, 49.0 | 44.6, 44.7 |
| the boot, frames 0–760 | 68.7, 70.5; interpreter 92.4 | 68.1, 68.1; interpreter 91.0, 91.1 (before again: 91.5) |
| Jak's start and loading by the script (frames 8–400) | 85.4, 86.3; interpreter 131.1 | 85.0, 85.0; interpreter 131.4 (before again: 132.1) |
| Jak, frames 0–900 | 60.5, 60.7; interpreter 94.8 | 60.1, 59.9; interpreter 95.0 (before again: 95.4) |

Unchanged within the rounds' spread: the menus run little VU micro mode.

### 57.7 Where the frame goes now (measured 2026-10-27)

**The method**: §56.7's sampler at 4 ms on every thread of one run of each window, the emulation thread classed by
§55.7's rules (VU micro mode with its float calls, as §57.2 counts it) and the call path by §57.2's.

| The emulation thread | `jak-dock` (12,481 samples) | `jak-island` (23,135) | `jak-arena` (36,470) | §57.2 before |
| --- | --- | --- | --- | --- |
| **VU micro mode**, with its float calls | **15.7 %** | **20.2 %** | **17.1 %** | 18.5 / 22.8 / 21.0 % |
| of which the catch-up | 3.5 % | 5.2 % | 4.9 % | 7.6 / 10.6 / 11.2 % |
| of which dispatch | 3.4 % | 5.3 % | 4.7 % | 2.6 / 3.4 / 3.1 % |
| of which the work | 8.0 % | 8.7 % | 6.8 % | 7.7 / 8.5 / 6.7 % |
| **EE**, all of it | 34.9 % | 25.7 % | 29.1 % | 32.3 / 24.7 / 28.3 % |
| of which the recompiler's dispatcher and exit replay | 10.9 % | 7.2 % | 6.2 % | 10.2 / 7.3 / 6.1 % |
| of which the timing model | 6.2 % | 5.1 % | 6.6 % | 5.8 / 4.8 / 6.6 % |
| **GIF** | 10.1 % | 9.2 % | 9.7 % | 9.1 / 8.8 / 9.2 % |
| **VIF** | 7.8 % | 8.1 % | 8.4 % | 8.3 / 7.7 / 8.3 % |
| **Compiled code** | 7.1 % | 8.5 % | 7.3 % | 7.4 / 8.0 / 6.4 % |
| **Idle skipping's own work** | 1.3 % | 3.1 % | 7.9 % | 1.2 / 2.9 / 7.5 % |
| Scheduler and synchronisation | 5.4 % | 4.4 % | 5.1 % | 5.4 / 4.3 / 4.9 % |
| GS on the emulation thread | 5.3 % | 8.2 % | 3.6 % | 5.0 / 8.7 / 3.5 % |
| IOP | 2.0 % | 1.3 % | 1.3 % | 2.1 / 1.1 / 1.1 % |

Dispatch's share rose because the calls that reach the dispatcher are now nearly all calls that run a pair, whose
entry the old split counted partly as catch-up, and the frame is shorter; its absolute cost is about what it was.

**The new top five** (the three windows together): **1. VU micro mode**, 15.7 to 20.2 %, now mostly its work and the
entry into compiled code a pair at a time (1.2 to 1.3 pairs a call); **2. the EE recompiler's dispatcher and exit replay**,
6.2 to 10.9 % (S37 to S39, §51.8 #6); **3. the GIF**, 9.2 to 10.1 %; **4. the VIF**, 7.8 to 8.4 % (3 and 4 together
§51.8 #10); **5. the EE's timing model**, 5.1 to 6.6 % (C1's share, refused at §44.9). On the arena idle skipping's own
work (7.9 %) comes before the fifth.

### 57.8 Tests and seeded faults (2026-10-27)

**Tests** [M, release]: the crate's lib tests 116 of 116 (5 ignored), `vu_micro` 25, `vu_jit` 19, `vu0` 14, `vif` 14,
`gif` 16, `micro` 12, `state` 11 and `threads` 10, all passing; the blast radius: Sapphire's crate **493 of 493** (5 ignored), WiseMan's `SapphireTests` **5 of 5**
on lavapipe. **The bench's gate**:
the three windows' digests (§56.8's `43D839015A0E4DA1`, `2C225C73DB4D3F9D`, `E220B19BD26D8EBC`; `D9F0AF56FD3DEF28`,
`4F27F29527E1B781`, `F074C69B5F66E65E`; `B77DC314C6AE837C`, `BCA8F00F4743A955`, `4ED52755B09F207E`) equal in every round
and in the census build; the boot's 760 frames (`50F06A02F1B76860`, `2EE7B3D10DF3C401`, `2051C6BB42228C53`) and Jak to
900 (`FFB0FCE98BD7C164`, `7CF6B8FA4F6C2701`, `0DCD2B18F0881625`) equal on both EE tiers. ps2autotests: **52 of 55 and 23 of 23** with both units recompiled, the
same three failing as §56.8 (`cpu/ee_fpu/sqrt`, `dma/vif/intstall`, `vu/lower/efu`).

**Seeded faults**, each made alone, the tests above run, the source restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | the landings left out of the due cycle (a flag write, Q or P due by the call's cycle not landed; an observer that does not wait for them) | `vu_jit`'s `q_and_p_land_at_their_latency_alike`, `random_microprograms_on_vu1_leave_the_units_as_the_interpreter_does` |
| F2 | the answer taken at the due cycle itself (`<=`: a pair issuing exactly at the call's cycle left for the next call, a wait to the wrong cycle) | `q_and_p_land_…`, `random_microprograms_on_vu1_…`, `every_operation_lands_on_the_same_cycle`, `a_state_saved_while_vu1_runs_continues_from_an_empty_cache` |
| F3 | the recompiler's memo read without the lending count (a stall planned before an MPG or a state's load trusted after it) | `random_microprograms_on_vu1_…` |
| F4 | the answered call leaving the unit's clock where it was (VGW and the clock the state writes) | `random_microprograms_on_vu1_…`, `q_and_p_land_…`, `every_operation_lands_…`, `a_state_saved_while_vu1_runs_…` |
| F5 | the frame's earliest landing set to none (compiled code never landing a flag, Q or P it would have) | `every_operation_lands_…`, `the_flags_land_four_cycles_on_and_their_readers_see_them_alike`, `q_and_p_land_…`, both units' `random_microprograms_…`, `a_state_saved_while_vu1_runs_…` |

All five were caught by `vu_jit`, whose verifier steps the interpreter's `Pipe::step` beside every compiled pair and
compares the units after each call; `vu_micro`, `vif`, `gif`, `state` and `threads` caught none.

**Left out of the gate, with the thread**: the threaded-against-unthreaded digests and states, the three runs of each
threaded case, and the faults planned for a thread (an observation that does not wait, a wait to the wrong
cycle, a VU1 write seen early, the thread left running across a state or a reset). F1 to F5 are their counterparts for
what was built: a landing an observer must see (F1, F5), a pair's cycle (F2), a memo across a change of memory (F3)
and the unit's clock in its state (F4).

### 57.9 Exposure

Running the player's BIOS and Jak and Daxter makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and
never a replacement writer.** The BIOS was handed to the machine by the link the step was given (market americas); it was
never opened, hashed, diffed, listed, disassembled or dumped, and no firmware folder was listed. Observed: the bench's
times, digests and counts, the census's counts of calls, steps, kicks and writes, and the host's instruction pointers in
Sapphire's own code; no program counter, operand value, microprogram word or memory of the BIOS or the game was printed
or logged. No emulator's source was read.

### 57.10 Named limits

- **The gain is the size of the island's spread**: three rounds there, two elsewhere; the dock's and the arena's are
  outside it.
- **The handoff is one machine's**: 63 ns between two cores of one CCD with both spinning. Two CCDs, a handheld's
  cores, or a waiting thread woken by the kernel would cost more, which only strengthens §57.4.
- **The census is the opening's three windows**; play may couple VU1 less or more. The argument of §57.4 rests on the
  board's structure (the GIF at every step), not on the counts.
- **The handheld's columns** remain owed with §51.9's.

### 57.11 Questions this step leaves

1. **[decision] VU1 on a thread.** An exact thread loses or breaks even on this board (§57.4). *Recommended*: do not
   build S42's threads now; take the board at events for the GIF first (§51.8 #12 extended to the GIF), which removes the
   per-instruction observation and is exact, and measure the thread again after it. C7 remains the timing-only
   alternative if the tester wants the thread sooner; speculation with rollback is not recommended.
2. **[routine] The plan's claim.** *Recommended*: amend §5.1 and §5.12 lever 3 to say the observation points are as
   §57.4 finds them on this board (done in `Sapphire_Plan.md` §5.1's amendment of 2026-10-27).
3. **[decision] The next step by overhead.** *Recommended*: the EE recompiler's remaining stages (S37 to S39, §51.8 #6),
   the largest exact lever left in absolute terms; or the GIF's and VIF's paths (#10, 16 to 18 % together), which the
   board at events would touch too.

### 57.12 What was read for this step

`Sapphire_Plan.md` §5.1, §5.12 lever 3 and the S42 and C7 rows; this page's §51.8, §55.7, §55.10 and §56;
`Sapphire_Native.md` §21.6, §23.1 and §23.2 (`Pipe`), §45.1 to §45.4 (`vu::jit`); `vu/micro.rs`, `vu/pipe.rs`,
`vu/mod.rs`, `vu/jit/mod.rs` and `emit.rs`'s entry, `board.rs`'s `run_to`, `run_gif` and `run_vifs`, `gif`'s
`Path1Port`, `vif`'s `store`. No emulator source, no firmware. 
