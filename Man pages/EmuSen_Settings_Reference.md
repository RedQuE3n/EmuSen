# EmuSen Settings Reference

Covers three static, plain-field "central hub" classes the rest of the codebase reads instead of hardcoding constants or scattering `Console.WriteLine` gates through individual files: `DebugSettings.cs` and `AudioSettings.cs` (both `EmuSen/Settings/`), and `GraphicsSettings.cs`, which lives in `EmuSen.Serenity/` instead — it moved there alongside `FramePresenter`/`BuiltInShaders` since it's presentation-layer config any frontend can share, not something coupled to the emulation core the way `DebugSettings`/`AudioSettings` are. Same reorganization as `Man pages/Hardware/` (see that folder's `README.md` for the full rationale): the long inline WHY-comments that used to live next to each flag now live here, and code comments stay short.

Unlike `Man pages/Hardware/`, these aren't hardware-abstraction notes — none of `DebugSettings`/`AudioSettings`/`GraphicsSettings` model real SNES hardware; they're this emulator's own configuration surface. That's why this page sits directly under `Man pages/` rather than under `Hardware/`.

---

## 1. `DebugSettings.cs` — diagnostic/logging toggles

Central hub for every debug/diagnostic toggle across the emulator, flipped here instead of hunting through individual files. Renamed from their original per-class names (several were all called `VerboseLogging` on different classes, which doesn't work once collected into one place) but otherwise behave identically to before.

Most of these flags aren't generic "turn on more logging" switches — they were each added to answer one specific historical investigation question, and the doc below preserves that context so a future investigator knows what a flag was originally *for* before reaching for it (or for the newer, core-agnostic `Debug/` toolchain instead — see §1.5).

- **`MasterLoggingEnabled`** (default `true`) — silences every `*Logging` flag below at once without changing any of their individually-set values: each is a property (not a plain field) whose getter ANDs its own backing value against this, so flipping it off and back on restores exactly whatever was individually configured before. Every call site throughout the codebase needed zero changes, since they already just read e.g. `DebugSettings.CpuVerboseLogging` — the intersection happens transparently in the property. Toggle live from the F4 prompt via `log on` / `log off` / `log status` (`DianaOS/Commands/LogCommand.cs`) instead of rebuilding. Doesn't touch `HvIrqEnabled` or `WindowingEnabled` — those are emulation-behavior toggles, not logging output, and this switch is scoped to trace output only.

### 1.1 CPU (`Cpu.cs`)

- **`CpuVerboseLogging`** — logs every executed instruction (PC, opcode, name, target address). Very high volume; intended for short, targeted traces, not routine play.
- **`CpuTraceCountdown`** — companion to `CpuVerboseLogging`: set to a positive number alongside enabling that flag to log that many more instructions and then auto-disable. Leave at 0 for an unbounded trace (rarely what you want) — a *bounded* trace was what found the H-blank wait-loop bug.
- **`HvIrqEnabled`** — master on/off switch for H/V-IRQ support (`$4200` bits 4-5, `$4207-$420A` — see `Venus_Memory.md` §4). A real, working feature (confirmed fixing a title-screen lockup via the H-blank flag), not a "broken, avoid" toggle — kept as a convenient full disable if a future regression needs isolating.

### 1.2 DMA/HDMA (`Dma.cs`)

- **`DmaVerboseLogging`** — logs every general (one-shot) DMA transfer: channel, direction, source, destination register, size. High volume during level loads. Originally enabled specifically to catch SMW's coin tile-animation upload (a periodic VRAM CHR rewrite via `$2118`/`$2119`, per SMWCentral's description of coins being animated as tiles rather than sprites) — look for repeating small transfers to `$2118`/`$2119` at a consistent destination VRAM address, roughly once per frame or every few frames, landing during vblank.
- **`DmaSourceAddrLogging`** — logs the CPU PC responsible for every DMA channel source-address write (see `Dma.cs`'s `LogSourceAddrWrite`, and `Venus_Memory.md` §3.3). Added for the Yoshi sprite-graphics DMA investigation — a constant, never-changing source address is the signature of an uninitialized or wrongly-computed pointer, and this flag shows which PC is responsible. Separate from `DmaVerboseLogging` so it can be toggled independently — narrower and more targeted than general DMA visibility.
- **`WindowHdmaLogging`** — logs HDMA writes/block-fetches specifically for the window-position registers (`$2126-$2129`) on channel 7, used to trace a title-screen window-wipe effect. High volume: fires every scanline while that channel is active.

### 1.3 APU (`Spc700.cs`)

- **`Spc700VerboseLogging`** — logs every executed SPC700 instruction, same tradeoffs as `CpuVerboseLogging` but for the audio CPU.

### 1.4 PPU/Renderer (`Ppu.cs`, `MemoryBus.cs`, `Renderer.cs`)

Several of these were added together to chase down one bug this project's history calls the **"BG2 parallax jitter"** — the scroll-register issue whose real fix (a separate write-twice latch for `BGnHOFS`'s `Prev2` term) is documented in `Venus_PPU.md` §2. They're grouped here because that's how they were actually used: turned on together, cross-referenced against each other's output to narrow down where in the pipeline (CPU math → PPU register write → renderer read) the jitter actually originated.

