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

**Built 2026-10-19, core ABI 1.15** (`EmuSen_CoreAPI.md` §6.40, `EmuSen_Backend.md` §31). Endou honours the routed
system: `Core::create_as` reads it with `Console::of_id`, and `Media::read_as` takes it in place of the rule, which
`Media::read` keeps for a host that names none. The rule is unchanged, and so is every image's board, market and
model given its console; a system Endou does not list is refused by `emusen-native` before Endou sees it, and
`create_as` refuses one again in words should the export ever pass one. Under a 1.15 host the three images above run as
their extensions' systems, the 512 KiB `.sg` as an SG-1000 and the two 8 KiB `.sms` as Master Systems, and the other
367 as before. `EndouTests.A_headerless_image_runs_as_the_system_it_was_routed_to` opens a 64 KiB `.sg` and an 8 KiB
`.sms` without the header through `CoreFactory`, the host's own routing. With the host passing no system beside the
image, as at the base, both failed, the machines reporting `sms` and `sg1000`, the rule's guesses; on the build they
pass, and the same images made with no system named still get the rule's guesses (measured on 2026-10-19).

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
  *At stage 5 (2026-10-19):* BlastEm, not among the four, plays Space Harrier's sound on Endou's course, matching it at
  lag 0, while the four keep theirs (§12.6).
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

*Decided 2026-10-19 by the tester: each of the four as recommended, and one more.* The go for stage 4 came with the
decisions; it covers stage 4 alone. No real hardware is used for any of them (`Endou_Plan.md` §10 Q11).

1. **D-1's boosted blue is taken** for the 315-5124 (the Mark III and the first Master System): blue level 1 shown at
   `#68`, as SMS Power!'s "Palette" gives it, marked as read from a document, not measured. The 315-5246 keeps the
   equal steps.
2. **A VHDL simulator (GHDL or nvc) is approved** for SMS_MiSTer as a black-box referee. The tester installs it, since
   it needs the administrator's rights. When it is present, SMS_MiSTer may be run as a black box for D-6, D-7 and D-8
   (`slots.s` stepped from 11 to 29 cycles) and for Space Harrier's opening; its RTL is read only in a logged dispute
   step, each file listed, by Nephrite's rule. When it is absent, those disputes stay open and the record says so.
3. **An (E) image runs in PAL only by the region setting**; no per-game table of PAL games is kept.
4. **MAME is not used** as a reference or a referee.
5. **Under core ABI 1.15, MercuryRT and Nephrite keep their own system rules**; a 32X header recogniser in
   `emusen-systems`' routing is a separate piece of work, and Endou's stages leave routing alone.

---

## 11. Stage 4: the TMS9918's modes, the SG-1000, and Nephrite on the shared composition (2026-10-19)

### 11.1 In one paragraph

`beryl-sms-vdp` gains the TMS9918's modes (Graphics I and II, Text, Multicolor and the four undocumented combinations)
with their sprites, on the 315-5124 and 315-5246 and on a fourth chip, the TMS9918A itself (`Beryl_SmsVdp.md` §9).
The SG-1000 is now a modelled console: the TMS9918A in place of stage 2's stand-in, its colours TI's (D-11), the
discrete PSG and 1 KiB of RAM, and the Mark III offered as its second model, whose 315-5124 shows the SG-1000's pictures
in its fixed colours. The 315-5124 takes the boosted blue the tester decided (D-1). Nephrite's mode 4 is drawn by the
crate's composition with a Genesis variant, its D-7 tests and its corpus digests unchanged (§11.6). Every SG-1000 game
of the library passes the kit on both models, and its pictures equal the references' up to a colour map but where the
references differ among themselves or a game's play takes another course (§11.10). SMS_MiSTer could not be run as a referee: GHDL is installed, but the core
needs Intel's `altera_mf` and a Verilog PSG (§11.11).

### 11.2 The decisions of 2026-10-19, as built

| Decision | Built |
|---|---|
| D-1's boosted blue for the 315-5124 | `machine::SMS1_COLOURS`, blue's first level `#68`, for the Mark III and the first Master System; `SMS_COLOURS` for the Master System II (D-1, read from a document) |
| A VHDL simulator for SMS_MiSTer | GHDL 6.0.0, installed without the administrator's rights (`Endou_References.md` §4a); the referee could not be elaborated (§11.11) |
| (E) images in PAL by the region setting only | nothing to build: the region setting already chooses PAL; no table was added |
| MAME not used | struck from the plan's references (`Endou_Plan.md` §1.7) and `Endou_References.md` §3 |
| MercuryRT and Nephrite keep their system rules; routing untouched | no change to `emusen-systems` or to either core's `create_as` |

### 11.3 The TMS9918's modes on the Master System's chips

`beryl-sms-vdp`'s new module `tms` composes a line in these modes as TMS9918 colour codes, 0 transparent; `Vdp` turns a
code into what the chip puts out (`Beryl_SmsVdp.md` §2.2, §9). On the 315-5124 and 315-5246 that is a fixed colour of
their 64, from SMS Power!'s "Palette" table [R], so that Endou colours it as it colours CRAM's, the 315-5124's blue
boosted. Register 0's bit 2 (M4) clear selects them; the Sega chips keep their line interrupt and counters in them [A:
they are the chip's, not the mode's]; their status's low five bits stay undriven [R MacDonald §4]; sprites are four a
line (D-10) and move a further eight pixels by register 0's bit 3 [R MacDonald §15]; and the 315-5124's registers 3
and 4 mask Graphics II's tables as the TMS9918A's do, the 315-5246's not [R MacDonald §15]. The invalid text mode (M4
with M1) is not modelled and shows the backdrop, a named loss: no document read says how its tiles "repeat four
times". `F-16 Fighting Falcon`, the one Master System game the documents name in these modes, is not in the library.

### 11.4 The TMS9918A, and the SG-1000 as a console

`Variant::TMS9918A` (`Beryl_SmsVdp.md` §9.3): eight registers written by a second byte with bit 7 set, the number in
bits 2-0; no CRAM; F, 5S, C and the fifth sprite's number in the status, 5S only while F is clear; no line interrupt;
the 4K mode's map (D-12). `model.rs` is the one owner of what each console holds, now its VDP, PSG and RAM too:

| | SG-1000 | Mark III running an SG-1000 cartridge |
|---|---|---|
| VDP | TMS9918A | 315-5124 |
| PSG | discrete SN76489AN (D-0) | Sega's |
| Work RAM | 1 KiB, mirrored | 8 KiB |
| Ports `$40`-`$7F` read | the open bus [R SMS Power!'s "I/O Port Map - SG-1000"] | the V and H counters |
| Status bits 4-0 | the fifth sprite's number | the bus's |
| Colours | TI's Table 2-3 as RGB (D-11) | the 315-5124's fixed colours |
| Spaces | RAM, VRAM, ROM | RAM, VRAM, CRAM, ROM |

**The model setting** (`Endou_Plan.md` §5.8) gains `sg1000`: an SG-1000 cartridge runs on the SG-1000, or on the Mark
III where the setting names it, the consoles whose slot takes it; a Master System cartridge never on the SG-1000, a
setting naming it falling back to the region's console. The SG-1000's system stays `sg1000` on either model.
`v1::tests::the_model_setting_offers_each_cartridge_the_consoles_whose_slot_takes_it` holds it. The stand-in and its
`has_vdp` are gone from `bus.rs`; the status's driven bits are the chip's (`Vdp::status_bits`).

### 11.5 What is argued

- **The TMS9918A's timing** is the family's (`Beryl_SmsVdp.md` §3): TI gives no dot for the interrupt or the sprite scan.
- **The TMS9918A's first control byte** is taken at once, as the 315-5124's is; no document read says otherwise.
- **The fifth sprite number while none is flagged** is where each line's scan ended, Young's account, which his document
  says was "not fully checked".
- **Collisions** among the four sprites shown, never past pixel 255, on any line, as mode 4's are measured to be; TI
  says sprites "partially or completely off the screen are also considered", which this reading takes as the lines
  outside the display.
- **The CPU's access windows** of TI's Table 2-2 are not modelled: no access is lost, as on the Sega chips (D-8).

### 11.6 Nephrite on the shared composition

`compose` gained the Genesis's variant and a `line` call that composes a span with the mix and the left column's mask,
which Endou's `Vdp` now uses as well. Nephrite's `render_mode4` is now that call with `Variant::GENESIS` and a `Vram` of
D-7's map; it is recorded in `Nephrite_Native.md` §67. The variant keeps two rules Nephrite drew that the Master
System's chips do not: the wrapped column under a fine scroll, and the right columns' lock from pixel 192 rather than
by the column counter (`Beryl_SmsVdp.md` §9.4), the second a question for the tester (§11.15).

**Unchanged, measured 2026-10-19** [M]: 200,000 random mode 4 lines drawn equal by the old and the new code before the
old was deleted, and again after the background was made to draw a column at a time, which a first measurement of the
cost asked for (§11.9); the crate's 194 tests on `test-checked`, D-7's two among them; the kit's corpus on `genesis-sample.txt`,
the 64 image reports of the base's library and this one's identical, digests included; the open Sega CD BIOS
byte-identical; WiseMan's Nephrite classes with the corpus and the games, **56 of 56** (`NephriteTests` 22, its debugger's, firmware's, media's, players' and start's 34). P4 of the plan is retired as predicted.

