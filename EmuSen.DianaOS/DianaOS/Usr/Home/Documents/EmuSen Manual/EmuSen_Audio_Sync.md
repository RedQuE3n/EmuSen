# EmuSen — Audio Sync

How generated samples reach a real output device without drifting, stalling, or dropping. Core-agnostic: `DynamicRateControl` and `LinearResampler` know nothing about the SNES, only about interleaved stereo shorts and how full a queue is.

| Piece | Lives in | Knows about |
|---|---|---|
| `LinearResampler` | `EmuSen/Audio/LinearResampler.cs` | nothing — plain sample arrays |
| `DynamicRateControl` | `EmuSen/Audio/DynamicRateControl.cs` | a queue depth and a ratio |
| `AudioPlayer` | `EmuSen.Hotaru/Audio/`, `EmuSen.Mistress/Audio/` | SDL, and the two above |

---

## 1. The problem, and the shape of the fix

Three independent oscillators are involved: the SNES's video clock, its audio clock, and the host sound card's. They always drift relative to one another, so *some* correction is permanently required — no amount of emulation accuracy removes the need.

The old mechanism paid for that correction in one lump. `SDsp` discarded down to 100 ms whenever its buffer passed 250 ms — **150 ms of audio deleted mid-waveform**, a gap and a click at once. Worse, two throttles were fighting: the consumer (`AudioPlayer.Pump`) refused to drain while the device queue held more than two buffers, which let the producer's buffer fill and trip that very discard. Fixing the underlying clock errors (`Venus_CPU.md` §8.5b) made it fire less often; it did not make it correct.

The fix inverts where buffering lives:

- **`Pump` now drains the core buffer completely, every call.** The core-side buffer is no longer a latency reservoir — measured at **0.0 ms** at baseline, immediately after a one-second stall, and after recovery, where it previously sat on a ~256 ms floor.
- **All buffering lives in the output device queue**, which is the only place that can be measured against real consumption.
- **Drift is absorbed by resampling within ±0.5%**, continuously, instead of by deleting samples. Nothing is ever discarded in normal operation.

---

## 2. The resampler

`LinearResampler` resamples interleaved stereo by a fractional ratio, where **ratio is output frames per input frame**.

**Linear interpolation is good enough here specifically because the ratio never leaves the neighbourhood of 1.0.** Linear interpolation is a poor general-purpose resampler — it rolls off highs and aliases badly at large ratios. At 1.000 ± 0.005 it is very nearly the identity: each output sample sits at most half a percent of a sample-period away from a real one, so the error is far below the quantisation floor. A polyphase/sinc resampler would buy nothing audible at these ratios and would cost real CPU per sample.

**Phase must carry across calls.** `Pump` hands over one frame's worth of samples at a time (~533 frames), and a resampler that restarted its phase each call would inject a discontinuity — a click — at every chunk boundary, 60 times a second. So the fractional position and the previous input frame are instance state, not locals. `Ratio_of_one_stays_continuous_across_chunk_boundaries` covers exactly this, and `Output_frame_count_tracks_the_ratio_over_a_long_run` confirms no drift accumulates: over ~1M frames the produced count tracks the requested ratio to within 5 frames.

The first ever call has no previous frame to interpolate from, so it primes from the first input frame and starts output one frame in. That is a one-frame startup latency, not an ongoing loss.

`Reset()` drops the carried phase. Call it whenever the sample stream is discontinuous — see §3.2.

---

## 3. Dynamic rate control

The control law, in `DynamicRateControl.ComputeRatio`:

```
delta = (queuedFrames - target) / target      clamped to [-1, +1]
ratio = 1 - delta * MaxDeviation
```

A queue fuller than target asks for a ratio below 1 (emit fewer frames than consumed, so the queue falls); an emptier one asks for above 1. `MaxDeviation` defaults to 0.005, the same ±0.5% bsnes/ares and RetroArch use.

**This is a proportional controller, so it has steady-state offset by design.** It settles where output exactly matches drain, which is:

```
equilibrium queue = target * (1 + (1 - drain/produced) / MaxDeviation)
```

So a persistent 0.19% over-production settles at ~1.37× target, not at target. That is intentional and matches the reference implementations — an integral term would remove the offset but adds a tuning problem and an overshoot mode for no audible benefit. In practice the offsets are small: this core's own clock drift is -0.008% (`Venus_CPU.md` §8.5b) and consumer sound cards are typically within ±0.1%, which puts the queue inside a few percent of target. `DynamicRateControlTests` asserts the settled depth against that formula directly rather than against a hand-picked range.

