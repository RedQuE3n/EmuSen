# Sapphire_Native — the build record

*Started 2026-10-24, at stage S1; stages S2 (§12), S3 (§13), S4 (§14), S5 (§15) and S6 (§16) the same day; S7 (§17), S8 (§18) and S9 (§19) on 2026-10-25.* `Sapphire_Plan.md` says what Sapphire is to be and how it is graded; this page records
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
  layout and its sector size, since an image records no physical format]. Never real game data.
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
  `WriteSet` writes `.cue` and `.bin`), `Dvd()` (2,048-byte sectors, the root at sector 300; `WriteSet` writes `.iso`),
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