### 11.7 The test programs and the synthetic cartridge

`programs/tms.s`, an SG-1000 program: a Graphics II picture of all three pattern and colour tables with six sprites on
one line (two overlapping, one moved by its early clock bit, a fifth and a sixth), then Graphics I from frame 30, Text
from 60 and Multicolor from 90; the frame interrupt keeps each of the first 16 frames' status, and the program reads a
byte written in the 16K mode back in the 4K mode. `programs::an_sg_1000_program_s_pictures_status_and_4k_reads_are_the_tms9918a_s`
holds its four pictures' digests, the status `$E4` (the fifth sprite, number 4, and the collision) every frame after
the first, the 4K read, and the Mark III's picture and status. `examples/referee.rs` writes `.sg` images too, and
`examples/screen.rs` takes `--system`.

WiseMan's `SyntheticSmsRom.TmsPicture()` is an SG-1000 cartridge in Graphics I; `EndouTests.The_tms9918a_draws_an_sg_1000_program_s_picture`
holds TI's colours 6 and 12 through the v1 adapter, and the kit's case runs on it.

### 11.8 The state, version 4

`ENDO` version 4: version 3's layout with the VDP's block two bytes longer (whether the line's sprites are the
TMS9918's, and the fifth sprite's number; `Beryl_SmsVdp.md` §6): 24,881 bytes for a plain `sms` board, pinned by
`the_version_4_layout_is_pinned`. Version 3 reached no player and is refused.

### 11.9 The gate (`Endou_Plan.md` §7, stage 4), measured 2026-10-19

| Gate item | Result [M] |
|---|---|
| TMS9918 test programs where fetched | none exists in a form that runs on an SG-1000 (`Endou_References.md` §2); `tms.s` was written for the purpose, and the crate's tests hold each mode's rules |
| `SyntheticSmsRom` cases | `EndouTests` **14 of 14**, the new picture case among them; with `SystemTableParity`, the registration and the debugger's parity, and the Nephrite classes, 83 of 83 |
| The kit | **every case it runs passes** (C1-C15, C17-C24): in corpus form on the library's 54 SG-1000 images, 600 frames, on the SG-1000 (54 of 54) and on the Mark III (54 of 54); on the 12 program images, 300 frames; and on WiseMan's synthetic cartridges, `tms.sg` among them |
| Pictures against the references on the library's SG-1000 games | §11.10 |
| Nephrite unchanged | §11.6 |
| The crates' tests | `beryl-sms-vdp` **37 of 37** on `test-checked`; `endou` **54 of 54** with the corpus (ZEXDOC, ZEXALL and FluBBa's test run); Nephrite 194 of 194 |
| Seeded faults | §11.12: 16, every one caught, three only after a test was added |
| The cost | the desktop's mean a frame, release, the median of three differences between runs of 3,600 and 600 frames, over 3,000, the desktop lightly loaded: Sonic the Hedgehog 0.206 ms, Space Harrier 0.190, OutRun 0.223, against 0.203, 0.202 and 0.221 for the base's build measured the same way within the hour; on the SG-1000 Girl's Garden 0.178 and Zaxxon 0.181, and Girl's Garden on the Mark III 0.188. The first build of the shared background, choosing its column a pixel at a time, cost 0.231-0.269 ms on the three games; it now draws a column at a time (§11.6) |

Toolchains: rustc 1.99.0, .NET 10.0.112, GHDL 6.0.0, on the desktop (Ryzen 7 7700X).

### 11.10 Pictures against the references

