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
| **F6** | Toggles a continuous frame recording on/off — same idea as F3 but for a whole span of frames instead of one instant. See §3.8. |
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

- **`GetMemorySpaces()`** — a list of named, sized, byte-addressable regions (`IDebugMemorySpace`: `Name`, `Size`, `IsWritable`, `HasSideEffects`, `Read(addr)`, `Write(addr, value)`). A generic memory viewer just lists whatever a target reports — SNES reports `CpuBus / WRAM / VRAM / CGRAM / OAM / SRAM`; an eventual NES target would report its own different set through the exact same shape. `HasSideEffects` (added alongside §3.9's `search` command, also used by §3.10's `snapshot`) flags spaces where `Read()` can do more than return a byte — SNES's `CpuBus` routes through live `MemoryBus.Read8`, which includes registers like RDNMI (clears the pending-NMI flag on read) or OPHCT/OPVCT (toggle a byte-order latch on read); `WRAM`/`VRAM`/`CGRAM`/`OAM` are plain arrays and always report `false`, and `SRAM` reports `false` too despite being bus-routed, since its address range never reaches an actual hardware register. Exists so a bulk, read-every-address tool can refuse to scan a space where doing so would silently perturb running emulation state.
- **`GetCpuRegisters()` / `GetVideoRegisters()`** — lists of `DebugRegisterValue` (name, value, bit-width for display formatting), not fixed struct fields — because different CPUs have wildly different register sets.
- **`GetSprites()`** — `DebugSpriteInfo` records (index, x, y, width, height, tile, palette, priority, flip flags) — generic enough to cover very different sprite hardware.
- **`GetPalettes()`** — `DebugPaletteInfo` records, colors *already converted* to display-ready RGB, so nothing downstream needs to know a console's native color format (SNES BGR555, etc.).
- **`Watches`** — the watchpoint registry (see §3.5). Exposed directly rather than re-wrapped into more `IDebugTarget` methods, since `WatchRegistry` is already core-agnostic on its own.
- **`FrameCount`** — a monotonic frame counter, owned as real state (not a frontend-local variable) so anything built against the interface can ask "what moment is this" consistently regardless of which core is running. Currently used to timestamp screenshots (§3.6); the same value is available to anything else that wants it.
- **`Disassemble(spaceName, address, count)`** — returns plain `DisassembledInstruction` records (address, raw bytes, mnemonic, formatted operand text). The interface knows nothing about a given CPU's addressing modes or operand-formatting conventions; all of that lives inside each core's implementation (§3.7). A target with no disassembler can legitimately return an empty list.
- **`GetSummaryText()`** — a free-text escape hatch for whatever isn't (yet) modeled as structured data above.

**Deliberately not included yet:** breakpoints and single-stepping. The execution loop can't pause mid-frame today (it runs a whole frame at a time) — that's real, separate future work, expected to be *additive* to this interface rather than a rework of it. Disassembly (previously in this same "not yet" list) now exists — see §3.7.

### 3.2 `SnesDebugTarget` (`Cores/Nintendo/Venus - SNES/Debug/SnesDebugTarget.cs`)

The SNES implementation. Almost entirely a *reshaping* of already-existing, already-verified logic (the same OAM size/high-table decoding `DumpActiveOam` always used, the same register fields `StateDump` already read) into the structured shapes `IDebugTarget` asks for — not new emulation logic.

Two small helper classes back the memory spaces:
- `ByteArrayDebugMemorySpace` — wraps a `byte[]` directly (WRAM, VRAM, CGRAM, OAM).
- `BusDebugMemorySpace` — routes through `MemoryBus.Read8`/`Write8` at a fixed bank offset (used for the raw CpuBus space, and for SRAM, which is more naturally viewed at its mapped CPU address `$70:0000`).

