# EmuSen.LunaP — the Avalonia toolkit, and where it went

*This revision: **LunaP is its own repository.** The full design record went with it. What is left here is what EmuSen needs to know: where the toolkit is, how this project consumes it, what it costs to work on both at once, and the one test about LunaP that stayed behind because it is about EmuSen's documentation rather than about the toolkit.*

---

## 1. Where it is

<https://github.com/RedQuE3n/EmuSen.LunaP>

`docs/LunaP.md` in that repository is the design record — all twenty sections of it, kept from the first commit, including the layering rule stated three different ways as the question it answered changed. Everything this page used to say is there, unedited except for the pointers back here. If you are looking for why a control takes plain data, what `AssertStable` is for, or why mutating `Application.Styles` at runtime strips realized controls, it is in that file and not this one.

The history came across too: sixteen commits, from *"Start LunaP: one theme for both frontends"* to the one that cut the last EmuSen reference, replayed onto their own line rather than squashed. `git log` there is the real thing.

## 2. Why it left

The short version. LunaP was always allowed to reference `EmuSen.Galaxia` and, after §16's amendment, `EmuSen.Cauldron` — both dependency-free leaves, so a launcher taking the toolkit did not take a core. That rule was sound and it was answering the wrong question once the toolkit was worth publishing. The question became *"can somebody outside this repository resolve this at all"*, and nothing named EmuSen passes it.

Three things carried those references and each went somewhere different:

| What | Where it went |
|---|---|
| `WindowPlacementStore`, `LunaTheme` reading `Galaxia.ConfigFile` | A seam — `Settings/ISettingsStore`, filled in by the host. §3 below. |
| `Input/DefaultPadKeyMap.cs` | `EmuSen.Endymion`, which already owns the mapping of physical input onto `PadButton`. `EmuSen_Input.md` §4.3. |
| `Dashboards/CoretopWindow.cs` | `EmuSen.Serenity`, which already is the core-agnostic Avalonia layer. |

No new project was created to hold either, and the near-miss recorded in the old §16.1 — a whole assembly stood up for one 137-line file and deleted the same day — is why. Asking what each file was *about* got a better answer than asking what it *referenced*.

## 3. How EmuSen consumes it

A `PackageReference` from **nuget.org**, in four projects: `EmuSen.Mistress`, `EmuSen.Hotaru`, `EmuSen.Serenity` and `EmuSen.WiseMan`. There is nothing to set up and nothing to hand-carry; a bare clone builds.

It was a folder feed for exactly as long as it took to publish the package, and `NuGet.config` records what that cost. LunaP is published from a tag by a workflow holding no credential at all — NuGet Trusted Publishing exchanges a GitHub OIDC token, which proves which repository and which workflow *file* is running, for a key valid for minutes.

**The frontends fill in the settings seam**, two lines each in `Program.cs`, beside the `ConfigDiagnostics.Sink` line that was already there:

    LunaSettings.Store = new JsonSettingsStore(ConfigStore.Directory);
    LunaSettings.Diagnostics = ConfigDiagnostics.Report;

That is the whole adapter, and its being two lines rather than a class is why no project was needed to hold it. `windows.json` and `luna.json` stay exactly where EmuSen has always put them, and a theme that will not load still reports on EmuSen's own sink.

### 3.1 What the split costs

Stated plainly, because it is real and it is paid by whoever changes both at once: **a change to the toolkit reaches this project only through a published version.** Tag LunaP, let the workflow publish, bump the `PackageReference` here.

For a change you are still iterating on, a local `dotnet pack` into a folder source is still the fastest loop — and the trap is waiting there: NuGet caches by package id **and** version, so repacking at a version already in `~/.nuget/packages` does not propagate and the build fails on code that was just written. Either use a prerelease version that changes every pack, or `rm -rf ~/.nuget/packages/emusen.lunap` first. `NuGet.config` carries the warning, because that is the file somebody will be looking at when it happens.

What this bought is worth naming against that cost: a clone of this repository builds with `dotnet build`, and so does a clone of `EmuSen.Pegasus`, and so does a clone of anything else that ever wants the toolkit.

`EmuSen.Cauldron` and `EmuSen.Galaxia` were made packable only because LunaP named them and a consumer outside this repository could not resolve a `ProjectReference`. Nothing outside wants them now, so both are back to `IsPackable=false` with their package metadata kept in place in case that changes.

## 4. What stayed behind

