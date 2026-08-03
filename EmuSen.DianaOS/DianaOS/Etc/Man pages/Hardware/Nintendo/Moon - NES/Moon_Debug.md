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
- **APU** — all twenty `$4000-$4013` bytes verbatim, plus the frame counter and its IRQ flag.
- **Coprocessor** — empty. The NES has no cartridge coprocessor in any board implemented here.

All published through `PollingProvider`, refreshed once per frame from `MoonCore.EndFrame` via the `FrameRefresh` hook, so `regs`/`coretop` never touch live core state from a console thread.

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
