# Endou_References — the documents, test programs, references and referees, before fetching

*Written 2026-10-19, at stage 0.* The record `Endou_Plan.md` §6 points to: every source the plan relies on or names,
with its provenance and licence as far as they are known before anything is fetched. Part of it is already in the
probe's cache from Nephrite's fetch of 2026-10-04 and is listed with the hashes `DOCS.txt` recorded then; the rest is
to be fetched at stage 1 on the tester's go (`Endou_Plan.md` §10 Q4), into `~/.cache/emusen/probe/endou/`, with each
file's source, version, date, hash and licence in `PROVENANCE.txt` and `docs/DOCS.txt` there, as Nephrite's and
Diamond's were. Nothing fetched is committed, and **nothing was fetched to write this page**.

Marks as in the plan: **[M]** measured on 2026-10-19 with the tool named; **[R]** read on 2026-10-19 from the source
named; **[A]** argued or recalled, to be confirmed at the fetch, and corrected in place if it reads otherwise.

---

## 1. The documents

### 1.1 Already in the probe's cache

Under `~/.cache/emusen/probe/nephrite/docs/`, each row from `DOCS.txt` [R], fetched 2026-10-04:

| Document | Path | Version | SHA-256 | Licence, as recorded | Endou uses it for |
|---|---|---|---|---|---|
| Charles MacDonald, *Sega Master System VDP documentation* | `macdonald/msvdp.txt` | 11/12/02 | `dc16f8e6…d1eb6207c` | none stated; its header reads "Unpublished work Copyright 2000, 2001, 2002 Charles MacDonald" [R] | The VDP family: ports, status, CRAM, the modes, registers, patterns, background, sprites, timing, interrupts, the Genesis's and the Game Gear's differences (§1–§15) |
| SMS Power!, *SN76489* (Maxim et al.), and its print view | `smspower/SN76489.html`, `-print.html` | "last modified Thu Sep 04, 2025" | `8bb723b9…c67526a` (the page) | none stated | Which consoles carry which PSG variant; already the source of `beryl-sn76489` |
| Texas Instruments, *SN76489* datasheet | `ti/SN76489.pdf` | 9 pages | `ad2b99be…e78a99f91` | none stated | The discrete chip, for the SG-1000 |
| Zilog, *Z80 CPU User Manual* | `zilog/um0080.pdf` | UM008011-0816 (2016) | `e3c83da5…603447b5e` | © Zilog, public | Already `beryl-z80`'s |
| Sean Young, *The Undocumented Z80 Documented* | `zilog/z80-documented.pdf` | 0.91, 2005-09-18 | `6413048f…643d7a9d` | none stated | Already `beryl-z80`'s |
| Gambrell and Niemitalo, *OPLx decapsulated* | `oplx/OPLx_decapsulated.txt` | 2008-04-20 | `56c3d6fa…b483b7b` | none stated | D-2: the OPL2's log-sine and exponent ROMs |
| Nemesis's FM documentation archive | `nemesis/FMDocs.7z` | archive of 2011-06-03 | `74d195e5…1f5581` | none stated | Its YM2413 folder (the datasheet), **not extracted**; the archive holds executables and one `.c` file, which are not opened [R `DOCS.txt`] |

Each SHA-256 is cut to its ends; the full values are in `DOCS.txt` and `docs/SHA256SUMS`.

### 1.2 To be fetched at stage 1