**`ThemeVocabularyTests`**, in `EmuSen.WiseMan/LunaP/`. It loads `avares://EmuSen.LunaP/Theme/Palette.axaml` out of the package and asserts every key in it is documented as a token in this project's own `man theme` page. That is an assertion about **EmuSen's documentation keeping up with the toolkit**, which is EmuSen's business to keep and not the toolkit's to enforce — so it runs here, against the package, and the other 132 LunaP tests went to the other repository.

**`VisualQuery`**, in the same folder — typed visual-tree lookups, used by `CoretopWindowTests`, `VstopWindowTests` and `FeedWindowTests`. Twenty lines of `GetVisualDescendants().OfType<T>()` wrappers. LunaP's own suite has its own copy, and duplicating that rather than inventing a shared test-helper package was judged the right trade at this size; it is recorded here so it is a decision rather than an accident.

*Superseded on 2026-08-10.* **The shared test-helper package exists now** — `EmuSen.LunaP.Testing`, carrying `VisualQuery`, `UiTest`, `AssertLaidOut` and the headless app builder. What tipped it was not this copy but the discovery that this copy is **byte-identical** to LunaP's apart from its namespace line, and that `EmuSen.Pegasus` had written a third harness in F# because it could reference neither. Three implementations of the same twenty lines is past the size where duplicating is the cheaper answer.

This file has not been migrated onto it, deliberately and for now: `EmuSen.WiseMan` is a 2,594-test suite and swapping its harness is its own piece of work with its own verification, not a rider on a package bump. The option is live and the reasoning above is no longer the reason to decline it. `LunaP.md` §22.8 is the package.

**`LeafAssemblyTests`** still pins what the toolkit carries — `LunaP_references_nothing_of_EmuSen` — now against the package assembly rather than a project in the same solution. It means the same thing and it is the assertion that would notice the split quietly regressing.

That test has a blind spot found by sabotaging it and watching it pass: `Assembly.GetReferencedAssemblies()` cannot see a dependency used only for `const` values, because the compiler inlines a constant and elides the reference. A first attempt to redden it used `ConfigStore.ProgramDirName`, a `const string`, and the built assembly named Galaxia nowhere. Repeating it against `ConfigStore.Directory`, an ordinary static property, reddened it immediately. A `const` carries no behaviour so nothing it inlines can drag a library in, but the guard covers less than it looks like it does — and the same hole is in every other assertion in that file.

## 5. Where to look next

- **`EmuSen.LunaP/docs/LunaP.md`** — the design record, in the other repository. §19 is what had to move for the split; §20 is the move.
- **`Man pages/Old/EmuSen_LunaP_Gameplan.md`** — the plan the toolkit was built to, and what each of its seven phases taught. Retired there on 2026-08-16, on its own §7.3's instruction, once this document existed to replace it; still the record of work done inside this project and still worth reading for *why*, which is why it is cited by name rather than left to rot.
- **`EmuSen_LunaP_Adoption_Gameplan.md`** — its successor and the live plan: what the frontends should adopt from LunaP 0.8.0, whose surface postdates the migration the older plan describes.
- **`EmuSen_Input.md` §4.3** — `DefaultPadKeyMap`, in the project that owns it now.
- **`EmuSen_Cauldron.md`** — `ICoreTelemetry` and the snapshot/provider contract `CoretopWindow` consumes; §3.1 for the Cauldron-versus-`IDebugTarget` split.

---

## 6. The 0.3.0 bump

The first time the toolkit changed under this project since the split, and therefore the first real exercise of §3.1's cost. All four `PackageReference`s moved 0.2.0 → 0.3.0 together.

**What it cost here was one man page and nothing else.** `ThemeVocabularyTests` compares `man theme` against the parser's allow-lists by set equality *in both directions*, so a toolkit that grows a control or a palette key reddens this suite until the documentation catches up. It did exactly that, on three assertions at once: the new `empty-state` element, its `.message` and `.detail` parts, and the `--luna-error` / `--luna-success` / `--luna-info` tokens. Four entries in `ManPages.cs` closed it.

**That is the arrangement working, not failing.** §4 kept that test here on the grounds that EmuSen's documentation staying in step with the toolkit is EmuSen's business. The bill for that decision arrives as a red suite in a repository the change never touched, and it is worth knowing in advance that the redness is expected and what closes it — which is why it is written down here rather than rediscovered next time.

### 6.1 Three copies of the frame hand-off, deleted

