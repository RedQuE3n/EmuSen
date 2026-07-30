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

`Settings/SettingsPaths.cs` is the single answer to "where does this frontend keep config". `ControllerKeyMap`, `GamepadBindingMap`, `HotkeyBindingMap` and `AppSettings` all resolve their own file through it (`keybindings.json`, `gamepadbindings.json`, `hotkeybindings.json`, `appsettings.json`), rather than each rebuilding an `%AppData%/EmuSen` path of its own.

It exists because those paths used to be hardcoded per class, which made them untestable: any test that rebound a key wrote the developer's real config, since rebinding saves immediately. `SettingsPaths.OverrideDirectory` redirects the whole set at once and is set only by tests.

### 4.2 Fixed bug: rebinding a key appeared to do nothing

The rebind flow is "click Rebind, then press a key" — the window listens for the next key and writes it into the map. It captured that key with a plain `KeyDown += ...`, which in Avalonia is the **bubbling** pass: the event reaches the focused control first and the window last.

The focused control at that moment is always the *Rebind button the user just clicked*. A focused `Button` handles `Enter` and `Space` itself as activation keys and marks them handled, so neither ever reached the window and the binding silently didn't change — the row kept saying "Press a key..." forever. Worse, the activation re-fired the button's own `Click`, re-arming the same row, so the window could never leave listening state. `Start` defaults to `Enter`, so anyone rebinding Start hit this immediately.

Fixed by capturing on the **tunnel** pass instead, which runs top-level-first:

```csharp
AddHandler(KeyDownEvent, OnPreviewKeyDown, RoutingStrategies.Tunnel, handledEventsToo: true);
```

and setting `e.Handled = true` once the key is consumed, so it never reaches the button underneath. `handledEventsToo` matters for keys something upstream has already claimed.

`MainWindow` captures gameplay input the same way and for the same reason — four of the twelve default game-button bindings are the arrow keys, which the focus manager claims for directional navigation whenever anything focusable (the menu bar) has focus.

**Regression coverage**: `EmuSen.WiseMan/Mistress/InputSettingsWindowTests.cs` drives real Avalonia key events through `Avalonia.Headless` rather than calling the handler directly — a direct call passes against the broken build, because the bug is purely in routing. Reverting the single `AddHandler` line back to `KeyDown +=` fails 3 of its 12 tests (`Enter`, `Space`, and the re-arm check). Note the arrow-key half of the problem does **not** reproduce headlessly, since headless has no real focus-navigation pass; it is reasoned from Avalonia's routing, not measured.

### 4.3 Hotkeys are bindings too (`Input/HotkeyBindingMap.cs`)

Fast-forward and rewind used to be hardcoded to `Tab` and `Backspace` inside `MainWindow.SetButtonFromKey`, undiscoverable and unchangeable. They're now entries in a `HotkeyAction` map with the same shape, persistence and rebind rules as `ControllerKeyMap`, and they appear as their own section in the settings window. Defaults keep `Tab`/`Backspace` so existing muscle memory survives, and add `F5`/`F8` save/load state, `P` pause, `F11` fullscreen.

`HotkeyBindingMap.IsHeld` splits the two shapes of action: fast-forward and rewind apply *while the key is down* (see `EmuSen_Rewind_And_FastForward.md` §4), everything else fires once on the press. Getting that wrong makes save-state fire once per frame while held.

The two maps police **each other** on rebind, not just themselves: one key doing both a game button and a hotkey would fire both at once. Binding a key that's in use anywhere clears it from wherever it was.

### 4.4 Gamepad options (`Input/GamepadManager.cs`, `Settings/AppSettings.cs`)

