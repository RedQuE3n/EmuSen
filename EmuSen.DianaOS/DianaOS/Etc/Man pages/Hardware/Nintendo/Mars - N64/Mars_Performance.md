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

## 18. The privilege mode kept, not recomputed

**What it cost.** `Cpu.Mode` was a property over the Status register — the exception and error levels, then the mode
field — and each instruction asked it at least twice: the fetch's direct-range test and the dispatch's
reserved-instruction prefix, with the loads' and stores' translation and reverse-endian test asking again. §17's
switch build put not asking at 0.7 to 0.8 ns an instruction, about the same as the fetch.

**The change.** The mode is a field, refreshed after every write of Status: a COP0 move, an exception's raising of
EXL, a return's clearing of EXL or ERL, and `Cop0Written` for anything that writes COP0 from outside an instruction
(the boot, a loaded state, a test). `Mode` reads the field. Its addressing-width companion is not cached; in kernel
mode nothing on the instruction's path asks for it.

**What stands behind "exact".** This is the cache that §6 warned of, kept right only while every writer remembers
— so Debug builds recompute the mode from Status on every step and throw if the field disagrees, and every Mars test
runs in Debug. The first run named 176 tests: all of them writing Status directly and running, most through
`PrivilegeFixture`, which now calls `Cop0Written` as §10's rule already asked. A direct write that leaves the mode as
it was — the TLB tests' addressing bit — is not a stale mode and passes. All 1,800 probe frames match, state and all.

**What catches a mistake in it.** Each of the five refresh sites removed, with the verifier on and with it off:

| refresh removed at | verifier on | verifier off |
| --- | --- | --- |
| a COP0 write | the corpus, by the verifier | **nothing** — the corpus enters user mode through `eret`, which refreshes anyway; `Writing_status_enters_the_named_mode_before_the_next_instruction` now holds it |
| an exception | the corpus | the corpus's verdicts |
| a return | the corpus | the corpus's verdicts |
| `Cop0Written` | 176 tests, all by the verifier | 124 tests, by their own assertions |
| the constructor | nothing, correctly | nothing, correctly |

The constructor's refresh is an equivalent mutant — the field's default is kernel and so is a zero Status — kept so
that the field's meaning is stated where it is born. The write-site row is the reason the verifier is not the only
net: every Mars test runs in Debug, but a Release build has only the tests' own assertions, and one of the five sites
had none until this round.

## 19. Where the phase stands after the processor's own step

Three changes to the step, each exact, timed by §12's method (three rounds, medians, fps):

| | before (`faedf85`) | store report gated (§16) | RDRAM direct (§17) | mode kept (§18) | together |
| --- | --- | --- | --- | --- | --- |
| Ocarina of Time | 21.2 | 22.5 | 23.2 | 24.1 | +14% |
| Super Mario 64 | 27.7 | 28.4 | 28.9 | 30.4 | +10% |
| Wave Race 64 | 32.5 | 33.2 | 34.0 | 35.3 | +9% |

Each stage is above the one before it in every round of every game. **The store report bought less in the games than
§16's loop promised** — 2 to 6 per cent against a loop where it was a third of the time — because these scenes store
less often than one instruction in five, and because the processor is only 40 to 55 per cent of the thread. On the
microbenchmark the three together take §17's ALU loop from 7.65 to 5.44 ns an instruction and the memory loop from
9.99 to 6.51; with the target's observer attached the memory loop runs at 6.65, where it ran at 15.45 — the interface
call a store now costs what §16 said it would, about a seventh of a nanosecond an instruction on that loop.

| | at the profile (§2) | now | |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50 fps) | 7.3 fps | 24.1 fps | 3.3×, 48% of the console |
| Super Mario 64 (PAL, 50 fps) | 9.6 fps | 30.4 fps | 3.2×, 61% of the console |
| Wave Race 64 (NTSC, 60 fps) | 14.7 fps | 35.3 fps | 2.4×, 59% of the console |

The profile of this build, §2's method:

| share of the emulation thread | CPU and bus | VI | RDP | RSP | the core's loop |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 46.9% | 27.6% | 13.8% | 9.5% | 2.1% |
| Wave Race 64 | 53.0% | 13.8% | 18.2% | 12.7% | 2.2% |
| Ocarina of Time | 34.0% | 38.1% | 20.2% | 5.8% | 1.7% |

`Cpu.Step` is 19 to 24 per cent of the thread's exclusive time and `Execute` 6 to 8. **In Ocarina of Time the video
interface is the largest share again** — `Vi.Walk` alone 28.5 per cent — as each processor change makes the scan-out
a larger fraction of what is left; the RDP is a fifth. The processor's step is no longer the wall in that game, and is
half the thread in the other two.

**What is left in the step**, from §17's table: the MI line's compare, the count kept each step, the instruction
count and the return hook — together about 0.7 ns of a step that is now 5.4 on the ALU loop, and each of them is what makes
something else exact. The rest is the interpreter: the fetch's checks, the dispatch's two switches, the register
file's bounds checks, the tick. The lever for that remains a cached interpreter or a recompiler, and remains a
decision.

## 20. A cached interpreter, measured before it was built

**The question.** §19 left the interpreter itself as the wall in two games, and `Mars_Cpu.md` §1 had deferred a
predecoded cache to this phase. Before building one — and its invalidation, which every path that writes RDRAM would
have to serve — its ceiling was measured on §17's two loops, in a switch build never committed (its source is kept
with the speed tooling outside the repository). Five shapes, medians of five, ns an instruction:

| | ALU loop | memory loop |
| --- | --- | --- |
| the interpreter as it is | 5.43 | 6.51 |
| a decode memo, validated by the fetched word, one flat switch on a predecoded kind | 6.01 | 6.98 |
| the same memo, a delegate an entry | 5.86 | 6.47 |
| the cache trusted — no fetch, no validation — one flat switch | 4.84 | 6.10 |
| the cache trusted, a delegate an entry | 4.86 | 5.73 |
| the floor: each instruction known by its address, no lookup, no decode | 4.02 | 5.06 |

**What the rows say.**

- *Decoding costs about 1.4 to 1.5 ns an instruction*, a quarter of the step: that is the floor's distance from the
  interpreter, with the fetch still paid in both.
- *A memo that keeps the fetch and validates against it is slower than decoding.* The table load, the compare and one
  flat switch, or one delegate call, cost more than the two jump tables the JIT builds from the raw word. This is the
  shape that would have needed no invalidation, and it is ruled out by measurement.
- *Trusting the cache — the block cache's best case, with nothing charged for keeping it right — saves 0.4 to 0.8 ns*,
  6 to 12 per cent of the step. The lookup and its dispatch keep more than half of what decoding costs; a C# switch on a
  compact kind, or a delegate, is not much cheaper than a switch on the word.

**What it would be worth in a game.** The processor is 34 to 53 per cent of the thread (§19); 6 to 12 per cent of its
step is 2 to 6 per cent of a frame — before the cache's own costs: a test on every store to RDRAM, an invalidation on
every DMA and on the display processor's writes, the cheat patcher and a loaded state, and the memory the entries take.
§6's rule asks for a few per cent in more than one game, twice; the ceiling clears it barely and the floor of what
invalidation would cost is unmeasured. **Not built.** The decision stands on this measurement and can be reopened by a
better one: a block walked sequentially would save the per-instruction index (a fraction of a nanosecond), and no
shape here was tried on a loop with a branch mispredicted.

**What the floor is evidence of.** The 1.4 ns between the interpreter and knowing the instruction by its address is
recoverable only by code that *is* the instruction — a recompiled block — and the per-instruction bookkeeping the
step still carries (about 2 ns: the try region, the two program counters, the MI compare, the tick, the timer's
compare, the count) is recoverable only by executing several instructions between checks, which a block can do
exactly if it is cut where the next event falls. Both are the recompiler's shape, whether or not it emits machine code,
and that remains the decision it was in §8.

## 21. The scan-out's fetches, once a line

**What the three games ask of the scan-out.** A survey of the registers over each game's probe run — all three run
the same picture: 16-bit, anti-aliasing mode 0 with resampling, the dither filter on, a 320-pixel frame buffer
stepped across at a half so the raster's 640 columns each mix two source pixels. Ocarina of Time steps down at 0.83
(PAL, `VI_Y_SCALE` 0x354), so every output pixel mixes four samples; the other two step down at exactly one, so a
row's fraction is zero and only the pair across is mixed. Ocarina of Time and Super Mario 64 turn divot on; Wave Race
does not. A scan of the frame the probe reaches, timed alone on a bench that scans it two hundred times:

| | a scan, before | of the frame at §19's speed |
| --- | --- | --- |
| Ocarina of Time | 16.2 ms | 39% |
| Super Mario 64 | 11.1 ms | 34% |
| Wave Race 64 | 4.2 ms | 15% |

