# EmuSen.LunaP — Shared UI Toolkit Game Plan

*This revision (2026-08-04): **Phases 0 through 6 are all done — the toolkit is built and both frontends are migrated onto it.** 863 lines net removed from the frontends, eight `.axaml` files deleted, full suite 2,180. What exists is documented in `EmuSen_LunaP.md`; what remains here is **Phase 7**, the widget library proper (dropdowns, switches, tabs, filters), which is the only unbuilt phase. Previous revision (2026-08-04): first version, written after an audit of every `.axaml`/`.axaml.cs` file in `EmuSen.Mistress` and `EmuSen.Hotaru` (~7,200 lines across 46 files); three questions were put to the project owner before drafting and are answered in §2.*

---

## 0. Orientation, if you're new to this

EmuSen has two Avalonia frontends today and a third planned:

- **`EmuSen.Mistress`** — the bug-testing GUI. Menu bar, ROM loading, save states, cheats, preferences. Ten windows.
- **`EmuSen.Hotaru`** — the development/driver frontend. Game window, DianaOS shell window, `coretop -w`/`feed -w` debug windows. Four windows.
- **The launcher** — an EmulationStation-shaped browsing shell, scoped in `EmuSen_Launcher_Multicore_Gameplan.md` as a separate, not-yet-created project. Its Phase 4 (presentation/theming) is the single biggest UI effort in the project's future.

There is already one shared Avalonia library: **`EmuSen.Serenity`**, which owns `GameFrameControl`/`FramePresenter`/shaders — the *game picture*. It is correctly scoped and this plan does not touch it. What has no home is everything *around* the picture: chrome, theme, dashboards, dialogs, window lifecycle.

Each new window is currently written from raw Avalonia primitives. This document plans a project that stops that.

---

## 1. The evidence

Not a hunch — this is what an audit of the two frontends actually found.

### 1.1 Theme constants are hand-typed

Five colours and one font stack, retyped as literals across six `.axaml` files in two projects, with no `ResourceDictionary` anywhere. Both `App.axaml` files are ten lines containing only `<FluentTheme />`.

| Literal | Occurrences | Means |
|---|---|---|
| `#D4D4D4` | 18 | body text |
| `#9CDCFE` | 15 | section header |
| `#1E1E1E` | 5 | tool-window surface |
| `#252526` | 2 | input surface |
| `#D08770` | 1 | (one-off warning colour) |
| `Consolas,Menlo,monospace` | 16 | the mono stack |

Code-behind adds its own vocabulary on top: `Brushes.Gainsboro` ×6, `Brushes.OrangeRed` ×4, `Brushes.LimeGreen` ×3, `Brushes.Gold` ×3, `Brushes.Gray` ×3, plus `Color.Parse("#808080")`. Nothing connects the XAML palette to the code-behind palette; they are two unrelated sets of magic values describing one theme.

### 1.2 Widgets are rebuilt by hand, in C#, per window

- **`BuildMeterRow`** (label / progress bar / value, with `Grid.SetColumn` wiring) — **three** implementations: `Mistress/Views/CoretopWindow.axaml.cs`, `Hotaru/Views/CoretopWindow.axaml.cs` (verbatim copies of each other), and a third in `Mistress/Views/VstopWindow.axaml.cs`.
- **`ColorForPercent`** (the green→gold→orange-red ramp at 60/85) — duplicated alongside it.
- **RGBA → `WriteableBitmap`** — three implementations, and they *disagree*. Both `CoretopWindow`s allocate a fresh bitmap every tick; `Hotaru/Views/FeedWindow.axaml.cs` reuses one across frames and only reallocates on a size change. The better implementation exists and does not propagate.
- **"Section header, then content"** — the `#9CDCFE`-bold-`TextBlock` idiom, ~10 hand-written repetitions.
- **"Label, grey 11pt hint, control"** — the settings-row idiom, in `PreferencesWindow`, `DebugSettingsWindow`, `ActiveCheatsWindow`.
- **"Read-only path box + Browse… button"** — 4 sites (3 in `PreferencesWindow` alone), each with its own `StorageProvider.OpenFolderPickerAsync` boilerplate.
- **Bottom status/button bar** — ~5 windows, plus 56 `StatusText.Text = …` assignments with no shared notion of what a status line is.

