# EmuSen Debugging Tools — Reference

This document covers every debugging tool currently in the project: what it does, where it lives, how to trigger it, and how the pieces fit together. It's organized from "things you press a key for" down to "the underlying reusable toolchain," since that's roughly the order you'd reach for them in.

*(Living document — updated as the toolchain grows. This revision adds the watchpoint mechanism, the generalized `tile` command, the shared `FrameCount`/screenshot timestamping, and a full 65816 disassembler — retiring `CoinTileDumpLogging` along the way. Also reflects a later decoupling pass: `WatchRegistry` moved from being owned by `MemoryBus` to being owned by `SnesDebugTarget`, communicating through a new, debug-agnostic `IWriteObserver` hook instead of `MemoryBus` holding a concrete `WatchRegistry`/`Cpu` reference directly.)*

---

## 1. Console hotkeys (Raylib console build, `Frontend/Program.cs`)

These all live in the same per-frame hotkey block in `Program.cs`, checked once per frame after `renderer.DrawFrame()`.

| Key | Does |
|---|---|
| **F1** | Full CPU + PPU state snapshot. Formatted to be directly comparable to MesenCE's Status panel. |
| **F2** | Dumps every active OAM sprite's X/Y/tile/attribute bytes — ground truth for sprite investigations. |
| **F3** | Saves the current frame as a PNG (`Logs/screenshot_frame<N>.png`), plus a companion metadata file (`Logs/screenshot_frame<N>.txt`) with the core name, frame number, and wall-clock time — see §3.6. |
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
| `DmaSourceAddrLogging` | **on** | Logs which instruction wrote each DMA channel's source address. **Fixed since first added:** originally read the CPU's live `PC`/`PB`, which had already advanced past the responsible instruction by the time a write's side effect ran (caught mid-investigation when the same PC kept appearing for many different, unrelated results). Now reads `Cpu.LastInstructionPC`/`LastInstructionPB` — captured once per `Step()` before fetch/execute, so it stays accurate for that instruction's entire execution including any writes it triggers. |
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

**Retired:** `CoinTileDumpLogging` (used to ASCII-decode the tile at a hardcoded VRAM address every N frames) and `YoshiWramTraceLogging` (a hardcoded WRAM-range write trace) are both gone — superseded by the general `tile` command and the watchpoint mechanism respectively (§3.3, §3.5). Neither needed its own permanent flag once a general, on-demand equivalent existed; this is the intended direction going forward — a narrow, investigation-specific flag is a sign something belongs in the toolchain instead, not a permanent fixture.

Most remaining flags default to **on** and are low-volume (change-triggered, not per-instruction/per-pixel), so it's generally safe to leave a normal play session running with all of them active and just grep the resulting log afterward for whatever tag matters.

---

## 3. The reusable debug toolchain

This is the newer, generalized layer — built specifically so it isn't SNES-only, and so the same code can eventually back a GUI debug window, not just console printouts. **Standing policy going forward: if something built for one investigation seems logical and reusable, it goes in here rather than staying a one-off `DebugSettings` flag.**

### 3.1 `IDebugTarget` (`Debug/IDebugTarget.cs`)

The core-agnostic contract. Any emulated console implements this to plug into the rest of the toolchain. Deliberately modeled around generic concepts, not SNES-specific ones:

- **`GetMemorySpaces()`** — a list of named, sized, byte-addressable regions (`IDebugMemorySpace`: `Name`, `Size`, `IsWritable`, `Read(addr)`, `Write(addr, value)`). A generic memory viewer just lists whatever a target reports — SNES reports `CpuBus / WRAM / VRAM / CGRAM / OAM / SRAM`; an eventual NES target would report its own different set through the exact same shape.
- **`GetCpuRegisters()` / `GetVideoRegisters()`** — lists of `DebugRegisterValue` (name, value, bit-width for display formatting), not fixed struct fields — because different CPUs have wildly different register sets.
- **`GetSprites()`** — `DebugSpriteInfo` records (index, x, y, width, height, tile, palette, priority, flip flags) — generic enough to cover very different sprite hardware.
- **`GetPalettes()`** — `DebugPaletteInfo` records, colors *already converted* to display-ready RGB, so nothing downstream needs to know a console's native color format (SNES BGR555, etc.).
- **`Watches`** — the watchpoint registry (see §3.5). Exposed directly rather than re-wrapped into more `IDebugTarget` methods, since `WatchRegistry` is already core-agnostic on its own.
- **`FrameCount`** — a monotonic frame counter, owned as real state (not a frontend-local variable) so anything built against the interface can ask "what moment is this" consistently regardless of which core is running. Currently used to timestamp screenshots (§3.6); the same value is available to anything else that wants it.
- **`Disassemble(spaceName, address, count)`** — returns plain `DisassembledInstruction` records (address, raw bytes, mnemonic, formatted operand text). The interface knows nothing about a given CPU's addressing modes or operand-formatting conventions; all of that lives inside each core's implementation (§3.7). A target with no disassembler can legitimately return an empty list.
- **`GetSummaryText()`** — a free-text escape hatch for whatever isn't (yet) modeled as structured data above.

