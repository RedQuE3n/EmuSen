# EmuSen.Galaxia — the librarian

*This revision: the DianaOS shell is now rooted at `/home` itself, so the install tree is unreachable from it, and `etc/EmuSen` moved under home to stay inside that root (§3.2). Before that, the data tree moved to `/home` and the ROM-library migration policy landed (§3.2, §3.3). Previously: Galaxia took over the data half. The `Usr/Home` directories moved out of `DianaOSSandbox` into `DataStore` (§2, §3), binary artifacts got the durable write config already had (§4), and there is now one spelling of what a ROM's save files are called (§5). Before that, config only — `EmuSen_Config_Reference.md`, which is still the per-file reference.*

---

## 1. What Galaxia is

Galaxia answers one question for the whole program: **where does a file live, and how is it read and written safely.** Not what is in it — the meaning of a config value or a save state's byte layout belongs to whoever owns that data. Galaxia owns the location and the I/O contract.

It has two halves, deliberately the same shape:

| | Config half | Data half |
|---|---|---|
| Where | `ConfigStore` → `<install>/home/etc/EmuSen` | `DataStore` → `<install>/home` (§3) |
| One artifact | `ConfigFile<T>` (typed JSON) | `AtomicFile` (bytes) (§4) |
| Naming policy | the filename is the constant | `SaveLibrary` (§5) |
| Test redirect | `ConfigStore.OverrideDirectory` | `DataStore.OverrideDirectory` (§3.1) |
| Failure | null, `LastLoadError`, `ConfigDiagnostics` | null / false, `ConfigDiagnostics` |

**It is a leaf, and that is load-bearing.** `EmuSen.Galaxia.csproj` has no `ProjectReference` and no `PackageReference`, by hard constraint rather than preference: `EmuSen.DianaOS` references Galaxia, and `EmuSen` (the core) references DianaOS, so anything Galaxia depended on upward would close a cycle. `EmuSen_Config_Reference.md` §1.1 has the full argument. That constraint is also what makes it the *only* possible home for this: DianaOS, all three cores, LunaP and both frontends already depend on it, and nothing else in the tree is reachable from all of them.

A consequence worth stating plainly: **the cores do not live here and never will.** A core needs DianaOS for logging and Cauldron for telemetry, and owns its own timing outright. What moved into Galaxia is the *question* a core asks ("what path does this ROM's save go to") and the *mechanism* it uses to write, not the cartridge, the mapper, or a single byte of SRAM.

---

## 2. Why the directories moved

`DianaOSSandbox` is a shell class. Its job is chroot-style path confinement — keeping `cd ../../..` from wandering the DianaOS shell off the project (see its own header, and `man hier`). It also happened to own `SavesDirectory`, `SaveStatesDirectory`, `FirmwareDirectory`, `CheatDatabaseDirectory` and `LogsDirectory`, which meant `Venus - SNES/Memory/Cartridge.cs` reached into the *shell project* purely to learn where to put a `.srm`. A core asking a debugger shell where the user's saves go is backwards.

This exact inversion was already corrected once. `DianaOSSandbox` used to own the walk-up that finds the project root; it now forwards to `ConfigRoot`, and `ConfigStoreTests` pins the two to the same answer. §2 of this doc is that same move applied to the other six directories.

**The six properties still exist and still mean the same thing — they forward.** Nothing that reads `DianaOSSandbox.SavesDirectory` changed, and `EnsureSkeleton` builds the tree from `DataStore`. That is deliberate: `DianaOS/HierTests.cs` and `DianaOS/DianaOSSandboxTests.cs` pin the physical paths and had to keep passing **without edits**. They do. If either had needed changing, the forwarding would have been wrong.

That move changed no path on disk. The *next* one did — see §3.2 for the relocation to `/home` and how existing data got there.

---

## 3. `DataStore` — where saved data goes

The data-side sibling of `ConfigStore`, rooted at the same `ConfigRoot.Directory`:

The shell is rooted here: to DianaOS, this directory **is** `/`, and the install above it cannot be reached (`man hier`).

