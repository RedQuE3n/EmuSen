# EmuSen.LunaP — the shared Avalonia toolkit

*This revision (2026-08-09): **a theme may now be written in CSS**, and building it turned up an Avalonia behaviour that had been sitting under the theme system unnoticed — mutating `Application.Styles` at runtime strips every already-realized control of its styling, LunaP's own included. §12.2 is the format, §12.3 is the finding and the reattach that answers it. Previous revision (2026-08-04): Phase 7 — LunaP is themeable and has its widget set; user themes load from `/etc/EmuSen/themes`, and dropdowns, switches, tabs and filter bars are in use across the settings windows, the library and the cheat database. All seven phases are done. Before that (2026-08-04): Phase 6, the migration; 863 lines net removed from the frontends. `EmuSen_LunaP_Gameplan.md` holds the plan and what each phase taught. This doc covers only what is real.*

---

## 1. What it is, and the rule that keeps it useful

`EmuSen.LunaP` is the shared Avalonia toolkit for `EmuSen.Mistress`, `EmuSen.Hotaru` and the future launcher: theme, controls, window scaffolding, and a fluent layout surface. It is the chrome *around* the game picture; `EmuSen.Serenity` owns the picture itself and the two do not overlap.

**The layering rule is load-bearing and is the reason the project is worth having:**

> LunaP may reference **Avalonia** and **`EmuSen.Galaxia`** and nothing else. Not `EmuSen` (the core), not `EmuSen.DianaOS`, not `EmuSen.Cauldron`, not `EmuSen.Endymion`.

The launcher's entire value is browsing a library with no core loaded. One upward reference here hands it the whole emulator, permanently. `EmuSen.Serenity` already holds this line on the video side — it presents frames while taking `(byte[] rgba, int width, int height)` rather than an `ICore` — and **every LunaP control takes plain data or a delegate for the same reason.** A meter row takes `(string, double, string)`, never a `DebugLoadInfo`. A console pane takes a `Func<string, string>`, never a `DianaOSInterpreter`.

This is also why the two frontends share *widgets* but not *windows*: `CoretopWindow` consumes `ICoreTelemetry`, so it stays a file in each frontend even though almost everything inside it is now shared. See `EmuSen_LunaP_Gameplan.md` §2.

The name is Luna-P, Chibiusa's floating gadget ball, which becomes whichever tool is needed. It is **not** `Luna`, the reserved codename for the Nintendo DS core — see `EmuSen_Core_Naming_Scheme.md` §9.

---

## 2. The theme

### 2.1 Two halves, and the test that pins them

The palette is spelled twice on purpose:

- **`Theme/Palette.axaml`** — a `ResourceDictionary` of `Color` and `SolidColorBrush` resources, for XAML (`Foreground="{DynamicResource LunaText}"`).
- **`Theme/LunaPalette.cs`** — the same values as `ImmutableSolidColorBrush` statics, for controls built in C# (`Foreground = LunaPalette.MeterText`).

XAML cannot read C# constants and a static cannot cheaply resolve an app resource before the app exists, so one of the two has to be a copy. **`EmuSen.WiseMan/LunaP/LunaPaletteTests.cs` is what stops them drifting**: it resolves every key out of the live headless application and asserts the resolved brush equals the C# field, colour for colour. Add a colour to one half without the other and that test fails immediately. It also serves as a smoke test that the whole `StyleInclude` → `Styles.Resources` → `TryGetResource` chain works at all — if `LunaTheme.axaml` ever stops reaching `Application.Styles`, every key fails to resolve and the first assertion says so by name.

**Every value in the palette is a literal that was already in the codebase.** Nothing here was chosen; the audit in `EmuSen_LunaP_Gameplan.md` §1.1 is where each one came from.

| Key | Value | Role |
|---|---|---|
| `LunaSurface` | `#1E1E1E` | tool-window background |
| `LunaInputSurface` | `#252526` | text-input background |
| `LunaVoid` | `#000000` | letterbox/no-signal area behind a game frame |
| `LunaText` | `#D4D4D4` | body and monospace text |
| `LunaMeterText` | `#DCDCDC` | meter-row labels and values |
| `LunaMuted` | `#808080` | hints, group headers, disabled captions |
| `LunaSectionHeader` | `#9CDCFE` | section headings |
| `LunaWarning` | `#D08770` | inline caution text |
| `LunaNominal` / `LunaBusy` / `LunaHot` | `#32CD32` / `#FFD700` / `#FF4500` | the load ramp, §2.2 |
| `LunaMonoFont` | `Consolas,Menlo,monospace` | the monospace stack |
| `LunaHintFontSize` / `LunaHeaderFontSize` | `11` / `14` | |

**Why `LunaText` and `LunaMeterText` are still two greys, four steps apart.** They are the same role — body text — and they should almost certainly converge. But `#D4D4D4` was what the XAML said and `#DCDCDC` (`Brushes.Gainsboro`) was what the meter-building code said, and merging them would have changed rendered pixels during a phase whose whole guarantee was that nothing changed. **Converging them is a deliberate one-line decision for whoever builds `MeterRow` in Phase 2**, not something to do silently.

One brush was deliberately *not* absorbed: `InputSettingsWindow`'s conflict highlight is still a raw `Brushes.OrangeRed`. It happens to equal `LunaHot`, but "this binding collides with another" is not "this subsystem is at 85% load", and giving them one key would encode a relationship that does not exist.

### 2.2 The load ramp

`LunaPalette.ForLoad(percent)` returns green below 60, gold from 60, orange-red from 85. Those thresholds came from three separate hand-written `ColorForPercent` copies (both `CoretopWindow`s and `VstopWindow`) that agreed by luck; all three are deleted and call this instead. `LunaPaletteTests` pins the boundaries at 59.9/60 and 84.9/85 so a future edit cannot quietly shift them.

The convention deliberately matches DianaOS's own terminal `ColoredBar`, so the GUI dashboards and `coretop` in a real terminal never disagree about what counts as hot.

---

## 3. Bootstrap

`LunaApp.Configure<TApp>()` is the single Avalonia startup sequence: `UsePlatformDetect()`, `WithInterFont()`, `LogToTrace()`, and `UseX11()` on Linux (which `UsePlatformDetect` does not select on its own under a Wayland session — see `EmuSen_Project_Overview_v2.md` §2a). There is an overload taking a `Func<TApp>` because `EmuSen.Hotaru` constructs its `App` by hand: its `Main` has to fully resolve a ROM and build a core before any Avalonia type is touched.

Both `Program.cs` files are now one expression each.

### 3.1 Why the theme include matters to the test harness

`EmuSen.WiseMan/Serenity/TestAppBuilder.cs` used to hand-build `FluentTheme` + `ThemeVariant.Dark`, because it could not reference either frontend's `App.axaml`. Its own comment recorded the hazard: **without a theme, templated controls have no template, render as nothing, and every render assertion over them silently passes.** That is a bug class produced entirely by having no shared application setup — a divergence between the harness's theme and the real one could not be detected by any test, because the failure mode is a test that passes.

