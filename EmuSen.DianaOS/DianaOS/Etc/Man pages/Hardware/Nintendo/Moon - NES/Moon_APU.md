# Moon (NES) — APU

Covers `Cores/Nintendo/Moon - NES/Apu/Apu.cs`.

---

## 1. Status: no sound

**Nothing is synthesized.** There are no pulse channels, no triangle, no noise, no DMC, no sweep units, no envelopes and no mixer. `ICore.DequeueAudioSamples` returns an empty array unconditionally and `IDebugTarget.GetAudioSamples` returns an empty snapshot.

`AudioSampleRate` still reports 44100 because a caller opening an output device needs a rate up front, before any samples exist to read one off — see `ICore.cs`'s own note on why that property is separate.

This is a deliberate first cut, not an oversight. The APU is a self-contained subsystem that no other part of the machine depends on for correctness, so leaving it out kept this pass on the two interfaces the work was actually about. Venus reached the same state in the other order — its S-DSP synthesized correct samples for a long time before anything played them.

---

## 2. What is modelled

Enough for a game not to hang or misbehave waiting on the APU:

- **The register file.** `$4000-$4013` are stored verbatim so the debug target can report what a game wrote.
- **Length counters.** `$4015` writes clear a channel's counter when its enable bit is low, and `$4015` reads report which counters are non-zero. Games do poll this.
- **The frame counter.** `$4017` sets the mode and clears the IRQ inhibit. In four-step mode the sequencer raises the frame IRQ on wrap; five-step mode never does.
- **IRQ acknowledgement.** Reading `$4015` clears the pending frame IRQ, which is how a handler dismisses it.

`MoonCore` feeds `Cpu.SetIrqLine` from `Apu.FrameIrqPending` OR'd with the mapper's IRQ, so a game that enables the frame IRQ really does get interrupted.

### 2.1 The sequencer runs at scanline resolution

`StepFrameSequencer` is called four times a frame, on scanlines that are multiples of 65. The real sequencer runs off the CPU clock at roughly 240 Hz, which is about 3.7 steps per 60 Hz frame in four-step mode.

Four steps a frame is right to within a scanline for IRQ *timing*, which is all it is used for here. It would not be adequate for driving envelopes and sweeps, so this is one of the things that has to change when synthesis is written — the value is picked for the one consumer that currently exists, not as a general model.

---

## 3. What a game will notice

Silence, and nothing else that has been observed. The length counters and frame IRQ are the two pieces of APU state games read back for logic rather than sound.

The DMC's sample-fetch DMA is **not** modelled, and that one does have a non-audio effect: on real hardware it steals CPU cycles and can corrupt a controller read in progress. Games affected by that are few and are documented as such in the wider NES literature; nothing here compensates for it.

---

## 4. When this gets written

The channels themselves are well-documented and largely independent, so the work is bounded. The parts that need care are the mixer's non-linear output tables, the DMC DMA's cycle theft (§3), and moving the frame sequencer onto real CPU-clock timing (§2.1) rather than the scanline approximation it uses now.

`EmuSen`'s resampler (`Audio/LinearResampler.cs`) and `DynamicRateControl` are core-agnostic and already used by Venus, so the output path is not new work — only the synthesis is.
