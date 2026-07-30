# EmuSen Debugging Tools — Reference

This document covers every debugging tool currently in the project: what it does, where it lives, how to trigger it, and how the pieces fit together. It's organized from "things you press a key for" down to "the underlying reusable toolchain," since that's roughly the order you'd reach for them in.

---

## 1. Console hotkeys (Raylib console build, `Frontend/Program.cs`)

These all live in the same per-frame hotkey block in `Program.cs`, checked once per frame after `renderer.DrawFrame()`.

| Key | Does |
|---|---|
| **F1** | Full CPU + PPU state snapshot. Formatted to be directly comparable to MesenCE's Status panel. |
| **F2** | Dumps every active OAM sprite's X/Y/tile/attribute bytes — ground truth for sprite investigations. |
| **F3** | Saves the current frame as a PNG (`Logs/screenshot_frame<N>.png`). |
| **F4** | Opens the interactive debug command prompt (see §3). |
| **F5** | Save state. |
| **F9** | Load state. |
| **P** | Starts a 300-frame bounded scroll-write trace (`AllScrollWriteLogging`), tagged `[BG SCROLL]`. |
| **O** *(console debug view only, `Renderer.cs`)* | Legacy combined dump: OAM + a BG1 "black tile" diagnostic + backdrop compositing math. Left over from an early investigation; still assumes 8x8 tiles internally (not tile16-aware), so treat its BG1 output with that caveat if you ever reach for it again. |

**F1 and F4 both go through the shared debug toolchain** (§3) rather than having their own separate logic — F1 is just `debugTarget.GetSummaryText()` printed once; F4 is the interactive version of the same underlying data, plus more. F2 is intentionally *not* routed through the toolchain — see §3.4 for why.

---

## 2. `DebugSettings.cs` — every logging toggle

One central static class (`Settings/DebugSettings.cs`), grouped by which file each flag affects. Flip these instead of hunting through individual files. All of them are **on/off switches for logging that already runs continuously** — none of them change emulation behavior, only whether/what gets printed.

| Flag | Default | What it logs |
|---|---|---|
| `CpuVerboseLogging` | off | Every executed instruction (PC, opcode, name, target address). Very high volume. |
| `CpuTraceCountdown` | 0 | Pairs with the flag above — set to a positive number to auto-stop the trace after that many instructions instead of running unbounded. |
| `HvIrqEnabled` | **on** | Master switch for H/V-IRQ support. A real, working feature (fixed the title-screen lockup) — not a "known broken" toggle, just a convenient full disable if a future regression needs isolating. |
| `DmaVerboseLogging` | **on** | Every general DMA transfer: channel, direction, source, destination, size. Decodes the destination VRAM address specifically for $2118/$2119 transfers. |
| `DmaSourceAddrLogging` | **on** | Logs which instruction (PC) wrote each DMA channel's source address — added for the Yoshi graphics investigation. Now uses the *corrected* pre-execution PC (see §5.3 — this used to have a real bug). |
| `WindowHdmaLogging` | off | HDMA writes/fetches for the window-position registers on channel 7 specifically. |
| `Spc700VerboseLogging` | off | Every executed SPC700 (audio CPU) instruction. |
| `CgWriteLogging` | off | Every CGRAM color write. |
| `Bg3ScrollWriteLogging` | off | Writes to BG3's scroll registers specifically. |
| `CameraRamLogging` | **on** | Writes to the zero-page RAM bytes the NMI routine stages scroll values in, right before flushing them to the PPU. Only logs on actual value change. |
| `RenderReadLogging` | **on** | What BG2's scroll values look like at the moment the renderer reads them for scanline 0 — compare against the last logged write to catch render-vs-write timing bugs. |
| `AllScrollWriteLogging` | **on** | Every write to all four backgrounds' scroll registers, with the shared latch value used in each calculation. |
| `HvIrqChangeLogging` | off | $4200/$4207-$420A writes, only when they actually change the H/V-IRQ enable bits or HTIME/VTIME. |
| `MathUnitLogging` | **on** | Every hardware multiply/divide operation, with operands and result. |
| `BgModeChangeLogging` | **on** | $2105 (BGMODE) writes, only on actual change — decoded into mode / BG3-priority / per-BG tile-size bits. |
| `MosaicWriteLogging` | **on** | Every $2106 (MOSAIC) write with the scanline it landed on — tells you whether a mid-frame write ever actually happens (the only case the mosaic starting-scanline latch changes behavior). |
| `WindowingEnabled` | **on** | Master switch for window masking. Re-enabled after being off during one earlier investigation — see the in-code comment if you're ever tempted to flip it off again; the masking logic itself is independently verified correct. |
| `CoinTileDumpLogging` | **on** | ASCII-decodes the tile at VRAM $C000 every `StatusEveryNFrames` frames, alongside `[STATUS]` — part of the coin-rendering investigation. |

