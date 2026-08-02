# EmuSen Config Reference — `EmuSen.Galaxia`

Every config file this program keeps on disk, where it lives, and how it is read and written. `EmuSen.Galaxia` is the one project that answers those questions; everything else asks it.

This document is about *persistence*. What each individual setting actually does is elsewhere: `EmuSen_Settings_Reference.md` §1 for debug flags, §2 for audio, §3 for graphics, §4 for the frontend's input bindings.

---

## 1. Where config lives

### 1.1 Galaxia is a leaf, and root discovery lives in it

`EmuSen.Galaxia` has no `ProjectReference` and no `PackageReference`. That is a hard constraint, not a preference.

`EmuSen.DianaOS` needs Galaxia, because `CheatRegistry` saves and loads cheat files and `cheat save`/`cheat load` are DianaOS commands. `EmuSen` already references `EmuSen.DianaOS` (`DebugSettings.MasterLoggingEnabled` forwards to `DianaOSLogging`). So anything Galaxia depended on upward would close a cycle.

That constraint is what makes the project genuinely agnostic rather than merely shared. Nothing in it can name `Avalonia.Input.Key`, `SDL.GamepadButton` or `SnesButton`, so:

- the **models** in Galaxia are plain data (`AppSettings`, `AudioConfig`, `GraphicsConfig`, `CheatFile`);
- the typed maps that *do* use foreign enums (`ControllerKeyMap`, `GamepadBindingMap`, `HotkeyBindingMap`) stay in their own projects and use Galaxia only for persistence.

The same constraint moved root discovery. `DianaOSSandbox` used to own the walk-up that finds the project root; it now delegates to `ConfigRoot`, which holds the walk and both markers. `DianaOSSandbox.RootDirectory`, `ComputeRootFor`, `PublishedRootDirName` and `RootMarkerFileName` all still exist and still mean the same thing — they forward. Nothing about how the root is found changed, only which project holds the code. `ConfigStoreTests` pins the two to the same answer.

### 1.2 The paths

Config lives inside DianaOS's own sandbox tree, at `<root>/etc/EmuSen` — reachable from the shell as `/etc/EmuSen`, which is the point. See `man hier`.

```
/etc/EmuSen/
  appsettings.json        general app preferences        §3.1
  keybindings.json        keyboard -> SNES button
  gamepadbindings.json    pad button -> SNES button
  hotkeybindings.json     keyboard -> emulator action
  audio.json              §3.2
  graphics.json           §3.3
  cheats/
    <name>.json           one saved cheat set per file   §3.4
```

`ConfigStore.For(fileName)` resolves the flat files. `ConfigStore.For(category, fileName)` resolves the one-subdirectory-per-category kind, which today means cheats — config there can be many of, named by the user, rather than one well-known file.

Because this is inside the sandbox, `cat`, `nano`, `find` and `>` all reach it. In a *published* tree it is also inside the walled garden along with the binaries, which is why `rm` and `mv` are suspended there — see `man hier`. `DianaOSSandbox.EnsureSkeleton` creates the directory at startup so it is always there to write into.

### 1.3 Two separate overrides, on purpose

`ConfigStore.OverrideDirectory` redirects config and **only** config. `ConfigRoot` is not redirectable and does not move with it.

This matters because the sandbox root is also where saves, save states, logs and firmware live. If redirecting config also moved the root, any test that so much as read a binding would relocate every other DianaOS path underneath the rest of the suite. `ConfigStoreTests.Override_does_not_move_the_sandbox_root` pins that separation.

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

The one thing failure must never do is destroy what is already there. A file that fails to parse is left exactly as it is on disk (`SettingsPersistenceTests.An_unreadable_file_is_left_on_disk_untouched`) — the user's hand edit is still there to be fixed, rather than being overwritten with defaults by the next save.

### 2.3 Writes are atomic

