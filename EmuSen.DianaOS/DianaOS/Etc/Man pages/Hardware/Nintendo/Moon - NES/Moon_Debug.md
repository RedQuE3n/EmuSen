# Moon (NES) — `IDebugTarget`

Covers `Cores/Nintendo/Moon - NES/Debug/MoonDebugTarget.cs` and the disassembler at `Cpu/Disassembler/Nes6502Disassembler.cs`.

**This is the second implementation `IDebugTarget` has ever had**, and the thing `EmuSen_Core_Gameplan.md` §7 calls the point of the whole architecture. §6 records what that actually proved.

---

## 1. What it took

`MoonDebugTarget` is one file. It implements every required member of `IDebugTarget`, plus `Coverage`, `Labels`, `InterruptVectors`, `DebugCpus` and `ResolvePhysical` from the defaulted set, and implements `Moon.Memory.IWriteObserver` so the watch mechanism has something to feed on.

**No change to `IDebugTarget` was needed**, and none to `WatchRegistry`, `BreakpointRegistry`, `FrameLogRegistry`, `CheatRegistry`, `CoverageRegistry`, `LabelRegistry`, `PollingProvider` or `DianaOSInterpreter`. The registries are owned by `MoonCore` rather than the target, so a label set or a cheat list outlives any one prompt session — the same lifetime Venus gives them.

---

## 2. Memory spaces

One `MoonDebugMemorySpace` class routes every space through `MoonCore.ReadSpace`/`WriteSpace` (`Moon_Core.md` §4), rather than the three shapes Venus needed (`ByteArrayDebugMemorySpace`, `BusDebugMemorySpace`, `DelegateDebugMemorySpace`). The NES needs only one because the core already funnels its spaces through one pair of methods for the cheat engine.

`PRGROM` is published read-only: a stray `mem PRGROM ... w` must not corrupt the loaded image. `CHR` is writable only on a CHR-RAM board, matching what the hardware would accept.

`CPUBUS` is the only space declaring `HasSideEffects`, for the `$2002`/`$2007` reasons in `Moon_Core.md` §4.

---

## 3. Registers

- **CPU** — `A`/`X`/`Y`/`S`/`PC`/`P`, the six real flags broken out as 1-bit values, and the cycle count.
- **Video** — `PPUCTRL`/`PPUMASK`/`PPUSTATUS`/`OAMADDR`, the loopy `v`/`t`/`x`/`w` (which is what a scroll bug is actually diagnosed from), the current scanline, and the three status flags.
- **APU** — all twenty `$4000-$4013` bytes verbatim, plus the frame counter and its IRQ flag, and then the cartridge board's own registers (§3.1).
- **Coprocessor** — empty. The NES has no cartridge coprocessor in any board implemented here.

All published through `PollingProvider`, refreshed by the host calling `RefreshProviders()` once per frame, so `regs`/`coretop` never touch live core state from a console thread.

Until 2026-08-04 this core refreshed itself instead: the constructor set `MoonCore.FrameRefresh = Refresh` and `EndFrame` invoked it, while `IDebugTarget.RefreshProviders()` was a defaulted no-op the NES never implemented. Both cores were then "correct" by different mechanisms — SNES pulled from the host loop, NES pushed from its own scheduler — and nothing checked that a core had picked one.

The push side could not express the case that broke it. `EndFrame` only fires when a frame *completes*, so it says "a frame ran" and nothing else. A rewind, a halt at a breakpoint and a single-step all change machine state without completing a frame, which is exactly why both hosts call `RefreshProviders()` in their rewind branch and why `EmuSen.Pharaoh` sets `OnHalted` to it. On the NES those calls landed on the empty default, so a rewinding or halted NES kept serving the previous frame's snapshot — `coretop` and `regs` showed state the machine had already left. The host knows about frame boundaries *and* about rewind, halt and step; the core's scheduler only knows the first. Pull is therefore strictly more expressive, and it is now the only rule: `RefreshProviders()` is a required interface member, so a core that forgets it fails to compile rather than going quietly stale.

