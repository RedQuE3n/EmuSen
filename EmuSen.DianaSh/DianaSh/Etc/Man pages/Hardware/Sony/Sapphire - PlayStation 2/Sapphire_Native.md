# Sapphire_Native — the build record

*Started 2026-10-24, at stage S1; stages S2 (§12), S3 (§13), S4 (§14), S5 (§15) and S6 (§16) the same day; S7 (§17), S8 (§18), S9 (§19), S11 (§20), S12 (§21), S21 (§22), S13 (§23), S16 (§24), S23 (§25), S22 (§26), S25's part A (§30), S24 (§35), S19 (§37) and S10 (§36) on 2026-10-25.* *Decided 2026-10-25 by the tester: S11 is built before S10*, whose gate runs on the player's BIOS once S24 boots it (`Sapphire_Plan.md` §7); the stages are recorded here in the order they are built. `Sapphire_Plan.md` says what Sapphire is to be and how it is graded; this page records
what was built, stage by stage, what was measured, and where the plan turned out to be wrong. The plan's marks are
kept: **[M]** measured on 2026-10-24 unless another date is given, on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10),
with the tool named beside it; **[R]** read from a source, named; **[A]** argued; **[C]** a choice, a rule set here or a
recommendation awaiting the tester; **P1–P19** the plan's predictions.

---

## 1. Stage S1, in one paragraph (2026-10-24)

The plan's §7 row S1, on the tester's go of 2026-10-24. The crate `sapphire` (`EmuSen/Cores/Sony/Sapphire - PlayStation
2/`) is a stub on the core ABI v1: it recognises an EE program by its ELF header and a disc by `SYSTEM.CNF`'s `BOOT2`
line, refuses everything else with words, declares the three BIOS entries, keeps its own state format `SPH2`, and shows
a black picture with silence; nothing is emulated. `RustCores.props` marks it `InDevelopment` and its system entry says
`development: true`, so no player is offered it. The systems table gains `ps2.json` and the C# its literal pack, with
a recogniser for `BOOT2`; the PlayStation's recogniser stops claiming a disc that names a `BOOT2` program, so a PS1
disc routes to Diamond and a PS2 CD or DVD to the PlayStation 2. WiseMan gains `SyntheticPs2Elf`, `SyntheticPs2Disc`
and `SapphireTests`. It wrote no emulation rule, and read no emulator source and no firmware.

## 2. The crate

### 2.1 On the core ABI v1

- **`Core` and `core_exports!(Machine;)`**, no optional group: `CAPABILITIES` is 0, as Diamond's and Endou's were at
  their first stage. The only dependency is `emusen-native`; no new cross-component dependency was added (§4).
- **Core info**: id `sapphire`, display name `Sapphire`, one system `ps2`, "Sony PlayStation 2", its extensions (§2.4),
  NTSC and PAL, the two controllers of the plan's §4.2, the three BIOS entries (§2.5) and `development: true`.
- **Machine info**: the system `ps2`, the region, the frame rate (§2.3), a picture of 640 by 448 NTSC or 512 PAL at
  4:3, 48,000 Hz, each port's controller from the settings, the spaces (§2.6), **the processors `R5900` (0), `IOP`
  (1), `VU0` (2, a 12-bit PC over its 4 KiB of micro memory) and `VU1` (3, 14 bits over 16 KiB)**, none with
  registers listed until the debugger's stage, no battery, the state `SPH2` version 1,
  `skip_rendering_state_neutral` false (the plan's §5.4), and the BIOS of the machine's market as `file` or `absent`.
- **A frame** advances the count and queues its silence, each frame's sample count exact to the rational rate; the
  pads' bits are kept per port and are in no state.

### 2.2 The image, read from its bytes

`media.rs`, recognition only:

- **An ELF** [R the Tool Interface Standard's ELF 1.2, the header and program header layouts]: the magic `7Fh "ELF"`,
  class 1 (32-bit), byte order 1 (little-endian), `e_type` 2 (an executable) and `e_machine` 8 (MIPS), program headers
  of 32 bytes, at least one `PT_LOAD` segment, every segment's file bytes inside the image and no more than its memory
  size. The entry and segments are recorded; nothing is loaded. Refused with what it is instead: a 64-bit or
  big-endian ELF, another machine, an IOP module (`e_type` FF80h, an `.irx`, the plan's §4.1 rule 3), another type, no
  segment to load; and as truncated, with the bytes needed, where a header or segment runs past the end.
- **A disc**: 2,048-byte sectors, or raw 2,352-byte ones told by their sync, their 2,048 data bytes 16 in for mode 1 and 24
  for mode 2 (ECMA-130 §14); the primary volume descriptor at sector 16 (ECMA-119 §8.4); `SYSTEM.CNF` in the root
  directory (§9.1). **A line whose key is `BOOT2`** makes it a PlayStation 2 disc, its serial the program's name in the
  form the PlayStation's pack already reads (`cdrom0:\SLUS_203.12;1` is SLUS-20312). A disc with a `BOOT` line and no
  `BOOT2`, or with no `SYSTEM.CNF` but `PSX.EXE` or Sony's licence text, is refused as **a PlayStation disc, naming
  Diamond**; any other ISO 9660 volume as naming no `BOOT2` program. Sony's licence text alone decides nothing: a
  synthetic PS2 disc carrying it is still the PlayStation 2's.
- **Why the core reads the volume itself, not through `emusen-disc`** [C]: the crate is a compact disc as sectors and,
  by its own rule, holds no file system (`EmuSen_Disc.md` §1); its DVD medium is S23's (the plan's §9.4), and opening a
  DVD image through it today would synthesise CD sectors for a DVD. S1 reads one descriptor and one root file at
  create, about forty lines; `emusen-disc` takes over sector reading at S23 with the DVD layout.
- **The `BOOT2` key** stays [A], as the plan's §4.1 left it: ps2tek's "BIOS Boot Process" was not read at S1 (the
  document is pinned and screened at S2, the plan's §2.2), so the confirmation from it moves to S2. One witness was
  measured: the tester's test copy of Jak and Daxter routes by `BOOT2` (§3.3).

### 2.3 The frame rates, the picture and the market (provisional)

- **The market** is the setting's, else the Americas [C]: no image names its market to the stub, since the serial's
  prefix and `VMODE` are S29's reading (the plan's §4.1). PAL exactly when the market is Europe.
- **The frame rate** [A, provisional until D-0 and S20]: the standards' field rates, 60,000/1,001 Hz NTSC and 50 Hz PAL.
  The PCRTC's exact rate follows from its clocks, which are D-0's subject (`Sapphire_Disputes.md`).
- **The sound**: silence at 48,000 Hz, the SPU2's rate (the plan's §5.5), as a count drained as zeros, bounded by
  `set_audio_limit` with the oldest dropped, no allocation per frame.
- **The picture**: 640 wide, 448 lines NTSC and 512 PAL, black [C, until the PCRTC decides it at S20].

### 2.4 Extensions, controllers and settings

- **Extensions** [C, the plan's Q16]: core info lists `.elf` alone. The system's table entry lists every form the
  console's games come as (§3.1), and the core's list is its checked subset. The core still creates a machine from an
  ISO image named to it directly, as the kit does, telling a disc by its content, never its name.
- **Controllers**: `ps2.dualshock2` (sixteen buttons, two sticks) and `ps2.digital` (fourteen buttons), the bits in
  the PlayStation pad report's order and Diamond's canonical controls, so bindings carry between the two [A: the
  DualShock 2's digital report is read at S25]. The sticks are listed and `AXES` is not claimed; pressure has no place
  in v1 (the plan's §4.2, §10).
- **Settings**, all `scope: create`, `restart`, `effect: none`: `region` (`auto`, `ps2.jp`, `ps2.us`, `ps2.eu`),
  `pad1` and `pad2` (`ps2.dualshock2`, `ps2.digital`, and `none` for port 2).

### 2.5 Firmware

Each entry `required: false`, 4,194,304 bytes, `replacement: none` with the cost "No PlayStation 2 program runs yet,
with this file or without it", as Diamond's stage 1 declared its BIOS; no MD5s until S2 (the plan's Q9). *Revised
2026-10-24 at S2 (§12.3): Japan's entry is `ps2-0230j-20080220.bin`, and each entry lists its market's MD5s.*

| File | Name | Market |
|---|---|---|
| 16 | `ps2-0210j-20040917.bin` | Japan |
| 17 | `ps2-0230a-20080220.bin` | the Americas |
| 18 | `ps2-0230e-20080220.bin` | Europe |

- **The numbers** start at 16 [C], leaving 0–15 to the battery media the plan's §4.4 names (two cards, a multitap's,
  the console's EEPROM), so that adding them renumbers nothing.
- **Against the tester's images** [M, a listing of names and sizes only, `ls -l` of `~/Documents/BIOS/PS2/`, nothing
  opened]: `ps2-0230a-20080220.bin` and `ps2-0230e-20080220.bin` are there, 4,194,304 bytes each. For Japan the newest
  retail name in that folder is `ps2-0210j-20040917.bin`, which Q9 named for the gates; the folder `Misc BIOS/` holds a
  newer `ps2-0250j-20100415.bin` and `ps2-0180j-20031028.bin`, which the tester filed apart from the retail set. **The
  Japanese entry is provisional** [C]: S2 replaces it with the newest Japanese retail version a public database lists,
  if that is newer, and the MD5 lists make the name secondary in any case.
- A player's file, when given, is held in the read-only `BIOS` space and not run; `firmware_for` names the Americas'
  BIOS for any image it recognises and nothing for another.

### 2.6 The spaces and the state, "SPH2"

| Id | Space | Size | |
|---|---|---|---|
| 1 | `RAM` | 32 MiB | the cheats' space |
| 2 | `SCRATCH` | 16 KiB | |
| 3 | `GSMEM` | 4 MiB | |
| 4, 5 | `VU0MICRO`, `VU0DATA` | 4 KiB each | |
| 6, 7 | `VU1MICRO`, `VU1DATA` | 16 KiB each | |
| 8 | `IOPRAM` | 2 MiB | |
| 9 | `SPU2RAM` | 2 MiB | |
| 10 | `BIOS` | 4 MiB | read-only; the player's image when given |

All zero-filled; the cards come at S25. **Version 1** of `SPH2`: the magic, the version, the frame count, the market,
then every writable memory in id order, **42,000,401 bytes** [M, the kit and `the_version_1_layout_is_pinned`], the
plan's "about 41 MiB" before the caches, FIFOs and registers join it. A state of another magic or market is refused as
foreign, of another version as a version, a truncated one as truncated, and a refused load changes nothing. No player
has a Sapphire state, so version 1 is redefined freely until the core is offered.

## 3. The system pack

### 3.1 The entry, and who owns which fact

`Platform/systems/data/ps2.json`, appended after `sg1000`, and the C# literal `EmuSen.Crystal/Systems/Ps2/Ps2System.cs`,
held equal by `SystemTableParityTests`:

| Field | Value | Source |
|---|---|---|
| `console`, `manufacturer`, `release_year` | PlayStation 2, Sony, 2000 | [C; the year the plan's references give] |
| `extensions` | `.elf .iso .chd .cue .bin .m3u` | the plan's §4.1 |
| `cheat_systems`, `openvgdb_systems` | **empty** | [M the plan's §4.1: no PlayStation 2 folder in libretro's cheat database, no PlayStation 2 system in OpenVGDB] |
| `esde_system`, `esde_full_name` | `ps2`, Sony PlayStation 2 | [R ES-DE's `es_systems.xml`, by the plan's citation] |
| `screenscraper_system` | 58 | [R ScreenScraper's `systemeinfos.php?plateforme=58`, titled "Playstation 2", read 2026-10-24]; the plan had it [A] |
| `cover_aspect` | 1.41 | **provisional** [A]: a DVD keep case's height over width; S29 sets it from the boxes |
| `cheat_space`, `media` | none, none | no codec until the player gate, no card until S25 |
| `byte_order` | little | the R5900's, as the EE runs it |

**A schema change** [C]: `cheat_systems` and `openvgdb_systems` were refused when empty, which no earlier system
tested. Writing a folder name that does not exist would state a false fact, so `emusen-systems` now accepts either
list empty, still refusing a name twice; `extensions` must still name one. The C# side read empty lists already.

### 3.2 Routing by content: `BOOT2`, and `BOOT` beside it

The plan's §4.1 rules, as built in both halves (`ps2/disc.rs` and `Ps2System.Recognise`; `psx/disc.rs` and
`PsxSystem.Recognise`), held alike by `SystemCodeParityTests`, the bytes each reads included:

- **The PlayStation 2's recogniser** claims an image whose root's `SYSTEM.CNF` has a line whose key, trimmed, is
  `BOOT2` (case aside); a longer key (`BOOT20`) or a line with no `=` is not one. Its serial is the program's; its
  market is read at S29.
- **The PlayStation's recogniser** keeps its licence-text rule and gains one exclusion: a disc whose `SYSTEM.CNF` names
  a `BOOT2` program is not its. **Not broadened** [C]: the plan's rule 1 would also give Diamond a disc with a `BOOT`
  line and no licence text; that changes which discs Diamond is offered, which is Diamond's decision, so it is left.
- **Reading far enough** [C]. A DVD's bridge volume puts its root directory and first files past the 160 KiB the
  PlayStation's recogniser read, so a root file is now read *through* the head: `DiscSectors::reach` (C#: `Reach`)
  says whether a file is found, absent, or past the head with the bytes of the image it needs, and
  `root_file_through` (C#: `RootFileThrough`) asks the host for exactly those, a directory block or the file's whole
  extent at a time, **up to 32 MiB** (`ROOT_FILE_LIMIT`); a file past it, or past the image's end, is none. The
  PlayStation's discs read as before: their root is inside the first 160 KiB, so their reads are the same lengths
  (`SystemCodeParityTests` held them on every existing image). The PlayStation 2's recogniser asks for 40 KiB first,
  enough for sector 16 in either sector size.
- **Without the exclusion**, a PlayStation 2 disc carrying Sony's licence text would be claimed by both recognisers and
  route nowhere; F1 of §7 shows the test that catches it.

### 3.3 Measured

- **By test** [M]: a synthetic PS1 disc (`.iso` and raw track) routes to `psx`, and a synthetic PS2 CD (`.iso` and raw
  mode 2 track, with and without the licence text) and DVD route to `ps2`, among the systems that take `.iso` or
  `.bin` (`mcd`, `psx`, `ps2`; `md`, `32x`, `psx`, `ps2`), whatever the folder (none, `psx`, `ps2`, `segacd`, `roms`), in
  both halves (`SapphireTests`, and the systems crate's own test); the DVD's directory and file are asked for where
  the volume places them (40 KiB, then sector 301's end, then 302's).
- **By hand, once** [M]: the tester's test copy of Jak and Daxter (an `.iso` of 1,460,305,920 bytes in the probe's cache,
  read in place) is claimed by the PlayStation 2's recogniser alone, routes to `ps2` with the folder named `ps2` and
  with none, with a serial found, and the recogniser read at most 1,314,816 bytes of it, well inside the limit. Only
  this result is recorded; no byte of the disc was printed, kept or copied, and the throwaway program was deleted.
- **What a host routes today**: a host's candidates for an extension are the systems an offered engine opens it with
  (`CoreCatalog.Route`). Sapphire lists only `.elf`, so in a frontend an `.iso` is still `mcd`'s or `psx`'s to claim,
  and a PS2 disc, which neither now claims, is refused with the words naming both unless its folder names one. Before
  this stage a PS2 disc carrying the licence text would have gone to Diamond. The PlayStation 2 joins `.iso`'s
  candidates when the core lists the disc forms at S23.

## 4. The registration

- **`RustCores.props`**: the row `sapphire` (`Sony\Sapphire - PlayStation 2\`, "the PlayStation 2", no fallback,
  `CoreAbi 1`, `InDevelopment` true). The build writes the sidecar marked; the library reaches the build output for the
  tests and the kit, and never a publish or its manifest (`EmuSen_Backend.md` §27.11).
- **The system entry's `development: true`**: no shelf, no Graphics Settings tab, no firmware row for a player, as
  `EmuSen_CoreAPI.md` §27.4 has it; with development cores asked for, a developer opens an `.elf` on Sapphire.
- **`EmuSen.csproj`** leaves the crate's `target/` out of its items; **`rust-cores.yml`** watches the folder, checks the
  crate on every target, builds, export-checks and sidecars it with `--development`, and runs `FullyQualifiedName~Sapphire`
  in the WiseMan job.
- **`.gitignore`** gains `*.elf` and `*.irx`, so a program or module dropped in the tree is never committed (the
  repository holds none).
- **Dependencies**: `sapphire` → `emusen-native` only. `emusen-systems` gains no dependency. Nothing crosses a surface
  but the core ABI and the systems table.

## 5. WiseMan's builders

- **`Fixtures/SyntheticPs2Elf.cs`**: an ELF32 header for a little-endian MIPS executable, one segment at 00100000h and,
  by default, a branch to itself with its delay slot; the type, machine and class can be spoiled for the refusals.
- **`Fixtures/SyntheticPs2Disc.cs`**: an ECMA-119 volume whose root holds `SYSTEM.CNF` with a `BOOT2` line naming a
  program of the test's own (`EMUS_000.02`, a serial no licensed disc uses) and that program. **`Cd()`** puts the root at
  sector 20 and also writes the volume as a raw mode 2 form 1 track through `CdSector`, with ECMA-130's EDC and parity;
  the licence text in sector 4 only on request. **`Dvd()`** writes 2,048-byte sectors alone with the root at sector 300,
  past the first 600 KiB [A: where a bridge volume's ISO 9660 directory lies; the synthetic "DVD" is defined by that
  layout and its sector size, since an image records no physical format]. Never real game data. *Since §40.14 the
  path tables at 257 and 259 and the root at 261, as the medium's discs place them and the player's BIOS reads them.*
- The crate's own tests build the same shapes in `media::sample`.

## 6. The CRT and the audits, under the development mark

The PlayStation 2 was a television console, so it is owed the CRT with a profile of its own and the pad and fit audits
when it reaches players (S29). **What the rules require now**:

- **The CRT rule's test reads every system of the table**, and a new system with no screen row fails it. A row now would
  either borrow the PlayStation's profile, which the test rightly refuses for a console that is no attachment, or carry
  numbers no document is behind: the PCRTC's clocks are D-0's subject. **Built** [C]: `ConsoleScreens.OwedRows`, a
  dated note per console whose row is owed; the test accepts an owed row only while the console has no shelf, so the
  step that offers the PlayStation 2 fails until the real row replaces it. Recorded in `EmuSen_CRT.md` §31.8.
- **The pad and fit audits** (`PadSettingsWindowTests`, `ThemedLibraryHostTests`, `WindowFitAuditTests`) walk the
  shelves, tabs and drawings a player sees; a console in development has none, so they require nothing yet and were
  run unchanged. The DualShock 2's drawing is S29's.

## 7. The gate (`Sapphire_Plan.md` §7, S1), measured 2026-10-24

| Item | Result [M] |
|---|---|
| The crate's tests (`cargo test --release`) | **13 of 13** (media 4, machine 2, state 2, v1 5) |
| `export_check.py` against the v1 baseline | passes: ABI 1.15, capabilities 0, 32 `emusen_core_` symbols |
| **The kit**, `emusen-core-conform --frames 120`, single-image form | **every case it runs passes** on the synthetic ELF: C1–C15 and C17–C24, 23 cases (C16 is `MEDIA`'s, not claimed); "EmuSen v1 compliant on this image"; the state 42,000,401 bytes. `SapphireTests` runs it on the ELF and on the CD's and the DVD's `.iso`, each passing C1–C15 with nothing failing |
| `emusen-systems` | **39 of 39**, four new; `emusen-library` and `emusen-platform` pass unchanged |
| WiseMan, `blast.sh`'s filter plus `DiscProbeTests`, `SegaCdRouting`, `CoreDiscovery`, `FirmwareOverview`, `SystemPack`, `CoreAdapter` and `DiamondTests` | **983 of 988**; the five: `SystemTableParityTests` counting eleven rows (corrected to twelve, then passing), and four `WindowFitAuditTests` recordings of the theme detail window, below |
| The four recordings | **Not this stage's**: the window shows the installed theme's folder under the temporary directory, and with `TMPDIR` set to the long `~/.cache/emusen/tmp` the line wraps once more than when the recordings were made. Run again with the default temporary directory: **112 of 112** |
| No commercial image in any test | the fixtures build every image; the Jak and Daxter check was by hand and is in no test |

## 8. Seeded faults (2026-10-24)

Each made in place, its tests run, the file restored from a copy:

| # | Fault | Caught by |
|---|---|---|
| F1 | The PlayStation's recogniser without its `BOOT2` exclusion | `a_playstation_disc_routes_to_the_playstation_and_a_playstation_2_disc_to_the_playstation_2` in `emusen-systems` (the licensed PS2 disc claimed twice) |
| F2 | A state load that leaves the frame count as it was | `a_state_round_trips_and_a_refused_one_changes_nothing`; and on the built library the kit's **C7, C8 and C12** |
| F3 | The C# PlayStation 2 recogniser reading 32 KiB first instead of 40 | `SystemCodeParityTests.Every_recogniser_answers_alike_and_reads_the_same_bytes`, and `SapphireTests`' routing test (32 KiB does not reach a raw track's sector 16, so the C# half missed the raw CD) |

## 9. Decisions, questions and what the plan got wrong

- **The market of an image** is the Americas until S29 reads the disc's [C]; one line to change.
- **The Japanese BIOS name is provisional** until S2 (§2.5). *Settled 2026-10-24 at S2: `ps2-0230j-20080220.bin` (§12.3).*
- **The BIOS file numbers start at 16** (§2.5).
- **The PlayStation's recogniser is not broadened** to `BOOT`-only discs without the licence text (§3.2).
- **The plan's §4.1 rule 2** (a DVD-sized image with `BOOT2` is the PlayStation 2's) needs no code: rule 1 decides every
  such image, and the size of an image is not a medium's kind.
- **The plan's §4.6 and its S29 row** assume the CRT row waits; the CRT rule's test, written after the plan's
  model (Diamond's stage 1), did not let it wait silently. The owed row of §6 is the form found for it.
- **The plan's §10** "128-bit registers in `DEBUG_REGISTERS`": not read at S1, since no register is listed; it moves to
  the debugger's stage, S31.

## 10. What was read for this stage

No emulator source, no SDK source and no firmware. Read: `Sapphire_Plan.md` §1, §4, §5.9, §5.12, §7, §9, §11;
`Diamond_Native.md` §1–§9 and `Endou_Native.md` §1–§8; `EmuSen_CRT.md` §31; `EmuSen_CoreAPI.md` on firmware `parts`;
the code they cite; the ELF 1.2 header layout; ECMA-119's descriptor and directory records and ECMA-130's sector
layout, as the project's builders already implement them; ScreenScraper's system page 58's title. Listed, never
opened: the names and sizes of the tester's PlayStation 2 BIOS images. ps2tek's BIOS chapter was **not** read, so this
stage's writer is not excluded from the open BIOS by it.

**CHD and chdman, credited (2026-10-25).** CHD (Compressed Hunks of Data) is MAME's image format, and chdman is MAME's
tool for making, checking and extracting it. chdman's source file is under the BSD-3-Clause licence, copyright Aaron
Giles; MAME as a whole is distributed under the GNU General Public License, version 2 or later ("The MAME License",
MAME Documentation 0.289, <https://docs.mamedev.org/license.html>). chdman's commands, codecs and defaults are
documented in "chdman – CHD (Compressed Hunks of Data) File Manager", MAME Documentation 0.289,
<https://docs.mamedev.org/tools/chdman.html>, read 2026-10-25; every chdman run the project records was of release
0.289. EmuSen reads CHD through the `chd` crate (BSD-3-Clause, Ronny Chan; `THIRD_PARTY_NOTICES.md`), whose source is
not read, and MAME's source is never read (`EmuSen_Disc.md` §1, §8).

## 11. What this stage does not cover

- No emulation: every picture is black and every sample silent.
- The references, documents and test programs (S2), the referees (S3) and the runners (S4).
- The disc forms in core info, and reading a disc through `emusen-disc` with the DVD medium (S23).
- The serial's market, `VMODE`, the shelf, the CRT row and the drawing (S29).

---

## 12. Stage S2, the references (2026-10-24)

### 12.1 In one paragraph

The plan's §7 row S2, on the tester's go of 2026-10-24. The documents of the plan's §2.1 are fetched and pinned in the
probe's cache with `DOCS.txt` and `SHA256SUMS`, and ps2autotests at `97469ff` with `PROVENANCE.txt`; ps2tek's BIOS
chapter is screened, its exclusions written by section and line; the BIOS entries cite Redump's datfile, each listing
its market's retail MD5s, and the Japanese entry is renamed; the IOP's cache facts and the SPU2's voice count are read.
The whole record is `Sapphire_References.md`; this section keeps what the build changed and what it measured. No
emulation rule was written; no emulator, SDK or loader source and no firmware was read.

### 12.2 The references

- **Fetched** (`Sapphire_References.md` §1–§2): ps2tek at `1c91660`, the two Toshiba TX79 manuals, GCC's MIPS options,
  the Hot Chips 11 slides on the VUs, ECMA-119 and ECMA-267, ITU-T H.262 (02/2000), Wikipedia's specification page,
  Batocera's page, Redump's datfile; ps2autotests, 78 programs with expected texts as the plan counted. Not fetched:
  the paywalled IEEE papers and Sony's Linux-kit manuals (no lawful copy found).
- **Screened** (`Sapphire_References.md` §3): S2's writer read ps2tek's whole BIOS chapter and is a non-replacement
  writer. Excluded for replacement writers: 21 passages, among them the boot process, the EE patches, the kernel's
  internal structures and scheduler, IOPBOOT, a ROMDIR hex dump, the boot banner and three traces of the BIOS's I/O
  outside the chapter. Excluded for every writer: **two passages of emulators' code** outside the chapter (the RDRAM
  initialisation's register model, lines 701–753, and the "IOP Console" hook, 4182–4208), read by the screening reader
  while finding their extent; and five files of ps2autotests' `common/` that are ps2dev's or ps2sdk's. Four lines of
  ps2tek name an emulator's choice and are marked as such.

### 12.3 The BIOS entries: Redump's names and MD5s

- **The database** [C]: Redump's *Sony – PlayStation 2 – BIOS Images (115) (2022-12-12)*, in the probe's
  `docs/redump/`, named by `bios.rs`'s header and `Sapphire_References.md` §4.
- **Renamed** [R]: `ps2-0210j-20040917.bin` is the DESR video recorders' image, not a console's; the newest Japanese
  retail console version is **`ps2-0230j-20080220.bin`** (SCPH-90000). The core's `bios()`, its descriptor test, the
  C# pack's `Ps2System.Bios` and these pages moved together. The Americas' and Europe's names stand.
- **`src/bios.rs`**, new: three arrays, Japan 27, the Americas 15, Europe 16, written by a script from the datfile
  (never typed), each line with its image's name; the canonical image first, the rest newest first. **Retail** means
  the consumer models (SCPH, DESR, KDL); the 19 DTL images are left out. **The market** is the description's letter, the
  Asian, Korean and Chinese images Japan's and the Russian Europe's. `bios()` gives each entry its list, so core info
  and `firmware_for` carry them under the core ABI's 1.12, and a player's dump is found by its content whatever its
  name (`EmuSen_Firmware.md` §7.4).
- **The gates' Japanese image**: the tester holds no `0230j` [M, names only]. The plan's Q9 named 0210j for the gates;
  it is a recorder's BIOS, so a console's image takes its place (§12.7, a question).
- **No dependency or surface changed**: the field existed since 1.12; Diamond's lists remain owed (`EmuSen_Firmware.md`
  §7.8), untouched here.

### 12.4 Facts read

- **The SPU2: 2 × 24 voices, confirmed** by ps2tek's register map (24 voices a core, the byte counts agreeing),
  Microprocessor Report's table of 1999 ("48-channel") and Wikipedia (`Sapphire_References.md` §5.2).
- **The IOP's caches: no public document gives their sizes.** Sony said in 1999 that the IOP's core is the
  PlayStation's "except for some minor enhancements to the cache" (Microprocessor Report); ps2tek gives only the cache
  control at FFFE0000h. S21 starts from the PlayStation's 4 KiB and 1 KiB, marked [A], with the enhancement a dispute
  candidate (§5.1 there).
- **D-0**: `cpu/ee_cop0/performance`'s expected text holds no count against a time base; Microprocessor Report adds
  the IOP at "33.8 MHz or 37.5 MHz selectable" (`Sapphire_Disputes.md` D-0, the addendum).
- **No PlayStation 2 FPGA core** [M, GitHub's API, 2026-10-24]: MiSTer-devel's 415 repositories include `PSX_MiSTer`
  and nothing for the PlayStation 2, and a search of GitHub's repositories finds none. The plan's §12 [A] is confirmed.

### 12.5 Tests (2026-10-24)

| Item | Result [M] |
|---|---|
| The crate's tests (`cargo test --release`) | **14 of 14**, one new: `each_market_lists_its_retail_images_with_the_canonical_one_first` (the counts, the first of each list, the form of every MD5, none twice, `firmware_for`'s list) |
| WiseMan, `blast.sh`'s filter (`ContractShapeTests`, `CrystalDebugParity`, `CrystalParityTests`, `DebugCodeParity`, `SapphireTests`, `SystemCodeParity`, `SystemTableParity`) plus `FirmwareOverview` | **86 of 86** |

**Seeded faults**, each made in place, its tests run, the file restored from a copy:

| # | Fault | Caught by |
|---|---|---|
| F1 | The core's Japanese entry left as `ps2-0210j-20040917.bin` | `the_descriptors_are_the_consoles` |
| F2 | Europe's entry given the Americas' list | `each_market_lists_its_retail_images_with_the_canonical_one_first` |
| F3 | The C# pack's Japanese name left as `ps2-0210j-20040917.bin` | `SapphireTests.The_library_is_on_the_core_abi_and_its_system_is_the_pack` (the pack's names against the library's) |

### 12.6 What the plan got wrong, or left open

- **§5.9 and Q9**: the newest Japanese retail image is `0230j`, not `0210j`, which is a recorder's; the plan's text
  guessed from the tester's folder. Amended there with a dated note.
- **§3.1's size**: ps2autotests' 1,850 KB is its compressed history; the files are 10.5 MB. The count of 78 stands.
- **§1.1's IOP row** asked for cache sizes "to be read at S2": none is published (§12.4).
- **The predictions**: none is retired or amended at S2; P2 (the output channel) waits for S4's first run.
- **§2.2** expected the interface sections to be cleared or excluded; most are cleared, with sentence-level exclusions
  in the syscalls and whole-section ones for IOPBOOT and the PlayStation compatibility.

### 12.7 Questions this stage leaves

- **The gates' Japanese image** [routine]: the tester holds no `0230j`. *Recommended:* `ps2-0200j-20040614.bin`
  (SCPH-70000), the newest Japanese console image the tester holds whose name carries `j`; `ps2-0220h-20060905.bin` is
  newer but an Asian console's. Either is recognised by its content through Japan's list.
- **DESR and KDL in the lists** [routine]: the recorders' and the television's images are consumer products and boot
  their market's games, so they are listed. *Recommended:* keep them; a list without them would refuse a player's
  recorder dump for no gain.

### 12.8 What was read for this stage

`Sapphire_Plan.md` §1 (the decisions), §2, §3.1, §5.5, §5.9, §7, §11–§14; this page; `Sapphire_Disputes.md`;
`EmuSen_Firmware.md` §0 and §7.4–§7.8; the code of `bios()`, `Ps2System.Bios` and Endou's MD5 lists; and the sources
`Sapphire_References.md` §6 lists. No emulator, SDK or loader source; no firmware. Listed, never opened: the names of
the tester's Japanese PlayStation 2 BIOS images.

### 12.9 What this stage does not cover

- Running anything of ps2autotests (S4), the referees (S3), and every emulation rule.
- Diamond's MD5 lists (`EmuSen_Firmware.md` §7.8), owed by Diamond, not by this stage.

## 13. Stage S3, the referees (2026-10-24)

### 13.1 In one paragraph

The plan's §7 row S3, on the tester's go of 2026-10-24. PCSX2 2.8.2 and Play! 0.72 are installed user-locally from
their pinned releases, and LRPS2, PCEE2 and `play_libretro` from the buildbot's pinned nightlies; PCSX2 runs headless
under a private KWin with its software renderer and is read through PINE, Play! under the same compositor with its RAM
found in its own process, and the cores through the probe, which now names each hardware context it refuses. The gate's
program `programs/record.s`, assembled with `emusen-mips`, gave the same twelve words on PCSX2 (three markets), LRPS2,
PCEE2 and Play!, equal to the values the architecture fixes; `play_libretro` alone runs nothing without an OpenGL
context. The whole record is `Sapphire_Referees.md`; this section keeps what the build changed. No emulator source, no
SDK or loader source and no firmware was read; the BIOS images were linked, never opened.

### 13.2 What was built

- **The crate**: `media::program` writes an EE program of the project's own as an ELF (the header, one segment at
  `PROGRAM_ORIGIN`, 00100000h), and the tests' sample uses it; `examples/elf.rs` assembles a program with `emusen-mips`
  into one; `programs/record.s` (the gate) and `programs/frames.s` (a frame counter for the referees' cost);
  `tests/programs.rs`. **A new dependency, said here** [C]: `emusen-mips` as a dev-dependency only, the assembler of
  the referees' programs, the crate the plan's §9.1 assigns to Sapphire; the library and its cdylib are unchanged.
- **The driver** `EmuSen.WiseMan/Reference/analysis/ps2referee.py` (`fetch`, `ram`, `boot`), modelled on Diamond's
  `psxreferee.py`; its pins `ps2-referees.sha256`; three rows in `libretro-cores.sha256`; `tests/test_ps2referee.py`.
- **The probe**: `SET_HW_RENDER` is answered false as before, and prints one line naming the context and version asked
  for (`hw_context_name`, `hw_refusal`); documented in `EmuSen_Debugging_Tools_Reference_v5.md` §3.50.

### 13.3 Measured

The referees' table, costs and the RAM each exposes are `Sapphire_Referees.md` §2; the record is §4.1. In short:
PCSX2 2.56 ms a frame unlimited and about 1.2 s a run, Play! 0.4 ms of CPU a frame at the console's speed, LRPS2
0.28 ms a frame, PCEE2 2.31 ms; Jak and Daxter booted once on PCSX2 (PINE: SCUS-97124, running after 58 s).

### 13.4 Tests (2026-10-24)

| Item | Result [M] |
|---|---|
| The crate's tests (`cargo test --release`) | **16 of 16**, two new in `tests/programs.rs` |
| The probe's tests (`cargo test --release --features libretro`) | **51 of 51**, one new: `a_refused_hardware_context_is_named_with_its_version` |
| `analysis/tests/test_ps2referee.py` (`python3 -m unittest`) | **18 of 18**, all new: PINE's framing against a fake server, the scan against a faked process with a decoy, the BIOS folder of links, the settings file, the probe's command line |
| WiseMan, `blast.sh`'s filter (`ContractShapeTests`, `CrystalDebugParity`, `CrystalParityTests`, `DebugCodeParity`, `SapphireTests`, `SystemCodeParity`, `SystemTableParity`) | **79 of 79** |

**Seeded faults**, each made in place, its tests run, the file restored from a copy:

| # | Fault | Caught by |
|---|---|---|
| F1 | PINE's size field not counting itself | `PineTests`, four tests (the fake server reads a short message) |
| F2 | The scan reading an address on the wrong side of the done word | `ScanTests.test_the_record_is_read_relative_to_the_done_word`, `test_a_read_past_what_is_mapped_is_refused` |
| F3 | The BIOS folder's check that every `.bin` is a link removed | at first **nothing**: the check of the gates' three names hid it; `test_any_other_image_copied_in_is_refused` was added and catches it |
| F4 | The scan taking the first place the done word appears, without the program beside it | four `ScanTests`, the decoy among them |
| F5 | The probe naming OpenGL core as OpenGL | `a_refused_hardware_context_is_named_with_its_version` |
| F6 | `record.s` writing its done word 4 bytes off | `tests/programs.rs`, `the_record_ends_with_its_done_word_and_a_branch_to_itself` |

### 13.5 What the plan got wrong, or left open

- **§1.6's first finding** is half right: LRPS2 and PCEE2 run without a hardware context on their software renderers,
  and PCEE2 hands pictures; only `play_libretro` cannot run. Amended there with a dated note.
- **§1.6's table**: PCEE2 is 2.9.138 and needs no `resources` folder for an ELF; Play! needs no CHD for an ELF.
- **PINE's draft numbers** the title, serial, CRC, version and status one lower than PCSX2 answers them.
- **§10's hardware render** is evaluated (`Sapphire_Referees.md` §6) and not built: the condition of §11 Q4 is not met.

### 13.6 Questions this stage leaves

- **A hardware context for the probe** [decision] (*decided 2026-10-24 by the tester, as recommended*): *recommended* none (form C of `Sapphire_Referees.md` §6): Play!'s
  pictures taken as RAM by the project's GS programs reading the GS's memory back to the EE, with headless EGL (form A)
  held in reserve for the first GS dispute that needs Play!'s picture and finds its readback wanting.
- **DobieStation** [decision] (*decided 2026-10-24 by the tester, as recommended*): *recommended* not installed; built user-locally only if a dispute needs a third lineage
  that ps2tek does not decide (`Sapphire_Referees.md` §7).
- **S3's writer is exposed** [routine]: PCSX2's log of the BIOS's boot was read while finding a start-up fault
  (`Sapphire_Referees.md` §5). *Recommended*: recorded, the writer excluded from the open BIOS, and the driver keeping
  no log, as built.

### 13.7 What was read for this stage

`Sapphire_Referees.md` §9 lists it. No emulator, SDK or loader source; no firmware.

### 13.8 What this stage does not cover

- ps2autotests' first run and its output channel (S4, P2), and the runners of §3.3 (S4).
- Pictures from any referee (`Sapphire_Referees.md` §6 proposes how they will be taken).
- The handheld: no referee was run on it.

---

## 14. Stage S4, the runners (2026-10-24)

### 14.1 In one paragraph

The plan's §7 row S4, on the tester's go of 2026-10-24, preceded by the two gates the tester asked for the same day
(G10, start without a disc, and G11, the Memory Cards window's PlayStation 2 tab, `Sapphire_Plan.md` §8, placed at S25
and offered at S29; nothing of either is built). The four runners of the plan's §3.3 are in WiseMan as data beside the
generic runner (`Nephrite_Native.md` §3): the synthetic program `SyntheticPs2Elf` with a recording program, the synthetic
disc builder for CD sets and single- and dual-layer DVDs on one ISO 9660 writer the PlayStation's builder now shares,
the expected-text grader `ExpectedText`, and the RAM-record reader `RamRecordForm`, whose one form serves Diamond's
programs and Sapphire's. One ps2autotests program, `cpu/ee/alu`, was run on PCSX2 with the player's BIOS: its whole
expected text reached **the IOP's console**, not the EE's, which a project program shows carries KPUTCHAR (P2 held,
§14.5). Each runner has a positive and a negative control. No emulator source, no SDK or loader source and no firmware
was read, and nothing of PCSX2's log but the program's own lines was seen.

### 14.2 The synthetic discs, and the writer they share

- **`Fixtures/Iso9660Image.cs`**: ECMA-119's primary descriptor, set terminator, both path tables and one root directory
  with its files, written into 2,048-byte sectors. It is the PlayStation builder's former private code moved, not
  copied: `SyntheticPsxDisc` and `SyntheticPs2Disc` both call it, and the PlayStation's disc is **byte-identical** to
  what it was, data and raw track and sheet, for three markets and both modes [M: a throwaway test comparing the old
  builder's output with the new, passing, then removed]. The PlayStation 2's volumes gain the path tables and the full
  descriptor they lacked at S1.
- **`SyntheticPs2Disc`**: `Cd()` (the volume, its raw mode 2 form 1 track through `CdSector` and a one-track sheet;
  `WriteSet` writes `.cue` and `.bin`), `Dvd()` (2,048-byte sectors, the root at sector 300, at 261 since §40.14;
  `WriteSet` writes `.iso`),
  and **`DualLayerDvd(layer0)`**: layer 0 of the given sectors, a file `LAYER1.BIN` placed 32 sectors past the break and
  listed in the root, and the primary descriptor and terminator again at layer 1's sectors 16 and 17. The record names
  the break (`LayerBreak`), since an image records none.
  - **Why layer 0 is a multiple of 16** [R ECMA-267, 3rd edition, §25 and the physical format's bytes 13–15]: in OTP
    mode layer 1's first sector number is layer 0's last inverted, and that last is chosen so its inverse is a multiple
    of 16; with the data zone starting at 30000h, layer 0 holds a multiple of 16 sectors. The builder refuses any other.
  - **Why a second descriptor at layer 1's sector 16** [A]: an image is its logical sectors only, and the layer break
    lives in the lead-in's physical format information, which no image carries. CDVDMAN exports a per-layer file search
    (`sceCdLayerSearchFile(fp, path, layer)`, ps2tek's export list, cleared for every writer) and a call returning layer
    1's start (`sceCdReadDvdDualInfo`), which suggests a volume readable from each layer. The layout is the builder's
    choice, to be confirmed or corrected at S23 when `emusen-disc` gains the DVD medium (§14.8 Q2).
- **The host's reader** (`EmuSen.Crystal/Media/DiscSectors.cs`) is the control: it reads `SYSTEM.CNF` and the program from
  every form, raw track included, and `LAYER1.BIN` whole across the break.

### 14.3 The RAM-record reader, and the recording program

`Fixtures/RomRunner/RamRecord.cs`. A **form** is a space, the done word's place and value, the record's start, and the
segment a program addresses them through: **Diamond's** `ram`, D1A0D00Eh at 0FFFF0h, words from 100000h, through
80000000h (`Diamond_Referees.md` §2.4); **Sapphire's** `ram`, 5A99D00Eh at 1FFFF0h, words from 200000h, through
00000000h (`Sapphire_Referees.md` §4.1). `Read` returns the words only once the done word is in place; `Grade` passes,
fails naming each word that differs, or is incomplete without the done word; `Protocol` is the form as data for
`TestRomGrader`. `Program(words)` writes a bare MIPS I program in the form (each word stored in turn, the done word, a
branch to itself) with WiseMan's `MipsAssembler`, and `SyntheticPs2Elf.Recording(words)` makes it an EE program.

**On the referees** [M, `ps2referee.py ram … --until 1FFFF0=5A99D00E 200000:12`]: `Recording` with `record.s`'s twelve
words, written by `SapphireRunnerTests` where `EMUSEN_SAPPHIRE_PROGRAMS` names a folder, read back equal on **PCSX2**
(Americas) and **Play!**; the same program in an ELF for machine 40 never wrote its done word on PCSX2 in 15 s.

### 14.4 The expected-text grader

`Fixtures/RomRunner/ExpectedText.cs`, after ps2autotests' README ("A test that has run accurately will produce the same
data as contained in the `.expected` file … take care to normalize newlines") [R]. Line ends are normalised (CR LF and
lone CR to LF, the final line feeds dropped); the output is graded from the expected text's first line on, so a
kernel's own lines before it are left out, as Diamond's heading rule does (`Diamond_Native.md` §11.12); after that every
line must be equal and nothing may follow. A run cut off mid-text is **incomplete**, any other difference **failed** with
its line named. `Protocol(suffix, expected, space)` is one program as data; `Corpus(root, space)` makes one protocol for
each `.elf` and `.irx` with its `.expected` beside it. The space defaults to `tty`, and `TtyReader.Text` takes a space,
so the channel is data (§14.5).

### 14.5 The output channel, by observation (P2)

`ps2referee.py tty pcsx2 ELF EXPECTED --channel ee|iop|none` (`Sapphire_Referees.md` §2.1) turns on one of PCSX2's two
console channels and returns only the program's own lines. `programs/kputchar.s` is the instrument's control: a bare EE
program writing `EMUSEN KPUTCHAR CONTROL` a byte at a time to **KPUTCHAR, 1000F180h** [R ps2tek "Memory Map", "Misc
registers": "KPUTCHAR - Console output"], through KSEG1.

| Program | BIOS | Channel | Expected lines found [M] | Wall |
|---|---|---|---|---|
| `kputchar.elf` (the project's) | Americas | EE | 1 of 1 | 1.7 s |
| `kputchar.elf` | Americas | IOP | 0 of 1 | 15 s (timed out) |
| `cpu/ee/alu.elf` | Americas | EE | 0 of 752 | 40 s (timed out) |
| `cpu/ee/alu.elf` | Americas | **IOP** | **752 of 752**, the 787-line region `diff`-identical to `alu.expected` | 1.7 s |
| `cpu/ee/alu.elf` | Americas | none (negative control) | 0 of 752 | 15 s (timed out) |
| `cpu/ee/lsudelay.elf` | Americas | IOP | 3 of 3, identical | 1.7 s |
| `cpu/ee/alu.elf` | Europe | IOP | 752 of 752 | 1.6 s |

**Finding** [M, A]: the EE test's text, `-- TEST BEGIN` and `-- TEST END` included, reaches the IOP's console and none
of it KPUTCHAR, so an EE program's output crosses the SIF to the IOP before it is printed: P2 held. The expected texts
were recorded on consoles through ps2link [R README], yet on a retail BIOS with no ps2link the same text appears whole on
the IOP's console, so the expected files and a retail BIOS run agree. **What this asks of Sapphire**: its TTY reader
must keep what the IOP prints. How PCSX2 catches that text is not known here and is not read; ps2tek's section "IOP
Console", excluded for every writer, is described in the S2 screening as an emulator's hook on program-counter values
inside a BIOS image (`Sapphire_References.md` §3), which Sapphire may not copy. The question is §14.8 Q1.

**Exposure**: the driver read PCSX2's log and printed only the program's region; this stage's writer saw no line of the
BIOS's output and stays eligible as a replacement writer as far as the log goes. Of ps2tek, the writer read the memory
map, the CDVD disk-type register and CDVDMAN's export list, each cleared for every writer (`Sapphire_References.md` §3);
of ps2autotests, the README, `common/common-ee.cpp` and `common-ee.h` (the project's own, not among the five excluded),
`cpu/ee/alu.cpp`'s first lines and the expected texts.

### 14.6 Tests (2026-10-24)

| Item | Result [M] |
|---|---|
| `SapphireRunnerTests` (new), with `EMUSEN_PS2AUTOTESTS` and `EMUSEN_SAPPHIRE_PROGRAMS` set | **9 of 9**: the program's fields and spoiled forms; every disc form through the host's reader and the core; the refusals and the builder's rule of 16; the grader on text; the grader on Diamond's TTY through the generic runner; the fetched corpus counted (78: 55 `.elf`, 23 `.irx`); the reader on snapshots; the reader on Diamond's own run and not in Sapphire's form; the referees' programs written |
| WiseMan, `blast.sh`'s filter (`DiamondTests`, `SapphireRunnerTests`, `SapphireTests`) plus `SystemCodeParityTests`, which builds the PlayStation 2's discs | **45 of 45** |
| The crate (`cargo test --release`) | **17 of 17**, one new: `the_channel_control_writes_its_line_to_kputchar_a_byte_at_a_time_then_its_done_word` |
| `analysis/tests/test_ps2referee.py` | **23 of 23**, five new (`TtyTests`): only the program's lines returned, a fixed prefix cut, nothing without the program, a wrong line kept and not counted, one channel a mode |

**Seeded faults**, each made in place, the tests run, the file restored:

| # | Fault | Caught by |
|---|---|---|
| E1 | `SyntheticPs2Elf`'s memory size without the bss | `The_synthetic_program_is_an_ee_executable_and_its_spoiled_forms_are_refused` |
| E2 | The segment's file offset pointing at the program header | the same |
| E3 | The entry 4 bytes past the origin | the same |
| D1 | The dual-layer image without layer 1's descriptor | `Each_synthetic_disc_reads_back_through_the_hosts_reader_and_sapphire_takes_it` |
| D2 | The builder's rule of 16 removed | `A_disc_that_names_no_program_is_refused_and_the_builder_refuses_a_layer_break_off_the_rule` |
| D3 | The CD's sheet naming mode 1 | `Each_synthetic_disc_reads_back…` |
| X1 | A lone CR left unnormalised | `The_expected_text_grader_passes_only_the_whole_text_with_line_ends_normalised` |
| X2 | One line past the end allowed | the same |
| X3 | A cut-off run graded failed, not incomplete | the same |
| R1 | The reader ignoring the done word | `The_ram_record_reader_reads_words_only_behind_the_done_word` and the Diamond test |
| R2 | The recording program storing each word 4 bytes on | `The_ram_record_reader_reads_a_record_diamond_left_and_not_one_in_another_form` |
| R3 | Diamond's form with its done word at 0FFFF4h | at first **nothing**: the recording program and the reader share the form, so the run was self-consistent; the snapshot test now lays each form's record out at the documents' addresses, not the form's, and catches it |
| T1 | `ps2referee.py`'s `program_text` returning the whole log | `TtyTests`, two tests |
| T2 | The fixed prefix not cut | `test_a_fixed_prefix_on_every_line_is_cut_by_the_first_lines` |
| T3 | The IOP mode switching both consoles on | `test_each_channel_switches_one_console_on` |

### 14.7 What the plan got wrong, or left open

- **§3.3 and S4's row** list `SyntheticPs2Elf` and the disc builder as S4's; both began at S1 (§5), and S4 extended them
  (the bss, the origin, the recording program; the CD set, the dual-layer DVD, the shared writer).
- **§3.3's TTY reader "over the output channels ps2autotests is found to use"**: the channel is found, but no core
  keeps it yet, so the reader is the existing `TtyReader` with the space as data; Sapphire's crate gains the space when
  it can print (S10), by Q1's answer.
- **The README's "same data"** is read as the text from its first line on, so that a kernel's lines before it do not fail
  a run; nothing after the last line is allowed.

### 14.8 Questions this stage leaves

1. **Q1 [decision], how Sapphire keeps what the IOP prints.** *Decided 2026-10-24 by the tester: as recommended;
   recorded in `Sapphire_Plan.md`'s decisions and at S10 and O8.* PCSX2 catches it by a means not read here, which ps2tek's
   excluded section describes as a hook on program-counter values inside a BIOS image; Sapphire may not do that.
   *Recommended*: on the open BIOS, its TTY device writes to a port of the project's own, as Diamond's open kernel prints
   through the DUART; on the player's BIOS, the console is caught at the IOP's module interface, the output export
   resolved at run time through the export tables ps2tek lists (cleared for every writer), never at an address inside
   the image, and kept in a `tty` space; decided before S10, whose gate needs it. Until then the EE stages are graded by
   the project's RAM records, as the plan's §11 Q6 already says.
2. **Q2 [routine], the dual-layer layout.** *Recommended*: at S23, a synthetic dual-layer image booted on PCSX2 and Play!
   with a program that reads `LAYER1.BIN` and records it, to learn whether they take the layout of §14.2, and the builder
   corrected if they do not.
3. **Q3 [routine], a referee baseline of the EE tests.** *Recommended*: when S10 begins, all 55 `.elf` programs run on
   PCSX2 through `tty` and graded by `ExpectedText` (about 2 s each), so that each later miss on Sapphire is known to be
   the core's or the referee's.

### 14.9 What was read for this stage

`Sapphire_Plan.md` §3, §4.1, §4.4, §7, §8, §9.5, §11 and §13; `Sapphire_Native.md`; `Sapphire_References.md` §3 (the
exclusions, by `DOCS.txt`); `Sapphire_Referees.md`; `Nephrite_Native.md` §3; `Diamond_Shell.md` §1.1, §1.2, §1.5 and
§4.5; `EmuSen_SaveMedia.md` §6.2 and §6.5; `MainWindow.ConsoleStart.cs`, `MemoryCardsWindow.cs` and the fit audit's
memory card cases; the project's runners, builders and `DiscSectors.cs`; ECMA-267 §25 and §26.5.1; ps2tek's memory map,
the CDVD disk-type register and CDVDMAN's export list; ps2autotests' README, `common-ee.cpp`, `common-ee.h`, `alu.cpp`'s
opening and the expected texts. PCSX2's own settings file was read for the names of its log switches. No emulator, SDK
or loader source; no firmware; none of PCSX2's log but the programs' own lines.

### 14.10 What this stage does not cover

- A `tty` space in the crate, and the IOP console's capture (S10, Q1).
- IOP programs (`.irx`): the protocol claims them, but they are loaded by the IOP's kernel and run from S21 on.
- Any ps2autotests program on Sapphire, which runs nothing yet.
- G10 and G11, which are S25's and S29's.

## 15. Stage S5, the R5900's and the vector units' tables (2026-10-24)

### 15.1 In one paragraph

The plan's §7 row S5 and §9.1, on the tester's go of 2026-10-24, preceded by the record of §14.8 Q1's decision in the
plan (the IOP's console caught at the module interface on the player's BIOS, a port of the project's own on the open
BIOS; `Sapphire_Plan.md`'s decisions, S10 and O8). `emusen-mips` gains `src/r5900.rs`, the EE core's decode, disassembly
and assembly as a table of its own (290 mnemonics), and `src/vu.rs`, the vector units' operations as one table read in
micro mode (a 64-bit pair, an upper word with five flags and a lower word) and in macro mode (the EE's COP2, VU0's names
with a `v`). The map is the TX79 manual's opcode tables where the EE agrees and ps2tek's where it differs, each of the
83 differing cells under one of three disputes (D-1 to D-3); the vector units' layouts are Sony's Hot Chips slides', and
two choices the documents leave open are D-4 and D-5. The R3000's, the VR4300's and the RSP's text is unchanged over
three million words, and Diamond's, BlackMoon's, the platform library's and Sapphire's tests pass. `EmuSen_Mips.md` §8 and
§9 are the record of the tables; this section records the stage. No emulator, assembler, binutils, LLVM or SDK source and
no Sony manual was read.

### 15.2 What was built

- **`r5900.rs`**: one Rust table per table of the manual (A.5, B.5, C.2, D.4), indexed by the same field, so that the
  source reads against the manual cell for cell; `operation(word)` gives the operation or `None` for a reserved
  encoding, `decode(word, pc)` the text and the address a word names, in the VR4300's `Instruction` shape; `Dialect::R5900`
  sends `disassemble`, `assemble` and `assemble_line` to it. Appendix A's spelling, the IOP's (`EmuSen_Mips.md` §8.4).
- **`vu.rs`**: ps2tek's "COP2 Special1" and "COP2 Special2" as two tables of operations, each with its micro name, its
  half (upper, lower or macro only) and its operand form; `operation`, `text`, `encode`, `disassemble(pair)`,
  `assemble_pair` and `assemble`. COP2's rs values from 10h reach it from `r5900.rs`.
- **The assembler** reads both: `assemble(source, origin, &Dialect::R5900)` for EE programs, `vu::assemble(source)` for
  microprograms; the helpers it shares with the MIPS I assembler moved to crate visibility, not copied.

### 15.3 Where the documents ran out

- **The VU micro-only lower instructions** (those with their own opcode in bits 31–25: the LQ and SQ kind, the integer
  immediates, branches, the flag instructions, the EFU, XGKICK, XTOP, XITOP, MFP, WAITP) have **no encoding in any
  source this stage may read**: ps2tek names some of them in its pipeline and VIF sections and gives no bits, and the
  Hot Chips slides draw only the `1000000` class. Such a lower word is written `.word`. S13 cannot start without them
  (§15.6 Q1).
- **Operand layouts** for 30-odd VU operations outside the FMAC family (ITOF, FTOI, ABS, CLIP, MOVE, MR32, the
  loads and stores but LQI, DIV, SQRT, RSQRT, MTIR, MFIR, ILWR, ISWR, the random unit, the integer ALU, CALLMS) are not
  given; they are written `Raw`, the name and the 25 bits, which reads back exactly and claims nothing.
- **COP2's interlock bit**: ps2tek names QMTC2.I and CTC2.I but not the bit; the moves are decoded without it.
- **VU0's control registers** are not numbered by any source read; CFC2 and CTC2 write them `$n`.

### 15.4 Tests (2026-10-24)

| Item | Result [M] |
|---|---|
| `emusen-mips` (`cargo test --release`) | **18 of 18**, five new: `every_encoding_of_the_tx79_tables_decodes_disassembles_and_reassembles` (the gate: 806 cells, 261 named, 462 reserved, 83 differing as ps2tek reads them), `the_ee_s_cells_by_ps2tek`, `the_spelling_is_appendix_a_s`, `every_encoding_of_the_documented_tables_reads_back_in_both_modes` (120 VU operations, both modes), `the_hot_chips_examples_and_the_flags`; clippy clean |
| The R3000's, VR4300's and RSP's text, unchanged | 3,020,480 words through `disassemble` (MIPS I with `decode` and the assembler's read-back, and the VR4300's), `vr4300::decode` and `rsp::decode`: the same four hashes before and after the stage, by a throwaway program kept out of the repository |
| `blast.sh`'s crates (`cargo test --release`) | `blackmoon-gte` **10 of 10**, `blackmoon-r3000` **38 of 38**, run again with `EMUSEN_BLACKMOON_CORPUS` set so that its r3000 suite's 55,000 opcodes go through the MIPS I decode and assembler (38 of 38), Diamond **306 of 306** (its goldens), Diamond's `precision` **8 of 8** and `v1` **5 of 5**, Sapphire **17 of 17**, `Platform/systems` **39 of 39** (MarsRT's VR4300 and RSP text through the platform library) |
| WiseMan, `blast.sh`'s filter (`CheatCodec`, `DebugCodeParity`, `Diamond`, `SapphireTests`, `SystemCodeParity`, `SystemTableParity`) | **116 of 116** |

**Seeded faults**, each made in place, the crate's tests run, the file restored:

| # | Fault | Caught by |
|---|---|---|
| F1 | MMI0's PADDW and PSUBW cells swapped | the gate test |
| F2 | C.LT.S given the `fd, fs` form | `the_spelling_is_appendix_a_s` (the round trip alone cannot see a form that reads back its own text) |
| F3 | the assembler's check that a word decodes to the mnemonic it was given, removed | `the_spelling_is_appendix_a_s` (`mfc0 v0, Debug` accepted) |
| F4 | BC1FL and BC1TL removed | the gate test (D-2's cells) and `the_ee_s_cells_by_ps2tek` |
| V1 | Special2 indexed without its column bits | both VU tests and `the_ee_s_cells_by_ps2tek` |
| V2 | the field mask's letters in reverse | both VU tests and `the_ee_s_cells_by_ps2tek` |
| V3 | the lower class `1000000` as `1000001` | at first **nothing**: the writer and the reader shared the constant, so every round trip agreed; the slides' literal words (`0x8104_2B7C`, LQI; `[e] nop | move`) are now checked bit for bit, and catch it |

### 15.5 What the plan got wrong, or left open

- **§9.1's "lacks some MIPS III instructions [R the TX79 manual, to be read at S5]"**: read; they are DMULT, DMULTU,
  DDIV, DDIVU, LL, LLD, SC and SCD (§3.3's note and A.5's η cells), and on the EE also the FPU's double and long formats
  (D-1), which the plan's §1.1 already said in other words.
- **S5's gate, "every encoding of the TX79 manual's opcode tables decoded, disassembled and reassembled to the same
  word"**, cannot hold literally where the EE differs: the 83 differing cells decode as ps2tek reads them (66 of them
  reserved), each named with its dispute in the test, as P6 requires. The other 723 cells meet the gate as written.
- **"Every VU encoding in the documented tables likewise"** holds for ps2tek's two tables (120 operations, both modes);
  the micro-only lower instructions are in no document this stage could read (§15.3).
- **P4** (the R5900 differs from the TX79 in an instruction's result that ps2autotests records) is not retired: S5
  decodes and does not execute. D-1 makes it likely for COP1.

### 15.6 Questions this stage leaves

1. **Q1 [decision], the VU micro-only encodings.** S13 needs the lower instructions with their own opcodes, and S12 and
   S13 the operand layouts of the operations now written `Raw`; no document allowed at this stage gives them. The
   plan's §11 Q1 (2026-10-24) allows Sony's Linux-kit manuals, among them a VU user's manual, from the tester's own or a
   lawful copy, while this stage's sources excluded Sony's VU manual. *Recommended*: the tester settles which holds; if
   Q1 does, a lawful copy of the Linux kit's VU manual is sought and read by S12's writer, each use cited by section, the
   `Raw` forms and the `.word` lower words replaced there; if not, a dispute step per instruction, learning the encoding
   by running candidate words on the referees as black boxes, which the disputes log's rules allow but which would
   take far longer and rest on emulators' choices.
2. **Q2 [routine], fields fixed at zero.** The decode ignores them, as the VR4300's does (`EmuSen_Mips.md` §8.2).
   *Recommended*: kept; S6's interpreter revisits it if `cpu/ee/*` or a project program shows a reserved instruction
   exception for one.
3. **Q3 [routine], evidence for D-1 and D-2.** *Recommended*: at S9, a project program executing each TX79-only COP1
   encoding and each likely FPU branch, its record read on PCSX2 and Play!, as the disputes' evidence.

### 15.7 What was read for this stage

`Sapphire_Plan.md` §2, §7, §9.1, §11 and §13; `Sapphire_References.md`; `Sapphire_Native.md` §14; `EmuSen_Mips.md`;
`emusen-mips`'s sources; Toshiba's *TX79 Core Architecture* rev. 2.0, §3.3, §4.2's Table 4-1, §10.5, the opcode tables
A.5, B.5, C.2 and D.4 and the Format lines and encoding pictures of Appendices A to D (the TX7901 manual was searched and
has no opcode table); ps2tek's "EE" chapter (architecture, registers, instruction decoding, COP0 registers, COP1) and
"Vector Unit" chapter; Kunimatsu et al.'s Hot Chips 11 slides whole. No emulator, assembler, binutils, LLVM, GCC or SDK
source, no Sony manual and no firmware.

### 15.8 What this stage does not cover

- Execution of any kind: the interpreter is S6, COP1 S9, VU0 macro mode S12, micro mode S13.
- The VU micro-only lower instructions and the `Raw` operations' operand layouts (§15.3, Q1).
- A debugger surface: the processors' disassembly through the platform library waits on S31.

## 16. Stage S6, the R5900 interpreter, integer (2026-10-24)

### 16.1 In one paragraph

The plan's §7 row S6, on the tester's go of 2026-10-24, preceded by its own commit: Sony's Linux-kit manuals, accepted
by the tester the same day as the lawful source §11 Q1 asks for, found as the kit's own Release 1.0 Disc 1 kept by the
Internet Archive, each title page checked (Version 5.0, October 2001, no confidentiality marking) and pinned
(`Sapphire_References.md` §1.1). The crate gains `src/ee/`: the EE core's integer interpreter, which decodes every word
through `emusen-mips`'s `r5900` table and executes it as the *EE Core Instruction Set Manual*'s Operation sections
write it, with 128-bit general registers, HI and LO of both pipelines, SA, the exceptions an integer instruction
raises, and the timing model of §5.2 with the *EE Core User's Manual*'s issue and latency tables pinned as data.
`sapphire` now depends on `emusen-mips` (a dependency the plan's §9.1 names; it was a development dependency since
S3). Seventeen programs of the project's own, generated as assembly over every integer encoding with the record the
manual's definitions give, run on the interpreter equal to that record, and on PCSX2 and Play! as black boxes: the
arithmetic, logic, shifts, both multipliers, both dividers, the loads and stores, LQ and SQ, every branch and jump and
the links made in kseg0 equal on both lineages; where they are not, the disagreement is a dispute (D-6 to D-11). The
interpreter is not yet run by the machine; the ELF loader is S10's. No emulator, SDK or loader source and no firmware
was read.

### 16.2 What was built, and the readings the manual leaves open

- **`ee/mod.rs`**: `Ee` (32 × 128-bit GPRs, HI and LO as 128-bit registers whose bits 63..0 are pipeline 0's and
  127..64 pipeline 1's, SA, the PC with its delay-slot state, COP0's 32 words), `step` and `run`, the level 1 exception
  sequence of the EE Core User's Manual §4.1.2 (EPC and Cause.BD left alone when Status.EXL is set; the vector by Status.BEV),
  the operating modes of §3.2's table and the address errors a mode raises (§4.2.9). An instruction a later stage
  executes stops the interpreter with `Halt::Unimplemented` and its word, never a made-up result; a coprocessor
  instruction whose coprocessor is unusable raises the unusable exception first, as §4.2.14 says.
- **`ee/ops.rs`**: one function a mnemonic, 128 mnemonics, looked up by the decoder's own name, so that the crate holds
  no encoding; a memo of 4,096 words keeps the lookup off the string table (the decode is a pure function of the word).
- **`ee/bus.rs`**: the `Bus` the core reads through (naturally aligned accesses by physical address), and `Ram`, RDRAM
  alone, for the tests.
- **`ee/timing.rs`**: §5.2's model, below.

**Readings** [R the *EE Core Instruction Set Manual* (INST) and *EE Core User's Manual* (CORE), Version 5.0; A]:

- **Bits 127..64.** Every Operation section but LQ's writes `GPR[rd]63..0` alone, so a 64-bit or narrower instruction
  leaves the upper half as it was; CORE §1.4.1 says the upper half is "only used by" the 128-bit instructions. Both
  referees agree (the program `upper`).
- **Links are zero-extended.** JAL and JALR write `zero_extend(PC + 8)`, the branch-and-link forms `PC + 8`, a 32-bit
  value; a link made at 8010xxxxh therefore reads 00000000_8010xxxxh. Both referees agree (`branches`'s kseg0 part).
- **An unusable restriction is not a trap.** A word instruction given an input that is not a word value, or a divide
  by zero, has an undefined result (each Restrictions section); the interpreter computes on bits 31..0 and gives a zero
  divisor the result D-6 records. MULT, MULTU, MADD and MADDU write rd as their Operation sections do, though MULTU's
  and MADD1's Descriptions name other registers (the Operation section is taken where the two differ).
- **MADD's accumulator** is HI31..0 || LO31..0 of its pipeline (INST, MADD), read through the multiplier's own
  forwarding: the timing model does not wait for HI and LO before a MADD, since §2.12 gives MADD a repeat rate of 2.
- **LQ and SQ** mask the address's four low bits and raise no alignment error (INST, LQ, SQ).
- **Cause.CE** is written by the unusable exception alone (§4.2.14 lists it there and for no other exception); the
  register table calls it undefined otherwise, and the programs leave it ungraded there.
- **COP0 at this stage**: MFC0, MTC0, ERET, EI and DI; Status's writable bits per §3.2's table, Cause, BadVAddr and PRId
  read-only, PRId's implementation number 2Eh with the revision, which the manual does not give, zero. The TLB, the
  caches and CACHE, Count and Compare and the performance counters are S7's; their registers hold what is written.
- **Memory at this stage**: kseg0 and kseg1 lose their top three bits (CORE §5.1.6); the mapped segments are taken the
  same way until S7's TLB, a stand-in named here, not a reading of the manual.

### 16.3 The documents' answers to S5's disputes

The EE Core manuals settle D-1 to D-3 and the VU User's Manual D-4 and D-5, each as S5 implemented it
(`Sapphire_Disputes.md`): INST §7.4's COP1 tables hold only S and W with the EE's own operations (D-1), §7.4.2 gives
BC1FL and BC1TL (D-2), §7.1.1 places COP2 at 12h and LQC2 and SQC2 at 36h and 3Eh (D-3); the VU manual's chapters 4.2
and 4.3 place ABS, FTOI, ITOF and NOP among the upper instructions and MOVE, MR32, MTIR, MFIR and WAITQ among the
lower (D-4), and its §4.1.3 puts x at bit 24 of a lower word and bit 56 of a pair (D-5). The gaps S5 named and the
sections that fill them at S12 and S13 are `Sapphire_References.md` §1.1's table; none is filled at this stage.

### 16.4 The timing model (§5.2, Q14)

Pinned as data and by test [R CORE §1.3.5, §1.3.6, §2.12]: §2.12's integer and floating-point latency tables row by row
(the TX79 manual's Table 3-21 agrees on every integer row; its floating-point rows are its own derivative's and are
not used); Table 1-1's categories and physical pipes; Table 1-2's logical pipes; Table 1-3's pairing cells, two X and
five Y. The rules:

1. Instructions issue in order, at most two a cycle, the second beside the first when Table 1-2 gives them different
   logical pipes, Table 1-3 allows the pair, and the second neither reads nor writes what the first writes.
2. An instruction issues no earlier than the cycle each register it reads is ready: the producer's issue cycle plus its
   §2.12 latency. A load's latency is 1, so the dependent instruction loses only the pairing; CORE §2.3.2's
   "hardware interlocks insert additional clock cycles" is read as that pair lost.
3. MAC0 and MAC1 each take a multiply or multiply-add every 2 cycles and a divide every 37 (§2.12's repeat rate);
   other instructions run beside a divide.
4. A Y cell issues the pair and stalls one cycle in A (Table 1-3's note).
5. A taken branch or jump pairs with its delay slot; its target starts a new cycle.

**Named losses, each a dispute candidate if an oracle ever measures it** [A]: no branch-prediction cost (CORE names the
BTAC and BHT but gives no misprediction penalty); no fetch alignment of the instruction pairs; no cache (S7), so every
load is §2.12's hit; COP0 moves, SYNC and ERET charged one cycle, since §2.12 tabulates no kernel instruction; an
exception charged as the instruction that raised it, its entry cost not documented; the short-loop defect the kit's
*Restriction* (1) describes ("may end after executing once or twice, under special conditions") not modelled, its
conditions unknown. No suite measures the EE's cycles but `ee_cop0/performance`, which S10 runs.

### 16.5 The gate: the programs (measured 2026-10-24)

`tests/s6/` generates seventeen bare programs as `emusen-mips` assembly (the R5900 dialect), each loaded and entered
at 00100000h, writing its record from 00200000h and 5A99D00Eh at 001FFFF0h as `record.s` does. Each case loads its
operands from constants, sets the destination to a sentinel, runs the instruction, and stores what the instruction
writes. The expected record comes from `tests/s6/manual.rs`, the Operation sections transcribed as functions on bit
strings (bit slices, `sign_extend`, `zero_extend`, 128-bit arithmetic), written apart from `ops.rs`. The operands:
eleven word values and six 64-bit values that are not words, seven immediates, five shift amounts; a word instruction
is given only the inputs its Restrictions section defines.

| Program | Cases | What it covers |
|---|---|---|
| `three` | 5,281 | every three-register operation over every pair of operands it may take; SYNC, SYNC.P, PREF |
| `immediates` | 1,769 | the immediate forms over every immediate, the shift-amount forms over every amount |
| `muldiv` | 1,408 | MULT, MULTU, MADD, MADDU, DIV, DIVU in both pipelines, HI and LO of both preset and read back after each |
| `sa` | 205 | MTSAB, MTSAH, MFSA, MTSA |
| `memory` | 374 records | every load and store at every byte of a quadword its alignment allows; LQ and SQ at every byte |
| `branches` | 886 records | every branch over operands whose 64-bit and 32-bit readings differ, taken and not, likely and not; J, JAL, JR, JALR; links made in kseg0 |
| `exc-*` (nine) | 894 records | address errors, overflow, traps true and false, SYSCALL and BREAK, reserved encodings, COP1 and COP2 unusable, an exception in a delay slot, EI and DI, user mode's refusals; through a handler at 80000180h that records Cause, CE, EPC, BadVAddr and Status and returns past the instruction |
| `upper` | 22 records | bits 127..64 kept by eleven narrower writes |
| `undefined` | 92 records | a zero divisor, word instructions on values that are not words, MTSA of an arbitrary value |

**Tests** [M, `cargo test --release`, `tests/interpreter.rs`]: **23 of 23** — the programs, each equal to the manual's
record word for word; `the_programs_execute_every_integer_encoding_the_interpreter_has` (all 128, counted from the
assembled words through the decoder); `record_s_gives_the_twelve_words_the_referees_gave`; a fetch from an unaligned
target, BEV's vector, an exception inside a handler keeping EPC, ERET through ErrorEPC, a later stage's instruction
halting; the latency and pairing tables cell for cell; and the issue rules on sequences whose cycles follow from the
manual by hand (two independent ALU instructions 1 cycle, dependent 2; two loads 2; MULT to MFLO 5; DIV to MFLO 38;
MULT beside MULT1 1; MADD after MADD 3; a Y pair 2). The crate's other tests: lib **14 of 14**, `programs` **3 of 3**;
clippy clean.

### 16.6 On the referees (measured 2026-10-24)

`examples/s6.rs` writes each program as an ELF with its expected record; `ps2referee.py ram … --until 1FFFF0=5A99D00E`
read each record from PCSX2 (Americas BIOS, fast boot) and from Play!. Words differing from the manual's record:

| Program (record words) | PCSX2 | Play! | Reading |
|---|---|---|---|
| `three` (10,562), `immediates` (3,538), `muldiv` (14,080), `branches` (1,772) | 0 | 0 | both lineages equal the manual |
| `sa` (410) | 0 | 0 | after D-9's ruling; 150 each against the manual's bit count |
| `undefined` (184) | 0 | 0 | D-6: both lineages give the interpreter's reading |
| `memory` (748), `upper` (44) | 12, 1 | 12, 1 | LWR at a byte other than 0: PCSX2 clears bits 63..32, Play! sign-extends, the manual keeps them (D-10) |
| `exc-trap` (1,344), `exc-call` (40) | 0, 0 | 455, 12 | PCSX2 takes traps, SYSCALL and BREAK exactly; Play!'s high-level BIOS takes none through the vector |
| `exc-eidi` (4) | 1 | 0 | DI left EIE set on PCSX2 (D-11) |
| `exc-address` (96), `exc-overflow` (144), `exc-reserved` (70), `exc-unusable` (20), `exc-delay` (40), `exc-user` (30) | 47, 65, 21, 8, 16, 15 | the same | neither referee raises an address error, an overflow, a reserved instruction or an unusable coprocessor exception; those rest on the manual alone until S10's console texts |

**A defect of the project's ELF builder, found by these runs.** `media::program` placed the segment at byte 84 with an
alignment of 16, which the ELF format forbids (an offset must equal the address modulo the alignment). Small programs
loaded; a 160 KiB one read its constants 12 bytes astray on PCSX2 [M: a probe program of three cases equal at its own
size and wrong with 40,000 NOPs before its constants, then equal with the segment at byte 60h]. The segment now starts
at `SEGMENT_OFFSET`, 60h; WiseMan's `SyntheticPs2Elf` keeps the old layout (§16.9 Q3).

**Exposure**: the referees were run on the project's own programs only; nothing of either's log was read.

### 16.7 Seeded faults (2026-10-24)

| # | Fault | Caught by |
|---|---|---|
| F1 | MADD1 accumulating from pipeline 0's HI and LO | `both_multipliers_and_dividers_equal_the_manual` |
| F2 | LWR sign-extending at every byte (Play!'s reading) | `every_load_and_store_at_every_byte_equals_the_manual`, `bits_127_to_64_survive_every_narrower_write` |
| F3 | BNEL not nullifying its slot | `branches_jumps_and_links_equal_the_manual` |
| F4 | EPC the slot's address for an exception in a delay slot | `exceptions_record_cause_epc_badvaddr_and_status_as_chapter_4_says` |
| F5 | MAC0's repeat rate ignored | `the_multipliers_are_four_cycles_to_their_result_and_two_between_issues` |
| F6 | SLTIU comparing with the zero-extended immediate | `immediate_and_shift_forms_equal_the_manual` |

### 16.8 What the plan got wrong, or left open

- **§5.2 says "the issue and latency the TX79 manual gives"**: the EE Core User's Manual now gives them for the EE
  itself; its integer rows equal the TX79's, its floating-point rows do not (4 cycles for ADD.S against the TX79's 6,
  DIV.S 8 against 21), so S9 uses the EE's.
- **S6's gate, "the project's programs over every integer encoding against the manual's definitions"**, holds for the
  results; for exceptions the referees give no evidence beyond traps, SYSCALL and BREAK (§16.6), so the exception rules
  are graded against the manual alone until S10's console texts.
- **The manual is not consistent with itself in four places**, each a dispute: Trap's code (D-7), Status.CU's polarity
  (D-8), what SA holds (D-9), and MADD1's and MULTU's Descriptions against their Operation sections (§16.2).
- **S5's Q2** (fields fixed at zero are ignored) stands: no program here set one. The kit's *Restriction* (16) and (17)
  show that some undefined COP0 and COP1 patterns are not cleanly decoded by the silicon (no exception, a destroyed TLB
  entry or data cache line); the programs avoid those patterns and the decoder still reports them reserved.

### 16.9 Questions this stage leaves

1. **Q1 [routine], D-10 (LWR's upper word).** *Recommended*: keep the documents' reading (the EE Core manual's
   Description, Operation and table, and the TX79's, agree), since the two referee lineages disagree with each other;
   look again when S10 runs `cpu/ee/*`, whose console texts may exercise LWR.
2. **Q2 [routine], exceptions without a referee.** *Recommended*: accept the manual as the only witness for address
   errors, overflow, reserved instructions and unusable coprocessors until S10, and run `exc-*` again on any referee
   that later models them.
3. **Q3 [routine], WiseMan's `SyntheticPs2Elf`.** It writes the same misaligned segment offset (84 with an alignment of
   16). *Recommended*: give it the Rust builder's 60h at S10, when the C# runners first build programs large enough to
   meet the defect; no program it writes today is.

### 16.10 What was read for this stage

`Sapphire_Plan.md` §1, §2, §5, §5.12, §7, §9.1 and §11; this page's §1 and §14–§15; `Sapphire_Disputes.md`;
`Sapphire_References.md`; `EmuSen_Mips.md` §8; `emusen-mips`'s `r5900.rs`; `ps2referee.py`'s usage. Sony's *EE Core
Instruction Set Manual* §1, chapter 2's pages for every integer instruction, chapter 3's pages but the 128-bit MMI
ones, chapter 4's pages for DI, EI, ERET and MFC0, §7; *EE Core User's Manual* §1.3, §1.4, §2.2–§2.12, §3.2's Status,
Cause, EPC and PRId, chapter 4; *VU User's Manual*'s contents, §3.6.1, §4.1; the kit's *Restriction*; the title pages
of the other manuals. Toshiba's *TX79* manual, Table 3-21 and LWR's page. ps2tek's text searched for the divide and
PRId. No emulator, SDK or loader source and no firmware.

### 16.11 What this stage does not cover

- The machine running programs: the ELF loader and the machine's EE are S10's; the S6 programs run through the tests.
- COP0 beyond the exception registers, the TLB, the caches and CACHE, the scratchpad and the memory map (S7); MMI and
  PLZCW (S8); COP1 (S9); COP2 (S12); interrupts (S11).
- D-0's clocks: the scheduler that would pin them does not exist yet; the interpreter counts cycles alone.

## 17. Stage S7, COP0, the TLB, the caches, the memory map and the counters (2026-10-25)

### 17.1 In one paragraph

The plan's §7 row S7, on the tester's go of 2026-10-24. The interpreter of §16 gains COP0 in full: the TLB of 48 page
pairs with its seven page sizes, the ASID and G, and the S bit that maps the scratchpad; the 8 KiB data cache with
write-back, write-allocate, the LRF replacement and line locking, the 16 KiB instruction cache, the uncached
accelerated buffer, and the CACHE instruction's twenty operations; the EE's physical memory map with its bus
errors; Count and Compare with the timer interrupt; the two performance counters with every event the interpreter
can know and the overflow's level 2 exception; and the hardware breakpoints with the debug exception. Every access
now goes through the operating mode's segments, the TLB and the caches, and charges its refill, write-back or bus
transfer to the timing model, closing S6's named loss "no cache". The core is written from the *EE Core User's
Manual* (CORE) chapters 3 to 7 and 9, the *EE Core Instruction Set Manual* (INST) chapter 4, the *EE User's Manual*
§2.1 and the kit's *Restriction*; ps2tek where they are silent. Seven bare programs of the project's, each owning the
machine, are equal on the interpreter to the records the manuals' rules give (400 graded words). On PCSX2 and Play!, run as black boxes, they show where each referee stops modelling: neither maps memory through a program's own TLB entries, takes a TLB, counter, debug or bus error exception, or models the caches, so those rules rest on the manuals alone; where the referees do answer (TLBR, Count, Compare's interrupt, the counters' moves, kuseg at ERL) the disagreements are disputes D-17 to D-19. No
emulator, SDK or loader source and no firmware was read.

### 17.2 What was built

- **`ee/cop0.rs`**: the register numbers, the bits MTC0 may change in each (§3.2's r/w columns), Cause's and
  Config's fields, the debug registers behind register 24 and the performance counter control behind 25, and the
  events of Table 7-1 by the EVENT value each counter selects them with.
- **`ee/tlb.rs`**: the entries as Figure 5-8's 128 bits; TLBWI's, TLBR's and TLBP's Operation sections; the lookup.
- **`ee/cache.rs`**: the tags of §6.2.3, the data, the LRF rule, the UCAB, and the BTAC as storage.
- **`ee/memory.rs`**: translation, fetch, load, store (with the byte merge SWL, SWR, SDL and SDR need), PREF, the bus
  errors, CACHE, and `peek`, the bytes at a physical address as the core sees them.
- **`ee/map.rs`**: the physical map (§17.4).
- **`ee/mod.rs`**: the level 1 sequence extended to the TLB exceptions (BadVAddr, Context's BadVPN2, EntryHi's VPN2)
  and the refill and interrupt vectors; the level 2 sequence (§4.1.3); the interrupt, counter and breakpoint checks
  before each instruction; Count, Random and the counters advanced by the cycles each instruction takes.
- **`ee/ops.rs`**: TLBR, TLBWI, TLBWR, TLBP, the fourteen debug moves, MFPS, MFPC, MTPS, MTPC, BC0F, BC0T, BC0FL,
  BC0TL and CACHE (155 mnemonics in all); loads and stores through `memory`; SYNC.L disabling the UCAB.
- **`ee/timing.rs`**: an instruction's fetch wait and data wait, blocking or not by Config.NBE (§6.3.2), and single
  issue when Config.DIE is clear.

### 17.3 The readings the manuals leave open

[R CORE, INST, Version 5.0; A]:

- **The segments** (CORE §5.1.3 to §5.1.6): kseg0 and kseg1 subtract their bases; kseg0's cache mode is Config.K0,
  kseg1 is uncached; kuseg is unmapped and uncached while ERL is set; the rest goes through the TLB. User mode reaches
  kuseg alone, supervisor mode kuseg and sseg, and any other address raises the address error (§4.2.9), before the
  TLB is consulted (Table 4-2's order). K0 0, "cached without writeback and write allocate", is taken as its words
  say: loads cached, stores written through and not allocating; the TLB's C 0 is reserved and taken as uncached, as
  are the other reserved values.
- **The TLB** (§5.2, INST TLBWI, TLBR, TLBP): an entry matches where VPN2 equals the address above the page pair under
  its mask, with G or the ASID; the odd page is chosen by the bit above the page offset; several matches are undefined
  and the lowest index is taken. TLBWI and TLBWR store G as the AND of both G bits and VPN2 masked by PageMask, as
  the Operation sections write; TLBR returns G in both EntryLo registers. TLBP on a miss sets P and leaves the index
  bits, which the manual calls undefined, as they were. An Index or Random above 47 writes nothing.
- **An S entry** (§5.2.5, §6.4.1, §6.4.3): it covers the 16 KiB its address names above bit 14 ("the upper 18 bits of
  the virtual address are used to access the TLB"), whatever VPN2's bit 13 holds, with V and D taken from EntryLo0
  (the manual requires both halves equal); its PFN and C are ignored and TLBR reads C as 2 in both halves (§5.2.5's
  "When read, the C field value is 2"). Two S entries at different addresses reach the same scratchpad (§5.2.5: "It
  can also be allocated to SPRAM using another TLB entry"). A fetch from the scratchpad reads it; no document says.
- **The TLB exceptions** (§4.2.6 to §4.2.8): BadVAddr, Context's BadVPN2 (address bits 31..13 into bits 22..4) and
  EntryHi's VPN2 are written, EntryHi's ASID kept; a refill vectors to 80000000h unless EXL is set, when it takes the
  common vector and leaves EPC and BD (§4.1.2). The interrupt vectors to 80000200h. Every exception disables the UCAB
  ("Some exception", §6.5.1, read as every one).
- **The data cache** (§6.2, §6.3): indexed by virtual address bits 11..6, tagged by physical bits 31..12, so the index
  lies inside every page's offset; a miss writes a dirty victim back, then refills the whole line and flips the
  refilled way's LRF bit (Table 6-4, D-16); a locked way is never the victim, and a store to a locked line stays in
  the cache and does not set D (§6.3.7). ps2tek's figure of the data cache's address (an index at bits 12..6) contradicts
  its own text ("the dcache index is 6-bit") and the manual; the manual is followed.
- **The instruction cache**: indexed by virtual address bits 12..6, tagged by physical bits 31..12; a miss refills an
  invalid way if exactly one is invalid, else the LRF way (§6.3.1). Stores never reach it: replaced code runs until
  CACHE IHIN, IXIN or IXSTG invalidates its line (§6.3.6), which `s7-icache` shows.
- **The CACHE instruction** (INST pages 292 to 311): the index operations name the line by bits 11..6 or 12..6 and the
  way by bit 0 and translate nothing; the hit operations, DXSTG and IFL translate and may raise the TLB exceptions and
  the address error their pages list, as loads. DHIN and DHWBIN write Status.CH (§3.2 names those two); DHWOIN does not.
  DXSTG's Operation writes "tag.Dirty ← TagLO[6] & TagLO[6]", taken as TagLo[6] AND TagLo[5], since Table 6-3 says D
  is never set without V. IFL takes the LRF way, as its page says. IXSDT keeps TagHi's steering bits and BHT beside the
  word. The BTAC is storage CACHE reads and writes, indexed by address bits 5..0; BFH clears every V bit, BHINBT the
  entries whose fetch address matches bits 31..3, IHIN those matching bits 31..6, IXSDT and a write to EntryHi all of
  them. The other rt values do nothing, though the kit's *Restriction* (17b) says some destroy the data cache.
- **PREF** (INST, PREF): the line is read into the data cache where the page is cached and the address translates; no
  exception is taken and no wait charged ("It does not affect the meaning of the program"). The manual allows that
  the prefetch "may not take place"; it always does here.
- **The UCAB** (§6.5): one line of 128 bytes tagged by physical bits 31..7; an accelerated load that misses refills
  it; any store, SYNC.L and every exception disable it. Accelerated stores go to the bus at once (the write-back
  buffer's merging has no effect a program can see until a second bus master exists, S11).
- **Bus errors** (EE User's Manual §2.1.2; CORE §3.2, BEM and BadPAddr): an access to 20000000h–3FFFFFFFh or 80000000h
  and above raises the instruction or data bus error (codes 6 and 7) and writes BadPAddr's bits 31..4 and sets BEM;
  with BEM set the access is dropped silently, a load reading zero. BadVAddr is left alone ("bus errors are not
  addressing errors"). A refill's bus error is taken before the victim is written back.
- **Count and Compare** (§3.2): Count advances by the cycles each instruction takes in the timing model, so a read
  sees the cycles of the instructions before it; IP7 is set when Count passes Compare and cleared by writing Compare.
- **Random** (§3.2): decremented once a cycle in which an instruction issues, from 47 down to Wired and round again;
  writing Wired sets it to 47, and that write is the cycle's last word.
- **The interrupt** (§3.2, §4.2.5): taken before an instruction when IE and EIE are set, EXL and ERL clear, and IP7, IP3
  or IP2 is set with its mask bit; IP2 and IP3 follow the lines Int0 and Int1 (D-15).
- **The performance counters** (chapter 7): MFPS reads PCCR whatever its reg field; MTPS writes it only for reg 0; an
  even reg names PCR0 and an odd one PCR1, as ps2autotests' console record says (`Sapphire_References.md` §5.3). A
  counter adds its events when CTE is set and the instruction's mode (U, S, K outside the handler, EXL) has its bit,
  never at level 2 (§7.2.1); VALUE is 31 bits and its carry sets OVFL. The events: processor cycles; single and dual
  issue, counted when an issue cycle closes; branches issued (conditional branches, J and JAL, BC0 to BC2, likely or
  not) and those at an even word address; TLB misses (refills); instruction cache misses; data cache misses (a cached
  miss, a cached load with DCE clear, an uncached load, an accelerated load that misses); DTLB accesses, read as the
  loads and stores to a cached page; non-blocking loads (a load's miss, an uncached load, a UCAB miss); instructions
  completed and non-BDS instructions completed (SYSCALL, BREAK and a taken trap complete; a nullified slot completes
  as the kind it is); loads and stores completed. The overflow's exception is D-13's.
- **The breakpoints** (chapter 9): judged on the virtual address, the mode by Figure 9-2 (none at level 2); an
  instruction breakpoint before the instruction and before its fetch's own exceptions (Table 4-2); a data breakpoint
  on the effective address before alignment is checked; a data value breakpoint compares the register's low 32 bits
  for a store and the loaded register's low 32 bits for a load (§9.2.4's four cases), and a load that meets it is
  cancelled, its register restored. BED's polarity is D-14, EXC2 D-12.
- **COP0's other registers**: Config's DIE, ICE, DCE, NBE, BPE and K0 writable, IC and DC read 16 KiB and 8 KiB, EC
  zero; BadPAddr's bits 31..4 writable; the reserved numbers 7, 17 to 22, 26, 27 and 31 read zero and ignore writes,
  the manual calling them undefined. MTC0 to EntryHi flushes the BTAC (INST, MTC0's note).

### 17.4 The physical memory map

[R EE User's Manual §2.1.1, its Figure, §2.1.2; ps2tek "Memory Map"]: 00000000h, 32 MiB of RDRAM; to 0FFFFFFFh the
main memory's window past what a retail console fits (read as zero); 10000000h–13FFFFFFh the EE's registers, the VU
memories and the GS's registers, which S11 to S16 fill and which read as zero now; to 1FBFFFFFh reserved; 1FC00000h
the 4 MiB boot ROM, empty at this stage (no firmware is read); 20000000h–3FFFFFFFh and 80000000h and above, bus
errors; 40000000h–7FFFFFFFh "Extend Main Memory", absent (zero). ps2tek's virtual map (RAM at 00000000h cached,
20000000h uncached, 30100000h accelerated, the scratchpad at 70000000h) is the TLB a kernel sets, not hardware, and
is §17.7's. The map is `ee::map::Map`; tests and the interpreter use it or RDRAM alone (`bus::Ram`).

### 17.5 The timing model's memory costs

[R CORE §6.3.3; A]: a line's refill costs 8 cycles, "a burst of 4 bus cycles (8 CPU cycles)", which the manual
itself calls a minimum ("Actual transfer time can be more due to bus arbitration"); a dirty victim's write-back the
same burst before it; an uncached transfer one bus cycle, 2 cycles [A]; the UCAB's refill of 128 bytes eight bus
cycles, 16 [A]; the scratchpad and a hit cost nothing beyond §2.12's table. A fetch that waits delays its
instruction's issue and ends the pair. A data wait delays the result; with Config.NBE clear (blocking loads, the
reset state) it also stalls the pipe, and with NBE set a second miss waits for the first (§6.3.2). S6's named loss
"no cache, so every load is §2.12's hit" is closed. The S6 tests of the issue rules now run their code once to warm
both caches first, as §2.12's "Load and store assume a cache hit" says.

### 17.6 What is not modelled

Each a dispute candidate if an oracle ever measures it [A]:

- RDRAM's latency and page behaviour (§5.2's loss, still named): every miss costs the documented minimum burst.
  ps2tek's "~40 EE cycles is closer to reality" is not a documented figure.
- The ITLB (2 entries) and DTLB (4): their replacement is undocumented, so their misses cost nothing and their
  counters (events 5 on PCR0 and PCR1) never count.
- The BTAC, the BHT and branch prediction: no misprediction cost and no counting of BTAC misses or mispredictions.
- The write-back buffer: no merging, no stall when full; its four counter events never count; the bus-busy events
  (11) neither.
- The counter exception's lateness (§7.2.7, §7.2.8) and the data value breakpoint's imprecision for loads, SWC1 and
  SQC2 (§9.1): both taken exactly at their instruction.
- Interrupts are taken before an instruction ahead of its fetch's exceptions, which Table 4-2 orders the other way;
  the manual itself says that order "has little meaning".
- CACHE operations, TLB writes and exception entries cost one cycle beyond any refill or write-back they do; the
  manual tabulates none.
- The kit's *Restriction* (2), (13), (14), (16) and (17): code and data that alias CACHE patterns, IFL on a cached
  address, TLBR near a branch or a page end, and the undefined COP0 patterns that destroy TLB entries are not
  modelled; programs that break them are outside the documents.

### 17.7 The state a program is entered in

`Ee::new(pc)` is the core as the project's programs and the referees' fast boot enter a program, the state a
kernel's loader leaves, not a reset: kernel mode, BEV clear, interrupts off; Config with both caches, dual issue,
non-blocking loads and branch prediction on and kseg0 cached (K0 3); the caches empty; and the TLB holding ps2tek's
virtual map as wired entries 0 to 18 (Wired 19): a 16 MiB pair at 00000000h onto RDRAM cached, the same at
20000000h uncached, sixteen 1 MiB pairs from 30000000h giving 30100000h–31FFFFFFh onto 00100000h–01FFFFFFh
accelerated, and an S entry at 70000000h. The page sizes and the indices are chosen here [A]; what a program can see
of them is the map ps2tek lists. `Ee::reset()` is §4.2.1's state: PC BFC00000h, ERL and BEV set, BEM clear, caches off
and invalid, Random 47, Wired 0, CTE and BPC's enables clear, K0 2 (the manual leaves it undefined). The S6 programs
run in `new`'s state unchanged and are graded through `Ee::peek`, which reads a valid data cache line over memory, so
a record still in the cache counts as written, as on a console.

### 17.8 The gate: the programs (measured 2026-10-25)

`tests/s7/` generates seven bare programs as `emusen-mips` assembly, entered at 00100000h like S6's. Each owns the
machine first: it writes handlers at the five vectors through kseg1 (refill at 80000000h, counter at 80000080h, debug
at 80000100h, common at 80000180h, interrupt at 80000200h), turns both caches on, writes back every data cache line
and clears its tag (CACHE DXWBIN, DXSTG), invalidates every instruction cache line (IXIN, with addresses below
80000000h as the kit's *Restriction* (12) asks), and clears TLB entries 0 to 7; it then runs from kseg0 and reads its
constants and writes its record and done word through kseg1, so that its record is in RAM and its own data never
enters the data cache it is testing. The refill handler records Cause, EPC, BadVAddr, Context and EntryHi and maps the
faulting pair onto a scratch page in entry 7, then retries; the common handler records the same and returns past the
instruction. Every expected word is derived in `tests/s7/mod.rs` from the manuals' rules, the LRF way of each refill
worked by hand from Table 6-4 beside its line.

| Program | Graded words | What it covers |
|---|---|---|
| `s7-tlb` | 176 | refills on loads and stores, even and odd pages, a pair already mapped; every page size's even page, odd page and last word; past a 16 MiB pair; TLBR's masked VPN2 and ANDed G; TLBP's hit and P; the ASID and G; TLB Invalid on a load and a store, TLB Modified; a refill at EXL to the common vector with EPC kept; kuseg unmapped at ERL; ksseg mapped |
| `s7-spr` | 44 | the scratchpad through two S entries at once, at bit 13's half and the last quadword, LQ and SQ; RAM under the ignored PFN untouched; TLBR's C of 2; a clean S entry refusing a store, an invalid one a load |
| `s7-dcache` | 68 | a store kept from memory until written back; DHWOIN; DHIN on a hit and a miss and Status.CH; dirty data discarded by DHIN; three lines of one set replaced by the LRF rule with the dirty victim written back; DXWBIN by index; DXLTG's tags after each; a locked line, its store kept from memory and from D, refills taking the other way; unlocking and DHWBIN; write-allocate; accelerated pages |
| `s7-icache` | 10 | a routine replaced in memory runs as cached until IHIN, replaced again until IXIN of both ways |
| `s7-counters` | 36 | Count across 100 instructions; Compare's interrupt, IP7 and its acknowledgement; MTPS's writable bits and its reg field, MFPC's and MTPC's even and odd specifiers; instructions completed, loads completed and stores completed by difference over two bodies; the EXL and K mode bits; the overflow's level 2 exception |
| `s7-debug` | 40 | an instruction breakpoint taken (EXC2, ErrorEPC, BPC's flag), suppressed by BED, filtered by mode; a data read breakpoint cancelling its load; a data value breakpoint passing a store whose value differs and stopping one that matches |
| `s7-buserror` | 26 | a load and a store mapped past 20000000h: ExcCode 7, BadPAddr, BEM set; a masked load passing |

**Tests** [M, `cargo test --release`]: `tests/cop0.rs` **19 of 19** — the seven programs, each equal to its record word
for word; `the_programs_execute_every_instruction_s7_adds` (the 27 new mnemonics, those no program reaches named:
TLBWR, the debug moves for IAB to DVBM that the programs only write, BC0, each with its own test); and, on the
interpreter directly, the reset state, Random's range and its reload by Wired, TLBWR at Random, BC0 on CPCOND0 and
BC0FL's nullified slot, the debug moves' fixed bits, the CACHE index operations' round trips (IXSDT and IXLDT, IXSTG
and IXLTG, BXSBT and BXLBT, BFH), an instruction bus error, the physical map's regions, an interrupt held by its masks
and taken at 80000200h with Int0 on IP2, the level 2 sequence with DEV, and a miss's refill and a dirty victim's
write-back in cycles. `tests/interpreter.rs` (S6) **23 of 23** unchanged in what it grades: its programs run in §17.7's
state and are graded through `peek`; its issue-rule tests run their code warm. The crate's other tests: lib
**16 of 16**, `programs` **3 of 3**; clippy clean.

### 17.9 On the referees (measured 2026-10-25)

`examples/s7.rs` writes each program as an ELF with its expected record; `ps2referee.py ram … --until
1FFFF0=5A99D00E 200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and from Play! 0.72, one
process at a time. Only the programs' records were read; nothing of either's log.

**A finding about the programs' form, by bisection.** The first runs on PCSX2 never wrote their done word. Ten probe
programs (`s7::probe`, `cargo run --example s7 -- <folder> probes`), each the prologue's first *n* parts or its TLB clearing
over a range of entries, and one record word, located it: the handlers, the caches' loops and clearing TLB entries 0 to 7 run, and clearing entries 8 to 47
stops PCSX2, with the program running from kseg0 and writing through kseg1 and whether the cleared entries' VPN2s lie
in kseg0 or kseg3 [M]. On the EE nothing the program does afterwards depends on those entries [R CORE §5.1.6], so the
cause is the referee's (its kernel's mappings are presumably kept there). The programs therefore own entries 0 to 7
only, leave Wired as they find it, and the refill handler writes entry 7 by TLBWI; TLBWR is graded on the interpreter.

**What each referee showed** [M, records aligned by hand where a missing exception shortens them]:

| Rule | PCSX2 | Play! | Reading |
|---|---|---|---|
| TLB refill, invalid and modified exceptions | none taken; the load reads 0, the store is lost | none taken | the manual alone |
| Accesses through the program's own entries (every page size, the ASID, the scratchpad's S entries) | read 0, stores lost | translated as if the address were masked (a load at 40000000h read the refill handler's first word at physical 0); only 16 MiB pages reached their frames | the manual alone |
| TLBR's masked VPN2, ANDed G, PageMask | equal to the manual | another entry's words | PCSX2 a witness for TLBWI and TLBR |
| TLBP | P set where the manual matches | — | the manual alone |
| TLBR of an S entry | S dropped, C as written | — | D-19 |
| kuseg unmapped at ERL | equal | equal | both lineages agree |
| The data cache (write-back, CACHE, the tags) | not modelled; DXLTG leaves TagLo as written | not modelled; same | the manual alone |
| The instruction cache | the replaced code runs at once | the program never wrote its done word | the manual alone |
| Count across 100 instructions | within 50 to 1,000 cycles | under 50 | the timing model's; D-0's clocks |
| Compare's interrupt: IP7, ExcCode 0 | equal | never taken | PCSX2 a witness |
| Writing Compare clears IP7 | IP7 stays set | — | D-17 |
| PCCR's writable bits | every bit kept | equal to the manual | D-18 |
| MTPS with reg ≠ 0; MFPC and MTPC by the field's parity | equal | equal | both lineages agree with ps2autotests' console record |
| Events by EVENT and mode | the same count whatever is selected | never count | the manual alone |
| The counter, debug and bus error exceptions | none taken | none taken | the manual alone (D-12, D-13, D-14) |

As at S6 (§16.6), neither referee raises the exceptions an emulator need not raise to run games; S10's console texts
(`cpu/ee_cop0/performance`, `memory/ee/default`) are the next evidence for the counters and the memory map.

### 17.10 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | a TLB entry's page mask read one bit low (the page offset `mask \| 0xFFF` for `mask >> 1 \| 0xFFF`) | `tlb_refills_pages_and_their_exceptions_equal_the_manual` |
| F2 | a store hitting a cached line leaving its dirty bit clear | `the_data_cache_writes_back_replaces_and_locks_as_chapter_6_says`, `a_miss_costs_its_refill_and_a_dirty_victim_its_write_back` |
| F3 | Compare's match never setting IP7 | `count_compare_and_the_performance_counters_equal_chapter_7`, `an_interrupt_waits_for_ie_eie_and_im7_and_vectors_to_200h` |

Each was applied, `tests/cop0.rs` and `tests/interpreter.rs` run, and the source restored [M]. A second fault found
while building, not seeded: the counters' mode bits were first indexed in the wrong order (U, S, K, EXL against
PCCR's EXL, K, S, U at bits 1 to 4), which `s7-counters`' EXL and K differences exposed before the first commit.

### 17.11 What the plan got wrong, or left open

- **§1.1's COP0 row puts "the debug registers" beside the counters**, and S7's row omits the breakpoints; they were
  built here, with the debug exception, since chapter 9 describes them fully and COP0 "in full" includes them.
- **§5.2's "a cache miss its refill from RDRAM at the documented cost"**: the only documented cost is the minimum
  burst (§17.5); RDRAM's latency stays the named loss §5.2 already lists.
- **§13's C2 (caches' coherency without their timing)** is measurable now: the tags are on every access; S32 measures
  their cost against the bench.
- **S7's gate assumed the referees would answer for some of the TLB and the caches**; neither does (§17.9), so the gate
  rests on the manuals, as for S6's exceptions. The programs had to be reshaped to run on PCSX2 at all (§17.9).
- **The memory map** of §1.1 lists ps2tek's virtual addresses beside the physical ones; the virtual ones are a kernel's
  TLB, not hardware, and are kept as §17.7's entry state.

### 17.12 Questions this stage leaves

1. **Q1 [routine], `Ee::new`'s TLB.** The project's bare programs are entered with ps2tek's virtual map in entries 0
   to 18 (§17.7), a kernel's choice standing in until S10 enters programs through the player's BIOS. *Recommended*:
   keep it for the bare programs, and have S10 enter ps2autotests through the BIOS's own loader, so that no kernel's
   mappings are assumed by the core itself.
2. **Q2 [routine], the timing model's memory costs.** Refills cost the documented minimum of 8 cycles, uncached
   transfers 2 and the UCAB's refill 16 [A] (§17.5). *Recommended*: keep them as named losses until S32's bench or a
   counting program on the referees gives evidence; ps2tek's "~40 EE cycles" is not documented.
3. **Q3 [routine], the referees' reach.** Neither models the TLB's mapping of a program's entries, the caches, the
   counter events, breakpoints or bus errors (§17.9). *Recommended*: rest those rules on the manuals, as S6 did for
   its exceptions, and look again at S10 with `cpu/ee_cop0/performance` and `memory/ee/default`'s console texts.
4. **Q4 [routine], D-17 to D-19.** *Recommended*: keep the manual's readings (Compare's write clears IP7; PCCR keeps
   its defined fields, as Play! agrees; TLBR returns S with C 2), each against one lineage.

### 17.13 What was read for this stage

`Sapphire_Plan.md` §2, §5 (§5.1 to §5.3, §5.12, §5.13's C1 and C2), §7, §11; this page's §16; `Sapphire_Disputes.md`;
`Sapphire_References.md` §1.1 and §5.3; `emusen-mips`'s `r5900.rs` (COP0's tables and the assembler's forms);
`ps2referee.py`'s usage. Sony's *EE Core User's Manual* §1.3.5, §1.3.6, §1.5 to §1.9, §2.12, §3.2 whole, chapter 4,
chapter 5, chapter 6, chapter 7 and chapter 9; *EE Core Instruction Set Manual* chapter 4 whole and PREF's page; *EE
User's Manual* §2.1 and §2.2's opening; *EE Overview* §2.2.2; the kit's *Restriction* whole. ps2tek at `1c91660`:
"Memory Map", "EE COP0 Registers", "EE COP0 Exception Handling", "EE COP0 Memory Management", "EE COP0 Caches", "EE
COP0 Timer". ps2autotests' `cpu/ee_cop0/performance.expected` (the console's text, not the program's source). No
emulator, SDK or loader source and no firmware.

### 17.14 What this stage does not cover

- The machine running programs: the EE is still run by the tests; S10 loads programs.
- The DMAC's CPCOND0 and the INTC's and DMAC's interrupt lines, which the core takes as inputs (S11); the scratchpad's
  DMA (S11); the VU memories, the EE's and GS's registers behind the physical map (S11 to S16).
- MMI (S8), COP1 (S9), COP2 (S12): still halting.
- The ITLB, DTLB, BTAC prediction and the write-back buffer's timing (§17.6).
- The state format's new contents (the TLB, the caches, the counters): `SPH2` gains them when the machine runs the EE
  (S10, S30).

## 18. Stage S8, the multimedia instructions (2026-10-25)

### 18.1 In one paragraph

The plan's §7 row S8, on the tester's go of 2026-10-25. The interpreter of §16 and §17 gains the EE core's multimedia
instructions: the 91 entries of the MMI space S6 did not already execute (MMI0 to MMI3, PMFHL's five forms and PMTHL.LW,
the six parallel shifts by sa, PLZCW and QFSRV), each decoded by `emusen-mips`'s `r5900` table alone and executed on the
128-bit GPRs, HI and LO as chapter 3 of the *EE Core Instruction Set Manual* (INST) writes it, and charged to the timing
model through the *EE Core User's Manual*'s (CORE) Wide Operate category and its multimedia multiply and divide rows.
Nine bare programs of the project's, 3,941 cases and 20,712 record words over every encoding with operands chosen for
the edges, run on the interpreter equal to the manual's record, transcribed apart from the interpreter. On PCSX2 the
nine are equal word for word; Play! does not execute eight of the instructions and clamps one otherwise. ps2autotests'
six `cpu/ee_simd` programs, whose expected texts are consoles' outputs, run on PCSX2 equal to those texts line for line,
so the PCSX2 lineage is a strong witness on the MMI; those texts decided where the manual leaves a result undefined
(D-21, D-22). Disputes D-20 to D-23 are opened and D-9 closed. No emulator, SDK or loader source, no ps2autotests source
and no firmware was read.

### 18.2 What was built, and the readings the manual leaves open

- **`ee/mmi.rs`**: one function a mnemonic, looked up by the decoder's own name as `ops.rs`'s are (the two tables are one
  index space, S6's and S7's entries first), on helpers that read and build a 128-bit value lane by lane. `Ee::set_quad`
  is the 128-bit write the multimedia instructions and LQ make.
- **`ee/timing.rs`**: the rows "Multimedia Multiply" and "Multimedia Divide" (§2.12, pinned since S6) now occupy both
  multipliers (§18.4).

**Readings** [R INST chapter 3, Version 5.0; A]:

- **Saturation and clamping** follow the Descriptions' signed or unsigned ranges; the Operation sections' comparisons of
  bit strings with hexadecimal bounds cannot be read literally (D-23).
- **PADDH's Operation section** lists seven of its eight halfwords, the eighth line missing; the Description ("eight
  16-bit integers") and the figure give the eighth, and every lineage agrees.
- **PDIVBW's remainder** is sign-extended, the Operation section and the figure against the Description (D-20); its
  divisor is GPR[rt]15..0 read as signed.
- **PHMADH's and PHMSBH's undefined words** of HI and LO hold the odd product beside each sum, complemented for PHMSBH,
  as the console's record shows (D-21).
- **Undefined inputs** (a zero divisor, operands that are not word values, PSRLH and PSRAH by 16 or more) take D-22's
  readings: D-6's rule word by word, the words 31..0 and 95..64, sa3..0.
- **PLZCW** writes GPR[rd]63..0 alone, as its Operation section does, leaving bits 127..64; the count includes bit 31
  itself, the page's example (000FFF0FFF0FF00Fh gives 0000000B00000007h) confirming it.
- **QFSRV** shifts GPR[rs] || GPR[rt] right by 8 × SA bits, SA holding the byte count (D-9); SA = 0 gives GPR[rt].
- **PMTHL.LW** leaves HI's and LO's words 1 and 3, as its Operation section and figure say ("not changed").
- **The multiply-add forms' accumulators** are read from HI and LO before the instruction, lane by lane; the halfword
  forms take product *i* to word (*i* mod 2) + 2⌊*i*/4⌋ of LO when ⌊*i*/2⌋ is even and of HI when odd, the even
  products' words also to GPR[rd], as the pages' assignments list them.

### 18.3 The documents' answer to S6's question on SA

D-9 left QFSRV's use of SA to this stage. QFSRV after every count MTSAB and MTSAH can set is equal on both lineages
with SA as the byte count, and PCSX2 reproduces the `funnel` console text whole; the entry is closed (D-9's addendum).

### 18.4 The timing model (§5.2, Q14)

[R CORE §1.3.5, §1.3.6, §2.12; A]:

1. **The categories.** Table 1-1 places "128-bit multimedia instructions" in Wide Operate, which issues to logical pipe 0
   alone (Table 1-2) and uses I0 and I1 both; PLZCW in LZC ("Leading Zero Count", pipe 1). PMFHI, PMFLO, PMTHI, PMTLO,
   PMFHL and PMTHL are taken as Wide Operate with §2.12's "Transfer to HI / LO registers" row, Table 1-1's MAC0 and MAC1
   rows ("Move to/from HI/LO") being read as the 64-bit moves [A]; QFSRV as Wide Operate reading SA, Table 1-1's SA
   Operate being "Move to/from SA register" [A].
2. **The rows.** The parallel arithmetic, logic, compare, shift and rearranging instructions take the "Add/Sub, and logical
   operations" row (latency 1); the ten multiplies "Multimedia Multiply" (latency 4 to GPR[rd], HI and LO, a repeat rate
   of 2) and the three divides "Multimedia Divide" (37, 37), both on "I0+I1": each occupies MAC0 and MAC1, so a MULT1
   delays a PMULTW issued after it, and a DIV1 waits for PDIVW's divider.
3. **Pairing** is S6's rule on Table 1-3: a Wide Operate instruction pairs with a load or store (O), and with an ALU or
   LZC instruction in pipe 1 with the one-cycle stall of its Y cell; two Wide Operate instructions never pair.

**Named losses** [A], each a dispute candidate if an oracle ever measures it: PLZCW's and QFSRV's latencies, which §2.12
does not tabulate, taken as 1; the multimedia multiply-adds' accumulators forwarded at the repeat rate, as S6's MADD
(§16.2), not waited for; whether the Wide Operate instructions that use only one physical pipe's worth of work (the
logic, for instance) hold I1 as well is not stated beyond Table 1-1's mark, which is followed for all of them.

### 18.5 The gate: the programs (measured 2026-10-25)

`tests/s8/` generates nine programs in S6's form (`tests/s6/mod.rs`'s `Program`, which gains a quadword record), entered
at 00100000h and writing their record from 00200000h with the done word at 001FFFF0h. Each case loads GPR[rs] and
GPR[rt] by LQ from constants, the destination from a sentinel quadword, and HI and LO by S6's moves (MTHI, MTHI1, MTLO,
MTLO1), runs the instruction, and records GPR[rd] by SQ and HI and LO by MFHI, MFHI1, MFLO and MFLO1 where it touches them.
The expected record comes from `tests/s8/manual.rs`, the pages' Operation sections transcribed field by field in the
pseudocode's vocabulary with 128-bit integer arithmetic, written apart from `ee/mmi.rs`. **The operands**: for each lane
width a list of edges (bytes 00h, 01h, 7Fh, 80h, FFh, 81h …; halfwords 0000h, 0001h, 7FFFh, 8000h, FFFFh, 8001h …;
words 7FFFFFFFh, 80000000h, FFFFFFFFh, 1 …) set against every rotation of itself, so that every pair of edges meets in
some lane, and five plain values (zero, all ones, a counting pattern, alternate bits) against one another.

| Program | Cases | Record words | What it covers |
|---|---|---|---|
| `s8-arith` | 755 | 3,020 | the wrapping, signed-saturating and unsigned-saturating adds and subtracts at every width; PADSBH |
| `s8-compare` | 472 | 1,888 | PCEQ and PCGT at every width, PMAX, PMIN, PABS at their limits |
| `s8-lanes` | 1,458 | 5,832 | the logic, the six extends, the three packs, PINTH, PINTEH, PCPYLD, PCPYUD; the nine rearrangements, PEXT5 and PPAC5 |
| `s8-shift` | 255 | 1,020 | the halfword shifts by sa 0 to 15 (PSLLH's by 16 to 31 too, its page defining them by sa3..0), the word shifts by 0, 1, 15, 16, 31, the variable forms by amounts with bits above 4 set and words 1 and 3 ignored |
| `s8-hilo` | 279 | 1,224 | PMFHI, PMFLO and PMFHL's five forms over HI and LO at every clamp's edge; PMTHI, PMTLO, PMTHL.LW read back in full |
| `s8-multiply` | 450 | 5,400 | the five halfword multiplies and the five word multiplies with GPR[rd], HI and LO; PHMADH's and PHMSBH's defined words alone |
| `s8-divide` | 105 | 840 | PDIVW and PDIVUW on word values with no zero divisor, minimum by −1 included; PDIVBW by seven divisors with bits above them |
| `s8-misc` | 49 | 200 | PLZCW over runs of every length class and the page's example; QFSRV at every MTSAB and MTSAH count |
| `s8-undefined` | 118 | 1,288 | D-21's and D-22's inputs, with the interpreter's reading as the record |

**Tests** [M, `cargo test --release`, `tests/mmi.rs`]: **13 of 13** — the nine programs, each equal to its record word
for word; `the_programs_execute_every_multimedia_encoding` (all 91, counted from the assembled words through the
decoder, and the interpreter's set equal to the decoder's MMI space less S6's entries); and the timing rows on
sequences whose cycles follow from the manual by hand (two Wide Operate instructions 2 cycles, beside a load 1, a Y pair
costing the next instruction a cycle; PMULTH to PMFHI 5 and to a reader of GPR[rd] 5, PMULTH after PMULTH 3, PMULTW
after MULT1 3, PMADDH after PMADDH 3; PDIVW to PMFLO 38, DIV1 after PDIVW 75, a load and an ALU instruction beside
PDIVBW 2). The crate's other tests: `interpreter` (S6) **23 of 23**, its check that a later stage's instruction halts
moved from PADDW to QMFC2; `cop0` (S7) **19 of 19**, its count of S7's instructions bounded to S7's 27; lib **16 of 16**;
`programs` **3 of 3**; `mmi` and `interpreter` also under the `test-checked` profile (overflow checks); clippy clean.
WiseMan's `SapphireTests` **5 of 5**.

### 18.6 On the referees (measured 2026-10-25)

`examples/s8.rs` writes each program as an ELF with its expected record and its source; `ps2referee.py ram … --until
1FFFF0=5A99D00E 200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and from Play! 0.72, one
process at a time. Only the programs' records were read.

| Program (record words) | PCSX2 | Play! | Reading |
|---|---|---|---|
| `s8-compare`, `s8-lanes`, `s8-shift`, `s8-misc` | 0 | 0 | both lineages equal the manual |
| `s8-arith` (3,020) | 0 | 164 | Play! does not execute PADSBH (GPR[rd] keeps the sentinel) |
| `s8-divide` (840) | 0 | 248 | Play! does not execute PDIVUW (HI and LO keep their presets) |
| `s8-hilo` (1,224) | 0 | 202 | Play! does not execute PMFHL.SLW or PMTHL.LW, and clamps PMFHL.SH otherwise (D-23) |
| `s8-multiply` (5,400) | 0 | 1,808 | Play! does not execute PMSUBH, PMADDW, PMADDUW or PMSUBW |
| `s8-undefined` (1,288) | 0 | 478 | after D-21 and D-22's rulings; Play! as above, and zero in PHMADH's and PHMSBH's undefined words |

The first run of `s8-undefined` differed on both lineages in the same 13 words, PDIVBW by zero, where the interpreter
then sign-extended the dividend's low halfword into HI: both give the whole dividend, as the console's record does, and
the interpreter was changed to it (D-22). PHMADH's and PHMSBH's undefined words, first left as they were, differed on
both, each differently (D-21).

### 18.7 ps2autotests' `cpu/ee_simd`, on PCSX2 (measured 2026-10-25)

The six programs need the BIOS's kernel, so Sapphire runs them at S10. Run now on PCSX2 through `ps2referee.py tty
pcsx2 … --channel iop` (Americas BIOS), each program's text is equal to its expected file after line ends are
normalised: `arithmetic` 744 lines, `compare` 182, `funnel` 46, `logic` 400, `muldiv` 770, `shuffle` 626, 2,768 lines
with no difference [M `diff`]. Those texts were recorded on consoles [R the README], so **the PCSX2 lineage is exact
on every multimedia instruction the six print**, and the MMI disputes lean on it where Play! disagrees. Play! was not
run: `ps2referee.py`'s `tty` path drives PCSX2 alone, and Play! does not execute eight of the instructions the texts
cover (§18.6). Only the expected texts and the programs' own output were read, never `.cpp` or `shared.h`.

### 18.8 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | PADDSH wrapping instead of saturating | `parallel_add_and_subtract_wrap_and_saturate_as_the_manual_says` |
| F2 | PSRLH shifting 32-bit lanes | `parallel_shifts_take_each_lane_widths_amount_as_the_manual_says`, `the_undefined_multimedia_inputs_give_the_interpreters_recorded_reading` |
| F3 | PMADDW accumulating doubleword 1 from HI's and LO's word 0, the wrong half | `parallel_multiplies_write_gpr_rd_hi_and_lo_as_the_manual_says`, `the_undefined_multimedia_inputs_give_the_interpreters_recorded_reading` |

Each was applied, `tests/mmi.rs` run, and the source restored [M].

### 18.9 What the plan got wrong, or left open

- **§3.2's MMI row** names "the TX79 manual's definitions" as the secondary oracle; the EE Core Instruction Set
  Manual's chapter 3 now gives the EE's own, and the TX79 manual was not needed.
- **S8's gate, "`ee_simd/*` waits on S10"**, holds for Sapphire running them; but their console texts serve now, through
  PCSX2, as the evidence that weighs the referees (§18.7) and decided D-21 and D-22.
- **§5.2's table** did not say which category PMFHI and its kin and QFSRV fall in; §18.4 records the reading.
- **Play! as the second lineage on the EE** is weaker on the MMI than on the integer set: eight instructions are not
  executed (§18.6, Q2).

### 18.10 Questions this stage leaves

1. **Q1 [routine], D-21.** The undefined words of PHMADH and PHMSBH are taken from the console's record and PCSX2, Play!
   writing zero. *Recommended*: keep the console's reading; S10 runs `muldiv` on Sapphire against the same text.
2. **Q2 [routine], the witnesses for the MMI.** *Recommended*: weigh PCSX2 as the strongest single witness on the
   multimedia instructions, since it reproduces all six `ee_simd` console texts, and keep Play! as the second lineage
   for the units it does model; record its eight unexecuted instructions in `Sapphire_Referees.md`'s table at the next
   stage that edits it.
3. **Q3 [routine], `ee_simd` on Play!.** *Recommended*: do not extend `ps2referee.py`'s `tty` path to Play! for this;
   Sapphire runs the six against the console texts itself at S10.
4. **Q4 [routine], the timing categories of §18.4.** *Recommended*: keep PMFHI and its kin and QFSRV in Wide Operate
   as named readings; no oracle measures the EE's pairing before S32's bench.

### 18.11 What was read for this stage

`Sapphire_Plan.md` §2, §3.2, §5 (§5.1 to §5.3), §7 and §11; this page's §16 and §17; `Sapphire_Disputes.md`;
`Sapphire_References.md` §1.1; `emusen-mips`'s `r5900.rs` (the MMI tables and the assembler's forms); `ps2referee.py`'s
usage; S7's referee script. Sony's *EE Core Instruction Set Manual* chapter 3's opening and every multimedia page, DIV1's,
PLZCW's and QFSRV's (pages 137 to 284); *EE Core User's Manual* §1.3.5, §1.3.6, §1.4, §2.12, §3.1. ps2autotests'
`cpu/ee_simd/*.expected` (the consoles' text). No emulator, SDK or loader source, no ps2autotests source, no ps2tek
section and no firmware.

### 18.12 What this stage does not cover

- The machine running programs and `ee_simd` on Sapphire: S10.
- COP1 (S9) and COP2 (S12): still halting.
- The multimedia instructions in the recompiler: S38, graded against this interpreter.

## 19. Stage S9, COP1, the FPU, in software floats (2026-10-25)

### 19.1 In one paragraph

The plan's §7 row S9, on the tester's go of 2026-10-25. The interpreter of §16 to §18 gains COP1: the 34 instructions of
the *EE Core Instruction Set Manual*'s (INST) chapter 5, each decoded by `emusen-mips`'s `r5900` table alone and executed
as its page writes it, with the 32 FPRs, the accumulator and FCR31's condition, cause and sticky flags, and charged to
the timing model through the *EE Core User's Manual*'s (CORE) floating-point rows. Their arithmetic is one module,
**`sapphire::float`** (`src/float.rs`), the PlayStation 2's single-precision format on integers, the same bits on every
host, which the vector units will use from S12: there is one owner of the format. Where the manuals stop, at the last
bit that ps2tek calls "not well understood", ps2autotests' `cpu/ee_fpu` console texts, recorded on consoles, decide:
the adder keeps no sticky bit, the multiplier recodes its second operand as Booth digits and loses its lowest column,
so that its product is not commutative, and DIV.S and SQRT.S round to the nearest; the module reproduces 900 of the
texts' 901 result lines, the 901st (one RSQRT.S quotient) named as a loss. Six programs of the project's, 48,648
record words over every instruction and every input class, are equal on the interpreter to chapter 5 as `tests/s9/manual.rs`
transcribes it. PCSX2 and Play!, run as black boxes, both clamp exponent 255 to IEEE 754's largest number and multiply
exactly, and PCSX2 reproduces 721 of the console texts' 1,068 lines: neither lineage is a witness for the FPU's last
bit, and PCSX2 is the one witness for the adder's single extra bit. Disputes D-24 to D-33. No emulator, SDK or loader
source, no ps2autotests source and no firmware was read.

### 19.2 What was built, and the readings the manual leaves open

- **`src/float.rs`**, `sapphire::float`: `add`, `sub`, `mul`, `div`, `sqrt`, `rsqrt`, `fused` (the multiply-adds),
  `max`, `min`, `order` (the compares'), `to_word`, `from_word`. A number is IEEE 754's single layout read with no NaN,
  infinity or denormal (CORE §8.1.1): an exponent field of 255 is an ordinary exponent, 0 is zero whatever the fraction.
  Each result comes with its exponent overflow (±7FFFFFFFh) and underflow (±0); the flags they become are the caller's.
- **`src/ee/fpu.rs`**: the 34 instructions in one table, the third index space after S6's and S7's and S8's; FCR0,
  FCR31's fixed and writable bits, its fields. `Ee` gains `fpr`, `acc` and `fcr31`; COP1 is usable when Status.CU1 is set.
- **`src/ee/timing.rs`**: the floating-point rows of §2.12 (pinned since S6) in use, the FPRs, ACC and FCR31 among the
  scoreboard's resources (now 69, so the masks are 128 bits), and the divider DIV.S, SQRT.S and RSQRT.S share (§19.5).
- **`emusen-mips`**: SQRT.S's operand in ft, the form `FdFt` (D-24).
- **`ps2referee.py --interpreter`**: PCSX2's EE on its interpreter, its recompiler written back every run (§19.6).

**Readings** [R INST chapter 5, CORE chapter 8, Version 5.0; the console texts; A]:

- **The format.** Arithmetic takes an exponent field of 0 as zero and keeps its sign for the zero rules of Table 8-5;
  MOV.S, ABS.S, NEG.S, MAX.S and MIN.S move the bits as they are (the console's `abs CF_MIN_EXP: 00000001`). A zero
  operand of ADD.S gives the other operand unchanged, exponent 255 or not.
- **The last bit**: §19.3. **Overflow** is an exponent above 255 and gives ±7FFFFFFFh (`maxmin` in the pages);
  **underflow** an exponent below 1, ±0. A cancelling sum is +0.
- **The flags** (D-32): an instruction writes the cause flags its page names, clearing them where their condition does
  not hold (Table 8-2), sets their sticky flags and keeps the others; DIV.S raises I and D but never O or U; SQRT.S raises
  I for a negative operand and clears D; RSQRT.S raises D for a zero ft and I for a negative one (D-28); ABS.S, NEG.S,
  MAX.S and MIN.S clear O and U; MOV.S and the conversions touch nothing.
- **The multiply-adds** (D-30): an overflowing product gives ±7FFFFFFFh of ±TEMP's sign with O; an accumulator whose
  exponent field is 255 is kept, with O; otherwise ACC ± TEMP by ADD.S, its O and U; a product that underflows sets SU
  alone, as the pseudocode's "multiply underflow occurred".
- **DIV.S** by zero (either zero, a would-be denormal included): ±7FFFFFFFh by the operands' signs combined, I for 0/0,
  D otherwise. **SQRT.S** of either zero gives +0 (D-29). **RSQRT.S** takes fs's sign alone (D-28).
- **MAX.S and MIN.S** order the bits as sign and magnitude, −0 below +0 (D-31); the **compares** read every zero as
  equal and exponent 255 as a number (`lt 7f800000, 7f800001: true`).
- **CVT.W.S** truncates towards zero and clamps above exponent field 9Dh, with no flag (its page); **CVT.S.W** keeps
  the top 24 significant bits (VU User's Manual §2.2).
- **The moves** (D-33): MFC1 and CFC1 sign-extend; MTC1 and CTC1 take bits 31..0; CFC1 reads FCR31 for a register
  number with bit 4 set and FCR0 (00002E30h) otherwise; CTC1 writes FCR31's C, I, D, O, U and sticky bits, bits 24 and 0
  staying 1, and leaves FCR0. **LWC1 and SWC1** are LW and SW to an FPR, their data breakpoint judged on the FPR's value.
- **BC1F, BC1T, BC1FL, BC1TL** test C; the likely forms nullify the slot when not taken (D-2's evidence: `s9-misc`,
  equal on both referees).

### 19.3 The last bit: what the documents give, and what the console texts decide

**The documents** [R]: "The FPU only supports 'Rounding towards 0' … the FPU does not use the Guard, Round and Sticky
bits during rounding … the results may differ from the IEEE 754 Rounding to 0. This difference is usually restricted to
the least significant bit only" (CORE §8.7, §8.8); "A 24-bit calculation including hidden bits is performed, and the
result is truncated" (VU User's Manual §2.2); for the significand the multiply stage performs "the Booth
function/Wallace multiplication" (CORE §1.3.3). No document says how many bits the adder aligns to, which multiplier
columns are summed, or how the divider rounds.

**The console texts** [R ps2autotests' `cpu/ee_fpu/*.expected`, the files alone]: 901 lines print an instruction's
operands and result in hexadecimal; the named constants were read from the text's own `mov` lines (`mov CF_MAX_EXP:
7f800001`). Every reading below was tried against all of them with a model written for the purpose in the probe's
scratch [M, 2026-10-25], and `examples/s9.rs texts` grades the module against them:

| Operation | Readings tried | Lines equal | What decides |
|---|---|---|---|
| ADD.S, SUB.S, ADDA.S, SUBA.S | IEEE 754 towards zero; alignment to 24 bits and 0, 1, 2 or 3 more, the rest lost | 132; 144 each | `add CF_MIN, CF_ONE: ffffffff`: no sticky bit (D-25); the number of extra bits not decided |
| MUL.S, MULA.S | the exact product truncated; ft recoded as radix-4 Booth digits, the columns below *k* lost | 66; 72 for *k* = 1–22 | `mul CF_ONE, CF_MAX_MANTISSA: 3ffffffe` against `mul CF_MAX_MANTISSA, CF_ONE: 3fffffff` (D-26) |
| DIV.S | truncated; nearest | 35; 36 | `div CF_ONE, CF_MAX_MANTISSA: 3f000001` (D-27) |
| SQRT.S | truncated; nearest | 15; 21 | six roots one unit above truncation (D-27) |
| RSQRT.S | §19.3's list in D-28 | at most 38 of 39 | no reading fits all (D-28) |
| the multiply-adds | D-30's rules over the multiplier above | 236 | |

**The module**: 900 of 901 lines; the miss is `rsqrt 3f800000, 41c80000`, 3E4CCCCDh on the console and 3E4CCCCCh here.

**Ties.** Neither rounding to the nearest can meet a tie [A]: a quotient of two 24-bit significands with 25 significant
bits and an odd last would need the divisor's odd part to divide the dividend, leaving at most 24 bits; a root with 25
would square to at least 49. So "nearest" needs no tie rule.

**What remains open**, each a named loss: the adder's extra bit (D-25, one taken on PCSX2's evidence, none being the
documents' literal reading), the multiplier's lowest summed column (D-26, 1 taken, the texts allowing 1 to 22), and
RSQRT.S's rounding (D-28). Each is one constant or one function in `float.rs`; a console text with an addition that
loses bits and renormalises, a product whose low columns matter, or more reciprocal roots would decide them.

### 19.4 The gate: the programs (measured 2026-10-25)

`tests/s9/` generates six programs in S6's form, entered at 00100000h with the record from 00200000h and the done word
at 001FFFF0h. Each sets Status.CU1, loads its operands into FPRs by MTC1 from constants, sets the destination to a
sentinel, clears FCR31 by CTC1, runs the instruction, and records the register written by MFC1 and FCR31 by CFC1; ACC is
read by MADD.S with a product of −0, which leaves it unchanged (D-30), and set by ADDA.S with −0. The expected record
comes from `tests/s9/manual.rs`, chapter 5 transcribed on exact integer arithmetic (a value as its significand and a
power of two, quotients and roots carried far past 24 bits and rounded at the end), written apart from `float.rs` and
`fpu.rs`. **The input classes** (32): ±0; the smallest normals 00800000h, 80800000h, 00800001h; 7FFFFFFFh, FFFFFFFFh and
IEEE 754's largest 7F7FFFFFh; the patterns IEEE 754 reads as infinities and NaNs, 7F800000h, FF800000h, 7FC00000h,
7F800001h; would-be denormals 00000001h, 007FFFFFh, 80400000h; 1.0, −1.0, 0.5, 1.5, 3.0, 3FFFFFFFh, 3F800001h; 2⁶⁴, 2⁶⁵,
2⁻⁶⁴ and 2⁻⁶³, whose products overflow and underflow; π, 0.1, −123.456, 4B7FFFFFh, 2³¹, 1/3.

| Program | Cases | Record words | What it covers |
|---|---|---|---|
| `s9-add` | 2,048 | 8,192 | ADD.S and SUB.S over every ordered pair of the classes |
| `s9-muldiv` | 2,048 | 8,192 | MUL.S and DIV.S over every ordered pair: both orders of the multiplier, x/0 and 0/0, overflow and underflow |
| `s9-acc` | 5,040 | 24,768 | ADDA.S, SUBA.S, MULA.S over 144 pairs; MADD.S, MSUB.S, MADDA.S, MSUBA.S over eight accumulators (±0, ±1, 7FFFFFFFh, 7F800000h, the smallest normal, π) by 144 pairs; the register written, FCR31 and ACC after each |
| `s9-roots` | 1,056 | 4,224 | SQRT.S of every class (ft, with π in fs); RSQRT.S over every ordered pair |
| `s9-misc` | 1,093 | 3,122 | ABS.S, NEG.S, MOV.S, CVT.W.S of every class and CVT.S.W of 44 integers, from an FCR31 with O, U, SO, SU set; MAX.S and MIN.S over 144 pairs; the four compares over 144 pairs from C set opposite to their answer; MFC1 and MTC1 with bits above 31; CFC1 of all 32 numbers; CTC1's writable bits; LWC1 and SWC1 at each word of a quadword and at negative offsets; the four branches taken and not |
| `s9-exc` | 15 | 150 | COP1 unusable with CU1 clear (ADD.S, MFC1, CTC1, LWC1, SWC1, BC1T); LWC1's and SWC1's address errors; the TX79's CVT.D.S, ADD.D, C.UN.S, DMFC1 and LDC1 reserved (D-1's evidence) |

**Tests** [M, `cargo test --release`, `tests/fpu.rs`]: **9 of 9** — the six programs, each equal to its record word for
word; `the_programs_execute_every_cop1_instruction_the_interpreter_has` (all 34, from the assembled words through the
decoder); `the_format_reproduces_the_console_lines_that_decide_its_readings` (36 console lines and two compares, cited
from the texts); and the timing rows (§19.5). Lib **18 of 18**, two new: the Booth sum is the exact product or at most
two units of its lowest column below it over 200,000 random significands, and no quotient reaches a tie. `interpreter`
(S6) **23 of 23**, its halting check moved to QMFC2 alone; `cop0` (S7) **19 of 19**; `mmi` (S8) **13 of 13**, its count of
S8's instructions bounded to S8's 91; `programs` **3 of 3**; `fpu` and lib also under `test-checked`; clippy clean.
`emusen-mips` **18 of 18**; `ps2referee.py`'s tests **24 of 24**, one new (the recompiler written back every run).
WiseMan's `SapphireTests` **5 of 5**. `examples/s9.rs texts`: 900 of 901 console lines.

### 19.5 The timing model (§5.2, Q14)

[R CORE §1.3.3, §1.3.5, §1.3.6, §2.12; A]:

1. **The categories.** Table 1-1's COP1 Move (pipe 1) takes MFC1, MTC1, CFC1, CTC1, LWC1 and SWC1 ("COP1 move, COP1
   Load/Store"); COP1 Operate (pipe 0) the rest; BC1 the branch category. A COP1 Move beside a COP1 Operate is Table
   1-3's Y cell.
2. **The rows** of §2.12's floating-point table: MTC1 2, MFC1 2, LWC1 2; ADD.S, SUB.S, ABS.S, NEG.S and C.cond.S 4; CVT 4;
   MUL.S 4; MOV.S 4; DIV.S 8, repeating every 7; SQRT.S 8 and 7; RSQRT.S 14 and 13; the MADD row 4 for the four
   multiply-adds. The three long operations share one divider, the others issue beside it.
3. **The resources**: each FPR, ACC and FCR31 is scoreboarded like a GPR; a flag-writing instruction writes FCR31 at its
   row's latency, and BC1 and CFC1 read it, so C.EQ.S followed by BC1F waits 4 cycles.

**Named losses** [A], each a dispute candidate if an oracle ever measures it: the rows §2.12 does not give — SWC1 (taken
as LWC1's), CTC1 and CFC1 (MTC1's and MFC1's), ADDA.S and SUBA.S (the add row), MULA.S (the multiply row), MAX.S and MIN.S
(the add row); one divider for DIV.S, SQRT.S and RSQRT.S, which no document states; the multiply-adds' accumulator
forwarded at the MADD row, not waited for; FCR31's sticky bits read and written as one resource.

**Pinned** [M, `the_floating_point_rows_give_their_latencies_and_repeat_rates`], on bodies repeated so that each pass
starts warm: two independent ADD.S 2 cycles, dependent 5; dependent MUL.S 5; two DIV.S 14 a pass (the divider's 7
twice), DIV.S and SQRT.S 14, DIV.S and an independent ADD.S 7, a dependent one 9; two RSQRT.S 26; MTC1 to its reader 3
(the reader beside the next MTC1 in a Y cell), MFC1 to its reader 2 (the next MFC1 beside the reader), LWC1 to its
reader 3; ADD.S, MTC1 and an ALU instruction 3; C.EQ.S, BC1F and its slot 5.

### 19.6 On the referees (measured 2026-10-25)

**The project's programs.** `examples/s9.rs` writes each as an ELF with its expected record; `ps2referee.py ram …
--until 1FFFF0=5A99D00E 200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, one
process at a time. Only the records were read.

| Program (graded words) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s9-add` (8,192) | 923 | 1,072 | both clamp exponent 255 to 7F7FFFFFh; neither raises O or U; on the 800 in-range sums PCSX2 equals the module (one extra bit, D-25) and Play! equals IEEE 754 towards zero |
| `s9-muldiv` (8,192) | 1,266 | 1,187 | clamping; both multiply exactly and commutatively (70 in-range products each, D-26); Play! truncates quotients (121), PCSX2 rounds them as the module does (D-27); Play! raises no D or I |
| `s9-acc` (24,768) | 5,162 | 4,769 | clamping, the exact multiplier, no O, U or SU from the multiply-adds (D-30) |
| `s9-roots` (4,224) | 654 | 1,067 | clamping; PCSX2's roots equal the module's, Play! truncates three; PCSX2 gives 45 reciprocal roots one unit above the module's, one of them against the console (D-28) and raises I for −0 (D-29) |
| `s9-misc` (3,122) | 446 | 447 | both return fs for max(+0, −0) and flush would-be denormals in MAX.S and MIN.S (D-31); both keep O and U through ABS.S, NEG.S, MAX.S, MIN.S (D-32); clamping in ABS.S and NEG.S; the compares, moves, loads, stores and branches equal on both, SQRT.S read from ft on both (D-24) |
| `s9-exc` (120) | 60 | 60 | neither raises the unusable, address error or reserved instruction exception from COP1, as at S6 (§16.6) |

**ps2autotests' `cpu/ee_fpu`, on PCSX2.** The nine programs need the BIOS's kernel, so Sapphire runs them at S10. Run now
through `ps2referee.py tty pcsx2 … --channel iop` (Americas BIOS) and compared with their expected texts, the consoles'
outputs, after line ends are normalised [M `diff`]:

| Text (lines) | PCSX2, recompiler | PCSX2, interpreter |
|---|---|---|
| `fcr` (53) | 6 differ: an overflow's O and SO not raised; ±7F7FFFFFh printed where the console prints exponent 255 | 25: FCR0 00002E00h, FCR1 to FCR31 0 |
| `transfer` (26) | equal | 2: FCR31's fixed bits and CTC1's mask not modelled |
| `branch` (26) | equal | equal |
| `branchdelay` (4) | none printed: the referee stopped on a signal (a branch in a branch's delay slot, undefined) | 1 |
| `arithmetic` (293) | 116: exponent 255 clamped, max(+0, −0) | 52 |
| `muldiv` (360) | 184: clamping, the commutative multiplier, the multiply-adds | 155 |
| `compare` (170) | 4: 7F800000h and 7F800001h compared as equal | 4 |
| `convert` (70) | equal | equal |
| `sqrt` (66) | 33: clamping, RSQRT.S's last bit | 34 |

So **the PCSX2 lineage cannot be trusted on the FPU's arithmetic**, reproducing 721 of the 1,068 lines on its
recompiler and 795 on its interpreter: it reads exponent 255 as IEEE 754's infinity clamped, its multiplier is exact, and its flags are partly
modelled. It is trustworthy on the moves, FCR31's bits (recompiler), the branches and the conversions, and in the
normal range on DIV.S and SQRT.S, where it agrees with the console's rounding. Its adder's single extra bit is the one
reading it alone supplies (D-25). Play! was not run on the texts (`tty` drives PCSX2 alone, §18.10 Q3). Only the
expected texts and the programs' own output were read, never `.cpp` or `shared.h`.

### 19.7 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | the addition rounding to the nearest instead of truncating | `add_and_subtract_over_every_pair_of_input_classes_equal_the_manual`, `the_accumulator_instructions_equal_the_manual`, `the_format_reproduces_the_console_lines_that_decide_its_readings` |
| F2 | an underflowing result kept as a denormal instead of flushed to ±0 | `add_and_subtract_…`, `multiply_and_divide_…`, `square_root_and_reciprocal_square_root_equal_the_manual`, `the_accumulator_instructions_…` |
| F3 | the overflow flag O left clear (SO still set) | `add_and_subtract_…`, `multiply_and_divide_…`, `the_accumulator_instructions_…` |

Each was applied, `tests/fpu.rs` run, and the source restored [M]. Found while building, not seeded: MAX.S's order first
put −0 equal to +0, which two console lines exposed through `examples/s9.rs texts`; and ACC first set by ADDA.S with +0,
which turned −0 into +0, exposed by `s9-acc`.

### 19.8 What the plan got wrong, or left open

- **§1.1's COP1 row and §5.3 say "rounding forced towards zero"**: for DIV.S and SQRT.S the console texts show rounding
  to the nearest (D-27), and the "towards zero" of the other operations is towards zero of an inexact intermediate
  (D-25, D-26), not of the exact result. §5.3's software floats were the right call: no host's floating point gives the
  multiplier's non-commutative last bit.
- **§3.2's FPU row names the referees as the secondary oracle**: on the FPU they are not one (§19.6); the console texts,
  read directly, carry the stage. PCSX2's weight is per unit, as §18.7 found for the MMI the other way.
- **§5.13's C3** (host floats with clamping on the classes not proven equal): the classes are now measurable — exponent
  255, the would-be denormals, overflow and underflow, and in the normal range every product (D-26), every sum that
  loses bits to alignment (D-25) and every quotient and root (D-27). A host's multiply agrees with the module only
  where the product's lowest column cannot reach the result, which no cheap test proves; C3's measurement at S39–S41
  should count how often games' products differ, against the speed.
- **S9's gate** ("the project's programs on every input class … equal to the manual's record") holds; the referees
  cross-check far less than at S6 to S8.

### 19.9 Questions this stage leaves

1. **Q1 [routine], the adder's extra bit (D-25).** *Recommended*: keep one bit, PCSX2's reading, since the console texts
   allow it and the lineage that separates the readings gives it on all 800 in-range sums; the documents' literal "no
   guard bit" stays the alternative in D-25, one constant away, until a console text decides.
2. **Q2 [routine], the multiplier's lowest summed column (D-26).** *Recommended*: keep *k* = 1, the departure from the
   exact product the texts force and no more; revisit if VU tests (S12, S13) show the same multiplier with more
   evidence, since `vu/*`'s console texts may exercise it.
3. **Q3 [routine], RSQRT.S (D-28).** *Recommended*: accept the 38-of-39 reading as a named loss; look again at S12, where
   VU0's RSQRT and its console texts may share the unit.
4. **Q4 [routine], the referees on the FPU.** *Recommended*: record in `Sapphire_Referees.md`'s next edit that neither
   lineage is a witness for the FPU's arithmetic (exponent 255 clamped, exact multiplier) and that PCSX2's EE interpreter
   is weaker than its recompiler on FCR access; weigh the console texts first for every FPU and VU dispute.
5. **Q5 [routine], C3 (§5.13).** *Recommended*: when C3 is weighed (S39–S41), measure the share of game products and sums
   where a host's arithmetic differs from `float`, per class, before any host path is offered.

### 19.10 What was read for this stage

`Sapphire_Plan.md` §1.1 (the COP1 row), §2, §3.2, §5 (§5.1 to §5.3, §5.13's C3), §7 and §11; this page's §16 to §18;
`Sapphire_Disputes.md`; `Sapphire_References.md` §1.1 and §2; `Mars_FpuMath.md` §1 to §5 and §11 for MarsRT's method;
`EmuSen_Mips.md` §8; `emusen-mips`'s `r5900.rs` (COP1's tables and forms); `ps2referee.py`. Sony's *EE Core Instruction
Set Manual* chapter 5 whole and §6.1; *EE Core User's Manual* §1.3.3, §1.4.2, §1.8, §2.10.1, §2.12 and chapter 8 whole;
*VU User's Manual* §2.2 to §2.4. ps2tek at `1c91660`: "EE COP1 Registers", "EE COP1 Floating Point Format".
ps2autotests' `cpu/ee_fpu/*.expected` (the consoles' text, not the programs' source). The first page of Tago et al.,
"CPU for PlayStation 2", DATE 2001, found by a search for the multiplier's design (nothing on it). PCSX2's own settings
file, for the name of its EE recompiler switch. No emulator, SDK or loader source, no ps2autotests source and no
firmware.

### 19.11 What this stage does not cover

- `ee_fpu/*` on Sapphire, and the machine running programs: S10.
- COP2 (S12): still halting. The VUs' own flags (MAC, status, clipping) and their FMAC's agreement with `float`: S12, S13.
- COP1 in the recompiler: S38, graded against this interpreter; C3's host path: S39–S41.
- The state format's new contents (the FPRs, ACC, FCR31): `SPH2` gains them with the rest of the EE (S10, S30).

## 20. Stage S11, the DMAC, the INTC and the EE timers (2026-10-25)

### 20.1 In one paragraph

The plan's §7 row S11, on the tester's go of 2026-10-25, built before S10 by the tester's decision of the same day
(recorded in its own commit, `Sapphire_Plan.md` §7). The crate gains the EE's peripherals around its core, written from
the *EE User's Manual* (EEUM) chapters 2 to 5 and ps2tek where the manual is silent: **`dmac`**, the ten channels in
normal, chain and interleave modes with source and destination tags, tag interrupts, the address stack, stall
control, the MFIFO, the priority control, the release cycle, D_ENABLEW's hold, INT1 and CPCOND0; **`intc`**, I_STAT and
I_MASK and INT0; **`timer`**, the four timers on BUSCLK, its dividers and H-blank, with their gates, compare and
overflow flags; **`blank`**, a stand-in for the H-blank and V-blank signals until the PCRTC (S20); **`clock`**, the one
clock of EE cycles they share (D-0's figures in one place); and **`board`**, which holds them with the physical map and,
as `System`, runs the core one instruction at a time and brings the units to its time, driving the core's Int0, Int1
and CPCOND0, the inputs S7 left open. Units are brought forward only at the events they schedule or when the core
touches them. The channels whose peripheral comes later end at `dmac::Peripherals`, an interface S13 to S27 implement;
the scratchpad channels work end to end. Eight bare programs of the project's, 1,230 graded record words, are equal on
the interpreter to the records the manual's rules give, the interleave arithmetic also to ps2autotests' console text.
On PCSX2 and Play!, run as black boxes, neither lineage models stall control or the priority control, and PCSX2 takes
an interrupt only when its request rises; the disagreements are disputes D-34 to D-42, D-15 is closed. The seven
ps2autotests programs of the stage's gate need the BIOS's kernel, so Sapphire runs them with S10; **PCSX2 reproduces all
seven console texts.** No emulator, SDK, loader or ps2autotests source and no firmware was read.

### 20.2 What was built

- **`clock.rs`**: the EE's 294.912 MHz, BUSCLK at two EE cycles, the NTSC and PAL lines and fields (§20.3), and the
  stand-in's H-blank width.
- **`intc.rs`**: the fifteen sources by bit (EEUM §3.1's table), I_STAT latching a request, I_STAT's bits cleared by
  writing 1 and I_MASK's reversed by writing 1 (§3.2), INT0 = I_STAT AND I_MASK not zero.
- **`timer.rs`**: a timer is a count valid at a time, advanced in closed form over the ticks since; its next flag's
  tick is the only event it schedules, and a settled timer (no enabled flag left to rise) schedules none.
- **`blank.rs`**: H-blank and V-blank as functions of time, and their next edge, H-blank's only when a timer counts or
  gates on it.
- **`dmac.rs`**: the registers of EEUM §5.8 and §5.9, and the engine. A transfer runs in *units*: a slice (up to 8
  qwords, ending at an 8-qword boundary of memory, §5.2.2) on the slice channels; on the burst channels (fromSPR, toSPR)
  a whole packet, or with RELE a part to the next 128-byte boundary, or an interleave block; and a tag. A unit costs
  its qwords at one a bus cycle and takes effect at its last cycle; between units the channels are arbitrated, group A
  (VIF0), then B (SIF2), then C by round robin (§5.3.1), each channel's zero-time steps (a packet's end, a finished
  transfer) taken at once. The DMAC is woken only by a unit's end, a release period's end, a register write or a
  peripheral's `Dmac::wake`.
- **The interfaces** [C]: `dmac::Peripherals` (DREQ for up to *n* qwords, a qword handed to the peripheral with a flag
  for a TTE tag, a qword taken from it, a destination chain's tags among them) and `dmac::Memory` (a qword by physical
  address, `None` for a bus error). `board::StandIns` implements `Peripherals` for channels 0 to 7 until their stages:
  it takes every qword (keeping them for tests on request) and gives a queue, holding its channel when empty.
- **`board.rs`**: `Board` (the map, the units, the stand-ins, the time) answers the EE's registers at 10000000h to
  1000FFFFh through `Bus`, the timers, the DMAC (with D_ENABLER and D_ENABLEW at 1000F520h and 1000F590h) and the INTC,
  other registers reading zero; `System` steps the core, runs the board to `Timing::cycles()` and sets the core's lines.
- No new cross-component dependency: `sapphire` still depends on `emusen-native` and `emusen-mips` alone. The scheduler
  is the crate's own, in Diamond's form (each unit's next event, the board taking the earliest).

### 20.3 The timings: the documents' and the argued

**The documents'** [R]: the EE at 294.912 MHz (ps2tek "EE Architecture"; D-0); BUSCLK at 147.456 MHz, half of it (EEUM
§2.1.2, Tn_MODE.CLKS); the timers' clocks BUSCLK, BUSCLK/16 and BUSCLK/256 (§4.2); RCYC's release of 8 to 256 bus
cycles (§5.8); the slice of 8 qwords ending at an 8-qword boundary and the burst's release at each 128-byte boundary
with RELE (§5.2.1, §5.2.2); a qword a bus cycle ("It can transfer one quadword (16 bytes) at bus speed", ps2tek "DMA
Controller"); the NTSC line of 9,370 bus cycles, 262 lines with V-blank on 22, and PAL's 9,436, 312 and 26 (ps2tek "EE
Timers").

**Argued or chosen**, each a named loss and a dispute candidate if an oracle measures it:

1. A tag read costs one qword's bus cycle, and a TTE tag is moved in the same cycle [A].
2. A unit has no set-up cost, and its effects (the data, MADR, QWC, SADR, D_STADR) become visible at its last bus
   cycle [A].
3. With RELE the release of RCYC bus cycles follows every unit, as if the core always asked for the bus (§5.2.1 makes
   it conditional on the core's request, which the core's model does not raise) [A].
4. The core is never stalled by the DMAC's use of the bus [A]: with RELE clear §5.3.1 makes the core's request the
   lowest, which would delay its uncached and refilling accesses while a burst runs.
5. The core reads and writes the units as of the cycle its instruction begins, and the units' lines reach it before
   the next instruction (instruction granularity, §5.2 of the plan) [A].
6. The dividers run freely from time 0, so BUSCLK/16 ticks at multiples of 32 EE cycles whenever a timer starts [A].
7. The stand-in's field starts at time 0, H-blank is high for 1,600 bus cycles at each line's start, a timer clocked by
   H-blank counts on its rising edge, and V-blank's edges fall on line starts [C].

**Measured** [M, `tests/dmac.rs`]: 64 qwords on toSPR take 128 EE cycles in one burst; with RELE and RCYC 64 they take
eight units of 8 qwords with seven releases of 64 bus cycles between them; 20 qwords on GIF from the third qword of a
slice move as 5, 8 and 7; a channel whose peripheral asserts no DREQ schedules nothing until woken.

### 20.4 The readings the manual leaves open (the DMAC and the INTC)

[R EEUM chapters 3 and 5; ps2tek "DMA Controller", "EE Interrupt Controller"; A]:

- **Registers.** CHCR keeps DIR, MOD, ASP, TTE, TIE, STR and TAG, bits 1 and 9 to 15 reading 0; while STR is set a
  write changes STR alone (§5.9: "The fields other than the STR can be written only while DMA is stopped"). MADR, TADR and
  the ASRs keep bits 31..4, MADR's bit 31 fixed at 0 on fromSPR and toSPR (§5.9; ps2autotests' `dma/spr/normal` text,
  "did not accept MADR: 80000000"). SADR keeps bits 13..4 and wraps the scratchpad at 16 KiB. QWC keeps 16 bits; D_SQWC
  every bit (D-40); the offsets §5.9's table gives a channel no register for are reserved, indeterminate on a read.
- **A chain's start.** STR set with QWC above zero moves that packet from MADR first and then acts on CHCR.TAG as the
  last tag read (§5.2.3's first paragraph; ps2tek "DMA Resuming"): refe and end stop after it; any other reads the next
  tag at TADR. With QWC zero the first tag is read at once.
- **Source tags** (§5.2.3, §5.6): MADR and TADR as the table gives them; cnt's TADR takes the address after its data
  when the packet ends ("updates Dn_TADR following the transfer"); end, and ret with nothing pushed, leave TADR at the tag
  (D-35); call pushes the address after its packet into ASR0 or ASR1 by ASP, ret pops it. refs is under stall control on
  VIF1, GIF and SIF1 and is ref elsewhere. call and ret on a channel without an address stack, and call with ASP at 2,
  end the transfer with CIS set, CHCR keeping the tag's bits, TADR at the tag and nothing moved (§5.4; D-37). A tag's
  PCE of 10 or 11 clears or sets D_PCR.PCE (§5.3.3; D-41).
- **TTE** sends the tag's whole quadword before its packet: to the scratchpad on toSPR (D-36), to the peripheral
  marked as a tag elsewhere, whose stage decides what of it the peripheral takes.
- **Destination tags** (§5.2.4) come from the peripheral, on fromSPR from the scratchpad at SADR: cnt, cnts (under stall
  control as a source) and end; MADR takes ADDR (its SPR bit ignored on fromSPR); any other ID ends the transfer (D-37).
- **Tag interrupts**: a tag with IRQ under TIE stops the chain when its packet ends, CIS set, TADR at the next tag
  (§5.4). IRQ without TIE does nothing (ps2autotests' `dma/dmac/tagintr` text).
- **Interleave** (§5.2.5): TQWC qwords moved, SQWC skipped on the memory side, until QWC is spent; the skip after the
  last block is taken, so MADR ends at QWC/TQWC × (TQWC + SQWC) qwords from its start, and each field is its low eight
  bits, as all thirteen of the console text's MADR lines show (`the_interleave_rule_gives_the_console_texts_madr_lines`).
  A TQWC of 0 is taken as 1 [A].
- **Stall control** (§5.3.2; D-34): the drain (STD) holds a normal-mode or refs packet while Dd_MADR + 8 qwords >
  D_STADR, the "8" read as qwords from Figure 5-7; every held request sets SIS, so SIS clears only once the stall is
  gone; the source (STS) copies its MADR to D_STADR after each unit of a normal-mode or cnts packet.
- **MFIFO** (§5.5): with MFD set, fromSPR's addresses and the drain's TADR and the data of its cnt, next, call, ret and
  end packets wrap into the ring (D_RBOR | address AND D_RBSR); the drain holds while its TADR equals fromSPR's MADR,
  setting MEIS when that happens at a tag read, not during a packet's data (the manual's restriction against ending a
  packet with ref). The ring's free space is the program's to check (§5.5's "After checking that there is a space") [A].
- **Holds**: DMAE clear, D_ENABLEW's CPND, and D_PCR.PCE with a channel's CDEn clear, each stop the arbitration of what
  they cover (§5.8's formula); a transfer held is resumed, not lost.
- **INT1** is §5.8's formula, BEIS unmasked; **CPCOND0** is §5.8's, true with no CPCn set (ps2autotests' `cpcond` text,
  "initial value: 1"). D_STAT's CIS, SIS, MEIS and BEIS clear by writing 1, its masks reverse by writing 1.
- **A bus error** (§5.4): a DMA address in the map's bus-error regions ends the transfer with BEIS and CIS.
- **INT0 and INT1 are levels** sampled before each instruction (§3.1: "if any bit is 1, an INT0 interrupt is
  asserted"); a request latched while masked is taken once unmasked (D-39). Int0 is Cause.IP2 and Int1 IP3 (D-15,
  measured).
- **Normal or interleave mode with QWC zero** ("will result in unsuccessful transfer", §5.10) ends at once with CIS [A];
  a chain mode on a channel without one, and interleave off the scratchpad channels, run as normal [A].

### 20.5 The timers and the blanking stand-in

**The timers** [R EEUM chapter 4; ps2tek "EE Timers"; A]:

- COUNT, COMP and HOLD keep 16 bits; MODE keeps bits 9..0, EQUF and OVFF clearing by writing 1. Writing MODE leaves
  COUNT as it is (D-38). HOLD exists on timers 0 and 1 and is plain storage until the SBUS interrupt drives it (S22).
- EQUF is set only with CMPE, at the tick the count reaches COMP; OVFF only with OVFE, at the tick it wraps from FFFFh
  ("set to 1 when a compare-interrupt occurs"). A flag's rise, not its level, latches the timer's bit in I_STAT, so a
  flag left set gives no further request (ps2tek: "an interrupt will only be sent to the EE if either interrupt flag
  goes from 0 to 1").
- **ZRET**: the tick that brings the count to COMP gives 0 and sets EQUF with CMPE, so COMP is never read and the period
  is COMP ticks (D-38, measured on PCSX2). With COMP below the count the counter runs to FFFFh and wraps first.
- **The gate**: GATS selects H-blank or V-blank, H-blank ignored while it is also the clock. GATM 0 holds the count
  while the gate is high; GATM 1, 2 and 3 count throughout and reset to 0 at the rising, falling or either edge (ps2tek:
  "Reset counter when gate goes from low to high"; the manual's "Resets and starts counting" read the same way, both
  referees counting before the first edge, D-38).
- **H-blank as the clock** (CLKS 3): one tick at each rising edge.

**The stand-in** (`blank.rs`), named so S20 can replace it: NTSC by default, a field of 262 lines of 9,370 bus cycles
from time 0, V-blank high on lines 240 to 261, H-blank high for the first 1,600 bus cycles of each line [C]; VB_ON at
V-blank's rise and VB_OFF at its fall (§3.1's edge column). Interlace and the half line are not modelled. The referees
disagree with each other on the field (D-42).

### 20.6 What is not modelled

Each a named loss, and a dispute candidate if an oracle ever measures it [A]:

- The DMAC's use of the bus does not stall the core (§20.3, 4), and the core's bus requests do not shape the release
  (§20.3, 3); RDRAM's latency is S7's loss, not charged to the DMAC either.
- The MFIFO's source never waits for space in the ring; D_RBOR and D_RBSR changed mid-transfer act at once (the manual
  calls it unstable).
- §5.10's restriction (VIF1 with TTE beside toSPR without it, which the manual says may stall) is not modelled.
- STR cleared during a unit drops the unit's effect, and D_ENABLEW is honoured at the next arbitration; the manual says
  a transfer "may not stop properly", which is not modelled further.
- SIF2 runs in normal mode only; the SFIFO, SBUS and the GS's interrupt sources are not driven (S16, S22); HOLD is not
  latched; the VU0 watchdog is S12's.
- The blanking stand-in (§20.5); the timers' count of H-blank therefore stands until S20.

### 20.7 The gate: the programs (measured 2026-10-25)

`tests/s11/` generates eight bare programs as `emusen-mips` assembly, in S7's form (entered at 00100000h, running from
kseg0, constants, buffers and record through kseg1, so neither cache holds what the DMAC moves), with the record from
00200000h and 5A99D00Eh at 001FFFF0h. Each starts by clearing the DMAC's channels, D_PCR, D_ENABLEW, D_STAT, the INTC and
the timers and enabling DMAE. Waits are bounded loops polling a register, so a referee that never sets a bit still ends
its record. Every expected word is derived in the generator from the manual's rules, written apart from `src/`.

| Program | Graded words | What it covers |
|---|---|---|
| `s11-spr` | 156 | toSPR and fromSPR in normal mode; SADR's dropped bits and its wrap at 16 KiB; MADR's fixed SPR bit; D_STAT's clearing and reversing writes; the holds by D_ENABLEW, DMAE and PCE with CDEn; CPCOND0 through BC0T in the six states ps2autotests' `cpcond` prints; QWC's and D_SQWC's widths |
| `s11-chain` | 294 | toSPR's source chain walked tag by tag by tag interrupts (cnt, next, ref, refs, cnt setting PCE, refe clearing it); IRQ without TIE; a start with QWC left under end and ref; TTE; call on toSPR; fromSPR's destination chain (cnt with IRQ, cnts moving D_STADR, end) and an undefined tag |
| `s11-callret` | 72 | VIF0's chain: two calls filling ASR0 and ASR1, a third refused at ASP 2, two rets, a ret ending the chain |
| `s11-interleave` | 480 | eight of the console text's upload cases (TQWC 257 and SQWC 256 among them) and two downloads, MADR, SADR and every qword moved or skipped |
| `s11-stall` | 38 | fromSPR as the source and VIF1 as the drain: held at once, released by 8 qwords at a time as D_STADR moves, finished, SIS cleared once free, and released by STD cleared |
| `s11-mfifo` | 62 | a 64-qword ring: VIF1 held with MEIS while empty, fromSPR wrapping, a cnt packet's data across the wrap, the chain's end |
| `s11-intc` | 74 | I_MASK's and I_STAT's writes; a timer's compare taken at its edge (IP2); a request latched while masked, taken once unmasked; the DMAC's end of transfer through CIM (IP3); both lines at once; V-blank's start and end and the span between them |
| `s11-timers` | 54 | the three dividers' ratios; CUE; COUNT's and MODE's writes; EQUF with and without CMPE and its edge, none for a flag left set; ZRET; OVFF with and without OVFE; MODE's, HOLD's and COMP's widths; the V-blank gate in modes 0 and 1; H-blank as the clock over a field |

**Tests** [M, `cargo test --release`]: `tests/dmac.rs` **17 of 17** — the eight programs, each equal to its record word
for word (VIF0's stand-in receiving the three packets' zeros and no tag; VIF1's the 70 qwords of the ring's cnt packets;
V-blank lasting 22 lines; 262 H-blank ticks a field; the dividers within a tick of 16:1); the console text's thirteen
MADR lines; on the units directly, the burst's and the release's cycles, the slice's division, a channel without DREQ
scheduling nothing until woken, group A before group C, a bus error with BEIS unmasked, a timer's flag at its tick and
nothing scheduled once settled, ZRET's sequence, and the board scheduling no H-blank edge until a timer asks for one.
The crate's other tests unchanged: lib **18 of 18**, `cop0` **19 of 19**, `fpu` **9 of 9**, `interpreter` **23 of 23**,
`mmi` **13 of 13**, `programs` **3 of 3**; `dmac` and lib also under `test-checked`; clippy clean.

### 20.8 On the referees (measured 2026-10-25)

**The project's programs.** `examples/s11.rs` writes each as an ELF with its expected record; `ps2referee.py ram …
--until 1FFFF0=5A99D00E 200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, one
process at a time. Only the records were read.

| Program (graded words) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s11-spr` (156) | 1 | 15 | neither holds a channel for PCE with CDEn clear (D-41); Play! loses the transfer held by D_ENABLEW and by DMAE and keeps 17 bits of QWC |
| `s11-chain` (294) | 17 | 14 | neither sets D_PCR.PCE from a tag (D-41); call on toSPR: PCSX2 moves its packet and ends with TADR at the tag, Play! performs the call (D-37); PCSX2 copies MADR to D_STADR after every destination packet, not only cnts (D-34), and does not end on an undefined destination tag (D-37); Play! ignores TIE in the destination chain; TTE's tag quadword equal on both (D-36) |
| `s11-callret` (72) | 1 | — | PCSX2 moves the call's packet at ASP 2 (D-37); Play! never wrote its done word |
| `s11-interleave` (480) | 0 | 0 | both lineages equal the manual's rule and the console text |
| `s11-stall` (38) | 10 | 12 | neither holds the drain; Play! does not move D_STADR (D-34) |
| `s11-mfifo` (62) | 0 | 8 | PCSX2 equal, MEIS, the wrap and TADR left at the end tag (D-35) among them; Play! sets no MEIS and holds no drain |
| `s11-intc` (74) | 17 | — | PCSX2 takes INT0 at its edge with IP2 and INT1 with IP3 (D-15), but no request latched while masked and unmasked later, and none pending when IE is set (D-39); Play! never wrote its done word, its BIOS taking no interrupt through the vector (§16.6) |
| `s11-timers` (54) | 1 | 4 | PCSX2 counts 525 H-blanks between two V-blank starts (D-42); Play! has no ZRET, sets EQUF without CMPE and does not wrap the count at 16 bits; both count in gate mode 1 before the first edge and keep COUNT across MODE's write (D-38) |

**ps2autotests on PCSX2.** The seven programs of the stage's gate call the BIOS's kernel, so Sapphire runs them with
S10. Run now through `ps2referee.py tty pcsx2 … --channel iop` (Americas BIOS) and compared with their expected texts,
the consoles' outputs, after line ends are normalised [M `diff`]:

| Program (lines) | PCSX2 |
|---|---|
| `dma/dmac/cpcond` (8), `dma/dmac/tagintr` (10) | equal |
| `dma/spr/chain` (25), `dma/spr/interleave` (629), `dma/spr/normal` (36) | equal |
| `kernel/ee/interrupt/dmac_order` (9), `kernel/ee/interrupt/intc_order` (10) | equal |

So **PCSX2's lineage reproduces every console text on the DMAC's scratchpad channels, its tag interrupt and CPCOND0, and
the kernel's interrupt handler order**, unlike the FPU (§19.6): it can be trusted on what those texts exercise, and it
supplies the readings D-35 and D-38 where the manual is silent. Its gaps are what no text exercises: stall control, the
priority control, undefined tags and the interrupt lines' levels. The kernel tests exercise the BIOS's handler lists
over the INTC and the DMAC, which on Sapphire waits for S10 and S24's BIOS.

### 20.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | a chain tag's ADDR taken from its first word instead of bits 62..32 | `source_and_destination_chains_walk_their_tags_as_chapter_5_says`, `call_and_ret_push_and_pop_the_address_stack` |
| F2 | the stall source never copying its MADR to D_STADR, so a held drain never resumes | `stall_control_holds_and_releases_the_drain`, `source_and_destination_chains_walk_their_tags_as_chapter_5_says` |
| F3 | BUSCLK/16 counted at BUSCLK/8 | `the_timers_count_compare_overflow_and_gate_as_chapter_4_says`, `a_timer_is_evaluated_at_its_flag_and_not_before` |

Each was applied, `tests/dmac.rs` run, and the source restored [M].

### 20.10 What the plan got wrong, or left open

- **§5.1's scheduler**, taken as S1's: S1 built none (§16.11 said so); the scheduler is built here, each unit reporting
  its next event and the board taking the earliest, as Diamond's bus does, with D-0's ratio in `clock.rs`.
- **§3.2's row for the DMAC, INTC and timers** names the referees as the secondary oracle: PCSX2 is one for what the
  console texts exercise (§20.8) and for nothing else here; Play! is one for interleave alone.
- **S11's gate** names three folders of ps2autotests that need the BIOS's kernel: they are met on PCSX2 against their
  texts and wait for S10 on Sapphire, as `ee_simd` and `ee_fpu` did at S8 and S9.
- **The H-blank and V-blank** belong to the PCRTC (S20) but S11's timers and INTC need them: the stand-in (§20.5) is the
  plan's gap made explicit, and the referees disagree on it (D-42).

### 20.11 Questions this stage leaves

1. **Q1 [routine], the stall condition's "8" (D-34).** *Recommended*: keep it as 8 qwords, Figure 5-7's reading, since
   neither lineage models stall control; look again if a game at S28 stalls a drain.
2. **Q2 [routine], the interrupt lines as levels (D-39).** *Recommended*: keep the manual's level semantics, which
   PCSX2 departs from; the kernel tests at S10 exercise the handlers over these lines on the player's BIOS and will show
   any program that depends on the difference.
3. **Q3 [routine], undefined tags (D-37).** *Recommended*: keep §5.4's reading (the transfer ends, nothing moved), the
   two lineages disagreeing with each other and with it.
4. **Q4 [routine], the blanking stand-in (D-42).** *Recommended*: keep ps2tek's 262 lines and 22 of V-blank, which Play!
   agrees with, until S20 builds the PCRTC; PCSX2's 525 H-blanks between V-blank starts is the first thing S20 should
   explain.
5. **Q5 [routine], the DMAC's cost to the core (§20.3, 3 and 4).** *Recommended*: leave the core unstalled by the DMAC
   until S32's bench shows a game whose timing depends on it; modelling the core's bus requests is a cost at every
   uncached access.

### 20.12 What was read for this stage

`Sapphire_Plan.md` §1.1 (the DMAC, INTC and timers row), §2, §3.2, §5.1, §5.2, §7 and §11; this page's §16 to §19;
`Sapphire_Disputes.md` D-0 to D-33; `Sapphire_References.md` §1.1 and §3; Diamond's `bus.rs` scheduling (the project's own
code, for the event form); `ps2referee.py`'s usage. Sony's *EE User's Manual* chapters 2, 3, 4 and 5 whole; the *GS
User's Manual* searched for the blanking's timing (none given). ps2tek at `1c91660`: "EE Timers", "DMA Controller" with
its I/O, chain mode, MFIFO and interrupt sections, "EE Interrupt Controller" (its D_ENABLEW line 1680, excluded for
replacement writers, was passed over by this writer, who writes no open BIOS). ps2autotests' `dma/dmac/*.expected`,
`dma/spr/*.expected` and `kernel/ee/interrupt/*.expected` (the consoles' texts, not the programs' source). No emulator,
SDK, loader or ps2autotests source and no firmware.

### 20.13 What this stage does not cover

- The machine running programs, and ps2autotests on Sapphire: S10. The machine (`machine.rs`) does not yet hold a
  `System`; the programs run through the tests.
- The peripherals behind channels 0 to 7: VIF0 and VIF1 (S14), the GIF (S15), the IPU (S27), the SIF (S22), each
  implementing `dmac::Peripherals`; the VU memories the scratchpad channels can reach in burst mode (S12, S13).
- The PCRTC's own H-blank and V-blank (S20); the SBUS interrupt and the timers' HOLD (S22); the GS's interrupt (S16).
- The state format's new contents (the DMAC's, INTC's and timers' registers, the board's time): `SPH2` gains them with
  the rest of the EE (S10, S30).

## 21. Stage S12, VU0 in macro mode and the VU float (2026-10-25)

### 21.1 In one paragraph

The plan's §7 row S12, on the tester's go of 2026-10-25. The crate gains **`vu`**, the vector units: one `Vu` per unit
(32 vector registers, 16 integer registers, ACC, I, Q, P, R, the MAC, status and clipping flags, TPC, its data and micro
memories), the operations both units execute (`vu::ops`, 120, each decoded by `emusen-mips`'s VU table alone and
computed on `sapphire::float`), and `Vpu`, VU0 and VU1 with VU0's sixteen control registers, VU0's divider under macro
instructions, VU0's window onto VU1's registers, and each unit's micro mode behind **`vu::micro::Micro`**, the interface
S13 implements. The EE gains **COP2** (`ee/cop2.rs`): QMFC2, QMTC2, CFC2 and CTC2 with their interlock bit, LQC2, SQC2,
BC2F, BC2T and the likely forms, every VU operation as a macro instruction, VCALLMS and VCALLMSR, with the interlocks of
the *VU User's Manual*'s §5.4 charged to the timing model. `sapphire::float` stays the one owner of the format: the VU
adds no arithmetic of its own, only the fixed-point conversions and a multiply-add's product as it was added; its flags
are the VU's. `emusen-mips`'s VU table takes the manual's operand forms in place of `Raw`. Seven bare programs of the
project's, 26,102 graded record words over every macro instruction and transfer, are equal on the interpreter to the
manual's pages as `tests/s12/manual.rs` transcribes them. PCSX2 and Play!, run as black boxes, compute the VU on IEEE 754's
rounding towards zero on every in-range lane, raise no U or O, and land a quotient in Q at once; PCSX2 reproduces
`cpu/vu0_macro/integer`'s console text whole and `transfer`'s but eight lines, which the project's model reproduces.
Disputes D-43 to D-54. No emulator, SDK or loader source, no ps2autotests source and no firmware was read.

### 21.2 What was built

- **`emusen-mips`** (`vu.rs`, `r5900.rs`): the forms of §4.1 and §6.1, each operation's mnemonic line of §4.2, §4.3 and
  §6.2 (`EmuSen_Mips.md` §9.3, §9.4); the transfers' I bit as `.i` (§6.1.1, type 11); VU0's registers in CFC2 and CTC2
  as `viN`.
- **`src/float.rs`**: `to_fixed` and `from_fixed` (FTOI0–15, ITOF0–15, §2.1.2, §2.2, §2.3); `Fused::product`, the term as
  added with its own overflow and underflow, which the FPU's SU takes as before and the VU's sticky flags take whole.
- **`src/vu/`**: `mod.rs` (`Vu`, `Units`, `Vpu`, the control registers, the EE's map of the four memories at 11000000h),
  `ops.rs` (the operations and their pipelines), `micro.rs` (`Micro`, `Unbuilt`).
- **`src/ee/cop2.rs`**: COP2's instructions and what the timing model is told of each; `ee/timing.rs`: COP2's six rows,
  VU0's registers 1 to 31 as scoreboard resources, VU0's divider, the stall after a transfer to VU0, an instruction's
  earliest cycle (an interlock) and an instruction nothing pairs with (VWAITQ); `Ee::planned`, the cycle the instruction
  being executed would issue in, which COP2 needs for the interlocks and for Q.
- **`Bus::vpu`** (default none, which halts a COP2 instruction, as before S12); `Board` owns the `Vpu`, maps its
  memories and brings micro mode to its time with the other units.
- **No new cross-component dependency**: `sapphire` still depends on `emusen-native` and `emusen-mips` alone.

### 21.3 The VU float: what it shares with the FPU, and where it parts

[R *VU User's Manual* §2, §3.3, §4.2, §4.3, §6.2; *EE Core User's Manual* chapter 8; A]:

| Rule | VU | FPU | Shared? |
|---|---|---|---|
| The format: no NaN, infinity or denormal; exponent 255 an ordinary exponent; exponent 0 zero | §2.1.1's table, "x 2(+128)" | CORE §8.1.1 | **shared**, `float`'s numbers |
| Overflow ±MAX, underflow ±0 | §2.3 | Table 8-4 | **shared**, `float::finish` |
| The adder: no sticky bit, one bit kept | §2.2's words, the FPU's | D-25 (console) | **shared**, argued (D-43) |
| The multiplier: ft recoded as Booth digits, lowest column lost | every multiply page's Remarks: "By using VF[fs] as a multiplicand, the results of multiplication with 1 are guaranteed" | D-26 (console) | **shared**, the VU manual corroborating D-26 (D-43) |
| DIV and SQRT to the nearest; RSQRT over the rounded root, truncated | §2.2 says truncation, as CORE did | D-27, D-28 (console) | **shared**, argued (D-43) |
| The multiply-adds' values: product overflow wins, an ACC of exponent 255 kept | §3.3.6's MADD and MSUB tables | D-30 | **shared**, `float::fused` |
| MAX and MINI on sign and magnitude | their pages | D-31 | **shared** (D-47) |
| The flags an FMAC result sets | MAC per lane Z, S, U, O; status collecting them; sticky; a multiply-add's product into the sticky flags (§3.3) | FCR31's O, U and sticky; SU from the product | **differ**: the VU's in `vu/ops.rs` (D-44) |
| DIV's, SQRT's flags | I for 0/0, D otherwise; SQRT clears D | the same (D-32) | shared in effect |
| RSQRT's flags | 0/√0 raises neither (§3.3.2's "except 0/0") | D for any zero ft (its page) | **differ** (D-46) |
| Conversions | FTOI and ITOF with 0, 4, 12, 15 fraction bits; FTOI clamps to ±MAX with no flag | CVT.W.S, CVT.S.W | **shared**, `to_word`'s clamp and `from_word`'s truncation, scaled |
| The compares | CLIP: values, every zero equal (D-49) | C.cond.S: the same order | **shared**, `float::order` |

So the units share every rule of the arithmetic and part only in the flags; the sharing of the
adder, the multiplier's column and the divider's rounding with the FPU's measured behaviour is the open question (D-43),
one constant or function in `float.rs` each, as at S9.

**Readings of the instructions** [R the pages; A]: VF00 and VI00 keep their constants against every write; an integer
field naming 16 to 31 reads zero and writes nothing; OPMULA and OPMSUB work on x, y and z whatever the mask; ILWR with
more than one field takes the lowest; ISWR zero-extends; MFIR sign-extends; MTIR takes the low 16 bits; RGET and RNEXT
write 0 01111111 then R; LQI and SQI post-increment and LQD and SQD pre-decrement the index register, wrapping at 16 bits.

### 21.4 The gate: the programs (measured 2026-10-25)

`tests/s12/` generates seven programs in S6's form (entered at 00100000h, the record from 00200000h, the done word at
001FFFF0h). Each sets Status.CU2, clears the divider's flags with VSQRT of +0, sets known FMAC flags by an addition into
VF00, and clears the sticky flags; each case loads vf1, vf2 (and ACC by VADDA with −0, I and Q by CTC2) from constants by
LQ and QMTC2, puts a sentinel in vf3, clears the sticky flags, runs the instruction, and records vf3 or ACC by QMFC2
(ACC read by VMADDx with a product of −0) and the status and MAC flags by CFC2. One lane carries one input case. The
expected record comes from `tests/s12/manual.rs`, the VU pages transcribed apart from `src/vu`, `src/ee/cop2.rs` and
`src/float.rs`, a lane's arithmetic being S9's transcription of chapter 5 (`s9::manual`), the flags the VU's.

| Program | Record words | What it covers |
|---|---|---|
| `s12-add` | 8,704 | VADD, VSUB, VADDA, VSUBA over every ordered pair of S9's 32 classes; their broadcast, I and Q forms with field masks; VADD under each of the sixteen masks |
| `s12-mul` | 4,288 | VMUL and VMULA over every ordered pair (both orders of the multiplier); the variants |
| `s12-fused` | 2,688 | VMADD, VMSUB, VMADDA, VMSUBA over eight accumulators and every pair of S9's twelve; the variants; VOPMULA then VOPMSUB over vectors of the classes |
| `s12-misc` | 650 | VABS, the eight conversions, VMOVE and VMR32 under masks, VMAX and VMINI with their forms, VNOP, VMFIR, VMTIR, CLIP's judgements, the flags read after each group, unchanged |
| `s12-fdiv` | 8,340 | VDIV and VRSQRT over every pair in fields chosen by fsf and ftf, VSQRT of every class, Q and the status flag after VWAITQ; Q read 0, 1, 5, 6 and 7 VNOPs after VDIV |
| `s12-int` | 1,144 | VIADD, VISUB, VIAND, VIOR over ten values, VIADDI over its immediates; VLQI, VLQD, VSQI, VSQD, VILWR, VISWR on VU0's memory written and read by the EE at B1004000h; the mirror past 4 KiB; VU1's VF05 and VI03 through the window; RINIT, RGET, RXOR, RNEXT |
| `s12-transfer` | 312 | QMFC2 and QMTC2 with and without the interlock, VF00 and GPR 0; CFC2's sign extension; CTC2 and CFC2 of every control register but TPC and CMSAR1, all ones then zero; LQC2 and SQC2 at three offsets; the four branches with VU1 idle; COP2 unusable for seven instructions and BC2T, CE 2; LQC2's and SQC2's address errors |

**Tests** [M, `cargo test --release`]: `tests/vu0.rs` **14 of 14** — the seven programs, each equal to its record word for
word; `the_programs_execute_every_macro_instruction` (the 120 operations and the 10 transfers and branches, VCALLMS and
VCALLMSR excepted, from the assembled words through the decoder); `the_manual_s_examples_and_the_console_lines_hold` (the
FTOI and ITOF Remarks' tables, `vu/lower/random`'s RNEXT steps and RXOR's line); `vcallms_starts_micro_mode_and_the_ee_waits`
and `micro_mode_unbuilt_halts_the_interpreter_at_its_start` (§21.6); `the_macro_timing_follows_section_5_4` (§21.5);
`the_board_maps_the_vu_memories_at_11000000h`; `vpu_stat_shows_the_macro_divide_busy`. The crate's other tests unchanged:
lib **18 of 18**, `cop0` **19 of 19**, `dmac` **17 of 17**, `fpu` **9 of 9** (its count of COP1 bounded to 34),
`interpreter` **23 of 23**, `mmi` **13 of 13**, `programs` **3 of 3**; `vu0` and lib also under `test-checked`; clippy
clean. `emusen-mips` **19 of 19**, one new (`the_manual_s_macro_pages_read_back`). `blast.sh`'s crates: `blackmoon-gte` **10 of 10**,
`blackmoon-r3000` **38 of 38**, Diamond **306 of 306**, its `precision` **8 of 8** and `v1` **5 of 5**, `Platform/systems`
**39 of 39**; WiseMan with `blast.sh`'s filter (`CheatCodec`, `DebugCodeParity`, `Diamond`, `SapphireTests`,
`SystemCodeParity`, `SystemTableParity`) **116 of 116**.

### 21.5 The timing model (§5.2, Q14)

[R *VU User's Manual* §5.4.1 to §5.4.7, §7.4; *EE Core User's Manual* Tables 1-1 to 1-3; A]:

1. **Categories.** QMFC2, QMTC2, CFC2, CTC2, LQC2 and SQC2 are COP2 Move (pipe 1), every VU operation COP2 Operate (pipe
   0), the BC2 forms branches; a COP2 Move beside a COP2 Operate is Table 1-3's Y cell.
2. **Hazards by register number** (§5.4.2): VF *n* and VI or CCR *n* are one resource; VF00 and VI00 none; ACC, I, Q and R
   none. An FMAC result reaches a macro instruction four cycles on and a transfer from VU0 five (§5.4.3, §5.4.4: the
   reader's T stage is one later for a macro instruction than for a transfer); an integer result the next cycle; a
   transfer to VU0 two cycles on for a macro instruction and three for a transfer from VU0 (§5.4.4's "stalls for two
   cycles"), and the macro instruction after it always one cycle late. The flags are written at the S stage, so CFC2 of
   CCR[2,16] after an FMAC instruction waits five cycles.
3. **The divider** (§5.4.2): VDIV and VSQRT take it 7 cycles, VRSQRT 13; the quotient reaches a macro instruction issued
   7 or 13 cycles after the divide (D-51); VWAITQ issues then, alone.
4. **The interlocks** (§1.3.2, §5.4.5, §5.4.6): a macro instruction, VCALLMS and VCALLMSR, QMFC2.I and CFC2.I wait for a
   running microprogram's end; QMTC2.I and CTC2.I for its first pair with M set (D-54); QMFC2.NI, the other forms without
   the bit, LQC2 and SQC2 do not wait.

**Pinned** [M, `the_macro_timing_follows_section_5_4`], each a figure's count: dependent FMAC 4; FMAC to QMFC2 5; a hazard
by number across fields 4; VF01 to VI01 4; integer 1; QMTC2 to an independent macro instruction 2; QMTC2 to QMFC2 of the
same register 3, of another 1; VDIV to VDIV 7; VRSQRT to VSQRT 13; VDIV to VWAITQ 7 and to the reader 8; FMAC to CFC2 of
the status flag 5.

**Named losses** [A]: §5.4.4's figures of CFC2 and CTC2 on the status flag show one stall more than the S stage gives;
VCALLMS's one-cycle stall after a macro instruction (§5.4.5) is not charged; BC2's "2/2" of §7.4 is charged as the EE's
branch row; LQC2's cache miss stalls as an ordinary load's, not as the always-blocking COP2 load CORE §6.3.2 describes;
the micro instruction overlap with the next macro instruction's T stage is S13's, through `Micro::finish`.

### 21.6 The micro-mode interface

`vu::micro::Micro` (S12 defines it, S13 implements it): `start(units, addr, at)` begins a microprogram at a doubleword
address at an EE cycle and answers whether micro mode exists; `run_to(units, t)` catches it up; `busy()`; `finish(units,
at)` runs it to its end and gives the cycle; `release(units, at)` the cycle of its first pair with M set; `force_break`,
`reset` and `stat` (VPU-STAT's bits) for FBRST and CCR[2,29]. The EE calls `finish` or `release` with the cycle its
instruction would issue in and waits until the cycle returned; the board calls `run_to` after every instruction, as for
the other units. `Unbuilt`, until S13, never runs and refuses a start, so VCALLMS, VCALLMSR and a write of CMSAR1 halt the
interpreter with the word, as an instruction of a later stage did before S12. The tests drive a scripted stand-in: QMFC2.NI
reads before the microprogram's end and QMFC2.I after it; a macro instruction and CFC2.I wait for the end; QMTC2.I and
CTC2.I for the M bit, QMTC2.NI not at all; VCALLMSR starts at CMSAR0; a second VCALLMS waits for the first.

### 21.7 On the referees (measured 2026-10-25)

`examples/s12.rs` writes each program as an ELF with its expected record; `ps2referee.py ram … --until 1FFFF0=5A99D00E
200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, one process at a time.
`s12 explain` sorts the lanes that differ by the class of their inputs and the flags by bit. Only the records were read.

| Program (graded words) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s12-add` (8,704) | 1,577 | 1,668 | exponent 255 (host NaNs and clamps); sums and differences in IEEE 754's rounding towards zero (D-43); no U or O (D-44) |
| `s12-mul` (4,288) | 1,156 | 1,106 | exponent 255; exact commutative products (D-43); no U or O |
| `s12-fused` (2,688) | 816 | 901 | the same through the multiply-adds; no product into the sticky flags (D-44) |
| `s12-misc` (650) | 4 | 44 | PCSX2: VMR32 with the empty mask writes every lane (D-53). Play!: FTOI's positive overflow 80000000h, CLIP's sign and width (D-49), one maximum (D-47) |
| `s12-fdiv` (8,340) | 2,787 | 1,855 | exponent 255; quotients and roots truncated (D-43); the divider's flags (D-46); Q landing at once (D-51) |
| `s12-int` (1,144) | 4 | 5 | PCSX2 keeps 128 bits in VU1's VI03 through the window; Play! maps no window (D-50) |
| `s12-transfer` (288) | 66 | 57 | the control registers' widths (D-45); the flags through FBRST's reset (D-52); neither raises COP2 unusable or LQC2's and SQC2's address errors, as at S6; Play!'s BC2FL and BC2TL do not nullify their slots |

Across `s12-add`, `s12-mul` and `s12-fdiv`, on every lane whose operands have exponent fields 1 to 253 and whose result is
in range (2,677 lanes), both lineages equal IEEE 754's rounding towards zero and nothing else [M]; 730 of those lanes
separate it from the module. So **neither lineage is a witness for the VU's arithmetic or its U and O flags**, as neither
was for the FPU's (§19.6); PCSX2 is one for the transfers, the integer unit, the random unit, CLIP, MAX and MINI, the
branches and the memories, Play! for the integer and random units.

### 21.8 ps2autotests' `cpu/vu0_macro` on PCSX2, and what the `vu/*` texts decide (measured 2026-10-25)

Both programs call the BIOS's kernel, so Sapphire runs them at S10. Run now through `ps2referee.py tty pcsx2 …
--channel iop` (Americas BIOS) and compared with their expected texts, the consoles' outputs, after line ends are
normalised [M `diff`]: **`integer` (118 lines) equal**; **`transfer` (36 lines) differs in eight**: `vi18`, `vi19`,
`vi23`, `vi24`, `vi25` and `vi27` read FFFFFFFFh, `vi28` 00000C0Ch and `vi31` 0 where the console reads the values D-45
gives, which `s12-transfer` reproduces on the interpreter; its TPC line and its other lines equal. PCSX2's own log lines
in the channel were not kept.

The VU micro tests' console texts are S13's; read now for the shared rules: `vu/lower/random` decides RNEXT's series
(D-48); `vu/lower/fdivdelay` divides only powers of two, so it decides nothing of the divider's rounding; `vu/upper/clip`
prints judgements of named vectors whose values a run must show before it can grade D-49; `vu/games/triace` prints a
game microprogram's twelve results, the first VU arithmetic a console records, which S13 can grade D-43 with.

### 21.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | an overflow left at exponent 255 (7F800000h, IEEE 754's infinity) instead of clamped to ±MAX, in `float::finish` | `add_and_subtract_…`, `multiply_…`, `the_multiply_adds_…`, `divide_square_root_…` |
| F2 | the MAC flags' S bit left clear for a negative result in lane w | `add_and_subtract_…`, `multiply_…`, `the_multiply_adds_…` |
| F3 | QMFC2.I not waiting for a running microprogram | `vcallms_starts_micro_mode_and_the_ee_waits` |

Each was applied, `tests/vu0.rs` run, and the source restored [M]. Found while building, not seeded: CFC2 of Q issued
beside VWAITQ read the old quotient, which `s12-fdiv` exposed and §5.4.7's figure settles (VWAITQ alone, D-51).

### 21.10 What the plan got wrong, or left open

- **§1.1's VU row and §5.3** speak of the FPU and the VUs together; the arithmetic is shared, but the sharing of the
  FPU's console-decided last bit is argued (D-43), and the flags part (D-44, D-46).
- **§3.2's VU row** names the referees as the secondary oracle: for the arithmetic and U and O they are not (§21.7); for the
  transfers and the integer and random units PCSX2 is.
- **S12's gate**, `cpu/vu0_macro/*`, needs the BIOS's kernel: met on PCSX2 against the console texts, Sapphire's at S10;
  `transfer`'s console text gave the control registers' widths that no document gives (D-45).
- **§15.6 Q1** (the micro-only encodings): Sony's *VU User's Manual*, fetched at S6, gives every macro operand form; S12
  used them; the micro-only lower instructions' encodings (§4.3) are S13's, as `Sapphire_References.md` §1.1 says.

### 21.11 Questions this stage leaves

1. **Q1 [routine], the VU's arithmetic (D-43).** *Recommended*: keep the FPU's console-decided readings for the VU, one
   owner, until S13 grades `vu/games/triace`; the referees' IEEE 754 towards zero is their host's and stays the
   alternative, three constants away.
2. **Q2 [routine], RSQRT's 0/√0 (D-46).** *Recommended*: keep §3.3.2's words (neither flag), the three lineages and
   readings disagreeing.
3. **Q3 [routine], Q's delay (D-51).** *Recommended*: keep the figures' delay; S13's `fdivdelay` console text measures
   the micro form of the same rule.
4. **Q4 [routine], C3 (§5.13).** *Recommended*: when C3 is weighed, count the VU lanes per class where a host's rounding
   towards zero differs from `float` (730 of 2,677 in-range lanes of these programs, §21.7) against VU1's speed.

### 21.12 What was read for this stage

`Sapphire_Plan.md` §1.1, §2, §3.2, §5.3, §5.13's C3, §7 and §11; this page's §15, §19 and §20; `Sapphire_Disputes.md`
D-0 to D-42; `Sapphire_References.md` §1.1; `EmuSen_Mips.md` §8 and §9. Sony's *VU User's Manual* (Version 5.0) chapters
1, 2, 3 (§3.1 to §3.4.5), 5 and 6 whole, §4.1, §4.2 and §4.3's pages of the operations macro mode takes, §7.4; *EE Core
User's Manual* §1.3.4, §2.10.2, §2.10.3, §6.3.2 and Table 7-1's COP2 event; the *EE User's Manual*'s memory map. ps2autotests'
`cpu/vu0_macro/*.expected` and `vu/*/*.expected` (the consoles' texts, not the programs' source). No emulator, SDK or
loader source, no ps2autotests source and no firmware.

### 21.13 What this stage does not cover

- VU micro mode, both units: S13 (the micro-only encodings, the pipelines, XGKICK, E, D and T, the EFU); VCALLMS,
  VCALLMSR and CMSAR1 halt until then.
- `cpu/vu0_macro/*` on Sapphire: S10.
- The DMAC's and VIF0's reach into VU0's memories (S14); the scratchpad channels' burst to them.
- COP2 in the recompiler: S38–S39; C3's host path: S39–S41.
- The state format's new contents (both units, the control registers): `SPH2` gains them with the rest (S10, S30).

## 22. Stage S21, the IOP on `blackmoon-r3000` (2026-10-25)

*§21 is stage S12's, built beside this one on its own branch; the IOP needs nothing of the VUs.*

### 22.1 In one paragraph

The plan's §7 row S21, on the tester's go of 2026-10-25. The crate gains **`iop`**: the IOP, `blackmoon-r3000`'s
R3000A used as its crate's design intends (`BlackMoon_R3000.md` §2), nothing moved out of the crate, on Sapphire's own
IOP bus, which holds the IOP's memory map (2 MiB of RAM with psx-spx's mirrors, the boot ROM as an empty slot for S24,
the SIF's registers for S22, the I/O regions, the cache control in KSEG2), a 4 KiB I-cache and 1 KiB scratchpad (the
PlayStation's sizes, a named reading, D-62), the wait states (D-63), and the IOP's units: the **INTC** with I_CTRL, the
**DMAC** with its 13 channels in burst (with chopping), slice, linked-list and chain modes, DPCR2, DICR2, DMACEN and
DMACINTEN, ending at `iop::dma::Peripherals` for the channels whose peripherals come later, OTC working end to end, and
the **six timers**, three of 16 bits and three of 32, on the IOP's clock, its prescalers, the pixel clock and H-blank,
with their gates and request modes. The IOP runs at an eighth of the EE's clock (`clock::IOP`) and joins S11's board as
a unit with its own next event; with no firmware it is held until a program is entered. PRId is **1Fh** (D-60): the
plan's "at least 59h" misread ps2tek, whose BIOS chapter puts the IOP below 59h and at or above 0Fh. Seven bare IOP
programs of the project's, 174 graded record words, equal on Sapphire the records ps2tek and psx-spx give. **A referee
can run an IOP program**: an EE program copies it into IOP RAM through the EE's window at 1C000000h and points the
IOP's exception vector at it, so that the IOP's next interrupt enters it; this works on PCSX2 and not on Play!, whose
EE has no window onto IOP RAM. PCSX2 differs on 41 of the 153 words its five programs grade, chiefly where it models no
IOP cache, wait state, bus error or register width; the disagreements are D-60 to D-68, and one about the processor
itself is `BlackMoon_R3000.md` D-22. `blackmoon-r3000`'s suite passes unchanged (38 of 38, the corpus 55,000 of
55,000). No emulator, SDK, loader or ps2autotests source and no firmware was read.

### 22.2 What was built

- **`iop/mod.rs`**: `Iop`, the R3000A with PRId 1Fh and processor number 1 beside `IopBus`; `enter` releases it at a
  program in kernel mode with SR clear; `run_to` runs it to an EE cycle by `R3000::run`, each run bounded by the bus's
  next event, the units serviced between runs; while held its clock follows the board's and nothing runs.
- **`iop/bus.rs`**: `IopBus`, the crate's `Bus`: `decode` (§22.3), the I-cache's fetch with its refill, the access
  times, the I/O registers dispatched to the units (registers of later stages reading zero), the isolated calls to the
  cache, coprocessor 2's calls answering zero (the IOP has no GTE in PlayStation 2 mode, `Sapphire_Plan.md` §9.2),
  `bus_error`, `running`. The units' events are kept in EE cycles and taken in time order by `run_units` (blanking
  edges, timer requests, the DMAC's units); before each run `service` brings them to the clock and holds the processor
  while the DMAC has the bus.
- **`iop/icache.rs`**: the I-cache and BCC (§22.3). **`iop/intc.rs`**: §22.4. **`iop/dma.rs`**: §22.5, with
  `StandIns` for the channels 0 to 12 until their stages. **`iop/timer.rs`**: §22.6.
- **`clock.rs`** gains `IOP` (8 EE cycles an IOP cycle) and `PIXEL` (8,192/375 EE cycles a pixel-clock tick).
  **`board.rs`** gains the field `iop`, brought to the board's time at the end of each `run_to`, and the EE's window
  onto IOP RAM at 1C000000h-1C1FFFFFh [R ps2tek "Memory Map"], answered before the physical map. Both changes are
  additions; `lib.rs` declares the module.
- **A new dependency**: `sapphire` now depends on `blackmoon-r3000` (with `emusen-native` and `emusen-mips`), as
  `Sapphire_Plan.md` §9.2 and the tester's Q10 decided; no crate it depends on was changed.

### 22.3 The memory map, the I-cache and the access times

[R ps2tek "Memory Map" and "I/O Maps"; psx-spx "Memory Map", "i-Cache", "FFFE0130h - BCC"; A where marked]:

- **Segments**: KUSEG's first 512 MiB, KSEG0 and KSEG1 decode to one physical space; KUSEG above it and KSEG2 but the
  control registers meet no device, a bus error.
- **RAM**: 2 MiB, mirrored through the first 8 MiB (psx-spx's default; ps2tek's "same as on PSX"); past it a bus error
  (D-61).
- **Scratchpad**: 1 KiB at 1F800000h in KUSEG and KSEG0 only, not executable; KSEG1's and the gap up to 1F801000h a bus
  error (D-61). The I-cache's "minor enhancements" are unread (D-62).
- **I/O**: 1F400000h-1F4FFFFFh (CDVD's registers), 1F801000h-1F80FFFFh (the INTC, DMAC and timers now; SIO2, DEV9 and
  the rest later) and the SPU2's 1F900000h-1F9007FFh answer, registers of later stages reading zero and taking writes
  without effect [C]; a fetch from them is a bus error [A, psx-spx: the PlayStation executes only from its DMA and SPU
  registers]. A byte or halfword store to a register is shifted to its lane, as Diamond's D-45.
- **SIF** at 1D000000h-1D0000FFh: zero, S22's. **Boot ROM** at 1FC00000h, 4 MiB: empty, reading zero, until S24 gives
  it the player's image (the EE's ROM and the IOP's being one, ps2tek "Same as EE BIOS"). **Cache control** at
  FFFE0130h; FFFE0000h-FFFE01FFh's other addresses read zero [C].
- **The I-cache** (D-62): 256 lines of four words, a tag of the physical address's bits 31-12 and a valid bit a word;
  KUSEG and KSEG0 cached while BCC's IS1 is set, KSEG1 never; a miss fills from the word to the line's end, two words
  from word 0 with IBLKSZ 0; a tag that matches with the word invalid refills the line; stores never touch it; with SR's
  IsC a store writes the tag (TAG set) or the word, and a load reads them, TAG's match in bit 4. BCC keeps every bit but
  6 and 10; it is 0 at power-on, the cache off [C], and the project's programs write the PlayStation kernel's 1E988h.
- **The access times** (D-63), in IOP cycles beyond the instruction's own, which a fetch carries: a cache hit 1 in all;
  an uncached fetch from RAM 1 + 4, a load from RAM 4, from an I/O register 2, from the boot ROM 6 [C]; a refill 1 + 4
  for its first word and 1 for each further one, as Diamond's [C]; the scratchpad and every store nothing more (no
  write queue).

### 22.4 The INTC

[R ps2tek "IOP Interrupts"; psx-spx "Interrupts"]: 26 sources (VBLANK start 0, the GPU's 1, CDVD 2, DMA 3, timers 0 to
2 on 4 to 6, SIO0 7, SIO1 8, SPU2 9, PIO 10, VBLANK end 11, 12 and 13, timers 3 to 5 on 14 to 16, SIO2 17, 18 to 25);
a request sets its I_STAT bit masked or not; I_STAT's bits written 0 are acknowledged, those written 1 kept; I_MASK
keeps 26 bits, I_CTRL one; a read of I_CTRL returns it and clears it; bits 26 to 31 ("garbage") read zero [C]. The line,
`I_CTRL AND (I_STAT AND I_MASK)`, is the processor's Int0, Cause's IP2, enabled by SR's IM2 (D-65). At this stage the
blanking stand-in raises VBLANK start and end (its V-blank's rise and fall), the DMAC IRQ 3, the timers their six;
`Intc::raise` is the interface the later peripherals call.

### 22.5 The DMAC

[R ps2tek "IOP DMA"; psx-spx "DMA Channels"; A where marked; D-66, D-68]:

- **Registers**: per channel MADR and TADR (24 bits), BCR (32), CHCR (ps2tek's bits; OTC's three, its bit 1 always
  set); channels 0 to 6 at 1F801080h, 7 to 12 at 1F801500h. DPCR (reset 07777777h, ps2tek) and DPCR2 (reset 0 [C]) give
  each channel three bits of priority and an enable; DICR keeps its per-block enables, bus error flag, masks and master
  enable, its flags cleared by writing 1, its bit 31 the master flag; DICR2 its tag enables (channels 4, 9 and 10 only),
  masks and flags; DMACEN and DMACINTEN their documented bits, both 0 at reset [C].
- **Starting**: a channel runs while its START is set, DPCR or DPCR2 enables it, DMACEN's bit 0 is set and either its
  trigger (bit 28, cleared as the transfer begins) or its peripheral's request; in burst mode bit 29 with the trigger
  pauses it. The ready channel with the best priority goes first, DPCR's and DPCR2's three bits on one scale, the higher
  channel first on a tie [A].
- **Modes**: a **burst** moves BC words at once (0 meaning 10000h), MADR and BCR left as written; with **chopping**, in
  windows of 2^N words with 2^M cycles for the processor between them, MADR and BC updated to the end. A **slice** moves
  BS words a block, BA blocks, MADR and BA updated, a whole block needing the peripheral's request. A **linked list**
  moves each node's words after its header and follows its address, ending at one with bit 23 set. A **chain** reads a
  two-word tag at TADR (address, IRQ and end bits; a size in words), with CHCR's bit 8 sends the tag's next two words to
  the peripheral first, moves its words from the address, TADR stepping over the tag, and ends at a tag with its end
  bit; a chain into memory takes its tags from the peripheral [A, S22's to settle]. OTC writes its reversed list, the
  last entry FFFFFFh.
- **Timing** [A]: a word a cycle on every channel; a unit (a burst, a chopped window, a block, a node, a tag with its
  data) holds the bus, and the processor waits until it ends, its effects taking place at its start and its completion
  (START cleared, the flag) at its end; a cycle's gap between blocks and nodes lets the processor in.
- **Flags**: a completion sets its channel's flag only with its mask and the master enable; DICR's per-block enables
  set it after each block or node; a chain's IRQ tag with DICR2's enable sets it regardless of the mask [A]. The master
  flag is the bus error flag or the master enable with any flag, the flags counting only with DMACINTEN's bit 0; IRQ 3
  is raised as it rises, unless DMACINTEN's bit 1 is set.
- **Errors**: an address past the 8 MiB window ends the transfer with DICR's bus error flag and no completion flag [A].

### 22.6 The timers

[R ps2tek "IOP Timers"; psx-spx "Timers"; A where marked; D-64, D-67]:

- **Registers**: count and target of 16 bits on timers 0 to 2 (1F801100h on) and 32 on 3 to 5 (1F801480h on); the
  mode's bits 0 to 9 and 13 to 14 written, 10 to 12 read-only. Writing the mode resets the count and the flags and sets
  bit 10; reading it clears the flags; writing the target sets bit 10 unless LEVL is set.
- **Clocks**: the IOP's; timer 0's external clock the pixel clock, 13.5 MHz, counted as a ratio of EE cycles; timers 1
  and 3's H-blank, a tick at each rise; timer 2 over 8 with bit 9; timers 4 and 5 over 1, 8, 16 or 256 by bits 13 and 14.
  The dividers run freely from time 0 [A, as the EE timers'].
- **Counting**: the target flag when the count reaches the target and the overflow flag when it reaches FFFFh or
  FFFFFFFFh, set whatever the enables; with ZRET the count runs to the target inclusive and then restarts, without it to
  the maximum and wraps.
- **Requests**: a target or overflow with its enable requests if bit 10 is set; one-shot then clears bit 10, toggle with
  repeat inverts it, pulse with repeat leaves it; a request sets the timer's I_STAT bit.
- **Gates**: timer 0's H-blank, 1's and 3's V-blank, in psx-spx's four modes (0 pausing in the blank, 1 resetting at its
  start, 2 resetting at both edges and counting only in it, 3 waiting for its first start); timers 2, 4 and 5 stopped by
  modes 0 and 3 and free in 1 and 2 [A for 3 to 5].
- **Evaluation**: as the EE timers', a count valid at a time advanced in closed form, event by event only while a
  request can still rise; the next request's tick is the only event a timer schedules, H-blank's edges reaching the IOP
  only while a timer counts or gates on them.

### 22.7 What is not modelled

Each a named loss, and a dispute candidate if an oracle measures it [A]:

- The processor waits through a whole DMA unit, where psx-spx lets it run from the cache and the scratchpad and queue
  up to four stores; the write queue is not modelled at all (a store costs nothing beyond its instruction).
- psx-spx's two-cycle holds of a timer after a write or a reset; the IOP's own DMA rates per channel; the cache's
  "enhancements"; BCC's lock, invalidate and scratchpad bits, and the data cache psx-spx calls broken.
- CHCR's bit 30 (snooping) and iLink's bits are kept and do nothing; DICR2's tag interrupt on a channel other than
  4, 9 and 10 cannot be enabled.
- The blanking stand-in (§20.5) gives the IOP its H-blank and V-blank, so timers 1 and 3 count 262 H-blanks a field
  until S20 (D-42, D-67).
- `machine.rs` does not yet hold the board; the IOP's state is not in `SPH2` (S30).

### 22.8 The gate: the programs (measured 2026-10-25)

`tests/s21/` generates the programs as R3000A assembly for `emusen-mips` with `blackmoon-r3000`'s dialect. Each is
entered at 00100000h through KSEG1, runs from KSEG0 (`s21-cache` stays in KSEG1, so that only its routines are cached),
puts the units in a known state first (interrupts off and acknowledged, every channel stopped, flags and masks clear,
every timer free-running, BCC at 1E988h), and writes its record of 32-bit words from 00180000h and 5A99D00Eh at
0017FFF0h. A second entry, through the exception vector, is the program's handler, which logs Cause, EPC, BadVaddr and SR
and resumes at the address the program set, interrupts disabled. Every expected word is derived in the generator from
the documents, written apart from `src/iop/`; where they give no value a word is recorded ungraded, or graded by the
range they give.

| Program | Graded words | What it covers |
|---|---|---|
| `s21-map` | 28 of 33 | PRId's range (and its value, ungraded); RAM through KSEG0, KSEG1 and KUSEG, its byte and halfword lanes and its three mirrors; bus errors past 8 MiB, at KSEG1's scratchpad, the gap after it and 1E000000h; the scratchpad's ends; BCC's bits; syscall's and break's codes and EPC |
| `s21-cache` | 18 of 21 | stale code until a flush; the 4 KiB conflict and none at 2 KiB; a refill from word 2, then the whole line; the isolated cache; cached against uncached and IS1 clear |
| `s21-intc` | 16 of 20 | I_MASK's width; I_CTRL's read that clears it; IP2 masked, unmasked and with I_CTRL clear; an interrupt with IEc alone (none) and with IM2 (taken, IP2 and code 0); I_STAT's acknowledge; V-blank's start and end and the field's and the blank's lengths |
| `s21-dma` | 44 of 45 | every register's width; OTC's list of 16 with the processor held, MADR, BCR and CHCR after; the flag with and without the mask; the bus error flag's master flag and IRQ 3; DMACINTEN 0 and 3; DMACEN and the enable holding a transfer |
| `s21-timers` | 47 of 52 | widths of count and target on all six; the 16-bit wrap with OVFF and none at 32; the mode's write; timer 2's stop and run; the four prescalers and the pixel clock against the IOP's clock; H-blank over a field on timers 1 and 3; ZRET; a repeated pulse, a one-shot re-armed by the target, a toggle; an overflow request; timer 1's V-blank gate in modes 0 and 3; timer 0's H-blank gate; timers 3 to 5's gates ungraded |
| `s21-fetch` | 3 of 3 | a fetch from the scratchpad: IBE at its address (Sapphire only, §22.9) |
| `s21-dma-modes` | 18 of 19 | slice to SPU2, a linked list to SIF2, a chain from memory to SIF0 with the tag's words first and an IRQ tag, a chain into memory from SIF1, a requested burst and a chopped one to SPU (Sapphire only: their channels reach the stand-ins) |

**Tests** [M, `cargo test --release`]: `tests/iop.rs` **13 of 13** — the seven programs, each equal to its record word
for word (the stand-ins receiving the slice's twelve words, the list's three, the chain's nine with each tag's two words
first, SIF1's tag and words taken, SPU's 5 and 32); PRId equal to D-60's; the cached loop 3,039 cycles for 1,000 passes
of three hits, uncached and IS1 clear 15,047; the IOP's clock at 50,000 cycles for 400,000 EE cycles and a loop of five
uncached instructions 2,000 passes in it; the EE writing IOP RAM through 1C000000h while a held IOP follows the clock;
on the units directly, DPCR's priorities and the tie, a channel without a request scheduling nothing until woken, a
burst's hold and chopping's windows, and the cache's refill rules. Lib **24 of 24** (the IOP's INTC and five timer
tests among them); the crate's other tests unchanged, on the tree with S12 beneath: `cop0` 19, `dmac` 17, `fpu` 9,
`interpreter` 23, `mmi` 13, `programs` 3, `vu0` 14; lib, `dmac` and `iop` also under `test-checked`; clippy clean. `blackmoon-r3000`: **38 of 38**, the r3000
corpus **55,000 of 55,000** (`EMUSEN_BLACKMOON_CORPUS`), the crate's source unchanged, so Diamond's tests, which reach
it only through it, were not re-run. `SapphireTests` (WiseMan, the blast radius's filter): **5 of 5**.

### 22.9 On the referees (measured 2026-10-25)

**Placing an IOP program.** `ps2referee.py` runs EE programs, and needs no change: `examples/s21.rs` writes each IOP
program inside an EE program (`tests/s21`'s `wrap`) that clears the IOP program's done word and handler flag through the
EE's window at BC000000h, copies the program to IOP RAM at 00100000h, and writes at IOP RAM 80h a jump to its KSEG1
entry, so that the IOP's next interrupt, on the BIOS's own kernel after fast boot, enters it at the general exception
vector; the program then owns the IOP, its first act undoing what the kernel left. The EE waits a bounded time for the
done word in IOP RAM and copies it and the record into EE RAM, where `ps2referee.py ram … --until 1FFFF0=5A99D00E
200000:<words>` reads them. Only the IOP's architectural vector is touched; nothing of the BIOS is read, traced or
patched. *Where it works*: PCSX2 2.8.2 (Americas BIOS, fast boot), every program but `s21-fetch` (§D-61). *Where not*:
Play! 0.72, whose EE reads CCCCCCCCh at BC000000h and drops the stores there (its IOP being its own high-level BIOS
besides, Q5); LRPS2 and PCEE2 are PCSX2's lineage and add no witness. So the IOP's programs have one referee, one
lineage, and the rulings that rest on it say so.

| Program (graded words) | PCSX2 | What the differences are |
|---|---|---|
| `s21-map` (28) | 12 | no bus error at any of the four loads (ten words: the counts and codes, and two later counts shifted by them); BCC reads 14000200h (D-61, D-62); RAM, its lanes and mirrors, PRId 1Fh, syscall's and break's codes and EPC equal |
| `s21-cache` (18) | 5 | no IOP I-cache (stale code, the 4 KiB conflict, the partial line); an isolated load reads memory; no wait states, the three loops 3,023, 3,024 and 3,029 cycles (D-62, D-63) |
| `s21-intc` (16) | 2 | no IP bit in Cause, pending or taken; the interrupt taken by IM2 and not by IEc alone, its field 614,405 and blank 53,837 cycles against the stand-in's 613,733 and 51,535 (D-65, `BlackMoon_R3000.md` D-22) |
| `s21-dma` (44) | 13 | every bit of CHCR, MADR, TADR and DICR kept; bit 28 left set after a transfer; a transfer run with DMACEN clear; the master flag with DMACINTEN 0 and IRQ 3 with its bit 1; the OTC list, the processor held through it, the flags and the enable's hold equal (D-66) |
| `s21-timers` (47) | 9 | timer 2 counting in gate mode 3; the pixel clock not 13.5 MHz; 525 H-blanks a field on timers 1 and 3; ZRET's largest count 8; bit 10 clear after a repeated pulse and set after a one-shot, which requests again; no H-blank gate on timer 0; the widths, wraps, prescalers, overflow, toggle, re-arming and V-blank gate equal (D-64, D-67) |

PCSX2 reproduces the IOP's register-level behaviour where it touches what the BIOS needs (the OTC list, the flags,
the prescalers, the V-blank gate, PRId) and models none of the cache, wait states, bus errors or undefined bits, so it
is a witness for the former and for nothing else here, as it was for the EE's DMAC (§20.8).

### 22.10 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | timers 0 to 2 counting to FFFFFFFFh, as 32-bit | `the_timers_widths_clocks_flags_requests_and_gates_equal_the_documents`, `sixteen_bit_timers_wrap_at_ffffh_and_thirty_two_bit_ones_do_not`, `the_prescalers_and_the_pixel_clock` |
| F2 | a burst ignoring CHCR's chopping | `each_dma_mode_reaches_its_stand_in`, `a_burst_holds_the_bus_and_chopping_opens_windows` |
| F3 | I_MASK's bit 15 not masking timer 4's request | `the_intc_masks_latches_and_drives_ip2` |

Each was applied, `tests/iop.rs` and the lib's IOP tests run, and the source restored [M].

### 22.11 What the plan got wrong, or left open

- **§9.2's PRId**: "the BIOS chooses the PlayStation 2's boot path when PRid >= 59h", so "at least that", reverses
  ps2tek's sentence: 59h and above is the EE's path; the IOP's is below it, and at least 0Fh not to take TBIN's (D-60).
  Amended in §9.2.
- **§3.2's referees for the IOP**: the plan did not say how a referee runs an IOP program. One can, through an EE program
  and the IOP's exception vector, on PCSX2 only (§22.9).
- **§7's S21 gate** names "the r3000 suite still passes"; the crate is unchanged, so the gate is met by construction and
  confirmed by running it.
- **The caches**: §9.2 and `Sapphire_References.md` §5.1 give no IOP data cache; the stage's "instruction and data
  caches" are the I-cache and the scratchpad, the PlayStation's data cache serving as the scratchpad (psx-spx).

### 22.12 Questions this stage leaves

1. **Q1 [routine], the IOP's PRId (D-60).** *Recommended*: keep 1Fh, PCSX2's value inside ps2tek's range, until S24's
   BIOS shows otherwise (a PRId it rejects would send the IOP down TBIN's path, which S24 would see).
2. **Q2 [routine], the IOP's cache size (D-62).** *Recommended*: keep the PlayStation's 4 KiB; `s21-cache`'s 4 KiB pair is
   the probe for an 8 KiB or set-associative cache if a lineage ever models one; otherwise S28's games decide.
3. **Q3 [routine], bus errors on the IOP (D-61).** *Recommended*: keep psx-spx's, since PCSX2 models none and the BIOS
   meets none at S24 if it never strays; revisit if S24's BIOS takes one.
4. **Q4 [decision], sharing the PlayStation's units.** The IOP's I-cache, its timers 0 to 2 and its DMA channels 0 to 6
   are the PlayStation's, which Diamond already models in its own bus (`icache.rs`, `timers.rs`, `dma.rs`); S21 wrote
   Sapphire's own, as the plan's §9.2 decided ("What moves: nothing"), and they differ (13 channels and DICR2, 32-bit
   timers and prescalers, the IOP's clock ratios). *Recommended*: leave them apart until S24 and S28 show what the IOP's
   really are; then consider a `blackmoon` crate for the R3000A's I-cache alone, the one part that is the same chip's,
   with Diamond's goldens as the gate.
5. **Q5 [routine], interrupt timing on the referee (D-65).** PCSX2 tests the IOP's interrupts only at intervals, so a
   program's window must be longer than four instructions to see one. *Recommended*: the project's programs keep a
   window of a hundred passes; no change to Sapphire, which tests at every instruction as the processor does.

### 22.13 What was read for this stage

`Sapphire_Plan.md` §2 (the IOP and SIF rows), §3.2, §5.1, §7 (S21 and S22), §9.2, §11; this page's §20;
`Sapphire_Disputes.md` (the rules, D-0, D-38, D-42); `Sapphire_References.md` §3 and §5.1; `BlackMoon_R3000.md` §2, §5.5,
§6 and §8; `Diamond_Native.md` §11.2-§11.4, §11.6 and §11.7 (the project's own PlayStation readings, for their form and
the measured access times); Diamond's `bus.rs` (its `Bus` implementation and DMA hold) and `icache.rs`'s header (the
project's own code, for the crate's use); `blackmoon-r3000`'s `lib.rs`, `exec.rs`'s step and `cop0.rs`; `ps2referee.py`'s
usage. ps2tek at `1c91660`: "PS2 Overview", "Memory Map", "I/O Maps", "IOP Interrupts", "IOP DMA", "IOP Timers" (to line
4181, stopping before "IOP Console", which DOCS.txt X.2 excludes for every writer), "EE Cache"'s coherency passage (lines
1090-1108), and the two PRId sentences of the BIOS chapter, lines 4616-4625 (in "BIOS Boot Process", excluded for
replacement writers by X.1, so this writer is not one) and 5090-5101; psx-spx at `6d7d1bc`: "Timers", "DMA Channels",
"Memory Map", "Memory Control"'s BCC. No emulator, SDK, loader or ps2autotests source and no firmware; the BIOS was only
passed to PCSX2 by path.

### 22.14 What this stage does not cover

- **ps2autotests' `cpu/iop/*` (7 programs) and `kernel/iop/*` (16)**: they are IRX modules that the BIOS's IOP kernel
  loads and links, so they run with S24's BIOS (or the open BIOS's O5-O8); recorded for S24's gate.
- The SIF's registers and its three channels on both sides (S22), CDVD (S23), SIO2 (S25), the SPU2 (S26), DEV9 and USB
  (left out, §1.2), each reaching the IOP through `Intc::raise`, `iop::dma::Peripherals` and `IopBus`'s I/O dispatch.
- The IOP in PlayStation mode (Diamond's, §4.1), its 33.8688 MHz clock and TBIN.
- The machine running the IOP with the EE (S10, S24), the IOP's state in `SPH2` (S30), its debugger view (S31) and its
  speed (S32 to S34).
## 23. Stage S13, VU micro mode, both units (2026-10-25)

*Numbered §23 because stage S21 (the IOP), built beside it, writes §22; its disputes start at D-80 for the same
reason.*

### 23.1 In one paragraph

The plan's §7 row S13, on the tester's go of 2026-10-25. `emusen-mips` gains the micro-only lower instructions,
§4.3's 46 operation codes read from the *VU User's Manual*: the 28 whose bits 31–25 select them (LQ, SQ, ILW, ISW,
IADDIU, ISUBIU, the twelve flag instructions, B, BAL, JR, JALR and the six conditional branches) in a table of their
own, and MFP, XTOP, XITOP, XGKICK, WAITP and the eleven EFU instructions in the 1000000 class's empty rows, none of them
a COP2 word; its VU assembler takes labels. Sapphire gains **`vu::pipe::Pipe`**, micro mode for each unit behind S12's
`vu::micro::Micro`: a pair of instructions issued a cycle at its T stage, held back only by §3.4's hazards (a register
field written by the FMAC pipeline, an integer register loaded by ILW or ILWR, the divider and the EFU busy, WAITQ,
WAITP, PATH1 busy behind an XGKICK), every result computed as its pair issues, and what the pipelines make visible later
modelled where no hazard check hides it: the MAC, status and clipping flags written at the S stage four cycles on, Q at
the divider's F stage, P at the EFU's, and an integer result that a branch in the very next pair does not yet see. The
E bit and its delay slot, the M bit, the D and T bits with FBRST's enables, TPC, VPU-STAT's bits and the D and T
interrupts to the INTC are §1.3's, §3.5's and §7.3's; XGKICK ends at a defined interface (`vu::Path1`, a stand-in until
S15), XTOP and XITOP read a link VIF drives at S14, and VIF's MSCAL family will start a unit through `Vpu::start`. VU1's
EFU is **`vu::efu`**, §7.2's nano code on `sapphire::float`. VCALLMS, VCALLMSR and CMSAR1 no longer halt the
interpreter. Seven bare programs of the project's (1,948 record words over 487 result quadwords, every micro instruction
but the three that reach outside the VPU) and 24 transcriptions of the manual's pipeline figures and rules equal the
manual on the interpreter. PCSX2 reproduces six of the seven `vu/*` console texts whole but for its own log lines; its
EFU and its arithmetic miss the console in `vu/lower/efu` and in one of `vu/games/triace`'s twelve values, while the
EFU's nano code on the format's arithmetic equals 193 of `efu`'s 208 console values and is within two in the last place
of the rest. Disputes D-80 to D-85; D-43 stands with console evidence beside its argument, and D-51's micro form is
measured. No emulator, SDK or loader source, no ps2autotests source and no firmware was read.

### 23.2 What was built

- **`emusen-mips`** (`vu.rs`): `Unit::Micro`; `LOWER`, the lower instructions by bits 31–25; the 1000000 class's micro
  rows; `lower(w)`, a lower word's operation in micro mode, beside `operation(w)`, which still decodes COP2 and the
  upper word and leaves the micro-only operations out; the forms of §4.1.2's types 7, 8 and 9 and the EFU's; an
  operation's fixed bits per operation (the EFU's vector instructions fix their field mask); `imm11`, `imm12` and
  `imm15` as §4.1.2 splits them; labels in `vu::assemble`, a branch's offset counted from its delay slot as IBEQ's page
  counts it. The zero lower word now reads as `lq vf0, 0x0(vi0)`, LQ's operation code being 0000000.
- **`src/vu/pipe.rs`**: `Pipe`, one unit's micro mode; `START` and `FMAC`, the two pipeline constants the EE's side
  shares.
- **`src/vu/efu.rs`**: the eleven elementary functions as §7.2's nano code computes them (§23.4).
- **`src/vu/ops.rs`**: the 46 micro-only operations; `Ctx::micro`, under which an operation hands its flags back in
  `Deferred` instead of writing them (S12's macro mode still writes them at once); the classes `Fcheck`, `FlagSet`,
  `Efu`, `WaitP`, `Branch` and `External`.
- **`src/vu/mod.rs`**: Q and P in flight (`Landing`) and the divider's and EFU's next free cycle move into each `Vu`, so
  that macro and micro mode share VU0's divider; `Link`, the units' reach outside the VPU (FBRST's enables, the D and T
  interrupts, VIF's TOP and ITOP, PATH1); `Path1` and `Path1StandIn`; `Vpu::start`, the one entry for VCALLMS,
  VCALLMSR, CMSAR1 and VIF; VPU-STAT's VDS, VTS, VFS, VGW and EFU bits.
- **`src/ee/cop2.rs`**: the waits of §5.4.5 and §5.4.6 (§23.6); a macro divide waits for a divide a microprogram left
  running; VCALLMS's one-cycle stall after a macro instruction, a named loss at S12, is charged.
- **`src/board.rs`**: the D and T interrupts raised at the INTC's VU0 and VU1 sources.
- **No new cross-component dependency**: `sapphire` still depends on `emusen-native` and `emusen-mips` alone.

### 23.3 The pipelines: what the manual gives, and how each is read

[R *VU User's Manual* §3.1, §3.3, §3.4, §3.5, §4.3's Remarks and Throughput/Latency lines, §5.4.5, §5.4.6, §7.3,
§7.4; A where marked]:

| Rule | Source | Implemented |
|---|---|---|
| A pair issues both halves together; a hazard in either stalls both | §3.4.2 | `plan`, one T stage a pair |
| A register field written by an FMAC-pipeline instruction is read from the writer's S stage, field by field; VF00 never waits | §3.4.1, §3.4.4, Figures 3-3 and 3-4 | `vf_ready`, T + 4 |
| An integer register loaded by ILW or ILWR is read 4 cycles on; integer results bypass to the next pair | §3.4.1, §3.4.7, Figure 3-14 | `vi_ready`; the integer unit's results at once |
| The MAC, status and clipping flags are written at the S stage, read at T | §3.4.4, Figures 3-16 and 3-17 | `FlagWrite`, T + 4 |
| FSSET and FCSET write at the S stage; their pair's upper instruction's effect on that flag is ignored | their pages, §3.3.4 | `drain` |
| Q at the F stage, 7 cycles after DIV or SQRT and 13 after RSQRT; no hazard check; another divide waits for F | §3.4.5, Figures 3-7 and 3-8, §7.4 | `Vu::q_next`, `div_free` |
| WAITQ issues at F and its pair's upper instruction reads the quotient | Figure 3-9 | `plan` |
| P at the EFU's latency; another EFU operation may issue at the last N stage, a cycle before; MFP has no hazard check; WAITP issues at the last N stage and the pair after reads P | §3.4.6, Figures 3-10 to 3-12, §7.4 | `Vu::p_next`, `p_later`, `efu_free` |
| A conditional branch needs a pair between it and the integer write it reads, but not after a flag check | §3.4.8, every branch page | D-80: the value before the write |
| One delay slot after a branch; the target counted from the slot; BAL and JALR link the pair after the slot | §3.5, §4.3 | `jump`; JALR reads its target first (the `branch` console text) |
| The upper instruction's write wins over the lower's to the same register, whole | §3.4.3, the lower pages' Remarks | the lower's write held and dropped |
| Both halves read their operands at T | §3.4.4 ("no stalls … between the Upper instruction and the Lower instruction in the same instruction") [A] | the lower runs first, its register write held back |
| The I bit loads the lower word into I at T for the next instruction | §3.1.4, Figure 3-1 | D-81: the pair's own upper reads the old I |
| The E bit stops the unit after one delay slot; TPC holds the next address | §3.5.3, Figure 3-19 | D-82: TPC in doublewords |
| A D or T bit stops after its pair, no delay slot, when FBRST enables it; on a branch after the slot, TPC the target; an interrupt to the EE | §4.1.3, §7.3.1 | `halting`; INTC VU0 and VU1 |
| QMTC2.I and CTC2.I proceed after the first pair with M set | §4.1.3, §5.4.6's figure | `release_t` + 1 (D-54) |
| A microprogram's first pair issues 3 cycles after VCALLMS, 4 after a macro instruction | §5.4.5's figures | `pipe::START`; D-85 for CMSAR1 |
| A second XGKICK while PATH1 is busy delays the pair after it, not itself | §3.4.9, Figure 3-18 | `kick_wait` |
| A divide or an EFU operation left running at the E bit lands at its latency | §3.5.3 | `Landing` outlives the run |
| The VU runs at the EE's clock | ps2tek "VU Architecture": "294.912 MHz (same as EE)" [R] | one cycle a pair at most |

**Pinned** [M, `tests/vu_micro.rs`]: Figure 3-3 (eight pairs, no stall), Figure 3-4 (T stages 0, 4, 5, 9), Figures 3-7
to 3-9 (the quotient at the seventh pair, the old Q before, the second DIV at 7, WAITQ at 7), Figures 3-11 and 3-12
(ESADD's second at 10, MFP at 11, WAITP at 10), Figure 3-14, Figure 3-15 (a three-pass loop, fourteen pairs in fourteen
cycles), Figures 3-16 and 3-17 (FSAND's flag, the stalled case's 0, 4, 8, 9, 10), Figure 3-18 (the third pair at the
transfer's end), Figure 3-19 (TPC 4); Q's latency read 1 to 14 pairs after DIV, SQRT and RSQRT; the MAC and clipping
flags 1 to 5 pairs after; FSSET and FCSET; the write priority and the read at T; the I bit; ILW's 4; the D and T bits
on VU0 and VU1; the M bit; JALR; the divider and the EFU as resources; VU0 without an EFU; a force break and a reset.

### 23.4 The EFU

§7.2 gives each function as "pseudo microinstructions for reference purpose": MUL, MULA, MADDA, MADD, ADD, SUB, DIV,
SQRT and RSQRT on the constants of §4.3's pages, in a fixed order with fixed operand places [R]. `vu::efu` computes that
code on `sapphire::float`, the multiplier recoding the second operand as D-26 has it, the divisions rounded as D-27,
RSQRT as D-28. One reading parts from the FMAC's: the EFU's accumulator adds a clamped product as ADD adds, without
D-30's rule that keeps an accumulator of exponent field 255, which the console's text decides (D-84). Against the
console's `vu/lower/efu` text, 13 functions over 16 named inputs whose values the lines themselves give (§23.8): **193
of 208 values equal, the other 15 within two in the last place** [M, `the_efu_against_the_console_s_text`]: five of
ESIN's one or two low, six reciprocals and reciprocal roots one low, two roots one high, one EEXP low and one EATAN high.
No document gives the EFU's own last bits, and no single rounding rule fits the fifteen: rounding the divides and roots
up, truncating them, or mixing those with rounding to the nearest equals 60 to 72 of the 96 values they produce, against
87 for the nano code's own rounding [M, a scratch comparison, not kept]; it is a named loss (§23.9).

### 23.5 The gate: the programs (measured 2026-10-25)

`tests/s13/` generates seven programs in S6's form. Each enables COP2, writes a microprogram and its constants through
kseg1 into the unit's micro and data memories, starts it (VCALLMS then VNOP for VU0; CTC2 to CMSAR1 then BC2T until VU1
ends), and copies the result quadwords the microprogram stored with SQ and ISW into the record. The expected record
comes from `tests/s13/manual.rs` (the pipeline's delays, §7.2's nano code as a transcription apart from `src/vu`, on
`s9::manual`'s arithmetic) and `s12::manual`.

| Program | Results (record words) | What it covers |
|---|---|---|
| `s13-integer` | 52 (208) | LQ and SQ with masks and negative offsets, ILW, ISW, IADDIU and ISUBIU over their edges, the integer ALU, MTIR, MFIR, LQI, LQD, SQI, SQD, ILWR, ISWR, MOVE, MR32, RINIT, RGET, RNEXT, RXOR |
| `s13-flags` | 18 (72) | FSAND, FSEQ, FSOR, FSSET, FMAND, FMEQ, FMOR, FCSET, FCGET, FCAND, FCEQ, FCOR, each four cycles after the flag's writer |
| `s13-branch` | 38 (152) | IBLTZ, IBGTZ, IBLEZ, IBGEZ over −1, 0 and 1, IBEQ and IBNE, B, each with its delay slot; BAL's link; JR and JALR, JALR with one register |
| `s13-divider` | 16 (64) | DIV, SQRT, RSQRT with WAITQ: Q, and I and D after it |
| `s13-upper` | 94 (376) | every upper operation once, each FMAC result into a register or ACC, OPMULA with OPMSUB, the conversions, ABS, CLIP, NOP |
| `s13-efu` | 209 (836) | VU1's eleven EFU operations over `efu`'s sixteen inputs, read by WAITP and MFP; MFP's mask |
| `s13-visible` | 60 (240) | Q 1 to 8 pairs after DIV and 1 to 14 after RSQRT; FSAND, FMAND and FCGET 1 to 5 pairs after their writers; a branch on an integer written just before, a pair before and after FSAND; the I bit; the write priority; the E bit's delay slot and TPC; P 1 to 12 pairs after ESADD on VU1 |

**Tests** [M, `cargo test --release`]: `tests/micro.rs` **12 of 12** — the seven programs, each equal to its record
word for word; `the_programs_carry_every_micro_instruction` (164 operations, XGKICK, XTOP and XITOP excepted);
`the_efu_against_the_console_s_text`; `the_ee_waits_for_micro_mode_as_section_5_4_draws_it`;
`xtop_and_xitop_read_the_link`; `a_t_bit_raises_vu1_s_interrupt`. `tests/vu_micro.rs` **24 of 24** (§23.3's pins, and
§23.8's two console texts). `tests/vu0.rs` **14 of 14**: S12's gate unchanged on the new micro mode, its stand-in moved
to the interface's new answers and its halting test kept for a machine built with `Unbuilt`. The crate's other tests
unchanged, on the tree with S21 and S16 beneath: lib **25 of 25** (one new, the EFU's own values), `cop0` 19, `dmac` 17,
`fpu` 9, `gs` 12, `interpreter` 23, `iop` 13, `mmi` 13, `programs` 3; lib, `micro`, `vu_micro` and `vu0` also under `test-checked`; clippy
clean. `blast.sh`'s crates: `blackmoon-gte` 10 of 10, `blackmoon-r3000` 38 of 38, Diamond 306 of 306, its `precision`
8 of 8 and `v1` 5 of 5, `Platform/systems` 39 of 39; WiseMan with `blast.sh`'s filter (`CheatCodec`, `DebugCodeParity`,
`Diamond`, `SapphireTests`, `SystemCodeParity`, `SystemTableParity`) 116 of 116. `emusen-mips` **21 of 21**,
two new (`every_micro_only_lower_operation_reads_back`, `the_manual_s_micro_pages_read_back`).

### 23.6 The EE's side (§5.4.5, §5.4.6)

`Micro::finish` now answers the T stage of a microprogram's last pair (the E bit's delay slot), and the EE waits a
distance behind it drawn from the figures: a macro instruction issues 3 cycles after it (its T stage meets that pair's
S stage), QMFC2.I and CFC2.I 4 (they read as they issue), and another VCALLMS at once (its first pair then follows by
`START`, the second VCALLMS's D stage stalling). `Micro::release` answers the cycle after the first pair with M set.
VCALLMS after a macro instruction waits a cycle. **Pinned** [M, `the_ee_waits_for_micro_mode_as_section_5_4_draws_it`].

### 23.7 On the referees (measured 2026-10-25)

`examples/s13.rs` writes each program as an ELF; `ps2referee.py ram … --until 1FFFF0=5A99D00E 200000:<words>` read each
record from PCSX2 2.8.2 (Americas BIOS, fast boot, its VU recompiler) and Play! 0.72, one process at a time; `s13
explain` lists the result quadwords that differ.

| Program (results) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s13-integer` (52) | 0 | 0 | |
| `s13-flags` (18) | 0 | 4 | Play!: FSEQ's equal case 0, FSSET clears the status flag's low bits, FCSET not written |
| `s13-branch` (38) | 0 | 0 | |
| `s13-divider` (16) | 8 | 8 | both: 0/0 and 1/0 give 7F7FFFFFh (IEEE 754's largest) for ±MAX; the divider's I and D as at S12 (D-46) |
| `s13-upper` (94) | 0 | 1 | Play!: SUBAq into ACC leaves ACC unchanged |
| `s13-efu` (209) | 115 | 119 | both compute the EFU on the host: NaNs and infinities, the last bits; §23.8 |
| `s13-visible` (60) | 0 | 2 | Play!: the lower write kept against the upper's (§3.4.3); P landing a cycle early (10 pairs after ESADD) |

So **PCSX2 is a witness for every pipeline reading the programs test**: the flags' four cycles, Q's 7 and 13, P's 11,
the branch on an integer written just before (D-80), the I bit's own pair (D-81), the write priority, the E bit's delay
slot and TPC in doublewords (D-82); Play! for all but the write priority and P's cycle. Neither is a witness for the
EFU's values or the divider's flags.

### 23.8 The console texts: `vu/*` on PCSX2, and what they decide (measured 2026-10-25)

Each of the seven needs the BIOS's kernel, so Sapphire runs them at S10. Run now through `ps2referee.py tty pcsx2 …
--channel iop` (Americas BIOS) and compared with their expected texts, the consoles' outputs, after line ends are
normalised and PCSX2's own `microVU0: Cached Prog` log lines in the channel dropped [M `diff`]:

| Text (lines) | PCSX2 | What it decides |
|---|---|---|
| `vu/lower/branch` (4) | equal | JALR with one register reads its target before it links (the target 4, the link 4), as implemented |
| `vu/lower/fdivdelay` (350) | equal | D-51's micro form: "DIV single delayed" and "SQRT single delayed" read 1.00 six times and then the quotient, "RSQRT single delayed" twelve times: the quotient reaches the seventh pair and the thirteenth; reproduced as `the_fdivdelay_console_text_s_single_delayed_lines_hold`, the lines' shape read as a divide and one ADDq a pair |
| `vu/lower/integer` (167) | one line | PCSX2 changes the status flag on the first IADD; the values all equal |
| `vu/lower/random` (53) | equal but trailing blanks | D-48 again |
| `vu/upper/clip` (32) | equal | D-49 and the clipping flag's four cycles: "Multi test" reads 0000 four times then 0015, 056A, 0A99 and 0666 three times, four CLIPs each beside an FCGET and then six FCGETs; reproduced as `the_clip_console_text_s_multi_test_holds` |
| `vu/lower/efu` (223) | 118 lines differ | the EFU (§23.4, D-84): on the 208 values PCSX2 equals 93 and Play! 88 (from `s13-efu`'s same inputs), the nano code 193 |
| `vu/games/triace` (5) | one value of twelve | the console prints 42546666h where PCSX2, whose VU arithmetic S12 measured as IEEE 754 towards zero (§21.7), prints 42546665h (D-43) |

**The inputs of `efu`** are named in its lines only (`CVF_ZERO` … `CVF_3PI_OVER2`); their values were read off the
lines themselves: ESUM gives four times a lane, ESADD the squares' sum of three, the scalar functions read z (ERCPR of
`CVF_INCREASING` prints 1/3), `CVF_GARBAGE2` is DEADBEEFh in every lane (ESQRT 4F152108h, ESADD 7EB0E16Bh, ERCPR
negative) and `CVF_GARBAGE1` has exponent field 0 (every function prints a zero's result). `tests/s13/manual.rs`
transcribes the 208 values as `EFU_CONSOLE`.

**D-43** (whether the VU computes as the FPU's console-decided arithmetic): `triace`'s microprogram is generated by its
program at run time, so it cannot be read out of the ELF without reading the test's code, and PINE reads VU0's data
memory but not its micro memory [M: zeros from 11000000h after a microprogram was written there]; the twelve values
cannot be computed on Sapphire before S10 runs the program. What the texts give now: IEEE 754 towards zero, the
referees' arithmetic, misses one of `triace`'s values, and in `efu` the nano code on the module's multiplier and divider
gives the console's last bit where IEEE 754 towards zero does not — EATAN of 1.0 is 1 × π/4 with π/4 recoded, 3F490FDAh
on the console as D-26's dropped column gives, 3F490FDBh on PCSX2 (0 on Play!); ERCPR of 3FFFFFFFh is 3F000001h, D-27's
rounding to the nearest, 3F000000h on both referees. That is the EFU's datapath, not the FMAC's, so it is evidence beside
D-43, not a measurement of it. D-43 stands, argued as at S12; no constant changes.

### 23.9 Named losses

- **The EFU's last bits**: 15 of the console's 208 values off by one or two in the last place (§23.4).
- **The divider's I and D in micro mode** land with the quotient at the F stage (D-83); no document times them.
- **CMSAR1's start**: VU1's first pair issues `START` cycles after the CTC2, as VCALLMS's; §5.4.5 draws only VCALLMS
  (D-85).
- **A store and a load in consecutive pairs**: the load sees the store; no document orders VU memory within the
  pipeline.
- **A COP2 transfer writing a register in the cycle a microprogram writes it**: §3.4.3 gives the transfer priority; the
  EE's write lands in its own instruction, outside the pairs' order.
- **What §3.5.3 forbids** in the E bit's delay slot, a branch in a branch's delay slot (taken as MIPS takes it), the E
  bit on a branch (the slot runs, then the unit stops at the target), and a word no page draws (executed as nothing) are
  not checked or modelled beyond that.
- **A microprogram that never ends**: `finish` and `release` run at most 2²⁴ pairs, about 57 ms of the console's time,
  then let the EE go on with the unit still running.
- **VU0's EFU, XGKICK and XTOP words** do nothing (the units are VU1's, §1.1.2, the pages' Remarks).
- **PATH1** takes a stand-in's fixed time until S15; VGW is set while a pair waits for it.
- **The micro memory's PC** wraps at the memory's size.

### 23.10 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | the flags written at T + 3, a cycle early | `what_each_pair_sees_…`, `figures_3_16_and_3_17_…`, `the_mac_and_clipping_flags_land_at_s`, `the_clip_console_text_s_multi_test_holds` |
| F2 | the E bit's delay slot skipped (the unit stops at the E bit's pair) | `what_each_pair_sees_…`, `the_ee_waits_…`, `figure_3_3_…`, `figure_3_15_…`, `figure_3_19_…`, `the_m_bit_…` |
| F3 | a branch right after a flag check reading the integer as it was (the manual gives the new value) | `what_each_pair_sees_…`, `a_branch_reads_an_integer_written_just_before_as_it_was`, `figures_3_16_and_3_17_…` |
| F4 | Q landing a cycle early | `what_each_pair_sees_…`, `figures_3_7_to_3_9_…`, `q_lands_at_the_divider_s_latency`, `the_divider_and_the_efu_are_resources`, `the_fdivdelay_console_text_…` |

Each was applied to `src/vu/pipe.rs`, `tests/micro.rs` and `tests/vu_micro.rs` run, and the source restored [M].

### 23.11 What the plan got wrong, or left open

- **§3.2's VU row** names the referees as the secondary oracle: for the pipeline's visibility PCSX2 is a witness (§23.7),
  for the EFU and the divider's flags neither is, and the EFU's own console text grades it better than either.
- **§5.3** speaks of the VUs' arithmetic as the FPU's; the EFU parts from it in its accumulator (D-84) and in its last
  bits.
- **S13's gate** names `vu/lower/*`, `vu/upper/clip` and `vu/games/triace`: they need the BIOS's kernel (S10); met now on
  PCSX2 against the console texts, two of them reproduced on the interpreter by programs whose shape their lines give.
- **C4 (§5.13)** prices dropping the flags' and Q's cycle-exact visibility: the interpreter's cost of keeping it is a
  queue of at most four flag writes and two landings a unit, with no allocation per pair.

### 23.12 Questions this stage leaves

1. **Q1 [routine], the EFU's last bits (D-84).** *Recommended*: keep the nano code on the format's arithmetic, 193 of
   208 console values exact and the rest within two in the last place, as the reference and a named loss; a table of
   corrections fitted to fifteen values would claim more than the evidence gives.
2. **Q2 [routine], a branch on an integer written just before (D-80).** *Recommended*: keep the value before the write,
   §3.4.8's need for a slot read so, with both lineages agreeing.
3. **Q3 [routine], `triace` and D-43.** *Recommended*: grade `vu/games/triace` on Sapphire when S10 runs its kernel, and
   decide D-43 then by its twelve values; until then the FPU's readings stay, as S12's Q1 recommended.
4. **Q4 [routine], TPC's unit (D-82).** *Recommended*: doublewords, as CMSAR0 and VCALLMSR take it and both referees
   read it; §7.3.1's restart of VU1 through CMSAR1, which takes bytes, then needs the address times 8.

### 23.13 What was read for this stage

`Sapphire_Plan.md` §2's VU row, §3.2's conformance line, §5.3, §5.13's C3 and C4, §7's S13 row and §11; this page's
§19, §20 and §21; `Sapphire_Disputes.md` D-43 to D-54; `EmuSen_Mips.md` §8.4 and §9; `Sapphire_References.md` §1.1.
Sony's *VU User's Manual* (Version 5.0) chapters 1 and 3 whole, §4.1, §4.3 whole, §5.1.3, §5.4.5 to §5.4.8, §5.5, the
VCALLMS and VCALLMSR pages of §6.2, §7.2 to §7.4; ps2tek's "Vector Unit (VU)" section (architecture, registers, format,
pipelining). ps2autotests' `vu/lower/*.expected`, `vu/upper/clip.expected` and `vu/games/triace.expected` (the consoles'
texts) and the `.elf` files run as black boxes; four bytes of `triace.elf` (its first instruction, as `--until`'s word)
and its section table. No emulator, SDK or loader source, no ps2autotests source and no firmware.

### 23.14 What this stage does not cover

- The `vu/*` console texts on Sapphire: S10 (the kernel), `triace`'s values and D-43 with them.
- VIF's MSCAL family, TOP and ITOP (S14); the GIF behind PATH1 and XGKICK's real transfer time (S15).
- The micro memory's state in `SPH2` with the rest (S10, S30); micro mode in the recompiler (S40–S41).
- The VU's own debug interrupts' handling in the BIOS's kernel (S10, S24).

## 24. Stage S16, the GS's registers, local memory and transfers (2026-10-25)

*Numbered §24 and its disputes from D-100, since S21 (§22, from D-60) and S13 (§23, from D-80) are built beside it.*

### 24.1 In one paragraph

The plan's §7 row S16, on the tester's go of 2026-10-25 for the next stage that needs nothing of the VUs or the IOP,
built beside S13 and S21. The crate gains **`gs`**: the general registers as the host interface writes them, the
privileged registers at 12000000h, CSR and IMR with the SIGNAL, FINISH and LABEL events, HSINT and VSINT, the interrupt
the GS gives the INTC, and its 4 MiB of local memory with **`gs::local`, the one owner of the layout** of all thirteen
storage formats (pages, blocks and columns as the *GS User's Manual*'s chapter 8 draws them, a base pointer inside a
page counted on in blocks, addresses wrapping at the memory's end), on which **`gs::transfer`** runs host to local,
local to host and local to local. The GS acts at once on each write; nothing of its timing is modelled. **`gs::HostInterface`**
is what the GIF (S15) will drive (register writes, image data, local-to-host reads, the stall, BUSDIR's direction); the
tests drive it directly. Five programs of the project's, 51,268 graded record words, each a script rendered twice (as an
EE program sending A+D and IMAGE packets through the GIF's DMA channel and reading local-to-host data through VIF1's into
its record, and as calls on the interface) are equal on Sapphire to the record a model transcribed from the manual
gives. **PCSX2 reproduces every format's layout and round trip at a page's start, the transfers' offsets, restart and
end, the registers' masks, the SIGNAL stall and the reset**, and departs at a depth buffer based inside a page (D-105),
X's wrap at 2048 (D-106), an overlapping copy (D-107) and the interrupt's edges (D-103); Play!'s local-to-host transfer
works for the colour and texture formats, so the S3 decision (the referees' pictures as RAM) holds for it, but it moves
no depth format and performs no local-to-local transfer. PCSX2 reproduces the console texts of `gs/label` and
`gs/signal`, which Sapphire runs with S10. Disputes D-100 to D-107. No emulator, SDK or loader source, no ps2autotests
source and no firmware was read.

### 24.2 What was built

- **`src/gs/psm.rs`**: the thirteen formats by their codes (§7.1's BITBLTBUF table), the bits a pixel takes (§4.3),
  the four page shapes (§8.1, §8.3: 64×32 pixels for the 32-bit formats and the H formats, 64×64 for the 16-bit, 128×64
  for PSMT8, 128×128 for PSMT4), and each format's block order, read from §8.3's figures as the interleaving of the
  block's coordinate bits, the depth orders being the colour orders with block bits 3 and 4 inverted.
- **`src/gs/local.rs`**: `Local`, the memory as 1 Mi words; `address(psm, bp, bw, x, y)`, a pixel's word and the lowest
  bit of its field; `page_table(psm)`, the eight page tables (45,056 entries, built at compile time), which the device
  path (S44) can upload as they are. Within a column the word order is §8.3.1's (two pixels across, then the two below);
  the 16-bit columns use it twice, the low halves then the high (§8.3.2); the 8- and 4-bit columns take the even byte or
  nibble in their first two rows and the odd in the last two, with the word order turned by four pixels in rows 2 and 3
  of even columns and rows 0 and 1 of odd ones (§8.3.3, §8.3.4). PSMT8H, PSMT4HL and PSMT4HH are the 32-bit layout's
  bits 31..24, 27..24 and 31..28 (§2.6.3); PSMCT24 and PSMZ24 keep bits 31..24.
- **`src/gs/transfer.rs`**: TRXDIR's three directions with BITBLTBUF, TRXPOS and TRXREG as they stand when it is
  written (§4.1); transfer data one stream of pixels packed from bit 0 in 64-bit units (§4.3), so a PSMT4 row of eight
  pixels ends inside a unit and the next row continues in it, as the *Restriction*'s "Texture Width" note ("There will be
  no offset of the transfer position") implies; coordinates wrapping at 2048 (§4.1.3); a TRXDIR written before the end
  starting a new transfer (§4.2.1); HWREG outside a host-to-local transfer doing nothing (HWREG's page); the bits after
  the last pixel dropped, and local-to-host data's last unit filled with zeros [A].
- **`src/gs/mod.rs`**: `Gs`, the registers (§7.3's addresses, every defined one stored as written for S17 to S19), CSR,
  IMR, SIGLBLID, the stalled SIGNAL and the writes held behind it, the interrupt line; `HostInterface`.
- **`board.rs`**: the `Gs` on the board, its privileged registers answered for 12000000h to 1200FFFFh, a bus error from
  12010000h to 13FFFFFFh (EE User's Manual §2.2.3), its interrupt latched as the INTC's source 0 at its rise, HSINT and
  VSINT from the blanking stand-in, and `sync_gs` for whoever drives the host interface.
- **No new cross-component dependency**: `sapphire` still depends on `emusen-native` and `emusen-mips` alone.

### 24.3 The readings the manual leaves open

[R *GS User's Manual* chapters 2, 4, 6, 7 and 8; *EE User's Manual* §2.2.3, §3.1, §7.1; the *Restriction* document,
"Restrictions on GS"; ps2tek "GS"; A]:

- **The registers.** The general registers are write-only (§7.1); a write to an address §7.3 does not list is dropped
  [A]. The privileged registers are reached at GA[6:0], the EE's address bit 12 and bits 9..4 (EEUM §2.2.3), so the
  images the map lists (bits 10 and 11) and, by the same sentence, bits 13 to 15 reach the same register (D-102). SIGLBLID
  is read and written whole (§7.2's "(r/w)"; `gs/label`'s console text implies a program writing it, [A]); every
  other address reads CSR (D-100). A store narrower than SD writes its own byte lanes, so a 32-bit store to CSR acts only
  on the bits it carries and `programs/frames.s`'s `sw` of 8 clears VSINT alone [A].
- **CSR** (§7.2): its five events cleared by writing 1; FIFO reads 01 (empty) unless writes are held behind a stalled
  SIGNAL, then 00; REV 1Bh and ID 55h (D-101); FLUSH drops the held writes; RESET clears the events, the stalled SIGNAL,
  the held writes and a transfer, and sets IMR to 7F00h ("All initial values after reset are 1", bits 13 and 14 "should
  be set to 1"), the registers and the memory kept [A]; NFIELD and FIELD read 0 until S20.
- **SIGNAL, FINISH, LABEL** (chapter 6, §7.1): SIGNAL and LABEL update SIGID or LBLID where IDMSK is 1; SIGNAL sets
  its flag, and a second SIGNAL while the flag is set stalls the GS with every write after it held, until the flag is
  cleared, when the second SIGNAL is taken at once (SIGID updated, the flag set again) and the held writes follow; FINISH
  sets its flag at once, the GS having no drawing or transfer left in progress to wait for [A].
- **The interrupt** (§6.1, §6.2.1, the *Restriction*'s "GS Interrupt Handler"): one line, any event whose IMR bit is
  clear; the INTC latches it at its rise (D-103). So an event while the line is high gives no request, and the stalled
  SIGNAL taken at its first's clear gives none until IMR's mask goes to 1 and back, as §6.2.1 says.
- **BUSDIR** (§4.2.2): bit 0 reverses the host interface; while reversed the general registers cannot be written
  ("cannot be accessed") and a write is lost [A], and local-to-host data can be read.
- **Local to local** (§4.2.3): each pixel read in the source's format and written in the destination's, in TRXPOS's
  order (figure 4-1: bit 0 bottom to top, bit 1 right to left), so overlapping rectangles copy in that order (D-107);
  formats of different widths copy the low bits [A], §4.1.1 forbidding them.
- **A format outside BITBLTBUF's table**: the transfer does not start [A] ("Operation is undefined").
- **A PSMT8 or PSMT4 buffer an odd number of units wide**: the pages across rounded down (D-104).

### 24.4 The interfaces: where the later stages attach

- **The GIF (S15)** drives `gs::HostInterface` on the board's `gs`: `write(reg, data)` for PACKED, REGLIST and A+D
  output, `image(data)` for IMAGE, `read()` for local-to-host data, `stalled()` to hold its paths, `reversed()` for
  BUSDIR's direction (which the GIF watches, EEUM §7.1.5, for GIF_STAT.DIR), and then `Board::sync_gs`. The GS keeps
  writes that arrive while stalled in order, so the GIF's FIFO depth and timing stay S15's.
- **VIF1 (S14)** carries local-to-host data to memory (EEUM chapter 5's channel table: channel 1 "can switch directions"; VIF1_STAT.FDR) by
  `read()`.
- **Drawing (S17 to S19)** reads the stored registers (`Gs::register`) and reaches memory only through `gs::local`;
  S19 owns the overlap of local-to-local transfers (D-107).
- **The PCRTC (S20)** reads `Gs::privileged_register`, replaces `hsync` and `vsync` with its own timing, and drives
  NFIELD, FIELD and EDWINT.
- **The device path (S44)** mirrors `gs::local`'s page tables.

### 24.5 The board

`Board` answers the privileged registers through `Bus` (LD and SD, and narrower accesses by lanes); a write brings the
interrupt line to the INTC and reschedules, since clearing HSINT asks for the next H-sync. The stand-in (§20.5) gives
HSINT at each line's start and VSINT at V-blank's start; H-blank's edges are scheduled while HSINT is clear or a timer
wants them, so a program that never clears HSINT costs one edge. Measured [M, `hsint_and_vsint_follow_the_blanking_stand_in`]:
HSINT at the first line's end, VSINT at line 240, VSINT unmasked reaching I_STAT bit 0.

### 24.6 The gate: the programs (measured 2026-10-25)

`tests/s16/` writes each program as a script (`Op`): A+D packets of general registers, IMAGE packets of data from a
32-bit linear congruential generator the EE program runs itself, privileged registers by SD and LD (and SW, LWU at
other addresses), the FINISH handshake of §4.2.2's steps 1 to 3, and steps 5 to 7 (BUSDIR 1, the data read into the
record by VIF1's channel, BUSDIR 0), I_STAT's bit 0. `s16::program` renders it as a bare EE program in S7's form (packets
built through kseg1 at 00300000h and sent by the GIF's channel in normal mode, the record from 00200000h, the done word
at 001FFFF0h); the tests run the same script on the board's GS. The expected record comes from `tests/s16/manual.rs`,
written apart from `src/gs/`: §8.3's block and column figures transcribed as printed and read cell by cell, the transfer
data as a list of bits, the events of chapter 6.

| Program | Graded words | What it covers |
|---|---|---|
| `s16-formats-c` | 15,872 | PSMCT32, PSMCT24, PSMCT16, PSMCT16S: one page-rectangle each from block 1 (§8.5), read back raw as PSMCT32 over the same 32 blocks and in its own format; PSMCT24's page filled first, so its kept bits show |
| `s16-formats-z` | 19,968 | PSMZ32 and PSMZ16 from a page's start, read raw; then PSMZ32, PSMZ24, PSMZ16, PSMZ16S as above |
| `s16-formats-t` | 12,800 | PSMT8 and PSMT8H read raw and in their own format; PSMT4, PSMT4HL and PSMT4HH raw only (§4.1.1 forbids them local to host) |
| `s16-places` | 2,496 | TRXPOS's offsets both ways; the wrap at 2048; pages across a buffer two units wide; PSMT8 one unit wide (D-104); local to local in PSMCT32, PSMCT16 to PSMZ16 and PSMT8H to PSMT8; an overlapping copy (D-107); TRXDIR restarting a transfer, TRXDIR 3 ending one, data beyond the end and outside a transfer |
| `s16-regs` | 132 of 154 | CSR's events and FIFO; HSINT and VSINT, VSINT unmasked to the INTC; the PCRTC's and other write-only registers, IMR, BUSDIR and seven reserved addresses read; SIGLBLID through its images and by 32-bit stores; LABEL's and SIGNAL's masks; the second SIGNAL's stall with a LABEL and a FINISH behind it; IMR and the INTC's GS bit through eleven cases; CSR's reset |

Every format is thus written host to local, read back local to host where §4.1.1 allows, and reinterpreted as PSMCT32,
which §8.3 defines ("Data with the same block number and column number in the same page shows the same memory regardless
of the pixel storage format"); local to local across PSMCT16 and PSMZ16 and across PSMT8H and PSMT8 reinterprets between
formats of one width.

**Tests** [M, `cargo test --release`]: `tests/gs.rs` **12 of 12** — the five programs, each equal to its record entry for
entry; `every_formats_page_falls_as_chapter_8s_figures_show` (every pixel of four pages of each format at four bases
and widths against the transcription); `every_word_of_a_page_is_reached_once_in_every_format`;
`the_conversion_tables_of_8_6_hold_in_every_column` (§8.6's PSMT4, PSMT8 and PSMT16 to PSMCT32 tables, a second
transcription independent of §8.3's figures); `the_console_texts_of_gs_label_and_gs_signal_follow_from_the_masked_updates`;
`the_privileged_registers_answer_their_images_and_above_them_a_bus_error`; `the_gs_interrupt_is_one_line_the_intc_latches_at_its_rise`;
`hsint_and_vsint_follow_the_blanking_stand_in`; `the_host_interface_stalls_on_a_second_signal_and_reads_only_when_reversed`;
`the_programs_assemble_and_their_records_end_in_the_done_word`. The crate's other tests unchanged, on the tree with S21
beneath: lib **24 of 24**, `cop0` **19 of 19**, `dmac` **17 of 17**, `fpu` **9 of 9**, `interpreter` **23 of 23**, `iop`
**13 of 13**, `mmi` **13 of 13**, `programs` **3 of 3**, `vu0` **14 of 14**; `gs`, `dmac` and lib also under `test-checked`; clippy clean. WiseMan with `blast.sh`'s
filter (`SapphireTests`) **5 of 5**.

### 24.7 On the referees (measured 2026-10-25)

`examples/s16.rs` writes each program as an ELF with its expected record; `ps2referee.py ram … --until 1FFFF0=5A99D00E
200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, one process at a time.
Only the records were read.

| Program (graded words) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s16-formats-c` (15,872) | 0 | 2,048 | Play! reads PSMCT16S back as zeros; its raw layout equal |
| `s16-formats-z` (19,968) | 1,024 | all | PCSX2 equal from a page's start; from block 1 four blocks of each depth format land elsewhere (D-105), the round trips equal. Play! moves no depth format |
| `s16-formats-t` (12,800) | 0 | 0 | both equal in every texture format |
| `s16-places` (2,496) | 62 | 392 | PCSX2 does not wrap X at 2048, writing the wrapped pixels elsewhere and reading them as 0 (D-106), and copies an overlap as a shift (D-107). Play! performs no local-to-local transfer (368 words) and does not end a transfer at TRXDIR 3 (24) |
| `s16-regs` (132) | 13 | 52 | PCSX2 reads CSR at the images (D-102) and raises the INTC at each unmasked event and each mask cleared over a set flag (D-103); every write-only and reserved register reads CSR (D-100); REV 1Bh, ID 55h (D-101). Play! reads CCCCCCCCCCCCCCCCh from the write-only registers, ID 0 and REV 7, keeps no images, ignores IDMSK in LABEL and SIGNAL, does not stall the second SIGNAL, and its reset leaves FINISH and IMR |

So **PCSX2 is a witness for the layout of every format at a page's start, the transfers' offsets, restart and end,
local to local without overlap, CSR, SIGNAL's stall and the reset**; Play! for the colour and texture formats' layouts,
their local-to-host transfer and X's wrap. Both answer D-104 alike. **The S3 decision holds**: Play!'s local-to-host
transfer reads PSMCT32, PSMCT24 and PSMCT16 back exactly, so the project's GS programs can read their pictures back from
Play! as RAM (`Sapphire_Referees.md` §6's form C); a picture kept in PSMCT16S or a depth format cannot be read from
Play! this way.

### 24.8 ps2autotests' `gs/label` and `gs/signal` on PCSX2 (measured 2026-10-25)

Both call the BIOS's kernel, so Sapphire runs them at S10. Run now through `ps2referee.py tty pcsx2 … --channel iop`
(Americas BIOS) and compared with their expected texts, the consoles' outputs, after line ends are normalised [M
`diff`]: **both equal** (4 and 4 lines). Their values follow from SIGLBLID written whole and a LABEL or SIGNAL masked to
bits 4 to 7, which `the_console_texts_of_gs_label_and_gs_signal_follow_from_the_masked_updates` reproduces on the board
[A: the sequence is inferred from the values, the programs' sources not read].

### 24.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | PSMCT16's columns 0 and 1 swapped within each block, in `local::entry` | `every_formats_page_falls_…`, `the_conversion_tables_of_8_6_…`, `every_format_round_trips_…`, `transfers_take_their_places_…` |
| F2 | TRXPOS's source offset ignored local to host | `transfers_take_their_places_…` |
| F3 | FINISH not raising its CSR bit | `the_registers_csr_imr_…`, `the_gs_interrupt_is_one_line_…`, `the_host_interface_stalls_…`, `every_format_round_trips_…`, `transfers_take_their_places_…` |

Each was applied, `tests/gs.rs` run, and the source restored [M]. A swap of two columns is a permutation, so
`every_word_of_a_page_is_reached_once_…` alone does not see F1, as expected.

### 24.10 What is not modelled

Each a named loss, and a dispute candidate if an oracle measures it [A]:

- **The GS's timing**: every write acts at once; FINISH is set at the write; the host interface's FIFO is never
  "almost full"; a transfer's data is written or read as it arrives. S15 decides the GIF's side, S17 whether drawing
  time is charged, S43 the threads.
- **TEXFLUSH** and the order of writes to memory it guards (§4.2.1): with an instant GS there is no texture page buffer
  to be stale (S18).
- **NFIELD, FIELD, EDWINT, EXTWRITE's feedback write** and the PCRTC's own H-sync and V-sync: S20.
- **The *Restriction*'s "Texture Width"** failure (a few pixels not written for odd widths, "in some rare cases"): every
  pixel is written.
- **The machine's GSMEM space** (`machine.rs`) is not yet the GS's memory: the machine holds no `System` until S10.

### 24.11 What the plan got wrong, or left open

- **§2.3** names "the page and block layouts of the 4- and 8-bit formats" as thin: the *GS User's Manual*'s chapter 8,
  fetched at S6, draws every format's blocks and columns and §8.6 gives a second table for the 4-, 8- and 16-bit columns;
  both lineages agree with them at a page's start. What the documents leave open is the depth formats' order from a base
  inside a page (D-105) and an odd width for PSMT8 and PSMT4 (D-104).
- **§3.2's GS row** names `gs/label` and `gs/signal` as the primary oracle: they test SIGLBLID's masked updates only
  (§24.8); the project's programs carry the rest.
- **S16's gate** asks for "referees' pictures of a transfer program": by the S3 decision these are the records the
  programs read back local to host, which both referees give for the colour formats (§24.7).
- **The privileged registers SMODE1, SRFSH, SYNCH1, SYNCH2 and SYNCV** are in the *EE User's Manual*'s map (§2.2.3) and
  ps2tek's, with no fields in either document or in the *GS User's Manual*; they are stored as written for S20.

### 24.12 Questions this stage leaves

1. **Q1 [routine], a depth format from a base inside a page (D-105).** *Recommended*: keep §8.5's order, which PCSX2 alone
   departs from and Play! cannot test; look again if a game at S28 draws or textures from such a base.
2. **Q2 [routine], the GS's interrupt as one line (D-103).** *Recommended*: keep the two documents' reading against
   PCSX2's per-event requests; the kernel tests at S10 and the bench at S28 exercise the handler.
3. **Q3 [routine], CSR's REV and ID (D-101).** *Recommended*: PCSX2's 1Bh and 55h until a console text prints CSR.
4. **Q4 [routine], an overlapping local-to-local copy (D-107).** *Recommended*: pixel by pixel in TRXPOS's order now,
   decided at S19 with programs that copy in each of the four orders.

### 24.13 What was read for this stage

`Sapphire_Plan.md` §2, §3.2, §5.4, §5.12, §5.13 (C5, C6), §7 (S15 to S20), §10 and §11; `Sapphire_Referees.md` §2, §6
and §7; this page's §20; `Sapphire_Disputes.md` D-0 to D-54; `Sapphire_References.md` §3. Sony's *GS User's Manual*
(Version 5.0) chapters 2, 4 and 6 whole, §1's contents, §3.1, chapter 7's register pages and §7.3, chapter 8 whole; the
*EE User's Manual* §2.2.3, chapter 5's channel table, §7.1 and the VIF1_STAT and GIF_STAT pages; the kit's *Restriction*
document whole. ps2tek at `1c91660`: "Memory Map", "I/O Maps" (the privileged registers), "Graphics Interface (GIF)" and
every "GS" section. ps2autotests' `gs/label.expected` and `gs/signal.expected` (the consoles' texts, not the programs'
source). No emulator, SDK or loader source, no ps2autotests source and no firmware.

### 24.14 What this stage does not cover

- The GIF and its paths (S15), VIF1's FIFO (S14): until then nothing on the EE's side writes the general registers or
  reads local-to-host data on Sapphire; the tests drive the interface.
- Drawing (S17 to S19) and the PCRTC (S20).
- `gs/label` and `gs/signal` on Sapphire: S10.
- The state format's new contents (the GS's registers and its 4 MiB): `SPH2` gains them with the rest (S10, S30).
## 25. Stage S23, the CDVD drive and emusen-disc's DVDs (2026-10-25)

### 25.1 In one paragraph

The plan's §7 row S23, on the tester's go of 2026-10-25, built beside S13, S16 and S21 and recorded here as §25 with its
disputes from D-120. `emusen-disc` gains DVDs as §9.4 designed them (`EmuSen_Disc.md` §14): a medium chosen when an
image is opened, one or two layers with the break taken from where the image's second volume begins, every sector read
as its ECMA-267 Data Frame, whole or synthesised with its ID, IED, CPR_MAI and EDC, CHD's DVD form, and images opened
over the host's bytes, never copied. The crate gains no drive's commands or timing. Sapphire gains **`cdvd`**, the
drive controller at its register interface (§5.6's open replacement of the Mechacon's firmware): the N commands (NOP,
Standby, Stop, Pause, Seek, the CD reads in three block sizes, CD-DA, the DVD read, GetToc) timed by ps2tek's formula and
seek classes, the S commands (the version, the sticky status, the tray, the clock, ForbidDVD, the EEPROM's sessions),
the drive's interrupt and a one-sector buffer for the IOP's DMA channel 3, which S21's bus and DMA will drive; disc
authentication is not modelled and every disc is taken as the console's own, as §5.6 says; MagicGate's and SECRMAN's
commands, and every other command no document describes, answer as absent. Where ps2tek is silent the answers are
PCSX2's, read as a black box from EE programs written for the purpose, the only lineage that lets the EE reach the
drive; nine disputes record them. Synthetic CDs and single- and dual-layer DVDs read sector for sector through the drive
with the layer break placed. P13's mapped open is 1.15 ms cold on the desktop; the handheld did not answer. No emulator,
SDK, loader or ps2autotests source and no firmware was read.

### 25.2 What was built

- **`emusen-disc`** (`EmuSen_Disc.md` §14): `ecma267.rs` (the Data Frame's codec and §25's numbers), `dvd.rs` (`Dvd<B>`,
  the layer break, the frames, `CD_LIMIT`), `medium.rs` (`Kind`, `Medium<B>`), `open::open_medium`, and `ChdImage`
  opening either medium by its unit. Its one owner of GF(2⁸) is `ecma130`'s, which the IED uses.
- **`cdvd/`** in Sapphire: `mod.rs` (the registers, the N and S commands, the events, the buffer, `open`), `timing.rs`
  (the formula, the classes, the radius), `rtc.rs` (the clock), `nvm.rs` (the EEPROM's sessions), `tests.rs`. Its time
  is the IOP's cycle (36.864 MHz, D-0), since the drive is the IOP's device. Its surface for S21 is three calls and a
  line: `read8(offset)` and `write8(offset, value, now)` for the registers at 1F402000h, `dma_ready()` and
  `dma_take(now, bytes)` for channel 3, `irq()` for the IOP's IRQ2; and `next_event()` and `run(now)` for the scheduler.
- **`cdvd::open`**, Sapphire's choices for a disc: CD-XA framing, and an image of 2,048-byte sectors taken for a DVD
  unless its sheet or bytes say a CD [C: the library's DVD titles are kept as `.iso` and its CD titles as `.cue` and
  `.bin`, Q8 and Q16]; more sectors than a CD's time code names are a DVD regardless. A PlayStation 2 CD kept as a bare
  `.iso` is therefore read as a DVD, a named loss until a disc is found that needs otherwise.
- **The dual-layer builder** (`SyntheticPs2Disc.DualLayerDvd`, S4) corrected to D-127's rule: layer 0's descriptor names
  layer 0's sectors and layer 1's descriptor its own, where both had named the whole image; `SapphireRunnerTests` pins
  it.
- **The referees' tools**: `ps2referee.py ram pcsx2 … --disc DISC` puts a synthetic disc in PCSX2's drive while the
  program runs in its place (`-elf`); `ps2cdvd.py` writes the drive programs of §25.3; `ps2discs.py` writes their sparse
  synthetic discs; `programs/s23/reach.s` and `ssweep.s` are the two written by hand.
- **New dependency, named**: `sapphire` depends on `emusen-disc` (as Diamond and Nephrite do); `emusen-disc` still
  depends on `chd` alone.

### 25.3 The drive on the referees (measured 2026-10-25)

ps2tek describes eight S commands, nine N commands and the registers in outline, and nothing of most commands'
parameters, results or errors. A dispute step therefore drove the drive's registers from EE programs, one referee
process at a time, with only RAM read back (`ps2referee.py ram`), on synthetic discs from `ps2discs.py`: a
single-layer DVD of 1,000,000 sectors, a dual-layer one of 1,100,000 broken at 600,000, and a CD of 30,000 raw sectors.

- **Who reaches the drive.** `reach.s` reads the registers at BF402000h and sends S command 03h: PCSX2 2.8.2, LRPS2 and
  PCEE2 answer alike (N status 4Ch, drive status 0Ah, MechaconVersion 03h 06h 02h 00h); **Play! 0.72 answers CCh at every
  register**, so the PlayStation 2's drive is visible from the EE on one lineage only, and every measurement below is
  that lineage's. A `.cue` given to PCSX2 with `-elf` never ran the program; the `.bin` alone did (disk type 12h).
- **The S commands** (`ssweep.s`, every command but 0Fh, which ends PCSX2's run, sent with no parameters, twice with no
  disc and once each on the DVDs; `ps2cdvd.py scmd config|tray|rtc`): D-124 and D-125. 202 of 255 answer the one byte
  80h; the answers did not change between a single- and a dual-layer disc.
- **The N commands' times** (`ps2cdvd.py ncmd dvd|dual|cd|thrdvd|thrcd`, each command timed by COP0 Count from its
  write until the busy bit clears, read at 294.912 MHz):

| Command (disc) | PCSX2 | ps2tek |
|---|---|---|
| Seek, delta 15 (DVD) or 7 (CD), and 0 | 0.168 ms | block_timing × delta |
| Seek, delta 16 (DVD) or 8 (CD) to 14,763 or 4,370 | 30.000 ms | ~30 ms below 14,764 or 4,371 |
| Seek, delta 14,764 or 4,372 and more | 100.000 ms | ~100 ms |
| Seek after Stop | 333.33 ms more | 333 ms to spin up |
| Stop | 166.667 ms | "166 ms delay?" |
| Pause | 10 EE cycles | — |
| Standby; a short seek after it until the first fast seek | 9.93 ms; 8.84 ms (DVD). 14.28 ms; 10.78 ms (CD) | — |
| Seek across the dual-layer break, 599,990 to 600,010 | 30.000 ms, as any 20 sectors | — |

- **The reads.** A read needs the IOP's DMA channel 3 to take each block, which an EE program cannot drive: ReadDvd and
  ReadCd stay busy (N status 8Dh) for the whole bounded wait. ReadDvd with the eight parameter bytes ps2tek lists ends
  at once with error 22h; with eleven it runs. No block's bytes were read from any referee (D-121, D-122).
- **The N commands' numbers** (`ps2cdvd.py nsweep`, 00h–1Fh with eleven zero parameters): D-126.

No referee's log was read: the driver reads it inside the run for the BIOS line and the swap chain only (§13).

### 25.4 The model

- **N commands** (ps2tek "CDVD N Commands"): a command written while the drive is busy is ignored [A]; its parameters
  are the bytes written to 05h before it, zeros past them, and their count is not checked (D-126, a named loss). Busy
  shows as N status 80h, idle as 40h, bit 0 for an error. Standby seeks to sector 0; Stop takes 166.67 ms and leaves the
  drive stopped at sector 0; Pause and NOP end at once; Seek moves the head; every N command ends with I_STAT bit 1.
  A command that needs the disc with none ready ends at once with error 01h [C].
- **Reads**: the seek first, with no interrupt of its own; then one block a block_timing, at ps2tek's speeds (24 ×
  153,600 bytes a second on a CD, 4 × 1,382,400 on a DVD), the block's size in the formula; each block waits in a
  one-sector buffer for DMA, and the next sector, once off the disc, waits for the buffer to be taken [A: ps2tek's "Once
  one sector has been read, the CDVD DMA channel can store the data in memory and allow the CDVD drive to continue"];
  the read ends, with I_STAT bits 0 and 1 (ps2tek: "both bits 1 AND 0"), when its last block has been taken [A: so that
  the data is in memory when the interrupt comes]. A count of zero ends after the seek. ReadCd's blocks are its 2,048
  bytes of user data, the sector from its subheader (2,328) or from its header (2,340) [A from ECMA-130's layout];
  CD-DA's the raw 2,352 bytes; ReadDvd's the Data Frame (D-121, D-122). ReadCd of a DVD gives its Main Data, the
  rest of a larger block zero [A]. LBA 0 is 00:02:00 on a CD.
- **GetToc** (09h, "a block size of 2064 bytes"): a DVD's physical format information (ECMA-267 §26.5.1: version 1,
  120 mm at 10.08 Mbit/s, the structure byte with OTP and Type C for a dual-layer disc, the Data Zone's first and last
  sectors and layer 0's last) and, for a CD, the lead-in's items A0h, A1h, A2h and each track's, ten bytes of Q each [C,
  since ps2tek gives the block's size alone].
- **S commands** answer at once [M: PCSX2's were done before the first poll], their results read from 18h until the
  status's bit 6 says none is left (D-124, D-125).
- **The tray and the disc's detection** (D-128): opening ends any N command, stops the disc and sets I_STAT bit 3
  ("Disk ejected"); closing over a disc leaves the drive status 00h and the disk type 01h for half the 333 ms spin-up,
  the kind's detecting value (02h a CD, 03h a DVD, 04h a dual-layer DVD) for the rest, then its type; the host changes
  the disc only with the tray open (`insert`).
- **The disk type**: 12h a CD of data, 13h a CD with audio tracks beside its data, FDh one of audio alone, 14h a DVD
  [R ps2tek's table]. The PlayStation's types (10h, 11h) are not given: Sapphire refuses those discs (§4.1).
- **Authentication**: no exchange is modelled; every disc reads as authenticated for the console's region, as §5.6
  requires, which the machine info will say in words when the drive reaches the machine (S24).
- **Losses, named**: the parameter count unchecked; ps2tek's other registers (byte 13h and the like) read 0; the spindle
  and retry bytes of a read ignored; the speeds fixed; no read errors, the Lead-out read past a disc's end; the BREAK's
  interrupt [A]; the detecting values' timing [C].

### 25.5 The clock and the EEPROM

- **The clock** (ReadRTC, WriteRTC): seconds, minutes, hours, day, month and a two-digit year in BCD, the first and
  fifth bytes zero [R ps2tek], running on the drive's time from what the host sets; 2000-based [C], every fourth year a
  leap year. PCSX2's answers have the same form and read back what was written [M].
- **The EEPROM** (`nvm.rs`): 1,024 bytes [C, the size of the file PCSX2 keeps beside each BIOS, S3's listing] in
  16-byte blocks, three areas of sixteen blocks [C]; OpenConfig's parameters mode, area, count (D-125, measured); a
  session reads or writes the next block of its count, zeros past it, a write in a reading session ignored. Blank is
  all zeros. The medium `ps2.nvm` and the layout the BIOS expects are S24's (§4.4), which may move the areas.

### 25.6 The gate: the synthetic discs, sector for sector (measured 2026-10-25)

`cdvd/tests.rs` drives the registers as CDVDMAN would, with its own builders (a DVD of 300 sectors, a dual-layer DVD of
1,100 broken at 1,024 by D-127's rule, CDs of 60 data sectors and of 60 data and 20 audio): **10 of 10**, and
`timing`'s two, `rtc`'s and `nvm`'s one each, 14 in the module.

| Test | What it holds |
|---|---|
| `at_power_on_the_registers_report_the_disc` | each disc's type, the drive and N statuses, no interrupt |
| `the_s_commands_answer_as_the_documents_and_the_record_say` | the version, an unknown subcommand, eight absent commands among them 84h, 85h and 8Eh, ForbidDVD, the clock ninety seconds on, an EEPROM area written and read |
| `the_tray_opens_closes_and_the_disc_is_detected_as_it_spins_up` | the tray's statuses and interrupt, a refused second opening, the sticky status, a seek with no disc, detection's stages for a dual-layer DVD and for a DVD put in with the tray open |
| `seeks_take_ps2teks_times_and_raise_the_interrupt_at_their_end` | each class's time, not a cycle early, the interrupt and its acknowledgement, Stop, the spin-up after it, Standby, Pause, a CD's bounds |
| `a_dual_layer_seek_is_timed_across_the_radius` | D-123 through the registers |
| `a_dvd_reads_sector_for_sector_with_the_layer_break_placed` | every sector of both DVDs read: its layer bit, b29, its number from the volume's start, IED and EDC, its data, one block after the last |
| `a_read_waits_for_its_buffer_to_be_taken` | the buffer's back-pressure and the interrupt after the last block is taken |
| `a_cd_reads_its_block_sizes_and_its_audio` | the three block sizes, CD-DA at its block time, the two wrong-medium errors, ReadCd of a DVD |
| `break_ends_a_read_and_get_toc_reports_the_disc` | BREAK; GetToc's physical format information on both DVDs and a CD's lead-in items |
| `an_image_opens_as_the_playstation_2s_disc_kinds` | `cdvd::open`: an `.iso`, a raw track, a sheet, a dual-layer image's break, junk refused |

S4's synthetic discs (`SyntheticPs2Disc`) are read through `emusen-disc` in the same terms by the crate's own DVD tests,
whose builders follow the corrected rule, and through the host's reader by `SapphireRunnerTests`.

### 25.7 P13, load time

`EmuSen_Disc.md` §14.5: the test copy of Jak and Daxter, opened mapped and read-only with none of it in the page cache,
mapped in 0.010 ms and opened in **1.151 ms** (the layer break's two sectors read from the drive), against 519 ms to
read it whole; the synthetic dual-layer image of 2.25 GB opens in 0.016–0.019 ms with its break found. **The open holds
P13 on the desktop; P13's "a copy would take tens of seconds" does not** (half a second on NVMe). **The handheld half is
left**: `hh.sh guard` found no route to the handheld on 2026-10-25.

### 25.8 The blast radius (measured 2026-10-25)

`blast.sh WiseMan sapphire-s23`'s crates, `cargo test --release`, `EMUSEN_DISC_CHDMAN` named, and its WiseMan filter, one
run at a time at CPU weight 20:

| Where | Result |
|---|---|
| `emusen-disc` | **42 of 42**: 18 unit, 3 damage, 13 disc, 8 DVD |
| Sapphire, after the branch was rebased onto S21's, S16's and S13's merges | lib **39 of 39** (14 the drive's), `cop0` 19, `dmac` 17, `fpu` 9, `gs` 12, `interpreter` 23, `iop` 13, `micro` 12, `mmi` 13, `programs` 3 (now reading `programs/s23/`), `vu0` 14, `vu_micro` 24; `cdvd` also under `test-checked`; clippy clean |
| Diamond, Diamond's `v1` | **306 of 306**, **5 of 5**: the crate's first user, unchanged |
| Nephrite | **194 of 194**: its second, unchanged |
| `emusen-core-conform`, `Platform/galaxia`, `library`, `library-tool`, `platform` | 36, 87, 11, 9 and 40, every one passing |
| WiseMan (`Diamond`, `Galaxia`, `GameRecords`, `Library`, `NephriteTests`, `SapphireRunnerTests`, `SapphireTests`) | **562 passed, 0 failed, 11 skipped**: eight `SettingPathsOpenTests` and three big-picture tools, skipped by their own conditions, none near the disc; on the rebased tree `SapphireRunnerTests` and `SapphireTests` again, **14 of 14** |

### 25.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | the layer break placed one sector late (`find_layer_break` answering V + 1) | `emusen-disc`'s `a_dual_layer_images_second_volume_places_the_layer_break` and `whole_data_frames_are_read_as_stored_and_told_by_their_first_id`; the drive's `a_dvd_reads_sector_for_sector_with_the_layer_break_placed`, `break_ends_a_read_and_get_toc_reports_the_disc` and `an_image_opens_as_the_playstation_2s_disc_kinds` |
| F2 | a DVD sector header's layer bit left clear (`sector_information` dropping b24) | `ecma267`'s `the_sector_information_and_numbers_are_section_16_and_25s`, `a_dual_layer_images_second_volume_places_the_layer_break`, the drive's `a_dvd_reads_sector_for_sector_with_the_layer_break_placed` |
| F3 | a seek timed from the wrong layer (`radius` giving layer 1's sectors their logical place) | `a_dual_layer_seek_is_measured_across_the_radius`, `a_dual_layer_seek_is_timed_across_the_radius` |

Each was applied, `emusen-disc`'s unit and DVD tests and the drive's module run, and the file restored from git [M].

### 25.10 What the plan got wrong, or left open

- **§5.6's "DVD reads report what ECMA-267's sector header gives"**: ps2tek describes the drive's block otherwise in two
  fields; D-121 and D-122 take ECMA-267's fields and ps2tek's numbering, open until a block is read on the IOP.
- **S23's "region and model queries ps2tek describes"**: ps2tek describes none at the S-command level, only
  CDVDMAN's exports for them; they answer as absent (D-125), as do MagicGate's and SECRMAN's.
- **§3.2's CDVD row** names the referees as the oracle for timing: one lineage reaches the drive from the EE, and none
  the reads' data; S21's IOP and S24's BIOS are the next witnesses.
- **§9.4's "a dispute candidate"** for the layer break is D-127, open: no referee shows its rule through the EE, and the
  one real disc is single-layer.
- **S4's Q2** (the dual-layer layout on the referees): not answerable from the EE, since PCSX2 shows nothing of layers
  there; the builder now follows D-127.
- **P13** is half held, half wrong on the desktop (§25.7), and waits on the handheld.

### 25.11 Questions this stage leaves

1. **Q1 [routine], the version bytes (D-124).** *Recommended*: keep 03h 06h 02h 00h, the only witness's answer, and let
   S24 show whether the BIOS's CDVDMAN accepts them; change them only on that evidence.
2. **Q2 [routine], the contiguous seek (D-120).** *Recommended*: keep ps2tek's block_timing × delta under PCSX2's measured
   bounds; S32's bench, where streaming games load, would show a game that depends on PCSX2's constant.
3. **Q3 [routine], dual-layer seeks across the radius (D-123).** *Recommended*: keep the OTP geometry; it costs nothing
   on single-layer discs, and the first dual-layer disc of the bench (God of War, Q8) is where it matters.
4. **Q4 [decision], a PlayStation 2 CD kept as `.iso`** (§25.2). *Recommended*: keep an `.iso` a DVD, as the library
   keeps its DVDs, and refuse nothing; if a player's CD title is found as `.iso`, a per-game setting can say CD.
5. **Q5 [routine], P13's handheld half.** *Recommended*: run `open_mapped` on the handheld's card the next time it
   answers, with the test copy kept there (Q8), and record it here.
6. **Q6 [routine], the EEPROM's areas (§25.5).** *Recommended*: leave them as chosen until S24, whose BIOS writes its
   settings through them and so shows the layout it needs.

### 25.12 What was read for this stage

`Sapphire_Plan.md` §1.3, §2, §4.4, §5.6, §5.9, §5.11, §7, §9.4, §11 and §13; `EmuSen_Disc.md` whole; `Diamond_Plan.md`
§5.6; this page's §14 and §20; `Sapphire_Disputes.md`; `Sapphire_Referees.md` §2 and §4; `DOCS.txt` §X (the
exclusions). ECMA-267 (3rd edition, 2001) §4, §7, §12.6, §15–§17, §23–§28; ECMA-119 §8.4; ECMA-130 §22.3.3.4. ps2tek at
`1c91660`: the IOP's memory map, "CDVD Drive" whole (its I/O ports, N commands, reads and seeks, S commands; the three
lines of `DOCS.txt` X.3 passed over as evidence), CDVDMAN's and SECRMAN's export lists. chdman 0.289's `info -v`. No
emulator, SDK, loader or ps2autotests source and no firmware; no referee's log.

### 25.13 What this stage does not cover

- The drive on the IOP's bus and DMA channel 3, and its IRQ2 into the IOP's INTC: S21's bus and DMA plug into
  `read8`, `write8`, `dma_take` and `irq` once both land.
- The drive in the machine and in the state format; the EEPROM as the `ps2.nvm` medium; the clock set from the host's
  time: S24 and S30.
- The PlayStation mode's CD-ROM commands (Diamond's), DVD-Video (§1.2), and the block-compressed ISO forms.
- P13 on the handheld.

## 26. Stage S22, the SIF between the EE and the IOP (2026-10-25)

*§23 to §25 are stages S13, S16 and S23's, built beside this one on their own branches; the SIF needs nothing of the
VUs, the GS or the CDVD.*

### 26.1 In one paragraph

The plan's §7 row S22, on the tester's go of 2026-10-25, once both sides of the SIF were merged. The crate gains
**`sif`**: the SIF's mailbox registers at the EE's 1000F200h and the IOP's 1D000000h (MSCOM, SMCOM, MSFLG, SMFLG, CTRL,
BD6, ps2tek "SIF Registers"), and a FIFO of 32 words, 128 bytes, for each of SIF0, SIF1 and SIF2 (the *EE User's
Manual*'s figure), joining the EE DMAC's channels 5 to 7 to the IOP DMAC's 9, 10 and 2 so that a transfer is one
transfer: the EE moves quadwords into and out of a FIFO, the IOP words, and a channel that would overfill or underrun
its FIFO holds until the other side's channel moves words, which wakes it. The stand-ins S11 and S21 left at those
channels are replaced. SIF0's EE destination tag is the two words the IOP sends after its own tag (D-142); the IOP takes
SIF1's tag as a quadword (D-143); the IOP's chain now moves only as many words as its FIFO allows, the packet going on in
later units. The EE's CTRL write with bit 18 raises the IOP's interrupt 1, the one witness's reading (D-141). Five
paired programs of the project's, an EE program carrying an IOP program and stepping with it through a mailbox in IOP
RAM, 194 graded record words, are equal on Sapphire to the records ps2tek's SIF and IOP DMA chapters and the EE manuals
give, each side checking what the other sent. PCSX2, the only lineage with an IOP window, runs them and differs on 57:
it holds no FIFO, ends an IOP chain at an IRQ tag, sends SIF0's tag words without bit 8, takes the EE's write to SMCOM
and moves nothing on SIF2; the disagreements are D-140 to D-145, and D-68 is amended. The SIF RPC protocol the BIOS's
modules speak over these is S24's (§26.13). No emulator, SDK, loader or ps2autotests source and no firmware was read.

### 26.2 What was built

- **`sif/mod.rs`**: `Sif`, the registers and the three FIFOs; `EePorts`, which wraps the EE's stand-ins and answers
  `dmac::Peripherals` for channels 5, 6 and 7 from the FIFOs, SIF2's direction its EE channel's CHCR.DIR; `IopPorts`,
  the same for `iop::dma::Peripherals` and channels 9, 10 and 2. A FIFO changed by one side marks the other to be woken.
  Two unit tests: the flags and COMs, and a quadword's four words and a full FIFO.
- **`dmac.rs`** gains two provided methods on `Peripherals`, `tag_ready` and `get_tag`, by which a peripheral whose
  destination tag is narrower than a quadword says so; the stand-ins keep the defaults. A destination chain's tag is
  read through them.
- **`iop/dma.rs`**: the chain mode keeps its packet (`in_packet`, the tag's first word, the address and words left) across
  units, reads its tag only when the peripheral can give or take the tag's words, moves as many data words as the
  peripheral allows, and acts on IRQ and end when the packet's last word has moved; a chain into memory takes a quadword
  tag (D-143); `ready` asks for the tag's words before a packet; `wake_from` wakes the DMAC not before a given cycle.
- **`iop/bus.rs`**: the field `sif`; the SIF's registers read and written at `Target::Sif`, at the I/O registers' cost;
  the DMAC stepped with `IopPorts`; a FIFO the EE has changed wakes the DMAC at the next `run_units`;
  `sif_write_from_ee`, the EE's write, raising the IOP's interrupt 1 when CTRL's bit 18 is set.
- **`board.rs`**: the EE's SIF registers answered from `iop.bus.sif`; the DMAC stepped with `EePorts`; before the
  DMAC's event at a time the IOP is first brought to that time, so that the EE's DMAC sees the FIFO as the IOP has left
  it then and the IOP never sees a word the EE moves later; after the IOP is brought to the board's time, a FIFO it has
  changed wakes the DMAC at once.
- **`lib.rs`** declares the module. No new dependency: `sapphire` depends on `emusen-native`, `emusen-mips` and
  `blackmoon-r3000`, and `sif` on the crate's own `dmac` and `iop::dma` interfaces only.

### 26.3 The registers

[R ps2tek "SIF Registers"; D-141]: MSCOM is written by the EE alone and SMCOM by the IOP alone, each read by both; a
write to MSFLG from the EE sets the bits written as 1 and one from the IOP clears them, SMFLG the reverse, a write of 0
changing nothing; CTRL reads F0000102h on the EE and F0000002h on the IOP (ps2tek's bits 1, 8 and 28 to 31), writes
changing neither, but the EE's write with bit 18 set raises the IOP's interrupt 1 (ps2tek's "GPU (used in PSX mode)")
[the one witness, D-141]; BD6 is a word both sides write and read [C]; the other offsets of 1000F200h-1000F2FFh and
1D000000h-1D0000FFh read zero [C]. A byte or halfword store on the IOP side is shifted to its lane, as the I/O
registers' are. The EE's INT_SBUS and the timers' HOLD have no driver: no document says what raises INT_SBUS, and no
write the probes tried raised it on the referee.

### 26.4 The FIFOs and the transfers

[R EEUM §1.2, §5.1, §5.9; *EE Overview* §2.6, §3.1.2; ps2tek "SIF DMA", "IOP DMA"; A where marked]:

- **Size** (D-140): 32 words, one FIFO a transfer. The EE asks for room or words in whole quadwords and moves a
  quadword as four words, its bits 31..0 first; the IOP asks for and moves words.
- **SIF0**, the IOP to the EE: the IOP's channel 9 in chain mode from memory, its tag's address and size in words, with
  CHCR's bit 8 the two words after the tag sent first; the EE's channel 5 in normal mode (data quadwords) or destination
  chain mode, whose tag is those two words, bits 63..0 of a DMAtag, bits 127..64 never crossing (D-142). The EE's cnt,
  cnts and end act as for fromSPR (§20.4).
- **SIF1**, the EE to the IOP: the EE's channel 6 in normal mode or source chain mode (TTE sending the DMAtag's quadword
  too); the IOP's channel 10 in chain mode into memory, taking a quadword as its tag, the first two words the IOP's tag
  and the last two dropped, whatever bit 8 (D-143). So an EE packet carries its IOP tag in its first quadword.
- **SIF2**, either way: the EE's channel 7 in normal mode, its direction CHCR.DIR; the IOP's channel 2 in any mode, its
  direction its own DIR (D-145).
- **IRQ and end** on the IOP's tags: IRQ with DICR2's tag enable sets the channel's flag and the chain goes on; the end
  bit ends it, its completion flag set under DICR2's mask (D-144, D-68).
- **Holding and waking**: a channel whose FIFO cannot take or give what it needs is held, the EE's scheduling nothing
  and the IOP's DMAC idle; a FIFO changed by the other side wakes it, the EE's at the board's next time (an EE
  instruction at most later), the IOP's at its bus's next service.
- **Interrupts at each end**: the EE's are its DMAC's, CIS5, CIS6 and CIS7 and INT1 through their masks (§20.4); the
  IOP's are its DMAC's, the channels' DICR2 flags and the master flag's interrupt 3 (§22.5). Neither side's transfer
  raises anything on the other.

### 26.5 Timing

The documents give the FIFO's size and nothing on its timing. **Argued**, each a named loss and a dispute candidate
if an oracle measures it [A]: a word crosses at each side's DMA rate, a quadword a bus cycle on the EE (§20.3) and a word
an IOP cycle on the IOP (§22.5, D-68), with no cost of the SBUS's own; the EE's read or write of a SIF register costs
nothing beyond its instruction, and the IOP's costs an I/O register's two cycles (D-63); a woken EE channel starts at
the board's time, up to an instruction after the IOP's change. No new clock figure: the stage adds nothing to `clock`.

### 26.6 What is not modelled

Each a named loss, and a dispute candidate if an oracle measures it [A]:

- The manual's one FIFO shared by the three transfers (EEUM §5.3.3) and the deadlock it warns of (D-140).
- INT_SBUS, its HOLD latch in timers 0 and 1, and INT_SFIFO ("Error detection during SFIFO transfer"): no document says
  what raises them.
- The IOP's reset by the EE's CTRL bit 19, which the referee shows (D-141): the IOP's boot path is S24's.
- CTRL's further bits as the referee shows them (D-141); BD6's meaning.
- An IOP packet whose size leaves part of a quadword: the words stay in the FIFO for the next packet; the EE's side
  waits for whole quadwords.
- A linked list or burst on the IOP's SIF2 that outruns its FIFO: those modes ask for their words a unit at a time as
  S21 built them, so the FIFO is not checked there.
- The SBUS's own timing and the EE core's waits on SIF registers (§26.5); the state format's new contents (S30).

### 26.7 The gate: the programs (measured 2026-10-25)

`tests/s22/` generates each program as a pair: an EE program for the R5900 (S7's form, entered at 00100000h, run from
kseg0, registers and buffers through kseg1) and an IOP program in S21's form (`tests/s21`'s `Program`, its quiescing
first). The EE program places the IOP program by S21's route and the two then step together through a mailbox of two
words in IOP RAM at 0017F000h, which the EE reaches through its window, every wait bounded; neither the handshake nor
the record depends on the SIF. The EE writes its record of words from 00200000h, waits for the IOP's done word,
records it, copies the IOP's record after its own and writes 5A99D00Eh at 001FFFF0h. Every expected word is derived in
the generator from the documents, written apart from `src/sif/`. On Sapphire the IOP is entered at its program once the
EE has written the exception vector, as a referee's IOP enters it at its next interrupt.

| Program | Graded words | What it covers |
|---|---|---|
| `s22-registers` | 28 of 29 | MSCOM and SMCOM written by their side, read by both, the other side's write ignored; MSFLG set by the EE and cleared by the IOP, SMFLG the reverse, a 0 changing neither; CTRL's documented bits on each side; the EE's CTRL with bit 8 raising nothing and with bit 18 setting the IOP's I_STAT bit 1, the interrupt taken with IM2 |
| `s22-sif1` | 75 | a normal-mode transfer read by an IOP chain without bit 8, its completion flag and interrupt 3; an EE chain stopped by its tag interrupt and resumed while the IOP waits; the EE first, eight of sixteen quadwords moved and the channel held until the IOP starts; last, an EE chain of cnt, ref and end read by an IOP chain with bit 8, its middle tag with IRQ setting DICR2's flag, the master flag and interrupt 3, the chain going on |
| `s22-sif0` | 39 | the IOP first, 30 of 64 words moved beside the EE's two-word tag and its channel held until the EE's destination chain starts; last, two IOP tags with bit 8 carrying the EE's cnt and end, the first with IRQ |
| `s22-sif0-normal` | 20 | an IOP tag without bit 8 into the EE's normal mode, the data alone crossing, TADR past two words, the completion flag and interrupt 3 |
| `s22-sif2` | 32 | the EE's channel 7 into the IOP's channel 2 in slice mode, then the reverse |

Two probes, `s22-probe-iop` and `s22-probe-ee`, record what the documents leave open about CTRL, BD6 and INT_SBUS,
ungraded, for D-141; `examples/s22.rs` writes them with the gate.

**Tests** [M, `cargo test --release`]: `tests/sif.rs` **5 of 5**: the four gate tests, the five programs each equal to its
record word for word (every FIFO drained at the end of `s22-sif1` and `s22-sif0`), and `a_full_fifo_holds_the_ees_channel_until_the_iop_takes_words`
(the EE's SIF1 of twenty quadwords stopping at eight with nothing scheduled, the rest crossing once the IOP's channel
starts, every word placed); on the tree with S13, S16 and S23 beneath, lib **41 of 41** (the two `sif` tests among
them); `tests/iop.rs` **13 of 13**,
`each_dma_mode_reaches_its_stand_in` now reading SIF2's and SIF0's FIFOs and filling SIF1's with a quadword tag (the
S21 program unchanged); the crate's other tests unchanged: `cop0` 19, `dmac` 17, `fpu` 9, `gs` 12, `interpreter` 23,
`micro` 12, `mmi` 13, `programs` 3, `vu0` 14, `vu_micro` 24; lib, `dmac`, `iop` and `sif` also under `test-checked`;
clippy clean on every target. `SapphireTests` (WiseMan, the blast radius's filter): **5 of 5**, before and after the
rebase.

### 26.8 On the referees (measured 2026-10-25)

`examples/s22.rs` writes each pair as one ELF with its expected record; `ps2referee.py ram pcsx2 … --until
1FFFF0=5A99D00E 200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot), one process at a time.
Only the records were read. Play! cannot run an IOP program (§22.9).

| Program (graded words) | PCSX2 | What the differences are |
|---|---|---|
| `s22-registers` (28) | 3 | it takes the EE's write to SMCOM (two words); the IOP's CTRL bit 8 reads as the EE last wrote it (D-141); the flags, MSCOM, the interrupt from bit 18 and its taking equal |
| `s22-sif1` (75) | 8 | the EE's sixteen quadwords all leave with the IOP stopped (two words, D-140); the IOP's chain ends at the IRQ tag, the third packet never taken (five), and DICR bit 31 reads 0 with the DICR2 flag set (one) (D-144); the quadword tag, the tag interrupt's stop and resume, the completion flags and every other word equal (D-143) |
| `s22-sif0` (39) | 11 | the IOP's 64 words all leave with the EE stopped (two, D-140); the IOP's chain ends at the IRQ tag, the EE's destination chain left waiting (nine, D-144); the two-word EE tag places every word of both chains (D-142) |
| `s22-sif0-normal` (20) | 9 | without bit 8 the two words after the tag cross anyway and TADR steps 16, the EE's buffer reading EEEE0002h, EEEE0003h, 63000000h, 63000001h, 63000000h to 63000003h (D-142) |
| `s22-sif2` (32) | 26 | neither channel moves a word, both left running (D-145) |

So **PCSX2 is a witness for the mailbox registers, SIF0's two-word EE tag, SIF1's quadword IOP tag, the EE's tag
interrupt across the SIF and the IOP's completion flags**, and for nothing on the FIFOs' depth or SIF2. Its first runs
showed that a disagreement which leaves words in a FIFO spoils every later part of a program, so each program puts the
part PCSX2 is known to stop on last, and the case without bit 8 has a program of its own.

### 26.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | the EE's write to SMFLG setting its bits instead of clearing them | `the_mailbox_registers_pass_values_and_flags_both_ways`, `the_flags_set_on_one_side_and_clear_on_the_other` |
| F2 | SIF0 losing the last quadword in its FIFO, the EE taking zeros for it | `sif0_carries_the_iops_packets_to_the_ee_with_the_ees_tags` |
| F3 | the EE's CTRL bit 18 raising the EE's INT_SBUS instead of the IOP's interrupt 1 | `the_mailbox_registers_pass_values_and_flags_both_ways` |

Each was applied, `tests/sif.rs`, `tests/iop.rs` and the lib's tests run, and the source restored [M].

### 26.10 What the plan got wrong, or left open

- **§1.1's SIF row** calls the unit "Exact"; the documents give the registers' names, the FIFO's size and SIF0's tag
  sentence, and no timing, CTRL or BD6 rule, and no driver for INT_SBUS. The stage is exact where they speak, and
  D-140 to D-145 say where they do not.
- **§3.2's row for the SIF** names the referees as the secondary oracle: one lineage, PCSX2, and a witness for only part
  of the unit (§26.8).
- **S21's D-68** left a chain into memory's tags to this stage; they are a quadword (D-143), and the chain's units now
  stop where the peripheral does.
- **§5.1's catch-up**, "the IOP at SIF registers and SIF DMA": the IOP is brought to the board's time after every EE
  instruction (S21) and now before each DMAC event too, so the EE's register accesses and its DMAC both meet the IOP at
  their own time.

### 26.11 Questions this stage leaves

1. **Q1 [routine], the FIFOs' depth (D-140).** *Recommended*: keep the manual's 128 bytes a transfer; PCSX2 holds no
   FIFO and so cannot say; S24's BIOS and S28's games stall on it only if they depend on it.
2. **Q2 [routine], the EE's CTRL bit 18 (D-141).** *Recommended*: keep the IOP's interrupt 1 from it, the one witness's
   reading and the only driver that interrupt has, until S24's BIOS shows otherwise.
3. **Q3 [decision], the IOP's reset by the EE's CTRL bit 19 (D-141).** The referee resets the IOP on it; the BIOS's
   reboot of the IOP (a disc's own modules, `Sapphire_Plan.md` O9's subject) needs some reset. *Recommended*: S24 models
   the IOP's reset with its boot path, on this bit if its BIOS writes it, since S24 is where the IOP first boots.
4. **Q4 [routine], the IRQ tag (D-144).** *Recommended*: keep ps2tek's reading, the chain going on, against PCSX2's stop;
   S24's BIOS modules show which they rely on.
5. **Q5 [routine], SIF0 without bit 8 (D-142).** *Recommended*: keep ps2tek's reading; a program that sends SIF0 without
   bit 8 into a destination chain would show PCSX2's reading wrong at once, so the BIOS presumably never does.
6. **Q6 [routine], INT_SBUS (D-141).** *Recommended*: leave it undriven until a document or S24's BIOS names its source.

### 26.12 What was read for this stage

`Sapphire_Plan.md` §1.1 and §2 (the SIF rows), §3.2, §5.1, §7 (S22 and S24) and §11; this page's §20 and §22;
`Sapphire_Disputes.md` (the rules, D-34 to D-42, D-60 to D-68); `Sapphire_Referees.md` §4.3; the crate's `dmac.rs`,
`board.rs`, `iop/` and `tests/s11`, `tests/s21` (the project's own code); `ps2referee.py`'s usage. Sony's *EE User's
Manual*: §1.2 (the built-in devices' table and §1.2.9), §2.1.2, §2.2.1's register map, §3.1, chapter 4's Tn_HOLD, §5.1,
§5.3.2, §5.3.3 and §5.9's tables; the *EE Overview* §2.6 and §3.1.2. ps2tek at `1c91660`: "SIF Registers" and "SIF DMA"
(lines 4209-4292), the EE and IOP I/O maps' SIF and DMA rows (lines 150-260), "IOP DMA" (lines 3960-4110), "IOP
Interrupts" (lines 3905-3960), "EE Timers" (lines 1180-1235), "EE Interrupt Controller" (lines 3095-3130), the headings
of "SIF RPC Basics" to "SIF RPC Commands" as a search listed them (lines 4294-4447, not read further), "BIOS IOP SSBUSC"'s
export list (lines 5415-5440, X.4, cleared), and a search's one-line hit at line 5086 (X.1, excluded for replacement
writers; this writer writes no open BIOS and no stage modelling the IOP's kernel interface). No emulator, SDK, loader or
ps2autotests source and no firmware; the BIOS was only passed to PCSX2 by path, and the probes' first words, the
registers as its firmware left them, were not used.

### 26.13 What this stage does not cover

- **The SIF RPC protocol** (ps2tek "SIF RPC Basics" onward): S24's. It is software, the BIOS's SIFMAN, SIFCMD and the EE
  kernel's SIF calls running on the two processors, and attaches to nothing in `sif` but what a program reaches: the
  registers (the receive addresses in MSCOM and SMCOM, the boot flags in MSFLG and SMFLG), SIF0 and SIF1's chains, and
  the interrupts at each end. S24 needs no new interface from this stage; a high-level form of it would sit above the
  DMACs, not in `sif`.
- The IOP's reset and boot (S24), the CDVD (S23), SIO2 (S25) and the SPU2 (S26) on the IOP's other channels.
- The machine running the two processors (S10, S24), the SIF's state in `SPH2` (S30), its debugger view (S31) and the
  cost of the IOP's catch-up at each EE DMAC event (S32).

## 27. Stage S17, the GS's primitives, shading, scissoring and depth (2026-10-25)

*Numbered §27 and its disputes from D-160, since S13 (§23), S23 (§25) and S22 (§26) are built beside it.*

### 27.1 In one paragraph

The plan's §7 row S17, on the tester's go of 2026-10-25. The GS draws: **`gs::draw`** keeps the vertex queue that the XYZ
registers advance, takes the attributes from PRIM or PRMODE as PRMODECONT says, and at the drawing kick reads the
environment of the primitive's context; **`gs::raster`** decides which pixels each of the seven primitive kinds covers
and the colour and Z each takes, flat or Gouraud, exactly in integers; **`gs::pixel`** runs each pixel through the
pipeline in the documented order (scissoring and SCANMSK, the places of texturing, fog and the alpha tests, the depth
test, the places of blending, dithering and clamping, then format conversion, FBMSK and ZMSK), writing through
`gs::local`, the layout's one owner, in every colour and Z format FRAME and ZBUF may name. Eleven programs of the
project's, 225,280 graded record words, each read back raw as PSMCT32 by S16's route, are equal on Sapphire to the record
of a model written from the manual apart from `src/gs/`, which also draws 2,400 generated primitives as the source does.
**PCSX2 equals four programs word for word** (points, sprites, scissoring with SCANMSK, Z beyond its range) and agrees
with every other picture on which pixels are drawn and which pass the depth test, save four line ends on a diamond's
edge (D-161); it departs only in the last step of Gouraud colour (D-160) and the low bits of interpolated Z (D-164).
Play! draws through its own GPU renderer and its local memory never sees the drawing, so it is no witness for this stage
(§27.7). Disputes D-160 to D-165. No emulator, SDK or loader source, no ps2autotests source and no firmware was read.

### 27.2 What was built

- **`src/gs/draw.rs`**: `Vertex` (coordinates in sixteenths, Z, RGBAQ, ST, UV, the fog coefficient as they stood at the
  vertex kick); `Kind`, PRIM's seven types and the prohibited eighth; `Attributes`, bits 3 to 10 of PRIM or PRMODE;
  `Queue`, emptied by PRIM, which a strip leaves holding its last two vertices, a fan its first and last, a line strip
  its last, the rest nothing, whether the queue filled by a drawing kick (XYZ2, XYZF2) or without one (XYZ3, XYZF3,
  §3.2.5); `Environment`, the context's FRAME, ZBUF, SCISSOR and TEST and SCANMSK, read at the drawing kick, with
  XYOFFSET subtracted then (D-162). XYZF2 and XYZF3 take a 24-bit Z and write their fog field into FOG (ps2tek "GS Vertex
  Attributes").
- **`src/gs/raster.rs`**: the coverage of §27.3 and the interpolation, edge functions in 64-bit integers, Z through
  128-bit products; loops over the bounding box within the scissoring area; no allocation on any path.
- **`src/gs/pixel.rs`**: the pipeline; `pack16`, RGBA32 to RGBA16 and FBMSK's corresponding bits (§3.9.4, §3.9.5);
  `z_max`, each Z format's range (D-163).
- **`src/gs/mod.rs`**: PRIM empties the queue and the four XYZ registers kick it, in the order the host interface
  delivers writes, so a stalled SIGNAL holds drawing behind it as it holds everything else.
- **No new cross-component dependency**; `gs::local` is unchanged.

### 27.3 The readings the documents leave open

[R *GS User's Manual* §2.3 to §2.5, §3.1 to §3.3, §3.7, §3.9, chapter 7's pages for PRIM, PRMODE, PRMODECONT, RGBAQ,
XYZ2, XYZ3, XYZF2, XYZF3, XYOFFSET, SCISSOR, SCANMSK, TEST, FRAME and ZBUF; the *Restriction*'s "Depth Test"; ps2tek "GS
Primitives", "GS Vertex Attributes", "GS Frame and Z Buffers", "GS Tests and Pixel Control"; A]:

- **Pixel centres** are where window coordinates are whole (§2.4.4); a vertex's coordinates are 12.4 fixed point less
  XYOFFSET, so signed.
- **Point**: "the pixel closest to the specified XY coordinates"; a half rounds up [A; PCSX2 agrees at all 128 points of
  `s17-points`, among them every tie].
- **Sprite**: centres in [x0, x1) × [y0, y1), figure 3-8's top and left in, bottom and right out; the corners in either
  order [A]; colour and Z the second vertex's (§3.2.8).
- **Triangle**: centres inside, or on a left or top side but not a right or bottom one (figure 3-6), by the edge
  function's sign and its inward normal; a triangle of no area draws nothing [A].
- **Line**: figure 3-5's diamonds, |dx| + |dy| < 1/2 about each centre, whose pixels a line passes through are drawn, the
  start's included and the end's not. In the source this is a DDA, one pixel in each column of the major axis (x for
  |dx| ≥ |dy|): the one whose diamond holds the line's crossing of that column, a crossing exactly at a corner shared by
  two going to the lower (x-major) or right (y-major) one; the columns run from the start's pixel when the start lies in
  a diamond, else from the first column crossed, to the column before the end's pixel when the end lies in one, else to
  the last column crossed. An end on a diamond's edge lies in none (D-161). The model draws instead every pixel whose
  diamond the segment meets, by the exact minimum of |dx| + |dy| along it; the two agree on every program and every
  generated line, so the DDA is the area reading and not a second rule. A line of no length draws nothing [A].
- **Colour**: the last vertex's when flat ("the vertex color information set immediately before each drawing kick",
  §3.2.8); when Gouraud, the vertices' colours interpolated linearly at the centre, exactly, rounded down (D-160), along
  a line by the major axis, continued past an end and held to 0 to 255 (D-165). Points and sprites are flat whatever IIP
  says (§3.2.2).
- **Z**: interpolated so whatever the shading, exactly and rounded down (D-164); a sprite's is its second vertex's.
  Beyond the Z buffer's format it is clamped to the format's largest value, for the test and the write (D-163).
- **XYOFFSET** at the drawing kick, by the drawing's context (D-162); the attributes, CTXT included, at the drawing kick
  from PRIM or PRMODE as PRMODECONT stands then (§3.2.6). PRMODECONT is 0 after the GS's reset, so PRMODE's attributes
  are used until it is written [A]: every program writes it first.
- **SCISSOR** inclusive (§3.7.1); **SCANMSK** 10 draws odd rows only, 11 even only, 01 (reserved) as 00 [A; PCSX2
  agrees].
- **The depth test** (§3.7.4): NEVER, ALWAYS, GEQUAL and GREATER against the Z buffer's value in its format; ZMSK keeps
  the buffer; ZTE 0 neither tests nor writes, which the *Restriction*'s "Depth Test" equates with ZTE 1, ALWAYS and ZMSK 1
  [PCSX2 agrees in every Z format]. ZBUF's four PSM bits are the Z codes' low bits; an undefined one reads and writes no
  Z [A]. The frame is written before the Z buffer [A: they meet only when the buffers overlap].
- **FRAME** may name a Z format (FRAME's page lists them); its pixel then takes the colour twin's bits (PSMZ32 as
  PSMCT32, PSMZ24 as PSMCT24, PSMZ16 and PSMZ16S as the 16-bit packing) in the Z layout [PCSX2 agrees in all four]; a
  reserved texture code draws nothing [A]. FBP and ZBP count 2,048 words, 32 blocks; the Z buffer's width is FRAME's.
- **Format conversion and FBMSK** (§3.9.4, §3.9.5): RGBA16 keeps bits 7..3 of each colour and bit 7 of alpha, and FBMSK
  is packed the same way, a 1 keeping the frame's bit; PSMCT24 and PSMZ24 write 24 bits, the upper byte kept.

### 27.4 The interfaces: where the later stages attach

- **Texturing (S18)** reads `Vertex::st`, `uv`, `rgbaq`'s Q and the context's TEX registers; it is interpolated in
  `raster` beside colour and slots into `pixel::draw` before the tests, where the pipeline's comment places it. Fog (S18)
  takes `Vertex::f`.
- **The other pixel operations (S19)**: the alpha test, the destination alpha test with AFAIL, blending with PABE,
  dithering (DTHE, DIMX), COLCLAMP, FBA and antialiasing's coverage and Z rule take the places `pixel::draw` marks;
  `Environment` carries TEST whole, and the attributes carry ABE, AA1 and FIX.
- **The GIF (S15)** drives the drawing registers through `HostInterface::write` as it does the others.
- **The device path (S44 to S46)** is graded against this rasteriser, whose interpolation is exact (§5.13's C5 is the
  measure of any departure).

### 27.5 Cost

Not measured for speed; the hot path allocates nothing, decides coverage by integer edge functions per pixel of the
bounding box, and reads memory through `gs::local` per pixel. S43 to S46 own the GS's speed.

### 27.6 The gate: the programs (measured 2026-10-25)

`tests/s17/` writes each program as an S16 script: the registers every program sets first (PRMODECONT 1, no SCANMSK,
dithering, PABE, FBA or blending, no depth test or Z write), its buffers filled with data from S16's generator through
IMAGE so that every pixel written shows, its drawing through A+D, and every page drawn read back local to host raw as
PSMCT32; `s16::program_for` renders it as a bare EE program with the record `tests/s17/manual.rs` gives, written apart
from `src/gs/` on S16's model of memory. A pixel drawn twice is shown, where it matters, by drawing with GREATER over Z
already written at the primitive's own Z, so that the first primitive to reach a pixel keeps it.

| Program | Graded words | What it covers |
|---|---|---|
| `s17-points` | 4,096 | 128 points at every fractional X and eight fractional Ys, the halves among them, under a scissoring area one or two pixels in; points about window 0, −0.5 rounding to 0 and −0.5625 to −1 |
| `s17-sprites` | 4,096 | 32 sprites with sides on, just before, just past and between centres, half given with the second vertex at the top left; one with no centre, one of no width, two across the scissoring area |
| `s17-lines` | 12,288 | 32 lines in 16×16 cells, Gouraud and flat by turns: level, upright and diagonal on centres and on halves, every octant both ways, slopes of a half, a diagonal through diamonds' corners, ends at a corner, a line inside one diamond, one of no length; a self-crossing line strip and lines meeting end to start, drawn twice-proof, with their Z |
| `s17-triangles` | 8,192 | 32 triangles in pairs, twice-proof: right triangles on centres both ways round, quads split on either diagonal on and off centres in both orders, level and upright sides through centres on both sides, slivers, a degenerate triangle, one with no centre; a strip, a fan, a strip with a triangle skipped by XYZ3, a strip whose shading PRMODE changes; Gouraud triangles; large triangles across a narrowed scissoring area |
| `s17-gouraud` | 12,288 | a Gouraud triangle over a whole page from colours at both ends of their range; a quad with Z from 10000000h to F0000000h, its PSMZ32 buffer read back; 24 Gouraud lines radiating; the triangle into PSMCT24 and PSMCT16 |
| `s17-formats` | 16,384 | every format FRAME may name (four colour, four Z) as a page, a Gouraud triangle and sprites under FBMSK 0, 0F3CA5F0h, the alpha bit, every bit, and all but the alpha bit |
| `s17-depth-32` | 73,728 | PSMZ32 and PSMZ24 under PSMCT32, PSMZ24 under PSMCT24, each with NEVER, ALWAYS, GEQUAL, GREATER (Z written), GEQUAL with ZMSK, and ZTE 0: a Gouraud-Z triangle spanning the format's range over generated Z, a sprite writing Z = M, then sprites at M and M − 1 under the test; frame and Z read back |
| `s17-depth-16` | 73,728 | the same for PSMZ16S under PSMCT32 and PSMCT16S, PSMZ16 under PSMCT16 (both groups of §2.5.4) |
| `s17-zrange` | 8,192 | Z = 01234567h into PSMZ24 and PSMZ16, a triangle with Z to FFFFFFFFh, a GREATER sprite at 00FFFFFFh |
| `s17-scissor` | 6,144 | SCISSOR's four bounds under a sprite larger than the page; SCANMSK 10, 11 and 01; SCAX0 above SCAX1; a triangle across the area; a sprite 128 pixels wide into a buffer one unit wide, its right half on the page below (§8.2) |
| `s17-kick` | 6,144 | PRIM between a triangle's vertices; XYZ3 completing a triangle; XYZF2's 24-bit Z and fog field; RGBAQ after the last vertex; PRMODECONT 0 with PRIM's attributes ignored; context 2 by PRIM's CTXT (FRAME, SCISSOR, XYOFFSET, TEST, ZBUF), and by PRMODE in a strip (§3.3.2); type 7; a line strip begun again by PRIM; a point through XYZ3 |

**Tests** [M, `cargo test --release`]: `tests/draw.rs` **13 of 13** — the eleven programs, each equal to its record word
for word; `generated_primitives_draw_as_the_model_draws_them` (six rounds of 400 primitives of every kind and both
shadings, vertices anywhere about a page and a little outside it, in odd rounds on halves and whole pixels only, where
the ties fall, Z over its whole range, drawn with GEQUAL into PSMZ32 over generated data, every pixel and every Z equal
between the source and the model); `the_programs_assemble_and_their_records_end_in_the_done_word`. The crate's other
tests unchanged, on the tree with S13, S23 and S22 beneath: lib **41 of 41**, `cop0` **19 of 19**, `dmac` **17 of 17**,
`fpu` **9 of 9**, `gs` **12 of 12**, `interpreter` **23 of 23**, `iop` **13 of 13**, `micro` **12 of 12**, `mmi`
**13 of 13**, `programs` **3 of 3**, `sif` **5 of 5**, `vu0` **14 of 14**, `vu_micro` **24 of 24**; `draw` and `gs`
also under `test-checked`; clippy clean. WiseMan with `blast.sh`'s filter (`SapphireTests`) **5 of 5**.

### 27.7 On the referees (measured 2026-10-25)

`examples/s17.rs` writes each program as an ELF with its expected record and a background record (the same script with
no vertex kicked), so that a difference reads as a pixel the referee did not draw, drew where the reading does not, or
drew otherwise; `ps2referee.py ram … --until 1FFFF0=5A99D00E 200000:<words>` read each from PCSX2 2.8.2 (Americas BIOS,
fast boot, software renderer) and Play! 0.72, one process at a time. Only the records were read.

| Program (words changed by drawing) | PCSX2: words differing | What they are |
|---|---|---|
| `s17-points` (117) | 0 | — |
| `s17-sprites` (712) | 0 | — |
| `s17-lines` (2,725) | 27 | 4 ends on a diamond's upper-right edge (D-161), 23 one-step Gouraud values (D-160) |
| `s17-triangles` (2,586) | 62 | one-step Gouraud values; every pixel's coverage equal |
| `s17-gouraud` (7,284) | 1,310 | one-step colour values, in PSMCT16 one step of a 5-bit channel (D-160); 1,037 PSMZ32 words by one or two (D-164) |
| `s17-formats` (9,912) | 118 | one-step Gouraud values; every format and mask equal |
| `s17-depth-32` (47,135) | 1,815 | one-step colours; 869 PSMZ32 words by up to 203, PSMZ24 by one (D-164); every depth decision equal |
| `s17-depth-16` (47,022) | 59 | one-step colours and Z |
| `s17-zrange` (6,032) | 0 | — (D-163) |
| `s17-scissor` (1,780) | 0 | — |
| `s17-kick` (2,332) | 67 | one-step Gouraud values; XYOFFSET at the drawing kick (D-162) |

So **PCSX2 is a witness for every coverage rule** (points' ties, sprites', triangles' sides, lines' interiors and the
corners their columns cross), the vertex queue and its kicks, the contexts, scissoring, SCANMSK, every frame and Z
format and FBMSK, every depth test with ZMSK and ZTE 0, and Z's clamp; **not for interpolation's last step**, where its
arithmetic departs by one either way near whole values, nor for line ends on an edge.

**Play! 0.72 shows no drawing at all**: every program's pictures read back from it as their background, while its
local-to-host transfer reads the data the programs wrote by IMAGE exactly, as at S16. Its GS draws through a GPU
renderer (its home folder holds Mesa's shader caches) that does not write the drawing back to the local memory the
transfer reads [A: inferred from the outputs; its source not read]. The S3 decision's form C therefore gives no Play!
picture of drawing, and the second lineage for S17 to S19 needs the reserve the tester kept at S3, the probe's headless
OpenGL context for `play_libretro` (`Sapphire_Plan.md` §10, `Sapphire_Referees.md` §6's form A), whose pictures would be
a GPU's rasterisation (§27.10 Q1).

### 27.8 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | A triangle's right sides drawn (`top_left` true for every slanted side) | `triangles_draw_left_and_top_sides_…`, `the_vertex_queue_prmode_…`, `generated_primitives_…` |
| F2 | Gouraud weights stepped in 8 fractional bits instead of exactly | `gouraud_colour_and_z_…`, `triangles_…`, `every_frame_format_…`, both depth tests, `the_vertex_queue_…`, `generated_primitives_…` |
| F3 | GEQUAL tested as GREATER | `every_depth_test_holds_in_psmz32_and_psmz24`, `every_depth_test_holds_in_psmz16_and_psmz16s` |

Each was applied, `tests/draw.rs` run, and the source restored [M]. The generated primitives do not see F3, as expected:
their Z, random over 32 bits, never equals the buffer's; the depth programs' sprites at M are there for it.

### 27.9 What is not modelled

- **Drawing time**: a primitive is drawn whole at its drawing kick; FINISH is set at once as before. S43 owns threads.
- **Texturing, fog, the alpha tests, blending, dithering, colour clamp, FBA, antialiasing** (S18, S19): their places in
  `pixel::draw` are marked; AA1's rule that lines and partly covered pixels write no Z is S19's.
- **FIX** (fragment value control) and **FST** are carried in the attributes for S18.

### 27.10 What the plan got wrong, or left open

- **§5.4 and the gate's "each picture equal to the referees'"**: the documents give the coverage rules but no precision
  for the DDA, so no exact referee exists for Gouraud's last step or Z's low bits; the exact reading is the reference and
  PCSX2's departures are logged (D-160, D-164).
- **S3's form C for Play!** holds for transfers (S16) and not for drawing (§27.7): the plan's G3 asks for agreeing
  lineages, which S17 to S19 cannot have without the reserved context.
- **§3.2's GS row** names "every rasterisation rule" as thin: the manual's figures give them all but the edges of a
  line's diamonds (D-161) and the arithmetic's precision.

### 27.11 Questions this stage leaves

1. **Q1 [decision], a second lineage for the GS's pictures.** Play! standalone shows no drawing through its local memory.
   *Recommended*: build the probe's headless OpenGL context for `play_libretro` (the S3 reserve, form A, 300–400 lines,
   one GPU test at a time on the RX 6800 only) before S19, and grade S17's programs on it then; until then PCSX2 stands
   alone, named as such. It is GPU work, so only after the tester judges the desktop safe for it.
2. **Q2 [routine], interpolation's precision (D-160, D-164).** *Recommended*: keep it exact; revisit with the second
   lineage, and at S44–S46 measure the device path against the exact reference as C5 asks.
3. **Q3 [routine], line ends on a diamond's edge (D-161).** *Recommended*: keep the edges in no diamond; PCSX2's
   upper-right edge is one emulator's choice, and four line ends in 2,725 drawn words the whole of its evidence.

### 27.12 What was read for this stage

`Sapphire_Plan.md` §2, §3.2, §5.4, §5.12, §5.13 (C5, C6), §7 (S16 to S20), §8, §10 and §11; this page's §24;
`Sapphire_Referees.md` §2, §3 and §6; `Sapphire_Disputes.md` D-100 to D-107. Sony's *GS User's Manual* (Version 5.0)
§2.3 to §2.5, chapter 3's §3.1 to §3.3 whole, §3.6 to §3.9, and chapter 7's pages for PRIM, PRMODE, PRMODECONT, RGBAQ,
SCANMSK, SCISSOR, TEST, XYOFFSET, XYZ2, XYZ3, XYZF2, XYZF3, FRAME, ZBUF and FOG, figures 3-3 to 3-8 as pictures; the
kit's *Restriction* document whole. ps2tek at `1c91660`: "GS Register List", "GS Primitives", "GS Vertex Attributes",
"GS Frame and Z Buffers", "GS Fog", "GS Alpha Blending", "GS Tests and Pixel Control", "GS Special Effects and
Optimizations". No emulator, SDK or loader source, no ps2autotests source and no firmware.

### 27.13 What this stage does not cover

- Texturing (S18), the other pixel operations (S19), the PCRTC (S20).
- The GIF (S15): until then nothing on the EE's side draws on Sapphire; the tests drive the host interface.
- The state format's new contents (the vertex queue): `SPH2` gains them with the rest (S10, S30).

## 28. Stage S14, VIF0 and VIF1 (2026-10-25)

*Numbered §28 and its disputes from D-180, since S22 (§26, from D-140), S17 (§27, from D-160) and S20 (§29, from D-200)
are built beside it.*

### 28.1 In one paragraph

The plan's §7 row S14, on the tester's go of 2026-10-25. The crate gains **`vif`**, VIF0 and VIF1 written from the *EE
User's Manual* (EEUM) chapter 6, with ps2tek's "Vector Interface (VIF)" beside it: each VIF takes a stream of words
through its FIFO (8 qwords on VIF0, 16 on VIF1), from the DMAC's channel 0 or 1 or from the EE's stores, and runs every
VIFcode in it: the registers it sets, MPG into micro memory, UNPACK into data memory in all thirteen formats with sign
and zero extension, the write mask, the addition modes and CL and WL's skipping and filling writes (`vif::unpack`), the
MSCAL family starting the VU through S13's `Vpu::start` with VIF1's double buffer and the TOP and ITOP that XTOP and
XITOP read, the FLUSH family's and MPG's waits for the VU, DIRECT, DIRECTHL and MSKPATH3 ending at **`vif::Gif`**, the
interface the GIF (S15) implements, the i bit's interrupt and stall, FBRST and ERR's masks; and VIF1, reversed by FDR,
carries the GS's local-to-host data to the DMAC through S16's host interface. S11's stand-ins at channels 0 and 1 are
gone. Seven bare programs of the project's, 4,190 graded record words, are equal on the interpreter to what
`tests/s14/manual.rs` gives from chapter 6; four of ps2autotests' five `dma/vif` texts of the stage's gate are reproduced
on the VIF directly in the shape their lines show (`stcycl`'s 171 cases a VIF, `stmod`'s 8, `unpack`'s 24, `intstall`'s
3), and the consoles' texts decide or bound seven readings the manual leaves open (D-180 to D-182, D-185 to D-187,
D-189). On PCSX2 the five texts run against their consoles' outputs: `basic` and `stmod` equal,
`unpack` but one line, `stcycl` but 26 of 508, `intstall` differing; on the programs PCSX2 is a witness for UNPACK, the
mask, the modes, the double buffer and the MSCAL family's waits, not for the stalls, FBRST or the errors; Play! for
little. Disputes D-180 to D-194. No emulator, SDK, loader or ps2autotests source and no firmware was read.

### 28.2 What was built

- **`src/vif/unpack.rs`**: `Format` (CMD's low four bits, §6.3.4's table), `decode` (a vector's four fields from its
  bits), `Cycle` (§6.3.5: the write's address, whether it takes a vector, its cycle), `write` (§6.3.6 and §6.3.7: each
  field's specifier, MODE's addition and the Row it keeps), `Unpack` (one UNPACK in progress: the data's bits held as a
  little-endian stream across words, qwords and DMA packets, its writes in order).
- **`src/vif/mod.rs`**: `Vif`, one unit: its registers (EEUM §6.5 and §2.2.1's addresses), the FIFO with the qword
  being taken apart beside it, the command in progress, its own clock; `Gif` and `Path2StandIn`; `Reach` (the VPU and
  the GIF, what a VIF touches while it runs).
- **`src/vu/`**: `Micro::last` (the last pair's T stage, read without running), `Vpu::run_unit`, `Vpu::ready` (neither
  running nor stopped, VU User's Manual §1.3.3: "From the point of view of the VIF, there is no difference between the
  Run state and the Stop state") and `Vpu::last`; additive, with `Micro::last` defaulting to none so `tests/vu0.rs`'s
  scripted stand-in is unchanged.
- **`src/board.rs`**: the two VIFs and PATH2's stand-in on the board; `Ports`, the DMAC's peripherals with the VIFs on
  channels 0 and 1 and S11's stand-ins on the rest; `GifStandIn`, PATH2's stand-in with the GS's host interface for the
  reversed FIFO; the VIF's registers at 10003800h and 10003C00h and the FIFOs at 10004000h (mirrored to 10004FF0h) and
  10005000h; VIF0 and VIF1 brought to time before the DMAC looks at their FIFOs, before the EE reaches them or the VUs'
  memories, and at the end of each instruction before the VUs; INT_VIF0 and INT_VIF1 raised at the INTC.
- **No new cross-component dependency**: `sapphire` still depends on `emusen-native`, `emusen-mips`,
  `blackmoon-r3000` and `emusen-disc` alone.

### 28.3 The readings the manual leaves open

[R EEUM §6.3, §6.4, §6.5, §3.1; VU User's Manual §1.3; ps2tek "Vector Interface (VIF)"; M the console texts of §28.7;
A where marked]:

| Rule | Source | Implemented |
|---|---|---|
| The stream is words; a DMAtag sent by TTE gives only its upper 64 bits | §6.3.1's figures | D-193 |
| NUM 0 is 256 (UNPACK, MPG); IMMEDIATE 0 is 65,536 (DIRECT) | §6.3.2, the pages | `Unpack::new`, `Vif::fetch` |
| An UNPACK's data is a bit stream, its last word padded; a vector's elements in order from bit 0 | §6.3.4's figures, the length formula | `Unpack::take`, `next` |
| V2's Z and W repeat X and Y; V3's W is 0 | D-180 | `decode` |
| V4-5: R, G, B shifted up 3, A up 7 | §6.3.4's figure; the console's `f8 f8 f8 80` | `decode` |
| CL >= WL skips, CL < WL fills; WL 0 is a block that never ends | §6.3.5; D-181 | `Cycle` |
| A filling write decompresses at the stream's pointer without taking | D-183 | `Unpack::next`, `Vif::peek` |
| The write cycle picks MASK's row and Col, the fourth from cycle 4 on | §6.3.6's matrix | `Cycle::row` |
| MODE 3 writes unadded and keeps the input in Row | D-182 | `write` |
| Addresses wrap at the VU's memory, 256 or 1,024 qwords | the console's `stcycl` lines (`cl 253, wl 4` writes 4 into qword 0 on VIF0 and not on VIF1) | `Vif::store` |
| ADDR keeps 10 bits on both VIFs; FLG adds TOPS on VIF1 and is ignored on VIF0 | the UNPACK page | `Unpack::new` |
| MPG and DIRECT take their data where it falls; §6.3.3's alignment is the program's | §6.3.3 [A] | `Cmd::Mpg`, `Cmd::Direct` |
| MPG's instructions wrap at the micro memory's size | [A] | `Cmd::Mpg` |
| OFFSET sets TOPS = BASE and clears DBF; the MSCAL family TOP = TOPS, DBF reversed, TOPS the buffer DBF names | D-184 | `fetch`, `wait` |
| The MSCAL family copies ITOPS to ITOP; ITOP and ITOPS keep 10 bits | ITOP's pages; D-192 | `wait` |
| MSCNT starts at TPC | MSCNT's page; D-82 (TPC in doublewords) | `wait` |
| MPG, the MSCAL family and FLUSHE wait for the VU ready, neither running nor stopped; FLUSH, FLUSHA and MSCALF also for the GIF's paths | the pages; VU User's Manual §1.3.3 | `wait`, `Vpu::ready` |
| UNPACK does not wait for the VU | §6.3.8 (double buffering writes while the VU runs) | — |
| The i bit; MARK's does not stall | D-189 | `complete`, `fetch` |
| Undefined codes; VIF1's codes on VIF0; UNPACK 63h, 67h, 6Bh | D-188 | `fetch` |
| FBK at once, STP at the next VIFcode, STC clears VSS, VFS, VIS, INT, ER0, ER1 | FBRST's pages; D-190 | `fbrst` |
| RST | D-187 | `fbrst` |
| VPS and FQC | D-186 | `status` |
| NUM: VIF0 8 bits, VIF1 16 | D-185 | `num_bits` |
| MARK keeps 16 bits; the EE's write clears MRK | MARK's page; D-191 | `write` |
| The registers but MARK, ERR, FBRST and VIF1_STAT's FDR are read-only | §6.5 | `write` |
| The EE's register reads are answered at any time | §6.5 says "accessible only when the VIF is stopped" [A] | `read` |
| VIF1 reversed: the FIFO filled from the GS's host interface while FDR is set and D1_CHCR.DIR is clear; the VIF decodes nothing meanwhile; either direction alone moves nothing | EEUM chapter 5's channel table, VIF1_STAT's FDR; GS User's Manual §4.2.2 [A for the mismatch] | `request`, `get`, `run_to` |
| An EE store into a full FIFO is taken all the same; FIFO stores narrower than a qword are lost | §2.2.1 ("only qword-accessible") [A] | `Board::write` |

### 28.4 The timings: the documents' and the argued

No document times the VIF. **Argued**, each a named loss (D-194): the VIF runs at BUSCLK, one bus cycle a VIFcode and
one a write (an UNPACK qword, filled or read; an MPG instruction; a DIRECT qword, or the cycle PATH2 takes it), the words
of STMASK, STROW and STCOL inside their VIFcode's cycle; a microprogram the MSCAL family starts issues its first pair
`pipe::START` (3) EE cycles after the VIFcode's cycle, as VCALLMS's (§5.4.5's figures, D-85's argument); a wait on the VU
ends at the cycle after its last pair's T stage; the GS's local-to-host data fills the reversed FIFO at once. **Measured**
[M, `the_dmac_waits_on_a_full_fifo`]: an S-32 UNPACK of 256 words, 65 qwords into VIF1's FIFO, ends its DMA at 402 EE
cycles, the VIF's pace (392 for the 49 qwords it must take before the last 16 fit) and not the DMAC's 130.

**How a VIF keeps time** [A]. A VIF runs on its own clock and is brought forward like the VUs: by the board before the
DMAC arbitrates (so DREQ is the FIFO's room as of that cycle), by an EE access to its registers, FIFO or VU's memories,
and after each instruction, before the VUs, so that a VU never runs past a VIF write it should see. It schedules an
event with the board only while the DMAC waits on its FIFO and it can free a slot, so a transfer the FIFO paces is
exact without an event per word otherwise. A wait on the VU is noticed when the board next brings the VIF forward (an
event or the next instruction) and is then timed from the VU's last pair.

### 28.5 The interfaces and the board

- **`dmac::Peripherals`** on channels 0 and 1: `request` answers the FIFO's room (forward) or its data (reversed), and a
  zero holds the channel until the VIF frees a slot and the board wakes the DMAC at that cycle; `put` takes a qword
  (a DMAtag marked); `get` gives the reversed FIFO's qword.
- **`vif::Gif`** (S15 implements it): `path2(q, hl, at)` gives a DIRECT or DIRECTHL qword to PATH2 and answers the cycle
  it is taken or none while PATH2 cannot have the GIF (VGW set); `idle_at(path3, at)` the cycle PATH1 and PATH2 (and with
  `path3`, PATH3's requests) are idle for FLUSH, FLUSHA and MSCALF; `mask_path3(masked, at)` MSKPATH3; `host_read()` the
  GS's local-to-host qwords. Until S15 `Path2StandIn` takes every qword at once and records the mask, and the board's
  adapter reads the host interface of S16's `gs`.
- **The VPU** (S12, S13): `Vpu::start` for the MSCAL family, `Vpu::ready` and `Vpu::last` for the waits, the units'
  memories for MPG and UNPACK, and `Link`'s `top` and `itop`, set as the MSCAL family runs, for XTOP and XITOP.
- **The INTC**: INT_VIF0 and INT_VIF1 (bits 4 and 5) at an i bit's VIFcode's end or an undefined VIFcode.

### 28.6 The gate: the programs (measured 2026-10-25)

`tests/s14/` generates seven bare programs in S11's form (kseg0 code; constants, packets, the VUs' memories at
11000000h, the registers and the record through kseg1), each starting with COP2 usable, the DMAC as S11's start leaves
it, the INTC cleared, both VUs reset by FBRST and both VIFs reset with ERR's ME0 and ME1 set (§6.5's advice). Packets are
built in RAM and sent by normal-mode DMA, or stored into the FIFOs; the programs wait for the channel and for the VIF's
VPS and FQC to clear, bounded, and read back what the EE can see. Every expected word comes from `tests/s14/manual.rs`,
chapter 6 transcribed apart from `src/vif`.

| Program | Graded words | What it covers |
|---|---|---|
| `s14-unpack` | 1,164 | the 21 format and USN cases of the console's text, three vectors each over a memory filled first, on both VIFs; a write wrapping at each memory's end; an ADDR beyond VU0's 256 qwords; NUM, CODE, CYCLE after; the console's three dataless UNPACKs (VPS, CODE, NUM) |
| `s14-mask` | 1,112 | the console's eight `stmod` cases; every specifier in every write cycle in normal and difference modes, with Row and Col after; the m bit clear ignoring MASK; V2-16 in difference mode with Z and W from Col (§6.3.7's example 4); V3-8 in offset mode with W masked |
| `s14-cycle` | 1,064 | skipping (CL 4 WL 2, CL 3 WL 1, CL 5 WL 2), filling from Col and with only 0 specifiers (D-183), CL 0, the console's CL 253 WL 4, a VIF1 UNPACK wrapping at 1,024, WL 0 with CL 1 and 0; a MARK after each UNPACK, so a wrong count of data words shows |
| `s14-micro` | 352 | on both units MPG read back from micro memory; ITOP, ITOPS and XITOP (and XTOP on VU1); MSCAL arriving while a 300-pass microprogram runs, which waits; MPG arriving while it runs, which waits; FLUSHE before an UNPACK into the qword the microprogram writes last; MSCNT at TPC |
| `s14-dbuf` | 164 | VIF1's BASE and OFFSET, MSCAL, MSCAL, MSCNT and OFFSET again, each microprogram storing XTOP and XITOP at the qword TOP names, UNPACKs with FLG between, BASE + OFFSET past 10 bits; every double-buffer register and DBF after each step |
| `s14-regs` | 312 | MARK by VIFcode and by the EE; the console's `intstall` shapes with STAT, MASK and I_STAT, then STC; the stall holding a packet's later qwords in the FIFO with the DMA ended; FBK and STP with STC; RST and every register after; MII; ER1 and its interrupt, ME1; OFFSET on VIF0 and 63h; STROW through the FIFO; a chain with TTE |
| `s14-direct` | 22 | VIF1's DIRECT with NUM and CODE after; MSKPATH3; FLUSH, FLUSHA and MSCALF with a microprogram; DIRECTHL last; the GS's SIGLBLID the packets write by LABEL, graded on the referees only (6 words) |

**Tests** [M, `cargo test --release`]: `tests/vif.rs` **12 of 12** — the seven programs, each equal to its record word
for word (`s14-direct`'s also checking PATH2's six qwords in order and the PATH3 mask set and cleared); the console texts
of §28.7; the DMAC waiting on a full FIFO (§28.4); VIF1 reversed carrying a 64-pixel local-to-host transfer to memory.
`tests/dmac.rs` **18 of 18** on the real VIFs, S11's assertions on its stand-ins at channels 0 and 1 moved to the VIFs
(VIF0 receiving three qwords of NOPs, VIF1 seventy). On the tree with S22, S17 and S20 beneath, the crate's other tests
unchanged: lib **44 of 44**, `cop0` 19, `draw` 13, `fpu` 9, `gs` 12, `interpreter` 23, `iop` 13, `micro` 12, `mmi` 13,
`pcrtc` 5, `programs` 3, `sif` 5, `vu0` 14, `vu_micro` 24; lib, `vif`, `dmac`, `micro` and `vu0` also under
`test-checked`; clippy clean. The programs' ELFs are byte for byte those the referees ran before the rebase.

### 28.7 The console texts: `dma/vif/*` on the VIF directly, and on PCSX2 (measured 2026-10-25)

Each of the five needs the BIOS's kernel, so Sapphire runs them whole at S10. Their lines are transcribed in
`tests/s14/console.rs` (the texts only; the programs' sources are not read) and reproduced on a board driven through
the EE's map, in the shape each line shows:

| Text | Reproduced | The shape |
|---|---|---|
| `unpack` (54 lines) | the 21 "Simple UNPACK" lines and the three "No data" lines on both VIFs; "Set 4096 bytes" not (its shape is not in its line) | each format's elements 88h, 99h, AAh, BBh widened, one vector; the dataless UNPACK the last word of its qword |
| `stmod` (116) | all eight cases on both VIFs | Row 1000h, MODE 0 to 3, V4-32 of 0 to 15, CL = WL = 4, without the m bit and with the diagonal mask |
| `stcycl` (508) | all 171 "normal" cases on both VIFs, the "truncated" ones identical in the text | Col 1 to 4, every specifier 2, an S-32 UNPACK with the m bit and NUM 8 under each CL and WL over 64 cleared qwords |
| `intstall` (16) | the three status lines on both VIFs, in `s14-regs` | one-qword packets: NOP, MARK or STMASK with the i bit, then an STMASK |
| `basic` (10) | not reproduced (its lines do not show their packets) | — |

Run on PCSX2 2.8.2 through `ps2referee.py tty pcsx2 … --channel iop` (Americas BIOS) against the expected texts, the
consoles' outputs, after line ends are normalised and PCSX2's own `recVifUnpk` log lines dropped [M `diff`]:

| Text | PCSX2 |
|---|---|
| `basic` | equal |
| `stmod` | equal |
| `unpack` | one line: VIF0's NUM after a dataless UNPACK of 256 reads 256, the console's 0 (D-185) |
| `stcycl` | 26 lines: with CL 251 to 255 and WL 1 to 6 PCSX2 writes nothing where the console wraps the address at VU0's 256 qwords and goes on at VIF1's |
| `intstall` | five of six status lines: FQC counting the qword in hand, MRK kept across the reset, VIF1's MARK and STMASK cases not run; five DMA time-outs the console does not print (D-189) |

So **PCSX2's lineage reproduces the console on UNPACK's formats, the mask, the modes and the write cycles but for the
wrap**, and departs from it on the FIFO's count and the stalls.

### 28.8 On the referees (measured 2026-10-25)

`examples/s14.rs` writes each program as an ELF with its expected record; `ps2referee.py ram … --until 1FFFF0=5A99D00E
200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, one process at a time; `s14
explain` lists the records that differ.

| Program (records) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s14-unpack` (582) | 20 | 61 | PCSX2: V3's W, the next element in some alignments (D-180), VIF0's NUM 256 (D-185), VIF1's dataless UNPACK through the FIFO at VPS 3 (D-186); Play!: V2's Z and W 0, against the console (D-180), VIF1's NUM |
| `s14-mask` (556) | **0** | 38 | Play!: MODE 3 as no addition with Row unchanged (D-182), Col in some fields |
| `s14-cycle` (532) | 104 | 112 | PCSX2: V3's W; VIF0's CL 253 case, which it writes elsewhere (as in the console's lines it misses); WL 0, where it takes more words, the MARK after them among them, and the rest of the program is out of step (D-181); Play!: filling writes 0 (D-183), V2's Z and W, WL 0 |
| `s14-micro` (176) | 2 | 6 | PCSX2: VIF0's ITOP and XITOP 8 bits (D-192); Play!: ITOPS and ITOP read 0, MPG not waiting for the running microprogram |
| `s14-dbuf` (82) | 1 | 33 | PCSX2: TOPS 410h unwrapped (D-184); Play!: no double-buffer register |
| `s14-regs` (156) | 36 | 60 | both: MARK's write kept whole (D-191), FBK and STP not stopping (D-190), no stall at an undefined VIFcode (D-188); PCSX2: no ER1, FQC with the qword in hand and the DMA held (D-186, D-189), MRK kept across RST (D-187); Play!: no interrupt for the i bit and nothing resumed by STC, ER1 never cleared, RST keeping ERR |
| `s14-direct` (14) | 6 | 2 | PCSX2: a second DIRECT waits for the GIF without end (VGW), so MSKPATH3, the FLUSH family and MSCALF never run (a GIF matter, S15's); Play!: CODE without IMMEDIATE |

So **PCSX2 is a witness for UNPACK, the mask and modes, the write cycles but WL 0 and the wrap, the double buffer, ITOP
and the MSCAL family's waits, MPG's wait and TTE** (D-182, D-183, D-184, D-193 rest on it beside the console), and not
for the stalls, FBRST, the errors or the FIFO's count, where the console's `intstall` text and the manual stand against
it; Play! is a witness for little beyond TTE and the formats' common fields.

### 28.9 Named losses

- **The VIF's timing** (§28.4, D-194): argued throughout; a wait on the VU noticed at the next event or instruction.
- **ER0** (DMAtag mismatch) is never raised; the manual itself says its detection fails and asks programs to mask it.
- **V3-16 with ME1 clear**: the manual asks ME1 set for V3-16, implying a false VIFcode error in hardware; not modelled.
- **A filling write's peek** (D-183) reads 0 for words not yet in the FIFO; what the hardware reads there is unknown.
- **A store into a full FIFO** is taken; the EE would wait.
- **Reading the registers while the VIF runs** answers the present value; §6.5 makes it undefined.
- **VCALLMS beside a VIF's start** ("indeterminate", VU User's Manual §3.5.1) is not checked.
- **VIF1 reversed** takes the GS's data without timing, and the FIFO's direction is FDR's; a DMA direction that
  disagrees moves nothing.

### 28.10 The blast radius (measured 2026-10-25)

`blast.sh WiseMan sapphire-s14` names the Sapphire crate and WiseMan's `SapphireTests`: the crate as §28.6 gives it,
and WiseMan's filter **5 of 5** before and after the rebase onto S22's, S17's and S20's merges, each run once at CPU
weight 20, `LunaP.props` pointed at the sibling LunaP checkout by a local change that is not committed.

### 28.11 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | V4-16 sign-extending when USN asks zero extension (`decode` ignoring USN for vn 3 at 16 bits) | `the_unpack_console_text_holds`, `every_unpack_format_equals_section_6_3_4` |
| F2 | difference mode adding Row twice | `the_stmod_console_text_holds`, `the_mask_and_the_addition_modes_equal_sections_6_3_6_and_6_3_7` |
| F3 | MSCAL starting the VU without waiting for the running microprogram | `mpg_and_the_mscal_family_start_and_wait_for_the_vus` |

Each was applied to `src/vif/unpack.rs` or `src/vif/mod.rs`, `tests/vif.rs` run, and the file restored [M].

### 28.12 What the plan got wrong, or left open

- **VIF1's reverse direction** is the GS's local-to-host data through VIF1's FIFO (EEUM chapter 5's channel table: "VIF1
  channel (ID=1) can switch directions of transfer"; VIF1_STAT's FDR; the GS User's Manual §4.2.2), as §24.4 planned,
  not VU1's memory read back: VU1's memory reaches the EE through the map at 1100C000h (EEUM §6.2.3). Built so.
- **§1.1's VIF row** ("the stall behaviour as ps2autotests' `dma/vif/*` records it"): `intstall` records it in three
  lines a VIF whose packets their lines leave open; read as one-qword packets (D-189), the reading PCSX2's FIFO count does
  not share.
- **§3.2's row** names the referees as the secondary oracle: PCSX2 is one for the data path (§28.8) and not for the
  control path; the console texts carried the readings the manual leaves open.
- **S14's gate** names five folders of ps2autotests that need the BIOS's kernel: met on PCSX2 against their texts
  (§28.7) and, but `basic` and one `unpack` line, reproduced on the VIF in the shape their lines show; whole on Sapphire
  at S10.

### 28.13 Questions this stage leaves

1. **Q1 [routine], V3's W and the filling peek (D-180, D-183).** *Recommended*: keep W 0 and the peek, the console's
   single vector and PCSX2's fills; S28's games, whose V3 unpacks with masks set are common, would show a difference.
2. **Q2 [routine], WL 0 (D-181).** *Recommended*: keep the endless block, the reading both console texts fit; look again
   if a game sends WL 0.
3. **Q3 [routine], RST (D-187).** *Recommended*: keep what both lineages keep and clear MRK as the console's text shows;
   S10 runs `intstall` whole and settles whether the program resets between its cases.
4. **Q4 [routine], the stalls and errors (D-188 to D-190).** *Recommended*: keep the manual's FBK, STP, ER1 and i-bit
   stalls; neither lineage models them, and S10's `intstall` and S24's BIOS, which masks errors through ERR, exercise
   them.
5. **Q5 [routine], the VIF's timing (D-194).** *Recommended*: keep a bus cycle a VIFcode and a write until S32's bench
   shows a game whose timing depends on it.

### 28.14 What was read for this stage

`Sapphire_Plan.md` §1.1's VIF and GIF rows, §2, §3.2's VIF line, §5.1, §7's S14 and S15 rows and §11; this page's §20,
§21, §23 and §24.4; `Sapphire_Disputes.md`. Sony's *EE User's Manual* (Version 5.0) chapter 6 whole, §2.2.1's register
map, §3.1's interrupt table, chapter 5's channel table, §5.2.3's Figure 5-3, §5.10; the *VU User's Manual* §1.3, §1.4,
§3.5.1 and XTOP's and XITOP's pages; the *GS User's Manual* §4.2.2. ps2tek at `1c91660`: "Vector Interface (VIF)" (its
registers, commands and UNPACK). ps2autotests' `dma/vif/*.expected` (the consoles' texts) and the `.elf` files run as
black boxes on PCSX2. No emulator, SDK, loader or ps2autotests source and no firmware; no referee's log beyond the
programs' own lines.

### 28.15 What this stage does not cover

- The `dma/vif` texts on Sapphire: S10 (the kernel); `mskpath3` is S15's.
- The GIF behind PATH2 and MSKPATH3, and PATH1's busy time for FLUSH: S15 implements `vif::Gif` and `vu::Path1`.
- The VIFs' state in `SPH2` with the rest (S10, S30).

## 29. Stage S20, the PCRTC: video output and timing (2026-10-25)

*Numbered §29 and its disputes from D-200, since S14, S17 and S22 are built beside it (§26 to §28).*

### 29.1 In one paragraph

The plan's §7 row S20, on the tester's go of 2026-10-25. The crate gains **`pcrtc`**: the video timing (`pcrtc::timing`)
in place of S11's blanking stand-in, on the PCRTC's own clock VCK, which the *GS User's Manual*'s Figure 5-1 feeds from a
video clock of its own; the two read circuits (DISPFB1/2, DISPLAY1/2 with their offsets and magnifications, every
displayable format through `gs::local`) and the merge circuit (PMODE's EN1, EN2, MMOD, AMOD, SLBG and ALP, BGCOLOR), each
raster read at its line's start with the registers as they then stand; NTSC, PAL, 480p and 1080i, interlace in FIELD and
FRAME modes, CSR's FIELD; H-blank, V-blank and the V-sync to the EE's timers, the INTC, HSINT, VSINT and the IOP's
timers. The documents' table read at 13.5 MHz fixes the line and the field (D-200): 59.94 Hz interlaced, 59.83 Hz
non-interlaced, where both referees run exactly 60 Hz whatever SMODE2 says. **D-42 is closed**: PCSX2 raises H-blank at
twice NTSC's line rate, 525 in its field (D-201). Seven display programs of the project's are equal on Sapphire to the
pictures a model transcribed from chapter 5 gives; the memory they read back is equal on PCSX2 and on Play! (but Play!'s
PSMCT16S, as at S16), and **PCEE2's own software pictures equal Sapphire's pixel for pixel** for circuit 1 magnified and
for both interlace modes; it rounds the merge to the nearest (D-203), reads the display once a frame (D-205) and frames
two-circuit pictures its own way. A defect S22 suspected in S11's board was proven and fixed first (§29.9). EXTBUF's
feedback write is a named loss (§29.6). No emulator, SDK, loader or ps2autotests source and no firmware was read.

### 29.2 What was built

- **`src/pcrtc/timing.rs`**: `Mode` (a video mode's VCK as a fraction of the EE's clock, its line in VCK, its lines, how
  it scans, V-blank, H-blank, the V-sync, the picture with no circuit on, the aspect), the four modes `NTSC`, `PAL`,
  `DTV480P`, `DTV1080I`; `Timing`, the signals as functions of EE time, a change of mode or INT taking effect at the next
  field's start; `Edges` (H-blank's and V-blank's rise and fall, the V-sync); fields, their lines and rasters.
- **`src/pcrtc/mod.rs`**: `Circuit` and `Pmode` decoded from the registers; `merge`; `geometry`, the picture's rectangle;
  `Picture` with `frame_info`, the core ABI's description of it; `Pcrtc`, which begins each field, reads each due raster
  and turns SMODE2's INT and a mode set by `set_mode` into the timing.
- **`clock.rs`**: `VCK_54` (375/2048 of the EE's clock, 54 MHz exactly) and `VCK_74` (12375/49152, 74.25 MHz); the old
  `Standard`, `NTSC` and `PAL` names now the PCRTC's modes, `HBLANK` gone. `blank.rs` is deleted.
- **`board.rs`**: the PCRTC in the stand-in's place, its line reads among the board's events; VB_ON and the field's
  picture geometry at V-blank's rise, VSINT and FIELD at the V-sync; a privileged write passing SMODE2 to the timing and a
  changed timing to the IOP's copy; `Board::set_mode`. **`gs/mod.rs`**: CSR's FIELD; `vsync` takes the field.
  **`iop/`**: the IOP's timers follow a copy of the same `Timing`.
- **`tests/s20/`** and **`tests/pcrtc.rs`**, **`examples/s20.rs`** (§29.7); `ps2referee.py --screen` (§29.8).
- **No new cross-component dependency**; no surface outside Sapphire changed (§29.5).

### 29.3 The timing: the documents' and the chosen

**The documents'** [R *GS User's Manual* §5.1.2's table, §5.2.1, §5.2.2; ps2tek "EE Timers", "IOP Timers"; D-200]:
NTSC's 15.734 kHz line, 59.940 Hz interlaced and 59.82 Hz non-interlaced; PAL's 15.625 kHz, 50.000 and 49.76 Hz; 480p at
31.469 kHz and 59.94 Hz; 1080i at 33.750 kHz and 60.00 Hz; V-blank 22 lines in NTSC and 26 in PAL; the 13.5 MHz pixel
clock. Read at 13.5 MHz they are 858 and 864 pixels a line, 525 and 625 lines a frame, 263 and 314 lines a non-interlaced
field, every figure of the table to its printed digits (`the_four_rates_of_the_manuals_table_follow_from_13_5_mhz`).

**Chosen**, each a named loss and a dispute candidate if an oracle measures it:

1. VCK is four pixels, 54 MHz, in NTSC, PAL and 480p (the MAGH table's 2,560 VCK across a 640-pixel picture), and
   74.25 MHz in 1080i (1,920 pixels at MAGH ×1 in a 2,200-VCK line) [A].
2. A field starts at V-blank's rise (VB_ON); the V-sync, VSINT and FIELD's change, 4 lines in (D-202); V-blank ends 22
   or 26 lines in. 480p's V-blank is 45 lines and 1080i's 22 a field [A].
3. H-blank is high for the first 552 VCK of each NTSC line, 576 in PAL (BT.601's blanking), 276 in 480p and 280 in
   1080i, every line including those in V-blank; HSINT at its rise (D-201).
4. An interlaced odd field starts half a line in, its lines on the frame's H-syncs; the even field fills the even
   rasters (D-204). A non-interlaced field's FIELD alternates too.
5. A change of INT or of mode takes effect at the next field's start, as a new field 0 (even) [C]. After a reset SMODE2
   reads 0, so a program that sets nothing sees a non-interlaced field.
6. **SMODE1, SYNCH1, SYNCH2, SYNCV and SRFSH change nothing.** They have no fields in the *GS User's Manual*, ps2tek or
   the *EE User's Manual* (§24.11), and the kernel's mode setting writes them (§5.2.1, ps2tek's SetGsCrt) with values no
   permitted source gives. The mode is the board's (`Board::new`'s standard, `Board::set_mode`) and INT is SMODE2's; the
   brief's "from SMODE1/SYNCH/SYNCV as written" waits on a source for their fields (§29.11 Q1). VESA's modes, which only
   those registers select, are not modelled.

The figures live in `pcrtc::timing`'s four modes, and VCK's ratio to the EE's clock in `clock.rs`, D-0's one place.

### 29.4 The read and merge circuits

[R *GS User's Manual* §5.2.2, §5.2.3, §7.2; A where marked]:

- **A raster** is read at its line's start, every pixel at once, from memory and registers as they are then; a register
  written within a line shows from the next line (D-205).
- **A circuit** shows at VCK *v* and raster *r* when its EN bit is set and DX ≤ *v* ≤ DX + DW, DY ≤ *r* ≤ DY + DH. It
  reads pixel DBX + (*v* − DX) / (MAGH + 1) of buffer line DBY + *s* / (MAGV + 1), where *s* = *r* − DY, or (*r* − DY) / 2
  in FRAME mode when interlaced (§5.2.2's Interlace Read: FIELD mode "+0, +2, +4 / +1, +3, +5", FRAME "+0, +1, +2"),
  coordinates wrapping at 2048 as a transfer's do (D-106).
- **Formats**: PSMCT32 with its alpha, PSMCT24 with 80h, PSMCT16 and 16S with 80h or 0 by bit 15 and each 5-bit
  component shifted up three (§5.2.3); PS-GPU24 on circuit 1, three bytes a pixel along a PSMCT16 line, DBX in 16-bit
  pixels [A]; circuit 2 with PS-GPU24, and a texture format, read black with alpha 0 [A]; a depth format reads as the
  colour format of its width through its own layout [A].
- **The merge**: OUT2 is circuit 2's colour inside its rectangle and BGCOLOR elsewhere; OUT1 blends circuit 1 over OUT2,
  or over BGCOLOR with SLBG, by ALP with MMOD or by circuit 1's alpha doubled and clamped at FFh, rounded to the nearest
  (D-203); where circuit 1 gives nothing OUT1 is what it would blend over (D-206). AMOD selects OUT1's alpha, which only
  the feedback write would use.

### 29.5 The picture handed to the frontend

- **What it is** [C]: RGBA rows covering the union of the switched-on circuits' rectangles, clipped to the line and the
  frame's rasters, one row a raster (both fields' rasters woven when interlaced, each field writing its own), a pixel the
  finest circuit's VCK (at most 1,920 pixels across); with no circuit on, the mode's picture (640 × 448 NTSC, 640 × 512
  PAL, 640 × 480, 1920 × 1080), BGCOLOR throughout. The geometry is fixed at each field's start, a new one starting black,
  as Diamond's is (`Diamond_GPU.md` §7). Its aspect is 4:3, 16:9 in 1080i. `Picture::frame_info` gives the core ABI's
  `frame_info` (§6.6): size, stride, aspect.
- **What a television shows that it leaves out**: BGCOLOR beyond the circuits' rectangles (the border), and the field
  in flight (the picture is the woven frame). Both are the frontend's to add if a filter wants them: the CRT filter
  needs the active picture in microseconds (`Sapphire_Plan.md` §4.6), which is now 53.33 µs of a 63.56 µs NTSC line and
  53.33 of 64 µs PAL at BT.601's 720 pixels, the circuits' rectangles inside it.
- **Surfaces**: no ABI change, no change outside Sapphire. The machine (`machine.rs`, `v1.rs`) still hands S1's black
  picture, since it holds no board until S10; there `frame_info` and `frame` become the board's picture, machine info's
  `max_width` and `max_height` become `MAX_WIDTH` × `MAX_HEIGHT` (1,920 × 1,125, a value, not a field), and the frame
  rate the timing's (60,000/1,001 interlaced, which S1's constant already is; 59.83 Hz non-interlaced). The core ABI has
  no field indicator; a CRT filter that draws interlace by field would need one, a minor proposed in §29.11 Q2, not
  built.

### 29.6 What is not modelled

Each a named loss, and a dispute candidate if an oracle measures it [A]:

- **The feedback write** (EXTBUF, EXTDATA, EXTWRITE, EDWINT; §5.3): the registers are stored and nothing is written.
  The manual leaves it short of an implementation: its YCbCr matrix is given in real numbers with no fixed point or
  rounding, Cb and Cr are labelled crosswise ("Cr = (E0h) x Ecb"), and neither when EDWINT is raised nor which field a
  WRITE of 1 captures in each WFFMD is said.
- **SMODE1 and the synchronisation registers' fields**, VESA's modes, DPMS (§29.3, 6).
- **A change within a line**: a raster is read whole at its line's start.
- **The border and the field in flight** in the picture (§29.5).
- **The PCRTC's memory bandwidth**: reading costs the GS's drawing nothing.

### 29.7 The gate: the programs (measured 2026-10-25)

`tests/s20/` writes `s20-timing` as an EE program (records ungraded; Sapphire's run is graded in the test) and seven
display programs as S16's scripts: images uploaded through the GIF's channel, the PCRTC's registers set, three V-syncs
waited for, the images read back through VIF1's into the record, then an idle loop; `s20-midframe` goes on switching
every field after its done word. On Sapphire the tests drive the board's GS with the same scripts and compare its picture
with `tests/s20/manual.rs`, chapter 5 transcribed apart from `src/pcrtc/` over S16's model of the memory.

| Program | What it shows |
|---|---|
| `s20-timing` | fields, V-blank, H-blank and H-syncs, the V-sync and FIELD, in four settings of SMODE2 |
| `s20-ct32` | circuit 1 alone in PSMCT32, ×4 across and ×2 down, from (8, 4) in its buffer, BGCOLOR around it |
| `s20-pair` | circuit 1 in PSMCT24 and circuit 2 in PSMCT16 (×5) side by side, OUT2 where circuit 1 is not |
| `s20-alpha` | circuit 1's alpha, doubled, blending it over circuit 2 in PSMCT16S where they overlap |
| `s20-alp` | ALP 60h over BGCOLOR (SLBG), circuit 2 on and not shown |
| `s20-field`, `s20-frame` | interlace in FIELD mode (64 buffer lines) and FRAME mode (32 lines over 64 rasters) |
| `s20-midframe` | DISPFB1 switched forty H-syncs past each V-sync |

**Tests** [M, `cargo test --release`]: `tests/pcrtc.rs` **5 of 5**: `the_display_programs_pictures_equal_the_manuals`
(six pictures, rectangle and every pixel, and their memory read back), `a_register_changed_mid_field_shows_from_the_next_line`
(the switch at raster 45, the line after the fortieth H-sync past the V-sync), `the_even_field_by_csr_reads_the_even_rasters`,
`the_picture_is_described_to_the_frontend_with_its_size_and_shape`, `the_timing_program_sees_the_documents_fields`
(`s20-timing` on the interpreter, 40 s); lib **44 of 44** with `pcrtc::timing`'s three. The crate's other tests on the tree
with S22 and S17 beneath, updated where they named the stand-in's figures: `cop0` 19, `dmac` **18** (one new, §29.9),
`draw` 13, `fpu` 9, `gs` 12, `interpreter` 23, `iop` 13, `micro` 12, `mmi` 13, `programs` 3, `sif` 5, `vu0` 14, `vu_micro` 24, all passing;
lib, `pcrtc`, `gs` and `dmac` also under `test-checked`; clippy clean. WiseMan with `blast.sh`'s filter (`SapphireTests`)
**5 of 5**.

### 29.8 On the referees (measured 2026-10-25)

`examples/s20.rs` writes each program as an ELF with its expected record, and each display program's picture as the
model gives it. `ps2referee.py ram …` read the records from PCSX2 2.8.2 (Americas BIOS, fast boot) and Play! 0.72, one
process at a time; `ps2referee.py ram pcee2 … --for 300 --screen FILE` kept PCEE2's software picture at frame 300 (the
probe's screen dump, which S3 found it hands; the option is new and keeps nothing else).

**The timing** (`s20-timing`): D-200 to D-202 and D-204. Both lineages run a field of exactly 4,915,200 EE cycles in every
setting; PCSX2 raises H-blank and HSINT every 4,681 BUSCLK (525 a field), holds V-blank 23 lines and puts the V-sync 4
lines in; Play! steps its lines by 19,200 cycles, does not gate the timers on H-blank and raises VSINT at VB_ON. Both
change FIELD inside V-blank and never set NFIELD.

**The display programs**:

| Program | Memory read back: PCSX2, Play! (words differing) | PCEE2's picture against Sapphire's |
|---|---|---|
| `s20-ct32` (2,048) | 0, 0 | 48 × 48 both, **every pixel equal** |
| `s20-pair` (3,584) | 0, 0 | framed 138 × 104 against 154 × 56; the same 5,017 colours (D-206) |
| `s20-alpha` (4,096) | 0, 2,048 (Play!'s PSMCT16S reads back as zeros, §24.7) | framed 96 × 112 against 96 × 64; 91 % of its pixels' colours Sapphire's, the rest off by one in a component |
| `s20-alp` (4,096) | 0, 0 | 48 × 64 against 64 × 32; rounding to the nearest produces 1,448 of its 1,535 colours (D-203) |
| `s20-field` (4,096) | 0, 0 | 64 × 64 both, **every pixel equal** |
| `s20-frame` (2,048) | 0, 0 | 64 × 64 both, **every pixel equal** |
| `s20-midframe` (4,096) | 0, 0 | buffer B on every row, each line twice (D-205) |

So by S3's decision the referees' pictures, taken as their memory with the registers the programs wrote, equal
Sapphire's wherever their memory does; and the one lineage whose own pictures can be taken agrees pixel for pixel where it
shows a circuit at its own size, in both interlace modes, which also confirms the FRAME reading of §29.4 and the woven
fields. Its framing of a picture of two circuits, or of a non-interlaced one at MAGV ×1 (each line twice), is its
presentation, not the PCRTC's, and is not graded.

### 29.9 A defect in S11's board, found by S22 and proven here

S22 suspected that a register write waking the DMAC at the board's time, when that time held a blanking edge, made
`Board::run_to` take the edge a second time. **Proven** on the tree before this stage's changes
(`a_register_write_on_a_blanking_edge_does_not_take_the_edge_again`: a timer clocked by H-blank counted 11 for 10 lines)
and fixed in its own commit: the board remembers the time whose edges it last took. The test now runs on the PCRTC's
timing; the timing's start at 0 is not an edge.

### 29.10 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | the interlace field order swapped (`Timing::raster`) | `the_even_field_by_csr_reads_the_even_rasters` |
| F2 | DISPLAY's MAGH read one too large | `the_display_programs_pictures_equal_the_manuals`, `a_register_changed_mid_field_…`, `the_picture_is_described_…` |
| F3 | VSINT a line late (NTSC's V-sync at 5 lines) | `hsint_and_vsint_follow_the_pcrtcs_timing`, `the_timing_program_sees_the_documents_fields` |

Each was applied, the tests run and the source restored [M]. F1 is invisible in a still woven picture, since both fields
read each raster's own line; the field-parity test switches the buffer at an even V-sync to show it.

### 29.11 Questions this stage leaves

1. **Q1 [decision], SMODE1 and the synchronisation registers.** No permitted source gives their fields, so the timing
   follows the board's mode and SMODE2 alone (§29.3, 6), and with the player's BIOS (S24) Sapphire cannot tell 480p or
   PAL from what the kernel writes. *Recommended*: a dispute step at S24 writing chosen values to SMODE1 on PCSX2 and
   measuring the field it gives, outputs only, PCSX2 the single witness; until then a game's mode comes from the disc's
   region.
2. **Q2 [decision], the field in the core ABI.** *Recommended*: a minor giving `frame_info.flags` a bit for "the picture's
   last field was odd" (and one for interlaced), so a CRT filter can draw interlace field by field; offered to every core
   with interlaced modes (Diamond's 480i too), weighed in `EmuSen_CoreAPI_Extensions.md` first. Not built.
3. **Q3 [routine], the merge's rounding (D-203).** *Recommended*: keep PCSX2's lineage's rounding to the nearest.
4. **Q4 [routine], the referees' 60 Hz (D-200).** *Recommended*: keep the documents' 59.94 Hz; a game timing itself by
   V-blank runs 0.1 % slower than on either referee, as on a console by the table.
5. **Q5 [routine], the feedback write.** *Recommended*: build it when a game is found that uses it (S28's bench), with
   the YCbCr rounding then a dispute.

### 29.12 What was read for this stage

`Sapphire_Plan.md` §1.1, §1.4, §1.6, §2, §3, §4.2 to §4.7, §5.1, §5.4, §7, §8, §10 and §11; this page's §20 and §24;
`Sapphire_Disputes.md`'s rules, D-0 and D-42; `Sapphire_Referees.md` §1, §2, §4 and §6; `EmuSen_CoreAPI.md` §6.5, §6.6,
§6.30 and §6.35; Diamond's `gpu/display.rs` and `v1.rs` picture code (the project's own); the crate's `board.rs`,
`blank.rs`, `clock.rs`, `gs/`, `iop/`, `machine.rs`, `v1.rs` and `tests/s6`, `s11`, `s16`. Sony's *GS User's Manual*
(Version 5.0) chapter 5 whole and §7.2's register pages; ps2tek at `1c91660`: "EE Timers", the GS's register lists, "IOP
Timers", the syscall SetGsCrt's and SetVSyncFlag's entries (interfaces, cleared). No emulator, SDK, loader or
ps2autotests source and no firmware; the BIOS was only passed to PCSX2 and PCEE2 by path, and nothing of its output was
kept.

### 29.13 What this stage does not cover

- The machine handing the picture through the ABI and running fields (S10); the CRT profile (S29, §4.6).
- Drawing (S17 to S19): the programs fill the memory by transfers.
- The state format's new contents (the timing, the field, the picture's geometry): `SPH2` gains them with the rest (S30).
- The picture on the device (S46).

## 30. Stage S25: SIO2, the DualShock 2, the memory card and `emusen-ps2-card` (part A), the start without a disc and the Memory Cards tab (part B) (2026-10-25)

*§27 to §29 are stages S17, S14 and S20's, built beside this one and merged before it; SIO2 needs nothing of them.
Part B, after S24 and on the tester's go of 2026-10-25, adds what needs the player's BIOS: the pad driving OSDSYS, a card
formatted by the BIOS read both ways, G10 and G11; it is §30.15 to §30.24, its disputes D-380 to D-383.*

### 30.1 In one paragraph

The plan's §7 row S25, on the tester's go of 2026-10-25 for the next logical stage, in the part that needs no firmware.
Q7's research step came first (§30.2): the card's authentication is documented in its sequence but not its cryptography,
and every source places MagicGate keys in the card and the drive, so Q7(b) holds and the card answers none of F0h, F3h
and F7h. The crate gains **`sio2`** on the IOP's bus (§30.3): SEND3's queue of up to sixteen commands, SEND1 and SEND2,
FIFOIN and FIFOOUT, CTRL, the three RECV words and the interrupt flags at 1F808200h, the IOP's interrupt 17 at a queue's
end, channels 11 and 12 filling and emptying the FIFOs in place of their stand-ins, and four ports, the two controller
connectors and the two card slots (D-220). Behind it, **the DualShock 2** (§30.4) as a state machine on psx-spx's and
ps2tek's protocol, digital, analogue and pressure modes, configuration mode and its commands, the reply bytes 4Fh
enables and both motors; **the memory card** (§30.5) on ps2tek's commands, pages with their spare bytes, the flash's AND
on a write and block erase, its program, erase and load times taken from its flash's data sheet, a command to a busy
card held until it is ready; **the multitap's frame** (§30.6), with no document, as the one witness gives it. The new
shared crate **`emusen-ps2-card`** reads and writes the card's file system from Ross Ridge's public-domain description
(`EmuSen_MediumFormats.md` §6). The core declares the medium **`ps2.card`**, files 0 and 1, and takes the sticks
(§30.7). Five programs of the project's, 1,007 graded words, are equal on Sapphire to the documents' records; PCSX2, the
only lineage with an IOP window, runs four of them and differs where D-220 to D-230 say (§30.9). No emulator, SDK,
loader or ps2autotests source, no card-emulating device's code and no firmware was read.

### 30.2 Q7(a): the card's authentication, researched (2026-10-25)

What was sought, as the tester decided on 2026-10-24: public documents of the exchange SECRMAN runs with F0h, F3h and
F7h, including the published documentation, not the code, of open card-emulating devices; never MagicGate's keys.

| Source | What it says |
|---|---|
| ps2tek at `1c91660`, "SIO2 PS2 Memcards" | F0h "used by SECRMAN while detecting the card and does some XOR checksumming", F3h and F7h "Unknown purpose" |
| ps2tek at `1c91660`, "BIOS IOP SECRMAN" (lines 6408-6412, cleared, `Sapphire_References.md` §3.4) | "authenticating PS2 memory cards"; "The bulk of this work is handled by the CDVD Drive, which contains the actual MagicGate keys"; the exports `SecrAuthCard`, `SecrResetAuthCard` |
| ps2tek, Matías Israelson's later edition, revision `295bc61` (<https://israpps.github.io/ps2tek/PS2/IOP/SIO2/MemoryCard.html>, read 2026-10-25; its page files pinned with their SHA-256s under `~/.cache/emusen/probe/sapphire/docs/ps2tek-israpps/`) | F0h's subcommands: 01h CardIV, 02h CardMaterial, 04h CardNonce requested from the card, 06h, 07h and 0Bh the drive's MechaChallenges sent to it, 0Fh, 11h and (as printed) 03h its CardResponses requested, the rest unknown, 0Ah "usually fails on magicgate mismatch"; F3h "instructs the card to reset cryptography", sent at the start of SecrAuthCard; F7h "orders the card to select a mg keyset": 0 developer, 1 retail, 2 unknown, 3 arcade. Its stated source is ps2sdk's homebrew SECRMAN driver, which was not read |
| SD2PSX (sd2psXtd's firmware), its README (<https://github.com/sd2psXtd/firmware>, read 2026-10-25) | console variants "use different magicgate keysets to ensure their memory cards are inaccessible in other devices", which the device "must actively support"; retail cards "use developer magicgate by default until the console actively requests" retail |
| psdevwiki, "Memory Card" | refused the read (HTTP 403); a search engine's abstract: F7h changes the card's key, an official card holds four MagicGate keys |
| A search for the protocol's cryptography (2026-10-25, two queries) | nothing public |

**Conclusion** [R]: the exchange is a challenge and response under MagicGate keys, the card computing its responses
with keys it holds and the drive controller checking them with its own; its sequence is now public, its cryptography
is not, and no document gives a way for a card to answer without the keys. **Q7(b) holds**: the card answers none of
F0h, F3h and F7h (no byte driven, RECV1 as for a missing device; `the_authentication_commands_get_no_answer`), the
player's BIOS is expected to refuse the card (P8, recorded at part B), and the open BIOS path never sends them. The
drive controller's side of the exchange, its S commands for SECRMAN, is S23's unit and was not sought: answering it
would mean forging the drive's verdict, which is the tester's to weigh (§30.12 Q1). Recorded in the plan's §11 Q7 and
§12.

### 30.3 SIO2 (`sio2/mod.rs`)

[R ps2tek "SIO2 Registers"; A and C where marked]:

- **Registers**: SEND3 at 1F808200h, sixteen words; SEND1 and SEND2 at 1F808240h, one of each for each port, by the
  address's bit 2, kept as written [C: their meaning, "Port1/2 Control?", is unknown]; FIFOIN (1F808260h) takes a
  written byte; FIFOOUT (1F808264h) gives a byte, FFh when empty [A: HiZ; PCSX2 reads the same]; CTRL (1F808268h): bit 0
  starts the queue, bits 2 and 3 empty both FIFOs, neither kept, the other bits kept as written; RECV1 to RECV3; the
  flags at 1F808280h, bit 0 set at a queue's end, cleared by writing it [A: SIO2MAN's handler "reads from this and
  writes to it the value read"; PCSX2 reads 1 after a queue].
- **The queue**: each SEND3 entry until a zero, its port in bits 0-1 and its length, counting the peripheral byte, in
  bits 18-26 (D-230); its bytes taken from FIFOIN (00h for any missing [C]) and sent to its port's device, whose reply
  bytes are held until the queue ends and then placed in FIFOOUT with RECV1 (D-221). A command is answered whole, which
  is the same as byte by byte, since a reply byte depends only on the bytes before it.
- **The ports** (D-220): 0 and 1 the controller connectors, 2 and 3 the card slots; the peripheral byte picks the device
  (01h a controller, 81h a card, 21h a multitap); one nobody answers reads HiZ.
- **Timing** [C]: a byte at 250,000 a second, 147 IOP cycles, on every port: no document gives SIO2's rates or says
  what SEND1 and SEND2 set. *Amended 2026-10-26: Wikipedia's specifications page gives the ports' clocks, 500 kHz for
  the controllers and up to 2 MHz for the cards (D-440, open); a command no device acknowledges ends at its peripheral
  byte (D-383, §30.25).* A queue ends after its bytes, each card command started no sooner than its card is ready
  (§30.5). The IOP's interrupt 17 is raised at the end [R ps2tek "IOP Interrupts"]; no CTRL bit is read as masking it
  [C].
- **DMA**: channel 11 (SIO2in) puts each word's four bytes into FIFOIN, lowest first, always ready; channel 12 (SIO2out)
  takes FIFOOUT's bytes four to a word, ready for as many whole words as FIFOOUT holds, the DMAC woken when a queue ends
  [R ps2tek "SIO2_FIFOIN": "The SIO2in DMA channel also writes to this register"; A for the byte order]. `sio2::Ports`
  wraps the stand-ins as `sif::IopPorts` does, so the IOP's bus steps its DMAC with both. *Amended 2026-10-26 (D-540,
  §30.31): from a queue's end until SIO2's FIFOs are reset or the next queue starts, channel 12 takes every block it
  asks for, FFh past FIFOOUT's bytes, so a block longer than the replies ends.*
- **What changed outside the module**, small and additive: `iop/bus.rs` gains the field `sio2`, its registers in the I/O
  read and write, its end in `run_units` and `reschedule`, and the DMAC stepped through `sio2::Ports`; `lib.rs` declares
  the module. No new dependency: `sio2` uses the crate's own `clock` and `iop::dma` interfaces.

### 30.4 The DualShock 2 (`sio2/pad.rs`)

[R psx-spx "Controllers - Configuration Commands" to "Controllers - Analog Buttons (Dualshock2)", "Standard
Controllers"; ps2tek "SIO2 Controller Commands", "SIO2 Standard Controllers"]:

- **Outside configuration mode**, 42h and 43h: the header FFh, the mode, 5Ah; the buttons (0 pressed); in analogue mode
  the bytes 4Fh enables, the sticks (right X, right Y, left X, left Y, D-227) and the twelve pressures in psx-spx's
  order, padded to whole halfwords, the mode 41h digital and 70h plus the halfwords analogue (73h with the sticks, 79h
  with every byte). 43h answers as 42h without the pressures and enters configuration mode on 01h (D-223); any other
  command is unanswered.
- **In configuration mode**, the mode F3h and six bytes: 40h (twelve attributes, the old value back, FFh past 0Bh),
  41h (D-225), 42h (buttons and sticks whatever the mode), 43h (00h leaves), 44h (the mode and the lock; a mode past 1
  answers FFh), 45h (03h, 02h, the mode, 02h, 01h, 00h), 46h, 47h, 48h, 4Ch, 4Dh (the motors' map, the old map back)
  and 4Fh (the reply bytes, refused if bits 18-39 are set); every other command six zeros (D-224 for those PCSX2
  answers otherwise).
- **The motors**: 42h's bytes 3 to 8 through 4Dh's map, 00h the small motor's bit 0 and 01h the large motor's speed;
  before configuration mode is ever entered, the first analogue pads' one-motor method (40h-7Fh then an odd byte). The
  motors' state is kept in the pad (`small`, `large`): the frontend has no rumble path yet, so it goes nowhere until the
  core ABI's output event the plans ask for exists (`Sapphire_Plan.md` §4.2, `Diamond_Plan.md` §10).
- **The Analog button** (`press_analog`): the mode toggled unless locked, the reply bytes reset, the motors stopped.
- **Input**: the frontend's buttons arrive in the report's bit order (`v1::BUTTONS`), the sticks through `set_axis`
  (§30.7); pressure follows the plan's rule until the ABI carries analogue buttons, FFh for a pressed button and 00h for
  a released one (`Input::from_digital`).
- **Not modelled**, each a named loss: the watchdog that resets a pad after about a second without a read once
  configuration mode was used (psx-spx's Caution); the 5Ah byte becoming 00h after the Analog button following
  configuration commands (its Caution 3); 40h's attributes having any effect.

### 30.5 The memory card (`sio2/card.rs`)

[R ps2tek "SIO2 PS2 Memcards"; Ridge "NAND Flash Memory Basics"; A and C where marked]:

- **Commands**: 11h and 12h (acknowledged), 21h to 23h (the address in pages; 22h opens a page's write buffer, 23h a
  read), 26h (512-byte pages, 16 an erase block, 16,384 pages, the XOR checksum 52h), 27h and 28h (the terminator, 55h
  at reset), 42h (up to 128 bytes into the page at the running offset, the spare bytes as the page's last 16), 43h (up
  to 128 bytes from it with their XOR), 81h (a write's page programmed), 82h (the block holding 21h's address erased to
  FFh), BFh (acknowledged); F0h, F3h and F7h unanswered (§30.2). The reply's place is D-222's: ps2tek's list from the
  command's third byte, HiZ before it; so the programs send two bytes more than ps2tek's command lists.
- **The flash** [R Ridge; M PCSX2's file card]: a write clears bits and never sets one, so a page written twice without
  an erase holds the AND of the two; an erase sets the whole block, sixteen pages and their spare bytes, to FFh (D-229).
  The spare bytes are stored as written: their error correction is the BIOS's, computed by MCMAN, and the crate's for
  the host (`EmuSen_MediumFormats.md` §6.2).
- **Its times** [A]: no document gives the card's own; Ridge names the flash in Sony's cards, Toshiba's TC58V64AFT,
  whose data sheet was not found, and its family's TC58128AFT data sheet (Toshiba, 128-Mbit NAND E2PROM; a copy at
  <https://www.insidegadgets.com/wp-content/uploads/2013/10/TC58128AFT.pdf>, read 2026-10-25) gives a page's programming
  200 to 300 µs typical (1 ms at most), a block's erase 2 ms typical (10 ms at most) and a page's load into its register
  25 µs at most. Sapphire takes 250 µs, 2 ms and 25 µs: a page programmed by 81h, a block erased by 82h, a page loaded
  by 23h each make the card busy for that long after its command, and the next command to the card waits. That is why a
  game's saving screen lasts as long as it does (`Sapphire_Plan.md` §5.8); the SIO2 transfer's own share is §30.3's
  byte time.
- **What the card is**: a battery file of 8,650,752 bytes, every page followed by its spare bytes (`Sapphire_Plan.md`
  §4.4), its `dirty` flag the battery's "changed".

### 30.6 The multitap (`sio2/tap.rs`)

No document gives its commands (D-228). ps2tek gives its peripheral byte, 21h, and psx-spx says it plugs into both a
slot's controller and card ports. The one witness, PCSX2 with a multitap in its first port, answers 12h and 13h with FFh
80h 5Ah 04h 00h 5Ah (four slots) and 21h and 22h with FFh 80h 5Ah 00h 00h 00h 5Ah, on both ports, and nothing else
(measured 2026-10-25). Sapphire answers the same, and its choice [C] for the selection the witness does not show: 21h's
byte after the command's 00h selects which of the tap's four controller slots the controller port then reaches, 22h
which card slot, any other value none; a tap starts with slot A selected for both. A `Slot` holds the tap's four pads
and four cards; without a tap only the first of each is used.

### 30.7 The medium `ps2.card`, the sticks, and what a frontend sees

- **The medium** (`Sapphire_Plan.md` §4.4; `EmuSen_SaveMedia.md` §3.2): core info's `battery[]` for `ps2` gains two
  removable slots of the medium `ps2.card`, files 0 and 1, suffixes `.ps2` and `.2.ps2`, labelled "Slot 1" and "Slot 2";
  machine info lists both; `battery` returns a card's bytes with `BATTERY_TRACKED`, and `BATTERY_CHANGED` once SIO2 has
  written it; `battery_saved` clears that. A card file of any other non-zero length is refused at create with words; an
  empty file (a card not yet saved, as the kit's C14 passes every listed battery file) or none leaves the slot an erased
  card [C], every byte FFh, as PCSX2 makes one; the card manager's *New Card* formats with the crate at part B. *Per
  game* is the runtime's default for every medium (`EmuSen_SaveMedia.md` §3.3), so nothing in the core chooses it. The
  systems table's `media` row (the medium's title and noun) and the card manager's format module come with G11 at part
  B; until then the runtime names the medium `Card`, its rule for a removable medium no pack knows. `ps2.nvm` is S24's.
- **The sticks**: `set_axis` keeps each port's sticks, the descriptor's left X, left Y, right X, right Y in the pad's
  order, -1.0 to 1.0 as 00h to FFh with 0.0 at 80h; the core now claims `AXES` (the core ABI's existing capability, its
  export named in `core_exports!`).
- **WiseMan**: `SapphireTests`' check of the capabilities now expects `AXES`; the kit's C14 first failed on an empty
  card file refused, which showed the rule above.
- **Nothing reaches a player**: the system stays `development: true`, so the runtime offers no shelf, no Memory Cards
  tab and no controller screen for it. The S1 machine does not yet run the board: the cards and sticks wait in `Machine`
  for S24, which joins the two.

### 30.8 The gate: the programs (measured 2026-10-25)

`tests/s25/` generates IOP programs in S21's form (`tests/s21`'s `Program`, run from kseg1), each exchange a SEND3
entry, its bytes written to FIFOIN from a table in the program and its reply read from FIFOOUT after the SIO2 interrupt,
every wait bounded; every expected byte derived from the documents, written apart from `src/sio2/`.

| Program | Graded words | What it covers |
|---|---|---|
| `s25-pad` | 108 | 42h digital; the empty port 2; 43h entering configuration mode; 45h, 41h, 46h, 47h, 48h, 4Ch, 44h (analogue and locked, then a refused mode), 41h, 4Fh enabling every reply byte, 4Dh's map set and read back, 40h's attributes, 42h in configuration mode, 43h leaving; 42h in mode 79h with twelve pressures; 43h from analogue mode; back to digital |
| `s25-pressure` (Sapphire only: a referee given no input holds nothing) | 7 | Right, Cross and R2 held: their bits clear and their pressures FFh in psx-spx's order |
| `s25-card` | 848 | probe; specs; the terminator read, changed to AAh, read and put back; BFh; 12h; a block erased, a page written with its spare bytes and read back, a page written twice without an erase reading the AND, a page of the block still erased, the block erased again by its last page and read; the second slot's card; a controller's command on a card port; F3h, F7h and F0h unanswered; each erase timed by the IOP's timer 3 across the next command, 2 ms and its bytes within 4,000 cycles |
| `s25-dma` | 6 | a queue of a controller read and a card probe sent by channel 11 and its replies taken by channel 12, both in slice mode, their completion |
| `s25-tap` | 38 | a multitap's frame on both ports; slot A selected and answering, slot B selected and empty, slot A again; the card behind it; an unknown command |

Probes, ungraded, for the referee: `s25-probe` (every port's answer to a controller read and a card probe, the status
registers, the pad's configuration commands, the card's pages and spare bytes, eight bytes more read after each reply),
`s25-probe-write` (an erase, pages written and read back on a file card and a folder card), `s25-tap-21-0`,
`s25-tap-21-2`, `s25-tap-61-0` (every command byte under each peripheral byte, with and without a multitap) and
`s25-tap-slots`. `examples/s25.rs` writes the gate's programs and the probes for the referees; `ps2referee.py ram` gains
`--cards A,B`, the cards in PCSX2's two slots for one run, and `--multitap`, both put back afterwards.

**Tests** [M, `cargo test --release`]: `tests/sio2.rs` **5 of 5**: the five programs, each equal to its record word for
word, with the pad's state after `s25-pad`, a card left erased and written (`dirty`) after `s25-card`, the multitap
routing. The lib's `sio2::tests` **5 of 5** (a queue's time and a busy card holding the next command, the erase's 2 ms
in EE cycles and the program and load times; a write's AND and a block erase; the authentication commands; both motors
through 4Dh and the old method; the pressure rule) and `v1`'s new test (the two slots of `ps2.card`, a card file of the
wrong size refused, a card file returned by `battery`, the sticks' order), lib **47 of 47** before the rebase;
`emusen-ps2-card` **5 of 5** (`EmuSen_MediumFormats.md` §6.5). Counts on the rebased tree are in §30.11.

### 30.9 On the referees (measured 2026-10-25)

`examples/s25.rs` wrote each program as an EE program carrying it (S21's route); `ps2referee.py ram pcsx2 … --until
1FFFF0=5A99D00E 200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot, the software renderer),
one process at a time, with two erased card files of the project's in its slots (`--cards`) and for `s25-tap` a
multitap (`--multitap`). Only the records were read. Play! cannot run an IOP program (§22.9).

| Program (graded words) | PCSX2 | What the differences are |
|---|---|---|
| `s25-pad` (108) | 8 | 43h outside configuration mode answers zeros for the buttons (two words, D-223); 46h past index 1, 48h, 44h's refused mode, 40h's stored value and index 0Ch (six, D-224); the identification, the mode bytes, 41h (D-225), the motors' map, the pressures and every other word equal |
| `s25-card` (848) | 342 | the first two bytes of every reply 00h, not HiZ (19 words, D-222); the reads' and writes' first bytes (40, D-222); a short read's checksum and terminator placed after its data (5, D-222); 27h answering the new terminator and 28h's last byte (3, D-229); the erases timed at PCSX2's own speed (2); an erase from page 47 leaving pages 32 and 33 written (273, D-229); the probe, the specs, the data and spare bytes written and read back, the AND of two writes and the erase of a block from its first page equal |
| `s25-dma` (6) | 3 | channel 11 completes; channel 12 is left running and moves nothing |
| `s25-tap` (38) | 4 | the selection of slot B changes nothing (two words, D-228); the reply's first bytes (one, D-222); 41h under the tap's byte is answered (one) |

So **PCSX2 is a witness** for the DualShock 2's identification, modes, 41h, motor map and pressure layout, for RECV1's
two disconnected values, for the card's specs, terminator, data placement, spare bytes and flash rules, and for the
multitap's frame; not for SIO2's ports (it answers by the port's low bit, D-220), the reply's first bytes, the erase's
alignment, SIO2out's DMA, the multitap's selection or any timing. It makes a new card with every byte FFh and formats
none without a BIOS; its folder card reads an empty folder as unformatted and returns a fixed spare area whose code is
the complemented form's for an erased chunk (D-226). **No card formatted by a referee alone exists**: the crate's
cross-check with a referee's formatted card waits for part B, when the player's BIOS formats one on PCSX2
(`EmuSen_MediumFormats.md` §6.4).

### 30.10 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | the pressure bytes' order with Right and Left swapped | `pressure_reads_ffh_for_a_pressed_button_and_00h_for_a_released_one`, `the_pressure_bytes_follow_the_buttons_held_in_the_documented_order` |
| F2 | a page write keeping only the 512 data bytes, its spare bytes skipped | `a_write_clears_bits_and_an_erase_sets_its_whole_block`, `the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes` |
| F3 | the card's erase finishing at once | `a_queue_ends_after_its_bytes_and_a_busy_card_holds_the_next_command`, `the_card_probes_reads_writes_and_erases_pages_with_their_spare_bytes` (the timed command after each erase) |
| F4 | `emusen-ps2-card` writing a page's data without its error correction, the code bytes left erased | `a_formatted_card_has_the_standard_layout_and_8000_clusters_free`, `damaged_cards_are_refused_in_words_never_a_panic` |

F3 was first missed: both tests compared the busy time with the constant they tested, so an erase of no time passed.
They now hold the documents' figure, 589,824 EE cycles and 73,728 IOP cycles, and catch it. Each fault was applied, the
lib's `sio2` tests and `tests/sio2.rs` run (F4: the crate's tests), and the source restored [M].

### 30.11 The blast radius (measured 2026-10-25)

`blast.sh WiseMan sapphire-s25` lists the two crates and WiseMan's `SapphireTests`. After the rebase onto WiseMan
(S14, S17 and S20 beneath), `cargo test --release` in `sapphire`: lib **50 of 50**, `cop0` 19, `dmac` 18, `draw` 13,
`fpu` 9, `gs` 12, `interpreter` 23, `iop` 13, `micro` 12, `mmi` 13, `pcrtc` 5, `programs` 3, `sif` 5, **`sio2` 5**, `vif`
12, `vu0` 14, `vu_micro` 24, every one passing; lib, `iop`, `sif` and `sio2` also under `test-checked`; clippy clean on
every target. `emusen-ps2-card` **5 of 5** under both profiles, clippy clean. `SapphireTests` (WiseMan, the blast
radius's filter) **5 of 5**, after its capabilities check was given `AXES` and the empty card file accepted (§30.7).

### 30.12 Questions this stage leaves

1. **Q1 [decision], the drive's side of the card's authentication.** Q7(b) holds for the card (§30.2). The exchange's
   verdict is the drive controller's, and Sapphire's drive (S23) is its own model: it could report every card genuine
   without any key. That would be answering for the drive rather than the card, a choice no document supports and one
   that makes the player's BIOS accept cards it would refuse. *Recommended*: not now; record P8 at part B as the plan
   says, and offer saving on the open BIOS path, which never authenticates; revisit only if the tester wants the
   player's BIOS to use cards.
2. **Q2 [routine], the ports (D-220).** *Recommended*: keep four ports; S24's BIOS shows on which ports PADMAN and MCMAN
   send, and the routing changes in one place if they differ.
3. **Q3 [routine], the card's reply place (D-222).** *Recommended*: keep it; part B's BIOS-formatted card is the test,
   since MCMAN reads the replies at fixed places.
4. **Q4 [routine], an empty slot's card.** A slot with no card file holds an erased card [C], so the player's BIOS can
   format it. *Recommended*: keep it; the card manager's *New Card* writes a formatted one through the crate at G11.
5. **Q5 [routine], rumble.** *Recommended*: the motors' state stays in the pad until the core ABI gains the output event
   both Sony plans ask for; a minor for it is the framework's, proposed with G11 or later.

### 30.13 What was read for this stage

`Sapphire_Plan.md`: the 2026-10-24 decisions on G10 and G11, §2 (the SIO2 row), §4.2, §4.4, §5.8, §7 (S24, S25), §8
(G10, G11), §9.5, §11 Q7, §12; this page's §22 and §26; `Sapphire_Disputes.md` (the rules, D-140 to D-145);
`Sapphire_Referees.md` §2.1, §4.3, §4.4; `Sapphire_References.md` §3 (the exclusions); `EmuSen_MediumFormats.md`;
`EmuSen_SaveMedia.md` §3, §4, §6.2, §7; the crate's `iop/`, `sif/`, `v1.rs`, `machine.rs`, `tests/s21`, `tests/s22`,
`examples/s21.rs`, `emusen-psx-card` (the project's own code); `ps2referee.py`. ps2tek at `1c91660`: "PS2 Serial Port
(SIO2)" to "SIO2 Command Sequence" (lines 3475-3905), its I/O map's SIO2 rows (lines 250-290), "IOP Interrupts" and "IOP
DMA" (lines 3905-4110), "BIOS IOP SECRMAN" (lines 6408-6440, cleared). Line 3529, the value SIO2MAN writes to CTRL on a
reset, excluded for the open BIOS's writers (`Sapphire_References.md` §3.1), was read while reading the register's
section: this writer writes no open BIOS, and the value is used nowhere. ps2tek's later edition (revision `295bc61`):
"SIO2 Registers", "SIO2 PS2 Memcards", "SIO2 PS2 Memcard Filesystem" (the same tables as the pinned edition but for the
memory card's authentication), saved with two more pages not read and their SHA-256s (`Sapphire_References.md` §1).
psx-spx (Diamond's pinned copy): "Controllers and Memory Cards", from "Controllers - Standard Digital/Analog
Controllers" to "Controllers - Analog Buttons (Dualshock2)", "Controller and Memory Card Multitap Adaptor" (its PS2
note), "MX4SIO", "Memory Card Read/Write Commands" (its timings), "Memory Card Notes". Ross Ridge, "PlayStation 2 Memory
Card File System" (public domain; <https://ps2savetools.com/ps2memcardformat.html>, read 2026-10-25, SHA-256 0735ce5c…,
under `~/.cache/emusen/probe/sapphire/docs/ridge/`). Toshiba's TC58128AFT data sheet (its characteristics). The SD2PSX
firmware's README; searches for the authentication's documents, the TC58V64AFT's data sheet and the PS2 multitap's
protocol (2026-10-25). No emulator, SDK, loader, ps2autotests or card-emulator source and no firmware; the BIOS was only
passed to PCSX2 by path.

### 30.14 What this stage does not cover

- **Part B** (after S24, on the tester's go): the pad driving OSDSYS; a card formatted by the player's BIOS on Sapphire
  read by the crate and by PCSX2, and the reverse, or P8's refusal recorded; G10, the start without a disc; G11, the
  Memory Cards window's PlayStation 2 tab with the format module behind the card manager.
- The machine running SIO2 with the frontend's input and the cards' files (S24 joins the S1 machine and the board);
  SIO2's state in `SPH2` (S30); its debugger view (S31); rumble out to the frontend (an ABI minor); the infrared
  receiver (61h) and other peripherals.

### 30.15 Part B in one paragraph (2026-10-25)

*Part A's sections end at §30.14; part B, on the tester's go of 2026-10-25, continues the stage here.* Three pieces were
built. **G10, the start without a disc** (§30.16): the `ps2` system says `starts_empty` in core info, `create` with a
null image makes the console with its tray closed and empty and its market the player's BIOS's, the core claims `MEDIA`
with one DVD drive whose tray takes a disc judged by the recogniser, and the host refuses a start whose machine would run
nothing, in the firmware entry's own words; Mistress, Hotaru and DianaSh offer it from core info alone, to a developer
only while the core carries the `InDevelopment` mark. **G11, the PlayStation 2's tab of the Memory Cards window**
(§30.17): `emusen-ps2-card` behind the card manager's format surface as the module `ps2.card`, its unit the kilobyte
the console's browser shows. **The card both ways on the referee** (§30.18): PCSX2, driven through its keyboard pad by a
new mode of `ps2referee.py`, ran the player's BIOS to its browser, formatted an erased card there, and showed the saves
the crate's copy and import wrote onto it, while a delete made in its browser was read back by the crate; the BIOS's
card set three of the crate's choices right (D-380) and its free space was the browser's (D-381). **OSDSYS on Sapphire**
(§30.19), started through the empty start with an empty tray, stepped through its initial setup to the main menu at the
pad's presses; its browser, entered with a formatted card in slot 1, never drew and took no press after it, which is
what is recorded for P8, with why the pictures cannot say more. No emulator, SDK, loader or ps2autotests source was
read; the BIOS image was handed by path to Sapphire and to PCSX2 and never opened, and no instruction, program counter,
memory holding BIOS code or data, or the BIOS's traffic to the card was looked at.

### 30.16 G10: the console started with nothing in it

**The core** (`src/v1.rs`, `src/machine.rs`, `src/media.rs`; `Sapphire_Plan.md` §8 G10, `Diamond_Shell.md` §1.1 and
§1.2):

- `starts_empty("ps2")`; `create_empty` makes the machine of `Media::Empty`, the new variant for no image, through the
  same `Machine::make` a game's create now uses, so the files are checked and placed alike. The market is
  `console_market`'s: the setting's, else the market of the BIOS entry the player's file came in, the Americas' first.
- **`MEDIA`** (core ABI 1.1; `EmuSen_CoreAPI.md` §6.23): one drive, number 0, "DVD drive", kind `dvd`, its lid the tray.
  `media_lid` is the player's hand on the tray (`IopBus::tray`, the units brought to the board's time first, then the
  drive's own `tray`, S23's, which ends an N command, stops the disc and, on closing over one, detects it in two
  stages); `media_insert` takes a disc only with the tray open, unless the host is setting the drive for a state
  (`MEDIA_RESTORE`), which goes through the tray as a hand would. A disc is judged by the recogniser first, as an image
  is at create (a sheet by its first track's file, a CHD on opening alone): an EE program is refused as no disc
  (`STATUS_NOT_A_DISC`, −16, S30 having taken −14 and −15), a PlayStation disc, an image of neither kind or a volume with no BOOT2 with create's own
  statuses, a disc image that does not open as `STATUS_BAD_DISC` (−17); then `cdvd::open` makes the drive's medium from
  the kept bytes, as S23's tests do. Nothing else of the drive changed.
- **States** are S30's (§38), merged beneath part B: a state names the image the console was created with, an empty
  start's `Media::Empty` by the word "EMPTY", and holds no disc, so a disc put in afterwards stays in the drive through a
  load; the state does not name that disc, and `WRONG_MEDIA` does not yet refuse a state made over another one
  (§30.21 Q7). Part B's first build kept the drive through S24's replay of a console with no BIOS, which S30 retired.
- **Create with a disc image** is unchanged: the drive stays empty, a disc in the drive at power-on being S29's work.

**The host's refusal** (`EmuSen/Cores/Native/CoreEngine.cs` and the runtime in Rust, `Platform/crystal`'s
`Engine::start_console`, which Mistress runs by default; both alike): the plan asks that with no BIOS the start be refused in
the core's words, never with a black screen. The core cannot refuse the create itself, since the kit's C22 creates
every console that starts empty with the session's files and none holds a BIOS; a refused create fails it. The rule is
therefore the host's and generic: `CoreEngine.StartConsole` refuses, with `ConsoleFirmwareException`, a machine whose
machine info reports its firmware `absent` where the core's entry offers no replacement (`effect: "none"`), in the
entry's label, name and cost, which `ManagedRuntime` turns into `CrystalNumbers.Firmware`, a game's refusal for missing
firmware; the Rust runtime's `runs_nothing` refuses the same machine with the same status and words, the engine with
its machine and its cards' locks dropped with the refusal. The words, measured 2026-10-25: "PlayStation 2 cannot start with nothing in it: PlayStation 2 BIOS (Americas)
(ps2-0230a-20080220.bin) is not in the firmware folder. Without a BIOS of the console's market the PlayStation 2 runs
nothing: it starts to its menu with the player's own BIOS in the firmware folder. Sapphire's own open BIOS, which is to
take its place, is not written yet." The cost text (`v1::BIOS_COST`) was rewritten for a player for this. The rule
touches no other console: Diamond's and Nephrite's entries offer replacements.

**The frontends**, unchanged in code, as `Diamond_Shell.md` §11.2 built them: `CoreCatalog.ConsolesThatStartEmpty`
reads the engines' info, so *File → Start Console ▸ Start PlayStation 2*, the library's *Start PlayStation 2*, the pad
menu's *Start a Console* row, big picture's *Start PlayStation 2 With No Disc*, *Insert Disc…* and *Eject Disc* in the
session, and DianaSh's `core sapphire --console` all appear for the PlayStation 2 where the engine is discovered.
**Behind the `InDevelopment` mark**: discovery leaves a development core out unless development cores are asked for
(`CoreDiscovery.UseDevelopment`), so a player's build offers none of them until S29; `SapphireConsoleTests` holds both
sides.

### 30.17 G11: the PlayStation 2's card in the card manager

- **The module** (`Platform/systems/src/ps2/card.rs`, a family module of `emusen-systems` over `emusen-ps2-card`, a new
  dependency of `emusen-systems` named in its manifest; `EmuSen_SaveMedia.md` §6.2): a save is a directory at the card's
  top level, its id the directory's name; its **title** from the save's `icon.sys`, the Shift-JIS title at byte 192, 68
  bytes, its second line beginning at the offset the half-word at byte 6 gives [R gothi, "icon.sys format",
  ps2savetools.com, published 2013-01-19, read 2026-10-25, its page kept with SHA-256 619e4b62… under
  `~/.cache/emusen/probe/sapphire/docs/iconsys/`], read through the PlayStation module's code page 932 and NFKC
  (`psx::shift_jis`, one owner) with its two lines joined by a space; its **game** from the name by the PlayStation's
  rule (`psx::card::game_of`: `BASLUS-20431…` gives `SLUS-20431`), since the PlayStation 2 names its saves the same
  way; copy (`copy_save`), delete (the directory and everything in it) and format through the crate, each refusal in
  words.
- **The unit** [M, §30.18]: **the kilobyte**, one cluster of 1,024 bytes, which is what the console's browser counts:
  a save of 42 clusters shows "42 KB", and a card's free space is the crate's `free_clusters`, now the browser's figure
  (D-381). The medium's capacity is a formatted card's free space, **7,998 kilobytes**; the module is `ps2.card`, length
  8,650,752, capacity 7,998, unit `kilobyte`, so that the window and `memcard` write "7651 of 7998 kilobytes free" and
  "603 kilobyte(s)".
- **The C# side**: `Ps2CardFormat` forwards to the library's module and has **no C# twin**: the PlayStation 2's file
  system with its error correction is written once, in the crate, as the backend's state 3 intends for every module
  (`EmuSen_Backend.md` §13.6). Under `EMUSEN_SYSTEMS_NATIVE=0` its reads say the module is switched off and its writes
  are refused, so the medium offers no save operations there. `Ps2System` names the medium (`CardLiteral`, title
  "Memory Card (PS2)" as the browser calls it, noun "Card", extension `.ps2`), the systems table's `ps2.json` gives it
  its words, and `SystemPacks.MediumLiterals` lists it, so the parity class holds the table and the literal together.
- **The window and `memcard`** needed nothing of the PlayStation 2's own (`EmuSen_SaveMedia.md` §6.5), and one thing
  of every console offered in development: the window built its tabs from the library's shelves, and a console gets a
  shelf only once its engine says its games run (`EmuSen_CoreAPI.md` §27.4), so no console in development could have a
  tab. It now lists, after the shelves' consoles, those an engine runs that no shelf shows
  (`RuntimeCatalog.UnshelvedConsoles`), and `CoreCatalog.EngineSystem`, with its twin `Catalog::engine_system` in the
  Rust runtime, reads such a pack console's media from the engine discovery offers; with development cores hidden, which is a player's build, neither adds anything, so the tab sits
  behind the same development mark as G10 (§30.21 Q3). `memcard` reads a `.ps2` file as the medium by its extension.
  WiseMan's fit audit gains `MemoryCardsPlayStation2` and `MemoryCardsPlayStation2Held`, each at 1280 × 800 and 1920 ×
  1200, a shared card with two saves (one with a long title) and a game's own card beside it, the held case saying the
  card is in use and its writes refused as the lock refuses them, all recorded in the interface specification and
  walked by the pad as every case is, beside the PlayStation's cases.

### 30.18 The card both ways on the referee (measured 2026-10-25)

`ps2referee.py pads` (`Sapphire_Referees.md` §2.1) boots the BIOS with no disc at the console's speed with a card file
in slot 1 and plays a script of key presses on PCSX2's keyboard pad, its own default bindings, and pictures taken from
states saved through PINE. XTEST's events, sent to the private compositor's Xwayland, never reached PCSX2; events sent
to its window with `XSendEvent`, the window given the input focus first, do. Only the pictures and the card file, the
black box's outputs, were kept.

| Run | What the browser showed | The card file afterwards |
|---|---|---|
| An erased card (every byte FFh) | "Memory Card (PS2)/1 Unformatted"; Cross: "Unformatted. Format now? Yes **No**"; Left, Cross: "Formatting completed.", "7,998 KB Free" | formatted by the BIOS; the crate opens it, no saves, 7,998 free |
| The crate's own formatted blank | "There is no data.", "7,998 KB Free" | unchanged |
| The BIOS's card with one synthetic save copied on by the module (`rt1`) and a second (`rt2`) | two saves, each icon drawn; the first "2026/10/25 7:30:00, 42 KB"; "7,897 KB Free"; the copy menu "No destination available for copying to." | unchanged |
| `rt2` with the first save deleted by the module (`rt3`) | one save, "7,939 KB Free" | unchanged |
| The BIOS's card with both saves imported by the card manager (`rt4`), then *Delete*, *Yes* in the browser | "Delete completed." | the crate reads one save, 7,939 free; the remaining save's three files equal their sources byte for byte |

The synthetic saves are the project's: an `icon.sys` laid out as gothi's page gives it, a one-triangle icon laid out as
Martin Åkesson's *PS2 Icon Format v0.5* (2003, the ps2savetools.com copy, read 2026-10-25) gives it, and a data file;
the time 12:00 of 2026-10-25 written as Japan's shows as 7:30 in the console's time zone (GMT +4:30), which says the
card's times are Japan's as Ridge writes. So **the crate's copy, delete and import were round-tripped on a card the
player's BIOS formatted, and read back by PCSX2 running that BIOS, and a delete made by that BIOS was read back by the
crate**. The titles are not on the screens the script reached; the browser's own copy and its title view were not
driven. What the BIOS's card showed of the format is D-380 and D-381, and its delete D-382.

### 30.19 OSDSYS on Sapphire, the empty tray, and P8 (measured 2026-10-25)

`examples/s25b.rs` starts the console through the core ABI's empty start (`create_empty` with the BIOS file and a card
file), plays presses from a control file written while it runs, and keeps pictures, the card's file and the hardware's
reports (§35.4's rules: no program counter, instruction or memory contents). On the Americas image, with the crate's
formatted card in slot 1, Cross pressed every 120 frames from frame 770:

| Frames | The picture |
|---|---|
| 690-720 | the initial setup's language screen, as at S24: **not black** (20,600 lit pixels) |
| 770-1600 | each press **steps the setup on**: User Preferences: Language, Time Zone (GMT +4:30 Kabul), Daylight Savings Time, and on to "Settings completed. Settings can be adjusted later in System Configuration." |
| 1720 | the main menu, "Browser" and "System Configuration" (without the light trails PCSX2 draws, §35.12 Q4) |
| 1730 | Cross on "Browser" |
| 1850-2700 | the screen black but for the date and clock in its corners; **the clock stops at 19:30:30** and no picture changes again, through Cross presses and a Circle at 2510 |

The EE and the IOP went on running (exceptions counted, no stop, the IOP not held), no unmapped access appeared that the
boot had not shown, the drive's commands were S24's, and **the card was never written**. PCSX2 at the same step shows
the browser's grey ground with the card's icon (with an erased card) or "There is no data." (with slot 1 empty).

So **G10's local check holds**: OSDSYS's picture with an empty tray is not black and changes at a press. **For P8**, what
the player's BIOS shows on Sapphire with a card in slot 1 is a browser that never draws and no longer answers the pad;
it does not show the card refused in words. Whether the card's unanswered authentication (Q7(b), §30.2) is the cause
cannot be told from the pictures: the control, a console with slot 1 empty, did not take the pad's presses at the
language screen at all (2,300 frames, the same nine presses; the run's reports as the other's), which is a second
question, D-383. Going further needs knowing where the BIOS waits, which §1.4 forbids looking for; both are put to the
tester (§30.21).

### 30.20 Tests and seeded faults (2026-10-25)

**Tests** [M]: Sapphire's lib, `cargo test --release`: `v1` **11 of 11**, two new
(`the_console_starts_empty_with_the_tray_closed_and_the_bios_given_naming_the_market`,
`a_disc_goes_in_through_the_tray_and_reaches_the_drive`: the tray refusing a closed insert, a program, a PlayStation
disc and noise refused, a DVD detected as a PlayStation 2 DVD 90 frames after the tray closes, the drive kept through a
state's save and load, Eject, a restore); `emusen-ps2-card` **5 of 5** with its format's measured fields and the browser's free
count; `emusen-systems` **41 of 41**, among them `ps2::card`'s two (a save listed with its two-line Shift-JIS title,
size and game; a copy carrying every byte, the last cluster's with it; the refusals) and the media table's.
WiseMan: `SapphireTests`, `SapphireConsoleTests` (three: offered from core info to a developer only; refused with no
BIOS in the core's words, no card held; a stand-in boot ROM of zeros made by the test, a synthetic DVD put in and
detected, a program refused, Eject), `Ps2CardFormatTests` (four, on cards whose saves the test writes into Ridge's
layout itself: the facts, the listing, copy, move, delete and the lock, the import all or none with its words), the
fit audit's and the specification's new cases, and `CrystalParityTests.A_console_in_development_started_empty_is_refused_without_its_bios_and_lists_its_cards_the_same`
(the two runtimes with development cores shown: the refusal's status and words, the card medium's listing, the start
with the stand-in, its session and save media, alike); counts in §30.22.

**Seeded faults:**

| # | Fault | Caught by |
|---|---|---|
| F1 | the host's refusal removed, so the empty start with no BIOS makes the black console | in the C# runtime `SapphireConsoleTests.With_no_bios_the_start_is_refused_in_the_cores_words`; in the Rust runtime the parity case above (the C# refusal against the Rust session) |
| F2 | `copy_file` stopping a cluster short, so a copied save loses its last cluster | `ps2::card`'s `a_save_copies_whole_deletes_and_refuses_in_words`; the crate's `a_save_copied_to_another_card_reads_the_same_and_a_full_or_clashing_copy_is_refused` |
| F3 | the module's free space counted in pages, the card's 512-byte unit, not kilobytes | `media::tests::a_formatted_blank_reads_as_formatted_with_its_capacity_free` and both `ps2::card` tests; all four `Ps2CardFormatTests` |

Each was applied, the tests named run, and the source restored.

### 30.21 Questions part B leaves

1. **Q1 [decision], P8 and the browser that never draws.** With a card in slot 1 the player's BIOS on Sapphire stalls
   on entering its browser (§30.19); the pictures cannot say whether the card's unanswered authentication is the cause.
   *Recommended*: prove D-383 first (Q2), then run the browser with slot 1 empty; if it draws there and stalls with a
   card, record P8 as "the browser stalls on a card the console cannot authenticate" and keep Q7(b) (cards on the
   player's BIOS path wait; saving is the open BIOS path's); anything past that needs where the BIOS waits, which
   §1.4 leaves to the tester.
2. **Q2 [routine], D-383, the language screen deaf with slot 1 empty.** *Recommended*: a program in S25's form sending
   a pad read with the card port empty, in one queue and in two, its RECV1 and replies read on Sapphire and PCSX2, then
   the rule fixed and the control run again. *Done 2026-10-26 on the tester's go (§30.25 to §30.30): a command no device
   acknowledges ends at its peripheral byte; the browser then stalls with slot 1 empty as with a card (§30.28), which
   answers Q1's condition in the negative.*
3. **Q3 [decision], the Memory Cards tab of a console offered in development.** The window listed shelved consoles
   only, and `EmuSen_CoreAPI.md` §27.4 gives a console its tab and shelf once its games run, so the PlayStation 2's tab
   could not appear before S29. It now also lists the consoles an engine runs that no shelf shows
   (`RuntimeCatalog.UnshelvedConsoles`), with their media read through `CoreCatalog.EngineSystem`'s new fallback, which
   with development cores hidden adds none. *Recommended*: keep it; it is G10's rule for the empty start, a developer
   seeing the console's tools while its games do not yet run, and the Graphics Settings tab and the shelf keep §27.4's.
4. **Q4 [routine], `ps2.card` with no C# twin.** *Recommended*: keep it so; the file system is written once, in the
   crate, and the medium offers nothing under `EMUSEN_SYSTEMS_NATIVE=0`.
5. **Q5 [routine], the host's refusal.** *Recommended*: keep the generic rule (a console started empty whose machine
   runs its firmware absent with no replacement is refused in the entry's words): it is how "never a black screen"
   holds while the kit's C22 runs the console with no BIOS.
6. **Q6 [routine], the card flags.** The BIOS on PCSX2 wrote 2Bh, Ridge's default is 52h (D-380). *Recommended*: keep
   52h until a second lineage or a document decides; the BIOS reads either.
7. **Q7 [routine], a state over a disc put in later.** S30's state names the image of the create, and after an empty
   start that is no disc, so a state made with a disc in the drive loads over an empty drive or another disc. *Recommended*:
   at S29, when a disc goes in at create, name the drive's disc in the state beside the image, so `WRONG_MEDIA` judges
   the drive as Diamond's does.

### 30.22 The blast radius (measured 2026-10-25)

`blast.sh WiseMan sapphire-s25b` lists Sapphire's crate, `emusen-ps2-card`, `emusen-systems` and a WiseMan filter of
21 classes (`BatterySaveTests`, `CheatCodec`, `ConsoleScreenRuleTests`, `ContractShapeTests`, `CoreRegistration`,
`CrystalDebugParity`, `CrystalParityTests`, `DebugCodeParity`, `GamePackageGateTests`, `GenesisBatteryTests`,
`ManagedCrystal`, `PadSettingsWindowTests`, `Ps2CardFormatTests`, `SapphireConsoleTests`, `SapphireTests`,
`SessionServiceParity`, `SystemCodeParity`, `SystemTableParity`, `ThemedLibraryHostTests`, `WindowFitAuditTests`), to
which the classes the change reaches were added: `DiamondShellTests`, `SaveMediaManagerTests`, `SaveMediaTests`,
`SaveMediaParityTests`, `MemoryCardsOfferTests`, `N64PakFormatTests`, `SegaCdBramFormatTests`, `NephriteMediaTests`,
`ConsoleStartTests`, `RuntimeCatalogTests`, `CoreCommandTests`, `PadMenuTests` and `GoldenCorpusTests`.

- Sapphire, `cargo test --release`: **321 of 321** (the lib 68, every test file as §35.10 lists it with its counts
  unchanged); the lib under `test-checked` 68; clippy clean on every target.
- `emusen-ps2-card` 5 of 5 under both profiles, clippy clean; `emusen-systems` 41 of 41, `emusen-platform` 33 and its
  two test files 4 and 3, `emusen-savemedia` 3, clippy clean on what part B wrote.
- WiseMan: the blast filter's classes but `WindowFitAuditTests` and `SapphireTests`, with the added ones, 453 tests, 451
  passing on the first run: `memcard.golden`'s list of known media, which now names `.ps2 (Memory Card (PS2))`, was
  recorded again (its one line and byte count), after which `GoldenCorpusTests` pass 102 of 102; and
  `CrystalParityTests.A_patched_package_s_core_is_handed_the_patched_image_and_the_file_is_never_written_both_ways`,
  a game package's patch, failed once and passed on the next run with nothing of part B between (the package code is
  untouched here). `WindowFitAuditTests` whole, with `RuntimeCatalogTests`, `SapphireConsoleTests` and
  `CrystalParityTests`, 745 tests, every one passing but the golden file then re-recorded; the four new
  specification files were recorded with `EMUSEN_INTERFACE_SPEC=record` and then hold. `SapphireTests` 5 of 5, the
  kit's C1-C15 passing on the program and both discs, with C16 (`MEDIA`, the drive empty for every image) and C22 (the
  console started empty, two machines alike over 120 frames, a state carried to a third, the drive empty and `MEDIA`
  claimed) now run on Sapphire and passing.
- **After the rebase onto WiseMan** (S19 and the game packages' patched saves and stores beneath): Sapphire 334 of 334;
  `emusen-ps2-card` 5, `emusen-systems` 41, `emusen-platform` 33, 4 and 3, `emusen-savemedia` 3; WiseMan, every class
  above with `SapphireTests` and every `MemoryCards` case of the fit audit and the specification, **501 of 501**; with
  the Rust runtime's twin of the refusal and of the media lookup added after the second rebase, `emusen-crystal`'s tests
  pass (13) and the runtime classes again (`CrystalParityTests` with the new case, `SessionServiceParity`,
  `ManagedCrystal`, the card and console classes above and every `MemoryCards` case), **285 of 285**.
- **After the third rebase** (S30 beneath, its states of every unit, which took statuses −14 and −15 and retired S24's
  replay): Sapphire **347 of 347**, the lib 70 under `test-checked`, clippy clean; WiseMan's `SapphireTests` (the kit with
  C16 and C22 on S30's states), `SapphireConsoleTests`, the parity case and `Ps2CardFormatTests`, **13 of 13**. The
  OSDSYS and referee runs of §30.18 and §30.19 were made on the tree before the rebase, S24's with part B.

### 30.23 What was read for part B

`Sapphire_Plan.md` §1.4, §4.4, §5.8, §5.9, §7 (S25, S29), §8 (G10, G11), §9.5, §11 Q7, §12, §13 (P8); this page's §30.1
to §30.14 and §35; `Diamond_Shell.md` §0, §1.1, §1.2, §1.5, §4.3 to §4.5, §11.2; `EmuSen_SaveMedia.md` §6 and §7,
§11; `Hardware/Shared/EmuSen_MediumFormats.md` §6; `EmuSen_Backend.md` §13; `Sapphire_Referees.md` §2.1, §4.5;
`Sapphire_Disputes.md` (the rules, D-220 to D-230, D-340 to D-351); the project's own code named above. Ross Ridge,
"PlayStation 2 Memory Card File System" (the cached copy of §30.13); gothi, "icon.sys format" (ps2savetools.com,
published 2013-01-19, read 2026-10-25, SHA-256 619e4b62f0d14bb12c9187655b183fd78cc7293600ee69dd3d7b08301cf6e408); Martin
Åkesson, *PS2 Icon Format v0.5* (2003, the ps2savetools.com copy, read 2026-10-25, SHA-256
a4cfe862b0bb8b95f6fee02de2693e03910752bf86e1b4010558d93d99802068), both under `~/.cache/emusen/probe/sapphire/docs/`;
ps2tek at `1c91660`, "SIO2_RECV1". PCSX2 2.8.2 run as a black box, its pictures and its card file only. No emulator,
SDK, loader, ps2autotests or card-emulator source and no firmware: the BIOS image was handed by path to Sapphire and to
PCSX2 and never opened by any tool; no program counter, instruction, memory holding BIOS code or data, or traffic
between the BIOS and the card was printed, logged or read; PCSX2's log was read only for the line naming the BIOS.

**Exposure** (§1.4): the writer of part B ran the player's BIOS on Sapphire and on a referee and looked at its pictures,
as S24's writer did; **this writer is exposed and will never write the open replacement**.

### 30.24 What part B does not cover

- G10 and G11 offered to players: S29, with the system (`Sapphire_Plan.md` §7).
- A disc in the drive at create (S29), and a state naming a disc put in after an empty start (§30.21 Q7).
- The card's authentication (Q7(b) holds; §30.21 Q1), the browser's own copy and title views, and the cause of D-383
  (*found 2026-10-26, §30.25*).
- `memcard`'s page names no PlayStation 2 card example and still calls the N64's pak unread: the medium joins it by its
  extension like every other, and only the golden corpus's list of known media changed.

### 30.25 Dispute D-383: the pad with card slot 1 empty (2026-10-26)

*A dispute step on the tester's go of 2026-10-26 (§30.21 Q2, accepted as recommended); its product is
`Sapphire_Disputes.md` D-383 and the fix it proves, with D-440 and D-441 opened on the way.* **The cause**: Sapphire made
a command to a port where no device answers its peripheral byte last the command's whole length, 4 µs a byte; ended at
that byte instead, as a device's absence shows by the /ACK that never follows the address byte (psx-spx "Device
addressing"), the language screen takes the pad with slot 1 empty exactly as with a card. Nothing else of SIO2 changed:
the reply to an empty port is HiZ for the command's whole length and RECV1 is D-221's.

**The programs** (`tests/s25/`): `s25-empty` and `s25-empty-len`, probes for the referee, and `s25-empty-port`, graded,
whose rule `tests/sio2.rs` holds. Each queue is timed on the IOP's timer 3 from just before CTRL's start to after the
interrupt. `examples/s25.rs run <name> <cards>` runs any S25 program on Sapphire's board and prints its record as a
referee's is read, `<cards>` two letters for the slots (`c` a card, `-` none); `ps2referee.py ram --cards` takes `none`
for a slot left empty.

| On PCSX2 2.8.2, slot 1 empty (measured 2026-10-26) | Sapphire | PCSX2 |
|---|---|---|
| a controller's read, alone, or before the empty slot's probe | HiZ 41h 5Ah FFh FFh; 1100h alone, 1D100h with the probe after | equal |
| the empty slot's probe alone; the empty controller port | HiZ throughout; 1D100h, 2D100h | equal |
| the empty slot's probe, then the read, in one queue | the read's reply at byte 4; 1D100h | at byte 6; 1D200h |
| a command of 2 to 20 bytes to the empty slot, then the read | the read's reply after the command's length | at byte 6 whatever the length |
| a queue's time, any queue | its bytes at 4 µs each (part A's rule) | 25 IOP cycles |
| with both cards: a card's probe, then the read | 1100h | 1200h |

So the referee agrees on every single command's reply and status, gives no evidence on time, and its two other effects,
a fixed six bytes from an empty slot and RECV1 counting up after an answered controller, are not taken (D-441).

### 30.26 The cause, found from the pictures and the hardware's counts (measured 2026-10-26)

`examples/s25b.rs` with `S25B_UNITS` set prints, at each picture, SIO2's own counts (CTRL writes, FIFOOUT and RECV1
reads) and the IOP DMA channels 11 and 12, and writes the count of every register access the IOP's units answered,
values left out; `S25B_TAP` plugs a multitap into slot 1. For the step a count of each queue's outcome (every command
answered, none, or some) and of the commands no device acknowledged, by port and peripheral byte, was kept in a build of
its own, not committed: the SIO2 counterpart of the reports' unmapped accesses (§35.4). No program counter, instruction,
memory content or command byte past the peripheral byte was printed. The Americas image, the crate's formatted card in
slot 1 or none, an erased card in slot 2 as the machine makes it, Cross at frame 770:

| Run | A command no device acknowledges takes | Queues to frame 360: none answered / all answered | The language screen after Cross |
|---|---|---|---|
| card in slot 1 | its whole length, 4 µs a byte | 338 / 81 | stepped on by frame 790 |
| slot 1 empty | its whole length | 415 / 22 | unchanged to frame 900 |
| slot 1 empty, a multitap in it | its whole length | 82 / 234 (and 511 of both) | stepped on by 790 |
| slot 1 empty | its peripheral byte | 98 / 310 | stepped on by 790 |
| slot 1 empty | its peripheral byte and 100 µs (the PlayStation kernel's wait, psx-spx) | 415 / 22 | unchanged |
| slot 1 empty, every byte at 1 µs | its whole length, at that rate | 98 / 310 | stepped on by 790 |

With slot 1 empty, from frame 300 on, nearly every queue was a single command no device acknowledged, the multitap's
peripheral byte 21h on card port 2, about one a frame (frames 360-420: 65 of them, 3 queues answered), where with the
card such commands stopped at frame 300 and the answered queues ran at about one a frame; so the controller was not
being read, while the IOP and its SIO2 traffic went on. Every reply in those runs was the same in the two cases, HiZ and
1D100h; only the time of the unanswered command decided whether the controller came to be read. With the rule the
language screen's text fades from frame 780 and the transition's lights are drawn at frame 900 as in the card run
(17,472 lit pixels at 780 in both); the three runs that took the press have equal counts at every picture. Why the BIOS
needs the answer sooner than the whole command is where it waits, which is not looked for (`Sapphire_Plan.md` §1.4); the
rule rests on the documents and the referee's replies, the measurement only showing that it is the one at fault.

**On the way**, a hardware state the counts show in every run, card or none: IOP DMA channel 12 (SIO2out) started in
slice mode for a block of 24h words about frame 50 and never ended (CHCR 41000200h, BCR 00010024h), since no queue
after it leaves 144 bytes in FIFOOUT at once; the controller is read through FIFOOUT, and the BIOS reaches its menu, so
it is recorded and not followed here (§30.29 Q3). *Followed 2026-10-26 (D-540, §30.31): SIO2out now ends at a
queue's end, and the browser's stall is unchanged.*

### 30.27 The rule built, tests and seeded faults (2026-10-26)

`sio2::Sio2::start` (`src/sio2/mod.rs`): a command is *addressed* when a device on its port takes its peripheral byte, a
pad for 01h on a controller port, a card for 81h on a card port, a multitap for 21h on either port of its slot (behind a
multitap, the pad or card of the slot it selects); an addressed command takes its length in bytes, as before, and one
that is not takes one byte. Its reply and RECV1 are unchanged, and a card that refuses a command it was addressed by (the
authentication's F0h, F3h and F7h) keeps the whole length. No new dependency.

**Tests** [M, `cargo test --release`]: the crate **349 of 349** (part B's 347 and two new), **360 of 360** after the
rebase onto WiseMan with S10 beneath and **364 of 364** with §40's IOP programs beneath too; the lib's
`sio2::tests::a_command_nothing_acknowledges_ends_at_its_peripheral_byte` (an empty slot's probe and a missing
multitap one byte each, a read after the empty slot's probe six bytes in all with its reply at byte 4, a card's refused
command its whole length) and `tests/sio2.rs`'s test of the same name running `s25-empty-port` (29 graded words: each
queue's time within 400 cycles above its bytes' worth, RECV1, the replies), both also under `test-checked`; clippy clean
on every target. WiseMan's blast filter, `SapphireTests`: §30.30.

| # | Fault | Caught by |
|---|---|---|
| F1 | the empty port's command taking its whole length again (part A's rule) | both tests named above |
| F2 | the empty port's command taking no time at all | both tests |
| F3 | a card that refuses a command treated as absent, its command cut to one byte | the lib's test (the program has no card) |

Each was applied, the tests run, and the source restored.

### 30.28 OSDSYS's browser again, and P8 (measured 2026-10-26)

`examples/s25b.rs` on the tree with the rule, the Americas image, part B's script (Cross every 120 frames from frame 770,
the ninth on "Browser" at 1730, Circle at 2510), two runs: slot 1 empty, and slot 1 holding the crate's formatted card
(`emusen_ps2_card::format`, no saves); slot 2 the erased card the machine makes, in both.

| Frames | Slot 1 empty | The formatted card in slot 1 |
|---|---|---|
| 770-1610 | each press steps the setup on: Language, Time Zone, Daylight Savings Time, "Settings completed." | the same |
| 1680 | the main menu, "Browser" and "System Configuration", now with its ring of lights | the same |
| 1790-1860 | the menu fading out on Cross | the same |
| 1920-2700 | **black, every pixel zero**, through the Circle at 2510 | the same |

The 54 pictures of the two runs are equal byte for byte; the card was never written (its file equal to the crate's);
SIO2's counts after frame 1680 are equal in the two runs, about 1.1 queues a frame and nothing more, so entering the
browser sent the card in slot 1 nothing at all; the reports name no unmapped access part B's runs did not, and the EE's
exception counts are equal in the two runs. PCSX2 at the same step draws "There is no data." with slot 1 empty (§30.19).
Both runs were made again on the tree rebased onto WiseMan, S10's IOP DMAC fixes beneath, and again with §40's drive
interrupt (D-481) and IOP region (D-482) beneath too: each time the setup took the pad with slot 1 empty, and the browser
stalled alike, the 54 pictures of the two runs equal, every pixel zero at frame 2700, the card unwritten, channel 12's
slice still unended, the reports' list unchanged.

So **for P8**: with D-383 fixed, the browser now stalls on Sapphire with slot 1 empty exactly as with a card, the
console never reaching the card in slot 1; the stall is not the card's unanswered authentication (Q7(b), §30.2), and P8
cannot be recorded as "the BIOS refuses the card": what the player's BIOS shows on Sapphire is a browser that never
draws, card or none. The clock part B saw in the corners of the menu and the browser is not drawn on this tree, in
either run. Going further needs where the BIOS waits, which is not looked for (`Sapphire_Plan.md` §1.4): it is put to
the tester (§30.29 Q1). The one unit the counts show waiting is channel 12 (§30.26), in every run since frame 50,
including those whose setup took the pad.

### 30.29 Questions the D-383 step leaves

1. **Q1 [decision], the browser that never draws, with or without a card.** *Recommended*: record P8 as observed ("the
   browser stalls on entry on Sapphire whatever is in slot 1; the card's refusal is not reached"), keep Q7(b), and
   choose between (a) a dispute step on SIO2out (Q3), the one unit the hardware's counts show waiting, which needs no
   look at the BIOS, and (b) leaving the browser until S10's fast-booted programs exercise the card's driver path
   through the project's own programs. (a) first: it is cheap and may be the cause. *Decided 2026-10-26 by the tester:
   P8 recorded as observed, Q7(b) kept, (a) taken (§30.31 to §30.37); the slice was not the cause.*
2. **Q2 [routine], the ports' clocks (D-440).** *Recommended*: a step of its own that sets the controllers' bytes at 500
   kHz and the cards' at 2 MHz, if SEND1 and SEND2 can be read as the rates by a program of the project's own, and runs
   the boot, the setup and the browser again; until then 4 µs a byte everywhere.
3. **Q3 [routine], SIO2out's slice that never ends.** Channel 12 starts a 36-word slice about frame 50 of every boot and
   no queue after it leaves 144 bytes in FIFOOUT. *Recommended*: an IOP program in S25's form that starts SIO2in and
   SIO2out for a queue shorter than their blocks, and longer, read on Sapphire and PCSX2 (whose channel 12 moved nothing
   at part A, §30.9), to rule what a slice gets when a queue's replies are fewer than its block. *Done 2026-10-26
   (D-540, §30.31): the slice takes its whole transfer at the queue's end, FFh past the replies.*
4. **Q4 [routine], D-441.** *Recommended*: keep D-221's RECV1 and the command's length; revisit if a second lineage or a
   document appears.

### 30.30 What was read, the blast radius, and exposure (2026-10-26)

**Read**: `Sapphire_Plan.md` §1.4, §5.8, §5.9, §11 Q7, §13 (P8); this page's §30 (parts A and B) and §35;
`Sapphire_Disputes.md` (its rules, D-220 to D-230, D-380 to D-383); `Sapphire_Referees.md` §2.1, §4.3, §4.5. ps2tek at
`1c91660`: "SIO2 Registers" (with "SIO2_RECV1" and the transfer's flow), "SIO2 Command Sequence", "IOP DMA", and in the
BIOS chapter "BIOS IOP SIO2MAN" (its export list only), as a writer of a player's-BIOS stage may; the later edition
(Israelson) "SIO2 Registers" and "SIO2 Controller Commands"; psx-spx at `6d7d1bc` (2026-10-08), "Controllers and Memory
Cards" (overview, device addressing, /ACK, signals, PS2 multitap, MX4SIO); Wikipedia, "PlayStation 2 technical
specifications" (the copy of S2). PCSX2 2.8.2 run as a black box, its RAM through PINE only. No emulator, SDK, loader,
ps2autotests or card-emulator source and no firmware: the BIOS image was handed by path to Sapphire and to PCSX2 and
never opened; no program counter, instruction, memory holding BIOS code or data, or command byte past a peripheral byte
was printed, logged or read; of SIO2's traffic only counts were kept, as §30.26 says.

**Blast radius** (`blast.sh WiseMan sapphire-d383`: Sapphire's crate, WiseMan's `SapphireTests`): Sapphire **349 of
349**; the lib and `tests/sio2.rs` under `test-checked`, 71 and 6; clippy clean on every target; WiseMan `SapphireTests`
**5 of 5** (the kit on Sapphire), and `SapphireConsoleTests` **3 of 3**, added since the change is Sapphire's core.
**After the rebase onto WiseMan** (S10 and the game packages' step 7 beneath): Sapphire **360 of 360**, the lib and
`tests/sio2.rs` under `test-checked` 76 and 6, clippy clean; `SapphireTests` and `SapphireConsoleTests` **8 of 8**.
**After the second rebase** (§40's IOP programs beneath, with the drive's interrupt of D-481 and the IOP's region of
D-482): Sapphire **364 of 364**, under `test-checked` 78 and 6, clippy clean; the two WiseMan classes **8 of 8**.

**Exposure** (§1.4, recorded in §35.13 as well): the writer of this step ran the player's BIOS on Sapphire and looked at
its pictures and the hardware's counts, and read the BIOS chapter's SIO2MAN entry; **this writer is exposed and will
never write the open replacement**.

### 30.31 Dispute D-540: SIO2out's slice that never ended (2026-10-26)

*A dispute step on the tester's go of 2026-10-26 (§30.29 Q1 option (a) and Q3, P8 recorded as observed, Q7(b) kept); its
product is `Sapphire_Disputes.md` D-540 and the fix it proves, with D-541 opened on the way.* **The cause**: Sapphire's
SIO2 asked channel 12 for a block only when FIFOOUT held all of it (part A's rule, §30.3), and the player's BIOS starts
SIO2out with one block of 36 words for a queue of one command of four or five bytes, so the slice of §30.26 waited for
144 bytes that never came and the replies stayed in FIFOOUT until the next reset emptied it. **The rule**: a queue's end
requests SIO2out until CTRL's bits 2 and 3 reset the FIFOs or bit 0 starts the next queue; meanwhile channel 12 moves
every block it asks for, FIFOOUT's bytes first and FFh past them, and ends with its flag as any slice does.

### 30.32 What the hardware showed while the BIOS ran (measured 2026-10-26)

A build of its own, not committed, printed at each write to channel 11's or 12's registers and at each queue's start
and end SIO2's SEND3 entries (port and lengths), CTRL, how many bytes FIFOIN, FIFOOUT and the pending replies held, and
channels 11's and 12's BCR and CHCR, with DICR2: the hardware's own register state, as §1.4 lets a writer of a
player's-BIOS stage observe; no program counter, instruction, memory content or command byte. On the Americas image
with the crate's formatted card in slot 1, to frame 60:

- the queues before about frame 50 are written by the processor: one command each, its SEND3 entry with bit 6 set (40h
  beside the port), its replies read from FIFOOUT;
- at about frame 50 four queues, two to each card port, go by DMA: channel 11 started with BCR 00010024h and CHCR
  00000201h then 01000201h, moving 144 bytes into FIFOIN; channel 12 with BCR 00010024h and CHCR 40000200h then
  41000200h; then CTRL's bit 0 for a queue of one command of four or five bytes, its SEND3 entry with bits 4 to 6 set
  (70h); DICR2 000D0400h, channels 11's and 12's masks clear, so the BIOS waits for SIO2's interrupt and not for the
  channels';
- each such queue took four or five of FIFOIN's 144 bytes and left as many in FIFOOUT; channel 12, asking for 144, never
  moved; the BIOS wrote its registers again for the next transfer, and after the last the slice stayed as §30.26 saw it.

Before them the BIOS writes zero to both channels' CHCR four times. With slot 1 empty it makes more such transfers (20
writes of channel 12's CHCR to frame 60, against 12), and none after frame 60 in either run.

### 30.33 The programs, on Sapphire and the referee (measured 2026-10-26)

`tests/s25/`: **`s25-dma-out-a` to `-r`**, probes for the referee, each a controller's read by FIFOIN and FIFOOUT, a
transfer by DMA, the read, the transfer again and the read, every queue's SIO2 flags written back as read, as a driver
acknowledges them; each transfer records the wait's passes, both channels' MADR, BCR and CHCR, DICR2 and I_STAT at
SIO2's interrupt and after a delay, RECV1, eight bytes more from FIFOOUT, and channel 12's buffer with the word past it.
The eighteen cases are D-540's list. **`s25-sio2out`**, graded, 179 words, with a card in slot 1: the BIOS's shape, a
card's probe of four bytes by SIO2in in a block of 36 words and its replies by SIO2out in one block of 36; the same in
36 blocks of one word; SIO2out started after the queue's end; started after a reset, still waiting 500 passes later and
moving the next queue's replies; a block of one word for a queue of nine replies, the other five left in FIFOOUT.
`examples/s25.rs` writes them for the referee and runs any of them on Sapphire's board.

| Case (PCSX2 2.8.2, both slots filled and slot 1 empty) | Sapphire before | PCSX2 | Sapphire with the rule |
|---|---|---|---|
| the BIOS's shape, every SEND3 flag tried (70h, 40h, 00h, 50h, 60h) | channel 12 never ends, its buffer untouched, the replies left in FIFOOUT | ends by SIO2's interrupt: CHCR 40000200h, MADR + 90h, DICR2's flag, IRQ 3; the replies at the buffer's start | as PCSX2 |
| blocks of 1, 2, 36 × 1 and 2 × 18 words | ends only when the replies fill whole blocks | ends after its whole transfer | as PCSX2 |
| started after the queue's end | never ends | runs at once | as PCSX2 |
| a block shorter than the replies | ends, the rest in FIFOOUT | the same | the same |
| the bytes past the replies | — | 00h where SIO2in gave FIFOIN more than the queue sent, A5h padding alike; FFh where it did not | FFh, an empty FIFOOUT's read |
| started after a reset, no queue since | waits | runs at once with the previous queue's reply: its reset leaves FIFOOUT | waits for the next queue |
| BCR at the end | its block count at zero | as written | its block count at zero (psx-spx's slice mode) |
| SIO2's interrupt with the flags not written back | every queue | the first queue only (D-541) | every queue |

On `s25-sio2out` PCSX2 differs in 150 of the 179 words: the card's first two reply bytes (4, D-222), 00h past the
replies (140), the reset's wait (2) and FIFOOUT after it (1), and its controller's bytes past five (3), since it clocks
the whole of FIFOIN rather than the SEND3 lengths (D-441's arithmetic); every end, MADR, flag, interrupt, reply's
place and untouched word past the transfer is equal. With slot 1 empty only the replies changed (HiZ, and PCSX2's six
bytes of D-441), on both, never the channel's end.

### 30.34 The rule built, tests and seeded faults (2026-10-26)

`src/sio2/mod.rs`: `Sio2`'s `wake`, which a queue's end set and the bus took once to wake the DMAC, is now SIO2out's
request, set at a queue's end and cleared by CTRL's reset bits and by a queue's start; `Ports::request` gives channel 12
every word it asks for while it is set and none otherwise, and `get` already read FFh from an empty FIFOOUT.
`src/iop/bus.rs` wakes the DMAC where a queue ends. The state's layout is unchanged: the flag keeps its place and name
(`Sio2.Wake`), so version 3 stands; a state saved by an earlier build, in which the flag was always clear between steps,
loads with SIO2out unrequested until the next queue ends, which no player can meet, the console being offered to none.
No new dependency.

**Tests** [M, `cargo test --release`]: the crate **366 of 366** (364 and two new): the lib's
`sio2::tests::sio2out_is_requested_from_a_queues_end_until_a_reset_or_the_next_queue` (nothing while the queue runs, 36
words for four reply bytes, FFh past them, still requested with FIFOOUT empty, cleared by a reset and by the next queue's
start) and `tests/sio2.rs`'s `sio2out_moves_its_whole_transfer_once_a_queue_has_ended` (`s25-sio2out`); `s25-dma`'s test
of part A passes unchanged; the lib and `tests/sio2.rs` under `test-checked`, 79 and 7; clippy clean on every target.

| # | Fault | Caught by |
|---|---|---|
| F1 | part A's rule again: channel 12 asked only for the whole words FIFOOUT holds | both tests named above |
| F2 | the request surviving CTRL's reset | both tests |
| F3 | 00h past FIFOOUT's bytes, PCSX2's fill | both tests |

Each was applied, the tests run, and the source restored.

### 30.35 OSDSYS's browser again, and P8 (measured 2026-10-26)

`examples/s25b.rs` on the tree with the rule, with `S25B_UNITS`, the Americas image and part B's script as §30.28 ran it
(Cross every 120 frames from frame 770, the ninth on "Browser" at 1730, Circle at 2510, to frame 2700), two runs: slot 1
empty, and slot 1 holding the crate's formatted card; slot 2 the erased card the machine makes, in both.

| Frames | Slot 1 empty | The formatted card in slot 1 |
|---|---|---|
| 770-1610 | each press steps the setup on: Language, Time Zone, Daylight Savings Time, "Settings completed." | the same |
| 1680 | the main menu, "Browser" and "System Configuration", with its ring of lights | the same |
| 1740-1860 | the menu fading out on Cross | the same |
| 1920-2700 | **black, every pixel zero**, through the Circle at 2510 | the same |

The 54 pictures of the two runs are equal byte for byte, and each picture's count of lit pixels is the one §30.28's runs
recorded at the same frame; the card was never written (its file equal to the crate's); channel 12 now ends after each
of the BIOS's transfers (CHCR 40000200h from frame 60 on, MADR past its block) and none starts after them; SIO2's counts
grow by about 1.1 queues a frame after frame 1680 in both runs; the reports name the unmapped accesses §30.28's did, and
the EE's and the IOP's exception counts are §30.28's runs' to the unit (the EE's system calls 1,157,984 with the card and
1,157,988 without). PCSX2 at the same step draws "There is no data." with slot 1 empty (§30.19). Both runs were made
again on the tree rebased onto WiseMan (S31's debugger beneath): the 54 pictures of each equal byte for byte to the runs
before the rebase, the card unwritten, the counts and the reports the same.

So **for P8**, recorded as observed as the tester decided: the player's BIOS on Sapphire, card or none, stalls on
entering its browser with every pixel black and never sends the card in slot 1 a command; the card's refusal (Q7(b),
§30.2) is not reached. D-540 put the BIOS's SIO2 replies in memory where the BIOS asks for them, and nothing the pictures
or the counts show changed: the slice was a defect of SIO2out, not the browser's cause. No unit of SIO2 or of the IOP's
DMA is now seen waiting; going further needs another look, put to the tester (§30.36 Q1).

### 30.36 Questions the SIO2out step leaves

1. **Q1 [decision], the browser that never draws.** With D-383 and D-540 fixed, no unit the hardware's reports cover is
   seen waiting while the browser stays black. *Recommended*: (a) a step of its own that traces the SIF link while the
   browser is entered, narrowly as §1.4 allows a writer of a player's-BIOS stage since 2026-10-26 (the SIF DMA tags and
   the SIF RPC packets in ps2tek's documented layouts, never committed, recorded as exposure), to see which server call
   the EE makes on entering the browser and whether the IOP answers it; it names a module's interface, not where the
   BIOS waits. (b) leave the browser until S10's fast-booted programs drive the card's driver through the project's own
   programs. (a) first: it is cheap and stays inside the decided rules.
2. **Q2 [routine], D-541, SIO2's interrupt.** *Recommended*: keep IRQ 17 at every queue's end until a second lineage or a
   document decides; a driver that acknowledges the flag, as ps2tek describes SIO2MAN's handler, sees the same.
3. **Q3 [routine], SEND3's bits 4 to 6.** The BIOS sets 70h beside the port on the queues it sends by DMA and 40h on the
   others; PCSX2 is the same with any of them. *Recommended*: keep them stored as written and given no effect.
4. **Q4 [routine], D-440, the ports' clocks.** This step measured no rate; *recommended*: leave it open, as §30.29 Q2
   proposes.

### 30.37 What was read, the blast radius, and exposure (2026-10-26)

**Read**: `Sapphire_Plan.md` §1.4, §5.8, §11 Q7, §12 (P8 is in §13); this page's §30 (parts A and B, the D-383 step) and
§35; `Sapphire_Disputes.md` (its rules, D-220 to D-230, D-383, D-440, D-441); `Sapphire_Referees.md` §2.1, §4.3, §4.5;
`Sapphire_References.md` §3 (the exclusions). ps2tek at `1c91660`: "PS2 Serial Port (SIO2)" and "SIO2 Registers" with
the transfer's flow, "SIO2 PS2 Memcards" (its command list, passed through on the way), "IOP Interrupts", "IOP DMA";
the later edition (Israelson) "SIO2 Registers" and "SIO2 Command Sequence"; psx-spx at `6d7d1bc`, "DMA Channels" (its
slice mode's BCR). PCSX2 2.8.2 run as a black box, its RAM through PINE only. No emulator, SDK, loader, ps2autotests or
card-emulator source and no firmware: the BIOS image was handed by path to Sapphire and never opened; no program counter,
instruction, memory holding BIOS code or data, or command byte was printed, logged or read; of SIO2 and the DMAC only
their registers, as §30.32 says; the SIF link was not traced.

**Blast radius** (`blast.sh WiseMan sapphire-sio2dma`: Sapphire's crate, WiseMan's `SapphireTests`): Sapphire **366 of
366**, the lib and `tests/sio2.rs` under `test-checked` 79 and 7, clippy clean; WiseMan `SapphireTests` and
`SapphireConsoleTests`, the second added since the change is Sapphire's core, **8 of 8**. **After the rebase onto
WiseMan** (S31's debugger, `emusen-jit` and D-483's DVDs beneath): Sapphire **383 of 383**, the lib and `tests/sio2.rs`
under `test-checked` 91 and 7, clippy clean; the two WiseMan classes **8 of 8**.

**Exposure** (§1.4, recorded in §35.13 as well): the writer of this step ran the player's BIOS on Sapphire and looked at
its pictures and the hardware's register state, SIO2's and the DMA channels'; **this writer is exposed and will never
write the open replacement**.

### 30.38 The SIF link traced while the browser is entered (2026-10-26)

*A dispute step on the tester's go of 2026-10-26 (§30.36 Q1, option (a)), under `Sapphire_Plan.md` §1.4's allowance of
the same day; its product is `Sapphire_Disputes.md` D-620.* The SIF DMA tags and the documented SIF RPC packets were
traced on Sapphire from power-on to field 2200, Cross on "Browser" at field 1730, with the crate's formatted card in
slot 1 and with slot 1 empty, and PCSX2's EE RAM was read through PINE at the EE's receive address while its BIOS was
stepped to the same place. **No documented call goes unanswered while the browser is entered**: the EE's two calls on
entering it are answered, as every call of the boot is but one no-wait call that PCSX2 leaves unanswered too. **What
differs is what the EE asks**: on PCSX2 the browser calls the card server (80000400h) and the multitap server's "Get
Connection" (80000903h) about fifteen times a second each from two seconds after Cross, and an EE thread calls the pad
server (80000100h) every 64 fields from the boot on; on Sapphire neither of the first two is ever called in the browser
and the pad server's calls stop after the third, at field 75, while the menu goes on answering the pad. The SIF, the
IOP's DMAC and the IOP's modules answer what they are asked; the stall is on the EE's side of the link or in a reply's
contents, which §1.4 does not let the step read, and the next look is put to the tester (§30.42 Q1). No code was
changed: no defect was found for a program to prove.

### 30.39 What crossed the link on Sapphire (measured 2026-10-26)

A build of its own, not committed, noted at the IOP's channels 9 and 10 each chain tag and, for each packet whose
destination was a side's receive address (SMCOM's for SIF1; for SIF0 the EE's, as the EE's SIFCMD Init and Change
SADDR packets set it, 00411940h after the IOP's reboot at field 13), the `SifCmdHeader` and the fields ps2tek gives
BIND, CALL and END (client, server id, function number, sizes and mode), in its layouts and no further; a packet sent
elsewhere, a call's data or a reply's, was kept as its tag alone. The runner printed them by field beside the pictures,
the EE's and the IOP's units' registers and the hardware's counts (§35.4). The first run's first build took a packet
for a command by its header's shape alone, and so printed the first four words of some of the pad module's buffers sent
into EE memory and of one transfer of sound data; it was corrected to the receive address before the other runs, and
those lines went with the trace, which was kept in scratch and deleted once this section was written.

| Server (ps2tek's name) | Calls, card in slot 1 | When | After Cross at 1730 |
|---|---|---|---|
| 80000006h (LOADFILE) | 5 | fields 9 to 51 | none |
| 80000001h (FILEIO) | 48 | fields 30 to 53 | none |
| 80000592h, 80000595h (CDVD init, N commands) | 1 each | fields 30, 33 | none |
| 80000593h (CDVD S commands) | 49 | from field 30, once a second from field 55 | 8, once a second |
| 80000596h (not in ps2tek's list) | 1, no-wait, no reply buffer | field 30 | **never answered** (nor on PCSX2, §30.40) |
| 80000400h (memory cards) | 3 | fields 30, 52, 53 | none |
| 80000100h (pad) | 4 | fields 32 (twice), 55, 75 | none |
| 80000901h (multitap "Port Open") | 4 | fields 30 to 32 | none |
| 80000902h, 80000903h (multitap) | bound, never called | field 30 | none |
| 80000003h, 80000C01h | 2, 3 | fields 50 to 53, 32 to 55 | none |
| 80000601h (not in ps2tek's list) | 116 | from field 50, function 5200h at every press | 2: 5200h at 1733, 5015h at 1735 |

**With slot 1 empty the trace is the same call for call**, the card server's three calls among them, and so are the
pictures and the EE's system-call count. Once the screen is black (field 1920 on) the link carries the S-command
server's call once a second and the pad's buffers that the IOP's module sends into EE memory every field, as before
Cross. The EE's units at the stall, read as registers at each picture: every EE DMA channel stopped (VIF1's CHCR
70000045h, GIF's 00000001h, D_STAT 00A00046h), GIF_STAT and both VIF_STATs zero; the GS's PMODE, DISPFB2 and DISPLAY2 as
in the menu (circuit 2, the frame at FBP 0, 640 wide); I_STAT zero, I_MASK 0000100Eh (SBUS, VBON, VBOF, TIM3); no EE
timer register written after field 13, T3 left at mode C83h with its interrupts off, so no kernel alarm was ever armed;
of the INTC's sources only VBON and VBOF raised; the EE's system calls about 100 a field in the menu, 58 during the fade
and 20 once black, its interrupts two a field. The EE reads the drive's disc type at 1F40200Fh about once a field from
field 54 on, a register Sapphire's EE map does not answer and which reads 0, "No disc", which an empty tray reports
anyway (§30.42 Q3).

### 30.40 What PCSX2's receive buffer showed (measured 2026-10-26)

PCSX2 2.8.2 was run as §30.18's `pads` mode runs it, with a copy of `ps2referee.py` in scratch, not committed, that
also read, about once a millisecond, the four header words at the EE's receive address 00411940h (the address the
packets name on both consoles) and, where the command was END, its eight documented fields, and kept each change with its
time from when PCSX2 began to draw. The buffer is overwritten by each packet, so packets that follow one another within
a poll are missed, and the counts below are what was caught, not every call. Seven runs with the reads and one taking a
picture at each step, slot 1 empty, PCSX2's own EEPROM showing the initial setup, Cross pressed every 3.5 seconds from 18
seconds in the later ones, the sixth press entering the browser:

| PCSX2 | What the buffer held |
|---|---|
| boot, 0.4 to 1.4 s | the same servers bound and called as on Sapphire to field 55: FILEIO, the CDVD servers, the card server, the multitap's "Port Open", 80000C01h, the sound server; no reply to 80000596h's call, in 30 s |
| the pad server | answered at 0.43, 1.01 and 1.35 s (Sapphire's fields 32, 55 and 75), **then every 1.07 s, 64 fields, without end**, through the setup, the menu and the browser |
| the S-command server | once a second, as on Sapphire |
| each press | the sound server's call, as on Sapphire |
| the setup and the menu | nothing else |
| from 2 s after Cross on "Browser" | **the card server and 80000903h, each about fifteen times a second**, the browser drawing "There is no data." |

So the two consoles agree on every call both make, and differ in two the EE makes on PCSX2 alone: the pad server's
fourth call onwards, and the browser's polling. The pad server's reply at field 75 (128 bytes into EE memory) is the
last exchange before the two consoles' traffic parts; its contents are a module's data, which the step did not read.

### 30.41 OSDSYS's browser again, and P8 (measured 2026-10-26)

The two traced runs are this step's runs of the browser (the tracing build changes no unit's behaviour; part B's
script, Cross every 120 fields from 770, the ninth on "Browser" at 1730, run to field 2200 with a picture every 60):

| Fields | Slot 1 empty | The formatted card in slot 1 |
|---|---|---|
| 770-1610 | the setup stepped on at each press | the same |
| 1680 | the main menu with its ring of lights | the same |
| 1740-1860 | the menu fading out | the same |
| 1920-2160 | **black, every pixel zero** | the same |

The 36 pictures of the two runs are equal byte for byte, and equal byte for byte to §30.35's at the same fields; the card
was never written; nothing reached slot 1 on entering the browser (no call to the card server after field 53). **P8
stands as observed**: card or none, the browser stalls before it asks for any card, and the card's refusal (Q7(b)) is not
reached.

### 30.42 Questions the SIF link step leaves

1. **Q1 [decision], the next look at the browser.** The link shows the EE no longer asking, not an answer missing.
   *Recommended*: (a) read the pad server's reply at field 75, the 128 bytes the END packet names, on Sapphire and on
   PCSX2 at the same address, compared word for word and never committed, the first exchange after which the consoles
   part; if they differ, trace SIO2's command bytes to the DualShock 2 against its documented protocol (ps2tek's and
   psx-spx's command lists) to find the command Sapphire's pad answers otherwise, then prove it with a program; if they
   are equal, (b) trace the EE kernel's system calls narrowly (number, the arguments ps2tek's "BIOS EE Syscalls" gives,
   the thread), never an instruction or a program counter, to see what the browser's thread and the pad loop wait on.
   Both read data the BIOS made, as the SIF trace did, so both need the tester's allowance under §1.4. (c) leave the
   browser until S29's disc and S10's fast-booted programs exercise the card's driver through the project's own programs.
2. **Q2 [routine], INT_SBUS.** It is unmasked while the browser is black and Sapphire never raises it (D-141).
   *Recommended*: keep it undriven until a document or a step names its source; nothing in the trace waits on it visibly.
3. **Q3 [routine], the EE's reads of 1F40200Fh.** *Recommended*: a program of the project's that reads the drive's
   registers from the EE on PCSX2 with a disc and without, at S29, before a disc's type can matter; with the tray empty
   the 0 read is the drive's own answer.

### 30.43 What was read, the blast radius, and exposure (2026-10-26)

**Read**: `Sapphire_Plan.md` §1.4 (with the decision of 2026-10-26), §5.8, §11 Q7, §12, §13 (P8); this page's §26, §30
from part B on, §35 and §36; `Sapphire_Disputes.md` D-360, D-383, D-440, D-540, D-541; `Sapphire_References.md` §3.
ps2tek at `1c91660`: "Subsystem Interface (SIF)" whole (lines 4209-4560: the registers, SIF DMA, "SIF RPC Basics",
"SIF RPC Structs and Definitions", "SIF RPC Commands", "SIF RPC System Servers"), the EE and IOP I/O maps (lines
140-270), "CDVD I/O Ports" and "CDVD S Commands", "EE Timers"' first lines, a search's hits for "alarm" (lines 1185,
5052-5058, 5736-5742) and "PADMAN" (its section, "(todo)"); Sony's *EE User's Manual* §3.1. PCSX2 2.8.2 run as a black
box, its EE RAM through PINE at the receive buffer only, and its pictures. No emulator, SDK, loader or ps2autotests
source and no firmware: the BIOS image was handed by path to Sapphire and to PCSX2 and never opened; no program counter,
instruction, or memory holding BIOS code was printed or read; of the BIOS's data only the SIF packets' documented
header fields, and the units' registers. The tracing build, the scratch copy of the referee tool and every trace were
deleted once this was written.

**Blast radius**: the step commits documents only (`blast.sh WiseMan sapphire-browser` lists no test); nothing was run
beyond the traces.

**Exposure** (§1.4, recorded in §35.13 as well): the writer of this step ran the player's BIOS on Sapphire and on PCSX2,
traced the SIF link's tags and documented RPC packets on Sapphire (with, in one run, the first four words of some
module buffers, §30.39) and read PCSX2's receive buffer, and looked at the EE's and the IOP's units' registers and
counts; **this writer is exposed and will never write the open replacement**.

### 30.44 Dispute D-640: the pad's SIO2 traffic around field 75 (2026-10-26)

*A dispute step on the tester's go of 2026-10-26 (§30.42 Q1, narrowed by the tester to the pad's command and reply bytes
on SIO2 around field 75), under `Sapphire_Plan.md` §1.4's two allowances of that day; its product is
`Sapphire_Disputes.md` D-640.* SIO2's registers and every byte exchanged with the emulated DualShock 2s and cards were
traced on Sapphire from power-on to field 130, with the crate's formatted card in slot 1 and with slot 1 empty, and each
controller exchange was checked against the documented DualShock 2 protocol. **No exchange is one a driver would
reject**: from field 56 the IOP's pad driver identifies each pad and configures it in the documented sequence, one
command a field, every reply as psx-spx and ps2tek give it, and from field 68 reads it with 42h every field without a
break; nothing on SIO2 changes at field 75, where the EE's calls to the pad server stop. PCSX2's own pad log, allowed
for this step, shows its pad configured to the same state as Sapphire's (digital, the Analog button usable, both motors
unmapped, the digital reply bytes). Two differences between the two consoles' conditions were tried and ruled out: the
second DualShock 2, which Sapphire's settings plug into port 2 by default and the referee's do not (§30.47), and the one
reply byte in the sequence that Sapphire gives otherwise than PCSX2, 43h's buttons when configuration mode is entered
(D-223, §30.48). The pad and SIO2 are therefore not where the EE's pad loop stops; no code was changed, since no defect
was found for a program to prove.

### 30.45 What SIO2 carried on Sapphire (measured 2026-10-26)

A build of its own, not committed, printed with the field each SEND3 entry of every queue (port, length), the command's
bytes and the device's reply (a card's to its twelfth byte), whether a device took the peripheral byte, each write to
CTRL, SEND1, SEND2 and the flags with FIFOIN's and FIFOOUT's fill, and each queue's end with RECV1: emulated hardware
traffic, as the tester allowed on 2026-10-26; no program counter, instruction or memory content. The runner was
`examples/s25b.rs` with the Americas image and no presses, to field 130. The trace was kept in scratch and deleted once
this section was written.

| Fields | Port 1's pad (port 0) | Port 2's pad (port 1) |
|---|---|---|
| 25 | a 7-byte command with the peripheral byte 61h (psx-spx: the DVD remote's receiver), unaddressed, HiZ | the same |
| 40, 56, 57 | 42h, 5 bytes: FFh 41h 5Ah FFh FFh, digital, nothing pressed | none |
| 58 | 43h with 01h (enter configuration): FFh 41h 5Ah FFh FFh | none |
| 59 to 67 | 45h, 46h index 0, 46h index 1, 47h, 4Ch index 0, 4Ch index 1, 4Dh with six FFh, 41h, 43h with 00h (exit), 9 bytes each, the unused bytes 5Ah | none |
| 68 on | 42h, 5 bytes, every field | 42h at fields 72 and 77 |
| 78 to 87 | 42h every field | 43h with 01h, then the same nine commands as port 1's at fields 59-67 |
| 88 on | 42h every field | 42h every field |

Each reply against the documents (psx-spx "Controllers - Configuration Commands", "Vibration/Rumble Control", "Analog
Buttons (Dualshock2)"; ps2tek, Israelson's edition, "SIO2 Controller Commands"):

| Command | Sapphire's reply | Documents | PCSX2 (`s25-pad`, §30.9) |
|---|---|---|---|
| 42h, digital | FFh 41h 5Ah FFh FFh | the mode 41h, 5Ah, the buttons (1 released) | equal |
| 43h 01h from normal mode | FFh 41h 5Ah FFh FFh, then configuration mode | as 42h (D-223) | 00h 00h for the buttons |
| 45h | FFh F3h 5Ah 03h 02h 00h 02h 01h 00h | 03h a DualShock 2, 00h the LED off | equal |
| 46h 0, 46h 1 | 00h 00h 01h 02h 00h 0Ah; 00h 00h 01h 01h 01h 14h | the same | equal |
| 47h | 00h 00h 02h 00h 01h 00h | the same | equal |
| 4Ch 0, 4Ch 1 | 00h 00h 00h 04h 00h 00h; 00h 00h 00h 07h 00h 00h | the same | equal |
| 4Dh FFh × 6 | the old map, FFh × 6 | the old map; FFh × 6 "disable motor control" | equal |
| 41h, digital | six 00h | ps2tek: zeros in digital mode (D-225) | equal |
| 43h 00h in configuration mode | six 00h, normal mode again | the same | equal |

The two traces, card and none, are the same command for command and byte for byte at every controller port; they
differ only at the card ports (fields 52 and 53). Every queue was preceded by CTRL's FIFO reset and the SEND1 and SEND2
words of its ports, and followed by the flag at 1F808280h written back; a controller's 5-byte queue took 20 µs and a
9-byte one 36 µs (§30.3's 4 µs a byte, D-440). From field 77 the two ports' commands go in one queue. Field 75 carries
port 1's 42h read and nothing else, as fields 68 to 76 each do; port 2's pad is first read at field 72, three fields
before.

### 30.46 PCSX2's pad, by its own log (measured 2026-10-26)

*Decided 2026-10-26 by the tester, for this step:* PCSX2's pad trace log may be read for the emulated controller's
command and reply bytes and their timing only, with no IOP program counter, BIOS address or module data, lines of that
kind filtered out before reading; the log stays in scratch, is never committed, and is deleted when the step is
recorded. A scratch copy of `ps2referee.py`, not committed, ran its `pads` mode with PCSX2's trace log on for the IOP's
pad (`EmuCore/TraceLog`, `IOP.pad`) and kept only the lines beginning `Pad: `. PCSX2 2.8.2's release build logs no
command or reply byte: it prints one line when a pad's configuration finishes, giving the port, the LED, whether the
Analog button is usable, each motor's state and the reply bytes' set. With slot 1 empty:

| PCSX2 | Its line | Sapphire at the same point |
|---|---|---|
| one pad (the referee's setting) | port 1: LED off, Analog button usable, both motors disabled, reply bytes digital | after field 67: digital, unlocked, the map FFh × 6, the mask 03h |
| a second pad in port 2 | the same for port 2, 0.334 s (20.0 fields) after port 1's | port 2's sequence ends at field 87, 20 fields after port 1's |

So PCSX2 configures each pad to the state Sapphire's reaches, and the second 20 fields after the first, as Sapphire
does; it is a witness for the result of the sequence and its spacing, not for each command, which its log does not show.
With trace logging on, the `-logfile` copy of PCSX2's log stops after its start-up lines and the rest goes to its
default log, so the scratch tool read the `Pad: ` lines from there (§30.51 says what that first cost).

### 30.47 The second pad (measured 2026-10-26)

Sapphire's "Port 2 controller" setting (`pad2`, `v1.rs`) plugs a DualShock 2 into port 2 by default, and every
browser run since part B had one, while the referee's settings have none in port 2 ([Pad2] Type None) and §30.40's
PCSX2 runs were made so. Port 2's pad is first read three fields before the pad server's last call, so both were run
the other way:

| Run | Fields 1920-2700 |
|---|---|
| Sapphire, port 2 empty, the formatted card in slot 1 | black, every pixel zero |
| Sapphire, port 2 empty, slot 1 empty | black, every pixel zero; the 45 pictures equal to the card run's byte for byte |
| PCSX2, a DualShock 2 in port 2, slot 1 empty, the sixth Cross entering the browser | "There is no data." |

The two Sapphire runs step through the setup to the menu as every run since D-383 has (17,472 lit pixels at field
780, as in §30.26); the EE's system calls were 1,158,276 in each (1,157,984 and 1,157,988 with two pads, §30.35); SIO2
carried about 1.1 queues a field after the menu, as with two pads. The second pad is not the cause.

### 30.48 D-223's byte, tried (measured 2026-10-26)

In the whole sequence of §30.45 one reply differs between Sapphire and the referee: 43h's two button bytes when it
enters configuration mode, the buttons as both documents give them on Sapphire (FFh FFh, nothing pressed) and 00h 00h on
PCSX2 (D-223, §30.9). The tracing build was run once more answering PCSX2's zeros there, an experiment not committed,
with slot 1 empty, two pads, the trace on from field 50 to 100 and part B's script to field 2700. The trace showed the
driver's commands at both ports unchanged, command for command, with only those two bytes different at fields 58 and
78; the browser was black from field 1920 to 2700, every pixel zero, the 45 pictures equal byte for byte to the port 2
empty run's of §30.47, and the EE's system calls 1,157,988, §30.35's for the same run with the documents' bytes. So the
referee's zeros change nothing a picture or the counts show, and D-223's documented reading stands.

### 30.49 OSDSYS's browser again, and P8 (measured 2026-10-26)

No unit was changed, so the browser runs of §30.41 stand for the tree as it is (two pads, card or none: black from field
1920, the card never written). The runs of this step add those with port 2 empty (§30.47), part B's script (Cross every
120 fields from 770, the ninth on "Browser" at 1730, Circle at 2510) to field 2700:

| Fields | Slot 1 empty | The formatted card in slot 1 |
|---|---|---|
| 770-1610 | each press steps the setup on: Language, Time Zone, Daylight Savings Time, "Settings completed." | the same |
| 1680 | the main menu, "Browser" and "System Configuration", with its ring of lights | the same |
| 1740-1860 | the menu fading out on Cross | the same |
| 1920-2700 | **black, every pixel zero**, through the Circle at 2510 | the same |

The card was never written (its file equal to the crate's). **P8 stands as observed**: card or none, one pad or two,
the browser stalls before it asks for any card, and the card's refusal (Q7(b)) is not reached.

### 30.50 Questions the pad step leaves

1. **Q1 [decision], the next look at the browser.** The pad's traffic, SIO2 and the SIF link all answer as documented
   and as the referee does; what stops is an EE thread that on PCSX2 calls the pad server every 64 fields. A cadence of
   64 fields is the kind a kernel alarm keeps, and ps2tek's "EE Timers" says the BIOS reserves timer 3 for alarms, which
   Sapphire's EE never writes after field 13 (§30.39). *Recommended*: (a) read EE timer 3's four registers on PCSX2
   through PINE, once a second while its menu runs, if PINE reaches the EE's I/O registers; they are hardware registers,
   not BIOS data, and whether the referee's BIOS arms timer 3 where Sapphire's never does names a unit (the EE's timer,
   its interrupt, or the INTC) without reading the BIOS; then a program of the project's own on that unit. (b) §30.42
   Q1(b), the EE kernel's system calls traced narrowly, if (a) shows nothing. (c) leave the browser until S29's disc and
   S10's fast-booted programs drive the card's driver.
2. **Q2 [routine], the second pad in the referee's runs.** *Recommended*: keep the referee's port 2 empty, as it was for
   every recorded run, and Sapphire's default of two pads, since the browser was shown to behave alike with either.
3. **Q3 [routine], D-223.** *Recommended*: keep the documents' reading, the buttons, as §30.48 ruled the referee's zeros
   out as the browser's cause; revisit if a second lineage disagrees.

### 30.51 What was read, the blast radius, and exposure (2026-10-26)

**Read**: `Sapphire_Plan.md` §1.4 (with the decisions of 2026-10-26), §5.8, §11 Q7, §13 (P8); this page's §30 from part
B on (the D-383, D-540 and D-620 steps), §35 and §36; `Sapphire_Disputes.md` D-223 to D-225, D-383, D-440, D-441, D-540,
D-541, D-620. psx-spx at `6d7d1bc`, "Controllers - Communication Sequence", "Configuration Commands", "Vibration/Rumble
Control", "Analog Buttons (Dualshock2)" and the controller IDs; ps2tek, Israelson's edition (revision `295bc61`), "SIO2
Controller Commands" and "SIO2 Command Sequence". PCSX2 2.8.2 run as a black box: its pictures from saved states and the
`Pad: ` lines of its log, three runs. No emulator, SDK, loader or ps2autotests source and no firmware: the BIOS image was
handed by path to Sapphire and to PCSX2 and never opened; no program counter, instruction, or memory holding BIOS code or
data was printed or read. The tracing build, the scratch copy of the referee tool, every trace and every PCSX2 log were
deleted once this was written.

**Blast radius**: the step commits documents only (`blast.sh WiseMan sapphire-padtrace` lists no test); nothing was run
beyond the traces and the runs above.

**Exposure** (§1.4, recorded in §35.13 as well): the writer of this step ran the player's BIOS on Sapphire and on PCSX2,
traced SIO2's registers and the bytes the emulated pads and cards exchanged on Sapphire, and read PCSX2's pad log. The
first PCSX2 run with trace logging on wrote its log to PCSX2's default file instead of the tool's, and to find the pad's
lines in it the shape of every line was printed once with its numbers and hexadecimal values masked, before any filter:
besides PCSX2's own start-up text, that showed lines of PCSX2's recompiler messages carrying instruction mnemonics
(operands masked), the IOP modules' library registrations by name and version, and the BIOS's name as PCSX2 reports it;
no address, operand or value was read, and the later runs kept only the `Pad: ` lines. **This writer is exposed and will
never write the open replacement**.

## 31. Stage S18, the GS's texture mapping, CLUTs, LOD, filtering, wrap modes and fog (2026-10-25)

*Numbered §31 and its disputes from D-240, since S14 (§28, from D-180), S20 (§29, from D-200) and S25 (§30, from D-220)
are built beside it.*

### 31.1 In one paragraph

The plan's §7 row S18, on the tester's go of 2026-10-25. The GS textures: **`gs::texture`** keeps the CLUT's temporary
buffer as the GS's own state, loaded from local memory when TEX0 or TEX2 is written as CLD says, reads the context's
TEX0, TEX1, CLAMP, MIPTBP1, MIPTBP2 and TEXA at the drawing kick, samples every texture format S16's layout holds
(nearest, bilinear and the four mipmapping modes, at a LOD from Q or from K), wraps or clamps on whole texels, expands
16- and 24-bit alpha by TEXA, applies the texture function and then fog, in the place S17's pipeline left for them;
**`gs::exact`** interpolates S, T and Q, or U and V, at each pixel without rounding and takes the result to sixteenths
of a texel, the texel coordinate system's own fixed point (§2.6.2), and computes the LOD's log2 the same way. Nine
programs of the project's, 196,608 graded record words, are equal on Sapphire to the record of S17's model extended by a
model of chapter 3.4 written apart from `src/gs/`, which also draws 720 generated textured primitives as the source
does. **PCSX2 equals four programs word for word** (every texture format, every CLUT case, every TEXA case, and the
twelve flat pages of the texture functions) and departs in five places, each a dispute: bilinear filtering rounded
after each of its two stages (D-249), the LOD held to eight fractional bits and Q ignored with UV (D-242), MMAG at a LOD
of 1/16 (D-241), a blend between mip levels that leaves blue and alpha unblended in two pixel columns of every four
(D-250), and texel boundaries crossed within 2^-22 of a texel (D-247); fog's arithmetic was measured on it and follows
FOG's page against §3.5's formula (D-248). Disputes D-240 to D-250. No emulator, SDK or loader source, no ps2autotests
source and no firmware was read.

### 31.2 What was built

- **`src/gs/texture.rs`**: `Clut`, the 512 sixteen-bit places of the temporary buffer (a PSMCT32 entry's low half at
  place k and its high half at 256 + k, as §3.4.7's CSA tables draw them) and CBP0, CBP1, loaded by `Clut::load` in
  CSM1's 8×2 and 16×16 arrangements or CSM2's row at TEXCLUT's offset (§2.7.3); `auto_miptbp1`, MTBA's calculation;
  `Sampler`, the context's registers at the drawing kick, with `sample` (LOD, level and filter), `function` (TFX and
  TCC) and the texel's conversion (CLUT, RGBA16's expansion, TEXA); `fog`; `Setup`, a primitive's texture coordinates
  ready to interpolate.
- **`src/gs/exact.rs`**: single-precision values as exact numbers (`Float`), their sums weighted by a pixel's integer
  barycentric weights (`Sum`, in fixed 768-bit integers), `quotient` (floor(2^scale × ΣSw / ΣQw)), and `log2_scaled`
  (floor(2^bits × log2 of a ratio)). Nothing allocates.
- **`src/gs/raster.rs`**: each primitive interpolates texture coordinates and the fog coefficient by the weights it
  already uses for colour and Z, only for a pixel that has passed scissoring and only when the primitive is textured or
  fogged. **`src/gs/pixel.rs`**: texture mapping, then fog, after scissoring and SCANMSK and before the depth test.
  **`src/gs/draw.rs`**: `Environment` carries the `Sampler` when TME is set and FOGCOL when FGE is. **`src/gs/mod.rs`**:
  TEX0 written runs MTBA's calculation and the CLUT's load; TEX2 written replaces TEX0's PSM, CBP, CPSM, CSM, CSA and
  CLD and loads (TEX2's page); the GS owns the `Clut`.
- **No new cross-component dependency**; `gs::local` is unchanged, and every texel is read through it.

### 31.3 The readings the documents leave open

[R *GS User's Manual* §2.6, §2.7, §3.1, §3.4 whole, §3.5, and chapter 7's pages for TEX0, TEX1, TEX2, CLAMP, TEXA,
TEXCLUT, TEXFLUSH, MIPTBP1, MIPTBP2, ST, UV, RGBAQ, FOG and FOGCOL; ps2tek "GS Textures", "GS Fog", "GS Primitives",
"GS Vertex Attributes"; A]:

- **Texture coordinates** (D-247). §3.4.10: "After linear-interpolating S, T, and Q, the GS calculates the
  normalization texel coordinate values from s = S/Q and t = T/Q"; §2.6.2: texel coordinates are fixed point with four
  fractional bits, a texel's centre at .5. The reading: S, T and Q are the exact numbers their single-precision bits
  denote (a zero exponent field is zero and the largest an ordinary exponent [A], as the EE's floats are taken at S9),
  interpolated by the pixel's exact barycentric weights, divided exactly, multiplied by USIZE or VSIZE, and floored to
  sixteenths of a texel: u16 = floor(2^(TW + 4) × ΣS_i w_i / ΣQ_i w_i). The weighted sums are held in 768-bit integers,
  enough for any exponents and weights the registers allow, so no input rounds. UV (int 0:10:4) is interpolated by the
  same weights and floored. A coordinate beyond ±2^40 sixteenths is kept as a stand-in beyond every clamp's bound on
  its side and equal to it in its low 24 bits, all that REPEAT or a region's mask reads. ΣQw of 0 gives 0 [A]. A sprite
  takes S and U along x from its first vertex to its second, T and V along y, and the second vertex's Q and fog
  coefficient (§3.2.8 gives a sprite's Z and fog from the second vertex; Q is taken with them [A]); a point its
  vertex's; a line by its major axis, continued past its ends as S17's colour is (D-165), the fog coefficient held to
  0 to 255. Texture coordinates and fog are interpolated whatever IIP says.
- **The LOD** (D-241, D-242). TEX1's page: LOD = (log2(1/|Q|) << L) + K with LCM 0, LOD = K with LCM 1, K being int1:7:4.
  The LOD is held in K's fixed point: floor(2^(L + 4) × log2(1/|Q|)) + K sixteenths, the logarithm exact. It is
  computed by `log2_scaled`: the ratio's exponent exactly, then its significand to 256 bits squared once a bit, so the
  result is the exact floor unless a power of the ratio lies within 2^-250 of a power of two. With STQ Q is the pixel's
  interpolated Q ("MIPMAP (Q Linear Interpolation): Possible", §3.4.4); with UV it is not interpolated ("Impossible")
  and §3.4.12 says "LOD is calculated from the Q value even if the texture coordinates are given by UV", so the Q of
  the vertex of the drawing kick is used [A], as flat colour is. Q of 0 is the farthest level [A]. A LOD of 0 or less
  takes MMAG at level 0, above 0 MMIN (§3.4.12; TEX1's page says "LOD < 0" and "LOD >= 0", D-241). MMIN 2 and 4 take
  level m or m + 1 by rounding a half up, 3 and 5 blend levels m and m + 1 by the LOD's fraction in sixteenths, every
  level held to MXL (7 taken as 6 [A]); MMIN 6 and 7, undefined, act as 0 and 1 [A].
- **Filters** (D-249). Point sampling: the texel the coordinates fall in, at level n the coordinates divided by 2^n
  (§3.4.8, §3.4.11). Bilinear: §3.4.8's formula with α and β the fractions of (u − 0.5, v − 0.5) in sixteenths, the four
  products summed and the sum rounded down once. Levels are blended the same way (D-250). A level's size is the
  texture's halved n times, at least 1.
- **Wrap modes** (D-245). §3.4.5 on whole texels, each of a bilinear filter's four separately: REPEAT modulo the size,
  CLAMP to 0 and the size less one, REGION_CLAMP to MIN and then to MAX, as the formulas stand in order, REGION_REPEAT
  (u & UMSK) | UFIX; at level n the region's bounds and masks are shifted right by n (stated for REGION_CLAMP, taken for
  REGION_REPEAT [A]). TW and TH above 10 are 10 (ps2tek's "min(2^value, 1024)", [A]).
- **Formats and TEXA** (§2.6.3, §3.4.6). Every PSM TEX0's page lists, through `gs::local`; a Z format reads as its
  colour twin in its own layout [A]; PSMCT24 alpha TA0, or 0 for black with AEM; RGBA16 each colour EDCBA000, alpha TA1
  for A 1, TA0 for A 0, or 0 for black with AEM; the same for a 16-bit CLUT entry.
- **The CLUT** (D-243, D-244). §3.4.7's load "when the TEX0 or TEX2 register is accessed", under CLD's six codes against
  CBP0 and CBP1, codes 6 and 7 loading nothing [A]; only an indexed PSM loads, 16 or 256 entries, the count its PSM
  gives; entries at CSA × 16 wrapping in the buffer (256 places of 32 bits, 512 of 16); CSA applies in CSM2 too; a CPSM
  the page does not list reads as PSMCT32 [A]. The CPSM at drawing decides how the buffer is read, so a CLUT loaded in
  one format and read in another, or overwritten in part, shows what the buffer holds.
- **TEX2 and MTBA** (D-246). TEX2 replaces TEX0's bits 20 to 25 and 37 to 63 (TEX2's page: "subsets of the TEX0
  register"). MTBA's calculation, run when TEX0 is written with TEX1's MTBA set, writes MIPTBP1: each of levels 1 to 3
  after the one before, a level taking its texels' bits rounded up to whole 64-word blocks, its width the level's texels
  over 64, at least 1; §3.4.11's PSMCT16 512×512 example comes out as printed.
- **The texture function** (D-240). §3.4.9's table, A * B = (A × B) >> 7, every result held to 0 to FFh; with TCC RGB
  the fragment's alpha is kept in every function, DECAL included, although the table prints At there.
- **Fog** (D-248). (F × C + (100h − F) × FOGCOL) >> 8, so that F 0 gives FOGCOL itself, as FOG's page says; §3.5's two
  products each shifted by 8 give FOGCOL less one at F 0.
- **TEXFLUSH** is accepted and does nothing: every texel is read from local memory as it is when its pixel is drawn
  (§31.9).

### 31.4 The interfaces: where the later stages attach

- **S19** takes the textured and fogged colour in `pixel::draw` where S17 marked the alpha tests, blending, dithering,
  COLCLAMP and FBA; antialiasing (AA1) replaces alpha after fog, before the tests.
- **The state format** (S10, S30) gains the `Clut` (512 entries and CBP0, CBP1) with the GS's registers and memory.
- **The device path** (S44 to S46) is graded against this sampler; `exact`'s arithmetic is the measure of any faster
  form (§5.13's C5).

### 31.5 Cost

Not measured for speed. A textured pixel with STQ costs two exact quotients of fixed 768-bit integers and, with LCM 0,
an exact logarithm; with UV, a 128-bit division. Nothing allocates. The project's nine programs and the generated
primitives run in 1.4 s on the desktop in release.

### 31.6 The gate: the programs (measured 2026-10-25)

`tests/s18/` writes each program as an S16 script, every texture, CLUT and frame filled first with generated data
through IMAGE and TEXFLUSH written after (§3.4.13's step 4), every frame page read back raw as PSMCT32; its expected
record comes from S17's model (`tests/s17/manual.rs`), which hands each pixel's weights to `tests/s18/manual.rs`, written
apart from `src/gs/`: the CSM1 arrangement transcribed from §2.7.3's figure, the buffer as figure 3-12's places, S, T
and Q aligned to one exponent in 128-bit integers, log2 in double precision asserted to be at least 10^-9 from a
boundary unless the ratio is a power of two, and each formula in its document's order.

| Program | Graded words | What it covers |
|---|---|---|
| `s18-formats` | 26,624 | every PSM TEX0 names, 13 textures of 64×32 read 1:1 at texel centres from memory written raw, the indexed ones through a PSMCT32 CLUT, DECAL with TCC RGBA, TEXA's TA0 and TA1 |
| `s18-clut` | 49,152 | CSM1 in PSMCT32, PSMCT16 and PSMCT16S for PSMT8 and PSMT4 with CSA 0, 5, 20 and 31; CSM2 at two TEXCLUT offsets; CLD 2, 0, 4, 4, 3, 5, 5 and 4 with the CLUT's memory rewritten between draws; TEX2; a 16-bit CLUT at CSA 20 over a 32-bit one's upper halves; PSMCT32 with CLD 1; CLD 6; CSA past 256 and past 512 entries; CSA in CSM2; CPSM 1 |
| `s18-tfx` | 26,624 | MODULATE, DECAL, HIGHLIGHT and HIGHLIGHT2 with TCC RGB and RGBA over 32 flat fragment colours, 0, 7Fh to 81h and FFh among them; PSMCT24 and PSMCT16 under TEXA; a Gouraud triangle under MODULATE |
| `s18-uv` | 8,192 | UV on an affine quad, on sprites magnified, minified, reversed, on and a sixteenth off texel boundaries, past 1,000 texels; lines, points, a strip and a fan |
| `s18-stq` | 8,192 | ST with Q 1 at texel centres, past 1.0 and below 0; a quad in perspective (Q 0.25 to 2); Q negative at every vertex; S, T and Q near 10^-3 and 10^3; lines and points |
| `s18-filter` | 26,624 | bilinear magnification through every sixteenth with REPEAT and CLAMP; bilinear over PSMCT16 and PSMT8; LODs of 0 and ±1/16; LODs between 0 and 1; every level by K and past MXL; MMIN 2 to 5; LCM 0 with Q per quarter and with L; LCM 0 per pixel in perspective; LCM 0 with UV; MTBA on 64×64 PSMCT32 with TBW 1 and 4 and on 32×32 PSMT8, then MTBA 0 |
| `s18-clamp` | 28,672 | a 16×16 texture read from 24 texels before it to 40 past it in every wrap mode and pairs of them, point sampled and bilinear; MINU above MAXU; REGION_REPEAT reaching past the texture; both region modes at level 1 |
| `s18-texa` | 14,336 | AEM 0 and 1 with PSMCT16, PSMCT24 and a PSMT4 texture's PSMCT16 CLUT, their black texels and entries (A 0 and A 1) drawn by the GS into them first |
| `s18-fog` | 8,192 | F 0, 1, 40h, 7Fh, 80h, FEh, FFh and 33h by FOG and by XYZF2's field, a sprite's first vertex's F ignored; F over Gouraud and flat triangles, lines and points; fog after MODULATE |

**Tests** [M, `cargo test --release`]: `tests/texture.rs` **11 of 11** — the nine programs, each equal to its record word
for word; `generated_textured_primitives_draw_as_the_model_draws_them` (six rounds of 120 textured primitives of every
kind over 16 pages of generated memory, each with random TEX0 in every PSM and CLUT mode, TEX1, CLAMP, MIPTBP1 and 2,
TEXA, TEXCLUT and FOGCOL, UV or STQ with Q of either sign, fog and shading at random: every pixel equal between the
source and the model); `the_programs_assemble_and_their_records_end_in_the_done_word`. Lib **46 of 46** (two new, on
`exact`'s quotient and logarithm against direct computation). The crate's other tests unchanged, on the tree with S17,
S20 and S14 beneath: `cop0` **19 of 19**, `dmac` **18 of 18**, `draw` **13 of 13**, `fpu` **9 of 9**, `gs` **12 of
12**, `interpreter` **23 of 23**, `iop` **13 of 13**, `micro` **12 of 12**, `mmi` **13 of 13**, `pcrtc` **5 of 5**,
`programs` **3 of 3**, `sif` **5 of 5**, `vif` **12 of 12**, `vu0` **14 of 14**, `vu_micro` **24 of 24**; lib, `gs`,
`draw` and `texture` also under `test-checked`; clippy clean. WiseMan with `blast.sh`'s filter (`SapphireTests`) **5 of
5**.

### 31.7 On the referees (measured 2026-10-25)

`examples/s18.rs` writes each program as an ELF with its expected record and a background record (S17's form);
`ps2referee.py ram pcsx2 … --until 1FFFF0=5A99D00E 200000:<words>` read each from PCSX2 2.8.2 (Americas BIOS, fast
boot, software renderer), one process at a time. Only the records were read. Play! was not run: it shows no drawing
through its local memory (§27.7).

| Program (words changed by drawing) | PCSX2: words differing | What they are |
|---|---|---|
| `s18-formats` (26,624) | 0 | — every PSM, the CLUT, TEXA |
| `s18-clut` (49,152) | 0 | — every CSM, CPSM, CSA, CLD and CBP0/CBP1 case, TEX2, the shared buffer, D-243, D-244 |
| `s18-tfx` (25,632) | 971 | all on the Gouraud page; the twelve flat pages equal (D-240). in each channel 107 to 134 of them are values no 8-bit fragment colour gives under MODULATE, so PCSX2 modulates by a Gouraud colour carried with more bits (D-160's precision) |
| `s18-uv` (5,164) | 5 | a line's end on a diamond's lower-left edge, whose pixel PCSX2 leaves undrawn (D-161, amended); four pixels of the strip whose U lies exactly on a whole texel, where PCSX2 takes the texel below (D-247) |
| `s18-stq` (5,874) | 112 | every one a coordinate a little above a whole texel, by 1.5 × 10^-8 to 3.6 × 10^-7 of a texel, where PCSX2 takes the texel below (D-247) |
| `s18-filter` (26,127) | 9,936 | bilinear rounded after each stage on the magnification pages, reproduced at every pixel (D-249); MMAG at LOD 1/16 (D-241); the blend's columns (D-250); LOD at eight fractional bits, and per pixel computed otherwise (D-242); Q ignored with UV (D-242); level choice by K, MXL, rounding and MTBA all equal |
| `s18-clamp` (28,672) | 1,959 | only the four bilinear pages, reproduced at every pixel by two-stage rounding (D-249); every wrap mode, MIN above MAX, level-1 regions equal (D-245) |
| `s18-texa` (14,336) | 0 | — AEM, TA0, TA1, 24- and 16-bit, the CLUT |
| `s18-fog` (5,208) | 1,249 | none on the flat sprites, whose 16 values measured D-248; one step of colour on the triangles, lines and points, where F and colour are interpolated (D-160's precision) |

So **PCSX2 is a witness for texel addressing in every format, the CLUT buffer and every CLD, CSA and CSM case, TEXA,
the texture functions with TCC, the wrap modes and their regions at levels, MTBA, level selection by a fixed LOD, and
fog's formula**; not for bilinear rounding, the LOD's precision or Q with UV, nor for the blend between levels, where
its departures are its arithmetic's or its renderer's, not a rule a document states. Its blend's departure was checked
by reproducing it: blending by the LOD's fraction with two-stage bilinear and leaving blue and alpha at level m in
columns 2 and 3 of every four reproduces every word of the five constant-LOD mipmap quarters it was tried on.

### 31.8 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | PSMT4HL's texel read from bits 31..28, PSMT4HH's nibble, in `Sampler::texel` | `every_texture_format_is_read_…`, `generated_textured_primitives_…` |
| F2 | Bilinear weights rounded after each stage, horizontal then vertical, in `Sampler::level` | `bilinear_lod_and_the_mipmap_filters_…`, `every_wrap_mode_acts_…`, `generated_textured_primitives_…` |
| F3 | CSM2's CLUT read in CSM1's arrangement, in `Clut::load` | `the_clut_buffer_loads_…`, `generated_textured_primitives_…` |

Each was applied, `tests/texture.rs` run, and the source restored [M]. F2 is PCSX2's arithmetic (D-249).

### 31.9 What is not modelled

- **The texture page buffer and TEXFLUSH** (§3.4.7, TEXFLUSH's page; ps2tek "Recursive Drawing"): every texel is read
  from local memory when its pixel is drawn, so a program that omits TEXFLUSH sees what a console shows after one, and a
  primitive whose texture is its own frame reads pixels it has already drawn. The page buffer's size (8 KB, §1.1.2),
  fill and replacement are not documented; a named loss, a dispute candidate when a game at S28 reads its own frame.
- **Drawing time**: the CLUT's load and every texture read take no time.
- **FIX** (PRIM bit 10) is carried and has no effect; antialiasing is S19's.
- **The size limits of TEX0's page and §3.4.12** for bilinear and trilinear filters ("8 x 8 texels or more", "four or
  less ... correct images cannot be obtained"): smaller textures filter by the same arithmetic.

### 31.10 What the plan got wrong, or left open

- **§2.3** names "the fixed point of texture coordinates and LOD" and "the filtering's weights" as thin. The texel
  coordinate system's four fractional bits (§2.6.2) and K's format give the fixed point; the precision of the division,
  the logarithm and the filters' sums is not given, so the exact reading is the reference (D-242, D-247, D-249).
- **§3.4.12 and TEX1's page disagree at a LOD of 0, §3.4.9's table and §3.4.3 at DECAL with TCC RGB, §3.5's formula and
  FOG's page at F 0** — inconsistencies within the manual, settled by D-241, D-240 and D-248.
- **The gate's "each picture equal to the referees'"**: as at S17, PCSX2 alone, and its arithmetic departs from any exact
  reading in the filters and the LOD.

### 31.11 Questions this stage leaves

1. **Q1 [routine], bilinear rounding and the LOD's precision (D-249, D-242).** *Recommended*: keep one rounding of the
   weighted sum and the LOD in K's fixed point; PCSX2's two stages and eight bits are one emulator's arithmetic. Revisit
   with a second lineage.
2. **Q2 [routine], the texture page buffer (§31.9).** *Recommended*: keep reading memory directly until a game at S28
   draws from its own frame; then measure on PCSX2 with a program that omits TEXFLUSH and one that draws its own
   texture, before modelling a buffer whose size and policy no document gives.
3. **Q3 [routine], Q with UV (D-242).** *Recommended*: keep the manual's sentence (Q used with UV, the drawing kick's
   vertex's), against PCSX2's LOD = K.
4. The second picture lineage stays S17's Q1 [decision], unbuilt.

### 31.12 What was read for this stage

`Sapphire_Plan.md` §2, §5.4, §7 (S16 to S20), §8, §10 and §11; this page's §24 and §27; `Sapphire_Referees.md` §2.1, §3
and §6; `Sapphire_Disputes.md` D-100 to D-107 and D-160 to D-165. Sony's *GS User's Manual* (Version 5.0) §1.1, §2.6 and
§2.7 whole, §3.1, §3.2.2, §3.2.3, §3.2.8, §3.4 whole, §3.5, and chapter 7's pages for CLAMP, FOG, FOGCOL, MIPTBP1,
MIPTBP2, RGBAQ, ST, TEX0, TEX1, TEX2, TEXA, TEXCLUT, TEXFLUSH and UV; the kit's *Restriction* document's "Texture Width".
ps2tek at `1c91660`: "GS Primitives", "GS Vertex Attributes", "GS Textures", "GS Fog", "GS Special Effects and
Optimizations". No emulator, SDK or loader source, no ps2autotests source and no firmware.

### 31.13 What this stage does not cover

- The alpha test, destination alpha, blending, PABE, FBA, colour clamp, dithering and antialiasing (S19); the PCRTC (S20).
- The GIF (S15): until then nothing on the EE's side textures on Sapphire; the tests drive the host interface.
- The state format's new contents (the CLUT buffer): `SPH2` gains them with the rest (S10, S30).

## 32. Stage S26, the SPU2 on the shared voice crate (2026-10-25)

*Numbered §32 because stages S14, S25 and S18, built beside this one, write §28, §30 and §31; its disputes start at
D-260 for the same reason.*

### 32.1 In one paragraph

The plan's §7 row S26, on the tester's go of 2026-10-25. **First, alone and gated**: Diamond's SPU voice moved into
`blackmoon-spu` in BlackMoon Hardware (ADPCM, the interpolation, ADSR and the sweeps, noise, pitch modulation, each
voice's state, the reverb's algorithm with its area and offsets as parameters), `no_std` and depending on nothing;
Diamond's goldens unchanged sample for sample, its tests unchanged and passing, before any Sapphire code used the crate
(`BlackMoon_SPU.md` §3). **Then the SPU2** (`spu2`): two cores of 24 voices each, every voice each sample, on the IOP's
bus at 1F900000h and its DMA channels 4 and 7 in place of their stand-ins, each core's registers, mix, effects area and
AutoDMA, core 0's output into core 1, the IRQ addresses, and the output at 48,000 Hz exactly, a sample each 768 IOP
cycles. The documents give little: ps2tek's map rows, AutoDMA's MEMIN and the channels. The rest was learned from the
one referee that reaches the SPU2, PCSX2, through the IOP route of S21, in two ways found at this stage: **the output
areas**, rings in sound RAM where PCSX2 writes each core's voices 1 and 3, core 0's output and every voice sum each
sample, read exactly by the program itself (D-261); and **the captured sound**, PCSX2's output written to a file by
SDL's disk driver, which passes a high-pass filter and so serves for presence and level, not for samples (D-270). Eight
gate programs grade the voices, the envelope, the noise, the mix, the reverb, AutoDMA, the IRQ addresses and the
register read-backs on Sapphire against the record the shared voice and the documented mix give; on PCSX2 the mix's
copies equal Sapphire's 28 times in 30, AutoDMA's stream and the reverb off equal, the registers 24 of 24, the IRQ
scenes 28 of 30, and the voice itself departs in the last bits and at a key on where PCSX2 reads psx-spx differently
from Diamond's three PlayStation referees, which keep the shared voice (D-269). Disputes D-260 to D-270. No emulator,
SDK, loader or ps2autotests source and no firmware was read.

### 32.2 What was built

- **`blackmoon-spu`** (`EmuSen/Cores/Sony/BlackMoon-HW/blackmoon-spu/`): `voice`, `envelope`, `noise`, `reverb`,
  `tables` and the trait `Touch` (`BlackMoon_SPU.md`). **New cross-component dependencies**: `diamond` on
  `blackmoon-spu`, and `sapphire` on `blackmoon-spu`; the crate depends on nothing.
- **`spu2/mod.rs`**: `Spu2`, 2 MiB of sound RAM, the two cores, the registers outside them, the output queue (the
  shared `SampleQueue`) and the output areas' position; the register dispatch by halfword, the data port and the DMA
  words, the IRQ watch across both cores (`Watch`), the sample (core 0, then core 1 with core 0's output, then the
  output areas), and `Ports`, the IOP DMAC's peripherals for channels 4 and 7. **`spu2/cores.rs`**: `Core`, a core's
  block of registers, its volumes, 24 voices of `blackmoon-spu`, key on and off, the noise, the mix by side, MMIX's
  gates, the input volumes, the reverb's parameters and area, and the master volume. **`spu2/adma.rs`**: AutoDMA's
  buffers. **`spu2/regs.rs`**: the map, each row's source in its comment.
- **`iop/bus.rs`**: the field `spu2`; SPU2 reads and writes by halfword with their access size, ahead of the other I/O
  registers; the SPU2 brought to the bus's time with its units, its event scheduled while an IRQ is armed or AutoDMA
  runs, a buffer AutoDMA frees waking the DMAC; IRQ 9 raised. **`lib.rs`** declares the module; **`Cargo.toml`** names
  the crate. `clock`, `board` and `machine` are unchanged: the SPU2 lives on the IOP's bus as the SIF does.
- **`tests/s26/`**: the measurement programs (`s26-probe`, `s26-outputs`, `s26-rings`, `s26-mmix`, `s26-out`,
  `s26-explore`, `s26-adma-probe`, `s26-adma-sound`), ungraded, which found the rulings; and `gate.rs`, the gate
  programs with their checks. **`tests/spu2.rs`**, the gate's tests; **`examples/s26.rs`** writes every program for the
  referees and grades (`grade`) or compares (`diff`) a run's record, Sapphire's (`sapphire`) or a referee's.
- **`ps2referee.py --wave DIR`**: PCSX2's sound captured for a run (§32.8).

### 32.3 The register map: what the documents give and what the referee showed

ps2tek's "I/O Maps" gives the voices' 180h of registers, the voice addresses at 1C0h, ATTR, IRQA, TSA, the data port,
AutoDMA's control, ENDX, STATX, the volumes at 760h, and key on and off at 190h and 194h [R]. The referee keys nothing
on at 190h and keys on at 1A0h, and acts on 180h–198h as pitch modulation, noise, the voices' dry and wet masks by side
and MMIX (D-260): the PlayStation's register order, its voice mask split by side. The voice's eight halfwords are
VOLL, VOLR, PITCH, ADSR1, ADSR2, ENVX, VOLXL, VOLXR (read back on the referee: VOLX the current level, D-266); the
addresses are 20-bit halfword addresses in a high halfword of 4 bits and a low one. The reverb's registers sit at
2E0h–33Ch (ESA, the 22 offsets, EEA) and after MVOLX in the volume block (the ten coefficients), in psx-spx's order
(D-264); the external input's volume is AVOL (ps2tek's "Core1 External Input Volume") and the sound data input's BVOL
(D-262). 7C2h shows the IRQ flags (D-265); the other registers from 346h of each block and 7B0h on are kept as written.

### 32.4 The cores and the mix

Each sample, in order [A, the PlayStation's order, Diamond_SPU.md §2]: the key offs then the key ons written since the
last sample; the noise; the sound data input from MEMIN; the 24 voices, voice n pitch-modulated by voice n−1 where
PMON says, each through its volumes into the dry and wet sums by its masks; the external input through AVOL and the
sound data input through BVOL; MMIX's gates (D-262) sending the three sources' wet sums to the reverb and their dry sums
to the output; the reverb in its area with ATTR's bit 7 its writes' enable; the dry sum and the reverb's output through
MVOL. Core 0's output is core 1's external input in the same sample [C]; core 1's output is the SPU2's. ATTR's bits 15
and 14 gate nothing (D-262). The output areas are then written (D-261) and every write watched (D-265).

### 32.5 AutoDMA and the transfers

AutoDMA's control bit for a core (bit 0 core 0, bit 1 core 1 [R]) turns its channel's words into MEMIN blocks: 256
words, the left side's 256 samples then the right's, into the buffer the core is not playing; the core plays MEMIN at a
sample a sample, buffer 0 then 1, and a buffer it leaves is free for the next block, which the channel's request waits
for (D-267). Otherwise a channel's words go to the core's TSA as two halfwords each, low first, and the data port
writes a halfword at TSA; each steps TSA by one (D-263).

### 32.6 The output areas

PCSX2 writes each sample into sound RAM's first 2000h halfwords, rings of 200h at one position: voices 1 and 3 of core 0
at 400h and 600h and of core 1 at C00h and E00h; core 0's output at 800h and A00h; each core's dry and wet voice sums
at 1000h–1FFFh. Sapphire does the same (D-261). They are what the gate reads its sound from: a program copies a ring
into its record, and the check asks that it equal some 200h-sample window of the scene's record, read from the ring's
write position, which makes the comparison independent of the moment either machine copies it.

### 32.7 The gate: the programs (measured 2026-10-25)

The record of a voice is `blackmoon-spu` run on the program's own blocks in `tests/s26/gate.rs`, apart from `src/spu2/`;
the mix's record is written out there from the rulings. A scene starts from silence (every voice keyed off with a fast
release) and copies its ring after about 1,300 samples, past the key on, but for the scene that is about the key on.

| Program | Copies or words | What it covers | Sapphire | PCSX2 |
|---|---|---|---|---|
| `s26-voices` | 14 copies | a key on's first samples; four streams through every filter and shift, the reserved shifts and filters, saturation; six pitches up to the limit; pitch modulation by a constant and a square voice and none without PMON | 14 of 14 | 0 of 14 (D-269): within 4 without modulation, up to 111 through the filters, the key on as D-269 says |
| `s26-envelope` | 10 copies | linear and exponential attack, decay to two levels, sustain in all four modes, both releases from a constant level | 10 of 10 | 0 of 10: one lower throughout in eight, the exponential attack and the decay's end as D-269 says |
| `s26-noise` | 2 copies | voice 1 in noise mode at the fastest clock and at shift 12 step 1, against every phase of the generator's whole cycle | 2 of 2 | 1 of 2: the fastest equal, the slow clock faster (D-269) |
| `s26-mix` | 30 copies | voice 0 through its masks by side, MMIX's voice gates and MVOL into core 0's output and sums; core 1's sums and voice 1; core 0's voice 3 silent | 30 of 30 | 28 of 30: one sample at a ring's seam, and core 1's voice 1 by D-269's rounding |
| `s26-reverb` | 2 copies | an area of 1000h halfwords settled under a constant wet input; an area with the reverb off | 2 of 2 | 1 of 2: off equal; settled 16,315 against 16,309 (D-264) |
| `s26-adma` | 4 copies, 1 word | four blocks streamed: the first three heard in order and not the fourth when the transfer ends, each side through BVOL; MEMIN's layout | 4 of 4, 1 of 1 | 2 of 4, 1 of 1: the stream equal, MEMIN empty (D-267) |
| `s26-irq` | 30 words | the address at a header, a sample word and past the block; at four output areas; a transfer's halfword and the next; a reverb's area off and on; core 1's address hit by core 0; the flags, IRQ 9 and the clear | 30 of 30 | 28 of 30: the reverb off (D-265) |
| `s26-registers` | 24 words | TSA after the port; SSA's and LSAX's widths; a voice's registers and VOLX; key on at ps2tek's row and at 1A0h; ENVX after a block's end; NAX, LSAX and ENDX after a key on; ESA and EEA; STATX | 24 of 24 | 24 of 24 |

**Tests** [M, `cargo test --release`]: `tests/spu2.rs` **10 of 10** — the eight programs, each graded word by word and
check by check; the output at 48,000 Hz exactly over a second; and core 0's output reaching the output through core 1's
AVOL and MMIX's external gates, which the areas cannot show. Also under `test-checked`. `tests/iop.rs` **13 of 13**, its
`each_dma_mode_reaches_its_stand_in` now finding the slice and both bursts in the SPU2's cores at their transfer
addresses, where S21 had stand-ins; `tests/sif.rs` 5 of 5; `tests/sio2.rs` 5 of 5; lib 52 of 52; on the tree with S14,
S18 and S25 beneath, S25's SIO2 sharing the IOP's DMA peripherals with the SPU2. Clippy clean. `blackmoon-spu` 6 of 6 (also
`test-checked`). Diamond, after the move: lib 176, programs 84, shell 10, `diamond-v1` 5, `precision` 8, the goldens
unchanged (`BlackMoon_SPU.md` §3). WiseMan, the blast radius's filter `Diamond|SapphireTests`: **80 of 80**.

### 32.8 On the referee (measured 2026-10-25)

**Running SPU2 programs.** As S21's (`Sapphire_Referees.md` §4.3): an EE program places the IOP program and copies its
record; the record carries the copies of the output areas, read by the IOP program through core 0's DMA channel. PCSX2
2.8.2 runs every program; Play! cannot (§22.9); LRPS2 and PCEE2 are PCSX2's lineage.

**The captured sound.** PCSX2's audio settings name an SDL backend, and its SDL 3 carries the disk driver ("direct-to-disk
audio", its hints `SDL_AUDIO_DRIVER` and `SDL_AUDIO_DISK_OUTPUT_FILE`, read from the library's strings). `ps2referee.py
--wave DIR` sets the backend to SDL with the driver `disk`, time stretching off, the volume at 100 and the output
unmuted, and the two variables for the run alone; PCSX2 then writes 32-bit float stereo at 48 kHz, which ×32,768 gives
the integers. Run at the console's speed (`--throttled`). Its spu2 debug options (a wave log among them) write nothing in
this release build. The captured sound is PCSX2's output stage's: a constant level decays by about 0.5 % a sample (a
high-pass), and a voice at unity through both cores comes out at about half (D-270). It showed which MMIX bits pass the
external input, AVOL's and MVOL's scale, ATTR's top bits doing nothing and AutoDMA's stream heard (D-262, D-267); the
samples themselves are compared through the output areas.

**What PCSX2 is a witness for**: the register map (D-260, D-266), the output areas (D-261), the mix and its gates
(D-262), the transfers (D-263), the effects area's place and enable (D-264), the IRQ addresses (D-265), AutoDMA's
stream (D-267). **Not**: the voice's last bits and its key on, where it reads psx-spx otherwise than Diamond's three
PlayStation lineages (D-269); MEMIN in RAM, which it does not keep (D-267); the reverb's last bits and its accesses
while off (D-264, D-265). Every ruling on it names it as the single lineage.

### 32.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | ADPCM filter 1's weight 60 made 61 (`blackmoon-spu`'s `tables.rs`) | the crate's `each_filter_predicts_from_psx_spx_weights`; Diamond's `the_capture_program_records_the_voices_as_the_referees_do`, `the_voice_scenes_play_to_their_end` and `the_projects_programs_give_their_goldens`. **Not** Sapphire's gate, whose record of a voice is the same crate; the referee would see it |
| F2 | core 0's output not routed into core 1 | `core_0s_output_reaches_the_output_through_core_1` |
| F3 | AutoDMA playing the other half of MEMIN | `s26-adma`'s two stream checks, once the copy waits about 60 samples after the transfer and the record ends before the fourth block; the first form of the check, a window of all four blocks copied at once, missed it, since the half not yet due is written at once and still sounds in order |

Each was applied, the tests named run, and the source restored [M].

### 32.10 What is not modelled

Each a named loss, and a dispute candidate if an oracle measures it [A]:

- The transfer FIFO and its timing; ATTR's transfer mode bits; a read through the data port (zero); AutoDMA reads
  (ps2tek's bit 2, "Unknown how ADMA reads work").
- The S/PDIF registers from 7C0h but the IRQ flags, kept as written; the SPU2 in the PlayStation's mode (44.1 kHz,
  Diamond's).
- A voice started off a block's boundary (D-268).
- The SPU2's state in `SPH2` (S30) and its view in the debugger (S31).
- **The sound reaching the player**: the machine does not yet run the board (S10, S24), so `v1`'s audio stays the
  stub's silence at 48,000 Hz; the SPU2's queue is there for the machine to drain when it does.

**Speed** [M, a timing run outside the tree, two runs, other work on the machine]: 48 voices keyed on looping blocks,
every mask set and both reverbs writing, 0.556–0.576 ms a 60 Hz frame of 800 samples on the desktop, inside the plan's
§5.12 estimate of 0.4–0.9 ms; nothing allocates a sample.

### 32.11 What the plan got wrong, or left open

- **§2's SPU2 row and §9.3** assumed ps2tek's map would serve; it gives the rows above and places key on wrongly
  (D-260). The plan did not foresee that the referee's sound would be read from sound RAM (D-261) rather than from its
  audio, whose output stage filters it (D-270).
- **§9.3's gate** ("Diamond's SPU goldens, every sample, unchanged") is met, with the bench discs' sound digests among
  them, and Diamond's tests unchanged; the GPU tests were not run at this stage.
- **§3.2's "the project's voice programs against referees' sound"**: met through the output areas exactly where PCSX2
  and the shared voice agree, and through the captured sound for the final output's routing; where they disagree it is
  the voice's rules, kept by D-269.

### 32.12 Questions this stage leaves

1. **Q1 [decision], the voice's rules on the PlayStation 2 (D-269).** PCSX2 reads psx-spx otherwise than Diamond's
   referees at a key on, in the interpolation's and the prediction's rounding, the exponential attack, rate 7Fh and the
   slow noise clock. *Recommended*: keep the shared voice, one set of rules for both consoles, and listen at S24 (the
   BIOS's menu sounds) and S28 (games) through the output areas; if the PlayStation 2's own evidence ever departs, the
   crate takes a per-console rule rather than forking.
2. **Q2 [routine], ATTR's bits 15 and 14 (D-262).** *Recommended*: keep them inert, as the referee shows; the BIOS at
   S24 shows whether any sound depends on them.
3. **Q3 [routine], MEMIN in RAM (D-267).** *Recommended*: keep ps2tek's layout in RAM, which costs nothing and which the
   output areas and IRQ addresses can see; PCSX2's omission is its own.
4. **Q4 [routine], the output's level (D-270).** *Recommended*: hand core 1's output to the host unscaled until a
   document or a measurement says otherwise.

### 32.13 What was read for this stage

`Sapphire_Plan.md` §2 (the SPU2 row), §5.5, §7 (S26), §9.3, §11 Q10; this page's §22 (S21: the IOP bus and its DMA);
`Sapphire_Disputes.md`'s rules and D-0; `Sapphire_Referees.md` §2.1 and §4.3; `Diamond_SPU.md` §1–§9 and Diamond's
`src/spu` whole (the project's own code, moved); `BlackMoon_R3000.md` §2 and BlackMoon's `README.md`. ps2tek at
`1c91660`: "Memory Map", "I/O Maps"' SPU2 rows, "SPU2 AutoDMA", "IOP DMA", "IOP Interrupts", and the two lines of the
BIOS chapter on the PlayStation mode's rates (5100–5101, cleared by S2's screening). psx-spx at `6d7d1bc` through
`Diamond_SPU.md`'s citations. PCSX2's settings file and the names of its SDL library's audio hints. A web search for a
public SPU2 register map found none outside an emulator's source or the SDK, so none was read. No emulator, SDK,
loader or ps2autotests source and no firmware; the BIOS was only passed to PCSX2 by path. **Exposure**: the voices
PCSX2's BIOS leaves on its block at 2800h showed in two read-backs (their NAX and ENDX); nothing of the BIOS's code or
data was read, and the block's eight halfwords were counted, not printed.

### 32.14 What this stage does not cover

- The BIOS's sound driver and the IOP modules that drive the SPU2 (S24); games' sound (S28).
- The SPU2's state, debugger view and speed levers (S30, S31, S32 on).

## 33. Stage S15, the GIF, its three paths and their arbitration (2026-10-25)

*Numbered §33 and its disputes from D-280, since S25 (§30, from D-220), S18 (§31, from D-240) and S26 (§32, from
D-260) are built beside it.*

### 33.1 In one paragraph

The plan's §7 row S15, on the tester's go of 2026-10-25. The crate gains **`gif`**, the GIF written from the *EE User's
Manual* (EEUM) chapter 7, with ps2tek's "Graphics Interface (GIF)" beside it: GS packets taken from **PATH1** (VU1's
XGKICK, through `gif::Path1Port` in place of S13's stand-in), **PATH2** (VIF1's DIRECT and DIRECTHL, through S14's
`vif::Gif`) and **PATH3** (the DMAC's channel 2 and the EE's stores at 10006000h, through a FIFO of 16 qwords), and
written to S16's `gs::HostInterface` in the PACKED, REGLIST and IMAGE formats by **`gif::packing`**, the one owner of
the GIFtag and of every register descriptor's packing. One path holds the GIF from its packet's first GIFtag to the end
of its EOP primitive; between packets the waiting paths are arbitrated PATH1, PATH2, PATH3 with an idle cycle (§7.1.2);
PATH3 is skipped while GIF_MODE's M3R or MSKPATH3 masks it, its data kept in the FIFO and the DMA held once the FIFO is
full (§7.1.4); in the intermittent mode PATH3's IMAGE data gives way to PATH1 and DIRECT at each slice of 8 qwords, not
to DIRECTHL (§7.1.3); GIF_CTRL resets and pauses, GIF_STAT and the tag and count registers report. S11's stand-in at
channel 2 and S14's PATH2 stand-in are gone, and **S16's, S17's and S18's programs now run whole on Sapphire through
PATH3 and VIF1**, every one of their 473,156 graded words equal. Six programs of the project's (6,728 graded words) are equal on
the interpreter to records `tests/s15/manual.rs` gives from chapter 7 over S17's model of the GS; PCSX2 equals the
REGLIST and IMAGE program whole, the PACKED program but two descriptors (D-281), the masks, the FIFO's depth and the
held DMA, TTE's tag (D-283), PATH1's wrap and DIRECT overtaking a running IMAGE transfer in the intermittent mode, and
departs on a path holding the GIF when its data stops (D-280), where Play! holds as the manual does; ps2autotests'
`dma/vif/mskpath3` text is reproduced by PCSX2 and by the GIF directly. **S14's second DIRECT that PCSX2 never takes
is a referee defect**: PCSX2, entered by fast boot without a GIF reset, grants PATH3 after the first PATH2 packet
although no PATH3 data or channel exists, and that PATH3 then holds the GIF (D-290). Disputes D-280 to D-291. No
emulator, SDK, loader or ps2autotests source and no firmware was read.

### 33.2 What was built

- **`src/gif/packing.rs`**: `Tag` (§7.2.2's fields, NREG 0 as 16, FLG 11 as IMAGE), `Packet` (one path's place in its
  GS packet: the primitive open, LOOPCNT, the descriptor next, the data qwords taken), `Packet::take` (a GIFtag, or one
  qword of data written out: PRE's PRIM, each PACKED descriptor's packing, REGLIST's halves with A+D and NOP skipped and
  an odd count's last half dropped, IMAGE's two halves to HWREG), the cost of each in bus cycles (§33.4), and
  `cycles`, a packet's cost read from memory, which PATH1's estimate uses.
- **`src/gif/mod.rs`**: `Gif`, the unit: its clock, the path holding it, a `Packet` a path, Q, GIF_MODE, PSE, the
  MSKPATH3 mask, PATH3's interruption with P3CNT and P3TAG, the last GIFtag, PATH1's VU address, the XGKICKs waiting,
  PATH2's request, the FIFO; `run_to` (arbitration, PATH1's qwords read from VU1's memory as they are sent, PATH3's
  from the FIFO), `path2`, `idle_at` and `mask_path3` (behind `vif::Gif`), `request` and `put` (behind
  `dmac::Peripherals`), `kick` and `path1_free`, the registers. `Path1Port`, PATH1 as VU1 reaches it (`vu::Path1`).
- **`src/vu/mod.rs`**: `vu::Path1` gains two methods with defaults, additive, `set_free` (the board's estimate of when a
  PATH1 transfer could start) and `take_kick` (a kick handed on to the GIF); `Path1StandIn` and its test are unchanged.
- **`src/vif/mod.rs`**: S14's `Path2StandIn` removed, `vif::Gif` itself unchanged.
- **`src/board.rs`**: the GIF on the board; `Ports` gives it channel 2; `GifPort`, the GIF as VIF1 reaches it, in
  place of the stand-in; the GIF's registers at 10003000h and FIFO at 10006000h (§33.5); `run_gif`, brought before the
  VIFs, before the DMAC acts, after the VUs run and around the EE's reaches of the GS's privileged registers.
- **No new cross-component dependency**: `sapphire` still depends on `emusen-native`, `emusen-mips`, `blackmoon-r3000`
  and `emusen-disc` alone; `src/gs/` is untouched (S18 works in it).

### 33.3 The readings the manual leaves open

[R EEUM §6.4's MSKPATH3, FLUSH, FLUSHA, DIRECT and DIRECTHL pages, chapter 7 whole; VU User's Manual §3.4.9 and
XGKICK's page; GS User's Manual §4.2.2 and §1.1.2's transfer speed; ps2tek "Graphics Interface (GIF)", "DMAC I/O"; M
the referees' records of §33.7; A where marked]:

| Rule | Source | Implemented |
|---|---|---|
| A path holds the GIF from its first GIFtag to its EOP primitive's end, though its data stops meanwhile | §7.1.2, §7.2.1 ("Transfer processing is performed in units of GS packets") | D-280 |
| Arbitration when the GIF is free: the earliest request decides when, the priority PATH1, PATH2, PATH3 which; one idle bus cycle | §7.1.2 | `Gif::choose`, `grant` |
| NLOOP 0: nothing output, the EOP kept; Q set to 1.0 at every GIFtag | §7.2.2; §7.3.2's RGBAQ | `Packet::take` |
| PACKED: PRIM 10..0; RGBAQ R, G, B, A from bits 7..0 of each word with the GIF's Q; ST the lower 64 bits, Q from 95..64; UV 13..0 and 45..32; XYZF2 X 15..0, Y 47..32, Z 91..68, F 107..100, ADC 111 to XYZF3; XYZ2 Z 95..64, ADC to XYZ3; FOG 107..100; A+D's address 71..64 | §7.3.2's figures; ps2tek agreeing | `packing::packed` |
| TEX0, CLAMP and the descriptors 0Ch and 0Dh write the lower 64 bits; 0Bh likewise (to an address the GS drops) | §7.2.3's table; ps2tek's "all registers not specified" | D-281 |
| Q is the GIF's, given with RGBAQ, not written to the GS by ST | §7.3.2; ps2tek reads ST as writing Q to the GS | D-291 |
| REGLIST: PRE ignored, A+D and NOP not output, an odd count's last half dropped | §7.4.1 | `Packet::take` |
| IMAGE (and FLG 11): NLOOP qwords, each two writes of HWREG | §7.5 | `Packet::take` |
| PATH3 masked by M3R or M3P only at the arbitration, so a packet in progress finishes | §7.1.4; GIF_MODE's page | D-289 |
| The FIFO takes PATH3's data while masked; the DMA holds once it is full | ps2tek "GIF PATH3 Masking"; PCSX2 | `Gif::request` |
| A DMAtag sent by TTE is not GIF data | D-283 | `Gif::put` |
| RST: the packets, the FIFO, the kicks, the interruption and GIF_MODE cleared; MSKPATH3's mask kept | GIF_CTRL's page; §7.1.4 | D-284 |
| PSE: nothing moves; the paths' data waits | GIF_CTRL's page | `Gif::run_to` |
| The intermittent mode's slice: 8 qwords of the IMAGE primitive's data; a request seen while PATH3 waits at a slice's end taken at once; DIRECTHL never overtakes, and waits behind an interrupted PATH3 | §7.1.3; DIRECTHL's page | D-285 |
| XGKICK: the packet read from VU1's memory as it is sent, its address wrapping at 1,024 qwords | §7.1.1; GIF_CNT's VUADDR | D-288 |
| FLUSH and MSCALF wait for PATH1 and PATH2 idle and FLUSHA also for no PATH3 request: data in the FIFO, a packet in progress or interrupted, or channel 2's STR | §6.4's pages | `Gif::idle_at` |
| GIF_STAT: P1Q a kick waiting, P2Q PATH2 refused and waiting, P3Q PATH3's data or interruption waiting while another path or none holds the GIF; OPH and APATH the path holding the GIF or whose last output is still being sent; DIR BUSDIR's | §7.6 | D-282 |
| GIF_TAG0 to GIF_TAG3, GIF_CNT, P3CNT and P3TAG answered at any time, the write-only GIF_CTRL and GIF_MODE reading 0 | §7.6 ("accessible only when stopped") [A] | `Gif::read` |
| While BUSDIR reverses the host interface the GIF sends nothing | §7.1.5 [A] | D-287 |
| The FIFO's 16 qwords mirrored over 10006000h to 10006FFFh; an EE store into it taken even when full | §2.2.1's map [A] | `Board::write` |

### 33.4 The timings: the documents' and the argued

**The documents'** [R]: one idle cycle at each arbitration and one when PRIM is not output (§7.1.2, §7.3.1, §7.4.1,
§7.5.1); the paths' inputs 128 bits wide at BUSCLK, the output to the GS 64 bits (§7.1's figure); the GS's host
transfers at most 1.2 GB/s (GS User's Manual §1.1.2), 64 bits a bus cycle. **Argued**, each a named loss (D-286): a
GIFtag costs one bus cycle (PRIM output or the idle one), a PACKED qword one, a REGLIST or IMAGE qword two (one for
REGLIST's last odd half), the arbitration one; the qwords of one DMA unit, which the DMAC hands over together at its end
(§20.3), arrive one a bus cycle up to it; the end XGKICK is told is estimated when it is kicked, from the packet in
VU1's memory and the GIF's state as the board last gave it (behind the kicks waiting, the PATH1 packet being sent, or
the primitive another path is sending), so a kick queued behind a PATH3 packet whose data comes later is told too early;
a request is noticed when the board next brings the GIF forward, at the instruction's granularity of §5.2. **Measured**
[M, `the_fifo_holds_sixteen_qwords_and_the_gif_paces_the_dma`]: 200 qwords to the GIF take 400 EE cycles as a PACKED
packet (the DMAC's pace) and 738 as an IMAGE one (the GIF's, the last 16 in the FIFO).

### 33.5 The interfaces and the board

- **`dmac::Peripherals`** on channel 2: `request` answers the FIFO's room, whatever the masks, a zero holding the
  channel until the GIF frees a slot and the board wakes the DMAC; `put` takes a qword (a TTE tag dropped).
- **`vif::Gif`** (S14's) through `GifPort`: `path2` is `Gif::path2`, `idle_at` `Gif::idle_at` with channel 2's STR as
  PATH3's request, `mask_path3` M3P, `host_read` the GS's local-to-host data as before.
- **`vu::Path1`** through `Path1Port`, installed in the VPU's link by `Board::new`: the kick recorded and its end
  estimated; the board sets the estimate before the VUs run and hands the kicks to the GIF after.
- **`gs::HostInterface`** (S16's): every write and HWREG's image data. The GS's own SIGNAL stall keeps holding what
  arrives behind it, unbounded, as S16 built it; the GIF does not stop for it (D-287).
- **The board** brings the GIF forward before the VIFs (so PATH2 meets it at the VIFs' time), before the DMAC acts at
  each event (so the FIFO's room is current), after the VUs run (so a kick starts at once), and around the EE's reaches
  of the GIF's registers and FIFO and of the GS's privileged registers (so CSR, SIGLBLID and BUSDIR see every write the
  GIF has made). It schedules an event only while the DMAC waits on the FIFO and the GIF can free a slot.

### 33.6 The gate: the programs (measured 2026-10-25)

`tests/s15/` writes six bare programs in S14's form (kseg0 code, everything else through kseg1), each starting with S14's
prologue, the GIF reset with GIF_MODE 0, MSKPATH3 cleared and the kick microprogram loaded by VIF1, CSR's events
cleared, and S17's drawing environment into a 64×32 PSMCT32 frame at page 100 cleared by a sprite. Packets are built in
memory and sent by normal or chain DMA, stored into the GIF's FIFO, sent as DIRECT or DIRECTHL by VIF1's channel, or
written into VU1's memory and kicked by a microprogram that VU1 starts at CMSAR1's write and that reads the packet's
address from qword 200h. The record is local memory read back local to host (S16's route), SIGLBLID's LBLID after
LABELs, and the GIF's and VIF1's registers. Each program states the order its packets reach the GS by §7.1, and
`tests/s15/manual.rs`, chapter 7 transcribed apart from `src/gif/` (each descriptor read at the bit positions its figure
prints), turns them into writes for S17's model.

| Program | Graded words | What it covers |
|---|---|---|
| `s15-packed` | 2,308 | PRE's PRIM; RGBAQ, XYZ2 and XYZF2 with junk in every bit the packing drops; XYZF2's Z into a Z buffer; ADC sending a vertex to XYZF3 (no kick); PRIM as data with PRE 0; A+D with junk above bit 71; NOP; sixteen descriptors (NREG 0); NLOOP 0 inside a packet and ending one, a second packet in the same DMA; 0Ch and 0Dh as raw vertices; ST, UV, FOG, TEX0, CLAMP and 0Bh; the frame and a Z rectangle read back |
| `s15-reglist` | 2,052 | REGLIST under PRE (ignored), an odd count's dropped half shown by the next sprite's colour, A+D and NOP skipped; IMAGE with FLG 2 and 3 in one packet, and with a PACKED primitive between two IMAGE ones |
| `s15-paths` | 2,064 | the same kind of packet by normal DMA, chain DMA without and with TTE, the EE's stores into the FIFO, DIRECT, DIRECTHL, XGKICK, and XGKICK across VU1's memory's end, each into its own region |
| `s15-arbitration` | 232 | PATH2 waiting for PATH3's packet held mid-way; PATH1 queued behind it without stalling VU1; PATH1 before PATH2 when both wait; MSKPATH3 holding a short packet in the FIFO while PATH2 goes and a long one with the DMA held at 16 qwords (QWC 8 left); M3R set during a packet and holding the next; the intermittent mode's DIRECT and XGKICK interrupting PATH3's IMAGE data at a slice's end (P3CNT 24, P3TAG 8020h under PSE), DIRECTHL and the continuous mode not; GIF_STAT and VIF1's VGW at each step, and the upload read back |
| `s15-regs` | 48 of 52 | GIF_STAT at rest and after GIF_MODE's writes; PSE mid-packet with GIF_TAG0 to GIF_TAG3 and GIF_CNT (LOOPCNT 2, REGCNT 2), and before a packet; RST ending a packet so PATH2 goes at once, clearing GIF_MODE and keeping M3P; DIR with BUSDIR; the EE's stores into the FIFO while M3R holds it |
| `s15-intermittent` | 24 of 32 | an IMAGE packet of 30,720 qwords started by DMA and at once a LABEL by DIRECT, DIRECTHL or XGKICK: SIGLBLID and D2_CHCR.STR as soon as the other path has its packet, and again after both |

**S16's, S17's and S18's programs through the real path**: every program of §24.6, §27.6 and §31, until now graded on
Sapphire by driving the host interface directly, runs whole as the EE program the referees ran, its packets through
channel 2 and the GIF and its local-to-host data through VIF1: **all 51,268, 225,280 and 196,608 graded words equal**,
S18's on the tree with S18 and S25 beneath after the rebase.

**Tests** [M, `cargo test --release`]: `tests/gif.rs` **15 of 15** — the six programs and the S16, S17 and S18 programs,
each equal to its record word for word; `every_packing_conversion_writes_what_section_7_3_draws` (2,000 generated packets
in every format, every descriptor with random bits, against the manual's transcription, the fog byte and Q among the
writes no picture shows); the FIFO's depth and the GIF's pace (§33.4); `Path1Port`'s estimates and a packet with no EOP;
the arbitration on the unit directly with GIF_MODE's mask; the `mskpath3` console text (§33.7); the second-DIRECT
probe's four forms (§33.8). `tests/vif.rs` **12 of 12** with PATH2 the GIF's: `s14-direct`'s three LABELs now reach
SIGLBLID in turn on Sapphire. The crate's other tests unchanged, on the tree with S18 and S25 beneath: lib 52, `cop0` 19,
`dmac` 18, `draw` 13, `fpu` 9, `gs` 12, `interpreter` 23, `iop` 13, `micro` 12, `mmi` 13, `pcrtc` 5, `programs` 3, `sif`
5, `sio2` 5, `texture` 11, `vu0` 14, `vu_micro` 24; lib, `gif`, `vif` and `dmac` also under `test-checked`; clippy
clean.

### 33.7 On the referees (measured 2026-10-25)

`examples/s15.rs` writes each program as an ELF with its expected record; `ps2referee.py ram … --until 1FFFF0=5A99D00E
200000:<words>` read each record from PCSX2 2.8.2 (Americas BIOS, fast boot, software renderer) and Play! 0.72, one
process at a time; `s15 explain` lists the records that differ. Only the records were read.

| Program (records) | PCSX2 | Play! | What the differences are |
|---|---|---|---|
| `s15-packed` (1,154) | 64 | 1,024 | PCSX2: the two sprites whose first vertex came by descriptor 0Dh and 0Ch, which it packs as XYZ and XYZF (D-281); every other descriptor, junk bit, ADC, NLOOP-0 tag, NREG 0 and the Z buffer equal. Play!: no drawing reaches its local memory (§27.7); both LABELs equal |
| `s15-reglist` (1,026) | **0** | 896 | Play!: the IMAGE rectangles equal, the drawing absent |
| `s15-paths` (1,032) | 2 | 1,024 | PCSX2: PATH1's LABEL read a step late, after VU1's end and GIF_STAT's idle; the frame equal, TTE's region (D-283) and the wrap across VU1's memory (D-288) among it. Play!: every LABEL equal, the drawing absent |
| `s15-arbitration` (116) | 27 | 94 | below |
| `s15-regs` (26) | 10 | 20 | below |
| `s15-intermittent` (16) | 5 | 6 | below |

**The arbitration and the registers.** PCSX2 lets PATH2 and PATH1 take the GIF while PATH3's packet waits for data
after its DMA ended, so the held-packet cases run in another order (D-280); **Play! holds PATH2 behind PATH3's packet
as the manual does**, in `s15-arbitration`'s first case and the intermittent cases with DIRECTHL and the continuous mode,
while delivering XGKICK at once. Both hold PATH3 while MSKPATH3 or M3R masks it, let PATH2 go meanwhile and send PATH3
once unmasked, and finish a packet M3R was set during and hold the next (D-289); PCSX2 takes 16 qwords into the FIFO and
holds the DMA with QWC 8, as Sapphire does, Play! holds the DMA without taking any (QWC 24). PCSX2's GIF_STAT shows M3R,
M3P, IMT, PSE, DIR and FQC as Sapphire does, never P1Q, P2Q or P3Q, OPH and APATH only while data moves (and OPH while
BUSDIR is reversed and after, D-287), and reads 0 from GIF_TAG0 to GIF_TAG3, GIF_CNT, P3CNT and P3TAG (D-282); it pauses
a packet for PSE, clears M3P on RST (D-284), and, in the intermittent mode, lets DIRECT overtake an IMAGE transfer whose
DMA still runs (`s15-intermittent`'s first case: the LABEL with STR set, D-285). Its other intermittent cases finish the
30,720-qword DMA before the other path is given its packet, so they decide nothing. Play!'s GIF_STAT reads E00h or 200h
whatever the state, it neither pauses nor resets a packet, and it has no intermittent mode.

**ps2autotests' `dma/vif/mskpath3`** (5 lines) needs the BIOS's kernel, so Sapphire runs it with S10. Run through
`ps2referee.py tty pcsx2 … --channel iop` against its expected text, the console's output, after line ends are
normalised [M `diff`]: **equal**. Its three values follow from GIF_STAT read whole with M3P its only bit, which
`the_mskpath3_console_text_follows_from_gif_stat` reproduces on the board through VIF1's FIFO [A: the program's source
is not read].

So **PCSX2 is a witness for the formats (but 0Ch and 0Dh), the masks with the FIFO and the held DMA, TTE, PATH1's wrap,
GIF_MODE's and GIF_STAT's mask bits and DIRECT overtaking IMAGE in the intermittent mode**; not for a packet holding the
GIF, the queue bits, the tag and count registers or PATH1's timing. Play! is a witness for PATH2 waiting on a held packet,
the masks and IMAGE's data.

### 33.8 S14's second DIRECT on PCSX2 (measured 2026-10-25)

`s14-direct` (§28.8) found PCSX2 taking its first DIRECT and never its second, VIF1 left with VGW. `s15::second_direct`
sends two DIRECT packets of a LABEL each in four forms, recording VIF1's CODE and STAT, GIF_STAT and SIGLBLID after
each (and GIF_STAT and D2_CHCR at the start, ungraded): (a) as `s14-direct` sent them, with no GIF reset; (b) with
GIF_CTRL.RST first; (c) with the second stream DIRECT alone; (d) with a PATH3 packet between the two.

| Form | PCSX2 | Play! |
|---|---|---|
| (a) | second DIRECT never taken: VGW, FQC 3, SIGLBLID kept; GIF_STAT 0 at the start, **E00h (OPH, APATH = PATH3) after the first DIRECT**, E80h after the second; D2_CHCR 0 throughout | both taken |
| (b) | both taken; GIF_STAT 0 after the first | both taken |
| (c) | as (a) | both taken |
| (d) | both taken | both taken |

So PCSX2, entered by fast boot, holds a PATH3 request that GIF_STAT does not show and that waits through an idle GIF,
is granted at the end of the first PATH2 packet, and then holds the GIF with no data in the FIFO and channel 2 stopped;
a GIF reset clears it and a PATH3 packet ends it. Under §7.1.2 an idle GIF grants a waiting request at once, so a request
that waits through an idle GIF has no reading in the manual, and PATH3 has nothing to send. **It is a referee defect,
not a protocol rule** (D-290): a program that resets the GIF first, as every S15 program does, has every DIRECT taken
on PCSX2 (`s15-arbitration` sends seven); Play! takes both in every form.

### 33.9 Named losses

- **The GIF's timing** (§33.4, D-286): argued throughout; XGKICK's end estimated; requests noticed at the instruction's
  granularity, so a path asking within an instruction of another's packet ending may meet the other order.
- **PATH1's data** is read from VU1's memory as the GIF sends it, with VU1 brought to the board's time first; a
  microprogram writing its own packet during the transfer meets what hardware would at a time the model approximates.
- **The GS's host FIFO** is unbounded behind a stalled SIGNAL (S16's), so the GIF never waits for the GS (D-287).
- **ps2tek's warning** that A+D on PATH3 while PATH1 runs may lock up the GS is not modelled.
- **The FIFO's** reads by the EE answer 0.

### 33.10 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | PACKED XYZF2 dropping the fog byte (`packing::packed`) | `every_packing_conversion_writes_what_section_7_3_draws` |
| F2 | PATH3 not masked by MSKPATH3 (`Gif::masked` reading M3R alone) | `the_paths_are_arbitrated_at_packet_ends_and_path3_masked_and_interrupted_as_section_7_1_says` |
| F3 | IMAGE's NLOOP counted in doublewords (two loops a qword) | the packing test, the FIFO's test, `s15-reglist`, `s15-arbitration`, S16's and S17's programs through PATH3 (before S18's joined them) |

Each was applied, `tests/gif.rs` and `tests/vif.rs` run, and the source restored [M]. No picture shows the fog byte, so
F1 has one witness, as expected; `tests/vif.rs` sees none of the three, its DIRECT packets having no fog, mask or IMAGE.

### 33.11 The blast radius (measured 2026-10-25)

`blast.sh WiseMan sapphire-s15` names the Sapphire crate and WiseMan's `SapphireTests`: the crate as §33.6 gives it, and
WiseMan's filter **5 of 5** before the rebase onto S18's and S25's merges and again after, each run once at CPU weight
20, `LunaP.props` pointed at the sibling LunaP checkout by a local change that is not committed.

### 33.12 What the plan got wrong, or left open

- **§1.1's GIF row** grades arbitration "as `dma/vif/mskpath3` records it": that text records only GIF_STAT's M3P bit
  before, during and after MSKPATH3 (§33.7); the arbitration rests on the manual, the project's programs and the
  referees, which split on a packet holding the GIF (D-280).
- **§3.2's row** names the referees as the secondary oracle: PCSX2 is one for the formats and the masks, Play! for the
  order of a held PATH2; neither for the registers chapter 7 marks readable only under PSE.
- **S15's gate** asks for drawings "cross-checked on PCSX2 and Play! where it reaches": Play!'s drawing never reaches
  its local memory (§27.7), so the drawings rest on PCSX2 and the LABELs on both.

### 33.13 Questions this stage leaves

1. **Q1 [routine], a path holding the GIF when its data stops (D-280).** *Recommended*: keep the manual's packet-level
   arbitration, which Play! follows for PATH2 and PCSX2 does not; S28's games, some of which split a packet across
   transfers, would show a difference.
2. **Q2 [routine], descriptors 0Ch and 0Dh (D-281).** *Recommended*: keep the lower 64 bits, as both documents give
   them, against PCSX2's packing.
3. **Q3 [routine], RST and MSKPATH3's mask (D-284).** *Recommended*: keep the mask, §7.1.4's "can be cancelled only by
   the VIF", against PCSX2.
4. **Q4 [routine], the GIF's timing (D-286).** *Recommended*: keep a bus cycle a 64-bit output until S32's bench shows a
   game whose timing depends on it.
5. **Q5 [routine], Q from ST (D-291).** *Recommended*: keep §7.3.2's internal Q given with RGBAQ; S18's texturing
   programs can decide it against ps2tek's reading, which writes Q to the GS at ST.

### 33.14 What was read for this stage

`Sapphire_Plan.md` §1.1's GIF and VIF rows, §1.5, §2, §3.2, §7's S14 to S17 rows and §11; this page's §20, §23, §24,
§27 and §28; `Sapphire_Disputes.md`. Sony's *EE User's Manual* (Version 5.0) chapter 7 whole, §2.2.1's map, §5.2.3 and
CHCR's TTE, chapter 6's MSKPATH3, FLUSH, FLUSHA, MSCALF, DIRECT and DIRECTHL pages; the *VU User's Manual* §3.4.9 and
XGKICK's page; the *GS User's Manual* §1.1.2 and §4.2; the *EE Overview* §2.5. ps2tek at `1c91660`: "Graphics
Interface (GIF)" with its I/O, GIFtags, data formats and PATH3 masking, "DMAC I/O". ps2autotests' `dma/vif/mskpath3`
expected text (the console's) and its `.elf` run as a black box on PCSX2. No emulator, SDK, loader or ps2autotests
source and no firmware.

### 33.15 What this stage does not cover

- `dma/vif/mskpath3` on Sapphire: S10 (the kernel).
- The GS's own timing and its host FIFO's depth (S43); texturing's use of Q (S18).
- The GIF's state in `SPH2` with the rest (S10, S30).

## 34. Stage S27, the IPU (2026-10-25)

*Numbered §34 and its disputes from D-300, since S25 (§30, from D-220), S26 (§32, from D-260) and S15 (§33, from D-280)
are built beside it.*

### 34.1 In one paragraph

The plan's §7 row S27, on the tester's go of 2026-10-25. The crate gains **`ipu`**, written from the *EE User's
Manual* (EEUM) chapter 8 with ps2tek's "Image Processing Unit" where the manual is silent, and from ITU-T H.262
(02/2000), the twin of ISO/IEC 13818-2, for the decoding: the ten commands, IPU_CMD, IPU_CTRL, IPU_BP and IPU_TOP with
their status and error bits, INT_IPU, the input FIFO with its internal buffer and the output FIFO, and the DMAC's
channels 3 and 4 in place of S11's stand-ins. IDEC and BDEC decode MPEG-2 macroblocks, and MPEG-1's with IPU_CTRL.MP1,
through Annex B's tables, the inverse scan, inverse quantisation, saturation and mismatch control of §7.3 and §7.4, and
Annex A's reference IDCT, evaluated in double precision and decided exactly near every half; IDEC, CSC and PACK convert
through §8.6's arithmetic, its dither and its vector quantisation. The gate's corpus is twelve streams of the project's
own making, encoded by ffmpeg run as a black box, and two written symbol by symbol for the cases no encoder writes; a
decoder of H.262 written apart from `src/ipu/` decodes them, ffmpeg's own decoding agreeing with it to within one
step in 1,983 of 147,456 samples, the two IDCTs' rounding. Driven through the stream by its commands as an FMV player
would drive it, **Sapphire's IPU gives that decoder's every block, picture and pointer**, and 32 replay programs and a
register program (163,964 graded record words) give the same on the interpreter with the board. On PCSX2 and Play!,
run as black boxes, the referees' IDCTs depart from the reference by one step (PCSX2) and two (Play!), as P7 predicted,
and each lineage departs elsewhere in ways two hypotheses each reproduce word for word; the disagreements are disputes
D-300 to D-326. No emulator, decoder, SDK or ps2autotests source and no firmware was read.

### 34.2 What was built

- **`src/ipu/vlc.rs`**: Annex B's Tables B.1 to B.4 and B.9 to B.15 as the standard prints them (codes as text,
  extracted from the standard's text and checked to be prefix-free and to leave exactly the code space the standard
  leaves, `every_table_is_prefix_free_and_as_complete_as_the_standard`), ISO/IEC 11172-2's stuffing and D-picture codes
  where H.262 §D.9 gives them, and a decoder that groups each table's codes by their leading zeros and indexes the bits
  after the first one; the tables are built once.
- **`src/ipu/bits.rs`**: the input, the FIFO's eight qwords behind the internal buffer's two, read as one stream from
  BP, each byte's highest bit first (H.262's bslbf order); a qword enters the internal buffer when a command reads it
  (D-302).
- **`src/ipu/idct.rs`**: Annex A's saturated mathematical integer-number IDCT (§34.5).
- **`src/ipu/csc.rs`**: §8.6.1 to §8.6.3 in the manual's order and precision, and the macroblock layouts of §8.2.
- **`src/ipu/mod.rs`**: the unit. A command runs symbol by symbol, each symbol waiting for the 32 bits VDEC's and
  FDEC's notes ask to be present; IDEC and BDEC keep a macroblock's state (its modes, its block, the coefficient index,
  the six transformed blocks) between symbols, and their output waits in a buffer of up to one macroblock for room in
  the output FIFO. BUSY clears when the command's last output is in the FIFO, INT_IPU with it (D-310).
- **`src/board.rs`**: the IPU on the board, its registers at 10002000h to 1000203Fh and FIFOs at 10007000h and 10007010h,
  channel 3 taking from the output FIFO and channel 4 giving to the input FIFO through `dmac::Peripherals`, the DMAC
  woken whenever a FIFO's count changes, INT_IPU latched by the INTC. Additive: a field, two match arms each in `Ports`,
  the bus's two ranges, `sync_ipu`.
- **No new cross-component dependency**: `sapphire` still depends on `emusen-native`, `emusen-mips`, `blackmoon-r3000`
  and `emusen-disc` alone. ffmpeg is a tool of the corpus's making, not a dependency.

### 34.3 Sources, and what each gave

- **ITU-T H.262 (02/2000)**, free from the ITU and pinned at S2 (`Sapphire_References.md` §1): §6.2 and §6.3 (the
  syntax the model and the driver walk), §7.1 to §7.6 (variable-length decoding, the inverse scan, inverse
  quantisation, saturation, mismatch control, the IDCT's place, motion compensation for the model), Annex A (the
  reference IDCT and its accuracy rules), Annex B (every table), §D.9 (MPEG-1's differences: its oddification, its
  escape lengths, stuffing, D-pictures). ISO/IEC 11172-2 itself is not lawfully available to the project; MPEG-1's
  arithmetic is §D.9's description, argued where it is short (D-307, D-326) and witnessed by ffmpeg's decoding.
- **EEUM chapter 8** (Version 5.0, the Linux kit's, Q1): the commands and their fields, the registers, the FIFOs'
  depth, the data formats, the symbols IDEC and BDEC decode, VDEC's tables with their output (§8.5.3, transcribed whole
  into `manual.rs` and graded row by row, `vdec_gives_every_row_of_the_manuals_tables`), §8.6's arithmetic.
- **ps2tek** at `1c91660`, "Image Processing Unit": BCLR's BP width (D-312), IPU_CMD's value before a command and
  IPU_TOP's BUSY below 32 bits (D-311), RST's interrupt (D-315).

### 34.4 The input, the pointer and the commands' readings

[R EEUM §8.3, §8.4; ps2tek; A; M where a referee measured]:

- **The internal buffer** (D-302): IPU_BP.FP counts the qwords a command's reads have reached, at most two, IFC the
  rest; the FIFO takes eight, a ninth is refused. Both referees keep pushed qwords in the FIFO until a command reads.
- **FB** takes its bits whole, 0 to 63 (D-313); BCLR's BP is 7 bits (D-312); BCLR acts even while a command waits,
  ending it, and any other command written meanwhile is ignored (D-314).
- **IPU_CMD** reads VDEC's or FDEC's result; after a reset or BCLR and before either, the stream's next 32 bits,
  reading which takes the first qword in (D-311). **IPU_TOP** looks at the next 32 bits, BUSY while a command runs or
  fewer than 32 bits are in (ps2tek, D-311).
- **Start codes** (D-305): at eight or more zero bits where a macroblock or an MBAI would begin, a start code is 23 or
  more zeros and a one that ends a byte; the pointer is left at its prefix, after the stuffing, so that FDEC then reads
  the whole code, as IDEC's notes ask. Anything else there is an error code, the pointer at its first zero.
- **VDEC** gives the code's length in bits 31..16 for every table, as §8.5.3 prints it (D-308); bits no code begins,
  a start code included, give 0 and ECD (D-306).
- **IDEC** resets the DC predictors at its start, decodes from the first macroblock's MBT, takes MBAI from the second
  on and ends with ECD at any increment but 1 (§8.4, D-318), passes MPEG-1's stuffing (D-326), decodes DT under DTD
  and QSC under Intra + Q; a D-picture's macroblocks are DC alone and end_of_macroblock (D-326).
- **BDEC** decodes CBP for a non-intra macroblock (CBP shown in IPU_CTRL, 3Fh for an intra one, D-309), resets the
  predictors only with DCR, and ends after the last EOB with no flag, or at a start code with SCD, or with ECD (§8.4's
  cases a to c).
- **SETIQ** takes 64 bytes from the pointer in the stream's order and stores them through the zigzag scan, as §7.3.1
  says a downloaded matrix is (the FMV player SETIQs straight from the sequence header); **SETVQ** 16 RGB16 entries;
  **CSC** and **PACK** take whole macroblocks of RAW8 and RGB32; **SETTH** the two 9-bit thresholds. Every command ends
  with INT_IPU (D-310).

### 34.5 The IDCT

Annex A defines the reference as the real transform rounded to the nearest integer, halves away from zero, saturated to
[−256, 255], and admits any approximation within IEEE 1180's bounds. The plan (§5.7) asks for the reference, and
for the IPU's departure as a dispute (P7, D-300). The real transform is evaluated in double precision,
separably, from cosines written as the nearest doubles; for coefficients in the standard's range the error stays below
10^-10, so a value further than 2^-20 from a half-integer rounds correctly. Near a half the value is decided exactly:
8f(x, y) gathers into integer multiples of cos(kπ/16), k = 0 to 7 (each product of two cosines being half the sum of
the cosines of the sum and the difference of their angles, and C(0) being cos(4π/16)); these are a basis of the field
Q(cos(π/16)) over the rationals, so f is a half only when the seven irrational multiples vanish, and otherwise its side
of the half is read from the multiples with the cosines in fixed point to 2^-100 [A]. Measured on the corpus
[M, `the_corpus_values_near_a_half_are_few_and_decided_exactly`]: of 159,744 real values none lay within 2^-20 of a half,
the nearest 2.1 × 10^-6 from one, which is why a tie rule on doubles alone, first tried, would not do: a game's hour of
FMV holds some 10^10 values. The test model decides near-halves by the same mathematics in its own code, and the random
blocks of `the_rational_terms_give_exact_halves_and_the_exact_reading_agrees_everywhere` agree with the exact reading at
every position. Annex A's item 4 blocks are exact (`annex_a_item_4_blocks_give_the_exact_saturated_transform`).

### 34.6 The post-processing's readings

[R EEUM §8.6; A]: §8.6.1's products "& ~0x003f" are floors of the two's-complement values, kept as halves; step 5 is
read with Y less 16, step 4's result, though it prints Y; step 8's "round off bit 0" adds a half and floors; each
chroma sample serves the 2 × 2 luminance samples it lies on, no document giving an interpolation (D-319). §8.6.2's
dither adds the table's halves and floors before clipping; A1 is A8 = 40h alone (D-320); with SGN the dither works on
the reversed values, which is what "clipping is not performed correctly" describes (D-321). §8.6.3 takes the first
entry at the least distance (D-320). PCSX2's PACK equals every word in both RGB16 forms and INDX4, its own witness for
the dither's orientation and flooring, A1 and the VQ's tie.

### 34.7 The board

Channel 3 asks the IPU for as many qwords as the output FIFO holds and channel 4 for its room; a qword handed over at a
unit's end runs the IPU as far as it can; the board wakes the DMAC after each step if a FIFO's count changed
(`sync_ipu`), and after each EE access to the IPU's registers or FIFOs. A tag sent by TTE on channel 4 is not given to
the IPU (D-325). The S11 stand-ins for channels 3 and 4 are no longer reached.

### 34.8 Timing, and what is not modelled

- **The IPU's decoding takes no time** (D-324): a command's progress is bound by its data's arrival and its output's
  departure, a qword a bus cycle each way on the DMAC. Measured by the programs' Count on each command, from IPU_CMD's
  write to BUSY's clearing [M, `examples/s27.rs timing`]: a BDEC of 48 qwords takes 90 to 144 EE cycles on Sapphire, 262
  to 346 on PCSX2 and 1 on Play!; an IDEC of four macroblocks in RGB32 (256 qwords) 636 to 684, 1,207 to 1,319 and
  4,801; in RGB16 (128 qwords) 288 to 324, 675 to 787 and 4,799; a CSC of 7 macroblocks to RGB32 (448 qwords) 1,230 to
  1,236 on Sapphire and 1,424 on PCSX2. No document gives the IPU's rate, and the two lineages disagree by a factor of
  four on IDEC and wholly on BDEC, so neither is a witness; FMV pacing on Sapphire is the DMA's and the EE's until a
  game at S28 shows otherwise.
- MPEG-2's 4:2:2 and 4:4:4 macroblocks, which the IPU's formats cannot hold; the scalable extensions; concealment
  vectors inside IDEC (no symbol of §8.5.1 carries them).
- The state format's new contents (the IPU's registers, FIFOs, tables and a command in progress): `SPH2` gains them
  with the rest (S10, S30).

### 34.9 The corpus

`tests/s27/streams/`, of the project's own making, 64 × 48, encoded by ffmpeg 8.1.3 (built by gcc 16), run as a black
box on 2026-10-25 from its `testsrc2` pattern with `noise=alls=40:allf=t` for the intra streams; each stream's
`ffmpeg -pix_fmt yuv420p` decoding is kept beside it as the witness. Escapes come from the noise at low quantiser
scales; every stream is closed with a sequence_end_code by the harness, ffmpeg writing none.

| Stream | Bytes | Frames | What it exercises (ffmpeg's options) |
|---|---|---|---|
| `intra-q2` | 10,771 | 3 | I only, quantiser 2, many escapes (`-g 1 -qscale:v 2`) |
| `intra-tools` | 4,223 | 2 | non-linear scale, Table B.15, alternate scan, 10-bit DC (`-non_linear_quant 1 -intra_vlc 1 -alternate_scan 1 -intra_dc_precision 2`) |
| `intra-matrix` | 2,708 | 2 | a loaded intra matrix, 9-bit DC (`-intra_matrix …`, `-intra_dc_precision 1`) |
| `intra-field` | 5,467 | 2 | interlaced, dct_type per macroblock (`-flags +ildct+ilme -top 1`) |
| `intra-alt`, `intra-nonlinear`, `intra-ivf`, `intra-dc11` | 2,441, 1,480, 1,875, 2,113 | 1 each | each tool of `intra-tools` alone, and 11-bit DC |
| `pb` | 3,402 | 7 | P and B, a loaded non-intra matrix (`-g 6 -bf 2 -qscale:v 4 -inter_matrix …`) |
| `pb-rate` | 7,647 | 5 | P and B under rate control, Intra + Q and MC Coded + Q, non-linear scale, alternate scan (`-b:v 400k -lumi_mask 0.3 -p_mask 0.3`) |
| `mpeg1-intra`, `mpeg1-pb` | 5,635, 2,891 | 2, 5 | MPEG-1 (`-c:v mpeg1video`), I only and P and B |

`encode.rs` writes two more symbol by symbol: `odd_mpeg2`, an I-picture of escapes for levels a code exists for,
±2047, the forbidden 0 and the reserved −2048 by escape, runs to the 64th coefficient; an I-picture at 11-bit DC with
Table B.15 and the non-linear scale under Intra + Q 31; a P-picture whose non-intra blocks begin with escapes, run 63
among them; and `odd_mpeg1`, an I-picture with stuffing before increments and both of MPEG-1's escape lengths, and a
D-picture.

### 34.10 The gate (measured 2026-10-25)

**The standard's model** (`tests/s27/standard.rs`) decodes every stream from §6.2's syntax with its own copy of Annex
B's tables matched bit by bit, its own inverse quantisation and a direct double sum for the IDCT, and frame prediction
for the P- and B-pictures. **ffmpeg's decodings agree with it within one step**: 1,983 of the 32 frames' 147,456
samples differ, every one by one, at most 1.5 % of a stream's samples (`the_corpus_decodes_as_the_standard_and_ffmpeg_agree`),
which is what two compliant IDCTs give and what a table's error would not.

**The driver** (`tests/s27/driver.rs`) runs the IPU through each stream by its commands alone, as §8.1.3's flow does:
the stream behind a preamble holding the two default matrices, BCLR, SETTH, SETIQ for each matrix the stream loads,
IPU_CTRL for each picture, VDEC on every MBAI, macroblock_type and motion_code, FDEC on every fixed-length field, then
IDEC on an I-picture's slices or BDEC on each coded macroblock, FB carrying the pointer to each symbol the model placed
and FDEC with FB 32 crossing longer gaps. Every VDEC and FDEC result, every block, picture, CBP, flag and pointer is
checked against the model and `manual.rs`, §8.6 written apart from `src/ipu/csc.rs` in the manual's own units.

| Test (`tests/ipu.rs`) | What it shows |
|---|---|
| `the_corpus_decodes_as_the_standard_and_ffmpeg_agree` | the model against ffmpeg, as above |
| `bdec_gives_the_standards_blocks_for_every_macroblock_of_the_corpus` | BDEC over all twelve streams: 1,931 commands, every coded macroblock's 48 qwords |
| `idec_gives_the_standards_pictures_through_the_manuals_csc` | IDEC on every I-picture's slices, RGB32 |
| `odd_escapes_levels_and_mpeg1s_forms_decode_as_the_standard` | both written streams, BDEC and IDEC |
| `error_and_start_codes_end_idec_and_bdec_where_section_8_4_says` | six cases: an increment of 2 inside IDEC, a code no table holds, a run past the 64th coefficient, zeros that make no start code, a start code after stuffing, one after a further zero byte |
| `csc_and_pack_follow_the_manuals_post_processing` | CSC to RGB32 and RGB16 truncated and dithered, under thresholds; PACK to RGB16 and INDX4, truncated and dithered; ramps of Y, Cb and Cr and random samples |
| `vdec_gives_every_row_of_the_manuals_tables` | §8.5.3's 105 rows, error rows and MP1's two among them |
| `every_table_is_prefix_free_and_as_complete_as_the_standard` | Annex B as built |
| `the_corpus_values_near_a_half_are_few_and_decided_exactly` | §34.5's measurement |
| `the_programs_replay_their_logs_through_the_dmac_on_the_board` | the 32 replay programs on the interpreter with the board, every graded word |
| `the_registers_and_fifos_answer_the_ee_as_chapter_8_says` | the register program, 346 records |

**The programs** (`tests/s27/program.rs`, `registers.rs`; `cargo run --release --example s27 -- <folder>`): a replay
program carries a driver's command log and the stream image behind its code; toIPU carries the image in normal mode
from the first BCLR on, fromIPU is started for each command's output straight into the record, and IPU_CMD (graded for
VDEC and FDEC), IPU_CTRL (its FIFO counts cleared, which depend on the DMA's pace), IPU_BP's pointer and the command's
EE cycles by Count are recorded. Thirty-two of them (BDEC over the twelve streams and the two written ones, IDEC over the
nine intra streams and the written ones, five IDEC variants on `intra-matrix`: RGB16 truncated and dithered, SGN, the
thresholds 60 and 140, SGN with the dither, and `s27-post` and `s27-errors`), and the register program: 163,964 graded
record words, every one equal on Sapphire.

**Tests** [M, `cargo test --release`, on the tree with S25, S15 and S26 beneath]: `tests/ipu.rs` **11 of 11**; lib
**59 of 59** (seven new: the input's order and its buffer, the CSC's steps and dither, Annex A's item 4, a DC's halves,
the exact reading against random blocks). The crate's other tests unchanged with the IPU on the board: `cop0` **19 of
19**, `dmac` **18 of 18**, `draw` **13 of 13**, `fpu` **9 of 9**, `gif` **15 of 15**, `gs` **12 of 12**, `interpreter`
**23 of 23**, `iop` **13 of 13**, `micro` **12 of 12**, `mmi` **13 of 13**, `pcrtc` **5 of 5**, `programs` **3 of 3**,
`sif` **5 of 5**, `sio2` **5 of 5**, `spu2` **10 of 10**, `texture` **11 of 11**, `vif` **12 of 12**, `vu0` **14 of
14**, `vu_micro` **24 of 24**; lib and `ipu` also under `test-checked`; clippy clean. WiseMan with `blast.sh`'s filter
(`SapphireTests`) **5 of 5**.

### 34.11 On the referees (measured 2026-10-25)

`ps2referee.py ram <referee> … --until 1FFFF0=5A99D00E 200000:<words>` read each program's record from PCSX2 2.8.2
(Americas BIOS, fast boot) and Play! 0.72, one process at a time; `examples/s27.rs explain` reads them by command and
counts the output's lanes (BDEC's samples, RGB32's bytes, RGB16's fields) by their difference. Only the records were
read. Each departure was then tested by building its hypothesis into a scratch copy of the IPU and comparing again.

**PCSX2.**

- **Agrees**: every VDEC and FDEC value but D-308's lengths, every pointer and flag on the streams, the FIFO's
  depth and its filling, BCLR's pointer and BCLR while busy, FB past 32, VDEC's error codes, CBP 3Fh for intra
  macroblocks, INT_IPU at each command, IPU_CMD before a command, non-intra inverse quantisation and mismatch control,
  MPEG-1's oddification and escapes (`bdec-pb`, `bdec-mpeg1-pb`: every lane within one step), Table B.15, the
  non-linear scale, field DCT, 9-bit DC, every PACK word, CSC's luma ramp.
- **Its IDCT** departs by one step in 1.4 to 6.6 % of the samples (678 of 12,288 lanes on `bdec-pb`): P7's departure, in
  a lineage whose transform is its own (D-300).
- **Intra BDEC output clamped to 0 to 255** (D-301): clamping every intra macroblock's samples reproduces every lane of
  the six intra streams but its IDCT's steps; above 255 it wraps on the written stream's largest levels.
- **Alternate scan** (D-303): pairing each QFS[n] with the matrix's n-th byte in the stream's zigzag order, instead of
  W[v][u] with QF[v][u], reproduces every lane of `bdec-intra-alt`, `bdec-intra-tools` and `bdec-pb-rate` but its IDCT's.
- **IDEC's QSC** (D-304): taking QSC as quantiser_scale itself (IPU_CTRL.QST forced to its non-linear table, equal to
  the code up to 8) reproduces IDEC's pictures within its CSC's and IDCT's steps (`idec-rgb16`: 708 of 24,576 fields one
  step low, 141 one step high, 3 by two).
- **CSC** (D-319): one step lower in 910 of the first CSC's 1,792 pixels, never on the luma ramp; flooring the products
  towards zero reproduces 1,072.
- **IPU_CTRL** keeps IDP's low bit alone (D-316), so 10- and 11-bit DC decode otherwise (`intra-dc11`); **VDEC** omits
  the length for macroblock_type, motion_code and dmvector (D-308); **IPU_TOP** reads 0 (D-311); **RST** raises no
  interrupt (D-315); **IDEC** stays busy at an increment of 2 (D-318) and **SGN with the dither** never ends
  (`idec-sgn-dither`, no done word in 90 s, D-321).

**Play!.**

- **Agrees**: VDEC's lengths for every table (D-308), IPU_TOP's BUSY and value (D-311), BCLR while busy (D-314), the
  pointer within a slice, the first slice's blocks within its IDCT's two steps.
- **Departs**: no CBP (D-309); the pointer left at the first zero after a slice's last EOB rather than at the start
  code (D-305), after which the replays are out of step with it; SETIQ's FB ignored; the FIFO unbounded (D-302); no
  INT_IPU and DATA unchanged after a VDEC error (D-306, D-310); MPEG-1 stuffing an error in IDEC (D-326); IDEC stuck at
  an increment of 2 (D-318); `s27-post` never ends (CSC). Its IPU completes every command at once but IDEC, which takes
  4,800 cycles (D-324).

So **PCSX2 is a witness for the commands' register behaviour, the bit stream's handling, non-intra decoding and PACK**,
and its departures in intra decoding, the alternate scan, IDEC's scale and CSC are each reproduced exactly by a rule no
document gives; **Play! is a witness for VDEC's output and IPU_TOP**, and departs too early in each stream to weigh its
decoding.

### 34.12 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | Table B.14's codes for run 0, levels 5 and 6 swapped, in `vlc.rs` | `bdec_gives_…`, `idec_gives_…`, `the_programs_replay_…` |
| F2 | Mismatch control skipped, in `Ipu::reconstruct` | `bdec_gives_…`, `idec_gives_…`, `odd_escapes_…`, `the_programs_replay_…` |
| F3 | The dither matrix transposed, `DITHER[x][y]`, in `csc::rgb16` | `csc_and_pack_follow_the_manuals_post_processing` |

Each was applied, `tests/ipu.rs` and the lib's tests run, and the source restored [M]. F1 passes the tables' own check,
which sees codes and not values, and is caught by the model's own copy of the tables.

### 34.13 What the plan got wrong, or left open

- **§5.7 and §2.3** expected the IDCT's rounding to be the IPU's one departure (P7). On the referees the IDCT departs as
  predicted, but it is the smallest of each lineage's departures; the alternate scan's weighting, intra output's range
  and IDEC's scale are larger, and no document decides them against the standard.
- **§3.2's row** names "the project's MPEG-2 streams against the standard" with the referees for IDCT rounding: the
  standard needed a decoder of its own on the project's side (the model) and a third lineage for the model itself,
  ffmpeg's decoding, run as a black box.
- **The gate's "output read back by an EE program"** needs the command sequence an FMV player issues; the replay
  program carries it as a table, so the EE program is one loop and the stream's walk is the driver's.
- **MPEG-1** rests on H.262 §D.9 alone (§34.3).

### 34.14 Questions this stage leaves

1. **Q1 [decision], the alternate scan's weighting (D-303).** PCSX2 pairs coefficients with the matrix's bytes in
   zigzag order under the alternate scan, which an FMV encoded with a loaded matrix and the alternate scan would show
   as wrong weights; the standard pairs them by place. *Recommended*: keep the standard's pairing, and at S28 compare an
   FMV of a bench game whose stream uses the alternate scan (interlaced FMVs often do) on Sapphire and PCSX2 frame by
   frame; if PCSX2's pictures are the right ones there, the IPU is PCSX2's and the dispute turns.
2. **Q2 [decision], IDEC's QSC (D-304).** *Recommended*: keep Table 7-6's mapping, which BDEC uses on both lineages;
   S28's FMVs decoded by IDEC decide it the same way as Q1.
3. **Q3 [routine], intra BDEC's range (D-301).** *Recommended*: keep RAW16's signed samples, the manual's format; an
   FMV player that adds a prediction to an intra block never does, so games cannot tell.
4. **Q4 [routine], the decoding's time (D-324).** *Recommended*: keep it untimed until S32's bench or a game at S28
   shows FMV pacing that depends on it; the two lineages disagree by a factor of four.

### 34.15 What was read for this stage

`Sapphire_Plan.md` §1.1 (the IPU row), §2, §3, §5.7, §7 and §11; this page's §20 (S11, the DMAC and channels 3 and 4);
`Sapphire_Disputes.md`'s shape and D-34 to D-42; `Sapphire_Referees.md` §2 and §3. ITU-T H.262 (02/2000) §4.1, §6.1.3,
§6.2, §6.3.9 to §6.3.18, §7.1 to §7.6, Annex A, Annex B and §D.9; the *EE User's Manual* (Version 5.0) §1.2.8, §2.2.1,
§3.1 and chapter 8 whole; ps2tek at `1c91660`, "Image Processing Unit" and its I/O map. No emulator, decoder, SDK or
loader source, no ps2autotests source and no firmware. ffmpeg's source was not read; its binary made and decoded the
streams.

### 34.16 What this stage does not cover

- The FMV library a game links, which drives these commands on the player's BIOS: S28's games.
- The IPU in the machine's frame (`machine.rs` holds no `System` yet): S10.
- The state format's new contents: S30.
## 35. Stage S24, the player's BIOS to its menu (2026-10-25)

*Numbered §35 and its disputes from D-340, since S27 (§34) was built beside it and took D-300 to D-326.*

### 35.1 In one paragraph

The plan's §7 row S24, its first half, on the tester's go of 2026-10-25. **`machine` now holds the board**
(`board::System`, the EE core and every unit of S6 to S26), the player's BIOS in the boot ROM at 1FC00000h for the EE and
the IOP alike, both processors started at their reset vectors, the IOP's reset line on the SIF's CTRL bit 19 (D-340), the
drive of S23 joined to the IOP's bus (its registers at 1F402000h, channel 3, IRQ 2), the pads and cards of S25 on SIO2,
and the console's market taken from the BIOS entry the player's file came in. The EEPROM is the console medium `ps2.nvm`,
one file per console model, blank (all zeros) until the BIOS writes it, read and written through S23's S commands.
`v1` hands on the PCRTC's picture (S20) and the SPU2's sound (S26) in place of S1's black stub. Run on the tester's
Americas image `ps2-0230a-20080220.bin`, observed only through the hardware's own reports (§35.4), **the BIOS reaches
OSDSYS's first screen, the initial setup's "Select language", 11.5 seconds of console time after power-on** (fully drawn
at 12.0), through the Sony Computer Entertainment animation and both logos; **the picture equals PCSX2's on the same BIOS
but for the date and time OSDSYS draws in the top corners on Sapphire** (§35.8), and **the menu answers the pad** (§35.9).
Fourteen behaviours of the board were missing or wrong on the way, each found from the reports, built from a document or
measured on PCSX2, and recorded (D-340 to D-351); five exact speed changes made a boot of about ten minutes possible
(§35.7). S10 (the EE's ps2autotests by fast boot) is next and attaches where §35.11 says. No emulator, SDK, loader or
ps2autotests source was read; the BIOS image was handed to the machine and to PCSX2 by path, never opened, hashed or
analysed, and no program counter, instruction or memory holding BIOS code or data was logged or read.

### 35.2 The machine

- **`src/machine.rs`**: `Machine` keeps the media, the market, `Box<System>`, the frame count, the ports and the frontend's
  buttons and sticks. `Machine::new` makes the board in the market's standard (NTSC for Japan and the Americas, PAL for
  Europe), the EE at its Reset exception (`Ee::reset`, BFC00000h with ERL and BEV), SMODE2 interlaced in frame mode from
  the first field [C, as the BIOS sets it, so the picture is the standard's 640 × 448 or 640 × 512 frame before the BIOS
  programs the PCRTC and with no BIOS], a DualShock 2 in each port the settings name and an erased card in each slot.
  `insert_bios` copies the image into the EE's map and the IOP's bus (4 MiB each, read-only to both) and resets both
  processors. `advance` runs the EE an instruction at a time, the board brought along after each, until the PCRTC begins
  its next field (V-blank's rise), so a frame is a field at the standard's 59.94 Hz or 50 Hz; with no BIOS, or once the EE
  stops on an instruction the interpreter lacks (`Stop::Unimplemented`, which keeps neither the address nor the word),
  the board alone runs in steps of 16,384 cycles. The memories of §4.3 are read and written where the board keeps them
  (`space_byte`, `set_space_byte`); the state keeps them alone until S30 (§35.6).
- **`src/iop/mod.rs`, `bus.rs`**: `Iop::reset` (BFC00000h, SR's BEV, its own INTC, DMAC, timers and cache control reset,
  RAM kept, held while the ROM is empty); the drive on the bus (`cdvd_read`, `cdvd_write`, `sync_cdvd` raising IRQ 2 at
  the drive's interrupt's rise, its events among the bus's); a held IOP's units still run, so the SPU2 makes its sound
  and the timers count with no BIOS.
- **`src/cdvd/mod.rs`**: `cdvd::Ports`, channel 3 taking the buffered sector four bytes a word; the region answer and the
  boot commands (D-348); counts of the S and N commands received and each S command's last parameters, the drive's own
  report.
- **`src/v1.rs`**: the market from the setting, else from the BIOS entry given (`console_market`: the Americas', then
  Europe's, then Japan's); the BIOS loaded when the market's entry is given, so a setting naming another market starts
  a machine with no BIOS; the frame and its size from `pcrtc::Picture`; the sound drained from the SPU2's queue; buttons
  and sticks handed to the pad in their port (`Machine::sync_pad`); the cards in SIO2's slots; the EEPROM's files
  (§35.5); the description's words now say the console starts to its menu.
- **`src/reports.rs`**, the hardware's reports (§35.4); **`src/mch.rs`**, the memory controller's serial interface
  (D-342).
- **No new cross-component dependency.** `blackmoon-r3000` was already a dependency; the tests reach its `Bus` trait.

### 35.3 Firmware, the market and the drive

The three entries of §5.9 are unchanged (S1, S2): 4,194,304 bytes, `required: false`, each with its market's retail MD5s,
by which the host finds a player's file whatever its name; the core never reads an image for anything but running it.
The console's market is the entry's: on *Automatic* a European file makes a European console, PAL. The drive answers the
region query (S command 36h) with that market's parameters, measured on PCSX2 with each gate image (D-348), and reports
every disc as authenticated as §5.6 says; no disc is put in the drive at this stage, an image's disc being S29's (and
G10's) work, so the BIOS boots to its menu with the tray closed and empty.

### 35.4 What the boot was allowed to show, and what it showed

`Sapphire_Plan.md` §1.4 and the tester's rules for this stage allow the emulated hardware's own reports and nothing of
the BIOS's code. The board
keeps (`reports::Reports`): every access to a register no unit answers or an address no device holds, by processor,
address and direction, with its count, first and last cycle and last value written, KPUTCHAR's values never kept (they
are the BIOS's text); on request, the same counts for the registers the units do answer; the EE's and the IOP's
exceptions counted by their code; the EE's bus traffic past its caches by memory, counts only; the drive's commands
(§35.2); the IOP's resets; and, when a run ended, the SIF's, the DMACs', the SPU2's, the GIF's and the VIF's registers.
`examples/s24.rs` runs a console on a BIOS given by path for a count of frames, presses buttons at chosen frames, writes
the pictures and prints these reports and registers; nothing else. The boot, run after run (measured 2026-10-25):

| Where it stopped | What the reports showed | The cause, and the rule built |
|---|---|---|
| The EE, 2.3 ms after power-on | uncached fetches from the boot ROM alone, no RAM access, no register after reading D_ENABLER | D_ENABLE 0 at reset (D-341) |
| The EE, at the memory controller | MCH_RICM written, MCH_DRD read once, then nothing | no answer from the RDRAMs (D-342) |
| The IOP and the EE, at the first SIF packet | SIF1 started with CHCR 41000300h; the EE's packet never reached the IOP's memory | SIF1 run in slice mode (D-343); then no IRQ 3, DMACINTEN 0 (D-344) |
| The EE, at the IOP's reply | SIF0's FIFO left two words short of the EE's quadword | the packet's padding (D-345) |
| The IOP, rebooted | a bus error at BE000000h; later 1F900B60h | the DVD player's ROMs' place, the SPU2's window (D-346) |
| The IOP, at the sound's setup | SPU2 channel 4's BCR written by halfwords, a transfer of 65,536-word blocks | the halfword lanes (D-347) |
| The IOP, at the drive | S command 36h, then 22h, then 24h retried without end | the boot S commands (D-348) |
| The IOP, after a SPU2 transfer | core 1's STATX polled 22 million times with interrupts off | STATX's bits (D-349) |
| The EE, at the first picture | D1_CHCR polled 203 million times; VU1 running, VIF1 waiting on it; GIF_STAT APATH 3, TAG0-3 zero | PATH3 held by a zero tag (D-350), then VU1's XGKICK waiting "never" (D-351) |

The EE's CTRL bit 19 was never written on the way (D-340 rests on its test). The BIOS read the EEPROM's area 1, two
blocks, and wrote nothing to it on the way to the menu.

### 35.5 The EEPROM and the pads

- **The medium `ps2.nvm`** (`Sapphire_Plan.md` §4.4): three console files in core info, 2, 3 and 4, `.jp.nvm`, `.us.nvm`
  and `.eu.nvm`, labelled "Inside the console", one per console model, since the EEPROM holds a model's and a market's
  parameters [R ps2tek "CDVD S Commands", in outline]; machine info lists the market's alone. A file of another length
  than 1,024 bytes is refused with words; an empty or absent one starts the console blank. `battery` hands the bytes
  back, `BATTERY_CHANGED` once WriteConfig has written; `battery_saved` clears it.
- **Blank but valid.** No permitted source gives the EEPROM's layout: ps2tek names OpenConfig, ReadConfig and CloseConfig
  and nothing of the contents, its BIOS chapter only CDVDMAN's exports; psdevwiki refused the fetch (HTTP 403, as at S2);
  the later ps2tek edition (Israelson's) is byte-identical in its CDVD chapter [M, SHA-256 equal]. So **blank is all
  zeros** (S23's), and its validity was measured instead of synthesised: PCSX2 run with its own default EEPROM and with
  one of zeros (`ps2referee.py menu --blank-nvm`) draws the same language screen at 20 s to the pixel [M], and the BIOS on
  Sapphire reaches that screen from zeros. The plan's "synthesised from the documented layout", and a checksum, have no
  document to stand on (§35.12, Q2).
- **The pads.** The frontend's buttons and sticks reach SIO2's DualShock 2 in their port whenever they change; the BIOS's
  PADMAN polls them (SIO2's SEND3 queue run 757 times in ten seconds of boot) [M].

### 35.6 The picture and the sound

`frame` is the PCRTC's woven RGBA picture (S20) at its own size, `frame_info` its size and the television's 4:3 shape;
machine info's base size is the standard's nominal frame. The sound is the SPU2's output at 48,000 Hz, drained from its
queue (S26); with no BIOS the units still run and the sound is the SPU2's silence. The state format keeps the memories
alone, version 1's layout unchanged, the processors' and units' registers being S30's. A console with no BIOS loading a
state is brought from power-on to the state's frame first (`Machine::replay_to`; its clocks are its whole state, the
cards, the EEPROM and the pads kept, the sound made on the way dropped), so the kit's C7 and C8 hold; a console running a
BIOS refuses every load (`StateError::Running`) until S30. *Superseded 2026-10-25 at S30 (§38.5): every unit is in the state, both rules
are gone, and a running console loads.*

### 35.7 Speed, and the exact changes that made a boot practical

The first boot ran at 2.3 frames a second of wall time while the EE spun in its boot ROM, and at 0.6 once OSDSYS drew.
Five changes, each a pure function's shortcut or a cache, every test and the boot's own traffic counts equal before and
after [M]:

1. The EE's timing model visits only the resources an instruction reads and writes (bit iteration) rather than all 100
   (`ee::timing`): 8.7 s to 3.4 s for 20 frames of the boot.
2. The board caches the PCRTC's next event while nothing changes the timing (`Board::pcrtc_next`): 3.4 s to 2.7 s.
3. The TLB memoises its last page's lookup until an entry is written (`ee::tlb`).
4. The board's idle shortcut: with no event due and the GIF, both VIFs and both VUs idle (`Gif::quiet`, `Vif::idle_to`,
   `Vpu::idle_to`), `run_to` moves the clocks and brings the IOP along, which is what the whole path does: 70 s to 39 s for
   120 frames.
5. The LOD's log2 (`gs::exact::log2_scaled`) taken from double precision where the estimate lies more than 2^-24 of a
   unit from an integer, the exact 256-bit steps otherwise; 20,000 random cases at four depths equal; OSDSYS's drawing
   0.57 to 1.1 frames a second.

A boot to the menu now takes about 7.5 minutes of wall time on the desktop (1 to 3 frames a second, about 30 times
slower than the console), which is the reference path's speed; the speed stages (S32 on) own the rest.

### 35.8 The gate: OSDSYS against PCSX2 (measured 2026-10-25)

The anchor is the initial setup's language screen, which neither console leaves without a press and whose only motion
is the clock's seconds on Sapphire. PCSX2 2.8.2 booted the same image with no disc (`-bios`), a blank EEPROM, software
rendering, and its picture taken from a state saved through PINE 20 s after it began to draw
(`ps2referee.py menu`); the state's screenshot is 640 × 480 whatever the settings, so Sapphire's 640 × 448 frame is
scaled to it bilinearly, and pixels are compared per channel.

| Sapphire's frame against PCSX2's picture | Pixels of 307,200 differing by more than 32 | in the top 70 rows | elsewhere | elsewhere by more than 64 |
|---|---|---|---|---|
| Americas `0230a`, frame 759 (12.65 s); PCSX2 weaving, top field first (`--deinterlace 2`, its default here) | 8,906 | 2,249 | 6,657 | 2,131 |
| **Americas, frame 759; PCSX2 weaving, bottom field first (`--deinterlace 3`)** | **2,518** | **2,249** | **269** | **0** |
| Europe `0230e`, frame 800 (16.0 s, PAL, 640 × 512 scaled); PCSX2 weaving, top field first | 7,283 | 2,364 | 4,919 | 687 |
| **Europe, frame 800; PCSX2 weaving, bottom field first** | **2,638** | **2,364** | **274** | **1** |

With PCSX2 showing the woven frame as Sapphire's PCRTC hands it on, the two equal but for the **date and time OSDSYS
draws in the top corners on Sapphire** (2,249 and 2,364 pixels: the clock of the drive's RTC, 1999/12/31 15:00 at
power-on as the BIOS shows the drive's 2000-01-01 00:00) and **269 and 274 pixels of the text's edges** where the scaling
meets PCSX2's own; outside the clock's rows no pixel differs by more than 64 on the Americas image and one on Europe's.
PCSX2 draws no clock on that screen, with its clock the host's and with its setting for a manual clock at 2000-01-01
(whose effect was not confirmed); why OSDSYS shows it on Sapphire is open (§35.12, Q3). The stated count for the gate is
therefore **at most 300 pixels outside the clock's rows differing by more than 32 after the scaling, and at most 10 by
more than 64**: met on both images.

**Boot time to the menu** [M]: Americas, the language screen beginning 11.5 s after power-on (frame 690) and whole at
12.0 s (frame 720), about 7.5 minutes of wall time; Europe, beginning 11.4 s (frame 570 at 50 Hz) and whole at 12.0 s
(frame 600); PCSX2 on the Americas image between 14 and 16 s after it begins to draw, the Europe image's not measured.

### 35.9 The menu answers the pad (measured 2026-10-25)

`examples/s24.rs` pressed the frontend's buttons on port 1 at chosen frames (Americas image, measured 2026-10-25):

| Frames | Pressed | What the picture did |
|---|---|---|
| 760-765 | Cross (bit 14) on the language screen | the text faded out by 820 and **"User Preferences: Language ◀ English ▶"** was drawn by 979 |
| 980-985 | Down (bit 6) | nothing, as the language is chosen with Left and Right |
| 1060-1065 | Cross | **"User Preferences: Time Zone ◀ GMT +4:30 Kabul ▶"** by 1100 |

So the presses reach OSDSYS through SIO2 and its PADMAN, and the initial setup steps on, which is the gate's "the menu
responds to the pad" [M]. On the first run Down, pressed on the language screen itself, changed nothing (§35.4's
pictures equal), that screen having no list to move in.

### 35.10 Tests and seeded faults (2026-10-25)

**Tests** [M, `cargo test --release`]: the crate whole, **301 of 301** before S27's rebase and **319 of 319** on the tree
with S27 beneath (its `tests/ipu.rs` 11 and seven lib tests added), among them: `tests/boot.rs` **5 of 5**, new (SIF1
in chain mode and SIF0's padding against PCSX2's records, CTRL bit 19's reset line, the power-on values with the memory
controller's answers, halfword DMA writes and the absent places); the lib **59 of 59**, 66 with S27's (new: `mch`'s serial answers, the
drive's boot S commands, the machine with and without a BIOS, the drive's market, the pads in their ports, the EEPROM's
files, a state brought to its frame and refused by a running console, the LOD shortcut against the exact steps over
20,000 cases, the reports' KPUTCHAR rule); `tests/gif.rs` **16 of 16** (a zero tag freeing PATH3; PATH1's estimate);
`tests/iop.rs` **13 of 13** (`s21-map` at 9E000000h, SIF0's padding in the FIFO); `tests/spu2.rs` **10 of 10** (`s26-irq`'s
STATX); every other test file unchanged and passing. The lib, `boot`, `gif`, `iop` and `sif` also under `test-checked`;
clippy clean on every target. `SapphireTests` (WiseMan, the blast radius's filter) **5 of 5**, the kit's C7 and C8 passing
by §35.6's rule after failing on a first run that kept the memories alone.

**Seeded faults:**

| # | Fault | Caught by |
|---|---|---|
| F1 | the EE's CTRL bit 19 not resetting the IOP | `ctrl_bit_19_holds_the_iop_in_reset_until_it_clears` |
| F2 | a European console answering the region query with the Americas' parameters; then given the Americas' EEPROM file | `the_drive_reports_the_consoles_market`; `the_eeprom_is_the_markets_console_file` |
| F3 | the pad's port swapped, port 1's buttons handed to port 2 | `the_buttons_and_sticks_reach_the_pad_in_their_port` |

Each was applied, the tests named run, and the source restored [M]. A wrong EEPROM checksum, the fault first planned,
has no checksum to break (§35.5); F2's second half stands in for it.

### 35.11 Where S10 attaches, and what this stage does not cover

- **S10**: an ELF given as the image is recognised and the console boots its BIOS to the menu without it; S10 enters it by
  fast boot once the kernel is up, and catches the IOP's console in a `tty` space at the module interface (§7's S10 row),
  then runs the 78 ps2autotests on this machine. The second half of S24's gate is S10's.
- **S25 part B** (the tester's go): the OSDSYS card checks, a card formatted by the BIOS both ways, G10's start without a
  disc, G11; P8, the BIOS refusing an emulated card, to be recorded there.
- **Not modelled, each named**: the EE's SIO beyond its transmit register (F100h-F150h written, read zero); the registers
  at 1000F400h-1000F4A0h other than MCH, 1000F500h, 1000F510h, 1F801470h, 1F803204h and 1F803800h on the EE, and the IOP's
  SSBUS and configuration registers (1F801000h-1F801020h, 1F801400h-1F801420h, 1F801450h, 1F801460h-1F80146Ch,
  1F801560h-1F8015F0h, 1F802070h, FFFE0140h, FFFE0144h), all taking their writes and reading zero as before, the reports
  listing them; the EE's reads of 1F8019xxh and 1F402005h; the drive's RTC from 2000-01-01, not the host's clock; a disc in
  the drive (S29, G10); the processors' state in `SPH2` (S30).
- **Seen and not fixed** (the intro, not the gate's anchor): white bands above and below the Sony Computer Entertainment
  animation's picture, and its light trails drawn as small textured squares where PCSX2 draws glowing points; GS work for
  S17 and S18's owners (§35.12, Q4).
- The CRT profile and the audits stay owed while the console is offered to no player (§6).

### 35.12 Questions this stage leaves

1. **Q1 [decision], the S commands taken from the referee (D-348).** The BIOS needs answers to eight S commands no
   document describes; Sapphire gives PCSX2's. *Recommended*: keep them, each measured and its parameters recorded, and
   add others only when a boot or a game shows the need, as this stage did.
2. **Q2 [decision], the EEPROM's layout.** No permitted document gives it; the blank of zeros is the one PCSX2 accepts in
   the same way. *Recommended*: accept zeros as "blank but valid" and drop the plan's synthesis from a layout until a
   permitted source appears.
3. **Q3 [routine], the clock on the language screen.** *Recommended*: a dispute step comparing the drive's RTC answers
   with PCSX2's under a set clock, before S29 offers the console.
4. **Q4 [routine], the intro's bands and light trails.** *Recommended*: hand them to the GS's next stage as defects to
   prove with a program.
5. **Q5 [routine], the field order.** Sapphire's woven frame equals PCSX2's weave with the bottom field first, not the top
   (6,657 pixels apart). *Recommended*: keep S20's parity; check it against a program that marks each field, at S10. *Settled at S10
   (§36.8): PCSX2's "bottom field first" lays the field CSR calls even on the first raster, as Sapphire does; S20's parity
   kept.*

### 35.13 What was read for this stage

`Sapphire_Plan.md` §1.3, §1.4, §4.4, §5.1, §5.6, §5.9, §7 (S10, S24, S25), §8, §11 (Q6, Q7, Q9), §12; this page's §20 to §33
(each unit's record and what it left for S24); `Sapphire_Disputes.md` (D-61, D-124 to D-126, D-141 to D-145, D-266,
D-280, D-288); `Sapphire_Referees.md` §2; `EmuSen_Firmware.md` §0; Diamond's `machine.rs` (the project's own reset of
the R3000A). ps2tek at `1c91660`: "EE RDRAM initialization", "DMAC I/O" (D_ENABLER), "IOP DMA", "CDVD I/O Ports", "CDVD
S Commands", "EE Timers", the I/O maps, "BIOS Boot Process", "BIOS PS Compatibility" (bit 19), "BIOS IOP IOPBOOT",
"SIFMAN", "IGREETING", "EESYNC", "EECONF" (the BIOS chapter, as a writer of a player's-BIOS stage may); the later
edition's index (Israelson, fetched 2026-10-25, byte-identical); Sony's *EE User's Manual* §2.2.1, §5.2 (D_ENABLE),
chapter 7 (§7.1, §7.2.1), *EE Overview* §2.6, §3.1.2, *EE Core User's Manual* (PRId, Count); psx-spx "SPU Status
Register", "Exception Vectors"; Samsung's K4R881869E Direct RDRAM data sheet (2003). PCSX2 2.8.2 run as a black box, its
outputs only: RAM through PINE, and pictures from states it saved. No emulator, SDK, loader or ps2autotests source, and
no firmware: the BIOS images were handed by path to Sapphire and to PCSX2 and never opened by any tool; no program
counter, instruction, or memory holding BIOS code or data was printed, logged or read, and the EE's console text was
never kept.

**Exposure** (§1.4): the writer of this stage ran the player's BIOS on Sapphire and compared its pictures with a referee's,
read ps2tek's BIOS chapter and saw BIOS boot banners quoted there; **this writer is exposed and will never write the open
replacement**.

*Added 2026-10-26 at D-383's step (§30.25 to §30.30):* that step's writer ran the player's BIOS on Sapphire, observing its
pictures and the hardware's own counts, and read ps2tek's BIOS chapter entry for SIO2MAN; **that writer is exposed too
and will never write the open replacement**.

*Added 2026-10-25 at S31 (§41.11):* the writer of S31, the debugger, ran the player's BIOS on Sapphire to compare its
digests with the debugger armed and is a writer of a player's-BIOS stage; that writer is likewise exposed and will never
write the open replacement.

*Added 2026-10-26 at the SIO2out step (§30.31 to §30.37):* that step's writer ran the player's BIOS on Sapphire,
observing its pictures and the register state of SIO2 and of the IOP's DMA channels 11 and 12 at their writes and at each
queue's start and end, never a program counter, instruction, memory content or command byte; **that writer is exposed
too and will never write the open replacement**.

*Added 2026-10-26 at the SIF link step (§30.38 to §30.43):* that step's writer ran the player's BIOS on Sapphire and on
PCSX2, traced on Sapphire the SIF DMA tags and the SIF RPC packets' documented header fields as `Sapphire_Plan.md` §1.4
allows since 2026-10-26, read PCSX2's EE receive buffer through PINE for the same fields, and observed the EE's and the
IOP's units' registers and counts, never a program counter, instruction, payload or memory holding BIOS code; **that
writer is exposed too and will never write the open replacement**.

*Added 2026-10-26 at the pad's SIO2 step (§30.44 to §30.51):* that step's writer ran the player's BIOS on Sapphire and on
PCSX2, traced on Sapphire SIO2's registers and the bytes exchanged with the emulated pads and cards, and read PCSX2's pad
trace log, both as the tester decided on 2026-10-26 (`Sapphire_Plan.md` §1.4); once, before the log's filter, the shape
of every line of PCSX2's default log was printed with its values masked, showing recompiler messages with instruction
mnemonics and the IOP modules' registrations by name (§30.51); **that writer is exposed too and will never write the open
replacement**.

## 36. Stage S10, the EE's ps2autotests on the player's BIOS (2026-10-25)

*Numbered §36 and its disputes from D-360, S25's part B writing in §30 from D-380 beside it.*

### 36.1 In one paragraph

The plan's §7 row S10 and the second half of S24's, on the tester's go of 2026-10-25. **An EE program given as the image
is entered by fast boot** (§36.2): the BIOS boots as at S24, and once the IOP has finished booting and the EE runs a
program in user mode, the machine writes the program into RAM past the caches and calls the kernel's ExecPS2 in place of
that program's next instruction, as ps2tek documents the call; no BIOS code is read, changed or traced. **The IOP's console
is kept in a `tty` space** (§36.3), caught at IOMAN's `write` resolved at run time through the export tables, as decided
on 2026-10-24, and answered as an attached TTY answers for a fast-booted program. Run so, **ps2autotests' 55 EE programs
print 52 of their console texts whole** on the Americas image (§36.5), among them all of S10's gate but one line that is
D-28's named loss, `vu/games/triace`'s twelve values, which settle D-43 (§36.6), and every kernel, DMA, VIF, GS and VU
program but `intstall` (D-363) and `efu` (S13's named loss). Six defects of the machine were found by the texts and fixed
(D-360 to D-362, D-364, D-80 and D-181 amended), the first two in the IOP's DMAC and the cause of a deadlock of the SIF's
RPC. **The 23 IOP programs (`.irx`) are not run** (§36.7): no permitted document gives a way to load one that does not
need a decision. S24's field-order question is settled by a field-marking program on PCSX2 (§36.8). No emulator, SDK or
loader source and no firmware was read; §36.12 records what this writer was exposed to.

### 36.2 The fast boot

[R ps2tek "BIOS Boot Process" (cleared for a writer of a player's-BIOS stage, `Sapphire_Plan.md` §2.2), "BIOS EE
Syscalls", "SIF Registers", "BIOS IOP EESYNC"; A where marked; C the choices]:

- **The documented process.** The EE kernel is copied to RAM and its first program loaded ("The EELOAD module is loaded,
  which in turn loads OSDSYS"); the IOP boots its modules, and EESYNC "calls SIFMAN.sceSifSetSMFlag(0x40000)", "the write
  does indicate that the IOP has finished booting"; ExecPS2 (07h) "clears all of the internal kernel state and creates a
  thread with priority 0 (main thread). The thread begins executing from the entry point".
- **The entry point** (`Machine::at_entry`): SMFLAG's bit 18 set and the EE in user mode at an instruction boundary
  outside a delay slot — a register and the processor's mode, never a program counter. Measured: the first user-mode
  instruction at frame 5, the flag at frame 7, **the entry at frame 8 (0.13 s of console time) on the Americas image and
  frame 7 (0.14 s) on Europe's**, about 2.5 s of wall time [M]. An entry at the first user-mode instruction alone, before
  the flag, also runs the programs [M]; the flag is kept so that the IOP's modules are up as they are when OSDSYS starts
  a program.
- **The loader's part** (`Machine::enter_program`): every dirty data cache line written back and both caches, the UCAB and
  the BTAC emptied (`Ee::flush_caches`), as a loader leaves them before writing code past the caches; each loadable
  segment written into RAM, its bss cleared; then the registers of an ExecPS2 call (v1 7, a0 the entry, a1 `gp` from the
  ELF's MIPS register information when it has one, a2 1, a3 the argument vector) and the SYSCALL exception taken at the
  running program's next instruction. The kernel does the rest.
- **One argument** [C]: argv[0] is `host:program.elf`, written at 01F00000h for the kernel to copy. With argc 0 every
  program stopped 13,446 instructions in, reading an instruction word as a pointer (BadVAddr 3C030011h, the program's own
  code read through a garbage argument) [M]; a loader always passes the program's name, and ps2link, which recorded the
  expected texts, passes a `host:` path.
- **The core ABI**: `create` with an ELF and the market's BIOS sets the program for the fast boot (`v1.rs`); without a BIOS
  the console stays held as before. A disc's boot is S29's.

### 36.3 The IOP's console in a `tty` space

[Decided 2026-10-24, `Sapphire_Plan.md`'s decisions; R ps2tek "BIOS IOP Module Linking", "BIOS IOP IOMAN", "SIF RPC
Commands"; M]:

- **The export.** IOMAN's `write`, export 7 (`int write(int fd, void *ptr, size_t size)`), found through the table named
  `ioman`; a call to it with descriptor 1 is the console's text. The PC compared is the table's function pointer, read
  at run time from IOP RAM, never an address in the image.
- **Finding the table.** A scan of IOP RAM for the magic word 41C00000h found one table at a time and none once the IOP
  had booted [M]: the word does not stay once LOADCORE has registered the table, while the name and the entries do. So
  the IOP's bus notes where a processor store completes the magic word (`IopBus::export_tables`, the newest 256), and the
  console takes the newest noted table whose name is `ioman` and whose entry points into RAM (`Tty::resolve`, at each
  frame's end); a table overwritten is let go and looked for again, so a rebooted IOP's modules are found anew.
- **The capture** (`iop::tty`): while the hook is armed the IOP runs an instruction at a time, each boundary shown to it
  (`Iop::run_watched`); a call's descriptor, buffer and size are taken from a0 to a2 and kept once its first instruction
  has run, an interrupt taken at the boundary entering it again later. The `tty` space (`TTY`, 11, 256 KiB, read-only)
  holds the bytes, NULs left out, as Diamond's TTY does; WiseMan's `TtyReader` reads it by name.
- **The device** (D-364) [C]: on a fast-booted program the console has a TTY attached, as a console running ps2link has:
  a write to standard output is answered with its size, the bytes kept and the caller resumed at its return address, the
  BIOS's stub not run. The retail TTY driver is "stubbed" (ps2tek); it returned 0, and FILEIO, having written the head
  of a misaligned buffer, never fetched the rest, so lines were cut to eight bytes. A disc's game keeps the retail
  console, the capture only observing.
- **What the texts are graded from**: the text from the expected text's first line to its closing line; what follows is
  the BIOS's once the program has exited (`examples/s10.rs`), as `ps2referee.py tty` returns only a program's region.
  The BIOS's own boot text in the space is never printed.

### 36.4 The runner

`examples/s10.rs <bios> <market> <program.elf> <expected> [frames] [out]` boots, enters the program, and grades its text
whole as WiseMan's `ExpectedText` does (every line equal, nothing after); it stops when the closing line arrives, when the
text has not grown for 600 frames, or at the frame limit, and prints the verdict with the frames, console seconds and wall
seconds, the capture's call counts and the IOP's resets. `S10_STATE` adds the DMACs', the SIF's and both INTCs' registers
at the end, the hardware's own report of which unit waits on which; `S10_ALL` and `S10_REPORTS` add S24's register
counts. One program a process, one process at a time.

### 36.5 The 55 EE programs (measured 2026-10-25)

Americas image `ps2-0230a-20080220.bin`, the build of this stage's last commit, one program at a time [M]:

| Folder | Programs | Equal to the console's text | Wall time, each |
|---|---|---|---|
| `cpu/ee` | alu, branch, branchdelay, lsu, lsudelay, muldiv | **6 of 6** | 2.9 to 12.9 s |
| `cpu/ee_cop0` | performance | **1 of 1** | 6.4 s |
| `cpu/ee_fpu` | arithmetic, branch, branchdelay, compare, convert, fcr, muldiv, sqrt, transfer | **8 of 9**: `sqrt` 65 of 66 lines, the one D-28 names | 2.8 to 16.8 s |
| `cpu/ee_simd` | arithmetic, compare, funnel, logic, muldiv, shuffle | **6 of 6** | 5.7 to 19.0 s |
| `memory/ee` | default | **1 of 1** | 2.8 s |
| `cpu/vu0_macro` | integer, transfer | **2 of 2** | 5.8, 13.3 s |
| `dma/dmac`, `dma/spr` | cpcond, tagintr; chain, interleave, normal | **5 of 5** | 2.8 to 6.9 s |
| `dma/vif` | basic, intstall, mskpath3, stcycl, stmod, unpack | **5 of 6**: `intstall` 14 of 18 lines, FQC (D-363) | 4.5 to 11.1 s |
| `gs` | label, signal | **2 of 2** | 4.8, 4.5 s |
| `kernel/ee` | interrupt/dmac_order, interrupt/intc_order, thread/delete, exit, resume, sleep, stat, suspend, terminate, wakeup | **10 of 10** | 2.7 to 13.8 s |
| `vu` | games/triace, lower/branch, lower/efu, lower/fdivdelay, lower/integer, lower/random, upper/clip | **6 of 7**: `efu` 209 of 223 lines, each other one a last-place difference of S13's named loss (§23.4) | 2.7 to 6.4 s |
| **All** | 55 | **52** | **353 s in all** |

**S10's gate** (`cpu/ee/*`, `ee_cop0/performance`, `ee_fpu/*`, `ee_simd/*`, `memory/ee/default`): 22 of 23 programs equal,
the 23rd but the one line D-28 names, on the Americas image; **the same on Europe's** `ps2-0230e-20080220.bin` (entered at
frame 7, 3.3 to 19.5 s each) [M].

**What the texts found, in the order they did** (each a dispute with its cause, and fixed where the documents and the
texts support it):

1. **A deadlock of the SIF's RPC** (D-360). The programs stopped after 14 to 400 lines. The hardware's record of the last
   exchange: the IOP's DMA interrupt handler clears DICR's master enable while it dispatches and sets it again 20 to 70
   IOP cycles later; an EE call packet completing in that window set no flag under D-68's reading (psx-spx's PlayStation:
   the flag needs the master enable), so no IRQ 3 came and the call was never answered. ps2tek, the PlayStation 2's
   document, gates the flag by the channel's mask alone and the master flag by bit 23; with its reading the flag waits and
   IRQ 3 is raised as the enable returns.
2. **A transfer that began before its start** (D-361). The record of the same exchange showed a SIF0 transfer the IOP
   started run from the DMAC's last free cycle, thousands of cycles before the write to CHCR: a register write on an idle
   DMAC did not move its clock. The write now wakes it from its own time.
3. **Lines cut to eight bytes** (D-364): the retail TTY stub's 0, §36.3.
4. **`cpu/ee_fpu/branchdelay`** (D-362): four taken BC1Ts in a row reach the fourth target on the console, BC1TLs and BEQs
   the second; the reading that a taken BC1F or BC1T in a taken one's slot supersedes it gives all three.
5. **`vu/lower/branch`** (D-80 amended): `JALR VI01, VI01` right after `IADDI VI01, VI00, 6` jumps to 6 on the console;
   Sapphire read the value before the write, as D-80 extended to JR and JALR by argument, jumped to 0 and ran the
   program's start again. JR and JALR now read the target as it is; the conditional branches keep D-80's measured rule.
6. **`dma/vif/unpack`'s "Set 4096 bytes" line** (D-181 amended): with its packet read from the source, WL 0 must count as
   256, a filling cycle; `stcycl`'s 171 cases hold unchanged.

The three misses left: `sqrt`'s 1/√25 (D-28's named loss, S9), `efu`'s fourteen last places (S13's named loss), and
`intstall`'s FQC (D-363, open: the console reads 0 with the DMA ended and two qwords past the stall; no document says
where they are kept).

### 36.6 D-43, by `vu/games/triace`

`triace` prints its twelve values equal to the console's text on Sapphire, the VU's FMAC and divider on the FPU's readings
(D-25 to D-28); PCSX2 on IEEE 754 towards zero misses one of them (§23.8). D-43's conclusion is now **measured**; no
float constant changes.

### 36.7 The 23 IOP programs

The `.irx` programs are IOP modules whose `_start` prints and runs the test (their sources, read at S10). The way
S10's row asks for, the documented way an EE program loads an IOP module, is LOADFILE's RPC (ps2tek "BIOS IOP LOADFILE": server
80000006h, function 00h `SifLoadModule`), and it is not enough: no permitted document gives the call's argument
layout (ps2tek: "Newer LOADFILE modules seem to have a different interface. Needs more research"), and the module must
be in a file the IOP can open, which on a retail console without ps2link means a disc in the drive (S29's). The other
documented route, MODLOAD's `LoadModuleBuffer` (0Ah) and `StartModule` (08h) called on the IOP, needs an IOP thread to
call them from, and THBASE's `CreateThread` takes a parameter block no permitted document lays out. So the 23 are not
run (cpu/iop: alu, branch, branchdelay, hilodelay, lsu, lsudelay, muldiv; kernel/iop: eventflag delete and poll,
interrupt register and release, libc sprintf and strtok, messagebox poll, receive and send, thread cancelwakeup, create,
delete, releasewait and stat, vpool create and pallocate), and the way to load them is §36.14 Q1.

### 36.8 The field order (S24's Q5), by `s10-fieldmark`

`tests/s20`'s `fieldmark` (written by `examples/s20.rs` as `s10-fieldmark.elf`): interlaced in FIELD mode, circuit 1 shows
one 64 × 32 buffer in each field CSR's FIELD calls even and another in each it calls odd, switched at every V-sync. On
Sapphire the woven picture's even rasters are the even field's (D-204; `the_fieldmark_program_weaves_the_even_field_on_the_even_rasters`).
PCSX2 2.8.2 ran it by fast boot (`ps2referee.py picture`, new: S24's `menu` capture for a program, the state saved 8 s
after drawing begins; its picture is the display rectangle scaled to 640 × 480) [M]:

| PCSX2's deinterlacing | Set beside the even field first | beside the odd field first |
|---|---|---|
| 3, weave with the bottom field first (S24's match) | **0 pixels differ by more than 32**, mean 0.52 | 291,331 of 307,200 |
| 2, weave with the top field first | 291,306 | **0**, mean 0.52 |

So PCSX2's "bottom field first" lays the field CSR calls even on the first raster, as Sapphire does, and S24's 6,657 pixels
at setting 2 were PCSX2's naming of its own setting: **S20's parity is kept**, the field CSR calls even filling the even
rasters.

### 36.9 Speed

The board runs at 1 to 3 frames a second as at S24; the programs' own work is short, so a program takes 2.7 to 19.5 s of
wall time, 2.5 s of it the boot to the entry, and the 55 together 353 s [M]. Before D-360 and D-361, a program that
deadlocked ran until the runner's idle limit (four minutes each).

### 36.10 Tests and seeded faults (2026-10-25)

**Tests** [M, `cargo test --release`, the crate whole]: **329 of 329**, and **342 of 342** on the tree with S19 beneath, the 55 EE texts unchanged (52 equal); ten new: `iop::tty`'s three (the hook found by its
table and followed, a call kept whole once its first instruction ran, the device's answer), the machine's
`the_fast_boot_enters_the_program_through_execps2_once_the_bios_is_up`, `tests/iop.rs`'s
`a_flag_under_its_mask_survives_the_master_enable_clear_and_raises_irq3_when_set` and
`a_channel_started_on_an_idle_dmac_begins_at_its_write`, `tests/fpu.rs`'s
`a_taken_bc1t_in_a_taken_bc1ts_delay_slot_supersedes_it`, `tests/vif.rs`'s `the_unpack_set_line_holds`,
`tests/vu_micro.rs`'s `jalr_reads_its_target_written_just_before_as_it_now_is`, `tests/pcrtc.rs`'s
`the_fieldmark_program_weaves_the_even_field_on_the_even_rasters`; every S21, S22, S25 and S26 program on the IOP's DMAC
passing with D-360 and D-361. WiseMan's `SapphireTests` (the blast radius's filter) **5 of 5**, on the final tree with S19, S30 and S25's part B beneath (1 min 54 s;
17 min on the tree before S30). The ps2autotests runs of §36.5 need the player's BIOS and are not in CI.

**`SPH2` version 3** (rebased onto S30, §38): the stage's state joins the format. The header gains `Entered` (the frame the
fast boot entered its program, absent while it waits), the EE's block `Bc1Taken` (D-362's slot, kept between two
instructions), and the IOP's block `Tty`: the device, the hook (the table's place and the function's address) and the
export tables the bus noted (a count and 256 slots); the console's text is an output and is not state. A state made after
the entry loads into a console still waiting and does not enter the program again, one made before it waits again
(`the_fast_boots_entry_and_the_consoles_capture_are_state`); the machine keeps the program's image for that. The listing,
534 lines, is pinned against `tests/sph2/version-3.layout`; version 2, never a player's, is not read. The crate on the tree
with S19, S30 and S25's part B beneath: **358 of 358**.

**Seeded faults**, each made in place, the tests named run, and the source restored [M]:

| # | Fault | Caught by |
|---|---|---|
| F1 | the console losing a write's last byte (`Tty::keep`) | `a_call_to_standard_output_is_kept_whole_once_its_first_instruction_ran`, `a_call_to_standard_output_is_answered_by_the_device_with_its_size` |
| F2 | the fast boot entering at the first user-mode instruction, before the IOP has finished booting | `the_fast_boot_enters_the_program_through_execps2_once_the_bios_is_up` |
| F3 | PEXTLW's lanes swapped, GPR[rt]'s word above GPR[rs]'s | `logic_interleave_pack_and_rearrangement_equal_the_manual` (S8's programs); and `cpu/ee_simd/shuffle` on the player's BIOS, failing at its line 280, `pextlw 0, 1` |

### 36.11 Where the plan was wrong, or left open

- **§7's S10 row and §3.1** take the `.irx` programs as loaded "the documented way an EE program loads an IOP module";
  no permitted document gives that way whole (§36.7).
- **§3.3's TTY reader over "the output channels"**: on the player's BIOS the console must also answer as a TTY for the
  texts to print whole (D-364), which no plan line foresaw.
- **D-68's flag rule** followed psx-spx's PlayStation where ps2tek's PlayStation 2 text differs; the BIOS's handler
  decided it (D-360).
- **D-43** waited on `triace` and is measured; **D-80's** extension to JR and JALR and **D-181's** WL 0 were argued and
  are now measured otherwise.

### 36.12 Exposure, and what was observed

**Exposure** (§1.4): this writer ran the player's BIOS on Sapphire and on PCSX2, read ps2tek's BIOS chapter including the
parts excluded for replacement writers (the ROMDIR dump, "BIOS Boot Process", IGREETING's banner example), and met the
BIOS's own console text in the `tty` space (never printed, but one module's line after a program's exit, shown once in a
runner's verdict before the runner cut the text at the program's closing line); **this writer is exposed and will never
write the open replacement**.

**Observed while the BIOS ran**, beyond S24's reports: the EE's operating mode at instruction boundaries and SMFLAG (the
entry), the EE's exception counts, BadVAddr and whether EPC lay inside the test program, the test program's own calls into
the kernel (v1 and a0 to a3, v0 on return) while finding the argument defect, the IOP's SR, I_CTRL, DICR and DICR2 with
their cycles, the SIF channels' DMA tags, the SIF RPC packets crossing them in ps2tek's documented `SifCmdHeader` and
`SifRpc*` layouts, and IOMAN `write`'s arguments. No program counter inside the boot ROM or inside BIOS code in RAM, no
instruction of the BIOS and no BIOS image was read; the packets are an interface's traffic, but they are data the BIOS's
modules made, so §36.14 Q4 asks whether such a trace is within a writer's allowance. These diagnostics are not in the
committed code; the runner keeps only the register state of `S10_STATE`.

### 36.13 Named losses

- **`efu`'s last places** (S13) and **RSQRT.S's 1/√25** (D-28), unchanged.
- **The fast boot's argument**: a program sees argv[0] `host:program.elf`, not the path it came from.
- **The TTY device** (D-364): a fast-booted program's writes to standard output succeed, as on a console with ps2link,
  where a retail console's stub returns 0.
- **The IOP runs an instruction at a time** once the console's hook is armed; its cost is in §36.9's times.

### 36.14 Questions this stage leaves

1. **Q1 [decision], how the 23 `.irx` programs are loaded** (§36.7). *Recommended*: a dispute step that learns LOADFILE's
   `SifLoadModule` arguments on PCSX2 with a project EE program speaking ps2tek's documented SIF RPC, then the modules
   read from a synthetic disc through the drive, the path a game's own modules take; it brings S29's disc in the drive
   forward. The alternative, the machine calling MODLOAD's exports from an IOP thread it borrows, needs no disc but
   reaches into the IOP's threads and loads modules no way a program does.
   *Decided 2026-10-26 by the tester: as recommended; built in §40.*
2. **Q2 [decision], the TTY device for fast-booted programs** (D-364). *Recommended*: keep it, named as a loss: the
   expected texts are a console with a TTY's, and a disc's game keeps the retail stub.
3. **Q3 [routine], `intstall`'s FQC** (D-363). *Recommended*: leave it open, D-186's reading kept, until a program of the
   project's on a referee faithful to the stall can decide where the qwords wait.
4. **Q4 [decision], interface traces while the BIOS runs** (§36.12). *Recommended*: allow a writer of a player's-BIOS
   stage the DMA tags and the documented SIF RPC packets, kept out of commits, the exposure recorded as here.
   *Decided 2026-10-26 by the tester: as recommended; written into the plan's §1.4.*
5. **Q5 [routine], the fast boot's argument**. *Recommended*: keep `host:program.elf`; give the image's own name when the
   frontend passes a path.

### 36.15 What was read for this stage

`Sapphire_Plan.md` §1.4, §2.2, §3, §5.9, §7 (S10, S24), §8, the decisions; this page's §14, §16, §19, §21, §23, §28, §29,
§35; `Sapphire_Disputes.md` D-10, D-28, D-43, D-68, D-80, D-181, D-185 to D-189, D-144; `Sapphire_References.md` §2, §3.
ps2tek at `1c91660`: the BIOS chapter's introduction, "BIOS File Structure", "BIOS Boot Process", "BIOS EE Syscalls" (01h to
15h), "BIOS IOP Module Linking", "BIOS List of IOP Modules", the sections of SYSMEM, LOADCORE, INTRMAN, THREADMAN and THBASE,
IOMAN, MODLOAD, ROMDRV, STDIO, SIFMAN, IGREETING, SIFCMD, REBOOT, LOADFILE, SIFINIT, FILEIO, SECRMAN and EESYNC; "Subsystem
Interface (SIF)" whole; "IOP DMA" (DICR, DICR2, the chain tags); the IOP timers' targets (lines 4160–4181, read while
finding where "IOP Console" begins; its lines 4182–4208, excluded for every writer, were not read). psx-spx "DMA Channels"
(DICR). Sony's *EE Core User's Manual* §6.5 (the UCAB). ps2autotests: the README, the expected texts, the first five lines
of each `common/` file (as at S2), `common-ee.cpp`, `common-iop.c` and `.h`, and of the tests' own sources `cpu/ee/alu.cpp`
(its macros), `cpu/ee/shared.h` (`PRINT_R`), `cpu/ee/branchdelay.cpp` (the BEQ chain), `cpu/ee_fpu/branchdelay.cpp`,
`cpu/iop/alu.c` and `kernel/iop/thread/create.c` (their `_start`), `dma/dmasend.cpp`, `dma/vif/intstall.cpp`,
`dma/vif/unpack.cpp`, `vu/lower/branch.cpp`, `vu/test_runner.cpp` and `.h` and `vu/assemble/block.h` (the JALR and B
emitters). PCSX2 2.8.2 run as a black box, its pictures only. No emulator, SDK or loader source; no firmware opened,
hashed, listed or traced.

### 36.16 What this stage does not cover

- The 23 `.irx` programs (Q1), and with them G1's `cpu/iop/*` and G2's IOP half.
- A disc in the drive and its boot (S29, G10); OSDSYS starting a program; `Exit` back to OSDSYS after a program.
- The processors' state in `SPH2` (S30): a fast-booted console still refuses a state load, as S24's running console does.

## 37. Stage S19, the GS's pixel operations, local-to-local overlap, and the BIOS intro's drawing defects (2026-10-25)

*Numbered §37 and its disputes from D-400, since S10 (§36, from D-360) and S25's part B (§30.12 on, from D-380) are
built beside it.*

### 37.1 In one paragraph

The plan's §7 row S19, on the tester's standing go of 2026-10-25. The GS's pipeline gains, in S17's marked places and
in the manual's order, **the alpha test with AFAIL's four outcomes, the destination alpha test, alpha blending with
FIX and PABE, dithering, COLCLAMP and FBA**, FBMSK and ZMSK after every one of them, and **antialiasing** as far as the
documents define it: AA1 on a line or a triangle replaces alpha by the pixel's coverage by §3.6.1's table, blends, and
writes Z only at full coverage. Eleven programs of the project's, 98,304 graded record words, are equal on Sapphire
to the record of S17's model extended by **`pixel_ops`**, written apart from `src/gs/` with each of the manual's tables
transcribed as a table, which also draws 2,400 generated primitives under generated pixel-operation registers as the
source does. **PCSX2 equals seven programs word for word**: every ATST under every AFAIL, all 81 blend selections with
FIX under both COLCLAMPs in three frame formats, PABE, FBA, the masks, and the two programs that isolate the intro's
defects. It departs in four places, each a dispute: DATE in PSMCT24 (D-401), COLCLAMP 0 after dithering an unblended
pixel (D-404), antialiasing's coverage (D-406), and overlapping local-to-local copies (D-107, closed); and it shows that
**dithering acts only on the 16-bit formats** (D-403), where the documents leave the others undefined. **Both intro
defects S24 saw were alpha blending missing**: switched off alone, it brings back the white bands and the square light
trails, and the intro now draws as PCSX2 draws it (§37.8). Disputes D-400 to D-406. No emulator, SDK or loader source,
no ps2autotests source and no firmware was read; the BIOS was only run, and only its pictures were looked at.

### 37.2 What was built

- **`src/gs/pixel.rs`**: `draw` in the order of figure 3-1 and §3.7: scissoring and SCANMSK, texture mapping and fog,
  antialiasing's alpha, the alpha test (`alpha_passes`) and AFAIL, the destination alpha test (`destination_alpha`), the
  depth test, then `output`: the blend over the frame's pixel (`destination`, Cd and Ad as §3.8.1 treats each format),
  dithering in the 16-bit formats, COLCLAMP, FBA; then format conversion, FBMSK (with RGB_ONLY's alpha byte kept), and
  the frame and Z writes. The frame's pixel is read once, and only when DATE, the blend or a mask needs it; nothing
  allocates.
- **`src/gs/draw.rs`**: `Environment` carries `Blend` (ABE, antialiasing, PABE, the context's ALPHA), DIMX when DTHE is
  set, COLCLAMP and the context's FBA, read at the drawing kick; antialiasing is cleared for points and sprites.
- **`src/gs/raster.rs`**: under AA1, a triangle's pixel's coverage (`coverage`, the 256 points at the centres of its
  sixteenths, in doubled integer coordinates, a pixel whose four corners lie inside taken whole at once) and a line's
  (`line_coverage`, from the DDA's exact minor coordinate); `FULL` (80h) otherwise.
- **`tests/s17/manual.rs`**: the model's `pixel` hands each pixel to **`pixel_ops::run`** (§3.6.1's table as its rows,
  §3.7.2's comparisons as a list indexed by ATST and AFAIL's table of what stays effective, §3.7.3's three tables, §3.8.1's
  selections as lists, the dither matrix decoded into a 4 × 4 array, the clamp and FBA in their words), and its triangle
  rule moves into `holds`, which the coverage count calls at the sub-pixel points.
- **`tests/s19/mod.rs`** and **`tests/pixel.rs`**: the programs, the generated primitives, the intro programs, and the
  assembly check; **`examples/s19.rs`** writes them for the referees.
- **No new cross-component dependency**; `gs::local` and `gs::transfer` are unchanged.

### 37.3 The readings the documents leave open

[R *GS User's Manual* figure 3-1, §3.6, §3.7, §3.8, §3.9 and figures 3-17 and 3-18, and chapter 7's pages for ALPHA,
COLCLAMP, DIMX, DTHE, FBA, PABE, TEST, FRAME and ZBUF; ps2tek "GS Alpha Blending", "GS Tests and Pixel Control"; A]:

- **The order** (figure 3-1, §3.7, figure 3-17). The alpha compared is the pixel's after texture mapping, after fog
  (which changes colour only) and after antialiasing's replacement. A pixel failing the alpha test goes on through the
  destination alpha test and the depth test with what AFAIL leaves effective (D-400); a pixel failing either of those
  writes nothing.
- **AFAIL** (D-400): KEEP writes nothing; FB_ONLY the frame alone; ZB_ONLY the Z buffer alone; RGB_ONLY the frame's
  colour alone, its alpha byte kept, in PSMCT32 (and PSMZ32 named as a frame), FB_ONLY in every other format.
- **DATE** (D-401): bit 7 of A in PSMCT32, the alpha bit in the 16-bit formats, every pixel passing in PSMCT24, as the
  manual's three tables say; the frame's present value is tested.
- **The blend** (D-402): (A - B) × C as a signed product, shifted right by 7 with the floor, D added, nothing clamped;
  RGBA16's Cd its five bits at the top of eight, its Ad 0 or 80h; RGB24's Ad 80h; As written as it is; a reserved
  selection (11) selects as 10. Blending runs when ABE is set, or AA1 on a line or a triangle; PABE then switches it off
  for a pixel whose alpha's MSB is 0 (D-405).
- **Dithering** (D-403): into PSMCT16 and PSMCT16S only; DIMX's DMyx (int 1:2:0 at bits 16y + 4x) added at [Y % 4][X % 4],
  Y and X the window coordinates, after the blend and before the clamp.
- **COLCLAMP** (D-404): 1 clamps the dithered value to 0 to FFh, 0 keeps its low eight bits, for every pixel.
- **FBA** (D-405): bit 7 ORed into the alpha written, after the tests and the blend have used the pixel's own; RGBA16's
  alpha bit is that bit.
- **FBMSK and ZMSK**: after every other operation, a 1 keeping the frame's bit (in RGBA16 packed as S17 packs it); ZMSK
  keeps the Z buffer whatever AFAIL says, the depth test still deciding.
- **Antialiasing** (D-406): AA1 on lines and triangles only (§3.2.2's points and sprites are drawn without it); alpha
  becomes Cov when ABE is 0, or when ABE is 1 and alpha is 80h, and the blend runs; Z is written only at Cov 80h, so never
  for a line. The pixels drawn are the primitive's own. A triangle's Cov is half the number of the 256 points at the
  centres of the pixel's sixteenths that its drawing rule holds; a line's is 80h × (1 - |e|), e the line's offset from
  the pixel's centre along the minor axis at the centre's column, held to 7Fh at most, as §3.6.2 says a line's always is.

### 37.4 Local-to-local overlap (D-107, closed)

S16 copied pixel by pixel in TRXPOS's order and left the reading open. `s19-overlap` copies a 6×4 rectangle of a PSMCT32
buffer of generated data onto itself at six offsets, (1, 0), (-1, 0), (0, 1), (0, -1), (2, 1) and (-1, -2), in each of
the four orders of figure 4-1; then a 9×7 PSMCT16 rectangle at (1, 1) in each order, and a 17×9 PSMT8 one a pixel left
and right in orders 0 and 2. PCSX2's records were compared with three readings [M]: pixel by pixel in TRXPOS's order
(the source's), as a shift (the source read whole before any pixel is written), and pixel by pixel in order 0 whatever
TRXPOS says. **PCSX2 copies as a shift in 23 of the 24 PSMCT32 cases and in all four PSMT8 ones; in TRXPOS's order in one
(order 2 at (2, 1)); and as neither in one PSMCT16 case.** No single rule reproduces it, so it is no witness for the
order. The documents give the four orders "only when transmitting from local buffer to local buffer", where they can
matter only to an overlapping copy, and draw where each starts; the copy therefore runs pixel by pixel in TRXPOS's
order, so that the order a program chooses (2 for a copy to the right, 1 for one downwards) gives the shift, and the
other orders smear. **Argued**, recorded in D-107.

### 37.5 Cost

Not measured for speed, as the stage asks. The pipeline adds per pixel a branch on each test, the blend's three
products, and one frame read when DATE, the blend or a mask needs it; an antialiased triangle's edge pixel costs up to
256 sub-pixel tests. Nothing allocates. The eleven programs and the generated primitives run in 1.0 s on the desktop
in release.

### 37.6 The gate: the programs (measured 2026-10-25)

`tests/s19/` writes each program as an S16 script: S18's registers, DIMX, and a DECAL texture with TCC RGBA over a page
of generated data, so that a sprite reading it 1:1 brings every pixel its own colour and alpha (texel addressing being
exact and equal on PCSX2 since S18); frames and Z buffers filled with generated data; every page drawn read back raw as
PSMCT32. The expected record is S17's model's with `pixel_ops`.

| Program | Graded words | What it covers |
|---|---|---|
| `s19-atest` | 16,384 | every ATST under every AFAIL at AREF 80h; AREF 00h, FFh, 3Ch and C3h with AFAIL turning; texels' alphas and flat alphas at AREF - 1, AREF, AREF + 1, 80h; over PSMZ32 with Z written; in PSMCT32, PSMCT16 and PSMCT24 |
| `s19-date` | 12,288 | DATE off, DATM 0 and 1, each under ZB_ONLY (a pixel the destination test stops writes no Z), DATM 0 with GEQUAL, DATM 1 with FB_ONLY and with RGB_ONLY, in context 2, over a blend by Ad, and reading alpha a first sprite wrote; in PSMCT32, PSMCT16 and PSMCT24 |
| `s19-blend` | 24,576 | the 81 selections of A, B, C and D with FIX from eight values, in PSMCT32 under COLCLAMP 1 and 0, in PSMCT16 and PSMCT24; the reserved selection of each field |
| `s19-pabe` | 4,096 | ABE 0 and 1 with PABE 0 and 1 under the normal and an additive blend, with FBA, flat alphas 7Fh and 80h; PSMCT32 and PSMCT16 |
| `s19-fba` | 4,096 | FBA 0 and 1 in both contexts, with blending, under RGB_ONLY and FB_ONLY, with FBMSK keeping the alpha bit; PSMCT32 and PSMCT16 |
| `s19-dither` | 8,192 | four matrices (§3.9.1's example, all 3, all -4, a generated one) under COLCLAMP 1 and 0 over texels and over flat colours 00h to 03h and FCh to FFh; after a blend; DTHE 0; blends past the range added and subtracted; PSMCT16, PSMCT16S, PSMCT32, PSMCT24 |
| `s19-masks` | 8,192 | generated FBMSKs over a blend, with RGB_ONLY, with FBA, with dithering; ZMSK with ZB_ONLY and with GEQUAL; PSMCT32 and PSMCT16 |
| `s19-overlap` | 10,240 | §37.4's copies in every order |
| `s19-aa` | 4,096 | AA1 triangles and lines under ABE 0 and 1 with alpha 80h and 40h, the same without AA1, an AA1 sprite and point; Z written |
| `s19-glow` | 4,096 | §37.8's light trails: a texture the GS draws as nested squares of rising alpha, drawn as sprites under the additive blend over a dark frame |
| `s19-band` | 2,048 | §37.8's bands: white sprites at alpha 0 under the normal blend over a dark frame, one at 40h, one unblended |

**Tests** [M, `cargo test --release`]: `tests/pixel.rs` **13 of 13**: the programs, each equal to its record word for
word (glow and band in one test, which also shows that the same scripts with ABE cleared, as the pipeline drew before
this stage, give white bands and the texture's squares); `the_programs_run_whole_on_the_interpreter_through_path3_and_vif1`
(each program as the referee runs it, an EE program on the interpreter with the board, its packets through the GIF's
PATH3 and its pages read back through VIF1, every record word graded);
`generated_primitives_under_generated_pixel_operations_draw_as_the_model_draws_them` (eight rounds of 300 primitives of
every kind, flat, Gouraud or textured, ABE and AA1 at random, each under generated TEST, ALPHA, PABE, FBA, DTHE, DIMX,
COLCLAMP, FBMSK and ZMSK, over generated frames in PSMCT32, PSMCT24, PSMCT16 and PSMCT16S with a PSMZ32 buffer: every
pixel and every Z equal between the source and the model); `the_programs_assemble_and_their_records_end_in_the_done_word`.
The crate whole **332 of 332** (S24's 319 and `pixel`'s 13), `pixel`, `draw`, `texture`, `gs` and the lib also under `test-checked`; clippy clean on every target. `SapphireTests` (WiseMan, the blast radius's filter) **5 of 5**.

### 37.7 On the referee (measured 2026-10-25)

`examples/s19.rs` writes each program as an ELF with its expected record and a background record (the same script with
no vertex kicked and no local-to-local copy made); `ps2referee.py ram pcsx2 … --until 1FFFF0=5A99D00E 200000:<words>`
read each from PCSX2 2.8.2 (Americas BIOS, fast boot, software renderer), one process at a time. Only the records were
read. One picture lineage, as the tester decided on 2026-10-25 ("Use one reference for now"); Play! is no witness for
drawing (§27.7).

| Program (words changed by drawing) | PCSX2: words differing | What they are |
|---|---|---|
| `s19-atest` (10,674) | 0 | — every ATST under every AFAIL, the frame and the Z buffer, in PSMCT32, PSMCT16 and PSMCT24 (D-400) |
| `s19-date` (5,074) | 1,920 | all in PSMCT24, where PCSX2 draws no pixel under DATE (D-401); PSMCT32 and PSMCT16 equal |
| `s19-blend` (17,859) | 0 | — every selection with FIX under both COLCLAMPs, Cd and Ad in PSMCT16 and PSMCT24, the reserved selections (D-402) |
| `s19-pabe` (2,304) | 0 | — (D-405) |
| `s19-fba` (2,223) | 0 | — (D-405) |
| `s19-dither` (5,630) | 226 | 113 in each 16-bit page, unblended pixels whose dithered value PCSX2 clamps under COLCLAMP 0 (D-404); the PSMCT32 and PSMCT24 pages equal with nothing added to them (D-403) |
| `s19-masks` (4,058) | 0 | — FBMSK after the blend, RGB_ONLY, FBA and dithering; ZMSK |
| `s19-overlap` (1,073) | 387 | against TRXPOS's order; 60 against a shift (§37.4, D-107) |
| `s19-aa` (498) | 239 | pixels PCSX2 adds outside the AA1 triangles' edges and its two-pixel AA1 lines (D-406); every interior pixel, the primitives without AA1 and the AA1 sprite and point equal |
| `s19-glow` (2,304) | 0 | — |
| `s19-band` (2,048) | 0 | — |

So **PCSX2 is a witness for the alpha test and every AFAIL outcome, the destination alpha test in PSMCT32 and the
16-bit formats, the blend's arithmetic with every selection and FIX, COLCLAMP 1, PABE, FBA, the masks after every
operation, and dithering in the 16-bit formats**; not for DATE in PSMCT24, COLCLAMP 0 after a dither, antialiasing or an
overlapping copy, where its departures follow no rule a document states.

### 37.8 The BIOS intro's two defects (measured 2026-10-25)

S24 saw, in the Sony Computer Entertainment animation, white bands above and below the picture and the light trails drawn
as small textured squares where PCSX2 draws glowing points (§35.11, Q4). Pictures of the intro on Sapphire are an allowed
observation (`Sapphire_Plan.md` §1.4), never a trace of the BIOS running: `examples/s24.rs` ran the Americas image
`ps2-0230a-20080220.bin`, handed to the machine by path, and wrote the PCRTC's pictures at chosen frames; nothing else of
the run was kept. PCSX2's pictures came from `ps2referee.py menu pcsx2 … --for 2, 3, 4 --blank-nvm` (states saved 2, 3 and
4 s after it begins to draw, their screenshots alone kept).

- **Before this stage** (the tree at e229078e5): frames 140 to 260 hold **120 rows of pure white (FFFFFFh), the top 60 and
  the bottom 60 of 448**, and each light is a square of its texture with a small mark in it; the company's name stands on
  a dark box.
- **The feature at fault, by ablation**: the S19 pipeline with ABE ignored alone (a build used for this measurement only)
  brings back all three, the bands' 120 white rows exact (rows 0 to 59 and 388 to 447) and the squares, 203,709 to
  234,260 pixels of frames 140, 180 and 220 differing from the pipeline with blending; with the alpha test ignored alone,
  neither defect returns (3,590 to 4,034 pixels differ, none in the bands). **Both defects were alpha blending missing.**
- **The project's programs that isolate it**: `s19-band` (a white sprite blended by an alpha of 0 leaves the dark frame;
  unblended it is white) and `s19-glow` (a texture whose alpha falls from 80h at the centre to 08h at the rim, drawn under
  the additive blend Cs × As + Cd, is a glow; unblended it is the texture's square). Both are equal on PCSX2 word for
  word (§37.7), and the test shows the scripts with ABE cleared drawing the white band and the square.
- **After**: on the final tree, frames 140, 180 and 220 hold no white pixel at all, every pixel of the band rows 000000h as on PCSX2, the lights points with fading trails, and the name without its box. **Frame 180 (3.0 s after power-on) against PCSX2's picture 3 s after it began to draw**, scaled to 640 × 480 bilinearly and compared per channel as at S24 (§35.8): **261 pixels differ by more than 32 and 65 by more than 64**, against 111,607 and 87,698 for the same frame before this stage; frames 140 and 220 lie further from that moment of the animation (820 and 9,750 by more than 32).

The light trails' alpha and blend in the BIOS's own drawing are not observed (only pictures are); the programs reproduce
the picture's form, and the ablation names the feature.

### 37.9 Seeded faults (2026-10-25)

| # | Fault | Caught by |
|---|---|---|
| F1 | AFAIL FB_ONLY writing Z as well, in `pixel::draw` | `every_alpha_test_under_every_afail_…`, `the_destination_alpha_test_…`, `generated_primitives_under_generated_pixel_operations_…` |
| F2 | The blend's C taken from the source's alpha where FIX is selected, in `pixel::output` | `every_blend_selection_with_fix_…`, `pabe_blends_only_…`, `fba_ors_the_alpha_msb_…`, `dithering_adds_dimx_…`, `generated_primitives_…` |
| F3 | The dither matrix indexed by X % 4 for the row and Y % 4 for the column, in `pixel::output` | `dithering_adds_dimx_…`, `fbmsk_and_zmsk_keep_their_bits_…`, `generated_primitives_…` |

Each was applied, `tests/draw.rs` and `tests/pixel.rs` run, and the source restored [M]; S17's programs do not see them,
as expected (they draw with the pixel operations off). While the stage was built, the model also
caught a defect of the source's own: antialiasing's alpha replaced when ABE was set and alpha was not 80h, the table's rows
inverted (`antialiasing_…`, `generated_primitives_…`).

### 37.10 What is not modelled

- **Drawing time**, as at S17 and S18: every pixel operation is free.
- **Antialiasing's pixels outside the drawing rules**: none are drawn (D-406); a named reading, against PCSX2's.
- **The texture page buffer** (S18's §31.9): a primitive that blends over its own texture reads the pixels it has written.
- **Dithering into PSMCT32 and PSMCT24**, which the documents leave undefined: nothing is added (D-403).

### 37.11 What the plan got wrong, or left open

- **S19's row** lists "local-to-local overlap" as a pixel operation; it is the transfers' (S16), settled here as D-107 asked.
- **The gate's "each picture equal to the referees'"**: as at S17 and S18, PCSX2 alone; equal on the operations the documents
  fix, departing where they leave a choice (antialiasing, dithering's undefined formats) and in three places they do not
  (DATE in PSMCT24, COLCLAMP 0 after a dither, the overlap).
- **S24's Q4** handed the intro's defects to "the GS's next stage" as texturing or drawing defects; both were blending, which
  S17 and S18 had marked as S19's.

### 37.12 Questions this stage leaves

1. **Q1 [decision], antialiasing's coverage (D-406).** The documents give the table and the Z rule but no precision, no line
   width and nothing on pixels outside the drawing rules; PCSX2 draws extra edge pixels and two-pixel lines. *Recommended*:
   keep the documented minimum (the primitive's own pixels, the area at sixteenths, a line one pixel wide), and look again
   if a game at S28 draws with AA1 and its picture shows the edges PCSX2 draws.
2. **Q2 [routine], dithering in PSMCT32 and PSMCT24 (D-403).** *Recommended*: keep PCSX2's measured behaviour, nothing added,
   while the documents call the result "not guaranteed".
3. **Q3 [routine], DATE in PSMCT24 and COLCLAMP 0 after a dither (D-401, D-404).** *Recommended*: keep the manual's words
   against PCSX2's two departures.
4. **Q4 [routine], an overlapping local-to-local copy (D-107).** *Recommended*: keep TRXPOS's order; PCSX2 follows no single
   rule.
5. The second picture lineage stays S17's Q1 [decision], unbuilt, as the tester decided on 2026-10-25.

### 37.13 What was read for this stage

`Sapphire_Plan.md` §1.4, §2, §5.4, §7 (S16 to S20), §8, §10 and §11; this page's §24, §27, §31 and §35; `Sapphire_Referees.md`
§2 and §6; `Sapphire_Disputes.md` D-100 to D-107, D-160 to D-165 and D-240 to D-250. Sony's *GS User's Manual* (Version 5.0)
figure 3-1 with its text, §3.6 to §3.9 whole with figures 3-17 and 3-18, §4.1.2 to §4.1.4 and figure 4-1, and chapter 7's
pages for ALPHA, COLCLAMP, DIMX, DTHE, FBA, PABE, TEST and TRXPOS. ps2tek at `1c91660`: "GS Alpha Blending", "GS Tests and
Pixel Control", "GS Special Effects and Optimizations". No emulator, SDK or loader source, no ps2autotests source and no
firmware: the BIOS image was handed by path to Sapphire and to PCSX2, never opened, hashed or analysed, and of its runs only
pictures were kept.

**Exposure** (§1.4): the writer of this stage ran the player's BIOS on Sapphire and on PCSX2 and compared the intro's
pictures; **this writer is exposed to the BIOS's output and will never write the open replacement**.

### 37.14 What this stage does not cover

- The GIF (S15) is built, so the EE's side now reaches every operation here through PATH3; the programs drive the host
  interface directly, as S17's and S18's do.
- The device path's pixel pipeline (S46), graded against this one.
- The state format's new contents: none; the operations keep no state beyond the registers.

## 38. Stage S30, save states and rewind's cost (2026-10-25)

*Numbered §38 and its disputes from D-420, since S10 (§36, from D-360), S25's part B (§30.12 on, from D-380) and S19
(§37, from D-400) took the numbers before it. The stage raised no dispute: a state format is the project's own choice,
and nothing in it is a reading of the hardware.*

### 38.1 In one paragraph

The plan's §7 row S30, on the tester's go of 2026-10-25. **`SPH2` version 2** holds every unit's state: after a header
(the frame count, the market, the BIOS entry by its declared identity, the image's identity and the EE's stop) comes the
picture, the one block whose length varies, and then every block in a fixed order (`Sapphire_Plan.md` §4.7) — the EE with COP0, COP1, its issue model, the TLB and the caches; RDRAM; the scratchpad; both VUs with their memories
and micro modes; VIF0, VIF1, the GIF; the GS with its 4 MiB, CLUT buffer and every register; the PCRTC; the DMAC, the
INTC, the timers; the IOP with its RAM, cache, scratchpad, DMAC and timers; the SIF; the drive; SIO2 with the pads and
the cards' bus state; the SPU2 with its RAM, voices and effects; the IPU; the memory controller; and the scheduler. **A
state is 43,231,819 bytes (41.2 MiB) for an NTSC console showing 640 × 448, and 43,395,659 for a PAL one showing 640 ×
512**; the plan's estimate of about 41 MiB held. Every queue is written in fixed slots, so a machine's state keeps one
length while its picture keeps one size, which the rewind chain needs (`EmuSen_Rewind_And_FastForward.md` §1.8). The
layout listing is pinned line for line against `tests/sph2/version-2.layout`. **S24's rule refusing every load on a
running console is gone**, and with it the replay of a console with no BIOS: a state loads on a console running the
player's BIOS, mid-frame or at a frame's end, and continues as the console that saved it. **The gate is met**: on both
0230 images, states saved at five frames of the boot to OSDSYS's language screen, each loaded into a console just made,
continued 30 frames with every picture, every sample and the state at the end equal (§38.6); 30 of the project's programs,
each saved at four instants of its run, held to the machine that saved them in state and sound; the kit's C1 to C15
passing. **P14 measured** (§38.7): a capture costs the emulation thread 6.5 ms at the median on the desktop, under a
frame's 16.7 ms, a restore 4.3 ms, and the delta's encoding 4.6 ms on the idle board; what binds is the budget, which at
96 MiB holds 1.5 s of history if a tenth of the state changes between captures. No emulator, SDK or loader source and no
firmware was read; the BIOS image was handed to the machine by path and never opened, hashed or analysed, and of its runs
only whether pictures, sound and states were equal was kept.

### 38.2 The format

- **The header**: `Magic` (`SPH2`), `Version` (2), `Frames`, `Market` (0 Japan, 1 the Americas, 2 Europe), `Bios` (the
  file number of the market's BIOS entry, 16 to 18, when the console runs one, else 0), `Image` (`Media::identity`, FNV-1a
  over what recognition read: a program's entry and segments, or a disc's framing, sector count, BOOT2 program and serial;
  never 0) and `Stop` (whether the EE has stopped on an instruction the interpreter lacks). A load refuses a state of
  another market (`FOREIGN`), another BIOS entry or none where this console has one or the reverse (`STATUS_OTHER_BIOS`,
  -14, in words), another image (`WRONG_MEDIA`), and any version but 2 (`VERSION`).
- **The picture** follows the header: its place and pitch, size, shape and field, and its RGBA rows. It comes first
  because it alone has a length of its own. A load reads the header and the picture's shape, computes the state's whole
  length (the raster's bytes and the blocks' fixed length, `Machine::blocks_len`), and refuses a shorter state as
  truncated and a longer one as foreign **before it changes anything**; past that check nothing can fail, so a load is
  read in place, with no copy of the machine. The kit's C7 holds a refused load to changing nothing (§38.8).
- *Version 3 since S10 (§36.10): the fast boot's entry in the header, the EE's BC1 slot, the IOP's console capture.*
- **The blocks**, each written by its unit beside the unit (`impl emusen_native::State` in each module, the board's
  composition in `board.rs`): every register and every place a unit keeps between two of the board's steps. What one
  instruction's execution keeps for itself and sets again before the next (the EE's stalls, entry, word and counting
  mode), pure memos (the EE's decode memo, the TLB's last lookup, the board's next PCRTC event, all cleared or rebuilt on
  a load) and the hardware's reports (the exception counts, the traffic counts, the drive's command counts, the
  unanswered accesses) are not machine state and are not written. The stand-ins behind DMA channels no unit answers keep
  nothing a program sees. The samples not yet taken by the host are not kept (Diamond's rule, `Diamond_SPU.md` §9).
- **Fixed length.** Every queue is written as a count and fixed slots, the slots past its length zero: the GIF's PATH3 FIFO
  64 qwords (16 are the DMAC's; the EE's stores are not refused when it is full), its XGKICKs 16, each VIF's FIFO 64 qwords,
  the GS's writes held behind a SIGNAL 64, the IPU's FIFOs 16 qwords each, the SIF's FIFOs 64 words each (32 are the
  hardware's), SIO2's three byte FIFOs 2,048 each, the VU pipe's flag writes in flight 16, the drive's sector buffer
  2,352 bytes. Each unit's `fits` says whether its queues are within their slots; a save at an instant where one is not is
  refused in words (`STATUS_QUEUE_PAST_STATE`, -15) rather than written short. No program of the project's comes near any
  bound. Enumerations are written by code; an optional value as a flag and a value; a 128-bit register as one
  (`StateWriter::u128`, §38.9). The listing names each field once, arrays as one line, so version 2's listing is 525 lines.
- **The picture's size** changes only at a field's start, when the read circuits or the mode change; the state's length
  changes with it, and `v1` raises `STATE_SIZE` with the new length after that frame, as MarsRT does for its save chip
  (`EmuSen_CoreAPI.md` §6.9's size contract is "constant for the life of a machine, except across a load that changes the
  machine's configuration"; Sapphire departs from it as MarsRT already does). A fixed raster at the PCRTC's largest
  picture, 1920 × 1125, would have kept the length constant at 8.6 MB more a state, a fifth of it; it was not chosen
  (§38.11, Q2). The rewind chain begins again at such a change.

### 38.3 What a state never holds

- **The BIOS image** (Sapphire_Plan.md §5.9): firmware, never written; the state names the entry by its file number. Two
  retail images of one market cannot be told apart that way, since the core is never told which file the host matched;
  a state of `0230a` loads on `0220a` (§38.11, Q3).
- **The disc and the program's bytes**: named by `Image`, never held.
- **The cards' images and the EEPROM's bytes**: the player's media, which `EmuSen_CoreAPI.md` §6.27 and §6.28 have the host
  keep apart and a load leave as they stood, with no `BATTERY_CHANGED` of the load's own. The core keeps that rule under both
  flags and without them: a card's bus state (terminator, page, offset, the page being gathered, a write in progress, the
  flash's busy time) and the EEPROM's open session are in the state; their contents are not. A console whose BIOS reads its
  EEPROM after a load reads the player's, not the state's; a card's contents are likewise the player's.
- **Which pads and cards are present, the multitap, and the frontend's buttons and sticks**: the machine's settings, files
  and input, held as they stand across a load. A pad's modes, masks, rumble map and motors are in the state.

### 38.4 Versions

Version 1 (S24) held the memories alone; the console has been offered to no player (`development: true`), so no player has
a state of it, and **the oldest version read is 2** (`OLDEST_STATE_VERSION`), as Nephrite's Sega CD's oldest is the first
it was offered with (`Nephrite_Native.md` §53.1). Machine info's `state.loads_from` lists 2 alone. Version 1's listing,
pinned at S24, is retired with its test; a later stage that changes a block raises the version, keeps the older layout's
reader back to the oldest a player has had, and adds the new listing's file beside version 2's.

### 38.5 Loading back

`Machine::load_state` checks the header and the length, writes the picture, reads every block into the machine in place,
sets the frame count and the stop, and clears the samples not yet taken. The board's next event is restored with the
scheduler; the PCRTC's cached next event, the TLB's memo and the units' derived values are rebuilt from the state on their
next use. `Machine::replay_to` and `StateError::Running` (§35.6) are removed: a console with no BIOS and one running a BIOS
load alike. A state taken at any instruction loads exactly (the lib test `a_running_console_loads_mid_frame_…` on a boot ROM
of zeros, written for it, and every program of §38.6 cut between instructions or between the board's steps).

### 38.6 The gate (measured 2026-10-25)

**The player's BIOS** (`examples/s30.rs bios`, the image handed by path; nothing printed but equality and timings). One
console booted the image; at each frame named its state was saved and loaded into a console just made on the same image,
both ran 30 frames, and every frame's picture and sound and the state at the end were compared, and the loaded console
saved again was compared with the state it loaded:

| Image | Frames saved at | Save, load, save | Pictures differing in 30 frames | Sound differing | State 30 frames on |
|---|---|---|---|---|---|
| Americas `0230a` | 60, 300, 600, 700, 760 (the language screen begins at 690) | equal at all five | 0 at all five | 0 at all five | equal at all five |
| Europe `0230e` | 60, 300, 500, 590, 640 (at 50 Hz; the language screen begins at 570) | equal at all five | 0 at all five | 0 at all five | equal at all five |

**The project's programs** (`tests/state.rs`, 11 tests): each run to its done word, then again with its state saved at a
fifth, two fifths, three fifths and four fifths of its run; each state loaded into a machine just made (the cards' images
carried, as a host keeps them, §38.3) and held to the machine that saved it until the next instant, their states and every
sample equal there. DMA chains (`s11-chain`, `callret`, `interleave`, `stall`, `mfifo`); VU microprograms (three of S13's);
VIF's UNPACK, the microprogram it starts and VIF1's double buffer (S14); the GIF's paths, arbitration and intermittent mode
(S15); GS drawing through PATH3 (S17's triangles, Gouraud triangles and sprites); the PCRTC's timing (S20); SIF1, SIF0 and
SIF2 between the EE and the IOP (S22); SIO2's pad, card and multitap (S25); SPU2 voices, reverb, AutoDMA and the IRQ
addresses with their sound (S26); two IPU programs (S27); and the board alone over six PAL fields. All equal [M].

**The kit** (`SapphireTests`, WiseMan): C1 to C15 on the synthetic program, CD and DVD, with C7's eight save points over
120 frames, a refused load changing nothing, save-load-save byte-identical, and C8's load at the middle; the picture's
change of size at the first field announced by `STATE_SIZE`, as C7's watch asks. All pass.

### 38.7 Rewind's cost, P14 (measured 2026-10-25)

`SapphireRewindTests.P14_the_chains_costs_on_a_41_MiB_state` (opt-in, `EMUSEN_SAPPHIRE_P14`; Release build) runs
`RewindBuffer` at its defaults (a capture every 4 frames, a 96 MiB budget) over `CoreEngine` on the synthetic program, the
board idle with no BIOS, the desktop loaded by the other stages' work (load average 3.6 to 5.6):

| Measure | Desktop | Handheld, estimated |
|---|---|---|
| A capture on the emulation thread (the core's `state_save` into the reused stream, and the copy into the spare buffer) | **6.5 ms** median of 60, 59 ms the first | 8.5 to 9.1 ms |
| The frame with the chain on, the idle board (1.16 ms without) | 3.20 ms, 2.0 ms more a frame | — |
| Encoding a delta on the pool thread, one frame apart on the idle board (240 bytes) | 4.6 ms | 6.0 to 6.4 ms |
| A restore (`Apply` and `state_load`) | **4.3 ms** median of 16, 23 ms the most | 5.6 to 6.0 ms |
| The core's own save into a buffer it allocates, and its load (`examples/s30.rs`, on the BIOS's boot) | 17.3 to 22.5 ms; 12.9 to 17.7 ms | — |

The handheld is estimated by the 1.3 to 1.4 times a thread is slower there than on the desktop (`Nephrite_Plan.md` §5.5,
argued and not yet measured, Sapphire_Plan.md §5.12); a capture is mostly copying and its true ratio is the two machines'
memory bandwidth, which no page records, so the estimate is a floor. The delta's size, and with it the budget's reach, is
the change between captures; on states of which a share differs in runs of 64 bytes:

| Share of the state differing | Encode | Delta | Apply | History held by 96 MiB at 4 frames a capture |
|---|---|---|---|---|
| 0.1 % | 4.6 ms | 46 KB | 0.6 ms | 146 s |
| 1 % | 5.8 ms | 453 KB | 0.7 ms | 15 s |
| 10 % | 16.5 ms | 4.5 MB | 7.6 ms | 1.5 s |
| 50 % | 94 ms | 22.3 MB | 12 ms | 0.3 s |
| 100 % | 107 ms | 43.2 MB | 23 ms | 0.2 s |

A game's GS memory alone rewrites its frame and depth buffers every frame, at 640 × 448 in PSMCT32 2.3 MB, about 5 % of the
state before RDRAM's own churn [A]: the reach under the default budget is of the order of a second or two, not the minutes of
the 16-bit consoles (§1.2 there). **Verdict on P14**: its first half does not hold on the desktop: a capture costs 6.5 ms,
under a frame's 16.7 ms (39 % of one frame, 10 % of the four it is spread over), and on the handheld's estimate it still
fits. Its second half holds for another reason than the one predicted: **rewind can be offered as it is for its cost, but
not with the default budget**, which holds seconds; it needs its own budget, of the order of 1 GiB for about 16 s if a tenth
of the state changes, or a longer interval, chosen once a game's change between captures is measured (S28). No new codec was built at this stage.
The deltas of the BIOS's boot were not measured: a delta's size is a measure of memory holding the BIOS's data, which
§1.4 keeps from a writer (§38.11, Q1).

### 38.8 Tests and seeded faults (2026-10-25)

**Tests** [M, `cargo test --release`]: the crate whole, **345 of 345**: the lib 68 (state's four: the listing pinned
against its file, a state round-tripping byte for byte with every refusal changing nothing, a running console loading
mid-frame, a queue past its slots refused), `tests/state.rs` **11 of 11**, new, every other file unchanged and passing; the
lib and `tests/state.rs` also under `test-checked`; clippy clean on every target. `emusen-native` 84 of 84 (its new
`wide_words_read_back_little_endian`), `blackmoon-spu` 6 of 6, Diamond checked (`cargo check --tests`); WiseMan's
`SapphireTests` 5 of 5 and `SapphireRewindTests` 2 of 2 (`Rewind_at_the_default_interval_lands_on_the_state_it_held`, new).
The other crates the blast radius lists depend on `emusen-native` alone, whose change adds two methods to each of the
writer and the reader and changes nothing else; they were not run.

**Seeded faults:**

| # | Fault | Caught by |
|---|---|---|
| F1 | a unit's field left out of the state: the DMAC's next wake, neither written nor read | `gs_drawing_round_trips_mid_run`, `the_gifs_paths_round_trip_mid_run`, `the_ipus_decoding_round_trips_mid_run` |
| F2 | a block read in the wrong order: the drive's before the SIF's | `a_state_round_trips_byte_for_byte_…` (once the test filled those blocks, below), `the_sifs_messages_round_trip_mid_run` |
| F3 | the scheduler's pending event lost: the board's next event not restored | every test of `tests/state.rs`, `a_state_round_trips_…`, `a_running_console_loads_mid_frame_…` |

Each was applied, the tests named run, and the source restored [M]. F2 first passed the lib's round trip, because a console
with no program leaves the SIF's and the drive's blocks at their defaults and a misplaced read of zeros reads zeros; the
test now sets values in the SIF, the INTC, a timer, a DMA channel, VU1 and the SPU2 before it saves, and catches it.

### 38.9 Shared code

- **`emusen-native`**: `StateWriter::u128`, `u128s` and `StateReader::u128`, `u128s`, a 128-bit word little-endian, for
  the EE's registers; nothing else changes and no other core's bytes move.
- **`blackmoon-spu`**: `reverb::RING` made public, so the SPU2's state writes the reverb's rings by the crate's own length
  rather than a copy of it.
- **No new cross-component dependency**: the R3000A's block is `blackmoon-r3000`'s own (`BlackMoon_R3000.md` §4); the voices'
  are written by Sapphire from the voice's public fields, as Diamond's are.

### 38.10 What the plan got wrong, or left open

- **§4.7's list of the state's contents** omits the picture, which a woven frame needs (the other field's rows), and the
  FIFOs' fixed slots; both are here. Its 41 MiB held: 41.2 MiB.
- **§5.10's "pinned by a test from version 1"**: version 1 was pinned at S24 with the memories alone; S30's format is
  version 2, and version 1 is not read (§38.4).
- **P14** predicted a capture over a frame's budget; it is under it, and the budget's reach is what needs the setting
  (§38.7).

### 38.11 Questions this stage leaves

1. **Q1 [decision], the deltas of the BIOS's boot.** The chain's delta sizes on OSDSYS would give the reach of the budget on a
   real workload before S28, but a delta's size is computed over memory holding the BIOS's data. *Recommended*: leave them
   unmeasured and measure the reach on the first bench game at S28; if the tester holds a delta's size to be a timing-like
   figure the rules allow, measure it on the boot then.
2. **Q2 [decision], the picture's raster in the state.** The state's length follows the picture's size, with `STATE_SIZE` raised
   when it changes, against §6.9's size contract, as MarsRT does. *Recommended*: keep it (8.6 MB a state saved, and a size
   change is rare: a mode change); the alternative is a fixed raster at 1920 × 1125.
3. **Q3 [routine], a state on another retail BIOS of the same market.** The core is told the entry, not the file, so it cannot
   refuse a state of `0230a` on `0220a`. *Recommended*: an ABI minor that passes the matched firmware's checksum, which the
   host already computes, to be named in the state; until then, the entry.
4. **Q4 [decision], rewind's budget for the PlayStation 2.** *Recommended*: a per-system budget (or interval) in the rewind
   settings, its default chosen at S28 from a game's measured change; rewind otherwise offered as it is.

### 38.12 What was read for this stage

`Sapphire_Plan.md` §1.4, §4.7, §5.9, §5.10, §5.12, §7 (S30), §13 (P14); this page's §35 and each unit's record of what it
left for S30 (§17 to §34); `EmuSen_Rewind_And_FastForward.md` §1; `EmuSen_CoreAPI.md` §6.9, §6.27, §6.28; the project's own
state code: Diamond's `state.rs` and `spu/state.rs`, Nephrite's `state.rs`, MarsRT's `tell_state_size`, `blackmoon-r3000`'s
`state.rs`, `emusen-native`'s writer and reader, the kit's C7 and C8, `RewindBuffer` and `XorDeltaCodec`. No hardware
document was needed. No emulator, SDK or loader source, no ps2autotests source and no firmware: the BIOS images were handed
by path to Sapphire alone, never opened, hashed or analysed by any tool; of their runs only equality and timings were
printed, never a value, an offset into a memory, a program counter or an instruction.

**Exposure** (§1.4): the writer of this stage ran the player's BIOS on Sapphire and compared its pictures, sound and states,
and is a writer of a player's-BIOS stage; **this writer is exposed and will never write the open replacement**.

### 38.13 What this stage does not cover

- A game's state and its reach under the budget (S28); the debugger's view of the state (S31).
- The snapshot capability (`SNAPSHOT`): Sapphire has no threads, so its state needs no settling and the snapshot would be
  the state.
- S10's ELF loading and S25's part B, beside it: their units' fields are those of this stage's blocks; a field they add is
  a new version.

## 40. The 23 IOP programs of ps2autotests on the player's BIOS, loaded from a synthetic disc (2026-10-26)

*Numbered §40 and its disputes from D-480, S28 writing §39 (from D-460) and D-383's step §30 (from D-440) beside it.
The work S10's §36.14 Q1 asked for, as the tester decided it on 2026-10-26 (as recommended), with §36.14 Q4 decided the
same day and written into the plan's §1.4.*

### 40.1 In one paragraph

ps2autotests' 23 IOP programs (`cpu/iop/*`, `kernel/iop/*`) are IOP modules; a program on a retail console loads one
through LOADFILE's SIF RPC from a file the IOP can open. **A dispute step learned the call's layout on PCSX2** with a
project EE program speaking ps2tek's documented SIF RPC (D-480: the path from byte 8 of a block of 512 bytes, the reply's
first word the module's id). **A synthetic disc** (`ps2discs.py files`) holds SYSTEM.CNF, the project's loader and the
module as `TEST.IRX`; on Sapphire the console is started empty, the disc goes in through the tray as the core ABI's
`MEDIA` puts it, and the loader its SYSTEM.CNF names is entered by S10's fast boot; it binds to LOADFILE and loads
`cdrom0:\TEST.IRX;1`, the IOP reads the module through the drive, and the module prints through S10's `tty` capture.
**All 23 print their console texts whole on both 0230 images** (§40.5). Two defects of the machine were found and
fixed on the way: the drive's interrupt was lost behind a reason raised before the BIOS ran (D-481), and the IOP took
a bus error on a load the console's texts show does not fault (D-482, D-61 amended). A disc of 2,048-byte sectors, which
Sapphire takes for a DVD, is not read by the BIOS's file system as a CD of the same volume is (D-483, open). PCSX2 run
with the same loader and discs prints 19 of the texts whole. No emulator, SDK or loader source and no firmware was read;
§40.8 records what this writer was exposed to.

### 40.2 The loader

[R ps2tek "SIF Registers", "SIF DMA", "SIF RPC Basics", "SIF RPC Structs and Definitions", "SIF RPC Commands", "SIF
RPC System Servers", "BIOS EE Syscalls" 76h-78h, "BIOS IOP LOADFILE"; M; C the choices]

`tests/irx` generates an EE program for the R5900 (assembled by `emusen-mips`, entered at 00100000h), the client side of
the SIF's RPC written from ps2tek's layouts alone:

- **Its steps**: wait for SMFLG's 40000h ("IOP has finished booting"); read SMCOM, the IOP's receive address for SIF1
  (00019600h on Sapphire and on PCSX2); arm SIF0 by SifSetDChain; clear SMFLG's 20000h and send SIFCMD's init packet
  (80000002h, opt 0) naming its receive buffer, 00180000h; wait for 20000h; send the init packet with opt 1; bind to
  80000006h (80000009h) until the end packet (80000008h) names a server; then for each call send the argument block to
  the server's buffer that end packet named and the call packet (8000000Ah: function, the block's size, a reply buffer of
  512 bytes filled with A5h, `rmode` 1) and wait for the end packet whose command field is 8000000Ah. It keeps a record
  from 00190000h and every packet it receives from 00198000h, and writes 5A99D00Eh at 001FFFF0h, for PINE or Sapphire's
  RAM. Every wait is bounded.
- **How a packet leaves.** The first form programmed the EE's channel 6 itself in normal mode, each block led by its IOP
  tag quadword as S22 models it (D-143). It ran on PCSX2 (the dispute step's runs, §40.3); on Sapphire with the player's
  BIOS the first store to D6_MADR (1000C410h) took a TLB modification exception, EPC inside the loader [M, the hardware's
  report]: the kernel's mapping leaves the SIF's channels read-only to a program. The loader therefore queues its packets
  by the kernel's SifSetDma (77h) and waits for them by SifDmaStat (76h, negative once done), passing the transfer list
  through the uncached view (a cached list was read stale by the kernel, the second transfer sent with the first's size,
  [M]).
- **SifSetDma's attribute** is in no document. Its bits were read on the SIF link (§1.4's decision of 2026-10-26): the
  loader sent its init packet with each bit alone and the IOP's chain tag was read as it crossed SIF1 [M]: **04h sets the
  tag's IRQ, 40h its end, 20h takes the data's first quadword as the tag**; 01h, 02h, 08h, 10h and 80h left the tag
  unchanged. A command packet goes with 44h and a payload before it with 0, as the link showed a ps2autotests EE program's
  packets carry them (IRQ and end on a command, neither on a payload). `rmode` 1 is what the same program's call packets
  carried [M, the link]; nothing else of that traffic was taken.
- **The calls** are given as bytes; `module_load(path)` lays one out as D-480 found (`tests/loader.rs` pins it).
  `examples/irx.rs` writes the dispute step's two probes and the loader of one call, `loader`.

### 40.3 The dispute step (D-480)

Run on PCSX2 2.8.2, the Americas BIOS, the probe entered by its fast boot (`-elf`) with the disc in its drive, RAM read
through PINE, its log not read; 1.4 s a run [M]. The first probe put the path at offsets 0 to 64 in steps of 4, then
128, 252 and 256, of a block of zeros: only offset 8 loaded the module (reply word 0 = 19h, the module's id), every other
offset −203. The second: the path at 8 loaded each time whatever the block's words 0 and 1 held (16, all ones, 512),
each load one id higher, the reply's word 1 0; a path naming no file gave −203. PCSX2's SMFLG read 30000h at the
probe's start, its fast boot entering before EESYNC's flag. The entry, `Sapphire_Disputes.md` D-480, was committed
before the loader took the layout, as the plan's rule asks.

### 40.4 The disc, and its way into the drive

- **`ps2discs.py files OUT NAME=FILE …`** (S23's tool, given a mode): an ISO 9660 volume of the root alone, the
  descriptors at 16 and 17, the L and M path tables at 18 and 19, the root at 20, SYSTEM.CNF naming the first file as
  BOOT2, each file in sectors of its own; `OUT.bin` writes it as a CD's raw data track of mode 2 form 1 sectors, an
  `.iso` as 2,048-byte sectors. The loader (`LOADER.ELF`) and the module (`TEST.IRX`) are the files; the program is the
  project's and the module ps2autotests', so no commercial byte is in a disc, and none is kept.
- **On Sapphire** (`examples/s10.rs` given a disc): a machine of `Media::Empty` with the player's BIOS, as the empty start
  makes it; the tray opened, the disc inserted and the tray closed through `Core::media_lid` and `Core::media_insert`,
  G10's `MEDIA`; the program SYSTEM.CNF names read from the disc's root by the runner (`media::Sectors::root_file`) and
  entered by S10's fast boot. A disc's own boot through OSDSYS stays S29's.
- **As a CD** (disk type 12h) the BIOS's file system reads sectors 16, 18 and 20, then the file [M, the drive's own
  report of its N commands]. **As a DVD** (the same volume as an `.iso`, which `cdvd::open` takes for a DVD, 14h) it reads
  sector 16 with the same ReadCd command, then asks for sector 257, past the volume, and the open fails ("open fail name
  \TEST.IRX;1", LOADFILE's reply −203). Which of the drive's answers on a DVD makes it ask so is not shown by the reports;
  D-483 leaves it open and every run of §40.5 uses a CD. *Closed at §40.14: the BIOS reads a DVD's path table at
  257.*

### 40.5 The 23 programs (measured 2026-10-26)

Each on its own disc as a CD, one process at a time, the build of this section's last commit [M]:

| Folder | Programs | Americas `ps2-0230a-20080220.bin` | Europe `ps2-0230e-20080220.bin` | PCSX2 2.8.2, the same loader and disc |
|---|---|---|---|---|
| `cpu/iop` | alu, branch, branchdelay, hilodelay, lsu, lsudelay, muldiv | **7 of 7** | **7 of 7** | 6 of 7: `lsudelay` departs (below) |
| `kernel/iop/eventflag` | delete, poll | **2 of 2** | **2 of 2** | 2 of 2 |
| `kernel/iop/interrupt` | register, release | **2 of 2** | **2 of 2** | 2 of 2 |
| `kernel/iop/libc` | sprintf, strtok | **2 of 2** | **2 of 2** | 2 of 2 |
| `kernel/iop/messagebox` | poll, receive, send | **3 of 3** | **3 of 3** | 2 of 3: `receive` but its trailing spaces |
| `kernel/iop/thread` | cancelwakeup, create, delete, releasewait, stat | **5 of 5** | **5 of 5** | 4 of 5: `create` (below) |
| `kernel/iop/vpool` | create, pallocate | **2 of 2** | **2 of 2** | 1 of 2: `create` (below) |
| **All** | 23 | **23**, 7.6 to 10.5 s each, 201 s in all | **23**, 5.8 to 10.2 s each, 148 s | 19 |

On Sapphire each program was entered at frame 8 (Americas) or 7 (Europe), and its text was whole by frame 30 (0.5 s
of console time), the module loaded, run and its end packet back within a few frames. **G1's `cpu/iop/*` and G2's IOP
half are met on the player's BIOS.** PCSX2 (`ps2referee.py tty … --channel iop --disc`, the module's own lines only)
departs where it is known to: `lsudelay`'s loads read in their delay slot see the new value (`lb 1: 00000037` where the
console has `11223344`, `lw then branch: 1` where it has 2) and the closing line repeats; `thread/create` and
`vpool/create` print "(0)" where the console prints "(~0)", text the test formats itself; `receive`'s lines lose their
trailing space in its log.

Before D-481 and D-482 the runs ended so: every program hung at "loadmodule: fname cdrom0:\TEST.IRX;1" (D-481);
then 16 printed whole and seven (`messagebox/poll`, `receive`, `send`, `thread/delete`, `releasewait`, `stat`,
`vpool/pallocate`) stopped at the line that calls the kernel with the object id ~0 (D-482).

### 40.6 What the texts found

1. **The drive's interrupt** (D-481). The tray opened before the BIOS ran left I_STAT's bit 3 set; S23's line, the OR
   of I_STAT's bits, never fell, so the read's completion made no rise and IRQ 2 never came; the BIOS acknowledges only
   what it handles (one write of 02h in the whole run). ps2tek: "When a CDVD IRQ is raised on I_STAT, this register shows
   the reason for the interrupt". IRQ 2 is now raised at each reason the drive sets (`Cdvd::reason`, taken by the bus at
   once in `sync_cdvd`), the line's level kept in the state as before, so `SPH2` is unchanged.
2. **A load past 8 MiB** (D-482). The kernel's refusal of an unknown id (−407, −410, −411 in the texts) came after one
   IOP load at physical 08000004h, a data bus error under D-61's reading of psx-spx; the console's texts show it does not
   fault. The region from 00800000h to the SIF's 1CFFFFFFh is now on the bus as 1E000000h is (D-346): zero, writes taken
   without effect; `s21-map`'s load at 00800000h reads zero, as PCSX2's does.

### 40.7 Tests and seeded faults (2026-10-26)

**Tests** [M, `cargo test --release`, the crate whole]: **362 of 362**; new: `tests/loader.rs`'s
`the_module_load_puts_the_path_at_byte_8` and `the_loader_is_an_ee_program_carrying_its_argument_blocks`, the drive's
`each_reason_raises_the_interrupt_while_an_earlier_one_is_pending`, the bus's
`the_drives_command_end_reaches_the_intc_with_the_opening_unacknowledged`; `s21-map`'s record changed at one word
(D-482). The 23 programs need the player's BIOS and are not in CI. WiseMan's `SapphireTests`, the blast radius's
filter, **5 of 5** (2 min 10 s).

**Seeded faults**, each made in place, the tests named run, the source restored [M]:

| # | Fault | Caught by |
|---|---|---|
| F1 | the drive raising IRQ 2 only when no reason was pending (`Cdvd::reason`), the line's rise again | `each_reason_raises_the_interrupt_while_an_earlier_one_is_pending`, `the_drives_command_end_reaches_the_intc_with_the_opening_unacknowledged` |
| F2 | the region past 8 MiB a bus error again (`iop::bus::decode`) | `the_memory_map_scratchpad_and_bus_errors_equal_the_documents` |
| F3 | the module's path at byte 12 (`tests/irx`'s `PATH_AT`) | `the_module_load_puts_the_path_at_byte_8` |

### 40.8 Exposure, and what was observed

**Exposure** (§1.4): this writer ran the player's BIOS on Sapphire and on PCSX2, wrote stages run on the player's BIOS,
traced the SIF link as §1.4 now allows, and read the BIOS's own console text in the `tty` space while finding where a
load stopped (the IOP kernel's and modules' banners, LOADFILE's "loadmodule: fname … args … arg", "open fail name …" and
"loadmodule: id …, ret …"); **this writer is exposed and will never write the open replacement**, as §1.4 asks to be
recorded.

**Observed while the BIOS ran**, each through a diagnostic not committed: the SIF DMA tags crossing SIF0 and SIF1 and the
packets they carried, in ps2tek's documented `SifCmdHeader` and `SifRpc*` layouts (a ps2autotests EE program's traffic
once, the loader's thereafter); the EE's exceptions whose EPC lay inside the loader (code, EPC, BadVAddr); the drive's N
commands with their parameters and its registers' state, the IOP DMAC's channel 3, the IOP's exception counts and the
accesses no unit answers (S24's reports); the `tty` text. No program counter inside the boot ROM or inside BIOS code in
RAM, no instruction of the BIOS, no memory holding BIOS code or data beyond the link's packets, and no BIOS image was
read; the images were handed by path to Sapphire and to PCSX2.

### 40.9 Named losses

- ~~**CDs only** (D-483): the IOP programs' discs are CDs; a DVD of the same volume is not read.~~ Lifted at §40.14.
- **The fast boot's loader**: the loader is entered by S10's fast boot from the disc's file, not by OSDSYS's boot of the
  disc (S29); argv[0] is S10's `host:program.elf`.
- **S10's TTY device** (D-364) answers the modules' writes as it answers an EE program's.
- **D-481's argued rule** and **D-482's extent**, each a choice where the documents are silent.

### 40.10 Where the plan was wrong, or left open

- **§7's S10 row and §3.1** took the `.irx` programs as loaded "the documented way"; the layout needed D-480, the
  attribute of SifSetDma the link, and the disc G10's `MEDIA` before S29.
- **S23's interrupt line** (§25.4) and **D-61's bus error past 8 MiB** were readings the console's texts overturned.

### 40.11 Questions this section leaves

1. **Q1 [routine], the DVD (D-483).** *Recommended*: a dispute step at S29, when a disc first boots as a DVD: the drive's
   answers to the BIOS's first reads of a single-layer DVD compared with PCSX2's on the same image (RAM and the drive's
   registers from an EE program), before any change to `cdvd`.
2. **Q2 [routine], D-482's extent.** *Recommended*: keep the whole region past 8 MiB below the SIF on the bus, as the
   documents place nothing there; narrow it only if a console text shows a fault.
3. **Q3 [routine], the 23 runs in CI.** *Recommended*: none, as for S10's 55: they need the player's BIOS; the runner
   and `ps2discs.py` keep them repeatable.

### 40.12 What was read for this section

`Sapphire_Plan.md` §1.4, §3, §7 (S10, S24, S29), §11 Q6; this page's §25, §26, §30.15-§30.24, §35, §36;
`Sapphire_Disputes.md` D-61, D-68, D-142, D-143, D-346, D-360 to D-364, D-383. ps2tek at `1c91660`: "Subsystem Interface
(SIF)" whole, "BIOS EE Syscalls" 76h-78h, "BIOS IOP SIFCMD", "BIOS IOP REBOOT", "BIOS IOP LOADFILE", "CDVD I/O Ports".
ps2autotests: the expected texts of the 23, `common/common-iop.c` and `.h`, and of the tests' own sources `cpu/iop/alu.c`
(its `_start`), `kernel/iop/thread/create.c`, `delete.c`, `releasewait.c`, `stat.c`, `kernel/iop/messagebox/poll.c`,
`receive.c` and `kernel/iop/vpool/pallocate.c` (the lines that pass ~0). PCSX2 2.8.2 run as a black box, RAM through
PINE and the module's own lines of its IOP console. No emulator, SDK or loader source; no firmware opened, hashed,
listed or traced.

### 40.13 What this section does not cover

- A disc's boot through OSDSYS (S29); a DVD's file system, done at §40.14 (D-483).
- Loading a module with arguments, LOADFILE's other functions, and the IOP's reboot into a disc's module image.

### 40.14 A DVD's file system: D-483 closed (2026-10-26)

*The tester's request of 2026-10-26 to settle D-483, since S28's three games are DVDs; disputes from D-520. A dispute
step and a writer's step of a player's-BIOS stage, recorded here after §40's last section.*

**In one paragraph.** The player's BIOS reads a PlayStation 2 DVD's ISO 9660 file system from an L path table at the
fixed sector 257, and a CD's from where the primary volume descriptor names it. §40's synthetic volume put its tables at
18 and 19, as a CD's, so as a DVD the BIOS read sector 257, past the 56-sector image, found zeros and failed the open.
The drive's answers were right throughout: the same ReadCd command and block size as on a CD, sector 16's bytes the same
in both media, and `emusen-disc` serving sector 257 (zeros past the image, the path table on a real disc). PCSX2 runs
the same BIOS the same way once it takes an image for a DVD; it took §40's small `.iso` for a CD, which is why D-480's
runs read it (D-520 records its boundary). Nothing in Sapphire or `emusen-disc` changes. The synthetic discs' DVD form
now lays its tables and root out as the medium's discs do (257, 259, 261) in `ps2discs.py files` and WiseMan's
`SyntheticPs2Disc`; with it **all 23 IOP programs print their console texts from DVDs** on both 0230 images, and from
CDs as before. Jak and Daxter's disc was never affected: its tables are at 257, and on Sapphire its program, entered by
the fast boot, reads 11,406 sectors of 30 files in 20 s of console time with no drive error.

**The drive's traffic on Sapphire (measured 2026-10-26).** The Americas image, the §40 loader with `cpu/iop/alu`, the
drive's N and S commands with their parameters, the blocks it handed to channel 3 and channel 3's registers at each
start recorded by a diagnostic not committed:

| Disc | The drive was asked (ReadCd, one sector unless named) | LOADFILE's reply |
|---|---|---|
| §40's volume as `.bin`, a CD (12h) | 16, 18 (the L table the descriptor names), 20 (the root), then the files, the speed byte 85h | the module's id; `alu` whole |
| the same as `.iso`, a DVD (14h) | 16, then 257; the speed byte 83h | −203 |
| the DVD with its descriptor's L table moved to 56 (a copy there) | 16, 257 | none; the module never read |
| the DVD grown to 600 sectors, tables at 18 | 16, 257 | none; the module never read |
| the DVD with a copy of the L table at 257, the descriptor naming 18; and naming 257 | 16, 257, 20 (the root the table at 257 names), the files | the module's id; `alu` whole |
| Jak and Daxter's `.iso`, the loader given `cdrom0:\DRIVERS\LIBSD.IRX;1` (a Sony library module of the disc's) | 16, 257, 262 (the directory), the module's 13 sectors | 19h, the module's id |
| the same, `cdrom0:\SYSTEM.CNF;1`; a name the directory lacks | 16, 257, 261 and the file; 16, 257, 262 | −201, the file opened but no module; −203 |

Sector 16's block was the same 2,048 bytes in both media (one FNV-1a over each), channel 3 was set alike for each read
(16 blocks of 32 words), and no S command and no GetToc came between the first two reads: the BIOS chooses by the disk
type alone. The three DVD images at hand, Jak and Daxter, Kingdom Hearts and Dark Cloud 2, all name the L table at 257,
its optional copy at 258, the M table at 259 and 260 and the root at 261, with UDF's anchor at 256 [M, their descriptors
read].

**On the referee (measured 2026-10-26).** PCSX2 2.8.2, the Americas image, outputs only (RAM through PINE and the
module's own lines of its IOP console): an EE program reading the drive's disk type register found §40's 56-sector
`.iso` a CD (12h), like the `.bin`; bisected over images of one volume padded with zeros, every image up to 452,849
sectors was a CD and every one from 452,850 a DVD (D-520). On a sparse DVD of 1,000,000 sectors holding §40's volume the
loader's reply was **−203 with the tables at 18** and **19h, the module's id, with a copy of the L table at 257**, where
`alu`'s IOP text equals its console text whole. The candidates the plan named are thereby answered: the disk type
decides which table the BIOS reads; ReadDvd's block (D-121, D-122) is not used by the BIOS's file system, which reads a
DVD with ReadCd; sector 257's address and bytes are served correctly; the disc was single-layer throughout.

**The fix, in the synthetic discs.** `ps2discs.py files OUT.iso` writes the L and M tables at 257 and 259 and the root
at 261, the files after it, as the medium's discs place them (`DVD_LAYOUT`); `OUT.bin` keeps 18, 19 and 20
(`CD_LAYOUT`), its output byte for byte as before (one MD5 over the `alu` disc). WiseMan's `Iso9660Image.Write` takes
the tables' sectors, and `SyntheticPs2Disc.Dvd()` and `DualLayerDvd()` place them at 257 and 259 with the root at 261,
where the root had been at 300; `Cd()` is unchanged. The synthetic DVDs hold no UDF structures, which the BIOS did not
read; that is named as a loss of the fixtures, not of the core.

**Confirmation (measured 2026-10-26).** Each of the 23 IOP programs on its own disc, one process at a time, with this
section's `ps2discs.py`: as a DVD (`.iso`, 14h) on the Americas image **23 of 23**, on the Europe image **23 of 23**; as
a CD (`.bin`) on the Americas image **23 of 23**, as in §40.5 (177, 262 and 191 s of wall time). Jak and Daxter's own
start, its program entered by the fast boot from its `.iso`: 605 N commands in 20 s of console time, all ended without
error, 473 ReadCd and, once the game's program runs, 132 ReadDvd (D-121's observation), 11,406 sectors (10,501 by
ReadCd, 905 by ReadDvd) from 30 of its files, among them its IOP image, its IOP modules and its first levels' data; what
the game does with them is S28's.

**Tests and seeded faults.** `test_ps2discs` (2 tests, new): a DVD's tables and root at 257, 259 and 261 and a CD's at
18, 19 and 20, each table naming the root and the root naming every file at an extent holding its bytes.
`SapphireRunnerTests.Each_synthetic_disc_reads_back_through_the_hosts_reader_and_sapphire_takes_it` now holds the
fixtures' descriptors and tables to the same places. Run [M]: `test_ps2discs` **2 of 2**; WiseMan's
`SapphireRunnerTests`, `SapphireConsoleTests`, `SapphireTests` and `SystemCodeParityTests` (every class that builds a
`SyntheticPs2Disc`) **23 of 23**. Nothing of Sapphire's or `emusen-disc`'s source changed, so their tests were not run
again.

| # | Fault | Caught by |
|---|---|---|
| F1 | `ps2discs.py`'s DVD L table at 258 (`DVD_LAYOUT`) | `test_a_dvds_path_tables_and_root_lie_where_the_bios_reads_a_dvds`; and the player's BIOS on Sapphire, `alu` from the disc it wrote INCOMPLETE |
| F2 | `SyntheticPs2Disc.DvdPathTables` at 18 and 19 again | `Each_synthetic_disc_reads_back_through_the_hosts_reader_and_sapphire_takes_it` ("dvd: the descriptor names its path tables and root") |

Each was made in place, the tests named run, and the file restored [M].

**Exposure, and what was observed.** This writer ran the player's BIOS on Sapphire and on PCSX2 and wrote a
player's-BIOS stage; **this writer is exposed and will never write the open replacement**, as §1.4 asks to be recorded.
Observed, each through a diagnostic not committed: the drive's N and S commands with their parameters, its I_STAT and
error at each command's end, the blocks it handed to channel 3 (their length and an FNV-1a, and bytes 0 to 8 and 132 to
160 of each, the disc's own bytes), and channel 3's CHCR, MADR and BCR when each was written; the loader's record in EE
RAM (its step and its call's reply); the hardware's reports of accesses no unit answers. On PCSX2 only RAM through PINE
and the module's lines of its IOP console, which `ps2referee.py tty` cuts from the log unread. The SIF link was not
traced; no `tty` text of the BIOS was read; no program counter, instruction or memory of the BIOS, and no BIOS image,
was read. Jak and Daxter's image was read for its descriptors and directories alone (§40.14's layout facts and which
file each read reached); none of its bytes is in a commit.

**Named losses.** The synthetic DVDs hold no UDF structures and no optional path tables, which the BIOS did not read;
the BIOS's choice by disk type is observed, not explained, and only for the first table it reads; D-520's referee
boundary is not followed, so a small PlayStation 2 volume kept as `.iso` is a DVD on Sapphire and a CD on PCSX2.

**Where the plan was wrong, or left open.** §40.11 Q1 put the DVD's dispute step at S29 and expected a difference in the
drive's answers; the difference was the BIOS's own reading of a DVD, shown by the drive's traffic alone. The candidates
named for the dispute (the disk type, ReadDvd's block, sector addressing past the descriptor, the layer, the read mode,
`emusen-disc`'s sector 257) are each answered in the referee paragraph above.

**Questions this section leaves.**
1. **Q1 [routine], D-520.** *Recommended*: keep §25.2's rule (an `.iso` is a DVD) and change nothing; the referee's
   boundary is an emulator's choice, and §25.11 Q4 already covers a CD title kept as `.iso`. A referee run that needs a
   small synthetic `.iso` taken as a DVD on PCSX2 pads it past 452,849 sectors, sparse.
2. **Q2 [routine], UDF in the synthetic DVDs.** *Recommended*: none until a program or a game's module is seen reading
   UDF's descriptors through the drive; the BIOS reads the ISO 9660 side.

**What was read.** `Sapphire_Plan.md` §1.3, §1.4, §5.6, §5.9, §9.4; this page's §25, §30.15 to §30.24 and §40;
`EmuSen_Disc.md` §14; `Sapphire_Disputes.md` D-120 to D-128 and D-480 to D-483. ps2tek at `1c91660`: "CDVD I/O Ports",
"CDVD N Commands", "CDVD Reads and Seeks". ECMA-119 §8.4 and §9.4. PCSX2 2.8.2 run as a black box. No emulator, SDK or
loader source; no firmware opened, hashed, listed or traced.

## 41. Stage S31, the debugger (2026-10-25)

*Numbered §41 and its disputes from D-500, since S28 (§39, from D-460), the IOP-tests step (§40, from D-480) and the
D-383 step (§30, from D-440) were begun beside it. The stage raised no dispute: a debugger reads the machine and is the
project's own design, and nothing in it is a reading of the hardware.*

### 41.1 In one paragraph

The plan's §7 row S31, on the tester's go of 2026-10-25. **Sapphire claims `DEBUG`, `DEBUG_STACK`, `DEBUG_REGISTERS`,
`DEBUG_DISASSEMBLE` and `DEBUG_READS`** (`EmuSen_CoreAPI.md` §6.14), with the plan's four processors (§4.7): the R5900
(processor 0), the IOP (1), VU0 (2) and VU1 (3). Machine info lists every register at its width, a 128-bit general
register as its two 64-bit halves, since the core ABI's register value is 64 bits (§41.3); each processor names the
space its code is listed from. Listings come from `emusen-mips`'s tables: the R5900's, the R3000A's, and the VUs' upper and
lower tables in micro mode. The EE is observed by a frame of its own (`debugger.rs`) around its interpreter's step, which
is untouched: the shared hooks are asked before each instruction, and what the instruction did (its stores and loads,
a call, a return, an exception) is read after it (`ee/observe.rs`). The IOP and the VUs carry a probe beside their units
while the debugger fits one, and the probe stops its processor in front of a breakpoint only where the board brings it
to time at an EE instruction's end, which makes every stop and resume exact (§41.4). **The gate is met**: each claim a
test (§41.7); every table armed, and every processor stopped in front of each of its steps and resumed, give the plain
run's picture, sound and state after every frame on fourteen of the project's programs; and on the OSDSYS boot of the
Americas 0230 image every table armed and the debugger attached unarmed gave the plain run's pictures and sound over 760
frames and its state at all 13 comparisons, digests only (§41.8). Attached and unarmed, the debugger's frame costs 16 per
cent over the plain frame on the boot and 26 per cent on a dense EE loop; the plain frame itself is unchanged (§41.8).

### 41.2 The four processors

| Id | Processor | `pc_bits` | Code space | Registers | Breakpoints and stops |
|---|---|---|---|---|---|
| 0 | `R5900` | 32 | `RAM` | 152 (§41.3) | the shared hooks: execution, `EACH`, the depth targets, data and read data breakpoints, interrupts |
| 1 | `IOP` | 32 | `IOPRAM` | 45, `blackmoon-r3000`'s `DEBUG_REGISTERS` | its probe: execution |
| 2 | `VU0` | 12 | `VU0MICRO` | 157 (§41.3) | its probe: execution, by byte address in micro memory |
| 3 | `VU1` | 14 | `VU1MICRO` | 157 | the same |

A VU's address is its pair's byte address in its micro memory (the pair index times eight), so that a breakpoint, a
coverage bit and a listing all name the same number and the host's `code_space` lists from it. VU0 in macro mode is the
EE's COP2 instructions and belongs to processor 0; processor 2 is VU0's micro mode. Coverage is kept over 24 bits of
address for the EE and the IOP, the host registry's width (`CoverageRegistry`), so that kseg0's, kseg1's and kuseg's
mirrors share a bit as on Diamond (`Diamond_Debugger.md`), and over 12 and 14 bits for the VUs, their memories' sizes.

### 41.3 Registers at their widths, and the 128-bit general registers

The core ABI carries each register as one `int64_t` (`emusen_core_debug_registers`), and the host takes one value for
each register machine info lists (`CoreDebugTarget`). The plan's §4.3 and §10 left open whether a 128-bit value fits;
read at this stage, it does not. Three forms were weighed:

1. **The halves, as registers of their own** (built): `at` is bits 63..0 and `at.hi` bits 127..64, for each of the 32
   general registers; HI and LO the same, their upper halves being pipeline 1's registers and named as the manual names
   them, `hi1` and `lo1`. Every value is at most 64 bits, so every host and the kit read it unchanged.
2. **A minor of the ABI** in which a register whose `bits` exceeds 64 takes ⌈bits/64⌉ consecutive values, low first, with
   the host told by `host_abi_version` (a core lists halves to an older host). It would show `at` as one row of 128 bits,
   at the cost of a change to the header, `emusen-native`, the C# host, the baseline and the kit (§41.10 Q1).
3. **The words** (four 32-bit rows a register): the MMI's lanes, but four times the rows, and not the register's width.

The EE's list, in order: the 32 registers each with its `.hi`, then `pc`, `hi`, `hi1`, `lo`, `lo1`, `sa`; COP0's 32
registers by `emusen-mips`' `r5900::COP0_NAMES` (a reserved one `$n`); the debug registers BPC, IAB, IABM, DAB, DABM,
DVB and DVBM; PCCR, PCR0 and PCR1; COP1's `f0` to `f31`, its accumulator `ACC` and `FCR31`: 152. A VU's, in order: each
vector register's four fields `vfN.x` to `vfN.w` at 32 bits (a field is the VU's own unit of a register, VU User's Manual
§3.1.1), `acc.x` to `acc.w`, `vi0` to `vi15` at 16 bits, `I`, `Q`, `P` at 32, `R` at 23, `status` at 12, `mac` at 16,
`clip` at 24, `TPC` at 16 and `PC` at the memory's width: 157. Reading changes nothing.

### 41.4 The frame, breakpoints and stepping

**The EE.** `Machine::run_frame_debug` runs the frame as `Machine::advance` runs it, step for step: the fast boot's check
and entry while a program waits, then the EE in batches of 1,024 instructions between checks of the PCRTC's field, the
batch's position kept across a stop, since the plain frame ends only at a batch's end. Before each instruction the shared
hooks are asked (`Hooks::stop_before`), and with read data breakpoints armed the bytes the instruction would load are
found from the word and the base register (`Ee::reach`) and offered to `note_read_ahead`; a stop leaves the instruction
unrun and the frame open, and `run::UNCHECKED` runs it unasked. After the step, the exception counts the core keeps for
the hardware's reports tell whether an exception or an interrupt was taken in the instruction's place (noted as a call
of kind `IRQ`, `BRK` for SYSCALL and BREAK, `NMI` for a level 2 exception, abandoning a call whose delay slot it
interrupted); otherwise JAL, JALR and a taken BLTZAL or BGEZAL (and their likely forms) are calls, `jr ra` a return once
its delay slot has run, ERET a return. Nothing of the interpreter changes: what it did is read from what it left.

**The IOP and the VUs.** Each carries a `debugger::Probe` while the debugger fits one (at a frame's start, for a
processor with breakpoints or coverage; the plain frame removes them): the processor's breakpoints and coverage, where
it stopped, and whether that step is to run unasked. The IOP's is asked through `blackmoon-r3000`'s observer
(`step_observed`), one instruction at a time while fitted; a VU's by `Pipe::step` before a pair runs, after its timing
has said the pair is due. **A probe stops its processor only where the board brings it to time at an EE instruction's
end** (`Iop::catch_up`, `Vpu::catch_up`, called from `Board::run_to`'s tail), and nowhere else. Elsewhere — an EE
instruction that waits on VU0 (a macro instruction after VCALLMS), reads a unit (CFC2, BC2F) or a VIF that waits for a
unit, or the IOP brought to a DMA event — the processor is needed now, so it runs on, and a breakpoint met there is noted
as passed: the frame stops after that EE instruction with the processor named, standing past the address. A stopped
processor is behind the board; when resumed, the board is brought again to its own time (`Board::run_to` at `now`), which
replays the tail of the EE instruction's run in the board's own order, and the EE's lines are set from it as
`System::step` sets them. A run the IOP's probe stopped resumes straight to the horizon it had, without the unit service
the plain run did not make there (the first form, which serviced the units again on every resume, left IOP RAM and every
register equal and the bus's sync time eight cycles later, found by the gate, §41.7).

**Why only there.** The first form stopped the VUs and the IOP anywhere. Stepped through VU0 on `s13-integer`, the frame
went on with the EE's VNOP after VCALLMS finding VU0 held at its first pair, where the plain run had run the whole
microprogram inside that instruction: the EE's timing, VU0's registers and every clock after differed (§41.9, fault 4).
A stop inside an EE instruction cannot be resumed as though the instruction had not been interrupted, so the probe does
not stop there.

**A held EE.** With no BIOS, or once the interpreter has stopped at an instruction it does not implement, the EE runs
nothing and the board steps alone (`Machine::advance`'s idle step). The frame then shows the EE standing in front of its
instruction: the hooks are asked before each step of the board, so a breakpoint at the reset vector stops there and
`EACH` stops before each step, which moves the board's time and nothing of the EE. The kit's C15, which runs Sapphire on
its synthetic program without a BIOS, holds the core to this (§41.7).

**Stepping.** The EE steps by `EACH` and by the depth targets (step over, step out). Another processor steps by a
breakpoint over every address: each `UNCHECKED` call runs one instruction or pair and stops in front of the next, until
that processor reaches the board's time, and the EE then runs on.

### 41.5 Stores, loads and the watches

The EE's loads and stores are the accesses the watches see (as on Diamond, not DMA's, a VU's or the IOP's): each byte, in
`RAM` or `SCRATCH` at the offset it lands at, by the EE's own placing with no exception raised and no event counted
(`Ee::place`), with the value as the core sees it, a valid data cache line over RDRAM (`Ee::peek_byte`); LQ and SQ as
sixteen bytes at the quadword below, LWL, LWR, LDL, LDR and their stores as the bytes of the unit they reach, LQC2,
SQC2, LWC1 and SWC1 included, CACHE and PREF not. A store is logged after it lands, a load after it reads; a read data
breakpoint stops in front of the load with nothing of it run. Machine info flags `RAM` and `SCRATCH` with
`reports_stores` and `reports_loads`. A load that faults may stop and then not read, as on Diamond.

### 41.6 Coverage, the profile and the call stack

Coverage marks each address a processor ran, the EE's through the hooks before its step, the others' through their probes
after the EE's step; the profile charges the EE's instructions to the innermost call. The call stack follows §41.4's calls
and returns; `DEBUG_STACK` pushes the host's registry down as the shared hooks take it.

### 41.7 The gate (measured 2026-10-25)

| Claim | Test | Result |
|---|---|---|
| Registers read back as set, at their widths | `debugger::tests::the_registers_read_back_as_set`: both halves of a 128-bit register, HI1 and LO1, SA, COP0, the debug registers, COP1 and ACC; the IOP's; a VU's fields, integers and specials | passes |
| Each processor's listing against its table | `each_processors_listing_is_its_table`: every word of an EE program against `r5900::decode` and by hand, a JAL's reference; the IOP's by hand; VU pairs as `upper \| lower` by hand, a BAL's reference; a space a processor runs nothing from refused | passes |
| Each breakpoint kind at its address and only there | `an_ee_breakpoint_fires_at_its_address_and_only_there` (in front, nothing run, asked again the same stop, one never reached never stops); `the_watches_see_the_ees_stores_and_loads` (watches in RAM and the scratchpad, SQ's sixteen bytes, LQ's reads, a data breakpoint after the SB, a read data breakpoint in front of the LW); `the_iop_and_vu1_stop_at_their_breakpoints_and_step` | pass |
| Single steps | `each_steps_the_ee_one_instruction_at_a_time` (through a JAL, its slot and the routine); `the_call_stack_follows_jal_and_jr` (the depth target steps over); the IOP and VU1 by a breakpoint over every address | pass |
| Coverage counts on a known program | `coverage_records_what_each_processor_ran`: the bits of the EE's loop and routine, the IOP's loop and VU1's three pairs, and the steps counted equal to the passes each processor's counter made, within one pass | passes |
| A state loaded at a stop | `a_state_loaded_at_a_stop_starts_a_whole_frame` | passes |
| A held EE | `a_held_ee_stands_in_front_of_its_reset_vector`; the kit's C15 through `SapphireTests` (a breakpoint at the next instruction, eight steps under `EACH`, the read range kinds, every table armed against the plain run) | pass |
| Every table armed gives the plain run's digests | `tests/debugger.rs`: fourteen programs (`s11-chain`, the first IPU program, `s13-integer`, the second S13 program, `s14`'s microprogram and unpack, `s15-paths`, `s17`'s triangles, `s20-timing`, `s26`'s voices on the IOP, `s22-sif1`, `-sif0`, `-sif2` with the IOP released at a frame's start), 3 or 4 frames each, every table armed (calls, stores, loads, interrupts, the profile, four processors' coverage, a breakpoint on each processor at no step's address, watches over RAM and the scratchpad, data and read data breakpoints) | picture, sound and state equal after every frame |
| … and every processor stopped at every step | the same programs but the IPU's, `EACH` on the EE and a breakpoint over every address on the IOP, VU0 and VU1, each stop resumed | equal after every frame, over 100,000 stops each |

The crate's whole suite passed with the stage (375 tests in 26 binaries, measured 2026-10-25; 381 in 27 on the tree rebased onto D-383's step and §40, 2026-10-26), `blackmoon-r3000`'s suite and
Diamond's debugger tests with the static reference moved (§41.10), and WiseMan's `SapphireTests`, `SapphireConsoleTests`,
`SapphireRewindTests`, `SapphireRunnerTests` and `DiamondDebugTests` (30 tests), the kit's C1 to C24 among them.

### 41.8 The OSDSYS boot, digests only, and the cost (measured 2026-10-25)

`examples/s31.rs bios` starts three consoles on the player's BIOS, handed by path: one plain, one through the debugger's
frame with nothing armed, one with every table armed as in §41.7, each stop resumed at once and every log and coverage map
cleared unread. Only equality and times are printed.

| Image | Frames | Attached, unarmed | Every table armed |
|---|---|---|---|
| Americas `0230a` | 760 (past the language screen at 690) | pictures 0 of 760 differing, sound 0, states 0 of 13 | pictures 0, sound 0, states 0 of 13 |
| The same, on the tree rebased onto D-383's step and §40 (2026-10-26) | 760 | pictures 0, sound 0, states 0 of 13 | pictures 0, sound 0, states 0 of 13; 625.9, 720.2 (+15.1 %) and 891.8 ms (+42.5 %) a frame |

| Workload | Plain frame | Attached, unarmed | Every table armed |
|---|---|---|---|
| The OSDSYS boot, 760 frames, the three consoles interleaved frame by frame | 705.9 ms | 820.3 ms (+16.2 %) | 1,005.6 ms (+42.5 %) |
| A dense EE loop of loads, stores and calls (`s31 cost`), 120 frames | 397.7 ms | 499.0 ms (+25.5 %) | 945.9 ms (+137.9 %) |

The boot's frames were timed while the crate's tests ran beside them, so the absolute times are high; the three consoles
share the load frame by frame, so the ratios stand. The unarmed cost is the frame's own loop (a check of the hooks and
the probes after every EE instruction, the board's catch-up flags); a host runs the plain frame while nothing is armed,
so a player pays it only with a table armed. **The plain frame**: S31 adds to it a test of the IOP's probe in each run of
the IOP, a test of a VU's probe before each pair, and the catch-up flags. Measured against the commit S31 began from
(`67435c5d8`, built apart under `~/.cache/emusen/` and removed after), plain frames alone, the two builds run in turn twice
on a quiet machine:

| Plain frames | Before S31 | With S31 |
|---|---|---|
| The OSDSYS boot's first 300 frames | 905.2 and 890.7 ms (mean 898.0) | 876.0 and 889.6 ms (mean 882.8) |
| The dense EE loop, 30 frames | 320.6 and 335.8 ms (mean 328.2) | 317.4 and 324.3 ms (mean 320.8) |

The plain frame is unchanged within the runs' spread.

### 41.9 Seeded faults (2026-10-25)

| Fault | Caught by |
|---|---|
| 1. A 128-bit register's high half dropped (`.hi` read as zero) | `the_registers_read_back_as_set` |
| 2. An EE breakpoint firing one instruction late (the stop taken after the instruction ran) | `an_ee_breakpoint_fires_at_its_address_and_only_there` |
| 3. VU lower words listed from the upper table | `each_processors_listing_is_its_table` |
| 4. A VU's probe stopping it anywhere, not only at the board's catch-up | `vu_microprograms_and_vif_are_unchanged_by_the_debugger` (`s13-integer`, stepped: the EE's timing, VU0's registers and every clock after differ) |
| 5. The IOP's probe stopping it anywhere | **survived**: on every program of the gate the IOP's lag never reaches the EE's side, the SIF's traffic waiting on its FIFOs' flow control. The rule stands on the argument of §41.4; a program that makes the IOP's lag visible at a DMA event is owed (§41.10 Q3) |

### 41.10 Shared code, and questions for the tester

- **`blackmoon_r3000::reference`**: the static reference of an R3000A word (a JAL's or BxxZAL's target as a call, a load
  or store based on `zero` as a read or a write), moved from Diamond's `debugger.rs`, which now re-exports it, so that the
  IOP's listing and Diamond's share one owner. No new dependency: both cores already depend on the crate.
- **The probe hooks** are additive: `Micro::pc` and `Micro::probe` with defaults, a probe field beside `Pipe` and `Iop`,
  `Vpu::catch_up` and `Iop::catch_up` in `Board::run_to`'s tail, a debugger field in `Machine`, and the frame closed when a
  state loads. S30's state is untouched: no probe, flag or debugger field is in `SPH2`.

1. **Q1 [decision], 128-bit registers in `DEBUG_REGISTERS`.** *Recommended*: keep the halves (§41.3, form 1), which every
   host reads today; propose the minor (form 2) only when a view needs a register of 128 bits in one row.
2. **Q2 [decision], breakpoints of the IOP and the VUs met where they cannot stop.** Such a stop comes after the EE
   instruction, the processor past the address (§41.4). *Recommended*: keep it, since it keeps every stop exact; the
   alternative, stopping in place, would let a resumed run differ from the plain one.
3. **Q3 [routine], a program for fault 5.** *Recommended*: an IOP program polling a SIF register while the EE's DMA sends it
   packets at a known instant, added to the gate at the IOP-tests step.
4. **Q4 [routine], the watches.** The EE's loads and stores only. *Recommended*: keep; the IOP's, in `IOPRAM`, are an
   extension for when a game's IOP module is debugged (S28 on).

### 41.11 What was read for this stage

`Sapphire_Plan.md` §1.4, §4.3, §4.7, §5.9, §7 (S31), §10; this page's §2.6, §35, §36 and §38; `EmuSen_CoreAPI.md` §6.14,
§6.14.1, §6.33, §6.38; `emusen-native`'s `debug.rs` and `core/exports.rs`; the project's own debuggers: Diamond's
`debugger.rs`, VenusRT's `debugger.rs` (its probes), MarsRT's hooks; `EmuSen_Mips.md` for the R5900, R3000 and VU tables
and `emusen-mips`' `r5900.rs` and `vu.rs`; `blackmoon-r3000`'s observer; the C# host's `CoreDebugTarget` and
`CoverageRegistry` for the register value's width and coverage's. No hardware document beyond the VU User's Manual §3.1
for the registers' widths. No emulator, SDK or loader source and no ps2autotests source. The BIOS image was handed by path
to Sapphire alone, never opened, hashed, listed or analysed; on its boot the debugger's tables were armed and cleared
unread, and only equality and times were printed, never an address, a value, a program counter or an instruction. Every
test that shows, steps or lists code runs the project's own programs.

**Exposure** (§1.4): the writer of this stage ran the player's BIOS on Sapphire and compared its pictures, sound and
states, and is a writer of a player's-BIOS stage; **this writer is exposed and will never write the open replacement**.

### 41.12 What this stage does not cover

- The console-shaped views (the GS's registers, the DMAC's channels, the VIFs) as `units` (`EmuSen_CoreAPI.md` §6.33): an
  extension, owed to the frontends with S29.
- `DEBUG_TRANSLATE`: the host lists the EE's code from `RAM` at kuseg's offsets; a translation through the TLB is a
  later step.
- DianaSh's commands over Sapphire on the player's BIOS, which WiseMan cannot run without one; the kit's checks of the
  debug exports run on the synthetic program (`SapphireTests`).

## 42. Stage S35, `emusen-jit`: the shared JIT crate extracted from MarsRT (2026-10-25)

*Numbered §42, after S31's §41. The stage raised no dispute: it moves
code between two of the project's crates and reads nothing of the hardware.*

### 42.1 In one paragraph

The plan's §7 row S35 and §9.6, with the tester's leave of §11 Q10 (decided 2026-10-24, as recommended), placed by
the order decided 2026-10-26 as the first speed stage that needs no Sapphire bench. **A new crate,
`EmuSen/Cores/Shared/emusen-jit`** (`Hardware/Shared/EmuSen_Jit.md`), holds what §9.6 lists and nothing more: the
compiler's thread and its queue, the publication slots (one word, stored with Release and read with Acquire), the
Cranelift module with its W^X batches and its code budget, the counters, and the shape of the verifier that steps an
interpreter beside the blocks; it is generic over a `Lowering` trait that turns a block's words into Cranelift IR and
names its exits. **MarsRT keeps its emitter, its block shapes and its dispatcher** and now uses the crate, the new
cross-component dependency `marsrt -> emusen-jit`. The crate's ten tests run a toy processor's lowering through the
thread, publication, the W^X batches, the budget and the verifier; ThreadSanitizer finds the publication made Relaxed.
**The gate is met**: MarsRT's block tests, its corpus and its game runs identical at every tier, and its speed
unchanged within its noise (§42.4). No Sapphire emitter is written here; that is S36 on.

### 42.2 What moved, and what stayed

| From MarsRT | To | Lines |
| --- | --- | --- |
| `cpu/blocks/jit/mod.rs`: `Slot`, `FAILED`, `Counters`, the queue, `Compiler`, the thread's loop, the module's flags | `emusen-jit/src/lib.rs`, generic over `L: Lowering` | 239 → 87 left in MarsRT; 287 in the crate |
| `cpu/blocks/mod.rs`: the code's bound and its check at a run's start | `Jit::budget`, `Jit::over_budget`, `Jit::flushes` | |
| `cpu/blocks/jit/emit.rs`: the clearing, declaring and defining around one block | the crate's `compile`; `emit::compile` became `emit::lower` | 1,076 → 1,070 |
| `cpu/blocks/verify.rs`: the step count, `follow`, `check`, `check_all` and their panics | `emusen-jit/src/verify.rs`, over a `Reference` trait | 77 → 81 (the machine and its comparison); 48 in the crate |

What stays in MarsRT is the VR4300's: the compiled block's signature, `Context_`, the exits `DONE`, `RAISED` and
`FINISH`, the three tiers' options, the shapes, the cache, the decoded tier, the dispatcher with its guard, and the
interpreter the verifier compares with. The interface is `EmuSen_Jit.md` §2, version 1.0.0.

### 42.3 The gate (measured 2026-10-25)

On the desktop at CPU weight 20; the corpus from `TestRoms/n64/n64-systemtest.z64`, the games from read-only copies of
the library under `~/.cache/emusen/pgo/games/measure/`, Super Mario 64 and Ocarina of Time only, as the Mars probe
grades them. MarsRT's GPU tests were not run.

| Suite | Result |
| --- | --- |
| MarsRT's block tests, every tier (`tests::blocks`) | 21 of 21 |
| the corpus at every tier | 1,722 lines identical to the interpreter's at each, "Failed 46 of 4637 tests" |
| the games at tier 1, 2 and 3: the recompiled machine against the interpreted one, unthreaded and on four workers deferred, and the run that carries its blocks against one loaded from its state; both games from power-on and from their states, 300 frames | 3 of 3 at each tier, identical after every frame |
| the tests that read the recompiler's switch or counters (`ffi::host`, `ffi::v1`, `tests::debug`, `machine`, `tests::rsp_blocks`) | 41 of 41 |
| WiseMan, the blast radius's filter `MarsRt`, the device cases left out | 257 of 257 (16 min 30 s), with `EMUSEN_MARSRT_STATES` holding the two games |
| `emusen-jit`'s own tests | 10 of 10 |
| ThreadSanitizer: the crate's tests; MarsRT's block tests | no report outside the blind spots `Mars_Native.md` §5.8.5 names (17 and 51 reports, every one an `Arc`'s last drop, `region`'s page-size `Once` or a std `OnceLock`) |

### 42.4 Speed (measured 2026-10-25)

**Speed** (measured 2026-10-25 and 2026-10-26, `Mars_Native.md` §5.8.8's method on MarsRT's own example): `examples/threads … split 4
blocks`, 600 frames from the gameplay states, the build before the move (WiseMan 9ac67498a) and after it, both
`release` and unguided, interleaved with their order alternated between rounds; every run under the shared timing lock
and, inside it, the quiet gate of `Mars_Native.md` §6.16.1 (the rest of the machine under 1.2 busy processors for a second; the gate read
1.00 to 1.10, other work running on one processor). Five rounds, ms a frame:

| game | before, median (range) | after, median (range) | paired, median | overlap |
| --- | --- | --- | ---: | --- |
| Super Mario 64 | 3.633 (3.524–3.723) | 3.581 (3.515–3.723) | −0.9% | yes |
| Ocarina of Time | 4.726 (4.589–4.881) | 4.573 (4.531–4.697) | −1.5% | yes |

Every run of both builds ended on one state hash a game (`C1D7EE67AFB92361` and `61F15559C5A23119`). The ranges overlap
on both games, so by `Mars_Performance.md` §6's rule **the speed is unchanged within its noise**; the small lead of the
new build is two disturbed rounds of the old one on Ocarina of Time (4.881 and 4.842), not a gain anyone should expect
from code that was moved. Nothing on the emulation thread's path changed: the dispatcher reads a slot with one Acquire
load as before, and the generic code is monomorphised into the library with LTO.

### 42.5 Tests and seeded faults (2026-10-25)

**The crate's tests** are `EmuSen_Jit.md` §5's ten, on a toy processor. **Seeded faults**, each made alone, the suites
named run, the source restored; the first in place, the others on a copy of the tree built apart [M]:

| # | Fault | Caught by |
|---|---|---|
| F1 | the slot's store Relaxed rather than Release (`Slot::publish`) | ThreadSanitizer alone: the test's read of the code's bytes against `cranelift-jit`'s copy on the compiler's thread, in `code_is_seen_whole_by_the_thread_that_reads_its_slot`; the ten tests pass with it, the store being one instruction either way on x86-64 |
| F2 | the code budget never enforced (`Jit::over_budget` always false) | the crate's `past_its_budget_the_module_goes_whole_and_compiling_starts_again` and `a_dispatcher_that_checks_its_budget_each_run_starts_no_run_past_it`; MarsRT's `dropping_every_block_at_the_codes_bound_changes_nothing` |
| F3 | the verifier skipping every compiled block (MarsRT's dispatcher handing compiled code no verifier) | MarsRT's `the_verifier_steps_beside_every_instruction_and_finds_nothing`, the interpreter left behind by eight steps |
| F4 | the verifier stepping without comparing (`Verifier::follow`) | the crate's `the_verifier_finds_a_miscompiled_instruction_at_that_instruction` and `the_verifier_finds_a_block_it_did_not_follow` |

### 42.6 PGO

MarsRT's profile has been void since d84b3bc1 (`Mars_Native.md` §6.17) and stays void: the moved functions are renamed
again, and the source digest, which follows path dependencies, now covers `emusen-jit`. A retrain covers the shared
crate as it does `emusen-native`; this stage adds nothing to what it owes.

### 42.7 Where the plan was wrong, or left open

- **§9.6 counted 239 lines to move**; what moved is most of them and the verifier's shape from a second file
  (`verify.rs`), and the code budget's check from the dispatcher, all three named in §9.6's list.
- **"Names its exits"** became a list of names a lowering gives (`Lowering::EXITS`); what an exit means stays with the
  dispatcher that handles it.

### 42.8 Questions this stage leaves

1. **Q1 [routine], the `Arc` hand-off.** *Recommended*: leave it until a stage that changes the compiler's behaviour;
   a slot index into storage the crate owns would let ThreadSanitizer see the hand-off, and S36 is the place to weigh
   it if Sapphire's dispatcher wants a different slot store.
2. **Q2 [routine], Diamond.** *Recommended*: no change; Diamond adopts the crate when its own recompiler is wanted
   (`Diamond_Speed.md` §7.2).

### 42.9 What was read for this stage

`Sapphire_Plan.md` §7 (S35), §9.6, §11 Q10; `Mars_Native.md` §5.6.7 (the ThreadSanitizer recipe), §5.8 whole,
§6.17.5 (the method of measurement); `Diamond_Speed.md` §7.2; MarsRT's `cpu/blocks/` and `cpu/blocks/jit/`; the
`cranelift-module` 0.136.0 crate's `Module` trait, for the methods a lowering may call on a module it does not own.
No emulator source, and no firmware.

### 42.10 What this stage does not cover

- Any emitter for the EE or the VUs (S36 to S41).
- MarsRT's GPU tests, which need the RX 6800 alone.

## 44. Stage S36, the EE recompiler: the integer instructions and the branches (2026-10-26)

*Numbered §44, after S35's §42, §43 being S32's; its disputes would be numbered from D-580, and it raised none: every
oracle below agreed with the interpreter, and the stage reads nothing of the hardware the interpreter does not already
implement.*

### 44.1 In one paragraph

The plan's §7 row S36, on the tester's go of 2026-10-26, beside S32's bench, under the rule decided the same day (cycle
accuracy where it is realistic, byte accuracy the floor) and with this stage held to exactness. **The EE's lowering on
`emusen-jit`** (`src/ee/jit/`) compiles blocks of the integer instructions, both multipliers and dividers with the HI1/LO1
pipeline, SA's moves, SYNC, the traps and every branch and jump with its delay slot, likely or not; everything else ends a
block before it and is the interpreter's. A compiled block holds the guest registers and every field of the issue model it
touches in host registers from its entry to its exit, with no call inside it, and computes each instruction's issue cycle
as `Timing::plan` and `Timing::commit` do with the instruction's category, row, reads and writes made constants. **The
board is brought along at the block's exit, not after each instruction** (§44.5): the exit hands each instruction's cycles
to a replay that runs the board through them in turn, exactly the calls `System::step` makes, and keeps the instructions
before the first interrupt the interpreter would have taken; when the replay finds one inside the block, nothing has been
written and the interpreter runs the instructions that stand. **The gate is met** (§44.7): the interpreter stepped beside
the blocks after every instruction and after every block; 240 random programs, compared every 400 steps and whole at their
end; fourteen mechanism tests, the S6 programs among them over all 99 mnemonics a block compiles; the 55 EE ps2autotests,
52 equal to the console's texts on both tiers and all 55 texts byte for byte the same; and the OSDSYS boot, Jak and
Daxter's own program and the project's integer loop, picture, sound and state equal frame by frame, and from a state
loaded into an empty cache. Faster on every workload measured (2.63 times on the project's loop, 1.49 on Jak and
Daxter, 1.20 on the GS-bound boot, §44.8), the recompiler is the default tier, decided by the tester, the interpreter a
switch away (§44.6). The stage raised no dispute.

### 44.2 What was built

| Module | What it holds | Lines |
| --- | --- | --- |
| `src/ee/jit/mod.rs` | the compiled block's signature, its context and exits, the options, the lowering for `emusen-jit`; the core's side of an entry: where a block may be entered, the words the instruction cache holds, whether it holds a block's words, the interrupt check by its inputs | 178 |
| `src/ee/jit/shape.rs` | what a block holds and where it ends | 75 |
| `src/ee/jit/emit.rs` | one block as Cranelift IR | 1,160 |
| `src/blocks.rs` | the dispatcher: the blocks by virtual address, the threshold, the replay and the run again before an interrupt, the verifier's machine and its comparison | 464 |
| `src/board.rs` | `Board::run_through`, `run_to` through several times with its checks made once (§44.5) | +60 |
| `src/ee/timing.rs` | the last category held as its code (`Category` is `repr(u8)`), Table 1-3's partner rule as a function, the model's field layout for the code | +40 |
| `src/vif`, `src/vu`, `src/sif` | the idle predicates without the clocks' moves, the SIF's wake looked at without taking it | +25 |
| `src/machine.rs`, `src/v1.rs` | the frame's 1,024 steps through the dispatcher when the tier is chosen; the `ee_tier` setting | +45 |
| `tests/blocks.rs` | the random programs, the verifier's runs and the mechanisms | 609 |
| `examples/s36.rs` | the whole-machine oracles and the timing (`equal`, `verify`, `state`, `time`) | 263 |

**The new cross-component dependency is `sapphire -> emusen-jit`** (with `cranelift-codegen`, `-frontend`, `-jit` and
`-module` 0.136.0, the crate's own), named in Sapphire's `Cargo.toml` and in `EmuSen_Jit.md` §1. `emusen-jit` itself is
unchanged: its interface, version 1.0.0, served a second processor without a change, so MarsRT's tests were not in the
blast radius.

### 44.3 What a block holds, and when one is entered

**Shape** (`shape.rs`): consecutive instructions from the entry, each one of the 99 mnemonics `blocks::mnemonics` lists
(the integer operations of S6 but the loads, stores, PREF, SYSCALL, BREAK and the coprocessors' instructions); a branch
or jump only with its delay slot, and only when the slot is such an instruction, the block ending after the slot; at most
64 instructions, and never past the end of the entry's 4 KiB page, so that one translation covers the block. A branch
whose slot is a branch is never in a block, as in MarsRT. The words are the instruction cache's, not memory's: a fetch
that hits reads the line, a store never reaches it (§17.3), and the block is what the interpreter's fetches would read.

**The entry** (`Ee::block_entry`, `blocks::block`) is refused, and the interpreter takes the step, unless every check
the interpreter makes before an instruction would do nothing there and every fetch of the block would hit:

- not in a delay slot; the performance counters off (CTE clear), so no event is counted and no counter exception is due;
  the instruction breakpoint off (BPC.IAE clear); at least one instruction issued since the reset, so the pairing rule's
  "an instruction before" holds;
- the interrupt check before the first instruction would take nothing; it is made by the step's own code, which writes
  Cause.IP2 and IP3 as the step would;
- the program counter aligned, Config.ICE set, and the address translating, in the current mode, to a cached page (C or
  K0 3, or K0 0's write-through); and every line of the block in the instruction cache holding exactly its words.

A block whose words differ from the cache's is shaped again; one with a line out of the cache is kept, and the
interpreter's fetch refills the line (`Stats::refills`); a new block that would run on into a line not yet fetched waits
for the interpreter to fetch it (`Stats::waits`), so that a block is not cut short by a cold cache. The block is handed to
the compiler at its 64th entry (MarsRT's threshold) and runs as interpreter steps until its code is published; a block
longer than the steps left in the frame's chunk of 1,024 is left to the interpreter, so that a frame ends after exactly the
instruction it ends after in the interpreter. Config.DIE is part of a block's options: a block compiled for dual issue is
not entered under single issue.

**Self-modifying code** is therefore exact by construction: a store to code changes memory and not the line, the
interpreter's fetches go on reading the line, and so does the block; when the program invalidates the line (CACHE IHIN,
IXIN or IXSTG) the next entry finds the line missing or different and the block is shaped again from what the refill
brings. Nothing is invalidated by a store, because nothing a store does can change what a fetch reads.

### 44.4 The emitter

A block is a straight line with side exits: there is no join, so every value is an SSA value carried by the emitter, not
a Cranelift variable. **Registers**: the low halves of the GPRs a block names, HI0/LO0/HI1/LO1 and SA are loaded at their
first use and stored at the exit, the high halves never touched (§16.2's reading that a narrower write keeps bits 127..64).
**The issue model**: `last`, `open`, `floor`, `group_ends`, `paired`, the last category and pipe and the low and high
halves of `last_writes` are loaded at the entry; each resource's ready cycle and the multipliers' free cycles at their
first use. Each instruction is then `plan` and `commit` with its constants: the ready cycles of what it reads, the units it
waits for, Table 1-3's partner (computed at compile time for each last category and pipe and chosen by the run-time pipe;
for the first instruction, whose last category is the entry's, a 30-bit table), the pairing condition, the issue cycle,
then the ready cycles of what it writes, the free cycles, the group's end and the flags. A value the block's own
instructions fix is folded (a constant, or a sum with one), so that `max(0, x)` and the conditions on known flags cost
nothing. Then the step's `advance`: Random once an issue cycle that does not pair, the group's single or dual mark.
**Count, Compare and the cycles Count saw** are written at the exit from the cycles after the last instruction: the sum of
every step's advance is the difference of the two counts, and the steps' tests of Compare, each over its own span of
cycles, together test the whole span, which one comparison makes.

**Exits**: the end of the line, before an instruction the block does not hold (the core at that instruction); after a
branch's slot (at the target or past the slot, as the condition gave it); a likely branch not taken (past the slot, which
did not run); and before an instruction that would raise, an overflowing ADD, SUB, DADD, DSUB, ADDI or DADDI or a trap whose
condition holds, so that the interpreter raises it — in a delay slot, the core left in the slot with the branch's outcome
as its next address. Each exit stores every instruction's cycle count into the context as it goes, calls the replay
(§44.5), and only if every instruction stands writes everything back.

**C1** (`Options::fixed_cost`, `EMUSEN_SAPPHIRE_C1=1`, off): each instruction one cycle after the last, never paired, its
results ready the next cycle, the multipliers' units ignored; the other fields as the model leaves them. It is `effect:
accuracy`, measured in §44.9, not taken.

### 44.5 The board at a block's exit

After every instruction `System::step` brings the board to the instruction's cycles (`Board::run_to`) and sets the core's
lines from it, and the next instruction's interrupt check reads them. A block skips those calls while it runs, and makes
them, in order and with the same times, at its exit. That is exact if nothing the block computes is read by the board and
nothing the board does in between is read by the block, and both hold: the block's instructions touch only the core's
registers and the issue model (no load, no store, no coprocessor), and `run_to` reads of the core only its scratchpad,
which no compiled instruction touches. What is left is the order of one thing: **the interrupt check before each
instruction of the block**, which in the interpreter sees the lines after the instruction before. So the replay checks it
with the lines after each time and the Count that time gives; if the interpreter would have taken an interrupt before
instruction j + 1, the replay stops after time j, the compiled code returns without writing anything (`ROLLBACK`), and the
dispatcher runs instructions 1 to j through the interpreter's `Ee::step`, setting before each the lines the replay saw
after the one before, so that the core leaves them as the interpreter would and the next step takes the interrupt. When all
stand, Cause.IP2 and IP3 are set from the lines after the next-to-last time, which is what the last step's own check wrote,
and the core's lines from those after the last.

**Why the replay is not `run_to` called once.** The board's units do not all keep their own time: when the IOP's side
moves a SIF FIFO, the EE's DMAC is woken at the board's time *after the EE instruction during which it happened*
(`run_to`'s wake), so the interpreter's granularity of one EE instruction is part of the DMA's timing. Calling `run_to`
once at the block's end would wake the DMAC late.

**`Board::run_through`** makes the same calls cheaper without changing them. While no board event falls by the last time,
the GIF, both VIFs and both VUs are idle and no wake is pending, `run_to` at each time is its quick path — the time moved,
the IOP brought along, the wake taken if the IOP's side set it — and nothing in it can change the quick path's conditions
but the wake. So the conditions are tested once, the IOP is caught up at each time in turn as the calls would catch it up,
the lines are those of the entry, and the units' clocks are moved once to the last time (their moves are `max(clock, t)`,
whose end is the same); at the first wake the wake is taken at that time exactly as the quick path takes it, and from there
each time is `run_to` itself. Measured on the project's integer loop (§44.8), this took the recompiled frame from 112 to
86 ms.

### 44.6 The switch, and what is recommended

`ee_tier` in Sapphire's settings, "EE processor", `interpreter` (the default) or `recompiler`, `effect: exact`, read at
create; `Machine::blocks` holds the dispatcher when it is chosen. For measurement the environment gives
`EMUSEN_SAPPHIRE_TIER=recompiler` (which `examples/s10.rs` and the core honour), `EMUSEN_SAPPHIRE_THRESHOLD`,
`EMUSEN_SAPPHIRE_C1=1`, `EMUSEN_SAPPHIRE_SPILL=1` (§44.8) and `EMUSEN_SAPPHIRE_PERFMAP=1`. The verifier is
`Blocks::verify` with `Verify::Steps` (the code calls it after every instruction, everything written back, and the board
replayed by those calls) or `Verify::Blocks` (the dispatcher steps the interpreter after each block, the code unchanged).
The debugger's frame (§41) runs the interpreter, as MarsRT's observed frame does. **Decided 2026-10-26 by the tester**
(§44.13 Q1): the recompiler is the default once the speed rounds showed it faster on every workload (§44.8), the interpreter
a switch away. The setting's default is `recompiler`; `Machine::new` alone still builds the plain interpreter, so the Rust
tests' oracle stays the interpreter, and `EMUSEN_SAPPHIRE_TIER=interpreter` chooses it for measurement (`examples/s10.rs`
follows the core's default).

### 44.7 The gate (measured 2026-10-26)

On the desktop at CPU weight 20 beside other work; the player's Americas BIOS `ps2-0230a-20080220.bin` executed,
never read; ps2autotests from `~/.cache/emusen/probe/sapphire/ps2autotests/`, Jak and Daxter read in place from
`~/.cache/emusen/probe/sapphire/discs/`.

| Oracle (`Mars_Native.md` §5.8.5) | Run | Result |
| --- | --- | --- |
| 1. The interpreter stepped beside the blocks, compared after every instruction | `the_verifier_steps_beside_every_compiled_instruction_and_finds_nothing` and `random_programs_under_the_verifier_find_nothing` (20 random programs, half with timer 0), each with `Verify::Steps` and with `Verify::Blocks`, 10,000 steps each; the whole machine compared at the end | held |
| | `s36 verify cost`, 20 frames, values named | held, 99,685,376 steps compared |
| | `s36 verify boot` and `s36 verify disc` (Jak and Daxter, attached after the fast boot's entry), 30 frames each, quiet (pass and count only) | held: 220,863,488 and 231,997,440 steps compared, and the whole states equal at the end |
| 2. Random programs, seeded | `random_programs_leave_the_interpreters_machine`: seeds 0 to 159, 24,000 steps; `…with_timer_interrupts…`: seeds 1000 to 1079, timer 0 through the INTC on IM2 beside Count on IM7; the core, COP0, the issue model, the lines, the board's time and INTC compared every 400 steps, the whole state at the end | equal at every comparison and whole: 3,750,756 of the 3,836,815 steps compiled; 1,478 exits found an interrupt inside their block and had 3,185 instructions run again by the interpreter; with timer 0, 1,868,787 of 1,918,580 compiled, 734 such exits and 1,420 instructions |
| 3. The mechanisms one at a time | fourteen tests, below | 14 of 14 |
| 4. ps2autotests' 55 EE programs | `s10` on each tier | 52 of 55 on both, the same three as §36.5 (`cpu/ee_fpu/sqrt`, `dma/vif/intstall`, `vu/lower/efu`); all 55 texts identical byte for byte between the tiers, and every verdict line equal in its frames, console seconds, TTY bytes, calls and IOP resets |
| 5. The bench workloads, frame by frame | `s36 equal`: the picture and the sound after every frame, the state every 60 | the OSDSYS boot, 760 frames (past the language screen at 690): 0 pictures, 0 sound, 0 of 13 states differing, 6,536,792,276 of 6,906,097,754 steps compiled; Jak and Daxter, entered at frame 8, 600 frames: 0, 0, 0 of 10, 3,018,382,015 of 3,899,591,997; the project's loop, 60 frames: 0, 0, 0 of 1 |
| 6. A state loaded into an empty cache | `a_state_saved_between_blocks_loads_into_an_empty_cache_and_runs_alike` (three random programs, saved at step 7,777, 10,000 steps on, against the interpreter); `s36 state` on the boot (state at frame 400, 100 frames on), Jak and Daxter (300, 100) and the loop (20, 20) | equal throughout; the boot's carried machine held 9,588 blocks, the loaded one compiled 5,327 afresh |

The sixth oracle of MarsRT's list, a C# core through the shim, has no counterpart: Sapphire has no other core; S32's bench
is not merged on this branch, so the workloads ran through `examples/s36.rs`, built on the same definitions (`boot`,
`elf`, `disc` and their fast boot) as S32's `examples/bench.rs`.

**The mechanisms**, each a program of the project's own:

| Test | What it holds |
| --- | --- |
| `a_hot_loop_runs_compiled_and_leaves_the_interpreters_machine` | a loop of arithmetic, a multiply and a branch with its slot, 4,000 steps |
| `the_stage_s6_programs_over_every_integer_encoding_leave_the_interpreters_machine` | §16.5's seventeen programs, every one equal to the manual's record on the blocks' machine too, and every one of the 99 mnemonics in a block that compiled |
| `a_block_leaves_before_an_instruction_that_raises_in_a_line_and_in_a_delay_slot` | ADD overflowing and TEQ trapping in a slot on some passes: the exceptions taken by the interpreter at the instruction |
| `code_replaced_runs_old_until_its_line_is_invalidated_and_the_block_is_shaped_again` | a routine's word replaced in memory runs old for thirty calls, until CACHE IHIN, then new |
| `a_state_saved_between_blocks_loads_into_an_empty_cache_and_runs_alike` | as above |
| `blocks_refused_at_a_small_step_budget_leave_the_machine_exact` | the dispatcher given 1, 3, 7 and 11 steps at a time |
| `single_issue_blocks_leave_the_interpreters_machine` | Config.DIE cleared first |
| `dropping_every_block_at_the_codes_budget_changes_nothing` | a 16 KiB budget, flushed during the run |
| `a_block_ends_at_its_pages_end` | a loop across a 4 KiB boundary |
| `the_fixed_cost_compromise_changes_the_cycles_and_not_the_results` | C1: the registers equal, the cycles not |
| the random programs and the verifier's runs | above; their interrupts fall inside blocks: 1,478 and 734 exits run their instructions again before an interrupt in the two random tests |

**What the random programs hold**: every compiled mnemonic but the coprocessors' among loads and stores in a scratch area,
forward branches of every kind with their slots, inner loops closed by BGTZ and BGTZL, calls through JALR and returns
through JR, J, Count's interrupt every 1,500 cycles and in half the timer programs timer 0's every 300 to 900, and a
handler at both vectors that acknowledges them and steps past an exception. **A lesson the generator taught**: the first
version had its handler at 80000180h alone, so the first interrupt sent each program through the zeros from 80000200h on
and most of its steps were a NOP slide; the comparison held (it is exact there too) but the code the test was written for
ran once. Measured on the 160 seeds at 20,000 steps: 232,120 steps compiled with the slide, 2,336,172 of 2,397,741 after the fix.

### 44.8 Speed (measured 2026-10-26)

**The method**: `examples/s36.rs time`, each frame's `advance` timed, the mean over the run; three rounds, each running every
workload on both tiers back to back, the order of the tiers alternated between rounds; every run alone, in a capped scope at
CPU weight 100 while other work built beside it at weight 20, the load average (1, 5, 15 minutes) read before and after each
run. The workloads are the project's integer loop (120 frames, no BIOS), the OSDSYS boot's first 300 frames and Jak and
Daxter's program for its first 400 frames (entered at frame 8). The C1 and spill runs followed in three more rounds.

| Mean ms a frame | Interpreter, rounds 1–3 | median (spread) | Recompiler, rounds 1–3 | median (spread) | Ratio of medians |
| --- | --- | --- | --- | --- | --- |
| The project's loop | 236.2, 221.3, 214.3 | 221.3 (9.9 %) | 87.9, 84.3, 82.7 | 84.3 (6.2 %) | **2.63×** |
| The OSDSYS boot, frames 0–300 | 898.7, 871.0, 847.1 | 871.0 (5.9 %) | 726.0, 727.7, 665.6 | 726.0 (8.6 %) | **1.20×** |
| Jak and Daxter, frames 0–400 | 343.7, 319.6, 313.0 | 319.6 (9.6 %) | 217.6, 215.2, 205.6 | 215.2 (5.6 %) | **1.49×** |

The load averages fell from about 7 in the first round to about 2 in the third, which is the steady fall within each
column; the rounds' order alternated, so the ratio holds within every round (2.69, 2.62, 2.59 on the loop; 1.24, 1.20,
1.27 on the boot; 1.58, 1.49, 1.52 on Jak and Daxter). The spreads are under 10 % on every workload. In frames a second the
loop goes from 4.5 to 11.9, the boot from 1.15 to 1.38, Jak and Daxter from 3.1 to 4.6: the machine as a whole stays 20 to
60 times short of an NTSC frame, and the EE recompiler alone does not change that order (§44.12).

**What the blocks covered** (the equality runs of §44.7): 95 % of the boot's steps, 77 % of Jak and Daxter's and 98.7 % of
the loop's ran compiled; blocks averaged 6.8, 3.6 and 6.2 instructions; the boot compiled 9,802 blocks into 14.9 MB of code
over 760 frames and Jak and Daxter 15,549 into 21.1 MB over 600, far under the 256 MiB budget.

**Where the time goes**, by an interrupt sampler (`ipsample.py`, the sampler `Mars_Native.md` §6.3 uses, with symbols
from the binary) on the recompiled integer loop: the compiled code 24 %, the replay and its per-step check 25 %, the IOP's
catch-up at each time (its units and the SPU2, the IOP being held without a BIOS) 27 %, the dispatcher's entry (lookup,
translation, comparison with the cache) about 12 %. On the OSDSYS boot the EE is a small part of the frame: the GS's exact
arithmetic (`gs::exact`) took 56 % of the samples over frames 40 to 90, which no EE tier changes. **Holding the registers
pays little where there is no call to force a write-back**: `EMUSEN_SAPPHIRE_SPILL=1` stores every written register to the
context after each instruction, as a block keeping them in memory would; measured in the same rounds on the loop, 81.6 ms a frame (81.6, 81.0, 81.8) against 80.6 (80.7, 80.3, 80.6): 1.3 %, about
the rounds' own spread. On this loop the stores are cheap beside the replay and the entry, so holding registers is
worth keeping (it costs nothing) and not worth more work at S36; it will matter more when S37 removes the block ends at
loads and stores and blocks grow.

### 44.9 C1, measured and not taken

`EMUSEN_SAPPHIRE_C1=1`, three rounds after the speed rounds: the loop at 70.4 ms a frame (70.5, 70.3, 70.4) against the exact
recompiler's 80.6 in the same rounds, **12.6 % less time a frame**; Jak and Daxter at 163.2 ms (163.5, 163.2, 162.5) against the exact
recompiler's 205.6 to 217.6 in the speed rounds, about 21 % faster, though not interleaved with them and at a lower load, so
an upper bound. What it gives up is visible at once: on the hot loop of `the_fixed_cost_compromise_changes_the_cycles_and_not
_the_results` the issue model's 3,818 cycles for 4,000 instructions become 4,016 (the model pairs instructions, so one cycle
an instruction is *slower* in guest time there), and every count, timer and interrupt a game reads moves with them; a C1 run
of Jak and Daxter is not the exact run, and its frames were not compared. By §5.13's rule C1 is a candidate whose measurement
is now recorded; decided 2026-10-26 by the tester, it stays off and is not taken.

### 44.10 Seeded faults (2026-10-26)

Each made alone on a copy of the crate built apart, `tests/blocks.rs` run whole (its version of that commit), the source
restored:

| # | Fault | Caught by |
| --- | --- | --- |
| F1 | the inline ADDU subtracts | 9 tests: the S6 programs, the hot loop's verifier, the random programs (all three), the step budget, replaced code, the state into an empty cache, the code budget |
| F2 | pairing ignores the last instruction's writes (a dependent instruction pairs) | 5: the S6 programs, the random programs (all three), the code budget |
| F3 | a likely BEQ or BNE not taken runs its slot | 5: the S6 programs, the random programs (all three), the code budget |
| F4 | the replay never stops for an interrupt | 5: the random programs (all three), the state into an empty cache, the code budget |
| F5 | a block's words not compared with the instruction cache | 1: replaced code |
| F6 | Count passing Compare inside a block leaves IP7 clear | 4: the random programs (all three), the code budget |
| F7 | Random not moved by a block's issue cycles | 13 of 14 |
| F8 | MULT1 and MADD1 write pipeline 0's HI and LO | 6: the S6 programs, the random programs (all three), the state into an empty cache, the code budget |

F5 has one catcher, the test written for the mechanism, as MarsRT's survivors taught (`Mars_Native.md` §5.8.6): a block
that rewrites its own words is never seen stale unless the stale word is executed after the line is invalidated.

### 44.11 Exposure, and what was observed

Running the player's BIOS on Sapphire makes this writer a writer of a player's-BIOS stage (§1.4): **exposed, and never a
replacement writer**. The BIOS image was executed by the machine and never opened, hashed, diffed, disassembled or dumped
by any tool of this stage; the recompiler compiled BIOS code as it compiles any guest code, and no block made from a BIOS
address, no program counter inside the boot ROM or in BIOS code in RAM, and no memory holding BIOS code or data was
printed, logged or looked at; the verifier on the BIOS ran quiet, reporting pass or fail and the count of steps. What was
printed of BIOS runs: counts (steps, blocks, rollbacks, bytes of code), equality of pictures, sound and states, and
times. One slip is recorded: the names of the files in the BIOS folder were listed once, while locating the Americas
image (whose name `v1.rs` gives); no file was opened by it, and the image was then named by path. No SIF traffic was
traced. No emulator, SDK or loader source was read.

### 44.12 Where the plan was wrong, or left open

- **§5.12 priced the interpreter at 1.2 to 9 times too slow.** Measured on this branch it is far slower: the OSDSYS boot
  at about 870 ms a frame over its first 300 frames, Jak and Daxter at 320 and the project's integer loop at 220 (§44.8),
  13 to 52 times an NTSC frame. S32 measures where that time goes.
- **§5.12's EE recompiler "inlines nearly every instruction"**; at S36 the loads and stores (S37), MMI (S38) and the
  coprocessors (S39) end a block, so blocks average two to six instructions in real code (§44.8's counts).
- **The board's catch-up after every EE instruction is the reference's** and is not the recompiler's to change: the DMAC's
  wake time depends on it (§44.5). `run_through` halves its cost without changing a call; what remains, the IOP brought
  along at each time and, without a BIOS, its units and the SPU2 run at each, is the next lever's (S33's idle skipping or a
  decided change of the wake's granularity, which would be a timing compromise of its own).
- **MarsRT's "holding registers in host registers bought nothing"** (`Mars_Native.md` §5.8.8) does not carry over as a
  rule: there every call wrote them back; here a block has no call, and §44.8 measures the difference directly.

### 44.13 Questions this stage leaves

1. ~~**Q1 [decision], the default tier.**~~ *Decided 2026-10-26 by the tester: the recompiler becomes the default once the
   speed rounds confirm it faster, the interpreter a switch away; they did (§44.8), and §44.6 records the change.*
2. ~~**Q2 [routine], C1.**~~ *Accepted 2026-10-26: C1 stays off and untaken (§44.9).*
3. ~~**Q3 [routine], the `Arc` hand-off** (§42.8 Q1).~~ *Accepted 2026-10-26: unchanged.*
4. ~~**Q4 [routine], the board's catch-up granularity.**~~ *Accepted 2026-10-26: S32's ranking decides between S33's idle
   skipping and a measured compromise on the DMAC's wake time.*

### 44.14 What was read for this stage

`Sapphire_Plan.md` §1.4, §5.2, §5.12 with its amendment, §5.13, §7's S32 to S39 rows and their amendment, §9.6;
`EmuSen_Jit.md`; `Mars_Native.md` §5.8; this page's §16, §17's first seven sections, §36.4 and §36.5, §41.8 and §42;
`emusen-jit`'s source; MarsRT's `cpu/blocks/jit/` for the crate's use; the interpreter's `ee/mod.rs`, `ops.rs`,
`timing.rs`, `memory.rs`, `cache.rs`, `tlb.rs`, the board's `run_to`, the IOP's `run_to` and the SIF's wake; S32's
`examples/bench.rs` on its branch, for the workloads' definitions; Cranelift 0.136's `InstBuilder`. No emulator, SDK or
loader source, no article, and no firmware.

### 44.15 What this stage does not cover

- Loads and stores with the RDRAM and scratchpad fast paths (S37), MMI as vector code (S38), COP1 and COP2's macro forms
  (S39), COP0, SYSCALL and BREAK: each ends a block at S36.
- Chaining blocks, loops inside a block (MarsRT's loop back), a decoded tier, and freeing code a block at a time.
- The VUs' recompilers (S40, S41) and the units on threads (S42).
- The handheld: no measurement was made there.