The harness now includes the same `avares://EmuSen.LunaP/Theme/LunaTheme.axaml` the frontends do, so there is one theme and no way for them to disagree.

---

## 4. What Phase 1 changed, and how "no visual change" was verified

Mechanically: every theme literal in both frontends became a resource reference (`#1E1E1E` ×5, `#D4D4D4` ×18, `#9CDCFE` ×15, `#252526` ×2, `#D08770`, `Gray` ×22, the monospace stack ×16), every code-behind theme brush became a `LunaPalette` static, and three `ColorForPercent` copies were deleted.

`grep -rn '#[0-9A-Fa-f]\{6\}\|"Gray"\|Consolas' --include=*.axaml EmuSen.Mistress EmuSen.Hotaru` now returns nothing. That is the check to re-run before claiming the palette is still centralised.

**The "pixel-identical" claim was measured, not asserted.** A `git worktree` of the pre-change commit and the current tree each rendered `CoretopWindow`, `DebugSettingsWindow` and `PreferencesWindow` through the headless Skia session, dumping raw RGBA. All three pairs compared **byte-for-byte identical** across 3,144,000 bytes. `VstopWindow` was deliberately excluded from that comparison rather than trusted: it prints live pid, uptime and CPU figures, so its pixels differ between any two runs and it can never be a reliable regression target.

The dump harness was a throwaway and was deleted. **Phase 5 of the gameplan is where it should come back properly**, as `UiTest.Capture` in WiseMan — a reusable golden-image comparison is worth having and hand-rolling it per migration is not.

### 4.1 One thing left alone

`ActiveCheatsWindow`'s cheat-detail column uses `FontFamily="monospace"` — the bare family, not the `Consolas,Menlo,monospace` stack everything else uses. Converting it would have changed which font actually resolves, and therefore pixels. It is a real (tiny) inconsistency, left for whoever migrates that window in Phase 6 to fix deliberately.

---

## 5. The control kit

Eleven controls in `Controls/`, all styled from `Theme/Controls.axaml`, all usable from XAML (`xmlns:luna="clr-namespace:EmuSen.LunaP.Controls;assembly=EmuSen.LunaP"`) and from C#. None of them names a core, a telemetry type or a DianaOS type — §1.

**Every control has a test that asserts its style actually applied**, because the failure mode of a style that stops matching is not an exception, it is a control that renders as plain or as nothing at all. §5.5 is a real instance of exactly that, caught by exactly that.

### 5.1 Text — `SectionHeader`, `HintText`, `MonoText`

Three `TextBlock` subclasses carrying no code at all; the whole definition is a style. `SectionHeader` is the blue bold heading (×15 in the old XAML), `HintText` the grey 11 pt wrapping explanation (×22), `MonoText` the monospace body used for register dumps and runtime figures (×16).

### 5.2 Meters — `MeterRow`, `MeterList`, `MeterEntry`

`MeterRow` is a label / percentage bar / value in a `140,*,55` grid — the exact layout all three hand-written `BuildMeterRow` copies used. Setting `Percent` recomputes `BarBrush` through `LunaPalette.ForLoad`, so the ramp cannot be forgotten at a call site.

`MeterList` takes an `IReadOnlyList<MeterEntry>` and rebuilds its rows wholesale on every assignment. **That is deliberate and is not the waste it looks like**: the original code rebuilt its rows from scratch four times a second on purpose, because a handful of cheap control allocations at 4 Hz is far simpler than diffing and updating a cached control per entry, and the refresh rate makes the cost irrelevant. Phase 3's "suspend the timer while hidden" removes even that.

**Grouping deliberately stays with the caller.** `coretop` groups its load bars by kind and labels each group with `DebugLoadKindText.Header(...)` — DianaOS vocabulary, which §1 forbids here. A window that needs groups emits a `SectionHeader` and a `MeterList` per group.

### 5.3 `RgbaImageView`

Takes a raw RGBA buffer and shows it, **reusing its `WriteableBitmap` across frames and reallocating only when the dimensions change**. Of the three implementations this replaces, only `FeedWindow`'s did that; both `CoretopWindow`s allocated a fresh bitmap on every one of their 4 Hz ticks. The better implementation is now the only one.

Since writing pixels does not change the bitmap instance, nothing downstream would know to repaint — the control invalidates its own `Image` part explicitly. A `0×0` buffer, or one shorter than `width * height * 4`, clears the view instead of throwing: "no tile memory" is a legitimate answer from a core, not an error.

### 5.4 Settings fields — `FieldRow`, `PathPickerRow`

`FieldRow` is bold label / optional grey hint / content, and collapses the hint entirely when it is empty rather than reserving blank space. `PathPickerRow` is the read-only path box plus `Browse...` button that appeared four times, wired to §6's pickers; it raises `PathPicked` only on a real selection, never on a cancel.

### 5.5 Bars — `StatusBar`, `ButtonBar`, and a trap worth knowing

`StatusBar` is the bottom strip: status text left, content right. `ButtonBar` is a right-aligned run of buttons.

**`ButtonBar` initially rendered as nothing, and this is the single most useful thing learned in Phase 2.** It derives from `ItemsControl`, and in Avalonia a control's *style key* defaults to its own runtime type — so `FluentTheme`'s `ControlTheme` for `ItemsControl` does not reach a subclass of `ItemsControl`. No template, no `ItemsPresenter`, no items, no error. It looked like a working control that simply had nothing in it.

