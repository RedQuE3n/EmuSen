# EmuSen Config Reference — `EmuSen.Galaxia`

Every config file this program keeps on disk, where it lives, and how it is read and written. `EmuSen.Galaxia` is the one project that answers those questions; everything else asks it.

This document is about *persistence*. What each individual setting actually does is elsewhere: `EmuSen_Settings_Reference.md` §1 for debug flags, §2 for audio, §3 for graphics, §4 for the frontend's input bindings.

---

## 1. Where config lives

### 1.1 Galaxia is a leaf, and root discovery lives in it

`EmuSen.Galaxia` has no `ProjectReference` and no `PackageReference`. That is a hard constraint, not a preference.

`EmuSen.DianaSh` needs Galaxia, because `CheatRegistry` saves and loads cheat files and `cheat save`/`cheat load` are DianaSh commands. `EmuSen` already references `EmuSen.DianaSh` (`DebugSettings.MasterLoggingEnabled` forwards to `DianaShLogging`). So anything Galaxia depended on upward would close a cycle.

That constraint is what makes the project genuinely agnostic rather than merely shared. Nothing in it can name `Avalonia.Input.Key`, `SDL.GamepadButton` or `SnesButton`, so:

- the **models** in Galaxia are plain data (`AppSettings`, `AudioConfig`, `GraphicsConfig`, `CheatFile`);
- the typed maps that *do* use foreign enums (`ControllerKeyMap`, `GamepadBindingMap`, `HotkeyBindingMap`) stay in their own projects and use Galaxia only for persistence.

The same constraint moved root discovery. `DianaShSandbox` used to own the walk-up that finds the project root; it now delegates to `ConfigRoot`, which holds the walk and both markers. `DianaShSandbox.RootDirectory`, `ComputeRootFor`, `PublishedRootDirName` and `RootMarkerFileName` all still exist and still mean the same thing — they forward. Nothing about how the root is found changed, only which project holds the code. `ConfigStoreTests` pins the two to the same answer.

**That move has since been repeated for saved data.** The `Usr/Home` directories — saves, save states, firmware, cheats, logs — left `DianaShSandbox` for Galaxia's own `DataStore` on the same reasoning, and the sandbox forwards those six properties too. This document remains the reference for the *config* half (the files under `/etc/EmuSen`, and how each is read and written); **`EmuSen_Galaxia.md` covers the project as a whole**, including the data half, `AtomicFile`'s durability contract and the save/state naming rules.

### 1.2 The paths

Config lives inside DianaSh's own sandbox tree, at `<root>/etc/EmuSen` — reachable from the shell as `/etc/EmuSen`, which is the point. See `man hier`.

```
/etc/EmuSen/
  appsettings.json        general app preferences        §3.1
  keybindings.json        keyboard -> console button      §3.6
  gamepadbindings.json    pad button -> console button    §3.6
  hotkeybindings.json     keyboard -> emulator action
  audio.json              §3.2
  graphics.json           §3.3
  cheats/
    <name>.json           one saved cheat set per file   §3.4
    <console>/<game>.json a game's own list, kept by a frontend   §3.4b
```

`ConfigStore.For(fileName)` resolves the flat files. `ConfigStore.For(category, fileName)` resolves the one-subdirectory-per-category kind, which today means cheats — config there can be many of, named by the player, rather than one well-known file.

Because this is inside the sandbox, `cat`, `nano`, `find` and `>` all reach it. In a *published* tree it is also inside the walled garden along with the binaries, which is why `rm` and `mv` are suspended there — see `man hier`. `DianaShSandbox.EnsureSkeleton` creates the directory at startup so it is always there to write into.

### 1.3 Two separate overrides, on purpose

`ConfigStore.OverrideDirectory` redirects config and **only** config. `ConfigRoot` is not redirectable and does not move with it.

This matters because the sandbox root is also where saves, save states, logs and firmware live. If redirecting config also moved the root, any test that so much as read a binding would relocate every other DianaSh path underneath the rest of the suite. `ConfigStoreTests.Override_does_not_move_the_sandbox_root` pins that separation.

`ConfigFile<T>.Path` is recomputed on every access rather than cached in the constructor, because the binding maps hold their `ConfigFile` in a `static readonly` field that outlives any individual test's override.

### 1.4 Migration out of the old location

Before Galaxia, these files lived outside the sandbox in the OS's per-user config directory (`~/.config/EmuSen` on Linux, `%AppData%\EmuSen` on Windows). Existing installs still have them there.

The first time a given file is read and no copy exists in `/etc/EmuSen`, `ConfigFile<T>.Load` **copies** it across and reads the copy. It copies rather than moves: nothing is destroyed if you roll back to an older build, and a config file is small enough that a stale duplicate costs nothing. Once the new file exists the legacy one is never consulted again, so later edits can't be silently reverted by whatever `%AppData%` still holds.

Migration applies only to the flat files. Category files (cheats) never lived in the old location, so there is nothing to migrate.

`ConfigStore.OverrideLegacyDirectory` exists so migration is testable. The rule enforced in `LegacyPathFor` is that a test which redirected the config directory but said nothing about the legacy one gets **no** legacy path at all — otherwise every test in the suite would be reading the developer's real `~/.config/EmuSen`.

---

## 2. How a config file is read and written

`ConfigFile<T>` is the whole mechanism. `T` is whatever actually gets serialized: a settings object for most, a plain `Dictionary<,>` for the three binding maps, which persist their dictionary directly and always have.

