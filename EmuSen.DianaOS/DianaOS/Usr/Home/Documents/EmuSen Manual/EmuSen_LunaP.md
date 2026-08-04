# EmuSen.LunaP — the shared Avalonia toolkit

*This revision (2026-08-04): Phases 0 and 1 landed — the project exists, the theme is shared, and both frontends plus WiseMan bootstrap through it. No controls or window scaffolding yet; those are Phases 2 and 3 of `EmuSen_LunaP_Gameplan.md`, which remains the plan of record for everything not yet built. This doc covers only what is real.*

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

## 5. Where to look next

- **`EmuSen_LunaP_Gameplan.md`** — the plan of record: the full duplication audit (§1), the settled decisions (§2), Phases 2–6 (controls, window scaffolding, the fluent surface, harness support, migration), and the questions deliberately left open (§6).
- **`EmuSen_Launcher_Multicore_Gameplan.md`** — the launcher this project is eventually for. Its Phase 4 (theming) is why §2's palette is a resource dictionary rather than a set of constants.
- **`EmuSen_Core_Naming_Scheme.md` §11** — the name reservation and the `Luna`/`LunaP` collision note.
