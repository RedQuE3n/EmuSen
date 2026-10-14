# Diamond_Native — the build record

*Started 2026-10-11, at stage 1.* `Diamond_Plan.md` says what Diamond is to be and how it is graded; this page records
what was built, stage by stage, what was measured, and where the plan turned out to be wrong. The plan's marks are
kept: **[M]** measured on 2026-10-11 unless another date is given, on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10,
Rust 1.98.1), with the tool named beside it; **[R]** read from a source, named; **[A]** argued, with no measurement
behind it; **[C]** a choice, either a rule set here or a recommendation awaiting the tester; **P1–P12** the plan's
predictions, retired where a measurement reaches them.

The processors' records are their crates' own pages in `../BlackMoon-HW/`. The references, the documents, the corpus
and the disputes log are recorded in the plan's §1.6 and §3.1 and in `Diamond_Disputes.md`, which this page does not
repeat.

---

## 1. Stage 1, its first part (2026-10-11)

The plan's §7 row 1, apart from pinning the references, measuring the probe's three cores and fetching the documents
and test programs, which are recorded in the plan's §1.6 and §3.1. What this part built:

- **The crate** `diamond`, `EmuSen/Cores/Sony/Diamond - PlayStation/`, on the core ABI v1 from its first commit, a stub
  that recognises a PS-EXE or a disc and emulates nothing (§2).
- **BlackMoon Hardware**, `EmuSen/Cores/Sony/BlackMoon-HW/`, with `blackmoon-r3000` and `blackmoon-gte` as skeletons
  with Beryl's interface, and the r3000 single-step harness with its controls in the first (§3; `BlackMoon-HW/README.md`).
- **The system pack** `psx` in `DianaSh/Sys/Systems/Psx/` (§2.4).
- **The registration**: `RustCores.props` names the crate with `CoreAbi 1` and `InDevelopment` true, its system entry
  says `development: true`, and `rust-cores.yml` builds, tests, export-checks and sidecars it on the four platforms
  and tests the BlackMoon crates with the shared ones (§4).
- **WiseMan's builders**: a PS-EXE builder and a synthetic disc builder (§5), and `DiamondTests` through the v1 adapter,
  discovery, the generic test-ROM runner and the conformance kit.
- **The gate**: the kit's C1–C15 on a synthetic PS-EXE and a synthetic disc, and the r3000 replay controls (§6).

