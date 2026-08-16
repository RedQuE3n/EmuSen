# EmuSen Debugging Tools — Reference

This document covers every debugging tool currently in the project: what it does, where it lives, how to trigger it, and how the pieces fit together. It's organized from "things you press a key for" down to "the underlying reusable toolchain," since that's roughly the order you'd reach for them in.

*(Living document — updated as the toolchain grows. **This revision: the test suite went from 6m01s to ~53s (§3.55, §3.56)** — and the lever was not threading. 69% of the whole suite was one class re-emulating the same frames: three theory methods each replayed every dump set from boot, so SMB3's 75 consecutive dumped frames cost 35,325 emulated frames where 194 would do. Sampling one replay instead took `ReferenceDumpTests` from 248.3s to 17.4s. Parallelism was then enabled — `AssemblyInfo.cs` had disabled it assembly-wide on the grounds that the suite was "small enough that parallelism wouldn't meaningfully speed it up", which was false in its reasoning and accidentally right in its conclusion — and it was worth about 3 seconds until the remaining long-pole class was split six ways, because xUnit parallelizes classes rather than test cases. Splitting it needed a production fix: Venus re-read the `--nobattery` switch on every save where Mercury and Moon already latch it at load, and latching it there too enforces the contract `CoreOptions` and `EmuSen_Multicore.md` §6 both already state. Turning it on immediately found two real defects: `CpuBinaryTrace` is one buffer for the whole process, so a `Reset()` under a running core crashes a test that never mentions it — and because every main 65816 sets `_traceBinary`, the exposure is every class that runs a Venus core, not the four that mention tracing. Also recorded: the first baseline was measured against a stale assembly and undercounted the suite by two tests. Previous revision: the trace differ is now generic, and the GSU has one (§3.41) — `TraceDiff` works over any `ITraceStep<T>`, so `CpuTraceDiff` and `GsuTraceDiff` are just record layouts and `--tracediff` picks the parser by magic. Pointed at Yoshi's Island's intro it found four real GSU bugs in an afternoon, two of them *flag* bugs on instructions whose results were already correct — the class of bug no state diff in this document can see. It also found that the harness's own battery save had been invalidating three days of state-anchored measurements, because on a SuperFX cart the `.srm` is GSU work RAM and a `tapuntil` anchor was firing at frame 0; `--nobattery` closes that. Previous revision: a fifth Mesen pass (§3.38), run over the two areas the previous four had least to say about: what the *hardware* does wrong, and the coprocessors as debuggable chips rather than as register dumps. Three things came out of it. **`bp when <condition>`** halts on a hardware event rather than an address — Mesen's `BreakSource` "exception" list is a good inventory of these and §3.33 dismissed the 65816-specific ones as NES/GB-only, which was simply wrong; `stp`, `wdm`, `ppuaccess` (VRAM/CGRAM/OAM written while the display is rendering) and `autojoy` ($4218-$421F read mid-refresh) are all cases where the machine and the game silently disagree and nothing in the output says so. Any of them can be armed `log` instead of halting, which is the right mode for anything a game does every frame. **The NEC DSP is now a real debug target** — `bp dsp`, `step dsp`, `bt dsp`, `cov dsp funcs` — closing the last "cannot be halted" entry in §3.34's table. Mesen has had this all along via `NecDspDebugger`, and the reason recorded here for not building it (mask ROM, stepped as a block) was not the real obstacle: the chip was already stepped one instruction at a time. Its hardware CALL/RET pair drives a genuine call stack, so firmware nobody has source for can be mapped by running it. And **`dma`** — the channel table plus a transfer log with per-destination bandwidth totals, which **neither emulator has as text**: Mesen shows DMA in the Event Viewer's 2D plot, the one thing §3.33 deferred for wanting a GUI, and the scanline column carries most of that plot's value without it. Previous revision: a fourth Mesen pass (§3.37), this time over the parts of Mesen outside `Core/Debugger/` — `DebuggerFeatures.h` and the `LuaApi` surface, which is a good inventory of what a debugger should be able to do because every entry is something someone wanted to script. It found that **the debugger could not write anything**: every one of the eight modelled processors' registers was read-only. `setreg` fixes that, and its point is counterfactuals — flip the register a branch compares, resume, and see what the game does down the path it did not take. Also new: `vectors` (dereference the interrupt vector table, grouped by CPU mode because a 65816 has two full sets and the E flag picks one, and marked `(ran)` when coverage says the target actually executed) and `addr` (decode a CPU address to a ROM file offset through the cartridge's own mapper). And **logpoints** — `bp add <addr> log "a, x"` records and carries on instead of halting, which is the only way to study a routine that runs four hundred times a frame, since halting even once changes the timing of everything after it. Mesen needs a Lua script for that; §3.33 declined to add a second scripting language because the shell already exists, and this is part of making that true. Previous revision: a third Mesen gap-analysis pass (§3.36), run specifically to audit what the first two waved through — and two rows of §3.33's comparison table turned out to be wrong rather than merely incomplete. `Breakpoint` was recorded as covered by `bp` (it was not: Mesen has address *ranges*, read breakpoints and forbid ranges; we had one address and writes only) and `CodeDataLogger` as covered by `cov` (it was not: Mesen persists a ROM map, enumerates discovered functions, and reports statistics). Closed this pass: `bp add`/`bp read`/`bp write` all take `<start>-<end>` ranges, `bp read`, `bp forbid`, `bp uninit` (§3.30 had the tally, not the halt), `cov funcs` and `cov save`/`cov load` (which merges rather than replaces, so maps accumulate across sessions). Three capabilities **neither emulator has**: `bp write ... changed` (halt only when the value actually changes, instead of on every one of sixty identical rewrites a second), `bp depth <n>` (catch runaway recursion while the chain is still readable with `bt`), and `cov mark`/`cov new` — differential coverage, the sharpest of the three, which turns "find the routine that handles X" into mark, press the button, read the list. Previous revision: a second Mesen gap-analysis pass, this time aimed at **coprocessors**. The old model of "which processor am I debugging" was a bool — main CPU or the one coprocessor — which cannot describe an SNES, since every SNES is at least two processors and the SPC700 present in every game had no breakpoints at all. `DebugCpu` (§3.34) replaces it with a named list, and one shared helper gives `bp`/`cov`/`bt`/`step`/`profile`/`eval`/`regs`/`disasm` an identical optional scope word, so `bp gsu add ...` and `bt sa1` work while every pre-existing unscoped form is untouched. New this pass: GSU and SPC700 breakpoints, SPC700 coverage, the SA-1's call stack (it is a 65816, so the existing seam applied unchanged), per-chip expression contexts derived from each chip's own `regs` output, `cpus`, and two disassemblers that were missing outright — `disasm APURAM` previously decoded SPC700 bytes with the 65816 table and produced confident nonsense. §3.35 adds `copflow`, which **Mesen has no equivalent of**: a log of the coprocessor register window plus poll-run detection, so "the game hung" becomes "it read $3030 ninety thousand times and the value never changed". Previous revision (retained below in §3.27–§3.33): the first Mesen pass, which closed seven capabilities — `eval` and conditional breakpoints, `bt`, `step over`/`step out`, `runto`, `label`, `counters`, `freeze` and `profile`. §3.33 records both comparisons, including what was deliberately **not** built (the Event Viewer, step-back, an assembler, disassembly search) and why.

Previous revision: every `IDianaOSCommand` now states `bool IsReadOnly { get; }` - true only if it never mutates core/session/interpreter state for any invocation (`mem`/`regs`/`echo`/`diff`/the shell text utilities and similar), false otherwise, conservatively including every command with a mixed read/write sub-verb surface (`watch`/`bp`/`framelog`/`cheat`/`snapshot`/`search`/`history`/`log`/`trace` - the property can't vary per invocation, so a command that's read-only for one sub-verb and mutating for another reports `false`) and `coretop`'s raw-terminal mode (blocks indefinitely on its own key-read loop until Ctrl+C, same category as `nano`, despite never touching core state). This is what powers `EmuSen.Hotaru`'s new **always-live** DianaOS terminal (see `EmuSen_Frontend_Driver.md`'s own top-of-file revision note for the full design) - from the moment a ROM loads, a `DianaOS $` prompt accepts commands continuously, with no F4 mode-switch needed: a read-only line answers instantly, off the emulation thread entirely, via `DianaOSInterpreter.TryGetReadOnlyFastPath` (a pure, parse-only classifier - single bare simple command, no pipes/`&&`/control-flow, resolves to a registered `IsReadOnly` command); anything else is queued and runs inline on the emulation thread's own next frame. F4/breakpoint halts are completely unchanged - still the fully-blocking classic prompt, still the only way to run a multi-line construct interactively.

Earlier: `EmuSen.Hotaru` moved off Raylib entirely, onto Avalonia (window, rendering, shaders, audio, input) - see `EmuSen_Frontend_Driver.md`'s own top-of-file revision note for the full picture. Relevant here: the on-window Raylib debug overlay (VRAM sheet/CGRAM swatch/register text drawn directly onto the game window, the old `O`/`GraphicsSettings.ShowDebugPanels` panels) is gone entirely - this doc's tools (`regs`/`sprites`/`pal`/`tile`/`vramsheet`/`paletteswatch`, `coretop`) already covered the same data and are unaffected. `coretop -w`/`feed -w` now open their windows via `Views/DebugWindows.cs` (a plain `Dispatcher.UIThread.Post`, since Avalonia is Hotaru's primary application now) instead of the old `AvaloniaHost.cs` (deleted - see git history), which had to bootstrap a second, independent Avalonia dispatcher thread specifically because Raylib owned the actual main thread and supported only one native window - see §3.17's updated `coretop`/`feed` writeups. F3's screenshot and F6's frame recording now encode real PNGs via SkiaSharp's `SKImage.Encode` directly (`EmuSen.Hotaru/Imaging/FrameImageWriter.cs`), not Raylib's `TakeScreenshot`, and capture the same raw pre-shader `ICore.GetFrameBufferRgba()` bytes every other consumer already uses.

*Earlier history, condensed (see `git log` for the full reasoning behind any of these):* a boxed MOTD-style welcome banner at shell launch (`GetWelcomeBanner`) · a standard `clear` command · `EmuSen.Hotaru` became a full DianaOS shell frontend (shell-first launch always; the always-on `[STATUS]`/`[FPS]` diagnostic prints were removed entirely - they're gone, not just currently unused; `feed`/`feed -w` added) · `coretop -w` (windowed dashboard, so it stops taking over the terminal) · `EmuSen.Mistress` gained a real windowed `coretop` (non-blocking, `WriteableBitmap`-based) · `coretop` itself added (htop-style hardware dashboard) · `nano <path>` and standalone-shell `core <name> <path>` added · `help`/`man` split apart again (`help` = standalone listing, `man [command]` = full manual pages) · the shell fully renamed to **DianaOS** (`EmuSen.Shell*` → `EmuSen.DianaOS*` everywhere, `ShellInterpreter`→`DianaOSInterpreter`, `IShellCommand`→`IDianaOSCommand`, `ShellResult`→`DianaOSResult`, `ShellSandbox`→`DianaOSSandbox`, `ShellConsoleWindow`→`DianaOSConsoleWindow`, `ShellDispatchTests`→`DianaOSDispatchTests` - see §3.3) · five plain project renames, no behavior change: `EmuSen.Validation`→`EmuSen.Tomoe` (the `EmuSen.Validation` *namespace* itself stayed - it's still live, generic single-step-test infrastructure, see `EmuSen/Validation/` and each core's own `Validation/` adapter folder; the `EmuSen.Tomoe` *project* has since been retired into `EmuSen.Pharaoh`'s `--singlestep` verb - §3.16), `EmuSen.HeadlessDebug`→`EmuSen.Pharaoh`, `EmuSen.TestingStudio`→`EmuSen.Mistress`, `EmuSen.RaylibFrontend`→`EmuSen.Hotaru`, `EmuSen.Presentation`→`EmuSen.Serenity` · `EmuSen.Pharaoh/Program.cs` broken up into `FrameRunner`/`HeadlessDebugOptions`/`CommandsScriptRunner`/`DiffShotRunner` plus shared imaging/audio helpers in `EmuSen/Common/` · `EmuSen.WiseMan` (§3.18) added as a committed xUnit project; real audio output added to `EmuSen.Mistress` · a core-agnosticism pass over `DianaOS/` (dead-code removal, `callers`/`writers`/`readers` dedup, `IDebugTarget.ClassifyStaticReference`/`DecodeTilePixels` added so those commands + `tile` stopped hardcoding 65816/SNES-specific knowledge) · `Debug.Commands`/`Debug/Commands/` merged into `DianaOS`/`DianaOS/`; `break` renamed to `bp` (shadowed by the shell's own loop-control keyword) · breakpoints/single-stepping docs corrected, `GetApuRegisters()` documented, command table filled in, `EmuSen.Pharaoh`/`EmuSen.Tomoe` given full write-ups · watchpoints, the generalized `tile` command, shared `FrameCount`/screenshot timestamping, and a full 65816 disassembler added (retiring `CoinTileDumpLogging`); `WatchRegistry` ownership moved from `MemoryBus` to `SnesDebugTarget` via the `IWriteObserver` hook.)*

---

## 1. Console hotkeys (`EmuSen.Hotaru/Views/GameWindow.axaml.cs`)

These all live in `GameWindow`'s per-frame hotkey dispatch, consumed once per frame inside the background emulation thread's `EmulationLoop`, after the frame is handed off for presentation — see `EmuSen_Frontend_Driver.md` §1/§2.

| Key | Does |
|---|---|
| **F1** | Full CPU + PPU state snapshot. Formatted to be directly comparable to MesenCE's Status panel. |
| **F2** | Dumps every active OAM sprite's X/Y/tile/attribute bytes — ground truth for sprite investigations. |
| **F3** | Saves the current frame as a PNG (`var/log/<CoreName>/screenshot_frame<N>.png`), plus a companion metadata file (`var/log/<CoreName>/screenshot_frame<N>.txt`) with the core name, frame number, and wall-clock time — see §3.6. |
| **F4** | Opens the interactive debug command prompt (see §3). |
| **F6** | Toggles a continuous frame recording on/off — same idea as F3 but for a whole span of frames instead of one instant. See §3.8. |
| **F5** | Save state. |
| **F9** | Load state. |
| **Tab** *(held)* | Fast-forward at `SpeedController.TurboPercent` (default 300%), with automatic frame skip and audio dropped — see `EmuSen_Rewind_And_FastForward.md` §2/§4. |
| **Backspace** *(held)* | Rewind — see `EmuSen_Rewind_And_FastForward.md` §1/§4. |
| **P** | Starts a 300-frame bounded scroll-write trace (`AllScrollWriteLogging`), tagged `[BG SCROLL]`. |
| **O** *(console debug view only, `Renderer.cs`)* | Legacy combined dump: OAM + a BG1 "black tile" diagnostic + backdrop compositing math. Left over from an early investigation; still assumes 8x8 tiles internally (not tile16-aware), so treat its BG1 output with that caveat if you ever reach for it again. |

**Tab and Backspace are the only *held* hotkeys here** — every other key in this table is edge-detected on `KeyDown` into a `volatile bool` request flag that the emulation loop consumes exactly once. Fast-forward and rewind instead track a key's held state across `KeyDown`/`KeyUp`.

**F1 and F4 both go through the shared debug toolchain** (§3) rather than having their own separate logic — F1 is just `debugTarget.GetSummaryText()` printed once; F4 is the interactive version of the same underlying data, plus more. F2 is intentionally *not* routed through the toolchain — see §3.4 for why.

---

## 2. `DebugSettings.cs` — every logging toggle

One central static class (`Settings/DebugSettings.cs`), grouped by which file each flag affects. Flip these instead of hunting through individual files. All of them are **on/off switches for logging that already runs continuously** — none of them change emulation behavior, only whether/what gets printed.

**`MasterLoggingEnabled` gates every `*Logging` flag below it, and defaults to `false`.** Each `*Logging` property's getter is actually `MasterLoggingEnabled && _theIndividualFlag` (`DebugSettings.cs:27` onward) - the "Default" column below is each flag's own individually-stored value, but **nothing logs out of the box** until `MasterLoggingEnabled` is also turned on, regardless of how many individual flags default to true. This was a deliberate fix: several individual flags below default to true (leftover from the investigations that added them), which meant a stock build used to log a steady stream of DMA/scroll/math-unit traces on every run whether anyone asked for it or not - flipping `MasterLoggingEnabled` off by default stopped that, while leaving every individual flag's own configured value intact underneath (turn `MasterLoggingEnabled` back on and whatever was individually enabled comes right back). `HvIrqEnabled`/`WindowingEnabled` are real feature switches, not logging, and are deliberately NOT gated by `MasterLoggingEnabled`.

| Flag | Default | What it logs |
|---|---|---|
| `CpuVerboseLogging` | off | Every executed instruction (PC, opcode, name, target address). Very high volume. |
| `CpuTraceCountdown` | 0 | Pairs with the flag above — set to a positive number to auto-stop the trace after that many instructions instead of running unbounded. |
| `HvIrqEnabled` | **on** | Master switch for H/V-IRQ support. A real, working feature (fixed the title-screen lockup) — not a "known broken" toggle, just a convenient full disable if a future regression needs isolating. Not gated by `MasterLoggingEnabled` - see above. |
| `DmaVerboseLogging` | **on** | Every general DMA transfer: channel, direction, source, destination, size. Decodes the destination VRAM address specifically for $2118/$2119 transfers. |
| `DmaSourceAddrLogging` | **on** | Logs which instruction wrote each DMA channel's source address. **Fixed since first added:** originally read the CPU's live `PC`/`PB`, which had already advanced past the responsible instruction by the time a write's side effect ran (caught mid-investigation when the same PC kept appearing for many different, unrelated results). Now reads `Cpu.LastInstructionPC`/`LastInstructionPB` — captured once per `Step()` before fetch/execute, so it stays accurate for that instruction's entire execution including any writes it triggers. |
| `WindowHdmaLogging` | off | HDMA writes/fetches for the window-position registers on channel 7 specifically. |
| `ColorMathBlendLogging` / `ColorMathBlendScanline` | off / `-1` | One target scanline's color-math blend: both operands going into a $2131 blend, added for the Zelda: A Link to the Past color-math investigation to see which operand was wrong without a full-frame dump. `ColorMathBlendScanline` picks the scanline (`-1` never matches, so this stays inert until both it and the logging flag are set). |
| `Spc700VerboseLogging` | off | Every executed SPC700 (audio CPU) instruction. |
| `DspKeyOnLogging` | off | Every KeyOn (note trigger): SRCN, resolved sample-directory entry, computed start/loop address, the BRR header byte found there, and the voice's pitch/volume - added to check whether the SPC700 sound driver triggers voices with sane sample pointers, independent of what `BrrDecoder`/`DspVoice` do with them afterward. |
| `CgWriteLogging` | off | Every CGRAM color write. |
| `Bg3ScrollWriteLogging` | off | Writes to BG3's scroll registers specifically. |
| `CameraRamLogging` | **on** | Writes to the zero-page RAM bytes the NMI routine stages scroll values in, right before flushing them to the PPU. Only logs on actual value change. |
| `RenderReadLogging` | **on** | What BG2's scroll values look like at the moment the renderer reads them for scanline 0 — compare against the last logged write to catch render-vs-write timing bugs. |
| `AllScrollWriteLogging` | **on** | Every write to all four backgrounds' scroll registers, with the shared latch value used in each calculation. |
| `HvIrqChangeLogging` | off | $4200/$4207-$420A writes, only when they actually change the H/V-IRQ enable bits or HTIME/VTIME. |
| `MathUnitLogging` | **on** | Every hardware multiply/divide operation, with operands and result. |
| `BgModeChangeLogging` | **on** | $2105 (BGMODE) writes, only on actual change — decoded into mode / BG3-priority / per-BG tile-size bits. |
| `MosaicWriteLogging` | **on** | Every $2106 (MOSAIC) write with the scanline it landed on — tells you whether a mid-frame write ever actually happens (the only case the mosaic starting-scanline latch changes behavior). |
| `WindowingEnabled` | **on** | Master switch for window masking. Re-enabled after being off during one earlier investigation — see the in-code comment if you're ever tempted to flip it off again; the masking logic itself is independently verified correct. Not gated by `MasterLoggingEnabled` - see above. |

**Retired:** `CoinTileDumpLogging` (used to ASCII-decode the tile at a hardcoded VRAM address every N frames) and `YoshiWramTraceLogging` (a hardcoded WRAM-range write trace) are both gone — superseded by the general `tile` command and the watchpoint mechanism respectively (§3.3, §3.5). Neither needed its own permanent flag once a general, on-demand equivalent existed; this is the intended direction going forward — a narrow, investigation-specific flag is a sign something belongs in the toolchain instead, not a permanent fixture.

To actually see any of this output, turn `MasterLoggingEnabled` on first (it's off by default - see above), then flip whichever individual flags matter; most default to a reasonable, low-volume state (change-triggered, not per-instruction/per-pixel) once the master switch is on, so it's fine to leave several active at once and grep the resulting log afterward for whatever tag matters.

---

### 2.1 `CategorizedLogWriter` — where the log lines actually go, and three measured freezes

*2026-08-16, from 182 lines across 13 blocks in `EmuSen/Common/CategorizedLogWriter.cs`. `EmuSen_Settings_Reference.md` §4.22 covers how Mistress installs it; this is the writer.*

**Why the split at all.** A single `console.log` routinely reached hundreds of thousands of lines in one play session — `WATCH`, `SCROLL` and `DMA-SRC` traces are each extremely high-volume — which made it slow to search and, in practice, too large to hand to anyone for review. Lines are routed by their bracketed prefix into the same CPU/PPU/APU/Memory split `Man pages/Hardware/` already uses, plus `debug` for the toolchain's own output and `general` for anything with no recognised tag (ROM load, cartridge info, the prompt itself). First match wins; every tag is a distinct literal, so ordering does not otherwise matter. Everything still reaches the real console too — only the file copy's destination changed.

**Three separate performance findings, each from a real session, each changing the design:**

1. **Per-line `AutoFlush` was a syscall per line on the emulation thread.** Every `Console.WriteLine` in the core's hot path — one per instruction under `CpuVerboseLogging`, one per write under `DmaVerboseLogging` — went straight into a `StreamWriter` with `AutoFlush = true`, on whichever thread drives emulation. The calling thread now only resolves a category (a prefix match against an in-memory table) and enqueues; a single consumer thread does the buffered I/O. `StreamWriter`'s own buffering is what makes that cheaper, which is why `AutoFlush` is deliberately **off** now.
2. **Synchronous console echo was the *next* bottleneck, and the assumption behind it was wrong.** Echo stayed on the calling thread on the reasoning that a terminal write is cheap relative to disk — true in normal play, false the moment `CpuVerboseLogging` and `Spc700VerboseLogging` are left on for a whole session. That is one write per instruction executed, hundreds of thousands to over a million a second, and a synchronous terminal write is far slower per call than an in-memory enqueue. **A real session showed FPS collapsing with both flags on.** Echo now goes through the same queue — `LogEntry.Target` is typed `TextWriter` rather than `StreamWriter` specifically so the console can share the path. What this does *not* remove is the cost of formatting a string for every instruction; that is inherent to logging at that granularity.
3. **Moving echo onto that thread then froze the emulator solid, which is worse than slow.** Within a handful of frames, with both verbose flags on. One worker thread now had to do buffered disk I/O (which keeps up) **and** terminal rendering (which fundamentally cannot, on any thread) serially, so the bounded queue filled and `BlockingCollection.Add` blocked the emulation thread — a freeze, not reduced FPS. The fix is that **`cpu` and `apu` specifically** suppress console echo, not "whatever is high-volume": `ppu`, `memory`, `debug` and `general` are proven fine by earlier sessions running clean with them always on, because nothing else in this codebase logs at every-instruction granularity.

**The bounded queue blocks rather than grows, deliberately.** Once full, `Add` blocks the producer — reintroducing exactly the blocking this class exists to avoid. That is accepted, because unbounded memory growth if disk I/O falls catastrophically behind is the worse failure. The cap was raised from its original 10,000 once every `Write`/`WriteLine` began enqueuing two entries (file plus echo) and verbose logging became something left on for a whole session rather than a short burst.

**`FlushIntervalMs` is damage control for ungraceful exits, not the durability guarantee.** Every category is flushed on a fixed timer — often enough that a hard kill, a native segfault, or a `Ctrl+C` skipping .NET's unwind loses at most a fraction of a second, rare enough to be nowhere near `AutoFlush`'s per-line cost. It exists because a real session lost a whole run's output: without a timer, only categories high-volume enough to fill their own internal buffer ever reached disk at all. `Dispose`'s drain-then-flush remains the real guarantee for a graceful shutdown, and the consumer thread is **background** so a caller that skips `Dispose` can never be the reason the process will not exit.

## 3. The reusable debug toolchain

This is the newer, generalized layer — built specifically so it isn't SNES-only, and so the same code can eventually back a GUI debug window, not just console printouts. **Standing policy going forward: if something built for one investigation seems logical and reusable, it goes in here rather than staying a one-off `DebugSettings` flag.**

### 3.1 `IDebugTarget` (`DianaOS/IDebugTarget.cs`)

The core-agnostic contract. Any emulated console implements this to plug into the rest of the toolchain. Deliberately modeled around generic concepts, not SNES-specific ones:

- **`GetMemorySpaces()`** — a list of named, sized, byte-addressable regions (`IDebugMemorySpace`: `Name`, `Size`, `IsWritable`, `HasSideEffects`, `Read(addr)`, `Write(addr, value)`). A generic memory viewer just lists whatever a target reports — SNES reports `CpuBus / IO / WRAM / VRAM / CGRAM / OAM / SRAM / APURAM`; an eventual NES target would report its own different set through the exact same shape. `HasSideEffects` (added alongside §3.9's `search` command, also used by §3.10's `snapshot`) flags spaces where `Read()` can do more than return a byte — SNES's `CpuBus` routes through live `MemoryBus.Read8`, which includes registers like RDNMI (clears the pending-NMI flag on read) or OPHCT/OPVCT (toggle a byte-order latch on read); `WRAM`/`VRAM`/`CGRAM`/`OAM` are plain arrays and always report `false`, and `SRAM` reports `false` too despite being bus-routed, since its address range never reaches an actual hardware register. Exists so a bulk, read-every-address tool can refuse to scan a space where doing so would silently perturb running emulation state.
- **`GetCpuRegisters()` / `GetVideoRegisters()`** — lists of `DebugRegisterValue` (name, value, bit-width for display formatting), not fixed struct fields — because different CPUs have wildly different register sets.
- **`GetSprites()`** — `DebugSpriteInfo` records (index, x, y, width, height, tile, palette, priority, flip flags) — generic enough to cover very different sprite hardware.
- **`GetPalettes()`** — `DebugPaletteInfo` records, colors *already converted* to display-ready RGB, so nothing downstream needs to know a console's native color format (SNES BGR555, etc.).
- **`Watches`** — the watchpoint registry (see §3.5). Exposed directly rather than re-wrapped into more `IDebugTarget` methods, since `WatchRegistry` is already core-agnostic on its own.
- **`FrameCount`** — a monotonic frame counter, owned as real state (not a frontend-local variable) so anything built against the interface can ask "what moment is this" consistently regardless of which core is running. Currently used to timestamp screenshots (§3.6); the same value is available to anything else that wants it.
- **`Disassemble(spaceName, address, count)`** — returns plain `DisassembledInstruction` records (address, raw bytes, mnemonic, formatted operand text). The interface knows nothing about a given CPU's addressing modes or operand-formatting conventions; all of that lives inside each core's implementation (§3.7). A target with no disassembler can legitimately return an empty list.
- **`ClassifyStaticReference(instr)`** — classifies whether a `DisassembledInstruction` statically references an address, returning `(StaticReferenceKind Kind, int Target)?` (`Kind` is `Call`/`Write`/`Read`) or `null` if the target isn't knowable from the bytes alone. Backs `callers`/`writers`/`readers` (§3.12) - added when those three commands turned out to be hardcoding a 65816 opcode table directly in the shell layer despite claiming to be core-agnostic; the ISA-specific knowledge now lives entirely in each core's own implementation, the same "core decodes, command scans/formats" split `DecodeTilemapEntry`/`DecodeTilePixels` below already use.
- **`DecodeTilePixels(space, address, bpp)`** — decodes one 8x8 tile's pixel-index grid (64 bytes, row-major, each a palette index) from raw bytes at `address` in `space`. Backs the `tile` command (§3.3's table) - same reasoning as `DecodeTilemapEntry` just above: an SNES tile is planar bitplane pairs, but nothing guarantees a future core's tile format looks anything like that, so the shell layer has no business assuming a shape here either. An unsupported `bpp` should throw a clear `ArgumentException` rather than guess.
- **`GetSummaryText()`** — a free-text escape hatch for whatever isn't (yet) modeled as structured data above.
- **`RenderTileSheet()` / `RenderPaletteSwatch()`** — plain `(byte[] Rgba, int Width, int Height)` image exports of tile/character memory and color palette memory, added for `EmuSen.Pharaoh`'s `vramsheet`/`paletteswatch` verbs (§3.15) so those images can be written to disk with no Raylib window at all. Not named "VRAM sheet"/"CGRAM swatch" at the interface level on purpose - an NES core's CHR pattern tables and its own palette RAM aren't the SNES's VRAM/CGRAM, but the shape (some tile memory decoded to a sheet image, some palette memory decoded to RGB swatches) is the same across consoles. A core with nothing analogous can return a `0x0` empty image.
- **`GetAudioSamples()`** — a non-destructive `(short[] Samples, int SampleRate)` snapshot of whatever's currently buffered for audio output, for the `audiodump` verb (§3.15). Non-destructive specifically so it never steals samples out from under a live audio-playback consumer of the same buffer - `Queue<short>.ToArray()` on the SNES side, not dequeuing. A core with no audio output modeled yet can return an empty array.
- **`GetHardwareLoad()` / `MaxSprites`** — added for the `coretop` command (§3.17). `GetHardwareLoad()` returns named `DebugLoadInfo` (`Name`, `Percent` 0-100) entries - "how hard is this piece of hardware working," normalized against whatever this core considers a full frame's wall-clock budget, so a generic dashboard just draws a bar without needing to know what unit any one core measures in. `MaxSprites` is a real fixed hardware capacity (128 total OAM entries on the SNES) rather than anything live-derived, letting a sprite count be shown as a genuine percentage-of-capacity instead of a bare, context-free number. Both are additive, "not modeled" (empty list / `0`) rather than a breaking change for any caller that doesn't wire them up - same pattern `GetApuRegisters()`/`GetAudioChannels()` already established.

**Breakpoints and mid-frame halt/resume now exist** (this used to be the "not included yet" item in this list, alongside disassembly - disassembly landed first, see §3.7). `IDebugTarget.Breakpoints` exposes a `BreakpointRegistry` (`bp add/list/remove` - §3.3's table), the same add/list/remove-only shape `Watches`/`FrameLog` already use; actually halting/resuming is NOT part of `IDebugTarget` itself (a registry edit doesn't know or care whether anything is currently running), it lives on `VenusCore.RunFrame()`: a breakpoint hit sets `IsHaltedAtBreakpoint`/`HaltedAddress` and returns immediately, mid-scanline if necessary, with all the scanline-loop's own state (`_scanlineStarted`, `_lineCycles`, etc.) left exactly as it was so the *next* `RunFrame()` call resumes the same in-progress frame instead of restarting it. The console's F4 prompt (`step`/`s`, `continue`/`c`) drives this by checking `IsHaltedAtBreakpoint` after each `RunFrame()` call and deciding whether to call it again immediately (continue) or wait for the next hotkey press (step) - see `EmuSen.Hotaru/Program.cs`. `EmuSen.Pharaoh` (§3.15) doesn't drive halting at all today; its `bp add` support is hit-counting only, useful for "did execution ever reach this address" questions in a scripted run without needing the halt/resume loop a live frontend provides.

### 3.1a The rules the member list above keeps restating

*2026-08-16. `IDebugTarget.cs` carried 162 lines across 23 blocks, most of them a per-member restatement of one of the four rules below. Stated once here, the file keeps one line per member.*

**The core decodes; the command formats.** `Disassemble`, `ClassifyStaticReference`, `DecodeTilemapEntry`, `DecodeTilePixels` and `TilemapEntryStride` all exist in the same shape for the same reason: the shell layer has no business assuming a hardware format. An SNES tile is `bpp/2` planar bitplane pairs and an SNES tilemap entry is one packed 16-bit word, but an NES core's nametable-plus-attribute-table split is not even the same *shape* — one byte per tile plus a coarser attribute byte covering a 2×2 block. So each core decides what one entry is and returns something already formatted, and the command walks the grid, matches the address or prints the text. The alternative — a generic bit-layout the interface parses itself — would encode one console's memory format into a core-agnostic contract.

**Returning `null` is how a core says "I cannot know", and it is never a guess.** `ClassifyStaticReference` returns `null` for any instruction whose target is not knowable from the bytes alone — a 65816 direct-page, indexed or indirect form, whose real address depends on runtime register or D-register state. Guessing would make `callers`/`writers`/`readers` quietly wrong rather than quietly incomplete, and incomplete is the recoverable failure. `Disassemble` follows the same principle in its formatting, and a target with no disassembler at all may legitimately return an empty list.

**A `null` registry means the feature is unavailable for this core, not that something is broken.** `CoprocessorBreakpoints`, `Coverage`, `CoprocessorCoverage`, `CallStack`, `Labels`, `AccessCounters`, `Freezes` and `Expressions` are all defaulted interface members. A core wires the seam or it does not, and the command reports the feature as unavailable rather than the interface growing a capability enum. **The coprocessor pair are separate from their main-CPU counterparts because the two run different code at the same addresses** — an SA-1 game's `$00:82D7` is not the S-CPU's `$00:82D7` (`Venus_SA1.md` §11.5).

**`Cheats` is the one registry whose data flows the other way.** Every other registry here is fed *by* the core — the memory bus reports writes into `Watches`, the instruction loop reports into `Breakpoints` and `Coverage`. `CheatRegistry` is read *by* the core, which re-applies every enabled cheat into its own memory spaces once per frame. It is on this interface, rather than behind the debug toolchain, precisely so a player who enables a cheat never has to open the F4 prompt for anything except adding it (§3.14).

**`HasSideEffects` exists for one caller and one refusal.** A space routed through a live hardware bus can have a real side effect on read — the SNES's `RDNMI` clears the pending-NMI flag, `OPHCT`/`OPVCT` toggle a byte-order latch — unlike a plain in-memory array. `search` (§3.9) reads every address in a space, so it needs to refuse a space where scanning would silently disturb running emulation rather than assume every `Read` is free. A core with no such space returns false everywhere. Muting via `SetChannelMuted` is the mirror-image case: the channel's playback and envelope state keep advancing exactly as real hardware muting does (`Venus_APU.md` §3.4), and only the final mix excludes it, so isolating a suspected-broken instrument needs no separate solo-rendering path.

### 3.2 `SnesDebugTarget` (`Cores/Nintendo/Venus - SNES/Debug/SnesDebugTarget.cs`)

The SNES implementation. Almost entirely a *reshaping* of already-existing, already-verified logic (the same OAM size/high-table decoding `DumpActiveOam` always used, the same register fields `StateDump` already read) into the structured shapes `IDebugTarget` asks for — not new emulation logic.

Two small helper classes back the memory spaces:
- `ByteArrayDebugMemorySpace` — wraps a `byte[]` directly (WRAM, VRAM, CGRAM, OAM, APURAM).

**`APURAM`** (added later) exposes the SPC700's own 64KB, so `mem`/`tile`/`search`/`snapshot`/`diff`/`watch` reach the sound driver exactly as they already reach the 65816's world. Before it existed the APU was a blind spot: an investigation could see the CPU↔APU mailbox (§3.2's `ApuRegisters`) and the SPC700's registers, but not a single byte of what the driver had actually loaded or was executing — checking the uploaded engine meant reading emulator source rather than the running machine. It deliberately wraps the raw `Spc700.Ram` array rather than routing through `Spc700.Read8`, for two reasons that matter: `Read8` consumes the timer counters at `$00FD-$00FF` (a debug read would silently perturb the driver's timing), and it returns the IPL ROM overlay for `$FFC0-$FFFF` rather than the RAM underneath (`Venus_APU.md` §1.1). Reading `APURAM FFC0` therefore shows what a game has actually stored beneath the overlay — which is precisely the evidence that distinguishes a correct overlay from the old "stamp the IPL into RAM" model that caused §2.7's Super Metroid boot hang.
- `BusDebugMemorySpace` — routes through `MemoryBus.Read8`/`Write8` at a fixed bank offset (used for the raw CpuBus space, for SRAM at its mapped CPU address `$70:0000`, and for the `IO` space at bank 0 - `watch add IO 4016 1`/`mem IO 4218 4` target hardware registers directly, added after a real investigation found `watch add` silently rejected them since nothing had registered that space name - see `Venus_Memory.md` §6/§7).

`SnesDebugTarget`'s constructor now also takes a `Renderer` (`SnesDebugTarget(Cpu, MemoryBus, Renderer)`, both call sites - `EmuSen.Hotaru/Program.cs` and `EmuSen.Pharaoh/Program.cs` - pass `core.Renderer!`) purely to back `RenderTileSheet()`/`RenderPaletteSwatch()` above, which delegate straight to `Renderer.GetVramTileSheetRgba()`/`GetPaletteSwatchRgba()`. Those two `Renderer` methods are themselves just `RenderVramSheet()`'s existing tile decode and `DrawDebugPanels`'s existing `SnesColor()` CGRAM conversion, reshaped into plain RGBA byte arrays instead of a Raylib `Color[]`/`DrawRectangle` calls - no new pixel-decoding logic, same "reshape what already exists" pattern the rest of this class follows.

`Watches` here just returns `SnesDebugTarget`'s own `WatchRegistry` instance. **This changed since first built:** the registry (and a `Cpu` back-reference, `DebugCpu`) used to live directly on `MemoryBus`, which mixed real emulation state with debug-toolchain plumbing in the same class — a coupling issue caught during a later architecture review. Now `MemoryBus` exposes only a tiny, debug-agnostic `IWriteObserver` hook (`Cores/Nintendo/Venus - SNES/Memory/IWriteObserver.cs`) that it calls on every write with no idea what's listening; `SnesDebugTarget` implements that interface, owns the `WatchRegistry` itself, and supplies the PC context from its own already-held `Cpu` reference. `MemoryBus` no longer references `Cpu` or the debug toolchain at all.

**`GetApuRegisters()`** (added investigating a Super Metroid boot hang - `Venus_APU.md` §2.7) exposes the SPC700's own A/X/Y/SP/PC/PSW plus both directions of the CPU↔APU communication ports (`InPort0-3` = what the CPU last wrote, `OutPort0-3` = what the SPC700 last wrote - see `Venus_APU.md` §1.2 for why those are two independent latches per port, not one). `regs` (§3.3's table) prints this as a third "APU registers" section whenever a target's list is non-empty; a core with no distinct sound co-processor just returns an empty list and `regs` skips the section. Before this existed, the only way to see SPC700 state at all was reading raw `Spc700VerboseLogging` trace text.

**`TilemapEntryStride`/`DecodeTilemapEntry`** back the `tilemap` command (§3.3's table) - added so a menu cursor's position or a HUD tile change can be confirmed by comparing tilemap entries as text instead of eyeballing screenshots. Deliberately NOT a generic bit-layout `IDebugTarget` parses itself: an NES core's nametable+attribute-table split isn't even the same shape as the SNES's single packed word (one byte per tile, plus a separate, coarser attribute byte covering a 2x2 tile block), so every core decides both how many bytes make up "one entry" (`TilemapEntryStride`) and how to render one (`DecodeTilemapEntry`) - same "core does the decoding, the command does the grid-walking" split as `Disassemble()`/`GetCpuRegisters()`. `SnesDebugTarget`'s implementation decodes the standard SNES BG screen word (2 bytes: bits 0-9 tile index, 10-12 palette, 13 priority, 14 h-flip, 15 v-flip) into a fixed 7-character label - 3 hex digits (tile index) + 1 digit (palette 0-7) + one character each for priority/h-flip/v-flip (`P`/`H`/`V` if set, `.` if not) - e.g. `1A32PHV` = tile `$1A3`, palette 2, priority set, both flips set; `1A32...` = same tile/palette with none of those three set.

### 3.2a What `SnesDebugTarget` decides that the interface does not

*2026-08-16, from 271 lines across 46 blocks in `SnesDebugTarget.cs`. The provider mechanism itself is `EmuSen_Cauldron.md` §2 and §5; these are the SNES-side choices.*

**`HasSideEffects` defaults to true for anything bus-routed, and SRAM is the one deliberate exception.** A `BusDebugMemorySpace` read can land on a live register — `RDNMI` clears the pending-NMI flag, `OPHCT`/`OPVCT` toggle a byte-order latch, the manual joypad port shifts on every read — so the conservative answer is the default and a caller must opt out. SRAM passes `false` because its range at `$70:0000+` only ever reaches inert cartridge SRAM even though it travels the same bus path (§3.1a).

**Providers are primed in the constructor, not left empty until the first `Refresh`.** Every WiseMan fixture constructs a target without a per-frame loop behind it, and a caller reading `Current` before anything has refreshed should see real state rather than "nothing published yet". The coprocessor history ring is sized at **600 refreshes — roughly ten seconds of frames** — because coprocessor bugs are *transitions*: the chip renders and then stops, and "what were SFR/PBR/R15 doing in the seconds before it stopped" is the question actually asked. `Current` alone can never answer it, and at that size the ring is not a memory cost worth thinking about.

**Frame timings arrive as an optional delegate rather than a constructor requirement.** `LastFrameCpuSpc700Ms` and friends live on the concrete core or session, not on anything this class already holds. A caller that wants real `HardwareLoad` numbers passes a reader; one that does not simply omits it and the section reports "not modeled", the same graceful degradation `ApuRegisters` already gives. The alternative would have every test fixture inventing plausible-looking stand-in numbers to satisfy a constructor.

**Write and read observation happen synchronously inside `MemoryBus`, which is what makes the PC context correct.** `OnWrite` is called from within `Write8`, so `_cpu.LastInstructionPC`/`PB` still name exactly the instruction that caused the write. That correctness property predates this class; what changed is that `MemoryBus` no longer needs a `Cpu` reference to get it. An access a **cartridge chip** made on its own is labelled with that chip's PC instead, because the S-CPU's says nothing about it (`Venus_SA1.md` §11.4). Breakpoints and coverage ride a matching per-instruction *pull* hook, costing one bool test while disarmed (§3.24).

**The spaces a cartridge adds are conditional on the cartridge actually carrying the chip**, because listing a space that cannot be read would break `spaces` and `FindSpace`'s "does this name exist" contract. `IO` is bank 0 only — registers mirror identically across every hardware bank, so `watch add IO 4016 1` means exactly the `$4016` a CPU trace already shows. `APURAM` is deliberately the **underlying array, not `Spc700.Read8`**: reading through the chip would consume timer counters and return the IPL overlay instead of the RAM beneath it (`Venus_APU.md` §1.1). `BW-RAM` needs its own space because `SRAM` is anchored at bank `$70` while an SA-1 cart maps BW-RAM at `$40`–`$4F`, so `mem SRAM` would read an unmapped bank and report 32 KB of zeroes (`Venus_SA1.md` §11.2). `GSURAM` is work RAM and framebuffer both, so `snapshot`/`diff` over it answers "is the chip still plotting" (`Venus_SuperFX.md` §5.1). The NEC DSP's RAM is `ushort[]` and is presented little-endian to match how its words reach the S-CPU over DR (`Venus_NecDSP.md` §6).

**A caveat on `Disassemble` that §3.7's own warnings do not cover.** The disassembler tracks `REP`/`SEP` as it walks, so a range crossing one decodes correctly on both sides. But M/X are properties of *a specific point in real control flow*, not global constants. Disassembling anywhere other than the CPU's current PC — the normal case, inspecting some other routine while paused elsewhere — means the flags in effect when that code actually runs may differ from the flags right now, and no static read-only disassembler can know that without tracing execution there or doing full control-flow analysis. **Treat immediate-operand widths away from the current PC as best-effort**; opcode and addressing-mode decoding are unaffected. Immediate widths also come from whichever CPU actually runs that space's code, not always the S-CPU (`Venus_SA1.md` §11.3).

**`ClassifyStaticReference` matches two addressing modes and excludes the rest on purpose** (§3.1a). *Absolute* (`JSR`/`JMP`/`STA`/`STX`/`STY`/`STZ`/`LDA`/`LDX`/`LDY $nnnn`) assumes the target bank equals the instruction's own — DBR-as-PB for loads and stores, PB-as-PB for jumps. That is **not always true at runtime**, since DBR can differ from PB; it is the same best-effort assumption this scan always made, stated rather than hidden. *Absolute long* (`JSL`/`STA`/`LDA $nnnnnn`) needs no assumption at all, and `LDX`/`LDY`/`STX`/`STY`/`STZ` have no long form so they only ever contribute the absolute case. Direct-page, indexed, indirect, stack-relative and immediate are excluded rather than guessed — including `JMP ($nnnn)` and `JMP ($nnnn,X)`, which share the mnemonic and length of the direct form but are opcodes `0x6C`/`0x7C` and are deliberately not matched.

### 3.3 `DianaOSInterpreter` (`EmuSen.DianaOS`, `DianaOS/*.cs`, `DianaOS/Commands/`) — user-facing and code name **DianaOS**

**Renamed to DianaOS, everywhere**: this shell is called **DianaOS** now, both to the person typing into it and in the code itself - a full rename, not just a display-name change (an earlier revision of this doc described a display-name-only version of this rename; that's since been superseded by this one). `Shell/` → `DianaOS/` (and `Shell/Commands/` → `DianaOS/Commands/`) as folder names, `EmuSen.Shell`/`EmuSen.Shell.Commands` → `EmuSen.DianaOS`/`EmuSen.DianaOS.Commands` as namespaces, and every "Shell"-prefixed type renamed to match: `ShellInterpreter` → `DianaOSInterpreter`, `IShellCommand` → `IDianaOSCommand`, `ShellResult` → `DianaOSResult`, `ShellSandbox` → `DianaOSSandbox`, `ShellConsoleWindow` → `DianaOSConsoleWindow` (`EmuSen.Mistress`), `ShellDispatchTests` → `DianaOSDispatchTests` (`EmuSen.WiseMan`). Types that only happened to live *inside* the (now renamed) folder/namespace without "Shell" in their own name - `Lexer`, `Parser`, `Ast`, `IDebugTarget`, `ConsoleLineReader`, `CommandHistory`, every individual `*Command` class, the various registries - keep their own names unchanged, same as `HeadlessDebugOptions` staying put through the `EmuSen.Pharaoh` rename noted at the top of this document: they describe what the code does, not the shell's own identity. User-facing surfaces match: the F4 console prompt (`EmuSen.Hotaru`), the Avalonia console window (`EmuSen.Mistress`'s `Settings > DianaOS Console...`), and `ConsoleLineReader`'s own comments all say "DianaOS", and the prompt itself reads `DianaOS #: ` (the bash-style secondary `> ` continuation prompt while a quote/`$(...)`/block is still open is unchanged - only the primary, "ready for a new command" prompt was renamed).

**Renamed and substantially rearchitected from `DebugCommandProcessor`** — what used to be a one-line-in/one-line-out dispatcher has grown into a genuine, if deliberately scoped-down, bash-alike shell, since the debug console's own composable-Unix-tools framing (`ls`/`xxd`/`objdump`) turned out to want the rest of what makes a *shell* useful too: variables, command substitution, real pipes, redirection to files, and if/for/while control flow, not just single dispatched commands. Lives in its own namespace (`EmuSen.DianaOS`, under a top-level `DianaOS/` folder) rather than `EmuSen.Debug`, since it's no longer debug-specific. **Update:** the SNES-facing debug commands (`mem`, `regs`, `watch`, ...) originally stayed behind in a separate `EmuSen.Debug.Commands` namespace/`Debug/Commands/` folder, "registered into the shell rather than merged away" (this doc's previous wording) — that split has since been undone. `IDebugTarget` and every registry it exposes (`WatchRegistry`, `BreakpointRegistry`, `CheatRegistry`, `FrameLogRegistry`), `DebugTools`, and `FrameRecorder` all moved from `EmuSen.Debug`/`Debug/` into `EmuSen.DianaOS`/`DianaOS/` alongside the commands that were already there, so the whole toolchain (interpreter, registry, and every command) now lives under one namespace and one folder. `EmuSen.Debug` still exists as a namespace, but only for `DebugSettings` (`Settings/DebugSettings.cs`, §2) — a logging-flag bag with no shell/debug-target coupling of its own, which is why it didn't move. Still takes a line of text and returns a line of text at the API surface frontends actually call (`Execute`/`Submit`) — doesn't know or care whether that text came from a console prompt, a headless script, or eventually a GUI debug window's command box.

**Architecture**: a small hand-written `Lexer` (quoting, `$VAR`/`${VAR}`/`$?`/`$(...)` expansion, operators) feeds a hand-written recursive-descent `Parser` (`DianaOS/Parser.cs`) producing an AST (`DianaOS/Ast.cs`: pipelines, and/or chains, if/for/while nodes), which `DianaOSInterpreter` walks to execute — the same hand-written-dispatch style this codebase already uses for the 65816/SPC700 CPUs (`Cpu.cs`, `Spc700.cs`) rather than a parser-generator dependency, applied to a much smaller grammar. Full writeup, including exactly what bash features are and aren't supported and why, is in §3.17 below - this section stays focused on the command registry itself.

**Every command implements `IDianaOSCommand`** (`Name`, `Usage`, `Execute(target, args, stdin)`) — a single interface unifying what used to be two separate ones (`IDebugCommand` for target-driven commands, `ITextFilterCommand` for pipe-only text tools): every command now gets both an ambient (possibly-null) `IDebugTarget` reference and an optional piped-in `stdin` string, and just ignores whichever it doesn't need (a debug command ignores `stdin`, a text tool like `sed` ignores `target`) - the same way real Unix commands are all just "a process with stdin/stdout/argv," whether or not they happen to use every one of those. `DianaOSInterpreter.CreateDefault(target)` builds the full standard registry (every class under `DianaOS/Commands/` plus this namespace's own shell builtins) in one call, so `EmuSen.Hotaru` and `EmuSen.Pharaoh` don't each need to re-list ~30 command classes and risk them drifting apart. `search`/`snapshot`'s per-session state (§3.9, §3.10) still lives as private fields directly on `SearchCommand`/shared via `SnapshotStore` between `SnapshotCommand`/`DiffCommand`, exactly as before this rename - only the interface and the class that owns dispatch changed.

| Command | Usage |
|---|---|
| `help` | List every command, one line each, with a brief explanation — see §3.17 |
| `man [command]` | With no argument, same as `help`; with a command name, that command's full manual page — see §3.17 |
| `spaces` | List memory spaces (name, size, writable) |
| `mem <space> <addr> [<len>]` | xxd-style hexdump, default length 16 |
| `write <space> <addr> <value>` | Write one byte, if the space is writable |
| `regs [<cpu>]` | CPU + video registers, plus APU and coprocessor sections when a target has them; with a chip name, just that chip (§3.2, §3.23, §3.34) |
| `cophist [<reg>] [<count>]` | Coprocessor register history, and how long it has sat unchanged — see §3.23a |
| `cpus` | Every processor a debug command can be scoped to, and what each supports — see §3.34 |
| `copflow on\|off\|clear\|tail\|stats\|poll` | Coprocessor register-window traffic, and stuck-handshake detection — see §3.35 |
| `sprites` | Active sprite/OBJ table |
| `pal [<index>]` | One palette, or all 16 if omitted |
| `channels` | Audio channel/voice table (active, envelope level 0-100, muted, core-specific detail) - core-agnostic (`IDebugTarget.GetAudioChannels()`), SNES reports its 8 S-DSP voices - see `Venus_APU.md` §3.5 |
| `mute <index> <on|off>` | Mute/unmute one audio channel for isolation testing - its own playback state still advances, just excluded from the final mix - see `Venus_APU.md` §3.5 |
| `tile <space> <addr> <bpp>` | ASCII-decode one 8x8 tile from any space (bpp 2, 4, or 8) |
| `tilemap <space> <addr> <cols> <rows>` | Decode a grid of raw tilemap entries as text (tile index/palette/priority/flip on SNES) — core-agnostic at the command level, see §3.2's `TilemapEntryStride`/`DecodeTilemapEntry` note |
| `disasm <space>\|<cpu> [<addr>] [<count>]` | Disassemble `<count>` instructions (default 10); a chip name picks its own code space and ISA, and with no address starts where it is executing — see §3.7, §3.34 |
| `watch add <space> <addr> <len> [write\|read\|both]` | Register a watchpoint (default write-only) |
| `watch list` | List active watchpoints with their IDs |
| `watch log <id> [<count>]` | Show a watchpoint's recorded events (default 20) |
| `watch summary <id>` | Group a watchpoint's accesses by site (`Context`) with **whole-run** hit counts — the dynamic, addressing-mode-agnostic equivalent of `readers`/`writers` (§3.12). Site totals are kept outside the event ring, so unlike `watch log` this never omits a site the ring has evicted |
| `watch clear <id>` | Clear a watchpoint's stored events (keeps the watch registered) |
| `watch remove <id>` | Remove a watchpoint entirely |
| `bp [<cpu>] add <addr>[-<end>] [if <expr>]` / `bp list` / `bp on\|off <id>` / `bp remove <id>` | Manage execution breakpoints (a 24-bit CPU address, or a range — §3.36) — see §3.1's breakpoints note. `if <expr>` makes one conditional (§3.27). Named `bp`, not `break` - the shell's own `break`/`continue` loop-control keywords (§3.17) are hardcoded, zero-argument parser statements, so a command literally named `break` is unreachable (`break add 8000` parses as bare loop-control followed by a syntax error on the leftover `add 8000`) |
| `bp write <space> <addr>[-<end>] [<value>] [changed] [if <expr>]` | Halt the moment anything writes `<addr>` — the state-at-the-write counterpart to `watch`, see §3.26. `changed` suppresses the halt unless the byte actually changes (§3.36) |
| `bp read <space> <addr>[-<end>] [<value>] [if <expr>]` | Halt on a read instead — catches consumers reached through a computed pointer that `readers` cannot resolve statically, see §3.36 |
| `bp uninit <space>` | Halt the first time never-written memory is read back — see §3.36 |
| `bp depth <n>` | Halt if the call stack ever gets deeper than `<n>`, while the chain is still readable with `bt` — see §3.36 |
| `bp forbid <addr>-<end>` | Suppress every halt while the PC is inside a range — see §3.36 |
| `bp add <addr> log <expr>` / `bp log [<count>]` / `bp log clear` | Logpoint: record `<expr>` and carry on instead of halting, for routines that run too often to step — see §3.37 |
| `eval [<cpu>] <expr>` / `eval symbols [<filter>]` | Evaluate an expression against live state, decimal/hex/binary at once; a chip name evaluates against its registers — see §3.27, §3.34 |
| `step [<cpu>] [<count>\|over\|out]` | Single-step, run a call to completion, or run until the current routine returns — see §3.28, §3.34 |
| `runto nmi\|irq\|brk\|cop` / `runto scanline <n>` / `runto frame [<n>]` / `runto <addr>` | Resume until a named event rather than an address — see §3.28 |
| `bt [<cpu>] [<count>\|reset]` | Backtrace the live call chain — the dynamic counterpart to `callers` (§3.12), see §3.28, §3.34 |
| `label add\|list\|at\|remove\|clear\|load\|save ...` | Name addresses; shown by `disasm`/`bt`/`bp list` and usable as expression symbols — see §3.29 |
| `counters on <space>` / `off` / `clear` / `<addr> [<len>]` / `top [r\|w\|x\|u] [<n>]` / `cold <addr> <len>` | Per-address read/write/execute tallies, including uninitialized-read detection — see §3.30 |
| `freeze add <space> <addr> [<value>]` / `list` / `remove <id>` / `clear` | Pin an address by undoing every write to it — see §3.31 |
| `profile [<cpu>] on\|off\|clear\|top [<n>]` | Which of the *game's* routines the instruction budget goes to — see §3.32, §3.34 |
| `cov [<cpu>] on\|off\|clear\|<addr> [<len>]` | Record which addresses actually executed, then ask whether a routine was ever reached — see §3.24, §3.34 |
| `cov mark` / `cov new <addr> [<len>]` | Differential coverage: freeze what has run, do the thing, then see only what the thing ran — see §3.36 |
| `cov funcs [<count>]` / `cov save\|load <file>` | Routines discovered by watching calls land, and a coverage map that persists and merges across sessions — see §3.36 |
| `setreg [<cpu>] <reg> <value>` | Write a CPU register — force a branch, skip a hang, test the counterfactual. See §3.37 |
| `vectors` | Dereference the interrupt vector table, resolved through labels and marked with what actually ran — see §3.37 |
| `addr <addr>` | Decode a CPU-bus address to a ROM file offset, RAM offset or hardware register — see §3.37 |
| `bp when [<condition>] [log\|off]` | Halt (or record) when the hardware does something it should not: `stp`, `wdm`, `brk`, `cop`, `ppuaccess`, `autojoy` — see §3.38 |
| `dma` / `dma log on\|off\|clear\|[<n>]` / `dma stats` | DMA/HDMA channel state, a transfer log with frame and scanline, and per-destination bandwidth — see §3.38 |
| `framelog add <space> <addr> [<width>]` / `list` / `show <id> [<count>]` / `clear <id>` / `remove <id>` | Per-frame value sampling, independent of reads/writes — see §3.13 |
| `callers <addr> [<scanstart> <scanlen>]` | Find instructions statically calling/jumping to `<addr>` — see §3.12 |
| `writers <addr> [<scanstart> <scanlen>]` | Find instructions statically writing to `<addr>` (absolute/absolute-long only on the SNES core — see §3.12 for why direct-page/indexed/indirect forms are excluded, and where that ISA-specific knowledge now lives) |
| `readers <addr> [<scanstart> <scanlen>]` | Find instructions statically reading `<addr>` (same scope as `writers`) |
| `dump <space> <addr> <len> <path>` / `load <space> <addr> <path>` | Save/restore a memory range to/from a file — see §3.11 |
| `search <space> <val> [<width>]` | Start a memory search — see §3.9 |
| `memfind <space> <bytes> [<max>]` | Find every offset where a byte *sequence* occurs (`??` wildcards) — see §3.25 |
| `search refine\|changed\|unchanged\|increased\|decreased\|list\|reset` | Narrow/inspect/clear the active search — see §3.9 |
| `snapshot <space> <name>` / `snapshot list\|remove <name>` | Capture/manage a named memory baseline — see §3.10 |
| `diff <name> [<count>]` | Compare a snapshot against current contents — see §3.10 |
| `trace <count>` / `trace off` | Arm/cancel a live CPU instruction trace |
| `cheat add\|poke\|gg\|rompatch\|list\|enable\|disable\|remove\|clear ...` | RAM-poke/ROM-patch cheat engine — see §3.14 |
| `echo <text...>` | Print `<text>` back, joined by spaces — see §3.17 |
| `sed s/pat/repl/[gi]` | Basic regex substitution, standalone or as a pipeline filter — see §3.17 |
| `grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]` | Filter lines by a regex pattern — see §3.17 |
| `wc [-l] [-w] [-c] [path]` | Count lines/words/bytes — see §3.17 |
| `sort [-n] [-r]` | Sort input lines — see §3.17 |
| `uniq [-c]` | Collapse adjacent duplicate lines — see §3.17 |
| `history` / `history clear` / `!N` / `!!` | List/clear/re-run past commands — see §3.17 |
| `true` / `false` | Always succeed/fail, no output — for `if`/`while` conditions, see §3.17 |
| `test EXPR` / `[ EXPR ]` | String/numeric/file-existence conditions for `if`/`while` — see §3.17 |
| `export NAME[=value]` / `unset NAME` | Set/remove a shell variable — see §3.17 |
| `source <path>` / `. <path>` | Run a script file's lines in this same shell's scope — see §3.17 |
| `if`/`then`/`elif`/`else`/`fi`, `for`/`in`/`do`/`done`, `while`\|`until`/`do`/`done`, `break`, `continue` | Control flow — see §3.17 |
| `summary` | Free-text fallback (delegates to `StateDump.DumpAll`) |

Addresses/values accept `0x`, `$`, or bare hex.

**`tile`** generalizes what used to be `DebugTools.DecodeTileAscii` plus `CoinTileDumpLogging`'s hardcoded call site — works against *any* memory space (not just VRAM) and supports 8bpp too (relevant since Mode 3/4's 8bpp BG1 and Direct Color are implemented, which didn't exist when the original helper was written). **Fully core-agnostic as of a later pass:** the bitplane decode itself used to be hardcoded directly in `TileCommand` (the exact same SNES planar layout `DecodeTileAscii` used, just moved rather than fixed) despite the command's own framing as the "core-agnostic, going-forward way to do this." Moved into `IDebugTarget.DecodeTilePixels(space, addr, bpp)` - same "core does the decoding, the command does the rendering" split `DecodeTilemapEntry`/`TilemapEntryStride` already established for `tilemap` (§3.2's `SnesDebugTarget` entry); `TileCommand` now just renders whatever pixel-index grid comes back as ASCII digits, with no idea what "bpp" even means for a given core's tile format. An unsupported `bpp` throws a clear `ArgumentException` from the target rather than `tile` guessing or silently producing garbage.

**Shared, stateless helpers** (`ParseHex`, `FindSpace`, `ReadValue`, `RequireTarget`) live in `DianaOS/Commands/DebugCommandHelpers.cs`, a static class in the same spirit as `DebugTools.cs` — used via `using static` in whichever commands need them, rather than duplicated per command or hung off `DianaOSInterpreter` itself. `RequireTarget` exists specifically because of the `IDebugTarget?`-is-nullable change above: every debug command that touches the target directly calls it first (`target = RequireTarget(target);`) so a shell running with no ROM loaded produces a clean "No ROM loaded" error instead of a `NullReferenceException`.

**Not interactive in the pause-emulation sense** — same reason as §3.1: each call to `Execute()` is one-shot, runs against current state, returns immediately. The console's F4 prompt just calls this in a loop. As of this doc's top note, `EmuSen.Hotaru` also calls it from its always-live `DianaOS $` terminal - either directly, off the emulation thread, for a command reporting `IsReadOnly` (see `IDianaOSCommand`'s own doc comment), or queued for the emulation thread's own next frame otherwise - but `Execute()`/`Submit()` themselves are unaware of any of that; they're still just "take a line, run it, return a result," the same as always.

### 3.3a `DianaOSInterpreterScheduler` — the four ways a line reaches the interpreter

*2026-08-16, moved out of `DianaOSInterpreterScheduler.cs` (57 lines across 6 blocks) and `GameWindow.axaml.cs`'s header. `EmuSen_Frontend_Driver.md` §1 and its top note describe what this does for Hotaru; this section is the class itself, because both frontends now depend on it and neither owns it.*

The "Diana always live" threading model started inside Hotaru's `GameWindow` and was lifted here so every host gets it rather than reimplementing it. **`EmuSen.Mistress`'s console window had none of it until that move**: it called `DianaOSInterpreter.Submit` straight from the UI thread with no synchronisation whatsoever against its own separately-running emulation thread.

One `lock` serialises every `Submit` this scheduler ever makes, by any path, because a `DianaOSInterpreter` has real mutable state beyond core reads — variables, history, `$?`. **It is never held across a blocking read**, so the thread that owns the core can never end up waiting on user input through this class. Four entry points, and which thread may call each is the whole contract:

| Entry point | Called from | What it does |
| --- | --- | --- |
| `SubmitFromAnyThread` | any thread | Classifies the line. A read-only fast path (§3.17) runs **immediately, on the calling thread**, because a snapshot read is safe from anywhere (`EmuSen_Cauldron.md`). Anything else is queued. Returns `null` when it queued rather than ran, so a caller can say "you will see this once the game ticks". |
| `DrainPending` | **only** the thread that owns the core | Runs everything queued since the last call, in submission order, one `Submit` per iteration rather than one lock across the batch — so a fast-path line from another thread can still interleave. A caller may `break` out early on a shutdown action; the rest stays queued. |
| `TakeHaltedLine` | the host's blocking prompt thread | Blocks until a line arrives while `Halted` is set. |
| `SubmitLocked` | a caller that already decided | Runs the line now, under the same lock, bypassing classification entirely. |

**`Halted` is a separate escape hatch, not a fifth state of the queue.** While it is set, `SubmitFromAnyThread` stops doing the fast-path/pending split and simply forwards every line into `TakeHaltedLine`'s queue, so a host's F4-style prompt loop pulls them one at a time on whichever thread is actually blocked — typically the thread that owns the core, which frees the UI and reader threads to stay responsive (`EmuSen_Frontend_Driver.md` §2). `Halted` is **per-scheduler, not per-host**, which is what a session switch mid-halt has to account for (`man tmux`, and `EmuSen_Frontend_Driver.md` §3f).

### 3.3b `HostAction` versus a constructor-injected delegate — which one a command needs

*2026-08-16, from `HostAction.cs` and `IDianaOSCommand.cs`. The rule was written down only in those two files, and it is the question every new command that touches the host has to answer.*

A command returns text. When it needs to do something to the *host* instead, there are exactly two mechanisms and they are not interchangeable:

- **A constructor-injected `Action`/`Func<T>`** — for anything that is purely a **side effect**: pausing a thread, opening a window, editing a file. `PauseCommand`, `CoretopCommand`'s `-w` opener and `NanoCommand` all take a plain delegate. This keeps `EmuSen.DianaOS` — a core-agnostic library with no UI toolkit dependency at all — from needing to know that Avalonia, or any windowing toolkit, exists. The frontend supplies the capability; the command never learns what backs it.
- **`HostAction`** — reserved strictly for changing the **caller's own control flow**: breaking out of a blocking prompt loop, or handing a value up several stack frames. `Resume`, `Step`, `Shutdown`, `LoadCore` and `SwitchSession`. A delegate cannot express these, because the thing that must change is the loop that called `Submit` in the first place.

The test is simply *whose control flow changes*. If the answer is "nobody's, something just happens", it is a delegate. `DianaOSResult.Action` being optional rather than the general return path is the same statement from the other side.

**A closed abstract record hierarchy, not an enum plus an object payload.** `LoadCore` carries two strings with real type safety at every consumption site instead of a cast, matching this codebase's existing preference for small explicit result shapes (`DebugLoadInfo`, `StaticReferenceKind`) over a stringly-typed grab bag. `LoadCore` carries the raw strings deliberately: validating them — unknown core name, missing file, wrong extension — is the *issuing* command's job, so by the time one reaches the driving loop both are already known-good (§3, `CoreCommand`).

**`IsReadOnly` is a per-class property and cannot vary by argument**, which is why a command with a mixed sub-verb surface (`watch add` against `watch list`) must report `false`. The cost of that conservatism is one frame of latency for its read-only sub-verbs (§3.17); the cost of getting it wrong is a mutation racing `RunFrame`. Note also that `stdin` distinguishes `null` from `""`: null means this command is standalone or the first stage of a pipeline, empty means a previous stage genuinely produced no output.

### 3.4 What's deliberately *not* routed through this yet

- **F2's sprite dump** still calls `renderer.DumpActiveOam` directly rather than the `sprites` command. That method has a second responsibility — it also returns rects consumed by the "O" key's overlay drawing — that `SnesDebugTarget.GetSprites()` intentionally doesn't take on, to keep the toolchain's data-producing role separate from the renderer's overlay-geometry role. The `sprites` command (via F4) is the generalized, going-forward equivalent of just F2's printed output.
- **`StateDump`, `DianaOS/DebugTools.cs`, `Renderer.Debug.cs`** (below) are all still called directly in a few places rather than fully absorbed into the new layer — they're lower-level building blocks the new layer sometimes wraps (`GetSummaryText()` → `StateDump.DumpAll`) rather than things it replaced outright.
- **The watch registry currently only hooks WRAM reads and writes.** `MemoryBus`'s `IWriteObserver`/`IReadObserver` hooks (§3.5) are only called from the WRAM read/write paths today — `Ppu`'s VRAM/CGRAM/OAM accesses and SRAM don't report to either yet. Built for what the Yoshi investigation needed, with wiring in the other paths as natural, additive follow-up whenever an investigation needs to watch one of them. **All three of WRAM's own write paths are covered, found the hard way, one bug at a time:** direct bank-$7E/$7F addressing (original), the WMDATA `$2180` port (missed initially — a game writing through it would have been invisible to every watch until this was caught), and the low-RAM `offset < 0x2000` mirror used by bank-$00-relative absolute addressing (also missed initially, caught while investigating why a WRAM job table the Yoshi investigation depends on never showed a single recorded write despite clearly holding real, varying data — see `EmuSen_Core_Gameplan.md`'s current-status section). The same three paths now also report reads via the mirrored `IReadObserver` hook, added alongside read watchpoints (§3.5).

### 3.5 `WatchRegistry` (`DianaOS/WatchRegistry.cs`) — watchpoints

The generalized version of the recurring "log every write to this address range" pattern this project kept building ad hoc (`CameraRamLogging`, `MosaicWriteLogging`, `DmaSourceAddrLogging`, and originally a one-off Yoshi-specific WRAM trace). Instead of adding a new `DebugSettings` flag and a hand-written range check in the emulation code every time an investigation needs one, register a watch through the toolchain instead.

Each watch has a `WatchKind` — `Write` (default), `Read`, or `Both` — mirroring GDB's `watch`/`rwatch`/`awatch`. Write-only was this mechanism's original (and still default) behavior, so every pre-existing `watch add` invocation keeps meaning exactly what it always meant.

- **`AddWatch(spaceName, address, length, kind = Write)` → id** — register a watch on a memory-space address range, optionally restricted to reads, writes, or both.
- **`RemoveWatch(id)`** — remove one.
- **`GetWatches()`** — list active watches, including each one's `Kind`.
- **`RecordWrite(spaceName, address, value, contextFactory)`** / **`RecordRead(spaceName, address, value, contextFactory)`** — called by whoever owns the registry (`SnesDebugTarget`, via its `IWriteObserver.OnWrite`/`IReadObserver.OnRead` implementations — see §3.2) in response to an access reported by the core it's watching. Both delegate to one shared private `Record(accessKind, ...)` so the matching/storage/printing logic exists exactly once. Cheap to call even with zero watches registered (a quick scan over however many are active) — this matters more for reads than writes, since a read happens on every instruction fetch and operand read that touches a watched space, not just the writes a game actually makes. When an access matches an active watch's kind, it's both **printed live** (`[WATCH #id] W ...` or `[WATCH #id] R ...` — so the existing "play, then grep the console log" workflow keeps working with zero changes) and **stored** in a bounded per-watch buffer (last 500 events), so it's also queryable later via `watch log` without needing a fresh log upload.
- **`GetEvents(id, maxCount)` / `ClearEvents(id)`** — read or clear a watch's stored buffer.

`DebugWatchEvent` carries a sequence number, an `AccessKind` (`Write`/`Read`), address, value, and a free-text `Context` string (typically `PC=0x00A358`) — kept as text rather than a structured field since what's useful context varies by core and shouldn't force an interface change every time a new kind becomes relevant.

Core-agnostic on purpose, same as `IDebugTarget` — lives under `DianaOS/`, not `Cores/Nintendo/Venus - SNES/`, since nothing about it is SNES-specific. `SnesDebugTarget` owns the instance and reports accesses to it via `MemoryBus`'s `IWriteObserver`/`IReadObserver` hooks (`Cores/Nintendo/Venus - SNES/Memory/IWriteObserver.cs`, `IReadObserver.cs`) — `MemoryBus` itself has no idea `WatchRegistry` exists. `IReadObserver` is a deliberately separate interface from `IWriteObserver` rather than an added method on it, so a future implementer that only cares about one kind of access can implement just the interface it needs; `SnesDebugTarget` implements both. A future NES `IDebugTarget` would own its own instance the same way, wired to its own core's equivalent observer hooks.

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

**Storing an event ignores the logging master switch; only the live echo respects it.** `watch log <id>` has to keep answering with logging silenced — the stored events are the point of the registry, not a side effect of printing. Only the console echo is gated on `DianaOSLogging.MasterEnabled`, which matches every `DebugSettings.*Logging` flag's behaviour (§2). Before that gate existed the echo printed unconditionally, so "turn all logging off" did not actually silence an active watch — the one trace in the project that ignored the switch.

**`RecordRead` is held to a stricter version of `RecordWrite`'s cheapness contract.** Reads happen far more often than writes — every instruction fetch and operand read, not just the writes a game makes — so the no-match path has to stay free. The linear scan over active watches costs the same either way; what is deliberately deferred to an actual match is the `contextFactory()` call and the event storage, which is why context arrives as a delegate rather than a formatted string.

**Per-site totals live outside the event ring** (`SiteHits`, `GetSiteHits`, `GetEventCounts`). The ring is bounded and evicts, so a site that stopped firing early can vanish from `watch log` entirely — and a truncated site list that *looks* complete is worse than an obviously partial one. The whole-run counts are kept separately and never evict, and `GetEventCounts` returns total against retained so the gap is visible rather than implied.

### 3.6 Screenshot timestamping (F3, `EmuSen.Hotaru/Program.cs`)

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

### 3.8 Frame recording (F6, `DianaOS/FrameRecorder.cs` + `EmuSen.Hotaru/Program.cs`)

Continuous version of F3's screenshot-plus-metadata pattern (§3.6): instead of one PNG for a single instant, captures a whole span of frames into a session folder, with one ledger file (`frames.log`, tab-separated: `Frame`, `WallClock`, `Core`, `ImageFile`) mapping every captured image back to exactly when it happened — the same cross-reference purpose F3's companion `.txt` already served, spread across a sequence instead of one moment.

```
F6   toggle recording on/off
```

Starts a new session folder under `var/log/<CoreName>/Recordings/<CoreName>_<timestamp>/` each time it's turned on (the F6 hotkey passes `var/log/<CoreName>/Recordings` as the base directory; `FrameRecorder` itself still prefixes its own session folder name with the core name too, so it stays self-sufficient even for a caller that doesn't already organize by core); `[RECORD] Started -> <path>` / `[RECORD] Stopped: <path>` confirm state in the console log. Supports an optional frame stride (capture every Nth frame rather than every single one — a long recording at every frame gets large fast) via `FrameRecorder.Start(baseDir, frameStride)`; the F6 hotkey itself uses the default of every frame, intended for short, targeted captures around a specific moment (e.g. wrapping the whole span of a suspected rendering bug) rather than recording an entire play session.

**Core-agnostic by the same pattern as the rest of this toolchain:** `FrameRecorder` only touches `IDebugTarget` (`CoreName`/`FrameCount`, same as F3 uses via `debugTarget`) and never anything console- or renderer-specific. The one frontend-specific piece — actually grabbing a frame's pixels and writing an image — is supplied by the caller as a callback (`path => FrameImageWriter.SavePng(core.GetFrameBufferRgba(), core.ScreenWidth, core.ScreenHeight, path)` in `GameWindow`'s F6 handler, encoding via SkiaSharp's `SKImage.Encode` directly - see `EmuSen.Hotaru/Imaging/FrameImageWriter.cs`) rather than living inside `FrameRecorder` itself, so a future frontend using a different capture mechanism reuses this class unchanged and only needs to supply its own callback.

**Automatic video encode on stop, via `ffmpeg`.** A PNG-per-frame folder is a hot mess to actually review, so `Stop()` shells out to `ffmpeg` (if it's on `PATH`) to mux the just-captured sequence into `recording.mkv` in the same session folder, using `ffmpeg`'s glob image2 demuxer (`-pattern_type glob -i "frame_*.png"`, which only needs lexically-increasing filenames — already true since they're named from `FrameCount`, zero-padded — not strictly-sequential integers, so this works unchanged even with a frame stride > 1). Framerate passed to `ffmpeg` is `60.0 / frameStride`, the same "~60fps" approximation already used elsewhere in this codebase (see `Program.cs`'s `SaveEveryNFrames` comment), not a timing-accurate real-hardware rate.

**Codec choice: `libx264rgb` at CRF 0, deliberately lossless, not a lossy codec like VP9/regular H.264.** This exists to inspect exact pixel-level rendering bugs (the whole reason it exists — Yoshi's invisible-sprite investigation); a lossy codec's own compression artifacts would work against that exact purpose. `-crf 0` is mathematically lossless in x264 (bit-exact, not just "visually lossless"); `libx264rgb` specifically (not plain `libx264`, which defaults to chroma-subsampled `yuv420p` — NOT lossless even at CRF 0) keeps the whole encode in RGB, matching Raylib's screenshots directly with no colorspace conversion to introduce rounding.

**Originally FFV1-in-Matroska, switched after real recordings came back far larger than expected for a short capture.** FFV1 is also lossless, but it's intra-frame-only — every frame is encoded independently, so a mostly-static stretch of gameplay (or the debug side panels, which barely change frame to frame, since the whole window is captured — see the capture callback note above) costs full data on every single frame. `libx264` still does inter-frame prediction even at CRF 0, so it can actually exploit that redundancy instead of re-encoding a nearly-identical frame from scratch 60 times a second. Container stays Matroska (`.mkv`) either way.

**Loose PNGs get zipped and deleted after a successful encode, not left lying around.** Once `ffmpeg` confirms success (`ExitCode == 0`), the frame PNGs are fully redundant — the encode is lossless, so `recording.mkv` already contains everything they do — and a folder of potentially thousands of individual images is exactly the mess this feature exists to avoid. They're archived into `frames.zip` (same session folder) and the loose files deleted; `frames.log` (the small frame/timestamp ledger) is left alone regardless, since it's cheap to keep and useful without unzipping anything. If the *zip* step itself fails, the PNGs are left in place rather than risking data loss over a tidiness step — the video already succeeded either way. If `ffmpeg` itself fails, none of this cleanup runs at all; the loose PNGs are the only artifact and stay untouched (see below).

**Not required.** If `ffmpeg` isn't on `PATH`, `Stop()` catches that (`Win32Exception`) and just logs that the PNG sequence + ledger are the usable result, same as before this existed — this is a convenience layer, not a hard dependency of the recording feature itself.

**Two knobs worth knowing, both deliberately crude.** `Start(baseDir, frameStride)` captures every Nth rendered frame; the default of 1 suits a short targeted capture around one moment, but a long session at 60 fps produces PNGs fast, so a caller chasing something slower — a multi-second animation — should pass a stride rather than thin the output afterwards. And the encode assumes a flat `AssumedFps = 60.0`, not the SNES's real 60.0988 Hz: this is a convenience encode for looking at pixels, not a timing-accurate export, and it uses the same approximation the rest of the codebase already does for "about a second" arithmetic.

**Blocks synchronously while encoding.** `Stop()` waits for `ffmpeg` to finish before returning, so stopping a long recording will visibly freeze the emulator for the encode duration — same class of tradeoff the F4 debug prompt already has (the execution loop can't pause mid-frame independent of this either way). `-preset slow` favors compression ratio over encode speed; raise to `-preset veryslow` for a bit more, or drop to `-preset medium`/`fast` if the freeze itself becomes the bigger annoyance — encode time trades directly against file size here, same as any x264 preset. Would need to move off the main thread if this ever needs to not block real-time play at all.

**Verified against real runs (Fedora 44, ffmpeg 8.1.2), twice, two real bugs caught.** First: `outputPath` was passed to `ffmpeg` as the full `sessionDir/recording.mkv` path *while `ProcessStartInfo.WorkingDirectory` was already set to `sessionDir`*, so `ffmpeg` tried to resolve it relative to a directory it was already inside, landing on a doubly-nested path that doesn't exist. Fixed by passing just the bare filename. Second: switching to `libx264rgb` (see above) regressed recording entirely on a real machine whose `ffmpeg` build doesn't include `libx264` — a real, common situation, not hypothetical (`libx264` is patent-encumbered and left out of some distros' default `ffmpeg` package, e.g. plain Fedora repos as opposed to RPM Fusion's build; `ffv1` is unencumbered and always present). **Fix: `TryEncodeVideo` now tries `libx264rgb` first and automatically falls back to `ffv1`** if that specific encoder isn't available (checked via `ffmpeg`'s exit code, not by probing for the encoder up front), rather than requiring per-machine `ffmpeg` setup. Both codecs in the fallback list are equally lossless — the fallback only trades away the inter-frame compression advantage, never fidelity.

### 3.9 Memory search (`search`, `DianaOS/Commands/SearchCommand.cs`)

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

One active search session at a time — starting a new `search <space> <value>` replaces whatever was running, matching the command's own "first scan" framing. Session state (candidate address list, last-known values for the changed/unchanged/increased/decreased comparisons) lives as plain fields directly on `SearchCommand`, not its own class — there's genuinely only ever one session, so a richer structure wouldn't buy anything.

**Refuses to scan a space where `HasSideEffects` is true** (see §3.1) — SNES's `CpuBus` is the concrete case: a full-range scan would sequentially read every hardware register, including ones with real read side effects (RDNMI clearing the pending-NMI flag, the manual joypad port shifting its serial data on every read), silently corrupting whatever's actually running. There's no legitimate reason to bulk-search live registers anyway — real game state lives in `WRAM`/`SRAM`, both of which are safe (`HasSideEffects == false`) and where every realistic use of this command should be pointed.

**One real bug caught before this ever shipped, not after:** `ParseHex` returns a signed `int`, so a width-4 search value with the high bit set (e.g. `FFFFFFFF`) would parse as `-1` and never match `ReadValue`'s always-non-negative multi-byte accumulation. Both the initial-search and `refine` value parses mask with `& 0xFFFFFFFFL` to reinterpret the bit pattern as unsigned before comparing — caught during review, not by a failed run, so flagged here in case the same shape of bug (`ParseHex` feeding a signed value into an unsigned comparison) shows up again in a future command.

### 3.10 Memory snapshot/diff (`snapshot`, `diff`, `DianaOS/Commands/{SnapshotCommand,DiffCommand,SnapshotStore}.cs`)

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

### 3.11 Memory dump/load to file (`dump`, `load`, `DianaOS/Commands/{DumpCommand,LoadCommand}.cs`)

Takes a `snapshot`-style capture out of the process entirely, to disk, as raw bytes — for handing off to an external hex editor, diffing against a known-good ROM's own data, or just keeping a capture around after the session that took it ends (unlike `snapshot`, which lives only in memory for that session). `load` is the reverse: poke a raw byte file back into a memory space at a given address.

```
dump <space> <addr> <len> <file>   write raw bytes to var/log/<CoreName>/<file>
load <space> <addr> <file>         write var/log/<CoreName>/<file>'s raw bytes into <space> starting at <addr>
```

Both always resolve `<file>` under `var/log/<CoreName>/` (via `IDebugTarget.CoreName`, §3.1) — the same core-scoped directory F3 screenshots and their companion `.txt` files already use, so debug artifacts from a session end up in one predictable, per-core place rather than scattered relative to wherever the process happened to be launched from, or mixed in with a different core's output once a second core exists.

**Same `HasSideEffects` guard as `search`/`snapshot`** on `dump` — refuses a bulk read over `CpuBus` or any other space where reading can disturb live hardware state. `load` has no equivalent read-side concern but does check `IsWritable`, refusing to write into a read-only space.

**No format, no header — just the bytes.** `dump`'s output is exactly `len` raw bytes starting at `addr`; `load` writes exactly however many bytes the file contains, starting at `addr`, with no length argument of its own (the file's own size *is* the length). This keeps both directly interoperable with any external tool that reads/writes plain binary — a hex editor, `xxd`, a Python script — without EmuSen needing to define or parse its own container format.

**Distinct from save states.** `EmulatorSession`/`VenusCore`'s `SaveState`/`LoadState` (F5/F9 in the frontend) capture *everything* needed to resume execution — CPU, PPU, full bus state — as one opaque blob. `dump`/`load` capture *one named memory space* as plain bytes, readable and editable by anything, with no claim to being a complete or resumable snapshot of emulation state. Different tools for different jobs: F5/F9 for "come back to this later," `dump`/`load` for "take this data somewhere else and look at it."

### 3.12 "Who calls this address" (`callers`, `DianaOS/Commands/CallersCommand.cs`)

The static-analysis counterpart to a write watch: instead of running the game and waiting to observe an access, `callers` reads the code itself and reports every instruction statically calling/jumping to a given address. Generalizes the exact manual step the Yoshi investigation used to find the real DMA-trigger dispatcher — grepping an already-captured CPU trace for `JSR`/`JSL` instructions targeting a known range (see `EmuSen_Core_Gameplan.md`'s current-status section) — into a real command that doesn't need a trace captured first.

```
callers <addr> [<scanstart> <scanlen>]   find instructions statically calling/jumping to <addr>,
                                           scanning CpuBus (default: <addr>'s own bank, $8000-$FFFF)
```

**Fully core-agnostic as of a later pass** - `callers`/`writers`/`readers` all used to hardcode a raw 65816 opcode-byte switch directly in the shell command (`0x20`=JSR, `0x8D`=STA absolute, `0xAD`=LDA absolute, etc.), despite claiming to be core-agnostic - a real gap, since nothing in `IDebugTarget` let a non-65816 core report which of its own instructions have a statically-known target. Fixed by adding `IDebugTarget.ClassifyStaticReference(instr)` - `((StaticReferenceKind Kind, int Target)?`, `Kind` is `Call`/`Write`/`Read` - returning the ISA-specific classification each core's own implementation knows, or `null` for anything not statically resolvable. `SnesDebugTarget.ClassifyStaticReference` now holds the exact opcode table that used to live in the three shell commands; `DebugCommandHelpers.ScanForStaticReferences` is the one shared scan/match/format loop all three call, parametrized only by which `StaticReferenceKind` they're looking for. Same "core does the decoding, the command does the walking" split `Disassemble()`/`DecodeTilemapEntry()` already established - `callers`/`writers`/`readers` just weren't built that way originally.

**Matches on the raw opcode byte, not the mnemonic** (inside `SnesDebugTarget.ClassifyStaticReference` now, same reasoning as before the move): `JMP $nnnn` (absolute, direct — opcode `$4C`), `JMP ($nnnn)` (indirect — `$6C`), and `JMP ($nnnn,X)` (indexed indirect — `$7C`) all disassemble to the mnemonic `"JMP"` with the same 3-byte length, but only the direct form has a target `callers` can know without actually running the code. Indirect forms are deliberately excluded rather than guessed at, same principle `writers`/`readers` apply to direct-page/indexed/indirect stores and loads (only `STA`/`STX`/`STY`/`STZ`/`LDA`/`LDX`/`LDY` in **absolute or absolute-long** addressing resolve to a `Write`/`Read` classification - everything else returns `null`, since the real target depends on runtime register/D-register state a static scan can't know). Same bank-assumed-equals-PB convention throughout.

**The opcode alone is not enough — the instruction's own length has to agree.** `disasm` annotates every line it prints by running it through `ClassifyStaticReference` (§3.29), including lines from spaces that are not 65816 code at all. A GSU `WITH R0` is opcode `$20` in one byte; the table reads `$20` as `JSR $nnnn` and goes looking for two operand bytes that do not exist, so `disasm GSUBUS` died outright with `Index was out of range` on any routine containing one — and GSU code is full of them (`$20`, `$4C` `PLOT`, `$8C`/`$8D` `MULT`, `$AC`/`$AD`/`$AE` `IBT`). Every arm of the table now carries a `when absolute` / `when absoluteLong` guard on `instr.Bytes.Count` (3 and 4 respectively).

This is a correctness fix, not just a crash guard: a one-byte instruction that happens to share a byte with a three-byte 65816 absolute instruction *is* evidence that this line is not that instruction, and returning `null` is the right answer for the same reason the indirect `JMP` forms are excluded above. It fixes `callers`/`writers`/`readers` over `GSUBUS`/`APURAM`/`DSPPRG` too, which had the same latent mis-annotation without the crash. Found while disassembling Yoshi's Island's GSU sprite builder, where the disassembler was the only way to read the routine at all.

**Same "best-effort, may misalign through data mixed with code" caveat as `disasm`.** A linear disassembler walking forward byte-by-byte has no way to know which bytes in a scanned range are really instructions versus embedded data (graphics, tables, text) — if the scan range includes non-code bytes, everything after the first misaligned read can decode to garbage opcodes, including spurious `callers` matches or missed real ones. Best used on a range that's actually known to be code.

**`writers`/`readers`** (`DianaOS/Commands/WritersCommand.cs`/`ReadersCommand.cs`) are the store/load-side counterparts to `callers` - "what code is capable of writing/reading this address," independent of whether that path was ever actually exercised in a traced run (the gap `writers` was built to close: watching an address live can show exactly one write from one PC and nothing else, which only proves what a specific run did, not what the ROM's code is capable of doing).

**A regex or a static scan can only find destinations written as immediates, and that limitation has produced at least one wrong conclusion.** `Venus_SuperFX.md` §10.5 retired "some code uploads to VRAM `$F800` and we lose it" on the strength of a ROM-wide regex for `LDA #imm16 : STA $2116` finding nothing. A runtime `watch add IO 2116 2 write` over the same scene finds **eighteen** sites, two of them writing exactly that destination — the game normally supplies it through a queue node, which no operand-parsing scan can see. Reach for the watch first when the question is "does anything ever address X".

**`watch summary` reports whole-run site totals; `watch log` does not.** The event ring holds 500 events, and `summary` used to group *that* rather than the run — so a long trace reported the last 500 events' site list as if it were complete, which is how the sweep above first returned "exactly one site". Per-site counts now live outside the ring, and the summary's footer says how many events the ring still holds when it is short of the total. Anything reading `watch log` on a busy watch is still seeing a tail, not the run.

**`watch summary <id>` (§3.5) is the dynamic complement to `writers`/`readers`, not a replacement.** It only reports what actually executed during a traced run, but unlike a static scan it doesn't care what addressing mode got it there - an indexed `LDA addr,X` or an indirect `STA (dp),Y` shows up in a summary exactly like an absolute one would, since it operates on the resolved runtime address rather than parsing the instruction's operand form. Neither subsumes the other: `writers`/`readers` finds code paths that exist but may never have been reached yet; `watch summary` finds exactly what a specific run actually did, including forms `writers`/`readers` structurally can't see.

### 3.13 Frame-scoped value logging (`framelog`, `DianaOS/FrameLogRegistry.cs`, `DianaOS/Commands/FrameLogCommand.cs`)

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

### 3.14 Cheat engine (`cheat`, `DianaOS/CheatRegistry.cs`, `Cores/Nintendo/Venus - SNES/Cheats/{ActionReplayCodec,GameGenieCodec}.cs`, `DianaOS/Commands/CheatCommand.cs`)

**Split location, on purpose, as of a core-agnosticism pass.** `CheatRegistry` (RAM-poke/ROM-patch mechanism) and `CheatCommand`'s `poke`/`rompatch`/`list`/`enable`/`disable`/`remove`/`clear` verbs are genuinely core-agnostic and stay under `DianaOS/` - see that class's own comment. `ActionReplayCodec`/`GameGenieCodec` (the `add`/`gg` code-string *decoders*) are not - both are real SNES-specific formats (SNES Game Genie's own cipher/alphabet, different from NES/Genesis/Game Boy's own variants; Pro Action Replay's wire format is at least generic-*looking* 8-hex-digits, but still a real device format, not an abstraction) - moved out of `DianaOS/Cheats/` into `Cores/Nintendo/Venus - SNES/Cheats/`, alongside `SnesDebugTarget`, rather than continuing to sit in the "core-agnostic" `DianaOS/` tree under a framing that never actually applied to them. `CheatCommand` itself stays under `DianaOS/Commands/` and references the SNES codecs directly for `add`/`gg` - a future core wanting its own code-format decoding needs its own equivalent of those two verbs, the same way it needs its own `IDebugTarget` implementation.

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

Core-agnostic on purpose, same as `WatchRegistry`/`FrameLogRegistry` - lives under `DianaOS/`, not `Cores/Nintendo/Venus - SNES/`. A future core's `IDebugTarget` implementation owns its own `CheatRegistry` instance, calls `ApplyAll` from its own per-frame hook, and implements its own core's `IRomReadPatcher`-equivalent forwarding to `TryPatchRom`.

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

### 3.15 `EmuSen.Pharaoh` — scripted CLI harness

A third consumer of `DianaOSInterpreter` alongside the console's F4 prompt (§3.3) and F1 hotkey — no window, no real-time input, built specifically for an AI agent (or any non-interactive caller) to drive an emulation session, script a repro, and read back structured results in one shot instead of needing a human at a keyboard. This has become the primary tool for investigating anything that needs precise, repeatable setup (a specific save state, a specific input sequence, a specific frame to screenshot) — most of the LttP color-math/subscreen investigation and the Super Metroid boot-hang investigation (`Venus_APU.md` §2.7) were done entirely through this, not the interactive console.

**Internally, this is now dispatch over a handful of small pieces**, not one 900-line `Main` - `EmuSen.Pharaoh/`:
- `Program.cs` - thin: dispatches to `DiffShotRunner` for `--diffshot`, otherwise parses args via `HeadlessDebugOptions.Parse`, builds a `FrameRunner`, and either drives the classic frame loop itself or hands off to `CommandsScriptRunner`.
- `Cli/HeadlessDebugOptions.cs` - `Parse(string[] args)` turns argv into a plain options object plus a `Warnings` list, with no I/O of its own (ROM-existence checking stays in `Program.cs`, since that's a filesystem side effect, not parsing) - genuinely unit-testable for the first time (§3.18).
- `FrameRunner.cs` - the one shared frame-stepping primitive both the classic loop and `--commands` mode drive: apply held input, apply `--cpulog` windowing, `RunFrame()`, bump a post-increment `CurrentFrame` ("frames completed so far" - `--commands` mode's own pre-existing convention), fire an autoshot callback, emit the progress heartbeat, enforce the frame-count safety cap. Replaces what used to be two separately-maintained copies of this same sequence. Owns `Hold`/`Release`/`Tap` and `WaitStable` too; does **not** own `FlushVerboseTrace()` - both callers still call that exactly once after their own loop/script finishes, matching before.
- `CommandsScriptRunner.cs` - the `--commands` verb interpreter (`frames`/`tap[2]`/`tapuntil`/`hold`/`release`/`screenshot`/`waitstable`/`waitchange`/`waitvalue`/`contactsheet`/`vramsheet`/`paletteswatch`/`spriteoverlay`/`audiodump`/`gsutrace`/`fastforward`/`rewind`/`layers`/`scanregs`/`perf`), now driving a shared `FrameRunner` instead of its own closures; unrecognized verbs still fall through to `DianaOSInterpreter.Execute`.
- `DiffShotRunner.cs` - the standalone `--diffshot` mode, logic unchanged, now reading/writing BMPs via `EmuSen.Common.Imaging.BmpFile`.

The classic loop's `--tap`/`--screenshot` frame-indexed timing and `--autoshot`'s filenames are bit-for-bit unchanged by this - `Program.cs` captures `FrameRunner.CurrentFrame` *before* each `RunFrames(1)` call for tap/screenshot checks (matching the old for-loop's pre-increment `frame` variable exactly), and passes an autoshot callback that subtracts 1 from `FrameRunner`'s post-increment counter to reproduce the classic loop's original 0-indexed `frame_<n>.bmp` names - only the `--commands` mode's own already-post-increment autoshot numbering (and the progress-log text, called out above) reflect the unified convention directly.

```
dotnet run --project EmuSen.Pharaoh -- <rom> <frames> [options...]
```

| Option | Does |
|---|---|
| `--loadstate <path>` | Load a save state before running any frames |
| `--savestate <path>` | Save a state after the run completes |
| `--tap <frame>:<button>[:duration]` (repeatable) | Hold `<button>` from `<frame>` for `<duration>` frames (default 1) — scripted input, e.g. `--tap 0:Right:60` |
| `--screenshot <frame>:<path>` (repeatable) | Write an uncompressed BMP of the frame buffer at `<frame>` (no PNG library available; convert externally if needed) |
| `--script <path>` | A text file of newline-separated `DianaOSInterpreter` commands (§3.3's table), run once after all frames finish. **Only the last `--script` wins** — passing it more than once silently drops the earlier ones rather than merging, since each flag just overwrites the same variable. Comment lines (`# ...`) and blank lines are skipped. If omitted, defaults to `watch list` + a `watch log` for every still-registered watch. |
| `--watch <space>:<addr>:<len>[:write\|read\|both]` (repeatable) | Register a watch **before** the run starts (unlike `watch add` inside `--script`, which only takes effect for whatever's left of the run *after* the script executes — since the script runs last). Two default watches are always active: `WRAM:0x8000:0x1800` and `WRAM:0xD80:0x80`, matching what `EmuSen.Hotaru` registers from power-on. |
| `--cpulog <start>:<end>` | Enables `CpuVerboseLogging` (+ `MasterLoggingEnabled`) only for frames in `[start, end)` — avoids capturing a trace of the entire run when only a narrow window matters. |
| `--flag <Name>[=<value>]` (repeatable) | Set any `DebugSettings` property or field by name via reflection — also forces `MasterLoggingEnabled = true`, since most `DebugSettings` flags are gated behind it (`Settings/DebugSettings.cs`, §2) and setting the individual flag alone is otherwise a silent no-op. `int` members take decimal or hex (`0x94` or `$94`, matching every address-taking shell command); `bool` members take `true`/`false`/`1`/`0`, or bare `--flag <Name>` for true. A value that will not convert, or a bare name on a non-`bool` member, is a parse **error** now rather than a `FormatException` thrown out of `Main` — `Convert.ChangeType` reads neither hex nor a boxed `true` into an `int`, so `--flag SuperFxRamWriteTraceAddr=0x94` used to crash the run, and piped through a `grep` for the trace's own output that was indistinguishable from "the chip never wrote there". |
| `--verbose` | Shortcut for `MasterLoggingEnabled = true` for the whole run. |
| `--nobattery` | The run neither reads nor writes the cartridge `.srm`. **Use it for every comparison against the Mesen probe.** A battery save is emulator state that survives between runs, so two invocations of one script are not the same run; on a SuperFX cart the save *is* GSU work RAM, which invalidated three days of `Venus_SuperFX.md` §10.7 by making a `tapuntil` anchor fire at frame 0. See `Venus_Memory.md` §2.4a and §3.41. |
| `--out <path>` | Write this harness's own `Emit()`-based log lines (`[ROM]`, `[RUN]`, `[SCREENSHOT]`, the `--script` output, etc.) to a file. |
| `--commands <path>` | Takes over the whole run - see below. Ignores `--tap`/`--tap2`/`--screenshot`/`--script` when present; `<frames>` becomes a hard safety cap instead of an exact count. |
| `--autoshot <dir>` | Works alongside every other mode (classic frame loop and `--commands` both). Hashes `GetFrameBufferRgba()` (64-bit FNV-1a) every frame and writes `<dir>/frame_<n>.bmp` only when it differs from the last saved hash - finds every distinct visual state a run passes through without having to guess frame numbers up front, e.g. confirming exactly when a menu transition settles instead of sampling `--screenshot 550`, `600`, `650`... and hoping one landed on it. |

**`--commands`: an interleaved script, for open-ended exploration `--tap`/`--screenshot` can't do.** Every other flag pre-declares its frame numbers before the run starts, which only works when the exact timing is already known. `--commands` reads an ordered text file and executes each line as it's reached instead, so frame-stepping, input, screenshots, and arbitrary debug commands can freely interleave in one process - built after the Super Mario All-Stars Select Game investigation needed a full relaunch (reboot + replay the whole boot sequence) for every single button guessed. Lines:
- `frames <n>` — advance `<n>` frames, applying whatever's currently held.
- `tap <button> [duration]` / `tap2 <button> [duration]` — press (P1/P2) for `duration` frames (default 4), then release. Inline equivalent of `--tap`/`--tap2`.
- `hold <button> [controller]` / `release <button> [controller]` — set a button's held state without advancing any frames (`controller` defaults to 1) - for holding something across several `frames`/other-command lines rather than one fixed-duration tap.
- `screenshot <path>` — capture the current frame right now, unlike `--screenshot`'s frame-number binding.
- `waitstable [maxframes=300] [quietframes=10]` — advance one frame at a time until the framebuffer hash stops changing for `quietframes` in a row (or `maxframes` is hit), removing the remaining "run N frames and hope it settled" guesswork plain `frames` still needs. Requires seeing at least one real change first - calling it while already sitting on a static screen (e.g. right after a `tap` whose transition hasn't started yet) would otherwise report "stable" instantly, since nothing was moving at the moment it was checked. A single multi-stage cutscene/menu sequence with several automatic pauses along the way needs one `waitstable` call per pause, not one call for the whole thing - confirmed against SMAS's own boot sequence, which turned out to need four separate real `Start` presses (each followed by its own `waitstable`) to reach Select Game, not the two originally assumed from an earlier, more manually-timed run.
- `waitchange <space> <addr> <len> [maxframes=600]` / `waitvalue <space> <addr> <hex[,hex...]> [maxframes=600]` — `waitstable`'s memory-side counterpart. `waitstable` answers "has the picture settled?"; these answer "**did the game actually advance?**", which is the question that matters when a title looks frozen but is still rendering animation. `waitchange` steps one frame at a time until the bytes at `<addr>` differ from what they were when the verb started; `waitvalue` steps until they equal a specific value. Both read through `IDebugTarget.GetMemorySpaces()`, so they work on any core and any space (`WRAM`, `APURAM`, `VRAM`, `SRAM`, ...) with no SNES knowledge baked in. Addresses and `waitvalue` bytes are hex; `<len>` is hex too.

  Both report the frame they stopped on and the before/after bytes, and — importantly — say plainly when the condition was *not* met, which is the useful outcome when investigating a hang:

  ```
  > waitchange WRAM 998 1 200
  [WAITCHANGE] WRAM 0x998 satisfied after 7 frame(s): 02 -> 1E (frame 2067).
  > waitchange WRAM 998 1 3000
  [WAITCHANGE] WRAM 0x998 NOT satisfied - still 1E after 3000 frame(s) (cap 3000, frame 5067).
  ```

  That pair is the whole "Super Metroid freezes during the new-game intro" finding in two lines: the game state advances into the intro, then never leaves it. Before this verb existed the same conclusion needed a hand-written ladder of `frames`/`mem` pairs and manual reading of the output. Both also stop at the run's own `FrameCap`, so a cap-less mistake can't spin forever.

- `contactsheet <path> <count> [every=1] [cols=8] [scale=4]` — capture `count` frames spaced `every` apart, downsample each by `scale` (nearest-neighbor - a debugging aid needs "did this move," not photographic fidelity), tile into one grid image. For confirming actual animation/movement across a span of frames without reviewing several separate screenshots one at a time.
- `vramsheet <path>` — export the current tile/character memory as a BMP, via the new core-agnostic `IDebugTarget.RenderTileSheet()`. On the SNES this delegates straight to `Renderer.GetVramTileSheetRgba()` (§4's `Renderer.Debug.cs` entry), which reuses `RenderVramSheet`'s existing 4bpp grayscale decode unchanged - that method already made zero GPU calls (it only fills a plain `Color[]`), so it works with no window at all. A core with nothing analogous can return a `0x0` empty image rather than needing a special case here.
- `paletteswatch <path>` — export the current color palette memory as a 16x16 swatch-grid BMP, via `IDebugTarget.RenderPaletteSwatch()`. On the SNES this reuses the same `SnesColor()` BGR555 conversion `DrawDebugPanels`'s on-screen CGRAM panel already used, just written into a plain RGBA buffer instead of `Raylib.DrawRectangle` calls (which need an active render target this harness doesn't have).
- `spriteoverlay <path>` — capture the current frame buffer and draw a green bounding-box outline for every entry the already-generic `IDebugTarget.GetSprites()` reports (no interface change needed for this one - `GetSprites()` was core-agnostic from when it was first added). For "is this OAM entry actually where I think it is on screen" questions without needing to cross-reference `sprites`' text output against a plain screenshot by hand.
- `audiodump <path> [maxsamples]` — write whatever's currently buffered in the new `IDebugTarget.GetAudioSamples()` out as a standard 16-bit PCM `.wav` file (`maxsamples` truncates rather than errors if fewer are actually buffered). On the SNES this is `Spc700.Dsp.AudioBuffer.ToArray()` - `ToArray()` specifically, not dequeuing, so this never steals samples out from under a live audio-playback consumer of the same queue. A core with no audio output modeled yet can return an empty array.
- `gsutrace <frames> <path>` — arms the GSU instruction sink, runs `<frames>` frames, and writes the trace, all in one verb. It exists as one verb rather than an arm/disarm pair specifically so it can sit **after** a `tapuntil` line: a GSU trace armed from power-on is mostly a boot sequence Mesen and this core do not even share, and the interesting part would be buried in it. Pairs with the Mesen probe's `--pressuntil` + `traceUntilFrame`, and is read by `--tracediff`. Full write-up: §3.41.
- `fastforward on|off` (alias `ff`) — toggles `ICore.SkipRendering`. Headless already runs unpaced, so there is no speed multiplier to set here; the win is purely from not compositing frames nobody will look at. Measured at **~2.5x** on SMW (3300 frames: 12.4s -> 5.5s), which makes long boot sequences meaningfully cheaper to script past. Turn it back off before any `screenshot`/`contactsheet`/`spriteoverlay` line — a skipped frame leaves the framebuffer holding whatever was last drawn. See `EmuSen_Rewind_And_FastForward.md` §2.2 for the one accuracy caveat (`RangeOver`/`TimeOver` go stale).
- `perf [frames=300] [worstCount=5]` — runs `frames` more frames and reports the wall-clock cost distribution (mean/p50/p95/max, plus a frames-over-budget count), the core's own cpu+spc700 / ppu / hdma attribution with an unattributed remainder, the ppu sub-split, and the worst individual frames by offset. Full write-up, including the Debug-vs-Release finding it was built for: §3.20.
- `framesum [frames=300]` — runs `frames` more frames and folds every one of their frame-buffer hashes into a single 64-bit digest. One number that stands in for "every pixel of every frame in this window," so a renderer change can be shown to be output-identical rather than spot-checked against a screenshot. Full write-up: §3.21.
- `audiosum [frames=300]` — the same idea for sound: folds every sample the DSP produced over the window into one digest, plus `samples`/`nonzero`/`peak`/`rms` so a change has a direction and not just a difference. Drains the audio queue as it goes (unlike `audiodump`, which peeks and therefore only ever sees the last ~2 seconds). Full write-up: §3.22.
- `rewind on [interval=4] [budgetMB=96]` / `rewind off` / `rewind back <n>` / `rewind` — a bounded, in-memory history of core states, captured automatically by every frame-advancing verb once switched on. `rewind back <n>` steps back `n` snapshots (i.e. `n * interval` frames) and keeps the harness's own frame counter in sync; bare `rewind` reports depth, seconds held, bytes held, and raw state size. Core-agnostic — built on `ICore` alone, no SNES knowledge. See `EmuSen_Rewind_And_FastForward.md` §1.

  **This turns "when did it break?" from a re-run into a bisection.** Locating the exact frame a glitch appears otherwise means relaunching from boot with a different `--screenshot` frame each attempt. With rewind you overshoot once, then walk back inside a single process:

  ```
  > rewind on 4 96
  [REWIND] On - snapshot every 4 frame(s), budget 96MB, 1160KB per raw state.
  > frames 700
  > frames 200
  > rewind back 50
  [REWIND] Stepped back 50 snapshot(s) - now at frame 700, 175 left.
  ```

  It reports honestly when the chain runs out rather than silently doing less:

  ```
  > rewind back 500
  [REWIND] Only 100 of 500 snapshot(s) available - now at frame 300, chain exhausted.
  ```

  Cost is ~0.7ms per capture and 4-84KB per snapshot depending on how much the game is churning (a raw state is ~1.15MB). `rewind` on its own is the verb to check that with before setting a long run going.

- anything else — passed straight to `DianaOSInterpreter.Execute`, exactly like `--script`'s lines.

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
dotnet run --project EmuSen.Pharaoh -- --diffshot <bmp1> <bmp2> <outpath>
```
Reads back two of this harness's own BMPs (`--screenshot`/`--autoshot`/`contactsheet` output - not arbitrary external images, since it relies on the exact fixed 54-byte header `EmuSen.Common.Imaging.BmpFile.Write` always produces) and writes a third: every differing pixel highlighted in magenta over a dimmed/grayed copy of the second frame, so a change stands out at a glance instead of two screenshots held side by side. Also prints the changed-pixel count/percentage and bounding box to stdout. Dispatched before `Main` even looks at the usual `<rom> <frames>` positional arguments, since no core/ROM is involved at all - `DiffShotRunner.Run` is the entire implementation.

**Two separate output streams, easy to conflate.** `--out` only captures this harness's own `Emit()` calls. Everything the *emulator itself* prints via raw `Console.WriteLine` — `CpuVerboseLogging`/`Spc700VerboseLogging` traces, `[DMA]`/`[PORT]`/etc. `DebugSettings` output, the `[FRAME] N` marker (`Venus_Memory.md`/`VenusCore.RunFrame`) — bypasses `Emit()` entirely and goes straight to real stdout. Redirecting shell output to `/dev/null` while relying on `--out` for everything discards all of that silently; capture real stdout to a file (`> file.log 2>&1`) instead whenever any `DebugSettings` trace flag is in play.

**`bp add` inside a script does halt** — this line used to say it only counted hits, which stopped being true once `FrameRunner` learned to stop a batch on `IsHaltedAtBreakpoint` (§3.23). A `frames N` that hits a breakpoint returns early with the core halted mid-frame, the next verb inspects that moment, and `resume` followed by another `frames` call continues to the next hit. That resume/inspect cycle is how the Yoshi's Island GSU object loop was walked one iteration at a time (§3.15c's worked example). One rough edge left: the `[BREAK]` line names any halted coprocessor "SA-1" regardless of which chip it was, so a GSU halt reads as `[BREAK] SA-1 halted at $098A05` — cosmetic, and the address is right.

### 3.15b `record` — a real video of a headless run

`contactsheet` answers "roughly what happened over this window" at thumbnail scale. It cannot answer "when exactly did this stop moving", and reading thirty separate images to find out is the wrong shape of work. `record` writes the actual frame sequence instead:

```
record on [<stride>] [<dir>]   start capturing; stride 1 = every frame
record off                     stop, and encode
record                         report whether a capture is running
```

The pieces already existed and were simply never wired together: `DianaOS/Var/FrameRecorder.cs` (session folder, `frames.log` ledger mapping every captured frame back to its emulator frame number, ffmpeg encode with automatic `libx264rgb` → `ffv1` codec fallback, PNG-sequence zip afterwards) was reachable only from `EmuSen.Hotaru`'s F6 hotkey, i.e. only by launching a real window — exactly what the headless harness exists to avoid. `record` gives the same recorder a `--commands` driver: `FrameRunner.AfterFrame` (a second per-frame hook alongside the constructor's own) ticks `CaptureFrame` while a capture is armed, and a script that never says `record off` still gets its video, because `CommandsScriptRunner.Run` stops any live recorder on the way out.

**Encoding is lossless, so the video is evidence, not an impression.** That matters more than it sounds: `ffmpeg -i recording.mkv -f framemd5 -` gives a per-frame checksum, and the longest run of identical checksums is the exact frame the picture froze on — 4150, in the investigation this was built for, found in one command after a contact sheet had only narrowed it to "somewhere between 3300 and 5400". Cross-reference the checksum index against `frames.log` to turn a video frame index back into an emulator frame number.

`record on` clears `SkipRendering`, since `fastforward` would otherwise leave nothing to capture. Frames are written through `EmuSen.Common.Imaging.PngFile`, which is now the project's single RGBA→PNG encoder: `EmuSen.Hotaru/Imaging/FrameImageWriter.SavePng` delegates to it rather than keeping a second copy of the same six SkiaSharp lines.

### 3.15c `savestate` / `loadstate` — stop paying for the boot sequence

```
savestate <path>
loadstate <path>
```

`--loadstate`/`--savestate` already existed as *command-line flags*, which meant a state could only be taken at the very end of a run and loaded at the very beginning. Mid-script there was nothing, and `savestate` as a script line was simply an unrecognized verb that fell through to the DianaOS interpreter and reported `Unknown command 'savestate'`.

This is the single biggest change to how expensive an investigation is. Driving Yoshi's Island from power-on to its opening cutscene costs ~6700 frames of scripted menu input, a minute or two per question asked. With a state pinned at the interesting moment, every subsequent probe — `sprites`, a GSU disassembly, a breakpoint walk, a memory diff — starts from `--loadstate` and answers in seconds. Pin the moment **once**, in the same run that records the video, then never boot again.

`loadstate` clears the rewind chain and calls `RefreshProviders()`, because a state load is a discontinuous jump: without the refresh, `regs`/`sprites`/`pal` would report the pre-load frame's values, which is exactly the moment being inspected. The script's own frame counter deliberately keeps counting rather than jumping to the state's frame number — it bounds the safety cap, and rewriting it would make the cap mean something different depending on which state was loaded.

**States do round-trip.** An older note held that saved `.state` files "resume into a dead machine"; that was verified false here — `frozen.state` reloaded in a fresh process renders the identical frame and continues executing correctly, coprocessor included.

### 3.16 `--singlestep` — ground-truth single-step validation

Not part of the `IDebugTarget` toolchain above (it doesn't run a full emulation session at all) — a separate, permanent harness that validates one CPU's opcode/addressing-mode/flag behavior in isolation against third-party ground-truth test vectors, independent of whatever a real ROM happens to exercise.

```
dotnet run --project EmuSen.Pharaoh -- --singlestep <target> <test-dir> [max-examples-per-file]
```

`<target>` is a registered `ISingleStepTarget` name (currently `65816`, `spc700` and `nes6502`); `<test-dir>` holds the target's JSON test files (one per opcode, from [TomHarte/ProcessorTests](https://github.com/TomHarte/ProcessorTests) — not vendored into this repo, fetch separately). Each test sets up initial registers/memory, single-steps the real emulation code exactly once via the target's `ISingleStepTarget` adapter, and compares final registers/memory byte-for-byte against the vector's expected result. Output is one `[PASS]`/`[FAIL] <file>: <pass>/<total>` line per file, a grand total, then up to `[max-examples-per-file]` (default 3) concrete failure examples per failing file — enough to see the shape of a failure without drowning in 10,000 near-identical ones. Exit code is 0 only if every file passed.

**Why this is a `--singlestep` verb on `EmuSen.Pharaoh` and not its own project.** It began as two throwaway scratchpad tools — one for the 65816, one for the SPC700 — built ad hoc during the same investigation that found and fixed real bugs in both, then generalized into a permanent tool so the next core (or the next bug hunt in an existing one) wouldn't rebuild the same harness from scratch. That permanent home was its own project, `EmuSen.Tomoe`, for a while; it was folded in here once it was clear the project was a 124-line `Main` wrapping a three-entry dictionary, while all the actual machinery already lived in `EmuSen/Validation/` and each core's own `Validation/` folder. It sits alongside `--diffshot` (§3.15) and `--tracediff` (§3.40) as a third standalone verb dispatched before any ROM or option parsing happens — same shape, same reason: a real tool in this toolchain that happens to need no emulation session. Retiring the project also released the `Tomoe` codename back to the pool in `EmuSen_Core_Naming_Scheme.md`, where Professor Tomoe is listed as a natural pull for a future core.

**Why this matters more than it might look:** this is how the real, shipped 65816 `STA [dp]` addressing-mode bug (wired to the wrong function, breaking the ALTTP lamp/inventory bug it was chased down from) and two real SPC700 bugs (direct-page word wraparound, DAA/DAS high-byte-checked-post-adjustment — `Venus_APU.md` §2.4/§2.5) were all found and confirmed fixed. A ROM exercising the exact wrong opcode/addressing-mode/operand combination that exposes a bug like this is rare and easy to miss by playtesting alone (small movements, most values); 10,000 randomized cases per opcode is not.

**Adding a target is: implement `ISingleStepTarget` once** (`Reset`, `SetRegister`/`GetRegister`, `SetMemory`/`GetMemory`, `Step`), **write a small JSON-shape loader** for however that test suite's author formatted their vectors (see `Cpu65816SingleStepTarget.cs`/`Spc700SingleStepTarget.cs` and their paired loaders for the two existing examples — SPC700's loader differs from 65816's because TomHarte's two suites use different JSON field names for the same concepts), and register both in `Program.cs`'s `Targets` dictionary. No changes needed to the runner itself.

**A target whose chip has memory-mapped I/O needs to route `SetMemory`/`GetMemory` through real `Read8`/`Write8`, not a raw array poke** — `Spc700SingleStepTarget`'s own comment covers this in detail: a raw poke into `Ram[]` would silently miss `$00F2-$00F7` entirely (DSP register access, APU communication ports never touch `Ram[]` at all), producing spurious failures that look like emulation bugs but are really just test-setup gaps. Even with that in place, TomHarte's vectors model flat, uninstrumented RAM with no peripherals at all - any test case whose randomly-generated address happens to land on a *real* hardware register EmuSen correctly special-cases (SPC700's `$F0-F3`/`$FD-FF` timers/DSP-address port, not yet routed through this same seeding) will still show as a "failure" that's actually the test harness disagreeing with correct emulated hardware behavior, not a bug — see `Venus_APU.md` §2.6 for the full accounting of which failures are which, the last time this was run.

### 3.17 The shell itself — quoting, variables, pipes, redirection, control flow (`DianaOS/*.cs`, `DianaOS/Commands/*.cs`)

What started as three small Unix-style text tools (`echo`/`sed`/`history`) plus basic `|` pipe support grew, on request, into treating the debug console as a real general-purpose shell rather than just a debug command dispatcher - "support most if not all the features of bash" was the explicit ask, scoped down deliberately to what actually maps onto "a command console embedded in a single-process .NET emulator" (§3.3's own header comment lists exactly what's out of scope and why: background jobs, heredocs, globbing, arithmetic/brace/tilde expansion, functions, arrays, true per-command-ephemeral variable scoping). What's in:

**Quoting** (`DianaOS/Lexer.cs`) - real bash rules, not the old naive whitespace split: `'literal, no expansion'`, `"expanded but not word-split"`, unquoted (expanded AND word-split on whitespace), and `\` escaping a single character. `echo 'a b'` is one argument now, not two.

**Variables** - `NAME=value` (only recognized as an assignment in command position, unquoted, `NAME` a valid identifier - `cmd NAME=value` after the command name is just a literal argument, matching bash), `$NAME`/`${NAME}` expansion, `$?` for the last pipeline's exit code, `export NAME[=value]` and `unset NAME` (there's no subprocess environment to actually export *to* - `export` is kept purely so a script written with real bash habits still does something sane). **Known simplification**: `FOO=bar cmd` sets `FOO` persistently here rather than just for that one command's ephemeral environment, since there's no real per-process environment to scope it to.

**`$(command substitution)`** - runs the inner command line recursively (through a fresh, isolated sub-interpreter - see below) and substitutes its trimmed output. Real bash semantics: it's a genuine subshell, so a variable assignment made inside `$(...)` does **not** leak back out to the caller - a deliberate, faithful reproduction of a real (if easy to be surprised by) bash behavior, not an accident of the implementation.

**Pipes and redirection** - `cmd1 | cmd2 | cmd3` (the first stage gets full `IDebugTarget` access; every later stage must implement the pipe-filter half of `IDianaOSCommand` meaningfully or piping into it errors clearly - `echo x | mem` says so rather than doing something surprising), `>`/`>>` write/append the pipeline's final output to a **real host file** instead of returning it to the caller, `<` reads a real file as the first stage's stdin. Redirection is scoped to the whole pipeline (attached to whichever `SimpleCommand` it textually follows, but only the first stage's `<` and the last stage's `>`/`>>` are meaningfully honored) rather than per-stage real file-descriptor semantics, since there's no OS process/fd model underneath this to make per-stage redirection mean anything different.

**Sequencing** - `;` (unconditional), `&&`/`||` (short-circuiting on the previous pipeline's exit code, left-to-right, same as bash), a leading `!` on a pipeline (logical NOT of its exit status).

**Control flow** - `if`/`then`/`elif`/`else`/`fi`, `for NAME in word...; do ... done`, `while`/`until ...; do ... done`, `break`/`continue`. These two are hardcoded, zero-argument parser keywords (`Parser.cs`'s reserved-word list), not ordinary dispatched commands the way real bash treats its own `break`/`continue` builtins - the one naming collision this causes is `bp` (§3.3's command table), which had to be renamed from `break` for exactly this reason. Real multi-line blocks work both at the interactive F4 prompt (a secondary `> ` prompt appears, bash-style, while a block is still open) and in a `--commands` script (§3.15) fed one raw file line per call - both cases go through the exact same buffering (`DianaOSInterpreter.Submit`/`IsAwaitingMoreInput`): the lexer/parser detect "not finished yet" (an open quote, an unbalanced `$(...)`, or a block missing its closing keyword) and the raw text accumulates across calls until it's actually complete, so neither caller needs to know anything about the grammar to get correct multi-line support. **Safety net**: any loop iteration checks a shared 10-second wall-clock budget (`DianaOSInterpreter`'s `MaxExecutionTime`, shared with any nested `$(...)` subshells so they can't bypass it) and aborts with a clear error rather than hanging the console on a typo'd `while true` with no `break` - verified live: a real `while true; do echo x; done` prints for ~10 seconds, then cleanly reports the timeout and returns control.

**New builtins for conditions**: `true`/`false` (always succeed/fail, no output), `test EXPR`/`[ EXPR ]` (`VALUE` alone = true if non-empty; `-z`/`-n` string emptiness; `-f`/`-d` real file/directory existence - genuinely meaningful here since redirection already touches real host files; `-eq`/`-ne`/`-lt`/`-le`/`-gt`/`-ge` numeric and `=`/`!=` string comparison; `!` negation - deliberately not a full `test(1)`, no `-a`/`-o`/compound expressions, combine conditions with the shell's own `&&`/`||` instead).

**`help` and `man [command]`** (`DianaOSInterpreter.Help`/`Man`, backed by `DianaOS/ManPages.cs`) - two separate, single-purpose commands, not aliases of each other (an earlier revision briefly made `help` forward to `man` for a given command; that's since been reverted - the two kept overlapping in confusing ways instead of each doing one clear job). `help` always prints the full command listing - one line per command with a brief explanation, exactly what it always showed - and ignores any arguments; it never shows a single command's detailed page. `man [command]` is the detail lookup: with no argument there's nothing more useful to show, so it falls back to that same `help` listing, but `man <command>` prints that command's full manual page - a NAME/SYNOPSIS/DESCRIPTION(/EXAMPLES) writeup, not just the one-line `Usage` string the listing shows - covering every command `DianaOSInterpreter.CreateDefault` registers, and every special-cased builtin this section documents (`export`/`unset`/`source`/`if`/`for`/`while`/`break`/`continue`/`man`/`help` itself). Deliberately a separate, centralized module rather than a `ManPage` member added to `IDianaOSCommand`: that would force every one of the ~40 existing command classes, most of them one-liners, to carry a paragraph of documentation alongside their actual logic, for a feature that is purely about the shell's own help system. One file, one place to keep in sync, no per-command interface churn — the same reasoning `DebugCommandHelpers` already uses for shared, stateless command-support code that does not belong wedged into any one command class. A command name with no page yet falls back to just its own `Usage` line instead of a hard failure, so a newly-added command isn't broken by `man` before its page gets written; only a name that's neither a real command nor a special builtin is a genuine error.

**Where the file lives, and what a page owes you.** `ManPages.cs` sits directly under `DianaOS/Bin/Commands/` rather than in `Commands/Unix/` or `Commands/EmuSen/`: it documents commands from both subfolders and is not an `IDianaOSCommand` itself, so neither one fits. Each page follows the loose shape a real Unix man page has — NAME, SYNOPSIS, DESCRIPTION, and EXAMPLES *where an example clarifies something the synopsis alone does not* — without being slavish about it. This is a debug console's help text, not a formal reference manual.

**`Lookup` returning `null` is an expected, handled case rather than a bug to close.** The `Usage` fallback exists so that writing a page can lag creating a command; the answer to a missing page is to write it when the command has settled, not to add a stub for absolutely everything the instant it is created. What is *not* optional is coverage of what the shell actually registers — `ManPageTests` (§3.18) fails a registered command with no real page. **That test reads `CreateDefault(null)`, so it structurally cannot see a command a frontend registers through `extraCommands`** — `pause`, `feed`, and the windowed `coretop`/`vstop` replacements are invisible to it. A second `[Theory]` names those explicitly for that reason; both `pause` and `feed` went unwritten until 2026-08-16 precisely because nothing was watching them.

Note the direction of reference here: this section is the explanation, and `ManPages.cs` carries a one-line pointer to it. An earlier revision of this paragraph had it the other way around, citing "`ManPages`'s own comment" as the authority for the design argument while that comment ran to nineteen lines — which is the arrangement the project's comment rule exists to prevent.

**Scripting: `source <path>` / `. <path>`** (special-cased in `DianaOSInterpreter.Dispatch`, alongside `help`/`export`/`unset` - not an ordinary `IDianaOSCommand`, since it needs to feed lines back through this same interpreter's own execution path, which the `IDianaOSCommand` interface has no way to reach). Runs a script file's lines **in the current shell's own scope**, the way real bash `source` does and running a separate script process wouldn't - a variable a sourced script sets (`X=42`) is still set once `source` returns, and `if`/`for`/`while` blocks spanning multiple physical lines in the file work exactly as they would typed interactively, reusing the exact same buffering (`SubmitCore`/`IsAwaitingMoreInput`) real multi-line control flow already goes through. Blank lines and lines starting with `#` are skipped. Two things are deliberately different from typing the same lines at the prompt: a sourced script's own lines are **not** recorded into `history`/`!N` recall (matching real bash - only what's actually typed interactively shows up there), and `!`/`!!` expansion is disabled for them (a script referencing "the interactive session's last command" would be confusing, since the script itself was never typed interactively). Walled to `DianaOSSandbox` like every other real-file command here. Two safety nets, both producing a clean error rather than a crash or a silent hang: a script ending mid-block (an `if` with no matching `fi`) reports "unexpected end of file" and resets rather than leaving a broken continuation waiting for whatever's typed next; a self-referential or mutually-recursive `source` chain (`a.txt` sourcing itself, or `a`↔`b`) is capped at 20 nested levels (`MaxSourceDepth`) - each level of `source` recurses through the C# call stack itself, which has no other circuit breaker the way a runaway `while`/`for` loop already has via the 10-second wall-clock budget above, so left unchecked it would end in an uncatchable `StackOverflowException` instead of a reported error.

**`nano <path>`** (`DianaOS/Commands/NanoCommand.cs`) - a small full-screen text editor for creating/editing a real file (a `source` script, say) without leaving the shell for an external editor. Genuinely different from every other command in this namespace: everything else is "take target/args/stdin, return a string," one synchronous call - `nano` takes over the real console (raw key reads via `Console.ReadKey`, full-screen redraws via `Console.SetCursorPosition`/`Console.Clear`) for as long as the user is editing, the same way `ConsoleLineReader` already does for single-line editing, just for a whole buffer. Arrow keys/Home/End/PageUp/PageDown move the cursor, Backspace/Delete/Enter/typing edit the buffer, Ctrl+O saves, Ctrl+X exits (prompting `y`/`n`/Esc-to-cancel first if there are unsaved changes) - not a full GNU nano clone (no search, no cut/paste ring, no syntax highlighting). `<path>` is created if it doesn't exist yet, walled to `DianaOSSandbox` like every other real-file command here. Because it needs a REAL interactive terminal, `Console.IsInputRedirected`/`IsOutputRedirected` are checked up front and refused cleanly rather than attempting to draw anywhere - covers a piped script, a `source`d file (running `nano` from inside a script would deadlock waiting for keystrokes a non-interactive stream will never supply), a headless test harness, and a GUI-hosted console (`EmuSen.Mistress`'s window is a `TextBox`, not a real terminal, so it has no `Console.ReadKey` stream of its own to intercept in the first place).

**`coretop`** (`DianaOS/Commands/CoretopCommand.cs`) - an htop-style live dashboard of the loaded core's "hardware," auto-refreshing 4x/second until Ctrl+C, entirely from `IDebugTarget` data so it's core-agnostic the same way every other command here is (a core with a section's data not modeled just makes that section shrink or disappear, never fake numbers). Sections: **hardware load bars** (`IDebugTarget.GetHardwareLoad()`, new - see `DebugLoadInfo`'s own comment, three bars on the SNES: CPU+SPC700/PPU/HDMA, each a percentage of one 60fps frame's ~16.67ms wall-clock budget, clamped to 100% so a frame that ran behind schedule can't report over-full); **CPU registers** (`GetCpuRegisters()`, one compact line); a **sprite-capacity gauge** (`GetSprites().Count` against the new `IDebugTarget.MaxSprites` - 128 on the SNES, a real fixed OAM capacity, not a live-derived number - falls back to a bare count if a core reports 0, meaning "no fixed capacity modeled"); **per-voice audio meters** (`GetAudioChannels()`, reusing the exact same 0-100 `Level` the `channels` command already shows, one bar per voice); a **live color-RAM palette swatch** (`GetPalettes()`, rendered as real ANSI 24-bit background-color blocks, not hex text - a genuine "show me the palette," not a numeric approximation of one); and, gated on `TilemapEntryStride > 0` as this dashboard's stand-in for "this core has a tilemap concept at all," a **downsampled live preview of `RenderTileSheet()`'s VRAM tile data**, same ANSI-block technique. Load bars are colored green/yellow/red under 60%/60-85%/over 85%, htop's own convention for "how busy is this."

`GetHardwareLoad()`'s real numbers needed a small, additive change to reach `SnesDebugTarget`: `VenusCore.LastFrameCpuSpc700Ms`/`LastFramePpuMs`/`LastFrameHdmaMs` (the same per-subsystem profiling counters `EmuSen.Hotaru`'s own `[FPS]` status line used to print, back when it existed - added during an `EmuSen.Mistress` slowdown investigation, and since removed entirely along with the rest of the always-on `[FPS]`/`[STATUS]` prints, per this doc's own top note) live on the concrete core/session, not on anything `SnesDebugTarget` already held a reference to (`Cpu`/`MemoryBus`/`Renderer`) - so its constructor gained an optional `Func<(double CpuSpc700Ms, double PpuMs, double HdmaMs)>? frameTimings` parameter, wired up at all three real call sites (`EmuSen.Hotaru`, `EmuSen.Mistress`, `EmuSen.Pharaoh`) but left as its default `null` at the `EmuSen.WiseMan` test fixtures that don't care about it, where `GetHardwareLoad()` correctly reports "not modeled" (an empty list) rather than needing every test call site to invent plausible-looking timing numbers just to satisfy the constructor.

Same "needs a REAL interactive terminal" gate as `nano`, for the same reason - refuses cleanly rather than trying to draw anywhere when `Console.IsInputRedirected`/`IsOutputRedirected`, covering a piped script, a `source`d file, or a headless test. Ctrl+C is genuinely read as a key here (`Console.TreatControlCAsInput = true`, restored the moment `coretop` exits) rather than raising a process-level interrupt the way it would anywhere else in this shell - the one command in this whole toolchain where that's actually the intended exit gesture instead of an accident.

**`EmuSen.Mistress` replaces `coretop` entirely rather than refusing it** (`Views/CoretopWindow.axaml(.cs)`, `Views/EmulationControlCommands.cs`'s `CoretopWindowCommand`) - its own console window is a `TextBox`, not a real terminal, so the raw-terminal implementation above genuinely cannot run there (no `Console.ReadKey`/ANSI-code stream to intercept in the first place), but there's no reason a GUI frontend should just lose the feature instead of getting a native version of it. `CoretopWindowCommand` opens a real, non-blocking Avalonia window (`CoretopWindow`) that polls the exact same `IDebugTarget` data on its own `DispatcherTimer` (same 250ms/4Hz cadence) and renders it as actual widgets - `ProgressBar`s for the load/sprite/audio meters, and real `Image`/`WriteableBitmap`s for the palette swatch and VRAM tile sheet (`RenderPaletteSwatch()`/`RenderTileSheet()`'s RGBA output copied straight into a bitmap, the same technique `MainWindow` already uses for `GameView`'s own frame - no downsampled ANSI-block approximation needed once real pixels are an option). Opening it never pauses emulation on its own, matching the console window's own philosophy (see `pause`/`resume`'s entry above) - gameplay keeps running exactly as if the shell console window itself were merely open. Reached this way because `DianaOSInterpreter.CreateDefault`'s `extraCommands` parameter learned a new trick for this: an extra command whose `Name` matches one already in the standard registry now **replaces** it (case-insensitive) instead of the constructor's `Dictionary` build throwing on a duplicate key - the first real use case for overriding rather than just adding alongside, now that a command's default implementation can be flatly wrong for a specific host. `ManPages.cs`'s `coretop` entry documents both behaviors in one place, since `man coretop` has no way to know which registry it's actually running against. `CoretopWindowCommand.Execute` never looks at `args` beyond checking for a live target, so `coretop -w` there is already identical to bare `coretop` with no code change needed - windowed is simply the only mode Mistress has.

**`coretop -w` brings the same "open a window instead" idea to `EmuSen.Hotaru`** - the one frontend where it didn't already exist for free (before this frontend moved onto Avalonia, a Raylib console build had exactly one native OS window and no built-in way to pop a second one - see `EmuSen_Frontend_Driver.md`'s own top-of-file revision note). `CoretopCommand` itself gained an optional `Action<IDebugTarget>? openWindow` constructor parameter and a small amount of `-w` parsing (`args.Any(a => a.Equals("-w", StringComparison.OrdinalIgnoreCase))` - case-insensitive, works regardless of where in `args` it appears); with `-w` present it calls the opener instead of ever touching the terminal, and with no opener wired up (every headless/`EmuSen.WiseMan` context, and `EmuSen.Hotaru`'s own no-ROM standalone shell where the flag would only ever fail on "No ROM loaded" first anyway) it fails cleanly with `"coretop: -w is not supported by this frontend."` rather than silently falling back to the raw-terminal dashboard. `EmuSen.Hotaru`'s in-game `debugCmd` wires a real opener in via the same `extraCommands` override-by-name mechanism Mistress's own windowed replacement uses - `new CoretopCommand(DebugWindows.ShowCoretopWindow)` replaces the standard registry's plain, opener-less `coretop`. An already-open `coretop -w` window also gets pushed the new target automatically after a `core <name> <path>` swap (`DebugWindows.UpdateCoretopWindowTargetIfOpen`, called from `GameWindow.SwapCore`) - without this it would silently keep showing data for the discarded core forever, since it has no way of noticing a swap happened on its own (only a refresh timer that re-polls whatever target it was last told about). Never creates the window and never steals focus (`Activate()`) on a swap - only `coretop -w` itself does that - so this is a no-op for anyone who never opened one.

`Views/DebugWindows.cs` (`EmuSen.Hotaru`) is what opens that second real OS window - and it's just a `Dispatcher.UIThread.Post(...)` now, nothing more, since Avalonia is Hotaru's primary application (see `EmuSen_Frontend_Driver.md` §1). Before this frontend moved off Raylib, this needed a whole separate class (`AvaloniaHost.cs`, deleted - see git history) to bootstrap a second, independent Avalonia dispatcher on its own dedicated thread, entirely apart from Raylib's own main-thread game loop - that machinery is gone now that there's only one GUI toolkit's event loop in the process at all. `ShowCoretopWindow(IDebugTarget)` posts onto the already-running UI thread to create-or-reuse a single cached `CoretopWindow` instance, the same at-most-one/reuse pattern Mistress's own `OpenCoretopWindow` already established. `EmuSen.Hotaru/Views/CoretopWindow.axaml(.cs)` itself is unchanged by any of this - a deliberate near-duplicate of Mistress's own `CoretopWindow` (same layout, same `DispatcherTimer`-driven 250ms refresh, same `ProgressBar`/`WriteableBitmap` rendering) rather than shared code, matching this project's established precedent that Hotaru and Mistress are intentionally separate, non-sharing UI projects.

**Closing the window is `-w`'s Ctrl+C** - there's no separate raw-terminal loop running alongside a windowed `coretop` to keep in sync (choosing `-w` means the terminal dashboard never starts at all for that invocation), so the window's own `Closed` event, which stops its refresh timer, is the entire "stop watching coretop" action, exactly the one decision Ctrl+C makes in the terminal version. Same accepted caveat as everywhere else `IDebugTarget` is polled off the emulation thread: the window's refresh timer calls `GetHardwareLoad()`/`GetCpuRegisters()`/etc. with no synchronization against `Program.cs`'s own concurrent `RunFrame()` calls - a narrow, accepted race for what's fundamentally a read-mostly debug view, not something feeding back into gameplay logic, and `EmuSen.Hotaru` has no `pause`/`resume` mechanism at all to close that gap with even if it wanted to (unlike Mistress - see that command's own entry above).

**`echo`/`sed`/`history`/`!N`/`!!`** - unchanged in behavior from when they were first added, just now living under the unified `IDianaOSCommand` interface (`DianaOS/Commands/EchoCommand.cs`, `SedCommand.cs`, `HistoryCommand.cs`) instead of the retired `IDebugCommand`/`ITextFilterCommand` split. `sed`'s substitution is still applied **per line** (without `g`, only the first match on each line, not the first match in the whole piped blob) and still supports any delimiter (`s#/bin#/usr/bin#`) with `\<delim>` escaping. `history`/`!N`/`!!` are backed by `CommandHistory` (`DianaOS/CommandHistory.cs`), a small shared-state object `DianaOSInterpreter` writes to on every completed (non-buffered) command line and both `HistoryCommand` and `ConsoleLineReader` (below) read from - same "neither side owns it" pattern `SnapshotStore` uses between `snapshot`/`diff` (§3.10). `!N`/`!!` expansion happens before lexing even starts, the same point a real shell expands `!`-references, and only at the start of a brand-new (not mid-block) line.

**`grep`/`wc`/`sort`/`uniq`** (`DianaOS/Commands/{GrepCommand,WcCommand,SortCommand,UniqCommand}.cs`) - the small coreutils subset that makes piping through this shell actually useful day to day, since almost everything worth filtering (`regs`, `watch log`, `sprites`, `history`) is multi-line text. All four read piped `stdin` when present, falling back to trailing literal arguments (`wc`/`sed` additionally accept a real file path directly, the same way real `wc <file>` does, without needing `<file` spelled out).

- **`grep [-i] [-v] [-n] [-c] [-q] <pattern> [text...]`** - pattern is a .NET regex, same choice `sed` made. `-i` case-insensitive, `-v` invert (non-matching lines), `-n` prefix each match with its 1-based line number, `-c` print only the match count, `-q` no output at all. Exit code is 0 if anything matched, 1 otherwise (real grep semantics) - `-q` exists specifically so `grep` can drive an `if`/`while` condition (`if regs | grep -q PC; then ...`), not just filter text. No `-E`/`-F`/`-P` mode switches, no `-A`/`-B`/`-C` context lines, no multi-file support - there's one input stream here, not a filesystem of them.
- **`wc [-l] [-w] [-c] [path]`** - counts lines/words/bytes; with no flags, prints all three in that fixed order (matching real `wc` regardless of what order flags are given in, when flags are given at all).
- **`sort [-n] [-r]`** - `-n` numeric comparison instead of ordinal string, `-r` reverses. Deliberately no `-u` - that's `uniq`'s job, pair them (`sort | uniq`) the way real shell usage does before reaching for `-u` as a shortcut.
- **`uniq [-c]`** - collapses **adjacent** duplicate lines only, same real semantic as bash's own `uniq` (it is not a global dedup - sort first if that's what's actually wanted). `-c` prefixes each remaining line with its consecutive repeat count.

**`ls`/`cd`/`pwd`/`mv`** (`DianaOS/Commands/{LsCommand,CdCommand,PwdCommand,MvCommand}.cs`) - real filesystem access, not text plumbing, made meaningful by the same fact that already justified `test -f`/`-d` and real-file redirection: this shell always runs inside a real dotnet process with a real, single, process-wide current directory (`Environment.CurrentDirectory`), not a sandboxed/simulated one, and permissions are explicitly out of scope for this project (there's nothing here to check). `cd` changes that process-wide directory directly - there's no shell-private cwd concept layered on top, so it also affects `pwd`, `ls`'s default path, and every relative path anywhere else in the shell (redirection, `wc`/`awk <path>`, `dump`/`load`'s `home/Logs/<core>/` paths) the same way real `cd` affects a whole process's later relative-path resolution. No-argument `cd` goes to `/home`, the one user data home (see `hier` below), not the bare sandbox root. There is exactly one - accounts are identities, not directories. `ls [-a] [-l] [path]` lists a directory (or a single file, listed as itself) one entry per line - no multi-column terminal layout (this shell's output is a text pane/pipe, not a live terminal), and `-l` prints a simplified type/size/modified-time/name line rather than real `ls -l`'s permission-bits/owner/group fields, since this project has no permissions/ownership model to show. `mv <src> <dst>` moves/renames a file or directory, landing *inside* an existing destination directory (`mv foo.txt logs/` → `logs/foo.txt`) the way real `mv` does, and silently overwriting an existing destination **file** (no `-i` prompt anywhere in this shell, by design).

**Walled to the project (`DianaOS/DianaOSSandbox.cs`)** - `cd`, `ls`, `mv`, `awk`/`wc`'s trailing file-path argument, and all three redirection forms (`>`/`>>`/`<`) all route through `DianaOSSandbox.TryResolve`, which resolves a path and then rejects it outright - `cd: '<path>' is outside the project sandbox (...)`, no partial effect - if the result would land outside `DianaOSSandbox.RootDirectory`. No amount of `cd ../../..` or `mv foo ../../../outside` gets out; navigating *within* the sandbox (`cd EmuSen.DianaOS`, then `cd ..` back, `echo x > ../y.txt` landing at the sandbox root from one level down, ...) works normally throughout. `RootDirectory` is resolved once per process by walking upward from `AppContext.BaseDirectory` (not `Environment.CurrentDirectory` - a GUI app's launch directory is exactly the kind of thing this project doesn't control) looking for `EmuSen.sln`, the one file that reliably marks this project's own root today. Once this project is ever published as a standalone binary and `EmuSen.sln` no longer ships alongside it, the upward walk finds nothing and falls back to `AppContext.BaseDirectory` itself - "the folder the binary lives in" becomes the walled root then, which is exactly the "still the parent directory" default this was asked to keep. Explicitly a walled garden against ordinary accidents/typos, not a security boundary against a hostile actor - nothing stops a `--commands` script or another part of this same process from just setting `Environment.CurrentDirectory` directly.

**A Unix-shaped layout inside that same sandbox root (`man hier`), added once the Users feature (§3.3) gave the shell a real concept of "whose home directory is this."** `TryResolve` now treats a leading `/` as THIS shell's own root, chroot-style, rather than the real OS filesystem root - `cd /home/Logs` works from anywhere, matching how a real `chroot` makes `/` mean the chroot directory. This creates an ambiguity `TryResolve` has to resolve explicitly: a leading-`/` path that's already a real, fully-qualified path landing inside `RootDirectory` (the shape every `DianaOSSandbox.RootDirectory`-built test path takes, e.g. `Path.Combine(DianaOSSandbox.RootDirectory, "home", "Logs", ...)`) is honored as-is rather than being re-rooted a second time; only a short virtual reference like `/home/Logs`, typed by a person or built as a bare string, actually goes through the chroot reinterpretation. `EnsureSkeleton()` creates the real directories this layout needs (`home/{Games,Saves,Saves/Save States,Firmware,Cheats,Logs}`, `etc`, `tmp`) the first time any shell starts in the process, and the initial cwd is `home` (a real login shell's own "boot into $HOME" convention), not the bare sandbox root. `useradd` creates **no** directory - per-account homes were dropped after they accumulated 6,179 empty folders from test runs, and an account is now an identity only (see §3.3 and `man hier`).

**The physical data-folder move that came with this**: `Logs/` → `var/log/`, `SaveStates/` (didn't exist yet) → `var/lib/`, `Saves/` → `var/games/` - real files moved (`SMW.srm`, `cputest-full.srm`, the existing `Logs/SNES/console_*` runs), not symlinked, since a "keep `Logs`/`Saves` physically where they were and only change what the shell *displays*" approach turned out to need `var/` to be a synthetic directory with no real backing (`Logs`/`Saves` would sit as *siblings* of a would-be `var/`, not children of it) - full physical move was both simpler to implement and more honestly "this is now a Unix tree." Every real reference to the old names was updated alongside the move: `DumpCommand`/`LoadCommand` (`var/log/<CoreName>/`), `EmuSen.Hotaru/Program.cs` and `GameWindow.axaml.cs` (console logs, screenshots, frame recordings, and the default save-state path), `Cartridge.cs` (the actual SRAM save directory), and every `EmuSen.WiseMan` fixture/test building a `Logs/...` scratch path. **Not touched:** `EmuSen.Mistress`'s own Saves/Logs paths live under `Environment.SpecialFolder.ApplicationData`, never inside this repo-root sandbox to begin with.

**Test-suite fallout worth knowing about:** three existing tests used `Path.GetTempPath()` (literally `/tmp/` on Linux) as a stand-in for "a path outside the sandbox" - `FilesystemCommandTests.Filesystem_commands_reject_paths_outside_the_sandbox`, `XxdCommandTests.Xxd_rejects_a_path_outside_the_sandbox`, `ScriptingTests.Source_of_a_path_outside_the_sandbox_is_rejected`. Under chroot semantics `/tmp/...` no longer means the real OS `/tmp` - it resolves to the shell's own (now legitimately in-sandbox) `tmp/` directory, so all three were rewritten to climb outside the root via a `..`-relative path instead (`string.Concat(Enumerable.Repeat("../", 15))`), which still resolves outside `RootDirectory` regardless of the new leading-slash behavior.

**`awk`** (`DianaOS/Commands/AwkCommand.cs`) - a deliberately small subset, not a real AWK clone: real AWK is a whole language (arithmetic, user functions, associative arrays, `printf`), and this shell already draws a hard "no arithmetic expansion" line for itself (§3.17's own "not yet done" list) rather than embed a second, more capable expression language behind one command. What's supported covers the actual common case - filtering/reshaping another command's text output by field: patterns are empty (matches every line), `/regex/`, `NR==N` (also `!=`/`<`/`<=`/`>`/`>=`), `BEGIN`, and `END`; actions are `{print expr[, expr...][; print ...]}` where `expr` is `$0`, `$N`, `$NF`, `NR`, `NF`, a `"quoted string"`, or a bare number - no concatenation, no arithmetic, no user variables. A bare pattern with no `{...}` defaults to `{print $0}` (real awk's own default action); an explicit empty `{}` does nothing (also matching real awk). Field splitting is whitespace-runs by default (leading/trailing trimmed first) or a literal separator string via `-F sep` (not a regex the way real awk's multi-character `-F` is - a documented simplification, same spirit as `sort`/`uniq` skipping `-u`) - `\t`/`\n` inside the `-F` argument are unescaped the way real awk's own `-F` handling does. Reads piped `stdin` when present, otherwise a trailing file path argument (`awk '{...}' file.txt`) or real `< file` redirection, same convention every other file-capable command here (`wc`, `sed`) already uses. One parsing wrinkle worth calling out: rules are normally separated by `;`/newline like this shell's own command sequencing, but a rule's own closing `}` also ends it with no separator needed before the next rule's pattern - required for the extremely common `BEGIN{...} {...} END{...}` one-liner idiom (three rules, space-separated only) to parse as three rules rather than one malformed one.

**Up/down-arrow recall (`DianaOS/ConsoleLineReader.cs`).** `Console.ReadLine()` gives no line editing beyond whatever the terminal itself does, and no history recall at all - readline's job in bash, with no .NET equivalent. `ConsoleLineReader.ReadLine(history)` is a small, deliberately "basic" replacement (reads one raw key at a time via `Console.ReadKey(intercept: true)`, manually echoes/redraws) supporting cursor movement (arrows/Home/End), insert/Backspace/Delete, and Up/Down history recall against the exact list `history`/`!N` already use - press Up to walk backward through everything typed this session, Down to walk back forward, past the newest entry to whatever you'd been mid-typing before the first Up press (same behavior real readline gives you). Falls back to plain `Console.ReadLine()` when stdin is redirected (`Console.IsInputRedirected` - a script/CI feeding input via a pipe), since `Console.ReadKey` throws in that mode and there's no "up arrow" to recall from a non-interactive stream anyway. The F4 prompt (`EmuSen.Hotaru/Program.cs`'s `RunDebugPrompt`) also suppresses its own single-word shortcuts (`exit`/`quit`/`continue`/`c`/`step`/`s`/`state ...`) while `DianaOSInterpreter.IsAwaitingMoreInput` is true, so typing the shell's own `continue`/`break` keywords (or any other word that happens to collide) while composing a loop body reaches the shell instead of being hijacked as "resume emulation."

Lives in `EmuSen.DianaOS` (not `EmuSen.Hotaru`, the only current caller) specifically so a future console-based entry point doesn't have to duplicate it - the same frontend-agnostic reasoning `DianaOSInterpreter`'s own header comment gives. **Not** what the Avalonia GUI's own console window uses, per the prediction this paragraph used to make before that window existed: `EmuSen.Mistress/Views/DianaOSConsoleWindow.axaml` (`Settings > DianaOS Console...`) is a real Avalonia `TextBox` that owns its own text editing, adding only Up/Down-arrow history recall on top via the `TextBox`'s own `KeyDown` event, reading directly from `DianaOSInterpreter.History` - exactly the division of labor predicted, now real. It's the same `DianaOSInterpreter`/`DianaOSInterpreter.CreateDefault` every other caller uses, not a cut-down GUI-only subset - quoting, variables, `$(...)`, pipes, redirection, if/for/while, all of it. Built against a `SnesDebugTarget` constructed from two new `EmulatorSession` escape hatches (`Cpu`/`Renderer`, alongside the pre-existing `Bus`) and rebuilt fresh on every ROM (re)load - `MainWindow` pushes the new target into an already-open console window via `DianaOSConsoleWindow.UpdateTarget(target, romName)` rather than leaving it pointed at an abandoned core, at the cost of losing `!N`/`!!` history and variables across a reload (an accepted tradeoff, not a bug).

**`pause`/`resume` (Mistress-only, `EmuSen.Mistress/Views/EmulationControlCommands.cs`).** Opening the console window does *not* stop the game - `EmulationLoop` keeps calling `RunFrame()` on its own background thread the entire time the console is open, same as always (see that method's own comment on why emulation runs off the UI thread at all). That's normally fine, but it means a shell command run from the console (`mem`, `write`, `cheat poke`, `bp add`, ...) executes on the UI thread while `RunFrame()` concurrently mutates the same `Cpu`/`Bus`/`Renderer` state on the emulation thread, with no synchronization between the two - a real, if narrow, data race (a read command could observe a mid-frame/inconsistent snapshot; a write command could get raced and silently overwritten next frame). `pause` and `resume` close that gap: they're two extra `IDianaOSCommand`s, not part of the standard ~30-command registry `DianaOSInterpreter.CreateDefault` normally builds (they can't be — they act on `MainWindow`'s own emulation thread, which `IDebugTarget` deliberately has no concept of), passed in via a new optional `extraCommands` parameter on `CreateDefault` instead. `MainWindow` builds them with plain delegates back to itself (`PauseEmulation`/`ResumeEmulation`/`IsPaused`) and hands them to `DianaOSConsoleWindow`, which re-adds them every time it rebuilds `_shell` in `UpdateTarget` (a ROM (re)load would otherwise silently drop them along with everything else `CreateDefault` builds fresh). Mechanically, pausing means `EmulationLoop`'s `while (_running)` body blocks on a `ManualResetEventSlim` (`_pauseSignal`) at the top of each iteration rather than calling `RunFrame()` - `StopEmulationThread` always `Set()`s it before `Join()`ing so a paused thread can still wake up and exit rather than deadlocking if the window is closed (or a new ROM is loaded) while paused. Not automatic on window-open/close by design: the user may want to leave a game running while merely typing a command, and only actually pause right before running something that touches live state.

**`feed`/`feed -w` - "let me see the game" in one word, everywhere DianaOS runs.** Not a registered command in `EmuSen.Hotaru` (same reasoning as `resume`/`shutdown`/`step`/`state` above - it needs to hand control back to the emulation thread's own loop, which a command's own "return a string" contract can't do); typed at the F4 prompt, `feed` resumes exactly like `resume` does, but also arms a way back into the prompt that doesn't need the game window focused: the per-frame hotkey dispatch polls `Console.KeyAvailable` once per frame (same non-blocking technique `coretop`'s own dashboard uses for Ctrl+C, just spread across frames here instead of one blocking loop, since gameplay has to keep running underneath it) and reopens the debug prompt the moment Ctrl+C arrives at the terminal - `Console.TreatControlCAsInput` is set for exactly as long as this watch is armed, and reset to normal (Ctrl+C means "shut down" again) the instant the prompt reopens, by whatever path got it there. `feed -w` does everything bare `feed` does, plus opens a small Avalonia window (`DebugWindows.ShowFeedWindow`, `Views/FeedWindow.axaml(.cs)`) mirroring the actual game picture (`ICore.GetFrameBufferRgba()`) rather than hardware/debug data the way `coretop -w`'s window shows - just a `Dispatcher.UIThread.Post` now, same as `coretop -w` (§3.17 above), with no second dispatcher thread involved at all. Refreshes at ~30Hz, fast enough to read as "watching the game" rather than a slowly-updating readout, reusing one `WriteableBitmap` for the session instead of rebuilding it every tick since the screen resolution never changes mid-session. `EmuSen.Mistress` gets a real registered `feed`/`FeedCommand` (`Views/EmulationControlCommands.cs`) instead, but there's nothing to mirror there - `MainWindow`'s own `GameView` already shows the live picture continuously in the same window the whole time `DianaOSConsoleWindow` is open, so `feed` there just calls `Activate()` to bring that window back to the front (in case the console window is covering it) and ignores `-w` entirely, same as this frontend's own `coretop` already does.

**`clear`** (`DianaOS/Commands/ClearCommand.cs`) - a real terminal `Console.Clear()`, part of the standard registry `DianaOSInterpreter.CreateDefault` builds for every frontend, unlike `feed`/`resume`/`shutdown`/`step`/`state` above: it doesn't need to hand control back to a caller's own loop, so it's an ordinary `IDianaOSCommand` like `echo`/`ls`/`pwd` rather than a special-cased keyword. Only meaningful against a real interactive console; a redirected or nonexistent one (a piped script, a `source`d file, a headless `EmuSen.WiseMan` test) throws `IOException` from `Console.Clear()`, caught and swallowed rather than surfaced as a shell error - there was never a real screen to clear in the first place, so failing loudly over it would just be noise. `EmuSen.Mistress`'s own console window replaces it (`ClearConsoleWindowCommand`, `Views/DianaOSConsoleWindow.axaml.cs`) via the same extraCommands override-by-name mechanism `coretop`/`pause`/`resume`/`feed` already use there - a TextBox has no real console underneath it for `Console.Clear()` to touch, so the override just empties that window's own `OutputText` instead. Unlike `coretop`/`feed`, this override lives inside `DianaOSConsoleWindow` itself rather than `EmulationControlCommands.cs`, since it only ever needs that one window's own output, not anything on `MainWindow`.

**The welcome banner (`GetWelcomeBanner`, `DianaOS/DianaOSInterpreter.cs`).** Printed exactly once per shell launch - not on every command, not on every F4/console-reopen - a boxed header (`Welcome to DianaOS v0.1a`), a "Supported cores" list, a short "Features" bullet summary, and a closing line pointing at `help`/`man` (`Type "help" for a list of commands. Man is supported for each.`) rather than dumping the full command listing inline - an earlier version did that (via the private `Help()` method), but repeating the whole registry on every launch buried the actual orientation content above it. Version is `DianaOSInterpreter.Version` (`"0.1a"` today) - a public constant so a frontend could surface it elsewhere (a window title, an about box) without needing to know it's really just banner detail. `supportedCores` is a plain `IEnumerable<string>` parameter rather than anything hardcoded in this core-agnostic library - `EmuSen.Hotaru`'s `RunStandaloneShell` passes the distinct display names out of its own `_coreRegistry`, `EmuSen.Mistress`'s `DianaOSConsoleWindow` passes its own small `SupportedCores` array (mirroring `PreferencesWindow.AvailableCores`, a separate hardcoded copy that already existed for that window's own core-selection combo - not worth unifying for one display string). Box-drawn with plain ASCII (`+`/`-`/`|`), not Unicode box-drawing characters or ANSI color - has to render identically in a real terminal (`EmuSen.Hotaru`) and a plain Avalonia `TextBox` (`EmuSen.Mistress`'s console window), and the latter has no ANSI interpreter to strip escape codes with.

**Not yet done, explicitly out of scope for this pass**: shell functions (`name() { ...; }` - the single biggest additional lift, deferred as a follow-up rather than attempted alongside everything else here), true per-command-ephemeral variable scoping, arithmetic expansion (`$((...))`  - a literal `$((` currently lexes as `$(` immediately followed by a mostly-inert nested `(...)`, which will not do what a bash user expects, since there's no numeric-expression evaluator behind it), and per-stage (rather than whole-pipeline) redirection. **Also absent at the lexer level, each a deliberate omission rather than an oversight:** heredocs (`<<`), backtick command substitution (only the modern `$(...)` form is supported), brace expansion (`{a,b}`), tilde expansion, and background jobs — a bare unpaired `&` throws a clear "not supported" error rather than silently misbehaving. There is no filesystem-of-interest to expand a glob against either, so globbing is not implemented. Each of these is flagged once here rather than re-litigated at every call site that could theoretically want one.

**Known limitation**: `ConsoleLineReader` anchors its redraw column to wherever the cursor was when it started (right after the `DianaOS #: ` prompt) and doesn't handle a line long enough to wrap past the terminal width, or the terminal scrolling mid-edit. An accepted gap for a "basic" implementation - not something normal debug-prompt usage is likely to hit.

### 3.17a Five things `DianaOSInterpreter` does that the grammar above does not explain

*2026-08-16. `DianaOSInterpreter.cs` carried 347 lines of comment across 40 blocks. Almost all of it restated §3.3 and §3.17 — the bash subset, the timeout, the source-depth cap, `help` versus `man`, the banner — and is deleted rather than moved. These five were only ever recorded in the file.*

**`break`/`continue` are implemented as exceptions, and that is not the antipattern it looks like.** `BreakSignal`/`ContinueSignal` are private exception types thrown by `ExecuteStatementList` and caught by `ExecuteFor`/`ExecuteWhile`. A tree-walking interpreter has no other clean way to unwind an arbitrary depth of nested `StatementList`/`If` frames back to the nearest enclosing loop — short of threading a completion status through every execute method and checking it at every level, which is the same mechanism written out by hand and easier to get wrong. This is the shape a real interpreter for a language with these constructs almost always converges on. Neither signal ever escapes past the loop methods.

**A bare top-level `continue` means "resume gameplay", not bash's no-op** — and the path it takes there is not obvious. Inside a loop, `continue` never reaches `Dispatch` at all: the parser recognises it as a loop-control keyword and the signal above unwinds it. So the *only* way `continue` arrives at the dispatcher is when it is typed with no enclosing loop, which is exactly the interactive "I am halted at a breakpoint" case. `SubmitCore`'s catch for an uncaught `ContinueSignal` sets `HostAction.Resume` there, matching this shell's convention that `continue` at a prompt resumes (`ResumeCommand`, §3.3's command table). The two behaviours cannot collide, because the loop case is consumed before dispatch and the prompt case only exists outside a loop.

**Alias normalisation happens in `Dispatch`, immediately before the registry lookup, and deliberately excludes `continue`.** `c`→`resume`, `quit`→`shutdown`, `s`→`step`, plus `jobs` and `hexdump`, are rewritten to their canonical names rather than registered as duplicate entries — one registry entry per alias would make `help` print the same `Usage` line two or three times. `continue` is **not** in that list precisely because of the paragraph above: it is a parser-level keyword, and normalising it would route the in-loop case through the dispatcher it is supposed to bypass.

**`_pendingHostAction` is a field, last-one-wins, read once at the end of `SubmitCore`.** A host action is delivered after a whole statement list finishes, not mid-list, and if several commands in one line produce one the last wins — the same rule `$?` already follows for exit codes. It is deliberately *not* threaded through the `(string, int)` return type of `ExecuteStatementList` and its five callers: that would be a signature change rippling through six methods to carry a signal only `SubmitCore`'s caller ever reads. It is reset immediately before each execution, so a stale value cannot leak from one `Submit` into the next.

**An unquoted word that expands to nothing vanishes entirely, and a quoted one never does.** `ExpandWord` tracks which characters came from a quoted span, so `"pre$X post"` stays one argument even containing a space, while unquoted `$X` holding `a b` becomes two. An entirely-quoted word — including an explicit empty `""` — always yields exactly one argument even when empty; an entirely-unquoted word expanding to nothing yields **zero**. That is why `echo a $EMPTY b` prints `a b` rather than `a  b` with a phantom empty argument in the middle. `ExpandWordSingle` is the deliberate exception, used where bash itself suppresses word-splitting regardless of quoting: assignment right-hand sides and redirection targets.

### 3.17b The grammar, and the lex/parse split

*2026-08-16, from `Lexer.cs`, `Parser.cs` and `Ast.cs` (133 lines across 24 blocks). The grammar in particular was reference material sitting in a comment.*

**Hand-written recursive descent**, matching this codebase's style everywhere else a dispatch table or interpreter loop is hand-built (`Cpu.cs`, `Spc700.cs`) rather than taking a parser-generator dependency for a deliberately small grammar:

```
Script        := StatementList EOF
StatementList := sep* (Statement (sep+ Statement)*)? sep*     -- sep = ';' | Newline
Statement     := IfStmt | ForStmt | WhileStmt | 'break' | 'continue' | AndOrList
AndOrList     := Pipeline (('&&'|'||') Pipeline)*
Pipeline      := ['!'] SimpleCommand ('|' SimpleCommand)*
SimpleCommand := (Assignment | Word | Redirection)+           -- at least one
IfStmt        := 'if' StatementList 'then' StatementList
                 ('elif' StatementList 'then' StatementList)*
                 ('else' StatementList)? 'fi'
ForStmt       := 'for' NAME 'in' Word* sep 'do' StatementList 'done'
WhileStmt     := ('while'|'until') StatementList 'do' StatementList 'done'
```

**"Not finished" and "wrong" are two different failures, and the split is load-bearing.** `ShellIncompleteException` is thrown when the parser runs out of tokens where it needed more — `if true; then echo hi` with no `fi` is not broken, it is unfinished — and `ShellSyntaxException` when a token *is* present and is simply wrong. `DianaOSInterpreter` handles the first exactly as it handles `LexResult.NeedsMoreInput`: hold the raw text, show the `> ` continuation prompt, wait for another line. Collapsing the two would make every half-typed block an error. A block parser reaching EOF with no terminator is the incomplete case; `terminators.Length == 0` happens only for the top-level script list, where EOF just means done.

**The lexer never expands anything.** It tags. A bash word is rarely one uniform string before expansion — `pre$VARpost"lit $OTHER"end` is a *single* word made of five differently-sourced pieces — so `Ast.WordPart` records each piece with where it came from, and `DianaOSInterpreter.ExpandWord` (§3.17a) applies the quoting rules afterwards. That tagging is precisely what makes real bash semantics expressible instead of an all-or-nothing per-word quoted flag: single quotes produce a `Literal` part that is never expanded and never split; double quotes still produce `Variable`/`CommandSubstitution` parts, expanded but not split.

A `CommandSubstitution` part holds the **raw, unparsed source text** between `$(` and its matching `)`, parsed and executed recursively at expansion time rather than at lex time — so a substitution's own contents, which may themselves contain quotes, pipes or further nesting, never have to be understood until the outer word actually needs a value. A bare `$` not followed by a valid name, `(`, `{` or `?` is a literal dollar sign, as in bash. An empty `''` still registers as real, empty word content, which is what makes it produce one argument rather than zero (§3.17a).

**Redirection attaches to the pipeline, not the stage**, and an assignment is only an assignment in command position, unquoted, with a valid identifier — `cmd NAME=value` after the command name is a literal argument, matching bash. A statement of nothing but assignments is itself valid.

### 3.18 `EmuSen.WiseMan` — committed xUnit test project

Named `WiseMan`, not `EmuSen.Tests` - see `CLAUDE.md`'s own naming note (WiseMan is this project's internal codename for its AI coding assistant). A permanent, committed home for verification that used to be reinvented as throwaway scratch code every session and discarded at the end of it - shell/debug-toolchain dispatch, the core-agnostic `IDebugTarget` hooks (`ClassifyStaticReference`, `DecodeTilePixels` - §3.1, §3.12), and the `ICore` audio drain path (`EmuSen_Frontend_Driver.md` §3) all started life exactly that way before landing here as real, `dotnet test`-runnable tests.

```
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj   # or: dotnet test EmuSen.sln
```

**`Fixtures/SyntheticRom.cs`** builds a minimal, synthetic (never real game data - `VenusCore`/`Cartridge` insist on a real file on disk, so there's no way to hand it ROM bytes directly) LoROM: NOP-filled (not zero-filled - zero decodes as `BRK`, a 2-byte 65816 instruction, and an odd-length gap of them between two placed instructions misaligns a linear disassembly scan before it reaches the second one) with named `(offset, bytes)` patches placed at known addresses, then loads a real `VenusCore`/`EmulatorSession` from a uniquely-named temp file. `LoadCore`/`LoadSession` are the two entry points every test class actually calls.

**`Fixtures/DianaShellFixtures.cs`** - the shell-testing counterpart to `SyntheticRom.cs`: `NewShellWithSessions()` builds a `DianaOSInterpreter` + `DianaOSSessionManager` pair with the initial session already registered (the three-line `CreateSession`/`RegisterInitial` dance `TmuxCommand`/`PsCommand`/`UserdelCommand`'s own tests all need), and `ScratchDir` is an `IDisposable` real directory under `DianaOSSandbox.RootDirectory/var/log/<subfolder>/run_<guid>/` (`var/` is already `.gitignore`d - see `man hier`) for any test exercising a sandboxed filesystem command (`cat`/`cp`/`find`/`xxd`/...) - a real file has to live inside that sandbox to be reachable at all. Both used to be reinvented as a near-identical private nested class in `PsKillCommandTests.cs`, `UsersCommandTests.cs`, `FilesystemCommandTests.cs`, and `XxdCommandTests.cs` before being pulled out here.

**What's covered today:**
- `DianaOS/DianaOSDispatchTests.cs` - every registered shell/debug command, dispatched against a null `IDebugTarget`, must not let an exception escape `DianaOSInterpreter.Submit` (it already catches everything internally and renders `Error: ...` text - a thrown exception here would mean that guard itself broke).
- `DianaOS/ScriptingTests.cs` - `source`/`.`: variable-scope sharing with the caller, comment/blank-line skipping, multi-line control flow inside a sourced file, history exclusion, nested `source`, the self-referential recursion guard, unterminated-block recovery, and `DianaOSSandbox` enforcement (outside-the-project and missing-file paths). Scratch script files live under `var/log/WiseManScriptingTests/` (`var/` is already `.gitignore`d) rather than `Path.GetTempPath()` the way every other WiseMan fixture writes scratch files, since `source` is one of the commands `DianaOSSandbox` walls to the project's own directory tree - a script has to physically live inside it to be sourceable at all.
- `DianaOS/ManPageTests.cs` - a `[Theory]` over every registered command name AND every special-cased builtin (`export`/`source`/`if`/... - see §3.17's own list) asserting `man <name>` returns a real page, not the "no detailed manual page yet" fallback - the regression test that catches a newly-added or renamed command quietly falling through to that fallback and staying that way. Also: `help` with no argument lists everything, bare `man` produces identical output to `help` (the one intentional overlap), `help` ignores arguments and always shows the same full listing rather than forwarding to `man` (unlike an earlier, since-reverted revision), and an unknown `man` name reports a clean error with a nonzero exit code (checked via `$?`, since `Submit` doesn't surface an exit code directly). Its registered-command-name list used to be a hand-maintained copy of `CreateDefault`'s own command list, kept separately on the reasoning that `IDianaOSCommand` instances weren't otherwise enumerable from outside the interpreter - but that copy silently fell out of sync with `CreateDefault` repeatedly as commands were added (`cp`/`cat`/`head`/`tail`/`touch`/`find`/`xxd`/`tmux`/`ps`/`kill`/`whoami`/`who`/`su`/`useradd`/`userdel`/`passwd` had all quietly stopped being covered by the one test whose entire job is catching exactly that). Fixed by adding `DianaOSInterpreter.CommandNames` (a public `_orderedCommands.Select(c => c.Name)` projection) and reading the list from a live interpreter instead - the class of bug is now structurally impossible rather than just periodically re-noticed. The special-builtins list (`man`/`help`/`export`/... - genuinely not `IDianaOSCommand` instances) still has to be maintained by hand, since `CommandNames` can't see them.
- `DianaOS/FilesystemCommandTests.cs`, `DianaOS/XxdCommandTests.cs`, `DianaOS/PsKillCommandTests.cs`, `DianaOS/UsersCommandTests.cs` - `cat`/`head`/`tail`/`touch`/`cp`/`find`, `xxd`/`hexdump`, `ps`/`jobs`/`kill`, and `whoami`/`who`/`su`/`useradd`/`userdel`/`passwd` respectively (see `EmuSen_Project_Overview_v2.md`'s DianaOS section for what each command does). `UsersCommandTests.cs` gives every test its own GUID-suffixed username rather than a fixed one, since `DianaOSUserRegistry` is static, process-wide state shared across the whole assembly's test run (parallelization is already off assembly-wide, so there's no race - just cross-test leakage to avoid). `Fixtures/DianaShellFixtures.cs` (below) is the shared scratch-directory/session-manager setup all four of these (and `ScriptingTests.cs`) now use instead of each maintaining its own near-identical private copy.
- `DianaOS/HardwareLoadTests.cs` - `IDebugTarget.GetHardwareLoad()`/`MaxSprites` (§3.1, backing `coretop`): no frame-timings delegate reports "not modeled" (an empty list, not fake zeros); a delegate's raw millisecond timings are normalized against a 60fps frame's ~16.67ms budget (half-budget timings read as ~50%); a frame that ran a full 3x over budget is clamped to 100% rather than reporting 300%; and `MaxSprites` is the real 128-entry SNES OAM capacity.
- `DianaOS/DianaOSDispatchTests.cs`'s override-by-name coverage - a fake extra command sharing a default command's `Name` (`echo`) replaces it without `DianaOSInterpreter.CreateDefault` throwing on a duplicate key, and an unrelated default command (`regs`) is untouched by an unrelated override - the mechanism `EmuSen.Mistress`'s windowed `coretop` replacement (below) depends on.
- `DianaOS/CoretopWindowFlagTests.cs` - `coretop -w`'s headless-testable surface: with no `openWindow` opener wired up, `-w` reports "not supported by this frontend" rather than silently running the terminal dashboard; with one wired up, `-w` calls it with the exact target instead of ever touching the console; no ROM loaded reports "No ROM loaded" before the flag is even checked; and `-W`/`-w` are recognized case-insensitively regardless of position in `args`. The actual window (`EmuSen.Hotaru`'s `DebugWindows`/`CoretopWindow`, `EmuSen.Mistress`'s `CoretopWindowCommand`) can't be exercised headlessly, same limitation `coretop`'s own console rendering already has.
- `DianaOS/StaticReferenceClassificationTests.cs` - `IDebugTarget.ClassifyStaticReference` (JSR/STA/LDA absolute classify correctly; direct-page STA has no static target) plus `callers`/`writers`/`readers` end to end against a real target.
- `DianaOS/TilePixelDecodeTests.cs` - `IDebugTarget.DecodeTilePixels` (bpp 2 decode correctness, unsupported bpp throws) plus `tile` end to end.
- `Audio/AudioDrainTests.cs` - `ICore.DequeueAudioSamples`/`EmulatorSession`'s pass-through (FIFO order, capping, empty-buffer behavior, the null-session-before-`LoadRom()` fallback).
- `Audio/AudioPlayerTests.cs` - `EmuSen.Mistress`'s `AudioPlayer` against SDL's real "dummy" audio driver (a genuine SDL backend built for headless testing, not a mock) - this is what actually proves `SDL3-CS`'s exact method/struct shapes (`OpenAudioDeviceStream`'s parameter order, `AudioSpec`'s field layout, the `AudioS16LE`/`InitFlags.Audio` constant names) are right, since compiling doesn't catch a P/Invoke marshaling mismatch. It is also what caught, on the SDL3 migration's first run, that the dummy driver's pacing is looser than SDL2's - see `EmuSen_Settings_Reference.md` §4.10.
- `Audio/WavFileTests.cs` - `EmuSen.Audio.WavFile`'s RIFF/WAVE/`fmt `/`data` header fields for a known sample-rate/channel-count input, plus the zero-samples edge case (header only, no data chunk).
- `Imaging/BmpFileTests.cs` - `EmuSen.Common.Imaging.BmpFile` write/read round-trips preserve pixels and dimensions exactly.
- `Imaging/FrameHashTests.cs` - `FrameHash.Compute` (FNV-1a) determinism and single-byte sensitivity - the property `--autoshot`'s whole "did this frame change" mechanism depends on.
- `Imaging/ContactSheetTests.cs` - `ContactSheet.Downsample`/`WriteContactSheet`'s grid/tiling dimension math across a few count/cols combinations, including the "fewer thumbnails than fill the last row" case.
- `HeadlessDebug/HeadlessDebugOptionsTests.cs` - `HeadlessDebugOptions.Parse`'s argv handling: usage/frame-count errors, `--tap`/`--tap2`/`--screenshot`/`--watch`/`--cpulog` spec parsing, and the fixed unknown-`--flag` warning routing (§3.15's revision note).
- `HeadlessDebug/FrameRunnerTests.cs` - `FrameRunner`'s safety-cap enforcement (and its generalized warning wording), `Hold`/`Release`/`Tap` state transitions against a real loaded `VenusCore`'s latched controller registers, the post-increment `onFrameAdvanced` callback numbering, and `WaitStable`'s "must see a real change before a quiet streak counts as settled" behavior - the same bug its own code comment documents having caught once.
- `DianaOS/IsReadOnlyClassificationTests.cs` - `IDianaOSCommand.IsReadOnly` correctness on representative commands, including the mixed-verb (`watch`/`snapshot`/...) and indefinitely-blocking (`coretop` raw-terminal) cases that conservatively report `false`.
- `DianaOS/FastPathClassificationTests.cs` - `DianaOSInterpreter.TryGetReadOnlyFastPath`: single bare read-only commands classify true; mutating, compound (`|`/`&&`/`;`), control-flow, unterminated-construct, and `!`-history-reference lines all classify false.
- `DianaOS/LexerTests.cs`, `DianaOS/ParserTests.cs` - direct characterization coverage of `Lexer.Tokenize` and `Parser.Parse` themselves, which until now had none at all (everything above reaches them only indirectly, through `Submit`). See §3.18a for what they pin and why.
- `DianaOS/HostActionPlumbingTests.cs` - `DianaOSResult`'s `HostAction` field surfaces via `Submit`, does not leak out of `$(...)` command substitution, and does propagate out of and halt a `source`d script.
- `DianaOS/ResumeShutdownStepCommandTests.cs` - `ResumeCommand`/`ShutdownCommand`/`StepCommand` and their aliases (`c`/`continue`, `quit`, `s`) each signal the right `HostAction`.
- `DianaOS/CoreCommandTests.cs` - `CoreCommand`'s validation logic (unknown core name, missing file, wrong extension) - pure string/file-existence checking, directly unit-testable with no window.
- `DianaOS/StateCommandTests.cs` - `StateCommand`'s `save`/`load [path]` parsing against fake `Action<string>`/`Func<string>` callbacks standing in for a real `ICore`/`EmulatorSession`.
- `DianaOS/WelcomeBannerTests.cs` - `GetWelcomeBanner()`'s content (version, supported cores, the `help`/`man` pointer line, and that the full command listing is NOT dumped inline) and that it never pollutes `History`.
- `Imaging/FrameImageWriterTests.cs` - `FrameImageWriter.SavePng` round-trips a synthetic RGBA buffer through a real PNG encode/decode with pixels intact.
- `Input/HotaruKeyMapTests.cs` - `HotaruKeyMap`'s default Avalonia-`Key` bindings, asserted directly against the map rather than duplicated as a second hardcoded table.
- `Serenity/BuiltInShadersTests.cs` - the `Scanlines`/`Crt` SkSL source strings compile-shaped-correctly (uniform declarations present, no obviously malformed GLSL-ism carried over from the old Raylib shaders).
- `Serenity/FramePresenterEffectCyclingTests.cs` - `FramePresenter.NextEffect`'s `None → Scanlines → Crt → None` cycle (F8's underlying logic).
- `Serenity/GameFrameControlTests.cs` - `GameFrameControl.ComputeLetterboxRect`'s pure aspect-fit-and-center math across a few control/image size combinations.
- `Serenity/GameFrameControlRenderTests.cs` - real Avalonia render passes through `GameFrameControl`'s actual `Render()`/`DrawOp` path (`Avalonia.Headless` + `UseSkia`, driven via `HeadlessUnitTestSession.Dispatch` rather than `Avalonia.Headless.XUnit`'s `[AvaloniaFact]` - that package pins xunit v3, which collides with this project's xunit v2 tests; see `Serenity/TestAppBuilder.cs`'s own comment): a solid-color frame renders with the correct pixel color at its center, 120 consecutive frames render without throwing, the direct regression guard for the per-frame-`SKSurface` flicker bug fixed in `GameFrameControl.cs` itself (`EmuSen_Project_Overview_v2.md` §2a) - rendering one unchanged input frame 30 times in a row produces byte-identical output every time - the same three checks again with a shader (Scanlines/Crt) active, and `Scanlines_darkens_odd_output_rows_by_the_exact_documented_factor` - the exact-pixel-math regression guard for the local-matrix rewrite (§2a) that replaced the shader path's own offscreen upscale surface, not just "some" output changed. Added specifically because nothing before this exercised the real render path at all - only pure math/shader-source-string/enum-cycling tests existed, and it found a real bug on its first real run: the shader-active "many consecutive frames" test segfaulted the test host outright (`SKRuntimeShaderBuilder.Dispose()` corrupting a reused `SKRuntimeEffect` - see `EmuSen_Project_Overview_v2.md` §2a's own write-up; both that crash and the underlying per-frame-allocation design flaw are fixed now, not just papered over). **Known gap:** headless rendering has no live `GrContext` (no real GPU), so this can't reproduce a live-GPU/compositor-specific bug the way the original flicker only showed up under a real window - it catches rendering-logic and native-crash regressions, not driver/compositor interaction bugs.

**A real bug caught building `AudioPlayerTests.cs`, not a hypothetical one:** `Environment.SetEnvironmentVariable("SDL_AUDIODRIVER", "dummy")` does **not** reach the real libc environment on this runtime - verified directly by calling `getenv(3)` via P/Invoke immediately afterward and getting back an empty string. Every SDL-touching test failed with `IsAvailable == false` (SDL falling through to ALSA, absent in a sandboxed test environment, instead of the dummy driver) until switched to a direct native call instead. Flagged here in case the same shape of bug (`Environment.SetEnvironmentVariable` not being visible to a native library's own `getenv()` call in the same process) shows up again anywhere else native interop and environment variables meet.

**Both of the above are history as of the SDL3 migration (2026-08-01).** SDL3 exposes the driver choice as a hint, so the suites call `SDL.SetHint(SDL.Hints.AudioDriver, "dummy")` in a static constructor and never touch the process environment at all - `Audio/NativeEnvironment.cs` was deleted along with the problem it worked around (`EmuSen_Settings_Reference.md` §4.10). The paragraphs are kept because the underlying trap is not SDL-specific: `Environment.SetEnvironmentVariable` still does not reach the real libc environment a P/Invoked library reads, anywhere else that combination comes up. What follows describes the removed file.

**`Audio/NativeEnvironment.cs` - the fix, made portable rather than Linux-only.** The first version of this fix P/Invoked libc's `setenv(3)` directly and unconditionally - it worked, but this project ships native SDL/Raylib runtimes for Windows and macOS too (`win-x64`/`x86`/`arm64`, `osx`), and there's no `libc.so` to resolve on Windows; that P/Invoke would have crashed the moment this suite ran anywhere but Linux/macOS, a real "worked here, breaks on port" landmine caught before it shipped rather than after. `NativeEnvironment.Set` branches on `RuntimeInformation.IsOSPlatform(OSPlatform.Windows)`: Unix keeps the verified `setenv(3)` P/Invoke (macOS shares the same POSIX libc API - not independently verified on real macOS hardware, since this project's dev/CI environment is Linux-only, but expected to work unchanged); Windows calls `Environment.SetEnvironmentVariable` directly rather than P/Invoking anything, on the documented reasoning that SDL2 on Windows reads environment variables via the Win32 `GetEnvironmentVariable` API (not a CRT-module-cached `getenv()` - the actual, well-documented reason this class of bug usually bites on Windows), and .NET's `SetEnvironmentVariable` already calls the matching Win32 `SetEnvironmentVariableW` under the hood there - so there's nothing to work around on that platform. Also not independently verified on real Windows hardware for the same reason as the macOS note. If either assumption ever turns out wrong, the fix is a `kernel32` `SetEnvironmentVariableW` P/Invoke in the same file, same shape as the Unix branch - not a deeper redesign.

**Assembly-wide `[CollectionBehavior(DisableTestParallelization = true)]`** (`AssemblyInfo.cs`) - `AudioPlayerTests` opens/closes real SDL audio devices via native calls; SDL's global init/quit state isn't worth risking under concurrent test execution, and this suite is small enough that parallelism wouldn't meaningfully speed it up anyway.

**Not yet done:** the vendored `TestSuites/snes-tests` ROMs (`TestSuites/README.md`) aren't wired in here - `cputest`/`spctest` signal pass/fail by writing "Success"/"Failed" text into the tilemap and looping forever, not via a WRAM flag or a distinct halt address easy to probe directly, so an automated check would need either real screen/tilemap inspection or parsing the linker `.map` files already vendored alongside them to find the `success`/`fail` label addresses and checking CPU PC against those after running enough frames. A real integration test, not attempted in this pass - `EmuSen.Pharaoh`'s `--commands` scripting (§3.15) is the natural driver for it once someone picks this up.

---

### 3.18a Lexer/parser characterization tests (`DianaOS/LexerTests.cs`, `DianaOS/ParserTests.cs`)

89 tests pinning the exact current behavior of `Lexer.Tokenize` and `Parser.Parse` (§3.17's grammar). Before these, nothing in `EmuSen.WiseMan` referenced either class directly: every existing shell test reaches them only through `DianaOSInterpreter.Submit`, so a behavior change deep in the lexer would only surface as some unrelated-looking command test failing, if at all. 836 lines of hand-written recursive descent had no direct coverage at all.

These were written during an evaluation of porting the shell language to F#, as the safety net that port would have needed. **The port was evaluated and rejected** - see §3.18b's note on why - but the tests were always the valuable half and stand entirely on their own.

Deliberately **characterization** tests, not specification tests: they assert what the code does today, right or wrong. They consume only `Lexer`/`Parser`/`Ast` public shapes, never either class's internals, so they stay valid across any future reimplementation of the grammar.

`LexerTests.cs` renders each lexed `Word` as `Kind[q]:text` parts joined by `+` (`L`/`V`/`C` for Literal/Variable/CommandSubstitution, `q` for quoted), so one assertion pins part kinds, quoting flags, and word-splitting together rather than needing three. Covered: the three quoting rules and their interaction (`'...'` fully literal, `"..."` expanded but not word-split, unquoted both), backslash escaping inside and outside double quotes, all four expansion references (`$NAME`/`${NAME}`/`$?`/`$(...)`), `$(...)`'s depth tracking and quote-skipping, operator lexing including the two-character forms, `#` comments, the unsupported-construct errors (`&`, `<<`), and every `NeedsMoreInput` path.

`ParserTests.cs` covers pipelines, `&&`/`||` chains, `!` negation, all three redirection forms and their attachment, assignment-vs-literal-argument in and out of command position, `if`/`elif`/`else`, `for`, `while`/`until`, `break`/`continue`, and the `ShellIncompleteException`-vs-`ShellSyntaxException` split (§3.17's "not wrong, just not finished" distinction, which the F4 prompt's secondary `> ` prompt depends on getting right).

**Three non-obvious behaviors these pin, all found by a test failing against a reasonable-looking expectation rather than by reading the code:**

- **An escaped character lands in its own `WordPart`.** `\$HOME` lexes as `Literal("$", quoted)` + `Literal("HOME", unquoted)`, two parts, not one - `AppendLiteralChar` flushes the buffer whenever the `quoted` flag flips. Harmless today (both are literals, they concatenate to `$HOME`, and nothing expands either way), but part boundaries are observable through `Word.Parts`, so a port that "tidily" merges them is a real behavior change.
- **A mismatched closing keyword reports incomplete, not a syntax error.** `if true; then echo a; done` throws `ShellIncompleteException`, because `done` isn't in that block's terminator set (`elif`/`else`/`fi`), so it's consumed as an ordinary command word and EOF then arrives with the block still open. The user gets a `> ` continuation prompt rather than an error.
- **A stray `fi`/`done`/`then` is not a parse error at all.** Outside any block these parse cleanly as ordinary command words; the interpreter rejects them later at dispatch as unknown commands. Keywords are only keywords in the grammar positions that look for them - `Parser.Keywords` is consulted by `IsKeyword` at specific points, it is not a reserved-word ban. Worth stating explicitly because an F# port modeling tokens as a discriminated union with a `Keyword` case would naturally turn these into parse errors, which would be a silent behavior regression that no existing test would have caught.

---

### 3.18b Property-based tests (`EmuSen.WiseMan/Properties/`, CsCheck)

19 properties over four targets - `XorDeltaCodec`, both cheat codecs, `LinearResampler`/`DynamicRateControl`, and `RewindBuffer`/`StateSerializer` - each running 300-5000 generated cases, about a tenth of a second in total. Uses **CsCheck**, a C#-native property-testing library, alongside the example-based tests in the same project.

**Why property tests at all.** Several things here state their correctness as an algebraic law in prose - `XorDeltaCodec.cs` says outright that `Apply` is "in place, and its own inverse" - and an example-based test can only ever spot-check a law. A generator turns the prose into an executable claim and, on failure, *shrinks* the counterexample to something minimal enough to read.

**What each target's properties claim:**

- **`XorDeltaCodec`** - encode-then-apply reconstructs the target; apply is its own inverse; an unchanged state encodes to a zero-length delta; a length mismatch is rejected rather than silently truncating. Two separate round-trip properties on purpose: one over independently random byte pairs (the dense-diff case) and one over a baseline with a handful of bytes patched (the long-identical-run case the varint run-length encoding actually exists for). The first alone barely exercises the run path, since independent random bytes collide only 1 time in 256.
- **Cheat codecs** - the agreement property, which is the one that matters: `CanDecode` returning true must never promise a `Decode` that then throws. Both codecs share a `TryNormalize` and could drift apart. Run against 5000 arbitrary strings, not just well-formed codes. Plus: a well-formed code decodes to a 24-bit address, and separators are cosmetic.
- **`LinearResampler`** - output is always whole interleaved stereo frames; **output never leaves the range its own input spanned** (linear interpolation between two samples cannot overshoot either, so this catches a sign or index error a spot-check would sail past); a non-positive ratio throws; fewer than two samples yields nothing.
- **`DynamicRateControl`** - the ratio never departs further than `MaxDeviation` permits; a queue exactly on target asks for no correction; no target means no rate control; and **a fuller queue never raises the ratio**, which is the entire steering direction the mechanism exists to provide.
- **`RewindBuffer`/`StateSerializer`** - rewinding lands on a frame whose RAM matches the counter that produced it; rewinding the *whole* chain reconstructs the first capture byte-for-byte; and write-then-read is the identity over the full field graph. Driven by the same stand-in `ICore` shape `RewindBufferTests` uses, so these test the delta chain rather than the SNES.

**These were verified capable of failing.** A property that passes because it is vacuously true is worse than no test. Two checks: a deliberately false property (`a delta is always empty`) was run and CsCheck falsified it after 9 shrinks with the minimal counterexample; and the rewind property, which has a conditional early return that *could* have made it vacuous, was checked by flipping its comparison and confirming it fails. Both probes were removed afterward.

**A note on language, since the git history shows the detour.** These tests were first written in F# with FsCheck, as part of evaluating a wider port of the shell language and cheat codecs to F#. That port was completed, measured, and then **reverted in full**. The short version of why: the exhaustiveness benefit that justified it turned out to be much smaller than assumed (the old dispatch already threw a descriptive error naming the unhandled node - the gain was compile-time versus run-time, on a grammar that gains a statement type rarely), it contributed nothing to either of this project's stated goals (a second core is authored in C# against a C# `IDebugTarget`; a nicer frontend is Avalonia work), and it put `FSharp.Core` - 2.4 MB - into every frontend and the standalone shell. Re-running the same properties under CsCheck showed comparable generation and shrinking, so the test suite gained nothing from F# either. **The accuracy came from testing previously-untested code, not from the language.** What survived the detour, and was worth it: these properties, §3.18a's 89 characterization tests, and §3.18d's continuation tests.

### 3.18c Differential testing as a technique

Not a test file - a technique worth recording, because it was used twice during the F# evaluation and both times gave far stronger evidence than a test suite could.

The shape: recover the previous implementation from git, compile it beside the new one under a `Reference` namespace, and compare the two over a large generated corpus - not just return values, but *exception type and message text* as well. It proves a rewrite is a behavioural no-op rather than merely "passes the tests we thought to write".

Applied to the cheat codecs, it compared 400,000 Game Genie codes, 400,000 Pro Action Replay codes, and 200,000 arbitrary junk strings four ways each, plus `null` - 1.6 million comparisons, zero mismatches. Applied to the shell grammar, it compared full token streams, fully rendered ASTs, the `NeedsMoreInput` path and exception messages over 300,095 inputs (95 curated, 300,000 randomly assembled from shell metacharacters) - zero divergences. Worth reaching for on any future rewrite of something whose failure mode is quiet corruption rather than a crash; the codec case is the clearest example, since a subtly wrong decode poisons whatever ROM byte a mistranslated address lands on and looks like it worked.

### 3.18d Multi-line continuation tests (`DianaOS/ContinuationTests.cs`)

12 tests over `DianaOSInterpreter.Submit`'s buffering across calls - the mechanism behind the F4 prompt's secondary `> ` prompt and `source`'s multi-line block support (§3.17). Before these it had almost no direct coverage: one incidental `Assert.False(shell.IsAwaitingMoreInput)` in `ScriptingTests.cs`, despite being the path every interactively-typed `if`/`for`/`while` goes through.

Covered: an open quote or `$(...)` waiting then completing; `if`/`for`/`while` blocks buffering until their closing keyword; nested blocks finishing only on the *outermost* terminator; a genuine syntax error clearing the buffer rather than leaving the prompt stuck forever; an unsupported operator (`&`) clearing it via the lexer-throws path; a completed block recorded in history as one entry rather than per typed line; and a bare `break` outside any loop being silently ignored.

**One behaviour worth knowing, which these pin:** feeding the *wrong* closing keyword does not error - `done` typed into an open `if` is just an ordinary word, so the block stays open and the prompt keeps asking until a real `fi` arrives. That follows directly from §3.18a's third finding, and it means a mistyped terminator costs you a line, not the whole block.

---

### 3.19 Layer isolation and per-scanline registers (`layers`, `ScanlineRegisterDumpLine`)

Two diagnostics added for the Super Metroid landing-site investigation, both aimed at the same blind spot: **`regs` only ever shows end-of-frame state**, which is useless for any SNES scene driven by HDMA — and most interesting scenes are. A game can change BGMODE, TM/TS, CGWSEL/CGADSUB, scroll and windows on every single scanline, then leave the registers holding whatever the last line set. Reading `regs` there tells you about vblank, not about the picture.

- **`layers <spec>`** (`--commands` verb) — sets `DebugSettings.LayerEnableMask`, which is ANDed into both TM and TS inside `CompositeScreen`, so it isolates a layer on the main *and* sub screens at once. `<spec>` is `all`, `none`, `bg1`..`bg4`, `obj`, or a raw hex mask (`layers 13` = BG1+BG2+OBJ). Answers "which layer is that garbage actually in", which otherwise takes guesswork against a tilemap hexdump.

  **Caveat worth knowing before reading the output**: isolation happens at the layer-enable stage, *not* after color math. If the scene has color math on (CGADSUB non-zero), `layers bg3` still shows BG3 blended against whatever the main screen resolves to — which, with everything else masked off, is the backdrop. So a "BG3 only" shot is really `backdrop + BG3`, and if CGRAM[0] is black that happens to equal BG3's raw colors, but it will not in general. Cross-check against `pal 0` before treating an isolated shot as raw layer content.

- **`--flag ScanlineRegisterDumpLine=<n>`** — dumps BGMODE/TM/TS/CGWSEL/CGADSUB/MOSAIC/INIDISP, the fixed color, and BG1-3 scroll as scanline `<n>` is rendered. `-1` (default) is off; **`-2` sweeps every scanline**, which is the useful mode: pipe it through `uniq` on the fields you care about and you get the frame's entire HDMA profile in one run.

- **`scanregs <n|all|off>`** (`--commands` verb) — the same dump, reachable from a script instead of only a process-launch flag. `off` (default) disables it, `all` is the `-2` every-scanline sweep, and a number pins one scanline. Added because the flag form can only be set before the run starts, so a long scripted boot had to dump every frame from power-on to reach one screen of interest; `scanregs` can be switched on for a single `frames 1` once the script has already navigated somewhere and switched straight back off. That is how the DKC2 vertical-scroll off-by-one (`Venus_PPU.md` §2.4) was isolated — one frame of output at the exact moment of interest showed lines 0-6 carrying *identical* registers, which is what ruled out register/HDMA timing and pointed at the BG fetch itself.

  That sweep is what showed Super Metroid's landing site is: BG3-only with no color math for the 31-line HUD band, then `TM=13 TS=04 CGWSEL=02 CGADSUB=33` (BG1+BG2+OBJ on main, BG3 on sub, full additive) for the rest, with BG2's horizontal scroll stepped in ~32-line bands for the parallax mountains and BG3 scrolled equally in X and Y - i.e. diagonally, which is how the rain moves.

### 3.20 `perf` — per-frame cost profile

`perf [frames] [worstCount]` (`--commands` verb, defaults `300 5`) runs `frames` more frames and reports what each one cost. Added for the Rocky Rodent "why is the framerate low" investigation, because until it existed the only frame-cost readout in the project was `EmuSen.Mistress`'s once-a-second `FpsText`, which needs a live GUI to read — exactly what the harness exists to avoid.

It reports three things the FPS text can't:

- **A distribution, not just a mean.** `p50`/`p95`/`max` plus an explicit `over budget: n/N` count against `1000 / core.FrameRateHz` (16.64 ms, from the real 60.10 Hz — not 60). A scene averaging 8 ms with 11% of frames at 17 ms stutters; a mean alone hides that.
- **The core's own phase attribution**, straight from `VenusCore.LastFrame*Ms` (§ `VenusCore.cs`): cpu+spc700 / ppu / hdma, the ppu sub-split (mainComposite / subComposite / objEval / blend), and an **unattributed** column — wall minus the three phases. A large unattributed figure means the cost is outside `RunFrame`'s instrumented phases, which is itself the finding.
- **The worst individual frames, by offset into the window**, so a spike can be correlated with what the game was doing.

**Read the first `perf` in a process with suspicion.** Tiered JIT makes the opening ~20 frames materially more expensive, and they land in the "worst frames" list looking like a real spike. The Rocky Rodent run showed exactly this: p95 17.57 ms in the first window, all five worst frames inside the first 17, and 5.9 ms flat forever after. Put a `frames 300` ahead of the first `perf`, or discard that window.

#### What it found: Debug builds are ~3.6x slower, and it is the CPU-bound titles that fall under 60

Measured on this machine, gameplay, same scene, same `--commands` script:

| Config | RR ms/frame | fps | over budget |
|---|---|---|---|
| Release | 4.6-4.9 | ~210 | 0/300 |
| Debug | 16.8-17.0 | ~59 | 269/300 |
| Debug + `-p:Optimize=true` | 4.6-4.9 | ~210 | 0/300 |

The third row is the point: the entire penalty is the **JIT optimizer being off**, not `DEBUG`-conditional code. Nothing in the core is compiled differently between the two configurations — it is purely `Optimize=false`.

**`-p:Optimize=true` silently does nothing on an incremental build.** Changing that property does not invalidate MSBuild's up-to-date check, so `dotnet run -c Debug -p:Optimize=true` reuses the unoptimized assemblies and reports the plain Debug number — measured 2026-08-03: `EmuSen.dll` came back byte-identical, same timestamp, and the run was 19.0 ms against Release's 5.0. It needs `-t:Rebuild` (then `--no-build` on the run), after which it matches Release to a tenth of a millisecond. **The failure mode is the dangerous direction**: the flag looks applied, the number looks like a real finding, and the conclusion is "the optimizer is not the problem." It also fails the other way — a forced optimized Debug build stays optimized until something else forces a rebuild, so a later plain `dotnet build -c Debug` can hand back a fast Debug binary and hide the cliff entirely.

Rocky Rodent matters because it is the heaviest title in the sample and therefore the first to cross the line. In Debug: RR 16.9 ms (over the 16.64 ms budget), FFVI 12.4, SMW 12.2, LttP 11.5, DKC 10.5, SM 8.0. In Release every one of those, plus CT/MMX/SoM/EB, sits between 1.9 and 4.9 ms — 3-9x headroom. So a Debug-built frontend is not "a bit slower", it is running with roughly a 1.02x margin on the heaviest game and a 1.4x margin on the next one down: any further renderer cost makes more titles fall under 60, one at a time, which is what "it's an issue in another game too" looks like from the outside.

**Re-confirmed 2026-08-03 on Kirby's Dream Land 3, which is now the worst case.** Reported from outside as "in game levels we are only getting 51 fps", and the harness reproduced it to a tenth on the same in-level scene:

| Config | KBL3 ms/frame | fps | over budget |
|---|---|---|---|
| Release | 4.93-5.03 | ~201 | 0/900 |
| Debug | 19.34-19.69 | **50.8-51.7** | 900/900 |
| Debug + `Optimize=true`, forced rebuild | 4.88-5.00 | ~202 | 0/900 |

KBL3 displaces Rocky Rodent as the title the cliff hits hardest — RR dropped to 15.21 ms once its wasted sub-screen composite was fixed (below), and KBL3 is heavier still. The reason is a different one, and it generalises: **`cpu+spc700` is 13.2 ms of KBL3's 19.4**, because it is an **SA-1** game and that means two 65816 cores stepping every frame, with `perf`'s own coprocessor line reading `357368 clocks/frame run of 357368 offered (100.0%)` — the game keeps the SA-1 saturated. RR was a renderer problem; KBL3 is a CPU-bound one, and no renderer work will move it. The Debug cliff scales with whichever phase dominates, so the first games under 60 are now the SA-1 titles.

**A frame-rate report from a frontend is a build-configuration question before it is a core question.** The core measured 5.0 ms on the same scene — a 3.3x margin — while the frontend was reporting 51 fps; the most recently built frontend assembly was in `bin/Debug`. Check which configuration is running *before* profiling anything.

Two ways to not hit it, with the tradeoff stated rather than picked:

- **Run Release builds.** Zero cost, but no debugger.
- **`<Optimize>true</Optimize>` in `EmuSen.csproj` only**, leaving the frontends unoptimized. Full-speed emulation while still stepping through frontend code — but the core is precisely where most debugging happens, and optimized code has unreliable locals and inlined frames. Not applied.

#### Follow-up: most of Rocky Rodent's cost was work being thrown away

The table above frames RR as legitimately "the heaviest title in the sample," with the Debug JIT cliff as the whole story. A second pass with the phase attribution showed that was only half right. RR's `subComposite` figure was 1.95 ms against a 2.08 ms `mainComposite` — a full second layer composite per scanline, where every other game in the sample sat far below its own main-screen cost or at a flat 0.00 ms.

The cause was not the game being demanding. RR runs mode 2 with `TM=$17 TS=$17 CGWSEL=$30 CGADSUB=$00` — every layer enabled on the sub screen, with color math switched off entirely — so the renderer composited a sub screen that the final blend then discarded, on all 224 scanlines of every frame. `Venus_PPU.md` §5.1 has the fix and the output-identity verification.

Corrected numbers on the same gameplay scene: Release 5.59 → 3.51 ms, Debug 23.74 → 15.21 ms (42 → 66 fps, from 300/300 frames over budget to 0/300). RR is no longer the outlier — it now sits alongside SMW and FFVI rather than 65% above them.

**The transferable lesson is about reading `perf` output, not about this one game.** A phase that is large *in absolute terms* is not a finding; a phase that is large *relative to the same phase in comparable scenes* is. `subComposite ≈ mainComposite` was visible in the very first RR profile and read as "this game does a lot of work," when the comparison across ten ROMs made it obvious no other title behaved that way. Always profile a suspect game against a cohort, not against its own budget.

---

### 3.21 `framesum` — output-identity digest

`framesum [frames]` (`--commands` verb, default `300`) runs `frames` more frames and FNV-1a-folds each frame's `FrameHash` into one running 64-bit digest, reporting it as a single hex number.

It exists because §3.20 turns performance into a number you can compare, but says nothing about whether a change that made things faster also changed what was drawn. Screenshots and `contactsheet` sample a handful of frames; `framesum` covers every pixel of every frame in the window at a cost of one hash per frame.

**The workflow it's for** — proving a renderer optimization is invisible:

1. Capture the digest for a window on every ROM you care about, on the current code.
2. `git stash` the change (or apply it) and re-run the identical script.
3. `diff` the two lists. Any differing line names the ROM to go look at.

This is how §5.1 of `Venus_PPU.md` (skipping the unused sub-screen composite) was verified: 400-frame windows across 20 ROMs, plus 2500-frame windows across 8 color-math-heavy ones, all byte-identical either side of the change. Without it, "the frame rate went up 37%" and "the picture is still right" would have been two separate acts of faith.

**Caveats.** The digest is order-sensitive and window-sensitive — both runs must start from the same state (same ROM, same `--loadstate`, same preceding verbs) and cover the same frame count, or the numbers differ for reasons that have nothing to do with the change under test. It tells you *that* something differs, never *what*: once a ROM's digest moves, fall back to `--autoshot` or `contactsheet` to find the frame. And it is a rendering check only — it says nothing about audio, timing, or save-state contents. For audio, see §3.22.

### 3.22 `audiosum` — output-identity digest for audio

`audiosum [frames]` (`--commands` verb, default `300`) is §3.21's counterpart for sound. It runs `frames` more frames and FNV-1a-folds every sample the DSP actually produced into one 64-bit digest, alongside activity statistics:

```
[AUDIOSUM] 900 frame(s) to frame 900: ACB024E02AB2A512 samples=958426 nonzero=881546 (92.0%) peak=10383 rms=2298.5
```

**Why it drains rather than snapshots.** `audiodump` (§3.15) writes a `.wav` from a *non-destructive* peek at whatever is currently queued, so it can only ever see the tail — in a headless run nothing consumes the audio queue, and the core's own safety valve starts discarding the oldest samples once it passes `AudioSettings.AudioBufferMaxSamples` (~2 seconds). `audiosum` calls `DequeueAudioSamples` every frame instead, so the digest covers the entire window rather than its last two seconds.

**Why the statistics are there and not just the hash.** A digest can only say "different." It cannot distinguish a fix from a regression that muted a voice — silence hashes to a perfectly stable, perfectly wrong number (every all-zero window in the sample produced the identical digest `B3BB38A76F1DFDCD`, which is how three ROMs with no music yet at frame 900 were spotted immediately). `nonzero`/`peak`/`rms` give the change a direction. Pair it with per-voice `KeyOns` from `channels` (§3.5) when the question is "are note events being lost," since that counts events rather than energy.

**The workflow it's for** — proving an APU change does what you think, on more than the one ROM that motivated it. This is how `Venus_APU.md` §3.3.1 (the KON latch) was verified across 37 ROMs: 29 came out sample-identical, and the 8 that changed all moved the same way — more key-ons, more non-silent output, higher RMS. A cohort run that shows *only* the expected direction of change is much stronger evidence than one game sounding better.

**Caveats.** Same order- and window-sensitivity as `framesum`, and the same "tells you *that*, never *what*" limit — when a digest moves, `audiodump` a narrowed window and listen, or use `mute` (§3.5) to isolate voices. Note that it consumes the queue: a script that runs `audiosum` and then `audiodump` will find little or nothing left to dump.

---

### 3.23 Cartridge coprocessors (`regs`, `cophist`, `GSURAM`/`SA1IRAM`, `IDebugTarget.CoprocessorRegisters`)

**The gap this closed.** Until this landed, the entire toolchain was blind to cartridge coprocessors. `SnesDebugTarget` had no reference to the SA-1, the SuperFX GSU or a NEC DSP at all, and the exposed memory spaces stopped at `CpuBus / IO / WRAM / VRAM / CGRAM / OAM / SRAM / APURAM`. So `regs`, `watch`, `framelog`, `waitvalue`, `snapshot`, `diff`, `search` and `coretop` — the whole surface — could not see a single GSU register. Every coprocessor investigation therefore degenerated into hand-patching `Console.WriteLine` into the interpreter and rebuilding, twice over in the SuperFX work (`Venus_SuperFX.md` §10). This is the same blind spot `APURAM` closed for the SPC700, and the same lesson §2 already states: a narrow, investigation-specific print is a sign something belongs in the toolchain.

**What is exposed.** `IDebugTarget.CoprocessorRegisters` is a provider like `CpuRegisters`/`ApuRegisters`, publishing whichever chip the cartridge carries — the GSU's `SFR`/`PBR`/`CBR`/`SCBR`/`SCMR`/`ROMBR`/`RAMBR` plus the whole `R0`-`R15` file, the SA-1's control registers and its 65C816's state, or a NEC DSP's `PC`/`SR`/`DR`/`DP`/`RP`. A cartridge with no coprocessor — most of them — publishes an empty list, and `regs` prints no section at all. Memory spaces appear only when the chip is present: **`GSURAM`** (Game Pak RAM: the GSU's work RAM, framebuffer and save data all at once, so `snapshot`/`diff` over it answers "is the chip still plotting"), **`SA1IRAM`**, **`BWRAM`**, **`SA1BUS`** and **`DSPRAM`** — the last three added later, see §3.23b.

**Reads must not perturb the chip, and this is not a formality.** The real register windows have side effects — reading `$3031` *acknowledges the GSU's interrupt*, and the NEC DSP's `DR`/`SR` reads advance its transfer handshake. So the provider reads dedicated side-effect-free `Debug*` views rather than routing through each chip's own `ReadRegister`, exactly as `APURAM` wraps raw SPC700 RAM instead of `Spc700.Read8`. `CoprocessorDebugExposureTests.Reading_the_provider_does_not_acknowledge_the_gsu_interrupt` pins it, and genuinely fails if the implementation is rerouted through `ReadRegister`.

**`DSPRAM`.** The NEC DSP's RAM is `ushort[]`, so a byte-addressable view has to pick an order; it presents **little-endian**, matching how the chip's own 16-bit words reach the S-CPU over `DR`. Writes recombine into the existing word rather than clobbering the other half.

### 3.23b Coprocessors as first-class debug targets

§3.23 made the chips *visible*. This made them *debuggable* — the difference being that a second CPU needs the same verbs the first one has, pointed at its own address space. Full design detail: `Venus_SA1.md` §11.

**`BWRAM` — a space that was silently lying.** BW-RAM is the SA-1's main work and save RAM. The `SRAM` space is anchored at `$70:0000`, and an SA-1 cart maps BW-RAM at `$40-$4F`, so `mem SRAM` decoded an unmapped bank and reported **32KB of zeroes** on Kirby's Dream Land 3 while the game was actively using it. Worse than a missing space: a present one that answers confidently and wrongly. `BWRAM` wraps the array directly.

**`SA1BUS` — the chip's own 24-bit address space**, Super MMC banking applied, side-effect-free (`DebugPeekRegister` answers `$2302`/`$230D` from their latches instead of re-latching and advancing them). This is what makes the rest of the toolchain work on coprocessor code for free:

```sh
regs                       # Coprocessor block: PB/PC of the second CPU
disasm SA1BUS 82D7 12      # what it is actually running
```

**`disasm` picks its decoding CPU from the space name.** Immediate widths come from M/X/E, and decoding SA-1 code with the *S-CPU's* flags mis-lengths every immediate — `LDA #$0001` read as `LDA #$01` desynchronises the stream and the remainder of the listing turns into garbage. `SA1BUS`/`SA1IRAM` decode with `Sa1.Cpu`.

**`watch` now sees both sides.** `MemoryBus.Write8` routes cartridge writes straight to `Cartridge.Write8` without notifying its observer, so writes to `SRAM`, `SA1IRAM` and `BWRAM` fired no watch at all — on any game, not just SA-1 ones. `Cartridge` carries its own `WriteObserver` now, and the SA-1 reports its own writes through `IWriteObserver.OnCoprocessorWrite` (a defaulted member) so events are labelled with the chip's PC rather than the S-CPU's, which is meaningless for them:

```
      24x  W  SA1 PC=0xC22698     <- the coprocessor
      16x  W  PC=0xC22709         <- the S-CPU
```

**`bp sa1` — breakpoints on the second CPU.** `Breakpoints` and `CoprocessorBreakpoints` are separate registries, because the two CPUs run different code at the same addresses. The scope word is optional, so existing forms are untouched:

```sh
bp add 8000            # S-CPU, exactly as before
bp sa1 add 0082D7      # the coprocessor
```

`EmuSen.Pharaoh`'s `FrameRunner` stops the batch when either CPU halts and says which one, so this works headlessly. A halted frame is deliberately **not** counted, snapshotted for rewind, or reported as advanced — `RunFrame()` returned mid-frame, so that frame did not happen:

```
> frames 30
[BREAK] SA-1 halted at $0082D7 (frame 0).
```

**`perf` reports whether the chip is running at rate** (see §3.20 and `Venus_SA1.md` §2.3):

```
  sa-1: 357368 clocks/frame run of 357368 offered (100.0% of the 357368 a full-rate frame allows)
```

A shortfall means starvation or halted time; a surplus means double-clocking. It settles a coprocessor's contribution to a pacing complaint in one line rather than by inference.

### 3.23a `cophist` — coprocessor register history

`regs` shows the instant. Coprocessor bugs are *transitions* — the chip renders, then stops — so the question that actually gets asked is "what were `SFR`/`PBR`/`R15` doing in the seconds before it stopped", which no latest-value view can answer.

`CoprocessorRegisters` is therefore backed by `EmuSen.Cauldron.HistoryProvider<T>` (§4) rather than `PollingProvider<T>`, retaining the last 600 refreshes (~10 seconds of frames). `cophist [<reg>] [<count>]` reads it back, oldest row first:

```
> cophist R15 10
Coprocessor history: 10 of 301 retained (capacity 600),
unchanged for 15 refresh(es).
  refresh     R15
      291    B2B1
      ...
      300    B2B1
```

The header's **"unchanged for N refresh(es)"** is the useful part: it is the direct answer to "when did the chip stop", available without storing or diffing anything by hand. The run above shows the GSU parked at `R15 = B2B1` for fifteen frames.

### 3.24 `cov` — execution coverage

`bp` answers "is execution at this address *right now*". It cannot answer "did this code ever run", and a breakpoint that never fires proves nothing on its own — it looks identical to a breakpoint on an address that is never reached and to one that was set wrong.

`cov` records every 24-bit address executed between `cov on` and `cov off` into a bitmap, and reports coverage over any range:

```
> cov on
> frames 300
> cov off
Coverage recording off, 4233705 instructions recorded.
> cov 10F452 10
Coverage $10F452-$10F461: 0/16 bytes executed
  Never reached.
```

Recording is off by default and costs one bool test per instruction while disarmed; armed, it allocates a 2MB bitmap. An optional `cop`/`sa1`/`gsu` scope word targets the coprocessor's own instruction stream, which is a separate address space — same convention as `bp sa1` (§3.23b).

**Three things it does that nothing else here did.**

- **Retires "the game never reaches this feature" in one command.** `Venus_SuperFX.md` §10.1 retired two suspects by patching a `Console.WriteLine` into an opcode handler and running headless to see whether it fired. That is the right instinct and the wrong mechanism — it needs a rebuild per question, and the probe has to be removed afterward.
- **Pairs with `callers` into a mechanical search.** Walk up from a routine that never ran until you reach a caller that did; the branch between the two is the one that skipped it. Neither half works alone: `callers` finds paths that exist without saying which ran, and coverage says what ran without saying what could have.
- **Recovers instruction boundaries `disasm` guessed wrong.** The static disassembler has to assume the CPU's *current* M/X flags apply at the address being decoded (see `SnesDebugTarget.Disassemble`'s own comment), so a 16-bit `LDA #$7000` in a routine disassembled while M is set renders as two shorter instructions. The recorded addresses *are* the real opcode boundaries, so `cov <routine> <len>` checks a decode without tracing it.

---

### 3.25 `memfind` — locate a byte sequence

`search` (§3.x) matches one 1/2/4-byte scalar and then narrows the candidate set over successive scans. That is the right shape for "where does the game keep the life counter" and the wrong shape for "where did this block of bytes come from", which is a single-pass question about a sequence.

```
> memfind WRAM 00FFFFFFFF00FF00FF11FF11FF11FF11
No match for 16-byte pattern in WRAM.
> memfind VRAM 00FFFFFFFF00FF00FF11FF11FF11FF11
1 match(es) for the 16-byte pattern in VRAM:
  0x2760
```

Separators (`,`, `:`, `-`, `_`, whitespace) between byte pairs are ignored, so a pattern pasted straight out of a `mem` dump works unedited. `??` in place of a pair matches any byte — the way to skip the parts of a block that legitimately differ between two copies. It stops after `<max>` hits (default 20) and says it truncated, so a pattern too short to be distinctive answers immediately instead of printing thousands of offsets. Same live-hardware-space refusal as `search`/`dump`.

The negative result is as useful as the positive one. In the Super Mario World title-screen investigation (§3.26) the 16 bytes of a tile were present in VRAM and *absent* from all 128KB of WRAM, which said immediately that the staging buffer had already been reused — so chasing it in a post-hoc dump was never going to work, and the question had to be asked at the moment of the write instead.

### 3.26 `bp write` — halt on data, not on control flow

`watch` records a write and lets the machine run on; `bp add` halts on an address being *executed*. Neither answers "what state produced this write" — the write's own PC is in the watch log, but by the time anything can read registers or memory the machine has moved on by thousands of instructions.

`bp write <space> <addr> [<value>]` halts the core the moment the write happens, so `regs`, `mem` and `disasm` all read the state that caused it:

```
> bp write VRAM 2769 11
Breakpoint #1 added on writes to VRAM 0x2769 = 0x11.
> frames 500
[BREAK] S-CPU halted at $00AAD4 (frame 277).
> regs
  A    = 0x11FF
  PC   = 0xAAD4
> mem WRAM 0 10
  000000: 90 B2 7E ...
```

That three-byte direct-page pointer (`$7E:B290`) is the whole answer: it is the source address the uploader was reading from, and nothing short of stopping at the write could have produced it. Repeating the same trick one level down — `bp write WRAM B291 11` — lands in the decompressor that filled that buffer, which is how a VRAM byte gets traced back to a compressed stream in ROM.

Two details worth knowing:

- **The halt lands one instruction late, on purpose.** A write happens part-way through an instruction; stopping there would leave the CPU mid-instruction with no consistent state to read. `BreakpointRegistry.NoteWrite` arms a pending flag that the existing per-instruction `ShouldBreak` consumes at the next boundary, so the reported PC is the instruction *after* the store — exactly the convention a hardware watchpoint uses.
- **Optional value matching is what makes it usable on a busy address.** A VRAM byte inside a tile is written on every upload of that tile; `bp write VRAM 2769 11` fires only on the upload that wrote the byte being investigated.

The data breakpoint also exposed a real gap in the halt path, now fixed: `regs`/`sprites`/`pal` read `IRealtimeProvider` snapshots refreshed once per *completed* frame, so at a mid-frame halt they reported the previous frame's end state — a breakpoint reporting stale registers is worse than no breakpoint. `FrameRunner.OnHalted` now refreshes the providers before the `[BREAK]` line is printed.

### 3.27 `eval` and conditional breakpoints (`DianaOS/Lib/ExpressionEvaluator.cs`, `DebugTargetExpressionContext.cs`, `Cores/.../Debug/SnesExpressionContext.cs`)

Added after a gap-analysis pass against Mesen's debugger (`Core/Debugger/`, cloned to `/home/red/Projects/mesen-reference` as a read-only reference) — see §3.33 for the full comparison and what was deliberately left out.

An expression language over live machine state, and the thing that turns a breakpoint on a hot routine from useless into precise. `bp add 80A31C` on a routine that runs 400 times a frame stops on the first one; `bp add 80A31C if x == 7` stops on the one that matters. Everything else in this section exists to serve that.

**Split along the core-agnostic line the rest of §3 already uses.** The language — tokens, precedence, short-circuiting, `[addr]`/`{addr}` memory forms — lives in `EmuSen.DianaOS` and knows nothing about any CPU. What a name *means* is an `IExpressionContext`, which each core implements:

- `SnesExpressionContext` publishes the 65816's registers, its individual P flags as `flag.c`/`flag.z`/…, `frame`/`scanline`/`cycle`, `opaddr`, `stackdepth`, and every label.
- `DebugTargetExpressionContext` is the **zero-per-core fallback**: it derives symbols from whatever registers a target already reports through `CpuRegisters`/`VideoRegisters`/`ApuRegisters`/`CoprocessorRegisters` (i.e. whatever `regs` prints) and reads memory through `GetMemorySpaces()`. A second core gets a working `eval` and working conditional breakpoints without writing a line of expression code; publishing its own context is an upgrade, not a prerequisite.

`DebugCommandHelpers.ExpressionsFor(target)` picks the target's own context when it has one and the fallback otherwise, so no command has to care which it got.

Three decisions worth recording because the obvious alternatives are wrong:

- **Short-circuiting genuinely skips, it does not just discard.** `ParseBinary` threads a `live` flag: a skipped branch is still *parsed* (tokens have to be consumed) but nothing in it is evaluated. That matters beyond `100 / [$7E0000]` not dividing by zero — an unevaluated `[...]` must not *read the bus*, because reading a hardware register has side effects (`RDNMI` clears the pending-NMI flag). A first cut of this evaluated both sides and threw the result away, which the short-circuit tests caught.
- **WRAM reads bypass the bus.** In `SnesExpressionContext.TryReadMemory`, a CPU address landing in WRAM (banks `$7E`/`$7F`, or the `$00-$3F`/`$80-$BF` low-RAM mirror) is served straight from `bus.Ram` rather than through `Read8`. It's faster on a path that runs per-instruction, and it makes the overwhelmingly common condition form (`[$7E0020] == 3`) side-effect-free by construction. Anything not resolving to WRAM still goes through the live bus, with whatever consequence that carries — `man eval` says so explicitly rather than pretending otherwise.
- **A condition that won't evaluate is dropped, not retried.** A typo'd symbol would otherwise fail on every single instruction the address is reached at, forever. `ConditionHolds` halts once, records the error in `LastConditionError`, and clears that breakpoint's condition — the breakpoint survives, its condition doesn't. A core with no `ConditionEvaluator` wired treats conditions as always-true, so the address still works.

`%` is both the binary-literal prefix (`%1010`) and modulo. It's read as a literal only when the next character is `0` or `1`; everywhere else it's the operator. Found by a test, not by inspection.

Note the shell interaction: `&&`, `||`, `<`, `>`, `|` are DianaOS's own operators (§3.17) and are consumed before the command ever sees them, so a condition using them must be quoted. Everything after the `if` word is rejoined with single spaces, so simple unquoted forms still work.

### 3.28 Execution control: `bt`, `step over`/`step out`, `runto`

Before this, the halt/resume surface was a single-instruction `step` and a breakpoint list. That covers "stop here" but not the two questions that actually dominate a session: *how did I get here*, and *get me to the interesting moment*.

**`bt` — the live call chain.** `CallStackRegistry` records a frame on every JSR/JSL and pops on every RTS/RTL, plus a labelled frame on NMI/IRQ/BRK/COP entry (and a pop on RTI). This is the dynamic counterpart to `callers` (§3.12) and the two answer different questions: `callers` scans code and lists every instruction that *could* reach an address, `bt` reports the one path that actually did. When a routine has six static callers, `callers` gives six suspects and `bt` gives the answer.

The seam is two nullable fields on `Cpu` (`CallStack`, `Breakpoints`), attached by `SnesDebugTarget` and null in a normal run — the same "core notifies a registry it knows nothing about" shape `BreakpointChecker` and `CoverageRecorder` already use, and both are `[SkipInState]` for the same reason `_verboseTrace` is.

**A tracked call stack is an inference, and the doc says so.** Code that manipulates its own stack — pushing a return address and jumping, pulling one it never returns through, unwinding several frames with a stack-pointer write — desynchronizes it. Rather than pretend otherwise, unmatched returns are *counted* and surfaced by `bt` whenever nonzero, depth is capped so a runaway chain stops recording instead of growing without bound, and `bt reset` resyncs from the current instruction. That honesty is what makes `step out` and `profile` trustworthy: when the reading is wrong, there is a visible reason.

**`step over`/`step out` work by depth, not by address.** `ArmStepToDepth(n)` halts at the first instruction executed once the call stack is no deeper than `n` — `step over` passes the current depth, `step out` passes depth − 1. Two things fall out for free: recursion is handled correctly (an inner call at the same address doesn't stop it early), and `step over` on a *non-call* instruction degrades to a plain `step`, because the depth never rises. An address-based implementation gets both of those wrong.

The one subtlety is why arming works at all: `VenusCore.RunFrame`'s `_justResumedFromBreakpoint` flag already skips exactly one breakpoint check after a resume, so the instruction being stepped over runs before any check happens. The existing single-step depends on the same property.

**`runto` — resume until an event.** `runto nmi|irq|brk|cop` (fed by `BreakpointRegistry.NoteInterrupt` from the CPU's interrupt entry points), `runto scanline <n>` (a new `MemoryBus.ScanlineObserver`, invoked once per scanline from `RunFrame`), and `runto frame [<n>]` (the existing per-frame `OnFrame` hook). All three set a pending flag consumed at the next instruction boundary — the same one-instruction lag `bp write` has, for the same reason: an event fires somewhere the core cannot safely stop.

`runto nmi` + `bt` + `label add opaddr NmiHandler` is the intended three-command sequence for finding and keeping a handler address you didn't previously know.

`runto <addr>` on a plain address is a convenience that adds an ordinary breakpoint and resumes. It is **not** removed when it fires — the registry has no one-shot concept — so the command reports the id and tells you to remove it. Silently leaving a permanent breakpoint behind would be worse than saying so.

### 3.29 `label` — named addresses (`DianaOS/Var/LabelRegistry.cs`)

The piece that makes a long investigation compound instead of restarting. Every prior section produces addresses; nothing kept them. `label add 7E13C6 CoinCount` names one, and from then on `disasm`, `bt`, `bp list`, `counters top` and `profile top` all print `$7E13C6 <CoinCount>`, and `CoinCount` works as a symbol in any expression (`bp add NmiHandler`, `eval [CoinCount]`).

`disasm` prints a label on its own line above the instruction it names, and annotates any operand whose statically-resolvable target (via the existing `ClassifyStaticReference`, §3.12) is itself labelled — so a `JSR` to a named routine says which one without a second lookup.

The map is one-to-one in both directions: renaming an address drops its old name, repointing a name moves it. Two entries pointing at the same thing is always a mistake, so it's made unrepresentable rather than documented around.

`label load`/`label save` use a deliberately trivial format (`<hex address> <name> [comment...]`, `#` comments, blank lines ignored) rather than any one assembler's symbol file — nothing here knows which assembler a given ROM was built with, and a real `.sym`/`.mlb` converts with one `awk` line, which this shell already has (§3.17). Bad lines are reported with line numbers and the good ones still load.

### 3.30 `counters` — per-address access tallies (`DianaOS/Var/AccessCounterRegistry.cs`)

`watch` records individual events with context; `cov` records whether an address ever executed. Neither answers *how many times*, which is the question behind "is this table live or dead", "which byte of this struct does the game actually touch", and "which half of this buffer is being updated". `counters` tallies reads, writes and executes per address for one space.

Armed against one space at a time on purpose: the tallies cost four arrays the size of that space, which is worth paying where a question is being asked and not worth paying everywhere. Same "cheap when disarmed" contract `CoverageRegistry.Record` already has — one bool test on every memory access.

**Uninitialized-read detection** is the part worth having beyond raw counts. An address read before anything wrote it (since arming) is counted separately. On hardware that read returns power-on RAM contents, so a game doing it is either relying on that state or has a real bug — and an emulator whose RAM fill differs from hardware diverges exactly there. `counters top u` finds those addresses directly. The "since arming" qualifier does real work: arm before the moment being studied, or an address initialized earlier looks uninitialized.

`counters cold` is the inverse and answers what `cov` answers for code but nothing answered for data: addresses in a range nothing ever touched.

### 3.31 `freeze` — pin an address (`DianaOS/Var/FreezeRegistry.cs`)

Holds an address at a value by writing that value straight back the instant anything writes something else. Deliberately **not** the same thing as `cheat poke` (§3.14): a cheat re-applies once per frame, so the game's own value is live for most of that frame and every read in between observes it. A freeze undoes the write immediately, so nothing ever sees what the game tried to store. When the question is "what breaks if this counter never changes" or "is this the variable driving that animation", a per-frame poke answers neither cleanly.

Fed from the same `IWriteObserver` seam `watch` uses; the restore writes through `IDebugMemorySpace`, so the registry needs no knowledge of how a space is stored. The restoring write is itself a write, so `FreezeRegistry.Restore` wraps it in a reentrancy guard — without that it reports straight back into itself.

Writes matching the frozen value are left alone and not counted, which makes the per-entry "writes undone" count meaningful: **a freeze with a blocked count of zero means nothing is writing the address you suspected**. That doubles as a cheap negative result, in the same spirit as §3.24's "a breakpoint that never fires proves nothing, but coverage that never records does".

### 3.32 `profile` — where the *game's* instructions go (`CallStackRegistry`)

Charges every executed instruction to whichever routine is innermost on the call stack, then ranks routines. Note the scope: this profiles the **game**, not the emulator — `perf` (§3.20) and `coretop` cover EmuSen's own frame cost, and the two are unrelated tools that happen to share a word.

Two deliberate limits:

- **Exclusive, not inclusive.** A routine that spends all its time inside a call it made scores low; the callee scores high. Inclusive attribution would compound any call-stack drift (§3.28) across every enclosing frame, where exclusive attribution localizes it to one.
- **Instructions, not cycles.** Instructions are what the per-instruction seam can count exactly. A cycle figure would have to be attributed across a boundary the seam doesn't see, and for "which routine is hot" the two rank almost identically.

Instructions executed with nothing on the stack are charged to `(outside any recorded call)` — normal, and it means the code was already running when profiling was armed, not that anything is broken.

### 3.33 What these passes took from Mesen, and what they left

The Mesen source (`SourMesen/Mesen2`, `Core/Debugger/`) was cloned as a read-only reference and its debugger feature set compared against this toolchain command by command. Everything above came out of that comparison. What was already covered, and what was consciously *not* built:

| Mesen | Here |
|---|---|
| `ExpressionEvaluator.*` | §3.27 `eval`, `bp ... if` |
| `CallstackManager` | §3.28 `bt` |
| `StepRequest` StepOver/StepOut/RunToNmi/RunToIrq/PpuFrame/SpecificScanline | §3.28 `step over`/`step out`, `runto` |
| `LabelManager` | §3.29 `label` |
| `MemoryAccessCounter` (incl. `BreakOnUninitMemoryRead`) | §3.30 `counters` (as a tally, not a halt) |
| `FrozenAddressManager` | §3.31 `freeze` |
| `Profiler` | §3.32 `profile` |
| `MemoryDumper`, `Disassembler`, `PpuTools`, `BaseTraceLogger` | already `mem`/`dump`, `disasm`, `tile`/`tilemap`/`sprites`/`pal`, `trace` |
| `Breakpoint` (`_startAddr`/`_endAddr` ranges, `BreakpointType::Read`, `BreakpointType::Forbid`) | §3.36 `bp add <a>-<b>`, `bp read`, `bp forbid` — the last pass claimed `bp` covered this and it did not |
| `MemoryAccessCounter::BreakOnUninitMemoryRead` | §3.36 `bp uninit` — §3.30 had the tally, not the halt |
| `CodeDataLogger` / `CdlManager` (`.cdl` files, `GetFunctions`, statistics) | §3.36 `cov funcs`/`cov save`/`cov load` — the last pass claimed `cov` covered this and it did not |
| *(nothing — Mesen has no equivalent)* | §3.36 `cov mark`/`cov new`, differential coverage |
| *(nothing — Mesen has no equivalent)* | §3.36 `bp write ... changed`, and `bp depth` |
| `DebuggerFeatures::ChangeProgramCounter`, `LuaApi::SetState` | §3.37 `setreg` — the debugger could read every register and write none of them |
| `DebuggerFeatures::CpuVectors` | §3.37 `vectors` |
| `LuaApi::ConvertAddress` | §3.37 `addr` |
| *(nothing — Mesen needs a Lua script for this)* | §3.37 `bp ... log <expr>`, logpoints |
| `LuaApi`/`ScriptManager` | the shell itself (§3.17) plus `EmuSen.Pharaoh` (§3.15) — a second scripting language isn't wanted |
| `CpuType` / per-CPU `IDebugger` (Snes/Spc/Gsu/NecDsp/Cx4/St018) | §3.34 `cpus` and the scope word every command takes |
| `ExpressionEvaluator.Gsu/.NecDsp/.Spc` (per-CPU token sets) | §3.34 `eval <cpu>`, derived from each chip's reported registers |
| `SpcDisUtils`, `NecDspDisUtils` | §3.34 `Spc700Disassembler`, `NecDspDisassembler` |
| *(nothing — Mesen has no equivalent)* | §3.35 `copflow`, the coprocessor register-window log and poll detection |
| `BreakSource::BreakOnStp`/`BreakOnWdm`/`SnesInvalidPpuAccess`/`SnesReadDuringAutoJoy` | §3.38 `bp when` — the row above dismissed these as NES/GB-specific, and all four are 65816 or SNES ones |
| `NecDspDebugger` (breakpoints, step, callstack on the DSP) | §3.38 `bp dsp`/`step dsp`/`bt dsp`/`cov dsp funcs` — §3.34 recorded this as impossible for a reason that was not the real obstacle |
| `SnesEventManager` (the DMA half of the Event Viewer) | §3.38 `dma`, `dma log`, `dma stats` — the data as text, including the scanline column |
| *(nothing — Mesen needs a Lua script for this)* | §3.38 `bp when ... log`, a hardware condition recorded rather than halted on |

**Not built, and why:**

- **`BaseEventManager` (the Event Viewer)** — a per-frame log of register writes, DMA, NMI/IRQ and sprite-0 hits plotted at scanline/dot coordinates. The single largest remaining gap, and a genuinely good fit for a PPU-focused emulator. Left out because its value is mostly in the *2D plot*, and everything in §3 is text; it wants the GUI debug window (§7) that still doesn't exist. `runto scanline` (§3.28) plus `layers` (§3.19) covers the narrow "what changed at this scanline" case in the meantime.
- **`StepBackManager` (step back one instruction)** — needs a rewind ring at instruction granularity, not frame granularity. Real work in the core, not the toolchain, and it should be built on whatever `EmuSen_Rewind_And_FastForward.md` already establishes rather than beside it.
- **`Base6502Assembler`** — assemble text to bytes and patch it in. `cheat rompatch` plus `write` already covers patching; the missing half is only the mnemonic-to-bytes direction, and the disassembler's own verification pass (§7) should land first so the two agree by construction.
- **`DisassemblySearch`** — `disasm ... | grep` is the composable-Unix answer this shell was built for, and it already works.
- **`BreakOnBrk`/`BreakOnCop`/`BreakOnStp`/unofficial-opcode breaks** — `runto brk`/`runto cop` cover the two that matter on a 65816; the rest are NES/GB-specific in Mesen. ~~**Wrong**, and corrected in §3.38: `BreakOnStp` and `BreakOnWdm` are both 65816 opcodes, not NES/GB ones, and Mesen additionally carries two SNES-specific break sources this list missed entirely (`SnesInvalidPpuAccess`, `SnesReadDuringAutoJoy`). All four are now `bp when`.~~

**Cost.** The call-stack and profiler hooks sit on the per-instruction seam, which §13.1 of `Venus_PPU.md` warns about. Measured on a deliberately JSR/RTS-dense synthetic ROM (a 3-instruction subroutine called in a tight loop — far denser than real game code): **1.038 ms/frame with no debug target attached, 1.060 ms/frame with one attached**, i.e. ~2% while debugging and nothing measurable during normal play, since `Cpu.CallStack` is null unless a `SnesDebugTarget` exists. Profiling and access counting are additionally opt-in behind an `IsArmed` bool, matching `CoverageRegistry`.

---

### 3.34 Named debug CPUs — the coprocessor pass (`DianaOS/Var/DebugCpu.cs`)

The toolchain's model of "which processor am I debugging" used to be a boolean: `Breakpoints` and `CoprocessorBreakpoints`, `Coverage` and `CoprocessorCoverage`, with commands taking a hardcoded `sa1`/`cop`/`gsu` scope word. That does not describe the hardware. An SNES is **always at least two** processors — the 65816 and the SPC700 — and a cartridge can add a third. One bool cannot name three chips, and the SPC700, which is present in every single game, had no breakpoints at all.

`DebugCpu` replaces the pair with a list. Each entry names a chip and carries whatever that chip actually supports: a `BreakpointRegistry` (never null, so commands need no second null check), and optionally a `CoverageRegistry`, a `CallStackRegistry`, an `IExpressionContext`, a code space, a register provider, and a program-counter delegate. `IDebugTarget.DebugCpus` defaults to empty, so a core that publishes nothing is unaffected and the legacy `Breakpoints`/`CoprocessorBreakpoints` properties still work.

Every scoped command resolves its optional first word through one shared helper (`DebugCommandHelpers.ResolveCpu`), so the convention is identical across `bp`, `cov`, `bt`, `step`, `profile`, `eval`, `regs` and `disasm`. With no scope word, the first CPU in the list — always the main one — is used, so **every pre-existing command form is untouched**. `cop` survives as an alias for whichever cartridge coprocessor is present.

**What the SNES now publishes:**

| Chip | `bp` | `cov` | `bt` | `regs` | `disasm` | Notes |
|---|---|---|---|---|---|---|
| `cpu` | yes | yes | yes | yes | yes | 65816, unchanged |
| `spc` | **new** | **new** | — | yes | **new** | present in every game; had no breakpoints before |
| `sa1` | yes | yes | **new** | yes | yes | a 65816, so the call/return seam applies unchanged |
| `gsu` | **new** | yes | — | yes | yes | coverage-only before |
| `dsp` | yes | yes | yes | yes | yes | breakpoints/coverage/call stack added in §3.38; addresses are word indices |

Support is deliberately **not** uniform, and `cpus` prints per chip what is actually wired rather than letting a command fail obscurely later. The remaining absence is an honest one: the GSU and SPC700 have no stack-based call convention to infer frames from, so `bt gsu` says so instead of inventing them. *(The DSP row above said `bp`/`cov`/`bt` were impossible until §3.38 found the stated reason was wrong — see there.)*

**Halting.** Only one chip halts at a time. `VenusCore` tracks *which* (`HaltedCpu`, replacing the old `_haltedOnCoprocessor` bool), because the resume path has to arm the skip-one-check flag on the chip that actually halted — arming the S-CPU's flag for a GSU halt would let the GSU re-break instantly on the same PC, forever. Each chip's unspent clock budget survives the halt, so resuming continues the frame rather than restarting it.

**Two disassemblers were missing outright.** `disasm APURAM` previously decoded SPC700 bytes with the 65816 table and produced confident nonsense. `Spc700Disassembler` and `NecDspDisassembler` fill that in; the SNES now needs four decoders to cover itself (65816, SPC700, GSU, NEC DSP), and naming a chip picks the right one. `disasm DSPPRG` indexes program *words* rather than bytes, because the DSP's PC is a word index and that is the only number a user ever has to paste in.

**Per-chip expressions.** `eval gsu r14` and `bp gsu add X if r14 > $100` evaluate against the *GSU's* registers. This needed no new expression language: `DebugCpuExpressionContext` derives symbols from whatever registers a chip already reports through `regs`, so a chip gets a working conditional-breakpoint context for free, and a core-specific context is an enrichment layered on top rather than a requirement.

### 3.35 `copflow` — the coprocessor handshake log (`DianaOS/Var/RegisterFlowRegistry.cs`)

Neither this toolchain nor Mesen had anything for the seam where coprocessor bugs actually live.

Every cartridge coprocessor talks to the main CPU through one narrow register window, and the failure is almost never the chip's arithmetic. It is the conversation: the CPU writes a parameter block, kicks the chip, the chip works and sets a status bit, the CPU polls that bit and moves on. Any of those four steps can fail silently, and none are visible in a register dump — by the time you look, the moment has passed. `cophist` (§3.26) shows the run-up in *register* terms; this shows the *traffic*.

`copflow` logs the window: every read and write with its value, frame, and which side did it. `copflow tail` replays the recent conversation in order; `copflow stats` gives whole-run per-register tallies.

**Poll-run detection** is the part worth the code. `copflow poll` tracks the longest run of consecutive reads of one register whose value never changed, plus the run in progress right now. A long run is unambiguous: if a game read `$3030` ninety thousand times and always got the same byte, the game is not slow and the plot is not wrong — it is spinning on a status bit this core never updates. That points at the register model rather than the chip's logic, and it is a different bug from anything `regs` or `cophist` would surface. A write by either side always breaks the run, because a write means someone learned something and acted.

Core-agnostic: `RegisterFlowRegistry` knows nothing about the SNES, and the SNES feeds it from the single `CartridgeRegion.CoprocessorRegister` choke point in `Cartridge.cs`. Disarmed it costs one bool test per coprocessor register access, and nothing at all on a cartridge without a coprocessor, which never reaches the seam.


### 3.36 The third Mesen pass — ranges, reads, and the ROM map (`DianaOS/Var/{BreakpointRegistry,CoverageRegistry}.cs`)

A third comparison, run specifically to find what the first two had waved through. It found that **two rows of §3.33's table were wrong**, not merely incomplete: `Breakpoint` was recorded as already covered by `bp`, and `CodeDataLogger` as already covered by `cov`. Neither claim survived reading the Mesen headers next to ours.

**What `bp` was actually missing.** Mesen's `Breakpoint` carries a `_startAddr`/`_endAddr` pair and a `BreakpointTypeFlags` of Read/Write/Execute/Forbid. Ours carried one address and could only break on writes. Three real capabilities were behind that:

- **Ranges.** Any address argument now takes `<start>-<end>`. This is a bigger difference than it looks, because it changes what can be *asked*: a single-address write breakpoint requires already knowing which byte moves first, and `bp write WRAM 0100-01FF` does not. The halt message names both the address that matched and the range it belongs to, so the range answers where it landed rather than only that it landed.
- **Read breakpoints.** `bp read` was simply absent. `readers` (§3.12) scans for instructions that *could* read an address, and misses every access through a computed pointer — which on a SNES game is most of them. The runtime halt catches what the static scan cannot.
- **Forbid ranges.** `bp forbid <start>-<end>` suppresses every halt while the PC is inside it. The use is noise: the interesting breakpoint is usually drowned by one loud caller, and forbidding the NMI handler's range leaves only the game logic. Steps are deliberately exempt — `step` still works inside a forbidden range, or stepping into one would run away with nothing able to stop it. A pending data break is *dropped* rather than deferred when it lands in a forbidden range, since deferring it would just fire it one instruction later and defeat the point.

**And `bp uninit`.** Mesen has `BreakOnUninitMemoryRead` as a break source; §3.30 had built the same detection as a whole-run tally in `counters` and stopped there. The tally answers "did this happen", the halt answers "what was on the stack when it did", and only the second one finds the cause. Each address reports once and is then treated as initialized, or a boot loop over uninitialized memory halts on the same byte forever.

**What `cov` was actually missing.** Mesen's `CodeDataLogger` persists to `.cdl` files keyed on ROM CRC, classifies bytes, exposes `GetFunctions()`, and reports statistics. Ours was an in-memory bitmap that answered one question. Two of those gaps were worth closing:

- **`cov funcs`** lists every address a call actually landed on, with an entry count and whether it was reached by a call or an interrupt vector. These are observed entry points, not inferred ones, which is the whole value — a static scan finds routines reached by a literal JSR and misses every jump table. Fed by a new `CallStackRegistry.EntryPointObserver` delegate, so the discovery rides the call seam that already existed rather than adding a second one. The output is shaped to paste into `label`.
- **`cov save`/`cov load`** persist the bitmap and the routine table, and **load merges rather than replaces**, so maps from several sessions accumulate. No single sitting reaches every routine, and a map of a whole playthrough is worth much more than a map of one.

The third CDL feature, per-byte code-vs-data classification, was **not** built. It needs the CPU to tag each memory access with an operation type (Mesen's `MemoryOperationType`, set at every read site), which is a change to every access site in the 65816 rather than a debug-layer addition. Our bitmap also stores instruction *starts* rather than a per-byte code flag, which is strictly more information — it is where `cov` gets its ability to settle a mis-sized immediate (§3.24) — so the two are not the same structure with one filled in.

**Two things neither emulator has.** Both came out of asking what the Mesen features were *for* rather than copying their shape:

- **`bp write ... changed`** halts only when the byte written differs from the last one seen there. A plain write breakpoint on a per-frame state byte trips sixty times a second and tells you nothing; games rewrite the same value constantly. The comparison is against what the breakpoint last *saw*, not against memory, so it costs no read and cannot disturb a side-effecting address. The first write to an address always counts as a change, or a value written exactly once would never be reported at all.
- **`cov mark` / `cov new`** are differential coverage — the coverage analogue of `snapshot`/`diff` (§3.10). Mark, press the button, and every address `cov new` reports is code that ran *because of what you just did*. Whole-run coverage cannot separate that from the boot sequence and the per-frame loop, which together dwarf it. This is the sharpest tool in this section, because it turns "find the routine that handles X" — the most common reverse-engineering question there is — into a mechanical two-command procedure.

**And `bp depth <n>`**, which Mesen also lacks: halt as soon as the call stack gets deeper than `<n>`. Runaway recursion presents as a game that slows down and then misbehaves long after the actual defect, by which time the evidence is gone; this stops the machine while the chain is still on the stack and readable with `bt`. Fires once and disarms, so it does not retrigger on every instruction that stays deep.

**Cost.** Everything here rides seams that already existed. `bp read` is one added call in `SnesDebugTarget.OnRead`, alongside the watch and counter notes already there. Forbid ranges cost a `Count > 0` test per instruction. The uninitialized-read map is one bool per byte of one space, allocated only when armed. `cov`'s mark is a second 2MB bitmap, allocated only by `cov mark`. Entry-point discovery is a dictionary write per call, and only while coverage is armed. Nothing new touches the per-instruction path when disarmed.

**Still not built, deliberately**, and unchanged from §3.33: the Event Viewer (wants the GUI window), step-back (wants instruction-granularity rewind in the core), and an assembler (wants the disassembler verification pass first). Mesen's `CpuCycleStep`/`PpuStep` step types are also still absent — `runto scanline`/`runto frame` cover the cases that have come up, and a dot-granular step wants the same 2D view the Event Viewer does.
### 3.37 The fourth Mesen pass — writing state, and the log that does not stop (`setreg`, `vectors`, `addr`, logpoints)

The third pass audited the *breakpoint and coverage* rows of §3.33. This one looked at the parts of Mesen that are not in `Core/Debugger/` at all — `DebuggerFeatures.h`, and the `LuaApi` surface, which is a good inventory of what Mesen thinks a debugger should be able to do because every entry is something someone wanted to script.

**The debugger could not write anything.** `LuaApi::SetState` and `DebuggerFeatures::ChangeProgramCounter` are both about modifying CPU state, and nothing here could. `mem`/`write` could poke memory, `freeze` could pin it, `cheat` could patch ROM — but every register was read-only, in a toolchain that otherwise models eight processors' registers in detail.

`setreg` closes that, and the point of it is **counterfactuals**. Halt on the compare that gates a routine, flip the register it compares, resume, and watch the game go down the branch it did not take. Coverage and breakpoints can establish that a branch was never taken; only this can show what would have happened if it were. It also skips a hang — forcing PC past a spin loop lets a boot sequence run far enough to show what the *next* problem is, which is frequently worth more than the one you are stopped on.

Deliberately **not** folded into `regs` as a `regs set` subverb, despite that reading better. `regs` is `IsReadOnly => true`, which is what lets the always-live shell answer it instantly off the emulation thread (§3.3); a write subverb would force the whole command false, dragging every register dump onto the emulation thread's next frame — and writing a register from the shell thread would be a race besides. A separate non-read-only command keeps both halves honest. Values are refused rather than truncated when they do not fit the register's width, since a silently masked value is a wrong answer reported as a success. Writability is per chip and reported by `cpus`: the 65816, the SA-1 and the SPC700 take writes, the GSU and NEC DSP report registers through debug accessors with no write path behind them and say so.

**`vectors`.** Mesen carries a `CpuVectorDefinition` table per CPU; we had `runto nmi` but no way to ask where NMI actually goes. `vectors` dereferences the table live through the CpuBus and resolves each target through the label registry. Two details make it more than a convenience: it groups by CPU mode, because a 65816 has two complete vector sets and which one the hardware uses depends on the E flag at the instant the interrupt is taken — a game that initializes only the native table and then takes an interrupt in emulation mode jumps somewhere it never intended, and seeing both tables together is what makes that visible. And when coverage is recording, a vector whose target has actually executed is marked `(ran)`; a vector pointing at plausible code that never ran says the interrupt is not firing at all, which is a different bug from it firing and misbehaving.

**`addr`.** `LuaApi::ConvertAddress` exists because the CPU's view and the ROM file's layout are different coordinate systems, and every task that crosses between them needs the translation. `addr` reports it using the cartridge's own decode — the same path a real read takes — so it is correct for whatever mapper the ROM uses rather than assuming LoROM, and ROM offsets come back modulo the real ROM size, matching how an undersized ROM mirrors on hardware. Addressable results print a ready-made `mem` command underneath. A hardware register reports as a register rather than being given a fabricated offset, and an address nothing claims reports as unmapped — itself an answer, since a pointer that decodes to nothing was computed wrong.

**Logpoints, which Mesen can only do with a Lua script.** A breakpoint with a `log <expr>` clause evaluates the expression, records it, and *carries on*.

This is the one capability here that is not catching up to anything. It exists because halting is the wrong tool for a whole class of question. A routine that runs four hundred times a frame cannot be stepped through four hundred times, and halting even once changes the timing of everything downstream — so the act of measuring destroys what is being measured. `bp add 80A31C log "a, x, [$7E0DB3]"` records all four hundred at full speed, and the *pattern across calls* becomes visible instead of one arbitrary call being inspected in isolation. Mesen's answer to this is `RegisterMemoryCallback` from a Lua script; §3.33 declined to add a second scripting language on the grounds that the shell already exists, and this is part of making that argument true rather than merely convenient.

Implementation is almost entirely reuse. The expression side is the existing evaluator, rendered per-chip through the same `DebugCpuExpressionContext` (§3.34) that conditions already use, so `bp gsu add X log r14` reads the GSU's R14. Comma-separated parts render as `expr=value` so a line names what it is showing. Logpoints compose with `if`, in either order, since both clauses are parsed by one scanner. A logpoint whose expression fails records the error text in place of the value rather than being dropped the way a bad *condition* is — a condition that throws must be dropped because it would otherwise fire on every instruction forever, and a logpoint has no such failure mode. The log is a 4096-entry ring shared per processor, so entries from several logpoints interleave in the order they really happened. Data logpoints record at the moment of the access rather than one instruction later, so a logged write shows the value that was written.

**Cost.** `setreg`, `vectors` and `addr` are all on-demand and cost nothing while not being run. A logpoint costs what a conditional breakpoint costs — an expression evaluation per match — and nothing when no logpoint is set, since the log expression is null and the check is a null test on a path that had already matched an address.

**Checked and deliberately still not built.** Mesen's `TraceLoggerOptions` carries a `Condition` and a `Format` string; our `trace` remains count-only. A conditional trace is real, but it wants the condition evaluated inside the core's own verbose-log path — core surgery — and logpoints now cover the same need better for the case that actually comes up, since they record *chosen* values rather than every field of every instruction. `LuaApi::SetInput` (programmatic controller input) is genuinely missing and genuinely useful — several open questions in `Games_Tested.md` are input-related — but it belongs with the frontend's input plumbing rather than in a debug registry, and it is the one item from this pass parked rather than dismissed.

### 3.38 The fifth Mesen pass — what the hardware does wrong, and the chips as chips (`bp when`, the NEC DSP, `dma`)

Four passes had compared *commands*. This one compared two things a command list does not surface: Mesen's `BreakSource` enum, which is the complete inventory of reasons its debugger will stop the machine, and its per-coprocessor `IDebugger` implementations, which are where a chip stops being a register dump and becomes something you can step.

**`bp when` — halting on a hardware fault rather than an address.** Every breakpoint here answered "where". Mesen's `BreakSource` list is mostly answers to "what": `BreakOnStp`, `BreakOnWdm`, `BreakOnBrk`, `BreakOnCop`, `SnesInvalidPpuAccess`, `SnesReadDuringAutoJoy`. §3.33 waved this row away as "the rest are NES/GB-specific in Mesen", **which was wrong on both halves** — STP and WDM are 65816 opcodes, and two of the entries are SNES-specific ones that list never mentioned.

These are worth having because none of them are visible in the output:

- **`stp`** is the sharpest. A 65816 that executes STP is stopped until reset, and a game only reaches one by jumping somewhere it never meant to. The symptom is that the picture simply stops, with nothing to say where — and by then the PC is parked and the call stack is the only remaining evidence. `bp when stp` halts on the instruction itself, while `bt` still reads.
- **`ppuaccess`** and **`autojoy`** are the two where the emulator and the game disagree quietly. Real hardware drops a VRAM/CGRAM/OAM write made during active display, and returns a half-updated byte from a joypad register read taken mid-refresh. A core more permissive than the hardware runs such a game correctly and hides a real bug; a core less permissive breaks one that worked. Either way the first thing worth knowing is that it happened at all, and neither is reported by anything else here. `autojoy` in particular has open questions waiting for it — `Games_Tested.md`'s input entries are exactly this shape.
- Only the PPU **data** ports count for `ppuaccess` ($2104, $2118/$2119, $2122). The address ports latch fine mid-frame, so flagging them would be noise, and noise is what makes a condition get turned off.

**Log mode is the half that makes it usable.** A condition can be armed `log` instead, recording every occurrence and carrying on. This is not a convenience: a title screen may read the joypad early on every single frame, and halting on the first one tells you far less than seeing the pattern across a hundred. Logged conditions land in the *same* ring as logpoints (§3.37) and read back through `bp log`, so a condition and a logpoint interleave in the order they really happened rather than being two logs that have to be correlated by hand.

Core-agnostic, as usual: `BreakpointRegistry` holds named conditions and knows nothing about what they mean, `IDebugTarget.BreakConditions` lets a core publish its own list with descriptions, and `bp when` with no argument prints that list rather than a hardcoded one. A second core publishes its own vocabulary and the command needs no change.

**The NEC DSP is a real debug target now.** §3.34's table recorded the DSP as breakpoint-less, coverage-less and stack-less, with the reason given as "firmware runs from mask ROM this core steps as a block". Reading `NecDspDebugger.cpp` next to `NecDsp.cs` showed that reason was not the obstacle. The mask ROM makes the firmware unreadable *from the cartridge*, which a debugger does not care about; and the chip was never stepped as a block — `Run()` already calls `Step()` in a loop, one instruction at a time, which is all a breakpoint needs. The seam was the same `BreakpointChecker` pull-hook the SA-1 and GSU already had, and it took the same shape: consulted before the instruction, returns with the clock budget intact, `ResumeFromBreakpoint()` skips one check so `continue` leaves the breakpoint.

Two details are specific to this chip:

- **Addresses are word indices, not bytes.** The DSP's PC counts 24-bit instruction words, so `bp dsp add 100` is the 257th instruction and `disasm DSPPRG` indexes identically. Deliberately not multiplied by three to look byte-addressed: the chip's own PC is the only number a user has to reason about, and a second coordinate system would mean every value needed labelling with which one it was in.
- **The call stack is observed, not inferred.** The uPD7725 has a hardware stack with a real CALL/RET pair — jump types `$140`/`$141` push, the `$400000` opcode class executes-and-returns. The observers fire from exactly those two sites, so `bt dsp` reports frames the chip actually pushed. That feeds `cov dsp funcs` (§3.36), which is the point: **firmware nobody has source for can be mapped by running it.** Every routine a game actually calls shows up with an entry count, and that is not a static analysis anyone could have done instead.

One honest blind spot: `Run()` returns immediately while the chip is in its RQM idle spin (`Venus_NecDSP.md` §4.3), so a breakpoint placed inside those two instructions never fires. Correct — nothing worth stopping on is executing — but worth knowing before concluding a breakpoint is broken.

**`dma` — the Event Viewer's data without the Event Viewer.** On a SNES nearly everything the player sees arrives by DMA, and per-scanline effects are HDMA. So a large class of "rendering bug" is not one: the data never arrived, or arrived somewhere else, and the renderer faithfully drew what it was given. There was no way to ask about any of that. Mesen's answer is the Event Viewer's 2D plot, which §3.33 deferred for wanting the GUI debug window — but most of that plot's value is one column, and a text command can have it:

- **`dma`** prints the channel table: direction, B-bus write pattern, the destination register *by name* (`VMDATAL $2118`, not a bare number), source, length, and for HDMA the table pointer and line counter reached. An idle channel still shows its last configuration, which is usually the thing being asked about.
- **`dma log`** records every transfer with its frame **and scanline**. That column is the one carrying the plot's value: a VRAM transfer during vblank is normal and the identical transfer at scanline 100 is a bug, and only the log distinguishes them. It also solves what the channel table structurally cannot — a general DMA reconfigures and fires many times a frame, and the table only ever shows the last one. Pairs directly with `bp when ppuaccess`, which halts on one.
- **`dma stats`** totals the armed run per channel (split general/HDMA, because they cost very differently) and **per destination register**, which answers "where is the bandwidth going". A game slow in one scene and not another is usually uploading something enormous, and the destination table names it.

`DmaLogRegistry` is core-agnostic — a ring plus counters, knowing nothing about the SNES — and the destination naming goes through a new `IDebugTarget.NameDmaDestination`, since only a core knows what base its register byte offsets. The SNES reuses the PPU's own `$21xx` name table rather than carrying a second copy.

**Cost.** `bp when` is one bool (`AnyConditionArmed`) tested at each detection site before anything is computed, so an unarmed condition costs a branch on paths that were already branching. The DSP hook is one null test per DSP instruction, matching what the SA-1 and GSU already pay, and the CALL/RET observers are null unless a debug target is attached. `dma log` is a null field test at the end of each transfer — per transfer, not per byte — so a disarmed run pays nothing measurable and an armed one pays a ring write per block rather than per byte moved.

### 3.39 Mesen as a *running* reference, not a source reference (`EmuSen.WiseMan/Reference/`)

> **Superseded 2026-08-06 by §3.45, and two of its claims are now known false.** `MesenProbe.cpp` and `build-mesen-probe.sh` no longer exist; the probe is a policy layer plus a backend per emulator, and Mesen is one backend. More seriously, every memory dump described below was read out of a *running* emulation thread — see §3.45a — so the run-to-run reproducibility asserted here and in §3.40 did not hold for dumps. The VRAM/CGRAM conclusions survive (those spaces are quiescent at a frame boundary); anything resting on WRAM, GSU RAM, OAM or the screen buffer at a mid-action frame does not. Kept for the reasoning, not as instructions.

Every Mesen pass above (§3.33, §3.36, §3.37, §3.38) read Mesen's **source** and imported capabilities from it. This is a different thing: driving Mesen as a second emulator on the same ROM and diffing the two machines' state at the same frame. It exists because the Yoshi's Island missing-sprites bug could not be found any other way — see `Venus_SuperFX.md` §9 for that investigation end to end.

**What's in the directory.** `MesenProbe.cpp` is a headless `main()` linked against a Mesen2 checkout's `MesenCore.so`; it runs a ROM to a frame, pauses on the frame boundary, prints the GSU and PPU register state, and writes `mesen_{vram,cgram,oam,gsuram,screen}_f<frame>.bin`. `build-mesen-probe.sh` builds it, including the two things that make Mesen build unattended on a machine without SDL2/X11 development packages: the headers are fetched as RPMs and unpacked into a work directory (`CPATH`/`LIBRARY_PATH` plus a shim `sdl2-config`), and `STATICLINK=false` is forced because the stock link line wants `libstdc++.a`. `mesen-gsu-trace.patch` adds the one hook Mesen has no equivalent of — a per-GSU-instruction record of address, opcode and all sixteen registers. `gsudiff.py` aligns that against our own `SuperFxTraceCountdown` trace and reports the first divergence. `MesenReferenceTests.cs` consumes the dumps from inside `EmuSen.WiseMan`.

**Driving it past a menu (`--press`).** The probe originally registered a key manager that reported "nothing pressed" forever, which confined every comparison to whatever the ROM reaches on its own — for Yoshi's Island that is the attract-mode intro and nothing else. `--press <frame>:<button>[:<duration>]` (repeatable; `A B X Y L R Up Down Left Right Start Select`, default 4 frames) holds a button on port 1 on a frame-indexed schedule, which is enough to skip an intro, pick a file and reach real gameplay. Two details it needs to work: the stock config has *no* keyboard binding at all, so the probe assigns its own synthetic scan codes to `Port1.Keys.Mapping1` and sets `Port1.Type = SnesController` before `LoadRom`; and the schedule is resolved against `Emulator::GetFrameCount()` *inside* `IsKeyPressed` rather than from a counter updated by the report loop, because the probe runs with `MaximumSpeed` set and the report loop only wakes on report frames — a cached counter skips whole presses. Positional arguments stop at the first `-`, so `--press` may follow `endFrame` directly without being read as a stride.

**A caution that comes with it.** Frame numbers are *not* comparable across the two emulators once input is involved: the two runs need their own press schedules, and a press that lands one message-box early diverges the whole run. Anchor the comparison on a state variable that is stable at the moment of interest (the Yoshi's Island camera work below anchored on GSU RAM `$0094 == 0x0100`, the last frame before the intro camera starts moving) rather than on a frame index.

**What it dumps.** `mesen_{vram,cgram,oam,gsuram,wram,screen}_f<frame>.bin` per report. WRAM was added 2026-08-03 for §10.7 of `Venus_SuperFX.md`, where the whole question turned out to be "does the divergence start on the CPU side or the chip side" - and it started on the CPU side, which nothing in the previous dump set could have shown.

**The alignment problem, because it will come back.** The two cores label a step differently: Mesen primes its pipeline with a NOP at every GSU start and logs `R15` *before* the fetch, so its `(address, opcode)` pairing is offset from ours — and the offset grows by one at each start, so no fixed shift aligns the streams. Register state is unambiguous, though, and an injected NOP changes none of it, so `gsudiff.py` collapses runs of identical register tuples on both sides and the artifact cancels exactly. Anything surviving that is real. Getting this wrong produces a confident "divergence at step 0" that is pure bookkeeping, which happened twice before the collapse trick.

**Two cautions worth carrying.** Mesen fills Game Pak RAM with **random** bytes at power-on, so any region the game has not written yet legitimately differs — compare only what the ROM touches. And trace buffers must be sized before concluding anything: a capped Mesen trace ends early and looks exactly like "EmuSen executed 1.8x more instructions", which is what it looked like right up until the cap was raised and the counts came out within 818 of each other over 300 frames.

**What the tests assert, and what they deliberately do not.** VRAM and CGRAM are compared **byte-exact** — they match, and that is the load-bearing fact, because it means a frame that looks wrong with matching video memory is a renderer bug and one that differs is an upload bug. OAM and GSU work RAM are *not* byte-exact yet (`Venus_SuperFX.md` §9's residual), so those two tests assert the property that actually failed — how many sprites reach the screen, and whether the GSU built its table at all — rather than pretending to an equality that does not hold. Neither the ROMs nor the dumps are committed (a VRAM snapshot of a commercial game is that game's data), so every test returns early when they are absent, the same shape as `NecDspRealFirmwareTests`.

**Also learned, and unfixed.** Three gaps this pass exposed in tools that already existed, all of which report something plausible rather than erroring, which is what makes them expensive:

- **`cov gsu` only records addresses fetched *outside* the GSU instruction cache.** In one measured frame it reported 140 of the 516 addresses actually executed — and the ones it did report were the first two bytes of each 16-byte cache line, which reads exactly like a genuine control-flow divergence against a reference that reports all of them. Not usable for "did this branch run" until fixed.
- **~~`copflow` logs nothing for the SuperFX.~~ Wrong - corrected 2026-08-03.** It works, and it is the right tool for the GSU register window: `copflow on 250000` over 301 frames of Yoshi's Island records **185,758 accesses across 23 registers**, including `$301F` written exactly 22 times for exactly 22 GSU jobs. The original reading was taken over 100 frames, and this game does not touch `$3000-$303F` at all between frame 0 and frame ~280 - a genuinely empty window, not a broken one. **Size the buffer and the frame range to the question before concluding a log is empty.** Note `watch`/`bp write` still cannot see this window (only `RegisterFlow?.Note` is raised for `CartridgeRegion.CoprocessorRegister`, not `WriteObserver`), so `copflow` is not merely the best tool here, it is the only one.
- **`dma` decodes some general transfers as `MPYL $2134`**, which is not a plausible DMA destination — `$2134` is a read-only multiply result. Seen on Yoshi's Island as seven transfers in one frame with otherwise sensible sources and sizes (`$7F0000` 65536 bytes, `$700000` 31744, `$702604` 20988). Every other destination in the same log decodes correctly (`$2118`, `$2104`, `$2122`), so this is either a destination-naming gap for those channels or the game really does write `BBAD = $34`. **Not chased** — it was noise relative to the sprite bug — but it is either a real decode gap or a real game behaviour worth knowing, and it should not stay unexplained.

**Superseded for CPU-side questions.** Everything above diffs *state* — memory contents at a frame boundary. §3.40 diffs *control flow* instead, which is the right tool whenever the two emulators are not in phase. Keep the state dumps for "is VRAM right"; reach for §3.40 for "why is the timing off".

**Checked and still not built.** Mesen's `StepBackManager` (rewind to instruction granularity), `Base6502Assembler`, `DisassemblySearch` and the Event Viewer's actual *plot* remain deferred for the reasons in §3.33 and §3.36, all unchanged. Mesen's `Cx4Debugger` and `St018Debugger` have no counterpart here for a simpler reason: this core does not implement the CX4 or the ST018, so there is nothing to debug — worth recording so a future pass does not read their absence as a debugger gap. `LuaApi::SetInput` is still parked for *our* side (§3.37); the probe's `--press` above covers the Mesen side of the same need, and `bp when autojoy` still makes the pair attractive, since a condition that fires on joypad reads goes naturally with being able to inject one.

---

### 3.15d `tapuntil` — navigating a scene by state, not by frame count

A `--commands` script that reaches a scene by counting frames (`frames 1250`, `tap Start 8`, `frames 60`, ...) breaks the moment core timing changes, and it broke immediately when §8.7's DRAM refresh landed: the same script that used to reach Yoshi's Island level 1-1 now stops on the intro stork, because every scripted tap lands somewhere else. §3.39 already warns not to anchor on frame indices; `tapuntil` is the verb that makes following that advice practical.

```
tapuntil <button> <space> <addr> <hexvalue> [cap] [every]
```

Taps the button (4 frames held, then released — a held button reads as one press to most menus) every `every` frames until the bytes at `<space>:<addr>` equal `<hexvalue>`, or `cap` frames pass. It reports how many taps and frames it took, and says plainly when it did **not** reach the value rather than continuing as if it had.

`tapuntil A GSURAM 1E1A F0,01 12000 40` walks the file-select and every cutscene message box and stops on the frame Yoshi's Island's intro camera target is set, in 20 taps.

**Two ways to pick an anchor address wrong, both paid for in §10.7.**

- **Never anchor on the value under investigation.** The first attempt used `$0094`, the camera position itself — the thing believed to be broken. An anchor has to be a value both emulators already agree about, or it either never fires or fires in the wrong place.
- **Never anchor on an address the battery save restores.** On a SuperFX cart the `.srm` *is* GSU work RAM, so once any run has played the scene, `$1E1A` holds its answer at power-on and this verb reports success at frame 0 having pressed nothing — which is exactly what it did, silently, for three days. Pass `--nobattery`. See `Venus_Memory.md` §2.4a.

---

### 3.40 Differential *control flow* — the S-CPU trace differ (`--cputrace`, `--tracediff`)

**Why this exists.** §3.39 compares memory contents at a frame boundary. That is only valid when both machines are at the same point in the program, and on Yoshi's Island they are not — EmuSen ran roughly a frame off the game's own `$0030` counter, and Mesen randomises power-on RAM. Every measurement in the §10.7 investigation paid for that: "317 diverged bytes" that were almost entirely random fill, a 4-run mask to erase it, a frame-offset hunt that *minimised* but never eliminated the difference, and `gsudiff.py` desyncing on a boot block only Mesen runs. Those were not four problems. They were one problem four times — **diffing snapshots of two clocks that are out of phase, where the noise scales with the skew.**

Control flow has no such coupling. If EmuSen simply runs a frame later but executes the same instructions, the two instruction streams align perfectly and the diff is empty until something real happens. Uninitialised RAM is never read at all, so the random fill cannot lie to you.

**The record.** Both emulators emit the same fixed 24-byte little-endian record, defined once in `CpuBinaryTrace.cs` and parsed by `CpuTraceDiff.cs`. The comparison itself lives in `TraceDiff.cs`, generic over `ITraceStep<T>`, so the GSU gets the same collapse/resync/cost machinery from its own record layout (§3.41):

| offset | size | field |
| --- | --- | --- |
| 0 | 3 | `(PB<<16)\|PC` — for an interrupt record, the *interrupted* address |
| 3 | 1 | reserved (0) |
| 4 | 1 | opcode (0 on interrupt records) |
| 5 | 1 | kind — 0 instruction, 1 NMI, 2 IRQ |
| 6..15 | 2 ea | `A X Y S D` |
| 16..19 | 1 ea | `DB P E` + pad |
| 20 | 4 | master clocks this instruction cost (backfilled after it runs) |

Files start with the 8-byte magic `ESCT` + version, so a blob written by an older layout is rejected rather than silently misread. Version 2 added the cost field and widened the record from 20 bytes to 24.

Registers are captured **before** the instruction executes, on both sides — Mesen in `SnesCpu::Exec()` before `RunOp()`, EmuSen in `Cpu.Step()` after the opcode fetch but before `Dispatch`. Getting that wrong by one instruction makes every register comparison useless.

**Interrupt records matter more than they look.** Mesen vectors NMI/IRQ inside `CheckForInterrupts()`, *outside* its per-instruction trace point. If one side recorded interrupt entry as a step and the other did not, the streams would desync at the very first NMI and everything after would be garbage. Both sides therefore emit an explicit kind-1/kind-2 record, and the differ treats kind as part of a step's identity, so an interrupt entry can never align with a plain instruction.

**Producing the pair.**

```
# Mesen, from the Mesen checkout (its DT_NEEDED is the relative bin/pgohelperlib.so)
cd /path/to/mesen2 && ~/.cache/emusen/probe/mesen/probe rom.smc outdir 80 80 --cputrace 80

# EmuSen
dotnet run --project EmuSen.Pharaoh -c Release -- rom.smc 81 --cputrace 80:outdir/emusen_cputrace_f00080.bin

# Diff (standalone, no ROM loaded)
dotnet run --project EmuSen.Pharaoh -c Release -- --tracediff outdir/mesen_cputrace_f00080.bin outdir/emusen_cputrace_f00080.bin
```

Volume is a non-issue: 80 frames of Yoshi's Island is ~1.0-1.1M steps, ~21 MB a side, and the whole diff runs in under half a second. There is no reason to trace a window rather than the entire boot.

**Loop collapsing, which is what makes the output readable.** A vblank spin loop iterating a different number of times is an *insertion*, not a substitution — an index-aligned diff would report every later step as different. `CpuTraceDiff.Collapse` detects a repeating address block (smallest period first, up to 32 instructions) and emits one node carrying its repeat count. A spin loop then becomes a single node, and a differing iteration count becomes a **number to report** rather than a desync: `loop [$008497,$00849A] ran 13x on the left, 15x on the right`. That line is the signal, not the noise.

**Resync.** When node shapes genuinely differ, `FindResync` looks for the smallest skip `(dl, dr)` that makes the next 8 nodes line up again — a full square search for small totals, then along each axis out to 4096 nodes, because a long skip is realistically one side running a block the other never runs, not both skipping different long blocks at once. The finding then reads `left executed N extra steps, right M, then the paths rejoin`, and the comparison continues instead of collapsing. This is precisely what `gsudiff.py` could not do, and why it was the wrong tool for §10.7.

**Two kinds of finding, and why the register one is worth more.** `Structure` and `LoopCount` say the *paths* differ. `Registers` says the same instruction, at the same point in the same path, saw different inputs — data went wrong *before* control flow did. Findings come back sorted by execution position, so whichever happened first is first, regardless of kind.

**What it found on the first run (2026-08-03).** Pointed at Yoshi's Island for 80 frames, the first divergence is at **step 238** — not frame 70, where three days of state diffing had placed it. It lands in the S-CPU's SPC700 IPL upload loop at `$00843B-$00844E`:

```
008440: CD 40 21  CMP $2140      ; wait for the APU to echo the byte back
008443: D0 FB     BNE $8440      ; spin
008445: 1A        INC A
008448: 8D 40 21  STA $2140      ; send the next one
00844E: D0 EB     BNE $843B
```

Both emulators upload **exactly 45,186 bytes** — the payload is right. But EmuSen spins **6.24 times per byte against Mesen's 5.45**, 14.5% more, and those extra spins account for ~71k of the 91,480-instruction excess EmuSen accumulates over the same 80 frames. Since 80 frames is the same emulated duration on both sides and both APUs complete the same upload within it, the APU is not slow — **the S-CPU is fast.** That inverts the natural first guess, and it is consistent with the `CMP Y, dp` fix (§`Venus_APU.md` 1.7) having moved the first main-loop frame from 79 to 77 against hardware's 78: we overshot.

The next step is the master-clock cost of that specific two-instruction spin against Mesen's, not another memory diff. The clock ratio itself is exact (`ApuClockHz`/`_masterClockHz`, not `/21`) and `GetAccessSpeedCycles` matches the fullsnes table, so neither is the suspect.

**The cost field is what actually found the timing bugs.** Recording *what each instruction cost* rather than an absolute clock makes the two sides directly comparable even after they drift, and lets the differ aggregate: it reports one row per `(opcode, leftCost, rightCost)` with a count and the total drift, sorted by how much damage each combination does. That table is what turned "we run 9% fast somewhere in 80 frames" into four named bugs in one afternoon — see `Venus_CPU.md` §8.7 and §8.8. Read it as a table, not as individual findings; a systemic bug shows up as one row with a huge count, and a `left = right + 40` row is DRAM-refresh *attribution* rather than a cost error.

**Mesen is not deterministic out of the box, and this was measured, not assumed.** *(Corrected 2026-08-06: the conclusion drawn below — that after `AllZeros` "every dump and the whole instruction trace are byte-identical across runs" — was true of the instruction trace and false of every memory dump, for a reason that had nothing to do with the RAM fill. See §3.45a.)* Its default power-on RAM fill is `RamState::Random`, and two identical no-trace runs of Yoshi's Island differ in VRAM, GSU RAM, WRAM *and* APU RAM at the same frame — which means every single-run comparison ever made against it carried noise, and the four-run masking in §3.39 was treating the symptom. The probe now sets `scfg.RamPowerOnState = RamState::AllZeros` before `LoadRom`, matching our own fill. After that every dump and the whole instruction trace are byte-identical across runs. **If a reference disagrees with itself, nothing measured against it means anything** — check that first, before believing any diff. Note `BaseCartridge::InitRamPowerOnState` overrides the setting for four named carts; none of them are relevant here, but check before trusting it on a new game.

**`--pressuntil`, the probe's half of state anchoring.** `tapuntil` (§3.15d) solves this on our side; the probe needs the same thing or the two runs still cannot be brought to the same place. `--pressuntil BTN:ADDR:VALUE[:CAP[:EVERY]]` taps a button until the GSU RAM word at `ADDR` equals `VALUE`, polling on the control thread while the emulator runs (the key manager grew an atomic `LiveKey` for it, since the frame-indexed schedule is fixed before boot and cannot answer a question decided at runtime).

Used in anger: `--pressuntil A:1E1A:01F0:12000:40` walks Yoshi's Island's file select and its whole opening cutscene and stops on the frame the camera target is set, in 17 taps. **Anchor on a value both emulators agree about** - anchoring on the buggy value itself cannot work, and the first attempt here did exactly that (`$0094`, which is the symptom) and never triggered.

**Cautions.**
- Both sides must start **from power-on**. The probe arms before `LoadRom` (which is what starts execution) and `FrameRunner` arms on the first `RunFrames`; `Reset()` executes no instruction on either, so the first traced step is the reset vector on both.
- Only the S-CPU is traced by `--cputrace`; the GSU has its own sink and verb (§3.41). `Cpu.cs` is shared with the SA-1's core, which has no Mesen counterpart to diff against, so recording is gated on the instance name.
- A `TruncatedSide` finding means the streams agreed for their whole shared length and one just ended first — raise its cap, do not read it as agreement.
- The trace buffer stops recording rather than throwing when it fills, and says so; a silently short trace would look exactly like an early divergence.

---

### 3.41 The same differ, pointed at the GSU (`gsutrace`, `--after`, `--nobattery`)

**What it is.** §3.40's machinery, generalised. `TraceDiff` is now generic over "a stream of steps with an address, an opcode, a cost and some registers" (`ITraceStep<T>`); `CpuTraceDiff` and `GsuTraceDiff` are just the two record layouts plus their parsers. `--tracediff` picks which one by the file's magic, so the same command reads either pair and refuses a mismatched one.

**The record.** 48 bytes, little-endian, magic `ESGT` + version, defined once in `GsuBinaryTrace.cs`:

| offset | size | field |
| --- | --- | --- |
| 0 | 3 | `(PBR<<16) \| ` the address the **opcode byte** was fetched from |
| 3 | 1 | kind (0; the GSU has no interrupt entries) |
| 4 | 1 | opcode |
| 5 | 1 | `SrcReg<<4 \| DestReg` |
| 6 | 2 | `SFR` |
| 8 | 32 | `R0`-`R15` |
| 40 | 4 | master clocks this instruction cost |

**Three conventions that have to match, and did not at first.**

- **The address is not `R15`.** Mesen's `R15` at the top of `Gsu::Exec` is already one byte past the opcode, and after a jump it names the *target* while the delay slot is still executing — so tracing it labels every delay slot with the wrong address. That is the same trap §4.1a fixed on this side, and it is the reason an earlier pass in §10.7 spent an afternoon disassembling code that never ran. Both sides now record the address the byte in the prefetch buffer came from: `_lastOpAddr` on Mesen's side, `_pipelineAddress` on ours. Pinned by `A_recorded_delay_slot_carries_its_own_address`.
- **`R15` is not compared.** It *is* the address, under two different conventions, so comparing it would report a divergence on every straight-line instruction. `GsuTraceDiff.ComparedRegisters` is 15 for that reason.
- **Three `SFR` bits are not compared.** `Go` (`$20`), ROM-pending (`$40`) and IRQ (`$8000`) are chip-state rather than program-visible arithmetic; ROM-pending in particular never sets here at all, because the ROM fetch is modelled as instantaneous (`Venus_SuperFX.md` §5.2). `ComparedSfrBits = $1F1E` is `Z`/`Cy`/`S`/`Ov` plus the four prefix bits, and those four prefix bits are worth their place — one of §10.7's four bugs showed up *only* in them.

**Producing the pair.** Both sides trace from a **shared state anchor**, not from power-on — the GSU runs a boot self-test on Mesen that this core never receives (§10.7), so power-on traces do not align at all and a crude longest-common-run match over them scored 158 of 80,000.

```
# Mesen: walk to the anchor, then trace 4 frames past it
cd /path/to/mesen2 && ~/.cache/emusen/probe/mesen/probe rom.smc outdir 0 0 4 1E1A 4 \
    --pressuntil A:1E1A:01F0:12000:40

# EmuSen: the same anchor, then the `gsutrace <frames> <path>` verb
#   tapuntil A GSURAM 1E1A F0,01 12000 40
#   gsutrace 4 outdir/emusen_gsutrace.bin
dotnet run --project EmuSen.Pharaoh -c Release -- rom.smc 13000 --nobattery --commands script.cmds

dotnet run --project EmuSen.Pharaoh -c Release -- --tracediff \
    outdir/mesen_gsutrace_f00710.bin outdir/emusen_gsutrace.bin
```

Four frames of Yoshi's Island is ~100k GSU steps and ~5 MB a side. `gsutrace` arms the sink, runs the frames and writes, all in one verb, precisely so it can sit *after* `tapuntil` in the same script — arming from power-on would drown the interesting part in the boot sequence.

**`--nobattery`, and why it is not optional here.** On a SuperFX cart the `.srm` is GSU work RAM, so a save written by an earlier run pre-loads the very address the anchor tests, and `tapuntil` reports success at frame 0 having pressed nothing. This invalidated three days of §10.7 measurements. `--nobattery` makes the run neither read nor write the save; the Mesen probe has always been isolated this way (`SetHomeFolder` into the dump directory). Full account: `Venus_Memory.md` §2.4a.

**`--after N`** runs the probe for N frames past a `--pressuntil` anchor *without* enabling the GSU trace, which is how a value is watched over hundreds of frames — 600 frames of trace would be ~800 MB, and the question "does `$0094` climb to `$0100`" needs a `DumpHex` line every 60 frames, not an instruction stream.

**What it found on its first run (2026-08-03).** Four real GSU bugs in one afternoon, each by walking to the first divergence and reading the instruction *before* it: `GETB` and `LDW` setting flags they must not, an `ALT` prefix that must clear `WITH`, and a 16-bit RAM access that pairs `addr ^ 1` rather than `addr + 1`. Distinct divergences went 119 → 67 → 10 → 0 across the four. See `Venus_SuperFX.md` §4.2, §4.4, §5.1a and §10.7.

**The lesson that generalises past the GSU.** Two of the four are *flag* bugs on instructions whose result was already correct, and a flag bug is invisible to every state-diffing tool in this document: the load is right, the register is right, and what goes wrong is a branch several instructions later. Carrying `SFR` in the record — and comparing the prefix bits, which looked like bookkeeping — is what made them findable at all. When adding a processor to this differ, record the flags.

---

### 3.42 Measuring a slow machine on a fast one (`--throttle`)

Every performance number this project has ever recorded came from one machine, a Ryzen 7700X. That was fine while the only question was "does Venus hold 60 fps?" — it does, with 3.3–12× headroom (`Venus_PPU.md` §13.1). It is useless for the question that opened on 2026-08-03: *how much machine does Venus need?* `--throttle` answers that without owning a second machine.

```sh
dotnet run --project EmuSen.Pharaoh -c Release -- <rom> <frames> --nobattery --commands script.cmds --throttle 33
```

**What it does.** `CpuThrottle.ReExec` relaunches the same argv inside a transient systemd scope with a CPU quota:

```
systemd-run --user --scope -p CPUQuota=33% -p CPUQuotaPeriodSec=5ms -- <self> <argv…>
```

`EMUSEN_THROTTLE_ACTIVE=1` in the child's environment is what stops it recursing. The re-exec happens in `Main` before every other mode, including `--diffshot` and `--tracediff`, because it replays argv verbatim.

**Why the 5 ms period matters.** The cgroup default is 100 ms, which is six frames. At that granularity a throttled run does not behave like a slow machine — it behaves like a fast machine that freezes for 60 ms at a time, and the per-frame distribution (`p95`, `max`, `over budget`) becomes meaningless while only the mean survives. `CPUQuotaPeriodSec=5ms` puts the enforcement granularity well inside a 16.64 ms frame, so latency figures stay readable. Against a fixed synthetic workload, 100/50/33/25% then measured 0.656/1.246/2.140/2.875 s against an ideal 0.68/1.36/2.06/2.72 — within about 5–9% of linear.

**It fails loudly and does not fall back.** No systemd, not Linux, `systemd-run` missing: the run exits 1 rather than quietly proceeding at full speed. This is deliberate, and the reason is §3.20's `-p:Optimize=true` footgun — a measurement flag that silently does not apply produces a *confidently wrong* number, which is worse than no number. Both the parent and the throttled child print a `[THROTTLE]` line, the child's into `--out` alongside the timings it qualifies.

**`--throttle 100` is not "off".** The quota is one core's worth summed across *every* thread in the scope, so 100% models a single-core machine, not an unconstrained one. Omit the flag entirely for unconstrained.

**Two honest limits.**

- **The scaling overstates slowdown, unevenly.** GC and tiered-JIT background threads draw from the same quota as the emulation thread, so the observed factor at 33% ran 3.6× (KBL3), 3.8× (SMW) and 5.5× (DKC) rather than 3.0×. DKC's `max 123.83ms` is a GC pause amplified by the quota. Use it for A/B comparison and for "does this miss the budget", not as a calibrated model of a specific CPU.
- **It is not a profiler.** It tells you the frame got slower, not which function. See below.

**On profiling, and why there isn't one here.** `dotnet-trace collect --profile dotnet-sampled-thread-time --format speedscope` runs without root and its *inclusive* percentages are trustworthy — it independently put 38.78% of a KBL3 level run under `RenderScanline` against the phase counters' 39.8%, which is a genuine cross-check of §13's instrumentation. Its *leaf* attribution is not usable: Release inlining collapses the 65816 interpreter into `RunFrame`, and the sampler's stacks bottom out on a GC-poll helper that absorbed 97.6% of self time. Real function-level attribution needs `perf` with perf-maps, which means installing it and lowering `perf_event_paranoid` from 2 — both root. Until someone wants that badly enough, the per-phase counters in `Venus_PPU.md` §13 are the finer-grained instrument, and they are the ones designed not to distort what they measure.

### 3.43 The probe learns a second console (NES mode, `--apulog`, `--wav`, `--ramstate`)

> **Superseded 2026-08-06 by §3.45 and §3.46.** The console fork described here is gone: a backend reports named memory spaces and the dump loop has no per-console branch. `apudiff`, described below as though it existed, did not — it was built on 2026-08-06 (§3.46). The worked SMB3 result at the end of this section was measured with the racing probe of §3.45a and has been re-taken: see §3.46a for what replaces it. The `ChannelVolumes` trap and the palette-mirror note are still current and still worth reading.

§3.39 built `MesenProbe.cpp` against a Mesen2 checkout so the SNES core could be diffed against a *running* reference rather than against Mesen's source. Moon (NES) needed the same thing the day it got a 2A03, and the probe is now one binary that drives either machine — it `dynamic_cast`s the loaded console and picks its dump set from that, so nothing at the call site changes.

**What an NES run dumps.** `mesen_{ram,work,sram,nametable,oam,palette,chr,screen}_f<frame>.bin`, in place of the SNES set. `screen` is 256×240 `uint16` **palette indices**, not RGB — mask with `0x3F` and index the 64-entry NTSC table (the same table Moon's `Ppu.NesPalette` carries) to render it. The per-report text line prints the PPU's control/mask/status decode plus `v`/`t`/`x`/scanline/cycle, and a full `ApuState` line: per-channel enabled/period/volume, DMC bytes-remaining and output, and the frame counter's mode and IRQ-inhibit bits.

**`--pressuntil` anchors on internal RAM here.** There is no GSU RAM on an NES, so the anchor watches the 2 KB internal RAM instead; the address is still a hex word address and the semantics are otherwise §3.41's. The `[PRESSUNTIL]` line names which memory it watched so a log is not ambiguous about it.

**`--apulog F`** records every write in `$4000-$4017` from boot to frame `F` as `mesen_apulog_f<frame>.bin`: an `ESAW\1` header then 12-byte records of `frame:u32, pc:u16, addr:u16, value:u8, pad:u8, opType:u8, pad`. This is the NES counterpart to §3.40's CPU trace but scoped to one register window, which is enough to answer "is the game's sound engine even running?" without a full instruction diff. It needs `mesen-nes-apu-trace.patch`, a hook at the top of `NesMemoryManager::Write` — the one funnel every CPU write passes through, including `DummyWrite` halves of read-modify-writes, which is why the record carries the operation type rather than filtering on it.

**`--wav PATH`** records Mesen's mixed output through `SoundMixer::StartRecording`. The recorder sits *before* the audio-device gate in `PlayAudioBuffer`, so it still captures under `EmulationFlags::MaximumSpeed` with no device open. This is the only ground truth for a silence question; the register log tells you what the game asked for, the WAV tells you what came out.

**The trap that cost the most time: `NesConfig::ChannelVolumes` defaults to all zeros.** It is declared `uint32_t ChannelVolumes[11] = {}`, and the Mesen GUI fills it from a config file that a headless probe never loads. `NesSoundMixer::UpdateRates` then computes `_volumes[i] = cfg.ChannelVolumes[i] / 100.0` and multiplies every channel by zero. A probe run therefore produces a WAV of exactly the right length, containing exactly nothing, for *every* ROM — which reads as "the reference agrees the game is silent" when it means "the reference was muted." The probe now sets all eleven to 100 after `GetNesConfig()`. The general lesson is §3.20's: a measurement that silently does not apply is worse than no measurement. Validate any new probe mode against a ROM known to make noise before trusting a negative result — SMB2's title music (peak 7398, mean |x| 755 over 300 frames) is the control used here.

**`--ramstate zeros|ones|random`** picks the power-on RAM fill for both machines. It defaults to `zeros`, which is what matches EmuSen's own fill and what makes a run reproducible; Mesen's own default is `random`, which is not. It exists because "does the power-on fill change this?" is a question worth answering in one flag rather than a rebuild — it did not change SMB3's behavior, but ruling it out cheaply was worth the knob.

**On diffing NES palette RAM.** Moon's `Ppu.PaletteOffset` folds `$3F10/$14/$18/$1C` onto `$3F00/$04/$08/$0C` on **read and write both**, so those four slots in `PaletteRam[]` are never stored and keep their power-on zero. Mesen stores the value in both the real entry and its mirror. Rendering is identical either way; a raw 32-byte diff is not, and will always show those four bytes as `00` against `0F`. Expect it and ignore it — it is not a divergence.

**The worked result.** SMB3 (MMC3) was carried as a known audio failure on the strength of a no-input harness run in which it never wrote `$4000-$400F`. The probe reproduced that exactly — Mesen is *also* silent through frame 1200 on the title screen, writing only `$4015` once at boot and then `$4010`/`$4017` as per-frame housekeeping. With Start tapped at frames 400/500/620 the engine starts on frame 501 in both, and the register log fills in with the full channel set. Frame-aligned, EmuSen's frame 900 is **pixel-identical to Mesen's frame 897** — 0 of 61 440 pixels differ — the three-frame offset being how long each took to accept the press, not a rendering difference. Every remaining byte-level difference at a fixed frame number (palette entry 3 blinking `3C`↔`36`, sprite slots rotating for flicker, one-pixel sprite positions) is animation phase from that same offset. See `Moon_APU.md` §6.

### 3.44 `--testroms` — third-party NES test suites, reporting through PRG RAM

The fourth standalone verb, alongside `--diffshot` (§3.15), `--singlestep` (§3.16) and `--tracediff` (§3.40): a real tool in this toolchain that needs no debug session, no window and no human.

```
dotnet run -c Release --project EmuSen.Pharaoh -- --testroms <dir|rom> [frame-budget]
```

Where `--singlestep` (§3.16) validates one CPU against vectors, this validates *everything else* against ROMs written on real hardware by someone who never saw this implementation. A blargg test ROM reports its verdict into the cartridge's PRG RAM — `$6000` status, `$DE $B0 $61` signature at `$6001`, ASCII detail from `$6004` — so the whole run is machine-readable. One line per ROM: `[PASS]`, `[FAIL]` with the ROM's own text, `[----]` for no verdict inside the budget, `[SKIP]` for an unimplemented board, `[ERR ]` for a bad image or a crash. Exit code 0 only if nothing failed, errored or went silent.

Two details are load-bearing and are easy to get wrong in the direction of a **false pass**. The status is read straight out of `PrgRam[]` rather than through `CPUBUS`, because `Mmc3.ReadPrg` returns `0` when WRAM is disabled — and `0` is also the code for "passed". And a completion code is ignored until `$80` has been seen at least once, because the poll runs once per frame and can otherwise land between the ROM writing its signature and writing its status, reading a pass off a test that has not started. Both are covered in `Moon_TestRoms.md` §2.

The ROMs are third-party and not vendored, the same rule §3.16's vectors follow — fetch from [christopherpow/nes-test-roms](https://github.com/christopherpow/nes-test-roms). The runner forces `--nobattery` for the whole run so a shared suite directory is never written to.

**What this is expected to find.** `Moon_PPU.md` §4 and `Moon_APU.md` §6 already name their gaps — per-dot timing, the odd-frame dot skip, the sprite overflow bug, DMC fetch alignment, the open-bus latch. A suite failing on one of those is the harness confirming something already written down. The interesting results are the ones nothing predicted, and `cpu_interrupts_v2` is the one to watch: `Moon_CPU.md` §5.4's instruction-boundary interrupt polling is a documented approximation that **nothing currently validates at all**, because the `nes6502` vectors do not exercise interrupts.

`MoonCore.Reset()` — the first soft reset in this project, on either core — exists because of this verb: status `$81` means "press RESET now", and a suite using it cannot run without one. See `Moon_Core.md` §6.

---

### 3.45 The probe stops being about Mesen (`Reference/probe/`, `IProbeBackend`, the libretro backend)

§3.39 built `MesenProbe.cpp` and §3.43 taught it a second console. Both are now history: the tool is one policy layer plus a backend per emulator, because a single reference is a single opinion, and the next cores to be checked against (SameBoy for Mercury, bsnes or ares for Venus) are not Mesen and never will be.

> **Status.** The architecture below is current and is the reason the rest of the port was possible. Only the *language* moved afterwards: the C++ paths named here (`probe/ProbeMain.cpp`, `probe/ProbeBackend.h`, `probe/backends/`) became `Reference/probe-rs/src/` on 2026-08-08 and `Reference/probe/` was deleted. Read the file names as roles, not as paths — §3.50 and §3.52 have the current ones. The section's central claim is unaffected and was in fact what made the port legal: **the contract is the file protocol, not any one language's interface.**

**The split.** `probe/ProbeMain.cpp` parses arguments, holds the press script, runs the state anchor, drives the report loop and writes dumps. It includes no emulator header and names no emulator. Everything machine-specific is behind `probe/ProbeBackend.h`:

| member | why it is in the contract |
| --- | --- |
| `Name()` | becomes the dump prefix, so a Mesen dump set keeps the filenames every consumer already globs |
| `System()` | `nes`/`snes`/`gb`; the console, which is not the same question as the emulator |
| `RunUntil(frame)` | advance and **stop on that frame**; everything else on the interface is only valid while stopped |
| `Spaces()` | named blocks, so the dump loop has no console fork and no fixed struct |
| `SetSchedule()` / `SetButton()` | the frame-indexed script, and the live override `--pressuntil` decides at runtime |
| `BeginTrace()` / `EndTrace()` | optional, and **false by default** |

Two of those carry the reasoning worth keeping.

**Why the schedule is sampled by the backend and not by the report loop.** A probe that free-runs between reports cannot set the pad itself without dropping presses, because it does not stop on every frame. Only the backend knows when its emulator polls input. So the *parsing* of `--press` is policy and the schedule is an emulator-neutral list of `(startFrame, endFrame, button)`, but the *sampling* is the backend's, resolved against its own frame counter at poll time. Getting this backwards silently drops presses on exactly the runs that need them.

**Why traces stay optional.** A per-instruction trace cannot be had from an arbitrary emulator without patching its source. A backend without such a hook returns false and the tool says so, rather than writing an empty file that reads as "the reference agrees nothing happened" — §3.20's rule again.

**The libretro backend, which is the point of the exercise.** `probe/backends/LibretroBackend.cpp` `dlopen`s any `*_libretro.so` and drives it through the standard ABI, with **no per-emulator code whatsoever**. `retro_run()` is one frame and is called from the probe's own thread, so frame landing is exact by construction — the entire class of problem the Mesen backend needed a source patch for does not arise. Cores are ordinary Fedora packages (`libretro-nestopia`, `libretro-gambatte`, `libretro-bsnes-mercury`, `libretro-mgba`, …) and `build-probe.sh libretro-core <package>` unpacks one into the work directory without installing anything, the same trick §3.39 already used for SDL2. `libretro.h` comes from `retroarch-devel` rather than being vendored, so it cannot drift from the real ABI.

**What libretro cannot reach, stated up front so it is not rediscovered.** The ABI standardises `SYSTEM_RAM`, `SAVE_RAM`, `VIDEO_RAM` and `RTC`, and nothing else. There is no OAM, no palette, no nametable, no CHR, and no trace of any kind. A libretro backend is a **breadth** instrument — main RAM and the framebuffer across a dozen emulators — and not a replacement for the depth a native backend buys. Nestopia, for instance, exposes exactly one space on an NES cartridge: the 2 KB of internal RAM.

**One binary per backend.** Linking is not free — Mesen's pulls in a 14 MB `MesenCore.so`, libretro's needs only `dlopen` — so the backend is a compile-time choice (`-DPROBE_BACKEND_*`) and `build-probe.sh <backend>` produces `~/.cache/emusen/probe/<backend>/probe`.

**The work directory is no longer in `/tmp`.** It holds both the unpacked headers and the built probe, and defaulting it to `/tmp` meant every session re-downloaded the RPMs and relinked — which was the whole of "we keep building it each time". The emulator's own object tree was never the cost. It is now `~/.cache/emusen/probe/<backend>/`, plus an up-to-date check that makes a no-op invocation take 0.024s instead of minutes.

**The manifest.** Each report also writes `<backend>_manifest_f<frame>.json`, naming every space with its size and file, and the screen's dimensions and pixel format. This is what makes a dump set from an emulator this project has never seen readable at all: the old layout could only be interpreted by knowing in advance which emulator wrote it and which console it was.

---

### 3.45a The probe was reading memory out of a running emulator, and had been all along

This is the most important thing in this section, and it invalidates measurements rather than adding a capability.

`Emulator::Pause()` sets `_paused`, and `Emulator::IsPaused()` returns that same field. The probe's idiom was therefore a no-op:

```cpp
emu->Pause();
while(!emu->IsPaused()) { sleep(1ms); }   // true on the first read, always
```

Every dump the probe ever took was read while the emulation thread was still running. This was confirmed against the **old** binary, not inferred: two consecutive runs of the same ROM at the same frames differ, by 22 bytes of NES internal RAM at frame 180. §3.40 states that after `RamState::AllZeros` "every dump and the whole instruction trace are byte-identical across runs"; that held for the instruction trace, which is written by the emulation thread itself, and was **false for every memory dump**. Its own warning is the one that applies — *if a reference disagrees with itself, nothing measured against it means anything.*

**Why the obvious fixes do not work.** `AcquireLock()` genuinely parks the emulation thread at a frame boundary, but only once the control thread has noticed the frame went by, and at `MaximumSpeed` that notice arrives two to five frames late — the probe then lands on frame 305 having been asked for 300. Polling `GetFrameCount()` more tightly cannot close this: the emulator runs thousands of frames a second and the control thread sleeps in milliseconds.

**The fix is `patches/mesen/probe-frame-stop.patch`**, a fourth patch alongside the three trace hooks. The emulation thread compares its own frame count against `g_probeStopFrame` and stops *itself*, inside the emulation loop, after `WaitForLock()` and before `WaitForPauseEnd()`. Stopping on the thread that owns the state is the only construction that is both exact and quiescent. After it: **35 of 35 files identical across three runs, landing exactly on the requested frames**, and 7 of 7 on the SNES side.

**How wrong the old dumps were**, measured against the corrected probe on SMB3 (differing bytes):

| space | f60 | f120 | f180 | f240 | f300 |
| --- | --- | --- | --- | --- | --- |
| `ram` | 10 | 2 | 34 | 20 | 37 |
| `oam` | ok | ok | 6 | 16 | 10 |
| `nametable` | ok | ok | 32 | ok | ok |
| `screen` | ok | ok | **96** | **287** | **167** |
| `work`, `palette`, `chr` | ok | ok | ok | ok | ok |

On Yoshi's Island at frame 1500 only `gsuram` was affected, by 384 of 32768 bytes; VRAM, CGRAM, OAM and the screen all matched. That is why §3.39's VRAM/CGRAM conclusions survive this and the SuperFX work was not built on sand — the spaces that investigation leaned on happen to be quiescent at a frame boundary, and the ones the coprocessor and CPU write continuously are the ones that were torn. **The rule to carry: a space that is being written when you sample it is the one that will lie to you, and which spaces those are depends on the game, not on the emulator.**

---

### 3.45b The frame stop covers advancing, not arriving — a report at an already-reached frame still reads a running machine

§3.45a is not quite complete, and the remainder was found on 2026-08-08 while verifying an unrelated fix. `MesenBackend::RunUntil` opens with

```cpp
if(_emu->GetFrameCount() >= frame) { return; }   // never sets g_probeStopFrame
```

so when the requested frame has **already** been reached the function returns without arming the stop and without parking anything. The emulation thread was left running by `LoadRom` and stays running; the report that follows samples a live machine. §3.45a fixed the case where the probe *advances* to a frame and forgot the case where it is *already there*.

**The reproduction is `--pressuntil` landing at frame 0**, which is ordinary: anchoring on a value the machine already holds at power-on succeeds before a single frame is run, so `RunUntil` is never called with an advancing target. Five consecutive runs of the same binary, same ROM, same arguments:

| run | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- |
| `line` / `cyc` | 0 / 216 | 3 / 113 | 2 / 129 | 2 / 225 | 5 / 169 |

Against a report that does advance (`probe rom out 60 60`), the same field is `line 240 cyc 5` on every run. The tell is exactly §3.45a's own rule — *a space that is being written when you sample it is the one that will lie to you* — and the PPU's scanline and dot counters are the fastest-moving state the probe prints.

**How much this actually cost, stated carefully so it is neither dismissed nor inflated.** In the measured cases the *memory dumps and the manifest were byte-identical across runs*; only the state line's `line`/`cyc` varied. That was not a guarantee, it was a consequence of RAM being quiescent so soon after power-on. A frame-0 anchor on a game that is mid-DMA would tear the way §3.45a's table shows. Every `--pressuntil` run that reached its anchor immediately was affected; runs that tapped their way to a later frame were not, because those advance and therefore park.

#### The fix, and the claim it retired

Two changes, both additive:

1. `probe-frame-stop.patch` gains `g_probeStopArmed` and moves the stop check to the **top** of the emulation loop. A separate armed flag is what allows a target of 0 to mean "stop", which `g_probeStopFrame != 0` could not express; checking before `RunFrame` rather than after is what parks a machine that has already arrived instead of one frame later.
2. `MesenBackend::Load` arms the stop **before** `LoadRom`, which is what starts the emulation thread. Arming afterwards is a race the probe cannot win — at `MaximumSpeed` an unknown number of frames elapses between reading the counter and storing a target — and `Load` then waits for the park before any caller reads identity, spaces or the first signature row. `RunUntil`'s early return is now correct rather than accidental: every exit from it leaves the machine parked, so "already at the frame" finally means "already stopped at the frame".

Six runs of the frame-0 anchor, before and after, same ROM and arguments:

| | 1 | 2 | 3 | 4 | 5 | 6 |
| --- | --- | --- | --- | --- | --- | --- |
| before | 1 / 0 | 2 / 53 | 2 / 51 | 2 / 100 | 2 / 323 | 0 / 177 |
| after | 0 / 25 | 0 / 25 | 0 / 25 | 0 / 25 | 0 / 25 | 0 / 25 |

Advancing reports still land exactly (`frame 60`, `line 240 cyc 5`, every run), and the dumps are byte-identical across runs.

**A prediction made while designing this was wrong and is retired.** The change was expected to make **frame 0** observable, on the reasoning that the loop ran a frame before the old bottom-of-loop check could fire. Instrumenting `Load` showed otherwise:

```
[DBG] before LoadRom frame=0 armed=1
[DBG] after  LoadRom frame=1 armed=0 paused=1
```

`Emulator::LoadRom` runs a frame before it returns, so the counter is already 1 by the time any external code can arm anything. **Frame 0 is not reachable on this backend at all**, and the first observable frame is 1 — which is what the pre-fix probe reported too, verified by rebuilding the old binary against a core carrying the old patch and running it six times. The fix therefore changes *determinism*, not *numbering*.

This leaves a real off-by-one between backends: Mesen's first observable frame is 1, libretro's is 0, because `retro_load_game` runs no frames. It is inherent to the two emulators rather than to the probe, the comparator's ±16-frame alignment search absorbs it, and it is recorded here so it is not diagnosed a third time.

---

### 3.46 `apudiff` — the NES register-write differ, and what it says about SMB3

§3.43 documented `apudiff` as the consumer for `mesen_apulog_*.bin`. It did not exist; nothing in the repo implemented it, and our side emitted no APU log to compare against, so the logs sat unread for two days. Both halves are now built.

**Our side.** `ApuWriteTrace` (`Moon - NES/Debug/`) emits the same 12-byte `ESAW\1` record the probe does, driven by `EmuSen.Pharaoh --apulog <frame>:<path>`. **The hook is at the very top of `MemoryBus.Write`**, before any branch, which matters more than it sounds: `$4014` and `$4016` are handled by their own early-returning branches, and a first attempt that hooked the APU branch logged **zero** writes to both while Mesen logged 1176 and 9430. The reference's hook is at the top of *its* funnel for the same reason.

**The differ** reuses §3.40's `TraceDiff` unchanged, through an `ApuWriteTrace.Step : ITraceStep<Step>` adapter. That reuse earns its keep here rather than being tidiness: a sound engine rewrites the same registers every frame, so loop collapsing is the difference between a readable diff and thousands of identical lines. `--tracediff` picks the parser off the magic, so the same verb now reads `ESCT`, `ESGT` and `ESAW`. It also prints a per-register write count for both sides, which is the summary that actually answers an audio question — the trace diff says *where* the streams parted, the table says *which channel*.

`TraceDiff.cs` moved to `EmuSen.Cores.Debug` for this. It was already generic over `ITraceStep<T>`; it now has a consumer on a second console, which is the bar a shared location has to clear. `CpuTraceDiff.cs` and `GsuTraceDiff.cs` stay where they are, being record-layout-specific.

**The SMB3 result, 1200 frames with Start tapped at 400/500/620.** 17,286 writes on Mesen against 16,802 on ours. Every register agrees within six writes except one:

| register | Mesen | EmuSen |
| --- | --- | --- |
| `$4010` | 2377 | **1899** |
| `$4014` | 1176 | 1178 |
| `$4016` | 9430 | 9424 |
| `$4017` | 1177 | 1179 |
| all others | — | within 1 |

`$4010` is the DMC rate/IRQ/loop register, and the gap is 478 writes — about 20%, against roughly two writes per frame on the reference and 1.6 on ours. A register the game writes conditionally, that we write measurably less often, on a console whose DMC is already the documented weak point (`Moon_APU.md` §6: flat 4-cycle fetch cost, the two failing `apu_test` suites). **Not chased here** — this section is the tool landing, not the investigation — but it is the first hard number anyone has had on it, and it is the obvious next thread.

---

### 3.46a What the SMB3 comparison actually says

The question that started this was "load SMB3 and find what we did wrong". On the evidence, in video memory: **nothing**.

- `nametable`, `palette`, `chr` and `oam` reproduce the reference **byte-exactly** across 75 consecutive frames, once compared at the right instant.
- The one apparent failure was `nametable` at frame 180, 64 bytes of 2048. It is not a divergence. **Our frame 180 is byte-identical to Mesen's frame 182**, and the difference grows by exactly 32 bytes per frame in both directions — SMB3 scrolls its title screen 32 nametable bytes per frame, so this is the same data, two frames later. A perfectly linear gradient like that is proof the *content* is right and only the *phase* differs.
- Internal RAM differs by 11-52 bytes of 2048, and **this is not a defect either**. Three-way against Nestopia (via the libretro backend):

| frame | Mesen↔Nestopia | Mesen↔EmuSen | Nestopia↔EmuSen |
| --- | --- | --- | --- |
| 60 | 9 | 11 | 11 |
| 120 | 16 | 30 | 26 |
| 180 | 32 | 40 | **21** |
| 240 | 39 | 52 | **28** |
| 300 | 30 | 46 | 30 |

Two mature, independent, unrelated emulators disagree with each other by 9 to 39 bytes at the same frames, and EmuSen sits inside that spread — closer to Nestopia than Mesen is, at three of the five. **Byte-exact internal RAM at a given frame is not a property any pair of emulators has**, because a frame boundary is not the same instant in two machines and whatever the CPU was part-way through writing differs accordingly. This is the first question in this project that a *second* reference answered and a single reference could not, and it is the argument for the libretro backend in one table.

What remains genuinely open is the two-frame lag and the `$4010` write gap. Both are timing, neither is data, and both now have a number attached.

**What the tests assert, and why they do not assert more.** `ReferenceDumpTests` replaces `MesenReferenceTests`; it discovers dump sets by globbing `<backend>_<space>_f<frame>.bin` and reads the console and ROM from the manifest, so it is no longer SNES-only nor tied to Mesen having produced the data. The old one keyed fixture discovery on `mesen_vram_f*.bin`, which no NES dump set contains, so **every NES dump was structurally invisible to it** — the suite went from 2268 tests to 2469 by making them visible. Video memory is asserted byte-exact, allowing a match against an adjacent dumped frame, which is a *stronger* claim than any tolerance would be and reports the offset when it uses one. A frame within the phase search of either end of the dumped window is skipped rather than failed, because the neighbour that would prove an offset may simply not have been dumped, and "no match" then says nothing. Internal RAM gets a 5% tripwire and an explanation of why it is a tripwire and not a proof: a real fault diverges hundreds or thousands of bytes, not tens.

---

## 4. Underlying helper libraries (pre-date the toolchain above)

### `EmuSen.Cauldron/` — real-time snapshot providers

The thread-safety seam between a live core and anything reading it. Deliberately knows nothing about DianaOS, cores, or any console.

- `IRealtimeProvider<T>` — `Current` (never blocks, safe from any thread) plus `Refresh()` (only ever from the thread that owns the core). Every provider property on `IDebugTarget` is one of these.
- `PollingProvider<T>` — the default: wraps a "go read the live core" delegate and publishes via a lock-free `Volatile` reference swap. Remembers only the latest snapshot.
- `HistoryProvider<T>` — same contract, but also retains the last `capacity` snapshots in a fixed-size ring, plus a **staleness signal**: `RefreshesSinceChange` counts how long `Current` has compared equal to its predecessor. `PollingProvider` answers "what is the machine doing now", which is what a live dashboard wants; this answers "what was it doing before it stopped", which is what an investigation into a transition wants. `GetHistory()` copies out under a lock — the one member here that can block, which is why it is a method rather than a property that looks as cheap as `Current`.
- `ListEqualityComparer<TItem>` — element-wise equality for the `IReadOnlyList<T>` snapshots providers publish. The staleness signal needs it: snapshots are rebuilt every `Refresh`, so reference equality would report "changed" every time and the signal would read 0 forever.

**Kept a separate type on purpose.** History costs a retained reference per refresh, so a consumer that only reads `Current` should not pay for a ring it never looks at. Only `CoprocessorRegisters` uses it today (§3.23a). Note this is *not* the old `DebugTools.ChangeTracker<T>` removed below — that had no call sites at all; this one is load-bearing.

### `Cores/Nintendo/Venus - SNES/Debug/StateDump.cs`
On-demand CPU+PPU snapshot formatter. Returns formatted strings (doesn't print directly) — `DumpCpuState`, `DumpPpuState`, `DumpAll`. Deliberately laid out to be directly comparable to MesenCE's own Status panel.

### `DianaOS/DebugTools.cs`
Generic, dependency-free helpers, kept genuinely reusable for a future core's data - no Raylib, no SNES-specific types:
- `BoundedTrace` — a start/countdown helper for "trace the next N frames then auto-stop" (used by the P-key scroll trace).
- `RepeatCollapsingTrace<TKey>` — collapses a repeating instruction cycle in a verbose CPU/APU trace into one summary line (§2's `CpuVerboseLogging`/`Spc700VerboseLogging`). **Built after a real session produced a 1 GB `cpu.log` almost entirely from polling loops** — the VBlank wait, the DMA busy-wait, the APU handshake — running the same handful of instructions thousands of times over. It detects **cycle lengths 1 through 8**, not just immediate duplicates, because those loops are typically two or more instructions (a load/branch or decrement/branch pair) rather than one repeated line, so a duplicate-line filter would have caught almost none of it. It compares on a caller-supplied **value key** (`PB`/`PC`/opcode/target as a small struct), never the rendered string, so a locked-in loop costs one struct comparison per call and allocates nothing, and the string formatting that previously ran for every single instruction now runs only for entries actually emitted. Nothing is ever dropped — every key that arrives is either emitted or folded into a summary that accounts for it, and that guarantee costs three specific pieces of machinery:

  - **Two tuning constants.** `MaxCycleLength = 8` is long enough for the short polling bodies real 65816/SPC700 code uses and short enough that scanning for a not-yet-confirmed cycle — the only part that is not O(1) — stays cheap; a cycle of length ≤ 8 must appear within the most recent 16 entries, so anything older can be emitted immediately rather than held. `MinRepeatsToCollapse = 4` exists because below it the marker line is often longer than the duplicates it replaced, so two instructions that merely happen to repeat once are written out in full instead of dressed up as a loop.
  - **A partial trailing repeat is replayed, not discarded.** When a cycle breaks mid-pass, the lines that did match are byte-identical to their template positions and genuinely executed, so they are re-emitted individually.
  - **`Flush()` is mandatory at two moments** — verbose logging toggled off mid-session, and process exit — or a loop still running, or a tail that never got to prove itself a cycle, is silently lost.

  **A bug worth keeping:** `renderMarker` takes the caller's `[TAG]` prefix rather than hardcoding one. The original version emitted a tagless `    ^ ...` marker, which `CategorizedLogWriter`'s prefix table could not route, so it fell through to the `general` category — which is not in the cpu/apu console-echo suppression list. Every collapsed loop therefore went straight back to blasting the live console, defeating the fix it was part of.

**Removed** (a code-agnosticism/redundancy pass): `HexDump`, `DecodeTileAscii`, `CgramToRgb`, `DumpPalette`, `DescribeBits`, and `ChangeTracker<T>` all had zero call sites left anywhere in the codebase, fully superseded by the generic `mem`/`tile`/`pal` shell commands (§3.3) — and three of them (`DecodeTileAscii`/`CgramToRgb`/`DumpPalette`) were hardcoded to the SNES's own tile/color formats despite this class's "reusable for a future NES core" framing, which the `tile`/`pal` commands now honor for real via `IDebugTarget.DecodeTilePixels`/`GetPalettes()` instead.

### `Cores/Nintendo/Venus - SNES/Ppu/Renderer/Renderer.Debug.cs`
- `RenderVramSheet` — renders the full VRAM tile sheet to an internal texture (feeds the console debug view's VRAM panel).
- `DumpActiveOam` — see §1/§3.4.
- `DumpBlackBg1Tiles` — an old, narrowly-scoped diagnostic from a "black squares" investigation; still wired to the O key.

---

## 5. Save states

**F5**/**F9** in the console build; Save/Load State menu items in the Avalonia frontend. Backed by `Common/StateSerializer.cs` — a reflective binary serializer that walks fields (not properties), skipping anything marked `[SkipInState]` (dispatch tables, back-references, and debug-only bookkeeping) or `[AliasOfSerializedField]` (a reference to an array another field already writes).

**Full format reference: `EmuSen_Save_States.md`.** States now carry a magic + version header, and pre-v1 files still load via a legacy read path. The same bytes back `RewindBuffer` in memory (`EmuSen_Rewind_And_FastForward.md` §1).

**Known limitation:** the layout is positional, with no field-name tagging — adding, removing, or reordering a serialized field still needs a version bump plus a read path for the old layout, or older files misalign into garbage rather than failing cleanly. See `EmuSen_Save_States.md` §1/§4.

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

- **Breakpoints / single-step / pause-resume** — done (§3.1's breakpoints note, §3.3's `bp` row), and since extended with conditions, `step over`/`step out` and `runto` (§3.27, §3.28). Still open: `EmuSen.Pharaoh` doesn't drive the halt/resume loop at all (its `bp add` is hit-counting only, §3.15) — a scripted, non-interactive equivalent of `step`/`continue` would need the harness to check `IsHaltedAtBreakpoint` and decide what to do next, which nothing does today.
- **A verification pass on the disassemblers** (§3.7, §3.34) — there are now four (65816, SPC700, GSU, NEC DSP). The 65816 one still hasn't had the scrutiny the execution opcode table got against oxyron.de; the SPC700 and NEC DSP decoders shipped with table-driven tests covering every opcode's mnemonic, operand form and length, but those check the decoder against its own documented table, not against hardware. Worth a dedicated pass rather than trusting any of them blind.
- **Watchpoints beyond WRAM** — VRAM/CGRAM/OAM and the general CPU-bus/SRAM write paths don't report to `MemoryBus`'s `IWriteObserver` hook yet (§3.4). This now also caps `bp read`/`bp uninit` (§3.36), which see exactly the accesses the read observer reports — today IO and WRAM.
- **Per-byte code-vs-data classification** — the one part of Mesen's `CodeDataLogger` §3.36 did not build. It needs the CPU to tag each memory access with an operation type at every read site in the 65816, which is core work rather than a debug-layer addition. `cov`'s instruction-start bitmap is a different and in some ways better structure; the two would need reconciling rather than one being bolted onto the other.
- **A cartridge-ROM read observer** — there is an `IRomReadPatcher` hook on every cartridge-routed read, but it exists to *change* bytes for the cheat engine, not to report them. A proper observer would let `counters`, `bp read` and any future data-classification pass see ROM reads, which today none of them do.
- **A Mesen-style Event Viewer** — the largest remaining gap from the Mesen comparisons (§3.33): a per-frame plot of register writes, DMA and interrupts against scanline/dot coordinates. Deliberately deferred rather than skipped — its value is in the 2D plot, so it wants the GUI debug window below, not another text command. `copflow` (§3.35) covers the coprocessor-register slice of the same data as text, and `dma` (§3.38) covers the DMA/HDMA slice, including the scanline column that carries most of the plot's value.
- **Step back one instruction** and **an assembler** — also from §3.33, both deferred for reasons recorded there (instruction-granularity rewind belongs in the core; the assembler should follow the disassembler's verification pass).
- **The Avalonia GUI debug window** — partially started: `Settings > DianaOS Console...` (`EmuSen.Mistress/Views/DianaOSConsoleWindow.axaml`) is a real terminal window onto the full `DianaOSInterpreter` (§3.17), text-only. The actual Mesen-style multi-pane debugger (register panels, hex viewer, disassembly view, sprite/palette viewers, event log, watch panel, call-stack panel) built against `IDebugTarget` directly still doesn't exist - everything in §3 was built with that as the eventual consumer, and the console window covers the same functionality today just as one text pane rather than dedicated graphical panels.
- **An NES (or other console) `IDebugTarget` implementation** — the interface was designed generically for this from the start, but no second implementation exists yet to prove it out. The §3.27 and §3.34 fallback contexts are a down-payment: a second core gets `eval`, conditional breakpoints and per-chip scoping working off its `regs` output alone, with no expression code of its own. An NES core would publish `cpu` plus whatever its cartridge mapper carries through the same `DebugCpus` list.
- **A GSU/SPC700 call stack** — neither reports one (§3.34), so `bt`, `step over`/`step out` and `profile` are unavailable for them. The GSU's subroutine convention is register-based rather than a stack the way the 65816's JSR/RTS is, so this needs a chip-specific inference (tracking LINK/R11) rather than the existing seam, and is only worth building if a real GSU investigation wants it.
- **Coprocessor-side `copflow` attribution** — every entry is currently tagged `cpu`, because the S-CPU's own register-window accesses are the only ones that route through `Cartridge`'s decode (§3.35). A chip reading its own registers internally never reaches that seam, so a genuinely two-sided log would need per-chip hooks.
- **The CX4 and the ST018** — not implemented as *chips*, which is why nothing here debugs them. Mesen carries a `Cx4Debugger` and an `St018Debugger`; their absence here is a core gap, not a toolchain gap, and `DebugCpus` (§3.34) would carry either of them the day the chip exists.
- **A visual, burned-in-pixel screenshot timestamp** — deliberately deferred in favor of the companion-file approach (§3.6); revisit if the on-image version is still wanted.
- **Programmatic controller input** — Mesen's `LuaApi::SetInput`. The one item from §3.37 parked rather than dismissed: several open questions in `Games_Tested.md` are input-related (the SMAS controller-port assignment, the IoG title screen), and a `pad` command that injects a button press would test them directly. It belongs with the frontend's input plumbing rather than in a debug registry, which is why it was not built alongside the rest of that pass.
- **A conditional trace** — Mesen's `TraceLoggerOptions.Condition`/`Format`. `trace` is still count-only. Deferred in §3.37 because the condition would have to be evaluated inside the core's own verbose-log path, and logpoints cover the case that actually comes up more precisely.
- **A ROM database/mapper for known per-title quirks** — not built yet. The idea: a small lookup (by ROM checksum, not filename) that lets a frontend auto-apply a known cartridge-specific behavior - the concrete case that prompted this is Super Mario All-Stars' Controller 2 quirk (`Games_Tested.md`'s SMAS entry), where the mirroring workaround exists but currently has to be turned on by hand (F7 in `EmuSen.Hotaru`, a checkbox in `EmuSen.Mistress`) rather than being applied automatically for that specific cartridge. Needs to stay **core-agnostic** if built - EmuSen is adding cores beyond the SNES before it ever reaches disk-based generations, and a "quirks database" keyed on some SNES-specific identifier (or living inside `EmuSen.Cores.Nintendo.Venus` at all) wouldn't carry over to an NES/other core needing the exact same kind of per-title override; the lookup mechanism belongs at a level every core can plug into, the same way `IDebugTarget` itself doesn't assume any one console's shape.

---

### 3.47 Two ways a screenshot diff lies, found while verifying the 2026-08-08 mappers

The probe makes it cheap to compare a frame against the reference, and cheap comparisons invite over-reading. Six new NES boards were verified this way; two of the comparisons were meaningless, and neither announced itself.

**The reference may not be running the board you asked for.** Mesen carries a game database and, on a CRC hit, overwrites the header's mapper number outright (`GameDatabase::SetGameInfo` assigning `romData.Info.MapperID`). For images with damaged headers — which, per `Moon_Memory.md` §2.2, is a large minority of any real library — the two emulators can in principle be running *different boards* on the same file, and nothing in the probe's output would reveal it.

> **Corrected 2026-08-08.** This section first named `Adventures in the Magic Kingdom (U) [a1]` as a case where that had happened, and asserted the reference was playing it as MMC3. **That was wrong, and it was asserted without being checked.** `GameDatabase::InitDatabase` loads `MesenNesDB.txt` from the emulator's home folder, and the probe points the home folder at its own dump directory (`ProbeOptions::HomeFolder`), which never contains one. No database is loaded in a probe run at all. Once the identity gate below existed it reported `mesen board=65` against `emusen board=65` — the same board — in the first second of being pointed at the problem.
>
> The mechanism is still real and still worth guarding, which is why the gate exists and why this paragraph stays. What was wrong was reaching for it as an explanation because it was available, rather than measuring. The blank screen that prompted all this is an unexplained difference in this core's Irem H3001 or in that dump, and is recorded as open in `Moon_Memory.md` §4.11 rather than as solved.

**A tapped run is not a comparison.** Feeding both sides the same `--press`/`--tap` schedule does not keep them in the same place. Mr. Gimmick with three Start taps diffed at 79% of the frame and After Burner at 1.22%; with no input at all, at a later frame, both were pixel-identical. The taps had landed on different menu states, and a menu state difference reads exactly like a catastrophic rendering bug. The existing `--pressuntil` anchor exists precisely for this and should be preferred; where an anchor is not known, **an unattended run at a later frame is stronger evidence than a driven run at an earlier one**, which inverts the intuition that more input means more coverage.

The general form of both: a differential test proves two systems agree, and says nothing about whether they were asked the same question. The cost here was roughly an hour split between a board that was never wrong and a divergence that was never a bug.

---

### 3.48 The comparability gate, and EmuSen as a peer backend

*Where it lives now: `EmuSen.WiseMan/Reference/analysis/compare.py`, not `--compare`. The gates read files and never load a ROM, which puts them on the Python side of the seam `EmuSen_Stack.md` §2.2 draws, and the signature stream they read is now a SQLite table rather than a CSV parsed per query. The design below is unchanged and was held to byte-identical output through the move — see `EmuSen_Stack.md` §3 for the differential result. `EmuSen.Pharaoh --compare` prints where the tool went and exits 1.*

§3.47 records two comparisons that were valid-looking and meaningless. The response is not a better differ — the differ was correct every time — but a layer that decides whether a difference *means* anything before measuring one, and a symmetric arrangement in which this project is one backend among several rather than the privileged consumer.

**The contract that matters is the file protocol, not the C++ vtable.** `IProbeBackend` stays what it was: the in-process adapter for an emulator that has to be driven. A *peer* is anything that writes the same files. `EmuSen.Pharaoh --probe` does exactly that, emitting `emusen_<space>_f<frame>.bin`, `emusen_manifest_f<frame>.json` and `emusen_screen_f<frame>.bin` beside the reference's. `compare.py` then reads two directories and never learns which emulator made either.

Two additions to the protocol:

- **`identity` in every manifest** — the board, region, PRG/CHR sizes, save presence and header trust the backend *settled on*, as opposed to what it did. Empty fields are legal and meaningful: a libretro core cannot report a mapper, and a gate that reads silence as agreement is worse than one that reports a one-sided check.
- **`<backend>_sig.csv`** — one row of CRC32s per frame, from frame 0, with a `#`-prefixed header carrying the identity and a column list that is the schema. A consumer intersects on column names rather than assuming two emulators expose the same spaces.

The gates, cheapest first:

1. **Identity.** Board, region, sizes and system, before a single pixel is read. A mismatch is `NOT-COMPARABLE` and the pixel count is not reported at all, because it would describe the mismatch.
2. **Divergence.** Where the two streams first parted, on columns that have first been shown capable of answering — see §3.48a, which is the part that did not work as designed.
3. **Informativeness.** Whether the frame could have disagreed: colour count, dominant-colour fraction, and whether the reference *moves* across the phase window. A reference that is static across the window makes every phase offset match, so a phase "match" proves nothing about timing.

The verdicts are `PASS`, `PASS(vacuous)`, `NOT-COMPARABLE`, `DIVERGED-AT`, and `DIFFERS`, carried out as exit codes 0/3/2/4/1 so a script can branch on the *kind* of answer.

**One inversion worth stating, because it was written wrongly first.** Vacuity grades an agreement. A frame that plainly disagreed is never vacuous however uniform either side is — the first cut reported Adventures in the Magic Kingdom as `PASS(vacuous)` while 87.53% of its pixels differed, on the grounds that our own screen was a single colour. A blank screen where the reference draws content is the strongest evidence of a fault available, and the gate had been built to file it as a pass.

**Measured against the four cases from §3.47, plus one nobody had identified:**

| ROM | Verdict |
|---|---|
| Klax | `PASS` — 14 colours, reference moving, phase +2 stable |
| After Burner | `PASS(vacuous)` — 3 colours, reference static across the window |
| Adventures in the Magic Kingdom | `DIFFERS: 53778 px (87.53%) and no gate explains it` |
| Mr. Gimmick (E) | `NOT-COMPARABLE: region differs: emusen=ntsc mesen=pal` |

The last was not one of the ghosts this was built to catch. `Mr. Gimmick (E)` is a PAL image, the reference runs it at PAL timing, and an hour earlier it had been recorded by hand as a pixel-exact confirmation of the FME-7. It matched because the screen was static. The tool found that on its first outing against real data, which is the strongest argument for it that exists here.

### 3.48a Memory is not comparable across emulators, and the gate has to measure that

The divergence gate was designed around a per-frame RAM hash: find the first frame where the two streams differ, and classify by whether an input preceded it. **It does not work, and the measurement that shows why is worth keeping.**

Agreement rate per column, aligned on the best offset, over ~300 frames:

| ROM | `ram` | `work` | `oam` | `palette` |
|---|---|---|---|---|
| Klax — pixel-identical | 62.0% | 100% | 1.6% | 0% |
| After Burner — pixel-identical | 0.3% | 100% | 100% | 0% |
| Magic Kingdom — 87% of pixels differ | 1.0% | 100% | 10.2% | 0% |

A frame that is *pixel for pixel identical* carries 62% RAM agreement in one game and 0.3% in another, and a genuinely broken one sits between them. Three causes, none of them a bug in either emulator:

- **The frame boundary is not the same instant.** Two emulators stop at different points relative to the game's own update, so a space the game is halfway through writing is sampled at different moments and disagrees while nothing is wrong.
- **Power-on contents are undefined.** NES palette RAM has no defined reset state; each emulator fills it differently and `palette` therefore agrees 0% of the time, permanently, on every ROM.
- **Layout and size differ.** This core's nametable mirror is 4 KB against the reference's physical 2 KB, and CHR banking differs in arrangement without differing in effect. Neither can agree byte for byte even when the machines do.

So the gate **measures each column's comparability rather than assuming it**, classifying by agreement rate as `stable` (≥95%), `intermittent`, or `never` (≤5%), and admits only `stable` columns as divergence evidence. With none available it reports `INCONCLUSIVE` and claims no divergence frame — a gate that cannot answer says so rather than inventing a number, which is the failure mode the whole section exists to prevent.

Three consequences fell out of building it:

- **Alignment cannot use a constant column.** A column that never changes agrees at every offset, so it carries no timing information; `work` is constant in most runs and dragged the alignment to a nonsense offset until it was excluded. Alignment now uses only columns that vary.
- **Alignment is mandatory, not a refinement.** This core runs two frames behind the reference on boot. Comparing frame *N* to frame *N* reports a divergence in the first row of every ROM ever tested.
- **A column must be rated on a leading window, not on the whole stream.** Rating over everything is self-defeating: a genuine divergence half way through a run drags that column's agreement rate below the threshold and disqualifies the one column that would have revealed it. The first cut did exactly this and reported `INCONCLUSIVE` on a fixture built to diverge at a known frame. The rating is now computed over the first quarter of the aligned frames and then used to judge the rest, which is an ordinary calibrate-then-detect arrangement and carries its usual limitation: a divergence *inside* the calibration window makes the column look unreliable and is missed. That case is left to the screen, which catches it as `DIFFERS`.

**The screen remains the reliable signal** — normalised to RGB, it compared correctly in every case all day. Memory is the cheap signal that must be calibrated before it is believed. That asymmetry was not the expected result and is the main thing this section has to hand over.

---

### 3.49 The known-differences dictionary, and why "proven" is a schema constraint

The gates in §3.48 tell you *that* two emulators disagree on a column. What they cannot tell you is whether that disagreement is already understood. Accumulating those explanations is obviously useful and just as obviously dangerous: **a wrong entry is worse than no entry**, because it teaches the comparator to explain away a real defect, silently, forever, on every future run. §3.47 records exactly that mistake being made by hand — an explanation reached for because it was available rather than because it was measured, and written into two man pages as fact.

So the dictionary is built around the assumption that its author is not to be trusted, including when its author is the person adding the row.

**Storage.** SQLite. The schema, seed and reader now live in `EmuSen.WiseMan/Reference/analysis/` and are read from Python's stdlib `sqlite3`, which needs no package and so retires the `SQLitePCLRaw.bundle_e_sqlite3` 3.0.5 pin *for this database* — that pin was needed because the 2.1.11 `Microsoft.Data.Sqlite` 10.0.10 resolves by default carries GHSA-2m69-gcr7-jv3q, surfaced as `NU1903`, and it is still required wherever the C# catalogue driver runs. **None of the discipline below is affected**: it is enforced by the schema's triggers, so it holds whatever language opens the file, which is the entire reason it was built that way rather than as reader-side checks.

**What is committed is SQL, not a database.** `schema.sql` and `seed.sql` are the reviewable source; `known-differences.db` is a build artifact, rebuilt when missing, and is not in the repository. A binary in git cannot be diffed, reviewed or merged, which would defeat the point: the entire value of a proof requirement is that a claim can be *argued with* on the merits before it is believed.

**Four tables, and the interesting part is the fourth.** `definition` is the claim, `evidence` is what motivated it, `assertion` is the falsifiable predicate a machine can re-run, and `verification` is one row per time an assertion was actually executed, carrying the commit it ran against. A definition is `provisional`, `proven` or `retracted`.

**Promotion is enforced by triggers, not by discipline.** Writing `status = 'proven'` in a seed file aborts the insert. Promoting a definition with no passing verification aborts. Promoting one with no assertion aborts. And because `verification` has a foreign key to `assertion`, a definition with nothing to re-run cannot have a verification recorded against it at all — the stronger invariant, which a test tried to state the weaker version of and was corrected by the schema.

**The dictionary annotates; it never suppresses.** A proven entry adds `known: <slug>` beside a column and nothing else. It does not remove the column, hide the rate, or change a verdict. This is the deliberate part: a difference that stopped being reported the day it was explained is a difference nobody notices the day its cause is fixed. Columns with no proven explanation are marked `UNEXPLAINED`, which is the state most of them should be in.

**Demotion is automatic.** `verify_dictionary.py <ours> <theirs>` re-runs every assertion against a real pair; anything that fails is demoted to provisional rather than deleted, because a claim that has stopped holding here may still hold elsewhere and its history is worth keeping.

The first run of the verifier demoted one of its own seed entries. `moon-reports-ntsc-unconditionally` was asserted with a fixture of "any NES pair" and evaluated against Klax, where both sides correctly report NTSC, so the assertion failed and the entry lost its status. The claim is true; the *assertion* was wrong, because a claim about PAL images cannot be demonstrated on an NTSC one. Assertions now name the fixture that demonstrates them and skip on any other pair. That a badly-scoped claim was rejected rather than believed is the system doing its job on its own author, an hour after that author had made the same class of error by hand.

The retracted game-database claim from §3.47 is seeded as a `retracted` row, with its reasoning in `retracted_why`, so the dictionary carries a worked example of the failure it exists to prevent.

---

### 3.50 The probe in Rust, and what a language change actually bought

The probe was ported to Rust on 2026-08-08. This section records what the port could and could not reach, because the interesting result is a negative one: **the C++ does not go away, it moves**, and the reasons it cannot go away are structural rather than incidental.

> **Status.** This section is the Phase 1 record and is kept for its reasoning, not as a description of the tree. Its file inventory is historical: `Reference/probe/` was deleted the same day once the Mesen backend followed, and the design sketched under "a fifth patch, not a Rust file" was built and accepted. **§3.52 is the current account of the Mesen side.** What remains live here is the analysis of *why* the split falls where it does, the two byte-compatibility prohibitions, and the three defects — all of which still hold.

**The three parts have nothing in common but a build rule.** The policy layer (`ProbeMain.cpp`, `ProbeDump.cpp`, `ProbeBackend.h`, 727 lines) is ordinary argument parsing, formatting and file writing, and transcribes directly. `LibretroBackend.cpp` (364 lines) is `dlopen` over a stable C ABI, which is the case Rust FFI handles best. `MesenBackend.cpp` (405 lines) cannot be written in Rust at all, and the reason is worth stating precisely rather than as "C++ interop is hard": it **subclasses `IKeyManager`**, and Rust cannot synthesise a C++ vtable. `dynamic_cast<SnesConsole*>`, the `std::vector<uint8_t>` trace globals and the by-value `SnesConfig`/`GsuState`/`NesPpuState` are each individually workable; an abstract base class that must be *implemented* is not.

**Mesen's own C ABI does not rescue this, which was checked rather than assumed.** `InteropDLL/` exports 210 `extern "C"` functions and looked like the obvious escape. It fails on four counts, all verified in the source: `InitializeEmu` only constructs a key manager when a real window handle is passed and on Linux constructs `LinuxKeyManager` (evdev), so headless `SetKeyState` is a silent no-op; there is **no `GetFrameCount` export at all** (the C# UI receives it through notification callbacks); there is no screen-buffer export, only `TakeScreenshot` to a PNG and the AVI recorder; and `GetMemoryState` is a `WithDebugger` API, so using it would force the debugger on. A probe needs exactly frame count, headless input, raw framebuffer and memory — the four things the C ABI does not offer.

**So the Mesen backend's replacement is a fifth patch, not a Rust file.** The design that follows is an add-only `Core/EmuSenProbeApi.cpp` exposing a small `extern "C"` surface, with the key-manager subclass living *there*, next to the class it derives from. This is cheap for a reason worth recording: `makefile:138` is `CORESRC := $(shell find Core -name '*.cpp')`, so a new file in `Core/` is compiled with no makefile edit, and a patch that only *creates* a file cannot conflict on `git apply` however far upstream drifts. The 112 lines that modify upstream stay at 112.

**Staging.** The libretro half landed first and the C++ probe was kept as the A/B baseline, because §3.45's own contract makes that legal: *the contract that matters is the file protocol, not the C++ vtable, and a peer is anything that writes the same files.* Two probes writing the same protocol is the arrangement the architecture already describes for Pharaoh.

#### What byte-compatibility forbade

The port is not an opportunity to improve the format, and two attractive Rust conveniences had to be refused:

- **No `serde_json` on the writing side.** A serialiser is free to reorder keys and reflow whitespace, and either breaks a consumer that is byte-shaped. The writer stays hand-rolled, on both the Rust and C# sides, and the byte-parity tests between them depend on it. *Reading* was the same way — `DumpSet.ReadManifest` and `ReferenceDumpTests.SystemOf` both used regexes — but that was a consequence of the writer's constraint rather than a requirement of its own, and the Python ingest (`dumpdb.py`) uses a real parser. `ReferenceDumpTests.SystemOf` still does not.
- **No `clap`.** The existing parser ignores unknown flags and ends optional positionals on a leading `-`; `clap` would reject both. Hand-rolled parsing, and `atoi`/`strtoul` semantics reimplemented deliberately, because a port that rejected `12abc` would refuse command lines that used to run.

The honest summary of the gain is therefore narrower than "modern": what Rust bought here is that `Spaces()` returns slices whose lifetime is tied to the backend, `libloading` in place of raw `dlsym`, `Result` in place of `printf`-and-return, and a dump layer with 27 unit tests pinning the wire format. Performance was not a goal and is not the argument, though the Rust probe runs a 400-frame signature sweep in 0.19s against the C++ probe's 0.35s.

#### Three defects the port surfaced

Transcribing code is an unusually thorough way to read it, and it found things.

**The state anchor was a dangling pointer, and this is a real use-after-free rather than a theoretical one.** `ProbeMain.cpp:226` read

```cpp
const MemorySpace* anchor = ProbeDump::Find(backend->Spaces(), anchorName);
```

`Spaces()` returns a `std::vector` **by value**, and `Find` returns a pointer into it; the temporary dies at the semicolon, and the pointer was then dereferenced on every iteration of the `--pressuntil` loop. It never visibly misbehaved because the freed bytes were not reused and the `Data` pointers inside addressed emulator memory that genuinely outlived the vector.

Built with `-fsanitize=address` and given a `--pressuntil` that has to tap, the unfixed probe aborts on the first read:

```
ERROR: AddressSanitizer: heap-use-after-free
READ of size 4 at 0x7b7f1aa314f8 thread T0
0x7b7f1aa314f8 is located 40 bytes inside of 48-byte region
freed by thread T0 here:   [~std::vector]
```

The arithmetic identifies the field exactly: a `MemorySpace` is 48 bytes (32-byte `std::string`, 8-byte pointer, 4-byte size, 4 padding), and the 4-byte read at offset 40 is `space.Size` — the first thing `AnchorWord` touches.

**Both implementations now look the anchor up by name on every read.** The Rust port could not express the original at all; the C++ was fixed to match rather than left to rot until Phase 3, because it is still the only way to drive Mesen. The fixed C++ is clean under ASan on the anchor-reached, cap-exhausted and advancing paths, and a full A/B — seven scenarios across two cores — shows the dump trees, stdout and exit codes unchanged, which is the evidence that the fix is behaviour-preserving and not merely quieter.

Verifying it also turned up §3.45b, which is a separate defect and is written up there rather than here.

**A documented argument behaviour was only half-implemented, and now matches its comment.** The comment at `ProbeMain.cpp:91` says a `-` flag "ends the optional positionals". The lambda tested each slot independently, so it did no such thing: it stopped `--press` being read as a *stride*, but the flag's **value** still fell into the next slot. `probe rom out 0 120 --press 10:Start` set `anchorAddr` to `0x10` rather than the default `0x0A00`, and `--backend libretro` set it to `0`. Only the report's hex line moved, which is why it went unnoticed for the tool's whole life.

It was preserved through the port so that Phase 1 could be accepted on byte-for-byte agreement, then fixed **in both implementations together** so the A/B stayed meaningful: a slot is a positional only if every slot before it was one too. `--press 10:Start` now leaves the anchor at `$00A00` in the Rust and C++ probes alike. Fixing it separately from the port is the point — a behaviour change to a documented CLI deserves its own before-and-after, not a free ride inside a transcription.

**Declining the log interface is not the same as suppressing it.** `retro_log_printf_t` is a printf-style variadic, and defining one is nightly-only in Rust, so the first implementation simply returned `false` for `GET_LOG_INTERFACE` on the assumption that a core's fallback logger writes to stderr. Measured, it does not: gambatte then emits five `[Gambatte]` INFO lines to **stdout**, in the middle of the dump report, which the C++ probe had been suppressing all along by answering the callback and dropping everything below `WARN`. The fix installs a non-variadic function into the variadic slot — sound on every ABI this builds for, since the callee reads only the two leading arguments — reproducing the C++ behaviour exactly for messages with no arguments and printing the format string literally for the rest, rather than silently wrong.

#### Acceptance

The port is accepted on a byte-for-byte A/B against the C++ probe, not on the dumps in `Reference/dumps/`. **That corpus cannot serve as a regression net and the assumption that it could was wrong**: it is gitignored, so `git ls-files` reports nothing, and its manifests carry no `identity` object, meaning they predate `ProbeDump.cpp`'s identity block and not even the current C++ probe reproduces them. A baseline has to be regenerated, not read off disk.

What was run, with both binaries built from the same sources and pointed at the same core and ROM:

| scenario | core | files | stdout | exit |
| --- | --- | --- | --- | --- |
| 0–400, stride 20, `--sig`, two presses | nestopia | identical | identical | 0/0 |
| 60–80, stride 10, `--sig` | gambatte | identical | identical | 0/0 |
| 0–300, stride 25, `--sig`, press | gambatte | identical | identical | 0/0 |
| `--pressuntil` reaching its anchor | nestopia | identical | identical | 0/0 |
| `--pressuntil` exhausting its cap | nestopia | identical | identical | **2/2** |
| `--after` span past an anchor | nestopia | identical | identical | 0/0 |

Argument and load failures agree on exit `1` across five paths. On the C# side, `--compare` reads the Rust probe's manifests and signature stream and reports `board=? trust=? prg=0 chr=0` — the honest "libretro cannot say" of §3.48 — and WiseMan's 267 `Reference` tests pass unchanged.

One difference is **not** attributable to the port: `bsnes_mercury_performance_libretro.so` segfaults during `retro_load_game` under **both** probes, at exit 139. It is a pre-existing limitation of the libretro backend with that core and is recorded here so it is not rediscovered as a port regression.

---

### 3.51 What a dump set costs, and why the audio codec was the wrong lever first

A dump set is disposable, but it is not free, and the two changes here were sized by measurement rather than intuition. The prompt was a fair objection — WAV in 2026 is a bulky choice — and the measurement agreed with the objection while disagreeing about where the bytes were.

**An 11-report SMB3 run, 4,865,532 bytes:**

| | bytes | share |
| --- | --- | --- |
| `wav` | 1,916,620 | 39.4% |
| `chr` | 1,441,792 | 29.6% |
| `screen` | 1,351,680 | 27.8% |
| everything else | 155,440 | 3.2% |

Audio really is the largest single component at a coarse stride. But the ranking inverts as the stride tightens, because the WAV is one file for the whole run while `screen` and `chr` are one file per report: at stride 1 over the same 600 frames those two would be ~152 MB against an unchanged 1.9 MB of audio.

**The two levers, measured against each other:**

| | bytes | share |
| --- | --- | --- |
| raw | 4,865,532 | 100% |
| FLAC the audio, leave the rest | 2,958,658 | 60.8% |
| `zstd -19` the whole directory | 72,520 | **1.5%** |

The codec is worth 1.6×; compressing the directory is worth 67×. The reason is visible in the blob census — `chr` was **11 files and one distinct value**, and a blank screen blob compresses from 122,880 bytes to 25 — so the redundancy is *between* files, where no per-file codec can reach it.

#### Deduplication

A blob unchanged since the last report is hard-linked to the file that already holds it. The 11-report run above drops to 3 distinct CHR inodes; a `--pressuntil` run reports `15 blob(s) written, 10 unchanged and hard-linked`.

A link rather than a `"sameAs"` manifest field, deliberately: every consumer already globs `<backend>_<space>_f<frame>.bin` and stats its size, and a new field would have to be taught to `DumpSet`, `ReferenceDumpTests`, the comparator and anything written later. The file is still there, still the right length, and `diff -r` between two dump sets still compares content — which is what kept the Rust/C++ A/B valid across this change.

The comparison is **byte-exact, not hashed**. A 32-bit digest over 128 KB collides rarely enough to survive testing and often enough to eventually publish one frame's memory as another's, silently. That is §3.49's rule about a wrong entry being worse than no entry, applied to storage.

#### FLAC by default, WAV as the fallback

`--audio PATH` is the new flag and writes FLAC unless `PATH` ends in `.wav`. The reference emulator can only write WAV, so the backend records a `.capture.wav` and the policy layer re-encodes after `Shutdown` — which is what closes the file.

Measured on 20 seconds of SMB3 gameplay: 3,833,464 bytes of WAV to 553,337 of FLAC, **14.4%**. `flac -8` is preferred and `ffmpeg -c:a flac` is the fallback, the latter being a dependency the project already carries for frame recording.

**WAV survives on three paths, and this is the point rather than an afterthought.** `--wav` asks for it outright, which is what deep troubleshooting wants: every language reads it with no dependency and `xxd` on a WAV is a real debugging step. An encode that fails leaves the capture in place. And an encode whose **round-trip does not match** does the same, because lossless is a claim rather than an observation until the samples come back out identical — the encoded file is decoded, its PCM compared against the capture's, and only a match retires the WAV. Verified end-to-end: decoded FLAC and a directly-recorded WAV of the same run hash to the same SHA-256 over their sample data.

Comparing PCM rather than whole files matters, and the chunk walk that makes it possible is not decoration: a decoder's WAV carries chunks Mesen's writer does not, so a byte-for-byte file comparison would fail on padding while the audio was identical.

#### A false alarm worth recording

Investigating this began with a WAV that compressed 1483:1 and contained zero non-zero samples across 958,288 of them, which read as the §3.43 muted-mixer defect returning. It was not: SMB1 and SMB3 title screens are genuinely silent until Start is pressed, and the same ROM with `--press 200:Start` records 56% non-zero samples at peak 10447. The pre-§3.45b probe produced identical silence, which is what settled it.

The same measurement retired an open caveat from §3.45b: that run parks four times, so `WaitForPauseEnd`'s `OnBeforePause` → `StopAudio` on each park does **not** damage the recording, and the suspicion that it might is closed.

---

### 3.52 The Mesen C ABI, and where a language boundary actually belongs

§3.50 ended on a prediction: that the Mesen backend's replacement would be *a fifth patch, not a Rust file*, and that the C++ would move rather than disappear. Both halves held, and this section records what the move cost and what it did not buy. The work landed on 2026-08-08 and completed the port; `Reference/probe/` no longer exists.

**The boundary is drawn at the vtable, and that is the whole design.** `probe-c-api.patch` creates `Core/Shared/EmuSenProbeApi.cpp`, exposing seventeen `extern "C"` functions. Two things stayed on the C++ side, and the reason each stayed is the reason the boundary is where it is rather than one function further out:

- **`ScheduledKeyManager`**, the `IKeyManager` subclass. Rust cannot synthesise a C++ vtable, so the class has to live next to the class it derives from. What crosses the ABI is the press schedule as a flat `uint32_t` triple array, and Mesen resolves it against `GetFrameCount()` *at the moment the pad is polled* — the invariant §3.45 measured, now preserved by construction rather than by discipline, since the caller has no way to sample it late.
- **`probe_state_line`**, the per-report register decode. It reads `SnesPpuState`, `NesPpuState`, `ApuState` and `GsuState` by value. A caller that re-declared those layouts in Rust would be silently wrong the first time upstream added a field, and silently wrong is the failure mode this whole toolchain exists to avoid. **The formatted line is the stable artifact; the structs behind it are not.**

Everything else — frame count, memory-space enumeration, screen, identity, traces, the live button override — marshals cleanly, and `mesen.rs` is 266 lines of marshalling with `mesen_sys.rs` at 74. That ratio is the honest summary of the exercise: the Rust file is thin because the hard part did not move.

**Add-only, and that is load-bearing.** The patch only *creates* a file, so `git apply` cannot conflict however far upstream drifts, and `makefile:138` (`CORESRC := $(shell find Core -name '*.cpp')`) compiles it with no makefile edit. It changes not one line of upstream Mesen.

The patch set as a whole is **173 insertions and zero deletions** across five upstream files, plus the add-only sixth. Zero deletions is checked rather than assumed, and it caught something: the NES CPU hook below initially reported two deletions, both of them upstream trailing whitespace this editor had tidied on lines it happened to touch. Restoring them cost nothing and keeps the property exact. The set is verified reproducible by reverting the checkout to pristine, re-running `build-probe.sh`, and confirming all six files come back **byte-identical** to what they replaced.

#### Two decisions that look like nits and are not

**The ABI is guarded by a version number rather than a shared header.** The two sides live in different repositories, so a header would have to be authored in one and consumed in the other, and would drift the moment a checkout carried an older patch. `probe_abi_version()` is checked once before the first call, and a mismatch is a startup error naming the fix. The alternative — generating the binding with `bindgen`, as the libretro side does — is right *there* and wrong *here*, and the distinction generalises: bindgen protects against drift from a header you do not control, and there is no such header in this case.

**The link is deliberately not improved.** Cargo could easily have produced a position-independent binary with an `RPATH`, freeing the probe from the documented `cd <checkout>` invocation. It does not: `build.rs` emits `-L<checkout>` plus `-l:bin/pgohelperlib.so` so the recorded `DT_NEEDED` is the relative `bin/pgohelperlib.so`, byte-identical to what the C++ probe carried. **An A/B is only meaningful if both binaries load the same library the same way**, and a nicer link would have been a worse experiment. `readelf -d` on both confirmed the match before any dump was compared.

One incidental finding, recorded because it contradicts what the Cargo book's phrasing suggests: **build scripts do receive the crate's feature `cfg`s**, not merely the `CARGO_FEATURE_*` environment variables. This was measured with a throwaway crate rather than assumed, and it matters because `bindgen` is an *optional* build-dependency — under `--features mesen` alone it is not linked into the build script at all, so an environment-variable check would still have to name `bindgen::` and fail to compile. `#[cfg(feature = ...)]` is the construction that works.

#### Acceptance

Both probes built from the same sources, run from the same checkout against the same ROMs, comparing dump trees, stdout and exit codes.

| scenario | ROM | files | stdout | exit |
| --- | --- | --- | --- | --- |
| 60–140, stride 20, `--sig`, two presses, `--cputrace`, `--apulog` | SMB3 (NES) | identical | identical | 0/0 |
| `--pressuntil` reaching its anchor, `--after 12`, `--sig` | SMB3 | identical | identical | 0/0 |
| `--pressuntil` exhausting its cap | SMB3 | identical | identical | **2/2** |
| too few arguments | SMB3 | identical | identical | **1/1** |
| 80–200, stride 40, `--sig`, GSU trace | Yoshi's Island (SuperFX) | identical | identical | 0/0 |
| 60–120, stride 30, `--sig`, press, `--cputrace` | Yoshi's Island | identical | identical | 0/0 |

Two consecutive Rust runs are identical to each other — the §3.45a determinism check that originally caught the 22-byte NES RAM drift, and it now passes with **no exclusions at all**, which it could not before the `.rgd` fix below. The traces are substantive rather than trivially empty: a 28.6 MB GSU trace (596,590 steps), a 19.3 MB SNES CPU trace (803,877 steps), a 13.6 MB NES CPU trace (567,455 steps, once the hook below existed) and a 6,956-byte NES APU log (579 writes) — each byte-identical and each additionally reporting **no divergence** through `EmuSen.Pharaoh --tracediff`, which compares semantically after loop collapsing rather than by bytes. The libretro A/B was re-run afterwards, since `dump.rs` is shared: still identical across nestopia, gambatte and the `--pressuntil` path. WiseMan's 267 `Reference` tests pass, 37 and 38 Rust tests pass under the two feature sets, and `clippy -D warnings` is clean under both.

#### Three defects the acceptance run surfaced, and their fixes

The A/B was clean, but running it turned up three things that were wrong independently of the port. All three are now fixed; they are written up together because each was found by the *next* one's investigation.

**The probe was writing a timestamped file into its own dump tree.** The first A/B reported `DIFFER` on all four substantive scenarios, on `mesenhome/RecentGames/<rom>.rgd`. What settled its cause was not inspection but the determinism run: **two consecutive runs of the same probe differ in that file too**. It is a zip of a screenshot and a save state that `Emulator::LoadRom` writes on every load, and it made every dump tree compare unequal to every other.

The first response was to exclude the directory from the diff. That was the wrong instinct and it is worth saying so plainly: an exclusion list is a standing invitation to hide a real difference behind it. Mesen already has the switch — `SaveStateManager::SaveRecentGame` returns early under `EmulationFlags::ConsoleMode` or `EmulationFlags::TestMode`, described in its own comment as the testrunner path — so the probe now sets it and the file is never created. **`TestMode` and not `ConsoleMode`**, and the difference is not cosmetic: the two gate this identically, but `ConsoleMode` also reaches `Emulator.cpp:519` and calls `InitDebugger()`. Choosing it would have switched the debugger on for every probe run, which is precisely the disqualification §3.50 records against Mesen's own C API. A dump tree now compares with **no exclusions at all**.

**`--cputrace` on an NES ROM produced an empty trace, and always had.** The file was 8 bytes — magic only — and the probe honestly reported `[cputrace 0 steps]`. The cause was that `cpu-trace.patch` instruments `Core/SNES/SnesCpu.cpp` and nothing else, so there was no 6502 hook to fire. `begin_trace` cannot warn about it either, because the console is not known until after load.

`nes-cpu-trace.patch` adds the hook, and it fills **the same buffer and the same 24-byte `ESCT` record** as the S-CPU rather than inventing a second format. One machine runs at a time so one buffer suffices, and one record layout means `--cputrace` and `--tracediff` mean the same thing on both consoles. The 65816-only fields become the constants the 6502 makes them — `D` and `DBR` zero, `E` set, the 8-bit registers in the low byte — and the cost field carries CPU cycles where the S-CPU carries master-clock ticks, which never meet because a trace is only ever compared with another trace of the same machine.

Being non-empty is not the same as being right, so the trace was checked against the cartridge rather than against expectation: the first traced PC is `$FF40`, which is the value at the ROM's own `$FFFC` reset vector; the opening stream is `SEI`/`CLD`/`LDA #`/`STA abs`/`LDX #` with `A` and `X` updating in step and `SP` at the 6502's power-on `$FD`; and **all 55,268 traced opcodes that fall in the MMC3 fixed bank match the ROM image byte for byte, with none mismatched**. A maximum instruction cost of 520 cycles is the OAM DMA stall, which is the right answer rather than a suspicious one.

**A trace ended at the next report rather than at the frame it was asked for.** This was found while checking the step count of the new NES trace: 567,455 instructions is roughly sixty frames of 6502, not the twenty that `--cputrace 20` had requested. The measurement is unambiguous — the same flag captured 179,959 steps from `startFrame` 20, 373,707 from 40 and 567,454 from 60. `PendingTraces::flush` only ran at report boundaries, so **the captured span was a function of `startFrame`**, and the flag's documented meaning, "from boot to frame F", was true only when the first report happened to land on F. It affected the S-CPU and APU traces identically and had done since the traces existed; the Phase 2 A/B agreed byte for byte precisely because both probes were wrong in the same way.

The report loop now stops at any pending trace's own frame before advancing to the next report. `--cputrace 20` records 179,959 steps from every `startFrame`, and the three files are byte-identical.

Two things had to be got right along with it, and the first attempt got the second wrong:

- `flush` now clears a due frame **whether or not the backend produced anything**. `end_trace` returning `None` is a statement about a backend's capabilities, not a transient failure, so retrying could only spin — and once the loop stops *at* a due frame, a due that never clears is an infinite loop rather than a harmless retry. Verified against libretro, which has no trace hook at all, on both the `--cputrace` and `traceUntilFrame` paths.
- A due frame **already behind the machine** is left to the next report. The first version of the fix did not check this, and it broke every GSU trace: `traceUntilFrame` is a positional whose idle value is `0`, so the loop dutifully stopped at frame 0 and wrote an empty 8-byte file where a 28.6 MB trace had been. It was caught by the same C++ baseline the port was accepted against, which is the argument for keeping a baseline around after it has served its original purpose. Three unit tests now pin the ahead/behind/at boundaries.

Also retired: §3.50 predicted that the Rust `write_trace`'s record-size validation — a `[WARN]` the C++ never emitted — would be a deliberate stdout divergence the Phase 2 A/B had to expect. It never fired. Every trace produced across both machines and all four kinds is a whole number of records, so the check guards a failure that has not yet occurred rather than a difference to account for.

#### What was retired, and one estimate that was wrong

`Reference/probe/` is gone: 8 files, 1,496 lines. `build-probe.sh` lost its `libretro-cpp` target and its C++ compile steps and is now a fetch-and-cargo script.

The plan's arithmetic does not survive contact, and the correction is worth stating because it was the headline number: C++ in this repository was predicted to fall from ~1,700 lines to ~400. It is **620** — 157 lines across five patches that modify upstream Mesen, plus 463 in the add-only C ABI — because that ABI came in far above an estimate that implicitly assumed something nearer the 112 the other patches then totalled. The direction was right and the magnitude was not: a 64% reduction rather than 76%.

**The count that actually matters was never the total.** It is the number of lines that modify code someone else maintains, and even that is no longer the 122 quoted above: `nes-cpu-trace.patch` took it to 157 by fixing a real gap. A patch set that grows because it closed a defect is not the same kind of growth as one that grows because the boundary leaked, and conflating the two is how a line count stops being a useful measure. What has not changed is the shape: still five small hooks in upstream files, still zero deletions, still one add-only file carrying everything that could be carried there.

---

### 3.53 The policy layer said it named no emulator, and named one twenty times

`backend.rs` has claimed since the Rust port that "nothing in this module names an emulator. A backend is the only place that knows what 'Mesen' or 'libretro' means; the policy layer above parses arguments, schedules presses, anchors on state and writes dumps without ever naming one." §3.45 makes the same claim for the architecture as a whole. The claim was false when it was written and stayed false through two ports.

**The argument.** A comment asserting an invariant is a prediction about future edits, and this project has now watched that prediction fail twice — once in the C++ probe, once in the transcription that inherited the comment verbatim. The interesting part is not that the leaks existed but that nobody could see them: the invariant was stated in prose in one file and violated in code in another, and no artifact connected the two. The fix that matters is therefore not the twenty edits, it is `tests::no_emulator_is_named_in_the_agnostic_layers` — the same move `PythonPaletteTests` makes in C# and the known-differences dictionary's triggers make in SQL, which §3.49 already argues for in the general case: **express the constraint as something an author cannot talk past, rather than as a convention they might forget.**

**The mechanism.** The test reads `main.rs`, `backend.rs`, `dump.rs` and `audio.rs` with `include_str!`, strips `//` comments, stops at `#[cfg(test)]`, and fails if any surviving line contains a name from a fixed list of nine emulators. Comments may still name a backend to explain why it exists — that is where the rationale belongs — and the test fixtures use real names deliberately. It reported twenty sites: three module declarations, the `DEFAULT_BACKEND` cfg pair, a usage line, the `make_backend` factory, an error message, and one that reached disk.

**The one that reached disk.** `ProbeOptions.home_folder` was built in `parse_args` as `format!("{dump_dir}/mesenhome")` — before any backend existed, so *every* backend got it. A libretro core is handed that path as its system, save and assets directory: nestopia and gambatte were writing into a folder named after Mesen, and on a libretro-only run the directory the core was told to use had never been created by anything. It is now `home_folder(dump_dir, backend.name())`, computed after the backend is constructed and before `load`, and the policy layer creates it. **Mesen's path is byte-identical** — `mesen` still yields `mesenhome` — which is why existing dump sets keep working and why this is a fix rather than a format change.

**The roster.** Backend construction moved to `src/backends/mod.rs`, which now owns the module declarations, the `DEFAULT` cfg pair, the compiled-in name list, the backend-specific usage lines and the factory. `main.rs` asks for `backends::DEFAULT` and `backends::make(name, core)` and prints `backends::usage_lines()` without knowing what any of them mean. This does not remove the knowledge — something must map a string to a constructor — it *locates* it, which is the only thing the invariant was ever asking for. `--core` survives in the policy layer as an opaque string, on the grounds that "core" names a concept rather than an emulator.

**A second-order finding.** With the profile directory now per-backend, the libretro backend's pre-load name was still the literal `"libretro"`, because it only learns its real name from `retro_get_system_info` during `load` — after `home_folder` has to be decided. Two cores probing into one dump directory would therefore have shared one profile and one save file, which is the same defect one level down. `core_stem` now derives the name from the core path (`gambatte_libretro.so` → `gambatte`) in `LibretroBackend::new`, and `load` still overwrites it with the authoritative `library_name`. Verified against gambatte and bsnes-mercury into a single directory: two profiles, correctly named.

**`gsudiff.py`, found while auditing the consumers.** It was reading the pre-Rust-port trace layout — 18 `u32` per step from byte zero — against a probe that has written an 8-byte `ESGT` header and 48-byte records with `u16` registers since §3.52. Given two byte-identical streams it reported a confident divergence at step 0 whose address was `$54475345`, which is `ESGT`. This is the failure mode §3.39 already warns about in its own terms — "a confident divergence at step 0 that is pure bookkeeping" — arriving by a different route, and it is worth noting that the warning did not prevent it, because the earlier instance was understood as being about pipeline alignment rather than about trusting a differ's frame of reference at all. Fixed, moved to `Reference/analysis/` where the stack doc says Python tools live, given a `unittest`, and given a label taken from the dump's own filename prefix so a nestopia trace is not reported as Mesen's. Its retirement for cross-core SuperFX work (§3.40, `Venus_SuperFX.md` §9) is unchanged and now recorded in the docstring rather than only here.

**What this does not cover.** Three things, deliberately:

- **The probe is emulator-agnostic, not console-agnostic.** `TraceKind::Gsu` names a SuperFX coprocessor in a trait every backend implements, `--apulog` is documented in terms of `$4000-$4017`, and the default anchor address `0x0A00` is a SNES habit. That is a different axis with a different argument — a trace kind genuinely is machine-specific in a way an emulator name is not — and the guard does not police it.
- **The guard's comment-stripping is a heuristic.** It cuts at the first `//`, so an emulator name inside a string literal that also contains `//` would be missed. No such literal exists today; the check is a ratchet against the failure that actually happened, not a proof.
- **The emulator list is manual.** A backend for an emulator not in `EMULATORS` can leak freely until someone adds the name. The list is append-only by convention, which is a convention — precisely the kind of thing this section argues is insufficient. It is accepted here because the cost of the alternative (a general "proper noun in code" check) is false positives on every ordinary identifier.

### 3.54 Three development hosts, and the difference between portable code and a portable toolchain

Asked whether EmuSen could be developed on a Mac, then on Windows, the answer divided the same way twice, and the division is the useful part of this section.

**The argument.** The emulator was already portable and nobody had noticed, because portability had never been tested at the layer where it was absent. `EmuSen.sln` is eleven `net10.0` projects with no `DllImport`, no `System.Runtime.Intrinsics.X86`, no `/proc` read and no path built from a literal separator; `out/` has been publishing `osx-arm64`, `osx-x64` and `win-x64` alongside `linux-x64` for as long as the publish list has existed. The `Avalonia.X11` package references in Hotaru, Mistress and LunaP read like a Linux dependency and are inert elsewhere. What was not portable was the *reference toolchain*: `build-probe.sh` spoke `dnf`, `rpm2cpio`, `usr/lib64` and `nproc`, `build.rs` emitted a GNU-ld flag, and `libretro.rs` imported `libloading::os::unix`. **The measure of a project's portability is its least portable required tool, not its source**, and here the source had been portable and untested for months while the tool that produces every ground-truth dump was tied to one distribution.

**The correction that matters is structural.** The first pass at this added a second branch for macOS, which is the wrong shape and was rebuilt: a toolchain acquires hosts forever, and a chain of `if host == ...` grows a bug per host. `build-probe.sh` now opens with a **host table** and nothing below it names a host — every difference is a declared capability:

| | `DEPS` | `CORE_EXT` | `EXE` | `MESEN` |
|---|---|---|---|---|
| Linux | `distro` | `so` | | yes |
| macOS | `upstream` | `dylib` | | yes |
| Windows | `upstream` | `dll` | `.exe` | **no** |

The functions branch on `DEPS`, not on the host, so macOS and Windows share one buildbot fetch that differs only in a URL fragment and an extension. A host that cannot do something declares it and is **refused with a reason**, rather than failing somewhere deep inside a toolchain that was never going to work. `need` names a missing tool instead of letting a pipeline die halfway through it. This is what "agnostic, limited by platform capability" has to mean to be worth anything: the limit is data, stated in one place, not an omission discovered by a user.

**What each host can do.**

| | build & run EmuSen | the 2,691-test suite | consume dumps | libretro probe | Mesen probe |
|---|---|---|---|---|---|
| Linux | yes | yes | yes | yes | yes |
| macOS | yes | yes | yes | yes | yes (unverified) |
| Windows | yes | yes | yes | yes (unverified) | **no — see below** |

**Why Windows has no Mesen backend.** Two independent reasons, either of which alone would be sufficient, and neither of which is a gap in this repo:

- §3.52 turns on `makefile:138` globbing `Core/Shared/*.cpp`, which is what makes `probe-c-api.patch` **add-only** and therefore incapable of conflicting on a rebase. `Core/Core.vcxproj` lists all **296** of its sources explicitly and mentions no glob, so on Windows the patch adds a file MSVC never compiles. Making it work means editing the vcxproj, which converts the patch from add-only into a project-file edit — reintroducing exactly the rebase conflict the design was built to avoid.
- §3.52 also defends a *behaviour*: the probe loads the checkout's own library by relative path, so it must be run from the checkout. A PE import table names a DLL by filename and resolves it through the DLL search order; there is no path in it to be relative. The invariant is not merely inconvenient to preserve on Windows, it is **inexpressible**.

So `build.rs` panics with that explanation when built for a Windows target, and `build-probe.sh` refuses before touching a checkout. WSL is the escape hatch and is named in both messages: it is a Linux host and gets the Linux path unchanged.

**The header pin, and why it is not a downgrade.** The libretro backend's stated invariant is that `libretro.h` comes from a real distribution rather than being vendored, so it cannot drift from the ABI a core was built against. Neither macOS nor Windows has `retroarch-devel`, and the replacement is a pinned fetch of RetroArch's `libretro-common/include/libretro.h` at tag `v1.22.2`. This was checked rather than assumed: Fedora's `retroarch-devel-1.22.0` ships a header **byte-identical** to that tag — 7846 lines, md5 `833e77b0eaa56e8f6bef5e7a00be29e7` — so the two acquisition paths are the same header by measurement. The pin is load-bearing: `master`'s header is 8300 lines and is not a released ABI.

**The Mach-O load path.** Mach-O cannot express §3.52's relative load at link time: there is no `-l:` and a consumer records the *dylib's own install name*, not the path it was linked against. Mesen's makefile sets no `-install_name` and, usefully, still does `cp $(SHAREDLIB) bin/pgohelperlib.so` on Darwin — the `.so` name survives on macOS. So `build.rs` passes the plain path there (branching on `CARGO_CFG_TARGET_OS`, not `cfg!(target_os)`, because a build script is compiled for the *host*), and `fix_mesen_load_path` rewrites the recorded name afterwards with `install_name_tool -change`, reading the old name out of `otool -D` rather than assuming `MesenCore.dylib`. The result is the same relative, cwd-resolved load the ELF probe has, reached by a different route. Contrast this with Windows above: the invariant §3.52 defends is *what the binary loads*, and that is preservable on macOS even though the link-time expression of it is not — on Windows it is not preservable at all. **A capability limit is a claim about the target format, not about effort**, which is why one host gets a workaround and the other gets a refusal.

**Loading a core on three hosts.** `libretro.rs` asked for `RTLD_NOW | RTLD_LOCAL` through `libloading::os::unix`, with the recorded reason that a core with an unresolved symbol should fail at open rather than mid-frame. That reason is host-agnostic; the flag is not. `open_core` is now two `cfg`-gated functions, and the Windows one passes no flags **because it needs none** — `LoadLibrary` resolves every import before it returns, so RTLD_NOW is the default rather than an option. The guarantee is identical; only one host has to ask for it. `core_stem` splits on both separators and strips all three extensions on every host rather than per-target, so the same core dumps into the same directory wherever it ran.

**Where the Fedora design does not generalise.** The Linux path's proudest property is that it installs nothing: RPMs are downloaded and unpacked into the work directory, so SDL2, X11 and every libretro core exist only under `~/.cache/emusen/`. That cannot hold for SDL2 on macOS. A Homebrew bottle is built for a fixed prefix and its dylibs carry absolute install names into `/opt/homebrew`; unpacking one elsewhere produces a library that links but cannot be loaded, and the failure would surface inside Mesen's build rather than here. So macOS requires `brew install sdl2-compat` and the script fails with that instruction. **The "nothing is installed" rule was a property of RPM's relocatability, not a principle the project chose**, and it is worth recording that distinction rather than quietly dropping the claim. The libretro half keeps the property on every host: cores arrive as archives and are unpacked into the work directory.

**Two negative results worth keeping.** Homebrew has **no `sdl2` formula** — checked against the full formula index, which lists `sdl12-compat`, `sdl2-compat`, `sdl2_image`, `sdl3` and the rest, but no `sdl2`. SDL2 on macOS is reached only through `sdl2-compat`, an SDL2 API implemented over SDL3, which is the same arrangement Fedora already ships and which `build-probe.sh` already carried a comment about; the platforms converged rather than diverged, which was not the expected finding. Separately, the first version of `sync-dev-host.sh` read `~/.config/EmuSen/appsettings.json` and was **wrong on every host** — that is `ConfigStore.LegacyDirectory`, while the live file for a repo run is `<repo>/home/etc/EmuSen/appsettings.json`. On the development box both exist and agree on `RomDirectory`, so it worked by luck. It now mirrors `ConfigStore`'s own order (Directory, PreviousDirectory, LegacyDirectory) and resolves `%APPDATA%` rather than `~/.config` on Windows, since that is where `SpecialFolder.ApplicationData` points there.

**The other half: being a dump consumer.** A host does not have to produce dumps to be useful, and the discovery path already half-anticipated this — `RomPath` has carried the comment "the dump may have moved between machines; the library is the answer" since the manifest format landed. `sync-dev-host.sh` makes that real for the two gitignored, ROM-derived trees git cannot carry: `Reference/dumps/` always, and the ROM library under `--roms`. It is **pull-only by construction and passes `--delete` to no rsync**, so it can only ever add files to the machine it runs on and the authoritative library is only ever read; the direction is deliberately not a flag. `appsettings.json` is not synced, because its directory settings are absolute paths belonging to the host that wrote them.

**Two things found while closing the silent-skip hole.** `ReferenceDumpTests` returns early whenever a fixture's ROM cannot be resolved, so a half-synced machine runs the whole differential suite green while comparing nothing. `Every_dump_set_that_names_its_rom_can_find_it` closes that, and writing it turned up a real defect one line away: `RomPath`'s pre-manifest fallback asked `RealRom.Find("SNES", stem + ".smc")` **regardless of the console it had just detected**, so no pre-manifest NES dump set could ever resolve. Fixed to branch on `fixture.System` the way the manifest branch two lines above already did. The new test is scoped to manifest-bearing sets on purpose: a pre-manifest set is matched by *directory name*, which was always a guess, and failing on a guess would indict the developer rather than the sync. The local `dumps/ctl/` is precisely that case and is left alone — nine NES blobs from 2026-08-03 with no manifest, whose `mesenhome/RecentGames/` breadcrumb identifies it as a Super Mario Bros 2 control run that nothing in the harness can infer from the name `ctl`.

**A test that passes is not yet evidence.** The first version of that assertion passed against a deliberately emptied `RomDirectory`, which looked like confirmation and was not: a manifest records an **absolute** ROM path, `File.Exists` on it succeeds on the machine that wrote it, and `RomDirectory` is never consulted at all. The library lookup is reached only when that absolute path is *missing* — which is exactly and only the cross-machine case. Teeth were then shown properly, with a synthetic dump set naming `/nonexistent/NotAGame.nes`: the test failed and named the directory, and the synthetic set was removed. The general point is the one §3.53 makes about guard tests: **a green guard proves nothing until it has been made to go red on purpose**, and an environment knob that looks like it should trigger it may not be on the path at all.

**What was verified, and on what.** All of it ran on Fedora, and the Linux results are measurements rather than expectations. The toolchain refactor was shown inert *before* any crate change landed: with `build-probe.sh`, and then `build.rs`, each swapped for its committed version and rebuilt, both probes came out byte-identical — libretro `357c58865ba7b8a743c0a548551a0a29`, Mesen `e9b23cd73deff5b7dce28740c9b65bea` — and `readelf -d` still showed `NEEDED bin/pgohelperlib.so`. After the whole three-host restructure the Mesen probe is *still* `e9b23cd7…`, which is not a coincidence worth glossing: it builds `--no-default-features --features mesen`, so `libretro.rs` is not compiled into it at all. The libretro probe was then run end to end against gambatte — ten frames of a real cartridge, correct `gambatte` stem, correct `gambattehome` profile directory — which is what proves the `open_core` refactor rather than merely compiling it.

**Windows was cross-checked, not assumed.** Fedora ships `rust-std-static-x86_64-pc-windows-gnu`; downloaded and unpacked into a scratch sysroot with the same `rpm2cpio` trick `build-probe.sh` already uses, `cargo check --target x86_64-pc-windows-gnu --features libretro` compiles clean with **zero warnings**, resolving `libloading 0.9.0` against `windows-link` — so the Windows loader path is genuinely selected, not merely written. The same target with `--features mesen` panics in the build script with the §3.54 message, which is the refusal doing its job. What this does *not* cover: no mingw linker is installed here, so the Windows probe has been type-checked and never linked or run.

**What is unverified.** No Mac and no Windows machine was involved in writing any of this. Specifically untested: that `brew --prefix sdl2-compat` satisfies Mesen's `sdl2-config` probe; that Mesen's Darwin link line (`-framework Foundation -framework Cocoa -framework GameController -framework CoreHaptics`) builds against a Homebrew rather than a MacPorts prefix, given the makefile hardcodes `-Wl,-rpath,/opt/local/lib`; that the six trace patches apply and compile under Apple clang; that `install_name_tool -change` produces a probe that resolves `bin/pgohelperlib.so` from the checkout's working directory; and that the buildbot's `.dll` cores load under a probe built by MSVC or mingw. The first thing to run on either host is `./build-probe.sh libretro`, which needs no Mesen checkout and exercises the header fetch, the buildbot core fetch and the extension handling in one pass. `sync-dev-host.sh` is equally untested against a real remote — only its argument handling, its config-file resolution order, its `RomDirectory` parsing (against both a real `appsettings.json` and malformed input) and the absence of `--delete` were checked here. These paragraphs should be replaced by results rather than deleted.

---

### 3.55 The reference-dump tests were 70% of the whole suite, and almost all of it was the same frames re-emulated

`dotnet test` took **6m01s** for 2,596 tests. A TRX run on a 16-core machine put **248.3s of that — 69% — inside one class**, `ReferenceDumpTests`, and a further 93.0s inside `MercuryCommercialRomTests`. The other 179 classes came to 19.6s between them. Two classes were 96% of the suite.

The cause was not the volume of comparison, it was the shape of it. Three `[Theory]` methods each took the same `(romStem, backend, frame)` and each called `Ours(fixture, romPath)` independently, and `Ours` constructed a fresh core and replayed **from boot** every time. `Reference/dumps/SMB3/` holds 75 *consecutive* dumped frames, 120 through 194, so reaching frame 194 re-emulated everything already done to reach 193 — and the whole ladder was walked three times over:

| Dump set | Frames emulated, per theory | × 3 theories |
|---|---|---|
| SMB3 (75 consecutive frames, 120–194) | sum(120..194) = 11,775 | 35,325 |
| SMW2 (frame 1500) | 1,500 | 4,500 |
| ctl (frame 300) | 300 | 900 |
| **total** | | **40,725** |

Running each set **once** to its highest dumped frame and sampling on the way past needs `194 + 1500 + 300 = 1,994` frames. The redundancy factor was **20x**, and no amount of threading recovers any of it — this is arithmetic, not scheduling.

`ReferenceDumpSnapshots` now does exactly that: it groups the discovered fixtures by `(RomStem, Backend)`, runs one core forward, and captures the memory spaces each time it reaches a frame the reference dumped. Measured, `ReferenceDumpTests` went from **248.3s to 17.4s** and the suite from 6m01s to just over two minutes. Discovery is memoized in the same file for a lesser reason: `Resolve()` was called once per test and walked the dump tree, regex-matched every blob and re-read the manifests each time.

**The assumption this rests on, and the guard that pins it.** Sampling one replay at frame N is only the same thing as booting a core and running straight to frame N if the core carries nothing over from construction. That is exactly the kind of claim that is *probably* true, is relied on by every comparison built on top of it, and was worth nothing until tested. `A_sampled_snapshot_equals_a_fresh_boot_to_the_same_frame` asserts the two byte-for-byte across every space, and `ReferenceDumpSnapshots.FreshBoot` deliberately keeps the old boot-per-frame path alive to be the other half of the comparison.

It was made to fail on purpose before being trusted, per §3.53's rule. Sampling one frame late — a single extra `RunFrame()` before capture — turned it red with `SMB3 mesen frame 120 ram: sampling the set's single replay disagrees with booting a core and running straight to that frame`. The sabotage was reverted and the guard passes.

**The battery switch, while here.** `Replay` disables the cartridge battery before constructing any core. It did not before, and SMW2 is an SRAM cartridge — so a `.srm` left beside the ROM could feed a previous run's state into a reference comparison, which is the exact failure `Venus_Memory.md` §2.4a records as having invalidated three days of state-anchored measurements. Found while reading this path for §3.56, not by a test.

**A stale assembly, and why the first measurement was wrong about the count.** The first two baseline runs reported 2,594 tests; the suite actually has 2,596. `dotnet build` reported success in 2.2s without recompiling, and the `bin/` assembly predated commit `ee78f17`, which added `FrameHandOffTests` — so two committed tests were absent from the run and from its TRX, silently, with no skip and no warning. It surfaced only because the class count moved from 181 to 182 when real compilation was finally forced. The timings above are unaffected (those two tests take 291ms), and the baseline was re-measured against a `--no-incremental` build of the unmodified code: **2,596 tests, 6m01s**. The general point is worth keeping: **a test count is part of a measurement, and a build that reports success in two seconds has not necessarily built anything.**

---

### 3.56 What the "small enough that parallelism wouldn't help" comment cost, and the two defects enabling it found

`EmuSen.WiseMan/AssemblyInfo.cs` carried `[assembly: CollectionBehavior(DisableTestParallelization = true)]` and justified it on two grounds: that `AudioPlayerTests` drives real SDL init/quit state, and that "this test suite is small enough that parallelism wouldn't meaningfully speed it up anyway."

**Correction, stated rather than quietly fixed.** The first claim is true and remains true, but it describes three classes and had been applied to all 181. The second was simply false: the suite was six minutes on one of sixteen cores. What makes it worth more than a one-line fix is that it was *self-confirming* — because parallelism was off, nobody measured what it was worth, and because nobody measured, §3.55's much larger redundancy went unfound too. The end state is **6m01s → ~53s**, and it took both changes plus a production fix to get there.

The three stages, each measured:

| | Wall clock | What changed |
|---|---|---|
| Baseline | 6m01s | 2,596 tests, one core |
| §3.55 only | 2m06s | single-pass dump snapshotting; still effectively serial |
| §3.55 + this | **~53s** | `MercuryCommercialRomTests` split, one serial collection |

Worth being exact about the middle row, because it is the interesting one. With §3.55's fix in place, running the suite *fully serial* took 2m09s against 2m06s in parallel. **Parallelism was worth about 3 seconds at that point** — the cost had simply moved into a different single class, and xUnit's unit of parallelism is the class. Threading only started paying once that class was split.

**Splitting the long pole.** `MercuryCommercialRomTests` was 93.8s in 42 tests: 6 theory methods over 7 real cartridges, all serial inside one class. It is now six classes — `MercuryCommercialRomBootTests`, `…DeterminismTests`, `…SaveStateTests`, `…RenderSkipTests`, `…AudioTests`, `…BatteryTests` — over a shared `MercuryCommercialRom` helper holding `Load`, `RunTo` and the frame counts. They run concurrently at 22.5s, 22.1s, 19.9s, 17.2s, 10.9s and 8.7s. The determinism test still boots twice on purpose; sharing a cached run there would assume exactly what it exists to prove.

**The production change that made the split legal.** The six classes could not leave the serial collection while `CoreOptions.BatteryRamDisabled` was read live, because `CartridgeSavePathTests` was the one class in the suite that ever assigned it `false` while ~10 classes needed it `true`, and xUnit collections cannot express mutual exclusion.

`CoreOptions` describes itself as "run switches a frontend sets **before** `LoadRom`", and `EmuSen_Multicore.md` §6 says the same in stronger terms. Mercury and Moon already honoured that — both read the flag once in `LoadSram()` during `LoadRom` and latch the outcome in `_savePath`. **Venus was the outlier**, re-reading the static on every save and load. It now takes a copy at construction (`Cartridge(string romPath, bool? batteryRamDisabled = null)`), which enforces the documented contract rather than changing it: nothing in the project assigns the switch after a cartridge exists (`Program.cs` sets it once from `--nobattery`, `TestRomRunner` sets it once). `CartridgeSavePathTests` now passes the value per cartridge and never touches the global, so **every remaining assignment in the suite is `true`** and concurrent same-value writes are harmless.

`The_battery_switch_is_taken_at_construction_not_read_live` pins it, and was made to fail on purpose: restoring the live read turns it red with *"the cartridge re-read CoreOptions at save time instead of latching it at construction"*, and takes three other save-path tests with it. Reverted, all pass.

**Defect one: a process-wide trace buffer, found by turning parallelism on.** The first parallel run failed with `System.IndexOutOfRangeException` in `CpuBinaryTrace.Record`, raised from `FrameRunnerTests` — a class that never mentions `CpuBinaryTrace`. That class holds one `byte[] _buffer` and one `int _length` for the entire process. `Record()` bounds-checks `_buffer`, then takes local copies and indexes; `Reset()` reassigns `_buffer` to `Array.Empty<byte>()`. A torn read between the two is the crash.

It is **not** fixed in production. The buffer is a process-wide singleton deliberately, because the DianaOS `trace` command owns it, and `trace` reports `IsReadOnly = false` so the always-live console queues it onto the emulation thread — production never races it. Making it per-core is a design question, not a test-suite question.

**Defect two, the one that decided the collection layout.** `Cpu.cs` sets `_traceBinary = name == "CPU"`, so *every* main 65816 records whenever the global `CpuBinaryTrace.Enabled` is true. The exposure is therefore not the four classes that mention tracing — it is **every class that runs a Venus core at all**, seventeen of them, which would both crash on the buffer swap and corrupt the trace tests' own assertions about `Count`. This is why the arrangement is one collection rather than four.

**`"Process globals"`** — 45 classes, ~31s combined, serial; the remaining ~145 run in parallel. It holds four kinds of shared state at once:

- **config** — `ConfigStore`/`DataStore`/`ConfigFile` overrides and `CheatDatabaseInstaller.FetchOverride`, plain statics set in a constructor and nulled in `Dispose`. `DianaOS/HierTests` is in here despite writing nothing: it depends on the *un*-overridden value, which makes it a victim rather than an aggressor and is easy to miss when auditing by grep for assignments.
- **SDL** — the three classes the original comment was actually about.
- **trace** — `CpuBinaryTrace`/`GsuBinaryTrace`/`ApuWriteTrace`.
- **Venus** — every class reaching `Cpu.Step()`, per defect two.

They are one collection and not four because `AudioLatencyDriftTests` drives SDL *and* a real Venus core, and xUnit gives a class exactly one collection. Merging costs little: the serial group is ~31s, of which `ReferenceDumpTests` is 17.0s and `AudioLatencyDriftTests` 5.7s.

**The remaining ceiling.** The serial group is now the critical path at ~31s against a ~53s wall, and `ReferenceDumpTests` is more than half of it — it is in the group only because it drives a Venus core. Hardening `CpuBinaryTrace.Record` to re-check bounds against its own local copy, which matches the "stops recording rather than throwing" rule `Grow()` already follows, would remove the crash but **not** the trace-buffer pollution that breaks the trace tests' assertions, so it is not on its own sufficient. Recorded as the next available step, not as something done.

**A hazard closed while here.** `ReferenceDumpTests` loaded SMW2 — an SRAM cartridge — without ever setting the battery switch, so a stray `.srm` could feed a previous run's state into a reference comparison. That is precisely the failure `Venus_Memory.md` §2.4a records as having invalidated three days of state-anchored measurements. `ReferenceDumpSnapshots.Replay` now disables the battery before constructing any core.

**Verification.** 2,598 tests pass — the 2,596 the suite actually had, plus §3.55's snapshot guard and this section's latch guard. Three consecutive full runs at 52.7s, 52.8s and 54.0s, no intermittent failures, with `user` time 2m20s against 54s wall confirming the work is genuinely spread across cores. Repeating the run matters here specifically because parallelism converts latent shared-state bugs into flakes, and a single green run establishes nothing about whether the collection is drawn correctly.

### 3.57 The fifth kind of process global, found by a flake rather than by the audit

§3.56 enumerated **four** kinds of shared state in `"Process globals"` — config, SDL, trace, Venus. There were five. `FirmwareLibrary.Directory` is a plain static, assigned in the constructors of `FirmwareLibraryTests` and `NecDspFirmwareRequestTests` and nulled by both in `Dispose`, and neither class was in the collection.

**How it presented.** `FirmwareLibraryTests.The_canonical_name_wins_over_an_alternate` failed once, then passed on a re-run against *identical binaries* with no rebuild in between. That is the least useful shape a failure can have — it looks like noise, and the natural response is to run it again and move on, which is exactly what happened the first time it was seen.

**The reproduction, because "it is a race" is a guess until it is a measurement.** Filter the run down to just the two classes that touch the static, so they are the only things scheduled and are therefore guaranteed to be concurrent:

    dotnet test --filter "FullyQualifiedName~FirmwareLibraryTests|FullyQualifiedName~NecDspFirmwareRequestTests"

Eight runs before the fix: **1 pass, 7 failures.** Eight runs after: **8 passes.** In the full suite the same race surfaces at roughly one run in ten, because the two classes are competing with 180 others for a scheduling slot rather than with each other.

**Re-measured before the commit landed, and the first numbers above did not survive it.** Ten runs of the same filter with the attributes removed again: **ten failures, no passes** — the race is not one-in-eight under the filter, it is essentially certain. More usefully, the test that fails is not the one this section was written around. Every one of the ten failed `NecDspFirmwareRequestTests.Installing_the_dump_settles_the_request_and_builds_the_chip`, and two of the ten *also* failed `FirmwareLibraryTests.Installing_the_wrong_file_fails_without_writing_anything`. `The_canonical_name_wins_over_an_alternate` — the flake that started the investigation, and the test named in the paragraph above — did not fail once in ten targeted runs.

That is worth stating plainly rather than quietly correcting, because it is a claim this document made and then failed to reproduce: **the test that reveals a shared-state race depends on the scheduling context, so the one you happened to see is not the one a reproduction will hand you.** The original observation stands as an observation — it was seen, in a full parallel run, and the re-measurement does not contradict it. What was wrong was the implied equivalence between "the flake I saw" and "what this command reproduces".

**The mechanism**, corrected to the arm that actually reproduces. Each class makes its own temp directory per instance, which is what makes the fixture look airtight. Then it points the process-wide `FirmwareLibrary.Directory` at it. Interleave the two and one class writes through the static while the other has already moved it:

- **The dominant arm, 10/10.** `Installing_the_dump_settles_the_request_and_builds_the_chip` calls `FirmwareLibrary.Install`, which writes `dsp1.rom` into whatever `Directory` names *at that moment*, then asserts `MissingFirmwareFor` is empty — a second read of the same static. Between the two, `FirmwareLibraryTests` repoints it, so the read looks in a directory the file was never written to and the request is still reported missing. It fails as `Assert.Empty() Failure: Collection was not empty`, listing the `dsp1.rom` request that was just satisfied.
- **The arm this section was written around.** The same write-then-read shape against a `Dispose` that nulls the static rather than repointing it, so `TryLoad` returns null and `TryLoad(...)![0]` throws. It needs the null to land inside a narrower window, which is why it is the rarer of the two and why it showed up in the full suite rather than under the filter.

Both are one bug and one fix. The distinction matters only for what a reproduction should be expected to print.

**Why the audit missed it, which matters more than the fix.** §3.56 found its four kinds by auditing state the suite was *already known* to share — the SDL init state the original comment named, then the trace buffer and the Venus cores that enabling parallelism crashed on immediately. Firmware crashed nothing. It produced one intermittent failure in a class whose per-instance temp directory reads as careful isolation, and careful-looking isolation is what makes the shared static easy to read straight past.

The reliable question is not "does this class share state", which needs judgement, but **"does this class assign a static"**, which `grep` answers. Running that question over every test class in `EmuSen.WiseMan` returns exactly two more sites, and both are already accounted for: `CoreOptions.BatteryRamDisabled = true` in `MercuryHardwareTestRomTests`, which is the same-value write §3.56 records as deliberately safe, and a false positive on a `switch` arm. **There is no sixth kind.** That sweep is written into `TestCollections.cs` beside the list, so the next person asking the question has the method rather than only the answer.

**Cost and verification.** The serial collection goes from 45 classes to 47 and gains 25 tests taking ~45ms. Three consecutive full runs: **2,621 tests pass** at 55.8s, 56.2s and 53.4s, against §3.56's ~53s — so the whole fix costs about two seconds of wall clock, and repeated runs are what establishes that, per §3.56's own rule that one green run establishes nothing.

Re-confirmed at commit time: full suite **2,621 passed, 0 failed, 53s**; the filtered pair **8 green runs out of 8** at 25 tests and ~45ms, against the ten-out-of-ten failures the same filter produces with the two attributes taken back off. The attributes were removed and restored from a copy to measure that, rather than reasoned about.

---

## 8. A note on the 2026-08-06 commit, for whoever runs `git log` and wonders

Four efforts landed in one commit, which is not the house style and is worth explaining rather than leaving as an oddity.

The Crystal fold (the master clock, scanline boundary and CPU-budget accumulator moving out of the deleted `EmuSen.Crystal` and into Moon and Venus, with the PPU gaining a per-dot `Ppu.Timing.cs`), the assembly pruning it belonged to (`EmuSen.Nehellania` folding into `EmuSen.Endymion`), the Venus renderer work alongside it, and the reference-probe rewrite in §3.45 were all in flight at the same time. They overlap in `MoonCore.cs` and `MemoryBus.cs` — the probe's APU write hook sits directly on top of the new timeline's frame counter — and there is no split by path that leaves both halves compiling. Committing the probe work alone produced a tree that failed with twenty errors; adding only the Moon core left nine; the smallest set that builds is all of it.

The lesson is the ordinary one about interleaving refactors, and the reason it is written down here rather than regretted quietly: **when a change needs a field that an in-flight refactor is still moving, the two are one commit whether or not they were one intention.** The alternative — landing the probe work first against the old timeline and rewriting it after — would have cost more than the muddled history does.

Each effort keeps its own documentation, and those are the places to read: `Moon_Core.md` §5 and `Venus_CPU.md` §8.5d for the timeline, `EmuSen_LunaP.md` §16.1 and the assembly-pruning notes for the pruning, §3.45 and §3.46 above for the probe.