**Where a scan's time went.** §3 and §7 remembered each source pixel's *sample* once a row and once a line, and §5
stopped asking for samples a zero fraction would discard. What they left was underneath: a sample is the pixel and its
eight neighbours through the dither filter, or its six through the anti-aliasing filter, each neighbour fetched from
RDRAM and decoded afresh — so a source pixel was fetched about nine times a scan as somebody's neighbour, and with
divot on, `Across` sampled each pixel three times over, as the centre and as each neighbour's neighbour: twenty-seven
fetches and three filters a source pixel.

**Two changes, both memos of pure functions.**

- *The window.* The filters address the frame buffer by one linear index, a line's number times the width plus the
  column, and reach at most two lines up and two down from the pair a row reads, one line further where a neighbour
  wraps at a row's end. So a scan keeps a window of whole lines, fetched once each into one array contiguous in that
  index, sliding down as the walk does (thirty-two lines deep; a slide copies what it keeps). A neighbour is one
  subtraction and one load, and an index outside the window is fetched directly, which is what makes the window's
  extent a matter of speed alone. It is emptied at every scan, because RDRAM has changed.
- *The pre-divot memo.* `Remembered` keeps each row's samples after divot; a second array beside it keeps them before,
  under the same stamp, so the three samples a divot needs are made once each and shared with the neighbours'
  divots.

A first attempt memoised the fetch itself by its index, tagged and stamped: Ocarina of Time and Super Mario 64 gained,
Wave Race lost 17 per cent, because a memo's lookup costs what a sixteen-bit fetch does and only pays where a fetch is
repeated many more times than nine. Replaced by the window before it was measured in a game.

**What it bought, on the bench:**

| | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 16.2 ms | 7.0 ms | 2.3× |
| Super Mario 64 | 11.1 ms | 4.9 ms | 2.3× |
| Wave Race 64 | 4.2 ms | 3.7 ms | 1.1× |

**And in the games**, §12's method, three rounds, medians (fps):

| | before (`c953491`) | after | | of the console |
| --- | --- | --- | --- | --- |
| Ocarina of Time | 24.0 | 30.5 | +27% | 61% |
| Super Mario 64 | 30.4 | 36.2 | +19% | 72% |
| Wave Race 64 | 35.3 | 36.1 | +2% | 60% |

Every round of the two PAL games is above every round before it; Wave Race, whose scan was a seventh of its frame and
gained a tenth on the bench, is within its spread. Against the phase's opening profile that is 4.2×, 3.8× and 2.5×.

**Why it is exact.** The window holds `Fetch` of the same index over the same RDRAM and origin, which a scan does not
change; the lookup and the fill use one stored width, so the window is a contiguous range of indices whatever the
walk's width, and a stale width could only move the range. The pre-divot memo is stamped by the slot's line as
`Remembered` is. All 1,800 probe frames match, state and all.

**What catches a mistake in it.** Eight breakages: the window not emptied between scans, a slide misaligning what it
keeps, the lookup's index or the fill's index off by one, the pre-divot memo stamped wrongly or unbounded — each fails
the VI's reference cases, from 22 to 152 of them. Two survive, both equivalent: a window too narrow, since the fallback
fetches; and a window not resized for a narrower buffer, since the stored width governs both the fill and the lookup.
`A_narrower_frame_buffer_after_a_wider_one_scans_as_it_would_alone` was written for the second before its equivalence
was understood, and stays: it pins the property, which a different implementation could break.

**What is left in a scan.** The filters' arithmetic once a source pixel, the folded duplicates of a line where the
fetch bug applies, and the walk's own per-output-pixel work: the lookups, the mixes, gamma and four stores.

## 22. Where the phase stands after the scan-out

| | at the profile (§2) | now | |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50 fps) | 7.3 fps | 30.5 fps | 4.2×, 61% of the console |
| Super Mario 64 (PAL, 50 fps) | 9.6 fps | 36.2 fps | 3.8×, 72% of the console |
| Wave Race 64 (NTSC, 60 fps) | 14.7 fps | 36.1 fps | 2.5×, 60% of the console |

The profile of this build, §2's method:

| share of the emulation thread | CPU and bus | RDP | VI | RSP | the core's loop |
| --- | --- | --- | --- | --- | --- |
| Super Mario 64 | 56.8% | 16.6% | 12.6% | 10.9% | 3.0% |
| Wave Race 64 | 54.6% | 18.9% | 12.2% | 12.0% | 2.2% |
| Ocarina of Time | 43.5% | 25.8% | 21.2% | 7.0% | 2.3% |

**The processor is the largest share in every game again**, and the display processor is second in all three for the
first time — a quarter of Ocarina of Time, whose two-cycle pipeline, texel fetches and combiner are the exclusive
entries under it. The video interface's walk and dither filter are what §21 left. `SpInterface.Step` shows 5 to 11
per cent, which §15 found to be mostly the sampler's lean.

## 23. The display processor: decoded once, selected once, looked up once

**A bench for the display processor alone.** Its work arrives as a display list the signal processor builds during
the frame, so to time it apart from everything else one drawn frame of each game was recorded — the state of the
machine before the frame, and every command word the interface handed the processor during it — and the bench loads
the state, replays the words into the processor and times the replay, hashing RDRAM and its hidden bits afterwards so
that a change which alters a pixel is seen at once. Ocarina of Time and Wave Race draw on fewer fields than they
display, so the recorder advances to a field that draws. The replay is deterministic, and its noise about one per cent
on the best of fifty. Kept beside the speed tooling, outside the repository, since it holds images of commercial games.

| one drawn frame, before | words | ms |
| --- | --- | --- |
| Ocarina of Time (frame 303) | 5,186 | 36.5 |
| Super Mario 64 (frame 401) | 3,741 | 15.0 |
| Wave Race 64 (frame 302) | 15,013 | 16.1 |

At the field rates the games draw at, that is the quarter to a sixth of a frame the profile showed.

**Where a pixel's time went**, from the bench's own profile: the two draw loops and the two-cycle mode's first cycles
at 35 to 45 per cent exclusive, the texel path (`Texel`, `FetchTexel`, `FourTexels`) at 15 to 32, the combiner at 10 to
14. And three things every pixel did that it did not need to:

- *Every mode bit was re-extracted from the two 64-bit mode words on every use* — the cycle type, the dither modes, the
  blender's and combiner's selector structs, some forty shifts and masks a pixel — though the words change only when
  `SetOtherModes` or `SetCombine` lands.
- *The combiner chose each input's source once a channel*, through a switch on the selector and a switch on the
  channel: twenty-eight switches a cycle for four inputs and an alpha.
- *The dither pattern was chosen by two switches a pixel*, on modes fixed for the primitive.

**The changes.** Each mode bit is a field decoded when its word lands and after a loaded state, and Debug builds
recompute all forty-one from the words at every draw and throw if any disagrees — §10's and §18's discipline again.
The combiner selects each input's source colour once, and reads its three channels; a scalar input is broadcast to
all three, which is what the per-channel switch returned. The dither is one lookup in one of sixteen tables built
once, a table a pair of modes. All exact by construction, and all 1,800 probe frames match, state and all.

| one drawn frame | before | modes decoded once | + combiner, dither | so far |
| --- | --- | --- | --- | --- |
| Ocarina of Time | 36.5 ms | 34.0 | 30.9 | −15% |
| Super Mario 64 | 15.0 ms | 14.5 | 13.7 | −9% |
| Wave Race 64 | 16.1 ms | 16.0 | 15.5 | −4% |

The RDRAM hash after each replay is unchanged by every change.

**Measured and not kept.** A fourth change cached the texture tile's derived values — its clamp limits, mirror bits
and the fetch's format-and-size key — in the tile when a tile command lands, and read the 16-bit texture-memory,
colour and depth accesses as halfwords: all exact, and all inside the bench's noise (30.9, 13.6 and 15.3–16.0 ms
against 30.9, 13.7 and 15.5). The tile's properties were cheaper than they looked. Reverted; the test it needed, a
draw after a loaded state, stays, since it is what pins the modes' refresh below.

**What catches a mistake in it.** Twelve breakages, every Mars test in Debug, each of the four decode sites with the
verifier on and with it off:

| breakage | verifier on | verifier off |
| --- | --- | --- |
| `SetOtherModes` not decoding | 1,497 tests, by the verifier | 1,424, by their own assertions |
| `SetCombine` not decoding | 1,485 | 564 |
| the dither table not following the modes | 1,484 | 225 |
| a loaded state not refreshing | **nothing** | **nothing** — no test drew after a load; `A_loaded_state_draws_with_the_modes_it_carried` now does, and fails both ways |
| a scalar C input short of a channel | 148 tests | |
| C's texel-1 alpha read from texel 0 | 7 | |
| keying passing input B through | 5 | |
| the dither tables' alpha rule | 35 | |

