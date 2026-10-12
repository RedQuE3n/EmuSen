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
per game being the decided default.

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
- A card shared between games (`EmuSen_CoreAPI_Extensions.md` §9); `ChtFile`'s RetroArch `cheat_type` 4 to 7, whose
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
   before, none for the zip and `scph5501.bin` loose. *Fix:* the probe reads the image play opens, `DiscsOf(path)[0]`,
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

- **The abort of §12.3** is the host's: input reaches a v1 machine from the interface thread while the emulation thread
  is inside it. The form that keeps §6.16 without a surface change is for `CoreEngine` to keep the pad's buttons and
  axes and pass them to the machine on the emulation thread before each frame, which also changes when a press is seen
  (at the next frame, as a poll between frames already is). It touches the running game rather than its launch, and
  wants its own proof (a run with input from a second thread, in a process of its own, since the abort ends the test
  host), so it is put to the tester rather than built here.
- **A sheet naming a file by another name** (`Game (USA).bin` beside a file `Game.bin`) opens nowhere, the load's words
  naming the missing file. Choosing another file for it would be a guess; none of the library's sheets does it.
- **A loose audio track goes to the Genesis even in a folder named `psx`**: the Genesis's recogniser claims anything
  that is not a disc's image (§1), and a claim decides before the folder. Only a track left without its sheet meets
  this; the library holds none. Whether a folder should outweigh a negative rule is the routing design's question.
- **The image is read twice before the first frame**, once for `firmware_for` and once for `create`, and Diamond copies
  it once more; on the handheld the three launches the journal times took two to three seconds from the per-game log's
  start to the first frame line, which is written a second after frames begin, so it is recorded and not changed.

### 12.6 Tests run (2026-10-12)

WiseMan, by area: `PlayStationLibrary`, `Diamond` (with `DiamondPlayers`), `Galaxia`, `RomLibrary`, `SystemPack`,
`CoreCatalog`, `CoreFactory`, `Firmware` and `EmulatorSession`, 396 cases, 388 passing and 8 skipped (the opt-in ones
without their folders); the opt-in run of §12.1 on the seven copies, with development cores off and on.
