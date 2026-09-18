# Mars — performance: the profile, and what each change bought

*Phase G, opened 2026-09-18. The probe is `EmuSen.WiseMan/Cores/MarsPerformanceProbeTests.cs`; the profile was taken
with `dotnet-trace` against a scratch harness outside the repository. The phase's brief is `Mars_Gameplan.md` §4.7:
"opens with a profile, not a plan".*

---

## 0. The rule this phase works under

**Every change must leave Mars's output bit for bit as it was.** Everything Phases A to F built was graded on exact
output — the corpus's verdicts, angrylion's pixels, the replays of `Mars_SaveStates.md` — and a faster Mars that draws
one pixel differently has thrown that grading away. So each change here is measured twice: for speed, and against a
recorded baseline of the three games' output (§1), frame by frame.

## 1. The probe

`MarsPerformanceProbeTests`, off unless `EMUSEN_MARS_PERF` names a directory, runs every game in the test folder for
600 frames through the bundle a frontend loads — debug target attached, the picture presented every frame — in a
data-store folder of its own, so no save from an earlier run changes the boot. It times `RunFrame` alone, and after
each frame hashes the presented picture, all of RDRAM and the audio drained, beside the cycle count — and, since §9,
the whole save state, so a divergence in a register or a device shows on the frame it happens.

- **With `EMUSEN_MARS_PERF_RECORD=1`** it writes those hashes as the baseline, one file a game, in
  `~/.cache/emusen/mars-golden/`. They are derived from commercial games and stay out of the repository.
- **Otherwise it compares**, and fails on the first frame whose picture, memory, audio or cycle count differs,
  naming it.

**It reproduces.** Recorded and then compared with no change between, all 600 frames of all three games matched,
and the times agreed within one per cent:

| | first run | second run | frame budget |
| --- | --- | --- | --- |
| Ocarina of Time (Europe) | 7.30 fps, 137.0 ms | 7.33 fps, 136.5 ms | 20 ms (PAL) |
| Super Mario 64 (Europe) | 9.64 fps, 103.7 ms | 9.57 fps, 104.5 ms | 20 ms (PAL) |
| Wave Race 64 (USA) | 14.71 fps, 68.0 ms | 14.68 fps, 68.1 ms | 16.7 ms (NTSC) |

That is 15, 19 and 25 per cent of the console's speed, in a Release build on this workstation, from a cold boot
through each game's title or attract mode.

## 2. The profile

**Method.** `dotnet-trace collect --profile dotnet-sampled-thread-time`, which samples each managed thread's stack at
about 100 Hz, over the same 600 frames of each game, in Release, through a frontend's bundle. The RSP and the RDP run
*inside* the processor's bus ticks, so a flat per-method count would bill their work to whatever instruction was
executing; instead each sample is charged to the **deepest Mars component on its stack** — the RSP if `Rsp.Step` is
on it, else the RDP, else the video interface, else the processor and its bus.

| share of the emulation thread | VI | CPU and bus | RDP | RSP | the core's loop |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | **53.0%** | 35.7% | 4.0% | 3.6% | 3.7% |
| Wave Race 64 | 33.6% | **47.9%** | 7.5% | 6.6% | 4.3% |
| Ocarina of Time | **61.6%** | 28.6% | 5.9% | 1.7% | 2.1% |

**The plan asked whether the interpreter or the RDP is the wall** (`Mars_Gameplan.md` §2.2). ~~"The first question it
asks is whether the interpreter or the RDP is the wall."~~ **It is neither: it is the video interface's scan-out.**
Turning RDRAM into the displayed picture — the 640-column walk with its anti-aliasing, dither and divot filters,
which a frontend asks for every frame — is the largest single cost in two of the three games, and `Vi.Walk` and
`Vi.Dither` alone take 30 to 61 per cent of the thread. The interpreter is second. The RDP and the RSP together are
under 15 per cent in every game.

**What that says about the plan's levers.** A recompiler answers only the processor's share: a third to a half of the
time in these scenes, so even an infinitely fast CPU would leave Mars at under twice its speed. HLE graphics, a GPU
rasterizer and microcode HLE — the levers `Mars_Gameplan.md` §2.1 deferred — answer shares of 4 to 14 per cent.

