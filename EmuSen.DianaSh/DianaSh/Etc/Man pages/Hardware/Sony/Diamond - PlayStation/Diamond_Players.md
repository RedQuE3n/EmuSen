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
  facts on four triples, passing against this branch's 1.1 baseline and reporting against WiseMan's 1.2 baseline only
  `json info.systems[].development`'s tag, the line both minors added; the kit, 7 + 15; Diamond, 148 + 57; WiseMan's
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

## 11. Reading log, and exposures

Read for this stage: psx-spx `stable` `6d7d1bc`, `cheatdevices.md` lines 1–48 (the devices and the first-digit
table), 282–347 (the Datel format) and 833–905 (the Xploder's format and decryption); the libretro database's
PlayStation files for their codes' forms and counts; the project's own pages and code. ScreenScraper's system id was
written from what is known of its list and is to be confirmed. No emulator's source, no BIOS, no BIOS listing, no
part of psx-spx's "BIOS Patches" or of `Diamond_References.md` §4 and §6, and nothing DOCS.txt excludes was opened;
DOCS.txt itself was read only for its list of exclusions. The player's BIOS folder was not read, and Diamond's
`kernel/` and `kernel-harness/` were not changed.
