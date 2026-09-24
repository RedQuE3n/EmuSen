# Running more than one core

*This revision: initial. Written when Moon (NES) became the second real core and the frontends had to stop assuming there was only one.*

Companion docs: `EmuSen_Input.md` (the pad contract, which came out of the same pass), `Moon_Core.md` §2 and `Venus_CPU.md` §8.5d (each core's own timeline — there is deliberately no shared timing assembly; see §9.1), `EmuSen_Project_Overview_v2.md` (the directory/namespace map), `EmuSen_Launcher_Multicore_Gameplan.md` (the separate, later-stage launcher project this is *not*).

---

## 1. Why this exists

`ICore` and `IDebugTarget` were both written to be core-agnostic and both are. The **frontends** were not. Every one of them opened with some variant of:

```csharp
var core = new VenusCore(headless);
core.LoadRom(path);
var debugTarget = new SnesDebugTarget(core.Cpu!, core.Bus!, core.Renderer!, ...);
```

Four copies of that, in `EmuSen.Hotaru`, `EmuSen.Mistress`, `EmuSen.Pharaoh` and `EmuSen/Common/EmulatorSession.cs`. Each one knew the concrete core type, its constructor, which debug target went with it, and which cheat codecs to build. Adding a second core meant editing all four the same way and hoping none drifted.

That was fine while "which core" had one answer. It stopped being fine the moment `.nes` had to mean something.

The rule this pass settled on: **a frontend names no core.** It hands over a path and gets back everything it needs. Anything genuinely console-specific is reached by *capability* (§5), never by `as VenusCore`.

---

## 2. `CoreFactory` — the one place a path becomes a core

`EmuSen/Cores/CoreFactory.cs`. Three entry points and a record:

```csharp
CoreBundle Load(string romPath, bool headless, CheatRegistry? cheats, ...)   // construct + LoadRom + wire
CoreBundle Bundle(ICore core, ...)                                            // wire a core you already made
ICore      Create(string romPath, bool headless)                              // construct only
```

`CoreBundle` is everything a frontend needs for one loaded ROM:

```csharp
record CoreBundle(ICore Core, IDebugTarget DebugTarget,
                  ICheatCodeCodec? CheatAutoDetectCodec,
                  ICheatCodeCodec? CheatExplicitCodec,
                  ICpuTraceSwitch? CpuTraceSwitch);
```

`Create` and `Bundle` are separate because they fail at different times and some callers only want one of them. `ForFirmwareProbe` needs a core that has *not* loaded a ROM, so it can be asked `GetFirmwareRequirements` before anything is committed (`EmuSen_Firmware.md` §3). `Hotaru`'s in-place ROM swap already holds a core and only wants the wiring rebuilt around it. Everyone else calls `Load`.

**`Bundle` is the only `switch` on a concrete core type in the project, and that is deliberate.** Something has to know that a `VenusCore` pairs with a `SnesDebugTarget`, and a type switch in one file is a better place for that than a registration mechanism nobody else needs. An unregistered core throws rather than silently returning a null target.

`headless` only means anything to a core that owns a window-ish resource; Moon ignores it.

---

## 3. `CoreCatalog` — which consoles exist

`EmuSen/Cores/CoreCatalog.cs` holds one `CoreDescriptor` per real core: display name, ROM extensions, and the libretro cheat-database folder names it claims. It is keyed by what a user would type, so both the codename and the console name reach the same core (`venus`/`snes`, `moon`/`nes`).

Everything that needs a "which consoles?" answer reads it:

| Consumer | What it takes |
| --- | --- |
| `CoreFactory.IsSupported` | `IsRomExtension` |
| `RomLibrary.Extensions` (Mistress' library scan) | `RomExtensions` |
| `MainWindow`'s Open ROM file picker | one `FilePickerFileType` per core |
| `PreferencesWindow`'s core combo | `Cores.Select(c => c.DisplayName)` |
| `DianaOSConsoleWindow`'s welcome banner | same |
| `cheat db prune` | `SupportedCheatSystems` |

Each of those was its own hardcoded copy of the list before. Three of them said `"SNES (Venus)"` as a string literal, and `RomLibrary` had `{ ".smc", ".sfc" }` — which is why, until this pass, an `.nes` file was invisible in Mistress' ROM library and unselectable in its file picker even though a working NES core existed behind it.

`Cores` is one entry per real core, not per alias — the registry holds each core twice under two names, and a "pick your console" list must not show it twice.

---

## 4. Cheats and the trace switch

Cheat *codecs* (Game Genie, Action Replay) are per-console: the same printed code means different things on different hardware, so they come out of the bundle rather than being constructed by the frontend. Both cores now fill both slots — Moon with NES Game Genie and a raw `AAAA:VV` format, which are genuinely different decoders from Venus's rather than the same ones renamed (`EmuSen_Cheats.md`). `DianaOSInterpreter.CreateDefault` still accepts null for every one of these, so a future core with no code format reports that `cheat` needs a codec instead of decoding one console's code against another's memory.

Same for `ICpuTraceSwitch`, which arms the verbose per-instruction trace: Venus supplies `VenusCpuTraceSwitch`, Moon supplies nothing yet.

One frontend case does not fit: **Mistress' cheat windows can be opened with no ROM loaded at all** (the database browser is a library tool, not a debugging one), and they still need to parse code text. Those fall back to `CoreFactory.DefaultCheatCodecs` rather than the frontend naming a codec — the fallback pair is Venus's, which keeps today's behaviour exactly, but the choice lives in the factory where a future "default console" preference would go.

---

## 5. Capability interfaces — the answer to "must this go on `ICore`?"

Venus has things no other core will ever have: HDMA, an SA-1, an SPC700, a per-scanline obj-eval/blend split. Two bad options:

1. **Put them on `ICore`.** Then the shared contract carries SNES nouns and lies for every future core, which must implement `HdmaMilliseconds` and return zero.
2. **Cast in the frontends.** `if (core is VenusCore v) ...` re-imports core knowledge into exactly the layers this pass was cleaning.

Neither. `EmuSen/Cores/CoreCapabilities.cs` holds small optional interfaces a core opts into, and frontends **probe by capability, never by core type**:

| Interface | What it says | Who implements |
| --- | --- | --- |
| `IFrameProfiler` | `LastFramePhases` — ordered `(Name, Milliseconds)` | Venus, Moon, MarsRT (`Mars_Native.md` §6.6.2) |
| `ICoprocessorHalt` | `IsHaltedOnCoprocessor`, `HaltedProcessorName` | Venus |
| `ICoprocessorLoad` | `CoprocessorClocks` — executed/offered against a per-frame budget | Venus |
| `ITraceFlushable` | `FlushVerboseTrace()` | Venus |
| `IFrameSerial`, `IRepeatedRows` | whether the picture changed, and how often its rows are shown (§14, §15) | Mars, MarsRT |
| `IFrameBufferPool` | `ReturnFrameBuffer(array)` — a lent picture handed back (§16) | MarsRT |

Same shape `IDebugTarget` already uses for `Coverage`/`CallStack`, and the `IBusTraceTarget` the validation rig grew for cycle-accurate 6502 vectors.

What this bought concretely, in Pharaoh's `perf` command: it used to name seven SNES timing fields and print an SA-1 line inline. It now iterates whatever phases the core publishes, and prints `"<core> publishes no per-phase breakdown."` for one that publishes none. Its break message reads `[BREAK] SA-1 halted at $...` without Pharaoh knowing what an SA-1 is — a core with no `ICoprocessorHalt` just says `CPU`.

**The `parent/child` phase-name convention.** Venus's seven phases are not seven slices of a frame: `obj-eval`, `blend`, `main-composite` and `sub-composite` are *inside* `ppu`. A generic printer that sums a flat list double-counts them and reports a negative unattributed time. So a name containing `/` is understood to be nested inside its parent, and only unnested phases sum to the frame. That is why they are published as `ppu/obj-eval` rather than `obj-eval`.

### Where casts still legitimately live

Three Hotaru hotkeys — backdrop dump, OAM dump, BG-scroll trace — still go through `_core as VenusCore`. These are one-console debug affordances that reach deep into `Renderer` and `Ppu` to print SNES-specific state; there is no generic thing they are an instance of. They are behind a `Venus` helper property that is null for any other core, so they no-op rather than crash.

---

## 6. Cross-core run switches

`--nobattery` was `Venus.Memory.Cartridge.BatteryRamDisabled`, a static on the SNES cartridge. Moon also writes a `.srm`, and did not check it — so the flag the harness relies on for reproducible runs (`EmuSen_Debugging_Tools_Reference_v5.md` §3.15) was silently a no-op on every NES run.

It now lives on `EmuSen/Cores/CoreOptions.cs` and both cartridges read it. Venus's old static is kept as a forwarding property so existing call sites and docs still resolve.

This is the pattern for any future switch that is a *property of the run* rather than of one console. It is deliberately not on `ICore`: it has to be set **before** `LoadRom`, which is exactly when there is no core to set it on.

**"Before `LoadRom`" is now enforced rather than assumed.** Mercury and Moon always honoured it — each reads the flag once in `LoadSram()` during `LoadRom` and latches the result. Venus did not: it re-read the static on every save and every load, so a switch flipped after a cartridge existed still reached it. `Cartridge` now takes a copy at construction, with an optional `bool? batteryRamDisabled` parameter for a caller that wants to name the value instead of inheriting it. Nothing in the project assigns the switch after a cartridge exists, so this changes no behaviour any frontend can observe — what it changes is that a *second* run in the same process can no longer disturb the first, which is what let the test suite stop serializing every cartridge test. `EmuSen_Debugging_Tools_Reference_v5.md` §3.56 has the measurement and the guard test.

---

## 7. What is still core-specific, on purpose

- **`Bundle`'s type switch** (§2) — one file, by design.
- **Hotaru's three debug hotkeys** (§5).
- **`SnesDebugTarget`'s constructor shape** — it takes `(Cpu, MemoryBus, Renderer)`, which only a Venus caller can supply. That is why `Bundle` exists: it is the one caller.
- **Pharaoh's two power-on WRAM watches** — an SNES-era investigation leftover, now guarded on a space of that name actually existing, so an NES run does not register watches that can never fire.

## 8. What is not done

- **No core swaps consoles mid-session.** `core <name> <path>` reloads through the factory, so loading a `.nes` after a `.smc` does construct a `MoonCore` — but the frontends rebuild a great deal around that and only Mistress' path is well covered by tests.
- **`AppSettings.SelectedCore` still drives nothing.** It persists a display name and no code reads it to choose a core; the ROM's extension decides. The combo is now populated from the catalog rather than a literal, which is the only part of it that improved.
- **Moon has no `ICpuTraceSwitch`.** A real gap rather than a deliberate omission. It does now have cheat codecs (`EmuSen_Cheats.md`, `Moon_Cheats.md`) and, since 2026-08-04, `IFrameProfiler` — a `cpu+apu`/`ppu` split that also fills its `coretop` load bars, see `Moon_Debug.md` §3.2.

### A bug this pass found

`EmulatorSession.ScreenHeight` was a `const int = 224` while `ScreenWidth` already asked the core. Mistress presented every frame as `(session.ScreenWidth, EmulatorSession.ScreenHeight)`, so Moon's 256×240 framebuffer would have been submitted as 256×224 — a wrong-sized blit from a buffer 16 rows longer than claimed. It is now an instance property that asks the core, like its neighbour. Worth noting as the class of bug this whole pass exists to prevent: a constant that was true of the only core there had ever been.

---

## 9. The leaf layers

Three assemblies sit below everything else and are **not allowed to know a core exists**. One of them is a true leaf — zero project references of any kind:

| Project | Project references | Job |
|---|---|---|
| `EmuSen.Galaxia` | **none** | config persistence, and `PadButton` (`EmuSen_Input.md` §3) |
| `EmuSen.Serenity` | `Galaxia` | video presentation |
| `EmuSen.Endymion` | `Galaxia` | the SDL3 device layer: audio out, gamepad in (`EmuSen_Audio_Sync.md` §7, `EmuSen_Input.md` §4) |

Serenity is the model the others were held to. Its contract is `UpdateFrame(byte[] rgba, int width, int height)` — payload plus the metadata to read it, and nothing else. Endymion's `Submit(short[] pcm, int sampleRate)` is the same contract for audio.

The rule that keeps them leaves: **a device layer is handed what it needs and reaches for nothing.** Not a core, not a session wrapper, not a global settings static. Every knob is a parameter. That is why `AudioPlayer` takes its sample rate, latency target and deviation band as constructor arguments instead of reading `AudioSettings`, which lives in `EmuSen` because the Venus DSP needs it.

`EmuSen.WiseMan/Common/LeafAssemblyTests.cs` asserts each of these reference sets against the actual compiled metadata. What it catches is a leaf *using* a type from the core assembly; what it does not catch is an unused `ProjectReference` sitting in a `.csproj`, because the compiler drops references nothing consumes.

**What is not a leaf, and should not be:** `EmuSen.Hotaru`, `EmuSen.Mistress` and `EmuSen.Pharaoh` are frontends — their whole job is to hold a core and drive it, so they reference everything. `EmuSen.DianaOS` is agnostic about which cores exist but is not a leaf for other reasons.

### 9.1 There was a fifth leaf, and timing is why it went away

`EmuSen.Crystal` was on that list until 2026-08-05: a core-agnostic timing scheduler with a master timeline, an event table indexed by a core-defined `int`, an `IClockedDevice` catch-up contract and exact clock-ratio conversion. It was built agnostic from the start, on the reasoning that §1 of this doc applies to timing as much as to anything else — every one of the reserved consoles needs a timeline, so extract it once rather than from Venus later.

**It was folded back into the two cores and deleted.** Both of them scheduled exactly one event, `ScanlineBoundary`, at a constant stride. The event table, the tie-break rule, the runaway-dispatch guard and `IClockedDevice` were exercised by nothing but the scheduler's own unit tests, and what each core actually wanted collapsed to two `long`s and a subtraction. Moon kept the one piece that carried its weight — the drift-free remainder carry, which it needs because 1364/12 is not a whole number and Venus's 1364/6 is (`Moon_Core.md` §2.1). Everything else was ceremony around a `+=`.

**The distinction worth taking from this**, because it cuts against §1's instinct and both are right in their place: `ICore`, `PadButton` and `ICoreTelemetry` were extracted to kill *duplication that already existed* — a second core arrived and the same frame loop, the same button enum, the same telemetry shape were about to be written twice. Crystal was extracted to serve consoles that do not exist yet, against a design (runtime-chosen event times, several devices contending for one timeline) that neither shipped core turned out to need. A shared abstraction is cheap to add once two callers are visibly doing the same work, and expensive to carry when its second caller is hypothetical.

That is not a verdict on schedulers. `HTime` (`$4207`/`$4208`) is still unimplemented and still the real case for a timeline with more than one appointment on it — see `Old/EmuSen_Crystal_Scheduler.md` §1, kept for exactly that argument. When it lands it should land in Venus, and it should be lifted out again only when a second core is provably asking for the same thing.

### 9.2 `Nehellania` folded into `Endymion`, and which half of the leaf rule was load-bearing

`EmuSen.Nehellania` was the gamepad half of the SDL3 layer — `GamepadManager`, `GamepadBindingMap`, `GamepadBindings`, 318 lines — sitting beside `EmuSen.Endymion`'s 262 lines of audio sink. On 2026-08-05 it was folded into Endymion and deleted. The two projects took **the same two SDL3 package references**, and the decisive fact is at the consumer end: `EmuSen.Hotaru`, `EmuSen.Mistress` and `EmuSen.WiseMan` each referenced *both*, and nothing anywhere referenced one without the other. The split bought no consumer any separability it was using.

**The interesting part is what this cost, because it exposes an ambiguity in §9's own rule.** Endymion was a *true* leaf — zero project references. Nehellania referenced Galaxia (for `PadButton` and `ConfigFile`), so the merged assembly inherits that edge and Endymion is no longer reference-free.

That turns out not to matter, and the reason is worth stating because it distinguishes two rules that had been treated as one:

| Rule | Status | What it protects |
|---|---|---|
| "true leaf" — zero outgoing references | **broken here, and it was ceremony** | nothing that exists; Galaxia is itself core-free, so nothing arrives transitively |
| "no leaf reaches the core assembly" | **kept, and it earns its keep** | a launcher browsing a ROM library with no core loaded (`EmuSen_LunaP.md` §1) |

`LeafAssemblyTests` had already encoded both — `Endymion_references_no_other_EmuSen_assembly` and the separate `No_leaf_reaches_the_core_assembly` theory — without anyone noticing which was doing the work. The strict one is now `Endymion_references_only_Galaxia`, matching Serenity exactly. The theory is untouched and still has teeth.

**The direction to keep straight**, since it is easy to argue past: "leaf" is about a project's *outgoing* references, not how many things depend on it. Every frontend needing Endymion is not an argument against Endymion being a leaf — if anything the opposite, since an assembly that widely depended-upon reaching *up* into the core would drag the core into all of them. What actually retired the strict rule was that no consumer takes Endymion without a core anyway, so zero-references guarded a case with no instance.

**What should not be folded in next:** `EmuSen.LunaP`. It shares no package with Endymion (six Avalonia packages against two SDL3 ones), neither names the other, and folding it in would put Avalonia on the dependency path of anything that just wants sound. It is also the one assembly whose core-free status has a real, roadmapped consumer.

### 9.3 Why a deleted project appears to come back

Both `EmuSen.Crystal` and `EmuSen.Nehellania` were reported as having returned, repeatedly, months after §9.1 and §9.2 deleted them. They had not. What survived was `bin/` and `obj/` — and because those are gitignored, **git cannot remove them and `git status` cannot see them**, so the directory stays in the file explorer with no source in it and looks exactly like a project that came back.

The regeneration was traced rather than guessed, and the mechanism is ordinary:

    02:51:34   checkout: moving from WiseMan to main
    02:51:35   obj/Debug/net10.0/EmuSen.Nehellania.AssemblyInfo.cs written
    02:51:53   merge WiseMan into main

The deletion happened on `WiseMan`. `main` had not yet taken it, so checking `main` out restored the `.csproj`, an IDE design-time build regenerated `obj/` one second later, and the merge eighteen seconds after that deleted the tracked files again — leaving the build output behind. Every `ls` afterwards shows a folder. Nothing was rebuilding it; nothing had to.

The general shape, worth carrying to any future pruning: **deleting a project from a branch does not delete it from the working tree.** The tracked files go, the ignored ones stay, and the leftovers are invisible to precisely the tool anyone would use to check. `git clean -xfd <path>` removes them; a plain `git clean -fd` does not, because it respects the ignore file.

The corollary is that this can still happen once more per stale ref. `backup-prepurge-WiseMan` and `backup-prepurge-main` both still contain these projects, by design — checking either out and building will recreate the directories. That is the backups doing their job, not a recurrence.

Both directories were removed on 2026-08-09 after confirming they contained no hand-written file: everything under them was SDK-generated (`AssemblyInfo.cs`, `GlobalUsings.g.cs`, `.cache`, `.deps.json`), nothing was tracked, and no `.csproj`, `.sln` entry or `ProjectReference` named either. The only surviving mention is the historical note in `EmuSen.Endymion.csproj`, which is correct and should stay.

---

## 10. The console context in Mistress

`AppSettings.SelectedCore` is one shared "which console am I working on" value, read by the game library and both cheat windows. It was inert scaffolding until a second core existed — a Preferences combo wrote it and **nothing ever read it**.

### 10.1 The bug this uncovered

Before this work, `RomLibrary.Scan` called `Directory.EnumerateFiles(dir)` — **top level only**. A real ROM collection is not laid out that way. The dev machine's is:

```
Documents/Roms/
    SNES/          48 files
    NES/           3,537 files across USA/, World/, Translated/, ...
```

Zero ROMs at the top level, so **the library screen listed nothing at all** and looked like a broken setting rather than a missing feature. The scan now recurses (`EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true }`), which also means one configured root covers every console.

Measured on that tree: **3,585 entries in 3 ms**, 48/3,537 when narrowed to one console. The walk is not the expensive part and does not need caching.

### 10.2 What the context drives

| Surface | Effect |
| --- | --- |
| Game library | scan narrowed to that console's extensions |
| Library rows | a `— <console>` tag, **only when the visible list actually spans more than one** |
| Cheat Database | systems list narrowed to that console's libretro folders (`CoreDescriptor.CheatSystemNames`) |
| Active Cheats | a code typed by hand is parsed with **that console's** codecs |

Changing the filter retargets any already-open cheat window in place, rather than leaving it showing the previous console's systems.

**A running game outranks the filter.** `MainWindow.ConsoleCodecs` prefers the loaded session's codecs, because those are the ones that can actually be applied to live memory. The filter only decides what happens with no game running.

**A console whose core has no codec says so.** Moon has neither a Game Genie nor an Action Replay codec (§4), so with NES selected the Add button is disabled and the window states why, instead of silently parsing an SNES code against NES memory.

### 10.3 Migration, and why it was necessary

`SelectedCore` defaulted to the literal `"SNES (Venus)"`, and that value is sitting in existing config files. Making the filter honour it would have hidden every NES game the first time this shipped — precisely the confusing outcome the feature exists to prevent.

Since nothing ever read the value, a stored `"SNES (Venus)"` cannot be distinguished from a real choice **and does not need to be**: it is the old default or a click on a control that did nothing. `AppSettings.Load` upgrades that exact string to `AllConsoles` once. A genuinely chosen console survives, because the only value that gets rewritten is the legacy default.

`AllConsoles` is spelled once, in `AppSettings` (Galaxia persists it), and re-exported by `CoreCatalog.AllConsoles` so UI code has one source. A test pins the two together.

### 10.3a The upgrade was not once, and a chosen Super Nintendo was lost

*2026-09-21.* §10.3 says `AppSettings.Load` upgrades the legacy string "once". It did so on every load. The claim
that "a genuinely chosen console survives, because the only value that gets rewritten is the legacy default" was true
of every console but one: the legacy default is also the Super Nintendo's display name, so a player who filtered the
library to it found every console back at the next start. The argument of §10.3, that the stored string "cannot be
distinguished from a real choice and does not need to be", holds for a file written *before* the filter read the
value and stops holding the moment a build that honours the value has saved it.

The defect went unseen because reaching it took a deliberate pick of one entry in a dropdown and a restart. It was
found when the pad's left and right began stepping the filter (`EmuSen_Settings_Reference.md` §4.29) and a test
stepped onto that entry and read the file back.

**The mechanism.** `AppSettings.SelectedCoreUpgraded`, false by default and so false for any file that lacks it.
`Upgraded` rewrites the legacy string only while it is false, and sets it; the next save carries it. A file from
before the filter is upgraded as before. If it is never saved again the upgrade simply repeats, which is harmless,
since nothing was chosen. A file saved since is believed.

A version number for the whole file was considered and not used: there is one migration, and a number invites the
reading that the file has a schema history it does not have. If a second migration arrives, that is the time.

**Coverage.** `ConsoleContextTests.The_super_nintendo_chosen_by_a_build_that_honours_it_is_kept` fails on the
unmodified code, reading "All consoles" where "SNES (Venus)" was saved, and passes with the flag.
`The_legacy_default_is_upgraded_rather_than_honoured` is unchanged and still passes, which is the other half: a file
without the flag is still upgraded.

**What it does not cover.** A player who chose the Super Nintendo under an earlier build and lost it is not given it
back; the file no longer says so. And a file hand-edited to carry the legacy string *and* the flag is believed, which
is the intent.

### 10.4 Search, not just filtering

3,537 titles under one console is not scrollable, so the library has a title search box next to the console combo — the same conclusion `CheatDatabaseWindow` already reached for its own game list ("a system holds thousands, so the filter is not optional in practice"). The disk walk happens once per console change; typing only re-filters what was already found.

### 10.5 What this is still not

- **`RomDirectory` is still one path.** Recursion makes that workable, but there is no multi-root library.
- **No metadata.** Titles are filenames. No box art, no dedupe of `(U)`/`(J)`/`[hM02]` variants, no "1,432 of these are the same game" grouping — which a 3,537-entry set very much invites.
- **The console filter does not gate loading.** Opening a `.nes` while SNES is selected still works and still runs Moon; the extension decides the core (§2). The filter is a view, not a mode.

---

## 11. The bus hooks are shared, not per core

`IFrameObserver` and `IRomReadPatcher` began life in `Cores/Nintendo/Venus - SNES/Memory/`, which was right while one core existed. Neither says anything about a console — one is "a frame ended", the other is "a cartridge read happened, do you want to change it" — so when the NES needed the second one they moved to `EmuSen/Cores/CoreHooks.cs` rather than being copied.

Nothing else had to change. Both interfaces were already in namespaces nested under `EmuSen.Cores`, so every existing `EmuSen.Cores.Nintendo.Venus.*` file resolves the new location outward with no `using` edit at all.

`CheatRomPatcher` (`EmuSen/Cores/`) is the shared wire between the second hook and `CheatRegistry`. Both ends were already core-agnostic; a core that owns its own registry can install it directly and get Game Genie-style patches with no debug layer attached — see `EmuSen_Cheats.md` §4 for why Venus does not do this yet.

## 12. Naming a console before its core exists

Two questions look the same and are not: *which console is this ROM* and *which console is loaded*. The second has an obvious answer (`ICore.CoreName`); the first has to be answered from the path alone, and getting them mixed up is what `CoreCatalog.ConsoleForRom` exists to stop.

**The bug it was added for.** `MainWindow.LoadRom` opened its log directory with `StartLogging(_session.CoreName)`, deliberately *before* `_session.LoadRom(path)` so the cartridge's own load-time output would be captured. But `EmulatorSession.CoreName` is `_core?.CoreName ?? "SNES"`, and `_core` is only assigned inside `LoadRom` — so the call always read the fallback. **Every Mistress session logged into `<root>/SNES/`, whatever console actually ran.** It was invisible because the fallback is a real console name: the path looked right, the directory existed, and the only symptom was a `Logs/` tree with no `NES` in it however many NES games you played.

`CoreCatalog.ConsoleForRom(path)` answers from the extension instead, via the same `ByExtension` lookup `CoreFactory.Create` dispatches on. It returns `CoreDescriptor.Console`, which is pinned equal to what that core's `ICore.CoreName` reports, so the directory a session logs into is the one it would have chosen after loading — just decided early enough to be right.

`EmulatorSession.CoreName` keeps its fallback, because callers after a load are entitled to a non-null string; its comment now says the fallback is only meaningful post-load rather than claiming there is only one core.

**Two names, one console.** A `CoreDescriptor` carries both `Console` (`"SNES"`) and `DisplayName` (`"SNES (Venus)"`), and different call sites hold different ones — `AppSettings.SelectedCore` is a display name, `ICore.CoreName` is a console. `CoreFactory.CheatCodecsFor` took a display name and fell back to the SNES pair for anything it did not recognise, so passing the *console* name got working-looking SNES codecs for every console. That is the same shape of bug as the log directory: a wrong answer that reads as a right one. It resolves through `CoreCatalog.ByAnyName` now, which accepts either. The fallback itself stays — a window opened with no console chosen still needs some pair — but it is no longer reachable by naming a console correctly in the wrong vocabulary.

## 13. Settings a console offers its frontend

*2026-09-20.* `ICoreSettings` (`EmuSen/Cores/CoreCapabilities.cs`) is the capability a core implements when it has
settings a frontend should offer per console: a list of `CoreSetting` — a key, a label, a hint, a kind (a switch, a
count within a range, or a choice among names) and a default — and `Get`/`Set` by key, every value in its text form.
The catalogue holds the same list statically per console (`CoreCatalog.SettingsFor`), beside the buttons and axes,
so a frontend can build the console's page with no core loaded; the core answers the same list when it is.

Three decisions worth stating. **The default is the frontend's, not the core's constructed state.** Mars is built
with its display processor on the calling thread, because that is what the harness and the tests want, and the
frontend used to turn the threads on by hand after `LoadRom`; the setting's default is what the frontend wants, so
applying every setting's default reproduces what the frontend did, and a config file that names nothing changes
nothing. **Values are text.** A frontend that stores them needs no type per key, a hand-edited file needs no schema,
and the core parses what it is given and refuses what it cannot — a count outside its range is clamped rather than
refused, since a range is advice about what is useful, not about what is safe. **Set runs on the emulation thread,
between frames.** Mars's setters join threads; a frontend calls them where its other requests to the core run.

Only Mars offers any today (`Mars_Core.md` §10). A console with none gets an empty list and a page that says so.

## 14. A picture that has not changed (2026-09-21)

`IFrameSerial` is a core's promise about its own frame: `FrameSerial` changes whenever `GetFrameBufferRgba` may return
a different picture, and **an unchanged value means unchanged pixels**. The promise runs one way only. A core may
advance the serial without the picture changing, which costs a frontend a needless copy and nothing else; it may
never keep the serial while the picture changes, which would leave a stale frame on the screen. A core that cannot
tell does not implement it, `EmulatorSession.FrameSerial` is null for it, and a frontend reads null as "new every
frame", which is what it did before.

Mars implements it because it knows exactly. Its frame is replaced in three places, the reset to a blank frame, the
immediate composition, and the swap when a deferred walk completes (`Mars_Video.md` §2.7), and the serial advances
in those three places and nowhere else. A scan skipped as a repeat (`Mars_Video.md` §2.8) replaces nothing, so the
many N64 games that draw every second or third field keep one serial across the fields they did not draw. In
immediate presentation every frame is composed, so the serial moves every frame; the promise holds and the saving is
lost, which is the right way round.

Mistress offers the frame control a picture only when the serial moved (`EmuSen_Serenity.md` §2.6).
`The_frame_serial_moves_when_the_picture_does_and_only_then` holds the promise frame by frame in both presentation
modes, and holds that a still picture keeps its serial under deferred presentation and a moving one moves it every
frame; one mutant advancing the serial every frame and one not advancing it at the swap both fail it.

## 15. Rows a core would otherwise repeat (2026-09-21)

Mars repeats every row of a progressive field so that a frontend drawing the buffer at its own aspect shows the
picture the right way up (`Mars_Core.md` §2). At four that makes a frame of 2,560 by 2,304 whose every other row is a
copy of the one above, and the frontend copies and uploads all of it (`EmuSen_Serenity.md` §2.5). `IRepeatedRows`
lets a frontend that can stretch take the rows once: `RepeatRows` is true by default and repeats them in the frame as
before, false sends each once, and `RowRepeat` says how many times the frame **on show** has each row shown, so it
travels with the frame through the deferred swap exactly as its height does.

**The default is the old behaviour, and only Mistress changes it.** Every other consumer of `GetFrameBufferRgba`
(the golden probe, the tests, Hotaru, screenshots, recordings) reads a buffer as square pixels, and keeps getting one
that is right that way. This is a channel for a **row** repeat, not a pixel aspect: the PAL picture is still drawn
20% too tall and Venus's at 8:7 (`Mars_Core.md` §2); a pixel aspect is the general form and would subsume this, and
was not taken on here.

`Rows_sent_once_are_the_repeated_frames_own_rows` holds that the rows sent once are the even rows of the repeated
frame, with half its height and a repeat of two, deferred and immediate, and that an interlaced frame, which has
nothing to repeat, is unchanged with a repeat of one. Dropping the repeat at the deferred swap fails it.

## 16. A picture a core lends (2026-09-23)

`GetFrameBufferRgba` returns an array, and until now the contract said nothing about who owns it. Two answers were in
use. Venus, Moon, Mercury and the C# Mars hand out their own live buffer, which the next frame rewrites, so a caller
that keeps it past the next `RunFrame` reads a changing picture; Mistress survives this because the frame control
copies an offer out on the render thread, usually before the next frame (`EmuSen_Serenity.md` §2.6), and the risk
of a torn picture is accepted (`Mars_Native.md` §5.5). MarsRT handed out a new array every call, which is safe and
cost an allocation the size of the picture per frame, 19 MB at four.

`IFrameBufferPool` is a third answer, opted into by a core and by a frontend together. A core that implements it
still hands every caller an array that is **the caller's alone**: nothing the core does writes it or lends it to
anyone else. A caller that is finished with it may call `ReturnFrameBuffer(array)` once, from any thread, and the core
may then lend it again. A caller that never returns anything, which is every caller but Mistress, sees exactly the old
MarsRT behaviour. MarsRT is the one implementer.

`EmuSen/Cores/FrameBufferLending.cs` is the bookkeeping, so that a second core could lend the same way. It keeps two
short arrays under one lock: at most **four** returned arrays waiting to be lent again, and a record of the last
**eight** arrays lent. `Lend(length)` takes a waiting array of that length, or makes an uninitialised one outside the
lock; a change of length throws the waiting ones away. `Return(array)` takes an array back only if it is in the record,
removing it from the record, and only if its length is the current one and there is room; everything else is counted
as dropped and left to the collector. So a second return of the same lend, an array the core never lent, one lent at a
size since changed, and one returned to a core closed by `Dispose` all do nothing
(`Returning_an_array_twice_does_not_lend_it_twice`, `A_foreign_array_or_one_lent_at_another_size_is_dropped`,
`A_closed_lending_takes_nothing_back`). A caller that keeps more than eight arrays out has the oldest forgotten, which
makes it the caller's for good rather than a hazard (`The_lending_stays_bounded_whatever_is_returned`). `ArrayPool<byte>.Shared` was
not used and not measured: it rounds a request up to a power of two, so a 19 MB picture would rent 32 MB, it keeps
arrays per processor where this needs a handful in all, and its arrays go to any renter in the process, so the record
that makes a foreign return harmless here would not exist.

**What it cannot tell.** An array is its own identity, so a caller that returns an array, receives the same array
again from a later lend, and then returns it a second time on the strength of the first lend, returns what someone
else now holds. No bookkeeping by array can see that; a lease object could, at an allocation a frame, and was not
built. The one caller, Mistress, returns each array once by construction (`EmuSen_Serenity.md` §2.8).

