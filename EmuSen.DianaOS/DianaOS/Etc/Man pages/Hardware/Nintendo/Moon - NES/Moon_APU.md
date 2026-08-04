# Moon (NES) — APU

*This revision: the 2A03 synthesizes. All five channels, a cycle-accurate frame sequencer, the non-linear DAC and the console's output filters.*

*Previous revision: register surface and frame-counter IRQ only, no sound.*

Covers `Cores/Nintendo/Moon - NES/Apu/`.

---

## 1. Clocking

Every timer here is expressed against the **NTSC CPU clock, 1789773 Hz** (`Apu.CpuClockHz`). PAL is not modelled — see §6.

`Apu.Step(cpuCycles)` is called from `MoonCore.RunCpuUntilBudgetSpent` with the **actual cycle count the instruction took**, not an average. That matters: the frame sequencer's step boundaries are cycle counts, and an approximation drifts the envelope and length-counter rates audibly within seconds.

| Clocked every CPU cycle | Clocked every second cycle |
| --- | --- |
| triangle timer, DMC timer | pulse 1, pulse 2, noise timers |

The triangle running at the full rate is why it reaches an octave above the pulses.

**This replaced a per-scanline approximation.** The old code called `StepFrameSequencer()` four times a frame from `MoonCore.Schedule` — within a scanline for the IRQ, useless for anything else. That call is gone; the APU counts its own cycles.

---

## 2. The frame sequencer

`$4017` picks a four- or five-step sequence. Boundaries, in CPU cycles since the last `$4017` write:

- **four-step**: 7457, 14913, 22371, 29829 — quarter frames on all four, half frames on 2 and 4, frame IRQ on step 4 unless bit 6 inhibits it.
- **five-step**: 7457, 14913, 22371, 29829, 37281 — quarter frames on all but step 4, half frames on 2 and 5, **never an IRQ**.

Writing `$4017` with bit 7 set also clocks a quarter *and* a half frame immediately, which is how a game forces an envelope reset at a known moment.

Quarter frames clock the three envelopes and the triangle's linear counter. Half frames clock the four length counters and both sweep units.

Reading `$4015` acknowledges the frame IRQ. The DMC IRQ is reported in bit 7 and is **not** cleared by that read — only `$4010` bit 7 going low, or `$4015`'s enable bit, clears it.

---

## 3. The channels

### 3.1 Envelope

Shared by both pulses and the noise channel: either a constant volume or a 15-step decay, looping when the length-halt bit is set. The start flag reloads the decay to 15 and the divider to the period, and takes effect on the next quarter frame rather than immediately.

### 3.2 Pulse

Eight-step duty sequence, four settings. Sequence 3 is 25% **inverted**, not 75% — identical alone, not identical when phase matters against the other pulse.

Silent when the length counter is zero, the timer period is under 8, or the sweep target would exceed 11 bits. The last two are the *sweep unit* muting the channel, not the envelope, so output goes to zero without the envelope resetting.

