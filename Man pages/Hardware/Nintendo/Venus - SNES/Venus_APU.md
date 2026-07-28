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

Notoriously easy to get subtly wrong, and this project's original implementation *was* wrong in one specific way despite being written against a structure believed correct at the time (a community-verified writeup, Overload/anomie, cross-checked on the ZSNES forums' "SPC700" thread): the high-byte check (`A > $99` or `C`/`!C`) was evaluated against `A`'s value **after** the low-nibble adjustment immediately above it had already run, instead of against the original operand. Real hardware's adjust logic evaluates both conditions off the original value - it doesn't chain them sequentially. Concretely, `A=$9C, H=1, C=1` needs both adjustments to fire (off the original $9C) to reach the correct `$36`; chaining them re-checks the already-adjusted `$96` against the `>$99` threshold and wrongly skips the second adjustment. Found and fixed via the ground-truth suite (§1.6) - both `OpDAA_A` and `OpDAS_A` now save `A`'s original value before the low-nibble adjustment and test that saved value in the high-byte check. `DAS` still mirrors `DAA` with inverted flag checks (H/C clear = borrow occurred), consistent with how this codebase's own `ADC`/`SBC` already treat H/C for addition vs. subtraction - that part was always right.

### 2.5 Direct-page 16-bit accesses wrap within the page, not past it

`CMPW`/`ADDW`/`SUBW`/`MOVW`/`INCW`/`DECW YA,dp` and the pointer fetch inside `[dp]+Y` indirect-indexed addressing all read two consecutive bytes starting at a direct-page address. Real hardware wraps the *high* byte's offset within the same 256-byte page rather than spilling into the next one - `dp=$FF, P=0` reads its high byte from `$00`, not `$100`. `AddrDirectPageIndexedXIndirect` (used by `[dp+X]` addressing) already wrapped this correctly; the seven other call sites used a plain `address + 1` instead, which is right for PC-relative and absolute addressing (no page to wrap within) but wrong once `address` is already page-based. Fixed via a shared `DpWrapNextByte()` helper in `Spc700.AddressModes.cs`, applied everywhere a direct-page word op reads or writes its second byte. Found via the ground-truth suite (§1.6) - every one of these opcodes' failures clustered on dp offsets near a page boundary (`$xFF`/`$x00`), the exact signature of this bug.

### 2.6 Ground-truth validation status

`EmuSen.Tomoe`'s `spc700` target (`Spc700SingleStepTarget`/`Spc700TestLoader`) was added alongside the 65816 harness but not exercised until this was run against all 256 TomHarte/ProcessorTests spc700 opcode files (256,000 cases): 255106/256000 passing after the two fixes above (§2.4, §2.5). The remaining ~894 "failures" are a single, well-understood, non-bug cause: TomHarte's vectors model a flat 64KB RAM with no memory-mapped I/O, so any test case whose randomly-generated address happens to land in `$F0-FF` collides with this project's (correct) hardware-register behavior there (DSP register access, timer counters, APU communication ports - see §1.2/§1.3) instead of the test's plain-RAM expectation. `Spc700SingleStepTarget.SetMemory`/`GetMemory` already special-case `$F4-F7` for exactly this reason (see that class's own comment); `$F0-F3`/`$FD-FF` just aren't plugged into that same seeding yet - a real gap in the *test harness*, not in the emulator, and low priority since it doesn't affect anything a real ROM does.

### 2.7 Known open issue: Super Metroid boot hang

A Super Metroid ROM hangs before its title screen, permanently stuck in a `BRA`-to-self trap in the 65816's own boot code (`$80:8077`) after a `DEC A : BNE` countdown times out waiting for the SPC700 to re-present the standard `$AA/$BB` IPL ready-signal on `$2140/$2141` for a *second* audio-upload stage (the first stage's upload, using the same shared routine, completes successfully). Investigated at length without finding the root cause; ruled out so far:

- **CPU↔SPC700 relative pacing** - tested directly by temporarily running the SPC700 at 7x and then 21x its normal rate (via `VenusCore.RunFrame`'s `scaledSpc700Cycles` scaling); the hang reproduced identically both times.
- **General-DMA cycle starvation** - fixed as a real bug regardless (`Venus_Memory.md` §3.1), confirmed via direct comparison to have zero effect on this specific hang (identical CPU/SPC700 register state at the hang point before and after).
- **SPC700 opcode correctness** - the two real bugs in §2.4/§2.5 were found chasing this hang; fixing them had no effect on it either.

At the hang point, the SPC700's own PC sits at `$FFDC` - still inside the stock IPL ROM, in the `CMP $F4,#$CC : BNE` loop waiting for the CPU to send the transfer-start byte, which the CPU never sends because it's stuck on its own earlier check. Next angle not yet tried: tracing exactly what happens between the first upload's completion and the second one's `JSL` call site (`$80:8F7E`, reading its pointer table from bank `$8F`) - in particular whether anything should be resetting `$F1` (timer control, which also clears the CPU→APU input latches per §1.3) or otherwise re-arming the IPL-style handshake between the two stages, since the second call clearly expects the SPC700 to be back in its post-reset "waiting to receive" state and it never gets there.

### 2.8 Fixed bug: CPU→SPC700 cycle-scaling used the wrong master-clock ratio, making audio play ~1.3x too fast

Reported as "you can hear part of the music, but it's going too fast and there's missing sounds." Confirmed via `EmuSen.Pharaoh`'s `audiodump` verb (added this same session): a steady-state 5-frame window produced ~6564 interleaved audio samples where ~2662 (5 frames x 532.45 stereo pairs/frame x 2, at the real NTSC ~60.0988Hz frame rate and 32000Hz audio rate) was expected - a ~2.47x overshoot in that particular measurement, consistent with the SPC700 running its own clock too fast relative to real elapsed video-frame time.

Root cause: `VenusCore.RunFrame`'s `scaledSpc700Cycles` calculation (added by the general-DMA-cycle-charging fix, `Venus_Memory.md` §3.1) converted `Cpu.Step()`'s returned CPU-cycle-units into master-clock-equivalent units by multiplying by 8, on the stated assumption that "1 CPU cycle = 8 master clocks (SlowROM)". That assumption was never checked against what this project's *own* video timing already assumes: `VenusCore.CyclesPerScanline = 227` (x 262 scanlines = 59474 CPU-cycle-units/frame), matched against the real NTSC frame rate and master clock (~357366 master clocks/frame), implies **1 CPU-cycle-unit is worth ~6 master clocks here**, not 8 - this project's scanline timing was effectively calibrated to FastROM's ratio from the start, uniformly, with no per-region tracking (same "doesn't yet track FastROM/region speed separately" limitation the removed comment already called out, just not connected to this specific number before). Multiplying by 8 instead of 6 fed the SPC700 roughly 8/6 = 1.33x too many of its own cycles per real elapsed video frame, which is exactly what made the SPC700's clock (and therefore its audio output) run fast relative to the game's own, correctly-timed video frame rate - and once `SDsp`'s `AudioBufferMaxSamples` cap started dropping the oldest overflowing samples, that's what made specific notes sound like they were missing rather than just pitched up.

**Fix**: changed the multiplier from 8 to 6 in `VenusCore.RunFrame`'s `scaledSpc700Cycles` line, matching what `CyclesPerScanline` already implies. Verified via the same `audiodump`-based measurement: steady-state 5-frame windows now produce ~4924 samples against the ~5324.5 expected (a ~7.5% *undershoot*, plausibly the remaining error from this project's per-opcode cycle counts not modeling real per-region wait-state variation - see the disassembler/opcode-table caveats elsewhere in this doc set) - a small residual, not the ~2.47x overshoot from before - and confirmed the audio buffer no longer saturates its 1-second cap within a full second of emulated frames (56324/64000 samples at frame 60, where it previously hit the 64000 cap by roughly frame 45-50).