**The band is also the authority limit.** Drift beyond ±0.5% cannot be corrected at all — the loop saturates and the queue runs to an end stop. `Drift_beyond_the_deviation_band_is_not_correctable` documents that plainly. Real drift is orders of magnitude smaller; what does exceed the band is fast-forward, which is why fast-forward bypasses this path entirely (`EmuSen_Rewind_And_FastForward.md` §2.3).

### 3.1 Shedding — the gross-backlog path

Correcting a large excess at 0.5% takes a long time: a 250 ms excess needs ~50 seconds. So a bounded emergency path exists for excursions the control loop cannot resolve in reasonable time.

Above `target * 3` the incoming chunk is **shed** — not queued — until the queue falls back under `target * 2`. This skips forward in content rather than cutting a hole in what is already queued, so the device never runs dry and there is no silence, and it self-limits by construction.

`SheddingEvents` counts engagements and **is expected to stay at 0 in normal play**; it is a stall counter. A one-second burst of unpaced frames does trip it once, which is the case it exists for.

### 3.2 When `Reset()` is mandatory

Whenever the sample stream jumps rather than continues:

- loading a ROM
- loading a save state
- every frame audio is being discarded rather than played — fast-forward outside the 50–200% band, and rewind

Without it the resampler interpolates across the seam between two unrelated waveforms.

### 3.3 `TotalInputFrames` / `TotalOutputFrames`

Two cumulative counters over everything `Process()` has taken in and handed on. Their difference *is* the control law's effect on the queue: frames withheld (resampled away, or dropped outright while shedding) are frames the output device will not get, which is the only lever this class has.

They exist because that effect is otherwise impossible to observe without reading the output queue, and the queue's depth is a function of the *device's* clock as much as ours. `AudioLatencyDriftTests` asserts against these instead of against a queue reading for exactly that reason — see `EmuSen_Settings_Reference.md` §4.10. Cumulative, never reset (not even by `Reset()`), so a test can subtract two samples and get the interval it cares about.

---

## 4. The core-side buffer is a safety valve now

`SDsp.AudioBuffer` still has a cap, but its role changed completely. It exists only for the case where **nothing is draining at all** — headless runs under `EmuSen.Pharaoh`, a paused session, no audio device — where it bounds memory instead of resyncing anything.

- Raised from 16000 samples (250 ms) to **128000 (2 s)**, since a live frontend now keeps it near empty regardless.
- Trims **in pairs**, never single samples. The old drain removed one `short` at a time until under a threshold, so any odd configured value could leave an odd count and permanently swap the left and right channels for the rest of the session. That latent bug is gone by construction rather than by keeping the numbers even.
- Behaves as a plain ring rather than a 150 ms cliff — it drops the oldest pair per new pair once full.

A useful side effect: `audiodump` (`EmuSen_Debugging_Tools_Reference_v5.md` §3.15) now captures **2.00 s instead of 0.25 s**, an 8x longer window for investigating audio.

---

## 5. Latency, and how to move it

Total output latency is the device queue depth, which rate control holds near `AudioSettings.OutputTargetLatencyMs` (default 256 ms).

That default is deliberately `2 x BufferFrames` — SDL pulls `BufferFrames` (4096) at a time, so a target below one full device buffer would underrun constantly. **The real lever for lower latency is `BufferFrames` in `AudioPlayer`, not the target**: halving it to 2048 allows a 128 ms target. That is a genuine latency/underrun tradeoff on slower machines and is left where it was rather than changed as a side effect of this work.

`AudioPlayer.QueuedFrames` exposes the live depth, and `RateControl.LastRatio` / `SheddingEvents` expose what the loop is doing.

---

## 6. Not done

- **Pitch-correct fast-forward audio.** Fast-forward currently discards samples outside 50–200%. Resampling at 3x through this path would work mechanically but sounds wrong; doing it properly means a real resampler and belongs with that work, not with the drift loop.
- **Audio-clock-mastered pacing** — driving the frame loop from device consumption rather than a wall clock. Strictly more accurate, but it restructures both frontends' loops, and rate control gets the audible benefit without that risk.
- **A higher-quality resampler.** Only worth it if the ratio ever needs to leave the ±0.5% neighbourhood (see §2).