**The two pulses are not interchangeable.** Pulse 1 negates with an extra −1 (ones' complement); pulse 2 uses two's complement. `PulseChannel` takes that as a constructor flag.

### 3.3 Triangle

The only channel with no volume control at all — on or off. Both the length counter and the linear counter must be non-zero for the sequencer to advance.

A period below 2 is silenced by refusing to *output*, not by refusing to *clock*. Stopping the sequencer would strand whatever step it was on as a DC level, which is a click. This is the one place the implementation deliberately differs in shape from the other channels.

### 3.4 Noise

A 15-bit LFSR. The mode bit picks the feedback tap — bit 6 for the short, tonal mode, bit 1 otherwise. The period table is in APU cycles, so it is indexed after the divide-by-two.

### 3.5 DMC

The only channel that reads memory, and the only one that stalls the CPU.

- Rates are in **CPU** cycles, so its timer runs at the full rate.
- The output level moves in steps of two, clamped to 0-127. A game writing `$4011` directly is doing DAC-level PCM, and that path works with no sample playing.
- A fetch costs the CPU **4 cycles**, banked into `DmcChannel.StallCycles` and drained by `MoonCore` exactly as OAM DMA already is.
- Reads go through the real bus (`Apu.Dmc.ReadMemory`), wired by `MoonCore` once the bus exists, so a sample pointing into a mapper window reads what that window currently holds.

---

## 4. Mixing

The DAC is **non-linear**, in two halves that are summed:

```
pulse_out = 95.88 / ((8128 / (pulse1 + pulse2)) + 100)
tnd_out   = 159.79 / ((1 / (tri/8227 + noise/12241 + dmc/22638)) + 100)
```

Two channels are louder than one but not twice as loud, which is pinned by a test rather than trusted to the formula's reputation.

**Output samples are box-filtered, not point-sampled.** Each emitted sample is the *mean* of the mixer output over every CPU cycle it covers (~40 at 44.1 kHz). Point-sampling a square wave at 44.1 kHz aliases badly; the mean is a cheap box filter that removes most of it.

### 4.1 The output filters

The console's output stage is a **90 Hz high-pass, a 440 Hz high-pass and a 14 kHz low-pass**, all one-pole, applied at the output sample rate.

The high-passes are not polish. The mixer output is **unipolar** — 0 to about 0.76 — so without them every sample carries a large DC offset. Measured on a real game before the filters: **mean 12967** against a peak of 20708. That is a pop on every start and stop, and it wastes over half the headroom. The same capture after filtering reads **mean 1.3**.

Two consequences:

- **Silencing a channel no longer produces exactly zero immediately.** The high-pass has a decay tail of a few milliseconds. The test asserts the tail falls under 2% of the playing level, not that it is bit-zero — bit-zero would be the wrong behaviour.
- **`OutputGain` is calibrated, not derived.** Set so the loudest game measured (Super Mario Bros. 2) lands near −2 dBFS. Louder games clamp.

---

## 5. Where the samples go

`Apu` owns a `Queue<short>` of interleaved stereo pairs — the 2A03 is mono, so both sides carry the same value.

- `Drain(maxFrames)` is destructive and backs `ICore.DequeueAudioSamples`, feeding `EmuSen.Endymion` (`EmuSen_Audio_Sync.md` §7). It never splits a pair.
- `Peek()` is non-destructive and backs `IDebugTarget.GetAudioSamples`, which is what `audiodump` writes to a WAV.
- The buffer is capped (`MaxBufferedSamples`, 128000) and drops the **oldest** pair when full, so a frontend that stops draining loses history rather than the present. Same role as Venus's cap: a safety valve for headless runs, not a latency knob.

`[SkipInState]` — audio in flight is not part of a save state.

---

## 6. Not done

- **PAL.** Every table and the clock constant are NTSC. A PAL cart runs at the wrong pitch and the wrong sequencer rate.
- **DMC fetch cost is a flat 4 cycles** rather than the real 3-4 depending on alignment, and the fetch does not steal the specific cycle that corrupts a controller read on hardware — the bug some games are written to work around.
- **No per-channel mute.** Venus has `mute` in the shell; Moon has no equivalent, which makes isolating a channel harder than it should be.
- **The sweep unit's reload subtlety** — a reload and a clock on the same half frame — follows the common implementation rather than being verified against hardware.

## 7. The SMB3 "silence" was not a bug

This section exists because the wrong conclusion was recorded here first, and the way it was wrong is worth keeping.

Super Mario Bros. 3 was carried as *the* known real-game audio failure: a harness run showed it writing `$4017` once and then never touching `$4000-$400F`, so the sound engine looked dead — and Super Mario Bros. 2, on the same MMC3 board, had working audio, which made it look like a CPU or mapper problem rather than a synthesis one.

The run had no input in it. **SMB3's title screen is genuinely silent**; its music starts when you leave the title.

Checked against Mesen through the probe's NES mode (`EmuSen_Debugging_Tools_Reference_v5.md` §3.43):

- With no input, Mesen writes exactly the same registers we do — `$4015 = $0F` once at boot from `$FF70`, then `$4010 = $00` twice a frame and `$4017 = $FF` once a frame from `$A005`, and nothing in `$4000-$400F` through frame 1200. Its recorded WAV is digital silence, peak 0.
- With Start tapped at frames 400/500/620, the first music-register write lands on **frame 501** and the full channel set follows. Mesen's own state line reads `sq1 per 106 vol 7`, `tri per 319 vol 1`, `dmc out 64`.
- EmuSen does the same thing: peak 0 / RMS 0 with no input, peak 17 373 / RMS 2 007 with the same presses.

Frame-aligned to identical press timing, EmuSen's frame 900 is **pixel-identical to Mesen's frame 897**, 0 of 61 440 pixels differing. The three-frame offset is how long each emulator took to accept the press. Every byte-level difference at a fixed frame number turned out to be animation phase riding on that offset — palette entry 3 blinking `3C`↔`36` (the item boxes), OAM slots rotating for sprite flicker, one-pixel sprite positions.

The two traps worth remembering, because both produce a *confident* wrong answer:

- A negative audio result from an emulator you have not first validated on a ROM that makes noise. The reference was muted by `NesConfig::ChannelVolumes` defaulting to zeros; it would have "agreed" with any silence claim.
- Reading a no-input run as evidence about a game whose behaviour depends on input. There was nothing wrong with the measurement — only with what it was taken to measure.
