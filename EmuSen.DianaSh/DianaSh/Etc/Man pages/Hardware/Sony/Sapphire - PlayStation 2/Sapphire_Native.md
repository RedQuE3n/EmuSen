# Sapphire_Native — the build record

*Started 2026-10-24, at stage S1; stage S2 the same day (§12).* `Sapphire_Plan.md` says what Sapphire is to be and how it is graded; this page records
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