`Save()` writes the whole file to `<path>.tmp` and then renames it over the target. A rename within a directory is atomic on both NTFS and ext4, so an interrupted save leaves the previous config intact rather than a truncated one.

None of the four original classes did this — they wrote in place with `File.WriteAllText`. Rebinding a key saves immediately, so a crash or a power loss in that window could leave a half-written `keybindings.json`, which then loads as corrupt and silently resets every binding to defaults. Not observed in practice, but it costs one rename to remove.

---

## 3. The config files

### 3.1 `appsettings.json` — `AppSettings`

Moved here from `EmuSen.Mistress/Settings/`; it holds no frontend types, so there was no reason for it to be frontend-specific. Log/ROM/state directories, the selected core, and the three input preferences the settings UI writes (`MirrorPlayer1ToPlayer2`, `AnalogStickAsDpad`, `StickDeadzone` — see `EmuSen_Settings_Reference.md` §4.4).

The class name and the file name are unchanged, so existing files load as-is.

### 3.2 `audio.json` — `AudioConfig`

`EmuSen.Audio.AudioSettings` is a static hub of mutable fields, which `System.Text.Json` cannot serialize. `AudioConfig` is its on-disk mirror; `AudioSettings.LoadFromDisk()`/`SaveToDisk()` own the mapping in both directions.

`LoadFromDisk` **clamps** every value it applies. That is not defensive habit — it follows from §2.1. This file exists to be hand-edited, and `AudioSettings.OutputTargetFrames` divides by `SampleRate`, so a typed-in `0` would be a crash rather than a bad setting.

On first run, when no file exists, `LoadFromDisk` seeds one from the current defaults so there is something to open. A file that already exists is never rewritten behind the user's back.

### 3.3 `graphics.json` — `GraphicsConfig`

The same arrangement for `EmuSen.Graphics.GraphicsSettings`: window size, target FPS, VSync, resizability, bilinear filtering. Clamped and seeded exactly as audio is.

`WindowTitle` is deliberately **not** persisted. It is branding rather than a preference, and a corrupt config blanking the application's title is a worse outcome than not being able to change it from a file.

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

**Files written before writes were a list still load.** A pre-multi-write entry carries `space`/`address`/`value` flat on the cheat itself instead of a `writes` array; `CheatFileEntry.EffectiveWrites` synthesizes the single 1-byte `Set` write those describe, so nothing saved earlier is stranded. Saving always writes the new shape and leaves the old fields null, so re-saving quietly migrates a file. The mapping between this carrier and `CheatWrite` lives in `CheatRegistry`, not on the carrier, because `EmuSen.Galaxia` cannot reference `EmuSen.DianaOS` — the dependency runs the other way.

`"address": 8257561` would be technically equivalent and useless to a person reading it with `cat`. Since the entire reason for putting config in `/etc` was to make it reachable from the shell, the on-disk shape follows the way SNES addresses are actually written. Reading tolerates `0x` and `$` prefixes and surrounding whitespace, because those are what people paste in.

Three behaviours worth stating:

- **A malformed entry is skipped, not fatal.** `LoadFrom` returns `(loaded, skipped)` and `cheat load` reports both. One typo in a hand-written file costs that one line, not the file.
- **`load` merges, it does not replace.** It adds to whatever is already loaded; `cheat clear` first to replace. This is the honest reading of "load", and the alternative silently discards cheats added since the last save.
- **Nothing loads automatically.** There is no per-ROM auto-apply. A cheat set changes the run only when asked for, so a file saved months ago can't quietly alter a later session — which matters when the emulator is being used to investigate whether a game behaves correctly.

`CheatFile.IsValidName` rejects a name rather than sanitizing it. The name arrives from a shell argument, so it must not be able to walk out of the cheats directory; and silently rewriting what the user typed would save to a file they didn't name.

### 3.4a `CheatDatabaseDirectory` — using a cheat tree you already have

