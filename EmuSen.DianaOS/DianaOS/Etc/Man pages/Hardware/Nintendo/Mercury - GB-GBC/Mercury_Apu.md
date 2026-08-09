# Mercury — the sound hardware

*Written 2026-08-08 with Phase C, the last phase of the Mercury gameplan. Mercury
makes sound as of this page.*

---

## 1. Four channels and a stereo mixer

Two pulse channels, one of which has a frequency sweep; a wave channel that plays
32 four-bit samples a game writes itself; and a noise channel built from a shift
register. Each produces a 4-bit digital level, each has its own DAC, and the four
DACs feed a two-channel mixer with per-channel panning and a three-bit master
volume per side.

The Game Boy is genuinely **stereo**, which is the first thing that distinguishes
this from Moon's 2A03 (`Moon_APU.md`), where both output channels carry the same
value. `NR51` is eight switches — each channel, each side — and a channel panned
to neither side is inaudible while still running.

## 2. The frame sequencer is a DIV bit, not a divider

The 512 Hz sequencer that drives length, sweep and envelope is **not** an
independent counter. It advances on the falling edge of a bit of the same 16-bit
counter DIV is the top half of (`Mercury_Memory.md` §5) — bit 12 of the counter,
which is DIV bit 4.

Modelling it that way rather than as its own divider is not extra fidelity for its
own sake; it makes a real behaviour fall out for free. **Writing to DIV zeroes the
counter, which can present a falling edge, which clocks the sequencer early.** In
Mercury the write clears `_divCounter`, the next cycle sees the bit low with the
previous sample high, and `OnDivBit` steps. A game that resets DIV in a tight loop
gets its length counters clocked faster than 256 Hz, exactly as on hardware, and
nothing in the APU had to know that DIV existed.

The eight steps are the usual ones: length on 0, 2, 4 and 6; sweep on 2 and 6; the
envelope on 7 alone. The envelope stepping *once* per full turn of the sequencer,
rather than on every step, is the most commonly misread part of this table.

### 2.1 What double speed does to it

Nothing, in effect, and that is the point. On a CGB in double speed the CPU and
DIV run twice as fast, so the sequencer would run at 1024 Hz if it kept watching
the same bit. It watches bit 13 instead, and comes out at 512 Hz again.

The channel frequency timers are a separate question with the same answer: they
run at the base 4.19 MHz clock, so `Apu.Tick()` is called from the same
half-rate gate as `Ppu.Tick()` (`Mercury_Cgb.md` §5). Double speed must not change
the pitch of a game's music, and a test asserts that it does not.

## 3. The channels

### 3.1 The envelope

Three of the four channels share it: an initial volume, a direction, and a period
in steps of the sequencer's turn.

**A period of zero is off, not "step every turn".** This is the one that produces
a plausible-looking bug — a note that fades when it should hold — and it is worth
stating as a rule because the reload value in that case is 8, which invites the
reading that it steps every eight turns. It does not step at all.

The envelope also stops permanently once it walks off either end rather than
clamping and continuing, which is why `Finished` exists as a flag separate from
the volume reaching 0 or 15.

### 3.2 The length counter

Counts down at 256 Hz to silence. Sixty-four steps on the pulse and noise
channels, **256 on the wave channel** — a different maximum, not a different
mechanism, which is why one class takes it as a constructor argument.

Two rules that are easy to invert: the counter only runs when `NRx4` bit 6 enables
it, and a trigger with the counter already at zero reloads it to full. The second
is how a game retriggers a note that has already expired.

### 3.3 The sweep

Channel 1 only. It keeps a *shadow* copy of the frequency and walks that, writing
each new value back to the channel — so a game reading `NR13`/`NR14` back sees the
sweep's work.

Overflowing eleven bits **disables the channel**; it does not wrap and it does not
clamp. Two details of when that check runs are modelled and both are real:

- The overflow check runs **at trigger time**, before any sweep step, whenever the
  shift is non-zero. A channel can therefore be disabled by the very write that
  started it.
- After a successful step, the check runs **a second time** with the new
  frequency. That second check can disable the channel without the new frequency
  ever being heard.

A sweep period of zero keeps the timer reloading at 8 but performs no step, which
is not the same as the sweep being off.

### 3.4 The wave channel

Sixteen bytes of RAM at `$FF30-$FF3F`, read as 32 four-bit samples, high nibble
first. Its DAC is `NR30` bit 7 rather than an envelope's top bits, and it has no
envelope at all — its only volume control is `NR32`, and that is a **shift**:
code 0 is silence, then 100%, 50% and 25%. There is nothing between those.