**What it is not evidence for.** These are boots, title screens and an attract race: 3D scenes with more geometry would
move work towards the RSP and the RDP, and nothing here says how far. 100 Hz sampling over about a minute a game
resolves shares of a per cent or so, not below. And "CPU and bus" includes whatever the JIT inlined into `Cpu.Step` —
device ticks, bus reads, the timer — so its exclusive share is not the interpreter's dispatch alone.

## 3. The VI's samples, once a row

**What was slow.** With resampling on, every output pixel mixes four samples — the pixel a step lands on, the next one,
and the two below — and every sample is the source pixel fetched and filtered: the dither filter reads eight
neighbours, the anti-aliasing filter six, and divot, when on, samples each neighbour across the row as well. The
picture is 640 columns wide and most frame buffers are 320, so each step lands on a source pixel twice, and each
source pixel was filtered about eight times a row — once for each output pixel that reached it, from either side.

**The change.** `Vi.Remembered` keeps each row's samples by their offset from the row's first step, for the row being
read and the row below it, and a stamp per row retires them without clearing anything. The first request makes the
sample; later ones in the same row take it. A sample is a function of RDRAM, its hidden bits, the video registers and
its own arguments, and a scan changes none of them — it writes only the raster — so the first answer is the answer.
The cache is presentation scratch, marked out of the save state, which keeps the state's layout as it was.

**What it bought**, from the probe, with every frame of all three games identical to the baseline:

| | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 7.33 fps | 13.32 fps | 1.82× |
| Super Mario 64 | 9.57 fps | 15.48 fps | 1.62× |
| Wave Race 64 | 14.68 fps | 19.30 fps | 1.31× |

**And what the profile says now:**

| share of the emulation thread | VI | CPU and bus | RDP | RSP |
| --- | --- | --- | --- | --- |
| Super Mario 64 | 23.4% (was 53.0%) | 58.1% | 6.7% | 5.5% |
| Wave Race 64 | 13.3% (was 33.6%) | 62.4% | 10.0% | 8.2% |
| Ocarina of Time | 31.5% (was 61.6%) | 49.3% | 11.2% | 3.6% |

The processor is now the largest share in every game. What remains of the VI is mostly the walk's own per-pixel work —
three mixes, gamma, four stores — and the samples of a source row that two output rows share, which this change
still makes twice, once for each row.

## 4. The processor's own addresses, taken directly

**Where the processor's time went.** A diagnostic build that kept the step's callees from being inlined, profiled on
Wave Race, split the processor's share for the first time: **the instruction fetch was 28.6 per cent of the whole
thread** — `FetchInstruction` and `TranslateAccess` — against 1.5 for executing the instruction, 16.5 for stepping
the devices every instruction, and 20.4 for the step's own body. The interpreter was not slow at interpreting; it
was slow at finding the instruction. Every fetch computed the privilege mode three times — once to decide
reverse-endian mirroring, twice more for the segment map and its addressing width — and ran the general segment
decoder, which handles six maps and 64-bit windows.

**The change.** Nearly every fetch, and most data, is kernel code in KSEG0 or KSEG1, where the segment map's answer is
the address's low 29 bits. `TranslateAccess` now answers that range directly when the processor is in kernel mode,
and the fetch checks it before mirroring. The answer is the same as the decoder's for every address in the range:
64-bit addressing hands anything at or above `0xFFFF_FFFF_8000_0000` to the 32-bit map, where kernel mode's KSEG0
and KSEG1 are direct; mirroring never applies in kernel mode; and supervisor and user code still take the general
path and fault as they did. The CPU, TLB, segment and FPU suites and the corpus, which exercises every mode and both
addressing widths, all pass unchanged (the corpus at 46).

**What it bought**, every frame identical to the baseline:

| | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 13.32 fps | 15.40 fps | 1.16× |
| Super Mario 64 | 15.48 fps | 18.43 fps | 1.19× |
| Wave Race 64 | 19.30 fps | 22.89 fps | 1.19× |