**And in the games**, §12's method, three rounds, medians (fps):

| | before (`96beb4b`) | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 30.4 | 31.6 | +4%, every round |
| Super Mario 64 | 36.4 | 36.1 | within the spread |
| Wave Race 64 | 36.0 | 36.6 | +2%, every round |

A sixth of the display processor's time is a twenty-fifth of Ocarina of Time's frame, which is what it measures as.
**What is left in a pixel** is spread thin — the shade corrections, the depth compare, the memory reads and writes,
the blender's selectors, the texel fetch's switch and the four fetches of a filtered texel — each a few nanoseconds
and none redundant in the way the mode words, the combiner's switches and the dither were. The bench's profile after
these changes puts the two draw loops at 35 to 45 per cent exclusive and the texel path at 20 to 33, and neither share
is one thing.


## 24. The recompiler, and where the phase stands after it

`Mars_Recompiler.md` is the page; this is the entry in the phase's ledger. §20 had measured that the interpreter's fetch,
decode and per-instruction checks were recoverable only by code that *is* the instruction, and the day's last change
built that: runs of instructions compiled into one method each, calling the interpreter's own opcodes for anything that
can fault, branch or reach a device, validated against memory on every entry, and compiled on a thread of their own.

**Measured before it was built.** The three tickbench loops hand-compiled into exactly the methods the emitter was to
produce, dispatched exactly as the dispatcher was to dispatch them, ran bit-identical to the interpreter for sixty
million instructions each and took 2.05, 1.80 and 1.14 ns an instruction against the interpreter's 5.4 to 6.6
(`Mars_Recompiler.md` §7). That was the decision to write the emitter.

**What the first version of the emitter measured, and what it took to find out.** Super Mario 64 slower than the
interpreter, because its idle loop is a block of two instructions and every iteration paid a dispatch; then Wave Race
at two thirds of the interpreter's speed, with the profiler putting the time in the dispatcher's own frame. Two
explanations — the emitted code's size, the RSP's stores — were each ruled out by a measurement, and the third was the
JIT: a millisecond a block, paid on a dynamic method's first call and not in creating its delegate, 5.8 seconds of a
10.3-second run. A loop stays in its block now, and the JIT runs on a background thread, its cost forced there with
`RuntimeHelpers.PrepareDelegate`. The three are §3.4, §7 and §2.4 of the recompiler's page.

| | at the profile | after the RDP (§23) | now | |
| --- | --- | --- | --- | --- |
| Ocarina of Time (PAL, 50 fps) | 7.3 fps | 31.6 fps | 39.6 fps | 5.4×, 79% of the console |
| Super Mario 64 (PAL, 50 fps) | 9.6 fps | 36.1 fps | 49.3 fps | 5.1×, 99% of the console |
| Wave Race 64 (NTSC, 60 fps) | 14.7 fps | 36.6 fps | 45.6 fps | 3.1×, 76% of the console |

Medians of three interleaved rounds of 600 frames from boot, §12's method, against the commit before. The 600 frames
include every block's compilation on the compiler's thread and every interpreted run before its code arrived; a longer
session pays less, and the number a scene of gameplay would give remains unmeasured, as it has all phase. The probe's
whole state matched the §1 baseline on every frame of all three games, in a Debug build with the block verifier
(`Mars_Recompiler.md` §5) on after every instruction, and the Mars suite's 3,373 tests passed the same way.

**What remains in the processor.** The blocks keep registers in the array between instructions, end at every branch
but a loop onto their own start, and reach each other through the dispatcher; loads, stores and every coprocessor
instruction are calls into the interpreter. Each is a lever the recompiler's §8 names, and on 2026-09-19 all three were
measured (its §10–§12) and none kept: extension and chaining measured nothing, registers in locals lost 2 to 5 per cent,
because an entry's cost is the target's code streamed in cold and none of them lowers it. The interpreter itself is
unchanged and runs wherever a block cannot.

## 25. Ahead-of-time compilation, re-evaluated on Mars

*2026-09-19.* `Venus_CPU.md` §8's note had settled the question for the SNES core: Native AOT ran 10 to 12 per cent
slower than the tiered JIT, for want of a dynamic profile, and trimming left `StateSerializer` writing a twenty-byte
save state. The question was reopened for Mars, on the recompiler's build, because the earlier result was not trusted
and because the recompiler changes what the question means.

**Two things are called AOT.** *Native AOT* compiles the whole program to a native executable with no JIT in it, and
therefore no `System.Reflection.Emit`: the recompiler cannot exist under it, and `MarsCore.UseBlocks` now defaults to
`RuntimeFeature.IsDynamicCodeSupported` so that such a build runs the interpreter rather than throwing at the first
block. *ReadyToRun* precompiles the assemblies at publish and keeps the JIT, which still compiles the dynamic methods
and still re-compiles hot methods at the top tier with a profile; it is the form the recompiler can live with.

**Correctness first.** A harness printing the whole save state's length and hash, and the picture's, every hundred
frames, run on Super Mario 64 under the three:

| | state | picture |
| --- | --- | --- |
| JIT, blocks and interpreter alike | 6,433,277 bytes, `011F94E2…` | `038854E3…` |
| ReadyToRun, blocks | 6,433,277 bytes, `011F94E2…` | `038854E3…` |
| Native AOT, interpreter | **48 bytes** | `038854E3…` |

The emulation is the same under all three; under Native AOT the reflection the serializer walks finds nothing, as
before. That alone rules Native AOT out as a way of shipping Mars until the serializer is rewritten for it.

**Speed, §12's way**, four builds interleaved, three rounds of 600 frames from boot, medians:

| | JIT, blocks | ReadyToRun, blocks | JIT, interpreter | Native AOT, interpreter |
| --- | --- | --- | --- | --- |
| Ocarina of Time | 39.3 fps | 40.2 fps (+2%) | 31.1 fps | 24.9 fps (−20%) |
| Super Mario 64 | 49.0 | 49.5 (+1%) | 36.4 | 29.4 (−19%) |
| Wave Race 64 | 44.4 | 47.4 (+7%) | 36.2 | 29.1 (−20%) |

Native AOT is a fifth slower than the JIT running the same interpreter, worse than the SNES core's figure, and against
the JIT with blocks it is 37 per cent behind. The earlier result stands and is stronger here. ReadyToRun is a small
gain over these 600 frames, most of it plausibly the warm-up that a run from boot includes — the JIT compiling several
thousand of the core's own methods twice — and it is exact. It is not a lever of the kind this page keeps: it is a
publish setting, it costs nothing at runtime, and whether it is turned on for the shipped builds is a decision about
the publish (`project_publish_out_folder`), not about the core.

**What was not measured.** Startup time, which is ReadyToRun's usual reason and which the harness does not time;
memory; a Native AOT build with a static profile (`.mibc`), which would narrow the interpreter's gap and would not
give it blocks.

## 26. The gameplay profile

*2026-09-19.* Every number in this page to here came from the probe's 600 frames from boot — title screens, an
attract race, a castle's exterior — and §24 said so: the number a scene of gameplay would give was unmeasured all
phase. With the state hotkeys on the emulation thread (`EmuSen_Settings_Reference.md` §4.21a) three states were
saved in play, Ocarina of Time in the field, Wave Race 64 on the course, Super Mario 64 inside the castle, and a
harness that loads a state and times frames from it (`playbench`, in the speed tooling beside the states) gives the
phase its first measurement of what it is for.

| from the state, 600 frames, blocks | fps | of the console | at boot (§24) |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 29.0 | 58% | 39.6 |
| Wave Race 64 (NTSC, 60) | 29.6 | 49% | 45.6 |
| Super Mario 64 (PAL, 50) | 34.7 | 69% | 49.3 |

The second half of each run, once the scene's blocks are compiled: 6,600, 4,700 and 3,400 of them in the first six
hundred frames, taking 5.4, 4.9 and 2.8 seconds of the compiler's thread. The processor runs 1.86, 1.48 and 1.86
million instructions a frame, 99.8 per cent of them in blocks, 40, 50 and 85 instructions an entry. Gameplay is a
quarter to a third slower than the boot sequences, and none of the three reaches its console.

**Where a frame goes.** A sampled profile of 1,000 frames from each state, §2's method:

| share of the emulation thread | processor and its blocks | RSP | RDP | VI |
| --- | --- | --- | --- | --- |
| Ocarina of Time | 19% | 27% | 28% | 24% |
| Wave Race 64 | 16% | 35% | 37% | 11% |
| Super Mario 64 | 19% | 36% | 27% | 17% |

