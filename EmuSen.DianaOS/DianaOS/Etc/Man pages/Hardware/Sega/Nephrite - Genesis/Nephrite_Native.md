# Nephrite_Native — the build record

*Started 2026-10-04, at stage 0.* `Nephrite_Plan.md` says what Nephrite is to be and how it is graded; this page
records what was built, stage by stage, what was measured, and where the plan turned out to be wrong. The plan's three
registers are kept: **measured** is a number taken here, on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10, Rust
1.98.1), with the tool named beside it; **argued** is reasoning with no measurement behind it; **predicted** is one of
the plan's P1–P8, retired where a measurement reaches it.

The CPUs' records are their crates' own pages in `../Beryl - Shared CPUs/`.

---

## 1. Stage 0 (2026-10-04)

The plan's §6 row 0. What it built:

- **The naming**, decided by the tester on 2026-10-04: Nephrite is the Genesis with the Sega CD and the 32X as its
  attachments, its folders renamed from `Nephrite - 32X` to `Nephrite - Genesis`; Beryl is the shared CPU library, its
  folders renamed from `Beryl - Genesis` to `Beryl - Shared CPUs` (`EmuSen_Core_Naming_Scheme.md` §3, with the reasoning
  for the crate's plain name).
- **The crate** `nephrite`, `EmuSen/Cores/Sega/Nephrite - Genesis/`, on the core ABI v1 from its first commit, with a
  stub machine and its own state format (§2). No part of any console is emulated.
- **The three Beryl crates**, `beryl-m68k`, `beryl-z80` and `beryl-sh2`, as skeletons: each processor's bus trait,
  registers and step signature, which refuses to step until the processor is built (`Beryl - Shared CPUs/README.md`).
- **`Observer` in `emusen-native`**, the trait through which a Beryl CPU reports to the shared `Hooks` at no cost when
  unobserved (`debug.rs`).
- **The single-step harnesses** in `beryl-m68k` and `beryl-z80`, with positive and negative controls run over the whole
  of both 68000 suites and the Z80 suite (`Beryl_M68k.md` §3, `Beryl_Z80.md` §3).
- **The system packs** for `md`, `mcd` and `32x` in `DianaOS/Sys/Systems/Genesis/` (§2.4).
- **The generic test-ROM runner** in `EmuSen.WiseMan/Fixtures/RomRunner/` and Nephrite's tests through it (§3).
- **The corpus, the referees and the references**, fetched and pinned outside the repository (§4).
- **The build's wiring**: `RustCores.props` names the crate with `CoreAbi 1` and `InDevelopment` true, so the build
  writes its sidecar marked for development: the tests and the kit run the core, a player's discovery never lists it,
  and a publish leaves it out until the Genesis gate (`EmuSen_CoreAPI.md` §27, added the same day after review; the
  first version of stage 0 offered the stub for any `.gen` file). `rust-cores.yml` builds, tests and export-checks it
  on the four platforms and tests the Beryl crates with the shared ones.

It wrote no emulation rule. The protocol of the plan's §1.3 applied throughout: no emulator source and no firmware was
read. The probe's own source was read to learn its command line and its dumps.

---

## 2. The crate

### 2.1 On the core ABI v1

- **`Core` and `core_exports!(Machine;)`**, with no optional group: `CAPABILITIES` is 0. Each capability is claimed in
  the stage that implements it, as VenusRT's were.
- **Core info**: id `nephrite`, name and display name `Nephrite` (no "(Rust)" suffix, since there is no C# engine to tell
  it from), three systems with their extensions, regions, the two pads of the plan's §4.2 and their firmware (§2.5).
- **Create.** An image shorter than 512 bytes, the cartridge's vectors and header, is refused with status −9 and words
  (an empty image included); any other image is accepted and its system decided by its contents (§2.2). File 0 is the
  battery file, copied into the battery memory clipped to its length; files 1 to 7 are the firmware's (§2.5), kept and
  not used; a higher number is `BAD_FILE`.
- **Machine info**: the system and region, the frame rate (§2.3), a picture of 320×224 with a maximum of 320×480 and a
  4:3 aspect, an audio rate of 48,000 Hz with no samples, both ports holding `md.pad3`, the spaces the system has (the
  plan's §4.3, zero-filled; `ROM` read-only, `WRAM` the cheats' space), the battery file (`.srm`, or `.brm` for the
  Sega CD's internal backup RAM), the state format, `skip_rendering_state_neutral` true, and each firmware file's path.
- **A frame** advances the frame count and nothing else; the picture is opaque black; the pads' bits are kept per port
  with the `changed` mask and are in no state.

### 2.2 The image, read from its bytes

`media.rs` reads the cartridge header at `$100` (system name, titles, serial, checksum, devices, the save-RAM
declaration at `$1B0` and the region field at `$1F0`), from the plan's §2.1 sources as layout; the field meanings below
are those documents' and are re-checked against the pinned copies at stage 3.

- **A disc** has `SEGADISCSYSTEM` at the start of its first sector's user data: offset 0 in a 2,048-byte-sector image,
  16 in a raw 2,352-byte one. Its header follows at `$100` of the system area.
- **A 32X cartridge** has `32X` in its header's system name. *Argued, unchecked:* a 32X image whose header does not say
  so would be taken for a Genesis cartridge; the 32X's own security header (the plan's §5.6) is the firmer test, read
  at stage 12.
- **The region field** in its old form (`J`, `U`, `E` letters) and its new single hex digit (bit 0 Japan, bit 2 the
  Americas, bit 3 Europe; `E` is read as the letter). The machine is PAL only when Europe is the one market allowed; a
  Sega CD's BIOS is chosen Americas first, then Japan, then Europe (the plan's §9, Q6).
