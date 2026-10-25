# Sapphire_Native — the build record

*Started 2026-10-24, at stage S1; stages S2 (§12), S3 (§13), S4 (§14) and S5 (§15) the same day.* `Sapphire_Plan.md` says what Sapphire is to be and how it is graded; this page records
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