Two corrections to the analyser's own attribution lie behind that table, and both matter. The display processor
draws inside `SpInterface.Step` — the signal processor writes the command registers, the interface takes the list
at once (`Mars_Rdp.md` §2) — so that method's *inclusive* share is 56, 72 and 64 per cent of the three frames,
and it is not the RSP's. And its *exclusive* share, 12.6, 16.8 and 17.7 per cent, is the RSP's fetch-and-dispatch
loop with `Rsp.Step` inlined into it, which the analyser's component rule files under the processor because the
interface lives in the memory namespace. The table above puts the loop where it belongs, with the RSP's own
`Execute` and vector methods; the processor's share is what remains of `StepBlock` once the tick's callees are
taken out. **The signal and display processors together are two thirds of a frame of Wave Race and Super Mario 64
and over half of Ocarina of Time**, and the processor the recompiler serves is a fifth. The scan-out is a quarter
of Ocarina of Time, as §21 left it.

**Three smaller things the profile shows.** Every interrupt the processor takes is a managed exception:
`CheckInterrupts` throws, the dispatcher catches, and the runtime's dispatch is 1.0, 0.7 and 0.8 per cent of the
thread — the only exceptions in play, and a cost the interpreter's design chose for every fault alike. The RSP's
loop is entered once per processor instruction with a count of one, so its overhead is paid per step and not per
run. And the games draw fewer pictures than the interface scans: from these states Ocarina of Time and Wave Race
change their picture every third field and Super Mario 64 every second (§27's lockstep counts them), so two scans
in three or one in two walk a frame buffer the last scan already walked.

**What the frontend adds is outside this.** Mistress captures a rewind state every fourth frame, submits audio and
copies the picture; none of it is in the harness, and none of it is measured here.

**What this sets.** The recompiler's remaining levers were priced and rejected in §9 to §12 of its page; a fifth
of the frame is not where the next factor of two is. The scan-out reads the machine and writes only the raster,
so it can leave the emulation thread whole (§27). The display processor reads and writes RDRAM, and the argument
for taking it off the thread is one about which pages it touches and when anyone else looks — the next section
after §27's is that one. The RSP's loop and the interrupt's throw are smaller, and exact to fix.

## 27. The scan-out walked on another thread

*2026-09-19.* §26 put the video interface's scan-out at a quarter of Ocarina of Time's frame in play, a sixth of
Super Mario 64's and a ninth of Wave Race's, and §21 had already taken from it everything a single thread could:
what remained was the walk's own arithmetic over every output pixel. The scan-out reads RDRAM and the registers and
writes only the raster; it is the one component of the frame that changes nothing the machine can see. So it now
runs on a pool thread while the emulation thread runs the next frame. `Mars_Video.md` §2.7 is the design: the scan
split at the walk, the frame buffer's reachable lines captured before the walk starts, the argument for the
capture's reach and the throw that guards it, and the one-frame lag a frontend accepts for it.

**Measured**, §12's method turned on the mode rather than the build: one binary, the mode on and off alternately,
order rotated each round, three rounds of 600 frames from each of §26's states on a quiet machine, the second half
of each run (`ab-defer.sh`, `play-states3/ab-defer.txt`):

| from the state, second 300 frames | at once | deferred | | of the console |
| --- | --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 29.3 fps | 37.8 fps | +29% | 76% |
| Wave Race 64 (NTSC, 60) | 29.5 fps | 32.7 fps | +11% | 55% |
| Super Mario 64 (PAL, 50) | 34.3 fps | 41.3 fps | +20% | 83% |

Medians; the spread within a mode is under half a frame a second in every cell (29.3 to 29.3, 37.5 to 37.8; 29.4
to 29.6, 32.6 to 32.8; 34.2 to 34.9, 41.2 to 41.4), and every deferred round is above every round at once. The whole
runs, with the scene's compilation in them, move the same way: 27.9 to 35.5, 28.7 to 31.8, 33.6 to 40.4. The gains
are the scan-out's shares of §26's profile, less the capture — a copy of a quarter megabyte a frame — and the
join, which by the next frame's end has nothing to wait for.

**Exact.** The probe's 1,800 frames are unchanged, since the probe presents at once and the immediate path is the
same code over live memory. The Mars suite passes, 3,395 tests. From each of the three states, an immediate core
and a deferred one run 600 frames in lockstep with their states compared after every frame and the deferred picture
against the immediate picture of the frame before: identical throughout, 200, 200 and 300 distinct pictures. And
`MarsDeferredPresentationTests` walks every geometry the reference test scans from a capture after live memory was
overwritten under it.

**What it costs.** A frontend that turns the mode on shows each frame one field late — 17 to 20 ms — and
`EmuSen_Settings_Reference.md` §4.21b records that Mistress does. A thread from the pool for the walk, which on the
machine of §26 has fifteen more. The capture is a copy the machine did not make before; on the bench it is under
the noise.

**Where the phase stands.** Ocarina of Time and Super Mario 64 are within a quarter of their consoles in play;
Wave Race, whose scan-out was the smallest share, is at 55 per cent of its sixty. In all three the signal and
display processors are now two thirds or more of the emulation thread's frame (§26), and the display processor is
the larger of the two in Wave Race and Ocarina of Time. It reads and writes RDRAM, so it cannot leave the thread
the way the scan-out did, by capture; it can leave it only behind an argument about which pages each command touches
and a wait at every other access to such a page, and that argument, with the display processor's own reads and
writes checking it as they go, is what comes next.

## 28. The display processor's list on another thread

*2026-09-19.* §26 left the display processor as the largest or second-largest share of every game's frame in play,
and §27's closing paragraph said why it could not follow the scan-out off the thread by capture: it reads and writes
RDRAM, and so does everyone else. `Mars_Rdp.md` §2.6 is the design that takes it off anyway — the interface still
reads the list's words at the end write and still raises the full sync at the word that completes it, but hands the
words to a thread from the pool, and keeps a mark for every 4 KB page of RDRAM a batch can reach; every other reader
and writer of a marked page waits for the words that marked it. The argument's correctness is checked by the
processor itself, byte by byte, wherever verification is on.

**Measured.** Four configurations interleaved, order rotated, three rounds of 600 frames from each of §26's states
on a quiet machine, second halves; two builds, the commit before this one (`224b12a`) and this one
(`ab-builds.sh`, `play-states3/ab-builds.txt`):

| from the state, second 300 frames | before, at once | before, scan-out deferred (§27) | now, deferred, list at once | now, both threads | of the console |
| --- | --- | --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 27.7 fps | 35.4 | 34.8 | **37.7** | 75% |
| Wave Race 64 (NTSC, 60) | 27.8 | 31.2 | 30.5 | **41.3** | 69% |
| Super Mario 64 (PAL, 50) | 32.5 | 38.8 | 38.5 | **52.7** | 105% |

Medians; every cell's spread is within half a frame a second but one (Ocarina of Time deferred, 35.2 to 36.1),
and in every row each configuration's rounds lie wholly above the next slower one's. The machine as a whole ran
about five per cent slower in this session than in §27's — the same committed build reads 27.7 here against 29.3
there — which is the reason every comparison this page makes is between runs interleaved in one session and never
across sessions.

Three things the table separates. **The marks' cost on the path that does not use them**: the third column
against the second, 1 to 2 per cent, the compare that every load, store, fetch and bus access now makes against a
page's mark, cached in the processor and the bus as one array read (the first version reached it through two
fields and cost six). It is paid whether or not the list is threaded, and a build that ships without the thread
would pay it for nothing. **The thread's gain**: the fourth column against the third, +8, +35 and +37 per cent.
**The whole against §26**: +36, +49 and +62 per cent in play, from 27.7, 27.8 and 32.5. Super Mario 64's castle
runs past its console's fifty; Ocarina of Time is at three quarters; Wave Race at two thirds.

**Why Wave Race and Ocarina of Time do not reach their consoles**, from the interface's own counters
(`playbench` prints them). *The list is the floor.* On its thread the processor runs a word in 1.33 µs — Wave
Race's 11,700 words a frame are 15.6 ms of the thread's time, near the 16.7 ms a sixtieth of a second allows, and
the verifying compare adds 4 per cent to that. Whatever the emulation thread saves, the frame cannot end before the
list is drawn, because the game, told by the full sync that its frame is done, displays it, and the scan-out then
waits for the thread to finish it (`Mars_Rdp.md` §2.6, last paragraph). *The waits.* Wave Race's signal processor
DMAs from pages a texture load marked, its water's data and its textures sharing pages, and waits for the load's
words; Ocarina of Time reads its display list from the page after its depth buffer, and its processor writes to the
page before it, so both wait for the batch that cleared the depth buffer. In the first version those waits were for
*everything handed over so far* and lasted the tail of the frame; a batch now writes its own count over an
image's pages when it ends, and the waits are for the words that drew them. What remains is inherent to a page as
the unit: a finer unit would cost more on every access, and the marks are already the compare of the paragraph
above. *The emulation thread's own frame.* Wave Race's processor and signal processor are 18 ms of it (§26), above
the sixtieth on their own; the recompiler's fifth of the frame was priced in `Mars_Recompiler.md` §9–§12, and the
signal processor's loop is the larger of the two.

