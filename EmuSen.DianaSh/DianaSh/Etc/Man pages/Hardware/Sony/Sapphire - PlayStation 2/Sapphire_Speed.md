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
