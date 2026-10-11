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
- **The frame rate** [A, provisional until D-0]: psx-spx's video clock for each standard over a progressive frame,
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

`firmware_for` names the image's market's BIOS (the Americas' for a program without a marker) and nothing for an image
it does not recognise; machine info names the machine's market's file. A player's file, when given, is held in the
read-only `BIOS` space and is not run. The names are the ones players' own dumps commonly carry (the plan's §5.9).
**The replacement is named but marked not yet available** [C]: `effect: none` is v1's word for "the game runs without
the chip", and no word says "a replacement that does not exist yet"; `none` with a cost whose first sentence says no
program runs is the honest reading, as Nephrite's stage 0 declared its Sega CD BIOS. It becomes `accuracy` with the
plan's §5.9 cost when the kernel runs (stage 3).

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

All zero-filled. `R3000BUS`, `CDBUFFER` and the cards of the plan's §4.3 come with the bus (stage 3), the CD-ROM (7)
and the cards (6).

Version 1 of the state: the magic `DMND`, the version, the frame count, the R3000A's block as `Cpu` (192 bytes) and
the GTE's as `Gte` (256), then every writable memory in id order: **3,671,504 bytes** [M, the kit], the plan's
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
- **Not decided yet** [C, open]: whether the I-cache and the scratchpad live in the processor's crate or in Diamond's
  bus, settled at stage 3 (`BlackMoon_R3000.md` §2).

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
