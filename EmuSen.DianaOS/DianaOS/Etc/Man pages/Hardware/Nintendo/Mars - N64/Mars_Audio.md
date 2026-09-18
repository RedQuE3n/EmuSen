# Mars — the audio interface

*Phase E's audio slice, landed 2026-09-18. `EmuSen/Cores/Nintendo/Mars - N64/Memory/AiInterface.cs`, stepped from
`MemoryBus.Tick`; tests in `MarsAudioTests`, and the game probe (`Mars_GameProbe.md`) for the end-to-end check.*

---

## 0. What grades this, when nothing in the corpus does

**The hardware corpus has no audio group** (`Mars_TestOracle.md` §1, and its source's test list confirms it:
`src/tests/` has no audio module). No differential covers the interface either. So the rules below are taken from one
implementation and checked against two others, and the page says which is which:

- **The source is the FPGA core**, `rtl/AI.vhd`. Its registers, its two-slot queue, its interrupt, its DAC period and
  its delayed carry are all read there, with line numbers below.
- **Two software implementations were read beside it**: mupen64plus (`src/device/rcp/ai/ai_controller.c`) and Project64
  (`AudioInterfaceHandler.cpp`). Where they agree with the RTL that is noted; where they do not, the disagreement is
  recorded (§3.1, §3.2) and Mars follows the RTL, which is this project's standing referee.
- **The end-to-end check is two commercial games**, run by the game probe: whether they keep running with the
  interface in place, whether they play samples at the rate they asked for, and what those samples are (§5).

None of that is a measurement of a console. Every rule on this page rests on the FPGA core's reading of one.

## 1. The registers

| offset | name | write | read |
| --- | --- | --- | --- |
| `0x00` | DRAM address | 8-byte aligned, 24 bits; goes to the playing slot if none is held, else to the waiting one | the playing buffer's remaining length |
| `0x04` | length | multiple of 8, 18 bits; queues a buffer (§3) | the playing buffer's remaining length |
| `0x08` | control | bit 0 enables the DMA | the remaining length |
| `0x0C` | status | any write clears the interrupt | full, busy, enabled, and two bits always set (§2) |
| `0x10` | DAC rate | 14 bits: the DAC's period, less one, in video-clock cycles | the remaining length |
| `0x14` | bit rate | accepted and ignored | the remaining length |

**Every register but status reads back what the playing buffer has left**, as the RTL's read path does
(`rtl/AI.vhd` §163–170). That is how a game paces its audio thread: it asks how much of the current buffer is left.

## 2. The DAC, the clock, and the sample

**One stereo sample every `DAC rate + 1` cycles of the video clock** — the same 48.68MHz (NTSC) or 49.66MHz (PAL) clock
the video interface's timing already counts (`Mars_VideoTiming.md`), which the VI now exposes as `VideoClock` so the two
cannot disagree. Mars counts the period off the one bus clock with the same integer debt the VI uses, so nothing
drifts. **A period shorter than `0x200` plays at `0x200`**, as the RTL clamps it (§290–293), which caps the rate near
95kHz.

**A sample is four bytes: left then right, each sixteen bits big-endian.** The RTL reads the same four bytes (§300–301),
and so do both software implementations, which hand the raw bytes to an audio backend.

**The status register reads as the RTL builds it** (§163–168): bit 31 and bit 0 when both slots are held ("full"), bit
30 when a buffer is playing ("busy"), bit 25 when the DMA is enabled, and bits 24 and 20 always. Nothing in reach says
what bits 24 and 20 mean; the RTL sets them unconditionally, and so does Mars.

**Two things the RTL does that Mars does not, on purpose:**

- **It negates both channels** on the way out (§303–304). That belongs to its analog output path, and polarity is
  inaudible; Mars hands a frontend the samples the game wrote.
- **It ramps a starved output down to zero**, sixteen steps a period after 2,047 empty periods (§306–322) — a
  click-avoidance measure of its own. Mars produces no samples while nothing plays, and leaves the frontend's sink to
  handle the gap.

## 3. Two buffers

The interface holds **at most two buffers: one playing and one waiting.**

- **A length written while nothing plays begins that buffer at once** — at the DRAM address last written — and raises
  the interrupt (§184–189).
- **A second length queues the waiting buffer**, and the status reads full.
- **A third is dropped**, address and length both, while two are held.
- **When the playing buffer runs out, the waiting one begins** and raises the interrupt again; with nothing waiting,
  the interface goes idle and raises nothing (§216–224).
- **With the DMA disabled nothing moves**: no sample plays, no length counts down, and no buffer ends. The RTL fetches
  nothing without the enable bit (§205).

### 3.1 The interrupt marks a buffer beginning — a dispute

The RTL raises the interrupt **when a buffer begins**: the first one the moment its length is written, each later one
as it takes over. mupen64plus and Project64 raise it **when a buffer ends**, including a last one with nothing behind
it. The two agree whenever a game keeps a buffer waiting, which is the way libultra drives the interface, and they
differ at a first buffer and at a final one. Nothing measures which is right; Mars follows the RTL.

### 3.2 The carry that lands late