**Not yet revisited**: `Dma.PendingCpuCycles`'s own DMA-cost formula (`Venus_Memory.md` §3.1) was written under the same now-corrected "1 CPU-cycle-unit = 8 master clocks" assumption ("real hardware charges ~8 master cycles... so the charge is 1 + bytes"). Given the actual ratio is ~6, that formula likely undercharges DMA's true cost in CPU-cycle-units (should be closer to `(1 + bytes) * 8/6` to represent the same real master-clock cost) - a smaller, separate inaccuracy from the one fixed here, left alone for now since it wasn't the reported symptom and touches scanline/frame timing (already independently confirmed correct via the cave/Yoshi/blocks/coins fixes) rather than audio pacing specifically.

**Superseded by §2.9 below.** The flat `*6` multiplier this section describes fixing is gone entirely now - replaced by real per-instruction, per-region master-clock accounting (`Venus_CPU.md` §8). `Dma.PendingCpuCycles`'s "not yet revisited" note above turned out to resolve itself as a side effect: `Cpu.cs`'s `DrainPendingDmaCycles()` now multiplies by 8 to convert into real master clocks directly, matching the DMA formula's own stated per-unit cost exactly, with no `8/6` correction needed since there's no longer a separate CPU-cycle-unit scale for it to be relative to.

---

### 2.9 Re-investigated and resolved: SMW coin-chime/title-music audio report — MesenCE comparison, the master-clock rewrite, and the real root cause

Reported this session: "[SMW's Nintendo bootloader coin chime] sounds off and when you get to the title screen, there's breaks in the music audio." Investigated by comparing this project's S-DSP/APU implementation directly against the newly-added MesenCE reference fork (`RedQuE3n/MesenCE-refernece`, see `EmuSen_Project_Overview_v2.md`'s licensing note) rather than documentation alone.

**Ruled out, confirmed correct against MesenCE:**
- **Gaussian interpolation** (`DspInterpolation.cs`) - programmatically diffed against MesenCE's `Core/SNES/DSP/DspInterpolation.h`: the 512-entry `gauss[]` table is byte-for-byte identical (0 diffs), and the 4-tap interpolation formula (including the "wrap first 3 terms at 15-bit signed, then clamp full sum" quirk) matches exactly. See §4.1's corrected note - this project's own docs had gone stale claiming this was still nearest-neighbor.
- **ADSR envelope math** (`DspVoice.StepAdsr`) - same standard formulas MesenCE uses; the coin chime's fast Attack→Decay-to-zero-Sustain shape, ending via an early Release-stage KeyOff, is a plausible/correct one-shot sound-effect envelope shape, not a bug.
- **A measurement artifact, not a real bug**: an initial pass mistakenly used a fixed 1600-sample analysis window per `audiodump` capture, intended to represent "the last ~10 emulated frames" but actually covering under 2 frames (real measured rate is ~980-985 samples/frame, not the ~160/frame the window size implicitly assumed) - made normal note-to-note silence in the title theme look like a severe periodic dropout bug. Recomputing the window using the real measured samples/frame rate showed the title theme's silence/activity pattern is normal music, not a dropout.
- **Not investigated further, flagged for awareness only**: a genuine ~2+ second stretch of total digital silence between the coin chime ending (~frame 90) and the title music starting (~frame 230) - not confirmed as a bug, not yet compared against real hardware/MesenCE timing for the same boot sequence.

**The real, quantifiable finding**: a systematic ~7.5-8.7% SPC700 audio-sample-generation-rate undershoot, measured via `EmuSen.Pharaoh`'s `audiodump` verb across short cumulative bursts from a fresh (unsaturated) audio buffer (1/5/10/20/40-frame checkpoints) to avoid the buffer-cap sampling artifact above: ~980-985 interleaved samples/emulated-frame actual, vs. ~1065.36/frame expected (`32000Hz * 2 stereo / 60.0988fps`) - present both before and after every change described below.

