# Endou_References — the documents, test programs, references and referees

*Written 2026-10-19, at stage 0, before anything was fetched; rewritten the same day at stage 1, after the fetch the
tester allowed (`Endou_Plan.md` §10 Q4, decided 2026-10-19).* The record `Endou_Plan.md` §6 points to: every source
the plan relies on or names, with its URL, version or commit, licence and SHA-256. Everything fetched is under
`~/.cache/emusen/probe/endou/`, with `PROVENANCE.txt` at its root, `docs/DOCS.txt` and `docs/SHA256SUMS` for the
documents, `roms/SHA256SUMS` for the programs, and each download's HTTP headers in `docs/.headers/`. **Nothing fetched
is committed.** No firmware and no commercial image was fetched.

Marks: **[M]** measured on 2026-10-19 with the tool named; **[R]** read on 2026-10-19 from the source named; **[A]**
argued. What the stage-0 page recalled and the fetch corrected is in §6.

---

## 1. The documents

### 1.1 Already in the probe's cache from Nephrite's fetch (2026-10-04)

Under `~/.cache/emusen/probe/nephrite/docs/`, as `DOCS.txt` there records them: MacDonald's `msvdp.txt` (11/12/02,
`dc16f8e6…1eb6207c`), SMS Power!'s *SN76489* page, TI's SN76489 datasheet, Zilog's *Z80 CPU User Manual*, Young's *The
Undocumented Z80 Documented*, Gambrell and Niemitalo's *OPLx decapsulated*, and Nemesis's `FMDocs.7z`, still not
extracted. The `msvdp-20021112.txt` fetched for Endou is byte-identical to Nephrite's `msvdp.txt` [M `cmp`].

### 1.2 Fetched at stage 1

Under `~/.cache/emusen/probe/endou/docs/`. The SMS Power! files were served from `smspower.org/uploads/Development/`
with `Last-Modified` 2009-11-09 [M the headers]; the wiki pages are their print views as served on 2026-10-19.

