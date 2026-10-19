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
| State | `ENDO` version 1: magic, version, console byte, frames, the Z80's block, the PSG's, then RAM, VRAM and CRAM; 24,687 bytes for `sms`, 17,487 for `sg1000`, pinned by `the_version_1_layout_is_pinned` | §5.7 |
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
aside [A: the copier convention]. 11 of the tester's 316 `.sms` files have such lengths (33,280, 262,656 and 524,800
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

## 8. Questions before stage 2

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
