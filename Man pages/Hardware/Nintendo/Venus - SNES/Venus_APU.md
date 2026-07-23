# Venus (SNES) — APU (SPC700 + S-DSP)

Covers `Cores/Nintendo/Venus - SNES/Apu/`: the SPC700 core (`Spc700.cs`, addressing modes, opcode table, opcodes) and the S-DSP sound chip (`SDsp.cs`, `DspVoice.cs`, `BrrDecoder.cs`). See `Man pages/Hardware/README.md` for what this doc set is.

The SNES APU is a genuinely separate computer: its own CPU (SPC700), its own 64KB RAM, its own sound chip (S-DSP), talking to the main CPU only through 4 shared I/O ports (`$2140-$2143` on the CPU side — see `Venus_Memory.md` §1.2). Everything in this page lives entirely on the APU side of that boundary.

---

## 1. SPC700 core

### 1.1 Boot sequence and the IPL ROM

`Spc700.Reset()` copies a hardcoded 64-byte IPL (Initial Program Loader) ROM image into `Ram[0xFFC0..]` and sets `PC = 0xFFC0` — real hardware boots from this same fixed, undumpable-by-software boot ROM every power-on, which is why it's baked into this emulator as a literal byte array rather than loaded from a file. The IPL ROM's job (on real hardware and here identically, since it's the same bytes) is to hand-shake with the main CPU over the 4 ports and receive the actual sound driver/data upload before jumping into it.

### 1.2 Communication ports

`ReadPort`/`WritePort` (called from `MemoryBus`, see `Venus_Memory.md` §1.2) are the CPU-side view of `_inPorts`/`_outPorts`; `Read8`/`Write8` at `$00F4-$00F7` are the APU-side view of the *same* 4 ports. `_inPorts` is what the CPU wrote and the SPC700 reads; `_outPorts` is what the SPC700 wrote and the CPU reads — two independent one-way latches per port index, not a shared register.

`LogPortTraffic` (off by default, flipped on a few frames into boot by `Program.cs`) logs only on **value changes**, not every read/write — the driver's idle polling loop re-reads the same port value every cycle while waiting for the next command, and logging that unconditionally would flood the console during the ~24k-byte initial upload handshake before any real investigation could see the interesting transitions.

### 1.3 Timers

Three hardware timers (`$00F1` control, `$00FA-$00FC` targets, `$00FD-$00FF` counters), each with its own internal prescaler:

- Timer 0 and Timer 1 tick their internal counter every 128 S-SMP cycles.
- Timer 2 ticks every 16 cycles (matching real hardware's faster timer 2).
- When the internal counter reaches its target byte, the corresponding 4-bit output counter increments (wrapping at 16) and the internal counter resets.
- **Reading a counter register clears it to 0** — real hardware behavior, not just "read the current value."
- `$00F1` bits 4/5 clear the CPU→APU input port latches (`PC10`/`PC32` in hardware terminology) — a documented side effect of writing the timer control register, not related to timers themselves.
- Disabling a timer (clearing its enable bit in `$00F1`) resets that timer's internal prescaler and counter to 0, so re-enabling it starts fresh rather than resuming mid-cycle.

### 1.4 WAI/STP equivalent — SLEEP/STOP

`SLEEP`/`STOP` set `_halted`, which only clears in `Reset()` — same "only a hardware reset wakes it" behavior as the 65816's `STP` (see `Venus_CPU.md` §4). While halted, `Step()` still consumes cycle budget (returns as if 2 cycles ran) so the caller's cycle accounting keeps advancing, but fetches/executes nothing.

### 1.5 One-shot dispatch-chain tracer

`TraceMilestone`/the `switch (PC)` block in `Step()` are a narrow, purpose-built trace for the SMW sound-driver boot sequence specifically (hardcoded PC addresses `$05A5`, `$0816`, `$09E5`, `$FFC0`), capped at 6 hits per milestone. This isn't general APU debugging infrastructure — it's a leftover from investigating a specific boot handshake, gated behind `LogPortTraffic` so it doesn't fire in normal operation. A future core-agnostic equivalent would be the `trace`/`watch` commands in the core-agnostic debug toolchain (`EmuSen_Debugging_Tools_Reference_v5.md` §3), not more hardcoded PC checks like this one.

---

## 2. SPC700 instruction set notes

### 2.1 `dd,ds` ALU family — operand order is not what the mnemonic suggests

For `OR`/`AND`/`EOR`/`ADC`/`SBC`/`CMP`/`MOV dd,ds`, the addressing mode fetches and resolves the **first** operand byte in the instruction stream — which is actually the **source** address (`ds`), despite the mnemonic being conventionally written "dd, ds" (destination first). This is a real, documented SPC700 byte-encoding quirk, not a bug: the second operand byte (destination, `dd`) is fetched separately from `PC` inside each handler. `MOV dd,ds` specifically is documented as "(no read)" of the destination — unlike the read-modify-write ALU ops in the same family — though this doesn't change behavior here since nothing in this emulator models bus read side-effects.

### 2.2 `(X),(Y)` family

`(X) = (X) OP (Y)` — both operands are direct-page addresses given by the X and Y registers themselves (SPC700's `(X)`/`(Y)` notation means "direct page address = register value," the same convention `AddrIndirectX` uses for the single-operand `(X)` forms). Implemented by hand rather than through the shared `Func<ushort> AddrMode` table shape, since these opcodes need both addresses at once.

### 2.3 `m.b` single-bit family

`AND1`/`OR1`/`EOR1`/`MOV1`/`NOT1`'s operand packs a 13-bit absolute memory address (bits 0-12) and a 3-bit bit index (bits 13-15) into one 16-bit operand word — verified against documented hardware encoding (sneslab.net's MOV1 page). `AddrMemBit` returns the packed word directly (fits the existing `Func<ushort> AddrMode` signature with no interface change); each `Op*1` handler unpacks it (`addr = raw & 0x1FFF`, `bit = (raw >> 13) & 0x07`).

### 2.4 DAA/DAS

Notoriously easy to get subtly wrong — implemented against a structure independently verified in the community against real SPC700 test vectors (Overload/anomie, cross-checked on the ZSNES forums' "SPC700" thread), rather than a generic x86-style DAA/DAS, which uses different flag polarity. `DAS` mirrors `DAA` with inverted flag checks (H/C clear = borrow occurred), consistent with how this codebase's own `ADC`/`SBC` already treat H/C for addition vs. subtraction.

---

## 3. S-DSP register handling (`SDsp.cs`)

### 3.1 Register decode split

`SDsp` owns register decode (raw bytes → the fields `DspVoice` actually uses), KON/KOFF edge detection, ENDX, and MVOL-scaled final mixing. BRR decoding and per-voice envelope logic live in `DspVoice` (§4) — the split mirrors real hardware's own division between the shared register file and per-voice logic.

### 3.2 ENDX (`$7C`)

Read returns the *live* per-voice end flags (one bit per voice, set when that voice's BRR playback reaches an end-marked block with no loop — see §4.3). **Any write to `$7C` clears all 8 bits, regardless of the value written** — documented hardware behavior, not a typo; games poll-and-acknowledge this register as a whole, not per-bit.

### 3.3 KON/KOFF — edge-triggered, not level-triggered

`ProcessKeyEvents` fires `KeyOn`/`KeyOff` on a bit **newly set since the last sample**, not on every sample where the bit happens to be set — so a game holding a KON bit set across multiple register writes doesn't re-key (restart) the voice every sample. Real hardware processes KON/KOFF roughly every 64 S-SMP clocks (fullsnes); gating on the same per-generated-sample cadence `GenerateSample` already runs at is close enough without modeling that separately. **If both KON and KOFF are newly set for the same voice in the same sample, KeyOff wins** — matches documented hardware behavior (key-on immediately followed by key-off silences the channel).

### 3.4 Muted/disabled audio still advances playback state

When `AudioSettings.AudioEnabled` is false or muted, `GenerateSample` still calls `voice.GetNextSample()` for every voice (discarding the result) rather than skipping voice processing entirely — so playback position doesn't "jump ahead" the instant audio is re-enabled mid-sound.

---

## 4. `DspVoice` — BRR playback + ADSR/GAIN envelope

### 4.1 Not implemented (documented, not silently wrong)

- **Echo** (EON, FIR filter, echo buffer) — entirely separate subsystem, not present at all.
- **Noise generation** (NON) — voices always play their BRR sample regardless of this bit.
- **Pitch modulation** from the previous voice's output (PMON).
- **Gaussian interpolation** — real hardware uses a 4-tap Gaussian filter when resampling; this uses nearest-neighbor instead, which gets the playback *rate* right but sounds rougher than hardware on pitched-up/down samples.
- **Exact period-table phase alignment across voices** (§4.4) — envelope *rates* are correct, exact per-clock *phase* isn't.

### 4.2 Loop blocks keep prediction-filter history

When a BRR block's loop flag is set, `AdvanceSourceSample` jumps `_blockAddr` to `_loopAddr` **without** resetting the `BrrDecoder`'s `_prev1`/`_prev2` filter history — real hardware carries the prediction filter straight into the loop block rather than resetting it, same as it would for any other mid-stream block transition. Whether this matters audibly depends on the loop block's own filter type.

### 4.3 End-of-sample vs. loop

A BRR block's header carries independent end and loop flags (see `BrrDecoder.IsEndBlock`/`IsLoopBlock`, §5). On reaching an end-marked block: loop flag set → jump to `_loopAddr` (§4.2) and keep playing; loop flag clear → voice stops, `Ended` is set (feeding ENDX, §3.2).

### 4.4 Envelope rate gating (`RateDue`)

The 32-entry period table (`PeriodTable`, verified against the SNESdev DSP_envelopes page) is denominated in S-SMP clocks per envelope step; index 0 ("Infinite") means that stage never advances on its own. `RateDue` approximates this in **output samples** rather than raw clocks — 32 S-SMP clocks per generated sample, matching `SDsp.Tick`'s own per-sample timing exactly, so the *rate* this produces is correct; only the exact clock-level *phase* relative to other voices isn't modeled (documented gap, §4.1).

### 4.5 ADSR vs. GAIN mode

`Adsr1` bit 7 selects the mode: set → ADSR envelope (`StepAdsr`, four stages: Attack/Decay/Sustain/Release, each with its own rate-table index derived from `Adsr1`/`Adsr2` bits); clear → direct GAIN control (`StepGain`, either an immediate 7-bit value or one of 4 rate-gated modes: linear/exponential decrease, linear/"bent line" increase). **Release always uses a fixed rate every sample** (`_envelope -= 8`), with no period-table gating, regardless of which mode (ADSR or GAIN) was active before key-off.

**Not independently cross-checked** (flagged honestly rather than silently trusted): the direct-gain-mode scaling factor (`(Gain & 0x7F) * 16`) is the one detail in this file not verified against a second source beyond the primary implementation. Worth revisiting first if a direct-gain-mode sound effect sounds off.

---

## 5. `BrrDecoder` — BRR (Bit Rate Reduction) sample decoding

Decodes the compressed format every SPC700 sample-based instrument is stored in. Algorithm confirmed against the SNESdev wiki's BRR_samples page rather than reconstructed from memory — the filter coefficients are precise fractions (e.g. 61/32, 115/64) that are easy to get subtly wrong from memory alone.

- **One instance per voice**, holding the rolling `_prev1`/`_prev2` prediction-filter state — reset to 0 at the start of a new sound (a filter-0 block, which every sample must start with, ignores these anyway).
- **Shift 13-15 is a documented hardware quirk**, not a normal magnitude: rather than an actual left shift by 13-15 bits, real hardware collapses this case to just the sign of the nibble scaled up (`raw = nibble < 0 ? -2048 : 0`).
- **The 4 filter modes** use specific fractional coefficients relative to the two previous samples (filter 0: no prediction; filter 1: `prev1 * 15/16`; filter 2: `prev1 * 61/32 - prev2 * 15/16`; filter 3: `prev1 * 115/64 - prev2 * 13/16`), each implemented as an integer shift-and-subtract combination that reproduces the fraction exactly.
- **Accumulator overflow wraps, not saturates**: real hardware's prediction accumulator overflows by wrapping through a 16-bit signed range, not clamping — a plain `(short)` cast on the predicted value reproduces that truncation/wraparound behavior exactly, which matters for BRR data that's (deliberately or not) right at the edge of representable range.