`Watches` here just returns `SnesDebugTarget`'s own `WatchRegistry` instance. **This changed since first built:** the registry (and a `Cpu` back-reference, `DebugCpu`) used to live directly on `MemoryBus`, which mixed real emulation state with debug-toolchain plumbing in the same class — a coupling issue caught during a later architecture review. Now `MemoryBus` exposes only a tiny, debug-agnostic `IWriteObserver` hook (`Cores/Nintendo/Venus - SNES/Memory/IWriteObserver.cs`) that it calls on every write with no idea what's listening; `SnesDebugTarget` implements that interface, owns the `WatchRegistry` itself, and supplies the PC context from its own already-held `Cpu` reference. `MemoryBus` no longer references `Cpu` or the debug toolchain at all.

### 3.3 `DebugCommandProcessor` (`Debug/DebugCommandProcessor.cs`) + `Debug/Commands/`

A small, composable command layer over `IDebugTarget` — modeled on Unix toolchain conventions (`ls`/`xxd`/`objdump`: small single-purpose commands) rather than one monolithic dump. Takes a line of text, returns a line of text — it doesn't know or care whether that text came from a console prompt, a future headless CLI, or eventually a GUI debug window's command box.

**Restructured from a growing pile of `Cmd*` methods into one small class per command**, each under `Debug/Commands/` implementing `IDebugCommand` (`Name`, `Usage`, `Execute(target, parts)`). This applies the exact "small composable tools instead of a monolith" idea the commands themselves were always designed around to the *implementation* too — `DebugCommandProcessor` had been quietly turning into the monolith its own commands were built to avoid, one tool at a time. `DebugCommandProcessor` itself is now a thin dispatcher: builds a `Name -> IDebugCommand` lookup once in its constructor, routes `Execute(commandLine)` to whichever command matches, and assembles `help`'s output from each command's own `Usage` text rather than one hand-maintained string. `search`/`snapshot`'s per-session state (§3.9, §3.10) now lives as private fields directly on `SearchCommand`/shared via `SnapshotStore` between `SnapshotCommand`/`DiffCommand`, instead of on `DebugCommandProcessor` — each command owns exactly the state it needs and nothing it doesn't. Same pattern `IDebugTarget` (core-agnostic contract, swappable per-core implementation) and `ICore` already proved works well in this codebase, applied one layer down.

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
| `search <space> <val> [<width>]` | Start a memory search — see §3.9 |
| `search refine\|changed\|unchanged\|increased\|decreased\|list\|reset` | Narrow/inspect/clear the active search — see §3.9 |
| `snapshot <space> <name>` / `snapshot list\|remove <name>` | Capture/manage a named memory baseline — see §3.10 |
| `diff <name> [<count>]` | Compare a snapshot against current contents — see §3.10 |
| `trace <count>` / `trace off` | Arm/cancel a live CPU instruction trace |
| `summary` | Free-text fallback (delegates to `StateDump.DumpAll`) |

Addresses/values accept `0x`, `$`, or bare hex.

**`tile`** generalizes what used to be `DebugTools.DecodeTileAscii` plus `CoinTileDumpLogging`'s hardcoded call site — works against *any* memory space (not just VRAM) and now supports 8bpp too (relevant since Mode 3/4's 8bpp BG1 and Direct Color exist now, which didn't when the original helper was written).

**Shared, stateless helpers** (`ParseHex`, `FindSpace`, `ReadValue`) live in `Debug/Commands/DebugCommandHelpers.cs`, a static class in the same spirit as `DebugTools.cs` — used via `using static` in whichever commands need them, rather than duplicated per command or hung off `DebugCommandProcessor` itself.

**Not interactive in the pause-emulation sense** — same reason as §3.1: each call to `Execute()` is one-shot, runs against current state, returns immediately. The console's F4 prompt just calls this in a loop.

### 3.4 What's deliberately *not* routed through this yet

