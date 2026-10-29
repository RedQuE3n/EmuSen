# EmuSen — game packages, and a CHD tool for the library

*Revised 2026-10-28: where the CHD tool writes is the player's choice (§20), as the tester approved the same day. A
persistent output folder, `AppSettings.ChdOutputDirectory`, refused where it overlaps the library either way; and, for
one batch at a time, each CHD written beside its original in the library through a partial renamed into place only
after the three checks, never over a file already there. The library then shows such a game once, as its package,
while the tool's record verifies the CHD; without the record both are shown.*

*Revised 2026-10-25, an eighth time: step 7 is built (§19). `pkg make --self-contained` copies a game into its package,
the library's file kept as its source and never moved; `--zip` and `pkg zip` write a package as one zip, which EmuSen
opens by unpacking it once into `Packages`, every file's time kept to the tick; `pkg identity` and the *Game Package*
window's switch set what the scraper sends; a package carried to a stand-in machine, as its folder or its zip, opens
there in both runtimes with its saves, states and records, settings, art and patches. What ES-DE and RetroArch see of
one is written down in §6, and the tester's checks of both are left (§19.8). Step 8 stays deferred (§9 Q2).*

*Revised 2026-10-25, a seventh time: the stores step 5 left are built (§18), as the tester decided §14.8's Q30. A
packaged game's state records sit beside its states, its crop and engine in its `settings.json`, its art in its
`media/`; both runtimes, the media store and both scrapers read and write them there, a moved package keeps them, and
`pkg migrate` copies them in. Step 7 waits for the tester's go.*

*Revised 2026-10-25, a sixth time: a patched game keeps its own saves (§16), as the tester decided §15.9's Q35. A game
opened with its patch list applied has battery saves, save media and states of its own, named by the list's save key,
in both runtimes; the platform interface 27. And §14.8's Q34 decided otherwise than recommended (§17): a claimed file
that is another dump now plays as a plain game by its plain path, and only its package refuses.*

*Revised 2026-10-25, a fifth time: step 6 is built (§15). Soft patching: `emusen-patch` applies IPS, UPS and BPS, each
written from its published specification, now read; `pkg patch add` and `pkg patch rm`; both runtimes hand the core the
patched image and never write the game; the `patched` identity for a state's record and the scraper; *Game Package* in
Mistress. Step 7 waits for the tester's go.*

*Revised 2026-10-25, a fourth time: step 5 is built (§14). Linked packages for every console: the claim index in
Galaxia, saves, states and cheats with a package root in both runtimes, the scan's package rules, and `pkg make`,
`pkg ls`, `pkg check` and `pkg migrate` in `emusen.library`. Step 6 waits for the tester's go.*

*Revised 2026-10-25, a third time: step 4 is built (§13). Mistress's Compressed Discs window runs the CHD tool
through the platform library's four job calls, reached from Settings, the pad menu and big picture as Memory Cards
is. Step 5 waits for the tester's go.*

*Revised 2026-10-25, again: step 3 is built (§12). chdman is obtained, pinned and shipped as §4.9 now records; the CHD
tool, its module and the platform library's calls are built; the synthetic gate passed and the tester's PlayStation and
Sega CD folders were converted on the desktop. The handheld's run is what step 3 leaves (§12.7).*

*Revised 2026-10-25: the tester answered §9 (§9.0). Q1 and Q4–Q17 as recommended; Q2, no removal of originals for
now, so step 8 is deferred; Q3 changed, so that EmuSen ships chdman (§4.1, §4.9). Every citation of MAME and chdman
now names its licence and the documentation's version (§10).*

*Written 2026-10-16. A proposal: nothing in it is built. It answers the tester's request of the same day for a way to
store games compactly and keep everything that belongs to one game together, and asks the tester to choose before any
code is written. Claims about the code were read on `WiseMan` at `227dc34a`. The measurements were taken on the
development desktop on 2026-10-16 with the tool named. They read the ROM library and wrote only to a probe folder
under `~/.cache/emusen/`, which was removed once the figures were recorded here.*

Every claim carries one of the marks the project's plans use:

- **[M]** *measured*: run on 2026-10-16, or read from the code at the file and line named, with nothing executed.
- **[R]** *read* from a source, named.
- **[A]** *argued* from the code and the documents, with no measurement behind it.
- **[C]** *a choice*: a recommendation awaiting the tester (§9), or a rule this page proposes.
- **P1–P7** are *predictions*, numbered in §8.1 so that the step that builds each item can retire it.

Companion pages:

- `EmuSen_Galaxia.md`: §3.3a on the library, §3.7–§3.9 on archives, mappings and reading ahead, §5 on save names,
  §5.4 on a ROM's identity, §7.2 on what stays JSON.
- `EmuSen_CoreAPI.md`: §6.2 on create, §6.23 on image sets, §6.24 on `BORROW`, §6.25 on `MEDIA_SEEK`.
- `EmuSen_CoreAPI_Extensions.md`: §1.1 on the three forms, §10.2 on streaming reads, §10.4 on archives.
- `Hardware/Shared/EmuSen_Disc.md`, on how the project reads a CHD.
- `EmuSen_SaveMedia.md`, whose generic tool, command module and window this page copies in shape.
- `EmuSen_BigPicture.md` §5, on ScreenScraper and `media.db`.
- `EmuSen_Settings_Reference.md` §4.32, §4.37, §4.66.5 and §4.100, on `games.db` and its per-game fields.
- `EmuSen_Config_Reference.md` §3.4b, on `GameCheats`.
- `EmuSen_Surfaces.md` §6, on the stability rule each component keeps.

---

## 0. The result in brief

The tester first proposed a custom compressed container that games would be converted into on import. After
discussion two things were chosen instead (§1.2): **a CHD tool for disc games**, opt-in, and **game packages**: a
plain folder per game holding the game or a link to it, a JSON manifest, and everything EmuSen keeps for that game.

**No part needs a change to the core ABI.** A core receives bytes and nothing else (`EmuSen_CoreAPI.md` §6.2), so
every new rule below sits in the host, in Galaxia or in a tool [A].

| § | Part | Form | Owner of the fact | Surface change |
|---|---|---|---|---|
| 3 | The manifest's schema, and where a game's files are | an **extension** of Galaxia | Galaxia, which already owns where a disc's files are (`EmuSen_Galaxia.md` §3.8) | `Media/GamePackage` and its Rust half; `DataStore.Packages`; the platform library's interface gains its calls |
| 3.3 | A dump's identity: CRC32, MD5 and SHA-1, whole, transformed and per track | an **extension** of Galaxia's `RomHash` | Galaxia for the arithmetic; the system pack for the transform; `emusen-disc` for where a track lies | `RomHash` gains three hashes and per-track hashing; Mistress's `RomHashes` folds into it |
| 5.4 | Applying IPS, BPS and UPS patches | a **component of its own**: the crate `emusen-patch`, pure functions | `emusen-patch` | new, reached through the platform library |
| 5.4 | Patching at load | **inside the runtime** | the host's `CoreEngine` (`EmuSen/Cores/Native/CoreEngine.cs`) | none at the ABI |
| 4 | Converting to CHD and verifying it | a **module**: the DianaSh command module `emusen.library`, over a tool library shared with Mistress's window | the tool library; chdman writes the file; `emusen-disc` reads it back | a new module, a new window, and chdman shipped as a pinned third-party program (§4.9) |
| 3.6 | A package as one entry in the library | an **extension** of the scan's existing rule | Galaxia (`ImageSets.Shelved`) | a folder with a `package.json` is one entry |

The decisions the page asks for are in §9: four standing rules, each as its own question, then thirteen design choices,
each with a recommendation.

---

## 1. The problem, and the decision record

### 1.1 What was proposed

On 2026-10-16 the tester proposed a compressed container of EmuSen's own. Games would be converted into it when
imported, with art, saves and settings stored inside it. Two needs lie behind the proposal:

- **storage**: a PlayStation library is large. The tester's `psx` folder holds 16 GB [M, `du`];
- **order**: what belongs to one game is spread over six stores today (§3.5).

### 1.2 What was chosen

After discussion, the tester chose two things in place of the container:

1. **A library compression tool on CHD** for disc games. It is opt-in, and offered both as a DianaSh command and as a
   Mistress window.
   - It converts cue/bin sets and loose images to CHD.
   - It verifies each conversion by decompressing it and checking the result against the original's hashes.
   - It records the original dump's hashes, which ScreenScraper, RetroAchievements and the Redump and No-Intro
     databases identify games by.
   - It removes originals only when the player explicitly asks, and only after a successful verification. Otherwise
     the compressed copies go to a new folder.
2. **Game packages for every game**, cartridge or disc. A package is a plain folder, not a new binary format. It holds
   the game, a text manifest, the game's art, saves, states, cheats and per-game settings, and its **soft patches**.
   - A soft patch is an IPS, BPS or UPS file applied at load. The original is never modified, so its hashes still
     match.
   - A game with extra files, such as MSU-1 audio tracks or 64DD disks, becomes one entry.
   - A package works in one of two **modes**:
     - *linked*, the default: the manifest points at the ROM where it already is, checked by hash, and nothing is
       copied;
     - *self-contained*, opt-in: the ROM is copied into the package.
   - Any tool can read a package, and ES-DE can treat its folder as one game.

   Compression for cartridge ROMs was decided in outline:
   - zip is read everywhere, and is chosen per package;
   - 7z is slower to decompress on weak machines;
   - a compressed ROM loses the zero-copy borrow, so the host decompresses it first;
   - scraping and achievements hash the ROM inside an archive, never the archive itself. *Corrected 2026-10-16:* this holds for RetroAchievements, which hashes contents; ScreenScraper lists archives under their own hashes and keys a game by whichever file the library names (`EmuSen_BigPicture.md` §46.4), so for ScreenScraper the archive's own hash is a valid identity.

### 1.3 Why a container of EmuSen's own was not chosen

Five reasons. Each would have been enough alone.

1. **Lock-in.** EmuSen's vision includes working with ES-DE and RetroArch (`EmuSen_CoreAPI.md` §11, where both
   frontends' features are catalogued). No other program reads a format of EmuSen's own. A player who leaves EmuSen
   would hold a library that no other emulator, frontend or verifier can open, and would need EmuSen to get their games
   out [A].
2. **Identification breaks.** Every service the project uses, or plans to use, names a game by hashes of the
   original dump:
   - ScreenScraper's `jeuInfos.php` takes `crc`, `md5` and `sha1` with the size (`EmuSen_BigPicture.md` §5.1);
   - RetroAchievements hashes each console's content in a form of its own, such as the iNES file without its 16-byte
     header, the big-endian N64 image, or a PlayStation disc's boot executable and its name [R,
     docs.retroachievements.org, "Game Identification", read 2026-10-16];
   - Redump and No-Intro list the dump's files with their hashes.

   A container's own hash matches none of these, and no other tool could look inside it to compute the right ones [A].
3. **The library rule.** EmuSen never alters or deletes anything under `AppSettings.RomDirectory`
   (`EmuSen_Galaxia.md` §3.3, §3.3a). Converting on import would have to choose between two failures [A]:
   - keep each original, which more than doubles the storage the request set out to save; or
   - replace it, which is the deletion the rule forbids.
4. **The borrow path's speed.** Since the core ABI's minor 1.3, the host maps a disc read-only and lends it to the
   core (`EmuSen_CoreAPI.md` §6.24). That took a cold launch on the handheld from 4.7–7.1 s to 95–120 ms, and peak
   memory from 878–1,382 MiB to 53–71 MiB (`EmuSen_CoreAPI_Extensions.md` §10.2.5) [R].
   - A container that must be unpacked whole before play would bring back the read that 1.3 removed.
   - A CHD does not: it is a file of compressed hunks of eight sectors, decompressed one hunk at a time as the drive
     reads, from bytes the host can map and lend as they are (`EmuSen_Disc.md` §5, §9.1, §10.2) [R].
5. **Maintenance.** A format of the project's own needs a specification, a writer, a reader in Rust and in C#, a
   verifier, tests and a migration rule for every later version. CHD is maintained by MAME. The project already reads
   it through the `chd` crate, a dependency whose code is not read (`EmuSen_Disc.md` §1) [R].

### 1.4 What is already built, and what this page builds on

- **Archives.** A `.zip` is unpacked once into `home/Unpacked/<key>/` and read from there, never beside the archive
  (`EmuSen_Galaxia.md` §3.7).
  - Both load paths go through it: `CoreFactory.cs:120` for the C# path and `CoreEngine.LoadRom` through `DiscSet` for
    the native one [M].
  - **So zip is already read for cartridges and discs alike, on every engine.**
- **A disc's files.** `ImageSets`, `DiscSet`, `DiscFiles` and `MappedFile` map a disc's files and hold them for a
  machine's life (`EmuSen_Galaxia.md` §3.8). `MEDIA_SEEK` reads ahead of the drive (§3.9 there).
- **CHD.** `emusen-disc` reads CHD v5. For a CHD, the place it names for read-ahead is the first byte of the hunk
  holding the sector (`EmuSen_Disc.md` §10.2).
  - Two cores claim `.chd` today: Diamond (`src/v1.rs:92`) and Nephrite's Sega CD (`src/v1.rs:311`) [M].
  - Without a decoder in C#, a CHD is routed to its system by its folder's name (`GenesisSystems.cs:47`;
    `EmuSen_CoreAPI_Extensions.md` §12 Q11) [M].
- **Borrowing.** Only Diamond and Nephrite claim `BORROW` (`Diamond … src/v1.rs:221`, `Nephrite … src/v1.rs:327`) [M].
  - The cartridge cores reach the host through `NativeRtCore` and `PortEngine`, which read the whole file with
    `File.ReadAllBytes` (`NativeRtCore.cs:125`, `PortEngine.cs:200`) [M].
  - So no cartridge core borrows today, except a Genesis cartridge on Nephrite, which claims the bit for its whole
    library.
- **Identity.** There are two hashers today, a duplicated contract of the kind `EmuSen_Surfaces.md` §4.8 lists:
  - `RomHash.Md5` in Galaxia: whole-file MD5, with a Rust half (`EmuSen_Galaxia.md` §5.4; `RomHash.cs`) [M];
  - Mistress's own `RomHashes`: CRC32, MD5 and SHA-1 in one read, used by the scraper (`Scraping/RomHashes.cs:8`) [M].
- **The scraper hashes the path it is given** (`Scraper.cs:275`, `RomHashes.Of(path)`), so for a `.zip` it hashes the
  archive [M]. ES-DE does the same, and its guide advises unzipping single-file games for that reason (USERGUIDE,
  "Scraping process") [R]. §5.5 changes this for packages.
- **Per-game stores**, each with one owner: `SaveLibrary`, `GameCheats`, `games.db`, `records.db` and `media.db`. They
  are listed with their files in §3.5.
- **The shape of a library tool.** `EmuSen_SaveMedia.md` §6 already has it: a generic tool that needs no core, a
  DianaSh command module over it (`memcard`), and a Mistress window over the same tool. §4 copies that shape.

---

## 2. The form of each part

The project's rule is that a feature which would give a component a second job, couple two components, or bloat a
stable surface first takes one of three forms (`EmuSen_CoreAPI_Extensions.md` §1.1):

- an extension of an existing surface;
- a module or plug-in;
- a component of its own.

This section weighs each part against the three, and names the owner of each fact.

### 2.1 The manifest, and where a game's files are: an extension of Galaxia

**What it is.** A package answers "which files are this game's, and where are they". Galaxia already answers that
question for discs:

- a sheet's companions;
- an `.m3u`'s discs;
- an archive unpacked;
- a disc's files mapped and held (`EmuSen_Galaxia.md` §3.8).

A manifest is the same answer written down for any game. Its model is plain data, which Galaxia already holds (§1 of
that page). So the manifest's schema, reading it, writing it atomically (`AtomicFile`), and resolving a package to its
files are **Galaxia's**, in `EmuSen.Galaxia/Media/GamePackage.cs` and `Platform/galaxia/src/package.rs`. The parity
class holds the two halves to one answer, as for every other rule of Galaxia's (`EmuSen_RustPlatform.md` §13) [C].

**Not a second job** [A]. Galaxia's job is where a file lives and how it is read and written safely, not what is in
it (`EmuSen_Galaxia.md` §1). The manifest records where and what kind; it interprets no ROM byte. Whether a file is a
game of a console stays with Crystal's system packs. What a patch does stays with `emusen-patch` (§2.3).

**The alternatives, and why each loses**:

- *A component of its own* for packages would be a second owner of save paths, beside `SaveLibrary`, which is exactly
  the duplication the project's reuse rule forbids [A].
- *A DianaSh module* alone cannot serve it: the runtime and Mistress must resolve a package without the shell, and
  DianaSh runs without an emulator (`EmuSen_DianaSh.md` §0.1).

### 2.2 A dump's identity: an extension of `RomHash`

**One owner for the arithmetic.** `RomHash` gains:

- CRC32 and SHA-1, beside its MD5, all taken in one read;
- hashing of a byte range, which per-track hashing needs.

Mistress's `RomHashes` becomes a caller of it. `RomHash` is already a rule with a Rust half, so it is the place the
Rust module of §2.4 can reach too [C].

**Two facts it does not own**:

- *The transform* that makes a console's comparable form: the iNES file without its header, the SNES file without a
  copier header, the N64 image in big-endian order. This is console knowledge, and already sits beside each console's
  declaration as `OpenVgdbBytes` (`EmuSen_BigPicture.md` §5.2, step 2). It stays there. `RomHash` takes a transform
  as a parameter and knows no console [A].
- *Where a track lies in its file.* This is `emusen-disc`'s (`EmuSen_Disc.md` §10.1), which Galaxia already asks
  through the platform library for a sheet's meaning (`EmuSen_Galaxia.md` §3.8) [R].

### 2.3 Applying a patch: a component of its own, `emusen-patch`

IPS, BPS and UPS are formats with no owner in the project today. They have to sit somewhere, and three existing places
were weighed:

- **Not a core.** Every core would carry the same three parsers, against the reuse rule. And a core that patched its
  own image would need the patch file passed to it, which is a new kind of file at the ABI for a job the host can do
  before create [A].
- **Not Galaxia.** Applying a patch rewrites content, and Galaxia interprets no content (§2.1).
- **Not `emusen-disc`.** A cartridge patch is not a disc.

So the patcher is **a crate of its own**, `Platform/emusen-patch`. It is a set of pure functions, bytes in and bytes
out, with no file I/O, no clock and no state. That is the shape of SaveMedia's format modules
(`EmuSen_SaveMedia.md` §6.2), and it lets the runtime, the shell module and a test call it the same way. The platform
library reaches it under Rule B (`EmuSen_Surfaces.md` §6.1) [C].

**A name collision, noted so that it is not repeated.** The core ABI's `ROM_PATCHES` capability
(`EmuSen_CoreAPI.md` §6.12) carries the cheat engine's address, value and compare triples. It is unrelated to patch
files and is not used for them. This page writes *soft patch* or *patch file* throughout [A].

### 2.4 The CHD tool: a DianaSh command module over a shared tool library

- **The command.** The tool needs no running game, so it is a module in the shell's default set, as `memcard` is
  (`EmuSen_SaveMedia.md` §6.4). The module is `emusen.library` and its command is `chd`. §3.7's `pkg` command joins it
  later. The module is in Rust, as the modules of S5 are (`EmuSen_DianaSh.md` §13.11) [C].
- **The tool library.** The logic is in one tool library, `Platform/library-tool`, which runs a batch and records it.
  - The module calls it.
  - Mistress's window calls the same library through the platform library, so the window and the shell cannot
    disagree.
  - This is SaveMedia's split of tool, module and window (§6.3–§6.5 there).
- **chdman, the one stated external dependency.** The library does not write CHD itself. It runs MAME's `chdman` as a
  program and then reads its output with EmuSen's own reader (§4.3). §4.1 and §9 Q3 give the reasons.

### 2.5 Patching at load: inside the runtime

The host already reads or maps the image before create (`CoreEngine.LoadRom`). Applying the manifest's patches there,
before create, changes nothing at the ABI. It is the runtime's job of preparing what create is given, as finding
firmware already is (`EmuSen_CoreAPI.md` §6.2) [A].

### 2.6 What is deliberately not given a second job

- **`emusen-disc`** stays a reader. Writing a CHD would put a compressor, a FLAC encoder and a writer into a crate that
  every disc core links (`EmuSen_Surfaces.md` §6.2, Rule R) [A].
- **`ArchiveImages`** stays the one reader of zip. A package's zip goes through it, and no second unpacker is
  written [A].
- **`games.db`** stays the player's library: favourites, play time, collections and edits. §3.5 decides which of its
  per-game fields are settings, which move.

---

## 3. The package: layout, manifest and stores

### 3.1 Where packages live

**Never in the ROM library** [C]. Packages live in a new directory, `DataStore.Packages`, at `<home>/Packages/`: ~~the
twenty-second directory of the tree~~ the twenty-third, since `Cores` took the twenty-second on 2026-10-20, numbered 22
(`EMUSEN_GALAXIA_DIR_PACKAGES`), with a constant in `emusen_platform.h` and in `tree.rs` as `Unpacked` has
(`EmuSen_Galaxia.md` §3.7, §3.10; built 2026-10-25, §11.2).

**A player-chosen folder** may replace it, as `AppSettings.PackageDirectory`, so that a player can point ES-DE at it
(§6.1). That setting refuses any folder inside `AppSettings.RomDirectory`, since a package is written to, and the
library never is.

### 3.2 The layout

A package's folder is named as ES-DE names a folder it treats as one game: `<stem>.<extension>`, after the file that
starts the game. ES-DE then passes the file of the same name inside the folder to the emulator (ES-DE USERGUIDE,
"Directories interpreted as files") [R].

```
<home>/Packages/<console>/Super Metroid (USA).sfc/
  package.json                 the manifest (§3.3)
  Super Metroid (USA).sfc      the game: self-contained only (§3.4)
  Super Metroid (USA).bps      soft patches, named as RetroArch finds them (§5.4)
  saves/                       battery files and per-game save media, by SaveLibrary's names
  states/                      save states and their pictures
  media/<type>/                art, under ES-DE's type names: covers, screenshots, marquees, videos ...
  cheats.json                  the game's cheat list, GameCheats' format
  settings.json                per-game settings (§3.5)
  extra/                       companions a core takes beside the image (MSU-1 tracks, 64DD disks)
```

A disc game's launch file is its sheet, its CHD or its `.m3u`:

```
<home>/Packages/PlayStation/Rayman (USA).chd/       one disc, self-contained, converted (§4)
  package.json
  Rayman (USA).chd
  Rayman (USA).cue              the original sheet's text, kept so that the original is described exactly
<home>/Packages/PlayStation/Final Fantasy VII.m3u/  a set: its .m3u lists the discs, as ES-DE's example does
```

**Multi-file games are one entry.** The roles are those image sets already carry to a core:

- a sheet's tracks, as files numbered from `EMUSEN_FILE_MEDIA` (`EmuSen_CoreAPI.md` §6.23);
- an `.m3u`'s discs, offered through `MEDIA`;
- other companions, such as MSU-1's `.msu` and `-n.pcm` files or a 64DD disk.

**No core reads MSU-1 or 64DD today**: VenusRT's plan lists MSU-1 as not supported (`VenusRT_Plan.md`, line 354), and
`Mars_Rom.md` says "No 64DD disk images" [M]. So `extra/` is reserved in the schema now, and a core that takes such
files declares them when it is built [C].

### 3.3 The manifest, version 1

JSON, the project's format for files a person may edit (`EmuSen_Galaxia.md` §7.2), written by `AtomicFile`. An
example for a patched cartridge in linked mode:

```json
{
  "format": "emusen-package",
  "version": "1.0",
  "title": "Super Metroid (USA)",
  "system": "snes",
  "mode": "linked",
  "game": {
    "role": "image",
    "where": "library",
    "path": "SNES/Super Metroid (USA).sfc",
    "size": 3145728,
    "seen": { "size": 3145728, "mtime": "2023-08-03T10:12:00Z" }
  },
  "dump": {
    "file":  { "crc32": "…", "md5": "…", "sha1": "…" },
    "transformed": null
  },
  "companions": [],
  "patches": [
    { "file": "Super Metroid (USA).bps", "format": "bps", "sha1": "…",
      "source_crc32": "…", "target_crc32": "…" }
  ],
  "patched": { "patches_sha1": "…", "crc32": "…", "md5": "…", "sha1": "…" },
  "identity": "original",
  "container": null
}
```

**The fields**:

- `format` and `version`: the schema and its version.
  - A minor adds optional fields. A reader ignores a field it does not know and keeps it on write.
  - A major is refused with words, as a newer `games.db` is (`EmuSen_Settings_Reference.md` §4.32) [C].
- `system`: the ES-DE system name, `CoreCatalog.LibraryShelf.EsdeSystem`'s spelling (`EmuSen_BigPicture.md` §5.3).
  - It is stored because a CHD cannot yet be routed by content without a decoder in C#. Today it is routed by its
    folder's name, which a package's folder does not carry (§1.4).
  - The manifest's `system` is the routing answer for a package, ahead of the folder rule [C].
- `game.where`: where the game file is.
  - `library`: relative to `AppSettings.RomDirectory`, so that a library that moves as a whole stays found.
  - `package`: inside the package.
  - `compressed`: inside §4.2's folder.
  - `absolute`: anywhere else.
- `game.seen`: the size and modification time when the file was last hashed, so that a file is hashed again only when
  it changes. This is `scrape_file`'s rule (`EmuSen_BigPicture.md` §17.5) [R].
- `dump`: **the original dump's identity**, always the bytes as dumped, whatever form the game is now stored in.
  - `file` holds the whole file's hashes.
  - `transformed` holds the hashes of the system pack's comparable form, or `null` when the transform changes nothing.
  - For a disc, `dump` holds instead:
    - `tracks`: one entry per track file, with its name, size and the three hashes;
    - `sheet`: the sheet's own hashes and its file name;
    - `form`: one of `cue-bin`, `lone-track` or `iso` (§3.3.1).
- `companions`: the same shape as `game`, for files of the set.
- `patches`: in the order they are applied (§5.4).
  - For BPS and UPS, the source and target CRC32 come from the patch's own footer.
  - IPS carries none (§5.4), so its entry has `source_sha1`: the dump it was made for, recorded when it was added.