| Document | File | Version | Licence, as stated | SHA-256 |
|---|---|---|---|---|
| Charles MacDonald, *SMS/GG hardware notes* | `macdonald/smstech-20021112.txt` | 2002-11-12 | "Unpublished work Copyright 2000, 2001, 2002"; no licence | `59c46ea5b881a041be7edf19af3ebd23880d9fd612f8599b8520376f75851dac` |
| MacDonald, *Sega SC-3000 Hardware Notes* (the SG-1000 family) | `macdonald/sc3000h-20040729.txt` | 2004-07-29 | none stated | `f9f3b4dab18f4f625470c18f720c13dddd6b448062b86f4010cc5cb5dc849944` |
| MacDonald, *SMS VDP documentation* | `macdonald/msvdp-20021112.txt` | 2002-11-12 | as §1.1 | `dc16f8e687b8b137d9f7d457659135ebb7b21e6e7874250fac0719dd1eb6207c` |
| Richard Talbot-Watkins, *Sega Master System Technical Information* | `talbot/richard.txt` | undated | none stated | `295994d375616d009ba0ab5591657c089303ba46965916daa076417a13f8f560` |
| Sega, *Software Reference Manual for the SEGA Mark III Console*, SMS Power!'s transcription | `sega/SMSOfficialDocs-print.html` | page as served | Sega's; used as Nephrite's Q1 allows, each use cited | `bd0b7b70207aca06b1d8ddf8ef8ab4fad537c18579910c97e5b0c1c4fc6c8d48` |
| The same as a PDF (`mk3man.pdf` inside) | `sega/sms-officialdocs-pdf.zip` | 2006 archive | the same | `a47fea7e50f9850a44d2a488d968417ffe8106935411f2873a7d80a3556fc8b0` |
| Sega's Game Gear hardware notes ("Precautions for Hardware", the system ports, the PSG manual, supplements), transcribed | `sega/GGOfficialDocs-print.html` | page as served | the same | `f13232e65fcfb18aaf6ea1db608a760fbd29af588389addb5b0cdfd8eb496b2d` |
| The same, the scans: 40 pages; `vdp-05.png` is absent from the archive [M `unzip -l`] | `sega/gg-officialdocs-png.zip` | 2006 archive | the same | `e616ff32ba6a17621eeeca8ec5e1ed62fdfa0d5a06853a73a3820fc98afce1dc` |
| SMS Power!'s index of the official documents | `sega/OfficialDocumentation-print.html` | modified 2023-12-14 | none stated | `6176a49fd8bfdb863573679f1326c34fcd65ec1da5f8b96109e2fa7f1bd2220f` |
| Texas Instruments, TMS9918A data sheet (scan) | `ti/TMS9918.pdf` | as SMS Power! hosts it | a manufacturer's document | `b5b5ac07ab8a3bf8c38c66baedd9da9708d135db58ee16c55119426c1641cedb` |
| Karl Guttag, *TMS9918 Arizona Technical Symposium Draft*, transcribed | `ti/TMS9918ArizonaTechnicalSymposiumDraft-print.html` | page as served | none stated | `0d5c1b406717e01ed7ca31c51bff22017fccc358bdff9b4f08ef65d94f73f2cc` |
| Sean Young, *Texas Instruments TMS9918A VDP* | `young/tms9918a.txt` | 0.4.2, September 2002 | none stated | `1edab44d35f3c56055551b102e9a17c2eaec02929701bc8d12de80fa0134861a` |
| Yamaha, *YM2413 Application Manual*, PDF of scans | `yamaha/chipsym2413am.pdf` | as SMS Power! hosts it | a manufacturer's document | `d5e4864a76ae538b80a910c56a32218919148adde1a7101aafdfd8eb63bcc656` |
| The same converted to HTML with redrawn diagrams (Maxim) | `yamaha/YM2413ApplicationManual.html` | page as served | none stated | `fdc12e0ea458077704d350e9ac9a04db03007c16ab2179b82de41e20545f96d1` |
| SMS Power!'s development wiki: 60 pages (the memory map, `$3E`, the I/O maps of the three consoles, the controller and peripheral ports, region and TV-type detection, the ROM header, Codemasters' header, the BIOS page, the card and cartridge slots, the clock, interrupts, Pause and Reset, the pads, the Light Phaser, the Paddle, the 3-D glasses, the keyboard, the YM2413, FM detection, the audio control port, the VDP's registers, VRAM map, palette, sprites, tiles, tilemap and its mirroring, the scanline counter, the SMS1 and SMS2 VDPs, the TecToy VDP, the Game Gear's VDP, LCD, stereo, cartridge and Gear-to-Gear cable, pixel aspect ratio, video output, RAM, the paging chips, the Action Replay and the Game Genie, the cartridge's board and pinouts, the bus signals, and the indexes) | `smspower/*-print.html` | as served 2026-10-19 | none stated on the pages | `docs/SHA256SUMS` |
| ES-DE's systems file, Linux | `esde/es_systems-linux.xml` | master `a529a9d` (2026-10-10) | MIT (ES-DE's) | `275bcd127fece2e467fd2e24976e05767c5d1308a1509f84d6733eff5abb7f89` |
| ScreenScraper's system pages 2, 109, 21 and 1 | `screenscraper/sys-<id>.html` | as served | the site's | `docs/SHA256SUMS` |

Three wiki names the stage-0 page expected returned 404 and are not kept: `SportsPad`, `HandleController` and
`JapaneseToExportSMS` [M]. The Sports Pad is described in Sega's Mark III manual ("How the trackball works").

## 2. The test programs

From SMS Power!'s homebrew archive, `https://www.smspower.org/uploads/Homebrew/<file>`, each archive in
`roms/<name>/_dl/` and unpacked beside it. **Sources were not opened at stage 1**; each is read at the stage that runs
its program, as a test's source says what it expects. Licences are as the page or the archive states them.

