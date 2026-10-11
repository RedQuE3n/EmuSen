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
- **The picture**: 320 wide, 240 lines NTSC and 256 PAL, black, the size fixed for a machine's life [C: the GPU's
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
has a Diamond state, so version 1 is redefined freely until the core is offered.

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
  (`io-access-bitwidth`'s DMA rows). On the CD-ROM's 8-bit bus without auto-increment every byte of a wider access
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

- **A field** is 263 lines (314 PAL), interlaced 263 and 262 alternately (314 and 313) [C: the average is the 262.5
  psx-spx gives; how a console splits the half line is D-0's open part].
- **Vblank** is the field lines outside GP1(07h)'s range, beginning at Y2's line; IRQ0 at its start; none when the
  range leaves no edge (psx-spx: a Y2 too large stops the interrupt). **Hblank** is the part of each line outside
  GP1(06h)'s X1..X2, beginning at X2; none with a range of no width (psx-spx's "endless hblank").
- **The dot clock** divides the video clock by 10, 8, 7, 5 or 4 by the mode; a line holds a whole number of dots, the
  fraction dropped (psx-spx), 426 at 320 pixels.
- **GPUSTAT's bit 31** is the line's parity in 240-line mode and the field's interlaced, zero in vblank; bit 13 the
  field, 1 when not interlaced.
- A change to the mode or the ranges takes effect from the current line, the counts so far kept [C].
- **The frame** is one field, from its first line to the next field's; machine info's rate the console's own standard
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
- **Not graded**: the sync-mode tables' values depend on where the program begins against hblank, so they are
  compared by pattern, not number (§11.12).

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
- **The gaps**, fitted to the console's logs (D-5): 10 cycles between blocks and between list nodes, 4 of a node's own,
  and a chopped window's CPU window held to 32 cycles a word of the DMA's window plus 6. On `dma/chopping`'s 130 rows,
  **129 within 10% of the console and the worst at 10.5%** (one-word windows with a one-cycle CPU window, 18,448
  against 16,693); `chain-looping`'s self-referencing chains take 24,032 cycles of work against the console's 25,632
  and 25,640 (−6.2%).

### 11.8 The stubs, and what each answers

| Device | What a stage 3 program reads | Source |
|---|---|---|
| GPU (stage 4) | GPUSTAT `14802000h` after GP1(00h), always ready for a command and a DMA block (bits 26, 28), bit 25 by GP1(04h), bits 31 and 13 from the video timing; GP0's commands counted out by their lengths and dropped, but E1h–E6h into GPUSTAT and GP1(10h)'s answers, and GP0(1Fh)'s IRQ1; GP1's display commands into the video timing; GP1(10h) index 7 the version, 2 for the 208-pin chip | psx-spx's GP0/GP1 and GPUSTAT pages |
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
| `timers` | each delay and frame row's settled values (the first two dropped) against the log's mean: a frame's hblanks exactly, its dots ±10, its cycles ±60 of the 16-bit count, a delay ±3 cycles or ±2 slower ticks; run as a PAL console (`region=psx.eu`), the log's console being PAL-clocked (D-0); the sync-mode tables not graded |

A run with no `tty` space, the references' through the probe among them, is `NoDump` (§11.15).

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
| `cop0-garbage.s`, MFC0 of 16–31 | 0 | values with a period of eight registers (10800h, 0, 304h, C1h, 304h, 1F800000h, 8, 4), and 10800h after a delay or after EPC | 0 | 0 |
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
| `timers` (as a PAL console) | pass, 18 of 18 rows |
| `cpu/access-time` | pass, 17 of 17 rows within 0.5 cycles, since the kernel's printf takes a flag after the width (`Diamond_OpenKernel.md` §4.10, 2026-10-11); before that its `%2-d` printed `-d` and every cell's hundredths were lost |
| `cpu/io-access-bitwidth` | not graded: a table to compare by eye; it differs from the console's log in the registers' upper "garbage" bits (16–31 of I_MASK, the timers' and SIO's), which Diamond reads as zero, and in DEV1, which reads FFh where the console's DTL-era window holds what was written |

**A player's BIOS** (`Ps1_tests_pass_by_their_tty_on_a_players_bios`, the American BIOS for NTSC and the European one
for `timers`): all seven pass, `access-time` with 50 of 51 cells within 0.20 cycles (§11.3).

**Bare-metal programs**: Peter Lemon's 72 programs, 120 frames each, Diamond's RAM against PCSX-ReARMed's from 10000h
up [M]: **64 identical but for the 16 bytes above the initial stack, which PCSX-ReARMed's own kernel writes**; GTEAVSZ,
GTENCLIP and Cube differ in the GTE's results (stage 5); DCTBlockDecodeCLUT4BPP and 8BPP in the MDEC's (stage 9);
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
- on a console (Q16): D-0's line, D-2, D-4's SPUCNT cell, D-5's gaps and the store's drain, D-6's mode 2;
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
