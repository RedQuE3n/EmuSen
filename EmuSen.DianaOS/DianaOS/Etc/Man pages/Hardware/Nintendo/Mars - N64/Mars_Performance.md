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
each frame hashes the presented picture, all of RDRAM and the audio drained, beside the cycle count.

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
