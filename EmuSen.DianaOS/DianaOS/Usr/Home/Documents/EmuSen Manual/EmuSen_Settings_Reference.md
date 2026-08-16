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

`MainWindow` captures gameplay input the same way and for the same reason — four of the twelve default game-button bindings are the arrow keys, which the focus manager claims for directional navigation whenever anything focusable (the menu bar) has focus. That capture had a cost of its own once a text box appeared on the same window; see §4.17.

**Regression coverage**: `EmuSen.WiseMan/Mistress/InputSettingsWindowTests.cs` drives real Avalonia key events through `Avalonia.Headless` rather than calling the handler directly — a direct call passes against the broken build, because the bug is purely in routing. Reverting the single `AddHandler` line back to `KeyDown +=` fails 3 of its 12 tests (`Enter`, `Space`, and the re-arm check). Note the arrow-key half of the problem does **not** reproduce headlessly, since headless has no real focus-navigation pass; it is reasoned from Avalonia's routing, not measured.

### 4.3 Hotkeys are bindings too (`Input/HotkeyBindingMap.cs`)

Fast-forward and rewind used to be hardcoded to `Tab` and `Backspace` inside `MainWindow.SetButtonFromKey`, undiscoverable and unchangeable. They're now entries in a `HotkeyAction` map with the same shape, persistence and rebind rules as `ControllerKeyMap`, and they appear as their own section in the settings window. Defaults keep `Tab`/`Backspace` so existing muscle memory survives, and add `F5`/`F8` save/load state, `P` pause, `F11` fullscreen, and `Escape` to step out of a game (§4.18).

`HotkeyBindingMap.IsHeld` splits the two shapes of action: fast-forward and rewind apply *while the key is down* (see `EmuSen_Rewind_And_FastForward.md` §4), everything else fires once on the press. Getting that wrong makes save-state fire once per frame while held.

The two maps police **each other** on rebind, not just themselves: one key doing both a game button and a hotkey would fire both at once. Binding a key that's in use anywhere clears it from wherever it was.

### 4.4 Gamepad options (`Input/GamepadManager.cs`, `EmuSen.Galaxia/Models/AppSettings.cs`)

- **`AnalogStickAsDpad`** (default on) — the left stick reports as the d-pad directions. No SNES game reads an analog axis, so an unmapped stick is simply dead input, which reads as a broken controller.
- **`StickDeadzone`** (default 0.5, clamped 0.05-0.95) — fraction of full deflection before a direction registers. Exposed because worn sticks drift, and a drifting stick mapped onto the d-pad walks the player into walls.
- **`ControllerName`/`IsConnected`** — surfaced in the window so "no controller detected" can be told apart from "connected but bound wrong". The window re-polls once a second, so hot-plugging is visible without reopening it.

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

The slot submenu's eight actions are built once and re-labelled on every `SyncMenuState`, so each entry shows that slot's file timestamp (or `empty`) as it stands right now, including for a slot saved a moment ago — which is why `SaveState` calls the sync after a successful write. *Before 2026-08-16 this was a rebuild on `SubmenuOpened`, with a second build at construction because a `MenuItem` with no children renders as a leaf with no submenu arrow. `LunaAction.Submenu` is set once and the arrow follows it, so that second build is gone.* Picking a slot only *selects* it; `Save State`/`Load State` then act on it, and both re-label themselves with the slot number. Two clicks to save to a specific slot is the RetroArch model, and it keeps the hotkeys meaningful — an `F5` that could not know which slot you meant would be worse than one that follows the selection.

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

**Menu shortcuts were deliberately not added.** `LunaAction.Shortcut` would draw and bind a key per menu entry, but Mistress's keys come from `HotkeyBindingMap` and are user-rebindable (§4.3). Two independent bindings for one command is how a menu ends up advertising a key that a rebind has since moved. If the menu should show its keys, the labels must be driven *from* `HotkeyBindingMap`, and that is a separate piece of work.