Each image run to frames 300 and 600 with no input on Endou and on the five references as black boxes through the
probe (`.sg` routed as the SG-1000; for the Mark III, Genesis Plus GX's `mark-III`, Gearsystem's "Master System / Mark
III", SMS Plus GX's and PicoDrive's "Master System", BlastEm's `sms1`), every other option at its default, and the
pictures compared up to a colour map, both ways (`compare.py` in the step's scratch; BlastEm's 256 by 224 aligned at
row 16). The library is read in place; no picture is committed.

**`tms.s`** at frames 20, 45, 75 and 105 (Graphics II, Graphics I, Text, Multicolor): equal, 0 pixels, to Genesis Plus GX
and PicoDrive in all four, to Gearsystem and SMS Plus GX in all but Text (66 and 19,604 pixels), and to BlastEm in
Text alone; the same on the Mark III's model. The status bytes equal Genesis Plus GX's, Gearsystem's and PicoDrive's
from the second frame on (`$E4`); the first read differs among them (`$80` Endou and Gearsystem, `$C0` Genesis Plus GX,
`$C4` PicoDrive). The 4K read is D-12's.

**The library's 54 SG-1000 images**, on the SG-1000 [M]:

| | Frame 300 | Frame 600 |
|---|---:|---:|
| Equal to at least one of Genesis Plus GX, Gearsystem, SMS Plus GX and PicoDrive | 49 | 45 |
| Equal to at least three | 46 | 44 |
| Equal to all four | 38 | 38 |
| Equal to Genesis Plus GX / Gearsystem / SMS Plus GX / PicoDrive / BlastEm | 43 / 48 / 44 / 47 / 35 | 43 / 44 / 41 / 44 / 30 |

Of the images equal to none: *Gulkave* at 300 and *Soukoban* at 600 are a single colour on Endou and on every reference;
*Terebi Oekaki* waits on its drawing tablet, which no setting gives; *Drol*, *Hustle Chummy*, *Pachinko 2*, *Golgo 13*,
*Flicky* and *Safari Race* differ from every reference by about as much as the references differ from each other
(*Drol*'s radar box, 50-72 pixels against 52-70 between the references). *Champion Pro Wrestling* at 600 is the one
image where three references (Genesis Plus GX, Gearsystem, PicoDrive, within 217-434 pixels of each other) agree against
Endou (2,632-3,066): the pictures show the wrestlers in other holds, a different course of play, and Endou's is within
434 pixels of SMS Plus GX's; it is drawn alike, and which course is the console's is the goldens' question (G5). On the Mark III's model the counts are the same within one image a row.

### 11.11 SMS_MiSTer, under GHDL

Approved 2026-10-19 for D-6, D-7, D-8 and Space Harrier's opening, run as a black box. Its VHDL imports, but its console
cannot be elaborated: `system`, `vdp` and `t80s` each need Intel's `altera_mf` library, and the PSG is Verilog
(`Endou_References.md` §4a). Making stand-ins would mean reading the RTL outside a dispute step, so it was not done, and
those disputes, and §10.10's Space Harrier, stay as they were.

### 11.12 Seeded faults (2026-10-19)

Each made in the source, the crates' tests run (`beryl-sms-vdp`'s, Endou's with the corpus but ZEXDOC and ZEXALL, and
Nephrite's for F9), the file restored and compared byte for byte (`seed.py` in the step's scratch):

| # | Fault | Caught by |
|---|---|---|
| F1 | Text shows a pattern's low six bits (Young's wording, against TI's) | `tms`'s Text test, `tms.s`'s digests |
| F2 | Graphics II unmasked on the TMS9918A | **nothing at first**: the test's TMS9918A case set every mask bit, and the 315-5124's case had the patterns of both thirds alike. It now clears the masks on the TMS9918A with the thirds' patterns apart, and catches it |
| F3 | the fifth sprite flagged with the frame flag set | the fifth sprite's test |
| F4 | eight sprites a line in the TMS9918's modes | the fifth sprite's and the TMS state's tests, `tms.s` |
| F5 | a colour 0 sprite not colliding | `tms`'s sprite test, the TMS state's test |
| F6 | the early clock bit ignored | `tms`'s sprite test, `tms.s` |
| F7 | the TMS9918A's register number from bits 3-0 | the register write's test |
| F8 | the boosted blue on the 315-5246 | `the_315_5124_boosts_only_its_first_blue_level`, the mode 4 picture program's digest |
| F9 | the Genesis's fine scroll blanked | the crate's Genesis test, and **none of Nephrite's** (§67 of its record) |
| F10 | the 4K mode not applied to a CPU write | **nothing at first**: the test wrote in the 16K mode and read in the 4K. It writes in the 4K mode now, and catches it |
| F11 | a state's sprites remade by mode 4's rules | the TMS state's test |
| F12 | an SG-1000 cartridge never on the Mark III | the model setting's test |
| F13 | Multicolor's nibbles swapped | `tms`'s Multicolor test, `tms.s` |
| F14 | a blanked TMS9918 mode showing CRAM's backdrop | the codes and colours test |
| F15 | the SG-1000's status bits 4-0 undriven | `tms.s`'s status |
| F16 | the Mark III's TMS9918 colours from the TMS9918A's table | **nothing at first**: no test ran an SG-1000 program on the Mark III's colours. The program's test now holds the Mark III's digest, and catches it |

### 11.13 Open

- **The invalid text mode** is not modelled (§11.3).
- **D-6, D-7 and D-8** stay as stage 3 left them; SMS_MiSTer cannot be elaborated without `altera_mf` (§11.11).
- **Space Harrier's late start** (§10.10) is not found.
- **The Genesis's right-column lock** is Nephrite's rule, unmeasured (§11.6).
- **A fifth sprite on the first status read**: the references disagree with each other (§11.10).
- **Champion Pro Wrestling's course of play** at frame 600 differs from three references' (§11.10).

### 11.14 What was read for this stage

No emulator source, no SMS_MiSTer RTL (only GHDL's messages and the entity names it lists), no firmware or firmware
listing. Read: `Endou_Plan.md`, `Endou_Native.md`, `Endou_References.md`, `Endou_Disputes.md`; `Beryl_SmsVdp.md` and the
crate; Nephrite's `render.rs` and `pictures.rs`, its D-7 and `Nephrite_Native.md` §15 and §38; `EmuSen_CoreAPI.md`
§6.40; TI's TMS9918A data manual §2.1-§2.4.5 as page images; Young's `tms9918a.txt`; MacDonald's `sc3000h` and `msvdp`
§4-§7, §10, §13-§15; Guttag's symposium draft; SMS Power!'s "Palette", "SMS1/SMS2 VDPs", "GG VDP", "VDP Registers",
"Sprites", "I/O Port Map - SG-1000".

### 11.15 Questions before stage 5

1. **The Genesis's right-column lock** (`Beryl_SmsVdp.md` §9.4): Nephrite locks the right columns' vertical scroll from
   pixel 192, the Master System's chips by the column counter. The move kept Nephrite's rule so that nothing changed.
   Should a Nephrite dispute step ask the board which the 315-5313 does? Recommended: yes, when Nephrite is next worked
   on; a game that needs it is unlikely, mode 4 on a Genesis being the Master System's games.
2. **SMS_MiSTer's `altera_mf`**: Intel's simulation library comes with Quartus, which needs a licence agreement and an
   account. Three ways remain: install Quartus's free edition for its `altera_mf` (the tester's to allow); write
   stand-ins for its memories in a logged dispute step, reading only the RTL lines that instantiate them; or leave
   SMS_MiSTer aside and keep D-6, D-7 and D-8 argued. Recommended: the dispute step, its files and lines logged, if the
   tester wants the referee; else leave them argued.
3. **The SG-1000's default model** stays the SG-1000, and the Mark III is offered: is the setting's wording right for the
   tester's front end? Recommended: as built.
4. **Stage 5** (sound) as the plan has it, the PSG's wiring in both variants and the mix. Recommended: the go.

*Decided 2026-10-19 by the tester: each of the four as recommended.* The go for stage 5 came with the decisions; it
covers stage 5 alone. No real hardware is used for any of them (`Endou_Plan.md` §10 Q11), and MAME is not used.

1. **The Genesis's right-column lock:** a Nephrite dispute step asks Nuked-MD's board whether the 315-5313 locks the
   right columns by the column counter or by the pixel, the next time Nephrite is worked on. It is not done at
   Endou's stages; it is recorded as pending in `Nephrite_Native.md` §67 and `Beryl_SmsVdp.md` §9.4.
2. **SMS_MiSTer's `altera_mf`:** a logged dispute step may write stand-ins for the memory blocks of `altera_mf` that
   SMS_MiSTer uses, so that the core runs under GHDL as a black-box referee for D-6, D-7, D-8 and Space Harrier's
   opening. The step lists every RTL file it reads and reads only what the stand-ins need: the entity interfaces of
   the `altera_mf` components the core instantiates. The PSG, `jt89`, is Verilog, which GHDL cannot run, so sound stays
   out of the referee unless a VHDL path exists. The step is optional within stage 5: it is taken only if it fits
   beside the sound work and does not hold it up, else left for a later stage.
3. **The SG-1000's default model** stays the SG-1000, with the Mark III offered, as built.
4. **The SG-1000 gets its own shelf.** Its console is `"SG-1000"` in `Platform/systems/data/sg1000.json`, apart from
   `"Master System"`, so the catalog gives it a row of its own once it is offered; stage 7 holds this with a test that
   `sms` and `sg1000` land on two shelves (`Endou_Plan.md` §7).

---

## 12. Stage 5: the sound (2026-10-19)

### 12.1 In one paragraph

The Master System and the SG-1000 now sound. Beryl's `beryl-sn76489`, clocked since stage 2 and its output discarded,
is mixed and output: `src/sound.rs` brings the chip to the bus's clock at each write and at the frame's end, places
every change of its output as a step at the Z80 clock it happened on, and resamples the steps to 48 kHz with
`emusen-native`'s band-limited step synthesiser, the one Nephrite mixes through, its two sides kept apart so that the
Game Gear's stereo register needs only its port at J2. Sega's PSG sounds on the Mark III and both Master Systems and
the discrete chip on the SG-1000 (D-0), each console's periodic noise measured from the output. Port `$F2` and the FM
unit's other ports are nothing on these consoles, and the FM unit's detection finds none on any of them (D-13). No
analogue stage is modelled, since no recording of a Master System's output is in the corpus: a named loss. The state
is version 5. The samples queue for the host through the v1 drain, peek and limit calls, and a sounding frame
allocates nothing.

### 12.2 The wiring

| Rule | Source |
|---|---|
| The PSG's internal clock is the Z80's over 16, 223,722 Hz NTSC | [R SMS Power!'s "SN76489"; `Beryl_SN76489.md` §2]; Sega's Mark III manual, "Tone generator frequency", for the counter's rule |
| A write through any port `$40`-`$7F` reaches the chip at the I/O cycle's clock: the chip is first run to its last clock at or before it, and the level the write makes is the output from that clock, between the chip's clocks | [R `smstech` §3]; the step's place [A], held by `a_write_s_step_is_at_its_own_clock_within_the_psg_s` |
| Each change of the chip's output is a step at the Z80 clock of the PSG clock that made it, and the synthesiser runs at the Z80's clock, `num / 15` Hz of the master clock, into 48,000 Hz; samples are taken up to the bus's clock at each frame's end, so each frame's count follows the frame's own end and a second's frames make a second's samples to the sample | `EmuSen_NativeCores.md` §12.6 for the synthesiser |
| `Variant::SEGA` on the Mark III, the Master System and the Master System II, `Variant::DISCRETE` on the SG-1000; on the Mark III an SG-1000 cartridge sounds through Sega's | D-0; `model.rs`, the one owner |
| Both sides are the chip's two sums: alike until a console writes the stereo register, which none does before J2 | [R SMS Power!'s "Game Gear Stereo"] |

The synthesiser is shared, not copied: Nephrite's `genesis/sound.rs` and Endou's `sound.rs` each own a `Steps` for the
same chip, and what differs is what lies after it (Nephrite's measured circuits and its YM2612; nothing yet here).

### 12.3 The level and the analogue stage

**The level** [C]: the chip's 8,192 for a channel at full volume is 4,096 in the output, so the four channels at full
volume fill half of a 16-bit sample and leave room for the YM2413 at stage 9, whose level against the PSG's is that
stage's measurement. A full-volume tone's fundamental is then 2,608; the five references put it at 1,470 (BlastEm),
2,672 (Genesis Plus GX), 2,933 (PicoDrive), 3,860 (SMS Plus GX) and 8,793 (Gearsystem), their median 2,933 [M §12.6].
The absolute level is no property of the console a player can compare, its line output being an analogue level.

**The analogue stage is not modelled**: a named loss. `Endou_Plan.md` §5.4 takes the consoles' filters and levels from
recordings where they exist; none is in the corpus (Nephrite's MDFourier recordings are the Genesis's and the Sega
CD's), and no console is run for the project (`Endou_Plan.md` §10 Q11). So the output is the chip's unipolar levels,
band-limited: no low-pass beyond the synthesiser's 21.6 kHz and no coupling capacitor, so a held level, which sample
playback writes, stays as the chip holds it. Of the references, PicoDrive and SMS Plus GX keep the held levels too;
Genesis Plus GX, Gearsystem and BlastEm remove them with a high-pass of their own [M §12.6]. Whether a recording of a
Master System's output can be found and fetched is a question for the tester (§12.12).

### 12.4 Port `$F2` with no FM unit

On the Mark III without the FM Sound Unit and on the export consoles, ports `$F0`-`$F2` are what A7, A6 and A0 make
them (§9.3): a write does nothing, and a read is the I/O chip's port `$DC`, or, with the I/O chip switched off by port
`$3E`, the console's undriven bus [R `smstech` §4, §7]. The FM unit's detection (SMS Power!'s "FM Chip Detection", the
routine taken from Phantasy Star) switches the I/O chip off, writes bit 0 to `$F2` and reads it back seven times;
`programs/fm.s` runs it and finds no FM unit on the Mark III and both Master Systems, each read `$FF` on the Master
System II and `$F2`, the instruction's last byte, on the others, and its tone still sounding after `$F2`, `$F0` and
`$F1` are written as an FM game writes them. The documents differ on what such a read returns: D-13. The Japanese
Master System and the Mark III with the FM unit, whose `$F2` is a register, come with the YM2413 at stage 9.

### 12.5 No allocation on the audio path

The synthesiser's finished samples and the queue keep their storage from frame to frame. `tests/alloc.rs`, a test
binary of its own with a counting allocator, runs the voices program for 720 frames on the Master System II, the
SG-1000 and the Mark III, draining each frame as a host does, after four frames of warming: **no allocation** [M].
Seeded fault F6 (§12.9), a frame's samples gathered in a vector of their own, fails it.

### 12.6 Against the references

**The voices.** `programs/voices.s`, assembled as `voices.sms` and `voices-sg.sg`, writes the PSG from a table, each
voice held for a second by the frame interrupt: tone 0 at `$0FE` at full volume and 8 dB down, tone 1 at `$020`,
periodic noise at rates 0 and 2 and from tone 2's `$040`, white noise at rates 0 and 1, a tone held high stepped
through the sixteen attenuations, and the four channels together. Endou (`examples/screen.rs --wav`) and the five
references through the probe (`--wav`), each recording aligned at its first sound, the second of each voice measured
from 0.2 s for 0.6 s under a Hann window (`audiotool` in the step's scratch) [M 2026-10-19]:

| | Endou | Genesis Plus GX | PicoDrive | BlastEm | Gearsystem | SMS Plus GX |
|---|---|---|---|---|---|---|
| Tone 0, `$0FE`: frequency, fundamental | 440.40 Hz, 2,608 | 440.40, 2,672 | 440.40, 2,933 | 440.40, 1,470 | 440.60, 8,793 | 440.40, 3,860 |
| Attenuation 4 against 0 | 8.00 dB | 8.01 | 8.00 | 8.07 | 8.61 | 8.01 |
| Tone 1, `$020` | 3,495.5 Hz | 3,495.5 | 3,496.0 | 3,495.5 | 3,497.5 | **3,390**: a period rounded to its output samples |
| `voices.sms`, periodic noise rate 0: 436.9 Hz (16 bits) against 466.1 Hz (15 bits) | 508 / 0 | 521 / 0 | 572 / 0 | 286 / 0 | 1,689 / 0 | 752 / 1 |
| `voices-sg.sg`, the same on an SG-1000 | **0 / 542** | **0 / 555** | 572 / 0 | 286 / 0 | **4 / 1,805** | 752 / 1 |
| Rate 2 and from tone 2 (109.2 Hz against 116.5) | as rate 0 on each system | as rate 0 | 16 bits on both | 16 bits on both | as rate 0 | 16 bits on both; from tone 2 at half the others' level |
| White noise, rate 0: its null at 6,991 Hz | 0.0 | 0.0 | 0.8 | 0.2 | 0.3 | 9.8, no null |
| The held tone stepped through the attenuations | levels held, 4,096 to 163 by 2 dB, then 0 | removed by a high-pass | held, 2,112 to 49 | removed | removed | held, 2,780 to 64 |

**D-0, by the references** [M]: Genesis Plus GX and Gearsystem give an SG-1000 cartridge the 15-bit register and a
Master System's the 16-bit one, as Endou does; PicoDrive, BlastEm and SMS Plus GX give every system the 16-bit one,
modelling no discrete chip. Where a reference models the difference it agrees with D-0. `programs::each_console_s_psg_sounds_its_own_periodic_noise`
pins the rule on the four consoles from the output.

**The fetched sound test programs**, played by pad presses (Heliophobe and Maxim's *SMS Sound Test* 1.2: the cursor to
tone 0's volume, its volume to 0, its pitch moved, the noise's volume and type; JoppyFurr's *SN76489 TestRom* 1.00 for
the Master System and for the SG-1000, NTSC: the cursor to the keyboard, three keys held in turn): at six moments of each
run the strongest frequency between 50 Hz and 4 kHz is the same on Endou, Genesis Plus GX, PicoDrive, Gearsystem and
SMS Plus GX to the half-hertz step of the search, 109.5 Hz, 112 Hz, 155.5 Hz, 164.5-165 Hz, 174.5 Hz and 207.5 Hz, and
its level stands to each core's own full-volume tone as the program set it. BlastEm took no pad press from the probe,
as at stage 3 (D-9's note), and stayed silent in all three.

**Library games** (read in place, 900 frames, no input on the Master System's; the SG-1000's with button 1 pressed at
frames 120, 240 and 360 and right held from 480; the references with their FM units off, Genesis Plus GX's, Gearsystem's
and SMS Plus GX's being on by default): each recording's band envelopes (twenty bands from 100 Hz to 10 kHz, a sixtieth
of a second a window) from 2 s to 14 s, correlated with another's at the best lag within half a second [M]:

| Game | Endou's audio digest (FNV-1a over the samples), samples | Endou against Genesis Plus GX / PicoDrive / BlastEm / Gearsystem / SMS Plus GX | Genesis Plus GX against PicoDrive / Gearsystem / SMS Plus GX |
|---|---|---|---|
| Sonic the Hedgehog | `84266002961E9FF1`, 720,929 | 0.983 / 0.973 / 0.980 / 0.976 / 0.978 | 0.971 / 0.973 / 0.971 |
| Alex Kidd in Miracle World | `34103FC0494AB8F1` | 0.966 / 0.956 / 0.977 / 0.950 / 0.960 | 0.953 / 0.943 / 0.954 |
| Phantasy Star | `54D60BDB25A00099` | 0.929 / 0.889 / 0.949 / 0.919 / 0.888 | 0.915 / 0.880 / 0.913 |
| Fantasy Zone | `2868F5FF1A569EA9` | 0.963 / 0.958 / 0.979 / 0.949 / 0.950 | 0.958 / 0.938 / 0.954 |
| Golden Axe | `281A40695DA6B9CD` | 0.973 / 0.964 / 0.980 / 0.972 / 0.957 | 0.970 / 0.954 / 0.965 |
| OutRun | `5E5DFA61070344B9` | 0.866 / 0.822 / silent / 0.878 / 0.818 | 0.898 / 0.855 / 0.894 |
| Micro Machines (E), Europe | `33B059B58F9DF21D`, 869,190 | 0.958 / 0.942 / no run / 0.984 / 0.943 | 0.945 / 0.956 / 0.947 |
| Space Harrier | `58A5E079EDB12A29` | 0.70-0.72 at the half second's edge / **0.983 to BlastEm at lag 0** | 0.910 / 0.914 / 0.909 |
| Girl's Garden (SG-1000) | `12973C7A9A747509` | 0.875 / 0.902 / silent / 0.866 / 0.893 | 0.855 / 0.881 / 0.859 |
| Flicky (SG-1000) | `5DEE3B43878EEF69` | 0.826 / 0.827 / silent / 0.878 / 0.841 | 0.895 / 0.828 / 0.875 |
| Zaxxon (SG-1000) | `22B281F3BDFD07A1` | 0.933 / 0.923 / silent / 0.928 / 0.917 | 0.918 / 0.913 / 0.907 |

Endou's agreement with each reference lies within the references' agreement with each other on every game but Space
Harrier. A first run with the references' FM units at their defaults put OutRun's Endou at 0.50-0.53 against the three
that played its FM music, 0.82 against PicoDrive, whose unit is off by default; with the units off it is as above.
**Space Harrier** is §10.10's late start heard: over 5-13 s Genesis Plus GX, PicoDrive, Gearsystem and SMS Plus GX
best match Endou 1.3-1.9 s apart (lags 77-113 windows) and at 0.80, while BlastEm, which was not among the pictures of
§10.8, follows Endou at lag 0 at 0.98 [M]. Two black boxes now take each course; the question stays G5's (§12.11).

### 12.7 The gate (`Endou_Plan.md` §7, stage 5), measured 2026-10-19

| Gate item | Result [M] |
|---|---|
| The fetched sound test programs | *SMS Sound Test* 1.2 and both *SN76489 TestRom*s: Endou's frequencies at six moments equal four references' (§12.6); no program self-reports. *PFR Detect*, which `Endou_References.md` §2 listed as FM detection, is a test of the paging registers (its source's title, read for this stage); the FM detection is held by `fm.s` |
| `SyntheticSmsRom` cases | `EndouTests` **15 of 15**: the new `The_psg_sounds_a_synthetic_program_s_tone` hears tone 0 through the v1 adapter on both systems, its fundamental within 1% of 2,607.6 and 48,062 samples in 60 frames, and the kit's case runs on the two tone cartridges too |
| Audio against black-box referees on library games | §12.6, eleven games and the voices on both systems |
| The kit | in corpus form, 600 frames, four at a time: **30 of 30 images pass** and the core's cases pass (C1-C15, C17-C24 where they run; C6, C7, C8 and C12 with the audio in their digests): the 15 program images, the three sound test programs, and eight Master System and three SG-1000 games of the tester's, read in place |
| Seeded faults | §12.9: 12, every one caught, two only after a test was added |
| No allocation per frame on the audio path | §12.5 |
| The crate's tests | **65 of 65** in the library and `tests/alloc.rs`, on release and on `test-checked` with the corpus (ZEXDOC, ZEXALL and FluBBa's test run) |
| Pictures | unchanged: at frames 600 and 3,600 the frame digests of Sonic the Hedgehog, Space Harrier, OutRun and Girl's Garden equal the base's (§12.8) |
| The cost | §12.8 |

Toolchains: rustc 1.99.0, .NET 10.0.112, on the desktop (Ryzen 7 7700X).

### 12.8 The cost

The desktop's mean a frame, release, `examples/screen.rs` draining the sound each frame, the median of three
differences between runs of 3,600 and 600 frames over 3,000, the base (WiseMan `67944957`, its silence drained) and this
stage measured alternately within minutes, load about 1.7 [M]: Sonic the Hedgehog 0.204 → **0.213 ms**, Space Harrier
0.189 → **0.203**, OutRun 0.219 → **0.232**, Girl's Garden (SG-1000) 0.175 → **0.180**; each range within 0.004 ms.
The sound costs 0.005-0.014 ms a frame. P1's budget is 0.6 ms without the OPLL. The frame digests at 600 and 3,600
frames were equal between the two builds on all four.

### 12.9 Seeded faults (2026-10-19)

Each made in the source, the crate's tests run on release (the library's and `tests/alloc.rs`), the file restored and
compared byte for byte (`seed.py` in the step's scratch):

| # | Fault | Caught by |
|---|---|---|
| F1 | the PSG clocked at the Z80's over 8 | the periodic noise program, the bus's cycle test, the allocation test |
| F2 | a write's step placed at the PSG's last clock, not the write's | **nothing at first**: the write's test wrote two clocks past a PSG clock, a step a fortieth of a sample late. `a_write_s_step_is_at_its_own_clock_within_the_psg_s` was added: a write 15 clocks later rises a fifth of a sample later, to 5%; it catches it |
| F3 | the SG-1000 given Sega's PSG | the console's PSG test, the periodic noise program |
| F4 | the synthesiser's pending differences not loaded with a state | `a_state_continues_the_sound` |
| F5 | the two sides swapped as the chip runs | **nothing at first**: the Game Gear's test held both channels, so nothing changed as the chip ran. It now sounds tone 0 on the right alone, and catches it |
| F6 | a frame's samples gathered in a vector of their own | `tests/alloc.rs` |
| F7 | the synthesiser given the master clock over 16 | the second's samples (machine and v1), the noise and FM programs |
| F8 | the chip's levels unscaled | the noise program's tone level, the four channels' test |
| F9 | writes to `$C0`-`$FF` reaching the PSG | `fm.s`'s test |
| F10 | the PSG's clock count saved as 0 | `a_state_continues_the_sound` |
| F11 | the stereo register's write lost | the Game Gear's test |
| F12 | the PSG not brought up at the frame's end | the bus's cycle test |

### 12.10 The state, version 5

`ENDO` version 5: version 4's fields, the PSG's block followed by its clock count and the step synthesiser's held
levels, sums and 64 pending differences a side, 1,056 bytes more: 25,937 bytes for a plain `sms` board, pinned by
`the_version_5_layout_is_pinned`. A state continues the sound exactly (`a_state_continues_the_sound`; the kit's C7 and
C8 over the audio). Version 4 reached no player and is refused. The queue of samples not yet drained is not part of a
state, as Nephrite's is not.

### 12.11 Open

- **The analogue stage** is not modelled (§12.3), a named loss until a recording is found.
- **The discrete chip's half-period 0** and its power-on registers stay as `Beryl_SN76489.md` §4 leaves them; no
  SG-1000 recording is in the corpus.
- **Space Harrier's course** (§10.10, §12.6): BlastEm's audio follows Endou's, the other four references another.
- **SMS_MiSTer** was not run (§12.13).
- **D-13**, the read of `$F2` with no FM unit, is argued.

### 12.12 What was read for this stage

No emulator source, no firmware or firmware listing; of SMS_MiSTer's RTL, only the two lines GHDL printed in its error
messages (§12.13). Read: `Endou_Plan.md`, `Endou_Native.md`, `Endou_References.md`, `Endou_Disputes.md`;
`Beryl_SN76489.md` and the crate; Nephrite's `genesis/sound.rs` and `Nephrite_Native.md` §18-§19 for how it mixes;
`emusen-native`'s `steps.rs` and `samples.rs`; the kit's C6-C8 and C12; MacDonald's `smstech` §1, §3, §4 and §7; Sega's
Mark III manual, "Programmable sound generator" and "Memory enables"; SMS Power!'s "Audio Control Port", "FM Chip
Detection", "Sound", "Port $3E", "I/O Port Map - SMS", "Game Gear Stereo" and "YM2413"; JoppyFurr's *SN76489 TestRom*
`main.c`, for its controls; the *SMS Sound Test*'s readme; *PFR Detect*'s source's opening and strings.

### 12.13 SMS_MiSTer's dispute step, not taken

*Decided 2026-10-19* (§11.15): a logged dispute step may write stand-ins for `altera_mf`'s memory blocks so that
SMS_MiSTer runs under GHDL for D-6, D-7, D-8 and Space Harrier's opening, optional within this stage. It was weighed
and left for a later stage. Analysing the RTL with GHDL (the messages only, nothing opened) shows what the step needs
beyond the stand-ins [M]: `dpram.vhd`, line 4, is the file naming `altera_mf` (the console's other memories may
name it too, which only their files would show); and the console's top entity `system` uses `work.jt89`
(`system.vhd`, line 6), the PSG, whose `rtl/jt89/` holds one `.vhd` beside six Verilog files, so that `system` cannot
be elaborated without a stand-in for the PSG as well, which the decision does not cover. The VDP's entity alone needs
only the memories, but a test bench for it would drive the VDP's own ports, which is reading more than the top
level's port list. And a program's verdict leaves the console only as its picture, so the bench must decode the video
output. Those two lines are the only RTL text seen, printed by GHDL; they are logged here as the rule asks. The
choices are the tester's (§12.14).

### 12.14 Questions before stage 6

1. **A recording of a Master System's output.** The analogue stage is a named loss for want of one (§12.3). May a
   search be made for a published recording of a Master System's or Mark III's audio output (a test tone or a
   test program's sweep, as MDFourier's are for the Genesis), and any found be fetched and recorded in
   `Endou_References.md`? Recommended: yes, the search and fetch as a step of their own; a filter fitted to it would
   be built as Nephrite's circuits were.
2. **SMS_MiSTer's step** (§12.13). Running `system` needs a stand-in for the Verilog PSG `jt89` beside `altera_mf`'s
   memories, reading its instantiation as the memories' are read; or the step runs the `vdp` entity alone, reading its
   port list, with a test bench that plays the CPU's accesses. Either decodes the picture to read a verdict.
   Recommended: the `vdp` entity alone for D-8 (the access slots, the open dispute most worth a referee), with the
   memories' stand-ins, its port list and every line read logged; `system` with a PSG stand-in only if Space Harrier's
   course is to be refereed.
3. **Stage 6** (firmware) as the plan has it: `Endou_OpenBoot.md` first, then the open boot program. The tester's own
   Master System BIOS images are now in `~/Documents/BIOS/SMS/` (Europe v2.0, Japan v2.1, USA/Europe v1.3, 8 KiB each);
   nothing of them was read, hashed or run at this stage, and how they serve as the player's own image under
   `EmuSen_Firmware.md` §0 is stage 6's question for the tester. Recommended: the go for stage 6, opened by the page.

*Decided 2026-10-19 by the tester: yes to all four.* The go for stage 6 came with the decisions; it covers stage 6
alone. No real hardware is used for any of them (`Endou_Plan.md` §10 Q11), and MAME is not used.

1. **A published recording of a Master System's audio output may be searched for and fetched**, so that the console's
   analogue output filter can be fitted as Nephrite's was from MDFourier's recordings. Its provenance and licence are
   recorded in `Endou_References.md`, and the filter is fitted only from it. If no suitable recording is found, the
   search and its negative result are recorded and the filter stays a named loss.
2. **SMS_MiSTer's `vdp` entity may be run alone under GHDL**, with stand-ins for its `altera_mf` memory blocks, in a
   logged dispute step for D-8 (the access slots). The step may read the `vdp` entity's port list and the interfaces of
   the `altera_mf` components it uses, and lists every file it reads; nothing else of the RTL is read. It may run
   beside the boot work, and its outcome is recorded in D-8.
3. **The go for stage 6**: the open boot program, opened by its design page `Endou_OpenBoot.md`, written before any of
   its code.
4. **The player's own BIOS is an opt-in accuracy upgrade per console model**, the open boot program being the default.
   A player who has a console's BIOS chooses it for that model in the settings; without the choice, or without the
   file, the open boot program runs. The tester's own images (`~/Documents/BIOS/SMS/`: Europe v2.0, Japan v2.1,
   USA/Europe v1.3, 8 KiB each) are run only by the tester; the path is built and tested with a synthetic stand-in
   image that the tests make, and no real BIOS is read, hashed or run for the project.

---

## 13. Stage 6: the firmware and the open boot program (2026-10-19)

### 13.1 In one paragraph

The Master System and the Master System II now start through a BIOS, as the consoles do. `Endou_OpenBoot.md`, the
design page, was written and committed before any of the code. Endou's open boot program is Z80 assembly in
`src/boot/`, assembled at compile time by `build.rs` with `beryl-z80`'s assembler into an 8 KiB image, its letters
`beryl-font`'s, Nephrite's font moved into a Beryl crate of its own so that one crate owns the glyphs. The program sets
the console up as Sega's Mark III manual says the BIOS does, shows its own screen with no logo, runs a program from
`$C700` that tries the card, the cartridge and the expansion slot in that order, makes an export console's header check
and refuses in words, and hands the game over on the VDP's last line with the slot's port `$3E` value at `$C000`. The bus
gains the BIOS's slot. The player's own BIOS is an opt-in per console, the open program the default (decided
2026-10-19); the Mark III and the SG-1000 have no BIOS and start their cartridges from reset, as before. `sms` starts
empty. The state is version 6. The checksum's range was measured on the library: Sega's cartridges leave the whole
header out of the sum, not only the checksum's two bytes as the manual's sentence reads, and *Micro Machines* states the
sum read through its own board.

### 13.2 The decisions of 2026-10-19, as built

| Decision | Built |
|---|---|
| A recording of a Master System's audio output may be searched for and fetched | searched; none exists in a published form (§13.11); the analogue stage stays a named loss |
| SMS_MiSTer's `vdp` entity alone under GHDL, a logged dispute step for D-8 | taken; GHDL cannot analyse the `vdp` entity's files in any one VHDL standard (§13.11, D-8) |
| The go for stage 6, opened by `Endou_OpenBoot.md` | the page committed first (`17c5fa50`), then the program |
| The player's own BIOS as an opt-in per console model, the open program the default | settings `bios_sms1` and `bios_sms2`, files 1 and 2 (§13.5) |

### 13.3 The bus and the boot

`src/boot.rs` owns what a console runs before its game: `Boot::None` (the cartridge from reset, port `$3E` at `$AB`, the
Mark III's and the SG-1000's way and every machine stages 1–5 built), `Boot::Open` or `Boot::Player`; `model.rs` says
which consoles have a BIOS (the Master System and the Master System II; the Mark III not, D-14). `bus.rs` gains the
BIOS's slot by `Endou_OpenBoot.md` §2: port `$3E` is `$E3` at reset with a BIOS, the BIOS alone in the slot area; the
8 KiB image mirrors through `$0000`–`$BFFF`; with the cartridge on as well, the cartridge's byte on the first Master
System and the two ANDed on the second [R `smstech` §4]; an empty card, expansion or cartridge slot drives nothing.
`Machine::with_boot` makes a machine with a boot; `Machine::with_board`, which the test programs use, keeps none. A
`BIOS` space, read-only, shows the image; an empty cartridge slot has no `ROM` space.

**The state, version 6**: version 5's layout with the boot's byte and the BIOS's CRC-32 after the model, 25,942 bytes
for a plain `sms` board, pinned by `the_version_6_layout_is_pinned`; a state of another boot or BIOS image is foreign,
as one of another model is. The CRC is taken once, when the machine is made. Version 5 reached no player and is refused.

### 13.4 The open boot program

| | |
|---|---|
| Sources | `src/boot/rom.s` (the ROM part, `Endou_OpenBoot.md` §3.1), `src/boot/ram.s` (the RAM program, §3.2) |
| Image | 8,192 bytes: the ROM program's 800 bytes from `$0000`, `beryl-font`'s 64 tiles at `$1000` (bitplane 0), the RAM program's 669 bytes at `$1800`, `$FF` between |
| SHA-256 | `625a92bd6aa6dd5ea523cd10a2eedef2947a0bba951d0a4cf6a0bb82f694f9f7`, FNV-1a `C16C9C33CA123EA4` pinned by `the_open_boot_program_is_8_kib_and_its_bytes_are_pinned` |
| Rebuilt byte for byte [M] | the same SHA-256 from the debug and release builds in the crate's target directory and from two clean builds in target directories of their own |
| Its pace | every VDP write at least 16 cycles after the one before with the display off and 29 with it on, Sega's manual's rates, so that it would not depend on D-8's slots |
| The sum | 16-byte groups, sixteen bytes unrolled, about 23 cycles a byte |

**When the game starts** [M, `examples/bootcheck.rs`]: a 32 KiB check hands over in the 25th frame (0.42 s NTSC), a
64 KiB one in the 38th, a 128 KiB one in the 64th and a 256 KiB one in the 116th (1.9 s), always 76 to 81 Z80 clocks
into the frame, on the VDP's last line (U10). Without a BIOS a game starts at frame 0's first clock.

**The state at the hand-over** (`the_open_boot_hands_a_cartridge_over_in_the_state_sega_s_manual_gives`, on both
consoles and in PAL): port `$3E` `$AB` and `$C000` `$AB`; port `$3F` `$A5`; the VDP's registers Sega's power-on values,
the display off, VRAM and CRAM clear, no interrupt pending; RAM clear but `$C000` and the program at `$C700`; IM 1,
interrupts disabled, SP `$DFF0`, A `$AB`; slot 2 back at bank 2 where the sum paged it, RAM's `$DFFF` keeping the 2.

### 13.5 The player's own BIOS, the firmware entries and the empty console

- **Settings**: `bios_sms1` and `bios_sms2` ("Master System BIOS", "Master System II BIOS"), each "Endou's open boot
  program" (`open`, the default) or "Your own BIOS" (`player`), `effect: none` (`Endou_OpenBoot.md` §5).
- **Firmware**: file 1, `bios_sms1.sms`, "The Master System's BIOS", and file 2, `bios_sms2.sms`, "The Master System
  II's BIOS", 8,192 bytes each, `required: false`, `replacement: accuracy` with the cost of `Endou_OpenBoot.md` §8,
  no MD5 (§13.12). Core info lists both for `sms`; `firmware_for` answers both for a Master System image, none for an
  SG-1000 one. A BIOS file of another size is refused with words (`BAD_FILE`).
- **Which runs**: the player's image where the console's setting chooses it and its file was given; the open program
  otherwise, the given file unused. Machine info names file 1 or 2 as `file` or `replacement`.
- **The stand-in**: a boot ROM written for the tests and no BIOS's code (`openboot.rs`'s `stand_in`, assembled at test
  time; `SyntheticSmsRom.StandInBios` in WiseMan) marks `$C100`, copies a four-instruction hand-over to `$C700` and runs
  it. Chosen and given, it runs and the cartridge finds its mark and `$AB` at `$C000`; given and not chosen, or chosen
  and not given, the open program runs (`the_player_s_bios_runs_when_chosen_and_given_and_the_open_program_otherwise`,
  `EndouTests.The_player_s_own_bios_runs_when_its_setting_chooses_it`). No real BIOS was read, hashed or run.
- **`starts_empty`** for `sms` (core ABI 1.11): no cartridge, the region's console, the Americas for `auto`; the open
  program shows "NO GAME IS IN THE CONSOLE." on both Master Systems; a Japanese region gives the Mark III, which has no
  BIOS to start into (`an_empty_master_system_shows_the_open_program_s_words`; the kit's C22).

### 13.6 The checksum's range, and stage 1's rule corrected

Stage 1 summed every byte the size code names but the checksum's own two, reading Sega's manual's sentence so
(§2.2). Measured on the library's 305 export headers (`Endou_OpenBoot.md` §1.6): that rule gives no image's stated
checksum, the whole header left out gives 300, and the sum read through the board as the console reads it adds *Micro
Machines*. `media::checksum` now leaves the 16-byte header out (it is informational outside the open program), and
`SyntheticSmsRom.Checksum`, `SyntheticSmsRom.Banked` and the program images (`programs/images.rs`) carry such a sum,
so that an export console takes them. Recorded as D-16.

### 13.7 The gate (`Endou_Plan.md` §7, stage 6), measured 2026-10-19

| Gate item | Result [M] |
|---|---|
| The open program boots every test program to the point it reaches with no BIOS, with the documented state | **Endou's 13 Master System programs** on both export consoles: the RAM they keep their results in, VRAM, CRAM and the picture equal 120 frames after the hand-over to 120 frames with no BIOS, but for two differences the hand-over makes (`every_program_boots_to_the_point_it_reaches_with_no_bios`, §13.8). **The fetched programs**: the 13 whose header names an export console boot, 11 to the same pictures at 300 and 600 frames and the other two explained; the other 11 run by default on the Mark III or the SG-1000, which have no BIOS (§13.8). The state at the hand-over: §13.4 |
| A sample of library games, read in place | 31 export games: **29 to the same pictures** at 300 and 600 frames after the hand-over, RAM differing in 0–5 bytes but one game's 76; the other two explained (§13.8). 4 refused in words (§13.9) |
| The export check refuses a Japanese cartridge on an export model in words | "THE GAME CANNOT START." / "IT IS FOR A JAPANESE CONSOLE." on both export consoles (`the_export_check_refuses_in_words`, `EndouTests.An_export_console_refuses_a_japanese_cartridge_in_words`); also no header, another region, an unlisted size code and a wrong checksum, each in its words; a Japanese cartridge runs on the Mark III by default |
| The player's-image path, with the synthetic stand-in | §13.5, through the crate and through the v1 adapter; and the kit with the stand-in as file 2 and `bios_sms2=player` on *Sonic the Hedgehog*: every case it runs passes |
| The kit | in corpus form, 600 frames, four at a time, on 15 library images (12 export games, *Bank Panic*, a refused Brazilian image and an SG-1000 game): **15 of 15 pass**, and the core's cases, C22 "sms started empty: two machines alike over 240 frames, a state carried to a third" and C23 "0 checksum(s) over 2 entries of core info and 2 of firmware_for" among them; on `EndouTests`' synthetic images as before |
| Seeded faults, at least 6 | 13, every one caught, one only after a test was added (§13.10) |
| The image rebuilds byte for byte | §13.4 |
| The crates' tests | `endou` **77 of 77** (76 and `tests/alloc.rs`) on release, and on `test-checked` with the corpus (ZEXDOC, ZEXALL and FluBBa's test run); `beryl-font` 1 of 1; Nephrite **194 of 194** on `test-checked` and its open Sega CD BIOS byte-identical, `fb4a1877…62efcb9` |
| WiseMan | `EndouTests` **18 of 18**, three new; with `SystemTableParity`, `CoreRegistration`, `FirmwareOverview`, the firmware page's and `CoreDiscovery`'s, **48 of 48**; the Nephrite classes with the corpus and the games **58 of 58** |
| The cost | the desktop's mean a frame after the hand-over, release, the median of three differences between runs of 3,600 and 600 frames over 3,000, the base (`2b2f7989`) and this stage alternately, load about 1.2–1.6: Sonic the Hedgehog 0.220 → 0.222 ms, Space Harrier 0.201 → 0.206, OutRun 0.236 → 0.236. A first build that tested the BIOS's slot on every read cost 0.003–0.005 ms more; the slot area's read now leaves the BIOS out at once when it is off |

Toolchains: rustc 1.99.0, .NET 10.0.112, GHDL 6.0.0, on the desktop (Ryzen 7 7700X).

### 13.8 Against no BIOS

Each image run on its default console through the core ABI's create, the open program first, and again with no BIOS;
the booted run aligned at its hand-over (`examples/bootcheck.rs`). The library is read in place. [M 2026-10-19]

**Endou's programs.** Of the 13, 10 equal in every byte compared. `picture.s` and `voices.s` do not write `$C000`, which
holds the `$AB` the hand-over leaves there (documented, §1.3 of the design page). `hlatch.s` latches the H counter one
step later, `$2D` against `$2C`: its poll of the V counter meets a start 76 clocks into the line.

**The fetched test programs** (24 `.sms` files of `Endou_References.md` §2). 13 have a header naming an export console
and boot. 11 of them, the two ZEXes, FluBBa's *SMS VDP Test*, sverx's *SMS Test Suite*, *SMS Padtest*, the *SMS Sound Test*, both
*SN76489 TestRom*s, the *YM2413 TestRom*, the *FM Tool* and the *48KB Test* reach the same pictures at 300 and 600
frames; their RAM differs in the program left at `$C700` where they do not clear it. FluBBa's *Pause Test* scrolls by
RAM it never initialises: with the booted run's RAM at the hand-over given to the run with no BIOS, the two are equal.
FluBBa's *SMS Cycle Counter* counts cycles from its own start: with the hand-over's phase and R given to the run with
no BIOS, the two are equal. The other 11 (MacDonald's programs, *Checkstate*, *Color Check*, *SMS Palette*, *PFR
Detect*, Enri's two) have region code 0 or `$2` or no header, run on the Mark III or the SG-1000 by default, and so are
unchanged; on an export console the documented check would refuse them, region code 0 not being an export console's [A,
not run].

**31 export games**, chosen for what each exercises (the goldens' candidates of `Endou_Plan.md` §8, G5, and a spread
of boards and sizes): 29 to the same pictures at 300 and 600 frames after the hand-over, their RAM differing in 0 to 5
bytes, *Missile Defense 3D*'s in 76 at 600 with the same picture. *Sonic the Hedgehog 2* differs by the hand-over's
phase alone: the run with no BIOS started 79 clocks into its first frame gives the booted run's pictures. *OutRun*'s
attract mode reads the Z80's R register: the booted run with R cleared at the hand-over differs from itself at frame
600, while a fresh VDP or PSG in its place changes nothing. R counts the fetches the boot program makes, as any boot
program's does; a real BIOS's value is unknown, and the no-BIOS start's 0 is no console's after a BIOS.

| Game | Its check | Handed over in frame |
|---|---|---:|
| 32 KiB code (*Cosmic Spacehead*, *Ghost House*, *Teddy Boy*) | `$C` | 25 |
| *Great Ice Hockey* | `$E`, 64 KiB | 38 |
| 128 KiB code (*Alex Kidd in Miracle World*, *Black Belt*, *R-Type* and 9 more) | `$F` | 63–64 |
| 256 KiB code (*After Burner*, *Golden Axe*, *Phantasy Star*, *Sonic the Hedgehog*, *Micro Machines* and 10 more) | `$0` | 115–116 |

### 13.9 What the export check refuses

Under the default settings four of the library's images meet the check and fail it, all four refused in words ("ITS
CHECKSUM DOES NOT MATCH ITS DATA."): three Brazilian images (*Felipe em Ação*, *Teddy Boy e Geraldinho*, *Treinamento
do Mymo*) and a Korean one (*Toto World 3*, whose board is stage 12's), none of whose stated sums any range explains
(`Endou_OpenBoot.md` §1.6). They ran before this stage, with no BIOS, and run on the Mark III by the Model setting.
*Bank Panic* (UE), whose region code `$2` names no market, runs on the Mark III by default and is unchanged. Whether to
keep the documented check for these is the tester's question (§13.14).

### 13.10 Seeded faults (2026-10-19)

Each made in the source, the crate's tests run on release with the image's pin left out (it catches every change to
the program's bytes, so it would hide which test sees the behaviour), the file restored and compared byte for byte
(`seed.py` in the step's scratch):

| # | Fault | Caught by |
|---|---|---|
| F1 | Japan's region code taken as an export console's | the refusal test and the Japanese cartridge's |
| F2 | the header summed where it lies inside the range | **nothing at first**: every test cartridge had its header at `$7FF0`, the range's end, where the second part of the sum is empty. `a_header_inside_the_range_is_left_out_of_the_sum` was added (headers at `$3FF0` and `$1FF0`), and catches it |
| F3 | slot 2 not put back after the sum | the 256 KiB test |
| F4 | the RAM not cleared before the hand-over | the programs' gate |
| F5 | port `$3F` left at `$FF` | the hand-over's state test |
| F6 | the Master System II giving the cartridge's byte where both are on | the BIOS slot's test |
| F7 | the player's image run without its setting | the player's BIOS test |
| F8 | the hand-over not waiting for the last line | the hand-over's state test, the programs' gate |
| F9 | a state's boot and BIOS not checked | the foreign state's test |
| F10 | the cartridge tried before the card | the programs' gate |
| F11 | VRAM cleared only in part | the hand-over's state test, the programs' gate |
| F12 | `firmware_for` answering nothing | the descriptors' test |
| F13 | a wrong checksum let through | the refusal, Codemasters and programs' tests |

### 13.11 D-8's dispute step, and the search for a recording

**D-8, SMS_MiSTer's `vdp` entity under GHDL** (decided 2026-10-19). The step read no RTL text: GHDL 6.0.0 was run with
`-fno-caret-diagnostics`, so that its messages name a file, a line, a column and a rule and print no source. It
analysed the VDP's files (`dpram.vhd`, `vdp_cram.vhd`, `vdp_sprite_shifter.vhd`, `vdp_background.vhd`,
`vdp_sprites.vhd`, `vdp_main.vhd`, `vdp.vhd`) with `-fsynopsys -fexplicit -frelaxed-rules`, an empty stand-in
`altera_mf` package ready for the components the messages would name [M]:

- under VHDL-93 (`--std=93c`): `vdp.vhd` is refused at two case choices whose subtype is not locally static,
  `vdp_sprites.vhd` at five actuals that are not globally static and two such choices, `vdp_main.vhd` at one actual;
- under VHDL-2008: `vdp_sprites.vhd` is refused at 26 case choices that are not locally static expressions, the other
  files analysing;
- GHDL keeps one standard for a library, so no standard analyses the entity whole. Quartus accepts the files; GHDL is
  stricter. Making it analyse means editing those lines, which is reading the RTL beyond the port list the decision
  allows. The step stopped there: the `vdp` entity's port list was not read, since nothing could be run with it, and
  the `altera_mf` stand-ins were never reached. D-8 stays open, implemented as no access lost (its entry).

**A recording of a Master System's output** (decided 2026-10-19). Searched on 2026-10-19 [R]: MDFourier's page
(`junkerhq.net/MDFourier`) lists the Master System among its "unreleased platforms", "release pending", and publishes
recordings only for the Mega Drive with the Sega CD and for the PC Engine; SMS Power!'s "SN76489" page links bit
patterns of the noise, not audio, and marks its waveform figures as needing "a real output sample"; web searches for a
Master System's output measured or recorded found none, an emulator's filter "picked entirely by ear" being the
nearest. Nothing was fetched. The analogue stage stays a named loss (§12.3).

### 13.12 Open

- **The BIOS images' MD5s** are not listed: the one public description at hand (Gearsystem's `.info`) gives one MD5
  for "bios.sms" without its console or version, and the page that would say (SMS Power!'s `BIOS`) is excluded for the
  open program's writer.
- **BIOS images other than 8 KiB** (those with a built-in game) are refused with words; how their ROM is mapped is in
  no document read.
- **`firmwarecheck.py`** against the real images is the tester's to run (`EmuSen_Firmware.md` §0).
- **What a real BIOS leaves** beyond the documents (R, the hand-over's phase, its RAM and VRAM) is unknown; the open
  program's choices are `Endou_OpenBoot.md` §2's.
- **D-8**, the analogue stage, and the earlier stages' open items stand.

### 13.13 What was read for this stage

No emulator source, no firmware image or firmware listing, and no SMS_MiSTer RTL text (GHDL's location messages only,
§13.11). The excluded `smstech` §6 and SMS Power!'s `BIOS` page were not opened; the six firmware files of the library
were left out of every survey by name, and the tester's images in `~/Documents/BIOS/SMS/` were not touched. Read:
`Endou_Plan.md`, `Endou_Native.md`, `Endou_References.md`, `Endou_Disputes.md`, `Nephrite_OpenBios.md` §1–§5 and §7–§8,
`EmuSen_Firmware.md` §0–§2 and §7, `EmuSen_CoreAPI.md` §6.2, §6.3, §6.13, §6.31, §6.32 and §28.17–§28.18; Diamond's
`bios()` and Nephrite's `build.rs`; Sega's *Software Reference Manual for the SEGA Mark III Console* (its contents and
the sections `Endou_OpenBoot.md` §0 names, screened mechanically first); MacDonald's `smstech` §1–§5 and §7; Talbot-
Watkins's VDP registers 2 and 5; SMS Power!'s "Port $3E", "RAM", "Memory Map", "ROM Header", "Card", "Card Slot",
"Cartridge Slot", "Bus Signals", "Memory and IO Ports", "Reset Button", "Interrupt Vectors", "Peripheral Ports",
"Region Detection", "TV Type Detection", "VDP Registers", "Mappers", "Codemasters Header", "FM Chip Detection",
"Palette" and "Pro Action Replay" (the lines naming a BIOS), and "SN76489" (for its links, §13.11); the libretro cores'
`.info` firmware lines; MDFourier's page. The headers of the library's game images and of the fetched test programs,
read in place, for §1.6 of the design page and §13.8.

### 13.14 Questions before stage 7

1. **The four images the export check refuses** (§13.9). Keep the documented check, the four running on the Mark III
   by the Model setting? Recommended: yes; a check of the header's presence alone would accept games an export
   console refuses.
2. **The BIOS checksums** (§13.12). Recommended: at stage 7, a public list naming each image's console and version
   (No-Intro's Master System list, or SMS Power!'s `BIOS` page read by someone who writes no open boot program)
   fetched and cited, each console's entry given its images' MD5s.
3. **BIOS images with a built-in game.** Recommended: a later stage, with a document for how a BIOS larger than 8 KiB
   is mapped, or a screener's reading of the excluded pages.
4. **`firmwarecheck.py`**: recommended that the tester run it on their three images when convenient.
5. **D-8's referee.** GHDL cannot analyse SMS_MiSTer's VDP. Two ways remain: nvc, another VHDL simulator, which may
   accept the files as Quartus does (installing it is the tester's to allow); or a dispute step allowed to read and
   edit the lines GHDL names, beyond the port list. Recommended: nvc, tried as GHDL was, before any further reading.
6. **The go for stage 7** (the Master System to players), as the plan has it.

*Decided 2026-10-19 by the tester: all six as recommended.* No real hardware is used for any of them (`Endou_Plan.md`
§10 Q11).

1. **The documented export check stays.** The four library images it refuses (three Brazilian, one Korean) play on the
   Mark III by the Model setting, and the refusal's words say so.
2. **A public BIOS checksum list may be fetched**, No-Intro's DAT or SMS Power!'s `BIOS` page, to cite each model's
   BIOS MD5 and SHA-1 in Endou's firmware list, as the other cores' firmware lists do. Only the checksum table is read,
   never a page's description of a BIOS's code or behaviour, since the reader is the open boot program's writer's
   successor and the clean room still holds; the fetch's provenance is recorded.
3. **BIOS images with a built-in game** (larger than 8 KiB) stay refused for now, recorded for a later stage.
4. **`firmwarecheck.py`** was run against the tester's three images, counts and offsets alone printed; stage 7 measures it as the tool's docstring
   prescribes for compiled code, with a control and the explained runs (`Endou_OpenBoot.md` §11).
5. **D-8's referee**: nvc is being set up apart from Endou's stages; D-8 is not worked on at stage 7.
6. **The go for stage 7.**