```
<install>/home/          <-- the shell's own '/'
  etc/EmuSen/   config, still reachable as /etc/EmuSen
  tmp/          scratch; tmp/WiseMan is the test suite's
  Games/        the user's ROM library
  Logs/         dump/load/screenshot/recording output
  Saves/        battery-backed cartridge SRAM (.srm)
    Save States/  whole-machine snapshots (.state)
  Firmware/     coprocessor dumps the user supplies    EmuSen_Firmware.md §2
  Cheats/       the user's own .cht tree               `man cheat`
```

`man hier` remains the user-facing description of this tree; this section is about who *owns* it.

### 3.1 Two overrides, still separate

`DataStore.OverrideDirectory` redirects data and only data. It does not move `ConfigStore.Directory`, and neither moves `ConfigRoot`.

This is the same separation `EmuSen_Config_Reference.md` §1.3 argues for, now in both directions: a test that redirects saves must not relocate config underneath the rest of the suite, and vice versa. `DataStoreTests.The_two_overrides_do_not_move_each_other` and `…Override_does_not_move_the_sandbox_root` pin it.

### 3.2 Migrating out of the old tree

This lived at `<root>/EmuSen.DianaOS/DianaOS/Usr/Home/` until the layout cleanup — five levels down, so a *published* user's save folder was named after a C# project. `DataMigration` copies the old tree's contents into `/home` on first run and is invoked from `DianaOSSandbox.EnsureSkeleton`. It also copies `etc/EmuSen`, which moved under home when the shell was rooted there — the *shell-visible* path `/etc/EmuSen` is unchanged, since `/` now means home; only the physical location moved.

It **copies and never overwrites**, the same rule as `ConfigFile<T>.MigrateFromLegacy`: the migration itself destroys nothing, and a destination file that already exists always wins. Running it twice copies nothing the second time.

Copying rather than moving buys a verification window, not a permanent fallback. In this repo that window has closed: the four source directories were confirmed byte-identical to their copies and then deleted, so `Usr/Home/` now holds only `Documents` (committed source) and the untouched `Games`/`Roms`. An older build run against this tree would come up with no saves — expected, and the reason the check happened before the delete rather than after.

### 3.3 The ROM library is never migrated

`Games` and `Roms` are excluded from that copy on purpose. They are the user's own content rather than something the emulator wrote, they are large (95 MB in the author's own tree at the time of the move), and the library is under a standing never-delete rule. Silently duplicating a ROM library is bad behaviour regardless of whether it is destructive.

So the old directories are left exactly as they are, and `DataMigration.RemainingLibraryDirectories()` reports any that still hold files so a caller can say where they went.

### 3.3a Where the library actually lives

**`AppSettings.RomDirectory` is the answer, and it is the only one.** It is user-configured, so a library works from wherever it sits, and nothing in the codebase resolves a game through a hardcoded path.

That is worth stating flatly because the surrounding names invite the opposite conclusion, and did:

- **`DataStore.Games` is not the ROM library.** It is one entry in the skeleton `DianaOSSandbox.EnsureSkeleton()` creates so `hier` and `ls` show a populated tree. Three things touch it — that `CreateDirectory`, the `RealRom` test fixture's fallback list, and a `DataStoreTests` path-shape assertion. Nothing looks up a game in it. Its comment used to claim otherwise, which is exactly how a second `DataStore.Roms` property nearly got added beside it as "where the library *actually* lives"; two properties, neither authoritative, and the real mechanism untouched in a config file.
- **`DataMigration.LibraryDirectories` is not a locator either.** It is the list of directory names that might still hold files after the tree move, so startup can report where they went.

**As of 2026-08-05 the in-tree ROMs are gone.** `Usr/Home/Roms/` held eight ROMs used as the test corpus; every one had a byte-identical copy in the author's configured library, and they were removed rather than kept as a second copy that could drift. `Usr/Home/Roms/` itself stays — an end user is free to keep a library inside the shell's tree in a finished build, and `RealRom` still falls back to it — but development reads `AppSettings.RomDirectory`.

`RealRom.Find` therefore tries the configured library first, and searches recursively under the console directory when the flat `<root>/<console>/<file>` path misses. A real library sorts by region (`NES/USA/Super Mario Bros 3 (U) (PRG 0).nes`), so the flat guess is the exception rather than the rule.

