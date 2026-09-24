# EmuSen — Rewind and Fast Forward

Both features are core-agnostic: they are built on `ICore` alone and contain no SNES knowledge. A second core gets them for free by implementing `SaveState(Stream)`/`LoadState(Stream)` and honoring `SkipRendering`.

| Piece | Lives in | Knows about |
|---|---|---|
| `RewindBuffer` | `EmuSen/Common/RewindBuffer.cs` | `ICore` only |
| `XorDeltaCodec` | `EmuSen/Common/XorDeltaCodec.cs` | nothing — plain byte arrays |
| `SpeedController` | `EmuSen/Common/SpeedController.cs` | nothing — pure arithmetic |
| `ICore.SaveState(Stream)` / `LoadState(Stream)` / `SkipRendering` | `EmuSen/Cores/ICore.cs` | the contract itself |

---

## 1. Rewind

### 1.1 Why the state had to become stream-addressable first

`ICore` only had `SaveState(string path)`/`LoadState(string path)`. Rewind needs a snapshot ten-plus times a second; routing that through the filesystem would mean a file create/write/delete per snapshot, and would put emulation latency at the mercy of whatever the disk is doing. So `SaveState(Stream)`/`LoadState(Stream)` were added as the primitives, and the two path overloads became one-line wrappers that hand them a `FileStream`.

The bytes are identical either way — a rewind snapshot and a `.state` file on disk are the same format, header included (see `EmuSen_Save_States.md` §3). Neither stream overload closes the stream it is given (`leaveOpen: true` on the `BinaryWriter`/`BinaryReader`), and both leave it positioned just past the state, so a caller can pack a state into a larger stream alongside its own data.

### 1.2 What one snapshot actually costs

Measured against real ROMs (`EmuSen.Pharaoh`, `rewind on 4 96` then `frames 3000`):

| | Raw state | 750 snapshots held | Per snapshot |
|---|---|---|---|
| Super Mario World (attract loop) | 584 KB | 2498 KB | ~2.6 KB |
| Donkey Kong Country | 584 KB | 6.6 MB | ~8.0 KB |
| Final Fantasy VI | 590 KB | 6.6 MB | ~8.0 KB |
| Chrono Trigger | 590 KB | 8.6 MB | ~10.6 KB |

A raw SNES state is ~585 KB. Storing 750 of those verbatim would be **428 MB**; the chain in §1.3 holds the same 50 seconds of history in 2.5–8.6 MB depending on how much the game is actually churning. Even the worst case measured is ~55x smaller than raw, and a quiet screen is ~225x smaller. At the default 96 MB budget that is roughly ten minutes of history even for Chrono Trigger.

> These numbers are all post-`[AliasOfSerializedField]`. Before that fix a state was ~1160 KB, half of it nine redundant copies of APU RAM, and the busy cases cost 48–84 KB per snapshot instead of 8–11 — a delta re-encoded every APU RAM change nine times over. See `EmuSen_Save_States.md` §2 for the full before/after.

### 1.3 The chain: XOR deltas anchored at the newest end

The obvious layout — periodic full keyframes with forward deltas between them — is what a video codec does, and it is the wrong shape here, because rewind only ever walks **backwards** from the present. Reconstructing "one step back" would mean seeking to the preceding keyframe and replaying forward to just before where you already were.

Instead the chain is anchored at the *newest* snapshot and needs no keyframes at all:

```
kept in memory:   D1   D2   D3  ...  Dn        _newest (one full state)
represents:       S0   S1   S2  ...  S(n-1)    Sn
                  where Dk = S(k-1) XOR Sk
```

Because XOR is its own inverse, `S(n-1) = Sn XOR Dn`. So stepping back is: pop the last delta, XOR it into the full buffer **in place**, hand that buffer to `LoadState`. The buffer now *is* the previous state, and the chain is still anchored at its newest end, ready for the next step. There is no seeking, no replay, and no keyframe.

Two consequences fall out of this for free:

- **Trimming to the memory budget drops the oldest deltas**, which costs nothing but reach. No re-encoding, no re-anchoring — the useful (recent) end of the chain is the end that is kept.
- **Playing forward again after rewinding just resumes appending.** `_newest` already holds the state the core was restored to, so the next capture deltas against it correctly. There is no separate "we are in rewind mode" state to unwind.

