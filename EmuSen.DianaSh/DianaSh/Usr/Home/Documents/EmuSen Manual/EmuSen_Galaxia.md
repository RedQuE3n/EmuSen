# EmuSen.Galaxia — the librarian

*This revision: on macOS the tree lives in `~/Library/Application Support/EmuSen`, outside the app bundle, and the bundle's manual is seeded into it (§3.4, 2026-09-27). Previously: the DianaSh shell was rooted at `/home` itself, so the install tree is unreachable from it, and `etc/EmuSen` moved under home to stay inside that root (§3.2). Before that, the data tree moved to `/home` and the ROM-library migration policy landed (§3.2, §3.3). Previously: Galaxia took over the data half. The `Usr/Home` directories moved out of `DianaShSandbox` into `DataStore` (§2, §3), binary artifacts got the durable write config already had (§4), and there is now one spelling of what a ROM's save files are called (§5). Before that, config only — `EmuSen_Config_Reference.md`, which is still the per-file reference.*

---

## 1. What Galaxia is

Galaxia answers one question for the whole program: **where does a file live, and how is it read and written safely.** Not what is in it — the meaning of a config value or a save state's byte layout belongs to whoever owns that data. Galaxia owns the location and the I/O contract.

It has two halves, deliberately the same shape:

| | Config half | Data half |
|---|---|---|
| Where | `ConfigStore` → `<root>/home/etc/EmuSen` | `DataStore` → `<root>/home` (§3; on macOS the root is in Application Support, §3.4) |
| One artifact | `ConfigFile<T>` (typed JSON) | `AtomicFile` (bytes) (§4) |
| Naming policy | the filename is the constant | `SaveLibrary` (§5) |
| Test redirect | `ConfigStore.OverrideDirectory` | `DataStore.OverrideDirectory` (§3.1) |
| Failure | null, `LastLoadError`, `ConfigDiagnostics` | null / false, `ConfigDiagnostics` |

**It is a leaf, and that is load-bearing.** `EmuSen.Galaxia.csproj` has no `ProjectReference` and no `PackageReference`, by hard constraint rather than preference: `EmuSen.DianaSh` references Galaxia, and `EmuSen` (the core) references DianaSh, so anything Galaxia depended on upward would close a cycle. `EmuSen_Config_Reference.md` §1.1 has the full argument. That constraint is also what makes it the *only* possible home for this: DianaSh, all three cores, LunaP and both frontends already depend on it, and nothing else in the tree is reachable from all of them.

A consequence worth stating plainly: **the cores do not live here and never will.** A core needs DianaSh for logging and for telemetry (the `EmuSen.Cauldron` namespace, an assembly of its own until 2026-10-07), and owns its own timing outright. What moved into Galaxia is the *question* a core asks ("what path does this ROM's save go to") and the *mechanism* it uses to write, not the cartridge, the mapper, or a single byte of SRAM.

---

## 2. Why the directories moved

`DianaShSandbox` is a shell class. Its job is chroot-style path confinement — keeping `cd ../../..` from wandering the DianaSh shell off the project (see its own header, and `man hier`). It also happened to own `SavesDirectory`, `SaveStatesDirectory`, `FirmwareDirectory`, `CheatDatabaseDirectory` and `LogsDirectory`, which meant `Venus - SNES/Memory/Cartridge.cs` reached into the *shell project* purely to learn where to put a `.srm`. A core asking a debugger shell where the player's saves go is backwards.

This exact inversion was already corrected once. `DianaShSandbox` used to own the walk-up that finds the project root; it now forwards to `ConfigRoot`, and `ConfigStoreTests` pins the two to the same answer. §2 of this doc is that same move applied to the other six directories.

**The six properties still exist and still mean the same thing — they forward.** Nothing that reads `DianaShSandbox.SavesDirectory` changed, and `EnsureSkeleton` builds the tree from `DataStore`. That is deliberate: `DianaSh/HierTests.cs` and `DianaSh/DianaShSandboxTests.cs` pin the physical paths and had to keep passing **without edits**. They do. If either had needed changing, the forwarding would have been wrong.

That move changed no path on disk. The *next* one did — see §3.2 for the relocation to `/home` and how existing data got there.

---

## 3. `DataStore` — where saved data goes

The data-side sibling of `ConfigStore`, rooted at the same `ConfigRoot.Directory`:

The shell is rooted here: to DianaSh, this directory **is** `/`, and the install above it cannot be reached (`man hier`).

```
<install>/home/          <-- the shell's own '/'
  etc/EmuSen/   config, still reachable as /etc/EmuSen
  tmp/          scratch; tmp/WiseMan is the test suite's
  Games/        the player's ROM library
  Logs/         dump/load/screenshot/recording output
  Saves/        battery-backed cartridge SRAM (.srm)
    Save States/  whole-machine snapshots (.state)
  Library/      Mistress's kept databases              EmuSen_Settings_Reference.md §4.32, §4.64
    games.db      the player's library: favourites, play time, collections, edits
    records.db    each save state's record and the shader pack's build
  Firmware/     coprocessor dumps the player supplies  EmuSen_Firmware.md §2
  Cheats/       the player's own .cht tree             `man cheat`
  Shaders/      shader packs downloaded on request     EmuSen_Settings_Reference.md §4.41
    spirv-cache.db  their compiled SPIR-V; a deletable cache   EmuSen_Serenity.md §9.4
```

`man hier` remains the user-facing description of this tree; this section is about who *owns* it.

### 3.1 Two overrides, still separate

