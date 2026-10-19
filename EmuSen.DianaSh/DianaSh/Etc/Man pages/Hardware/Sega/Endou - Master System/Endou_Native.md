# Endou_Native — the build record

*Opened 2026-10-19, at stage 1.* What each stage of `Endou_Plan.md` §7 built, how it was graded, and what was measured.
Marks as in the plan: **[M]** measured on the date given, with the tool named; **[R]** read from the source named;
**[D]** stated as the documents describe it, the document cited; **[A]** argued; **[C]** a choice.

---

## 1. Stage 1, in one paragraph (2026-10-19)

The crate `endou` (`EmuSen/Cores/Sega/Endou - Master System/`) is a stub on the core ABI v1. It reads a cartridge's
header, chooses the console the image is for, reports the systems `sms` and `sg1000`, each marked in development,
with their pads, spaces, frame rates and a region setting, keeps its own state format `ENDO`, and shows a black
picture with silence. It holds Beryl's Z80 and SN76489 through their public surfaces, at reset, so that their state
blocks are in the format from the first version; nothing runs. `RustCores.props` names it `InDevelopment`, so the
build writes its sidecar marked and no player is offered it. The systems table gains `sms.json` and `sg1000.json`, the
C# gains their literal packs, WiseMan gains `SyntheticSmsRom` and `EndouTests`, CI builds, tests and export-checks the
library, and the reference probe runs the Master System's cartridges on five libretro cores, two of them newly pinned.
The documents, test programs and referees the plan named were fetched and pinned (`Endou_References.md`).

## 2. The stub

### 2.1 What it answers