The delta encoding itself (`XorDeltaCodec`) is run-length over the identical stretches, since most of a state is unchanged four frames later. Records are `[varint identicalRun][varint diffLen][diffLen XOR bytes]`, repeated, stopping early once the tail is identical (so two identical states encode to zero bytes). The identical-run scan compares 8 bytes at a time. Measured cost is **~0.2 ms per capture** on a ~585 KB state — at the default 4-frame interval that is ~1% of runtime, confirmed by 3000 frames of SMW taking 12.01 s without rewind and 12.16 s with it.

### 1.4 When the chain MUST be cleared

`RewindBuffer.Clear()` has to be called whenever core state jumps discontinuously by any route other than `Rewind()` itself:

- loading a ROM
- loading a save state (menu, hotkey, or `state` console command)

Skipping this splices two unrelated timelines together: the deltas still apply cleanly (they are just XOR), so nothing throws — you would simply rewind into a state that never existed. Both frontends call `Clear()` at all three sites.

### 1.5 Tuning: interval and budget

- **`IntervalFrames`** (default 4) — frames between snapshots, i.e. rewind granularity. This is the dominant cost knob: halving it doubles both CPU and memory. 4 frames is ~15 captures/sec, which feels continuous when scrubbing and costs ~1% of runtime.
- **`BudgetBytes`** (default 96 MB) — a cap on the deltas held, not counting the one full anchor state. At the measured per-snapshot sizes this is roughly 42 minutes of history for a quiet screen and ~10 minutes for Chrono Trigger's worst case, so in practice it rarely binds at all now. It existed to stop a busy game eating unbounded memory, and that ceiling is what it still is.

### 1.6 Accuracy note

A rewound state is exactly as accurate as a save state, because it *is* one — same serializer, same fields, same version header. The known gaps are `StateSerializer`'s (positional layout, no field tagging — see `EmuSen_Save_States.md` §4) plus the handful of `VenusCore` sub-frame fields (`_lineCycles`, `_scanlineStarted`, `_spc700CycleRemainder`) that were never in the state format. Since snapshots are only ever taken at frame boundaries, those are sub-scanline quantities and are not observable.

Verified end-to-end against SMW: dumping `regs` + `mem WRAM 0 100` + `mem APURAM 200 40` at frame 700, running 200 more frames, then `rewind back 50`, produces a **byte-identical** dump — 65816 registers, PPU registers, SPC700 registers, WRAM and APU RAM all restored exactly.

### 1.7 A core with no state format

**An empty snapshot is not history, and `CaptureNow` now treats it as none: it clears the chain and stores nothing.** The case arrived with Mars (`Hardware/Nintendo/Mars - N64/Mars_Core.md` §6), whose stream overloads write zero bytes because it has no state format yet and because throwing from them would stop both frontends' emulation loops, which call `OnFrameCompleted` inside the `try` their `catch` ends the session from.

**What the chain did with zero bytes before, measured rather than inferred.** Two empty states are the same length, so the shape check in `CaptureNow` passed, and `XorDeltaCodec` encodes two identical buffers to nothing (§1.3) — so every capture appended an empty delta that cost no bytes and was therefore never trimmed by the budget (§1.5). 600 frames of Wave Race 64 left a depth of 149 at 0 bytes; 400 of Super Mario 64 left 99. `Rewind()` then answered true once per delta while `LoadState` restored nothing, and Pharaoh's `rewind back` counted steps that were never taken. `MarsCoreTests.Mars_leaves_the_rewind_buffer_with_no_history` failed with a depth of 4 after five captures before the guard went in, and passes after it.

**What a user sees did not change.** A frontend runs no frame while rewind is held, so the picture holds whether `Rewind()` answers true or false; the difference is the memory, and the harness's report. The growth it prevented was small — a list node per capture, about one a second at Mars's present speed — which is why this is recorded as a reporting defect rather than a leak that mattered.

**Why the guard does not generalise into a rule about cores.** Every other core writes at least a magic word and a version (`EmuSen_Save_States.md` §3), so for them a zero-length state cannot occur and the guard never fires. It is a statement about one sentinel, not a capability check; a core that someday cannot snapshot *some* of the time would need a real way to say so, and this is not it.


### 1.8 A core with work on other threads: the snapshot, and the delta encoded elsewhere