`DataStore.OverrideDirectory` redirects data and only data. It does not move `ConfigStore.Directory`, and neither moves `ConfigRoot`.

This is the same separation `EmuSen_Config_Reference.md` §1.3 argues for, now in both directions: a test that redirects saves must not relocate config underneath the rest of the suite, and vice versa. `DataStoreTests.The_two_overrides_do_not_move_each_other` and `…Override_does_not_move_the_sandbox_root` pin it.

### 3.2 Migrating out of the old tree

This lived at `<root>/EmuSen.DianaSh/DianaSh/Usr/Home/` until the layout cleanup — five levels down, so a *published* user's save folder was named after a C# project. `DataMigration` copies the old tree's contents into `/home` on first run and is invoked from `DianaShSandbox.EnsureSkeleton`. It also copies `etc/EmuSen`, which moved under home when the shell was rooted there — the *shell-visible* path `/etc/EmuSen` is unchanged, since `/` now means home; only the physical location moved.

It **copies and never overwrites**, the same rule as `ConfigFile<T>.MigrateFromLegacy`: the migration itself destroys nothing, and a destination file that already exists always wins. Running it twice copies nothing the second time.

A tree is not migrated onto itself. The two roots' full paths are compared without regard to case, which on Windows also unifies `/` and `\`, `..`, and trailing dots and spaces; a spelling the text cannot tell is the tree (the `\\?\` form, an 8.3 name, a junction or link, a decomposed character on macOS) is walked, and copies nothing, because every file finds itself already there. Both halves hold this on Linux, Windows and macOS (`EmuSen_RustPlatform.md` §26.1).

Copying rather than moving buys a verification window, not a permanent fallback. In this repo that window has closed: the four source directories were confirmed byte-identical to their copies and then deleted, so `Usr/Home/` now holds only `Documents` (committed source) and the untouched `Games`/`Roms`. An older build run against this tree would come up with no saves — expected, and the reason the check happened before the delete rather than after.

### 3.3 The ROM library is never migrated

`Games` and `Roms` are excluded from that copy on purpose. They are the player's own content rather than something the emulator wrote, they are large (95 MB in the author's own tree at the time of the move), and the library is under a standing never-delete rule. Silently duplicating a ROM library is bad behaviour regardless of whether it is destructive.

So the old directories are left exactly as they are, and `DataMigration.RemainingLibraryDirectories()` reports any that still hold files so a caller can say where they went.

### 3.3a Where the library actually lives

**`AppSettings.RomDirectory` is the answer, and it is the only one.** It is user-configured, so a library works from wherever it sits, and nothing in the codebase resolves a game through a hardcoded path.

That is worth stating flatly because the surrounding names invite the opposite conclusion, and did:

- **`DataStore.Games` is not the ROM library.** It is one entry in the skeleton `DianaShSandbox.EnsureSkeleton()` creates so `hier` and `ls` show a populated tree. Three things touch it — that `CreateDirectory`, the `RealRom` test fixture's fallback list, and a `DataStoreTests` path-shape assertion. Nothing looks up a game in it. Its comment used to claim otherwise, which is exactly how a second `DataStore.Roms` property nearly got added beside it as "where the library *actually* lives"; two properties, neither authoritative, and the real mechanism untouched in a config file.
- **`DataMigration.LibraryDirectories` is not a locator either.** It is the list of directory names that might still hold files after the tree move, so startup can report where they went.

**As of 2026-08-05 the in-tree ROMs are gone.** `Usr/Home/Roms/` held eight ROMs used as the test corpus; every one had a byte-identical copy in the author's configured library, and they were removed rather than kept as a second copy that could drift. `Usr/Home/Roms/` itself stays — an end user is free to keep a library inside the shell's tree in a finished build, and `RealRom` still falls back to it — but development reads `AppSettings.RomDirectory`.

`RealRom.Find` therefore tries the configured library first, and searches recursively under the console directory when the flat `<root>/<console>/<file>` path misses. A real library sorts by region (`NES/USA/Super Mario Bros 3 (U) (PRG 0).nes`), so the flat guess is the exception rather than the rule.

### 3.4 The data home on macOS (2026-09-27)

Decided 2026-09-27: macOS installs as `EmuSen.app` in `/Applications`. Linux has no application standard, so its publish is a self-contained tree with the player's data inside it (`man hier`); macOS has one, and a bundle is read-only by that standard. It is replaced whole by an update, it is signed (a file written into it breaks the seal), and `/Applications` may not be writable by the player at all. So on a Mac the root moves out of the install:

| Where the program runs from | `ConfigRoot.Directory` |
|---|---|
| inside a bundle (`…/X.app/Contents/MacOS`), on macOS | `~/Library/Application Support/EmuSen` |
| a checkout (`EmuSen.sln` above it), on any platform | the checkout, as before |
| a marked tree (`.dianashroot` above it), on any platform | the tree, as before |
| none of those, on macOS | `~/Library/Application Support/EmuSen` |
| none of those, on Linux or Windows | `DianaShRoot/` beside the binary, as before |

Everything else follows from the root without a change of its own: `DataStore.UsrHome` is `<root>/home`, `ConfigStore.Directory` is `<root>/home/etc/EmuSen`, so saves, `games.db`, themes, media, shaders and the settings files all land under Application Support, and the DianaSh shell's `/` is `~/Library/Application Support/EmuSen/home`. The empty `<root>` level above `home` is kept rather than rooting at `EmuSen` itself, so that every platform has the same shape below the root and nothing that composes paths from `ConfigRoot` needs a Mac case.

