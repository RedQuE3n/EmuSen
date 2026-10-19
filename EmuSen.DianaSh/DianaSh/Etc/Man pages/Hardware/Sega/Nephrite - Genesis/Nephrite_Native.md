# Nephrite_Native — the build record

*Started 2026-10-04, at stage 0.* `Nephrite_Plan.md` says what Nephrite is to be and how it is graded; this page
records what was built, stage by stage, what was measured, and where the plan turned out to be wrong. The plan's three
registers are kept: **measured** is a number taken here, on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10, Rust
1.98.1), with the tool named beside it; **argued** is reasoning with no measurement behind it; **predicted** is one of
the plan's P1–P8, retired where a measurement reaches it.

The CPUs' records are their crates' own pages in `../Beryl-HW/`.

---

## 1. Stage 0 (2026-10-04)

The plan's §6 row 0. What it built:

- **The naming**, decided by the tester on 2026-10-04: Nephrite is the Genesis with the Sega CD and the 32X as its
  attachments, its folders renamed from `Nephrite - 32X` to `Nephrite - Genesis`; Beryl is the shared CPU library, its
  folders renamed from `Beryl - Genesis` to the library's own name, and on 2026-10-05 to `Beryl-HW` (`EmuSen_Core_Naming_Scheme.md` §3, with the reasoning
  for the crate's plain name).
- **The crate** `nephrite`, `EmuSen/Cores/Sega/Nephrite - Genesis/`, on the core ABI v1 from its first commit, with a
  stub machine and its own state format (§2). No part of any console is emulated.
- **The three Beryl crates**, `beryl-m68k`, `beryl-z80` and `beryl-sh2`, as skeletons: each processor's bus trait,
  registers and step signature, which refuses to step until the processor is built (`Beryl-HW/README.md`).
- **`Observer` in `emusen-native`**, the trait through which a Beryl CPU reports to the shared `Hooks` at no cost when
  unobserved (`debug.rs`).
- **The single-step harnesses** in `beryl-m68k` and `beryl-z80`, with positive and negative controls run over the whole
  of both 68000 suites and the Z80 suite (`Beryl_M68k.md` §3, `Beryl_Z80.md` §3).
- **The system packs** for `md`, `mcd` and `32x` in `DianaSh/Sys/Systems/Genesis/` (§2.4).
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
  at stage 12. *Measured on 2026-10-19* over the 946 `.bin` files of the tester's `genesis/` folder: 18 headers name the
  32X, the same 18 whose file names say 32X, and no other; routing reads the same rule (`EmuSen_Backend.md` §32).
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

## 9. Stage 3, step 1: the Genesis's buses, the cartridge, I/O, the scheduler and the interrupts (2026-10-05)

### 9.1 What it built

- **`src/genesis.rs`**: the 68000 and the Z80 on one master clock (the 68000 at ÷7, the Z80 at ÷15) with the VDP's
  lines of 3,420 master clocks, 262 a frame in NTSC and 313 in PAL. The 68000 runs instruction by instruction; after
  each, the VDP's line events up to the present are taken and the Z80 is run up to the same time. The 68000's map is
  MacDonald's "Sega Genesis hardware notes" §1: the cartridge to `$3FFFFF`, the Z80's space at `$A00000` while the
  68000 holds its bus, I/O at `$A10000`, BUSREQ at `$A11100`, RESET at `$A11200`, the cartridge's registers at
  `$A130F0`, TMSS at `$A14000`, the VDP in its mirrors (bits as MacDonald's `110n n000 nnnn nnnn 000m mmmm`), and RAM
  mirrored from `$E00000`. The Z80's map is §2.1 there: its RAM mirrored to `$3FFF`, the YM2612 (reading not busy
  until stage 5), the bank register shifting a bit in from the top at each write, the VDP at `$7F00`, and the window at
  `$8000` onto the 68000's space. Releasing RESET starts the Z80 from its reset state.
- **`src/cart.rs`**: ROM with mirrors (an address masked to the ROM's size rounded up to a power of two, `$FF` past
  its end); save RAM on its declared lanes and range, mapped always when it lies above the ROM's end and through bit 0
  of `$A130F1` otherwise, bit 1 protecting it (plutiedev's "Saving progress with SRAM" and "Going beyond 4MB"); the
  Sega mapper's eight 512 KiB banks for "SEGA SSF" or an image above 4 MiB, bank 0 fixed.
- **`src/io.rs`**: the version register, the three ports' data, control and serial registers as MacDonald lists them
  at power-on, and the pads: TH's multiplexing, and the six-button pad's count of TH's falls with its timeout.
- **`src/vdp.rs`**, the VDP's interface without its timing: the ports and registers, VRAM, CRAM and VSRAM written at
  once, the three DMAs done at once, the status register with the FIFO always empty, the HV counter, the line counter
  and the two interrupts as MacDonald's "Sega Genesis VDP documentation" §4 gives them. Stage 4 replaces all of it.
- **The state, version 2**: the memories, then both CPUs' blocks, the VDP's registers and latches, the ports and the
  pads' counts, the cartridge's registers, the clocks and the lines, read whole before any of it is applied.

### 9.2 Readings

Each is a rule in hardware terms that the documents leave open or that a test showed; *argued* unless measured.

- **The open bus is the last word the 68000 read in program space**, which by an operand's read is the next
  instruction (its prefetch). An unmapped read returns that word's high byte with the low byte zero, for bytes and
  words alike, as MacDonald's notes say and as the expected values his memory test prints beside its results show.
  Genesis Plus GX and BlastEm return the whole word to a word read; the document is followed. That was seen by running
  them as black boxes through the reference probe on `memtest_68k.bin` to frame 600: in both, the test's record at
  `$FF0000`, its word read of `$400000`-`$7FFFFF`, holds `$4E71` in the 68000's RAM (Nephrite holds `$4E00`), and their
  pictures show `4E71` in that row. The VDP's unused
  addresses return the whole word, its status register's top six bits are taken from it, and `$A11100` reads it with
  bit 0 (or 8) the bus's state. **Measured** with the memory test (`memtest_68k.bin`, no source): twelve of its
  thirteen rows match the values printed as expected. The thirteenth is open (§9.4).
- **Lock-ups are not modelled** but for TMSS: a write or read MacDonald says locks the machine is ignored, or reads the
  open bus.
- **The Z80's window reaches the 68000's RAM**, reads included. MacDonald's console returned `$FF` to a read, and he
  reports Steve Snake saying it works; the references' RAM at anchors is the test (§9.3).
- **The window's waits are provisional**: three T-states for the Z80 and three 68000 clocks for the 68000 per access
  (the plan's P5; §2.3 lists the Z80's waits as thin). BUSREQ is granted at the 68000's next instruction boundary.
- **The interrupts.** The vertical interrupt is raised at line 224 (240 in PAL's V30) 128 master clocks in, MacDonald's
  "roughly at H counter cycle 08h"; the line counter is decremented at the start of every line to 224 and reloaded
  from register 10 when it expires and on the lines after. The Z80's INT is raised with it and held for one line, a
  pulse the Z80 misses with interrupts disabled. The 68000 is autovectored and an acknowledge clears the level's
  pending flag. The external interrupt (level 2) is not built.
- **The model**: the version register says overseas unless the header names Japan alone, PAL when it names Europe
  alone, no expansion unit, and version 0, a console without TMSS. A model with TMSS (version 1 on) locks the VDP
  until "SEGA" is written at `$A14000`, a VDP access before that hanging the machine as the hardware does; it needs no
  boot ROM, and the player's own image stays optional (`EmuSen_Firmware.md` §0). Only the default is reachable until
  the model becomes a setting at stage 6.
- **The six-button pad** forgets its count 8,192 68000 clocks after TH's last fall, MacDonald's "about 8192 (probably
  less)". The pad's kind is the three-button one until the frontend can choose it.

### 9.3 Measured (2026-10-05)

- **The crate's tests**: 23 pass, among them the cartridge's mirrors, lanes, register and mapper; the ports at
  power-on, both pads; the VDP's ports, fill and line counter; and the layout of state version 2.
- **The console test programs** (`src/programs.rs`, through `EMUSEN_NEPHRITE_ROMS`), each verdict read where its source
  says the program leaves it: **the BCD verifier, no failure** in any of its six counts (finished at frame 538); **the
  opcode sizes, both CRCs match** (frame 683); **the illegal-instruction test, green** (frame 10).
- **The 68000's RAM against Genesis Plus GX** (bytes of 65,536 equal; the references expose RAM with each word's bytes
  swapped, which the comparison undoes):

| Game | Frame 120 | Frame 600 |
|---|---|---|
| Super Street Fighter II (the mapper) | 65,536 | 65,536 |
| Streets of Rage 2 | 65,536 | 65,470 |
| Sonic the Hedgehog 2 | 65,534 | 65,470 |
| Columns | 65,472 | 65,498 |
| Castlevania: Bloodlines | 65,421 | 65,488 |
| Phantasy Star IV (save RAM over ROM) | 65,487 | 65,477 |
| Sonic the Hedgehog | 65,523 | 65,150 |
| Thunder Force IV | 65,177 | 65,178 |

  The differences are expected where the VDP's and the sound chips' timing feed a game (stages 4 and 5).
- **The cost**: 600 frames of a game in 0.11–0.20 s on the desktop, under 0.35 ms a frame with both CPUs, against
  the plan's 1.5 ms for the Genesis (`examples/dump.rs` times it).
- **The conformance kit**, `--frames 600`, on the 240p suite, Sonic the Hedgehog and Phantasy Star IV: C1–C15 pass on
  all three. WiseMan's Nephrite tests, registration and discovery: 19 pass.

### 9.4 What the next steps owe

- The memory test's first `$A11100` row: the hardware's values are "busy" and then "granted", Nephrite's "busy"
  twice; the BUSREQ and RESET sequence the test makes is not known without its source, so it waits for the test's
  own screen at stage 4 or a dispute step.
- Serial EEPROM boards (by serial, MacDonald's §4.1 and his `eeprom.txt`), Sonic & Knuckles lock-on (a second image),
  the window's waits measured, BUSREQ's latency, the external interrupt, the six-button pad chosen by the frontend, and
  the references at anchors through WiseMan's runner over the game corpus.

## 10. Stage 3, step 2: EEPROM, lock-on, the pads chosen, the external interrupt, the anchors (2026-10-05)

### 10.1 Serial EEPROM boards

`src/eeprom.rs`: a two-wire EEPROM, the X24C01's protocol as MacDonald's commented Monster World routines drive it
(`eeprom.txt`): a start (SDA falling while SCL is high), a byte of seven address bits and R/W, data bytes each followed
by an acknowledge on the ninth clock, writes in pages of four that wrap within the page, sequential reads, a stop. A
board is known by its serial, the header saying only that it has one: Sega's Technical Overview (§5's header table,
field 10) defines the field as `'RA', %1x1yz000, %00100000`, the last byte `$20` for RAM, and the boards with an
EEPROM carry `$40` there, which is what Monster World's routine tests. The documents give two boards' wiring, both an
X24C01 of 128 bytes with SDA on bit 0 and SCL on bit 1 of `$200001` (`gen-eeprom.txt`, `gen-hw.txt` §4.1): Monster
World III / Wonder Boy in Monster World (G-4060) and Mega Man: The Wily Wars / Rockman Megaworld (T-12046, T-12053).
The EEPROM's bytes are the battery's; its transfer in progress is not kept in a state.

**Owed:** the other boards whose headers mark an EEPROM, sixteen in the corpus (NBA Jam, NFL Quarterback
Club and its '96, College Slam, Frank Thomas Big Hurt Baseball, Evander Holyfield's Real Deal Boxing, Greatest
Heavyweights, Putter Golf, Dodge Ball, MLBPA Sports Talk Baseball, and Accolade's Barkley 2, Brett Hull Hockey '95,
Jack Nicklaus' Power Challenge Golf, Pelé!, Pelé's World Tournament Soccer, Unnecessary Roughness '95), whose wiring no
document in the corpus gives. SpritesMind topic 2227 names a document of Eke's on the Genesis's save EEPROMs; it is
to be fetched into the corpus as a document. Until then those boards run without their EEPROM. *Fetched and wired at
step 3, §11.4.*

### 10.2 Sonic & Knuckles' lock-on

Plutiedev's "Sonic & Knuckles Lock-on": the cartridge on top shows at `$200000`-`$3FFFFF`, its own upper 2 MiB, mirrored
as its own ROM mirrors; `$A130F1`'s bit 0 maps the 256 KiB patch ROM over `$300000`-`$3FFFFF` and is passed on to the
cartridge on top, whose save RAM it switches (Sonic 3's). The cartridge on top and the patch ROM are files 8 and 9 beside
the image, the player's own cartridges and not firmware, declared for Sonic & Knuckles (`GM MK-1563`) only; a combined
dump is split at 2 MiB, the patch ROM taken from its end when what follows is a whole number of mebibytes and 256 KiB.
**Sonic & Knuckles alone**: its upper 2 MiB is the empty slot and reads the open bus, where at first it mirrored the
cartridge's own 2 MiB, which made the game find its own header there as if a cartridge were on top. **Measured** (bytes
of the 68000's RAM equal to Genesis Plus GX's, of 65,536):

| Image | Frame 120 | Frame 600 | Frame 1200 |
|---|---|---|---|
| Sonic & Knuckles, slot mirrored (before) | 24,364 | 21,411 | 21,287 |
| Sonic & Knuckles, slot empty | 65,517 | 65,436 | 65,502 |
| with Sonic 1 (Blue Spheres) | 65,451 | 65,447 | 65,453 |
| with Sonic 2 and the patch ROM | 65,432 | 65,358 | 47,339 |
| with Sonic 3 | 65,532 | 65,485 | 65,496 |
| Wonder Boy in Monster World (EEPROM) | 65,525 | 65,528 | 65,518 |
| Mega Man: The Wily Wars (EEPROM) | 65,536 | 65,534 | 65,503 |
| Rockman Megaworld (EEPROM) | 65,536 | 65,493 | 65,486 |

Sonic 2 locked on parts from the reference by frame 1200; not yet examined. *Examined at step 3, §11.3: Sonic 2's own
lead, not the lock-on's.*

### 10.3 The pads chosen, and the external interrupt

- **The pads.** Two settings, `pad1` and `pad2` ("Port 1 controller"), each `md.pad3` or `md.pad6`, applied between
  frames; the port's controller in the machine information follows them. The core ABI has no call for a controller,
  so the frontend chooses through these.
- **The external interrupt.** A device driving TH on a port where TH is an input and the port's control bit 7 is set
  raises level 2 when register 11's bit 3 enables it, and with register 0's bit 1 set the HV counter is latched and
  read as latched (MacDonald's VDP document §4). Pads drive no TH, so nothing raises it yet; a light gun will.
- **The Z80's INT** lasts one line: Sega's Technical Overview (p. 91) gives the interrupt as "generated 16ms period
  and 64ms length", read as 64 µs.

### 10.4 The window's waits and BUSREQ's latency

No document and no test program gives them; `Nephrite_Disputes.md` D-1 opens the dispute step, with Nuked-MD's whole
board run as a black box. Its bench builds and boots and is not yet measuring; the provisional values of §9.2 stand.
*Measured at step 3, §11.1.*

### 10.5 The references at anchors through WiseMan's runner

`NephriteTests.The_ram_against_genesis_plus_gx_at_anchors_over_the_games`, opt-in through `EMUSEN_NEPHRITE_GAMES`
(a folder of games, the corpus's copy), runs every game through Nephrite's library and Genesis Plus GX's through the
probe at frames 120 and 600 and writes the bytes of the 68000's RAM that agree to `EMUSEN_NEPHRITE_REPORT`.
`LibretroProbeEngine` takes the spaces a reference holds as little-endian words and puts them back in the 68000's
order. **Measured 2026-10-05 over the corpus's 946 games**, in 5 min 41 s: 944 compared; the two others (a 32X
image and an unlicensed one) crashed the reference's probe. Bytes of the 68000's RAM equal, of 65,536:

| Frame | Median | 25th percentile | 10th percentile | Identical | 65,000 or more | Below 60,000 |
|---|---|---|---|---|---|---|
| 120 | 65,518 | 65,482 | 65,324 | 118 | 879 | 9 |
| 600 | 65,489 | 65,376 | 64,514 | 36 | 828 | 40 |

The lowest at frame 600 are games whose logic the VDP's or the sound chips' timing reaches (Ninja Gaiden, Sub-Terrania,
Mega Turrican, Bubba 'n' Stix), two EEPROM boards the documents do not wire (§10.1: Greatest Heavyweights, Evander
Holyfield's Real Deal Boxing), and a Sonic 3 image of 2,097,664 bytes, a copier's 512-byte header before the ROM,
which Nephrite does not strip yet. *Stripped from step 3, §11.2.*

### 10.6 Measured

The crate's tests: 29 pass (EEPROM, lock-on, the pad settings, the external interrupt and the latch, and §9's, the
three console test programs included). The state layout grew by the external interrupt's latch and the HV latch. The
conformance kit passes C1-C15 on Sonic & Knuckles alone, with Sonic 3, and on Mega Man: The Wily Wars; WiseMan's
Nephrite, runner, registration and discovery tests, 31, pass.

## 11. Stage 3, step 3: the window measured, copier images, Sonic 2's lead, the EEPROM boards (2026-10-05)

### 11.1 The window's waits and BUSREQ's latency, measured

`Nephrite_Disputes.md` D-1 is settled by Nuked-MD's board run as a black box. The `$FFFF`s of step 2 were the bench's
own: the board leaves the cartridge's `cart_cs` and `cart_oe` low on cycles that are not the cartridge's, and the
bench's cartridge drove the bus whenever they were, so the 68000's RAM took `$FF`s. Its cartridge now decodes its own
4 MiB, as a cartridge does. **Measured:** a window access costs the Z80 2.75 T-states on average, reads and writes
alike, and the 68000 9.5 of its clocks; BUSREQ was granted by the 68000's first poll in every one of 1,406 requests.
Nephrite takes the means, 41 and 66 master clocks (`genesis.rs`), where it had the plan's provisional three T-states
and three clocks, and keeps the grant at the next instruction boundary. The wait's dependence on where the Z80's access
meets the 68000's bus cycle (the Z80's rounds fall on the 68000's clock, 2.5 to 3.5 T-states of wait) is not modelled.

**The bench.** `EmuSen.WiseMan/Reference/rtl68k/`: `build-referees.sh board` builds `tb_md.cpp` against Nuked-MD's
`md_board` (Verilator, the VRAM's FPGA memory block replaced by `vram_ip.v`) into `~/.cache/emusen/probe/nukedmd-board`;
`python3 mdboard.py <program> [MCLK2 cycles]` writes a hand-assembled cartridge, runs it and reads the pins. Programs:
`window` and `window-write` (the Z80 reading or writing the cartridge through the window beside the 68000 counting in
its RAM, against a control with the Z80 in its own RAM), `busreq` and `busreq-tight` (a request and polls, marked in
RAM), `rom-loop` (§11.3). The bench prints `c` for a cartridge read, `x` for a write to its range and `w` for a write
of the 68000's RAM, with the time in MCLK2 cycles (twice the master clock); `TB_TRACE=from:to` prints the pins each
cycle. The RAM's address lines are `{ VA14, IA14, VA12-VA0 }`, so `$FF1000` shows as `$5000`.

**The anchors** over the corpus, before and after (bytes of the 68000's RAM equal to Genesis Plus GX's, the medians
unchanged at 65,518 and 65,490): of 944 games, 35 rose and 28 fell at frame 120, 77 rose and 71 fell at frame 600.
One fell far, Barver Battle Saga (65,391 to 16,302 at frame 600): its Z80 writes `$400004` and reads `$400006`
through the window, a protection device that no document in the corpus describes; Nephrite answers with the open
bus, the 68000's prefetch, so the value the game indexes a table by follows where the 68000 is when the Z80 reads, and
the new stall moved it to one that sends the 68000 into an address error. The device is owed to a document.

### 11.2 Copier images

`media::cartridge_bytes`, applied to the image and to the cartridge on top of Sonic & Knuckles before anything reads
them. A Super Magic Drive image (d0nut and Felipe XnaK, "The complete documentation about Genesis ROM format" 1.1,
1998, now in the corpus) is a 512-byte header (the number of 16 KiB blocks, `$03`, the split flag, `$AA $BB` at bytes
8 and 9) before 16 KiB blocks, each holding one half of the cartridge's bytes and then the other. An image is taken
for one when its size is 512 bytes past a whole number of blocks, its own `$100` does not name SEGA, its header
carries `$AA $BB`, and the deinterleaved image names SEGA at `$100`; one with a header and no interleave is stripped
when what follows names SEGA. Anything else is left as it is. **The document's even and odd are counted from one:**
its "even bytes at the beginning" are the bytes at odd addresses, as the corpus shows: the five copier images in it
(Sonic the Hedgehog 3 (E), Sub-Terrania, Mega Turrican, Bubba 'n' Stix, D&D: Warriors of the Eternal Sun, all named
`.bin`) read `SEGA GENESIS` or `SEGA MEGA DRIVE` at `$100` with the first half at odd addresses and garbage the other
way. `.smd` joins the Genesis's extensions, in the core and in the system pack. The `.md` format of the same document
(the whole image interleaved, no header) is not detected: nothing in its bytes tells it from a plain image, and the
corpus's `.md` files are plain.

| Image | Frame 120, before | after | Frame 600, before | after |
|---|---|---|---|---|
| Sonic the Hedgehog 3 (E) | 57,644 | 65,532 | 13,564 | 65,496 |
| Sub-Terrania (E) | 33,014 | 65,532 | 14,101 | 48,145 |
| Mega Turrican (E) | 55,807 | 65,513 | 14,643 | 65,494 |
| Bubba 'n' Stix (U) | 61,411 | 65,510 | 19,752 | 65,531 |
| D&D: Warriors of the Eternal Sun (U) | 65,187 | 65,526 | 20,827 | 65,529 |

### 11.3 Sonic 2 locked on, and Sonic 2 alone

The difference of step 2 (47,339 bytes at frame 1200) is not the lock-on's. Nephrite's RAM at frame 1191 matches
Genesis Plus GX's at 1200 in 64,776 bytes: the game is the same, nine frames ahead, at a moment when a level is being
loaded and the RAM changes fast. Sonic 2 alone runs ahead in the same way, by 12 frames from frame 600 on. The lead
starts at boot: the game's checksum of its own ROM (`$326`-`$33C`: `add.w (a0)+,d1`, `cmp.l a0,d0`, `bcc.s` over
524,032 words), a loop of the 68000 alone, ends in Nephrite at frame 101, and the vertical-interrupt counter at
`$FFFE0C` first counts at frame 114 in Nephrite and 118 in Genesis Plus GX; further leads come at the later loads.
Neither the window's waits (0, 41 or 120 master clocks) nor a stall for DMA (33 or 66 master clocks a word) changed
the lead; both were tried and reverted.

The loop takes 24 clocks a round by the 68000's manual, which is what Nephrite charges. **Nuked-MD's board takes
24.38** (`mdboard.py rom-loop`): one round in about five, 446 of 2,341, takes 26, one loss of two clocks every 128.0
clocks, the main RAM's refresh period that the MegaDrive Wiki gives. The board puts the real console about 1.5 frames
behind Nephrite over the checksum and Genesis Plus GX about 3 frames behind the board, so neither reference's lead is
the hardware's. `Nephrite_Disputes.md` D-2 opens the refresh: the cartridge's loss is clear, the RAM's (24.94 a round,
in losses of two, three and five clocks) is not yet separated, and Nephrite takes no refresh until it is.

### 11.4 The EEPROM boards of Eke's document

Eke's document, "Serial EEPROMs in Sega Genesis / Mega Drive cartridges" (version 2, 2010), is in the archive
SpritesMind topic 2227 links (`md_tech.zip`), fetched on 2026-10-05 and recorded in the corpus's `DOCS.txt` with its
URL, date and hashes; only it and the archive's EEPROM data sheets were taken out of the archive, and the archive's
one source file, a disassembler plug-in's, was not opened. It describes the protocol in three modes (seven address bits
in the command byte; a word address after it, with the command's three device bits above it, for 24C01-24C16; two
address bytes after it for 24C32 and up), each read going on from an address counter that a write's address sets, and
lists the games by company with their lines, mode, size and page. It names games, not serials; the serials are read
from the corpus's images. `eeprom.rs` now carries a board's three lines separately (SDA written, SDA read, SCL), its
mode, size and page; a word write that reaches both of a board's lines changes them together, as one bus cycle does.

Wired, by serial (21 boards):

| Company | Lines | Games in the corpus (chip) |
|---|---|---|
| Sega, Capcom | SDA bit 0, SCL bit 1, `$200001` | Wonder Boy in Monster World, Monster World III, Mega Man: The Wily Wars, Rockman Megaworld, Evander Holyfield's Real Deal Boxing, Greatest Heavyweights, Honoo no Toukyuuji Dodge Danpei (the corpus's "Dodge Ball"), MLBPA Sports Talk Baseball (X24C01) |
| Acclaim, first type | SDA in bit 0, SDA out and SCL bit 1, `$200001` | NBA Jam (24C02) |
| Acclaim, second type | SDA bit 0 of `$200001`, SCL bit 0 of `$200000` | NBA Jam Tournament Edition, NFL Quarterback Club (24C02); NFL Quarterback Club 96 (24C16); College Slam, Frank Thomas Big Hurt Baseball (24C64) |
| Codemasters | SDA in bit 0 and SCL bit 1 of `$300000`, SDA out bit 7 of `$380001` | Brian Lara Cricket (X24C01); Micro Machines 2 (24C08); Micro Machines Military (24C08), Micro Machines 96 (24C16), told by checksum from Micro Machines, which shares their serial and has none; Brian Lara Cricket 96, Shane Warne Cricket (24C64) |
| Electronic Arts | SDA bit 7, SCL bit 6, `$200001` | John Madden Football '93 Championship Edition, NHLPA Hockey '93, Rings of Power (X24C01) |

Not wired, because the document does not list them: Putter Golf and Accolade's six (Barkley Shut Up and Jam 2, Brett
Hull Hockey '95, Jack Nicklaus' Power Challenge Golf, Pelé!, Pelé's World Tournament Soccer, Unnecessary Roughness
'95). Not wired because the corpus has no image to read a serial from: NBA Jam (J), Bill Walsh College Football, John
Madden Football '93, Ninja Burai Densetsu. Blockbuster Competition 2 carries NBA Jam Tournament Edition's serial and
header and is wired as it is.

**Brian Lara Cricket 96's page.** The document leaves it open ("PAGE_MASK: ?"). The 24C65 data sheet in the same
archive gives eight-byte pages gathered in a cache of 64 bytes, a write's six low address bits counting; with pages of
eight, the game's signature landed in pieces and its RAM fell to 63,561 bytes at frame 600, and with the datasheet's
64 its EEPROM holds what Genesis Plus GX's holds (`31 41 59 26 00 96`, then zeros) and its RAM is back to 65,529.

**Measured** against Genesis Plus GX at frames 120 and 600 (its save memory as the probe dumps it beside the RAM):
the EEPROM's bytes are identical at frame 600 for Micro Machines 2 (928 bytes written), Frank Thomas (8,190), John
Madden '93 (127), NHLPA Hockey '93 (127) and Shane Warne Cricket, and College Slam's agree in their first 6,632 of
8,192 bytes. In Micro Machines Military and Micro Machines 96 the game writes its signature (`hasreset`) in Nephrite
and nothing in Genesis Plus GX, whose memory stays `$FF`, which looks like a reference that does not wire these two
images; the document is followed. The RAM, before the boards and the copier headers (with §11.1's window) and after:

| Game | Frame 120, before | after | Frame 600, before | after |
|---|---|---|---|---|
| Greatest Heavyweights of the Ring | 60,506 | 65,534 | 26,086 | 65,503 |
| Evander Holyfield's Real Deal Boxing | 63,653 | 65,535 | 33,165 | 65,490 |
| College Slam | 63,964 | 65,435 | 55,763 | 65,524 |
| NBA Jam Tournament Edition | 63,965 | 65,438 | 57,600 | 65,490 |
| NFL Quarterback Club 96 | 65,407 | 65,520 | 59,232 | 63,151 |
| Frank Thomas Big Hurt Baseball | 65,410 | 65,515 | 63,869 | 65,397 |
| Brian Lara Cricket | 62,228 | 65,345 | 64,503 | 65,521 |
| Honoo no Toukyuuji Dodge Danpei | 65,507 | 65,507 | 64,917 | 65,520 |
| Rings of Power | 59,627 | 62,670 | 65,483 | 65,483 |
| NHLPA Hockey '93 | 65,324 | 65,324 | 64,588 | 65,011 |
| Micro Machines 2 | 65,496 | 65,496 | 56,286 | 57,466 |
| Micro Machines Military | 65,505 | 65,505 | 58,000 | 56,624 |
| Micro Machines 96 | 65,509 | 65,509 | 54,451 | 46,719 |
| Blockbuster Competition 2 | 63,954 | 65,438 | 64,043 | 58,772 |

The two Micro Machines that fell are the two whose EEPROM the reference leaves untouched. Blockbuster Competition 2
rose at frame 120 and fell at 600, when its menu has passed to a game; it is wired by NBA Jam Tournament Edition's
serial, which it carries, and the document does not list it. NBA Jam, Shane Warne Cricket, Brian Lara Cricket 96 and
MLBPA Sports Talk Baseball are unchanged within 10 bytes.

**Over the corpus**, step 2's report against this step's (944 games compared; the 32X image and the unlicensed one
still crash the reference's probe):

| Frame | Median | 25th percentile | 10th percentile | 65,000 or more | Below 60,000 |
|---|---|---|---|---|---|
| 120, step 2 | 65,518 | 65,483 | 65,324 | 879 | 9 |
| 120, step 3 | 65,518 | 65,486 | 65,390 | 888 | 6 |
| 600, step 2 | 65,489 | 65,377 | 64,514 | 828 | 40 |
| 600, step 3 | 65,491 | 65,384 | 64,849 | 839 | 33 |

Of the window's change alone (§11.1) the medians did not move; the copier images and the EEPROM boards are what the
percentiles show. Against the run with the window's change alone, the copier headers and the boards raised 17 games at
both anchors and lowered 4 at frame 600: the three above and John Madden '93 by 20 bytes.

### 11.5 Measured

- **The crate's tests**: 33 pass, among them the copier header (stripped, deinterleaved, left alone when neither
  names SEGA), mode 2 with the command's upper bits and a random read, mode 3 on Codemasters' lines, a word write that
  changes both lines at once, and the boards by serial and checksum; the three console test programs still pass.
- **The conformance kit**, `--frames 600`: C1-C15 pass on Sonic the Hedgehog 3 (E) (a copier image), College Slam,
  Shane Warne Cricket, NFL Quarterback Club 96 and Sonic the Hedgehog 2.
- **WiseMan**: the Nephrite, runner, registration and discovery tests, 31, pass, `.smd` among the extensions a player
  is not offered while the core is in development; the anchors over the corpus took 3 min 26 s with the reference's
  runs cached.

## 12. Stage 3 closed (2026-10-05)

**The oracle** (the plan's §6, row 3), as met:

- **The BCD verifier, the opcode sizes and the illegal-instruction test** pass on the console (§9.3), and still did
  at each later step.
- **The memory test**: twelve of its thirteen rows match the values it prints as the hardware's (§9.2). The
  thirteenth, the first `$A11100` row, reads "busy" twice where the hardware reads "busy" then "granted"; the test's
  BUSREQ and RESET sequence is unknown without its source, and its own screen can now be read, so it goes to stage 4.
- **The RAM against Genesis Plus GX at anchors over the corpus** (§11.4): of 944 games, at frame 120 a median of
  65,518 equal bytes of 65,536 and 888 at 65,000 or more; at frame 600 a median of 65,491 and 839. The differences
  that remain are, as far as they have been examined, the VDP's and the sound chips' timing (stages 4 and 5) and the
  68000's refresh (D-2).

**Carried forward:**

- **D-2, the main RAM's refresh**: two clocks lost every 128 on the cartridge measured on the board, the RAM's pattern
  not yet separated; to be settled beside the VDP's DMA and refresh slots at stage 4 if those measurements need it.
- **Seven EEPROM boards** that Eke's document does not list: Putter Golf, and Accolade's Barkley Shut Up and Jam 2,
  Brett Hull Hockey '95, Jack Nicklaus' Power Challenge Golf, Pelé!, Pelé's World Tournament Soccer and Unnecessary
  Roughness '95. They run without their EEPROM until a document wires them.
- **Barver Battle Saga's protection device** at `$400004`/`$400006`, which no document in the corpus describes.
- **The memory test's first `$A11100` row** (above).
- **The window's alignment**: its waits are the measured means, not their dependence on where the Z80's access meets
  the 68000's bus cycle (§11.1).
- **Not built at stage 3**, though in the plan's §1.2: the multitaps (Team Player, EA 4-Way Play) and the J-Cart's
  extra ports, for stage 6's controller settings; the light gun that would drive the external interrupt.

## 13. Stage 4, step 1: the VDP's ports, FIFO and DMA on the slot schedule, the HV counter, the interrupts (2026-10-05)

### 13.1 The line, from the documents

A line now begins where the V counter is incremented, and everything in it is placed in master clocks from there
(`vdp.rs`'s `Timing`, built once per width):

- **The H counter** (Nemesis's tables, SpritesMind topic 1291, from his logic analyser): H32 counts `$000`-`$127`
  then `$1D2`-`$1FF`, every pixel ten master clocks; H40 counts `$000`-`$16C` then `$1C9`-`$1FF`, eight master clocks
  a pixel but in hsync, where the serial clock runs at MCLK/5 save for three pixels at MCLK/4, and `$1ED` at MCLK/5.
  Both sum to 3,420. The V counter increments at `$10A` (H32) and `$14A` (H40), which is where a line starts. The HV
  counter port gives the 9-bit counter's top eight bits, MacDonald's 8-bit table (`m5hvc.txt`) in agreement.
- **The access slots** (Nemesis's VRAM timing charts, H32 and H40, in the documentation folder): a slot is four serial
  clock ticks, slot 1 begins at H `$1E9` in both widths, which places hsync where the charts and the H counter tables
  agree. On an active line the external slots are H40's 15, 23, 31 and every 32 after to 159, then 174, 175 and 199
  (18), and H32's to 127, then 142, 143, 157 and 171 (16); refresh takes 39 and every 32 after (5 and 4). On a blank
  line every slot but refresh is external: 205 and 167, the counts of Sega's DMA table. A line follows the active
  pattern when the display is on and it is drawn, or it is the frame's last, whose blanking flag clears for the first
  line's sprites (Eke and Nemesis, topics 851 and 1291); the two slots at the change of render line within a line are
  given to the line they fall in (argued).
- **The interrupts and flags**: the line interrupt at the line's start, as before; the vertical interrupt and the F
  flag at H `$001` of line 224 (Nemesis: H `$000` to `$001`), where MacDonald had "roughly H counter cycle 08h"; the
  vertical blanking flag at H `$150` (H40) and `$10E` (H32) of lines 224 and the last (MacDonald's `m5hvc.txt`), forced
  on while the display is off; the horizontal blanking flag set at `$166` and `$126` and cleared at `$00B` and `$00A`
  (Nemesis).
- **The V counter**: NTSC's jump from `$EA` to `$1E5` (MacDonald §5). PAL's jumps, `$102` to `$1CA` in V28 and `$10A`
  to `$1D2` in V30, are argued from the line count: no document in the corpus gives them, and the board can measure
  them.

### 13.2 The FIFO, the read buffer and DMA, from VDPFIFOTesting

VDPFIFOTesting's sources (Nemesis, the test program and not an emulator) give each test's expectation as measured on
consoles; the rules below are the ones its tests demonstrate, each traced to the test that failed without it.

- **The FIFO is a ring of four entries** that keep their data after they are written out. A 68000 write takes the next
  entry with the code and address it was made under; the 68000 waits while all four are pending. A slot writes the
  oldest pending entry: a VRAM word takes two slots, a CRAM or VSRAM word one. A write to a target that is not one
  still takes an entry and its slots.
- **A CRAM or VSRAM read** returns that memory's bits (`$0EEE`, `$07FF`) with the rest from the entry the next write
  will take (FIFO Buffer Size, Separate FIFO Read/Write Buffer). VSRAM past its 40 words reads its first (argued: the
  test shows the scroll value the VDP last fetched, which the renderer will supply). The 8-bit VRAM target (`01100`)
  reads one byte, bit 0 of the address inverted, under the next entry's high byte. Reads wait for the FIFO to empty
  and are made at a slot; the 68000 waits for the read buffer.
- **A 68000 transfer** reads the bus once a bus cycle while the FIFO has room, so the FIFO fills between slots, and
  its source counts within its 128 KiB (Source Address Wrapping); the 68000 is held until the last word is read. The
  length and source registers count as it goes.
- **A fill** waits for its data word, which is written as an ordinary entry; then each slot writes one byte with bit 0
  of the address inverted (MacDonald's pseudocode, once the address has passed the first word). VRAM takes the data's
  high byte, CRAM and VSRAM the next entry's word (DMA Fill to CRAM, to VSRAM). A word written during a fill is
  written where the fill is, and becomes its data. A fill advances the source registers as a copy does (DMA Fill
  Source Reg Update).
- **A copy** reads a byte and writes it, a slot each, both addresses with bit 0 inverted (the DMA Copy tests' odd and
  even addresses and their overlapping copies).
- **Starting and the busy flag**: a second command word with CD5 set and DMA enabled starts a DMA, and nothing else
  does: CD5 left set while DMA was off does not start one when a first word follows (DMA Busy Flag DMA Toggle Fill),
  and disabling DMA does not stop a fill already set up. The busy flag is set from the command until the DMA ends, a
  fill's wait for its data included.
- **Register writes**: in mode 4 only registers 0-10 take a write (Register Write Mode4 Mask); any register write
  clears the two code bits a command's first word sets, the rest of the code and the address kept (Register Writes and
  Code Reg).
- **The HV latch**: setting register 0's bit 1 holds the HV counter at its value then until the bit is cleared (HV
  Counter Latch); TH takes it after that, as stage 3 built.

### 13.3 Measured (2026-10-05)

- **VDPFIFOTesting**, A pressed at frame 60 to run it through every page, its records read from RAM by
  `fifo_records.rs` (the reader the plan's §3.3 owed) and counted as the program counts them: **121 of 122 pass**,
  from 25 before this step. The one left is FIFO Wait States' tenth part, which writes three words and then sets up a
  transfer without writing register 23, so that the DMA it starts is whatever the earlier tests left there: the console
  sees the FIFO full at some point after it and Nephrite does not. Pinned by `programs.rs`'s
  `vdpfifotesting_passes_all_but_one`.
- **The cost**: 600 frames of Sonic the Hedgehog in 0.252 s on the desktop, of Sonic 2 in 0.247 s, Thunder Force IV
  0.218 s and Streets of Rage 2 0.223 s, about 0.4 ms a frame, from 0.2-0.3 before the slots, against the plan's
  1.5 ms.
- **The crate's tests**: 36 pass, the slot counts and counter ranges of both widths, a fill, a CRAM read through the
  FIFO and the state's layout (version 3: the FIFO, the read buffer, the DMA and the VDP's clocks) among them.
- **The RAM against Genesis Plus GX at anchors over the corpus** (944 games), stage 3's last report against this
  step's: at frame 120 a median of 65,523 equal bytes (from 65,518), 903 games at 65,000 or more (from 888), 3 below
  60,000 (from 6); at frame 600 a median of 65,506 (from 65,491), a 10th percentile of 65,214 (from 64,849), 872 at
  65,000 or more (from 839), 18 below 60,000 (from 33); 503 games rose at frame 600 and 216 fell, the largest falls
  X-Men 2 (64,925 to 61,468) and Pete Sampras Tennis 96 (64,633 to 61,552), not yet examined.
- **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the Hedgehog, Phantasy Star IV and
  VDPFIFOTesting. WiseMan's Nephrite, runner, registration and discovery tests, 31, pass.

### 13.4 What the next steps owe

- **The logic-analyser ROMs** of Nemesis (status register and HV counter sampled at known points, sent out of pad port
  2 by a nibble handshake) need the pad-port receiver; they are the oracle for §13.1's placements, which are argued
  from the documents until then.
- **PAL's V counter**, on the board or by those ROMs.
- **FIFO Wait States' tenth part** (§13.3).
- **The planes, the window, scrolling and sprites** (the next step), which also give VSRAM's latch its source.

## 14. Stage 4, step 2: planes, the window, sprites, priority and shadow/highlight; the picture (2026-10-05)

### 14.1 What it built

`render.rs` draws mode 5 a line at a time into the frame the v1 video path hands out, 256 or 320 pixels wide by the
active lines (224, or 240 in PAL's V30), from MacDonald's §12-§17 and the findings below.

- **When a line is drawn.** At the end of its line, after the line's writes have landed, which is a scanline
  renderer's loss: a change made while the line is being fetched takes effect for the whole line. The exception is the
  scroll: VSRAM and the line's horizontal scroll entries are taken as the line begins, since the fetches read them
  before a line interrupt's handler can write them. OutRun shows it: its line interrupt writes VSRAM every line, and
  with the values taken at the line's end its sky showed the road's pattern; taken at the start, its title is drawn as
  the reference draws it (read by eye; the frames compared are a few apart). The slot-stamped renderer of the plan's §5.2
  is a later step.
- **A frame ends where its last line does.** A transfer from the 68000's bus holds the 68000 from the write that
  starts it, and the hold now yields at the frame's end and goes on before the next instruction, where before a long
  transfer could carry the frame's end into the middle of the next frame. Phantasy Star IV found it: its intro's
  transfer spanned the frame's end, so the picture of a run that loaded a state differed from the run that did not
  (the kit's C7 and C8). The resumption is also at the transfer's last bus read, not the next slot, which is what
  FIFO Wait States' tenth part needed (§14.3). A line narrower than the frame leaves black, not an older frame, and
  the open-bus word the 68000 last fetched is now in the state.
- **Planes A and B**: the name tables and size of register 16 (the prohibited setting read as 32 cells, the pair held
  to 4,096 cells, and a width of `10` showing the first row on every line, MacDonald §17); horizontal scrolling full,
  per cell or per line from the table of register 13 (mode `01` as MacDonald's pseudocode has it, the first eight
  lines'); vertical scrolling full or per 2-cell column from VSRAM, the column the one the VDP fetched the pixel in
  given the fine horizontal scroll, the partial column at the left reading column 0 (argued).
- **The window** replaces plane A in its vertical range (whole lines) or else its horizontal range, unscrolled, 32 or
  64 cells wide. MacDonald's window bug (the column after a left window fetched from the next column when the scroll's
  low bits are set) is not modelled.
- **Sprites**: the link list from entry 0, read from a cache of each entry's first four bytes that follows writes into
  the table and not register 5 (Nemesis, topic 1291; Castlevania: Bloodlines relies on it); at most 64 or 80 entries
  a frame, a link beyond them ending the list, 16 or 20 sprites and 256 or 320 dots a line, a dot overflow cutting a
  sprite partway; masking per Nemesis's measurements (topic 541): a sprite at X 0 stops the pixels of every later
  sprite on its line once a sprite not at X 0 has preceded it on the line, or when the line before ended in a dot
  overflow; a mask counts its dots and the parsing goes on behind it. The overflow and collision flags of the status
  register are set by this pass and cleared by a status read; the pass runs on every line even when a frame is not
  drawn, since its flags and the dot overflow it carries to the next line are the VDP's state.
- **Priority**: high sprites, high A (or window), high B, low sprites, low A, low B, the backdrop (register 7); the
  first sprite pixel that is opaque wins among sprites, and a second opaque pixel under it sets the collision flag.
  Register 0's bit 5 blanks the left eight pixels to the backdrop.
- **Shadow/highlight** (register 12, bit 3), as plutiedev states it (D-3): a pixel is shadowed unless plane A, plane
  B or an opaque sprite pixel there has priority; palette 3's colour 14 on top brightens what lies under it and colour
  15 darkens it, both drawing that pixel instead, an operator counting as transparent for priority; brighter and
  darker give normal, and a sprite's colour 14 is never shadowed.
- **Colour**: TmEE's measured output levels (topic 2188), one 15-step ladder per channel: shadow takes steps 0-7,
  normal the even steps, highlight steps 7-14 (0, 29, 52, 70, 87, 101, 116, 130, 144, 158, 172, 187, 206, 228, 255).
  Register 0's bit 2, the eight-colour palette mode, is not modelled.
- **VSRAM past its 40 words** now reads the vertical scroll the renderer last fetched: plane B's at the end of a drawn
  line, or plane B's first entry on a line not drawn, which keeps VDPFIFOTesting at 121 of 122.
- **The frame's size** follows the VDP's width and height, which the v1 frame information reports; before a game sets
  register 12 the VDP is in H32, so a synthetic cartridge's first frame is 256 pixels wide. The state gains the sprite
  cache, the sprite flags and the latch (version 4).
- **Tools**: `examples/dump.rs` writes the picture as RGBA beside the memories and takes `NEPHRITE_PRESS` for the pad;
  `examples/regs.rs` prints the VDP's registers, scroll values and the processors' program counters at a frame.

### 14.2 Not yet

Interlace and its double resolution, mode 4, the CRAM dots, mid-line changes below the line (the slot-stamped
renderer), the display enable taking effect at the pixel, the sprite list parsed on the line before it is shown, and
the eight-colour palette mode: the steps after this one.

### 14.3 Measured (2026-10-05)

- **Nemesis's sprite masking and overflow test**, both widths (C switches them; the program reads its "Start" with TH
  an input, which a pad answers with C): all nine verdicts green, as on his console's photographs of H32 and H40.
  Pinned by `programs.rs`'s `the_sprite_masking_test_passes_in_both_widths`.
- **The 240p suite** (1.32) against Genesis Plus GX, the same presses given to both (the reference's A is its
  RetroPad's Y), the picture compared up to a one-to-one colour map: the main menu and the ten screens of "Color &
  Black Levels" (PLUGE's help, the colour bars, EBU, SMPTE, referenced bars, colour bleed, grey ramp, white and RGB,
  100 IRE, sharpness) match exactly but for PLUGE's help, whose prompt names Start where the reference's six-button
  pad has it name Z; the four "Geometry" entries' display-mode dialogs match.
- **The shadow/highlight programs** of Genesis Plus GX's `md_test` (D-3): `SHLTEST`, `SHLTEST2` and `stetest` match
  exactly; `STETEST2` matches but for pixels the measured ladder puts on one level.
- **The corpus at anchors**, the picture now beside the RAM (`NephriteTests`' report has the pixels off the colour
  map at frames 120 and 600), 944 games: at frame 120, 717 pictures match Genesis Plus GX's exactly up to the map and
  862 are within 1,000 pixels; at frame 600, 626 and 728, the 90th percentile 8,129 pixels of 71,680. The RAM is as
  step 1 left it (frame 600 median 65,506 equal bytes, 18 below 60,000); the transfer's hold of §14.1 moved 177
  games by a few bytes either way at frame 600 (84 up, 93 down).
- **The games that fell in step 1**: X-Men 2 runs about three frames ahead of the reference through its intro (its
  RAM at the reference's frame 140 matches Nephrite's 137 in 65,506 bytes) and from frame 150 its state, a buffer
  being filled and its falling snow, no longer matches at any offset; Pete Sampras Tennis 96 matches to frame 520 and
  then shows another player's card in its attract sequence. Both pictures are drawn as their RAM says; neither is the
  renderer's. Where the two machines part is the 68000's and the sound processor's timing, D-2's class, not yet
  traced to an instruction.
- **Pictures examined among the largest differences**: International Rugby matches exactly when shifted six pixels
  each way (its intro slides everything, sprites included, and the two machines are at different points of it);
  16 Tiles Mahjong, Ecco, Raiden Trad and Desert Strike differ by animation phase; Super Kick Off and Rampart stay
  black in Nephrite while their RAM parts from the reference's early (Super Kick Off at frame 150, where its sound
  routine stores a request the reference does not make), so not the renderer; OutRun is drawn with its lower part
  black in its lower part: its sky showed the road's pattern until the scroll latch of §14.1, and its title is right
  after it, but at frame 600 Nephrite has already turned the display off below line 73 for the next screen, which the
  reference reaches a few frames later.
- **The cost**: 600 frames of Sonic the Hedgehog in 0.63 s on the desktop, Sonic 2 0.58, Thunder Force IV 0.64,
  Streets of Rage 2 0.52: 0.87-1.07 ms a frame with the picture (0.4 without), against the plan's 1.5 ms.
- **VDPFIFOTesting: 122 of 122**, the tenth part of FIFO Wait States now among them (§14.1: the 68000 resumes at a
  transfer's last read, when the FIFO is still full). Pinned by `vdpfifotesting_passes_every_test`.
- **The crate's tests**: 37 pass, the state's layout re-pinned (version 4: the sprite cache, the sprite flags, the
  VSRAM latch, the line's scroll values and the open-bus word). The conformance kit, `--frames 600`, passes C1-C15 on
  the 240p suite, Sonic the Hedgehog, Phantasy Star IV, OutRun, Castlevania: Bloodlines and the sprite masking test.

## 15. Stage 4, step 3: interlace, PAL and V30, mode 4, the eight-colour mode, CRAM dots, the window bug (2026-10-05)

### 15.1 What it built

- **Interlace.** Register 12's interlace bits are taken at the start of vertical blanking, where the field changes;
  the status register's bit 4 is set in the odd field. Fields alternate 262 and 263 lines (PAL 313 and 312), and the
  V counter's jumps and its port formats in both modes are the board's (D-4). Double resolution (mode 2) doubles the
  plane's rows to 16 a cell, its vertical scroll to 11 bits and the sprites' Y to 10 bits (from 256), with a pattern
  64 bytes, name n at n x 64 (D-8), and a sprite's cells stepping by one name. The picture is 448 lines, both rows
  of a line drawn each field (§15.2); the sprite pass's flags and dot-overflow carry come from the field's own row.
- **PAL and V30.** PAL's V counter jumps as the board showed, `$102` to `$1CA` in V28 and `$10A` to `$1D2` in V30,
  which settles step 1's argued values; V30 shows 240 lines on either board, and on an NTSC board the counter runs
  to `$1FF`, a frame of 512 lines (D-4). A frame whose length falls under its current line, V30 cleared late in such
  a frame, now ends at once: 777 Casino boots with V30 set on its NTSC board for two frames, and the frame's end,
  tested for equality, was never reached (a sweep of the corpus, 600 frames a game, found it hung).
- **Mode 4**, the Master System's (register 1, bit 2 clear): MacDonald's SMS VDP document's name table, planar
  patterns, scroll locks, left-column mask and eight sprites a line from a 64-entry table, 192 lines with the frame
  interrupt at line `$C0` and a status read clearing the pending interrupts (MacDonald's `vdpint.txt`), read through
  mode 4's map of VRAM; a write lands there as a word, the address steps by
  one whatever register 15 holds, CRAM takes a Master System colour byte an entry, and colours are shown at the
  board's four levels a channel (D-7).
- **The TMS9918's modes** (register 1's bit 2 and register 0's bit 2 both clear, as at power-on) show black, 256 by
  224, as MacDonald's SMS VDP document says the Genesis does and the board's first frames show; mode 4 needs
  register 0's bit 2 set.
- **The eight-colour mode** (register 0, bit 2 clear): each component's lowest bit, at the level of step 1 (D-8).
- **CRAM dots**: a CRAM write while a mode 5 line is shown draws its colour on the pixel at H minus `$18`; one after
  the V counter's step falls on the last pixels of the line before, which the next line event patches; the first
  blank line keeps the active line's slots until H `$14E` (H40) or `$10E` (H32), so that a full FIFO drains where the
  board drains it (D-6).
- **Sprites are parsed on the line before** they show, into a 320-pixel line buffer (colour and priority) that the
  next line composes; the pass runs on every line, the last line of the frame parsing line 0's. From the MegaDrive
  Wiki's VDP article ("tile data for sprites is also fetched during the blanking interval of the line before") and
  Eke's topic 1291; no program in the corpus separates it from parsing on the line itself.
- **MacDonald's window bug**: with the window on the left, not covering the line, and plane A's fine scroll `s` not
  zero, the `s` pixels after the window take plane A's data from 16 pixels on; and registers 17 and 18, like the
  scroll, are taken as the line begins (D-5).
- **Register 16's prohibited width** `10` shows the name table's first row on every line, the pattern's row still
  following the line (MacDonald §17): step 2 had forced the pattern row to 0 as well, which drew stripes where the
  window tests show text.
- **The state** (version 5) gains the field (interlace and odd), the sprite line buffer, the line's window registers
  and the CRAM dots a line has gathered and not drawn. Restoring a state set the VDP's current line from the
  machine's line before restoring that line, so a state loaded mid-run drew its first line's dots from the old line
  (the kit's C7 and C8 on the CRAM dots program found it); the order is now right.

### 15.2 Argued

- **Both rows each field.** On the board a double-resolution field is 224 lines of its own rows; Nephrite draws both
  rows every field from the field's state. The other field's rows are not the VDP's state, and the kit's C7 and C8
  hold the picture to follow from the state; keeping them would add a 448-line picture to every state. A program
  that changes the picture between fields shows both rows as the field being drawn has them (D-8).
- **Mode 4's left column under a fine scroll**: the board shows garbage there, as MacDonald's `newreg.txt` reports;
  Nephrite draws the wrapped tile. **Mode 4's levels** are the board's DAC model's; the corpus has no measurement of
  a console's mode 4 output. **H40 in mode 4** (320 by 192, its left eight columns garbage) is not modelled.
- **Which interlaced field is the longer** (the odd one): the counter program did not read the status register.
- **Sprites parsed on the line before**, from the documents above.

### 15.3 The board's pictures

`tb_md.cpp` gains `TB_PICTURE=path:from:to`, which writes, for each MCLK2 cycle in the window, the board's video
pins: the two display enables, the syncs, the field, the pixel clock and mode 5, then red, green and blue.
`mdboard.py`'s `frames()` cuts a capture into frames by the display enables and samples each pixel mid-way on the
pixel clock; `picture <name>` builds one of its programs (a register list, blocks written through the data port and
a tail), runs the board for a few frames and writes the frames as PNGs beside the program's image, which Nephrite
then runs. The board's DAC levels for mode 5 (49, 84, 114, 142, 172, 206, 255 for steps 2-14 of the even ladder)
are within three of TmEE's measured ladder (52, 87, 116, 144, 172, 206, 255), so pictures are compared on that map
exactly, or up to a colour map. A frame of the board costs about 8 seconds on the desktop. The programs:
`palette64`, `palette8`, `cram-dots`, `cram-dots-h32`, `mode4`, `mode4-bytes`, `mode4-colours-0` and `-32`,
`interlace1`, `interlace2`, `field-colour`, `pal-v30` (the board in PAL) and `ntsc-v30`; and `mode4-ports`, which
reads mode 4's writes back through mode 5. `pictures.rs` builds the same programs and holds Nephrite to the board's
results. No RTL source was opened; the video pins' names came from the generated model's port list (D-4).

### 15.4 Measured (2026-10-05)

- **Against the board**, every program of §15.3: `palette64`, `palette8`, `interlace1`, `interlace2` (each board
  field against Nephrite's rows of its parity), `pal-v30`, `ntsc-v30` and both mode 4 colour pictures match exactly;
  the mode 4 pictures written in mode 5 and in mode 4 match but for the five garbage pixels at the left (§15.2); the
  CRAM dots match on every line but seven of H40's 224 (D-2's class) and one dot on each width's last line; MacDonald's
  window bug program, its scroll stepped every frame, matches all 16 fine scrolls exactly; the Exodus CRAM flicker
  program's dots match on two frames of three exactly and on all but one line of the third. Genesis Plus GX draws no
  CRAM dots, so it shows that program black.
- **Against Genesis Plus GX at frame 300**, up to a colour map: `wtest`, `wtest_i1`, `wbug`, MacDonald's window bug,
  Fonzie's window test, `vctest`, `VDPTEST`, `TEST1536`, `MD1536`, `SHLTEST`, `SHLTEST2` and `stetest` match
  exactly; `wtest_i2` matches the reference's field exactly on Nephrite's rows of that field's parity, at frames 300
  and 301 alike; `STETEST2` is as step 2 left it; Tristan Seifert's register test differs by 362 pixels: 336 as at
  step 2, in its colour-cycled bars, and a line of CRAM dots from its palette cycling. The PAL colour programs `512PAL`, `960PAL` and `PAL512` change
  CRAM within the line and differ from the reference by 7,936, 5,472 and 5,149 pixels (7,808, 5,120 and 2,880 at step
  2, before the dots); the board's `512PAL` looks like neither (§15.5).
- **The crate's tests**: 46 pass, among them `pictures.rs`'s seven, the window bug's and the V counter's; the state's
  layout re-pinned (version 5). VDPFIFOTesting stays at 122 of 122.
- **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the Hedgehog, Sonic the Hedgehog 2,
  Phantasy Star IV, OutRun, Castlevania: Bloodlines, the sprite masking test, `wtest_i2`, MacDonald's window bug
  program, and the board programs `mode4-colours-0`, `mode4-bytes`, `cram-dots`, `cram-dots-h32`, `field-colour`,
  `pal-v30` and `ntsc-v30`.
- **The corpus at anchors** (944 games, against Genesis Plus GX): at frame 120, 716 pictures match up to the colour
  map (717 at step 2) and 862 are within 1,000 pixels; at frame 600, 626 and 727 (626 and 728), the 90th percentile
  8,129 pixels as before; the RAM's median at frame 600 is 65,506 equal bytes with 18 games below 60,000, as at
  step 2. 74 games moved by a few bytes or pixels: the dots, which the reference does not draw (Hurricanes, Jammit
  and Steel Talons gain a few pixels at frame 120), the slots of §15.1 shifting a 68000's timing by a little (VR
  Troopers is at another point of its logo's fade at frame 600), and 777 Casino's two frames of 512 lines. A sweep of
  every game through 600 frames found the one hang of §15.1 and no panic. WiseMan's Nephrite, runner, discovery
  and registration tests pass (31); the TMS9918 rule of §15.1 is what two of them needed, whose synthetic cartridge
  leaves both mode bits clear and expects 256 by 224.
- **The cost**, the machine busy with other work (load average about 6), best of five runs of 600 frames beside the
  step 2 build in the same minutes: Sonic the Hedgehog 0.66 s against 0.62, Thunder Force IV 0.68 against 0.65,
  about 1.1 ms a frame against the plan's 1.5. The plane's per-pixel wrap, a division once the cell's rows became a
  variable, is a mask now (the plane sizes are powers of two); before that change the step cost 15 % more.

### 15.5 Not yet, and the next step

- **The slot-stamped renderer** of the plan's §5.2 was not reached: a register, VRAM or CRAM change during a line
  still takes effect for the whole line (the scroll and the window's registers excepted, which are taken as the line
  begins). It is the next step. Its first oracle is the PAL colour programs: `512PAL` on the board in PAL (frames
  3-5, captured with `TB_PICTURE` and `TB_PAL`) shows bands of colour with dense columns of dots where Nephrite and
  Genesis Plus GX each show a grid of 512 colours, unlike each other; the board's picture is the one to meet. The
  display enable taking effect at the pixel belongs to it too.
- **D-2**, the main RAM's refresh, is still open; the CRAM dots program shows its trace as seven lines whose dots
  the 68000 lands a slot apart from the board's.
- **Mode 4's** left-column garbage and its H40 mode (§15.2); and the cartridge's M3 pin, which puts the machine in
  the Master System's mode for a Master System cartridge through a converter, not modelled.


## 16. Stage 4, step 4: the slot-stamped renderer (2026-10-05)

### 16.1 What it built

- **A line drawn in spans.** The VDP holds the line being shown, the picture's rows and its sprite pixels, taken as
  the line opens. Before any write that changes what mode 5's picture reads (VRAM, CRAM, VSRAM, a register), the line
  under the beam is drawn up to the beam's pixel from the state before the write; at the V counter's step the line is
  drawn to the step and finished when its last pixels, after the step, have passed. A line with no write in it is
  drawn once, as before. The beam's pixel is H minus `$18`; a line's last 14 pixels in H40 (and in H32) come after
  the V counter's step, and the next line's scroll and window values, taken at the step, are kept apart from the
  open line's. The CRAM dots of step 3 are now this mechanism's: a CRAM write draws its colour on the beam's pixel
  and the new colour from there. The frame lives in the VDP. Mode 4 is still drawn a line at a time.
- **Where each write shows** (D-9): CRAM at the beam's pixel; VSRAM by 2-cell column, a column's scroll read 27
  pixels before it is shown, so that a write reaches the columns read after it, on the present line and the next;
  VRAM with the 8 pixels fetched before the write drawn from the old data; a register two and a half pixels after
  the 68000 writes it (20 master clocks in H40, 25 in H32); the display bit's blanking 12 pixels later than that, and
  its return on the first 16-pixel boundary 24 pixels on.
- **Register 1's bit 7**, the 128 KiB mode, writes VRAM as a 64 KiB board does: one byte, the word's low one, at the
  interleaved address (D-10).
- **The palette** is kept between the writes that change it (CRAM, register 0) and made again at each frame's
  start, so that a line drawn in many spans does not make it again for each.
- **The state** (version 6) keeps the open line's number, how far it was drawn, its sprite pixels and its latched
  scroll and window values, not its pixels, which are the picture's: whether frames were drawn must not change a
  state (the kit's C8 caught the first version, which kept them), and a loaded state draws the line so far again.
- **The bench**: `tb_md.cpp`'s `TB_PA` holds port 1's pins (a button for the whole run); `mdboard.py`'s pictures
  keep the 68000's RAM writes beside the frames, and gained the transfer programs, the register programs, the
  transfer-start sweep and `vram128` (§16.3, D-9 to D-11).

### 16.2 Argued, and measured in part

- VRAM's fetch lead of 8 pixels is weakly bracketed (4 to 16) (*24 pixels, bracketed 20 to 32, since §17's commit
  settled D-11: D-9's correction of 2026-10-07*); the display bit's return fits 0.56 of the board's
  distribution, whose returns fall mostly on 16-pixel boundaries and partly on 8-pixel ones.
- Writes after a line's V counter step that change VRAM or a register reach that line's last 14 pixels from the state
  at the next change, not at their own pixel, except CRAM and the dot, which are exact.
- Mode 4 and the TMS9918 modes are drawn a line at a time.

### 16.3 Measured (2026-10-05)

- **The board's transfer pictures** (D-9), with D-11's start delay set for the comparison: `cram-dma` and
  `cram-dma-one` match on every row of two frames, `vsram-dma` on every row, `pattern-dma` on all but 4 rows (9
  pixels). Without the delay (the shipped setting) every write lands one slot earlier: `cram-dma` differs by 1,736
  pixels on 82 rows, `vsram-dma` by 208 on 13.
- **Registers** (D-9): the backdrop's change 9 to 11 pixels after the HV read, as the board's, in H40 and H32; the
  display's blanking 21 to 23 after.
- **`512PAL`** on the board in PAL: 10,544 pixels from the board's picture with the 128 KiB writes (65,348
  without); with register 1's bit 7 cleared, 2,028 to 2,208, the 68000's timing. `960PAL` resembles the board's
  picture in its structure (the same mode); `PAL512`'s transfers come from the cartridge, which the bench cannot
  serve (D-11).
- **The HV logic-analyser ROM** (Nemesis, H40 V28) on the board, its samples read from the RAM log: the V counter
  steps at the same H; the board's port reads vary in duration where Nephrite's do not (D-11).
- **Against Genesis Plus GX at frame 300**: unchanged from step 3 (§15.4) but for `512PAL` and `960PAL` (75,492 and
  60,368 pixels; the reference ignores the 128 KiB bit) and `PAL512` (7,387: mid-line CRAM, which it draws a line at
  a time).
- **The crate's tests**: 48 pass, the state's layout re-pinned (version 6). VDPFIFOTesting 122 of 122.
- **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the Hedgehog and its sequel,
  Phantasy Star IV, OutRun, Castlevania: Bloodlines, 777 Casino, `512PAL`, `wtest_i2`, both of TiTAN's Overdrive
  demos and the board programs `cram-dma`, `pattern-dma`, `vsram-dma`, `reg-backdrop`, `reg-display`, `cram-dots`,
  `mode4-bytes` and `field-colour`.
- **The corpus at anchors** (944 games, against Genesis Plus GX): no game's picture or RAM at frames 120 and 600
  differs from step 3's (716 and 626 pictures matching up to the colour map); the reference draws mid-line changes a
  line at a time, so the anchors neither gain nor lose by the spans. WiseMan's Nephrite, runner, discovery and
  registration tests pass (31).
- **The cost**, best of five runs of 600 frames beside step 3's build in the same minute: Sonic the Hedgehog 0.62 s
  against 0.62, Thunder Force IV 0.71 against 0.65 (its mid-line writes; 1.03 before the palette was kept between
  changes).

### 16.4 Stage 4's oracle

Of the plan's oracle for stage 4: VDPFIFOTesting passes 122 of 122; the pictures against the references and the
board, the sprite-masking photographs and the skip-versus-draw state test (the kit's C8) are met. The logic-analyser
ROMs are not: the HV ROM, run on the board in place of the pad-port receiver, shows the port's read time varying on
the board and not in Nephrite, and the status-register ROMs are not yet run. With D-11 open, stage 4 stays open for
one more step: the bench's cartridge serving the VDP's reads, the transfer's start and busy flag (D-11), the port's
read time and the status-register ROMs.

## 17. Stage 4, the dispute step, and stage 4 closed (2026-10-05)

### 17.1 What it settled

- **The bench's cartridge answers the VDP.** During a transfer from the cartridge the board raises `cart_dma` and
  `vdp_dma` with `cart_cs` high; `tb_md.cpp` now serves the cartridge's range then too, which the first step's
  transfers from the cartridge, `PAL512` and VDPFIFOTesting's own, lacked (D-11).
- **D-11, settled by one rule.** A word that reaches an empty FIFO, with nothing waiting to be written, is written at
  the first slot at least 176 master clocks on (`vdp::WRITE_START`), and a word behind others follows them; a
  transfer from the 68000's bus reads first 88 master clocks after its command (`genesis::DMA_START`) and then every
  20, the 68000 released at its last read; the busy flag is set only while the transfer reads. Each FIFO entry keeps
  the time from which it may be written; an entry the FIFO has given up at its slot waits in a short queue until
  then. `Hw::transfer_start_delay`, which
  step 4's tests set to 270 to hold the renderer to the board, is gone: the board's pictures and VDPFIFOTesting are
  met by the same settings. The words of a transfer made with the display off do not wait the start, as
  MD1536 on the board to its frame 300 shows.
- **D-12, the status register.** Vertical blanking sets at H `$14C` (H40) and `$10C` (H32), two pixels after the V
  counter's step, not at `m5hvc.txt`'s `$150` and `$10E`; the odd flag changes with the frame interrupt, at H `$001`
  of line 224, not as the line begins; horizontal blanking and F were already where the board has them. A port read
  takes four 68000 clocks on the board with no wait of its own.
- **The vertical scroll latched at the line's end** is taken whether or not the frame is drawn: the kit's C8 (a state
  must not depend on whether frames were drawn) failed on VDPFIFOTesting at step 4, the latch having been set only
  while a plane was drawn.
- **The state**, version 7, keeps each FIFO entry's ready time and the words waiting to be written.
- **The bench's programs**: `dma-start-rom-*` (the transfer sweep from the cartridge), `write-landing-*` (single
  writes, in both widths, the display on and off), `status-hv-h40`, `-h32`, `status-boot` and `fifo-wait-states`, the
  last four with their samples in the pictures' RAM logs.

### 17.2 Measured (2026-10-05)

- **The board's sweeps**: the transfer's first write, from RAM and from the cartridge, lands where the board's does
  at all twelve points; single writes with the display on at all sixteen, in H40 and H32; with the display off at five
  of eight in H40 and four of eight in H32, the rest 2 to 4 pixels from the board's (D-11).
- **The board's transfer pictures**, with the shipped settings: `cram-dma`, `cram-dma-one` and `vsram-dma` on every
  row, `pattern-dma` on 220 of 224.
- **`PAL512`** on the board in PAL, now that the cartridge answers: Nephrite's frames 3 to 5 are the board's on every
  pixel, up to a one-to-one map of colours. **`512PAL`**: 9,676 to 9,748 pixels from the board's (10,544 at step 4;
  the display's reads in the 128 KiB mode, D-10). **MD1536**: the board's at frames 3 to 6, and, run on the board to
  its frame 300, Nephrite's frame 300 on every pixel; with the write path's start applied to its display-off
  transfers, 259 pixels of CRAM dots differ.
- **The logic-analyser ROMs** (D-12): Nemesis's status-register ROMs in H40 and H32 and in interlace, and his HV
  ROMs, on the board with their samples from its RAM log: every flag's edge as the board's within one sample, after
  the vertical blanking and odd flag changes.
- **The memory test**, its screen now read at frame 600: all thirteen rows show the values it prints as the
  hardware's, the first `$A11100` row among them (`4F00 4F00` beside `(4F00 4F00)`). Stage 3's reading of that row,
  taken from RAM before the picture existed, is superseded.
- **Against Genesis Plus GX at frame 300**: as at step 4, but for `PAL512` (7,954 pixels: the reference draws
  mid-line CRAM a line at a time, and Nephrite now has the board's picture), `512PAL` (75,508) and `960PAL` (60,376).
- **The crate's tests**: 51 pass, among them `writes_land_where_the_board_lands_them` and
  `md1536_shows_the_boards_picture`; VDPFIFOTesting 122 of 122.
- **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the Hedgehog and its sequel,
  Phantasy Star IV, OutRun, Castlevania: Bloodlines, 777 Casino, `512PAL`, `wtest_i2`, VDPFIFOTesting, both of
  TiTAN's Overdrive demos, MD1536 and the board programs `cram-dma`, `pattern-dma`, `vsram-dma`,
  `reg-backdrop`, `cram-dots`, `mode4-bytes`, `write-landing-14`, `dma-start-6` and `fifo-wait-states`.
- **The corpus at anchors** (944 games, against Genesis Plus GX): pictures matching up to the colour map, 715 at
  frame 120 and 626 at frame 600 (716 and 626 at step 4); 14 pictures changed, all in games whose picture depends on
  when a write lands (Tyrants' Sega logo one step of its colour cycle apart; Batman Returns, Crystal's Pony Tale and
  X-perts by a few thousand pixels), which the reference does not time; the RAM's median equal bytes 65,522.5 and
  65,504.5 of 65,536 (65,522 and 65,506). WiseMan's Nephrite, runner, discovery and registration tests pass (31).
- **The cost**, best of five runs of 600 frames beside step 4's build (`cd939fc1`) in the same minute: Sonic the
  Hedgehog 0.62 s against 0.62, Thunder Force IV 0.73 against 0.71.

### 17.3 Stage 4 closed

**The oracle** (the plan's §6, row 4), as met:

- **VDPFIFOTesting** passes 122 of 122 on the shipped settings, which also meet the board's sweeps and pictures.
- **The logic-analyser ROMs**: run on Nuked-MD's board, its RAM log standing in for the pad-port receiver, and
  graded flag by flag against Nephrite's samples (D-12). The board is the console here, as it has been for every
  measurement since stage 3; the ROMs were not run on a console.
- **The pictures against the references** up to a colour map: 626 of the corpus's 944 at frame 600 against Genesis
  Plus GX, the test programs against it and against the board (§14.3, §15.4, §16.3, §17.2), where the board is the
  one to meet when the two differ.
- **The sprite-masking photographs**: Nemesis's test passes in both widths as on his console (§14.3).
- **The skip-versus-draw state test**, the kit's C8: passes on every image of §17.2.

**Carried forward:**

- **D-2, the main RAM's refresh**: the 68000's RAM writes on the board take 98 bench clocks in 24 of 278 where the
  rest take 56, and the cartridge loses two clocks every 128; Nephrite takes no refresh. As far as examined, it is
  why the 68000 runs a slot or a few ahead of the board in the CRAM dots program and in `512PAL` with the 128 KiB bit
  cleared; stage 5's sound timing may need it.
- **Mode 4's left column** under a fine scroll, which the board shows as garbage (D-7), and mode 4's H40 mode; mode 4
  and the TMS9918 modes are drawn a line at a time.
- **The display's reads in the 128 KiB mode** (D-10): Nephrite reads VRAM as in 64 KiB mode; `512PAL`'s remaining
  9,700 pixels are this.
- **Single writes with the display off** land 2 to 4 pixels from the board's at half the sweep's points, and why a
  transfer with the display off skips the write path's start is not known (D-11).
- **VRAM's fetch lead** (4 to 16 pixels) and **the display bit's return** (0.56 of the board's distribution), measured
  in part (§16.2).
- From stage 3, unchanged: the seven EEPROM boards no document wires, Barver Battle Saga's protection device and the
  Z80 window's alignment (§12).

## 18. Stage 5, step 1: the PSG, the YM2612's interface, the mix and the output (2026-10-05)

### 18.1 What it built

- **The PSG**, as Beryl's `beryl-sn76489` (`Beryl-HW/Beryl_SN76489.md`): crates shared between the Sega cores live
  in Beryl (decided 2026-10-05), and the Master System and the Game Gear carry the same chip. Nephrite gives it
  Sega's variant (16-bit noise register, bits 0 and 3 tapped) and its clock, the Z80's over 16 (240 master clocks a
  tick), and writes it through `$C00011`-`$C00017` from the 68000 and `$7F11`-`$7F17` from the Z80, each write at the
  master clock of the processor making it.
- **The YM2612's interface** (`ym2612.rs`): its four ports at the Z80's `$4000`-`$5FFF`, which the 68000 reaches
  through `$A04000` with the Z80's bus. Its rules, each from a document read as prose:
  - one address register of nine bits, the address and the part of the port it was written through, so that data
    through either data port goes to that part (Nemesis's test, SpritesMind topic 386);
  - the chip's own registers, `$21`-`$2F`, are part 0's only (Sauraen's reading of the die, the same topic);
  - every register is zero at power-on but the left and right bits of `$B4`-`$B6`, which are set (Nemesis and Snake,
    the same topic);
  - timer A counts once a sample, from its ten-bit value to 1,024; timer B once every sixteen samples, from its value
    to 256; loading either (bit 0 or 1 of `$27` rising) starts it from its value, an overflow reloads it and, with
    bit 2 or 3 set, raises its flag, which bit 4 or 5 written as 1 clears (plutiedev's register page, the MegaDrive
    Wiki and the YM2608's terms, as Nemesis's test confirmed them);
  - the busy flag reads set for 32 internal clocks, 192 of the 68000's, after a data write and not after an address
    write, and only at port 0 of the discrete YM2612 (Eke's tests; D-13);
  - channel 6's DAC is `$2A` with `$2C` bit 3 as a ninth bit below it, enabled by `$2B` bit 7, in channel 6's place on
    the sides its `$B6` names (plutiedev; Sauraen's reading of `$2C`).
  The FM operators are not built: until the next step every channel but the DAC puts out nothing.
- **The mix and the output.** Each chip is caught up when touched and at the frame's end, and each has its own step
  synthesiser (`EmuSen_NativeCores.md` §12.6), so that its changes come in time order; the two are summed into a
  `SampleQueue` and drained at 48 kHz through the v1 calls. A step of the YM2612's nine-bit output is 16 units, a PSG
  level a quarter of its 8,192: a PSG square at full volume is 0.251 of the DAC's whole swing (§18.3).
- **Accesses ahead of the frame.** The Z80 runs up to an instruction past the 68000's clock, so the sound keeps the
  latest time any access or frame's end brought it to, and takes an access from behind that time as made then, an
  error of under one Z80 instruction. The first WiseMan run of this step found the need: a Z80 write to the PSG after
  the frame's last 68000 clock left the PSG's synthesiser a sample ahead of the YM2612's, and summing the two indexed
  past the shorter (the core's panic aborted the test host). `sound.rs`'s
  `an_access_ahead_of_the_frames_end_moves_the_end_with_it` fails the same way without the rule.
- **The state**, version 8, keeps both chips, the PSG's clock and the sound's latest time, and both synthesisers'
  pending differences.
- **For the bench and the references**: `mdboard.py sound dac-square` (the DAC's whole swing as a square, written by
  the 68000 through `$A04000`) and `sound psg-tone` (tone 0 at full volume), and Nephrite's `examples/wav.rs`, which
  records its own output.

### 18.2 Argued

- ~~**The mix's levels**: from the references (§18.3), the four of which put the PSG's square at 0.214 to 0.282 of the
  DAC's swing; Nephrite's 0.251 is their median. No recording of a model 1 is in the corpus.~~ Measured on seven
  model 1 consoles' recordings at §28: 0.324.
- ~~**The analogue path**: none yet. The PSG's output is unipolar, as the chip's is, and no filter or coupling
  capacitor removes its offset; the model 1 filter is a later step of the stage.~~ Built at §28.
- **Timer B's divider** runs free, so that its first count after a load falls anywhere in sixteen samples; a timer's
  count falls on the sample's boundary, and a register write takes effect at the next sample.
- **Writes while busy are taken.** Sauraen reads the busy flag as a timer that blocks nothing; TmEE's report that DAC
  writes beyond about 26 kHz garble is not modelled.
- **The 68000's accesses to the YM2612** take its four clocks, as any access does (D-14).

### 18.3 Measured (2026-10-05)

- **Against the references**, `dac-square` and `psg-tone` at frame 180, one second from 1 s, the square measured as
  the median of its samples above the mean less the median below:

  | Core | DAC square | PSG tone | PSG over DAC | DAC square's frequency |
  |---|---|---|---|---|
  | Genesis Plus GX | 16,487 | 4,186 | 0.254 | about 530 Hz |
  | PicoDrive | 14,241 | 4,018 | 0.282 | 535.3 Hz |
  | BlastEm | 10,815 | 2,315 | 0.214 | 535.0 Hz |
  | ClownMDEmu | 8,264 | 2,044 | 0.247 | 543.7 Hz |
  | Nephrite | 8,160 | 2,048 | 0.251 | 543.6 Hz |

  The tone is 440.4 Hz on all five. The square's frequency is the 68000's loop timing: BlastEm and PicoDrive give its
  accesses to `$A04000` about 110 more of its clocks a half period than Nephrite and ClownMDEmu (D-14).
- **The crate's tests**: 60 pass, among them the YM2612's (the address register's part, the busy flag by port and
  model, the timers' periods and reset, the DAC's panning), the mix's (a PSG tone at its frequency, the DAC and the PSG
  adding, an access ahead of the frame's end) and two 68000 programs polling `$A04000` (the busy flag for 192 clocks
  at port 0 alone; timer A at `$3F0` overflowing 15 to 16 samples after its load); VDPFIFOTesting 122 of 122.
  `beryl-sn76489`'s 8 and the step synthesiser's 5 pass; Beryl's CPU crates pass from their new folder (68000 20, Z80
  15, SH-2 1).
- **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the Hedgehog and its sequel, Phantasy
  Star IV, OutRun, VDPFIFOTesting, TiTAN's Overdrive 2, `dac-square`, `psg-tone` and Nemesis's CSM test, the audio
  now part of C8's digests across a save and load (801 samples a frame on NTSC, 960 on PAL).
- **The cost**, best of five runs of 600 frames beside the stage's starting point (`94a87e39`) in the same minute:
  Sonic the Hedgehog 0.628 s against 0.622, Thunder Force IV 0.730 against 0.722, Phantasy Star IV 0.629 against 0.623.
- **The corpus at anchors** (944 games, against Genesis Plus GX), now that the YM2612's status answers: pictures
  matching up to the colour map, 718 at frame 120 and 630 at frame 600 (715 and 626 at stage 4's close); 24 pictures
  changed, 16 nearer the reference at frame 600 and 7 further. Rampart, Super Kick Off, Hit the Ice, Alien Storm and
  Slap Fight now match there, games that wait on the chip's timers or its busy flag. Devilish, which matched, is now
  8,019 pixels off: its intro's text is on screen at frame 600 where the reference still shows the screen before it,
  Nephrite running ahead through the sound driver's waits; to be looked at again with the FM operators built and D-13
  and D-14 settled. The RAM's median equal bytes, 65,523 and 65,506 of 65,536 (65,522.5 and 65,504.5). WiseMan's
  Nephrite, runner, discovery and registration tests pass (31).

### 18.4 Next

The FM operators: the phase generator (frequency number, block, detune and multiple), the envelope generator (its
rates, key scaling, sustain and release), the operator's output through the sine and exponent tables as the documents
describe them, the algorithms and feedback, and the LFO's amplitude and phase modulation. After them, in the plan's
order: SSG-EG, CSM and channel 3's special mode, the multiplexed output and its ladder effect, the YM3438 of model 2
as a setting, the model 1 filter; then the oracle (Nemesis's tests against the recording and the board's output pins
as a black box, MDFourier) and the stage's dispute step (D-13, D-14).

The YM2612 stays in Nephrite: of the Sega cores to come only the Genesis carries one (the Master System's FM unit is
the YM2413), so it is not a Beryl crate by the rule of 2026-10-05; `ym2612.rs` keeps the Genesis's clock out of the
chip's rules all the same.

## 19. Stage 5, step 2: the FM operators (2026-10-05)

### 19.1 What it built

`fm.rs`, the YM2612's FM unit a sample at a time, fed by `ym2612.rs`'s registers:

- **The phase generator** (Nemesis's account, topic 386, 2008; the YM2608 manual §2-4): the 11-bit frequency
  number shifted by the block into 17 bits (block 0 shifting down one and losing its low bit), the detune added
  there (its magnitude from the key code and the manual's Table 2-6, whose Hz values are whole steps of 0.053 Hz, one
  of the phase increment's at the manual's 8 MHz; a negative detune wrapping in 17 bits), the multiple applied (a
  half for 0) into 20 bits, the counter's top ten bits the operator's phase, and the counter cleared at key-on. The
  key code is the manual's (block and two bits of the frequency number).
- **The envelope generator** (Nemesis, 2008 and his 2010 corrections; the manual §2-5): a cycle every third sample
  advancing a global counter; each operator's rate, recomputed each cycle, twice its register plus the key code scaled
  by KS, 0 for a register of 0, at most 63, the release rate's register doubled plus one; an update on the cycles the
  rate's shift selects, its increment the next of the rate's eight from Nemesis's measured tables; the attack
  stepping `attenuation += (~attenuation × increment) >> 4` and the others adding the increment, to `$3FF`; the attack
  ending at 0, the decay at the sustain level (16 steps of `$20`, the last `$3FF`), one way; a rate of 62 or 63 setting
  the attenuation to 0 at key-on and stalling an attack already begun; keys taken every sample; TL added as `TL << 3`,
  to `$3FF`.
- **The operator** (Nemesis's and Sauraen's prose): the phase plus the modulation, a quarter-wave log-sine table of
  256 entries in 4.8 fixed point with the phase's bits 8 and 9 mirroring and signing it, the attenuation shifted into
  4.8 and added, a 256-entry exponent table of eleven bits shifted by the sum's whole part, a 14-bit signed result.
  The tables are written out as constants so that every machine has the same bits; how their entries' last bits were
  fixed is D-16.
- **The algorithms**, evaluated as Sauraen read the die (2015-16): operators in the order 1, 3, 2, 4; operator 1
  modulated by the sum of its own last two outputs shifted right by 10 less the feedback (which gives the manual's
  Table 2-3, π/16 at 1 to 4π at 7); operator 3 by the stored outputs of 1 and 2; operator 2 by operator 1's newest;
  operator 4 by the stored 1 and 2 and operator 3's newest; a modulator's 14-bit output summed and shifted down one
  into the ten-bit phase. The carriers' outputs, cut to their top nine bits (Sauraen), are summed per channel and
  saturated to −256..255; channel 6's DAC replaces it when enabled.
- **The frequency registers**: `$A4`-`$A6` and `$AC`-`$AE` held in a latch per part until the low byte's write
  carries them in (the manual's order of writing; whether the latch is the channel's or the part's is D-15's kind of
  question, argued here); channel 3's own frequencies for S1-S3 in its special modes.
- **For comparison**: a channel trace (`Ym2612::trace`, off and outside the state), `examples/fmtrace.rs`, and the FM
  voices of `mdboard.py`'s `SOUNDS` (`fm-sine`, `fm-tl16`, `fm-mul3`, `fm-dt3`, `fm-dt7`, `fm-chain`, `fm-feedback5`,
  `fm-decay`, `fm-attack`).
- **The state**, version 9, adds every operator's phase, attenuation, envelope phase, key and last output, the
  channels' stored outputs and keys, the envelope's counter and divider, and the latches.
- **Not built in this step**: the LFO, SSG-EG and CSM, the test register, the ladder effect and the model 1 filter,
  and the write timing of D-15.

### 19.2 The sources, and one that was not clean

Read as documents: the YM2608 manual in its translated edition (`docs/yamaha/YM2608J_Translated.PDF`, §2); topic 386's
prose and, of its code blocks, only those that are its authors' tables and statements of the hardware's arithmetic:
Nemesis's attenuation weighting, his Tables 1 and 2 (counter shifts and increments), his statements of the update
cycle and of the attack and decay steps (page 8, and their 2010 corrections on page 28), his tables of the block
shift and the key code (page 12). (*The ruling this follows, an author's own table being a document and pasted
program code not, is stated in D-18 since 2026-10-07.*) A block quoting MAME's source in a member's post (page 8)
was displayed and not used. Nemesis's page 11 blocks building the two tables and converting the exponent turned out to be his emulator's
code (one is a method of its YM2612 class) and were displayed before that was seen: the form of the tables is in the
prose beside them, the two offsets that fix their entries' last bits are not, and Nephrite's offsets are therefore
recorded as not cleanly sourced (D-16) until a measurement replaces them. Two page 12 blocks carrying an emulator's
identifiers were not opened, found by a search for them; of his page 28 SSG-EG code, which he says is taken from his
core, only a two-line fragment was displayed, and none of it is used. No RTL file
was opened; the board's OPN2 was run as a black box through its output pins (D-15, D-16).

### 19.3 Measured (2026-10-05): the references' audio

Each voice through Genesis Plus GX, PicoDrive, BlastEm and ClownMDEmu (`--wav`) and Nephrite (`examples/wav.rs`),
the fundamental and harmonics measured over one second:

| Voice | Nephrite | Genesis Plus GX | PicoDrive | BlastEm | ClownMDEmu |
|---|---|---|---|---|---|
| `fm-sine` (frequency 1,081, block 4): frequency | 439.30 Hz | 439.30 | 439.00 | 439.30 | 439.35 |
| its level over the DAC square's swing | 0.500 | 0.504 | 0.501 | 0.500 | 0.497 |
| `fm-tl16`: level over `fm-sine`'s | 0.250 | 0.263 | 0.250 | 0.263 | 0.262 |
| `fm-mul3`: frequency | 1,318.0 Hz | 1,318.0 | 1,317.0 | 1,318.0 | 1,318.0 |
| `fm-dt3`, `fm-dt7`: offset from `fm-sine` | +0.46, −0.46 Hz | +0.46, −0.46 | +0.46, −0.44 | +0.46, −0.46 | +0.46, −0.46 |
| `fm-chain` (algorithm 0): harmonics 2-5 over the fundamental | 10.96, 0.61, 3.63, 1.18 | 10.96, 0.57, 3.51, 1.15 | 11.03, 0.60, 3.65, 1.19 | 10.72, 0.52, 3.11, 0.96 | 11.05, 0.55, 3.22, 0.98 |
| `fm-feedback5`: harmonics 2-5 over the fundamental | 0.596, 0.311, 0.213, 0.171 | 0.597, 0.310, 0.211, 0.167 | 0.585, 0.310, 0.211, 0.171 | 0.579, 0.286, 0.184, 0.138 | 0.579, 0.287, 0.185, 0.138 |
| `fm-decay`: level 0.78 s after the key-on | −7.6 dB | −7.2 | −7.7 | −7.0 | −7.6 |
| `fm-attack` (AR 12): −6 dB to −1 dB | 40 ms | 50 | 40 | 40 | 40 |

The three references that put small sines at 0.263 rather than 0.250, and give a pure sine odd harmonics of 0.5-2%,
model the ladder effect, which Nephrite and PicoDrive do not yet; BlastEm's and ClownMDEmu's weaker upper harmonics
in `fm-chain` and `fm-feedback5` fit a model 1 output filter. The attack's first 20 dB come up faster in Nephrite and
PicoDrive than in the others (25 ms against 45-50 from −40 dB): those two cut each carrier to its top nine bits, as
Sauraen reads the die, so the attack's quietest part is silent in them; between −6 and −1 dB they agree.

- **The crate's tests**: 66 pass, among them the phase increment against Nemesis's worked examples (block, detune,
  multiple, detune overflow, block 0's lost bit), the rate formula and increments, the operator's quarter-wave
  symmetry and levels, the tables against their description, a sine at 439.3 Hz peaking at 255 and TL 16 at 63, and
  feedback 5's harmonics against the references within 3% (which fails with the feedback shifted by 9 less FB).
- **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the Hedgehog and its sequel,
  Phantasy Star IV, OutRun, Thunder Force IV, VDPFIFOTesting, TiTAN's Overdrive 2, Nemesis's CSM and detune tests,
  `fm-chain`, `fm-feedback5`, `fm-decay` and `dac-square`.
- **The cost**, best of five runs of 600 frames beside the stage's starting point (`94a87e39`): Sonic the Hedgehog
  0.710 s against 0.622, Thunder Force IV 0.806 against 0.720, Phantasy Star IV 0.733 against 0.621, about 0.15 ms a
  frame for the FM unit, inside §5.5's 1.5 ms.
- **The corpus at anchors** (944 games): no game's RAM or picture at frames 120 and 600 differs from step 1's, the FM
  unit changing what is heard and nothing a game reads. WiseMan's Nephrite, runner, discovery and registration tests
  pass (31).

*Added 2026-10-05: the two tables now come from `fm_tables.rs`, written independently from "OPLx decapsulated"'s
printed ROMs (`Nephrite_OperatorTables.md`); they agree with this step's to the bit, and the sentences above about
their sourcing describe the step as committed (D-16).*

### 19.4 Open, and the bench

- **The bench as a referee for the FM unit.** `tb_md.cpp` logs the OPN2's output pins (`TB_AUDIO`): each channel's
  value appears once a sample in its slot, `value + 1` when positive and `value + 1024` when negative, the DAC's
  register showing there as a signed value. A single operator plays a clean sine when its setup writes come without
  waits; ~~with waits between all writes, from the 68000 or from a Z80 program, the channel stays at rest, and a 68000
  write followed by a taken branch was lost where the same with no-operations in its place was not.~~ Until the bench's
  YM2612 is understood (D-16), the references' audio is this step's oracle; MDFourier and Nemesis's recordings come
  with the stage's oracle step. *Both struck statements were wrong, and §20 has what was: the channel played and was
  looked for at the wrong place in the sample, and the lost writes were left at 0 by the busy poll's read.*
- **D-15**, the write timing; **D-16**, the tables' last bits.
- **Next**, in the plan's order: the LFO, SSG-EG, CSM and channel 3's special mode's key-ons, the test register,
  the ladder effect and the multiplexed output, the YM3438 of model 2 as a setting, and the model 1 filter.

## 20. Stage 5, the bench step: the board made the FM unit's referee (2026-10-05)

A dispute step: it changes no code of the core and ends in rules (`Nephrite_Disputes.md` D-14 to D-19). It was put
before the LFO and SSG-EG so that those, the ladder effect and the filter have something to be measured against that
is not another emulator.

### 20.1 What it settled

- **The bench plays, with every write applied** (D-16). It always had: channel 1's turn on the pins moves with the
  Z80's reset line, and step 2 looked for it where a program of another kind puts it. `mdboard.py pins` logs a
  program's pins and `mdboard.py fm` compares the busiest channel with `examples/fmtrace.rs`'s output, sample for
  sample, aligned on nothing but each one's first sample off rest.
- **The lost writes** (D-15). A status read in the instruction after a data write leaves 0 in the register at two of
  every six places in a slot, on the board, for an operator's register and a chip-level one alike; one NOP between
  the two keeps every write. The first voices polled the busy flag at once, and lost a third of their writes to it.
  Sauraen's write held for its operator's turn is not what was lost, and remains unmeasured.
- **The tables on the board** (D-16). With the board's order in, moving any one of the 512 entries of the two ROMs up
  or down by one (1,015 changes, a zero entry not going down) changes at least one of 200,220 compared samples, and
  as they stand none differs: the board's FM unit computes with the OPL2's ROMs as printed. The tables in the tree
  were not touched.
- **Three rules the committed unit does not have**, each measured on the board and left for the writer's step that
  follows:
  - operator 1's output reaches operators 2 and 4 and the sum a sample late, and operator 3 a sample later still;
    the envelope's step shows from the sample after its cycle (D-17);
  - rates 48 to 59 take their double steps on other cycles of the four than step 2 read from Nemesis's table (D-18);
  - the Z80's reset line resets the YM2612, which Nephrite does not do at all (D-19).
- **The square's slower loop** (D-14) is D-2's refresh, not the access: the board's `dac-square` is 535.04 Hz with
  BlastEm and PicoDrive, 112.9 of the 68000's clocks a half period over Nephrite's, of which the refresh is 110.2.

### 20.2 Measured (2026-10-05)

Against the board's pins, 62 voices and 200,220 samples (the voices by name in D-16):

| The FM unit | Algorithms and levels (13 voices, 67,482) | Envelopes (3 voices, 11,700) | Decays by rate (20, 31,038) | Attacks by rate (16, 48,000) | S3 into S4 (10, 42,000) | All |
|---|---|---|---|---|---|---|
| as committed | 43,200 | 7,603 | 28,821 | 9,523 | 40,708 | 129,855 |
| operator 1 a sample late | 67,482 | 7,603 | 28,821 | 9,523 | 40,708 | 154,137 |
| and the envelope's step after the output | 67,482 | 11,448 | 30,439 | 38,900 | 42,000 | 190,269 |
| and rates 48 to 59 on the board's cycles | 67,482 | 11,700 | 31,038 | 48,000 | 42,000 | 200,220 |

Only the first row is the tree's `fm.rs`; the others were builds made outside it for the comparison, and none of them
is committed. The measure counts exact samples and nothing else: an envelope whose step comes a sample early or late
differs in most of the samples that follow, so 9,523 of 48,000 says that the attacks' steps are misplaced, not how
far their levels are out, which §19.3's figures against the references bound. Those figures are the committed unit's
and stand as measured; how the three rules move them is for the writer's step to measure.

- **The writes**: `mdboard.py read-after-write` with 0, 1 and 2 NOPs, 49 trials each: 18 writes left at 0 and 31 kept;
  49 kept; 49 kept. On the DAC's register with none: 16 left at 0, 33 kept. With the status reading `$03`: 14 left
  at 0, 35 kept, none at 3.
- **The reset**: `mdboard.py reset-replay`, with the line held 14.4 samples and with it held 1.3: D-19's figures.
- **The crate's tests** were not run again for this step: nothing they build changed. `mdboard.py`'s new programs
  were checked to be the ones measured, byte for byte, for the 46 voices added to `SOUNDS` and for the reset's two.

### 20.3 The bench's tools

- `tb_md.cpp`: `TB_PINS` (the Z80's reset and bus acknowledge at each change, and the YM2612 clock's edges counted)
  and `TB_ZBUS` (the Z80 bus between the chips, at each change of its strobes, address or data). The second reads four
  of the board's own nets by name, which the top level does not bring out; the generated headers were searched for the
  names and no logic was read (D-16).
- `mdboard.py`: `rate_voice`, `rom_voice`, `read_after_write` and `reset_replay`, with 46 voices more in `SOUNDS`;
  `bench`, `pin_records`, `channel_slot`, `fm_series` and `fm_compare`; the commands `pins`, `fm`, `read-after-write`
  and `reset-replay`. `ym_writes` now waits out the busy flag before each write and leaves four NOPs after it, where
  step 2's left eighty; every voice's key-on therefore comes sooner than in §19.3's runs, none of whose figures is
  taken from when it comes. `z80_writer` takes the 68000's code to run once the Z80 is started.

### 20.4 The sources

No RTL file was opened. Two files Verilator generates from the board were searched for the names of pins and nets,
and the names of the generated files listed (D-16). Of topic 386, from the copy saved at step 2: page 54's table of
the chip's 24 slots, read as a table with the three lines of that post that carry code left undisplayed (D-17), and,
in passing while looking for the increments' table, Nemesis's table of the block's shift, which step 2 had already
read, its neighbouring lines that carry code likewise left undisplayed. Charles MacDonald's `gen-hw.txt` on `$A11200`
(D-19). No emulator's source was read, and no entry of either operator table was written or changed.

### 20.5 Open, and next

- **For a console, not the board**: whether a register read at once after its write is left at 0 (D-15). The board
  is a model of the chip, and this is the one finding of the step that the board alone should not decide: a driver
  that polls the busy flag straight after a write would be made to lose notes by it.
- **Unmeasured still**: when a write held for its operator's turn lands (D-15); the status at ports 1-3 (D-13), for
  which the board has an input; whether an access costs one clock more or sometimes two (D-14); the short reset
  pulse's one odd case (D-19). The ten-bit pins show a level that is not negative one higher than a negative one,
  which looks like the ladder's step and is for the step that builds the ladder effect to measure there.
- **Next**: a writer's step that puts D-17, D-18 and D-19 into `fm.rs` and `ym2612.rs` and pins each with a test
  that the board's samples decide; then, in the plan's order, the LFO, SSG-EG, CSM, the test register, the ladder
  effect, the YM3438 of model 2 as a setting, and the model 1 filter, each against the board's pins.

## 21. Stage 5, the writer's step: the board's FM rules in the tree (2026-10-05)

### 21.1 What it built

- **D-17 and D-18 in `fm.rs`.** Operators 2 and 4 and algorithm 7's sum take operator 1's output of the last sample
  and operator 3 its output of the one before; the envelope's cycle is taken after the sample's output and the
  phases' advance; rates 48 to 59 double their step on the board's cycles. The channel's stored copy of operator
  2's output, which was the operator's own, is gone.
- **D-19 in `ym2612.rs`, `sound.rs` and `genesis.rs`.** The YM2612 has the Z80's reset line (`reset_line`, called
  where `$A11200` changes it): its assertion puts every register, the timers, the keys and the operators back to
  their power-on state and restarts the sample cycle from that moment (`RESTART`, 87 master clocks to the first
  sample); held, the chip makes silence and takes no write; its release starts the envelope's counter with the
  sample it comes before, or the one it follows by no more than `RELEASE_LEEWAY`, 67; and the counter's first cycle
  is the fourth sample from its start, for which the envelope's divider now counts down the samples before its next
  cycle. At power-on the line starts asserted and the sample cycle is left where Nephrite starts it.
- **The state** is version 10: the held line and its first sample, and one stored output fewer in each channel.
- **`board_fm.rs` and `voices.rs`** (tests): the board's samples as block hashes, the voices built as `mdboard.py`
  builds them and checked by their programs' hashes to be the ones the board ran, and the chip driven at the board's
  own write times (§21.2).
- **`mdboard.py`**: `board-fm <logs>` writes `board_fm.rs` from the board's logs, making any that is missing;
  `reset_sweep` and `status_ports`; `board_channel` reads a window of a log; the bench takes `TB_MOL` (both forms of
  the FM output) and `TB_YMSTATUS` (the board's `ym2612_status_enable` input).

### 21.2 How the tests hold it

The board's 62 voices and 200,220 samples (§20.2) are in the tree as the FNV-1a hash of each block of 64, with the
cycle at which the board's bus carried each of a voice's 74 strobes. A test gives the chip, `Ym2612` alone, the reset
line's release and each write at those cycles and counts the blocks that are the board's. Nothing of the 68000's
timing is in it, which is the point: the first form of these tests ran each program on the whole machine, passed
200,220 of 200,220 with D-17 and D-18 in, and was found to pass for a wrong reason (§21.4).

| Test (`voices.rs`) | Voices | Samples the board's |
|---|---|---|
| `the_algorithms_and_levels_are_the_boards_sample_for_sample` | 13 | 67,482 of 67,482 |
| `the_envelopes_are_the_boards_sample_for_sample` | 3 | 11,700 of 11,700 |
| `the_decays_by_rate_are_the_boards_sample_for_sample` | 20 | 31,038 of 31,038 |
| `the_attacks_by_rate_are_the_boards_sample_for_sample` | 16 | 48,000 of 48,000 |
| `a_modulator_into_a_carrier_is_the_boards_sample_for_sample` | 10 | 42,000 of 42,000 |
| `a_voice_replayed_after_a_reset_is_the_boards_wherever_the_release_falls` | 40 replays | 20,480 of 20,480 |

Each rule was taken out in turn and the tests run: with step 2's rows for rates 48 to 59 the envelopes, decays and
attacks fail (`fm-dr49` 895 samples of 1,919); with operator 2 taking operator 1's newest output `fm-chain` has no
block of the board's, and with the sum taking it `fm-alg7` has none; with the envelope's step before the output the
envelopes, decays, attacks and modulator voices all fail; with the counter's first cycle a sample sooner or later,
49 voices and all 40 replays fail. `the_whole_machine_plays_the_algorithms_as_the_board_does` runs the 13 voices in
which no envelope moves through the 68000 and the whole core, and `the_z80s_reset_line_resets_the_ym2612` the line
through `$A11200`.

### 21.3 Measured (2026-10-05)

- **The reset rule** (D-19), on 36 pulses of 1.83 to 2.82 samples and the loader's four of 0.28: every replay is the
  board's for a restart of 76 to 100 master clocks, a leeway of 56 to 80 and a first cycle on the fourth sample, and
  for nothing outside them at the steps of 4 tried. On the pins themselves a key-on written 73 master clocks into the
  sample is heard a sample sooner than one written 103 into it.
- **Which chip** (D-16): the bench's FM chip is the YM3438 inside the FC1004, the model 2 ASIC. Its busy flag reads
  at every port with the board's `ym2612_status_enable` at 1 and at 0 (D-13); its `MOL` is the YM3438's output, level
  plus 256 over three slots of a channel's four, and its `MOL_2612` a rendering of the YM2612's DAC, the level in
  the first slot and a rest of +1 or −1 by its sign in the other three. The two carry the same level in every sample
  compared. Every FM measurement of this stage therefore speaks for the ASIC's YM3438, and for model 1's YM2612 as far
  as the two share their logic.
- **The increments' table** (D-18): Nemesis's Table 2 reads as step 2 read it; it is the table that differs from the
  board, in which cycles of four take a rate's doubled steps and not in how many.
- **The 68000's side** (D-13, D-14): the busy poll's round is 19 clocks on the board where the instructions make 18,
  21 where the refresh falls; the busy flag stands about 1,400 master clocks where Nephrite counts 1,344; a voice's
  key-on is written 65,702 master clocks after the 68000 starts on the board and 62,132 on Nephrite.
- **The crate's tests**: 80 pass. **The conformance kit**, `--frames 600`: C1-C15 pass on the 240p suite, Sonic the
  Hedgehog and its sequel, Phantasy Star IV, Thunder Force IV, VDPFIFOTesting, Nemesis's CSM test, `fm-chain`,
  `fm-decay` and two of the reset programs.
- **The corpus at anchors** (944 games, against Genesis Plus GX at frames 120 and 600): 40 games' RAM moves by a few
  bytes, 21 nearer the reference and 21 further, between −17 and +27; the medians stay 65,523 and 65,506. Two
  pictures move at frame 600: Jerry Glanville's Pigskin Footbrawl, 53 to 58 pixels off the colour map, and OutRun,
  45,990 to 46,143. A sound driver that resets the Z80 now resets the YM2612 and its timers with it, so its timer
  polls come out otherwise; no game stopped or failed to boot. WiseMan's Nephrite, runner, discovery and
  registration tests pass (31).

### 21.4 What the whole machine does, and why it is less than the chip

The same programs run on the whole core, on its own 68000's timing, and compared with the pins as in §20.2:

| The tree | Algorithms and levels (67,482) | Envelopes (11,700) | Decays (31,038) | Attacks (48,000) | S3 into S4 (42,000) | All (200,220) |
|---|---|---|---|---|---|---|
| step 2's unit | 43,200 | 7,603 | 28,821 | 9,523 | 40,708 | 129,855 |
| with D-17 and D-18, the counter cycling on its first sample | 67,482 | 11,700 | 31,038 | 48,000 | 42,000 | 200,220 |
| with D-19's counter as the board has it (this step) | 67,482 | 8,411 | 28,770 | 38,563 | 38,116 | 181,342 |
| the chip alone, at the board's write times (this step) | 67,482 | 11,700 | 31,038 | 48,000 | 42,000 | 200,220 |

The second row is what §20.2 reported from builds outside the tree, and it was right about D-17 and D-18 and wrong
in what it seemed to say of the envelope's timing. Two things were each out by one envelope cycle of three samples
and hid each other: Nephrite's counter cycled on the first sample after the reset's release where the board's waits
for the fourth, and Nephrite's 68000 wrote the key-on 3.54 samples before the board's (D-14). The reset sweeps,
where the Z80 writes the voice and the 68000's timing is not in the question, showed the first; the board's bus log
showed the second. With the counter right, the whole machine's envelopes start a cycle out against the board's
until its 68000 polls the busy flag at the board's pace: the poll's fifth clock, the refresh of D-2 and the busy
flag's length. Every voice in which no envelope moves is unaffected. The difference is an envelope's steps falling
three samples from where the board has them, some 56 microseconds, relative to a key-on the program itself placed
only to within its poll's round; it is a matter of the comparison's exactness and not of anything heard.

### 21.5 The sources

- **Opened, as a logged search**: the bench's `md_board.v` and `fc1004.v`, for how the FM chip is instantiated and
  configured, the lines listed in D-16. *This step was a writer's, and the plan's §1.3 opens the RTL to a dispute
  step alone: D-16 acknowledges the breach since 2026-10-07.* Two lines of the board's mix were printed by the search; they are noted as
  seen and used for nothing. No other RTL file was opened.
- **Topic 386, from the copy saved at step 2**: Nemesis's Table 2 on page 8, its rows read with every identifier
  in the block masked (D-18). **The same search printed, masked the same way, the numeric rows of two blocks in C
  array form on pages 32 and 33**, eight values to a row, which look like tables of vibrato depth and sit in blocks
  that carry an emulator's identifiers. They were not sought, are used for nothing, and are recorded here as
  displayed: the step that builds the LFO must not take its vibrato table from memory of them, and has the board to
  measure it on instead. *The table was then written by another writer, from the board alone: D-20's note of
  2026-10-07.*
- No emulator's source was read otherwise, and the operator tables were not touched.

### 21.6 Open, and next

- **For the 68000's side**, to bring the whole machine to the board's 200,220: D-14's fifth clock for an access to
  the Z80's area, D-13's busy flag at the board's length, and D-2's refresh, which the stage 4 measurements were all
  made without. It is a step of its own, with stage 4's tests in its blast radius.
- **D-15** stays open and unimplemented: a status read straight after a write needs a console, and
  `read_after_write` is kept as its program. When a write other than a key lands is still unmeasured.
- **D-19's short pulses**: a reset held less than about 1.8 samples does not always take on the board; Nephrite's
  always does.
- **Next**, in the plan's order: the LFO, SSG-EG, CSM and the test register, each refereed on the bench, the LFO's
  tables measured there.

## 22. Stage 5, the 68000's side: the whole machine to the board's samples (2026-10-05)

### 22.1 What it built

- **The Z80's area** (`genesis.rs`, D-14): an access to `$A00000`-`$A0FFFF` takes the 68000 five clocks, and the
  YM2612, its Z80 RAM or the reset line sees it four clocks before the 68000's cycle ends, when its strobe on the
  Z80's bus does (`Z80_AREA_WAIT`, `Z80_AREA_LEAD`).
- **The busy flag** (`ym2612.rs`, D-13): 33 slots of 42 master clocks from the first slot edge at or after the data
  write, on both chips.
- **Two refreshes** (`genesis.rs`, D-2): the bus's, every 128 of the 68000's clocks, two clocks to the first access
  at or after one if it is to the cartridge's area and none if it is anywhere else; the main RAM's, three clocks to
  a RAM access within 133 master clocks of its request, the next request 806 after that access or after the 133.
  Both are part of the state, version 11.
- **The phases** (D-2, D-19): the bus refresh and the YM2612's power-on cycle are placed where the board has them
  against the picture (762 and 793 master clocks after power-on), the RAM refresh's first request from the 68000's
  start (1,414).
- **Tests**: `voices.rs`'s `the_whole_machine_plays_every_voice_as_the_board_does`, the 62 voices on the whole
  machine; `genesis.rs`'s `ram_loops_keep_the_boards_time_through_both_refreshes`, three RAM loops read for read
  against the board's (`board_bus.rs`, written by `mdboard.py board-bus`), and
  `the_refreshes_cost_the_boards_share_of_rounds`, D-2's first loops; `ym2612.rs`'s busy flag at the slot edges.
- **The bench**: `tb_md.cpp`'s `TB_MEM` (the 68000's strobes and the board's memory strobes); `mdboard.py`'s
  `RAM_LOOPS` and `board-bus`. Every earlier command is unchanged.

### 22.2 Measured (2026-10-05)

| On the whole machine, against the board | Before this step | After |
|---|---|---|
| The 62 voices' samples | 181,342 of 200,220 | 200,220 of 200,220 |
| A voice's 3,930 reads, at the board's 68000-start phase | from the first access to the Z80's area, 7 to 3,570 master clocks out | every read to the master clock |
| D-2's cartridge loop: rounds of 26 clocks | 0% (the board 19.0%) | 19.1% |
| D-2's RAM loop: rounds of 26, 27, 29 | 0%, 0%, 0% (13.0, 16.2, 3.8) | 13.2%, 16.2%, 3.7% |
| D-13's busy sweep, 112 reads | (not run) | 107 the board's |

The busy sweep's five others are reads near the flag's end, not traced one by one; run from the board's 68000-start
phase, the first place its reads leave the board's is a read of the YM2612 that took six clocks where a bus refresh
fell, which no rule here gives.

- **Stage 4's tests**: every one passes, VDPFIFOTesting 122 of 122, the write and transfer sweeps at every point
  with the write path's start and the transfer's first read unchanged (176, 88; D-11). With the bus refresh placed
  from the 68000's start instead of against the picture, one point of the write sweep moves a slot; with either
  refresh taken out, the register sweep and the write sweep fail, which they did not before the refreshes, so those
  pictures now depend on them.
- **The crate's tests**: 82 pass. **The conformance kit**, `--frames 600`: C1-C15 on the 240p suite, Sonic the
  Hedgehog and its sequel, Phantasy Star IV, Thunder Force IV, VDPFIFOTesting, the CSM test, `fm-chain`,
  `fm-decay`, two of the reset programs, `ram-reads-3` and the busy sweep.
- **The corpus at anchors** (944 games, against Genesis Plus GX at frames 120 and 600): 789 games move, and toward
  the reference. Pictures on the reference's to the pixel, up to a colour map: 718 to 827 at frame 120 and 630 to
  772 at frame 600; of the pictures that change, 361 come nearer and 61 go further. RAM's median equal bytes: 65,523
  to 65,527 and 65,506 to 65,514; of the RAM counts that change, 806 come nearer and 423 go further. No game fails to
  run. The largest moves are pictures whole frames apart before and the reference's after (International Rugby,
  ECCO, Heavy Unit, OutRun, Traysia), with Streets of Rage the other way at frame 600. WiseMan's Nephrite, runner,
  discovery and registration tests pass (31).

### 22.3 Why the phases are placed as they are

Nephrite's 68000 starts 0.61 of a frame later against the picture than the board's: the board's line interrupt on
line 150 comes 864,577 master clocks after its 68000's first bus cycle, Nephrite's 1,409,169 after its own, the
picture otherwise the board's to within its interrupt latency. It is why stage 4's picture tests compare Nephrite's
frame 3 with the board's frame 2. Every phase the board shows is fixed to its own power-on, so one of two things in
Nephrite must differ from the board: the refreshes' and the sound's place against the 68000's start, or their place
against the picture. ~~The picture is what a game is timed by, its sound drivers by the vertical interrupt, and the
write sweep of D-11 shows it: placed from the 68000's start, the bus refresh moves one of its points. So the bus
refresh and the YM2612's cycle are placed against the picture, and the voices, which start from the 68000's start,
still give the board's every sample, their reads 113 master clocks from the board's where a refresh falls.~~ The RAM
refresh's next request follows the accesses made, so it has no place against the picture to keep; its first is
where the fit put it.

*Decided 2026-10-06, at stage 5's placement step (§25.1):* the YM2612's cycle is placed where the board has it
against its 68000's start, its first deadline 910 master clocks after power-on. The bus refresh stays against the
picture (762): placed from the 68000's start (875) it still moves one point of D-11's write sweep (H40, the display
on, 18 NOPs: master clock 61 where the board has 45), which the decision made a condition of moving it. Between the
two, the 68000's cycles in a sound program fall a refresh's wait from the board's here and there, and §25.3 gives
what that costs the sound's tests; placed as the board is in both, every cycle of every program measured is the
board's.

~~Moving Nephrite's picture to the board's place against the 68000's start would remove the choice, and would move
every game's frames against Genesis Plus GX at the corpus anchors and every stage 4 picture test by a frame. It is
not done here.~~

*Decided 2026-10-06, at stage 5's picture step (§27):* the picture is moved. As the 68000 starts, Nephrite's VDP
stands where the board's does, 745 master clocks into line 159, measured from the HV counter; the bus refresh, the
RAM's and the YM2612's cycle are placed against the 68000's start as the board has them; and the frame offset is
gone. The move did not cost a stage 4 point: the points that moved with it on the way (the write sweep's 24 NOPs,
a transfer sweep's 4, VDPFIFOTesting's FIFO Wait States, the register sweep) were each traced on the board to a rule
of the 68000's cycle against the VDP (D-26), and every stage 4 test passes with those rules. Nephrite's picture after n
frames is the board's frame n-1 counted from 0, the same frame, Nephrite's first being the rest of the board's
frame 0 from line 159.

### 22.4 The sources

No RTL file was opened. The generated header was searched for the names of eight more of the board's nets (`AS`,
`UDS`, `LDS`, `RW`, `DTACK`, `RAS1`, `CAS1`, `OE1`), read by `TB_MEM`. Charles MacDonald's notes and the
68000's manual, as before, for the accesses' lengths. No emulator's source was read.

### 22.5 Open, and next

- **D-2**: the RAM's few four- and five-clock waits, and ~~one six-clock read of the YM2612 where a bus refresh fell~~
  (the Z80's area under a refresh, §25.2).
- **Stage 4's frame offset** (§22.3): Nephrite's picture against its 68000's start.
- **D-15** stays open and unimplemented; **D-19's short pulses** as before.
- **Next**, in the plan's order: the LFO (its vibrato table written separately, on `nephrite-lfotables`), SSG-EG,
  CSM and the test register, each refereed on the bench.


## 23. Stage 5, the LFO (2026-10-06)

### 23.1 What it built

- **The counter** (`fm.rs`, D-20): seven bits, a step every 108, 77, 71, 67, 62, 44, 8 or 5 samples by `$22`'s
  speed, taken at the start of a sample before its operators. Turned off, the count is held at 0 and the divider
  that times its steps runs on, so the first step after it is turned on again is short.
- **The tremolo**: the count as a triangle, 126 down to 0 and up to 126 again, shifted by AMS (none, 3, 1, 0) and
  added with TL to the attenuation of each operator whose AMON bit is set.
- **The vibrato**: PMS and the channel's frequency through `lfo_tables.rs`'s `pm_fnum`, the table written and
  measured separately (`Nephrite_LfoTables.md`); the phase's increment from the modulated frequency, the key code
  (detune and key scaling) from the register's.
- **Each operator's count**: S1 of channels 1 to 5 take the count the next sample will have, S1 of channel 6 and
  S2-S4 of channels 1 to 5 this sample's, S2-S4 of channel 6 the one before, for the tremolo and the vibrato alike.
- **Channel 3's special mode**: S1-S3 take the vibrato of their own frequencies (§23.3).
- **The state**: version 12, the count, the one before it and the divider (`Sound.FmLfo`).
- **Tests**: `voices.rs`'s `the_lfo_voices_are_the_boards_sample_for_sample`, the 61 LFO voices against the board's
  samples (`board_fm.rs`'s `LFO_VOICES`, written by `mdboard.py board-fm`).
- **The bench**: `mdboard.py`'s LFO voices (D-20 lists them) and `fm_voice`'s `lfo`, `b4` and `pre`; every earlier
  image is byte for byte what it was.

### 23.2 Measured (2026-10-06)

| Against the board | Voices | Samples |
|---|---|---|
| The eight speeds, AMS 1-3, the switch off and on, AMON on a modulator | 14 | every one |
| PMS 1-7, PMS 7 at frequencies 1,151 and 1,500, the key code under vibrato, S1 alone | 10 | every one |
| The 24 operators under vibrato alone, nine more under tremolo alone | 33 | every one |
| Channel 3's special mode without vibrato, and with it on the normal mode | 2 | every one |
| Channel 3's special mode under vibrato (`fm-pm-ch3`, `fm-pm-ch3-swap`) | 2 | to the 65th and the 25th |

59 voices of 61 are the board's to the sample, 574,600 samples. Every rule in §23.1 was tried against its
alternatives and the test fails with each alternative (D-20).

The rate counter's figures in `Nephrite_LfoTables.md` §4 were checked, not assumed: the steps of 432, 308, 284, 268,
248, 176, 32 and 20 samples a vibrato step are four of these counts; the tremolo steps four times a vibrato step;
clearing bit 3 zeroes the count at the next sample, so the offset within two; and turning the LFO on again starts at
count 0 with a first step as short as the free-running divider leaves it, which is the short first step measured
there. The manual's speeds in Hz are those of a counter one sample longer a step; the board's are implemented and
the difference is recorded in D-20.

The vibrato table agrees with the board in every voice that uses it.

- **The crate's tests**: 90 pass, with the ROMs.
- **The conformance kit**, `--frames 600`: C1-C15 on the 240p suite, Sonic the Hedgehog and its sequel, Phantasy
  Star IV, Thunder Force IV, VDPFIFOTesting, the CSM test, `fm-chain`, `fm-decay`, two of the reset programs,
  `ram-reads-3`, `fm-lfo-restart` and `fm-pm-c6s2`.
- **The corpus at anchors** (944 games, against Genesis Plus GX at frames 120 and 600): no game's picture or RAM
  moves from §22's; the two the reference cannot run are as before. WiseMan's Nephrite, runner, discovery and
  registration tests pass (31).
- **The frame cost** after the refreshes and the LFO (best of five runs of 600 frames, the desktop, release build):

| Game | Mean per frame |
|---|---|
| Sonic the Hedgehog | 1.255 ms |
| Sonic the Hedgehog 2 | 1.147 ms |
| Thunder Force IV | 1.269 ms |
| Streets of Rage 2 | 1.045 ms |
| Phantasy Star IV | 1.314 ms |

  All are within P1's 1.5 ms. Sonic the Hedgehog was 1.18 ms at stage 5's second step.

### 23.3 Channel 3's special mode under the vibrato

Without vibrato the special mode is the board's. With it, no rule found gives the board's samples past the first
few dozen: S1 at its own frequency keeps them to the 65th and 76% of them after (the two frequencies swapped, the
25th and 83%), the channel's frequency for all three or no offset at all fails within six. The board's phase runs
ahead by about one of its 1,024 steps in thirty samples, and no one frequency for each vibrato step explains it:
within a step the frequency moves, which no normal channel's does. The rule implemented is the best of those tried,
and the test holds both voices to their first agreement and no further, so that a rule that mends them shows.

### 23.4 Streets of Rage at frame 600

At §22 Streets of Rage's picture at frame 600 left Genesis Plus GX's. The board was run to frame 604, its pictures
captured from frame 596 and every write to the 68000's RAM logged, and Nephrite's writes were traced the same way.
Nephrite is now the board's.

- **The game is the board's, interrupt for interrupt.** From the vertical interrupt of frame 80, where the game's
  frame loop begins, to that of frame 600, every interval between two vertical interrupts holds the same writes on
  the board and in Nephrite: 483 intervals of 483. In each interval, every write is within 607 master clocks of the
  board's place, and in the median interval the largest difference is 179. The 38 intervals of a loading phase
  between frames 288 and 326 are left out of that count, because the interrupt does not fall in the game's wait
  loop there. That phase's 81,389 writes are the board's, in the board's order. By its end Nephrite's are up to
  27,000 master clocks early (0.09% of the phase), and the wait that ends it brings them back.
- **The picture.** The intro scrolls its city by a line every fourth frame. The scroll is written to VSRAM in the
  vertical interrupt and shown in the next picture, on the board as in Nephrite. Each of the board's pictures from
  frame 597 to 602 is named by the frame its first line is drawn in. Named that way, each is Nephrite's picture of
  the same frame to the pixel, up to a colour map, and the scroll moves between frames 600 and 601 on both.
- **Before §22.** Without the refreshes' waits, Nephrite's game ran two vertical interrupts ahead of the board's.
  At frame 600 its frame counter at `$FFFB09` read 232, where the board's game and Nephrite's now give 230.
- **The reference.** Genesis Plus GX's RAM at frame 600 is Nephrite's, 65,521 bytes of 65,536 with the same frame
  counter. Its picture, though, is the board's picture of frame 601, so it shows a game state one frame earlier
  than the board does. A picture one frame off a four-frame scroll differs on one frame in four. At §22, Nephrite
  went from one frame early, which agreed with the reference at frame 600, to the board's place, which is one frame
  behind the reference and differs from it there.

Nephrite and the board stand differently against their own power-on. Nephrite's 68000 makes its first write
0.02 of a frame after power-on and the board's 0.80, and a given vertical interrupt comes 1.17 frames sooner in
Nephrite than on the board. The comparison therefore pairs the board's interrupt at 0.03 of a frame with
Nephrite's at 0.86 of the frame two before. That a picture named by the frame it starts in then agrees is a
finding for this game; it is not a rule for stage 4's frame offset (§22.3).

The bench: `tb_md`'s RAM writes (its `w` lines) give the RAM's own address, which is the 68000's with bit 14
inverted. The comparison corrects for it.

### 23.5 The sources

The YM2608's manual (§2-6) for `$22`, PMS, AMS and AMON; `Nephrite_LfoTables.md` for the vibrato's offsets and the
counter's steps as measured there. The tremolo's triangle, its shifts and every operator's place were taken from
the board's samples alone. No RTL file was opened; no emulator's source was read.

### 23.6 Open, and next

- **D-20**: channel 3's special mode under the vibrato.
- **D-15** stays open and unimplemented; **D-2**'s few long RAM waits and **stage 4's frame offset** as in §22.5.
- **Streets of Rage's loading phase** runs 0.09% faster in Nephrite than on the board (§23.4). It is not yet
  traced to a rule.
- **Next**, in the plan's order: SSG-EG, CSM and the test register, each refereed on the bench.

## 24. Stage 5, SSG-EG, CSM and the test register (2026-10-06)

### 24.1 What it built

- **SSG-EG** (`fm.rs`, D-21): the four bits of `$90`-`$9E`; the decay, sustain and release four times as fast and
  stopping at `$200`; each sample's turn at `$200` (restart, alternate, hold); the output inverted about `$200`; a
  key-on clearing the inversion.
- **The key-off** (D-21): the release starts from the level the envelope last put out, for every operator.
- **CSM** (`ym2612.rs`, `fm.rs`, D-22): in mode `10` the load of timer A and each overflow at a sample's tick key
  channel 3's four operators for that sample, OR'd with `$28`'s keys.
- **The timers** (D-22): a tick `LOAD_LEAD` before each deadline, timer B's every sixteenth; a load taken at the
  next tick and its value a slot later; a flag reading set from 276 (timer A) or 234 (timer B) master clocks before
  the deadline of its overflow's sample; `$27` taken at once.
- **The test register** (D-23): `$21` bits 1 to 5 (the LFO and the timers by slots, the phase from nothing, the
  outputs' top bit, the envelopes held at full level), `$2C` bit 5 (the DAC on every channel), the DAC's data at `$80`
  from reset.
- **The channel's sum** (D-23): the carriers added S1, S3, S2, S4, held to nine bits at each addition.
- **The operators' order and the key** (D-24): S1 keyed a sample after S2-S4 and its output used at once; each
  channel's key taken a slot after the last channel's, channel 1's `KEY_LEAD` (472 master clocks) before the
  deadline, a key that comes later waiting for the next sample; the LFO's count before its step and the envelope's
  cycle the same for every operator. D-20's open item, channel 3's special mode under the vibrato, is settled by it.
- **The frame** (D-24, D-19): a deadline is the last moment a write is in time for its sample; `RESTART` 559 and
  `POWER_ON` 1,265, D-19's placements expressed in it. The busy flag's slot edges 32 master clocks into a slot (D-13).
- **The state**: version 13: the timers' loads, the control register as they have it, their next tick, the flags'
  rises, up to eight keys waiting, SSG-EG's inversion and the envelope's last level for every operator, S1's keys,
  CSM's key.
- **Tests** (`voices.rs`):
  - `the_chip_voices_are_the_boards_sample_for_sample`: the step's 120 voices on the chip alone, each write at the
    master clock the board's 68000 made it (`board_chip.rs`, written by `mdboard.py board-chip`).
  - `the_lfo_voices_on_the_chip_are_the_boards`: §23's 61 voices the same way.
  - `keys_and_the_timers_load_land_in_the_boards_samples`: the key sweeps of channels 1 to 3 and the CSM sweep on the
    whole machine placed as the board is.
  - The whole-machine tests at both placements (§24.3).
  - `ym2612.rs`'s timer tests and `sounds.rs`'s polling programs, at the board's counts.
- **The bench**: `mdboard.py`'s `fm_voice` with SSG-EG registers, `$27`, NOPs before a write and after one; the
  voices of D-21 to D-23 and `LANDING`'s sweeps; `status_loop`; `board-chip`. `examples/chip.rs` gives the chip a bus
  log's writes and reads. Every earlier image is byte for byte what it was.

### 24.2 Measured (2026-10-06)

| On the chip alone, each write at the board's master clock | Voices or runs | The board's to the sample |
|---|---|---|
| SSG-EG (D-21) | 62 | 62 |
| CSM, the envelope by operator, the channel's sum | 29 | 29 |
| The test register (D-23) | 29 | 20, and 9 that differ where their write lands (D-15) |
| The LFO's voices of §23 | 61 | 60, and `fm-lfo-restart` where its `$22` writes land |
| The key sweeps, S1-S4 of channels 1-6 | 960 | 960, each in the board's sample |
| The CSM sweep | 40 | 40 |
| The status loops: the timers' flags; the busy flag | 103 | 103; 102 |

- **The crate's tests**: 96 pass. **The conformance kit**, `--frames 600`: C1-C15 on §23's 14 images and
  `fm-ssge`, `fm-csm` and `fm-test-timer`. **WiseMan**'s Nephrite, runner, discovery and registration tests pass
  (31).
- **The corpus at anchors** (944 games, against Genesis Plus GX at frames 120 and 600): 79 games move from §23's,
  their sound drivers timed by the busy flag, the timers and the keys. Pictures on the reference's to the pixel are
  827 and 772, as before; of the five pictures that change at frame 600, two come nearer and three go further. Of the
  RAM counts that change, 11 of 22 come nearer at frame 120 and 31 of 70 at frame 600; the medians are 65,527 and
  65,513 (65,514). The two games the reference cannot run are as before, and no game fails to run.
- **The frame cost**, best of five runs of 600 frames, the desktop, release build:

  | Game | Mean per frame | At §23 |
  |---|---|---|
  | Sonic the Hedgehog | 1.270 ms | 1.255 ms |
  | Sonic the Hedgehog 2 | 1.160 ms | 1.147 ms |
  | Thunder Force IV | 1.258 ms | 1.269 ms |
  | Streets of Rage 2 | 1.060 ms | 1.045 ms |
  | Phantasy Star IV | 1.345 ms | 1.314 ms |

  All within P1's 1.5 ms.

### 24.3 Where the whole machine stands

The chip was refereed alone because the whole machine is not the board's in these programs' timing. Its 68000 makes
the YM2612's writes at the board's master clocks only when the bus refresh and the YM2612's cycle are placed where the
board has them against its 68000's start (the refresh 875 after power-on, as `genesis.rs`'s RAM loop test places it,
and the first deadline 910): then every write of a program on part 0 lands at the board's master clock, and the key
sweeps of channels 1 to 3 and the CSM sweep are the board's. Programs on part 1 still drift there, by a poll of the
busy flag here and there and once by 15 of the 68000's clocks, which is not traced (§24.5).

Nephrite keeps §22.3's placement against the picture, a third of a sample from the board's against the 68000, so that
a write the 68000 times lands in another sample now and then:

| On the whole machine | Placed as the board is | Nephrite's placement |
|---|---|---|
| §21's 62 voices | 62 | 62 |
| §23's 61 LFO voices | 51 (`fm-lfo-restart`, nine on part 1) | 58 (`fm-lfo-restart`, channel 3's two in its special mode) |
| The step's 120 voices | 108 (the test register's nine, three on part 1) | 89 |
| The key and CSM sweeps, channels 1-3 | 520 of 520 | not tested |

The earlier per-operator LFO counts (D-20) and the busy flag's edge (D-13) were fitted on the whole machine at
Nephrite's placement, and absorbed it; on the chip they are replaced by D-24's rules and D-13's new edge. Whether the
YM2612's cycle, or the refresh with it, should be placed against the 68000's start, as the sound's tests are, rather
than against the picture is §22.3's decision, and is not changed here.

### 24.4 The sources

The YM2608 manual (§2-6) for `$90`-`$9E`, `$27` and CSM. SpritesMind topic 386, as prose: Nemesis's SSG-EG and CSM
sections and his 2010 corrections; Sauraen's test register posts. The thread was read for this step with its code
blocks withheld, and none was displayed; of the page 28 SSG-EG code, the two lines displayed at stage 5 step 2
(§19.2) are not used. Every rule was measured on the board's pins and bus, its alternatives tried. No RTL file was
opened; no emulator's source was read.

### 24.5 Open, and next

- **D-15**: when each register's write is taken, measured for 15 kinds (D-15's table) and not implemented: the
  unit would make each sample up to two samples after its deadline.
- **The test read** (`$21` bits 0, 6, 7, `$2C` bit 4): not built.
- ~~**The whole machine's placement** (§24.3) and **the 68000's drift on part 1**.~~ Both at §25.
- **D-2**'s few long RAM waits, **stage 4's frame offset** and **Streets of Rage's loading phase** as in §23.6.
- **Next**, in the plan's order: the multiplexed output and its ladder effect, the YM3438 of model 2 as a setting,
  and the model 1 filter.

## 25. Stage 5, the placement and part 1's drift (2026-10-06)

### 25.1 What it built

- **The YM2612's cycle** (`ym2612.rs`, D-19, D-24): placed where the board has it against its 68000's start,
  `POWER_ON` 910 (it was 1,265, against the picture), as §22.3 now decides.
- **The bus refresh**: kept against the picture, `REFRESH_FIRST` 762. Placed from the 68000's start (875) it moves
  one point of D-11's write sweep, H40 with the display on and 18 NOPs, to master clock 61 where the board has 45
  (`pictures::writes_land_where_the_board_lands_them`), and the decision made every stage 4 test a condition of
  moving it.
- **The Z80's area under a refresh** (`genesis.rs`, D-2): a cycle to `$A00000`-`$A0FFFF` that starts one to six of
  the 68000's clocks after the latest refresh request waits a clock more, after its strobe on the Z80's bus
  (`REFRESH_Z80_LATE`); the YM2612, the Z80's RAM and the reset line see the access where they would without it.
- **The bench**: `mdboard.py`'s `ym_latches`, a write's moment from its strobe's fall rather than its rise, which a
  held cycle moves; `board_chip.rs`'s write times regenerated from it (28 voices' times change), and `SWEEPS`
  extended to the key sweeps of all six channels.
- **Tests** (`voices.rs`): `keys_and_the_timers_load_land_in_the_boards_samples` over all six channels' key sweeps
  and the CSM sweep; the LFO's and the step's voices on the whole machine at both placements, each with its own
  expected list or count; `sounds.rs`'s timer poll at both.

### 25.2 The drift on part 1

§24.3 left the 68000's writes on channels 4 to 6 drifting from the board's, by a poll of the busy flag here and
there and once by 15 clocks. Loops of `$A04000` reads and writes, 0 to 7 NOPs apart, were traced on the board with
`TB_MEM` and compared with Nephrite's cycles one by one. The bus refresh's request comes every 896 master clocks at a
phase the board's own slow cycles fix: in three loops, each of the 670 cartridge cycles first after a request waits
its two clocks. A cycle to the Z80's area first after a request does what D-2 had not recorded: starting within a
clock of it, it has its ordinary five clocks and the cartridge cycle after it waits nothing, the refresh taken free;
starting two or four clocks after it, it takes six. With that rule every cycle of the part 1 sweeps traced starts at
the board's master clock, the slip and the extra polls gone. The held cycle's strobe rises a clock late on the Z80's
bus, but the YM2612 takes the write where the strobe would have risen: the busy flag of all 36,180 data writes in the
sweeps and status loops ends where D-13's edge has it from that moment, and from the late rise the held ones' do not.

### 25.3 Measured (2026-10-06)

| On the whole machine | Placed as the board is | Nephrite's placement (762, 910) | Before: 762, 1,265, without the rule |
|---|---|---|---|
| The 68000's cycles of 16 traced runs (12 loops, 4 key sweeps) | 83,451 of 83,451 at the board's master clock | not compared | (the drift) |
| §21's 62 voices | 62 | 62 | 62 |
| §23's 61 LFO voices | 60 (`fm-lfo-restart`, D-15) | 16 | 58 |
| §24's 120 voices | 111, as on the chip (the test register's nine, D-15) | 94 | 89 |
| The key and CSM sweeps, 25 of 40 runs | 1,000 of 1,000 | not tested | channels 1-3 only |
| Timer A's poll (the board: 77) | 77 | 77 | 79 |

Placed as the board is, the whole machine is now what the chip alone is: every voice that differs differs at D-15.
At Nephrite's own placement it is not. Its refresh stands 113 master clocks from the board's against the YM2612's
cycle, so the 68000's sound programs meet the refresh, and the Z80's area its extra clock, at other cycles than the
board's, and a write that falls near a sample's deadline goes to the other sample. Kept at 1,265 instead, with the
rule, the counts are 50 LFO voices and 82 of the 120, with the timer's poll 79: neither placement of the YM2612 gives
the board's sound while the refresh is kept against the picture. The cause is stage 4's frame offset, Nephrite's
picture 0.61 of a frame from the board's against its 68000's start (§22.3): with the picture moved to the board's
place, the refresh could stand at 875 and the stage 4 sweep would keep its point.

- **The crate's tests**: 96 pass, VDPFIFOTesting 122 of 122 and stage 4's picture tests among them, the write
  sweep's H32 point included. **The conformance kit**, `--frames 600`: C1-C15 on §24's 17 images. **WiseMan**'s
  Nephrite, runner, discovery and registration tests pass (31).
- **The corpus at anchors** (944 games, against Genesis Plus GX at frames 120 and 600): 239 games move from §24's,
  their sound drivers' writes now landing where the YM2612's new cycle takes them. Pictures on the reference's to the
  pixel: 827 at frame 120, as before, and 772 to 773 at frame 600; of the 11 pictures that change at frame 600, 7 come
  nearer and 4 go further, and of the 2 at frame 120, one each way. Of the RAM counts that change, 61 of 116 come
  nearer at frame 120 and 88 of 176 at frame 600; the medians are 65,527 and 65,513 (65,513.5). No game fails to
  run.

### 25.4 The sources

No RTL file was opened. `TB_MEM`'s strobes, `TB_ZBUS`'s Z80 bus and the board's pins, as at §22 and §24;
Charles MacDonald's notes and the 68000's manual, as before. No emulator's source was read.

### 25.5 Open, and next

- **Stage 4's frame offset** (§22.3): moving Nephrite's picture to the board's place against its 68000's start would
  let the refresh stand where the board's does and bring Nephrite's own placement to the board's sound.
- ~~**D-15**, when each register's write is taken: next, as the write pipeline.~~ §26, D-25.
- **The test read** and **D-2**'s few long RAM waits, as in §24.5.

## 26. Stage 5, the write pipeline (2026-10-06)

### 26.1 What it built

- **The pipeline** (`ym2612.rs`, D-25): a write to any register but the timers' waits, in the order made, with the
  deadline of the sample that takes it: the first whose slot edge for that part of the register comes at or after
  the write. Each part has its edge, measured (D-25's table): the operator's registers and the algorithm the 18th
  after the deadline, the feedback the 6th, the frequency and the LFO's sensitivities the 24th, the panning the 42nd,
  `$22` and `$21` bit 4 the 35th, `$21` bit 3 the 11th and bit 5 the 31st, channel 3's mode the 13th, `$2C` bit 5 the
  48th, the DAC's data and enable the 64th; the key the 12th before, a slot less for each channel after the first, as
  D-24 had it. A sample is made once its 64th edge has passed (`TAKE_LAG`, 2,720 master clocks), the writes it takes
  applied first. The timers keep their own moment ahead of it and leave CSM's key for the sample it falls in. Both
  edges of the reset line make every sample already due before they act, so that a sample is made under the line as
  it stood at its deadline. The state holds up to 64 waiting writes, version 14; a program that writes faster has its
  oldest taken at once.
- **The trace** (`ym2612.rs`): each channel as the left output pins carry it, 0 where `$B4` takes it off the left and
  -1 where it is then negative, as the board's pins show. The sound is the samples as before, each handed to the
  output as it is made, `TAKE_LAG` (51 µs) after its deadline.
- **The bench**: `mdboard.py`'s `board-chip` writes `LANDINGS` into `board_chip.rs`: for each of the 21 sweeps, its
  writes at the master clocks the board's chip took them, each run's differences from the first, its program's hash
  and its channel from its first sample not at rest; the DAC's on channel 6.
- **Tests** (`voices.rs`): `every_write_is_taken_by_the_boards_sample`, the sweeps on the chip at the board's write
  times; `every_write_is_taken_by_the_boards_sample_on_the_whole_machine`, the same programs, rebuilt as `LANDING`
  builds them and checked by hash, on the whole machine placed as the board is; the chip voices, the LFO voices and the
  timer tests at their new counts. `ym2612.rs`'s tests read the registers once the samples taking their writes are
  made.

### 26.2 Measured (2026-10-06)

| Against the board | Before (§24, §25) | With the pipeline |
|---|---|---|
| The 21 landing sweeps, 40 runs each, on the chip at the board's write times | 227 runs; 2 sweeps in every run | 800 runs; 20 sweeps in every run (`lfo` in none) |
| The same programs on the whole machine placed as the board is | (not run) | the same |
| §24's 120 voices, on the chip; on the whole machine placed as the board is; at Nephrite's placement | 111; 111; 94 | 114; 114; 96 |
| §23's 61 LFO voices, on the chip; placed as the board is; at Nephrite's placement | 60; 60; 16 | 61; 61; 16 |
| §21's 62 voices, the key and CSM sweeps (1,000 runs), on the whole machine | all | all |

The "before" counts of the sweeps are the take-at-once chip given the same data, its trace as before.

- **The crate's tests**: 98 pass, VDPFIFOTesting 122 of 122 and stage 4's picture tests among them. **The
  conformance kit**, `--frames 600`: C1-C15 on §24's 17 images. **WiseMan**'s Nephrite, runner, discovery and
  registration tests pass (31).
- **The corpus at anchors** (944 games): no game moves from §25's, in pictures or RAM at frames 120 and 600. The
  pipeline moves a write's effect on the output by under three samples and changes nothing a program reads: the
  status register, its busy flag and the timers keep their own moments.
- **The frame cost**, best of five runs of 600 frames, the desktop, release build, §25's build and this one measured
  back to back:

  | Game | Mean per frame | At §25 |
  |---|---|---|
  | Sonic the Hedgehog | 1.245 ms | 1.242 ms |
  | Sonic the Hedgehog 2 | 1.148 ms | 1.132 ms |
  | Thunder Force IV | 1.257 ms | 1.252 ms |
  | Streets of Rage 2 | 1.035 ms | 1.027 ms |
  | Phantasy Star IV | 1.320 ms | 1.300 ms |

  All within P1's 1.5 ms; the pipeline costs up to 1.6%.

### 26.3 What it leaves

- **`lfo`**, the LFO switched on just before the key-on: unlike the board's in every run, before the pipeline and
  with it, and at every moment of `$22` tried. A level louder than the board's in many samples from the attack on,
  it is a question for the LFO's model (D-20), not for when the write is taken.
- **Six test-register voices**: `fm-test-lfo-late` (no moment of `$21` bit 1 changes a sample of it), `fm-test-pg0`,
  `fm-test-eg`, `fm-test-ugly`, `fm-test-2c` and `fm-test-2c-dac`, not traced.
- **The other operators and channels**: only S4 of channel 1 was swept, and every operator and channel takes a part
  by the same edge. The key is taken a slot later on each channel (D-24); whether the other registers are is not
  measured, and needs the sweeps run on another operator and channel.
- **D-15**, a status read straight after a data write leaving the register at 0 on the board: open and not built,
  until a console is recorded.

### 26.4 The sources

As §24: the board's pins and Z80 bus under `tb_md`, the sweeps of `mdboard.py`'s `LANDING`. Sauraen's reading of the
die (topic 386, as prose) that an operator's write is held until the register file's turn for it, of which the edges
are the measured form. No RTL file was opened; no emulator's source was read.

### 26.5 Open, and next

- The three items of §26.3, ~~**stage 4's frame offset** (§25.5)~~ (removed at §27), **the test read** and **D-2**'s
  few long RAM waits.
- **Next**, in the plan's order: the multiplexed output and its ladder effect, the YM3438 of model 2 as a setting,
  and the model 1 filter.

## 27. Stage 5, the picture placed against the 68000's start (2026-10-06)

### 27.1 What it built

- **The start** (`genesis.rs`, D-26): as the 68000 starts, the VDP stands 745 master clocks into line 159
  (`START_LINE`, `START`), as the board's does; the line is begun as a line's start begins it, and the picture's
  size taken as its frame's line 0 would have taken it (`size_frame`). The machine's first frame is the rest of the
  board's frame 0, from line 159, so that Nephrite's picture after n frames is the board's frame n-1.
- **The bus refresh from power-on**: its request before the 68000's start is pending at the first access, and the
  placements against the 68000's start count that wait: the bus refresh at 889, the RAM's first request at 1,428,
  the YM2612's first deadline at 924 (`ym2612.rs`). Nephrite's own placement is now the board's, and the voices'
  tests have one placement only; `place_as_the_board` is gone.
- **The 68000's clock**: every bus cycle starts on one of the 68000's clocks from its start.
- **A write held for a full FIFO** ends 24 master clocks after the slot that frees an entry (`FIFO_RELEASE`); a bus
  refresh requested during the hold is done in it, a cartridge read starting within 126 master clocks of the request
  waiting for its end, two clocks at most (`REFRESH_BUSY`, kept in the state, version 15).
- **The control port** is taken 14 master clocks before the 68000's cycle ends: the status and the HV counter read
  there, a write to it taken there (`HV_LATE`); the data port at the cycle's end, as before.
- **The bench**: `mdboard.py port-zbus` (the HV counter or the status read through the Z80's area from the reset
  vector) and `landing-frames` (a picture program's first changed pixel in each of the board's frames).
- **Tests**: every stage 4 test unchanged but MD1536's picture, re-recorded (§27.3); `genesis.rs`'s RAM loops
  counted from the board's first strobe; the voices and the timer poll at the one placement.

### 27.2 Measured (2026-10-06)

| Against the board | Before (§26) | Moved |
|---|---|---|
| The HV program's 5,806 bytes | (not run) | 5,715; the rest in horizontal sync and on lines 229 to 234 (D-26) |
| The status program's 5,800 bytes | (not run) | 5,796 |
| §23's 61 LFO voices on the whole machine | 16 | 61 |
| §24's 120 voices on the whole machine | 96 | 114, as on the chip |
| §21's 62 voices, the key and CSM sweeps, the 21 landing sweeps | all, 20 | all, 20 |
| MD1536 at Nephrite's frames 298 to 302 | (not compared) | the board's frames 297 to 301 to the pixel, up to a colour map |
| The write-landing and transfer sweeps, frame by frame (264 points) | (not compared) | 213 to D-11's tolerance, 168 exactly |

- **The crate's tests**: 97 pass (one fewer, the placement now single), VDPFIFOTesting 122 of 122. **The
  conformance kit**, `--frames 600`: C1-C15 on §24's 17 images. **WiseMan**'s Nephrite, runner, discovery and
  registration tests pass (31), its goldens for the first frame's size among them: the frame the machine starts in
  takes the power-on registers' size, 256 by 224, as before.
- **The corpus at anchors** (944 games, against Genesis Plus GX at frames 120 and 600): 719 games move. Pictures on
  the reference's to the pixel, up to a colour map: 827 to 888 at frame 120 and 773 to 803 at frame 600; of the
  pictures that change, 84 of 100 come nearer at frame 120 and 90 of 158 at frame 600. RAM's median equal bytes:
  65,527 to 65,528 and 65,513.5 to 65,515; of the RAM counts that change, 330 of 518 come nearer at frame 120 and 365
  of 605 at frame 600. No game fails to run. The reference shows a game's picture a frame early against the board
  (§23.4), so a picture counted equal may be a frame from the board's, and the counts are the corpus's movement, not
  a grade.
- **The frame cost**, as §24.2: Sonic the Hedgehog 1.252 ms, its sequel 1.140, Thunder Force IV 1.252, Streets of
  Rage 2 1.028, Phantasy Star IV 1.305; within P1.

### 27.3 The re-baseline

Moving the picture moves every game against its own power-on and against the reference, so the corpus's anchors are
recorded again, and §27.2 gives what moved. Of stage 4's goldens, only MD1536's frame-300 hash
changes: it is now the board's frame 299, checked with the board run to its frame 302 and compared at five frames;
the old hash was Nephrite's frame 300 under the frame offset (D-11). Every other stage 4 test holds its
old expected values and passes: they are sets of what the board shows across its frames, or relative measures, and
the board's frames are where they were. That the per-frame sweeps are 213 of 264 points says the sets are wider than
Nephrite's frame-by-frame agreement, which §27.4 leaves open.

### 27.4 Open, and next

- **The sweeps frame by frame** (D-26): the board's alternation between two slots in `write-landing-10` comes on the
  other frames in Nephrite; the interrupt's entry against the 68000's loop is the first suspect, not traced.
- **The FIFO entering the frame's last line**, **the H counter in horizontal sync and on lines 229 to 234**, **the
  status's horizontal-blank bit**, **mode 4's HV read**, and the refresh's one-clock overlap seen a few times off
  the rule (D-26).
- **D-2**'s long RAM waits, **D-15**, **the test read**, `lfo` and the six test-register voices as in §26.
- **Next**, in the plan's order: the multiplexed output and its ladder effect, the YM3438 of model 2 as a setting,
  and the model 1 filter.

### 27.5 The sources

No RTL file was opened; the board's nets read were those `TB_MEM`, `TB_PICTURE` and `TB_ZBUS` already name. The
68000's manual for DTACK and the cycle's states. No emulator's source was read.


## 28. Stage 5, the output stage (2026-10-06)

### 28.1 What it built

- **The DAC** (`ym2612.rs`, D-27): a channel's turn at each output pin. The YM3438's is linear. The discrete
  YM2612's has the ladder effect: a level of 0 or more a step higher, and a rest a step to the sign's side of the
  middle in the three slots of its turn that do not carry it, and in all four on a side it is panned off.
- **The pins' pulses** (`sound.rs`): the pins carry each channel for a part of its sixth of the sample, so the band has
  each sample at full strength. The step synthesiser still holds each sample, and seven taps on the FM stream at the
  output rate undo the hold's droop (2.1 dB at 20 kHz) to 0.08 dB; the PSG waits three output samples beside it.
- **The output circuits** (`Circuit`): the mix goes through the model's, a digital filter at 48 kHz fitted to the
  analogue magnitude up to 20 kHz. Model 1: a first-order low-pass at 3,216 Hz and a high-pass at 23 Hz. Model 2: a
  second-order low-pass at 5,805 Hz with a Q of 1.16, a first-order one at 4,111 Hz and a high-pass at 16 Hz.
- **The PSG's level**: a channel at full volume is 0.324 of the DAC's swing (it was 0.251).
- **The Model setting** (`v1.rs`, key `model`, read at create): `md.model1`, the default, is the discrete YM2612 and
  model 1's circuit; `md.model2` the YM3438 and model 2's. TMSS, which the plan gives the model 2, joins the setting
  with its boot program.
- **The state**, version 16: the circuit and each side's place in it, and the two streams' waiting samples.
- **The bench**: `mdrecordings.py`, which cuts a recording of the 240p suite's MDFourier sequence by its sync trains
  and measures it against Nephrite's run; `examples/wav` takes pad presses, the model and `NEPHRITE_CIRCUIT=flat`.

### 28.2 Measured (2026-10-06)

Against MDFourier's recordings of consoles, each cut by its own sync trains (3,500 frames between them on all 21 and in
Nephrite), with a gain alone between the recording and Nephrite's finished output:

| Console, by its recording's name | Left side's FM partials, 40 Hz to 16 kHz | Right side's | PSG against FM |
|---|---|---|---|
| Model 1, VA3 (two consoles) | 0.21 and 0.23 dB rms, all within 1 dB | 1.54 dB, 90% within 1 dB | +0.03, -0.07 dB |
| Model 1, VA6 (four) | 0.21 to 0.24 dB, all within 1 dB | 1.52 to 1.54 dB, 89 to 91% | -0.57 to +0.27 dB |
| Model 1, VA6.5 | 0.22 dB, all within 1 dB | 1.51 dB, 92% | -0.20 dB |
| Model 1, VA1 and VA2 (not offered) | 0.43 and 0.53 dB | 1.52 dB | +0.78, +0.07 dB |
| Model 2, VA0, VA1, VA1.8 (two) | 1.69 to 1.83 dB, 88 to 90% within 1 dB | 1.62 to 1.68 dB, 79 to 82% | -0.69 to -2.03 dB |

- **The ladder**: the partials Nephrite has only with it are in five model 1 recordings at -0.5 to +0.5 dB of the
  model's level on the left side, and absent from the YM3438 boards' (D-27).
- **The crate's tests**: 101 pass. **The conformance kit**, `--frames 600`: C1-C15 on §24's 17 images.
  **WiseMan**'s Nephrite, runner, discovery and registration tests pass (31).
- **The corpus at anchors**: no game moves from §27's in pictures or RAM; the step changes only what is heard.
- **The frame cost**, as §24.2: Sonic the Hedgehog 1.258 ms, its sequel 1.150, Thunder Force IV 1.257, Streets of
  Rage 2 1.052, Phantasy Star IV 1.333; within P1.

### 28.3 What the two models speak for

Model 1 is the boards VA3 to VA6.5, seven consoles whose left sides fit one first-order low-pass between 3,165 and
3,320 Hz. VA1 and VA2 are 500 to 600 Hz higher and VA7 is another circuit; none of the three is offered. Model 2 is
the boards whose YM3438 is inside the main chip, VA0 to VA1.8: their amplifier distorts, no filter fits them to
better than 2.4 dB, and the circuit built is the nearest one. The model 2 board VA2.3 carries a discrete YM2612,
ladder and all, behind a clean second-order filter; it and VA3, VA4, the Nomad and the model 3s are measured in D-27
and not offered.

### 28.4 The sources

SpritesMind topic 386 and Kabuto's hardware notes, as prose, their sections named in D-27; the YM3438's data sheet;
MDFourier's recordings of 21 consoles. The ladder's constants are as Eke's post in the topic gives them from Nuked's
reading of the die (*accepted as prose by the tester, D-27's decision of 2026-10-07*), held here to Kabuto's
measured DAC levels and to the recordings; the circuits' corners and the
PSG's level are fitted to the recordings and come from nowhere else. The bench's `MOL_2612`, a model of the discrete
DAC and not the die, referees none of it. The four references' audio was not used. No RTL file was opened; no
emulator's source was read.

### 28.5 Open, and next

- **D-27's open items**: the right side's weaker agreement; the model 1 DAC's levels that stand high; model 2's
  distortion and its PSG's level; the boards not offered.
- ~~**The test register's read side**, the six test-register voices and the `lfo` sweep (§26.3).~~ At §29.
- **Carried forward from D-26**, not chased here: the sweeps' frame alternation, the FIFO at the frame's last line,
  the H counter in horizontal sync and on lines 229 to 234, mode 4's HV read.
- **D-2**'s long RAM waits and **D-15**.
- ~~**Next**: the stage's closing oracle, Nemesis's FM tests against his recording, MDFourier's own comparison and the
  references' audio envelopes.~~ At §29, which closes the stage.

## 29. Stage 5, the test read and the stage's close (2026-10-06)

### 29.1 What it built

- **The test read** (`ym2612.rs`'s `test_read`, D-23): with `$21` bit 6 set, a read of the status port gives a byte
  of what the chip is working on in the slot the read ends in. With `$2C` bit 4 clear it is an operator's fourteen-bit
  output, from the sample's 19th slot edge, a slot each in the order S1 of the six channels, S3, S2, S4; with it set,
  a channel's nine bits from the 50th edge, four slots each in the pins' order. `$21` bit 7 picks the high byte, whose
  bit 7 carries channel 4's S4's phase a bit a slot from the 40th edge and whose bit 6 carries each operator's
  envelope step from the 16th edge (or, with `$21` bit 0 set, a pulse in the 20th slot). A read in a sample not yet
  made runs a copy of the chip on to it.
- **The phase counters' power-on value**: all ones, as the test read shows them; a key-on still starts one from 0.
- **A key-on under `$21` bit 3**: its sample's phase starts from 0, the next ones from one increment (D-23).
- **Each channel's frequency taken a slot after the last channel's**, as the keys are (D-25).
- **The bench**: `mdboard.py`'s `TEST_READS` and `board-reads`, which writes `board_reads.rs` (each program's hash
  and its accesses to the chip with the bytes the board's 68000 read); five sweeps in `LANDING` that clear a test bit
  as a voice sounds, or set bit 1 (`pg-off`, `eg-off`, `ugly-off`, `dac-all-off`, `lfo-test`); `examples/wav` takes
  `NEPHRITE_PAL=1`.

### 29.2 Measured (2026-10-06)

- **The test read**, on the chip given the board's accesses at their master clocks: every one of the 1,536 reads of
  nine programs is the board's (both bytes of both forms, the phase bits, the envelope's steps, the frequency's slot
  by channel), and every read of twelve further runs not kept. Two programs stay open at their counts: the decaying
  operator's high byte one apart in nine reads (1,527), and bit 6 with `$21` bit 0 set in the decaying channel's
  program (956).
- **On the whole machine** the same programs give 1,030 to 1,491 reads of 1,536 as the board did. Their loop stores
  each read to the main RAM, where D-2's long waits are not built, and the whole machine's time through it is the
  first suspect; it is not traced, and the test is held on the chip.
- **The voices**: 115 of the 120 of D-21 to D-23 are the board's to the sample, on the chip and on the whole machine,
  `fm-test-pg0` now among them. The five others change a test bit as the voice sounds, and that bit reaches the
  operators slot by slot, where the unit makes a sample's four at once (D-23): `fm-test-ugly`, `-2c` and `-2c-dac`
  differ in the one or two samples at the change, `fm-test-eg` by an envelope step in alternate runs of twelve samples
  after the clear, and `fm-test-lfo-late` from its first sample under bit 1.
- **The crate's tests**: 102 pass. **The conformance kit**, `--frames 600`: C1-C15 on §24's 17 images. **WiseMan**'s
  Nephrite, runner, discovery and registration tests pass (31). **The corpus at anchors**: every game's pictures and
  RAM are §28's; the step changes only what is heard and the test read, which no game in the corpus uses.

### 29.3 The closing oracle

**Nemesis's FM tests against recordings.** Of his six YM2612 tests, one has a recording: his own of
`ym2612csmmode`, 96 kHz, one channel. Its CSM retrigger comes every 19.396 ms, which is timer A's longest period
(1,024 samples) on a PAL console's clock and not on an NTSC one's (19.229 ms); the recording is of a PAL console. Run
as a PAL model 1 (`NEPHRITE_PAL=1`) against the recording resampled to 48 kHz:

| | The recording | Nephrite, PAL model 1 | Nephrite, PAL model 2 |
|---|---|---|---|
| Retrigger period, median and mean | 19.3958, 19.4009 ms | 19.3958, 19.4008 ms | the same |
| The three largest spectral peaks | 876.0, 878.9, 928.7 Hz | the same | the same |
| One retrigger's envelope, against the recording's | | correlation 0.99996, 0.4% of the peak rms | 0.99993, 1.2% |
| Decay, dB a second, from -6 to -12 and -12 to -18 dB | -1,922, -1,853 | -1,913, -1,842 | -2,019, -2,062 |
| Peak to -10 dB, to -20 dB | 4.48, 9.79 ms | 4.56, 9.88 ms | 4.42, 9.13 ms |
| The level the decay settles at | -28.0 dB | -27.4 dB | -32.4 dB |

The recording's console has the discrete YM2612's ladder: the floor a decaying voice settles on is the ladder's
rest (D-27), which model 1 has and model 2 does not. The decay's rate, the retrigger and the pitch are Nephrite's.

The other five have no recording. All six were run on the board for 19 million of its cycles (0.18 s of sound), and
every channel each sounds on is the board's to the sample from its first sound on the whole machine: `csmmode` 4,075
samples, `detune` 4,079, `FrequencyWriteOrder` 4,077, `ssg-eg` 4,077, and `multiplexing`'s six channels 3,950 to
3,962. Run longer and compared at the place the others share (the whole machine 695 samples ahead of the board's
count): `multiplexing` over 0.4 s, 14,367 to 14,379 samples a channel, every one the board's; `ssg-eg` over 1.2 s,
58,821 of 58,828; `attackandrelease` over 1.1 s, 53,826 of 53,868. The 49 samples apart are a level of one apart in
quiet passages. In `multiplexing` the six key-ons come 2.15 samples apart on the board and 2.0 in Nephrite, so a
channel's first sound falls up to two samples from the board's against its neighbours; the 68000's time through the
test's write routine is not traced.

**MDFourier.** The 240p suite's sequence against MDFourier's recordings of 21 consoles is §28.2: a model 1's left side
is Nephrite's to 0.21-0.24 dB rms, all partials within 1 dB, on all seven boards offered. MDFourier's own program was
not run; the sequence was cut and measured by `mdrecordings.py`.

**The references' audio envelopes**, as corroboration: each test through Genesis Plus GX, PicoDrive, BlastEm and
ClownMDEmu (`probe --wav`), its envelope in 10 ms windows against Nephrite's (NTSC, model 1) at the best of ±1 s of
lag, two to eight seconds in:

| Test | Genesis Plus GX | PicoDrive | BlastEm | ClownMDEmu |
|---|---|---|---|---|
| `csmmode` | 0.27 dB | 7.43 | 0.12 | 0.76 |
| `attackandrelease` | 0.13 | 2.27 | 0.10 | 0.14 |
| `detune` | 0.00 | 0.02, at 2,756.8 Hz against the others' 3,329.6 | 0.08 | 0.02 |
| `FrequencyWriteOrder` | 0.00 | 0.06 | 0.01 | 0.05 |
| `ssg-eg`, its first 800 ms | 19.71 | 2.27 | 17.02 | 1.70 |

Genesis Plus GX and BlastEm agree with Nephrite within 0.3 dB on the CSM, attack and frequency tests and ClownMDEmu
within 0.8; PicoDrive's CSM and detune are its own. On `ssg-eg` the references are 1.7 to 19.7 dB from Nephrite and as far from one another; the board
referees it, and it is Nephrite's. On `multiplexing` three references sound a 13.3 kHz tone for the whole run where
BlastEm and Nephrite fall silent after the key-on burst; the board's channels are Nephrite's sample for sample, so
the tone is the references' rendering of the multiplexed output.

### 29.4 The stage's oracle

The plan's oracle for stage 5 (§3.2, §6) and its scope:

| Criterion | Met? |
|---|---|
| Nemesis's FM tests against recordings | Met for the one test with a recording, `csmmode`, on a PAL model 1. The other five have none; the board grades them, every channel to the sample. |
| MDFourier from the 240p suite | Met on the left side of the seven model 1 boards offered (§28). Exceptions: the right side (1.5 dB rms), model 2 (1.7 to 1.8 dB), MDFourier's own program not run. |
| The references' audio envelopes | Corroborate: within 0.8 dB for three of the four on the four tests where they agree among themselves. |
| The scope: envelope, phase, LFO, SSG-EG, CSM, DAC, timers, busy, ladder effect, PSG, mix, model 1 filter, resampler | Built, each refereed on the board or the recordings. Exceptions: the five test-register voices and the `lfo` sweep, D-15's status read. |
| P1, at most 1.5 ms a frame on the bench games | Met, as §24.2 measures it: Sonic the Hedgehog 1.278 ms, its sequel 1.160, Thunder Force IV 1.260, Streets of Rage 2 1.052, Phantasy Star IV 1.335; the mean 1.217. |

Stage 5 is closed with the exceptions above carried forward.

### 29.5 Carried forward

- **D-15**: a status read straight after a data write leaves the register at 0 on the board; whether a console does
  needs a console.
- **D-26's leftovers**: the sweeps' alternation between frames, the FIFO entering the frame's last line, the H counter
  in horizontal sync and on lines 229 to 234, the status's horizontal-blank bit, mode 4's HV read, and the refresh's
  one-clock overlap seen a few times off the rule.
- **D-27's open items**: the right side's weaker agreement; Kabuto's model 1 DAC levels that stand high; the other
  model 1 boards (VA1, VA2, VA7, not offered); model 2's distortion and its PSG's level.
- **Streets of Rage's loading phase**, 0.09% faster in Nephrite than on the board (§23.4).
- **D-19's short pulses**: a reset held under about 1.8 samples does not always take on the board, and always does in
  Nephrite.
- **D-2**'s long RAM waits, with the test read's programs and `multiplexing`'s key-ons as two more places they may
  show.
- **The test register**: the five voices and the `lfo` sweep (slot-by-slot changes the unit does not make), bit 6
  with `$21` bit 0 set, `$21` bit 7's sense against Sauraen's report, and whether the channel form shows the DAC
  (D-23).
- **An operator's output a little apart from the board's** at some levels: nine reads of a decaying operator (D-23)
  and 49 samples a level of one apart in the quiet passages of `attackandrelease` and `ssg-eg`.

### 29.6 The sources

SpritesMind topic 386, Sauraen's test register posts, as prose. Nemesis's six YM2612 test programs, run as they are,
and his recording of the CSM test. MDFourier's recordings as §28. The references run through the probe, their audio
as corroboration only. Every rule was measured on the board's bus and pins; no RTL file was opened; no emulator's
source was read.

## 30. Stage 6, the cheat codecs (2026-10-06)

The plan's §6 row 6, its first item: the system packs' codecs (§4.5 there).

### 30.1 What it built

- **The Genesis's two formats** in the system pack (`Sys/Systems/Genesis/GenesisCheatFormats.cs`), shared by the
  console, the Sega CD and the 32X: the Game Genie as the explicit codec, a 16-bit ROM patch, and the Action Replay as
  the auto-detect codec, a byte, word or long word poked into `WRAM`, its codes for the cartridge's range taken as ROM
  patches. The formats, their sources and what is argued are `EmuSen_Cheats.md` §9.
- **`ROM_PATCHES` in the core** (`cart.rs`, `v1.rs`): every read of the cartridge port at a patch's address answers
  its byte, with its compare where one is given; the range is `$000000`-`$3FFFFF`, on the Genesis alone. The patches
  are the host's and not in the state.
- **The frontend's two gaps the Genesis met** (`EmuSen_Cheats.md` §8): a ROM patch format wider than a byte, added
  whole by `cheat add`, `cheat gg` and the Active Cheats window, and `.cht` files whose Game Genie codes import as ROM
  patches when the explicit codec is passed, which `cheat import`, `cheat load` and the cheat database window now do.

### 30.2 Measured (2026-10-06)

- The Game Genie's worked example (`SCRA-BJX0`, `$009C76`, `$5478`) and 2,000 random codes round-trip; the Action
  Replay's forms, mirrors, widths and joined codes; the ROM patch reaching the core as two bytes high first.
- The libretro database's 2,094 Genesis files: 36,727 of 38,546 entries load, 24,427 of them as ROM patches, where the
  import loaded 12,300 before.
- **The crate's tests**: 103 pass, a patch answering the cartridge port among them. **WiseMan**'s cheat, `.cht` and
  Nephrite tests: 438 pass.

### 30.3 The gate's evidence, begun

`Nephrite_Plan.md` §7's gates for the Genesis, as each step of the stage adds to them:

| Gate | So far |
|---|---|
| G4, the frontend | The system's cheats decode (§30.2). The kit on games, the battery round trips and the fit audit are this stage's later steps. |

The other gates are assembled at the stage's later steps. G6's handheld half is the tester's, after one install at the
stage's end; G8's reading of the disputes log is a reader's who did not write it.

### 30.4 The sources

"Sega Genesis Game Genie Conversion Method" (after Merlyn LeRoy's postings) and ~~Charles MacDonald's~~ `genhw.txt`,
as prose (*corrected 2026-10-07: `genhw.txt` is Rick McTeague's "Sega Genesis Hardware Internals", revised 1993,
which MacDonald's site hosts; his own notes are `gen-hw.txt`*); Sega Retro's "Action Replay (Mega Drive)"; the libretro database's code strings, read for their forms. No
emulator's source and no decoder program was read.

## 31. Stage 6, the settings (2026-10-06)

The row's second item: the region, the model and the pads (`EmuSen_Settings_Reference.md` §4.90).

### 31.1 What it built

- **Region** (`v1.rs`, key `region`, read at create): from the cartridge's header, or the Americas, Europe, Japan or
  Asia, each its version register's overseas and PAL bits; PAL also sets the frame's rate, which machine info now
  takes from the machine's model rather than the header. **Console model** is read at create too, as both now say.
- **The pads** were chosen by setting since stage 3 (§10); the core now exports `SETTINGS`, so a change reaches a
  running game between frames. Their buttons reach the RetroPad as §4.2 of the plan gives them; no `PadButton` was
  added.
- **The frontend's path for a console only a discovered engine runs**: the system pack names it (Genesis, Sega CD,
  32X), `CoreCatalog.SettingsFor` gives the engine's schema for it, Graphics Settings has its tab after the catalog's,
  a choice is shown by its label and stored by its value, and `CoreFactory` hands a v1 engine the stored values at
  create, which no v1 engine had been given before.

### 31.2 Measured (2026-10-06)

- **The crate's tests**: 104 pass, the region's five choices against the version register and the rate among them.
- **WiseMan**: the Genesis's settings, the graphics window, the Nephrite, Mercury model, descriptor and engine tests
  (33), and the window fit audit with the Genesis tab at 1280×800 and 1920×1200, the VenusRT, factory, discovery and
  registration tests (209), pass.

### 31.3 The gate's evidence

| Gate | So far |
|---|---|
| G4, the frontend | The cheats decode (§30). The fit audit passes with the Genesis tab of Graphics Settings at both sizes. |

## 32. Stage 6, the systems that run, battery files and the EEPROM boards (2026-10-06)

The row's third item, with the second's tabs limited to the systems that run.

### 32.1 Only the systems that run are offered

Nephrite's info marks its Sega CD and 32X in development, a system entry's `development` field (`EmuSen_CoreAPI.md`
§27.4), and the host offers such a system nowhere a player chooses: its extensions are claimed only where development
cores are asked for, and its console has no Graphics Settings tab or firmware page row even then. The Genesis has its
tab; the Sega CD's and the 32X's come when stages 10 and 12 remove the mark, with no frontend change. The kit's C2
lists such a system and fails one that shares an extension with a system the core offers.

### 32.2 Battery files

- **The form.** The battery file is the RAM's bytes as the game sees them: a one-lane RAM's lane alone (Sonic 3's 512
  bytes on the odd addresses of `$200001`-`$2003FF`), a two-lane RAM whole, an EEPROM's memory. The plan's §4.4 left
  the references' form to be checked; measured on 2026-10-06 by the four references' save RAM after 600 frames:

  | Game | Nephrite | Genesis Plus GX | PicoDrive | BlastEm | ClownMDEmu |
  |---|---|---|---|---|---|
  | Sonic the Hedgehog 3, odd-lane RAM | 512 bytes | 65,536, both lanes from `$200000`, the other lane `$FF`; its odd bytes are Nephrite's 512 to the byte | 1,024, both lanes, the other `$00` | 512, one lane | 65,536, both lanes, the other `$00` |
  | Mega Man: The Wily Wars, EEPROM | 128 bytes | 65,536, its first 128 Nephrite's | 8,192, its first 128 Nephrite's | 128, Nephrite's | 65,536 |

  *Superseded the same day by §33: the tester chose Genesis Plus GX's form.* Three references write a one-lane RAM in both lanes and one, BlastEm, as Nephrite does; their contents differ from
  one another where the games' timing does. Nephrite keeps its form, which is BlastEm's, and **reads** the two-lane
  form too (`Cart::load_battery`): a file at least twice the lane's length is taken a byte in two from the RAM's even
  start, on the RAM's lane. So a save brought from Genesis Plus GX, PicoDrive or ClownMDEmu loads; one taken from
  Nephrite to those three does not, which is the cost of the choice, argued: it keeps the file the game's own bytes
  and every Nephrite save as it was. An EEPROM's file is its bytes in every reference.
- **The folder** is `Saves/Genesis` (`EmuSen_Settings_Reference.md` §4.91), a development build's `Saves/MD` copied in.
- **Round trips**: a save RAM's and an EEPROM's bytes written through the space, saved, and read back at the next
  load; a two-lane file read by its lane and written back in Nephrite's form; `--nobattery` reading and writing
  nothing (`GenesisBatteryTests`, and the crate's `a_battery_file_round_trips_and_the_references_two_lane_form_is_read`).

### 32.3 The EEPROM boards

The database (`eeprom.rs`, §10.1 and §11.4) is keyed by the header's serial, and by its checksum where two boards share
one (Codemasters' `00000000`, two boards by checksum, and Micro Machines' first game, with none). Checked against the
corpus's headers on 2026-10-06: the 21 boards are found for 26 images, and of the 20 headers that mark an EEPROM in
`RA`'s fourth byte, the seven not found are §12's (Putter Golf and Accolade's six, `T-119016` to `T-119186`), whose
wiring no document gives; they stay carried forward.

### 32.4 Measured (2026-10-06)

- **The crate's tests**: 105 pass. **The kit**: C1-C15 on §24's 17 images, and its own tests (10), the systems in
  development among them. **WiseMan**: the battery, save library, migration and Nephrite tests (68); the settings,
  discovery, firmware, registration, adapter, graphics window and the three Rust engines' tests (588).

### 32.5 The gate's evidence

| Gate | So far |
|---|---|
| G4, the frontend | The cheats decode (§30). The fit audit passes with the Genesis tab (§31). `.srm` files round-trip, for save RAM and an EEPROM, and the references' two-lane saves load (§32.2). The kit on games is the stage's next step. |

## 33. Stage 6, the battery file in Genesis Plus GX's form (2026-10-06)

*Decided by the tester 2026-10-06:* a save RAM's battery file is the one Genesis Plus GX writes, so that saves move
both ways with RetroArch's usual Genesis core; the compact form is still read; an EEPROM's file stays its bytes, on
which the references agree (§32.2).

### 33.1 What Genesis Plus GX writes, measured

A synthetic cartridge for each layout the corpus's headers declare, its `RA` field set to it, writes a marker byte at
the RAM's first addresses and its last, and Genesis Plus GX's save RAM is read through the probe at frame 30:

| Layout (headers in the corpus) | Range | Its file | Where the markers are |
|---|---|---|---|
| Odd lane (82) | `$200001`-`$203FFF` | 65,536 bytes, the rest `$FF` | offsets 1, 3, 5 and 16,383: the address less `$200000` |
| Odd lane (2) | `$200001`-`$2003FF` | 65,536 | 1, 3, 5 and 1,023 |
| Odd lane (31) | `$200001`-`$20FFFF` | 65,536 | 1, 3, 5 and 65,535 |
| Odd lane from `$300001` (2, to `$303FFF` and `$30FFFF`) | `$300001`-`$303FFF` | 65,536 | 1, 3, 5 and 16,383: the address less `$300000` |
| Odd lane from an even start (Rent A Hero) | `$200000`-`$203FFF` | 65,536 | 0, 2, 4 and 16,383: its writes to even addresses kept |
| Both lanes from an odd start (Tecmo Super Bowl) | `$200001`-`$203FFF` | 65,536 | 1, 2, 3, 5, 16,382 and 16,383 |
| Both lanes from an even start (none in the corpus) | `$200000`-`$203FFF` | 65,536 | 0, 1, 2, 4, 16,382 and 16,383 |

The file is 64 KiB whatever the layout, each byte at its address less the RAM's even start, `$FF` elsewhere. Nephrite
writes that form now (`Cart::battery_file`), longer only for a range past 64 KiB, which no header in the corpus
declares. Rent A Hero is the one layout where the two differ in more than form: its header names the odd lane from an
even start, Genesis Plus GX keeps writes to the even addresses too, and Nephrite, taking the header's lane, keeps the
odd ones alone; its file has the same bytes on the odd offsets and `$FF` on the even. ~~Not chased: the game's own
writes decide whether it matters.~~ *Checked 2026-10-07 (§35.3): the game uses the odd addresses alone, so the header
is right for it and it needs no board entry.* Mahjongg Lover's header names `$FF0000`-`$FFFFFF`, the main RAM, and Sonic & Knuckles' `$20200001`, past the bus; neither is a save.

### 33.2 What it built

- **The file** is made when the save RAM's bytes change (a write by the game, the host, a load or a state), at the
  frame's end or at once, and handed to the host as the battery's bytes; the SRAM space stays the RAM's bytes.
- **Reading**: a file longer than the RAM's bytes is taken by address; a file of their length or less is the compact
  form and is taken in order. Both a save from BlastEm or a development build and PicoDrive's 1,024-byte range load.
- **The tests**: the crate's `a_battery_file_is_genesis_plus_gxs_and_the_other_forms_are_read` and
  `a_two_lane_save_ram_is_filed_by_address`; WiseMan's `GenesisBatteryTests`, the file now in this form and the
  compact one read and written back in it.

## 34. Stage 6, the Genesis shelf (2026-10-06)

The row's fourth item, a frontend change through the system packs (`EmuSen_Settings_Reference.md` §4.92): a console
only a discovered engine runs is a row of `CoreCatalog`'s lists, made from its pack, so the library, big picture and
every window that lists consoles show the Genesis while Nephrite's development mark is shown, and nothing otherwise.
The plan's §8.3 said the library's shelves were unaffected by discovery by design; this step is the change that note
waited for. The Sega CD and the 32X stay off every list until their systems drop the mark (§32.1).

### 34.1 Measured (2026-10-06)

- **WiseMan**: the library shelf, the fit audit with the Genesis's tabs in four windows at both sizes, and the console
  and catalog tests (122); the Mistress, core, DianaSh and Galaxia tests, 8,279 pass and 51 are skipped as before.
- **The crate's tests**: 106 pass. **The kit**: C1-C15 on §24's 17 images and on three battery games (Phantasy Star
  IV, Sonic the Hedgehog 3, Wonder Boy in Monster World), the battery file now 64 KiB.

### 34.2 The gate's evidence

| Gate | So far |
|---|---|
| G4, the frontend | The cheats decode (§30); the battery files round-trip, now in Genesis Plus GX's form (§32.2, §33); the fit audit passes with the Genesis's tabs and shelf at both sizes (§31, §34). The kit on games is the stage's next step. |

## 35. Stage 6, the Genesis pad drawn, the fit audit's gaps and Rent A Hero (2026-10-07)

### 35.1 The pad in Controller Bindings

Asked by the tester: the Genesis tab shows a Genesis controller. The two pads are drawn in LunaP as its other pads are
(its `docs/LunaP.md` §198.12, on the toolkit's branch `genesis-pad`, which merges before this), and the window shows
the one the player's port is set to, its buttons bound through the RetroPad mapping of the plan's §4.2
(`EmuSen_Settings_Reference.md` §4.93).

### 35.2 The fit audit's gaps

The library's sidebar and big picture's system carousel are audited with the Genesis shelf present at 1280×800 and
1920×1200 (§4.93 there), beside §34's four windows; `ControllerBindingsGenesisSix` adds the six-button drawing.

### 35.3 Rent A Hero

Its header declares save RAM on the odd lane from an even start, `$200000`-`$203FFF`, where Genesis Plus GX keeps
writes to both lanes (§33.1). Traced on 2026-10-07 with every access of the cartridge at `$200000`-`$20FFFF` printed:
through the title and its menu the game touches none of it; choosing Continue reads 45 bytes, all at odd addresses, in
three blocks from `$200011`, `$200F91` and `$201F11`, the three saves it looks for. It read no even address and wrote
nothing, there being no save to load and none made. The game's use is the odd lane, as its header says: the header is
right for it, and it needs no board entry. A save made in play was not traced.

### 35.4 Measured (2026-10-07)

- **LunaP**: its controller diagram tests over every layout, 96 pass; its whole suite, 1,559, with the public surface's
  baseline approved for the two new layouts.
- **WiseMan**: the window fit audit (every case, the Genesis's new ones among them), the controller bindings, input
  settings, library screen, pad navigation and settings, Genesis settings, Nephrite, player preferences, accessibility
  and hand-rolled control tests: 288 pass.

## 36. Stage 6, the Genesis offered to players (2026-10-07)

*Decided 2026-10-07: the Genesis is offered to players* (the plan's §7), before the kit on games and the gates' table.

- **What changed.** Nephrite's row of `RustCores.props` no longer carries `InDevelopment`, nor its row of
  `rust-cores.yml` `development: true`, so its sidecar is written without the mark, a normal build discovers it, and a
  publish carries `libnephrite.so` and its sidecar beside the other cores' (`EmuSen_CoreAPI.md` §27.2). A publish for
  another platform takes the prebuilt library from `EmuSenNativePrebuilt` as for the other cores, and without it warns
  that the publish has no engine for the Genesis. The core's info is unchanged: the Sega CD and the 32X stay marked in
  development (§27.4 there), so their extensions are unclaimed and they have no shelf, tab or firmware row.
- **What the player sees** is `EmuSen_Settings_Reference.md` §4.94: the Genesis shelf, its Graphics Settings and
  Controller Bindings tabs, its firmware page row (the optional TMSS boot ROM), and `.md`, `.gen`, `.bin` and `.smd`
  opened by Nephrite, in every build.
- **The tests** that switched development cores on only to reach the Genesis now run with them off: the Genesis's
  settings, battery, pad drawing, shelf and window fit audits. `NephriteTests.A_player_is_offered_the_genesis_and_not_the_sega_cd_or_the_32x`
  replaces the test that the core was not offered. `FirmwareOverviewTests` lists the Genesis's row and keeps its test
  of a core in development on a copy of Nephrite's library with its sidecar rewritten with the mark. The registration
  golden (`EmuSen_CoreAPI.md` §25) is re-recorded; its difference is the Genesis's lines alone (the catalogue's cores,
  release order, filter, extensions and cheat systems, the Genesis's core and shelf rows, and Nephrite discovered in
  each scenario).
- **Measured (2026-10-07)**, with development cores off: the discovery, registration, firmware overview and page,
  Nephrite, Genesis settings and battery, Genesis shelf, Genesis pad and the sidebar and carousel fit audits, 47 pass;
  the window fit audit's ten Genesis cases pass. A run of the kit across the corpus was begun and stopped unread at the
  tester's request; it is not a result, and the kit on games remains owed under G4.

## 37. Stage 6, the references cited and the record's gaps closed (2026-10-07)

### 37.1 G8, decided

*Decided 2026-10-07: the separate clean-room review is skipped for the Genesis* (the plan's §7, G8): neither the
reading of the disputes log by someone who did not write it nor `clonecheck.py` over Nephrite's and Beryl's sources
is run. *In its place the references are cited on the repository's `README.md`*, in the section "References for the
Genesis core": the documents, recordings, test programs, referees and reference emulators by group, with author,
title, year and address where there is one and what each was used for, the method in a paragraph, and links to
`Nephrite_Disputes.md`, `Beryl_M68k.md` §6 and the plan's §1.3 and §2. The other cores' source lists are linked from
it and not repeated.

### 37.2 What a partial reading found

A reader who had not written the entries began G8's reading that day and stopped part way at the decision. Read in
full: `Nephrite_Disputes.md` D-1 to D-27; `Beryl_M68k.md` §5.7 to §6.5; `Beryl_Z80.md`; `Beryl_SN76489.md`;
`Nephrite_OperatorTables.md`; `Nephrite_LfoTables.md`; the plan's §1 to §2.3; `EmuSen_Firmware.md` §0; and this page's
sources subsections. Constants were checked against their entries in `genesis.rs`, the top of `ym2612.rs` and `vdp.rs`.
Not done: the rest of the constants (the filter corners, the unhold taps, the ladder in `Ym2612::turn`, D-12's
blanking, D-6's dot, D-18's increments, the removal of `fm.rs`'s old tables), Beryl's constants, and the clone check,
which was not started.

It found no rule taken from an excluded source, and nine gaps in the record, each closed here as a dated addition
that deletes nothing:

| Gap | Closed in |
|---|---|
| D-9 and §16.2 gave VRAM's lead as 8 pixels where `PATTERN_LEAD` has been 24 since §17's commit | D-9's correction, with what is measured and what is argued of the reason; §16.2's note |
| The search of `md_board.v` and `fc1004.v` was made in a writer's step, which the plan's §1.3 forbids | D-16's acknowledgement; §21.5's note |
| The rows that look like vibrato tables were recorded in §21.5 alone, `Nephrite_LfoTables.md` pointed to D-16 for them, and nothing said who wrote the table | D-20's note; the page's §7 |
| D-27 took the ladder's two constants from a post relaying an excluded implementation's reading, with no decision recorded and no tag on its conclusion | D-27's decision and its conclusion's tags; §28.4's note |
| Tables were read from forum code blocks under a ruling no entry stated; the MAME-quoting block of page 8 was in §19.2 alone | D-18's ruling; §19.2's note |
| D-13's title still read "OPEN." | Its title and a note |
| D-6 and D-12 spoke of D-2 as not taken and still open | A dated note in each |
| `Nephrite_OperatorTables.md` §4 said the OPN2's tables waited on a measurement D-16 had since made | Its §4's pointer, with the board's caveat |
| `Beryl_M68k.md` §6.4's cell stopped at D-14 | The cell |

One more was found while the references were being cited: §30.4 gave `genhw.txt` to Charles MacDonald. It is Rick
McTeague's "Sega Genesis Hardware Internals", which MacDonald's site hosts; §30.4 is corrected and the `README.md`
cites each under its author.

### 37.3 What stands open after it

The clone check has not been run, and the unfinished constants above were not checked by a second reader. Neither
blocks anything since the decision of §37.1; both are on the follow-up list of the gates' table.

No code changed in this step, and no test was run for it.

## 38. Stage 6, the kit on games made quick: the corpus form and the sample (2026-10-07)

The row's last item, the kit on games, was first run one process an image, and a run over the corpus was stopped at
the tester's request (§36): each image repeated the cases whose answer does not depend on the game. The kit now has a
corpus form that runs those once (`EmuSen_CoreAPI.md` §28), and Nephrite a sample to run it on.

### 38.1 The sample

`EmuSen/Cores/Sega/Nephrite - Genesis/kit/genesis-sample.txt`: 64 names of the corpus's images, with what each is
there for, and no image. It is what the kit is run on after a change to Nephrite; the whole corpus is the occasional
run.

```
emusen-core-conform --core libnephrite.so --images kit/genesis-sample.txt --from ~/.cache/emusen/probe/nephrite/games/genesis
```

| Group | Images | Chosen by |
|---|---|---|
| The EEPROM boards | 22 | One for each of `eeprom.rs`'s 21 boards, by the serial or checksum that names it, and Putter Golf, which declares an EEPROM the database does not list |
| Save RAM layouts | 10 | Each form of the header's declaration in the corpus: 1, 2, 4, 16 and 64 KiB on the odd lane; 16 and 64 KiB at `$300001`; Rent A Hero's even start; the type bytes `$E0` and `$20` |
| Mappers, lock-on, a coprocessor | 4 | Super Street Fighter II (the corpus's one image, 4 MiB, so the mapper's banks past 4 MiB are not reached); Sonic & Knuckles alone and combined with Sonic 3; Virtua Racing, whose SVP Nephrite does not have |
| Pads and multitaps | 6 | The header's `6` and `4`, and two four-player games whose headers declare neither |
| One market only | 5 | Europe and Japan, by the letters and by the digit |
| Largest and smallest | 3 | 4 MiB; the Game Genie's 31,973 bytes; 128 KiB |
| The record's own games | 8 | Those §11 to §29 and the disputes log measure with |
| Out of the ordinary | 6 | Two copier images, a 32X cartridge, two images with no header and one whose header is `$FF` |

Several images serve two groups (three EEPROM boards are also 4 MiB; two are one market's). The sample cannot hold
what the corpus does not: a 5 MiB Super Street Fighter II and an 8 MiB image are not in it.

### 38.2 Measured (2026-10-07)

600 frames, sixteen images at once, the release library of this commit.

| | One process an image, as before | The corpus form |
|---|---|---|
| The sample, 64 images | 156.2 s | 37.3 s |
| The corpus, 948 files | about 39 minutes by the sample's rate; not run | 8 min 19 s |

- **The same verdicts**: every one of the sample's images passes or fails in the corpus form as in the old, case by
  case, the evidence equal to the digest (`EmuSen_CoreAPI.md` §28.6).
- **The sample**: the core's cases pass; 62 of 64 images pass. **Frank Thomas Big Hurt Baseball** and **John Madden
  Football 93** fail C7 and C8.
- **The corpus**: the core's cases pass; 941 of 948 files pass. Six games fail C7 and C8, the two above and **FIFA
  Soccer 2000 Gold Edition**, **Mickey Mouse - Fantasia**, **Pirates! Gold** and **Worms**; the seventh is
  `delete.me`, a file of one byte that is no game and is refused with words, which is the right answer to it.
- **What the six have in common**: a machine loaded with another's state shows other pictures than the machine the
  state came from over the next 300 frames, and the same sound to the sample (C7's "B, loaded with A's state, did
  not continue as A"; C8's "across a save and load at frame 300 the run differs"). Two machines fed alike without a
  load agree. So something that reaches the picture is not in the state, or is not restored from it. It is not
  traced here; it is a defect, and the gates' table takes it up. *Traced and fixed 2026-10-07, §39: two things the state did not hold. All six pass.*

### 38.3 What was run

`emusen-core-conform`'s own tests, 20 (`EmuSen_CoreAPI.md` §28.5); the old and the new runner on the sample and on
two images each of MoonRT, MercuryRT and VenusRT; the corpus form on the corpus, once. No WiseMan test: nothing of
the frontends or of Nephrite's source changed, and the runner's `--sidecar`, which the build and `CoreDiscoveryTests`
use, writes the bytes it wrote.

## 39. Stage 6, two things a state did not hold (2026-10-07)

§38's corpus run found six games in which a machine loaded with another's state drew other pictures than the
machine the state came from. Players use save states and rewind daily, so it was fixed before the gates' table.

### 39.1 Found

`examples/statediff.rs` runs an image to a frame, saves, loads the state into a second machine, runs both on with the
same pads, and prints the first frame whose picture or state differs and the state's fields that differ
(`NEPHRITE_KIT_PADS=1` presses pad 1's buttons in turn as the kit does). Two causes, **measured**:

- **The picture's size.** A frame runs from line 0 to line 0, and a state is taken at a frame's end, after the step
  that started the next frame. A write to the VDP in that step's remaining clocks opens line 0, and line 0 is where
  the picture takes its size from the registers. The size was not in the state. A machine that loaded such a state
  kept its own: 256 × 224, from its power-on registers, where the machine saved from had 320 × 224. Its next frame was
  then cut to 256 pixels a row, and the frame after was right again. Worms, FIFA Soccer 2000 Gold Edition and Mickey
  Mouse - Fantasia at frame 300, each with line 0 open: the state equal in both machines frame after frame, the
  first frame's picture differing on 45 to 152 rows. Pirates! Gold and John Madden Football 93 are the same case
  under the kit's input.
- **An EEPROM transfer under way.** The board's bytes were in the state (its `SRAM` space), and the transfer was not:
  the step of the protocol, the two lines, the byte being shifted and its bits, the bytes taken, the address and the
  read and acknowledge flags. Frank Thomas Big Hurt Baseball reads its EEPROM across the frame the kit saves at, so
  the loaded machine read other bits, and one frame later its RAM, its 68000's registers and its EEPROM's bytes all
  differed from the machine it came from.

Each cause was shown alone with a library built without the other's fix: without the size, five of the six fail C7
and C8 and Frank Thomas passes; without the EEPROM's transfer, Frank Thomas alone fails.

### 39.2 Fixed

- **`FrameSize`**, the size line 0 gave the frame, after `OpenLineWindow`, and applied on load. It is taken whether or
  not the frame is drawn, so a state stays independent of drawing, which the kit's C8 checks. The open line's pixels
  are still not kept: a loaded state draws them again from itself, as before. *Argued, not shown:* a CRAM dot that
  fell on the open line before the save is not drawn again, since the dot is the write's and not the state's. A state
  at a frame's end with a CRAM write in line 0's first pixels would show it, for one row of one frame.
- **`EepromTransfer`**, eleven words after `MapperPages`, written only for a cartridge with an EEPROM board, since the
  layout is the machine's.
- **The state is version 17.** ~~A version 16 state is refused with words, as every older one is.~~ A version 16
  state loads, §39.4.

**Pinned by** `state.rs`'s `a_loaded_state_keeps_the_size_line_0_gave_the_frame` (line 0 opened in H40, the width then
set to H32; it fails without the size applied) and `a_loaded_state_keeps_an_eeprom_transfer_where_it_stood` (a start
and three bits of a command; it fails without the transfer applied), and `eeprom.rs`'s
`a_transfer_taken_into_another_board_goes_on_as_it_would_have` (a board given the bytes alone reads other bytes);
`the_layout_is_pinned` holds the new layout.

### 39.3 Measured (2026-10-07)

- **The kit's corpus form**, 600 frames: the six games, 6 of 6 pass; the sample, the core's cases and 64 of 64
  images pass, where Frank Thomas Big Hurt Baseball and John Madden Football 93 failed before.
- **The crate's tests** in the state's blast radius: `state.rs`, `eeprom.rs` and `v1.rs`'s, 23 pass.

### 39.4 Version 16 states still load

The handheld's build, the one the Genesis was first offered with (§36), writes version 16, and the tester's resume
and save states there are version 16. Refusing them would lose a player's place in every game. A version 16 state is
read in its own layout, which is version 17's without `FrameSize` and `EepromTransfer`, and the two are supplied:

- **The picture's size** is taken from the state's VDP registers, as line 0 would have read them (`size_frame`).
- **An EEPROM board** is left at rest, its lines released, as at power-on (`Eeprom::IDLE`).

**The known cost of reading an older state**, accepted rather than refusing the state: where the state was taken with
line 0 opened under other registers than those it holds (a width or a height changed in that step's remaining
clocks), the first frame after the load is drawn at the registers' size and not the frame's, one frame, the cause of
five of §39.1's six; and where it was taken in the middle of an EEPROM transfer, that transfer is lost and the game
reads what an EEPROM that was not spoken to answers, once, until it starts the next. Nothing else differs from a
version 17 load: the two fields are all 17 added.

**Which versions a player can have.** The Genesis reached players at version 16 (f6ad8ba0, 2026-10-07); every
version before it was in builds that marked the core in development and offered it to no player (§36). So 16 is the
only older version a player's state can carry, and `OLDEST_STATE_VERSION` is 16. Versions 15 and older, and any newer
than 17, are refused with "a state version this core does not read", the machine unchanged.

**Measured (2026-10-07)**: version 16 states written by the version 16 build itself (Worms, Frank Thomas Big Hurt
Baseball, Sonic the Hedgehog and Phantasy Star IV, each at frame 300 under the kit's input) load in this build, and
its next three frames are the version 16 build's, to the pixel, in all four. Frank Thomas's state was taken in the
middle of an EEPROM transfer by §39.1's account; its pictures over those three frames are unchanged, and its RAM was
not compared.

**Pinned by** `state.rs`'s `a_version_16_state_loads_and_runs` (a state with the two fields taken out and the version
set to 16: the size from the registers, the picture the saving machine's, an EEPROM board at rest with its bytes kept)
and `a_state_of_another_version_is_refused` (15, 2 and 18, with the ABI's words, the machine unchanged).

## 40. Stage 6's gates, as the follow-up list (2026-10-07)

`Nephrite_Plan.md` §7's gates for the Genesis. The Genesis is offered to players (§36), so none of these blocks
anything; what is not met is the follow-up list, §40.2.

### 40.1 The gates

| Gate | Status | The evidence | What is missing |
|---|---|---|---|
| **G1**, the CPU suites | Met, with named exceptions | SingleStepTests' 68000 suite: 310,649 of 317,500, every miss in a disputes entry on which both referees agree with the processor (D-2, D-3, D-4, D-15; `Beryl_M68k.md` §6.5). Its Z80 suite: 1,604,000 of 1,604,000. ZEXDOC and ZEXALL: 67 of 67 each (`Beryl_Z80.md` §6.1). TomHarte's 680x0 suite: 794,139 of 1,000,060, every miss in an entry | D-9, open: 3,609 divisions whose N and Z after an overflow are in dispute, for a test program on a 68000 |
| **G2**, the self-grading programs | Met | VDPFIFOTesting 122 of 122 (§17.3); the BCD verifier, the opcode sizes and the illegal-instruction test (§12); the memory test's thirteen rows (§17.2) | Nothing for the Genesis; mcd-verificator is the Sega CD's |
| **G3**, the pictures | Met, with named exceptions | The test programs against the references up to a colour map and against the board where the two differ (§14.3, §15.4, §16.3, §17.3); MD1536 the board's to the pixel (§27.2); Nemesis's sprite-masking test as on his console | `512PAL`'s 9,700 pixels, the display's reads in the 128 KiB mode (D-10); mode 4's left column under a fine scroll (D-7); single writes with the display off 2 to 4 pixels from the board's (D-11); VRAM's fetch lead and the display bit's return, measured in part (D-9) |
| **G4**, the frontend | Met | The kit's C1-C15 on the corpus, 941 of 948 files (§38.2), the six state failures since fixed and passing (§39), the seventh a one-byte file refused with words; the sample, 64 of 64 (§39.3). Battery files round-trip in Genesis Plus GX's form, save RAM and EEPROM (§32, §33). The fit audit with the Genesis's tabs, shelf, pad drawings, sidebar and carousel at both sizes (§31, §34, §35). The cheats decode (§30) | — |
| **G5**, the goldens | Not met | The corpus at two anchors, frames 120 and 600, against Genesis Plus GX: pictures equal up to a colour map in 827 to 888 games of 944 at frame 120 and 773 to 803 at frame 600; the RAM's median 65,527 and 65,513 bytes of 65,536 (§27.2) | A third anchor; the games that differ, about 150 at frame 600, each examined and logged as a dispute or mended; and a grade that allows for the reference showing a game's picture a frame early against the board (§23.4) |
| **G6**, speed | Met in part | §5.5's desktop budget: 1.217 ms a frame on the bench games, at most 1.335 (§29.4), against 1.5. The tester reports the Genesis plays well on the Legion Go S | The bench at a 33% CPU quota, not measured; a run on the handheld on battery, not confirmed |
| **G7**, the debugger | ~~Not met~~ *Met, with a named exception, since 2026-10-07 (§41-§43)* | ~~The kit's C15: `DEBUG` not claimed~~ Breakpoints, single steps, watches, data breakpoints and coverage on both processors, the call stack and the profile on the 68000, each a test in the crate and through DianaSh's generic debugger (`NephriteDebugTests`); the kit's C15 on the sample and on eight games | ~~All of it~~ The Z80's call stack: the ABI's call stack is processor 0's, and the host keeps one; the Z80's calls are not tracked |
| **G8**, the clean room | Decided otherwise | *Decided 2026-10-07:* the separate review is skipped and the references are cited on the repository's `README.md` (§37) | The clone check was not run, so the code's freedom from emulators' identifiers has no mechanical test; the rest of the second reader's constant check (§37.2) |
| **G9**, firmware | Met, with a named exception | Every game runs with no firmware folder: the TMSS boot ROM is optional and its replacement `none`, Nephrite running as a console without TMSS (§2, `v1.rs`), which machine info and the firmware page name | With the player's `bios_MD.bin`, the same game does not yet run on it: `v1.rs` takes the file and machine info names it as the player's, but its boot program is not run |

### 40.2 The follow-up list

From the gates and the tester's requests, in the order they would be taken:

1. ~~**G7, the debugger**: stage 7, next in the plan's order.~~ Done 2026-10-07 (§43), but for the Z80's call stack.
2. **The multitaps (Team Player, EA 4-Way Play) and the J-Cart's two ports**, at high priority (*added 2026-10-07;
   the two adapters are in the core since §44 and their players in the frontend since §45, where the games read with
   four pads are named; the J-Cart since §48*). The tester has asked for players two to four across the platform. Not
   a gate; it ranks with the first.
3. **G5, the goldens**: a third anchor, and the differing games examined one by one.
4. **G6**: the bench at a 33% quota, and a battery run on the handheld.
5. **G9**: the TMSS boot program run from the player's image.
6. **G8**: the clone check, if it is ever wanted, and the constants not yet read by a second reader.
7. **D-9**, for a 68000 test program (G1); **D-10**, **D-7**, **D-11**'s display-off residual and **D-9**'s VRAM lead
   (G3).

Not gates, carried from the stages and still open:

- **Super Street Fighter II's mapper above 4 MiB.** The corpus's one image of it is 4 MiB, so no test reaches the
  banks the mapper adds, and the sample cannot hold what the corpus does not (§38.1).
- **D-15**: whether a status read straight after a data write leaves the register at 0 on a console, as on the
  board. It needs a console, and it is not implemented until one answers.
- **§29.5's list**: D-26's leftovers (the sweeps' alternation between frames, the FIFO entering the frame's last
  line, the H counter in horizontal sync and on lines 229 to 234, the status's horizontal-blank bit, mode 4's HV
  read, the refresh's one-clock overlap); D-27's open items (the right side's weaker agreement, Kabuto's model 1 DAC
  levels, the model 1 boards not offered, model 2's distortion and its PSG's level); Streets of Rage's loading phase,
  0.09% fast; D-19's short reset pulses; D-2's long RAM waits; the test register's five voices, the `lfo` sweep and
  its other bits; an operator's output a level apart from the board's at some levels.
- **From stage 3** (§12, §17.3): the seven EEPROM boards no document wires (Putter Golf and Accolade's six); Barver
  Battle Saga's protection device; the Z80 window's alignment.
- **A CRAM dot on line 0 when a state is taken** (§39.2), argued and not shown to matter.
- **From the four-player adapters** (§45.5, §46.1): the 4 Way Play's first pad for a game not made for it, a choice
  pending a console; the Team Player's lines where no document speaks.
- **Hotaru and a port count that changes while a game runs** (`EmuSen_Input.md` §8.11). It wants a WiseMan test that
  the harness cannot yet express, Hotaru's `GameWindow` hosted headless with simulated pads as `MultiplayerTests`
  hosts Mistress's window: `HotaruMultiplayerTests.Four_pads_play_a_genesis_team_player_set_while_the_game_runs`. The
  change is one call to `PortRouter.Resize` in `GameWindow.PollGamepad` once that test exists to fail first.
- **Games that do not reach a moving picture where the reference does** (§46.3): ~~Fatal Rewind (and The Killing
  Game Show, the same program), Time Killers, The Smurfs 2, Superman,~~ *found and fixed, §47;* the unlicensed Mahjongg
  Lover and Pocket Monsters, beside Barver Battle Saga above, after the J-Cart. ~~Whac-A-Critter~~ moves since §47.
- ~~**Three cartridges that look for a save RAM their header omits and run alike without it** (§46.2): Summer
  Challenge, Winter Challenge and Test Drive II.~~ *Given it on Sega Retro's word, §47.5.*
- **The HV counter read in an interrupt handler** stands 3 to 5 counts before the board's in `mdboard.py interrupts`
  (`Nephrite_Disputes.md` D-28); not examined.
- **The board locks up on a read of `$A14000`, `$B00000` and the PSG's `$C00010`** (§47.2, D-29); Nephrite gives the
  open bus there. Not examined against a console.
- **From the J-Cart** (§48.5): Pete Sampras Tennis, the first J-Cart, is not in the corpus and not known by its header;
  Micro Machines 96 and Super Skidmarks are J-Carts by their programs and Sega Retro but no reference runs their images
  as such, so they were checked in Nephrite alone; the J-Cart word's bit 7 is the reference's, argued.

## 41. Stage 7, step 1: the processors, their registers, their buses and their code (2026-10-07)

Stage 7 is the debugger (`Nephrite_Plan.md` §6, row 7; G7). It is taken in three steps: this one, which shows the two
processors without running anything; the observed frame, with breakpoints and stepping on both; and the call stack,
watches, coverage and the profile, each a test. This step claims `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`; `DEBUG`
comes with the next.

### 41.1 What it built

- **Two processors in machine info** (`src/debugger.rs`), named as the plan's §4.3 named them: `M68K`, processor 0,
  24 bits of address, and `Z80`, processor 1, 16 bits, each with its bus as `code_space`.
- **Their registers** (`DEBUG_REGISTERS`), in their lists' order. The 68000's: D0-D7, A0-A7 (A7 the stack pointer of
  the mode in use), PC, SR, USP and SSP, so both stack pointers show whichever mode the processor is in. The Z80's: AF,
  BC, DE, HL and their alternates, IX, IY, SP, PC, I, R, WZ, the interrupt mode and the two flip-flops.
- **The two buses as spaces**, at the ids the plan reserved for them so that no id moves: `M68KBUS` (0, 16 MiB) and
  `Z80BUS` (1, 64 KiB), both read-only. A debugger reads them without disturbing the machine: the cartridge as the
  port gives it (its mapper, its save RAM or EEPROM, its patches), the 68000's RAM in its mirrors, the Z80's RAM in
  its mirror and, through the Z80's bank window, the 68000's space. What a read would change is not read: the VDP's
  ports, the I/O ports, the YM2612, the bank register, the PSG and the bus arbiter read as `$FF` here, as do unmapped
  addresses. The buses hold nothing of their own, so they are not in the state; the memories they show are. *Since 2026-10-07
  both are flagged `view` (`EmuSen_CoreAPI.md` §29), and a host that copies every memory leaves them out.*
- **Disassembly** (`DEBUG_DISASSEMBLE`) by Beryl's two disassemblers: the instruction set is the space's where the
  space is one processor's (`M68KBUS` the 68000's, `Z80BUS` and `Z80RAM` the Z80's), else the processor's asked. A
  68000 instruction starts at an even address.
- **Each processor's number and the space its stores are reported in** are set on both (0 and `M68KBUS`, 1 and
  `Z80BUS`), for the next step's hooks.
- **Machine info's `state.loads_from`** now lists 16 and 17, as §39.4's load does; it listed 17 alone.

### 41.2 Measured (2026-10-07)

- **The crate's tests** in the step's reach: `debugger.rs`'s three (each processor's registers against the processor,
  the buses read as each processor sees them with the VDP's latches unchanged by a read, and each processor's code in
  its own set from its bus, its RAM and the ROM), `v1.rs`'s, among them
  `the_processors_and_their_buses_reach_the_host`, and `state.rs`'s: 20 and 12 pass.
- **The kit**, corpus form on the sample: the core's cases pass (C2 lists the two new capabilities, C5 finds both
  answering) and 64 of 64 images pass, C14 finding nine spaces and two processors where it found seven and none.
- **WiseMan**: `NephriteTests`, the registration golden and the Genesis battery tests, 19 pass. Two of `NephriteTests`
  were updated: the library's capabilities and the generic runner's spaces now include the two buses.

### 41.3 Next

Step 2: `DEBUG`, the observed frame. Each processor's step asked before it runs, breakpoints on both (the 68000's in
the shared hooks, the Z80's the core's own), stepping by `EACH`, and the stop reporting which processor stopped, with
the kit's C15 (every table armed gives the plain run's digests).

## 42. Stage 7, step 2: the observed frame, breakpoints and stepping on both processors (2026-10-07)

This step claims `DEBUG`. A debugger can now stop either processor in front of an instruction, step the 68000, and
read the logs and coverage the shared hooks keep.

### 42.1 What it built

- **One frame loop, two ways.** `Genesis::run_frame` and the observed `run_frame_observed` share the 68000's turn and
  the Z80's catch-up, each generic over an `Observer`: the plain frame passes `Unobserved`, which compiles to the loop
  as it was. The observed loop asks before each instruction and reports what it ran.
- **The order of a turn**, kept from the plain loop: the Z80 is caught up to the 68000, the frame's end is taken if it
  came, and only then is the 68000 asked about and stepped. The 68000 is asked only before a turn that runs an
  instruction: not during a transfer's hold, a halt or a STOP.
- **The 68000** (processor 0) is asked through the shared hooks: its breakpoints, `EACH`, the depth, a data
  breakpoint's store and a full log. Its steps, stores and calls go to the hooks as it runs (`MainWatch`): coverage over
  24 bits, the profile, the writes log and the call stack.
- **The Z80** (processor 1) is asked against its own breakpoints, which the host sets with processor 1 and the core
  keeps, as the ABI's §6.14 has it for a second processor. Its steps go to its coverage (16 bits) and its stores to the
  writes log, in `Z80BUS`; its calls do not enter the call stack, which is the 68000's (`Z80Watch`). `EACH` and the
  depth are the 68000's: a step is the 68000's step.
- **A stop leaves the frame open**, `run_frame_debug`'s next call going on from it; the stop names its processor, and
  the ABI reports that processor's PC. A stop on the Z80 leaves it behind the 68000, part-way through its catch-up, and
  the resumed frame finishes the catch-up before the 68000's next instruction, as the plain frame would have.
  `run::UNCHECKED` runs the stopped processor's next instruction unasked.
- **A store's data breakpoint** stops in front of the 68000's next instruction, whichever processor stored.
- **A state loaded while a frame is open** closes it: the next frame begins as a new one.

### 42.2 Measured (2026-10-07)

- **The crate's tests**, all 120, the frame loop being in every test's path. Five are new in `debugger.rs`: every table
  armed with nothing to hit gives the plain frames' state and picture, with both processors' coverage and the 68000's
  calls recorded; a 68000 breakpoint stops in front of its instruction and an unchecked resume runs one pass to it
  again; a Z80 breakpoint likewise, on processor 1, with the Z80 behind the 68000; a frame stopped more than a hundred
  times on both processors and resumed is the plain frame, state and picture (it fails with the 68000 stepped before
  the stopped Z80 is caught up); `EACH` stops in front of each 68000 instruction in order.
- **The plain frame's cost**, best of five runs of 600 frames beside the build before this step: Sonic the Hedgehog
  1.287 ms (1.288), its sequel 1.177 (1.172), Thunder Force IV 1.273 (1.275), Streets of Rage 2 1.070 (1.072),
  Phantasy Star IV 1.348 (1.348). Unchanged, and within P1.
- **The kit**: the corpus form on the sample, the core's cases with C15 passing and 64 of 64 images; and the
  single-image form, all fifteen, on Sonic the Hedgehog, Streets of Rage 2, Thunder Force IV, Phantasy Star IV,
  Frank Thomas Big Hurt Baseball, Sonic & Knuckles with Sonic 3, Super Street Fighter II and Mickey Mania, C15's armed
  run equal to the plain one on each.
- **WiseMan**: `NephriteTests` (the library's capabilities now with `DEBUG`) and the registration golden, 14 pass.

### 42.3 Next

Step 3: the call stack (`DEBUG_STACK`), watches and data breakpoints, coverage and the profile, each claim a test, and
the generic debugger in DianaSh shown working on a Genesis game, which is what G7 asks.

## 43. Stage 7, step 3: the call stack, watches, coverage, the profile and the Z80's steps; stage 7 closed (2026-10-07)

### 43.1 What it built

- **Stepping the Z80.** The host steps a processor other than processor 0 with a breakpoint over its whole space
  (`CoreDebugBridge`, as for VenusRT's SPC700), which the Z80's own breakpoints already ask before each instruction it
  runs, its catch-up included. A step on the Z80 therefore stops in front of its next instruction and names processor
  1. A step on the 68000 while the Z80 is the processor stopped runs the Z80's instruction unasked, finishes its
  catch-up, and stops in front of the 68000's next instruction, the order of §42.1. No new flag was needed:
  `EACH` stays the 68000's, as the ABI's §6.14 and the host have it.
- **The call stack** (`DEBUG_STACK`): the 68000's JSR and BSR push a call, its interrupts and exceptions an IRQ frame
  (level 7 an NMI), and RTS, RTR and RTE pop, as Beryl's 68000 reports them; the host pushes its stack down at the
  frame's start. Depth targets give step over and step out. The Z80's calls are not the call stack's.
- **Watches and data breakpoints.** A 68000 store is reported in `M68KBUS` at its bus address and, where it lands in
  the 68000's RAM or the Z80's, in `WRAM` or `Z80RAM` at its offset too, so a watch on any of the four sees it; a Z80
  store in `Z80BUS` and, below `$4000`, in `Z80RAM`. Those four spaces say `reports_stores`. Not reported: stores
  through the VDP's ports (VRAM, CRAM, VSRAM), stores to a cartridge's save RAM, and a Z80 store through its bank
  window, which is reported in `Z80BUS` alone.
- **Coverage** on each processor over its own width, and **the profile**, which charges the 68000's instructions to
  the routine they run in, as §42 recorded them; both now under test.

### 43.2 Measured (2026-10-07)

- **The crate's tests**, all 125. New in `debugger.rs`: a Z80 step stops in front of each of its instructions in
  order on processor 1; a frame stepped through on both processors (more than a thousand 68000 steps, the Z80 stepped
  every fiftieth, its catch-up finished before each 68000 step) is the plain frame, state and picture; the call stack
  follows a JSR, the vertical interrupt and RTE, and a depth target steps out of the routine; watches see each
  processor's stores, with the storing instruction's address, in the RAM and on the bus, and a data breakpoint stops
  in front of the 68000's next instruction; coverage is each processor's, and the profile charges the routine three
  instructions a call.
- **WiseMan**, `NephriteDebugTests`, eight tests through DianaSh's generic debugger (`CoreDebugTarget`, the debug
  bridge and the shell) on a synthetic cartridge whose 68000 loads the Z80's program (INC A, LD ($0100),A, JP 0) and
  starts it, turns the vertical interrupt on, then calls $300 in a loop from $28C, the routine counting in D2 and storing
  it at $FF0020, the interrupt's handler at $400 returning. The tests cover the processors, spaces and
  registers (`cpus`, `regs z80`); a 68000 breakpoint halts and resumes; each processor steps one instruction at a time
  (`step z80 3`); the call stack holds the JSR (`bt`) and a step over runs the routine whole; write watches on `WRAM`
  and `Z80RAM`; coverage per processor; disassembly from each bus (`disasm z80`); every registry armed with nothing to
  hit gives the plain run's state frame for frame. With `NephriteTests` and the registration golden, 22 pass.
- **The kit**: the corpus form on the sample, the core's cases with C5 finding `DEBUG_STACK` answering and C15
  passing, and 64 of 64 images.

### 43.3 Stage 7 closed

The plan's oracle for stage 7, "each claim a test; every table armed gives the plain run's digests (C15)", is met:
each claim of §41-§43 is a test in the crate or in WiseMan, and C15 passes on the sample's reference image and on the
eight games of §42.2. G7 is met but for the Z80's call stack, which the ABI does not carry for a second processor
(§40.1).

*2026-10-18: the frame log reads these spaces big-end first, the Z80's two excepted as a known limit (§66).*

## 44. The four-player adapters, step 1: the Team Player and the 4 Way Play in the core (2026-10-07)

The follow-up list's second item (§40.2), taken before stage 8 at the tester's choice. This step puts the two
adapters in the core, behind the port settings; the frontend's players 3 and 4 and the J-Cart are the next steps.

### 44.1 What it built

- **A port holds a plug** (`io.rs`): a pad, Sega's Team Player in its MULTI setting, or Electronic Arts' 4 Way Play,
  which is port A's plug and takes port B as well. The Team Player's EXTRA setting is the 4 Way Play's protocol, as
  MacDonald found it, so the two are one plug.
- **Eight pads at the most**, a Team Player on each port. The players are port A's pads and then port B's
  (`Io::first_pad`), and machine info's `ports` lists the pads plugged in, each a three- or six-button pad.
- **The Team Player.** TH and TR are the console's. With TH high it rests, D3-D0 `%0011`. TH's fall starts a packet
  and gives `%1111`; with those two reads the peripheral ID is 7. Each change of TR while TH is low asks for the next
  nibble, and TL follows TR when the nibble is ready. The packet: two zero nibbles; each of the four ports' kind, 0 a
  three-button pad and 1 a six-button one; then each pad's nibbles, a pressed button 0: Right Left Down Up, Start A
  C B and, for a six-button pad, Mode X Y Z.
- **The 4 Way Play.** Port B's lines 6 to 4, as the console drives them, choose what port A reads: 0 to 3 a pad, read
  as a pad on the port is; 7 the adapter's mark, D1 and D0 low. The four pads share port A's TH, so a six-button pad
  counts every fall, chosen or not. *Amended by §45.2: line 6 high is the mark whatever lines 5 and 4 are.*
- **The settings** `pad1` and `pad2` gain the adapters' choices (`EmuSen_Settings_Reference.md` §4.101), and
  `SETTING_NOTES` says port 2's choice is not used while port 1 holds the 4 Way Play. Nothing is chosen from the
  cartridge: the header's `4` says an adapter is supported and not which (plutiedev).
- **The state is version 18**: the eight pads' TH and counts where 17 held two, and each tap's place in its packet.

### 44.2 What is argued, not measured

- **Past the packet's end** a Team Player gives `%1111`. No document says what is there; Sega's sample stops at the
  end.
- **TL at rest** is high, and **TL after TH's fall, before any nibble is asked**, follows TR. The documents state TL
  only for a nibble asked. *Retired by §45.2: a game waits for TL to follow TR at rest, so it does there too.*
- **The 4 Way Play's other lines under its mark** are high (`$7C`), and a choice of 4 to 6 gives nothing (`$7F`).
  MacDonald measured the two low bits alone, on a Team Player in EXTRA mode and not on Electronic Arts' own adapter.
  *The second half retired by §45.2: a choice of 4 to 6 gives the mark.*
- **The Team Player's packet is made when it is read**, from the buttons as they stand. A real adapter reads its
  pads as the packet goes; the buttons do not change within a frame here, so nothing shows the difference.
- **The mouse** through a Team Player (kind 2) is not built: Nephrite has no mouse.

### 44.3 Measured (2026-10-07)

- **Sega's I/O check program**, the sample Technical Bulletin 16 gives developers for the adapter, run as it is. Its
  work area at `$FFFD00` is the bulletin's layout. With a Team Player and four six-button pads on connector 1 and a
  pad on connector 2, each player holding other buttons: the connectors' IDs `07 0D`; the four kinds `01 01 01 01`;
  each pad's `ST A C B R L D U` and `MD X Y Z` bytes what was held (`01`/`01`, `12`/`02`, `24`/`04`, `48`/`08`);
  connector 2's pad `80`. Its screen names a "4P TAP" with four "JOYPAD 6bt" and lights the held buttons. With
  three-button pads the kinds are `00`; with the adapter on connector 2 the IDs are `0D 07`. It is Sega's program
  for Sega's device, so it speaks for what a Team Player game expects, the reads it makes; it does not make the
  reads of §44.2.
- **The crate's tests**, all 132 with the roms found: five new in `io.rs` (the Team Player's ID and packet by
  plutiedev's procedure; an adapter on port B after port A's pad; the 4 Way Play's detection and its four pads as
  MacDonald and plutiedev read them; its shared TH), `segas_io_check_program_reads_the_team_player_and_its_pads`,
  `the_pad_settings_plug_in_the_adapters_and_number_the_players`, and the state's.
- **The kit**: the corpus form on the sample, 64 of 64, and the single-image form on Mega Bomberman with
  `pad1=md.teamplayer6` and with `pad1=md.4way3`, all fifteen each.
- **WiseMan**: `NephriteTests` (the library now claims `SETTING_NOTES`), `NephriteDebugTests`, the registration
  golden, `GenesisSettingsTests` (the rows' new choices), the Genesis battery tests, the Genesis pad's hit regions and
  the window fit audit's ten Genesis cases: 43 pass.
- **No game was verified with four pads in this step.** The references' four-pad runs need the probe to give input
  to more than its first port, which is the next step's.

### 44.4 Older states

A version 16 or 17 state holds two pads and no adapter. It loads as before (§39.4), the six other pads and the two
taps at rest, which is what they were: neither version had an adapter. **Measured**: version 17 states written by the
version 17 build, with a six-button pad on port 1 (Worms, Frank Thomas Big Hurt Baseball, Sonic the Hedgehog and
Streets of Rage 2 at frame 300), give that build's next three frames to the pixel; §39.4's version 16 states still
load. Pinned by `state.rs`'s `a_version_17_state_loads_with_the_other_pads_at_rest`; machine info's `loads_from` is
16, 17 and 18.

### 44.5 The sources

Plutiedev's "Sega multitap", "EA multitap" and "Peripheral ID", as prose and tables, their 68000 routines being a
programming guide's examples of the reads. Charles MacDonald's "Sega Genesis I/O Chip and Peripherals" (from the
tech-docs archive of §10, its `gen_io.pdf`): the 4 Way Play, which he worked out from a game's reading routine and a
Team Player in EXTRA mode, and the Team Player's switch. Sega's Genesis Technical Bulletin 16 (1993-04-05) for the
sample program's work area. Sega Retro's pages on the three devices for what they are and which games use them. No
RTL file was opened and no emulator's source read. The board bench cannot referee an adapter: it models the console,
and the adapter is outside it.

### 44.6 Also in this step

`NephriteDebugTests.Each_processor_steps_one_instruction_at_a_time` now runs its cartridge with the vertical
interrupt off (the six words that turn it on are NOPs, the loop where it was): a step that fell on the interrupt would
land in the handler, and the test is of the steps' order, not of the interrupt, which the crate's call-stack test
covers.

## 45. The four-player adapters, step 2: the probe's ports, the players past the second, and the games read with four pads (2026-10-07)

§44 put the two adapters in the core and verified no game with them, because the probe gave input to its first port
alone and no frontend gave a Genesis game a third player. This step is those two things, and the games.

### 45.1 What it built

- **The probe gives input to every port** (`EmuSen_Debugging_Tools_Reference_v5.md` §3.65): `--presson P:F:BTN[:DUR]`
  holds a button on pad P, `--device P=NAME` plugs one of the devices a libretro core lists into port P, and
  `--list-devices` prints the list. A reference's adapter is chosen the way its own frontend would choose it, by the
  core's own name for the device, so nothing of the reference is read to run it.
- **Two programs beside the probe** (`Reference/analysis/`). `fourpads.py` asks of a game, for each of four pads,
  which bytes of the 68000's RAM a held direction moves in the reference, and whether Nephrite's bytes at those
  addresses hold the reference's values with the direction held and without (§45.3). `padprotocol.py` builds a
  cartridge that writes a list of values to the two ports and stores each read, and runs it on both (§45.2). The
  crate's `dump` example takes the port settings and a pad for each press from its environment for them.
- **The players in the frontends.** The platform's player model already went to eight (`PlayerSlots.MaxPlayers`, the
  seats, the bindings per player and the router: `EmuSen_Input.md` §8), so nothing of it was widened; what was missing
  is how many players a Genesis has, which its settings decide. `EmuSen_Settings_Reference.md` §4.102 and
  `EmuSen_Input.md` §8.11 have it: machine info's ports, read again when a port setting changes; a console's pack
  saying each player's pad from the stored settings, for the Controller Bindings window with no game loaded; and the
  router following a port count that changes while a game runs. No `PadButton` was added.
- **The core says when its ports change**: `set_settings` raises `MACHINE_INFO` when the pads plugged in differ, the
  event on which a host reads machine info again (`EmuSen_CoreAPI.md` §6.2, §6.19).

### 45.2 What the references and the games changed in the core

`padprotocol.py`'s cartridge on Genesis Plus GX and on Nephrite, a Team Player with three-button pads, then with
six-button pads, then a 4 Way Play, each pad holding other buttons. Two differences were Nephrite's to change, and a
game showed each to matter:

- **A Team Player's TL follows TR at rest too.** §44.2 argued TL high with TH high. Gauntlet 4 writes TR low with TH
  high and waits for TL to fall before it starts a packet; with TL held high it never read the adapter. Pinned in
  `io.rs`'s Team Player test.
- **The 4 Way Play gives its mark for any choice with port B's line 6 high**, lines 5 and 4 choosing the pad when it
  is low. §44.1 took 7 alone for the mark and gave nothing for 4 to 6; the reference gives the mark for `$4C` as for
  `$7C`, and the games that look for the adapter write either. Pinned in `io.rs`'s 4 Way Play test.

What still differs from the reference, and is left as argued, because a second emulator's answer is not a
measurement of the adapter and no document speaks:

| Where | Reference | Nephrite | Why it is left |
|---|---|---|---|
| Team Player, the first read after power-on, before any write to the port | `$0F` | `$73`, the rest state | The port's data latch at power-on; a game writes before it reads |
| Team Player, nibbles asked past the packet's end | pad 1's first nibble again | `%1111` | §44.2; Sega's sample stops at the end, and so does every game seen |
| Team Player, TR made an input | TR reads 0 | TR reads 1, the pull-up's | MacDonald's I/O document has an input line pulled high |
| 4 Way Play, port B's choosing lines left inputs | a pad on port A | the mark, the lines pulled high | The same pull-ups; see §45.5. *Changed by §46.1: the first pad* |

### 45.3 The games read with four pads

**The method.** Each game from power-on to frame 900 with no input but one pad's, a direction held from frame 200
(pad 1 Up, pad 2 Down, pad 3 Left, pad 4 Right), once for each pad and once with nothing held, in Genesis Plus GX
through the probe and in Nephrite. The bytes of RAM that the held direction moves in the reference are where the
game keeps what it read of that pad. A pad is *exact* when every one of them holds the reference's two values in
Nephrite; *read* when some do, the rest being the game's course, which the two do not run frame for frame; *unread*
when the direction moves nothing in the reference, the game not reading that pad at a title screen; *missing* when it
moves the reference and nothing of it is in Nephrite. A game is counted only when all four pads are exact or read.
What this shows is that the game found the adapter and read four different pads through it, each where the reference
has it read them; it is not four people playing a match, which no headless run is.

**Sega's Team Player, three-button pads, on port 1: 39 games of 46 tried** (images picked from the corpus by title
against Sega Retro's list of the adapter's games).

- *Exact on all four pads (22)*: ATP Tour Championship Tennis, Double Dribble: The Playoff Edition, Dragon: The Bruce
  Lee Story, Head-On Soccer, International Superstar Soccer Deluxe, J. League Pro Striker, J. League Pro Striker 2,
  Mortal Kombat 3, NBA Jam, NBA Jam Tournament Edition, NCAA College Football, NCAA Final Four College Basketball,
  NFL Quarterback Club, NFL Quarterback Club 96, Olympic Summer Games Atlanta 96, Pengo, Slam Dunk: Shikyou
  Gekitotsu!, Tiny Toon Adventures: Acme All-Stars, Ultimate Mortal Kombat 3, Unnecessary Roughness 95, World
  Championship Soccer 2, WWF Raw.
- *All four read, some exact (17)*: Barkley Shut Up and Jam!, Barkley Shut Up and Jam 2, College Football's National
  Championship, College Slam, Gauntlet 4, Joe Montana NFL 95, Mega Bomberman, NBA Action, NBA Action 95, NBA Hang
  Time, NBA Live 98, NFL 98, NFL Prime Time, Pele's World Tournament Soccer, Ultimate Soccer, Wayne Gretzky and the
  NHLPA All-Stars, World Cup USA 94.
- *Not counted (7)*: Blockbuster Competition 2 (NBA Jam and Judge Dredd) and Columns III read one and three of the
  pads at that point, each exact; General Chaos, Hyper Dunk: The Playoff Edition, Micro Machines and NHL 98 read none
  of the four there in the reference either (General Chaos and NHL 98 read the other adapter, below); Madden NFL 98
  does not start in Nephrite (§45.5), and its pads move nothing in the reference under this adapter.

**Electronic Arts' 4 Way Play, three-button pads: 22 games of 101 tried** (the corpus's images whose header carries
Electronic Arts' maker code, T-50, and a few named for the adapter besides; most are games for one or two players,
tried because the header does not say which are not).

- *Exact on all four pads (2)*: MLBPA Baseball, NBA Showdown 94.
- *All four read (20)*: Bill Walsh College Football 95, FIFA International Soccer, FIFA Soccer 95, 96, 97 Gold
  Edition, 98: Road to World Cup and 2000 Gold Edition (an unlicensed image), General Chaos, IMG International Tour
  Tennis, Madden NFL 95, 96 and 97, Mutant League Hockey, NBA Live 95, NHL Hockey 94, NHL 95, 96, 97 and 98, Rugby
  World Cup 95.
- *Not counted (79)*: 11 where no pad moves anything in the reference; 3 that read one or two of the pads; 64 in
  which pad 1 alone moves the reference and nothing of it is in Nephrite (§45.5); and Madden NFL 98, all four pads
  missing because it does not start.

**Through the frontend's path** (`NephritePlayersTests`, WiseMan): the stored port settings, `CoreFactory`, and each
player's buttons given by port as a frontend's router gives them. NBA Jam with a Team Player and NBA Showdown 94 with
a 4 Way Play, four players each holding a direction from frame 200, have at frame 900 the four bytes the reference
has (`$FF02B5` and `$FF801D` on, every second byte: 1, 2, 4, 8). Beside them a cartridge of the test's own that reads
a Team Player as plutiedev's page has it, with four players' buttons found in the four pads' nibbles, and in Mistress
itself (`MultiplayerTests`) the same cartridge with four pads plugged in and the adapter set while the game runs.

The lists are `~/.cache/emusen/probe/nephrite/runs/fourpads-2026-10-07/`; the games are the tester's library and are
not in the repository, so the two game cases run only where `EMUSEN_NEPHRITE_GAMES` names their folder.

### 45.4 Found on the way: a game for Japan and Europe ran as an overseas console

J. League Pro Striker never touched the ports under a Team Player. Its header allows Japan and Europe; `default_model`
called a machine overseas when the header allowed the Americas *or Europe*, and PAL only for Europe alone, so the
game ran as an American console, which its header does not allow, and never reached its reads of the ports. The
plan's §9 (Q6) had already decided the order, the Americas, then Japan, then Europe, and §2 states it for the Sega
CD's BIOS; the cartridge's model had not followed it. `default_model` now does: overseas when the header allows the
Americas or does not allow Japan.

**Measured.** Three images of the corpus's 946 (§46.3) allow Japan and Europe and not the Americas. The bytes of RAM equal to
Genesis Plus GX's at frames 120 and 600 (§10's anchors), and the pixels off the map between the two pictures, before
(the last run over the corpus, 2026-10-06) and after:

| Game | RAM equal, frame 120 | RAM equal, frame 600 | Pixels off the map, 120 / 600 |
|---|---|---|---|
| Ball Jacks | 65,532 → 65,532 | 65,507 → 65,507 | 0 / 0 → 0 / 0 |
| Golden Axe III | 63,533 → 65,515 | 50,457 → 65,513 | 5,330 / 29,798 → 0 / 2,251 |
| J. League Pro Striker | 61,835 → 65,500 | 61,774 → 65,451 | 3,607 / 12,531 → 0 / 0 |

No other image's model changes: a header with the Americas was overseas before and is now, one with Japan alone or
Europe alone likewise. Pinned by `v1.rs`'s region test over eight headers. The Region row's help says the order
(`EmuSen_Settings_Reference.md` §4.90).

### 45.5 What stands open

- **Madden NFL 98 does not start**: a black screen, the 68000 in a loop at `$0462E0` over a table at `$FFEBB0` with
  the interrupt mask at 7, with plain pads as with an adapter. It is not the adapters'; it is on the follow-up list.
  *Fixed by §46.2: its save RAM is not in its header.*
- **A 4 Way Play under a game not made for it.** In 64 of the games tried, pad 1 moves the reference's RAM and not
  Nephrite's. `padprotocol.py` shows the one difference that would do it: with port B's choosing lines left inputs,
  which is how a game that does not know the adapter leaves them, Nephrite pulls them high, the adapter's mark, and
  the reference gives pad 1. It was not traced in one of the games. On the argument of the pull-ups a console would do
  as Nephrite does, and the settings' text says to leave the port on a pad for such a game
  (`EmuSen_Settings_Reference.md` §4.101); but nothing here measured an adapter, and giving pad 1 would be the kinder
  thing for a player who leaves the setting on. Not decided. *Decided 2026-10-07: the first pad (§46.1).*
- **The three Team Player rows of §45.2's table**, argued.
- **Hotaru** does not follow a port count that changes while a game runs; it hears the ports the game started with
  (`EmuSen_Input.md` §8.11).
- **Six-button pads through either adapter** were compared by `padprotocol.py` and by Sega's program (§44.3), not
  game by game.

### 45.6 Measured (2026-10-07)

- **The crate's tests**, all 132 with the roms found: the two `io.rs` tests of §45.2 and the region test of §45.4
  among them.
- **The kit**, the corpus form on the sample: 64 of 64.
- **The probe's tests**, 49: `a_press_is_its_own_pads` and `a_press_and_a_device_name_their_port_from_one` new.
- **WiseMan**: `NephritePlayersTests` (the port settings to each player's pad; four players through a Team Player;
  an adapter set while the game runs; the two games), `PortRouterTests` (ports added and taken away mid-game),
  `MultiplayerTests` (four pads and a Team Player set mid-game in Mistress), `ControllerBindingsDiagramTests` (the
  Genesis tab's five players and each one's pad drawn), the window fit audit's Genesis cases at both sizes, the new
  one a Team Player's five players with the fourth shown, and the tests of what the change touches beside them:
  `ControllerPortTests`, `PlayerBindingsWindowTests`, `InputSettingsWindowTests`, `GenesisSettingsTests`,
  `NephriteTests`, `NephriteDebugTests`, the registration golden, `CoreAdapterTests` and the Genesis battery tests.
- **Two expectations found stale and corrected**: `InputSettingsWindowTests` listed the window's tabs without the
  Genesis's, and `PlayerBindingsWindowTests` the consoles' player counts without it; both had failed since §36. The
  fit audit's three Genesis cases of Controller Bindings had been measuring the SNES tab, the window opening on the
  first console with a game; they now choose the Genesis tab and say so.

## 46. A 4 Way Play under a game not made for it; Madden NFL 98 and the save RAM a header omits; which games reach a moving picture (2026-10-07)

Three things §45.5 left open or brought up, taken before the J-Cart.

### 46.1 The 4 Way Play gives the first pad while its choosing lines are not driven

**Decided 2026-10-07: the first pad.** §45.5 had the adapter's mark there, argued from the port's pull-ups: lines the
console leaves as inputs read high, and three high lines are the choice of the mark. The setting is the console's, so
a player who leaves port 1 on the adapter would lose player 1 in every game not made for it, 64 of the 101 tried; and
the one reference that can be watched gives such a game pad 1. So this is a choice, recorded as one: the behaviour
observed of Genesis Plus GX, against the argument of the pull-ups, pending a console with the adapter on it.

**The rule** (`io.rs`, `Io::device`): port B's lines 6 to 4 choose only while the control register makes all three
outputs; otherwise port A reads the first pad. Port B itself still gives nothing, so player 2 of such a game needs the
row put back to a pad (`EmuSen_Settings_Reference.md` §4.101).

**Measured.** `padprotocol.py`'s cartridge now differs from the reference at four of its 49 steps, where it differed
at nine. Steps 0 to 2 are the data register at power-on, `$7F` here by MacDonald's list of the registers and 0
there: with the lines made outputs and nothing yet written, they are high here. Step 40 is the lines made inputs
again after the mark was chosen: the reference keeps the mark, as though the adapter remembered its last choice, and
Nephrite gives the first pad; no game seen does this. Of the 64 games, all 64 now read pad 1 where the reference has
them read it, 14 of them exact (Buck Rogers was the 64th, and was §46.2's, not the adapter's). The 22 games that read
four pads are unchanged, and Madden NFL 98 makes 23 now that it starts (`fourpads-2026-10-07/4way-2.txt`).

**Pinned** by `io.rs`'s `a_4_way_play_with_its_choosing_lines_undriven_gives_the_first_pad`; by
`NephritePlayersTests.A_4_way_play_answers_the_protocol_cartridge_and_gives_player_1_when_its_choosing_lines_are_not_driven`,
the protocol cartridge through the frontend's path, every step the reference's but the four named; and by three of
the 64 games there, Urban Strike, James Pond 2 and John Madden Football '93, each keeping Up held on player 1 where
the reference keeps it (`$FF46E5`, `$FFB4CD`, `$FFD2CB`: 0 with nothing held, 1 with Up).

### 46.2 Madden NFL 98 did not start: its save RAM is not in its header

**Found.** Nephrite's RAM is the reference's to the byte through frame 23 and leaves it at frame 24, at a table at
`$FFEBB0`: the reference has `$FFFF` and zeroes there, Nephrite numbers that change places from frame to frame. The
68000 is then in the loop at `$0462E0` for good. The routine at `$04627C` builds the table from the odd bytes at
`$204575` to `$2046B5`, read with `MOVEP`, stopping at a byte of `$FF`, and then looks each entry up in the table it
built, in a loop with no other end than finding it. At power-on the game looks for `DJ04` at `$200005`; not finding
it, it clears the odd bytes from `$200001` with `MOVEP.L` and writes its saves' opening structure. Its other routines
walk lists there with `$20FFFF` as their bound. That is a save RAM on the odd bytes of `$200001`-`$20FFFF`, which the
game takes to be there. The header declares none: where `RA` would be it has spaces, and its ROM's end is given as
`$07FFFF` on a 2 MiB image. With nothing declared, Nephrite gave the cartridge what a board without the RAM has
there, the ROM's mirror; the game read program bytes as its saved rosters, and the lookup never ended.

It is the cartridge's board, not a rule of the 68000 or the bus, so the board bench has nothing to say to it.

**What the references do, as black boxes.** A cartridge of the test's own that writes above its ROM and reads back,
with no save RAM declared, in four of them: Genesis Plus GX and PicoDrive give *every* such cartridge of 2 MiB or less
a RAM at `$200000`, on both bytes, whatever its serial; BlastEm gives it the ROM's mirror, as Nephrite did, unless
the serial is one it knows, when it gives a RAM on the odd bytes (it knows Madden NFL 98's, NHL 96's, NHL 98's, Buck
Rogers' and HardBall III's of those below, and not the rest). So the references divide between a rule for all and a
list.

**Decided: a list, by serial** (`media.rs`, `UNDECLARED_SAVE`), as `eeprom.rs` knows its boards. A rule for all would
give a RAM to the cartridges that have none, and a program can tell the two apart by writing there, which is how
these very games look for theirs. The board each is given is the one plutiedev's "Saving progress with SRAM"
describes: 32 KiB on the odd bytes of `$200001`-`$20FFFF`, with a battery, mapped without the register since it lies
above the ROM. A header that declares its save RAM is believed first. No size was measured: a board with a smaller
chip would repeat within the range, and BlastEm's answers repeat at 8, 16 and 32 KiB for different serials.

**Who is on the list, and why.** The corpus was run with a count of the writes above the ROM that nothing takes (a
build made for the survey, not kept), 900 frames each. A cartridge is listed when its header declares no save RAM and
its game either writes its saves' structure there without asking whether anything answers, or loses its way without
it. The bytes of RAM equal to the reference's at frame 900, of 65,536, before and after:

| Serial | Game | Writes above the ROM, undeclared | RAM equal, before → after | Otherwise |
|---|---|---|---|---|
| `T-172196` | Madden NFL 98 | 33,015 odd bytes, `$200001`-`$208003` | 63,656 → 65,524 | one colour → a moving picture |
| `ACLD012` | HardBall III | 1, at `$20FFFF` | 23,426 → 65,431 | a still picture → a moving one |
| `T-50286` | Buck Rogers: Countdown to Doomsday | 2, at `$200005` and `$200007` | 51,261 → 65,517 | 2 pictures of 3 colours → 5 of 10, the reference's |
| `T-172176` | NHL 98 | 47,976 words, `$200000`-`$20FFFE` | 34,509 → 65,493 | |
| `T-172036` | NHL 96 | 10,430 words, `$200000`-`$203FFE` | 57,927 → 65,465 | |
| `T-50086` | PGA Tour Golf | 5,468 words and 56 bytes, `$200003`-`$203EFD` | 64,438 → 65,465 | |
| `T-50216` | Starflight | 4,096 odd bytes, `$201001`-`$203FFF` | 63,465 → 65,490 | |
| `T-172156` | FIFA Soccer 97 Gold Edition | 210 odd bytes, `$200003`-`$204001` | 65,192 → 65,250 | |
| `T-183457` | FIFA Soccer 2000 Gold Edition | the same: FIFA 97's program under a pirate's serial | 65,231 → 65,250 | |
| `T-172046` | College Football USA 96 | 8,193 odd bytes, `$200001`-`$202003` | 65,494 → 65,495 | |

With the RAM, what each leaves in it at frame 900 is what Genesis Plus GX's RAM holds, byte count for byte count,
where both keep the odd bytes (Madden NFL 98 15,247 bytes not `$FF`, HardBall III 30,990, FIFA 97 5,336, College
Football USA 96 4,098). No other image's row of §46.3's survey moved.

**Not listed, and open:** Summer Challenge (`ACLD013`), Winter Challenge (`ACLD007`) and Test Drive II (`ACLD008`)
write one byte at `$200001`, and given a RAM keep 18, 18 and 704 bytes there, Test Drive II's behind a signature.
They ask first, so their programs allow for a cartridge without the chip, and they run alike to the eye either way
(RAM equal 64,999 → 65,328, 65,423 → 65,425 and 65,502 → 65,517 with it). Whether their boards carried it is not
known here; they are left as they were, on the follow-up list.

**Other writes above the ROM the count found**, none of them this cause: `$38FFFE` and `$3FFFFE` in Micro Machines 2,
96 and Military, Pete Sampras Tennis 96 and Super Skidmarks, which is the J-Cart's (the next step); Barkley Shut Up
and Jam 2 at `$230001`, one of §12's EEPROM boards not wired; FIFA Soccer 96 and 98 one byte past their declared 8 KiB
and Jack Nicklaus' Power Challenge Golf past its 1 KiB; the Tecmo Super Bowls on the even bytes of an odd RAM;
Virtua Racing's processor at `$300000`; and the unlicensed Commandos and Harry Potter at `$208002`.

**Pinned** by `media.rs`'s `a_cartridge_known_to_omit_its_save_ram_is_given_the_standard_board` (three of the
serials, a header's own word first, a serial not listed);
`v1.rs`'s `a_cartridge_whose_header_omits_its_save_ram_has_it_by_its_serial` (the bytes on the bus and the battery
file); `GenesisBatteryTests.A_cartridge_whose_header_omits_its_save_ram_keeps_a_battery_file_by_its_serial` through
the frontend; and `A_game_whose_header_omits_its_save_ram_starts`, Madden NFL 98 and HardBall III each reaching a
picture that moves between frames 600 and 900, where `EMUSEN_NEPHRITE_GAMES` names the tester's library.

### 46.3 Which games reach a moving picture

The kit's pass says a core keeps the ABI's rules on an image, not that the game on it starts: Madden NFL 98 passed
all fifteen cases with a black screen. So a second, cheap measure over the corpus.

**The measure** (`Reference/analysis/moving.py`). Each image from power-on with no input, its picture taken every 50
frames from 600 to 900, in Nephrite and in Genesis Plus GX. *Moving*: Nephrite's seven pictures differ among
themselves. *Still, as the reference*: they do not, and the reference's do not either. *Still where the reference
moves*: a game to look at. It does not show that a moving picture is the right one; the last column of its output,
the RAM bytes equal to the reference's at frame 900, is a second look at that.

**The corpus is 946 images**: its folder has 948 files, and two of them, `delete.me` and `systeminfo.txt`, are not
images. The kit's run of §38.2 counted all 948.

| | Before §46.2 | After |
|---|---|---|
| Moving | 825 | **827** |
| Still, as the reference | 108 | 108 |
| Still where the reference moves | 11 | **9** |
| The reference does not run the image | 2 | 2 |

**The nine that do not** (eight games), none of them looked into yet:

| Image | Nephrite at frames 600 to 900 | The reference | RAM equal at 900 |
|---|---|---|---|
| Fatal Rewind (U), and The Killing Game Show (U), the same program | one colour; the 68000's PC is `$FFFFFFFB` | 3 pictures of 3 colours | 65,358 |
| Time Killers (U) | one colour, the display off, a loop at `$02E95C` | 4 pictures of 15 colours | 65,284 |
| The Smurfs 2 (E) | one picture of 14 colours, the 68000 at `$0193F6` | 3 pictures of 30 colours | 35,677 |
| Superman (E) | one picture of 8 colours, a loop at `$0004FC` | 3 pictures of 21 colours | 48,594 |
| Mahjongg Lover (unlicensed) | one picture of 24 colours | 7 pictures | 64,934 |
| Pocket Monsters (unlicensed) | one colour | 7 pictures of 20 colours | 64,860 |
| Whac-A-Critter (unlicensed) | one picture of 7 colours | 7 pictures | 62,681 |
| Barver Battle Saga | one colour | 7 pictures | 16,275: §12's protection device, already open |

**The 108 still in both**: 68 show one unchanging picture of more than one colour in each, a game that waits, every
one within 262 bytes of the reference's RAM. 39 are one colour in both: 16 are 32X images, which neither runs as a
Genesis; 17 are marked bad dumps or hacks; five (Hyper Dunk: The Playoff Edition (E), James Pond 3, Sesame Street
Counting Cafe, Theme Park (U) and Thunder Force III (U)) are one colour through frame 1,900 in Genesis Plus GX,
PicoDrive and BlastEm alike, so it is the corpus's copies of them: the first three have a header checksum `$0100`
above the sum of their bytes, the other two a sum nothing like their header's; and the Pro Action Replay is a cheat
cartridge with nothing under it. One, Knuckles Chaotix, is a 32X image the reference draws two colours of.

**The two the reference does not run**: Blackthorne (32X) and the unlicensed Magic Girl, on which Genesis Plus GX
stops the probe; Nephrite runs both and nothing is compared.

**Moving, and far from the reference's RAM** (under 50,000 bytes equal): Barkley Shut Up and Jam 2 (34,932, the
EEPROM board above), Super Skidmarks (38,415, a J-Cart), and two marked bad dumps, Triple Play Gold and Wayne's
World.

The lists are `~/.cache/emusen/probe/nephrite/runs/moving-2026-10-07/before.tsv` and `after.tsv`.

### 46.4 Measured (2026-10-07)

- **The crate's tests**, all 135 with the roms found: the three new ones of §46.1 and §46.2.
- **The kit**: the corpus form on the sample, 64 of 64, and the single-image form on Madden NFL 98 and HardBall III,
  compliant on each.
- **WiseMan**: `NephritePlayersTests` and `GenesisBatteryTests` with the tester's library named, 17 pass (the protocol
  cartridge, the three games of §46.1, the battery file by serial and the two games of §46.2 among them);
  `NephriteTests` and the registration golden, 14 pass.
- **The two four-pad lists run again** on this build: the Team Player's is unchanged, 39 of 46; the 4 Way Play's is
  23 of 101 with Madden NFL 98, and no game is missing a pad the reference reads.

## 47. Four licensed games that stood still: two interrupt rules, the open bus and a region spelled out (2026-10-07)

§46.3's survey left eight games standing still where Genesis Plus GX moves. The licensed four are taken here, before
the J-Cart. Each was traced to where its RAM leaves the reference's and to what its program waits for; the two whose
cause was a rule of the 68000 or the bus were refereed on the board bench. They are three causes, not four.

### 47.1 Fatal Rewind and The Killing Game Show: two interrupt rules

The two images are one program under two titles. Its RAM is within a few hundred bytes of the reference's, and equal
to the byte from frame 297 to 309, until frame 310; at frame 316 the 68000 halts, its PC `$FFFFFFFB`. The frame interrupt's handler at `$028A94` calls through a pointer
at `$FFE03E`, which the game sets to 0 while it builds a new screen and sets again after it. Nephrite took the frame
interrupt in that window, called address 0 and ran into a double fault. The window is opened by two instructions:

```
0224B0  MOVE.W #$8014,$FFC00004.L    ; the line interrupt on
0224B8  MOVE.W #$8164,$FFC00004.L    ; the frame interrupt on
```

at mask 0, with both interrupts pending. Nephrite took the line's after the first and, after its handler, the frame's
after the second. The board does neither (D-28 in the disputes log, `Beryl_M68k.md` §7):

- **The 68000 samples its interrupt lines before a MOVE's store**, so the request the first write raises is taken
  after the second instruction, not the first; the second's own write is then also behind the sample.
- **The VDP's acknowledge clears the request it is making then**, not the level taken. The line interrupt is taken,
  but by its acknowledge the frame interrupt is enabled, so the frame flag is cleared and the line's request is made
  again: the handler for level 4 runs twice and the frame's not at all. That is the game's way through the window.

**Both are needed**: either alone leaves the game stopping as before (measured, the other reverted). Neither rule
was argued from the game: each was measured on the bench with programs written for it (`mdboard.py interrupts`, eleven
programs; D-28 has the board's logs), and Nephrite's runs of the same programs now agree with the board on every
interrupt's order and level and on the last one's stacked PC and SR. Before, nine of the eleven differed.

### 47.2 The board bench as referee

D-28 and D-29 were run on Nuked-MD's board through `tb_md` as for every earlier entry, about a minute of the board's
time a program. Three places in the 68000's map lock the board when read (`$A14000`, `$B00000`, `$C00010`); Nephrite
does not model a lock there, which is not this step's.

### 47.3 Time Killers: the open bus's low byte

Its RAM leaves the reference's at frame 6 and the game never draws. The 68000 is in a loop that asks for the Z80's
bus and waits:

```
02E954  MOVE.W #$0100,$00A11100.L
02E95C  MOVE.W $00A11100.L,D1
02E962  BTST #0,D1
02E966  BEQ.S $02E95C
```

The bus request's state is bit 8 of that word, not bit 0. Bit 0 is the open bus's: the low byte of the next
instruction, BTST's opcode `$0801`, whose bit 0 is set, so on a console the loop ends at once whatever the Z80 is
doing. MacDonald's notes say the low byte reads zero, which Nephrite followed, and the loop never ended. The bench
reads the word last fetched in full, at `$A11100` and at every other place that answers nothing (D-29). Built
(`Hw::open8`, `read16`), Time Killers starts.

### 47.4 Superman and The Smurfs 2: a region field that spells the word

Superman's header gives its region as `EUROPE`, The Smurfs 2's as `Europe`; `Markets::parse` read the first three
letters as the old form's codes, E and U, so the two ran as American consoles, and both stopped where they check for
a European one. Another World's header also says `EUROPE`, and it ran, a little off the reference (65,429 bytes of RAM
equal at frame 900). The documents (plutiedev's "ROM header", §45's) know the letters and the hex digit, not the word.
The references, as black boxes, divide: of a test cartridge carrying each spelling, Genesis Plus GX runs `EUROPE`,
`Europe`, `EUR` and `eur` as European; PicoDrive as American; BlastEm as European for `Europe` and `Eur` only. Decided:
a field beginning `EUR`, in any case, is Europe alone, which is what the word says and what the games need. `EU`,
`UE` and the rest are read as before. **Measured**: Superman 48,594 → 65,531 bytes equal at frame 900, The Smurfs 2
35,677 → 65,499, Another World 65,429 → 65,535, each now moving as the reference does. No other header in the corpus
begins `EUR`.

### 47.5 The Accolade three: Sega Retro says battery

§46.2 left Summer Challenge (`ACLD013`), Winter Challenge (`ACLD007`) and Test Drive II (`ACLD008`) without the save
RAM they write a byte to, for want of a source saying their boards carried one. Sega Retro's pages for the three give
`savetype` *battery* for the Mega Drive (Winter Challenge and Summer Challenge also a password), fetched 2026-10-07 as
wikitext and kept with the documents (`docs/segaretro/`). The same field says *battery* for all ten games §46.2 listed,
so the source agrees with the measurement where both speak. A retailer's list of Genesis cartridges taking a CR2032
names the three too. They are added to `UNDECLARED_SAVE`. RAM equal at frame 900: Summer Challenge 65,002 → 65,326,
Winter Challenge 65,423 → 65,425, Test Drive II 65,502 → 65,486; all three moved before and move now.

### 47.6 The survey again

`moving.py` over the corpus's 946 images on this build, against §46.3's last run:

| | §46.3 | Now |
|---|---|---|
| Moving | 827 | **833** |
| Still, as the reference | 108 | 108 |
| Still where the reference moves | 9 | **3** |
| The reference does not run the image | 2 | 2 |

The six that now move are the four games and The Killing Game Show, and the unlicensed Whac-A-Critter, whose cause
was not separated. The three left are the unlicensed Mahjongg Lover and Pocket Monsters and Barver Battle Saga, after
the J-Cart. Bytes of RAM equal to the reference's at frame 900 rose in 96 images and fell in 75: the falls are small
(median 2 bytes, the largest 176 in Bill Walsh College Football 95, then 119, 95 and 84), games whose course runs a
frame apart from the reference's; the rises include Streets of Rage (+234) and Mighty Max (+391), and Ninja Gaiden
(+53,522), a bad dump still one colour in both. The lists are `moving-2026-10-07/after-47b.tsv`.

### 47.7 Measured (2026-10-07)

- **Beryl's 68000**: its 21 tests, the new one failing on the old rule (shown); SingleStepTests' suite 310,649 of
  317,500 and TomHarte's 794,139 of 1,000,060, both unchanged.
- **The crate's tests**, all 137 with the roms found: the acknowledge, the open bus and the region's spelling.
- **The bench**: `mdboard.py interrupts` and `open-bus` on the board, and their images on Nephrite before and after.
- **The kit**: the corpus form on the sample, 64 of 64, and the single-image form on the four games, compliant on each.
- **WiseMan**: `NephriteStartTests` (the four games reach a moving picture through the frontend's path), with
  `GenesisBatteryTests`, `NephritePlayersTests` and `NephriteDebugTests`, 29 pass with the tester's library named;
  `NephriteTests` and the registration golden, 14 pass. The Accolade serials were added after that run and are
  covered by the crate's media test and the survey.

## 48. The J-Cart: Codemasters' two extra ports, measured from the games (2026-10-07)

The follow-up list's second item, its last part (§40.2). A J-Cart is a Codemasters cartridge with two pad ports on
it (Sega Retro's "J-Cart": six were released). No prose document says how they are read, so it was measured from the
games, starting from the addresses §46.2's survey of writes above the ROM found: `$38FFFE` in the Micro Machines games
and Super Skidmarks, `$3FFFFE` in Pete Sampras Tennis 96.

### 48.1 Which cartridges

Six images of the corpus touch those addresses, and each program holds the same two routines there: a detection and a
read. The detection writes 0 to the word, reads it and wants bit 6 clear, writes 1 (Pete Sampras Tennis 96 writes
`$FFFF`), reads and wants bit 6 set; it keeps the answer in RAM (Micro Machines 2 at `$FFF92A`, Micro Machines 96 at
`$FFEE94`, Military at `$FFF23E`). The read writes TH low and high and stores the two words. With no J-Cart, Nephrite
answered from the ROM's mirror and every game found none.

The boards are known by their headers (`cart.rs`, `JCARTS`), the serial and, where the serial is shared, the checksum:

| Game | Serial | Checksum | A J-Cart in Genesis Plus GX / PicoDrive / BlastEm |
|---|---|---|---|
| Micro Machines 2 (both images) | `T-120096` | any | yes / yes / yes |
| Micro Machines 96 | `00000000` | `$2C41` | no / no / no |
| Micro Machines Military | `00000000` | `$168B` | yes / yes / no |
| Pete Sampras Tennis 96 | `T-123456` | `$AABC` | yes / yes / no |
| Super Skidmarks | `XXXXXXXX` | `$71AB` | no / no / no |

For the three Micro Machines, the game's own detection flag in each reference's RAM at frame 100; for the other two, a
test program under the image's header reading the word. The references evidently know the boards by lists of their
own, which do not agree.

Micro Machines 96 and Super Skidmarks are kept although no reference agrees: each program has the J-Cart's routines,
and Sega Retro lists both. Psycho Pinball shares Military's checksum and not its serial, and has no J-Cart. Pete Sampras
Tennis (1994), the first J-Cart, is not in the corpus, and its header is not known here.

### 48.2 The word

**Measured from the games.** TH is bit 0 of a word written to the address and drives both pads. The word read holds
the first port's pad in bits 5-0 and TH in bit 6, and the second's in bits 13-8, each pad's six lines as a pad gives
them on a console port (Super Skidmarks takes the low byte for player 3 and the high byte for player 4, `CMPI #3`).
**Bits 15-14 are low**: Micro Machines 2 carries Codemasters' own J-Cart test (`$006186`, reached from a hidden menu;
its strings are "LEGAL LINE", "TESTING JCART", "JCART 1 PASS"), which writes TH high, reads, masks `$C000` and shows
"HIGH POSSIBLE FAILURE!" if either bit is set, "LOW NORMAL" if not.

**The references, as black boxes**, a test program reading the word with each TH under each game's header:

| | TH low, players 3 and 4 holding Up and Left | TH high | Bit 7 | Bit 14 |
|---|---|---|---|---|
| Genesis Plus GX | `$33B2` | `$3BFE` | high | low |
| PicoDrive, BlastEm | `$3333` (no press seen) | `$7F7F` | low | TH |
| Nephrite | `$33B2` | `$3BFE` | high | low |

Genesis Plus GX agrees with the game's test where the test speaks, and PicoDrive and BlastEm do not (their bit 14 is
TH, which the test calls a fault), nor do they hear players 3 and 4 through the probe. **Bit 7** is high as Genesis
Plus GX has it, which no game reads: argued, on the reference that agrees with the program elsewhere. A first build
had it low, and every game's RAM then differed from the reference's at the bytes holding the word.

**Where it answers.** At `$38FFFE` and `$3FFFFE` and their two bytes, the addresses the games use, on every J-Cart
board. Genesis Plus GX and PicoDrive answer across `$380000`-`$3FFFFF`, BlastEm at `$38xxxx` alone; the games' EEPROM
reads at `$380001` (§10) are left the EEPROM's. A byte write reaches TH on the odd address only (argued; the games
write words).

**The players** (`io.rs`, `Io::jcart_first`): the J-Cart's two pads are the two after the console ports' pads, players
3 and 4 with a pad on each port, and three-button pads; with eight already plugged in they are not heard. Machine info
lists them, so a host gives them their players. No state is added: TH is the two pads' own, which the state holds.

### 48.3 The games read with four pads

**The method** is §45.3's, a direction held on each pad of the four, the two console ports with a pad each, Genesis
Plus GX as the reference; and the raw J-Cart words the games keep compared directly.

- **Micro Machines 2** (`Micro Machines II (UE)`, and the `Turbo Tournament` bad dump): **exact on all four pads** at
  frame 300, the direction held from 240; at frame 900 players 3 and 4 exact and 1 and 2 read. Its J-Cart words at
  `$FFF92E` with players 3 and 4 holding Up and Right: `$33B2 $37FE`, the reference's.
- **Micro Machines Military**: all four read at frames 300 and 900, the J-Cart words at `$FFF242` the reference's,
  `$33B2 $37FE`; the one byte that differs for pad 4 is a counter of the game's course.
- **Pete Sampras Tennis 96**: all four read; each pad's own bytes (`$FF7799` for player 3, `$FF77A1` for 4) are the
  reference's, the rest of what differs being the stack's.
- **Micro Machines 96** and **Super Skidmarks**: no reference runs these images as J-Carts, so nothing to compare; in
  Nephrite each finds the J-Cart and keeps players 3's and 4's presses in its J-Cart words (`$FFEE98`, `$FF001C`).

**Through the frontend's path** (`NephritePlayersTests.A_j_cart_game_reads_players_3_and_4`): Micro Machines 2 and
Military loaded by `CoreFactory`, players 3 and 4 given Up and Right by port, the words at frame 300 the reference's.
**In Mistress** (`MultiplayerTests`): four pads in a big-screen session on a cartridge of the test's own with Micro
Machines 2's serial, players 3 and 4 found in its J-Cart words; and the Controller Bindings window, opened then, offers
four players and draws players 3 and 4 as the three-button pad (`EmuSen_Settings_Reference.md` §4.103).

### 48.4 The survey

`moving.py` on the six images: all six move, as before. RAM equal at frame 900 changes by at most 35 bytes; Micro
Machines 96 falls by 35, the reference finding no J-Cart where Nephrite finds one.

### 48.5 What stands open

- Pete Sampras Tennis (1994), and any J-Cart image whose header differs from the corpus's.
- Pete Sampras Tennis 96's second routine (`$001786`), which writes TH in pairs and checks single bits of the word in
  turn; it did not run in these runs, and what it asks of the cartridge is not known.
- The decode beyond the two addresses, and bit 7.

### 48.6 Measured (2026-10-07)

- **The crate's tests**, all 140 with the roms found: the detection by header, the word at both addresses through the
  bus, and machine info's players with and without adapters.
- **The kit**: the sample, 64 of 64, and the single-image form on the five J-Cart titles, compliant on each.
- **WiseMan**: `NephritePlayersTests` (with the two J-Cart games), `MultiplayerTests`, `ControllerBindingsDiagramTests`,
  `PlayerBindingsWindowTests`, `ControllerPortTests`, `InputSettingsWindowTests` and `NephriteStartTests`, 79 pass with
  the tester's library named; the fit audit's Controller Bindings cases, 10; `NephriteTests`, the registration golden,
  `GenesisBatteryTests`, `NephriteDebugTests` and `GenesisSettingsTests`, 34.

## 49. Stages 8 to 10, split into steps before stage 8's code (2026-10-14)

*Decided 2026-10-14:* the Genesis line goes on, stages 8 to 14 (the Sega CD, the SH-2 and the 32X, speed and the
gates), and MercuryRT after it. Stage 8 resumes where it paused on 2026-10-07, at its start. The plan's §6 gives the
Sega CD three stages: the hardware (8, six steps), the rest of the hardware (9, four) and the open BIOS (10, six). This
section divides them into steps, says what each step needs and what it can run with, and puts the questions only the
tester can answer. It was written and committed before stage 8's first line of code.

### 49.1 What there is to run

- **The player's BIOS.** *Added 2026-10-14 by the tester:* a set of the tester's own Sega CD BIOS images (American,
  European and Japanese, models 1 and 2), copied for development to the probe's system folder. It is used only as a
  firmware path, as Diamond's player's BIOS is: given to Nephrite as its declared firmware, and to a referee as its
  system directory; never read, dumped, traced, logged, disassembled or broken into, and nothing made with it is kept
  in the record but results (passes, failures, frames, pictures described in words), so that stage 10's writer sees
  none of it. Runs on it watch Nephrite's own drive, CDC and CD-DA serve a BIOS and real games; mcd-verificator stays
  stage 8's gate. The handheld's `segacd`, `megacd` and `megacdjp` folders hold only a frontend's `systeminfo.txt` (checked
  read-only on 2026-10-14). The library's 18 `.bin` files with "32X" in their names are 32X cartridges, stage 12's.
- **Five Sega CD discs**, added to the library by the tester on 2026-10-14 in `SCD/`, each a `.cue` with its `.bin`
  tracks (114 tracks, 2.6 GB): Chuck Rock II - Son of Chuck (USA), Lunar - Eternal Blue (USA), Mickey Mania - The
  Timeless Adventures of Mickey Mouse (USA), Sonic CD (Europe) and Sonic CD (USA). They are copied to
  `~/.cache/emusen/probe/nephrite/discs/` and read from there, never committed. With no BIOS they cannot start; stage 8
  uses them where it needs no BIOS: each sheet opened through `emusen-disc`, the drive's TOC, subcode and seeks on a
  real layout, the CDC decoding their data tracks (each disc's sector 0 is a check), and CD-DA from their audio tracks;
  and, on the player's BIOS, to watch the drive and the CDC serve real games. They are stages 9's and 10's commercial
  discs.
- **mcd-verificator** V1.02 (krikzz, GPL-3.0, `MEGA-PRO` at `8b72838`, cached under `roms/mcd-verificator/`). Its
  header is a Genesis cartridge's. At start it reads the long at `$400100`: "SEGA" there is the boot ROM of a Sega CD
  beneath a cartridge (mode 1), and anything else means that it is itself the Sega CD's boot ROM at `$000000` (mode 2),
  the first of the two ways its readme gives. The main CPU loads its own 8 KiB sub-CPU program (`sub_bios.s`, not
  Sega's) into PRG-RAM and serves read, write, copy and delay commands through the communication registers, the
  interrupt handlers recording each level in a status word. Its list (`testSTD` in `test_std.c`, the one routine
  `main.c` calls): RAM CART, COLOR CALC, VAR TESTS, IRQ TEST, REG X000, REG X002, REG 2006, REG X00C, REG 8030,
  CDC REGS, PROG RAM, WORD RAM, WRAM PMOD, then CDC INIT, CDC FLAGS, CDC DMA2, CDC DMA3 and CDC DMA1. Each prints "OK"
  or "ERROR: nn" to plane A, a row of 64 words at VRAM `$C000` each holding a character's code, so the verdicts can be
  read from VRAM. The CDC tests first read sector 00:02:00 of the disc in the drive through the drive and the CDC and
  compare its header and the "SEGA" that opens the system area, so they need a Sega CD disc: any disc does. Its readme
  says the results hold on NTSC consoles only. Its other tests (the graphics ASIC, the PCM, the CDD at length) are in
  its source and not in its binary.
- **The 240p Test Suite for the Sega CD**, 1.32 (Artemio Urbina, GPL-2.0, cached under `roms/240p-scd/`): one mode 1
  data track (a 1 MiB `.iso`), a two-second post-gap, and two audio tracks as `.wav` files, with a sheet for each
  market. It boots only through a BIOS, but as a disc it is whole: the system area, a data track, audio, a pause. It
  is the disc in the drive for mcd-verificator's runs, being homebrew whose licence lets a test carry it.
- **Not fetched**: jvisser's mode 1 graphics test (CC0) and viciious's PCM demo (MIT), named at the pause as
  candidates; fabiodl's `megacdTest` is cached as source only, unbuilt, with no licence stated.
- **mcd-verificator's `cd-log/`**, "cdd communication logs from real hardware" by its readme, screened by its file
  names on 2026-10-14. `BI-mcd-tst/` (`bios-start`, `read-sector-bios`, `play-track`, `stop`) and `sonic-bios/`
  (`boot-log`, `boot-to-player-paused` and six more) record a BIOS driving the drive: **not opened**, and closed to
  stage 10's writer (the plan's §1.3). `LL-mcd-tst/` ("low level": `seek`, `toc-commands`, `read+get-pos` and eight
  more) records mcd-verificator's own commands; the first 25 lines of `toc-commands.txt` were opened to confirm it
  (one status and one command per exchange, no BIOS in the path), and the folder is read as a recording of the drive.
  `notes/` holds krikzz's research notes (the drive's commands and statuses, Word RAM's mode bits, timings) and is
  read; its `random notes.txt` includes ten lines of a TOC exchange with a Sonic CD disc whose host it does not name,
  read as the drive's answers and named to stage 10's writer (§49.6).

### 49.2 Mode 2, and how stage 8 runs with no BIOS

A disc's machine is the Genesis in mode 2 (Mega-CD Hardware Manual, "Mega-CD construction" and §1-1, §1-3): the Sega
CD's boot ROM at `$000000`, PRG-RAM's window at `$020000`, Word RAM at `$200000`, the cartridge slot moved to `$400000`
(where a RAM cartridge sits), the gate array's registers at `$A12000`. The boot ROM is the firmware file the core
already names for a disc (`bios_CD_U.bin`, `_E`, `_J`, files 2 to 4). **The tests place mcd-verificator there**, and it
runs as the Sega CD's boot ROM; nothing in the core knows of it, and the same path runs a player's BIOS.

Given the player's BIOS there, a disc boots as on a console, which is how the five discs are run (§49.1). With no
boot ROM no disc starts: the main CPU's vectors read the open bus, and the machine stands with a black picture,
machine info naming the firmware `absent`. **What a player sees, decided by the plan** (§5.6 "until it exists, the Sega
CD runs only with the player's image"; §7, the Sega CD offered after stage 10): nothing. The `mcd` system stays marked
in development (`EmuSen_CoreAPI.md` §27.4) through stages 8 to 10, so no host offers it; a development build (the kit,
WiseMan, the crate's tests) runs a disc with whatever boot ROM it is given. The firmware entry's words change from "no
disc runs, with the image or without it" to say that a disc runs on the player's image and not without one.

Mode 1, a cartridge with the Sega CD beneath it, is not in stage 8 (Q4 below).

### 49.3 Stage 8's steps

| Step | What it builds | Its gate | Its oracle |
|---|---|---|---|
| 8.1 | The attachment as its own module (`src/mcd/`): the second Beryl 68000 at 12.5 MHz on its own bus and clock, an exact ratio of the master clock, brought up to the main CPU at each main access to the attachment; mode 2's map; the gate array's registers on both sides (reset and bus request, LEDs and version, the write protection, PRG-RAM's banks, the H-INT vector, the communication flags, commands and statuses, the stopwatch, the timer, the interrupt mask, the sub CPU's levels 2 and 3); backup RAM; the font registers | mcd-verificator's RAM CART (none present), COLOR CALC, VAR TESTS, IRQ TEST, REG X000, REG 2006, REG X00C, REG 8030, PROG RAM | the Hardware Manual §1-§4; mcd-verificator's source for what each test expects |
| 8.2 | Word RAM: 2M and 1M, the swaps by DMNA and RET from both sides, 1M's dot image at the sub CPU's `$080000`, its bank at `$0C0000`, the main CPU's cell image at `$220000`, the priority modes | REG X002, WORD RAM, WRAM PMOD | the same, with krikzz's mode notes |
| 8.3 | Discs and the drive: WAVE tracks in `emusen-disc`; the disc opened through `emusen-disc` (`BORROW`, sheets, `.iso`, CHD; `SEGADISCSYSTEM` the console's signature); the CDD by its protocol (the host clock, the ten-nibble status and command with their checksum, level 4 at each exchange), its commands, statuses and TOC reads, the Q subcode it reports, seeks and their timing; `MEDIA` and `MEDIA_SEEK` | CDC INIT's drive half; the drive's exchanges against `LL-mcd-tst`'s sequences; the 240p disc's TOC | krikzz's notes and the LL logs; the LC7883 and LC8951 datasheets for the signals; ECMA-130 through `emusen-disc` |
| 8.4 | The CDC (LC8951): its registers, the decoder (header, status, `DECI`'s place in the sector), its 16 KiB buffer, host data to either CPU (`DSR`, `EDT`), DMA to PRG-RAM, Word RAM in both modes and PCM RAM (the RF5C164's memory window and bank; its sound is stage 9's) | CDC REGS, CDC INIT, CDC FLAGS, CDC DMA1, DMA2, DMA3 | Sanyo's LC8951 datasheet; the Hardware Manual §3-2, §4-2 |
| 8.5 | CD-DA into the mix at the fader's volume; the subcode buffer and level 6; the backup RAM cartridge; the battery files (`.brm` the internal RAM, file 0; `.crm` the cartridge, file 1); the Sega CD's state (a new version) with the disc's fingerprint | round trips; the kit's C1-C15 on the 240p disc with mcd-verificator as boot ROM; old states still loading | the Hardware Manual §3-6, §3-9 |
| 8.6 | The gate (the plan's §7 G2 for the Sega CD): mcd-verificator end to end, each test passing or a dispute or a named loss; the cartridge Genesis unchanged (its program tests, the kit on the sample, the cost of a frame); the Sega CD's cost; the record | as stated | — |

**What runs with no BIOS and no commercial disc:** all of stage 8. mcd-verificator in the boot ROM's place, with the
240p disc in the drive, is every step's gate; the five commercial discs add the drive's and the CDC's checks on real
layouts (8.3 to 8.5), and none of them is needed for the gate.

### 49.4 Stage 9's steps

| Step | What it builds | Its oracle | Needs |
|---|---|---|---|
| 9.1 | The graphics ASIC: stamps, the stamp map, the trace vector table, the image buffer, level 1 and its timing (krikzz's note: a 16 KiB read takes 29.6 ms with the ASIC running and 23.6 without) | the Hardware Manual §3-8; homebrew; the five discs on the player's BIOS against Genesis Plus GX (Sonic CD's special stages and Mickey Mania use the ASIC) | the player's BIOS |
| 9.2 | The RF5C164: eight channels, sign-magnitude samples, loops, envelope and pan, its clock | the PCM manual and the RF5C68A datasheet; viciious's demo; references with a BIOS | the same |
| 9.3 | The fader's ramp, de-emphasis, and the mix's levels between the Genesis, the PCM and CD-DA | the Hardware Manual §3-6; references with a BIOS | the same; recordings optional (Q7) |
| 9.4 | The stage's close: the 240p Sega CD suite and discs on the player's BIOS against Genesis Plus GX | as stated | the same |

**With the player's BIOS** (§49.1) the references run the five discs and the 240p suite, Genesis Plus GX given the
BIOS as its system directory, and stage 9 is graded as the plan's §6 says. **With no BIOS**: the ASIC's and the PCM's
arithmetic as unit tests written from the manuals, and mcd-verificator's own ASIC, PCM and CDD tests if a variant of it
is built from its source (Q6).

### 49.5 Stage 10's steps

A fresh writer, under the plan's §1.3 replacement writers' rule: no BIOS image, listing, disassembly or trace, and none
of `cd-log/BI-mcd-tst/` or `cd-log/sonic-bios/`; the Mega-CD BIOS Manual and the public documents are the
specification.

| Step | What it builds |
|---|---|
| 10.1 | The boot on both CPUs (the system area's check, the initial program loaded, the sub CPU's system program, its interrupt dispatch) and a screen with no disc |
| 10.2 | `_CDBIOS`'s drive calls (initialise, play, seek, stop, pause, scan, TOC and status, the fader, the LEDs) over the drive of 8.3 |
| 10.3 | Data reads: `ROMREAD` and its relatives, the CDC's start, stop, read, transfer and acknowledge |
| 10.4 | `_BURAM` in the real BIOS's format, on the internal RAM and the cartridge |
| 10.5 | The main CPU's routines and tables that games use by address |
| 10.6 | The gate: the 240p Sega CD suite, mcd-verificator in mode 1 (it finds a boot ROM at `$400000` by the "SEGA" at `$400100`), games against the same games on the player's BIOS |

It has the five discs and, for its comparisons, the player's BIOS run as a black box (§49.1); its writer sees no run made on that BIOS beyond its results. Mode 1 arrives with it (Q4).

### 49.6 Questions for the tester

1. **Test discs.** *Narrowed 2026-10-14, when the tester added five discs (§49.1):* stage 8 needs no more, and the five
   are stages 9's and 10's commercial discs. *Recommended:* add, when convenient, discs that use what the five may not:
   streaming full-motion video (Sewer Shark, Night Trap, or Snatcher's), the PCM heavily (Silpheed, Popful Mail), and
   one using backup RAM's cartridge; and fetch jvisser's graphics test and viciious's PCM demo for stage 9, their
   licences allowing it.
2. ~~**A BIOS for stage 9's references.**~~ *Answered 2026-10-14:* the tester added their own (§49.1).
3. **mcd-verificator in the BIOS's place for the references.** Genesis Plus GX, PicoDrive and BlastEm refuse a Sega
   CD without a BIOS; given mcd-verificator under a BIOS's file name in the probe's own system folder, they would run
   it as Nephrite does, and each reference's verdicts would show which of krikzz's results it meets. *Recommended:*
   yes, as a local instrument in the probe's scratch folder, never shipped: a test that Nephrite fails and every
   reference fails is weak evidence against the test, and one that all pass names Nephrite's defect. It is a homebrew
   program in a firmware slot, not firmware, so the firmware policy does not forbid it; the decision is the tester's.
4. **Mode 1** (a cartridge with the Sega CD beneath, mcd-verificator's second way and a few programs').
   *Recommended:* stage 10, with a Create setting for cartridges, off by default, since the cartridge finds the Sega
   CD by the boot ROM at `$400000`, which the replacement then supplies.
5. **The backup RAM cartridge.** *Recommended:* a Create setting, "none" by default as a console without one, or
   Sega's 128 KiB cartridge, its contents the second battery file (`.crm`).
6. **A variant of mcd-verificator with its other tests.** Its ASIC, PCM and CDD tests are in its source but not in its
   binary. *Recommended:* yes for stage 9, built locally with a 68000 compiler as a local instrument and not
   committed, which GPL-3.0 allows; it needs a toolchain this machine lacks (`m68k-elf-gcc`), which the tester would
   approve installing.
7. **Recordings** of a console's CD-DA and PCM output, as MDFourier's were for the Genesis (§28). *Recommended:* not
   needed for stage 8; for stage 9 useful but not required.
8. **`MEDIA` for a core whose machines may have no drive** (added at §53.2): whether a core ABI minor should let a
   `MEDIA` core answer an empty drive list for a machine without one. *Recommended:* yes; until then Nephrite claims
   neither `MEDIA` nor `MEDIA_SEEK`.

## 50. Stage 8, steps 1 and 2: the sub CPU, the gate array and Word RAM (2026-10-14)

The two steps were built together, Word RAM being reached through the gate array's memory mode register, and are
recorded together. Their gate (§49.3) is met: mcd-verificator's twelve tests of them pass.

### 50.1 What they built

- **The attachment** (`src/mcd/`): `MegaCd`, the sub CPU (a second `beryl_m68k::M68000`) and `Sub`, everything on its
  bus. The Genesis holds it as `Hw::mcd`, present for a disc, with `mcd_low` set when it fills `$000000`-`$3FFFFF`
  (mode 2). A cartridge's Genesis has none, and its map is the one it had: the two arms the attachment adds to each
  access are guarded by a field set only for a disc.
- **The sub CPU's clock** counts its own 12.5 MHz clocks, the 50 MHz crystal over four; the ratio to the master clock is
  kept as a reduced fraction (44/189 for NTSC's 4,725,000,000/88 Hz, 500,000/2,128,137 for PAL), so nothing drifts.
  The sub CPU is brought up to the main CPU's present after each main instruction and before each main access to the
  gate array, instruction by instruction; held in reset, with its bus granted, stopped with no request it can take,
  or waiting for Word RAM, its clock moves to the present without it. A STOP with nothing to wake it skips to the
  timer's next count.
- **Mode 2's map** on the main CPU (the Hardware Manual's §1-1, §1-3): the boot ROM at `$000000`, PRG-RAM's window
  at `$020000` in four banks by BK1-BK0 and reachable only while SRES holds the sub CPU or SBRQ has its bus, Word RAM
  at `$200000`, each repeated through its half of `$000000`-`$3FFFFF`; the H-INT vector read at `$000070` as `$FFFF`
  and `$000072` as `$A12006` (§4-1); the empty cartridge slot at `$400000` reading `$FF` (D-32).
- **The sub CPU's map** (§1-2): PRG-RAM at `$000000` with WP7-WP0's protection of the first `WP × $200` bytes from
  the sub CPU alone (not from the main CPU, nor, by mcd-verificator's CDC test, from DMA); Word RAM at `$080000`;
  backup RAM on the odd bytes of `$FE0000`, 8 KiB repeated every 16 KiB through `$FEFFFF`; the PCM chip's registers
  and its memory window at `$FF0000` (the window and the bank alone; its sound is stage 9's); the registers at
  `$FF8000`.
- **The gate array's registers**, both sides (§3, §4): SRES and SBRQ, the reset exception taken when SRES is
  released; the LEDs, RES0 (0 for 100 ms after power-on and after a reset of the peripherals); the memory mode and
  write protection; the H-INT vector, kept through a reset as mcd-verificator's REG 2006 has it; the communication
  flags (each side's byte written by any write of that side, at either address), commands and statuses; the stopwatch
  (30.72 µs a count, 384 sub clocks, cleared by a write without restarting its 30.72 µs); the timer (its value counted
  down at 30.72 µs from a fresh count at each write, level 3 at 0, every `(n + 1) × 30.72 µs`); the interrupt mask,
  levels 2 and 3 and their requests, autovectored, a request ended by its acknowledge, a level whose enable is clear
  making no request (D-30); the font registers' colour arithmetic; the forced reset, the main CPU's four writes of
  §4-1, which returns the sub side's timer, stopwatch, mask, requests, LEDs and font registers to their power-on
  values and resets the peripherals.
- **Word RAM** (`src/mcd/word.rs`), stored in 2M order, bank 0's word n at word 2n and bank 1's at 2n + 1 (§1-1,
  "data arrangement"): 2M held whole by one CPU; 1M as two banks by RET, the main CPU's bank at `$200000` with its
  cell image at `$220000` in five strips of cells, the sub CPU's at `$0C0000` with its dot image at `$080000`, a pixel a
  byte, written through the priority modes; the swap rules of D-31. A main CPU access to Word RAM it does not hold
  reads the open bus and writes nothing. **A sub CPU access waits** (the manual's §3-1 note says the sub CPU hangs
  until a forced reset; mcd-verificator's WORD RAM test 0x20-0x21 has its write wait and finish once DMNA gives the RAM
  back): the instruction finishes, its writes held and made when the RAM is the sub CPU's again, and the CPU waits till
  then. A read in that instruction gets 0, a loss argued to be unseen, no program known reading Word RAM it gave away.
- **Byte writes to the registers that take words**: the 68000 drives a byte on both halves of the data bus, and those
  registers take the whole word. This is mcd-verificator's word for `$FF8002`, `$FF8030`, `$FF804C` and `$FF800A`
  ("not support byte writes", "any writes should hit low byte").
- **Machine info and the spaces**: `PRGRAM`, `WORDRAM`, `PCMRAM` and `BRAM` are the attachment's memories, no longer
  stage 0's placeholders. A disc's machine runs its frames from this step: with a boot ROM it runs it, with none its
  68000 reads the open bus and stands.

### 50.2 Sources

The Mega-CD Hardware Manual (Sega, version 1.0, 1991-10-14), read from its page images: "Mega-CD construction", "Bus
construction of CD-ROM interior", §1-1 to §1-3, §2, §3-1 to §3-5, §3-7, §3-9, §4-1 to §4-3. Its OCR text is too
damaged to cite and was used only to find pages. mcd-verificator's source (`test_std.c`, `mcd.c`, `mcd.h`, `sys.c`,
`sega.s`, `sub_bios.s`, `main.c`) for what each test expects, and krikzz's `notes/` (the Word RAM mode notes and the
timings). No emulator source and no RTL was read. The disassembler of Beryl's 68000 was run over mcd-verificator's
binary to read the instruction timing of IRQ TEST's checks (`examples/dis.rs`), the program's own code.

### 50.3 Measured (2026-10-14)

- **mcd-verificator**, the 240p disc as its disc and no BIOS: RAM CART ("not present"), COLOR CALC, VAR TESTS, IRQ
  TEST, REG X000, REG X002, REG 2006, REG X00C, REG 8030, PROG RAM, WORD RAM and WRAM PMOD answer OK; CDC REGS
  answers "ERROR: 01" and CDC INIT waits for the drive, both step 8.4's and 8.3's. The program reports the Sega CD at
  `$00000000`, mode 2. VAR TESTS, X00C, 8030 and IRQ TEST count the main CPU's loops against the sub CPU's work and
  the timers, so their passes say that the two clocks, the main CPU's refresh waits and the sub CPU's unwaited bus
  stand in the console's proportion within those tests' windows (the narrowest ±0.2%).
- **IRQ TEST** failed twice on the way, each failure pointing at a rule: 0x0D with a queue of one request behind the
  pending one, and 0x0E with the sub CPU's step allowed to take a request past the main CPU's present (D-30).
- **The crate's tests**: 145 pass, among them `the_swaps_follow_dmna_and_ret`, the cell image's strips and
  `mcd_verificator_passes_the_tests_built`, which asserts the twelve OKs.
- **The cartridge Genesis's cost**, `examples/bench` (best of five runs of 600 frames, drawn), this build beside the
  build before the step, two alternations each: Sonic the Hedgehog 1.344-1.350 ms (before 1.343-1.348), its sequel
  1.230-1.233 (1.230-1.231), Thunder Force IV 1.329-1.331 (1.323-1.332), Streets of Rage 2 1.129-1.141
  (1.133-1.139), Phantasy Star IV 1.402-1.409 (1.396-1.398). Unchanged within the runs' spread but for Phantasy
  Star IV's 0.5%, inside its own spread on a second pair.

## 51. Stage 8, step 3: the drive (2026-10-14)

### 51.1 What it built

- **The CDD** (`src/mcd/cdd.rs`), the drive's microcontroller as the sub CPU meets it. The Hardware Manual blacks out
  its protocol (§3-6, "direct access ... is prohibited"), so its rules are D-33's: an exchange every 1/75 s (500,000/3
  sub clocks, in periods of 166,667, 166,667 and 166,666 so that three are exact), in which the drive's ten status
  nibbles reach `$FF8038`-`$FF8041` and level 4 is raised while HOCK is set, and the ten command nibbles written at
  `$FF8042`-`$FF804B` are taken at the next exchange, the write of the last sending them (DTS set meanwhile). A
  nibble checksum closes each ten: the first nine summed, inverted, four bits.
- **Its commands**: stop, the reports (absolute and relative position, track, the disc's length, its first and last
  tracks, a track's start), play, seek, pause, resume, fast forward and back, close and open the tray; a command
  whose checksum fails is answered with status 6 and not taken. **Its statuses**: stopped, playing, seeking, scanning,
  paused, tray open, ready (after the TOC is read), no disc, lead-out.
- **Its timing**, from the LL logs: ~~after HOCK rises the drive spins up and reads its TOC (80 exchanges, the logs
  giving 53 to 150), answering reports only then;~~ *the start corrected in §53.1;* a seek is 12 exchanges of "seeking" with no report, then a coarse
  travel of about 0.9 √(sectors) exchanges reporting places on the way, landing 14 sectors short of its target and
  reading on; a play reports "playing" from 3 sectors short, a seek pauses there and loops over the eight sectors
  up to it, an exchange without a report between laps.
- **The Q subcode it reports** is `emusen-disc`'s for the sector under the head: the absolute and relative times and
  the track as BCD nibbles, the data flag in the status's flags nibble (bit 2), bit 0 set while not playing. A track's
  start comes, as on the logs, when the lead-in's cycle of TOC items passes that track's.
- **The disc** is opened through `emusen-disc` (`mcd::open_disc`): a sheet with its tracks, a CHD, a raw track, or
  2,048-byte sectors whose first bytes are `SEGADISCSYSTEM`, which is the console's signature and so the core's
  word that headerless sectors are a disc (`Lone::User`), framed as mode 1. Its bytes are a `Kept`, the host's own
  where the host lends them. Each sector the head reads is named to the host (`MEDIA_SEEK`, file and byte from
  `Disc::place`) unless it lies in the megabyte after the place last named, as Diamond's drive does.

### 51.2 Sources

mcd-verificator's `cdd.c`, `cdd.h`, `test_cdd2.c` and `test_cdc_new.c` (`cddInitToc`, `cddUpdate`, the command
builders); krikzz's `notes/cdd-cmd-info.txt` and `random notes.txt` (the commands, statuses and reports, the seek's
loop); the LL logs of `cd-log/LL-mcd-tst/`, each exchange of a status and a command on a console with mcd-verificator
as host, read collapsed by repeats (`toc-commands`, `seek`, `seek-resume`, `LL-read-sector`, `read+get-pos`,
`read+get-relpos`, `read+get-track`, `read-pause-resume`, `read-stopall`, `LL-stop-rd`, `LL-bad-crc`). The Hardware
Manual §3-6 for the registers' bits. Not read: `cd-log/BI-mcd-tst/`, `cd-log/sonic-bios/`.

### 51.3 Measured (2026-10-14)

The crate's drive tests: the checksum against the logs' exchanges; a spin-up, the TOC read and its reports in the
logs' form (`94.010200.1E` for tracks 1 to 2); a play that is busy for twelve exchanges, lands 14 short and reports
playing from 3 short, sector by sector; a seek that loops in the eight sectors short of its target; a bad checksum
answered with status 6 and not taken. mcd-verificator's CDC INIT, which plays from 00:01:73 through the drive and waits
for the sector 00:02:00 (§52), passes.

## 52. Stage 8, step 4: the CDC (2026-10-14)

### 52.1 What it built

- **The LC8951** (`src/mcd/cdc.rs`): its sixteen registers through the register address at `$FF8005` (a fifth bit,
  RS, the gate array's own: with it set no register answers) and the data at `$FF8007`, the address moving on at
  each access but R0's, from 31 back to 0; IFSTAT, the byte counter, the header, the block pointer, the write
  address, STAT0-STAT3 on reads; IFCTRL, the counter, the data address, DTTRG, DTACK, the write address, CTRL0,
  CTRL1, the block pointer and the chip's reset on writes (Sanyo's manual §10-§11 and appendix E).
- **The decoder**: each 1/75 s with DECEN, the decoder interrupt (DECI), which reading STAT3 clears and which clears
  itself 67,000 sub clocks (40%) into the sector (D-34); a data sector the drive read written, with WRRQ, into the
  16 KiB buffer at the write address, its 12 sync bytes first, the block pointer at its header, HEAD0-HEAD3 its
  header, STAT0 CRCOK, STAT2 its mode.
- **The host transfer**: DTTRG with DOUTEN sends `DBC + 1` bytes from the data address, a word at a time, the buffer
  wrapping at 16 KiB. To a CPU (DD 2 or 3) each word waits in the host data register (`$A12008`, `$FF8008`) with DSR
  set; an access by that side, read or write, takes it and loads the next; the other side reads it as it stands.
  By DMA (DD 4, 5, 7) a word every 16 sub clocks, a byte to the PCM chip, at the DMA address of `$FF800A` (its bits
  18-3), into PRG-RAM (not write-protected for it), Word RAM (2M, or the sub CPU's bank in 1M) or the PCM chip's
  memory in its selected bank (the address counting two a byte); held while the destination is the main CPU's. The
  last word loaded sets EDT and the transfer end interrupt (DTEI); an even count's last byte is not sent.
- **Level 5** on each fall of the chip's INT pin, which is low while an interrupt is set and enabled: a second
  interrupt while the first stands makes no request.
- **`$FF8004` and `$A12004`**: EDT, DSR and the destination on both sides, the register address on the sub CPU's;
  a write of the destination clears EDT and DSR and sets the DMA address to 0 without stopping a transfer.

### 52.2 Sources

Sanyo, "LC8950 Real-Time Error Correction and Host Interface Processor Design Manual", revised for the LC8951 (as
held on Sega Retro): §10 (the register tables), §11.1-§11.16, §12.5-§12.6 and appendix E. The Hardware Manual §3-2,
§4-2. mcd-verificator's `test_cdc_new.c`, `cdc.c`, `cdc.h`, `test_std.c` (CDC REGS).

### 52.3 Measured (2026-10-14)

- **mcd-verificator**, all eighteen tests: CDC REGS, CDC INIT, CDC FLAGS (with DECI's 49 and 72 polling loops each
  within one of krikzz's console), CDC DMA2, DMA3 and DMA1 pass beside §50's twelve; the program prints "Diagnostics
  complete." at frame 1,399. `mcd_verificator_passes_every_test` asserts all eighteen.
- **Each rule shown to be held**: DECI cleared after 30,000 sub clocks fails CDC FLAGS 0x40 (22 loops); an even count
  sending its last byte fails CDC DMA2 0x04; DTACK not clearing DTEI fails the run.
- **The crate's tests**: 152 pass.

## 53. Stage 8, step 5: CD-DA, backup RAM, discs at create, the Sega CD's state (2026-10-14)

### 53.1 What it built

- **CD-DA** (`src/mcd/cdda.rs`): each audio sector the drive plays, while it reports playing, gives its 588 stereo
  samples spread over the sector's 1/75 s, scaled by the fader (FD10-FD0 of `$FF8034`, `$400` full), to a step
  synthesiser of its own at the master clock, which the mix adds after the Genesis's output circuit (CD-DA does not
  pass through it). A full-scale sample is half the output's range (D-35). The fader's ramp and de-emphasis are
  stage 9's.
- **The drive's start**, settled again on the player's BIOS (§53.4): from HOCK's first rise the drive is busy (`0F`)
  for 40 exchanges and deaf to commands, then stopped and not busy (`00`); the first report asked for reads the TOC
  (busy for 20 exchanges), after which status 9 and the reports, as krikzz's notes say ("9 only after toc cmd") and
  the LL logs time it (D-33).
- **Backup RAM**: the internal 8 KiB as battery file 0 (`.brm`, `BRAM`); **the backup RAM cartridge**, Sega's 128 KiB,
  plugged in by the Create setting `ramcart` (hidden while the Sega CD is offered to no player), its bytes battery
  file 10 (`.crm`, `CARTRAM`, space 21). The plan's §4.4 named it file 1, which the TMSS ROM already has. Core info
  lists both files for `mcd` (`battery_files`). mcd-verificator's RAM CART passes with it plugged in (size, write
  protection, memory) and with it absent.
- **Discs at create**: a sheet, a CHD or an image whose first sector carries `SEGADISCSYSTEM` is opened through
  `emusen-disc` with its companions kept by `keep`, so a host's lent bytes are read in place (`BORROW` is claimed);
  the system is read from the disc's system area at 00:02:00; a sheet whose disc is no Sega CD's is refused with
  status −11 and words. The region's BIOS (any of the three else) is the boot ROM.
- **State version 19**: the Sega CD's block after the Genesis's (the sub CPU, its clock, the gate array, Word RAM's
  switches, the sub CPU's wait and its owed writes, the PCM chip's registers, the drive, the CDC with its 16 KiB
  buffer, the fader, CD-DA's samples in flight, the cartridge's switch) and the disc's fingerprint (`emusen-disc`'s, of
  the tracks, the lead-out and the system area's first two sectors). A state over another disc is refused with
  `WRONG_MEDIA`. A Genesis state of version 16 to 19 loads; a Sega CD state of version 18 or older is refused with
  the ABI's words, the Sega CD having run in no build before 19. The Genesis's layout is unchanged, 19 adding nothing
  to a cartridge's state.
- **What a player sees**: nothing new. The `mcd` system stays marked in development, the firmware's words now saying
  that a disc runs on the player's BIOS and that without it no disc starts (§49.2).

### 53.2 Not claimed: `MEDIA`, and with it `MEDIA_SEEK`

*Answered 2026-10-15:* core ABI 1.6 lets a `MEDIA` machine list no drive, and Nephrite claims `MEDIA` and `MEDIA_SEEK`
(§56).

The tray and a disc change are written (`media_lid`, `media_insert`, `drives`, tested in the crate), and the
conformance kit's C16 passed on the 240p disc with them claimed. But a capability is the library's, and C16 then
failed on every Genesis cartridge: "MEDIA is claimed but machine info lists no drives". A Genesis has no drive. So
`MEDIA` is not claimed, the exports are not made, and the drive names no place to the host (`NAMES_PLACES`), since a
`MEDIA_SEEK` must name a drive of machine info. **A question for the tester**: whether the core ABI should gain, in a
minor, a `MEDIA` that a machine with no drive may answer with an empty list (C16 then checking only machines that
list one), which would let Nephrite claim it; recommended, since every core with a cartridge console and a disc
attachment (the PC Engine's CD-ROM², the Saturn's cartridge slot) meets the same rule.

### 53.3 The five discs (no BIOS)

`src/mcd/discs.rs`, with `EMUSEN_NEPHRITE_DISCS` naming the copies: each disc's sheet and tracks opened as a host
passes them, the system area at 00:02:00 reading `SEGADISCSYSTEM`, a data track followed by audio tracks, and the
drive answering, through its own protocol, the first and last tracks, the disc's length and track 2's start as the
sheet places them. All five pass: Chuck Rock II (13 tracks), Lunar - Eternal Blue (3), Mickey Mania (28), Sonic CD
for Europe and for the USA (35 each).

### 53.4 The five discs on the player's BIOS

`examples/disc.rs` runs a disc with a boot ROM given as a firmware path and prints the devices as their registers
stand (the drive's status and last command, HOCK, the interrupt mask, the communication flags), the picture every
few frames and the sound's level; nothing of the boot ROM is read or printed. Run on 2026-10-14 with a model 1 and
two model 2 American BIOS and a European one, 1,800 frames each, Start pressed at frame 900:

- **Every disc stops at the BIOS's first screen**, its own still picture, drawn by frame 180 and unchanged after. No
  disc's data is read and no sound is made.
- **What the devices show**: the sub CPU's program writes the interrupt mask (levels 2 and 4) and HOCK at about
  frame 75, the drive answers its first status, and the program then clears the mask and HOCK, resets the
  peripherals, waits for RES0 and starts again: after one exchange on the first attempt, after about 45 frames on the
  second, and after one exchange on every later one, every 7 frames, never sending a drive command but NOP. The
  main CPU sets its communication flag and waits for the sub CPU's, which never comes.
- **What was tried, each at the devices only**: the drive's start from power-on rather than from HOCK; a TOC read
  automatically after the spin-up; the peripheral reset returning the drive to idle; the first status held back by
  1 to 8 exchanges; the main CPU's level 2 request held while IEN2 is clear (which fails mcd-verificator's IRQ TEST
  0x06, 127 interrupts of 128). None changed the loop, and none was kept but the drive's start of §53.1. What the
  BIOS waits for is not visible at the registers; reading its code is not allowed, and stage 9, whose graphics ASIC,
  PCM and subcode are not yet built, is where it is looked for next (§54.3).

### 53.5 Measured (2026-10-14)

- **The crate's tests**, 157 pass, with the programs and the discs named: the eighteen verdicts and RAM CART with the
  cartridge; the five discs; CD-DA's samples a sector apart at the fader's volume and silent at 0; a disc's machine
  through create, its state round trip, `WRONG_MEDIA` over another disc, version 18 refused, the tray and the insert;
  the cartridge's setting and battery files.
- **The conformance kit**: the Genesis sample, the core's cases and 64 of 64 images, in 171 s; the 240p disc with
  mcd-verificator as its boot ROM, all eighteen cases (C17: lent, every frame as from a copy), and with `MEDIA`
  claimed C16 too before it was withdrawn (§53.2); Mickey Mania's and Sonic CD's sheets, 29 and 36 files lent,
  compliant, the latter with a player's BIOS as file 2.
- **`emusen-disc`**: 13 and 13 (§11 there); Diamond's crates, its other user, 167 unit and 81 program tests with the
  rest, all pass.
- **WiseMan**: `NephriteTests`, `NephriteDebugTests`, `NephritePlayersTests`, `NephriteStartTests`, the Genesis
  settings, battery and cheat tests and the registration golden, 83 pass. Three were updated: the library's
  capabilities (with `BORROW`), the `.iso` stub now running its frames (its picture the VDP's 256 × 224, as a blank
  cartridge's), and none for the hidden setting, which the frontend does not list.

## 54. Stage 8's gate, and stage 8 closed (2026-10-14)

### 54.1 The gate

| What §49.3 asked | Result |
|---|---|
| mcd-verificator's self-graded results, each test passing or a dispute or a named loss | **All eighteen pass**: RAM CART (absent and present), COLOR CALC, VAR TESTS, IRQ TEST, REG X000, REG X002, REG 2006, REG X00C, REG 8030, CDC REGS, PROG RAM, WORD RAM, WRAM PMOD, CDC INIT, CDC FLAGS, CDC DMA2, CDC DMA3, CDC DMA1; "Diagnostics complete." at frame 1,399 |
| The Genesis's goldens and kit unchanged | The kit's sample 64 of 64; the crate's program tests (BCD verifier, opcode sizes, illegal instructions, VDPFIFOTesting and the rest) pass |
| The new state version loading, the old ones still loading | Version 19; Genesis states of 16, 17 and 18 load (`a_version_16_state_loads_and_runs`, `a_version_17_...`), 15 and 20 refused; a Sega CD state round trips and is held to its disc |
| The Genesis's frame cost unchanged | Best of 6 × 3 runs of 600 frames, this build beside the build before stage 8: Sonic the Hedgehog 1.333 ms (1.338), Thunder Force IV 1.320 (1.316), Phantasy Star IV 1.393 (1.390), Sonic the Hedgehog 2 1.215 (1.230), Streets of Rage 2 1.118 (1.127). Unchanged; a 1.4% loss measured on the way, the attachment's checks on every access, was removed by one check for anything attached to the map (the J-Cart's and the Sega CD's), so the cartridge pays what it paid before |
| The Sega CD's cost recorded | 1.513 ms a frame, mcd-verificator's run on the 240p disc (best of 5 × 1,200 frames), against the plan's 2.5 |

### 54.2 Disputes

D-30 to D-35, each with its evidence; none open but D-33's spin-up length and travel curve and D-34's DMA rate,
which are argued within what the logs and tests allow.

### 54.3 Carried to stage 9

- ~~**The player's BIOS does not get past its first screen** (§53.4).~~ *Found and fixed 2026-10-15 (§55).* Stage 9's first task, before the ASIC and the
  PCM, since its references (the five discs on that BIOS against Genesis Plus GX) need it. Places to look, all at
  the registers: the subcode registers and level 6 (`$FF8068`, `$FF8100`, not built); the graphics ASIC's registers
  (`$FF8058`-`$FF8066`, not built, reading 0); the PCM chip's registers, which read 0; the version bits of `$FF8000`;
  what a peripheral reset resets.
- ~~**`MEDIA` and `MEDIA_SEEK`**, waiting on the tester's answer to §53.2's question.~~ *Claimed 2026-10-15 (§56).*
- **`.cue` and `.chd` for the `mcd` system**: the core opens them, but the system pack claims `.iso` alone, since a
  frontend tells a Sega CD's sheet from a PlayStation's only by opening its first track, which the system pack's
  recogniser does not yet do. Due with the Sega CD offered (stage 10).
- mcd-verificator's tests that are in its source but not its binary (the ASIC, the PCM, the CDD at length), with the
  tester's answer to §49.6 Q6.

### 54.4 Stage 8 closed

Stage 8's oracle, mcd-verificator with no BIOS, is met in full. Its steps were six in the split; 8.1-8.2 and 8.3-8.4
were each built as one, and 8.5 and 8.6 are §53 and this section.

### 54.5 Reading log (stage 8)

Read: the Mega-CD Hardware Manual's page images (§50.2); Sanyo's LC8950/LC8951 design manual's text and register
pages (§52.2); the Mega-CD BIOS Manual was not opened; mcd-verificator's readme, its `src/` (all of `test_std.c`,
`test_cdc_new.c`, `mcd.c`, `mcd.h`, `cdc.c`, `cdc.h`, `cdd.c`, `cdd.h`, `sys.c`, `sys.h`, `main.c`, `main.h`, `sega.s`,
`sub_bios.s`, `sub_app.s`, the head of `test_cdd2.c` and of `pcm.c`'s memory helpers), its `notes/`, and its
`cd-log/LL-mcd-tst/` logs read collapsed by repeats; railgun's "Mega CD Docs: Memory Map" (wikitext); the 240p Sega
CD build's README. Its own binary disassembled at IRQ TEST's checks with Beryl's disassembler. Screened by name and
not opened: `cd-log/BI-mcd-tst/` and `cd-log/sonic-bios/`, logs made under a BIOS. Not read: any emulator's source,
any RTL, any BIOS image, listing, trace or log; the player's BIOS was given to Nephrite as a file and nothing of it
was read or printed. The five discs were read only through the drive and `emusen-disc`, as a console reads them.

## 55. Stage 9, first step: the player's BIOS past its first screen's stall (2026-10-15)

§53.4 left every disc standing at a player's BIOS's first screen, and §54.3 listed where to look. This step looked at
the devices' register interfaces only, as the firmware rule of §49.1 requires, and found the stall in the drive.

### 55.1 What was built to look

- **A watch on the registers** (`Sub::seen`, `mcd::Seen`): when a development tool asks for it, every access at the
  attachment's register interfaces is kept with its sub clock: the sub CPU's reads and writes of `$FF8000`-`$FF81FF`,
  the PCM chip's registers, the main CPU's of `$A12000`-`$A1203F`, each level the sub CPU takes, and each time the sub
  CPU begins to wait for Word RAM. Nothing of memory is kept. `examples/disc.rs` prints it for a span of frames
  (`NEPHRITE_SEEN`), a run of one access as one line, and writes PRG-RAM at the end (`NEPHRITE_PRG`) for a homebrew
  boot ROM's records, never with a player's BIOS. The watch is `None` in every other use, at the cost of one branch
  on a register access.
- **A drive probe** (`tools/cddprobe/`, 68000 assembly, D-36): a boot ROM for mode 2 whose sub program records every
  status the drive sends while it is answered with NOPs, through four phases.
- **The references in the BIOS's slot** (§49.6 Q3): the probe's libretro backend given a scratch system folder whose
  BIOS names are links to a program's file, so the program runs as the Sega CD's boot ROM. Genesis Plus GX and
  PicoDrive run mcd-verificator and the drive probe this way; BlastEm loads neither.

### 55.2 What the devices saw

With a model 1 American BIOS and Sonic CD (USA), the gate array and the drive saw this, by frame: the sub CPU
released at frame 4; the interrupt mask written with levels 2 and 4 and HOCK set at frame 75; one exchange, its
status `0F.000000.1F` read at `$FF8038`-`$FF8041` after `$FF8037` was read with DRS 0, a NOP sent, `$FF8034` read
and the LEDs written; then the main CPU's level 2, and the sub side's mask cleared, HOCK cleared and the peripherals
reset (`$FF8001` written 0). After RES0 the same again, every time a level 2 followed an exchange, whether the status
was busy (`0F`) or the stopped `00.000000.1E` of D-33. A level 2 before any exchange changed nothing, and with level
4 withheld nothing was reset. The sub side never read `$FF8058`-`$FF8068`, the subcode buffer or the PCM chip before
this point, and never touched backup RAM, PCM RAM or an unmapped address: none of §54.3's unbuilt registers was
involved.

Varied one at a time at the registers, each a run of 300 frames: the version bits of `$FF8000` (all sixteen values),
EFDT and the de-emphasis bits of `$FF8034`, DM and DTS of `$FF8036`, DRS set for up to 3,000 sub clocks after each
exchange, the first status after HOCK all zeros (as three of the LL logs open), and the drive reading its TOC on its
own after the spin-up: none changed the reset. DRS held at 1 for 20,000 clocks, or always, stopped it, the status
then never read (DRS polled and, after about 6,700 clocks, written 0, which the manual says aborts the reception).
**The flags nibble decided it**: with bit 0 clear in the stopped, not busy status before a TOC is asked for, the
first level 2 after such a status reset nothing and the drive was asked for its TOC; a busy status earlier in the
same attempt still led to a reset. D-36 has the drive probe's evidence and the three rules taken from it.

### 55.3 What changed

- **The drive** (D-36): it spins up from power-on, 75 exchanges, whether or not HOCK is set; spun up and stopped with
  no TOC asked for, it reports `00.000000.0F`; a play or a seek during the TOC's read is taken.
- **The version register's bit 5** reads 0 with the Sega CD attached ("Check if bit 5 of HwVersion is 0", plutiedev's
  "Sub CPU in mode 1"); a cartridge's console keeps 1.

### 55.4 The discs on the player's BIOS now (Nephrite)

The four American discs on a model 1 American BIOS, 1,800 frames, Start at frame 900: the drive is asked for its TOC
at frame 79, answers status 9 at frame 96, and the communication flags move from frame 82 on; the sub side writes the
graphics ASIC's eight registers once, the last the trace vector base, with level 1 enabled, and the PCM chip's
registers from then on; the drive seeks and pauses near the disc's start (sector 152 to 157 at frame 1,800), the CDC
interrupts (level 5) are taken, and the BIOS's music plays (peak about 3,300 of 32,767). The picture is the first
screen's background and its copyright line without the logo, which Genesis Plus GX draws over it by frame 180 with
"Press the START BUTTON." below; Start does not leave it. The ASIC has no level 1 to give (step 9.1). Sonic CD (Europe)
on a model 1 European BIOS shows its first screen and, from frame 1,080, "Press RESET to open the CD tray.", the
drive stopped with no data read: taken up after the ASIC (§56 on).

### 55.5 mcd-verificator in the references' slot

Each run on the 240p disc, 1,800 frames, the verdicts read from the picture:

| Test | Nephrite | Genesis Plus GX | PicoDrive |
|---|---|---|---|
| RAM CART | OK (none) | OK | ERROR: 01 |
| COLOR CALC | OK | OK | OK |
| VAR TESTS | OK | OK | ERROR: 02 (17) |
| IRQ TEST | OK | ERROR: 08 (850) | ERROR: 06 (127) |
| REG X000, X002, 2006, X00C | OK | OK | OK |
| REG 8030 | OK | ERROR: 07 (1281) | ERROR: 07 (1281) |
| CDC REGS, PROG RAM, WORD RAM, WRAM PMOD | OK | OK | OK |
| CDC INIT | OK | ERROR: 02 (no CDC interrupt in time) | OK |
| CDC FLAGS | OK | not run | ERROR: 41 (74) |
| CDC DMA2 | OK | not run | OK |
| CDC DMA3 | OK | not run | ERROR: 44 |
| CDC DMA1 | OK | not run | OK |

BlastEm loads no Sega CD from the slot. Neither reference passes everything krikzz's console passes, and they fail
different tests; REG 8030's 0x07 in both, the timer's period, Nephrite passes as the console does (§50.3).

### 55.6 Measured (2026-10-15)

- **The crate's tests**: 159 pass, two new for D-36 and the version bit in the boot-ROM-less test.
- **The drive probe** in Nephrite after the change: 67 busy exchanges after its HOCK, early in its first frames, then
  `00.000000.0F`; after a reset of the peripherals the same idle status at once; a TOC request busy for 20 exchanges,
  then `94.013500.18`.

## 56. `MEDIA` and `MEDIA_SEEK` claimed, on core ABI 1.6 (2026-10-15)

§53.2's question was answered yes and built elsewhere: core ABI 1.6 (`EmuSen_CoreAPI.md` §6.26, §17, §28.11) lets a
`MEDIA` machine list no drive, its media calls then answering `NO_SUCH_DRIVE` and changing nothing, and the kit's C16
checks the drive rules only where a drive is listed. Nephrite already answered so (`drives` empty without the Sega
CD, both media calls `NO_SUCH_DRIVE`), so the claim is three lines: `caps::MEDIA` in the capabilities, `media` among
the exports, and the drive naming its places (`cdd::NAMES_PLACES`), which turns `MEDIA_SEEK` on as §51.1 built it.

**Measured (2026-10-15)**, `emusen-core-conform` 1.6:

- The Genesis sample: the core's cases and 64 of 64 images pass, C16 by the no-drive rule and C18 with no place
  named, in 77 s at 16 at a time.
- The 240p disc with mcd-verificator as file 2, Sonic CD (USA) and Mickey Mania with a player's BIOS as file 2, 600
  frames each: compliant, C16 (an insert refused with the tray closed, the disc changed through the tray, a state held
  to its disc over 240 frames), C17 and C18 (one place named in 600 frames, every frame's picture, sound and state the
  same whether the host acted on it or not).
- The crate's tests, 159; WiseMan's Nephrite, Genesis and Registration tests, 83, the library's capabilities now
  with `MEDIA`.

## 57. Stage 9, step 1: the graphics ASIC, and the three faults it uncovered (2026-10-15)

### 57.1 What it built

- **The graphics ASIC** (`src/mcd/asic.rs`), from the Hardware Manual §3-8: `$FF8058` (GRON, SMS, STS, RPT) to
  `$FF8066`. A write of the trace vector base in 2M starts an operation; each line reads its four vector words (the
  13.3 start, the 4.11 steps), steps across the stamp map a dot at a time, and writes each dot into the image buffer,
  whose cells stand in columns of `Vcell + 1` as the VDP's sprites do, from the buffer's start and offset, by Word
  RAM's priority mode (overwrite and underwrite as for the CPU's dot image). A map entry's stamp, flip and quarter
  turns pick the dot from the stamp generator, stamp 0 and dots off the map (RPT 0) being 0. The operation is spent a
  line at a time on the sub side's events, `Vdot` counting down, GRON set till the last line and level 1 with it; its
  time, the refresh's share and the three clocks each sub CPU or DMA access to Word RAM costs it are D-37's.
- **A forced reset** returns the ASIC to rest ("Rotation" in the manual's §4-1 list).
- **State version 20**: the ASIC's registers, line and times in the Sega CD's block; a Genesis state of 16 to 20
  loads, the Genesis's layout unchanged; a Sega CD state of 19 is now refused, no player build having run the Sega CD.

### 57.2 Three faults the BIOS's logo and boot showed

Each was proven by a run that failed with it and passed without it, then pinned by a test.

- **The VDP's DMA from Word RAM** reads the word before the one addressed. The manual's "Precautions in using DMA of the
  VDP of the Mega Drive to transfer Word RAM" (§4, page 61 of the copy): the VDP's read of Word RAM "will be output with
  a one cycle delay", and programs set the source a word later; krikzz's `asic.c` does the same (`ptr + 1`). Without it
  the logo's rows were torn, four dots across; with it the frames match (§57.3).
  `a_transfer_from_word_ram_reads_a_word_late`.
- **The CDC's monitor-only mode**: with DECEN set and WRRQ clear the decoder still loads HEAD0-HEAD3 (the subheader
  with SHDREN) and the status for each block, writing nothing to the buffer (Sanyo's manual §12.5.3). Nephrite had
  loaded them only with WRRQ, so the header read after each decoder interrupt was zero, and the drive was told to play
  from 00:01:73 again and again. `with_wrrq_clear_the_header_is_decoded_and_nothing_written`.
- **A track's start carries the track's number** in the flags nibble (D-33, amended). With the state flags there the
  drive was asked for track 1's start every 90 frames and no disc was read.

### 57.3 Measured (2026-10-15)

- **The player's BIOS (a model 1 American one) with Sonic CD (USA)**, 1,400 frames and no button pressed: the first
  screen's animated logo, the planet and "Press the START BUTTON.", then at frame 960 the disc boots by itself to
  Sonic CD's licence screen with its stars. Against Genesis Plus GX given the same BIOS, frames 300, 480, 540, 600
  and 720, still and turning logos, are its pictures to the pixel up to a colour map; frame 180 differs only in the
  text, which Nephrite shows from frame 300 and the reference from 180, and the boot is 120 frames later (840 there).
  The lag is the drive's: the reference answers a track's start at once, and the LL logs, a console's drive, answer it
  as the lead-in's cycle passes the track (114 exchanges on a 35-track disc), as Nephrite does.
- **The crate's tests**: 165 pass, the four of the ASIC and three for §57.2 among them.

## 58. Stage 9, step 2: the RF5C164 (2026-10-15)

### 58.1 What it built, and the oracle

- **The RF5C164** (`src/mcd/pcm.rs`), from the Hardware Manual's "PCM Sound Source" and Ricoh's RF5C68A data sheet:
  the control register (ON; MOD choosing the wave bank or the channel bank), the channel switches (a channel sounds
  while its bit is 0), each channel's ENV, PAN, FD, LS and ST, the addresses read back at `$10`-`$1F`. A sample every
  384 of the chip's clocks, the sub CPU's 12.5 MHz (32,552 Hz). A channel that is off while the chip sounds holds its
  address at ST; the chip off holds every address; a channel's first sample reads ST's byte, and each later one steps
  its 16.11 address by FD; a byte of `$FF` sends the address to LS and the RAM is read there. Samples are sign and
  magnitude, bit 7 set for positive, summed and limited as D-38 says. The chip is brought up to the present at every
  access to it, before the CDC's DMA writes its RAM, and at a frame's end.
- **The mix**: CD-DA's samples and the PCM chip's go to one synthesiser in time order, as the sum of the two sources'
  levels (`cdda.rs`'s `feed`).
- **State version 20** holds the chip in the Sega CD's block in place of stage 8's nine register bytes.
- **The oracle, built locally (§49.6 Q6)**: mcd-verificator V1.02's binary carries `pcm.c`'s three self-grading tests
  though its `main` never calls them. Rebuilding its source with this machine's GCC 16 cross compiler (`-m68000`, with
  32-bit multiply and divide routines written for the 68000 in place of libgcc, whose only build is for the 68020, and
  the linked program's code shown free of instructions outside the 68000's set by disassembling it both as 68000 and
  as 68020 code) gave a program whose timed loops differ from krikzz's compiler's: its standard list then failed VAR
  TESTS, REG X002 and REG 8030 in Nephrite, which passes all of them in the original. So the rebuilt binary is no
  oracle. Instead the original binary was patched in a scratch copy: `main`'s pointer to its test list now points at
  a routine of 58 bytes in its unused space, which call `pcmTestRam`, `pcmTestClocking` and `pcmTestVar` at their
  places in the binary (found by the strings they print) and store their verdicts in work RAM at `$FFA004`. The tests
  run as krikzz's compiler built them. Not committed (GPL-3.0 allows the local copy).

### 58.2 Measured (2026-10-15)

| Test | What it checks | Nephrite | Genesis Plus GX | PicoDrive |
|---|---|---|---|---|
| PCM RAM | the 64 KiB through the 4 KiB window and its banks | OK | OK | OK |
| PCM CLOCK | eight channels at eight steps, run for 8,192 turns of a `NOP`/`SUBQ`/`BPL` loop: the addresses 2, 4, 9, 79, 159 and 318 to 320, which only 636 to 639 samples give | OK | OK | ERROR 1 |
| PCM VAR | a looped 15-byte sample read 4,096 times (the address from 0 to 14, never the marker's 15); an address held at ST while a channel is off | OK | ERROR 2 | ERROR 2 |

Each rule shown held: a sample every 386 clocks fails PCM CLOCK (5), one every 382 passes it, so the test pins the rate
from above to within 0.6%; the address left at the marker fails PCM VAR (1). The first sample's reading ST without a
step is not pinned by the three. Both references fail PCM VAR at 2, the address reaching the marker; a test that
every reference fails and Nephrite passes is not counted against Nephrite, the test's values being krikzz's console's.

- **The crate's tests**: 168 pass, three new for the chip.

## 59. Stage 9, step 3: the fader, de-emphasis and the mix, measured on recordings of Sega CDs (2026-10-15)

### 59.1 What it built

- **The fader** (`cdda.rs`): the LC7883's attenuator moving a unit of FD a sample toward the level `$FF8034` asks for,
  1,024 samples from full to silent, its gain by the data sheet's table; when the drive gives no audio sector the
  output falls to silence, which it had held at the last sample. DEF1-DEF0 (bits 3-2) turn on the 50/15 µs
  de-emphasis for the rate they name. CD-DA's samples pass a five-tap filter that undoes the droop of the
  synthesiser's held steps (D-39).
- **The PCM's analogue stage** (`pcm.rs`): a first-order low-pass at 1,600 Hz after the chip.
- **The levels**: CD-DA's full scale 158/1,024 of the output's range and the PCM's 267/1,024, settling D-35 and D-38.
- **The drive**: a pause sent while it is stopped seeks to where its head is and pauses there (D-39).
- **State version 20** gains the attenuator, the two filters' memories and the low-pass's.

### 59.2 The oracle: MDFourier's recordings of Sega CDs

§49.6 Q7 had no recordings of a Sega CD. MDFourier's set already fetched for stage 5 (2020-06-14) has nine, made with
the 240p suite's MDFourier sequence for the Sega CD, whose last part plays a PCM sweep and then CD-DA (a full-scale
sweep, the suite's track 2). Nephrite ran the suite's 1.32 disc on a player's BIOS (model 1 American) through its menus
to the same sequence (`examples/disc.rs` with `NEPHRITE_WAV`); each run was cut at its sections' starts, the FM notes
being the level each section is held to.

| Console | PCM sweep over the FM notes | CD-DA over the FM notes | Nephrite's error, PCM, CD-DA |
|---|---|---|---|
| Model 1 VA3 with a model 1 Sega CD | −4.68 dB | +7.31 dB | −0.07, −0.33 dB |
| Model 1 VA6 with a model 1 Sega CD | −4.81 dB | +6.69 dB | +0.06, +0.30 dB |
| Model 2 VA2.3 with a model 2 Sega CD | +0.82 dB | +12.14 dB | −5.57, −5.15 dB |
| CDX | +0.22 dB | +15.42 dB | −4.97, −8.43 dB |

CD-DA's sweep against the model 1 pair is flat within 0.55 dB to 17.8 kHz (it lost 2.9 dB at 18.8 kHz before the
droop filter); the PCM's sweep fits the low-pass to 1.3 to 1.4 dB rms on all three Sega CDs measured. Before the step
CD-DA stood 10.2 dB and the PCM 5.7 dB too loud against the model 1 pair. The X'Eye's recording was not cut.

### 59.3 Measured (2026-10-15)

- **The crate's tests**: 170 pass, among them de-emphasis's curve, the fader's ramp, the low-pass and the pause.
- **The 240p suite (1.32) on the player's BIOS**: it boots, its menus answer the pad, and its MDFourier sequence runs
  to the end, PCM and CD-DA included.

## 60. Stage 9's close: the discs and the 240p suite on the player's BIOS, and stage 9 closed (2026-10-15)

### 60.1 Found on the way: a stopped drive's TOC request

Sonic CD (Europe) on a European model 1 BIOS stood at the first screen's "Press RESET to open the CD tray.": the BIOS
stopped the drive when track 1's start had not come in 44 frames, asked for the TOC again, and answers carrying status
0 ended its asking. A stopped drive that has read its TOC now goes to status 9 on a TOC request (D-40), and the disc
boots.

### 60.2 The five discs on the player's BIOS against Genesis Plus GX

Each disc run 4,800 frames (the two slowest 7,200) with no button pressed, on a model 1 American BIOS (the European
disc on a model 1 European one), and in Genesis Plus GX given the same BIOS as its system folder; pictures every 30
frames, each reference picture matched to Nephrite's best up to a one-to-one colour map. Genesis Plus GX and PicoDrive
start with a formatted backup RAM, a console whose RAM was never formatted does not: Nephrite's runs were made both
ways, the formatted RAM an 8 KiB image PicoDrive made (a reference's output, kept in the scratch folder).

| Disc | Unformatted RAM | Formatted RAM | Against Genesis Plus GX |
|---|---|---|---|
| Sonic CD (USA) | boots at frame 960; stops at "Internal RAM is not initialized." | title at 1,740; its streamed intro from 5,160 | the first screen's frames 300-720 exact; the title exact (1,470 there); the intro from 3,600 there |
| Sonic CD (Europe) | boots at 960; stops at the same message | title at 1,860; the intro plays | title at 1,500 there, 16% of pixels apart by the clouds' phase |
| Lunar - Eternal Blue | "BACKUP RAM IS NOT FORMATTED!" at 1,860, as there | the LUNAR logo at 3,870 | the message and the logo exact (1,680 and 3,000 there) |
| Mickey Mania | its own credits at 3,240, the title at 3,870, then its attract play | — | both exact (2,520 and 2,880 there) |
| Chuck Rock II | its intro's scenes from 2,910 | — | the scenes exact (1,440 there), the lead growing to about 1,900 frames by 6,240 |

Every picture the two have in common is the reference's to the pixel up to a colour map but for animation's phase;
Nephrite is later throughout. The lead is the drive's: 120 frames at the boot from the lead-in's cycle (§57.3), and
more wherever a game plays in short steps (each play at least 26 exchanges, D-33's busy, travel and approach from the
LL logs) or seeks far: Sonic CD's seek from its title's track 2 to its data, 200,000 sectors, takes 5 s on D-33's
curve, which the LL logs measured only over short distances. A console's load times were not measured; whether the
reference or Nephrite is nearer a console is not known.

PicoDrive, given the BIOS, boots Sonic CD (USA) to its title by frame 1,440; BlastEm loads no Sega CD from the slot.

### 60.3 The 240p Sega CD suite on the player's BIOS

It boots by the BIOS's Start, shows its intro and menus, answers the pad, and runs its MDFourier sequence to the end
(§59). Against Genesis Plus GX, the same presses given: the main menu at frames 1,400 and 1,500 and the audio menu at
1,600 and 1,800 are its pictures exactly, and frame 1,700 all but 80 pixels.

### 60.4 The gate

| What §49.4 asked | Result |
|---|---|
| The player's BIOS past its first screen (§54.3) | Done (§55): the drive's idle status and its start; the BIOS then needed the ASIC, three faults it uncovered (§57.2) and D-40 |
| 9.1 the graphics ASIC | Built (§57); the BIOS's turning logo the reference's to the pixel |
| 9.2 the RF5C164 | Built (§58); mcd-verificator's PCM RAM, CLOCK and VAR pass (the references fail VAR) |
| 9.3 the fader and the mix | Built (§59); against two model 1 Sega CDs' recordings, PCM and CD-DA within 0.33 dB; D-35 settled |
| 9.4 the discs and the 240p suite on the player's BIOS against Genesis Plus GX | §60.2, §60.3: all five discs and the suite boot and play; pictures equal, Nephrite later |
| mcd-verificator in the references' slot (Q3) | §55.5 |
| mcd-verificator's other tests (Q6) | The PCM's three pass (§58). Its ASIC test (`asic.c`) and its CDD tests (`test_cdd2.c`) print counts for a person to read and grade nothing, so they give no verdict to hold Nephrite to |
| The Genesis unchanged | The kit's Genesis sample 64 of 64 (C1-C19, core ABI 1.7); the crate's program tests pass; the cost as §60.6 |

Stage 9 is closed. Stage 10, the open BIOS, is a fresh writer's (§49.5): nothing of §55-§60's BIOS runs but their
results is for it, and the register watch of §55.1 records nothing of a BIOS but its devices' registers.

### 60.5 What stands open

- **The backup RAM a new console has.** A disc that needs a formatted internal backup RAM stops at its message until
  the player formats it with the BIOS's control panel; both references start formatted. Whether Nephrite should supply
  a formatted RAM on a machine's first run is a question for the tester (the format is the BIOS's, stage 10's).
- **The drive's long seeks and load times** (§60.2), and **its DRS timing** (no status is read during reception).
- **The sub CPU's wait while the ASIC runs** (D-37), not built.
- **EFDT** reads 0: no document says how long a fader write takes to reach the LC7883.
- **The mix on other consoles**: the Sega CD's level against the Genesis is 5 to 8 dB higher on the model 2 pair and the
  CDX than on the model 1 pair Nephrite holds to (D-39).
- **The PCM's RAM read while it sounds**, which the manual calls impossible, still answers.
- **Mode 1** is stage 10's (Q4).

### 60.6 Measured (2026-10-15)

- **The crate's tests**: 171 pass (159 at stage 9's start), mcd-verificator's eighteen among them.
- **The conformance kit**, core ABI 1.7: the Genesis sample 64 of 64 with the core's cases; the 240p disc with
  mcd-verificator as file 2, Sonic CD (USA), Mickey Mania and Lunar with a player's BIOS as file 2 and Sonic CD (Europe)
  with a European one as file 3, 600 frames each: compliant (C1-C19), state round trips with the ASIC, the PCM and the
  filters in them included.
- **WiseMan**: the Nephrite, Genesis and Registration tests, 83 pass after the merge of core ABI 1.7.
- **The cartridge Genesis's cost**, `examples/bench`, best of 10 runs of 600 frames on one core, this build beside the
  build before stage 9, three alternations, with other work running on the machine (load 4 to 7): Sonic the Hedgehog
  1.370-1.409 ms (before 1.370-1.409), its sequel 1.261-1.274 (1.247-1.255), Thunder Force IV 1.347-1.372
  (1.353-1.373), Streets of Rage 2 1.152-1.192 (1.145-1.155 but one run of 1.897), Phantasy Star IV 1.438-1.452
  (1.407-1.455). The only change on a cartridge's path is one test of the attachment flag in a DMA's read; each stage 9
  commit built alone, on a quieter run, measured within 0.3% of the build before. A 1% difference cannot be told from
  the noise of a loaded machine here.
- **The Sega CD's cost**: mcd-verificator on the 240p disc 1.589-1.624 ms (the build before 1.574-1.589); the player's
  BIOS's first screen, the ASIC turning its logo, 2.193 ms; Mickey Mania's title and attract play 2.085 ms; against the
  plan's 2.5.
- **State version 20**: a Genesis state of 16 to 20 loads; a Sega CD state of 20 round trips and is held to its disc;
  19 is refused.
- **What a player sees**: nothing new. The `mcd` system stays marked in development until stage 10 (§49.2); a
  development build runs a disc on the player's BIOS through to its games, the attachment's sound mixed as §59 measured.

### 60.7 Reading log (stage 9)

Read: the Mega-CD Hardware Manual's page images at 110 and 300 dpi: the register table (§2), §3-1 and the initial
values, §3-6 (`$FF8034`, `$FF8036`), §3-8 whole, §3-9, §4-1, and §4's "precautions in using DMA of the VDP"; its "PCM
Sound Source" section, §1-§4.9; Ricoh's RF5C68A data sheet (its text); Sanyo's LC7883 data sheet (the filter,
de-emphasis and attenuation sections, in its text); Sanyo's LC8950/LC8951 manual §12.5; plutiedev's "Sub CPU in mode
1"; mcd-verificator's `pcm.c`, `pcm.h`, `asic.c`, `main.c`, `cdd.c`, `cdd.h`, the head of `test_cdd2.c`, `test_cdc_new.c`'s
CDC INIT, `sub_bios.s`, its `Makefile` and `rom.ld`; krikzz's `notes/` (`asic-render.txt`, `cdd-cmd-info.txt`, `random
notes.txt`); the LL logs, counted by status and flags; the 240p suite's Genesis/Sega CD source (`mdfourier.c`'s
sequence and its PCM and CD-DA parts, `main.c`'s menus); MDFourier's recordings (§59.2); the repository's
`mdrecordings.py`. Disassembled, with Beryl's disassembler: mcd-verificator V1.02's own binary, to find its PCM tests
and `main`'s pointer to its test list. Run as black boxes: Genesis Plus GX, PicoDrive and BlastEm, by their pictures,
sound and the RAM of homebrew programs (the drive probe's, the patched mcd-verificator's). Screened and not opened:
`cd-log/BI-mcd-tst/`, `cd-log/sonic-bios/`. Not read: any emulator's source, any RTL file, any BIOS image, listing,
trace or log. The player's BIOS was given to Nephrite and to the references as a file; of its runs only their devices'
registers, pictures, sound and frame numbers were looked at, never its code, its RAM before a game's hand-over or a
program counter.

## 61. The Sega CD's save media on core ABI 1.8 (2026-10-15)

*Decided 2026-10-15 that the card mode is every console's (`EmuSen_SaveMedia.md`).* The Sega CD has two save memories
of different kinds, and core ABI 1.8 lets the core say so: the backup RAM inside the unit, which every game on the
console shares, is of kind `console`; the backup RAM cartridge, which a player carries between consoles, is
`removable`. Under the host's create flags a state load leaves each as it stands, so a player's saves on a shared
backup RAM survive another game's state, as `Diamond_Cards.md` §1 showed the PlayStation's did not before 1.7.

### 61.1 Exactly what changed

This section is the whole of the change, kept to the battery declarations and what they need, since the attachment
interface (`Nephrite_Attachments.md`) is built next on another branch:

| File | Change |
|---|---|
| `src/v1.rs` | `battery_files("mcd")` and `batteries()` list file 0 `.brm` as `Battery::console(..).medium("mcd.backup").label("Inside the console")` and file 10 `.crm` as `Battery::removable(..).medium("mcd.ramcart").label("Cartridge slot")`, through two functions `backup_ram()` and `ram_cartridge()`, in place of two `Battery::game` each; `create_disc` sets `Machine::kept_apart` from `Create.shared_removable` and `Create.shared_console`; every test's `Create` literal gains `shared_console: false`; one test added |
| `src/machine.rs` | `BRAM_ID` (20, the last of `MCD_MEMORIES`, which is unchanged); `Machine::kept_apart`, a `KeptApart { cartridge, backup }` set at create and never part of a state, with `keeps(id)`; the constructor's literal sets it to the default |
| `src/state.rs` | `load_state` skips copying the state's bytes of `CARTRAM` when the cartridge is kept apart and of `BRAM` when the backup RAM is, and is otherwise as it was |

The state's format and version are unchanged: a state still holds both memories, and a machine made without the flags
loads them as before, so per-game behaviour, every state's bytes and the pinned layout are unchanged (the crate's
172 unit tests pass, the layout's among them). The Genesis cartridge's `.srm` stays of kind `game`. Machine info
names the same files with their kinds; core info gains each slot's medium and label.

**The new test**, `the_backup_ram_and_the_cartridge_are_kept_apart_under_the_flags`: core info's two slots with their
kinds, media and labels; machine info's kinds; and a disc machine with the cartridge plugged in, made over backup RAM
11h and cartridge 22h, whose state is loaded into one made over 33h and 44h: with both flags the second keeps 33h and
44h, without them it takes 11h and 22h back.

**Through the host** (`NephriteMediaTests`): the Genesis pack names the two media (`Internal Backup RAM`, noun `Backup
RAM`; `Backup RAM Cartridge`, noun `Cartridge`), each the first of its kind, so their rows are `ConsoleMemory` and
`Cards`; with the backup RAM shared, `Saves/Sega CD/Console/Backup RAM 1.brm` is given to the machine and a state of
the same game leaves it as it stands, while per game the game's own `.brm` is put back by the state.

### 61.2 Not done here

- **What the BIOS sees of a changed memory.** Nephrite presents nothing on a load that keeps a memory: unlike the
  PlayStation's card, the Sega CD's backup RAM has no flag of insertion, and whether the BIOS's backup RAM calls read the
  directory from the RAM each time, as argued in `EmuSen_SaveMedia.md` §7.3 [A], is to be confirmed against Sega's
  *Mega-CD BIOS Manual* at stage 10, when the BIOS replacement writes the same format.
- **A state marking its memories as kept apart**, as Diamond's `SHRD` block does: a state made with the backup RAM
  shared and loaded per game puts the shared memory's contents into the game's own `.brm`. Nothing shared is lost by it,
  and the game's own file is rolled back as any per-game load rolls it back; the mark is a refinement for the stage that
  next changes the Sega CD's state.
- **The format module** for `mcd.backup` and `mcd.ramcart`, and a formatted image for a new machine, both deferred to
  stage 10 (decided 2026-10-15).

## 62. The attachment interface built (2026-10-15)

*Decided 2026-10-15 (the tester's go):* `Nephrite_Attachments.md` built as designed, its §8 recommendations accepted,
after stage 9 and before stage 10. That page's §11 is the record, commit by commit; this section is its summary.

### 62.1 What changed

- **`src/interface.rs`**, the only module an attachment imports: `Attachment` (the 68000's map asked first, the VDP's
  transfer apart, `run_to`, the one sound stream, `EXPANSION`), `AttachmentState`, `AttachmentView`, `Board`, `Answer`,
  `Refusal`, `SpaceRef`, the ranges of space and processor ids, and `Bare`.
- **`Genesis<A = Bare>`** over `Wired<A>`, the Genesis's devices (`Hw`, not generic) with the attachment beside them,
  each configuration compiled apart and reached through one pointer; the machine's `Fitted` matched once per ABI call.
- **The Sega CD** on the three traits, its frontend's facts in `mcd/frontend.rs`, CD-DA given the mix's rate by
  `Board`; **the 32X's stub** in `src/s32x/`, holding the three memories a 32X image's machine had and its facts, and
  compiling no frame.
- **The J-Cart** decoded in the cartridge's arm; **the Genesis's files** in `src/genesis/`, its header out of
  `media.rs`, the debugger's composition in the machine; every item of the two trees narrowed to its tree and widened
  by the compiler only as far as it is used.
- **Two checks in the ordinary test sweep**: `boundary.rs`'s source check, and the three builds without the
  attachments (`--no-default-features`, with `mcd` alone, with `s32x` alone) compiled with their tests and examples.
- **`examples/digest.rs`** and two pinned layouts (a cartridge's, and now a disc's).

### 62.2 Held at every commit

The 71 digests (state, picture and sound at frames 300, 600 and 1,200 over the kit's Genesis sample, the bench's games,
the 240p disc as an image and as a sheet with mcd-verificator and without a boot ROM) equal to the baseline's; state
version 20 and both pinned layouts unchanged; mcd-verificator's eighteen and the crate's tests; the kit's sample 64 of
64 and the 240p disc with mcd-verificator as file 2 compliant (C1–C19, core ABI 1.8).

### 62.3 Measured

- **Cost**: the cartridge Genesis and the 240p disc within the runs' spread of the baseline (about 1%) at commits 3 and
  5 and at the end. A first form of the generic Genesis, the attachment lent beside a borrowed `Hw`, cost 1.3 to 2.4%
  and was not committed.
- **Size and build**: the `dist` library 1,445,920 bytes at the baseline, 1,631,832 at the end (+12.9%); a clean
  release build of the crate 4.9 s, then 6.1 to 6.2 s (+23 to 26%), under Q1's half. The Genesis-only library, for the
  record, 975,960 bytes, its kit sample 64 of 64.
- **Found on the way**: seven items no library code calls, which `pub` had hidden (four test-only, now `#[cfg(test)]`;
  three kept); and the Genesis-only library's media calls answering `NOT_SUPPORTED` where a machine with no drive must
  answer `NO_SUCH_DRIVE`, fixed.

### 62.4 For stage 10

Mode 1 is `Genesis<MegaCd>` with the cartridge in the slot and the Sega CD declining `$000000`–`$3FFFFF`; its battery
numbering is Q8's, consistent with §61 if mode 1 is a machine of the `md` system (`Nephrite_Attachments.md` §8); its
state needs the configuration in the header (Q5). The Sega CD's processor, `S68K` 2, arrives with its debugger step,
which adds `peek` and the `Watch` to the interface.


## 63. Stage 10: the open Sega CD BIOS, as far as its backup RAM's directory (2026-10-15)

*Written by a writer under the plan's §1.3 replacement writers' rule.* `Nephrite_OpenBios.md` is the stage's plan and
record, written before its code; this section is its summary. The stage stands open at step 10.4: the backup RAM's
directory waits for a specification made by a separate observer (that page's §5 Q2), and with it the gate's `_BURAM`
round trips and a new machine's formatted memory.
*Since 2026-10-16* the directory is built from the observer's `Nephrite_Bram.md` and the gate is passed but for two
open disputes (§63.5).

### 63.1 What was built

- **Beryl's 68000 assembler** (step 10.0, `Beryl_M68k.md` §8): every one of the 45,816 opcodes the 68000 decodes
  assembles back from the disassembler's text to its words.
- **The open BIOS** (`src/mcd/bios/`, assembled by `build.rs` into `mcd::bios::OPEN_BIOS`, 128 KiB): the main CPU's
  boot, its own screen with the drive's state and START for the tray, the jump table at `$FFFD00`, the hand-over to the
  initial program (D-45, D-46); the sub CPU's boot (TOC, system area, initial and system programs, the user calls),
  `_CDBIOS` over the drive and the CDC (D-41, D-44), the status table, the fader, the LEDs; `_BURAM`'s BRMINIT,
  BRMSTAT and BRMFORMAT from the footer document; `$0368` (D-42) and RTS for every other routine of `$0200`-`$1FFF`
  (D-43). The interface's numbers and slots come from the 240p suite's `cdbios.asm`, cleared by a screener, each
  confirmed by a program's call (D-41: 21 numbers so far).
- **The replacement**: with no player's BIOS, a disc runs on the open one; machine info names the firmware
  `Replacement`, and its words are `Accuracy` (`mcd/frontend.rs`, `BIOS_COST`, as `Nephrite_OpenBios.md` §4 drafts them,
  with the backup RAM's gap stated; reordered for the firmware page's footer in its §17). With the player's image, theirs
  is used.
- **Mode 1** (§49.6 Q4): a Create setting, hidden, puts the Sega CD beneath a cartridge; an `md` machine with the
  backup RAM as file 11 and a state of its own system byte (D-47).
- **The system pack** claims `.cue` and `.chd` for the Sega CD, as does the core.

### 63.2 The gate, as it stands

| Item | Result |
|---|---|
| The 240p Sega CD suite on the open BIOS | boots to its main menu, the picture equal byte for byte to the player's BIOS's; its sound test's CD-DA plays as on the player's |
| mcd-verificator in mode 1 | 18 of 18 on the open BIOS at `$400000` |
| mcd-verificator's mode 2 run | unchanged, 18 of 18 |
| The five discs against the player's BIOS, games only | Chuck Rock II and Mickey Mania run as on the player's; Sonic CD reaches its title with its music once START passes its "RAM is full" message; Lunar stops at its save data message (`Nephrite_OpenBios.md` §12) |
| `_BURAM` round trips | not yet: the directory's calls wait for the specification |
| The Genesis unchanged | the 71 digests equal the baseline's; the kit's sample 64 of 64 |
| The kit on the Sega CD | the 240p disc with mcd-verificator, the 240p sheet and the five discs on the open BIOS: compliant |
| Cost | 1.3 to 2.7 ms a frame on the open BIOS, as on the player's (`Nephrite_OpenBios.md` §13) |

### 63.3 Defects of the hardware model it found

A seek's travel count loaded as 1 where it was saved as 0 (C7 on Chuck Rock II, `Nephrite_OpenBios.md` §12), and the
PCM chip's waiting samples converted to master clocks again each frame (D-48). Both fixed with tests; neither changed a
digest.

### 63.4 Measured (2026-10-15)

The crate's 183 tests (with `EMUSEN_NEPHRITE_ROMS` and `EMUSEN_NEPHRITE_DISCS`), and those of the builds without each
attachment; `beryl-m68k`'s 26 with its `asm` feature and 21 without; WiseMan's Nephrite, Genesis, Registration,
SaveMedia and routing tests, 98.

### 63.5 Step 10.4 and the gate (2026-10-16)

- **`_BURAM`** (`bios/bram.s`) from `Nephrite_Bram.md` alone: all nine calls, the block code, the footer's checks, the
  directory's placement, counts and compression; the choices where the specification is open are D-49 to D-52 and
  D-56 (`Nephrite_OpenBios.md` §14). `_mburam` is not placed: its address is in no document the writer may read (D-55).
- **The format module** `SegaCdBramFormat` in the Genesis system pack for `mcd.backup` and `mcd.ramcart`; the Sega CD's
  media now default to *Shared*, and a new machine's backup RAM with no file is given the module's formatted blank.
- **The gate**: the round trips pass in both directions with Sonic CD and Lunar, judged by the games; Lunar reaches its
  intro (after D-53: a sub CPU read now carries the sector's header first) and Sonic CD saves; the 71 digests equal the
  baseline's; the kit compliant on the sample and the discs. Open: Sonic CD's film is black (D-54, two boot ROM routines
  no document describes) and the cartridge is unreachable (D-55). The replacement's words lose their backup RAM clause.
- **Measured**: the crate's 192 tests (9 new for `_BURAM`), 190 and 144 without each attachment; `beryl-m68k`'s 26;
  WiseMan's in the blast radius (`Nephrite_OpenBios.md` §15).
- **Offered** (`Nephrite_Plan.md` §7, decided 2026-10-16): the `mcd` system's development mark is removed from the
  core's info; `SegaCdRoutingTests` route the tester's library, PlayStation discs to Diamond, Sega CD discs to
  Nephrite, cartridges to the Genesis, a CHD by its folder's name (`Nephrite_OpenBios.md` §15).

## 64. Loading waits on the player's BIOS, and the drive's track jump (2026-10-16)

**Observations of the player's BIOS at work: excluded from the open BIOS's writers.** This section, and D-57 beyond
its heading, record what the drive was asked for while a player's BIOS ran, and recordings of a console's drive made
under a BIOS. Whoever writes or reviews the open BIOS (`Nephrite_OpenBios.md`) does not read them (that page's §9).

### 64.1 What the tester reported

On the first day the Sega CD was offered, the tester played on the handheld with the player's own BIOS installed (the
American and the European) and reported films that did not load, or loaded after about twenty seconds; then, more
precisely, that Chuck Rock II waited long right after its boot and again after START on its title screen, that Sonic
CD's films did the same, and that after START on Sonic CD's "New Game" no film played at all. The handheld's sessions
ran at a steady 60 frames a second at about 2 ms a frame, so a wait of twenty seconds was emulated time or something
that never came.

### 64.2 How the waits were looked at

`examples/waits.rs` runs a disc headless on a boot ROM given as a file, with a formatted backup RAM, and prints from
the devices alone: each drive command the sub CPU sends but the idle and report requests (from its writes to
`$FF8042`-`$FF804B`), each change of the drive's status nibble as the sub CPU reads it (`$FF8038`) with the head's
sector, the levels 4 and 5 the sub CPU took between lines, and each frame where the picture turns one colour or
shows again. A wait is read as frames from the request (the picture going blank after a scene, or a press) to the next
picture shown. `examples/disc.rs` kept the pictures, every 20 to 60 frames, to name the scenes. Nothing of the boot
ROM's code, memory or registers was read or printed; the player's BIOS was given as a file, the American M1 V1.10
for the four American discs and the European M1 V1.00 for Sonic CD (Europe), each on the copies under
`~/.cache/emusen/probe/nephrite/discs/`, measured on 2026-10-16.

### 64.3 What the drive was asked for

Every long wait had the same shape. Before reading far across the disc, the player's BIOS sent **command A** with a
count in nibbles 4-7 and a direction in nibble 3, then the play or seek to its target. Nephrite's drive took command
A for no command at all (`cdd.rs` matched it to nothing), so its status stayed what it was:

- **sent while paused** (status 4), as at Sonic CD's boot, Lunar's and Mickey Mania's intros, the BIOS sent its play
  one exchange later and the drive covered the whole distance by its coarse travel, about 0.9√sectors exchanges
  (§51.1): 4.2 to 5.8 s for a jump across the disc.
- **sent while playing** (status 1), as after Sonic CD's title, on START at Sonic CD's "New Game" and Chuck Rock II's
  title, and at Chuck Rock II's boot, the BIOS waited, the drive went on playing CD-DA with no data read and no level
  5, level 4 each exchange and an idle command answering each, until **600 frames** had passed; then it sent stop,
  waited another 600 frames, sent pause, and sent command A again, which now came while paused, and the play after
  it. **Twice 600 frames is the twenty seconds the tester saw.**

### 64.4 What a console's drive does with command A

krikzz's `notes/cdd-cmd-info.txt` names it only "0A track jump ?". mcd-verificator's `cd-log/sonic-bios/` holds
recordings of a console's drive under a BIOS, one status and one command per exchange; three carry command A
(`boot-log.txt` lines 897 and 1475, `stop-to-play (track 3).txt` line 152):

| From | Command | Exchanges `AF` | Landed (paused, report 0) | Then asked for |
|---|---|---|---|---|
| 00:02:21 | `A0.00020B` (523 tracks out) | 8 | 01:29:26 | play 02:18:35 |
| 00:01:66 | `A0.001313` (4,883 tracks out) | 28 | 13:17:49 | play 13:20:25 |
| 02:16:64 | `A0.0033EC` (13,292 tracks out) | 54 | 47:26:29 | play 45:59:50 |

The drive answers `AF.000000` with its flags, busy with no report, for a time that grows with the count; at the last
of those exchanges the flags are the landing place's (`15` where it landed in audio); then it reports paused at the
landing, and the BIOS sends its play. The count is hexadecimal, not BCD (`33EC` = 13,292). The direction is read from
the runs on Nephrite, where the BIOS sent `A0.0F32B7` and `A0.0F12BF` with the head past its target, and from the count
fitting the distance both ways (§64.5); no recording has a jump inward.

The time and the landing follow a spiral read at constant linear velocity, the Mega-CD Disc Format Specifications
fixing game discs at 1.25 m/s "to improve the accuracy of track jump access": tracks from the centre at sector *s*
are √(R² + K·s), and a jump of *n* tracks lands where that is *n* more or fewer. A least-squares fit to the three
landings gives R = 18,300 and K = 3,385 (the nominal 1.6 µm pitch, 25 mm start and 1.25 m/s give 15,625 and 3,316; the
drive's count is not exactly a physical track), landing within 31 sectors of the console on the long jump and 227 on
the middle one, and 789 short on the 523-track jump. The busy exchanges fit √n / 2 − 4.5 (7, 30 and 53 for the
console's 8, 28 and 54). Both are measured against the recordings, three points from one disc; a fourth jump, or a
disc of another pitch, would test them.

The documents' figures for the whole access: the Mega-CD Outline gives the drive's access time as at most 1.5 s and
0.8 s on average, and its §4-3 a maximum seek of 1.5 s; Sega Retro's specification page 800 ms and 150 KB/s at 1×. The
console's long access above, jump and play together, took 54 + 85 exchanges, 1.85 s; Nephrite's seek of the same 6,504
sectors after the jump takes 96 (§51.1's model, from the LL logs), 13% longer.

### 64.5 What changed

`cdd.rs` takes command A while ready: the head's track count from the spiral, the landing from the count and the
direction, busy exchanges from the fit, status A with no report and the place's flags, the landing's on the last, then
paused at the landing looping as a pause does. The drive's mode `Jumping` is saved as tag 10 of the drive's mode words;
the layout is unchanged and no state of an earlier build holds the tag. D-57 records the rule.

`a_track_jump_is_busy_then_pauses_where_it_lands` replays the long recorded jump: it failed before the change (no
exchange busy, the head unmoved) and passes after it, with the landing within 150 sectors of 47:26:29 and a jump of the
same count inward landing where it began.

### 64.6 The waits, before and after, on the player's BIOS

Frames from the request to the next picture (60 a second; Sonic CD (Europe) 50):

| Disc | Moment | Before | After | What the wait is now |
|---|---|---|---|---|
| Sonic CD (USA) | boot, after Sega's logo, to the title | 326 (5.4 s) | 326 (5.4 s) | the game's own files read; the title music's access 338 → 71 frames |
| Sonic CD (USA) | title left alone, to the opening film | 1,781 (29.7 s) | 270 (4.5 s) | fade, a jump and seek of 1.33 s, the film's first sectors |
| Sonic CD (USA) | START on "New Game", to Palmtree Panic | 1,992 (33.2 s) | 476 (7.9 s) | two accesses of 1.37 and 1.53 s, 247 sectors read |
| Sonic CD (Europe) | boot to the title | 232 (4.6 s) | 230 (4.6 s) | unchanged |
| Sonic CD (Europe) | title left alone, to the opening film | 1,474 (29.5 s) | 227 (4.5 s) | as the American disc |
| Chuck Rock II | boot, after Sega's logo, to Virgin's | 1,633 (27.2 s) | 357 (6.0 s) | a jump and seek of 1.57 s, then the game's reads |
| Chuck Rock II | START in the intro, to the title | 513 (8.6 s) | 486 (8.1 s) | the game's reads; the title music's access 250 → 107 frames |
| Chuck Rock II | START on the title, to the stage's card | 1,583 (26.4 s) | 204 (3.4 s) | a jump and seek of 1.65 s, then reads |
| Lunar | boot, after Sega's logo, to Working Designs' | 1,119 (18.6 s) | 707 (11.8 s) | three jumps and seeks of 1.37 to 1.55 s, a pause of 169 frames that is the game's own, reads |
| Lunar | each scene change of the intro | 136 | 136 | unchanged; no long access |
| Mickey Mania | boot, after Sega's logo, to the first picture | 1,071 (17.9 s) | 1,071 (17.9 s) | unchanged: fourteen short seeks and reads near the disc's start and a stop of 102 frames the game asks for |
| Mickey Mania | the intro's scenes | 429, 514, 762 | 167, 271, 526 | each had one access across the disc, 5.2 to 5.6 s, now 1.13 to 1.27 s |

Every access across more than 20,000 sectors, a jump and its seek together, took 2.4 to 5.8 s before and takes 1.13
to 1.78 s now, against the documents' 1.5 s at most and the console's recorded 1.85 s. What remains of each wait is
reads at 75 sectors a second and short seeks, both §51.1's and §52.1's rules, and pauses the games ask for. The level
5 count during reads equals the sectors read: the CDC's decoder interrupt each sector, its DMA (16 sub clocks a word,
D-34) finishing a sector's 2,048 bytes in about 1.3 ms of the sector's 13.3 ms, so the CDC never bounded a wait.

### 64.7 Sonic CD's "New Game" plays no film, on the player's BIOS as on the console

After START on "New Game" the game asks for `R11A__.MMD` (sector 1,100), `SNCBNK1B.BIN` (171) and the first stage's
CD-DA (60,025), before and after the change alike; it never asks for `OPN.STM` (sectors 10,410 to 17,253), the film's
stream, which it reads only after its title has been left alone. On the console the boot's drive traffic
(`boot-log.txt`) is Nephrite's command for command through to the title's music, but for one stop more at its start
and the jumps' counts (`20B` and `33EC` there, `20A` and `328F` here, from heads a few sectors apart): it reads
02:18:35 to 02:18:50 and goes to the music, never reaching the film's stream at 02:18:60, so on the console too the
game goes to its title with no film. What the tester saw after "New Game" was the 33 s of black before the stage, with no film in it,
which the change makes 7.9 s; the film itself plays after the title, and plays on the player's BIOS (it is black only
on Nephrite's own, D-54).

### 64.8 For the tester

Whether the open BIOS's writer may read `cdd.rs`'s command A (`Cdd::jump` and the `Jumping` arm), which states the
drive's rule in hardware terms but was found under a BIOS, is not decided here; until it is, it is listed with this
section as closed to that writer (`Nephrite_OpenBios.md` §9). Nephrite's own BIOS does not send command A, and the five
discs' drive traffic on it is unchanged, line for line, over 4,000 frames.

## 65. Which BIOS a session runs on, and the Sega CD on the Genesis's screen (2026-10-16)

- **Which BIOS.** The host loads each file `firmware_for` names from the firmware folder by name and exact size
  (`FirmwareLibrary.TryLoad`, `DataStore.Firmware`, which Galaxia resolves to the sandbox's `home/Firmware`) and hands
  them to create as files 2 to 4 (`CoreEngine.LoadRom`). For a sheet or a CHD `firmware_for` names all three markets'
  BIOS, for an image its market's. Create boots the market's file, else another market's given, in the order U, E, J,
  else Nephrite's own. Machine info had named the market's file as `Replacement` when another market's had booted;
  it now names the file that booted (`chosen_bios`, `booted_bios`), shared by the disc's machine, the cartridge's
  mode 1 and machine info, so the choice has one owner. `CoreEngine` writes a line per firmware entry at load,
  `[Firmware] The American Sega CD's BIOS (131,072 bytes): the player's bios_CD_U.bin` or `...: Nephrite's own
  replacement`, into the game's `general.log` (`EmuSen_Firmware.md` §3.1). With the player's American and European
  files in the folder under those names, a session on the five discs boots the player's: `NephriteFirmwareTests`.
- **The screen.** The Sega CD's system pack says it is attached to the Genesis (`SystemEntry.AttachedTo`, the 32X's
  too), and the frontend gives it the Genesis's television filters, the CRT's Genesis row and parameters, overscan and
  the Genesis's pads from that one fact (`EmuSen_Serenity.md` §3.12).
- **Measured** (2026-10-16): the crate's 193 tests; the 71 digests of `Nephrite_Attachments.md` §11.1 equal the
  baseline's; the five discs' drive traffic on the open BIOS unchanged; the conformance kit compliant on its 64-image
  Genesis sample, on the 240p disc with mcd-verificator and on Sonic CD (USA) with the player's BIOS over 2,400 frames.

## 66. The frame log's byte order for the Genesis family (2026-10-18)

The systems table's `byte_order` (`EmuSen_Backend.md` §18.9) tells `CoreEngine` in which order to assemble a frame
log's value of two or four bytes from a space. It had been left unstated for the three Genesis systems, which a reader
takes as little-endian. Decided 2026-10-18 by the tester: read how Nephrite lays out a word in each space it lists,
and set it with a test.

### 66.1 What each space holds

Every space is a byte array read through `space_read` byte for byte (`src/v1.rs:554–567`); the question is only how
the core stores a word in it. Paths are under the crate's `src/`.

| System | Space | A 16-bit value at an even address | Where |
|---|---|---|---|
| all | `M68KBUS` (view) | big-end first: each byte is the cartridge's, the 68000's RAM's or the Z80's RAM's as stored | `machine.rs:388–397`, `genesis/debugger.rs:100–108` |
| all | `WRAM` | big-end first: the 68000's word read takes the high byte from the even address, its word write puts it there | `genesis/mod.rs:607`, `:647–648` |
| all | `ROM` | big-end first: the image's bytes in the cartridge's order, a copier's interleaved blocks put back in it at load | `genesis/cart.rs:178–183`, `:214–216`; `media.rs:68–98` |
| all | `VRAM` | big-end first: a data-port word's high byte goes to the even address (an odd address swaps the two, as the VDP does), and the read takes the word the same way | `genesis/vdp.rs:622–629`, `:678–680`, `:689` |
| all | `CRAM`, `VSRAM` | big-end first, written and read as VRAM is | `genesis/vdp.rs:642–643`, `:651–652` |
| all | `SRAM` | big-end first on a board that wires both lanes; one byte per word on a board that wires one, so a value the game spreads over consecutive words lands high byte first | `genesis/cart.rs:164–176`, `:214–216` |
| all | `Z80RAM` | **the Z80's order, little-end first**: one byte per address, and what the Z80 stores as a word it stores low byte first; a 68000 word write lands its high byte alone | `genesis/mod.rs:739`, `:751`, `:643` |
| all | `Z80BUS` (view) | the Z80's RAM below `$4000`, little-end first as above; through the bank window, `$8000` up, the 68000's space, big-end first | `genesis/debugger.rs:112–121` |
| Sega CD | `PRGRAM` | big-end first: the sub CPU's reads and the CDC's DMA both put the high byte at the even address | `mcd/mod.rs:841`, `:597`, `:307` |
| Sega CD | `WORDRAM` | big-end first, in 2M order; in 1M mode a bank's word *n* is the space's word 2*n* + *b*, so its two bytes stay together while consecutive words of one bank are four bytes apart | `mcd/word.rs:6`, `:41–44`; `mcd/mod.rs:842–849`, `:599–601` |
| Sega CD | `PCMRAM` | one byte per address, the RF5C164's 8-bit samples; no word is stored | `mcd/mod.rs:595`, `:935`, `:943` |
| Sega CD | `BRAM`, `CARTRAM` | one byte per word, on the odd lane; the format's 16-bit fields are big-end first (`Nephrite_Bram.md` §3, §5) | `mcd/mod.rs:851`, `:907`, `:199`, `:211` |
| 32X | `SDRAM`, `FRAMEBUFFER`, `PALETTE` | not decided by the core: the stub's memories are zeroed and no processor reads or writes them (§62; `Nephrite_Attachments.md` §3.9). The SH-2 and the 68000 that share them are both big-endian | `s32x/mod.rs:11`, `:36–45`, `:82–95` |

### 66.2 The decision

The three data files and the three pack literals say `big`. Every space a 68000 or a device on its side owns stores
a word big-end first, and those are the spaces a player logs: the 68000's RAM (the cheat space), the cartridge, the
VDP's three memories, the save RAM, the Sega CD's program and word RAM. The 32X's memories will hold SH-2 and 68000
words, so stages 11 and 12 are to store them big-end first and keep this field true.

**The spaces differ.** `Z80RAM`, and `Z80BUS` below `$8000`, hold the Z80's own words low byte first. A system has one
`byte_order`, so a frame log of two bytes there now reads a Z80 pointer with its bytes swapped, where before this
change every 68000 value was swapped instead. That trade favours the spaces a player logs; it is not correct for
the Z80's. The 8-bit spaces (`PCMRAM`, `BRAM`, `CARTRAM`) are unaffected at width 1, the width their contents have.

**The smallest correct form, proposed and not built.** The core owns the fact of how it lays out each space, so the
fact belongs beside the space in machine info: a byte order per space in the space list, which `CoreEngine` would
read first and fall back to the table's `byte_order` where a core does not say. That is an addition to the core ABI
(a minor), and Nephrite would state `Z80RAM` and `Z80BUS` little-endian. `Z80BUS`'s bank window would still read
big-endian values the wrong way through that space; they are the 68000's and are read right through `M68KBUS`. A
smaller form, a per-space exception in the systems table's data file, needs no ABI change, but it would tie a
system's data to one core's space names, which a second Genesis core need not share; it is not recommended.
*Chosen by the tester on 2026-10-18 and built the same day as core ABI 1.14 (`EmuSen_CoreAPI.md` §6.39): Nephrite
states `Z80RAM` and `Z80BUS` little, §66.5.*

### 66.3 The test

`NephriteTests.The_frame_log_reads_a_genesis_value_big_end_first` logs a known value through `CoreEngine`'s frame
log over one frame on a synthetic image of each system (`SyntheticMdRom`): the header's `SE` and `SEGA` at `$100` of
`ROM`, two and four bytes, on a Genesis and a 32X cartridge; `$12 $34` written into `WRAM` on a Genesis cartridge and
a Sega CD disc, into the Sega CD's `PRGRAM` and the 32X's `SDRAM`. All seven cases failed before the change, the frame
log giving `0x4553`, `0x41474553` and `0x3412`, and pass after it.

### 66.4 Measured (2026-10-18)

- **WiseMan**, with `EMUSEN_SYSTEMS_NATIVE` unset: `NephriteTests` (its seven new cases among them),
  `SystemTableParityTests`, `GoldenCorpusTests` and `CoreRegistrationEquivalence`, 129 of 129. With
  `EMUSEN_SYSTEMS_NATIVE=0`, the pack literals' path: `SystemTableParityTests` and the seven new cases, 15 of 15. No
  golden was re-recorded: the corpus's frame-log case runs on its fake target, not on a core.
- **`emusen-systems` and `emusen-platform`**, `--profile test-checked`: 24, and 31 with the four of `dianash_abi`.
  `a_byte_order_is_big_little_or_unstated` now finds the Genesis `big` and still reads an unstated order as none.

### 66.5 The Z80's spaces on core ABI 1.14 (2026-10-18)

The tester chose §66.2's machine-info form on 2026-10-18, and it was built as core ABI 1.14: a space in machine info
may carry `byte_order`, `big` or `little`, absent meaning the system's (`EmuSen_CoreAPI.md` §6.39). `CoreEngine`'s
frame log reads a space's own order first and the systems table's after. Nephrite states two spaces, through
`genesis/debugger.rs`'s `byte_order`, which `v1.rs`'s machine info reads beside `reports_stores`; every other space
states nothing and keeps the table's `big`, so §66.2's decision for the 68000's side stands unchanged.

**`Z80RAM`: little.** §66.1's row: the space is the Z80's 8 KiB, one byte an address, and a word the Z80 stores there
it stores low byte first.

**`Z80BUS`: little, with the bank window as a documented exception.** The space is the Z80's 64 KiB as a debugger
reads it (`peek_z80`, `genesis/debugger.rs:112–121`), in three parts:

| Addresses | What reads there | Stored words |
|---|---|---|
| `$0000`–`$3FFF` | the Z80's RAM, its 8 KiB twice | the Z80's, low byte first |
| `$4000`–`$7FFF` | the YM2612, the bank register, the PSG and the VDP, read as `$FF` | none |
| `$8000`–`$FFFF` | the bank window: 32 KiB of the 68000's space at the bank register's page (`$FF` over the Z80's own range) | the 68000's, high byte first, or data laid out for the Z80 |

The order is decided by what a frame log of a Z80-side value names. The Z80's own variables, its stack and the pointers
its program keeps are in its RAM, so most such entries name the first part, where `little` is right. Through the
window the Z80 reads the cartridge: mostly bytes (the DAC's 8-bit samples a sound driver streams, the commonest use of
the window), for which the order does not matter at width 1, and data a Z80 program reads as words with its own 16-bit
loads, which the game's author lays out low byte first for the Z80 even though it sits in the 68000's space [argued:
a Z80 load takes the low byte from the lower address wherever the bytes are]. A value that is the 68000's (a word the
68000's program wrote or reads) is the one that reads swapped through the window, and it reads right through
`M68KBUS` or `ROM`, the spaces a frame log of a 68000 value names. So `little` is the order of every word the Z80
itself reads as a word, and the exception is the 68000's words seen from the Z80's side.

**The converse exception** is `M68KBUS`'s window onto the Z80's RAM, `$A00000`–`$A03FFF` (`peek_m68k`,
`genesis/debugger.rs:100–108`): `M68KBUS` states no order and takes the table's `big`, so a Z80 word read there reads
swapped; it reads right through `Z80RAM`. Neither bus is split: each is one space because its processor sees one
address space, and its id, name and size are what the debugger, the cheats and saved frame logs key on.

**The test.** `NephriteTests.The_frame_log_reads_a_z80_word_little_end_first_as_the_z80_stores_it` writes the Z80's
word `0x1234` as the Z80 stores it, `$34` at `$1F00` and `$12` at `$1F01` of `Z80RAM`, and logs two bytes there over one
frame: from `Z80RAM` on a Genesis cartridge, a 32X cartridge and a Sega CD disc, and from `Z80BUS` on a Genesis
cartridge. On the tree without the change (the test alone, against `WiseMan` at `08812dde`'s code), all four failed,
the frame log giving `0x3412` with both bytes in place; with it all four pass, and §66.3's seven 68000-side cases pass
unchanged. The crate's `the_stub_reports_each_system_and_a_blank_picture` holds the two stated orders and no other.

**Measured (2026-10-18).** WiseMan's blast radius, 187 of 187 (`EmuSen_Backend.md` §25); the crate's `v1::` tests, 21
of 21 (`--profile test-checked`); the kit's corpus on `genesis-sample.txt`, 64 of 64 images passing as before, each
report the same as at 1.13 but for the minor's number and C14's note, which now ends "2 space(s) state their byte
order".

## 67. Mode 4 drawn by Beryl's shared composition (2026-10-19)

Nephrite's Q9 (decided 2026-10-04) moved mode 4 into a shared crate when the Master System's core began, and
`Endou_Plan.md` §3.3 (decided 2026-10-19) placed it: `beryl-sms-vdp`'s `compose` module, built at Endou's stage 3, with
a `Variant` for each chip. At Endou's stage 4 Nephrite moved onto it, gated on its D-7 tests and its corpus digests
staying exactly unchanged (`Endou_Plan.md` §7, P4).

**What moved.** `render.rs`'s `render_mode4`, about a hundred lines of sprite pass, background and mix, is now a call of
`compose::find_sprites`, `sprite_pixels` and `line` with `Variant::GENESIS`, reading VRAM through `Mode4Vram`, which
applies D-7's map (`vram4_address`), and colouring CRAM indices at the board's levels as before (`rgb4`). What a mode 4
line does to the VDP's state stays Nephrite's: the sprite pass runs only with the display on, and its overflow and
collision set the status flags as they did. `Variant::GENESIS` carries what MacDonald's §13 says a composition sees (no
zoom, no taller displays, no masks) and two rules the old code had that the Master System's chips do not: the fine
scroll's first pixels are the wrapped column (§15.2's argued loss), and the right columns' vertical lock starts at
pixel 192 rather than at the column counter's 24th column (`Beryl_SmsVdp.md` §9.4). The second was kept so that the move
changed nothing; whether the 315-5313 locks by the column counter is unmeasured, and the board could say. A new
dependency, `beryl-sms-vdp`, said here as the project's rules ask.

**Measured unchanged (2026-10-19)** [M]:

- **The two renderers drew the same lines.** Before the old code was deleted, a test built both into one library and
  drew 200,000 lines from random VRAM, CRAM and registers in mode 4 (the sprite table crowded about the line on half,
  the display off on a quarter, a fine scroll with the right columns' lock on 72,550), comparing each line's RGBA and
  the overflow and collision flags: all 200,000 equal. The test was not kept; it held two copies of one renderer.
- **The crate's tests**: 194 of 194 on `--profile test-checked`, D-7's `mode_4_writes_land_where_the_board_put_them`
  and `mode_4_shows_the_master_system_colours` among them.
- **The kit's corpus** (`kit/genesis-sample.txt`, 64 images, 600 frames), the base's release library (`bac87b11`)
  against this one's: 64 of 64 pass on both, and **the 64 image reports are identical**, frame, audio and state digests
  included; the summaries differ only in the library's path, the timings, the memory figures and C13's count of rounds.
- **The open Sega CD BIOS** built by each is byte-identical (`open_bios.bin` and its symbols, SHA-256 `fb4a1877…` and
  `9a61aab0…`).
- **WiseMan's Nephrite classes** with the corpus and the games: 56 of 56 (`NephriteTests` 22).

A fault seeded in the shared composition, the Genesis's fine scroll blanked as the Master System's is, was caught by
the crate's own test and by none of Nephrite's crate tests: D-7's pictures leave the left five pixels out, where the
board shows garbage (`Endou_Native.md` §11.12).