*2026-09-19.* Mars runs its display processor's list on a thread of its own (`Hardware/Nintendo/Mars - N64/Mars_Rdp.md` §2.6), and a state must hold what that thread drew, so `SaveState` waits for it. At a frame boundary the thread is typically a whole list behind — 7 ms in Wave Race 64, 5 ms in Super Mario 64 outside the castle, measured by the frontend bench — and a capture every fourth frame therefore cost the emulation thread that wait plus the 3.3 ms of writing, copying and encoding a 6 MB state: Wave Race ran at 61 fps without the buffer and 54 with it, against a console of 60 (`Mars_Performance.md` §34).

Two changes, one offered by the core and one in the buffer:

- **`ISnapshotCore.SaveSnapshot`** (`EmuSen/Cores/CoreCapabilities.cs`) is an optional capability: a state written without waiting for work the core has in flight elsewhere, which its own `LoadState` reads. `CaptureNow` asks for it when the core offers it and falls back to `SaveState` otherwise. Mars implements it by holding its thread between two command words and writing the words it has not run after the state's body (`Mars_SaveStates.md` §1, `Mars_Rdp.md` §2.7); a load runs those words before the machine continues, so the chain restores the same machine a joined state would.
- **The delta is encoded on a pool thread.** `CaptureNow` copies the state into a spare buffer — the previous state's, reused, so a capture allocates no state-sized array — and hands the pair to a task. `Depth`, `BufferedBytes`, `Rewind` and `Clear` settle that task first, so nothing reads or moves the chain while a delta is missing from it. The buffer is still driven from one thread; the task only computes.

What a capture then costs the emulation thread is the state write alone, under a millisecond for Mars. What it costs the core is the standstill during that write, which the display processor's thread would otherwise have spent drawing: a fraction of a millisecond a capture. The accuracy note of §1.6 holds unchanged, since a rewound state is a state and Mars's tail is the same words the thread would have run, run at the load instead. `RewindBufferTests` cover the capability and a capture whose delta is still encoding; `MarsThreadedRdpTests` cover a snapshot taken with every word of a list pending.

**Measured** (`Mars_Performance.md` §34.1): Wave Race 64 with the buffer on went from 55.2 to 58.7 fps and Super Mario 64 outside the castle from 54.0 to 56.4, the two states whose thread was behind at the frame's end; the other two were unchanged. A capture costs the emulation thread 1.4 to 1.7 ms now, against 3.3 plus the join. Two things to know. Mars's tail is a fixed 32k words, because the chain needs every snapshot of a machine to be the same length — a tail as long as the backlog restarted the chain at every capture, which the bench found. And a delta now carries the tail's changed words, 400 to 600 KB a capture in Wave Race, so the default budget holds about forty seconds of that game rather than minutes; `IntervalFrames` is the knob (§1.5).

---

## 2. Fast forward

`SpeedController` holds the policy and nothing else — no core reference, no timing, no threads. A frontend's own loop asks it three questions per frame.

### 2.1 Speed as a percentage

`SpeedPercent` is 100 for normal, higher for fast-forward, lower for slow motion. `FrameInterval(coreFrameRateHz)` returns how long to wait between frames, derived from the core's *real* hardware rate (60.0985 Hz on NTSC SNES — see `Venus_CPU.md` §8.5b), not a hardcoded 60.

`SpeedPercent = 0` is the special "unthrottled" case and means **no pacing at all**, not infinitely slow: `FrameInterval` returns `TimeSpan.Zero` and the caller skips its wait entirely.

### 2.2 Frame skip, and what it costs

Emulating at 300% but *drawing* at 300% is wasted work — no display shows it and the compositor is a large share of frame time. So `RenderEveryNthFrame` draws 1 frame in every `SpeedPercent / 100`, keeping the **presented** rate near the hardware rate however fast emulation runs. `MaxFrameSkip` (default 9) caps it so an extreme speed can't stop drawing altogether.

The frontend turns that into `ICore.SkipRendering` for the frames it is going to discard. On Venus this drops exactly one thing: the per-scanline `Renderer.RenderScanline` call. Everything with emulation-visible side effects still runs — in particular **HDMA still executes**, because it writes real hardware registers.

Measured headless (`fastforward on`, 3300 frames of SMW): **12.4 s → 5.5 s**, about 2.5x once process startup is subtracted. That makes it genuinely useful in `EmuSen.Pharaoh` for long boot sequences, not just a frontend convenience.

