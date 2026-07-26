# EmuSen Debugging Tools — Reference

This document covers every debugging tool currently in the project: what it does, where it lives, how to trigger it, and how the pieces fit together. It's organized from "things you press a key for" down to "the underlying reusable toolchain," since that's roughly the order you'd reach for them in.

*(Living document — updated as the toolchain grows. This revision: corrects the breakpoints/single-stepping section, which existed by the time this doc was last touched but was still described as future work; documents `GetApuRegisters()`/`regs`' new APU section; fills in the command table's missing rows (`break`, `framelog`, `callers`/`writers`/`readers`, `dump`/`load`, `cheat`); and adds the two harness-level tools that had no coverage at all despite becoming primary tools in practice - `EmuSen.HeadlessDebug` (§3.15) and `EmuSen.Validation` (§3.16). Previous revision added the watchpoint mechanism, the generalized `tile` command, the shared `FrameCount`/screenshot timestamping, and a full 65816 disassembler — retiring `CoinTileDumpLogging` along the way. Also reflects a later decoupling pass: `WatchRegistry` moved from being owned by `MemoryBus` to being owned by `SnesDebugTarget`, communicating through a new, debug-agnostic `IWriteObserver` hook instead of `MemoryBus` holding a concrete `WatchRegistry`/`Cpu` reference directly.)*

---

## 1. Console hotkeys (Raylib console build, `EmuSen.RaylibFrontend/Program.cs`)

These all live in the same per-frame hotkey block in `Program.cs`, checked once per frame after `presenter.Present(...)` (`FramePresenter` — see `EmuSen_Frontend_Driver.md` §1 step 6).

| Key | Does |
|---|---|
| **F1** | Full CPU + PPU state snapshot. Formatted to be directly comparable to MesenCE's Status panel. |
| **F2** | Dumps every active OAM sprite's X/Y/tile/attribute bytes — ground truth for sprite investigations. |
| **F3** | Saves the current frame as a PNG (`Logs/<CoreName>/screenshot_frame<N>.png`), plus a companion metadata file (`Logs/<CoreName>/screenshot_frame<N>.txt`) with the core name, frame number, and wall-clock time — see §3.6. |
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

**Breakpoints and mid-frame halt/resume now exist** (this used to be the "not included yet" item in this list, alongside disassembly - disassembly landed first, see §3.7). `IDebugTarget.Breakpoints` exposes a `BreakpointRegistry` (`break add/list/remove` - §3.3's table), the same add/list/remove-only shape `Watches`/`FrameLog` already use; actually halting/resuming is NOT part of `IDebugTarget` itself (a registry edit doesn't know or care whether anything is currently running), it lives on `VenusCore.RunFrame()`: a breakpoint hit sets `IsHaltedAtBreakpoint`/`HaltedAddress` and returns immediately, mid-scanline if necessary, with all the scanline-loop's own state (`_scanlineStarted`, `_lineCycles`, etc.) left exactly as it was so the *next* `RunFrame()` call resumes the same in-progress frame instead of restarting it. The console's F4 prompt (`step`/`s`, `continue`/`c`) drives this by checking `IsHaltedAtBreakpoint` after each `RunFrame()` call and deciding whether to call it again immediately (continue) or wait for the next hotkey press (step) - see `EmuSen.RaylibFrontend/Program.cs`. `EmuSen.HeadlessDebug` (§3.15) doesn't drive halting at all today; its `break add` support is hit-counting only, useful for "did execution ever reach this address" questions in a scripted run without needing the halt/resume loop a live frontend provides.

### 3.2 `SnesDebugTarget` (`Cores/Nintendo/Venus - SNES/Debug/SnesDebugTarget.cs`)

The SNES implementation. Almost entirely a *reshaping* of already-existing, already-verified logic (the same OAM size/high-table decoding `DumpActiveOam` always used, the same register fields `StateDump` already read) into the structured shapes `IDebugTarget` asks for — not new emulation logic.

Two small helper classes back the memory spaces:
- `ByteArrayDebugMemorySpace` — wraps a `byte[]` directly (WRAM, VRAM, CGRAM, OAM).
- `BusDebugMemorySpace` — routes through `MemoryBus.Read8`/`Write8` at a fixed bank offset (used for the raw CpuBus space, for SRAM at its mapped CPU address `$70:0000`, and for the `IO` space at bank 0 - `watch add IO 4016 1`/`mem IO 4218 4` target hardware registers directly, added after a real investigation found `watch add` silently rejected them since nothing had registered that space name - see `Venus_Memory.md` §6/§7).

`Watches` here just returns `SnesDebugTarget`'s own `WatchRegistry` instance. **This changed since first built:** the registry (and a `Cpu` back-reference, `DebugCpu`) used to live directly on `MemoryBus`, which mixed real emulation state with debug-toolchain plumbing in the same class — a coupling issue caught during a later architecture review. Now `MemoryBus` exposes only a tiny, debug-agnostic `IWriteObserver` hook (`Cores/Nintendo/Venus - SNES/Memory/IWriteObserver.cs`) that it calls on every write with no idea what's listening; `SnesDebugTarget` implements that interface, owns the `WatchRegistry` itself, and supplies the PC context from its own already-held `Cpu` reference. `MemoryBus` no longer references `Cpu` or the debug toolchain at all.

**`GetApuRegisters()`** (added investigating a Super Metroid boot hang - `Venus_APU.md` §2.7) exposes the SPC700's own A/X/Y/SP/PC/PSW plus both directions of the CPU↔APU communication ports (`InPort0-3` = what the CPU last wrote, `OutPort0-3` = what the SPC700 last wrote - see `Venus_APU.md` §1.2 for why those are two independent latches per port, not one). `regs` (§3.3's table) prints this as a third "APU registers" section whenever a target's list is non-empty; a core with no distinct sound co-processor just returns an empty list and `regs` skips the section. Before this existed, the only way to see SPC700 state at all was reading raw `Spc700VerboseLogging` trace text.