**Exact.** The probe's 1,800 frames are identical to the baseline with the list at once and with it on the thread,
the latter with verification on, so every byte the processor touched in those frames was in a marked page. The Mars
suite passes, 3,399 tests (`MarsThreadedRdpTests` are described in `Mars_Rdp.md` §7). From each of §26's states an
immediate core and a threaded one ran 600 frames in lockstep, verifying, with whole states and pictures compared
after every frame: identical throughout.

**Where the phase stands.** Two of the three games run at or near their consoles in play on this machine, with
three of its cores in use. What stands between Wave Race and its sixty is the display processor's own speed on its
thread and the signal processor's loop on the emulation thread, both of them the same code that ran before, now
each the floor of its own thread; and Ocarina of Time's remaining quarter is shared between the two threads' floors
and the waits its layout of memory makes.

## 29. Which pages, and which bytes, a mark speaks for

*2026-09-19, the same day as §28.* §28's closing paragraph named the waits its own counters showed and called the
page "inherent to a page as the unit". That was wrong twice over, and the counters said so plainly enough that the
two mistakes could be separated and priced.

**A mark did not say what the processor would do to the page.** A page a texture load reads from carried the same
mark as a page an image is drawn into, so a *reader* of that page waited for a load it has no order with: two
reads of the same bytes may happen either way round and agree. Wave Race's signal processor DMAs its microcode and
its data from pages its water textures are loaded from, and waited 3.6 ms a frame for them; Ocarina of Time's did
the same, and the interface's own reading of a display list waited too.

**And a mark named a page, not the bytes.** A page is four kilobytes; an image's rows or a load's bytes seldom fill
one, and a bystander sharing the page waited for the whole of it. Ocarina of Time keeps its display list in the page
after its depth buffer's last rows, which cost it 2.1 ms a frame in the list's own reading and 5.1 ms in its signal
processor's.

`Mars_Rdp.md` §2.6.1 is what replaced them: two marks a page, one for a writer and one for a reader, and behind
each mark the byte ranges that set it, so a waiter that finds its page marked can ask whether any range still
pending reaches the bytes it wants. A waiter that finds none is a **bystander**; it waits for nothing and leaves the
mark, which still speaks for the range's own bytes.

**Measured.** Two builds interleaved, order rotated, three rounds of 600 frames from each state, second halves;
the commit before this one against this one, and a fourth state the user saved outside the castle:

| from the state, second 300 frames | §28's build | with the split marks | of the console |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 40.5 fps | **52.2** | 104% |
| Wave Race 64 (NTSC, 60) | 44.6 | **51.5** | 86% |
| Super Mario 64, in the castle (PAL, 50) | 57.5 | **59.8** | 120% |
| Super Mario 64, outside it (PAL, 50) | 47.0 | **49.8** | 100% |

Medians of three; every round of the right column lies above every round of the left. **The machine was faster this
session than in §28's** — the same committed build reads 40.5 here where §28's table reads 37.7 — which is again
why this page compares only runs interleaved together, and why §28's absolute figures should not be read against
these.

**What each half bought, from the interface's counters.** Wave Race's waits were all reads of pages marked only for
writing, so the reader's mark alone removed them: its signal processor's 5,445 waits and 2,178 ms became none, and
it recorded no bystanders at all. Ocarina of Time's were bytes beside the marked ones, so the ranges removed them:
its 12,464 signal-processor waits and 2,731 list waits, 4,319 ms together, became none, and it counts 3,542
bystanders a frame — each one a wait the page would have cost and the range showed to be about other bytes.

**What still waits.** Ocarina of Time reads a page being drawn 399 times in 600 frames, 1.8 ms a frame, which is a
true read of the buffer the list is writing. Super Mario 64 outside the castle enters a block whose words span a
page the list draws into, 297 times for 4.2 ms a frame. Both are the list's own speed on its thread, not the marks.
The drainer's own rate is unchanged, 1.20 to 1.88 µs a word across the four states.

**Exact.** From each of the four states an immediate core and a threaded one ran in lockstep with verification on,
whole states and pictures compared after every frame: identical. The probe's frames are identical to the baseline
with the list at once and on the thread, and the Mars suite passes. The verifier found both of the defects in
`Mars_Rdp.md` §2.6.1's last two paragraphs, and found them within forty frames.

## 30. The signal processor's vector unit, eight elements at a time

*2026-09-19.* §26 left the signal processor as the largest share of the emulation thread's frame once the display
processor and the scan-out had gone to their own threads, 27 per cent of Ocarina of Time's frame and 35 to 36 of the
other two. Its arithmetic is written element by element because the specification is, not because it needs to be,
and the host has 256-bit vectors. `Mars_RspVector.md` §14 is the same arithmetic in host vectors, with the
element-by-element unit kept as the reference it is graded against, and §14.1 has the table: **+4, +19 and +22 per
cent** in play, Wave Race 64 reaching its console's sixty from this change alone.

**Where the three levers of this day leave the four states.** All three on, interleaved in one session, second
halves of 600 frames from each state:

| | at boot (§24) | in play, §26 | now | of the console |
| --- | --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 39.6 | 29.0 | 49.5 | 99% |
| Wave Race 64 (NTSC, 60) | 45.6 | 29.6 | 60.1 | 100% |
| Super Mario 64, in the castle (PAL, 50) | 49.3 | 34.7 | 72.9 | 146% |
| Super Mario 64, outside it (PAL, 50) | — | — | 54.8 | 110% |

That is from 29.0, 29.6 and 34.7 fps at the start of the day to 49.5, 60.1 and 72.9, a factor of 1.7 to 2.1, with
every frame of every run identical to the frame the single-threaded, element-by-element build produces. The machine
is a sixteen-thread desktop part and three of its cores are in use.

**A fourth state, and a negative result.** The user saved a fourth state outside the castle during this work, and
§30's change is worth **nothing** there: 54.5 fps with the loops and 54.8 with the vectors, three rounds each whose
spreads overlap entirely. The reason is in the interface's counters rather than in the vector unit. That scene hands
the display processor 8,058 words a frame and the drainer runs them at 1.88 µs each, 15.1 ms of a 20 ms frame, so
the thread that bounds the frame is not the one the vector unit is on. It is recorded here because a change that
helps three scenes and not a fourth is the normal case, and because the counter that explains it is the same one
that would have predicted it.

**What each state's remaining floor is.** Ocarina of Time is the one that does not reach its console, and it is the
only one of the four whose frame is bounded by the display processor's own speed on its thread rather than by the
emulation thread: 1.65 µs a word over roughly seven thousand words a frame, which is `Mars_Rdp.md` §2.6's last
paragraph, plus the scan-out's quarter of a frame that §21 measured and §27 moved rather than shortened. Nothing in
§29 or §30 touches either. The signal processor's remaining share is its fetch-and-dispatch loop, entered once per
processor instruction (§26), and its vector loads and stores, which are byte loops and which the references also
keep as byte loops; neither has been priced.

## 31. The scan the machine had already been shown

*2026-09-19.* §26 recorded that the games draw on every second or third field while the interface scans on every
one, and called the repeated scans a thing to look at. `Mars_Video.md` §2.8 is that: a deferred scan whose geometry
and whose bytes both equal the last walk's is not walked, because the walk is a pure function of those two and the
raster it would write is the one already there.

**It is the one change of this day that no reference suggested.** The scan-out study behind it read angrylion, its
parallel-n64 fork, parallel-rdp, the MiSTer register-transfer code and mupen64plus-core, and none of the five
declines to scan for any reason but the registers being degenerate — a zero origin, two blank fields, an invalid
width. None looks at whether the frame buffer changed. Two of them *cannot*: angrylion reseeds the gamma dither's
noise from a field counter and parallel-rdp from a frame count, so a scan over identical bytes genuinely produces a
different picture in both. Mars keeps no such counter (`Mars_VideoPasses.md`), which is what makes the skip exact
here and not there. The fork has a measured campaign against the cost of a scan instead — its census found 97 to
100 per cent of pixels fully covered and filtering nothing, and it skips the vertical resample at a zero fraction —
and §5 and §21 had already taken both of those.

**Measured**: +4.6 per cent in Ocarina of Time and +1.3 in Wave Race, both outside their rounds' spread, and
nothing in either Super Mario 64 state, which are bound by the display processor's thread. `Mars_Video.md` §2.8 has
the table and the first version that cost what it saved.

**Where the four states stand at the end of the day.** All three of this day's kept changes on, interleaved, second
halves of 600 frames:

| | in play, §26 | now | of the console |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 29.0 | 52.2 | 104% |
| Wave Race 64 (NTSC, 60) | 29.6 | 60.4 | 101% |
| Super Mario 64, in the castle (PAL, 50) | 34.7 | 73.0 | 146% |
| Super Mario 64, outside it (PAL, 50) | — | 55.6 | 111% |

**All four are at or past their consoles**, from 1.8 to 2.1 times what they ran at when the day began, with every
frame identical to the frame the single-threaded build produces.

**What the next factor would have to come from.** Three of the four are now bound by the display processor's own
speed on its one thread, which `Mars_Rdp.md` §2.6 leaves at 1.2 to 1.9 µs a word. The feasibility study for
splitting the rasteriser across scanlines (`Mars_Rdp.md` §10) says Mars can do it bit-identically, and that its span
tables are already precomputed for every row with no thread term in them — the architecture angrylion lacks and
parallel-rdp needed a second pass to build. Two narrow mode cases block it and both are detectable from the mode
bits. Thirteen of this machine's sixteen threads are idle.

## 32. Where the time goes once three components are on three threads, and two things that measured nothing

*2026-09-19.* §26's profile was taken before the scan-out, the display processor's list and the signal processor's
vector unit moved or changed, so it no longer describes the machine. A fresh sampled profile of every thread, 600
frames from two of the gameplay states, says something different from every profile this page has taken so far:
**the emulation thread is mostly idle.**

| share of the emulation thread | Ocarina of Time | Wave Race 64 |
| --- | --- | --- |
| spin-waiting (see below) | 43% | 63% |
| waiting for the display processor's thread by name | 15% | 6% |
| sleeping, or joining the scan-out | 7% | 2% |
| the processor, its blocks and the signal processor | ~22% | ~16% |

**The largest entry is not what it looks like.** The sampler names it `Thread.PollGCWorker`, the garbage-collection
poll the compiler inserts into loops that call nothing, which is where a spin loop's time lands. It is not
collection: over those runs the collector paused for 72 ms of 12.6 s and 56 ms of 10.5 s, **0.6 and 0.5 per cent**,
across 9 and 7 generation-zero collections, and the program allocates about 12 MB a second, almost none of it per
frame. Garbage collection is not a cost here and this page should stop wondering about it.

**What the thread is actually waiting for**, from the interface's own counters, is a very small number of very long
waits:

| | waits in 600 frames | total | each |
| --- | --- | --- | --- |
| Ocarina of Time, the signal processor's DMA | 218 | 2,895 ms | 13.3 ms |
| Ocarina of Time, the processor's load | 398 | 716 ms | 1.8 ms |
| Wave Race 64, the scan-out's capture | 133 | 1,762 ms | 13.2 ms |

Both of the thirteen-millisecond waits are genuine read-after-write dependencies on what the display processor
writes — Wave Race's scan-out waits for the buffer the full sync just told the game was finished (`Mars_Rdp.md`
§2.6's last paragraph), and Ocarina of Time's signal processor reads back a page the processor draws into. They cost
a whole frame each **because a mark names the batch's last word**: a reader of bytes written early in a batch waits
for the entire batch to drain. Marking each primitive's own rows with its own word, rather than the whole image
extent with the batch's tail, would bound each wait by the primitives that actually cover the rows being read. That
is the best-targeted lever this page can now name, and it is not built.

**And the two pool threads are idle half the time.** The display processor's drain and the scan-out's walk share the
same thread-pool threads, each busy 24 to 33 per cent of the run. So the work is not overlapping, it is
ping-ponging: the emulation thread blocks, the drain runs, the drain empties, the emulation thread runs. That is the
same finding from the other side.

### 32.1 Measured and not kept: padding the two counts apart, and the verifier's per-word write

Two small changes were built on the strength of the profile, proven exact by the threaded, deferred and save-state
tests, and timed interleaved against `c70a9ee`, three rounds of 600 frames from each of the four states.

*The counts.* `_issued` and `_completed` are declared adjacent, one written by each thread and read by the other,
and `_completed` is written **once per command word** — seven thousand times a frame — while the emulation thread's
spin loop reads it. That is a textbook false-sharing hazard, so each was given a cache line of its own with an
explicit-layout 128-byte struct. *The verifier's write.* The drain loop set `_runningWord` on every word, a value
only the byte-level verifier reads, in every build.

| second 300 frames, medians of three | `c70a9ee` | both changes |
| --- | --- | --- |
| Ocarina of Time | 52.1 fps | 51.8 |
| Wave Race 64 | 60.7 | 60.4 |
| Super Mario 64, in the castle | 73.0 | 72.4 |
| Super Mario 64, outside it | 55.4 | 55.6 |

Every row overlaps and three of the four medians are *lower*. **Neither is kept.** The reason the padding buys
nothing is in the numbers above: a word costs the display processor 1.15 to 1.57 µs of real work, and a contended
cache line costs a fraction of that, so the line's ping-pong was never the cost of the loop it sits in. The
128-byte structs also add 256 bytes to the interface and push its other hot fields apart, which is the likeliest
reason for the small loss. The lesson worth keeping is the general one: a hazard that is real in the abstract is
still only worth what the loop around it leaves it room to cost.

## 33. What the runtime offers, and four of its answers measured

*2026-09-19.* §25 settled ahead-of-time compilation and `project_dotnet11_measured` settled the runtime version, so
this section is about what is left: the techniques the .NET runtime's own library uses in its hottest loops. Four of
them could be checked without writing anything, and all four came back negative. A negative that costs one command
is worth more than a plausible lever that costs a day.

**The no-optimisation cliff is not being hit, and this is the one that mattered.** The compiler compiles an entire
method with *no* optimisation — no common-subexpression elimination, no range-check removal, essentially no register
allocation — if it crosses any one of five hard-coded limits: 60,000 bytes of intermediate language, 20,000
instructions, 2,000 basic blocks, 2,000 locals, or 8,000 local references. A retail runtime hard-codes them, so they
cannot be raised. This matters here more than in ordinary code because **an emitted block is compiled exactly once,
at first call, and never re-compiled** — a dynamic method is excluded from tiered compilation altogether, so it gets
no second chance and no profile. `DOTNET_JitDisasmSummary=1` over a 200-frame run reports every method the runtime
compiled: **6,658 of them, and not one at `MinOpts`.** All 3,613 emitted blocks read `FullOpts`. That also retires a
confound in `Mars_Recompiler.md` §9 to §12, two of whose rejected shapes — registers in locals, and extended blocks —
push exactly the local count and the block count that trip this. They were slower for their own reasons, not because
they fell off a cliff. The check costs one command and belongs in the recompiler's ratchet.

**Garbage collection is not a cost, with numbers.** §32 has them: 72 ms and 56 ms of pause in runs of 12.6 s and
10.5 s, 0.5 to 0.6 per cent, over 9 and 7 generation-zero collections. Every server-mode, heap-count, affinity and
dynamic-adaptation knob is therefore measuring noise here, and the page can stop asking. The one mechanism that
would matter — allocating RDRAM on the pinned object heap — only applies if raw pointers into it are ever cached,
which they are not.

**AVX-512 is worth nothing to the vector unit of §30**, which the runtime lets one measure without a code change.
Wave Race, 600 frames, second halves:

| | fps |
| --- | --- |
| default | 60.9 |
| 512-bit vector forms disabled | 60.9 |
| AVX-512 disabled entirely | 60.7 |

That is the expected answer for this part — Zen 4 executes 512-bit operations over 256-bit datapaths, so per-element
throughput matches AVX2 — and §14 of `Mars_RspVector.md` never asked for 512-bit lanes: the unit is eight sixteen-bit
elements, which is 128 bits. The useful consequence is for the weak-machine effort
(`project_optimization_for_weak_machines`): **the vector unit does not depend on AVX-512 and will not regress on a
part that lacks it.**

**The defensive-copy audit is almost empty.** A struct that is not declared read-only makes a defensive copy at every
member access when it is reached through a read-only reference, which for a per-pixel struct would be one copy per
access. Every struct Mars passes that way is already read-only except `SoftFloat`, in two floating-point comparison
helpers. Real, mechanical, and in code that is not hot.

**What the runtime's own library does that Mars has not tried**, recorded so that it is not re-derived. Two things,
and both aim at the display processor's thread rather than the emulation thread, which is the only place §32 leaves
room to win:

- *The wait between the threads.* `SpinWait.SpinOnce(-1)` disables the millisecond sleep, which is the important
  part, but after ten turns it alternates spinning with yielding and makes every fifth yield a `Sleep(0)` — so a long
  wait settles into a loop that is half system calls. With three busy threads on sixteen, a bounded spin on the pause
  instruction and then a single blocking wait would be strictly cheaper. And the drain is started through the thread
  pool, which puts the pool's own dispatch latency on the path every time a drain begins, and whose injection
  controller adds threads about once every 500 ms when its workers are busy. A dedicated thread parked on an event
  removes both.