Most of these default to **on** and are low-volume (change-triggered, not per-instruction/per-pixel), so it's generally safe to leave a normal play session running with all of them active and just grep the resulting log afterward for whatever tag you care about.

---

## 3. The reusable debug toolchain

This is the newer, generalized layer — built specifically so it isn't SNES-only, and so the same code can eventually back a GUI debug window, not just console printouts.

### 3.1 `IDebugTarget` (`Debug/IDebugTarget.cs`)

The core-agnostic contract. Any emulated console implements this to plug into the rest of the toolchain. Deliberately modeled around generic concepts, not SNES-specific ones:

- **`GetMemorySpaces()`** — a list of named, sized, byte-addressable regions (`IDebugMemorySpace`: `Name`, `Size`, `IsWritable`, `Read(addr)`, `Write(addr, value)`). A generic memory viewer just lists whatever a target reports — SNES reports `CPU Bus / WRAM / VRAM / CGRAM / OAM / SRAM`; an eventual NES target would report its own different set through the exact same shape.
- **`GetCpuRegisters()` / `GetVideoRegisters()`** — lists of `DebugRegisterValue` (name, value, bit-width for display formatting), not fixed struct fields — because different CPUs have wildly different register sets.
- **`GetSprites()`** — `DebugSpriteInfo` records (index, x, y, width, height, tile, palette, priority, flip flags) — generic enough to cover very different sprite hardware.
- **`GetPalettes()`** — `DebugPaletteInfo` records, colors *already converted* to display-ready RGB, so nothing downstream needs to know a console's native color format (SNES BGR555, etc.).
- **`GetSummaryText()`** — a free-text escape hatch for whatever isn't (yet) modeled as structured data above.

**Deliberately not included yet:** breakpoints, single-stepping, disassembly. The execution loop can't pause mid-frame today (it runs a whole frame at a time), and there's no disassembler (only an opcode *executor*) — both are real, separate future pieces of work, expected to be *additive* to this interface rather than a rework of it.

### 3.2 `SnesDebugTarget` (`Cores/Snes/Debug/SnesDebugTarget.cs`)

The SNES implementation. Almost entirely a *reshaping* of already-existing, already-verified logic (the same OAM size/high-table decoding `DumpActiveOam` always used, the same register fields `StateDump` already read) into the structured shapes `IDebugTarget` asks for — not new emulation logic.

Two small helper classes back the memory spaces:
- `ByteArrayDebugMemorySpace` — wraps a `byte[]` directly (WRAM, VRAM, CGRAM, OAM).
- `BusDebugMemorySpace` — routes through `MemoryBus.Read8`/`Write8` at a fixed bank offset (used for the raw CPU Bus space, and for SRAM, which is more naturally viewed at its mapped CPU address `$70:0000`).

### 3.3 `DebugCommandProcessor` (`Debug/DebugCommandProcessor.cs`)

A small, composable command layer over `IDebugTarget` — modeled on Unix toolchain conventions (`ls`/`xxd`/`objdump`: small single-purpose commands) rather than one monolithic dump. Takes a line of text, returns a line of text — it doesn't know or care whether that text came from a console prompt, a future headless CLI, or eventually a GUI debug window's command box.

| Command | Usage |
|---|---|
| `help` | List commands |
| `spaces` | List memory spaces (name, size, writable) |
| `mem <space> <addr> [<len>]` | xxd-style hexdump, default length 16 |
| `write <space> <addr> <value>` | Write one byte, if the space is writable |
| `regs` | CPU + video registers |
| `sprites` | Active sprite/OBJ table |
| `pal [<index>]` | One palette, or all 16 if omitted |
| `summary` | Free-text fallback (delegates to `StateDump.DumpAll`) |

Addresses/values accept `0x`, `$`, or bare hex.

**Not interactive in the pause-emulation sense** — same reason as §3.1: each call to `Execute()` is one-shot, runs against current state, returns immediately. The console's F4 prompt just calls this in a loop.

### 3.4 What's deliberately *not* routed through this yet

