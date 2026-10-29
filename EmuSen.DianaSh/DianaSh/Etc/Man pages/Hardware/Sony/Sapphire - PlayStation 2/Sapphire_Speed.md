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

*Superseded 2026-10-28 by §58.7: the ceiling measurement showed that no lever on this list, nor all of them together, reaches full speed; the redesign of §58.7 replaces it as the plan.*

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

**Revised 2026-10-28, for *k* = 15 (D-1104).** Steps 1 and 2 hold for any *k*: only the rows at columns i < *k*
lose bits, the rows at 0, 2, …, 14 for *k* = 15, so the loss is read from `y`'s low 16 bits and `x`'s low *k*, and
`loss(x, y)` sums those eight rows' bits below *k* and their carries. Two facts make the product cheap [M, every low
part, `the_loss_never_exceeds_its_bound_over_every_low_part_it_reads` and a scratch count of 2³¹ and 2³² pairs]:

5. **The loss is at most 8 · 2^*k*** (2¹⁸ for *k* = 15), reached by three low parts in 2³¹, the first x ≡ 0 and
   y ≡ AAAAh, eight digits of −2.
6. **The loss is congruent to x · y modulo 2^*k*** on every low part, as it must be: the dropped columns hold the
   product's own low bits, less what the rows above carried out of them.

So `product` returns x · y unchanged when `y`'s low 16 bits are zero (no digit below *k* is non-zero) or when the
product's bits below column 23 are at least 2¹⁸ (then x · y − loss has the same bits from column 23 up, which is all
`mul` reads), and otherwise subtracts `loss`. On uniform significands the second test fails in 1 in 32; on an exact
product with a non-zero low part it always does. `product` is now defined by its bits from column 23 up, and the
tests compare those with `booth`'s. **The one-unit-low rule** that follows: for *k* = 15 the result is RZ(x · y) one
unit low exactly when ⌊loss / 2^*k*⌋ > ⌊D / 2^*k*⌋, D the product's bits below the 24 kept (by 6); D ≥ 2¹⁸ always
keeps RZ(x · y). For 1.0 × y with 1.0 as the multiplicand, x ≡ 0 modulo 2^*k* and the loss is 2^*k* for each negative
digit at a column below *k*, which is F0bes's list (D-1104).

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

## 58. The ceiling: what the design itself costs, measured with the synchronisation relaxed (2026-10-28)

*Decided 2026-10-28 by the tester: stop climbing the lever list. §51.8 put every exact lever together at about 2×,
still 12 to 38 times short of 16.68 ms, so before more speed work the step measures what the design costs. This is a
measurement and nothing in the core changes: the two builds it ran were patches kept outside the repository and reverted.
Framed by three decisions of the same day: Sapphire's goal is compatibility, more games played correctly than other
PlayStation 2 emulators, with Unix-style modularity and results that are bit-accurate rather than timing that is
cycle-accurate, so each point below is judged by its results and a difference only in timing is acceptable; the first
scoreboard is the games on the handheld, correct and at full speed, with no per-game fixes, the defaults working as
shipped and the trade-offs offered as the player's options. Disputes would start at D-1080; the step raised none.*

### 58.1 In one paragraph

**The target is 16.68 ms a frame on every window.** A bench frame is one field, and Jak and Daxter changes its
displayed buffer at 47.9, 47.1 and 33.8 new pictures a second on the dock, Misty Island and the arena (§58.2). It is a
60-field game that drops pictures, not a 30-frame one, and even a game that drew at 30 would need every field emulated
in real time. **The sweep** (§58.3): a board that catches up only every N EE cycles, or at the next scheduled event,
**breaks Jak's 3D at N = 64 already**: VU1's geometry turns to spikes on all three windows, and four VIF result tests
fail, because the EE reads the units at a stale time. A second build that **still brings the board to the EE's time at
every EE access to a unit**, and batches only the catch-ups nothing observes, keeps ps2autotests at 52 of 55 and 23 of
23 at every N, the boot's picture and sound digests at every N, and **Jak's pictures right at N = 64**, with dock 234–243 →
207 ms, island 414–434 → 302 ms and arena 715–744 → 488 ms (1.1 to 1.5 times). From N = 512 the game stalls: the
pictures freeze, though the geometry stays right. **The floors at that point** (§58.4): the EE's interpreter, its
dispatcher and exits, the VUs and the board lead; MMI, COP1 and COP2 are interpreted entirely, and the interpreter tier
alone is *faster* than the recompiled tier on all three windows. **The projection** (§58.5), the emulation thread on the
desktop: the best workable N 207 / 302 / 488 ms; with every EE instruction compiled at today's measured 7 to 9 ns, 171 /
263 / 419; with the GS's drawing off the thread as S44's mirror costs, 156 / 232 / 388. That is 9 to 23 times short on
the desktop, 12 to 33 on the handheld, and the software GS's workers alone are busy 90 to 98 ms per dock frame. **Full
speed is not reachable by this design's levers.** It needs a design whose cost per EE instruction, per VU pair and per
board observation is about ten times lower (§58.7).

### 58.2 The target (measured 2026-10-28)

The bench, extended in the throwaway build to count over each window's 300 timed frames the frames whose displayed
picture's digest changed and the changes of DISPFB1's base pointer (the buffer the PCRTC shows, GS User's Manual §4.2):

| Window | Picture changed | DISPFB1 changed | New pictures a second (of 59.94 fields) | Target |
| --- | --- | --- | --- | --- |
| `jak-dock` | 300 of 300 | 240 | 47.9 | 16.68 ms |
| `jak-island` | 300 of 300 | 236 | 47.1 | 16.68 ms |
| `jak-arena` | 210 of 300 | 169 | 33.8 | 16.68 ms |

The picture changes on every field because the display is interlaced; DISPFB1 is the game's own count of pictures.
Jak aims at 60 and drops pictures where its frame's work exceeds a field. That is the emulated console's timing (P58a:
on a console the dock and the island hold nearer 60; not measured, since no hardware is used). **The target per bench
frame is 16.68 ms in every case**: the emulator must run every field's cycles in real time, whatever rate the game draws
at. A 30-picture game is not a 33.4 ms target.

### 58.3 The sweep (measured 2026-10-28)

**The builds.** *Plain*: `Board::run_to` returns without doing anything when its time lies before both the next
scheduled event and N cycles past the last full catch-up (after the quick path for idle units, so both tiers see the
same board), so the units, the IOP and the lines move only every N cycles or at an event. *Synced*: as plain, but every
EE access to a unit's registers, FIFOs, VU memories, the GS's privileged registers, IOP RAM or COP2 first runs the
catch-ups the slack skipped, and the compiled code's pure register reads take the slow path while any are owed. RDRAM is
current in both builds, since the DMAC runs only at its events. N = 1 is the shipped path: its digests are §57.8's.
**The method**: §57.6's windows with `--defaults`, at CPU weight 100 on an otherwise idle machine (one-minute load 1.2
to 4.2); two rounds at N = 1 and at synced N = 64, the second under the sampler, and one round elsewhere. Where a run's
final state could not be saved (§58.8) the mean is taken from the bench's progress lines, frames 6,100 to 6,300 and
alike, whose resolution is a second over 200 frames, ±5 ms. Pictures every 100 frames were compared by eye with N = 1's.

| N | Build | `jak-dock` ms (fps) | `jak-island` ms (fps) | `jak-arena` ms (fps) | Picture digests | By eye | Language screen ms | ps2autotests |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | shipped | 234.1, 242.5 (4.2) | 413.7, 433.8 (2.4) | 714.6, 744.1 (1.4) | §57.8's | right | 44.7 | 52 / 55, 23 / 23 |
| 64 | plain | 323.4 (3.1) | 319.6 (3.1) | 1,036.0 (0.97) | all drift | spiked geometry, then wrong scenes | 45.6 | 51, 23 |
| 512 | plain | ~365 | ~500 | ~400 | all drift | broken, frozen | 46.7 | 51, 23 |
| 4,096 | plain | 446.3 | ~390 | ~495 | all drift | broken, frozen | 49.9 | 48, 23 |
| 18,743 (a line) | plain | 414.9 | ~385 | ~400 | all drift | broken, frozen | 66.7 | 48, 23 |
| 64 | synced | 204.6, 210.5 (4.8) | ~305, ~300 (3.3) | 485.6, 490.1 (2.0) | drift | **right: the same scenes, a few frames out of phase** | 46.5 | **52, 23** |
| 512 | synced | ~170 | ~260 | ~225 | drift | geometry right, the game stalls | 47.0 | 52, 23 |
| 4,096 | synced | ~180 | ~300 | ~355 | drift | geometry right, the game stalls | 46.9 | 52, 23 |
| 18,743 | synced | ~190 | ~300 | ~210 | drift | geometry right, the game stalls | 46.0 | 52, 23 |

The boot's picture digest (frames 0 to 760) is §57.8's at every N in both builds; its sound digest is too, but at plain
N = 4,096. Jak's sound digests are equal to N = 1's at every point where the summary printed. The plain build's times
are **not an upper bound on speed**: once the game misbehaves it does different work, and on the dock and the arena it
does more of it (more pictures drawn at N = 64). The synced times from N = 512 are those of a stalled game, pictures
changing 0 to 30 times in 300 frames, and are listed for scale only.

**The emulation thread's top five** (§55.7's classes by the innermost frame, the EE's blocks named through the
recompiler's perf map, §58.4's table for the rest; N = 1's island from §57.7's samples, whose compiled code is not split
by unit):

| Point | `jak-dock` | `jak-island` | `jak-arena` |
| --- | --- | --- | --- |
| N = 1 | EE interpreter 17.2, VU 15.4, EE dispatcher and exits 12.8, board 12.6, GIF and VIF 7.7 each | VU 19.8, EE interpreter 14.6, board 13.1, dispatcher 8.7, compiled code 8.5 | EE interpreter 20.9, VU 17.3, board 13.8, dispatcher 8.8, VIF 6.8 |
| synced N = 64 | EE interpreter 22.3, dispatcher 19.7, VU 11.3, board 9.5, GS 7.3 | EE interpreter 15.7, VU 15.6, dispatcher 14.8, board 11.6, GS 10.3 | EE interpreter 18.0, VU 16.9, dispatcher 16.7, board 10.1, VU compiled code 7.5 |

