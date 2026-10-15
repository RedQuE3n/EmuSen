# Diamond_Players — stage 11, the PlayStation to players

*Written 2026-10-11, at stage 11.* The plan's stage 11 row (`Diamond_Plan.md` §7) and the four framework pieces stage
7 proposed (`Diamond_CDROM.md` §8.4), approved by the tester on 2026-10-11: archives, routing by content, the runtime's
companions with `.m3u` and the disc change, and the track-files constant in the core ABI's header. Each piece is an
extension of an existing surface, written up where that surface is recorded; this page records the stage, its gate
and what it found. The marks are the plan's: **[M]** measured on 2026-10-11 with the tool named, **[R]** read, **[A]**
argued, **[C]** a choice.

No dispute was opened; D-41 and later stay stage 10's and stage 12's.

---

## 1. Routing by content

**The surface** (`EmuSen_CoreAPI_Extensions.md` §8.4, the runtime's): `SystemEntry` gains `Recognise`, a function of
an `IMediaProbe` (the file's path, the image's own name where it is in an archive, and its first bytes read once and
kept) to a `Recognition` (the system, and the serial and market where the pack can read them). `SystemRouting.Decide`
is the one decision among the systems that take a file's extension: the only one; the one whose recogniser alone
claims the file; the one its folder is named for, by ES-DE's system name; else none, with words naming the candidates
and the folders that would decide. It is cached by path, length and time. `CoreCatalog.Route` gathers the candidates,
the consoles' rows and, where development cores are shown, the systems an engine offers in development, and every
caller asks it: `ConsoleForRom`, `ShelfFor`, `CoreForRom` (which Mistress's former `ByExtension` callers use),
`DiscoveredConsoleForFile` and `SystemOfFile`, the factory's choice of engine, and the battery folder. The player's
chooser, the decision's fifth step, is not built: a file no step decides opens nowhere, its words saying why [C].

**The recognisers.** The PlayStation's is `PsxSystem.IsDisc` (the licence text's first line in sector 4 of an `.iso`
or a raw track) with the market from the licence's second line and the serial from `SYSTEM.CNF`'s `BOOT` line,
`cdrom:\\SLUS_007.70;1` read as SLUS-00770 (`Diamond_Plan.md` §4.1), through the runtime's ISO 9660 lookup
(`Crystal/Media/DiscSectors.cs`), which reads the root directory of a raw or a user-data track; the disc crate keeps no
filesystem. The Genesis's claims anything that is not a compact disc's image, a raw track's sync or an `.iso`'s volume
descriptor, which is Nephrite's own rule (`media.rs`); the Sega CD's claims `SEGADISCSYSTEM` at its first sector's data.
The probe reads a `.cue`'s first track, an `.m3u`'s first disc and an archive's image entry without unpacking it; a
`.chd` is not read, as only the PlayStation takes one.

**Measured** [M, `DiamondPlayersTests.A_bin_goes_to_the_console_its_content_is_and_an_iso_and_a_cue_to_the_playstation`]:
a synthetic PlayStation track and a synthetic Genesis cartridge, both `.bin`, go to their consoles; the disc's
recognition is `psx`, EMUS-00001, Europe; a raw track of sync patterns and nothing else goes nowhere, with words, and to
the PlayStation in a folder named `psx`; with development cores shown a synthetic Sega CD `.iso` goes to the Sega CD and
opens on Nephrite as `mcd`, a PlayStation `.iso` beside it to Diamond. The registration golden
(`EmuSen_CoreAPI.md` §25) changed only by the PlayStation's lines, with the catalog consoles' answers unchanged, which
retires `EmuSen_CoreAPI_Extensions.md`'s P3 for the cartridge consoles on WiseMan's synthetic images; the tester's
library was not walked for it [M].

**One owner for each fact.** The PlayStation's pack now lists `.psexe`, `.psx`, `.cue`, `.bin`, `.iso`, `.chd` and
`.m3u`; the core's info lists the same but `.m3u`, which no core receives, held to be the pack's subset by
`DiamondTests`. `.pbp` stays declared and not routed.

## 2. Image sets, `.m3u` and archives

**The constant** (`EmuSen_CoreAPI.md` §6.23): `EMUSEN_FILE_MEDIA`, 65,536, in `emusen_core.h` and `emusen-native`'s
`flags`, the header's minor 1.1, `abi/v1/baseline.txt` regenerated with every new line tagged @1.1, and
`emusen-core-abi-check` holding the header, the Rust and the baseline to one another on the four triples. Diamond's
`MEDIA_FILE` is now `flags::FILE_MEDIA`.

**The companions** reach the runtime through the platform library (`EmuSen_RustPlatform.md` §22, interface 8), so a
sheet has one parser, the disc crate's: `ImageSets.FilesFor` names a sheet's files and an `.sbi` beside a sheet or a
lone track, `CoreEngine.LoadRom` passes them as `FILE_MEDIA + n`, and the kit does the same (`EmuSen_CoreAPI.md` §28.7).
An `.m3u` never reaches the core: the runtime passes its first disc and keeps the list as the game's `Discs`. **The
library shelves a set once** (`ImageSets.Shelved`): a file that a sheet or a list beside it names is that game's.
Battery files and states are named after the file the player chose, the `.m3u` for a set, so a game of several discs
keeps one card.

**Archives** (`EmuSen_Galaxia.md` §3.7, `Crystal/Media/ArchiveImages.cs`): a `.zip` is listed by its image entry, a
sheet first and else the first lone image of an extension a console takes, routed by that entry's content, and
unpacked once into Galaxia's `Unpacked` folder for play, the chosen image renamed with the archive's stem so that saves
follow the archive. Nothing is written beside the archive; the folder keeps the archives opened last up to 4 GiB and
removes partial unpackings an hour old. It is a component of the runtime's media module rather than of Galaxia: Galaxia
names the folder in both its halves and reads no archive, and the cores never see a path. **Measured** [M,
`DiamondPlayersTests.A_zipped_sheet_and_a_zipped_track_unpack_beside_galaxia_never_beside_the_archive`]: a zip of a
sheet and its two tracks and a zip of a lone track open, their shelves PlayStation, their images under `Unpacked`
named after the archives, the library's files unchanged in name, length and time; the clean-up rule removes an older
folder past the size and a stale partial one, keeping the one in use.

## 3. The disc change

**The surface** (`EmuSen_CoreAPI.md` §6.23, minor 1.1): `MEDIA`, bit 18, with `emusen_core_media_lid` and
`emusen_core_media_insert`, `emusen_media` (48 bytes), `EMUSEN_MEDIA_RESTORE`, three statuses (−265 to −267) and machine
info's `drives`, written by `emusen-native` from `Core::drives` so that no core's `MachineInfo` changed shape; the C#
host's table and structs, held to the baseline by `CoreAbiTests`; the kit's C16.

**In Diamond.** One drive, 0, "CD-ROM drive", kind `cd`. `media_lid` opens or closes the controller's lid as stage 7
modelled it (the drive stopping, INT5 with error 08h, the shell-open flag, the spin-up and TOC read at closing);
`media_insert` puts a disc in the drive while the lid is open, or with `RESTORE` for a state about to load, or empties
it, refusing a PS-EXE as no disc. **The state, version 9**, ends with the disc's fingerprint, FNV-1a over its tracks'
starts and kinds, its lead-out and its raw sectors 4 and 16 to 23, never 0, 0 being an empty drive; `state_load` refuses
another disc's state with `EMUSEN_WRONG_MEDIA` and changes nothing; a version 8 state, which has no fingerprint, loads
unheld. **In the runtime**, `CoreEngine.ChangeDisc` opens the lid, inserts the disc and closes the lid one second of the
game's own time later (`LidOpenSeconds`), the interval ps1-tests' `disc-swap` leaves (`Diamond_CDROM.md` §9.1) [C]; a
state that the core refuses as another disc's is loaded again over each other disc of the set, put back with
`RESTORE`, and the drive is restored where none is the state's. That replaces the record beside each state that
`EmuSen_CoreAPI_Extensions.md` §3.5 planned: the core's fingerprint is the record, and no record gained a field [C].
Mistress's game menu has **Change Disc** for a game of more than one disc (`EmuSen_Settings_Reference.md` §4.106).

**Measured** [M]: Diamond's `a_disc_changes_through_the_lid_and_a_state_keeps_to_its_disc` (the closed lid's
refusal, the swap, `WRONG_MEDIA`, `RESTORE`, the empty drive, a program's empty drive); WiseMan's
`An_m3u_changes_its_discs_through_the_lid_and_a_state_brings_its_own_disc_back` on two synthetic discs (the shelf
showing the list alone, Change Disc's lid open then closed 60 frames later, a state of each disc loading over the other
with its own disc put back); C16 on every disc of the kit's sample (§6). `EmuSen_CoreAPI_Extensions.md`'s P5 is not
tested: no multi-disc game is among the tester's discs.

## 4. The cheats, the settings and the memory cards

**The cheats** are `EmuSen_Cheats.md` §10: the GameShark (Game Buster and Action Replay being its format) and the
Xploder with its decryption, told apart by psx-spx's first-digit table, decoded into RAM writes and the cheat model's
tests with `IfLess` and `IfGreater` added; the refusals named; 97.5% of the database's PlayStation codes that are codes
load. The pack's cheat systems are `Sony - PlayStation`, which `cheat db prune` keeps.

**The settings** were built at stages 1 to 7; stage 11 gives them their tabs and checks their effects: *Region* (the
market, so the rate, the height and which BIOS file is asked for, `the_settings_choose_the_region_the_gpu_and_the_ports`),
*Graphics chip* (the console model's one difference a program can see, GPUSTAT's version, `Diamond_GPU.md`), the two
ports' pads and multitaps (the ports machine info lists) and the two memory card slots (which battery files exist). The
plan's "card mode" is the slots' setting; a card shared between games (`EmuSen_CoreAPI_Extensions.md` §9) is not built,
per game being the decided default. *Built 2026-10-15 as a card mode of the runtime's, Per game, Shared and Named
cards, with Per game still the default (`Diamond_Cards.md`).*

**The cards round-trip through their battery files** [M,
`DiamondPlayersTests.A_card_a_program_writes_round_trips_through_its_battery_file`]: a program of the test's own writes
one frame to the card in slot 1 with psx-spx's write command over SIO0, the card answering 5Ch 5Dh 47h; the runtime
writes `Saves/PlayStation/<game>.mcr`, 131,072 bytes with the frame at its offset; a byte changed in the file is read
back by the next load. The card is the raw form other emulators store, as stage 6 measured.

## 5. The pads, the shelf, the CRT

- **The drawings**: LunaP's `ControllerLayout.PlayStation` and `DualShock` (`docs/LunaP.md` §198.13 in the toolkit,
  branch `diamond-s11`), shown by the pad the player's port is set to; the face buttons mapped position for position
  (Triangle X, Circle A, Cross B, Square Y). The DualShock's sticks are shown by their knobs, as the modern pad's are,
  so its stick directions have no regions of their own; drawing direction bands around both sticks was tried and ran
  three labels' lines more than 150 pixels over the drawing, which the toolkit's tests refuse. The Analog button is
  drawn and binds to nothing, being an extra with no RetroPad control.
- **The shelf**: the PlayStation's row, big picture's `psx` with its year 1994 and ScreenScraper's id 57 (to be
  confirmed at a first scrape), its cheats, its tabs (`EmuSen_Settings_Reference.md` §4.105).
- **The CRT** (`EmuSen_CRT.md` §20.6): the PlayStation's row from its clocks and Diamond's line lengths, which waited on
  D-0's clock and now follows the core's provisional line.