**`TilemapEntryStride`/`DecodeTilemapEntry`** back the `tilemap` command (§3.3's table) - added so a menu cursor's position or a HUD tile change can be confirmed by comparing tilemap entries as text instead of eyeballing screenshots. Deliberately NOT a generic bit-layout `IDebugTarget` parses itself: an NES core's nametable+attribute-table split isn't even the same shape as the SNES's single packed word (one byte per tile, plus a separate, coarser attribute byte covering a 2x2 tile block), so every core decides both how many bytes make up "one entry" (`TilemapEntryStride`) and how to render one (`DecodeTilemapEntry`) - same "core does the decoding, the command does the grid-walking" split as `Disassemble()`/`GetCpuRegisters()`. `SnesDebugTarget`'s implementation decodes the standard SNES BG screen word (2 bytes: bits 0-9 tile index, 10-12 palette, 13 priority, 14 h-flip, 15 v-flip) into a fixed 7-character label - 3 hex digits (tile index) + 1 digit (palette 0-7) + one character each for priority/h-flip/v-flip (`P`/`H`/`V` if set, `.` if not) - e.g. `1A32PHV` = tile `$1A3`, palette 2, priority set, both flips set; `1A32...` = same tile/palette with none of those three set.

### 3.3 `DebugCommandProcessor` (`Debug/DebugCommandProcessor.cs`) + `Debug/Commands/`

A small, composable command layer over `IDebugTarget` — modeled on Unix toolchain conventions (`ls`/`xxd`/`objdump`: small single-purpose commands) rather than one monolithic dump. Takes a line of text, returns a line of text — it doesn't know or care whether that text came from a console prompt, a future headless CLI, or eventually a GUI debug window's command box.

**Restructured from a growing pile of `Cmd*` methods into one small class per command**, each under `Debug/Commands/` implementing `IDebugCommand` (`Name`, `Usage`, `Execute(target, parts)`). This applies the exact "small composable tools instead of a monolith" idea the commands themselves were always designed around to the *implementation* too — `DebugCommandProcessor` had been quietly turning into the monolith its own commands were built to avoid, one tool at a time. `DebugCommandProcessor` itself is now a thin dispatcher: builds a `Name -> IDebugCommand` lookup once in its constructor, routes `Execute(commandLine)` to whichever command matches, and assembles `help`'s output from each command's own `Usage` text rather than one hand-maintained string. `search`/`snapshot`'s per-session state (§3.9, §3.10) now lives as private fields directly on `SearchCommand`/shared via `SnapshotStore` between `SnapshotCommand`/`DiffCommand`, instead of on `DebugCommandProcessor` — each command owns exactly the state it needs and nothing it doesn't. Same pattern `IDebugTarget` (core-agnostic contract, swappable per-core implementation) and `ICore` already proved works well in this codebase, applied one layer down.

| Command | Usage |
|---|---|
| `help` | List commands |
| `spaces` | List memory spaces (name, size, writable) |
| `mem <space> <addr> [<len>]` | xxd-style hexdump, default length 16 |
| `write <space> <addr> <value>` | Write one byte, if the space is writable |
| `regs` | CPU + video registers, plus an APU section when a target has one (§3.2) |
| `sprites` | Active sprite/OBJ table |
| `pal [<index>]` | One palette, or all 16 if omitted |
| `tile <space> <addr> <bpp>` | ASCII-decode one 8x8 tile from any space (bpp 2, 4, or 8) |
| `tilemap <space> <addr> <cols> <rows>` | Decode a grid of raw tilemap entries as text (tile index/palette/priority/flip on SNES) — core-agnostic at the command level, see §3.2's `TilemapEntryStride`/`DecodeTilemapEntry` note |
| `disasm <space> <addr> [<count>]` | Disassemble `<count>` instructions (default 10) — see §3.7 |
| `watch add <space> <addr> <len> [write\|read\|both]` | Register a watchpoint (default write-only) |
| `watch list` | List active watchpoints with their IDs |
| `watch log <id> [<count>]` | Show a watchpoint's recorded events (default 20) |
| `watch summary <id>` | Group a watchpoint's recorded events by access site (`Context`) with hit counts, instead of one line per event — the dynamic, addressing-mode-agnostic equivalent of `readers`/`writers` (§3.12) |
| `watch clear <id>` | Clear a watchpoint's stored events (keeps the watch registered) |
| `watch remove <id>` | Remove a watchpoint entirely |
| `break add <addr>` / `break list` / `break remove <id>` | Manage execution breakpoints (24-bit CPU address) — see §3.1's breakpoints note |
| `framelog add <space> <addr> [<width>]` / `list` / `show <id> [<count>]` / `clear <id>` / `remove <id>` | Per-frame value sampling, independent of reads/writes — see §3.13 |
| `callers <addr> [<scanstart> <scanlen>]` | Find JSR/JSL/JMP instructions targeting `<addr>` — see §3.12 |
| `writers <addr> [<scanstart> <scanlen>]` | Find STA/STX/STY/STZ instructions targeting `<addr>` (absolute/absolute-long only — see that command's own usage text for why direct-page/indexed/indirect forms are excluded) |
| `readers <addr> [<scanstart> <scanlen>]` | Find LDA/LDX/LDY instructions reading `<addr>` (same absolute-only scope as `writers`) |
| `dump <space> <addr> <len> <path>` / `load <space> <addr> <path>` | Save/restore a memory range to/from a file — see §3.11 |
| `search <space> <val> [<width>]` | Start a memory search — see §3.9 |
| `search refine\|changed\|unchanged\|increased\|decreased\|list\|reset` | Narrow/inspect/clear the active search — see §3.9 |
| `snapshot <space> <name>` / `snapshot list\|remove <name>` | Capture/manage a named memory baseline — see §3.10 |
| `diff <name> [<count>]` | Compare a snapshot against current contents — see §3.10 |
| `trace <count>` / `trace off` | Arm/cancel a live CPU instruction trace |
| `cheat add\|poke\|gg\|rompatch\|list\|enable\|disable\|remove\|clear ...` | RAM-poke/ROM-patch cheat engine — see §3.14 |
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

### 3.6 Screenshot timestamping (F3, `EmuSen.RaylibFrontend/Program.cs`)

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

### 3.8 Frame recording (F6, `Debug/FrameRecorder.cs` + `EmuSen.RaylibFrontend/Program.cs`)

Continuous version of F3's screenshot-plus-metadata pattern (§3.6): instead of one PNG for a single instant, captures a whole span of frames into a session folder, with one ledger file (`frames.log`, tab-separated: `Frame`, `WallClock`, `Core`, `ImageFile`) mapping every captured image back to exactly when it happened — the same cross-reference purpose F3's companion `.txt` already served, spread across a sequence instead of one moment.

```
F6   toggle recording on/off
```

Starts a new session folder under `Logs/<CoreName>/Recordings/<CoreName>_<timestamp>/` each time it's turned on (the F6 hotkey passes `Logs/<CoreName>/Recordings` as the base directory; `FrameRecorder` itself still prefixes its own session folder name with the core name too, so it stays self-sufficient even for a caller that doesn't already organize by core); `[RECORD] Started -> <path>` / `[RECORD] Stopped: <path>` confirm state in the console log. Supports an optional frame stride (capture every Nth frame rather than every single one — a long recording at every frame gets large fast) via `FrameRecorder.Start(baseDir, frameStride)`; the F6 hotkey itself uses the default of every frame, intended for short, targeted captures around a specific moment (e.g. wrapping the whole span of a suspected rendering bug) rather than recording an entire play session.

**Core-agnostic by the same pattern as the rest of this toolchain:** `FrameRecorder` only touches `IDebugTarget` (`CoreName`/`FrameCount`, same as F3 uses via `debugTarget`) and never anything console- or renderer-specific. The one Raylib-specific piece — actually grabbing a frame's pixels — is supplied by the frontend as a callback (`path => Raylib_cs.Raylib.TakeScreenshot(path)` in `Program.cs`'s F6 handler) rather than living inside `FrameRecorder` itself, so a future frontend using a different rendering API reuses this class unchanged and only needs to supply its own capture callback.