The batching takes most of the GIF's and VIF's share (15.4 → 7.6 % on the dock, 13.4 → 7.9 % on the arena) and some of
the board's: the per-step checks that find nothing to do; the EE's own costs do not move and now lead.

### 58.4 The units' floors at synced N = 64 (measured 2026-10-28)

Each class's share of the sampled run times its mean, ms a frame. VU0 and VU1 are split by their pairs a frame (dock
231 k and 340 k, island 123 k and 1,220 k at N = 1, arena 421 k and 2,534 k), an approximation, since their costs a pair
differ. The GS workers' time is the bench's own busy count, each of four.

| ms a frame | `jak-dock` (210.5) | `jak-island` (~300) | `jak-arena` (490.1) |
| --- | --- | --- | --- |
| EE: compiled code | 6.4 | (not attributed) | 10.0 |
| EE: interpreter (the fallbacks: every MMI, COP1 and COP2, part of the rest) | 47.0 | 47.1 | 88.2 |
| EE: dispatcher, exits and the replay's per-step work | 41.5 | 44.3 | 81.9 |
| EE: timing model | 13.8 | 12.1 | 26.8 |
| floats (`sapphire::float`, EE and VU) | 5.3 | 11.8 | 21.4 |
| VU1 (its calls, work and compiled code) | about 18 | about 66 | about 103 |
| VU0 | about 12 | about 7 | about 17 |
| GIF | 7.7 | 11.1 | 16.2 |
| VIF | 8.5 | 15.5 | 22.6 |
| GS on the emulation thread | 15.4 | 31.0 | 31.2 |
| GS workers, busy, each | 90 to 98 | 34 to 46 (N = 1) | 35 to 48 |
| IOP and its units | 6.2 | 7.4 | 8.4 |
| SPU2 | under 0.5 | under 0.5 | under 0.5 |
| Scheduler and board | 20.1 | 34.9 | 49.7 |
| Idle skipping's own work | 1.1 | 1.1 | 1.3 |

**The EE's instructions a frame by tier and class** at N = 1, where both tiers' digests are equal, so their streams are
the same: the interpreter tier counts every instruction executed (idle skipping's passed steps apart: 4.1, 3.8 and 1.7
million a frame are passed, not run); the recompiled tier counts those it interprets; compiled is the difference, which
equals the bench's compiled steps to within 1 %.

| Thousands a frame | Loads and stores | MMI | COP1 | COP2 (macro) | Other | All |
| --- | --- | --- | --- | --- | --- | --- |
| `jak-dock`: executed; interpreted | 431; 188 | 241; 241 | 137; 137 | 145; 145 | 712; 200 | 1,665; 911 |
| `jak-island` | 433; 230 | 233; 233 | 153; 153 | 136; 136 | 976; 449 | 1,930; 1,200 |
| `jak-arena` | 789; 516 | 306; 306 | 187; 187 | 210; 210 | 2,494; 1,620 | 3,986; 2,840 |

So 29 to 46 % of the executed instructions are compiled. Every MMI, COP1 and COP2 instruction is interpreted (S38 and
S39 are not built), and so are 44 to 65 % of the loads and stores and 28 to 65 % of the rest, which run where a block
leaves for one of them. **The recompiled tier is slower than the interpreter tier on all three windows**: 234.1 and
242.5 against 229.6 ms, 413.7 and 433.8 against 403.3, 714.6 and 744.1 against 692.9, one interpreter-tier round each.
At this coverage the dispatcher and the exits cost more than compiling saves.

### 58.5 The projections

**The recompiler's** (§58.4's numbers, synced N = 64): the compiled code costs 6.4 ms over 693 k compiled instructions
on the dock (9.2 ns each) and 10.0 ms over 1,343 k on the arena (7.4 ns); the interpreter costs about 40 ns an
interpreted instruction before the dispatcher's share. With every instruction compiled at that cost, the interpreter's
47.0 / 47.1 / 88.2 ms become 10.9 / 8.4 / 19.0, the dispatcher and the timing model left as measured (if blocks stopped
leaving for the fallbacks the dispatcher would fall too; it is not projected, for want of a measurement). As a second
reference, Mars's recompiler measured 1.1 to 2.1 ns a compiled instruction against its interpreter's 5.4 to 6.6
(`Mars_Recompiler.md` §7, `Mars_Performance.md` §24) and MarsRT's RSP's compiled blocks about 3 ns a step (`Mars_Native.md`,
after `Mars_Rsp.md` §11.2): Sapphire's compiled instruction costs four to eight times Mars's, and its interpreted one six times.

**The critical path**, the emulation thread (no other thread bounds it on the desktop: the GS workers' busy time per
frame is under the frame's on every window), and the handheld at 1.3 to 1.4 times (`Sapphire_Plan.md` §5.12):

| ms a frame | `jak-dock` | `jak-island` | `jak-arena` | Short of 16.68 ms (desktop; handheld) |
| --- | --- | --- | --- | --- |
| (a) synced N = 64 | 207 | 302 | 488 | 12–29×; 16–41× |
| (b) (a) with every EE instruction compiled at 7–9 ns | 171 | 263 | 419 | 10–25×; 13–35× |
| (c) (b) with the GS off the emulation thread (S44: 0.06 to 0.12 ms a frame of recording, `Sapphire_Native.md` §52.5) | 156 | 232 | 388 | 9–23×; 12–33× |

(c) assumes the device draws what the workers draw now at no cost to the processor: S44 built the mirror and the
transfers, and its device rasteriser's throughput is owed (§52.6.4). That is also the bound after (c) on the dock,
where the software workers are busy 90 to 98 ms a frame. **After (c) the emulation thread is bound by** the VUs (30 /
73 / 120 ms, their floats apart), the EE's dispatcher, exits and timing model (55 / 56 / 109), the board and scheduler
(20 / 35 / 50), and the GIF and VIF (16 / 27 / 39). None of them is near 16.68 ms alone on the island or the arena.

### 58.6 The accuracy side: what a batched board must keep exact

**ps2autotests by point**: the three failures of §57.8 (`cpu/ee_fpu/sqrt`, `dma/vif/intstall`, `vu/lower/efu`, all
result tests failing for their own named reasons) at every point. The plain build adds `dma/vif/unpack` from N = 64, and
`dma/vif/basic`, `stcycl` and `stmod` from N = 4,096. The synced build adds none at any N; the IOP's 23 pass
everywhere.

**Timing tests and result tests.** Of the 78 programs, the timing tests are those whose expected text depends on
cycles or on the order of events: `cpu/ee/branchdelay`, `lsudelay`, `cpu/ee_fpu/branchdelay`, `cpu/ee_cop0/performance`,
`vu/lower/fdivdelay`, `cpu/iop/branchdelay`, `hilodelay` and `lsudelay` (one processor's pipeline, which the board's
slack cannot touch), and `dma/dmac/cpcond`, `tagintr`, `gs/signal`, `gs/label` and the kernel's
`interrupt/dmac_order` and `intc_order` (the order of interrupts and of the DMAC's ends). The rest are result tests:
values computed and read back. **No timing test failed at any N in either build.** The four that failed under the plain
slack are result tests that read a VU's memory or the VIF's registers after a transfer, and they failed because the read
saw a VIF that had not yet run. Bringing the board to the EE's time at the access was enough to pass them at every N.

**What this says for a block-scheduled design.** (1) **The observations must stay exact**: every access the EE makes to
a unit sees the unit at the access's time. Without that, results change within 64 cycles (the plain build's spikes and
VIF tests). (2) **The background catch-ups can be batched** to at least 64 cycles without any result changing that a test,
the boot or Jak's pictures show: a timing-only compromise. (3) **Something else must be kept exact beyond 64 cycles.**
From N = 512 the game stalls with its geometry right and every test passing. The candidates, not separated by this step,
are the interrupt lines (sampled only at catch-ups in this build, so a VIF, VU or GS interrupt reaches the EE up to N
cycles late), VU1's end and XGKICK as VIF1's MSCAL and FLUSH see them, and idle skipping's waits, which pass cycles the
batched board did not run. Finding which is the redesign's first measurement.

### 58.7 Is full speed reachable, and what a redesign must do

**Not with this design's levers.** Batching the board's background work (exact in results at N = 64), compiling every
EE instruction at today's cost and taking the GS's drawing off the thread together leave the dock at 156 ms, the island
at 232 and the arena at 388 on the desktop, 9 to 23 times short, and 12 to 33 times on the handheld. No single unit
remains whose removal closes that. The cost lies in every unit's per-instruction, per-pair and per-observation
overhead: the EE's dispatcher and exits at about 60 ns per compiled step, its timing model, VU1 entered a pair or two at
a time through calls, and a board consulted for every unit after every instruction or batch. **What a redesign serving
compatibility and speed together must do**, each piece judged by results as §58.6 sets out:

- **Run the EE in long compiled blocks covering every class** (MMI, COP1, COP2 included), with the board brought along
  only at the block's I/O accesses and at scheduled events. Exact in results by §58.6 (1) and (2), timing-only otherwise,
  and the first default the handheld needs.
- **Run VU1's microprograms whole on their own thread**, synchronised only at VIF1's and the EE's accesses, XGKICK and
  the E bit, once the GIF runs at its own events (§57.4; on §57.7's samples the GIF's arbitration, checked twice at every board step and finding
  nothing, is about 3 % of the thread). By §58.3 the VIF's and GIF's per-step work mostly disappears under batching anyway.
- **The GS on the device** with its rasteriser (S45 and S46), since the software workers alone are near 100 ms per dock
  frame.
- **Drop the per-instruction timing model for a cost per block** (C1, timing-only; refused at §44.9 under the old rule,
  and acceptable under results-first).
- **Keep exact what results depend on**: observations, the interrupts' order (§58.6 (3)), every value read back.

**For the handheld's defaults**: batched synchronisation with exact observations (the synced build's form, at the
largest N that leaves the game behaving, 64 measured) and C1 would have to be on by default. Both are timing-only
compromises that no test or picture here distinguishes. N past the point where the game stalls, the plain build's
unsynchronised observations, and C3 (the floats on the host with clamping, byte-changing) would be options only. Even
so, the measured costs say the cycle-stepped architecture cannot reach the target, so the recommendation is the redesign
above, not further levers.

