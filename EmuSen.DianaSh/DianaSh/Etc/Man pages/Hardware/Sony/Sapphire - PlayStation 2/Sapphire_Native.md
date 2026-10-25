# Sapphire_Native — the build record

*Started 2026-10-24, at stage S1; stages S2 (§12), S3 (§13), S4 (§14), S5 (§15) and S6 (§16) the same day; S7 (§17), S8 (§18), S9 (§19), S11 (§20), S12 (§21), S21 (§22), S13 (§23), S16 (§24) and S23 (§25) on 2026-10-25.* *Decided 2026-10-25 by the tester: S11 is built before S10*, whose gate runs on the player's BIOS once S24 boots it (`Sapphire_Plan.md` §7); the stages are recorded here in the order they are built. `Sapphire_Plan.md` says what Sapphire is to be and how it is graded; this page records
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
  the data is in memory when the interrupt comes]. A count of zero ends after the seek. ReadCd's blocks are the user
  data (2,048), the sector from its subheader (2,328) or from its header (2,340) [A from ECMA-130's layout]; CD-DA's the
  raw 2,352 bytes; ReadDvd's the Data Frame (D-121, D-122). ReadCd of a DVD gives its Main Data, the rest of a larger
  block zero [A]. LBA 0 is 00:02:00 on a CD.
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
| Sapphire | lib **32 of 32** (14 the drive's), `cop0` 19, `dmac` 17, `fpu` 9, `interpreter` 23, `mmi` 13, `programs` 3 (now reading `programs/s23/`), `vu0` 14; `cdvd` also under `test-checked`; clippy clean |
| Diamond, Diamond's `v1` | **306 of 306**, **5 of 5**: the crate's first user, unchanged |
| Nephrite | **194 of 194**: its second, unchanged |
| `emusen-core-conform`, `Platform/galaxia`, `library`, `library-tool`, `platform` | 36, 87, 11, 9 and 40, every one passing |
| WiseMan (`Diamond`, `Galaxia`, `GameRecords`, `Library`, `NephriteTests`, `SapphireRunnerTests`, `SapphireTests`) | **562 passed, 0 failed, 11 skipped**: eight `SettingPathsOpenTests` and three big-picture tools, skipped by their own conditions, none near the disc |

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
- **The brief's "region and model queries ps2tek describes"**: ps2tek describes none at the S-command level, only
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