- `patched`: the hashes of the image the core is given after patching, keyed by `patches_sha1` (a hash of the list
  and the files' hashes), and recomputed only when that key changes. It is kept apart from `dump`; §5.5 says who reads
  which.
- `identity`: which identity the scraper sends, `original` or `patched` (§5.5, §9 Q11).
- `container`: for a game stored compressed, what holds it.
  - For a CHD, the program and version that wrote it, its codecs, and the two SHA-1s that `chdman info` reports
    ("SHA1" and "Data SHA1" [M, chdman 0.289]).
  - For a zip, the archive's own hashes and the entry's name.

#### 3.3.1 The hash conventions, recorded rather than chosen

The manifest records each convention side by side, so that each service is sent its own form and no form replaces
another. This continues `EmuSen_Galaxia.md` §5.4's rule: a lookup "would add [the headerless hash] rather than
replace" the whole-file one [R].

| Consumer | What it hashes | Field it reads |
|---|---|---|
| EmuSen's own identity (`games.db`, `StateRecord`) | the whole file, header included (`EmuSen_Galaxia.md` §5.4) | `dump.file`, or `patched` for a state made on a patched image (§5.5) |
| ScreenScraper | the file's CRC32, MD5 and SHA-1 with its size, then the transformed form after a 404 (`EmuSen_BigPicture.md` §5.2) | `dump.file`, then `dump.transformed` |
| ScreenScraper, discs | the file the library names, whole, at any size: the sheet for a cue game, the image for a lone `.bin` or `.iso`, the file itself for a `.chd`, the list for an `.m3u` (`EmuSen_BigPicture.md` §46.4, measured on 2026-10-16) | `dump.sheet` for a cue game, `dump.file` for a lone image, `container` for a CHD |
| RetroAchievements | a per-console form: the NES file without its header, the SNES file without a 512-byte header, the big-endian N64 image, the whole file for the Game Boy, Genesis and 32X; for the PlayStation the boot executable named by `SYSTEM.CNF` with its path, for the Sega CD the first 512 bytes of track 0 [R, "Game Identification"] | the cartridge forms are `dump.transformed`; the disc forms are computed from sectors, so they do not depend on the container (§5.5) |
| Redump | each track file as dumped (2,352-byte raw sectors) with the sheet [R, Redump's published datfiles; their wiki's page on the format could not be reached on 2026-10-16, so its field list is to be read again at stage 1's gate] | `dump.tracks`, `dump.sheet` |
| No-Intro | cartridge contents; community reports conflict on whether the NES datfiles are headered [R, the libretro forum threads cited in §10] | both `dump.file` and `dump.transformed` are kept, so either can be compared |

### 3.4 Linked and self-contained

**Linked, the default.**

- The manifest names the file where it is. Nothing is copied, and nothing is written in the library: no sidecar, no
  link, no rename.
- At open, the file's size and modification time are compared with `game.seen`. Only when they differ is the file
  hashed again and compared with `dump`:
  - a match updates `seen`;
  - a mismatch refuses the package with words that name both hashes, and offers to look for the dump in the library.
    *Decided 2026-10-25 (§14.8 Q34, built in §17):* only the package itself is refused. The changed file opened by its
    plain path opens as a plain game, with its plain saves, and the package is reported `changed`.
- That search finds the file the way `games.db`'s orphan pass does: by size first, then by hash
  (`EmuSen_Settings_Reference.md` §4.37) [R]. A file found elsewhere updates `path`.
- A missing file says the same, and the package's saves, states and art are untouched.

**Self-contained, opt-in.**

- The game is copied into the package, read from the library and never moved. The copy is checked against `dump`
  before the manifest's `game.where` changes to `package`. *Built 2026-10-25 (§19.1):* the game is first held to
  `dump` as an opening would hold it, then each file is copied with its time and the copy held to the SHA-1 of the
  bytes read from its original in the same run; the entries it was copied from stay in the manifest as `sources`.
- The library keeps its file. Removing it is the player's own business outside EmuSen, as it is today. *Since step 7
  (§19.2)* the library file stays claimed by the package while it is still the file copied, so that the game is listed
  once and has one save.
- **Zip is a choice per package** (§9 Q8). The game may be stored as a zip of the one ROM.
  - It is read through `ArchiveImages`, unpacked once into `Unpacked`, mapped, and lent where the core borrows (§5.2).
  - Its hashes stay those of the ROM inside, `dump.file`; the archive's go under `container`.
  - *Built otherwise at step 7 (§19.3, Q43):* the zip is of the whole package, one file that carries the game with its
    saves, states, settings, art and patches, unpacked once into `Packages` when it is opened; a game the library holds
    as a zip is copied in as that zip and read as above.

### 3.5 How a package relates to EmuSen's existing per-game stores

The rule is **migration, not duplication** [C]. Each fact keeps exactly one owner of its name and place.

- When a game becomes a package, each file below is **copied** into the package once.
- That is `DataMigration`'s rule: copy and never overwrite, so a destination that exists wins
  (`EmuSen_Galaxia.md` §3.2) [R].
- From then on EmuSen reads and writes only the package's copy. The old file is left exactly as it was, and is never
  deleted by the migration.

| Fact | Today's owner and place | In a package | The owner after |
|---|---|---|---|
| Battery save | `SaveLibrary.SramPathFor`, `Saves/<console>/<stem>.srm` (`EmuSen_Galaxia.md` §5) | `saves/<stem>.srm` | **`SaveLibrary`**, which gains a package root as a parameter, as its state directory already is one (§5.1 there) |
| A game's own save media (per-game cards, the `.brm`) | the runtime's *Per game* rule (`EmuSen_SaveMedia.md` §3.3) | `saves/` | `SaveLibrary` |
| Shared and named cards, console memory | `Saves/<console>/Cards`, `Console` | **not moved**: they are the console's, shared between games (`EmuSen_SaveMedia.md` §3.3) | unchanged |
| Save states and pictures | `StateDirectory`; on the author's machine that is the ROM folder itself (`EmuSen_Galaxia.md` §5.2), and its root holds 123 states, pictures and records [M, `ls`] | `states/` | `SaveLibrary.StatePathFor`, with the same parameter. **The copies in the library are read and never removed** |
| A state's record | ~~`records.db`~~ `states.db` since D8, keyed by the state's full path (`EmuSen_Galaxia.md` §5.3a) | ~~a row for the new path, copied from the old one~~ beside the state, its name with `.json` (§18.1) | the runtimes' `StateRecords` |
| The cheat list | `GameCheats.For`, `etc/EmuSen/cheats/<console>/<game>.json` (`EmuSen_Config_Reference.md` §3.4b) | `cheats.json` | `GameCheats`, which gains the same parameter |
| The game's crop | `games.db`, `crop_*` columns (`EmuSen_Settings_Reference.md` §4.100) | `settings.json`'s `Crop` (§18.2) | the package (§9 Q12), read by `GameRecords` |
| The game's engine | ~~`games.db`, `game_edit` field `altemulator` (§4.66.5 there)~~ since SR10 the runtime's `/etc/EmuSen/games/<system>/<stem>.json` | `settings.json`, that file moved whole (§18.2) | the package (§9 Q12), read by both runtimes |
| Favourite, play time, collections, edits | `games.db` (§4.32, §4.38 there) | **not moved**: these are the player's library, not the game | `games.db`, keyed by the package's path and its hash |
| Scraped metadata | `media.db` (`EmuSen_BigPicture.md` §17.5) | **not moved**: a cache keyed by hash | `media.db` |
| Art | `home/Media/<system>/<type>/<stem>` (`EmuSen_BigPicture.md` §5.6) | `media/<type>/<package's name>` (§18.3) | the media store, which reads a package's folder for a packaged game (§9 Q13) |
| Firmware | `home/Firmware` (`EmuSen_Firmware.md` §0) | **never in a package** (§9 Q4) | unchanged |

**One game, one save.** Once a package links a file, opening that file by its plain path, from DianaSh's `core`
command or a recent-games list, resolves to the package. Galaxia keeps an index of the paths packages claim, and both routes
reach the same save [C, §9 Q14]. Without that rule, a game opened two ways would write two saves.

### 3.6 Packages in the library

Mistress's scan walks `RomDirectory` for files (`RomLibrary.cs:70`) and folds a sheet's tracks into one entry through
`ImageSets.Shelved` [M]. It gains two rules:

- a folder holding a `package.json` is one entry, and its contents are never listed as games;
- a library file that a package claims is not shown a second time.

Big picture follows the same rule: a packaged game is one entry at its console's top, not inside the folder of its library file, and shows the package's art (`EmuSen_BigPicture.md` section 30.3a).

*Decided 2026-10-25 (§14.8 Q34):* a claimed file that is no longer the dump its package was made for still opens by its
plain path, as a plain game with its plain saves; opening its package is refused with both hashes named, and `pkg ls`,
`pkg check` and the game's *Game Package* window say the package is changed (§17).

The scan's roots become `RomDirectory`, `Packages` and §4.2's `Compressed`. Hashing for the claim uses the index, not
a read [C].

**Packages are made on request, not on scan** [C, §9 Q6]. Every game *can* be a package. But writing a folder and
hashing a file for each of the library's 9,000-odd files (4,166 NES and 1,930 Game Boy among them [M, `find`]) is a
cost nobody asked for at scan time. A package is made:

- by *Make Package* on a game;
- by `pkg make`;
- for a batch, by `pkg make <folder>`;
- automatically, by a conversion (§4.4).

### 3.7 The `pkg` command

Added to `emusen.library` in stage 5 (§7), with its page at `Lib/dianash/emusen.library/man/pkg.1`. *Built
2026-10-25 (§14.4)*; `pkg patch` *built at step 6 (§15.2)*; `--self-contained`, `--zip`, `pkg zip` and `pkg identity`
*built at step 7 (§19.4)*; `pkg wait` joins it as `chd wait` joined `chd`:

```
pkg make <game>... [--self-contained] [--zip]   a package for each game, linked unless asked
pkg ls [<console>]                              the packages, with mode and state (ok, moved, missing, changed)
pkg check <package>...                          the game and every file hashed against the manifest
pkg patch add <package> <file>                  a patch file copied in and appended to the list
pkg patch rm <package> <n>
pkg migrate <package>                           the game's existing stores copied in (§3.5)
pkg zip <package>...                            a self-contained package written as one zip beside it (§19)
pkg identity <package> original|patched         which identity the scraper sends (§5.5, §19)
```

---

## 4. The CHD tool

### 4.1 What writes the CHD, and why not a writer of the project's own

**chdman writes it** [C, §9 Q3]. The CHD v5 format has no published specification apart from MAME's source.

- What the project knows of its layout was measured from chdman's output and the `chd` crate's documented API, with
  neither source read (`EmuSen_Disc.md` §5, §8) [R].
- A compatible writer means writing the hunk map's compressed encoding and each CD codec's framing. Those are defined
  only by that source [A].
- The house rule forbids reading emulator source, and MAME's is that [R, the project's rules].
- The `chd` crate is a reader; its project says it has no plans to implement writing in its own chdman substitute
  [R, docs.rs `chd`, read 2026-10-16].

**What chdman offers** [R, docs.mamedev.org "chdman", read 2026-10-16]:

- `createcd`, `createdvd` and `createraw`;
- `extractcd`, with `--splitbin` for one file per track;
- `verify`, which checks a read-only CHD's integrity;
- `info`;
- codecs: `cdlz` (LZMA for CD data, zlib for subchannel), `cdzl` (zlib), `cdzs` (Zstandard) and `cdfl` (FLAC for
  audio, zlib for subchannel);
- `createcd`'s default codecs `cdlz,cdzl,cdfl`, and a default hunk of "eight sectors per hunk (19,584 bytes)";
- `-np` to limit compression threads.

**Where chdman comes from.** ~~The tool finds chdman on `PATH` or at `AppSettings.ChdmanPath`. It runs `chdman` with no
arguments and records its version line, as 0.289 printed [M]. It neither ships nor fetches it in the first stages
[C, §9 Q3]. With none found, the command says so, says where chdman is obtained (MAME's own releases, and a
distribution's package such as Fedora's `mame-tools`), and converts nothing.~~ *Decided 2026-10-25 by the tester
(§9 Q3): EmuSen ships chdman.* The published tree carries one pinned chdman beside the platform library, and the tool
runs that copy, after checking its SHA-256 against the pin, and records its version line, as 0.289 printed [M]. How
and where it is built or obtained for each platform is settled at step 3; §4.9 records the plan and its obligations.

**Credit.** CHD (Compressed Hunks of Data) is MAME's format, and chdman is MAME's tool for it. chdman's source file is
under the BSD-3-Clause licence, copyright Aaron Giles; MAME as a whole is distributed under the GNU General Public
License, version 2 or later, and most of its files under BSD-3-Clause [R, "The MAME License", MAME Documentation
0.289]. Both are compatible with EmuSen's GPL-3.0. What chdman does is read from MAME's own documentation, "chdman –
CHD (Compressed Hunks of Data) File Manager", MAME Documentation 0.289 (§10); MAME's source is not read.

### 4.2 What it converts, and where to

**Inputs**:

- A cue sheet and its tracks, any number of them.
- A lone raw track with no sheet: two-thirds of the tester's PlayStation library (`EmuSen_Disc.md` §4).
  - chdman reads sheets, not lone tracks, so the tool writes a one-line sheet into a scratch folder of its own.
  - **A sheet naming the library's file by its absolute path was refused by chdman 0.289**, which printed only "Fatal
    error occurred: 1". The same sheet naming a symbolic link beside it, in the scratch folder and pointing at the
    library file, was read [M]. The link is made in the scratch folder, never in the library.
- An `.iso`.
- A `.zip` holding any of these, unpacked through `ArchiveImages` first (`EmuSen_Galaxia.md` §3.7).
- An `.m3u`: each disc is converted, and a new `.m3u` naming the CHDs is written in the output.

**Not converted**: `.gdi` (no Dreamcast core yet), DVD images, and a CHD.

**Where the output goes** [C]: by default `<home>/Compressed/<the library-relative path, with .chd>`.

- This is `DataStore.Compressed`, ~~the twenty-third directory~~ the twenty-fourth, numbered 23
  (`EMUSEN_GALAXIA_DIR_COMPRESSED`; built 2026-10-25, §11.2).
- Keeping the library's relative path keeps the console folder's name, which is how a CHD is routed today (§1.4). It
  also lets a player see which CHD came from which original.
- A player-chosen folder may replace it. That setting refuses any folder inside `RomDirectory`, unless the player has
  chosen §4.5's replacement. *Built 2026-10-28 (§20.1): `AppSettings.ChdOutputDirectory`, refused where it lies inside
  the library or holds it; and, for one batch, each CHD beside its original in the library (§20.2), which writes the
  CHD alone and never replaces a file.*

### 4.3 Verification: three checks, two of them independent of chdman

A conversion counts as verified only when all three checks pass in the same run [C]:

1. **chdman's own.** `chdman verify -i <file>` decompresses every hunk and compares the overall SHA-1. It printed
   "Overall SHA1 verification successful!" on both measured discs [M]. Its documentation does not say which hashes it
   compares [R], so this check is necessary but not sufficient.
2. **EmuSen's reader, track by track.** `emusen-disc` opens the CHD through the `chd` crate, which is an implementation
   other than chdman's writer.
   - For each original track file, it reproduces the bytes that file held: the track's stored range, with a stored
     pregap, as `Disc::place` locates them (`EmuSen_Disc.md` §10).
   - It compares their CRC32, MD5 and SHA-1 with the hashes the tool took of the original before converting.
   - For an `.iso`, it compares the 2,048 user bytes of every sector.
   - This is the comparison `emusen-disc`'s own CHD test already makes, sector for sector, on synthetic sheets
     (`EmuSen_Disc.md` §5) [R].
3. **The table of contents.** Track count, modes, index points and lead-out equal the sheet's.

**The round trip was measured** [M, chdman 0.289, `sha1sum`], using `extractcd --splitbin` in place of check 2's
reader, which is not built:

| Disc | Form | Tracks | Byte-identical after the round trip |
|---|---|---|---|
| Rayman (USA) | cue sheet, 1 data and 50 audio tracks | 51 | **51 of 51** |
| Crash Bandicoot (USA) | lone raw track | 1 | **1 of 1** |

**The record.** The originals' hashes are taken in one read before chdman runs. They go into `dump` (§3.3), and the
CHD's own go into `container`. The record is a package manifest (§4.4), so the CHD tool and the packages share one
schema and one owner.

### 4.4 Each conversion makes a package

A converted disc gets a linked package whose `game` is the CHD and whose `dump` is the original [C].

- The record then has a home that every reader already knows.
- A game opened by the CHD's plain path reaches the record through the claim index (§3.5).
- The scraper sends the original's hashes without reading the original again (§5.5).
- A player who chooses self-contained conversions gets the CHD inside the package instead of in `Compressed`.

### 4.5 The originals: never removed by default

*Decided 2026-10-25 by the tester (§9 Q2): no removal for now. CHDs stay in `Compressed` beside the originals, and
step 8 is deferred until the tester revisits it after seeing the tool verify their discs. This section stays as the
design for that day; nothing in it is built.*

**The default leaves the library exactly as it was.** The CHD goes to `Compressed`, and nothing under `RomDirectory`
is written, renamed or removed [C]. Stage 3's gate checks this with a listing of the library before and after: every
path, size, time and hash unchanged.

**Removal is the one exception the tool could make to the library rule, and it is offered only as follows** [C, §9 Q2,
asked as a standing rule]:

- It is off by default, and is never a setting that persists. It is a choice made for one batch, called *Replace
  originals in the library*.
- Before anything is touched, the tool names every file it would remove, with its size, and the total freed, and asks
  for confirmation.
  - In the shell that is a typed `yes` at a prompt, refused without a terminal, so that no script removes originals.
  - There is no flag or environment variable that answers it.
- Only discs whose three checks (§4.3) all passed in that same run are eligible. A disc that failed any check, or
  whose source changed during the run (size or time compared before and after), is skipped and reported.
- The order is:
  1. The CHD is written to `Compressed` and verified.
  2. It is copied into the library beside the original, as `<stem>.chd.partial-<process>`, and renamed into place.
  3. Its SHA-1 is checked at that place.
  4. Only then do the original's files go to the **system's trash**: the freedesktop trash on Linux, the Trash on
     macOS, the Recycle Bin on Windows. They are not unlinked, so that a wrong removal can be undone.
- A track named by two sheets is not removed while the other sheet's disc still needs it.
- Linked packages never take part in removal, and the package tool has no removal at all.

**What makes removal safe enough to offer at all**: a CHD extracts back to byte-identical tracks (§4.3, measured on
both discs), and the manifest keeps the original's per-track hashes and sheet. So the original can be reconstructed and
checked against a Redump datfile after removal [A].

### 4.6 Failure handling and resumability

- **Writes are atomic.** chdman writes to `<out>.partial-<process>`. The file is renamed into place only after §4.3's
  three checks. A failed check deletes the partial and reports what failed. This is `AtomicFile`'s pattern applied to
  a file chdman writes [C].
- **Space is checked first.** A disc is started only when free space at the output is at least its original's size,
  measured, plus 64 MiB.
- **The source must not change.** Its size and time are taken before and after. A change discards the result.
- **chdman exits non-zero, or is killed.** The partial is deleted, and the next disc goes ahead. One bad disc does not
  stop a batch.
- **Resumability is the file system's** [C]. A disc counts as done when its package exists and its `container` names a
  CHD of the recorded SHA-1. A partial is never done. So a stopped batch run again converts only what is not done:
  - there is no job database to drift from the files;
  - a partial left by a dead process is removed after an hour, as `ArchiveImages` removes its own
    (`EmuSen_Galaxia.md` §3.7).
- **The log.** Each batch appends one line per disc to `Logs/chd_<day>.log`: in, out, sizes, codecs, the three checks,
  and the time.

### 4.7 Cost, measured on the desktop and estimated for the handheld

Measured with chdman 0.289 on the Ryzen 7 7700X (8 cores, 16 threads), from the NVMe drive, at `nice 19`. Times are
wall clock; CPU is user time [M, `/usr/bin/time`]:

| | Rayman (USA), 51 tracks | Crash Bandicoot (USA), 1 data track |
|---|---|---|
| Original | 617,209,488 B | 632,083,536 B |
| CHD, default codecs (`cdlz,cdzl,cdfl`) | 323,075,270 B, **52.3%** of the original | 464,161,384 B, **73.4%** |
| CHD, `cdzs,cdfl` / `cdzs,cdzl` | 325,037,434 B, 52.7% | 473,073,556 B, 74.8% |
| Create, `-np 16` | 8.2 s, 81.9 CPU-s | 5.7 s, 83.4 CPU-s |
| Create, `-np 1` | 56.9 s, 90.0 CPU-s | 54.0 s, 87.5 CPU-s |
| Create, zstd codecs, `-np 16` | 6.2 s, 84.4 CPU-s | 11.5 s, 172.4 CPU-s |
| `verify`, one thread (default codecs) | 6.6 s | 9.8 s |
| `verify`, one thread (zstd codecs) | 5.0 s | 5.4 s |
| `extractcd` | 5.0 s | 8.3 s |
| Peak resident memory | 363–378 MiB creating, 39 MiB verifying | 474–485 MiB creating, 39 MiB verifying |

Notes on the figures:

- `-np 1` still used more CPU time than wall time. chdman runs threads beyond its compressors (reading and hashing),
  so `-np` limits compression only [M].
- Reading the original once for its hashes cost 0.26 s warm (`sha1sum` of the 632 MB track) [M]. On the handheld's
  card that read is the card's speed: 88–92 MiB/s, so about 7 s for that track (`EmuSen_CoreAPI_Extensions.md`
  §10.2.1) [R].

**The tester's whole PlayStation folder (16 GB)** [A, scaled from the two discs at 82–88 CPU-s per 0.6 GB]: about 36
CPU-minutes to convert. That is about 4 minutes of wall clock on the desktop with every thread, plus check 2's read of
each CHD and each original.

**The handheld (Legion Go S, Ryzen Z1 Extreme, 8 cores and 16 threads)** [A, estimated, not measured]:

- *Method.* The desktop's CPU time is scaled by 1.5–2.5 for the handheld's lower sustained clocks under its power
  limit. Reading is bounded by the card's 88–92 MiB/s.
- *Result.* About 130–220 CPU-s per disc. A batch limited to half the threads (§9 Q9) would take roughly 20–40 s of
  wall clock per disc. A disc's reads alone take 7–14 s.
- Stage 3's gate replaces these estimates with a measured batch, and records the battery's drain and the device's
  temperature.

### 4.8 The codec choice per system

**The drive's demand is tiny next to decoding.** A PlayStation drive at double speed reads 150 sectors a second
(`EmuSen_CoreAPI_Extensions.md` §10.2.2) [R]. That is 18.75 hunks a second.

- `verify`'s single-thread rate on Crash Bandicoot was 67 MB/s of logical data with the default codecs, and 121 MB/s
  with zstd [M]. Of that time, SHA-1 is a small part: `sha1sum` read 632 MB in 0.26 s.
- So a hunk of 19,584 bytes decodes in about 0.29 ms (LZMA), against 0.16 ms (zstd).
- Sustained play therefore spends about 5.5 ms of decoding per second on LZMA: **0.55% of one desktop core**, and an
  estimated 0.8–1.4% on the handheld [A].
- A frame that crosses a hunk boundary gains about 0.3–0.7 ms. Diamond's emulation of a frame on the handheld takes
  2.7 ms (`EmuSen_CoreAPI_Extensions.md` §10.2.5, citing `Diamond_Players.md` §12.8.9) [R].

**Recommendation** [C, §9 Q7]:

- **chdman's defaults** (`cdlz,cdzl,cdfl`) for every CD system: smallest on both discs; chdman picks the best listed
  codec per hunk [R].
- `cdzs,cdfl` offered as *Faster to decode*. It decodes 1.3–1.8 times faster for 0.4–1.4 points more size. On Crash
  Bandicoot it took twice the CPU to create.
- The choice is recorded per CHD in `container`.
- **Whether the `chd` crate as `emusen-disc` builds it decodes `cdzs` is checked at stage 3's gate.** Its features list
  `fast_zstd` as optional and `ruzstd` as a dependency [M, its `Cargo.toml`]. Check 2 would catch a CHD it cannot read
  in any case.

**DVD systems and the GD-ROM** wait for a core that reads them.

### 4.9 Shipping chdman: the plan (decided 2026-10-25, built at step 3)

*Decided 2026-10-25 by the tester (§9 Q3), in place of the recommendation that the player supply chdman. Recorded
here as a plan; nothing is built or fetched before step 3's go.*

**Licences** [R, "The MAME License", MAME Documentation 0.289, read 2026-10-25; the tester's reading of chdman's file
header, 2026-10-25]. chdman's source file is BSD-3-Clause, copyright Aaron Giles. MAME as a whole is GPL-2.0-or-later.
EmuSen is GPL-3.0, with which both are compatible, so a chdman binary may be shipped beside EmuSen's own when the
obligations below are met.

**What shipping means**, each an item of step 3's gate:

1. **The licence texts**: a row for chdman in `THIRD_PARTY_NOTICES.md`, naming MAME, chdman, the release, both
   licences and the copyright holders, and the texts themselves in `licenses/` (BSD-3-Clause with Aaron Giles's
   notice, and GPL-2.0), as each shipped crate's are today.
2. **A written source offer** naming the exact MAME release tag the binary was built from (for 0.289 that tag is
   `mame0289` in MAME's public repository), as GPL-2.0's section 3 asks of a binary distributed without its source.
3. **The README's credits**: MAME and chdman under "Credits and references", with the licences and a link to MAME's
   documentation page for chdman.
4. **A pinned version, checked by SHA-256.** The release and the binary's SHA-256 for each platform are recorded in one
   place; the tool refuses a chdman whose hash differs, with words naming both hashes, and converts nothing.
5. **Built or obtained as a black box.** MAME's source is never read, not even to build it. A build follows MAME's
   documented build instructions only ("Compiling MAME", MAME Documentation, of the pinned release), and an obtained
   binary comes from MAME's own release or a distribution's package, chosen at step 3.

~~**Settled at step 3, not now**: which platforms get which binary (the desktop, the handheld's SteamOS, and later
Windows and macOS when they resume), whether it is built in CI or obtained, and where it sits in the published tree.
Each comes back to the tester as part of step 3's go.~~ **Settled at step 3 (2026-10-25)**, with the research and the
measurements in §12.1:

- **The release.** MAME 0.289, published 2026-07-30, is the latest: it is the newest release and the newest tag of
  MAME's repository on 2026-10-25, and MAME's release page offers it [R, §10].
- **The route: built, not obtained.** MAME publishes no Linux binary, only Windows ones and the source [R]. Fedora 44's
  `mame-tools` 0.289 is a distribution's own package, but its chdman links `libutf8proc.so.3` and `libSDL3.so.0`,
  and would not start on the desktop for want of the first [M]. So chdman is **built from MAME's source archive for
  the tag, `mame0289s.exe`, checked against the SHA-256 MAME publishes**, with the options MAME's "Compiling MAME"
  gives for the tools alone (`TOOLS=1 EMULATOR=0`). The documented prerequisites are installed with `dnf`, which
  needs root, so the build runs **in a throwaway Fedora 44 container** (rootless podman), where they are installed
  as documented and nothing is installed on the machine; no sudo is used. MAME's source is unpacked and built and
  never read. The recipe is `Platform/library-tool/build-chdman.sh`.
- **One binary for both machines.** Linking GCC's C++ library in (`ARCHOPTS="-static-libstdc++ -static-libgcc"`,
  and the distribution's `libstdc++-static` in the container) leaves a binary that needs glibc 2.38 or later and
  `libSDL2-2.0.so.0`: the desktop has glibc 2.43, and the handheld's SteamOS glibc 2.43 and SDL2 2.32.70, and the
  binary printed its version line on both [M]. Windows and macOS are paused (`chdman.pin` has no binary for them,
  and the tool says so and converts nothing there).
- **Where it sits**: beside the platform library, as `chdman` (`chdman.exe` on Windows), copied there by
  `Platform.targets` from `~/.cache/emusen/toolchains/chdman/<rid>/` where the recipe leaves it. It is not built in CI:
  the binary is made on the desktop, and its hash is the pin.
- **The pin**: `Platform/library-tool/chdman.pin`, the one place for the release, the tag, the source archive and its
  SHA-256, and each platform's binary's SHA-256. The binary is never committed.
- **The obligations**: 1, `THIRD_PARTY_NOTICES.md` §1.7 and `licenses/` (`chdman-BSD-3-Clause.txt`, `GPL-2.0.txt`,
  and the licences of the libraries the binary carries); 2, `licenses/chdman-SOURCE-OFFER.txt`, naming `mame0289`;
  3, the README's "Disc images: CHD and chdman", linking MAME's chdman page; 4, the pin checked before chdman is run,
  a mismatch refused with both hashes named (§12.4); 5, built as a black box, as above.

---

## 5. Loading

### 5.1 A disc in a package: CHD through the existing reader

The package resolves to its launch file. From there the path is today's [R, `EmuSen_Galaxia.md` §3.8;
`EmuSen_CoreAPI.md` §6.24, §6.25]:

1. `DiscSet` opens the file.
2. `MappedFile` maps the CHD whole and read-only.
3. A core that claims `BORROW` is lent the mapping.
4. `emusen-disc` decompresses one hunk at a time from the lent bytes as the drive reads (`EmuSen_Disc.md` §9.1).
5. `MEDIA_SEEK` names each hunk's first byte, so the host reads ahead in the compressed file (§10.2 there).

**A CHD keeps the borrow** [A]. What is lent is the compressed file, and the core's cache holds one decompressed hunk.
The launch keeps 1.3's shape: no whole read, and no second copy.

### 5.2 A cartridge in a zip

The zip goes through today's path, into `Unpacked`, and nothing new is written [C]:

- `ArchiveImages.Unpack` unpacks it once (`CoreFactory.cs:120`; `CoreEngine.LoadRom` through `DiscSet`) [M];
- the unpacked file is mapped like any other.

So after the first launch, a zipped ROM is read and lent exactly as a loose one is. The cost is the first launch's
inflate and the folder's space, kept under `ArchiveImages.KeptBytes` (4 GiB). A zip is never inflated into a second,
in-memory path.

**The loss of `BORROW`, and its cost.** The tester's outline expects a compressed ROM to lose the borrow, with the host
decompressing into memory. Measured against the code, that holds less widely than expected:

- **Today, no cartridge engine borrows**, apart from Nephrite's Genesis cartridges (§1.4) [M]. The cartridge engines
  copy the whole file already, so a zip loses them nothing at create.