---

## 4. `AtomicFile` — the durability contract

`ConfigFile<T>` has always written tmp-then-rename, so an interrupted save leaves the previous config intact rather than a truncated one (§2.3 of the config reference). Binary artifacts had nothing equivalent — save states and `.srm` files went through bare `File.WriteAllBytes`.

That was not a theoretical gap. **All three cores autosave cartridge SRAM every 300 frames — roughly every five seconds of play** (`VenusCore.Schedule.cs`, `MoonCore.Schedule.cs`, `MercuryCore.cs`). Every one of those writes was a window in which a crash, a kill, or a power loss would leave a half-written save. It is the one file a player genuinely cannot afford to lose, and it was the least protected thing the emulator wrote.

`AtomicFile` is deliberately **static, not a type to instantiate**. `ConfigFile<T>` earns its object-ness by carrying a category, a filename and migration state across many reads; a `.srm` write is "these bytes, to this already-known path, atomically." A handle type would be symmetry for its own sake.

```csharp
byte[]? TryRead(string path)          // null for missing or unreadable
bool    Write(string path, byte[] b)  // creates the directory, .tmp, rename
```

### 4.1 Why rename rather than write-in-place

`File.Move(temp, path, overwrite: true)` is atomic at the filesystem level: the destination is either the whole old file or the whole new one, never a partial write. The truncation risk moves onto the `.tmp` file, which nothing ever reads — `AtomicFileTests.An_abandoned_temp_file_never_becomes_the_live_file` pins that a leftover temp from a killed process is ignored.

Failure stays best-effort, exactly as it is for config: a save that cannot reach the disk reports through `ConfigDiagnostics` and returns false. It does not throw, because the call sites are a cartridge mid-frame and a UI button handler, and neither should take the program down over a full disk.

---

## 5. `SaveLibrary` — what the files are called

One spelling of the naming rules, previously duplicated across two frontends and computed three different ways by three cores.

```csharp
string SramPathFor(romPath)                            // Saves/<stem>.srm
string StatePathFor(romPath, slot, directoryOverride)  // Save States/<stem>[.slotN].state
```

### 5.1 Two decisions worth knowing

**Slot 1 is the unsuffixed name.** `<stem>.state`, not `<stem>.slot1.state` — that is what Hotaru, Mistress and DianaOS's `state save` have always written, and changing it would orphan every existing save state. Slots 2 and up take `.slotN`.

**The directory override is a parameter, not a read.** `StatePathFor` could have consulted `AppSettings.StateDirectory` itself, since `AppSettings` lives in Galaxia too. It does not, because that would give path resolution a dependency on settings load order — a `ConfigFile` read on a code path that a cartridge constructor reaches. Mistress passes its Preferences value in; everything else passes nothing and gets the default.

---

## 6. What this pass deliberately did not do

Two things were separated out rather than bundled in, so each stands on its own review:

- **Moon and Mercury still write `.srm` beside the ROM.** Venus writes to `Saves/`; the other two use `Path.ChangeExtension(romPath, ".srm")`, which puts emulator output inside the user's own ROM folder. They gained the durable write in this pass and nothing else. Relocating them is a user-visible behavior change and needs a copy-don't-move migration in the shape of `ConfigFile<T>.MigrateFromLegacy`, read-only on `Games/`.
- **`SaveLibrary` cannot enumerate.** It resolves names; it does not list which slots exist or when they were written. That is the piece which would let a launcher show a save list through LunaP without loading a core — worth building when a UI actually consumes it, not before.

---

## 7. Test coverage

In `EmuSen.WiseMan/Galaxia/`:

- **`DataStoreTests`** — every directory's location, that the shell forwards all six, and both override-separation properties.
- **`AtomicFileTests`** — directory creation, replacement, no temp left behind, an abandoned temp never read, round-trip, and that an unwritable path reports and returns false instead of throwing.
- **`SaveLibraryTests`** — both names, slot 1's unsuffixed form, blank-override fallback, and that the ROM's own folder never leaks into the answer.

And, unchanged and still green, `DianaOS/HierTests.cs` and `DianaOS/DianaOSSandboxTests.cs` — see §2 for why their staying untouched is the point.