`RefreshProviders()` also refreshes the coprocessor register provider, which this core hard-wires to empty. It costs nothing — `Array.Empty` hands back the same instance every time — and refreshing it keeps "refresh everything I expose" literally true, which is the divergence that caused this in the first place.

### 3.1 Board registers

`IMapper.DebugState` is a defaulted, optional list of `(Name, Value, Bits)`. A board with nothing worth showing reports none and costs a `Board:<name>` header line; MMC3 reports its bank select, both mode bits, all eight bank registers, the whole IRQ block, and a cumulative `IrqsFired` counter.

That counter is diagnostic rather than hardware state, and it earned its place immediately: it is what established that MMC3's IRQ was firing 889 times over 900 frames while Super Mario Bros. 3's status-bar split was still landing in the wrong place, which ruled out "the IRQ never fires" and pointed at counter *phase* instead — the pre-render-line A12 clock (`Moon_Memory.md` §4.6a).

### 3.1a Cheats, and why `ApplyCheats` stays defaulted

`IDebugTarget.ApplyCheats()` keeps its defaulted no-op body, and that is deliberate rather than the same oversight left half-fixed. This core applies cheats from `EndFrame` via `Cheats.ApplyAll`, one frame at a time, with no host involvement — so there is genuinely nothing for a host-driven call to do. The distinction is that cheat application is a *write* the core performs on its own schedule, whereas a provider refresh is a *read* whose correct moment only the host knows. Mistress's Apply Cheats button therefore takes effect on the NES at the next frame boundary rather than immediately, which is invisible at 60fps and is not the staleness bug in §3.

### 3.2 Hardware load

Two bars, **`CPU+APU`** and **`PPU`**, each a percentage of one native frame's wall-clock budget (`1000 / FrameRateHz`, ≈16.639ms at 60.0985Hz) and clamped to 100 so a frame that ran behind — a debug prompt having just eaten real time — cannot report 300% and look like a rendering bug in a bar only meant to reach "full".

Until 2026-08-04 `HardwareLoad` was hard-wired to `Array.Empty<DebugLoadInfo>()`, so `coretop` skipped the section entirely and the NES had no load bars at all. The empty list is the documented "this core models no timing breakdown" signal, and `coretop` correctly draws nothing rather than fake zeroes — but the NES had no breakdown only because none had been written, not because the concept did not apply.

**What these measure is emulator cost, not guest hardware utilization.** They are wall-clock timings of this emulator's own work, exactly like the SNES's. That distinction is not modeled in `DebugLoadInfo` and a reader will conflate the two; emulator cost is the more useful of the pair and the only one every core can produce. It doubles as the per-subsystem profile the low-end-laptop optimization effort needs on this core.

**Why `CPU+APU` is one bar rather than two.** `Apu.Step(cycles)` is called from inside `RunCpuUntilBudgetSpent`'s instruction loop, clocked from the real CPU cycles each instruction consumed — which is what makes its timers correct (§`Moon_APU.md`). Timing the two separately would mean a `Stopwatch.GetTimestamp()` pair per *instruction* rather than per scanline, which would cost more than the thing being measured. The SNES groups `CPU+SPC700` for the same reason.

The instrumentation sits in `RunFrame`'s scheduler loop: one timestamp before the CPU phase, one between it and `_schedule.RunUntil(deadline)` (which is where `EndScanline` drives the PPU), one after. Three per scanline. Accumulators live in `MoonCore.Schedule.cs` and are converted and reset in `EndFrame`, not per scanline — so a breakpoint halt, which returns from `RunFrame` mid-loop without closing the PPU phase, leaves the partial frame accumulating into the resumed one rather than reporting a frame's cost twice.

`MoonDebugTarget` takes an optional `frameTimings` delegate, defaulting to reading the core. It exists only so a test can assert exact bar percentages against injected values; nothing in the product passes it. `SnesDebugTarget` has the same parameter for a different reason — it is constructed from loose `Cpu`/`MemoryBus`/`Renderer` components and cannot reach `VenusCore` at all.