Before this project each of `ControllerKeyMap`, `GamepadBindingMap`, `HotkeyBindingMap` and `AppSettings` carried its own private `ConfigPath` property plus a `Save()`/`Load()` pair — four copies of the same twenty lines, differing only in the file name and the type argument. They are now four one-line delegations.

### 2.1 JSON options

One `JsonSerializerOptions` for every file (`ConfigJson.Options`): indented output, plus `ReadCommentHandling = Skip`, `AllowTrailingCommas`, `PropertyNameCaseInsensitive`, and an enum converter that writes names (§6.2).

The read options follow directly from putting config where the shell can reach it. A file meant to be opened and edited by hand should tolerate what a person actually types — a `//` note explaining why a value was changed, a trailing comma after the last entry, a lowercased property name.

**Enums are written as names.** This is what the format change was actually for. The default converter writes enum *values*, which made the binding files effectively unreadable:

```json
{ "Up": 11, "B": 0, "Start": 6 }        // before
{ "Up": "DPadUp", "B": "South", "Start": "Start" }   // now
```

Note the dictionary *keys* (`"Up"`, `"B"`) were always names — `System.Text.Json` writes enum keys that way already. Only the values were numeric.

Three properties of this, all measured rather than assumed (`ConfigEnumFormatTests`):

- **Numbers still read.** The converter accepts both forms, so every binding file written by an earlier build loads unchanged and is rewritten as names on the next save. A half-edited file mixing both forms also loads.
- **Names are matched case-insensitively**, so `"dpadup"` works — consistent with `PropertyNameCaseInsensitive`.
- **A misspelled name fails the whole file**, not one entry, because dictionary deserialization is all-or-nothing. Under §2.2 that means falling back to defaults. This is a real cost of the change: with numbers, `"Up": 999` would have parsed into a meaningless-but-harmless enum value, whereas `"Up": "DPadUpp"` resets every binding in the file. It is no longer *silent* — §6.2 names the value and suggests the correction — but the whole-file scope is unchanged, and a test pins it so it stays a known behaviour.

There is one further trade-off worth stating plainly, because it partly gives up something an earlier piece of work relied on. `gamepadbindings.json` survived the SDL2 → SDL3 migration *because* it stored numbers: SDL3 kept SDL2's numbering, so the file kept meaning the same thing even though `A`/`B`/`X`/`Y` were renamed to `South`/`East`/`West`/`North` (see `EmuSen_Settings_Reference.md` §4.6). Storing names inverts which kind of change is survivable — a future renumbering no longer matters, a future *rename* does. That is the right way round for a file meant to be read by a person: `South` says which physical button it is, and `0` never did.

### 2.1.1 What XML would and wouldn't have bought

XML was considered for the same hand-editing goal and rejected, on two measured grounds rather than taste:

- `XmlSerializer` throws `NotSupportedException` on any `IDictionary`, and three of the six config files are dictionaries. `DataContractSerializer` handles them but emits type-hashed element names (`<KeyValueOfSnesButtonKeyaI91Yjnm>`), which is worse to hand-edit than what it replaced. A usable XML shape would have needed a wrapper type per binding file plus a JSON→XML migration on top of the §1.4 one.
- **Comments do not survive a save in XML either.** `XmlSerializer` deserializes into an object and reserializes from scratch, exactly as `System.Text.Json` does; a `<!-- -->` in the file is gone after the next write. Any format keeps comments only if saving merges into the existing document rather than regenerating it.

Since hand-editing already worked, and the actual complaint was enum opacity, the converter above addressed it for one line and no migration.

### 2.2 Failure is not an exception

`Load()` returns `null` for missing, unreadable, or corrupt. `Save()` returns `false` if the write failed. Neither throws.

This preserves what all four original classes did, and it is deliberate rather than lazy: a settings change that can't reach the disk shouldn't take the program down with it — it just won't survive a restart. A corrupt file falls back to defaults instead of refusing to launch. `Load(Func<T>)` is the convenience form for callers that always have a default.

The one thing failure must never do is destroy what is already there. A file that fails to parse is left exactly as it is on disk (`SettingsPersistenceTests.An_unreadable_file_is_left_on_disk_untouched`) — the hand edit is still there to be fixed, rather than being overwritten with defaults by the next save.

### 2.3 Writes are atomic

`Save()` writes the whole file to `<path>.tmp` and then renames it over the target. A rename within a directory is atomic on both NTFS and ext4, so an interrupted save leaves the previous config intact rather than a truncated one.

None of the four original classes did this — they wrote in place with `File.WriteAllText`. Rebinding a key saves immediately, so a crash or a power loss in that window could leave a half-written `keybindings.json`, which then loads as corrupt and silently resets every binding to defaults. Not observed in practice, but it costs one rename to remove.

---

## 3. The config files

### 3.1 `appsettings.json` — `AppSettings`

Moved here from `EmuSen.Mistress/Settings/`; it holds no frontend types, so there was no reason for it to be frontend-specific. Log/ROM/state directories, the selected core, and the three input preferences the settings UI writes (`MirrorPlayer1ToPlayer2`, `AnalogStickAsDpad`, `StickDeadzone` — see `EmuSen_Settings_Reference.md` §4.4).