**Hypothesis tested and disproven**: that this gap was caused by `VenusCore.RunFrame`'s flat "1 CPU-cycle-unit = 6 master clocks" scaling (§2.8) implicitly assuming FastROM timing for a SlowROM game, undercounting real elapsed master-clock time. Implementing the real fix - dynamic, per-instruction, per-region master-clock accounting (`Venus_CPU.md` §8, replacing the flat multiplier entirely) - and re-measuring with the identical `audiodump` methodology produced **no measurable change** (984.7 samples/frame after, vs. 980-983 before). Understanding why, after the fact: `VenusCore.CyclesPerScanline` (`227` before, `1364` after §8's rescaling) was in both cases back-derived specifically to make the *aggregate* per-frame budget match the real NTSC master-clock total (~357366/frame) - which is tautological with respect to per-instruction accounting accuracy. Whatever internal unit-conversion scheme is used, the scanline loop's `while (_lineCycles < CyclesPerScanline)` condition self-corrects to the same total budget per frame by construction; only per-scanline *overshoot variance* differs between schemes, not the frame total. So the flat-multiplier theory, while plausible-sounding and consistent with the multiplier bug §2.8 genuinely did fix (that one *was* a real, aggregate-budget-changing bug, since 8 vs. 6 is a different total, not just a different per-instruction split), was never going to explain a gap that persists under a correctly-calibrated total either way.

**One confirmed, real, related bug found and fixed along the way** (not the root cause, but a genuine correctness gap in the same area): `Spc700.Step()`'s SLEEP/STOP halt path (§1.4) decremented `CycleBudget` without calling `Dsp.Tick()`/`TickTimers()` at all - real hardware's DSP is a separate chip that keeps generating samples (and the timers keep ticking) independently of whether the SPC700 core itself is halted. Fixed by ticking both for the same 2 cycles the halt path already charges. Verified via `audiodump` to have zero effect on SMW's specific measured undershoot (SMW's sound driver evidently never executes SLEEP/STOP during the boot/title sequence measured), but left in since it's unambiguously correct and could matter for a driver that does idle via STOP.

**Root cause, found by direct instrumentation rather than more formula comparison**: temporarily counting three things independently (total master clocks `Cpu.Step()` produced, total SPC700 cycle budget `VenusCore` added, and total cycles the SPC700 actually ticked through to `Dsp.Tick()`/`TickTimers()`) over the same real frames showed `Spc700.CycleBudget` draining to ~0 every scanline exactly as intended (so the CPU→SPC700 hand-off itself was never the problem, confirming the "aggregate budget" reasoning above), but the *cumulative cycles reaching the DSP* fell further and further behind the cumulative budget added - a real, growing loss, not a rounding artifact.