**The bundle rule comes before the walk.** A bundle published into a checkout's `out/` would otherwise climb to `EmuSen.sln` and use the checkout as its home, which is the one outcome a bundle must not have. The checkout and the marker still win everywhere else, including on a Mac: `dotnet run` from source roots at the checkout, and a Linux-shaped tree published for `osx-*` with `-p:DianaShMacAppBundle=false` roots at its marker. The path is built as `Library/Application Support` under `SpecialFolder.UserProfile` rather than taken from another `SpecialFolder`, so that it does not depend on how .NET maps its Windows-named folders onto macOS.

**Testability.** `ConfigRoot.ComputeFor(baseDirectory, macOS, userHome)` takes the platform and the home directory as parameters, so `DianaShSandboxTests` runs every branch on a Linux host against temporary trees; the one-argument form passes `OperatingSystem.IsMacOS()` and the real home. Linux and Windows pass `false` and take exactly the old path, which `Off_macOS_a_bundle_shaped_path_keeps_the_old_fallback` pins.

**The manual is seeded, not read in place.** A Linux publish ships the EmuSen Manual and the hardware notes in `home/Documents`, inside the shell's root, so `cat`, `find` and `grep` reach them. The bundle carries the same tree at `Contents/Resources/home`, and `DataMigration.SeedFromBundle` copies it into the player's `home` each time Mistress starts, with `CopyTree`'s rule: copy, never overwrite, so a file the player already has always wins. Reading the docs straight from the bundle was the alternative and was rejected, for a reason specific to this program rather than a general one: the shell has one root and resolves every path inside it (`man hier`), so a second, read-only root inside the bundle would need a mount-like exception in `DianaShSandbox.TryResolve`, which is exactly the kind of hole the walled garden exists not to have. Copying costs about 6 MB once and one existence check per file (149 files) per start.

The cost of never overwriting is recorded rather than hidden: **an update does not refresh a document the player already has.** A page that is new in the update appears on the next start, and a page the player deleted comes back, but a page that changed keeps its old text until the player deletes it. Refreshing unedited pages would need a record of what was seeded (a version stamp and a hash per file); that is not built. The same trade was made for the legacy migration in §3.2, where it was the right one because the files were the player's; here they are the program's, and the question stays open.

No migration moves data from an earlier Mac build laid out as a Linux tree; that tree's data stays where it is, and the player copies it across if they want it.

---

## 4. `AtomicFile` — the durability contract

`ConfigFile<T>` has always written tmp-then-rename, so an interrupted save leaves the previous config intact rather than a truncated one (§2.3 of the config reference). Binary artifacts had nothing equivalent — save states and `.srm` files went through bare `File.WriteAllBytes`.

That was not a theoretical gap. **All three cores autosave cartridge SRAM every 300 frames — roughly every five seconds of play** (`VenusCore.Schedule.cs`, `MoonCore.Schedule.cs`, `MercuryCore.cs`). Every one of those writes was a window in which a crash, a kill, or a power loss would leave a half-written save. It is the one file a player genuinely cannot afford to lose, and it was the least protected thing the emulator wrote.

`AtomicFile` is deliberately **static, not a type to instantiate**. `ConfigFile<T>` earns its object-ness by carrying a category, a filename and migration state across many reads; a `.srm` write is "these bytes, to this already-known path, atomically." A handle type would be symmetry for its own sake.

```csharp
byte[]? TryRead(string path)          // null for missing or unreadable
bool    Write(string path, byte[] b)  // creates the directory, .tmp, rename
```


### 3.5 The names a tree published before the rename carries (2026-10-10)

Decided 2026-10-10: the shell DianaOS is renamed DianaSh, with its project, namespaces and folders, and its publish
writes `.dianashroot` where it wrote `.dianaosroot`. A tree already published carries the old marker, and a player's
data is inside that tree (`man hier`), so the walk honours both: at each directory it looks for `EmuSen.sln`, then
`.dianashroot`, then `.dianaosroot`, and stops at the first directory that has any of them. The fallback beside the
binary is `DianaShRoot/`, or `DianaOSRoot/` where that folder exists and the new one does not, so a player's data in
an old fallback is not left behind; where both exist, the new one is used. A tree published after the rename and
over an old one has both markers, which is harmless. The old names are
`ConfigRoot.LegacyRootMarkerFileName` and `ConfigRoot.LegacyPublishedRootDirName`, and the library's walk
(`Platform/galaxia/src/tree.rs`) does the same, which `GalaxiaParityTests` holds.

`DataMigration`'s legacy root, the tree a checkout kept its data in before the data moved to `<root>/home`, is renamed
with the folders it names, because it is the checkout's own tree and moved with them. No published tree ever held it.

### 3.6 The application-data folder in a home with no `.config` (2026-10-10)

**What was seen.** On 2026-10-10 a folder `EmuSen/Logs` appeared under the working folder of a test process whose home
had no `.config` folder. `EmuSen_RustPlatform.md` §15 had recorded the cause as an open item on 2026-10-08.

**The cause.** Three paths start from .NET's application-data folder: the error log's default folder
(`ErrorLog.DefaultRoot`, `~/.config/EmuSen/Logs` on Linux, `EmuSen_Settings_Reference.md` §4.70), the configuration
folder from before Galaxia (`ConfigStore.LegacyDirectory`, read once to migrate out of), and the value the host hands
the platform library at its start, from which the library makes the same two (`EmuSen_RustPlatform.md` §5.5, Q15).
All three asked `Environment.GetFolderPath(SpecialFolder.ApplicationData)`. On Unix that call, with its default
option, checks that the folder exists and answers the empty string when it does not, so each path became relative:
`EmuSen/Logs` and `EmuSen`, resolved against whatever folder the process was started in. Writing an error then made
the folder there. Under `dotnet test` that folder is the test host's, which in this repository is the checkout's
`home/`, and the log was written to `home/EmuSen/Logs/`.