`MirrorPlayer1ToPlayer2` is off by default. Forcing it on would break any real two-controller game, by feeding Controller 2 the same input as Controller 1 even when a second pad is plugged in. It exists for games that read Controller 2 where Controller 1 is expected, for reasons of being a classic game inside a compilation (Super Mario All-Stars' three Super Mario Bros. games are the known case, `EmuSen_Games_Tested.md`); it is toggled from the input settings window, and is the workaround players of the hardware and of other emulators use, not a redesign of input.

`LibrarySearch` and `CheatSearch` are the search halves of the two filter bars, and they live here rather than in a file of their own because `SelectedCore` is already the *facet* half of the library's. Splitting one filter bar's state across two files to keep a config file tidy is the wrong trade. Both are written on window close rather than on change, and both default to the empty string, so an older `appsettings.json` loads unchanged. See `EmuSen_Settings_Reference.md` §4.23 for why restoring them was not safe before LunaP 0.10.0.

`SelectedCoreUpgraded` records that the one-time rewrite of the legacy `SelectedCore` default has been done, so that the same string chosen afterwards is kept. It is absent from older files, which is the point. See `EmuSen_Multicore.md` §10.3a.

`BigScreen` (default `false`) starts Mistress full screen with no menu bar and larger library text, for a handheld or a television. It is read once at start; `--bigscreen` and a Steam Deck session's `SteamDeck=1` ask for the same thing without it. See `EmuSen_Settings_Reference.md` §4.29.

The class name and the file name are unchanged, so existing files load as-is.

### 3.2 `audio.json` — `AudioConfig`

`EmuSen.Audio.AudioSettings` is a static hub of mutable fields, which `System.Text.Json` cannot serialize. `AudioConfig` is its on-disk mirror; `AudioSettings.LoadFromDisk()`/`SaveToDisk()` own the mapping in both directions.

`LoadFromDisk` **clamps** every value it applies. That is not defensive habit — it follows from §2.1. This file exists to be hand-edited, and `AudioSettings.OutputTargetFrames` divides by `SampleRate`, so a typed-in `0` would be a crash rather than a bad setting.

On first run, when no file exists, `LoadFromDisk` seeds one from the current defaults so there is something to open. A file that already exists is never rewritten behind the player's back.

### 3.3 `graphics.json` — `GraphicsConfig`

The same arrangement for `EmuSen.Graphics.GraphicsSettings`: window size, target FPS, VSync, resizability, bilinear filtering. Clamped and seeded exactly as audio is.

`WindowTitle` is deliberately **not** persisted. It is branding rather than a preference, and a corrupt config blanking the application's title is a worse outcome than not being able to change it from a file.

*2026-09-20:* the file also holds **`Consoles`**, each console's own settings by key in their text form, as its core
reads and writes them (`EmuSen_Multicore.md` §13): `"Consoles": { "N64": { "RdpWorkers": "4", "ThreadedRdp": "true" } }`.
A key that is absent means the setting's default; a console that is absent means all of them. The values are text
on purpose, so the file needs no schema per console and a hand edit that the core refuses falls back to the default
rather than failing the load. `Value`, `SetValue` and `Forget` are the accessors; Mistress's graphics window writes
them and saves at once.

*2026-09-24:* two more maps, written by Mistress's Shaders window (`EmuSen_Settings_Reference.md` §4.48.3).
**`ShaderParameters`** is each console's values for each shader's parameters, the shader keyed exactly as that
console's `ScreenFilter` value would be (a built-in's name, or `slang:` and a preset's path in the pack), the values
text: `"ShaderParameters": { "SNES": { "CRT (Lottes)": { "maskDark": "0.3" } } }`. Only values that differ from the
default are held; resetting one removes it, and a shader or console left empty goes with it. **`RecentShaders`** is
each console's last five shaders used, newest first: `"RecentShaders": { "SNES": [ "slang:crt/crt-royale.slangp" ] }`.
The accessors are `ParametersFor`, `SetParameter`, `ForgetParameter`, `RecentFor` and `NoteRecent`; `Forget(console)`
leaves both maps alone.

*2026-10-01:* **`SyncToDisplay`**, a boolean, on when absent (`EmuSen_Settings_Reference.md` §4.87.7). Preferences
writes it into the same object Mistress's graphics window saves, so neither overwrites the other's keys.

*2026-10-15:* a console whose engine lists a removable battery file holds the runtime's **card mode** under its key in
`Consoles`, beside the core's own keys and never passed to the core: **`Cards`**, `game` (the default, each game's own
cards), `shared` or `named`, and under `named` **`Card1`** and **`Card2`**, the names of the cards in the two slots,
`Card 1` and `Card 2` when absent: `"Consoles": { "PlayStation": { "Cards": "named", "Card1": "Blue" } }`. An unknown
mode reads as `game`, and a name that is not one file name (a separator, a leading dot) as the slot's default, so a hand
edit can never point a card outside the `Cards` folder (`MemoryCards.Read`; `Diamond_Cards.md` §9;
`EmuSen_Settings_Reference.md` §4.114).

*2026-10-15, later:* the same keys for every console's save media (`EmuSen_SaveMedia.md` §3.3): a console's first
removable medium keeps `Cards` and `Card<n>`, its own memory takes **`ConsoleMemory`** and `ConsoleMemory<n>`, any
further medium **`Media.<id>`** and `Media.<id>.<n>`. Absent, a mode is its medium's default, `shared` for the
PlayStation's card and `game` for a medium no format module reads. **`<mode key>Offered`** (`CardsOffered`) is `true`
once Mistress has offered the import of the games' own cards (§4.115 there).

### 3.4 `cheats/<name>.json` — `CheatFile`

New capability, not a migration: `CheatRegistry` previously had no persistence at all, so every cheat was lost on exit.

`CheatRegistry.ToCheatFile()` and `LoadFrom(CheatFile)` map between the registry and the file; `cheat save`, `cheat load` and `cheat files` are the shell surface (see `man cheat`).

A cheat is one description, one enable flag, and a **list of writes** — real cheats are rarely one address, and the ones that aren't have to arm and disarm together (see `man cheat` for the write model). Addresses and values are stored as **hex text**, not JSON numbers:

```json
{
  "cheats": [
    {
      "kind": "RamPoke",
      "description": "99 lives",
      "enabled": true,
      "writes": [
        {
          "space": "WRAM",
          "address": "7E0019",
          "value": "09",
          "width": 1,
          "type": "Set",
          "bitPosition": null,
          "bigEndian": false,
          "repeatCount": 1,
          "repeatAddAddress": "0",
          "repeatAddValue": "0"
        }
      ]
    }
  ]
}
```

**Files written before writes were a list still load.** A pre-multi-write entry carries `space`/`address`/`value` flat on the cheat itself instead of a `writes` array; `CheatFileEntry.EffectiveWrites` synthesizes the single 1-byte `Set` write those describe, so nothing saved earlier is stranded. Saving always writes the new shape and leaves the old fields null, so re-saving quietly migrates a file. The mapping between this carrier and `CheatWrite` lives in `CheatRegistry`, not on the carrier, because `EmuSen.Galaxia` cannot reference `EmuSen.DianaSh` — the dependency runs the other way.

`"address": 8257561` would be technically equivalent and useless to a person reading it with `cat`. Since the entire reason for putting config in `/etc` was to make it reachable from the shell, the on-disk shape follows the way SNES addresses are actually written. Reading tolerates `0x` and `$` prefixes and surrounding whitespace, because those are what people paste in.

Three behaviours worth stating:

- **A malformed entry is skipped, not fatal.** `LoadFrom` returns `(loaded, skipped)` and `cheat load` reports both. One typo in a hand-written file costs that one line, not the file.
- **`load` merges, it does not replace.** It adds to whatever is already loaded; `cheat clear` first to replace. This is the honest reading of "load", and the alternative silently discards cheats added since the last save.
- **Nothing loads automatically.** There is no per-ROM auto-apply. A cheat set changes the run only when asked for, so a file saved months ago can't quietly alter a later session — which matters when the emulator is being used to investigate whether a game behaves correctly.

`CheatFile.IsValidName` rejects a name rather than sanitizing it. The name arrives from a shell argument, so it must not be able to walk out of the cheats directory; and silently rewriting what was typed would save to a file nobody named.

### 3.4a `CheatDatabaseDirectory` — using a cheat tree you already have