Traced to seven opcode handlers in `Spc700.Opcodes.cs` (`TakeBranch` - the shared BCC/BCS/BEQ/BNE/BMI/BPL/BVC/BVS handler - plus `OpBranchBit`, `OpCBNE_dp_rel`, `OpDBNZ_Y`, `OpDBNZ_dp`, `OpBCC`, `OpBCS`): each one's real, correctly-documented "+2 cycles if the branch is taken" penalty was applied by decrementing `CycleBudget` **directly from inside the opcode handler**, bypassing `Step()`'s own `TickTimers(inst.Cycles)`/`Dsp.Tick(inst.Cycles)` calls entirely (those only ever saw the opcode's *base* cost, `SpcInstruction.Cycles`, never the dynamic penalty). Exactly the same category of bug as the SLEEP/STOP fix above, but far more consequential: a real sound driver's dispatch/polling loops execute a *lot* of taken conditional branches, so this was a small, continuous, compounding leak on essentially every frame, not a rare edge case.

**Fix**: the same side-channel pattern used for the 65816's addressing-mode penalties (`Venus_CPU.md` §8.4). All seven handlers now report the penalty through a new `_branchExtraCycles` field instead of touching `CycleBudget` directly; `Step()` reads it once, after `Operate()` returns, and folds it into the *same* total (`inst.Cycles + _branchExtraCycles`) it feeds to `TickTimers()`, `Dsp.Tick()`, and the `CycleBudget` decrement - so the DSP, the timers, and the budget accounting can never disagree about how many cycles a branch actually cost.

**Verified**: re-measuring SMW with the identical `audiodump` methodology took the rate from ~980-985 samples/frame (the ~7.5-8.7% undershoot) to ~1072/frame against the ~1065.36/frame expected at real NTSC timing - within ~0.7%, and in the *opposite* direction (a small overshoot, not undershoot) from the multiplier-fudge this section's earlier revision would have applied. Re-run against the TomHarte/ProcessorTests spc700 suite: 255106/256000, exactly matching the pre-existing baseline (§2.6) - no regressions, since the fix only changes *when* a real cycle cost reaches the DSP/timers, not any instruction's actual register/memory/PC behavior.

**Remaining ~0.7% residual**: not chased further - plausible candidates (not yet confirmed) include the SPC700's own crystal being approximated as `master/21` rather than its real, independent ~24.576MHz oscillator (a ~0.12% source, too small alone to explain the rest), scanline-count rounding, or simply normal measurement noise from the short-burst `audiodump` methodology itself. Worth a second look only if a future report suggests audio is still audibly off after this fix.

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

### 3.5 DSP debug toolchain (`channels`, `mute`, `regs`'s DSP_* rows)

Added diagnosing the "part of the music is missing" report (§4.4) - previously the debug toolchain could only see the *final mixed* audio output (`audiodump`/`GetAudioSamples`) or a KeyOn event as it happened (console-only `DebugSettings.DspKeyOnLogging`); neither answers "is voice N active right now, and what's its envelope actually doing."

- **`regs`'s APU section** now also reports the S-DSP's global (non-per-voice) registers - `DSP_MVOLL/R`, `DSP_EVOLL/R`, `DSP_EFB`, `DSP_KON`, `DSP_KOFF`, `DSP_ENDX`, `DSP_EON`, `DSP_NON`, `DSP_PMON`, `DSP_DIR`, `DSP_FLG`, `DSP_ESA`, `DSP_EDL` - via `SDsp.PeekRegister()`, a non-mutating read that (unlike `ReadRegister()`) never touches `_registerAddress`, so inspecting DSP state can't disturb whatever multi-step address/data sequence the actual sound driver is mid-way through. Used to directly rule NON/PMON out as the cause here (both stayed 0x00 throughout normal music playback in testing) before the real cause (§4.4) was found.
- **`channels`** (core-agnostic `IDebugTarget.GetAudioChannels()`/`DebugAudioChannelInfo`, same "generic shape, core does the reshaping" pattern as `sprites`/`pal`) lists all 8 S-DSP voices: active flag, envelope level rescaled to 0-100 (from the SNES's native 0-2047 range), muted flag, and a free-text detail string (Srcn, pitch, vol, ADSR stage/registers, ended flag, KeyOn count, samples since last KeyOn). `DspVoice.KeyOnCount`/`LastKeyOnSample` are now tracked unconditionally (not just when `DspKeyOnLogging` is on), so this works without needing that console-logging flag enabled first.
- **`mute <index> <on|off>`** (`IDebugTarget.SetChannelMuted`) excludes one voice from the final mix (and from feeding the echo buffer) without pausing its own playback/envelope state - matching real muting semantics (§3.4's "still advance playback" behavior) rather than a separate solo-rendering pipeline. Lets a suspected-broken instrument be isolated (mute every other voice, then `audiodump`) or ruled out (mute just it, confirm the rest of the mix is unaffected). Backed by `SDsp._debugMuteMask`, explicitly not real hardware state (`[SkipInState]`, same reasoning as `_audioBuffer`/`Renderer._sheetPixels`).

---

## 4. `DspVoice` — BRR playback + ADSR/GAIN envelope

### 4.1 Not implemented (documented, not silently wrong)

- **Echo** (EON, FIR filter, echo buffer) — entirely separate subsystem, not present at all.
- **Noise generation** (NON) — voices always play their BRR sample regardless of this bit.
- **Pitch modulation** from the previous voice's output (PMON).
- **Exact period-table phase alignment across voices** (§4.4) — envelope *rates* are correct, exact per-clock *phase* isn't.

**Stale note corrected**: this section previously listed Gaussian interpolation itself as not implemented ("uses nearest-neighbor instead"). That's no longer accurate as of `DspInterpolation.cs`/`DspVoice.GetNextSample`'s current code - real 4-tap Gaussian resampling (`Gauss4Point`, backed by the same 512-entry hardware constant table every reference implementation uses) is implemented and, per §2.9's MesenCE comparison, byte-for-byte identical to MesenCE's own `DspInterpolation.h` table and formula, including its "wrap first 3 terms at 15-bit signed, then clamp full sum" quirk. Left uncorrected in this doc for some time after the code changed - a reminder to re-check "Not implemented" lists against current code before trusting them, not just when a session happens to be looking directly at that file.

### 4.2 Loop blocks keep prediction-filter history

When a BRR block's loop flag is set, `AdvanceSourceSample` jumps `_blockAddr` to `_loopAddr` **without** resetting the `BrrDecoder`'s `_prev1`/`_prev2` filter history — real hardware carries the prediction filter straight into the loop block rather than resetting it, same as it would for any other mid-stream block transition. Whether this matters audibly depends on the loop block's own filter type.

### 4.3 End-of-sample vs. loop

A BRR block's header carries independent end and loop flags (see `BrrDecoder.IsEndBlock`/`IsLoopBlock`, §5). On reaching an end-marked block: loop flag set → jump to `_loopAddr` (§4.2) and keep playing; loop flag clear → voice stops, `Ended` is set (feeding ENDX, §3.2).

### 4.4 Envelope rate gating (`RateDue`)

The 32-entry period table (`PeriodTable`, verified against the SNESdev DSP_envelopes page) is denominated directly in **audio samples per envelope step** - the same table appears verbatim (2048, 1536, ..., 1) in essentially every reference S-DSP implementation (bsnes, snes9x, Mesen2), always used as a sample-count period with no further conversion; index 0 ("Infinite") means that stage never advances on its own.

**Fixed bug: `RateDue` divided the table's value by 32 before using it**, on the mistaken assumption the table was denominated in raw S-SMP clock cycles needing conversion to samples via `SDsp.Tick`'s 32-cycles-per-sample constant - it wasn't; the table's values are already in samples. That extra division made every envelope step (attack ramp, decay, sustain decay) advance up to 32x too fast, and for most rate indices (table value already under 32) the `Math.Max(1, ...)` clamp made it fire on literally every generated sample regardless of the real intended rate. Reported as "you can hear part of the music, but there's missing sounds" (after the separate CPU->SPC700 pacing fix, §2.8, resolved a related "too fast" complaint) - confirmed via the new `channels` debug command (§3.5): a voice sitting in Sustain stage had already decayed to envelope level 0 within ~5900 samples (~0.18s) of KeyOn, when real hardware would still be clearly audible there. This is exactly a "some of the music is missing" symptom rather than total silence - sharp one-shot percussion (little or no meaningful sustain/decay phase) still played close to normally, while any note relying on its sustain to actually ring out collapsed to silence almost immediately. Fixed by using the table's value directly, no division.

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