- **F2's sprite dump** still calls `renderer.DumpActiveOam` directly rather than the `sprites` command. That method has a second responsibility — it also returns rects consumed by the "O" key's overlay drawing — that `SnesDebugTarget.GetSprites()` intentionally doesn't take on, to keep the toolchain's data-producing role separate from the renderer's overlay-geometry role. The `sprites` command (via F4) is the generalized, going-forward equivalent of just F2's printed output.
- **`StateDump`, `Debug/DebugTools.cs`, `Renderer.Debug.cs`** (below) are all still called directly in a few places rather than fully absorbed into the new layer — they're lower-level building blocks the new layer sometimes wraps (`GetSummaryText()` → `StateDump.DumpAll`) rather than things it replaced outright.
- **The watch registry currently only hooks WRAM reads and writes.** `MemoryBus`'s `IWriteObserver`/`IReadObserver` hooks (§3.5) are only called from the WRAM read/write paths today — `Ppu`'s VRAM/CGRAM/OAM accesses and SRAM don't report to either yet. Built for what the Yoshi investigation needed, with wiring in the other paths as natural, additive follow-up whenever an investigation needs to watch one of them. **All three of WRAM's own write paths are covered, found the hard way, one bug at a time:** direct bank-$7E/$7F addressing (original), the WMDATA `$2180` port (missed initially — a game writing through it would have been invisible to every watch until this was caught), and the low-RAM `offset < 0x2000` mirror used by bank-$00-relative absolute addressing (also missed initially, caught while investigating why a WRAM job table the Yoshi investigation depends on never showed a single recorded write despite clearly holding real, varying data — see `EmuSen_Core_Gameplan.md`'s current-status section). The same three paths now also report reads via the mirrored `IReadObserver` hook, added alongside read watchpoints (§3.5).

### 3.5 `WatchRegistry` (`Debug/WatchRegistry.cs`) — watchpoints

The generalized version of the recurring "log every write to this address range" pattern this project kept building ad hoc (`CameraRamLogging`, `MosaicWriteLogging`, `DmaSourceAddrLogging`, and originally a one-off Yoshi-specific WRAM trace). Instead of adding a new `DebugSettings` flag and a hand-written range check in the emulation code every time an investigation needs one, register a watch through the toolchain instead.

Each watch has a `WatchKind` — `Write` (default), `Read`, or `Both` — mirroring GDB's `watch`/`rwatch`/`awatch`. Write-only was this mechanism's original (and still default) behavior, so every pre-existing `watch add` invocation keeps meaning exactly what it always meant.

- **`AddWatch(spaceName, address, length, kind = Write)` → id** — register a watch on a memory-space address range, optionally restricted to reads, writes, or both.
- **`RemoveWatch(id)`** — remove one.
- **`GetWatches()`** — list active watches, including each one's `Kind`.
- **`RecordWrite(spaceName, address, value, contextFactory)`** / **`RecordRead(spaceName, address, value, contextFactory)`** — called by whoever owns the registry (`SnesDebugTarget`, via its `IWriteObserver.OnWrite`/`IReadObserver.OnRead` implementations — see §3.2) in response to an access reported by the core it's watching. Both delegate to one shared private `Record(accessKind, ...)` so the matching/storage/printing logic exists exactly once. Cheap to call even with zero watches registered (a quick scan over however many are active) — this matters more for reads than writes, since a read happens on every instruction fetch and operand read that touches a watched space, not just the writes a game actually makes. When an access matches an active watch's kind, it's both **printed live** (`[WATCH #id] W ...` or `[WATCH #id] R ...` — so the existing "play, then grep the console log" workflow keeps working with zero changes) and **stored** in a bounded per-watch buffer (last 500 events), so it's also queryable later via `watch log` without needing a fresh log upload.
- **`GetEvents(id, maxCount)` / `ClearEvents(id)`** — read or clear a watch's stored buffer.

`DebugWatchEvent` carries a sequence number, an `AccessKind` (`Write`/`Read`), address, value, and a free-text `Context` string (typically `PC=0x00A358`) — kept as text rather than a structured field since what's useful context varies by core and shouldn't force an interface change every time a new kind becomes relevant.

Core-agnostic on purpose, same as `IDebugTarget` — lives under `Debug/`, not `Cores/Nintendo/Venus - SNES/`, since nothing about it is SNES-specific. `SnesDebugTarget` owns the instance and reports accesses to it via `MemoryBus`'s `IWriteObserver`/`IReadObserver` hooks (`Cores/Nintendo/Venus - SNES/Memory/IWriteObserver.cs`, `IReadObserver.cs`) — `MemoryBus` itself has no idea `WatchRegistry` exists. `IReadObserver` is a deliberately separate interface from `IWriteObserver` rather than an added method on it, so a future implementer that only cares about one kind of access can implement just the interface it needs; `SnesDebugTarget` implements both. A future NES `IDebugTarget` would own its own instance the same way, wired to its own core's equivalent observer hooks.

**Example — reproducing the Yoshi WRAM trace on demand** instead of it being a permanent flag: press F4 once after launching, then:
```
watch add WRAM 8000 1800
```
Matching writes for the rest of that session get printed live exactly as before, and can also be pulled on demand later with `watch log <id>`.

**Example — watching for reads instead of (or as well as) writes:**
```
watch add WRAM D80 20 read
watch add WRAM D80 20 both
```
The first only logs reads of the $0D80-$0D9F job table; the second logs both directions. `watch list` shows each watch's kind alongside its range.

### 3.6 Screenshot timestamping (F3, `Frontend/Program.cs`)

F3 saves a PNG the same way it always has, but now also writes a companion `screenshot_frame<N>.txt` next to it, and the `[SCREENSHOT]` console log line states the same information explicitly:

```
Core: SNES
Frame: 1378
Wall-clock: 2026-07-22 07:28:14.203
```

All three (filename, companion file, log line) are sourced from the same `debugTarget.FrameCount`/`CoreName` — so they can't drift out of sync with each other the way a hand-drawn on-screen frame counter and a separately-incremented save filename could. This is what `IDebugTarget.FrameCount` (§3.1) is for: a single, reliable, core-agnostic way to answer "what moment was this" that a screenshot, a log line, and eventually a GUI's own display can all agree on.

**Deliberately not done:** burning the timestamp into the image's pixels (a visual overlay baked into the PNG itself, closer to a security-camera timestamp). That would need Raylib's `LoadImageFromScreen`/`ImageDrawText`/`ExportImage` functions, and given a couple of build-error round trips this session already came from guessing at exact API shapes, the companion-file approach was chosen deliberately as the zero-new-API-surface, guaranteed-to-build option. Worth revisiting if the visual version is still wanted later.

### 3.7 Disassembler (`Cores/Nintendo/Venus - SNES/Cpu/Disassembler/Snes65816Disassembler.cs`)

A full 65816 disassembler, wired in via `IDebugTarget.Disassemble` and the `disasm` command:
```
disasm <space> <addr> [<count>]     disassemble <count> instructions (default 10)
```
```
  00A358: 8D 22 43  STA $4322
  00A35B: A9 40 00  LDA #$0040
```

**Deliberately a separate table from the execution opcode table** (`Cpu.OpcodeTable.cs`), not built from it — that table's addressing-mode delegates have real execution side effects (advancing PC, consuming cycles) and have no way to report "how many bytes would this take" without actually running the instruction. `Snes65816Disassembler` is a completely independent, read-only mnemonic + addressing-mode table built specifically for display, living under `Cores/Nintendo/Venus - SNES/Cpu/Disassembler/` rather than inside `Cpu.cs` itself.

**Important, deliberately-stated caveat — read this before trusting it for anything beyond casual use.** This project's *execution* opcode table got a dedicated verification pass against oxyron.de before being trusted (see the original project handoff notes — "completed 236→256/256, verified against oxyron.de, caught one cross-reference typo"). This disassembly table has **not** had the equivalent treatment; there's no way to build and run this project from wherever it gets edited to cross-check it the same way. It's built carefully against the standard, universally-documented 65816 opcode matrix, and two entries were spot-checked against real bytes captured during the Yoshi DMA investigation earlier this session (`8D 22 43` → `STA $4322`, `A9 40 00` → `LDA #$0040`, both correct) — but that's meaningfully short of a real verification pass. Treat it as a solid first draft, not a verified reference, until it gets one.

**Fixed (previously a documented simplification):** the 65816's immediate-mode instructions (`LDA #`, `LDX #`, etc.) have an operand length that depends on the *current* M/X flags in the P register — 1 byte if 8-bit, 2 if 16-bit. `Disassemble` used to read the CPU's *current* E/M/X flags once and apply that width to the *entire* requested range, so a `REP`/`SEP` inside the range desynced every instruction after it until decoding happened to resync on a real opcode boundary by luck — exactly what happened live during the Yoshi/coin WRAM investigation (`disasm CpuBus A317 40` produced a run of garbage — a spurious `BRK`, nonsense operands — starting right after an `LDA #` before resyncing several instructions later). `Disassemble` now tracks `REP`/`SEP` live as it walks the requested range, so both sides of a width change decode correctly in the same call.

**Remaining, still-real gap:** M/X (and E itself, via `XCE`) are properties of a specific point in the CPU's actual control flow, not global constants. The fix above only tracks changes *within* the requested range — it still starts from the CPU's flags *right now*, which may not match the flags actually in effect when the code at `<addr>` really executes, if `<addr>` isn't the current PC (the normal case when disassembling some other routine while paused elsewhere). There's no way to know that without either tracing real execution to that address or full control-flow analysis, neither of which a static, read-only disassembler does. `XCE` changing emulation mode mid-range is a related, separate gap: the disassembler has no carry flag to swap with, so it can't follow an `XCE`-driven mode change even within one call. Treat immediate-mode operand widths as best-effort for anywhere other than the current PC; opcode/addressing-mode decoding itself is unaffected either way. **Confirmed live** against the Yoshi investigation's ground-truth CPU trace: real execution at `$00A31F` proved the accumulator genuinely is 16-bit there (`LDA #$imm` really is 3 bytes), while `disasm`'s starting guess — taken from wherever the CPU happened to be paused when F4 was pressed — assumed 8-bit and produced the same "garbage, then lucky resync" pattern four separate times in that one routine. The `REP`/`SEP`-tracking fix itself worked correctly throughout (proven by a later `LDA #$1801` in the same dump decoding right once it saw the preceding `REP #$20`); the remaining garbage is entirely this documented, harder, not-yet-solved gap, not a regression in the fix.

### 3.8 Frame recording (F6, `Debug/FrameRecorder.cs` + `Frontend/Program.cs`)

Continuous version of F3's screenshot-plus-metadata pattern (§3.6): instead of one PNG for a single instant, captures a whole span of frames into a session folder, with one ledger file (`frames.log`, tab-separated: `Frame`, `WallClock`, `Core`, `ImageFile`) mapping every captured image back to exactly when it happened — the same cross-reference purpose F3's companion `.txt` already served, spread across a sequence instead of one moment.

```
F6   toggle recording on/off
```

Starts a new session folder under `Logs/Recordings/<CoreName>_<timestamp>/` each time it's turned on; `[RECORD] Started -> <path>` / `[RECORD] Stopped: <path>` confirm state in the console log. Supports an optional frame stride (capture every Nth frame rather than every single one — a long recording at every frame gets large fast) via `FrameRecorder.Start(baseDir, frameStride)`; the F6 hotkey itself uses the default of every frame, intended for short, targeted captures around a specific moment (e.g. wrapping the whole span of a suspected rendering bug) rather than recording an entire play session.

**Core-agnostic by the same pattern as the rest of this toolchain:** `FrameRecorder` only touches `IDebugTarget` (`CoreName`/`FrameCount`, same as F3 uses via `debugTarget`) and never anything console- or renderer-specific. The one Raylib-specific piece — actually grabbing a frame's pixels — is supplied by the frontend as a callback (`path => Raylib_cs.Raylib.TakeScreenshot(path)` in `Program.cs`'s F6 handler) rather than living inside `FrameRecorder` itself, so a future frontend using a different rendering API reuses this class unchanged and only needs to supply its own capture callback.