| Question | Answer | Source |
|---|---|---|
| Systems | `sms` (`.sms`), `sg1000` (`.sg`), both `development: true` | `Endou_Plan.md` §4.1 |
| Controllers | `sms.pad`: Up, Down, Left, Right (bits 0–3, port `$DC`'s order), 1 → B, 2 → A, Pause → Start, Reset → Select (bits 4–7); `sg1000.pad` the same without Reset; both on ports 0 and 1 | §4.2 [C] |
| Spaces | `sms`: RAM 8 KiB (cheats), VRAM 16 KiB, CRAM 32 bytes, ROM read-only; `sg1000`: RAM 1 KiB, VRAM 16 KiB, ROM | §4.3; the SG-1000's 1 KiB [D `sc3000h-20040729.txt`], as Genesis Plus GX also exposes it [M §5.2] |
| Processors | `Z80`, 16-bit PC, no registers listed until the debugger's stage | below |
| Video | 256 × 192, at most 256 × 240, a 4:3 television, RGBA8888, opaque black | §1.2 |
| Audio | 48,000 Hz silence, each frame's count exact to the rational rate | as Nephrite's output rate [A] |
| Frame rate | NTSC 53,693,175 / (15 × 228 × 262) = 59.922743 Hz; PAL 53,203,424 / (15 × 228 × 313) = 49.701459 Hz | §5.1 [D]; §5.2 for a witness |
| Region | `auto` (the header's), Japan, the Americas, Europe (PAL), Brazil; a header naming no market, or none, runs as Japan | §5.8, §10 Q9 |
| Firmware | none declared | §2.5 |
| State | `ENDO` version 1: magic, version, console byte, frames, the Z80's block, the PSG's, then RAM, VRAM and CRAM; 24,687 bytes for `sms`, 17,487 for `sg1000`, pinned by `the_version_1_layout_is_pinned`. *Replaced by version 2 at stage 2 (§9.7)* | §5.7 |
| Refused | an empty image, and one larger than 4 MiB and a copier's header, with words (`-9`, or the ABI's `BAD_IMAGE` for no bytes); any other file number than 0 | below |

**Why no registers.** Nephrite lists the Z80's eighteen registers in a table of its own
(`genesis/debugger.rs`'s `Z80_REGISTERS`). A second user means the list belongs in `beryl-z80`, one owner for both, and
the move is stage 8's, when Endou's debugger needs it; a copy now would be the duplicate the project's rules forbid.

**The PSG by console.** `Variant::SEGA` for `sms`, `Variant::DISCRETE` for `sg1000`, D-0's conclusion
(`Endou_Disputes.md`), held by `each_console_has_its_memories_and_its_psg`.

### 2.2 The header and its checksum

Sega's header is looked for at `$7FF0`, `$3FF0` and `$1FF0`, in that order [R SMS Power!'s "ROM Header"]. Its fields
are read as that page gives them: the checksum a little-endian word at +10, the product code's four BCD digits and its
high nibble, the version, the region code (`$3` SMS Japan, `$4` SMS export, `$5`–`$7` the Game Gear's) and the size
code. **The checksum computed** is the one Sega's own manual states: "a 16-bit checksum for the ROM (low-hi order).
`$7FFA` and `$7FFB` are not included in this checksum" [R *Software Reference Manual for the SEGA Mark III Console*,
"The cartridge header"], over the range the size code names (8 KiB to 1 MiB, SMS Power!'s table), cut at the image's
end. SMS Power!'s page warns that the export BIOSes' own checks have bugs for some codes; which range a BIOS really
sums is stage 6's question, since only the open boot program acts on the sum. At stage 1 the sum is only compared and
reported, never acted on.

**A copier's header.** An image whose length is 512 bytes more than a multiple of 16 KiB has its first 512 bytes set
aside ~~[A: the copier convention]~~ [R, M: confirmed against a document at stage 2, §9.2]. 11 of the tester's 316 `.sms` files have such lengths (33,280, 262,656 and 524,800
bytes), and every one of them has Sega's header 512 bytes later than the others [M below], which is the rule's
evidence so far; it is to be confirmed against a document at stage 2, where boards are mapped.

### 2.3 Which console an image is for, and an ABI question

**The finding.** A core is never told which system the host routed a file to: `emusen_create_params.system` is set
only with an image of no bytes, and "a core ignores it beside an image" [R `EmuSen_CoreAPI.md` §6.31]. Endou runs two
systems whose images share no header that tells them apart: an SG-1000 cartridge has no Sega header, and neither do
some Japanese Master System cartridges.

**The stage-1 rule** [C, provisional]: an image with Sega's header is `sms`; an image without one and no larger than
48 KiB, the largest SG-1000 board, is `sg1000`; anything larger is `sms`. **Measured** on the tester's library
(`mastersystem/`, read in place, 2026-10-19, a Python reader of each file's header and length, the copier's header
set aside as §2.2 does) [M]:

| | header, ≤ 48 KiB | header, larger | no header, ≤ 48 KiB | no header, larger | the rule right |
|---|---:|---:|---:|---:|---:|
| `.sms` (316) | 11 | 298 | 2 | 5 | 314 |
| `.sg` (54) | 0 | 0 | 53 | 1 | 53 |

The three it gets wrong: two copier BIOS images of 8 KiB filed as `.sms`, and a 512 KiB `.sg` (*Music Station*). The
rule is a guess at what the host already knew. **The proposed form** [C, for the tester before stage 4]: an extension of the existing surface, the core
ABI's next minor, letting a host set `system` beside an image as well, the field already at offset 104; a core that
reads it takes the routed system, and an older core ignores it as today. Stage 4, which makes the SG-1000 a modelled
console, is the first stage whose behaviour depends on it.

## 3. The systems and the packs

`Platform/systems/data/sms.json` and `sg1000.json`, appended to the table after `n64`, and the C# literals
`EmuSen.Crystal/Systems/Sms/SmsSystems.cs`, held equal field by field by `SystemTableParityTests`. Each value's source:

| Field | `sms` | `sg1000` | Source |
|---|---|---|---|
| `console` | Master System | SG-1000 | [C] |
| `release_year` | 1985 | 1983 | the plan's §4.1 [R Art Book Next] |
| `esde_system`, `esde_full_name` | `mastersystem`, Sega Master System | `sg-1000`, Sega SG-1000 | [R ES-DE's `es_systems.xml`, master `a529a9d`, 2026-10-10]; ES-DE also has `mark3`, "Sega Mark III", with the platform `mastersystem` |
| `screenscraper_system` | 2 | 109 | [R ScreenScraper's system pages `systemeinfos.php?plateforme=2` ("Master System") and `=109` ("SG-1000"); `=21` is the Game Gear's and `=1` the Mega Drive's, a control matching `md.json`] |
| `cheat_systems` | Sega - Master System - Mark III | Sega - SG-1000 | [M the plan's §4.1] |
| `openvgdb_systems` | `SMS` | `SG1000` | [M the plan's §4.1] |
| `cover_aspect` | 0.7 | 0.7 | **provisional** [C], the Genesis's value; the table's schema needs a number now, and stage 7 sets it from the boxes, as the plan's §4.1 says |
| `cheat_space`, `media` | none, none | none, none | the codecs and battery files are stage 7's |
| `byte_order` | little | little | the Z80's [D] |

Both packs carry no codec, view or player rule yet. Routing needs no recogniser: `.sms` and `.sg` are each one
system's, and `.bin` stays the Genesis's and the PlayStation's (`a_master_system_or_sg1000_image_routes_to_its_one_system`).

## 4. SyntheticSmsRom

`EmuSen.WiseMan/Fixtures/SyntheticSmsRom.cs` builds images and never holds game data:

- `Cartridge(region, size, product, version, rightChecksum)`: 32 KiB or more, Sega's header at `$7FF0` with the size
  code for its size and the checksum of §2.2 (or a wrong one on request);
- `Sg1000(size)`: no header;
- `WithCopierHeader(rom)`: 512 bytes before it;
- each with a **boot stub and a pad reader**, hand-assembled until stage 2 brings `beryl-z80`'s assembler: `DI`, `IM 1`,
  `LD SP,nn` (`$DFF0`, or `$C3F0` in the SG-1000's 1 KiB), then a loop of `IN A,($DC)`, `LD ($C000),A`, `IN A,($DD)`,
  `LD ($C001),A`. The stub does nothing at stage 1, since nothing runs; it is there so that stage 2's first test of the
  pads has its program already.

## 5. The gate (Endou_Plan.md §7, stage 1), measured 2026-10-19

### 5.1 Results

| Gate item | Result [M] |
|---|---|
| The crate's tests (`cargo test`) | **16 of 16** (media 4, machine 3, state 2, v1 7) |
| `export_check.py` against the v1 baseline | passes: ABI 1.14, capabilities 0, 32 `emusen_core_` symbols |
| The kit, single-image form, 120 frames, on four synthetic images (an export cartridge, a 256 KiB Japanese one, a copier's image, an SG-1000 cartridge) | **every case it runs passes on each**: C1–C15, C17–C23; C16 is `MEDIA`'s, which Endou does not claim, so the kit does not run it. Also passes on FluBBa's *SMS VDP Test* from the cache |
| The systems crate (`emusen-systems`) | **28 of 28**, two new; the other Platform crates that read the table (cheats, crystal, crystal-debug, library, platform, savemedia, scrape) pass unchanged |
| WiseMan, the blast radius: `EndouTests` (7), `SystemTableParity`, `SystemCodeParity`, `CheatCodecParity`, `CoreDiscovery`, `CoreRegistration`, `CoreHostParity` with `EMUSEN_PUBLISHED_TREE` naming the publish below, `FirmwareOverview`, `SystemPack`, `CoreAdapter` | **94 of 94**; `CrystalParityTests` **31 of 31**; the registration golden unchanged, as no player's discovery sees the core |
| A published tree's `sha256sum -c cores.manifest` (`dotnet publish` of Mistress, linux-x64, Release, self-contained) | **failed before the fix of §5.3, passes after it**: six libraries `OK` |
| The references' costs | §5.2 |

Toolchains: rustc 1.99.0, .NET 10.0.112, on the desktop (Ryzen 7 7700X).

### 5.2 The references through the probe

Five libretro cores run the Master System's and the SG-1000's cartridges with no BIOS; Gearsystem and SMS Plus GX were
fetched and pinned this stage (`EmuSen.WiseMan/Reference/libretro-cores.sha256`). The probe gained one line: `.sg`
names the system `sg1000`, which only names its spaces' system in the log (`system_comes_from_the_rom_extension`; the
probe's 50 tests pass).

**The cost** a frame, measured 2026-10-19 on the desktop as the difference of wall times between runs of 600 and
3,600 frames over 3,000, three runs of each, the median and its range, every option at its declared default [M]:

| Core | Version | Options | *SMS VDP Test* | *SMS Test Suite* 0.36 | *SN76489 TestRom* (SG-1000, NTSC) | Timing it reports | Spaces through the standard calls |
|---|---|---:|---:|---:|---:|---|---|
| Genesis Plus GX | v1.7.4 `58c3414` | 62 | 0.137 ms | 0.082 | 0.085 | 59.922743 Hz, 44.1 kHz, 256 × 192 | `ram` 8 KiB (1 KiB on the SG-1000), `sram` 64 KiB |
| PicoDrive | 2.05 `1890c29` | 21 | 0.029 (0.029–0.030) | 0.050 | 0.048 | 60.000000 Hz, 44.1 kHz, 320 × 240 | `ram` 8 KiB, `sram` 32 KiB, `vram` 64 KiB |
| BlastEm | 1.0.1-pre | 21 | 0.228 | 0.233 | 0.380 (0.379–0.386) | 59.922743 Hz, 53,267 Hz, 320 × 224 | `ram` 8 KiB; none on the SG-1000 |
| Gearsystem | 3.9.22-20-g190873a | 28 | 0.278 | 0.197 | 0.144 | 59.922743 Hz, 44.1 kHz, 256 × 192 | `ram` 8 KiB |
| SMS Plus GX | 1.8 `3844b46` | 6 | 0.083 | 0.071 | 0.080 | 60.000000 Hz, 44.1 kHz, 256 × 192 | `ram` 8 KiB, `sram` 32 KiB, and 64 map spaces of 1 KiB covering the Z80's 64 KiB |

Ranges not shown are within 0.001 ms. Three cores report 59.922743 Hz, which is §2.1's NTSC rate to the digits shown,
a witness to it [M]; two report a round 60. These are black boxes' wall times, not a budget for Endou [A].

**Agreement, a first look** [M]: on *SMS VDP Test* at frame 600 the five cores' 8 KiB of RAM differ pairwise in 3 to
10 bytes. On sverx's *SMS Test Suite* Genesis Plus GX and SMS Plus GX agree exactly and differ from the other three in
about 4,300 bytes, which agree among themselves to within 54–313; on the SG-1000 program PicoDrive's 8 KiB differ from
the 1 KiB the others hold, consistent with the 1 KiB mirrored [A]. None of this is a dispute yet: the stages that run
these programs compare their pictures and RAM at anchors, and a disagreement that matters becomes an entry. No core
exposes VRAM through the standard calls but PicoDrive (64 KiB, the Genesis's size).

**The probe's options for the Master System** (plan §3.6): pinning is already `--option KEY=VALUE`; the keys that
choose the model and region are Genesis Plus GX's `genesis_plus_gx_system_hw` and `_region_detect`, PicoDrive's
`picodrive_smstype` and `picodrive_region`, BlastEm's `blastem_sms_model` and `blastem_region`, Gearsystem's
`gearsystem_system` and `gearsystem_region`, SMS Plus GX's `smsplus_hardware` and `smsplus_region`, and the FM unit's
`picodrive_smsfm` and `smsplus_fm_sound` [M the run logs]. Which values each takes is read with `--list-options` when a
stage pins one.

### 5.3 A defect found by the gate: the published manifest named a library it did not carry

`EmuSen_Backend.md` §27.11 recorded that "a library in development is listed too, harmlessly, since a publish does not
carry the file a line names". It was measured when no core was in development. With Endou marked, a fresh publish's
`cores.manifest` listed `libendou.so`, the publish rightly did not carry it, and `sha256sum -c cores.manifest` exited 1
("1 listed file could not be read") [M]. **The fix**, in `EmuSen/EmuSen.csproj`: the build writes two manifests, the
build output's listing every library it holds, and a publish's listing only those not in development, copied to a
publish as `cores.manifest`. After it the publish verifies (six `OK`, exit 0), the build output still lists all seven
(`OK` each), and `CoreHostParityTests` pass over both trees [M]. `EmuSen_Backend.md` §27.11 carries the amendment.

## 6. Seeded faults (2026-10-19)

Each made in a copy, its tests run, the file restored and compared byte for byte (`seed.sh` in the step's scratch):

| # | Fault | Caught by |
|---|---|---|
| F1 | The checksum sums its own two bytes | three media tests |
| F2 | An NTSC frame of 263 lines | `the_rates_are_the_master_clock_over_a_frame` |
| F3 | The PSG's block saved but not loaded | `a_state_round_trips…`; and, on the built library, the kit's **C7, C8 and C12** on both synthetic images |
| F4 | The header looked for only at `$7FF0` | `a_copier_header_is_set_aside_and_the_console_follows_the_header` |
| F5 | `sms.json` names ScreenScraper's 3 | the systems crate's new test |
| F6 | A frame writes the wall clock into RAM | the kit's **C4, C7, C8, C10, C12, C13, C17 and C18** |
| F7 | A cartridge shorter than 1 KiB makes the machine panic | **nothing at first**: the kit's garbage image is long and no test created a machine from a short image. `an_image_of_any_length_up_to_the_largest_board_is_taken_and_runs` was added, and catches it |
| F8 | The C# literal names ScreenScraper's 3 | `SystemTableParityTests`, two cases |
| F9 | The SG-1000 offered while the core is in development | `EndouTests.The_library_is_on_the_core_abi_and_its_systems_are_the_packs`; the kit passes it, since the core is marked whole |

## 7. What was read for this stage

No emulator source, and no firmware or firmware listing. Read: `Endou_Plan.md`, `Endou_References.md`,
`Endou_Disputes.md`; Diamond's and Nephrite's stage-1 commits and `Diamond_References.md` §3–§5; `EmuSen_CoreAPI.md`
§6.31 and §27; `EmuSen_Backend.md` §27.11; SMS Power!'s "ROM Header" page; the cartridge header section of Sega's
*Software Reference Manual for the SEGA Mark III Console* (SMS Power!'s transcription), and its contents list; the
contents list of Sega's Game Gear document; the opening of MacDonald's `smstech-20021112.txt` and of Young's
`tms9918a.txt`; FluBBa's *SMS VDP Test* readme's first lines; the licence files of the test programs; SMS_MiSTer's
README and directory names; Mesen 2's README; ES-DE's `es_systems.xml`; ScreenScraper's system pages' titles; the
libretro cores' `.info` files; the run logs of §5.2.

## 8. Questions before stage 2, decided 2026-10-19

*Decided 2026-10-19 by the tester: each of the four as recommended.* The go for stage 2 came with the decisions; it
covers stage 2 alone. No real hardware is used for any of them (`Endou_Plan.md` §10 Q11).


1. **The system beside an image** (§2.3): may a host set `system` beside an image under the core ABI's next minor, so
   that a core running two systems takes the routed one? Recommended, before stage 4.
2. **MAME** is in Fedora's repositories (0.289) and not installed; installing a system package is the tester's to
   allow. **Mesen 2** runs the Master System and the Game Gear [R its README], but its repository is archived since
   mid-2025, with MesenCE its successor; the probe's Mesen backend would need its patch extended to the console.
   Recommended: neither before stage 3, where the VDP's pictures first need a fourth witness.
3. **A VHDL simulator for SMS_MiSTer**: its VDP, its YM2413 (`VM2413`) and its Z80 (`T80`) are VHDL [M its file
   names], and neither GHDL nor nvc is installed. Recommended: installed when the first dispute needs the referee,
   on the tester's word.
4. **The copier's header and the 512 KiB `.sg`** (§2.2, §2.3): stage 2 confirms the copier rule from a document and
   identifies the 512 KiB image's board.

*Decided 2026-10-19:*

1. **The system beside an image: accepted.** A host may set `system` beside an image under the core ABI's next minor.
   It is a change across components (`emusen-native`'s header, `emusen-core-host`, both Crystal implementations and
   the kit), so it is a piece of work of its own, decided and built before stage 4 and not inside stage 2. Its design
   is `Endou_Plan.md` §3.7.
2. **MAME is not installed before stage 3**, and Mesen 2 is not built before then either.
3. **A VHDL simulator** (GHDL or nvc) for SMS_MiSTer is installed only when the first dispute needs the referee, and
   only after asking the tester.
4. **The copier's header is confirmed against a document at stage 2**, with the 512 KiB `.sg`'s board.

---

## 9. Stage 2: the machine without its picture (2026-10-19)

### 9.1 In one paragraph

Endou now runs Beryl's Z80 on a bus of its own a frame of the master clock at a time: the memory map with port `$3E`'s
enables and the work RAM's mirrors; the ports decoded by A7, A6 and A0, with what an undriven read returns on each
console; the I/O chip's port `$3F`, the joystick ports `$DC` and `$DD`, the region they read back and the Reset button;
the Pause button's edge on NMI; and the cartridge's board: plain ROM, Sega's mapper with its RAM, Codemasters' mapper
and the SG-1000's RAM board, chosen by a table keyed by the image's CRC-32, Codemasters' own header, then the size.
The models are a setting: the Mark III, the Master System and the Master System II for `sms`, the SG-1000 for `sg1000`.
The battery file is the cartridge's RAM on the boards the table names. `beryl-z80` gained an assembler behind its
`asm` feature, its front half shared with the 68000's in a new crate, `beryl-asm`, and a power-on state. The VDP is
not built: its ports are a stand-in that reads `$FF` and takes no write until stage 3, so the picture stays black,
and the PSG is written and clocked but not mixed until stage 5. ZEXDOC and ZEXALL for the Master System pass on the
machine, 79 of 79 each, reporting through the cartridge's RAM.

### 9.2 Which board an image is on

`src/board.rs`, the rule of `Endou_Plan.md` §5.6 [C]: the **table** of boards keyed by the CRC-32 of the image (the
copier's header set aside), the key public per-game listings use; else **Codemasters' header** at `$7FE0` when its
checksum agrees [R SMS Power!'s "Codemasters Header": the little-endian words of the banks byte `$7FE0` names, summed
with Sega's header at `$7FF0`–`$7FFF` left out, the word after it summing with it to zero, `$7FEA`–`$7FEF` zero]; else
**Sega's mapper** for anything larger than 48 KiB; else **plain ROM**. A board the table cannot place runs on Sega's
mapper and is named by the survey (§9.10).

**Sega's boards are given the mapper's 32 KiB of RAM** unless the table says otherwise [C]: the board's RAM is not in
the image, and a program that enables it expects it (ZEXALL writes its report there, §9.9); it is no battery file.

**The table** (`TABLE`, nine rows): eight images of seven games on Sega's mapper with 8 KiB of battery RAM, and one
SG-1000 RAM board. Penguin Land and Phantasy Star are the games SMS Power!'s "SMS Paging Chips" lists with "Battery
Back-up RAM" [R]; Golden Axe Warrior, Golf Mania, Ultima IV and Ys are the others the survey saw write their
cartridge's RAM, and all eight wrote only its first 8 KiB [M §9.10], so 8 KiB with a battery is argued for them [A].
The Castle is the one SG-1000 image the survey saw write `$8000`–`$9FFF` [M], an 8 KiB board at `$8000` of the kind
MacDonald describes (`sc3000h` §3, RAM on the cartridge's `$8000`–`$BFFF` select) [R]. A battery game the table does
not name keeps its RAM only while the game runs: a named loss until a public listing completes the table (§9.15).

**Why a table and not a battery that appears.** `Endou_Plan.md` §4.4 preferred a machine info that gains its battery
the first time a game enables cartridge RAM, if the host saves a battery that appears after create. It does not:
`CoreEngine` takes the battery files machine info names at create, and a `MACHINE_INFO` event re-reads machine info
without taking them again [R `CoreEngine.cs`, `LoadRom` and `Drain`]. So the table form is used and nothing is
proposed (§3.6 of the plan).

**The copier's header, confirmed against a document** (stage 1's question 4, decided 2026-10-19). d0nut's *The
complete documentation about Genesis ROM format* (version 1.1, January 1998, in the probe's cache from Nephrite's fetch:
`nephrite/docs/zophar/genesis-rom-format.txt`, SHA-256 `2f7353ea…`) gives the Super Magic Drive's 512-byte header,
"Number of blocks" of 16 KiB in its first byte and the rest fixed values or zeros, and says to "skip the 512 bytes
header" [R]. The same copier family loads Master System cartridges: two of its own boot programs are in the tester's
library filed as `.sms`. **Measured on the 11 images** of §2.2: each one's first byte is its number of 16 KiB blocks (2,
16 or 32), its second `$01`, and the rest zero but for a title from byte 132 in three [M, a Python reader]; the
document's fixed `$03` and `$AA $BB` are the Genesis format's and do not appear, and the data after the header is not
interleaved, as Sega's header 512 bytes in shows. The rule (an image 512 bytes longer than a multiple of 16 KiB has its
first 512 bytes set aside) is kept as stage 1 built it, now [R] and [M].

**The 512 KiB `.sg`** (*Music Station*, stage 1's question 4): no Sega header, an 8 KiB-RAM stack pointer, and writes
to `$FFFC`–`$FFFF` in its first frames [M §9.10]: a cartridge on Sega's mapper, which the size rule already gives it.

### 9.3 The bus and the boards

`src/bus.rs` and `src/cart.rs`. The rules, each with its source, and the test that pins it (§9.8):

| Rule | Source |
|---|---|
| `$0000`–`$BFFF` the slot area, `$C000`–`$FFFF` 8 KiB of work RAM mirrored; on the SG-1000 1 KiB mirrored every kilobyte | [R SMS Power!'s "Memory Map", "RAM"; MacDonald's `smstech` §2] |
| Port `$3E`: bit 6 the cartridge, bit 4 the work RAM, bit 2 the I/O chip, each enabled when clear; the BIOS, card and expansion slots hold nothing until stage 6, so only the cartridge drives the slot area. A write to the slot area reaches the cartridge only while its slot is enabled; a write at `$C000` and above reaches it always, since Sega's registers lie there | [R `smstech` §4; Sega's Mark III manual, "Memory enables"; the always-reaching write [A]] |
| `$3E` at the start: `$AB`, what both export BIOSes MacDonald watched leave for a cartridge, since no BIOS runs before stage 6 | [R `smstech` §4] |
| Ports by A7, A6 and A0: `$00`–`$3F` even `$3E`, odd `$3F`; `$40`–`$7F` the PSG on write, the counters on read; `$80`–`$BF` the VDP; `$C0`–`$FF` `$DC` even and `$DD` odd on read, nothing on write; the high byte not decoded | [R `smstech` §3; SMS Power!'s "I/O Port Map - SMS"] |
| A read nothing drives: `$FF` on the Master System II; on the first Master System the bus's last byte, "the last byte of the instruction which read the port" — the bus keeps the last byte it carried; the Mark III and the SG-1000 taken as the first. The same for the acknowledge cycle's byte, `$FF` on the Master System II, "random" on the first | [R `smstech` §3–§5; the Mark III and SG-1000 [A]] |
| Ports `$C0`–`$FF` with the I/O chip off: undriven, as above | [R `smstech` §4] |
| Sega's mapper: `$FFFD`–`$FFFF` bank the three slots, `$0000`–`$03FF` stays bank 0, a bank past the ROM wraps; `$FFFC` bit 3 puts the RAM in slot 2, bit 2 its second 16 KiB, bit 4 the RAM over `$C000` (its first bank [A]); bits 1–0 shift every bank by `$00`, `$18`, `$10` or `$08` when a bank register is next written; the registers start at `0, 0, 1, 2`; the work RAM keeps a copy of each write | [R SMS Power!'s "Mappers"; `smstech` §2] |
| Codemasters' mapper: a write anywhere in a slot banks it, the first kilobyte not kept; banks 0, 1, 0 at the start; bit 7 of slot 1's value maps 8 KiB of RAM over `$A000`–`$BFFF` | [R "Mappers"; Endou_Disputes.md D-5] |
| Plain ROM mirrored over `$0000`–`$BFFF`; an SG-1000 RAM board's RAM mirrored through `$8000`–`$BFFF` | [A from the boards' address lines; `sc3000h` §3 for the RAM] |
| ROM writes do nothing; `$FFFC` bit 7, the "ROM write" enable, is not modelled, no known software using it | [R "Mappers"] |

**Each machine cycle takes the manual's length** (fetch 4, read and write 3, input and output 4 with the automatic
wait, the acknowledge 6), its time added before its access, as Nephrite's Z80 bus does; the Master System adds no wait
states [D `Endou_Plan.md` §5.1].

### 9.4 The models

`src/model.rs`, the one owner of the consoles' differences at this stage, and the `model` setting (`auto`, `mark3`,
`sms1`, `sms2`; `Endou_Plan.md` §5.8): from the region, a Japanese cartridge runs on a Mark III and any other on a
Master System II (§10 Q9); the `sg1000` system is the SG-1000 alone until stage 4.

| | Mark III | Master System | Master System II | SG-1000 |
|---|---|---|---|---|
| Ports `$3E`, `$3F` | yes | yes | yes | no |
| TH, TR as `$3F` sets them | TR an input; a TH output reads low, an input high (D-3) | yes | yes | — |
| Reset button | no | yes | no | no |
| An undriven read | the bus's last byte [A] | the bus's last byte | `$FF` | the bus's last byte [A] |

### 9.5 The I/O chip

`src/io.rs`. Port `$3F` starts `$FF`, every pin an input, as software's region checks assume [R "Peripheral Ports"].
`$DC`: port A's Up, Down, Left, Right, TL (button 1) and TR (button 2), then port B's Up and Down; `$DD`: port B's
Left, Right, TL and TR, the Reset button, CONT (high), and the two TH pins; active low [R `smstech` §4]. A pin set as an
output reads its level on the export consoles, so the region check of SMS Power!'s "Region Detection" finds an export
console on the Master System and II and a Japanese one on the Mark III [M §9.8]. The SG-1000's `$DD` reads its high four
bits as unconnected inputs [A]. **The Pause button** is an edge on NMI: a press reaches the latch at the frame's start,
so a held button makes one NMI [R Sega's Mark III manual, "NMI": "if the PAUSE button is held down, you will get a
single interrupt only"].

### 9.6 The scheduler

`src/machine.rs`: the bus counts Z80 clocks, the master clock's over 15. A frame ends at an absolute clock, 59,736 Z80
clocks later NTSC (228 a line, 262 lines) and 71,364 PAL (313 lines); the Z80 runs until the bus passes it, the
overshoot carried into the next frame. The PSG is brought to the bus's time at each write and at the frame's end, a
clock of it every 16 of the Z80's [R `Beryl_SN76489.md` §2], its output discarded until stage 5. The VDP joins at stage
3, brought to the present at each port access and line end (`Endou_Plan.md` §5.1). The Z80 starts from
`Z80::power_on()` (`Beryl_Z80.md` §8.4).

### 9.7 The state, version 2

`ENDO` version 2: the magic, the version, the console and model bytes, the frame count, the bus clock and the frame's
end, the Z80's block, the PSG's with its clock count, ports `$3E` and `$3F`, the data bus, the pending NMI, the Pause
latch, the mapper's registers and the three slots' offsets, then RAM, VRAM, CRAM and the cartridge's RAM where the
board has it: 24,733 bytes for a plain `sms` board, 57,501 with Sega's 32 KiB, pinned by `the_version_2_layout_is_pinned`.
Version 1, stage 1's stub, reached no player and is refused; a state of another console or model is foreign, and one
whose slot offsets lie outside the ROM is refused.

### 9.8 The test programs

`programs/*.s`, assembled at test time with `beryl-z80`'s assembler and run on the machine (`src/programs.rs`), and the
same images through the v1 adapter in `EndouTests` (`SyntheticSmsRom`'s hand-assembled programs):

| Program | What it pins |
|---|---|
| `pads.s` | the joystick ports, the Reset button and TH on each model, the SG-1000's pads |
| `pause.s` | one NMI a press, none more while held, RETN |
| `region.s` | SMS Power!'s region check: export on the Master System and II, Japanese on the Mark III |
| `sega.s` | Sega's mapper: each slot, the kept first kilobyte, the wrap, the RAM's copy of `$FFFF`, the cartridge RAM in both banks |
| `codemasters.s` | Codemasters' mapper: slots 2 and 1 by their addresses, the RAM over `$A000` with `$8000` still ROM |
| `port3e.s` | the cartridge and the work RAM off the bus from a routine in RAM, a lost write, port `$3E`'s own read, per model |
| `sgram.s` | the SG-1000 RAM board's mirror and the 1 KiB work RAM's |
| `psg.s` | the PSG written through `$7F` and `$41`; the VDP stand-in's `$FF` |

`examples/run.rs` runs an image and prints the cartridge RAM's report; `examples/survey.rs` is §9.10's;
`examples/referee.rs` writes the programs' images for the references and prints Endou's RAM (§9.11).

### 9.9 ZEXDOC and ZEXALL on the machine

The Master System port of ZEXALL, 0.20 (`Endou_References.md` §2), writes its report to the cartridge's RAM at
`$8000` through Sega's mapper, as well as to the SDSC console and the screen. Its source was read for that, and for its
start: it looks for a mode 4 VDP by the status port, and with the stand-in's `$FF` it reports "TMS9918 Text Mode" and
goes on. **Measured 2026-10-19 on the desktop, release build:**

| Program | Tests | Frames to "Tests complete" | Z80 clocks | Wall time |
|---|---:|---:|---:|---:|
| ZEXDOC | **79 of 79 OK** | 267,420 (74.4 minutes of the console's time) | 15,974,601,124 | 20.2 s, 0.076 ms a frame |
| ZEXALL | **79 of 79 OK** | 316,800 (88.1 minutes) | 18,924,364,804 | 23.5 s, 0.074 ms a frame |

The SCF/CCF pre-test, which the program says differs between Z80 makes, printed `0000 0000 1fff 1fff 0000 0000` and
`1fff` six times, twice over: Beryl's Z80, whose Q rule follows the published measurements (`Beryl_Z80.md` §4.2). The
test `zexdoc_and_zexall_pass_on_the_machine` runs both where `EMUSEN_ENDOU_CORPUS` names the probe's cache, and passes
unrun without it: 43.9 s for the two on the `test-checked` profile [M].

The frame's cost with no VDP, 0.074–0.076 ms, is the Z80 and the bus alone; §5.9's budget is for the whole machine.

### 9.10 The library survey

`examples/survey.rs` over `mastersystem/`, read in place: each image's board as the core chooses it, then 3,000 frames
(50 seconds of the console's time) with no VDP, every write to Sega's registers, to the slot area and to cartridge RAM
tallied. **Six of the 370 files are firmware** (four Master System BIOS images, two copier boot programs) and are left
out by the clean-room rule (`Endou_Plan.md` §1.4); 364 game images remain. Measured 2026-10-19 [M]:

| | Images | Board |
|---|---:|---|
| `.sms`, larger than 48 KiB | 297 | Sega's mapper by size; 277 wrote its registers within the 3,000 frames, 20 did not (most waiting on the VDP; two Korean images and a Brazilian one are other mappers, stage 12) |
| `.sms`, 48 KiB or less | 11 | plain ROM: ten of 32 KiB and one with a copier's header |
| `.sms` with Codemasters' header | 2 | Codemasters' mapper, the header's checksum right in both; no other image has one |
| `.sg`, 48 KiB or less | 53 | plain ROM, but The Castle, on its RAM board by the table |
| `.sg`, larger | 1 | *Music Station*, Sega's mapper (§9.2), its registers written |

36 of the 277 wrote bank numbers past their ROM, most with bit 7 set (`$82`, `$87`), which the wrap answers as the
mapper's page says [R "Mappers": "Some software may also set higher-order bits"]. Nine images enabled Sega's cartridge
RAM: eight wrote it, every one below its 8 KiB mark, and one (*Desert Speedtrap*) enabled it and wrote nothing in the
frames run. None put the RAM over `$C000`. On the SG-1000, The Castle wrote `$8000`–`$9FFF`, and *Sinbad Mystery*
wrote `$FFFF` and `$0000`, which its plain board ignores. The tallies are in the step's scratch; the table of §9.2 holds
what they decided.

### 9.11 The referees, run as black boxes

The programs of §9.8 as `.sms` images (Sega's export header added), run for ten frames on the five pinned libretro
cores through the probe, each pinned to a model and region by `--option`, and RAM's first 16 bytes compared with Endou's
on the same model [M 2026-10-19]:

- **`region.s`**: export on every export configuration; Japanese on Genesis Plus GX's Mark III, SMS Plus GX's and
  Gearsystem's Japanese settings; PicoDrive's and BlastEm's Japanese settings say export. Endou agrees with the three.
- **`pause.s`** with Start pressed at frame 3 for three frames and at frame 7: two NMIs on Genesis Plus GX, SMS Plus
  GX, Gearsystem and PicoDrive, as `a_pause_press_is_one_nmi_and_a_held_button_no_more` pins; none on BlastEm, whose
  Start is not the Pause button.
- **`sega.s`**: every reference and Endou agree on every bank, the kept first kilobyte, the wrap and the RAM's copy of
  `$FFFF`. The cartridge RAM's second bank, never written, reads `$FF` on Genesis Plus GX and Gearsystem and `$00` on
  the others and Endou: a power-on fill, not a rule.
- **`codemasters.s`**: only PicoDrive took the synthetic image as Codemasters' board, and its bytes equal Endou's
  exactly; Genesis Plus GX, SMS Plus GX and Gearsystem ran it on Sega's mapper, and BlastEm stopped with a fault.
- **`port3e.s`**: the references disagree with each other and with MacDonald. BlastEm, Gearsystem and PicoDrive read
  the cartridge with its slot off and keep a write to work RAM switched off; Genesis Plus GX and SMS Plus GX lost the
  program; every one reads port `$3E` as `$FF`, the first Master System's setting included. Endou follows the document
  (Endou_Disputes.md D-4).
- **`pads.s`**: idle pads `$FF $FF` everywhere but BlastEm, whose `$DD` reads CONT low as a Genesis's does
  [R `smstech` §4]. Genesis Plus GX's Mark III reads TH high as an input, which D-3 took.
- **`psg.s`**: the counters and the status differ, as the stand-in reads `$FF` until stage 3.

### 9.12 The gate (`Endou_Plan.md` §7, stage 2), measured 2026-10-19

| Gate item | Result [M] |
|---|---|
| The crate's tests | **43 of 43** (board 2, bus 4, cart 5, io 3, machine 4, media 4, programs 10, state 3, v1 8), ZEXDOC and ZEXALL's run where the corpus is given |
| The SMS-hosted ZEXALL on the whole machine | **79 of 79**, and ZEXDOC 79 of 79 (§9.9) |
| Synthetic programs for each port, mapper and interrupt rule | §9.8, every one passing; `EndouTests` **9 of 9**, two new: each mapper's banking on `SyntheticSmsRom`'s images, and the pads with the Reset button |
| The kit | every case it runs passes (C1–C15, C17–C23) on `EndouTests`' four synthetic images, the seven program images, ZEXALL, *SMS VDP Test*, both SG-1000 PSG programs, and four of the tester's games read in place (one on each board kind: a battery board, Codemasters', Sega's, the SG-1000 RAM board), 120 or 300 frames each. **C7 found a defect first**: a plain 32 KiB board's state was refused by a fresh machine (its third slot's offset lay past the ROM's end); fixed, and `every_board_s_state_loads_on_a_machine_just_made` added |
| RAM against the references at anchors on test programs | §9.11 |
| `beryl-z80` | **19 of 19** with the assembler, 15 without; `beryl-asm` 3 of 3; `beryl-m68k` 26 with its assembler, 21 without; Nephrite's crate **194 of 194** on `test-checked` and `NephriteTests` with the corpus 58 of 58; the open Sega CD BIOS byte-identical; **the kit's corpus on Nephrite** (`genesis-sample.txt`, 64 images, 600 frames), the base's library against this branch's: 64 of 64 pass on both, **the 64 image reports identical**, the summaries differing only in the library's path, the timings, the peak memory and C13's count of rounds |

Toolchains: rustc 1.99.0, .NET 10.0.112, on the desktop (Ryzen 7 7700X).

### 9.13 Seeded faults (2026-10-19)

Each made in the source, the crate's tests run, the file restored and compared byte for byte (`seed.py` in the step's
scratch):

| # | Fault | Caught by |
|---|---|---|
| F1 | Sega's first kilobyte banked with slot 0 | `cart`'s and the program's Sega tests |
| F2 | Sega's bank shift applied when `$FFFC` is written | `sega_s_shift_applies_when_a_bank_is_next_written` |
| F3 | Codemasters' RAM over all of slot 2 | `cart`'s and the program's Codemasters tests |
| F4 | an export console's TH output read as its input | the `io` region test and `the_region_reads_back_as_each_console_s` |
| F5 | the Reset button on every console | `only_the_first_master_system_has_a_reset_button` and the pads program |
| F6 | a held Pause button an NMI every frame | `a_pause_press_is_one_nmi_and_a_held_button_no_more` |
| F7 | `$DC` and `$DD` swapped by A0 | the pads and region programs |
| F8 | port `$3E`'s cartridge bit ignored | `bus` and `port3e.s` |
| F9 | the Master System II floating its bus | `bus` and `port3e.s` |
| F10 | a write leaving the data bus as it was | **nothing at first**: no program reads the bus right after a write without an instruction byte between. Pinned by the acknowledge cycle after a write in `each_machine_cycle_takes_its_length`, which catches it |
| F11 | Sega's RAM bank bit ignored | `sega_s_ram_takes_slot_2_in_either_bank` and the Sega program |
| F12 | the battery file not loaded at create | **nothing at first**: no test made a battery board through `create`. `create` now goes through `make` with the table as a parameter, and `a_battery_board_takes_its_file_at_create_and_gives_it_back` catches it |
| F13 | Codemasters' header taken without its sum | the board test |
| F14 | port `$3F` not restored by a state | the state round trip |
| F15 | the assembler's `JR` counted from the instruction | three assembler tests |
| F16 | the assembler's index displacement losing its sign | two assembler tests |

### 9.14 What was read for this stage

No emulator source, no SMS_MiSTer RTL, and no firmware or firmware listing; the firmware images in the library were
left out of the survey. Read: `Endou_Plan.md`, `Endou_Native.md`, `Endou_References.md`, `Endou_Disputes.md`; the Beryl
pages and crates; SMS Power!'s "Mappers", "SMS Paging Chips", "Memory Map", "RAM", "Port $3E", "I/O Port Map" (SMS and
SG-1000), "Peripheral Ports", "Controller Port", "Joypad", "Region Detection", "Reset Button", "Pause Button",
"Interrupt Mechanism", "Clock Rate", "TV Type Detection", "ROM Header", "Codemasters Header", "Cartridge Slot";
MacDonald's `smstech` §1–§5 and `sc3000h` §2–§5; Sega's Mark III manual, "The CPU", "Memory management" and
"Input/Output ports" with appendix B; "The Undocumented Z80 Documented" §2.4; d0nut's Genesis ROM format document,
its copier sections; ZEXALL-SMS's README and source; `CoreEngine.cs`'s battery handling; `EmuSen_CoreAPI.md` §4,
§6.11, §6.19, §6.27, §6.31–§6.32 and §6.39.

### 9.15 Questions before stage 3

1. **The battery table.** Seven games have their battery boards in the table, from one SMS Power! page and the survey.
   May a public per-game listing (SMS Power!'s per-game pages, or a No-Intro listing of the Master System) be fetched
   at stage 7 to complete it? Recommended: yes, at stage 7, recorded in `Endou_References.md` as every fetch is.
2. **Port `$3E`** (D-4): the references disagree with the document and with each other, so the documents' rules stand,
   argued. SMS_MiSTer would referee it, which needs a VHDL simulator (§8, question 3). Recommended: not now; a game
   shown to need otherwise reopens it.
3. **The Mark III's TH** (D-3) is argued from SMS Power!'s pages and one reference. Recommended: as it is, until the
   Light Phaser's stage, which reads TH.
4. **CI** now tests Beryl's crates with all their features, so both assemblers' tests run there.

*Decided 2026-10-19 by the tester: each as recommended.* The go for stage 3 came with the decisions; it covers stage 3
alone. No real hardware is used for any of them (`Endou_Plan.md` §10 Q11).

1. **The battery table:** at stage 7, a public per-game listing (SMS Power!'s game pages or a No-Intro list of the
   Master System) may be fetched to complete it, recorded in `Endou_References.md` as every fetch is.
2. **D-3 and D-4 stay as argued** from the documents (the Mark III's TH, port `$3E`) until a game shows otherwise; TH
   comes back at the Light Phaser's stage, which reads it.
3. **CI testing the Beryl crates with `--all-features`** is confirmed.

---

## 10. Stage 3: the VDP's mode 4 (2026-10-19)

### 10.1 In one paragraph

The Master System's consoles now have their picture. A new shared crate, `beryl-sms-vdp` (`Beryl-HW/Beryl_SmsVdp.md`),
is the 315-5124, the 315-5246 and the 315-5378 in mode 4: the ports and the command word, the registers, VRAM and
CRAM, the H and V counters with the H counter's latch, the frame and line interrupts, the status flags, the 224- and
240-line displays, the 315-5124's mask bits, and a composition of sprites (eight a line, tall, zoomed, the early clock,
collision and overflow) and background (scroll locks, the fine scroll's blank, the left column's mask) drawn up to the
dot of every write. Its timing within a line is held to FluBBa's *SMS VDP Test*, whose 54 Master System cases all pass
on every model, NTSC and PAL. Endou wires it to the bus at the master clock over five, gives the status's undriven bits
the console's bus, latches the H counter as TH rises, takes INT at each instruction's end, and turns the chip's CRAM
entries into colours at D-1's equal steps. The SG-1000 keeps stage 2's stand-in until stage 4 builds its TMS9918A. The
access slots are not modelled: no access is lost, which the library's games argue for (D-8).

### 10.2 Why a crate, and what it holds

`Endou_Plan.md` §3.3 (decided 2026-10-19, Q7): the family's chip is one Beryl crate, its line composition a public
module that the Genesis's mode 4 is to share at stage 4. The crate holds the chip whole; Endou holds the wiring and the
colours. The surface Endou uses: `Vdp::new(variant, pal)`, `run(to)`, the five port calls, `latch_h`, `irq`,
`picture` and `height`, `skip`, and the state block. A new dependency of Endou's, said here as the project's rules ask;
nothing else gains one, and no existing Beryl crate changed.

### 10.3 The wiring

`src/bus.rs`, as `Endou_Native.md` §9.3's decoding gives the ports:

| Rule | Source |
|---|---|
| The VDP's clock is the bus's times three, the master clock over five against the Z80's over fifteen; it is brought to the present at each of its ports, at each instruction's end where INT is sampled, and at the frame's end | `Endou_Plan.md` §5.1 |
| Ports `$40`-`$7F` read the V counter at even addresses and the latched H counter at odd ones; `$80`-`$BF` the data port at even addresses and the status at odd ones, and write the data and control ports | [R MacDonald's `smstech` §3, `msvdp` §2] |
| The status's bits 4-0 are the console's undriven read: the bus's last byte on the Mark III and the first Master System, `$FF` on the second | [R `msvdp` §4: "garbage"; the undriven read of §9.3] |
| A write to port `$3F` that makes either TH pin rise, as §9.5 gives each console's TH, latches the H counter | [R `msvdp` §11; D-3's note of stage 3] |
| The 315-5124 in the Mark III and the first Master System, the 315-5246 in the second | [R `msvdp` §1] |
| The SG-1000's VDP ports keep stage 2's stand-in: the counters undriven and the VDP `$FF`, the picture black | stage 4's |

The machine's frame of 262 or 313 lines starts on the chip's last line (`Beryl_SmsVdp.md` §3.3): a frame's picture is
whole before the frame ends, and nothing of the next is drawn.

### 10.4 The picture

The chip's picture is CRAM entries; `machine::SMS_COLOURS` turns a `--BBGGRR` entry into RGBA at 0, 85, 170 and 255 a
component (D-1), once a frame and never when the frame's picture is skipped. The picture's height is the frame's active
lines, 192, 224 or 240; machine info keeps 256 by 192 with a largest of 256 by 240, and frame info gives each frame's.
Machine info now says `skip_rendering_state_neutral`, which the kit's C8 checks on every image of §10.7.

### 10.5 The test programs

Three join `programs/` (`Endou_Native.md` §9.8): `picture.s`, a mode 4 picture of tiles, a sprite, a fine scroll, the
masked column and a CRAM entry changed mid-line by the line interrupt and back by the frame interrupt; `slots.s`, the
data port written at four rates and read back (D-8); and `hlatch.s`, the H counter latched by TH rising through port
`$3F` and not by it falling. WiseMan's `SyntheticSmsRom` gains `Ports` and `Picture`, a program of port writes and a
mode 4 picture whose masked column shows the backdrop, which `EndouTests.The_vdp_draws_a_synthetic_program_s_picture`
holds through the v1 adapter. `examples/screen.rs` runs an image with pad presses and shows its name table as text,
its picture as a PPM or raw RGBA, its RAM, VRAM and registers.

### 10.6 The state, version 3

`ENDO` version 3: version 2's fields to the slots, then RAM, the VDP's block (`Beryl_SmsVdp.md` §6) with VRAM and
CRAM, then the cartridge's RAM where the board has it: 24,879 bytes for a plain `sms` board, 57,647 with Sega's 32 KiB,
pinned by `the_version_3_layout_is_pinned`. Version 2 reached no player and is refused.

### 10.7 The gate (`Endou_Plan.md` §7, stage 3), measured 2026-10-19

| Gate item | Result [M] |
|---|---|
| The VDP test programs, self-reporting ones by their verdicts | **FluBBa's *SMS VDP Test* 1.31: all 54 Master System cases pass** (pages 1-3: the data port, the counters and interrupts, the sprites) on the Mark III, the Master System and the Master System II, NTSC, and the Master System II in PAL; `flubba_s_vdp_test_passes_on_every_model` with the corpus. Of the references Gearsystem and SMS Plus GX pass every case, Genesis Plus GX all but "Offscreen Y, col", PicoDrive fails the five H count cases, and BlastEm did not start the test from a pad press (D-9). **sverx's *SMS Test Suite* 0.36**, which reads the console it runs on, names "Master System II" with "USA/BRA 60Hz (NTSC)" on the default, "Master System" on the first Master System, "Master System" with "JPN/KOR" on the Mark III (a 315-5124) and "EUR/AUS 50Hz (PAL)" in Europe |
| Pictures equal to the references up to a colour map, or a dispute | §10.8 |
| The skip-versus-draw state test | `skipping_rendering_leaves_the_state_as_drawing` on `picture.s`, the crate's own, and the kit's C8 with rendering skipped on alternate frames on all 24 images below, eight of them games (P9) |
| The kit | **every case it runs passes** (C1-C15, C17-C23), 600 frames, on the eleven program images, *SMS VDP Test*, *SMS Test Suite*, MacDonald's *64 Color Palette*, *Scroll Test* and Bock's *Fullscreen Palette Test*, eight of the tester's games read in place (Sonic the Hedgehog, Space Harrier, Micro Machines, Cosmic Spacehead, Phantasy Star, Alex Kidd in Miracle World, After Burner, OutRun) and an SG-1000 game |
| The crates' tests | `beryl-sms-vdp` **23 of 23**, on `test-checked` too; `endou` **48 of 48** (ZEXDOC, ZEXALL and FluBBa's test where the corpus is given); `EndouTests` **10 of 10**, with `SystemTableParity` and `CoreRegistration` 21 of 21 |
| Seeded faults | §10.9 |
| The cost | the desktop's mean a frame, release, median of three runs of 3,600 frames with the machine otherwise busy (load 5-6): Sonic the Hedgehog 0.22 ms, Space Harrier 0.22, Golden Axe 0.26, OutRun 0.23, Micro Machines 0.23; stage 2's machine without the VDP was 0.075. P1's budget is 0.6 ms with the sound still to come |

Toolchains: rustc 1.99.0, .NET 10.0.112, on the desktop (Ryzen 7 7700X).

### 10.8 Pictures against the references

Each program or game run to a frame on Endou's Master System II and on the five references as black boxes through the
probe, every option at its default, and the pictures compared up to a colour map (`compare.py` in the step's scratch;
BlastEm's 256 by 224 cropped at row 16, PicoDrive's 248-wide pictures against Endou's columns 8-255). Digests are
Endou's frame, FNV-1a over its RGBA; no picture is committed.

| Image, frame | Endou's digest | Genesis Plus GX | Gearsystem | SMS Plus GX | PicoDrive | Note |
|---|---|---|---|---|---|---|
| *SMS VDP Test*, menu (300) and page 1 (700) | `027107BD777DCA10`, `F6EDA2E6A2D8EE03` | 0 | 0 | 0 | 0 | BlastEm stays on its menu |
| `picture.s` (10) | `FDEA985F7BF46025` | 0 | 8, one line | 0 | 0 | Gearsystem changes the colour a line earlier |
| *Color Check*, *Checkstate*, *SMS Palette*, *Fullscreen Palette*, *Mark III BG Test*, *V Counter Test* (300-600) | | 0 each | 0 each | 0 each | 0 each | on the Master System II; on the Mark III, which a cartridge with no region gets, *Color Check* shows the 315-5124's name table mirroring its first rows, as `msvdp` §7 and "Tilemap Mirroring" describe for homebrew assuming a 315-5246 |
| *Scroll Test* (300) | `45A050A0FFC88C21` | 511 | 511 | other phase | 5,208 | the fine scroll's first pixels: D-7 |
| *64 Color Palette* (600) | `76BA586C31A8B325` | 448 | 832 | 672 | 416 | CRAM written during each band's first line; the four disagree with each other on which pixels take the new colours, so the dot of a mid-line write stands as `Beryl_SmsVdp.md` §3.2 and §5 have it |
| *SMS Test Suite* (600) | `521C73F1B9F96C1B` | blank | blank | blank | 304, one text line | PicoDrive names itself "Genesis/MegaDrive"; the other three show a blank screen at this frame |
| Sonic the Hedgehog (600) | `1D034E67A78698FD` | 0 | 0 | 0 | 0 | |
| Phantasy Star (600) | `12DC77F77BA1D5BE` | 0 | 0 | 0 | 0 | |
| Fantasy Zone (600) | `8733F30473C625D7` | 0 | 0 | 0 | 0 | |
| Alex Kidd in Miracle World (600) | `21C1F9765DFA24F4` | 213 | 0 | 0 | 0 | |
| Cosmic Spacehead (600), 224 lines | `4835AB494F15F669` | probe crashed | 0 | 0 | 0 | |
| Micro Machines (600), 224 lines | `0083A0F3A6AFC66F` | probe crashed | 5,966 | 5,966 | 5,966 | the references run the (E) image in PAL by their own databases; Endou's region setting at Europe gives `8BB6DDD1A9EC173F`, equal to all three |
| Space Harrier (600) | `996D9B42D3ACFB7D` | 51 | 51 | 51 | 51 | Endou's picture is the references' four frames later: §10.10 |
| OutRun (600) | `79F18FD1D7D3B16A` | 471 | 471 | 462 | 462 | a different moment of the attract mode's road, the references agreeing with each other |

### 10.9 Seeded faults (2026-10-19)

Each made in the source, the crates' tests run with the corpus (but ZEXDOC and ZEXALL), the file restored and compared
byte for byte (`seed.py` in the step's scratch); then those caught only by the corpus or by nothing were run again
without it:

| # | Fault | Caught by |
|---|---|---|
| F1 | the V counter's NTSC jump a line late | `the_v_counter_runs_as_the_document_lists_it`, FluBBa's test |
| F2 | the line counter not stepping on the line after the display | **only FluBBa's test at first**: the line counter's test ran register 10 at 2, whose interrupts miss line 192. It now runs it at 0 as well, and catches it |
| F3 | overflow at the eighth sprite | `a_ninth_sprite_is_overflow_and_the_first_eight_keep_their_order` |
| F4 | a priority tile's colour 0 covering sprites | `a_priority_tile_covers_sprites_but_not_with_its_colour_0` |
| F5 | the left column's mask seven pixels wide | the crate's mask test and the picture program |
| F6 | a status read leaving the line flag | the line counter's test and the picture program |
| F7 | code 0 not reading ahead | the command word's test, `slots.s`, FluBBa's test |
| F8 | the H counter latched at TH's rise, not a dot after | **only FluBBa's test at first**: the latch's test latched at a dot where the two give one value. It latches at the next dot now, and catches it |
| F9 | the Y scroll taken every line | **nothing at first**: no test wrote register 9 mid-frame. `the_y_scroll_is_taken_as_a_frame_begins_and_the_x_scroll_as_each_line_does` was added, and catches it |
| F10 | the 315-5124's name table mask ignored | `the_first_master_system_masks_name_rows_and_pattern_fetches_with_its_old_registers` |
| F11 | the X scroll not restored by a state | **nothing at first**: the state's round trip saved with both scrolls at 0. It sets them first now, and catches it |
| F12 | collisions found only while drawing an active line | the crate's skip test, FluBBa's test |
| F13 | the status's undriven bits read as 0 | the PSG program's status read |
| F14 | red and green swapped in the colours | the picture program |
| F15 | the H counter latched as TH falls | **only FluBBa's test at first**; `hlatch.s` and `the_h_counter_is_latched_as_th_rises_through_port_3f_and_not_as_it_falls` were added, and catch it |
| F16 | the frame interrupt a line early | **only FluBBa's test at first**: the frame interrupt's test looked for INT a clock before line `$C0` rather than `$C1`. It looks before `$C1` now, and catches it |

### 10.10 Open

- **Space Harrier starts four to five frames later than on all four references.** Its RAM equals Gearsystem's from frame
  4 to 10; during its opening, a run-length copy of 16 KiB of patterns to VRAM with the display off, Endou's VRAM fills
  at about 0.86 of PicoDrive's rate (PicoDrive exposes VRAM), so that Endou's frame 34 is the references' 29. The copy
  is one routine of plain instructions with no port read, and three test programs that run the same instruction mix
  (the routine itself over a table of literal and repeated runs, with an interrupt counting the frames) give the same
  count on Endou and on every reference; the cycles a frame are equal too (1,659 polls of the frame flag on each). The
  cause is not found; the goldens' stage (G5) takes it with the commercial games' anchors.
- **The fine scroll's first pixels** (D-7) and **the access slots** (D-8) are argued.
- **A mid-line write's dot** is the dot of the write; the chip's fetch pipeline, which runs ahead of its pixels, is not
  modelled (`Beryl_SmsVdp.md` §5).
- **The probe's Genesis Plus GX crashed** on the two Codemasters images; the probe's other cores ran them.

### 10.11 What was read for this stage

No emulator source, no SMS_MiSTer RTL, no firmware or firmware listing. Read: `Endou_Plan.md`, `Endou_Native.md`,
`Endou_References.md`, `Endou_Disputes.md`; the Beryl pages and `beryl-sn76489`'s and `beryl-z80`'s surfaces;
Nephrite's `render.rs` mode 4 and its D-7 and `Nephrite_Native.md` §15; MacDonald's `msvdp` whole and `smstech` §3;
SMS Power!'s "VDP Registers", "Scanline Counter", "SMS1/SMS2 VDPs", "Sprites", "Tilemap", "Tilemap Mirroring", "VRAM
Memory Map", "Tiles", "Palette", "Pixel Aspect Ratio", "Video Output", "VDP", "Tec Toy VDP", "TV Type Detection",
"Interrupt Mechanism", "GG VDP", "I/O Port Map - SMS"; Sega's Mark III manual, "Interrupt system" and "VDP registers"
through register 10; Talbot-Watkins's VDP registers; FluBBa's *SMS VDP Test* readme and `testmain.asm`; MacDonald's
*Scroll Test*'s and *64 Color Palette*'s `TEST.ASM` and `vdp.inc`. Of Space Harrier's cartridge, for §10.10, the bytes of
its reset code and of the copy routine at `$06A6`, as hexadecimal; nothing of it is kept.

### 10.12 Questions before stage 4

1. **D-1's boosted blue.** SMS Power!'s "Palette" gives the 315-5124 (the Mark III and the first Master System) a
   first blue level of `#68` where the 315-5246's is `#55`, naming no measurement. Should Endou take it for those two
   models? Recommended: yes, as the one document that speaks to the 315-5124's levels, marked [R] and not [M]; the
   equal steps stay on the 315-5246.
2. **A VHDL simulator for SMS_MiSTer** (GHDL or nvc), to place the access slots (D-8) with `slots.s` stepped from 11 to
   29 cycles, and to referee D-6, D-7 and Space Harrier's opening as a black box. Recommended: yes, run as Nephrite's
   rule and §1.6 of the plan allow, the RTL opened only in a logged dispute step.
3. **The region of an (E) image.** Micro Machines (E) carries an export header, so Endou runs it as an American NTSC
   console; the references run it in PAL from their databases. Recommended: as `Endou_Plan.md` §10 Q9 decided, PAL by the
   region setting, and a table of PAL-only games only when one is shown to need it; Micro Machines runs in either.
4. **MAME** as a fourth witness for the pictures (`Endou_Native.md` §8, question 2): recommended not yet; the four
   libretro references agreed with each other or disagreed in ways MAME would not settle.