| Program | Author | Version, date | Licence | Source | Covers | Archive SHA-256 |
|---|---|---|---|---|---|---|
| *ZEXALL* (SMS port, with ZEXDOC) | Frank D. Cringle, J. G. Harston, brettk, Maxim, Eric R. Quinn, Rene Dare, FluBBa, asynchronous | 0.20, 2025-05-16 | **GPL-2.0** (`LICENSE`) | included | the Z80 on the whole machine, printing on the VDP (G1) | `acf1b84561523e0410ae01e507c3ddf8932025d92deae34e67d93922739318e2` |
| *SMS VDP Test* | FluBBa | 1.31, 2009-11-04 | none stated | included | the VDP's rules and timing; its readme asks for ZEXALL first and "exactly 228 cpu cycles per scanline" [R its first lines] | `6b8f113e54c084d542ae2f87951451724b4dfa85eae95f3b9a5121e6b1bd1179` |
| *SMS Test Suite* | sverx | 0.36, 2025-02-11 | none stated | **no** | the 240p Test Suite's patterns for the Master System ("inspired by Artemio's 240p, but not a fork of it" [R its page]) | `7ea8ab6ee2bff3782e27dada89fc3efff15a7ed87acaf0ca47d1de25f9f48461` |
| *V Counter Test* | Charles MacDonald | 1.00, 2001-07-30 | none stated | included | the V counter | `c7015c35e3cee79f9cf718da3f0d2a9388a7761bcdd03ccdded954e48501877f` |
| *Scroll Test Program* | MacDonald | 1.00, 2001-07-30 | none stated | included | the scroll registers and locks | `3f4ceb9b178bc92745a2924380d15503f8a472b3c1eadf9afc5a1ac6bab647ea` |
| *64 Color Palette Test Program* | MacDonald | 1.00, 2001-07-30 | none stated | included | the 64 colours (D-1's picture) | `c5f8001f337439aa7503523cea4144d55387151ee18126ba24d141729b77615d` |
| *Fullscreen Palette Test Program* | MacDonald | 1.00, 2001-07-26 | none stated | no | CRAM written mid-frame | `6017b6fcf9dbd1a0aa4153c3991b4061ee88adbdee8af6e918cf9f83703a8c7e` |
| *Mega Drive 6 Button Controller Test* | MacDonald | 1.00, 2001-07-30 | none stated | included | TH on the pad ports | `9df2650b9188fc8443731fbe075c01dede87ccf44227069f3ace338338ae0f73` |
| *Pause Test* | FluBBa | 1.00, 2006-04-27 | none stated | included | the Pause NMI | `364ec7f12792cf80a971e11675c995d97031f25a8bc973a14bc2b292dada4551` |
| *SMS Cycle Counter* | FluBBa | 1.0, 2006-02-02 | none stated | included | instruction timing on the machine | `a32bdd7ae257a37ad6d87439c4defcb3aa100116f0bc491cbd47b1dbf685efac` |
| *Sega Mark III BG Test* | Enri | 0.01, 2002-03-11 | none stated | included | the background on a Mark III | `e277a4717a22fbf4ba8e18d9a7b3d738cdfbdb23cfcc1f8031567dd1bd9208ed` |
| *Sega Mark III Port Test* | Enri | 0.1, 2002-08-10 | none stated | included | the I/O ports on a Mark III | `8b3d4af2a1d7d0a330bd37be5a98d2c3706324a9a8a94749e7ab536c57e1b833` |
| *Color Check* | Dave | 1.00, 2001-07-26 | none stated | no | colours | `50741742b9d13858979ef7e40ef883fcf20b305fc591a6d0ff270481fca8f2bc` |
| *Checkstate* | Dave | 1.00, 2001-07-28 | none stated | included | the state the machine starts in | `21ecae241031305a0ad0ffdccf6df6ea48c380e76f30a112cf74ece4e237b9af` |
| *SMS Palette* | Bock | 1.00, 2001-07-27 | none stated | included (`sources.zip`) | the palette | `626934fe90b725cd7d4252418cc2111fd9beb44f776c8e3cdf193821919a4bd9` |
| *SMS Padtest* | TacTheCat | 0.25, 2026-02-25 | **2-Clause BSD NON-AI** (its page) | separate, not fetched | the pads, the Genesis's 3- and 6-button pads | `8f53f07275fcf390aa9d91efdde00e5a1804aa6519d34ac37d2bdf3bd65e57bf` |
| *SMS Sound Test* | Heliophobe, Maxim | 1.2, 2024-01-28 | none stated | not stated | the PSG | `a7cfdf04119789fd9c74310c1eb9814c2cc97f6b71a43cf73cdd6d735a3cb40d` |
| *SN76489 TestRom* (SMS, NTSC and PAL) | JoppyFurr | 1.00, 2024-03-27 | none stated in the archive | included | the PSG against an emulator, by ear and scope | `7b92df368b417b2834f5c1cff2f4eb769cdb05d6044fbdb5e052409520eced3b` |
| *SN76489 TestRom* (SG-1000, NTSC and PAL) | JoppyFurr | 1.00, 2024-03-27 | none stated in the archive | included | the discrete PSG (D-0) | `35f103ec7063657b9dd71b1339cafcabd480f8f035b65829c28a247b21898fdb` |
| *YM2413 TestRom* | JoppyFurr | 1.00, 2024-03-27 | **MIT** (`LICENSE.txt`) | included | the OPLL, stage 9 | `15b3f259cc96719219e5ebd539ce4fc757ad0979332623275cc954b085342d19` |
| *Sega Master System YM2413 FM Tool* | ccovell | 1.00, 2015-03-27 | none stated | included | the OPLL's registers by hand | `514d06d9b3d760491616fa9bd0f85ac4d107b32ca571c0252aa802ec30e7583a` |
| *Sega Master System 48KB Test* | ichigo | 1, 2021-03-14 | none stated | not stated | a 48 KiB board | `42ef6ae564f0dee2b0f217caf002c123449371f94e6d4879148f1bcc49f78bf8` |
| *PFR Detect* | Eric R. Quinn | 2.00, 2001-03-20 | none stated | included | FM detection by the audio control port | `dda8ae7e52f81484eb3950ea7c82f077454e0f16a8337fb2f575cb489a6ba351` |

**The search, with its negative results** [M the homebrew index's `Technical` and `Tool` listings and web searches]:
no self-grading suite like AccuracyCoin for the Master System; FluBBa's *SMS VDP Test* and ZEXALL report on screen,
the rest are read by picture or by ear. No Master System build of Artemio's 240p Test Suite: the suite's own tree
(Nephrite's clone) has no Master System folder, and sverx's suite takes its place. No TMS9918A test program from the
ColecoVision or MSX scenes was found in a form that runs on an SG-1000; the SG-1000's only program is JoppyFurr's PSG
test. No hardware test for the Light Phaser, the Paddle or the Sports Pad was found. Not fetched by rule: the
homebrew BIOS replacements (*Simple BIOS*, *DBIOS*), which are firmware, not tests.

## 3. The black-box references

| Reference | Where | Version | Licence [R its `.info`] | State |
|---|---|---|---|---|
| Genesis Plus GX | the buildbot, pinned | v1.7.4 `58c3414` | non-commercial | run [M] |
| PicoDrive | pinned | 2.05 `1890c29` | MAME (non-commercial) | run [M] |
| BlastEm | pinned | 1.0.1-pre | GPLv3 | run [M] |
| **Gearsystem** | `buildbot.libretro.com/nightly/linux/x86_64/latest/gearsystem_libretro.so.zip`, nightly of 2026-10-19; `.so` SHA-256 `c6cf3b7ed31bf264be362937fdbf45a0f680824474cb147509d5185ca7b7e766` | 3.9.22-20-g190873a | GPLv3 | **pinned and run** [M] |
| **SMS Plus GX** | `…/smsplus_libretro.so.zip`, nightly of 2026-10-19; `.so` SHA-256 `e2b29b3571a28fbb49275cb7c213d26bddef5a077db006d195d5210bb28c93c2` | 1.8 `3844b46` | GPLv2 | **pinned and run** [M] |
| MAME | Fedora's `mame` 0.289 is in the repositories and not installed | — | GPL-2.0 or later [A] | not fetched: installing a system package is the tester's to allow (`Endou_Native.md` §8) |
| Mesen 2 | `github.com/SourMesen/Mesen2`, head `b9fa69d` (2026-06-04) | — | GPL-3.0 [R GitHub] | not built. Its README: "This repository is an archive of Mesen as it was around version 2.1.1 in mid-2025", the latest being MesenCE (`nesdev-org/MesenCE`); it lists "SMS/Game Gear" among its systems [R] |
| Emulicious | its author's site | — | freeware, closed [A] | not fetched: no headless mode |
| Nuked-OPLL | `codeload.github.com/nukeykt/Nuked-OPLL/tar.gz/1269cf5a783b65583b50fa2464d08be75830aaa0` (2023-01-19, the repository's last commit) → `refs/nuked-opll/Nuked-OPLL-1269cf5.tar.gz` | — | GPL-2.0 [R GitHub's detection] | **fetched, not extracted**; it holds `LICENSE`, `opll.c`, `opll.h` and a funding file [M `tar tzf`]. Stage 9 opens `opll.h` only. No public release or prebuilt binary exists |

Their `.info` files, from `buildbot.libretro.com/assets/frontend/info.zip`, are in `refs/info/`. Gearsystem's lists
`bios.sms` with MD5 `840481177270d5642a14ca71ee72844c` and `bios.gg` with `672e104c3be3a238301aceffc3b23fd6`, a public
description of two firmware images for stage 6 (`EmuSen_Firmware.md` §0 item 4) [R]. BlastEm's `.info` now lists its
version as v0.6.3-pre while its library reports 1.0.1-pre [R, M]; the library's word is kept.

Each core's cost, timing and spaces are in `Endou_Native.md` §5.2.

## 4. The referees

| Referee | Where | Licence | State |
|---|---|---|---|
| SMS_MiSTer | `github.com/MiSTer-devel/SMS_MiSTer`, cloned with depth 1 to `~/.cache/emusen/probe/endou/refs/SMS_MiSTer` at `8443a0d19a88aff7a5900011bccf386daf146945` (2026-10-14) | **no licence file** at the top level, and GitHub detects none [M] | README and directory names read, nothing else, by Nephrite's rule (`Endou_Plan.md` §10 Q6). It names itself "a port of Ben's Sega Master System implementation for the Papilio" [R]. Its folders: `rtl/T80` (the Z80, confirming the plan's guess), `rtl/VM2413` (the OPLL), `rtl/jt89` (the PSG), the VDP as `vdp*.vhd`; VHDL throughout [M file names]. No simulation folder or script [M]. Not run: no VHDL simulator is installed (`Endou_Native.md` §8) |
| Nuked-MD's board | `~/.cache/emusen/probe/nukedmd-board` | GPLv2 | unchanged; run in a dispute step for the 315-5313's mode 4 |

## 5. Exclusions, as `docs/DOCS.txt` records them

- **For replacement writers**, until a screener who writes no open boot program clears them: `smstech-20021112.txt`'s
  section 6, "BIOS information"; SMS Power!'s `BIOS` page, not opened. Not fetched: any page that lists or
  disassembles a BIOS, and SMS Power!'s per-game pages.
- **For every writer**: any emulator's source, Nuked-OPLL's included beyond `opll.h` at stage 9; SMS_MiSTer's RTL
  outside a logged dispute step; any finding published only as code. The `.c` file and executables inside
  `FMDocs.7z` stay unopened.

## 6. Corrections to what stage 0 recalled

- **Sega's developer documents exist** as the page guessed: a *Software Reference Manual for the SEGA Mark III
  Console* and Sega's Game Gear hardware notes, both hosted by SMS Power! (§1.2). The Game Gear's is a set of notes
  and a PSG manual, not a "hardware reference manual" by that name.
- **`smstech.txt`** is on SMS Power!, not the `dreamjam.co.uk` mirror the page named, as `smstech-20021112.txt`.
- **The TMS9918A data manual** is SMS Power!'s scan, `TMS9918.pdf`, not a 1982 "TMS9918A/TMS9928A/TMS9929A Video
  Display Processors Data Manual" found elsewhere; whether it is that manual whole is read when stage 4 opens it.
- **Young's TMS9918A document** is version 0.4.2 of September 2002, hosted by SMS Power!.
- **The YM2413's datasheet** was not found in a form that could be fetched (bitsavers' path refused, its mirror 404);
  the application manual, which the plan needs, was fetched twice over (§1.2). `FMDocs.7z` still holds a copy.
- **The OPLL's die research**: none found; the only die-level work is the OPL2's in *OPLx decapsulated*. D-2's
  "SMS Power page the summary came from" is the wiki's `YM2413` page, fetched and not yet read for D-2.
- **An SMS-hosted ZEXALL exists**, and so do the VDP tests by FluBBa the page recalled (§2).
- **Gearsystem and SMS Plus GX** are on the buildbot under the names the page guessed, with the licences it guessed.
- **MAME** was not fetched (§3); **Mesen 2** is archived, its successor MesenCE.
- **SMS_MiSTer's Z80 is T80**, as the plan guessed, by its folder's name; it has no licence file.

## 6a. Used at stage 2 (2026-10-19)

Nothing was fetched. One document already in the probe's cache from Nephrite's fetch was used: d0nut's *The complete
documentation about Genesis ROM format*, version 1.1 (`nephrite/docs/zophar/genesis-rom-format.txt`, SHA-256
`2f7353ea8e361019afaf6da1b3fea597b18982f59e58f63cc63f4ff6876dcaf0`, none stated), for the copier's 512-byte header
(`Endou_Native.md` §9.2). ZEXALL-SMS's source was read for where it writes its report and how it detects the VDP. The
references and test programs were run as `Endou_Native.md` §9.9 and §9.11 record.

## 6b. Used at stage 3 (2026-10-19)

Nothing was fetched. Read from the stage 1 fetch: MacDonald's `msvdp-20021112.txt` whole and `smstech` §3; SMS
Power!'s VDP pages and "GG VDP", "Palette" and "I/O Port Map - SMS"; the VDP sections of Sega's Mark III manual;
Talbot-Watkins's VDP registers; FluBBa's *SMS VDP Test* readme and source, and the sources of MacDonald's *Scroll Test*
and *64 Color Palette*, as a test's source says what it expects. Run: those programs, *SMS Test Suite*, *V Counter
Test*, *Color Check*, *Checkstate*, *SMS Palette*, *Fullscreen Palette Test* and *Mark III BG Test*, on Endou and on
the five references through the probe (`Endou_Native.md` §10.7-§10.8). The SMS Power! "Palette" page's colour values
were read from its HTML's styles, as the page shows them (D-1).

## 7. What was read for this page

The pages and listings of §1–§4 as far as needed to identify, date and license each item: titles, contents lists,
headers, licence files, README files, `.info` files and run logs; GitHub's API for commits, licences and pushes; ES-DE's
systems file; ScreenScraper's page titles. No emulator source, firmware or firmware listing.

## 8. Negative results

- No self-grading suite for the Master System; no TMS9918A test for the SG-1000 beyond its PSG test; no Light Phaser,
  Paddle or Sports Pad test (§2).
- No YM2413 datasheet fetched; no die-level research on the OPLL (§6).
- No VHDL simulator; MAME and Mesen 2 not built (§3, §4).
- None of the five references exposes VRAM through the standard calls but PicoDrive, whose 64 KiB is the Genesis's
  size (`Endou_Native.md` §5.2).