0.3.0 carries `EmuSen.LunaP.Threading`, and `Latest<T>` in it is the mechanism `EmuSen.Serenity`'s `FramePresenter`, `EmuSen.Mistress`'s `MainWindow` and `EmuSen.Hotaru`'s `GameWindow` had each written out — byte-identically, which is what argued it into the toolkit in the first place. All three now call `Offer`.

**All three carried the same bug**, found by generalising them rather than by anything going wrong here: the scheduled flag was cleared *after* the frame was handed to the control, so a frame submitted while the UI thread was inside `UpdateFrame` could neither schedule a callback nor be collected by the running one. It sat until the next frame displaced it.

Invisible at 60 fps — the next frame is 16 ms away and carries the fix — and visible the moment the stream stops, where the frame at risk is the last one drawn. `EmuSen_Serenity.md` §4 carries the correction in full; `LunaP.md` §22.1 has the fix and the test that pins it.

The point worth keeping is the one about duplication rather than the one about frames: **three identical copies meant one bug in three places and no single place to fix it.** Deleting them is what makes that impossible to repeat.
---

## 7. The 0.5.0 bump, and what a toolkit fix is worth downstream

All four `PackageReference`s moved 0.3.0 → 0.5.0 together, skipping 0.4.0. **The bump cost nothing at all** — no man page, no code change, 2598 tests green on the first run — which is the opposite of §6's experience and worth the same amount of writing down.

The reason is stated in `LunaP.md` §24.7 and holds: 0.4.0 added a light palette column and 0.5.0 added accessibility, and **neither added a control or a palette token**. `ThemeVocabularyTests` compares `man theme` against the parser's allow-lists by set equality in both directions, so it goes red for new vocabulary and is indifferent to everything else. §6 predicted the bill would arrive for a toolkit that grows a control; this bump grew none, and the prediction held in the negative direction too.

### 7.1 The measurement, and what the bump bought for free

Eleven windows, probed with `ControlAutomationPeer.CreatePeerForElement` — the route a screen reader's platform bridge takes — counting how many of the controls a keyboard can reach announce a name at all. The same instrument `LunaP.md` §24.1 and `Pegasus_Design.md` §13.1 used.

**The identical windows, the identical application code, two versions of the toolkit:**

| Window | LunaP 0.3.0 | LunaP 0.5.0 |
|---|---|---|
| `DebugSettingsWindow` | 1 of 16 | **16 of 16** |
| `PreferencesWindow` | 4 of 8 | **8 of 8** |
| `InputSettingsWindow` | 20 of 23 | 22 of 23 |
| `MainWindow` | 20 of 22 | 22 of 22 |
| `CheatDatabaseWindow` | 8 of 10 | 9 of 10 |
| `ActiveCheatsWindow` | 12 of 13 | 13 of 13 |
| `DianaOSConsoleWindow` | 1 of 2 | 2 of 2 |
| **Total** | **69 of 97** | **95 of 97** |

**Twenty-six controls became reachable to a screen reader without a line changing in this repository.** `DebugSettingsWindow` is the clearest case: fifteen of its sixteen tab stops are `LunaSwitch`es, and every one of them announced as an unnamed button because §14.1 of the toolkit's own record put their labels in `OnContent` rather than `Content`, which is where Avalonia's toggle peer looks. `PreferencesWindow` doubled because `PathPickerRow` and `FieldRow` now name their own parts.

This is the strongest evidence so far for §3.1's arrangement being worth its cost. A defect in shared chrome is fixed once and lands in four applications; the same defect in three hand-rolled copies is what §6.1 had to go and delete.

### 7.2 The two the toolkit could not reach

**A `Slider` with no name**, in `InputSettingsWindow`. Its label is the `TextBlock` to its left, which a reader has no reason to connect to it. A nameless slider is worse than a nameless button: it announces a bare number that changes as you press an arrow key, with nothing to say what it measures.

**A hand-rolled path row** in `CheatDatabaseWindow` — a read-only `TextBox` and a `Browse...` button, built as XAML rather than as `luna:PathPickerRow`, so it got none of the naming the toolkit control gives itself.

The first repair was to name the copy by hand, and that was the wrong repair. **It is the toolkit control now.** Naming the copy would have closed the visible gap and kept the thing that produced it: `PreferencesWindow` used the real `PathPickerRow` three times and this window spelled out its own, the two were indistinguishable on screen, and they stopped being indistinguishable only when 0.5.0 taught the real one to name its own parts. The copy did not learn. That is §6.1's argument about three copies of a frame hand-off, in slower motion — **a copied control does not go wrong the day it is written, it goes wrong the day the original learns something.**