> **`SkipRendering` is a fast-forward-only knob and has no business being set during normal play.** Sprite range/time-over (`ppu.RangeOver`/`ppu.TimeOver`, readable by games at `$213E`) are set *by* the sprite evaluation inside `RenderScanline`, so a skipped frame leaves them holding the previous frame's values. Real hardware has no such notion. This is the standard frameskip tradeoff and MesenCE makes the same one; it is acceptable precisely because the frames it affects are being thrown away.

### 2.3 Audio outside normal speed

At 300% the core produces samples 3x faster than any real output device consumes them; at 25% it starves one. `ShouldPlayAudio` is therefore true only in the 50%–200% band, where the existing buffer throttle can absorb the difference.

Outside that band the frontend does **not** simply stop pumping — it drains the core's buffer and discards the samples (`DequeueAudioSamples(int.MaxValue)`), and resets the rate controller so its resampler doesn't interpolate across the seam (`EmuSen_Audio_Sync.md` §3.2). Stopping the pump alone would let the core's internal queue grow the whole time fast-forward is held, and then dump a large backlog of stale audio the instant normal speed resumed. Rewind drains and resets for the same reason.

Inside the band, drift is handled by dynamic rate control rather than by dropping anything — see `EmuSen_Audio_Sync.md`. Fast-forward deliberately bypasses that path, since 300% is far outside the ±0.5% the loop has authority over. Pitch-correct fast-forward audio (MesenCE resamples) remains undone and is tracked in `EmuSen_Audio_Sync.md` §6.

---

## 3. Harness support (`EmuSen.Pharaoh`)

`FrameRunner` owns a `RewindBuffer` and feeds it from `RunFrames()`, so every existing verb that advances frames (`frames`, `waitstable`, `waitchange`, `contactsheet`, ...) contributes to the chain automatically. `FrameRunner.StepBack(n)` steps back and keeps the harness's own `CurrentFrame` counter in sync by subtracting `n * IntervalFrames`.

Verbs are listed in `EmuSen_Debugging_Tools_Reference_v5.md` §3.15.

**Why rewind is worth having in a headless harness at all:** it turns "when did this go wrong?" into a bisection instead of a re-run. Finding the exact frame a glitch appears currently means re-launching from boot with a different `--screenshot` frame each time; with rewind you overshoot once, then step back until it disappears, in a single process.

**A rewind must refresh the debug target's providers.** `IDebugTarget.RefreshProviders()` normally runs right after `RunFrame()`, and a rewind runs no frame — so without an explicit refresh, `regs`/`sprites`/`pal` keep reporting the frame you rewound *away from*. This was a real bug during bring-up and it is quietly misleading rather than obviously broken: the readout looks plausible, it is just the wrong frame. All three drivers (harness and both frontends) refresh explicitly after a rewind.

Those explicit calls did nothing on the NES until 2026-08-04. `RefreshProviders()` was a defaulted no-op that the Moon core never implemented — it refreshed itself from its own `EndFrame` hook instead, which never runs during a rewind. So the exact bug this paragraph warns about was live on one of the two cores the whole time, in a driver that looked like it had handled it. The interface member is now required rather than defaulted; see `Moon_Debug.md` §3.

---

## 4. Frontend hotkeys

Both `EmuSen.Hotaru` and `EmuSen.Mistress` use the conventional emulator bindings:

| Key | Does |
|---|---|
| **Tab** (hold) | Fast-forward at `TurboPercent` (default 300%) |
| **Backspace** (hold) | Rewind |

These are **held**, not edge-triggered, unlike every other hotkey in either frontend (which sets a `volatile bool` request flag consumed once). A held key sets state on `KeyDown` and clears it on `KeyUp`.

`Tab` sets `e.Handled = true`, or Avalonia consumes it for focus traversal and the emulation thread never sees it.

Rewind takes over the frame entirely: the loop steps the chain back, refreshes providers, drains audio, presents, and skips `RunFrame()` altogether for that iteration. Running out of history resyncs the pacing clock so holding Backspace at the start of the buffer doesn't accumulate a backlog of missed ticks.

---

## 5. Moments: the history as something to choose from

