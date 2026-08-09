# Mercury — where this stands, and what comes next

*Pinned 2026-08-04. **All four phases landed 2026-08-08**, and on 2026-08-09 the core ran commercial cartridges for the first time (`Mercury_RealCartridges.md`). §3 is a list of conditions rather than phases. This is the "what should I work on next" doc for Mercury, the same role `EmuSen_Core_Gameplan.md` plays for Venus. It stays a living document: what belongs in §3 from here is whatever a real cartridge turns out to need, not a plan written in advance.*

---

## 1. What is built and verified

All of it is covered by `EmuSen.WiseMan` — 226 tests as of 2026-08-09, across `MercuryCpuTests`, `MercuryCartridgeTests`, `MercuryCoreTests`, `MercuryPpuTests`, `MercuryCgbTests`, `MercuryDebugTargetTests`, `MercuryApuTests`, `MercurySerialTests`, `MercuryCycleTimingTests` and `MercuryAccessBlockingTests`, run headless against `SyntheticGbRom`; plus `MercuryCommercialRomTests` and `MercuryHardwareTestRomTests` against whatever is in the gitignored `TestRoms/`. No ROM is committed, and the suite passes with none present.

| Piece | State |
|---|---|
| **SM83 instruction set** | Complete. Unprefixed and `$CB`, including the illegal-opcode set (which throws rather than acting as NOP). |
| **Flags** | All four, with the rules that differ from the 8080/Z80 people assume — `RLCA` clearing Z where `CB RLC` sets it, `INC` preserving carry, `BIT` leaving carry alone, `ADD SP,e8` carrying off the unsigned low byte, `DAA`. |
| **Interrupts** | Five sources, priority order, vectoring, the `EI` one-instruction delay, `RETI`'s immediate enable, HALT waking regardless of IME, and the HALT bug. |
| **Timer** | DIV as the top half of the real 16-bit counter; TIMA as a falling-edge detector on a selected bit, with the 4-cycle reload window. |
| **Bus** | Cycle-granular: every memory access is a machine cycle that runs the rest of the machine first. `Mercury_Cpu.md` §3. |
| **Serial** | Output only — a sink that captures what a ROM clocks out, which is how the hardware test corpus reports. `Mercury_Memory.md` §10. |
| **Joypad** | The two-nibble matrix, pressed-reads-0, both-halves-selected ANDing. |
| **OAM DMA** | 160 machine cycles, one byte each, with OAM closed to the CPU throughout. `Mercury_Memory.md` §6. |
| **PPU** | The mode machine on a per-cycle clock, LY/LYC, STAT as one level-triggered line, background, window with its own line counter, sprites with both DMG orderings, and VRAM/OAM access blocking. Renderer is per scanline. `Mercury_Ppu.md`. |
| **Colour** | VRAM and WRAM banking, both palette ports, map and sprite attributes, CGB sprite priority, HDMA in both modes, double speed. `Mercury_Cgb.md`. |
| **APU** | Four channels, the frame sequencer off a DIV bit, stereo panning, the DC-removing high-pass. `Mercury_Apu.md`. |
| **Cartridge** | Header, title (both lengths), colour flag, checksum, ROM/RAM sizing. |
| **Boards** | No-MBC, MBC1 (both modes), MBC2, MBC3 with a latched RTC, MBC5. |
| **Core** | `ICore` surface, frame budget, breakpoints, coverage, cheats, named debug spaces, save states. |
| **Debug** | `MercuryDebugTarget`, the SM83 disassembler, Game Genie and GameShark, and registration in `CoreCatalog`/`CoreFactory`. `Mercury_Debug.md`. |

## 2. Decisions already made — do not relitigate these

- **One core for GB and GBC.** DMG first, colour additive. Reasoning in `Mercury_Core.md` §1. `Cartridge.Cgb` already reads `$0143`.
- **Registered in `CoreCatalog`/`CoreFactory` as of Phase B**, claiming `.gb` and `.gbc`. The condition this waited on was real and is now met: `CoreFactory.Load` builds a `CoreBundle` whose `IDebugTarget` is non-nullable, and one now exists.
- **Illegal opcodes throw.** A game reaching one means something upstream already went wrong; treating them as NOPs hides the real bug.
- **`.gb` and `.gbc` both reach the same core**, and which console it becomes is the header's answer rather than the extension's — a `.gb` file with `$0143 = $C0` runs in colour. The prediction that the extension would arrive with Phase B rather than Phase D held.
- **Colour mode is decided by the header, once, at load.** Both `$80` and `$C0` select it, matching the real console. No runtime toggle — `Mercury_Cgb.md` §1.

