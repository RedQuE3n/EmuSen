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
through each game's title or attract mode. **Its timings are not the grade of a small change**: runs an hour apart
drift by about six per cent on identical code, so from §12 on a speed claim comes from interleaved builds and the probe
grades exactness alone.

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

## 11. Where the phase stands after event timing

| | at the profile (§2) | §8 | now | |
| --- | --- | --- | --- | --- |
| Ocarina of Time (PAL, 50 fps) | 7.3 fps | 17.4 fps | 19.7 fps | 2.7×, 39% of the console |
| Super Mario 64 (PAL, 50 fps) | 9.6 fps | 21.0 fps | 24.6 fps | 2.6×, 49% of the console |
| Wave Race 64 (NTSC, 60 fps) | 14.7 fps | 24.5 fps | 28.2 fps | 1.9×, 47% of the console |

Every frame of all three games has matched the baseline after every change, and since §9 the baseline includes the
whole save state. ~~The profile, taken again the same way (§2):~~ **Retracted the same day: this profile measured §8's
build, not this one** (§12). The table is kept as it was recorded; §14 has the profile of the build it claimed to be.

| share of the emulation thread | CPU and bus | VI | RDP | RSP | the core's loop |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 57.3% | 18.5% | 9.1% | 7.9% | 7.1% |
| Wave Race 64 | 59.1% | 9.4% | 13.1% | 11.9% | 6.5% |
| Ocarina of Time | 46.6% | 27.7% | 15.0% | 4.9% | 5.7% |

~~**The shares barely moved, and that is not evidence that nothing changed.** §9 and §10 cut 15 to 17 per cent of the
time, but part of what they removed was billed to the VI and AI — their per-tick steps were theirs — and the sampler's
lean towards call sites (§6) moves with every call removed. A share is not a measure of what one change bought; the
probe is.~~ They barely moved because the build had not changed: the harness linked a copy of the core taken before
§9. The paragraph above was an explanation written to fit a wrong measurement, which is the failure this page exists
to prevent; §12 says how it was found and what now stops it.