The fix is to template it explicitly rather than inherit (the alternative, overriding `StyleKeyOverride` to point back at the base type, works too but silently re-couples the control's look to whatever the Fluent theme does next). **Anything added to this kit that derives from a templated Avalonia control needs its own `Template` setter and a test that finds a real part in the visual tree** — asserting on a property alone would have passed here.

### 5.6 `ConsolePane`

The terminal-shaped pane: scrolling output, prompt, input box, and Up/Down history recall — the whole of the byte-identical XAML plus the recall state machine both console windows had. It knows nothing about DianaOS:

- `Submitted` is an `Action<string>`; running the line is the caller's business.
- `HistorySource` is a `Func<IReadOnlyList<string>>`, which is exactly the seam the two callers need — Mistress reads its interpreter's history, Hotaru reads the *live core's* history once a game attaches.

**Output is held in the control, not in the `TextBlock`.** Both console windows print a welcome banner from their constructor, long before a template exists, so writing straight to the template part would have silently dropped it — a bug that would have appeared in Phase 6 as "the banner is gone" with nothing to point at. The pane buffers and flushes on `OnApplyTemplate`, and a test pins it.

---

## 6. Pickers

`Windowing/Dialogs.cs` wraps `StorageProvider` for folder, open-file and save-file selection, resolving the `TopLevel` itself so a caller passes only a control. A start location that no longer exists is not an error — the picker just opens where it would have anyway.

**This was pulled forward from Phase 3**, where the gameplan filed it, because `PathPickerRow` is meaningless without it. The rest of that phase's `Dialogs` — `ConfirmAsync`/`ErrorAsync` — genuinely does belong with the window scaffolding, since those need a window of our own rather than an OS dialog.

---

## 7. The gallery

`Gallery/GalleryWindow.cs` is every control once, with sample data, built entirely in C# — which also dogfoods the claim that the kit does not require XAML. Two tests cover it: one asserting each control type is realised in the visual tree, one taking a real Skia render pass and asserting the result is not a flat image.

That second test is the cheap net for the §5.5 failure mode across the whole kit at once. Its threshold is deliberately far above the ">8 distinct colours" the older window tests use, because the gallery's image-view ramp alone contributes hundreds; a templating failure collapses it to a handful.

The gallery ships in Release. It is ~110 lines with no dependencies beyond the kit, and a widget library without a visible reference page is much harder to extend correctly than one with.

---

## 8. The windowing layer

`Windowing/` is where the kit stops being widgets and starts being a framework. Still nothing consumes it — Phase 6 does that.

### 9.1 `ToolWindow`

The base class, and **deliberately thin: both of its features are opt-in, so inheriting it changes nothing by itself.** That was a design choice, not an oversight. Phase 6 rewrites a dozen windows onto this base, and a base class that silently altered how they close or where they open would make every one of those migrations a behaviour change hiding inside a refactor.

- **`WindowKey`** — set it and the window's size, position and maximised state are remembered in `windows.json`; leave it null and nothing is written at all.
- **`ClosesOnEscape`** — off by default, because Escape inside a console pane means "stop what I am typing", not "close the window".

Restoring geometry has one non-obvious rule: **a remembered position is checked against the attached screens before it is used.** A window last closed on a monitor that is no longer plugged in would otherwise reopen off every screen, where it cannot be dragged back. The check is split into a pure `IsOnAScreen(IReadOnlyList<PixelRect>, PixelRect)` precisely so it can be tested without a display, and "no screens known" is treated as *allow* — refusing there would strand the window at the default position for a reason no user could see.

A maximised window's own bounds are the screen's, so saving them would lose the restore size. Closing while maximised keeps the previously stored normal geometry and records only the flag.

### 9.2 `PollingWindow`

Declare `RefreshInterval` and override `Refresh()`. Timer construction, start, priming, stop-on-close and disposal happen once, here, instead of five times across two frontends.

**It also does something none of the five hand-written copies did: it stops while the window is hidden or minimised.** A forgotten-but-open dashboard was a permanent 4 Hz tax, which matters directly to the weak-machine work. Restoring the window refreshes immediately, so the first thing seen is current rather than however stale it got. Occlusion is not detectable portably and is not attempted.

Two details worth knowing before writing one:

- **`StartPolling()` is called by the derived constructor, not the base one.** Priming from the base constructor would call `Refresh()` before the derived class had assigned its own fields — `CoretopWindow` would render "no target" against a `_target` that was about to be set. `Opened` calls `StartPolling()` too, so forgetting it costs a slightly later first paint rather than a window that never updates.
- **`IsPolling` is public for the tests.** Asserting "it stopped" by counting ticks would mean racing a real clock inside a dispatcher the test is itself blocking; asserting on the timer's state is deterministic. That the tests are not vacuous was checked by mutation — replacing the visibility gate with `true` fails both of them.

### 9.3 `WindowSlot<TWindow>`

The "at most one of these, else bring it forward" pattern, which seven call sites hand-wrote (five in Mistress's `MainWindow`, two in Hotaru's `DebugWindows`), each with its own nullable field and its own `Closed` unhook.

```csharp
_coretop.Show(owner: this,
              create: () => new CoretopWindow(target),
              refresh: w => w.UpdateTarget(target));
```

`RefreshIfOpen` is the second, quieter half: it **never creates and never activates**. Hotaru needs exactly this after a `core <name> <path>` swap — refreshing a dashboard that happens to be open, without popping one up for someone who never asked and without stealing focus mid-game. That was a hand-written policy in one place; now it is a method.

Thread marshalling is absorbed, but **not by always posting**: the slot runs inline when it is already on the UI thread and posts otherwise. Always posting would make `Current` unset when `Show` returns, which is surprising for Mistress, where every call is already on the UI thread. Hotaru's calls arrive from the emulation and console-reader threads and are posted.

### 9.4 Confirm and error dialogs

`Dialogs.ConfirmAsync` and `ErrorAsync` complete §6's pickers. These are the half that needed a window of our own rather than an OS dialog, which is why they waited for this phase — `MessageWindow` is built from the Phase 2 kit. Confirm returns false for cancel, for Escape and for closing the window: anything that is not a deliberate yes.

---

## 9. The fluent surface

`Fluent/` is a terser spelling of what XAML already says — `Ui` for the layouts and kit controls a window is made of, and one extension method per layout attribute. It composes §5 and §8's types rather than raw panels, which is exactly why it was built last: written first, it would have been a fluent API over `StackPanel` that the controls then had to fight.

```csharp
Content = Ui.Scroll(Ui.Stack(8,
    _header,
    Ui.Section("Load",     _load),
    Ui.Section("Palette",  _palette)).Margin(12));
```

**Every extension is named after the XAML attribute it sets** — `Margin`, `Spacing`, `Width`, `Height`, `MaxHeight`, `Grow`, `Left`, `Right`, `Center`, `Dock`, `AtColumn`, `AtRow`, `Visible`, `Bold`, `FontSize`, `Wrap`. That is the whole contract, and a test asserts it property by property: the two ways of building a window stay one vocabulary, so nobody has to learn a second layout model.

**That naming turned out to be possible only by checking.** An extension method whose name matches an existing property looks like it cannot work — `Margin` *is* a property on `Layoutable`, so `control.Margin(12)` reads like invoking a `Thickness`. It compiles: C# only falls back to extension methods when member lookup fails to produce a *method group*, and a property is not one. This was verified with a throwaway probe project before the API was designed around it, because the alternative was an invented vocabulary (`Pad`, `Spaced`, `Sized`) that would have broken the one-vocabulary rule for no reason.

`Ui.Cols` is where the phase pays for itself:

```csharp
Ui.Cols("140,*,55", label, bar, value)   // instead of three Grid.SetColumn calls
```

Columns are assigned by position, and **an explicit `.AtColumn(2)` still wins** — the convenience never becomes a rule it imposes. Spans work the same way.

### 11.1 The success criterion, proved rather than claimed

The goal set in the gameplan was that a new dashboard is *a constructor and a `Refresh()` body*, with no `.axaml` file. `EmuSen.WiseMan/LunaP/DashboardShapeTests.cs` builds exactly that — an `ExampleDashboard` shaped like `CoretopWindow` but reading plain data instead of `ICoreTelemetry` — and drives it end to end: empty state on first paint, populated after a refresh, polling suspended when hidden. It is both the proof and the worked example for Phase 6.

`GalleryWindow` was rewritten onto the fluent surface as the second check. **Its render tests passed unchanged**, which is the useful part: the fluent spelling produces an equivalent visual tree, not merely a compiling one.

---

## 10. The test harness

`EmuSen.WiseMan/Fixtures/UiTest.cs` is the one place a UI test dispatches, captures and asserts. Five files were hand-rolling the capture (`CaptureRenderedFrame` → `Lock()` → `Marshal.Copy`) and two were hand-rolling the dump; all of them now call this.

- **`UiTest.Run(body)`** — dispatches onto the one headless UI thread the session owns.
- **`UiTest.Capture(window)`** → a `RenderedFrame` (RGBA8888, width, height) with `Hash`, `DistinctColours(stopAt)` and `SavePng`. The same shape `FrameHash` and `BmpFile` already take for the core's own frame buffer, so a captured window is directly comparable against `EmuSen.Pharaoh`'s `--autoshot` tooling.
- **`UiTest.AssertLaidOut(window, name, minColours = 8)`** — the always-on assertion: a window that failed to lay out, or whose controls have no template, renders as one flat colour. It dumps, asserts, and checks the baseline if one is configured.
- **`UiTest.AssertStable(name, build)`** — builds and renders twice, asserting the two are identical.

### 13.1 `EMUSEN_UI_DUMP` is now a directory

It used to be a *file path*, and that had already stopped working: `InputSettingsWindowRenderTests` appended `_{console}` to the basename to get three files out of one variable, and the two sites that used it disagreed about whether it wrote BMP or PNG. It now names a **directory**, and every capture in the run lands in it as `<name>.png`.

```
EMUSEN_UI_DUMP=/tmp/ui dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~RenderTests"
```

### 13.2 Baselines, and why they are not committed

`AssertLaidOut` also calls `AssertMatchesBaseline`, which is **a no-op unless `EMUSEN_UI_BASELINE` is set**. That is deliberate: the surrounding test always has its own real assertion, so nothing becomes vacuous when the baseline is absent, and a fresh clone or a CI run has nothing to fail against.

The migration workflow is two commands — record on the commit before the change, compare after:

```
git worktree add /tmp/before HEAD
EMUSEN_UI_BASELINE=/tmp/frames EMUSEN_UI_BASELINE_MODE=write   dotnet test /tmp/before/EmuSen.WiseMan/...
EMUSEN_UI_BASELINE=/tmp/frames EMUSEN_UI_BASELINE_MODE=compare dotnet test EmuSen.WiseMan/...
```

A mismatch reports how many pixels differ, not just that something changed. **This was verified by mutation rather than assumed**: changing `LunaSectionHeader` from `#9CDCFE` to `#9CDCFF` — one channel, one value — failed the comparison with "gallery rendered 904 pixels differently from its baseline."

**Reference images are deliberately not committed.** They are binary blobs that churn on any font, Skia or theme change, and a stale one fails in a way that looks like a real regression. Recording a baseline from the previous commit costs one command and is never stale.

### 13.3 `AssertStable`, and the trap it encodes

Phase 1 found that `VstopWindow` can never be a baseline target: it prints live pid, uptime and CPU figures, so its pixels differ between any two runs. `AssertStable` makes that an explicit, testable property rather than something discovered when a comparison mysteriously fails — a window that shows a clock, a pid or a frame counter fails it by design, and the message says so.

The gallery is held to it, since it is the kit's own baseline target.

### 13.4 The layering rule is now enforced

`Common/LeafAssemblyTests.cs` already pinned Endymion, Serenity and Galaxia to their allowed references. **LunaP is in that list now.** §1's rule was documentation until this phase; adding one `ProjectReference` in a hurry is exactly the kind of thing that would otherwise go unnoticed until the launcher inherited the emulator.

What it asserts has since tightened twice: first to `EmuSen.Galaxia` and `EmuSen.Cauldron`, then — when the toolkit was made extractable — to **nothing at all**, as `LunaP_references_nothing_of_EmuSen`. Serenity's own assertion widened to `Cauldron, Galaxia, LunaP` in the same change, because it took what LunaP put down.

**A blind spot worth knowing about, found by sabotaging this guard and watching it pass.** These tests read `Assembly.GetReferencedAssemblies()`, and the C# compiler elides a reference used *only* for `const` values, because a constant is inlined at the call site. A first attempt to make the guard fail added a `ProjectReference` back to Galaxia and used `ConfigStore.ProgramDirName` — a `const string` — and the built assembly named Galaxia nowhere. The guard was correct and the sabotage was not. Repeating it against `ConfigStore.Directory`, an ordinary static property, reddened it immediately.

The consequence is real and applies to every assertion in that file: **a project can depend on another project's constants and these tests cannot see it.** That is a narrow hole — a `const` carries no behaviour, so nothing it inlines can drag an emulator in — but a reader should not believe the guard covers more than it does.

---

## 11. The migration

Six windows moved onto the kit, in six commits: `VstopWindow`, both `CoretopWindow`s, `FeedWindow`, both DianaOS console windows, `PreferencesWindow`, `DebugSettingsWindow`, plus every `WindowSlot` call site. **863 lines net removed from the two frontends**, and eight `.axaml` files deleted — the frontends are down to `App.axaml` plus five windows that were never in scope (`MainWindow`, `GameWindow`, `ActiveCheatsWindow`, `CheatDatabaseWindow`, `InputSettingsWindow`, `RomBrowserWindow`).

What actually went away: three `BuildMeterRow` copies with their `Grid.SetColumn` wiring, three RGBA→bitmap paths (two of which reallocated a `WriteableBitmap` on every 4 Hz tick), five hand-rolled `DispatcherTimer`s, seven "at most one, else `Activate()`" blocks, two copies of the Up/Down history recall state machine, and two byte-identical console layouts.

Every migrated dashboard also stops polling while hidden, which none of them did before.

### 12.1 What the verification caught

Three things, none of which a passing build would have shown:

- **`CoretopWindow`'s empty state was 11,060 pixels wrong on the first attempt.** `HintText` is 11 pt by definition; the original "No ROM loaded." was body-sized. It is a plain muted `TextBlock` now, and both states are byte-identical to the pre-migration render.
- **`PreferencesWindow` never showed its own Close button.** Rendering the pre-migration window from a `git worktree` showed it stopping at "ROM Directory": the content needed ~420 px in a window fixed at 330 with `CanResize=false` and no scrolling, so the only button on it was unreachable. **This long predates the toolkit** — the migration's verification is just what surfaced it. The window sizes to its content now.
- **An assumption of mine, not the code's.** I asserted `coretop`'s no-core state draws no `ProgressBar`; it draws one, because the sprite bar is a fixed part of the layout sitting at zero. Writing the tests against the *unmigrated* window is what caught that, and the migration preserves the behaviour.

`DebugSettingsWindow`, `CoretopWindow` (both states) and the gallery all came out byte-identical.

### 12.2 Two judgement calls

**`DrainPendingFromEmulationThread` goes through `slot.Current`, not `RefreshIfOpen`.** `RefreshIfOpen` marshals to the UI thread, which is correct for every other caller and exactly wrong for this one — it must run on the thread that owns the core. Hotaru's `UpdateCoretopWindowTargetIfOpen` is the opposite case and is now a single line.

**`FeedWindow` needs `.Grow()`.** `RgbaImageView` is left-aligned by default, which is right for a palette swatch in a column and wrong for a live game mirror that should fill the window. There is a test asserting the picture is actually wider than 400 px, because the wrong alignment renders as a working window that simply drew small.

### 12.3 What was still duplicated, and how it was actually resolved

*Superseded on 2026-08-04 by §16 — kept because the reasoning recorded here was wrong in an instructive way.*

The two `CoretopWindow`s were 138 lines each and **differed by six lines, all namespace or comment**. This section framed that as the deliberate consequence of sharing widgets but not windows, and named the only remaining option as "a small third assembly referencing LunaP *and* Cauldron."

**That framing was too narrow, and taking it at face value cost a wrong turn.** A third assembly was actually built before anyone asked the prior question: *does referencing Cauldron from LunaP violate what the layering rule is for?* It does not — see §16. The window is in `Dashboards/` and there is no third assembly.

### 12.4 Where the tests moved

Windows that build their own tree have no XAML namescope, so `GetControl<T>(name)` no longer resolves. Test lookups go through `FindNamed<T>` over the visual tree instead (the idiom `InputSettingsWindowLayoutTests` already used).

The eleven `DianaOSShellWindow` tests are the case worth noting: **their bodies were not touched at all**, only the three lookup helpers. Those tests drive real key routing — Enter through `KeyPress`, Up-arrow recall, live-shell attach — so keeping the bodies intact is what makes them a genuine safety net across the rewrite rather than a restatement of whatever the new code happens to do.

`CoretopWindow` and `FeedWindow` had no tests at all before this; they have sixteen now, along with a `FakeTelemetry` fixture that lets any dashboard be driven with no core loaded.

---

## 12. Themes

A theme is a file in **`/etc/EmuSen/themes/<name>.<ext>`**, written either as an `.axaml` `ResourceDictionary` (below) or as `.css` (§12.2). Both spell the same thing — overrides of whichever `Luna*` keys the theme cares about — and one name is one theme: `Available()` lists it once however many formats are on disk, and `.axaml` wins if both exist.

The `.axaml` form overrides whichever `Luna*` keys it cares about — the same category shape `cheats/<name>.json` already uses, so `man hier` covers where it lives. `LunaTheme.Apply(name)` merges it *last*, so its keys win, and persists the choice in `luna.json`; `LunaApp.Configure` calls `ApplySaved()` at startup.

```xml
<ResourceDictionary xmlns="https://github.com/avaloniaui" xmlns:x="http://schemas.microsoft.com/winfx/2006/xaml">
  <SolidColorBrush x:Key="LunaSurface" Color="#12131A" />
  <SolidColorBrush x:Key="LunaSectionHeader" Color="#7AA2F7" />
</ResourceDictionary>
```

Four properties, each pinned by a test:

- **Applying one repaints live**, with no restart, because everything resolves the palette through `DynamicResource`.
- **Keys a theme does not mention keep their built-in value**, so a two-line theme is a legitimate theme.
- **Switching replaces rather than stacks** — the previous dictionary is removed first, so a key the new theme is silent about reverts rather than keeping the old override.
- **A broken or deleted theme falls back without erasing the choice.** Fixing the file and restarting is enough to get it back; the failure is reported through `ConfigDiagnostics`, which became public for exactly this (a theme file is the same "user-editable file did not load, and why" case config files already had).

### 12.1 Two things in the kit would have frozen under a theme

Worth knowing because both were invisible until a test rendered them:

- **`MeterRow` computed its bar colour into a `BarBrush` property.** A computed brush can never follow a palette, so the load ramp would have stayed green/gold/red under every theme. It sets `:nominal`/`:busy`/`:hot` pseudo-classes now and the ramp lives in styles. `LunaPalette.LevelFor` keeps the thresholds in one place; `ForLoad` remains for code that genuinely cannot take a themed brush.
- **Window backgrounds were static `LunaPalette.Surface` assignments.** `ToolWindow` binds its own background instead, and the five migrated windows dropped the line.

`ToolWindow` *binds* rather than styles it, which is the non-obvious part: a plain `Style` selector lost to FluentTheme's own `Window` `ControlTheme` and the window stayed near-black. That was caught by a test asserting the rendered background, not by reasoning about priority order.

### 12.2 The CSS form

**`man theme` is the definition**, and it is the one a theme author should be reading: the complete token list, the element/state/part vocabulary, the properties, and what happens when a file will not load. This section is the argument behind it, not a second copy of it — the two cannot disagree, because `EmuSen.WiseMan/LunaP/ThemeVocabularyTests.cs` compares the page against `CssTheme`'s real allow-lists by **set equality in both directions**. A control added to the kit and left undocumented fails it; a token documented for a palette key that no longer resolves fails it too. That second direction is not hypothetical — it failed on the first run, against a `var(--luna-x)` placeholder in the page's own prose, which is exactly the class of thing hand-written reference text accumulates.

`Theme/CssTheme.cs` compiles a restricted CSS to the same `ResourceDictionary` §12 already merges, plus — for rule blocks — a `Styles` collection. It is a **format, not an engine**: there is no cascade, no specificity, no inheritance and no box model, and none is planned.

```css
/* Nocturne. */
:root {
  --luna-surface:        #12131A;
  --luna-section-header: #7AA2F7;
  --luna-mono-font:      "Fira Code", monospace;
  --luna-hint-font-size: 12;
}

section-header       { font-weight: normal; }
meter-row.hot .bar   { color: var(--luna-hot); }
console-pane .output { font-family: "Fira Code"; }
```

**`:root` is the palette.** `--luna-section-header` is the resource key `LunaSectionHeader` in kebab-case, and a colour defines *both* halves the palette spells (§2.1) — `LunaSectionHeaderColor` and the brush — because a theme that set only one would half-apply. Either spelling of the key is accepted and means the same declaration.

**The key's suffix decides the type, not the value's shape.** `…Size` is a number, `…Font` a font family, everything else a colour. Inferring from the value was the obvious alternative and is a coin flip: `monospace` and `gainsboro` are the same token shape. The cost of the rule is real and worth stating — a future palette key that is none of those three needs a line here, and until then it would be read as a colour and reported as unparsable.

**Colours may be written any CSS way**: `#RGB`, `#RRGGBB`, `#RGBA`, `#RRGGBBAA`, `rgb()`, `rgba()` (alpha `0`–`1`, unlike every other channel) and the named colours. The eight-digit form is the one place CSS and Avalonia genuinely disagree — `Color.Parse` reads `#AARRGGBB`, CSS puts alpha last — and **inside a `.css` file CSS wins**. A test pins both orders.

**Rule blocks are two allow-lists, deliberately.** A selector is `element[.state] [part]`: the element name is the control's own type name in kebab-case and is *derived from the type*, so a rename moves the CSS name with it instead of leaving a selector that silently matches nothing. States and parts are per-element; only `meter-row` has states today (`.nominal`/`.busy`/`.hot`, the pseudo-classes §12.1 introduced), and parts exist for `meter-row`, `filter-bar` and `console-pane`. Properties are `color`, `background`/`background-color`, `font-family`, `font-size`, `font-weight`, resolved against the *target's* type through `AvaloniaPropertyRegistry` — so `color` means `TextBlock.Foreground` on a text control and `TemplatedControl.Foreground` on a progress bar without the format needing to know they are different properties.

`var(--luna-hot)` compiles to a `DynamicResourceExtension`, so a rule that points at a token follows it. A rule that restated the colour instead would freeze exactly the way §12.1's computed brush did.

**Failure is two-tier, and the split is the design.** A *syntax* error refuses the whole file and leaves the previous theme in force, the same outcome a malformed `.axaml` theme already had: an unbalanced brace, a declaration with no colon, an unterminated comment, an at-rule, a nested rule. An *unknown* selector, state, part, property or unparsable value is reported through `ConfigDiagnostics` and skipped, and the rest of the theme applies — because a theme written against a later LunaP has to keep loading, and refusing the file would make every control added to the kit a breaking change for every theme on disk.

Reported line numbers are the file's own: comments are replaced by whitespace of the same shape rather than deleted, so a warning after a twenty-line comment block still names the right line. There is a test for that specifically, since it is the one part of a hand-written parser that silently drifts.

**Why a parser and not `AvaloniaRuntimeXamlLoader`.** Two arguments, and the second is the stronger one. Hand-editability is a stated goal for user-facing files here — the same argument `EmuSen_Stack.md` §4.1 makes for config staying JSON. And the XAML loader will instantiate *arbitrary Avalonia types* out of a file in `/etc`, where this parser structurally cannot: it emits brushes, doubles, font families and setters, or it emits nothing. The `.axaml` form keeps its capability and its exposure; the CSS form has neither.

**A negative result, recorded so nobody looks for the missing test.** The "this property does not apply to this target" guard is currently *unreachable*. Every property in the allow-list is registered on `TemplatedControl` or `TextBlock`, and every selectable target is one or the other, so no allow-listed property can miss. The check stays, because the two allow-lists are meant to grow independently and the first `Image` or `Panel` part will reach it — but no test covers it and none can be written today. A test asserting it was written, failed, and was deleted rather than weakened.

### 12.3 Mutating `Application.Styles` at runtime strips realized controls

This is the finding the CSS work turned up, and it had been sitting under the theme system since Phase 7 without being visible, because until rule blocks existed nothing ever added a style at runtime.

Measured, from a throwaway diagnostic run against a live `SectionHeader`:

```
before=#ff9cdcfe   mutated=White   reparented=#ff7aa2f7   clearedThenReparented=#ff9cdcfe
```

A header on screen shows its themed `#9CDCFE`. Adding a single `Style` to `Application.Styles` drops it to `White` — **not** to the new rule's colour and **not** back to the built-in one. The new style is not winning wrongly; the control has lost the LunaP style it already had, and removing the style again does not give it back. Controls realized *after* the mutation pick the new style up correctly, which is exactly why this is invisible at startup: `LunaApp.Configure` applies the saved theme before any window exists, so only a live theme *switch* can hit it.

Detaching and reattaching the window's content re-runs the style pass and fixes both directions. That is all `LunaTheme.Restyle(ContentControl)` is, and `ToolWindow` subscribes to `LunaTheme.StylesChanged` so every LunaP window does it for itself.

Three consequences worth carrying:

- **The event fires only when `Application.Styles` actually changed.** Every `.axaml` theme and most `.css` ones are palette-only, and those repaint through `DynamicResource` exactly as before at no cost. A test asserts the restyle count is zero for that case, so the cheap path cannot quietly become the expensive one.
- **A reattach is not free**: keyboard focus and scroll position inside the window are lost. That is acceptable for a deliberate, rare theme switch and would not be acceptable anywhere near a frame path. `ConsolePane` survives it only because it buffers output in the control rather than in the template part — §5.6's decision paying off a second time, for a reason that did not exist when it was made.
- **A window that is not a `ToolWindow` is not covered**, and the frontends still have five of those (§11). They need `LunaTheme.Restyle(window)` if a rule-block theme is switched while they are open. The uncovered case has its own test — asserting that a plain `Window` *does* lose its styling — so the hook is not deleted later as redundant.

---

## 13. The widgets

Four, all **wrapping** Avalonia's own controls rather than reimplementing them — the value added is the theme plus an API shaped like the calls the frontends actually make.

### 13.1 `LunaSwitch`, `Dropdown`, `Tabs`, and the style-key trap for the second time

All three wrappers pin `StyleKeyOverride` to their base type. §5.5 recorded this for `ButtonBar`, where the symptom was a control that rendered as nothing. **`LunaSwitch` is worse: `ToggleSwitch.OnApplyTemplate` does not degrade to blank, it throws on the missing `PART_MovingKnobs`.** The rule to carry forward: *anything here that derives from a stock Avalonia control needs its style key pinned to that control, and a test that finds a real template part.*

`LunaSwitch` puts its `Label` into `OnContent` **and** `OffContent` rather than `Content`. That places the text beside the knob and keeps it there — the same single line the `CheckBox` it replaces already drew. `Content` stacks it above, and the stock On/Off captions say nothing the knob's own position does not.

`Dropdown.Fill(items, selected)` exists for a real bug from the other direction: setting `ItemsSource` then `SelectedItem` raises `SelectionChanged`, and `PreferencesWindow` already needed an `_initializing` flag so filling a list did not look like a user choice and get written straight back to config. `Fill` does that suppression once, and `Chose` fires only for a genuine pick.

`Tabs.Add(header, content)` and `RemoveFrom(index)` replace the "construct a `TabItem`, push it into `Items`" chore both frontends hand-wrote for their per-console tabs.

### 13.2 `FilterBar`

A search box, optionally preceded by a labelled facet dropdown. Two windows had built this independently, and it owns the detail one of them had a comment about:

> **It watches `TextBox.TextProperty`, not `TextChanged`** — only the property change reacts to a `Text` set that did not come from typing.

A test covers the programmatic case specifically, because that is the half a naive rewrite drops. `FilterBar.Matches` is the case-insensitive "empty matches everything" test both callers wanted, and `Submitted` is Enter in the search box.

The gap between facet and search sits on the *dropdown*, so it collapses with it: the library shows both, the cheat database only the search box, and neither gains a stray indent.

---

## 15. `Input/DefaultPadKeyMap` — moved to `EmuSen.Endymion`

**This no longer lives here.** It is `EmuSen.Endymion/Input/DefaultPadKeyMap.cs`, in the namespace `EmuSen.Endymion.Input`, and `EmuSen_Input.md` §4.3 is now the section that describes it. Everything below is kept because the reasoning that put it here is what decided where it went instead — see §19.3.

The keyboard scheme both frontends start from — arrows for the d-pad, `Z`/`X`/`A`/`S` for B/A/Y/X, `Q`/`W` for the shoulders, `Enter`/`RightShift` for Start/Select — plus the `Key → PadButton` reverse lookup they both need.

**It was spelled twice**, in Hotaru's `HotaruKeyMap` and Mistress's `ControllerKeyMap`, along with two copies of the reverse-lookup loop. A test asserted the two tables stayed equal, which is the shape of a problem being *guarded* rather than *fixed*.

**Why here and not in `EmuSen.Galaxia`, which is where the config models live.** Galaxia's csproj states the constraint plainly: it is a leaf with no `ProjectReference` and no `PackageReference`, because `EmuSen.DianaOS` references it and `EmuSen` references DianaOS — anything Galaxia depended on upward would close a cycle. So **Galaxia cannot name `Avalonia.Input.Key`**, and "the typed maps that use foreign enums keep their own homes" is that csproj's own conclusion. This table needs `Key` *and* `PadButton`, and LunaP was the one project that already referenced both Avalonia and Galaxia.

**And that is exactly why it had to leave.** "The one project that references both" was a statement about this repository, not about the toolkit's subject. A general Avalonia toolkit has no business naming a console gamepad button, and the argument above never claimed otherwise — it argued from what was convenient. Endymion already owns the mapping of physical input onto `PadButton` (`GamepadBindingMap`), already references Galaxia, and is already referenced by both frontends; it took one `Avalonia` base package reference to hold this file too, which is a smaller price than a toolkit that cannot leave the repository.

`Bindings()` returns a **fresh dictionary per call**, not a shared readonly instance: Mistress rebinds into its copy, and a shared instance would leak one frontend's edits into the other.

---

## 16. `Dashboards/` — moved to `EmuSen.Serenity`

**This no longer lives here either.** `CoretopWindow` is `EmuSen.Serenity/Dashboards/CoretopWindow.cs`, in the namespace `EmuSen.Serenity.Dashboards`, and the amendment below was withdrawn with it — §1's rule is back to its unamended form and is now stricter than it ever was: Avalonia and nothing else.

The amendment was not wrong. Cauldron is a dependency-free leaf and referencing it really did cost a consumer one small interfaces assembly rather than an emulator. What changed is the question being asked. It stopped being "does this reference hand the launcher a core" and became "can somebody outside this repository resolve it at all" — see §19. A reference to anything called EmuSen fails that second test whatever it costs.

Serenity took it because Serenity already *is* the core-agnostic Avalonia layer: it presents game frames while taking `(byte[] rgba, int w, int h)` rather than an `ICore`, which is the same standard a dashboard reading `ICoreTelemetry` is held to. It gained `EmuSen.LunaP` and `EmuSen.Cauldron`, both core-free, and `LeafAssemblyTests` pins the new list.

Everything below is the original reasoning, kept.

`Dashboards/` holds windows that are LunaP chrome plus an `ICoreTelemetry`. `CoretopWindow` is the only one today.

**This is the one place in the project allowed to name `EmuSen.Cauldron`**, and §1's rule is amended to permit it. The reasoning matters more than the amendment:

> §1's rule exists because *"the launcher's whole value is browsing a library with no core loaded, and one upward reference here hands it the whole emulator."* **`EmuSen.Cauldron` is a dependency-free leaf** — six files of read-only telemetry contracts, no `PackageReference`, no `ProjectReference`. Referencing it hands the launcher one small interfaces assembly and no core at all. The rule's *purpose* is untouched; only its letter changed.

The distinction that keeps this from becoming a slippery slope: **controls take plain data or a delegate, dashboards may take a contract.** A `MeterRow` still takes `(string, double, string)` and never a `DebugLoadInfo`. If a control wants an `ICoreTelemetry`, it is a control that should have taken plain data.

`EmuSen`, `EmuSen.DianaOS` and `EmuSen.Endymion` remain forbidden, and none of them is a leaf. A window needing `IDebugTarget` is not a dashboard — that interface stays in DianaOS (`EmuSen_Cauldron.md` §3.1) and such a window belongs in a frontend.

### 16.1 The wrong turn, recorded because the doc caused it

§12.3 named "a small third assembly referencing LunaP *and* Cauldron" as the only remaining option. **A whole project was created on that basis** — csproj, solution entry, references from both frontends and the test project, a codename claimed out of the Sailor Moon pool, its own reference doc — to hold one 137-line file. It was deleted the same day.

The prior question was never asked: *is Cauldron actually the kind of dependency this rule is about?* It is not. **An assembly per layering exception grows the project faster than an entry on an allow-list does**, and a codename is a permanent claim on a finite pool (`EmuSen_Core_Naming_Scheme.md` §11 carries the same lesson from the other side).

The general form, since a plan naming exactly one option is how this happened: **a doc that says "the remaining option is X" is recording what was considered, not what is possible.** Re-derive before building on it.

### 16.2 `CoretopWindow`

The GUI counterpart to DianaOS's own `coretop` (`man coretop`). A `PollingWindow` on the same 250 ms/4 Hz cadence the console version uses. Both frontends open the same class and differ only in how they reach it — Hotaru via `DebugWindows.ShowCoretopWindow` from `coretop -w`, Mistress via `MainWindow.OpenCoretopWindow` from the Hardware Dashboard menu item.

Two behaviours that are load-bearing and non-obvious, both pinned by tests:

- **`UpdateTarget(null)` is a real state**, not a defensive check — the window drops to "No ROM loaded." That text is a plain muted `TextBlock` and **not** a `HintText`, because it is the window's whole content in that state rather than an explanation under something. `HintText` is 11 pt by definition, and using it here measured 11,060 pixels wrong (§12.1).
- **The sprite bar stays on screen at zero when no core is loaded.** It is a fixed part of the layout, not a per-core meter. The natural assumption is that the empty state draws no `ProgressBar`; it draws exactly one.

The two frontends' `CoretopWindowTests` merged too. Mistress's was a strict superset — it covered the no-tile-memory case and the `UpdateTarget(null)` unload — so the merged file is its body, now in `EmuSen.WiseMan/Serenity/` beside the code it tests. What was genuinely dropped is the second pair of render baselines: with one window class there is one render to pin, and a second baseline under another name asserted the same pixels twice.

---

## 17. LunaP as a package, and the consumer outside this solution

`EmuSen.Pegasus` left this repository on 2026-08-09 for
<https://github.com/RedQuE3n/EmuSen.Pegasus>. It still builds its window from
LunaP, so LunaP is now packed and consumed as a NuGet package rather than as a
`ProjectReference`.

Three projects are packed at 0.1.0: `EmuSen.LunaP`, and the two leaves its
layering rule already allowed it to name, `EmuSen.Galaxia` and `EmuSen.Cauldron`.
The two leaves are packed only because a package's dependencies must themselves
be resolvable — a consumer outside this repository cannot follow a
`ProjectReference` into it.

**This does not relax the layering rule in the `.csproj`; it is the first thing
that has ever enforced it.** That rule — LunaP may reference Avalonia, Galaxia
and Cauldron and nothing else — exists so the eventual launcher can browse a
library with no core loaded. Until now it was a comment that a careless
`ProjectReference` could contradict. A package cannot reach up into a core at
all, so the constraint is now a property of the artifact rather than of
somebody's attention.

The practical consequence for work in this repository: **LunaP's public surface
has a consumer that does not appear in `EmuSen.sln`.** Renaming `LunaApp.Configure`,
`Windowing.ToolWindow`, the `Fluent.Ui` helpers, or the
`avares://EmuSen.LunaP/Theme/LunaTheme.axaml` URI will not break any build here
and will break Pegasus. That URI in particular is load-bearing across the
boundary: it resolves out of the packaged assembly's compiled resources, and a
consumer that fails to include it gets a window in which every control occupies
layout and draws nothing. That failure has shipped once — the account is now in
the Pegasus repository's `docs/Pegasus_Design.md` §11, having left with the
project, and `LunaTheme.axaml`'s own comment predicted it before it happened.
§5.5 and §13 are the in-repository cousins of the same failure, where an
untemplated control renders as nothing.

Two limitations, recorded now rather than discovered later:

- **The feed is a folder.** Pegasus's `NuGet.config` points at a `local-packages/`
  directory populated by `dotnet pack` here. GitHub Packages is the intended
  destination; nothing about the arrangement depends on which feed serves it,
  and the folder exists only because the packages have not been pushed yet.
- **Galaxia's catalogue schema does not travel.** `Library/Catalogue/*.sql` are
  `None` items copied to build output, which is not the same as packaged content,
  so the package carries the assembly and not the SQL. Pegasus never touches the
  catalogue, so this is free here. Anyone packaging Galaxia for a consumer that
  *does* want the catalogue must fix it first, and should not assume the 0.1.0
  package is a working example.

A version discipline is not yet established, and pretending otherwise would be
worse than saying so: 0.1.0 was chosen to start somewhere, and there is no
release process, no changelog and no automated republish. The first time LunaP
changes under Pegasus, that gap is what will be felt.

---

## 18. Where to look next

- **`EmuSen_LunaP_Gameplan.md`** — the plan of record: the full duplication audit (§1), the settled decisions (§2), Phases 2–6 (controls, window scaffolding, the fluent surface, harness support, migration), and the questions deliberately left open (§6).
- **`EmuSen_Launcher_Multicore_Gameplan.md`** — the launcher this project is eventually for. Its Phase 4 (theming) is why §2's palette is a resource dictionary rather than a set of constants.
- **`EmuSen_Cauldron.md`** — `ICoreTelemetry` and the snapshot/provider contract §16's dashboards consume; §3.1 for the Cauldron-versus-`IDebugTarget` split that keeps this reference safe.
- **`EmuSen_Core_Naming_Scheme.md` §11** — the name reservation and the `Luna`/`LunaP` collision note, plus the closing note on §16.1's near-miss.
- **`EmuSen_Input.md` §4.3** — `DefaultPadKeyMap`, in the project that owns it now.

## 19. Cutting the toolkit loose

`EmuSen.Pegasus` consumes LunaP from another repository already (§17), and it has turned out to be worth more than that: a general Avalonia toolkit — theme, chrome, remembered geometry, a fluent layout surface — is useful to people who will never run an emulator. The goal is its own repository, so it can be taken on its own terms.

That goal changes the question §16's amendment was answering. It stopped being *"does this reference hand a launcher a core"*, where `EmuSen.Cauldron` passed honestly, and became *"can somebody outside this repository resolve this at all"*, where nothing named EmuSen passes. `LunaP_references_nothing_of_EmuSen` is that question written as an assertion.

Three things carried the old references, and each went somewhere different:

| What | Reference | Where it went |
|---|---|---|
| `Windowing/WindowPlacementStore.cs`, `Theme/LunaTheme.cs` | `Galaxia.ConfigFile`, `ConfigStore`, `ConfigDiagnostics` | §19.1 — a seam, so the toolkit keeps the behaviour and stops naming the provider |
| `Input/DefaultPadKeyMap.cs` | `Galaxia.Input.PadButton` | `EmuSen.Endymion` — §15 |
| `Dashboards/CoretopWindow.cs` | `Cauldron.ICoreTelemetry` | `EmuSen.Serenity` — §16 |

**No new project was created, and §16.1 is why.** That section records a whole assembly being stood up — csproj, solution entry, references from three projects, a codename out of a finite pool — to hold one 137-line file, and deleted the same day. The same file was in play here. Two files needing a home is a stronger case than one, and it was still the wrong shape: each file has a subject, and each subject already had a project that owned it. Input mapping is Endymion's. A core-agnostic Avalonia window is Serenity's. Asking "what is this file *about*" got a better answer than asking "what does it *reference*".

### 19.1 The settings seam

`Settings/ISettingsStore.cs` is three methods — `Directory(category)`, `Load<T>`, `Save<T>` — and `Settings/LunaSettings.cs` holds the one the host has chosen, plus a `Diagnostics` sink for "this file would not load, and why".

`Settings/JsonSettingsStore.cs` is the default, and it is deliberately a near-copy of what Galaxia's `ConfigFile` does for these two files: indented JSON, comments and trailing commas tolerated, case-insensitive properties, and a full-write-then-rename so an interrupted save leaves the previous file intact rather than a truncated one. A toolkit that only worked when a host supplied a store would be a toolkit with a required setup step, so `LunaSettings.Store` resolves on first use to a store named after the entry assembly.

Two things Galaxia's `ConfigFile` does that this does not, both deliberate:

- **No migration from a legacy directory.** That is a fact about EmuSen's own history and belongs to the host, which can hand LunaP a store pointed wherever it likes.
- **No `SuggestingEnumConverterFactory`.** It exists for config files full of enum names; LunaP stores a `Dictionary<string, WindowPlacement>` and a one-string `ThemeChoice`, neither of which has an enum in it.

The two files themselves — `windows.json` and `luna.json` — are unchanged in name, location and content for EmuSen, because §19.2 points the store at the same directory.

### 19.2 What the frontends do about it

Two lines each, in `Program.cs`, beside the `ConfigDiagnostics.Sink` line that was already there:

    LunaSettings.Store = new JsonSettingsStore(ConfigStore.Directory);
    LunaSettings.Diagnostics = ConfigDiagnostics.Report;

That is the whole adapter, and its being two lines rather than a class is why no project was needed to hold it. EmuSen's files stay where EmuSen puts them and its diagnostics keep arriving on its own sink.

The three test fixtures that used to redirect `ConfigStore.OverrideDirectory` now assign `LunaSettings.Store` instead, which is a small improvement on top: they exercise the seam the toolkit actually ships rather than a provider it no longer knows about. `ThemeTests` and `CssThemeTests` capture through `LunaSettings.Diagnostics` for the same reason.

### 19.3 What is left before it can move

The references are gone and the guard holds. What has not been done: the repository itself, a `README` written for somebody who has never heard of EmuSen, and the package version stamping — LunaP, Cauldron, Galaxia and `EmuSen.Pegasus.Core` are all `0.1.0` and static, which is the root of the folder-feed trap `EmuSen.Chariot`'s `NuGet.config` records. Nothing in this section is blocked on that; it is the next thing.
