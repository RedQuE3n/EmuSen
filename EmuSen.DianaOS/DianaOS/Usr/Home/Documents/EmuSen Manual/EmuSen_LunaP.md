# EmuSen.LunaP — the shared Avalonia toolkit

*This revision (2026-08-04): Phase 4 landed — the fluent layout surface, bringing LunaP to 76 headless tests, and with it the end of the build phases. Everything the toolkit offers now exists; nothing in either frontend consumes any of it yet, which is Phase 6. Previous revision (2026-08-04): Phase 3 — the windowing layer (`ToolWindow`, `PollingWindow`, `WindowSlot`) and the confirm/error dialogs. Phase 2 before that — eleven controls, the file/folder pickers, and a gallery window. Phases 0 and 1 before that — the project, the shared theme, and the one Avalonia bootstrap. `EmuSen_LunaP_Gameplan.md` remains the plan of record for everything not yet built. This doc covers only what is real.*

---

## 1. What it is, and the rule that keeps it useful

`EmuSen.LunaP` is the shared Avalonia toolkit for `EmuSen.Mistress`, `EmuSen.Hotaru` and the future launcher: theme, controls, window scaffolding, and a fluent layout surface. It is the chrome *around* the game picture; `EmuSen.Serenity` owns the picture itself and the two do not overlap.

**The layering rule is load-bearing and is the reason the project is worth having:**

> LunaP may reference **Avalonia** and **`EmuSen.Galaxia`** and nothing else. Not `EmuSen` (the core), not `EmuSen.DianaOS`, not `EmuSen.Cauldron`, not `EmuSen.Nehellania`.

The launcher's entire value is browsing a library with no core loaded. One upward reference here hands it the whole emulator, permanently. `EmuSen.Serenity` already holds this line on the video side — it presents frames while taking `(byte[] rgba, int width, int height)` rather than an `ICore` — and **every LunaP control takes plain data or a delegate for the same reason.** A meter row takes `(string, double, string)`, never a `DebugLoadInfo`. A console pane takes a `Func<string, string>`, never a `DianaOSInterpreter`.

This is also why the two frontends share *widgets* but not *windows*: `CoretopWindow` consumes `ICoreTelemetry`, so it stays a file in each frontend even though almost everything inside it is now shared. See `EmuSen_LunaP_Gameplan.md` §2.

The name is Luna-P, Chibiusa's floating gadget ball, which becomes whichever tool is needed. It is **not** `Luna`, the reserved codename for the Nintendo DS core — see `EmuSen_Core_Naming_Scheme.md` §11.

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

## 9. The windowing layer

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

## 11. The fluent surface

`Fluent/` is a terser spelling of what XAML already says — `Ui` for the layouts and kit controls a window is made of, and one extension method per layout attribute. It composes §5 and §9's types rather than raw panels, which is exactly why it was built last: written first, it would have been a fluent API over `StackPanel` that the controls then had to fight.

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

## 12. Where to look next

- **`EmuSen_LunaP_Gameplan.md`** — the plan of record: the full duplication audit (§1), the settled decisions (§2), Phases 2–6 (controls, window scaffolding, the fluent surface, harness support, migration), and the questions deliberately left open (§6).
- **`EmuSen_Launcher_Multicore_Gameplan.md`** — the launcher this project is eventually for. Its Phase 4 (theming) is why §2's palette is a resource dictionary rather than a set of constants.
- **`EmuSen_Core_Naming_Scheme.md` §11** — the name reservation and the `Luna`/`LunaP` collision note.