- *The pixel loop's modes.* The runtime's library monomorphises hot loops by passing a mode as a **value-type type
  parameter**, which the compiler instantiates exactly rather than sharing, so the mode's constant members inline to
  constants and the untaken branches are eliminated with everything that fed them. That is the C# form of the
  specialised span loops angrylion measured at 3.5 to 8 per cent (`Mars_Rdp.md`'s study), and §23's work already
  reduced Mars's per-pixel mode tests to predictable branches on hoisted booleans, which is most of the way there.
  The risk to price is code size: each instantiation is compiled separately, and enough of them could reach the
  cliff this section just proved Mars is clear of.

**What was advised against, and is not being done.** Suppressing the zeroing of locals and stack allocations is worth
nothing to a method that allocates a small span per call, by its own author's measurements, and it would turn a
read-before-write from deterministic zeros into whatever the last frame left on the stack. For a core whose whole
claim is bit-identical output graded by differential runs, that trades nothing for a class of defect that reproduces
differently every run. Not adopted.

## 34. The frontend's path, the transfer that was not the cost, and what the interface's counters say

*2026-09-19.* §31 left the four states at or past their consoles in the play harness, and Mistress does not run them
so. This section measures what the frontend adds — the rewind buffer's capture every fourth frame, the audio dequeue
and the picture copy, which `frontbench` in the speed tooling runs after each frame as Mistress does — and then two
things the day's profile-reading turned up.

**The frontend's baseline.** Second halves of 600 frames from each state, two rounds, the capture on and off:

| | buffer off | buffer on | of the console, buffer on |
| --- | --- | --- | --- |
| Ocarina of Time (50 Hz) | 50.6 / 50.1 | 49.1 / 48.8 | 98% |
| Wave Race 64 (60 Hz) | 60.9 / 61.4 | 54.0 / 53.5 | 90% |
| Super Mario 64, in the castle (50 Hz) | 73.3 / 75.0 | 69.1 / 70.0 | 139% |
| Super Mario 64, outside it (50 Hz) | 55.4 / 55.8 | 52.0 / 51.1 | 103% |

The picture and the audio are free. The capture is not: a join of the display processor's thread of 7.0 ms in Wave
Race and 4.9 ms outside the castle (nothing in Ocarina and the castle, whose thread has caught up by the frame's
end), then 3.3 ms of writing the 6.4 MB state, copying it and encoding the delta, on the emulation thread. Mistress
therefore ran Wave Race at 90 per cent of its console with the buffer on. What was done about it is §34.1.

**The transfer that was not the cost.** §32's per-thread profile put 47.6 per cent of Wave Race's emulation thread
under the signal processor's DMA transfer with the garbage-collection poll as the leaf, and the transfer copies a
byte at a time through the bus — a 32-bit read with the page-mark test for each byte read, a read and a write for
each byte written. That is the profile a byte loop would give. The loop was replaced by a bulk copy for rows inside
RDRAM, exact by construction (the same bytes, and the write counter advanced by the same count so the blocks' exit
is unchanged), and timed interleaved against `c70a9ee`: three rounds of 600 frames from each of the four states,
second halves, medians.

| | `c70a9ee` | bulk copy |
| --- | --- | --- |
| Ocarina of Time | 52.1 | 52.7 |
| Wave Race 64 | 60.0 | 60.0 |
| Super Mario 64, in the castle | 73.6 | 75.2 |
| Super Mario 64, outside it | 56.5 | 55.8 |

Every row's rounds overlap, and the change is **not kept**, by the phase's rule. The finding is the attribution. The
sampler suspends the runtime to take each sample, and a thread answers the suspension at the next poll it passes:
the poll a spin loop calls, the poll after a native call returns, the poll the compiler places in a loop that calls
nothing. The leaf is therefore always the poll, and the frame above it is whichever method the poll was *inlined*
into — for a wait inlined through four small methods, the caller of the wait. The 47.6 per cent was the transfer's
wait on the marks, not its copy loop. §32's reading of the same leaf as spinning was right; its attribution to
methods by name, at this depth, is not to be trusted, and the interface's own counters are what to read instead.

**What the interface's counters say.** From the play harness on `c70a9ee`, 600 frames from each state:

| | waits | each | of the run |
| --- | --- | --- | --- |
| Wave Race 64: the scan-out's capture, on the two frame buffers | 133 | 12–25 ms | 18% |
| Ocarina of Time: the processor's loads | 399 | 2.7 ms | 8% |
| Ocarina of Time: the signal processor's DMA writing a texture page | a handful | 40–46 ms | — |

Wave Race's is the whole of its waiting: the scan-out captures the buffer the game has just swapped to, and the
thread has 12 to 25 thousand words of it still to draw. No finer mark shortens that, because the scan-out needs
every byte of the buffer; only a faster rasteriser does, and the drainer's own count says the same from the other
side — 7.03 million words in 600 frames at 1.17 µs each is 13.7 ms of drawing in a 16.7 ms frame. Ocarina's 399
loads are the case per-primitive marks (§32) would shorten, and its DMA waits are the thread two frames behind the
machine, which the ring allows and a snapshot's tail has to fit (`Mars_Rdp.md` §2.7).

### 34.1 The snapshot: the capture without the join, measured

Built as `Mars_Rdp.md` §2.7 and the rewind manual's §1.8 describe: the buffer asks the core for a snapshot, Mars
holds its thread between two words and writes the words it has not run in a fixed 32k-word tail, and the buffer
encodes the delta on a pool thread. Three rounds of 600 frames from each state, the capture every fourth frame, the
old join and the snapshot interleaved on the same build; second halves, medians, the rounds in brackets:

| | join before the capture | snapshot | per capture, join → snapshot |
| --- | --- | --- | --- |
| Ocarina of Time | 50.9 (50.9–51.7) | 51.3 (50.9–52.1) | 1.4 → 1.4 ms |
| Wave Race 64 | 55.2 (54.9–55.5) | **58.7** (58.5–59.5) | 8.3 → 1.7 ms |
| Super Mario 64, in the castle | 71.4 (70.7–72.4) | 72.8 (68.8–73.6) | 1.5 → 1.4 ms |
| Super Mario 64, outside it | 54.0 (53.5–54.4) | **56.4** (55.5–56.6) | 6.3 → 1.5 ms |

The two states whose thread was behind at the frame's end gain, and their rounds do not overlap; the two whose
thread had caught up are unchanged, as they should be. The capture now costs the emulation thread 1.4 to 1.7 ms
against 3.3 before, which is the encode leaving the thread. Wave Race is still short of its console — 98 per cent
with the buffer on, against 90 — and the reason is in the frames themselves: they grew by 0.6 ms when the join went,
because the thread's backlog moved from the join to the scan-out's wait. That is §34's finding again: the frame is
the rasteriser's, and the buffer's cost was only ever the part of the wait that the join added on top of it.

**Two defects on the way, recorded.** The first version's tail was as long as the backlog, so the state's length
changed at every capture, and the buffer, which requires equal lengths for a delta, restarted its chain each time:
the bench showed a 10 ms capture and no history at all. The tail is fixed at 32k words, 256 KB, and a backlog longer
than it is waited for down to it. The second was the bench: its own second, joined state write after each capture
put back the join the snapshot had removed, and the first comparison read as equal; that write is now opt-in. Both
are the kind of thing the bench catches and reasoning does not.

**What it costs in history.** A delta now carries the tail's changed words as well as the frame's — about 400 to
600 KB a capture in Wave Race, against the buffer's 96 MB, which is forty seconds of rewind rather than minutes. The
interval is the knob, and it is a product decision (`EmuSen_Rewind_And_FastForward.md` §1.5).

## 35. The rasteriser split across scanlines, and where the bound moves to

*2026-09-20.* `Mars_Rdp.md` §10.2 priced the split and §2.8 records what was built: N complete processors reading the
same words, each shading the rows whose number modulo N is its own, a barrier at an image change, a load from drawn
bytes and a primitive whose carry crosses rows, and the read at the width made by the next row's owner. This section
is what it measures.

**The rasteriser alone.** The three recorded frames of §7's bench, replayed thirty times from their states, best of
thirty; the RDRAM and hidden-bit hashes were the same at every count.

| best, ms | direct | 1 worker | 2 | 4 |
| --- | --- | --- | --- | --- |
| Ocarina of Time, frame 302 (5,186 words) | 31.4 | 31.8 | 16.7 | 10.4 |
| Super Mario 64, frame 400 (3,741 words) | 14.0 | 14.0 | 8.0 | 4.8 |
| Wave Race 64, frame 301 (15,013 words) | 16.4 | 16.0 | 9.2 | 6.6 |

That is 1.8 to 1.9× at two and 2.5 to 3.0× at four, against §10.2's estimate of 1.7 and 2.5 from a fifth of the
work being replayed rather than divided. None of these frames has a primitive drawn alone or a load waited for; the
barriers are the image changes, three to nine a frame.

**In play.** The four gameplay states through the play harness, deferred scan-out and the threaded list, one build
in four modes interleaved, three rounds of 600 frames; second halves, medians, the rounds' range in brackets:

| fps | 1 worker | 2 | 3 | 4 | of the console at 4 |
| --- | --- | --- | --- | --- | --- |
| Ocarina of Time (50 Hz) | 51.1 (50.7–51.8) | 67.2 (67.1–67.4) | 71.8 (71.2–72.0) | 74.1 (73.4–74.2) | 148% |
| Wave Race 64 (60 Hz) | 60.2 (60.0–60.9) | 70.7 (70.5–72.1) | 70.1 (65.8–73.6) | 73.2 (72.4–73.7) | 122% |
| Super Mario 64, in the castle (50 Hz) | 73.5 (73.2–73.6) | 79.4 (78.6–79.7) | 80.6 (79.2–80.8) | 80.3 (79.9–80.8) | 161% |
| Super Mario 64, outside it (50 Hz) | 54.8 (54.2–55.7) | 80.7 (80.7–80.9) | 84.6 (82.2–85.4) | 84.8 (84.0–84.9) | 170% |

Every state is past its console at two workers and the second pair of workers adds a tenth or less, where the
rasteriser alone gained a further third. **The bound has moved.** With the list drawn in half the time, the
emulation thread — the processor, its blocks and the signal processor's loop, which §32 found idle two thirds of
the time — is what a frame waits for in three of the four states; Wave Race's 73 fps is 13.7 ms a frame, which is
close to what §26 measured that thread doing on its own. §33's advice about the signal processor's loop, recorded
and not taken because the thread was idle, is worth taking now, and `Mars_Rsp.md` §10 has it.

**What the split costs when it is off.** The stamps are a few stores a row and one a pixel under a mode predicate,
and the coverage buffer's stamp is filled beside it; the classification is two compares a primitive. The one-worker
column above is the split's code at one worker, and it is within the rounds of §34.1's numbers for the same build
without it, so the cost is below what the harness resolves.

**Graded.** The probe's 1,800 frames from boot, threaded, with the byte-level verifier on, are identical to the
baseline at four workers and at two, save state and all; the Mars and Common suites pass, 3,505 tests, with the
verifier on in Debug.

**What is not measured.** A weak machine, where the workers share cores with the emulation thread and the scan-out
(`project_optimization_for_weak_machines`); a scene with many primitives drawn alone, which none of the four states
or three frames has; and the frontend's own path, §34's, with the workers on. The count the frontend uses is a
product setting and is not this section's to decide.

## 36. The emulation thread, once it is the bound: an honest profile, and what the sampler had been measuring

*2026-09-20.* §35 left the emulation thread as what a frame waits for, and every profile of it in this page was
taken with `dotnet-trace`'s sampled-thread-time provider. Two things this section establishes before anything is
optimised: that sampler cannot attribute this thread's time, and what does.

**The sampler stops threads at safepoints.** The runtime's sample profiler suspends the runtime to walk stacks, and
a thread is suspended where it next reaches a safepoint: a garbage-collection poll at a loop's back edge, or a
return hijacked on the way out of a call. The sample is charged to the frame at that point, which is the loop the
thread passes through most often, not where its time went. §32 read `PollGCWorker` under the signal processor's
DMA transfer as a wait inlined into it, and §34 read it as the transfer's own loop and tested a bulk copy, which
measured nothing — and the test was blind, because the thread was idle two thirds of the time and nothing on it
could shorten a frame. With four rasteriser workers the same sampler charges 56 to 65 per cent of the thread to the
transfer. A cycle counter around the transfer (`SpInterface.TransferTicks`, printed by the play harness) says
**0.95 ms a frame in Wave Race and 1.12 in Ocarina of Time**, seven per cent, for about two thousand rows and 190 KB
a frame. The sampler was wrong by a factor of eight, in the direction of the loop with the most polls.

**The instrument that replaced it.** `ipsample.py` in the speed tooling spawns the harness, and at each interval stops
its main thread with `ptrace`, reads the instruction pointer, resumes it, and attributes the address through the
runtime's perf map; the stop is an interrupt, not a poll, so the sample is where the thread was. Linux `perf` would do
the same and is not installed; Yama's default scope permits tracing one's own child. Twenty thousand samples of Wave
Race's emulation thread and, by component:

| share of the emulation thread, four workers | Wave Race 64 | Ocarina of Time |
| --- | --- | --- |
| the signal processor's vector unit | 30% | 21% |
| the signal processor's scalar half, its dispatch and the per-instruction loop | 24% | 19% |
| the processor's opcodes a block calls the interpreter for | 17% | 23% |
| the processor's compiled blocks | 13% | 18% |
| the processor's software floating point | 4% | 4% |
| the signal processor's DMA | 3.5% | 3.3% |
| the display processor's interface, mostly the kick a word | 3% | 2% |
| the runtime and native code | 2% | 8% |

The signal processor is half the thread in Wave Race and two fifths in Ocarina of Time, and within it the
per-instruction loop — the interface's loop with the step inlined into it, called once per processor instruction with
a count of one — is eleven per cent of the thread by itself. The vector unit's small helpers (the 48-bit wrap, the
accumulator's low half, the clamps, the accumulator's load) appear as methods of their own in the samples, which
means the compiler did not inline them into the fifty-case dispatcher, whose inlining budget they exhausted; each
vector operation therefore made calls that pass 256-bit vectors through memory. Those two, and the transfer's byte
loop, are the three exact changes §36.1 measures. The interpreted opcodes and the blocks are the recompiler's own
levers (`Mars_Recompiler.md` §9 to §12) and are not touched here.