**A buffer that ends exactly on an 8KB page makes the next one start a page further on.** In the RTL the address
advances in its low thirteen bits only, and the carry into the page above is applied at the *next* fetch (§205–211,
§241–245). Inside one buffer that next fetch is the continuation, so crossing a page is heard as nothing. At the end of
a buffer the next fetch belongs to the following buffer, and the carry lands on *its* address instead. mupen64plus has
the same rule in its own terms (`delayed_carry`, `ai_controller.c` §89–94); Project64 has no carry at all.

Mars models it the RTL's way — the address wraps inside its page and the carry is applied before the next sample — so
the page-end case falls out rather than being special-cased. **One disagreement remains**: mupen64plus clears a pending
carry when the interface goes idle, and the RTL keeps it for whatever buffer comes next. Mars keeps it.

## 4. The queue a frontend drains

Samples go into a queue a frontend drains once a frame through `ICore.DequeueAudioSamples`: interleaved left and right,
whole pairs only, destructive. **Nothing draining must not grow memory without bound**, so past 128,000 samples the
oldest pair is dropped for each new one, as Mercury's and Venus's queues do (`EmuSen_Audio_Sync.md` §4). The game probe
drains as it runs, since it has no frontend.

**`AudioSampleRate` is the rate the game set the DAC to**, rounded to the nearest hertz: 32,006Hz for Wave Race 64's
`1520` on an NTSC clock, 31,995Hz for Super Mario 64's `1551` on a PAL one — both libultra's 32kHz, as near as the DAC can
divide. Before a game sets a rate it is 44,100, which a frontend only uses to open its device.

**That departs from `ICore`'s own comment**, which says the rate is fixed for the session. The sink already follows a
change — `AudioPlayer.Submit` reopens its device when the rate it is given differs from the open one
(`EmuSen_Audio_Sync.md` §7.2) — so a game that changes its rate mid-session costs a reopen, not wrong-speed audio. The
alternative, resampling inside the core to a fixed rate, would put a second resampler in front of the one the sink
already runs for drift, and was not taken.

## 5. What was measured

**Both games that run keep running, and play what they asked for.** The game probe, 90 seconds a game, after this
slice:

| | Super Mario 64 (Europe) | Wave Race 64 (USA) |
| --- | --- | --- |
| console time covered | 5.74s | 5.53s |
| rate the game set | `1551` → 31,995Hz (PAL) | `1520` → 32,006Hz (NTSC) |
| stereo samples played | 175,645 (5.49s) | 169,430 (5.29s) |
| framebuffers handed to the VI | 98 | 86 |
| audio / graphics tasks | 268 / 99 | 311 / 100 |

The shortfall between console time and audio time is the boot: both recordings open with about a second of silence and
then carry sound to the end — Wave Race steadily near −30dBFS with slight stereo (left–right correlation 0.96 to 1.00),
Super Mario 64 a burst between 1.0 and 4.0 seconds peaking at 10,399 and then quiet, which is the shape of an intro
cue and the pause after it. Neither game stalled over the time covered, and each kept handing the video interface
new frames throughout.

**The byte order was checked, not assumed.** Read as the interface plays it — big-endian, left first — the recordings
move smoothly: a mean step of about 300 between samples, and 1,500 to 4,600 zero crossings a second. Read with each
half's bytes the other way round, the same data is noise: a mean step near 21,000 and about 15,500 crossings a second,
which is half the sample rate, the signature of white noise. A wrong byte order could not produce the first.

That is evidence the interface delivers what the games put in RDRAM, at the rate they chose. It is not evidence that
what they put there is right — that is the signal processor's audio microcode, graded by its own slices — and nobody
has compared the recordings with a console's.

**The rules hold as named cases.** `MarsAudioTests` pins each one in §1–§4 separately, and a breakage round is recorded
in §5.1.

### 5.1 The breakage round

Eighteen rules were broken in turn with the named cases run each time. **Fifteen were caught at once, and three were
not** — and all three for the same reason: the tests had never used a value that would show them.

- **The period's floor.** The case compared the rate against `ShortestPeriod` itself, so moving the constant moved the
  expectation with it; a floor of `0x100` passed. The case now names the RTL's number: rates `0x010`, `0x1FE` and `0x1FF`
  all play at a 512-cycle period and `0x200` at 513.
- **The low three bits of an address and of a length.** Every case wrote doubleword-aligned values, so a mask that kept
  bit 2 passed. Two cases now write a buffer four bytes into a doubleword and a length of `0x0F`, and see the RTL's
  masking: the buffer plays from the doubleword's start, and the length reads back as 8 and plays two samples.

Each of the three was put back and is now caught by the case written for it. A rule that a test reads from the
implementation rather than states is a rule that test does not check — the same lesson `Mars_Memory.md` §7.9 recorded
for the cartridge bus's decay constant, learned again here.

## 6. What this is not evidence for

- **The console.** Every rule is the FPGA core's reading; §3.1 and §3.2 are the places another reading exists.
- **Timing finer than a sample.** The RTL fetches eight bytes at a time into a FIFO, so its remaining length runs a few
  samples ahead of what is heard; Mars counts what is played. A game that polls the length finely could see the
  difference.
- **The bit rate**, which Mars ignores as the RTL does.
- **Audio quality in a frontend.** Mars runs well under full speed (`Mars_GameProbe.md` §3), so a live frontend starves
  the sink; the probe's recordings are made off console time and are the only faithful listening copy.
