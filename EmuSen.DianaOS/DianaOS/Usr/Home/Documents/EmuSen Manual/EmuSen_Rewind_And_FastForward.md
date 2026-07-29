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

Outside that band the frontend does **not** simply stop pumping — it drains the core's buffer and discards the samples (`DequeueAudioSamples(int.MaxValue)`). Stopping the pump alone would let the core's internal queue grow the whole time fast-forward is held, and then dump a large backlog of stale audio the instant normal speed resumed. Rewind drains for the same reason.

This is deliberately cruder than resampling. Pitch-correct fast-forward audio is a real option (MesenCE resamples), but it belongs with the dynamic-rate-control work the audio pipeline still needs, not bolted onto the speed knob — see `Venus_APU.md`.

---

## 3. Harness support (`EmuSen.Pharaoh`)

`FrameRunner` owns a `RewindBuffer` and feeds it from `RunFrames()`, so every existing verb that advances frames (`frames`, `waitstable`, `waitchange`, `contactsheet`, ...) contributes to the chain automatically. `FrameRunner.StepBack(n)` steps back and keeps the harness's own `CurrentFrame` counter in sync by subtracting `n * IntervalFrames`.

Verbs are listed in `EmuSen_Debugging_Tools_Reference_v5.md` §3.15.

**Why rewind is worth having in a headless harness at all:** it turns "when did this go wrong?" into a bisection instead of a re-run. Finding the exact frame a glitch appears currently means re-launching from boot with a different `--screenshot` frame each time; with rewind you overshoot once, then step back until it disappears, in a single process.

**A rewind must refresh the debug target's providers.** `SnesDebugTarget.RefreshProviders()` normally runs right after `RunFrame()`, and a rewind runs no frame — so without an explicit refresh, `regs`/`sprites`/`pal` keep reporting the frame you rewound *away from*. This was a real bug during bring-up and it is quietly misleading rather than obviously broken: the readout looks plausible, it is just the wrong frame. All three drivers (harness and both frontends) refresh explicitly after a rewind.

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