**Deliberately not included yet:** breakpoints and single-stepping. The execution loop can't pause mid-frame today (it runs a whole frame at a time) — that's real, separate future work, expected to be *additive* to this interface rather than a rework of it. Disassembly (previously in this same "not yet" list) now exists — see §3.7.

### 3.2 `SnesDebugTarget` (`Cores/Snes/Debug/SnesDebugTarget.cs`)

The SNES implementation. Almost entirely a *reshaping* of already-existing, already-verified logic (the same OAM size/high-table decoding `DumpActiveOam` always used, the same register fields `StateDump` already read) into the structured shapes `IDebugTarget` asks for — not new emulation logic.

Two small helper classes back the memory spaces:
- `ByteArrayDebugMemorySpace` — wraps a `byte[]` directly (WRAM, VRAM, CGRAM, OAM).
- `BusDebugMemorySpace` — routes through `MemoryBus.Read8`/`Write8` at a fixed bank offset (used for the raw CpuBus space, and for SRAM, which is more naturally viewed at its mapped CPU address `$70:0000`).

`Watches` here just returns `SnesDebugTarget`'s own `WatchRegistry` instance. **This changed since first built:** the registry (and a `Cpu` back-reference, `DebugCpu`) used to live directly on `MemoryBus`, which mixed real emulation state with debug-toolchain plumbing in the same class — a coupling issue caught during a later architecture review. Now `MemoryBus` exposes only a tiny, debug-agnostic `IWriteObserver` hook (`Cores/Snes/Memory/IWriteObserver.cs`) that it calls on every write with no idea what's listening; `SnesDebugTarget` implements that interface, owns the `WatchRegistry` itself, and supplies the PC context from its own already-held `Cpu` reference. `MemoryBus` no longer references `Cpu` or the debug toolchain at all.

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
| `tile <space> <addr> <bpp>` | ASCII-decode one 8x8 tile from any space (bpp 2, 4, or 8) |
| `disasm <space> <addr> [<count>]` | Disassemble `<count>` instructions (default 10) — see §3.7 |
| `watch add <space> <addr> <len>` | Register a watchpoint |
| `watch list` | List active watchpoints with their IDs |
| `watch log <id> [<count>]` | Show a watchpoint's recorded events (default 20) |
| `watch clear <id>` | Clear a watchpoint's stored events (keeps the watch registered) |
| `watch remove <id>` | Remove a watchpoint entirely |
| `summary` | Free-text fallback (delegates to `StateDump.DumpAll`) |

Addresses/values accept `0x`, `$`, or bare hex.

**`tile`** generalizes what used to be `DebugTools.DecodeTileAscii` plus `CoinTileDumpLogging`'s hardcoded call site — works against *any* memory space (not just VRAM) and now supports 8bpp too (relevant since Mode 3/4's 8bpp BG1 and Direct Color exist now, which didn't when the original helper was written).

**Not interactive in the pause-emulation sense** — same reason as §3.1: each call to `Execute()` is one-shot, runs against current state, returns immediately. The console's F4 prompt just calls this in a loop.

### 3.4 What's deliberately *not* routed through this yet

- **F2's sprite dump** still calls `renderer.DumpActiveOam` directly rather than the `sprites` command. That method has a second responsibility — it also returns rects consumed by the "O" key's overlay drawing — that `SnesDebugTarget.GetSprites()` intentionally doesn't take on, to keep the toolchain's data-producing role separate from the renderer's overlay-geometry role. The `sprites` command (via F4) is the generalized, going-forward equivalent of just F2's printed output.
- **`StateDump`, `Debug/DebugTools.cs`, `Renderer.Debug.cs`** (below) are all still called directly in a few places rather than fully absorbed into the new layer — they're lower-level building blocks the new layer sometimes wraps (`GetSummaryText()` → `StateDump.DumpAll`) rather than things it replaced outright.
- **The watch registry currently only hooks WRAM writes.** `MemoryBus`'s `IWriteObserver` hook (§3.5) is only called from the WRAM write path today — `Ppu`'s VRAM/CGRAM/OAM writes and the general CPU-bus/SRAM path don't report to it yet. Built for what the Yoshi investigation needed, with wiring in the other write paths as natural, additive follow-up whenever an investigation needs to watch one of them.

### 3.5 `WatchRegistry` (`Debug/WatchRegistry.cs`) — watchpoints