### 1.3 Window lifecycle is retyped

The **"at most one, else `Activate()`"** pattern is hand-written **seven** times: five in `Mistress/Views/MainWindow.axaml.cs` (`_cheatDatabaseWindow`, `_activeCheatsWindow`, `_consoleWindow`, `_coretopWindow`, `_vstopWindow`) and two in `Hotaru/Views/DebugWindows.cs`. Each carries its own nullable field, its own `Closed += (_, _) => _x = null;`, and — in Hotaru only — its own `Dispatcher.UIThread.Post` marshalling.

The **refresh-timer** shape (construct `DispatcherTimer`, `Tick += Refresh`, `Start()`, `Closed += Stop`, prime with one `Refresh()`) is written **five** times at four cadences. Three further `DispatcherTimer`s exist for genuinely different jobs (the frame driver, a 50 ms pad poll, a 60 Hz gamepad poll) and are *not* part of this pattern.

None of the five pause when their window is hidden — relevant to the weak-machine work, where a forgotten-but-open dashboard is a permanent 4 Hz tax.

### 1.4 Whole files are duplicated

- `Mistress/Views/DianaOSConsoleWindow.axaml` and `Hotaru/Views/DianaOSShellWindow.axaml` are **byte-identical** apart from `x:Class` and a 40 px width difference.
- The two `CoretopWindow.axaml.cs` files are a deliberate ~200-line copy. `Hotaru`'s carries a comment justifying it: *"duplicating one small, self-contained ~200-line file costs far less than standing up a shared Avalonia UI library project would for the two frontends to both reference."* **That calculus was correct when there were two consumers of it and no library project. It stops being correct here** — the library is being stood up anyway, and a third frontend is coming.

### 1.5 Bootstrap is duplicated three ways

`UsePlatformDetect().WithInterFont().LogToTrace()` plus a Linux-only `UseX11()` appears in both `Program.cs` files. `EmuSen.WiseMan/Serenity/TestAppBuilder.cs` has a third, divergent copy that hand-adds `FluentTheme` and `ThemeVariant.Dark` *because it cannot reference either `App.axaml`* — with a comment noting that without the theme, render assertions silently pass over untemplated controls. That is a real bug class created purely by having no shared application setup.

---

## 2. Decisions already made

Settled with the project owner before this document was drafted. Do not re-litigate these.

1. **The project is `EmuSen.LunaP`.** Luna-P is Chibiusa's gadget ball, which becomes whichever tool is needed — the toolkit metaphor, exactly. See §7.2 for the required naming-doc bookkeeping and the `Luna` (DS core) collision note.
2. **Layered API: controls first, fluent surface on top.** Real Avalonia controls usable from XAML, *and* a thin fluent builder that composes those same types. Existing `.axaml` windows keep working untouched; new screens may be written either way. Explicitly not "retire XAML" and explicitly not "controls only".
3. **Share widgets, not windows.** LunaP stays core-agnostic. `CoretopWindow` shrinks to a shell of LunaP widgets in *both* frontends but remains two files; it never moves into LunaP, because that would drag `ICoreTelemetry` — and the DianaOS interpreter, via the console panes — into a project the future launcher must be able to reference cheaply. The one exception is the DianaOS console *pane* in §4.2, which is a pure text widget with the interpreter injected, not a window.

---

## 3. Shape and the layering rule

```
EmuSen.LunaP/
  Theme/
    Dark.axaml            ResourceDictionary: colours, brushes, font stacks
    Controls.axaml        default styles for the controls below
  Controls/
    SectionHeader.cs      MeterRow.cs        MeterList.cs
    FieldRow.cs           PathPickerRow.cs   StatusBar.cs
    ButtonBar.cs          RgbaImageView.cs   ConsolePane.cs
  Windowing/
    ToolWindow.cs         PollingWindow.cs   WindowSlot.cs
    Dialogs.cs
  Fluent/
    Ui.cs                 Layout.cs
  Gallery/
    GalleryWindow.cs      every control, one window, for the render test
```

