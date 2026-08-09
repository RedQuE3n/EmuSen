# Moon (NES) — APU

*This revision: the 2A03 synthesizes. All five channels, a cycle-accurate frame sequencer, the non-linear DAC and the console's output filters.*

*Previous revision: register surface and frame-counter IRQ only, no sound.*

Covers `Cores/Nintendo/Moon - NES/Apu/`.

---

## 1. Clocking

Every timer here is expressed against the **NTSC CPU clock, 1789773 Hz** (`Apu.CpuClockHz`). PAL is not modelled — see §6.

`Apu.Step(1)` is called **once per CPU bus cycle**, from `MemoryBus.Tick` via `ICpuBus.Tick` — the CPU's own clock, not a per-instruction batch (`Moon_CPU.md` §2). That matters twice over: the frame sequencer's boundaries are cycle counts, so an approximation drifts the envelope and length-counter rates audibly within seconds; and a `lda $4015` has to observe the APU as it stands *at that cycle*, which a batch applied after the instruction retires cannot do. Cycles stolen by OAM DMA and by DMC fetches are handed to the APU too — they are cycles the chip lived through even though the CPU was not driving the bus.

| Clocked every CPU cycle | Clocked every second cycle |
| --- | --- |
| triangle timer, DMC timer | pulse 1, pulse 2, noise timers |

The triangle running at the full rate is why it reaches an octave above the pulses.

**This replaced a per-scanline approximation.** The old code called `StepFrameSequencer()` four times a frame from `MoonCore.Schedule` — within a scanline for the IRQ, useless for anything else. That call is gone; the APU counts its own cycles.

---

## 2. The frame sequencer

`$4017` picks a four- or five-step sequence. Boundaries, in CPU cycles since the last `$4017` write:

- **four-step**: quarter frames at 7457, 14913, 22371 and 29829, half frames at 14913 and 29829, and a frame IRQ across the end of the sequence unless bit 6 inhibits it.
- **five-step**: quarter frames at 7457, 14913, 22371 and 37281, half frames at 14913 and 37281, and **never an IRQ**.

Writing `$4017` with bit 7 set also clocks a quarter *and* a half frame, which is how a game forces an envelope reset at a known moment.

Quarter frames clock the three envelopes and the triangle's linear counter. Half frames clock the four length counters and both sweep units, and a half frame always clocks the quarter units with it.

Both of those lists are the *clocks*. The sequence they belong to is two cycles longer than its last clock and the IRQ is wider than one cycle — see §2.1, which is where the accuracy actually lives.

Reading `$4015` acknowledges the frame IRQ. The DMC IRQ is reported in bit 7 and is **not** cleared by that read — only `$4010` bit 7 going low, or `$4015`'s enable bit, clears it.

### 2.1 The sequence has six steps, not four

The four boundaries above are where the *clocks* happen. The sequence itself runs two cycles longer, and modelling it as a four-entry table that wraps at the last clock is wrong in a way that is very hard to see from the outside. `Apu.StepFrameCounter` therefore uses a six-entry table, taken from Mesen's `ApuFrameCounter`:

| | step 0 | 1 | 2 | 3 | 4 | 5 |
|---|---|---|---|---|---|---|
| four-step, cycle | 7457 | 14913 | 22371 | **29828** | **29829** | **29830** |
| five-step, cycle | 7457 | 14913 | 22371 | 29829 | 37281 | 37282 |
| clock | quarter | half | quarter | none | half | none |

A "half" entry clocks the quarter units too — envelopes and the triangle's linear counter — and then the length counters and both sweeps. That is one `FrameCounterTick` on hardware, and the two calls here are the same thing spelled out.

**The frame IRQ is asserted across all three of the last steps** in four-step mode — 29828, 29829 *and* 29830 — not on one cycle. That is the detail that made several tests unfixable from the outside.

**A retraction.** This section previously claimed the short period was deliberate and was "compensating for a larger error in the opposite direction", after correcting it in isolation regressed `apu_reset/4017_timing`. That was wrong, and it was wrong because it was reasoned from which tests moved rather than from what the hardware does. The period is not compensating for anything; the table was simply missing two entries and the IRQ window was a third the width it should be. With the real table, `4017_timing`, `4017_written`, `5-len_timing` and `6-irq_flag_timing` all pass and nothing regresses. The general rule this cost a lot of time to relearn is in `Moon_TestRoms.md` §5.

### 2.2 `$4017` lands late, and RESET rewrites it

Writing `$4017` does **not** restart the sequencer at once. The inhibit bit takes effect immediately, but the mode and the restart are held for three or four CPU cycles — three if the write landed on an APU cycle, four if it landed between two, which is decided by the parity of the CPU cycle count. `_pendingFrameValue` and `_writeDelayCounter` carry that; `FrameCounter` keeps the raw byte for `IrqInhibited` and for `regs`, while `FiveStepMode` reads the separately-applied `_stepMode`.

