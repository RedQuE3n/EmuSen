# Running more than one core

*This revision: initial. Written when Moon (NES) became the second real core and the frontends had to stop assuming there was only one.*

Companion docs: `EmuSen_Input.md` (the pad contract, which came out of the same pass), `EmuSen_Crystal_Scheduler.md` (the core-agnostic timing mechanism), `EmuSen_Project_Overview_v2.md` (the directory/namespace map), `EmuSen_Launcher_Multicore_Gameplan.md` (the separate, later-stage launcher project this is *not*).

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
| `IFrameProfiler` | `LastFramePhases` — ordered `(Name, Milliseconds)` | Venus |
| `ICoprocessorHalt` | `IsHaltedOnCoprocessor`, `HaltedProcessorName` | Venus |
| `ICoprocessorLoad` | `CoprocessorClocks` — executed/offered against a per-frame budget | Venus |
| `ITraceFlushable` | `FlushVerboseTrace()` | Venus |

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

---

## 7. What is still core-specific, on purpose

- **`Bundle`'s type switch** (§2) — one file, by design.
- **Hotaru's three debug hotkeys** (§5).
- **`SnesDebugTarget`'s constructor shape** — it takes `(Cpu, MemoryBus, Renderer)`, which only a Venus caller can supply. That is why `Bundle` exists: it is the one caller.
- **Pharaoh's two power-on WRAM watches** — an SNES-era investigation leftover, now guarded on a space of that name actually existing, so an NES run does not register watches that can never fire.

## 8. What is not done

- **No core swaps consoles mid-session.** `core <name> <path>` reloads through the factory, so loading a `.nes` after a `.smc` does construct a `MoonCore` — but the frontends rebuild a great deal around that and only Mistress' path is well covered by tests.
- **`AppSettings.SelectedCore` still drives nothing.** It persists a display name and no code reads it to choose a core; the ROM's extension decides. The combo is now populated from the catalog rather than a literal, which is the only part of it that improved.
- **Moon has no `ICpuTraceSwitch` and no `IFrameProfiler`.** Both are real gaps rather than deliberate omissions. It does now have cheat codecs — see `EmuSen_Cheats.md` and `Moon_Cheats.md`.

### A bug this pass found

`EmulatorSession.ScreenHeight` was a `const int = 224` while `ScreenWidth` already asked the core. Mistress presented every frame as `(session.ScreenWidth, EmulatorSession.ScreenHeight)`, so Moon's 256×240 framebuffer would have been submitted as 256×224 — a wrong-sized blit from a buffer 16 rows longer than claimed. It is now an instance property that asks the core, like its neighbour. Worth noting as the class of bug this whole pass exists to prevent: a constant that was true of the only core there had ever been.

---

## 9. The leaf layers

Four assemblies sit below everything else and are **not allowed to know a core exists**. Three of them are true leaves — zero project references of any kind:

| Project | Project references | Job |
|---|---|---|
| `EmuSen.Galaxia` | **none** | config persistence, and `PadButton` (`EmuSen_Input.md` §3) |
| `EmuSen.Crystal` | **none** | the core-agnostic scheduler (`EmuSen_Crystal_Scheduler.md`) |
| `EmuSen.Endymion` | **none** | audio output sink (`EmuSen_Audio_Sync.md` §7) |
| `EmuSen.Serenity` | `Galaxia` | video presentation |
| `EmuSen.Nehellania` | `Galaxia` | gamepad polling and the rebindable pad map |

Serenity is the model the others were held to. Its contract is `UpdateFrame(byte[] rgba, int width, int height)` — payload plus the metadata to read it, and nothing else. Endymion's `Submit(short[] pcm, int sampleRate)` is the same contract for audio.

The rule that keeps them leaves: **a device layer is handed what it needs and reaches for nothing.** Not a core, not a session wrapper, not a global settings static. Every knob is a parameter. That is why `AudioPlayer` takes its sample rate, latency target and deviation band as constructor arguments instead of reading `AudioSettings`, which lives in `EmuSen` because the Venus DSP needs it.

`EmuSen.WiseMan/Common/LeafAssemblyTests.cs` asserts each of these reference sets against the actual compiled metadata. What it catches is a leaf *using* a type from the core assembly; what it does not catch is an unused `ProjectReference` sitting in a `.csproj`, because the compiler drops references nothing consumes.

**What is not a leaf, and should not be:** `EmuSen.Hotaru`, `EmuSen.Mistress` and `EmuSen.Pharaoh` are frontends — their whole job is to hold a core and drive it, so they reference everything. `EmuSen.DianaOS` is agnostic about which cores exist but is not a leaf for other reasons.

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
