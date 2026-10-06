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

- VRAM's fetch lead of 8 pixels is weakly bracketed (4 to 16); the display bit's return fits 0.56 of the board's
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

- **The mix's levels**: from the references (§18.3), the four of which put the PSG's square at 0.214 to 0.282 of the
  DAC's swing; Nephrite's 0.251 is their median. No recording of a model 1 is in the corpus.
- **The analogue path**: none yet. The PSG's output is unipolar, as the chip's is, and no filter or coupling
  capacitor removes its offset; the model 1 filter is a later step of the stage.
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
shift and the key code (page 12). A block quoting MAME's source in a member's post (page 8) was displayed and not
used. Nemesis's page 11 blocks building the two tables and converting the exponent turned out to be his emulator's
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
  configured, the lines listed in D-16. Two lines of the board's mix were printed by the search; they are noted as
  seen and used for nothing. No other RTL file was opened.
- **Topic 386, from the copy saved at step 2**: Nemesis's Table 2 on page 8, its rows read with every identifier
  in the block masked (D-18). **The same search printed, masked the same way, the numeric rows of two blocks in C
  array form on pages 32 and 33**, eight values to a row, which look like tables of vibrato depth and sit in blocks
  that carry an emulator's identifiers. They were not sought, are used for nothing, and are recorded here as
  displayed: the step that builds the LFO must not take its vibrato table from memory of them, and has the board to
  measure it on instead.
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

Moving Nephrite's picture to the board's place against the 68000's start would remove the choice, and would move
every game's frames against Genesis Plus GX at the corpus anchors and every stage 4 picture test by a frame. It is
not done here.

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

- The three items of §26.3, **stage 4's frame offset** (§25.5), **the test read** and **D-2**'s few long RAM waits.
- **Next**, in the plan's order: the multiplexed output and its ladder effect, the YM3438 of model 2 as a setting,
  and the model 1 filter.