The generalized version of the recurring "log every write to this address range" pattern this project kept building ad hoc (`CameraRamLogging`, `MosaicWriteLogging`, `DmaSourceAddrLogging`, and originally a one-off Yoshi-specific WRAM trace). Instead of adding a new `DebugSettings` flag and a hand-written range check in the emulation code every time an investigation needs one, register a watch through the toolchain instead.

- **`AddWatch(spaceName, address, length)` → id** — register a watch on a memory-space address range.
- **`RemoveWatch(id)`** — remove one.
- **`GetWatches()`** — list active watches.
- **`RecordWrite(spaceName, address, value, contextFactory)`** — called by whoever owns the registry (`SnesDebugTarget`, via its `IWriteObserver.OnWrite` implementation - see §3.2) in response to a write reported by the core it's watching. Cheap to call even with zero watches registered (a quick scan over however many are active). When a write matches an active watch, it's both **printed live** (`[WATCH #id] ...` — so the existing "play, then grep the console log" workflow keeps working with zero changes) and **stored** in a bounded per-watch buffer (last 500 events), so it's also queryable later via `watch log` without needing a fresh log upload.
- **`GetEvents(id, maxCount)` / `ClearEvents(id)`** — read or clear a watch's stored buffer.

`DebugWatchEvent` carries a sequence number, address, value, and a free-text `Context` string (typically `PC=0x00A358`) — kept as text rather than a structured field since what's useful context varies by core and shouldn't force an interface change every time a new kind becomes relevant.

Core-agnostic on purpose, same as `IDebugTarget` — lives under `Debug/`, not `Cores/Snes/`, since nothing about it is SNES-specific. `SnesDebugTarget` owns the instance and reports writes to it via `MemoryBus`'s `IWriteObserver` hook (`Cores/Snes/Memory/IWriteObserver.cs`) — `MemoryBus` itself has no idea `WatchRegistry` exists. A future NES `IDebugTarget` would own its own instance the same way, wired to its own core's equivalent write-observer hook.

**Example — reproducing the Yoshi WRAM trace on demand** instead of it being a permanent flag: press F4 once after launching, then:
```
watch add WRAM 8000 1800
```
Matching writes for the rest of that session get printed live exactly as before, and can also be pulled on demand later with `watch log <id>`.

### 3.6 Screenshot timestamping (F3, `Frontend/Program.cs`)

F3 saves a PNG the same way it always has, but now also writes a companion `screenshot_frame<N>.txt` next to it, and the `[SCREENSHOT]` console log line states the same information explicitly:

```
Core: SNES
Frame: 1378
Wall-clock: 2026-07-22 07:28:14.203
```

All three (filename, companion file, log line) are sourced from the same `debugTarget.FrameCount`/`CoreName` — so they can't drift out of sync with each other the way a hand-drawn on-screen frame counter and a separately-incremented save filename could. This is what `IDebugTarget.FrameCount` (§3.1) is for: a single, reliable, core-agnostic way to answer "what moment was this" that a screenshot, a log line, and eventually a GUI's own display can all agree on.

**Deliberately not done:** burning the timestamp into the image's pixels (a visual overlay baked into the PNG itself, closer to a security-camera timestamp). That would need Raylib's `LoadImageFromScreen`/`ImageDrawText`/`ExportImage` functions, and given a couple of build-error round trips this session already came from guessing at exact API shapes, the companion-file approach was chosen deliberately as the zero-new-API-surface, guaranteed-to-build option. Worth revisiting if the visual version is still wanted later.

### 3.7 Disassembler (`Cores/Snes/Cpu/Disassembler/Snes65816Disassembler.cs`)

A full 65816 disassembler, wired in via `IDebugTarget.Disassemble` and the `disasm` command:
```
disasm <space> <addr> [<count>]     disassemble <count> instructions (default 10)
```
```
  00A358: 8D 22 43  STA $4322
  00A35B: A9 40 00  LDA #$0040
```

**Deliberately a separate table from the execution opcode table** (`Cpu.OpcodeTable.cs`), not built from it — that table's addressing-mode delegates have real execution side effects (advancing PC, consuming cycles) and have no way to report "how many bytes would this take" without actually running the instruction. `Snes65816Disassembler` is a completely independent, read-only mnemonic + addressing-mode table built specifically for display, living under `Cores/Snes/Cpu/Disassembler/` rather than inside `Cpu.cs` itself.

**Important, deliberately-stated caveat — read this before trusting it for anything beyond casual use.** This project's *execution* opcode table got a dedicated verification pass against oxyron.de before being trusted (see the original project handoff notes — "completed 236→256/256, verified against oxyron.de, caught one cross-reference typo"). This disassembly table has **not** had the equivalent treatment; there's no way to build and run this project from wherever it gets edited to cross-check it the same way. It's built carefully against the standard, universally-documented 65816 opcode matrix, and two entries were spot-checked against real bytes captured during the Yoshi DMA investigation earlier this session (`8D 22 43` → `STA $4322`, `A9 40 00` → `LDA #$0040`, both correct) — but that's meaningfully short of a real verification pass. Treat it as a solid first draft, not a verified reference, until it gets one.