**How it was shown.** `LogFolderWithoutConfigTests` starts a second test process with `HOME` pointed at an empty
temporary folder and `XDG_CONFIG_HOME` unset, since the library takes .NET's answer once per process and only a
process started that way shows what it is handed. The child moves to a temporary working folder, writes one error
through the library and one through the C#, and reports the default folder each uses, the files written and the
legacy folder. Before the change it reported `EmuSen/Logs`, `EmuSen/Logs/emusen_<day>.log` for both and `EmuSen`,
and the files were made under the working folder. After it, all are under `<home>/.config/EmuSen`, and nothing is made
under the working folder.

**The change.** One member, `ConfigRoot.ApplicationData`, asks for the folder with
`SpecialFolderOption.DoNotVerify`, which answers the path whether or not it exists yet; the three call sites read it.
The error log creates its folder when it first writes, as it already did for any other folder it was given, so a home
that has no `.config` gains one on the first error. The library
needed no change: when no host value is given it already computes `~/.config` without checking that it exists
(`tree.rs`, `application_data`), and the host now gives it the same.

**What was not changed.** The default stays `~/.config/EmuSen/Logs` rather than moving under `DataStore`'s home, since
§4.70 documents it and the handhelds' logs are read from there. A process with no home at all still gets a relative
path; .NET has nothing to answer with, and no such process was found.

### 3.7 `Unpacked`: archives opened for play (2026-10-11)

*Added at Diamond's stage 11, for the archives `EmuSen_CoreAPI_Extensions.md` §10.4 proposed and the tester approved
on 2026-10-11.*

`DataStore.Unpacked` is `<home>/Unpacked`, the twenty-first directory of the tree, `EMUSEN_GALAXIA_DIR_UNPACKED` (20)
in `emusen_platform.h` and `Directory::Unpacked` in `tree.rs`, so both halves name it and `GalaxiaParityTests` holds
them to one answer. *Until 2026-10-12* Galaxia named the folder and nothing else, and what went in it was the
runtime's archive component; since then the component is Galaxia's, `EmuSen.Galaxia/Media/ArchiveImages.cs` (§3.8).
It has no Rust half, since it reads `.zip` through .NET's own `System.IO.Compression` and nothing else in the program
reads one.

**The rule** [C]. A game in a `.zip` is unpacked once into `Unpacked/<key>/`, the key the first sixteen hex digits of
SHA-256 over the archive's full path, its length and its time, so that a changed or moved archive unpacks anew and
two archives of one name do not meet. It is unpacked into `<key>.partial-<process>` first and moved into place
whole, with `.unpacked` written last, so a folder without that file is never used. The image play opens, a cue sheet
first and else the first lone image of an extension a console takes, is named with the archive's own stem (an `.sbi`
of its stem too), so that its saves, states and records follow the archive's name as they would a loose file's; every
other entry keeps its name, as a sheet names it. An entry whose path would leave the folder is not written. **Nothing
is ever written beside the archive**, and the ROM library is only read.

**Clean-up.** After each unpacking the folder keeps the archives opened most recently, the one opened now always,
up to `ArchiveImages.KeptBytes` (4 GiB) in all; the least recently opened go first, a folder's time being moved on
each opening. A partial folder older than an hour, which only a process that stopped part way leaves, is removed.
~~A core holds its own copy of the disc from create, so removing a folder never takes a disc from a running game.~~
*Since 2026-10-12* a core may read its disc in place (§3.8), so a folder a game holds is never removed: `Prune` takes a
folder only after opening its marker alone, which a hold refuses, and deletes the marker first, so a folder caught part
way is never used (`Unpack` clears a folder without its marker and unpacks it anew).

### 3.8 A disc's files: sets, archives and mappings (2026-10-12)

*Decided 2026-10-12 with the core ABI's minor 1.3 (`EmuSen_CoreAPI_Extensions.md` §10.2.3): Galaxia owns where a
disc's files are and keeping them available; `emusen-disc` owns what a disc is (`EmuSen_Disc.md` §9); Crystal keeps
which system a disc is for.* Moved from the runtime's media module (`EmuSen.Crystal/Media/`) with their rules
unchanged: `ImageSets` (a sheet's companions, found as written else part by part ignoring case; an `.m3u`'s discs; the
`.sbi` beside an image; the shelf's rule) and `ArchiveImages` (§3.7). `ImageSets` asks the platform library's
`emusen_disc_*` calls what a list means, so Galaxia uses the disc crate's knowledge and the crate knows nothing of
Galaxia. `MediaProbe` and `DiscSectors` stayed in Crystal: they serve routing.

| Type | What it is |
|---|---|
| `MappedFile` | a file mapped whole and read-only: its address and length, `Span` and `Head` for a part, released when disposed. Opening reads none of it; the system reads each page from the medium when it is first touched |
| `DiscFiles` | a disc's image and companions (a sheet's tracks, then an `.sbi`), each a `MappedFile`, with a hold on the unpacked folder they are in; one `Dispose` lets all go |
| `DiscSet` | a library file's discs: an `.m3u`'s in order, else the one image, an archive unpacked; `Open(n)` maps disc *n*, unpacking again where the folder has gone since |
| `ArchiveImages.Hold(path)` | the hold `DiscFiles` takes: the folder's marker opened sharing reading, which `Prune`'s opening of it alone refuses |

