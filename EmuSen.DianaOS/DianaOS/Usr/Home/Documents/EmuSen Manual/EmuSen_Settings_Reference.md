# EmuSen Settings Reference

Covers three static, plain-field "central hub" classes the rest of the codebase reads instead of hardcoding constants or scattering `Console.WriteLine` gates through individual files: `DebugSettings.cs` and `AudioSettings.cs` (both `EmuSen/Settings/`), and `GraphicsSettings.cs`, which lives in `EmuSen.Serenity/` instead — it moved there alongside `FramePresenter`/`BuiltInShaders` since it's presentation-layer config any frontend can share, not something coupled to the emulation core the way `DebugSettings`/`AudioSettings` are. Same reorganization as `Man pages/Hardware/` (see that folder's `README.md` for the full rationale): the long inline WHY-comments that used to live next to each flag now live here, and code comments stay short.

Unlike `Man pages/Hardware/`, these aren't hardware-abstraction notes — none of `DebugSettings`/`AudioSettings`/`GraphicsSettings` model real SNES hardware; they're this emulator's own configuration surface. That's why this page sits directly under `Man pages/` rather than under `Hardware/`.

---

## 1. `DebugSettings.cs` — diagnostic/logging toggles

Central hub for every debug/diagnostic toggle across the emulator, flipped here instead of hunting through individual files. Renamed from their original per-class names (several were all called `VerboseLogging` on different classes, which doesn't work once collected into one place) but otherwise behave identically to before.

Most of these flags aren't generic "turn on more logging" switches — they were each added to answer one specific historical investigation question, and the doc below preserves that context so a future investigator knows what a flag was originally *for* before reaching for it (or for the newer, core-agnostic `DianaOS/` toolchain instead — see §1.6).

- **`MasterLoggingEnabled`** (default **`false`**) — silences every `*Logging` flag below at once without changing any of their individually-set values: each is a property (not a plain field) whose getter ANDs its own backing value against this, so flipping it off and back on restores exactly whatever was individually configured before. Defaults to `false` specifically because several individual flags below default to `true` (leftover from the investigations that added them) - without this master switch, a stock build logged a steady stream of DMA/scroll/math-unit traces on every run whether anyone asked for it or not. **Nothing logs until this is turned on**, regardless of how many individual flags below are already `true`. Every call site throughout the codebase needed zero changes for this, since they already just read e.g. `DebugSettings.CpuVerboseLogging` — the intersection happens transparently in the property. Toggle live from the F4 prompt via `log on` / `log off` / `log status` (`DianaOS/Commands/LogCommand.cs`) instead of rebuilding. Doesn't touch `HvIrqEnabled` or `WindowingEnabled` — those are emulation-behavior toggles, not logging output, and this switch is scoped to trace output only.

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
- **`DspKeyOnLogging`** (`DspVoice.cs`, not `Spc700.cs` - still APU-domain, just a different file) — logs every KeyOn (note trigger): SRCN, the resolved sample-directory entry, computed start/loop address, the BRR header byte actually found there, and the voice's pitch/volume. Added to check whether the SPC700 sound driver is triggering voices with sane-looking sample pointers at all, independent of whatever `BrrDecoder`/`DspVoice` do with them afterward.

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
- **`ColorMathBlendLogging`/`ColorMathBlendScanline`** (`Renderer.Scanline.cs`) — a separate investigation from the parallax-jitter group above: logs both operands going into a `$2131` color-math blend for one target scanline, added for the Zelda: A Link to the Past color-math investigation (an aggregate "wrong blended color" symptom gave no way to see which of the two blend operands was actually wrong without this). `ColorMathBlendScanline` picks which scanline to watch (default `-1`, never matches a real one, so this stays inert until both it and the logging flag are set).

### 1.5 Windowing (`Renderer.cs`)

- **`WindowingEnabled`** — master on/off switch for window masking (`$2123-$212F` — see `Venus_PPU.md` §7). **Re-enabled** after a period of being off: the masking logic itself was independently verified correct against three sources (SMWCentral, the SNESdev wiki, fullsnes) when it was originally built. It was switched off because *one specific, separate* bug (a stuck HDMA table on a title screen — a CPU-side issue unrelated to windowing itself) froze a window at a single pixel and hid most of the screen there. But leaving windowing off globally to avoid that one scene meant every *other* game effect relying on windows (status bar splits, spotlight/darkness effects) silently rendered with no windowing at all — a much bigger correctness cost than the one known, already-identified title-screen bug. If that specific scene regresses, it's a real but separate, already-tracked HDMA bug to chase down on its own, not a reason to disable this globally again.

### 1.6 `DebugSettings` vs. the `DianaOS/` toolchain

These flags predate — and still coexist with — the core-agnostic `DianaOS/` toolchain (`WatchRegistry`, `FrameLogRegistry`, the `watch`/`framelog`/`search`/`callers` commands documented in `EmuSen_Debugging_Tools_Reference_v5.md`; merged from a separate `Debug/`/`EmuSen.Debug` namespace some time ago - only `DebugSettings` itself stayed in `EmuSen.Debug`, see that doc's own §3.3 note). A `DebugSettings` flag hardcodes *what* gets logged and *when* at compile time; the newer toolchain lets you register a watch/frame-log/search *at runtime* through the always-live DianaOS terminal or the F4 debug console, without a rebuild. New investigations should generally prefer the `DianaOS/` toolchain where it covers the need — these flags remain because migrating each one isn't automatic (some, like the parallax-jitter group, log derived/contextual data a generic watch wouldn't reconstruct on its own), not because they're the preferred pattern going forward.

---

## 2. `AudioSettings.cs` — audio output configuration

`SDsp.cs` generates real audio (BRR-decoded voices, ADSR/GAIN envelopes, MVOL-scaled mixing — see `Venus_APU.md` §§3-5), though echo/noise/pitch-modulation aren't implemented yet. Both `EmuSen.Hotaru` and `EmuSen.Mistress` play `AudioBuffer`'s contents to a real output device via their own `Audio/AudioPlayer.cs` (SDL's queue-based API, no callback) — these settings configure both generation and the buffer that sits between generation and playback.

- **`SampleRate`** (default 32000 Hz) — the DSP output rate. A real hardware fact, not a preference, but named here rather than left as a bare `32000` (or the derived "32 cycles per sample" at the SPC700's ~1.024MHz clock) scattered inside `SDsp.cs`.
- **`AudioBufferMaxSamples`** (default 128000 = 2s) — a **safety valve, not a resync**. `AudioPlayer.Pump` drains this buffer completely every call now, so it only fills when nothing is draining at all (headless `EmuSen.Pharaoh` runs, a paused session, no audio device). It trims in L/R pairs as a plain ring rather than cutting to a second threshold. Raising it from 250ms also means `audiodump` captures 2s instead of a quarter second. See `EmuSen_Audio_Sync.md` §4.
- **`OutputTargetLatencyMs`** (default 256ms) — where dynamic rate control steers the *output device* queue, which is where all buffering lives now. Deliberately `2 x` SDL's own 4096-frame device buffer; a target under one full device buffer underruns constantly. The lever for lower latency is `BufferFrames` in `AudioPlayer`, not this. See `EmuSen_Audio_Sync.md` §5.
- **`RateControlMaxDeviation`** (default 0.005) — how far the resample ratio may depart from 1.0 to absorb clock drift. ±0.5% is inaudible and matches bsnes/ares/RetroArch. It is also the loop's authority limit: drift beyond it cannot be corrected. See `EmuSen_Audio_Sync.md` §3.

**The 150ms-discard mechanism this section used to describe is gone.** Three independent oscillators (SNES video, SNES audio, host sound card) mean *some* correction is permanently required, and the old approach paid for it by deleting 150ms of audio mid-waveform — a gap and a click — whenever the buffer passed 250ms. Two throttles were also fighting: the consumer refused to drain while the device queue was full, which let the producer's buffer fill and trip that very discard. It is replaced by dynamic rate control: drift is absorbed continuously by resampling within ±0.5%, and nothing is discarded in normal play. The core-side backlog measures 0.0ms at baseline, after a one-second stall, and after recovery, where it previously sat on a ~256ms floor. Full design: `EmuSen_Audio_Sync.md`.

`AudioBufferResyncTargetSamples` is removed. The latent left/right channel swap that setting could cause (draining one `short` at a time could leave an odd count) is gone with it — the ring trims in pairs by construction.

- **`MasterVolume`** — overall gain applied to the final mixed stereo sample. Wired into `SDsp.GenerateSample`.
- **`Muted`** — silences output without touching `MasterVolume`. Voice playback/envelope state still advances while muted (see `Venus_APU.md` §3.4), so audio doesn't jump ahead the moment this is turned back off.
- **`AudioEnabled`** — master on/off for audio generation entirely.

---

## 3. `GraphicsSettings.cs` — display/presentation configuration

Window size, title, vsync, target frame rate, and texture filtering. **None of this affects emulation correctness** — that's `DebugSettings`'s domain. This is purely "how the output window looks." Lives in `EmuSen.Serenity/GraphicsSettings.cs` (namespace stays `EmuSen.Graphics`) and feeds `EmuSen.Hotaru`'s `GameWindow` (`Title`/`Width`/`Height`/`CanResize`) and `GameFrameControl`'s `BilinearFiltering`-driven `SKSamplingOptions` — Avalonia/Skia now, not Raylib. See `EmuSen_Frontend_Driver.md`'s own top-of-file revision note for the Raylib→Avalonia migration this doc reflects.

- **`WindowWidth`/`WindowHeight`** (default 1060x580) — window size in pixels. The real SNES resolution (256x224) is fixed hardware fact and lives in `Renderer.cs` as `ScreenW`/`ScreenH`, not here — `GameFrameControl.ComputeLetterboxRect` fits that native frame into whatever the window's actual bounds turn out to be, so these settings are just the window's own initial/default size, not a fixed reference canvas the game has to share space with.
- **`WindowTitle`** — default `"EmuSen"`.
- **`TargetFps`** (default 60) — not currently wired to anything (kept for parity/future use); `VSyncEnabled` separately asks the OS/driver to sync to the display's refresh rate.
- **`VSyncEnabled`**, **`WindowResizable`** — self-explanatory Avalonia `Window` settings (`CanResize`, etc.).
- **`BilinearFiltering`** — Bilinear smooths the upscaled image; Point (the alternative) keeps hard pixel edges (the classic "sharp pixel" look). Point is generally more period-authentic for pixel art; Bilinear can look better at non-integer scale factors. Wired via `SKSamplingOptions(SKFilterMode.Linear/Nearest, ...)` in `GameFrameControl`'s draw path.

**`ShowDebugPanels`/`PanelBackgroundColor`/`LetterboxColor` are gone** — they existed to configure the on-window Raylib debug overlay (VRAM tile sheet + CGRAM palette panels drawn directly onto the game window), which was removed entirely as part of the Raylib→Avalonia migration: `DianaOS`'s `regs`/`sprites`/`pal`/`tile`/`vramsheet`/`paletteswatch` commands and the `coretop` dashboard already cover the same data, and `EmuSen.Mistress` never had this overlay at all, which was the strongest evidence it wasn't load-bearing. See `EmuSen_Frontend_Driver.md`'s own top-of-file revision note and `EmuSen_Debugging_Tools_Reference_v5.md`'s own revision note on the same change.

---

## 4. `EmuSen.Mistress` — the frontend's own input settings

Everything above is emulator configuration. This section covers the per-user settings the Mistress GUI owns: keyboard/gamepad bindings and the Settings > Controller Bindings... window that edits them. These are ordinary instance classes with JSON persistence, not static hubs, because they're per-user preference rather than global program behavior.

### 4.1 Where the files live

**Moved to `EmuSen.Galaxia` — see `EmuSen_Config_Reference.md` for the current answer.** All four files (`keybindings.json`, `gamepadbindings.json`, `hotkeybindings.json`, `appsettings.json`) now live in the sandbox at `/etc/EmuSen`, are read and written through `ConfigFile<T>`, and are migrated out of the old per-user location automatically on first read.

The history is worth keeping, because the reason has outlived the class: these paths were once hardcoded per class, which made them untestable — any test that rebound a key wrote the developer's real config, since rebinding saves immediately. `SettingsPaths` existed to redirect the whole set at once. That role is now `ConfigStore.OverrideDirectory`, and `AppSettings` itself moved to Galaxia (it holds no frontend types, so nothing about it was ever Mistress-specific).

### 4.2 Fixed bug: rebinding a key appeared to do nothing

The rebind flow is "click Rebind, then press a key" — the window listens for the next key and writes it into the map. It captured that key with a plain `KeyDown += ...`, which in Avalonia is the **bubbling** pass: the event reaches the focused control first and the window last.

The focused control at that moment is always the *Rebind button the user just clicked*. A focused `Button` handles `Enter` and `Space` itself as activation keys and marks them handled, so neither ever reached the window and the binding silently didn't change — the row kept saying "Press a key..." forever. Worse, the activation re-fired the button's own `Click`, re-arming the same row, so the window could never leave listening state. `Start` defaults to `Enter`, so anyone rebinding Start hit this immediately.

Fixed by capturing on the **tunnel** pass instead, which runs top-level-first:

```csharp
AddHandler(KeyDownEvent, OnPreviewKeyDown, RoutingStrategies.Tunnel, handledEventsToo: true);
```

and setting `e.Handled = true` once the key is consumed, so it never reaches the button underneath. `handledEventsToo` matters for keys something upstream has already claimed.

`MainWindow` captures gameplay input the same way and for the same reason — four of the twelve default game-button bindings are the arrow keys, which the focus manager claims for directional navigation whenever anything focusable (the menu bar) has focus. That capture had a cost of its own once a text box appeared on the same window; see §4.17. Capturing a key is not the same as claiming it, and this handler only captured; what that omission cost the game screen is §4.24.

**Regression coverage**: `EmuSen.WiseMan/Mistress/InputSettingsWindowTests.cs` drives real Avalonia key events through `Avalonia.Headless` rather than calling the handler directly — a direct call passes against the broken build, because the bug is purely in routing. Reverting the single `AddHandler` line back to `KeyDown +=` fails 3 of its 12 tests (`Enter`, `Space`, and the re-arm check). Note the arrow-key half of the problem does **not** reproduce headlessly, since headless has no real focus-navigation pass; it is reasoned from Avalonia's routing, not measured.

### 4.3 Hotkeys are bindings too (`Input/HotkeyBindingMap.cs`)

Fast-forward and rewind used to be hardcoded to `Tab` and `Backspace` inside `MainWindow.SetButtonFromKey`, undiscoverable and unchangeable. They're now entries in a `HotkeyAction` map with the same shape, persistence and rebind rules as `ControllerKeyMap`, and they appear as their own section in the settings window. Defaults keep `Tab`/`Backspace` so existing muscle memory survives, and add `F5`/`F8` save/load state, `P` pause, `F11` fullscreen, and `Escape` to step out of a game (§4.18).

`HotkeyBindingMap.IsHeld` splits the two shapes of action: fast-forward and rewind apply *while the key is down* (see `EmuSen_Rewind_And_FastForward.md` §4), everything else fires once on the press. Getting that wrong makes save-state fire once per frame while held.

The two maps police **each other** on rebind, not just themselves: one key doing both a game button and a hotkey would fire both at once. Binding a key that's in use anywhere clears it from wherever it was.

### 4.4 Gamepad options (`Input/GamepadManager.cs`, `EmuSen.Galaxia/Models/AppSettings.cs`)

- **`AnalogStickAsDpad`** (default on) — the left stick reports as the d-pad directions. No SNES game reads an analog axis, so an unmapped stick is simply dead input, which reads as a broken controller.
  **Not for a console that reads the stick as a stick** (since 2026-09-18): when the loaded ROM's core lists `LeftX`
  among its axes, `GamepadManager.LeftStickIsAnalog` is set and the left stick goes to the core as an axis instead,
  since on the N64 the stick and the d-pad are different things (`EmuSen_Input.md` §7.3). The option still governs
  every digital console.
- **`AnalogDeadzone`** (0.1, not yet exposed) — the share of an axis's travel that reads zero, so a pad at rest does not
  drift the stick a console reads. Separate from `StickDeadzone`, which is a threshold for turning a stick into
  presses, not a dead zone for an analog value.
- **`StickDeadzone`** (default 0.5, clamped 0.05-0.95) — fraction of full deflection before a direction registers. Exposed because worn sticks drift, and a drifting stick mapped onto the d-pad walks the player into walls.
- **`ControllerName`/`IsConnected`** — surfaced in the window so "no controller detected" can be told apart from "connected but bound wrong". The window re-polls once a second, so hot-plugging is visible without reopening it.

- **The rebind window lists a console's stick directions as rows** (`LS Up` … `RS Right`, short enough for the button
  column; the help text carries the full name). Each has a keyboard binding like any button. Its gamepad column names
  the stick it comes from and its pad buttons are disabled, because the direction comes from the pad's own stick and
  there is no pad button to bind it to (`EmuSen_Input.md` §7.3).

### 4.5 Conflict reporting

`Rebind` guarantees one owner per key going forward, but a hand-edited or older config can still contain duplicates. The window recomputes conflicts after every change, colours the offending rows, and names them in a status line along the bottom rather than leaving the user to work out why one key does two things.

**Clear the property, don't null it.** `MarkConflict` un-highlights a row with `label.ClearValue(TextBlock.ForegroundProperty)`, not `label.Foreground = null`. The second sets a *local* null brush, and a `TextBlock` with no brush paints nothing at all — that blanked the entire Keyboard column while every assertion-based test still passed, since `Text` was correct throughout. Caught only by rendering the window (§4.7).

### 4.6 How the window is built

**General first, then one tab per console.** This was one scrolling page, on the reasoning that keeping every row in the visual tree let conflict detection see all of them at once. That reasoning stopped applying when bindings became per console (`EmuSen_Input.md` §5.1): two consoles are *allowed* to share a key, so conflicts are computed per console from the maps rather than from what happens to be on screen, and no clash can hide behind an unselected tab because nothing consults the visual tree to find one. Hotkeys are global and live on General, and they still police every console's map.

The practical cost is that only the selected tab is realised, so a test asserting on rows has to open the window on the tab it means — `InputSettingsWindow` takes the console to select, and passes null for General.

**Rows are built in code, not XAML**, because they're one per `PadButton`/`HotkeyAction` — and now one *set* per console, so the console tabs are built entirely in `BuildConsoleTabs`; adding a core adds a tab with no XAML change. The header `Grid` and the rows `StackPanel` carry `Name` properties purely so the layout tests can still find them, since a code-built control is in no XAML name scope and `GetControl<T>(name)` cannot see it. Value labels use `TextTrimming.CharacterEllipsis` because `Key` and `GamepadButton` names (`RightBracket`, `LeftShoulder`) routinely overflow their column.

**Fixed bug: the `Gamepad` header sat over the keyboard column.** The header `Grid` in the `.axaml` and the row `Grid`s in the code-behind each declared the same column string, `90,130,Auto,Auto,140,Auto,Auto`, and a comment on each told the next reader to keep them in step. They *were* in step, and the header was still wrong by 160 px, because `Auto` sizes to content and each `Grid` sizes its own: the header has nothing in the four button columns, so they collapsed to zero there while the rows gave them the width of `Rebind Key` and `Clear`. Everything past column 1 in the header therefore rendered 160 px left of the data it labelled, putting `Gamepad` squarely over the keyboard rebind buttons.

The obvious repair — `Grid.IsSharedSizeScope` on the containing `StackPanel` and a `SharedSizeGroup` on each `Auto` column — does not work in Avalonia 12.1. The scope registers and the groups attach (both confirmed by probe), but the header's members stay 1 px wide across any number of layout passes; Avalonia's port does not equalise content-sized columns the way WPF's does. Rather than carry a framework workaround, **no column is `Auto` any more.** `ButtonRowColumns`/`HotkeyRowColumns` are the single source of truth for both the header and every row — the header takes its columns from the same constant the rows do, so the duplication that made the bug possible is gone rather than merely re-synchronised — and every width is explicit. That is no loss of flexibility: three of the seven columns were already fixed pixel widths.

They are now `ColumnDefinitions` *strings* (`"90,130,110,68,140,130,85"` and `"130,130,110,56"`) rather than hand-built `ColumnDefinition` lists, because that is the form `Ui.Cols` takes — see `EmuSen_LunaP.md` §9. **What each button-row width is holding**, in order, since the string cannot say so itself:

| Column | Width | Holds |
|---|---|---|
| 0 | 90 | button name |
| 1 | 130 | bound key |
| 2 | 110 | Rebind Key / `Press a key...` |
| 3 | 68 | Clear, plus its margin |
| 4 | 140 | bound pad button |
| 5 | 130 | Rebind Pad / `Press a button...` |
| 6 | 85 | Clear Pad, plus its margin |

`Ui.Cols` assigns columns by child position, which is exactly the order a row reads in, so `BuildButtonRow` passes its seven controls in order and sets no column explicitly. The header is the one place that does: it labels columns 0, 1 and 4, so `Gamepad` carries an explicit `.AtColumn(4)` and the other two fall where they are passed.

The fixed widths are sized to the *widest* content each column can hold, not the resting content, which fixes a second bug in the same stroke. A rebind button's text changes to `Press a key...` (102 px against `Rebind Key`'s 92) or `Press a button...` (125 px against `Rebind Pad`'s 93) while it waits for input, so under `Auto` the column grew the instant you clicked it and shoved the whole gamepad half of the row sideways — mid-rebind, which is exactly when you are looking at it. The columns are 110 and 130 px, so nothing moves.

**Pad names come from the pad.** SDL3 names the face buttons by position (`South`, `East`, `West`, `North`) rather than by an assumed Xbox label, so `PadName` asks `GetGamepadButtonLabel` what the *connected* controller actually has printed on it and falls back to the position name when SDL doesn't know (nothing connected, or a non-face button). A Nintendo-layout pad now reads `B`/`A`/`Y`/`X` where the SDL2 build always claimed `A`/`B`/`X`/`Y`. The stored binding is unaffected either way — see §4.10.

**`_initialized` guards the option handlers.** `IsCheckedChanged`/`ValueChanged` are wired in the `.axaml`, which means Avalonia raises them from *inside* `InitializeComponent()` — before the constructor has assigned `_appSettings` or anything else. Setting `Minimum`/`Maximum` on the deadzone `Slider` coerces its value and fires `ValueChanged` on the spot, so every handler returns early until the constructor finishes. Without the guard the window throws `NullReferenceException` on construction.

**`PollForPadButton` null-checks `_gamepad` itself** rather than relying on `StartListeningForPad` having done so. The invariant otherwise lives in the caller where the compiler can't see it (CS8602), and checking locally also means the 50 ms poll timer stops itself if the pad disappears mid-rebind.

### 4.7 Test coverage

`EmuSen.WiseMan/Mistress/` holds six files, all running on the shared `HeadlessUnitTestSession` from `Serenity/TestAppBuilder.cs` — this section's input-settings pair, the layout assertions of §4.6, the library coverage of §4.11, and the menu coverage of §4.12–4.13.

`InputSettingsWindowTests.cs` drives **real Avalonia key events** through `Avalonia.Headless` rather than calling the capture handler directly — the §4.2 bug is purely in event routing, so a direct call passes against the broken build. `ClickAsUser` focuses a button before raising `Click`, because that focus is exactly what used to swallow the follow-up key. Reverting the single `AddHandler` line back to `KeyDown +=` fails 3 of its tests (`Enter`, `Space`, and the re-arm check).

Two gotchas worth knowing before adding cases here:

- **Match the row by its name cell (column 0) only.** A loose "any `TextBlock` in this row" match picks the wrong row: `Y`'s default keyboard binding is literally `A`, so searching for row "A" finds `Y`'s *value* column first.
- **Redirect `ConfigStore.OverrideDirectory`** in the fixture. Rebinding saves immediately, so without it the suite overwrites the developer's real bindings.

The arrow-key half of §4.2 does **not** reproduce headlessly — headless has no real focus-navigation pass — so it is reasoned from Avalonia's routing rather than measured, and the theory cases for `Up`/`Left` pass either way.

`InputSettingsWindowRenderTests.cs` renders the window through Avalonia's real Skia pass and asserts the result isn't one flat colour, which catches an unparseable `.axaml` or a collapsed layout. Set `EMUSEN_UI_DUMP=/some/dir` to also write the capture out and look at it — that is how the blank-column bug in §4.5 was found. The variable names a *directory* and every capture in the run lands in it as `<name>.png`; it used to be a single file path, which could not serve more than one test. See `EmuSen_LunaP.md` §13.1.

### 4.8 `EmuSen.Hotaru` suppresses AVLN3001

Hotaru's `.csproj` carries `<NoWarn>$(NoWarn);AVLN3001</NoWarn>` — "XAML resource won't be reachable via runtime loader, as no public constructor was found", for `App.axaml` and `Views/GameWindow.axaml`.

Both types are constructed by hand with their real dependencies: `Program.cs` resolves a ROM and builds the core *before* Avalonia starts, then hands them over via `AppBuilder.Configure<App>(() => new App(...))` and `new GameWindow(core, ...)`. URI-based instantiation (`AvaloniaXamlLoader.Load(uri)`) is a capability this frontend never uses; `App.Initialize()` calls the *instance* overload, which needs no parameterless constructor.

Adding one is not an improvement. `GameWindow`'s constructor starts the emulation thread, the console-reader thread, a 60 Hz gamepad timer and an SDL handle, so a parameterless overload chaining into it would spawn all of that from the previewer — and one that skipped it would leave the class's `readonly` fields unassigned, trading this warning for a fistful of CS8618s and a half-built window.

Scoped to that one project deliberately. `EmuSen.Mistress` satisfies the analyzer naturally (its `MainWindow` and `InputSettingsWindow` both have real parameterless constructors) and keeps the warning live.

### 4.9 Publishing a frontend that actually starts

Two things have to be right or a published build dies on launch, and neither shows up when testing on a development machine.

**SDL3 must be shipped, not assumed.** `SDL3-CS` is bindings only — it contains no native binary. `GamepadManager` and `AudioPlayer` both call into SDL from `MainWindow`'s constructor, so without a loadable SDL3 the app throws `DllNotFoundException: Unable to load shared library 'SDL3'` before a window ever appears. The native lives in a separate package, **`SDL3-CS.Native`**, which carries `SDL3.dll`, `libSDL3.so` and `libSDL3.dylib` for x64 and arm64. Both `EmuSen.Mistress` and `EmuSen.Hotaru` reference the pair, pinned to the same version (`3.4.2`) so the bindings never call an entry point the shipped runtime lacks.

A Linux dev box usually has SDL installed system-wide, so this fault is invisible locally and appears only on a clean Windows or macOS machine.

**`-p:PublishSingleFile=true` is no longer barred by SDL.** Under `Silk.NET.SDL` it produced one tidy executable that then failed to resolve SDL2, because Silk.NET's own loader did not find the library the single-file host extracted. `SDL3-CS` resolves through the ordinary .NET `DllImport` path instead, and `SDL3-CS.Native` leaves `libSDL3.so` beside the executable rather than inside the bundle. Verified 2026-08-01 with a self-contained single-file publish of a console harness that opens an SDL3 audio device and plays through it. That test covers the SDL half only — nothing here says Avalonia and SkiaSharp survive single-file publishing, so the folder recipe below is still the supported one.

So the working recipe per RID is:

```
dotnet publish <app>/<app>.csproj -c Release -r <rid> --self-contained true \
  -p:DebugType=none -p:ErrorOnDuplicatePublishOutputFiles=false -o <out>
```

`ErrorOnDuplicatePublishOutputFiles=false` is needed because the `EmuSen.DianaOS` executable project reference gets built twice (§ its own csproj comment), emitting `EmuSen.DianaOS.runtimeconfig.json` from both the target RID and the host.

**Verifying a build launches.** Running it with no `DISPLAY` is *not* a launch test: Avalonia dies at X11 initialisation before `MainWindow`'s constructor runs, so everything SDL-related is never reached and a broken build looks fine. Launch it on a real display and check it is still alive several seconds later.

**macOS cannot be finished from Linux.** Apple Silicon refuses to execute an unsigned arm64 binary outright, so clearing quarantine is not enough; the user must ad-hoc sign (`codesign --force --deep --sign - <name>.app`). There is no signing tool on the Linux build machine.

### 4.10 The SDL3 layer (`EmuSen.Endymion`, formerly `EmuSen.Nehellania`)

Both frontends moved from `Silk.NET.SDL` (SDL2) to `SDL3-CS` on 2026-08-01. Silk.NET has no SDL3 binding and none is planned in the 2.x line, so this was a binding swap, not a version bump.

**One copy, in its own project.** `AudioPlayer`, `GamepadManager`, `GamepadBindingMap` and the config-root helper used to be duplicated per frontend — `EmuSen.Mistress` and `EmuSen.Hotaru` each carried a near-identical file, and the SDL3 port had to be written twice and kept in sync by hand. They now live once in **`EmuSen.Endymion`**, which both GUIs reference. It depends on `EmuSen.Galaxia` and on SDL3; deliberately not on Avalonia, since nothing in it touches a window — that stays split between `EmuSen.Serenity` (presentation) and the frontends themselves.

Two later moves got it there, and the section below describes the first of them. The merge landed in `EmuSen.Nehellania`; audio was then split out into `EmuSen.Endymion` (`EmuSen_Audio_Sync.md` §7.1), and on 2026-08-05 the gamepad half was folded back the other way and Nehellania deleted (`EmuSen_Multicore.md` §9.2). The reference on `EmuSen` named here went with the first of those — `PadButton` moved to Galaxia (`EmuSen_Input.md` §3).

The merge resolved three real differences between the two old copies rather than picking one at random:

- **`Pump`** took `EmulatorSession` in Mistress and `ICore` in Hotaru. The shared class takes `ICore?` and keeps a one-line `Pump(EmulatorSession)` overload forwarding to it, so both call sites are unchanged. `EmulatorSession` lives in the core library, so this costs the shared project no extra dependency.
- **`GamepadManager`** carried the rebind-capture members (`GetAnyPressedButton`, `ControllerName`, `ButtonLabel`, the stick-as-d-pad options) in Mistress only. The shared class is Mistress's superset; Hotaru simply doesn't call them.
- **`GamepadBindingMap`** resolved its config path through Mistress's redirectable `SettingsPaths` in one copy and a hardcoded `%AppData%/EmuSen` in the other. `SettingsPaths` moved into the shared project too, so both frontends went through the redirectable one — which is what lets `EmuSen.WiseMan`'s fixtures point rebind tests at a scratch directory instead of overwriting the developer's real bindings. That role has since moved again, out of the SDL3 layer entirely and into `EmuSen.Galaxia` (`ConfigStore`), where every config file in the project shares it — see `EmuSen_Config_Reference.md` §1.

`AllowUnsafeBlocks` moved with the code: `EmuSen.Endymion` sets it (for `AudioPlayer`'s pinned sample array), and both frontends dropped it, having no unsafe code of their own left.

**Subsystem init, never `SDL_Init`/`SDL_Quit`.** `AudioPlayer` takes `InitFlags.Audio`, `GamepadManager` takes `InitFlags.Gamepad`, and each calls `QuitSubSystem` for its own flag on dispose. `SDL_Quit` tears down the whole library regardless of who asked, so whichever object disposed first would break the other. `InitSubSystem` is refcounted per subsystem and does not have that problem. Neither one ever asks for `InitFlags.Video`: Avalonia owns the window, and SDL is only ever an input/audio source here. The return type flipped in SDL3 — `true` means success, where SDL2 returned `0`.

**Audio is a stream, not a queue.** SDL3 deleted `SDL_QueueAudio` outright. `AudioPlayer` now calls `OpenAudioDeviceStream(AudioDeviceDefaultPlayback, …)`, which opens the default playback device and hands back a stream bound to it, then `ResumeAudioStreamDevice` once (streams open paused), `PutAudioStreamData` per pump, and `DestroyAudioStream` on dispose — destroying the stream closes the device it opened, so there is no separate close call. `GetAudioStreamQueued` replaces `GetQueuedAudioSize` and reports bytes still in the stream, in the *input* format, which is what `QueuedFrames` divides down and hands to `DynamicRateControl` (see `EmuSen_Audio_Sync.md` §1). Like SDL2's queue depth, it excludes whatever the device has already pulled, so the rate-control target did not need re-tuning.

`AudioSpec` lost its `Samples` field; the device buffer is set through the `SDL_AUDIO_DEVICE_SAMPLE_FRAMES` hint instead, held at the same 4096 frames the SDL2 build asked for. `AudioS16Sys` is gone too — SDL3 exposes only `AudioS16LE`/`AudioS16BE`, so the constructor picks by `BitConverter.IsLittleEndian`. The device itself may well open at a different rate (44100 Hz is common) and SDL resamples 32 kHz up inside the stream; that is normal and invisible to the queue accounting above. `PutAudioStreamData` takes a raw pointer, which is the only reason either frontend still sets `AllowUnsafeBlocks`.

**Gamepads are enumerated, not indexed.** SDL2's joystick indices are gone. `GetGamepads(out count)` returns instance IDs for the gamepads specifically, so `TryOpenFirstController` no longer walks every joystick asking `IsGameController` about it — the one-second hot-plug rescan interval (§4.4) is unchanged, but the scan behind it is cheaper. `GameController*` handles became plain `IntPtr`, `GameControllerGetButton`'s `0`/`1` became a `bool`, and `GameControllerUpdate` is `UpdateGamepads`.

**Saved bindings survived the rename.** `GamepadButton` renamed the face buttons to their positions (`A`/`B`/`X`/`Y` → `South`/`East`/`West`/`North`) but kept every numeric value SDL2 used, and `gamepadbindings.json` persisted the numbers, not the names. An SDL2-era config therefore loads into the SDL3 build as the same physical buttons; `EmuSen.WiseMan/Input/GamepadBindingMapTests.cs` pins that with a verbatim copy of a file the old build wrote.

> **Since superseded, deliberately.** The file now stores names (`"B": "South"`), because `11` told a person reading it nothing. Numbers still *read*, so nothing written by an older build is orphaned and the SDL2-era file above still loads — but this does invert which kind of change is survivable, from renumbering to renaming. See `EmuSen_Config_Reference.md` §2.1 for the full trade-off.

**Testing against the dummy driver.** The audio suites force SDL's `dummy` playback driver, which is a real backend built for headless CI, not a mock. SDL3 exposes the driver choice as the `SDL_AUDIO_DRIVER` hint, so `SDL.SetHint` in a static constructor replaces the whole `SDL_AUDIODRIVER` environment-variable dance the SDL2 suites needed (a P/Invoked `setenv(3)`, because .NET's managed environment view was not what SDL read — see `EmuSen_Debugging_Tools_Reference_v5.md`). `EmuSen.WiseMan/Audio/NativeEnvironment.cs` existed only for that and was deleted.

One measurement did have to change, and the fix is worth understanding because the original was measuring the wrong thing. `AudioLatencyDriftTests` asserts that a stall's backlog drains, and it used to do that by comparing the output queue's depth before and after a three-second recovery: `QueuedFrames < postStallQueue`. Under SDL3 that assertion started flipping a coin — 22157 → 19969 on one run, 21620 → 22404 on the next.

The queue's depth is not ours alone. It is `produced − consumed`, and *consumed* belongs to the output device's clock. SDL2's dummy driver happened to consume on a tight schedule; SDL3's paces against the wall clock more loosely, and a percent of drift there is ~320 frames/s, against a control law whose entire authority is 0.5 % (~160 frames/s). The queue reading was mostly reporting the driver, and only incidentally the emulator. Lengthening the window to ten seconds made it pass again, which is the tempting fix and the wrong one — it buys signal by waiting, without ever measuring the thing under test.

What the test wants to know is whether rate control is *acting* to drain the queue, and that is observable directly: `DynamicRateControl.TotalInputFrames`/`TotalOutputFrames` (`EmuSen_Audio_Sync.md` §3.3) count frames in against frames handed on, so their difference over the recovery window is exactly the audio the control law withheld — no device clock involved. The test samples both counters at the stall and again at the end, and asserts the withheld fraction is at least half the configured maximum deviation. With the queue sitting well above target the law should be pinned near full authority, and it is: `produced=96375, emitted=95893, 0.500 % withheld`, identical on every run. The queue reading is still checked, but only as the loose bound it can actually support — that it has not climbed to the shedding entry point. The window went back to three seconds.

### 4.11 The game library (`Library/RomLibrary.cs`, `Views/MainWindow.axaml`)

`MainWindow`'s viewport is two screens sharing one `Grid`: `LibraryView` (a list of every ROM in `AppSettings.RomDirectory`) and `GameFrame` (the live emulator output). Exactly one is visible; the library is what you get whenever no game is running. Visibility is toggled in code-behind rather than bound, matching how the rest of this window is written.

Activating a title — double-click, or Enter with it selected — prompts for any missing firmware (§`EmuSen_Firmware.md` §3, same path the OS picker uses) and then calls `LoadRom`, which hides the library and shows the frame. `ShowLibrary` goes back, and is the unload path: it stops the emulation thread, flushes SRAM and verbose logs, and drops the session. Two menu items reach it — `Emulation > Close Game` (§4.12) and `File > Game Library`, which is the same call and doubles as a re-scan when nothing is running. A failed load returns there too, rather than leaving a black viewport with only a status line to explain it.

**Enter reaches the list even though it is also bound to Start.** `MainWindow`'s key handlers are registered `Tunnel` with `handledEventsToo: true` (§4.2), so they see Enter first — but `SetButtonFromKey` does not mark a bound *button* handled, so the event still bubbles to `LibraryList`. The library's own handler sets `Handled` to stop it going further. The cost is that launching with Enter also latches Start for as long as the key is held, which is harmless and arguably wanted.

The same handler is attached to the search box, so Enter starts the top match after a search too. That "also latches Start" cost is now gone in both cases: the pad map is not consulted from the library screen at all (§4.18).

**Why the scan is its own class.** `RomLibrary.Scan` has no Avalonia dependency, so the interesting behaviour is testable without a window: extension filtering (`.smc`/`.sfc`, case-insensitive), case-insensitive title sort, and the four outcomes a directory can produce — `NoDirectoryConfigured`, `DirectoryNotFound`, `Empty`, `Ok`. `RomLibrary.DescribeEmpty` is the single place the "why is my list empty" wording lives, so the inline library and the older modal `RomBrowserWindow` cannot drift; `RomBrowserWindow` was rewritten onto the same scan and no longer carries its own copy of the extension list.

The list is refreshed on construction, whenever the library is shown, and when `PreferencesWindow` closes — that window is non-modal, so the ROM directory can change while the library sits on screen behind it.

Two deliberate limits: the scan is **not recursive** (it matches what `RomBrowserWindow` always did — ROMs in subfolders are not listed), and titles are filenames with the extension stripped, with no header-name lookup or box art. Both are worth revisiting; neither is a bug.

**Test coverage.** `EmuSen.WiseMan/Mistress/RomLibraryTests.cs` covers the scan directly. `MainWindowLibraryTests.cs` drives a real `MainWindow` through `HeadlessUnitTestSession` (§4.7's harness): that a configured directory lists and sorts its games, that an unset one points at Preferences, that activating a title really switches to the game screen — it boots a `SyntheticRom` for that — that `Game Library` returns, and that a ROM added afterwards shows up on refresh.

**The headless harness needed a theme first.** `TestAppBuilder` (§4.7) built a bare `Application` with no styles. Templated controls — `ListBox`, `ListBoxItem`, `Button`, `TextBox` — then have no control template and render as *nothing*, while untemplated `TextBlock`s still draw. A render assertion counting distinct colours therefore passed on the header and hint text alone, with the entire list invisible; the library screen looked correct to the test and blank in a captured frame. `TestAppBuilder` now adds `FluentTheme` and `ThemeVariant.Dark` to match `App.axaml`, and the library's render test asserts specifically that the *selected row's accent colour* is present, which only a real templated `ListBoxItem` can produce. Set `EMUSEN_UI_DUMP=/some/dir` to write the captured frames out and look at them — a directory, one `<name>.png` per capture. See `EmuSen_LunaP.md` §13.1.

### 4.11a The lists are `LunaList<T>` now, and what measuring the contract changed

*2026-08-16. `EmuSen_LunaP_Adoption_Gameplan.md` Path A.*

Three lists kept a collection parallel to a `ListBox` of projected strings and recovered the model by indexing with `SelectedIndex`. `MainWindow`'s own comment named the shape — *"Parallel to `LibraryList`'s item strings, which are titles only"* — which is a comment explaining a hazard rather than a design. `LunaList<T>` keeps the model type, so `Selected` hands back a `RomEntry` and the parallel collection goes.

Migrated: `MainWindow.LibraryList` (deleting `_libraryEntries`), `RomBrowserWindow.RomList` (deleting `_entries`), and `CheatDatabaseWindow.SystemsList` (deleting the re-derive-and-index in `OnSystemSelected`, whose comment explained that the system name *"has to come back off"* the label because the label carries a count).

**Generics in XAML.** `x:TypeArguments` had no precedent in this repository. It works — `<luna:LunaList x:TypeArguments="lib:RomEntry" x:Name="LibraryList" />` — and it is worth preferring over building the list in code, because a control built in code is not in the XAML namescope and `GetControl<T>(name)` cannot find it. Every existing test lookup survived unchanged as a result.

**Why the existing tests did not need rewriting.** `LunaList<T>` derives from `ListBox`, and `Refresh` sets `ItemsSource` to the *projected label strings* while keeping the models in `Models` alongside. So `GetControl<ListBox>("LibraryList")`, `SelectedIndex` and `ItemsSource.Cast<string>()` all still mean what they meant. The toolkit does the parallel-array bookkeeping — the point is that it does it once, correctly, instead of three times.

**The contract was measured, and one of the measurements changed the code.** `LunaListContractTests` pins five facts, all of which are silent behaviour changes rather than compile errors if assumed wrongly:

1. `Refresh` selects **nothing** — `SelectedIndex` is `-1` afterwards. A window wanting a default must say so.
2. **`Chose` fires on a selection change, not on activation.** Setting `SelectedIndex` raises it.
3. `Select(model)` sets the selection and raises **nothing** — the application stating what is true, the same distinction `LunaAction.IsChecked` draws in §4.12.
4. `Refresh` restores the selection by `Key` across rebuilt objects, and stays quiet doing it, so a rescan cannot look like a click.
5. `Refresh` drops the selection when the key is gone — the case a narrowing filter produces.

Fact 2 is the one that mattered. `RomBrowserWindow` was first migrated with `RomList.Chose += entry => Close(entry.FullPath)`, which reads correctly and is wrong: the dialog would have closed on a single click, where it had always required a double-click or the Open button. The window is a modal returning a path, so this would have been immediately visible — and it had **no tests at all**, which is why it was not immediately visible. `RomBrowserWindowTests` exists now, and one of its tests asserts specifically that selecting a row does not complete the dialog.

The general form is worth stating because the next list will meet it: **`Chose` is not `DoubleTapped`.** A list whose activation does something irreversible — launching a game, closing a dialog — must keep an activation gesture and use `Selected` for the value.

**The library's selection reset is deliberate and was kept.** `ShowLibraryEntries` ended with `LibraryList.SelectedIndex = 0`, and `LunaList.Refresh` preserving selection would have quietly replaced that. It must not: `RefreshLibrary` wires `LibraryFilter.Submitted` to `LaunchSelectedLibraryEntry`, so typing a search and pressing Enter launches *the top match*. Preserving the previous selection instead — or dropping it, per fact 5, when the filter excludes it — would break that flow, and `Enter_in_the_search_box_starts_the_narrowed_selection` is the test that says so. The migration therefore calls `Select(shownEntries[0])` explicitly, which is behaviour-identical and now states the intent rather than implying it.

**The one site that could not be migrated.** `CheatDatabaseWindow.GamesList` still holds `_games` and indexes it. `LunaList<T>` is declared `where T : class`, and `CheatDatabaseEntry` is a `readonly struct` in `EmuSen.DianaOS/DianaOS/Var/CheatDatabase.cs`. Making it a reference type would reach `CheatCommand`, `CheatDatabasePruner` and their tests — four call sites outside this frontend — and a struct-to-class change carries allocation and equality consequences that have nothing to do with a list widget. **The parallel array stays there on purpose**, and the constraint is recorded in the XAML beside it so the next person does not rediscover it by compiler error, as this was. The plan claimed four migratable sites; there were three.

**Cost.** Measured against the *unmodified* tree on the same machine rather than against an earlier recorded number, which is what caught it: the suite reads 1m5s before the change and 1m5s after, with the thirteen new tests costing about 700ms in total. An earlier session had recorded ~53s for the same suite, and comparing against that number would have produced a twelve-second "regression" that does not exist.

### 4.12 The Emulation menu (`Views/MainWindow.axaml`)

`Emulation` used to carry a `Pause` and a `Reset` item hardcoded to `IsEnabled="False"`, with a comment saying neither was implemented. Pause in fact *was* — `MainWindow` has had `PauseEmulation`/`ResumeEmulation` since the DianaOS console window needed them (a shell command reading `Cpu`/`Bus`/`Renderer` state races `RunFrame()` on the emulation thread unless that thread is actually stopped), reachable from the console as `pause`/`resume` and from the keyboard as `HotkeyAction.TogglePause`. The menu item was simply never wired to it. The menu now carries `Pause`, `Reset`, `Close Game`, and the two state items.

**Enabling was synced on open, and is not any more.** *Retired 2026-08-16, when the menu moved onto LunaP's `LunaAction`. The original argument is kept because it was correct about the problem it was solving.* Every item in this menu needs a loaded ROM, and the states that decide that — a load, a failed load, a close, a reset, a `CPU HALT` — arrive from four different places including the emulation thread. Rather than have each of them remember to update five `IsEnabled` flags, `SubmenuOpened` recomputed all of them from `_session is { IsRomLoaded: true }` at the moment the menu was actually opened, on the reasoning that a menu that is closed has no state worth keeping correct.

What retired it is that the premise stopped holding — not that the reasoning was bad. A `LunaAction` *is* the state, and the menu item, the toolbar button and the keystroke all follow it, so there are no longer five flags to forget: `SyncMenuState` writes the actions, and it is called from `LoadRom`, `ShowLibrary`, `PauseEmulation`, `ResumeEmulation`, `SetBaseSpeed`, `SaveState` and `SelectStateSlot` — the places where the state actually changes. This is more call sites than the old design had, which is exactly the cost the old design was avoiding; what makes it the better trade now is that a missed one is visibly wrong on screen rather than invisible until a menu opens.

**The pause check mark was set in exactly one place, for a reason that no longer applies.** The old `Pause` was a `ToggleType="CheckBox"` item whose `IsChecked` only `SubmenuOpened` assigned. The hazard was real: pause has three other entry points (the `P` hotkey, the console's `pause`, the console's `resume`), and Avalonia's `MenuItem` flips `IsChecked` itself while handling a click, so a click routed through `PauseEmulation`/`ResumeEmulation` writing the property too would toggle twice and invert the mark. Syncing only on open sidestepped the ordering question rather than answering it.

`LunaAction` answers it instead, and the two halves are worth stating because they are what make the sidestep unnecessary. Setting `IsChecked` directly **does not** run the handler — that is the application stating what is true. `Invoke()` on a checkable action flips `IsChecked` **before** the handler runs — that is the user asking for a change, and the handler reads the new state. So `PauseEmulation` may now write the mark freely, and it does. A consequence worth knowing when reading the tests: `Click` is `Invoke()` alone, and a test that flips `IsChecked` *and* invokes toggles twice — the same double-toggle in a new place, found by writing exactly that helper and watching the fullscreen tests fail.

**Reset is a power cycle, not a soft reset.** `ResetEmulation` calls `LoadRom` again with the current path and display name. No `ICore.Reset()` exists to call instead, and adding one would mean deciding, per coprocessor, what a reset line does and does not clear across `Sa1`/`SuperFx`/`NecDsp` — real work with real ways to be subtly wrong, for a result the user experiences as "the game started over" either way. Reloading gets that from code that is already exercised on every single ROM load: the session is torn down, SRAM is flushed and read straight back by the new `Cartridge`, the rewind buffer and audio rate control are cleared as they are for any other discontinuity, and `SnesDebugTarget` plus any open console/`coretop` window are repointed at the new core. The one visible difference from a hardware reset button is that RAM does not survive, which no game can observe as wrong. Two smaller behaviours are deliberate: a reset always comes back *running* (`ResumeEmulation` first, so a reset issued while paused is not left frozen on a machine that no longer matches what is on screen), and the status line reads `Reset: <name>` rather than `Running: <name>`, matching how `Save State`/`Load State` acknowledge themselves — but only if the reload actually succeeded, since a failed one has already routed to the library with its own message.

**Close Game is `ShowLibrary`**, which grew three things it should have been doing all along. It now calls `StopLogging`, because that ROM's `gui_<timestamp>/` log files otherwise stay open with no next writer until the process exits or another ROM loads. It clears the rewind buffer, which was previously only cleared on the next load — several seconds of a game you have closed is a pointless thing to keep in memory. And it nulls `_debugTarget` and pushes that null into `_consoleWindow`/`_coretopWindow`, without which an open shell console would keep running commands against a `Cpu`/`Bus`/`Renderer` whose emulation thread has already been joined — exactly the staleness `UpdateTarget` was added to prevent on a ROM *swap*, which a close is a special case of. The console prints `--- ROM unloaded ---` and its `state`/debug commands go back to answering "No ROM loaded".

**Test coverage.** `EmuSen.WiseMan/Mistress/MainWindowEmulationMenuTests.cs`, in the same headless harness as §4.7. *Rewritten 2026-08-16 by changing its lookups and leaving its bodies alone* — `Item(window, "PauseMenuItem")` now returns the `LunaAction`, which carries `IsEnabled` and `IsChecked` under the same names the `MenuItem` did, so the assertions below are the ones that guarded the XAML menu rather than a restatement of the new code. The `SubmenuOpened` calls were dropped rather than kept as no-ops: not opening a menu before asserting is a stronger claim than the old tests could make. One test is new and covers what asserting on models cannot — `The_menu_bar_really_builds_its_items` opens the real `_Emulation` item, finds the `_Pause` `MenuItem` the `MenuBar` built, and requires it to go disabled when the action does, because a menu bar that rendered nothing would satisfy every other test in the file. What is covered: that all five items are disabled with no ROM and enabled with one, that clicking `Pause` twice stops and restarts the thread and moves the status line between `Paused` and `Running:`, that the check mark follows `IsPaused` across opens, that `Reset` replaces the `_session` instance and stays on the game screen, that it clears a pause, and that `Close Game` drops both `_session` and `_debugTarget`, returns to the library, disables itself, and leaves the library able to start the same game again.

### 4.13 Speed, save slots, fullscreen and the dashboard

Four more things the emulator could already do that the menu bar never exposed.

**Speed (`Emulation > Speed`).** `SpeedController` has supported fast-forward, slow motion and unthrottled since it was written (§`EmuSen_Rewind_And_FastForward.md` §2), but the only way to reach any of it was holding the Fast Forward hotkey. It could not simply be wired to a menu item, because `EmulationLoop` called `_speed.SetTurbo(_turboHeld)` at the top of *every frame* — anything a menu set would have been overwritten within 16 ms. The loop now reads `_baseSpeedPercent`, a `volatile int` the menu writes, and computes `_speed.SpeedPercent = _turboHeld ? _speed.TurboPercent : _baseSpeedPercent`. Held turbo therefore layers over whatever the menu picked and releasing the key drops back to it, rather than the two fighting over one field. `volatile` because the menu writes it on the UI thread and the loop reads it on the emulation thread; it is a single `int`, so that is the whole synchronisation story. Changing speed resets `DynamicRateControl` — the target the audio control law is chasing has just moved, and letting it converge from its old error term produces an audible swoop (§`EmuSen_Audio_Sync.md` §3.2). Note that `Unthrottled` and `Fast Forward` both fall outside `ShouldPlayAudio`'s 50–200 % band, so audio is dropped rather than played at the wrong rate; that is `SpeedController`'s existing policy, not something this menu chose.

**Save slots (`Emulation > State Slot`).** Save and Load State wrote one file per ROM, `<rom>.state`. There are now eight slots, and `_stateSlot` (1 by default) selects which one every save/load path uses — the menu items, the `F5`/`F8` hotkeys, and the DianaOS console's `state save`/`state load` default, since all of them already went through `CurrentStatePath`. **Slot 1 is deliberately still the plain `<rom>.state`**; only slots 2–8 take the `<rom>.slotN.state` suffix. That is not cosmetic — `EmuSen.Hotaru` and the shell's `state` command both write the unsuffixed name, so numbering slot 1 like the rest would have quietly orphaned every state saved before this existed and split the two frontends' idea of where a state lives.

The slot submenu's eight actions are built once and re-labelled on every `SyncMenuState`, so each entry shows that slot's file timestamp (or `empty`) as it stands right now, including for a slot saved a moment ago — which is why `SaveState` calls the sync after a successful write (posted back from the emulation thread, which does the write, since §4.21a). *Before 2026-08-16 this was a rebuild on `SubmenuOpened`, with a second build at construction because a `MenuItem` with no children renders as a leaf with no submenu arrow. `LunaAction.Submenu` is set once and the arrow follows it, so that second build is gone.* Picking a slot only *selects* it; `Save State`/`Load State` then act on it, and both re-label themselves with the slot number. Two clicks to save to a specific slot is the RetroArch model, and it keeps the hotkeys meaningful — an `F5` that could not know which slot you meant would be worse than one that follows the selection.

The eight slots and the four speeds are both `ActionGroup`s, LunaP's exclusive set. **`ActionGroup.Checked` is read-only**, despite the 0.8.0 XML documentation describing a setter ("Setting it checks that one and unchecks the rest without running any handler"). The behaviour the doc describes is real and is what this code relies on — it is simply reached by assigning `member.IsChecked = true`, which unchecks the siblings and runs no handler. Verified by reflection over the shipped assembly rather than inferred from the compiler error, because "the doc is wrong" and "the doc describes a member that was renamed" want different fixes upstream.

**Fullscreen (`View > Fullscreen`).** The `F11` hotkey's toggle. See §4.19 — the sync-on-open design concealed two defects here, and this is now `ToolWindow.IsFullScreen`.

**Hardware Dashboard (`Settings > Hardware Dashboard...`).** `CoretopWindow` — the live register/sprite/palette/hardware-load dashboard — could only be opened by typing `coretop` into the DianaOS console, which meant discovering it required already knowing it existed. The menu item calls the same `OpenCoretopWindow` the `CoretopWindowCommand` does, with the same at-most-one/bring-to-front behaviour, and is enabled on `_debugTarget is not null` rather than on the session, matching `CoretopWindowCommand`'s own "this command needs an active debug target" guard.

**Runtime Dashboard (`Settings > Runtime Dashboard...`).** `VstopWindow`, the GUI half of `vstop` (`man vstop`) — the .NET VM underneath rather than the emulated machine. Structurally it is `CoretopWindow`'s twin, down to reusing that window's `BuildMeterRow`/`ColorForPercent` shape so the two never disagree about what a bar at 70% should look like, but it differs in the one way that matters: it takes no `IDebugTarget`, so it has no no-ROM state and **the menu item is never disabled**. `OnSettingsMenuOpened` touches only `HardwareDashboardMenuItem` for exactly that reason, and a test asserts the pair — one enabled, one not — on a window with nothing loaded.

The console-side wiring mirrors `coretop`'s too. `VstopWindowCommand` (in `EmulationControlCommands.cs`) is registered through `MakeEmulationControlCommands`, and because `CreateDefault` treats a same-named extra command as a *replacement* rather than an addition, typing `vstop` in Mistress's console window opens the window instead of hitting the terminal dashboard's "needs a real interactive terminal" refusal. That refusal would otherwise be guaranteed there, since that console is a `TextBox`. It ignores `-w` for the same reason `CoretopWindowCommand` does — there is only one mode in this frontend.

**Test coverage.** In `MainWindowEmulationMenuTests.cs` alongside §4.12's: that each speed item sets the value `EmulationLoop` actually reads and that the radio marks follow it; that slot 1 writes `Playable.state` and slot 3 writes `Playable.slot3.state` without disturbing it; that loading an untouched slot says so; that the slot list reports occupancy and checks the selected one; that fullscreen toggles `WindowState` both ways and reports itself; and that the dashboard item is disabled until a ROM is loaded. `VstopWindowTests.cs` covers the runtime dashboard separately: that every section fills in, that all four meters stay in range, that Collect now clears the no-collection-yet state, that the console's `vstop` really is replaced rather than added alongside, and that the menu entry opens exactly one window and reuses it. Layout is covered separately in `InputSettingsWindowLayoutTests.cs`, which asserts §4.6's header alignment by comparing translated X positions rather than by eye — the original bug was a 160 px offset that a render test counting colours passed straight over.

### 4.14 Cheats in the GUI (`Views/CheatDatabaseWindow.axaml`, `Views/ActiveCheatsWindow.axaml`)

`Settings > Cheat Database...` shipped as an installer and nothing more: it listed how many `.cht` files sat under each system folder and stopped there. Selecting a system did nothing, so a database the user had just downloaded — tens of thousands of files — had no route into the emulator from the GUI at all. Everything below is the missing half.

**The database window is now two panes.** Systems on the left, that system's games on the right, with a filter box over the games list because a real libretro SNES folder holds several thousand entries and an unfiltered list is not a usable control. Selecting a system fills the right pane; picking a game and pressing **Load into Active Cheats** (or double-clicking it) reads that `.cht` and puts its cheats in the active list.

Two implementation notes. The systems list shows `name  (count)`, so the selected system's *name* is recovered by index against `CheatDatabase.Systems()` rather than by parsing the label back apart. And `CheatDatabase` now caches its own scan: every query on it (`Systems`, `Games`, `Find`) walks the whole tree, and the window asks twice per interaction, so one instance scans once and a new instance is constructed whenever the folder changes or a download finishes.

The filter box is wired to `TextProperty` via `PropertyChanged`, not to the `TextChanged` routed event. `TextChanged` does not fire for a `Text` set that did not come from typing, which makes the behaviour untestable headlessly and would silently do nothing if anything ever set the filter programmatically. The Active Cheats master switch is wired the same way and for the same reason.

**`Settings > Active Cheats...`** is the GUI half of `cheat list`/`enable`/`disable`/`remove`/`clear`/`master` (see `man cheat`). Each row is one cheat — a checkbox, its kind (`RAM`/`ROM`), its description, and its write formatted exactly the way `cheat list` prints it, by calling the same `CheatCommand.FormatWrite`. The checkbox writes straight through to `CheatRegistry.SetEnabled`; there is no apply step and no local copy of the list to fall out of date. Codes can be added by hand through the same best-effort format guess `cheat add` makes, so the Game Genie/Pro Action Replay distinction behaves identically in both interfaces.

**The master switch is a second axis, not a bulk edit.** `CheatRegistry.MasterEnabled` gates `ApplyAll` and `TryPatchRom` without touching any individual cheat's `Enabled` flag, so turning cheats off to check whether one is causing a bug and turning them back on returns exactly the arrangement you had. It is deliberately **runtime-only and never persisted**: `cheat save` does not write it and `cheat load` does not set it, because a set saved months ago with the switch off would otherwise silently kill every cheat in a session that had it on. It defaults to on, which is harmless — nothing is enabled by default anyway, since everything imports disabled.

**Who owns the cheat list.** `CheatRegistry` used to be constructed by `SnesDebugTarget`, which `MainWindow` rebuilds on every `LoadRom` — including the one a Reset performs. A cheat list attached to that would not survive a reset and could not be built before a game started. `MainWindow` now owns one registry for the life of the window and passes it to each `SnesDebugTarget` it constructs (the constructor parameter is optional; a caller that passes nothing still gets its own, which is what every test fixture and `EmuSen.Hotaru` does). The list therefore survives a Reset, and cheats can be loaded before any ROM is.

It is cleared when a **different** ROM loads, tracked by `_cheatsRomPath` rather than `_currentRomPath` — the latter is nulled when a game is closed, which would wrongly count reopening the same game as a change. Another game's addresses are meaningless in this one, and silently poking them every frame is worse than losing a list the user can reload in two clicks.

**Staying in sync.** Both windows are non-modal and can be open at once, so the database window takes a callback it invokes after a load; `MainWindow` points it at the Active Cheats window's `Refresh()`. It takes a second callback to *open* that window, shown as an **Active Cheats...** button beside Close — everything arrives disabled, so turning the list on is what anyone does next, and the alternative was closing this window to go back to the menu bar. The button is a callback rather than a `new ActiveCheatsWindow(...)` of its own because only one may exist: `MainWindow` owns it, and both the menu item and this button go through the same `ShowActiveCheats()`, which reuses and refocuses an open one. The button is disabled when no owner supplied the callback, which is the standalone-constructor case the designer preview uses. Loading a game **replaces** the active list rather than merging into it, unlike `cheat db load`, which merges — picking the same game twice from a list is an ordinary thing to do and must not double every cheat, whereas typing the command twice is not.

**Test coverage.** `CheatMasterSwitchTests.cs` covers the switch at the registry level (that it stops pokes landing and patches substituting, that it leaves each cheat's own flag alone, that a cheat added while it is off stays inert) and through the shell. `CheatDatabaseGamesTests.cs` covers `Games()` and the shared `CheatImport` path, including replace-versus-merge and that everything arrives disabled whatever the file's own `enable` flag said. `CheatDatabaseWindowTests.cs` gained the system → games → load flow and the Active Cheats button (that it asks its owner, that it is off without one, and that both doors reach one window), and `ActiveCheatsWindowTests.cs` covers the list, the master switch, manual code entry for both formats, remove/clear, and the external `Refresh()`.

### 4.14a The cheat list is a `LunaTable<CheatRow>`, and the last `INotifyPropertyChanged` is gone

*2026-08-16. `EmuSen_LunaP_Adoption_Gameplan.md` Path B.*

`ActiveCheatsWindow` showed its cheats in a `ListBox` whose `DataTemplate` laid out four columns by hand — a checkbox, a fixed-width kind label, an ellipsised description and a monospace detail. `CheatRow` implemented `INotifyPropertyChanged` for exactly one reason: so `IsChecked="{Binding Enabled, Mode=TwoWay}"` could write back.

**That was the only `INotifyPropertyChanged` in either frontend**, and `EmuSen_LunaP_Gameplan.md` §5 says plainly that this codebase is *"deliberately code-behind with direct control manipulation… no `INotifyPropertyChanged` layer smuggled in under 'framework'."* `LunaTable`'s checkbox column takes a projection and a writer — `new LunaColumn<CheatRow>("On", r => r.Enabled, (r, on) => r.Enabled = on, r => r.Description)` — so there is no binding, no `DataTemplate`, and nothing on the model. `CheatRow` is now a plain class whose `Enabled` setter writes through to the registry, as it always did.

**`Kind` and `Code` are template columns, not text columns.** Their muted colour, 11pt size and monospace face are what made the list scannable at a glance, and a plain text column takes the theme's body style. A template column takes a `Control` and — required, not optional — the sentence a screen reader hears in its place.

**Selection needed rewiring, because a table is not a `ListBox`.** There is no `SelectionChanged` and no `SelectedItem`; there is `Chose` and `Selected`. The Remove button follows `Chose` for a user's selection and is synced explicitly at the end of `Refresh`, because — measured, exactly as in §4.11a — **`LunaTable.Select` is deliberately silent**. Anything that selects a row in code must update what depends on it.

`Refresh` also replaced the hand-rolled selection-preservation: the window used to read the selected id, rebuild `ItemsSource`, and search the new rows for that id. `Key = r => r.Id` makes that the table's job.

**One deliberate visual change: the list has column headings now.** A table has a header row and a list does not, and there is no way to adopt the one without the other. It is an improvement on balance — the four columns were previously unlabelled — but it is a change, and it is recorded here rather than left to be noticed, in the same spirit as Phase 7's "every settings toggle became a switch". **Sorting is deliberately *not* enabled**: `LunaTable` can sort, remember column widths and remember a sort order, and turning any of that on is a separate decision from replacing the list. `TableKey` is unset, so nothing is remembered yet.

**Test coverage kept its bodies.** All 42 cheat-window tests survive with two lookups changed — `Rows(w)` reads `Models` instead of casting `ItemsSource`, and selecting a row goes through the table's own inner `ListBox` rather than `SelectedIndex`. That last one is not incidental: `Select` being silent means a test that used it would assert against a Remove button the window was never told to update, so the test drives the selection a click actually makes.

**A LunaP defect this surfaced, fixed upstream and bridged here.** `AccessibilityTests` requires every `ListBox` in a window to have a name, and it went red immediately: the caller names the `LunaTable`, but the table's template puts an unnamed `ListBox` inside it, so a screen reader met an anonymous list within a named table. That is fixed in LunaP (`LunaP.md` §78.4 — the table now forwards its name, re-applying it on change because a window built in a constructor is named after its template is applied). **The fix is not in 0.8.0**, so `BuildCheatColumns` names `PART_Rows` itself on `TemplateApplied`, with a comment saying which version retires it. Delete that when the package moves.

### 4.15 Applying and saving cheats (Views/ActiveCheatsWindow.axaml, Var/CheatRegistry.cs)

Reported after §4.14 shipped: ticking cheats on in the Active Cheats window did nothing in game. The symptom was real but the cause was not the one it looked like — the write-through in §4.14 works, and ticking a box does reach `CheatRegistry.SetEnabled`. What actually happened was earlier and worse.

**The bug: `CheatRegistry` was not safe to touch from another thread.** Its cheats lived in a `List<Cheat>`, and `ApplyAll` walked that list with a `foreach` — from the emulation thread, once per frame. Every GUI mutation ran on the UI thread. Loading a game from the cheat database is a `Clear()` followed by several hundred `AddCheat` calls (the real *A Link to the Past* file holds 437), so it structurally modified the list several hundred times while the emulation thread was enumerating it. `List<T>` detects exactly that and throws `InvalidOperationException: Collection was modified`.

That exception surfaced on the emulation thread, inside the `try` around `session.RunFrame()` in `MainWindow.RunEmulationLoop`. That handler is deliberately fatal — it sets `_running = false`, posts `[CPU HALT] <message>` to the status bar and breaks out of the loop, matching the console build's "halt and print on a core exception" behaviour rather than trying to recover. So loading cheats into a running game **stopped emulation**, and every checkbox ticked afterwards was ticked against a dead machine. Hence "nothing happens."

This was a threading-model violation, not an accident of one window. `DianaOSInterpreterScheduler` exists precisely because Mistress's console window used to submit mutating commands straight from the UI thread with no synchronisation against its own emulation thread; mutating commands are queued there and drained once per frame *by the thread that owns the core*. The §4.14 windows reintroduced the same mistake by a different door.

**The fix: copy-on-write inside the registry.** `_cheats` is now a `Cheat[]` published with `Volatile.Write` under a private lock and read with `Volatile.Read` by everything else. Readers take one snapshot and walk it to the end; a mutation arriving mid-walk builds a new array and swaps the reference, so the walk in progress finishes against the list it started with and the next one sees the new state. Mutations are rare and small, so the array copy costs nothing that matters; readers are on the hot path and now pay one volatile field read instead of a `List<T>` field read.

This was chosen over routing GUI edits through a scheduler queue because the registry is a small data structure whose readers only ever need a stable snapshot, and because a queue would have made every window action asynchronous — the Active Cheats window could no longer refresh itself synchronously after an edit. It also fixes the same race for any future frontend without each of them having to know about it. `SetEnabled` is the one mutation that writes in place rather than publishing a new array: a reader seeing the old or the new flag is equally correct and a `bool` write cannot tear, so a snapshot swap would buy nothing.

**Apply Cheats.** Ticking a box already takes effect on the next frame, so this button is not an arm/commit step, and it would have been a placebo if it were. It does two things that are not otherwise reachable. It forces one immediate application through the new `SnesDebugTarget.ApplyCheats()` — extracted from the body of `OnFrame`, which now just calls it — which matters while paused, because a paused game has no next frame coming. And it reports how many cheats it applied, which is the confirmation the user was actually looking for when they asked for the button.

It turns the master switch back on if it is off, because "Applied 12 cheat(s)" is a lie for as long as the switch would swallow all twelve. With no game running it says the cheats are armed rather than claiming to have poked anything.

Cheats must be poked from the thread that owns the core, so the button never does it directly. `MainWindow.RequestCheatApply` sets `_applyCheatsPending`, which the emulation loop clears next to its existing `DrainPendingFromEmulationThread()` call — the same rule, the same place. The exception is while paused: the emulation thread is then parked in `_pauseSignal.Wait()` rather than inside the core, so nothing is racing and the apply happens immediately, which is the case the button exists for.

**Save Cheat List, and why the list comes back by itself.** The request was to stop reloading a game's cheats from the database every session. Saving alone would not have done that — it would have moved the chore from "find it in the database" to "remember to load it". So the button writes the list and `LoadRom` reads it back without being asked.

Both ends go through the existing per-game store `cheat save`/`cheat load` already use (`ConfigStore`'s `cheats/<name>.json`, see EmuSen_Config_Reference.md §3.4), so a list saved from the GUI is loadable from the shell and vice versa. The name is the ROM's file name without its extension. `CheatFile.IsValidName` deliberately *validates* rather than sanitises, because the name `cheat save` takes is typed by a user and silently rewriting it would save to a file they did not name; `MainWindow.CheatListName` sanitises instead, because that name is derived from a path rather than typed, and refusing to save because a ROM's file name contains a character Windows dislikes would be useless behaviour.

Restoring happens inside `DropCheatsFromAnotherGame`, which already knew whether this load is a new game, and only when the active list is **empty**. Both guards matter: a Reset and a close-and-reopen both run `LoadRom` against the same ROM, and either one silently replacing the list with the last saved copy would throw away everything edited since. A saved list keeps each cheat's own enabled state, since the arrangement is most of what is worth saving.

**Test coverage.** `CheatRegistryThreadSafetyTests.cs` reproduces the original failure — a thread applying frames while another clears and refills the list 100 times — and asserts a snapshot already being walked runs to its end. `ActiveCheatsApplyAndSaveTests.cs` covers Apply against a running core, with no game, with nothing ticked and with the master switch off, and Save's file contents, its preservation of enabled state, and its no-game message. `MainWindowCheatListTests.cs` gained the round trip that matters: save under one window, and a new window starting that ROM has the cheats back — but not for a different game, and not over a list already in hand.

### 4.15a The Active Cheats window's console tabs and file buttons

The window is tabbed like the rebind window (§4.6): **General first, then one tab per console oldest-first**, appended by `BuildConsoleTabs` from `CoreCatalog.ConsolesInReleaseOrder`, so a third core adds a tab with no XAML.

**The cheat list is not per tab.** There is one live `CheatRegistry` — the one the running game actually reads — so the list, the master switch and Apply/Remove/Remove All sit *outside* the TabControl and stay on screen whatever tab is selected. Giving each tab its own list would have meant the window no longer showed what was applied, which is the one thing it exists to show.

What *is* per console is the only genuinely console-specific operation: **parsing a typed code**. Each console tab carries its own Add box, and the tab a code is typed under decides which codec reads it — the same eight characters are a Pro Action Replay poke under SNES and nothing at all under NES. Each tab names what it accepts (`Accepts: Game Genie, Pro Action Replay.`) from the codec objects rather than a hardcoded string, so a console whose core has no decoder says so and disables its button instead of offering a dead box. That branch is currently unreachable — both shipped cores have codecs — and is kept for the core that does not.

`OnAddClick` resolves the tab from `sender` rather than from the selection, so the button always parses under the tab it is drawn on even if selection changed underneath it.

**Two names, one console.** Tab headers use `CoreDescriptor.Console` (`"NES"`), but `CoreFactory.CheatCodecsFor` resolves a `DisplayName` (`"NES (Moon)"`), and callers hand this window whichever they happen to hold — `MainWindow` passes `AppSettings.SelectedCore`, a display name, while `ICore.CoreName` is a console. Both the tab match and the codec lookup accept either now; see `EmuSen_Multicore.md` §12 for why that mattered more than it looks.

**Save, Load, Save As, Load From.** Saving stays **per game**, not per console: `Save` writes `CheatFile.For(<game>)`, the file `LoadRom` already looks for by itself, so a Game Genie code written for SMB1 comes back when SMB1 starts and never when anything else does. `Load` is its counterpart and **replaces** the live list rather than merging into it, so what is on screen is what the file says. `Save As...` and `Load From...` use a real file picker and `CheatFile.SaveTo`/`LoadFrom`, writing the identical JSON shape to any path — those copies are never loaded automatically, which is the whole point of them.

`Load From...` is not something that was asked for; a `Save As...` with no way to read the file back would have been a one-way door.

Button states say what is possible rather than failing on click: `Save` needs a running game to name the file after, `Load` needs that file to exist, and `Save As` only needs something in the list.

### 4.16 Pruning the cheat database (Var/CheatDatabasePruner.cs, Cores/CoreCatalog.cs)

`cheat db update` downloads the whole libretro cheat database: 44 systems, 28,301 files, 250MB. This build implements one core, so 42 of those systems are dead weight — and not merely idle weight, because `CheatDatabase.Scan()` walks the entire tree and a new instance is constructed every time the folder changes or a download finishes. Measured against a real download, pruning to the SNES family leaves 2,779 files: **a 94% reduction**.

**The agnostic part is where the mapping lives, not how the pruner is written.** Something has to know that this core corresponds to the folder `Nintendo - Super Nintendo Entertainment System`, and the only question is whose knowledge that is. Putting it in the pruner would mean a second core requires editing the pruner, which is exactly the coupling to avoid.

It lives on `CoreDescriptor` instead, which already existed as the frontend-injected, deliberately core-agnostic record of "which console, which extensions" that `CoreCommand` validates against. It gains `CheatSystems`, the libretro folder names that core can use. `CheatDatabasePruner` is then handed a set of names to keep and has no idea what a core is; `CoreDescriptor.SupportedCheatSystems` derives that set from a registry, deduplicating because one core is registered under several aliases (`venus` and `snes` are the same entry). **Adding a second core is one registry entry, not a pruner change** — and `CheatDatabasePrunerTests` asserts the shipped catalog claims what it should, so filling in a second core's folders shows up as a test result rather than a silently wrong prune.

**`Cores/CoreCatalog.cs` is new, and it removed a duplicate.** The registry of implemented cores previously lived as a private dictionary in `EmuSen.Hotaru/Program.cs`; the pruner would have been its third reader, and three private copies of "which consoles exist" is how they drift apart. It sits in the `EmuSen` assembly rather than in `EmuSen.DianaOS` because DianaOS stays agnostic about which concrete cores exist (the same reasoning behind `IDebugTarget` and `ICheatCodeCodec`) while that assembly already knows `VenusCore` by name. Hotaru's dictionary now forwards to it.

Venus claims **two** folders: the main SNES one and `Nintendo - Satellaview`, which libretro files separately but which is the same 65816 cartridge hardware. That is a claim about the core, so it is declared next to the core.

**Two refusals, not one confirmation.** The pruner deletes directories of files the user downloaded, and the way back is another 250MB fetch, so `Plan()` returns a refusal with a reason rather than a deletion in the two cases where an empty or unmatched keep-set would wipe everything:

- **Nothing claimed.** A registry whose cores declare no cheat systems is a build that has not filled `CheatSystems` in, not a build that wants every cheat gone. A frontend that never injects the callback at all therefore prunes nothing rather than everything — the null default is the safe one.
- **Nothing matches.** If no system on disk matches anything claimed, that is a wrong folder or a wrong mapping. It is also the one situation where "delete all 44 systems" looks exactly like correct behaviour, which is precisely why it has to be refused instead.

`Apply()` additionally resolves every path and refuses anything that is a symlink or that does not sit directly under the cheat directory, so neither a crafted folder name nor a symlinked system folder can delete outside the tree the plan was measured against. A folder that will not delete is reported, not thrown — one locked directory must not abort the other forty-one.

**Confirmation in both interfaces.** `cheat db prune` lists what would go and does nothing; `--apply` is what deletes. The Mistress button is two-stage for the same reason: the first click measures and re-labels itself `Delete N system(s)?`, the second carries it out, and changing the cheat folder disarms whatever was armed, since the plan was measured against the old one. A finished download arms the offer automatically, because the moment the 250MB lands is the moment the choice is worth putting in front of someone.

**Test coverage.** `CheatDatabasePrunerTests.cs` covers the plan, the deletion, both refusals, case-insensitive folder matching, the symlink escape, and the registry derivation including a core that claims nothing. `CheatPruneCommandTests.cs` covers the shell: report-without-`--apply`, delete-with-it, the already-pruned message, and that a shell built without the injection prunes nothing. `CheatDatabasePruneWindowTests.cs` covers the two-stage button, that one click deletes nothing, that the systems pane updates after, and that the button is off when nothing supplies a supported set.

### 4.17 Fixed bug: the library search box ate keystrokes

Typing in the library screen's search box (§4.11) dropped characters and ignored Backspace. The cause is §4.2's fix meeting the search box: `MainWindow` captures gameplay input on the **tunnel** pass with `handledEventsToo: true`, so its handler runs before the focused control sees anything, and `SetButtonFromKey` marks a key handled whenever it resolves to a `HotkeyAction`.

With the default hotkeys — `Tab`, `Backspace`, `F5`, `F8`, `P`, `F11` (§4.3) — that made three things go wrong in a focused text box:

- **`Backspace` never deleted.** `TextBox.OnKeyDown` runs from a class handler registered without `handledEventsToo`, so a key already marked handled never reaches it.
- **`p` never typed.** This one is a platform contract rather than a routing detail: `X11Window.DispatchInput` raises the key event, checks `RawInputEventArgs.Handled`, and **returns before constructing the `RawTextInputEventArgs`**. A handled `KeyDown` therefore suppresses the character outright. Win32 does the equivalent by only calling `TranslateMessage` on an unhandled key. So exactly one printable character was unusable, which reads as "the search box is flaky" rather than as a bound hotkey.
- **`Tab` could not leave the field**, and the game-button half of the map was still latching pad presses (`z` pressing B and so on) while the user typed a title.

**The fix is that a focused text field owns the whole keyboard.** `SetButtonFromKey` returns immediately when the event's source sits inside a `TextBox`:

```csharp
private static bool TypingIntoATextField(RoutedEventArgs e) =>
    e.Source is Visual source && source.FindAncestorOfType<TextBox>(includeSelf: true) is not null;
```

It tests `e.Source` rather than asking the `FocusManager` because the tunnelling event is already being routed *to* the focused element — the answer is in the arguments, and no second source of truth can disagree with it. `includeSelf` matters: focus lands on the `TextBox` itself, not on the `TextPresenter` in its template.

Suppressing *hotkeys* too, not just pad buttons, is deliberate rather than lazy. The alternative is a rule like "F-keys still fire, letters don't", which has an arbitrary boundary and breaks the moment someone rebinds pause to `F5`. Nothing is lost by the blunt rule, because **the search box only exists on the library screen, and no game is running there** — there is no pad to press and no state to save. Text entry elsewhere in the app lives in separate windows, which never had this handler.

**Enter now starts a title from the search box as well.** The library's hint has always read "Double-click a title, or press Enter, to start it", but `OnLibraryKeyDown` was wired only to the `ListBox`; from the search box, Enter hit the tunnel handler, latched Start (§4.11) and did nothing visible. The same handler is now attached to both controls, so a search narrows the list and Enter starts the top match. This is an addition rather than part of the reported bug.

**Test coverage** is in `MainWindowLibraryTests.cs`, and each case fails against the un-fixed build: the hotkey-letter, the Backspace and the focused/unfocused pair fail without the guard, and the Enter case fails without the XAML wiring. The unfocused case is the one that matters for regressions — it presses `z` and `Tab` with the list focused and asserts the pad and fast-forward *do* still respond, so a future over-broad guard cannot quietly disable gameplay input.

**The harness had to be extended to see the character half.** Headless raises key and text input independently — `KeyPress` never produces text at all, whatever the key symbol — so a naive test passes against the broken build for the same reason §4.2's did. `MainWindowLibraryTests.Type` models `DispatchInput`'s contract instead: it raises the `KeyPress`, observes the final `Handled` state through a bubbling probe registered with `handledEventsToo`, and only then raises `KeyTextInput`. `Press` is the raw form, for keys that carry no character.

### 4.18 Stepping out of a game (`HotkeyAction.ExitToLibrary`)

`Escape` leaves a running game for the library screen and **pauses the emulation thread on the way out**; pressing it again returns to the game and resumes it. The game is not unloaded, so this is a different operation from `ShowLibrary` (§4.11), which stays the unload path behind `File ▸ Close Game` and behind a failed load.

It is a `HotkeyAction` rather than a hardcoded key, for the reason §4.3 gives: hardcoding `Tab`/`Backspace` inside `SetButtonFromKey` made them undiscoverable and unchangeable, and repeating that for `Escape` would repeat the mistake. Being an action, it appears in the settings window and rebinds like any other.

**Why pausing matters.** The library screen is a full-window view: switching to it hides `GameFrame` but does nothing to the emulation thread, which would otherwise keep advancing frames — burning CPU, running audio and consuming input for a game nobody is looking at. `ToggleLibrary` pauses *before* swapping the two views, so no frame ever runs unwatched.

**The library screen no longer feeds the pad.** With a game suspended rather than closed, `_session` is live while the user is arrowing through a list and typing in the search box, and every one of those keys is also a game binding — the arrows are the D-pad, `Enter` is Start, `z` is B. `SetButtonFromKey` therefore skips the pad map entirely whenever `LibraryView` is visible, and `TogglePause` is ignored there too, since resuming a game that is off-screen is not something any key on the library screen should be able to do. This is the same principle as §4.17's guard, one level up: the library screen is not the game.

A consequence worth noting against §4.11: launching with `Enter` no longer latches Start for the duration of the keypress, because the pad map is not consulted from that screen at all.

**The suspended game has to stay findable.** Nothing on the library screen otherwise shows that a game is still loaded behind it, so `ShowLibraryEntries` swaps the hint line for `Press <key> to return to <game>` and forces it visible even when the list is empty — a suspended game must not become unreachable just because the ROM directory scan came back with nothing. The key name is read from the binding rather than written as "Escape", so a rebind cannot make the hint lie.

**Adding an action to `HotkeyBindingMap` needed a migration.** `Load()` replaced its whole dictionary with the saved file's, so any config written before `ExitToLibrary` existed would have left it silently unbound — the feature would simply not work for every existing user, which is exactly the class of bug that never shows up in development. `Load()` now backfills defaults for actions the file has no entry for, and **skips a default whose key is already spoken for**, so backfilling can never steal a key the user has deliberately bound elsewhere. `InputSettingsWindowTests` pins both halves: an older file gains `Escape` while keeping its own `TogglePause` preference, and a file that has since bound `Escape` to save-state keeps it, leaving the new action unbound rather than creating a conflict.

**Test coverage** is in `MainWindowLibraryTests.cs`: that `Escape` suspends rather than unloads (checked by the status line, which reads `Paused: <name>` where `ShowLibrary` would read `No ROM loaded`), that a second `Escape` comes back running, that arrowing/`z`/`P` on the library screen reach neither the pad nor the pause signal, and that `Escape` with no game loaded does nothing at all.

### 4.19 Two fullscreen defects the sync-on-open menu was hiding

`MainWindow` is a `LunaP.Windowing.ToolWindow` as of 2026-08-16, and full screen is `IsFullScreen`/`ToggleFullScreen`/`FullScreenChanged` rather than assignments to `WindowState`. The migration was undertaken for the menu (§4.12); these two defects were found while writing tests for it, and both had been shipping.

**The tick could lie, and the old design guaranteed it.** §4.13 recorded the check mark as syncing "from the live `WindowState` on open, so a fullscreen entered by hotkey shows correctly in the menu" — which is true of the hotkey and false of everything else. A window manager's own full-screen affordance (on this desktop, `Super+Up`) moves the window without going through `OnFullscreenClick`, so between that keystroke and the next time somebody opened the View menu, the tick said the opposite of the window. Syncing on open cannot fix this, because the stale interval is precisely the interval when the menu is closed. `FullScreenChanged` is the seam that does: `IsFullScreen` is read from the window rather than stored beside it, so the event fires whoever moved it.

**Leaving full screen always returned to `Normal`.** `WindowState = WindowState == FullScreen ? Normal : FullScreen` throws away what the window was before. Go full screen from a *maximized* window and coming back out leaves it un-maximized — a small thing, and the kind of small thing nobody files. LunaP restores the state it came from.

**Both were demonstrated before they were fixed**, per the project's standing rule. Two tests were written against the unmodified build and run there first: `The_fullscreen_tick_follows_a_change_this_window_did_not_make` (set `WindowState = FullScreen` directly, as a window manager does, and read the tick without opening a menu) and `Leaving_fullscreen_returns_a_maximized_window_to_maximized`. Both failed — `Assert.True() Failure` and `Assert.Equal() Failure: Values differ` — against the 16 that then passed. Both pass after the migration.

**What this does not cover.** Full screen is deliberately *not* remembered across a restart, which is LunaP's choice and is left alone here: a window that reopens maximized still has a title bar and a close button, and one that reopens full screen has neither, with the key that would let it out bound to whatever `HotkeyAction.ToggleFullscreen` currently is. `MainWindow` also sets no `WindowKey`, so its geometry is still not remembered at all — `ToolWindow` remembers nothing without one, which is what makes this migration behaviour-preserving in that respect. Turning it on is a separate decision, not a side effect of moving the base class.

**Menu shortcuts were deliberately not added — and the reason given was half wrong.** *Corrected 2026-08-16.* The concern was right: Mistress's keys come from `HotkeyBindingMap` and are user-rebindable (§4.3), and two independent bindings for one command is how a menu ends up advertising a key a rebind has since moved. The **factual claim was wrong**: `LunaAction.Shortcut` does not bind anything by itself. `MenuBar.SetMenus` explicitly does not bind — its own documentation says so and points at `Menus.BindShortcuts`, which only `AppWindow` calls. What a menu item does with `Shortcut` is set Avalonia's `MenuItem.InputGesture`, which *draws* "F5" and binds nothing.

Since Mistress builds its menu with a bare `MenuBar` and never calls `BindShortcuts`, setting `Shortcut` is display-only and carries none of the risk this paragraph attributed to it. The work is done, and it is smaller than "a separate piece of work" implied: `ShowHotkeysOnTheMenu` reads `HotkeyBindingMap.ActionToKey` and is called from `SyncMenuState`, so a rebind moves the label with it, and `InputSettingsWindow`'s `Closed` triggers a sync. `HelpText` carries the action's display name for a screen reader.

Two tests hold the pair apart: one asserts the gestures shown match the bindings, and one asserts `window.KeyBindings` is **empty** — which is the claim that matters, because it is what makes `HotkeyBindingMap` still the only thing dispatching those keys.

### 4.20 The pointer gets out of the way, and a ROM can be dropped on the window

*2026-08-16. `EmuSen_LunaP_Adoption_Gameplan.md` Path C — the one path in that plan that deletes nothing and says so.*

Neither frontend had any cursor management (`grep -rn "Cursor"` over both returned nothing) and neither accepted a dropped file (no `AllowDrop`, no `DragOver`, no `DataFormats` anywhere). Both can go full screen. A mouse pointer parked over an emulated game was the ordinary result.

**`IdleCursor` attaches to a control, and in Mistress that distinction is the whole point.** It is attached to `GameFrame`, not to the window, so the pointer disappears over the video and stays visible over the menu bar and the status line. A window-level flag cannot express that. Hotaru attaches it to the window, because in Hotaru the window *is* the frame.

**`FileDrop` differs between the two frontends for a threading reason.** Mistress's handler runs `OpenDroppedRomAsync`, which is the same sequence `Open ROM...` runs — the firmware prompt (§`EmuSen_Firmware.md` §3) and then `LoadRom` — so a dropped ROM cannot skip a prompt the picker would have shown. Hotaru's cannot do that: its `SwapCore` runs on the **emulation thread**, drained from `ProcessHotkeys`, and calling it from a UI-thread drop handler would race `RunFrame`. The drop therefore sets a `volatile string?` that `ProcessHotkeys` consumes on its own next tick — the same shape as every `_request*` flag beside it.

Both frontends refuse a multi-file drag (`Accept = paths => paths.Count == 1`), before the drag indicator has promised anything: a folder of ROMs has no single answer, and picking the first would be a guess.

**Disposal is the risk, and is what the tests are for.** Both types are `IDisposable`, and a cursor left hidden by an object nobody disposed is an application whose pointer is gone for good. `PointerAndDropTests.Closing_the_main_window_gives_the_pointer_back` hides the cursor, closes the window, and requires it back. **Sabotage:** removing `_idleCursor?.Dispose()` from `MainWindow`'s `Closing` handler turns that test red, and nothing else in the suite notices — which is the point of having it.

**What this does not do.** No idle delay was tuned; both use `IdleCursor`'s default. Nothing hides the pointer on a keystroke, deliberately — only pointer movement counts as activity, or an application somebody is holding four keys down in would never hide it at all. And neither frontend hides the pointer when *entering* full screen; it waits for the idle period like any other time.

### 4.21 The emulation thread, and the pause signal a console command needs

*2026-08-16. Moved here from `Views/MainWindow.axaml.cs`, which carried it as inline comment blocks of up to 32 lines. Nothing about the code changed; this is the prose finding its documented home.*

**Emulation used to run inside a `DispatcherTimer` tick.** `RunFrame()` and the frame-buffer readout after it therefore blocked Avalonia's input and paint pump every single frame. That is the actual reason Testing Studio ran markedly slower than the console build, which never had a UI pump to block in the first place — it does not have one. `_emuThread` (`StartEmulationThread`/`EmulationLoop`/`StopEmulationThread`) is that fix. The timer survives, carrying gamepad polling only (§4.10), because polling SDL is a UI/init-thread concern and always was.

Only the final present needs the UI thread. `GameFrameControl` is an ordinary Avalonia `Control`, and `Control`/`Visual` are UI-thread-affine exactly as the `WriteableBitmap` this replaced was, so `PresentPendingFrame` is dispatched and everything else stays on `_emuThread`.

**`_pauseSignal` exists for the console window, not for the player.** It is a `ManualResetEventSlim` where signalled means running. Mistress's console window runs on the UI thread *concurrently* with the emulation thread, so a shell command reading or writing `Cpu`/`Bus`/`Renderer` state has nothing else to stop the race with — see `man pause`. Hotaru needs no equivalent because its prompt runs **on** the emulation thread and the two cannot overlap by construction (`EmuSen_Frontend_Driver.md` §1 step 7).

Three details of that mechanism are load-bearing and none of them are obvious from the type:

- **`StopEmulationThread` always `Set`s before it `Join`s.** A thread parked in `_pauseSignal.Wait()` never observes `_running = false`, so stopping while paused would deadlock the UI thread forever. The `Set` in a *stop* path exists for that and nothing else.
- **The loop checks `IsSet` before calling `Wait()`.** Both are correct; always calling `Wait()` costs a syscall every frame even when nobody has ever paused. The check keeps the overwhelmingly common case a plain read.
- **`nextTick` is reset to now on resume.** Without it, the whole paused duration is attributed to one artificially slow frame: the "fell behind" branch at the bottom of the loop fires, and the fps window reports a stall that never happened.

`StopEmulationThread` is synchronous by contract, because `LoadRom` and the `Closing` handler both go on to replace or dispose `_session` immediately afterwards. It is safe to call from the UI thread: the loop checks `_running` at least once per paced frame interval and has no blocking wait of its own to get stuck in, so the `Join` is bounded in practice without being bounded in code. Hotaru's is not, and bounds it explicitly — `EmuSen_Frontend_Driver.md` §5 is why the two differ.

**The input race is accepted, and was not introduced by moving emulation off the UI thread.** Button state is written straight into `_session.Bus.Input` from the UI thread (`SetButtonFromKey`, `PollGamepad`'s `ApplyButtonState`) while this thread calls `RunFrame()`, which reads it. Every field involved is a single `ushort`/`bool` with exactly one writer, so the worst case is a button observed one frame later than it otherwise would be — not a torn read and not a crash. A frame-latched input model already has that latency inherently. It is the same accepted single-writer/single-reader race as Hotaru's `_request*` hotkey flags (`EmuSen_Frontend_Driver.md` §2).

**The fps readout measures emulation, not presentation, and says so on purpose.** It counts completed `RunFrame()` calls rather than presented frames, because the coalescing hand-off (`Latest<T>`, `EmuSen_Serenity.md` §4) legitimately drops presented frames without emulation being behind — a presented-frame counter would report a problem that is not one. The number is split into `RunFrame()` time against the rest of the loop's own per-frame work (`GetFrameBufferRgba`'s copy plus the hand-off) because "boot and title screens hold 60 fps and gameplay does not" has two candidate causes that look identical in a single figure: `RunFrame()` itself getting more expensive under a heavier scene, which is shared core code and would cost Hotaru exactly the same, or this loop's own wrapper overhead scaling with scene complexity, which is frontend-specific. Reporting both is how to tell which. Phase names come from `IFrameProfiler` (`EmuSen_Multicore.md` §5); a phase whose name contains `/` sits *inside* its parent, so it is printed in a second group rather than in a list that invites adding the numbers up.

**`SleepUntil` spins the entire remaining interval, and a hybrid was tried first.** The obvious design — sleep most of the remaining time, busy-spin only the last ~2 ms — was implemented and was not enough. Measured CPU usage while the frontend sat stuck at ~56 fps was **~5%**, which says the process is nowhere near compute-bound: the shortfall was oversleeping, not slow work. `Thread.Sleep`'s wakeup latency in this environment is simply larger than the 2 ms margin the hybrid budgeted for it. Rather than guess at a bigger margin and re-measure, the sleep was removed altogether. With that much headroom, pegging one core for the ~14 ms a frame spins is the trade real-time audio and emulation loops routinely make, at the cost of that core's power draw. Hotaru's `SleepUntil` is the same function for the same measured reason and does not repeat it.

**What this does not cover.** The `catch` around `RunFrame()` is deliberately fatal — `_running = false`, `[CPU HALT] <message>` posted to the status bar through the UI thread, break — matching the console build's halt-and-print behaviour rather than attempting recovery. It does **not** flush SRAM; the core's own periodic autosave inside `RunFrame()` is what covers a crash, and that is unchanged from before this thread existed. Nothing here throttles the *presenter* — dropping stale frames is `Latest<T>`'s job, not this loop's — and nothing here is a general job queue: ~~`DrainPendingFromEmulationThread` and `_applyCheatsPending` are the only two things the UI thread hands over, and both exist for the same reason `pause` does (§4.15).~~ *Three since 2026-09-19: the state hotkeys hand over their save and load as well (§4.21a), after a torn state showed that they had to. The console's queue and the cheat flag exist for the reason `pause` does (§4.15); the third exists because a save is not a read.*

*Addendum 2026-09-21: the readout now also shows presentation, after a bar: frames offered to the frame control and
frames it drew, per second, with the render thread's copy and draw time a frame, the backend, and the frame's size
(`EmuSen_Serenity.md` §2.5). The first figure is still `RunFrame`'s count, for the reason above; the presentation
figures sit beside it, labelled as what they are, so the dropped frames the hand-off makes on purpose read as the
difference between two named rates rather than as a slow emulator. The same line goes to the session's
`general.log` once a second, prefixed `[fps]`.*


**The loop's work outside the core, split (2026-09-23).** The `[fps]` line now ends with `| outside: requests R
audio A hand-off H sleep+rest S ms`, each a mean over the second's frames: *requests* is the debugger's refresh, the
console's queue, the cheats flag and the core requests that follow `RunFrame`; *audio* is draining the core's samples
and submitting them to the player; *hand-off* is taking the picture and offering it to the render thread
(`GetFrameBufferRgba`, `SubmitFrame`); *sleep+rest* is everything from there to the next `RunFrame`, which is the
pacer's sleep when the loop is ahead and the pacer's own work and the line's printing when it is not. `run` plus the
four is `total`. It was added when a handheld's `total` stood ~4.7 ms above `run` on Donkey Kong 64's title at 2×
after the frame lending (`Mars_Native.md` §6.13) had removed the collections, so that the remainder could be read
instead of argued.
### 4.21a Save and load states run on the emulation thread

*2026-09-19.* Until this date `SaveState()` and `LoadState()` — the `F5`/`F8` hotkeys and both Emulation menu items — called `_session.SaveState`/`LoadState` on the UI thread while `EmulationLoop` ran `RunFrame()` on its own. Nothing ordered the two. They now hand the work to the emulation thread, as the console and Hotaru already did.

**How it was found.** Through Mars, and not by looking here. A Wave Race 64 state saved with `F5` during play loaded into a machine whose program counter came from one instant and whose next program counter came from another: `Pc` on the delay slot of the idle thread's `b .`, `NextPc` also on the slot, the branch still pending and the slot flag already up. No instruction boundary produces that combination, in the interpreter or in a compiled block; the two legal ones are the branch just run (next program counter on the target) and the slot just run (program counter on the branch), and the Super Mario 64 and Ocarina of Time states saved the same evening hold one each. Loaded, the idle thread ran its slot twice, fell out of the loop into libultra's thread cleanup, destroyed itself, and the dispatcher restored the empty run queue's sentinel as a thread: a stack pointer of ASCII spaces, a return address of `"0000"`, user mode, and an address error that the game's handler then took forever. The recompiler was ruled out first, by running each state with blocks and without and finding every frame identical (`Mars_Recompiler.md` §0).

**The mechanism.** The serialiser reads the CPU's fields and then the bus while the CPU goes on changing both, so a state saved during a frame is the registers of one instant and the memory of a later one. A Mars save is also not a read: `MemoryBus.WriteState` settles the VI and the AI before serialising them (`Mars_Performance.md` §9), so the UI thread was *writing* to devices the emulation thread was writing too, the AI's sample queue among them, and a `Queue<T>` is not safe for two writers. A load replaced fields under a frame in flight and cleared the rewind buffer the emulation thread was appending to. None of this is Mars's alone — the call path is the frontend's, for every core — but the window in which a save overlaps a frame grows with the time a save takes, and a Mars state is 6.4 MB. No torn state of another core has been observed, which is not evidence that none was written.

**It was half-known.** §4.13's slot tests paused before saving, with the comment *"SaveState reads core state the emulation thread is writing"*: the race was worked around in the test rather than removed from the frontend. The DianaOS `state` command was never affected, because the console queues anything that is not read-only for the emulation thread (§4.21); Hotaru was never affected, because its hotkeys set request flags its own loop consumes between frames (`EmuSen_Frontend_Driver.md` §2).

**Demonstrated before it was changed.** `MainWindowSaveStateThreadTests` runs a synthetic N64 program that increments a register and stores it to RDRAM on every pass of a four-instruction loop, so in any state that is one instant the stored copy equals the register or trails it by one. Against the unmodified frontend the test that saves through the hotkey while the game runs failed on its first save in each of four runs, memory ahead of the register by 3,370 to 5,977 passes — the CPU serialised first, the memory after, the loop running between. A first version of the paused test failed as well, and that fault was the test's: it took sixty quiet milliseconds as proof that the thread had parked, and this program's frames end on the cycle cap, paced at about 44 ms by the cycles the last one ran. With a 250 ms window the paused save was consistent in three runs out of three. A save while the thread is truly parked was always safe; what `pause` could not give the hotkeys is certainty that it had parked.

**The change.** The handlers still check that a game is loaded and that a load's slot is not empty, then capture the path and the slot and queue a closure. `RunCoreRequests` runs the queue on the emulation thread: after each frame, beside the console's queue and the cheat flag and before the rewind snapshot, so a loaded state seeds the next snapshot rather than following a stale one; after a rewound frame on that path; and while paused. The paused wait is now a wait on the pause signal *or* a request signal, so a save or load is served with no frame run, and `StopEmulationThread`'s `Set` still wakes the thread to exit. The signal is reset before the queue is drained, so a request queued during the drain wakes the next wait instead of being lost. The status text and the slot labels are posted back to the UI thread. Requests queued for a session that is then replaced are dropped when the next thread starts.

**What it does not cover.** A state on disk written before this change may be torn, and nothing tells a torn state from a good one in general; the Wave Race state of 2026-09-19 is one, and should be discarded rather than debugged. A save requested and the window closed before the next frame is lost, since stopping drops the queue. The input race of §4.21 is unchanged and still accepted. The UI thread no longer waits for the disk write, so the status arrives a frame later than it used to.

**Tests.** The three in `MainWindowSaveStateThreadTests`: states saved through the hotkey during play are each one instant (twelve saves spread across the frame); a save while paused is written and the frame count does not move; a load during play takes the machine back and leaves it one instant. §4.13's three slot tests keep their pause, which now exercises the woken thread, and wait for the posted status instead of reading the file at once.

### 4.21b The N64's picture is one frame behind the machine

*2026-09-19.* When the loaded core is Mars, `LoadRom` sets `MarsCore.DeferredPresentation`, and the video
interface's scan-out — a quarter of an Ocarina of Time frame's time in play — runs on a pool thread while the
emulation thread runs the next frame (`Mars_Video.md` §2.7). The picture `GetFrameBufferRgba()` hands the loop
after frame *n* is therefore frame *n* − 1's, exactly; audio, input and states are the frame's own. At 50 or 60
fields a second that is 17 to 20 ms of display latency, the price of the frame time it returns
(`Mars_Performance.md` §27). A loaded state, from a slot or the rewind, is presented at once. Hotaru does not set
it. Nothing here is a setting yet; if the latency is ever unwanted, the property is where a toggle would go.

The same `LoadRom` sets `MarsCore.ThreadedRdp`, which runs the display processor's lists on a pool thread behind
marks on the RDRAM pages they reach (`Mars_Rdp.md` §2.6, `Mars_Performance.md` §28). That one adds no latency and
changes nothing the frontend sees: a state, a cheat and the picture all wait for the list. Between the two, an N64
frame now uses up to three cores of the host.

*2026-09-20.* The same place sets `MarsCore.RdpWorkers` to one processor per three logical cores, at most four,
so a sixteen-thread host gets four, an eight-core host two, and a four-core host the one thread it had. Each is a
complete display processor on a thread of its own shading every fourth (or second) row of every primitive, exact
to the byte (`Mars_Rdp.md` §2.8); on this machine the second pair of workers adds a tenth where the first pair
adds a third to a half (`Mars_Performance.md` §35), which is where the four comes from. The weak-machine case,
where those threads share cores with the emulation thread and the scan-out, is unmeasured, and the divisor is the
number to revisit when it is. Not a setting; the property is where one would go.

*Later still:* the three values above are no longer set by hand here; they are Mars's defaults in the graphics
window (§4.26), and a user can change each.

*Later the same day:* **rewind is off for the N64 in this frontend for now.** The first play test of the workers froze
Super Mario 64 in a level, and the rewind buffer's snapshot every fourth frame is the one path the play harness never
took with several workers; a snapshot asked for while every worker waits inside a command — a list handed over up to
the middle of one, which games do — could not be answered, and a test reproduces the hang. Until the snapshot with
workers is proven in play, `OnFrameCompleted` is skipped for Mars; the hotkey then has no history to step back
through. The other cores' rewind is unchanged.

*2026-09-23: rewind is on for MarsRT (Rust) and still off for Mars (C#).* ~~The C# core's pause can still deadlock its
workers, and a WiseMan test still shows it (`Mars_Native.md` §5.6.6), so nothing changes for it.~~ *That deadlock was
fixed the same day, in the next paragraph; the C# core's rewind stays off until it is played.* MarsRT's snapshot
pauses its workers at a word each answers by number, and rewinding it with four workers and the picture deferred —
Mistress's settings — was checked frame by frame on three games: every step back lands on exactly the state the
machine had at that frame and shows the picture a load of it shows, rewinding and playing alternated at random for
600 operations a game never froze, and the pictures the screen still holds are never overwritten
(`Mars_Native.md` §6.6.3). For a player on MarsRT the hotkey now works as on the other consoles: every fourth frame
is kept, as far back as the 96 MB budget reaches, and a step back shows the frame at once rather than a frame late.
Each capture writes a 13 MB state on the emulation thread, into an array kept for it: on the development desktop 2.5 to 3.8 ms one frame in four, about 1 ms a frame in all with the buffer's own encoding, and 0 to 2 of the buffer's garbage collections in 300 frames (`Mars_Native.md` §6.6.3). The handheld was not measured.

*2026-09-23:* the C# core's snapshot with several workers had a second way to hang, a pause that found some workers at
a barrier and the rest short of it; a snapshot every frame from Ocarina of Time's state met it within a second of play.
It is fixed, with the race that parted the split from itself in the same game (`Mars_Rdp.md` §2.9.1, §2.9.5). Rewind
stays off until it is proven in play, which headless runs cannot do.

*2026-09-24:* the history is now also a reel of pictures the player chooses from, from the pad's menu or Emulation →
Rewind... (§4.49). Each snapshot carries a 160-pixel picture; the C# Mars still keeps no history, so its menu entry says
so. On MarsRT a moment chosen from the reel was proven to be exactly the state saved at it, on a synthetic system with
four workers and the picture deferred (`Mars_Native.md` §6.6.3, `EmuSen_Rewind_And_FastForward.md` §5.2).

### 4.26 The graphics window: one tab per console, the settings its core offers

*2026-09-20.* Settings → Graphics Settings... opens a LunaP `ToolWindow` built in code like the preferences: a hint,
a `Tabs` with one tab per console in release order, and two buttons. Each tab is the console's `CoreSetting`s from
`CoreCatalog.SettingsFor` (`EmuSen_Multicore.md` §13), one `FieldRow` each with the label and the hint, its control a
`LunaSwitch` for a switch, a `Dropdown` of the range's counts for a count, or of the names for a choice; a console
with none shows an `EmptyState`. The controls are named `<console>.<key>` so a test can find them. The tab opened
is the running game's console, as the controller window does.

**A change is saved at once and reaches a running game between frames.** The window writes `GraphicsConfig.Consoles`
and saves on every change, and tells the main window which console changed; if that is the running one, the main
window queues `ApplyConsoleSettings` on the emulation thread, where the core's setters may join their threads.
`ApplyConsoleSettings` is also what `LoadRom` runs, in place of the values §4.21b hand-set for Mars: every setting
the core offers, from the config or its default, and a value the core refuses (a hand edit) falls back to the
default. *Reset This Console* forgets the console's entries and shows the defaults again.

What is not here: the frontend's own window size, vsync and filtering (`graphics.json`'s other fields, which this
frontend does not read), and anything a core does not offer. §4.21b's numbers are now Mars's defaults, and the
weak-machine question it left open is now the user's to answer from this window.

**Addendum, 2026-09-20: the first choice setting.** The N64's tab gained *Internal resolution*, a dropdown of 1 to
4: the multiple the core draws the picture at (`Mars_Rdp.md` §11). It is internal. The frame the window scales is
640×480 at 2 and 1,280×960 at 4 rather than 320×240 stretched, so edges and textures resolve where the console's
could not, and the window's own size is not changed by it. The game is not changed by it either: what the game
reads back, save states and the probe are the console's exact drawing, which is why it can be switched while
playing and takes effect at the next frame. Its cost is the display processor's — each step draws its square in
pixels, four times the work at 2 and sixteen at 4, on the RDP threads, so `RdpWorkers` helps it directly — and the
memory for the multiple's frame, 32 to 128 megabytes. What it cannot show: a frame the game's CPU writes rather than
draws, which the core's drawing does not see, comes out black or stale at a multiple. What it costs in play is
`Mars_Rdp.md` §11.1.

**Second addendum, 2026-09-20: the Expansion Pak.** The N64's tab gained *Expansion Pak*, a switch, on by default,
and the first setting there that is not about the picture. Games that need the accessory (Majora's Mask shows its
own "not installed" screen without it) now start, because the console a frontend builds has it; the switch is for
taking it out. The game reads the memory's size once as it boots, so a change while playing waits for the next load
of a game, while the value applied at `LoadRom` takes effect at once. Save states made before this, on four
megabytes, still load: the core rebuilds the console to the state's memory (`Mars_SaveStates.md` §1), so such a
game resumes without the Pak until it is loaded afresh.

**Third addendum, 2026-09-20: antialiasing.** The N64's tab gained *Antialiasing*, a dropdown of Off, 2x, 3x and 4x.
It draws the picture that many times finer each way and averages it down to the internal resolution, which smooths
edges and texture shimmer without making the frame larger (`Mars_Video.md` §2.10). It costs what the same internal
resolution would, and the two multiply, so together they are held to four: at 2x resolution the most antialiasing
is 2x, and at 3x or 4x resolution it does nothing. The game is unchanged by it, as by the resolution.

**Fixed, 2026-09-20: the tab could not scroll.** Each tab has been a `ScrollViewer` since the window was written,
and with four settings nothing showed it did not work. With seven the N64's tab ran off the bottom of the window and
no scroll bar came. The window's content was a `Ui.Stack`, and a stack panel measures its children with unbounded
height, so the tabs, and the scroll viewer inside, were told they could be as tall as they liked: the viewport was
710 pixels high in a window of 400, and a viewport that holds everything never scrolls. The content is now a
`DockPanel`, the hint docked to the top, the buttons to the bottom and the tabs filling what is left, which bounds
them. `A_tab_taller_than_the_window_scrolls_and_its_last_setting_can_be_reached` fails against the stack with that
710 and passes with the dock. The hint also stopped claiming that every setting keeps the output exact, which the
internal resolution, the antialiasing and the Expansion Pak had made untrue. The controller window has the same
scroll viewer per tab but is laid out from markup, and was not examined.

**Added, 2026-09-21: an eighth N64 setting, "Draw the multiple on the graphics card".** `MarsCore.Gpu`, a switch, off
by default. With an internal resolution or antialiasing above one it shades the picture at the multiple on a Vulkan
compute device instead of on the processor's own threads (`Mars_Gpu.md` §11). The window needed no change: it builds
its rows from the catalogue, and the setting arrived as one more entry there. It does nothing at one, which
`At_one_the_device_setting_changes_nothing_and_holds_no_device` holds, and nothing without Vulkan, where the CPU path
draws as before. What the setting actually got — a device's name, or the sentence saying why there is none — is
`MarsCore.GpuReport`, which the window does not yet show; that is the one frontend change still owed to it.

### 4.27 A fault leaves a report

*2026-09-20.* A game stopped when its pause menu was opened, twice, and not a third time from a state saved just
before; eight headless runs from that state, opening and closing the menu at different frames, found nothing. The
fault was intermittent, and the frontend had kept no record of either occurrence. The emulation loop catches
whatever the core throws — including what the display processor's threads threw, which the core rethrows on the
machine's thread — and showed only the message, as `[CPU HALT]` in the status bar; and nothing at all recorded a
fault on any other thread, which ends the process. `CrashLog` now writes either to
`crash_<date>_<time>.txt` in the log directory (§4.22's root): the kind, the console, the frame, the console's
settings as the window would show them, and the exception whole, inner exceptions and traces included.
`Program.Main` installs it before anything else for unhandled exceptions and unobserved tasks, and the emulation
loop calls it from its catch and names the file in the status bar. Writing is best effort, since a report that
cannot be written must not become a second fault. `CrashLogTests` holds the file's contents. What this does not
do is find the fault: it is the instrument for the next occurrence, and the pause-menu fault is still open.

### 4.28 A late frame's time is owed, up to three frames

*2026-09-20.* §4.21's loop, finding itself past a frame's tick, reset the tick to the present. That is right for a
stall and wrong for a core whose frames cost unevenly: the N64's games draw one picture every two or three fields,
the drawing field costs more than its slot and the others far less, and resetting the tick threw away about ten
milliseconds at every drawing frame. Ocarina of Time ran at 96.8 per cent of full speed while averaging twelve
milliseconds against a slot of twenty (`Mars_Performance.md` §37). `FramePacer.Settle` now leaves a late tick where
it is, so the frames after it run without waiting until the time is made up, and moves it only when more than three
frame intervals are owed, to three: a long stall is still forgotten rather than replayed as a burst. The same run
holds 100.0 per cent. `FramePacerTests` holds both halves. What it does not do is make a drawing frame cheaper;
the picture's cadence is still the game's.

### 4.29 The interface steered from a pad, and the big screen

*2026-09-21.* Until now a controller played the game and did nothing else: the library, the menus and every settings
window wanted a keyboard or a pointer, which a handheld has neither of. The gamepad was not even polled unless a game
was running. This section is the design that closes that, what was borrowed for it, and what it does not reach.

**What was borrowed.** EmulationStation's button grammar, because it is the one a player of these machines already
knows: the south button accepts and the east button goes back, Start opens the main menu, the shoulders move a page,
the triggers go to the first and last entry, left and right change the system, and a footer names the buttons.
OpenEmu contributed one idea rather than a layout: a menu laid *over* the running game that carries the save states,
so that nothing a player wants mid-game needs the menu bar. OpenEmu's console sidebar and cover grid were not
borrowed. The library here is a titled list with a console filter, there is no cover art to show, and a grid of
identical tiles is a worse list.

**The layers.** `GamepadManager` gained `IsRawPressed` and `RawAxis`, which read the physical pad and ignore the
game's bindings: the interface must work on a pad whose bindings are wrong, since fixing them is one of the things it
is for. `PadNavigator` turns what is held into presses. Every button presses once on the way down; the four
directions and the two shoulders then repeat, after 400 ms and every 80 ms, and nothing else repeats, because a held
accept that repeated would start a game and then answer its first prompt. It takes the time as an argument and owns no
clock, which is what lets its tests state those numbers exactly. `MainWindow.Pad.cs` polls at about sixty times a
second from a dispatcher timer of its own, so the pad is read whether or not a game is running, and routes each press
to one of three places in a fixed order: another window if one is active, else the pad's menu if it is open, else the
library if it is showing.

**The mapping.**

| Button | Library | Pad menu | Another window |
|---|---|---|---|
| D-pad, left stick | up and down move one title; left and right step the console filter, wrapping | up and down wrap; left and right change an entry that has a value | ~~up and down move the focus (Tab and Shift+Tab), or the rows of a list, or an open dropdown; left and right step a closed dropdown~~ *since 2026-09-24 all four move the focus by position, and a list, a dropdown or a slider keeps the ones it uses (§4.45.3)* |
| South | start the game | choose | press the button, flip the switch, open or commit the dropdown |
| East | back to a suspended game | close | close an open dropdown, else the window |
| L1, R1 | ten titles | | previous and next tab |
| L2, R2 | first and last title | | |
| North | focus the search box and ask for the keyboard (§4.30) | | |
| Start | open the menu | close | |
| Guide, or Back and Start together | open the menu | | |

**While a game is on screen the pad is the game's.** Only the chord reaches the interface: the guide button, or Back
and Start held together. Start alone cannot open the menu there, because Start is a button every one of these
consoles has. The guide button alone would not do either, since Steam takes it on a Deck, which is why the two-button
chord exists. The menu pauses the game before it shows, the library's rule (§4.18), and resumes only what it paused:
a game the player had already paused stays paused when the menu closes.

**The press that closes a menu is not the game's.** The east button that dismisses the menu is, a sixtieth of a
second later, still held, and to the game it is a button. `PollGamepad` therefore hears nothing while the menu is up,
while another window is active, and afterwards until every button has been let go. `PadNavigator.Forget` is the same
idea in the other direction, so the chord's own Start is not a press in the menu it opened.

**Other windows are driven through their keys, not rewritten.** `PadWindowRouter` synthesises the key events a
keyboard user would type, and sets a dropdown's index or a switch's state directly where a key would need the popup
open. Every LunaP window is therefore reachable without knowing a pad exists, including ones not yet written. The
cost is that it is only as good as the window's tab order. *Retired 2026-09-24: the focus now moves by position, and
a window's tab order no longer decides what the pad reaches (§4.45.3).* A window takes the pad only while it is the *active*
window, so a debug window left open beside a game takes nothing from the game.

**Big screen.** `AppSettings.BigScreen`, the `--bigscreen` argument, or `SteamDeck=1` in the environment, which a
Deck's own session sets, starts the window full screen with the menu bar hidden and the library's text at 24 points.
Everything the bar held that a player wants is in the pad's menu; the rest is a desktop session's business. It is
read once, at start, because hiding and restoring the bar under a running game bought nothing worth the states it
adds.
*Retired 2026-09-26 (§4.54):* the user asked for a way into big picture from the desktop, and a Big Picture entry
below the plain Fullscreen one in the View menu now enters it while the window runs. The argument above priced the states a
switch adds against nothing bought; the request is what it buys, and §4.54 lists each state and the test that holds it.
*Corrected 2026-09-22 (§4.43):* the premise that `SteamDeck=1` marks a Deck's own session was wrong. Steam sets it on
a Deck in Desktop Mode as well, so a game started from Steam on the desktop lost its menu bar and sidebar. The
environment trigger is now a gamescope session, and `SteamDeck=1` counts only where no desktop is named.

**The menu's entries.** Over a game: Resume, Save State, Load State, State Slot, Speed, Reset, then the three
settings windows, Full Screen, Game Library, Close Game and Exit. In the library the game's entries give way to a
single "Back to" line when a game is suspended. State Slot and Speed are changed with left and right and do not
close the menu; every other entry closes it first and then acts, so a window it opens is not opened behind it.

**Coverage.** `PadNavigationTests`, twelve cases: the repeat timing to the millisecond, accept never repeating, the
chord firing once, `Forget`; the library's moves and the clamp at either end; the console filter stepping and being
saved; the menu pausing and resuming, and leaving a paused game paused; its wrap and its slot entry; and the graphics
window driven end to end, tabs, focus walk, dropdown and switch. Four mutants were each caught by exactly one case:
resuming unconditionally, letting accept repeat, dropping the synthesised Tab, and clamping instead of wrapping.

**What it does not cover.**

- ~~*No test holds a real pad.* The cases enter at `OnPadCommand` and at `PadNavigator.Feed`. The mapping from SDL's
  buttons to `UiButton`, the stick threshold of 0.55 and the release wait after a menu closes are read from a
  physical pad that a headless run does not have, and are verified only by hand. A fake `GamepadManager` is the
  harness extension that would close this.~~ *Closed 2026-09-24 by §4.45.1: the pad is now simulated inside
  `GamepadManager`, beneath the mapping, and the threshold is under test.*
- *Text entry.* Closed the same day by §4.30, as far as it can be closed without a Deck to try it on. *Closed again,
  differently, by §4.45.6.*
- *File pickers and the binding capture.* The system's file dialog is not an Avalonia window and the router cannot see
  it. The controller-binding window can be walked and its capture started from the pad, but what the capture then
  hears is that window's own business and was not changed. *Since 2026-09-24 the capture is the pad's too (§4.45.4),
  and a path can be typed where the dialog would be needed (§4.45.6); the dialog itself is still out of reach.*
- *Menus of the menu bar.* A popup is its own top level (§4.24), and the router does not drive it. In big-screen mode
  the bar is hidden, so this is a desktop-mode gap only.
- *A defect the tests found on the way, fixed beside this work:* stepping the filter onto "SNES (Venus)" was saved as
  such and read back as every console, because the legacy upgrade ran on every load rather than once. See
  `EmuSen_Multicore.md` §10.3a.

### 4.30 Text from a pad: Steam's keyboard is asked for, not rebuilt

*Retired 2026-09-24 by §4.45.6: a pad now opens Mistress's own on-screen keyboard, and `SteamKeyboard` is gone. The
section is kept for the argument it made and the reason it no longer holds.*

*2026-09-21.* §4.29 left text entry to a keyboard the player does not have. On SteamOS there is already an on-screen
keyboard, the player already knows it, and Steam opens it for any client that asks through the URL
`steam://open/keyboard`. `SteamKeyboard.Show` hands that URL to the running client. What it types arrives as
ordinary key events, so nothing downstream of the text box knows where they came from.

**Why not a keyboard of our own.** It would be a second one on the machine this is for, laid out differently from
the one a finger already knows, with no languages and no swipe, and it would be ours to maintain. It would earn its
place only on a machine with a pad and no Steam, which is not the target (§4.29). That case is left open rather than
argued away: see below.

**When it is asked for.** In the library the north button focuses the search box and asks; the hint footer says so.
In any other window, accept on a focused text box asks, where it would otherwise have sent Enter. The library keeps
moving under a focused search box, so a match can be chosen with the D-pad and started with the south button without
leaving the box, and the east button leaves the box for the selected row, keeping what was typed. Enter from the
keyboard starts the top match, which is what Enter in that box has always done (§4.11a).

**Only a Steam that is already there is asked.** The command that delivers the URL to a running client is the same
command that starts a client when none is running, and a search box that launches Steam on a desktop is a defect.
`IsAvailable` is therefore true only under a Deck's session (`SteamDeck=1`), under a game launched by Steam
(`SteamGameId`), or when `~/.steam/steam.pid` names a live process. Otherwise `Show` does nothing and says so by
returning false. A launch that throws is also false and nothing more: a missing keyboard must not take a window down.

**Coverage.** Four cases in `PadNavigationTests`, with the launcher, the environment and the running check replaced,
so that no test starts Steam: the gate's three ways of being open and its being shut; a throwing launcher; the
library's search, list movement under it and the way back; and accept on a text box in the preferences window. One
mutant, removing the router's text-box case, was caught by the last.

**What it does not cover.**

- *It has not been seen to open.* No test can, and this machine was not in a Steam session when it was written. The
  URL is Steam's documented one; that Game Mode honours it for a non-Steam shortcut is the claim to check first on a
  Deck. Steam and X together open the same keyboard by hand, so a failure here costs a chord, not the feature.
- *Where it opens.* The URL takes a position and size; none is passed, so it docks where Steam puts it and may cover
  the list's lower rows while typing.
- *A pad with no Steam.* Nothing opens and the box wants a real keyboard, as before.
- *The system file picker* is still out of reach (§4.29); typing a path into a path field's box is now the way round it.

*A plan, not a record, sits beside this file: `EmuSen_Mistress_LibraryPlan.md` reads OpenEmu's frontend and says
what it would take to model this library on it, which of its ideas are worth taking, and which are against this
project's own rules. Nothing in it is built.*

### 4.22 Logging is redirected per ROM, and redirected unconditionally

*2026-08-16, from the same comment-block move as §4.21. `EmuSen_Project_Overview_v2.md` describes what `CategorizedLogWriter` produces; this is why this frontend calls it the way it does.*

**Once per `LoadRom`, not once per process.** The console build loads one ROM per process and can set logging up at startup; this frontend can load several across one session, so each gets its own timestamped directory. The naming mirrors the console build's `<CoreName>/console_<timestamp>/` convention with a `gui_` prefix instead, rooted at `AppSettings.LogDirectory`.

**No second logging mechanism was built.** `CategorizedLogWriter` works by redirecting `Console.Out`, and the core already emits every diagnostic through plain `Console.WriteLine`, so this frontend gets the same categorized cpu/ppu/apu/memory/debug/general files the console build does for free — cartridge load info, `DebugSettings`-gated traces and all.

**The redirect is unconditional, and that is a performance decision rather than a logging one.** Several `DebugSettings.*Logging` flags default to true — `Dma`, `CameraRam`, `RenderRead`, `AllScrollWrite`, `MathUnit`, `BgModeChange`, `MosaicWrite` (§1) — so the core writes a steady stream of lines whether or not anybody asked for logging. Leaving `Console.Out` un-redirected does **not** turn that off. It sends the same volume of lines to the raw, synchronous console writer one syscall at a time, instead of to `CategorizedLogWriter`'s batched background thread. That gap is part of the same story as §4.21: it is one of the reasons this frontend was so much slower than the console build. Falling back to a default under `%AppData%` when `LogDirectory` is unset, rather than skipping the redirect, keeps the fast path always on — the setting only needs touching to put the files somewhere specific.

**`_originalConsoleOut` is captured at startup, before anything has redirected anything.** Restoring *that* — rather than whatever `_activeLogWriter` happened to be at the time — is what lets logging be turned off again, or repointed, without leaving `Console.Out` aimed at a disposed writer.

**Two orderings matter on the way out.** `StopLogging` calls `_session?.FlushVerboseLogs()` **before** `Console.SetOut`/`Dispose`, for the reason that method's own contract gives. And `ShutDownCurrentSession` flushes on the *old* session explicitly, because `StartLogging` begins by calling `StopLogging`, which would otherwise run against the session that has already been replaced.

A failed setup is best-effort: an unwritable log directory leaves logging off for that session and says so in the status bar, rather than blocking the ROM load over it.

### 4.23 Filters that outlive a session

The library filter bar and the cheat database's game filter both start empty on every launch, and both drive real work — a ROM-directory scan and an on-disk cheat query. Their state is now persisted, in `AppSettings`, beside the `SelectedCore` that is already the *facet* half of the same library bar. A separate file was considered and rejected for exactly that reason: one filter bar's two halves living in two files is a worse arrangement than one field more in `appsettings.json`.

Three fields, two of them new: `SelectedCore` (already there), `LibrarySearch`, `CheatSearch`. See `EmuSen_Config_Reference.md` §3.1.

**This was not safe to write until LunaP 0.10.0**, and that is the whole reason it was not written earlier. `FilterBar` raised `Changed` when an application assigned `SearchText`, so restoring a saved filter on open would have run a library scan or a cheat-database query nobody asked for — the exact re-query LunaP's §80.1 describes, with these two windows named as the consumers that would have paid for it. The summary always said setting it did not raise `Changed`; the code did not do that until 0.10.0. `FilterPersistenceTests.Restoring_a_search_does_not_raise_Changed` pins the guarantee here rather than trusting the toolkit's own suite, because a LunaP upgrade that regressed it would break this feature silently and in a way that costs disk I/O rather than throwing.

**Written on close, not on `Changed`.** No `SearchDelay` is set anywhere in this repository, so `Changed` fires per keystroke, and saving there would be a config write per character typed. `MainWindow` already had a `Closing` handler; `CheatDatabaseWindow` gained one.

**The restore happens before the first query, not after.** In `MainWindow` it sits inside the one-time block in `RefreshLibrary`, ahead of the `ShowLibraryEntries()` that ends it, so the first list a user sees is already filtered. In `CheatDatabaseWindow` it is set before `Refresh()`; that window selects no system on open, so nothing is queried either way, but the ordering is the same on purpose rather than by accident.

**`HotkeyHelpWindow`'s table remembers its columns.** Hotaru's hotkey list is the only `LunaTable<T>` in the tree and already carried `WindowKey = "hotkeys"`, so the window remembered where it was while its column widths and sort reset every time. It now carries `TableKey = "hotkeys"` to match. This too was gated on 0.10.0: LunaP §79.2 was a saved layout being applied to a table that had since gained a column, which for a three-column help table is a narrow risk, but the fix is what makes the key worth setting rather than a thing to remember about.

**What is deliberately not persisted.** The cheat window's selected system, and the library's selection. Both are position within a result set rather than a description of one, and a window reopening with row 400 selected in a list that has since been rescanned is restoring a coincidence. The filter says what the user was looking *for*; the selection says where they had got to, which is not the same claim.

### 4.24 Fixed bug: the pad drove the menu bar

Playing with the keyboard operated the menus. Pressing right walked the strip `_File` → `_Emulation` → `_View` → `_Settings`, and pressing Start dropped the selected one open over the game.

**The cause is §4.2 stopping one step short.** That section moved gameplay capture to the tunnel pass so the arrow keys reach `SetButtonFromKey` before anything downstream can take them, and it was right about the reading half. It never made the handler *claim* what it read: the game-input branch set `_keyboardHeld`, called `ApplyButtonState` and returned with `e.Handled` still false, while the hotkey branch immediately below it had always set it. The event therefore carried on down the tunnel to the focused element.

On the game screen that element is the menu bar, because it is the only focusable control the window has — `GameFrameControl` is not focusable and nothing focuses it, so focus lands on the strip when the window opens and never leaves. Avalonia's menu interaction handler reads Left/Right on a focused top-level item as movement along the strip and Enter as "open this one". Five of the twelve default bindings (§4.3's four arrows, and `Enter` for Start) are exactly those keys, so the default map and the menu's own navigation are the same keys by construction, not by coincidence.

**The fix is one line**: the game-input branch sets `e.Handled = true` before returning. Its scope is the two guards that already sit above it — a focused `TextBox` returns first (§4.17) and the library screen returns first (§4.18) — so the claim it makes is only "a key bound to a pad button, while a game is on screen".

**What the fix does not cover.** `Alt`-based access keys are untouched, because `Alt` is not a pad binding: `Alt`+`F` still opens File during a game, which is the documented way to reach the menus without a pointer. Nothing here changes what happens once a menu *is* open — the popup is its own top level, this window's tunnel handler is not in that route, and the pad has always been dead until the menu closes. And it is a Mistress fix only: Hotaru has no menu bar, so its window never had the defect to begin with.

**The alternative was focusing the video surface**, making `GameFrameControl` focusable and taking focus on launch. Rejected on two grounds: it leaves the menu bar in the tab order, so the arrows reach it again the moment focus moves for any other reason, and `GameFrameControl` is shared with Hotaru, where the change would buy nothing. Claiming the key states the narrower and truer thing — these keys belong to the game — rather than arranging the window so that nobody else is listening.

**Regression coverage**: `MainWindowLibraryTests.Start_presses_the_pad_instead_of_opening_the_focused_menu` and `The_d_pad_does_not_walk_the_menu_strip`, which focus the real `_File` item the `MenuBar` built and then press pad keys. Both fail against the unmodified build — `Right` moved the strip's selection onto `_Emulation`, and `Right` then `Enter` left a submenu open — and both pass with the line added, with the other 179 Mistress tests unchanged.

This retires half of a prediction §4.2 recorded. That section says the arrow-key half of its problem "does **not** reproduce headlessly, since headless has no real focus-navigation pass". True of the focus manager's directional navigation, and it is still not measured here. The menu strip's own key handling is not that pass — it is ordinary routed-event handling on a focused control — so the visible consequence of an unclaimed arrow key does reproduce headlessly, and is now measured.

### 4.25 The theme picker, and the promise it closes

*Added 2026-09-06, out of the windowing audit (`EmuSen_LunaP.md` §8).* `man theme`
has told users to drop a theme in `/etc/EmuSen/themes` and "pick it in Preferences;
there is no registration step and no restart" for as long as themes have existed.
`PreferencesWindow` offered three directories and a core. `LunaTheme.Apply` was
called nowhere in this repository and `LunaTheme.Available` was read nowhere, so the
only way to change theme was to hand-edit `luna.json` — `LunaApp.Configure` calls
`ApplySaved` on the way up, which is why the feature nevertheless *worked* for
anyone who knew that.

**What it is**: a `Dropdown` in a `FieldRow` labelled Theme, filled from
`LunaTheme.Available()` — the built-in first, then each theme file by name without
its extension — and selected on `LunaTheme.Current`. Choosing calls
`LunaTheme.Apply`, which persists the choice itself; nothing here writes
`luna.json`.

**A theme that will not parse must not leave the row lying.** `Apply` returns false
and leaves `Current` untouched, by design, so that a bad file cannot leave the
application unstyled. The handler re-fills the row from `Current` on false, because
a dropdown reading "broken" over a window still drawn in the old theme is a worse
failure than the one it is reporting. The diagnostic itself goes to
`ConfigDiagnostics` through the `LunaSettings.Diagnostics` seam `Program.cs`
already sets (`EmuSen_LunaP.md` §3).

**"No restart" is measured, not assumed.** `A_chosen_theme_reaches_a_window_that_was_already_open`
opens a window, chooses a theme in another, and asserts the first window's
`Background` moves from `#ff1e1e1e` to the new theme's `#ff101018`. The palette is
bound with `DynamicResource` throughout, so this was expected to work; the man page
makes the claim to users, which is the reason it is pinned rather than reasoned.

**The core row became a `Dropdown` too, and that deleted something.** It was a bare
`ComboBox` with an `_initializing` flag guarding against the spurious
`SelectionChanged` that setting `ItemsSource` raises. `Dropdown.Fill` restores a
selection *without* raising `Chose`, which is that flag's entire job done by the
toolkit. The flag, the field and the `SelectionChanged` handler are gone; the
control is still named `CoreComboBox` and `Dropdown` derives from `ComboBox`, so
nothing looking for it by type or name had to change.

**Test coverage** is `EmuSen.WiseMan/Mistress/PreferencesThemeTests.cs`, four tests,
all four red against the build before this one. Removing only the re-fill on a
failed apply reddens exactly `A_theme_that_will_not_load_leaves_the_applied_one_alone`
and nothing else.

**A finding about the harness, recorded because the first version of these tests was
wrong.** `LunaSettings.Store` is process-global and documented "set it once at
startup". Giving each test its own store — the reflex, and what every other window
test here does with `ConfigStore.OverrideDirectory` — makes `LunaTheme.Saved` report
the *first* store's value while `LunaTheme.Current` reports the new one, so a test
that passed alone failed after a sibling. That is the toolkit's contract being kept,
not broken: the remembered choice is read through the store LunaP first saw. The
tests take one store for the class through an `IClassFixture` and clear the themes
folder per test instead.

**What is not covered.** No test asserts rendered *pixels* change, only the resolved
brush. Only the `.css` theme form is exercised; the `.axaml` form is untested here
and is LunaP's own to cover. And nothing offers to open the themes folder, so the
first theme a user installs still requires knowing where it goes — which is what
`man theme` is for.

### 4.31 Resume where you left off (2026-09-21)

Leaving a game writes its state; starting it again offers that state back. This is OpenEmu's auto-save on quit and its "Would you like to continue your last game?" prompt, and it is stage 0 of `EmuSen_Mistress_LibraryPlan.md`, taken first because a handheld is suspended and closed rather than saved from a menu.

**Where it is written.** `ShutDownCurrentSession` writes `SaveLibrary.ResumeStatePathFor` (`EmuSen_Galaxia.md` §5.2) after the emulation thread has stopped, so the state is one instant of the machine without going through the request queue of §4.21a. The window's own Closing handler does the same. A Reset calls `LoadGame(..., reset: true)` and writes nothing, since it is not leaving the game; this is the case the reset test pins, because a reset that wrote the resume state would make the next start offer the moment before a reset the player asked for.

**Where it is asked.** Every way of starting a game from outside it (Open ROM, a drop, Browse ROMs, the library) now runs `StartGameAsync`: the firmware prompt, then the resume question, then the load. `AppSettings.ResumeOnLaunch` is `Ask`, `Resume` or `Restart`; the dialog's "Do not ask again" writes the answer given. Closing the dialog, or B on a pad (§4.29's router), cancels the launch rather than choosing, because a player who backs out of a question has not answered it.

**When the state will not load.** The half-loaded session is dropped and the game started again from nothing, with the reason in the status line. The alternative, running a machine whose state load threw part way through, is not a machine anyone asked for.

**What it does not cover.** A crash is not a clean leave: if the process dies, no resume state is written, and one from an earlier session may be offered instead. A core that halted (a fault) is still written on leaving, and resuming it resumes the fault; Restart is the way out. The state is tied to the file's name, not its contents, so a renamed ROM loses it. Tests: `ResumeWhereYouLeftOffTests` (8), and three mutants each caught (no write on leaving; a reset that writes; a resume that does not load).

### 4.32 The player's library database (2026-09-21)

What Mistress remembers about each game it has seen (favourite, last played, play count, time played) is kept in SQLite at `home/Library/games.db`, one row per path, only for games that have been played or marked.

**Why a database and not a JSON file.** `EmuSen_Galaxia.md` §7.2 draws the line at authorship: a human writes the preferences, so they stay JSON; the program writes this, and it has shape, so it is a database. A first version of this section's code wrote `games.json`, on the reasoning that config stays JSON. The reasoning was wrong because this is not configuration, and it was replaced before anything read the file.

**Why not the catalogue's file.** `catalogue-schema.sql` describes a cache that is "always safe to delete". A favourite or a year's play time is not safe to delete, so the two must not share a file whose documented remedy for trouble is removal. The catalogue stays a cache beside it.

**Schema versioning.** `EmuSen_Stack.md` §4.3 names schema versioning as the sharpest gap in the project's storage. This file carries `PRAGMA user_version`, and `GameRecords` holds an append-only list of migrations, each run in its own transaction with the version bump. A file stamped newer than the build is refused with `InvalidDataException` rather than opened, because an older build writing into a newer schema is how a downgrade destroys data. Tests: `GameRecordsTests` (4).

**Play time.** A stopwatch runs while a game is on screen and unpaused; pausing, or stepping out to the library (which pauses, §4.18), stops it, and leaving the game adds it to the row. The count and the last-played stamp are written at the start, so a crash loses at most the time of the session that crashed.

**What it does not cover.** Rows are keyed by path, so moving the library orphans them; identity by hash waits on the catalogue (stage 3 of the plan). *Corrected the same day by §4.37: identity by hash came without the catalogue, in this file.* The driver sits in Mistress rather than in `EmuSen` beside `SqliteCatalogue` (§7.1's convergence argument), because Mistress is its only reader; if Hotaru or the shell ever wants it, that argument applies and it moves.


### 4.33 The library as OpenEmu lays it out (2026-09-21)

The library screen is OpenEmu's library window: a sidebar of collections and consoles on the left, a toolbar, and the games as a grid of covers or as the list of §4.11. It is stage 4 and stage 5 of `EmuSen_Mistress_LibraryPlan.md`, with stage 1's favourites and recency surfacing as two of the sidebar's collections.

**Built from LunaP's controls.** The sidebar is LunaP's `SourceList`, the grid its `TileGrid<T>`, the view and category switches `ActionToggle`s over `LunaAction` radio groups that the View menu also holds, and every context menu is `Menus.Context` over `LunaAction`s. The four new controls were written into the toolkit itself rather than into Mistress, because none of them names anything of an emulator (LunaP's `docs/LunaP.md` §88); while they are unreleased the four projects that use LunaP take it from the sibling checkout through `LunaP.props` (`EmuSen_LunaP.md` §21). What Mistress adds is only what is about games: `CoverTile`, which draws one cover, and `ArtworkIndex`, which finds it.

**The sidebar.** Library (All Games, Favourites, Recently Played) and Consoles (each core in release order), each row with its count. A console row is the console filter of §4.23 and a collection row spans every console, as OpenEmu's does, so the two are one choice rather than two that could contradict each other. The folder is walked once for every console (`RomLibrary.Scan` then `RomLibrary.Narrow`), which is what lets every row carry a count; the walk was already whole-tree, since it recursed. Recently Played is the thirty most recently started, OpenEmu's limit for its Recently Added. In big-screen mode (§4.29) the sidebar is hidden, because on a handheld its width is the screen, and the console facet returns to the filter bar where the pad's shoulders step it.

**The grid.** Each cover is OpenEmu's cell: the art standing on the bottom of a square, the title in 13-point medium beneath it, a second line in the muted colour. Missing art is OpenEmu's placeholder, a faint rounded box ruled with scanlines, in the console's box shape: `CoreDescriptor.CoverAspect` carries OpenEmu's North American ratios (NES 1.43, SNES 0.73, N64 0.70, Game Boy 1.0), so the placeholder is where console knowledge meets the frontend and that is why it sits beside the core's declaration rather than here (plan §2). **One departure from OpenEmu, on purpose:** the placeholder carries the console's name and the game's title. OpenEmu's is blank, which is right for a library whose art is fetched; this one has no art until the user supplies it, and a wall of identical blank boxes was the case the plan's §6 said would make a grid worse than a list. The cover size is a slider (0.5 to 2.5 of the default, OpenEmu's range), also on the View menu. On a pad the grid moves in two dimensions and the shoulders take the console instead (§4.29's grammar otherwise); Select marks a favourite.

**Cover art.** `ArtworkIndex` walks the art folder (Preferences, default `home/Artwork`) off the UI thread and matches a picture to a game by name: the ROM's own file name first, then the name without its region and revision tags ("Super Mario 64 (Europe) (En,Fr,De)" finds "Super Mario 64 (USA).png"), in the console's own folder before a shared one. A folder counts as a console's if it is named for the console or for one of the libretro system names the core already declares for cheats, so a libretro-thumbnails checkout works where it is dropped. Names are compared under libretro's substitution of `&*/:\`<>?\|"` by `_`. Pictures are decoded at tile size on a worker and kept in a cache of six hundred; an evicted picture is not disposed, because a tile may still be drawing it. "Add Cover Art from File..." copies the chosen picture into the console's folder under the game's name, and never touches the chosen file or the ROM folder.

**Nothing is fetched.** The plan's §4 is why: the one dataset OpenEmu uses is unlicensed and hotlinks someone else's scans.

**What this does not cover.** No test renders a cover and inspects its pixels; the tests pin which picture is chosen, not how it is drawn. The fallback match can pick the wrong region's art when two regions' boxes differ, since it discards exactly the tags that say so. The folder is still walked on every return to the library, now for every console at once. Measured on this machine's library (5,520 files, Release, six runs): the whole walk takes 2.7 ms warm and 14 ms cold, and narrowing it to the NES's 3,537 takes 0.6 ms, against 2.4 to 5.9 ms for the one-console walk it replaced, so walking everything costs nothing a person can see and was kept.

### 4.34 The bar over the game, and its notices (2026-09-21)

OpenEmu's heads-up bar is LunaP's `OverlayBar` over the picture: 442 by 42, centred 19 points above the bottom, shown when the pointer moves over the game and hidden a second and a half after it stops, never while the pointer is on the bar or one of its menus is open. Left to right, as OpenEmu orders them: quit (the red pill), pause or resume, restart, a save menu (save, load, the slot), an options menu (speed, graphics, bindings, cheats, a screenshot), the volume between its two speaker glyphs, and full screen (the dark pill). The menus are `Menus.Items` over the same `LunaAction`s the menu bar holds, so the bar can never show a label the menu bar has already changed. The glyphs are path data rather than characters, so no fallback font decides what they look like.

**Notices** are LunaP's `NoticeLayer`, top right, for 1.75 seconds with OpenEmu's keyframes: a state saved or loaded (with its slot), a screenshot, fast-forward starting. A handheld has no status line to say a button did something.

**Screenshots** are a new hotkey (F9, rebindable, `HotkeyAction.Screenshot` appended so existing binding files keep their keys) and an options entry. Like a state's picture (§4.31) the PNG is written on the emulation thread with the frame the machine is on, into `home/Screenshots`, named by the game and the moment.

**Volume** is `AudioPlayer.Volume` (`EmuSen_Audio_Sync.md` §7.3), kept in `AppSettings.Volume`.

**Pausing in the background** is OpenEmu's default and `AppSettings.PauseInBackground`: a running game pauses when the window stops being the active one and resumes when it comes back, **but only if this was what paused it**, so a pause the player chose survives a trip to another window, and a game behind the pad menu (§4.29) stays paused.

**What this does not cover.** The bar has no keyboard or pad route of its own; the pad already has the menu of §4.29 with the same entries, and the keyboard has the menu bar. OpenEmu's "Always Hide HUD" is not offered. The background pause is not exercised by a test, since the headless platform's activation events are not the desktop's.

### 4.35 Save states and screenshots as their own views (2026-09-21)

OpenEmu's toolbar switches between Library, Save States and Screenshots, and so does this one. Both media views are a second `TileGrid` whose tiles are the pictures Mistress writes beside each state (§4.31) and the screenshots of §4.34, each titled by its game and labelled by slot ("Where you left off" for the resume state) or by time. The sidebar narrows them by the game each belongs to, the same way it narrows the library, and the search box by the game's title.

**A state is found by its name.** `MediaLibrary.Parse` reads back `SaveLibrary`'s spelling (`<stem>.state`, `<stem>.slotN.state`, `<stem>.resume.state`) and the game is the library entry with that file name; a state whose game is not in the library is still shown, and says so when played. Playing a state starts its game at that instant with no resume question, since choosing the state was the answer. Deleting asks first.

**What this does not cover.** A state from before §4.31 has no picture and shows the placeholder. Two ROMs with one file name in different folders share their states, which has been true of `SaveLibrary` since it existed and is not introduced here. A screenshot opens in a window at its own size; OpenEmu's share and rename are not offered.

### 4.36 Preferences as OpenEmu's panes (2026-09-21)

The Preferences window is tabs: **Library** (the ROM, cover art, save state and log folders), **Gameplay** (what happens when a game with a resume state starts, pausing in the background, big screen), **Appearance** (the theme of §4.25) and **System Files** (the firmware installed, and where). OpenEmu's Controls pane is Settings > Controller Bindings here, which already had tabs per console, and its Cores pane has no counterpart because the cores are built in.

**The "Emulator Core" dropdown is gone.** It was scaffolding from when one core existed, it said it drove nothing, and it wrote `AppSettings.SelectedCore`, which by then was the library's console filter (§4.23); so choosing a "core" in Preferences silently changed which games the library showed. The sidebar is now the one place that value is chosen.

### 4.37 A game known by its contents, and a state that says who wrote it (2026-09-21)

**A renamed ROM keeps its record.** `games.db`'s second migration gives each game's row the MD5 and size of its file (`EmuSen_Galaxia.md` §5.4) and adds `file_hash`, a cache of path, size, modification time and MD5, so a file is hashed again only when it changes. A game is identified, on a worker, when it is started, marked a favourite or put in a collection, which are the three things that make a row worth keeping. After each walk of the library, rows whose file is gone and which carry a hash are **orphans**; files that have no row and are the size of some orphan are hashed (from the cache when possible), and an orphan whose hash matches moves to the new path with `GameRecords.Move`, taking its collections with it and merging into any row the new path already had (favourite kept, counts and time added, the later of the two last-played stamps). Only files of a missing file's size are hashed, so a library in which nothing was renamed costs one query and no reads.

**What it does not cover.** A file edited in place (patched, or re-dumped) is a different file and keeps nothing. Two identical copies are matched to the first found. Save states are still named by the ROM's file name (`EmuSen_Galaxia.md` §5), so a renamed game's states stay under the old name: the Save States view finds them by the hash in their record, but the resume question and the slot menu, which look up by name, do not. Renaming the state files to follow would mean writing into the state folder on the user's behalf, which on this machine is the ROM folder, and was not done.

**A state records who wrote it.** Every state written from the window (a slot, or the resume state) gets `StateRecord` beside it (`EmuSen_Galaxia.md` §5.3), naming the console, the core, the state version it wrote (`EmuSen_Save_States.md` §6), the build, and the ROM's name and hash. Before a state reaches a core:

- **a state from another console is refused**, with both consoles named, rather than handed to a core that would read another machine's bytes as its own;
- **a state from a newer state version is refused**, naming the build that wrote it;
- **a state from a different copy of the game is asked about**, since loading it may crash the game but may also be exactly what the player wants (a patched copy, a revision), so the question is the player's to answer;
- **anything else goes to the core**, which knows what it can read, and if it refuses, its message is followed by the record's provenance.

The resume question applies the first check before asking and the version check when the core exists; a refused resume starts the game from the beginning and says why. The Save States view shows the record on each state's menu ("Saved by Mars (state version 1), build ...") and deletes it with its state.

**What it does not cover.** States written before this build have no record and are loaded as before, unexamined. A state written by the DianaOS `state save` command gets no record. The build string is whatever the assembly says, which is a version and a commit when the SDK has one and `1.0.0` otherwise. Tests: `IdentityAndCollectionsTests` (the rename, the record's contents, both refusals, the different-copy question) and `GameRecordsTests` (the migration from a first-version file, the cache, the merge); seven mutants each caught.

### 4.38 Collections the player makes (2026-09-21)

OpenEmu's sidebar has a Collections group beneath the consoles, and so does this one: each collection with its count, and a last row, **New Collection...**, which is an action in the place OpenEmu puts its "+". A collection is a list of games and nothing else; `games.db`'s third migration holds them (`collection` and `collection_game`, deleting a collection cascades to its memberships and never to a game). Names are unique regardless of case, because two collections called "RPGs" and "rpgs" are a mistake nobody means.

**Where it is reached.** A game's context menu has **Add to Collection** (New Collection..., then every collection with a tick where the game already is, so the same item adds and removes), and inside a collection **Remove from** it. The sidebar's context menu has New, Rename and Delete; File has New Collection. Names are asked for with LunaP's `Dialogs.PromptAsync`, which was added to the toolkit for this (`docs/LunaP.md` §89) rather than built here. A collection's games follow a rename by hash with the rest of the record (§4.37).

**What it does not cover.** No drag and drop onto a collection, and no smart collections beyond the two fixed ones. A pad cannot yet add a game to a collection, only view one. A collection is shown across every console, as OpenEmu's are, so choosing one clears the console filter.

**A flaky test, found here and older than this work.** During the blast-radius run for §4.37 and §4.38, `MainWindowLibraryTests` failed one test in some runs and none in others, on the same binary: `The_search_box_accepts_a_character_that_is_also_a_hotkey` (the typed "p" never reached the box), `Escape_does_nothing_with_no_game_loaded`, `Enter_in_the_search_box_starts_the_narrowed_selection`, and once `Browsing_the_library_cannot_reach_a_suspended_game`. Measured by running the class alone repeatedly: **1 failed run in 10 at c0f1a72, before any of the OpenEmu work**, 1 in 5 at 072fdbd, 2 in 10 with §4.37 and §4.38 in place. So the flake predates this work, and the difference between the rates is within what ten runs can distinguish. **One hypothesis was tested and refuted:** that a window left open by an earlier test in the class takes the next test's keys, since keyboard focus belongs to the process rather than to a window. Closing every window the class opened, in its `Dispose`, left the rate at 3 failed runs in 20. The cause is not known and is recorded here rather than guessed at.


### 4.39 Covers from OpenEmu's source, if the player asks (2026-09-21)

*Revised 2026-09-26 by §4.60, and kept as the record of how it was built.* OpenEmu's sources are now the **failover**
behind ScreenScraper: the switch is Preferences ▸ Scraping ▸ **OpenEmu Failover** (`OpenEmuFallback`), **on** by default,
and it asks only inside a scrape the player started, for a game ScreenScraper has no cover for or while ScreenScraper
cannot be used; a tile drawn without a cover no longer asks anything. What it fetches goes to
`home/Media/openemu/<console>/`, no longer into the cover art folder, so that a picture the player placed there can be
told apart and win. The old `OnlineCovers` key is read once and dropped (§4.60). What follows describes the switch as it
was on 2026-09-21; the identification, the libretro names, the size check and the "never replace" rule are unchanged.

**Off by default.** Preferences, Library, **Online Covers**. The switch's hint says what it does, what it sends and where, and that the database states no licence and the covers are other people's scans; the library plan's §4 had asked that a fetch be "off by default, one explicit action, clear about what it sends, and preceded by a decision about the licence", and the user made that decision on 2026-09-21 by asking for it.

**What OpenEmu does, and why it could not be copied whole.** OpenEmu identifies a game in OpenVGDB, a SQLite file it downloads from GitHub, and downloads the cover from the address the matching row gives (`EmuSen_Mistress_LibraryPlan.md` §4). Both halves were measured against this library before anything was built:

- **The identification works.** OpenVGDB v29.0 (the newest, 11 November 2021, 9.1 MB zipped, 42 MB unpacked) matched 21 of 40 randomly chosen files by hash. The misses were hacks, prototypes and bad dumps, which it does not list.
- **The covers do not.** All 8 cover addresses tried, on `gamefaqs.gamespot.com`, answered **403 Forbidden**. The refusal was not worked around, since getting past a server that says no by disguising the request is not a feature.
- **So the covers come from elsewhere.** OpenVGDB gives each file's canonical No-Intro name, and that is exactly how libretro's thumbnail server (`thumbnails.libretro.com`, RetroArch's) names its boxes; all four names tried there answered 200 with a PNG. OpenVGDB's own address is still tried last, once per game, in case its host ever answers again.

**How a game is identified.** First by its file name against OpenVGDB's `romExtensionlessFileName`, which costs no read of the file and is enough for a No-Intro-named library; then by MD5 of **the bytes OpenVGDB hashed**, which are not always the file's. That policy is console knowledge and lives beside each core's declaration (`CoreDescriptor.OpenVgdbBytes`, the plan's §2): the NES without its 16-byte iNES header (OpenVGDB's `SYSTEMS` row says 16), the SNES without a 512-byte copier header, the Game Boy as it is, and **the N64 in halfword-swapped order whatever order the file is in**. That last one was found, not assumed: this library's four `.z64` files matched none of OpenVGDB's N64 hashes as they were, and all four after swapping each pair of bytes; OpenVGDB's N64 names end in `.n64`. Files over 384 MiB are looked up by name only, ES-DE's default ceiling. ES-DE, for comparison, hashes every file whole and strips nothing, and its own guide tells users to unzip so that hashes match.

**When a cover is asked for.** When a tile with no art is drawn, so only the games the player actually scrolls past are looked up, one at a time on one worker, a quarter of a second apart. The box is tried under the exact name, then with trailing tags dropped one at a time (at most two more tries), because libretro keeps a revision's box under the plainer name: "Legend of Zelda, The - Ocarina of Time (Europe) (En,Fr,De) (Rev A)" is found as "(Europe) (En,Fr,De)". Only a response that says it is an image and is at least 80 bytes is kept (ES-DE's threshold); it is written to the cover folder under the ROM's own name (so the art index of §4.33 finds it with no change), beside its final name and moved into place, and **a file already there is never replaced**. The answer is kept in `games.db`'s fourth migration (`cover_lookup`: Found, Unknown, NoArt), so a game is asked about once; a network failure is not kept, so the next session asks again, and the game's menu has **Look Up Cover Online** to ask again by hand. The database itself is downloaded the first time the switch is on, from the newest GitHub release, checked to open as an OpenVGDB before it replaces anything.

**Measured live**, with the real servers and ten of this library's games (three SNES, four N64, three Game Boy): **8 covers found** in 4.4 seconds; one Game Boy prototype is not in OpenVGDB; one game, Wave Race 64 (USA) (Rev A), is known to OpenVGDB but has no box on libretro's server under any of the three names tried. Before the tag fallback the same run found 6.

**What it does not cover.** No region preference: the first release OpenVGDB lists is taken, and a European file gets the European box only when that is how it is named. No description, rating or other metadata is kept, although OpenVGDB has them. Tests (`OnlineCoverTests`, `OnlineCoverWindowTests`, 16) run against a synthetic OpenVGDB and a fake server, never the network; seven mutants were each caught, one only after the test's fake page was made larger than the size check it was hiding behind. ScreenScraper, which ES-DE uses and which needs a developer account the project does not yet have, is the natural second source and is not built.

### 4.40 A screen filter per console (2026-09-21)

*Superseded 2026-09-24 by §4.48: the dropdown described here is gone from Graphics Settings, replaced by a row that
opens the Shaders window. The stored `ScreenFilter` value, and everything below about when it is applied, is
unchanged.*

Graphics Settings now opens every console's tab with **Screen Filter**, a dropdown of `ScreenFilters` (`EmuSen_Serenity.md` §3.1), whatever the core offers below it. It is stored in `graphics.json` beside that console's core settings under the key `ScreenFilter`, which no core declares and so no core is ever handed (`ApplyConsoleSettings` passes a core only the keys it lists). The filter is applied to the frame control on the UI thread when a game starts, from its console's value, and at once when the value changes while that console's game is running; another console's change leaves the running picture alone. "Reset This Console" clears it with the rest. A console with no core settings used to show only "No graphics settings yet"; it now has the filter row and a note that the core has nothing else to offer.

**Why per console.** The filter a person wants is a property of the screen the console was played on: a CRT for the home consoles, an LCD for the handhelds, and not the same CRT for all of them. Accurate filters of both kinds are being planned (the CRT and handheld LCD surveys of 2026-09-21); this row is where they will appear, with no change to it.

**What it does not cover.** ~~The two filters available are the simple single-pass ones Serenity already had; nothing here is an accurate CRT or LCD.~~ *Superseded the same evening: each console's row now lists the accurate filters that suit it (`EmuSen_Serenity.md` §3.3 to §3.5): CRT (Lottes) for the NES, SNES and N64; the Game Boy, Pocket, Light and Color LCDs for the Game Boy.* The in-game bar's options menu does not offer the filter yet. Tests: `ScreenFilterSettingTests` (every tab has the row, a choice is saved for that console alone, a game starts with its console's filter and a change reaches it, an unknown name draws no filter); three mutants each caught.


### 4.41 RetroArch's shaders, downloaded on request (2026-09-21)

*Superseded 2026-09-24 by §4.48: the presets are listed in the Shaders window beside the built-in filters, and
`SlangPresetWindow` is retired. The storage, the pack and its download below are unchanged.*

Each console's **Screen Filter** dropdown (§4.40) now ends with **RetroArch Preset...**. Choosing it opens a picker, `SlangPresetWindow`, that lists every `.slangp` in the downloaded pack. The list can be narrowed by the pack's top-level folder (`crt`, `handheld` and so on) and by a search over the path. **Use This Preset**, a double-click or Enter stores the choice. Cancel, or closing the window, leaves the dropdown as it was.

**Storage.** The choice is stored under the same `ScreenFilter` key as `slang:` followed by the preset's path inside the pack, for example `slang:crt/crt-royale.slangp`. It is stored as a relative path so that an updated pack keeps the choice. The dropdown then shows it as `RetroArch: crt-royale`. An unknown name still draws no filter (§4.40).

**The pack.** The picker's **Download Pack** button fetches `shaders_slang.zip` from `buildbot.libretro.com/assets/frontend/`, the file RetroArch's own online updater fetches. Once a pack is there, the button reads **Update Pack**.

- It is unpacked into `home/Shaders/RetroArch/` (`DataStore.Shaders`).
- The server's `Last-Modified` is recorded in `.emusen-pack` beside the presets, and the picker shows it as the pack's build date.
- The zip is written beside its final place and unpacked into a sibling folder. **A failed download, or one holding no presets, leaves the old pack as it was.** A complete one is swapped in.
- .NET's extractor refuses an entry that would land outside the folder.
- The download has its own HTTP client with a thirty-minute timeout, because 54 MB outlasts the thirty seconds a cover lookup is given (§4.39).

**Nothing is fetched unless the player presses the button.** EmuSen ships no shader. The licence position is `EmuSen_Serenity.md` §3.6: a download on the player's request redistributes nothing.

**When a game starts**, and whenever the value changes while that console's game runs, `ApplyScreenFilter` resolves the stored path against the pack and hands the full path to the frame control (`EmuSen_Serenity.md` §7.5).

- If the preset is no longer in the pack, the status bar says so and the picture is drawn plain.
- If the preset is there but cannot be built, for example when no Vulkan device is available or a shader does not compile, the status bar gives the reason and the picture is drawn plain.

**What it does not cover.**
- A preset's parameters cannot be changed from Mistress yet. The preset's own values, or the shaders' defaults, are what is drawn.
- The in-game bar's options menu does not offer presets.
- Presets are shown by path, with no preview.

**Tests** (`ScreenFilterSettingTests`):
- The last entry opens the picker over a pack laid out as libretro's is.
- The search narrows the list.
- The preset used is stored and shown.
- Cancelling leaves the filter.
- A game hands its preset's full path to the frame control, and a missing one is reported and drawn plain.
- The download reads libretro's address and replaces the old pack, leaving no stray files.
- A server error, or a zip with no presets, leaves the old pack.

Two mutants were made, one dropping the no-presets check and one dropping the missing-file check. Each was caught.

### 4.42 Cold start: ReadyToRun and a deferred gamepad (2026-09-22)

**Measured, not estimated.** `~/.cache/emusen/probe/startbench/` runs Mistress's own startup in a fresh process: the same settings loads as `Program.Main`, `App` with its theme, and a real `MainWindow` over a copy of the published sandbox's `home` tree, reading the real ROM library read-only. It runs on Avalonia's headless platform with Skia and prints a timestamp per phase, measured from the process's start. It does **not** include X11's connection or the GPU context, and it runs with the file cache warm, since dropping the cache needs root. Its numbers are therefore a lower bound on a real cold start, and the savings below are real savings of that bound. Online covers were turned off in the copy, so that no run reached the network.

**Where the time went** (sampled with `dotnet-trace`, a ReadyToRun build): the `MainWindow` constructor took 329 ms, and **193 ms of it was `GamepadManager`'s `SDL.InitSubSystem(Gamepad)`**. Timed alone, the time is SDL's joystick layer enumerating input devices (about 150 ms on this machine), not the gamepad mapping or HIDAPI. `SDL_JOYSTICK_HIDAPI=0` saved about 20 ms, and the other hints tried saved nothing.

The rest of the constructor measured small with the file cache warm: the library scan, SQLite open and the list fill together 23 ms, XAML 31 ms, and audio device open 6 ms. A read-through of the startup path had marked the scan, the SQLite open and the audio device as candidates, and on this measurement none is worth moving. On a truly cold disk the scan of a large library may cost more; that is unmeasured.

**Two changes.**
- `PublishReadyToRun` is on for every Mistress publish with a runtime identifier. The run-time cost was measured before and found to be none (`Mars_Performance.md` §25: exact output, 1 to 7 per cent faster), because the JIT still re-tiers hot code and still compiles the recompiler's blocks. The publish grows by about 20 MB.
- The gamepad is constructed unstarted, and `Start` runs from a job posted at background priority by the window's first animation frame. SDL asks for its initialisation on the thread that polls, so it stays on the UI thread rather than moving to a pool thread, but after the first frame is committed instead of before. A pad therefore connects about 150 ms after the window appears. Hotaru still starts its pad at construction.

**Results.** Time to the first render tick, median of ten interleaved runs:

| Build | First render tick |
|---|---|
| JIT, before | 742 ms |
| JIT, gamepad deferred | 587 ms |
| ReadyToRun, before | 488 ms |
| ReadyToRun, gamepad deferred | **329 ms** |

Together the two changes cut the time to the first render tick by 56 per cent.

**A measurement trap, recorded because it hid the result twice.** On the headless platform nothing is drawn until the window is captured, and `CaptureRenderedFrame` pumps the dispatcher before returning. A bench that pumps jobs after `Show`, or that times the capture's return, runs the deferred start before its own frame mark and shows no gain at all. The first render tick can only be read from inside the tick: an animation-frame callback registered after `Show` fires in the same tick as the one that posts the gamepad's start. On a real platform, that tick commits its frame before the posted job runs.

**Not done, and why.**
- A source-generated JSON context for the settings types, and parsing the keybinding and gamepad files once rather than twice. The whole settings phase before Avalonia measured 38 ms.
- Coalescing the per-cover `LibraryGrid.Refresh` after the first frame. That work happens after the window is shown, so it affects responsiveness rather than the start, and it is unmeasured here.

**Test:** `PadNavigationTests.The_gamepad_starts_after_the_window_s_first_frame_and_not_before`. A mutant that starts the pad at construction fails it.
### 4.43 Game Mode is gamescope, not `SteamDeck=1` (2026-09-22)

**The report.** Mistress started from Steam on a Steam Deck showed neither the console sidebar (§4.33) nor the menu
bar, in Game Mode and in Desktop Mode alike.

**The mechanism.** Both are hidden by one branch, `StartPadNavigation` in `MainWindow.Pad.cs`, which big-screen mode
(§4.29) takes: it clears `MenuStrip.IsVisible` and `LibrarySidebarPane.IsVisible`, moves the console choice into the
filter bar's facet and asks for full screen. Nothing else in the window hides either control: no width breakpoint,
no collapse of the pane, nothing tied to full screen. Of the three ways into that branch, the setting and the
`--bigscreen` argument are off in a fresh install (`AppSettings.BigScreen` defaults to false, and the published
`home/etc/EmuSen/appsettings.json` carries `false`). The third was `SteamDeck=1` in the environment. Steam puts that
variable into every game it starts on a Deck, whichever mode the Deck is in, so the branch was taken in both.

**The argument, separated from the mechanism.** In Game Mode the result is the design: §4.29 hides the bar because the
pad's menu holds what a player needs from it and the pad cannot drive the bar's popups (§4.24); §4.33 hides the sidebar because its width is the handheld's
screen. In Desktop Mode none of that holds. The Deck is then a KDE Plasma desktop with a pointer and, often, a
keyboard and a monitor, and the window is one among others. What §4.29 needed was "the session is Game Mode", and it
tested "the machine is a Deck". The two coincide only in the mode the section was written for.

**What decides now.** `MainWindow.InGameModeSession` reads `XDG_CURRENT_DESKTOP`. SteamOS's Game Mode session names
`gamescope` there; Desktop Mode names `KDE`. The variable is a colon-separated list, and any entry equal to
`gamescope` (ignoring case) is Game Mode. Only when the variable is absent or empty does `SteamDeck=1` still decide,
so that a Game Mode session which does not pass the desktop name through keeps its big screen rather than losing it.
The setting and `--bigscreen` are unchanged and still force the mode anywhere, including Desktop Mode.

**Why this does not generalise.** The rule is about SteamOS. Another handheld distribution that runs its game session
under a compositor other than gamescope, or a desktop that names no `XDG_CURRENT_DESKTOP` on a Deck, is not
distinguished by it; the second is taken for Game Mode. A desktop user who wraps Mistress in `gamescope` by hand gets
whatever that nested gamescope sets, which was not examined.

**The retired prediction.** §4.29 said `SteamDeck=1` is set by "a Deck's own session". That was an assumption and
was never checked on a Deck; the report above is the evidence against it, since Desktop Mode had no other way into
the branch.

**Tests** (`LibraryScreenTests`), run with the variables set around the window's construction, since the mode is read
once, there:

- A Steam launch in Desktop Mode (`SteamDeck=1`, `SteamGameId`, `XDG_CURRENT_DESKTOP=KDE`) keeps the sidebar and the
  menu bar and leaves the facet hidden. This case failed on the unmodified build, on the sidebar.
- Game Mode (`XDG_CURRENT_DESKTOP=gamescope`) still hides both and shows the facet.
- A Deck that names no desktop is taken for Game Mode.

Three mutants were made: deciding a named desktop by `SteamDeck=1` again, treating an unnamed desktop as not Game
Mode, and matching `KDE` instead of `gamescope`. Each was caught, the last by two cases.

**What it does not cover.**

- *No Deck was used.* That Game Mode sets `XDG_CURRENT_DESKTOP=gamescope` and that Steam passes it to a game it
  starts, and that Desktop Mode sets `KDE`, are taken from SteamOS's session scripts as documented, not observed here.
- *Game Mode's layout is unchanged.* If the sidebar is wanted there too, that is a change to §4.33's decision, not a
  defect, and it was not made.
- *Big Picture on the desktop.* Steam's full-screen interface in Desktop Mode sets `SteamTenfoot` and
  `SteamGamepadUI`; they are not read, so a game started from it opens in the desktop layout.

### 4.44 The N64's engine: Mars (C#) or MarsRT (Rust) (2026-09-22)

Graphics Settings' N64 tab now begins, after the screen filter, with **Engine**, a dropdown of *Mars (C#)* and
*MarsRT (Rust)*. It is stored in `graphics.json` as `Consoles.N64.Engine`. ~~The default is Mars (C#), so a player who
never opens the row plays exactly as before.~~ *Since 2026-09-24 the default is MarsRT (Rust), listed first: a player
who never opens the row plays on MarsRT, and one who chose Mars (C#) keeps it, since the stored value wins. Every
frontend resolves the row the same way, `CoreCatalog.EngineChosen` (the stored choice, else the row's default). The
factory asked for no engine at all still builds Mars (C#), the reference, so a test that names nothing grades Mars;
a frontend always names one.* ~~And so does every frontend but Mistress: Hotaru, Pharaoh, the probe and
the tests build the C# Mars whatever the file says.~~ *Since 2026-09-22 Hotaru, Pharaoh and the probe read the same
value (`CoreFactory.ConfiguredEngine`) and print the same notice when the engine asked for cannot run; the tests
build what they ask for.* MarsRT is the N64 core in Rust (`Mars_Native.md` §5), exact against Mars in state, picture
and sound; what it does and does not do in a frontend is `Mars_Native.md` §5.5.

**When it takes effect.** At the next load of a game. The value decides which core is built, so it is read by
`LoadGame` before any core exists; a running core cannot become the other engine, and the row's change is not handed
to it. The Expansion Pak row behaves the same way once a game has run a frame.

**Why here and not in the preferences.** The choice is per console, and `graphics.json`'s `Consoles` is the one store
keyed by console; the window builds the row from a catalogue entry (`CoreCatalog.EngineFor`) with the control it
already builds for a choice, so no window was designed for it. The preferences' one previous core picker was removed
in §4.36 because it drove nothing. The engine is **not** one of the core's own settings: those are applied to a
running core between frames, and this one cannot be, so it is declared beside them by the catalogue, as the screen
filter is by the frontend (§4.40), and `ApplyConsoleSettings` never hands it to either core.

**The rows below it.** They are Mars's, and they keep their values whichever engine runs. MarsRT honours the
Expansion Pak, the three thread rows and Skip Repeated Scans, adds a Recompiler row of its own, and ~~ignores the
resolution multiple, antialiasing and the device; each row's hint says which~~ *since 2026-09-23 honours the
resolution multiple, antialiasing and the graphics card as well, exact against Mars at each (`Mars_Native.md` §6.4),
so no row is ignored*. Its defaults are Mars's since
2026-09-22. ~~At the time of writing MarsRT draws on the emulation thread and takes about twice Mars's time per frame
(`Mars_Native.md` §5.7: 13 ms against 6 ms in Super Mario 64 on the development desktop), which is inside a frame on
that machine and was not measured on a slow one; the row is for choosing it, not a recommendation of it.~~ *Retired
2026-09-22: with its threads and recompiler on, MarsRT measures a little faster than Mars on the development
desktop and eight to nine per cent faster on a handheld (`Mars_Native.md` §5.8.8 and §6.1). The default stays Mars
until it has been played on both.* *It was played on the Legion Go S, Donkey Kong 64 at full speed at 2x and 3x on the
graphics card with no crash log (`Mars_Native.md` §6.15.6), and made the default on 2026-09-24 at the player's word,
without the desktop session §6.2 of that page had asked for.*

**When MarsRT cannot run.** If `libmarsrt.so` is missing, speaks another interface, or is turned off with
`EMUSEN_MARS_NATIVE=0`, the game runs on Mars (C#), and the status bar says so after the game's name, quoting what
the library loader found, for example *"MarsRT (Rust) is not available (turned off by EMUSEN_MARS_NATIVE=0); Mars
(C#) is running."* The same line is printed with `[core]`. A value no build knows, such as a hand edit, runs ~~the
default~~ *Mars (C#), the reference,* and says that instead, naming the engine that is running. *Since MarsRT became
the default, a platform without the library shows the "not available" line for every N64 game until Mars (C#) is
chosen in the row.*

**What stays the same across the two.** Save states are one format (a state saved on either loads on the other, and
see below), battery saves are the same `.srm` and `.mpk` files, cheats go
through the same registry and apply under the same rule, and the controller map is the same. A state's record
(§4.37) gives the same core name and state version for both, so it does not say which engine wrote it. ~~**Rewind stays off for
the N64 on either engine** (§4.21b).~~ *Since 2026-09-23 rewind is on for MarsRT and off for Mars (C#) alone (§4.21b,
`Mars_Native.md` §6.6.3), so this is the one thing that differs between the engines in play.*

**What MarsRT does not offer yet.** ~~ROM-patch cheats (no N64 code format makes one, but one added by hand does
nothing on MarsRT)~~ *(applied since 2026-09-23, exact against Mars frame by frame, `Mars_Native.md` §6.6.1; a patch
changed while a frame runs reaches MarsRT's cartridge at the next frame and Mars's at once)*; ~~breakpoints, stepping, watches and coverage in the DianaOS console, which refuses `bp` and `step`
with "cannot be halted by this core" while `regs`, `mem` and `disasm` work;~~ *(the debugger's hooks arrived
2026-09-22, `Mars_Native.md` §6.5)*; ~~internal resolution,
antialiasing, the graphics card, the threaded display processor and deferred presentation~~ *(the threaded display
processor and deferred presentation arrived 2026-09-22, and internal resolution, antialiasing and the graphics card
2026-09-23)*.

**Tests:** `MarsRtEngineTests` drives a real window on the headless platform: the row and its storage; a game on Mars
until MarsRT is chosen and then on MarsRT, with rewind empty; states through the hotkeys that load into the C# Mars;
a battery save read at the start and written at the stop; a `.cht` cheat reaching RDRAM; and the fallback, in a child
process started with the library off, since the library is loaded once per process. `MarsRtFrontendTests` and
`MarsRtSaveFileTests` hold the core's side. The mutants are `Mars_Native.md` §5.5.2. *Since 2026-09-23 the run test
asserts rewind empty on Mars and filling on MarsRT, and `Holding_rewind_on_MarsRT_steps_the_game_back_and_letting_go_plays_it_on`
holds the hotkey; the ROM patches, the phases and rewind have their evidence and mutants in `Mars_Native.md` §6.6.1 to
§6.6.3.*

### 4.45 Every window from the pad, and Game Mode's one window (2026-09-24)

§4.29 made the library and a menu over the game work from a controller, and drove every other window through the keys
a keyboard would type. That left the settings windows only as usable as their tab order, the cheats out of the pad's
reach, and one question unasked: whether a second window appears at all in SteamOS's Game Mode. This section is the
work that closes those, in the order it was done.

#### 4.45.1 A pad with no device behind it

The cases of §4.29 entered at `OnPadCommand`, above the layer that reads the pad, so the mapping from SDL's buttons,
the stick threshold and the wait for release were verified only by hand. `GamepadManager.Simulated` now takes a
`SimulatedPad`, a set of held buttons and axis values; while one is set, every read the manager makes of SDL (a
button, an axis, the first button held, the controller's name) reads it instead, and `Poll` leaves SDL alone. The
seam is inside the manager rather than a second implementation beside it, so everything above it, from
`IsRawPressed` to the window's own `PadTick`, is the code a physical pad runs.

WiseMan's `PadDriver` installs one in a `MainWindow`, stops the window's 16 ms timer and ticks `PadTick` itself, so a
press is one poll down and one poll up, as the shortest real press is. `PadInputPathTests` holds the two things that
had no test: the stick moves the library at 0.6 and not at 0.5 (a mutant lowering the threshold to 0.45 fails it),
and in a game Start alone stays the game's while Back and Start together, or the guide button, open the menu.

What it still does not reach: SDL's own mapping of a device's buttons to `South`, `East` and the rest, and the
120 Hz rate at which a real pad changes, are SDL's and the device's.

#### 4.45.2 Game Mode shows one window, so the others are drawn inside it

**The question.** Every settings window was a second top-level window shown with `Show(owner)`. What SteamOS's Game
Mode does with one was not known, and no device was at hand. The answer below is read from gamescope's source
(`src/steamcompmgr.cpp` on its `master` of 2026-09-23; which build SteamOS stable ships was not established) and from
reports, and is therefore a reading, not an observation.

- Game Mode runs `gamescope -e`, which gives every window of a process the process's Steam app id. A window without
  one is never shown, so a file chooser drawn by another process (a desktop portal) would not appear.
- A second top-level window of the same process **is** shown and takes the keyboard: `pick_primary_focus_and_override`
  follows `WM_TRANSIENT_FOR` from the focused window, which Avalonia sets for an owned window, and a newer mapped
  window wins a tie anyway. But it is shown **instead of** the main window, which is not painted under it, and it is
  scaled to fill the screen, so a 680 by 560 window is enlarged and letterboxed. Closing it returns focus to the main
  window. gamescope draws no title bar and sends no close request, so a window without a close button of its own
  can only be left by quitting the game from Steam.
- A dropdown (an override-redirect window) is drawn above the focused window, and while it extends past a small
  window gamescope rescales the whole picture to fit it in.
- If the player has pinned the main window through Steam's window switcher, transient windows are not followed and a
  settings window would not appear at all.

**The decision.** In a big-screen session, which Game Mode starts (§4.43), windows are presented on sheets inside the
main window (LunaP's `SheetLayer`, `LunaP.md` §90) instead of being shown. The reading above says a window would
appear, so the change is not a workaround for a window that never shows; it is chosen because a sheet avoids every
cost the reading lists: no letterboxed enlargement of a desk-sized window, no rescaling while a dropdown is open, no
dependence on the window switcher's pin, the game still visible around the sheet, and one focus model for the pad to
drive. Desktop Mode, and any session that is not big-screen, keeps real windows, where a pointer and a window
manager make them the better choice. *Since 2026-09-26 (§4.54) the answer follows the mode as it switches: a window
opened in big picture on the desktop is a sheet, and one opened after leaving is a window again.*

**The mechanism.** `MainWindow.axaml` places a `SheetLayer` named `Sheets` over everything, and `StartPadNavigation`
sets its `PresentsWindows` to the big-screen flag, its hint to the pad's buttons, and its scale to the window's
height over 720 points, between 1 and 2. Every window Mistress opens goes through `SheetLayer.Show` (the settings
windows, the cheat windows through their `WindowSlot`s, the screenshot viewer, and the RetroArch preset picker, whose
owner is itself on a sheet), and the two dialogs through `SheetLayer.ShowDialog`: the resume question of §4.31, which
a big-screen session previously answered in a second window at the start of every game with a resume state, and
Browse ROMs. `Dialogs` in LunaP now does the same for the confirmations, so the "different copy of the game" question
(§4.37) lands on a sheet too.

Three rules follow the pad menu's (§4.29). A sheet over a running game pauses it, and the last sheet to close resumes
it only if the sheet was what paused it. The game hears no key while a sheet is up: `SetButtonFromKey` leaves a key
bound to a game button alone while `Sheets.IsPresenting`, since the pad router's synthesised arrows and Enter tunnel
through the main window on their way to the sheet and were otherwise claimed as game input (a mutant that drops the
check fails two cases, on the slider and on the dropdown). And `OtherWindow` answers the sheet on screen before any
real window, so the pad goes to it.

**Tests.** `PadSettingsWindowTests` starts a big-screen window with a synthetic SNES game running and opens each
window from the pad's menu with the pad; each case asserts that no owned window exists, that the game is paused while
the sheet is up and running again after B. `The_resume_question_is_asked_on_a_sheet_and_answered_by_pad` closes a game
from the pad's menu, starts it again, finds the question on a sheet with Resume focused, and answers it with A (the
game resumes) and with B (it does not start).

**What only the device can confirm.** That Game Mode sets the session up as the source says; that the scale chosen
reads well on a Legion Go S's 8-inch 1920 by 1200 panel; and, for Desktop Mode, nothing new.

#### 4.45.3 Focus moves by position, and every control is reached

§4.29 moved the focus with Tab and Shift+Tab. That reaches every control, but in the order the window declared them,
so down on the bindings grid walked along a row (Rebind Key, Clear, Rebind Pad, Clear Pad) before reaching the next,
and there was no way out of a list: down in a list moves its rows, and a list's last row is not a Tab. The router now
moves by position.

**Directions.** Up, down, left and right ask Avalonia 12.1's directional search (`FocusManager.FindNextElement`, its
port of WinUI's XY focus) for the nearest control that way, first inside the innermost scrolling area holding the
focus, then in the ancestors' areas, then in the whole sheet. Searching the area first matters: a control scrolled
below the visible part of a page is further down than the buttons under the page, and a search of the sheet alone
jumped to those buttons and left the rest of the page unreachable (a mutant that drops the area-first pass fails the
bindings case). A scrolling area that is itself the nearest thing is entered at its nearest control, or passed over
if it holds none. The focus change scrolls the area to show the control, which is Avalonia's own behaviour for a
focus given by navigation.

When Avalonia's search answers nothing, the router takes the nearest control that way at all, scored by the distance
that way plus twice the distance across. The search answers nothing for a control that lies below and wholly to one
side of the focus with every strategy it offers (projection, rectilinear distance, direction distance) and with
occlusion ignored, which is how Preferences' Close button, right-aligned under left-aligned switches, was unreachable
in Desktop Mode though reachable on a sheet, where the switches are wider. Found by the Desktop Mode audit below; a
mutant without the fallback fails it.

Controls that take left and right keep them: a closed dropdown steps its choice (and saves it, as choosing does), a
slider moves by its small change. Up and down in a list move its rows until the first or last, then leave it.

**Tabs.** The shoulders change tab and put the focus on the new page's top control. The tab strip is one stop for
the d-pad: moving up into it lands on the selected tab, never another, because a tab header that receives the focus
by navigation selects its tab, and the first version of this router switched tabs whenever up happened to be nearer
another header. Left and right on the strip change tab. Down from the strip goes into the page. That last rule is a
workaround: Avalonia 12.1's search, asked for the control below a tab header, answers the header beside it on the same
row, and alternates between two headers for as long as it is asked.

**Dropdowns.** Accept opens one; up and down then move the highlight, which moves the focus and not the choice;
accept chooses; back closes without choosing. Previously up and down set the choice at every step, so passing over
"RetroArch Preset..." on the way to another entry opened the preset picker.

**Lists.** A row given the focus by navigation is selected by it, which is Avalonia's own rule and was measured
rather than assumed: a line that also selected the row on accept changed no test's outcome when removed, and was
removed. Accept then sends Enter to the row, which is how a list that acts on a row (the cheat database's games, the
RetroArch preset list) is told to. Accept on a row of a `LunaTable` whose row holds a checkbox ticks it instead,
through the checkbox, so the table's own write-through runs (the cheat list, §4.45.5).

**The audit.** WiseMan's `PadAudit.Reachable` presses every direction from every control reached, starting where the
sheet puts the focus, on every tab, and lists the controls a player could operate (buttons, switches, dropdowns,
sliders, text boxes, list rows and the selected tab) that no press reached. It found, before the fixes above: in
Preferences, the Close button, below the edge of a sheet shorter than the window, because the window stacked its tabs
over a Close button with no scrolling (the panes now scroll, and the buttons are docked); on System Files, which has
nothing to focus, no way down from the tab strip at all; and in every tabbed window, a walk that changed the selected
tab. `PadAudit.Reach` finds a pad path to a named control and walks it with the pad alone, which is how the operating
cases below get to each control.

**Desktop Mode.** Real windows are driven by the same router. The headless platform never makes a window active,
so `OtherWindow` cannot find one there, and `In_desktop_mode_a_real_window_is_driven_and_every_control_reached` hands
the router the window directly and audits Debug Logging (the one settings window only the menu bar opens), Graphics,
Active Cheats and Preferences. That the pad reaches the active window on a desktop is the code of §4.29 and was not
changed; it is verified by hand only.

**The RetroArch preset picker**, opened from a screen-filter dropdown on the graphics sheet, goes on a sheet over it
and comes back to it. The audit reached everything in it, and choosing a preset with A did nothing: the list had a
plain `KeyDown` handler for Enter, and the list claims Enter before a plain handler hears it, so Enter from a real
keyboard never used a preset either. It listens to handled events now, as the cheat database's games list does for
the same reason. `The_RetroArch_preset_picker_opens_over_the_graphics_sheet_and_a_preset_is_chosen_by_pad` failed
before the change.

**What each window's case operates.** Graphics: a dropdown stepped left and right with the stored value checked, opened,
moved and chosen, opened, moved and backed out of; a switch; Reset This Console; the tabs. Preferences: a switch and a
dropdown on another tab. Controller Bindings: the slider, a switch, a pad rebind, and a key rebind backed out of
(§4.45.4). Each leaves with B.

**Mutants.** Five were made against the router and the window: dropping the tab-strip rule (four cases fail),
dropping the scrolling-area search (one), the game keeping its keys under a sheet (two), a sheet not pausing (three),
and never letting a list go at its edge, **which survived**: no window in this section has a list. It is caught by
three of §4.45.5's cases, where the cheat list and the database's lists stand between the code boxes and the buttons.

**What it does not cover.** The system file pickers (Browse... beside a path, Save As... and Load From... in the
cheat window) are reached and pressed but open the platform's dialog, which in Game Mode is drawn by another process
and, on the reading of §4.45.2, never shown. A path can be typed instead (§4.45.6).

#### 4.45.4 Rebinding from the pad

Two defects made the Controller Bindings window unusable with a pad alone, and neither was visible with a mouse.

- **Rebind Pad bound the button that pressed it.** The capture polls for the first held button every 50 ms, and the
  A that chose Rebind Pad was still held at the first poll, so it was bound. The capture now waits for every button to
  be let go before it listens, and after binding waits again until the bound button is let go before the pad is the
  interface's, or binding A would press Rebind Pad a second time. A capture no one answers gives up after five
  seconds, because every button it could be cancelled with is a button it could bind.
- **Rebind Key heard the pad.** The router synthesises keys, and the capture's tunnel handler took the first one (the
  Tab of a d-pad press, in §4.29's router) as the binding. The window now says what it is capturing through
  `IPadCapturing`; the router stands aside entirely while a pad button is captured, and while a key is captured
  passes only B, which cancels it.

The key capture's handler was on the window, and a window on a sheet does not see keys typed on its content
(`LunaP.md` §90.6), so it moved to the window's content. Tests: the rebind case holds A through the first poll and
asserts nothing was bound (a mutant that listens at once fails it), binds North, and checks the pad stays the
window's until North is let go; `A_pad_capture_nobody_answers_gives_up` holds the timeout.

#### 4.45.5 Cheats from the pad, during play

**The route.** The pad's menu has a Cheats entry, over a game and in the library, which opens Active Cheats (§4.14) the
way the menu bar does. It opens on the running game's console tab: both cheat windows were handed the library's console
filter, so with the filter on all consoles a running SNES game's cheats window opened on General, and the running
game's codecs (§4.14's "a running game's codecs beat the catalog's") reached no tab. `CheatConsole` is now the running
game's console when there is one, and the filter otherwise; this is a change for Desktop Mode too, and a correction.

Active Cheats gained a **Cheat Database...** button beside Apply, the mirror of the database's Active Cheats button,
since with the menu bar hidden it was the only way to the database. The database opens on a sheet over the cheats and
B comes back to them.

**The flow, as the test drives it.** On the SNES tab the pad reaches the code box, and A opens the on-screen keyboard
(§4.45.6) with the hexadecimal layout; `7E010042` is typed with the d-pad and A, and Start puts the keyboard away. The
description box opens it with letters. Add adds the cheat, enabled. A on the cheat's row ticks it off and on through
the table's checkbox. Apply Cheats, with the game paused under the sheet, pokes at once (§4.15's paused case), and the
test reads `0x42` at `$7E:0100` through the running core's debug target. B closes the sheet and the game runs again.
A second case loads a game's cheats from a two-game database folder with the pad alone: the system row, the game row,
A to load (the games list now loads on Enter as it does on a double click), B back to the cheats, A to tick one on.

**Code layouts per console.** The code box offers the layout the console's codes are written in first: the sixteen
Game Genie letters for the NES, hexadecimal for the others (a Game Boy Game Genie or GameShark code, an SNES Game Genie
or Pro Action Replay code and an N64 GameShark code are all hexadecimal with separators), then the other layouts
behind the Next key.

**Tests.** `PadCheatsTests`, five cases: the flow above; the database from the cheats and back; the database loaded by
pad; and the reachability audit of both cheat sheets with two cheats in the list. Mutants: a table row that A does not
tick (two cases fail), the keyboard not taking the pad (two), and the list edge of §4.45.3 (three).

**What it does not cover.** Save As... and Load From... open the platform's file dialog (§4.45.3). Download from
libretro works from the pad but its progress and failure are only shown as status text, unchanged.

#### 4.45.6 Text from a pad: a keyboard of Mistress's own

**The decision, and the one it replaces.** §4.30 asked Steam for its keyboard through `steam://open/keyboard` and argued
that a keyboard of Mistress's own would be a second keyboard, laid out differently, on a machine that already had one.
That argument was about prose. What a pad user is asked to type here is mostly a cheat code, sixteen hexadecimal digits
and a separator, on which Steam's QWERTY layout puts every digit three or four moves away and the digits on a separate
page. The URL's reliability was also the claim §4.30 said to check first on a device, and it was never checked; the
evidence found since (§4.45.2's reading) is indirect: SDL's X11 backend opens the keyboard the same way under Steam,
and an osu! report says it popped up in Game Mode, while a 2025 forum report says Steam's own keyboard chord stopped
working there. A keyboard drawn in the window has none of those unknowns, and it is testable: the flow in §4.45.5 types
through it with the pad.

So a pad now opens LunaP's `OnScreenKeyboard` (`LunaP.md` §91) on any text box: A on a focused box in any window or
sheet, and Y in the library, which focuses the search box. Steam's keyboard is still the player's, through Steam's own
chord (the Steam button and X), and types into a focused box as it always did; Mistress just no longer asks for it.
`SteamKeyboard` and its two tests were removed; the two cases that used it now assert the on-screen keyboard.

**The buttons on the keyboard** (`PadKeyboard.Send`): the d-pad moves the highlight, A types the key, B erases the
character before the caret and, with nothing left to erase, puts the keyboard away, Y types a space, Select shifts,
the shoulders change the layout, and Start is Done. Done keeps the text; the keyboard's own Escape (from a real
keyboard) puts back the text the box had. The hint line under the keys says all of this.

**Paths.** The path rows in Preferences and the cheat database's folder are now editable (`PathPickerRow.IsEditable`,
`LunaP.md` §91.4), so a folder can be typed with the keyboard, committed when the focus leaves the box, where
Browse... opens a picker Game Mode may never show.

**What only the device can confirm.** That the keys are big enough to read on a handheld at arm's length (they are 44
points, unscaled by the sheet), and that nothing in Game Mode steals the pad's buttons from the keyboard.

#### 4.45.7 What is left, and what only the device can confirm

**Windows not made pad-operable, on purpose.** The DianaOS console is a shell and wants a keyboard; the hardware and
runtime dashboards show numbers and have nothing to operate; Browse ROMs, the menu bar's second library, is reached
only from the menu bar, which big-screen mode hides, and the library itself is the pad's. All four are still reached
from the menu bar in Desktop Mode. The online cover window and the collection prompts were not audited.

**Gaps that remain.** The platform file dialog (Browse..., Save As..., Load From...), in any mode. The menu bar's
popups in Desktop Mode (§4.29). A key rebind from the pad is refused rather than possible, since the pad has no keys.
The pad menu resumes the game for the moment between closing itself and a sheet opening over it.

**Only the device can confirm:**

- that SteamOS's shipping gamescope treats a second window as its source does (§4.45.2); the sheets make the answer
  matter less, and the one window that is still a second window in Game Mode is the platform's file dialog;
- that `XDG_CURRENT_DESKTOP=gamescope` reaches a game started from Game Mode, which decides big-screen mode (§4.43)
  and therefore whether windows become sheets;
- that the sheet's scale (the window's height over 720 points) and the keyboard's 44-point keys read well on a Legion
  Go S's 8-inch panel at arm's length;
- that SDL maps the Legion Go S's buttons to South, East and the rest as a Deck's are mapped, which everything above
  reads through;
- that Steam's own chord for its keyboard still types into a focused box, for a player who prefers it.

**What the device confirmed (2026-09-24, Legion Go S, SteamOS 3.8.27, build 78bf8ea).** Mistress started from Game
Mode carries `XDG_CURRENT_DESKTOP=gamescope` and `XDG_SESSION_DESKTOP=gamescope` (read from its `/proc` environment),
so big-screen mode and the sheets are on; gamescope runs the session at 1280×800, a sheet scale of 800/720 ≈ 1.11. The
player reported every control of the settings windows and the cheat flow reachable and usable by pad, which also
answers the button-mapping question for the South/East grammar these read through. Still unconfirmed: the second-window
behaviour of the shipping gamescope, which no sheet now exercises except the file dialog, and Steam's own keyboard
chord.

#### 4.45.8 Dropdown lists drawn inside the window

**The defect.** The same day, on the same device: every dropdown on a sheet (the screen filter, the core version)
opened its list at the size of the screen. On Linux, Avalonia 12.1 opens a dropdown's list, like a menu or a tooltip,
as a top-level window of its own, and gamescope scales each new top-level window to fill the screen. The sheets of
§4.45.2 moved windows inside the main one and left popups where they were; the gamescope reading behind §4.45.2 had
said a window is rescaled when a dropdown sticks out past it, not that the dropdown is itself a window.

**The fix.** `Program.BuildAvaloniaApp` calls LunaP's `EmbedPopups` (LunaP §92) with the same decision that turns on
big-screen mode (`MainWindow.WantsBigScreen`: the setting, `--bigscreen`, or a Game Mode session, §4.43). It binds
Avalonia's `X11PlatformOptions.OverlayPopups`, which draws a popup in the window's overlay layer instead of in a
window. The decision is made before the first window exists, from `AppSettings.Load()` and the environment, because
platform options are read once at startup: turning the big-screen setting on or off takes effect at the next launch.
Desktop Mode is unchanged.
*Amended 2026-09-26 (§4.54):* the setting no longer takes part. A start the setting put in big picture can now be left
for the desktop, and a process-wide option cannot follow a mode the window switches, so
`MainWindow.EmbedsPopupsAtStart` asks only `--bigscreen` and a Game Mode session, the two answers that hold for the
process's whole life. Big picture entered or started on the desktop embeds the window's own popups instead, with
LunaP's `EmbeddedPopups` on the window (§92.5 there), which a switch turns on and off.

**What it gives up.** An embedded list cannot reach past the window's edge, which in a full-screen session has
nothing beyond it.

**Evidence.** LunaP's `BootstrapTests` shows the option is bound when asked for and not otherwise, and a mutant that
binds nothing is caught. That the list is now drawn at its own size under gamescope is for the device to confirm; the
player confirmed it the same day.

#### 4.45.9 The highlight in an open list, and the page behind it

**What the player saw.** With the list drawn in the window (§4.45.8), up and down in an open dropdown moved no
visible highlight, the page behind the list moved instead, and backing out showed the choice had been changed.

**What was reproduced, and what was not.** A headless run (whose platform draws every popup in the window's
overlay layer, as the device now does - LunaP §92.6) showed the focus moving from item to item as it should and the blue staying on the
item selected before the list opened: the moved focus was marked only by FluentTheme's one-pixel ring. That is fixed
in LunaP (§93): while a list is open the accent follows the focus. The page moving was **not** reproduced
headlessly: the page's scroll offset did not change. The router used to move the focus by sending an arrow key to
the focused item, and a key that nothing marks handled rises through the dropdown to the page's scroll viewer. It
now moves the focus among the open list's items itself (`PadWindowRouter.Highlight`) and chooses with A by setting
the selection (`Choose`), so no key is sent while a list is open, whichever way the platform routes one. Whether
that was the page's cause is for the device to confirm.

**Sheets ask for it themselves.** LunaP's `SheetLayer` now sets `EmbeddedPopups` on every sheet (LunaP §92.5),
which puts `Popup.ShouldUseOverlayLayer` on every list under it, so a sheet's dropdowns stay in the window on any
platform, and independently of the startup option of §4.45.8.

**Tests.** `PadSettingsWindowTests.An_open_dropdown_drawn_in_the_window_moves_its_highlight_and_not_the_page`
checks that the sheet's list asks for the overlay layer, and checks the focus on each item in turn, the selection unchanged by moving or by
B, the scroll offset unchanged throughout, and A choosing and storing the highlighted item. A mutant that leaves the
highlight where it is and one that makes A choose nothing are both caught. The old key-sending path passes this test
too, since the headless platform does not move the page, so the test does not show the page defect gone.

### 4.46 Game Boy and Game Boy Color on separate shelves (2026-09-24)

**What changed.** The library's console filter (the facet in big-screen mode, the sidebar's Consoles group on the
desktop) listed one entry per core, so every Game Boy and Game Boy Color game shared *Game Boy (Mercury)*. There is
now a second entry directly after it, *Game Boy Color (Mercury)*, labelled **GBC** in the sidebar, with its own count.
The player asked for it on 2026-09-24.

**What decides the shelf.** A `.gbc` file, or a `.gb` file whose header byte at `0x143` is `0x80` (Color-enhanced) or
`0xC0` (Color only), is a Game Boy Color game; every other Game Boy file is a Game Boy game. These are the two values
Mercury itself reads to decide the model (`Cartridge.cs`), so the shelf and the machine agree. The extension alone
would not do: on the development desktop all 1,915 Game Boy files end in `.gb` and 17 of them carry the Color flag,
while the handheld's library keeps 426 `.gbc` files in a folder of their own. A `.gb` file costs one byte read per
scan, cached by path for the process's life; a file too short to have a header, or one that cannot be read, is a
Game Boy game.

**What is a shelf and what is a core.** The shelf is the library's word (`CoreCatalog.LibraryShelf`,
`ShelvesInReleaseOrder`, `ShelfFor`); the core is still one. A game's `CoreDisplayName` stays *Game Boy (Mercury)*
on both shelves, so artwork, covers, the database lookup and the core that loads are unchanged; its `Shelf` is what
the filter, the sidebar's counts and the list's console tag use. The stored filter (`appsettings.json`'s
`SelectedCore`) may now name the Color shelf; the two cheat windows, which know cores, are handed Mercury for it.
The Color shelf sits after the Game Boy rather than in release order (1998, after the N64), because the player asked
for the two together.

**Tests.** `RomLibraryTests.Game_Boy_Color_games_sit_on_their_own_shelf_by_extension_or_header` (both header values,
the extension, a plain `.gb`, a header-less file, the core name unchanged, narrowing by scan and by filter) and
`LibraryScreenTests.Game_Boy_Color_games_have_their_own_row_in_the_sidebar` (the GBC row after GB with its count,
choosing it, the stored filter), with the sidebar rendered and looked at. A mutant that ignores the header byte is
caught by both.

**Not the same as the model setting.** Which machine a Game Boy game *runs on* (a Game Boy or a Game Boy Color, the
choice being built separately) does not move it between shelves: the shelf is the cartridge's, not the player's
setting.

### 4.47 The Game Boy's Model: Auto, Game Boy or Game Boy Color (2026-09-24)

**What the player sees.** The GB tab of Graphics Settings has a **Model** row under Engine: *Auto* (the default),
*Game Boy* and *Game Boy Color*. The row is an ordinary core-setting dropdown. So the pad cycles it with Left and
Right, and the mouse opens it (§4.45).

- *Auto* is the old behaviour: the cartridge's header decides.
- *Game Boy* runs every game on a Game Boy, a colour game in its own monochrome mode.
- *Game Boy Color* runs every game on a Game Boy Color, a Game Boy game in the colours the Color's start-up picks for
  it.

The choice is stored in `graphics.json` under `Consoles.GB`, key `Model`, and it takes effect when a game is next
loaded, as the Engine row's choice does (§4.44).

**Why it is a core setting and not a frontend switch.** The model is Mercury's `ICoreSettings` setting, declared once
as `MercuryCore.ModelSettings` and returned by `CoreCatalog.SettingsFor("GB")`, as the N64 declares its video settings.
Both engines honour it:

- `MercuryCore` and `MercuryRtCore` each read it at load;
- a change before a game's first frame loads the game again at once, as the Expansion Pak row does.

Mistress does nothing model-specific. It hands the core its console's settings after the load and before the first
frame (`ApplyConsoleSettings`), and the core rebuilds itself if the stored model differs.

**One change to Mistress that the row needed.** `_activeConsole` names the console whose settings, pad bindings,
screen filter and cheats apply. It was set from the core's `CoreName`, which a Game Boy Color game reports as `GBC`.
No tab and no binding set is named `GBC`, so the GB tab's settings never reached colour games. A Model choice of
*Game Boy* would have been ignored for exactly the games it matters to.

`_activeConsole` is now the catalogue's console for the ROM (`CoreCatalog.ConsoleForRom`, `GB` for every Game Boy
game). `CoreName` still says which machine is running. The same change gives colour games the GB tab's screen filter,
the GB pad bindings and the Game Boy cheat console, which they did not get before. That is intended, and nothing
depended on the old behaviour: no binding set or graphics key is named `GBC` (the `GBC` library shelf, §4.46, is a shelf and not a console).

**States, saves and the shelf.**

- A save state records the console it was made on, and resumes on it whatever the row now says (Mercury state
  version 7, `Mercury_Model.md` §5).
- Battery saves and state files are keyed by the ROM's path, so they do not move when the model changes.
- The library shelf is the cartridge's (§4.46), so it does not move either.

**Tests.** `MercuryModelSettingTests` covers two things:

- the row's three choices, its default and a stored choice, with the window rendered to PNG and looked at;
- a game loaded in Mistress on the chosen console, on both engines: a Game Boy game on *Game Boy Color*, and a `$80`
  game on *Game Boy*.

The machine-level behaviour of each choice is `Mercury_Model.md` §6.

### 4.48 Shaders: a window of their own, tabbed by console (2026-09-24)

**What was asked.** The player wanted a settings entry for shaders alone, tabbed by console as Graphics Settings is,
where a shader could be chosen and configured, and said that RetroArch's names could not be read whole and the list
could not be filtered. Before this, a console's shader was the last row of Graphics Settings' tab (§4.40), and
RetroArch's presets were a picker of 2,658 relative paths (§4.41), cut at the window's edge, searched as one
substring, with no way to change a parameter.

#### 4.48.1 The window, and one list of every shader

**Where it opens.** Settings ▸ **Shaders...**, the pad menu's **Shaders**, the in-game bar's Options ▸ **Shaders...**,
and the **Shaders...** button on each of Graphics Settings' tabs (§4.48.4). It is `ShaderSettingsWindow`, a LunaP
`ToolWindow`, drawn on a sheet in Game Mode like every other window (§4.45.2). Its tabs are
`CoreCatalog.ConsolesInReleaseOrder`, the same tabs in the same order as Graphics Settings, and it opens on the running
game's console, or on the console of the Graphics tab its button was pressed on.

**One list.** Each tab's left half is a `GroupedList` (`LunaP.md` §94.1) holding the built-in filters and the pack's
presets together (`ShaderCatalog`):

- **Built into EmuSen** comes first: *None*, then `ScreenFilters.NamesFor(console)`, so a console is offered the
  filters that suit it, as §4.40's dropdown was.
- **Each preset sits under its folder** in the pack, the group a heading drawn on the group's first row, with the
  folder's levels joined by " / " and each level made readable: `bezel/Mega_Bezel/Presets/Base_CRT_Presets` is
  *bezel / Mega Bezel / Presets / Base CRT Presets*. A heading counts its group's rows.
- **A name is the file's name made readable**: the extension dropped, underscores as spaces, runs of spaces as one, so
  `MBZ__0__SMOOTH-ADV__GDV.slangp` reads *MBZ 0 SMOOTH-ADV GDV*. Hyphens are kept, because they are how the pack's
  authors write names (`crt-royale-kurozumi`) and how a player will type them. Names and headings wrap; nothing in the
  list is cut.
- **The shader in use is marked** with an *In use* pill, and the list opens with it selected and scrolled into view.
- **The selected shader's whole path is always shown**, above its sliders, under its name: the path inside the pack
  and, on the next line, the pack's folder on this machine, in the monospace face, wrapping. A built-in shows its
  credit (`ScreenFilter.Credit`) instead.

**Search and category.** The search box above the list matches **every word** typed, in any case, against a row's
name, its folder heading and its relative path (`FilterBar.MatchesWords`, `LunaP.md` §94.3), so "royale kuro",
"mega gdv" and "handheld/lcd" each find what a person means. The **Category** dropdown beside it narrows to *All*,
*Built into EmuSen* or one of the pack's top-level folders. **None stays at the top whatever is searched**, so going
back to no shader never needs the search cleared.

**Choosing is two steps.** Moving onto a row shows that shader and its sliders; **Use This Shader**, Enter, A on the
pad, or a double click applies it. A pad's highlight selects every row it passes (§4.45.3), and applying on selection
would build every RetroArch preset passed over on the way down the list (a chain compiles each pass, `EmuSen_Serenity.md`
§7.4); showing costs a read of the preset's sources and nothing on the device. The button reads **In Use**, disabled,
for the shader already applied.

**Recents, not favourites: the decision.** The last five shaders used on a console head its list as *Recently used*,
newest first, with the folder as a second line, while nothing is searched and the category is *All*; a search finds
them in their own folders anyway. Favourites were not built. A favourite needs a control on each row, and a
`GroupedList` row is not interactive (its heading rides on the row, §94.1); on a pad it would be one more button in
the one place where A already means "use". Recents cost the player nothing, and in practice they are the short list a
favourites feature would be: few people alternate between more than a handful of shaders per console. If that
judgement proves wrong, a favourite is a second list beside `RecentShaders` in the same file, and the group already
exists.

**Without the pack.** The list is then only the built-ins, and directly under them, where the presets would be, is a
box headed *RetroArch's shaders* with what the pack is (libretro's `shaders_slang.zip`, about 54 MB, the file
RetroArch's updater fetches, none of it shipped) and **Download Pack** (`SlangPackDownload`, unchanged from §4.41).
One download fills every tab. With a pack, the box is one line at the list's foot, the preset count and the pack's
build date, and **Update Pack**.

#### 4.48.2 Configuring the shader shown

**A slider per parameter** (`SliderRow`, `LunaP.md` §94.2) in the right half, under the shader's name: the parameter's
description as its label (its id when it has none), its declared minimum, maximum and step, its value, its default
beside the value while the two differ, and a **Reset** button above the slider's right end. **Reset All** beside Use
returns every slider of that shader on that console to its default, and is enabled only while something differs.

- **A RetroArch preset's parameters** are `SlangParameters.Read` (`EmuSen_Serenity.md` §7.6): every pass's
  `#pragma parameter`, the first declaration of an id winning, and **the preset's own values as the defaults**, so a
  reset returns to what the preset's author chose rather than to the bare shader's number.
- **A built-in filter's** are its `ScreenFilter.Parameters` (`EmuSen_Serenity.md` §3.7): eleven for CRT (Lottes), the
  response and dot shadow for the LCDs. Scanlines and Simple CRT have none, and say so.
- **A parameter whose range has no width is a heading** (`SlangParameters.IsHeading`), which is how presets label
  groups in RetroArch's menu; it is drawn as a section heading, and one that is only decoration (`---`, `===`) is left
  out.

**Reading off the UI thread.** A preset's sources are read on a worker and the sliders built when they arrive, so a
large preset never stalls the list; a row passed over before its read finishes does not build sliders for it. The
read is cached per path for the window's life and cleared by a download. Measured on the development desktop with the
pack of 2026-09-22 (`A_pack_preset_with_hundreds_of_parameters_is_shown_in_measured_time`, with `EMUSEN_SLANG_PACK`):
`crt-royale`, 46 sliders, read and built in 288 ms; `bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV`, 944 sliders (953
parameters less nine headings), in **1.9 s**, nearly all of it building 944 rows on the UI thread, since the read
alone is 61 ms (§7.6). That second number is the cost of not virtualising the sliders, and it is recorded in §4.48.7.

**Numbers shown as they were written.** A parameter's numbers are `float`s, and 0.041f widened to a double is
0.041000001, which the row showed to four places (0.0410). The window widens through `decimal`, which gives the
seven significant digits the float was written with, so the row shows 0.041. Found in the first rendered picture.

#### 4.48.3 Where the values are kept, and how they reach a running game

**Storage.** `graphics.json` (`EmuSen_Config_Reference.md` §3.3) gains two maps beside `Consoles`. The shader is keyed
by exactly the value its console's `ScreenFilter` would hold, so a built-in is its name and a preset is `slang:` and
its path in the pack:

```json
"ShaderParameters": {
  "SNES": {
    "CRT (Lottes)": { "maskDark": "0.3", "brightBoost": "1.05" },
    "slang:crt/crt-royale.slangp": { "crtgamma": "2.2" }
  },
  "GB": { "Game Boy LCD": { "shadowOpacity": "0" } }
},
"RecentShaders": {
  "SNES": [ "slang:crt/crt-royale.slangp", "CRT (Lottes)" ]
}
```

- **Per console and per shader**, as asked: the same preset may be tuned differently for the NES and the SNES, and a
  console's values for one shader survive a switch to another and back.
- **Only values that differ from the default are written.** A slider moved back to its default, a Reset, or Reset All
  removes the entry, and a shader or console left with nothing is removed with it, so the file holds exactly the
  player's changes.
- **Values are text**, written with the invariant culture, as `Consoles`' are (§3.3's reason: no schema per shader, and
  a hand edit that does not parse is skipped rather than failing the load).
- `RecentShaders` holds at most five per console, newest first; *None* is never recorded.
- **Graphics Settings' Reset This Console** clears the console's `ScreenFilter` with its core settings (§4.40) and
  leaves its shader parameters and recents: those belong to the Shaders window, which has its own resets. The
  accessors are `ParametersFor`, `SetParameter`, `ForgetParameter`, `RecentFor` and `NoteRecent`.

**Live changes.** A choice or a slider saves at once and calls the frontend's `ShaderChanged(console)`, which does
nothing unless a game of that console is running, and then runs `ApplyScreenFilter` on the UI thread. That reads the
stored shader, parses its stored values to floats (skipping any that do not parse), and sets
`GameFrameControl.ShaderParameters` **before** `ActiveFilter` and `ActiveSlangPreset`, so that a chain or runner built
for a newly chosen shader starts from the player's values. For a slider, the shader has not changed, and both setters
return at once (the same filter reference, the same preset path), so nothing is rebuilt: the frame control hands the
values to the running `FilterChain` or `SlangRunner`, which use them at the next draw, and a runner draws again even
when the game is paused (`EmuSen_Serenity.md` §7.6). A slider for a shader that is not the one in use, or on another
console's tab, is saved and reaches nothing until that shader is used.

#### 4.48.4 Graphics Settings' row

The **Screen Filter** dropdown and its **RetroArch Preset...** entry are gone from `GraphicsSettingsWindow`. Each tab's
first row is now **Shader**: the current shader's name (*CRT (Lottes)*, or *crt-royale (RetroArch, crt)*) and a
**Shaders...** button that opens the Shaders window on the same console, on a sheet over Graphics Settings; the name
follows the choice when it closes. `ScreenFilterKey` and the `slang:` format are unchanged, so every existing
`graphics.json` means what it meant. `SlangPresetWindow` is retired, superseded by this window, along with the
dropdown's `ChooseRetroArch` and `SlangLabel`. A shader change made from inside Graphics Settings calls
`ShadersChanged` rather than the window's own callback, so moving a slider does not re-apply the core's settings on the
emulation thread.

#### 4.48.5 From the pad, and on a Game Mode sheet

Everything is operated with the pad (`PadWindowRouter`, §4.45.3): the shoulders change tab; A on the search box opens
the on-screen keyboard (§4.45.6) and Start puts it away with the list already narrowed; up and down walk the list,
showing each shader; A uses one; right goes to the right half; left and right move a slider by its step; up from a
slider reaches its own Reset; Reset All and Use sit above the sliders; B closes. At 1280×800 the sheet is scaled by
800/720 (§4.45.2) and the text is legible in the rendered pictures.

**A defect the pad found.** Using a shader refreshes the list, to move the *In use* pill, and a refreshed
`GroupedList` replaced every row's container, the focused one with it, which left the focus on nothing: the next
d-pad press did nothing at all. With a mouse the defect is invisible. `GroupedList.Refresh` now puts the focus back on
the row it keeps selected, or on the first row, unselected, when none is kept (`LunaP.md` §94.5). Before the rule the
pad-menu case stopped with no focused control after A on a row (observed); a mutant without it fails LunaP's case.

**A second, of the same kind.** **Use This Shader** is disabled once it has been pressed (it reads *In Use*), and
**Reset All** once nothing differs from the default; a button disabled under the focus loses it, and the pad was dead
again. Whichever of the two was pressed now hands the focus on, to the other button if it is enabled, else to the
first slider, else to the list's selected row (`FocusNearby`). Found by the pad-menu case, whose next walk had no
starting point; a mutant that never hands it on fails that case.

**Where the two buttons sit.** They are right-aligned above the column of Reset buttons, Reset All then Use, so that
from a row a pad reaches Use in three presses (right to the first slider, up to its Reset, up again, measured on a
Game Boy LCD row). They were first at the left under the shader's name, and the audit then reported the Game Boy
tab's Use unreachable; **that report came from the audit, not from the layout** (it was made by the version that did
not yet put scrolling back, below): with the audit corrected, the left placement is reachable too (the mutant that puts it back survives, §4.48.6). The right
placement is kept for the shorter path, which is a judgement, not a measured need.

**The audit had to change.** `PadAudit` walked breadth first, remembering each control it reached by instance and
later focusing it directly to press every direction from it. This window broke that in two ways:

- **The right half is rebuilt by the list's selection**, and a row given the focus is selected by it, so a slider
  reached from one row was gone once the walk had visited another; and a list is entered at its selected row, so the
  same press from the same place led elsewhere once the walk had moved the selection. The Game Boy tab's **Update
  Pack** was reported unreachable, though a walk down the list with the pad reaches it (checked row by row). The walk
  now reaches every control it explores **by replaying the path it was found by**, from the starting control, with
  every list's selection and every scrolling area's offset put back as they were, never by focusing it directly.
  This is slower (the path is replayed before each press) and exact: the state a control is explored in is the state
  it was found in. The first version restored only the lists and lost the walk down the sliders once their column had
  scrolled, since a move is by position. Controls other than rows are known by where they sit in the tree, so a
  pane rebuilt the same way is the same controls.
- **A virtualised list recycles its row containers as it scrolls**: walking 60 rows down the real pack's list used
  13 containers, 47 of them reused for a later row, while the Game Boy tab's 11 rows reused none. A row is therefore
  known by its list and index, not by the container drawing it. **This was not the cause of the Update Pack report**
  (no container was reused there), and no audit case today has a list long enough to need it: the mutant that keys
  rows by container survives (§4.48.6). It is kept because a longer list in any audited window would make the walk
  treat a row it has not explored as seen.

`Every_control_of_each_settings_sheet_is_reached_by_the_pad("Shaders")` audits the sheet and
`In_desktop_mode_a_real_window_is_driven_and_every_control_reached("ShowShaderSettings")` the desktop window; the
four existing windows' audits and the cheat sheets' pass under the new walk unchanged.

#### 4.48.6 Tests, pictures and mutants

**Tests.** `ShaderSettingsWindowTests` (10): the built-ins, None first, and the one in use selected on every tab;
Use storing for one console and telling the frontend, by Enter; grouping, readable names, word-by-word search over
name, folder and path, the category dropdown, and the wrapping path; recents newest first and out of the way of a
search; the download filling every tab; a built-in's sliders stored per console and per shader, reset one and all;
a preset's own values as defaults and its headings; **a slider moved while a game runs reaching the `FilterChain` the
frame control draws with** (the chain's own value read back before and after, and another console's slider leaving it
alone); the desktop pictures; the large-preset timing. `ScreenFilterSettingTests` now drives the Shaders window for
§4.40's cases and checks Graphics Settings' row on every tab. `PadSettingsWindowTests`: the window from the Graphics
sheet searched with the on-screen keyboard, a preset used and a slider moved and reset by pad; the window from the pad
menu, CRT (Lottes) used, a slider moved with the value reaching the frame control, and Reset All; and both audits.

**Pictures** (`EMUSEN_UI_DUMP`, looked at): `shaders-desktop-preset`, `-lottes`, `-bezel`, `-search` and `-nopack`
at the window's own 980×660 against the real pack, and `pad-shaders-search`, `pad-shaders-slider` and
`pad-shaders-lottes` on a 1280×800 Game Mode sheet.

**The cost of drawing a filter headlessly.** The first version of the running-game case took 85 s: every capture of
the main window drew CRT (Lottes) on the CPU, where a Skia runtime effect costs about 150 µs a pixel, some 18 s for
the 400×300 picture. The case now sizes the frame control to 24×24 before it draws; the chain it checks is the same.
Likewise the pad-menu case moves its sliders before the filter is in use and applies it last.

**Mutants** (each built and run against the cases in its blast radius, then restored from git):

| Mutant | Result |
|---|---|
| `ApplyScreenFilter` never sets `ShaderParameters` | caught: the running-game case and the pad-menu case |
| search over the name alone | caught: the grouping and search case |
| a value moved back to its default is stored rather than removed | caught: the built-in parameters case |
| Use does not record the recent | caught: the recents case |
| a float widened directly, not through `decimal` | caught: the built-in parameters case (0.041) |
| a disabled button keeps the focus | caught: the pad-menu case |
| a search hides None | caught: the search case and the pad case over the graphics sheet |
| the audit does not put the lists' selections back | caught: the desktop audit of the Shaders window |
| **any console's change re-applies the running console's shader** | **survived**, and is equivalent: re-applying the running console's own stored shader and values changes nothing that can be observed, so the console check in `ShaderChanged` saves work and guards nothing |
| **the audit keys rows by container** | **survived**: no audited list is long enough to recycle a container (above) |
| **the Use and Reset All buttons back at the left** | **survived**: both placements are reachable (above) |

Before this, LunaP's new case caught the `GroupedList` rule's removal (`LunaP.md` §94.5).

#### 4.48.7 What is not done

- **The sliders are not virtualised.** A Mega Bezel preset's 944 rows take 1.9 s to build, during which the window
  does not answer; nothing else measured comes near it (`crt-royale`, 46, in 288 ms). The fix is a virtualised list of
  rows, or building them in batches; neither was done. Nor can the parameters be searched, which is what a list of 944
  wants. *(Retired 2026-09-24: the list is virtualised, §4.48.9. The parameters still cannot be searched.)*
- **No preview.** A preset is known by its name and folder, not by a picture of what it draws.
- **Favourites** were decided against (§4.48.1).
- **A value cannot be typed**, only stepped (`LunaP.md` §94.4).
- **A preset that will not build is found only when it is used**: the status bar says why (§4.41) and the list does
  not mark it beforehand.
- **Only the built-in path is proved end to end.** The test that a slider reaches the chain a running game draws
  with uses CRT (Lottes). For a RetroArch preset the window's values reach `GameFrameControl.ShaderParameters` by the
  same code, and Serenity's own tests prove the control hands them to a built preset (`EmuSen_Serenity.md` §7.6), but
  no one test runs a preset in a running game, since it needs a Vulkan device.
- **Stored values are never pruned.** A preset removed from the pack, or a parameter renamed by an update, leaves its
  values in `graphics.json`; an id the shader no longer declares is ignored, so this is untidy and not harmful.
- **The pad and the 1280×800 text are proved headlessly only.** The walks and the audit are WiseMan's; the pictures
  are Avalonia's headless renderer at the size Game Mode uses. Neither has been seen on the handheld.
- The headings do not fold (`LunaP.md` §94.4), and a download cannot be cancelled from the window.

#### 4.48.8 A RetroArch preset builds faster, and faster again the second time (2026-09-24)

Choosing a RetroArch preset, in this window or when a game starts, now compiles its passes at once on every processor
but one, and keeps what each shader stage compiled to in `home/Shaders/spirv-cache.db`. On the desktop a Mega Bezel
preset that took 4.7 s to build now takes 0.64 s the first time and 0.48 s after that; `crt-royale` went from 1.2 s to
0.18 and 0.11. Nothing is
asked of the player and there is no setting: the file may be deleted at any time and is rebuilt as presets are used,
it keeps itself under 64 MB by dropping what was used least recently, a pack update cannot be served from it stale,
and if it cannot be read the preset is compiled as before. What was built and measured is `EmuSen_Serenity.md` §9;
the frame a running preset costs fell as well (§9.6), which the player sees only as headroom.

#### 4.48.9 Moving onto a preset is fast: a virtualised parameter list, a settle, and the rows beside read ahead (2026-09-24)

**What was asked.** "Is there a way we can speed up browsing through the shaders?", and then, "the moving onto a preset
specifically". The measurement the work started from (headless, Release, the real 2,658-preset pack): the first move
onto `crt-lottes` took 196 ms, `crt-royale` 239 ms, `crt-guest-advanced` 560 ms and Mega Bezel's `MBZ__0__SMOOTH-ADV`
1,862 ms; a held pad walking Mega Bezel-class rows cost 1.18 s a row. For six presets of about 900 parameters, reading
and parsing off the UI thread took 89–188 ms and **building the sliders on the UI thread took 972–1,536 ms**: one
`SliderRow` per parameter in a plain `StackPanel`, nearly all of them below the window (§4.48.7 recorded it as not done).

**Three changes.**

1. **The parameter list is virtualised.** It is LunaP's new `SliderList` (`LunaP.md` §97): the parameters are
   `SliderItem`s, the headings strings, and only the rows in view and half a view either side are built. The values
   live in the items, so Reset All, the stored values and the preset's defaults need no row: Reset All sets every
   item back, and a row built later shows it. Every behaviour of §4.48.2–§4.48.5 is kept: headings, values per console
   and per shader, the preset's values as defaults, per-row Reset and Reset All, live application, the "N parameters"
   note and the names `{console}.Parameter.{id}`, which each row still carries.
2. **A settle while the selection moves.** The name and the path follow every row at once. The read and the sliders
   wait until the selection has held still for 120 ms (`ShaderSettingsWindow.Settle`), **but only when a key moved
   it**, that is, the arrow keys or the pad's d-pad, which the router sends as keys (§4.45.3). A click, a selection set
   by code, **Use** (the button, Enter or A), and the focus leaving the list end a settle at once: a click is not a
   walk, Use needs nothing shown to apply a shader (it stores `Shown`, and `ApplyScreenFilter` reads the stored values
   itself), and a player who presses right from the list to reach the sliders has plainly stopped. Stale reads are
   still dropped by the reading counter, as before.
3. **The rows beside are read ahead.** After a settle, the preset rows directly above and below are read in the
   background into the window's parameter cache, at most two reads at once (`PrefetchLimit`); a row already read,
   being read or missing is skipped. A read already running is shared rather than started again, so a player who
   moves onto a row whose prefetch is still running waits for that read, not a second one. Nothing further is read.

**Predictions, made before the after-build was measured.** From the before-build's own numbers (below) and the split
above: a click onto Mega Bezel **150–300 ms** (a read of 100–190 ms plus a dozen rows), `crt-royale` 120–170,
`crt-guest-advanced` 150–250, `crt-lottes` 50–70; a held walk **80–90 ms a row**, the repeat interval being the floor;
the stop at the end of a walk **250–350 ms** (settle, read, rows); a pad step from a row the player stopped on,
whose neighbour was therefore prefetched, **140–170 ms** (settle and rows). One prediction was retired before it could
be tested: that a keyboard *jump* (a selection set by code) would cost a click plus 120 ms. Making the settle apply only
to key moves, below, removed the case.

**Measured.** `ShaderBrowseBenchTests`, gated on `EMUSEN_SLANG_PACK` like
`A_pack_preset_with_hundreds_of_parameters_is_shown_in_measured_time`, run from two Release builds, before (WiseMan
2b9664e with only the bench added) and after, interleaved before/after three times, on the development desktop with
the pack of 2026-09-24. **The time to a usable row is end to end**: from the act to the shown preset's sliders built,
laid out and drawn (a render of the window). Each case opens a fresh window, so the parameter cache starts empty; the
pack's files are in the OS's cache after the unmeasured warm-up. Medians of three, in ms (the runs are in
`~/.cache/emusen/probe/shader-browse/runs/`):

| Case | `crt-lottes` | `crt-royale` | `crt-guest-advanced` | `MBZ__0__SMOOTH-ADV` |
|---|---|---|---|---|
| click on the row | 90 → **75** | 139 → **86** | 215 → **26** | 1,121 → **84** |
| selection set by code | 78 → **71** | 100 → **90** | 149 → **26** | 1,133 → **58** |
| pad, two rows without stopping | 36 → **150** | 99 → **190** | 156 → **140** | 1,205 → **177** |
| pad, one row from a row stopped on | 32 → **137** | 80 → **148** | 155 → **140** | 1,033 → **148** |

| Case | Before | After |
|---|---|---|
| held pad over 40 Mega Bezel-class rows, per row (a press every 80 ms, the next delayed while the UI thread is busy) | 478 (388–542) | **80.4–80.5** |
| from the last press of that walk to a usable row | 879 (829–957) | **160** |

Against the predictions: the walk (80.5, predicted 80–90) and the stopped-row step (148, predicted 140–170) landed;
the click onto Mega Bezel (84) and the stop after a walk (160) came in **under** the predicted 150–300 and 250–350.
The predictions used the 89–188 ms cold read; here the whole click, read included, is 81–96 ms, so the read of files
the OS has cached is well under that (the read was not timed on its own). The before column is this bench's, not the
figures quoted at the top (a walk of 1.18 s a row there, 0.39–0.54 s here; 1,862 ms onto Mega Bezel there, 1,121–1,288
ms here): those were taken by another harness whose method is not reproduced, and the difference was not
investigated. Before and after here were measured by the same code in the same runs.

**What got slower, and why it is kept.** A single pad step onto a small preset now waits the settle: `crt-lottes`
went from 32–36 ms to 137–150 ms, `crt-royale` from 80–99 to 148–190. That is the price of the settle as asked: the
step cannot know it is not the first of a walk. A leading-edge settle (show at once when the previous key move was
more than 120 ms ago, wait only for the ones after) would give single steps back their old cost while still reading
nothing during a walk, at the cost of one read and build of the first row of every walk; it was not built, since the
request was a trailing settle, and it is recorded below. A real held pad settles once on the first row anyway, by
reading of the code rather than measurement: `PadNavigator` waits 400 ms before it repeats. The bench's walk does not
model that delay.

**Found on the way.**

- **Avalonia recycles the container holding the focus in a plain `ItemsControl`** (LunaP §97.3), which left the pad
  on nothing after the tab changed onto a scrolled parameter list. Found by the graphics-sheet pad case, fixed in
  `SliderList`.
- **`PadAudit` keyed a control by its place in the visual tree**, which a virtualising panel does not keep: it appends
  and hides containers as it scrolls, so one index could mean two rows. A control inside an item's container is now
  known by its items control, the item's index and its way down from the container, the rule §4.48.5 already applied
  to list rows; `Refocus` scrolls the item into view first. Without it the sheet audit reported the SNES, N64 and NES
  tabs' rows unreachable.
- **`PadAudit` counted operable controls in whatever state its last press left.** Before the settle a shown shader's
  sliders were rebuilt at once whichever row the last press selected, and the count happened to match; with the
  settle, the audit's end state on the Game Boy tab showed a preset's second row that the walk had never seen. The walk
  now returns to the state it began in before counting, which is what "reachable from where the focus is" means.
- *(Changed since: §4.48.10.)* **Observed, not changed.** `PadWindowRouter.FirstIn`, which puts the focus on a page's top control when the tab
  changes, chose a row of the parameter list scrolled above the view (a Reset 47 points above the window's top), since
  a row clipped by its scroll viewer is still "visible". The old `StackPanel` list had the same exposure. A `LostFocus`
  handler on the window (with `handledEventsToo`) saw nothing when that tab change moved the focus off a button on the
  page being switched away, possibly because the page was already detached; not investigated. The panel ends the
  settle on the list's `IsKeyboardFocusWithin` turning false, not on `LostFocus`.
- **The end of a 944-row list is an estimate until reached**: setting the offset to the end reached the last row on
  the third attempt (LunaP §97.4).

**The scroll bar.** The parameter list's bar is now always drawn at full width with a thumb of at least 40 points,
as the shader list's has been since `LunaP.md` §96; the old parameter list's bar auto-hid.

**Tests.** `ShaderBrowseTests` (5), each against a pack the test writes, one preset of 944 parameters under nine
headings and twelve small presets in a row:

- a 944-parameter preset builds 8 of its rows at the top and at most 40 anywhere; scrolled to the end, its last row is
  there, named `SNES.Parameter.P0952`, and a key moves it and stores 0.55; Reset All sets all 944 back, rows or not;
- the pad walks sixty rows down that list from the first slider, Reset and slider alternating, with at most nine rows
  realised at any time, then moves the slider and goes up to its Reset;
- a fast walk of six rows updates the name and path at every row, reads nothing and builds nothing until it stops, then
  reads once, where it stopped;
- with the settle set to 30 s, A on a row reached by a key move applies it and builds it at once, a click shows a row at
  once, and right from the list ends a pending settle;
- after a settle the rows either side are read in the background, two reads and never more than two at once, rows two
  away and the 944-parameter preset are not read, and the next step reads only its own next neighbour.

`PadSettingsWindowTests.A_long_preset_s_sliders_on_the_sheet_are_reached_and_walked_by_pad` puts the 944-parameter
preset on the 1280×800 Game Mode sheet, audits every tab (nothing unreachable), walks thirty rows down by pad and moves
the slider there. The existing cases changed only where they asked for every row: `ShaderSettingsWindowTests` reads
the whole list from `ShaderPanel.Parameters` and scrolls a row into view before pressing it (`Row`), and the
running-game case selects the NES tab before pressing a slider on it, which a person would have to. The audits of
§4.48.5, on the sheet and on the desktop, pass unchanged: `Every_control_of_each_settings_sheet_is_reached_by_the_pad`
and `In_desktop_mode_a_real_window_is_driven_and_every_control_reached`, with the cheat and rewind pad cases that share
`PadAudit`. 56 cases in the blast radius, all passing.

**Pictures** (`EMUSEN_UI_DUMP`, the real pack where named, looked at; in
`~/.cache/emusen/probe/shader-browse/png/`): `shaders-desktop-preset`, `-lottes`, `-bezel` (a 223-parameter preset),
`-search`, `-nopack`, `shaders-browse-944-top`, `-end` and `-walked` at 980×660, and `pad-shaders-search`,
`pad-shaders-slider`, `pad-shaders-lottes` and `pad-shaders-944-walked` on the 1280×800 sheet.

**Mutants** (each alone, built and run against the cases in its blast radius, then restored):

| Mutant | Result |
|---|---|
| every parameter row realised (a `StackPanel` under the list) | caught: the 944-parameter case and the pad walk |
| no settle: every key move reads and builds | caught: the fast-walk case and the Use/click case |
| prefetch without bound: every preset row, no limit | caught: the fast-walk case and the prefetch case |
| no prefetch | caught: the prefetch case |
| the focus leaving the list does not end the settle | caught: the Use/click case |
| **Use waits for the settle** | **survived, and is covered by another path**: Use refreshes the list to move the *In use* pill, and `GroupedList.Refresh` replaces the focused row and hands the focus back (`LunaP.md` §94.5), which ends the settle through the focus rule. With the focus rule removed as well, the Use/click case fails (Loads 2, expected 3). The explicit call is kept, so Use does not depend on how `Refresh` restores the focus |
| the audit keys controls in an item's container by tree path | caught: the pad-menu case |
| the audit counts in the state its last press left | caught: the graphics-sheet case and the long-preset sheet case |

LunaP's own six mutants are in `LunaP.md` §97.5.

**What is not done.**

- **The parameters cannot be searched**, which a list of 944 still wants (§4.48.7). *(Retired: §4.48.10.)*
- **A leading-edge settle** (above) was not built; a single pad step onto a small preset is about 100 ms slower. *(Retired: §4.48.10.)*
- *(Retired: §4.48.10, eight entries.)* **The parameter cache is not bounded.** It holds every preset read in the window's life, now including prefetches;
  a walk that stops on many rows holds each one's parameters until the window closes or the pack is downloaded.
- **Prefetched reads may go unused**: a player who moves two rows, or elsewhere, has paid for up to two reads on a
  worker thread for nothing. They are bounded at two at a time and one pair per stop.
- *(Retired: §4.48.10, a clock the tests own.)* **The settle cases run in real time.** The fast-walk case presses six rows with no pause between; a machine that
  stalled for more than 120 ms between two presses would let a settle fire mid-walk and fail it.
- **Proved headlessly only**, as everything in §4.48: the numbers are the headless renderer's, on the desktop; a real
  window adds the compositor's frame to every figure, and the handheld has not been measured.

#### 4.48.10 A step after a pause reads at once, the parameters can be searched, and the cache is bounded (2026-09-24)

§4.48.9 left five things undone, and the player asked for them in turn: the ~100 ms a single pad step onto a small
preset had lost to the settle, the focus a tab change put on a row out of view, a search of the parameters, a bound on
the parameter cache, and settle tests that depended on the speed of the machine.

**1. A leading edge on the settle.** A key move made after the selection has held still for the settle (120 ms) reads
and builds at once; only the moves that follow it inside the window wait, and the last of those reads when the
selection stops. So a single step costs what a click does, and a held walk still reads nothing between its first row
and its last. The price is one read and build of a walk's first row that the walk then leaves; a real held pad pays
it anyway, since `PadNavigator` waits 400 ms before it repeats (read from its code, not measured). A preset whose
parameters are already in the cache is now built in the same step as the move rather than a dispatcher turn later.

*Predictions, made before measuring:* a pad step from a row the player stopped on (so its neighbour was read ahead),
**15–40 ms** (from 137–163); two steps without stopping, unchanged, **+0 to +20 ms**, the first step's build now done
at once; the held walk and its stop, unchanged at 80.5 ms a row and about 160 ms; a click and a selection set by
code, unchanged.

*Measured:* `ShaderBrowseBenchTests`, the build of §4.48.9 (dc321fc5, "before") against this one ("after"),
interleaved three times, medians in ms, runs in `~/.cache/emusen/probe/shader-browse/runs/`:

| Case | `crt-lottes` | `crt-royale` | `crt-guest-advanced` | `MBZ__0__SMOOTH-ADV` |
|---|---|---|---|---|
| pad, one row from a row stopped on | 161 → **28** | 148 → **25** | 141 → **15** | 150 → **18** |
| pad, two rows without stopping | 151 → **155** | 199 → **156** | 145 → **138** | 182 → **149** |
| click on the row | 75 → **83** | 86 → **88** | 27 → **30** | 89 → **91** |
| selection set by code | 71 → **82** | 89 → **95** | 15 → **30** | 67 → **65** |
| held pad over 40 Mega Bezel rows, per row | 80.5 → **80.6** | | | |
| from the last press of that walk to a usable row | 163 → **160** | | | |

The single step landed inside its prediction on every preset, and against the build before any of this work (2b9664e:
32, 80, 155 and 1,033 ms, §4.48.9) it is now as fast or faster on all four. Two steps without stopping came in
**under** the prediction for `crt-royale` and Mega Bezel (−43 and −33 ms): the first step, now built at once, reads the
rows beside it ahead, and the second step's row is one of them, so the settle that follows ends on a read already
done. That effect was not predicted. **Not explained:** a click and a selection set by code on `crt-lottes` measured
about 10 ms slower (71 → 82, 75 → 83), in both runs made after the parameter search was added and not in the run made
before it (83 → 70, 84 → 76, `runs-leading-first/`); `crt-guest-advanced`'s selection moved 15 → 30 but its before
column ranged 15–28 across runs. Showing the search box without hiding it between presets (below) did not change it.

**2. A tab change never starts on a row out of view.** Changing tab puts the focus on the page's top control
(`PadWindowRouter.FirstIn`, §4.45.3). A row of the parameter list scrolled above the view is still "visible" to
Avalonia, and with the list scrolled it was the topmost control of all, so the pad landed on a Reset some rows up and
the list scrolled back to show it (recorded as observed in §4.48.9). `FirstIn` now passes over a control that some
scrolling area around it, inside the page, does not show. Scrolling the chosen control into view instead was the
other option offered; choosing the first control in view is what "the page's top control" already meant for a page
that does not scroll, and it leaves the list where the player left it.
`A_tab_change_never_puts_the_focus_on_a_row_scrolled_out_of_view` walks thirty rows down a 944-row list on the
sheet, changes tab twice, and requires the focus outside the list and the list's offset unchanged; written before
LunaP's §97.7, it failed without the rule with the focus on "Reset Parameter 0017". **It no longer tells the two
apart**: the row above the view it had caught was the container LunaP kept for a focus long gone, and once §97.7 let
that go, the rows the list builds above its view (half a view, §97.2) lie below the page's top controls. The rule is
proved instead by `A_tab_change_back_to_a_scrolled_page_starts_at_a_control_in_view`, on the N64 Graphics tab,
whose page scrolls: reached to its last control (the page at 651 points), left for NES and come back to, the focus
lands on `N64.RenderScale` and the page stays at 651; without the rule the focus went to the page's **Shaders...**
button and the page scrolled back to 66. So the defect was the router's all along, and wider than the Shaders window:
every scrolling settings page was put back near its top when its tab was left and taken again, which is also a change
of behaviour a player may notice. The first-control rule for a sheet opening (`FocusFirst`) was not changed.

**3. Searching a shader's parameters.** A search box above the list ("Search parameters", `{console}.ParameterSearch`)
narrows the rows to those whose description or slang id holds every word typed, in any case; a heading that matches
keeps every row under it, and a heading over a matching row stays above it. The narrowing is LunaP's
(`SliderList.Search`, `LunaP.md` §97.8), with the id given as each row's keywords. The note reads, for example, "1 of
944 parameters match “0917”" while a search is set. The search is kept as the shader changes, so a word looked for in one
preset is looked for in the next; Reset All still resets every parameter, matched or not. The box is shown while the
shader shown has parameters, and is reached by the pad: on the desktop (a new audit of the window with the 944-row
preset shown) and on the 1280×800 sheet, where it is typed with the on-screen keyboard.

**4. The parameter cache keeps eight.** It was every preset read in the window's life. It is now `RecentCache`, the
eight used most recently (`ShaderSettingsWindow.CacheLimit`): a read or a move onto a preset makes it the most recent,
and the least recent goes when a ninth arrives. Reads ahead take places like any other; one dropped before anyone
moved onto it is counted (`EvictedUnused`), so what the prefetch wastes can be read off. Eight is a guess at "the last
few stops and their neighbours", not a measurement: a Mega Bezel preset's parameters are a few hundred kilobytes, and
reading one again costs 40–190 ms.

**5. The settle is timed by a clock the tests own.** `ShaderSettingsWindow.Time` is a `TimeProvider`: the settle's
timer and the leading edge's measure of stillness both come from it. WiseMan's `ManualClock` moves only when a test
advances it, and fires the timers it made as their time comes, so the settle cases press, advance 119 ms, check that
nothing was read, advance 1 ms, and check that it was. No machine is fast or slow enough to change them. The reads
themselves are still real work on real threads; the cases wait for those, not for time.

**Found on the way.**

- **`PadAudit` counted a row built out of view.** LunaP's fix for the focused row (§97.3) kept that row's container
  after the focus had gone, so a row two hundred rows down stayed built; the audit counted it as shown and could not
  reach it (`Parameter 0192`). Fixed in LunaP (`LunaP.md` §97.7), and the audit no longer counts a row a virtualising
  panel keeps built outside its view.
- **Two shaders' rows at one index are different controls.** The audit knew a control in an item's container by list,
  index and path, so the "Glow strength" slider of a neighbouring preset and "Parameter 0001" were one key, and the
  walk never explored the second. The key now includes the control's accessible name.
- **A path that will not replay hid a control.** The walk records each control by the first path that reaches it; one
  whose shortest path depended on a scroll offset was never explored, and the pad-menu case could not find "Mask dark"
  once the search box stood between the buttons and the rows. A control first found by a path that would not replay
  is now explored again from the next path that reaches it.
- **The audit now stops once it has found every control shown**, instead of walking to its limit of 400 paths. On the
  944-row sheet the full walk took 7 min 23 s, since each path is replayed from the start; it now takes seconds.
- **A focus left on a hidden control.** The desktop audit of the 944-row preset threw from Avalonia's directional
  search ("FocusedElementBounds needs to be set"): a replay had left the focus on the search box after a shader with
  nothing to adjust hid it. The router now treats a focus that is not effectively visible as no focus and starts again
  at the first control. A direct reproduction was tried and failed: hiding the focused box in a test made Avalonia
  clear the focus at once, so how the replay left it there was not found; the audit case is the one that fails without
  the rule.
- **A pack without its stamp file lays the list out unbounded.** The browse tests' pack had no stamp, so the window took
  it for "not downloaded" and gave the shader list an `Auto` row (§4.48.1's layout without a pack), which grew past the
  window: a click on a row near the bottom hit nothing. The tests now write the stamp. A real pack always has one;
  presets present without it are not a state the download leaves, and the layout was not changed.

**Tests** (the blast radius, 84 with the Graphics window's, all passing): `ShaderBrowseTests` gains the leading-edge walk
(`A_fast_walk_reads_its_first_step_and_where_it_stops_and_nothing_between`, on a `ManualClock`), the search
(`The_parameters_are_narrowed_by_a_search_of_their_descriptions_and_ids`), the cache
(`The_parameter_cache_keeps_the_most_recent_eight_reads_ahead_included`, and `RecentCache` alone) and the desktop audit
with the long preset shown; `PadSettingsWindowTests` gains the two tab-change cases and the search typed on the sheet.

**Pictures** (in `~/.cache/emusen/probe/shader-browse/png/`, looked at): `shaders-browse-944-search` on the desktop,
`pad-shaders-944-search` and `pad-shaders-tab-return` on the sheet, with §4.48.9's set rendered again. The search box
sits under the note, the width of the list; after a tab change the Category dropdown holds the focus and the
parameter list is where it was left.

**Mutants** (each alone, against the cases in its blast radius, then restored):

| Mutant | Result |
|---|---|
| no leading edge | caught: the fast-walk case and the prefetch case |
| the settle timed by the machine's clock, not `Time` | caught: the fast-walk case |
| the cache unbounded | caught: the cache case |
| reads ahead counted as used | caught: the cache case (`EvictedUnused`) |
| the search box narrows nothing | caught: the search case and the sheet search case |
| ids not searched | caught: the search case |
| a tab change may start on a row out of view | **survived at first**: the Shaders case no longer separates them (above); caught by the Graphics case written for it |
| a focus on a hidden control is moved from | caught: the long-preset desktop audit and the pad-menu case |
| the audit gives up on a path that would not replay | caught: the pad-menu case |
| the audit counts rows built out of view | caught: the long-preset sheet case |
| **the audit keys rebuilt items without their names** | **survived**: the collision it prevents was seen (a neighbouring preset's "Glow strength" and "Parameter 0001" under one key, in a failed walk's dump), but the walk that showed it was cured by building cached parameters in the same step; no case today depends on the name. It is kept, since nothing stops two shaders' rows sharing an index |

LunaP's five new mutants are in `LunaP.md` §97.8.

**For the handheld.** `~/.cache/emusen/probe/shader-browse/handheld-bench.sh` builds two commits beside a checkout
(git worktrees, removed afterwards), runs the bench from each interleaved, and prints the medians:
`bash handheld-bench.sh <EmuSen checkout> <slang pack folder> [rounds] [before] [after]`. It needs the .NET 10 SDK,
python3, and LunaP beside the checkout at 9e8d201 or later. It was run here once, before only (the sibling LunaP
checkout lacked 9e8d201, which the after build needs, and the script stopped as it should).

**What is not done.**

- The ~10 ms on `crt-lottes`' click, above, is measured and not explained.
- Eight entries is a chosen number; the cache's memory was not measured.
- `FocusFirst`, the rule for a sheet's first control, still counts a control out of view.
- How the audit's replay left the focus on a hidden control is not known.
- The handheld numbers are the coordinator's to take.

### 4.49 Rewind as a reel of pictures (2026-09-24)

**What the player asked for.** "When you're in game and select the rewind button from the quick menu, I want it to
pull up a reel you can go back through and select where you want to rewind to." Holding Backspace already walked the
history back a snapshot at a time (§4.21b, `EmuSen_Rewind_And_FastForward.md` §4); there was no way to see the
history, and nothing on the pad reached it.

**Where it is.** The pad's menu over a game has **Rewind** after State Slot. With no history it reads *Rewind (nothing
to go back to yet)*, or *Rewind (not kept for Mars (C#))* on the C# N64 core, which keeps none (§4.21b); choosing it
then does nothing and the menu stays, so an empty reel never opens. On the desktop, **Emulation → Rewind...** opens the
same reel. No hotkey was added: Backspace is already rewind, held.

**What it shows.** A window titled Rewind — on a sheet in Game Mode (§4.45.2), a dialog on the desktop — holding a large
picture of the selected moment with how long ago it was, and under it a strip of every moment that has a picture, oldest
at the left, **Now** at the right and selected first. Each tile is labelled with its distance back, from the frames the
buffer counted and the console's frame rate: two decimals under ten seconds, since moments are a fifteenth of a second
apart; one decimal under a minute; then minutes and seconds. The strip is LunaP's `TileStrip<T>` (LunaP `docs/LunaP.md`
§95), built for this: one row, virtualised, the selected tile centred and ringed as the library's covers are.

**The controls.**

| Input | Does |
| --- | --- |
| Left, Right (d-pad or stick) | one moment older or newer; stops at the ends |
| L1, R1 | the nearest moment at least five seconds older or newer, else the end |
| L2, R2 | the oldest moment, Now |
| A | rewind to the selected moment and resume; on Now, resume with nothing changed |
| B | cancel: resume exactly where the game was |
| Down, then Left or Right | the Rewind Here and Cancel buttons, for a pad user who wants them |
| pointer | a press selects a tile, a double-press rewinds to it; Rewind Here and Cancel |
| keyboard | the strip's own keys (Left, Right, Home, End, Page Up and Down, Enter); Escape cancels |

On a sheet, the footer names these buttons in place of the sheets' usual ones, and is put back when the reel closes.
The reel handles the shoulders, the triggers, and Left, Right and A while the strip has the focus, through a new
`IPadDriven` a window may implement; everything else is `PadWindowRouter`'s as before (§4.45.3).

**Pausing.** Opened from the pad's menu, the reel takes over the pause the menu made: the game does not run a frame
between the menu and the reel, and after A or B it resumes only if the menu had paused it. Opened from the desktop menu
with the game running, it pauses first and resumes after. A game the player had paused stays paused either way.

**How it is built.** The moments are read on the emulation thread, which owns the buffer, as a request (§4.21a) while the
game is paused, together with a picture of the frame on screen for the Now tile; the reel is built from them on the UI
thread. A choice goes back to the emulation thread as another request: `RewindBuffer.RewindTo` (one load, however far
back), then the providers refreshed, the audio drained and the rate control reset, and the restored frame presented —
what the held rewind does after its step. Cancelling sends nothing: the machine is not touched after the reel's
moments are read.

**The pictures.** Every snapshot Mistress takes gets a picture 160 pixels wide (`EmuSen_Rewind_And_FastForward.md`
§5.3), from the frame the loop is about to show. When the core says its picture has not changed since the last one it
offered (`FrameSerial`), the picture made for that frame is attached again, or, if none was, the frame is fetched for
it. A frame skipped by fast-forward gets none, and its moment is not on the reel (§5.6 there). The pictures are capped
at 32 MB; past that the older ones thin out and the snapshots stay (§5.4 there). On MarsRT, whose picture is a frame
behind the machine (§4.21b), a moment's picture is the frame before it.

**Choosing a moment discards the newer history,** as the held rewind does; the reasoning, and what it costs, is
`EmuSen_Rewind_And_FastForward.md` §5.2. Nothing is discarded until A.

**Tests.** `PadRewindReelTests`, eight cases and a ninth that only renders, each on a real `MainWindow` running a synthetic SNES ROM whose backdrop
changes colour every frame, played unthrottled until it has history and driven by the simulated pad:

- A, three moments back, with the game already paused: the whole state afterwards equals `StateAt` of that tile byte for
  byte, the buffer's frame is the tile's, the game stays paused, and the tile's label is its distance at the console's
  rate.
- B after moving five back and a stride, with the game paused by the player: the whole state, measured before the reel
  opened, equal after it closed; the history's frames and depth unchanged; the game still paused.
- A on Now: state and history unchanged, and nothing said about a rewind.
- A from the menu with the game running: rewound, and resumed.
- The shoulders stride five seconds within half a second, twice and back; the triggers reach both ends; Left at the
  oldest stays; every tile's label is its distance at the console's rate; B resumes the game the menu paused.
- The pad audit: the strip has the focus when the reel opens, and every operable control on the sheet — the strip,
  Rewind Here, Cancel — is reached by the d-pad (`PadAudit.Reachable`); A on Cancel is the button's.
- No history (the buffer cleared while paused): the entry names why, A leaves the menu open, no sheet.
- On the desktop: the reel is a dialog; a press on a tile selects it, a second press rewinds to it, and the state is
  that tile's `StateAt`.

The core's side is `RewindBufferTests` and `RewindToMomentTests` (§5.2 there), on the SNES, the Game Boy and MarsRT.

**Pictures looked at.** Rendered with `EMUSEN_UI_DUMP`: the reel on a sheet at 1,280×800, with Now selected and three
back; and the desktop dialog, with Now and a chosen tile. The first renders showed the large picture and each tile's
picture pinned to the left of its box, and at 1,280×800 the reel's own hint and buttons cut off below the sheet; the
pictures are centred and the stage lowered from 360 to 280 pixels, after which everything fits. The synthetic ROM's
pictures are flat colours, so `A_real_game_on_the_reel_is_rendered_for_a_look` renders the same reel over a real game
when `EMUSEN_REWIND_REEL_GAME` names a ROM and a state copied out of the library (unset, it does nothing). Over Yoshi's
Island and Super Mario Land 2, on the sheet and on the desktop, the 160-pixel pictures are legible as tiles — Yoshi,
the coin counter, the Game Boy's status line — and the large picture is the same 160 pixels enlarged about twice,
blocky but readable; it is not the frame (§5.6 of the rewind document).

**Mutants.** Fifteen, each alone, built and run against `RewindBufferTests`, `RewindToMomentTests` and
`PadRewindReelTests` (37 cases), with the source restored after each:

| Mutant | Caught by |
| --- | --- |
| `RewindTo` applies one delta too few | twelve cases, on every core |
| `RewindTo` keeps the newer moments | nine |
| `RewindTo` loads at every step | the one-load test, three cases |
| a picture goes to the oldest moment | seven |
| the budget thins the newer half | the budget test |
| a picture ignores row repeat | the downscale test |
| opening the reel loads the newest snapshot | A three back, and B |
| cancel does not resume the game the menu paused | the stride test's B, and Cancel reached by the pad |
| the router does not consult `IPadDriven` | four reel cases |
| the loop attaches no pictures | seven reel cases |
| the stride is five tiles, not five seconds | the stride test |
| an empty history is not refused | the no-history test |
| the labels use 60 Hz | **survived** at first; caught once the stride test compared every tile's label at the console's rate |
| Now rewinds to the newest snapshot | **survived** at first — Now's frame is a moment's only one time in four, and otherwise the load is refused and nothing changes; caught once the Now test also read the status line |
| the menu's pause is not handed over to the reel | **survives** |

The survivor is near-equivalent: without the hand-over, closing the menu resumes the game and the reel pauses it again
in the same call on the UI thread, microseconds later, so a frame runs between them only if the emulation thread wakes
and starts one inside that window. No test holds that window open without a hook into the loop, and none was added.

**Not done.** The preview is the 160-pixel picture enlarged, not the frame; a moment on a frame fast-forward skipped
has no picture; there is no redo; nothing for the C# Mars; the reel opened from the desktop menu while running reads
its moments between a frame and that frame's capture, so its Now can be one frame newer than its labels count; and
none of this has been tried on the handheld.

### 4.50 Rewind takes four snapshots a second (2026-09-24)

**What changed.** Mistress used `RewindBuffer`'s default interval, a snapshot every 4 frames: fifteen a second on a
60 Hz console. The player found that too many save states once the reel (§4.49) made them visible, and asked for four
a second. Mistress now sets the interval when a game loads from that console's own frame rate,
`RewindIntervalFor(hz) = round(hz / 4)`: 15 frames on the NES, Game Boy, SNES and N64 at 60 Hz, 12 on a 50 Hz PAL
game (`Math.Round` takes 12.5 to the even 12, so 4.17 a second). `RewindBuffer.DefaultIntervalFrames` stays 4, so the harness and every other user
of the buffer are unchanged.

**A held rewind keeps its speed.** The hotkey held used to step back one snapshot every frame, four frames of history
a frame: four times real speed. With snapshots 15 frames apart, a step a frame would run at fifteen times. It now steps
once every `round(interval / 4)` frames (4 at 60 Hz), so it still plays back at about four times real speed, showing
about fifteen pictures a second rather than sixty. `HeldRewindSpeedup` is that 4.

**What it buys.** A quarter of the captures: a quarter of the snapshot encoding on the emulation thread, of the reel's
pictures (§4.49 measured them as most of rewind's memory on the SNES and Game Boy), and the same memory budget now
reaching about four times as far back. What it costs is granularity: the reel and a held rewind land on quarter-second
moments rather than fifteenth-second ones.

**Tests.** `PadRewindReelTests` asserts the interval is a quarter second of the running console's frames and that the
reel holds at least one moment per interval played; a mutant that leaves the default interval is caught. The held
rewind's pacing is not asserted by a test; `MarsRtEngineTests`' hold test still passes under it.

### 4.51 The status bar can be hidden, whole or in part (2026-09-24)

Preferences ▸ Appearance has three switches, all on by default and stored in `appsettings.json`:

- **Status Bar** (`ShowStatusBar`): the bar along the bottom of the main window. Off, it is gone, and with it every
  message it would have carried ("State saved", errors), since nothing else shows them.
- **Messages** (`ShowStatusText`): the text at its left, the game's name, Paused, State saved and errors.
- **Frame Rate** (`ShowFpsBar`): the frames-per-second readout at its right. Off, the emulation loop stops posting
  the line to the window; it still writes `[fps]` to the console, which the Steam log keeps.

The bar also goes when both of its parts are off, so an empty strip is never left. A change applies at once
(`PreferencesWindow.StatusBarChanged` → `MainWindow.ApplyStatusBar`) and holds across starts. The player first asked
for the frame rate alone, then for the whole bar and a switch for each part. `LibraryScreenTests.The_status_bar_and_its_parts_follow_preferences_at_once_and_are_remembered`
walks the switches and a restart; mutants that drop the live hook or keep an empty bar are caught.

### 4.52 Big picture: an ES-DE theme as the big-screen library (2026-09-25)

`EmuSen_BigPicture.md` plans a big-picture mode that draws EmulationStation-DE themes. Its stages (a) to (c) built the
theme loader and a scene drawn with LunaP's controls (§12–§14 there). This section is stage (e) as a player meets it:
the theme's views shown in the library's place in a big-screen session, steered by the pad. The design record, the
predictions and the measurements are §15 of that plan.

**The settings.** Preferences ▸ Appearance holds four rows, stored in `appsettings.json`:

| Row | Setting | Default | What it does |
|---|---|---|---|
| Library Style | `LibraryStyle` | `Theme` | `Mistress` keeps this library (§4.33) in a big-screen session; `Theme` shows the theme's views instead when one loads |
| ES-DE Theme | `BigPictureTheme` | none | a theme folder holding `capabilities.xml`, read in place and never written |
| ES-DE Media | `EsdeMediaDirectory` | none | an ES-DE `downloaded_media` folder, read in place; without one the theme shows the covers of §4.33 |
| Navigation Sounds | `NavigationSounds` | on | the theme's seven navigation sounds |

A change takes effect when the Preferences sheet closes, since closing it refreshes the library.

**When the themed view is shown.** All four must hold: the session is big-screen (§4.29, §4.43); Library Style is
`Theme`; a theme folder is set; and the theme reads without error and has a view for at least one system with games.
Otherwise the library of §4.33 is shown, as before, and when a set theme could not be read the status line says why.
The desktop keeps its sidebar library whatever the style says: the user decided on 2026-09-25 that the themed view is
for big-screen sessions only. *Amended by the user on 2026-09-26 (§4.54): the desktop keeps its sidebar library, in a
window or plain full screen, and gains Big Picture in the View menu, below Fullscreen, behind which the themed view is shown
under the same four conditions.* While the themed view is showing, the status bar is hidden, so the view has the whole
window; it comes back with the game.

**What is shown.** One system per library shelf that has games, in the shelves' release order, under ES-DE's system
name for it (`nes`, `snes`, `n64`, `gb`, `gbc`, carried by `CoreCatalog.LibraryShelf.EsdeSystem`). A gamelist lists
favourites first, then by title, as ES-DE does by default. Games carry Mistress's own records (§4.32): favourite, last
played, play count and play time. Media come from the ES-DE folder when one is set, and a game's cover otherwise from
the art folder of §4.33. *Since 2026-09-26 (§4.60) the order is the player's own cover, then ScreenScraper's media
store, then the ES-DE folder, then OpenEmu's failover, and the gamelist's description, developer, publisher, genre,
players, rating and release date come from ScreenScraper.* The theme's clock is off, as ES-DE's `DisplayClock` is by default; there is no switch for it
yet. The status bar reads the battery and radios from Linux's sysfs (`DeviceStatusReader`): the system's own battery,
not a mouse's or a pad's (whose `scope` is `Device`); Wi-Fi as on when a wireless interface is up; Bluetooth as on when
its radio switch is blocked neither by software nor by hardware. Nothing is shown that sysfs does not report.

**The pad.** §4.29's grammar, as `EmuSen_BigPicture.md` §4.9 tables it:

| Button | System view | Gamelist |
|---|---|---|
| Left, right | move the carousel; held, it repeats after 500 ms and then every 200 ms (180 and 80 ms with the theme's `fastScrolling`) | the next or previous system's gamelist, at the game last chosen there; once per press |
| Up, down | (a vertical carousel moves on these instead) | move the list; held, 500 ms, then 114 ms, then from 1.7 s four at once and every 15.9 ms; a held list stops at its ends, a tap wraps |
| South | the system's gamelist, at the game last chosen there | start the game |
| East | back to a suspended game, if there is one | clear the search if there is one, else back to the system view |
| L1, R1 | | a page: the rows the list shows at once |
| L2, R2 | | the first and the last game |
| North | | search, on the on-screen keyboard (§4.45.6) |
| Select | | mark or unmark a favourite; the game stays selected where the list moves it |
| Start, Guide | the pad menu (§4.29) | the same |

The repeats are ES-DE's, measured in `EmuSen_BigPicture.md` §14.7, and run on the view's own clock, not on
`PadNavigator`'s 400/80 ms: a held direction reaches the view as held and let go. Everything else reaches it through
`PadNavigator` as before. The search narrows every gamelist to the titles that match until it is cleared, and the list
moves under it as the library's search box lets it (§4.30). The pad menu, the settings sheets, the cheats and the
resume question are the same as over the library of §4.33 and are drawn inside the window (§4.45.2); no window is
opened.

**Starting a game and coming back.** South runs `StartGameAsync`, the path every start takes (§4.31): the firmware
prompt, then the resume question on a sheet. The menu over the game's "Game Library" (§4.18) brings the themed view back
at the same system and game, in the same view, with the search still applied; "Close Game" does the same after writing
the resume state. The selection is kept by the system's name and the game's file, not by position, so a list that
changed underneath (a favourite marked, a file added) still finds it.

**The help bar follows the pad.** Its entries are the theme's layout filled with the actions above, and its icons are
drawn by LunaP (`PadGlyph`, `LunaP.md` §103) in the set for the connected pad's family: Xbox, PlayStation, Nintendo, or
a generic set drawn by position when the family is not known. The family is SDL's own reading of the pad
(`SDL_GetGamepadType`, from its vendor and product) when SDL knows it; when SDL answers Standard or Unknown, the pad's
name decides, and a name holding "Legion Go", "Steam Deck" or "Xbox" is taken for an Xbox layout. The window asks at
every pad poll, so a different pad changes the icons at once and nothing else moves. A theme's own `customButtonIcon`
files are used when it sets them, under ES-DE's key for the family (`_XBOX`, `_PS`, `_switch`); a generic pad takes
the Xbox keys, as ES-DE's own default controller type is Xbox. The drawings are Mistress's own through LunaP; none is
copied from ES-DE or a vendor.

*Limits of the detection.* A pad behind Steam Input reaches SDL as Steam's virtual pad and is classified as that pad
is, not as the hardware in the player's hands. A generic pad whose name carries none of the words is drawn generic
even when its printing is known to the player. `GamepadManager` still opens only the first pad; it now lets a pulled
pad go (`SDL_GamepadConnected`), so its one-a-second rescan can open the next one.

**Sounds.** The theme's seven sounds (`systembrowse`, `quicksysselect`, `select`, `back`, `scroll`, `favorite`,
`launch`) play on `UiSoundPlayer` (Endymion), a second SDL audio stream opened on the default playback device beside
the game's `AudioPlayer`. SDL mixes the two logical devices on the one physical device, so neither stream's clearing or
gain touches the other. Each WAV is decoded once with `SDL_LoadWAV` and converted to 48 kHz stereo float; a new sound
clears the stream first, so the steps of a held list replace one another rather than queue. The gain is 0.7, ES-DE's
default navigation volume (its `SoundVolumeNavigation` of 70). The stream opens on the first sound, so a session that
plays none never touches the device. With the switch off nothing reaches the stream.

**Drawing only while something moves.** The view asks for a frame (`RequestAnimationFrame`) only while something moves
on it: a carousel sliding, a list held, a fade. When a text is in its pause before scrolling (the selected name's three
seconds, a description's start delay), the window sets a timer for the end of the pause and draws nothing until then;
when everything is still for good it draws nothing at all. The answers come from LunaP's scroll queries (§103.3 there).

**Tests.** `ThemedLibraryPadTests` (13 cases: each row of the table above, the conditions for showing the view, and the
page by the list's own rows), `ThemedLibraryFlowTests` (3: the whole walk from system to game and back, the frame after
the return, the resume question on a sheet), `ThemedLibraryHostTests` (16: the render loop's sleep, the pad families, the help entries,
the help bar alone changing, the sounds and their switch and stream, sysfs, Preferences, the media scan, and every sheet
reached by the pad) and `ThemedLibraryReferenceTests` (2 on Art Book Next, skipping visibly without it, and the PNG
tool). They run on WiseMan's `PadDriver` with a clock the test moves. Mutants are in `EmuSen_BigPicture.md` §15.

**What it does not cover.**

- *No device was used.* The families of real pads, the Legion Go S's name and type under SDL and under Steam Input, and
  whether the sound stream's latency is acceptable beside the game's 4,096-frame device buffer are for the device.
- The first showing scans the media folder for each system on the UI thread: 224 ms for 3,508 games with an empty
  media folder on the desktop (§15 of the plan). Later showings reuse the answer.
- The search is Mistress's, not ES-DE's: ES-DE offers no search box in a gamelist.
- The system order is the shelves' release order; ES-DE's own order was not established.

### 4.53 Big picture: the theme's settings, the grid, and themes downloaded on request (2026-09-26)

Stage (f) of `EmuSen_BigPicture.md` (its §16 is the record: predictions, measurements, mutants). This section is what a
player meets.

**Theme Settings.** A sheet over the view, opened from the pad menu's **Theme Settings** (big-screen sessions, outside a
game) or from Preferences ▸ Appearance ▸ **Theme Settings…**. Its **Options** tab is built from the theme's
`capabilities.xml`, so it offers exactly what the theme declares:

| Row | What it lists | Stored as |
|---|---|---|
| Variant | the variants the theme marks selectable, in its own order, under their `en_US` labels | `Variant` |
| Colour Scheme | every declared scheme, under its label | `ColorScheme` |
| Font Size | the declared sizes, in THEMES.md's menu order (Medium, Large, Small, Extra Large, Extra Small) | `FontSize` |
| Aspect Ratio | Automatic, then the declared ratios in the table's order | `AspectRatio` (none for Automatic) |
| Language | only when the theme declares languages; Art Book Next declares none | `Language` |
| Transitions | Automatic, the theme's selectable profiles, then the built-in ones it does not suppress | `Transitions` (none for Automatic) |

A row the theme declares nothing for is left out. A label two entries share is followed by the entry's name. The first
time, each row shows what the loader would pick on its own (§12.4 of the plan): the first selectable variant, the first
scheme, Medium, Automatic.

- **A choice applies at once.** It is saved and the view beneath the sheet is rebuilt at the same system and game before
  the sheet closes; there is no Apply button.
- **Choices are kept per theme.** `appsettings.json` holds them under `BigPicture`, keyed by the theme folder's full
  path, so switching themes and back restores each one's. A stored name the theme no longer declares (an update renamed
  a variant, say) is not an error: the loader falls back to its default for that row.

**The grid variants.** A theme's game list can be a grid, as Art Book Next's three "Grid" variants are. Mistress draws
it with LunaP's `ImageGrid` and moves it as ES-DE 3.4.1 was measured moving (`EmuSen_BigPicture.md` §16.5):

| Button | In a grid game list |
|---|---|
| Left, right | the previous or next game, across the end of a row to the next; a tap wraps at the ends of the list, a hold stops there |
| Up, down | the game a row above or below; they stop at the first and last rows, and down into a short last row takes its last game |
| L1, R1 | a page: the whole rows shown |
| Held | 500 ms, then a step every 200 ms, with no faster speed |

Each step eases the selected cover up and the last one down over 250 ms; when the selection passes the last row shown,
the rows slide over the same 250 ms and the selected row stays on the bottom row. The game's metadata fades out while a
direction is held, as over the list. A grid game list takes all four directions, so left and right do not change the
system there as they do over a list; East goes back to the system view, where they do.

**Variant triggers use the media Mistress has.** A theme can switch a variant for the game lists when a system's games
have no media of some kind (`noMedia`) or no videos (`noVideos`). Mistress answers from the ES-DE media folder when one
is set (§4.52), and the cover type also from its own art folder (§4.33). A type counts when any game of the system has a
file of it. Files are found under ES-DE's own names and extensions: `.png`, `.jpg` and `.webp` for pictures, and
`.mp4`, `.mkv`, `.avi`, `.wmv`, `.mov` and `.webm` for videos (ES-DE's `USERGUIDE.md`). Before this stage videos were
never found, so `noVideos` fired on every system; a `.jpeg` file, which ES-DE does not read, is no longer read either.
Each type folder is listed once per system, and read again at the next showing when it has changed.

**Themes.** The **Themes** tab lists the themes Mistress downloaded, then the folder set in Preferences when it is
another one, with where each came from:

- **Download Art Book Next** fetches GitHub's archive of the theme's `main` branch
  (`codeload.github.com/anthonycaccese/art-book-next-es-de/zip/refs/heads/main`, about 220 MB) into
  `home/Themes/art-book-next-es-de/` (`DataStore.Themes`). Nothing is fetched until the player presses it; EmuSen ships no
  theme. The newest commit is asked of GitHub's API first and recorded, with the date, in `.emusen-theme` beside the
  theme. When no theme was set, the new one is used at once, and its About sheet opens, once, after the first download.
- **The archive is checked before it replaces anything.** It is written beside its final place (`.zip.part`), unpacked
  into a sibling folder (`.part`), entry by entry, refusing any entry that would land outside it, and swapped in only
  when its `capabilities.xml` reads without error and a `theme.xml` loads. A failed, stopped or broken download leaves
  the theme that was there as it was, and nothing beside it.
- **Check for Update** asks GitHub for the branch's newest commit; when it differs from the stamp the button becomes
  **Update**. An update keeps the player's `theme-customizations/` folder byte for byte, over anything the archive
  carries there: that folder is where Art Book Next's README tells a player to put a custom `colors.xml`, artwork and
  logos.
- **Remove** asks first, then deletes the folder. It is offered only for a folder Mistress downloaded (directly under
  `home/Themes`, with its stamp); a folder read in place is never written, let alone removed.
- **Use** makes a listed theme the big-screen library's theme and sets Library Style to the theme.
- **Closing the sheet stops a download in flight**, as does closing the window under it; nothing keeps fetching behind a
  closed sheet (a sheet is not closed when its window is, so both are handled).

**About.** Each listed theme has an **About** sheet: its name; its author (for a downloaded theme, the owner of its
GitHub repository); its licence and credits, read at display time from the theme's own `README.md` (the sections whose
headings name a licence and credits), else from a `LICENSE` file, else a line saying it states none; its source and
commit; and a statement that the theme was downloaded at the player's request, or is read in place, and is not part of
EmuSen. EmuSen carries no copy of any theme's text: for Art Book Next the sheet shows the licence line of its README, a
CC BY-NC-SA 2.0 licence, as the README states it on the day it is read.

**Tests.** `ThemeSettingsSheetTests` (the rows from a synthetic theme and from Art Book Next, a choice applied beneath
the open sheet and equal to a fresh build, choices per theme and a stale one, Preferences, every control reached by the
pad, the download flow with its About sheet and an update, and the two closing cases), `ThemeDownloadsTests` (a fake
GitHub: the download and its stamp, an update keeping `theme-customizations`, five broken downloads, a cancelled one,
removal, the attribution) `ThemedLibraryTriggerTests` (the extensions, presence by listing, the variant following the
media), `GridSceneTests` (the measured layout, a step, a slide, the ends, the repeats, the metadata fade, a step while
moving, the units, and Art Book Next's grid against ES-DE's still) and `ThemedGridPadTests` (the pad over a grid). Every server is a fake written for the tests; no test reaches the network.

**What it does not cover.**
- Only Art Book Next has a download button. Another GitHub theme, ES-DE's theme list and a GitLab theme are §6's plan and
  are not built; a theme folder can still be chosen in Preferences and read in place.
- The download is not resumable: a stopped download starts again from the beginning.
- The stamp is the only record of where a theme came from; a downloaded folder whose stamp is deleted is treated as read
  in place, and cannot be removed from the sheet.

### 4.54 Big picture from the desktop: Big Picture below Fullscreen in the View menu (2026-09-26)

**The request.** The user, 2026-09-26, in three messages:
1. "There needs to be a button to enter big picture mode on desktop as well". A relay of a second message read it as
   "fullscreen enters big picture", and the first build made the desktop's full screen and big picture one state.
2. The user corrected that: "i did not want the fullscreen button to trigger big picture on desktop automatically, i
   wanted a separate button that triggers big picture mode separate from the fullscreen button. Clicking the button
   would make emusen fullscreen, but also put it into big picture mode. i want the user to have the option between both
   in desktop". The second build put a Fullscreen and a Big Picture button in the library's toolbar.
3. The user then placed it: "The big picture button for desktop mode should be placed under the view menu, below
   fullscreen". The toolbar buttons were removed.

This section describes the third build. The first two are kept in `EmuSen_BigPicture.md` §18, with what each got wrong.
Until any of them, big screen was decided once, when the window was made (§4.29, §4.43), and the themed view was offered
in big-screen sessions only (§4.52; the plan's §10.1, Q8).

**What the player meets on the desktop.** The **View** menu holds two adjacent entries, each showing its key:

- **Fullscreen** (F11) makes the window full screen as it is, with the menu bar, sidebar and library, and ticks itself.
  F11, the in-game bar's full-screen button, the pad menu's **Full Screen** and the window manager's own full screen all
  do the same. None of them enters big picture.
- **Big Picture** (F10), directly below it, makes the window full screen *and* enters big picture. The **Big Picture**
  hotkey (F10 by default, listed with the other hotkeys in Controller Bindings and rebindable there) and the pad menu's
  **Big Picture** do the same. It is a command, not a tick, because the menu bar it sits in is hidden while big picture
  runs, so no one could see a tick on it.

After that:

- **Big picture** is what a big-screen session is (§4.29, §4.45.2, §4.52): the window full screen, no menu bar and no
  sidebar, the console choice back in the filter bar, and the library's text at 24 points. The library is the theme's
  view when Library Style is Theme and a theme loads, else Mistress's own big-screen library. Every window opened from
  there is a sheet, and the pad menu, the resume question and the pausing rules are the session's.
- **Leaving big picture on purpose** (the pad menu's **Exit Big Picture**, or the Big Picture key) puts the window back in
  the state it was entered from: normal, maximised, or plain full screen. The menu bar, sidebar and text sizes come back, and the desktop library returns to the console, search and
  game it had when big picture was entered, whatever the pad did meanwhile in Mistress's big-screen library, which
  shares those controls.
- **Esc** leaves the same way, but only where it has nothing else to do: the library showing, no game suspended behind
  it, and nothing over it (no sheet, no pad menu, no on-screen keyboard). With a game suspended, Esc keeps its meaning
  (§4.18) and goes back to the game.
- **Leaving full screen while in big picture leaves big picture too.** F11, the in-game bar's full-screen button or the
  window manager take the window out of full screen, and big picture follows. The window stays in the state that route
  chose: F11 goes back to the state full screen was entered from (LunaP §75.2), so a big picture entered from a
  maximised window comes back maximised. Refusing was the alternative. It was rejected because a refusal fights the
  window manager, which may already have resized the window, and leaves a big-screen layout in a desktop window with no
  menu bar. F11 in big picture therefore never strands the player: it is one press back to a desktop window. It does
  not return to plain full screen when big picture was entered from there, because the press asked to leave full screen.
- **Game Mode is always big picture.** In a gamescope session (§4.43) there is no menu bar, no pad-menu entry for Full
  Screen or Big Picture, and neither F11, the Big Picture key, Esc nor a change of window state leaves. Exit is not
  offered, rather than offered and argued safe. gamescope shows the window full screen whatever it asks for, and the
  desktop layout there would have a menu bar with no pointer and no window manager to drive it (§4.24, §4.29).

**How the two are kept apart.** Big picture no longer follows the window's full screen. LunaP's
`ToolWindow.FullScreenChanged` (raised for every change into or out of full screen, whatever made it) is used in one
direction only: leaving full screen calls `SetBigPicture(false, restoreWindow: false)`. Entering full screen does
nothing. `SetBigPicture(true)` records `WindowState` before asking for full screen, and `SetBigPicture(false)` sets it
back. It has already cleared its flag by then, so the event that restoring raises finds nothing to do. `SetBigPicture`
is also the one guard for Game Mode.

**What was decided once, and what it is now.** Every decision the window took at start for big screen, found by
reading `StartPadNavigation`, `Program.BuildAvaloniaApp` and every reader of the flag:

| Decision | Before | Now |
|---|---|---|
| the flag itself (`_bigScreen`) | set once in `StartPadNavigation` | `ApplyBigScreen(bool)`, run at start and by every switch |
| menu bar, sidebar, filter facet, the library's text sizes | set only for a big-screen start | set for either answer; the text sizes are cleared back to the inherited ones, not set to a copy |
| full screen at start | an `Opened` handler added for a big-screen start | one `Opened` handler that reads the flag |
| `SheetLayer.PresentsWindows` | set once | set by every switch. LunaP reads it when a window is shown, so windows opened after a switch follow the new answer |
| process-wide popup embedding (`LunaApp.EmbedPopups`) | the setting, `--bigscreen` or Game Mode | `--bigscreen` or Game Mode only. A platform option cannot be switched, and a start the setting put in big picture can now be left, so the desktop after it would otherwise draw every popup inside the window for the whole session |
| the window's own popups | not embedded | `EmbeddedPopups` on the window (LunaP §92.5), switched with the mode, so the facet's list stays in the window in big picture |
| the themed library's setup (`SetUpThemedLibrary`) | at start, for a big-screen start only | on the first entry, once for the window's life. Its handlers do nothing on the desktop, since they ask `ThemedStyleWanted`, which reads the flag |
| the pad's 16 ms timer | started at construction | unchanged: it serves the desktop too (§4.29). The tests hold that it is one timer after every switch |
| the pad menu | Full Screen or Leave Full Screen; Theme Settings in a big-screen session | Full Screen outside big picture; Big Picture or Exit Big Picture, not in Game Mode; Theme Settings reads the flag when the menu opens |
| F11 | set `WindowState` to Normal or FullScreen | `ToggleFullScreen`, so leaving returns to the state full screen was entered from. A maximised window used to come back normal. The fix is independent of big picture and was kept through the rework |

**What leaving stops, and what it keeps.** Leaving shows the library again through `ShowLibraryEntries`, which hides
the themed view's host. `ScheduleThemedFrame` then stops the wake timer and clears the next wake, and a frame already
asked for finds the host hidden and draws nothing. The interface's sound stream (`UiSoundPlayer`, §4.52) is disposed,
so a desktop session holds no second stream on the device. The next showing decodes the sounds again, and the next
sound opens the stream again. A direction held at the moment of leaving is let go by the pad's own poll, which does so
on every tick that the themed view is not taking the pad. The `ThemedLibrary` itself is kept, with its stage, its
capabilities and its selection, so coming back finds the same system, game and view without paying the cold first build
again (P41 in the plan). A Theme Settings sheet open at the switch stays open, and a download in it runs until the
sheet or the window is closed (§4.53).

**Which mode the next start takes.** Preferences' **Start in big screen mode** (`BigScreen`) alone decides, as Steam's
own "start in Big Picture" setting does. A switch writes nothing. The first build wrote the mode last used, which fitted
one control that was both full screen and big picture. With two controls it would make one press of Big Picture
decide every later start, which a desktop user who pressed it once to try it would not expect. It would also turn a
setting the player sets into a record the program keeps. A session the setting started in big picture can be left for
the desktop, and the setting stays on.

**Tests.** `BigPictureSwitchTests`, nine cases on WiseMan's `PadDriver` with headless keys and the menu bar's actions:
- **The two choices,** with a theme and without:
  - The View menu holds Fullscreen with F11 and, at the next index, Big Picture with F10, not checkable.
  - The pad menu offers Full Screen and Big Picture on the desktop, and no Full Screen in big picture.
  - Plain full screen by the View menu, F11 and the window manager never enters big picture and keeps the sidebar, menu
    bar and text, with the Fullscreen entry ticked; F11 and the entry come back maximised.
  - Big picture comes in by the View menu, its key and the pad menu, and the pad steers the carousel with its sound, or
    the list, its console and a search.
  - It goes out by the pad menu, Esc and the key, each back to maximised; from plain full screen, the pad menu comes
    back to plain full screen.
  - F11 and the window manager in big picture leave both.
  - After each exit the desktop has the same game, console and empty search, and `BigScreen` is untouched. There is
    one pad timer and one themed library throughout.
- **The wake:** leaving with a text in its pause, then 2.6 s of real dispatcher time, draws nothing. Entering again
  comes back to the same gamelist and draws.
- **The sound stream** is held in big picture, let go on the desktop, and taken again.
- **Windows and popups:** in big picture, Preferences opens on a sheet, no window is owned, its "Start in big screen
  mode" is still off, and the facet's popup is in the window. After leaving, Preferences is a window and the popup is
  the platform's.
- **In a game,** Esc goes to the library and back without leaving, and F11 leaves big picture with the game on screen
  and running.
- **The next start:** a window made while another is in big picture starts on the desktop. A window the setting starts
  in big picture can be left, and the setting stays on.
- **Game Mode:** no menu bar, no pad-menu entries, and F11, the Big Picture key, Esc, a window-state change and a direct call
  all leave it in big picture.
- **Process-wide popups:** only `--bigscreen` and Game Mode ask for them, with a saved big-screen setting on disk.

`DesktopButtonPictureTool`, run with `EMUSEN_BIGPICTURE_PNG=1`, writes the desktop window, its View menu open on the two
entries, plain full screen, big picture and its pad menu (synthetic theme, Art Book Next when cloned, and Mistress's own
library), and the desktop after leaving, at 1280×800, to `~/.cache/emusen/bigpicture/png/desktop-button/`.

**What it does not cover.**

- *Windows already open at a switch stay what they were.* A desktop Preferences window stays a window over big
  picture, and a sheet open when leaving stays a sheet until it is closed. Neither is lost; neither is moved.
- *No real window manager was used.* The headless platform takes every `WindowState` it is given. Whether KDE and GNOME
  report their own full-screen command through Avalonia's `WindowState` on X11 and on Wayland, and whether a normal
  window gets its exact size back, was not observed.
- *F10 is a guess at a free key.* GTK applications open their menu on F10. Mistress's key handler sees keys first and
  takes it, but a desktop that binds F10 globally would take it before Mistress. The key can be rebound.
- *The library's own view settings are shared, not given back:* grid or list, cover size, and the Library, Save States
  and Screenshots choice follow whatever was last chosen in either mode, as they are stored settings.
- *Steam's own Big Picture on the desktop* (`SteamTenfoot`, `SteamGamepadUI`) is still not read (§4.43), so a game
  started from it opens on the desktop layout until the player chooses Big Picture.
- *No pointer way out.* In big picture the menu bar is hidden, so a player with only a mouse leaves by the window
  manager's own full-screen command, which leaves big picture too. The pad menu, Esc and the key are the other ways. The
  toolbar button of the second build was a pointer's way out, and it went with the user's placement.
- *The popup embedding by the window* reaches the popups under the window. A tooltip or context menu that Avalonia
  parents elsewhere may still open as a window of its own on the desktop, where a window manager draws it at its size.

### 4.55 The application icon (2026-09-26)

**What it is.** A gold crescent, horns up, holding a cream letter E, on a deep indigo rounded square: concept A of
four drawn on 2026-09-26, chosen by the user. It nods to the crescent of *Sailor Moon*, after which EmuSen is named
(§"The names" in the README), without taking anything from it: the crescent is two plain circles subtracted, the E is
four straight bars, and no lettering, emblem or palette of the series is used. The colours are `#231E52` (ground),
`#F4C542` (moon) and `#FBF3E4` (letter).

**Two drawings, not one.** `EmuSen.Mistress/Assets/Icon/emusen.svg` draws 48 px and up. At 16–32 px its 22-unit bars
become 1.4 px and blur, so `emusen-small.svg` redraws the same mark on a 16-unit grid (every edge a multiple of 16,
so at 16 px each lands on a whole pixel): a heavier E, a thicker crescent and a full-bleed square. The first render
went through ImageMagick's default Lanczos resize and smeared that grid back into grey; `build_icons.py` now draws at
256 px or more and area-averages down (`-filter Box`), which keeps the 16 px E's edges on pixel boundaries. At 24 px
the grid falls on half pixels, and the E's spine shows a soft column there; that is accepted.

**Outputs**, all made by `python3 EmuSen.Mistress/Assets/Icon/build_icons.py` (ImageMagick 7 with librsvg) and
committed beside the sources:

| File | Used by |
| --- | --- |
| `png/emusen-{16,24,32,48,64,128,256,512,1024}.png` | the window icon (256), the Linux hicolor theme, the others |
| `emusen.ico` (16–256, seven sizes) | Windows: `ApplicationIcon`, the `.exe` in Explorer and on the taskbar |
| `emusen.icns` (eleven PNG entries, 16–1024 with the @2x types) | macOS, for an app bundle; none is built yet |

**Where it shows.**
- **Every Mistress window's frame and taskbar entry.** `App.ShowIconOnEveryWindow` registers one class handler on
  `Window.WindowOpenedEvent` that gives any window without an icon of its own the 256 px resource, so LunaP windows
  Mistress opens (Preferences, the settings sheets when they are windows) carry it too, and a window that sets its own
  keeps it. The window manager scales it to the size it needs.
- **The Windows executable**, through `<ApplicationIcon>`.
- **The Linux application menu and desktop.** A Linux publish ships `share/icons/hicolor/<size>/apps/emusen-mistress.png`
  (and the scalable SVG) and `bin/install-desktop-entry.sh`. Run once from the unpacked build, it copies the icons into
  `~/.local/share/icons/hicolor` and writes `~/.local/share/applications/emusen-mistress.desktop`, which points at
  that build's own `bin/EmuSen.Mistress`; `--desktop` adds a desktop shortcut, `--remove` takes everything away. It
  installs for the current user only and needs no root. `StartupWMClass=EmuSen.Mistress` is Avalonia's X11 default
  (the entry assembly's name), which LunaP does not change, so the running window groups under the entry. The entry
  records the build's path, so moving the folder means running the script again.

**Tests.** `AppIconTests`: the resource is a 256 × 256 image of Mistress's; a window opened after the hook shows it,
and one that set its own icon keeps its own. With the hook's handler removed the second test fails. The installer
was run against a scratch `XDG_DATA_HOME`: eight sizes and the SVG installed, a desktop entry with the quoted path
(the development tree's path has a space), a desktop shortcut, and `--remove` cleaning up.

**Not done.** Steam's own artwork for a non-Steam game (the Game Mode library's capsule, hero and logo images) is
set in Steam, not by Mistress. No macOS app bundle exists to carry the `.icns`. Hotaru and Pegasus keep Avalonia's
default icon. Whether KDE shows the frame icon from `_NET_WM_ICON` or the desktop entry's `Icon=` depends on the
entry being installed, and was not checked on a real session.

### 4.60 ScreenScraper: covers, screenshots, marquees and game text, with OpenEmu's sources as the failover (2026-09-26)

Stage (d) of `EmuSen_BigPicture.md` (its §17 is the record: predictions, the live run, mutants). ScreenScraper
(`screenscraper.fr`) is Mistress's first source of cover art and game information: the library's covers in the grid and
the list, and the themed view's pictures and metadata. OpenEmu's sources (§4.39) fill in behind it. The section is
numbered 4.60 because stage (f) was writing §4.53 at the same time.

**Where it works, and where it cannot.** Every request carries EmuSen's developer credentials, which ScreenScraper issued
to the project's author and which no build carries (the user's decision, Q5 of the plan). Mistress reads them from a file
called `screenscraper-developer.json`, holding `devid`, `devpassword` and `softname`, and looks for it in two places, in
this order:

1. the config directory, `<root>/home/etc/EmuSen/screenscraper-developer.json`, where `<root>` is the folder holding the
   `.dianaosroot` marker: for a build published to `out/linux-x64/Mistress/`, that folder; for a copy installed at
   `~/Apps/Mistress/`, that one; running from source, the checkout;
2. then `~/.config/EmuSen/screenscraper-developer.json` (the config directory from before Galaxia moved it, §1.4 of the
   config reference).

The file should be mode 0600. Where neither place has it, ScreenScraper cannot be used; the Scraping tab says so, and
OpenEmu's failover does the covers alone.

**Nothing is asked until the player starts it.** The user's rule, 2026-09-26: "I do not want to spam the screenscraper
api". No server, ScreenScraper or OpenEmu's, is asked anything when Mistress starts, when the library is shown or
refreshed, when a game is selected or shown in the themed view, when a game is added, when the developer file appears,
or because a run was left unfinished. What is already in `home/Media` and `media.db` is shown with no request. The
ScreenScraper switch (`Scraping`, on) only says ScreenScraper may be used when a run is started; it starts nothing.

**Starting a run.** A run is one of these, chosen by the player:

| Scope | Where |
|---|---|
| **this game** | the library's context menu, **Scrape This Game...**; the pad menu, **Scrape This Game...**, for the library's selected game or the themed gamelist's |
| **a console**, Game Boy Color its own shelf | Preferences ▸ Scraping ▸ **Scrape**: the console list, then **Scrape...** |
| **games with no cover**, in one console or all | the same, with **Only games with no cover** on (the default) |
| **every game** | the same, with **Every console** and **Only games with no cover** off |

The pad menu's **Scrape Games...** opens Preferences on the Scraping tab. Before a run starts, a confirm sheet gives the
count of games, how many ScreenScraper has not been asked about, the most requests it can cost against what is left
today (one lookup and one per picture kind a game), the time at about 13 seconds a game (plan §17.9), and whether
OpenEmu's sources will be asked. Declining asks nothing. While it runs, the status line and the Scraping tab show "n of
m", a bar follows it, the pad menu's entry reads "Scraping (n of m)...", and **Cancel Scraping** stops it with a request
in flight cancelled.

**Resuming.** A run that was cancelled, stopped by the quota, or cut off by closing Mistress leaves its games queued in
`media.db`. The Scraping tab says how many and offers **Resume**; nothing resumes by itself. Starting a new run replaces
that queue instead of adding to it. **Scrape This Game** asks ScreenScraper again even about a game it once did not
know, and asks the failover again too; a wider run does not re-ask what has been answered.

**From the pad.** In a big-screen session (Game Mode), Start or Guide opens the pad menu; **Scrape This Game...** and
**Scrape Games...** are there outside a game; the shoulders move between Preferences' tabs. Every row, the console
list, the switch and the Scrape, Resume and Cancel buttons are reached by the d-pad (`PadAudit`, §4.45.3) and pressed
with A. The member account's two boxes open the on-screen keyboard of §4.45.6; the password's preview above the keys
shows its mask, not the text (LunaP §110).

**The settings,** stored in `appsettings.json` except the member account:

| Row | Setting | Default | What it does |
|---|---|---|---|
| ScreenScraper | `Scraping` | on | a run the player starts may use ScreenScraper when the developer file is present; it starts nothing |
| Member Account | `screenscraper.json` (`ssid`, `sspassword`) | none | optional; a free account at screenscraper.fr, whose contributions or donation raise the day's requests and threads. Its own file in the config directory, mode 0600, written when a box is left or the sheet closes; never in `appsettings.json` |
| Fetch: Covers | `ScrapeCovers` | on | ScreenScraper's `box-2D` |
| Fetch: Screenshots | `ScrapeScreenshots` | on | `ss` |
| Fetch: Marquees | `ScrapeMarquees` | on | `wheel-hd`, else `wheel` |
| Fetch: Mix images | `ScrapeMiximages` | on | `mixrbv2`, ScreenScraper's ready-made mix, as the theme's miximage (Q7) |
| Fetch: Title screens | `ScrapeTitleScreens` | off | `sstitle` |
| Region | `ScrapeRegion` | `auto` | whose box and name are preferred; Automatic reads the file's No-Intro tag, (USA) us, (Europe) eu, (Japan) jp, (World) wor, and the other country names ScreenScraper has a code for |
| Else world, USA, Europe, Japan, then any | `ScrapeRegionFallback` | on | ES-DE's documented fallback order, then any region; off, only the preferred region (and a file with no region) is taken |
| Language | `ScrapeLanguage` | `en` | the description's and genre's; English when there is none in it |
| (no row) | `ScrapeThreads` | 1 | the workers wanted; never more than the member's `maxthreads` |
| OpenEmu Failover | `OpenEmuFallback` | on | below |

A change takes effect when the sheet closes.

**What is sent, and to whom.** For each game: the file's name (without its folder), its size, its MD5, CRC32 and SHA-1,
and ScreenScraper's system number (NES 3, SNES 4, Game Boy 9, Game Boy Color 10, Nintendo 64 14), with the developer
credentials and the member account when there is one. ScreenScraper therefore sees which games the player has. What comes
back is written by its contributors, and the art is its publishers'; Mistress keeps it for the player and shares none of
it.

**How a game is looked up**, inside a run. Only the file's own hashes are asked first; if ScreenScraper does not know them and the
console's core declares another form of the bytes (the NES without its iNES header, the SNES without a copier header, the
N64 halfword-swapped: `CoreDescriptor.OpenVgdbBytes`), those hashes are asked once more. There is no search by name. A
game answered once is not asked again: a renamed file takes its pictures to its new name, and a copy of it gets copies,
with no request. A whole library is days of a free account's quota (plan §17.10).

**Where things are kept.** `home/Media/`, laid out as ES-DE's `downloaded_media`: `<system>/covers`, `screenshots`,
`marquees`, `miximages`, `titlescreens`, each file named after the ROM's own file name. Beside them `media.db` holds each
game's text by its MD5 and size, which file each picture is, the queue, and the day's counts (with `PRAGMA user_version`
and a newer file refused, as `games.db` is). A picture is written beside its final name and moved into place, never over
a file already there, and only when the server calls it an image of at least 80 bytes. Nothing is ever written in the
ROM folder. Deleting `home/Media` loses what was fetched and nothing else.

**The order a picture is looked for in**, the same in the library and the themed view:

1. what the player placed: the cover art folder of §4.33, or **Add Cover Art from File…**;
2. ScreenScraper's, in `home/Media/<system>/`;
3. the ES-DE media folder of §4.52, when one is set;
4. OpenEmu's failover's, in `home/Media/openemu/<console>/`, only while the failover is on;
5. the placeholder of §4.33.

Screenshots, marquees and the rest take 2 then 3; only covers have a player's folder and a failover. A cover the player
already has is not fetched from ScreenScraper.

**OpenEmu's failover** (`OpenEmuFallback`, on, the coordinator's choice on 2026-09-26, which the user may reverse) is
asked only inside a run the player started, never for a tile being drawn as §4.39's switch once did. It asks OpenVGDB and
libretro's thumbnails (§4.39) for a cover in exactly these cases:

- ScreenScraper answered, and had no cover: the game is unknown to it, it has no `box-2D`, or the lookup failed for good;
- ScreenScraper cannot be used: no developer file, the ScreenScraper switch or its Covers switch off, the day's quota used
  up (its limit less 2%, or a 430 or 431 answer), the service closed (423), the developer credentials refused (403), or
  this build blocked (426).

A game ScreenScraper has queued in the run waits for its answer first; when a run stops on the quota, the games it did
not reach go to the failover. Its hint says what it sends and to whom: OpenVGDB's 9 MB database from GitHub the first
time, then the game's name to thumbnails.libretro.com and, last, to the address OpenVGDB gives. §4.39's **Look Up Cover
Online** is now **Scrape This Game...**.

*The old setting.* `OnlineCovers`, the §4.39 switch, is read from an older `appsettings.json` and dropped at the next save.
The failover is on after the upgrade whatever it held: a stored false was the default every save wrote, and cannot be
told from a choice. Covers the old switch fetched into the cover art folder stay there and now count as the player's own.

**The quota**, which ScreenScraper requires software to manage itself. Every answer carries the member's limits and
today's counts; Mistress reads them from every answer and:

- never runs more workers than `maxthreads` (one until an answer says);
- spaces requests, the media downloads included, at `maxrequestspermin` less 10% (30 a minute until an answer says);
- waits after a download that came faster than `maxdownloadspeed`;
- stops for the day at `maxrequestsperday` less 2%, or `maxrequestskoperday` less 2% for unrecognised games, or on a 430
  or 431 answer, and resumes the next day (taken as Paris's, where the service runs) where it stopped; the stop is kept
  in `media.db` and outlives a restart;
- on a 429 halves its pace and waits a minute; on a 401 waits five minutes;
- on a 403, 423 or 426 stops until the next start or the next change in Preferences, and says why in the status line.

The Scraping tab's **Today** row shows the day's requests against the limit, the unrecognised ones, the threads, and
whether it is running, how many games are queued, or why it stopped and until when. It fills from the first answer of a
session; until then it shows no bar.

*Measured on 2026-09-26 without a member account* (plan §17.9): 1 thread, 128 KB/s, 10,000 requests a day and 1,000
unrecognised. **Every picture is a request**, as the game's lookup is; with the default kinds a found game costs about
4.7 requests and 13 seconds, half of it the 128 KB/s allowance, so a 5,520-file library takes three days of that quota.
Turning **Mix images** off saves the largest of the files; a member account raises all of it.

**The credentials never leave.** Neither credential file is ever committed: `.gitignore` names both, and a WiseMan test
fails if either is tracked or if any tracked file holds a `devpassword=` value that is not a placeholder, or the
developer's real password when its file is on the machine. Every address or message that could reach the status line, a
log, a crash report or an exception passes through one redactor that blanks `devid`, `devpassword`, `ssid` and
`sspassword` (and the four values wherever they appear); `CrashLog` writes through it. A published build's zip must not
carry `home/etc/EmuSen/screenscraper*.json` or `home/Media`, as it already leaves out the sandbox's cheats and saves.

**Everything stops with the window.** Closing Mistress's window stops the workers (a request in flight is cancelled),
closes `media.db` and the refresh timer, before the HTTP client they share is disposed; Preferences lets go of the window
when it closes.

**Tests,** all on a fake ScreenScraper written for them (no test reaches the network, and the suite's windows start with
no network and no developer file unless a test installs its own): `ScrapeRulesTests`, `ScreenScraperClientTests`,
`ScrapeQuotaTests`, `MediaStoreTests` (with the order of sources), `ScraperTests`, `ScrapeCredentialTests` (the
redactor, the two places, the member file's mode, never in git), `CrashLogTests`, `ScrapeWindowTests` (each failover
case, the window closing, Preferences, the old setting) and `ThemedScrapeTests`; `OnlineCoverTests` and
`OnlineCoverWindowTests` keep §4.39's rules. Mutants and the live run are in the plan's §17.

**What it does not cover.**
- No video (Q4), no back cover, fan art, 3D box or physical media: ES-DE's folders for them exist, nothing fills them.
- The game's name stays the file's; ScreenScraper's is kept in `media.db` but not shown.
- A game found without a cover because the player had one, whose cover is later removed, goes to the failover rather
  than back to ScreenScraper.
- No "refresh": a picture already there is never fetched again, so a better one at ScreenScraper is not seen; delete the
  file to have it fetched.
- No search by name for a game the hashes miss.
- Nothing ran on the handheld.