- **Save RAM** declared by `RA` with its type byte: bit 6 the battery, bits 4–3 the lanes (both, even, odd), its size
  the range halved for one lane. Only battery-backed RAM becomes the battery file.

### 2.3 The frame rates (argued)

NTSC: 15 times the colour subcarrier (4,725,000,000/88 Hz) over 262 lines of 3,420 master clocks, 59.9227 Hz. PAL: 12
times 4.43361875 MHz (53,203,425 Hz) over 313 lines of 3,420, 49.7015 Hz. The documents quote the NTSC crystal as
53.693175 MHz, 0.1 ppm from the subcarrier's multiple; which to use is stage 3's to settle with its line timing, and the
pacing difference is far below anything a player could see. A test pins both values to four decimal places.

### 2.4 The system packs

`GenesisSystems` holds the three entries of the plan's §4.1: names, consoles (`Genesis`, `Sega CD`, `32X`), Sega and the
years (1988, 1991, 1994), the extensions, the libretro cheat folders (read from the libretro database's listing on
2026-10-04), OpenVGDB's systems (`MD`, `SCD`, `32X`, read from the database's `SYSTEMS` table), a cover aspect of 0.7
(argued), and ES-DE's names (read from ES-DE's `es_systems.xml` on 2026-10-04). `SystemPacks.All` lists them; their
codecs are null until stage 6, so a v1 engine on these systems gets no cheat codec, as before. `CoreCatalog` is
untouched: no console row exists for these systems, and Mistress's library does not list their files (the plan's §8.3).

### 2.5 Firmware

Every entry is `required: false` with `replacement` words, so the kit's C4 creates every image with no files:

| File | Name, size | System | Replacement words at stage 0 |
|---|---|---|---|
| 1 | `bios_MD.bin`, 2,048 | all three | `none`: a console without TMSS |
| 2, 3, 4 | `bios_CD_U.bin`, `_E`, `_J`, 131,072 | `mcd` | `none`: the replacement is not written; no disc runs either way |
| 5, 6, 7 | `32X_G_BIOS.BIN` 256, `32X_M_BIOS.BIN` 2,048, `32X_S_BIOS.BIN` 1,024 | `32x` | `none`: the replacements are not written |

`firmware_for` names nothing for a Genesis cartridge, the region's BIOS for a disc and the three boot ROMs for a 32X
cartridge; machine info reports each named file as `file` when given and `absent` otherwise. The file names are the
ones players' own dumps commonly carry.

### 2.6 The state, "NPHR"

Version 1: the magic `NPHR`, the version, the system (0 Genesis, 1 Sega CD, 2 32X), the frame count, then every
writable memory in id order. *Measured* by the kit: 147,681 bytes for a Genesis cartridge with 8 KiB of SRAM, 672,481
for a 32X cartridge, 999,649 for a disc. A state of another magic is refused as foreign, of another version as a
version, and of another of Nephrite's systems with status −10; a refused or truncated load changes nothing. The version
1 layout is pinned by `the_version_1_layout_is_pinned`.