| Document | Where it is expected | Provenance and licence, as known before fetching | For |
|---|---|---|---|
| MacDonald, *Sega Master System technical information* (`smstech.txt`) | the mirror `msvdp.txt` came from (`dreamjam.co.uk/emuviews/txt/`) [A] | His own write-up, like `msvdp.txt`; licence to be read | The memory map, the ports, the mappers, the region read-back, the peripherals |
| SMS Power!'s development pages (`smspower.org/Development`) | the site [R the SN76489 page's URL] | A community wiki; no licence was stated on the page already fetched [R `DOCS.txt`]; each page's to be read | The memory map; memory control (`$3E`) and I/O control (`$3F`); region detection; the VDP's registers and timing; the ROM header and its checksum; the mappers (Sega's, Codemasters', the Korean and multicart boards); the BIOSes' behaviour as described in prose; the YM2413 and the FM unit's detection; the Light Phaser, the Paddle, the Sports Pad, the 3-D glasses; the Game Gear's ports, Gear-to-Gear and its EEPROM boards |
| Sean Young, *TMS9918A/TMS9928A/TMS9929A* notes | the MSX community's archives [A] | A public write-up; licence to be read | The TMS9918's undocumented modes and flags, for stage 4 |
| Texas Instruments, *TMS9918A/TMS9928A/TMS9929A Video Display Processors Data Manual* (1982) | public scans [A] | A manufacturer's manual | The TMS9918A variant, its palette and timing |
| Yamaha, the YM2413 (OPLL) application manual and datasheet | the copy in `FMDocs.7z`, extracted for its documents alone; public scans [A] | A manufacturer's document | Stage 9: the OPLL's registers, operators, rhythm mode and instruments by name |
| Public research on the OPLL (die photographs and prose findings) | to be searched [A] | Write-ups only; findings published as an emulator's code are excluded (§5) | D-2; the instrument table's provenance |
| Notes on the VDP's timing by the homebrew scene (the line counter, the access slots, the H counter) | SMS Power's forums and development pages [A] | Forum findings: mixed reliability, as Nephrite treated SpritesMind's [R `Nephrite_Plan.md` §2.1] | `Endou_Plan.md` §6.2's thin areas |
| Sega's developer documents for the Mark III and the Game Gear (a software reference manual for the Mark III; a hardware reference manual for the Game Gear) | as recalled, hosted by SMS Power [A]; existence and contents to be confirmed | Sega's confidential documents, used as Nephrite's Q1 allows, each use cited by manual and section [R `Nephrite_Plan.md` §9 Q1] | Where the public documents are thin; any page listing a BIOS is screened first (§5) |

---

## 2. The test programs

| Program | Source | Licence | Covers | How it reports |
|---|---|---|---|---|
| **ZEXDOC/ZEXALL** (Frank Cringle) | `agn453/ZEXALL` @`8f71d41`, in the cache at `~/.cache/emusen/probe/nephrite/roms/zexall` [M: `ls`] | GPL-2.0 [R `Nephrite_Plan.md` §3.1] | The Z80's instructions and flags; CRCs taken on real Z80s | Under `beryl-z80`'s CP/M shim, 67 of 67 each [R `Beryl_Z80.md` §6.1] |
| An SMS-hosted port of ZEXALL | as recalled, SMS Power's homebrew listings [A] | to be read | The Z80 on the whole machine, with the VDP printing | Text on screen; a reader is written for it |
| VDP test programs from the homebrew scene: as recalled, a VDP timing test by FluBBa, and others testing sprites, collision and overflow, the line interrupt and the counters [A] | SMS Power's development and homebrew pages | each to be read | The VDP family's rules and timing | Some self-reporting (verdicts on screen), some by picture; a protocol per program, as Nephrite's runner holds them as data [R `Nephrite_Plan.md` §3.3] |
| The 240p Test Suite | whether a Master System build exists is to be checked; the Genesis builds are GPL-2.0 [R `Nephrite_Plan.md` §3.1] | — | Video patterns, the pads | By picture |
| TMS9918A test programs from the ColecoVision and MSX scenes | to be searched [A] | each to be read | Sprite limits and flags, the modes | Usually by picture; a program needing another console's memory map is rewritten with `beryl-z80`'s assembler from its documented intent, not ported from its source |
| Test programs written for Endou | in the crate, assembled at build time | the project's | Every rule a document states and no program tests, as Nephrite's and Diamond's were | Self-grading, by construction |
| Sega's own test cartridges, if any | only if the tester supplies them | — | — | Used locally, never fetched [R `Nephrite_Plan.md` §9 Q14] |

**The search is recorded with its negative results**, as Nephrite's and Diamond's were [R `Nephrite_Plan.md` §3.1;
`Diamond_References.md` §1]: whether anything like AccuracyCoin exists for the Master System, a self-grading VDP suite,
an OPLL test program, or a hardware test for the Light Phaser or the Paddle.