- **`CgWriteLogging`** — logs every CGRAM color write (index, palette/entry, raw bytes). High volume — a full palette refresh alone is 100+ writes.
- **`Bg3ScrollWriteLogging`** — logs every write to BG3's scroll registers specifically.
- **`CameraRamLogging`** — logs writes to `$001A-$0021`, the zero-page RAM bytes the title-screen/level NMI routine reads from immediately before flushing them to BG1HOFS/VOFS (`$210D`/`$210E`) and BG2HOFS/VOFS (`$210F`/`$2110`), per the MesenCE disassembly. Used to check whether the jitter originated *upstream* of the PPU register write — i.e. whether the CPU was computing a different raw camera value than real hardware — once the write-twice register formula and write order were both confirmed correct against MesenCE. Only logs on actual value change, so it stays low-volume.
- **`RenderReadLogging`** — logs `BgScrollX[1]`/`BgScrollY[1]` (BG2) at the moment `RenderBg2` reads them for scanline 0 of each frame, tagged `[RENDER-READ]`. Compared against the last `[SCROLL]` BG2 write logged for that frame: if they ever differ, the renderer is sampling before the NMI handler's writes have landed — a timing bug distinct from the write-twice register formula itself.
- **`AllScrollWriteLogging`** — logs every write to all four backgrounds' scroll registers, in order, with the shared latch value going into each calculation. Used to check whether BG3's known-zero writes were interleaving with BG2's and corrupting the shared latch all scroll writes depend on.
- **`HvIrqChangeLogging`** (`MemoryBus.cs`) — logs `$4200`/`$4207-$420A` writes when they actually change the H/V-IRQ enable bits or HTIME/VTIME. Low volume (only logs on real change) — safe to leave on if debugging IRQ timing again.
- **`MathUnitLogging`** (`MemoryBus.cs`) — logs every hardware multiply/divide operation (`$4203`/`$4206` triggers) with operands and result. Also part of the parallax-jitter investigation — checking whether SMW's scroll math depends on this unit and whether the results looked sane.
- **`BgModeChangeLogging`** (`MemoryBus.cs`) — logs `$2105` (BGMODE) writes when the byte actually changes, broken out into mode/BG3-priority/per-BG tile-size bits. Low volume (games set this rarely, usually once per level) — added alongside the 16x16 BGMODE tile-size support in `Renderer.Backgrounds.cs` so a play session can confirm whether a given level ever actually sets bits 4-7, rather than trusting that fix blind.
- **`MosaicWriteLogging`** (`MemoryBus.cs`) — logs every `$2106` (MOSAIC) write with the scanline it landed on. Added alongside the mosaic starting-scanline latch (`Ppu.MosaicStartScanline`) to confirm whether a level ever writes this mid-frame (outside vblank) — the only case where that latch does anything different from the old always-anchor-at-0 behavior. Low volume — most games only touch this in NMI.

### 1.5 Windowing (`Renderer.cs`)

- **`WindowingEnabled`** — master on/off switch for window masking (`$2123-$212F` — see `Venus_PPU.md` §7). **Re-enabled** after a period of being off: the masking logic itself was independently verified correct against three sources (SMWCentral, the SNESdev wiki, fullsnes) when it was originally built. It was switched off because *one specific, separate* bug (a stuck HDMA table on a title screen — a CPU-side issue unrelated to windowing itself) froze a window at a single pixel and hid most of the screen there. But leaving windowing off globally to avoid that one scene meant every *other* game effect relying on windows (status bar splits, spotlight/darkness effects) silently rendered with no windowing at all — a much bigger correctness cost than the one known, already-identified title-screen bug. If that specific scene regresses, it's a real but separate, already-tracked HDMA bug to chase down on its own, not a reason to disable this globally again.