**Automatic video encode on stop, via `ffmpeg`.** A PNG-per-frame folder is a hot mess to actually review, so `Stop()` shells out to `ffmpeg` (if it's on `PATH`) to mux the just-captured sequence into `recording.mkv` in the same session folder, using `ffmpeg`'s glob image2 demuxer (`-pattern_type glob -i "frame_*.png"`, which only needs lexically-increasing filenames — already true since they're named from `FrameCount`, zero-padded — not strictly-sequential integers, so this works unchanged even with a frame stride > 1). Framerate passed to `ffmpeg` is `60.0 / frameStride`, the same "~60fps" approximation already used elsewhere in this codebase (see `Program.cs`'s `SaveEveryNFrames` comment), not a timing-accurate real-hardware rate.

**Codec choice: FFV1-in-Matroska, deliberately lossless, not VP9/H.264.** This exists to inspect exact pixel-level rendering bugs (the whole reason it exists — Yoshi's invisible-sprite investigation); a lossy codec's own compression artifacts would work against that exact purpose. Both FFV1 and Matroska are open formats.

**Loose PNGs get zipped and deleted after a successful encode, not left lying around.** Once `ffmpeg` confirms success (`ExitCode == 0`), the frame PNGs are fully redundant — FFV1 is lossless, so `recording.mkv` already contains everything they do — and a folder of potentially thousands of individual images is exactly the mess this feature exists to avoid. They're archived into `frames.zip` (same session folder) and the loose files deleted; `frames.log` (the small frame/timestamp ledger) is left alone regardless, since it's cheap to keep and useful without unzipping anything. If the *zip* step itself fails, the PNGs are left in place rather than risking data loss over a tidiness step — the video already succeeded either way. If `ffmpeg` itself fails, none of this cleanup runs at all; the loose PNGs are the only artifact and stay untouched (see below).

**Not required.** If `ffmpeg` isn't on `PATH`, `Stop()` catches that (`Win32Exception`) and just logs that the PNG sequence + ledger are the usable result, same as before this existed — this is a convenience layer, not a hard dependency of the recording feature itself.

**Blocks synchronously while encoding.** `Stop()` waits for `ffmpeg` to finish before returning, so stopping a long recording will visibly freeze the emulator for the encode duration — same class of tradeoff the F4 debug prompt already has (the execution loop can't pause mid-frame independent of this either way). Would need to move off the main thread if this ever needs to not block real-time play.

**Verified against a real run (Fedora 44, ffmpeg 8.1.2)** — and it caught a real bug on the first attempt: `outputPath` was passed to `ffmpeg` as the full `sessionDir/recording.mkv` path *while `ProcessStartInfo.WorkingDirectory` was already set to `sessionDir`*, so `ffmpeg` tried to resolve it relative to a directory it was already inside, landing on a doubly-nested path that doesn't exist (`Error opening output ...: No such file or directory`). Fixed by passing just the bare filename (`recording.mkv`) as the output argument, since `WorkingDirectory` already puts `ffmpeg` in the right place. Also dropped the forced `-pix_fmt rgb24` — Raylib's screenshots are RGBA, `ffmpeg` flagged `rgb24` as incompatible with FFV1 and auto-selected `bgr0` anyway (successfully), so removing the forced flag in favor of that auto-negotiation is both simpler and matches what actually worked.

### 3.9 Memory search (`search`, `Debug/Commands/SearchCommand.cs`)

Classic "first scan, then narrow" memory search — Cheat Engine's model, adapted to this console: find where a game stores something (a score, a life counter, a flag) without already knowing the address, which none of `mem`/`write`/`watch`/`tile` cover on their own (they all need an address up front). Pure `IDebugMemorySpace.Read()` scans; no SNES-specific logic, so it works unchanged for a future core's memory spaces.

```
search <space> <value> [<width>]   start a new search: every address currently equal to <value>
                                    (width in bytes: 1, 2, or 4 - default 1, little-endian)
search refine <value>              narrow to addresses now equal to <value>
search changed | unchanged         narrow to addresses whose value did/didn't change since last search/refine
search increased | decreased       narrow to addresses whose value went up/down since last search/refine
search list [<count>]              list current candidates + values (default 20)
search reset                       clear the current search
```

One active search session at a time — starting a new `search <space> <value>` replaces whatever was running, matching the command's own "first scan" framing. Session state (candidate address list, last-known values for the changed/unchanged/increased/decreased comparisons) lives as plain fields on `DebugCommandProcessor`, not its own class — there's genuinely only ever one session, so a richer structure wouldn't buy anything.

**Refuses to scan a space where `HasSideEffects` is true** (see §3.1) — SNES's `CpuBus` is the concrete case: a full-range scan would sequentially read every hardware register, including ones with real read side effects (RDNMI clearing the pending-NMI flag, the manual joypad port shifting its serial data on every read), silently corrupting whatever's actually running. There's no legitimate reason to bulk-search live registers anyway — real game state lives in `WRAM`/`SRAM`, both of which are safe (`HasSideEffects == false`) and where every realistic use of this command should be pointed.

**One real bug caught before this ever shipped, not after:** `ParseHex` returns a signed `int`, so a width-4 search value with the high bit set (e.g. `FFFFFFFF`) would parse as `-1` and never match `ReadValue`'s always-non-negative multi-byte accumulation. Both the initial-search and `refine` value parses mask with `& 0xFFFFFFFFL` to reinterpret the bit pattern as unsigned before comparing — caught during review, not by a failed run, so flagged here in case the same shape of bug (`ParseHex` feeding a signed value into an unsigned comparison) shows up again in a future command.

### 3.10 Memory snapshot/diff (`snapshot`, `diff`, `Debug/Commands/{SnapshotCommand,DiffCommand,SnapshotStore}.cs`)

The general-case complement to `search`'s `changed`/`unchanged`/`increased`/`decreased`: those narrow a *fixed set of candidate addresses* search already found. `snapshot`/`diff` need no prior candidates at all — capture an entire memory space's contents now, compare against its later contents whenever, see every address that's different. The exact "what changed between frame X and frame Y" shape most investigations in this project (including the Yoshi/coin one) end up asking by hand via log-grepping; this makes it a real command instead.

```
snapshot <space> <name>      capture <space>'s full current contents under <name>
snapshot list                list saved snapshots (name, space, size)
snapshot remove <name>       delete a saved snapshot
diff <name> [<count>]        compare snapshot <name> against that space's CURRENT contents,
                              print addresses that changed (default 20 shown)
```

**Multiple named snapshots can coexist**, unlike `search`'s single session — stored in a `Dictionary<string, (SpaceName, Data)>` rather than plain fields, since there's real value in keeping more than one baseline around (e.g. one taken right before Yoshi's block opens, another right after, each diffable independently and repeatedly without disturbing the other).

**`diff` never mutates the saved snapshot.** Running `diff` twice against the same name, with real time passing in between, always compares against the *original* capture — not the last `diff` call's result the way `search`'s `changed`/`unchanged` chains forward each time. Re-`snapshot` with the same name if a new baseline is actually wanted; otherwise the same snapshot is a stable reference point for as many `diff` calls as needed.

**Same `HasSideEffects` guard as `search`, same reasoning** — a full-space capture of `CpuBus` would sequentially read every hardware register, some with real side effects on read. `snapshot` refuses outright for any space where `HasSideEffects` is true.

**Byte-granularity only, unlike `search`'s configurable width.** A "what changed" comparison doesn't need to already know a value's byte width the way a value-search does — any single byte differing is itself the useful signal, and a wider changed *value* just shows up as multiple adjacent single-byte diffs in the output. Kept deliberately simpler than `search` for that reason, not as an oversight.

### 3.11 Memory dump/load to file (`dump`, `load`, `Debug/Commands/{DumpCommand,LoadCommand}.cs`)

Takes a `snapshot`-style capture out of the process entirely, to disk, as raw bytes — for handing off to an external hex editor, diffing against a known-good ROM's own data, or just keeping a capture around after the session that took it ends (unlike `snapshot`, which lives only in memory for that session). `load` is the reverse: poke a raw byte file back into a memory space at a given address.

```
dump <space> <addr> <len> <file>   write raw bytes to Logs/<file>
load <space> <addr> <file>         write Logs/<file>'s raw bytes into <space> starting at <addr>
```

Both always resolve `<file>` under `Logs/` — the same directory F3 screenshots and their companion `.txt` files already use, so debug artifacts from a session end up in one predictable place rather than scattered relative to wherever the process happened to be launched from.

**Same `HasSideEffects` guard as `search`/`snapshot`** on `dump` — refuses a bulk read over `CpuBus` or any other space where reading can disturb live hardware state. `load` has no equivalent read-side concern but does check `IsWritable`, refusing to write into a read-only space.

**No format, no header — just the bytes.** `dump`'s output is exactly `len` raw bytes starting at `addr`; `load` writes exactly however many bytes the file contains, starting at `addr`, with no length argument of its own (the file's own size *is* the length). This keeps both directly interoperable with any external tool that reads/writes plain binary — a hex editor, `xxd`, a Python script — without EmuSen needing to define or parse its own container format.

**Distinct from save states.** `EmulatorSession`/`VenusCore`'s `SaveState`/`LoadState` (F5/F9 in the frontend) capture *everything* needed to resume execution — CPU, PPU, full bus state — as one opaque blob. `dump`/`load` capture *one named memory space* as plain bytes, readable and editable by anything, with no claim to being a complete or resumable snapshot of emulation state. Different tools for different jobs: F5/F9 for "come back to this later," `dump`/`load` for "take this data somewhere else and look at it."

---

## 4. Underlying helper libraries (pre-date the toolchain above)

### `Cores/Nintendo/Venus - SNES/Debug/StateDump.cs`
On-demand CPU+PPU snapshot formatter. Returns formatted strings (doesn't print directly) — `DumpCpuState`, `DumpPpuState`, `DumpAll`. Deliberately laid out to be directly comparable to MesenCE's own Status panel.

### `Debug/DebugTools.cs`
Generic, byte-array-level helpers — reusable for any console's data since none of it hardcodes SNES specifics beyond parameter naming:
- `HexDump(label, data, address, count)` — plain hexdump formatter.
- `DecodeTileAscii(vram, address, bpp)` — ASCII-art tile decoder, parameterized by bit depth. **Note:** the `tile` command (§3.3) is now the more general, going-forward way to do this (works against any memory space, supports 8bpp) — this function itself is unchanged and still callable directly, but new code should probably reach for the command instead.
- `CgramToRgb(lo, hi, brightness)` / `DumpPalette(cgram, paletteIndex, bpp, brightness)` — SNES color-format helpers.
- `DescribeBits(label, value, bits)` — labeled bit-flag formatter.
- `BoundedTrace` — a start/countdown helper for "trace the next N frames then auto-stop" (used by the P-key scroll trace).
- `ChangeTracker<T>` — "did this value change since last time" helper, used throughout `DebugSettings`-gated change-only logging.

### `Cores/Nintendo/Venus - SNES/Ppu/Renderer/Renderer.Debug.cs`
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