### 36.1 Three exact changes on that thread, measured

The per-instruction loop restructured (`Mars_Rsp.md` §10.1: the halt tested once, the loop counting down, the fetch
without the span's check); the vector unit's small helpers marked for aggressive inlining, twenty of them, so that
the dispatcher's exhausted budget no longer leaves them as calls passing 256-bit vectors through memory; and the
transfer's rows inside RDRAM copied whole, §34's change, after the same wait for the marks. All three are exact by
construction, and the suites and the probe grade them. Interleaved against the split's commit, four workers, four
states, three rounds of 600 frames, second halves, medians with the rounds' range:

| fps | before | after | |
| --- | --- | --- | --- |
| Ocarina of Time | 74.6 (74.4–75.0) | 80.3 (79.5–80.3) | +7.6% |
| Wave Race 64 | 73.2 (71.4–75.7) | 84.0 (82.1–86.1) | +14.8% |
| Super Mario 64, in the castle | 80.0 (78.4–80.3) | 92.2 (91.8–92.4) | +15.3% |
| Super Mario 64, outside it | 84.4 (83.6–86.5) | 96.6 (96.6–97.0) | +14.5% |

No round overlaps. The sampler of §36, run again on the changed build, says which of the three did what: in Wave
Race the vector unit's share of the thread fell from 30 to 25 per cent and its helpers no longer appear as methods
of their own; the DMA fell from 3.5 to 0.3 per cent; and the per-instruction loop kept its twelve per cent of self
time, so the restructure bought the least of the three — its remaining cost is the loop's own shape, one call and
one fetch-and-dispatch per processor instruction, which §10.1 records as still open. The thread is now the
signal processor's for half its time in Wave Race (28 per cent scalar and dispatch, 25 per cent vector) and the
processor's for a third (20 per cent interpreted opcodes, 15 per cent blocks); in Ocarina of Time the shares are
the other way about, and a tenth of that thread is native code the map cannot name, most likely the exceptions the
interrupts throw and the runtime's own copies.

**A measurement that had to be thrown away.** The first A/B of these changes read +42 to +74 per cent, and it was
wrong: the head harness's program had been copied from the live one, referenced counters the head commit lacks,
failed to build, and its bin still held the binary from before the split, which ran silently as "head". It was
caught because its numbers matched the one-worker column of §35 to the decimal. The harness scripts cannot tell a
stale binary from a fresh one; the harness's own last line, which names its worker count, can, and the tooling's
README now says to read it.

**What is next on this thread.** The recompiler's share — the opcodes a block hands the interpreter, and the blocks
themselves — is a third to two fifths of the thread, and `Mars_Recompiler.md` §9 to §12 priced its shapes with a
sampler that this section has shown cannot attribute time on this thread. Those measurements were interleaved
timings, which stand; the attributions behind them were the sampler's, which do not. The signal processor's
dispatch, and one program counter in place of two, are the other half.