It wrote no emulation rule. The protocol of the plan's §1.3 applied throughout: no emulator source and no firmware or
firmware listing was read. **Read for this part** [R]: psx-spx at `6d7d1bc` ("CPU Specifications" for the register
names and coprocessor 0's table, "Geometry Transformation Engine (GTE)" for the GTE's register names, "CDROM File
Formats" for the PS-EXE header, "CDROM Format" for the system area, the licence string, the volume descriptors and
the sector encodings, "Controllers and Memory Cards" for the pad report, "GPU Versions" by the plan's citation);
ECMA-130 (2nd edition, 1996) §14 and Annexes A and B; ECMA-119 for the directory and path table records; IDT's R30xx
manual for the register set; SingleStepTests r3000's README and `transcode_json.py`; EmuSen's own pages and code as
cited.

---

## 2. The crate

### 2.1 On the core ABI v1

- **`Core` and `core_exports!(Machine;)`**, with no optional group: `CAPABILITIES` is 0. Each capability is claimed in
  the stage that implements it, as Nephrite's were.
- **Core info**: id `diamond`, name and display name `Diamond` (no "(Rust)" suffix, there being no C# engine to tell it
  from), one system `psx` with its extensions (§2.4), regions NTSC and PAL, the two controllers of the plan's §4.2, its
  firmware (§2.5) and `development: true` (§4).
- **Create.** An image is read by `media.rs` (§2.2). Anything that is neither a PS-EXE nor a disc is refused with −9,
  a PS-EXE shorter than its header says with −10, each with words; the machine is otherwise created. File 0, the
  battery, is accepted and unused (no battery is declared before the cards, stage 6); files 1 to 3 are the BIOS
  images (§2.5), and one of another size than 524,288 bytes is `BAD_FILE`; a higher number is `BAD_FILE`.
- **Machine info**: the system `psx` and the region (§2.3), the frame rate, a picture of 320 by 240 lines NTSC or 256
  PAL with a maximum of 640 by 512 and a 4:3 aspect, an audio rate of 44,100 Hz, each port's controller from the
  settings (§2.6), the spaces (§2.7), **the processors main first: `R3000` (id 0, 32-bit PC, the 45 registers of
  `blackmoon_r3000::DEBUG_REGISTERS`) then `GTE` (id 1, no PC, its 64 registers by psx-spx's names)**, no battery,
  the state format `DMND` version 1, `skip_rendering_state_neutral` false (the plan's §5.4: VRAM is machine state the
  CPU reads back, so the claim is not made even while it would hold), and the BIOS file of the machine's market as
  `file` when given and `absent` otherwise.
- **A frame** advances the frame count and queues its silence; the picture is opaque black; the pads' bits are kept
  per port with the `changed` mask and are in no state.

### 2.2 The image, read from its bytes

`media.rs`, from psx-spx [R]; recognition only, nothing is loaded:

- **A PS-EXE** begins with `PS-X EXE`; its header is `800h` bytes; the word at `1Ch` is the size that follows it, and
  the initial PC and the load address are at `10h` and `18h`. An image shorter than the header and that size is
  refused; bytes past it are padding and accepted. The market is the header's ASCII marker at `4Ch` ("…for Japan
  area", "…for North America area", "…for Europe area"), and none when the marker is absent, as homebrew often leaves
  it (psx-spx: the BIOS does not check it).
- **A disc** has the licence string's first line, `"          Licensed  by          "`, at the start of sector 4's
  user data. Two image forms are read: 2,048-byte user sectors (an `.iso`), the line at `2000h`; and raw 2,352-byte
  sectors (a track's `.bin`), told by the sync pattern at the start of the image and of sector 4, the user data 16
  bytes into the sector in mode 1 and 24 in mode 2. The market is the second line: Sony's company name followed by
  `Inc.` (Japan), `Amer…` (the Americas) or `Euro…` (Europe).
- **Not read at stage 1** [C]: `SYSTEM.CNF` and the volume descriptors, which the plan's §4.1 uses for the serial; a
  disc set's other tracks; a disc without the licence text, which the BIOS of a Japanese console refuses (psx-spx) and
  which some homebrew discs are. All belong to stage 7 with the disc crate and to routing by content
  (`EmuSen_CoreAPI_Extensions.md` §8), whose recogniser is the pack's.

### 2.3 The frame rates, the picture and the market (provisional)

- **The market** is the setting's when one is chosen, else the image's, else the Americas [C: a program that names no
  market runs on the console most homebrew is tested on; the plan's Q11 says only "the region from the disc"]. PAL
  exactly when the market is Europe.
- *Superseded at stage 3 (§11.5): the rates are 59.8261 and 49.7614 Hz, D-0's clocks over 3,412.5- and 3,405-clock
  lines, still provisional.* **The frame rate** [A, provisional until D-0]: psx-spx's video clock for each standard over a progressive frame,
  53,693,175 Hz over 263 lines of 3,413 clocks for NTSC (59.8173 Hz) and 53,203,425 Hz over 314 lines of 3,406 for PAL
  (49.7468 Hz), the nominal clocks the plan's §4.6 already uses for the CRT filter's phases. psx-spx's own frame-rate
  lines compute 59.29 and 49.76 Hz from 53.2224 MHz instead [R]; which an NTSC console's GPU runs on is D-0 in
  `Diamond_Disputes.md`, and when it is settled these two constants change, with the test that pins them to four
  decimal places. A stub's rate decides only its pacing and its count of silent samples.
- **The sound**: silence at 44,100 Hz, the SPU's rate (33.8688 MHz over 768, psx-spx), reported as a fixed rate. Frame
  *n* makes ⌊44,100·(n+1)/f⌋ − ⌊44,100·n/f⌋ stereo frames, with *f* the exact rational rate, so any run's samples sum
  to the rate exactly; they are a count, drained as zeros, with no buffer and no allocation per frame, bounded by
  `set_audio_limit` with the oldest dropped. Measured by the kit [M]: 884,692 samples over 600 NTSC frames, which is
  2 × ⌊44,100 × 600 / 59.8173…⌋.
- *Superseded at stage 4 (§12): the picture is the display area at its own size, 256 by 240 and blank after
  GP1(00h).* **The picture**: 320 wide, 240 lines NTSC and 256 PAL, black, the size fixed for a machine's life [C: the GPU's
  display registers decide it from stage 4].

### 2.4 The system pack, and who owns which fact

`PsxSystem`, `DianaSh/Sys/Systems/Psx/PsxSystem.cs`, listed in `SystemPacks.All`:

- **The entry**: id `psx`, "Sony PlayStation", console `PlayStation`, Sony, 1994; the libretro cheat folder
  `Sony - PlayStation` and OpenVGDB's system `PSX` (both read for the plan's §4.1); a cover aspect of 1.0 [A: a CD
  jewel case's front is about square]; ES-DE's `psx`, "Sony PlayStation" [R ES-DE's `es_systems.xml`, by the plan's
  citation]; `SelfDelimitingImages` false, since a disc records no length (a PS-EXE does, and the core refuses a
  truncated one anyway). OpenVGDB matches the PlayStation by serial, not by hash [R the plan's §4.1], so its hashing
  rule is the identity until the serial is read (stage 7). No cheat codec until stage 11.
- **Its extensions, staged as the plan's §11 Q9 decided** [C]:
  - **Opened at stage 1**: `.psexe` and `.psx`, the names PS-EXEs are distributed under. `.exe` was claimed at first
    and dropped the same week (§7): it waits for routing by content. The core's info lists the same two, a subset of the
    pack's as `EmuSen_CoreAPI_Extensions.md` §8.4 asks, and `DiamondTests` holds them equal. A test program named
    `.exe` (ps1-tests' are) still runs on Diamond when it is named directly, as the kit and the test-ROM runner do: the
    core tells a PS-EXE by its header, never by its name.
  - **Declared and not routed**: `PsxSystem.DiscExtensions`, `.cue`, `.chd`, `.iso`, `.m3u` and `.pbp`. They are kept
    out of the entry's list, which is what the host routes and shelves by, for two reasons. *Collision*: `.iso` is the
    Sega CD's today and `.cue` and `.chd` will be, and a second claimant of an extension is never reached; routing by
    content (`EmuSen_CoreAPI_Extensions.md` §8, decided 2026-10-11, the pack's recogniser over the system area) is
    what lets both claim them. *Image sets*: a `.cue` names its track files, which reach a core only through the
    image-set extension (§2 there, `EMUSEN_FILE_MEDIA`) and the disc crate, both Diamond's stage 7. `.m3u` follows at
    stage 11 with disc change, `.pbp` later and unencrypted only (the plan's §1.2). `.sbi` is read beside a disc and is
    never opened alone; `.ccd` and `.ecm` are later still.
  - **Why `.exe` waits** [A]: it is also every Windows program's extension, so a library folder holding programs would
    list them on the PlayStation's shelf once the console has one. Routing by content is the fix, the pack's recogniser
    telling a PS-EXE by its header; `.exe` returns with it.
- **The screen**: `PsxSystem.DisplayAspect`, 4:3, which the core reports as machine info's aspect; held equal by
  `DiamondTests`. The pack's entry record has no field for it, and none was added: the frontends read the aspect from
  the engine (`ICore.DisplayAspect`), and the constant states the console's fact beside the entry for the test.
- **The firmware**: `PsxSystem.Bios`, the three regional files with their size, held equal to the core's declared
  firmware by `DiamondTests`, as the extensions are. The core's info stays the runtime's source (the firmware page
  reads it).
- **`PlayerControllers`**: each player's pad from the two port settings, a port set to nothing having no player.

One owner, two copies: the Rust core cannot read the C# pack, so the facts the core must report itself (its
extensions, firmware and aspect) are written in both and held equal by a test, as Nephrite's extensions are.

### 2.5 Firmware

Every entry is `required: false` with `replacement: none` and its cost, so the kit's C4 creates every image with no
files and a frontend never prompts:

| File | Name, size | Market | Replacement words at stage 1 |
|---|---|---|---|
| 1 | `scph5500.bin`, 524,288 | Japan | `none`: "No PlayStation program runs yet, with this file or without it." Then: Diamond's open BIOS takes its place when it exists (kernel at stage 3, disc boot, files and cards at stage 10); a player's own BIOS stays the exact path |
| 2 | `scph5501.bin`, 524,288 | Americas | the same |
| 3 | `scph5502.bin`, 524,288 | Europe | the same |

*Since stage 3 (§11.10)*: the open kernel takes the BIOS's place and every entry is `effect: accuracy` with the plan's
§5.9 cost; machine info names `replacement` when it runs.

`firmware_for` names the image's market's BIOS (the Americas' for a program without a marker) and nothing for an image
it does not recognise; machine info names the machine's market's file. A player's file, when given, is held in the
read-only `BIOS` space and is not run. The names are the ones players' own dumps commonly carry (the plan's §5.9).
**The replacement is named but marked not yet available** [C]: `effect: none` is v1's word for "the game runs without
the chip", and no word says "a replacement that does not exist yet"; `none` with a cost whose first sentence says no
program runs is the honest reading, as Nephrite's stage 0 declared its Sega CD BIOS. It becomes `accuracy` with the
plan's §5.9 cost when the kernel runs (stage 3). The kernel's first phase is built and recorded in `Diamond_OpenKernel.md` (2026-10-11); the declaration
changes with its integration into the machine (that page's §1 and §10).

### 2.6 Settings

The schema's first entries, those of the plan's §4.7 that cost nothing to declare, all read at create (`scope:
create`, `restart`), all `effect: none` since each is a real console:

| Key | Choices (default first) | What the stub does with it |
|---|---|---|
| `region` | `auto` (from the disc or program), `psx.jp`, `psx.us`, `psx.eu` | the market: the rate, the picture's height, the BIOS named |
| `gpu` | `psx.gpu208` (the 208-pin family of the later consoles), `psx.gpu160` (the 160-pin chip of the first) | held for the GPU of stage 4; Q11's default |
| `pad1` | `psx.dualshock`, `psx.pad` | port 0's controller in machine info; Q11's DualShock, which starts in digital mode |
| `pad2` | `psx.dualshock`, `psx.pad`, `none` | port 1's controller, or an empty port |

Not declared yet: the console model beyond the GPU family (the CD controller's firmware date and the rest follow
from a model, stage 7), the memory card mode (stage 11), the enhancements (§6 of the plan, E1 on). The pads' Analog
button is not in the DualShock's descriptor: v1 has no canonical control for it, and the named-extra extension that
gives it one lands at stage 6 (`EmuSen_CoreAPI_Extensions.md` §5). The sticks are listed (`LeftX` to `RightY`), but
`AXES` is not claimed, so a host sends no axis until stage 6 models analog mode.

### 2.7 The spaces and the state, "DMND"

| Id | Space | Size | Flags |
|---|---|---|---|
| 1 | `RAM` | 2 MiB | the cheats' space |
| 2 | `SCRATCH` | 1 KiB | |
| 3 | `VRAM` | 1 MiB | |
| 4 | `SPURAM` | 512 KiB | |
| 5 | `BIOS` | 512 KiB | read-only; the player's image when given, zero otherwise |

*Since stage 3 (§11.16)*: a sixth space, `TTY` (6, 64 KiB, read-only), and the state carries the bus and its
devices, 3,743,658 bytes.

All zero-filled. `R3000BUS`, `CDBUFFER` and the cards of the plan's §4.3 come with the bus (stage 3), the CD-ROM (7)
and the cards (6).

Version 1 of the state: the magic `DMND`, the version, the frame count, the R3000A's block as `Cpu` (192 bytes) and
the GTE's as `Gte` (256), then every writable memory in id order: **3,671,504 bytes** [M, the kit] (3,671,512 since
stage 2, whose R3000A block is 200 bytes, §10), the plan's
"about 3.6 MiB" before the caches and the controllers' state join it. A state of another magic is refused as
foreign, of another version as a version, and a truncated one as truncated; a refused load changes nothing, the
processor blocks being read into copies first. The layout is pinned by `the_version_1_layout_is_pinned`. The disc's
identity (its hash and its number in a set, the plan's §4.7) is not in it until a disc is read (stage 7); no player
has a Diamond state, so version 1 is redefined freely until the core is offered. *Since stage 5 (§13.6), version 2,
the bus block carrying the GTE's deadline, pinned by `the_version_2_layout_is_pinned`.* *Since 2026-10-12 (§23),
version 10: the picture after the disc's fingerprint, 5,267,532 bytes, versions 8 and 9 still loading.* *Since 2026-10-13
(§24), optional tagged blocks after the picture, an enhancement layer's; with none the state is version 10's to the
byte, and no version was taken.*

### 2.8 What was measured (2026-10-11)

- **The crate's tests**: 15 pass (`cargo test --release`): the PS-EXE's header and market, a truncated PS-EXE, a disc
  in each image form and market, refusals, the rates, a run's silence summing to the rate, the bound on it, the
  picture, the read-only BIOS, the state's layout, round trip and refusals, the stub's report, the settings, a frame's
  silence, refusals with words, the player's BIOS and the pads, the descriptors.
- **The export check**: `export_check.py` on the library: ABI 1.0, capabilities 0, the 32 required `emusen_core_`
  symbols and no other.
- **The sidecar** carries `"development": true`; the registration golden (`EmuSen_CoreAPI.md` §25) is byte-identical,
  since a player's discovery does not see the core.

---

## 3. BlackMoon Hardware

The two crates and their contract are in `../BlackMoon-HW/README.md`; their records are `BlackMoon_R3000.md` and
`BlackMoon_GTE.md`. In short:

- `blackmoon-r3000`: the bus in the R3000A's terms (fetch, read and write by width, internal cycles, the interrupt
  request, coprocessor 2's transfers and commands), the bus owning the clock, the `Observer` step, the registers with
  both delay slots and coprocessor 0's, a pinned state block, and the harness over SingleStepTests' r3000 suite. **The
  replay control passes 55,000 of 55,000 cases and each spoiled replay 0 of 55,000** [M, `cargo test --release` with
  `EMUSEN_BLACKMOON_CORPUS`].
- `blackmoon-gte`: the 64 registers and psx-spx's names for them, the transfers and the command's signatures, the
  caller owning the clock, a pinned state block.
- Diamond holds one of each and writes their blocks into its state; machine info lists their registers.
- ~~**Not decided yet** [C, open]: whether the I-cache and the scratchpad live in the processor's crate or in Diamond's
  bus, settled at stage 3 (`BlackMoon_R3000.md` §2).~~ *Decided at stage 3*: in Diamond's bus (§11.4).

**The suite's data** [M]: SingleStepTests r3000 at `51e2274`, 24 MB as fetched, 55 files of 1,000 cases, transcoded
by its own script to 142 MB of JSON, in `~/.cache/emusen/probe/diamond/r3000/`, with every file's MD5 appended to
`~/.cache/emusen/probe/diamond/PROVENANCE.txt`. Never committed.

---

## 4. The registration

- **`RustCores.props`**: the row `diamond` (`Sony\Diamond - PlayStation\`, console "the PlayStation", no fallback,
  `CoreAbi 1`, `InDevelopment` true). The build writes the sidecar with `--development`; the library and the sidecar
  reach the build output for the tests and the kit and never a publish (`EmuSen_CoreAPI.md` §27.2).
- **The system entry's `development: true`** (§27.4 there): the core does not run the system's games yet, which is
  that field's meaning. With development cores asked for, a developer's frontend opens `.psexe` and `.psx`
  files on Diamond, and the PlayStation still gets no Graphics Settings tab, no shelf and no firmware row, as the
  Sega CD and the 32X are treated; without them, nothing offers the core. *Found while building* [R, the code]: with only the
  build's mark, asking for development cores would give the PlayStation a tab and a shelf
  (`CoreCatalog.DiscoveredConsoles` keeps a console whose engine says its system runs), against the rule
  `GenesisSettingsTests` holds, that a console in development has none; the two marks together say what is true, and
  removing both is the whole of offering the system after stage 11.
- **`EmuSen.csproj`** leaves the crates' `target/` folders out of its items; **`rust-cores.yml`** watches the two new
  folders and `EmuSen.WiseMan/Cores/Diamond*`, tests the BlackMoon crates in the shared job, builds the library on the
  four platforms with its sidecar marked for development, and runs `FullyQualifiedName~Diamond` in the WiseMan job.
- **`.gitignore`** gains `*.psexe`, `*.cue`, `*.chd` and `*.sbi`, so a dumped disc or program dropped in the tree is
  never committed; `.exe`, `.iso` and `.bin` are left alone, since the repository has its own files of those kinds.

---

## 5. WiseMan's builders

### 5.1 The PS-EXE builder

`Fixtures/SyntheticPsExe.cs`, the counterpart of `SyntheticRom`: the `800h`-byte header of §2.2 with the PC, the load
address, the size rounded up to `800h`, the stack base `801FFFF0h` and the market's marker (or none), then the code;
by default a jump to itself and its delay slot at `80010000h`, so a program is well-formed for the stages that run it.
`Words` writes MIPS words little-endian. Never real program data.

### 5.2 The synthetic disc builder

`Fixtures/SyntheticPsxDisc.cs` and `Fixtures/CdSector.cs`, so CI never needs a commercial disc:

- **The volume** (ECMA-119): the system area of psx-spx (sectors 0–15, the licence string in sector 4 for the market
  chosen, the logo sectors left zero, since the logo is Sony's data), the primary volume descriptor at 16 ("CD001",
  system and application `PLAYSTATION`, the volume's size, 2,048-byte blocks), the terminator at 17, the little- and
  big-endian path tables at 18 and 19, the root directory at 20 holding `.`, `..`, the program `EMUS_000.01;1` (a name
  in the serial's form that no licensed disc uses) and `SYSTEM.CNF;1`, whose `BOOT = cdrom:\EMUS_000.01;1` line,
  `TCB`, `EVENT` and `STACK` follow psx-spx; the files after it.
- **Raw sectors** (ECMA-130 §14 and Annex A): the sync pattern, the header with its BCD address (the block plus the
  150 of the two-second pregap) and its mode; **mode 1** with its EDC over bytes 0–2,063, eight zero bytes and the P-
  and Q-parity; **mode 2 form 1** with its subheader twice, the EDC over the subheader and data, and the parity
  computed with the header taken as zero (psx-spx's `encode_sector`), the last sector of each descriptor set, the
  directory and each file marked end-of-record and end-of-file; **mode 2 form 2** for the system area's sectors 12–15,
  with its EDC and no parity. The data track is built in mode 2 (PlayStation discs) or mode 1, and as an `.iso` of its
  user sectors.
- **The EDC** is the CRC of ECMA-130 §14.3, P(x) = (x¹⁶ + x¹⁵ + x² + 1)(x¹⁶ + x² + x + 1), taken least significant bit
  first, which is the reflected polynomial `D8018001h`, stored little-endian (x⁰ in byte 2,067's top bit, as §14.3
  says). **The parity** follows Annex A's definition, not a table: each P-vector (26 bytes of a column) and Q-vector
  (45 bytes of a diagonal) over GF(2⁸) with x⁸ + x⁴ + x³ + x² + 1, its two parity bytes solved from H = [1 … 1;
  αⁿ⁻¹ … α 1], so p₀ = (S₀ + S₁)/(α + 1) and p₁ = S₀ + p₀.
- **The CD-DA track**: the two-second pregap of silence, then a 441 Hz square wave, left and right in opposite phase,
  16-bit little-endian stereo; **the cue sheet** names both track files (`MODE2/2352` or `MODE1/2352`, then `AUDIO`
  with `INDEX 00` and `INDEX 01 00:02:00`), and `WriteSet` writes the set as a dump holds it.
- **XA interleave waits for stage 7** [C]: interleaving form 2 sectors by file and channel is small, but a useful XA
  stream needs an XA-ADPCM encoder, which is the larger part and is best written beside stage 7's decoder and graded
  against it. Also left for stage 7: the XA attributes in directory records, which the open kernel may want, and
  subchannel data.
- **How it is checked** [M, `DiamondTests`]: the EDC divides every kind of sector (the CRC over data and EDC is zero);
  Annex A's two checks hold on all 138 vectors of a mode 1 and of a mode 2 form 1 sector (the latter with the header
  cleared, and not without); one changed byte breaks both the EDC and the parity; every raw sector of the track
  carries its own user data and address; the volume descriptor, the root directory, `SYSTEM.CNF`'s `BOOT` line and the
  program's bytes read back from the `.iso`. *Argued, not shown*: that a drive or a reference would accept the
  sectors. The checks are Annex A's equations, written apart from the encoder's solution of them, so they catch an
  encoder that does not meet its own definition, not a misreading of ECMA-130 shared by both; the references of the
  plan's §1.6, which read raw images, are the test of that at stage 7. The disc crate of stage 7 (`emusen-disc`)
  synthesises sectors itself and is then graded against this builder.

---

## 6. The gate

The plan's §7 row 1, measured 2026-10-11 [M].

**The kit**, `emusen-core-conform --frames 600`, single-image form, on four synthetic images: the PS-EXE (4,096 bytes),
the disc's mode 2 data track (56,448 bytes, 24 sectors), its `.iso` and its mode 1 track. **C1–C15 pass on all four**,
"EmuSen v1 compliant on this image", each run 2.2–2.3 s and 42 MB.

| Case | Verdict | What the stub showed |
|---|---|---|
| C1 loading | pass | ABI 1.0, every required export, 32 `emusen_core_` symbols |
| C2 info | pass | capabilities 0; "system psx in development, offered to no player" |
| C3 the schema | pass | 4 settings |
| C4 create and refusal | pass | the empty and the garbage image refused, −9 with words; the half PS-EXE refused, −10 with words; each half disc accepted and run 300 frames (sector 4 lies in its first half); the one firmware entry optional, created with no files |
| C5 capability honesty | pass | nothing claimed, every optional export absent |
| C6 the frame | pass | 600 frames; 884,692 samples at 44,100 Hz |
| C7 state | pass | 3,671,504 bytes; round trip, constant size, refusals |
| C8 determinism | pass | equal frame, audio and state digests, across a save and load |
| C9 exact settings | pass, nothing to check | no setting declares `effect: exact` |
| C10 isolation | pass | two machines on two threads, a third created and freed |
| C11 error paths | pass | `status_text` for every code returned (−10, −9, −3, −2) |
| C12 skew | pass | hosts of minor 0 and of minor 99 with longer structs |
| C13 library-level concurrency | pass | 13,623 to 15,134 rounds across the runs, beside 300 frames |
| C14 descriptors against exports | pass | 5 spaces, no battery file, 2 processors |
| C15 debug | pass, nothing to check | `DEBUG` is not claimed |

C9 and C15 are the two a stub cannot meaningfully pass, as the plan allows: they pass by having nothing to check,
and become real when the first `exact` setting (stage 13's levers) and `DEBUG` (stage 12) are claimed. C6, C7 and C8
check a black, silent machine whose only moving state is its frame count. `DiamondTests` runs the kit on the PS-EXE
and the mode 2 track at 120 frames, so CI keeps the gate.

**The r3000 controls**: the replay passes 55,000 of 55,000 cases; the spoiled replays (HI's lowest bit flipped, the
last cycle dropped) pass 0 of 55,000 each (§3).

**WiseMan** [M]: `DiamondTests`, 10 tests: the library and the pack (extensions, firmware, controllers, settings,
`firmware_for` by market), a machine of each image kind with the pack's aspect and its rate, the generic runner on
the program and each disc form (the spaces, a black 320×240 picture standing still, 44,100 Hz, a `Visual` verdict),
discovery with development cores asked for and without, the kit, the PS-EXE builder, the sectors, the disc's volume
and files, and the pack's players.

---

## 7. Decisions, questions and what the plan got wrong

- **The program extensions**, *decided 2026-10-11 by the tester*: `.psexe` and `.psx` are kept, and `.exe` is dropped
  until routing by content exists, so that no Windows program reaches the PlayStation's shelf.
- **The market of a program with no marker** is the Americas [C], and **port 2 holds a DualShock by default** [C, Q11
  speaks of port 1 only]; both are a setting's default and change by one line each.
- **The system entry is marked in development as well as the build** (§4): the plan's §8 names only §27's mark.
- **The frame rate's constants** are the nominal clocks, provisional on D-0 (§2.3).
- **Where the I-cache and the scratchpad live** is left to stage 3 (§3).

## 8. Negative results

- The r3000 suite has no memory list: a read's value is its cycle's (`BlackMoon_R3000.md` §3.1).
- The suite's README prints the two delays' names exchanged in its example's initial state; the data does not
  (ibid.).
- No public check of a synthesised sector's correctness was run against a drive or a reference at this stage (§5.2).

## 9. What this part does not cover

- No emulation: every verdict in the runner is `Visual`, correct for a black machine and saying nothing yet.
- The references, their costs and spaces, the documents and the test programs, and the disputes log: the other part
  of stage 1, recorded in the plan's §1.6 and §3.1 and in `Diamond_Disputes.md`.
- No speed measurement of Diamond.
- `kit/psx-sample.txt`, the corpus list of the plan's §3.4, waits for discs and for a kernel that runs the test
  programs; the kit's memory estimate per image (`EmuSen_CoreAPI_Extensions.md` §7) is being built apart from this
  stage.

---

## 10. Stage 2: the R3000A (2026-10-11)

The plan's §7 row 2: the R3000A, with the shared MIPS crate's decode, disassembler and assembler, and its state. The
processor's own record is `../BlackMoon-HW/BlackMoon_R3000.md`; `emusen-mips`'s is `../../Shared/EmuSen_Mips.md`.
This section records what changed for Diamond and what the stage measured.

### 10.1 What was built

- **`emusen-mips`** in `EmuSen/Cores/Shared/`: the fields, the decode of MIPS I as the R3000 decodes it with an
  extension point per coprocessor, the disassembler with a table per coprocessor, and an assembler for test
  programs; no execution (`EmuSen_Mips.md`).
- **`blackmoon-r3000`**, built: MIPS I with both delay slots, coprocessor 0 as the PlayStation has it (the mode stack,
  RFE, cache isolation, the debug unit, PRId, usability), exceptions and interrupts with their Cause codes, the
  multiplier's interlock through the bus's idle cycles, the observed step, coprocessor 0's table for the disassembler,
  and the state block grown by the multiplier's deadline (`BlackMoon_R3000.md` §4–§7). Its bus gained the three-byte
  access, the isolated-cache calls and a clock the processor reads.
- **`blackmoon-gte`** gained its table for the disassembler, its register names and 22 commands (`BlackMoon_GTE.md`
  §2); nothing of the GTE executes before stage 5.
- **Diamond** carries the 200-byte block as `Cpu`: version 1 of `DMND` was redefined (no player has a Diamond state,
  §2.7), its layout test re-pinned, the state 3,671,512 bytes. The machine does not step the processor yet: the bus
  it steps against is stage 3's.

### 10.2 The gate

**SingleStepTests r3000: 55,000 of 55,000 cases** on registers, HI/LO, the PC, both delay states and every cycle's
fetch, read and write [M, `cargo test --release` with `EMUSEN_BLACKMOON_CORPUS`, 0.4 s]. One dispute names cases:
**D-1**, the 1,022 LWL, LWR, SWL and SWR cases that move three bytes, which the suite records as a halfword and a byte
and the manuals as one access; they are graded with the two cycles joined, and pass. The result by instruction group
is `BlackMoon_R3000.md` §9's table. Two more disputes were opened without naming suite cases: **D-2**, the multiplier's
latencies, where IDT's figures and psx-spx's disagree and psx-spx's PlayStation table is taken, open until a program
times it; **D-3**, LWC0 and SWC0 raising CpU only with CU0 clear, settled by ps1-tests' `cpu/cop` console log against
psx-spx's text. Stage 1's controls hold: the replay passes every case and the spoiled replays none.

**P1 retired** (the plan's §13): "The R3000A passes every case of the r3000 suite's registers before any bus exists,
and its cycle lists on all but a handful of cases that become disputes about fetch ordering." The registers half held,
before any bus. The cycles half was wrong in both its count and its subject: every cycle list passes but 1,022 cases'
(1.9%, not a handful), and their dispute is about the width of an access, not the order of fetches; the suite's order
of fetch, then read or write, matched the processor everywhere.

### 10.3 Tests run (2026-10-11)

`cargo test --release`: `emusen-mips` 10; `blackmoon-r3000` 25 (the suite, its controls, D-1's cases, every suite
opcode through the assembler, 17 cases of the project's own, the state block, the names held equal); `blackmoon-gte`
4; `diamond` 15. `cargo clippy --all-targets` clean on the three crates. The kit's C1–C15 pass on a PS-EXE at 120
frames with the new state size. No C# changed, so WiseMan's tests were not run.

### 10.4 What the stage leaves

- The I-cache, the scratchpad and the cache control register: inside the processor's crate or in Diamond's bus, for
  stage 3 (`BlackMoon_R3000.md` §2), with the isolated calls ready for either.
- Every reading the suite cannot reach (interrupts, coprocessor 0 beyond an exception, user mode, the multiplier's
  timing) is held by the crate's own cases against itself; amidog's CPU and CPX tests and ps1-tests' `cpu/*` grade them
  once the bus and the open kernel run them (stage 3).
- D-2's measurement, by a program of the project's own at stage 3.


---

## 11. Stage 3, the machine half (2026-10-11)

The plan's §7 row 3 for the machine: the bus and its timing, memory control, the I-cache, the interrupts, the root
counters, the seven DMA channels, the video timing the counters see, PS-EXE loading, the kernel's seam, and the TTY
reader. The kernel's first phase was written apart, under the replacement writers' rules (`Diamond_OpenKernel.md`),
and joined through its §3.3 interface; nothing of its source was read for this half. Measured on 2026-10-11 on the
desktop [M] unless marked.

### 11.1 What was built

- **`bus.rs`**, the R3000A's `Bus`: the address decode (§11.2), every access timed (§11.3), the write queue, the
  I-cache's fetches and refills (§11.4), the I/O ports, the clock's events, and the DMA's moves (§11.7).
- **`memctl.rs`** memory control; **`icache.rs`** the cache and BCC; **`irq.rs`** I_STAT and I_MASK; **`timers.rs`**
  the root counters; **`dma.rs`** the DMA controller's registers and decisions; **`video.rs`** the video timing;
  **`devices.rs`** the later stages' register stubs and the DUART; **`exe.rs`** PS-EXE loading and building;
  **`kernel.rs`** the seam; `machine.rs` and `state.rs` reworked.
- **`blackmoon-r3000`** gained `Bus::bus_error`, a default method answering false, so a flat bus and the crate's own
  harness are unchanged: the processor takes IBE after a fetch and DBE after a load or store that met no device
  (`BlackMoon_R3000.md` §2).
- **Four programs of the project's own** in `programs/`, written for `emusen-mips`'s assembler (§11.13), and two
  development tools in `examples/`: `run` (a PS-EXE on the open kernel, a player's BIOS or none, printing its TTY or
  RAM) and `psexe` (a source assembled into a PS-EXE).
- **WiseMan**: the TTY reader and ps1-tests' protocols as data (§11.12); `CoreAbiTestRomEngine` takes create's
  settings by path.

**Read for this half** [R]: psx-spx at `6d7d1bc`, "Memory Map", "I/O Map", "Memory Control", "Interrupts", "Timers",
"DMA Channels", "GPU Display Control Commands (GP1)", "GPU Status Register", "GPU Timings", "MDEC Status Register",
"SPU Control and Status Register", "Serial Interfaces (SIO)", "DEV0 Expansion ROM Header" and "Mid-Boot Hook",
"CDROM File Formats" for the PS-X EXE header, and the kernel chapter's "BIOS Function Summary", "BIOS File Execute
and Flush Cache" (Exec, LoadExec) and "BIOS TTY Console (std_io)", all interface descriptions outside the replacement
writers' exclusions; ps1-tests' sources for `cpu/access-time`, `cpu/code-in-io`, `dma/chain-looping`,
`dma/chopping`, `dma/otc-test`, `common/measure.hpp`, `common/timer.h`, `common/log.h`, `common/test.h`,
`common/io.h`, `common/delay.c`, `common/dma.cpp` and `common/dma.hpp`, and the release's console logs; amidog's two
readmes; Peter Lemon's GTE sources at their `mtc0` lines; `Diamond_OpenKernel.md` §3.3, §4.10, §8, §9 and §10, the
interface page. **A reading to declare**: ps1-tests' `common/dma.hpp` says on its third line that it was "Copied from
Avocado", an emulator; its first 120 lines (the MADR, BCR and CHCR bitfields and the CHCR presets) were read while
reading `otc-test` before that line was noticed. Nothing was taken from it: the DMA's registers are psx-spx's, whose
layout it repeats. It is proposed for `DOCS.txt`'s exclusions beside the kernel page's §10 Q4 list. No emulator's
source, no firmware or firmware listing, no PSX_MiSTer file, and no reference core's log of a run on the player's
BIOS was read (§11.15).

### 11.2 The map

From psx-spx's "Memory Map", "I/O Map" and "Memory Control" [R]:

- **Segments.** KUSEG's first 512 MiB, KSEG0 and KSEG1 decode to one physical space; KUSEG above 512 MiB and KSEG2
  but `FFFE0130h` raise a bus error [A: psx-spx says such an access "causes an exception" without naming it].
- **RAM**: 2 MiB mirrored through the window the RAM size register gives, /RAS0's size and /RAS1's when enabled; the
  kernel's `0B88h` makes it 8 MiB, the three accidental mirrors psx-spx records, and past it a bus error.
- **The scratchpad** in KUSEG and KSEG0 only; not executable, a fetch there a bus error (psx-spx "Scratchpad").
- **The windows**: DEV0 at its base register's address for the size its delay register gives; DEV1 at `1FA00000h`;
  the BIOS at `1FC00000h` for DEV2's size (512 KiB from the kernel's value); DEV8 at `1F802000h`. A window beyond its
  size is a bus error (psx-spx: "Trying to access addresses that exceed the selected size causes a bus exception").
  DEV0, DEV1 and DEV8 read FFh with nothing fitted [A], DEV8 but for the DUART (§11.8) and the POST byte.
- **The I/O ports** at their documented ranges; the gaps between them a bus error ("Memory Exceptions"). A fetch from
  I/O is a bus error but from the DMA registers and the SPU's, which `cpu/code-in-io` shows a console executing
  [R its source and `psx.log`].
- **Widths.** A 32-bit register takes the whole register on a byte or halfword store, as the processor drives it
  (`io-access-bitwidth`'s DMA rows). *Since 2026-10-12* (`Diamond_Disputes.md` D-45): at an address not word-aligned
  the word is shifted to the store's byte first, as psx-spx's "(w32)" has it ("left-shifted if address isn't
  word-aligned"), for memory control, the RAM size register, I_STAT and I_MASK, every DMA register but the lengths
  (which psx-spx marks "OK" at every width, and which keep the unshifted write they had), DPCR, DICR and the timers;
  a game's `sb` to DICR's third byte sets the channels' interrupt enables, which Silent Hill, Castlevania: SotN, Tekken 3
  and Ape Escape stream their movies by. On the CD-ROM's 8-bit bus without auto-increment every byte of a wider access
  meets one address, so a 32-bit read returns the status byte four times and a 16-bit store writes the index register
  twice (`18181818h`, `1A1A1A1Ah` in the console's log, and DEV5's bit 13 clear) [R, A]; the SPU's 16-bit bus with
  auto-increment takes a word as two halfwords.

### 11.3 The access times, from memory control and the console (D-4)

A load's cost is its instruction's cycle plus the region's stall; a fetch's, the cache's or the region's (§11.4).

| Region | Rule | Source |
|---|---|---|
| Scratchpad, BCC | no stall | `access-time` on a console [M by ps1-tests] |
| RAM | 4 cycles' stall, 5 in all | the same |
| Internal registers (memory control, SIO, IRQ, DMA, timers, GPU, MDEC) | 2 cycles' stall, 3 in all | the same |
| BIOS, DEV0, DEV1, DEV8, CD-ROM, SPU | the delay register: a first bus cycle of read delay + 4, each further one of read delay + 2 + COM0 if bit 8 + COM2 if bit 10; a unit a byte, or a halfword on a 16-bit bus | the console's figures, against psx-spx's formula: `Diamond_Disputes.md` D-4 |

On the kernel's register values this gives 7, 13 and 25 cycles for the BIOS's three widths, 11, 26 and 56 for DEV8,
6, 6 and 10 for DEV1, 8, 14 and 26 for the CD-ROM and 18 and 18 for the SPU, the console's to the cycle
(`memctl::tests`). **The table against the console's** (`cpu/access-time`, on the player's BIOS, printed as cycles a
load in hundredths):

| Segment | Console 8 / 16 / 32 | Diamond 8 / 16 / 32 |
|---|---|---|
| RAM | 5.21 / 5.03 / 5.14 | 5.01 / 5.01 / 5.08 |
| BIOS | 7.06 / 12.94 / 24.94 | 7.15 / 13.08 / 25.01 |
| Scratchpad | 1.05 / 1.01 / 0.94 | 0.99 / 0.99 / 0.99 |
| DEV0 (EXPANSION1) | 6.94 / 13.07 / 25.07 | 7.01 / 13.01 / 25.08 |
| DEV8 (EXPANSION2) | 10.99 / 25.99 / 55.98 | 11.07 / 26.00 / 56.00 |
| DEV1 (EXPANSION3) | 6.07 / 6.01 / 9.95 | 6.19 / 6.01 / 10.08 |
| DMA control, SIO0 and SIO1 status, RAM size, I_STAT, timer 0, MDEC status | 2.92–3.18 | 3.00–3.12 |
| CD-ROM status | 8.00 / 14.00 / 25.93 | 8.00 / 14.00 / 26.00 |
| GPUSTAT | 2.93 / 2.92 / 3.08 | 3.07 / 3.07 / 3.00 |
| SPUCNT | 17.99 / 17.99 / **38.94** | 18.07 / 18.00 / **36.07** |
| BCC | 0.95 / 1.09 / 1.09 | 1.01 / 1.08 / 1.01 |

**50 of 51 cells within 0.20 cycles**, the test's tolerance taken as half a cycle, since the program prints whole
cycles and a remainder over 100 loads [C]. The fifty-first, SPUCNT's 32-bit load, reads a word at `1F801DAAh`, which
is not word-aligned: it times an address error's entry and return through the kernel's handler, not the SPU, and is a
named disagreement of D-4 (2.87 cycles). The hundredths that remain are not modelled: the console's RAM row gains 21
cycles over 100 loads, a refresh or the measurement's overhead [A].

**Stores** go through the four-entry write queue (psx-spx "Write queue") unless they are uncached: a store waits only
while all four entries are busy; a load from memory or I/O, or any KSEG1 access, waits for the queue to drain [C:
psx-spx says the queue is flushed before a read of the same location; the whole queue is the simpler reading]. **A
RAM store takes 8 cycles to drain** [M, fitted]: `dma/chain-looping`'s loop of a load, an add and a store to one RAM
word takes 16 cycles an iteration on the console (16,040 for 1,000) and 16,016 with this figure; with one cycle it took
11,024. That is the one figure the program pins, and how the eight divide between the store and the load that follows
it is open (D-5).

### 11.4 The I-cache, in Diamond's bus

*Decided at stage 3*: the I-cache and BCC sit in Diamond's bus, not the processor's crate (`BlackMoon_R3000.md` §2's
open item), because their timing is the bus's (a refill's cost is the region's) and the PlayStation 2's I/O processor
differs in its caches [A]; the isolated calls the processor already makes answer them.

From psx-spx's "i-Cache" and "FFFE0130h" [R]: 256 lines of four words, a tag of the physical address's bits 31–12 with
a valid bit per word; a miss fills from the word to the end of the line, two words from word 0 with IBLKSZ 0; a tag
that matches with the word invalid refills all four; stores never touch it; with SR's IsC set, a store writes the
tag (TAG set: the offset's address bits, the datum's low four bits as the valid bits) or the word, and a load reads
them back with the match in bit 4. KUSEG and KSEG0 are cached while IS1 is set; KSEG1 never. A hit costs one cycle; a
RAM miss five for the first word and one for each further one [C: psx-spx gives no refill timing; DMA's "1 clk/word"
in hyper page mode argues for the burst]; a BIOS fetch the region's word access. A tag of zero matches the first 4 KiB
of RAM, so a cold cache "hits" there with invalid words and refills whole lines [A, from the tag format].

### 11.5 The clocks and the video timing (D-0)

Plan §5.1's two domains, settled by D-0's second step: the CPU's 33.8688 MHz and the console's video clock,
53.693175 MHz on an NTSC console and 53.203425 MHz on a PAL one, related by the crystals' ratio, 715,909/451,584 and
709,379/451,584. Positions are counted in half video clocks so that **D-0's provisional line, 3,412.5 video clocks
in NTSC mode and 3,405 in PAL mode**, is whole; the console's clock follows its market, the line its GP1(08h) mode, as
psx-spx's maintained text says ("Consoles will always use the video clock for its region") [R].

- **A field** is 263 lines (314 PAL), interlaced 263 and 262 alternately ~~(314 and 313)~~ (313 and 312, since D-0's
  referee step: 314 and 313 averaged 313.5, not psx-spx's 312.5) [C: the average is the 262.5 psx-spx gives; how a
  console splits the half line is D-0's open part].
- **Vblank** is the field lines outside GP1(07h)'s range, beginning at Y2's line; IRQ0 at its start; none when the
  range leaves no edge (psx-spx: a Y2 too large stops the interrupt). **Hblank** is the part of each line outside
  GP1(06h)'s X1..X2, beginning at X2; none with a range of no width (psx-spx's "endless hblank").
- **The dot clock** divides the video clock by 10, 8, 7, 5 or 4 by the mode; a line holds a whole number of dots, the
  fraction dropped (psx-spx), 426 at 320 pixels.
- **GPUSTAT's bit 31** is the line's parity in 240-line mode and the field's interlaced, zero in vblank; bit 13 the
  field, 1 when not interlaced.
- A change to the mode or the ranges takes effect from the current line, the counts so far kept [C].
- *Superseded at stage 4 (§12): a frame ends as a field's vblank begins, the rate unchanged. Decided 2026-10-11: the
  frame ends as vblank begins, and stays so.* **The frame** is one
  field, from its first line to the next field's; machine info's rate the console's own standard
  non-interlaced, **59.8261 Hz NTSC and 49.7614 Hz PAL**, replacing stage 1's 59.8173 and 49.7468 (§2.3), provisional
  on D-0's closing test; pinned by `the_rates_are_d0s_provisional_ones` in `video` and `machine`, and by DiamondTests
  to four places.
- **The sound** is still silence, now counted from the clock: ⌊cycles / 768⌋ samples since power-on, so any run's
  count is exact against 44,100 Hz on the CPU's clock whatever the frames' lengths [A: the SPU runs on that clock,
  33.8688 MHz over 768].

**What `timers` shows** [M, on the player's PAL BIOS with the region set to Europe, the console of its log being
PAL-clocked by D-0]: 263 hblanks a frame in NTSC mode, as the log; 982–983 dots in 5,000 cycles, the log's 982–983;
71,410–71,412 system clocks over 8 a frame against the log's 71,407–71,412; the system-clock frame rows 112,550 raw
against 112,508–112,557 settled, so 571,303 cycles unwrapped by D-0's arithmetic against 571,285 ± 6, the difference
the programs' overheads. On the NTSC clock the same program reads 992 dots in 5,000 cycles and 566,090 cycles a
frame, the NTSC column of D-0's table less the overhead, which only a console can confirm.

### 11.6 The root counters

From psx-spx's "Timers" [R], each counter brought to the present from its source's tick count when it is read or
written, at its target or FFFFh, and at the blanks it syncs to: the system clock, the dot clock (timer 0), hblank
(timer 1), the system clock over 8 (timer 2); the four sync modes against hblank or vblank, and timer 2's stop or free
run; ZRET, the target and FFFFh flags cleared when the mode is read, the target and overflow interrupts, one-shot or
repeated, pulsed or toggled (an I_STAT edge when INTF falls). A write to the mode resets the count and flags.

- **The holds**: "it will stay at 0000h for 2 clock cycles" after a mode write, a count write or a target's reset, 1
  after FFFFh, psx-spx's example to the cycle (`a_target_with_zret_holds_zero_two_cycles`). The hold is of the system
  clock only [A]: applied in ticks of a slower source it swallowed a whole hblank, and `timers` read 262 hblanks a
  frame against the console's 263 until it was removed [M].
- **Sync mode 2 reads zero outside the blank** [M by the console's log: `timers`' mode-2 rows read 0 between hblanks
  and count only inside them, 36, 159, 282, 405, 528, then 0], where psx-spx says "reset at hblank and pause outside";
  the counter is reset at the blank's end as well as its start (D-6). Mode 3's "pause until hblank once" and modes 0
  and 1 follow psx-spx.
- ~~**Not graded**: the sync-mode tables' values depend on where the program begins against hblank, so they are
  compared by pattern, not number (§11.12).~~ *Graded since 2026-10-11 by their shape* (§11.12): the values still
  depend on where in the line each test begins, but no row can start above what a free-running count reaches in the
  same time, nor rise faster, nor fall where its mode neither resets nor wraps.
- **A mode write with the sync on** (*2026-10-11*, D-6's second part, `Diamond_Records.md` §8.1): the write resets the
  count, sets its running from the blank's level at the write, and no blank edge before it acts on the timer. Hblank's
  edges are offered to the event loop only while timer 0 syncs, an economy of about 526 events a field otherwise; so
  when a write turns them on, `Video::skip_hblanks` first moves the edges' processed mark to the present, stopping just
  short of any vblank-side edge still due, which the loop then takes in its turn. Without it the loop took every hblank
  edge since the last one processed at its own past time, set the timer's tick base back to it and applied it to the
  new mode, crediting up to 48,340 cycles a write [M]. One case is left [A]: an hblank edge falling in the same
  instruction as a still-due vblank-side edge would be taken after the write; with the display ranges `timers` sets an
  hblank edge lies at least 160 cycles from a line's start, where the vblank-side edges fall, and of the 98 writes
  that turn the sync on in `timers`' release and rewrite and in `timer-sync.s`, none found such an edge still due [M].
  *Measured on 2026-10-11*: `timer-sync.s` reads 3 and 6, or 0 where the mode pauses or waits, in all 32 cases, where
  timer 0 had read up to 46,876; `timers`' rewrite and release meet all 14 sync tables' shape (6 and 4 before) and their
  other rows are unchanged; the cost is below. Pinned by `a_mode_write_with_sync_resets_the_count`,
  `the_timers_records_sync_tables_have_the_logs_shape` and the TTY gate.

  *Read for this step* [R]: psx-spx `6d7d1bc`, "Timers" (the counter "gets forcefully reset to 0000h on any write to the
  Counter Mode register"; "Reset and Wrap"); ps1-tests' `timers` console log; Diamond's own `timers.rs`, `bus.rs`'s timer,
  event and I/O paths, `video.rs`'s edges, `state.rs`'s video fields (to keep the state format unchanged: the fix adds no
  state), `machine.rs`'s step; the project's pages `Diamond_Records.md`, `Diamond_Disputes.md` (the rules, D-6),
  `Diamond_Referees.md` and this page's §11.5–§11.6, §11.12, §11.14; DOCS.txt's exclusions; the referees' RAM for
  `timer-sync.s` and the records' earlier RAM and TTY of `timers` (`Diamond_Referees.md` §2). Not read: any emulator's
  source, any BIOS or listing (the player's BIOS was handed to the referees by path only). **Cost** [M, `examples/run`,
  600 frames, three runs each, before and after side by side]: the release `timers` on the open kernel as a PAL
  console 6.20–6.29 ms a frame before and 6.11–6.28 after, `gpu/triangle` 6.39–6.41 and 6.37–6.46. The edges passed
  over are never processed, so the fix removes work rather than adds it, one edge search a write; the differences are
  within the runs' spread.

### 11.7 DMA

From psx-spx's "DMA Channels" [R]: MADR, BCR and CHCR per channel with their writable bits (OTC's three, its step
always backward), the CHCR mirror at `xCh`, DPCR with its reset `07654321h` and DICR with its flags written 1 to clear,
its master flag recomputed and IRQ3 on its rise. A channel starts when its START bit is set and DPCR enables it and
either its trigger (bit 28, cleared as it begins) is set or its device requests; DPCR ranks the ready ones, the
higher channel on a tie. The devices' requests are the stubs' (§11.8): the GPU's by GP1(04h), the SPU's by its
transfer mode, the MDEC's by its control bits, the CD-ROM and PIO always.

- **Modes**: a burst at once (mode 0); a chopped burst in windows of 1 << N words with the CPU's window between,
  MADR and BCR updated; blocks (mode 1), BA counted down; linked-list nodes (mode 2), the header's count of words to
  the device and the next address, ending at bit 23 (the FFFFFFh end code and any address above 8 MiB) [A]. OTC writes
  its reversed list, the last entry FFFFFFh.
- **Words per cycle**: psx-spx's table, 17/16 of a cycle a word for MDEC, GPU and OTC, 66/16 for the SPU, 24 or 40 for
  the CD-ROM by DEV5's recovery bit, 20 for PIO.
- **The bus** [C]: a piece holds the bus for its time; the CPU waits only when it needs the bus (a cache miss, an
  uncached fetch, a load from memory or I/O, an uncached store), as psx-spx's "CPU Operation during DMA" says, and
  queued stores wait with the DMA. A piece that fell due while the CPU was on the bus delays the CPU by its length.
  A transfer's end, its flags and IRQ3 are an event at the end of its last piece; a read of its registers waits for
  the bus, so CHCR reads done once the CPU can see it (`testOtcControlBitsAfterTransfer`).
- **IRQ3** is set when DICR's bit 31 rises from zero, at a completion or a write, never by a write that leaves it up
  (D-29). **Arbitration** (D-43): a channel in sync mode 1 or 2, between two pieces with the bus let go, steps aside
  for any other ready channel and is arbitrated again when that one ends.
- **The gaps**, fitted to the console's logs (D-5): 10 cycles between blocks and between list nodes, 4 of a node's own,
  and a chopped window's CPU window held to 32 cycles a word of the DMA's window plus 6. On `dma/chopping`'s 130 rows,
  **129 within 10% of the console and the worst at 10.5%** (one-word windows with a one-cycle CPU window, 18,448
  against 16,693); `chain-looping`'s self-referencing chains take 24,032 cycles of work against the console's 25,632
  and 25,640 (−6.2%).

### 11.8 The stubs, and what each answers

| Device | What a stage 3 program reads | Source |
|---|---|---|
| GPU (stage 4; *replaced at stage 4 by the GPU, §12*) | GPUSTAT `14802000h` after GP1(00h), always ready for a command and a DMA block (bits 26, 28), bit 25 by GP1(04h), bits 31 and 13 from the video timing; GP0's commands counted out by their lengths and dropped, but E1h–E6h into GPUSTAT and GP1(10h)'s answers, and GP0(1Fh)'s IRQ1; GP1's display commands into the video timing; GP1(10h) index 7 the version, 2 for the 208-pin chip | psx-spx's GP0/GP1 and GPUSTAT pages |
| SPU (stage 8) | its 512 halfword registers, STATX's low six bits ATTR's, the transfer address and the data FIFO into sound RAM, DMA by ATTR's mode, `1F801E80h` on FFh | psx-spx's SPU control pages, "I/O Map" |
| CD-ROM (stage 7) | the index; HSTS `18h` with the index (both FIFOs empty and ready); HINTMSK and HINTSTS with their top three bits set; no command is answered | psx-spx's CD-ROM ports; `io-access-bitwidth`'s console log |
| MDEC (stage 9) | status `80040000h`, reset by bit 31 of MDEC1, its requests from bits 30 and 29 | psx-spx "MDEC Status Register" |
| SIO0, SIO1 (stage 6, later) | ready and idle; a byte sent answered with FFh and no /ACK; the mode and control registers, bit 6 of control a reset | psx-spx's SIO page; `io-access-bitwidth` |
| The DUART at `1F802020h` | channel A's mode registers by their pointer, its status `0Ch` (ready, empty), its transmit register the TTY; nothing received | psx-spx "DEV8 Dual Serial Port" |

**The DUART is fitted** [C]: psx-spx says no retail console carries one; Diamond's DEV8 answers as a console with the
debug terminal's chip, so that the open kernel's standard output (`Diamond_OpenKernel.md` §4.10) reaches the runner.
DEV8's other addresses read FFh as on a console (`io-access-bitwidth`).

### 11.9 PS-EXE loading

From psx-spx's header and "A(43h) - Exec" [R]: the body copied to its load address in RAM, the memfill region
zeroed, PC and GP from the header, SP and FP at the stack base plus offset when the base is not zero, R4 = 1 and
R5 = 0, both delays cleared. With a kernel, the load happens when the CPU reaches the shell's entry (§11.10), and the
I-cache's tags are cleared as the kernel's FlushCache would before a program runs [A]. With none, it happens at
create, SR zero and the kernel's memory control and BCC values in place. `exe::build` writes a PS-EXE for the
project's programs, the Rust counterpart of WiseMan's `SyntheticPsExe`, the two kept apart by language [C].

### 11.10 The kernel's seam

The interface of the plan's §5.9 and of `Diamond_OpenKernel.md` §3.3, which this half provides and nothing more:

- **Which kernel**: a player's BIOS for the console's market from the firmware files (the exact path); otherwise the
  open kernel's committed image, `kernel/diamond-kernel.bin`, included in the library; a program started directly only
  by a development tool (`run --bare`) and the crate's tests. Machine info names the source: `file`, `replacement`.
- **Reset**: the image in the BIOS region, the CPU at `BFC00000h` with SR's BEV set, memory control at its documented
  maximum delays with the kernel's sizes, BCC zero [C]; the kernel sets them itself.
- **The hand-over**: when the CPU is about to execute at physical `30000h`, outside a delay slot, the PS-EXE is
  side-loaded (§11.9). psx-spx's "Mid-Boot Hook" names `80030000h` as the point where the kernel starts its shell with
  its tables installed [R]; the open kernel writes a loop there for this purpose, and a player's BIOS reaches the same
  address [M: `cpu/cop` and the rest run on it]. The machine reads nothing else of a kernel.
- **The TTY**: the open kernel's standard output is the DUART's transmit register, one byte store a character
  (`Diamond_OpenKernel.md` §4.10). A player's BIOS mounts the dummy TTY by default (psx-spx "Activating std_io"), so its
  output is taken from the program's putchar calls at the documented entries, `A0h` with R9 = 3Ch and `B0h` with R9 =
  3Dh, R4 the character, counted only when the call executes (an interrupt at the boundary does not repeat it).
  psx-spx says printf "does internally use putchar", and the programs' printf output arrives this way [M]. The tap is
  a comparison of the PC and two registers at an instruction boundary; nothing of the kernel's code or RAM is looked at.
- **The firmware entries** are `effect: accuracy` since the kernel runs, with the plan's §5.9 cost and phase 1's
  sentence (`Diamond_OpenKernel.md` §8).

### 11.11 The development tools

`cargo run --release --example run -- <exe> [frames] [--bios FILE | --bare] [--pal] [--dump ADDRESS WORDS] [--ram
FILE]` prints a program's TTY, or RAM's words, after a number of frames; `cargo run --release --example psexe --
<source.s> <out.psexe>` assembles one of `programs/`. Neither is a frontend; a BIOS they are given is the player's own,
named on the command line and never kept.

### 11.12 The TTY reader and ps1-tests' protocols

`EmuSen.WiseMan/Fixtures/RomRunner/TtyReader.cs`, the plan's §3.3: `TtyReader` takes the text of the `tty` space (6,
64 KiB, read-only, in the state) up to its first NUL with carriage returns dropped, from a program's heading line on so
that a kernel's own messages before it are left out. `Ps1Tests.Programs` is the data: each program's path, heading,
rule, tolerance, named disagreements and create settings.

| Program | Rule |
|---|---|
| `cpu/cop`, `cpu/code-in-io`, `dma/otc-test`, `dma/dpcr` | pass and fail lines: no `fail`, at least one `pass`, and `Done.` |
| `cpu/access-time` | the table against the console's `psx.log`, each cell within 0.5 cycles, SPUCNT's 32-bit cell named (D-4) |
| `dma/chain-looping` | line by line against the log: the words equal, each number within 10% (D-5) |
| `timers` | each delay and frame row's settled values (the first two dropped) against the log's mean: a frame's hblanks ~~exactly~~ ±1 *since 2026-10-12*, its dots ±10, its cycles ±60 of the 16-bit count, a delay ±3 cycles or ±2 slower ticks, *since 2026-10-12* one delay sample longer than that dropped in each row as one an interrupt landed in (§11.12a); run as a PAL console (`region=psx.eu`), the log's console being PAL-clocked (D-0); ~~the sync-mode tables not graded~~ *since 2026-10-11* each of the 14 sync tables in the log's shape: every row's first sample at most 1.2 times the log's largest first sample in that table and 2 more, every rise at most 1.1 times its largest rise and 2 more, the rows of modes 0 and 3 never falling (§11.6) |

A run with no `tty` space, the references' through the probe among them, is `NoDump` (§11.15).

**§11.12a The `timers` rule and the kernel's phase** (decided 2026-10-12, `Diamond_OpenKernel_Phase2.md` §25.5 Q8). Two
kinds of `timers` row measure where an interrupt falls against the program's own loops rather than a timer: a delay row
reads an interrupt's whole cost in the sample it lands in, and a frame's hblank count loses or gains one edge when the
program's reset of the counter falls within the few dozen cycles after an hblank's edge. Both are set by the kernel's
timing modulo a frame and a line (its time to the hand-over, the cost of its calls and of its interrupt path), which no
document gives; the console's log is one draw of them, on a BIOS whose timing is not known. So a delay sample longer
than the log's mean by more than the row's tolerance is taken as one an interrupt landed in and dropped, at most one a
row (an interrupt only lengthens a delay, and a vblank fits in a row of 10 samples of 5,000 cycles once); a second, or
any sample short of the mean, still fails; and a frame's hblanks may differ from the log's by 1. The rule applies to
every kernel image, the player's BIOS too. `The_timers_rule_drops_one_interrupted_delay_sample_and_allows_an_hblank_either_way`
pins it [M]: one interrupted sample passes, two or a short one fail, 262 hblanks against 263 passes and 261 fails. With
it `timers` passes on the open kernel's image of 2026-10-12 (`4e01c2ac…`), which drew a vblank into its fifth 5,000-cycle
sample and 262 hblanks a frame, and still passes on the image `7af83c9b…` of 2026-10-11, which drew neither.

### 11.13 The project's own programs, and the witnesses

Each in `programs/`, assembled by `emusen-mips`, runs with no kernel or on one (it installs what it needs, with
interrupts off), writes its results from `80100000h` and a done word at `800FFFF0h`, and is pinned on Diamond by
`tests/programs.rs`. The witnesses are the three reference cores through the stock probe (Beetle PSX and SwanStation
on the player's BIOS by `--sysdir`, PCSX-ReARMed on its own kernel), their RAM read at frame 600 (120 for
PCSX-ReARMed); their logs were not read.

| Program | Diamond | Beetle PSX | SwanStation | PCSX-ReARMed |
|---|---|---|---|---|
| `exceptions.s`, CAUSE after twelve exceptions | CE the opcode's low two bits on every one; 12 taken | the same on all eleven it took; **no CpU for LWC2 with CU2 clear** | the first three, then no address error on misaligned LW, LH, SH, SW and no CpU for SWC3, MFC2, LWC2: 5 taken | RI for ADD's overflow, then stops |
| `loads.s`, the load delay's corners | a second load cancels the first (55h); the instruction's own write wins; LWL/LWR merge with the pending value; MFC0 through the delay cancels a pending load | the same, all eight | the same, all eight | the first load lands (11111111h); MFC0's case reads 0 |
| `cop0-garbage.s`, MFC0 of 16–31 | 0 | values with a period of eight registers (10800h, 0, 304h, C1h, 304h, 1F800000h, 8, 4), and 10800h after a delay or after EPC; *read again 2026-10-11 (`Diamond_Referees.md` §4.12): the registers left unchanged, those being the program's own earlier values* | 0 | 0 |
| `multiply.s` (D-2) | §11.14 | +2 cycles on every case | 1 on every case: no latency | 0: no timing |

`BlackMoon_R3000.md` §8 and §10 record what each means for the processor.

### 11.14 The gate, this half

**The open kernel, no firmware folder** [M, `Ps1_tests_pass_by_their_tty_on_the_open_kernel`, 2026-10-11]:

| Program | Result |
|---|---|
| `cpu/cop` | pass, 17 of 17 |
| `cpu/code-in-io` | pass, 7 of 7 |
| `dma/otc-test` | pass, 15 of 15 (the console's log lists 13; the release adds two) |
| `dma/dpcr` | pass, 2 of 2 and both time-outs |
| `dma/chain-looping` | pass: 16,024 / 16,016 / 24,032 / 24,024 cycles against 16,040 / 16,032 / 25,632 / 25,640, the booleans equal |
| `timers` (as a PAL console) | pass, 18 of 18 rows; *since 2026-10-11* and 14 of 14 sync tables in the log's shape (4 before D-6's second part) |
| `cpu/access-time` | pass, 17 of 17 rows within 0.5 cycles, since the kernel's printf takes a flag after the width (`Diamond_OpenKernel.md` §4.10, 2026-10-11); before that its `%2-d` printed `-d` and every cell's hundredths were lost |
| `cpu/io-access-bitwidth` | not graded: a table to compare by eye; it differs from the console's log in the registers' upper "garbage" bits (16–31 of I_MASK, the timers' and SIO's), which Diamond reads as zero, and in DEV1, which reads FFh where the console's DTL-era window holds what was written |

**A player's BIOS** (`Ps1_tests_pass_by_their_tty_on_a_players_bios`, the American BIOS for NTSC and the European one
for `timers`): all seven pass, `access-time` with 50 of 51 cells within 0.20 cycles (§11.3).

**Bare-metal programs**: Peter Lemon's 72 programs, 120 frames each, Diamond's RAM against PCSX-ReARMed's from 10000h
up [M]: **64 identical but for the 16 bytes above the initial stack, which PCSX-ReARMed's own kernel writes**; GTEAVSZ,
GTENCLIP and Cube differ in the GTE's results (stage 5; *at stage 5 the first two pass and Cube is found to use
no GTE and to wait on the pad, §13.5*); DCTBlockDecodeCLUT4BPP and 8BPP in the MDEC's (stage 9);
VBLANK, hit-green, PSXNICCC and ROOTEXMP in counters and buffers paced by the frame, which the two engines time
differently (PCSX-ReARMed reports 60 Hz). The project's four programs run as §11.13 records.

**D-2**, measured by `multiply.s` [M]: on Diamond, MULTU 6 cycles for rs below 800h, 9 below 100000h, 13 above; MULT
the same by the magnitude of a signed rs, 13 for 80000000h; DIV and DIVU 36 whatever the operands, by zero included;
MFLO after MULTU of a small rs and N NOPs waits 6 − N cycles, none from N = 5. Beetle PSX reads 2 more on every
case and 8, 7, 6, 5, 4, 3 then 1 for N = 0..6; SwanStation and PCSX-ReARMed time nothing. **Open**: the witnesses
disagree with each other and with the documents' reading by two cycles; the console decides (`BlackMoon_R3000.md` D-2).

**The stage-2 open items** (`BlackMoon_R3000.md` §10): checked by the project's programs against the witnesses
(§11.13); **amidog's CPU and CPX tests wait on the GPU** (stage 4): they report on screen, and print only "args: 0" to
the TTY, on either kernel.

**What waits on whom**:

- on the open kernel's writer: `cpu/access-time`'s hundredths (the printf prefix above);
- on stage 4's GPU: amidog's CPU and CPX tests, so the CE bits, the load-after-load corner and the garbage registers
  on an independent program;
- ~~on a console (Q16): D-0's line, D-2, D-4's SPUCNT cell, D-5's gaps and the store's drain, D-6's mode 2;~~
  decided 2026-10-11: no console is used; these went to the referees of `Diamond_Referees.md` §4, which ruled on D-0's
  line and D-2's shape and left the rest open;
- on stage 7's CD-ROM: the 240p test suite, a disc (`Diamond_References.md` §8).

**The kit**: C1–C15 pass on the synthetic PS-EXE and the synthetic disc at 120 frames, now on the open kernel
(`The_kit_passes_on_a_synthetic_program_and_a_synthetic_disc`), the state 3,743,658 bytes.

### 11.15 The references and ps1-tests

The kernel-calling programs ran on the references (Beetle PSX and SwanStation on the player's BIOS, PCSX-ReARMed on
its own) but **their TTY cannot be read**: libretro carries no TTY, a retail BIOS sends standard output to its dummy
device, and PCSX-ReARMed's log at every level holds none of it [M, `cpu/cop`]. The reference cores' logs of runs on
the player's BIOS were not read, since they may describe the BIOS's own execution. The witnesses for this half are
therefore RAM-recording programs (§11.13). Also found [M]: the stock probe segfaults on PCSX-ReARMed with
`cpu/access-time` and on SwanStation with `cpu/cop` at some frame windows, and PCSX-ReARMed refuses a `.psexe`
name, which the probe then reports as needing firmware; `.exe` runs. Neither was investigated further.

### 11.16 The state, the spaces, the cost

- **Spaces**: RAM, SCRATCH, VRAM, SPURAM, BIOS (read-only, the kernel running) and **TTY** (6, 64 KiB, read-only).
- **State** `DMND` version 1, redefined (no player holds one): the frame count, `Cpu`, `Gte`, `Bus` (the clock, the
  write queue, the DMA's bus times, POST, memory control, BCC with the cache's tags and words, I_STAT and I_MASK, the
  three counters with their ticks, holds and flags, the DMA's registers and its piece in flight, the video timing's
  segment and field, the stubs' registers, the DUART and the TTY's length), `ExePending`, then RAM, the scratchpad,
  VRAM, sound RAM and the TTY: **3,743,658 bytes**, pinned by `the_version_1_layout_is_pinned`; a state from before
  the side-load side-loads again.
- **Cost** [M]: 6.0 ms a frame on the open kernel (`timers`, 600 frames in 3.60 s, three runs) and the same bare
  (CPUADD), against the plan's §5.12 budget of 3.8 ms; speed is stage 13's.

### 11.17 Tests run (2026-10-11)

`cargo test --release`: `diamond` 37 unit tests and 4 program tests; `blackmoon-r3000` 25, the r3000 suite 55,000 of
55,000 with `EMUSEN_BLACKMOON_CORPUS`, after the bus error; `cargo clippy --all-targets` clean on both. WiseMan,
after merging WiseMan at `aeb9610b`: `DiamondTests` 13 of 13 with the corpus and the firmware folder named, and the
blast radius (`CoreAbi`, `CoreAdapter`, `CoreDiscovery`, `CoreRegistration`, `SystemPack`, `Nephrite`, `TestRom`),
139 of 139 with the open-kernel test's one known waiting program.

### 11.18 Negative results

- The references' TTY is out of reach (§11.15).
- amidog's tests print nothing gradeable to the TTY (§11.14).
- No document gives the I-cache's refill timing, the write queue's drain, or the DMA's gaps; each is a choice or a fit
  named in D-4 and D-5.

---

## 12. Stage 4, the GPU (2026-10-11)

The plan's §7 row 4: GP0 and GP1, the FIFO, VRAM's transfers by the CPU and by DMA channel 2 in its three modes, every
primitive, texturing with its caches, blending, the mask bit, dithering, the drawing area and offset, the display at 15
and 24 bits with interlace, GPUSTAT and the drawing time, and the picture handed to the host. Its record is
`Diamond_GPU.md`; this section says what changed for the machine and what the stage measured. Measured on 2026-10-11 on
the desktop [M] unless marked.

### 12.1 What was built

- **`src/gpu/`**, seven modules of one job each (`Diamond_GPU.md` §1), in place of stage 3's `GpuStub` (§11.8).
- **The bus** gives the GPU its ports and IRQ1, an event at each moment its FIFO moves, and DMA channel 2's words at
  the channel's rate, each of which the GPU may hold back; blocks and list nodes wait for the GPU's request without
  holding the bus (`Diamond_GPU.md` §2.4). A word the GPU holds back lengthens the piece it belongs to.
- **The frame** ends as a field's vblank begins, or at the field's end with no vblank (§11.5 superseded), so the picture
  handed over is complete (`Diamond_GPU.md` §7). Decided 2026-10-11: kept.
- **The picture** is the display's: the display area at its own size, 15- or 24-bit, interlaced fields woven, a 4:3
  aspect, at most 640 by 576 (machine info's maximum, raised from 512 lines).
- **`programs/gpustat.s`**, GPUSTAT's ready bits in ps1-tests' `gpustat`'s four cases (`Diamond_GPU.md` §2.3), and
  `examples/run` gains `--vram` and `--picture`.
- **WiseMan**: `VramGrader` and `Ps1GpuTests` (`Fixtures/RomRunner/VramGrader.cs`), the plan's §3.3 VRAM grader with
  ps1-tests' eleven programs as data and their named disagreements; `Ps1Tests` gains `gp0-e1`, `mask-bit`,
  `bandwidth` (a rule per row of hblanks) and `version-detect` (an expected line), each with its own frame.

### 12.2 The gate

As `Diamond_GPU.md` §9 tables it [M]: **ps1-tests' eleven GPU programs with a picture, VRAM exact** but for 64 pixels
of `lines` and `transparency`'s VRAM outside its screen, both named in D-12 as the programs' own; `gp0-e1` 10 of 10,
`mask-bit` 5 of 5, `version-detect`, and `bandwidth`'s fourteen rows within 0.6% of the console's; GPUSTAT's twelve
readings as `gpustat`'s log. Stage 3's seven programs still pass. **amidog's GPU test waits on the pad** (stage 6): its
menu draws and waits for Start (`Diamond_GPU.md` §10). **Peter Lemon's 25 GPU programs**: 16 equal SwanStation's
picture up to its colour levels, five differ by one level on some shaded pixels, four only inside the rectangles they
flip, which the references do not flip and a console does (`Diamond_GPU.md` §11).

### 12.3 amidog's CPU and CPX tests, and stage 2's items

`psxtest_cpu` runs to its end with every column OK or N/A, its delayed- and advanced-load groups among them, which
confirms on an independent program the load delay's corners `BlackMoon_R3000.md` §10 left to it. `psxtest_cpx` waits
for Start, so the CE bits and the COP0 registers 16-31 wait with it on stage 6 (`Diamond_GPU.md` §10).

### 12.4 What else the stage measured

- **`dma/chopping`** keeps 129 of 130 rows within 10%, and its "(GPU busy)" mark now agrees with the console's on 117
  rows, against 60 with the stub; the 13 others are D-11.
- **The disputes** D-7 to D-12: the rasteriser's rules where psx-spx is silent (D-7), the x-flip's offset (D-8),
  GP0(E3h)-(E5h) in order (D-9), GPUSTAT bit 28 (D-10), the drawing time with the busy state after long transfers open
  (D-11), and the two pictures the release's programs cannot meet (D-12). None needs the FPGA referee yet: each open
  point is a console measurement.
- **Cost** [M, `examples/run`, 600 frames, three runs each]: 6.1 ms a frame on `timers` (stage 3: 6.0), 6.3 on
  `triangle`, 5.3 on `rectangles`, 7.8 on `bandwidth`, whose 400 full-screen primitives a row leave the GPU no idle
  frame. Speed is stage 13's.
- **The state** 3,747,681 bytes (`Diamond_GPU.md` §12).

### 12.5 For `EmuSen_CRT.md` §20.6

The PlayStation's display timing now exists in the core: the picture's width is GP1(06h)'s range over the dot clock of
GP1(08h)'s mode (256, 320, 368, 512 or 640 at the standard ranges), its height the vertical range's lines, doubled in
480-line interlace, on D-0's clocks with D-0's line still provisional. The CRT row itself waits as §20.6 decided.

### 12.6 Tests run (2026-10-11)

`cargo test --release`: `diamond` 53 unit tests and 5 program tests; `cargo clippy --all-targets` clean. WiseMan, under
the test lock with `EMUSEN_DIAMOND_PS1TESTS`: `DiamondTests` 15 of 15, and with the blast radius (`CoreAbi`,
`CoreAdapter`, `CoreDiscovery`, `CoreRegistration`, `SystemPack`, `Nephrite`, `TestRom`), 141 of 141. The player's-BIOS
gate was not run (no `EMUSEN_DIAMOND_FIRMWARE` named); the probe ran the three references on Peter Lemon's programs.

### 12.7 Negative results

- amidog's CPX and GPU tests cannot run without the pad.
- Beetle PSX's pictures of Peter Lemon's programs are displaced by 150 to 160 lines and were not graded.
- No program run here observes the texture cache's staleness.

---

## 13. Stage 5, the GTE (2026-10-11)

The plan's §7 row 5: the GTE, exact with its timing. The engine's own record is `../BlackMoon-HW/BlackMoon_GTE.md`;
this section says what changed for the machine and what the stage measured. Measured on 2026-10-11 on the desktop [M]
unless marked.

### 13.1 What was built

- **`blackmoon-gte`**, built (`BUILT` true): the registers with their read-back quirks, the 22 commands with sf and lm,
  MVMVA's every selection with the far colour's fault and the garbage matrix, FLAG with its summary bit, the UNR division
  with its table, the cycle table, and an observer seam for the enhancement layer (`BlackMoon_GTE.md` §2–§9). The
  transfers no longer return `NotBuilt`; the registers are behind the transfers (`registers()` for the state).
- **The interlock in Diamond's bus**: `gte_ready`, the cycle the command in flight is done; every one of the five
  coprocessor-2 calls waits for it, and a command sets it from the bus's clock after its fetch (`BlackMoon_GTE.md` §6,
  D-13). **Nothing changed in `blackmoon-r3000`**: its `Bus` named the five calls from stage 1, and MFC2's and CFC2's
  load delay, the CU2 check and the command of an instruction an interrupt pre-empts were already the processor's.
- **`programs/gte-timing.s`**, the GTE's timing and hazards in five tables, and **`programs/gte-fuzz.s`**, 904 fuzzed
  cases in the manner of ps1-tests' `gte-fuzz` (`BlackMoon_GTE.md` §7.3), each pinned by `tests/programs.rs`.
- **WiseMan**: `Ps1Tests` gains `gte/test-all`, graded by its console log's two lines at frame 300.
- The machine info's description names the GTE among what runs.

### 13.2 The gate

| Program | Result |
|---|---|
| ps1-tests `gte/test-all` | **1,150 of 1,150 bit for bit**, in the crate from its table and through Diamond on the open kernel by its TTY (`Passed tests: 1150`, `Failed tests: 0`) |
| amidog `psxtest_gte` | **waits on the pad**: its menu is navigated with the D-pad (its readme); the pad is stage 6's, on branch `diamond-pad` |
| `gte-fuzz` against the references | `gte-fuzz` itself starts on Start and reports by TTY, so the project's own `gte-fuzz.s` in its manner: **the 880 cases of the 22 commands identical with Beetle PSX, SwanStation and PCSX-ReARMed**; the 24 of six N/A numbers identical with Beetle PSX and different from the other two (D-16) |

Four disputes: D-13 (the interlock, open), D-14 (the accumulators, settled by `test-all`), D-15 (the write hazards,
open) and D-16 (the unlisted numbers, open), all in `BlackMoon_GTE.md` §12.

**P8** (the plan's §13): "The GTE passes `test-all` and amidog's test bit for bit within its stage, with no more than two
disputes." Not retired: the `test-all` half held within the stage; amidog's half waits on the pad; and the disputes are
already four, so its count was wrong, though only D-14 concerns a result the gate grades, the other three timing and
corners the gate does not reach.

### 13.3 How the first build failed

psx-spx's plainest reading missed 358 of `test-all`'s 1,150 cases; four readings found from the misses (IR saturated from
the 32-bit MAC; the same in the depth cue; the 44-bit check after each addition; the accumulator wrapping at 44 bits)
brought it to none. The order, the evidence for each and the table of what removing each rule costs are
`BlackMoon_GTE.md` §5.2 and D-14.

### 13.4 Timing, against the witnesses

`gte-timing.s` [M]: on Diamond, a read at once after a command waits the command's cycles less one, a read N
instructions later N fewer, and every coprocessor-2 access waits, the three writes included. Beetle PSX agrees on every
count but AVSZ4's (5 where psx-spx gives 6) and holds the writes too; SwanStation counts one cycle more everywhere and
holds the writes; PCSX-ReARMed times nothing. The writes' wait is a choice argued in D-13, and the boundary is open as
D-2's is, until amidog's timing tests or a console.

*Revised 2026-10-11, after amidog's timing tests* (`BlackMoon_GTE.md` §6, §7.2, D-13, D-16): an access that waits now
resumes one cycle after the command ends, so a read at once costs the command's whole count and a read N instructions
later N fewer, until at N = cycles − 1 it costs nothing; the numbers psx-spx lists as N/A take the cycles amidog's table
gives them. Every row of the test's OFFICIAL TIMING passes, and 40 of 42 of UNOFFICIAL TIMING.

### 13.5 Peter Lemon's programs

`GTEAVSZ` and `GTENCLIP` pass every row and draw their bundled pictures. `Cube`, counted at stage 4 among the programs
differing in the GTE's results (§12.2), uses no GTE instruction: it waits for the pad buffer the kernel's pad driver
fills, so it waits on the pad (`BlackMoon_GTE.md` §7.4).

### 13.6 The state

`DMND` **version 2** (decided 2026-10-11): the bus block gains `GteReady`, after `CpuBusy`, so a state taken while a
command runs resumes with the processor still held; the `Gte` block is unchanged at 256 bytes (`BlackMoon_GTE.md` §4
retires stage 1's forecast of it). **3,747,689 bytes**, pinned by `the_version_2_layout_is_pinned`; a version 1 state is
refused as a version, as §2.7 provides.

### 13.7 The cost

[M, `examples/run`, 600 frames, three runs each, WiseMan at `bf297ca8` against this stage, built side by side]:

| Program | Before (ms a frame) | After |
|---|---|---|
| `timers` | 6.33 | 6.23 |
| `gpu/triangle` | 6.26 | 6.30 |
| `gpu/rectangles` | 5.48 | 5.46 |
| `gpu/bandwidth` | 8.01 | 7.90 |
| Peter Lemon's `GTEAVSZ` (bare) | 6.14 | 6.18 |

Each difference within the runs' own spread (0.4% to 7.2% between the fastest and slowest of three): the five calls' comparison against `gte_ready` costs nothing
measurable, and the bench programs issue few commands. The commands themselves cost 3 to 114 ns each
(`BlackMoon_GTE.md` §10); nothing allocates.

### 13.8 Tests run (2026-10-11)

`cargo test --release`: `blackmoon-gte` 9 with `EMUSEN_BLACKMOON_CORPUS` (`test-all`'s 1,150 cases among them);
`diamond` 53 unit tests and 7 program tests (`gte-timing.s` and `gte-fuzz.s` added). `cargo clippy --release
--all-targets` clean on both crates (Fedora's 1.98.1). `blackmoon-r3000` was not touched and its tests not rerun.
WiseMan, under the test lock with `EMUSEN_DIAMOND_PS1TESTS`: `DiamondTests` 15 of 15, `gte/test-all` passing among
ps1-tests' twelve TTY programs and every earlier program still passing; the player's-BIOS gate was not run (no
`EMUSEN_DIAMOND_FIRMWARE` named). The rest of the blast radius was not rerun: no C# but `Ps1Tests`' data changed.

### 13.9 What waits

- On the pad (stage 6, `diamond-pad`): amidog's GTE test, with its timing tests, which grade D-13 short of a console;
  `gte-fuzz` itself; Peter Lemon's `Cube`.
- On a console (Q16): D-13's writes and boundary, D-15, D-16.

---

## 14. Stage 6, SIO0's devices (2026-10-11)

The plan's §7 row 6, built on the digital pad brought forward the same day; its record is `Diamond_Pad.md`. Stage 6 had
its go on 2026-10-11, with one change to the gate: no reference emulator was run, and the card's round trip against a
reference's card is a separate step, for which the card a program wrote is kept (`Diamond_Pad.md` §8).

### 14.1 What was built

- **`src/pad.rs`**: the digital pad and the DualShock (modes, configuration commands, LED and lock, both motor methods,
  the Analog button); **`src/card.rs`**: the memory card's Read, Write and Get ID with the flag, the checksum and the
  read's late /ACK; **`src/multitap.rs`**: four slots each of controllers and cards, the long read's toggle;
  **`src/sio0.rs`** routes each port's address byte to its controller, card or multitap slot.
- **The core ABI**: ports 0-7, the settings `tap1`, `tap2`, `card1`, `card2`, the Analog button as bit 16 with no
  canonical control, `set_axis` (`AXES`), the cards as battery files 0 and 4 (`BATTERY_DIRTY`).
- **The open kernel** reads both ports over SIO0 on each vblank (`Diamond_OpenKernel.md` §4.13); its image's SHA-256 is
  `135549cd26268e0a50f56649ac4a1cdfdf6287ca2660698023648bc3bf011629`.
- **Programs**: `card.s`, `multitap.s`, `kernel-pad.s` beside `pad.s`.
- **The state**: `DMND` version 4, 3,747,794 bytes with a DualShock on port 1 and no card; each card adds its 131,072
  bytes. *Version 5 since the GPU's cache step (2026-10-11), 3,747,806 bytes: the GPU's repeated GPUREAD word, a
  transfer's first word and the texture cache's page (`Diamond_GPU.md` §12).*

### 14.2 The gate

`Diamond_Pad.md` §11's table: the kernel's pad functions, the cards through the protocol and as raw 128 KiB battery
files, the multitap's four ports and the DualShock's modes met; the round trip against a reference's card not run here. Peter
Lemon's `Cube`, which waited on the pad at stage 5 (§13.9), now answers it on the open kernel.

### 14.3 Tests run (2026-10-11)

`cargo test --release`: `diamond` 70 unit tests and 17 program tests (`card.s`, `multitap.s`, `kernel-pad.s` and
`pad.s` among them); `diamond-kernel-harness` 49 with `DIAMOND_KERNEL_REBUILD=require`; `blackmoon-r3000` 28 with
`EMUSEN_BLACKMOON_CORPUS` (the 55,000 single-step cases among them). `cargo clippy --release --all-targets` clean on
`diamond` (Fedora's 1.98). WiseMan, under the test lock with `EMUSEN_DIAMOND_PS1TESTS`: `DiamondTests`, `CoreAbi`,
`CoreAdapter`, `CoreDiscovery`, `CoreRegistration`, `SystemPack` and `WiseMan.Input`, 185 of 185, amidog's opt-in tests
among them. No reference emulator was run.

### 14.4 What waits

- The card's round trip against a reference's card, with `~/.cache/emusen/probe/diamond/cards/diamond-card-1.mcr`.
- The multitap's slots B-D's cards (no battery numbers yet), the DualShock's watchdog, the DualShock 2's commands, a
  pad combination for the Analog button, rumble's event (`EmuSen_CoreAPI_Extensions.md` §4).
- On a console (Q16): D-22, SIO0's timing, and the card's late /ACK.

---

## 15. Stage 7, the CD-ROM and the shared disc crate (2026-10-11)

The plan's §7 row 7, with the tester's go of 2026-10-11. Its record is `Diamond_CDROM.md`; the disc crate's is
`../../Shared/EmuSen_Disc.md`.

### 15.1 What was built

- **`emusen-disc`** (`EmuSen/Cores/Shared/`): a disc as sectors from a cue sheet, an `.iso`, a lone raw track or a CHD
  (through the `chd` crate, unread, its CD layout measured from chdman's output); ECMA-130's EDC and RSPC; subchannel
  Q; the TOC; `.sbi`; the set module's readers.
- **The controller** (`src/cdrom/`): the decoder's host interface, every command of psx-spx's summary with its first and
  second responses and their delays (D-24), the drive's spin-up, seeks by distance and held head (D-25), reads at 75 and
  150 sectors a second with the buffer's oldest and newest, the file and channel filter, XA-ADPCM with its
  interpolation to 44,100 Hz (D-27), CD-DA with reports and AutoPause, the volume matrix, subchannel Q and LibCrypt by
  `.sbi`, the lid, SCEx for the console's region, the firmware's date, DMA channel 3. Its audio leaves at 44,100 Hz for
  the SPU of stage 8.
- **The create** (`src/medium.rs`): a sheet with its tracks as files from 65,536 (`EMUSEN_FILE_MEDIA`), a CHD, an
  `.iso` or a lone track; refusals −11 and −12.
- **The pack**: `.chd` routed; `.cue`, `.iso`, `.bin`, `.m3u`, `.pbp` declared; the recogniser `PsxSystem.IsDisc`.
- **The state**: `DMND` version 6, 3,788,042 bytes (§10 of the CDROM page).
- **Programs and tools**: `cd-read.s` (the kernel's read sequence, polled), `cd-kernel.s` (the kernel's CD functions);
  the run tool's `--disc`, `--boot`, `--trace-game`, `--swap`, `--pictures` and the rest.

### 15.2 The gate

| Item | Result |
|---|---|
| ps1-tests' `cdrom/getloc` | **passes** by its own verdict on the player's BIOS with one of the tester's discs, every Getloc succeeding or failing as on the console; the eighth byte of GetlocP prints as zero (D-28) |
| ps1-tests' `cdrom/disc-swap` | **every line of the console's log** but the last, whose wording is an older build's |
| Synthetic discs read sector for sector | **met**, at both speeds, user data and whole sectors, by the controller's tests and by DMA in `cd-read.s` |
| XA and CD-DA streams equal to their sources | **met**: XA's decoded samples exactly, its 44,100 Hz frames equal to the source through psx-spx's interpolation; CD-DA frame for frame |
| Games boot on the player's BIOS to their title screens | **not met the console's way** (D-29: the intro stays on its first screen); **through the kernel's own LoadExec** Crash Bandicoot and Spyro the Dragon reach their title screens, equal to Beetle PSX's, SwanStation's and PCSX-ReARMed's within the animation's phase; five others reach their opening movies, which wait for the MDEC (stage 9) |
| Load time and the copy at create | 0.35–0.57 s on the desktop for 432–662 MB, the copy acceptable there; the handheld's measurement left with what it needs |

Beyond the gate, ps1-tests' `cdrom/timing` prints every figure within the console's range.

### 15.3 Defects found and fixed in the stage, each by a test

- BFRD written twice restarted the data FIFO and the DMA reading it (`bfrd_set_again_…`).
- A Read with no new Setloc began at the last delivered sector rather than where the seek left the head.
- Clearing the result FIFO on acknowledge, psx-spx read literally, left a PSn00bSDK program one byte of each response.

### 15.4 Tests run (2026-10-11)

After merging WiseMan at `e9ebe218`: `diamond` 101 unit tests and 41 program tests, all passing with the disc inputs;
`emusen-disc` 19 with chdman named; WiseMan's `DiamondTests`, `SystemPack`, `CoreAbi`, `CoreAdapter`, `CoreDiscovery`
and `CoreRegistration`, 94 of 94.

### 15.5 What waits

D-29; the framework's items of `Diamond_CDROM.md` §8.4 (the header constant, the runtime's companions, routing by
content, archives); `.m3u` and the disc change at stage 11; the handheld's load measurement; a LibCrypt disc.

---

## 16. Stage 9, the MDEC (2026-10-11)

The plan's §7 row 9, with the tester's go of 2026-10-11. Its record is `Diamond_MDEC.md`.

### 16.1 What was built

- **The MDEC** (`src/mdec/`), replacing stage 3's stub: MDEC0's commands and parameters, MDEC1's status and reset,
  the input FIFO held from full until emptied, the run-length decoder, one block in the IDCT, the block buffer and a
  17-word output FIFO (D-32), a read that waits for the block being computed (D-33), 512 cycles a block (D-31); the
  arithmetic of psx-spx's decode with each pass of the IDCT truncated toward zero and the 15-bit pixel rounded from the
  channel's fraction (D-30); 4-, 8-, 15- and 24-bit output, signed or not, bit 15.
- **DMA channels 0 and 1** (`bus.rs`'s `mdec_piece`): word by word while the MDEC can take or give, a waiting block
  letting go of the bus, DMA1 writing colour blocks in their macroblock's order. The other channels are unchanged.
- **The state**: `DMND` version 7, 3,789,828 bytes (`Diamond_MDEC.md` §6).
- **A program**: `programs/mdec-timing.s`, the decode time by DMA on five streams.

### 16.2 The gate

| Item | Result |
|---|---|
| ps1-tests' `mdec/4bit` | **VRAM exact** |
| `mdec/8bit` | one pixel differs at five bits (D-30) |
| `mdec/frame` | the DMA builds differ by D-30's colour (11,035 pixels at 15 bits, 63,378 halfwords at 24); the CPU builds wait after their last word by their own logic, as a console would |
| `mdec/movie` | 319 and 752 differ (D-30), each picture met at one frame of the run |
| `mdec/step-by-step-log` | 414 of 422 status lines as the console's log (D-32) |
| Peter Lemon's DCT decode | `CLUT8BPP` 3 pixels off SwanStation's picture; `CLUT4BPP`'s three blocks behind the texture cache (D-23) |

**Not met exactly**: the IDCT's and the colour's rounding is open (D-30), with a search of its fixed point recorded.

### 16.3 What the tester's games now do

Rayman's opening movie plays at the referee's pace; Tekken 3 reaches its title screen and attract mode, its movie
skipped; Castlevania: SotN, Silent Hill and Ape Escape stop where they did, the first two before any decode the MDEC
could act on (`Diamond_MDEC.md` §9). D-29 is unchanged.

### 16.4 Tests run (2026-10-11)

`diamond` 109 unit tests and 45 program tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, all passing; WiseMan's
`DiamondTests`, `SystemPack`, `CoreAbi`, `CoreAdapter`, `CoreDiscovery` and `CoreRegistration`, 95 of 95 with the corpus.

### 16.5 What waits

D-30 to D-33; the streams behind three games' movies; the SPU's stage 8, built beside this one, with which the state's
version and the DMA's channels are merged.

---

## 17. Stage 8, the SPU (2026-10-11)

The plan's §7 row 8, with the tester's go of 2026-10-11, built on WiseMan after stage 9's merge. Its record is
`Diamond_SPU.md`.

### 17.1 What was built

- **The SPU** (`src/spu/`), replacing stage 3's register stub in `devices.rs`: the 24 voices with SPU-ADPCM decoded a
  word at a time (D-34, D-40), the Gaussian interpolation from psx-spx's table transcribed by a script (D-35), pitch and
  pitch modulation, ADSR and the sweeps on psx-spx's envelope step, the envelope's step after its sample (D-36), noise
  (D-37), the loop flags, ENDX and LSAX, key on and off at the next sample, the reverb at 22,050 Hz with its accesses
  in the state machine's order and its 39-tap filter (D-39), the capture buffers, IRQ9 on every access to IRQA's unit,
  the FIFO at a halfword every 32 cycles (D-38), CD audio in from stage 7's ring, MVOL, AVOL, mute and enable.
- **The audio path**: one sample every 768 cycles into `emusen-native`'s `SampleQueue`, which the core ABI's
  `drain_audio` empties; the machine's silence counter and its CD tap removed.
- **DMA4** (`bus.rs`): paced by the SPU's request, its waits letting the other channels move (`Diamond_SPU.md` §6.3, a
  defect the stage's first build had and a test shows).
- **The state**: `DMND` version 8, 3,792,964 bytes.
- **Programs and tools**: `programs/spu-voices.s`, `spu-capture.s` and `spu-dma.s`; the run tool's `--wav`;
  `psxreferee.py wav`, a libretro referee's audio; `psxaudio.py`, two recordings compared sample for sample; WiseMan's
  TTY gate grades `spu/memory-transfer` and `spu/test`, the second by a new rule, `Exactly`.

### 17.2 The gate

| Item | Result |
|---|---|
| ps1-tests' `spu/memory-transfer` | **9 of 9 pass** on the open kernel, its DMA timing tests among them, as on DuckStation |
| ps1-tests' `spu/test` | **no mismatch** at either width |
| Synthetic voice programs against the references' audio | `spu-voices.s`'s scenes equal SwanStation's but where a register write lands a sample apart on the two machines, by their vblanks' cycles (D-0); `spu-capture.s`'s records, synchronised to the SPU's own IRQ, **equal DuckStation's sample for sample in every voice test** (decode, interpolation, envelopes, pitch modulation, reserved filters, the key on), and Beetle PSX's and SwanStation's in the envelope tests |
| The reverb against the references | the work area equals DuckStation's in every stage of the side whose input instant coincides (D-39); the audio's reverb scene within 148 of SwanStation's, mean 1.2 |
| `spu/stereo` | not run: graded by ear only |

**Not met exactly**: D-38's rate and D-39's instants are open between the referees, and Beetle PSX's interpolation at
fractional indices differs from every reading tried.

### 17.3 The games, and D-29

D-29 unchanged by the SPU and by five variations of its answers, by the screen only. Through `--boot`, the seven games
do what they did at stage 9, Crash Bandicoot once DMA4's waits let the CD-ROM's channel move; Tekken 3's and
Castlevania's malformed MDEC words are the same (`Diamond_SPU.md` §11).

### 17.4 The cost

0.245 ms a frame for the SPU with every voice and the reverb running; the bench programs within their spread and Crash
Bandicoot 2–4% slower over 3,600 frames (`Diamond_SPU.md` §12).

### 17.5 Tests run (2026-10-11)

`diamond` 133 unit tests and 49 program tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, all passing; WiseMan under
the test lock with the corpus: `DiamondTests`, `SystemPack`, `CoreAbi`, `CoreAdapter`, `CoreDiscovery` and
`CoreRegistration`, 95 of 95, the TTY gate grading the two SPU programs among them.

### 17.6 What waits

D-38 to D-40; the unstable DMA read and RAM_CTRL's other bank sizes; external audio; D-29.

## 18. Stage 10, the open kernel's second phase (2026-10-11)

The plan's §7 row 10, with the tester's go of 2026-10-11, built on stage 9's merge and joined with stage 8's. Its record
is `Diamond_OpenKernel_Phase2.md`, written under the replacement writers' rules.

### 18.1 What was built

- **The kernel** (`kernel/src/`): the CD driven at its host interface with sectors by DMA channel 3 (`cdrom.rs`), ISO
  9660 on "cdrom:" (`cdfs.rs`), the memory cards' sectors on vblank (`card.rs`, over `sio.rs`, the byte exchange the pads
  now share) and their on-card file system on "bu00:" and "bu10:" (`bu.rs`), patterns with psx-spx's wildcards
  (`names.rs`), the boot by SYSTEM.CNF and the warm boot (`shell.rs`), and the screen with no disc, its font and its
  card manager (`screen.rs`, `font.rs`, `manager.rs`). 222 of the 576 calls implemented; no phase 2 stub left.
- **Two defects found and fixed**, each shown first: StartPAD2 enabled vblank before queueing the element that
  acknowledges it, an endless exception with interrupts on (§16 of the record); the low-level card calls delivered only
  their F0000011h event, where Rayman waits on the callbacks' F4000001h (§13).
- **The crate**: the firmware entry's cost in `v1.rs`, the phase 1 sentence on discs and cards withdrawn and the
  character sets named; the bus's SPU catch-up made public for the run tool. No state change: `DMND` stays version 8.
- **The run tool**: `--calls` (the kernel calls a program makes, by number), `--trace-at FRAME`, `--card` and `--card2`
  (cards from files and back), `--press FRAME:MASK[:FRAMES]`.
- **Tests**: `programs/kernel-cd.s` and `kernel-card.s`, an ISO 9660 builder for tests (`tests/support/iso.rs`), four
  program tests (the record's §17), and the kernel harness's tests brought to phase 2.

### 18.2 The gate

| Item | Result |
|---|---|
| Games boot with no firmware folder | **All seven of the tester's copied discs boot by their SYSTEM.CNF on the open kernel**, each as far as it goes on the player's BIOS through `--boot` (record §18.1): Crash Bandicoot, Spyro the Dragon and Tekken 3 to their title screens, Rayman's opening movie, Silent Hill to its card warning; Castlevania black and Ape Escape past its DUALSHOCK screen to white, Q3 |
| A game saves on the open kernel and loads its save back | **Met**: Tekken 3 formats a blank card, saves ("SAVE OK!") and loads it back ("LOAD OK!"); the card in psx-spx's on-card format (record §18.3) |
| The image rebuilds byte for byte | **Met**: SHA-256 `0efba49b4d1bf011a5b59ce580e7fc29143ac68a0155012952c1791c72b8ef3f`, `DIAMOND_KERNEL_REBUILD=require`, and from a copy at another path |
| Stage 3's seven test programs on the open kernel | **Met**, WiseMan's `DiamondTests` |
| Cards round-trip with the player's BIOS; `firmwarecheck.py`; G9 | ~~**Not this stage's**, a separate step by the stage's terms; what it needs is the record's §20.4~~ Taken the same day (record §22, §18.5 below) |
| The cards a game writes are read by the same game on the player's BIOS, and the reverse | **Met**: Tekken 3's save made on each kernel loads on the other ("LOAD OK!"), and the open kernel's formatted card is a formatted card with no file to the player's BIOS; a blank card fails on both (record §22.1) |
| `firmwarecheck.py` within the policy's limits | **Not met**: FAIL against all 18 images in the firmware folder, by the default thresholds, plain (20.3% to 31.0% equal, runs to 114,386 bytes, all in the zero padding) and on the image's first 92,440 bytes (13.7% to 15.3% equal, the longest run 200 bytes of its own zero fill); the record's Q5 asks for thresholds for a kernel-sized image (record §22.6) |
| G9's open-BIOS condition | **Not met in full**: the games and the cards are; the repetition check is not (record §22.7) |

### 18.3 Tests run (2026-10-11)

`kernel-harness` 49 tests with `DIAMOND_KERNEL_REBUILD=require`, all passing; `diamond` 133 unit tests and 53 program
tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, all passing; WiseMan under the test lock with the corpus:
`DiamondTests`, 24 of 24, stage 3's seven programs on the open kernel among them, with no firmware variable set.

### 18.4 What waits

~~The cross-kernel card exchange and `firmwarecheck.py`; the record's questions Q1 to Q4 (the screened sections, Spyro's
CdRead retries, Ape Escape's DUALSHOCK screen, an exposure to list); no dispute was taken, D-41 on being the next free
number.~~ The exchange and the check are taken (§18.5), Q2 and Q3 answered there; Q1, Q4 and the new Q5
(`firmwarecheck.py`'s thresholds for a kernel-sized image) wait for the tester. No dispute was taken.

### 18.5 The cross-kernel step (2026-10-11)

Its record is `Diamond_OpenKernel_Phase2.md` §22, written for the kernel's writer in terms of what a game sees at the
interface; the player's BIOS was used only as the firmware (`scph5501.bin`, games started through `LoadExec`, D-29) and
as `firmwarecheck.py`'s input.

- **Cards**: met both ways on Tekken 3 (§18.2's rows); the kernels' formats compared frame by frame, with a defect of
  the open kernel's write test found (frame 63 on a new card is not the header). The BIOS's card manager was not
  reached (D-29).
- **Q2**: Spyro prints the same 402 "CdRead: sector error" retries on the player's BIOS, and its CD commands match;
  not the kernel's. **Q3**: Ape Escape leaves its DUALSHOCK screen by itself and turns white on both kernels, its pad
  exchanges equal; the earlier "waits" predates stages 8 and 9.
- **Two documented results differ** (record §22.5): `ChangeClearRCnt(3, 0)`'s first call returns 1 on the player's
  BIOS and 0 here; the games' first `OpenEvent` descriptor is F1000000h there and F1000005h here.
- **`firmwarecheck.py`**: FAIL, §18.2.
- **The run tool** gained `--game`: the game's calls by number with their arguments and results, its own register
  accesses and its text, counted only for code in user RAM from the game's entry on, so that it serves on a player's
  BIOS without observing it (`examples/run.rs`).
- **Tests**: `diamond`'s 145 unit and 56 program tests pass after the run tool's change, merged with stage 12
  (`cargo test --release`); no test was added, the step's results needing the player's BIOS and the tester's discs.

## 19. Stage 12, the debugger (2026-10-11)

The plan's §7 row 12, with the tester's go of 2026-10-11, built beside stage 10 and merged with it. Its record is
`Diamond_Debugger.md`.

### 19.1 What was built

- **The observed frame** (`src/debugger.rs`, `machine::step_on`): the plain step made generic over the R3000A's
  observer, the plain frame passing `Unobserved`; the debugger's frame asks the shared hooks before each instruction,
  records coverage over 24 bits and the profile, reports each store in `R3000BUS` and in `RAM` or `SCRATCH`, notes calls
  at the branch and `jr ra`'s return after its delay slot, and abandons a call an exception interrupts in its slot.
- **The descriptors** (`src/v1.rs`): `DEBUG`, `DEBUG_STACK`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE` claimed;
  `R3000BUS`, id 0, a read-only 4 GiB view of the processor's space read by `Board::peek` (`src/bus.rs`); the R3000A's
  code space; the stores `RAM` and `SCRATCH` report; the GTE's 64 registers beside the R3000A's 45.
- **The listing** by `emusen-mips` with the GTE's names, now a dependency of the crate rather than of its tests.
- **The run tool**: `--armed`, each frame through the debugger's loop with the kit's C15 tables set.
- **No state change**: the debugger keeps nothing in the state, and `DMND` stays version 8; a state loaded at a stop
  closes the open frame.

### 19.2 The gate

| Item | Result |
|---|---|
| Each claim a test | **Met**: twelve crate tests, three program tests over nine tables, nine WiseMan tests through DianaSh's commands (`Diamond_Debugger.md` §4) |
| Every table armed gives the plain run's digests (C15, G7) | **Met**: each of nine tables alone and all together on six of the project's programs, six of ps1-tests' and a copied disc on the open kernel, 117 of 117; the kit's C1–C15 on seven images (§6 of the record) |
| The plain frame's cost unchanged | **Met**: −3.4% to +1.6% on five programs, inside the runs' spread; armed, 18% to 27% more |

### 19.3 Tests run (2026-10-11)

`diamond` 145 unit tests and 56 program tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, and the disc test with
`EMUSEN_DIAMOND_DISC`, all passing; `blackmoon-r3000` with `EMUSEN_BLACKMOON_CORPUS`, unchanged by the stage, passing;
WiseMan under the test lock with the corpus: `DiamondDebugTests`, `DiamondTests`, `CoreAbi`, `CoreDebug`,
`CoreAdapter`, `CoreDiscovery`, `CoreRegistration`, `NephriteDebug`, `DebugPlumbing` and `SystemPack`, 122 of 122.

### 19.4 What waits

Read watchpoints, which need an extension of v1's ranges (`Diamond_Debugger.md` §5.1, proposed for the tester's
decision); the console views outside v1. No dispute was taken.

### 18.5 After the cross-kernel step (2026-10-11)

The cross-kernel step's frame 63 was a half-stored card request taken by the vblank element, shown by a failing test
and fixed (`Diamond_OpenKernel_Phase2.md` §23.1); its two call results are recorded and unchanged (§23.2). The image's
SHA-256 is now `7af83c9be629827e850b4ce450e0d2fcf3400e428923ea2b676918393f5ea1b4`.

### 18.6 Found in play, and fixed (2026-10-12)

The three defects the gameplay step found (`Diamond_OpenKernel_Phase2.md` §24) were each shown by a failing test and
fixed (§25 there): Krom2RawAdd answers the kernel's own letters and a box, so that Castlevania passes its file screen; an accepted asynchronous card
read or write returns 0, so that Spyro the Dragon creates, writes and reads its file; and the exception path runs from
the kernel's RAM with an empty port given up 100 µs after its byte, so that Crash Bandicoot prints no "intr timeout".
The harness gained SIO0 with a memory card and the run tool `--kernel-profile`. Two rows of ps1-tests' `timers` that
depend on the kernel's timing are graded by §11.12a's rule since 2026-10-12 (Q8 there). The image's SHA-256 is now
`4e01c2ac1b8582e46fd002363f3a87f145fe96058c80fdc67222d5a533cfeb7b`.

### 18.7 Mega Man Legends (2026-10-12)

The tester's copy of Mega Man Legends stopped on the open kernel at a black screen; three defects of the kernel were
each shown by a failing test and fixed: the game's pad library calls and stores at addresses it computes past
ChangeClearPAD's table entry, where the kernel now holds a stretch of fill that returns (`Diamond_OpenKernel_Phase2.md`
§27, which replaced §26.1's first form); LoadExec loaded the game's second program over the first with the first's exit
structure and vblank still live, and now masks the sources and restores the default exit first; and the kernel's
synchronous CD operations now mask IRQ2, so that a program's handler is never offered one of their responses (§26.2,
§26.3). The game plays its intro, starts a new game and is played into its first ruin. The crate gained the program
tests `loadexec_starts_its_program_free_of_the_callers_interrupt_handlers` and
`calls_and_stores_past_b_5bh_s_entry_come_back_and_leave_the_pads_read`, and the run tool `--game-range`. The image's
SHA-256 is now `c1a002e88aa4e8b1be0257ca486bdb8309cd63f229e1f5070e36e4c72559c785`.

## 20. Stage 11, the PlayStation to players (2026-10-11)

Recorded in `Diamond_Players.md`: routing by content, image sets and archives, the disc change under the core ABI's
minor 1.1 (`MEDIA` in `v1.rs`, the drive, `media_lid` and `media_insert` over stage 7's lid), **the state's version 9**
(version 8 and the disc's fingerprint at the end, a version 8 state loading unheld), the core's extensions now `.cue`,
`.bin` and `.iso` beside `.psexe`, `.psx` and `.chd`, its system no longer marked in development, the cheats, settings,
cards, pads, shelf and CRT row, the kit on nine of the tester's discs, and the gate. Diamond is offered to players
since this stage (§8 there). Diamond's crate tests: 146 and 56.

---

## 21. Stage 13, speed (2026-10-12)

The plan's §7 row 13, with the tester's go of 2026-10-11. Its record is `Diamond_Speed.md`.

### 21.1 What was built

- **The bench** (`examples/bench.rs`): frames timed on the emulation thread alone, every frame's picture, the sound and
  the state digested outside the timed span, on the open kernel with no firmware; `--plain`, `--verify N`, `--armed`.
- **Idle skipping, the plan's third lever, built first by the profile** (`src/idle.rs`; `Machine::advance`): a loop's
  turn proven over the I-cache's words with one induction, observed for four turns against the bus's timing, and its
  further whole periods passed up to the next event; the debugger's frame never skips. `programs/idle.s` holds the loop
  forms; `Idle::verify` checks a skip against the interpreter as it is made.
- **The decoded-instruction cache's first tier** (`src/icache.rs`, `src/bus.rs`; `blackmoon-r3000`'s
  `Bus::fetch_decoded`, a method with a default, `BlackMoon_R3000.md` §2): each I-cache word decoded as it enters,
  a hit fetched without decoding the address or the word; memory control's RAM window kept with its register; data
  accesses to RAM answered before the map is decoded; the debug unit's checks returning at once while it is off.
- **No state change**: `DMND` stays version 9; neither the skipper's tables nor the decoded words are state, and a
  state load decodes the cache's words again.

### 21.2 The gate (G6)

| Item | Result |
|---|---|
| §5.12's budget at the 33 per cent quota on the bench set | **Met**: 7.1 to 11.2 ms against 13.4; before, Ape Escape, Castlevania and Spyro the Dragon missed it at 15.1 to 17.0 |
| The desktop's figure of about 3.8 ms | **Met**: 2.3 to 3.7 ms, from 4.1 to 5.5 |
| Full speed on the Legion Go S on battery | **Met** on the tree before D-29's and D-45's merges (1.2 to 4.5 ms on battery) and on the current tree on the charger (2.6 to 4.1 ms); the current tree's battery run is owed |
| Each lever bit-identical to the plain path | **Met**: the seven games' digests with and without, each sixteenth skip verified; the kit's and the debugger's armed-against-plain tests on the plain run with the levers |

### 21.3 Tests run (2026-10-12)

`diamond` 155 unit tests and 68 program tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, and the disc tests with
`EMUSEN_DIAMOND_DISC`; `blackmoon-r3000` with `EMUSEN_BLACKMOON_CORPUS`, 55,000 of 55,000; WiseMan under the test lock
with the corpus, `DiamondTests` and `DiamondDebugTests`.

### 21.4 What waits

The current tree's battery run on the handheld; the threaded rasteriser, the decoded cache's second tier and the
recompiler, not needed for G6 (`Diamond_Speed.md` §7); the inexact speed settings and the shipped link, proposed for
the tester's decision (§8 there). The handheld's emulation thread is at about a fifth of a frame, so the slowness the
tester reported there is to be looked for outside it. No dispute was taken.

## 22. Speed, the second pass (2026-10-12)

With the tester's request that the work return to speed after G6. Its record is `Diamond_Speed.md` §11 to §18.

### 22.1 What was built

- **Runs** (`blackmoon-r3000`'s `R3000::run` and `Bus::running`, a method with a default, `BlackMoon_R3000.md` §2 and
  §13; `Machine::steps`, `Machine::runs`): the plain frame's steps made in runs between events, the boundary's work
  held while SR, Cause, DCIC and the line cannot change; the common instructions on a path of their own inside the
  step, shared by `step` and `run`. Diamond's bus runs while its clock is short of the next event; the kernel's seams
  step one at a time. `Machine::verified_run` checks a run against its steps on a copy, byte for byte.
- **The rasteriser by rows** (`Painter::row`): the same pixels in the same order, the paint's settings read once a
  row, VRAM read only where a pixel is blended or the mask is checked, a texture cache miss filled out of line.
- **The bench** gains `--stepped` (runs off, the skipper on) and `--verify-runs N`, and prints the runs' counts.
- **WiseMan** gains an opt-in test, `The_frame_loop_reports_its_pace_on_a_named_disc`
  (`EMUSEN_DIAMOND_PACE_DISC`), which opens a disc from Mistress's library headless and prints the frame loop's line
  each second, for the frontend's share (§16 there).
- **No state change**: `DMND` stays version 9.

### 22.2 Results

On WiseMan 414cac36's tree the desktop's aligned medians fell by 23 to 32 per cent (Castlevania 4.31 to 2.93 ms, the
slowest game before and after); under the 33 per cent quota every game is at 5.8 to 9.9 ms against 13.4, Castlevania
having been over it before (14.2). Every game's digests every 600 frames equal the base's.

### 22.3 Tests run (2026-10-12)

`blackmoon-r3000` 37 unit tests with `EMUSEN_BLACKMOON_CORPUS`, 55,000 of 55,000, and without it; `diamond` 156 unit
tests and 76 program tests, with and without `EMUSEN_DIAMOND_PS1TESTS`, and the disc tests with `EMUSEN_DIAMOND_DISC`
on Crash Bandicoot; WiseMan's Diamond tests under the test lock.

### 22.4 What waits

The handheld's runs of the levers, on the charger and on battery, the handheld having left the network after the
base's battery run (`Diamond_Speed.md` §14.3), and the removal of the bench's copies from it; the threaded
rasteriser, a block form of the step and the recompiler (§15 there); the inexact settings and the shipped link (§8
there); and the tester's answer on the speed row (§16 there). No dispute was taken.

## 23. The picture in the state (2026-10-12)

A defect found by the kit and fixed between stages; no stage of the plan's §7. The state's earlier record is §2.7,
§11.16, §13.6 and the devices' own pages.

### 23.1 The defect

The kit's C7 and C8 at 1,800 frames, first run at that length for C17 (`Diamond_Players.md` §12), failed on Ape
Escape, Rayman, Silent Hill and Tekken 3 and passed on Crash Bandicoot, Spyro the Dragon, Castlevania and Mega Man
Legends [M]: a machine that loaded the state saved at frame 900 did not continue as the machine that saved it. The
digest of the pictures differed; the digest of the sound, the count of samples and the state after the run were equal.
It is no regression: WiseMan `6141ad60`, from before the disc work, gives Ape Escape's two digests to the bit [M, that
day's record], and both halves of the cause (§23.2), the weaving and the state without the picture, entered with stage
4's commit `54b8ca2c`, so there was nothing to bisect.

### 23.2 The cause

Two machines were run through the ABI under the kit's own input, the second made new and given the first's state at
frame 900, and their pictures compared frame by frame [M]:

| Disc | The picture at frame 900 | The first frame that differs | What differs in it | Frames differing of 900 |
|---|---|---|---|---|
| Ape Escape | 640 by 480 | 901 | 110 odd lines, 129 to 357 | 1 |
| Rayman | 640 by 478 | 901 | 81 even lines, 154 to 342 | 1 |
| Silent Hill | 640 by 448 | 901 | 224 odd lines, 1 to 447 | 1 |
| Tekken 3 | 368 by 480 | 901 | 224 even lines, 20 to 466 | 1 |

In each the one frame after the load differs, in lines of one parity alone, and the states are equal from that frame
on. The four are in 480-line interlace at frame 900; the four that pass are not.

**The mechanism.** `display.rs` reads a field's lines from VRAM as the video timing reaches them. In 480-line
interlace it writes them to alternate lines of the picture, and the lines between, the other field's, stay as they were
read a field earlier (`Diamond_GPU.md` §7). The state held the display's registers and its place in the field and not
the picture, by a choice stage 4 recorded: "the picture is not kept, and a state loaded between frames re-reads it
within the next frame" (`Diamond_GPU.md` §12, [C], now struck there). That holds for a 240-line picture whose range
stands still; it does not hold for a woven one, half of which is a field old at every frame's end. A machine that
loaded a state therefore wove the next field into whatever picture it had: black, on a machine just made.

**The ablation.** With the saving machine's picture copied into the loading machine after the load, and nothing else
changed, no frame of the 900 differs on Ape Escape, Rayman or Silent Hill, run without input [M]; Tekken 3 shows a
480-line picture only under the kit's input, where the probe goes through the ABI and cannot reach the picture, and
was confirmed by the fix instead. The picture is thus sufficient for the whole of the difference. The other suspects
named when the defect was reported were not ablated one by one, since nothing was left for them to explain: the
texture and CLUT caches, the FIFO and the command in flight are in the state (`Diamond_GPU.md` §12); the decoded
words are made again at a load and the idle skipper's tables at every frame (§21.1); the interlace field is the video
timing's `Odd`. The scan of §23.4 looked for a second cause and found none.

**Why it is state and not derived.** The held lines are VRAM as it stood a field ago, and nothing keeps VRAM so: a
640 by 480 picture leaves no room in VRAM for a second buffer, and a game showing one draws into the lines displayed.
`a_state_holds_the_picture_whose_other_field_vram_no_longer_has` asserts as much of its program: the picture read
whole from VRAM at the load is not the picture saved. What cannot be made again from the rest of the state is state.

**A second symptom, found by the scan.** A load did not change the picture until a frame had run: a machine paused at
a load went on showing what it had shown before. The kit has no case for it. On the base it is seen at 82 to 108 of
a disc's 119 save points, on all eight discs (§23.4); the fix ends it, the picture being loaded with the rest.

### 23.3 The fix

`DMND` **version 10** [C]: the state ends with `Picture`, the display's picture whole, 640 by 576
RGBA, **1,474,560 bytes** after `Disc`. The layout before it is version 9's to the byte.
`the_version_10_layout_is_pinned`: **5,267,532 bytes** for a machine with one DualShock and no card (3,792,972 at
version 9); with the kit's two DualShocks and a card, 5,398,770 (3,924,210).

- **A state of version 8 or 9 still loads**, and has no picture: the display is read whole from VRAM as the state has
  it, both fields of a woven one (`Display::reread`). That is not the picture saved, which nothing can give, but it
  is the nearest the state allows and the same on every machine, where before it was the loading machine's leftover.
- **The whole picture, not the held field** [C]. The held field alone would be half the bytes at most and exact at a
  frame's end in a standing mode. The whole is exact wherever a state is taken: at a debugger's stop mid-field, where
  part of the field being read is new and part old; after a range or a mode written mid-field, which leaves lines the
  field does not read again; under a range that shows no line. It is also one copy with no case analysis, and it makes
  a load show the picture saved.
- **RGBA as the display holds it, not packed to three bytes** [C]: packing saves 368,640 bytes of a constant, which
  costs a delta nothing (below), for a loop and a scratch buffer at every save.
- **Not a wider reading of VRAM at the load** (the first form considered, since derived data is better rebuilt than
  carried): it is exact only where VRAM has not been drawn over, and the test above shows a program for which it is
  not.

**The cost** [M, a probe in the cache, the desktop, medians of 100]: the state is saved in 0.19 to 0.27 ms where it
took 0.13 to 0.20, and loaded in 0.33 to 0.44 ms where it took 0.30 to 0.38, on four discs: one copy of the picture
more each way. The rewind stores what differs between one state and the next it takes, four frames on
(`RewindBuffer`), and that grows by the picture's changed bytes. From frame 1,500 without input, 100 pairs: Tekken 3 265,795 bytes a pair before and 397,270
after, Spyro the Dragon 409,817 and 634,091, about half as much again; on a screen that stands still next to nothing
(Crash Bandicoot 12,264 and 16,435; Ape Escape's 640 by 480 screen from frame 700, 10,066 and 10,834). If the coder's
output follows the differing bytes, a moving game is kept about two thirds as long under the same budget [A]. That is
what an exact state costs here, and it is the frontend's budget to weigh (§23.7).

The frame's cost is unchanged, as nothing on the frame's path changed but a parameter passed to the line reader
[M, the bench of `Diamond_Speed.md` §2 on builds with every function aligned to 64 bytes, the base and the branch
alternating, the fastest mean of five, ms a frame]:

| Run | Frames | Base | Branch | Difference |
|---|---|---|---|---|
| `gpu/benchmark` | 1,200 | 2.313 | 2.306 | −0.3% |
| `gpu/bandwidth` | 1,200 | 2.972 | 2.970 | −0.1% |
| Crash Bandicoot | 3,000 | 2.300 | 2.309 | +0.4% |
| Tekken 3 | 3,000 | 2.386 | 2.395 | +0.4% |

One build's five runs spread by 1 to 29 per cent, other work sharing the machine, which is why the fastest is taken;
the differences, 0.4 per cent at most and of either sign, are inside it.

Each pair's picture and sound digests are equal, so an unbroken run is what it was; the state digests differ, as they
must.

### 23.4 What was measured after it

**The kit**, single-image form, each disc at three lengths on the base and three on the branch, the save at each
length's middle [M]:

| Disc | Base, 600 (save at 300) | Base, 1,400 (700) | Base, 1,800 (900) | Branch, 1,400 | Branch, 1,800 | Branch, 3,000 (1,500) |
|---|---|---|---|---|---|---|
| Ape Escape | pass | **C7, C8 fail** | **C7, C8 fail** | pass | pass | pass |
| Rayman | pass | **C7, C8 fail** | **C7, C8 fail** | pass | pass | pass |
| Silent Hill | pass | **C7, C8 fail** | **C7, C8 fail** | pass | pass | pass |
| Tekken 3 | pass | **C7, C8 fail** | **C7, C8 fail** | pass | pass | pass |
| Castlevania: Symphony of the Night | pass | pass | pass | pass | pass | pass |
| Crash Bandicoot | pass | pass | pass | pass | pass | pass |
| Mega Man Legends | pass | pass | pass | pass | pass | pass |
| Spyro the Dragon | pass | pass | pass | pass | pass | pass |

Where C7 and C8 fail so does C12, which runs them again under a host of another minor; every other case passes in
all 48 runs, and on the branch all seventeen in every run, the state 5,398,770 bytes. Ape Escape's C6 digests at 1,800
frames are the base's to the bit (`A02BFAE09D41CF89` for the pictures), so the kit sees an unbroken run unchanged.
**The corpus form** on the sample's nine discs at 1,800 frames, `kit/psx-sample.txt`'s command: 9 of 9 and the core's
cases pass, 190 s, nine at once, 8.98 GB peak resident under a 10 GiB limit [M].

The length of 1,400 was chosen from the scan: frame 700 is inside the 480-line stretch of all four discs that failed.
At 600 frames, the save at 300, the base passes too, which is §23.5's matter.

**The scan** [M]: one machine through the ABI under the kit's input to frame 3,000; at every 25th frame its state
loaded by a machine just made, the picture at the load compared with the picture saved, then three frames and the
state compared. 119 save points a disc, 952 in all:

| Disc | Save points in a picture of more than 300 lines | Base: continuation fails | Base: the picture at the load is not the one saved | Branch: either |
|---|---|---|---|---|
| Ape Escape | 27, from frame 575 | 24 | 84 | 0 |
| Rayman | 28, from 500 | 23 | 91 | 0 |
| Silent Hill | 15, from 575 | 14 | 108 | 0 |
| Tekken 3 | 100, from 425 | 98 | 100 | 0 |
| Castlevania: Symphony of the Night | 0 | 0 | 82 | 0 |
| Crash Bandicoot | 0 | 0 | 88 | 0 |
| Mega Man Legends | 0 | 0 | 108 | 0 |
| Spyro the Dragon | 0 | 0 | 108 | 0 |

On the base the continuation fails at 159 of the 170 save points in a picture of more than 300 lines and at none of
the other 782; in each of the eleven that continue, one field's lines of the picture saved are all black, which is
what a machine just made holds. On the branch nothing fails at any of the 952.

### 23.5 Why the gate did not see it

- **Stage 11's G4** ran the kit's corpus form on nine discs at the kit's default 600 frames (`Diamond_Players.md` §6).
  C7 and C8 save once, at the middle: frame 300. Under the kit's input the first save point of the scan in a picture
  of more than 300 lines is frame 575 on Ape Escape and Silent Hill, 500 on Rayman and 425 on Tekken 3 (§23.4); at 300
  none is. The base still passes C7 and C8 on all eight at 600 frames [M, §23.4]. The gate's one save point fell
  before the first frame that could fail.
- **WiseMan's routine case**, `The_kit_passes_on_a_synthetic_program_and_a_synthetic_disc`, runs a program that
  jumps to itself, alone and booted from a synthetic disc, at 120 frames. The program never writes to the GPU, and
  the case has passed at every merge, so its save at frame 60 met no 480-line picture with anything in it.
- **Stage 14's goldens** (the plan's G5) are not built: no page records them and the tree holds none [R, the pages
  and WiseMan searched 2026-10-12]. A golden would not have seen this in any case: it compares one unbroken run with
  another, and an unbroken run was never wrong.
- **The crate's own state test** round-trips a machine in an idle loop and compares the clock and the program counter,
  not the picture.

What now sees it:

- `a_state_holds_the_picture_whose_other_field_vram_no_longer_has` (`src/state.rs`): a program of nineteen words
  shows 640 by 480 interlaced and fills it in a colour that changes about every field; a state is saved after five
  frames, and a machine just made must show the picture saved and weave the next field into it. On the code before,
  all 480 lines differ at the load and the held field's 240 after the next frame [M].
- **WiseMan's routine kit case runs the same program as a third image**, `woven.psexe`: on the base C7, C8 and C12
  fail on it at 120 frames, the save at frame 60 [M]. It adds one kit run to the case.
- `a_state_loads_as_it_was_saved_all_along_a_disc` (`tests/programs.rs`, opt-in by `EMUSEN_DIAMOND_DISC` as the
  three disc tests beside it, which now share `named_disc`): thirty save points in a disc's first 1,500 frames, each
  loaded by a machine just made, the picture at the load, three frames and the state compared. On the base it fails
  on Ape Escape at frame 600 [M].
- `kit/psx-sample.txt`'s command is now 1,800 frames, with the reason beside it.

### 23.6 Tests run (2026-10-12)

`diamond`, `cargo test --release`: 159 unit tests and 78 program tests, with `EMUSEN_DIAMOND_PS1TESTS` and without
(158 and 77 before; one added to each), and the disc tests with `EMUSEN_DIAMOND_DISC` on Crash Bandicoot, the new one
on Ape Escape too, where on the base it fails. WiseMan under the test lock with the corpus,
`FullyQualifiedName~Diamond`: 46 pass, the kit case with its third image among them. The kit: the 48 single-image runs
of §23.4 and the corpus form at 1,800 frames. `git merge WiseMan` brought nothing, WiseMan standing at `6b4ae644`
where the work began. No test was run on the handheld, which this work does not touch, and none on a player's BIOS:
every run above is on the open kernel.

### 23.7 What waits

- **The kit saves once a run.** A C7 that saved at several points, or a rule that the picture after a load is the
  picture saved, would have caught this at any length and on any core; either changes C7 for every core
  (`EmuSen_CoreAPI.md` §12.1) and is proposed to the tester, not built [C]. The scan's probe, which does both, stayed
  in the cache.
- **The rewind's budget.** Its deltas grow by the picture's changed bytes (§23.3). The form that would avoid it, the
  picture kept only while it is woven and read from VRAM at a load otherwise, is exact at a frame's end in a standing
  mode and not elsewhere (§23.3's second item); it is not built, accuracy not being traded by default, and is the
  tester's to ask for.
- No dispute was taken: no rule of the hardware is at stake, the display weaving as it did.

---

## 24. Stage E1, the enhancement layer's frame (2026-10-13)

The record is `Diamond_Enhancements.md`. In brief:

- **The goldens**, recorded on WiseMan 177f5041 before any of E1's code (`tests/goldens/`): the 49 programs of
  `programs/`, ps1-tests' 48 and the eight bench discs played by their press scripts to frame 3,000 with 80 anchors;
  the rule for recording them again is its §6.2.
- **The observer**: the R3000A's moves told to its bus (`BlackMoon_R3000.md` §14), the GTE's projections with the
  camera-space sums and the quotient (`BlackMoon_GTE.md` §9), the GPU's receipts with their RAM addresses
  (`Diamond_GPU.md` §13); a layer attached to the board, the frame stepping through `Layered` only while one is.
- **The state**: optional blocks after `Picture`, each a tag, a length and its bytes (§4 there); with no layer the state
  is version 10's to the byte, 5,267,532 bytes, its layout test unchanged; no version was taken.
- **The settings**: `Enhancement`, a switch or a choice in the "Enhancements" group with the hardware's value its
  default; none offered (§5 there).
- **The harness**: every golden equal with the layer compiled in and off and again with the test observer attached;
  the kit's cases on the eight discs at 1,800 frames; the frame's cost unobserved within the noise, after two causes of
  1 to 2.5 per cent were found and removed (§9 there).

## 25. Stage E3, the Vulkan renderer at 1× (2026-10-13)

The record is `Diamond_Enhancements.md` §14 to §20. In brief:

- **The form**: a compute rasteriser over the renderer's own mirror of VRAM on `emusen-gpu`, argued against a graphics
  pipeline and committed before the code (§14); the crate gained `dispatch_again`, version 1.1 (`EmuSen_Gpu.md` §3.1).
- **The seam**: commands, their continuations and the display's lines told to an attached layer by the GPU, the layer
  now held by the GPU, the GPU's decoding shared from `gpu/decode.rs` (§15).
- **The renderer**: the crate `diamond-renderer`, a layer, records binned into tiles and planned as stretches ordered by
  what they read and write, four shaders, seeded from the GPU and keeping no state block (§16).
- **The gate**: every picture of the 49 programs, ps1-tests' 48 and the eight discs to frame 3,000 equal to the software
  rasteriser's at every frame, VRAM equal but for the texture cache's halfwords on its two programs, E1's goldens equal
  with the renderer attached (§17).
- **The cost**: unattached the base's within noise; attached +0.29 to +0.57 ms a frame on the RX 6800 (§18).

## 26. Stage E4, precise geometry (2026-10-13)

The record is `Diamond_Enhancements.md` §21 to §31. In brief:

- **The design**, committed before the code (§21 to §24): the precise vertex the GTE's projection without its
  truncations, guarded within two pixels of the console's; serials in a shadow of the FIFO, the registers, RAM and the
  scratchpad over a ring of 32,768 projections, the integer checked once at the receipt; whole polygons only.
- **The processor and the GTE**: a store told before it writes (`BlackMoon_R3000.md` §14); `Projection`'s `h` and
  `offset` (`BlackMoon_GTE.md` §9).
- **The crates**: `precision`; the renderer's `PRECISE` record and its block; `diamond-v1`, which holds the exports, the
  machine's crate an rlib whose `impl Core for Machine` it passes on; `RustCores.props`' `Dir` and CI's build at `v1/`.
- **The state**: a layer's blocks per tag; `PREC` 2.6 MiB and `RNDR` 2.5 MiB with the setting on, none with it off,
  version 10's layout unchanged.
- **The gate** (§25): every golden equal off and on; E3's equality through the precise record at zero fraction on the
  discs; the grid's jitter and warp; the 2D program and the strips; the state both ways; P10; the kit off and on.
- **The crack fill**, the same day (§32 to §36): the renderer's precise record carries each vertex's fraction and the
  walk fills a precise triangle to the console's own; a mark a pixel in the mirror's spare half; `RNDR` 3.5 MiB with
  the marks; no crate, export or setting added.