- **Where a core does borrow**, the unpack cache keeps the borrow from the second launch on. Only the first launch pays
  the inflate.
- **Had the in-memory path been chosen**, the loss would be the copy at create. Disc images were measured copying at
  150–290 ms per 430–660 MB on the desktop (`EmuSen_CoreAPI_Extensions.md` §10.2.1) [R]. Scaled by size, a 64 MiB
  N64 cartridge, the largest cartridge the project runs, would take about 20–30 ms on the desktop and an estimated
  40–75 ms on the handheld [A, scaled linearly from the disc copy, with the handheld factor of §4.7]. That is small;
  the choice of the cache is made for one path rather than for speed.

**The inflate**, measured on the 32 MiB *Legend of Zelda: Majora's Mask (Europe)* image, one thread, warm, three runs
each [M, Info-ZIP `zip` 3.0 and `unzip` 6.00, `7z` 26.04, at `nice 19`]:

| Container | Size | Decompression |
|---|---|---|
| zip, deflate level 6 | 28,069,381 B, 83.7% | 0.16 s |
| 7z, LZMA2, level 7 | 24,602,337 B, 73.3% | 0.73 s |

A 1 MiB SNES image (*Family Dog (U)*) shrank to 32.0% as zip and 25.7% as 7z [M]. So 7z saves 6–10 points of size and
takes **4.6 times as long** to decode.

**7z is not read** [C, §9 Q5]. It would need a new dependency, since .NET has no 7z reader and `ArchiveImages` reads
zip through `System.IO.Compression` (`EmuSen_Galaxia.md` §3.7). It is slower where the project's weak-machine target
is weakest, and the format choice for cartridges was zip. The library does hold 13 `.7z` files (9 NES, 4 Game Boy)
[M, `find`]. They stay as they are; a package can link them only after the player unpacks them.

### 5.3 What a core is handed

Nothing about a package reaches a core. A core sees the image bytes and its files as create always gave them
(`EmuSen_CoreAPI.md` §6.2):

- the companions as `EMUSEN_FILE_MEDIA + n`;
- the battery files from the package's `saves/`.

The kit's cases are unchanged [A].

### 5.4 Soft patching in the host

**Where it happens** [C]. In `CoreEngine.LoadRom`, after the image is mapped and before create. `emusen-patch` applies
the manifest's patches in order to a host-owned buffer, and that buffer is the image create receives.

- For a core that claims `BORROW`, the buffer is native memory held by `CoreMachine` for the machine's life, as a
  `DiscFiles` is. The loan's promise is that the bytes stay readable, unchanged and in place until the loan ends,
  which a held buffer satisfies as a mapping does (`EmuSen_CoreAPI.md` §6.24) [R].
- For any other core, the buffer is released once create returns.