## 5. Samples a zero fraction throws away, never asked for

**What was wasted.** The walk mixes the four samples by the step's fractions, and a mix by a zero fraction returns the
near pixel unchanged. With the common 320-to-640 scale, every other column lands exactly on a source pixel and its
horizontal fraction is zero, so its `next` samples were fetched, filtered and discarded; a row whose vertical fraction
is zero did the same with the whole row below. The cache of §3 made each such sample cheap to fetch twice, but not
free to make once.

**The change.** The walk asks for `next` only when the horizontal fraction is not zero, and for the row below only
when the vertical fraction is not zero. A sample has no effect but its value, so not asking for one the arithmetic
would ignore changes nothing, and a row whose vertical fraction is zero no longer filters the line below at all.

**What it bought**, every frame identical to the baseline:

| | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 15.40 fps | 15.34 fps | unchanged |
| Super Mario 64 | 18.43 fps | 21.17 fps | 1.15× |
| Wave Race 64 | 22.89 fps | 25.06 fps | 1.09× |

Ocarina of Time's picture is scaled so that its fractions are rarely zero, which is what the unchanged figure says.

## 7. A source line filtered once a scan, not once a row

**What was still repeated.** §3's cache lasted a row. A picture stretched vertically reads each source line in two or
more consecutive rows — as the line a row lands on, then as the line below the next row's — and each of those rows
filtered it again. Ocarina of Time, whose fractions are rarely zero (§5), paid for that in full.

**The change.** Two slots, each keyed by the source line it holds and by whether it was sampled with the fetch bug
folding the row below onto it — the only way the bug changes a sample, since the filters test it for the value 1 and
nothing else. A row looks for its two lines among the slots and takes over the one it does not need when a line is
missing; a new stamp retires what the slot held before. Every scan starts with both slots empty, because RDRAM has
changed since the last one.

**What it bought**, two runs, every frame identical to the baseline both times:

| | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 15.34 fps | 17.44 and 17.35 fps | 1.13× |
| Super Mario 64 | 21.17 fps | 21.04 and 20.83 fps | unchanged |
| Wave Race 64 | 25.06 fps | 24.49 and 24.55 fps | unchanged, within the noise |

## 6. Measured and rejected

- **The video interface's timing, worked out when its registers change** rather than on every instruction. Its step
  reads two registers, tests the TV standard and multiplies twice, every instruction, and a cache of the three values
  it derives — forgotten on a write to either sync register and after a loaded state — is exactly equivalent. Two runs
  of the probe, against the build before it: +2.3 and +1.5 per cent for Super Mario 64, +2.4 and +1.8 for Wave Race,
  −3.2 and +0.8 for Ocarina of Time, whose own two runs differed by four per cent. That is inside what the probe can
  distinguish, and the cache carries an invalidation duty on every state load that a later change could forget.
  Reverted.

**What that rejection taught about the probe.** §1's first pair of runs agreed within one per cent, and a later pair
of one build differed by four. The noise is wider than it first looked, so a change is kept here only if it clears a
few per cent in more than one game, or clears it twice.

**And about the profile.** The diagnostic build that split the processor's share (§4) marked the step's callees
non-inlinable and read exclusive time per method. .NET's sampler stops a thread at a safe point, and safe points
cluster at calls and loop back-edges, so a method made mostly of calls — the bus's `Tick`, the step itself — collects
more samples than it costs. Its component shares are sturdier than its method shares, and neither decides a change:
the probe does.

- **Ranking the processor's per-instruction costs by removing them.** Builds with the timer update, the interrupt
  check or a device's step deleted ran Super Mario 64 in 7.5, 5.6 and 7.5 seconds against 12.7, which measures
  nothing: without interrupts, or without the RSP, the game does different work — it idles in a cheap loop and
  never feeds the display processor — so the time saved is the game's, not the check's. The one removal that leaves
  the game's behaviour alone, the debugger's per-instruction gate (the breakpoint check, coverage and the profiler's
  count), is worth about four per cent (12.2 against 12.7 seconds, twice), which is recorded as a known cost and not
  acted on.