### 58.8 Named limits

- **One round** at the intermediate points, two at N = 1 and synced N = 64; for scale, not precision.
- **Times from progress lines** where the run could not save its final state: with the board lagging, a queue (a FIFO
  or the kicks) grew past its slots in the state, which is a fault of the slack itself and the reason the plain and
  synced builds are not shippable as built.
- **The island's N = 1 profile** is §57.7's, which does not split the compiled code by unit. Synced N = 64's island
  perf map attributed almost none of the EE's blocks.
- **VU0 and VU1 split by pairs**, not by samples.
- **The stall from N = 512 is not diagnosed** (§58.6 (3)).
- **The handheld's columns are the desktop's times 1.3 to 1.4**, not measured.

### 58.9 Exposure

Running the player's BIOS and Jak and Daxter makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and
never a replacement writer.** The BIOS was handed to the machine by its link (market americas); it was never opened,
hashed, diffed, listed, disassembled or dumped, and no firmware folder was listed. Observed: the bench's times, digests
and counts, the instruction classes' counts (opcode fields counted, no program counter or word logged), the pictures
every 100 frames looked at as a scene, and the host's instruction pointers in Sapphire's own code. No emulator's source
was read.

### 58.10 What was read for this step

This page's §51.8 and §57; `Sapphire_Plan.md` §5.1 and §5.12's handheld factor; `Sapphire_Native.md` §33.1 to §33.5
and §52.1 and §52.5 (S44, from its branch); `Mars_Performance.md` §24 and `Mars_Native.md`'s RSP figure for the second
reference; `board.rs`, `gif/mod.rs`, `vif/mod.rs`, `machine.rs`'s frame loop, `blocks.rs`'s replay and `ee::step`. No
emulator source, no firmware.

## 59. The VU's multiply-add on the host's floats, exact (2026-10-28)

*Numbered §59 after §58. A measurement kernel behind the `lanes` feature, beside R0.4's (`Sapphire_Redesign.md`, R0
results): nothing in the default build uses it. It is built against the multiplier as D-1104 settles it, *k* = 15, the
same day. Disputes would start at D-1105; the step raised none.*

### 59.1 In one paragraph

`float_host` computes `float::add`, `mul` and `fused` for four lanes, or two operations in eight, on the host's
single-precision arithmetic, bit for bit with the MAC flags, in two forms: one with AVX-512's embedded rounding towards
zero (`{rz-sae}`, `_mm512_*_round_ps`), and one with AVX2 and no rounding mode at all, its sums and products formed
exactly in doubles and cut to 24 bits by a mask. **No rounding mode is set anywhere**; MXCSR is never read or
written. A lane the host cannot take exactly (§59.2, §59.3) sends its vector to R0.4's integer form, `float_lanes`.
Graded by R0.4's suites and a billion random lanes, with seven seeded faults, a fused multiply-add among them, all
caught. **Timed on VU1's recorded mix, the AVX2 form is the fastest exact form measured, 1.56–1.64 ns a lane operation
on the desktop and 1.66–1.70 on the handheld, against the handheld's mark of 1.125: it fails by 1.47–1.51 times.**
The `{rz-sae}` form is slower than the AVX2 one on both machines (2.07–2.12 on the handheld).

### 59.2 The product

The host's product of two normal operands rounded towards zero is RZ(x · y), and the console's is RZ(x · y) or one
unit below it (§56.4, revised for *k* = 15). The rule for a lane, every test on integers beside the one multiply:

- **a zero operand** (field 0, the would-be denormals included): ±0 with the operands' signs combined, taken;
- **both fields 1 to 254 and their sum 129 to 380**: the result's field is 2 to 254, so neither the host's denormals
  nor its overflow is met, and a decrement below cannot leave the normal range;