**The formats** [R, each format's documentation as cited in §10]:

- **IPS**: the header `PATCH`, then records of a 3-byte offset, a 2-byte size and that many bytes, or, for size 0, a
  run of one byte. The footer is `EOF`.
  - It has **no checksum**, so the manifest records the source's SHA-1 when the patch is added, and refuses a game
    whose dump differs (§3.3).
  - A 3-byte offset reaches only the first 16 MiB.
- **UPS**: a format by byuu that records differences as XOR with the source, and carries CRC32s of the input, the
  output and the patch.
- **BPS**: Near's format of 2012 (romhacking.net, document 746, version 1.0). Four actions: copy from the source at
  the same place, take bytes from the patch, copy from the source at a relative place, or copy from the target already
  written. It carries CRC32s of the source, the target and the patch.
  - The CRC fields' order and the variable-length number encoding are to be read from the specification itself at
    stage 6. *Read at step 6 (§15.1): source, target, then the patch's own CRC-32, little-endian.* On 2026-10-16 only secondary descriptions were reachable: vidua's notes, and RetroArch's implementation
    named in a search.

**The checks** [C]:

- A BPS or UPS patch whose source CRC32 differs from the image is refused, with both values in the words.
- Where the system pack's transform yields the patch's source (a patch made for a headerless NES image applied to a
  headered file, say), the refusal says so and offers the transformed form. It is never applied silently.
- The target CRC32 is checked after applying, and a mismatch refuses the launch.
- A patch that would write past 64 MiB is refused.

**RetroArch's convention, kept for interoperability.** RetroArch applies `<stem>.ups`, `<stem>.bps` or `<stem>.ips`
found beside the content. Several IPS patches are named `.ips`, `.ips1`, `.ips2` and applied in that order. This
happens only with cores that load content from memory [R, docs.libretro.com "Softpatching"]. RetroArch also has
`--ips`, `--bps` and `--ups` to name a patch, and `--no-patch` [R, the `retroarch(6)` manual].

So a package's patch files sit **in its root**, named as RetroArch finds them, whenever the manifest's list fits that
convention: one BPS or one UPS, or IPS patches in order [C, §9 Q10]. A list that does not fit keeps its own names, and
the package says it will not patch under RetroArch.

**Discs** [C, §9 Q15]. Disc patches are mostly PPF or xdelta and apply to a track's raw bytes. Patching a CHD would
need an overlay of patched sectors inside `emusen-disc`, which is a reader. They are deferred to a later page.

### 5.5 Keeping the original's identity apart from the patched image's

- **`dump` never changes** after the package is made. It is the original as dumped, whatever its storage form. A CHD's
  `dump` is the original tracks, never the CHD file.
- **`patched`** is the image the core runs, recomputed when the patch list changes.
- **Who reads which** [C]:

| Reader | Unpatched package | Patched package |
|---|---|---|
| The scraper | `dump.file`, then `dump.transformed` (`EmuSen_BigPicture.md` §5.2) | `dump` by default (`identity: original`). With `identity: patched`, `patched` first and `dump` after a 404. A KO costs from the smaller daily allowance (§5.2 there), so this is chosen per package, not tried for every package |
| RetroAchievements (when built: `EmuSen_CoreAPI.md` §6.22 keeps the door open) | the console's form, from `dump` | the hash RetroAchievements lists for the patched game, taken from `patched` in the console's form. The hardcore policy is the runtime's (§6.22 there) |
| `StateRecord`'s ROM hash | `dump.file` | `patched`'s MD5, since a state is made on the bytes the core ran. A state made on another patch list is then "a different copy", which is asked about (`EmuSen_Settings_Reference.md` §4.37) [R] |
| `games.db`'s identity | `dump.file` | `dump.file`, so that adding a patch keeps the game's record |
| Battery saves, the game's own save media, states | `<package>/saves/<name>…`, `<package>/states/<name>…` (§14.2) | the same folders as `<name>.patched-<key>…`, `<key>` the list's save key (§16) |

- **A patched game keeps its saves apart** [C, decided 2026-10-25 by the tester, §9 Q35 in §15.9]. A game opened with
  its patch list applied reaches battery saves, its own save media and save states of its own, never the original's,
  so that a hack can never write the original game's save. Each list has its own, the same list always the same ones,
  and a package whose every patch is removed returns to the original's. The names are §16's.
- **Disc identity is independent of the container** [R]. RetroAchievements' disc hashes are computed from sector
  contents, so they come out the same from a cue/bin and from its CHD whenever they are computed through
  `emusen-disc`'s sectors:
  - for the PlayStation, the boot executable named in `SYSTEM.CNF`;
  - for the Sega CD, the first 512 bytes of track 0.
- ScreenScraper's rule for discs waits on `scrape-discs` (§3.3.1). Whatever that branch decides, the package already
  holds every hash it could ask for.

---

## 6. Interoperability

### 6.1 ES-DE

- **A self-contained package is one game to ES-DE**, unchanged. ES-DE treats a folder named with a supported extension
  as one entry, and passes the file of the same name inside it to the emulator (USERGUIDE, "Directories interpreted
  as files", with its own `Final Fantasy VII.m3u/` example of CHDs) [R].
  - ES-DE is not expected to read `package.json`, `saves/` or `states/`; it is shown only the folder [A].
  - A player who points an ES-DE system at `Packages/<console>` sees the packages as games.
- **A linked package has nothing for ES-DE to launch**, because its game is elsewhere. Two answers [C, §9 Q16]:
  - a symbolic link named as the launch file, made inside the package (never in the library), on Linux and macOS
    where links need no privilege, opt-in;
  - or *Make Self-Contained* for the games a player wants in ES-DE.
- **Patches.** ES-DE's own instructions for RetroArch ROM hacks are this layout: a folder named as the game, holding
  the game and its `.ips` (USERGUIDE, "RetroArch ROM hacks") [R]. A package with RetroArch-named patches (§5.4) is
  that example.
- **Media.** ES-DE reads its `downloaded_media/<system>/<type>/` tree, not a package's folder. A later `pkg export
  esde` could copy a package's `media/` there, since the type names are already ES-DE's (§3.2). It is not proposed for
  the first stages.
- **What ES-DE is shown of a self-contained package, as built at step 7 (§19.5)** [A, from the USERGUIDE sections
  cited above; not run]:
  - The folder `Packages/<console>/<name>.<ext>/` holds the file it is named after: a cartridge's copy is named as the
    folder (§19.1), a disc's sheet and a converted disc's CHD give the folder its name. So a folder ES-DE takes as one
    game is passed to the emulator as that file, the sheet with its tracks beside it, the CHD with the original sheet
    beside it under its own name, which ES-DE is not given.
  - Everything else in the folder (`package.json`, `saves/`, `states/`, `media/`, `settings.json`, `cheats.json`) is
    inside a folder ES-DE does not list.
  - The console folders under `Packages` are named by EmuSen's consoles (`SNES`, `PlayStation`), not ES-DE's system
    names (`snes`, `psx`), so a player points an ES-DE system at `Packages/<console>` rather than at `Packages` as a
    ROM directory.
  - A package's zip is a file of EmuSen's for carrying a package; ES-DE would take it as a game's zip holding many
    files, and it is not offered to ES-DE.
  - *Left for the tester (§19.8):* ES-DE launching a self-contained package.

### 6.2 RetroArch

- RetroArch loads a package's launch file as any content.
- It applies RetroArch-named patches beside it, for cores that load from memory [R].
- It reads CHD through the cores that support it. The `chd` crate, which `emusen-disc` uses, describes itself as
  compatible with libchdr [R, docs.rs `chd`], the reader libretro cores commonly use.
- Saves are not shared: RetroArch keeps its own save folder. A package's `.srm` is a plain battery file that a player
  can copy across.
- **What RetroArch is shown of a self-contained package, as built at step 7 (§19.5)** [A, from docs.libretro.com
  "Softpatching" as cited; not run]: the game file in the folder, named as the folder, and beside it the package's
  patches under the package's own name (`<name>.bps`, `<name>.ips`, `.ips1`, …), which is the game file's stem, so that
  RetroArch's `<stem>.<ext>` rule finds them for a core that loads from memory. A list that does not fit the
  convention keeps its own names and says so (§15.2). A patched game's saves are EmuSen's, named by the list's key
  (§16), and RetroArch keeps its own. *Left for the tester (§19.8):* RetroArch applying a package's patch.

### 6.3 What a player has if they leave EmuSen

| Kept | In what form |
|---|---|
| The ROM library | **exactly as it was**: linked packages never touched it, and the CHD tool wrote nothing there unless the player chose replacement (§4.5) |
| Converted discs | standard CHD files. `chdman extractcd --splitbin` gives back byte-identical tracks (§4.3, measured), and `dump` lists the hashes to check them against |
| Self-contained games | the ROM as a plain file in its package's folder, named as the folder; or the whole package as one zip, whose top folder any unzip gives back (§19.3) |
| Patches, saves, art, cheats, settings | ordinary IPS/BPS/UPS files, `.srm` files, PNG and JPEG, and JSON |
| Save states | EmuSen's own format, readable only by the engine that wrote it (`EmuSen_Save_States.md`, `EmuSen_RustState.md`). That is true today and is not changed here |

---

## 7. Phasing

Smallest useful step first, each with a gate that must pass before the next begins [C, §9 Q17]. *Status, 2026-10-25:*
steps 1 and 2 are built and their gates passed (§11); step 3 is built, its synthetic gate passed and the desktop's
run done (§12), and its handheld run is left (§12.7); step 4 is built and its gate passed (§13); step 5 is built
and its gate passed (§14); step 6 is built and its gate passed (§15); the stores step 5 left are built and their gate
passed (§18); step 7 is built and its gate passed (§19), the tester's checks in ES-DE and RetroArch left. Every test uses
synthetic images: SyntheticRom for cartridges, and generated sheets for discs, as `emusen-disc`'s own CHD test makes
them. No commercial image is committed.

| Step | What | Gate |
|---|---|---|
| **1** *(built 2026-10-25, §11.1)* | **The dump identity.** `RomHash` gains CRC32, SHA-1, ranges and the transform parameter; per-track hashing via `emusen-disc`; Mistress's `RomHashes` moves onto it | The parity class holds C# and Rust to one answer. The scraper's hashes are unchanged on a sample of the library, taken before and after. Per-track hashes reproduce the hashes of generated sheets' files |
| **2** *(built 2026-10-25, §11.2)* | **The manifest v1** in Galaxia: schema, read, atomic write, unknown fields kept, a newer major refused; `DataStore.Packages` and `Compressed` | Round-trip and refusal tests. A manifest written by hand in an editor loads |
| **3** *(built 2026-10-25, §12; the handheld's run left)* | **The CHD tool**: `emusen.library` with `chd convert`, `chd check` and `chd ls`, and the man page `chd.1`; output to `Compressed`; a linked package per disc (§4.4); no removal | Generated sheets round-trip through all three checks; a corrupted CHD fails check 2. Then the tester's PlayStation and Sega CD folders run on the desktop, and then on the handheld (P1–P4): every disc verified, the library's before and after listings identical, and the handheld's batch time, drain and temperature recorded. *Since 2026-10-25:* the shipped chdman's obligations of §4.9 met, with its pin checked |
| **4** *(built 2026-10-25, §13)* | **The Mistress window** for the tool, a LunaP `ToolWindow` reached from Settings, the pad menu and big picture, as Memory Cards is (`EmuSen_SaveMedia.md` §6.5) | The fit audit at 1280 × 800 and 1920 × 1200 with nothing cut off or overlapping; every control reached by the pad |
| **5** *(built 2026-10-25, §14)* | **Linked packages for every console**: the scan rules (§3.6), the claim index, `SaveLibrary` and `GameCheats` with a package root, `pkg make`, `pkg ls`, `pkg check` and `pkg migrate` | WiseMan: the library tree is unchanged after making and migrating packages. A game opened by its plain path and by its package reaches one save. A moved library file is found again by hash |
| **6** *(built 2026-10-25, §15)* | **Soft patching**: `emusen-patch`, the patch list, `pkg patch`, the `patched` identity | Each format applied to synthetic images reproduces targets made independently in the test. Mismatched sources are refused with words. The scraper sends `dump` for a patched package under `identity: original` |
| **7** *(built 2026-10-25, §19; the tester's ES-DE and RetroArch checks left)* | **Self-contained packages and zip per package** | A package copied to another machine opens there. The tester checks that ES-DE launches a self-contained package and RetroArch applies its patch |
| **8** | *Deferred 2026-10-25 (§9 Q2): not built until the tester revisits it.* **Replacing originals** (§4.5) | Removal goes to the trash and is refused without a terminal, for a failed check, and for a changed source. A shared track is kept. The library after replacement holds exactly the CHDs, at their checked hashes |
| **9** | *If wanted:* `pkg export esde` (media, and optionally a `gamelist.xml`) | ES-DE shows the exported art for the packaged games |

---

## 8. Risks

| Risk | What it would cost | Mitigation |
|---|---|---|
| **A removal of originals goes wrong** | the player's own dumps, which the project treats as irreplaceable | off by default, per batch, typed confirmation, three checks in the same run, the trash rather than unlinking, and Q2 asked as a standing rule (§4.5) |
| **chdman is missing, or differs between versions** | no conversion; or CHDs whose bytes differ by version | the tool needs a version line and records it in `container`; check 2 is independent of chdman's version. The handheld runs SteamOS, so how chdman reaches it is stage 3's to settle (§9 Q3) |
| **`emusen-disc` cannot read a codec chdman wrote** | a converted disc that will not play | check 2 reads every CHD through `emusen-disc` before it counts as done; `cdzs` support is confirmed at stage 3 (§4.8) |
| **Two saves for one game** | a player's progress split between two files | the claim index makes the plain path open the package (§3.5); migration copies, so nothing is lost while the rule settles |
| **A linked package goes stale** when the library is reorganised | a package whose game is "missing" | paths relative to `RomDirectory`; a search by size, then hash, as the orphan pass already does |
| **The scraper's KO allowance** spent on two identities | a day's scraping stopped early | `identity` chosen per package, never both tried by default (§5.5) |
| **Patch headers** (headered or headerless NES and SNES patches) | a patch that refuses, or a wrong image | BPS and UPS source CRCs refuse with words; the transformed form is offered, never applied silently; IPS is pinned to the source hash it was added for |
| **ES-DE and linked packages** | an entry ES-DE cannot launch | an opt-in link, or self-contained (§6.1, §9 Q16) |
| **Storage**: self-contained packages duplicate the library | twice the space for those games | self-contained is opt-in and per package; zip is per package |
| **Windows**: path length and links | a package folder a Windows system refuses | long-path-aware I/O, as Galaxia's §3.5 work already handles odd spellings (`EmuSen_RustPlatform.md` §27); no links on Windows |
| **The handheld's heat and battery** during a batch | an uncomfortable device, a drained battery | the thread limit (§9 Q9) and a notice when on battery; the gate records both |
| **The Unpacked cache's 4 GiB budget** filled by self-contained zips | first launches unpacking again | the budget is already a constant; it is revisited only if stage 7 measures evictions in normal play |

### 8.1 Predictions to be retired

- **P1.** Converting the tester's lone-track and cue/bin PlayStation discs to CHD with the default codecs takes each
  to 50–75% of its original size. Measured on two discs: 52.3% and 73.4%. Zipped discs are not predicted.
  *Retired 2026-10-25, otherwise than stated (§12.5): of 37 PlayStation discs, 30 fell in 50–75% and 7 below it, from
  26.9%; none above.*
- **P2.** Every disc of the tester's PlayStation and Sega CD folders passes all three checks of §4.3. The round trip
  was measured on two discs. *Retired 2026-10-25 as stated, on the desktop (§12.5): 42 of 42, at conversion and again
  by `chd check`.*
- **P3.** A CHD launch from the handheld's card takes no longer than 1.3's 95–120 ms for the same disc loose
  (`EmuSen_CoreAPI_Extensions.md` §10.2.5), since less is read from the card.
- **P4.** In 30 s of play on the handheld, CHD decoding adds under 1.5% of one core, and no frame over its period that
  the same disc loose does not have.
- **P5.** A zipped 64 MiB cartridge's first launch on the handheld spends under 1.5 s unpacking.
- **P6.** Making a linked package for a game already played, and migrating its stores, takes under 1 s on the desktop,
  dominated by hashing. *Retired 2026-10-25 as stated (§14.7): 18 ms to make a 4 MiB game's package and 3 ms to
  migrate its save, three states and their pictures; 237 ms for a 64 MiB game, the largest N64 image, with the file in
  the page cache.*
- **P7.** No step of §7 writes, renames or removes anything under `RomDirectory`, except step 8 when chosen. Checked by
  the before and after listings. *Holds for step 5 (§14.7): the listings before and after making and migrating
  packages, in Rust and in WiseMan, were the same file for file.* *Holds for step 6 (§15.8): the listings before and after
  adding and removing patches were the same, and both runtimes left the patched game's file as it was.* *Holds for step 7
  (§19.6): the listings before and after making packages self-contained, zipping and checking them were the same, in
  Rust and in WiseMan, a converted disc's included.*

---

## 9. Questions for the tester

The standing rules this page touches are asked first, each on its own. The design choices follow, each with a
recommendation.

### 9.0 The answers, decided 2026-10-25

Decided 2026-10-25 by the tester:

- **Q1 and Q4–Q17: as recommended.** Each question below carries its answer.
- **Q2: no removal for now.** The CHDs stay in `Compressed` beside the originals. Step 8 is deferred until the tester
  revisits it after seeing the tool verify their discs. §4.5 stays as the design for that day.
- **Q3, changed by the tester: EmuSen ships chdman.** chdman's source file is BSD-3-Clause (copyright Aaron Giles) and
  MAME as a whole GPL-2.0-or-later, both compatible with EmuSen's GPL-3.0. Shipping means the licence texts in
  `THIRD_PARTY_NOTICES.md` and `licenses/`, a written source offer naming the exact MAME release tag, the README's
  credits, and a pinned version checked by SHA-256; the binary is built or obtained as a black box, and MAME's source
  is never read, not even to build it. How and where it is built for each platform is settled at step 3; §4.9 records
  the plan.
- **The documentation's references.** The tester asked on 2026-10-25 that the documentation follow appropriate
  references. Every page that mentions chdman or CHD credits MAME and chdman with their licences, and cites MAME's own
  chdman documentation by URL and version in its sources; `THIRD_PARTY_NOTICES.md` and the README credit chdman as
  §4.9 lists when it ships. The same standard holds for everything this work cites.

### 9.1 Standing rules

1. **Q1, the ROM library rule: linked packages.** *Recommended:* linked packages read the library and never write to
   it. That covers sidecars, links and renames, and the save states that `StateDirectory` puts in the ROM folder on
   this machine, which are copied into a package and never removed (§3.4, §3.5).

   *Decided 2026-10-25 by the tester: as recommended.*
2. **Q2, the ROM library rule: removing originals after a CHD conversion.** Is this exception permitted at all?
   *Recommended:* yes, but only as §4.5 specifies:
   - off by default, and chosen per batch;
   - typed confirmation naming every file and the bytes freed, refused without a terminal;
   - only discs whose three checks passed in the same run;
   - the CHD placed and checked in the library first, the originals sent to the system's trash, never unlinked;
   - never for linked packages.

   *The alternative, equally coherent:* no removal at all. The CHDs stay in `Compressed`, and a player who wants space
   back removes originals by hand.

   *Decided 2026-10-25 by the tester: no removal for now; step 8 is deferred until the tester revisits it (§9.0).*
3. **Q3, dependencies: chdman as an external program.** It is a third-party binary the project neither ships nor
   builds, and the project's rule asks that every new dependency be stated. *Recommended:* use the player's own
   chdman, found on `PATH` or at a set path, recorded by version, and neither shipped nor fetched. How the handheld
   gets one is settled at stage 3: a static build beside the published tree is the candidate, and it would come back
   to the tester as its own question.

   *Why no writer of the project's own:* CHD v5 is defined only by MAME's source, which the house rule forbids
   reading (§4.1).

   *Decided 2026-10-25 by the tester, otherwise than recommended: EmuSen ships chdman, pinned and checked by SHA-256, built or obtained as a black box (§4.9, §9.0).*
4. **Q4, the firmware policy and packages.** *Recommended:* no package ever holds firmware, self-contained or not.
   Firmware stays in `home/Firmware` under `EmuSen_Firmware.md` §0: never shipped, fetched or prompted for.

   *Decided 2026-10-25 by the tester: as recommended.*
5. **Q5, the no-new-binary-format rule, applied to archives.** *Recommended:* the formats are:
   - CHD for discs;
   - zip, read through `ArchiveImages`, for cartridges;
   - JSON and ordinary files for everything else.

   7z is not read: it needs a new dependency and decoded 4.6 times slower in the measurement (§5.2).

   *Decided 2026-10-25 by the tester: as recommended.*

### 9.2 Design choices

6. **Q6, when packages are made.** *Recommended:* on request: *Make Package*, `pkg make`, a batch, or a conversion.
   Not at scan, which would write some 9,000 folders and hash the whole library unasked (§3.6).

   *Decided 2026-10-25 by the tester: as recommended.*
7. **Q7, the CHD codecs.** *Recommended:* chdman's defaults (`cdlz,cdzl,cdfl`) for every CD system, with `cdzs,cdfl`
   offered as *Faster to decode* and recorded per file (§4.8).

   *Decided 2026-10-25 by the tester: as recommended.*
8. **Q8, zip for self-contained cartridges.** *Recommended:* a choice per package, off by default. Reading goes through
   the existing unpack cache, so the borrow survives from the second launch on, and no in-memory path is added (§5.2).

   *Decided 2026-10-25 by the tester: as recommended.*
9. **Q9, threads for a batch.** *Recommended:*
   - on the desktop, all threads, since a batch is something the player starts and waits for;
   - on the handheld, half the threads by default, with a notice when on battery;
   - a setting either way.

   *Decided 2026-10-25 by the tester: as recommended.*
10. **Q10, where patch files sit.** *Recommended:* in the package's root, named as RetroArch finds them, whenever the
    list fits that convention. Otherwise their own names, with the package saying that RetroArch will not patch it
    (§5.4).

    *Decided 2026-10-25 by the tester: as recommended.*
11. **Q11, the scraper's identity for a patched package.** *Recommended:* the original's by default, with `patched`
    chosen per package for hacks that ScreenScraper lists in their own right (§5.5).

    *Decided 2026-10-25 by the tester: as recommended.*
12. **Q12, per-game settings.** *Recommended:* the game's crop and engine move from `games.db` into the package's
    `settings.json`, since the player authors them, and Galaxia's §7.2 keeps such files JSON. Favourites, play time,
    collections and edits stay in `games.db` (§3.5).

    *Decided 2026-10-25 by the tester: as recommended.*
13. **Q13, art.** *Recommended:* a packaged game's art lives in its `media/<type>/`, under ES-DE's type names, and the
    media store reads it there. `media.db` stays the scraper's cache (§3.5).

    *Decided 2026-10-25 by the tester: as recommended.*
14. **Q14, one game, one save.** *Recommended:* a library file that a package links opens as that package from every
    route, the shell's included, through Galaxia's claim index (§3.5).

    *Decided 2026-10-25 by the tester: as recommended.*
15. **Q15, disc patches.** *Recommended:* deferred to a page of their own. PPF and xdelta over a CHD need an overlay of
    patched sectors, which `emusen-disc`, a reader, does not have (§5.4).

    *Decided 2026-10-25 by the tester: as recommended.*
16. **Q16, linked packages in ES-DE.** *Recommended:* an opt-in symbolic link inside the package on Linux and macOS,
    none on Windows, and *Make Self-Contained* for anything else (§6.1).

    *Decided 2026-10-25 by the tester: as recommended.*
17. **Q17, the order.** *Recommended:* §7's steps in order, each behind its gate, with the CHD tool (step 3) as the
    first player-visible result, and step 8 only if Q2 is answered yes.

---

    *Decided 2026-10-25 by the tester: as recommended.*

## 10. Sources

**The project's code and pages**, read on `WiseMan` at `227dc34a`:

- `EmuSen.Galaxia/Media/ArchiveImages.cs`, `DiscFiles.cs` and `Library/RomHash.cs`;
- `EmuSen/Cores/CoreFactory.cs:120`, `Native/CoreEngine.cs` (`LoadRom`, `Lends`), `Native/NativeRtCore.cs:125` and
  `Native/PortEngine.cs:200`;
- `EmuSen.Mistress/Library/RomLibrary.cs`, `Scraping/Scraper.cs:275` and `Scraping/RomHashes.cs`;
- `EmuSen.Crystal/Systems/Psx/PsxSystem.cs:24` and `Systems/Genesis/GenesisSystems.cs:39–47`;
- the cores' `CAPABILITIES` (Diamond `src/v1.rs:221`, Nephrite `src/v1.rs:327`, MoonRT, MercuryRT, VenusRT and MarsRT);
- `EmuSen/Cores/Shared/emusen-disc/Cargo.toml`, and the `chd` 0.3.4 crate's `Cargo.toml` (its features, not its
  source);
- the pages listed at the top.

**External** (read 2026-10-16 unless dated otherwise):

- MAME and chdman. CHD is MAME's format and chdman MAME's tool. chdman's source file is BSD-3-Clause, copyright Aaron
  Giles; MAME as a whole is GPL-2.0-or-later. "The MAME License", MAME Documentation 0.289,
  <https://docs.mamedev.org/license.html>, read 2026-10-25.
- "chdman – CHD (Compressed Hunks of Data) File Manager", MAME Documentation 0.289,
  <https://docs.mamedev.org/tools/chdman.html>, read 2026-10-16 and again 2026-10-25: the commands, codecs,
  `createcd`'s defaults ("eight sectors per hunk (19,584 bytes)", `cdlz,cdzl,cdfl`), `-np` and `verify`. The
  measurements of §4 were made with chdman 0.289, the same release.
- ES-DE, `USERGUIDE.md` on GitLab (`es-de/emulationstation-de`, branch `master`): "Multiple game files installation",
  "Directories interpreted as files", "RetroArch ROM hacks", "Scraping process", and the scraper options "Search using
  file hashes" and "Hash searches max file size".
- docs.libretro.com, "Softpatching ROMs with RetroArch": the formats, naming, numbered IPS patches, and the
  load-from-memory limitation. The `retroarch(6)` manual, on `--ips`, `--bps`, `--ups` and `--no-patch`.
- docs.retroachievements.org, "Game Identification": each console's hash method.
- zerosoft.zophar.net, "IPS file format": the records, the RLE form and the footer.
- romhacking.net, document 746, "BPS File Format Specification", by Near, version 1.0 of 27 August 2012. Its listing
  was read; the text was not reachable, and vidua's notes (vidua.readthedocs.io, "BPS") were read in its place. The
  specification itself is to be read at stage 6.
- The UPS format by byuu, through secondary descriptions: mrcrowbar's documentation of its UPS container, and the
  `mkups` crate's documentation. The specification, linked from mrcrowbar as `ups-spec.pdf`, is to be read at stage 6.
- docs.rs, the `chd` crate: a reader, compatible with libchdr, with no plans for writing in its chdman substitute.
- The No-Intro header question: the libretro forum threads "NES playlist creation bug" and "For library scanning how do
  the games need to be named?", and igir's discussion #347. They conflict, which is why §3.3.1 keeps both forms.
- ScreenScraper's API, as `EmuSen_BigPicture.md` §5.1 cites it.
- For §11.1, read 2026-10-25 as a search engine's summaries only: datoso's documentation, "DATs and ROMs"
  (<https://www.mintlify.com/laromicas/datoso/concepts/dats-and-roms>), and the libretro forum thread "Redump DAT
  parsing bug? (Libretro-DATs/libretro-db)" (<https://forums.libretro.com/t/redump-dat-parsing-bug-libretro-dats-libretro-db/21440>).

- For §4.9 and §12, read 2026-10-25: "Compiling MAME", MAME Documentation 0.289,
  <https://docs.mamedev.org/initialsetup/compilingmame.html> (the Fedora prerequisites, `TOOLS`, `EMULATOR`,
  `NOWERROR`, `ARCHOPTS`, `USE_QTDEBUG`, `NO_OPENGL`, `NO_USE_PORTAUDIO`, `NO_USE_PULSEAUDIO`, `USE_TAPTUN`); MAME's
  release page, <https://www.mamedev.org/release.html>, and its GitHub releases and tags
  (<https://github.com/mamedev/mame/releases/tag/mame0289>, with the published `SHA256SUMS`); Fedora 44's `mame-tools`
  0.289-1.fc44 as `dnf` describes it (its requirements, and its files unpacked from the package, never installed). The
  licence texts of §12.1 are the upstream projects' own files, read the same day: GNU's GPL-2.0
  (<https://www.gnu.org/licenses/old-licenses/gpl-2.0.txt>), libFLAC's `COPYING.Xiph`, Zstandard's `LICENSE`,
  utf8proc's `LICENSE.md` and zlib's `LICENSE`, each from its project's repository.

- For §15, read 2026-10-25: zerosoft, "IPS file format" (<http://zerosoft.zophar.net/ips.php>); SnesLab, "IPS file
  format" (<https://sneslab.net/wiki/IPS_file_format>), for the truncation after `EOF`; the nesdev forum thread "Patch
  formats (such as xdelta, vcdiff, bps)" (<https://nesdev.nes.science/f13/t12046.xhtml>), as a search engine's summary,
  on the truncation's byte order and Lunar IPS; "UPS File Format Specification", 18 April 2008, the Internet Archive's
  copy of <http://individual.utoronto.ca/dmeunier/ups-spec.pdf>; byuu (Near), "BPS File Format Specification" 1.0,
  `bps_spec.md` in the Internet Archive's copy of romhacking.net document 746's download
  (<https://www.romhacking.net/documents/746/>); vidua's notes, "BPS" (<https://vidua.readthedocs.io/en/latest/bps.html>),
  on the CRCs' byte order and the offsets; docs.libretro.com, "Softpatching ROMs with RetroArch", again, for the names.

**Not read**: MAME's source, libchdr, chd-rs's source, RetroArch's patching source, or any emulator's code; for §15, no
patcher's source (Flips, beat, upset, xdelta or any other). MAME's
source archive was unpacked and built at step 3 and none of its files was opened.

---

## 11. Built 2026-10-25: steps 1 and 2

*Measured on the development desktop on 2026-10-25. The ROM library was only read; nothing under
`AppSettings.RomDirectory` was written.* Marks as at the top: what was run is [M].

### 11.1 Step 1: the dump identity

**What was already there.** The Rust half of `RomHash` had gained CRC-32, SHA-1 and ranges for the library and the
scraper in Rust (`EmuSen_Backend.md` §28.2, §30.4), with Mistress's `Scraping/RomHashes.cs` named as its C# twin until
state 3. So step 1 built the rest:

- **`RomHash` in C#** gains `Of` (CRC-32, MD5 and SHA-1 in one read, of a whole file, a range or bytes held, as the
  record `RomHashes`), `Transformed` (the system pack's transform as a parameter: the hashes of its form, or null where
  it changes nothing) and `Tracks` (§3.3's `dump.tracks`). Each forwards to the library while the switch has chosen
  it, with `RomHash.Managed` as the C# rule (`EmuSen_Galaxia.md` §5.4). The Rust half gains `transformed` and `tracks`,
  and the scraper in Rust calls `transformed` in place of its own comparison.
- **Mistress's `RomHashes` folded in.** `Scraping/RomHashes.cs`, with its `Crc32`, is deleted; the record and the CRC
  moved into Galaxia, and the scraper, its client and their tests call `RomHash`. One owner of a dump's hashes remains.
- **Per-track hashing via `emusen-disc`.** Where a track lies is `set::track_spans`' alone (`EmuSen_Disc.md` §12): from
  its first INDEX, 00 where given, to the next track's in the same file; a WAVE file's one track is the whole file.
  The C# half reads and hashes in C# and asks the platform library where each track lies, as `ImageSets` asks what a
  sheet names; so with `EMUSEN_GALAXIA_NATIVE=0` the tracks still need the library, which is the state of every
  sheet rule since 2026-10-12.
- **A stop flag** crosses the interface for the first time (`EmuSen_RustPlatform.md` §39).

**The gate** [M]:

1. *The parity class*: `GalaxiaParityTests`' identity cases pass, both halves equal to .NET's own MD5 and SHA-1 and a
   CRC-32 written in the test, for files from 0 bytes to 3 MB, nine ranges each, bytes held, a transform, and the
   tracks of three generated sheets; refusals alike.
2. *The scraper's hashes unchanged on the tester's library*: 231 files sampled with a fixed seed, up to 25 from each of
   the ten console folders (6,834 MB in all: cartridges, zips, `.7z`, lone tracks, sheets), hashed by the deleted
   `RomHashes` before the change, and after it by `RomHash.Of` through the library and through the C#. The three
   listings were byte-identical, and all three equal to Python's `hashlib` and `zlib`.
3. *Per-track hashes reproduce generated sheets' files*: the parity case and `rom_hash.rs`'s test build each track's
   bytes, write them as a file per track or joined in one file, and require each track's hashes to equal its bytes'.
   Beyond the gate, the 167 tracks of the tester's eight cue sheets (all of one file per track) were hashed by both
   halves and each equalled its file's hashes.

**Redump's fields**, which §3.3.1 left to be read again at this gate: Redump's own wiki page was still not reached on
2026-10-25. Secondary descriptions agree that its datfiles are Logiqx XML, one `<rom>` per track file with its name,
size, CRC-32, MD5 and SHA-1, the sheet listed beside them [R, a search engine's summaries of datoso's documentation,
"DATs and ROMs", and of the libretro forum thread "Redump DAT parsing bug?", 2026-10-25; §10], which `dump.tracks` and
`dump.sheet` hold. The primary source is still owed.

### 11.2 Step 2: the manifest v1

Built in Galaxia as §2.1 planned, described in `EmuSen_Galaxia.md` §3.10: `GamePackage` and `GamePackage.Managed` in
C#, `package.rs` in Rust, `emusen_galaxia_package_read`, `_write` and `_format` between them, and `DataStore.Packages`
and `DataStore.Compressed`.

**The schema as built** [C], beside §3.3's fields:

- *Required*: `format` (`emusen-package`), `version` (two numbers), `system` (not empty), `mode` (`linked` or
  `self-contained`), `game` with `role` (not empty), `where` (§3.3's four) and `path`.
- *A path* under `library`, `package` or `compressed` is relative, with no `..` part, either slash; under `absolute`
  it begins with a slash or a drive and a slash.
- *Optional, each null or absent*: `title`; `size` and `seen` (`size`, `mtime`); `dump` with `file`, `transformed`,
  `tracks` (each with `file`), `sheet` (with `file`) and `form` (`cue-bin`, `lone-track`, `iso`); `companions`;
  `patches` (each with `file` and `format`, `ips`, `bps` or `ups`); `patched`; `identity` (`original` or `patched`,
  not null); `container`, an object whose fields step 3 defines.
- *Hashes* are 8, 32 or 40 hexadecimal digits, either case; a whole number is an integer token of 0 or more.
- *A member written twice, or text that is not valid Unicode*, anywhere, is refused, since System.Text.Json's
  `JsonObject` cannot hold the one and cannot write the other.
- *Unknown fields*, at any depth, are kept in their places and not checked; that is what lets a minor version add
  fields an older build passes through.
- *A newer major* is refused before any other field is checked, since it may change any of them.

**Otherwise than §3.3 said**: nothing in the fields. Two consequences are recorded rather than hidden. Text outside
ASCII is written as `\uXXXX`, the escapes every config file has (`EmuSen_RustPlatform.md` §11.3); it reads back the
same, and a person may write it unescaped. And comments in a manifest written by hand are lost when the program writes
it again (§11.4, Q18).

**The gate** [M]: `GamePackageTests` pass in both states of the switch, a round trip through the file keeping unknown
fields in their places and tokens as written; a newer major is refused with words and a refused document leaves the
file as it was. The manifest written by hand (`EmuSen.WiseMan/Galaxia/HandWrittenPackage.json`: comments, tabs, a
trailing comma, CRLF, a byte-order mark, upper-case hex, a field of the player's own) loads. The parity class holds the
two halves to the same bytes for nine manifests, the same refusals with the same words for 72 texts, and the same files
on disk.

### 11.3 Tests, and the faults seeded

Run: `cargo test --profile test-checked` of `emusen-galaxia` (86), `emusen-platform` (33 and its integration tests)
and `emusen-scrape`; `emusen-disc` (14 and 13); WiseMan's Galaxia classes, `GamePackageTests` and `DataStoreTests` (86
with the library, and the two facade classes, 19, with `EMUSEN_GALAXIA_NATIVE=0`); and the blast radius of
`blast.sh`: every class of `EmuSen.WiseMan.Mistress`, the scraping classes, Galaxia's, Diamond's and Nephrite's, with
`DianaShSandboxTests` and `HierTests`, 2,647 passed, 62 skipped (the picture and bench tools) and none failed, in 20
minutes. Four faults were seeded, each caught and put back:

| Fault | Caught by |
|---|---|
| a track's start counted at its own sector size, not the size of the track before it in the file | `set.rs`'s span test; the parity track case |
| the C# range hash reading past its length | the parity range cases (5) and the track case |
| the Rust checks naming a malformed `game` before a newer major | `package.rs`'s newer-major test; the parity refusal case |
| the C# path check splitting at `/` alone, letting `a\..\b` through | the parity refusal case |

### 11.4 Left, and for the tester

- ~~**Step 3**, the CHD tool and shipping chdman (§4.9), waits for the tester's go.~~ Built 2026-10-25 (§12).
- **Q18 [routine]: comments in a hand-written manifest are dropped when the program rewrites it**, as the config files'
  are. *Recommended:* keep it so; a person's notes belong in an unknown field (`"my notes": ...`), which is kept.
- **Q19 [routine]: Redump's own description of its datfile fields** is still unread (§11.1). *Recommended:* read it
  when step 3 compares a converted disc against a datfile, which is the first use of the fields.

---

## 12. Built 2026-10-25: step 3

*Measured on the development desktop on 2026-10-25, with the chdman of §12.1. The ROM library was only read: its
`psx` and `SCD` folders were listed with every file's size, time and MD5 before and after the run of §12.5, and the
listings were identical. MAME's source archive was unpacked and built and none of its files was opened.* Marks as at
the top: what was run is [M].

### 12.1 chdman: obtained, pinned and shipped

**The release** [R, read 2026-10-25]. MAME 0.289, published 2026-07-30, is the newest release and the newest tag of
MAME's repository (`mame0289`), and the one MAME's release page offers. The page offers Windows binaries and the source
archive `mame0289s.exe`, a self-extracting 7-Zip archive, and no Linux binary; the release publishes each file's
SHA-256 in `SHA256SUMS`.

**Routes weighed.**

| Route | What was found | Taken |
|---|---|---|
| A binary from MAME | none for Linux [R] | no |
| Fedora 44's `mame-tools` 0.289-1.fc44, a distribution's package, unpacked and not installed | its chdman needs `libutf8proc.so.3`, which the desktop lacks, and `libSDL3.so.0`; it would not start [M] | no |
| Build on the desktop as "Compiling MAME" writes, `make TOOLS=1 EMULATOR=0` | the documented Fedora prerequisites are installed with `sudo dnf`; without them the generated project stopped for want of `sdl2`, `alsa`, `fontconfig` and `qmake6`, and with the documented options `USE_QTDEBUG=0 NO_OPENGL=1 NO_USE_PORTAUDIO=1 NO_USE_PULSEAUDIO=1 USE_TAPTUN=0` the tools' own OSD library still needed `SDL2/SDL.h` [M] | no: it needs sudo |
| The same, in a throwaway `registry.fedoraproject.org/fedora:44` container (rootless podman), the prerequisites installed in it as documented | built; nothing installed on the machine, no sudo [M] | **yes** |

**The build** [M]: `build-chdman.sh` checks the archive's SHA-256 against the pin (it equals MAME's published one),
unpacks it, and runs in the container the documented prerequisites' `dnf install`, with `libstdc++-static` beside them,
then `make TOOLS=1 EMULATOR=0 NOWERROR=1 ARCHOPTS="-static-libstdc++ -static-libgcc"` at `nice 19`, the container at a
fifth of the default CPU weight. About three minutes of wall clock on the 7700X, most of it the first run's 1,468
compilations. The first link failed for want of a static C++ library, which `libstdc++-static` supplied. The binary is
17,536,288 bytes, SHA-256 `55abb0d2…87e0f`, and prints `chdman - MAME Compressed Hunks of Data (CHD) manager 0.289
(unknown)`, "unknown" standing for the Git revision an archive does not carry.

**Why the C++ library is linked in.** Fedora 44's `libstdc++` exports `GLIBCXX_3.4.36` and the handheld's SteamOS
`GLIBCXX_3.4.35` [M, `strings`], so a binary linked to the desktop's would not be sure to load there. Linked in, the
binary needs only glibc 2.38 or later (`objdump -T`) and `libSDL2-2.0.so.0`; both machines have glibc 2.43, and the
handheld SDL2 2.32.70. Copied to the handheld's scratch folder through `hh.sh`, it printed its version line there, and
was removed [M]. **One binary serves the desktop and the handheld.**

**Inside the binary** are libraries of MAME's tree, found by the names and notices it carries (`strings`): zlib 1.3.2,
the LZMA SDK's coder, libFLAC, Zstandard and utf8proc, besides GCC's C++ library. `THIRD_PARTY_NOTICES.md` §1.7 lists
each with its licence; the texts in `licenses/` are the upstream projects' own (§10).

**Where it sits, and the pin**: §4.9. `Platform.targets` copies `~/.cache/emusen/toolchains/chdman/<rid>/chdman` beside
the platform library wherever it exists, so a build or publish on the desktop carries it; one without it carries none,
and the tool says so ("chdman was not found at …: nothing was converted").

### 12.2 The tool: `Platform/library-tool`

The crate `emusen-library-tool`, over Galaxia, `emusen-disc`, the library's routing (`emusen-library`) and the systems
table. Its parts: `pin.rs` (the pin read and a binary checked), `chdman.rs` (chdman run as a program, stopped by killing
it), `discs.rs` (the library's discs found and prepared), `checks.rs` (checks 2 and 3), `convert.rs` (one disc
converted, listed or checked, and its package) and `job.rs` (a request on a thread, its events queued).

**Finding the discs.** The files under each named path, a sheet's and a list's own files taken as their game's
(Galaxia's `ImageSets.Shelved`), with the extensions `.cue`, `.bin`, `.iso`, `.zip` and `.m3u`, each routed by the
library scan's own `route` among every system of the table; those routed to the PlayStation or the Sega CD are the
tool's. Where the routing names none, because no recogniser claimed the file and its own folder is no ES-DE system's
(`psx/Rayman (USA)/` holds Rayman), the nearest folder above it named for one of the two systems decides. The tester's
`SCD` folder routes by the Sega CD's recogniser, its name being none of ES-DE's.

**Preparing a disc.** A scratch folder `Compressed/.scratch-<process>-<n>` holds the sheet chdman reads and a link named
as the sheet names each file, pointing at it where it lies (§4.2). A lone raw track gets a sheet of one `MODE1/2352`
or `MODE2/2352` track by its mode byte, an ISO 9660 image one of `MODE1/2048`; a zip is unpacked once into `Unpacked`
through Galaxia's `disc_set::unpack`. **Skipped with the reason**, before anything is unpacked or written: a zip of
several track files and no sheet ("its tracks' order and gaps are unknown"), a sheet naming a WAVE file (a CHD keeps
the samples and not the file's header, so check 2 could not equal the original file), a file that is neither a raw data
track nor an ISO image, and an `.m3u`, whose discs are named one by one (§12.6).

**The checks, as built.**

- *The original's record* is taken before chdman runs: every track's three hashes through `RomHash::tracks` (§11.1),
  read through the links; the sheet's own hashes; and the original disc's table of contents, the sheet over its files
  mapped read-only (`emusen-disc`'s `Disc::from_sheet`).
- *Check 1* is `chdman verify`, passed only on a clean exit that prints "Overall SHA1 verification successful".
- *Check 3* compares, track for track, the number, data or audio, the pause's first sector, index 01 and the end, then
  the lead-out and point A0h's disc type, of the CHD opened through `emusen-disc` against the original's.
- *Check 2* rebuilds each track's bytes from the CHD as its file held them and hashes them with Galaxia's `Hasher`.
  Where a track begins in the CHD is its index 01 less the pause its file stores (its first INDEX to index 01), equal
  to the original's by check 3; how many sectors is the recorded size over the mode's sector size. So the record and
  the sheet alone place every track: `chd check` needs neither the original files nor their layout, and a track whose
  file ends in part of a sector is refused with words, since no CHD keeps that part. Audio comes back little-endian,
  a cooked sector's 2,048 or 2,336 bytes are cut from the raw sector `emusen-disc` frames.
- **Otherwise than §4.3 said: checks 3 and 2 are not run over a CHD that check 1 found damaged.** Measured on
  2026-10-25 on two synthetic CHDs: 66 corruptions (first written 67), four bytes changed at a stored hunk's start
  or 5 or 200 bytes into it, read through `emusen-disc`; **10 made `chd` 0.3.4 panic** (every one at a hunk's first byte), and the platform
  library is built to abort on a panic. So a damaged CHD would end the program whose shell or window checked it. The
  disc has failed by check 1 in any case, and the words say why the others were not run. Where check 1 passes, every
  hunk decompressed in chdman, and check 2 runs as designed. *Since `EmuSen_Disc.md` §13 (the same day)* no damaged
  hunk panics, but a damaged compressed map still can, at open, so the departure is kept for that reason alone
  (§12.6, Q25); check 2 names a hunk the reader refuses where chdman's verify has passed.
- *Not compared*: index points past 01, which a CHD's track metadata does not hold, and a sheet's `FLAGS`, which the CHD
  reader does not carry into the control nibble.

**The rules of §4.6, as built.** A disc is started only with its original's size and 64 MiB free at the output
(`statvfs`); chdman writes `<name>.chd.partial-<process>`; the original files' sizes and times are taken before and
after, and a change discards the result; a failure removes the partial; one disc failing never stops the batch; a stop
kills chdman and removes its partial. **A disc is done** when its package names this CHD, the CHD has the size and
time recorded beside its SHA-1, and every library file it was converted from has the size and time recorded: the
file system is the job's record, and a CHD changed underneath is converted again rather than trusted. Partial files and
scratch folders an hour old are removed at the next conversion. Each disc's event is appended to
`Logs/chd_<day>.log`.

**The package** (§4.4): `Packages/<console>/<path less the console folder>.chd/package.json`, linked, its fields as
`EmuSen_Galaxia.md` §3.10 lists them, with the original sheet's bytes beside it. Written after the CHD is in place, so
a package never names a CHD that is not there.

**Threads.** chdman compresses with every thread the machine has, Q9's desktop default; the handheld's half is the
caller's to ask for (`-j`), since the tool does not know which device it runs on (§12.7, Q20).

### 12.3 The surfaces

- **The platform interface, 24.** `emusen_tool_job_start` (a request in, a job out; `PARSE` with words for a request
  that is not one), `emusen_tool_job_next` (the next event by the length-query idiom, taken only when the buffer holds
  it; `ABSENT` when none came within the wait; 0 after the last), `emusen_tool_job_stop` and `emusen_tool_job_free`.
  The library never calls its host: a window polls. The prefix `emusen_library_` is the library's stable header's
  alone (`emusen_library.h`, whose test holds every such export to it), so the calls are `emusen_tool_job_`. The C#
  side gains nothing yet but the version, 24; step 4's window binds them.
- **The service `emusen.library` 1.0** (`Platform/library-tool/include/emusen_library_tool.h`): the same four calls
  and `emusen_platform_last_error`, as a table. A module never links Galaxia (`EmuSen_DianaSh.md` §16 Q11), and the tool
  reads Galaxia's tree and settings, so it is lent; the platform library hands the table to every shell it makes, in
  `dianash_new` and again in `dianash_set_services` unless the host names its own.
- **The module `emusen.library`** (`Platform/emusen-library-module`), linked into the platform library beside the
  other four of EmuSen's modules, its command `chd` and its page `Lib/dianash/emusen.library/man/chd.1`.
- **The request**: `{"op": "convert" | "check" | "ls", "paths": [...], "fast": false, "threads": 0}`, paths relative
  to the library with `/` between parts and no `..`, none for the whole library.
- **The events**, one JSON object a line:

| Event | Fields |
|---|---|
| `start` | `op`, `discs`, `refused` (the words for each path that names nothing); for convert and check `chdman` (its version line) and, since step 4, `log` (the batch's log, §13.4); for convert `codecs`, `threads`, `output` |
| `disc` | `source`, `system`, `state`; `index` and `of` for convert and check; then by state: `words`, `output` and `package` (paths under `Compressed` and `Packages`), `original` and `chd` (bytes), `seconds`, `checks` (`chdman`, `tracks`, `toc`, each `passed` or `failed: <words>`); for ls `size`, the disc's bytes in the library: the file named and, for a sheet, every file it names (since step 4, §13.4) |
| `end` | `op`, `counts` (each state's number), `seconds`, and `error` where the job refused to run: an unset or missing library, an overlap of it and the folders written, a chdman not found or not the pinned one |

A disc's states: convert gives `working`, then `converted`, `already`, `skipped`, `failed` or `stopped`; check gives
`verified`, `failed` or `not converted`; ls gives `converted`, `not converted`, `changed` or `missing`.

**`chd` and the line's budget.** DianaSh gives a line ten seconds (`EmuSen_DianaSh.md` §6.6), and `io->cancelled`
answers 1 once they are spent as once the player cancels, without saying which. A batch takes minutes. So each `chd`
request is a job the module follows, one line a disc, until it ends or the line is cancelled; a job still running then
goes on in the background as `chd<n>`, which `ps` lists with how far it has got, `chd wait chd<n>` follows again, and
`kill chd<n>` stops (the manifest's `jobs` claims the prefix `chd`). `chd wait` is the one subcommand beyond §7's three;
it exists for this. The module keeps its jobs in its per-shell slot, and a shell that ends stops and frees them.

### 12.4 The gate, the tests, and the faults seeded

**The synthetic gate** (`Platform/library-tool/tests/gate.rs`, generated sheets made as `emusen-disc`'s own CHD test
makes them; the chdman runs need `EMUSEN_CHDMAN`, set to the pinned binary) [M]:

1. *Generated sheets round-trip through all three checks*: a two-file dump with a stored pause, one file of three tracks
   with an unstored `PREGAP`, a lone raw track, a disc whose audio no codec shrinks, a zipped sheet and an ISO in a
   `segacd` folder are each converted, each check `passed`, each package written; `ls` calls each `converted`, `check`
   each `verified`, a second `convert` each `already`; the same with `--fast`, which answers §4.8's question: **CHDs
   written with `cdzs,cdfl` read through `emusen-disc` as `chd` 0.3.4 builds it**. The library's listing (every
   file's path, size, time and MD5) is the same after as before.
2. *A corrupted CHD fails check 2*: a CHD made from the same dump with one sample of track 2 changed, whole, so that
   chdman's verify passes it, with the package's SHA-1 made its own: check 1 and check 3 pass, and check 2 fails,
   "track 2 rebuilt from the CHD has SHA-1 … and the original …". A second case: bytes changed at a stored hunk's start
   fail check 1, and checks 2 and 3 are not run, the program still running.
3. *A CHD of another layout* (the same tracks with a longer unstored pause) fails check 3, and check 2 is not run.
4. *A resumed batch finishes*: a batch stopped after its first disc leaves no partial file and no scratch folder; a
   partial two hours old is removed; run again, the first disc is `already` and the rest `converted`.
5. *The pin refuses a wrong binary*: a file that is not the pinned chdman ends the job with "chdman at … has SHA-256 …,
   but this build is pinned to … (MAME 0.289, linux-x64): it was not run, and nothing was converted", both hashes
   named, and nothing written. A library that holds the output folders is refused before anything is written.

**Through the platform library** (`Platform/platform/tests/library_tool.rs`): a shell `dianash_new` makes runs `chd ls`,
`chd convert`, `chd check` and the usage over a library read from a tree's `appsettings.json`, writing only under the
tree's `Compressed` and `Packages`; a request that is not one is refused `PARSE`; with the line's budget made 1 ms a
conversion goes on in the background as `chd1`, `ps` lists it until it ends, and its package is there.

**Run** [M]: `cargo test --profile test-checked` of `emusen-library-tool` (2 and the gate's 7, with
`EMUSEN_CHDMAN` set; without it the gate's chdman cases say they are skipped and pass), `emusen-library-module` (3),
`emusen-platform` (33, `dianash_abi` 4 and `library_tool` 3), `emusen-galaxia` (86 and 1) and `emusen-library` (11, its
`library_abi` among them, which caught the first spelling of the calls, §12.3). WiseMan, the blast radius `blast.sh`
named (`DataStoreTests`, `DianaSh`, `Galaxia`, `GamePackageTests`, `GameRecords`, `Golden`, `Library`): 1,744 passed,
11 skipped and 5 failed, four of them the golden corpus meeting `chd`, which only the Rust shell has; with the corpus's
`shell rust` and the listings' patterns (`EmuSen_DianaSh.md` §13.1) and `chd.golden` recorded, `GoldenCorpusTests`,
`ManPageTests` and `NativeSwitchTests` pass on both shells, 208 and 207, but for the fifth. **The fifth is not this
step's**: `core.golden`'s refusals list the supported cores, and this tree's list has `endou`, which the recording of
WiseMan at `f4522996` does not; nothing of step 3 touches the core registry.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| check 2 placing each track at its index 01, the stored pause forgotten | five of the gate's cases, the round trip first: the two-file dump's second track, whose file stores its pause, no longer equals the dump |
| the pin compared by the hashes' length, not their digits | `pin.rs`'s refusal test |
| check 3 comparing each track's number alone | the gate's other-layout case: check 3 then named only the lead-out, two sectors on, and not the track whose pause moved |

### 12.5 The tester's PlayStation and Sega CD folders, on the desktop

**How it was run** [M, 2026-10-25]. Over the published sandbox `out/linux-x64/Mistress`, whose settings name the library
`/home/red/Documents/Roms/`, through `Platform/library-tool/examples/run_tool.rs`, which finds the tree as the program
does and prints the resolved folders before anything is written: `Compressed`, `Packages`, `Unpacked` and `Logs` under
that sandbox's `home`, none of them inside the library, which `checked_library` confirmed. `convert psx SCD`, chdman
with all 16 threads, the whole run at `nice 19` and a fifth of the default CPU weight, on the Ryzen 7 7700X from the
NVMe drive.

**The library, before and after.** Both folders were listed with every entry's path, every file's size, modification
time to the nanosecond and MD5, and every folder's time: 219 entries (92 files in `psx`, 119 in `SCD`, and their
folders), 18.6 GiB read. **The two listings were byte-identical.** Nothing under `RomDirectory` was written (P7 holds
for step 3).

**The result: 42 discs found, 42 converted, every one passing all three checks; none skipped or failed.**

| | Discs | Original | CHD | CHD / original | Time, summed per disc |
|---|---|---|---|---|---|
| PlayStation (`psx`) | 37 | 19,651,378,656 B | 11,109,573,404 B | 56.5% | 962.4 s |
| Sega CD (`SCD`) | 5 | 2,743,610,352 B | 1,365,864,840 B | 49.8% | 98.3 s |
| **Both** | **42** | 22,394,989,008 B | 12,475,438,244 B | **55.7%** | 1,060.7 s, the batch's own clock |

- *The forms*: 3 cue sheets in `psx` (Rayman's 51 tracks among them) and 5 in `SCD`, 17 zips unpacked through
  `Unpacked` (each held a sheet; none was refused for want of one), and 17 lone raw tracks.
- *Time per disc*: 12.2 to 40.2 s, 25.3 s on average, each disc's hashing of the original, unpacking where zipped,
  chdman's compression, chdman's verify, the rebuilt tracks' hashing and the table of contents. The whole batch took
  17 min 41 s of wall clock and 4,224 CPU-seconds (407% of one thread on average: compression is chdman's many
  threads, the verify and check 2 one each), at most 680 MiB resident.
- *Size*: from 26.9% (Ape Escape) to 73.4% (Crash Bandicoot) of the original. **P1 is retired, not as stated**: of the
  37 PlayStation discs, 30 fell in its 50–75%, and 7 below it (26.9% to 44.6%), discs whose data holds long runs that
  LZMA takes well; none above.
- *Against §4.7's measurements of 2026-10-16* (Fedora's chdman 0.289): Rayman's CHD is 323,075,192 B against
  323,075,270 B then, and Crash Bandicoot's 464,082,238 B against 464,161,384 B. The release is the same and the bytes
  are not: Fedora's build links the system's compression libraries and this one MAME's own, so CHDs of one release
  differ by build. Check 2 is indifferent to it, since it compares the rebuilt tracks with the dump, not CHD with CHD;
  the manifest records the binary's SHA-256 beside the version.

**Checked again, from the packages alone.** `check psx SCD` ran the three checks over the 42 CHDs in place against
their packages, the originals not read: **42 verified**, in 723.4 s. `ls psx SCD` then called all 42 `converted`, and a
third listing of the two folders, taken after both, was still identical to the first. **P2 is retired as stated** on
the desktop: every disc of the two folders passed all three checks, twice.

**What the run leaves**: 12 GB of CHDs under the sandbox's `home/Compressed`, 42 packages under `home/Packages`
(`PlayStation/` and `Sega CD/`), the zips' unpacked copies in `home/Unpacked` (3.9 GB, kept by that folder's own
4 GiB rule, §3.7 of `EmuSen_Galaxia.md`), and one line a disc in `home/Logs/chd_2026-10-25.log`.

### 12.6 Otherwise than the plan, and found on the way

- **Checks 2 and 3 skipped a CHD check 1 found damaged** (§12.2), because `chd` 0.3.4 panics on some damaged hunks,
  and the platform library aborts on a panic. The same reader serves play: a damaged CHD that a player launched could
  stop the core's process the same way. *Built the same day as its own `emusen-disc` step (Q21, `EmuSen_Disc.md`
  §13):* each hunk is checked before the crate decodes it (its block within the file, a cdlz, cdzl or cdzs block's
  stream length within the block) and after (the map's CRC-16), and a damaged hunk is refused in words that name the
  sector and the hunk. Of the 66 corruptions, 10 panicked and 56 were refused without words before; all 66 are refused
  in words after, with 0 panics, and a player who launches such a disc is protected. The cost on a clean CHD's read is
  6–12% of rates of 320–440 MB/s on the synthetic CHDs (`EmuSen_Disc.md` §13.4). **The departure is kept**, for a
  reason other than the first: a wider sweep found 25 single-byte changes to the compressed map that still panic in
  the crate at open (Q25), and chdman's verify failing is what keeps the tool's reader from such a file. Check 2 now
  names a hunk the reader refuses, should one pass chdman's verify.
- **The 10-second line budget** made `chd` a job that may go on in the background, with `chd wait` (§12.3).
- **Where the console is found**: the nearest console folder, where the scan's routing names no system (§12.2).
- **A zip's sheet is recorded under the name it is unpacked as**: `disc_set::unpack` names the image entry after the
  archive's stem (`EmuSen_Galaxia.md` §3.7), so `dump.sheet.file` is `Tekken 3.cue` whatever the entry was called.
  The bytes and their hashes are the entry's.
- **`.m3u`** lists are not converted as one: each disc is named on its own, and a new list in `Compressed` is left to
  a later step; the tester's two folders hold none.
- **The C# shell has no `chd`**: under `EMUSEN_DIANASH_NATIVE=0` the command is unknown, since a module is the Rust
  shell's. The C# shell is retired at S7 (`EmuSen_DianaSh.md` §13.11).
- **The done-check reads sizes and times**, not the CHD's SHA-1, which §4.6 named: reading a header's SHA-1 needs
  chdman or the `chd` crate's API, whose documentation for 0.3.4 did not build on docs.rs, and a size and time
  recorded beside the SHA-1 answer the same question cheaply, as `games.db`'s `seen` does (§3.3).

### 12.7 Left, and for the tester

- **The handheld's run (P1–P4 there)**: the same two folders converted on the Legion Go S, its batch time, battery
  drain and temperature recorded, as §7's gate asks. The binary already runs there (§12.1); the run is what is left.
  P2 holds on the desktop; P3 and P4, the handheld's launch and play, wait with it.
- **Step 4**, Mistress's window over the four calls of §12.3, waits for the tester's go.
- **Q20 [routine]: the handheld's thread default.** Q9 gives the handheld half its threads, and the tool, which does not
  know the device, uses every thread unless asked. *Recommended:* step 4's window and the handheld's run pass half
  (`-j 8` there); the tool keeps no device rule of its own. **Accepted 2026-10-25 as recommended**; step 4's window
  defaults to half (§13).
- **Q21 [decision]: a damaged CHD can stop the program that reads it.** `chd` 0.3.4 panicked on 10 of 66 damaged
  hunks measured (first written 67), and the cores and the platform library abort on a panic. *Recommended:* a step of
  its own for `emusen-disc`, measured with the same corruptions. **Decided 2026-10-25 by the tester as recommended,
  and built the same day** (`EmuSen_Disc.md` §13): the newer releases 0.3.5 and 0.4.0 panic on the same 10, so the
  hunks are checked before they are decoded and after; 0 of the 66 panic and all are refused in words. §12.2's departure
  is kept, since a damaged map can still panic at open (Q25).
- **Q25 [decision]: a damaged compressed map can still stop the program at open.** In a wider sweep of 49,298
  single-byte changes and truncations of seven synthetic CHDs (`EmuSen_Disc.md` §13.3), no hunk panics any more, but
  25 changes to the compressed map at a CHD's end panic in the crate's decoding of the map, in `Chd::open`, in 0.3.4,
  0.3.5 and 0.4.0 alike. Checking the map before the crate decodes it would need its compressed form, which no public
  document describes. *Recommended:* report it to the `chd` crate's maintainer with one of the synthetic files and the
  panic's message, take a release that fixes it when one comes, and measure it with the same sweep; until then accept
  it: the map is tens of bytes at a CHD's end, and the tool keeps its reader from any CHD chdman's verify fails (§12.2),
  so the risk is a player's launch of a CHD whose map is damaged. The departure of §12.2 is removed when it is fixed.
  **Decided 2026-10-25 by the tester:** accepted as it stands and not reported upstream for now; §12.2's departure stays.
  A core given a damaged hunk keeps receiving empty sectors, as for sectors an image does not hold (decided the same day,
  as recommended). The 42 CHDs of §12.5 were read whole through the new reader on 2026-10-25: none refused, none panicked.
- **Q22 [routine]: chdman's copyright line.** `licenses/chdman-BSD-3-Clause.txt` says "Copyright (c) Aaron Giles", as
  the tester read chdman's file header; its exact line, with any years, was not read here, since MAME's source is not.
  *Recommended:* the tester copies the header's line into that file. **Answered 2026-10-25:** at tag `mame0289` the
  header reads `// license:BSD-3-Clause` and `// copyright-holders:Aaron Giles`, with no years, so the file's line
  stands; the file now quotes both lines (read at the tester's request, those lines alone).
- **Q23 [decision]: chdman needs `libSDL2-2.0.so.0` on the player's machine.** MAME's tools link its SDL layer; both
  the desktop and SteamOS have the library, but a Linux system without SDL2 would refuse to start chdman, and the tool
  would say so and convert nothing. *Recommended:* accept it for Linux now (SDL2 is in every desktop distribution's
  base and in SteamOS) and look again when Windows and macOS resume, where a build will be needed in any case.
  **Decided 2026-10-25 by the tester as recommended.**
- **Q24 [routine]: `chd wait`**, a fourth subcommand, exists because a batch outlives DianaSh's ten-second line
  budget (§12.3). *Recommended:* keep it; the window of step 4 polls the job and needs nothing of it. **Accepted
  2026-10-25 as recommended.**

## 13. Built 2026-10-25: step 4

*Measured on the development desktop on 2026-10-25. No batch was run on the ROM library: every test made its own
discs in a temporary folder, as step 3's gate makes them, and the fit audit's states are fed a scripted job.* Marks as
at the top: what was run is [M].

### 13.1 The binding

`EmuSen.Galaxia/Native/LibraryToolJob.cs` binds the four calls of §12.3 on the managed side of the platform library,
beside Galaxia's own, and finds them by name through `PlatformLibrary.Export`, as `MediumLockNative` finds the medium
lock's.

- **`LibraryToolJob.Start(request)`** returns an `IToolJob`. A request the tool refuses (`PARSE`) is a
  `FormatException` carrying the tool's words. A library that is not loaded, or lacks a call, is an
  `InvalidOperationException` that names `PlatformLibrary.Report`.
- **`Next(wait)`** follows the length-query idiom. A buffer of 4 KiB is grown when an event is longer, and the event is
  taken only once the buffer holds it. `ABSENT` returns null, and 0 marks the job `Ended`.
- **`Stop()`** asks the job to stop, and **`Dispose()`** frees it, which stops it and waits for its thread.

The four calls are not added to `GalaxiaNative.Exports`. A library without them is therefore not refused for the rest
of Galaxia; only the tool is unavailable. The tool has no C# implementation, so a frontend without the platform library
says so and converts nothing.

**The library never calls its host.** The window polls: every 100 ms while a job runs, it takes every queued event
with no wait, at most 256 a tick. With no job, nothing runs.

### 13.2 The window

`EmuSen.Mistress/Views/CompressedDiscsWindow.cs`, a LunaP `ToolWindow` titled *Compressed Discs*.

**Where it is reached.**

- *Settings → Compressed Discs…* in the desktop's menus, after *Memory Cards…*.
- *Settings → Compressed Discs* in the pad menu's main menu, which big picture draws as its own panel.
- In big picture the window is framed as a sheet (`MainWindow.FramedWindows`), so it is used by the controller grammar
  alone.
- It is not in a game's *Game Settings*, where Memory Cards is: a batch holds half the machine's threads for minutes
  while the game plays. Q26 asks the tester.

**What it shows**, from the top:

1. **A plain statement.** "The library is left untouched: nothing in `<RomDirectory>` is changed, moved or removed.
   The CHDs go to `<Compressed>`."
2. **Convert or check.** The buttons *Convert*, *Check Converted*, *Stop* and *Resume*.
   - The first two name what they act on: *Convert 3 Chosen*, *Convert All 37* for a folder, *Convert Every Disc*.
   - Their help says what a CHD is kept for: the three checks of §4.3.
   - It also says that closing the window stops a batch, and that converting again does only what is left.
3. **Progress.** A bar of the discs the job has finished out of `of`, then "*n* of *m* discs done", and the disc now
   working.
   - The log's path, from the start event's `log` (§13.4).
   - A disc's own progress is its state (waiting, converting and checking, then its result with its seconds). The tool
     does not report chdman's progress within a disc (Q28).
4. **Discs from.** *Every folder*, or each of the library's top folders that holds a disc, with its count and its
   console: "psx: 37 PlayStation discs".
5. **Threads.** "*n* of *total*". The default is half the machine's threads, at least one, as §12.7's Q20
   recommended and the tester decided on 2026-10-25. The request's `threads` carries the choice, and the tool keeps no
   device rule of its own.
6. **Discs.** Each disc is a check box with its path below the folder, then a line with its size and state.
   - Size: the listing's `size`, the whole disc's (§13.4).
   - State: one of §12.3's states in the window's words (§13.3).
   - Where an event carried them, a third line gives the three checks' results, each named as a player would read it:
     "chdman's verify: passed; every track against the original: failed, track 2's file ends in part of a 2352-byte
     sector, which a CHD does not keep; the table of contents: passed."

Under all of these sits the status line: what the last batch did ("2 discs converted and verified, 1 already
converted, 1 failed, 1 left as it is, in … s. Nothing in the library was changed."), or why the tool did nothing.

**How it runs.**

- *Opening* lists the whole library with an `ls` job. An empty listing shows "No discs to convert" with the forms the
  tool takes.
- *What a batch names*: the chosen discs among those shown; with none chosen, the folder shown; with *Every folder*,
  the whole library.
  Discs lying in the library's own folder are named one by one, since an empty path is the whole library.
- *Stop* is `emusen_tool_job_stop`.
  - The tool ends chdman and removes its partial file.
  - The discs the job had not reached are marked *stopped* by the window, since the tool counts them without an event
    (§12.3's `run`).
  - *Resume* is offered once a batch ended with any disc stopped.
- *Resume* sends the last conversion's request again. The file system is the job's record (§4.6), so the discs already
  converted come back `already` and only the rest are converted.
- *Closing* the window frees its job, which stops it.
- **No removal of originals anywhere**, as §9 Q2 decided. The window has no control that writes the library, and its
  first line says so in plain words.

**Hidden rather than disabled.** While a batch runs, the folder and thread choosers are hidden, so that the choice
cannot change under the batch.

- A disabled dropdown or check box paints Fluent's disabled colours (`#FFFFFF33` and `#FFFFFF66`), and the menu look's
  palette declares neither. The recorded rules' palette reach (`EmuSen_InterfaceSpec.md` §4.4) found both in the
  running state's first recording [M].
- The check boxes stay enabled during a batch, and a disc chosen then is chosen for the next batch.
- Declaring the disabled colours would be a change to LunaP's menu palette for one window, and was not made.

### 13.3 The words

`CompressedDiscsWords` gives the window's own words for everything the tool says.

- **The table.** It holds one entry for each refusal (the end event's `error`, the start event's `refused`) and one
  for each reason a disc gives in `words`, as `Platform/library-tool` writes them.
- **An unmatched phrase.** The tool's own words are kept after "Nothing was done:". The words of a package the tool
  could not write are passed on as written.
- **Examples:**

| The tool says | The window says |
|---|---|
| `AppSettings.RomDirectory is not set: there is no library to convert.` | No library folder is set, so there are no discs to convert. Choose the library's folder in Preferences, then open this again. |
| `Compressed (…) and the library (…) overlap: …` | EmuSen's Compressed folder (…) and the library (…) overlap. The library is never written to, so nothing was done: keep the library outside EmuSen's own folders. |
| `chdman at … has SHA-256 …, but this build is pinned to …` | The chdman beside EmuSen is not the one this build was made with, so it was not run and nothing was converted. Installing EmuSen again puts the right one back. |
| `chdman at … could not be run: …` | chdman could not be started (…), so nothing was converted. On Linux it needs SDL2 (libSDL2-2.0.so.0) installed. |
| `1048576 bytes are free at …, and the disc needs 627108864 with the margin` | Failed, and no CHD was kept: there is not room for it: 1.0 MB free at …, and it needs 598 MB |
| `Dusk.zip holds 2 track files and no sheet, …` | Left as it is: Dusk.zip holds 2 track files and no cue sheet, so the order of its tracks and the gaps between them are unknown |

**A disc's states** are said as follows:

| State | Words |
|---|---|
| `not converted` | Not converted |
| `converted`, in a listing | Converted and verified: its CHD's size, and the share of the original's |
| `converted`, in a batch | the same, with its seconds |
| `changed`, `missing` | Changed since it was converted, or its CHD is gone: converting it again replaces or makes it |
| `already` | Already converted and verified |
| `skipped` | Left as it is: the reason |
| `failed` | Failed, and no CHD was kept: the reason; in a check, Failed the check: the reason |
| `stopped` | Stopped before it finished: Resume converts it |
| `verified` | Verified again: all three checks passed |

### 13.4 Two extensions of §12.3's events

Both are fields that the shell's module does not print, so `chd.golden` is unchanged.

- **The start event of a conversion or a check carries `log`**, the path of the batch's log. The window shows it.
  - Each disc's line now goes to the file named by the day the batch started, not by the day each disc ended.
  - So one batch keeps to one file, and the path the start event names is exact. §4.6's "each batch appends … to
    `Logs/chd_<day>.log`" reads so.
- **The listing's `size` is the whole disc's**: the file named and, for a sheet, every file it names, found with
  Galaxia's `image_sets::companions`.
  - It was the named file's alone, which for a cue sheet is a few hundred bytes.
  - The window's "see each disc's size" needed the whole.

**The service stays `emusen.library` 1.0.** Its table is unchanged. The one module that reads the events ships in the
same library as the tool, and it reads neither field.

### 13.5 The gate, the tests, and the faults seeded

**The fit audit and the pad** [M]. `WindowFitAuditTests` gained five cases: `CompressedDiscsEmpty`, `CompressedDiscs`
(a listing), `CompressedDiscsRunning`, `CompressedDiscsFinished` and `CompressedDiscsFailed`.

- *How each opens.* The window is opened as Settings opens it, over a running game, with a scripted job. Each state is
  therefore the same in every run.
  - The listing has five discs: one converted, one not converted with a title as long as a No-Intro name gets, one
    changed, one missing, and a Sega CD disc.
  - The batch states press *Convert* on three of them. The failed state has a check's failure, a disc left as it is,
    and the free space short.
- *The audit.* Both sizes pass `FitAudit.Check`, with nothing cut off, past its panel or overlapping.
- *The recordings.* All ten states are recorded in `Specification/`, and `InterfaceSpecFiles.Check` holds them to the
  rules of `EmuSen_InterfaceSpec.md` §4: every operable control is reached by the pad's walk, named, and painted from
  the palette.
- *Repeatability.* A second run compared equal to the recording.
- `PadMenuTests` opens the window from the main menu's *Settings*, and its list of that submenu's rows gained the
  entry.

**Real batches** (`CompressedDiscsWindowTests`, headless, with the pinned chdman that `Platform.targets` copies beside
the tests; without it the batch test says why it is skipped and passes) [M]. The test library holds five discs:

- a cue sheet with an audio track, built by `SyntheticPsxDisc`;
- a lone data track 152 seconds long, the one to stop on;
- a sheet whose audio file ends 100 bytes into a sector;
- a zip of two tracks and no sheet;
- one more sheet.

The test drives the window through a whole batch:

1. *The listing.* The window lists the five, each `not converted`, the first with the size of its sheet and two tracks.
   The plain statement names the library, and the threads default to half the machine's.
2. *Stop.* *Convert* is pressed, then *Stop* as soon as the first disc is `converted`. The last disc is `stopped` in
   the window's words. No partial file or scratch folder is left in `Compressed`, and the log the start event named
   exists.
3. *Resume.* The first disc comes back `already`, the long track and the last sheet are converted, and the zip is left
   as it is, in the window's words.
   - The short audio file fails check 2 with its words, and no CHD is kept for it.
   - The summary reads "2 discs converted and verified, 1 already converted, 1 failed, 1 left as it is, … Nothing in
     the library was changed."
4. *Check.* With two discs chosen, *Check 2 Chosen* verifies both, and each disc's checks line names the three passes.
5. *The library.* Its listing (every file's path, size, time and MD5) is the same at the end as at the start.

The whole batch takes about a second. A second test opens the window with no library set, and over a library that
holds EmuSen's own folders. Each refusal is said in the window's words, and *Convert* is disabled. A theory of 30 cases
holds each phrase of the tool to a sentence of the window's own.

**Rust** [M]: `cargo test --profile test-checked`, with `EMUSEN_CHDMAN` set.

- `emusen-library-tool` passes 2 unit tests and the gate's 7. The round trip now also holds the start event's `log`,
  with a line for each disc in it, and a sheet's listed `size` equal to its three files.
- `emusen-library-module` passes 3, and `emusen-platform`'s `library_tool` 3.

**WiseMan, the blast radius `blast.sh` named** [M]: 2,645 passed, 62 skipped and none failed, in 20 min 42 s, over `CompressedDiscsWindowTests`, `DataStoreTests`, every test under `EmuSen.WiseMan.Mistress`, `Galaxia`, `GamePackageTests`, `GameRecords`, `Library`, `PadMenuTests` and `WindowFitAuditTests`. Cargo: `emusen-galaxia` (86 and 1) and `emusen-library` (11) pass as before.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| the threads default set to every thread, not half | `CompressedDiscsWindowTests`' batch test, the threads chooser's default (the recordings do not show the chooser, which lies below their view) |
| the discs a stopped job never reached left `waiting` rather than marked `stopped` | the batch test: the last disc "waiting" where "stopped" was expected |
| the listing's `size` back to the named file alone | the gate's round trip: the sheet's size was not its three files' |

### 13.6 Left, and for the tester

- **The handheld's run** of §12.7 is still left; this step did not touch it.
- **Q26 [decision]: the window in a game's Game Settings.** Memory Cards is there, and this window is not, because a
  batch holds half the machine's threads while the game plays. The desktop's *Settings* menu reaches it during a game
  all the same. *Recommended:* leave it out of *Game Settings*, and keep the desktop menu as it is. **Decided
  2026-10-25 by the tester as recommended.**
- **Q27 [decision]: closing the window stops a batch.** A batch of the tester's two folders took 17 minutes on the
  desktop (§12.5). The window owns its job, and closing it frees the job. Because the file system is the record,
  converting again does only what is left, and the window says so. *Recommended:* keep it. A batch that outlives its
  window would need a job owned by the main window, with a status a player can find again, as the scraper's has.
  **Decided 2026-10-25 by the tester as recommended.**
- **Q28 [routine]: a disc's progress is its state, not a percentage.** The tool runs chdman with its output read
  only at the end, and reports a disc when it is done. *Recommended:* keep it. A disc takes 12 to 40 seconds on the
  desktop (§12.5), and the overall bar moves at each. **Accepted 2026-10-25 as recommended.**
- **Q29 [routine]: the faster codecs (`--fast`, §4.8) are not offered in the window.** *Recommended:* leave them to
  `chd convert --fast` until the handheld's run measures what they buy. **Accepted 2026-10-25 as recommended.**

## 14. Built 2026-10-25: step 5

*Measured on the development desktop on 2026-10-25. No package was made or migrated on the ROM library: every test made
its own library in a temporary folder (SyntheticRom, SyntheticNesRom, generated sheets), and the measurements of
§14.7 ran over random files in a probe folder under `~/.cache/emusen/`, removed once the figures were recorded here.*
Marks as at the top: what was run is [M].

### 14.1 The claim index

Built in Galaxia as §3.5 and §3.6 planned, described in `EmuSen_Galaxia.md` §3.11: `GamePackages` in C# and
`claims.rs` in Rust, with `emusen_galaxia_package_index` and `emusen_galaxia_package_resolve` between them.

- **The manifests are the index.** No second file is kept. A package claims its game, its companions, and the
  library files a converted disc was made from (`container.sources`), each by full path.
- **Resolving a path** gives the game file to open and the package whose stores it uses. A package's folder opens its
  game; a claimed file opens its package's game; a file whose size and then hashes are those of a package's game that
  is gone is that game moved, and the manifest names its new place; anything else opens as before.
- **A changed dump is refused at open**, as §3.4 has it: a claimed game whose size or time moved is hashed, its time
  recorded when it is still the dump, and refused with both hashes named when it is not. The words point at `pkg check`
  and `pkg make`. *Since §17 (Q34, decided 2026-10-25) only the package opened itself is refused; the file opened by its
  plain path opens as a plain game.*

### 14.2 Saves, states and cheats with a package root

`SaveLibrary` and `GameCheats` take the package root as a parameter (`EmuSen_Galaxia.md` §5.5), and both runtimes
resolve it once at open: `ManagedRuntime.Open` in C# and `open` in Crystal's Rust. The engine's battery files take it
through `CoreOptions.PackageRoot` (C#) and `Engine::package` (Rust), the session's states through
`ManagedSession.PackageRoot` and `Session::package`. Mistress asks Galaxia for the root where it names a resume state
or a cheat list itself, and takes it from the library entry where the scan already knows it.

- **A package's saves are named by the package**, not by the file opened: `<package>/saves/<name>.srm`, where
  `<name>` is the package folder's name less its extension. The folder is named after the game when it is made
  (§3.2), so the two agree then, and a game moved or renamed in the library keeps its save.
- **The plain save is read until the first write copies it in**, by `BatterySave`'s existing rule for a save kept
  elsewhere before; the cheat list likewise. A package made by a conversion, which no one migrated, therefore starts
  from the game's save rather than from nothing.

### 14.3 The scan

`RomLibrary.Scan` walks `Packages` and `Compressed` beside the library, in both its halves (`ScanManaged`, and
`emusen_library_scan` 1.2's `packages` and `compressed` in `emusen-library`). Nothing inside a package's folder is a
game of its own; a claimed file is not listed; each package whose game is there is one entry, shelved by the
manifest's `system`, and its entry carries the package's folder (`RomEntry.Package`, the games answer's `package`). A
package whose game is gone is not listed; `pkg ls` names it. No file is read for a claim. With no `Packages` folder,
or with the two fields left out, the scan is the one before packages.

### 14.4 The `pkg` command

`Platform/library-tool/src/packages.rs`, run as jobs of the library tool through the `emusen.library` service (1.1:
the table unchanged, four request ops more), printed by the module's `pkg` (`man pkg`).

| Request op | What it does | A package's states |
|---|---|---|
| `pkg-make` | a linked package for each game under each library path, routed as the scan routes it; `Packages/<console>/<the path less the library's console folder>/` | `made`, `already` (a package claims it), `failed`, `stopped` |
| `pkg-ls` | each package, or one console's, read and never written; a file is hashed only where its size or time moved | `ok`, `changed`, `moved` (with `to`), `missing`, `refused` |
| `pkg-check` | the game and every file hashed against the manifest; a moved game found and the manifest rewritten; a touched one's time recorded | `verified`, `moved`, `changed`, `missing`, `refused`, `stopped` |
| `pkg-migrate` | the game's battery saves and own save media, states, resume state, pictures and cheat list copied in, each renamed to the package's name; nothing overwritten | `migrated` (with `copied`, `kept`), `nothing`, `failed` |

Events are `start` (`op`, `packages`, `refused`, `log`), one `package` a game or package (`index`, `of`, `source` or
`package`, `state`, `words`, then `mode`, `system`, `where` and `game` where a manifest was read), and `end`. Make,
check and migrate write each outcome to `Logs/pkg_<day>.log`. A manifest of `pkg make` holds §3.3's fields: the game
with `size` and `seen`, `dump.file` with `dump.transformed` where the system pack's transform changes the bytes (the
OpenVGDB forms of `emusen-systems`), a sheet's `dump.tracks`, `sheet` and `form`, its tracks and `.sbi` as companions,
an `.m3u`'s discs as companions, and for a zip the image inside as `dump.file` with the archive's own hashes and the
entry's name in `container` (`type: zip`).

**Which files migrate takes** [C]: in `Saves/<console>/` (and the earlier places, the flat `Saves` for the SNES and
N64, `MD` and `MCD` for the Genesis and Sega CD, beside the ROM for the NES and Game Boy) a file whose name is the
game's stem and then one or two parts of a dot and up to sixteen letters or digits, so that `Super Mario Bros. 3.srm`
is never `Super Mario Bros`'s; in the state folder (`AppSettings.StateDirectory`, else `Saves/Save States`) the
game's `.state`, `.slotN.state` and `.resume.state` and their `.png`; and its cheat list, the console's or the flat
one. The console's folder is read first, so where both hold a save, the newer place's wins.

### 14.5 The surfaces

| Surface | Change |
|---|---|
| The platform interface | 24 → **25**: `emusen_galaxia_package_index`, `emusen_galaxia_package_resolve`, save kinds 5–7 (`PACKAGE_SRAM`, `PACKAGE_STATE`, `PACKAGE_RESUME_STATE`, the package's folder as `rom`) |
| `emusen_library.h` | 1.1 → **1.2**: the scan's optional `packages` and `compressed`, the games answer's `package` |
| The service `emusen.library` | 1.0 → **1.1**: the four `pkg-*` request ops and the `package` event; the table unchanged |
| The module `emusen.library` | the command `pkg` and its job prefix `pkg`; its manifest asks for the service at 1.1 |
| Galaxia, C# | `GamePackages`, `PackagePlaces`, `PackageIndex`, `ClaimingPackage`, `PackageResolution`; a `packageRoot` parameter on `SaveLibrary`'s three game paths and on `GameCheats.For`, `GameCheats.PackageFile`; `GamePackage.SavesFolder`, `StatesFolder`, `CheatsFile` |
| Galaxia, Rust | `claims` (the index, `resolve`, `Places`), `saves::package_*`, `package::SAVES_FOLDER`, `STATES_FOLDER`, `CHEATS_FILE` |
| The runtimes | `CoreOptions.PackageRoot`, `BatterySave.Open`'s `packageRoot`, `ManagedSession.PackageRoot`; Crystal's `Engine::package`, `Session::package`, `BatterySave::open`'s `package` |
| Mistress | `RomEntry.Package`, `RomLibrary.Beside` and the scan's overloads that take it |

The core ABI is unchanged, as §0 said it would be.

### 14.6 Otherwise than the plan, and found on the way

- **A library entry's path is the game file, not the package's folder.** The folder is what ES-DE would take; Mistress
  keys everything by a game's path (`games.db`, art, the scraper, the launch), so an entry names the file and carries
  the package beside it. For a linked package that is the path the game had before, so favourites, play time,
  collections and edits stay where they were: §3.5's "keyed by the package's path and its hash" is not needed for
  linked packages, and is left to step 7, where a self-contained package's game has a path of its own (Q32).
- **Saves are named by the package's name** (§14.2), which §3.5 left to `SaveLibrary`'s stem rule; with the stem a
  game moved to another name would have lost its save.
- **A converted disc's package claims the library files it was made from**, so the library lists the disc once, as
  its CHD, and opening the original sheet opens the package's CHD with its save (Q31).
- **The plain save is read until copied in** (§14.2), beyond §3.5's migration on request.
- **`pkg ls` sees a change by size or time**, and so misses a file rewritten to the same size within the second its
  time was recorded; `pkg check` hashes every file and catches it. Measured in the Rust test, which rewrote a
  cartridge's bytes within that second.
- **Left by this step**: the state records of `records.db`, the crop and engine §9 Q12 moves to `settings.json`, and
  the art §9 Q13 moves to `media/` are not migrated, and nothing reads them from a package yet (Q30). A state without
  its record loads as before (`EmuSen_Galaxia.md` §5.3a). *Built 2026-10-25 (§18).*

### 14.7 The gate, the tests, and the faults seeded

**The gate** [M]:

1. *The library tree unchanged after making and migrating packages*: `GamePackageGateTests` (WiseMan, through the
   platform library's job calls as a frontend reaches them) and `library-tool/tests/packages.rs` (Rust) list every
   library file's path, size, time and MD5 before and after `pkg make /` and `pkg migrate`, with states kept in the
   library as the author's machine keeps them: the same.
2. *A game opened by its plain path and by its package reaches one save*:
   `CrystalParityTests.A_game_opened_by_its_plain_path_and_by_its_package_reaches_one_save_both_ways`, the test core in
   both runtimes. The plain path's session names the package's states; its battery save is copied into the package and
   the plain one left as it was; the package's session names the same slots and loads the state the first saved; the
   two runtimes' answers and the files they leave in `Saves`, `Packages` and the library are equal.
3. *A moved library file is found again by hash*: in the same test the game moved and renamed opens by its new path
   with the same slots, and the manifest names the new place; `pkg ls` names where it is and `pkg check` rewrites the
   manifest; the parity case resolves a moved file and a package whose game moved, alike in both halves.
4. *`pkg check` catches a changed file and a missing one*: a cartridge's byte changed, a sheet's track changed, a
   track file removed and a game removed, each named with its words.
5. *The fit audit*: `WindowFitAuditTests.The_library_with_packages_lists_each_game_once_and_nothing_is_cut_off` at
   1280 × 800 and 1920 × 1200, the library holding a packaged game with a 92-character title and a converted disc's
   package: each listed once, nothing in the list or the sidebar cut off or overlapping. The scan's entries reach no
   window of their own; a package is a library entry like any other.

**P6, measured** [M, 2026-10-25, the files just written, so in the page cache; `run_tool` over a probe tree]: `pkg make`
took 3.6 ms for a 512 KiB Game Boy image, 18 ms for a 4 MiB SNES image and 237 ms for a 64 MiB N64 image; `pkg migrate`
of the SNES game's save, three states and three pictures 2.8 ms; `pkg check` of the three 229 ms. `pkg ls` of 1,003
packages took 19 ms, so the claim index read at each open costs less than that, and no second index was built.

**Run** [M]: `cargo test --profile test-checked` of `emusen-galaxia` (89), `emusen-crystal` (6, `crystal_abi` 6,
`frame_alloc` 1), `emusen-library` (2, `library_abi` 6, `routing` 2; its `library.rs` found no SQLite this machine's
`emusen-sqlite` would load and ran nothing, as before this step, so the package scan is also a unit test of `scan.rs`),
`emusen-library-tool` (4, the gate's 7 with its chdman cases skipped, `packages` 2), `emusen-library-module` (4) and
`emusen-platform` (33, `dianash_abi` 4, `library_tool` 3). WiseMan, the blast radius `blast.sh` named with
`BatterySave`, `CoreAdapterTests` and `GenesisBatteryTests` added for the three files of `EmuSen/Cores` it did not map:
4,238 passed, 62 skipped and 2 failed, in 26 minutes. The two were `help.golden` and `man.golden`, whose listing of the
Rust shell's module commands had not been given `pkg` correctly; corrected (and `pkg.golden` and `man pkg`'s case
recorded), `GoldenCorpusTests`, `ManPageTests`, `NativeSwitchTests`, `GalaxiaParityTests`, `GamePackageGateTests` and
the runtime case passed, 284.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| Crystal's Rust session naming a packaged game's states in the state folder, as for a plain file | `CrystalParityTests.A_game_opened_by_its_plain_path_and_by_its_package_reaches_one_save_both_ways`: the plain path's slots did not name the package's states |
| the claim index leaving out the library files a converted disc was made from | `scan.rs`' package case: the original sheet listed beside its package's CHD |
| `pkg migrate` taking every file whose name begins with the game's | `packages.rs`' name rule case, and the Rust gate: "Alpha Two"'s save and state copied into Alpha's package |

### 14.8 Left, and for the tester

- ~~**Step 6**, soft patching, waits for the tester's go.~~ Built 2026-10-25 (§15).
- **Q30 [decision]: the stores this step did not move.** State records, the crop and engine (§9 Q12) and art (§9 Q13)
  stay where they are, and nothing reads them from a package. *Recommended:* a step of their own before step 7, since a
  self-contained package is what would carry them to another machine, and each needs its reader changed with it
  (`records.db`'s key, Mistress's crop and engine, the media store). *Decided 2026-10-25 by the tester: as
  recommended; built in §18.*
- **Q31 [decision]: opening a converted disc's original opens its CHD.** The package claims the sheet it was made
  from, so the library lists the disc once and both routes reach one save, but the original files are then played only
  where the CHD is gone. *Recommended:* keep it; a player who wants the original played makes no package of it, or
  removes the package.
- **Q32 [routine]: a library entry is keyed by its game file** (§14.6). *Recommended:* keep it for linked packages; at
  step 7 a self-contained package's game moves into the package, and its `games.db` rows follow by the orphan pass's
  hash rule.
- **Q33 [routine]: no *Make Package* in Mistress yet.** §3.6 lists it among the ways a package is made; step 5's row
  names the command only. *Recommended:* add it to a game's options with step 6, whose *Add Patch* needs the same
  window. *Built so at step 6 (§15.5).*
- **Q34 [routine]: a changed dump refuses its plain path too.** A claimed file that is another dump now is refused
  however it is opened, so that its saves are not written against another game. *Recommended:* keep it; the words name
  both hashes and what to do. *Decided 2026-10-25 by the tester, otherwise: refuse only when the game is opened as the
  package. By its plain path the changed file opens and plays as a plain game, with its plain saves, and the package is
  marked changed; built in §17.*

---

## 15. Built 2026-10-25: step 6

*Measured on the development desktop on 2026-10-25. No patch was added to, and no package made of, the ROM library: every
test made its images in a temporary folder (generated bytes in Rust, SyntheticRom and the test core's images in
WiseMan), and each format's targets were built by the tests themselves.* Marks as at the top: what was run is [M].

### 15.1 `emusen-patch`: the three formats

A crate of its own, `Platform/patch`, as §2.3 planned: `ips.rs`, `ups.rs` and `bps.rs` are pure functions, bytes in and
bytes out, with no file, clock or state; `lib.rs` names the format by its marker, reads what a patch says of itself
(`info`), applies one (`apply`) and a list in order (`apply_list`). Its one dependency is Galaxia, for CRC-32 and
SHA-1, whose owner is `RomHash` (§2.2); `package.rs` is the one composition with files, below. §2.3 lists IPS, BPS and
UPS, and so those are the formats; PPF and xdelta stay with disc patches (§9 Q15). Each was written from its published
description, read 2026-10-25, and no patcher's source was read [R]:

- **IPS**: zerosoft's "IPS file format" and SnesLab's "IPS file format": `PATCH`, records of a three-byte offset and
  a two-byte size, big-endian, then the bytes, or a two-byte run length and one byte where the size is zero, then
  `EOF`; and the truncation extension, three big-endian bytes after `EOF` (SnesLab, and the nesdev thread "Patch
  formats (such as xdelta, vcdiff, bps)", which names Lunar IPS among its writers).
- **UPS**: "UPS File Format Specification" of 18 April 2008, the `ups-spec.pdf` §10 had been unable to read, found in
  the Internet Archive's copy of `individual.utoronto.ca/dmeunier/ups-spec.pdf`. Its sizes, blocks of a relative
  offset and XORed bytes to a terminating zero, the input read as zero past its end, and three CRC-32s, the patch's
  over every byte before it.
- **BPS**: byuu's (Near's) "BPS File Format Specification" 1.0, romhacking.net document 746, whose download §10 had not
  reached: `bps_spec.md` from the Internet Archive's copy of the document's zip. The number encoding, the four actions
  with their offsets, metadata, and the footer's order: source, target, then the patch's own CRC-32 over every byte
  before it. Neither specification says the CRCs' byte order; vidua's notes say little-endian, and the gate's targets
  agree.

**Decided where the descriptions are silent** [C], each named in its module so that a later reading can overturn it:

1. *IPS*: a record past the source's end lengthens the image, the bytes between left zero; a record whose offset reads
   as `EOF` (0x454F46) is the end marker, as any reader must take it; after `EOF`, nothing or exactly three bytes,
   anything else refused; a truncation longer than the image refused, since the extension only shortens.
2. *UPS*: the terminating zero stands for one byte equal in both files, so the position moves past it: the reading in
   which a block ends at the first byte the files share. An image that is already the output is refused ("it has been
   patched before") rather than unpatched, although the format runs both ways, since a package's list only patches
   forward.
3. *BPS*: an image that is the target is refused the same way; every offset is bounded as the specification says, a
   TargetCopy never reaching the byte being written.

**The checks.** A UPS or BPS patch checks its own CRC-32 first ("the patch file is damaged: its bytes have CRC-32 …,
and it records …"), then the image against the source's size and CRC-32, refused with both in the words before a byte
is written ("the BPS patch was made for an image of 1048576 bytes with CRC-32 …, and this image has 1048576 bytes with
CRC-32 …"); after applying, the result against the target's CRC-32. IPS carries none, so a package pins each IPS patch
to the SHA-1 of the image it is added to, its place in the list (`source_sha1`), and `apply_list` refuses another
image naming both SHA-1s. A patched image past 64 MiB is refused from the header, before anything is allocated.

**No allocation beyond the output image.** Each format allocates its result once, at the size its header or its
records give: IPS scans its records first for the size and the truncation, UPS takes its output size, BPS its target
size. A list of several patches makes one image a patch and lets the last go. **Cost** [M, the desktop, `cargo run
--release`, best of five, with WiseMan's tests running beside it, so an upper bound]: over a 64 MiB image changed one
byte in 4,096, IPS 23 ms, UPS 234 ms and BPS 233 ms, the two checksummed formats dominated by two CRC-32s of 64 MiB
each at RomHash's byte-at-a-time table. A 4 MiB SNES image is a sixteenth of that.

### 15.2 The patch list in the manifest, and `pkg patch`

**The fields** (§3.3), all of them allowed by step 2's schema, so the manifest stays 1.0 [C]:

- each `patches` entry: `file` (its name in the package's root), `format`, `name` (the name the file had when it was
  added, kept for the layout below; a field older builds keep unread), `sha1` (the file's), and either `source_crc32`
  and `target_crc32` (UPS, BPS, from the footer) or `source_sha1` (IPS, the image at its place in the list);
- `patched`: `patches_sha1`, `size`, `crc32`, `md5` and `sha1` of the image the list makes. **`patches_sha1` is the
  SHA-1 of one line a patch, in order, of its format, a space and its file's SHA-1, each ended by a line feed**, so it
  changes when the list or any file of it does (`emusen_patch::package::key`);
- `identity`, `original` unless the player sets `patched` (§15.4).

**`pkg patch add <package> <file>`** (the request op `pkg-patch-add`, the service 1.2): the file read and its structure
and own checksum checked; the game's image read (the entry inside, for a zip) and held to `dump.file`, so that a patch
is never pinned to another file; the list so far applied to it; the new patch applied to that result, which checks it;
the file copied into the package's root under a hidden name, the list laid out, `patched` taken again and the manifest
written. The file given is only read. A disc game is refused ("disc patches are left to a page of their own"), as §9
Q15 decided. Where a UPS or BPS patch's source is the system pack's comparable form of the image (an NES image without
its iNES header, say), the refusal says so, "It was made for this image's comparable form, … (its copier header taken
off): EmuSen gives the core the image as dumped, so the patch was not added", as §5.4 asked: said, never applied
silently.

**`pkg patch rm <package> <n>`** (`pkg-patch-rm`): patch `n` taken out, from 1 in the order `pkg ls` lists them; the
rest applied again to the image first, and the removal refused, nothing changed, where they no longer apply ("without
patch 1 the rest no longer apply …", as when an IPS pinned to the image a BPS made is left without the BPS); the
removed copy set aside, the rest laid out, `patched` taken again (null for an empty list), then the copy removed. It is
the package's copy that goes: the file the player gave was never moved.

**The layout, RetroArch's convention** (§5.4, §9 Q10): a list of one BPS or one UPS is named `<name>.bps` or
`<name>.ups`, a list of IPS patches alone `<name>.ips`, `<name>.ips1`, `<name>.ips2` in order, `<name>` being the
package's own (§14.2); any other list keeps each file's own name, made distinct, and `pkg ls` and the `added` event say
that RetroArch will not apply it (`retroarch: false`). Renames go through hidden names, so that two files may swap, and
are refused before anything moves where a name is taken by a file that is no patch of the list.

**`pkg check`** also reads every patch, holds it to its SHA-1, and applies the list to the image, so that a changed patch
file is `changed` with words naming it; **`pkg ls`** lists each package's patches, whether RetroArch applies them, and
its identity. The man page is `man pkg`.

### 15.3 Patching at load, in both runtimes

Where §5.4 put it: after the image is opened and before create, in `CoreEngine.LoadRom` (C#) and `Engine::load_rom`
(Crystal in Rust), for every v1 engine, whenever the session opened as a package (§14.1). The list is read from the
package, each file held to its recorded SHA-1, applied by `emusen-patch`, and the result held to `patched.sha1` where
`patches_sha1` names this same list, which gives an IPS list its target check. A refusal ends the open with the words
of the patch that refused ("hack.tst was not opened, since its soft patches did not apply: hack.ips is not the patch
that was added: its SHA-1 is …, and the package recorded …"); the core is never handed a half-patched image.

- **The bytes the core is handed** are held in memory standing in for the mapping: Galaxia's `Mapped::held` in Rust,
  and in C# a mapping the platform library made (`MappedFile.FromNative`), swapped into the disc's files by
  `DiscFiles.Patch`. A core that claims `BORROW` keeps them for the machine's life as it keeps a mapping, whose
  promise (readable, unchanged, in place) they keep; a core that copies has them freed when create returns, as before.
  A machine made again, by a reset that loads afresh, patches again. Nothing advises the system for held bytes.
- **The C# runtime has no rule of its own**: patching is `emusen-patch`'s alone, reached through the platform library
  as §2.3 said (Rule B), by `emusen_patch_package_image` (interface **26**), which answers a Galaxia mapping the host
  reads and frees as any other. Without the platform library, a package that lists patches refuses to open with words
  naming the library, rather than starting unpatched.
- **The original is never written** (P7): the game file is mapped read-only and only read; the patched image exists
  only in memory.

### 15.4 The `patched` identity

As §5.5's table has it:

- **The scraper** sends the dump's hashes for a patched package, the file's own, which is the dump the claim index
  holds it to; under `identity: patched` it sends `patched` first and the dump after a 404, recording "patched" or
  "file" as how the game was matched. Both halves: `Scraper.PatchedFirst` in Mistress and `run::patched_first` in
  `emusen-scrape`, over `GamePackage.ScrapesAsPatched` and `package::scrapes_as_patched`. The identity is set in the
  manifest; no command sets it yet (Q37).
- **A state's record** names the patched image's MD5 and size, in both runtimes, so that a state made on one list
  is "a different copy" to another (`EmuSen_Settings_Reference.md` §4.37); Mistress compares against the same MD5.
- **`games.db`** keeps the dump's identity, so adding a patch keeps the game's favourites, play time and edits.
- **Saves.** ~~§5.5 keeps them apart only through the state's record; the battery save is the package's, patched or
  not (Q35).~~ *Decided 2026-10-25 (Q35): no sharing.* A patched game's battery saves, own save media and states are
  its own, named by the list's save key, in both runtimes; the original's are never read or written by it (§5.5,
  built in §16).

### 15.5 *Make Package* and *Add Patch* in Mistress

§14.8's Q33, as recommended: `GamePackageWindow`, a LunaP `ToolWindow` titled *Game Package*, reached as *Game
Package…* from a game's context menu in the library and from the game options sheet, which the pad opens with Select
in big picture and which the desktop's options menu shares. It shows the package's place, *Make Package* where there is
none (the request `pkg-make` for the game's library path), the list with *Remove* on each patch, and *Add* for each
IPS, UPS or BPS file beside the game in the library, where RetroArch and ES-DE keep a hack; on the desktop *Browse…*
picks any file, and in big picture, where a system file picker cannot be driven by the pad, the files beside the game
are the choice, and the window says so when there are none. Every action is a job of the library tool, so the window
and `pkg` cannot disagree; a package made or changed scans the library again when the window closes.

### 15.6 The surfaces

| Surface | Change |
|---|---|
| `emusen-patch` | new: `Format`, `Refusal`, `Info`, `info`, `apply`, `Step`, `apply_list`, `MAX_IMAGE`; `package::{read_list, apply_listed, image, image_at, key, patched_json, patched_identity, entries, inside}` |
| The platform interface | 25 → **26**: `emusen_patch_package_image` |
| The service `emusen.library` | 1.1 → **1.2**: the request ops `pkg-patch-add` (`patch`) and `pkg-patch-rm` (`index`), the `package` event's `patch`, `patches`, `retroarch`, `identity`, `patched_crc32`; the table unchanged |
| The module `emusen.library` | `pkg patch add` and `pkg patch rm`, the patch file a shell path resolved through the sandbox; asks for the service at 1.2 |
| Galaxia, Rust | `Mapped::held`, `rom_hash::sha1_of`, `package::{Patched, patched, scrapes_as_patched}` |
| Galaxia, C# | `GamePackage.Patches`, `Patched`, `PatchedIdentity`, `ScrapesAsPatched`; `DiscFiles.Patch`; `MappedFile.FromNative` (internal); `Native.SoftPatches` |
| The runtimes | `CoreEngine` and `Engine` patch at load; `ManagedSession` and Crystal's `open` record the patched identity |
| Mistress | `GamePackageWindow`, `ShowGamePackage`, the *Game Package…* entries; `Scraper.PackageOf` |

The core ABI is unchanged, as §0 said it would be.

### 15.7 Otherwise than the plan, and found on the way

- **The primary specifications were read** (§15.1), which §5.4 and §10 had left owed; nothing in them contradicted the
  secondary descriptions the design had used.
- **IPS is pinned to the image at its place in the list**, not to the dump, which §3.3's words ("the dump it was made
  for") read as for a first patch only. A second IPS patch is made for the image the first made.
- **A manifest gains `patches[].name`**, the file's own name, so that a list that stops fitting RetroArch's convention
  can give each file its own name back; it is a field the schema already allows.
- **The transformed form is said, not offered.** §5.4 asked that the refusal "offers the transformed form"; giving the
  core the headerless image would change the bytes a whole console's games are handed, so the words name the form and
  the patch is not added (Q36).
- **No command sets `identity`.** §3.7 lists none; the field is set in the manifest by hand (Q37).

### 15.8 The gate, the tests, and the faults seeded

**The gate** [M]:

1. *Each format applied to synthetic images reproduces targets made independently in the test*:
   `Platform/patch/tests/formats.rs` builds each target from a source of generated bytes (bytes changed, blocks moved
   back and forward, a run, a repeated phrase, the length grown and cut), then the patch from source to target with the
   test's own encoders (`tests/encoders`, written from the same descriptions with a bit-at-a-time CRC-32 and no code
   of the crate's), and requires the crate's result to equal the target: IPS with records, runs and the truncation, UPS,
   and BPS with all four actions used. The same encoders make the patches of `library-tool/tests/patches.rs`, whose
   images the package applies at load to the targets the test built, and WiseMan's runtimes are handed an IPS patch
   written in the test.
2. *Mismatched sources are refused with words*: both checksums named for UPS and BPS, both SHA-1s for IPS in a list, an
   image patched before named as such, the headerless form named, a changed patch file named, at `pkg patch add`, at
   `pkg check` and at load.
3. *The scraper sends `dump` for a patched package under `identity: original`*:
   `ScraperTests.A_patched_package_is_scraped_by_its_dump_unless_its_identity_is_patched_and_then_by_the_patched_image_first`,
   the request's `md5`, `crc`, `sha1` and `romtaille` those of the dump; under `identity: patched` the patched image's
   first and the dump's after the 404. In Rust, `run.rs`' identity case.
4. *The library tree unchanged*: `library-tool/tests/patches.rs` and `GamePackageWindowTests` list every library file's
   path, size, time and MD5 before and after making packages and adding and removing patches;
   `CrystalParityTests.A_patched_package_s_core_is_handed_the_patched_image_and_the_file_is_never_written_both_ways`
   holds the game's bytes and the library's tree, both runtimes alike. **P7 holds for step 6.**
5. *Both runtimes hand the core the patched image*: in the same case, a state saved after 20 frames of the patched
   package equals, byte for byte, one saved after 20 frames of the target image opened as a plain file, in each
   runtime, and the two runtimes' answers and files are equal; the state's record names the patched image's MD5.
6. *The window*: `WindowFitAuditTests` at 1280 × 800 and 1920 × 1200 for a game with no package and for a long-named game
   whose package holds two patches with two more beside it, recorded in the interface specification; and
   `ThemedGameOptionsTests.The_game_package_window_is_reached_by_the_pad_and_so_is_every_control_in_it`.

**Run** [M]: `cargo test --profile test-checked` of `emusen-patch` (formats 8), `emusen-library-tool` (4, gate 7,
packages 2, patches 1), `emusen-library-module` (4), `emusen-galaxia` (89, advise 1), `emusen-crystal` (6, `crystal_abi` 6,
`frame_alloc` 1), `emusen-scrape` (3 with the identity case, and its integration tests), `emusen-platform` (33,
`dianash_abi` 4, `library_tool` 3) and `emusen-library` (2, 3, `library_abi` 6, `routing` 2). WiseMan: the blast
radius `blast.sh` named, with `CoreAdapterTests` added for `CoreEngine`: 4,256 passed, 62 skipped (the picture and
bench tools) and 2 failed, in 25 minutes. The two were the big-picture game options view's recordings, at both sizes,
whose list had gained *Game Package…*; recorded again, the specification's classes passed, 300. The golden corpus's
`pkg`, `help` and `man` were given `pkg patch` and the new page; `GoldenCorpusTests` passed, 102.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| UPS's terminating zero not moving the position past its byte | `formats.rs`' `ups_reproduces_independent_targets` |
| BPS's SourceCopy not moving its source offset past what it copied | `formats.rs`' BPS gate and the list case |
| Crystal's Rust engine handing the core the image unpatched | `CrystalParityTests.A_patched_package_s_core_is_handed_the_patched_image_and_the_file_is_never_written_both_ways`: the state differed from the target's |
| Mistress's scraper sending the patched image first whatever the identity | `ScraperTests.A_patched_package_is_scraped_by_its_dump_unless_its_identity_is_patched_and_then_by_the_patched_image_first`: two requests for an `identity: original` package |

### 15.9 Left, and for the tester

- **Step 7**, self-contained packages and zip per package, waits for the tester's go.
- **Q35 [decision]: a package's battery save is shared by the game patched and unpatched.** One file is claimed by one
  package, so a patch added to a package that has been played runs the hack on the original's save; §5.5 keeps them
  apart only in the state's record. *Recommended:* keep it for linked packages, and say so in the window and `man
  pkg`; at step 7 a hack becomes a self-contained package of its own, with its own saves. *Decided 2026-10-25 by the
  tester: no sharing. A patched game keeps its battery saves, save media and states apart from the original's; built
  in §16.*
- **Q36 [decision]: a patch made for the headerless form is refused, not applied to that form.** *Recommended:* keep
  the refusal; applying it would hand the core an image other than the dump, which is a choice per console for a later
  page.
- **Q37 [routine]: no command or control sets `identity`.** *Recommended:* add `pkg identity <package>
  original|patched` and a switch in the window with step 7.
- **Q38 [routine]: the checksummed formats' cost is RomHash's CRC-32**, a byte-at-a-time table at about 0.6 GB/s, two
  passes over a 64 MiB image at load. *Recommended:* leave it; a slicing CRC in `RomHash` is a measured change of its
  own if a 64 MiB patched game's launch is ever seen to lag.

---

## 16. Built 2026-10-25: a patched game's own saves

*Measured on the development desktop on 2026-10-25. Every test made its images in a temporary folder (the test core's
images and IPS patches written in the test); nothing was read from or written to the ROM library.* Marks as at the
top: what was run is [M].

**The decision** [C, decided 2026-10-25 by the tester, §15.9 Q35]: a package's battery save is not shared by the game
patched and unpatched. A game opened with its patch list applied keeps its battery saves, its own save media (the
*Per game* rule, `EmuSen_SaveMedia.md` §3.3) and its save states apart from the same package opened unpatched, so that a
hack can never write the original game's save. §5.5 and §15.4 state it as the rule.

### 16.1 The names

A patched game's files are in the package's own folders, by the package's name (§14.2) with `.patched-<key>` after it:

| File | Unpatched | Patched |
|---|---|---|
| Battery save, and each battery file by its suffix | `saves/<name>.srm` | `saves/<name>.patched-<key>.srm` |
| A *Per game* medium | `saves/<name><suffix>` | `saves/<name>.patched-<key><suffix>` |
| State, slot *n*, resume state, pictures | `states/<name>[.slotN].state`, `.resume.state`, `.png` | `states/<name>.patched-<key>[.slotN].state`, … |

**`<key>` is the list's save key**: the first eight hexadecimal digits of §15.2's `patches_sha1` rule (the SHA-1 of one
line a patch, in order, of its format, a space and its file's SHA-1, each ended by a line feed), taken over the list as
its entries record it. An entry with no recorded SHA-1, which only a hand-written manifest can have since `pkg patch add`
always records one, is keyed by its file's name in its place. An empty list has no key, and its game is the original.

**Why this scheme** [C]:

1. *Stable across rename and move*, step 5's rule (§14.2): the key is made of the patches' contents and order only,
   never a path or a file name, and the package's name already does not change when its game moves. The list's own
   file names were rejected for that reason: §15.2 renames the copies to keep RetroArch's convention as the list
   changes.
2. *Each list its own, and the same list always the same*: another file, another order or another patch gives another
   key; adding a patch and removing it again returns to the first list's saves; removing every patch returns to the
   original's.
3. *Cheapest to know at open*: it is read from the manifest with no image hashed, where the patched image's own SHA-1
   would need the list applied first (the runtimes apply it after the battery files are named) or a `patched` block
   that a hand edit may have left stale; and it is recomputed from the entries, not read from `patches_sha1`, for the
   same reason.
4. *Eight digits* keep the name readable; two lists of one package share a key with odds of 2^-32 a pair.

**Resolved once at open**, as the package root is (§14.2): `ManagedRuntime.Open` and Crystal's `open` read the key
beside the package and pass it to the engine's battery files (`CoreOptions.SaveKey`, `Engine::save_key`) and to the
session's states (`ManagedSession.SaveKey`, `Session::save_key`), so that a list changed while the game runs (from the
*Game Package* window) never moves a running game's files; the next open takes the new list. Mistress names a resume
state before opening by the same rule (`GamePackage.SaveKeyAt`).

### 16.2 What a patched game reads first

**Nothing.** §14.2's rule, the plain save read until the first write copies it in, holds for the unpatched game only:
a patched game whose file is absent starts its save empty, and reads neither the package's original save, nor the
plain `Saves/<console>/` one, nor their earlier homes. A hack's save layout is the hack's, and the decision is that the
two never meet.

**A save left under the shared name by step 6's build.** Between step 6 and this change a patched game wrote
`saves/<name>.srm`, the name the unpatched game uses. No such file is in the tester's data, since no package with
patches had been played, but one is possible in principle. It is left exactly where it is and becomes, again, the
unpatched game's save; the patched game starts its own beside it. Nothing is moved, renamed or deleted, and which game
last wrote the file cannot be known from the file; a player who wants a hack's progress from that build copies the
file to `<name>.patched-<key>.srm` by hand.

**Not moved**: the cheat list (`cheats.json`) stays one a package, and `pkg migrate` copies plain saves to the
unpatched names only, since a plain save in the library is the original game's.

### 16.3 Both runtimes, and the surfaces

| Surface | Change |
|---|---|
| The platform interface | 26 → **27**: `emusen_galaxia_save_path`'s package kinds (5–7) take the save key as `extra`, eight hex digits, null or empty for the unpatched game; any other text is `EMUSEN_PLATFORM_BAD_ARGUMENT`, so a key can never name a place outside the package |
| Galaxia, Rust | `package::{patches_key, save_key, save_key_at, is_save_key, SAVE_KEY_DIGITS}`; `saves::package_name`, and a `key` on `package_sram_path`, `package_state_path`, `package_resume_state_path` |
| Galaxia, C# | `GamePackage.SaveKey`, `SaveKeyAt`, `SaveKeyDigits`; a `saveKey` on `SaveLibrary`'s three game paths; `SaveLibrary.Managed.PackageName` |
| `emusen-patch` | `package::key` now composes Galaxia's `patches_key`, the rule's one owner in Rust |
| The runtimes | `CoreOptions.SaveKey`, `BatterySave.Open`'s `saveKey`, `ManagedSession.SaveKey`; Crystal's `Engine::save_key`, `Session::save_key`, `BatterySave::open`'s `key` |
| Mistress | the resume state's name before opening |

The key's rule has two halves, `GamePackage.SaveKey` in C# and `package::save_key` in Rust, as `PatchedIdentity` and
`package::patched` are; the parity case below holds them to one answer through the names and files both runtimes
leave. The core ABI is unchanged.

### 16.4 The gate, the tests, and the faults seeded

**The gate** [M], `CrystalParityTests.A_patched_game_keeps_its_own_battery_save_and_states_apart_from_the_original_s_both_ways`,
the test core in both runtimes, the package holding an original save and the library a plain one:

1. *The patched and unpatched game reach different battery saves and states*: the package opened with an IPS list
   names its slots `hack.patched-<key>…`, the key worked out in the test from §15.2's rule, and writes
   `saves/hack.patched-<key>.srm` (the pad's A raises the test core's save byte); with every patch removed the same
   package names `hack.slot2.state` and no `.patched-`.
2. *Two lists reach different saves*: the list replaced by another IPS patch names another key, no slot path in common
   with the first, and writes another file.
3. *The unpatched game's save is untouched by the patched one*: the original save's bytes and time, the plain save's
   bytes and the game file's bytes are as planted after both lists played.
4. Both runtimes' answers and the trees they leave in `Saves`, `Packages` and the library are equal.

The names' two halves are held alike for the three package kinds, with and without a key, by
`GalaxiaParityTests.A_packaged_game_s_saves_and_states_are_named_alike_both_ways_by_the_package_s_own_name`, which also
requires a key that is not hex digits to be refused; the key's rule by `package.rs`' save-key case, and the names by
`saves.rs`' patched case.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| The patched name ignoring the patch list: C#'s `GamePackage.SaveKey` hashing the list's length instead of its entries | the parity case: the C# run's slots did not name `hack.patched-<key>` for the key of the list |
| Crystal's Rust battery file sharing the original's: `BatterySave::open` naming `package_sram_path(root, None)` for a patched game | the parity case, in the Rust run: "the patched game wrote its own save", no `hack.patched-<key>.srm` written |

### 16.5 Left, and for the tester

- **The cheat list stays shared** between a package patched and unpatched (§16.2). A hack's addresses may differ, so
  a cheat may misbehave, but a cheat list is not a save and writes nothing the original reads.

---

## 17. Built 2026-10-25: a changed dump plays by its plain path

*Measured on the development desktop on 2026-10-25, over synthetic images in temporary folders.* Marks as at the top:
what was run is [M].

**The decision** [C, decided 2026-10-25 by the tester, §14.8 Q34]: a library file a package claims, whose contents
changed since the package was made, is refused only when the game is opened as the package (its folder or its
`package.json`), with the words naming both hashes as before. **Opened by its plain path, the changed file opens and
plays as a plain game**, with its plain saves and states (`Saves/<console>/<stem>`, the state folder), and the
package's are neither read nor written. The package is reported `changed` by `pkg ls` and `pkg check`, as step 5 built
them, and its game's *Game Package* window says so.

**Where** [C]: the resolution, both halves. A claimed file whose package would refuse it because its game is another
dump resolves as a plain path with no package (`claims::resolve` in Rust through a refusal that says whether it is that
one, `GamePackages.Managed.Resolve` in C# through an `IOException` of its own kind that the claim branch catches); a
package opened itself, or any other refusal (a manifest that does not read, a game gone and not found), is refused as
before. Both runtimes take the resolution at open, so both play the file plainly; every Mistress reader of
`GamePackages.RootFor` (the resume state's name, the cheat list, the scraper's package) follows, since `RootFor` is
the resolution. Nothing writes the manifest for a changed file: its `seen` stays the dump's, so the next open, `pkg ls`
and `pkg check` find it changed again.

**Mistress** [C]: the library list shows no package state of its own, so nothing changes there. The *Game Package*
window, whose package `RootFor` no longer names for such a file, is given the claiming package by the index and says it
is changed: "the game's file is not as the package recorded it (pkg check names why, most often another dump), so the
game plays as a plain one with its plain saves, and the package refuses to open until it is checked". The library's
*Restart* entry names the resume state by the resolution for a packaged entry, so that it agrees with the launch.

**The gate** [M]:

1. *A changed claimed file opens by its plain path in both runtimes and reaches the plain save*:
   `CrystalParityTests.A_game_opened_by_its_plain_path_and_by_its_package_reaches_one_save_both_ways`, whose last part
   rewrites the moved game as another dump; opened by its path it names its slots in the state folder and none in the
   package, writes `Saves/TEST/plain (moved).srm` when the pad's A raises the test core's save, and leaves the package's
   save as it was; both runtimes' answers and files equal.
2. *Opening the package still refuses in words*: in the same case, the package opened by its folder is refused with
   "has SHA-1"; `GalaxiaParityTests.A_path_resolves_alike_both_ways_and_each_rewrites_its_manifests_alike` resolves the
   changed file plainly and refuses its package, alike in both halves; `claims.rs`' changed case does the same by the
   folder and by `package.json`.
3. *`pkg check` reports it changed*: `library-tool/tests/packages.rs` resolves the changed cartridge by its plain path
   (plain, no package) and by its package (refused), and `pkg check` and `pkg ls` then still report it `changed` with
   its hashes.
4. *The window*: `GamePackageWindowTests.A_changed_game_resolves_as_a_plain_one_and_its_package_is_shown_changed`, a
   package made through the window's job, its game then changed: `RootFor` names no package, the index names the
   claimant, the package refuses, and the window says changed.

**Run** [M], for §16 and this section together: `cargo test --profile test-checked` of `emusen-galaxia` (92 in all),
`emusen-patch` (8), `emusen-crystal` (13), `emusen-platform` (40), `emusen-library-tool` (14), `emusen-library` (13) and
`emusen-library-module` (4), all passing. WiseMan, the blast radius `blast.sh` named with `CoreAdapterTests` added for
`CoreEngine`: 4,254 passed, 62 skipped and 6 failed, in 25 minutes. The six were `GoldenCorpusTests`, which could not
parse `man.golden` after `man pkg`'s new paragraphs had been written into it by hand, its byte count left behind;
recorded again through the corpus's own recording, `GoldenCorpusTests` and `ManPageTests` passed, 208.

**Fault seeded**, put back:

| Fault | Caught by |
|---|---|
| Rust's `claims::resolve` refusing a changed claimed file by its plain path, as before | `CrystalParityTests.A_game_opened_by_its_plain_path_and_by_its_package_reaches_one_save_both_ways` (the Rust run's open refused with the hashes) and `GalaxiaParityTests.A_path_resolves_alike_both_ways_and_each_rewrites_its_manifests_alike` (answer 16: C# plain, Rust refused) |

---

## 18. Built 2026-10-25: the stores

*Measured on the development desktop on 2026-10-25. Every test made its library, packages, `states.db`, `games.db`,
settings and media store in a temporary folder; nothing was read from or written to the ROM library, and `pkg migrate`
was never run on it.* Marks as at the top: what was run is [M].

**The decision** [C, decided 2026-10-25 by the tester, as recommended for §14.8 Q30]: the three stores step 5 left,
a state's record, the crop and engine (§9 Q12) and art (§9 Q13), get a step of their own before step 7, each moved
together with its reader. Each follows step 5's two rules: a packaged game's store is named so that the package moved or
renamed keeps it, and what was kept before is read until the package holds its own (§14.2), and `pkg migrate` copies it
in once, overwriting nothing and removing nothing (§3.5).

### 18.1 A state's record

**Where** [C]: beside the state in the package, the state's name with `.json`, as the picture is the state's name with
`.png` (`EmuSen_Galaxia.md` §5.2): `states/<name>.slot2.json` for `states/<name>.slot2.state`. One line of JSON holds the
record's members, as `StateRecord` names them, and the state's size and modification time:

```
{"Console":"TEST","Core":"V1TestCore (test)","StateVersion":1,"Build":"…","SavedAt":"…","RomFile":"plain.tst","RomMd5":"…","RomBytes":40,"StateBytes":4106,"StateModified":639…}
```

- **Keyed by its place, not by a full path.** `states.db` keys a record by the state's full path
  (`EmuSen_Galaxia.md` §5.3a), which a package moved or renamed would lose. A file beside the state moves with it.
- **Stamped as `states.db` stamps it.** The record is answered only while the state is the file it was written for,
  the same size and time in .NET's ticks; a state rewritten by anything else has no record, and a state with no
  record loads as before. This keeps §5.3a's rule that a record fails towards "no record", never towards a wrong one.
- **Why not a row in a package's own database.** The record would then need SQLite wherever a package is read, and the
  Rust shell lends none to its runtime (`with_records` is `None` there, `EmuSen_Backend.md` §14.12); a file beside the
  state is read by every program. §5.3a removed the sidecars because the state folder on the author's machine is the
  ROM library, which is never written; a package's `states/` is EmuSen's own folder, so that reason does not hold
  there.
- **A patched game's states keep their own records** (§16): their names carry `.patched-<key>`, so their records do.

**The readers and writers, both runtimes** [M]: a state is a package's when its folder is named `states` and the folder
above holds `package.json` (`SaveLibrary.PackageRecordPathFor`, `saves::package_record_path`). For such a state the
record is written beside it and never to `states.db`; it is read there first, and only where no file is there is
`states.db`'s row read (a state saved into a package by step 5's or 6's build, recorded by its full path then). Forgetting
removes both. The C# runtime's `StateRecords` routes in its `Read`, `Write` and `Forget`; Crystal's Rust routes through
`states::read`, `write` and `forget`, which take `states.db` as an option, so that a packaged state's record is kept and
answered with no SQLite lent at all (`state_record` and `state_forget` no longer refuse for such a state). Mistress
reads a record only through the runtime's `state_record`, so it follows.

### 18.2 The crop and engine: the package's `settings.json`

**One file, the game's own settings** [C]. Since SR10 the engine a game chooses is not `games.db`'s `altemulator`
(§3.5's table said so before that move) but the runtime's file of the game's own settings,
`/etc/EmuSen/games/<system>/<stem>.json`, a map of strings (`EmuSen_Backend.md` §14.7). A packaged game's
`settings.json` **is** that file, moved into the package, with the same format, so that the runtime reads one more place
and no second schema exists. The crop joins it as one more string, `"Crop": "<left> <right> <top> <bottom>"`, the four
percentages in .NET's invariant shortest form, or `""` for a game that has none of its own and takes its console's.
A string, because the runtime reads the file as a map of strings and reads as none a file that holds anything else.

- **The switch is the file** for the runtime's settings: with no `settings.json` the plain file is read, and the first
  change writes the package's, the plain values copied in by that read and the plain file left as it was
  (`GameSettings.ReadPath` and `WritePath` in Galaxia, `settings::read_place` and `write_place` in Crystal's Rust).
- **The switch is the key** for the crop: where `settings.json` holds no `Crop`, `games.db`'s row is read; `ClearCrop`
  writes `""`, so that a cleared crop stays cleared while the row is kept. `games.db` is never written for a packaged
  game.
- **Either path reads the same.** Every reader resolves the path it is given first (`GamePackages.OpenedAs`,
  `claims::opened_as`), so a package's folder reads its game's settings and crop, the plain file's included, before
  the package has its own.
- **Readers** [M]: the runtimes' session settings (`GetSettingOf`, `setting_of`, `apply_stored`) and the engine chosen
  at open take the session's package root, resolved once at open as the saves' is; the calls with no session,
  `game_setting_get` and `game_setting_set`, and firmware asked before a load resolve the path. Mistress's crop is
  `GameRecords.Crop`, `SetCrop` and `ClearCrop`, which route before either half of `games.db`, so the C# and Rust
  records agree trivially and `LibraryRecordsParityTests` holds as before.
- **Not followed** [C]: the engine a route answer names (`emusen_crystal_route`) is the plain file's, since a route is
  asked of every library file and resolving each would walk the claim index thousands of times. Nothing opens a game
  by it; the open names its own engine.

### 18.3 Art: the package's `media/`

**Where** [C]: `<package>/media/<type folder>/<package's name><extension>`, ES-DE's type folders (`covers`,
`screenshots`, `marquees`, …) as the media store uses them (`EmuSen_BigPicture.md` §5.6), named by the package's own
name as its saves are (§14.2), so that the game renamed or moved keeps its art (`GamePackage.MediaPathFor`,
`package::media_path`).

- **Reading** [M]: a picture is looked for in a packaged game's package after the cover the player chose from another
  game and what the player placed, and before the media store, an ES-DE folder and OpenEmu's failover
  (`MediaSources.Locate`, `media.rs`' `locate`); its source is `package`. The store's picture is therefore read until
  the package has its own of that type, the store's named by the library file's own game folder whatever folder a list
  shows the game in (`EmuSen_BigPicture.md` §30.3a). The variant triggers count a package's types (`Present`), and the stamp that
  tells the library to draw again includes each package's `media` folders.
- **Which games are packages** is the library scan's answer (`RomEntry.Package`), handed to both halves as a map of
  game file to package: to the C# `MediaSources` directly, and to the scraper in Rust as its settings' `packages`
  (a minor extension of that request; absent, no package is consulted). A picture lookup is asked for every tile
  drawn, so no lookup walks the claim index.
- **Writing** [M]: both scrapers (`Scraper.KeepAsync`, `run.rs`' `keep`) write a packaged game's art into its package,
  resolving the game as they already did for the patched identity (§15.4), and `media.db` names the file by its full
  path, which the store's relative paths never are. A file the store kept before is never removed for a packaged game,
  and the rename follower passes over a full path.
- **Not moved**: `media.db`'s scraped text stays the scraper's cache (§9 Q13); Clear (`MediaStore.Forget`) still
  deletes only below the store's own folder, so a package's art is left by it.

### 18.4 `pkg migrate`

Beside step 5's saves, states, pictures and cheat list (§14.4), `pkg migrate` now copies [M]:

| Store | From | Into the package | Rule |
|---|---|---|---|
| A state's record | `states.db`'s row for each state copied, else `records.db`'s, read only and stamped against the original state | `states/<name>…json`, stamped against the copy | only where the package has none; a state already there takes its record only while its bytes are the original's |
| The game's own settings | `/etc/EmuSen/games/<system>/<stem>.json`, copied byte for byte | `settings.json` | only where the package has none |
| The crop | `games.db`'s `crop_*`, read only | `settings.json`'s `Crop` | added where `settings.json` holds no `Crop` |
| Art | the media store, `home/Media/<system>/<type>/<game's folder>/<stem>.<ext>` | `media/<type>/<package's name>.<ext>` | each type the package has no file of |

**SQLite for the tool** [C]. The records and the crop are in databases, and the library tool had no SQLite. It takes
the process's own: the e_sqlite3 a .NET host already loaded, found by its file name without loading a second copy
(`emusen_sqlite::Sqlite::of_process`, `RTLD_NOLOAD` on Linux and macOS, `open_already_loaded` on Windows), else the file
the program ships beside the platform library. The rule that a process holds one SQLite (`EmuSen_Backend.md` §28.3)
holds, and no interface changed. Where neither is there, as in the Rust shell alone, migrate copies the files and says
"the states' records were not copied: this program has no SQLite loaded to read states.db with"; run again where there
is one, it copies them. The outcome gains `records` (how many) and `records_words`; the shell prints both.

**The library tool's new dependencies**, said as the code design asks: `emusen-crystal`, for a record's place, format and
`states.db`'s read, and a game's own settings file, each read through its owner rather than copied; `emusen-scrape`, for
the media store's layout and kinds; `emusen-sqlite`. All three were already linked into the platform library.

### 18.5 The surfaces

| Surface | Change |
|---|---|
| The platform interface | unchanged, 27 |
| The frontend API | unchanged; `state_record` and `state_forget` answer a packaged state with no SQLite lent |
| The scraper's settings | `packages`, an optional map of game file to package folder; the media answer's `source` gains `package` |
| The service `emusen.library` | unchanged, 1.2; the `package` event of `pkg-migrate` gains `records` and `records_words` |
| Galaxia, C# | `GameSettings` (moved from the runtime: `PlainPath`, `PackagePath`, `ReadPath`, `WritePath`, `Read`, `Write`, `CropKey`, `CropText`, `ParseCrop`); `SaveLibrary.PackageRecordPathFor`; `GamePackage.SettingsFile`, `MediaFolder`, `NameOf`, `MediaPathFor`; `GamePackages.OpenedAs` |
| Galaxia, Rust | `saves::package_record_path`; `package::SETTINGS_FILE`, `MEDIA_FOLDER`, `media_path`; `claims::root_for`, `opened_as` |
| `emusen-sqlite` | `Sqlite::of_process`, `SHIPPED_NAME`; `Debug` |
| The runtimes | the packaged record in `StateRecords` and `states::{read, write, forget, read_packaged, package_record_text, kept_record, write_package_copy}`; the game's own values with a package root in both |
| Mistress | `GameRecords.OpenedAs`, `SystemOf`; `MediaSources`' `packages`, `MediaSource.Package`; `EsdeMediaFolder.FindInPackage`, `PackageStamp`; the window's `PackagesByGame`, sent to the scraper in Rust after each scan |

The core ABI is unchanged.

### 18.6 Otherwise than the plan, and found on the way

- **§3.5's table named `games.db`'s `altemulator` as the engine's owner**; it has been the runtime's file since SR10, so
  that file is what moves, whole.
- **§3.5's table had the record copied as "a row for the new path"**; a row keyed by a path is what a moved package would
  lose (§18.1).
- **The library tool needs SQLite** for two of the four stores, which §3.5 did not foresee (§18.4).

### 18.7 The gate, the tests, and the faults seeded

**The gate** [M]:

1. *For each store, a packaged game opened by its plain path and by its package reads the same record, crop/engine and
   art*: `CrystalParityTests.A_packaged_game_s_state_records_and_own_settings_are_its_package_s_by_either_path_and_kept_when_it_moves_both_ways`,
   both runtimes, the session-less settings by the game file and by the package's folder, the session's by each route,
   the slot's record by each; `PackageCropTests` through both halves of `games.db`'s records; the art by its game file,
   the key both routes share (§14.6), in `ScrapeMediaParityTests`, `GamePackageGateTests` and the fit audit's library.
2. *`pkg migrate` copies them in and leaves the originals byte-identical*:
   `GamePackageGateTests.Migrating_copies_the_state_records_settings_crop_and_art_in_and_leaves_their_originals_as_they_were`,
   through the platform library as Mistress reaches it: `states.db`, `games.db`, the settings file, two store pictures
   and the state hashed with SHA-256 before and after, equal; a second migrate copies nothing. In Rust,
   `library-tool/tests/packages.rs` copies the settings and the art and says why the records were not copied.
3. *A moved package keeps them*: the parity case moves the package's folder and reads the record and the settings by the
   plain path; the gate case moves it and reads the record, the engine and the cover; `PackageCropTests` moves it and
   reads the crop.
4. *The library tree is unchanged*: the gate case's listing of every library file's path, size, time and MD5, before
   and after making, migrating and moving; the parity case's tree of the library.
5. *Both runtimes and both languages alike*: the parity case's answers and the trees it leaves in `Packages`, the
   library and the settings folder (the records' moments and builds hidden); `ScrapeMediaParityTests` over three seeds
   with packaged games, every game asked every type and the shelves' present types and the packages' stamp compared;
   `ScrapeReplayParityTests.A_packaged_game_s_art_is_written_into_its_package_alike`, each half's package compared.
6. *The fit audit for the windows that show art or crop*:
   `WindowFitAuditTests.The_library_with_packages_lists_each_game_once_and_nothing_is_cut_off` at 1280 × 800 and
   1920 × 1200, its packaged game's cover now in its package and found there; the crop window and the big-picture
   views run in the blast radius below.

**Run** [M]: `cargo test --profile test-checked` of `emusen-crystal`, `emusen-galaxia`, `emusen-library`,
`emusen-library-tool`, `emusen-platform`, `emusen-scrape`, `emusen-sqlite` and `emusen-library-module`: 195 passed, none
failed. WiseMan, the blast radius `blast.sh` named with `GoldenCorpusTests`, `ManPageTests` and `CoreAdapterTests`
added: 3,149 passed, 62 skipped (the picture and bench tools) and 2 failed, in 23 minutes. The two were step 5's and step
6's parity cases, whose tree of `Packages` now held the slot's record with each run's own moment; given this section's
tree, which hides the moment and the build as `Timeless` does, both passed. The golden corpus's `pkg`, `help` and `man`
were given `pkg migrate`'s new line and page, the help listing's pattern edited in place so that it still takes either
shell; `GoldenCorpusTests` and `ManPageTests` passed, 210. The new parity case once compared two manifests whose `seen`
times fell in different seconds; its game is now given one time for both runs.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| Crystal's Rust `states::read` not looking beside a packaged state, reading `states.db` alone | `CrystalParityTests.A_packaged_game_s_state_records_and_own_settings_are_its_package_s_by_either_path_and_kept_when_it_moves_both_ways`: the Rust run found no record of the moved package's slot, the C# one did |
| C#'s `GameSettings.ReadPath` ignoring a package's `settings.json` | the same parity case (answer 3: the C# read the plain file's values after the package had its own) and `PackageCropTests`, both halves (the package's crop not read; the row's answered) |
| The scraper in Rust looking in the media store before the package | `ScrapeMediaParityTests`: a packaged game's 3D box from the store in Rust and from its package in C# |

### 18.8 Left, and for the tester

- ~~**Step 7**, self-contained packages and zip per package, waits for the tester's go.~~ Built 2026-10-25 (§19), its
  own copy and zip keeping every time to the tick. A record is stamped with its
  state's modification time, so a package copied to another machine keeps its records only where the copy keeps the
  times (`cp -p`, `rsync -a`, a zip); step 7's copy should keep them, and a copy that does not loses the records, never
  the states.
- **Q39 [routine]: a state's record in the package is a JSON file beside the state** (§18.1), not a row of a database in
  the package. *Recommended:* keep it; it is read with no SQLite and travels with the state.
- **Q40 [routine]: the route answer's engine stays the plain file's** for a packaged game (§18.2). *Recommended:* keep
  it; the open and every settings call follow the package, and resolving each library file on each route would walk
  the claim index thousands of times.
- **Q41 [routine]: Clear leaves a packaged game's art** (§18.3), as it leaves everything outside the media store.
  *Recommended:* keep it until a player asks Clear of a packaged game; deleting in a package is then its own change.

---

## 19. Built 2026-10-25: step 7

*Measured on the development desktop on 2026-10-25. Every test made its library, packages and stand-in machines in
temporary folders (SyntheticRom, the test core's images, generated sheets, IPS patches written in the test); nothing
was read from or written to the ROM library, and `pkg make` was never run on it. No emulator's or archiver's source was
read: the zip writer is written from PKWARE's APPNOTE.TXT, the format the project's zip reader already follows
(`EmuSen_Galaxia.md` §3.7).* Marks as at the top: what was run is [M].

### 19.1 Self-contained packages: `pkg make --self-contained`

**What it does** [M]: for a game no package claims, the linked package is made as before (§14.4), then made
self-contained; for a game a linked package claims, that package is made self-contained in place (`contained`), its
saves, states, settings and art staying where they are. A converted disc's package is made self-contained by naming
the disc's original sheet, which it claims, and its CHD is copied from `Compressed`.

1. The game is held to `dump` as an opening holds it (§3.4, `claims::open_package`): a changed game is refused with
   both hashes named, a gone one is said gone, and nothing is copied.
2. The game and every companion (a sheet's tracks and `.sbi`, a list's discs and theirs) are copied into the package's
   folder at the places relative to the game's folder that the sheet or the list names them by. A companion outside
   the game's folder, or a file whose place would be one of the package's own (`package.json`, `saves`, `states`,
   `media`, `cheats.json`, `settings.json`), refuses the copy with words.
3. Each copy is written under a hidden name, given its original's modification time, flushed, and held to the SHA-1 of
   the bytes read from the original in the same run, then renamed into place. A copy already there of the same bytes
   is kept, so a stopped `make` run again finishes; any other file there refuses it.
4. **The copy of a cartridge is named as the package's folder** where their extensions agree, which they do unless
   the library's file was renamed after a linked package was made: ES-DE, given the folder, passes the file of the
   folder's name, and RetroArch finds patches by the game file's stem, which `pkg patch` names by the package's own
   name (§15.2). A sheet keeps its name, since its tracks are named by it and the folder by it.
5. The manifest then names the copies (`game.where` and each companion's `where` are `package`, with `size` and
   `seen`), says `"mode": "self-contained"`, and keeps the entries the copies came from as a top-level `sources`, each
   with its `size`, `seen` and `sha1`.

**Cost** [M, a 64 MiB image of a semi-compressible pattern, `cargo run --release`, WiseMan's blast radius running
beside it, so upper bounds]: making the linked package 230 ms, making it self-contained 263 ms (one read, one write and
one read of the copy), `pkg check` of the folder 203 ms.

### 19.2 The library file a package was copied from

**Decided here, and asked as Q42** [C]: a self-contained package claims the files its game was copied from (its
`sources`) as well as its own copy. So:

- the library lists the game once, as the package's copy, as it lists a linked or converted game once (§3.6, §14.3);
- opening the library file by its plain path opens the package's copy with the package's saves, states and settings:
  one game, one save (§3.5, §9 Q14), as opening a converted disc's original opens its CHD (§14.8 Q31);
- the library file's `games.db` rows follow the package's game by the orphan pass's hash rule, as §14.8's Q32
  foresaw, since the claimed file is no longer listed [A; the reclaim itself is unchanged and was not run here].

**Only while it is that file.** Such a claim holds while the library file's size and time are those `sources` records,
or else while its SHA-1 is the one recorded; changed, it opens as a plain game with its plain saves, as a changed
claimed game does (§17). Touched but unchanged, it still opens the package. Gone, nothing changes: the package needs no
library. A self-contained package whose own copy is gone is said missing by `pkg ls` and `pkg check`, and **is never
linked to a library file again unasked**: the search for a moved game (§14.1) now runs only for a game linked in the
library or elsewhere.

### 19.3 A package's zip: `--zip` and `pkg zip`

**What is written** [M] (`EmuSen_Galaxia.md` §3.13): `Packages/<console>/<name>.zip` beside the folder, holding every
file of the folder under one top folder named as the package, the manifest first, hidden files and links passed over.

- **Stored or deflated, as §5.2 decided for cartridges, and discs as their CHDs**: a CHD and what is already compressed
  (`.chd`, `.zip`, `.7z`, pictures, video, `.pdf`) is stored and never passed to the compressor; everything else is
  deflated at level 6, the level §5.2 measured, or stored where deflating would not make it smaller.
- **Every file keeps its time to the tick.** The NTFS extra field (0x000A) carries the modification time as a FILETIME,
  in units of 100 ns, which is .NET's tick: a state's record is stamped with its state's time in ticks (§18.1), so a
  zip that kept seconds would drop every record it carried. The extended timestamp (0x5455) and the DOS fields carry it
  to the second and to two seconds for other readers.
- **No Zip64**, which the reader does not read either: 65,534 files and 4 GiB at most, refused with words beyond.
- Written under a hidden name and renamed into place whole; **a zip already there is never replaced** (Q46).
- `--zip` implies `--self-contained`, since a linked package's zip would hold no game. `pkg zip <package>...` zips a
  package that is already self-contained, as it now is with its saves: the way to carry a package on from a machine
  whose library does not hold its game, as when it came there as a zip (Q45).

**Opening it** [M]: a zip that holds a package, opened by any route (§3.5's resolution, so both runtimes, the shell's
`core` and the library), is unpacked once into `Packages`: beside itself where it lies under `Packages`, straight under
`Packages` where it lies anywhere else, never beside a zip in the library. Every entry is checked against its CRC-32
and given its time, into a hidden folder first, renamed into place whole. From then on the folder is the package: a zip
whose folder is there and holds the same game opens the folder and is not read beyond its manifest, and **a zip is
never unpacked over a folder** (Q44). A zip whose entries climb out of its top folder is refused before anything is
written. The library lists a zip beside its own folder once, as the folder, and a zip without its folder as one entry
shelved by its manifest's `system`.

**`pkg check` of a zip** unpacks it into a hidden folder of `Packages`, checks the package there as a folder is checked,
and removes the folder; the zip is only read.

**Cost** [M, the same 64 MiB package]: writing the zip 748 ms, to 44,145,689 bytes (65.8%); `pkg check` of the zip 447
ms; opening it on a fresh tree, unpacking included, 242 ms, and again, once its folder is there, 0.1 ms. P5 (a zipped
64 MiB cartridge's first launch on the handheld under 1.5 s) is not retired by this: it is the desktop, and a package's
zip, not a game's.

### 19.4 `pkg identity`, and the window's switch

`pkg identity <package> original|patched` (the request op `pkg-identity`) sets the manifest's `identity` (§5.5, §15.4),
as §15.9's Q37 recommended and the tester accepted; `patched` is refused for a package with no soft patches. The *Game
Package* window gains a *Scraped as* row, shown where the package lists patches, with *Scrape as Patched* or *Scrape as
Original* running the same job, and says self-contained packages are such, offering the patch files beside the library
file a self-contained game was copied from.

**What the switch does not change** [C]: which saves a patched game uses. §16 names a patched game's saves by its patch
list's key, and the identity only chooses which hashes the scraper sends; the window's hint says the saves stay the
patch list's either way (Q47).

### 19.5 The surfaces

| Surface | Change |
|---|---|
| The service `emusen.library` | 1.2 → **1.3**: `pkg-make`'s `self_contained` and `zip`, a package's zip among `pkg-check`'s paths, `pkg-identity` (`identity`), `pkg-zip`; the `package` event's `mode`, `zip` (`path`, `bytes`), `copied`; the states `contained`, `zipped`, `set`; the table unchanged |
| The module `emusen.library` | `pkg make --self-contained --zip`, `pkg zip`, `pkg identity`; asks for the service at 1.3; `man pkg` |
| The manifest | stays **1.0**: a top-level `sources` (each with `role`, `where`, `path`, `size`, `seen`, `sha1`), a field the schema already allows, as `patches[].name` was at step 6 (Q48) |
| Galaxia, Rust | `package_zip` (`zip_beside`, `files_of`, `write`, `manifest_in`, `target`, `unpack`, `same_game`, `opened`); `archive::{ZipWriter, copy_entry, STORED_EXTENSIONS, MAX_ENTRIES, MAX_OFFSET}`, `ZipEntry`'s `crc32` and `modified`; `claims::source_holds`, `resolve` opening a package's zip |
| Galaxia, C# | `GamePackages.ZipBeside`, `ZipManifest`, `Managed.SourceHolds`; the managed resolution refusing a package's zip with words |
| The library's scan | a zip beside its package's folder passed over; a package's zip without its folder shelved by its manifest (`scan.rs`, `RomLibrary.ScanManaged`) |
| The library tool | `Making`, `further`, `contain`, `copy_in`, `zip`, `identity`, `check_zip`, `named_or_zipped`; the CHD tool finds a converted disc made self-contained done in its package and checks its CHD there (`convert::chd_of`) |
| Mistress | `GamePackageWindow`'s *Scraped as* row, `SelfContained`, `Identity`, `SetIdentity` |

The platform interface stays **27** and the core ABI is unchanged.

### 19.6 The gate, the tests, and the faults seeded

**The gate** [M]:

1. *A package copied to another folder opens there with its saves, states, settings, art and patches*:
   `CrystalParityTests.A_self_contained_package_carried_to_another_machine_as_its_folder_or_its_zip_opens_there_with_its_saves_states_settings_art_and_patches_both_ways`.
   A self-contained package with an IPS patch is played on the first machine by the Rust runtime (the battery save
   written, slot 2 saved with its record, a setting of the game's own), given a cover, and zipped by the tool. Then a
   stand-in machine, a fresh data root and settings in a temporary folder with a library that lacks the game, is given
   the folder copied with its times; in each runtime the package opens, its patch applied, its slot listed under the
   patched key and its record naming the patched image, the slot loading, the setting read and the battery save the
   first machine's; the two runtimes' answers and the trees they leave are equal; the scan lists the game once and the
   media store finds its cover in the package.
2. *A zip opens the same*: on a third machine the zip alone is put in `Packages`, listed by the scan as one entry, and
   opened by its path in each runtime, which unpacks it beside itself and answers as the folder did; the slot's and
   its record's times came through to the tick. System.IO.Compression, a reader of its own, reads every file of the
   tool's zip back byte for byte. In Rust, `self_contained.rs` carries a package by `cp -p` and by its zip to two more
   trees, every file and its time to the tick alike, and a zip opened again after play never unpacks over its folder.
3. *A copy that lost the times*: on a fourth machine the folder is copied with every time lost: the state's record
   drops out and the state still loads, in both runtimes, as §18.8 said.
4. *The library tree is unchanged after making self-contained packages and zips*: `self_contained.rs` (cartridges, a
   disc's sheet and tracks, a linked package contained, zips written and refused, checks run), the chdman gate's new
   case (a converted disc made self-contained), `GamePackageWindowTests` and the runtimes' case each hold the library's
   every path, size, time and MD5 to its first listing. **P7 holds for step 7.**
5. *`pkg check` passes on both forms*: `self_contained.rs` checks the folder, its zip, a contained cartridge and a
   disc, all `verified`, and the carried copies on the other trees.
6. *The fit audit*: `WindowFitAuditTests` at 1280 × 800 and 1920 × 1200 for the new `GamePackageSelfContained` state (a
   long-named self-contained package scraped as patched) and the `GamePackageLongTitle` state, whose patched package now
   shows the *Scraped as* row, both recorded again in the interface specification; every control reached by the pad.

**Besides the gate**: `GalaxiaParityTests` holds the two halves of the claim index to one answer for a self-contained
package and its source, touched, changed and gone, and holds a zip another archiver wrote to the platform library's
unpacking and the C# half's refusal; `package_zip.rs` refuses a damaged entry and an entry climbing out, and leaves
nothing; `scan.rs` lists a self-contained package once and a zip only where its folder is not; `GamePackageWindowTests`
switches the identity through the job and back, and opens a self-contained package's window over a package the tool
made; the golden corpus records the new command lines and page.

**Run** [M] `cargo test --profile test-checked` of `emusen-galaxia` (94 and 1), `emusen-library` (3, `library` 3,
`library_abi` 6, `routing` 2), `emusen-library-tool` (4, the gate's 8 with `EMUSEN_CHDMAN` naming the pinned chdman,
`packages` 2, `patches` 1, `self_contained` 1), `emusen-library-module` (4), `emusen-platform` (33, `dianash_abi` 4,
`library_tool` 3), and for the readers of the zip reader and the claim index `emusen-cheats` (9), `emusen-crystal` (6,
`crystal_abi` 6, `frame_alloc` 1) and `emusen-scrape` (3 and its integration tests 15): all passing, 209, again on
the tree rebased onto WiseMan. WiseMan, the blast radius `blast.sh` named: 4,016 passed, 62 skipped (the picture and
bench tools) and 1 failed, in 26 minutes; the one was `ScraperTests.Never_more_requests_at_once_than_maxthreads`, known
to fail under load and untouched by this step. Rebased onto WiseMan at `01dabfeed` (Sapphire's S30 and S25 part B
beneath), the same radius: 4,030 passed, 62 skipped and none failed, in 27 minutes. The step 5 parity case §18.8 warned
of passed, unmasked. The golden corpus's `pkg`, `help` and `man` were given the new command lines and page,
`help`'s patterns edited in place so that they still take either shell; `GoldenCorpusTests`, `ManPageTests` and
`NativeSwitchTests` passed, 211.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| `claims::source_holds` holding every source, changed or not | `claims.rs`' sources case: the changed library file still opened the package's copy |
| The zip writer's NTFS time rounded down to the second | `package_zip.rs`' round trip and `self_contained.rs`: the times out of the zip were not the ticks that went in (a record stamped in ticks would have dropped out) |
| The C# scan listing a zip beside its package's folder | `CrystalParityTests.A_self_contained_package_carried_to_another_machine_…`: on the third machine, after the zip was opened, the library held two entries for the one game |

### 19.7 Otherwise than the plan, and found on the way

- **The zip is of the whole package** (§3.4, Q43), not of the game alone inside the folder, since a package carried to
  another machine is step 7's gate and §6.3's promise; the game stays as dumped inside the folder, so the borrow and
  the unpack cache are untouched.
- **A zip is unpacked into `Packages`**, not into `Unpacked` as §5.2 has a game's zip: a package's saves and states are
  written where it lies, and `Unpacked` is a cache whose 4 GiB budget evicts.
- **`File.Copy` keeps a file's time in .NET, on Linux as on Windows**, which the first draft of the test's "copy that
  loses the times" relied on not doing; the test sets the time itself.
- **`pkg zip`** is a subcommand beyond §3.7's (Q45), for the round trip back from a machine whose library lacks the
  game.
- **A copy is named as its package's folder** (§19.1), so that ES-DE and RetroArch agree with the folder and the patches
  even after a library file was renamed.
- **The CHD tool and self-contained packages**: `chd ls`, `chd check` and `chd convert` would have taken a converted
  disc made self-contained for one changed, and converted it again over its package; they now find its CHD in the
  package (`convert::chd_of`).

### 19.8 Left, and for the tester

- **The tester's checks, by hand, not claimed here**: that ES-DE launches a self-contained package (point an ES-DE
  system at `Packages/<console>`, §6.1), and that RetroArch applies a self-contained package's patch, loading the game
  file in its folder with a core that loads from memory (§6.2).
- **Step 8** stays deferred (§9 Q2); **step 9**, `pkg export esde`, is optional and needs the tester.
- **The handheld's runs** of §12.7 are still left; nothing here touched them.
- **Q42 [decision]: the library file a self-contained package was copied from stays claimed** while it is still that
  file (§19.2): listed once, opening the package's copy and saves. *Recommended:* keep it, for one save per game (Q14)
  and as a converted disc's original is (Q31); a player who wants the library file played on its own removes the
  package, or the file changes and plays plainly.
- **Q43 [decision]: `--zip` zips the whole package, not the game inside the folder** (§19.7). *Recommended:* keep it;
  storing the game zipped inside the folder to save space would cost each first launch an inflate and fill `Unpacked`
  (§8), and is a choice of its own if storage ever asks for it.
- **Q44 [decision]: a zip is never unpacked over its folder.** A zip brought back from another machine with newer saves
  does not replace the folder still here; the player moves the folder aside, and opening the zip then unpacks it.
  *Recommended:* keep it; choosing the newer of two copies, or merging them, is a step of its own.
- **Q45 [routine]: `pkg zip <package>`**, beyond §3.7. *Recommended:* keep it; `pkg make --zip` names a library game,
  and a package that came as a zip has none here.
- **Q46 [routine]: a zip already there is never replaced** by `--zip` or `pkg zip`. *Recommended:* keep it; the words say
  to move it aside.
- **Q47 [routine]: the identity switch changes the scraper's identity only**; a patched game's saves follow its patch
  list (§16). *Recommended:* keep it.
- **Q48 [routine]: the manifest stays 1.0** with `sources`, a field the schema allows. *Recommended:* keep it, as for
  `patches[].name`.
- **Q49 [routine]: no *Make Self-Contained* or *Make Zip* in Mistress yet**; the window shows a self-contained package
  and switches the identity. *Recommended:* add both buttons to the *Game Package* window when a player is to carry a
  package without the shell.

## 20. Built 2026-10-28: where CHDs go — a chosen output folder, and beside the original

*Approved by the tester on 2026-10-28 as two extensions of the CHD tool, not components of their own: a persistent
output folder, and an opt-in mode that writes each disc's CHD beside its original in the library. Measured on the
development desktop on 2026-10-28. No batch was run on the ROM library or the NAS: every test made its own discs in a
temporary folder, as §12.4 and §13.5 make them.* Marks as at the top: what was run is [M].

**The form.** Both are extensions of existing surfaces (the rule of `EmuSen_Surfaces.md`): one optional request field
each in the `emusen.library` service, which becomes 1.4 with its table unchanged; one folder setting in
`AppSettings`; one rule in the claim index; two rows in the window. Nothing gains a second job: the tool still converts,
the scan still lists, the window still fronts the tool.

### 20.1 The output folder (decided 2026-10-28)

**The setting.** `AppSettings.ChdOutputDirectory`, blank by default, which is `<home>/Compressed` as before (§4.2). It
holds the rules of every folder setting (`EmuSen_Config_Reference.md` §7.7): `~` and `~/` are the home, a relative path
or a file is refused with words, and a folder not yet there is made at the first disc. `SettingFolders.ChdOutput` is
the C# reading of it and `Context::destination` the tool's, both over `SettingFolders.Checked`'s one rule.

**Where it is reached.**

- The Compressed Discs window's *Output folder* row, a LunaP `PathPickerRow` that can be typed as well as browsed, as
  Preferences' folder rows are, so that Game Mode and big picture, where no platform picker opens, reach it with the
  pad and the on-screen keyboard. A pick is saved at once and the library listed again, so that the tool says at once
  whether the folder can be used.
- `chd convert --output <folder>` for one command: a shell path, so `/` is EmuSen's home and the walled garden holds
  (`man hier`); a folder outside the home is named in the setting.
- `appsettings.json` by hand.

**The overlap refusal, both ways.** Before anything is written, `Context::destination` refuses a chosen folder that
lies inside the library or holds it, by the same canonical comparison `checked_library` makes of `Compressed`,
`Packages`, `Logs` and `Unpacked` (§12.3), which it still makes. The words: "The output folder (…) and the library (…)
overlap: the library is never written, so nothing was done." `ls` and `check` are refused alike, as they already were
for an overlapping `Compressed`: the window learns of a bad folder from its listing, and keeps the row shown so that it
can be chosen again.

**The relative structure is kept**: `<output>/<the library-relative path, with .chd>`.

**The package names a CHD outside `Compressed` by its full path** (`where: "absolute"`), and one inside it as before
(`where: "compressed"`). So a later change of the setting leaves every earlier CHD found where it lies, and the
default's packages are written byte for byte as before.

**A disc is done wherever its package names its CHD** (§12.2's rule, generalised). It was "the package names this
CHD under `Compressed`"; it is now "the package names a CHD, at the recorded size, time and SHA-1, from originals that
have not changed", the place read through Galaxia's `claims::place`, which resolves every `where`. Changing the folder
therefore converts nothing again, and `ls` and `check` find each CHD in the folder it went to. Under the default every
package names `Compressed`, so the default's answers are unchanged.

### 20.2 Beside the original (decided 2026-10-28)

**What it writes.** For each disc, `<stem>.chd` in the library folder of its original: `psx/Rayman (USA)/Rayman
(USA).cue` gets `psx/Rayman (USA)/Rayman (USA).chd`; a zip or a lone track the same, beside it. No folder is made in the
library. The package stays under `Packages`, its game `where: "library"`.

**A choice for the batch, not a setting.** *Decided on 2026-10-28, from the existing settings:* §4.5 made the one other
operation that would write the library (*Replace originals*) "never a setting that persists", and this follows it. A
persistent mode would let a script's plain `chd convert`, or a batch started months after the setting was forgotten,
write into the library. So the shell takes `--beside` per command and reads nothing of it from config, and the window's
*CHDs go* row opens on *In the output folder* every time. The output folder, which writes nothing in the library, is
the persistent half.

**Named before the batch starts.**

- In the window, while *Beside each original, in the library* is chosen, a line under the choice says: "This writes
  into the library: converting adds up to *n* CHD files to `<library>`, each beside its original as `<name>.chd`. No
  original is changed, moved or removed, and a disc whose name is already taken is left as it is." *n* follows the
  discs the batch would name, counted from the listing: those not converted whose name is free (the listing's new
  `taken`). The plain statement at the top changes with it, and the summary after the batch counts the files added.
- In the shell, the start event of a batch beside the originals carries `beside` and `adds`, computed before the first
  disc, and `chd` prints them as its first line: "Converting 3 disc(s) beside their originals, writing into the
  library …: up to 2 CHD file(s) added, no original changed, moved or removed, a name already taken skipped."
- "Up to": a disc skipped for its form (§12.2) or failing a check adds nothing, and is known only when it is reached.

**The originals.** Never removed, renamed or overwritten: the tool writes no file of a disc's but its new CHD. The
before-and-after check of the originals' sizes and times (§4.6) still discards a result whose original changed.

**A name already taken** is skipped and reported ("psx/X.chd is already in the library, so nothing was written: a file
there is never replaced"), checked before chdman runs, and again at the rename, which refuses to replace a file:
`renameat2` with `RENAME_NOREPLACE` on Linux, and where a file system answers `EINVAL` to it, an existence check and a
plain rename (the window between them is the one race left, and a player writing that very name in that instant is
what it would take).

**The temporary file.** chdman writes `<stem>.chd.partial-<process>` in the same folder, step 2 of §4.5's design, and it
is renamed into place only after §4.3's three checks pass. A failed check, a stop or a refused rename removes it. A
partial a killed process left is removed an hour later, the next time that disc is converted: only a file of that exact
name, `<stem>.chd.partial-` and digits, beside that disc. Should the package not be written after the rename, the CHD
just placed is taken out again, so that the library never holds a CHD without its record (in `Compressed`, as before, it
stays).

**Free space** is checked at the library folder, as at any output.

### 20.3 One game in the library: the verified CHD beside its original

**What already did most of it.** The scan lists a package once and no file it claims (§3.6, §14.3), and a converted
disc's package claims its originals (`container.sources`). A CHD written beside its original is its package's game, so
it is claimed too. With the record in place the library and big picture therefore show the game once, as the package,
launching the CHD, with no new rule in the scan; big picture takes the same scan's entries.

**"Verified" is the tool's record, not the name.** A matching `<stem>.chd` beside a cue sheet proves nothing: it may be
the player's own, another dump's, or half a file. So the claim index (`claims.rs` and its twin `GamePackages`) holds a
package whose game is a CHD in the library (`container.type` is `chd` and `game.where` is `library`) to its record: the
originals are claimed only while

1. its container records each of the three checks as `passed`, and a SHA-1;
2. the CHD has the size and time recorded in `game.seen`;
3. every original in `container.sources` has the size and time recorded with it.

These are the same facts §12.2's "done" reads, taken with a `stat` of each file and no read (§3.6's rule that no file
is read for a claim). Where any fails the package keeps its game, so a played CHD keeps its saves, but claims no
original, and the original is listed beside it.

**Without the record** (the package deleted, or never written): the CHD and the original are two plain entries, each
opening as itself, and a conversion beside the originals leaves that disc as it is, its name being taken. Nothing is
hidden on a name alone, and the originals stay on disk in every case.

**The default is unchanged.** A CHD in `Compressed` or a chosen folder is not in the library, and its package claims its
originals as before (§14.1), checked or not; the rule above applies only to the new case. Q53 asks whether it should.

### 20.4 The surfaces

- **`emusen.library` 1.4** (`emusen_library_tool.h`): the request's `output` (convert, check and ls) and convert's
  `beside`; `beside` with `output`, or `beside` on another op, is refused `PARSE`. Events gain fields only: the start
  event's `beside` and `adds`; a disc event's `where` (the recorded place's `where`) for convert and ls, and the listing's
  `taken`. The module requires 1.4, and ships in the same library.
- **`AppSettings.ChdOutputDirectory`**, in both halves' schema (`models.rs` 62 fields).
- **The claim index**: §20.3's rule, in both halves, held equal by `GalaxiaParityTests`.
- **`CompressedDiscsWindow(library, settings)`**: the settings it saves the folder to; `ChooseOutput`, `Beside` and
  `BesideAdds`.

### 20.5 The window

From the top, the window of §13.2 gains two rows after *Progress*:

1. **CHDs go**: *In the output folder* or *Beside each original, in the library*, a LunaP `Dropdown`, and under it the
   line of §20.2 while the second is chosen.
2. **Output folder**: the `PathPickerRow`, its placeholder "(default) `<home>/Compressed`". Hidden while the batch writes
   beside the originals, since it does not apply.

Both are hidden while a batch runs, as §13.2's choosers are, so that the choice cannot change under it. Desktop and big
picture reach them by the same pad walk; `WindowFitAuditTests` gained the state `CompressedDiscsBeside`.

### 20.6 The gate, the tests, and the faults seeded

**Rust** [M], `cargo test --profile test-checked` with `EMUSEN_CHDMAN` naming the pinned chdman:

- `emusen-library-tool`'s gate, 10, two of them new. *A chosen output folder*: the setting used, a request's `output`
  replacing it, the package naming the CHD by its full path, `ls` and `check` finding it with the setting cleared, a
  folder inside the library and one holding it each refused with the words of §20.1 and nothing converted, a relative
  value refused, and the library's listing (every file's path, size, time and MD5) the same after. *Beside the
  original*, over a two-file dump, a sheet whose `<stem>.chd` is already a file of the player's, and a disc whose audio
  file ends in part of a sector: the start event's `adds` is 2; the first converted, its package's game
  `where: "library"`; the second skipped and its file's bytes unchanged; the third failed; the library's listing after is
  the listing before plus the one CHD, no partial anywhere; `ls` gives `where` and `taken`; a second batch calls the
  first `already`, `check` verifies it in place, and `beside` on `check` is refused.
- `emusen-galaxia`, 95: `claims`' new case holds §20.3's rule (verified; a check not passed; the CHD at another time;
  recorded afresh; an original changed), and the schema's count is 62.
- `emusen-library-module`, 4: `--output` resolved as a shell path, `--beside`, both refused together, `--beside` not
  an option of `ls`, and the start and listing lines beside the originals.

**WiseMan** [M], headless, with the pinned chdman beside the tests:

- `CompressedDiscsOutputTests`, 3, over generated sheets. The output folder chosen in the window is saved to
  `appsettings.json` and used, Compressed left empty; a folder inside the library, then one holding it, is refused in
  the window's words with *Convert* disabled and the row still offered; cleared, the setting is blank and the disc is
  still found. Beside the originals: the window opens on the output folder; the line counts 2 to add (one disc's name
  taken); after the batch one disc converted beside its original, one left as it is, one failed, the summary counting
  the one file added, the player's file unchanged, and the library's listing the one before plus that CHD, every
  original's path, size, time and MD5 as they were, no partial left. Duplicate hiding, by the managed scan and the
  library's: one entry, the package's, launching the CHD; with the CHD's time moved, both entries; with the package
  removed, both entries, neither a package's; the originals on disk throughout.
- `GalaxiaParityTests`' new case: §20.3's three states give the same index from both halves.
- `CompressedDiscsWindowTests`, the theory gaining four phrases (34 cases).
- `WindowFitAuditTests`: the six CompressedDiscs states at both sizes, `CompressedDiscsBeside` new; their twelve
  recordings re-recorded (the two new rows), and a second run compared equal. No other state's recording changed.

**The blast radius** `blast.sh` named, run once: see the commit that records it.

**Faults seeded**, one at a time, each put back:

| Fault | Caught by |
|---|---|
| §20.3's rule without its third condition, the originals' times | `claims`' new case: an original changed still claimed |
| beside the originals, the check for a taken name removed and the rename made a plain one | the gate's beside case: the player's `One.chd` replaced, the disc not skipped |
| the overlap refused one way only, a folder that holds the library let through | the gate's output case: the folder holding the library was converted into, with no refusal |
| a failed check's partial left beside the original | the gate's beside case: the library's listing gained the partial |
| the window's count of files to add not leaving out a taken name | `CompressedDiscsOutputTests`: "adds up to 3" where 2 was due |

### 20.7 Left, and for the tester

- **Q50 [routine]: beside the originals is a choice for one batch**, not a setting (§20.2), following §4.5.
  *Recommended:* keep it; a setting would let a forgotten choice or a script write the library.
- **Q51 [routine]: a disc already converted elsewhere is `already` in a batch beside the originals** (§20.1's done
  rule): one disc, one package, one CHD. Moving a CHD from `Compressed` into the library is not offered.
  *Recommended:* keep it; a player who wants it beside deletes the CHD in `Compressed`, and the next batch writes it
  beside the original.
- **Q52 [decision]: a CHD already beside its original with no record is never adopted.** It is listed beside its
  original and a batch leaves the disc as it is (§20.3). *Recommended:* a later extension, `chd adopt`, running checks
  2 and 3 of an existing CHD against the original and writing the package without writing the library.
- **Q53 [decision]: §20.3's rule applies only to a CHD in the library.** A CHD in `Compressed` or a chosen folder
  hides its originals while its package exists, checked or not, as since §14. *Recommended:* keep the default as it
  is, as the approval asked; holding every converted disc to its record is a one-line change if the tester wants the
  two alike.
- **Q54 [routine]: `chd convert --output` takes a shell path**, inside EmuSen's home; a folder elsewhere is named in the
  setting or the window. *Recommended:* keep the walled garden (`man hier`).