**The fit audit** [M, WiseMan's `WindowFitAuditTests`, 129 cases]: the PlayStation's Graphics Settings, Shader Settings,
Active Cheats and Controller Bindings tabs, the DualShock and the digital pad, at 1280×800 and 1920×1200, with the
sidebar and big picture's carousel with the PlayStation shelf, nothing cut off or overlapping; every case of the audit
passed, the four that failed on WiseMan before this stage (Graphics Settings' Genesis tab at both sizes and the
big-screen status line) among them.

## 6. The kit on the tester's discs

`kit/psx-sample.txt` names nine of the tester's discs, every form the library holds; the copies are in the probe's
cache, the library only read to make them. With the kit's image sets and memory estimate (`EmuSen_CoreAPI.md` §28.7),
600 frames, no firmware folder, the open kernel booting them [M, 2026-10-11]:

| Disc | Form | C4 C6 C7 C8 C14 C16 |
|---|---|---|
| Ape Escape | a sheet, one track | pass |
| Castlevania: Symphony of the Night | a sheet, two tracks | pass |
| Crash Bandicoot | a lone dump | pass |
| Spyro the Dragon | a lone dump | pass |
| Star Wars: Jedi Power Battles | a sheet beside its track | pass |
| Rayman | a sheet of 51 tracks | pass |
| MediEvil | a "(Track 1)" file alone | pass |
| Tekken 3 | a sheet of three tracks, unpacked from the library's zip | pass |
| Silent Hill | a sheet, unpacked from the library's zip | pass |

The core's cases (C1–C5, C9–C13, C15) pass on Ape Escape. 42.8 s, nine discs at once, 8.82 GB peak resident under a
10 GiB limit. The first run failed C4 on every sheet: the kit's firmware rule created with no files, the companions
among them; it now keeps them (§28.7 there).

*2026-10-12*: the passes of C7 and C8 above were those of a save at frame 300, which no disc of the sample reaches in a
480-line picture. At 1,800 frames, the save at 900, four of them failed, by a defect present on the day of this run;
it is fixed, and the sample's command is now 1,800 frames (`Diamond_Native.md` §23).

## 7. The gate, item by item

| Item | Result |
|---|---|
| The kit's C1–C15 on the tester's discs, corpus form | 9 of 9 discs and the core's cases pass, with C16 (§6) |
| Cards round-trip through the battery files | the program's frame saved, read back and changed (§4) |
| The fit audit with the PlayStation's tab, drawings and shelf at both sizes | every case passes (§5) |
| The cheats decode (G4) | `PsxCheatFormatsTests`, 21, and the database's decode (§4) |
| A `.zip` and a lone `.bin` from the library's forms through Mistress's headless path | `PlayStationLibraryTests`: each opened from Mistress's library with the pad, the game running as the PlayStation on Diamond, its path the library's file, the library unchanged |

G4 is met. G9 was met the same day on another branch (the cross-kernel card exchange and `firmwarecheck.py`).

## 8. Offered to players

*Decided 2026-10-11: with stage 11's gate and G9 met, Diamond is offered to players*, as the plan's §8 says and as the
Genesis was (`Nephrite_Native.md` §36). Its row of `RustCores.props` no longer carries `InDevelopment` and its row of
`rust-cores.yml` no `development`, so its sidecar is written unmarked and every build, a player's among them,
discovers it. The core's info names its three BIOS files optional, the open BIOS replacing each with the accuracy cost
`v1.rs` states. The tests that switched development cores on to reach the PlayStation run with them off;
`DiamondTests.A_player_is_offered_the_playstation` replaces the test that it was not; the registration golden is
re-recorded, its difference the PlayStation's lines alone; Nephrite's and the firmware overview's tests name the
PlayStation beside the Genesis, `.iso` now being the PlayStation's while the Sega CD is in development.

## 9. Tests run (2026-10-11)

- **Rust**: `emusen-native`, 60; `emusen-core-abi-check`, 4, and the check itself against the 1.0 baseline (407 facts on
  four triples); `emusen-core-conform`, 7 + 15 (with stable cargo, whose layout the tests' library paths assume; the
  private nightly's new build layout moves the test binaries and breaks those paths, which is not this stage's); the
  platform's `emusen-galaxia` and `emusen-platform`, 82 + 24; Diamond, 146 + 56.
- **WiseMan**, by area: the PlayStation and the runtime (`DiamondTests`, `DiamondPlayersTests`, `PlayStationLibraryTests`,
  `PsxCheatFormatsTests`), the ABI and the adapter (`CoreAbiTests`, `CoreAdapterTests`, `CoreDiscoveryTests`, the
  registration golden), every engine whose library was rebuilt (VenusRT, MoonRT, MercuryRT, Nephrite), the cheats,
  the library and its shelves, Controller Bindings, the firmware page, scraping, the CRT's console row, and the whole
  window fit audit: the last full pass of those areas was 1,383 of 1,394 with 8 skipped, then the areas the offering
  touched again, every one passing. Galaxia's tests on both halves (`EMUSEN_GALAXIA_NATIVE` unset and 0), 260 each,
  after `GalaxiaParityTests` learnt the new directory.
- **LunaP**, branch `diamond-s11`: the whole suite, 1,586, the API baseline approved for the two layouts and for two
  members the `crt-choices` merge had added without approving.

- **After 1.2 merged in** (2026-10-12, the read watches taking minor 2 and bit 19, the disc change keeping minor 1
  and bit 18, `EmuSen_CoreAPI.md` §5.5): `emusen-native`, 63; `emusen-core-abi-check`, 5, and the check itself, 416
  facts on four triples, passing against this branch's 1.1 baseline and against WiseMan's 1.2 baseline once
  `json info.systems[].development`, the line both minors added, kept the `@1.2` it was published with; the kit, 7 + 15; Diamond, 148 + 57; WiseMan's
  `CoreAbi`, `CoreAdapter`, `CoreDiscovery`, `Diamond`, `PlayStation` and registration filters, 93; Galaxia's and
  the stage's runtime tests on both halves, 216 each; the cheats' and the tabs' tests, 79.

**A build trap met on the way.** Running `dotnet build` once with the private nightly's cargo and once with the
system's put two layouts of artifacts into one target folder, and a library newer than its sidecar's was copied to the
outputs, so discovery refused every v1 core's library as "not the library its sidecar describes". Building only with
the toolchain the build names, after removing the mixed artifacts, cured it; nothing was changed for it.

## 10. What waits

- The player's chooser for a file no recogniser decides (§1); a `.chd` probe; Pharaoh's own extension table.
- ~~A card shared between games (`EmuSen_CoreAPI_Extensions.md` §9)~~ built 2026-10-15 (`Diamond_Cards.md`); `ChtFile`'s RetroArch `cheat_type` 4 to 7, whose
  "next code" is the import's.
- `.pbp`; P5 with a multi-disc game; the DualShock's rumble (§4 there).
- The DualShock's stick directions as regions of their own, if a drawing can give them a way out (§5).
- Input passed to a v1 machine on the emulation thread, so that no call overlaps another (§12.5).

## 11. Reading log, and exposures

Read for this stage: psx-spx `stable` `6d7d1bc`, `cheatdevices.md` lines 1–48 (the devices and the first-digit
table), 282–347 (the Datel format) and 833–905 (the Xploder's format and decryption); the libretro database's
PlayStation files for their codes' forms and counts; the project's own pages and code. ScreenScraper's system id was
written from what is known of its list and is to be confirmed. No emulator's source, no BIOS, no BIOS listing, no
part of psx-spx's "BIOS Patches" or of `Diamond_References.md` §4 and §6, and nothing DOCS.txt excludes was opened;
DOCS.txt itself was read only for its list of exclusions. The player's BIOS folder was not read, and Diamond's
`kernel/` and `kernel-harness/` were not changed.

## 12. The launch path, on the tester's library and the handheld (2026-10-12)

*Written 2026-10-12, after the tester ran WiseMan `f5c30e69` on the handheld and reported "slow, and many games don't
work or launch".* This section concerns only the path from the library to a running game: the scan, the shelf, routing
by content, unpacking an archive, a sheet's companions, a game in a subfolder and the core's creation. Games that start
and then stop, and the speed of emulation, are recorded elsewhere. The marks are the page's.

### 12.1 The seven games, in the forms the library holds them

Each was copied as it is (the archives not extracted, the subfolder whole) into a scratch library under the probe's
cache, the library itself only read, and driven through Mistress's headless window by
`PlayStationLibraryTests.The_testers_discs_list_route_and_open_from_the_library`, opt-in by
`EMUSEN_DIAMOND_LAUNCH_LIBRARY` (with `EMUSEN_DIAMOND_LAUNCH_HOME` for the unpacked copies and
`EMUSEN_DIAMOND_LAUNCH_DEVELOPMENT=1` for development cores shown): the scan, the list, the pad's A on the entry, 120
frames [M]. Before and after the fixes of §12.4 the table is the same:

| Game | Form | Listed as | Routed | Opened | 120 frames |
|---|---|---|---|---|---|
| Ape Escape | a sheet beside its track | the sheet; the track not listed | PlayStation | Diamond, `psx`, the library's path | yes, 3.2–3.3 s |
| Castlevania: Symphony of the Night | a zip, the sheet and two tracks in a folder inside it | the zip | PlayStation | Diamond, `psx` | yes, 3.2–4.5 s |
| Crash Bandicoot | a lone track | the track | PlayStation | Diamond, `psx` | yes, 2.9–3.1 s |
| Rayman | a subfolder holding a sheet and its 51 tracks | the sheet, inside its folder | PlayStation | Diamond, `psx` | yes, 2.7–3.4 s |
| Silent Hill | a zip, the sheet and one track in a folder | the zip | PlayStation | Diamond, `psx` | yes, 3.1–4.0 s |
| Spyro the Dragon | a lone track | the track | PlayStation | Diamond, `psx` | yes, 2.9–3.4 s |
| Tekken 3 | a zip, the sheet, three tracks and a page of text in a folder | the zip | PlayStation | Diamond, `psx` | yes, 3.2–4.9 s |

The times run from the pad's press to the 120th frame on the bench, unpacking included where nothing was unpacked
before; the library's files were unchanged in name, length and time after every run, and the result was the same with
development cores shown [M].

### 12.2 The library's other forms

The rest of the PlayStation folder was read by names, sizes and an archive's list of entries, and a sheet's `FILE`
lines were compared with the names beside them; no other game was opened or run [M]. The handheld's SD card holds the
same forty files and the same subfolder, name for name and byte count for byte count [M].

- **Fifteen more lone tracks**, five of them a "(Track 1)" file with no sheet and no other track.
- **One more sheet beside a single track.**
- **Fourteen more zips**: seven with the sheet and its tracks in a folder inside the archive, seven with them at the
  archive's root, from two to sixty-one tracks.
- **No** `.m3u`, `.chd`, `.iso` or `.pbp`, no other subfolder, and no sheet naming a file that differs in case or in name:
  all 285 `FILE` lines of the twenty sheets name their file exactly. Two files are not games (a Finder `.DS_Store` and
  ES-DE's `systeminfo.txt`) and are not listed, having no extension a console takes.

Every form is now in `PlayStationLibraryTests.A_zipped_disc_and_a_lone_track_open_from_the_library_and_play` on
synthetic discs: a zip of a sheet and a track, a lone track, a lone "(Track 1)" file and a zip of a sheet at its root
with twelve tracks. The forms the library does not hold were probed on synthetic discs through the scan and the
factory's load [M]: a sheet in
an upper-case `.CUE`, an ES-DE multi-disc folder (`Game.m3u/` holding the list and both sheets), a cooked `.bin` of
2,048-byte sectors, a zip of a lone track in a folder and a zip carrying macOS's `__MACOSX/` entries open, and a `._`
file beside a game is not listed; three forms did not open, which §12.4 and §12.5 take.

### 12.3 What the handheld recorded

Read only, from the tester's session after 00:50 [M, 2026-10-12]. The settings name the SD card's `ROMs` as the library,
the themed library, and a log folder of the bench's (`/home/red/Documents/Logs/`), which the handheld cannot make, so
the logs went to `~/.config/EmuSen/Logs`, Galaxia's fallback.

- **Four PlayStation games were opened, and every one ran**: a "(Track 1)" lone file, two lone tracks and a zip, each
  with its per-game log of frame lines at 59.7–60.5 frames a second and 4–8 ms of emulation in a 16.7 ms frame. The
  zip was unpacked into `home/Unpacked` as §2 describes. No launch failed: no error log was written that day, and
  neither `Failed to load` nor a firmware line appears.
- **The process then aborted**, 15 seconds into the fourth game, on a Rust panic in `emusen_core_debug_registers`,
  `RefCell already borrowed` at `emusen-native/src/core/exports.rs:115` (the outbox borrowed by `look` to settle the
  call), with a core dump and no crash file of the program's own. A `RefCell` is borrowed twice only when two calls on
  one machine overlap, which `EmuSen_CoreAPI.md` §6.16 forbids ("one thread at a time per machine"). The emulation
  thread calls `debug_registers` after each frame (`RefreshProviders`); the window's gamepad timer, on the interface
  thread, calls `CoreEngine.SetAxis` on every poll and `SetButton` on each change, and both go into the machine at once
  (`CoreMachine.SetAxis`, `set_axis` settling through the same outbox) [R]. Diamond is the first v1 engine with a pad's
  sticks (the DualShock is the default pad), so it is the first whose machine is entered from the interface thread
  sixty times a second; the overlap is rare per poll, which is why a game ran for minutes first [A]. It is outside the
  launch path and is not fixed here (§12.5).

So the handheld's record shows the launch path working on every game tried; "doesn't work" in that session is the
abort, which takes the program down with the game, and is left to the branch that owns the running game.

### 12.4 Defects found and fixed

Each was shown by a WiseMan test failing before the change and passing after [M].

1. **A sheet naming its tracks in another case, or with Windows separators, opened nowhere, and its tracks were
   shelved as games of their own.** *Cause:* `ImageSets.Companions` joined each `FILE` name to the sheet's folder as
   written, so on a filesystem that keeps case (the handheld's SD card is ext4) a sheet made where case is ignored named
   files that do not exist; the shelf's rule then hid nothing, and an audio track was listed on the Genesis shelf (its
   content is not a disc's sync, which the Genesis's recogniser claims, §12.5). *Proof:*
   `DiamondPlayersTests.A_sheet_naming_its_tracks_in_another_case_or_with_backslashes_opens_and_is_shelved_once`, a
   loose sheet naming `TRACKS\` and its tracks in lower case and a zipped one naming them in upper case: before, the
   shelf listed the tracks and the load threw `FileNotFoundException`. *Fix,* in the media module that owns a sheet's
   companions: a name is taken as written where that file exists, else part by part as the one entry of the folder that
   matches it ignoring case, `/` and `\` both separating parts; no match, or two, leaves the name as written, so the
   load's words name it. The zipped probe already read an entry ignoring case; it now also takes `\`.
2. **A zipped game was asked for the firmware of the archive's bytes.** *Cause:* `CoreEngine.GetFirmwareRequirements`
   read the library file whole, so `firmware_for` received the archive (and the whole archive was read from the card
   once more before the load unpacked it); Diamond reads no disc in a zip and asked for no BIOS, where the same disc
   loose asks for its market's. *Proof:* `DiamondPlayersTests.A_zipped_disc_is_asked_for_the_firmware_its_image_is`:
   before, none for the zip and the declared American file loose. *Fix:* the probe reads the image play opens, `DiscsOf(path)[0]`,
   which unpacks an archive once, as the load then finds it unpacked.
3. **A file that opens nowhere closed the program instead of saying why.** *Cause:* every start from the library, the
   list and big picture asks `EmulatorSession.MissingFirmwareFor` before the load; it creates the core, which for a file
   routing decides for no console throws the decision's words, and the exception left `StartGameAsync`, which an
   `async void` handler awaits, so the dispatcher raised it out of the window. *Proof:*
   `PlayStationLibraryTests.A_file_that_opens_nowhere_says_why_from_the_library`, a `.bin` of sync patterns at the
   library's root: before, `NotSupportedException` out of the pad's press. *Fix:* the firmware question answers nothing
   for a file that cannot be opened or read (`NotSupportedException`, `IOException`, `InvalidDataException`,
   `UnauthorizedAccessException`), and the load after it fails in `LoadGame`'s own handling, the status line reading
   "Failed to load … could be … move it into a folder named genesis or psx".

### 12.5 Not fixed, and why

- **The abort of §12.3** was the host's, and is fixed since; §12.7 has the record. *The cause, proven:* a v1 machine
  entered from a second thread while its emulation thread was inside it, which `EmuSen_CoreAPI.md` §6.16 forbids; a
  child test host driving input that way aborted on `RefCell already borrowed` in 10 runs of 10, and one driving the
  console's read-only lines in 10 of 10 [M]. *The fix:* `CoreEngine` holds the pad's buttons and axes and sends them
  on the emulation thread as each frame starts, and every call on a machine's handle holds that machine's gate, so no
  two threads are in it at once; 0 runs of 10 aborted after, either way [M]. *The crossings found:* the pad timer and
  the keys in both frontends, the console's read-only lines in both, the Hardware Dashboard and Hotaru's feed window,
  Apply Cheats while paused and Hotaru's bounded shutdown; the rest were already on the emulation thread or behind a
  stopped one (§12.7.2).
- **A sheet naming a file by another name** (`Game (USA).bin` beside a file `Game.bin`) opens nowhere, the load's words
  naming the missing file. Choosing another file for it would be a guess; none of the library's sheets does it.
- **A loose audio track goes to the Genesis even in a folder named `psx`**: the Genesis's recogniser claims anything
  that is not a disc's image (§1), and a claim decides before the folder. Only a track left without its sheet meets
  this; the library holds none. Whether a folder should outweigh a negative rule is the routing design's question.
- **The image is read twice before the first frame**, once for `firmware_for` and once for `create`, and Diamond copies
  it once more; on the handheld the three launches the journal times took two to three seconds from the per-game log's
  start to the first frame line, which is written a second after frames begin, so it is recorded and not changed.
  *Measured and in part changed 2026-10-12, §12.8:* the question no longer reads the image, big picture loads under
  its launch screen, and the whole read and the copy at create are proposed for the tester's decision.

### 12.6 Tests run (2026-10-12)

WiseMan, by area: `PlayStationLibrary`, `Diamond` (with `DiamondPlayers`), `Galaxia`, `RomLibrary`, `SystemPack`,
`CoreCatalog`, `CoreFactory`, `Firmware` and `EmulatorSession`, 396 cases, 388 passing and 8 skipped (the opt-in ones
without their folders); the opt-in run of §12.1 on the seven copies, with development cores off and on.

### 12.7 The abort of §12.3, proven and fixed (2026-10-12)

*Written 2026-10-12, on WiseMan `3fb0e1e8`.* The marks are §12's; [M] is measured on this date with the test named.

#### 12.7.1 The proof

`CoreEngineThreadTests.A_second_thread_entering_a_running_machine_does_not_end_the_process` runs a synthetic program on
the open kernel, `DiamondTests.AnalogPadReading` (the DualShock put in analog mode by commands 43h and 44h, then read
over SIO0 without end), through `EmulatorSession` as Mistress loads it. One thread runs 900 frames unpaced in the
emulation loop's order, the frame and then the debugger's `RefreshProviders`. A second thread, at once, does one of
two things:

- **input**: `SetAxis` on both left-stick axes and `SetButton` on Start, in a loop, the calls `PortRouter` makes from
  the window's pad timer (every axis on each poll, a button on each change);
- **debugger**: `mem ram 10000 16` and `disasm r3000` through `DianaShInterpreterScheduler.SubmitFromAnyThread`, the
  console's read-only fast path, and the target's `FrameCount` and `GetSummaryText`, which the Hardware Dashboard and
  `cpus` read.

A panic across the boundary aborts the whole process, so each case is a child test host that the test starts, judged
by its exit, its output and a mark file it writes only if it finishes. Ten runs of each case [M]:

| | Before (`3fb0e1e8`) | After |
|---|---|---|
| input | 10 of 10 aborted: `exports.rs:105` eight times (the settle of a call), `:115` (the settle of a read, the handheld's line) and `:135` once each | 0 of 10; in the two runs timed, 900 frames in 5.7 and 5.8 s, the second thread making 12.1 and 12.2 million rounds |
| debugger | 10 of 10 aborted: `:105` five times, `:115` four, `:410` (the events' outbox) once | 0 of 10; in the two runs timed, 900 frames, the second thread finishing 33 and 53 rounds |

The aborted children's runs took 2–3 s in all, against 7 s for a finished one, so each abort came early in the 900
frames [M]. The panic names whichever of the outbox's borrows met the other thread's, so the
handheld's line is one of several the race produces, not a separate defect [A].

#### 12.7.2 Every crossing, and what was done with it

Searched in Mistress, Hotaru, the shell's window, Pharaoh, `EmulatorSession` and the engines over `CoreEngine` and
`CoreMachine` (Diamond, VenusRT, Nephrite, and MoonRT and MercuryRT through `PortEngine`) [R]:

| Path | Thread | Machine calls | Done |
|---|---|---|---|
| Mistress's pad timer and keys (`PollGamepad`, `ApplyKeys`, `PortRouter`) | interface | `set_axis`, `set_buttons` | **proven; latched** (§12.7.3) |
| Hotaru's pad timer and keys | interface | the same | latched |
| The console's read-only lines (`mem`, `xxd`, `dump`, `disasm`, `cpus`, `regs`, `eval`, `find`, `memfind`, `tile`, `tilemap`, `pal`, `vectors`, …), Mistress's window and Hotaru's reader thread | interface, reader | `space_size`, `space_read`, `debug_disassemble`, `debug_pc`, `debug_registers`, `frame_count` | **proven; behind the gate** |
| The Hardware Dashboard, 4 Hz | interface | `frame_count`, and `space_size` and `space_read` for the palette and tile sheet | behind the gate |
| Hotaru's feed window (`feed -w`), 30 Hz | interface | `frame_info`, `frame_copy` | behind the gate; the engine's scratch picture is shared with the emulation thread's copy, so the feed can show a torn frame (a debug view; not changed) |
| Apply Cheats while paused (`RequestCheatApply`) | interface, the emulation thread parked | `set_rom_patches`, `set_cheat_pokes`, `space_read`, `space_write` | behind the gate; the engine's cheat fields are shared with a request the parked thread would run only if one were queued in the same instant (not observed; not changed) |
| Hotaru's shutdown: a 500 ms join, then `SaveSram` | interface | `battery`, `battery_saved` | behind the gate, for a frame that outlives the join |
| States, Change Disc and the lid, live settings, screenshots, the rewind reel, state pictures, running Apply Cheats, the console's queued lines | emulation, by `RequestOnEmulationThread`, the cheats flag and the console's queue | — | already safe: run between frames by the owner (`EmuSen_Settings_Reference.md` §4.21a, §4.26) |
| Load, reset, the library, closing | interface, after `StopEmulationThread` joins | `create`, `battery`, `state_save`, `free` | already safe: the thread is stopped first |
| `IsRomLoaded`, `ControllerPorts`, `SupportedAxes`, `CanChangeDisc`, `Discs`, `LidOpen`, `FrameRateHz` | interface | none: managed fields and machine info, replaced whole | safe |
| `SetControllerConnected` | interface | none for a v1 engine (`ICore`'s default) | safe |
| Memory card actions | — | none exist in either frontend; a card is a battery file, written on the emulation thread or after it stops | safe |
| The shell's own window, Pharaoh | — | no emulator, and one thread | safe |

Outside `CoreEngine` and `CoreMachine`, and not changed: **MarsRT's shim** (`MarsRtCore`) presses buttons, moves the
stick and plugs a pad from the interface thread straight into `mars_machine_press`, `_set_stick` and `_set_present`,
which take the machine mutably while `advance` may hold it. They are plain stores with no `RefCell`, so nothing
panics, but by Rust's rules it is a data race; no misbehaviour has been shown, so it is recorded rather than changed, and
the same latch would take it [R]. The C# cores keep the single-writer race that `EmuSen_Settings_Reference.md` §4.21
accepts.

#### 12.7.3 The fix

- **The input latch** (`InputLatch.cs`). `CoreEngine.SetButton` and `SetAxis` route by the descriptors as before and
  hold the result: for each port the buttons held and the bits changed since the last frame, for each port and axis the
  latest value. `RunFrame` sends it before `advance`, `set_buttons(port, held, changed)` for a port with changes and
  `set_axis` for each axis moved, copying under the latch's lock into arrays kept between frames, so a frame
  allocates nothing. `PortEngine`'s whole-pad routing (`changed` = FFh, `EmuSen_NativeCores.md` §3.8) goes into the same
  latch. The interface thread never enters the machine for input.
- **The machine's gate** (`CoreMachine.Gate`). Every call on a machine's handle holds a lock of that machine: each of
  `CoreMachine`'s methods for its whole body, so a status and its `last_error` are one call's, and the debug bridges
  and the ports' extensions around their own calls on `Handle`. §6.16 asks for one thread at a time, which mutual
  exclusion gives; it does not ask for one fixed thread [R].
- **Why both, and not one.** The gate alone would serialise input too, and the pad timer would wait for the advance in
  flight, about 6 ms on the test's program, on each poll of the interface thread. The latch alone would leave the
  debugger's reads. Making every read a request to the emulation thread would turn the console's read-only lines,
  which answer at once by design (`EmuSen_Debugging_Tools_Reference_v5.md` §3.3a), and the dashboard's reads into
  asynchronous ones; the gate keeps them synchronous, waiting at most for the call in flight [C].
- **Cost.** Uncontended, the gate costs 8.5 ns a call: 16.4 ns for `TotalFrames` against 7.9 ns for the bare export,
  five million calls, three rounds [M]. Mistress's loop makes about 25 machine calls a frame on Diamond, counted from
  the code [R], so about 0.2 µs in a 6.4 ms frame of the test's program [A]. A read on another thread waits for the
  call in flight and, because a monitor is not fair, for frames run back to back: unpaced, the debugger case's thread
  finished 33 and 53 rounds of four reads in 5.7 s, `mem`'s sixteen byte reads each waiting about a frame [M]. A paced
  game leaves the gate free between frames, so the wait is at most one advance; fast-forward is the unpaced case [A].
- **No surface changed.** The core ABI, `ICore` and `IDebugTarget` are as they were.

#### 12.7.4 When a change reaches the machine

`CoreEngineThreadTests.A_press_and_a_stick_set_before_a_frame_reach_that_frame`: Start, LeftX at 1 and RightY at −1, set
on another thread that is joined before `RunFrame`, are in that frame's last pad read (F7h, FFh, 00h), and their release
in the next frame's [M]. It passes before the change and after, so a change made before a frame starts still reaches
that frame, and input latency has not grown. What differs is a change made while a frame runs: it reached the machine
part way through that frame, where a game reading the pad later in the frame could see it, and now reaches it as the
next frame starts. So a press and its release both made after one frame starts and before the next does are now not
seen at all, as with any frame-start latch; before, the press was seen if the game read the pad after it within the
frame in flight [R]. The pad timer polls once a frame interval, so only a key tapped and released within one frame can
meet this [A].

#### 12.7.5 The crash file

- **What the handheld recorded.** No report of Mistress's own (`CrashLog`'s `crash_*.txt`), and §12.3 does not say
  whether the core's own was written. The core's panic hook writes one (`emusen_core_set_crash_log`, §6.18) [R].
- **What a run shows** [M]. In the child runs before the change, the hook had been given
  `DataStore.Logs/diamond_crash_<pid>.txt`, and no such file was written anywhere under the run's folder. The hook
  makes the file and not its folder; the sandbox's `Logs` folder is made by DianaSh's skeleton when the shell's
  tree is first touched, which that process never did. A Mistress that had touched it would have written the report
  to the sandbox's `/Logs`, which is not the folder the tester reads, `ErrorLog.Root()` (§12.3: `~/.config/EmuSen/Logs`
  on the handheld) [A].
- **The change**, inside the existing surface: the host passes a path in `ErrorLog.Root()`, the folder of `CrashLog`'s
  reports and the per-game logs, and makes the folder first (`CoreLibrary.CrashLogPath`, used by the v1 loader and by
  `NativeCoreLibrary`'s). `set_crash_log` already took any path.
- **Its proof**: `A_panic_in_the_core_leaves_its_crash_file_in_the_log_folder` provokes the panic on purpose, `set_axis`
  called on a second thread past the gate as a host ignoring §6.16 would, in a child whose log folder does not exist.
  Before the change no file was written; after it `diamond_crash_<pid>.txt` is in the log folder, holding
  `Diamond panic`, `RefCell already borrowed` and the backtrace [M].
- **Two panics at once.** Both threads of a race can see the borrow, and the hook ran once for each: in 30 runs of the
  provoked child, 4 printed two panics [M]. The hook wrote with `std::fs::write`, which truncates, so a second report
  started while the first thread aborted could leave the file empty, as one area run did (the test above failed on an
  empty `diamond_crash_<pid>.txt`) [M]. The hook in `emusen-native` (`abi::install_crash_log`) now appends, and holds a
  lock while it writes, so the first report is whole before its thread aborts. In 30 runs after it, every file held a
  whole report and 8 held two [M].
- **What `CrashLog` cannot do.** A core's panic aborts the process (`panic = "abort"`), and no managed code runs after
  `abort` raises SIGABRT, so a report of Mistress's own cannot be written for it [A]. *Proposed, not built:* Mistress
  could name, at its next start and in its error log, a core crash file left by a process that is no longer running.
  It needs no new surface (`ErrorLog`), but it changes start-up, so it is put to the tester.

#### 12.7.6 Tests run (2026-10-12)

`CoreEngineThreadTests` (the two child cases, the crash file and the latency), and WiseMan by area for the blast radius
after `git merge WiseMan` (`c27bbfbf`): `CoreEngine`, `CoreMachine`, `CoreAdapter`, `CoreAbi`, `CoreDebug`,
`CoreFactory`, `Diamond` (with `DiamondPlayers`), `PlayStationLibrary`, `Input`, `Controller`, `Pad`, `MoonRt`,
`MercuryRt`, `VenusRt`, `Nephrite`, `MoonDebugTargetContract`, `NativeHost` and `EmulatorSession`: 1,130 cases, 1,125
passed, 3 skipped (the picture tools, opt-in) and 2 failed. One was the crash file's test, on the empty file of two
panics, fixed as above and passing since. The other,
`ConsoleBindingsTests.The_catalog_lists_consoles_by_manufacturer_then_release`, lists the consoles without the
PlayStation, which stage 11 offers to players; it fails alone as well, the change does not touch the catalog, and it is
left to the branch that owns the catalog [R]. On the run before the merge,
`MercuryRtCoreTests.A_second_ports_buttons_are_ignored_on_both_engines` failed: it pressed and read the joypad register
with no frame between, which a latched press no longer reaches, and it now runs a frame after the presses.
`emusen-native`'s own tests: `cargo test --release`, 63 passed.

### 12.8 Launch time, measured and shortened (2026-10-12)

*Written 2026-10-12, after the tester reported "an awkward delay when launching psx games" in big picture mode, play
speed on the handheld being fine by then.* This section is the time from the pad's press on a game in big picture to
the game's first frame, step by step, before and after the changes it records. The marks are the page's.

#### 12.8.1 How it was measured

- **The engine's steps**, through `DiamondLaunchTimeTests.The_testers_discs_launch_step_by_step` (WiseMan, opt-in by
  `EMUSEN_DIAMOND_LAUNCHTIME_LIBRARY`, with `EMUSEN_DIAMOND_LAUNCHTIME_HOME` for the unpacked copies): the calls
  `EmulatorSession.MissingFirmwareFor` and `CoreEngine.LoadRom` make, one at a time through the same public types
  (routing, the probe's engine, the firmware question, the unpack, the image's and the companions' reads,
  `firmware_for`, `create`, a second `create` with a memory card among the files, the first frame), then the real
  `CoreFactory.Load` as a check on their sum. Each game three ways: **first**, an archive not yet unpacked and every
  file out of the page cache; **cold**, unpacked but out of the cache; **warm**, straight after.
- **Big picture's launch**, through `PlayStationLaunchTimeTests.The_testers_discs_launch_from_big_picture_timed`
  (opt-in alike): a headless themed window whose `psx` folder holds a hard link to each copy, the pad's A in the game
  list, the interface clock stepped every 16 ms in step with the wall clock, and timed: the press until the launch
  screen is up, the 16 ms step in which the game was loaded, and the press until the first frame. Each game cold and
  warm, with the screen at Normal (3.0 s, the handheld's setting) and Disabled. Each window has a home of its own, so a
  zip is unpacked in every one of its launches.
- **Cold without root:** `posix_fadvise(POSIX_FADV_DONTNEED)` on each file, which drops a file's clean pages from the
  page cache; `fincore` showed 0 of 154,318 pages resident after it on Crash Bandicoot's track, and `cat` then read it
  in 0.20 s against 0.03 s warm [M].
- **The card, simulated:** the same runs under `systemd-run --user -p "IOReadBandwidthMax=/dev/nvme1n1 90M"`, systemd's
  per-account manager delegating the `io` controller; `cat` of the 632 MB track then took 6.9 s, the speed `Diamond_CDROM.md` §9.7
  assumed for the handheld's microSD card [M]. The runs keep WiseMan's 8 GiB memory limit, under which the page cache is
  reclaimed between launches: a few warm rows under the cap are slower than their cold row for that reason, and are
  noise, not a property of the code [A].
- **The discs:** the seven copies of §12.1 in the library's forms (two sheets beside their tracks, two lone tracks, three
  zips, every zip holding a sheet), with no BIOS in the firmware folder, so the open kernel ran. Nothing in the library
  or on the handheld was written.

#### 12.8.2 Before: where the time went

[M, WiseMan `414cac36`, desktop.] Each row's numbers are milliseconds; the totals are the probe's own, which add the
check's second load to its steps.

- **The firmware question read the whole image, on the interface thread, before the launch screen opened.**
  `MissingFirmwareFor` created the probe's engine and asked `GetFirmwareRequirements`, which unpacked an archive
  (`DiscsOf`) and read the image whole to hand it to `firmware_for`, only to learn that Diamond offers its market's BIOS
  as an optional file. A sheet is its own small file, so a sheet's question cost nothing; a lone track's cost its whole
  read, 60–270 ms on the desktop and 4.8–6.9 s under the card's cap; a zip's first launch cost its unpack, 1.0–1.4 s
  and 3.7–5.6 s. In big picture that was the carousel frozen between the press and the screen.
- **The launch screen then held the game for its full time, and only then was the game loaded**, on the interface
  thread, with the screen frozen: 0.30–0.71 s on the desktop, 2.0–6.6 s for a sheet's tracks read cold under the cap.
- **The load reads the image and its tracks once more** (from the page cache when the question had just read them),
  then `create`, whose 150–290 ms is the core's copy of the image into fresh pages (the disc's parse reads nothing);
  `firmware_for` a second time for the notice; and, wherever a memory card is on disc, **a second `create`** of the same
  cost (`EmuSen_CoreAPI.md` §19.1), since Diamond's cards are file 0 as `.mcr` and file 4, which the first create
  cannot know. The first frame took 3 ms.

The engine's steps, desktop, and under the card's cap (before → after for the question and the total; the other
columns after):

| Game | Condition | Firmware question, before → after | Unpack | Image and tracks read | One create | Second create | Total, before → after |
|---|---|---|---|---|---|---|---|
| Ape Escape (sheet) | first | 0 → 1 | 0 | 160 | 156 | — | 708 → 691 |
| Ape Escape (sheet) | cold | 0 → 0 | 0 | 89 | 150 | 148 | 636 → 638 |
| Ape Escape (sheet) | warm | 0 → 0 | 0 | 42 | 149 | 151 | 589 → 593 |
| Castlevania (zip) | first | 1296 → 0 | 1307 | 156 | 197 | — | 2078 → 2084 |
| Castlevania (zip) | cold | 0 → 0 | 0 | 166 | 202 | 208 | 910 → 928 |
| Castlevania (zip) | warm | 0 → 0 | 0 | 56 | 200 | 202 | 828 → 803 |
| Crash Bandicoot (track) | first | 251 → 0 | 0 | 226 | 216 | — | 1100 → 894 |
| Crash Bandicoot (track) | cold | 199 → 0 | 0 | 199 | 215 | 220 | 1232 → 1016 |
| Crash Bandicoot (track) | warm | 59 → 0 | 0 | 68 | 216 | 217 | 926 → 856 |
| Rayman (sheet, 51 tracks) | first | 1 → 0 | 0 | 262 | 191 | — | 779 → 750 |
| Rayman (sheet, 51 tracks) | cold | 1 → 0 | 0 | 278 | 191 | 192 | 999 → 997 |
| Rayman (sheet, 51 tracks) | warm | 0 → 0 | 0 | 57 | 193 | 204 | 357 → 785 |
| Silent Hill (zip) | first | 964 → 0 | 965 | 179 | 212 | — | 1588 → 1689 |
| Silent Hill (zip) | cold | 0 → 0 | 0 | 164 | 209 | 213 | 944 → 934 |
| Silent Hill (zip) | warm | 0 → 0 | 0 | 65 | 213 | 213 | 844 → 842 |
| Spyro the Dragon (track) | first | 268 → 0 | 0 | 269 | 225 | — | 1223 → 995 |
| Spyro the Dragon (track) | cold | 160 → 0 | 0 | 159 | 237 | 243 | 1240 → 1070 |
| Spyro the Dragon (track) | warm | 64 → 0 | 0 | 65 | 241 | 240 | 978 → 942 |
| Tekken 3 (zip) | first | 1424 → 0 | 1426 | 82 | 252 | — | 2154 → 2151 |
| Tekken 3 (zip) | cold | 0 → 0 | 0 | 204 | 256 | 251 | 1093 → 1157 |
| Tekken 3 (zip) | warm | 0 → 0 | 0 | 72 | 247 | 244 | 931 → 987 |

Under the 90 MB/s cap:

| Game | Condition | Firmware question, before → after | Unpack | Image and tracks read | One create | Second create | Total, before → after |
|---|---|---|---|---|---|---|---|
| Ape Escape (sheet) | first | 0 → 1 | 0 | 1745 | 182 | — | 2413 → 2331 |
| Ape Escape (sheet) | cold | 0 → 0 | 0 | 1823 | 175 | 171 | 2429 → 2446 |
| Ape Escape (sheet) | warm | 0 → 0 | 0 | 43 | 153 | 155 | 603 → 615 |
| Castlevania (zip) | first | 4628 → 0 | 4622 | 193 | 215 | — | 5446 → 5470 |
| Castlevania (zip) | cold | 0 → 0 | 0 | 5334 | 227 | 230 | 6278 → 6179 |
| Castlevania (zip) | warm | 0 → 0 | 0 | 58 | 207 | 207 | 834 → 837 |
| Crash Bandicoot (track) | first | 6923 → 0 | 0 | 6927 | 246 | — | 7821 → 7670 |
| Crash Bandicoot (track) | cold | 6948 → 0 | 0 | 6948 | 245 | 249 | 8012 → 7857 |
| Crash Bandicoot (track) | warm | 62 → 0 | 0 | 63 | 223 | 229 | 924 → 896 |
| Rayman (sheet, 51 tracks) | first | 1 → 0 | 0 | 6311 | 218 | — | 6851 → 6871 |
| Rayman (sheet, 51 tracks) | cold | 1 → 0 | 0 | 6313 | 217 | 217 | 7109 → 7112 |
| Rayman (sheet, 51 tracks) | warm | 0 → 0 | 0 | 70 | 193 | 195 | 571 → 727 |
| Silent Hill (zip) | first | 3736 → 0 | 3739 | 203 | 218 | — | 4394 → 4500 |
| Silent Hill (zip) | cold | 0 → 0 | 0 | 5247 | 239 | 226 | 6092 → 6071 |
| Silent Hill (zip) | warm | 0 → 0 | 0 | 60 | 213 | 215 | 852 → 845 |
| Spyro the Dragon (track) | first | 4820 → 0 | 0 | 4821 | 252 | — | 5794 → 5600 |
| Spyro the Dragon (track) | cold | 4827 → 0 | 0 | 4830 | 265 | 256 | 5941 → 5787 |
| Spyro the Dragon (track) | warm | 65 → 0 | 0 | 65 | 235 | 237 | 976 → 936 |
| Tekken 3 (zip) | first | 5565 → 0 | 5568 | 85 | 287 | — | 6275 → 6330 |
| Tekken 3 (zip) | cold | 0 → 0 | 0 | 6834 | 278 | 275 | 7790 → 7810 |
| Tekken 3 (zip) | warm | 0 → 0 | 0 | 67 | 248 | 248 | 966 → 977 |

Big picture, from the pad's press (milliseconds, before → after). "Held at its end" is the interface thread's one step
at the screen's end, in which the game was loaded before and is only wired after:

| Game | Cache | Normal: A to the screen, before → after | Normal: held at its end | Normal: game started | Disabled: game started |
|---|---|---|---|---|---|
| Ape Escape (sheet) | cold | 44 → 36 | 414 → 50 | 3486 → 3118 | 340 → 379 |
| Ape Escape (sheet) | warm | 1 → 1 | 326 → 22 | 3352 → 3047 | 319 → 357 |
| Castlevania (zip) | cold | 1309 → 0 | 461 → 17 | 4792 → 3036 | 1668 → 1805 |
| Castlevania (zip) | warm | 1281 → 1 | 302 → 113 | 4607 → 3132 | 1673 → 1812 |
| Crash Bandicoot (track) | cold | 256 → 0 | 523 → 23 | 3805 → 3064 | 536 → 465 |
| Crash Bandicoot (track) | warm | 199 → 0 | 360 → 21 | 3600 → 3045 | 357 → 377 |
| Rayman (sheet, 51 tracks) | cold | 1 → 0 | 587 → 29 | 3612 → 3049 | 502 → 522 |
| Rayman (sheet, 51 tracks) | warm | 0 → 0 | 298 → 21 | 3325 → 3045 | 151 → 446 |
| Silent Hill (zip) | cold | 991 → 0 | 335 → 16 | 4350 → 3037 | 1536 → 1532 |
| Silent Hill (zip) | warm | 959 → 0 | 710 → 22 | 4699 → 3048 | 1316 → 2132 |
| Spyro the Dragon (track) | cold | 278 → 0 | 547 → 25 | 3856 → 3045 | 592 → 690 |
| Spyro the Dragon (track) | warm | 224 → 0 | 401 → 24 | 3650 → 3053 | 414 → 318 |
| Tekken 3 (zip) | cold | 1582 → 0 | 340 → 22 | 4945 → 3046 | 1739 → 1731 |
| Tekken 3 (zip) | warm | 1400 → 1 | 398 → 17 | 4823 → 3037 | 1712 → 1726 |

Under the 90 MB/s cap:

| Game | Cache | Normal: A to the screen, before → after | Normal: held at its end | Normal: game started | Disabled: game started |
|---|---|---|---|---|---|
| Ape Escape (sheet) | cold | 32 → 37 | 2046 → 50 | 5106 → 3119 | 2020 → 2008 |
| Ape Escape (sheet) | warm | 1 → 1 | 327 → 20 | 3351 → 3040 | 355 → 315 |
| Castlevania (zip) | cold | 4616 → 1 | 469 → 22 | 8112 → 5082 | 5375 → 10566 |
| Castlevania (zip) | warm | 4198 → 1 | 5888 → 22 | 13109 → 7109 | 10704 → 2889 |
| Crash Bandicoot (track) | cold | 10746 → 0 | 487 → 84 | 14261 → 9361 | 7413 → 7209 |
| Crash Bandicoot (track) | warm | 73 → 1 | 350 → 22 | 3450 → 3048 | 370 → 392 |
| Rayman (sheet, 51 tracks) | cold | 1 → 0 | 6563 → 26 | 9588 → 6578 | 6563 → 6563 |
| Rayman (sheet, 51 tracks) | warm | 0 → 0 | 151 → 20 | 3179 → 3047 | 207 → 205 |
| Silent Hill (zip) | cold | 3723 → 0 | 322 → 25 | 7067 → 4075 | 4429 → 4215 |
| Silent Hill (zip) | warm | 953 → 0 | 491 → 22 | 4472 → 3045 | 1414 → 1402 |
| Spyro the Dragon (track) | cold | 4818 → 0 | 4098 → 53 | 11944 → 5066 | 9311 → 5135 |
| Spyro the Dragon (track) | warm | 5895 → 0 | 1331 → 22 | 10257 → 3048 | 485 → 362 |
| Tekken 3 (zip) | cold | 5560 → 0 | 499 → 20 | 9084 → 5917 | 6465 → 6510 |
| Tekken 3 (zip) | warm | 1481 → 0 | 325 → 22 | 4824 → 3042 | 1717 → 1709 |

#### 12.8.3 What was changed

Each change keeps the launch exact: the same firmware passed, the same battery files, the same machine.

1. **The firmware question reads no image where the core's info leaves no firmware required** (`cfd44cf8`).
   `CoreEngine.RequiredFirmwareFor` answers empty, without the image, when every system of the core's info lists its
   firmware and no entry is required; any other info, a system listing none included, reads the image as before.
   *Why it is exact:* core info's `firmware` is "every firmware file the system can use … one entry for each image a
   game can name … as `firmware_for` gives them" (`EmuSen_CoreAPI.md` §6.3), and the host prompts only for a required
   entry (§6.2 there), so a library whose lists hold only optional files can name no required one; the condition is the
   conservative reading, since a foreign core may leave the optional `firmware` field out. Diamond lists its three
   BIOS images as optional, as the project's firmware policy has every core (`EmuSen_Firmware.md` §0). The load still
   asks `firmware_for` of the image and passes what it names, so the files given at create are unchanged. *Proof:*
   `DiamondPlayersTests.The_firmware_question_reads_no_image_where_no_entry_can_be_required` failed before the change
   ("the zip was unpacked for the question") and passes after it; with the track held under an exclusive lock the full
   read throws and the question does not; `Only_info_listing_every_system_s_firmware_as_optional_answers_without_the_image`
   pins the rule, a required entry or an empty list sending the question to the image. **The load's second
   `firmware_for`,** for the notice, now reuses the first answer, the same call on the same bytes.
2. **Big picture loads a v1 engine's game while the launch screen shows** (`79af6005`). The load begins on a worker as
   the screen opens (`CoreFactory.Loaded`, the part of `Load` before its wiring), and the game starts at the screen's
   end or when the load is done, whichever is later; the wiring, the cheats, the `EmulatorSession` and the emulation thread stay
   on the interface thread, after the screen, in the order they had. Two conditions keep it exact. *Only a v1
   engine's load goes there* (`CoreFactory.RunsOnV1Engine`): a v1 engine's load writes nothing to the console, while a
   C# core's load prints lines that belong in the game's own log, which `LoadGame` opens after the screen. *The game
   already loaded is loaded after the screen, as before:* the running game's battery files are written by its
   shutdown, and loading the same game early would read its memory card before that write. A load that fails under the
   screen is rethrown inside `LoadGame` and reported as any failed load. *Proof:*
   `LaunchScreenTests.A_disc_is_loaded_while_the_screen_shows_and_starts_at_its_end`, a zipped synthetic disc whose
   unpacking must finish while the screen is up and the game not started, failed before the change ("the disc was not
   loaded while the screen showed") and passes after it, the game still starting within one 16 ms poll of the screen's
   end; `A_disc_the_core_refuses_under_the_screen_is_reported_at_its_end` has Diamond refuse a malformed `.sbi` under
   the screen, and the status line reads "Failed to load … Diamond refused the game" with no game and the screen gone.
   The existing launch-screen cases, whose SNES games now load on the worker through VenusRT, passed three runs of
   three.
3. **A defect of change 2, found by its own tests' run, proven and fixed** (`edff1ae8`). A wider run of the areas
   stalled once, 254 cases in, with no failure printed: a dump of the test host showed no interface thread left and
   xunit waiting on a case dispatched to it. *Cause:* change 2 lets a launch wait for its load past the screen's time,
   and a window closed in that wait was still given the game when the load ended; `LoadGame` then ran on the closed
   window and threw out of the pad's handler on the dispatcher, which in a headless run ends the interface thread, and
   in the program would have been an exception at the moment of quitting. *Proof:*
   `LaunchScreenTests.A_window_closed_while_the_load_outlasts_the_screen_starts_nothing` holds the load behind a gate
   past the screen's time, closes the window and lets the load go: before the fix, `InvalidOperationException` from
   `ShowLibraryEntries` by way of `StartGameAsync`. *Fix:* when the load ends, a launch whose screen has been taken
   down starts nothing, and the core loaded for it is disposed; after it the case passes, and its companion,
   `A_load_longer_than_the_screen_keeps_the_card_and_starts_the_game_when_it_is_done`, shows the card kept and the game
   started when a held load is let go. The same wider run then failed twice in three on two existing cases, the game
   not yet running at the screen's end: those cases step the interface clock through the screen's seconds in a tenth
   of a second of the wall's, which a worker's load cannot keep to. *Decided:* `ThemedSession`, whose clock is stepped,
   loads after the screen as it always did (the window's `EarlyLoader` set to none), and the cases about the load
   under the screen install the loader and wait on its end; the wider run then passed three times in three.

#### 12.8.4 After

From the tables above [M, desktop]:

- **The screen is up at the press:** 0–37 ms from A to the screen, where a lone track took 0.20–0.28 s and a zip's
  first launch 0.96–1.58 s.
- **The interface thread is never held for the disc:** its step at the screen's end is 16–113 ms on the desktop and
  20–84 ms under the cap, where it was 0.30–0.71 s and up to 6.6 s; the screen moves throughout.
- **The game starts at the screen's time whenever the load fits inside it:** 3.04–3.13 s on the desktop for all seven,
  where it was 3.33–4.95 s; under the cap warm, 3.04–3.05 s for six, Castlevania's 7.1 s being its unpack, which
  every window of the probe does afresh.
- **A cold launch from the card is now the read itself:** 3.1–9.4 s under the cap, where it was 5.1–14.3 s; Crash
  Bandicoot, 632 MB as one track, is the longest, 6.9 s of reading, 0.25 s of create and the screen's time under them.
- **Disabled, and the desktop's list,** still load on the interface thread, as before; a lone track's launch is one
  read shorter there (the second, from the page cache, 60–70 ms on the desktop), the zips' and the sheets' unchanged.

#### 12.8.5 Not changed, and why

- **The cold read of the whole image.** With the core taking its image whole at create, nothing on the host's side
  reads less. Diamond's boot reads 2.3–3.6 MB of the disc in its first 30 s, and the first frame none, so a read of the
  sectors as the game asks for them would end the wait; it changes a stable surface, and is proposed with these figures
  in `EmuSen_CoreAPI_Extensions.md` §10.2 (Q15) for the tester to decide.
- **The second create** when a memory card exists, 150–290 ms on the desktop: v1.0 gives the host no way to know the
  cards before the first create. The remedy recorded in `EmuSen_CoreAPI.md` §19.1, battery files in core info, is a
  surface change, proposed in `EmuSen_CoreAPI_Extensions.md` §10.5 (Q16).
- **The core's copy at create** is the ABI's rule ("the core copies what it keeps; no pointer in `params` outlives the
  call", `EmuSen_CoreAPI.md` §6.2); §10.2's shape B would remove it with the read.
- **A zip's first launch** inflates and writes the disc once, 1.0–1.4 s on the desktop and 3.7–5.6 s with the card's
  read; it now runs under the screen. Later launches find it unpacked (`EmuSen_Galaxia.md` §3.7).
- **Disabled and the desktop's list** keep loading on the interface thread: with no screen there is nothing to cover
  the load, and a window that took input during it could start a second game; a loading indicator there would be a
  design of its own.

#### 12.8.6 The handheld

[M, 2026-10-12, 16:30–16:40, the handheld on its charger, Mistress not running, nothing else under load.] A
self-contained console over `EmuSen.dll` and `libdiamond.so`, built from this branch before WiseMan `992546e4` was
merged in and kept in the probe's cache (`launchtime/tool`), ran §12.8.1's engine steps there, the question as it was
(`before`) or as it is (`after`), from `~/launchtime` on the internal drive, with its home, configuration and firmware
folder under that folder, so that nothing of the player's was read but the discs on the card. `fincore` chose the
files, those with none of their pages in the page cache, and showed the cache before each run. The folder was removed
afterwards, and the card's PlayStation folder was the same in names, sizes and times as that morning's listing.

| Game (form, size) | Sequence, cache | Firmware question | Unpack | Image and tracks read | Create | Create with a card | Total |
|---|---|---|---|---|---|---|---|
| Mega Man X5 (track, 556 MiB) | before, cold | 6,305 | 0 | 402 | 84 | 72 | 6,966 |
| | before, 294 MiB cached | 2,971 | 0 | 62 | 182 | 175 | 3,516 |
| | before, 472 MiB cached | 973 | 0 | 130 | 129 | 91 | 1,434 |
| Mega Man X6 (track, 572 MiB) | after, cold | 0 | 0 | 6,247 | 189 | 172 | 6,734 |
| | after, 519 MiB cached | 0 | 0 | 635 | 171 | 164 | 1,102 |
| | after, all cached | 0 | 0 | 60 | 64 | 53 | 278 |
| Star Wars Episode I: Jedi Power Battles (sheet, track of 424 MiB) | after, cold | 1 | 0 | 4,648 | 53 | 45 | 4,839 |
| | after, all cached | 0 | 0 | 51 | 45 | 45 | 228 |
| Tekken 3 (zip, 479 MiB) | before, first launch, cold | 5,860 | (in the question) | 131 | 156 | 128 | 6,412 |
| | after, unpacked | 0 | 5 | 120 | 193 | 214 | 677 |
| Silent Hill (zip, 320 MiB) | after, first launch, cold | 0 | 3,975 | 86 | 136 | 131 | 4,456 |
| | after, unpacked | 0 | 4 | 103 | 164 | 151 | 561 |

Milliseconds; the totals include 80–100 ms of routing and of the library's first opening, which a fresh process pays
and Mistress pays once at its start; the first frame took 3 ms in every row.

- **The card reads at 88–92 MiB a second**, as `Diamond_CDROM.md` §9.7 assumed and the desktop's cap stood for: 6.3 s
  for a 556 MiB track, 4.6 s for a 424 MiB one. That prediction is confirmed and retired.
- **Before the change, that read was the frozen carousel**: 6.3 s for a lone track and 5.9 s for a zip's first
  launch, before the launch screen opened. After it the question takes no time, and the same read runs under the
  screen.
- **A second launch is not a warm one there.** After Mega Man X5's first run 294 of its 556 MiB were still in the page
  cache, and after the second 472: the launch's own memory, the host's buffer and the core's copy, 1.1 GB beside a
  cache that fills the handheld's 23 GiB, pushes out the pages the launch has just read once. The desktop did not show
  this. It adds to the case for reading the disc in place (`EmuSen_CoreAPI_Extensions.md` §10.2, shape B), which
  allocates neither.
- **A create is 45–214 ms**, no slower than the desktop's, and five games there already have a memory card on disc
  (their files were listed, not read), so their launches create twice (§12.8.5).
- **The launch from big picture, put together from these steps** [A; no window ran on the handheld]: a lone track cold
  took about 10 s before the change (6.3 s of frozen carousel, the screen's 3.0 s, 0.6 s of load) and takes about 6.7 s
  after it, the read and the creates, the card moving throughout; a zip's first launch about 9.4 s before and
  4.5–6.3 s after; any game whose files the cache holds, 3.3–3.5 s before and the screen's 3.0 s after.


#### 12.8.7 Tests run (2026-10-12)

WiseMan, by area, on this branch: the firmware question's change with `Firmware`, `EmulatorSession`, `CoreAdapter`,
`DiamondPlayers`, `VenusRtCoreAbi`, `CoreRegistrationEquivalence` and `PlayStationLibrary`, 115 passing; the load under
the screen with `LaunchScreen`, `ThemedLibrary`, `Screensaver`, `PlayStationLibrary`, `EmulatorSession`,
`CoreFactory`, `DiamondPlayers`, `Resume`, `Firmware` and `CoreAdapter`, 269 passing and 4 skipped (opt-in cases without
their folders); the new cases each failing before their change where §12.8.3 says so. After WiseMan `992546e4` was
merged in (the processor run in stretches and the rasteriser by rows, none of the launch path), the same areas with
all of `Diamond` and `CoreEngine`, 317 cases: once before change 3's fixture rule 311 passing and 2 failing as that
item records, then three runs of 313 passing and 4 skipped; `LaunchScreen` alone, 24 passing and 1 skipped, three
times. The two probes ran opt-in on the seven copies, plainly and under the cap, before the changes and after them,
and big picture's again on the merged tree: 3.03–3.22 s from the press to the first frame at Normal, cold and warm.

#### 12.8.8 Read for this section

Mistress's launch path (`MainWindow.Resume.cs`, `MainWindow.LaunchScreen.cs`, `MainWindow.BigPicture.cs`,
`MainWindow.axaml.cs`), the engine and its factory (`CoreEngine.cs`, `CoreFactory.cs`, `CoreLibrary.cs`,
`CoreMachine.cs`, `CoreDiscovery.cs`, `EmulatorSession.cs`), the runtime's media module (`ArchiveImages.cs`,
`ImageSets.cs`), Diamond's `v1.rs`, `media.rs` and `medium.rs`, `emusen-disc`'s `disc.rs` and `sbi.rs`,
`EmuSen_CoreAPI.md` §4.4, §6.2, §6.3 and §19.1, `EmuSen_CoreAPI_Extensions.md` §10, `EmuSen_Firmware.md` §0–§3 and
`Diamond_CDROM.md` §9.7. No emulator's source and no BIOS image was read; the firmware folder of every run was empty.

#### 12.8.9 Reading the disc in place: minor 1.3 measured (2026-10-12)

*Decided 2026-10-12 and built the same day: the core ABI's minor 1.3, `BORROW` and battery files in core info
(`EmuSen_CoreAPI.md` §6.24, §6.3; `EmuSen_CoreAPI_Extensions.md` §10.2.5, §10.5).* Diamond claims `BORROW` and reads
its disc where Galaxia has mapped it (`EmuSen_Galaxia.md` §3.8), and lists both cards' files in core info, so a game
with its cards on disc is created once. §12.8.5's first three items are thereby done: the cold read of the whole
image, the second create and the core's copy at create are gone. The marks are §12's; [M] is measured on this date.

**How it was measured.** A console tool over this branch's `EmuSen.dll` (in the probe's cache, `discstream/tool`)
makes one launch per process in two ways, so that its peak resident memory (`VmHWM`) is the launch's own: **1.2**,
the launch as it was, reproduced with today's types (the image and its tracks read whole, created from copied bytes,
then created again with a card file); and **1.3**, the engine's steps (Galaxia's `DiscSet` and `DiscFiles`, the
firmware question asked of the mapping, one create lent the files with a card file among them). Cold is the files'
pages dropped from the page cache by `posix_fadvise`, as §12.8.1's; the cap is §12.8.1's. The big picture rows are
`PlayStationLaunchTimeTests` again, unchanged, on this branch. On the handheld the tool ran from `~/launchtime` on its
internal drive with Mistress not running, the card only read, the load average written beside each launch (the
interface used about a core while idle; the averages rose to 1.2–1.4 as the batch went on).

Each 1.2 row's steps are the read of the image and its tracks, the create, and the create with the card; each 1.3
row's the mapping and its one create. Totals include 100–150 ms of routing, the probe's engine and the library's first
opening that a fresh process pays and Mistress pays once.

Desktop:

| Game | Condition | 1.2: read, create, create with the card | 1.2: total | 1.3: map, create with the card | 1.3: total | Peak resident, 1.2 → 1.3 |
|---|---|---|---|---|---|---|
| Ape Escape (sheet) | cold | 182, 169, 160 | 633 | 3, 10 | 109 | 878 → 53 MiB |
| Ape Escape (sheet) | warm | 140, 162, 157 | 581 | 3, 8 | 106 | 878 → 53 MiB |
| Castlevania (zip) | first | unpack 1413, 196, 213, 208 | 2,167 | unpack 1438, 3, 8 | 1,558 | 1,175 → 60 MiB |
| Castlevania (zip) | cold | 235, 221, 216 | 822 | 3, 9 | 124 | 1,173 → 58 MiB |
| Castlevania (zip) | warm | 188, 221, 214 | 763 | 3, 8 | 123 | 1,173 → 58 MiB |
| Crash Bandicoot (track) | cold | 244, 239, 229 | 858 | 2, 8 | 114 | 1,261 → 54 MiB |
| Crash Bandicoot (track) | warm | 198, 237, 231 | 805 | 2, 8 | 118 | 1,261 → 54 MiB |
| Rayman (sheet, 51 tracks) | cold | 337, 202, 201 | 869 | 4, 10 | 111 | 1,233 → 53 MiB |
| Rayman (sheet, 51 tracks) | warm | 206, 204, 200 | 736 | 3, 8 | 108 | 1,233 → 53 MiB |
| Silent Hill (zip) | first | unpack 1015, 206, 224, 225 | 1,804 | unpack 1019, 3, 8 | 1,136 | 1,238 → 60 MiB |
| Silent Hill (zip) | cold | 241, 229, 223 | 838 | 3, 10 | 122 | 1,236 → 58 MiB |
| Silent Hill (zip) | warm | 191, 226, 221 | 782 | 3, 8 | 121 | 1,236 → 58 MiB |
| Spyro the Dragon (track) | cold | 278, 247, 238 | 905 | 2, 8 | 115 | 1,317 → 54 MiB |
| Spyro the Dragon (track) | warm | 209, 245, 236 | 825 | 2, 8 | 115 | 1,317 → 53 MiB |
| Tekken 3 (zip) | first | unpack 1507, 231, 254, 248 | 2,383 | unpack 1482, 3, 8 | 1,598 | 1,375 → 60 MiB |
| Tekken 3 (zip) | cold | 292, 255, 256 | 950 | 3, 10 | 122 | 1,373 → 58 MiB |
| Tekken 3 (zip) | warm | 226, 252, 250 | 872 | 3, 8 | 120 | 1,373 → 58 MiB |

Desktop under the 90 MB/s cap:

| Game | Condition | 1.2: read, create, create with the card | 1.2: total | 1.3: map, create with the card | 1.3: total | Peak resident, 1.2 → 1.3 |
|---|---|---|---|---|---|---|
| Ape Escape (sheet) | cold | 1819, 170, 162 | 2,274 | 3, 10 | 110 | 879 → 53 MiB |
| Ape Escape (sheet) | warm | 141, 162, 157 | 578 | 3, 8 | 109 | 878 → 53 MiB |
| Castlevania (zip) | first | unpack 4488, 191, 222, 216 | 5,255 | unpack 4484, 3, 8 | 4,603 | 1,175 → 60 MiB |
| Castlevania (zip) | cold | 5275, 216, 211 | 5,846 | 3, 9 | 121 | 1,174 → 58 MiB |
| Castlevania (zip) | warm | 189, 215, 213 | 757 | 3, 8 | 119 | 1,173 → 58 MiB |
| Crash Bandicoot (track) | cold | 6799, 239, 233 | 7,416 | 2, 8 | 118 | 1,261 → 53 MiB |
| Crash Bandicoot (track) | warm | 199, 233, 227 | 798 | 2, 8 | 111 | 1,261 → 54 MiB |
| Rayman (sheet, 51 tracks) | cold | 6329, 203, 204 | 6,862 | 4, 10 | 112 | 1,233 → 53 MiB |
| Rayman (sheet, 51 tracks) | warm | 209, 203, 200 | 738 | 3, 8 | 107 | 1,233 → 53 MiB |
| Silent Hill (zip) | first | unpack 3614, 205, 232, 224 | 4,416 | unpack 3612, 3, 8 | 3,731 | 1,237 → 60 MiB |
| Silent Hill (zip) | cold | 5093, 233, 221 | 5,688 | 3, 10 | 126 | 1,236 → 58 MiB |
| Silent Hill (zip) | warm | 201, 230, 223 | 798 | 3, 8 | 120 | 1,236 → 58 MiB |
| Spyro the Dragon (track) | cold | 4694, 242, 239 | 5,315 | 2, 8 | 115 | 1,317 → 54 MiB |
| Spyro the Dragon (track) | warm | 210, 252, 246 | 844 | 2, 8 | 114 | 1,317 → 53 MiB |
| Tekken 3 (zip) | first | unpack 5478, 219, 250, 245 | 6,331 | unpack 5444, 3, 8 | 5,561 | 1,374 → 60 MiB |
| Tekken 3 (zip) | cold | 6693, 257, 249 | 7,342 | 3, 9 | 121 | 1,373 → 58 MiB |
| Tekken 3 (zip) | warm | 216, 251, 248 | 856 | 3, 8 | 121 | 1,373 → 58 MiB |

The handheld, from its card (the load average beside each pair):

| Game | Condition | Load | 1.2: read, create, create with the card | 1.2: total | 1.3: create with the card | 1.3: total | Peak resident, 1.2 → 1.3 |
|---|---|---|---|---|---|---|---|
| Crash Bandicoot (track) | cold | 0.43–0.43 | 6,612, 63, 53 | 6,822 | 8 | 103 | 1,263 → 58 MiB |
| Crash Bandicoot (track) | warm | 0.43–0.48 | 63, 59, 53 | 261 | 7 | 96 | 1,263 → 57 MiB |
| Spyro the Dragon (track) | cold | 0.48–0.48 | 6,923, 64, 56 | 7,136 | 8 | 99 | 1,319 → 57 MiB |
| Spyro the Dragon (track) | warm | 0.48–0.56 | 65, 61, 56 | 272 | 7 | 95 | 1,321 → 59 MiB |
| Ape Escape (sheet) | cold | 0.56–0.56 | 4,517, 49, 37 | 4,688 | 10 | 101 | 882 → 59 MiB |
| Ape Escape (sheet) | warm | 0.56–0.60 | 44, 42, 39 | 216 | 8 | 97 | 887 → 60 MiB |
| Jedi Power Battles (sheet) | cold | 0.60–0.60 | 4,658, 52, 38 | 4,835 | 10 | 97 | 910 → 63 MiB |
| Jedi Power Battles (sheet) | warm | 0.60–0.63 | 46, 44, 40 | 213 | 9 | 98 | 909 → 64 MiB |
| Tekken 3 (zip) | first | 0.63–1.03 | unpack 13,934, 78, 64, 61 | 14,233 | unpack 14,042, 8 | 14,167 | 1,382 → 71 MiB |
| Tekken 3 (zip) | cold | 1.03–1.23 | 176, 66, 58 | 403 | 7 | 111 | 1,380 → 68 MiB |
| Tekken 3 (zip) | warm | 1.03–1.23 | 75, 66, 62 | 303 | 7 | 102 | 1,380 → 68 MiB |
| Silent Hill (zip) | first | 1.23–1.27 | unpack 3,954, 69, 59, 51 | 4,241 | unpack 8,698, 9 | 8,804 | 1,245 → 71 MiB |
| Silent Hill (zip) | cold | 1.27–1.24 | 156, 58, 51 | 365 | 8 | 120 | 1,243 → 66 MiB |
| Silent Hill (zip) | warm | 1.27–1.24 | 63, 58, 52 | 271 | 7 | 107 | 1,243 → 68 MiB |

The handheld's zips are unpacked to its internal drive, where the tool's home is, so their cold rows read from it and
not from the card; their first rows are the unpacking, the same code in both ways, which took 4.0–14.0 s with the load
rising, against 3.7–5.9 s in the morning's session (§12.8.6).

**A launch twice in a row on the handheld.** §12.8.6 found that a second launch there was not a warm one, the launch's
1.1 GB pushing the pages it had just read out of a full page cache. This afternoon, with 14 GiB of memory free, 1.2's
second launch was warm (all of its 539 MiB still cached); and 1.3's launch brings less than 1 MiB of the disc into the
cache and allocates 57–71 MiB, so it has nothing to push out and nothing to wait for, first or second [M]:
| Game | Way | Launch | Cached before | Total | Cached after |
|---|---|---|---|---|---|
| Spyro 2 - Ripto's Rage! (USA) | 1.2 | 1 | 0 MiB | 6,123 ms | 539 MiB |
| Spyro 2 - Ripto's Rage! (USA) | 1.2 | 2 | 539 MiB | 244 ms | 539 MiB |
| Crash Bandicoot - Warped (USA) | 1.3 | 1 | 0 MiB | 99 ms | 0 MiB |
| Crash Bandicoot - Warped (USA) | 1.3 | 2 | 0 MiB | 97 ms | 0 MiB |

**Big picture, from the pad's press** (the probe of §12.8.1, Normal at 3.0 s and Disabled) [M]:

Desktop, milliseconds from the press:

| Game | Cache | Normal: the step that loaded | Normal: first frame | Disabled: first frame |
|---|---|---|---|---|
| Ape Escape (sheet) | cold | 55 | 3,134 | 33 |
| Ape Escape (sheet) | warm | 22 | 3,043 | 11 |
| Castlevania (zip) | cold | 16 | 3,035 | 1,359 |
| Castlevania (zip) | warm | 23 | 3,046 | 1,348 |
| Crash Bandicoot (track) | cold | 23 | 3,043 | 28 |
| Crash Bandicoot (track) | warm | 29 | 3,052 | 6 |
| Rayman (sheet, 51 tracks) | cold | 32 | 3,050 | 33 |
| Rayman (sheet, 51 tracks) | warm | 20 | 3,037 | 11 |
| Silent Hill (zip) | cold | 15 | 3,032 | 1,028 |
| Silent Hill (zip) | warm | 140 | 3,161 | 1,008 |
| Spyro the Dragon (track) | cold | 22 | 3,044 | 24 |
| Spyro the Dragon (track) | warm | 20 | 3,044 | 6 |
| Tekken 3 (zip) | cold | 22 | 3,042 | 1,749 |
| Tekken 3 (zip) | warm | 20 | 3,040 | 1,634 |

Under the 90 MB/s cap, milliseconds from the press:

| Game | Cache | Normal: the step that loaded | Normal: first frame | Disabled: first frame |
|---|---|---|---|---|
| Ape Escape (sheet) | cold | 58 | 3,137 | 29 |
| Ape Escape (sheet) | warm | 24 | 3,046 | 12 |
| Castlevania (zip) | cold | 22 | 4,657 | 5,216 |
| Castlevania (zip) | warm | 19 | 3,041 | 1,304 |
| Crash Bandicoot (track) | cold | 24 | 3,045 | 6 |
| Crash Bandicoot (track) | warm | 15 | 3,037 | 7 |
| Rayman (sheet, 51 tracks) | cold | 19 | 3,037 | 10 |
| Rayman (sheet, 51 tracks) | warm | 26 | 3,046 | 11 |
| Silent Hill (zip) | cold | 21 | 3,737 | 3,752 |
| Silent Hill (zip) | warm | 22 | 3,041 | 1,248 |
| Spyro the Dragon (track) | cold | 17 | 3,039 | 22 |
| Spyro the Dragon (track) | warm | 16 | 3,034 | 6 |
| Tekken 3 (zip) | cold | 22 | 6,488 | 6,139 |
| Tekken 3 (zip) | warm | 24 | 3,044 | 1,446 |

Every disc that is not a zip starts at the screen's own time, 3.03–3.14 s, cold or warm and capped or not, the load
under it taking 15–58 ms of the interface's step where it took 0.30–6.6 s before §12.8.3 and 16–113 ms after it; and
with the screen Disabled, where the read was waited for with nothing moving, the first frame is 5–33 ms after the
press, against 0.15–7.4 s (§12.8.2). A zip's launch is its unpacking, since every window of the probe has a home of
its own: 1.0–1.7 s on the desktop and 3.7–6.5 s under the cap.

**Peak memory.** 53–71 MiB a launch, where it was 0.88–1.38 GB: the host's buffer and the core's copy both gone [M].

**First touches, the first 30 s of play.** A page the game has not read is read from the medium on the emulation
thread when first touched. Traced from a cold cache over 1,800 frames, each frame's time and the process's major
faults in it [M]:

- *Unpaced on the desktop under the cap*, seven games: 0 to 8 major faults in 30 s, the system's read-ahead covering
  the drive's sequential reads; the frames that faulted took at most 7 ms, but for Crash Bandicoot's frame 282, 65–80 ms
  in three runs of three, 3.9 ms without the cap.
- *Unpaced on the handheld's card*: Crash Bandicoot 16 faults, the slowest frame 6.5 ms; Spyro the Dragon 5, 8.0 ms;
  **Mega Man X5's frame 355 took 20.6 ms** for one fault, where its emulation takes 2.7 ms. A first touch can hold a
  frame past its period on the card, so P7's third clause failed as first built.
- *The remedy*, `DiscFiles.Prefetch` (`EmuSen_Galaxia.md` §3.8, `EmuSen_CoreAPI_Extensions.md` §10.2.5): the disc's
  files read once, in order, beside the game, at most a third of the medium. Paced at 60 frames a second under the
  cap, Crash Bandicoot's faults fell from 8 to 1 and Spyro's from 3 to 0, no frame over 7.5 ms; at full speed the
  prefetch had made one fault wait 160 ms behind it, which is why it rests.
- *Paced on the handheld's card*, cold, without the prefetch and with it (21:20–21:40, Mistress not running, load
  averages 0.3–1.0), the frames with a major fault, the slowest of them, and the frames over 16.7 ms:

  | Game | Without: faults, slowest, over | With: faults, slowest, over | Prefetched in 30 s |
  |---|---|---|---|
  | Mega Man X5 (556 MiB) | 4, 13.1 ms, 0 | 0, —, 0 | all |
  | Crash Bandicoot (603 MiB) | 16, 8.0 ms, 0 | 1, 6.6 ms, 0 | all |
  | Spyro the Dragon (631 MiB) | 3, 10.1 ms, 0 | 0, —, 0 | all |
  | Mega Man X6 (572 MiB), four runs each | 5–6; 8.0–31.5 ms; 0, 1, 2, 1 | 4; 23.8–49.6 ms; 2, 1, 1, 3 | 483–564 MiB |

  The prefetch removes the first touches it has had time to pass: three games play their first 30 s with none or one.
  **It does not remove those of the first seconds, and appears to lengthen them.** Mega Man X6 reads four to six new
  places of its disc between frames 125 and 343, two to six seconds into play, before a prefetch at a third of the
  card has reached them. Its slowest faulting frame was 8.5, 8.0, 31.5 and 24.1 ms in four runs without the prefetch
  and 49.6, 23.8, 38.2 and 25.7 ms in four with it: longer with it in the middle of the range, the two overlapping,
  and four runs each too few to size the difference. The handheld's interface ran beside every run, and two runs
  without the prefetch had a frame of 31.5 and of 62.1 ms with no fault in it. So **on the card some games drop one
  to three frames in their first seconds of play**, which a copied disc did not, having paid for it with 5–7 s before
  the first frame; the prefetch trades the first touches of a game's later play for somewhat longer ones at its start.
  P7's third clause is not met for those games.
  *Not built, for the tester to choose:* the prefetch at the medium's full speed until the game's first frame, which
  under the launch screen's 3.0 s would have read about 270 MiB before play begins; and, for a remedy that is exact
  whatever the game reads first, the drive's seek target raised by the core as an event, so that the host reads ahead
  of the drive during the emulated seek, which is a surface and so a proposal (`EmuSen_CoreAPI_Extensions.md` §10.2.5).


**What the player meets.** A launch from the library or big picture no longer waits for the disc, from any medium; a
game's file is read as the game reads it, and the whole disc soon after, beside it. On Linux and on the handheld a
disc's file must stay where it is while the game runs: renaming or deleting it is harmless, but the card taken out
or the file rewritten by another program ends the game, where a copied disc outlived it. On Windows the files of a
running game cannot be moved or deleted until it is closed or its disc changed. A zip's first launch still unpacks it.

**Exactness.** `EmuSen_CoreAPI_Extensions.md` §10.2.5: the kit's C17 on the seven discs at 1,800 frames, every frame,
sample and state of a lent run the copied run's; and WiseMan's `DiscStreamingTests` through the C# host.

**Tests run** (2026-10-12, after `git merge WiseMan`, which brought nothing new). `cargo test --release`: emusen-disc
12 and 11 (19 before), emusen-native 69 (63), emusen-core-abi-check 6 (5) with `--check --against` the base branch's
baseline, emusen-core-conform 7 and 16, the kit's tests three parallel runs of three; the platform workspace 25, 85,
25 and 30; Diamond 157 and 76, with ps1-tests and without; VenusRT 143, Nephrite 140, MoonRT 6, MercuryRT 8, 16 and 4,
over the new kit. WiseMan by area, `CoreAbi`, `CoreAdapter`, `CoreDiscovery`, `CoreEngine`, `CoreRegistration`,
`CoreFactory`, `Diamond` (with `DiamondPlayers`), `DiscStreaming`, `PlayStationLibrary`, the whole `Galaxia` folder
with the parity tests, `RomLibrary`, `SystemPack`, `Firmware`, `EmulatorSession`, `LaunchScreen`, `LeafAssembly`,
`VenusRt`, `Nephrite`, `MoonRt` and `MercuryRt`: 981 cases, 972 passed and 9 skipped (the opt-in probes without their
folders) [M]. The registration-equivalence golden is unchanged, so P1 holds for 1.3 too. After WiseMan `fd680853`
was merged in (the GPU's D-47), Diamond's own tests again, 158 and 77 with ps1-tests and without, and WiseMan's
`Diamond`, `DiscStreaming`, `PlayStationLibrary`, `CoreEngine`, `CoreAbi`, `CoreAdapter`, `CoreRegistration` and
`GalaxiaParity`, 152 passing [M].

#### 12.8.10 The drive's place named ahead: minor 1.4 measured (2026-10-12)

*Decided 2026-10-12, of §12.8.9's two remedies, the exact one: the core ABI's minor 1.4, by which Diamond's drive names
the file and byte it is going to read and the host has the system read there during the emulated seek
(`EmuSen_CoreAPI.md` §6.25; the design and its record `EmuSen_CoreAPI_Extensions.md` §10.2.6 and §10.2.7). Decided the
same day: the measurement is the desktop's, under the 90 MiB/s cap; its confirmation on the handheld's card is left to
play.* The marks are §12's; [M] is measured on this date, after WiseMan `a253a250` was merged.

**How it was measured.** A console tool over this branch's `EmuSen.dll` (`discseek/tool` in the probe's cache) makes
one launch per process as the engine's steps make it (Galaxia's `DiscSet` and `DiscFiles`, one create lent the
files, a card file passed), then runs 1,800 frames paced at 60 a second, draining the events after each frame, four
ways: **1.3's prefetch** (each file read once in order beside the game, as `DiscFiles.Prefetch` did), **the event**
(each `MEDIA_SEEK` handed to the disc's reader, as `CoreEngine` does), **both**, and **neither**. For each frame it
records its time, the emulation thread's own major faults (`/proc/thread-self/stat`) and its time off the processor
without waiting to run (`schedstat`), read without allocating. A frame is *held by a wait* when it took longer than
16.7 ms and in it the thread took a major fault or spent 1 ms or more off the processor. Each run is cold, the disc's
pages dropped by `posix_fadvise`, and under `IOReadBandwidthMax=90M` on the library's drive, as §12.8.1's. The four
discs are copies under the probe's cache. Other work was running on the machine: the load average is beside each run.

**What the cap models.** Under it a cold page took 1.9–3.3 ms, the card's 2.5–3.1 ms, and the sustained rate is the
card's 88–92 MiB a second. A short read it does not: an advised 512 KiB arrived in 1.4 ms, where the card takes 7.3 ms,
and a request-rate cap tried beside it held single reads 50–154 ms rather than adding a few milliseconds to each, so
it was not used (`EmuSen_CoreAPI_Extensions.md` §10.2.7).

Mega Man X6:

| Way | Run | Load | Frames over 16.7 ms, held by a wait | Frames with a major fault, the slowest | Slowest frame | Median, 99th percentile | Places named, KiB advised |
|---|---|---|---|---|---|---|---|
| 1.3: the prefetch | 1 | 1.29 | 1, 1 | 3, 194.7 ms | 194.7 ms at 125 | 0.66, 3.43 ms | 18, — |
| 1.3: the prefetch | 2 | 2.46 | 1, 1 | 3, 116.2 ms | 116.2 ms at 176 | 0.67, 3.60 ms | 18, — |
| 1.3: the prefetch | 3 | 2.00 | 0, 0 | 4, 5.2 ms | 5.8 ms at 56 | 0.67, 3.46 ms | 18, — |
| 1.3: the prefetch | 4 | 1.80 | 1, 1 | 4, 47.4 ms | 47.4 ms at 125 | 0.69, 3.64 ms | 18, — |
| 1.3: the prefetch | 5 | 2.94 | 1, 1 | 3, 130.3 ms | 130.3 ms at 176 | 0.66, 7.49 ms | 18, — |
| 1.3: the prefetch | 6 | 2.54 | 1, 1 | 3, 27.0 ms | 27.0 ms at 125 | 0.66, 3.37 ms | 18, — |
| 1.4: the event | 1 | 1.76 | 0, 0 | 0, — | 7.9 ms at 459 | 0.89, 3.75 ms | 18, 41,856 |
| 1.4: the event | 2 | 1.92 | 0, 0 | 0, — | 6.0 ms at 14 | 0.74, 3.44 ms | 18, 41,856 |
| 1.4: the event | 3 | 1.80 | 0, 0 | 0, — | 4.4 ms at 439 | 0.66, 3.42 ms | 18, 41,856 |
| 1.4: the event | 4 | 1.81 | 0, 0 | 0, — | 5.1 ms at 24 | 0.72, 3.69 ms | 18, 41,856 |
| 1.4: the event | 5 | 2.69 | 0, 0 | 0, — | 5.6 ms at 3 | 0.67, 3.76 ms | 18, 41,856 |
| 1.4: the event | 6 | 1.77 | 0, 0 | 0, — | 5.7 ms at 216 | 0.66, 3.46 ms | 18, 41,856 |
| both | 1 | 3.26 | 0, 0 | 0, — | 5.3 ms at 243 | 0.70, 3.62 ms | 18, 41,856 |
| both | 2 | 2.55 | 0, 0 | 0, — | 5.0 ms at 219 | 0.69, 3.58 ms | 18, 41,856 |
| both | 3 | 1.68 | 0, 0 | 0, — | 5.9 ms at 133 | 0.73, 3.68 ms | 18, 41,856 |
| both | 4 | 1.72 | 0, 0 | 0, — | 5.9 ms at 18 | 0.66, 3.50 ms | 18, 41,856 |
| both | 5 | 2.18 | 0, 0 | 0, — | 5.1 ms at 100 | 0.72, 3.24 ms | 18, 41,856 |
| both | 6 | 1.32 | 0, 0 | 0, — | 5.6 ms at 89 | 0.65, 3.35 ms | 18, 41,856 |
| neither | 1 | 2.80 | 0, 0 | 3, 5.6 ms | 6.0 ms at 86 | 0.72, 3.93 ms | 18, — |
| neither | 2 | 2.32 | 0, 0 | 3, 5.2 ms | 5.2 ms at 176 | 0.69, 3.51 ms | 18, — |
| neither | 3 | 1.60 | 0, 0 | 4, 5.0 ms | 5.0 ms at 176 | 0.68, 3.67 ms | 18, — |
| neither | 4 | 1.60 | 0, 0 | 3, 6.5 ms | 6.5 ms at 176 | 0.74, 3.55 ms | 18, — |
| neither | 5 | 3.74 | 0, 0 | 3, 4.9 ms | 5.6 ms at 53 | 0.65, 3.93 ms | 18, — |
| neither | 6 | 1.36 | 0, 0 | 3, 5.0 ms | 5.0 ms at 125 | 0.66, 3.64 ms | 18, — |

Mega Man X5:

| Way | Run | Load | Frames over 16.7 ms, held by a wait | Frames with a major fault, the slowest | Slowest frame | Median, 99th percentile | Places named, KiB advised |
|---|---|---|---|---|---|---|---|
| 1.3: the prefetch | 1 | 1.15 | 0, 0 | 0, — | 6.8 ms at 1158 | 2.60, 4.53 ms | 16, — |
| 1.3: the prefetch | 2 | 1.01 | 0, 0 | 0, — | 5.6 ms at 1588 | 2.81, 4.37 ms | 16, — |
| 1.3: the prefetch | 3 | 0.80 | 0, 0 | 0, — | 7.2 ms at 1540 | 2.65, 5.97 ms | 16, — |
| 1.4: the event | 1 | 1.16 | 0, 0 | 0, — | 6.1 ms at 1030 | 2.60, 4.54 ms | 16, 37,760 |
| 1.4: the event | 2 | 0.95 | 0, 0 | 0, — | 6.5 ms at 1521 | 2.80, 4.23 ms | 16, 37,760 |
| 1.4: the event | 3 | 1.01 | 0, 0 | 0, — | 6.2 ms at 1588 | 2.65, 4.48 ms | 16, 37,760 |
| both | 1 | 1.15 | 0, 0 | 0, — | 6.1 ms at 1537 | 2.85, 4.60 ms | 16, 37,760 |
| both | 2 | 0.81 | 0, 0 | 0, — | 5.8 ms at 532 | 2.70, 4.39 ms | 16, 37,760 |
| both | 3 | 1.60 | 0, 0 | 0, — | 6.7 ms at 854 | 2.72, 4.74 ms | 16, 37,760 |
| neither | 1 | 1.45 | 0, 0 | 2, 4.8 ms | 6.3 ms at 1009 | 2.67, 4.87 ms | 16, — |
| neither | 2 | 1.20 | 0, 0 | 2, 5.5 ms | 5.5 ms at 141 | 2.59, 4.39 ms | 16, — |
| neither | 3 | 1.39 | 0, 0 | 3, 8.0 ms | 8.0 ms at 99 | 2.87, 5.79 ms | 16, — |

Crash Bandicoot (USA):

| Way | Run | Load | Frames over 16.7 ms, held by a wait | Frames with a major fault, the slowest | Slowest frame | Median, 99th percentile | Places named, KiB advised |
|---|---|---|---|---|---|---|---|
| 1.3: the prefetch | 1 | 1.86 | 0, 0 | 1, 7.1 ms | 7.1 ms at 145 | 2.31, 4.34 ms | 18, — |
| 1.3: the prefetch | 2 | 1.06 | 1, 1 | 1, 131.7 ms | 131.7 ms at 145 | 2.22, 4.59 ms | 18, — |
| 1.3: the prefetch | 3 | 1.74 | 0, 0 | 1, 5.8 ms | 5.8 ms at 145 | 2.34, 4.48 ms | 18, — |
| 1.4: the event | 1 | 1.67 | 0, 0 | 0, — | 6.3 ms at 74 | 2.32, 4.52 ms | 18, 43,008 |
| 1.4: the event | 2 | 1.08 | 0, 0 | 0, — | 5.8 ms at 1642 | 2.22, 4.56 ms | 18, 43,008 |
| 1.4: the event | 3 | 1.39 | 0, 0 | 0, — | 5.8 ms at 1536 | 2.31, 4.39 ms | 18, 43,008 |
| both | 1 | 1.41 | 0, 0 | 0, — | 5.9 ms at 1632 | 2.32, 4.43 ms | 18, 43,008 |
| both | 2 | 1.02 | 0, 0 | 0, — | 7.0 ms at 1606 | 2.18, 4.65 ms | 18, 43,008 |
| both | 3 | 1.20 | 0, 0 | 0, — | 5.9 ms at 1694 | 2.33, 4.38 ms | 18, 43,008 |
| neither | 1 | 1.26 | 0, 0 | 8, 5.8 ms | 5.8 ms at 145 | 2.33, 4.55 ms | 18, — |
| neither | 2 | 1.45 | 0, 0 | 8, 5.6 ms | 5.9 ms at 1298 | 2.22, 4.53 ms | 18, — |
| neither | 3 | 1.23 | 0, 0 | 8, 5.4 ms | 6.3 ms at 1628 | 2.33, 4.63 ms | 18, — |

Spyro the Dragon (USA):

| Way | Run | Load | Frames over 16.7 ms, held by a wait | Frames with a major fault, the slowest | Slowest frame | Median, 99th percentile | Places named, KiB advised |
|---|---|---|---|---|---|---|---|
| 1.3: the prefetch | 1 | 1.02 | 0, 0 | 0, — | 8.4 ms at 1369 | 1.03, 6.36 ms | 12, — |
| 1.3: the prefetch | 2 | 1.29 | 0, 0 | 0, — | 7.8 ms at 1373 | 1.05, 5.94 ms | 12, — |
| 1.3: the prefetch | 3 | 1.38 | 0, 0 | 0, — | 7.7 ms at 1261 | 1.11, 6.23 ms | 12, — |
| 1.4: the event | 1 | 0.83 | 0, 0 | 0, — | 8.6 ms at 1761 | 1.05, 6.01 ms | 12, 28,288 |
| 1.4: the event | 2 | 1.28 | 0, 0 | 0, — | 7.3 ms at 1315 | 1.08, 6.14 ms | 12, 28,288 |
| 1.4: the event | 3 | 1.22 | 0, 0 | 0, — | 8.7 ms at 793 | 1.08, 6.27 ms | 12, 28,288 |
| both | 1 | 1.11 | 0, 0 | 0, — | 7.8 ms at 1217 | 1.16, 6.46 ms | 12, 28,288 |
| both | 2 | 0.96 | 0, 0 | 0, — | 7.6 ms at 1219 | 1.07, 5.94 ms | 12, 28,288 |
| both | 3 | 1.12 | 0, 0 | 0, — | 8.3 ms at 1513 | 1.06, 6.36 ms | 12, 28,288 |
| neither | 1 | 0.90 | 0, 0 | 3, 6.8 ms | 8.0 ms at 857 | 1.05, 6.05 ms | 12, — |
| neither | 2 | 1.06 | 0, 0 | 3, 7.7 ms | 8.7 ms at 1417 | 1.04, 5.99 ms | 12, — |
| neither | 3 | 1.38 | 0, 0 | 3, 6.8 ms | 8.4 ms at 1403 | 1.04, 6.34 ms | 12, — |

**In brief** [M]: with the event, **no run of any of the four games took a major fault** in its first 30 s, and no frame
took longer than 8.7 ms; the places named were 12 to 18 a game, 28–42 MiB advised. With 1.3's prefetch, Mega Man X6 had
one frame held 27–195 ms in five runs of six and Crash Bandicoot one of 132 ms in one run of three, each a single
fault waiting in the cap's throttle behind the bandwidth the prefetch had spent, which overstates what the card does
(24–50 ms, §12.8.9). With neither, every game faulted 2 to 8 times, each fault 4.8–8.0 ms on the desktop. "Both" was
the event's figures. **So `EmuSen_CoreAPI_Extensions.md`'s P8 holds on the desktop under the cap**, and the prefetch,
which added nothing the event did not do sooner, is removed (`EmuSen_Galaxia.md` §3.8).

**What the player meets.** As in §12.8.9, a launch waits for nothing of the disc; now a game's first seconds read
the places the drive goes to before it is there, and nothing reads the rest of the disc beside the game. Whether the
handheld's card agrees is left to play.

**Tests run** (2026-10-12, after `git merge WiseMan`). `cargo test --release`: emusen-native 71, emusen-core-abi-check 6
with `--check --against` the base branch's baseline, emusen-core-conform 7 and 17, emusen-disc 12 and 12 with chdman
named and without, the platform workspace 25, 86, 1, 25 and 30; Diamond 161 and 78, with ps1-tests and without; over
the changed kit crate VenusRT 143, Nephrite 140, MoonRT 6, MercuryRT 8, 16 and 4, MarsRT 490. WiseMan
by area, `CoreAbi`, `CoreAdapter`, `CoreEngine`, `CoreRegistration`, `CoreDiscovery`, `Diamond`, `DiscStreaming`,
`PlayStationLibrary`, the whole `Galaxia` folder with the parity tests, and `LaunchScreen`: 353 cases, 344 passed and 9
skipped (the opt-in probes without their folders) [M].

## 13. Gameplay: the seven games driven into play (2026-10-12)

*Written 2026-10-12, on WiseMan `66bb54b5` and the merges after it (`c27bbfbf`, D-41; `60dce4e5`, stage 13 and the
latched input).* After D-45 every one of the tester's seven games reached its title on the open kernel; none had been
driven past it. Each was then played by a script of presses, the same script on Diamond (open kernel) and on
SwanStation (the player's American BIOS, by path), to about frame 18,000. The marks are §12's; [M] is measured on this date.

### 13.1 How they were driven

- **The scripts** are `EmuSen.WiseMan/Reference/analysis/psxplay/*.txt`, buttons and frames only, each counted from an
  anchor T, the frame the game's title (or its first screen after the BIOS's part) shows on each emulator, since
  SwanStation boots through the BIOS's intro and every load after it runs on the referee's own clock. Each script's
  second line gives T on both. `psxplay.py` drives a disc by one of them on Diamond's run tool (`examples/run`, its new
  `--stick` beside `--press`) or on SwanStation through `psxreferee.py pictures`, which gained the presses, a stick,
  the device, core options, a card in and out and the sound (`Diamond_Referees.md` §4.16). Pictures every 60 frames on
  both, kept in the cache, never committed.
- **The cards.** Every game but Spyro the Dragon was given a card formatted by the open kernel's own card manager
  (`formatted-by-open-kernel.mcr` of `Diamond_OpenKernel_Phase2.md` §20.4, MD5 `b57f32ef…`); Spyro a blank card of
  zeros. The same file went to SwanStation as its save RAM.
- **Ape Escape** needs a DualShock in analog mode: Diamond's DualShock answers the game's own configuration commands;
  SwanStation was given its DualShock (`--device 1=261`) with `swanstation_Controller1_ForceAnalog=true`.
- **The comparison**: for each 1,200 frames of Diamond's run, the referee offset (searched within 600 frames of the
  last) that gives the smallest median mean-absolute grey difference on 80×60, and that median; and by eye, from
  contact sheets of both runs. Once a game's path depends on its own randomness or on a load's length (Tekken 3's
  opponent, Spyro's flight), the medians grow without anything being wrong, and the eye decides. **The sound**: the
  SPU's output and the core's audio, RMS per 1,200 frames at the same offset.

### 13.2 The games

**Crash Bandicoot** (T: 2,520 on Diamond, 3,900 on SwanStation)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **yes**: the map at T+420, N. Sanity Beach's loading screen, play from T+840 | the same, from T+840 |
| Play against the referee | the same path step for step; window medians 3.3 to 5.3 grey levels, one window of 12.3 while the camera turns | — |
| Pictures | nothing wrong seen | — |
| Sound | −22.9 dB, the same as the referee's (median difference 0.0 dB) | −22.9 dB |
| Hangs | none: at T+4,600 the script walks Crash into a hollow where he stays, the camera resting on the same scene on both to the end of the run | the same |
| Cards | no save made by the script | — |

The game's TTY prints `intr timeout(0040:004d)` six times in 15,500 frames: §13.4's third item.

**Spyro the Dragon** (T: 1,980 and 3,480)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **yes**: the card menus, the opening, the Artisans' home world from T+2,940 | the same |
| Play against the referee | to T+4,800 within 1.4 to 5.3; then the two flights part, and by eye both play alike (flying, gems, dragons freed, "Entering Stone Hill") | — |
| Pictures | nothing wrong seen. *Before D-46* the logos and the title came 420 frames late against the referee's pace (title at 2,400) | — |
| Sound | −22.3 dB (median difference 0.1 dB) | −22.4 dB |
| Hangs | **on the card**: with a formatted card and "create a save file" chosen, "CREATING SAVE FILE…" stays up for ever; with a card holding a save made on SwanStation, "ACCESSING MEMORY CARD…" stays up for ever (§13.4) | creates the file and goes on; the save it made reads back |
| Cards | the blank card left as it was (the script declines to format it) | the same |

**Tekken 3** (T: 6,540 and 7,440)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **yes**: Arcade, Xiaoyu, the first fight from T+1,000; two stages fought, rounds won and lost, "CONTINUE?" at the end of the run | the same course, other opponents |
| Play against the referee | the opponent is the game's random choice (Hwoarang, Law, Jin, Lei, Paul across runs), so the medians are 31 to 42 throughout; by eye both fight alike | — |
| Pictures | *before D-46*: the stage drawn and both fighters in pieces, an arm and a leg each, in every fight. *After*: whole, as on the referee | — |
| Sound | −15.2 dB (median difference 0.6 dB) | −16.8 dB |
| Hangs | none | none |
| Cards | the open kernel's formatted card in slot 1; no save made by the script (saves and loads between the kernels are `Diamond_OpenKernel_Phase2.md` §22.1's) | the same |

**Castlevania: Symphony of the Night** (T: 1,320 and 2,340)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **no**: PRESS START BUTTON taken at T+60, then black to the end of the run | the file screen at T+270, a new game, the name, the prologue (Richter at Dracula's castle) from T+1,800 |
| Hangs | at T+265, frame 1,585: the game asks B(51h) Krom2RawAdd for the Shift-JIS characters 8163h and 819Ah to draw its file screen, the open kernel answers −1 (it carries no character set), the game reads a byte at FFFFFFFFh and the bus error it takes never returns (§13.4) | — |

**Silent Hill** (T: 1,320 and 2,340, the fear-of-blood screen's text)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **yes**: New Game, Normal, the opening movie skipped, the opening scene with Cheryl to T+10,500, then Harry walking the foggy street | the same |
| Play against the referee | window medians 0.9 to 6.6 to T+15,500, at a constant offset of 1,020 frames through the opening | — |
| Pictures | nothing wrong seen: the fog, the snow, the dialogue boxes | — |
| Sound | −24.1 dB (median difference 0.0 dB) | −24.1 dB |
| Hangs | none | none |

**The fear-of-blood screen** (the question D-45's step left): at 10-frame steps, with a blank card, its text fades in
at frame 1,210 on Diamond and 2,210 on SwanStation, the picture behind it at 1,390 and 2,430, the cut to the movie's
first picture at 2,180 and 3,210: 180 against 220 frames from the text to the picture, and 970 against 1,000 from the
text to the cut, so the screen holds as long on both, within 40 frames [M]. With no card, both show the "no MEMORY
CARD" warning for 300 frames first, and the screen then holds 180 frames on both. The 350 against 90 frames recorded
in `Diamond_CDROM.md` §9.8 is not reproduced, on this build nor on WiseMan `66bb54b5` before D-46 [M]; what those two
figures measured is not known.

**Ape Escape** (T: 5,160 and 6,300)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **yes**: NEW GAME, the professor's scene, the first stage from T+1,800, Spike walked by the left stick about the stage, the tutorial's boxes | the same |
| Play against the referee | to T+1,200 within 2.0; then the scenes' skips land at different moments and the stage's play parts; by eye both alike | — |
| Pictures | nothing wrong seen | — |
| Sound | −17.9 dB (median difference 0.1 dB) | −17.8 dB |
| Hangs | none | none |

**Rayman** (T: 1,740 and 3,060, the opening movie's first table)

| | Diamond | SwanStation |
|---|---|---|
| Reaches play | **yes**: the movie skipped, the title, "Use memory card", a game chosen and named, the Dream Forest from T+3,840 | the same |
| Play against the referee | to T+2,400 within 0.4 to 5.7; then the scripts' presses meet different menus (Diamond's run opened the options once) and the play parts; by eye both alike | — |
| Pictures | nothing wrong seen | — |
| Sound | −29.3 dB (median difference 1.1 dB) | −30.6 dB |
| Hangs | none | none |
| Cards | **saved** on the open kernel's formatted card: a file `BISLUS-00005aaa63281` in block 2 and the same name erased in block 1, the game's own create-then-erase; every frame's checksum valid | saved `BISLUS-00005aaa63120` in block 1 on the same card |

### 13.3 What was found and fixed

1. **A triangle's setup of 876 GPU cycles** (D-46). Tekken 3's fights left both fighters half drawn: the frame's 729
   triangles took 0.71 of a frame in setup alone. The setup is gone and a triangle is charged 3.65 cycles a row it
   spans, its sides clipped or not, which keeps `gpu/bandwidth` within a hblank of the console's log and makes a small
   triangle as cheap as the referees read it. *Proof*: `a_triangle_is_charged_by_its_rows`, failing before. Tekken 3's
   fighters are whole and Spyro's logos keep the referee's pace; Castlevania's pictures to its stop are byte for byte
   as before, and the other games' attract runs keep their scenes, some of them at a pace that moves by up to a few
   seconds once their 3D scenes begin, as the referee's alignment above follows [M].

### 13.4 What is still wrong, ranked

1. *Fixed 2026-10-12 (`Diamond_OpenKernel_Phase2.md` §25.1): the call answers a box, and the game plays its
   prologue.* ~~**Castlevania stops at its file screen on the open kernel** (above): B(51h) Krom2RawAdd answers −1 and the game
   dereferences it. The kernel's own decision (`Diamond_OpenKernel_Phase2.md` §19, "absent from an open kernel"); the
   game uses the call for its English menus. The kernel is not this step's to change: recorded for its writer
   (`Diamond_OpenKernel_Phase2.md` §24).~~
2. *Fixed 2026-10-12 (§25.2 there): an accepted asynchronous request returns 0; the game creates, writes and reads
   its file.* ~~**Spyro's saves and loads hang on the open kernel**: the game opens its file asynchronously (access mode 8001h), then
   calls B(34h) read or B(35h) write in a loop that ends only when the call returns 0 (its code at
   80067018h–8006702Ch and 800672D4h–800672E8h). The open kernel returns the length at once, as psx-spx's "Number of
   bytes" reads; the second call then waits for the first job and returns C00h, and every call after it −1. That the
   loop ends on 0 is the game's evidence of what the call returns in asynchronous mode. Kernel territory, recorded for
   its writer (§24 there). The cards themselves are not at fault: SwanStation reads the open kernel's formatted card
   as formatted and empty, and creates Spyro's file on it.~~
3. *Fixed 2026-10-12 (§25.3 there): the exception path runs from the kernel's RAM and an empty port is given up 100 µs
   after its byte; a vblank reaches the game's handler after 15,881 cycles, timer 2 is acknowledged after 1,507, and
   the game prints no "intr timeout" over its script.* ~~**Every interrupt costs the open kernel 6,000 to 9,000 cycles before it is acknowledged** [M, frame 6,660 of Crash
   Bandicoot's run, the kernel's stores to I_STAT and SIO0 timed]: timer 2, which Crash runs at its target every 32,776
   cycles, is acknowledged 5,800 to 8,800 cycles after its exception, and on a vblank the game's own handler is reached
   36,434 cycles after it (8,052 before the pads, 9,253 for port 1's five bytes, 9,800 for empty port 2, 9,191 after),
   longer than timer 2's period, so timer 2 stands in I_STAT whenever the game's handler looks. That handler counts each
   look that leaves an interrupt standing and at 2,049 prints "intr timeout" and writes 0 to I_STAT, which would drop any
   other interrupt standing then. No harm was seen in Crash's run; the console's own figures are not known, since the
   player's BIOS is never traced. Recorded for the kernel's writer; Diamond's SIO0 bytes take their documented time.~~
4. **The scripts are not yet long walks through each game**: Crash's ends in a hollow, and the Tekken, Spyro, Ape
   Escape and Rayman runs part from the referee's as each game's randomness or a load's length decides, after which
   only the eye compares them.

### 13.5 The checks kept

- ps1-tests' pictures and records: `cargo test --release` in the crate with `EMUSEN_DIAMOND_PS1TESTS` set and without
  it, every test passing (§13.7).
- The seven titles: every game still reaches its title on the open kernel with no presses (runs to frame 7,000 after
  each change): Castlevania at 1,320, Silent Hill's warnings and movie, Rayman's movie and title, Crash Bandicoot at
  2,520, Ape Escape at 5,160, Tekken 3 at 6,540, and Spyro the Dragon at 1,980, 420 frames sooner than before (D-46).
- The cards: the open kernel's formatted card read as formatted by SwanStation on the player's BIOS (Spyro, Rayman) and
  by the games on the open kernel (Rayman, and Silent Hill, which warns of no card otherwise); Rayman saved to it on both.

### 13.6 The cost

[M, `examples/run` built from WiseMan `60dce4e5` and from this step's last commit, both with
`RUSTFLAGS="-C llvm-args=-align-all-functions=6"`, run alternately on one processor, the fastest of five, ms a frame,
before → after]: `timers` 0.337 → 0.338, `gpu/triangle` 2.153 → 2.161, `gpu/bandwidth` 6.683 → 6.673,
`gpu/benchmark` 3.885 → 3.936, `spu/test` 0.294 → 0.295, `mdec/movie-15bit` 1.656 → 1.662; Crash Bandicoot's first
600 frames 2.675 → 2.743, Tekken 3's first 3,000 3.015 → 3.118, Spyro the Dragon's first 3,000 3.244 → 3.101.
The programs differ within what placement moves. The games differ because they now run differently: a GPU that is
free sooner leaves the idle skipper less to skip in Crash's and Tekken 3's frames [A], and Spyro reaches its title 420
frames sooner. The change adds no work: one counter moved above a clip and a constant changed.

### 13.7 Tests run (2026-10-12)

`cargo test --release` in Diamond's crate after the merge of `60dce4e5`: 155 unit tests and 69 program tests, with and
without `EMUSEN_DIAMOND_PS1TESTS`; WiseMan's `Diamond` tests under the test lock, 42 of 42. The probe's own tests
(`cargo test --release` in `probe-rs`), 50 of 50.

### 13.8 Read for this section, and exposures

psx-spx `6d7d1bc`: `kernelbios.md`'s "BIOS File Functions" (open, lseek, read, write, close), "Asynchronous Access",
B(19h) HookEntryInt and C(0Ah) ChangeClearRCnt; "DMA Channels" and "GPU Timings" for D-46. ps1-tests' `gpu/benchmark`
source (`main.cpp`, `modes.h`), its screenshot and the suite's README; the `bandwidth` console log. EmuSen's own: the
open kernel's `exception.rs` (its chain elements and ChangeClearRCnt's flags), Diamond's GPU, the probe and
`psxreferee.py`. The games' own code and RAM on the open kernel, through the run tool's `--game` watch and a debugging
build outside the repository that printed GP0 words, GTE commands and stores by address. The referees ran as black
boxes, their pictures, sound, card files and RAM at a test program's done word only; the player's BIOS was given to
them by path and to nothing else, and nothing ran on Diamond but the open kernel. No emulator's source, no firmware or
listing of one, and none of the documents `DOCS.txt` excludes was opened.

### 13.9 An eighth game: Mega Man Legends (2026-10-12)

The tester's copy stopped at a black screen two frames after its entry on the open kernel; the kernel's writer found
and fixed three defects (`Diamond_OpenKernel_Phase2.md` §26, the first of them in the form of §27). Its script is
`psxplay/mega-man-legends.txt`, T the CAPCOM logo at frame 440 on the merged image; it has not been run on a referee.
With §27's image and that script the game was driven to frame 12,600: the opening scene, the first ruin's instructions,
its corridor and a fight with Reaverbots (§27.6).

| | Diamond |
|---|---|
| Reaches play | **yes**: the Capcom animation and logo, "now loading…", the story's text over the sea; with the script the title, NEW GAME, Mega Man in the first ruin from about frame 3,360, moving with the presses, a shutter lifted, a corridor walked and the refractor's scene |
| Pictures | nothing wrong seen |
| Cards | the open kernel's formatted card in slot 1; no save made by the script |

## 14. Two reports from the handheld: Crash Bandicoot's right edge, and Mega Man Legends' box (2026-10-12)

*Written 2026-10-12, on WiseMan `6141ad60`.* The tester played on the handheld and reported two faults: in Crash
Bandicoot "weird graphical artifacts to the right of the screen in game", and in Mega Man Legends, "after Roll said
some dialogue in game. it wouldnt close the dialogue box", the game closed by hand. The second ran on the player's
own BIOS (the declared American file) with one cheat on, "Infinite Life", the GameShark code `800B521E 00C4`, at a steady 60
frames a second for ten minutes; its log holds pacing lines only. The marks are §12's; [M] is measured on this date.

### 14.1 Crash Bandicoot: the ground at the right of the screen (D-47)

- **Reproduced** [M] on the desktop on the open kernel with `psxplay/crash-bandicoot.txt`: at frame 12,600 a wedge of
  the right third, ground that SwanStation draws sand-textured, is a flat olive with edges torn along polygons. Counted
  as pictures holding more than 500 pixels of that olive (49,49,0), 58 of the run's 300 to frame 18,000, in three
  stretches (3,540–4,200, 12,420–13,680, 15,300–16,680), wherever the camera puts ground just past the screen's right
  edge.
- **Cause** [M, a debugging build outside the repository that logged a frame's GP0 commands, every polygon as placed,
  the GTE's RTPT and NCLIP results and FLAG, and the command that last wrote each VRAM pixel]: the olive is the frame's
  clear fill, GP0(02h) with 003236h, showing through where no polygon drew. The game sent the polygons (as many to the
  faulty buffer as to a whole one) and the GTE raised no screen-coordinate flag; the GPU dropped them. The game draws
  its right buffer with the offset at 768, so a vertex at SX 286, just past the screen, sums to 1,054, which the 11-bit
  rule of D-23 wraps to -994; the rest of the polygon is placed from it and lies off the area. The coordinates are 11
  bits, so -994 and 1,054 are the same column; Diamond had treated them as different ones (`Diamond_Disputes.md` D-47).
- **Proof**: `programs/offset-fold.s` and `a_primitive_past_the_offsets_eleven_bits_is_drawn_modulo_2048` fail before
  the change (the triangle from 286 reads 0 where the referees draw it), as does the unit test
  `a_primitive_reaching_past_minus_1024_is_drawn_2048_further_on`. DuckStation, SwanStation and Beetle PSX read the
  program back word for word as Diamond does after the change.
- **Fix**: `raster.rs`'s `fold`, applied by `mod.rs` to each triangle and line segment it hands the rasteriser: one
  reaching left of -1,024 (or above it) is drawn 2,048 further on.
- **Before and after** [M]: no picture of the run holds the olive after the change (0 of 300). The right third's mean
  grey difference from SwanStation's picture 1,020 frames later (its BIOS's intro), at 12,600, 13,200, 15,600 and
  16,200: 17.1, 10.2, 14.3 and 8.4 before, 11.4, 0.7, 5.4 and 0.7 after; at 12,600 the ground is sand to the edge as on
  the referee, and what remains is the crab, which stands elsewhere on the two.
- **The other six games** [M, `d5` runs of §13 again with both builds, the same presses]: Silent Hill, Ape Escape,
  Tekken 3, Spyro the Dragon, Rayman and Castlevania, 280, 344, 367, 291, 287 and 280 pictures, every one byte for byte
  the same before and after: none of their runs places a primitive past -1,024.
- amidog's OFFSET, POSITION, SIZE and AREA groups print the same TTY, line for line, before and after (D-47).

### 14.2 Mega Man Legends: the box after Roll's lines

- **The cheat** [R, `PsxCheatFormats.cs`]: `800B521E 00C4` is type 80h, decoded as `Write(l, 2, Set)` at
  `Command & 1FFFFFh` = 0B521Eh with the line's value 00C4h: a 16-bit write of 00C4h to RAM 0B521Eh each frame, the
  write the code means in RetroArch's cheat files. A decoding difference was the only way it could matter, and there is
  none. The run tool gained `--poke ADDR:VALUE:WIDTH` to apply it; every Diamond run below has it on.
- **The hypothesis** (a voiced box that closes when its clip ends, and a clip whose end Diamond never reports)
  [M, the controller's trace on both kernels and the game's own code read from RAM on the open kernel]: Roll's radio
  lines in the first ruin are five clips, channels 0 to 4 of `XA1806E.XA` (32 channels, 18,900 Hz mono, interleaved
  1/32 with data sectors of channel 0 that the game reads as it plays). Each is Setfilter (1, n), Setloc 10:57:46 (the
  file's start), ReadS, and ends with a Pause; the game decides the end itself, from GetlocP's position against an end
  sector (its code at 8001C9D4h–8001CA64h), about 30 sectors past the channel's last sector, the one with the submode's
  EOF bit. So the box's end depends on neither the EOF bit, INT4, nor the read's end, and on Diamond every clip ends,
  its box closing as the voice stops. *Rates and phases* [M, 48 runs from a state at frame 8,990, Cross every 7 to 53
  frames at four or more phases each]: the same five clips, the same CD commands and the same boxes every time; the
  lines advance with the voice, not with Cross. *The second attempt*: on the open kernel one of channel 0's data
  sectors (49,713) waited behind an interrupt and was then refused by the filter, as psx-spx's delivery rule has it,
  and the clip still ended; on the player's BIOS none waited. *The player's BIOS* [M, six runs each kernel with the
  Cross presses shifted by 7 to 35 frames]: the same commands and boxes on both. The hypothesis is refuted for these
  clips.
- **The box that does not close** [M, `psxplay/mega-man-legends-prompt.txt`, Cross every 30 frames after NEW GAME and
  Up for 90 frames every 900]: when Roll's last line ends the game shows its instructions and then asks "Read
  instructions again?" with the cursor on **Yes**. Cross takes Yes, and the instructions are shown again, then the
  question again: to the end of each run (80 seconds of boxes after Roll's lines) on Diamond's open kernel, on Diamond
  with the player's BIOS, and on SwanStation on the player's American BIOS with no cheat, whose prompt and cursor are the same
  picture as Diamond's. The box closes only with Right, then Cross on No, which is what `mega-man-legends.txt` does.
  This is the game's own prompt, and the course SwanStation follows; it is the fault the tester described as far as
  the description goes ("after Roll said some dialogue in game", a box that will not close, a game running at full
  pace), but the tester's play itself was not recorded, so that it is the same box is an inference [A].
- **The cheat's part**: none: the decoded write is the code's, and the referee loops without it.
- **Not reached**: the game past the first ruin's corridors (about frame 14,000 of the script, Mega Man in the second
  corridor after the first Reaverbots); any clip of the game's 47 other XA files.

### 14.3 The cost

[M, `examples/run` built from WiseMan `6141ad60` and from this step's last change, both with
`RUSTFLAGS="-C llvm-args=-align-all-functions=6"`, run alternately on one processor, the fastest of five, ms a frame,
before → after, the emulated cycles identical]: `timers` 0.329 → 0.329, `gpu/triangle` 1.516 → 1.516,
`gpu/bandwidth` 4.427 → 4.418, `gpu/benchmark` 2.325 → 2.322 over 600 frames; Crash Bandicoot 2.323 → 2.322, Mega Man
Legends 1.810 → 1.812, Tekken 3 2.416 → 2.411 over their first 3,000. The fold's first form, inside the triangle's
rasteriser, had read `gpu/benchmark` 2.2% to 2.7% slower (D-47); the form kept adds one comparison a triangle and a
call taken only by a primitive that needs it.

### 14.4 The checks kept, and the tests run (2026-10-12)

- ps1-tests' pictures and records, and every program test: `cargo test --release` in the crate, 157 unit tests and 77
  program tests, with `EMUSEN_DIAMOND_PS1TESTS` set and without, all passing; WiseMan's `Diamond` tests under the test
  lock, 46 of 46.
- amidog's OFFSET, POSITION, SIZE and AREA groups: the same TTY, line for line, on the base and the final build.
- The eight games' play as §13 records it: the six other games' `d5` runs byte for byte the same pictures before and
  after the change (§14.1), their titles among them; Crash Bandicoot reaching play by its script, with no olive in any
  picture; Mega Man Legends through its title, NEW GAME, the first ruin, Roll's lines and the first Reaverbots, on the
  open kernel and on the player's BIOS.

### 14.5 Read for this section, and exposures

psx-spx `6d7d1bc`: "Vertex", "GP0(E3h)" to "GP0(E5h)" for D-47; `cdromdrive.md`'s "Data/ADPCM Sector Filtering/
Delivery", "Pause", "GetlocL", "HSTS" and "INT4"'s rows, and `cdromformat.md`'s "Submode" and "Audio/Video
Interleave", for the stream. EmuSen's own: `PsxCheatFormats.cs`, Diamond's GPU and CD-ROM controller, the run tool and
the referees' driver. The games' own code and RAM on the open kernel only: Crash Bandicoot's GP0 words and GTE results
through a debugging build outside the repository, and Mega Man Legends' CD routines disassembled from RAM by the run
tool's `--disasm` and its accesses to the controller by `--game-log`. On the player's BIOS Diamond logged only its own
controller's commands and its sectors' delivery, from frame 8,200 after the anchor, long after the BIOS had handed over;
nothing of the BIOS's code or RAM was read, and no state was saved while it ran. The referees ran as black boxes: their
pictures, and their RAM at `offset-fold.s`'s done word. No emulator's source, no firmware or listing of one, and none of
the documents `DOCS.txt` excludes was opened.

### 14.6 Mega Man Legends: the radio line that stays (D-48)

*Written 2026-10-12, on WiseMan `6ad01c5d`.* The tester played on the handheld again and reported: "It will likely
load to the map screen, so hit select/options on the gamepad to get out of the map. Once out of the map, walk forward
with up on the d-pad until you encounter the dialogue. I believe it's supposed to allude to an enemy on the ceiling,
but it just stays on the dialogue." The game ran on the player's BIOS with the cheat of §14.2 on. What came from the
handheld: the save state made just before the scene by Mistress's state slot, at frame 15,795; the state Mistress
kept when the tester left the game, at frame 17,123; the memory card; Mistress's log, which holds pacing lines only.

- **Not §14.2's prompt** [M]: that box is the game's own question after the instructions, "Read instructions
  again?", which Cross on Yes repeats; this one is Roll's radio line before any instructions, which from the second
  state stays on "MegaMan! I'm picking up a Reaverbot!" and never reaches its second line.
- **The states** [M]: both are Diamond's own `DMND` version 10, unwrapped, 5,398,770 bytes, the size the kit's
  configuration gives (a DualShock in each port, a card in the first, whose contents the state carries). The run tool
  gained `--load-state FILE`, which loads one into the machine its other options make, here the player's BIOS as the
  machine's firmware, `--two-pads` and `--card`, and counts `--press` and `--stick` from the state's frame. The disc's
  fingerprint in the state matches the cache's copy of the tester's disc.
- **From the save state** [M, pictures every 30 frames]: the map at the load; Select at +30 leaves it; Up held from
  +90 for 900 frames walks Mega Man down a corridor of hexagon-lit walls, and Roll's box opens at about +200. Whether
  the line goes on depends on the moment the walk begins: with Up from +90 it runs on ("He's above you! Watch out!"),
  then "Use the R2 button to track enemies above and below you that are out of your field of sight" and the prompt;
  with Up from +93 the box stays on the first line to the end of the run. *By phase*, Up begun at +90 to +177 in steps
  of 3: the box stays in 29 of 30 runs.
- **From the second state** [M]: Up from +30 for 200 frames walks into the scene again; the box opens at 17,250 and
  stays on its first line to the end of every run, the longest's last picture at frame 18,600. Cross every 30 frames from +400 to
  +1,400, R2 at +300 and Down at +600 change nothing in the box; Mega Man's idle animation runs and the game keeps
  issuing commands to the drive, so the game is running and waiting.
- **What the game waits for** [M, the controller's trace from the state's frame, and the game's code and RAM above
  10000h]: each radio line is an XA channel of one file, started by Setfilter (1, n), Setloc 10:57:46 (the file's
  start) and ReadS. Each time the game's routine at 8001C9B8h–8001CA64h runs (while the stream's state byte at
  800988D0h is 2) it asks Getstat through its command routine at 8001D2BCh and, once the Read bit is set, GetlocP; it
  turns the absolute time into a sector number less 150 (800989D8h) and, once that is not below the line's end
  (800989B0h, compared at 8001CA28h), pauses the line. The first line, channel 24, plays to near the file's end and is
  paused there, the head then holding at sector 49,721. The second, channel 25, ends at 49,523 (sector 49,673 of the
  disc). Its ReadS seeks back to the file's start; Getstat reads 22h as soon as the seek ends; GetlocP, asked then,
  answers 11:02:71, the place the first line left (800989D8h holds 49,571); 49,571 is not below 49,523, so the game
  pauses the line before its first sector. It then takes Init, Demute, Pause, Setmode C8h, Setfilter (1, 25), Setloc
  10:57:46 and ReadS again; that GetlocP answers 00:01:68, Init's hold, which the subtraction turns into a number past
  every end, and so on: five ReadS and three Inits in 1,500 frames, each line paused before a sector of it plays. The
  box closes when the line's voice ends, and this line's voice never begins.
- **The cause** (`Diamond_Disputes.md` D-48): a seek that went on into a read set the Read bit at its end, but kept the
  subchannel Q of the last sector the head had passed until the first sector it read, a sector period later; for that
  period GetlocP answered with the place the head had left. psx-spx says a seek decodes subchannel position, and the
  console's `cdrom/getloc` log finds the Q beside the target after every seek. The game asks at once, so its answer
  fell in that period at most phases; where the line before ended earlier on the disc than this one's end, or the
  first GetlocP came after the first sector, the line played, which is why §14.2's five lines and the save state's
  first phase did.
- **The proof**: `cdrom::tests::getlocp_after_a_reads_seek_reports_the_target_not_the_place_left`, on a synthetic
  disc, fails before the change (GetlocP reads 651 for a seek to 250, and track 1 for a play of track 2) and passes
  after.
- **The fix**: the seek leaves the Q of the sector before its target as it turns into a read or a play (`drive.rs`'s
  `found`, called by `seek_done`); a held seek is as before. It is a separate function of the drive beside the
  seek's end, so it merges beside the drive's other changes in progress.
- **After the change** [M, the second state, pictures every 20 or 30 frames]: the line runs on, "He's above you! Watch
  out!" by 17,460, the instructions from 17,580, the prompt; Cross, then Right and Cross on No, close it at about
  18,180, and Mega Man walks on into the room beyond, the camera tilting with R2. From the save state none of the 30
  phases stops. The enemy on the ceiling was not seen in the pictures taken (to frame 19,520, R2 held for 400 frames
  in the room); the scene went on and the player has control again.
- **The cheat** [M]: none: from the second state, with and without `--poke B521E:C4:2`, each build's pictures are the
  same byte for byte, the box staying before the change and going on after.
- **A referee, and the open kernel**: not reached. The states hold the player's BIOS's kernel in RAM, so they run on
  that firmware only. From boot, `psxplay/mega-man-legends.txt` on the open kernel to frame 14,000 walks the
  green-walled corridors and fights the first Reaverbots; the tester's scene lies in the hexagon-walled part of the
  ruin, on a path the script does not take, and the presses to it were not worked out. `psxreferee.py` runs a program
  without a disc or a disc with its pictures alone, so a program at the controller's registers had no referee either.
  The cause rests on the controller's trace, the game's code, psx-spx and the console's log.

### 14.7 The checks kept, and the tests run (2026-10-12)

- ps1-tests' pictures and records, and every program test: `cargo test --release` in the crate after the merge of
  WiseMan `929ec2e7` (the drive's place named, minor 1.4), 162 unit tests and 78 program tests, with
  `EMUSEN_DIAMOND_PS1TESTS` set and without, all passing, and with the opt-in disc inputs (`EMUSEN_DIAMOND_FIRMWARE`,
  `EMUSEN_DIAMOND_DISC`) `cdrom/getloc`, `disc-swap` and `timing` on the player's BIOS among them; WiseMan's `Diamond`
  tests under the test lock with the corpus, 46 of 46. The second state replayed on the merged build gives the pictures of the
  change alone, byte for byte.
- The eight games' play as §13 and §14 record it [M, `d5`'s presses on the open kernel, both builds]: Silent Hill,
  Ape Escape, Tekken 3, Spyro the Dragon, Rayman, Castlevania and Crash Bandicoot, 280, 344, 367, 291, 287, 280 and 300
  pictures, every one byte for byte the same before and after; Mega Man Legends by `mega-man-legends.txt` to 14,000
  frames and by `mega-man-legends-prompt.txt` to 16,000, 233 and 266 pictures, byte for byte the same: no answer
  that Roll's five lines of §14.2 act on was changed.
- The kit at 1,800 frames, on the change alone and again on the merged build: the corpus form on
  `kit/psx-sample.txt`'s nine discs, 9 of 9 and the core's cases passing; Mega Man Legends in the single-image form,
  every case (seventeen, and eighteen with the merge's C18), C7 and C8 among them.

### 14.8 The cost

[M, `examples/run` built from this step's tree before the change and after it, both with
`RUSTFLAGS="-C llvm-args=-align-all-functions=6"`, run alternately on one processor, the fastest of five, ms a frame,
before → after, the emulated cycles identical]: `gpu/benchmark` 2.375 → 2.359 over 600 frames; Crash Bandicoot
2.344 → 2.364, Mega Man Legends 1.855 → 1.838, Tekken 3 2.482 → 2.455 over their first 3,000. The differences are of
either sign and under one per cent, what placement and the machine's other work move. The change adds one read of a
sector's Q at the end of a seek that goes on into a read or a play, a few a second at most, and nothing a frame.

### 14.9 Read for this section, and exposures

psx-spx `6d7d1bc`: `cdromdrive.md`'s "Seek-Busy Phase", "Stat Seek/Play/Read bits", "GetlocL", "GetlocP" and the
command table's rows for them; ps1-tests' `cdrom/getloc` `psx.log` (the console's log; its `main.cpp` no further than
its include lines, `dma.hpp` among them and not opened). EmuSen's own: Diamond's CD-ROM controller and its tests, the
state's format, the run tool, `psxplay.py` and `psxreferee.py`'s usage text, the kit's sample.

On the player's BIOS, from the tester's two states: the screen; Diamond's own controller, traced from the state's
frame on (the game was running; its commands, responses and the head's place); the game's code at 8001C900h–8001CADCh
by the run tool's `--disasm`, and its RAM at 800988D0h and 800989A0h–800989EFh by `--dump`, all above 10000h. The first
load printed the end of the state's TTY buffer: an address, the game's graphics library's "ResetGraph…" line and a
pad driver's banner, which is probably the kernel's own text; printed text only, and later runs suppressed it. The player's BIOS was given to the machine as its firmware by
path, and to ps1-tests' opt-in disc tests the same way; it was not read, dumped, traced or disassembled, and nothing
of RAM below 10000h or of code running there or in ROM was observed. The routines at 8001D2BCh and 80077F8Ch that the
loop calls were not followed; what they do is taken from their arguments and results. No referee was run. No
emulator's source, no firmware or listing of one, and none of the documents `DOCS.txt` excludes was opened (`DOCS.txt`
itself for its list); Diamond's `kernel/` and `kernel-harness/` were not touched.

---

## 15. Precise geometry, as players meet it (2026-10-13)

*Stage E4 of `Diamond_Plan.md` §7; the design and its gate are `Diamond_Enhancements.md` §21 to §31, and the crack
fill that closed its seams the same day §32 to §36.* The PlayStation's first enhancement, offered with its gate met and
every accurate golden held (`Diamond_Plan.md` §8, GE).

**Where it is.** Graphics Settings, the PlayStation's tab, under the heading **Enhancements**: one switch, **Precise
geometry**, off. Its hint says what it does and what it costs: the picture drawn on the graphics card with each 3D
corner where the game's geometry puts it, between the pixels, and textures in perspective, so polygons stop wobbling and
textures stop bending as they move; the game itself running exactly as on a console. Its cost: a Vulkan graphics card,
some of its time and about half a millisecond of the processor's a frame, save states about twice their size, the
corners a game moves itself (2D menus and overlays) left on whole pixels, and an outline's top and left edges left on
the console's line.

**Turning it on and off.** It applies between frames, at once, while a game runs; the picture changes from the next
frame and nothing else does (the sound, the game's own state and what it computes are those of a console;
`DiamondPreciseGeometryTests`, since 2026-10-14 `DiamondEnhancementsTests`). Off is the console's picture, drawn by the software rasteriser as before.

**Without a usable graphics card.** The switch can still be turned on; the game then runs on the console's own picture,
and a note under the row says so: "No usable Vulkan graphics card was found (the reason), so the console's own picture
is shown." Where the only Vulkan is the processor's own (llvmpipe) the picture is drawn there, the same, slowly, and
the note says that instead. Nothing fails to load because of it, and a game's stored setting is never refused.

**Save states.** *Since 2026-10-14 a state is the same 5.4 MB with it on or off, and the picture settles a moment after a
load (§17).* With it on a state was about 11.6 MB against 5.4 MB, since it held what the enhancement needs to draw
the next frame exactly as the machine that saved it would. A state saved with it on loads with it off, and one saved
with it off loads with it on; the first frame or two after such a load may be drawn with fewer precise corners.

**What to expect, by game** (`Diamond_Enhancements.md` §25.5): Crash Bandicoot, Mega Man Legends, Ape Escape and Tekken
3's fights are drawn 81 to 99.7 per cent precise; Silent Hill's play wholly, its opening not; Spyro the Dragon 96 to
98 per cent since 2026-10-14 (`Diamond_Enhancements.md` §55), its corners followed through the shift it keeps them in,
where at first it was under a fifth, so its grass, walls and hills now stay steady as the camera turns; Rayman and
Castlevania, which draw in 2D, almost nothing, and their pictures are the console's.

**Cracks between polygons, closed** (`Diamond_Enhancements.md` §32 to §36). As first offered the setting showed
hairline cracks where a precise polygon met one the game had built itself: lines of the sky's colour along the ledges
of Ape Escape's first level and its lab's floor, dashes across Spyro the Dragon's grass. The tester reported them from
the handheld on 2026-10-13, and since the same day a precise polygon is also drawn wherever the console would have
drawn it, so nothing shows between it and its neighbour. What a player may notice in exchange: where a 3D shape's
outline has nothing precise beside it, its top and left edges stand on the console's line and move a pixel at a time
as they do there, while its bottom and right edges, its inner edges, its corners and its textures stay steady; and a
few cracks are left in see-through surfaces (a haze over the sky, a fade between scenes), which are not filled so
that none is tinted twice. Whether the fill should be a switch of its own is the tester's to say
(`Diamond_Enhancements.md` §36.2).

**The handheld** is not yet measured with it; the estimate is `Diamond_Enhancements.md` §27.

## 16. Internal resolution, true colour and texture filtering, as players meet them (2026-10-14)

*Stage E5 of `Diamond_Plan.md` §7; the design, the gate and the costs are `Diamond_Enhancements.md` §37 to §51.* Three
more rows under the PlayStation tab's **Enhancements** heading, after *Precise geometry*, each off by default (the
console's picture) and each applied at once, between frames, while a game runs. The game itself runs exactly as on a
console with any of them: its sound, its timing and what it computes are unchanged, and a save state made with them
loads without them and the reverse.

**Internal resolution** (*1x*, *2x*, *3x*, *4x*). The picture drawn at a multiple of the console's, so that 3D edges and
textures are finer on a large screen. Sprites, text and 2D backgrounds stay on the console's pixels, enlarged, each a
square block: nothing in a menu moves or blurs. A screen a game copies and draws back (a backdrop, a reflection, a
transition) keeps the finer picture. What does not get finer: effects a game makes by reading the screen back through
its processor, lines, and the console's dither, which shows as a coarser grain unless *True colour* is on. With a CRT
filter on, the filter draws the enlarged picture's lines rather than the console's (the core does not yet tell the
frontend the console's line count; `Diamond_Enhancements.md` §39.4). Each step costs its square in the graphics card's
work and memory: 2x is four times the drawing, 4x sixteen.

**True colour**. Shading, lighting, fog and see-through effects drawn with eight bits a colour where the console keeps
five, and without the console's dither: gradients and fog smooth instead of grainy. Pictures drawn at the console's own
precision (menus, 2D, uploaded images, movies) look exactly as on the console. What a player may notice in exchange:
fades and glows that work in steps smaller than the console could show now show them, a faint square about Silent
Hill's publisher logo as it fades and the edge of Ape Escape's sun glow; and art made for the dither looks smoother than
it was drawn to.

**Texture filtering** (*Off*, *3D polygons*, *Everything*). Textures smoothed between their texels instead of shown as
blocks. *3D polygons* filters 3D surfaces and leaves sprites, text and menus sharp, including menus built of polygons
drawn one texel to a pixel; *Everything* filters them too, which softens text once the internal resolution is above
1x. Cut-out shapes keep the console's exact edges with no dark fringe, and a texture packed beside another in memory
does not take its neighbour's colours.

**Every enhancement shows its picture a frame late**: about 17 ms at 60 frames a second, the same for all four,
*Precise geometry* included since this stage. It is what lets the game run without ever waiting for the graphics card.

**Save states** grew with them as first offered: about 11.6 MB with *Precise geometry* alone at 1x, 26.5 MB at 2x and
82 MB at 4x with every enhancement on, against 5.4 MB with none. *Since 2026-10-14 they do not grow at all (§17).*

**Without a usable graphics card** each row can still be turned on; the console's picture is shown and a note under each
row set says why, as for *Precise geometry* (§15).

**On the handheld.** Not yet measured with this stage: the tester's own play of Mega Man Legends with *Precise geometry* on
(2026-10-14, the build before this stage) showed the enhancement costing far more there than the desktop had suggested,
a frame of about 12 ms at the median against 2 ms without it, and 18 to 19.5 ms in a boss fight, where the game slowed to
48 to 52 frames a second. This stage hands the picture a frame late so that the game no longer waits for the graphics
card, which should bring that game back to about 2.5 to 3 ms of the processor's time a frame with the card drawing
beside it, the boss fight at or near full speed (`Diamond_Enhancements.md` §50.5, an estimate to be confirmed in play).
**Recommended there**: *Precise geometry*, *True colour* and *Texture filtering* on *3D polygons*, at an *Internal
resolution* of *1x*: true colour and filtering cost the card little, and the handheld's screen is the console's
resolution or near it, where a higher internal resolution shows least. *2x* is worth trying in a game that runs with
room to spare; *3x* and *4x* are for a desktop graphics card, where every bench game draws its 4x picture in under 1.5 ms
of a 16.7 ms frame.

## 17. Save states and rewind with the enhancements (2026-10-14)

*`Diamond_Enhancements.md` §52 and §53; decided by the tester the same day.* A save state with any of the four
enhancements on is now the console's state alone, the same 5.4 MB as with none, at every internal resolution: it no
longer holds the enhanced picture, which the graphics card draws again after a load.

**What a player gains.** Save states a fifteenth of their size at 4x with everything on, and the same file whichever
enhancements are set, so a state saved with them loads without them and the reverse, as before. Rewind reaches far
further back in the same memory: with every enhancement on at 2x, about 82 seconds in Spyro the Dragon where it reached
10, 81 in Ape Escape where it reached 22, and 124 in Crash Bandicoot where it reached 22, at Mistress's two snapshots a
second. Turning an enhancement on or off no longer empties the rewind history, since the state's size no longer
changes. Saving and loading a state no longer waits for the graphics card to copy its pictures back.

**What it costs.** For a moment after a state is loaded, or while a rewind is held, the picture is the console's,
enlarged, without the enhancements' detail: the game's next frames draw it again. In most games that takes a tenth of a
second (six frames at the most on the bench discs); in a game that builds its picture from the one before, such as
Silent Hill's fog, the last traces take about half a second to fade, or two to three seconds with *True colour* on,
since its finer shades fade more slowly. A game that draws something once and keeps it, as a backdrop drawn at the
start of a level, shows that at the console's detail until it draws it again. The game itself is never affected: what
it computes, its sound and its timing are exactly those of the state.

**States made before.** A state saved with the enhancements on by an earlier build loads, its saved pictures set aside,
and the game continues exactly; its picture settles as from any state.