*2026-09-24.* Holding the rewind key walks back one snapshot at a time and shows each as it goes (§4). Mistress's
reel (`EmuSen_Settings_Reference.md` §4.49) shows the whole history at once as pictures and jumps to the one chosen.
That asks three things of the buffer that stepping never did: a list of what it holds, with when each was taken; a
picture of each; and a way to reach any of them. The buffer still knows nothing of any console.

### 5.1 A moment per snapshot

`RewindBuffer.Moments()` returns one `RewindMoment` per snapshot held, oldest first: `Frame`, the buffer's own count
of frames completed while it was enabled; `CoreFrames`, the core's `TotalFrames` at the capture; and `Thumbnail`, its
picture or null. The list is kept in step with the chain by the same four events that move the chain: a capture
appends one, a step back (`Rewind`) removes the newest, the budget's trim (§1.5) removes the oldest with its delta,
and `Clear` — including the restart a change of state size forces (§1.3) — empties it. So whenever a snapshot is held,
`Moments().Count == Depth + 1`, and the tests assert it after every kind of move.

The buffer counts frames itself rather than trusting the core's counter, because the reel's labels are *time played*
and a core's `TotalFrames` is whatever its state format says it is. `Frame` is rewound with the chain: a step back sets
it to the newest remaining moment's. `CoreFrames` is kept beside it for a check, not for display: on the SNES, the
Game Boy and MarsRT a chosen moment's `CoreFrames` equals the core's `TotalFrames` after the load
(`RewindToMomentTests`).

`Moments()` settles an encoding still in flight (§1.8) and must be called on the thread that drives the buffer.

### 5.2 Going straight to a moment, and what happens to the newer ones

**`RewindTo(core, frame)`** takes the moment with that `Frame`, `k` snapshots back. It XORs the newest `k` deltas
into the anchor in place — exactly the arithmetic `k` calls of `Rewind` do — drops the `k` newer moments with them,
and then loads **once**. Stepping instead would load `k` times. **`StateAt(frame)`** rebuilds a moment's bytes on a copy
of the anchor without moving anything, which is what a test compares against and what a preview of the real frame
would need; it costs a state-sized copy and the same `k` XORs.

*Stepping versus going straight, measured.* The XORs are the same either way; what going straight saves is `k − 1`
loads. On MarsRT's synthetic system (a 12,986,889-byte state) the oldest of 75 moments took **6.64 ms** straight and
**275.7 ms** by 74 steps — one load of that state is about 3.6 ms. On the SNES synthetic ROM (a 727,850-byte
state) the oldest of 751 moments took **1.05 ms** straight and **576 ms** by 750 steps. Two later runs, beside other
tests, gave 12.4 and 14.9 ms against 323 and 566 ms on MarsRT, and 2.50 against 487 ms on the SNES: the ratio, forty to
five hundred times, is the finding, and it grows with `k`. Keyframes, so that a far moment
needs fewer XORs, were not built: the XORs are not where the time goes.

**Choosing a moment discards the newer history, as stepping back does.** The chain is anchored at its newest end
(§1.3). To keep the snapshots newer than the chosen one would take a second anchor — the newest full state set aside,
13 MB on the N64 — and, at the first capture after resuming, a second timeline branching from the chosen moment, which
the reel would then have to draw as a fork. That was judged not worth building for the case it serves, a choice made
by mistake, which the reel answers before it happens: nothing is discarded until the player presses A, the reel shows
the picture and the time of what A will restore, B leaves everything as it was, and the rightmost tile, *Now*, resumes
without a load. What it costs: once A is pressed, the present is gone, and a second look at the reel starts from the
chosen moment.

**Proofs** (`RewindToMomentTests`, `RewindBufferTests`). On each core a run at Mistress's interval hashes the core's own
state at every capture; then three choices in turn — one back, the middle, the oldest — each with 40 frames of play
re-recorded between them, must each leave the core's `SaveState` hashing to the hash taken at that moment, with
`CoreFrames` restored:

| Core | Machine | Moments | Result |
| --- | --- | --- | --- |
| Venus (SNES) | synthetic ROM counting in direct page, 600 frames | 151 | three choices, each the saved state |
| Mercury (Game Boy) | synthetic ROM counting through work RAM, 600 frames | 151 | three choices, each the saved state |
| MarsRT (N64) | synthetic system with the RSP, four RDP workers, picture deferred, 300 frames | 75 | three choices, each the saved state |