## 3. Phases

**None left.** A, B, C and D are all in §1.

What comes next is not another phase, because the remaining gaps are not a plan — they are conditions. Each of the four subsystem pages ends with its own "what is not modelled" list, and every entry there names the evidence that would justify doing the work.

### 3.1 A retired prediction: the audio differential was never blocked

*Written 2026-08-09. This section previously said an audio differential was blocked
because "the probe's libretro backend refuses `--wav`". That was true as an
observation and wrong as a diagnosis, and it is recorded here rather than deleted
because the reasoning error is the reusable part.*

The libretro backend does refuse `--wav`, at `libretro.rs`'s `load`. But a libretro
core does not write audio files at all — it *pushes* samples to the frontend through
`retro_set_audio_sample_batch`, which the probe registers and then discards:

```rust
unsafe extern "C" fn audio_batch(_data: *const i16, frames: usize) -> usize { frames }
```

Returning `frames` claims every sample was consumed. The data was arriving the whole
time. The warning was a stub someone wrote while porting, not a statement about the
backend's capability, and it was read as a hardware fact for a day.

**The general lesson:** a `[WARN] not supported` printed by our own code is evidence
about our code, not about the thing it names. The probe's own diagnostics are not an
independent source. Confirming this cost one `grep` for the callback body.

### 3.2 The oracle problem, and why serial answers it

Behind all three remaining conditions sits one question: *diff against what?*

The reflex is another emulator, and for the SNES that reflex is right — `Reference/`
exists, the GSU diffs to zero, and `known-differences.db` records where Mesen and
Venus legitimately disagree. For the Game Boy it is a dead end in three ways. Mesen
has no Game Boy core. A libretro core's audio can be captured (§3.1) but two
emulators mixing at different rates and phases never match sample-for-sample, so the
comparison needs a tolerance, and a tolerance needs a reason. And most of all,
agreement with gambatte is not correctness — it is agreement.

**The Game Boy has a better oracle than any emulator: its own test corpus.** blargg's
and mooneye's hardware tests were written against real DMG and CGB silicon with a
logic analyser, and they report their verdict *through the serial port* — write a
character to `SB` (`$FF01`), write `$81` to `SC` (`$FF02`), and hardware clocks it
out. A headless harness that captures those bytes reads `Passed` or `Failed #3` as
plain text. No display, no reference emulator, no screenshot for a human to judge.

Mercury has no `$FF01`/`$FF02` at all — `grep -rn "FF01\|FF02"` over the core returns
nothing, and §4 has said "No serial link. Nothing needs it yet." since Phase A. That
is the cheapest unmet prerequisite in the whole core, and it is what §4's entry was
waiting for without knowing it.

This is what makes the work **reference-agnostic** in the sense the probe already
means it (`EmuSen_Debugging_Tools_Reference_v5.md` §3): the oracle names no emulator.
It is an assertion about a DMG.

### 3.3 The order, and why it is that order

**A — the serial sink.** A write-only `$FF01`/`$FF02` and a capture buffer. No link
cable, no partner, no transfer timing: an unconnected Game Boy sees all-ones on the
wire, which is exactly what these ROMs assume. Roughly twenty lines, and it hands B
and C their pass/fail oracle.

**C — the cycle-granular bus.** Second, because A's `mem_timing` is what proves it
landed. The seam is already built and simply unused: `ICpuBus.Tick(int)` is
documented as *"runs the rest of the machine forward while the CPU is mid-instruction"*
and `grep -rn "_bus.Tick" Cpu/` returns nothing — the CPU never calls it, and
`MercuryCore` ticks afterwards instead. The surface is 26 accesses, 5 in `Cpu.cs`
and 21 in `Cpu.Opcodes.cs`. Route each through a `ReadCycle`/`WriteCycle` that ticks
four T-cycles, add the internal cycles that are not memory accesses, and stop
`MercuryCore` double-ticking. Then §4's three linked simplifications — access
blocking, OAM DMA's 160 cycles, HDMA's stall — become writable, and should land
together as §4 has always said.

**B — probe-side audio capture.** Independent of both, and deliberately last because
its claim is the weakest. Implement the discarded callbacks, ask
`retro_get_system_av_info` for the true sample rate instead of assuming one, and
write the RIFF header in the probe. This makes the probe *more* agnostic, not less:
`audio.rs` currently says capture is "always a WAV, because that is the only thing
the reference emulator can write", and once the probe writes the container itself
that dependency is gone — the same move the Rust port made for everything else.