`AppSettings.CheatDatabaseDirectory` points at a directory tree of `.cht` files, one per game in per-system folders — the exact shape RetroArch stores its cheats in. Point it at an existing RetroArch cheats folder and it is indexed in place: no copying, no conversion, no import step. Unset, it falls back to `DianaOSSandbox.CheatDatabaseDirectory` (`Usr/Home/Cheats`).

This is deliberately a *directory setting* rather than a bundled asset. **EmuSen ships no cheat data and redistributes none.** The libretro cheat database is licensed CC BY-SA 4.0, but its own README states the codes were "collected from any available source on the web" — so the grant is only as good as libretro's rights in an aggregated corpus, and the EU sui generis database right applies independently of copyright either way. Indexing a folder the user already has, or downloading one to their machine at their explicit request (`cheat db update`, `Settings > Cheat Database...`), keeps that question off this project entirely. Attribution is shown at the point of download, since it is a licence condition rather than a footnote.

`CheatDatabase` and `CheatDatabaseInstaller` both live in `EmuSen.DianaOS` and are core-agnostic; the installer extracts only `.cht` entries and refuses any archive entry that resolves outside the target directory.

### 3.5 What is deliberately not persisted

`DebugSettings` has no config file. Its flags are trace toggles for a specific investigation, and persisting them means a run three weeks later starts spewing DMA traces because of something enabled once and forgotten — which is exactly the failure `MasterLoggingEnabled` was added to fix (see `EmuSen_Settings_Reference.md` §1). Debug flags default off every run, and `log` sets them for the session.

---

## 4. Wiring

`AudioSettings.LoadFromDisk()` and `GraphicsSettings.LoadFromDisk()` are called from `Main` in both GUI frontends (`EmuSen.Mistress/Program.cs`, `EmuSen.Hotaru/Program.cs`), before any window exists — graphics config decides the window's size, so it has to be applied first.

`EmuSen.Pharaoh` and `EmuSen.Tomoe` do not load either. They are headless CLI tools with no window and no audio device; window size and master volume mean nothing there.

No GUI or shell command edits `audio.json`/`graphics.json` yet — they are read at startup and hand-edited. The Preferences window still writes `appsettings.json` as before, and the input settings window still writes the three binding files.

---

## 5. Test coverage

`EmuSen.WiseMan/Galaxia/`:

- `ConfigFileTests` — round-trip, missing/corrupt/hand-edited input, the temp file not being left behind, category paths, `Path` following a moved override, and both migration directions (it happens once; it never runs again once the new file exists).
- `ConfigStoreTests` — the `/etc/EmuSen` shape, DianaOS and Galaxia agreeing on the root, and the config override *not* moving the sandbox root.
- `CheatFileTests` — both cheat kinds through a registry round-trip, hex-text output, the `0x`/`$`/whitespace input forms, a broken entry being skipped while the rest load, and the name-validation cases that could otherwise escape the directory.
- `SettingsPersistenceTests` — both hubs round-tripping, clamping of hand-edited zeroes, first-run seeding, and a corrupt file being left untouched on disk.
- `ConfigEnumFormatTests` — §2.1's enum-name format: a numeric file from an older build mapping to the same physical buttons, saves writing names, the numeric-to-name upgrade in one pass, half-edited mixed files, case-insensitive names, and the misspelled-name-resets-the-file cost.

- `SuggestionTests` — §6.1's engine: transposition scoring, the length-scaled tolerance, joint-best-only results, prefix tie-breaking, and the empty/absurd inputs.
- `ConfigDiagnosticsTests` — §6.2: a misspelled enum value and a misspelled dictionary *key* both being named and corrected, the report carrying the file path, malformed JSON reported too, and a missing file reporting nothing.
- `SuggestionMessageTests` (`EmuSen.WiseMan/DianaOS/`) — the shell's unknown-command path and four of the five subcommand sites, plus the "Try a/b/c" list still naming everything.

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