Going straight and stepping land on the same bytes and play on the same (`Rewinding_to_a_moment_and_stepping_back_to_it_are_the_same_machine`,
and the timing tests on SNES and MarsRT, which compare the two landings' hashes). In Mistress, `PadRewindReelTests`
compares the whole state after A with `StateAt` of the tile chosen, byte for byte.

**A finding on the way: a fresh MarsRT is not its own state.** The first MarsRT proof failed. Saving a machine that
had never been loaded, loading that, and saving again differs in exactly one byte,
`Bus.Si.Controllers[0].Pak.Dirty`: a load marks the controller pak dirty, in the Rust core (`machine.rs`) as in the C#
one (`MemoryBus.cs`), so that the frontend writes the loaded pak's contents to its file. It is a request to the host,
not machine behaviour, and after one load the round trip is exact, which is why `MarsRtRewindTests` — which always
start from a game's state — never met it. The proof now loads the machine's own state first, and
`On_a_fresh_MarsRT_the_first_load_changes_the_pak_s_dirty_flag_alone` pins the one byte by name from the state's layout.
Its consequence for a player is one extra write of the pak file after the first rewind of a session.

### 5.3 Pictures

The buffer has no picture of its own — the frame belongs to the frontend's loop — so the frontend hands it one.
`OnFrameCompleted` now answers whether it took a snapshot, and when it did, `AttachThumbnail(rgba, width, rows,
rowRepeat)` gives the newest moment a picture of that frame; a second call for the same moment does nothing.
`ThumbnailWidth` (0 by default, so Pharaoh and Hotaru keep none) is the picture's width; Mistress sets 160.

`RewindThumbnail.From` keeps the frame's *shown* proportions — `rows × rowRepeat` tall, so a core that leaves row
doubling to the frontend is not squashed — never enlarges, and averages four samples per output pixel, at the quarter
points of the source box it covers, whatever the source's size. It stores RGB565, two bytes a pixel, half of RGBA:

| Console | Frame | Picture | Bytes |
| --- | --- | --- | --- |
| SNES | 256×224 | 160×140 | 44,800 |
| Game Boy | 160×144 | 160×144 (kept) | 46,080 |
| N64, NTSC | 320×240 | 160×120 | 38,400 |
| N64, PAL (Super Mario 64 Europe) | 640×576 | 160×144 | 46,080 |

`ToRgba` expands one for display. The first version computed each sample's column with a 64-bit division per pixel and
took 158 to 189 µs a picture; computing the columns once per picture took it to 70 to 93 µs, bit-identical on eight
frame shapes from 100×90 to 1280×960 with row doubling (checked against the first version, then the check was removed).

A picture that did not change — a core that reports the same `FrameSerial` (`EmuSen_Multicore.md` §14) — is attached again as the same
object (`AttachThumbnail(RewindThumbnail)`) rather than made again; §5.4's accounting counts it once per moment, which
over-states the memory and never under-states it. The frontend's rules for which frame a moment gets are `EmuSen_Settings_Reference.md` §4.49's.

### 5.4 The picture budget

`ThumbnailBudgetBytes`, 32 MB by default, caps the pictures, not the snapshots. Past it, every second picture of the
older half goes, repeatedly, until the pictures fit: the newest stay dense, the density falls with age, the oldest
picture survives, and **no snapshot is dropped** — a moment without a picture is still in the chain, and is only absent
from the reel. At 44,800 bytes a picture the budget holds 749 SNES pictures, 50 seconds at 15 a second, before the
first thinning. `Past_the_picture_budget_older_pictures_thin_out_and_the_snapshots_stay` runs 400 captures against a
budget of 40 pictures: never over, at least 20 kept, the first and last kept, the newest ten contiguous, the oldest gap
wider than the newest.

### 5.5 Costs, predicted and measured

Predicted on 2026-09-24 before anything was measured (the list is kept beside the bench, in
`~/.cache/emusen/probe/rewind-reel/predictions.txt`), then measured with `rewindbench`, a console that runs the loop
Mistress runs — `RunFrame`, the audio drained, `OnFrameCompleted`, the frame fetched and given back, a picture attached
on a capture — flat out, 300 frames of warm-up excluded. Timing runs interleave three modes on one build — rewind off,
rewind without pictures, rewind with pictures — rotating their order each round, each run under
`flock ~/.cache/emusen/probe/mars-speed/bench.lock`, on three games from the library copied to a scratch folder and
started from states in play: Yoshi's Island and Super Mario Land 2 with Right held, Super Mario 64 (Europe) in the
castle, MarsRT with four workers and the picture deferred. One build rather than two, because pictures are a property
(`ThumbnailWidth`), not a compile-time change.

| | Predicted | Measured |
| --- | --- | --- |
| A picture's bytes | 44,800 SNES, 46,080 Game Boy, 38,400 N64 | as predicted for the SNES and the Game Boy. **Wrong for this N64 game**: a PAL Super Mario 64 frame is 640×576, so 160×144 and 46,080 bytes; 38,400 is an NTSC 320×240 frame's |
| One picture | 20 to 100 µs | **first version outside it, 158 to 189 µs**; after the columns were computed once (§5.3), medians of 68.0 µs (SNES), 68.8 µs (Game Boy), 105.8 µs (N64 640×576) |
| A frame, pictures against rewind alone | +5 to 25 µs, not distinguishable from the noise | SNES: −110 µs, median of five paired rounds spread from −461 to +16 — not distinguishable. **Game Boy: +36 µs, four of five rounds +24 to +48 — distinguishable**: 5.7 % of a 0.62 ms headless frame, 0.2 % of a frame's 16.7 ms at full speed, and twice the picture's own share of 17 µs, the rest not attributed. N64 (six rounds, four modes): +50 µs, 4.41 against 4.36 ms, inside the spread |
| Rewind itself, for scale | — | SNES +256 µs a frame (9 %), Game Boy +23 µs (4 %), N64 +307 µs (8 %), headless |
| Memory | the 32 MB cap reached after about 50 s, then held | Yoshi's Island: 750 moments at 50 s, thinned to 563 pictures, 24.1 MB; at 499 s, 7,500 moments, 581 pictures, 24.8 MB, **chain 2.8 MB**. Super Mario Land 2: the pictures swing between 24.7 and 31.7 MB from one thinning to the next, chain 2.2 MB at 502 s. Super Mario 64: chain 93.4 MB at 60 s, where the 96 MB budget binds, pictures 25.0 MB |
| Choosing the oldest moment | direct saves `k − 1` loads | §5.2: forty to five hundred times faster than stepping |

*The N64 runs, and a noise source.* The first interleaved N64 run showed pictures costing 1.4 to 1.8 ms a frame in
three rounds of five. A second run of six rounds with a fourth mode, pictures made and dropped, found runs of the same
size — +1 to +3 ms a frame, `RunFrame` itself slower — in single rounds of the rewind-only and picture modes alike, and
once with rewind off in the first run: it is the machine's (other work sharing the processor with MarsRT's workers,
which the bench's lock does not exclude), not the pictures'. Garbage collection was the same with and without pictures,
4/3/3 collections against 3/3/3 and one to two milliseconds of pauses in 1,200 frames. The medians above are over those
episodes.

*What the memory runs show that was not predicted.* **On the SNES and the Game Boy the pictures, not the snapshots,
are now most of rewind's memory**: 24.8 MB of pictures against a chain of 2.8 MB in Yoshi's Island with eight minutes
held. Before the reel those games' rewind cost a few megabytes; with it, 27 to 35. On the N64 the chain is still the
larger, 93 MB to 25. `ThumbnailBudgetBytes` is the knob, and Mistress does not expose it. The N64 memory run's managed
heap reached 372 MB at its peak, against about 120 MB of chain and pictures held; it was not measured without
pictures, and is recorded here, not attributed.

### 5.6 Not done

- **No picture for a frame fast-forward skipped.** A capture on a frame that was not drawn has no picture, and its
  moment is not on the reel. At 200 % with the default skip, captures fall on drawn frames always or never, depending
  on how the buffer's count lines up with the core's. Forcing a draw on capture frames would cost up to a quarter of
  the drawing fast-forward skips, and was not done.
- **The preview is the picture enlarged**, not the frame: the reel shows the 160-pixel picture at the size of its
  stage. A full-size preview would need a second machine to load `StateAt` into, or a larger picture.
- **No redo** (§5.2), no reel in Pharaoh, and nothing for the C# Mars, which keeps no history (§4.21b of the settings
  reference).
- **Not measured on the handheld.**