- **F2's sprite dump** still calls `renderer.DumpActiveOam` directly rather than the new `sprites` command. That method has a second responsibility — it also returns rects consumed by the "O" key's overlay drawing — that `SnesDebugTarget.GetSprites()` intentionally doesn't take on, to keep the new toolchain's data-producing role separate from the renderer's overlay-geometry role. The `sprites` command (via F4) is the generalized, going-forward equivalent of just F2's printed output.
- **`StateDump`, `Debug/DebugTools.cs`, `Renderer.Debug.cs`** (below) are all still called directly in a few places rather than fully absorbed into the new layer — they're lower-level building blocks the new layer sometimes wraps (`GetSummaryText()` → `StateDump.DumpAll`) rather than things it replaced outright.

---

## 4. Underlying helper libraries (pre-date the toolchain above)

### `Cores/Snes/Debug/StateDump.cs`
On-demand CPU+PPU snapshot formatter. Returns formatted strings (doesn't print directly) — `DumpCpuState`, `DumpPpuState`, `DumpAll`. Deliberately laid out to be directly comparable to MesenCE's own Status panel.

### `Debug/DebugTools.cs`
Generic, byte-array-level helpers — reusable for any console's data since none of it hardcodes SNES specifics beyond parameter naming:
- `HexDump(label, data, address, count)` — plain hexdump formatter.
- `DecodeTileAscii(vram, address, bpp)` — ASCII-art tile decoder, parameterized by bit depth.
- `CgramToRgb(lo, hi, brightness)` / `DumpPalette(cgram, paletteIndex, bpp, brightness)` — SNES color-format helpers.
- `DescribeBits(label, value, bits)` — labeled bit-flag formatter.
- `BoundedTrace` — a start/countdown helper for "trace the next N frames then auto-stop" (used by the P-key scroll trace).
- `ChangeTracker<T>` — "did this value change since last time" helper, used throughout `DebugSettings`-gated change-only logging.

### `Cores/Snes/Ppu/Renderer/Renderer.Debug.cs`
- `RenderVramSheet` — renders the full VRAM tile sheet to an internal texture (feeds the console debug view's VRAM panel).
- `DumpActiveOam` — see §1/§3.4.
- `DumpBlackBg1Tiles` — an old, narrowly-scoped diagnostic from a "black squares" investigation; still wired to the O key.

---

## 5. Save states

**F5**/**F9** in the console build; Save/Load State menu items in the Avalonia frontend. Backed by `Common/StateSerializer.cs` — a reflective binary serializer that walks fields (not properties), skipping anything marked `[SkipInState]` (dispatch tables, back-references, and the new `DebugCpu` field added for the DMA trace work).

**Known limitation:** no version header — a save state can break across builds if the underlying fields change shape. Not something to fix casually; flagged so it isn't a surprise.

---

## 6. A worked example: how these fit together in practice

This is roughly how the toolchain got used across the coin/Yoshi rendering investigations this session, as a template for future ones:

1. **Form a hypothesis** about what's failing (data upload vs. rendering vs. game logic) using SMWCentral/wiki research on how the feature actually works on real hardware.
2. **Turn on the relevant `DebugSettings` flags** (usually already on by default) and play through the scenario.
3. **Grep the log** for the specific tag that matters (`[DMA]`, `[OAM DUMP]`, `[DMA-SRC]`, etc.) rather than reading the whole thing.
4. **If the log doesn't have enough detail, add a narrow, purpose-built trace** rather than turning on something broad — e.g. the palette-byte addition to `DumpActiveOam`, or the instruction-bytes addition to the DMA source-address trace. Small, targeted, temporary-if-needed additions beat leaving on something noisy forever.
5. **Watch out for the tool itself being wrong**, not just the game — the DMA source-address PC trace had a real bug (reading the live, already-advanced PC instead of the pre-execution one), caught by noticing the "same PC, many different results" pattern didn't make sense. Debug tooling is still code; it gets debugged too.

---

## 7. Roadmap (things this doc deliberately doesn't cover because they don't exist yet)

- **Breakpoints / single-step / pause-resume** — needs real execution-loop support for pausing mid-frame, which doesn't exist today.
- **A disassembler** — there's an opcode *executor* (the instruction table drives execution) but nothing that turns arbitrary bytes back into mnemonics for display.
- **The Avalonia GUI debug window** — the actual Mesen-style multi-pane debugger (register panels, hex viewer, disassembly view, sprite/palette viewers, event log), built against `IDebugTarget` once the above exist. Everything in §3 was built with this as the eventual consumer, but it isn't built yet.
- **An NES (or other console) `IDebugTarget` implementation** — the interface was designed generically for this from the start, but no second implementation exists yet to prove it out.