**Both halves.** `MappedFile` is in C# over `System.IO.MemoryMappedFiles` and in Rust over the `memmap2` crate
(`Platform/galaxia/src/mapped.rs`; `emusen_galaxia_map`, `_mapped_bytes` and `_unmap`, interface 9), chosen by the
switch as every other rule is. `GalaxiaParityTests.A_file_is_mapped_to_the_same_bytes_and_refused_with_the_same_exception_both_ways`
holds the two to the same bytes and the same exception for a missing file (`FileNotFoundException`), a missing folder
(`DirectoryNotFoundException`) and a folder (`UnauthorizedAccessException`); writing it found the library answering a
folder with the system's "no such device", and it now refuses one as .NET does. *Amended 2026-10-15:* Windows refuses
to open a folder at all, so the library's refusal was the system's "Access is denied" there; a folder is now refused
with the same words, "is a directory", on every system (`EmuSen_RustPlatform.md` §26.1). The conformance kit, which has
no C#, maps through the Rust module.

**The rule of keeping** [C]. A `DiscFiles` is kept for as long as a machine may read it and no longer: by the host's
`CoreMachine` while a core that borrows holds the disc, released at `free` or when the drive takes another disc; for a
core that copies, released as soon as create or the insert returns. Nothing of the program writes a file it maps.
Another program may: on Linux and macOS a mapped file can be renamed or deleted and stays readable, but one truncated
or rewritten under a running game is read as it is then, and a page past a truncation ends the process with the
system's signal for a mapped page that no longer exists; removing the card has the same effect, where a copied disc
outlived it. On Windows a mapped file cannot be deleted, renamed or written while the game runs; the player is told
it is in use, until the game is closed or its disc changed. Each half makes that so by holding the file open for the
mapping's life and sharing only reading, the C# by its `FileStream` with `FileShare.Read`, the library by its file
with `FILE_SHARE_READ`; *until 2026-10-15 the library closed its file once the view was made, and a view refuses
nothing, so on Windows this held for the C# half alone* (`EmuSen_RustPlatform.md` §26.2), and
`GalaxiaParityTests.A_held_mapping_lets_in_the_same_openers_both_ways` now holds the two alike. .NET's own advisory locks are not seen by the library's
mapping, as by any program that is not .NET, so the folder hold, and not a lock on each file, is what keeps pruning
away.

**The prefetch, built and removed** (`DiscFiles.Prefetch`, 2026-10-12). A page of a mapped file is read from the
medium when first touched, on the thread that touches it, which for a lent disc is the emulation thread; on the
handheld's card that held a frame of play 8–32 ms (`Diamond_Players.md` §12.8.9). 1.3's remedy was a background thread
of the lowest priority reading each of the disc's files once, in order, 256 KiB at a time, resting twice each read's
time. It passed the later first touches and not those of a game's first seconds, which it appeared to lengthen.
**It was removed with minor 1.4**: the drive now names where it is going and the disc's reader advises the system
there (§3.9), and under the 90 MiB/s cap that left no first touch in 30 s of four games, where the prefetch, raced by
the game's own reads, held a frame 27–195 ms in six runs of nine (`Diamond_Players.md` §12.8.10).

**Tests** (`DiscStreamingTests`): a held unpacked folder survives `Prune` with nothing kept and goes at the next prune
once let go; the engine holds a borrowing core's files for the machine's life and a copying core's not past create
(read from the system's list of the process's mappings); Change Disc lets the earlier disc go only once the core has
taken the next, and a refused disc leaves the earlier in place.

### 3.9 Reading ahead of a drive (2026-10-12)

*Decided 2026-10-12 with the core ABI's minor 1.4 (`EmuSen_CoreAPI.md` §6.25, `EmuSen_CoreAPI_Extensions.md`
§10.2.6): a core names where its drive is going, and the host has the system read there before the game does.*

- **`MappedFile.Advise(offset, length)`**: the system told that the range will be read soon, so that it reads it from
  the medium on its own threads before a game touches it. One call, which returns before the bytes arrive; the range
  is clipped to the file; false for an empty file, a range outside it, or a system that refuses. *Both halves*: the C#
  calls `madvise` (`MADV_WILLNEED`, 3) on the whole pages of the range on Linux and macOS and `PrefetchVirtualMemory`
  on Windows, over the view it maps; the library's `Mapped::advise` makes the same calls through `memmap2`
  (`emusen_galaxia_mapped_advise`, interface 10). What a read of the mapping returns never changes; only when a page
  reaches memory does.
- **One call reads no more than the device's largest request** [M, `EmuSen_CoreAPI_Extensions.md` §10.2.6]: 4 MiB on
  the desktop's drive and 512 KiB on the handheld's card, whatever was asked. So a reader asks in pieces.
- **`DiscFiles.ReadAhead(file, offset)`**: a place of one of the disc's files handed to the disc's reader, a thread of
  its own started at the first place and ended when the `DiscFiles` is disposed, which waits for its one call in
  flight so that no mapping is released under it. The call takes a lock and pulses a monitor; it reads nothing,
  allocates nothing and never waits on the reader. The reader advises from `ReadAheadBefore` (256 KiB) before the
  place to `ReadAheadPast` (2 MiB) past it, in pieces of `ReadAheadPiece` (128 KiB): the piece holding the place
  first, then onward, then the pieces before it. A newer place replaces an older one between two pieces.
  `Advised` counts the bytes the system took the advice for.