`AppSettings.CheatDatabaseDirectory` points at a directory tree of `.cht` files, one per game in per-system folders — the exact shape RetroArch stores its cheats in. Point it at an existing RetroArch cheats folder and it is indexed in place: no copying, no conversion, no import step. Unset, it falls back to `DianaShSandbox.CheatDatabaseDirectory` (`home/Cheats`); since 2026-10-10 both `cheat db` and the Cheat Database window ask `SettingFolders.Cheats` for that answer, and §7 records what a relative or `~` value does (it was the handheld's `genesis`).

This is deliberately a *directory setting* rather than a bundled asset. **EmuSen ships no cheat data and redistributes none.** The libretro cheat database is licensed CC BY-SA 4.0, but its own README states the codes were "collected from any available source on the web" — so the grant is only as good as libretro's rights in an aggregated corpus, and the EU sui generis database right applies independently of copyright either way. Indexing a folder the player already has, or downloading one to their machine at their explicit request (`cheat db update`, `Settings > Cheat Database...`), keeps that question off this project entirely. Attribution is shown at the point of download, since it is a licence condition rather than a footnote.

`CheatDatabase` and `CheatDatabaseInstaller` both live in `EmuSen.DianaSh` and are core-agnostic; the installer extracts only `.cht` entries and refuses any archive entry that resolves outside the target directory.

### 3.4b `cheats/<console>/<game>.json` — `GameCheats`

*Added 2026-10-10.* The list a frontend restores when a game starts and keeps when it is left (`EmuSen_Settings_Reference.md` §4.15, §4.15b). `GameCheats.For(console, romPath)` names it: the console as the catalogue gives it (`CoreCatalog.ConsoleForRom`, the same name the library and `Saves/<console>/` use), and the ROM's file name without its extension, both sanitised rather than refused because neither was typed. A console the catalogue does not know falls back to the flat `cheats/<game>.json`.

**Why a folder per console.** Until this date the list was the flat `cheats/<game>.json`, named by the file name alone, so two consoles' games of one name shared a file: a list saved for `Gamma.smc` was restored into `Gamma.nes`, writing SNES addresses into NES memory every frame. This was shown by a test on the code before the change (*measured on 2026-10-10*, an SNES list saved, the NES game started, the list not empty) and is the case `GameCheatListTests.Two_consoles_games_of_one_name_keep_separate_lists` keeps closed. The battery saves made the same move on 2026-09-28 for the same reason (`EmuSen_Galaxia.md` §5).

**Earlier lists are read, never moved.** `Load` takes the console's file when it exists and otherwise the flat file of an earlier build; `Save` always writes the console's file, and the flat file is left exactly as it was. The old ambiguity therefore survives for a flat file until one of the two games saves, and no further: a flat file cannot be assigned to one console by anything in it.

**`SaveIfChanged`**, the write on leaving a game, compares the list with what `Load` returns, both serialised by `ConfigJson.Options`, and writes only on a difference. An empty list where nothing is saved writes nothing, so starting and leaving a game leaves no file behind; an empty list over a saved one is written, because removing every cheat is a change the player meant. It returns null for "nothing to write", which keeps a failed write distinguishable from an unchanged list.

**Scope.** The shell's typed names (`cheat save <name>`, `cheat files`) are unchanged and stay flat; `ListNames` lists only files directly in `cheats/`, so the console folders do not appear among them. `GameCheats` composes `ConfigFile` and `CheatFile`, which already reach the platform library, and its only rule of its own is the naming above. *Since 2026-10-10 that rule has its Rust half* (`cheats::game_list` in `Platform/galaxia`, exported as `emusen_galaxia_cheat_game_list`, interface 7): `GameCheats.For` asks the library while Galaxia runs on it and keeps the C# as `GameCheats.Managed`, held to the library by the parity class (`EmuSen_RustPlatform.md` §20). `Load`'s fallback and `SaveIfChanged` stay C#, being composition over `ConfigFile` and not rules of their own.

### 3.5 What is deliberately not persisted

`DebugSettings` has no config file. Its flags are trace toggles for a specific investigation, and persisting them means a run three weeks later starts spewing DMA traces because of something enabled once and forgotten — which is exactly the failure `MasterLoggingEnabled` was added to fix (see `EmuSen_Settings_Reference.md` §1). Debug flags default off every run, and `log` sets them for the session.

### 3.6 `keybindings.json` / `gamepadbindings.json` — one map per console

Both were flat `PadButton -> Key` (or `-> SDL.GamepadButton`) dictionaries, shared by every core. They are now nested one level, keyed by console name:

```json
{
  "NES":  { "A": "X", "B": "Z", "Start": "Enter" },
  "SNES": { "A": "X", "B": "Z", "Y": "A", "X": "S", "L": "Q", "R": "W" }
}
```

The key is `CoreDescriptor.Console`, the same string as `ICore.CoreName` (`EmuSen_Input.md` §5.1). `ControllerKeyBindings` and `GamepadBindings` own the file; the individual maps no longer have `Load`/`Save`.

**The old flat file still loads, and every console inherits it.** That is what it used to mean — one set of keys applied to whatever core was running — so migration is faithful rather than merely lossless. Each console gets its own *copy* of the dictionary, not a shared reference, or rebinding one console afterwards would silently move the others. The next save rewrites the file nested.

Telling the two shapes apart takes some care, because a wrong guess is worse than either. The load path first reads the file as `Dictionary<string, JsonElement>` purely to ask whether the values are objects (new) or scalars (old). `JsonElement` accepts *any* value, so that probe never fails on a bad enum — which matters, because the whole point of §6.2's converter is that `"DPadUpp"` gets named and corrected rather than swallowed. The probe decides the shape; the typed load that follows is what reads the values and reports. A bad enum is therefore still reported exactly once, and still falls back to defaults.

**Since 2026-09-18 the keyboard file's keys are `PadControl` names** (`EmuSen_Input.md` §7.2): every `PadButton` name
as before, plus `L2`, `R2`, `L3`, `R3` and the eight stick directions (`LeftStickUp` … `RightStickRight`). The gamepad
file's keys stay `PadButton` names and gain `L3` and `R3`. No existing file needed rewriting — each name it holds still
parses — and nothing about the shape changed. What a file written before that date cannot hold is the new controls, so
on load **each console's map gives a control it does not mention its default key, unless that map already uses the
key** (`EmuSen_Input.md` §7.4). A user's own choice of a key always wins over a new default.

---

## 4. Wiring

`AudioSettings.LoadFromDisk()` and `GraphicsSettings.LoadFromDisk()` are called from `Main` in both GUI frontends (`EmuSen.Mistress/Program.cs`, `EmuSen.Hotaru/Program.cs`), before any window exists — graphics config decides the window's size, so it has to be applied first.

`EmuSen.Pharaoh` does not load either. It is a headless CLI tool with no window and no audio device; window size and master volume mean nothing there.

No GUI or shell command edits `audio.json`/`graphics.json` yet — they are read at startup and hand-edited. The Preferences window still writes `appsettings.json` as before, and the input settings window still writes the three binding files.

---

## 5. Test coverage

`EmuSen.WiseMan/Galaxia/`:

- `ConfigFileTests` — round-trip, missing/corrupt/hand-edited input, the temp file not being left behind, category paths, `Path` following a moved override, and both migration directions (it happens once; it never runs again once the new file exists).
- `ConfigStoreTests` — the `/etc/EmuSen` shape, DianaSh and Galaxia agreeing on the root, and the config override *not* moving the sandbox root.
- `CheatFileTests` — both cheat kinds through a registry round-trip, hex-text output, the `0x`/`$`/whitespace input forms, a broken entry being skipped while the rest load, and the name-validation cases that could otherwise escape the directory.
- `SettingsPersistenceTests` — both hubs round-tripping, clamping of hand-edited zeroes, first-run seeding, and a corrupt file being left untouched on disk.
- `ConfigEnumFormatTests` — §2.1's enum-name format: a numeric file from an older build mapping to the same physical buttons, saves writing names, the numeric-to-name upgrade in one pass, half-edited mixed files, case-insensitive names, and the misspelled-name-resets-the-file cost.

- `SuggestionTests` — §6.1's engine: transposition scoring, the length-scaled tolerance, joint-best-only results, prefix tie-breaking, and the empty/absurd inputs.
- `SettingFoldersTests` and `SettingPathsOpenTests` — §7: a blank folder setting is unset at every reader, and the measured relative, `~` and file cases, the open ones skipped until the tester answers §7.5.
- `ConfigDiagnosticsTests` — §6.2: a misspelled enum value and a misspelled dictionary *key* both being named and corrected, the report carrying the file path, malformed JSON reported too, and a missing file reporting nothing.
- `SuggestionMessageTests` (`EmuSen.WiseMan/DianaSh/`) — the shell's unknown-command path and four of the five subcommand sites, plus the "Try a/b/c" list still naming everything.

`GamepadBindingMapTests` (`EmuSen.WiseMan/Input/`) still pins the on-disk *contract* for the pad map across the SDL2 → SDL3 rename — that is about the file's meaning rather than its persistence, and is unaffected by this move.

---

## 6. "Did you mean ...?"

A name that doesn't match anything used to produce a list and nothing else — `Unknown command 'chaet'. Type 'help' for a list.` The list is the right thing to keep; it just doesn't help when the answer is one keystroke away.

### 6.1 The engine (`Text/Suggestion.cs`)

`Suggestion.Hint(typed, candidates)` returns `" Did you mean 'cheat'?"` or `""`, so a call site appends it to an existing message with no null check:

```csharp
$"Unknown command '{cmd}'.{Suggestion.Hint(cmd, _commands.Keys)} Type 'help' for a list."
```

It lives in Galaxia because that is the lowest project both callers reach — the shell references it, and so does config. It is a pure string function with no dependencies, so it costs the leaf property in §1.1 nothing. If a Unix-style `spell` command is ever wanted, this is the core to build it on.

Four decisions in it are worth knowing, each pinned by `SuggestionTests`:

- **Optimal string alignment, not plain Levenshtein.** Typing `chaet` for `cheat` is one slip — a transposition. Plain Levenshtein scores it 2, the same as two unrelated edits, which puts it behind worse guesses.
- **Tolerance scales with length**: 1 edit up to 3 characters, 2 up to 6, 3 beyond. `rm` is one edit from `ls`, `cp` and `mv`, and offering any of them is worse than saying nothing.
- **Only the joint-best distance is offered.** `chaet` is 1 from `cheat` and 2 from `cat`; both are inside tolerance, but showing the second dilutes the first. Ties are still shown together (`lst` → `'last' or 'list'`), because there genuinely isn't a best answer there.
- **An exact match suggests nothing.** If the typed name *is* a candidate, the caller rejected it for some other reason, and a near-miss would only mislead.

Wired into the shell's unknown-command path and the five `Unknown '<sub>' subcommand` sites (`cheat`, `watch`, `bp`, `framelog`, `tmux`). At each of those the "Try a/b/c" list is now generated from the same array the suggestion searches, so the two can't drift apart — which they already had scope to, since the list was a hand-maintained string literal.

### 6.2 Config values, and the end of silent fallback

§2.1 noted a real cost of writing enums as names: a misspelling fails the whole file, and §2.2's best-effort contract turned that into every binding silently reverting to defaults. That is now addressed from both ends.

**`ConfigJson` uses `SuggestingEnumConverter` rather than the stock `JsonStringEnumConverter`.** It behaves identically — numbers still read, names still write, dictionary keys still work through `ReadAsPropertyName`/`WriteAsPropertyName` — but it names the offending value and suggests:

```
'DPadUpp' is not a valid GamepadButton. Did you mean 'DPadUp'?
```

**`ConfigDiagnostics` gives that message somewhere to go.** `Load()` still returns `null` and callers still fall back to defaults; the difference is that the reason is reported rather than swallowed:

```
[config] /etc/EmuSen/gamepadbindings.json: 'DPadUpp' is not a valid
GamepadButton. Did you mean 'DPadUp'? Falling back to defaults.
```

`ConfigDiagnostics.Sink` is set once by each GUI frontend's `Main`; unset, reporting is a no-op, which is what tests and any caller with nowhere to print want. `ConfigFile<T>.LastLoadError` holds the same text for a caller that would rather ask than subscribe. A **missing** file reports nothing — that is the ordinary first-run case, not a failure.

This applies to every config file, not just bindings: malformed JSON in `audio.json` is reported the same way.

---

## 7. Folder settings: a blank, relative, `~` or file value (2026-10-10)

### 7.1 What started it

On 2026-10-10 the handheld's `appsettings.json` held the bare word `genesis` as `CheatDatabaseDirectory`. Nothing read
that value as anything but a path, so the cheat database was downloaded and indexed relative to the process's working
folder, and 74 MB of `.cht` files landed inside the install, at `~/Apps/Mistress/lib/EmuSen/genesis`. The same day a
second defect of the same class was found and fixed: three paths built from an empty application-data folder became
relative and wrote `EmuSen/Logs` under the working folder (`EmuSen_Galaxia.md` §3.6). Both are one question asked of
different values: what a path setting means when it is not an absolute path to a folder. This section audits every
folder setting the player can set against four kinds of value, records what each does, fixes what an existing rule
already decides, and puts the rest to the tester. The `genesis` folder on the handheld is left where it is; whether
and when to remove it is the tester's decision and is not part of this work.

### 7.2 The settings and their readers

Eight settings name a folder: seven in `AppSettings` and one in `BigPictureInterface`. No setting in `audio.json`,
`graphics.json` or the binding files names a folder; `graphics.json`'s `slang:` filter is a path inside the shader
pack, relative to the pack by design (`EmuSen_Settings_Reference.md` §4.41), and the shader pack's own folder is
`DataStore.Shaders`, not a setting. Two settings cross into the platform library: `LogDirectory`, which the library's
`error_log.rs` reads from the settings file itself, and `StateDirectory`, which Mistress passes to `SaveLibrary` as a
parameter (`EmuSen_Galaxia.md` §5.1) and which the library's `saves.rs` receives. The other six are read only by C#.

The values reach the file three ways: Preferences' path rows and the cheat database's folder row, the same rows drawn
as text rows in Game Mode's menus (`BigMenuForm.PathRow`), and a hand edit (`nano /etc/EmuSen/appsettings.json` in the
shell, or any editor). Every row is a LunaP `PathPickerRow` with `IsEditable` set (`EmuSen_Settings_Reference.md`
§4.45.6), which commits whatever was typed, trimmed, as a pick, and refuses only a blank or unchanged box. In Game Mode
Browse… is hidden, so a typed value is the only kind there; `genesis` was typed on such a row. No DianaSh command
writes any of these settings.

