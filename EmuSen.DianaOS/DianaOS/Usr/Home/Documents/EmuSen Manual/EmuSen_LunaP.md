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

A `PackageReference`, from a folder feed, in four projects: `EmuSen.Mistress`, `EmuSen.Hotaru`, `EmuSen.Serenity` and `EmuSen.WiseMan`.

    dotnet pack src/EmuSen.LunaP/EmuSen.LunaP.csproj -c Release \
        -o "<path>/EmuSen Project/local-packages"

`NuGet.config` points at `local-packages/`, which is gitignored — the package is built from the other repository and is not this one's to carry. Replace that source with a real feed when the package is pushed to one.

**The frontends fill in the settings seam**, two lines each in `Program.cs`, beside the `ConfigDiagnostics.Sink` line that was already there:

    LunaSettings.Store = new JsonSettingsStore(ConfigStore.Directory);
    LunaSettings.Diagnostics = ConfigDiagnostics.Report;

That is the whole adapter, and its being two lines rather than a class is why no project was needed to hold it. `windows.json` and `luna.json` stay exactly where EmuSen has always put them, and a theme that will not load still reports on EmuSen's own sink.

### 3.1 What the split costs

Stated plainly, because it is a real cost and it is paid daily: **iterating on the toolkit while working on EmuSen now takes a `dotnet pack` and a version bump.**

NuGet caches by package id **and** version. Repacking at a version already in `~/.nuget/packages` does not propagate, and the build then fails on code that was just written as though it did not exist. Either bump `<Version>` in LunaP's csproj, or `rm -rf ~/.nuget/packages/emusen.lunap` before repacking. `NuGet.config` carries this warning too, because that is the file somebody will be looking at when it happens.

`EmuSen.Cauldron` and `EmuSen.Galaxia` were made packable only because LunaP named them and a consumer outside this repository could not resolve a `ProjectReference`. Nothing outside wants them now, so both are back to `IsPackable=false` with their package metadata kept in place in case that changes.

## 4. What stayed behind

**`ThemeVocabularyTests`**, in `EmuSen.WiseMan/LunaP/`. It loads `avares://EmuSen.LunaP/Theme/Palette.axaml` out of the package and asserts every key in it is documented as a token in this project's own `man theme` page. That is an assertion about **EmuSen's documentation keeping up with the toolkit**, which is EmuSen's business to keep and not the toolkit's to enforce — so it runs here, against the package, and the other 132 LunaP tests went to the other repository.

**`VisualQuery`**, in the same folder — typed visual-tree lookups, used by `CoretopWindowTests`, `VstopWindowTests` and `FeedWindowTests`. Twenty lines of `GetVisualDescendants().OfType<T>()` wrappers. LunaP's own suite has its own copy, and duplicating that rather than inventing a shared test-helper package is the right trade at this size; it is recorded here so it is a decision rather than an accident.

**`LeafAssemblyTests`** still pins what the toolkit carries — `LunaP_references_nothing_of_EmuSen` — now against the package assembly rather than a project in the same solution. It means the same thing and it is the assertion that would notice the split quietly regressing.

That test has a blind spot found by sabotaging it and watching it pass: `Assembly.GetReferencedAssemblies()` cannot see a dependency used only for `const` values, because the compiler inlines a constant and elides the reference. A first attempt to redden it used `ConfigStore.ProgramDirName`, a `const string`, and the built assembly named Galaxia nowhere. Repeating it against `ConfigStore.Directory`, an ordinary static property, reddened it immediately. A `const` carries no behaviour so nothing it inlines can drag a library in, but the guard covers less than it looks like it does — and the same hole is in every other assertion in that file.

## 5. Where to look next

- **`EmuSen.LunaP/docs/LunaP.md`** — the design record, in the other repository. §19 is what had to move for the split; §20 is the move.
- **`EmuSen_LunaP_Gameplan.md`** — the plan the toolkit was built to, and what each of its seven phases taught. Kept here because it is the record of work done inside this project.
- **`EmuSen_Input.md` §4.3** — `DefaultPadKeyMap`, in the project that owns it now.
- **`EmuSen_Cauldron.md`** — `ICoreTelemetry` and the snapshot/provider contract `CoretopWindow` consumes; §3.1 for the Cauldron-versus-`IDebugTarget` split.