### 2.7 What was measured (2026-10-04)

- **The crate's tests**: 10 pass (`cargo test --release`): the region forms, the system detection, the save RAM's size,
  the layout, the state round trip and refusals, each system's report and blank picture, the rates, a short image and
  the battery file, the read-only ROM and the pads, the descriptors.
- **The export check**: `export_check.py` on the release library: ABI 1.0, capabilities 0, the 32 required
  `emusen_core_` symbols and no other `emusen_core_` symbol; `nm -D` lists no other exported symbol.
- **The conformance kit**, `emusen-core-conform --frames 600` on three synthetic images (a Genesis cartridge with SRAM,
  a 32X cartridge, a 2,048-byte-sector disc): **C1–C15 pass on all three**, "EmuSen v1 compliant on this image". C4: the
  empty image refused with words, a garbage image and the half-length image accepted and run, every firmware entry
  optional. C9 and C15 have nothing to check (no exact setting, no `DEBUG`). C13 ran 7,561–8,504 rounds of library calls
  beside a machine.

---

## 3. The test-ROM runner

### 3.1 Engines

`EmuSen.WiseMan/Fixtures/RomRunner/`, not `TestRoms/`, a name the repository ignores everywhere so that dumped ROMs
are never committed. An `ITestRomEngine` runs a ROM from power-on to a list of frames, with presses on
port 0 in the probe's `--press` form, and returns a snapshot at each: the spaces by lower-case name, and the picture as
8-bit RGB whatever the engine's format.

- **`CoreAbiTestRomEngine`**: any library on the core ABI v1 through `CoreMachine`. Its spaces are machine info's, its
  presses go to the bits of port 0's controller whose canonical control is the button pressed. It names no type of any
  core: VenusRT, MoonRT or any later v1 core runs through it as Nephrite does.
- **`LibretroProbeEngine`**: a libretro core through the probe, as a process, each dump set cached by the ROM's MD5, the
  frames, the presses and both binaries' sizes and times; its snapshots are read from the probe's per-frame manifests,
  the screen converted from the format the manifest names (`Xrgb8888`, `Rgb565`, `Bgr555`).