The swap also deleted the window's own folder-dialog call, its `null`-check for a cancelled pick, and one of the two `IsEnabled` assignments it was making by hand: `PathPickerRow.PathPicked` raises only for a real pick, and disabling the row disables the box and the button together.

`HandRolledControlTests` is what stops it coming back. A `Browse...` button whose `TemplatedParent` is not a `PathPickerRow` is a picker somebody rebuilt, and the test names the window it found one in — checking the templated parent rather than counting buttons is what makes it specific enough to be worth having. Restoring the old XAML turns it red with exactly the sentence a reader needs: *"CheatDatabaseWindow: a 'Browse...' button outside a PathPickerRow"*. Its companion asserts the two windows really do hold one and three rows, so it cannot pass by there being no path rows at all.

### 7.3 Fourteen buttons, two captions

The real defect in this repository, and it does not show up in the count above because every one of these buttons *has* a name.

`InputSettingsWindow` builds seven binding rows, each with `Rebind Key`, `Clear`, `Rebind Pad` and `Clear Pad`. A sighted user reads across the row to see which button it belongs to. A screen reader user got the same two or four words, seven times over, with nothing to tell them apart.

**The captions stay.** The obvious repair — renaming the button to `Rebind A on SNES` — breaks voice control, because somebody saying "click rebind key" needs those words to be the accessible name. So the row context goes in `HelpText`, which is announced after the name. Same trade `LunaP.md` §24.2 made for `PathPickerRow`'s Browse buttons, and the reason it is the same trade is that it is the same problem.

### 7.4 What only the application could say

`LunaP.md` §24.2 draws a line: the toolkit gives `MeterList` and `RgbaImageView` a control type and puts them in the automation tree, and deliberately supplies **no name**, because it cannot know whether a run of meters is audio channels or core load, and a guessed description of a live pixel buffer would be a wrong alt text — which is believed — rather than a missing one, which is asked about.

This is the consumer's half of that line. `CoretopWindow` names its audio meters, its palette view and its tile sheet; `FeedWindow` names its one control "Game screen". Five list boxes across four windows gained names, because a list whose rows announce as themselves is still a list of nothing in particular. Three status lines became `Polite` live regions — `MainWindow`'s is assigned from seventeen places and carries transient results, permanent state and failures on one line, none of which arrives for a reader who is not watching that corner of the window.

### 7.5 The guards, and a test that survived a sabotage it should have caught

`EmuSen.WiseMan/LunaP/AccessibilityTests.cs`, 21 tests, plus `HandRolledControlTests.cs`, 2. The suite is **2621**, from 2598.

Six sabotages were run. Five turned the expected test red immediately. The sixth stripped the help text off every console rebind button and **`Every_rebind_button_says_which_binding_it_belongs_to` stayed green.**

The cause is a fact this repository already knew and had written down: **Avalonia realises only the selected tab.** The test built the window with no console, which opens on General, so the seven console binding rows it claimed to cover were not in the visual tree at all — it was asserting over the hotkey rows and nothing else. `InputSettingsWindowRenderTests` had a comment saying exactly this, with a `§` citation, and the new test was written without reading it.

It is a `[Theory]` over `null`, `NES` and `SNES` now, and the sabotage turns two of the three red. The general lesson is the one §5 of the design record keeps making — a guard is not trusted until it has failed on purpose — with a sharper corollary: **a guard whose subject is empty passes, and passing is what that looks like from the outside.** `Pegasus_Design.md` §13.5 records the same failure arriving by a different route, where a window that was never shown reported no tab stops at all.

### 7.6 What is not covered

**No screen reader has been run against any of this.** Every measurement is of Avalonia's automation tree, which is what a platform bridge reads; it is not Orca or NVDA reading it aloud. Being in the control view is necessary and is not the same as verified end to end.

`GameWindow` is not in the table. It is 888 lines that build a fullscreen surface with a menu, and it has no ordinary tab stops to count; what it needs is a pass of its own against how it behaves during play rather than at rest.

`ToolTip` remains unused across the repository, and no control has an explicit `TabIndex` — tab order follows the visual tree and read correctly in every window measured, so there was nothing to reorder.