**Automatic video encode on stop, via `ffmpeg`.** A PNG-per-frame folder is a hot mess to actually review, so `Stop()` shells out to `ffmpeg` (if it's on `PATH`) to mux the just-captured sequence into `recording.mkv` in the same session folder, using `ffmpeg`'s glob image2 demuxer (`-pattern_type glob -i "frame_*.png"`, which only needs lexically-increasing filenames — already true since they're named from `FrameCount`, zero-padded — not strictly-sequential integers, so this works unchanged even with a frame stride > 1). Framerate passed to `ffmpeg` is `60.0 / frameStride`, the same "~60fps" approximation already used elsewhere in this codebase (see `Program.cs`'s `SaveEveryNFrames` comment), not a timing-accurate real-hardware rate.

**Codec choice: `libx264rgb` at CRF 0, deliberately lossless, not a lossy codec like VP9/regular H.264.** This exists to inspect exact pixel-level rendering bugs (the whole reason it exists — Yoshi's invisible-sprite investigation); a lossy codec's own compression artifacts would work against that exact purpose. `-crf 0` is mathematically lossless in x264 (bit-exact, not just "visually lossless"); `libx264rgb` specifically (not plain `libx264`, which defaults to chroma-subsampled `yuv420p` — NOT lossless even at CRF 0) keeps the whole encode in RGB, matching Raylib's screenshots directly with no colorspace conversion to introduce rounding.

**Originally FFV1-in-Matroska, switched after real recordings came back far larger than expected for a short capture.** FFV1 is also lossless, but it's intra-frame-only — every frame is encoded independently, so a mostly-static stretch of gameplay (or the debug side panels, which barely change frame to frame, since the whole window is captured — see the capture callback note above) costs full data on every single frame. `libx264` still does inter-frame prediction even at CRF 0, so it can actually exploit that redundancy instead of re-encoding a nearly-identical frame from scratch 60 times a second. Container stays Matroska (`.mkv`) either way.

**Loose PNGs get zipped and deleted after a successful encode, not left lying around.** Once `ffmpeg` confirms success (`ExitCode == 0`), the frame PNGs are fully redundant — the encode is lossless, so `recording.mkv` already contains everything they do — and a folder of potentially thousands of individual images is exactly the mess this feature exists to avoid. They're archived into `frames.zip` (same session folder) and the loose files deleted; `frames.log` (the small frame/timestamp ledger) is left alone regardless, since it's cheap to keep and useful without unzipping anything. If the *zip* step itself fails, the PNGs are left in place rather than risking data loss over a tidiness step — the video already succeeded either way. If `ffmpeg` itself fails, none of this cleanup runs at all; the loose PNGs are the only artifact and stay untouched (see below).

**Not required.** If `ffmpeg` isn't on `PATH`, `Stop()` catches that (`Win32Exception`) and just logs that the PNG sequence + ledger are the usable result, same as before this existed — this is a convenience layer, not a hard dependency of the recording feature itself.

**Blocks synchronously while encoding.** `Stop()` waits for `ffmpeg` to finish before returning, so stopping a long recording will visibly freeze the emulator for the encode duration — same class of tradeoff the F4 debug prompt already has (the execution loop can't pause mid-frame independent of this either way). `-preset slow` favors compression ratio over encode speed; raise to `-preset veryslow` for a bit more, or drop to `-preset medium`/`fast` if the freeze itself becomes the bigger annoyance — encode time trades directly against file size here, same as any x264 preset. Would need to move off the main thread if this ever needs to not block real-time play at all.

**Verified against real runs (Fedora 44, ffmpeg 8.1.2), twice, two real bugs caught.** First: `outputPath` was passed to `ffmpeg` as the full `sessionDir/recording.mkv` path *while `ProcessStartInfo.WorkingDirectory` was already set to `sessionDir`*, so `ffmpeg` tried to resolve it relative to a directory it was already inside, landing on a doubly-nested path that doesn't exist. Fixed by passing just the bare filename. Second: switching to `libx264rgb` (see above) regressed recording entirely on a real machine whose `ffmpeg` build doesn't include `libx264` — a real, common situation, not hypothetical (`libx264` is patent-encumbered and left out of some distros' default `ffmpeg` package, e.g. plain Fedora repos as opposed to RPM Fusion's build; `ffv1` is unencumbered and always present). **Fix: `TryEncodeVideo` now tries `libx264rgb` first and automatically falls back to `ffv1`** if that specific encoder isn't available (checked via `ffmpeg`'s exit code, not by probing for the encoder up front), rather than requiring per-machine `ffmpeg` setup. Both codecs in the fallback list are equally lossless — the fallback only trades away the inter-frame compression advantage, never fidelity.

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
dump <space> <addr> <len> <file>   write raw bytes to Logs/<CoreName>/<file>
load <space> <addr> <file>         write Logs/<CoreName>/<file>'s raw bytes into <space> starting at <addr>
```

Both always resolve `<file>` under `Logs/<CoreName>/` (via `IDebugTarget.CoreName`, §3.1) — the same core-scoped directory F3 screenshots and their companion `.txt` files already use, so debug artifacts from a session end up in one predictable, per-core place rather than scattered relative to wherever the process happened to be launched from, or mixed in with a different core's output once a second core exists.

**Same `HasSideEffects` guard as `search`/`snapshot`** on `dump` — refuses a bulk read over `CpuBus` or any other space where reading can disturb live hardware state. `load` has no equivalent read-side concern but does check `IsWritable`, refusing to write into a read-only space.

**No format, no header — just the bytes.** `dump`'s output is exactly `len` raw bytes starting at `addr`; `load` writes exactly however many bytes the file contains, starting at `addr`, with no length argument of its own (the file's own size *is* the length). This keeps both directly interoperable with any external tool that reads/writes plain binary — a hex editor, `xxd`, a Python script — without EmuSen needing to define or parse its own container format.

**Distinct from save states.** `EmulatorSession`/`VenusCore`'s `SaveState`/`LoadState` (F5/F9 in the frontend) capture *everything* needed to resume execution — CPU, PPU, full bus state — as one opaque blob. `dump`/`load` capture *one named memory space* as plain bytes, readable and editable by anything, with no claim to being a complete or resumable snapshot of emulation state. Different tools for different jobs: F5/F9 for "come back to this later," `dump`/`load` for "take this data somewhere else and look at it."

### 3.12 "Who calls this address" (`callers`, `Debug/Commands/CallersCommand.cs`)

The static-analysis counterpart to a write watch: instead of running the game and waiting to observe an access, `callers` reads the code itself and reports every `JSR`/`JSL`/`JMP`/`JML` whose target matches a given address. Generalizes the exact manual step the Yoshi investigation used to find the real DMA-trigger dispatcher — grepping an already-captured CPU trace for `JSR`/`JSL` instructions targeting a known range (see `EmuSen_Core_Gameplan.md`'s current-status section) — into a real command that doesn't need a trace captured first.

```
callers <addr> [<scanstart> <scanlen>]   find JSR/JSL/JMP/JML targeting <addr>, scanning CpuBus
                                           (default: <addr>'s own bank, $8000-$FFFF)
```

Reuses `IDebugTarget.Disassemble` rather than re-decoding opcodes itself, so a `callers` result and the equivalent `disasm` output for the same address always agree — same disassembler, same bytes. Matches on the **raw opcode byte**, not the mnemonic: `JMP $nnnn` (absolute, direct — opcode `$4C`), `JMP ($nnnn)` (indirect — `$6C`), and `JMP ($nnnn,X)` (indexed indirect — `$7C`) all disassemble to the mnemonic `"JMP"` with the same 3-byte length, but only the direct form has a target `callers` can know without actually running the code. Indirect forms are deliberately excluded rather than guessed at.

**Same "best-effort, may misalign through data mixed with code" caveat as `disasm`.** A linear disassembler walking forward byte-by-byte has no way to know which bytes in a scanned range are really instructions versus embedded data (graphics, tables, text) — if the scan range includes non-code bytes, everything after the first misaligned read can decode to garbage opcodes, including spurious `callers` matches or missed real ones. Best used on a range that's actually known to be code.

**`writers`/`readers`** (`Debug/Commands/WritersCommand.cs`/`ReadersCommand.cs`) are the store/load-side counterparts to `callers` - "what code is capable of writing/reading this address," independent of whether that path was ever actually exercised in a traced run (the gap `writers` was built to close: watching an address live can show exactly one write from one PC and nothing else, which only proves what a specific run did, not what the ROM's code is capable of doing). Deliberately narrower in scope than `callers`: only `STA`/`STX`/`STY`/`STZ` (for `writers`) or `LDA`/`LDX`/`LDY` (for `readers`) in **absolute or absolute-long** addressing are matched - direct-page, indexed, and indirect forms are excluded outright rather than guessed at, since their real target depends on runtime register/D-register state a static scan can't know. Same bank-assumed-equals-PB convention as `callers`' indirect-`JMP` exclusion.

**`watch summary <id>` (§3.5) is the dynamic complement to `writers`/`readers`, not a replacement.** It only reports what actually executed during a traced run, but unlike a static scan it doesn't care what addressing mode got it there - an indexed `LDA addr,X` or an indirect `STA (dp),Y` shows up in a summary exactly like an absolute one would, since it operates on the resolved runtime address rather than parsing the instruction's operand form. Neither subsumes the other: `writers`/`readers` finds code paths that exist but may never have been reached yet; `watch summary` finds exactly what a specific run actually did, including forms `writers`/`readers` structurally can't see.

### 3.13 Frame-scoped value logging (`framelog`, `Debug/FrameLogRegistry.cs`, `Debug/Commands/FrameLogCommand.cs`)

The complement to `watch` for values that don't reliably *trigger* an access-based watch — a counter written once at level start and only ever read afterward would show exactly one write event forever; `framelog` instead samples a value **once per frame, unconditionally**, so its evolution over time is visible even when nothing about how it's touched would make a good watch.

```
framelog add <space> <addr> [<width>]   register a per-frame value sample (width 1/2/4 bytes, default 1)
framelog list                           list active frame logs with their IDs
framelog show <id> [<count>]            show a frame log's recorded (frame, value) samples (default 20)
framelog clear <id>                     clear a frame log's stored samples (doesn't remove it)
framelog remove <id>                    remove a frame log entirely
```

**Fed from a new per-frame hook, not the read/write observer hooks §3.5 already uses.** `MemoryBus.FrameObserver` (`IFrameObserver`, mirroring `IWriteObserver`/`IReadObserver`) is notified once per frame from `VenusCore.RunFrame`, right after `TotalFrames`/`Bus.FrameCount` are updated — the same moment `IDebugTarget.FrameCount` (§3.1) reports elsewhere, so a frame log's frame numbers line up exactly with a screenshot's or a save state's. `SnesDebugTarget` implements `IFrameObserver` the same way it implements the other two observer interfaces, and owns the `FrameLogRegistry` instance the same way it owns `WatchRegistry`.

**Deliberately does NOT print live**, unlike `watch`. A watch fires on a comparatively rare event, so printing every match keeps the "play, then grep the console log" workflow useful; a frame log fires 60 times a second by design, and printing every sample would just flood the console. Samples are stored silently (capped at 3600 per entry — about a minute at 60fps) and pulled on demand with `framelog show`.

**Example — watching a WRAM counter's value across a whole play session** instead of eyeballing it in `mem` one frame at a time:
```
framelog add WRAM 1234 2
```
Then, any time later, `framelog show 1` lists however many of the most recent per-frame samples are still in the buffer.

### 3.14 Cheat engine (`cheat`, `Debug/CheatRegistry.cs`, `Debug/Cheats/{ActionReplayCodec,GameGenieCodec}.cs`, `Debug/Commands/CheatCommand.cs`)

Two fundamentally different mechanisms era cheat devices used, unified behind one `CheatRegistry` (one ID space, one `list`/`enable`/`disable`/`remove` UX) since a player thinks of both as just "my cheats":

- **RAM pokes** (`CheatKind.RamPoke`) - the **Pro Action Replay**/**Game Wizard** (both Datel) mechanism: rewrite one fixed CPU-bus address to one fixed byte every frame, cheaply overpowering whatever the game itself writes there. A "999 lives" code is nothing more than "make this address always read back as 0x09" - no ROM patching, no read interception, just a write that keeps winning.
- **ROM patches** (`CheatKind.RomPatch`) - the **Game Genie** mechanism: substitutes the byte a specific *cartridge* read returns, optionally only while the real byte there matches a compare value. The underlying ROM is never touched - the substitution happens at read time.

```
cheat add <code> [description]              decode a code, guessing whether it's Pro Action
                                             Replay/Game Wizard or Game Genie from its
                                             formatting (see `gg` if it guesses wrong)
cheat poke <space> <addr> <value> [desc]    add a raw RAM-poke cheat directly, bypassing
                                             code decoding (e.g. an address already found
                                             with `search`)
cheat gg <code> [description]               decode a real SNES Game Genie code (8
                                             characters) and add it as a ROM patch, enabled
cheat rompatch <addr> <value> [<cmp>|-] [desc]
                                             add a Game Genie-style ROM-read intercept
                                             directly, bypassing code decoding; <cmp> gates
                                             it to only apply while the real byte there
                                             equals it (- = unconditional)
cheat list                                  list every cheat with its ID, kind, and state
cheat enable <id> / cheat disable <id>      toggle a cheat without removing it
cheat remove <id>                           remove a cheat entirely
cheat clear                                 remove every cheat
```

**SNES Game Genie is a real two-stage transposition cipher, not a straight hex substitution** (`GameGenieCodec.cs`). Each of a code's 8 characters decodes to a scrambled 4-bit nibble via a SNES-specific 16-character alphabet (`DF4709156BC8A23E` - different from NES/Genesis/Game Boy's own Game Genie alphabets); those 32 scrambled bits then get reassembled into 8 output nibbles by pulling from specific, non-contiguous bit positions - a real cipher, not just a different letter-to-hex mapping. Unlike NES/Genesis Game Genie, **real SNES codes never carry a compare byte** - Galoob's SNES device patches ROM instructions directly using 3 full address bytes instead of 2, so it never needed one; every `cheat gg`-decoded patch is unconditional (`Compare = null`).

**Verified before writing, not implemented from memory.** Getting this cipher subtly wrong would silently corrupt whatever ROM byte a mistranslated address happened to land on - worse than not having the feature at all, since it would look like it worked. The bit-offset logic was checked against an independently-published, MIT-licensed reference implementation by hand-tracing a full encode→decode round-trip, then fuzz-verified by replicating both the reference algorithm and this project's port in a throwaway script and round-tripping 2000 random (address, value) pairs through both - all 2000 passed and both implementations agreed on every one, before any of this landed in the actual codebase.

**`cheat add`'s format guess is a heuristic, not a guarantee - and it can't be otherwise.** The original plan for a unified `add` assumed Game Genie codes were letters-only and Pro Action Replay codes were plain hex, the way it works on NES - but SNES Game Genie's own alphabet (`DF4709156BC8A23E`, verified above) turned out to be a scrambled *ordering* of the same `0-9A-F` characters `ActionReplayCodec` already accepts, not a distinct letter set. Both formats decode from identical 8-hex-digit input, so there is genuinely no way to tell them apart from content alone. `CheatCommand.LooksLikeGameGenieFormat` falls back to how each device's codes are conventionally *published* instead: a separator right after the 4th character (`XXXX-XXXX`) reads as Game Genie; anything else (no separator, or one after the 6th character, `AAAAAA-VV`) reads as Pro Action Replay/Game Wizard. This matches real-world cheat lists' own formatting most of the time, but it's a guess - `cheat gg`/`cheat poke` are the reliable, unambiguous fallback whenever a code is copied without its original punctuation or formatted unconventionally.

**Pro Action Replay/Game Wizard code format.** `ActionReplayCodec` decodes the plain 8-hex-digit wire format both devices publish - a 24-bit CPU-bus address (bank + 16-bit offset) followed by the one byte to hold there, e.g. `7E01F663` means "poke CPU address `$7E:01F6` to `0x63`". No cipher, unlike Game Genie. Non-hex separators (`7E01F6:63`, `7E01F6-63`) are stripped before decoding, since that's how some published code lists format them.

**RAM pokes target `CpuBus`, not a raw WRAM array offset.** A decoded code (or a manual `cheat poke` against `CpuBus`) resolves through `MemoryBus.Write8`, so address mirroring (WRAM's `$7E`/`$7F` pair, the `$00-$3F` low-page mirror, etc.) behaves the same way it would on real hardware. `cheat poke` can still target any other named space (`WRAM`, `SRAM`, ...) directly when that's genuinely what's wanted.

**RAM pokes are fed from the same per-frame hook as `framelog` (§3.13), not a new one.** `CheatRegistry.ApplyAll` runs inside `SnesDebugTarget.OnFrame` (`IFrameObserver`), right alongside `FrameLogRegistry.RecordFrame` - re-writing every *enabled* `RamPoke` cheat's byte once per frame, resolving the target space the same `GetMemorySpaces()` lookup `framelog`'s per-frame callback already uses.

**ROM patches use a different, new hook - `IRomReadPatcher` (`Cores/Nintendo/Venus - SNES/Memory/IRomReadPatcher.cs`, see `Venus_Memory.md` §6).** Unlike every other observer hook `MemoryBus` calls (`WriteObserver`/`ReadObserver`/`FrameObserver`, all pure notifications), this one can override what the CPU actually reads. `MemoryBus.ReadInternal` consults it exactly where it falls through to `_cartridge.Read8(address)` - the one place that ever reaches the cartridge at all, matching a real Game Genie device's own physical placement on the cartridge edge connector. `SnesDebugTarget.TryPatch` forwards to `CheatRegistry.TryPatchRom`, which only ever matches `RomPatch`-kind entries, checking address and (if present) the compare byte against what the cartridge itself returned.

**Works without the debug toolchain otherwise being used.** `IDebugTarget.Cheats` exposes the same `CheatRegistry` instance `SnesDebugTarget` applies every frame (for `RamPoke`) and consults every cartridge read (for `RomPatch`) - unlike `Watches`/`FrameLog`, which the core only *feeds*, `Cheats` acts *on* the core's own memory/reads, so a cheat someone added and then never touched again keeps working for the rest of the session with no further F4 interaction required.

Core-agnostic on purpose, same as `WatchRegistry`/`FrameLogRegistry` - lives under `Debug/`, not `Cores/Nintendo/Venus - SNES/`. A future core's `IDebugTarget` implementation owns its own `CheatRegistry` instance, calls `ApplyAll` from its own per-frame hook, and implements its own core's `IRomReadPatcher`-equivalent forwarding to `TryPatchRom`.

**Example — a simple infinite-lives-style RAM poke**, assuming a hypothetical lives counter at WRAM `$7E:0DC0`:
```
cheat poke WRAM DC0 9 infinite lives
```
Or, given a real published Pro Action Replay code for the same effect:
```
cheat add 7E0DC009 infinite lives
```
No separator at all, so `cheat add` guesses Pro Action Replay/Game Wizard correctly and routes to `ActionReplayCodec` automatically - both add an enabled cheat; `cheat list` shows it as `#1: [on ] RAM  WRAM 0xDC0 = 0x9  infinite lives` (or `CpuBus` for the decoded version).

**Example — a manual Game Genie-equivalent ROM patch**, substituting a specific ROM byte only when it still holds its original value:
```
cheat rompatch 8091F2 A9 EA looks-safer-if-guarded
```
`cheat list` shows it as `#2: [on ] ROM  0x8091F2 = 0xA9 if==0xEA  looks-safer-if-guarded`.

**Example — decoding a Game Genie-format code** (illustrative, not a verified real published code - swap in an actual one from a game-specific cheat list):
```
cheat add DF47-0915 example code
```
The dash lands right after the 4th character, so `cheat add` guesses Game Genie and routes to `GameGenieCodec` automatically - equivalent to `cheat gg DF47-0915 example code` directly. Decodes through the cipher above into a raw ROM address/value pair, then adds it exactly like `rompatch` would (unconditional, since real SNES codes never carry a compare) - `cheat list` shows it the same way, `#3: [on ] ROM  0x... = 0x...  example code`.

### 3.15 `EmuSen.HeadlessDebug` — scripted CLI harness

A third consumer of `DebugCommandProcessor` alongside the console's F4 prompt (§3.3) and F1 hotkey — no window, no real-time input, built specifically for an AI agent (or any non-interactive caller) to drive an emulation session, script a repro, and read back structured results in one shot instead of needing a human at a keyboard. This has become the primary tool for investigating anything that needs precise, repeatable setup (a specific save state, a specific input sequence, a specific frame to screenshot) — most of the LttP color-math/subscreen investigation and the Super Metroid boot-hang investigation (`Venus_APU.md` §2.7) were done entirely through this, not the interactive console.

```
dotnet run --project EmuSen.HeadlessDebug -- <rom> <frames> [options...]
```

| Option | Does |
|---|---|
| `--loadstate <path>` | Load a save state before running any frames |
| `--savestate <path>` | Save a state after the run completes |
| `--tap <frame>:<button>[:duration]` (repeatable) | Hold `<button>` from `<frame>` for `<duration>` frames (default 1) — scripted input, e.g. `--tap 0:Right:60` |
| `--screenshot <frame>:<path>` (repeatable) | Write an uncompressed BMP of the frame buffer at `<frame>` (no PNG library available; convert externally if needed) |
| `--script <path>` | A text file of newline-separated `DebugCommandProcessor` commands (§3.3's table), run once after all frames finish. **Only the last `--script` wins** — passing it more than once silently drops the earlier ones rather than merging, since each flag just overwrites the same variable. Comment lines (`# ...`) and blank lines are skipped. If omitted, defaults to `watch list` + a `watch log` for every still-registered watch. |
| `--watch <space>:<addr>:<len>[:write\|read\|both]` (repeatable) | Register a watch **before** the run starts (unlike `watch add` inside `--script`, which only takes effect for whatever's left of the run *after* the script executes — since the script runs last). Two default watches are always active: `WRAM:0x8000:0x1800` and `WRAM:0xD80:0x80`, matching what the Raylib frontend registers from power-on. |
| `--cpulog <start>:<end>` | Enables `CpuVerboseLogging` (+ `MasterLoggingEnabled`) only for frames in `[start, end)` — avoids capturing a trace of the entire run when only a narrow window matters. |
| `--flag <Name>[=<value>]` (repeatable) | Set any `DebugSettings` property or field by name via reflection (bool if no `=value`, otherwise `Convert.ChangeType`'d to the member's real type) — also forces `MasterLoggingEnabled = true`, since most `DebugSettings` flags are gated behind it (`Settings/DebugSettings.cs`, §2) and setting the individual flag alone is otherwise a silent no-op. |
| `--verbose` | Shortcut for `MasterLoggingEnabled = true` for the whole run. |
| `--out <path>` | Write this harness's own `Emit()`-based log lines (`[ROM]`, `[RUN]`, `[SCREENSHOT]`, the `--script` output, etc.) to a file. |
| `--commands <path>` | Takes over the whole run - see below. Ignores `--tap`/`--tap2`/`--screenshot`/`--script` when present; `<frames>` becomes a hard safety cap instead of an exact count. |
| `--autoshot <dir>` | Works alongside every other mode (classic frame loop and `--commands` both). Hashes `GetFrameBufferRgba()` (64-bit FNV-1a) every frame and writes `<dir>/frame_<n>.bmp` only when it differs from the last saved hash - finds every distinct visual state a run passes through without having to guess frame numbers up front, e.g. confirming exactly when a menu transition settles instead of sampling `--screenshot 550`, `600`, `650`... and hoping one landed on it. |

**`--commands`: an interleaved script, for open-ended exploration `--tap`/`--screenshot` can't do.** Every other flag pre-declares its frame numbers before the run starts, which only works when the exact timing is already known. `--commands` reads an ordered text file and executes each line as it's reached instead, so frame-stepping, input, screenshots, and arbitrary debug commands can freely interleave in one process - built after the Super Mario All-Stars Select Game investigation needed a full relaunch (reboot + replay the whole boot sequence) for every single button guessed. Lines:
- `frames <n>` — advance `<n>` frames, applying whatever's currently held.
- `tap <button> [duration]` / `tap2 <button> [duration]` — press (P1/P2) for `duration` frames (default 4), then release. Inline equivalent of `--tap`/`--tap2`.
- `hold <button> [controller]` / `release <button> [controller]` — set a button's held state without advancing any frames (`controller` defaults to 1) - for holding something across several `frames`/other-command lines rather than one fixed-duration tap.
- `screenshot <path>` — capture the current frame right now, unlike `--screenshot`'s frame-number binding.
- `waitstable [maxframes=300] [quietframes=10]` — advance one frame at a time until the framebuffer hash stops changing for `quietframes` in a row (or `maxframes` is hit), removing the remaining "run N frames and hope it settled" guesswork plain `frames` still needs. Requires seeing at least one real change first - calling it while already sitting on a static screen (e.g. right after a `tap` whose transition hasn't started yet) would otherwise report "stable" instantly, since nothing was moving at the moment it was checked. A single multi-stage cutscene/menu sequence with several automatic pauses along the way needs one `waitstable` call per pause, not one call for the whole thing - confirmed against SMAS's own boot sequence, which turned out to need four separate real `Start` presses (each followed by its own `waitstable`) to reach Select Game, not the two originally assumed from an earlier, more manually-timed run.
- `contactsheet <path> <count> [every=1] [cols=8] [scale=4]` — capture `count` frames spaced `every` apart, downsample each by `scale` (nearest-neighbor - a debugging aid needs "did this move," not photographic fidelity), tile into one grid image. For confirming actual animation/movement across a span of frames without reviewing several separate screenshots one at a time.
- anything else — passed straight to `DebugCommandProcessor.Execute`, exactly like `--script`'s lines.

Example (the actual script that confirmed SMAS's Select Game screen needs Player 2's Start, in one run instead of six):
```
frames 120
tap Start 4
frames 180
tap Start 4
frames 300
screenshot select_game.bmp
tap2 Start 4
frames 250
screenshot file_select.bmp
```

**`--diffshot`: a standalone mode, unrelated to running a ROM.**
```
dotnet run --project EmuSen.HeadlessDebug -- --diffshot <bmp1> <bmp2> <outpath>
```
Reads back two of this harness's own BMPs (`--screenshot`/`--autoshot`/`contactsheet` output - not arbitrary external images, since it relies on the exact fixed 54-byte header `WriteBmp` always produces) and writes a third: every differing pixel highlighted in magenta over a dimmed/grayed copy of the second frame, so a change stands out at a glance instead of two screenshots held side by side. Also prints the changed-pixel count/percentage and bounding box to stdout. Dispatched before `Main` even looks at the usual `<rom> <frames>` positional arguments, since no core/ROM is involved at all.

**Two separate output streams, easy to conflate.** `--out` only captures this harness's own `Emit()` calls. Everything the *emulator itself* prints via raw `Console.WriteLine` — `CpuVerboseLogging`/`Spc700VerboseLogging` traces, `[DMA]`/`[PORT]`/etc. `DebugSettings` output, the `[FRAME] N` marker (`Venus_Memory.md`/`VenusCore.RunFrame`) — bypasses `Emit()` entirely and goes straight to real stdout. Redirecting shell output to `/dev/null` while relying on `--out` for everything discards all of that silently; capture real stdout to a file (`> file.log 2>&1`) instead whenever any `DebugSettings` trace flag is in play.

**`break add` inside a script only counts hits — it never halts.** Headless has no frontend loop to resume from a halt (§3.1's breakpoints note), so a breakpoint here answers "did execution ever reach this address, and how many times" (via `break list`'s hit count) rather than pausing anything.

### 3.16 `EmuSen.Validation` — ground-truth single-step validation

Not part of the `IDebugTarget` toolchain above (it doesn't run a full emulation session at all) — a separate, permanent harness that validates one CPU's opcode/addressing-mode/flag behavior in isolation against third-party ground-truth test vectors, independent of whatever a real ROM happens to exercise.

```
dotnet run --project EmuSen.Validation -- <target> <test-dir> [max-examples-per-file]
```

`<target>` is a registered `ISingleStepTarget` name (currently `65816` and `spc700`); `<test-dir>` holds the target's JSON test files (one per opcode, from [TomHarte/ProcessorTests](https://github.com/TomHarte/ProcessorTests) — not vendored into this repo, fetch separately). Each test sets up initial registers/memory, single-steps the real emulation code exactly once via the target's `ISingleStepTarget` adapter, and compares final registers/memory byte-for-byte against the vector's expected result.

**Why this matters more than it might look:** this is how the real, shipped 65816 `STA [dp]` addressing-mode bug (wired to the wrong function, breaking the ALTTP lamp/inventory bug it was chased down from) and two real SPC700 bugs (direct-page word wraparound, DAA/DAS high-byte-checked-post-adjustment — `Venus_APU.md` §2.4/§2.5) were all found and confirmed fixed. A ROM exercising the exact wrong opcode/addressing-mode/operand combination that exposes a bug like this is rare and easy to miss by playtesting alone (small movements, most values); 10,000 randomized cases per opcode is not.

**Adding a target is: implement `ISingleStepTarget` once** (`Reset`, `SetRegister`/`GetRegister`, `SetMemory`/`GetMemory`, `Step`), **write a small JSON-shape loader** for however that test suite's author formatted their vectors (see `Cpu65816SingleStepTarget.cs`/`Spc700SingleStepTarget.cs` and their paired loaders for the two existing examples — SPC700's loader differs from 65816's because TomHarte's two suites use different JSON field names for the same concepts), and register both in `Program.cs`'s `Targets` dictionary. No changes needed to the runner itself.

**A target whose chip has memory-mapped I/O needs to route `SetMemory`/`GetMemory` through real `Read8`/`Write8`, not a raw array poke** — `Spc700SingleStepTarget`'s own comment covers this in detail: a raw poke into `Ram[]` would silently miss `$00F2-$00F7` entirely (DSP register access, APU communication ports never touch `Ram[]` at all), producing spurious failures that look like emulation bugs but are really just test-setup gaps. Even with that in place, TomHarte's vectors model flat, uninstrumented RAM with no peripherals at all - any test case whose randomly-generated address happens to land on a *real* hardware register EmuSen correctly special-cases (SPC700's `$F0-F3`/`$FD-FF` timers/DSP-address port, not yet routed through this same seeding) will still show as a "failure" that's actually the test harness disagreeing with correct emulated hardware behavior, not a bug — see `Venus_APU.md` §2.6 for the full accounting of which failures are which, the last time this was run.

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

- **Breakpoints / single-step / pause-resume** — done (§3.1's breakpoints note, §3.3's `break` row). Still open: `EmuSen.HeadlessDebug` doesn't drive the halt/resume loop at all (its `break add` is hit-counting only, §3.15) — a scripted, non-interactive equivalent of `step`/`continue` would need the harness to check `IsHaltedAtBreakpoint` and decide what to do next, which nothing does today.
- **A verification pass on the disassembler** (§3.7) — it exists now, but hasn't had the equivalent scrutiny the execution opcode table got against oxyron.de. Worth a dedicated pass rather than trusting it blind.
- **Watchpoints beyond WRAM** — VRAM/CGRAM/OAM and the general CPU-bus/SRAM write paths don't report to `MemoryBus`'s `IWriteObserver` hook yet (§3.4).
- **The Avalonia GUI debug window** — the actual Mesen-style multi-pane debugger (register panels, hex viewer, disassembly view, sprite/palette viewers, event log, watch panel), built against `IDebugTarget` once the above exist. Everything in §3 was built with this as the eventual consumer, but it isn't built yet.
- **An NES (or other console) `IDebugTarget` implementation** — the interface was designed generically for this from the start, but no second implementation exists yet to prove it out.
- **A visual, burned-in-pixel screenshot timestamp** — deliberately deferred in favor of the companion-file approach (§3.6); revisit if the on-image version is still wanted.