- **Who calls it**: `CoreEngine`, for each `EMUSEN_EVENT_MEDIA_SEEK` it drains after a frame, finding the drive's lent
  `DiscFiles` through `CoreMachine.LoanOf` and the file by the numbering create gave it (the image for
  `EMUSEN_FILE_IMAGE`, companion *n* for `EMUSEN_FILE_MEDIA + n`). A drive lent nothing, a file the call did not pass,
  and a byte outside the file are dropped.
- **Tests**: `GalaxiaParityTests.Advice_is_taken_for_the_same_ranges_both_ways_and_the_advised_bytes_reach_memory`;
  `DiscStreamingTests.A_place_a_drive_names_is_read_ahead_in_the_file_galaxia_holds_for_it` (the reader's ranges at a
  file's middle, start and end, nothing advised after disposal, the engine's routing and its drops) and
  `A_run_whose_host_reads_ahead_of_the_drive_is_the_run_whose_host_does_not` (Diamond on a synthetic sheet, 600
  frames, picture, sound and state equal whether the host acts, through the machine and through the engine); in Rust,
  `mapped.rs`'s `advice_is_taken_inside_the_file_and_changes_no_byte` and `tests/advise.rs`, which reads the page cache
  with `mincore`.

### 4.1 Why rename rather than write-in-place

`File.Move(temp, path, overwrite: true)` is atomic at the filesystem level: the destination is either the whole old file or the whole new one, never a partial write. The truncation risk moves onto the `.tmp` file, which nothing ever reads — `AtomicFileTests.An_abandoned_temp_file_never_becomes_the_live_file` pins that a leftover temp from a killed process is ignored.

Failure stays best-effort, exactly as it is for config: a save that cannot reach the disk reports through `ConfigDiagnostics` and returns false. It does not throw, because the call sites are a cartridge mid-frame and a UI button handler, and neither should take the program down over a full disk.

### 4.2 What the guarantee is, and what it is not (stated 2026-10-07)

Read from `AtomicFile.cs` when its rules were reproduced in Rust (`EmuSen_RustPlatform.md` §10.4), so that the copy would promise the same and no more:

- The directory is created; the bytes are written whole to `<path>.tmp` with `File.WriteAllBytes`; the temp file is renamed over the live one with `File.Move(temp, path, overwrite: true)`, which is `rename(2)` on Linux and macOS.
- **So it is atomic against an interrupted write.** A reader, or the next start after a crash or a kill, sees the whole old file or the whole new one. That is the window §4 describes, and it is closed.
- **It is not a promise against power loss.** Nothing is flushed: neither the temp file nor its directory is `fsync`ed. After a power cut a filesystem may hold the rename and not the data, and what it does then is the filesystem's choice (ext4's `auto_da_alloc`, on by default, writes out a file that is renamed over another, as its documentation describes; nothing here relies on that, and no filesystem was tested). §4's phrase "a crash, a kill, or a power loss" named the three ways a write is interrupted; the mechanism answers the first two.
- A write that fails after the temp file was written leaves the temp file behind (a path that is a directory is the case the tests hold). Nothing reads a temp file, and the next successful write of that name replaces it.

Adding an `fsync` would be a behaviour change with a cost on the path that runs every five seconds of play, on a handheld's storage, and is recorded as a decision not taken rather than an oversight. It would be made in the C# and the Rust together, with a measurement of what the autosave then costs.

---

## 5. `SaveLibrary` — what the files are called

One spelling of the naming rules, previously duplicated across two frontends and computed three different ways by three cores.

```csharp
string SramPathFor(romPath, console)                   // Saves/<console>/<stem>.srm, since 2026-09-28
string StatePathFor(romPath, slot, directoryOverride)  // Save States/<stem>[.slotN].state
```

*Revised 2026-09-28: the save is in a folder per console, named as the ROM library names them, and the flat
`Saves/<stem>.srm` of earlier builds is copied in once (`EmuSen_Settings_Reference.md` §4.85.11).*

### 5.0 Save media kept apart from the game (2026-10-15)