Wave RAM survives a power cycle of the APU. Every other register does not (§4).

Hardware returns the byte the channel is currently reading when the CPU reads wave
RAM while the channel is playing; Mercury returns the stored byte. No game is
known to depend on the difference, and depending on it would require
cycle-accurate knowledge of the channel's position that the CPU cannot reliably
obtain anyway.

### 3.5 The noise channel

A 15-bit linear feedback shift register, XOR of the low two bits fed back into the
top. Short mode feeds the same bit into bit 6 as well, which cuts the sequence
from 32767 steps to 127 — audibly a metallic pitch rather than a hiss.

The register **starts all ones** and the channel is loud when bit 0 is *low*. Both
halves of that matter: starting at zero would make the sequence degenerate, and
inverting the output test makes the channel loud exactly when hardware is quiet.

The period is one of eight divisors shifted left by `NR43`'s top nibble. Divisor 0
is 8, which is *half* of divisor 1's 16 — the table is not a straight multiple of
the index, and treating it as one puts the lowest setting an octave out.

## 4. Power

`NR52` bit 7 is the master switch. Turning it off clears every register and
silences every channel; turning it on restarts the frame sequencer at step 0, so
the first length clock is a known distance away.

While the power is off, **every register except `NR52` ignores its write.** Wave
RAM is the exception that is not a register: it stays.

One documented divergence: on a DMG the length counters remain writable while the
power is off, and on a CGB they do not. Mercury implements the CGB rule for both,
because the DMG exception exists to support a specific power-up trick that no
commercial game uses and because a single rule is easier to reason about than a
model-dependent one. This is recorded rather than hidden — if a DMG test ROM ever
fails on it, this paragraph is the place to start.

## 5. The mixer

Each channel's DAC maps its 0-15 digital level onto −1.0 … +1.0, the sides are
summed, divided by four, and scaled by `(volume + 1) / 8`.

### 5.1 A live DAC at digital zero is not silence

This is the part that surprises people and it is the reason §5.2 exists. A channel
whose DAC is *enabled* but whose digital output is 0 sits at −1.0, the bottom of
the swing — not at the centre. Only a channel whose DAC is *disabled* contributes
nothing.

So four channels with live DACs and nothing playing produce a full-scale negative
DC offset, not silence. That is what real hardware does too, and it is why a naive
mixer sounds like a loud click every time a game enables its DACs.

Muting, from the debug target, drops a channel's contribution to zero — the centre
— rather than to its DAC's floor. That is a deliberate difference from a channel
being disabled, and it matches the semantics `IDebugTarget.SetChannelMuted`
documents: the channel keeps running, it is just excluded from the mix.

### 5.2 The high-pass filter

A one-pole high-pass per side removes the offset from §5.1, with the same
`0.999958`-per-clock charge constant the console's own output capacitor has,
raised to the number of clocks between output samples. Without it the output is a
DC step rather than a waveform, and a test asserts that a steady live DAC settles
back to a mean near zero over thirty frames.

## 6. Output

Samples are produced at the core's fixed 44100 Hz by a fractional accumulator
against the 4194304 Hz clock — 95.11 clocks per sample, with the remainder carried
so the rate cannot drift. One frame yields about 738 stereo frames.

`Drain` is destructive and matches `ICore.DequeueAudioSamples`; `Peek` is a
non-destructive snapshot for `audiodump`. When the buffer is full the *oldest*
pair is dropped, so a frontend that stops draining loses history rather than the
present — the same choice Moon made and for the same reason.

The pending sample queue and the filter's state are `[SkipInState]`. A save state
therefore restores every channel exactly and restores no queued audio, which is
correct: the samples in the queue have not been played yet and belong to the
timeline being abandoned.

## 7. What is not modelled

- **No sub-sample channel timing.** Channel timers advance one T-cycle at a time,
  which is the hardware's own resolution, but the *mixer* only samples them at
  44100 Hz rather than filtering the full-rate signal. A 4 T-cycle pulse period is
  above Nyquist for that rate and will alias. No game plays notes there.
- **The length-counter obscure behaviours** around a trigger landing on a
  sequencer step that also clocks length are not modelled. They change a note's
  duration by one 256 Hz tick.
- **No `PCM12`/`PCM34` (`$FF76`/`$FF77`).** The CGB's read-only channel-output
  registers exist for debugging and no game reads them.
- **The DMG's writable-length-while-off exception**, as §4 records.