**One documented, deliberate simplification:** the 65816's immediate-mode instructions (`LDA #`, `LDX #`, etc.) have an operand length that depends on the *current* M/X flags in the P register — 1 byte if 8-bit, 2 if 16-bit. `Disassemble` reads the CPU's *current* E/M/X flags once and applies that width to the *entire* requested range. If a `REP`/`SEP` instruction sits inside that range and actually changes M/X, everything disassembled after it in the same call will still use the snapshot's width rather than the correct one. Same category of tradeoff as this project's scanline-granularity rendering elsewhere — documented, not silent.

---

## 4. Underlying helper libraries (pre-date the toolchain above)

### `Cores/Snes/Debug/StateDump.cs`
On-demand CPU+PPU snapshot formatter. Returns formatted strings (doesn't print directly) — `DumpCpuState`, `DumpPpuState`, `DumpAll`. Deliberately laid out to be directly comparable to MesenCE's own Status panel.

### `Debug/DebugTools.cs`
Generic, byte-array-level helpers — reusable for any console's data since none of it hardcodes SNES specifics beyond parameter naming:
- `HexDump(label, data, address, count)` — plain hexdump formatter.
- `DecodeTileAscii(vram, address, bpp)` — ASCII-art tile decoder, parameterized by bit depth. **Note:** the `tile` command (§3.3) is now the more general, going-forward way to do this (works against any memory space, supports 8bpp) — this function itself is unchanged and still callable directly, but new code should probably reach for the command instead.
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

**F5**/**F9** in the console build; Save/Load State menu items in the Avalonia frontend. Backed by `Common/StateSerializer.cs` — a reflective binary serializer that walks fields (not properties), skipping anything marked `[SkipInState]` (dispatch tables, back-references, and debug-only bookkeeping).

**Known limitation:** no version header — a save state can break across builds if the underlying fields change shape. Not something to fix casually; flagged so it isn't a surprise.

---

## 6. A worked example: how these fit together in practice

This is roughly how the toolchain got used across the coin/Yoshi rendering investigations this session, as a template for future ones:

1. **Form a hypothesis** about what's failing (data upload vs. rendering vs. game logic) using SMWCentral/wiki research on how the feature actually works on real hardware.
2. **Turn on the relevant `DebugSettings` flags** (usually already on by default) and play through the scenario.
3. **Grep the log** for the specific tag that matters (`[DMA]`, `[OAM DUMP]`, `[DMA-SRC]`, `[WATCH #id]`, etc.) rather than reading the whole thing.
4. **If the log doesn't have enough detail, reach for the toolchain first** — a new `watch`, a `tile` decode, a `mem` hexdump — before writing a new one-off trace. Only build something new and narrow if the existing commands genuinely can't answer the question, and if you do, ask whether it's generalizable enough to fold into the toolchain rather than staying a one-off (this is now the standing policy, not just a one-time cleanup).
5. **Watch out for the tool itself being wrong**, not just the game — the DMA source-address PC trace had a real bug (reading the live, already-advanced PC instead of the pre-execution one), caught by noticing the "same PC, many different results" pattern didn't make sense. Debug tooling is still code; it gets debugged too.

---

## 7. Roadmap (things this doc deliberately doesn't cover because they don't exist yet)

- **Breakpoints / single-step / pause-resume** — needs real execution-loop support for pausing mid-frame, which doesn't exist today. (The watch registry in §3.5 covers *observing* writes; the disassembler in §3.7 covers *displaying* code; neither pauses anything.)
- **A verification pass on the disassembler** (§3.7) — it exists now, but hasn't had the equivalent scrutiny the execution opcode table got against oxyron.de. Worth a dedicated pass rather than trusting it blind.
- **Watchpoints beyond WRAM** — VRAM/CGRAM/OAM and the general CPU-bus/SRAM write paths don't report to `MemoryBus`'s `IWriteObserver` hook yet (§3.4).
- **The Avalonia GUI debug window** — the actual Mesen-style multi-pane debugger (register panels, hex viewer, disassembly view, sprite/palette viewers, event log, watch panel), built against `IDebugTarget` once the above exist. Everything in §3 was built with this as the eventual consumer, but it isn't built yet.
- **An NES (or other console) `IDebugTarget` implementation** — the interface was designed generically for this from the start, but no second implementation exists yet to prove it out.
- **A visual, burned-in-pixel screenshot timestamp** — deliberately deferred in favor of the companion-file approach (§3.6); revisit if the on-image version is still wanted.