---

## 3. The black-box references

| Reference | Where | Licence | State |
|---|---|---|---|
| Genesis Plus GX | libretro buildbot; pinned v1.7.4 `58c3414` [R `libretro-cores.sha256`] | non-commercial [R `.info`] | **In the cache** [M] |
| PicoDrive | pinned 2.05 `1890c29` [R] | MAME (non-commercial) [R] | **In the cache** [M] |
| BlastEm | pinned 1.0.1-pre [R] | GPLv3 [R] | **In the cache** [M] |
| Gearsystem | libretro buildbot, `gearsystem_libretro.so` [A] | GPLv3 [A] | to fetch and pin |
| SMS Plus GX | libretro buildbot, `smsplus_libretro.so` [A] | GPL [A] | to fetch and pin |
| MAME | a release, built or packaged; its Master System, SG-1000 and Game Gear drivers run headless, memory read by a Lua script [A] | GPL-2.0 or later [A] | to fetch; its source not read |
| Mesen 2 | the probe's Mesen backend, which links a patched Mesen build [R `backends/mesen.rs`]; its Master System and Game Gear support to be confirmed from its README [A] | GPL-3.0 [A] | the backend's patch gains the console; the source tree is built, not read |
| Emulicious | its author's site [A] | freeware, closed source [A] | no headless mode: a reference only through the tester's own runs |
| Nuked-OPLL | its public release [A] | to be read | built and driven through its public header alone, outputs only, for stage 9, if the tester allows (`Endou_Plan.md` §10 Q4) |

Each core's options, spaces and cost per frame are measured at stage 1, and the probe gains whatever the Master
System needs (a model and region option pinned through `GET_VARIABLE`, a `--system` for `.sg` if a core is unsure), as
Nephrite's §1.6 listed for the Genesis [R `Nephrite_Plan.md` §1.6].

---

## 4. The referees

| Referee | Where | Licence | State |
|---|---|---|---|
| SMS_MiSTer | `github.com/MiSTer-devel/SMS_MiSTer` | to be read from its licence file | not cloned; cloned with depth 1 when a dispute first needs it, its README, licence and directory names read, the rest by `Endou_Plan.md` §1.6 and §10 Q6 |
| Nuked-MD's board | `~/.cache/emusen/probe/nukedmd-board` [R `Nephrite_Native.md` §11] | GPLv2 (`rtl/nuked-md/LICENSE`) [R `Nephrite_Plan.md` §1.5] | built; run as a black box in a dispute step, its evidence the 315-5313's |

---

## 5. Exclusions, to be written into `DOCS.txt` at the fetch

- **For replacement writers**: any page that lists or disassembles a Master System or Game Gear BIOS, wherever
  published; a page mixing hardware findings with a listing is screened by someone who does not write the open boot
  program and is cleared or excluded, as Nephrite's rule has it [R `Nephrite_Plan.md` §1.3].
- **For every writer**: any emulator's source (the list of `Endou_Plan.md` §1.4), and any finding published only as
  an emulator's code, including the OPLL's die analyses where they are code; code fragments inside documents are
  read as a document's illustration of a stated rule, never copied, as `Beryl_SN76489.md` §1 treated SMS Power's.
- The `.c` file and the executables inside `FMDocs.7z` stay unopened [R `DOCS.txt`].

## 6. What was read for this page

`DOCS.txt`'s entries for the rows of §1.1; the header and contents list of `msvdp.txt`; the libretro cores' `.info`
files and `libretro-cores.sha256`; the probe's `backends/mesen.rs` header; `Nephrite_Plan.md` §1.5, §1.6, §2 and §3;
`Diamond_References.md` §1–§2; `Beryl_Z80.md` §6.1. No network request was made.

## 7. Negative results

- No Master System, Game Gear or SG-1000 test program is in the cache [M].
- No document on the Master System beyond `msvdp.txt` and the PSG's is in the cache; the YM2413's datasheet is there
  only inside an unextracted archive [M].
- None of the libretro cores in the cache lists checksums for the Master System's or the Game Gear's firmware [R their
  `.info` files].