### 7.3 What each value does today

*Measured on 2026-10-10*, on both halves of Galaxia (`EMUSEN_GALAXIA_NATIVE=0` and the default), by
`SettingPathsOpenTests`: a child test process with its own temporary home and working folder runs each reader with the
value `genesis`, the value `~/EmuSenTilde`, and the path of an existing file, and reports what was made under the
working folder, the home and the data home. The two halves' reports were identical, row for row, in all fourteen
cases. Rows marked *read* were established from the code and not measured, because the reader writes nothing.

| Setting | Readers | Blank (null, empty, whitespace) | Relative (`genesis`) | `~/x` | An existing file |
|---|---|---|---|---|---|
| `LogDirectory` | `ErrorLog` (C# and library), `CrashLog`, per-game logs | `~/.config/EmuSen/Logs` | the folder is made under the working folder and every error is written there | a folder literally named `~` is made under the working folder and written into | cannot be made, so the default is used (`EmuSen_Settings_Reference.md` §4.70's rule) |
| `StateDirectory` | `SaveLibrary` (C# and library) for Mistress's slots and resume state; the state list | `home/Saves/Save States` | slot states, and the resume state Mistress writes on leaving **every** game, are written under the working folder | a literal `~` folder under the working folder | the save fails with a message and an error-log entry; nothing is written |
| `CheatDatabaseDirectory` | `cheat db` (DianaSh), the Cheat Database window | `home/Cheats` | indexed and **downloaded** under the working folder: the handheld's case | a literal `~` folder under the working folder | `cheat db update` reports that the file exists; nothing is written |
| `ArtworkDirectory` | the library's cover index; Add and Remove Cover Art | `home/Artwork` | scanned there, and Add Cover Art copies into it, under the working folder | a literal `~` folder under the working folder | the scan finds nothing; Add Cover Art fails with a message, logged |
| `RomDirectory` | the library scan, the shell's `core` boundary, the Pharaoh indexer, scraping's folders and orphans, `%ROMPATH%` | "No ROM directory set"; the shell read whitespace as a folder until this date (§7.4) | read against the working folder; the shell's boundary becomes `<working folder>/genesis`; nothing is written. *Read:* the scan's entries then carry relative paths | "ROM directory not found" | "ROM directory not found" |
| `EsdeMediaDirectory` | the media sources' ES-DE folder | none | *read:* looked up against the working folder; nothing is written | *read:* a literal `~`; nothing found | *read:* nothing found |
| `BigPictureTheme` | the themed view, the theme list, Theme Settings | Mistress's own look | *read:* loaded from the working folder, and listed under its full path; nothing is written | *read:* a literal `~`; the theme does not load | *read:* the theme does not load |
| `ScreensaverSlideshowCustomDir` | the slideshow screensaver | no pictures | *read:* against the working folder; read only | expanded to the home, as ES-DE does (`EmuSen_Settings_Reference.md` §4.75) | *read:* no pictures |

So the harmful cases are the relative and `~` values of the four settings something writes into: `LogDirectory`,
`StateDirectory`, `CheatDatabaseDirectory` and `ArtworkDirectory`. Each is proven by one of `SettingPathsOpenTests`'s
eight cases, which assert that nothing is made under the working folder and fail today. They are skipped unless
`EMUSEN_SETTING_PATHS_OPEN=1` is set, because they test behaviour the tester has not yet chosen (§7.5); any option
there except leaving things as they are makes all eight pass, so they are the acceptance test for whichever is chosen.
A file value is harmless everywhere: the log falls back, and every other writer fails with a message and writes
nothing. A blank value already meant "unset" at every reader but one.

**Which folder "the working folder" is.** It is not one folder, which matters to §7.5. *Read:* Mistress starts in
whatever folder its launcher gives it, which on the handheld is the install, `~/Apps/Mistress/lib/EmuSen`; that is
where `genesis` was made. The first DianaSh interpreter a process builds moves it to the data home
(`DianaShSandbox.EnsureInitialWorkingDirectory`, called by `DianaShInterpreter.CreateDefault`), which in Mistress
happens when the console is first opened; and `cd` in the console moves it again, wherever the player goes. A relative
value therefore names a different folder before and after the console is opened, and after each `cd`. A related
consequence, also read and not measured: `cheat db update` writes to the setting's folder without passing it through
the shell's walled garden (`man hier`), so a relative value with `..` in it writes outside the shell's root.

### 7.4 What an existing rule already decided, and was fixed

**The rule.** A folder setting that is blank — null, empty, or only whitespace by .NET's `IsNullOrWhiteSpace` — is
unset: the reader uses its default folder, or none. That rule was already applied by `ErrorLog.Usable` and `saves.rs`
on both halves (their blank test is the one `GalaxiaParityTests.Blank_text_and_caseless_equality_follow_dotnet_for_every_character`
holds), and by each C# reader separately, in four copies with two spellings.

**The one reader that broke it.** `DianaShSandbox.RomLibraryDirectory`, the folder outside the shell's root that
`core` may read a ROM from, tested `{ Length: > 0 }`, so a whitespace `RomDirectory` became a library at
`<working folder>/<spaces>` while Mistress's scan said "No ROM directory set". `SettingFoldersTests.The_shell_reads_a_blank_rom_folder_as_no_library`
failed on the code before the change (*measured on 2026-10-10*, both cases) and passes after it. The harm was small;
the disagreement was the point.

**The change.** `SettingFolders`, in `EmuSen.Galaxia.Library`, is now the one owner of the rule: `Named(value)` is
null for a blank value and the value as written otherwise, and `States`, `Cheats`, `Artwork` and `Roms` give each
setting's folder with its default. Every C# reader that decided the rule for itself asks it instead: `cheat db`, the
Cheat Database window, the library's cover folder and state list, the shell's ROM boundary, the Pharaoh indexer,
scraping's folder layout and orphan list, the folders' status line, and the theme-collection check. `ErrorLog` keeps
its own test, because it also falls back from a folder that cannot be made and its library half reads the settings
file itself; `SaveLibrary` keeps its own, because its directory is a parameter on both halves.
`SettingFoldersTests.The_states_folder_is_the_one_save_library_writes_into` holds `SettingFolders.States` to the folder
`SaveLibrary.StatePathFor` names, for blank, absolute, relative, `~` and padded values, so the state list and the
writer cannot drift; it passes on both halves.

No Rust change was needed: the two settings that cross into the library already had the rule there, and the
resolver's other four settings never reach it. `SettingFolders` passes relative, `~` and padded values through exactly
as written, by design, until §7.5 is answered; that is the one place the answer will go.

**A refactor that the proof needed.** Add Cover Art's copy moved from `MainWindow` into `ArtworkIndex.Place`, beside
`ArtworkIndex.PathFor`, which already owned where the copy goes, so that a test could run the writer itself rather
than a copy of it.

### 7.5 Put to the tester

**Decided 2026-10-11:** all four, each as recommended: a relative value is refused at entry and one already stored is read as unset and reported once; `~` and `~/` expand to the home; a file value is refused at entry; LunaP's `PathPickerRow` gains an opt-in blank-means-unset, a minor version, so Game Mode can clear a folder setting.

No rule exists for a relative value, a `~`, or a file value in a folder setting. The two places that expand `~` do so
because ES-DE does, for values ES-DE defines: the screensaver's custom folder (`EmuSen_Settings_Reference.md` §4.75)
and paths inside a theme (§4.73). The theme rule that leaves a bare relative path to the working folder was adopted
for the same reason (`EmuSen_BigPicture.md` Q111). Neither is a decision about EmuSen's own settings, and nothing
below is built.

1. **A relative value.** Options: (a) refuse it at the point of entry, with a message saying a full path is needed,
   and read a stored relative value (a hand edit, or a file from an older build) as `EmuSen_Settings_Reference.md`
   §4.70 reads a log folder that cannot be made: use the default, say so once through `ConfigDiagnostics` and the
   error log, and leave the setting as written; (b) resolve it against the data home, the shell's `/`; (c) resolve it against the player's home;
   (d) leave it as it is. **Recommended: (a).** §7.3 shows a relative value has no stable base, so (d) is the defect
   itself. (b) and (c) each choose a base the player cannot see when typing, and the handheld's case is exactly a
   value whose effect the player could not see: the data would land somewhere sensible but still unannounced. (b)
   agrees with the shell, where `/` is the data home, but not with a desktop folder picker, which only ever gives an
   absolute path. Refusing at entry is the only option that tells the player at the moment the value is typed, and the
   fallback for stored values extends a rule the project already has rather than inventing one.
2. **A `~`.** Options: (a) expand `~` and `~/…` to the player's home in `SettingFolders`, and on the library's side for
   `LogDirectory` and `StateDirectory`, which would need the home from the library's tree (`tree.rs` already computes
   it); (b) treat it as relative under question 1. **Recommended: (a)**, with `~user` forms not expanded and so
   refused under question 1. `~/Roms` is what a player types on a handheld's keyboard, and the screensaver's folder already
   reads `~` this way in the same settings file, so a second meaning for the same character there would be the worse
   inconsistency.
3. **A file value.** Today harmless (§7.3). Options: refuse at entry when the typed path names an existing file, or
   change nothing. **Recommended: refuse at entry**, since the message costs one check and a row that accepts a file
   for a folder invites the question of what it did with it; no change at read.
4. **Clearing a setting.** A typed blank is not committed (`PathPickerRow.CommitTyped`), so in Game Mode a folder
   setting, once set, cannot be returned to its default except by typing another path or editing the file; the
   handheld's `genesis` could not have been undone from where it was typed. Options: (a) an opt-in extension of
   `PathPickerRow` in LunaP, a minor version, under which a committed blank box raises `PathPicked` with an empty
   string, used by the rows that have a default; (b) a separate "Use Default" action beside each such row; (c) nothing.
   **Recommended: (a)**: it is a capability of the existing surface rather than a new control, and the empty string is
   already what "unset" means to every reader (§7.4).

### 7.6 Tests

- `SettingFoldersTests` (`EmuSen.WiseMan/Galaxia/`): blank values (null, empty, spaces, a tab, a no-break space) take
  each setting's default; a named value is passed on as written; the state list's folder is `SaveLibrary`'s for every
  kind of value; the shell reads a whitespace `RomDirectory` as no library. Run on both halves.
- `SettingPathsOpenTests`: §7.3's measurement in a child process whose home and working folder belong to the test, and
  the eight open cases of §7.5's questions 1 and 2, skipped unless `EMUSEN_SETTING_PATHS_OPEN=1`. The child runs on
  the half its parent runs on, and the report it writes names the half.