Selecting five-step mode clocks a quarter *and* a half frame the moment the write lands, not when it is issued.

`_blockFrameCounterTick` stops a second clock inside the two cycles after any tick, so a `$4017` write that arrives on top of a sequencer boundary cannot double-clock the length counters.

**RESET rewrites `$4017` with the mode it already had.** Power-on writes `$00`; the RESET line re-issues whatever mode was in force, through the same three-cycle delay. `Apu.SoftReset` therefore leaves `_stepMode` alone and only `Apu.Reset` clears it — which is exactly what `apu_reset/4017_written`'s third case checks. The other half of that test lives in `Moon_CPU.md` §5.1: the eight idle cycles a reset spends before its first instruction are what put this delayed write in the right place relative to the code that measures it.

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
- **DMC fetch cost is a flat 4 cycles** rather than the real 3-4 depending on alignment, and the fetch does not steal the specific cycle that corrupts a controller read on hardware — the bug some games are written to work around. This is what `apu_test`'s `7-dmc_basics` and `8-dmc_rates` still fail on, and it is the whole of what is left in that suite.
- **The sweep unit's reload subtlety** — a reload and a clock on the same half frame — follows the common implementation rather than being verified against hardware.

Per-channel mute **is** now wired: `Apu.SetChannelMuted` drops that channel's term in `Mix()`, and `MoonDebugTarget` forwards `IDebugTarget.SetChannelMuted` to it. It had been a field the debug target recorded and nothing read, so the dashboard's mute button silently did nothing — the same shape of bug as the audio meters, which reported a hardcoded level of 0 and the caption `"no synthesis"` long after the APU started making sound. Both now report the real thing: level is the channel's volume rescaled to 0-100 (the envelope for the pulses and noise, output level for DMC, on/off for the triangle, which has no volume control), matching the convention Venus's voices already used. `[SkipInState]`, because a mute is a debugging aid and not hardware.

## 7. The SMB3 "silence" was not a bug

*Partly superseded: see §8. The audio argument here stands; the pixel-identity claim at the end was measured against a reference that disagreed with itself and is withdrawn.*

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

## 8. Re-measured 2026-08-06, and the `$4010` gap that survived it

Everything in §7 was measured with a probe that read memory out of a running emulation thread — see `EmuSen_Debugging_Tools_Reference_v5.md` §3.45a. That does not overturn §7's argument, and it is worth being precise about why: the *audio* conclusions there rest on WAV captures and on register-write logs, neither of which the race touched, so "the title screen is genuinely silent" and "the engine starts on frame 501" both stand. What does not stand is the pixel comparison. The screen buffer at frames 180/240/300 was off by up to 287 pixels between two runs of the same binary, so "frame 900 is pixel-identical to Mesen's 897, 0 of 61 440 differing" was measured against a reference that disagreed with itself, and is withdrawn rather than merely re-stated.

Re-taken with the corrected probe, the picture is better than §7 claimed and more specific:

- **Video memory is byte-exact.** `nametable`, `palette`, `chr` and `oam` reproduce Mesen exactly across 75 consecutive frames.
- **The phase offset is two frames, not three, and it is provable rather than inferred.** Our nametable at frame 180 is byte-identical to Mesen's at 182, and the difference grows by exactly 32 bytes per frame either side — SMB3 scrolls its title 32 nametable bytes per frame. A linear gradient like that is proof the content is right and only the timing differs.
- **Internal RAM is not a defect.** Against a third emulator (Nestopia, through the libretro backend), Mesen and Nestopia disagree with *each other* by 9-39 bytes of 2048 at the same frames, and EmuSen sits inside that spread — closer to Nestopia than Mesen is, at three frames of five. See §3.46a for the table.

**The one real finding.** Over 1200 frames with Start tapped at 400/500/620, the register-write logs agree within six writes on every register except `$4010`, where Mesen writes **2377** and we write **1899** — a 478-write gap, roughly two per frame against our 1.6. `$4010` is the DMC rate/IRQ/loop register, and §6 above already names the DMC as this APU's weakest area: the fetch cost is a flat 4 cycles rather than the real 3-4 by alignment, and `7-dmc_basics` and `8-dmc_rates` are the only `apu_test` failures left. A conditional write path we take less often than the reference is consistent with that, and is the first hard number attached to it.

This is **not yet diagnosed** — it is a lead, recorded so the next pass starts from a measurement instead of from §7's "SMB3 is fine". Reproduce with `--apulog` on both sides and `--tracediff`, per §3.46.