- **`TestRomCorpus`**: a corpus root from a variable and its `unique-roms.txt`, read in either form (VenusRT's
  `md5 size path` or the Nephrite corpus's `md5 size kind path`, whose paths hold spaces).

### 3.2 Pictures

Compared over the rectangle both cover, exactly and **up to a one-to-one map of colours**: the second count is the
number of pixels that break a bijection between the two pictures' colours, zero when two engines differ only in
colour levels. The probe measured why both are needed: Genesis Plus GX, PicoDrive and BlastEm draw the 240p suite's
menu identically up to such a map, and an exact comparison reports 91–96% of pixels different (the plan's §1.6).

### 3.3 Grading

`TestRomGrader` takes protocols as data, each a test of the path and a verdict on the run; a ROM no protocol claims is
`Visual`. No Nephrite protocol is written at stage 0: the self-grading programs of the plan's §3.1 report in ways
(VDPFIFOTesting's RAM records, green boxes, text in plane tiles) that are written with the stages that can run them.

### 3.4 Nephrite's tests through it

`EmuSen.WiseMan/Cores/NephriteTests.cs`:

- the library loads, claims nothing, and its three systems' extensions are the system packs' entries, every firmware
  entry optional, both pads on each; `firmware_for` answers for a cartridge, a 32X cartridge and a Japanese disc;
- the stub runs each system through the generic runner: the spaces each system has, a black 320×224 picture standing
  still, 48 kHz, a `Visual` verdict, and the declared SRAM's size;
- a `.gen` file opens through `CoreFactory` on the discovered engine with no code naming it, when development cores are
  asked for: the generic adapter, the system and a PAL region from the header, 49.70 Hz, no codecs and no notice;
- without them, nothing offers the core: no extension is supported, claimed or listed by Mistress's library;
- the colour-map comparison and the probe's screen formats;
- the corpus (`EMUSEN_NEPHRITE_CORPUS`, opt-in): every ROM of the manifest taken by the stub and run, or refused with
  words;
- the references (the corpus and the probe's cores present): Genesis Plus GX and PicoDrive on the corpus's first
  cartridge to frame 600, through the same runner.

**Measured 2026-10-04.** The seven tests pass, with the corpus. Of the manifest's 105 files the stub took 95 and refused
10 with words, every refusal a file shorter than 512 bytes (palette and configuration blobs and three cue sheets,
which the manifest lists by extension). The two references on the corpus's first cartridge (Sega's multitap sample
program) at frame 600: their 68000 RAM differs in 20 bytes, their pictures in 7,174 of 71,680 pixels exactly and in 288
up to a colour map, an agreement of the order the probe's survey found and the first case for the gates' rule that
references are compared, not trusted.

**The registration golden** (`EmuSen_CoreAPI.md` §25), re-recorded with `EMUSEN_RECORD_REGISTRATION=1`, first gained
three lines (the discovered engine `nephrite` in the `built`, `off` and `refused-discovered` worlds) and, once the core
was marked in development, lost them again: it is byte-identical to its state before Nephrite, since a player's
discovery does not see the core. The tests that run Nephrite through discovery ask for development cores
(`CoreDiscovery.UseDevelopment(true)`), and `A_player_is_not_offered_the_core_in_development` checks the other side.

**The blast radius run**: WiseMan's `Nephrite`, `CoreDiscovery`, `CoreAdapter`, `CoreAbi`, `CoreRegistration` and
`SystemPack` tests, 67 of 67 after the re-recording; `emusen-native`'s 53 tests (one new, for `Observer`).

---

## 4. The corpus, the referees and the references

Recorded in full in the plan's §1.5, §1.6 and §3.1; in short, all outside the repository:

- `~/.cache/emusen/probe/nephrite/singlestep/`: SingleStepTests 68000 (`64b2531`, 317,500 cases), TomHarte's 680x0
  (`e0d5ece`, 1,000,060), SingleStepTests Z80 (`ebe1875`, 1,604,000).
- `~/.cache/emusen/probe/nephrite/roms/`: 105 ROM-like files, 91 distinct, with `PROVENANCE.txt` and
  `unique-roms.txt`; sources in `_src/`, archives in `_dl/`.
- `~/.cache/emusen/probe/nephrite/docs/`: the documents of the plan's §2.1 and §2.2, 329 files with their SHA-256 in
  `SHA256SUMS` and their sources, dates and status in `DOCS.txt`, whose first section lists what is excluded for
  replacement writers (the SpritesMind topic t=2061, a disassembly of the Sega CD BIOS) and five pages to be screened
  before such a writer reads them (the plan's §1.3).
- `~/Projects/megadrive-mister-reference` (`4b11b2c`), `megacd-mister-reference` (`a3a3da8`),
  `s32x-mister-reference` (`b438679`).
- `~/.cache/emusen/probe/libretro/cores/`: Genesis Plus GX, PicoDrive, BlastEm and ClownMDEmu from the libretro
  buildbot, with their `.info` files; the probe unchanged.

---

## 5. What the plan got wrong, found at stage 0

- **The 68000 suite's data bus.** The plan's first reading took a byte cycle's value as the byte. SingleStepTests' form
  gives it on its lane, so a byte on the upper lane (UDS) reads shifted left by eight, as the suite's README says; the
  replay control failed 29,003 of 317,500 cases until the harness read it so (`Beryl_M68k.md` §3.1).
- **TAS in TomHarte's suite** is one transaction of ten clocks, not the read, two idle clocks and the write that the
  other suite records; the harness expands it.
- **The probe needed no work to run these cores**, against the expectation that a Genesis core might need new backend
  code: it ran all four unchanged. What it needs is options, memory maps and a sturdier loader (the plan's §1.6).

## 6. Negative results

- No SH-2 single-step suite exists; no homebrew 32X hardware test was found (the plan's §3.1).
- No reference exposes VSRAM or the VDP's registers through the standard calls.
- No reference but ClownMDEmu runs a disc without a BIOS, and it did not get past the 240p suite's loading screen.

## 7. What stage 0 does not cover

- No emulation: every Nephrite verdict in the runner is `Visual`, correct for a blank machine and saying nothing yet.
- No speed measurement of Nephrite; the references' times are the plan's §5.5.
- The documents are pinned by hash but not yet read for the design; each stage cites the sections it reads.

## 8. Stage 2, the probe's work (2026-10-05)

The plan's §1.6 asked five things of the reference probe's libretro backend (`EmuSen.WiseMan/Reference/probe-rs`,
`src/backends/libretro.rs`). Each is built; the references are run as black boxes throughout, their outputs read and
nothing else.

- **Core options.** The backend keeps the options each core declares (`SET_VARIABLES`, `SET_CORE_OPTIONS` and its V2,
  with their international forms; `GET_CORE_OPTIONS_VERSION` is answered 2). It answers `GET_VARIABLE` with the value
  pinned by `--option KEY=VALUE`, or else the default the core declared; a key the core never declared gets no
  answer. Stage 0's probe answered nothing, and a core refused an answer falls back on defaults of its own, which need
  not be the declared ones: ClownMDEmu came up Japanese that way. Every run prints each option with its value and
  whether it was pinned; a pinned key the core does not declare, or a value it does not list, is a warning;
  `--list-options` prints the allowed values and stops. **Measured:** Genesis Plus GX declares 62 options, PicoDrive
  21, BlastEm 21, ClownMDEmu 32. ClownMDEmu now runs its declared `clownmdemu_overseas_region=elsewhere`; pinned to
  `japan`, its picture at frame 600 of the 240p suite changes and its RAM differs in 19 bytes.
- **Memory maps as named spaces.** `SET_MEMORY_MAPS`'s descriptors become spaces named by where they start
  (`map_ff0000`), beside the standard ones. A core that declares or replaces its maps after a frame has its spaces
  rebuilt then, and the probe says so. **Measured:** ClownMDEmu declares five at load, among them `map_ff0000` (64 KiB)
  and `map_a00000` (8 KiB); PicoDrive declares its 32X maps at frame 3 of the 32X 240p suite, `map_6000000` among
  them (256 KiB). `--memory-id NAME=ID` dumps a memory id the ABI does not name: PicoDrive's id 4 is 128 bytes, its
  CRAM.
- **The loader.** libm is opened into the global namespace before the core, on Linux; the loader's error is printed,
  as before; `--system NAME` names the machine where the extension cannot (`.bin`, `.cue`, `.iso`), and `.32x` is
  read as the 32X.
- **The cores, pinned by hash.** `EmuSen.WiseMan/Reference/libretro-cores.sha256` pins each reference core by the
  SHA-256 of its unpacked binary, per buildbot directory. `build-probe.sh libretro-core <core>` uses a cached copy
  only when its hash matches. Otherwise it fetches the buildbot's nightly into `~/.cache/emusen/probe/libretro/cores`,
  unpacks only the core's binary, and refuses a nightly whose hash has moved on, keeping nothing of it; re-pinning is
  a deliberate edit of the file. Fedora packages none of the four. **Measured:** the four cached cores match their
  pins; a fresh fetch of ClownMDEmu matched; a deliberately wrong pin was refused with the download deleted.
- **Firmware only from a person's own dumps.** `GET_SYSTEM_DIRECTORY` is the folder `--sysdir` names, or without it
  an empty folder beside the run's dumps; the probe fetches no firmware from anywhere. A run whose system needs
  firmware (the Sega CD), refused by the core while `--sysdir` is absent or holds no file, is reported `[SKIP]` and
  exits 4. WiseMan's `LibretroProbeEngine` turns that into `FirmwareSkippedException`, and takes pinned options,
  `--system` and `--sysdir` into its cache key. **Measured with a blank disc image**, there being no BIOS on the
  machine: Genesis Plus GX, PicoDrive and BlastEm are skipped, with no `--sysdir` and with an empty one, and the empty
  folder stays empty; ClownMDEmu boots its own replacement and runs.
- **The picture gate up to a colour map** was built at stage 0 (§3.2).

**Agreement after the change (measured 2026-10-05).** On the 240p suite's menu at frame 600, with the declared defaults
answered, the pairs of references agree in 65,326–65,532 of 65,536 bytes of the 68000's RAM, as at stage 0
(65,325–65,530).

**A consequence beyond Nephrite.** Every core the libretro probe drives now receives its declared defaults rather than
no answer. For a core whose built-in defaults are the declared ones nothing changes; the reference-dump fixtures
recorded earlier for other machines were not re-recorded in this step, and `LibretroProbeEngine` re-makes its cached
sets because its key includes the probe's build.

Pinned by `probe-rs`'s unit tests (the option answer's order of precedence, the legacy variable's default, the map
spaces' names, the firmware folder, the system from the extension) and by `NephriteTests`'
`The_reference_probe_skips_a_sega_cd_run_without_the_players_firmware` and `The_reference_probe_pins_a_core_option`.