A console's shared and named save media are files of their own beside its per-game saves:
`Saves/<console>/Cards/<name><ext>` for a removable medium and `Saves/<console>/Console/<name><ext>` for the console's own memory
(`SaveLibrary.MediumPathFor`, `MediumNames`; `CardPathFor` and `CardNames` are the removable kind's). `MediumLock` holds
`<file>.lock` beside a medium's file, opened exclusively while a game or the card manager has it, so that two programs
never write one card (`EmuSen_SaveMedia.md` §3.3, §3.5).

### 5.1 Two decisions worth knowing

**Slot 1 is the unsuffixed name.** `<stem>.state`, not `<stem>.slot1.state` — that is what Hotaru, Mistress and DianaSh's `state save` have always written, and changing it would orphan every existing save state. Slots 2 and up take `.slotN`.

**The directory override is a parameter, not a read.** `StatePathFor` could have consulted `AppSettings.StateDirectory` itself, since `AppSettings` lives in Galaxia too. It does not, because that would give path resolution a dependency on settings load order — a `ConfigFile` read on a code path that a cartridge constructor reaches. Mistress passes its Preferences value in; everything else passes nothing and gets the default.

### 5.2 The resume state and a state's picture (2026-09-21)

```csharp
string ResumeStatePathFor(romPath, directoryOverride)  // Save States/<stem>.resume.state
string PicturePathFor(statePath)                       // the same name with .png
```

**The resume state is not a slot.** It is written when a game is left (closed, replaced by another, or the window shut) and offered back at that game's next start (`EmuSen_Settings_Reference.md` §4.31). It takes a name no slot can take, so no manual save overwrites it and no automatic one overwrites a manual save. A Reset does not write it, because a Reset is not leaving the game.

**Every state Mistress writes gets a picture beside it**: the frame on screen when it was written, as a PNG named by `PicturePathFor`. The picture is taken on the emulation thread in the same request as the state, so it shows the instant the state holds rather than a later frame; a core that sends each scanline once (`EmuSen_Multicore.md` §15) has its rows repeated back to the displayed height before writing. The picture is a sidecar and never read by a core: a state without one loads exactly as before, and deleting one loses nothing but the thumbnail.

**What this does not cover.** The state still carries no record of which core version wrote it; OpenEmu's per-state plist does, and that remains stage 2's second half in `EmuSen_Mistress_LibraryPlan.md`. States written by the DianaSh `state save` command get no picture. And the resume state goes wherever `StateDirectory` points, which on this machine is the ROM folder itself; that is a local setting and not changed here, but it means leaving a game now writes two files there.

### 5.3 A state's record (2026-09-21)

`StateRecord` is a JSON file beside each state Mistress writes, named by `StateRecord.PathFor` (the state's name with `.json`): the console, the core's name, the state version the core wrote (`EmuSen_Save_States.md` §6), the build (the assembly's informational version, which carries the commit when the SDK knows it), when it was written, and the ROM's file name, size and MD5. It is the half of OpenEmu's per-state plist that the plan's stage 2 wanted, and it is a sidecar for the same reason the picture is (§5.2): a state with no record loads exactly as before, a record that will not parse is treated as absent, and no core ever reads one.

The record's ROM hash is whatever was known when the state was written; the hash is taken on a worker when a game starts, so a state saved in the first instant of a very large ROM's session can have none, and is then never questioned about which copy it came from.

*Revised 2026-09-26 by §5.3a: the record is no longer a file beside the state. The paragraph above is kept as the design the record was first built to; the fields and the sidecar argument are unchanged, the place is not.*

### 5.3a The record moves into records.db (2026-09-26)

`StateRecord` is still Galaxia's type, and Galaxia still has no database driver (§7.1). What changed is who stores it: Mistress keeps each record as a row of `state_record` in `home/Library/records.db` (`EmuSen.Mistress/Library/FileRecords.cs`), keyed by the state's full path and stamped with the state file's size and modification time, and returns it only while the file still matches. `StateRecord.Write` was removed, so nothing writes a sidecar any more; `StateRecord.ReadSidecar` and `SidecarPathFor` remain, for the import.

The sidecars users already have are **imported on first sight and never removed**: a state with no row whose sidecar parses gets a row from it, and the sidecar is left, both because the state folder on the author's machine is the ROM library, which this project never writes, and because a source left in place makes an interrupted import lose nothing. The argument, the staleness check and what each costs are in `EmuSen_Settings_Reference.md` §4.64.

The sidecar argument of §5.3 survives the move in one respect and not in another. It survives in that a state with no record still loads exactly as before and no core ever reads one. It does not survive in that a record no longer travels with its state: a states folder moved or copied elsewhere arrives without records, and fails towards "no record" rather than towards a wrong one.

### 5.3b The record names the engine, not the core (2026-10-04)

Until 2026-10-04 the record's `Core` held `ICore.CoreName`, which names the console's core family rather than the
implementation: C# Venus wrote `SNES`, and VenusRT on the generic adapter `VenusRT`. With two engines selectable for
one console, that told a reader neither which engine had written a state nor what its version number counts. Mistress
now writes the engine's catalog name, `CoreFactory.Running` (`Venus (C#)`, `VenusRT (Rust)`, `MarsRT (Rust)` and so
on), the same name the engine notice uses.

The version check that refuses "a newer build's" state compares a number with the running engine's own, so it applies
only when the record's engine is the running one. Where the record names another engine of the same console, Mistress
offers the state to the core, which either reads it (MoonRT, MercuryRT and MarsRT read their C# cores' formats) or
refuses it in its own words. VenusRT's words for a C# Venus state name Venus (C#) (`VenusRT_Native.md` §41.12). The
record's provenance line is added either way. A record written before this change holds a core name that is no
engine's, and is judged as before. That is the one case still judged by the old rule, and it errs as before: a
pre-change VenusRT record read by C# Venus is called a newer build's.

### 5.4 A ROM's identity (2026-09-21)

`RomHash.Md5` is the MD5 of every byte of the file, header included. That is a choice with a stated cost. The No-Intro and screenscraper conventions hash some consoles without their copier header, and OpenEmu hashes the whole file at import and the headerless file at lookup, which the library plan's §2 calls a trap. Here the hash has one job, to recognise the same file after a rename, and for that job the whole file is the right thing to hash: two files differing only in a header are different files to the core that loads them. A lookup against an external database would need the headerless hash as well, and would add it rather than replace this one.

---

## 6. What this pass deliberately did not do

Two things were separated out rather than bundled in, so each stands on its own review:

- **Moon and Mercury still write `.srm` beside the ROM.** Venus writes to `Saves/`; the other two use `Path.ChangeExtension(romPath, ".srm")`, which puts emulator output inside the player's own ROM folder. They gained the durable write in this pass and nothing else. Relocating them is a user-visible behavior change and needs a copy-don't-move migration in the shape of `ConfigFile<T>.MigrateFromLegacy`, read-only on `Games/`.
  *Done 2026-09-28: both, and their Rust engines, save in `Saves/` through one `BatterySave`, and a save beside the ROM is copied once, never moved (`EmuSen_Settings_Reference.md` §4.85.6).*
- **`SaveLibrary` cannot enumerate.** It resolves names; it does not list which slots exist or when they were written. That is the piece which would let a launcher show a save list through LunaP without loading a core — worth building when a UI actually consumes it, not before.

---

## 7. The catalogue — an index of the library, which the librarian did not have

Galaxia's own csproj calls her "the one answer to where anything this program keeps on disk lives", and until 2026-08-08 that was true of everything except the largest thing on disk. There was no index of the ROM library at all. `RomBrowserWindow` walked the filesystem live on every open, and answering "which images use mapper 5" meant walking 3,536 files and re-parsing every header — which was done four separate times in one afternoon before anyone noticed it was a missing table rather than a slow question.

`ICatalogue` and `catalogue-schema.sql` live in `Library/Catalogue/`. One row per image: path, name, system, size, the board the **loader resolved** (not the one the header claimed — the two differ across 758 images here, see `Moon_Memory.md` §2.2), header trust, PRG/CHR sizes, and whether a board exists for it today.

**The catalogue is a cache and never an authority.** It is always safe to delete, it is rebuilt by one pass, and it is read-only with respect to the library it describes. `playable = 0` is a fact about this program, not about the image, which is why such rows are recorded rather than skipped: an image whose board is unimplemented is exactly the one a coverage question is about.

The first index recorded 393 images as unreadable, which was wrong and instructive. `Cartridge.FromImage` builds the board as part of loading, and an unimplemented board throws — so "damaged dump" and "board this program lacks" arrived as the same exception, and the images a coverage query most wants were catalogued with no board at all. `Cartridge.Describe` now parses the header without building a board, and `IsBoardImplemented` answers the other half separately. The real count of unreadable images is **15**.

### 7.1 Why Galaxia owns the schema and not the driver

Galaxia is a leaf on purpose: `EmuSen.DianaSh` references her and `EmuSen` references DianaSh, so anything she depended on upward would close a cycle, and the no-`PackageReference` half of that rule keeps her models plain data.

A database engine is not plain data. Taking `Microsoft.Data.Sqlite` here would put a native driver at the bottom of the stack, where every consumer ships it — including the emulator cores, which have no use for one. So the split is: **Galaxia owns the contract, the models and the schema; the driver lives above her and is injected.** She remains the answer to where things live and what shape they are in; she simply does not hold the engine.

This follows the split `EmuSen.Cauldron` already uses for `ICoreTelemetry`.

**The driver lives in `EmuSen`** (`Common/Catalogue/SqliteCatalogue.cs`), decided 2026-08-08. The reasoning is convergence rather than fit: every consumer that wants a catalogue — both frontends, `EmuSen.Pharaoh`, `EmuSen.WiseMan` — already references `EmuSen`, and no other assembly is common to all of them. The alternatives were all worse on inspection: `LunaP` is a themeable UI toolkit, `Endymion` is the SDL3 device layer, `Serenity` is neither, and a new assembly would have to justify a boundary against a project actively pruning them. `EmuSen` already carries package references (SkiaSharp, explicitly), so nothing about it forbade one.

The cost is stated plainly rather than hidden: the emulator core assembly now carries a native database driver it does not itself use. That is a real objection and it was overruled deliberately, on the grounds that no consumer of this project takes the cores without the application around them.

**The one thing that does not follow is the shell.** `EmuSen` references `EmuSen.DianaSh`, not the other way round, so DianaSh cannot construct a driver that lives here. Anything in the shell that wants a catalogue — or, when it moves, a cheat store — takes the interface it is handed and reports honestly when nobody handed it one. That is the same seam, applied in the one direction where convergence on `EmuSen` does not reach.

### 7.2 What stays JSON, and why that is not inconsistency

The user-authored configuration — `appsettings.json`, `keybindings.json`, `gamepadbindings.json` — is 570 bytes across three files, and it stays exactly as it is.

A database would be a straight downgrade there. Those files are edited by hand, they follow the XDG convention every other program on the machine follows, and a corrupted one is a five-second fix in an editor where a corrupted SQLite file is unrecoverable. The atomicity a database would buy is already provided by `AtomicFile` (§4).

The distinction is authorship, and it is the same one that governs the known-differences dictionary (`EmuSen_Debugging_Tools_Reference_v5.md` §3.49), whose schema is committed as SQL while its `.db` is a build artifact: **where a human is the author, human-readable wins; where the program is the author and the data has shape, a database wins.** Preferences are written by a person. A catalogue of 3,586 images is not.

## 8. Test coverage

In `EmuSen.WiseMan/Galaxia/`:

- **`DataStoreTests`** — every directory's location, that the shell forwards all six, and both override-separation properties.
- **`AtomicFileTests`** — directory creation, replacement, no temp left behind, an abandoned temp never read, round-trip, and that an unwritable path reports and returns false instead of throwing.
- **`SaveLibraryTests`** — both names, slot 1's unsuffixed form, blank-override fallback, and that the ROM's own folder never leaks into the answer.

And, unchanged and still green, `DianaSh/HierTests.cs` and `DianaSh/DianaShSandboxTests.cs` — see §2 for why their staying untouched is the point.