- then one of three, each exact [A, §56.4's facts 5 and 6]:
  1. `y`'s low 16 bits are zero: no digit below the column is non-zero, nothing is lost, the result is RZ(x · y);
  2. the product's bits below column 23 are at least 2¹⁸ (`MOST_LOST`), read from the low 32 bits of the integer
     product (`vpmulld`): the loss cannot reach column 23, the result is RZ(x · y);
  3. **`x`'s low 15 bits are zero** (a short multiplicand: 1.0, 0.5, 1.5 and the like): every row's bits below the
     column are zero, a positive digit loses nothing and a negative one 2^*k*, so the loss is 2^*k* times the negative
     digits below the column, which a few shifts and masks count (`negative_digits`,
     `a_multiplicand_with_no_bits_below_the_column_loses_two_to_the_k_a_negative_digit`, every such x against every
     low part); the result is one unit low exactly when that count exceeds ⌊D / 2^*k*⌋, D the product's bits below the
     24 kept, so the kernel decrements those lanes' bits (which crosses a binade correctly, §56.4).
- anything else (a field of 255, a result at field 1 or out of range, a long multiplicand with a small D) goes to the
  integer form.

**The exact-product weakness** the check was expected to have: on the mix's operands **55.8 % of the products of two
non-zero operands are exact** and the first two tests refuse **17.4 %** of them; with the third, **1.4 %** are refused
[M, `lanes mix … host`'s shares line, both windows]. The operands are §56.2's classes as R0's fourth addendum drew
them, a quarter with a zero fraction and a quarter with a short one; the games' own operands were not recorded, so the
share in play is not measured.

### 59.3 The sum

D-25's adder aligns the smaller operand S to the larger L's 24 bits and one more and truncates; that is RZ(L + S′),
S′ being S cut towards zero at L's guard bit, 2^(e−24) for L in [2^e, 2^(e+1)) [A]. With L's field f:

- the `{rz-sae}` form: when the fields differ by 2 or more, |S| < 2^(e−1), and with C = ±2^(e−1) of S's sign,
  RZ(S + C) lies in [2^(e−1), 2^e), whose unit is the guard bit, so **S′ = RZ(S + C) − C**, the subtraction exact
  (Sterbenz); when they differ by 0 or 1, S's own unit is at or above the guard bit and S′ = S. Then RZ(L + S′);
- the AVX2 form: S′ by clearing d − 1 fraction bits of S (d the fields' difference, 2 to 24; nothing left from 25),
  then L + S′ in double precision, exact since its bits span at most 26, and cut to 24 bits by masking 29.

Taken when f is at most 253 (so no sum reaches field 255, where the host would give its largest finite number) and
f is at least 25 or S is zero (so a non-zero sum is at least 2^−126 and the host never forms a denormal where the
console flushes). Zeros, the would-be denormals among them, become the host's signed zeros first; the host's
+0 + −0 = +0 and −0 + −0 = −0 under rounding towards zero are Table 8-5's. An exact cancellation gives +0, as `add`
does.

**The multiply-add** is the product and then the sum, **two separately truncated steps, never fused**: a fused
multiply-add would round x · y + acc once, which D-25 and D-26 exclude (seeded fault F1). An overflowing product and
an accumulator at field 255 are never on the host path. The MAC flags are Z and S of the bits (U and O cannot arise
on the path), the term's for the sticky flags.

### 59.4 Grading and seeded faults (2026-10-28)

`cargo test --release --features lanes --lib float` [M]: `float_host`'s tests against `float` lane by lane, each of
add, sub, mul and both multiply-adds, four lanes and paired, both forms on the desktop (AVX-512 present), field masks
random or full:

| Test | Cases | Taken whole by the host |
| --- | --- | --- |
| R0.4's reduced set, every pair beside three others, every fourth pair filling the vector | 4,596,736 vectors | over 100,000 each of add, mul and multiply-add (asserted) |
| §56.5's classes, half the vectors one lane repeated | 4,000,000 vectors | the same |
| random (ignored; 404 s with the other three billion-case tests, two at a time) | 1,000,000,000 lanes | add 175,957,153, mul 55,842,346, multiply-add 95,538,758 vectors |
| the vector masks against the scalar rules (`mul_taken`, `add_taken`, `fused_taken`) | 1,000,000 vectors | — |

All equal, and `float_lanes`' own three suites with its loss revised for *k* = 15 (§59.6). **Seeded faults** (each
made alone, the tests run, the source restored by the script that made it):

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | **the multiply-add fused**: the sum as `_mm512_fmadd_round_ps` of acc, a and b, rounded once | the reduced set, the classes |
| F2 | the guard cut left out (S′ = S) | the reduced set, the classes, the kept vector |
| F3 | a short multiplicand's lanes not decremented | the reduced set, the classes |
| F4 | the sum taken to field 254 | the reduced set, the classes, the masks |
| F5 | the AVX2 form's cut at 28 bits | the reduced set, the classes |
| F6 | `float_lanes`' loss for long multiplicands left out | both kernels' reduced sets and classes |
| F7 | the would-be denormals left to the host's adder | the reduced set, the classes, the kept vector |

### 59.5 A failure from a stale mutant

The first billion-lane run (2026-10-28) failed one assertion: `four::add` on a = [07F3A3B1h, 15F3A3B1h, 858A71D6h,
92AA88B6h], b = [807FFFFFh, …] gave 07F3A1B1h in lane 0 where `float::add` gives 07F3A3B1h. The same inputs run alone
gave 07F3A3B1h, and the whole seeded run passed when repeated after a source edit (245 s). **The cause:** the run
followed the seeded faults, the last of them F7 (the would-be denormals left to the host: b's lane 0 is one), and the
script restored each source by moving its copy back, which keeps the copy's older time, so cargo saw nothing newer than
the mutant's build and ran it. F7 is caught by this very vector (§59.4). Every later run in this step followed an
edit or a touch of the sources. The vector is kept as `the_vector_a_stale_mutant_failed_on_equals_float`, named for
how it was first met.

### 59.6 What *k* = 15 does to R0.4's lane form

`float_lanes`' multiplier took §56.4's eight-entry loss table for *k* = 1, so D-1104 made it wrong; it now takes the
loss as `float::product` does: zero where `y`'s low 16 bits are zero, 2^*k* times the negative digits where `x` is
short (`negatives`, in lanes), and otherwise, only in the lanes whose bits below column 23 are under 2¹⁸, `float::loss`
lane by lane after a test of the whole vector. R0.4's suites pass on it. Its time on VU1's mix grows: **1.81–2.12 ns a
lane operation on the desktop and 1.92–1.96 on the handheld for the AVX2 form** (R0's fourth addendum: 1.28–1.31 and
1.43–1.47 at *k* = 1), 1.63–1.69 and 1.79–1.85 for the AVX-512VL form. R0.4's verdict, a failure, stands and widens.

### 59.7 Speed (measured 2026-10-28)

`lanes mix FILE 7 FORM` on R0's recorded mixes of VU1 (`mix-arena.txt`, `mix-island.txt`), pairs of one kind in eight
lanes and the rest in four, ns a lane operation written, the median of seven, two rounds. The desktop at a one-minute
load of 2.7 to 3.1 (other jobs running, CPU weight 20); the handheld on the charger (ACAD online before and after),
EmuSen not running, load 0.19 to 0.26, under `systemd-run --user`, the binary needing GLIBC_2.34 against its 2.43.

| Form | Desktop, arena r1 / r2, island r1 / r2 | Handheld, arena r1 / r2, island r1 / r2 | Against 1.125 |
| --- | --- | --- | --- |
| `float_host`, AVX2 | **1.563 / 1.568, 1.566 / 1.638** | **1.658 / 1.667, 1.668 / 1.702** | fails, 1.47–1.51× |
| `float_host`, `{rz-sae}` | 1.913 / 1.907, 1.941 / 1.951 | 2.071 / 2.082, 2.100 / 2.115 | fails, 1.84–1.88× |
| `float_lanes`, AVX2 (*k* = 15) | 1.824 / 1.806, 1.832 / 2.121 | 1.916 / 1.924, 1.944 / 1.956 | fails |
| `float_lanes`, AVX-512VL (*k* = 15) | 1.634 / 1.634, 1.693 / 1.640 | 1.790 / 1.793, 1.852 / 1.804 | fails |
| the stream alone (`floor`: the same calls, no arithmetic) | 0.521 (arena, one round) | 0.591 / 0.546, 0.553 / 0.547 | — |

**What the numbers say.** Measured: the host's floats take 13 % less time than the integer form (AVX2 against R0's AVX2 at
*k* = 15) on the handheld and fail the mark. The stream's own calls and copies cost 0.55–0.59 ns of each lane
operation on the handheld; the arithmetic is the rest, about 1.1 ns (an estimate by subtraction: the floor's calls are
kept out of line where the kernels' may be inlined). So even a kernel of no cost through this interface would leave
half the budget used. The `{rz-sae}` form is slower, not faster, on both machines (Zen 4's and the handheld's 512-bit
operations, their mask registers and the two embedded-rounding additions S′ takes, against the AVX2 form's integer
cut); R0's AVX-512VL form showed the same direction (R0.4 in the conclusion).

### 59.8 Named limits

- **The operands are drawn, not recorded**: §56.2's classes in R0's shares; the exact products' share in play, and so
  the share refused, is not measured.
- **The mark is per lane operation through `lanes mix`'s interface**, which costs about half of it alone (§59.7).
- **The default build is unchanged by this step**; D-1104's change to the multiplier is the other commit's.

### 59.9 Questions this step leaves

1. **[decision] The host path's place.** *Recommended*: keep `float_host` as a measurement kernel only. It is exact
   and the fastest form measured, but 1.47–1.51 times over the handheld's mark, and the interface alone takes half of
   it; a VU1 kernel that keeps ACC and the operands in registers across a microprogram's pairs (the recompiler's form)
   is where the per-call cost could go, and it would need the same proofs.
2. **[routine] The `{rz-sae}` form.** *Recommended*: keep it beside the AVX2 form as the measured counter-example, the
   research's preference for embedded rounding not holding on either machine.
3. **[routine] The games' operands.** *Recommended*: when VU1's recompiler is next profiled, count the exact products
   and the refused lanes on recorded operands (a counting build, as §56.2's census), since §59.2's shares are drawn.

### 59.10 What was read for this step

`~/.cache/emusen/tmp/vu-madd/research.md` (theory, with its sources: the Intel SDM on VADDPS's embedded rounding,
Rust's `_mm512_add_round_ps` documentation, Ogita, Rump and Oishi 2005 and Rump, Zimmermann, Boldo and Melquiond 2009
for the exactness of the error terms); this page's §56; `Sapphire_Redesign.md`'s R0.4 sections; `Sapphire_Disputes.md`
D-25, D-26, D-43, D-44; `float.rs`, `float_lanes.rs` and `examples/lanes.rs`. No emulator source, no firmware.

## 60. Where an arena field goes on the tree with E1, by unit (2026-10-29)

A measurement only: nothing in the core changed. `EmuSen_Recompiler.md` §15.4 timed the arena at about 757 ms a field
on Cranelift's tier and about 790 with E1's baseline, against a real-time 16.7 ms, while the EE's compiled code was a
small share. This section finds where the rest of the field goes, what the baseline does more of, and why its prefix
entries do not converge.

### 60.1 The method

The tree is `recompiler-e1` (99c50bcdc), as merged into WiseMan as `ba1cef883`. The bench was built with the crate's
release profile twice: as shipped, and with `--features baseline`. The window is `jak-arena`: from the state at frame
9,000, timed over 9,060 to 9,360, `--defaults`, desktop only (Ryzen 7 7700X), each run in its own scope at CPU weight
100, one-minute load 0.5 to 3.7.

- **The sampler**: §51.6's copy of S32's ptrace sampler (§43.4), 4 ms on every thread from the progress line of frame
  9,100 to the end, with the recompiler's perf map. The emulation thread's samples are classed by their innermost
  Sapphire frame, with waits apart (a futex, a park or a system call, named by the caller). Each class's share is
  multiplied by the run's own mean, since the sampled run is its own timing round.
- **Two scratch changes for this section, not committed**:
  - the sampler names the baseline's code region, hot and cold halves apart, from a line `emusen-jit` writes under
    `EMUSEN_NATIVE_REGION=1`;
  - for a sample inside a `sapphire::float` function, the sampler scans 24 stack words for the first return address
    outside those functions and takes it as the caller. A frame without a frame pointer makes this a heuristic, which
    the result below checks.
- **One scratch counter, not committed**: `EMUSEN_SAPPHIRE_PREFIX_WHY=1` re-runs the convergence test at the last step
  the prefix tried and names the first condition of §15.2 that failed.

| Run | Build | Settings | Mean ms a field | Picture, sound digests |
| --- | --- | --- | --- | --- |
| A | shipped | sampled | **765.4** | `611B58F7CFE67E8B`, `BCA8F00F4743A955` |
| B | baseline feature, not dispatched | sampled | 767.8 | the same |
| C | baseline feature, dispatched | sampled, `PREFIX_WHY=1` | **814.2** | the same |
| R | baseline feature, dispatched | `PREFIX_WHY=1`, `FORMS_BYTES=1`, unsampled | 819.2 | the same |
| WA | shipped | `EMUSEN_SAPPHIRE_WAIT=1`, unsampled | 1,915.1 (median 1,243, max 14,635) | the same |
| WC | baseline feature, dispatched | `EMUSEN_SAPPHIRE_WAIT=1`, unsampled | 4,766.1 (median 3,556, max 16,823) | the same |

A and B agree within 0.3 %, so the feature compiled in costs nothing where it is not dispatched; B stands for §15.4's
Cranelift tier (754.1 and 760.9 there). C began while B's sampler was still naming its samples (one core busy), and R
while C's was. The second timing rounds are §15.4's.

### 60.2 Where the field goes, by unit (measured, run A)

The emulation thread carries all the units below; the field is bound by it. Run A's emulation thread had 40,419
samples.

| Unit | ms a field | Share | Of which |
| --- | --- | --- | --- |
| **VU0 and VU1**, all of it | **264.9** | **34.6 %** | compiled microprograms 63.4; floats in software 39.5 (`float.rs`, called by the VUs, §60.3); the call path and the operations' wrappers around compiled code (`vu/pipe.rs` `run_to`, `drain`, `due`; `vu/ops.rs` `lanes`, `mac_bits`; `vu/mod.rs` `vf`, `settle`) 94.9; the recompiler's dispatcher and helpers (`vu/jit` `pending` 26.2, `run`, `h_upper`, `covers`, `h_push`) 67.1. No pair is interpreted: VU0 compiles 554,650 pairs a field and VU1 2,239,138. |
| **EE**, all of it but idle skipping | **214.4** | **28.0 %** | interpreter (fetch 14.5, `entry`, TLB, translation, decode, ops) 93.1; dispatcher and replay (`blocks.rs` `block` 47.0) 58.3; timing model (`plan`, `commit`, `partner_of`) 30.2; compiled blocks 16.9; the compiled code's helpers in `ee/jit` (`held_words`, `block_entry`) 15.9; COP1's floats 0.1 |
| **Board: scheduling and synchronisation** (`board.rs`, `machine.rs`) | **62.1** | 8.1 % | `run_gif` 16.1, `reschedule` 15.7, `run_vifs` 9.5, `run_to_held` 8.3 |
| **GIF** | **57.0** | 7.4 % | `choose` 23.4, `run_to` 14.9, `take_kick` 5.7; packing about 2 |
| **VIF** | **55.2** | 7.2 % | `run_to` 14.8, `step` 14.4, `take_irq` 6.0, `has_word` 3.1 |
| EE timers, INTC, DMAC, PCRTC | 43.6 | 5.7 % | `timer::period` 12.3, `wants_h` 4.9, `dmac::next_work` 4.1 |
| GS on the emulation thread | 35.5 | 4.6 % | publishing jobs and the local memory's reads 31.1; **waiting for the workers 4.4** (`publish`) |
| Idle skipping's own work | 11.6 | 1.5 % | `prove`, `mark`, `idle_admits`; it passes 45.0 % of the EE's cycles (2.21 M a field) |
| IOP, SIF and SIO2, SPU2, CDVD, IPU | 10.8 | 1.4 % | IOP 8.2, SIF and SIO2 1.1, SPU2 1.1, CDVD under 0.1, IPU 0.4 |
| Other (no Sapphire frame) | 10.5 | 1.4 % | 128-bit division `__udivti3` 3.9, the blocks' hash map 2.5 |

**VU0 against VU1** [E]: the sampler cannot tell their compiled code or call path apart. By pairs (554,653 and 2,239,138
a field), VU1 is about 80 % of the 264.9 ms (≈ 212) and VU0 about 20 % (≈ 53).

**The other threads.**

- **EE compiler**: busy in 100 % of its samples for the whole window, Cranelift 48 % and regalloc2 49 % of them (7 %
  in a `memset` clearing regalloc2's tables). Its 15,013 blocks over the run come to about 15 ms of compile a block.
- **GS workers**: by the bench's own count, busy 66.1, 64.0, 62.7 and 46.9 ms a field and stalled 0.8 to 6.0. The
  sampler sees them spinning (`workers.rs:567`) or parked the rest of the time. The emulation thread's waits on them
  are the 4.4 ms above, plus CLUT loads (973 ms over the run, about 3 ms a field) and display waits (0.4).
- **VU compiler**: idle, with nothing new to compile.

**The top five and their nature.**

1. **The EE**, 214 ms. Compute 140 ms (the interpreter, the timing model and the compiled blocks). Overhead 74 ms: the
   dispatcher, at 47 ms for about 0.51 M entries and 2.1 M interpreted steps a field, and the compiled code's helpers.
2. **VU1**, ≈ 212 ms. Compute ≈ 82 ms (its compiled code and floats). Call overhead ≈ 130 ms: the crossings between
   compiled microcode and Rust, which is synchronisation of the pipeline model with each pair, not arithmetic.
3. **The board**, 62 ms. Synchronisation: deciding which unit runs next, and rescheduling.
4. **The GIF**, 57 ms. Synchronisation: `choose` and `run_to` asked at the board's every step. Packing, the transfer
   itself, is about 2 ms.
5. **The VIF**, 55 ms. About a quarter compute (`step`, the unpacking, 14.4 ms); the rest is synchronisation (`run_to`,
   `take_irq`, `has_word`, `idle`).

VU0 (≈ 53 ms) and the timers' 44 ms come next, both mostly call overhead and polls. Waiting proper is small: the GS
costs the field under 8 ms of waits, and its workers are below 9 % busy.

### 60.3 The floats belong to the VUs (measured, run C)

In run C, float functions carried a caller from the stack scan. Of the samples in `float.rs`'s functions, 38.5 ms
named a `sapphire::vu` caller or compiled code with no map entry (the VUs' microprograms). 0.3 ms named the EE's FPU
or its blocks, and 0.1 ms named neither. Run A's float samples, which carry no caller, are therefore the VUs' 39.5 ms
in the table above, not the EE's. §51.6's row "COP1's floats in software" on this window should be read as mostly VU
work since the VUs were compiled (S40 on).

### 60.4 The baseline's loss (measured, runs B and C; the cause inferred)

C is 46.4 ms a field slower than B (6.0 %; §15.4 had 4.4 %). The EE accounts for all of it: 260.7 ms against 212.4,
while the VUs, the board, the GIF, the VIF and the GS differ by under 3 ms each.

| ms a field | B (Cranelift) | C (baseline) | Change |
| --- | --- | --- | --- |
| EE interpreter | 92.3 | 114.1 | +21.8 |
| EE timing model | 29.3 | 42.1 | +12.8 |
| EE dispatcher and replay | 58.7 | 67.1 | +8.4 |
| the baseline's exit helpers (`prefix_of` 3.3), prefix (scratch counter 0.6 of it) and `commit_static` | — | 11.9 | +11.9 |
| EE compiled code (Cranelift's, then the baseline's hot 3.0 and cold 0.8) | 16.8 | 7.4 | −9.4 |
| the compiled code's helpers in `ee/jit` | 15.3 | 18.1 | +2.8 |

**What the tailored tier does more of** is interpretation. Over the run:

- compiled steps 326.9 M against 466.7 M (−30 %), and steps interpreted 919.5 M against 767.6 M (+20 %);
- entries 118.9 M against 167.7 M;
- exits at an event 31.7 M against 13.7 M, 27 % of entries against 8 %;
- rollbacks 70,828 against 53,899, which are few;
- the translation cache's generations 12 against 6.

There is no fallback to the Cranelift function, since a covered block always enters the baseline's code. The
baseline's own new work, its prefix and exit helpers, is about 12 ms of the 46.

**Why fewer steps run compiled** (inferred from the counters; the budget's flushes are not printed by the bench):

- **The code budget.** The EE's code budget is 256 MiB (`blocks::BUDGET`), and the compiler's counters restart when the
  budget drops the module. A covered block's bytes are its Cranelift function, the tailored code and the prefix's
  table: C's compiler ended its run at 201 MB for 7,609 blocks (26 KB a block), against B's 166 MB for 13,664 (12 KB).
  With the compile thread saturated in both runs, and the tailored lowering under 1 % of its samples, C cannot have
  compiled half as many blocks in the same time. The counts fit one drop of every block in mid-window, followed by
  recompiling from cold.
- **R shows the drop directly**: 13,434 blocks lowered by the baseline across the run (a static counter), against the
  final compiler's 1,652.
- **WC is the same effect amplified.** Each block's code is waited for, so every recompile after a drop is paid on the
  emulation thread: 4,766 ms a field and 74 generations.

**Every block still pays a full Cranelift compile.** The tailored lowering costs 43.0 µs a block (R's own count), but
the runtime compiles each block's Cranelift function first and the tailored code beside it. In C, Cranelift and
regalloc2 are 94 % of the compile thread and the tailored lowering is under 1 %. So the baseline does not yet bring
its compile speed to the field: it adds its bytes to the Cranelift function's. That costs the cache its warmth through
the budget, and compiles no faster.

### 60.5 Cold against waited compiles (measured)

| ms a field | Cold (compiled in the background) | `WAIT=1` (each block's code waited for) |
| --- | --- | --- |
| Cranelift's tier | 765.4 (A); compiled share 40.1 %; 15,013 blocks | 1,915.1 (WA); 51.8 %; 15,145 blocks |
| the baseline | 814.2 (C); 26.7 %; 7,609 since the last drop | 4,766.1 (WC); 45.1 %; 6,356 since the last drop |

`WAIT=1` does not warm the window. The arena keeps meeting new blocks through all 300 fields: WA's progress lines show
100 fields taking 225 s, then 185 s. WA's extra 1,150 ms a field is about 345 s of waits for some 15,000 compiles,
about 23 ms a block on Cranelift's tier.

With every block's code ready when it is entered, the compiled share rises only from 40.1 % to 51.8 %. The other 48 %
of the EE's steps run interpreted for reasons of the dispatcher's own (the threshold, refused shapes, steps between
blocks), not the queue.

So a warm cache would remove at most the queue's 11.7 points of steps:

- about 0.41 M steps a field move from the interpreter's ≈ 58 ns a step (the interpreter and the timing model over
  2.11 M steps) to compiled code's ≈ 23 ns, plus the dispatcher;
- **about 14 ms a field** [E], which is under 2 %.

### 60.6 Why the prefix's entries do not converge (measured, run R)

R ran the prefix 13.70 M times on entries not at rest. 5.03 M converged after one step, 0.83 M later, and **7.73 M never
did** (57 %). The counter counts prefix runs, not entries, and an entry with a commit runs the prefix twice, so these
are 0.4 % over the bench's 7.70 M entries. Each never-converging run is named by the first condition of §15.2 that
failed at the last step it tested, and by why testing stopped there:

| Condition failed at the last test | Exit came first | `kmax` at the branch | `PMAX` (8) | All |
| --- | --- | --- | --- | --- |
| (1) the pair slot, open against closed | 4,448,794 | 3,078,129 | 35,141 | **7,562,064 (97.8 %)** |
| (4) a GPR the rest reads, still in flight from before the entry | 11,360 | 141,016 | 17,240 | 169,616 (2.2 %) |
| (3) the floor | 277 | 2 | 0 | 279 |
| (1) the pending group's end | 10 | 0 | 0 | 10 |
| **All** | **4,460,441 (57.7 %)** | **3,219,147 (41.6 %)** | **52,381 (0.7 %)** | 7,731,969 |

At the first test (*k* = 0) of every entry not at rest, the reasons were: the slot 12.91 M, the floor 0.48 M, a GPR in
flight 0.31 M, the group end 163. The never-converging runs ended at exit steps 2 (2.33 M), 3 (2.69 M), 4 (1.32 M),
5 (0.70 M), 7 (0.39 M) and others (0.31 M).

**The pairing parity is the reason**, as §15.4 named it among the candidates. An entry that arrives with the pair slot
open (its predecessor issued alone and the slot stayed open) never meets the static form's slot, which is closed at
rest, before it leaves. 58 % leave first: their blocks are 2 to 5 steps. 42 % reach the branch's `kmax` still out of
step. Long-latency results (a GPR in flight) are 2 %, and the multipliers, HI/LO and SA never decide it here.

The prefix itself costs little: `prefix_of`, `prefix::run`, `model` and `snap` together are about 3 ms a field in C.
So the parity matters for the bytes and the static share (§15.4), not for the field's time.

### 60.7 The levers the profile points at (estimated, nothing built)

Each is ranked by the ms a field it could remove on run A's 765.4.

1. **The units at their events: the board, the GIF, the VIF, and the timers, DMAC and PCRTC** (218 ms together). Most
   of it is the per-step questions that find nothing to do (`choose`, `run_to`, `reschedule`, `take_irq`, `has_word`,
   `period`, `next_work`). Exact if each unit's next event is computed exactly and the board skips to it, as
   `Sapphire_Scheduler.md` §54 began and `Sapphire_Redesign.md`'s R-stages plan. §58's synced N = 64 removed 254 ms,
   but changed the digests. An exact event form keeping the VIF's unpacking, the GIF's packing and the board's real
   work (about 50 ms) could remove **≈ 120 to 170 ms** [E].
2. **VU1's and VU0's call path** (162 ms of crossings around 103 ms of compute). The pipeline's bookkeeping (`pending`,
   `due`, `drain`, `stalled_until`) and the operations' lane wrappers, done in the compiled microprogram, or called once
   a pair rather than once an operation. It is exact if the same pipeline model runs with the same functions, and
   `Sapphire_Speed.md` §57's due-cycle answers are the start. **≈ 80 to 120 ms** [E].
3. **The EE's dispatch and its compiles** (the dispatcher's 58 ms, and the interpretation behind the queue).
   - Cheaper per-step dispatch in `blocks::block` (47 ms for every step not run compiled) could remove **≈ 20 to
     35 ms** [E], exact.
   - A warm cache is worth ≈ 14 ms on the shipped tier (§60.5).
   - For the baseline, compiling a covered block without its Cranelift function, or not counting that function against
     the budget, would remove the 46 ms §60.4 found and make 43 µs compiles real. That is exact, since it is the code
     E1 already proves.

Not levers on this window: the GS (under 8 ms of waits, workers under 9 % busy), the IOP's side (11 ms), idle
skipping's own work (12 ms), and the prefix (3 ms).

### 60.8 Named limits

- **The VU0 and VU1 split is by pairs** [E]; the sampler sees one emulation thread.
- **The float callers come from a stack scan**, a heuristic. Its result (99 % VU) agrees with the inline chains, which name
  `ee/fpu.rs` around only 0.1 ms of float samples in run A.
- **The code budget's drops are inferred**, not counted. The bench prints the compiler's counters since the last drop
  and no drop count; R's static counter is the direct evidence.
- **One sampled run of each configuration.** The shares are each run's own, and the timing rounds' spread is §15.4's
  (about ±4 ms).
- **Idle skipping's refusals are not counted** by the bench; only its passes (45.0 % of the EE's cycles) and its own
  work (11.6 ms).

### 60.9 Questions this step leaves

1. **[decision] The baseline's compile path.**
   - *The question*: whether a covered block skips its Cranelift function (or keeps it out of the code budget) before
     E2.
   - *Recommended*: decide it before E2. Today every covered block pays Cranelift's ≈ 15 to 23 ms and twice the bytes,
     which drops the cache mid-window and is the whole of E1's loss on the arena.
2. **[decision] The next unit after the EE.**
   - *The question*: which unit's work comes next.
   - *Recommended*: the units at their events (lever 1), then VU1's call path (lever 2). Together they are over half
     the field, against the EE's compiled code at 2 %.
3. **[routine] A drop counter.**
   - *Recommended*: print `Jit::flushes` for the EE on the bench's `blocks` line, so a budget drop is counted rather than
     inferred.

### 60.10 What was read for this step

This page's §51.6, §58.3 and §58.4; `EmuSen_Recompiler.md` §15.2 to §15.4; `Sapphire_Native.md` §43.4; and Sapphire's
`blocks.rs`, `blocks/baseline.rs`, `ee/jit/baseline/prefix.rs`, `ee/timing.rs` (`converges`), `ee/jit/mod.rs`,
`ee/jit/forms.rs` and `emusen-jit`'s `lib.rs` and `native.rs`. No emulator source was read and no firmware was listed.
The BIOS image was given only as the bench's argument.

## 61. The units at their events: the timers' cached event, and the GIF's and the VIF's exact skips (2026-10-29)

*Decided 2026-10-29 by the tester: build §60.7's lever 1, exact and legal in both faces (`EmuSen_TwoFaces.md` §3: an
optimisation that keeps both results and timing identical serves both faces). Each step's saving was estimated before
it was built, from §60.2's run A (arena, 765.4 ms a field), and the stage stops at a step that saves less than half
its estimate or cannot be proven exact.*

### 61.1 The estimates, written before building

| Step | What | Run A's rows it acts on | Estimate, ms a field (arena) |
| --- | --- | --- | --- |
| 1 | The timers' next event and H-blank wish cached in the board, recomputed at each change to a timer; `run_to`'s first `reschedule` of its slow path, whose result the second overwrites unread, removed | `timer::period` 12.3, `wants_h` 4.9, about half of `reschedule`'s 15.7 | **≈ 20 to 30** [E] |
| 2 | The GIF's exact skip (`Sapphire_Redesign.md` §4.4): a `run_to` before its resume with nothing arrived returns at once; VU1's kicks taken only after a kick; a VIF whose last run ended on a stall or for want of a word not stepped until something reaches it | GIF `choose` 23.4, `run_to` 14.9, `take_kick` 5.7; board `run_gif` 16.1 in part; VIF `run_to`, `step`, `has_word` in part | **≈ 45 to 65** [E] |

Together ≈ 65 to 95 ms, below §60.7's 120 to 170: the VIF's waits on VU1 (MSCAL, FLUSH and MPG behind a running
microprogram) are not skipped, since the VIF's catch-up after the VU's end runs against FIFO contents and GIF state
whose order with the other units a skip would change (§61.3); and the board's own per-step catch-up of the VUs, the IOP
and the lines stays.

### 61.2 Step 1: the timers' next event, cached

**Before.** `Board::reschedule` ran up to three times in each `run_to` that took the slow path (a unit busy, which on
the arena is most EE steps, since VU1 runs most of the field): once after each event, once after the VIFs were brought
to the step's time, and once at the end. Each run asked all four timers for `Timer::next_event`, whose
`(synced / p + k) × p` divides by the clock's period, chosen at run time, and asked each timer's `wants_h`.

**After.** The board keeps `timers_next`, the least of the four timers' next events, and `timers_h`, whether any wants
the H-blank's edges, and recomputes both in `timers_changed`. `reschedule` reads the two fields. The slow path's middle
`reschedule` is gone.

**Why it is exact.**

1. *The cache is a function of the timers alone.* `Timer::next_event` reads `mode`, `count`, `comp`, `synced` and
   `gate_high` (through `period`, `counting` and `settled`) and nothing else; `wants_h` reads `mode`. So the cached values
   are right as long as no timer changed since `timers_changed` ran.
2. *Every change to a timer is followed by `timers_changed` before the next `reschedule`.* A timer changes only through
   `sync`, `edges`, `write` and `read_state` (`read` takes `&mut self` and changes nothing). The board calls them at four
   places: the event loop's `sync_timer` for each timer and `edges` for each, followed by one `timers_changed` before the
   loop's `reschedule`; `register_value`'s `sync_timer` before a timer register is read; `write_register`'s `write`; and
   `System::read_state`. `Board::new` fills the cache before its first `reschedule`. The timers' fields are public, but
   only a test writes one directly (`state.rs`'s round trip), and it saves the state without running the board.
3. *The middle `reschedule` was dead.* Between it and the slow path's last `reschedule` the board calls
   `Gif::path1_free`, `Path1::set_free`, `Vpu::catch_up`, `take_interrupts`, the INTC's `raise` and `run_gif`. None of them
   reads `Board::next`, and `reschedule` writes only `next` and the PCRTC's cache, which is a pure function of the time
   and the H-blank wish (§35.7). The last `reschedule` overwrites `next`, so the state's `Scheduler` block is unchanged.
4. A `debug_assert` in `reschedule` compares the cache with the four timers' own answers on every call, so a test run
   with debug assertions catches any change to a timer that misses `timers_changed`.

The step changes no answer, only when it is computed, so it serves both faces.

### 61.3 Step 2: the GIF's and the VIF's exact skips

**Before.** Every slow-path `run_to` of the board brought the GIF to its time twice (`run_gif` at the head of
`run_vifs` and again after the VUs), and both VIFs once. `Gif::run_to` arbitrated afresh each time (`choose` with the
step's time, then again with no limit for `resume`), `run_gif` asked VU1's PATH1 port for kicks through its trait
object, and a VIF waiting for data or stalled stepped into its command to find the same missing word.

**After.**

- *The GIF* keeps `still`, the BUSDIR level its last `run_to` saw, set when a run begins and cleared by everything that
  reaches the GIF from outside: `kick`, `path2` (through `request2`), `put`, `request`, `mask_path3`, `write` (and so
  `reset`) and `read_state`. A `run_to(t)` while `still` matches BUSDIR and `t` is before `resume` (or `resume` is none)
  returns at once.
- *VU1's kicks*: the PATH1 link carries `kicked`, set by both XGKICK paths (the interpreter's `pipe` and the
  recompiler's helper) and by loading a state; `run_gif` drains the port only when it is set.
- *A VIF* keeps `dry`, set when its `run_to` stops on a step that returned none outside a wait (`Cmd::Wait`) and outside a
  DIRECT refused by the GIF (VGW), and cleared by `write`, `request`, `put`, `get` and `read_state`, the calls through
  which anything reaches a VIF. A `run_to(t)` while `dry` only moves the VIF's clock to `t`, as the stopped run does.

**Why it is exact.**

1. *The GIF.* `run_to` reads only its own fields, BUSDIR, the time `t` and, once a path holds the GIF, VU1's memory for
   PATH1's next qword. Each way it returns leaves no step that begins by `t`, and `resume` names the earliest one there
   could be: (a) paused or reversed, `resume` none; (b) no path granted by `t`, `resume` the grant's time from `choose`
   with no limit, and `choose(t')` is none for every `t'` before it, since it grants at the later of the GIF's time and
   the earliest request; (c) PATH1 holding the GIF with its next qword after `t`, `resume` that time; (d) PATH3 holding it
   with its next qword arrived after `t`, `resume` that time, the slice check having found no request to yield to;
   (e) PATH3 holding it with the FIFO empty and its packet open or already output (so D-350's release does not apply), or PATH2 holding it,
   `resume` none. With nothing from outside since, a run at `t'` before `resume` meets the same state and returns the
   same way, leaving `resume` as it was, which is the field the state holds, and changing nothing else; VU1's memory is
   read only to step PATH1, which by (c) does not happen before `resume`. Every call that changes the GIF's fields from
   outside clears `still`; its fields are private.
2. *The kicks.* `Path1Port::kick` is called only by the two XGKICK paths, and its queue is filled otherwise only by
   `read_state`; both set `kicked`. `take_kick` with `kicked` clear would return none.
3. *A VIF.* A step returns none outside a wait in four cases: stalled (STAT's stall bits, changed only by `write`'s
   FBRST and a state's load); the stream empty while a command or a fetch needs a word (the FIFO and the qword in hand,
   changed only by `put`, `get`, `request` in the reversed direction, `write`'s reset and the load); a fetch that stalls or
   stops (it sets a stall bit, case one); and a refused DIRECT, which depends on the GIF and is excluded. `Unpack::next`
   returns `Word` without changing the unpack or the fill registers (`vif/unpack.rs`), and the other commands restore
   their state before returning none, so a repeated run takes the same path to the same none. `run_to`'s own stop sets
   the clock to the later of it and `t`, the move the skip makes. The VIF's public fields (MARK, CYCLE, the fill
   registers and the like) change no stalled or dry decision; nothing outside `vif` and the board's ports writes them.

**Not skipped, and why.** A VIF waiting for VU1 (MSCAL, MSCALF, MSCNT, FLUSH, FLUSHE, FLUSHA, MPG) checks the VU at every
board step, and that check also brings VU1 to the step's time. Skipping it until VU1's end is not exact by this argument:
the VIF resumes at the end's cycle and runs on to the step's time against FIFO contents and GIF state as they stand at
that step, so seeing the end one step later would let the DMAC's and the GIF's work of that step reach the VIF first.
That needs the units on one timeline (`Sapphire_Redesign.md` §4.4), not a cache.

### 61.4 Results (measured 2026-10-29)

The bench as §60.1's, `--defaults`, desktop only. Timed runs at CPU weight 100 with no other build or bench running;
the one-minute load at each run's start is given (about 3 is the bench's own GS workers and compiler).

| Window | Base (5ed26aaf8) | Step 1 | Steps 1 and 2 | Digests (picture, sound, state) |
| --- | --- | --- | --- | --- |
| short (6,060 to 6,120) | 270.4 | 257.2 | 226.2 (weight 20) | `0F8671F57AB63C25`, `D5F15509CCDD1709`, `818361BFE5D2DBF9` in all three |
| dock (6,060 to 6,360) | 231.8 (load 1.2) | not valid (another bench ran) | 203.7 (weight 20, load 2.7) | `830E3FEFEE1BE6B1`, `2C225C73DB4D3F9D`, `9CF1BED1AA64EAEE` in all three |
| arena (9,060 to 9,360) | 771.7 (load 3.3); **763.2** (load 3.2) | 757.8 (load 1.2, another bench from its middle) | **681.0** (load 2.6) | `611B58F7CFE67E8B`, `BCA8F00F4743A955`, `8B710CA05FDDF3A0` in all three |

- **Steps 1 and 2 together: −82 ms a field on the arena** (763.2 → 681.0, −10.8 %, one interleaved round), against
  65 to 95 estimated. The dock's weight-20 run, −28 ms, points the same way.
- **Step 1 alone** measured about −14 ms in one run that overlapped another bench: inside the noise, so neither
  confirmed nor refused against its 20 to 30. Step 2's share is therefore about 68 ms, against 45 to 65 estimated.
- **Exactness:** every digest equals the base on all three windows; §61.2 and §61.3 give the argument. The crate's
  tests for the units touched ran in release with debug assertions on (so §61.2's check of the timers' cache ran on
  every `reschedule`): `blocks` 28, `gif` 16, `scheduler` 5, `state` 11 and `vif` 15, all passed.
- **Not done**, the stage being stopped on 2026-10-29 for a structural redesign of the byte face: the seeded faults, a
  second timing round, and a profile of what remains of §60.2's 218 ms. The lever's remainder by §61.3: the VIF's
  waits on VU1 and the board's per-step catch-up need the units on one timeline (`Sapphire_Redesign.md` §4.4).

## 62. The board in rounds, and VU1 to its end (2026-10-29)

*Decided 2026-10-29 by the tester: for the PlayStation 2 and later, accuracy is to the player (games look, sound and play
the same); the cycle-ordered build is kept as a debugging tool and no longer gates speed work; per-game fixes stay
banned and results stay deterministic. Sapphire is no longer clean-room, and this section's approaches were taken from
reading PCSX2, Play! and DobieStation (§62.9), written anew in Rust on Sapphire's own units. The target is the handheld
at a mean of 13.4 ms a field with the 95th percentile under 16.7.*

### 62.1 The plan, written before building

The arena's field is 681 ms (§61.4). §60.2 put about 218 ms in the units polling and scheduling one another (the board,
the GIF, the VIF, the timers; §61 took about 82 of it) and about 162 ms in VU1's and VU0's call path. Both have one
cause: the board is brought to the EE's time after every EE instruction (§54), so each unit is asked about 4 million
times a field whether it has anything to do, and VU1 runs 1.2 pairs a call.

1. **Rounds** (from DobieStation's run loop, which runs the EE for at most 32 cycles and then every unit for the same
   time, and PCSX2's event test, which brings the units along only when the EE reaches its next event, the IOP in
   arrears). `Board::run_to(t)` brings the board along only when an event falls by `t` (the board's own `next`, so the
   DMAC, the timers, the PCRTC's edges and the FIFOs' wakes keep their cycles) or when `t` reaches the round's end,
   `ROUND` EE cycles after the last catch-up; otherwise it records the time as owed and returns. **Every EE observation
   of a unit first brings the board to the owed time**: the unit registers at 1000_0000h, the VIF, GIF and IPU FIFOs,
   VU memory, COP2, the GS's privileged registers, IOP RAM and the drive's registers. So the EE never reads a unit at a
   stale time (§58.3's plain build broke Jak at N = 64 for want of this; its synced build did not). What changes is
   when the units no observation reaches catch up, and that an interrupt line rising between events is seen up to a
   round late: timing only. A compiled block's replay takes a whole run of times inside a round in one call.
   `ROUND = 0` (`EMUSEN_SAPPHIRE_ROUND=0`) is the cycle-ordered build, the debugging tool; `System::new` keeps it, so
   the unit tests stay on it, and `Machine` takes `ROUND`.
2. **VU1's end as the board's event** (from PCSX2's VU1 micro mode, where `vu1ExecMicro` runs a microprogram to its end
   at once under the default "instant VU1", and VIF1's MSCAL waits find it done). §58.3's synced build stalled the game
   from N = 512: a VIF waiting on its VU (MSCAL, FLUSH) resumed only at the next catch-up, so each microprogram cost up
   to a round. With VU1's end known, a waiting VIF resumes at that cycle as an event, and the round can grow.

Estimates against the 681: step 1 removes most of the per-instruction catch-ups, as §58.3's synced N = 64 removed
227 ms of 715 then: **≈ 150 to 250 ms** [E]. Step 2 takes most of what remains of the call path once rounds grow:
**≈ 50 to 100 ms** [E]. Not reached by either: the EE's interpreter and dispatcher (§60.2's 214 ms) and the VUs' compute.

### 62.2 Step 1: rounds (measured 2026-10-29)

**What was built.** `Board` keeps `owed`, the latest time `run_to` was asked for, and `round_end`; neither is state, and
a state is written with nothing owed (`Machine::advance` settles the board at each frame's end). `run_to_held` returns
`Ran::Owed` without touching a unit while `t` is before `round_end`, no event is due by `t` and the IOP's side has no
wake pending; otherwise `run_now` takes the quick path or the whole one and starts the next round. Every branch of
`Bus::read` and `Bus::write` that reaches a unit, and `Bus::vpu`, first calls `observe`, which brings the units the tail
brings (the GIF, the VIFs, the VUs, the IOP) to the owed time; the IOP's wake of the DMAC, which needs the scratchpad,
is left pending, so the next `run_to` is whole. `run_through` takes a replay whose times all fall inside the round in one
call: the lines stand, so `Times::quiet` is shown them once. Idle skipping (§47) settles the board before it tries a
loop, proves a turn with the board settled after each step, and replays a busy turn whole, so that it passes turns as
they would run.

`pure_register` answers none while time is owed, so a proof never takes a register's value from a board behind the EE.
The debugger (`run_frame_debug`) steps the cycle-ordered board, a round of 0, and restores the round after.

**Exactness at a round of 0.** With `EMUSEN_SAPPHIRE_ROUND=0` the short window's digests are §61.4's exactly
(`0F8671F57AB63C25`, `D5F15509CCDD1709`, `818361BFE5D2DBF9`), so the restructuring itself changes nothing.

| Window | Base (8309ff51b) | Rounds of 64 | Digests at 64 (picture, sound, state) |
| --- | --- | --- | --- |
| short | 226.3 (round 0, load 2.6) | **183.3**, 182.9 (load 2.8) | `EF589A2EEC6FE039`, `D5F15509CCDD1709`, `80EBEB2B95A6E71C` in both runs |
| arena | 681.0 (§61.4) | **536.9** (load 2.8; p95 687.0) | `F15E7C2A86349885`, `BCA8F00F4743A955`, `233A3EE02974BF95` |

- **−144 ms a field on the arena** (−21 %), against 150 to 250 estimated: at the estimate's bottom. The short window
  −43 ms (−19 %).
- **The sound digests equal the base's** on both windows. The pictures are the same scenes a few frames out of phase:
  at fields 9,060, 9,120, 9,180, 9,240, 9,300 and 9,360 of the arena, 10.2, 19.4, 27.9, 10.7, 13.2 and 0.7 % of the
  pixels differ, by a mean of 0.04 to 2.4 levels of 255; looked at side by side, the differences are the particles and
  the characters' poses a frame or two apart, the geometry and textures right.
- **Two runs give the same digests** (the short window above).
- **ps2autotests**: 52 of 55 EE and 23 of 23 IOP, the same three failures (`sqrt`, `efu`, `intstall`, §36.5), and every
  program's text identical to its text at a round of 0. No timing test's text changed.
- **Longer rounds do not pay** on the short window: 186.0 at 256 and 187.4 at 1,024 (load 2.1 to 3.6), against 183.3
  at 64; at 4,096 more than 16 XGKICKs wait at the window's end, beyond the state's slots, and the state cannot be
  written. So the board's remaining cost is not the rounds' ends but the observations (§62.3).

### 62.3 Where the rounds' catch-ups come from, and VU1 to its end refused (measured 2026-10-29)

**The census.** A scratch build (not committed) counted, on the short window from the state at 6,000 to field 6,120,
each `observe` that found time owed, by the access that made it, and each whole `run_to`, by whether an event was due.
A field: **142,000 from COP2** (`Bus::vpu`), 2,600 reads and 1,400 writes of the unit registers at 1000_0000h, 160 from the VIFs' registers
and FIFOs, a few from the GIF and the GS; **75,000 whole runs at an event** and **35,000 at a round's end**. So after
step 1 the EE's COP2 instructions bring the board whole about twice as often as the events and rounds together, which
is why longer rounds do not pay (§62.2).

**VU1 to its end, as planned (§62.1 item 2), refused.** A VIF waiting for its VU (`Vif::wait`) ran the VU on to its end
first (`Vpu::ready_ahead`, `Micro::finish`), the GIF's PATH3 slice check seeing only the requests made by its run's
time. On the short window it reached field 6,100 in 47 to 52 s from the load, against 19 s for step 1 alone, at rounds
of 64, 256 and 1,024, and at each the window's end held more XGKICKs than the state's 16 slots, so the state could not
be written. A microprogram run to its end hands the GIF every packet it kicks at once, and the VIF's next waits find the
GIF still holding them; and `finish`'s bound (2^24 pairs) lets a microprogram that waits on the VIF run on. It was not
pursued: the census shows the rounds' ends are not where the board's time goes.

**Step 2, COP2 to VU0 alone.** The COP2 instructions that reach VU0 alone, QMFC2, QMTC2, LQC2, SQC2, CFC2 and CTC2 but
for FBRST, VPU-STAT and CMSAR1, and every VU operation issued as a macro instruction, ask the bus for `vpu0` instead of
`vpu`. On the board, with a round above 0, `vpu0` brings only VIF0 (which writes VU0's memories and starts it) and VU0 to
the owed time, waking the DMAC and raising VIF0's interrupt as the tail would, and leaves the rest to the round. BC2F,
BC2T and the three control registers above reach VU1 and still bring the board whole. On the cycle-ordered board `vpu0`
is `vpu`.

| Window | Step 1 | Step 2 | Digests at step 2 (picture, sound, state) |
| --- | --- | --- | --- |
| short | 183.3, 182.9 | **174.8** (load 0.9) | `0FEAA87AB0100A65`, `D5F15509CCDD1709`, `9929F57C6DC08CBC` |
| arena | 536.9 | **500.4** (load 1.0; p95 649.8) | `D324EC9EE11A8DEC`, `BCA8F00F4743A955`, `4DDB6D74ECC37C7B` |

- **−37 ms a field on the arena** (−7 %), and 681 → 500 for the stage (−27 %). No estimate was written for this step
  before it was built, since it replaced the refused one; it is measured, not judged against one.
- **The sound digests still equal the base's.** The arena's pictures at the six fields differ from the base's by 10.6,
  20.5, 27.9, 10.7, 12.7 and 0.6 % of the pixels, mean 0.05 to 2.4 levels: the same scenes, a few frames out of phase,
  as step 1's.

### 62.4 What is left (measured 2026-10-29, the arena under the sampler)

§43.4's sampler (`rtsample.py`, 4 ms, the emulation thread, from the state's load plus 60 s for 150 s) on step 2's
build, 29,741 samples, the run's mean 507.7 ms under the sampler. Shares of the emulation thread:

| Unit | Share | ms a field [E, share × 500] |
| --- | --- | --- |
| EE: interpreter, timing model, TLB, fetch, FPU floats in software | 32.7 % | 164 |
| EE dispatcher (`blocks.rs`) and compiled code with no symbol (EE and VU, `anon`) | 20.8 % | 104 |
| VU micro mode: the recompiler's helpers 6.4, `ops.rs` 5.9, `mod.rs` 2.8, `pipe.rs` 2.8, floats 0.7 | 18.6 % | 93 |
| GS on the emulation thread | 8.5 % | 42 |
| VIF 4.6, DMAC 2.6, GIF 2.0, PCRTC 1.1, timers and INTC 0.8 | 11.1 % | 56 |
| Scheduler and synchronisation (`board.rs`, `machine.rs`) | 3.7 % | 19 |
| IOP, SPU2, SIF, IPU | 1.6 % | 8 |

§60.2's board, GIF, VIF, timers, DMAC and PCRTC came to 218 ms of 765 (28 %); they are now about 75 ms (15 %). What leads
is the EE (about 270 ms with its dispatcher) and VU micro mode's call path (the helpers and wrappers, about 80 of its 93).

### 62.5 Checks, and what this stage does not do

- **ps2autotests** on step 2: 52 of 55 EE and 23 of 23 IOP, every program's text identical to its text at a round of 0;
  no timing test's text changed.
- **Determinism**: two runs of step 2's build give the same digests on the short window (174.8 and 175.6 ms).
- **The dock** (6,060 to 6,360), the same build: at a round of 0, 204.5 ms (load 1.9) with §61.4's digests exactly
  (`830E3FEFEE1BE6B1`, `2C225C73DB4D3F9D`, `9CF1BED1AA64EAEE`); at 64, **164.4** (load 3.4; p95 183.8), −20 %, picture
  `CBDAE0A825C42E52`, sound `2C225C73DB4D3F9D` (the base's), state `0FD5ABCCCD365B18`.
- **Named limits.** (1) An interrupt a unit raises between events is seen by the EE up to a round (64 cycles) late. (2)
  A VU operation issued as a macro instruction that reaches VU1's registers through VU0's window (§5.4.7) sees VU1 as
  the last round left it. (3) The IOP runs up to a round behind the EE's side between observations. Each is timing only;
  none showed in the windows' pictures or sound or in the test programs.
- **Not done**: the VU call path (§60.7 lever 2) beyond the fewer calls rounds give; the EE's dispatcher and interpreter;
  the dock's and island's profiles. The handheld target, 13.4 ms, stays about 37 times away on the desktop's arena.

### 62.6 What was read for this stage, and questions

**Read.** This page's §58.3, §60 and §61; `Sapphire_Redesign.md` §4.0, §4.1, §4.3 and §4.4; `Sapphire_Scheduler.md`
§54's headings. Of the references (`~/.cache/emusen/sealed/ps2-reference/`): PCSX2's `R5900.cpp`
(`_cpuEventTest_Shared`), `VUmicro.cpp` (`ExecuteBlock`), `VU1micro.cpp` (`vu1Finish`, `vu1ExecMicro`), `Vif_Codes.cpp`
(MSCAL, FLUSH) and `VUops.cpp` (`_vuXGKICKTransfer`); DobieStation's `emulator.cpp` (`Emulator::run`) and
`scheduler.cpp` (`calculate_run_cycles`); Play!'s `PS2VM.cpp` (`UpdateEe`). Nothing was copied; the rounds and the
light COP2 observation are written on Sapphire's board. No firmware was listed, and the BIOS image was given only as
the bench's argument.

**Questions.**

1. **[decision] The next step.** *Recommended*: VU micro mode's call path (§62.4, about 80 ms on the arena): the
   recompiler's per-pair helpers and the operations' lane wrappers done inside the compiled microprogram. The EE's
   dispatcher and interpreter (about 270 ms) are larger but are the recompiler's own work (`EmuSen_Recompiler.md`).
2. **[routine] The round's length.** *Recommended*: keep 64. Longer rounds measured no gain (§62.2), and at 4,096 the
   XGKICK queue outgrows the state's slots.

### 62.7 A state written inside a round (2026-10-29)

The crate's tests found that a state written between two frames' ends (`state::tests`, a console stepped 12,345
instructions into a frame) carried no owed time, so the console that loaded it brought its board whole at the next
`run_to` while the one that saved it went on owing, and the two diverged. The round's end is now derived, `now + round`,
not kept, and a load takes the owed time from the EE's time (`run_to` is always last asked for the EE's time), so a
loaded console continues as the one that saved it, and `SPH2` is unchanged. ps2autotests on this build: 52 of 55 and 23 of 23, every text identical to a round of 0. Re-measured on that build: the short window
176.6 and 178.6 ms with step 2's digests in both runs; the arena **498.2** (load 3.1; p95 668.8), picture
`73644478C1BFD800`, sound `BCA8F00F4743A955` (the base's), state `66A722AE67ED92E4`, the six fields' pictures differing
from the base's as step 2's did (10.6 to 27.9 % of pixels, mean 0.05 to 2.4 levels).

**Tests** (release with debug assertions, so §61.2's check of the timers' cache ran on every `reschedule`): the
library 117, `blocks` 28, `boot` 6, `debugger` 6, `dmac` 21, `gif` 16, `idle` 7, `pcrtc` 6, `s34` 4, `scheduler` 5,
`sif` 5, `state` 11, `vif` 15, `vu0` 15, `vu_jit` 19 and `vu_micro` 25, all passed. They run at a round of 0
(`System::new`), but for the library's machine tests and `state`, which run `Machine` at 64. The WiseMan filter
`SapphireTests` was not run here.