### 1.6 `DebugSettings` vs. the `Debug/` toolchain

These flags predate — and still coexist with — the core-agnostic `Debug/` toolchain (`WatchRegistry`, `FrameLogRegistry`, the `watch`/`framelog`/`search`/`callers` commands documented in `EmuSen_Debugging_Tools_Reference_v5.md`). A `DebugSettings` flag hardcodes *what* gets logged and *when* at compile time; the newer toolchain lets you register a watch/frame-log/search *at runtime* through the F4 debug console, without a rebuild. New investigations should generally prefer the `Debug/` toolchain where it covers the need — these flags remain because migrating each one isn't automatic (some, like the parallax-jitter group, log derived/contextual data a generic watch wouldn't reconstruct on its own), not because they're the preferred pattern going forward.

---

## 2. `AudioSettings.cs` — audio output configuration

`SDsp.cs` generates real audio (BRR-decoded voices, ADSR/GAIN envelopes, MVOL-scaled mixing — see `Venus_APU.md` §§3-5), though echo/noise/pitch-modulation aren't implemented yet. **Nothing plays `AudioBuffer`'s contents to an actual output device yet** — that's the remaining piece for audible sound; these settings configure generation, not playback.

- **`SampleRate`** (default 32000 Hz) — the DSP output rate. A real hardware fact, not a preference, but named here rather than left as a bare `32000` (or the derived "32 cycles per sample" at the SPC700's ~1.024MHz clock) scattered inside `SDsp.cs`.
- **`AudioBufferMaxSamples`** (default 64000) — caps `AudioBuffer`'s size (in samples, not sample-pairs) so it can't grow unbounded if a future output-device consumer falls behind. 64000 ≈ 1 second of 32kHz stereo audio.
- **`MasterVolume`** — overall gain applied to the final mixed stereo sample. Wired into `SDsp.GenerateSample`.
- **`Muted`** — silences output without touching `MasterVolume`. Voice playback/envelope state still advances while muted (see `Venus_APU.md` §3.4), so audio doesn't jump ahead the moment this is turned back off.
- **`AudioEnabled`** — master on/off for audio generation entirely.

---

## 3. `GraphicsSettings.cs` — display/presentation configuration

Window size, title, vsync, target frame rate, texture filtering, and whether to show the debug side panels. **None of this affects emulation correctness** — that's `DebugSettings`'s domain. This is purely "how the output window looks." Lives in `EmuSen.Serenity/GraphicsSettings.cs` (namespace stays `EmuSen.Graphics`) and is read by both `EmuSen.Serenity.FramePresenter` (window/render-target/letterbox setup) and `Renderer.Debug.cs`'s `DrawDebugPanels` (the `ShowDebugPanels` gate) — the split exists because `Renderer` stayed in the emulation-core project while window ownership moved out to `FramePresenter`, see `EmuSen_Frontend_Driver.md` §1 step 6.

- **`WindowWidth`/`WindowHeight`** (default 1060x580) — window/canvas size in pixels. The real SNES resolution (256x224) is fixed hardware fact and lives in `Renderer.cs` as `ScreenW`/`ScreenH`, not here — these settings just control how big the window is on screen, with room for the debug side panels alongside the game view.
- **`WindowTitle`** — default `"EmuSen"`.
- **`TargetFps`** (default 60) — the emulator's own frame-pacing target, separate from `VSyncEnabled`, which asks the OS/driver to sync to the display's refresh rate.
- **`VSyncEnabled`**, **`WindowResizable`** — self-explanatory Raylib window flags.
- **`BilinearFiltering`** — Bilinear smooths the upscaled image; Point (the alternative) keeps hard pixel edges (the classic "sharp pixel" look). Point is generally more period-authentic for pixel art; Bilinear can look better at non-integer scale factors.
- **`ShowDebugPanels`** — show the VRAM tile sheet + CGRAM palette panels alongside the game view (see `Renderer.Debug.cs`). Turn off for a clean, game-only window. **Known layout limitation**: turning this off currently just skips drawing the panels — the game view itself stays in its current position/size rather than re-centering to fill the freed space, which would be a separate, bigger layout change.
- **`PanelBackgroundColor`** — background behind the debug panels (the dark canvas the game view and side panels sit on top of).
- **`LetterboxColor`** — color of the letterbox bars around the final scaled output when the window aspect ratio doesn't match the virtual canvas.