The comparison it feeds must be a **feature stream, not samples**: windowed RMS,
onset times, per-channel activity, compared with a tolerance through the same
dictionary pattern `compare.py` uses. A sample-exact diff would report a wall of
noise and teach nothing.

### 3.4 What this does not cover

Stated plainly, because the fix landing does not make these true:

- **A serial sink is not a serial link.** Nothing clocks bits, no transfer takes
  512 T-cycles, and two Mercurys cannot be connected. A game that waits on an
  external clock still hangs. Only the test-ROM output path is served.
- **The test corpus does not cover the APU's *mixer*.** blargg's sound tests assert
  register and length-counter behaviour, not that the DAC output sounds right.
  §4's aliasing entry is untouched by all of this.
- **A CGB-exclusive cartridge is still not a commercial one.** `cgb_sound` is a
  genuine `$C0` ROM and closes the *code-path* gap; what a shipped colour game does
  to the core remains untested.
- **`oam_bug` will fail until access blocking exists**, and that is the intended
  result — a numbered failing assertion in place of a suspicion.

## 4. Known simplifications, and when each will matter

*Five entries left this list on 2026-08-09 — instruction-granular timing, instant
OAM DMA, absent access blocking, the unmodelled HDMA stall, and "no serial link".
They are not deleted; each is now a section in its subsystem page recording what the
old choice bought and what retired it.*

- **Internal cycles settle at the end of an instruction** rather than at their true
  position within it. Memory accesses are placed correctly, which is what blocking
  and `mem_timing` depend on — `Mercury_Cpu.md` §3.1.
- **The access ordering inside a machine cycle is reasoned, not measured.**
  `ReadCycle` ticks then reads; `mem_timing` is what would settle it —
  `Mercury_Cpu.md` §3.2.
- **The CPU is not locked out of ROM and WRAM during an OAM DMA**, only out of OAM.
  The remaining piece of the cycle-granular cluster, and the one with a real risk of
  looking like an emulator fault if done without the test that asserts it —
  `Mercury_Memory.md` §6.1.
- **A general-purpose HDMA copies at once and charges the CPU afterwards** rather
  than interleaving the copy with the clock. The observable half is modelled and the
  unobservable half is not — `Mercury_Cgb.md` §4.1.
- **The renderer is per scanline** under a per-cycle clock. `Mercury_Ppu.md` §1.
- **Mode 3 does not lengthen for sprites**, so the window in which blocking applies
  is slightly short on a busy line — `Mercury_Ppu.md` §7.2.
- **The serial port is a sink, not a link.** Output only; nothing clocks bits and two
  Mercurys cannot be connected — `Mercury_Memory.md` §10.2.
- **No boot ROM.** Mercury starts at `$0100` with the post-boot register state
  hardcoded, which is also why the header checksum is computed but never enforced.
- **`STOP` performs the speed switch on a CGB** instantaneously rather than over the
  ~2050 cycles hardware spends on it — `Mercury_Cgb.md` §5.
- **No call stack, expression context, access counters or freezes** on the debug
  target. Each is a seam the core does not have — `Mercury_Debug.md` §6.
- **The APU mixes at the output rate**, so a channel period above Nyquist aliases
  rather than being filtered — `Mercury_Apu.md` §7. This is now a live suspect rather
  than a theoretical one: it is one of the candidate explanations for the 7 dB
  per-window disagreement with gambatte, and `dmg_sound` is what would separate them
  (`Mercury_HardwareTests.md` §6.3).
- **Seven commercial cartridges run** and have not been *played* — everything so far
  is title screens and attract demos. `Mercury_RealCartridges.md` §6.
- **Mercury boots about four frames ahead of gambatte.** Ruled out as the explanation
  for the audio difference, but still true and still something a frame-indexed
  differential has to align for — `Mercury_HardwareTests.md` §6.2.
- **No SGB, and no CGB-exclusive commercial cartridge.** `cgb_sound` closes the
  code-path half of the second one; the shelf has no `$C0` game and the DX proto is
  not one — `Mercury_RealCartridges.md` §6.1.

## 5. Resuming

```
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~Mercury"
```

Read `Mercury_Cpu.md` before touching the CPU — §1 lists the ways the SM83 is not the chip people assume it is, and most of them are one-line mistakes that pass casual review.

Mercury is now in `CoreCatalog`, so a `.gb` or `.gbc` file also reaches it through either frontend and through DianaOS. `emusen` and `disasm cpu` work against it the same way they do against Moon.