**The layering rule, and it is the load-bearing constraint of this whole plan:** LunaP may reference **Avalonia and `EmuSen.Galaxia` (config, for window-geometry persistence) and nothing else.** Not `EmuSen` (the core), not `EmuSen.DianaOS`, not `EmuSen.Cauldron`, not `EmuSen.Nehellania`.

The moment LunaP references any of those, the launcher — whose entire value is being able to browse a library without a core loaded — inherits the emulator. `EmuSen.Serenity` already demonstrates the discipline: it presents game frames while referencing only `EmuSen.Galaxia`, taking `(byte[] rgba, int w, int h)` rather than an `ICore`. **Every LunaP control takes plain data or a delegate for the same reason.** `MeterRow` takes `(string, double, string)`, never a `DebugLoadInfo`. `ConsolePane` takes a `Func<string, string>` submit handler, never a `DianaOSInterpreter`.

Consumers: `EmuSen.Mistress`, `EmuSen.Hotaru`, `EmuSen.WiseMan`, and later the launcher.

---

## 4. The phases

Each phase is independently useful and independently revertable. Phases 1–5 add code without changing behaviour; only Phase 6 edits existing windows.

### Phase 0 — Stand up the project ✅ done

`net10.0` library, `Nullable`/`ImplicitUsings` enabled, `AvaloniaUseCompiledBindingsByDefault`, Avalonia `12.1.0` + `Desktop` + `Themes.Fluent` + `Fonts.Inter` + `X11` (versions pinned to match Mistress exactly, the convention Serenity's csproj already follows). Added to `EmuSen.sln`; referenced by Mistress, Hotaru and WiseMan up front so later phases are drop-in.

**Landed as planned, with one deviation:** the `EmuSen.Galaxia` reference was *not* added. Nothing needs it until Phase 3's window-geometry persistence, and an unused `ProjectReference` documents a dependency that does not exist. The layering rule it was meant to express lives in the csproj header comment instead. Add it when `ToolWindow` actually persists geometry.

**Done when:** the solution builds with an empty LunaP referenced everywhere and no behaviour has changed. — *Verified: clean build, 0 warnings.*

### Phase 1 — The theme ✅ done

One `ResourceDictionary` naming what §1.1 currently spells as literals — `LunaSurface` (`#1E1E1E`), `LunaInputSurface` (`#252526`), `LunaText` (`#D4D4D4`), `LunaSectionHeader` (`#9CDCFE`), `LunaMuted` (grey 11 pt), `LunaMono` (the font stack) — plus the load ramp (`LunaNominal`/`LunaBusy`/`LunaHot` at the existing 60/85 thresholds) so XAML and code-behind finally share one palette.

Both `App.axaml` files reduce to a `StyleInclude`. `TestAppBuilder` includes the *same* file instead of hand-building a theme, closing §1.5's silent-vacuous-assertion hole.

A `LunaApp.Configure<TApp>()` helper absorbs the `UsePlatformDetect`/`WithInterFont`/`LogToTrace`/Linux-`UseX11` sequence; both `Program.cs` files call it.

**Done when:** `grep -r '#[0-9A-Fa-f]\{6\}' --include=*.axaml EmuSen.Mistress EmuSen.Hotaru` returns nothing, and both frontends look pixel-identical to before. — *Verified: the grep is clean (and also for `"Gray"` and `Consolas`); `CoretopWindow`, `DebugSettingsWindow` and `PreferencesWindow` render **byte-for-byte identical** to the pre-change commit across 3,144,000 bytes of RGBA. Full suite 2,118/2,118. Method and caveats in `EmuSen_LunaP.md` §4.*

Two things came out of doing it that the plan did not anticipate, both recorded in `EmuSen_LunaP.md` §2.1 and §4.1: `LunaText` (`#D4D4D4`) and `LunaMeterText` (`#DCDCDC`) are the same *role* in two shades and want converging in Phase 2, and one `FontFamily="monospace"` in `ActiveCheatsWindow` is not the shared stack. Neither could be fixed in a phase that guaranteed no visual change; both are one-line deliberate decisions later.

### Phase 2 — The control kit ✅ done

Real `TemplatedControl`/`UserControl` types, styled by `Theme/Controls.axaml`, usable from XAML *and* constructible in C#:

| Control | Replaces |
|---|---|
| `SectionHeader` | ~10 hand-written `#9CDCFE` `TextBlock`s |
| `MeterRow` / `MeterList` | 3 `BuildMeterRow` + 2 `ColorForPercent` implementations |
| `RgbaImageView` | 3 divergent RGBA→bitmap paths; takes FeedWindow's reuse-the-bitmap version as the one true implementation |
| `FieldRow` | the label/hint/control settings idiom, 3 windows |
| `PathPickerRow` | 4 read-only-box + Browse… pairs |
| `StatusBar` / `ButtonBar` | ~5 bottom bars, 56 raw `StatusText.Text` assignments |
| `ConsolePane` | the byte-identical prompt+output XAML in both frontends (§1.4) |

`ConsolePane` is the one control that needs care: it must expose a submit hook and a "clear" method as plain delegates, so `DianaOSConsoleWindow` and `DianaOSShellWindow` inject their own interpreters and their existing `ClearConsoleWindowCommand`/`ClearShellWindowCommand` keep working, without LunaP knowing DianaOS exists.

**Done when:** the gallery window (Phase 5) shows every control, and no frontend has been modified yet. — *Verified: eleven controls, 48 headless tests, full suite 2,137/2,137, no frontend touched. Built as documented in `EmuSen_LunaP.md` §5.*

Three things came out of doing it:

- **The gallery was built here rather than in Phase 5**, because this phase's completion check depends on it and it is ~110 lines. Phase 5 keeps the rest of its harness work.
- **The file/folder pickers were pulled forward from Phase 3**, because `PathPickerRow` is meaningless without them. `ConfirmAsync`/`ErrorAsync` stay in Phase 3, where they belong — they need a window of our own, not an OS dialog.
- **A trap worth carrying into every future control** (`EmuSen_LunaP.md` §5.5): deriving from a templated Avalonia control silently loses its default template, because a control's style key defaults to its own runtime type. `ButtonBar` rendered as nothing, with no error. Anything added here that derives from a templated control needs its own `Template` setter *and* a test that finds a real part in the visual tree — a property assertion alone would have passed.

### Phase 3 — The windowing layer ✅ done

*Landed as planned; `EmuSen.Galaxia` was added as the ProjectReference Phase 0 deferred. 17 tests, full suite 2,154. Built as documented in `EmuSen_LunaP.md` §9. Three notes:*

- ***`ToolWindow` is deliberately inert by default.*** *Both its features are opt-in (`WindowKey` enables geometry persistence, `ClosesOnEscape` enables Escape). Phase 6 rewrites a dozen windows onto this base, and a base class that silently changed how any of them close or where they open would turn every one of those migrations into a behaviour change hidden inside a refactor.*
- ***`StartPolling()` is called by the derived constructor, not the base one.*** *Priming from the base constructor calls `Refresh()` before the derived class has assigned its fields — `CoretopWindow` would render against a `_target` that is about to be set. `Opened` starts it anyway, so forgetting costs a later first paint, not a dead window.*
- ***The suspension tests assert on timer state, not tick counts.*** *Counting ticks means racing a real clock inside a dispatcher the test is itself blocking. `IsPolling` is public for exactly this. Non-vacuousness was checked by mutation: replacing the visibility gate with `true` fails both tests.*

<details>
<summary>Original plan text</summary>

This is the part that makes it a *framework* rather than a widget bag.

- **`ToolWindow`** — base class applying the theme, standard min-sizes, `Esc` to close, and per-window geometry persistence keyed by a `WindowKey` string (through `EmuSen.Galaxia`). Every dashboard window inherits it.
- **`PollingWindow : ToolWindow`** — declare `Interval` and override `Refresh()`; construction, start, prime, `Closed`-stop and disposal happen once, in one place. **Additionally suspends the timer while the window is not visible**, which none of the five current copies do and which the weak-machine effort wants.
- **`WindowSlot<TWindow>`** — the at-most-one-else-`Activate()` pattern (§1.3) as a type, absorbing the nullable field, the `Closed` unhook, and the `Dispatcher.UIThread.Post` marshalling that only Hotaru currently needs:
  ```csharp
  _coretop.Show(owner: this,
                create: () => new CoretopWindow(target),
                refresh: w => w.UpdateTarget(target));
  ```
  Seven call sites collapse onto it; `Hotaru/Views/DebugWindows.cs` becomes two `WindowSlot` fields. Its "refresh if open, never create, never steal focus" variant (which `UpdateCoretopWindowTargetIfOpen` implements by hand today) becomes a second method rather than a second hand-written policy.
- **`Dialogs`** — `ConfirmAsync`, `ErrorAsync`, `PickFileAsync`, `PickFolderAsync`, `SaveFileAsync`. Five hand-rolled `StorageProvider` blocks become five calls.

</details>

### Phase 4 — The fluent surface ✅ done

*11 tests, full suite 2,165. Built as documented in `EmuSen_LunaP.md` §11. Two notes:*

- ***The planned names work, but only because that was checked rather than assumed.*** *`Margin` and `Spacing` are existing properties, so `control.Margin(12)` looks like it cannot compile. It does — C# falls back to extension methods when member lookup produces something that is not a method group. A throwaway probe project confirmed it before the API was designed; the fallback would have been an invented vocabulary (`Pad`, `Spaced`) that broke the one-vocabulary rule for no reason.*
- ***The success criterion is a test, not a claim.*** *`DashboardShapeTests` builds a `CoretopWindow`-shaped dashboard over plain data with no `.axaml` and drives it end to end. `GalleryWindow` was also rewritten onto the fluent surface, and its render tests passed unchanged — the fluent spelling produces an equivalent visual tree, not merely a compiling one.*

<details>
<summary>Original plan text</summary>

Deliberately last of the build phases, because it must compose Phase 2/3's types rather than raw panels — building it first would produce a fluent API over `StackPanel` that the controls then have to fight.

Small and mechanical: `Ui.Stack`, `Ui.Dock`, `Ui.Cols`, `Ui.Scroll`, `Ui.Section`, and extension methods `.Margin()`, `.Spacing()`, `.Grow()`. The target is that `BuildMeterRow`'s three manual `Grid.SetColumn` calls become one `Ui.Cols("140,*,55")`.

Success criterion: a new dashboard window is a constructor and a `Refresh()` body, with no `.axaml` file if its author doesn't want one — and no new concept the XAML path doesn't also have.

</details>

### Phase 5 — Harness support ✅ done

*Full suite 2,168. Built as documented in `EmuSen_LunaP.md` §13. Three notes:*

- ***`EMUSEN_UI_DUMP` changed meaning, from a file path to a directory.*** *The file-path form had already failed: one test appended `_{console}` to the basename to get three files out of it, and the two sites using it disagreed on BMP versus PNG. Both doc references in `EmuSen_Settings_Reference.md` were updated.*
- ***Baselines are recorded, not committed.*** *Reference images churn on any font, Skia or theme change, and a stale one fails looking exactly like a real regression. `AssertMatchesBaseline` is a no-op unless `EMUSEN_UI_BASELINE` is set, so nothing goes vacuous in CI, and the migration workflow records from the previous commit in one command. Verified by mutation: a one-value change to a single colour channel failed the comparison with a pixel count.*
- ***The layering rule is enforced now, not just documented.*** *`Common/LeafAssemblyTests.cs` already pinned Endymion/Nehellania/Serenity/Galaxia; LunaP joined it. §3's rule was prose until this phase, and adding one `ProjectReference` in a hurry is exactly what would otherwise go unnoticed.*

<details>
<summary>Original plan text</summary>

Per the repo's standing rule, testing extends `EmuSen.WiseMan` rather than launching a window. The infrastructure is already there (`HeadlessUnitTestSession`, `UseSkia`, `CaptureRenderedFrame`, the `EMUSEN_UI_DUMP` BMP escape hatch in `Mistress/InputSettingsWindowRenderTests.cs`) but every render test re-types it.

- `UiTest.Run(…)` over `Session.Dispatch`.
- `UiTest.Capture(window)` → pixels, honouring `EMUSEN_UI_DUMP` generically instead of per-test. **Phase 1 hand-rolled exactly this as a throwaway to prove the palette swap changed no pixels, then deleted it** — a reusable golden-image comparison is worth having, and rebuilding it once per migration is not. Note the trap Phase 1 hit: `VstopWindow` prints live pid/uptime/CPU, so it can never be a golden-image target; the API should make "this window is not deterministic" an explicit choice rather than a surprise.
- `UiTest.AssertLaidOut(window)` — the ">8 distinct colours, or layout failed" check, written once instead of inline.
- **`GalleryWindow`** in LunaP itself: every control, one window. One render test then covers the whole kit, and it doubles as the visual reference when adding a control.

</details>

### Phase 6 — Migrate ✅ done

*Six commits, one window group each, full suite green throughout and 2,180 at the end. **863 lines net removed from the two frontends; eight `.axaml` files deleted.** Written up in `EmuSen_LunaP.md` §11. What is worth carrying forward:*

- ***The pixel baseline paid for itself twice.*** *`CoretopWindow`'s empty state came out 11,060 pixels wrong on the first attempt (`HintText` is 11pt; the original "No ROM loaded." was body-sized), and `PreferencesWindow` turned out to have never shown its own Close button — content needing ~420px in a window fixed at 330 with `CanResize=false` and no scrolling. That bug long predates the toolkit; the migration's verification is just what surfaced it.*
- ***Write the tests against the unmigrated window first.*** *`CoretopWindow` and `FeedWindow` had no coverage at all. Writing it first also corrected an assumption of mine — the no-core state does draw one `ProgressBar`, because the sprite bar is a fixed part of the layout sitting at zero.*
- ***Change lookups, not test bodies.*** *The eleven `DianaOSShellWindow` tests survived the rewrite with only their three lookup helpers changed. They drive real key routing, so intact bodies are what makes them a safety net rather than a restatement of the new code.*
- ***The two `CoretopWindow`s now differ by six lines,*** *all namespace or comment — the deliberate consequence of sharing widgets but not windows. §11.3 records the one remaining option if that residue is ever worth removing.*

<details>
<summary>Original plan text</summary>

One window per commit, its existing WiseMan tests staying green throughout. Suggested order, easiest-first, each step delivering real deletion:

1. `VstopWindow` — `PollingWindow` + `MeterList`. Deletes a `BuildMeterRow` copy.
2. Both `CoretopWindow`s — `PollingWindow` + `MeterList` + `RgbaImageView` ×2. Deletes two more `BuildMeterRow`/`ColorForPercent`/`ToBitmap` copies; the two files shrink to their genuinely different parts, and §1.4's justifying comment gets replaced with a pointer to this document.
3. `FeedWindow` — `PollingWindow` + `RgbaImageView`.
4. Both DianaOS console windows — `ConsolePane`. Deletes a byte-identical `.axaml`.
5. `PreferencesWindow`, `DebugSettingsWindow` — `FieldRow`/`PathPickerRow`/`Dialogs`.
6. `MainWindow`'s five window-opening handlers and `DebugWindows` — `WindowSlot`.

</details>

`MainWindow` and `GameWindow` themselves (1,379 and 893 lines) are **not** migrated wholesale. They own the frame driver, the emulation thread and hotkey dispatch; only their window-opening and dialog code is in scope. Rewriting either is a different project and not this one.

---

### Phase 7 — The widget library proper

**Added 2026-08-04, on the project owner's direction**, after Phase 2 landed: LunaP is not only a de-duplication exercise, it is *the* common windowing widget set, and it is expected to grow toward the feature surface an EmulationStation-style shell needs.

Named so far: **dropdowns, switches/toggles, tabs, and filters.** Sequenced after Phase 6 rather than before it, for one specific reason — Phases 2 and 6 are anchored to widgets the codebase demonstrably needs today, and every one of them can be validated by deleting the hand-written copy it replaces. Phase 7's widgets have no such anchor yet, so they are the first work here that is genuinely speculative, and the cheapest way to stop them being speculative is to have the migration done first: a real consumer is a far better specification than a guess.

Concretely, when this phase starts:

- **Prefer wrapping over reinventing.** Avalonia already ships `ComboBox`, `ToggleSwitch`, `TabControl`. The value LunaP adds is the *theme* and a consistent API, not a reimplementation — the same relationship §4/Phase 2's controls have to `ProgressBar` and `TextBox`. A LunaP widget that reimplements a working Avalonia one needs a stated reason.
- **"Filters" is the one that is not a widget.** A filter bar over a game library is a data concept (predicate, facets, live result count) with a widget attached, and it will want a home for the non-visual half. Decide then whether that half is in LunaP at all — §3's layering rule is the test, and a filter model that knows what a "system" or a "ROM" is fails it.
- **Theming stops being deferrable.** §6's open question — how themes are authored — is answerable at leisure while there is one dark palette and eleven controls. A tabbed, skinnable browsing shell is where it becomes load-bearing, so it should be decided at the *start* of this phase rather than discovered in the middle of it.

---

## 5. What this is not

- **Not an MVVM/binding framework.** The codebase is deliberately code-behind with direct control manipulation. No ReactiveUI, no CommunityToolkit, no `INotifyPropertyChanged` layer smuggled in under "framework".
- **Not a re-skin.** Phase 1 must be pixel-identical. Visual changes are a later, separate, opt-in conversation.
- **Not game presentation.** That is `EmuSen.Serenity` and it is already right.
- **Not speculative launcher architecture.** LunaP is justified entirely by duplication that exists *today* in Mistress and Hotaru. The launcher benefits, but nothing here is built for it in advance — the same discipline `EmuSen_Launcher_Multicore_Gameplan.md` §0 sets out.

---

## 6. Open questions, deliberately not decided here

- **Theming beyond one dark theme.** The launcher's Phase 4 wants user-authored themes; LunaP's `ResourceDictionary` is the natural seam, but whether themes become files, `StyleInclude`s or something EmulationStation-shaped is that project's decision, not this one's. Phase 1 should not pre-empt it.
- **Whether `MeterRow` should animate.** Every meter currently jumps 4×/second. Smoothing is a real question and is orthogonal to sharing the widget; decide it after the migration, in one place, instead of three.
- **Whether `EmuSen.Serenity` eventually references LunaP.** No reason today — Serenity has no chrome. Left alone.
- **Hotkey/keybinding help overlays.** Both frontends have hotkeys, neither has a UI for discovering them. A plausible future LunaP control, but not scoped here.

---

## 7. Bookkeeping this plan creates

### 7.1 Comment debt at the migration sites

The files Phase 6 touches carry the project's largest comment-rule violations: `Hotaru/Views/CoretopWindow.axaml.cs` opens with a 28-line block, `Mistress/Views/CoretopWindow.axaml.cs` with 20, `Hotaru/Views/DebugWindows.cs` with 14. **Migrating a file is the moment to move its prose here** (or to `EmuSen_Frontend_Driver.md` where it is genuinely driver-specific) and leave the one-line pointer the repo convention asks for. Do not carry those blocks into LunaP.

### 7.2 The naming doc

`EmuSen_Core_Naming_Scheme.md` §10 records the lesson that infrastructure project names are drawn from the same Sailor Moon pool as cores but were never tracked in that doc — which is how `Endymion` got claimed twice. **`LunaP` must be recorded there when Phase 0 lands**, together with the note that it is distinct from `Luna`, the reserved codename for the Nintendo DS core (`Cores/Nintendo/Luna - DS/`) — the same kind of collision note §8 already carries for `Wiseman`/`EmuSen.WiseMan`.

### 7.3 This document's own successor

Once Phase 6 completes, this gameplan stops being a plan. At that point it should be replaced by a reference doc (`EmuSen_LunaP.md`) covering what the controls are and how to add one — the shape `EmuSen_Cauldron.md` and `EmuSen_Crystal_Scheduler.md` already have — with this file retired to `Man pages/Old/`.