## 8. Where the phase stands

| | at the profile | now | |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50 fps) | 7.3 fps | 17.4 fps | 2.4×, 35% of the console |
| Super Mario 64 (PAL, 50 fps) | 9.6 fps | 21.0 fps | 2.2×, 42% of the console |
| Wave Race 64 (NTSC, 60 fps) | 14.7 fps | 24.5 fps | 1.7×, 41% of the console |

Every frame of all three games has matched the §1 baseline after every change. The profile now:

| share of the emulation thread | CPU and bus | VI | RDP | RSP | the core's loop |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 56.9% | 18.0% | 9.4% | 8.7% | 7.0% |
| Wave Race 64 | 60.8% | 9.7% | 12.7% | 10.8% | 6.0% |
| Ocarina of Time | 46.2% | 27.8% | 14.9% | 5.0% | 6.0% |

**The interpreter is now the wall** in all three games, which is where the plan expected to start. What is left in it is
not decoding — executing the instruction was 1.5 per cent in §4's split — but the work around each instruction:
the interrupt check, the timer, three device steps and the loop's own checks, each small and none removable without
changing what a game does. The levers that remain are of a different size from the ones this page has used:

- **A cached or threaded interpreter**, decoding blocks once. §4 says decoding is not where the time goes, and
  `Venus_PPU.md` §13 records threaded dispatch measured and rejected on the SNES core.
- **A recompiler**, which removes the per-instruction work by compiling blocks — the plan's §2.2 names it as a choice
  that "makes some class of divergence unobservable", and it is a project, not a change.
- **Timing restructured around events** — the devices told when their next event falls, instead of stepped every
  instruction — which can be exactly equivalent, and is the largest of the three in what it touches.

Each is a decision about the core's design rather than a change to it, and is left for one. **Decided the same day:
timing restructured around events** (§9). **What none of this is
evidence for**: speed in 3D gameplay. The probe runs boots, title screens and an attract race; a scene with more
geometry moves work towards the RSP and the display processor, whose shares here are the smallest.

## 9. The VI and the AI told when they are next due

**What the step had left.** §8 named the work around each instruction as the wall; the question for a design change is
how much of it there is. A microbenchmark measured it directly: a three-instruction loop in KSEG0 with interrupts off,
the VI programmed for NTSC and a buffer playing, the RSP halted, stepped through `Cpu.Run` sixty million instructions
at a time in a Release build carrying switches that turned each piece off. The build was never committed. Removing
work from a loop that cannot take an interrupt changes nothing the loop does, which is what makes this measurement
admissible where §6's deletions from a running game were not.