- **`AnalogStickAsDpad`** (default on) — the left stick reports as the d-pad directions. No SNES game reads an analog axis, so an unmapped stick is simply dead input, which reads as a broken controller.
- **`StickDeadzone`** (default 0.5, clamped 0.05-0.95) — fraction of full deflection before a direction registers. Exposed because worn sticks drift, and a drifting stick mapped onto the d-pad walks the player into walls.
- **`ControllerName`/`IsConnected`** — surfaced in the window so "no controller detected" can be told apart from "connected but bound wrong". The window re-polls once a second, so hot-plugging is visible without reopening it.

### 4.5 Conflict reporting

`Rebind` guarantees one owner per key going forward, but a hand-edited or older config can still contain duplicates. The window recomputes conflicts after every change, colours the offending rows, and names them in a status line along the bottom rather than leaving the user to work out why one key does two things.

**Clear the property, don't null it.** `MarkConflict` un-highlights a row with `label.ClearValue(TextBlock.ForegroundProperty)`, not `label.Foreground = null`. The second sets a *local* null brush, and a `TextBlock` with no brush paints nothing at all — that blanked the entire Keyboard column while every assertion-based test still passed, since `Text` was correct throughout. Caught only by rendering the window (§4.7).

### 4.6 How the window is built

**One scrolling page, not tabs.** Every row stays in the visual tree, so conflict detection sees all of them at once and no clashing binding can hide behind an unselected tab.

**Rows are built in code, not XAML**, because they're one per `SnesButton`/`HotkeyAction` enum member. Column widths are shared by hand between `BuildButtonRows`/`BuildHotkeyRows` and the header `Grid`s in the `.axaml`; changing one means changing the other. Value labels use `TextTrimming.CharacterEllipsis` because `Key` and `GameControllerButton` names (`RightBracket`, `Leftshoulder`) routinely overflow their column.

**Pad names are cleaned for display.** Silk.NET's `GameControllerButton` spells some members `ControllerButtonX` and others plainly (`Start`, `DpadUp`), so `PadName` strips a leading `ControllerButton` prefix to stop the column reading as a mix of both.

**`_initialized` guards the option handlers.** `IsCheckedChanged`/`ValueChanged` are wired in the `.axaml`, which means Avalonia raises them from *inside* `InitializeComponent()` — before the constructor has assigned `_appSettings` or anything else. Setting `Minimum`/`Maximum` on the deadzone `Slider` coerces its value and fires `ValueChanged` on the spot, so every handler returns early until the constructor finishes. Without the guard the window throws `NullReferenceException` on construction.

**`PollForPadButton` null-checks `_gamepad` itself** rather than relying on `StartListeningForPad` having done so. The invariant otherwise lives in the caller where the compiler can't see it (CS8602), and checking locally also means the 50 ms poll timer stops itself if the pad disappears mid-rebind.

### 4.7 Test coverage

`EmuSen.WiseMan/Mistress/` holds two files, both running on the shared `HeadlessUnitTestSession` from `Serenity/TestAppBuilder.cs`.

`InputSettingsWindowTests.cs` drives **real Avalonia key events** through `Avalonia.Headless` rather than calling the capture handler directly — the §4.2 bug is purely in event routing, so a direct call passes against the broken build. `ClickAsUser` focuses a button before raising `Click`, because that focus is exactly what used to swallow the follow-up key. Reverting the single `AddHandler` line back to `KeyDown +=` fails 3 of its tests (`Enter`, `Space`, and the re-arm check).

Two gotchas worth knowing before adding cases here:

- **Match the row by its name cell (column 0) only.** A loose "any `TextBlock` in this row" match picks the wrong row: `Y`'s default keyboard binding is literally `A`, so searching for row "A" finds `Y`'s *value* column first.
- **Redirect `SettingsPaths.OverrideDirectory`** in the fixture. Rebinding saves immediately, so without it the suite overwrites the developer's real bindings.

The arrow-key half of §4.2 does **not** reproduce headlessly — headless has no real focus-navigation pass — so it is reasoned from Avalonia's routing rather than measured, and the theory cases for `Up`/`Left` pass either way.