One deliberate difference from the SNES: this core normalizes against its real `FrameRateHz`, where `SnesDebugTarget` hardcodes a 60fps budget. The real-rate version is what `EmuSen_Cauldron.md` §4.5 documents the contract as; the SNES's hardcode is a ~0.15% error nobody will see on a bar 30 characters wide, and was left alone rather than changed as a side effect of this work.

---

## 4. Decoders

`TilemapEntryStride` is **1**. `IDebugTarget`'s own comment predicted this: an NES nametable entry is one byte, with the palette coming from a separate, coarser attribute byte covering a 2×2-tile block — not the SNES's single packed 16-bit word. `DecodeTilemapEntry` looks up the attribute byte itself and renders `$A3 p2`; addresses at or past offset `0x3C0` in a page are the attribute table and render as `attr $1B`.

`DecodeTilePixels` accepts **only** `bpp == 2` and throws `ArgumentException` otherwise, as the interface asks. An NES tile is 16 bytes: eight rows of the low bitplane, then eight of the high.

`RenderTileSheet` renders CHR 16 tiles wide in grayscale, because a pattern-table tile has no palette until a nametable entry picks one. `RenderPaletteSwatch` renders the 32 palette bytes as a 16×2 grid of 16-pixel swatches, resolved through the NTSC table.

---

## 5. The disassembler

`Nes6502Disassembler` is a **second, independent opcode table**, deliberately not shared with the executing one in `Cpu.OpcodeTable.cs` — the same separation Venus keeps, so a transcription error in one does not silently agree with the other. It carries an addressing mode per opcode; lengths and operand text derive from that.

`ClassifyStaticReference` returns a target **only** for absolute and zero-page forms. Indexed, indirect and immediate operands depend on runtime register state, and the interface is explicit that guessing is worse than declining — so `STA $0200,X` and `LDA ($20),Y` both classify as nothing. `JSR` and `JMP abs` are calls; the stores and the read-modify-writes are writes; the loads, compares and ALU reads are reads.

`ResolvePhysical` follows the same principle: it answers for RAM, the register ranges and PRG RAM, and returns null for `$8000+` because which bank is mapped there is live mapper state.

---

## 6. What this proved, and what it did not

**Proved.** The core-agnostic claim holds for a genuinely different console. A 6502 with a 16-bit address space, one-byte nametable entries, 2bpp tiles, a 64-colour fixed palette and no coprocessor plugged into a toolchain built entirely against a 65816 with a 24-bit bus, packed 16-bit tilemap words, 2/4/8bpp tiles, CGRAM and four coprocessors — and the interface did not move. The two places `IDebugTarget`'s comments explicitly speculated about an NES core (the nametable/attribute split, and CHR not being "VRAM in the SNES sense") turned out to be right.

**Not proved.** Coverage of the *optional* surface is thin: `AccessCounters`, `Freezes`, `Expressions`, `CallStack`, `DmaChannels`, `RegisterFlow` and `BreakConditions` are all left at their defaults. Those defaults existing is part of why this was cheap, but a second implementation that declines two thirds of the optional surface tests the required core much harder than it tests the rest.

`HardwareLoad` publishes an empty list — there is no per-subsystem timing breakdown, so `coretop` skips that section rather than showing fake bars. `AudioChannels` reports the five channels with real length-counter state but no levels, because nothing is synthesized (`Moon_APU.md`).

**Untested against the shell.** Everything here is exercised by `EmuSen.WiseMan/Cores/MoonDebugTargetTests.cs` against the interface directly. No DianaOS command has been run against a Moon target, and `EmuSen.Pharaoh`/`EmuSen.Mistress` have not been wired to construct one — they still build `SnesDebugTarget` from a concrete `VenusCore`. That wiring is the obvious next step and is where any remaining assumption that "the core" means Venus will surface.