| removed | ns an instruction, best of three |
| --- | --- |
| nothing | 10.5 |
| the three device steps (the RSP's halted test kept inline) | 7.7 |
| those, the interrupt check and the timer | 5.7 |

**The single removals are not in the table because they did not separate.** Each sat inside the JIT's own spread — the
same build measured 10.5 and 11.9 — and two measured *slower* than removing nothing, which is code layout and the
tiered compiler's profile, not cost. Only the combined removals cleared the spread, and they did in every run.

**The change.** The VI and the AI each owe a debt against a threshold, growing at a constant rate while their registers
stand still (`Mars_VideoTiming.md` §1.1, `Mars_Audio.md` §3). Each now records the cycle its debt was last brought up
to and the first cycle at which its next half line or sample is owed, and the bus keeps the earlier of the two. A tick
is the cycle count, the RSP's steps while it runs, and one comparison. A tick that reaches the due cycle runs the
devices as a stepped tick would have — every half line owed, then every sample — and schedules the next.

**Why it is exact, and not merely close:**

- *Between writes a debt is linear in cycles*, so the sum over any run of ticks is `(now − then) × rate` however the
  ticks were cut. The one rule that is not linear — an idle AI zeroes its debt on every tick — is applied on the first
  settle after a tick has passed idle, which is when a stepped tick would have applied it; a settle in the same cycle
  the buffer ran out, when no tick has passed, keeps what was owed.
- *The rates change only on register writes.* The VI's sync registers set the clock both devices divide, and the AI's
  own registers start, stop and re-pace it, so a write to either device first settles both at the old rate and then
  reschedules both at the new one. A write lands in `Execute`, before its instruction's tick, so the new rate governs
  that tick, as it did.
- *Events are eager; only the debts are lazy.* A tick at or past the due cycle fires, so the registers a game reads —
  the current line, the AI's length and status, the MI's pending bits — hold at every instruction what they held when
  stepped. Nothing but the devices reads a debt.
- *A save settles both devices first*, so a state holds the bytes a stepped machine's would. The format is unchanged,
  states saved before this change load, and the derived fields are left out of the state and rebased on load
  (`Mars_SaveStates.md` §2).

The RSP is not scheduled. It runs in step with the processor, and anything coarser would change what each sees of the
other.

**What grades it.** §1's probe now also hashes the whole save state after every frame — the processor and COP0, the
MI, the devices' debts, the RSP — so a divergence is caught on the frame it happens rather than when it first reaches
RDRAM or the picture. The baseline was re-recorded on the build before this change: its first five columns matched
the old baseline exactly, and a second run matched the new one on all 600 frames of all three games. After the change,
all 1,800 frames matched, state and all.

**What it bought:**

| | before (two runs) | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 17.18, 17.09 fps | 18.84 fps | +10% |
| Super Mario 64 | 20.80, 20.57 fps | 23.38 fps | +13% |
| Wave Race 64 | 24.14, 24.02 fps | 26.92 fps | +12% |

**What catches a mistake in it.** Fourteen deliberate breakages of the scheduler, each run against every Mars test
and, where those all passed, against the probe (`mutate_events.py` in the workbench):

| breakage | before `MarsEventTimingTests` | after |
| --- | --- | --- |
| a VI write not rescheduling | three Wave Race boot tests | unchanged |
| an AI write not rescheduling | the undrained-audio state test | unchanged |
| a load not rebasing the debts | the fresh-core replay | unchanged |
| the bus, or the AI, skipping an event on its own due cycle | two audio tests | unchanged |
| a VI write not settling first | the probe alone (frames 1–2) | two tests |
| an AI write not settling first | the probe alone (frames 6–9) | one test |
| an idle AI tick not zeroing its debt | the probe alone, in two games of three | one test |
| a save not settling | the probe alone (frames 6–9) | two tests |
| the VI skipping its own due cycle | the probe alone (frames 1–2) | three tests |
| an AI settled twice in one cycle zeroing its debt | **nothing** | one test |
| a load into a running bus not rescheduling | **nothing** — the probe never loads | one test |
| either due cycle rounded down rather than up | nothing, correctly | — |

Five breakages had only the probe between them and a commit, and two had nothing; the new tests hold all seven without
a game image. The last row is an equivalent mutant, recorded so that it is not mistaken for a gap: a device asked
early finds nothing owed and asks again later.

**What this does not cover.** The tests for the two rarest rows state the stepped design's rule, not the console's: a
DAC whose phase resets once a tick passes idle is how Mars models the AI (`Mars_Audio.md` §3), and this change keeps
the model rather than measuring the hardware. Nor does it reverse §6's rejection of a cache of the VI's timing: that
cache kept the per-instruction step and made it cheaper, by an amount the probe could not see; this removes the step.

## 10. The interrupt check and the timer, asked only when their answer can change

**What they did every instruction.** Before each fetch the processor copied the MI's line into Cause, read Status
twice and worked out the pending set; after each successful instruction it read the count and Compare and asked
whether Compare fell inside the interval the instruction had covered (`Mars_Cpu.md` §12). Both answers are functions of
a handful of inputs — the MI's line, Status and Cause, Compare and the count's bias — which change a few times a frame,
against a million and more instructions.

**The change.**

- *The interrupt check* runs when the MI's line differs from the value it last saw — compared every instruction, so a
  device that raises or clears anything reaches it on the step it always did — or when `_recheck` is set: by any COP0
  write, entering an exception, returning from one, the timer's hit, and `Cpu.Cop0Written`, which anything that writes
  COP0 from outside an instruction must call (the boot, a loaded state, a test).
- *The timer* asks its interval question once, when Compare or Count is written or the timer fires, and the answer is a
  cycle: the first at which the count, advancing every second cycle, reaches Compare from the count it was settled at —
  a whole wrap away if the two are equal. Each successful instruction compares its end against that cycle. The count
  each successful instruction ends on is still kept, as before, so a state's bytes are unchanged.

**Why it is exact.** The check is a pure function of its inputs; between changes it gives the answer it gave last
time, which — since it did not raise then — is to do nothing. The timer fires on the first successful instruction to
end at or past its cycle, which is the instruction whose interval first contained Compare. An exception's step never
asked about the timer and still does not, so a crossing inside one is found by the next successful instruction, as
before.

**What stands behind "exact" in Debug builds.** A cache that is right only while every writer remembers to invalidate
it is the failure `Mars_Memory.md` §3.1 designed the counter against, so this one is checked. In Debug builds every
instruction whose check is skipped verifies that the full check would have done nothing, and that Compare and the
count's bias are what the timer's cycle was worked out from, and throws if not. Every Mars test runs in Debug — the
corpus included, whose interrupt and timer cases run a great many instructions through the verifier. Its first run
found seven tests that set Compare directly and expected the timer to notice; each now calls `Cop0Written`, which is
the rule the verifier enforces. In Release the check is compiled out.

**What it bought**, two runs, against §9's:

| | §9 | this change | |
| --- | --- | --- | --- |
| Ocarina of Time | 18.84 fps | 19.81, 19.64 fps | +5% |
| Super Mario 64 | 23.38 fps | 24.68, 24.53 fps | +5% |
| Wave Race 64 | 26.92 fps | 28.20, 28.21 fps | +5% |

That clears the few per cent §6 asks for in all three games, twice, and by not much more; it is the smaller of the two
halves, as §9's microbenchmark said it would be.

**What catches a mistake in it.** Fourteen breakages (`mutate_events_cpu.py`), every Mars test in Debug, and the probe
for what those passed:

| breakage | every Mars test, in Debug | the probe | now |
| --- | --- | --- | --- |
| a COP0 write not rechecking | five (the corpus, Wave Race's boot) | — | unchanged |
| the timer's hit not rechecking | three | — | unchanged |
| the MI's line not compared by value | five | — | unchanged |
| a Compare write not rescheduling | eleven | — | unchanged |
| a Count write not rescheduling | the corpus's two | — | unchanged |
| the boot not calling `Cop0Written` | thirty-two, all by the verifier | — | unchanged |
| a return not rechecking | nothing | two games of three (frames 2, 103) | one test |
| the timer skipping its own due cycle | nothing | all three (frames 22–36) | one test |
| the cycle counted from an odd one as if even | nothing | two games (frames 22, 36) | the same test |
| an equal value firing at once, not after a wrap | nothing | one game (frame 0) | one test |
| a load not calling `Cop0Written` | nothing | nothing — the probe never loads | one test |
| each step not keeping the count it ended on | nothing | all three, frame 0, by the state hash alone | — |
| entering an exception not rechecking | nothing | nothing | equivalent |
| a hit scheduling from the count before it | nothing | nothing | equivalent |

Four breakages had only the probe to stop them and one had nothing; four new tests hold all five. The row with no
test is kept by the probe alone, and on purpose: without the per-step count the machine behaves identically, but a
state's bytes stop being the stepped machine's, and a state saved more than 2³² counts (about 92 seconds) after the
last timer write would load with its timer a wrap out — the elapsed count is taken modulo the counter's width. Only a
reference recorded from the stepped build can grade the bytes, and that is what the probe's baseline is. The two
equivalent rows are recorded so that they are not mistaken for gaps: an exception sets EXL, which blocks every line
until a return or a Status write, both of which recheck; and a hit scheduled from the count before it fires once more
on the next instruction, setting a bit that is already set.