**What event timing leaves.** Nothing that can be scheduled exactly is still stepped: the VI, the AI, the timer and
the interrupt check all act only when due. The RSP is stepped every tick while it runs, and must be, since anything
coarser changes what it and the processor see of each other. What remains around each instruction is the
interpreter's own work — ~~`Cpu.Step` alone is 38 to 48 per cent of the thread's exclusive time, against 4 to 5 for
executing the instruction~~ (the stale profile's figures; §14 has 27 to 28 against 6 to 7) — and the loop's debugger
gate, which §6 put at about four per cent.

**The levers now.** Two sizes, as before:

- *Small and exact:* a loop with no debugger gate, taken when no breakpoint, coverage or profiler is armed — about
  four per cent, if §6's measurement holds, and only if nothing can arm one mid-frame.
- *Large:* a cached interpreter or a recompiler for the step's body. §8's reasons for caution stand: decoding is still
  not the cost, and a recompiler is a project, not a change.

**What none of this is evidence for** is unchanged from §8: speed in 3D gameplay, where the RSP's and the RDP's shares
would grow.

## 12. Timing by interleaving, and the profile that measured the wrong build

**What went wrong twice in one afternoon.** §13's change measured four per cent *slower* on the probe, in all three
games, twice. The committed build, run again minutes later, measured five to six per cent *faster* than the same code
had an hour before. Identical code had drifted by more than the effects this phase now chases, so the probe's timings,
taken one run after another, cannot decide a change of this size — whatever the machine is doing between runs is in
them. Setting up the comparison properly then exposed the second fault: the two harness builds came out byte for byte
identical. The harness linked copies of the core placed in its own folder at 09:27 for §4's diagnostic build, not the
project, and §11's profile ran against those — §8's build.

**The method now.** Each variant is built as its own harness — a git worktree per commit, the harness referencing the
worktree's project, never a copied DLL — and the builds are run alternately: 600 frames of each game, three rounds,
the order rotated each round. Within such a run one build's three times differ by at most 0.6 fps. The probe still
grades exactness; it no longer grades speed.

**§9 and §10, measured again that way**, median of three (fps):

| | before §9 (`2745f74`) | §9 (`81bc070`) | §10 (`a434c9c`) | §13 |
| --- | --- | --- | --- | --- |
| Ocarina of Time | 17.5 | 19.2 (+10%) | 19.9 (+4%) | 21.3 (+7%) |
| Super Mario 64 | 20.5 | 23.8 (+16%) | 25.5 (+7%) | 27.3 (+7%) |
| Wave Race 64 | 24.2 | 28.2 (+17%) | 30.1 (+7%) | 31.9 (+6%) |

Both stand, a little larger than recorded. **What this does not cover**: §3 to §5's and §7's figures are sequential
probe runs too. Their effects, 1.2 to 1.8 times, are far larger than the drift, so they stand as effects; their exact
percentages carry a few per cent of it.

## 13. A frame loop with no debugger in it

**What it did every instruction.** `RunFrame` asked three questions before each instruction: could a breakpoint fire
(`CouldBreak`), is coverage armed, is the profiler. §6 measured the three at about four per cent and left them, because
an armed break can appear in the middle of a frame — a data breakpoint's write, a run-to's interrupt — so the answer
for one instruction is not the answer for the next.

**The change.** `BreakpointRegistry.IsQuiet` makes a stronger promise than `CouldBreak`: true only when nothing is
armed that anything a running frame does could set off — no breakpoint, step, depth guard, data breakpoint,
uninitialised-read check, run-to of any kind, or hardware condition (`EmuSen_Debugging_Tools_Reference_v5.md` §3.26).
Coverage and the profiler are armed only by commands. So Mars asks all three once, at the start of a frame, and when
none is armed runs the frame in `RunQuietly`: the same frame-end test, the processor's step, nothing else. The watched
loop then finds its own condition already false.

**Why it is exact.** In a quiet frame the watched loop's three tests are false on every instruction and nothing the
frame does can make them true, so the two loops run the same instructions and end on the same one. All 1,800 probe
frames match, state and all. **What it changes**: a frontend that arms a breakpoint from another thread while a quiet
frame runs gets it from the next frame, not the next instruction. The headless runner arms between frames, and is
unaffected.

**What it bought** (§12's table): +7, +7 and +6 per cent. **What catches a mistake in it**: dropping any one of
`IsQuiet`'s seven conditions fails two registry tests — one arming each thing directly, one running 400 random
histories of commands and, whenever the registry says quiet, eight kinds of thing a frame does, none of which may halt
it. Asking the loop's gate without the registry fails seven Mars debugger tests, and without coverage one; without the
profiler it failed nothing, and `The_profiler_counts_what_the_processor_ran_only_while_armed` now holds it.

## 14. Where the phase stands

| | at the profile (§2) | now, §12's method | |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50 fps) | 7.3 fps | 21.3 fps | 2.9×, 43% of the console |
| Super Mario 64 (PAL, 50 fps) | 9.6 fps | 27.3 fps | 2.8×, 55% of the console |
| Wave Race 64 (NTSC, 60 fps) | 14.7 fps | 31.9 fps | 2.2×, 53% of the console |

The first column is the morning's probe, run sequentially, so the multiples carry a few per cent of §12's drift. The
profile of this build, taken with the harness that now references the project:

| share of the emulation thread | CPU and bus | VI | RDP | RSP | the core's loop |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 51.3% | 23.6% | 12.1% | 9.0% | 3.8% |
| Wave Race 64 | 54.7% | 12.8% | 17.1% | 11.9% | 3.4% |
| Ocarina of Time | 40.0% | 33.7% | 17.7% | 5.2% | 3.4% |

`Cpu.Step` is 27 to 28 per cent of the thread's exclusive time and executing the instruction 6 to 7. Two things stand
out that the stale profile hid. **The RSP's step loop** — `SpInterface.Step`, which runs the signal processor a tick at
a time and reads its break state twice around each instruction — is 4 to 11 per cent on its own, billed to the
processor's share because the processor's tick calls it; with the RSP's own 5 to 12, the signal processor is 9 to 23
per cent of the thread. **The VI's scan-out** is back to a third of Ocarina of Time's time, `Vi.Walk` alone a quarter.

**The levers now**: the RSP's step loop, which can change without changing what either processor sees; the scan-out's
walk again, for Ocarina of Time; and, for the processor's step, a cached interpreter or a recompiler, which remain
decisions about the core's design.

## 15. The RSP's step loop

**What the profile said.** §14 put `SpInterface.Step` — the loop that runs the signal processor a tick at a time — at
4 to 11 per cent of the thread's exclusive time. It read the processor's break bit before and after every
instruction, to notice the one instruction that set it, and the processor fetched each instruction as four separate
byte reads.

**The change.** The break interrupt is raised by the BREAK instruction itself, the only place the bit rises, under the
condition the loop used — interrupt-on-break set and the bit down before — so the loop reads nothing around each
instruction. And the fetch reads the word whole, `BinaryPrimitives.ReadUInt32BigEndian` over the same four bytes of
instruction memory, through a reference the processor now keeps (the bus never replaces the array, only writes into
it). One consequence outside the loop: a caller that steps the processor directly — a test, a debugger — now gets the
interrupt a break raises, where before only the interface's loop raised it.

**Measured.** A microbenchmark — the RSP running a four-instruction scalar loop beside the processor's three-instruction
one, median of seven runs — went from 11.76 to 11.46 ns per pair of instructions without the break bookkeeping, and to
10.65 with the whole-word fetch: 9 per cent, with the RSP busy every cycle. The games, six interleaved rounds (§12):

| | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 21.3 fps | 21.4 fps | within the spread |
| Super Mario 64 | 27.45 fps | 27.7 fps | +1%, five rounds of six |
| Wave Race 64 | 32.0 fps | 32.6 fps | +2%, every round above every round |

All 1,800 probe frames are identical, state and all.

**What that says about §14's profile.** The microbenchmark and the games agree with each other and not with the
profile: about a nanosecond saved per RSP instruction, with the RSP running something like a third of Wave Race's
cycles, is two per cent, not the ten the loop's exclusive share implied. The sampler's lean towards loops and call
sites (§6) put samples there that the loop did not cost. The loop was never a large lever. It is kept because the
effect is measurable, the change exact, and the loop simpler than the one it replaced — and it is measurable only
because of §12: §6's rule of a few per cent in more than one game was set by the probe's noise, and interleaved builds
resolve one to two.

**What the breakage round found.** Removing the raise fails the new
`A_break_raises_the_interrupt_only_when_interrupt_on_break_is_set` and two Wave Race boot tests; raising whether or not
interrupt-on-break is set failed nothing until that test; letting the loop run past a halt is equivalent, since the
processor's own step checks it. **Two rules survive on purpose, because the references dispute them:**

- **"Only if the bit was down."** The FPGA core (`RSP.vhd`'s break handler), Project64 (`Special_BREAK`) and
  mupen64plus all raise the interrupt on every break with interrupt-on-break set, whatever the break bit held. Mars's
  condition came from how the old loop noticed a break — by the bit rising — not from evidence, and the corpus tests the
  register bit, not the interrupt. The two differ only when the RSP is restarted without its break bit cleared and
  breaks again.
- **Single-step.** Mars halts the processor after every instruction while the bit is set; all three references store
  the bit and ignore it, and the corpus does not test it.

This change keeps both, because it changes no output. Whether to follow the references is a decision about behaviour,
left for one; until it is made, neither rule is pinned by a test.

## 16. A store reported only while something listens

**What the microbenchmark could not see.** §9's and §15's loops ran a bare bus. A frontend attaches a debug target,
which installs itself as the bus's write observer when it is built (`Mars_Debug.md` §3), and the bus then reported
every processor store: a test of the three whole-word windows, then one interface call per byte into the watch
registry and the breakpoint registry, each of which walked its empty list and returned. `Mars_Debug.md` §7 had
measured the hooks as costing nothing; it measured them without a target attached and with one, on a build whose
breakpoint check dwarfed everything else. The loop of §9 with a load and a store in five instructions, run with an
observer doing exactly what the target's does and nothing watched:

| | ns an instruction |
| --- | --- |
| no observer | 9.97 |
| the target's observer, nothing watched | 15.45 |

About 27 nanoseconds a store — as much again as the store itself — in every frontend and in every probe run so far.

**The change.** `IWriteObserver.Listening`, false exactly when `OnWrite` would do nothing: the target answers with the
watch registry's `HasWatches` and the breakpoint registry's `WatchesWrites` (a data breakpoint or the
uninitialised-read check). The bus asks before it reports. A watch armed while a frame runs is seen by the very next
store, as before; the only cost left is the question, an interface call a store.

**Why it is exact.** With no watch, no data breakpoint and no uninitialised-read check, `RecordWrite` walks an empty
list and `NoteWrite` returns at its first two tests, so the skipped report changed nothing. All 1,800 probe frames
match, state and all. **What catches a mistake**: six breakages — the bus not asking, the bus never reporting, the
target or the registry missing any one of its three terms — each fail one to four named tests, three of them new.

## 17. RDRAM taken directly by the fetch, the loads and the stores

**What the step's cost is made of**, from a switch build of §9's loop and a second loop with a load and a store in
five instructions, medians of five, ns an instruction (the memory loop's spread is under 0.1; the ALU loop's is
wider):

| removed, or changed | ALU loop, of 7.65 | memory loop, of 9.99 |
| --- | --- | --- |
| RDRAM read and written as whole words | −0.36 | −0.39 |
| and the fetch straight from RDRAM | −0.68 | −1.02 |
| and the loads and stores straight from RDRAM | — | −1.79 |
| the privilege mode not recomputed each use (measure only) | −0.78 | −0.71 |
| the MI line not compared (measure only) | −0.31 | −0.33 |
| the count not kept each step (measure only) | −0.23 | −0.19 |
| the instruction count and the return hook (measure only) | −0.16 | −0.16 |

The first three rows are this section; the fourth is §18. The rest are stated so that they are not measured again:
the MI compare is what makes §10 exact, and the count is in the state's bytes.

**The change.** The bus's `ReadArray32` and `WriteArray32` read and write one big-endian word over the same four bytes
(`BinaryPrimitives`), for every caller. And the processor takes RDRAM without calling the bus: a kernel-direct fetch
whose physical address is below the array's length reads the word there; an aligned load below it reads the bytes
named, a byte, a halfword, a word or a doubleword; an aligned store below it writes the bytes named — unless the MI's
repeat is armed or a watcher wants the store reported, when it goes to the bus as before.

**Why it is exact.** For a physical address below RDRAM's length the bus's `Read32` is `ReadArray32(Rdram, …)` and its
`Load` reaches the same word through `Read8`, `Read16`, `Read32` or `Read64`, each of which is the bytes named for an
aligned access; the loads are aligned, since `RequireAlignment` runs first. Its `Store` for that range passes the
cartridge test, then the repeat, then the whole-word windows, and writes byte-precisely; the two conditions that would
make it do otherwise are the two the fast path steps aside for. Above the array's length the bus reads zero and drops
stores, and the fast paths stop at the array. The FPU's `LWC1`/`LDC1`/`SWC1`/`SDC1` were already calling the bus's
`Read32`/`Write32` directly, bypassing `Load` and `Store` — the cartridge latch, the repeat, the whole-word windows and
the report — which this change neither adds to nor fixes; it is recorded as a pre-existing gap. All 1,800 probe
frames match, state and all.

**What catches a mistake in it.** Seven breakages: a store not stepping aside for the repeat fails the new
`A_store_from_the_processor_is_repeated_too` and the corpus; not stepping aside for a watcher fails the three watch
and data-breakpoint tests; a halfword read little-endian, a doubleword read as a word, a byte store writing two, each
fail from six to forty-six tests. A fetch, load or store bound widened past the installed RDRAM failed nothing —
nothing accessed the unpopulated half of a stock console through the processor — and
`The_processor_reads_zero_and_drops_stores_above_a_stock_consoles_rdram` now does.