`InputSettingsWindowRenderTests.cs` renders the window through Avalonia's real Skia pass and asserts the result isn't one flat colour, which catches an unparseable `.axaml` or a collapsed layout. Set `EMUSEN_UI_DUMP=/path/to.bmp` to also write the capture out and look at it — that is how the blank-column bug in §4.5 was found.

### 4.8 `EmuSen.Hotaru` suppresses AVLN3001

Hotaru's `.csproj` carries `<NoWarn>$(NoWarn);AVLN3001</NoWarn>` — "XAML resource won't be reachable via runtime loader, as no public constructor was found", for `App.axaml` and `Views/GameWindow.axaml`.

Both types are constructed by hand with their real dependencies: `Program.cs` resolves a ROM and builds the core *before* Avalonia starts, then hands them over via `AppBuilder.Configure<App>(() => new App(...))` and `new GameWindow(core, ...)`. URI-based instantiation (`AvaloniaXamlLoader.Load(uri)`) is a capability this frontend never uses; `App.Initialize()` calls the *instance* overload, which needs no parameterless constructor.

Adding one is not an improvement. `GameWindow`'s constructor starts the emulation thread, the console-reader thread, a 60 Hz gamepad timer and an SDL handle, so a parameterless overload chaining into it would spawn all of that from the previewer — and one that skipped it would leave the class's `readonly` fields unassigned, trading this warning for a fistful of CS8618s and a half-built window.

Scoped to that one project deliberately. `EmuSen.Mistress` satisfies the analyzer naturally (its `MainWindow` and `InputSettingsWindow` both have real parameterless constructors) and keeps the warning live.

### 4.9 Publishing a frontend that actually starts

Two things have to be right or a published build dies on launch, and neither shows up when testing on a development machine.

**SDL2 must be shipped, not assumed.** `Silk.NET.SDL` is bindings only — it contains no native binary. `GamepadManager` and `AudioPlayer` both call `Sdl.GetApi()` from `MainWindow`'s constructor, so without a loadable SDL2 the app throws `FileNotFoundException: Could not load from any of the possible library names!` before a window ever appears. The native lives in a separate package, **`Ultz.Native.SDL`** (Ultz is the Silk.NET org; there is no `Silk.NET.SDL.Native`), which carries `SDL2.dll`, `libSDL2-2.0.so` and a universal `libSDL2-2.0.dylib`. Both `EmuSen.Mistress` and `EmuSen.Hotaru` reference it.

A Linux dev box usually has SDL2 installed system-wide, so this fault is invisible locally and appears only on a clean Windows or macOS machine.

**Do not publish `-p:PublishSingleFile=true`.** It produces one tidy executable that then fails to resolve SDL2 even with the native package referenced — Silk.NET's own loader does not find the library the single-file host extracts. A plain self-contained folder publish puts `libSDL2-2.0.so` next to the executable where it resolves normally. Verified by launching both, same machine, same commit: folder publish runs, single-file throws at `Sdl.GetApi()`.

So the working recipe per RID is:

```
dotnet publish <app>/<app>.csproj -c Release -r <rid> --self-contained true \
  -p:DebugType=none -p:ErrorOnDuplicatePublishOutputFiles=false -o <out>
```

`ErrorOnDuplicatePublishOutputFiles=false` is needed because the `EmuSen.DianaOS` executable project reference gets built twice (§ its own csproj comment), emitting `EmuSen.DianaOS.runtimeconfig.json` from both the target RID and the host.

**Verifying a build launches.** Running it with no `DISPLAY` is *not* a launch test: Avalonia dies at X11 initialisation before `MainWindow`'s constructor runs, so everything SDL-related is never reached and a broken build looks fine. Launch it on a real display and check it is still alive several seconds later.

**macOS cannot be finished from Linux.** Apple Silicon refuses to execute an unsigned arm64 binary outright, so clearing quarantine is not enough; the user must ad-hoc sign (`codesign --force --deep --sign - <name>.app`). There is no signing tool on the Linux build machine.
