# Nephrite_Plan — a clean-room Genesis, Sega CD and 32X core in Rust

*Written 2026-10-04, at stage 0.* This page plans Nephrite, EmuSen's Sega Genesis / Mega Drive core, written in Rust
from hardware documentation and graded by test programs, with the Sega CD (Mega-CD) and the 32X as attachments of the
same machine. It has no predecessor: there is no C# Genesis core to port or to reach parity with, so the page is
shaped like `VenusRT_Plan.md` with its parity section replaced by what a frontend must be given (§4).

The page keeps three registers apart, as VenusRT's does:

- **Measured** means a number taken on 2026-10-04 on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10, Rust 1.98.1),
  with the tool named beside it. Raw results are outside the repository, in `~/.cache/emusen/probe/nephrite/` (§3.6).
- **Argued** means reasoning from the documents or from other cores' measurements, with no measurement behind it here.
- **Predicted** means a claim numbered P1–P8 (§11), kept so that a later stage can retire it.

The CPUs are not Nephrite's own. They are the Beryl crates (`beryl-m68k`, `beryl-z80`, `beryl-sh2`), shared with the
later Sega cores and with any other core that has one of these processors (§8). Each Beryl crate has its own oracle
and its own record (`Beryl - Shared CPUs/`); this page plans them because Nephrite is their first user.

---

## 1. Purpose, scope and the clean-room protocol

### 1.1 Why one core for three machines

*Decided 2026-10-04 (`EmuSen_Core_Naming_Scheme.md` §3):* the Genesis is Nephrite, and the Sega CD and the 32X are
its attachments. Neither runs without a Genesis beneath it. The Sega CD's sub-68000 and the 32X's two SH-2s share the
controllers, the cartridge slot's bus and the video output of the console they plug into, and both are reached by the
Genesis's 68000 through its own address space. A Sega CD core apart from the Genesis would contain a second Genesis.
Three system ids are reported (`md`, `mcd`, `32x`, §4.1) so that a frontend can shelve, theme and match cheats per
system, while one machine runs all three.

### 1.2 Hardware scope

Everything below is in scope for the core; §9 lists what is deferred and asks the tester where the line falls.

**The Genesis.**

- **The 68000** at the master clock over 7 (about 7.67 MHz NTSC): the full instruction set, the prefetch queue,
  exceptions with their stack frames, address errors, the group 0/1/2 priorities, interrupts at levels 2 (external),
  4 (horizontal) and 6 (vertical), and the bus cycle by cycle (Beryl, §8).
- **The Z80** at the master clock over 15 (about 3.58 MHz): instructions with the undocumented flags, WZ and Q; the
  68000's control of its bus (BUSREQ, RESET); its 8 KiB of RAM; its banked window onto the 68000's space and the wait
  states that window costs both processors.
- **The VDP (315-5313):** modes 4 (the Master System's) and 5; the H32 and H40 widths and the V28 and V30 heights;
  NTSC and PAL timing; the two planes and the window, with full, column and line scrolling; sprites with their link
  list and the per-line limits (sprite count, pixel count, masking); priority, shadow and highlight; the CRAM, VSRAM
  and VRAM ports with the FIFO and its slot timing; the three DMA kinds (68000 to VDP, fill, copy) with their bus
  arbitration, the 68000 held during transfer, and their speeds by mode and blanking; the HV counter and its latch;
  horizontal and vertical interrupts and their acknowledgement; interlace modes 1 and 2; the display-off and border
  behaviour and the CRAM write dots.
- **The YM2612 (OPN2):** six four-operator FM channels, the envelope generator, LFO, detune and multiple, SSG-EG, the
  CSM mode on channel 3, the DAC on channel 6, the two timers and the busy flag; the output's ladder distortion on the
  discrete YM2612 and its absence on the YM3438 of later models (§5.5).
- **The SN76489 PSG** inside the VDP: three tones and the noise channel with its feedback, attenuation.
- **I/O and controllers:** the three ports with their TH line; the 3-button and 6-button pads (the latter's TH counter
  and its timeout); the version register (domestic/overseas, NTSC/PAL, Sega CD presence); multitaps (Sega Team Player,
  EA 4-Way Play) and the J-Cart.
- **TMSS:** the `$A14000` register and the VDP lock on consoles that have it, and the optional boot ROM (§5.6).
- **Cartridges:** plain ROM up to 4 MiB; battery SRAM on either byte lane, mapped over ROM through `$A130F1` on large
  boards; serial EEPROM (I²C) boards, identified by serial since the header cannot say; the SSF2 bank mapper
  (`$A130F3`–`$A130FF`); Sonic & Knuckles lock-on with its patch ROM and the cartridge above it.

**The Sega CD (Mega-CD).**

- **The sub-68000** at 12.5 MHz, a second Beryl 68000 on its own bus.
- **The gate array:** the main and sub CPUs' communication registers and flags, PRG-RAM (512 KiB) with its write
  protection and the main CPU's banked window onto it, the reset and bus request of the sub CPU by the main, the
  interrupt sources (graphics, timer, CDD, CDC, subcode) and the stopwatch and timer.
- **Word RAM** (256 KiB) in its two modes: 2M, held whole by one CPU and swapped by handshake, and 1M, split into two
  banks with the dot-mapped views the sub CPU sees.
- **The graphics ASIC:** rotation and scaling of stamps (16×16 or 32×32 cells from a stamp map) into an image buffer in
  Word RAM, driven by a trace vector table, with its timing.
- **The CDC (Sanyo LC8951)** with its 16 KiB buffer, the sector decoder, the host data transfer to either CPU, PRG-RAM,
  Word RAM or PCM RAM, and its DMA; **the CDD**, the drive's microcontroller, by its command and status protocol, with
  seek and read timing; **CD-DA** playback with the fader and the de-emphasis.
- **The RF5C164 PCM:** eight channels of 8-bit sign-magnitude samples from 64 KiB, with loop addresses, pan and the
  envelope.
- **The BIOS** (128 KiB, three regional versions): its boot, its sub-CPU call interface and its backup-RAM interface,
  all of which an open replacement must provide (§5.6).
- **Backup RAM:** the 8 KiB inside the unit and the backup RAM cartridge (128 KiB in Sega's), with the BIOS's format.

**The 32X.**

- **Two SH-2s (SH7604)** at the master clock times 3/7 (about 23.0 MHz NTSC), master and slave, each with its 4 KiB
  four-way cache, its on-chip peripherals (DMAC, FRT, WDT, DIVU, INTC, SCI, BSC) and the inter-CPU interrupts (Beryl,
  §8).
- **The system registers:** the adapter's enable and the 68000's view of the cartridge through the 32X (its bank
  register and the vector override), the 68000-to-SH-2 FIFO (the DREQ path), the comm ports (eight 16-bit registers
  both sides see), the SH-2s' interrupt masks and sources (V, H, CMD, PWM, VRES).
- **The 32X VDP:** two 128 KiB frame buffers swapped at vertical blank; the packed-pixel, direct-colour and run-length
  modes; the line table; the 256-entry palette and its priority bit; the fill and the shift register; the access rules
  during display.
- **The overlay with the Genesis VDP:** the two pictures' composition through the priority bit and transparency, and
  the 32X's picture's horizontal alignment.
- **PWM audio:** two channels with their FIFOs, cycle register and timer interrupt.
- **The boot ROMs:** the 68000's (256 bytes) and the master's and slave's (2 KiB and 1 KiB), with their open
  replacements (§5.6).
- **The 32X with the Sega CD** ("CD32X"): both attachments at once, which a small number of discs require.

### 1.3 Who may read what

A **writer** is anyone who writes Nephrite's or a Beryl crate's code, tests or design pages. A **dispute step** is a
separate, supervised step whose only product is an entry in the disputes log (§1.4). It changes no code.

| Source | A writer | A dispute step | Notes |
|---|---|---|---|
| Hardware documents (§2): Motorola's, Zilog's and Hitachi's manuals, datasheets, Charles MacDonald's documents, plutiedev, the public research write-ups | yes | yes | The primary sources |
| Sega's developer manuals (§2.2: the Genesis Software Manual, the Mega-CD BIOS, hardware and software manuals, the 32X hardware manual) | yes, each use cited by manual and section (§9, Q1) | yes | Sega-confidential documents that circulated without Sega's release; the one description of the Sega CD BIOS's call interface |
| A listing or disassembly of any firmware (the Sega CD BIOS, the 32X boot ROMs, TMSS), wherever published: for example the SpritesMind topic t=2061, "Sega CD (US) BIOS Disassembly" | **no** for whoever writes or reviews an open replacement; others per the general rules | no | Marked "excluded for replacement writers" in the corpus's `docs/DOCS.txt`, with the pages still to be screened |
| Test programs, their sources and READMEs (§3) | yes | yes | A test's source says what it expects |
| Reference emulators **run** through the probe (§1.6): Genesis Plus GX, PicoDrive, BlastEm, ClownMDEmu | yes | yes | Outputs only: memory, picture, sound |
| The SingleStepTests suites' **data** (§3.1), whichever emulator generated them | yes | yes | Outputs; the generators' code is not read (the Z80 suite's `generation/` folder is a translation of an emulator) |
| The MiSTer RTL (§1.5) | **no** | yes, after documents and a test program, each file logged | The referee; never transcribed |
| Other emulators' **source**: Genesis Plus GX, BlastEm, PicoDrive, MAME, Kega Fusion, Gens, DGen, Ares/higan, Exodus, Mesen, ClownMDEmu, jgenesis, Nuked-OPN2's code and the rest | **no** | **no** | Not in a dispute step either, by the decision VenusRT's Q4 records (§9, Q2) |
| Firmware images (the BIOS, the boot ROMs, TMSS) | **no**: not read, disassembled or traced | **no** | §5.6: the replacements are written from documents |

**The replacement writers' rule.** Whoever writes an open firmware replacement (§5.6) reads no listing,
disassembly or trace of the firmware it replaces, in any form, and the same holds for whoever reviews it. The open
Sega CD BIOS is written from the call interface that the Mega-CD BIOS Manual and the public documents describe, never
from the code; the 32X's boot ROMs from the 32X hardware manual's account of the boot and the cartridge's documented
header. This is the rule the NEC DSP replacements follow (`VenusRT_DspHle.md`), which forbids program listings. A page
that mixes hardware findings with a listing is screened first by someone who does not write the replacement, and is
either cleared or added to the exclusions in `DOCS.txt`.

Three rules follow, as in VenusRT's plan:

- **The order of recourse is fixed.** A dispute is settled by the documents, then by a test program written for the
  purpose, then by the RTL. No emulator source is ever the fourth step.
- **Nothing read in a dispute step is typed into code in that step.** The step ends with a rule stated in hardware
  terms, and a later writer's step implements that sentence and cites the entry.
- **The code carries no reference to an emulator's identifiers**, which the clone check of §7's G8 tests mechanically.

### 1.4 The disputes log

`Nephrite_Disputes.md`, created with its first entry, in the shape `VenusRT_Plan.md` §1.4 gives (the rule in hardware
terms, the documents, the test program, the referee's files, the conclusion, what pins it, where it was implemented).
The Beryl crates keep theirs in their own record pages, since a CPU dispute belongs to every core that uses the crate.

### 1.5 The MiSTer referees

Cloned on 2026-10-04 (depth 1). Only the READMEs, the licence files and the directory names were read.

| Core | Checkout | Commit | Licence | What its folders show |
|---|---|---|---|---|
| MegaDrive_MiSTer | `~/Projects/megadrive-mister-reference` | `4b11b2c` (2026-10-01) | GPLv2 (`rtl/nuked-md/LICENSE`) | A port of Nuked-MD-FPGA: a board transcribed from die images, with its own 68000 (`68k.v`, with microcode and nanocode tables), Z80, VDP (`ym7101.v`), OPN2 (`ym3438*.v`), the I/O and bus ASICs (`ym6045.v`, `ym6046.v`, `fc1004.v`) and TMSS; outside it, cartridge, EEPROM, SVP, pads, multitaps, light gun |
| MegaCD_MiSTer | `~/Projects/megacd-mister-reference` | `a3a3da8` (2026-07-30) | GPLv3 | `rtl/FX68K/` (fx68k); `rtl/GEN/` with its own VDP (VHDL), `jt12` (YM2612), `jt89` (SN76489), `T80` (Z80); `rtl/MCD/` with the graphics ASIC, CDC, CD-DA, PCM and the sub CPU's glue; `docs/` holds Sega's Mega-CD manuals and Sanyo's LC8950/LC8951 datasheet |
| S32X_MiSTer | `~/Projects/s32x-mister-reference` | `b438679` (2026-06-22) | **none stated** | Its own SH-2 core and SH7604 peripherals (BSC, cache, DIVU, DMAC, FRT, INTC, SCI, UBC, WDT); the 32X glue and VDP; fx68k and a Genesis built from the MegaCD core's parts. `rtl/32X/` holds files named for boot ROMs (`mdbios.mif`, `shbios.mif`), not opened; §5.6 forbids reading them |

**What agreement is worth, argued before any finding** (to be revised subsystem by subsystem, as `Venus_Referee.md`
§0 did for the SNES): the Nuked-MD-FPGA parts are transcriptions of the chips' own logic, so their agreement is strong
evidence, the strongest available for the VDP and the OPN2. fx68k is microcode-derived (strong for the 68000). T80 is a
general Z80 core not specific to Sega (moderate). `jt12`'s and the MegaCD core's VDP are their authors' readings (weak
where they differ from Nuked). The MegaCD core's CD hardware has no transcription behind it and Sega's manuals beside
it (moderate). S32X_MiSTer's SH-2 and VDP are its author's own and the only RTL for the 32X (moderate, and the core has
no licence, which matters only if a file were ever quoted, which the protocol forbids).

### 1.6 The black-box references

The reference probe's libretro backend (`EmuSen.WiseMan/Reference/probe-rs`, `build-probe.sh libretro`) runs a
libretro core headlessly. *Measured 2026-10-04,* unchanged:

| Core | Version (buildbot, 2026-10-01 to 04) | Licence | Mega Drive | Sega CD without a BIOS | 32X | Spaces through the standard calls | 600 frames, wall |
|---|---|---|---|---|---|---|---|
| Genesis Plus GX | 1.7.4 `58c3414` | non-commercial | runs | refuses to load | loads as a Mega Drive cartridge, black | RAM 64 KiB, SRAM | 0.13–0.21 s |
| PicoDrive | 2.05 `1890c29` | MAME (non-commercial) | runs | refuses | runs, with no boot ROMs | RAM, SRAM, VRAM; memory id 4 is CRAM; maps for SDRAM | 0.08–0.13 s |
| BlastEm | 1.0.1-pre | GPLv3 | runs | refuses | refuses without boot ROMs | RAM | 0.62–0.78 s |
| ClownMDEmu | 1.6.12 `d43c270` | AGPLv3 | runs | boots its own replacement; the 240p suite stayed on its loading screen to frame 3,000 | black | RAM, SRAM, VRAM; maps for Z80 RAM, PRG-RAM, Word RAM | 0.60–0.64 s |

On the 240p suite's menu at frame 600, the 68000's RAM agreed among all four in 65,325–65,530 of 65,536 bytes; the
pictures agreed in Genesis Plus GX, PicoDrive and BlastEm up to a one-to-one map of colours (every core has its own
colour levels, so an exact comparison reports 91–96% of pixels different); two runs of 1,200 frames repeated exactly
on every core. Nobody exposes VSRAM or the VDP's registers. The cores are run only as local instruments, which is why
two non-commercial licences matter only as a record (§9, Q2).

**What the probe needs** (argued, about five steps; §6's stage 2 carries them): core options through `GET_VARIABLE`
with declared defaults applied, so region, model, FM chip and pad type can be pinned (ClownMDEmu came up Japanese with
nothing answered, against its declared default); memory maps turned into named spaces, refreshed after the first frame
because PicoDrive declares its 32X maps late, and a flag for non-standard memory ids (PicoDrive's CRAM); the loader
made robust (load libm globally, print the loader's error, a `--system` override since `.bin` is ambiguous); the cores
fetched from the buildbot where Fedora has no package, their hashes in the manifest; and a picture gate up to a colour
map (built in WiseMan at stage 0, §3.3). The Sega CD has no BIOS-free reference but ClownMDEmu's incomplete one; a
`--sysdir` lets a developer point at their own dump, which the firmware policy keeps optional. *Built 2026-10-05, at stage 2: `Nephrite_Native.md`
§8.*

### 1.7 What was read to write this page

No emulator source of any kind. The probe's own source (`backends/libretro.rs`, `dump.rs`) and libretro's API header;
the READMEs and licence files of the three MiSTer checkouts and their directory listings; the READMEs of the
SingleStepTests suites and a few of their cases; the corpus's fetch records (`PROVENANCE.txt`, `DOCS.txt`, which
describe each test program and document from its README or source); OpenVGDB's `SYSTEMS` table, the libretro cheat
database's folder list and ES-DE's `es_systems.xml`, for the system packs; `VenusRT_Plan.md`, `VenusRT_Native.md` §1,
`EmuSen_CoreAPI.md` and `EmuSen_Firmware.md` §0. No firmware, no firmware listing and no document of §2 beyond its
title and contents list was read: the documents are read stage by stage, each citation naming its section.

---

## 2. Sources

### 2.1 The public documents

Fetched where they could be into `~/.cache/emusen/probe/nephrite/docs/`, every file with its SHA-256 in `SHA256SUMS`
and its source, date and status in `DOCS.txt`.

| Source | Where | Status | What it covers | Thin on |
|---|---|---|---|---|
| **M68000 Family Programmer's Reference Manual** (M68000PM/AD) and **M68000 User's Manual** (M68000UM/AD), Motorola | NXP's site | Public, current editions | Instructions, addressing, exceptions, bus cycles, the timing tables | The order of bus cycles inside an instruction, the prefetch's refill points, undefined flags (ABCD/SBCD's V), the exact stacked frames of address errors |
| **Yacht.txt** (Nemesis), per-instruction bus timing of the 68000 | Nemesis's documentation folder, archived | Public write-up | The order of reads, writes and idle cycles per instruction | — |
| **68k-undoc** (flamewing): the exception stack frames | GitHub | Public research | Undocumented stacked words | — |
| **Z80 CPU User Manual** (UM0080), Zilog | Zilog's site | Public | Instructions, timing by machine cycle, interrupts | Undocumented flags and instructions |
| **The Undocumented Z80 Documented** (Sean Young) | Public | Public | Undocumented opcodes, flags X/Y, WZ (MEMPTR) effects, interrupt details | Q's effect on SCF/CCF (later research) |
| **SH-1/SH-2 Programming Manual** and **SH7604 Hardware Manual**, Hitachi | Renesas's site; copies in the 32XDK release on GitHub | Public | The instruction set with pseudocode, pipeline and timing, cache, DMAC, FRT, WDT, DIVU, INTC, SCI, BSC | Pipeline stalls in combination, cache timing under contention |
| **Charles MacDonald's documents** (`genvdp.txt`, `gen-hw.txt`, his EEPROM notes) | A live mirror; archived copies | Public | VDP registers and timing, the memory map, I/O, the Z80's bus, EEPROM boards | DMA slot timing, the FIFO. **His index lists no Sega CD or 32X notes** (its `scdpart.txt` is the PC Engine's) |
| **plutiedev** | plutiedev.com (99 pages saved) | Public, living | Registers, DMA, the YM2612, controllers, the SSF2 mapper, TMSS, the cartridge header, SRAM | Edge timing; the attachments |
| **md.railgun.works** | 34 articles saved as wikitext | Public, dormant since 2015–2017 | Register-level descriptions gathered from research | Uneven; its uploaded datasheets are missing |
| **Nemesis's research**: the VDP FIFO, DMA and status-register write-ups, the YM2612 documentation | SpritesMind; archived hacking-cult pages | Public write-ups | FIFO slots, DMA timing, the status register, YM2612 envelope and SSG-EG details | Written as findings, not specifications |
| **SpritesMind topics** (18 saved: VDP internals and timing, DMA and FIFO, the YM2612 information thread and envelope timing, decapping, Mega CD and 32X questions) | gendev.spritesmind.net | Public forum | Findings and measurements | Mixed reliability; the Sega CD BIOS disassembly topic is excluded (§1.3) |
| **YM2608 (OPNA) Application Manual**, Yamaha | Public scans | Public, the nearest datasheet for the OPN2 | FM register semantics | The OPN2's DAC, ladder effect and test register |
| **Die analysis of the OPN2 and the VDP** | Die images and forum records | Research results | Exact envelope and phase arithmetic | Published as emulator and RTL code, which §1.3 excludes; only the prose findings and images are sources |
| **SN76489**: SMS Power's document (Maxim) and Texas Instruments' datasheet | Public | Public | Registers, noise feedback, volume table | — |
| **Ricoh RF5C68A datasheet** | Console5 | A manufacturer's datasheet, for the RF5C164's predecessor | The PCM's registers and operation | **No RF5C164 datasheet exists publicly**; the differences come from Sega's PCM manual (§2.2) and tests |
| **Sanyo LC8951 and LC7883 datasheets** | Sega Retro; the MegaCD MiSTer checkout's `docs/` | Manufacturers' datasheets | The CDC's registers and transfer; the CD-DA DAC | The Sega CD's wiring of them |

### 2.2 Sega's confidential documents

Sega's developer documents, hosted on Sega Retro and in the 32XDK release with no licence or provenance statement:
confidential manuals that circulated without Sega's release. *Decided 2026-10-04 (§9, Q1):* a writer may use them, each
use cited by manual and section; the replacement writers' rule of §1.3 stands beside it. Each is listed with what the
plan relies on it for.

| Document | Relied on for | Would a public source do? |
|---|---|---|
| **Genesis Software Manual** (several editions, 1990–1991), Technical Bulletins, Reference Sheets, Sound Software Manual, the 6-Button Control Pad specification | the Genesis's registers and timing as Sega stated them; the 6-button pad's protocol | Mostly: MacDonald, plutiedev and the research cover the Genesis; the pad's timeout is the one item they treat thinly |
| **Mega-CD BIOS Manual** | the BIOS's call interface (`_CDBIOS`, `_BURAM`, the boot, the jump tables): the specification of the open replacement (§5.6) | **No**: no public document describes the call interface |
| **Mega-CD Hardware Manual**, PCM Sound Source manual, Software Development Manual, Disc Format Specifications, Software Standards, Technical Bulletins | the gate array, Word RAM, the graphics ASIC, the CDC's and CDD's use, the RF5C164's differences from the RF5C68A, the disc's system area and security block | In part: the research covers some registers; the ASIC's timing, the CDD protocol and the PCM's differences have no public source |
| **32X Hardware Manual** (MAR-32-R4-072294), its US supplement, Technical Information, PWM driver notes | the 32X's registers, the boot sequence and the cartridge's 32X header (§5.6), the VDP's modes, PWM | **No** for the boot and the header; in part for the rest |

### 2.3 Where the public documents are thin

The places where the core will depend on test programs and disputes more than on text, each a prediction of where the
disputes log will fill (P6):

- **The VDP's slot timing.** Which access slots of a line serve the FIFO, DMA and refresh in each mode and in blanking,
  and how a 68000 write waits on a full FIFO. Nemesis's write-ups and VDPFIFOTesting are the main evidence.
- **The 68000's cycle order and its corner cases.** The documents give totals; Yacht.txt and the single-step suites the
  order; address-error stack frames and the prefetch's refill after branches come from the suites.
- **The YM2612's arithmetic.** Envelope and phase details, the busy flag's duration and the DAC's ladder effect are
  known from die analysis published mostly as code; the prose findings and hardware recordings are what may be used.
- **The Z80's window onto the 68000's bus**: the wait states and the 68000's stall on each access.
- **The Sega CD's CDD protocol and seek timing**, which Sega's manuals treat as the BIOS's business and describe from
  the BIOS's side. No public notes on the Sega CD or the 32X by MacDonald exist.
- **The RF5C164**, for which no datasheet is public; the RF5C68A's is the nearest.
- **The graphics ASIC's timing**, described in Sega's manual only by its effect.
- **The SH-2's pipeline and the 32X's bus contention** between the two SH-2s, the 68000 and the frame buffer.
- **Region and model differences**: TMSS, the YM2612 against the YM3438, model 1's audio filter, the PAL 32X.

---

## 3. The oracles, by component

### 3.1 What exists, and how each reports

Fetched on 2026-10-04 into `~/.cache/emusen/probe/nephrite/`, never committed; `PROVENANCE.txt` lists every file with
its source, version and MD5, and `unique-roms.txt` 105 ROM-like files, 91 distinct.

| Oracle | Source, version | Licence | Covers | How it reports |
|---|---|---|---|---|
| **SingleStepTests 68000** (`singlestep/m68000/v1`) | `SingleStepTests/m68000` @`64b2531` (2024-07-31); 127 files × 2,500 | MIT | Every instruction form; registers, memory, every bus transaction with function code, lanes and address-error cycles | JSON, decoded from the suite's binary by its own `decode.py`. **Generated from MAME's microcoded 68000** (§9, Q3); its README names TAS's timing and TRAPV as unverified |
| **TomHarte's 680x0 tests** (`singlestep/680x0/68000/v1`) | `SingleStepTests/680x0` @`e0d5ece` (2024-05-13); 124 files × 8,065 | none stated | The same, without lanes or address-error cycles | JSON (gunzipped from the pinned `.json.gz`); generated by its author's emulator, checked against the other published sets |
| **SingleStepTests Z80** (`singlestep/z80/v1`) | `SingleStepTests/z80` @`ebe1875` (2026-04-11); 1,604 files × 1,000 | MIT | Every opcode, prefixes included; registers with WZ, Q and P; memory; I/O; the bus T-state by T-state | JSON; generated from a translation of an emulator's Z80, its `generation/` folder not read |
| **SH-2 single-step tests** | none exist (searched 2026-10-04) | — | — | A negative result. SingleStepTests has an SH-4 suite, a partial oracle for the shared integer instructions' results (§3.2) |
| **VDPFIFOTesting** (Nemesis) | Exodus technical documents, the 2013-09-12 build, with its source (61 test files) | none stated | VDP ports, FIFO, DMA, the HV latch, the code and address registers: about 122 tests | **Self-grading**: records of name, expected and actual in RAM at `$FF0000`, and a pass/fail/total counter on screen |
| **68k BCD verifier** (flamewing) | `68k-bcd-verifier` @`39a01be` | GPL-3.0 | ABCD, SBCD, NBCD exhaustively, flags included | Self-grading: failure counts on screen |
| **68000 opcode sizes** (realmonster) | `smd_emu_tests` @`d8af819` | MIT | Instruction lengths | Two green boxes for a pass |
| **Illegal-instruction test** (Charles MacDonald) | Exodus documents | none stated | Illegal, line A and line F exceptions | Green or red screen |
| **Sprite masking, window, shadow/highlight, CRAM dots, V counter, direct-colour DMA and other VDP tests** | Exodus documents; Genesis Plus GX's published `md_test` collection (2015), including krikzz's VDPTEST | mostly none stated | The named effects | Visual; the sprite-masking tests come with photographs of a console; VDPTEST prints HV values to compare with hardware |
| **Status-register and HV "logic analyser" ROMs** (Nemesis) | archived hacking-cult pages, with sources | none stated | The status register and HV counter sampled at known points | **Machine-readable**: samples sent out of pad port 2 by a nibble handshake, which a harness can receive |
| **YM2612 tests** (Nemesis): attack/release, CSM, detune, write order, SSG-EG, DAC multiplexing | archived, with sources and one hardware recording (CSM) | none stated | The FM unit's edges | Sound, against recordings of a console |
| **Overdrive and Overdrive 2** (Titan) | Titan's releases | freeware | Stress tests of undocumented behaviour | A demo: by picture against references and recordings |
| **240p Test Suite** (Artemio Urbina): Mega Drive 1.32, Sega CD 1.32, 32X 1.0 | itch.io, GitHub sources @`89adfdd`, Dasutin's 32X port @`91f7d78` | GPL-2.0 | Video patterns, overscan, interlace, MDFourier audio, controllers and the multitap, Mega CD memory and comm tests | By picture and audio; **the Sega CD build needs a BIOS to boot** |
| **mcd-verificator** (krikzz) | `MEGA-PRO` @`8b72838` (2025-01-06), V1.02 | GPL-3.0 at that commit | Sega CD RAM cartridge, gate-array registers, interrupts, CDC registers, PRG-RAM, Word RAM in both modes | **Self-grading** text, "OK" or "ERROR"; it **runs in place of the BIOS** or as a mode-1 cartridge, so it needs no firmware |
| **ZEXDOC/ZEXALL** (Frank Cringle) | `agn453/ZEXALL` @`8f71d41` | GPL-2.0 | The Z80's instructions with their flags, documented and all | CP/M text; the CRCs were taken on real Z80s, so this is the one hardware-grounded Z80 oracle; run in the crate under a small CP/M shim |
| **The probe** | §1.6 | — | Everything, frame by frame | RAM, SRAM, VRAM (two cores), CRAM (one), picture, sound |
| **Commercial games** | the tester's Genesis library, `AppSettings.RomDirectory`'s `genesis/` (948 files, 946 of them `.bin`, added 2026-10-04), read only and copied to scratch, with a copy at `~/.cache/emusen/probe/nephrite/games/genesis/`; no Sega CD or 32X games yet (§9, Q13) | not redistributable | Integration; §7's goldens | References at anchors |

**Nothing like AccuracyCoin exists for the Genesis** (searched 2026-10-04, a negative result). The self-grading programs
are VDPFIFOTesting, the BCD verifier, the opcode sizes, the illegal-instruction test and, for the Sega CD,
mcd-verificator. No homebrew 32X hardware test was found; Sega's own self-test programs (the 32X "Mars Check Program",
MD Soft Checker) are hosted only on ROM sites and were not fetched (§9, Q14). No Genesis-hosted ZEXALL port, no 68000
cycle-timing test ROM and no PSG test ROM were found.

### 3.2 By component

| Component | Primary oracle | Secondary | Thin (§2.3) |
|---|---|---|---|
| 68000 (Beryl) | SingleStepTests 68000, with every transaction; TomHarte's suite | BCD verifier, opcode sizes, illegal test on the console; Yacht.txt | address-error frames |
| Z80 (Beryl) | SingleStepTests Z80, T-state by T-state | ZEXDOC/ZEXALL (hardware CRCs) | — |
| SH-2 (Beryl) | test programs written from the manual's pseudocode with an in-crate assembler | the SH-4 suite's results for the shared integer instructions; PicoDrive on 32X homebrew | pipeline, cache |
| Bus, map, Z80 window | the CPU suites' cycles; the memory test ROM | references' RAM at anchors | Z80 waits |
| VDP ports, FIFO, DMA | VDPFIFOTesting (self-grading) | the logic-analyser ROMs through a pad-port receiver | slot timing |
| VDP picture | references up to a colour map on steady pictures: the 240p suite, the VDP tests | sprite-masking photographs; Overdrive | — |
| YM2612, PSG | Nemesis's tests against recordings; MDFourier from the 240p suite | references' audio envelopes | arithmetic |
| I/O, pads | the 240p suite's controller tests; `gfx_joy_sampler`; `joy_speed` | references | 6-button timeout |
| Sega CD hardware | mcd-verificator (self-grading, BIOS-free) | ClownMDEmu (its own BIOS replacement); the tester's BIOS run through Genesis Plus GX for comparison | CDD timing |
| Sega CD BIOS replacement | the 240p Sega CD suite, mcd-verificator's mode-1 path, then games | the same games on the tester's BIOS in Nephrite itself | the call interface's corners |
| 32X | 240p 32X; homebrew (roq32X and the like) against PicoDrive | S32X_MiSTer in disputes | contention |

### 3.3 The runners

Built at stage 0 (`Nephrite_Native.md` §3):

- **The generic test-ROM runner** in `EmuSen.WiseMan/Fixtures/RomRunner/`: an engine over any library on the core ABI
  v1, reading spaces by machine info's names and pressing buttons by the controller's canonical controls; the libretro
  probe as an engine; a corpus read from a variable; protocols as data; and a differential that compares pictures both
  exactly and up to a one-to-one colour map. It is not Nephrite's: VenusRT or any later v1 core can run through it.
- **The crate tests over the single-step suites**, in each Beryl crate, comparing every transaction (the 68000's) or
  every T-state (the Z80's), with positive and negative controls.

To come with the stages that need them: the VDPFIFOTesting reader (its RAM records), the pad-port receiver for the
logic-analyser ROMs, the CP/M shim for ZEXALL, and protocols for the green-box and text-on-screen programs.

### 3.4 Where the results are

`~/.cache/emusen/probe/nephrite/`: `PROVENANCE.txt`, `unique-roms.txt`, `make_provenance.py`, `singlestep/`, `roms/`
(with `_src/` and `_dl/`), `docs/`; the probe's runs under `runs/` once the runner has made them. The libretro cores are
in `~/.cache/emusen/probe/libretro/cores/` with their `.info` files.

---

## 4. What a frontend gets, with no predecessor

There is no C# core whose public surface to match, so this section states the console's facts as the descriptors
carry them. Nothing here needs per-core code in a frontend (`EmuSen_CoreAPI.md` §2.4).

### 4.1 Systems

| id | Name | Extensions (stage 0) | ES-DE | libretro cheats | OpenVGDB |
|---|---|---|---|---|---|
| `md` | Sega Genesis / Mega Drive | `.md`, `.gen`, `.bin` | `genesis`, "Sega Genesis" | `Sega - Mega Drive - Genesis` | `MD` |
| `mcd` | Sega CD / Mega-CD | `.iso` | `segacd`, "Sega CD" | `Sega - Mega-CD - Sega CD` | `SCD` (hashless: matched by name) |
| `32x` | Sega 32X | `.32x` | `sega32x`, "Sega Mega Drive 32X" | `Sega - 32X` | `32X` |

The system packs in `DianaOS/Sys/Systems/Genesis/` carry these for any engine. The system is decided by the image's
contents, not its extension, because the ABI passes only bytes: a disc by `SEGADISCSYSTEM` at the start of its first
sector's data, a 32X cartridge by its header's system name, and anything else as a Genesis cartridge. `.smd`
(interleaved, with a 512-byte header) joined `md` at stage 3 (`Nephrite_Native.md` §11.2). **Later:** `.cue` with its
track files and `.chd` (§5.8, §9 Q5), `.68k`, `.sgd`.

### 4.2 Controllers

`md.pad3` (eight bits) and `md.pad6` (twelve), on ports 0 and 1. The bits follow the hardware's two reads (Up, Down,
Left, Right, B, C with TH high; A, Start with TH low), then the six-button pad's Z, Y, X, Mode. Canonical controls:
A→Y, B→B, C→A, X→L, Y→X, Z→R, Start→Start, Mode→Select, so the three face buttons fall on the RetroPad's bottom row as a
player holding a Genesis pad expects (§9, Q10). Multitaps and the light guns are later device kinds (`EmuSen_CoreAPI.md`
§6.8).

### 4.3 Spaces

Named so that cheats and the debugger key on stable words: `WRAM` (64 KiB, the cheats' space), `Z80RAM`, `VRAM`,
`CRAM`, `VSRAM`, `SRAM` (the cartridge's battery RAM when it has one), `ROM` (read-only); with the Sega CD `PRGRAM`,
`WORDRAM`, `PCMRAM`, `BRAM`; with the 32X `SDRAM`, `FRAMEBUFFER`, `PALETTE`. The buses (`M68KBUS`, `Z80BUS`, `S68KBUS`,
`MSH2BUS`, `SSH2BUS`) arrive with the bus stages, numbered 0, 1, 16, 32 and 33 so that ids never move; the
processors will be named `M68K`, `Z80`, `S68K`, `MSH2` and `SSH2`, so the host's `<name>BUS` rule finds each one's code.

### 4.4 Battery files

File 0: the cartridge's SRAM or EEPROM as `.srm`; on the Sega CD the internal backup RAM as `.brm`, and the backup
RAM cartridge as file 1 (`.crm`) when the stage that builds it decides its suffix. Raw bytes, in the order the
hardware's bus presents them (one lane for an odd- or even-lane SRAM), so that saves move between Nephrite and other
emulators that also store raw lanes (to be checked against the references at stage 3).

### 4.5 Cheats

Both formats are console knowledge and go in the system pack (`EmuSen_CoreAPI.md` §15 Q14), scheduled with stage 6:
the **Game Genie** (`ABCD-EFGH`, a 24-bit ROM address and a 16-bit value; a ROM patch, sent through `ROM_PATCHES`) and
the **Pro Action Replay** (`AAAAAA:VVVV`, a RAM poke at `$FFxxxx`, or a ROM patch elsewhere). The Sega CD and the 32X use
the same two; the 32X's SH-2 side has no common format. Not written at stage 0.

### 4.6 States, settings, the debugger

The state format is Nephrite's own (§5.7). Settings arrive with the stages that make them meaningful (§5.9). The
debugger is the generic one over `DEBUG`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`, with the Beryl crates' observers
and disassemblers (§8); console views (planes, sprites, the CD's stamps) are extensions outside v1.

---

## 5. Design decisions

Each states the trade-off, the recommendation and what it does not cover.

### 5.1 The master clock and the scheduler

- **One master clock**, the Genesis's (53.693175 MHz NTSC as the documents quote it; 15 times the colour subcarrier,
  4,725,000,000/88 Hz, as the exact rational the stub reports, a difference of 0.1 ppm, argued at stage 0 and settled at
  stage 3), from which the 68000 (÷7), the Z80 (÷15), the VDP's pixel clock (÷8 or ÷10), the FM unit (÷7) and the PSG
  (÷15) are derived. The Sega CD runs on its own 50 MHz crystal (the sub-68000 at 12.5 MHz) and the 32X's SH-2s at ×3/7
  of the master clock; each domain's ratio to the master clock is an exact rational, so nothing drifts.
- **Recommended: every bus access advances the clock** (as `VenusRT_Plan.md` §5.2). Each Beryl CPU reports its accesses
  and idle cycles to a bus that owns the time; a scheduler holds the line's fixed events (the VDP's slots and
  interrupts, refresh, the YM2612's sample points) as master-clock times.
- **Catch-up between devices.** The VDP is brought up to the present when a port is touched and at the end of each
  line; the Z80 runs in slices bounded by its next access to the 68000's bus or to a shared device; the sound chips
  catch up at their register writes and at the frame's end. Exact where the access that triggers the catch-up is the
  only coupling (argued); the test programs check it.

### 5.2 The VDP's granularity

- **Recommended: a slot-stamped scanline renderer**, the VenusRT span design adapted: register and memory writes are
  stamped with the slot at which they take effect, the line is drawn up to a stamped write before it is applied, and a
  line with no mid-line write costs a scanline renderer's work. The FIFO and DMA run on the slot schedule regardless of
  drawing, so skipping the picture leaves the state as a drawn frame would (`skip_rendering_state_neutral`).
- **What spans do not cover**, known losses at first: changes below the slot (the CRAM dots are modelled, since they
  are the CRAM write's own visible effect; others are listed as they are found), and the analogue output (no composite
  artefacts; the screen filters are the frontend's).

### 5.3 The 68000 and the Z80 sharing a bus

The Z80's accesses to the 68000's space take the 68000's bus for a cycle and stall it; the 68000's access to the Z80's
RAM requires BUSREQ. **Recommended:** the Z80 runs in slices interleaved with the 68000 on the master clock, each slice
ending at a cross-bus access, which is performed at its exact time with both processors' waits applied. The slice bound
is a measured parameter (P5).

### 5.4 The attachments' clocking

- **Sega CD:** the sub-68000 runs in slices interleaved with the main, bounded by accesses to the gate array, Word RAM
  or the comm registers and by a maximum slice measured as a parameter; the CDC and CDD are event-driven on the
  sub-CPU's clock with the drive's documented sector period (75 sectors a second at single speed).
- **32X:** the two SH-2s and the 68000 interleave in slices bounded by accesses to the comm ports, the system registers
  and the frame buffer; within a slice an SH-2 runs from its cache without the bus. The slice bound trades contention
  accuracy against speed and is measured; if §5.5's budget is missed, it becomes a setting with `effect: accuracy`
  before anything else is loosened (§9, Q11).

### 5.5 The speed budget

**What the references cost** (measured 2026-10-04 by the probe, desktop, the 240p suite's menu; light content, not a
game): Genesis Plus GX 0.22–0.35 ms a frame, PicoDrive 0.13–0.22, BlastEm 1.03–1.30, ClownMDEmu 1.00–1.07; PicoDrive on
a 32X video player 0.52. BlastEm and ClownMDEmu, the two that schedule finely, cost about four times the others.

**The targets.** A frame is 16.68 ms NTSC (59.92 Hz) or 20.12 ms PAL. The binding machines are the weak laptop, by
the proxy VenusRT measured (a 33% CPU quota on the desktop, a factor of 3.3–3.5), and the Legion Go S, whose single
thread is argued at 1.3–1.4 times the desktop's from MarsRT's figures. *Proposed, for the tester's decision (§9, Q11):*
every bench game at full speed at 33% quota with a fifth of the frame left for the frontend, a mean of at most 13.3 ms
there, which is a desktop mean of about 3.8 ms, for each of the three systems.

**What the design adds, priced (argued):**

| System | Desktop budget, mean ms/frame | What drives it |
|---|---|---|
| Genesis | ≤ 1.5 | the 68000 per access (a bus cycle is four of its clocks, so at most about 1.9 million a second, 32,000 a frame), the Z80 slices, the VDP's slots and spans, the FM unit at its native 53 kHz |
| Sega CD | ≤ 2.5 | a second 68000 at 12.5 MHz, the graphics ASIC, PCM and CD-DA |
| 32X | ≤ 3.8 | two SH-2s at 23 MHz: about 770,000 cycles a frame between them, which an interpreter at 2–4 ns an instruction makes 1.5–3 ms by itself, beside the Genesis |

**The 32X is the hardest load** and the one where the budget is most likely missed (P2). The exact levers, in order:
a decoded-instruction cache per SH-2 keyed by cache line (exact: invalidated by the writes that change code); skipping
a processor's idle polling loop to its next possible wake-up (exact where the loop's exit depends only on registers the
skip watches, as MarsRT's `RunIdle` showed, `Mars_Native.md`); then a recompiler on MarsRT's Cranelift path (exact by
construction and checked bit for bit against the interpreter). An accuracy lever (§5.4's slice bound) comes only after
those, as a setting. No tuning is done before the stage that owns the code ends.

### 5.6 Firmware: the open replacements

`EmuSen_Firmware.md` §0 (decided 2026-10-03): every game runs with no firmware folder; the player's own image is the
exact path when present, never prompted for, never shipped or fetched. Nephrite declares every image
`required: false` with its `replacement` words (`EmuSen_CoreAPI.md` §6.2). The replacements are written from public
documents (and Sega's manuals if §9's Q1 allows them); **no firmware image, listing or disassembly is read by a writer
(§1.3's replacement writers' rule), and no firmware is used as an expected value**. A tester's own image may be run in Nephrite as a black box beside the
replacement, to compare the games' behaviour, never the firmware's bytes.

**TMSS boot ROM (Genesis, 2 KiB, `bios_MD.bin`).** Optional on hardware: the first models had none. *Replacement:*
none is needed; without the image Nephrite is a console without TMSS (`effect: none`, "licensed games start the same,
without the licence screen"). With a Model setting that asks for a TMSS console, the VDP lock and the `$A14000`
register are modelled and the image, if supplied, is run; an open boot program that performs the unlock and shows its
own screen is possible (a step) and needed only if the tester wants the TMSS model without an image. Risk: none for
games; the licence screen differs.

**The 32X's boot ROMs** (the 68000's, 256 bytes; the master SH-2's, 2 KiB; the slave's, 1 KiB). What a replacement
must provide, from the 32X hardware manual and the cartridge's own documented start-up code: the 68000's vectors while
the adapter overrides the cartridge's first 256 bytes, and the adapter's enabling sequence; on each SH-2, the reset
vectors, the initialisation of the on-chip modules and the cache to the documented post-boot state, the check of the
cartridge's 32X header ("MARS" security header and the user header giving the SH-2 program's source, destination,
size and entry points), the copy of the SH-2 program into SDRAM, the comm-port handshake by which each SH-2 reports
itself ready (the "M_OK"/"S_OK" values games wait for), and the jump to the entry point with the stack and VBR the
header names. *Accuracy cost (argued):* a program reading the boot ROMs' address range sees other bytes; the boot takes
a different number of cycles, which no game is known to measure; register values not in the documented post-boot state
may differ. *Risk:* low to medium; the ROMs are small and their contract narrow, but undocumented post-boot state that a
game relies on would show only as that game failing. About 2 steps, written in SH-2 and 68000 assembly with the Beryl
assemblers (§8), the source committed and the binary built at compile time.

**The Sega CD BIOS (128 KiB, `bios_CD_U.bin`, `_E`, `_J`), the hardest.** Every disc needs it: the BIOS boots the
disc, and games call its routines throughout. *What the replacement must provide:*

1. **The boot.** On the main CPU: reset, the region and security check of the disc's system area, the loading of the
   initial program from the disc's first sectors into work RAM, the hand-over. On the sub CPU: loading the system
   program into PRG-RAM and running its initialisation, the sub CPU's interrupt dispatch and its user-call vectors.
2. **The sub CPU's call interface** (`_CDBIOS` and its function codes): drive control (initialise, open, stop, play,
   seek, pause, scan), data reads (`ROMREAD` and its variants with the CDC's buffering and transfer), table-of-contents
   and status queries, subcode, the fader, the LEDs, the CDC's start, stop, read, transfer and acknowledge. The calls'
   timing matters: games stream video and audio through them.
3. **The backup-RAM interface** (`_BURAM`): initialise, status, search, read, write, delete, format, directory and
   verify, on the internal RAM and the cartridge, **in the real BIOS's on-RAM format**, so that a save made under either
   is readable under the other.
4. **The main CPU's routines and tables** that games use by address: the vertical and horizontal interrupt hooks in
   work RAM, the communication helpers, and the utility routines (VDP set-up, decompression, font loading) the
   Software Development Manual documents. Routines games call that no document names are the long tail.
5. **A screen with no disc**: a simple open screen, not the BIOS's CD player or backup-RAM manager.

*Approach (recommended, §9 Q4):* an open 68000 program occupying the BIOS's address range and run by the emulated
CPUs, written in assembly with the Beryl assembler, its source committed and its binary built at compile time. Running
real code keeps the CPUs' timing, the interrupts and the memory traffic as a game sees them, and a game that hooks the
BIOS's RAM vectors or calls through its jump table finds code there. The alternative, trapping calls in the core and
answering in Rust, is simpler for the documented calls and breaks every game that looks inside.
*Accuracy cost:* `accuracy`, in words like "the BIOS's documented calls answer as the real BIOS does; a game that
calls an undocumented routine, reads the BIOS's code or depends on its loops' exact timing may run differently, and the
screen without a disc is Nephrite's own". *Risk: high.* The documented interface is large; the CDC/CDD driver's timing
affects every streaming game; the backup-RAM format must match bit for bit; the undocumented routines are found game by
game. The oracles are the 240p Sega CD suite and mcd-verificator's mode-1 path first, then games compared with the same
games on the tester's own BIOS in Nephrite itself. About 6 steps for the documented interface (P7), with a per-game tail
after. Until it exists, the Sega CD runs only with the player's image, and machine info says so (`firmware` `absent`).

**Not firmware by the policy's definition, recorded for later:** the SVP chip in Virtua Racing has an internal program
ROM, which would need its own open replacement (§9, Q12).

### 5.7 The state format

Nephrite's own, `NPHR`, written with `emusen-native`'s `StateWriter`: the magic, a version, the system, the frame count,
then each block in a fixed order, a block present only for the system that has it. The layout listing is pinned by a
test, there being no other record of what a version means. Sizes (argued from the memories): about 147 KiB for a
Genesis, 1 MiB with the Sega CD, 660 KiB with the 32X; rewind's snapshots are those sizes.

### 5.8 Disc images and the ABI

Two facts of the v1 ABI meet the Sega CD (argued; to be decided, §9 Q5):

- **One image, not a set.** `create` receives one image's bytes; a `.cue` names separate track files. Recommended: the
  system pack parses the cue sheet (console knowledge, in DianaOS, as `EmuSen_CoreAPI.md` §8.2 places it) and the host
  passes the tracks as numbered files after the firmware's, which v1 allows today; Nephrite then claims `.cue`.
- **Size.** A disc is up to about 700 MB, which `create` copies (the core copies what it keeps, §6.2). Workable on the
  target machines (8 GB and more), at a load-time cost; a streaming read would need a minor's new export, since the
  core never calls the host. CHD needs zlib, LZMA, FLAC and zstd decoders written without dependencies; later.

### 5.9 Settings, when their stages arrive

Region (auto, Japan, Americas, Europe; `effect: none`, since the cartridge's choice is the console's), the console
model (model 1 with the YM2612 and its filter, model 2 with the YM3438 and TMSS: `effect: none`, each a real console),
the pad type per port (`effect: none`), the output sample rate (`effect: none`), the 32X slice bound if it is ever
needed (`accuracy`, its cost stated). Defaults are a real console's, the accurate configuration
(`EmuSen_CoreAPI.md` §2.1).

---

## 6. Stages

Each stage is a run of supervised steps of about three hours with a check-in after each, as VenusRT's and MoonRT's
ran. **The effort is predicted** (P3). A stage's oracle must be met, or its misses recorded, before the next begins.

| Stage | What it covers | Its oracle | Steps |
|---|---|---|---|
| 0 | This plan; the crates' skeletons on the v1 ABI (`nephrite`, three Beryl crates); the system packs; the generic test-ROM runner; the single-step harnesses with controls; the corpus, the referees and the references pinned | The kit on the stub; the replay controls pass every case and the spoiled replays none | 1 (done 2026-10-04) |
| 1 | Beryl 68000: instructions and addressing; exceptions, address errors, interrupts, STOP and the prefetch; every transaction; the disassembler; the 68000 assembler (dev tool) | Both 68000 suites, state and transactions | 4 |
| 2 | Beryl Z80: instructions with WZ, Q and P; interrupts in all modes; the disassembler; the CP/M shim; the probe's work of §1.6 | The Z80 suite T-state by T-state; ZEXDOC and ZEXALL | 3 + 2 |
| 3 | The Genesis's bus: the memory map, cartridges (ROM, SRAM, EEPROM, SSF2, lock-on), the Z80's bus and window with its waits, I/O and the pads, the version register, TMSS, the scheduler and interrupts | BCD verifier, opcode sizes, illegal test, memory test on the console; RAM against the references at anchors | 3 |
| 4 | The VDP: ports, FIFO and DMA on the slot schedule; HV counter; interrupts; planes, window, scrolling; sprites and limits; shadow/highlight; H32/H40, V28/V30, interlace, PAL; mode 4; CRAM dots; the slot-stamped renderer | VDPFIFOTesting (self-grading); the logic-analyser ROMs through the pad-port receiver; pictures against references up to a colour map; the sprite-masking photographs; skip-versus-draw state test | 8 |
| 5 | Sound: the YM2612 (envelope, phase, LFO, SSG-EG, CSM, DAC, timers, busy, ladder effect), the PSG, the mix, the model 1 filter, the output resampler | Nemesis's FM tests against recordings; MDFourier; references' audio envelopes | 5 |
| 6 | The Genesis to players, ending with the `InDevelopment` mark removed (`EmuSen_CoreAPI.md` §27): the system packs' codecs (Game Genie, Pro Action Replay); settings (region, model, pads); battery files and EEPROM by serial; a shelf for consoles with only a discovered engine (a frontend change through the system packs); the kit on games | The kit's C1–C15 on games; `.srm` round trips; the fit audit | 3 |
| 7 | The debugger: processors `M68K` and `Z80` through the Beryl observers; registers; disassembly; breakpoints, stepping, coverage, the call stack | Each claim a test; every table armed gives the plain run's digests (C15) | 3 |
| 8 | Sega CD hardware: the sub CPU, gate array, PRG-RAM, Word RAM modes, comm registers, interrupts and timer, backup RAM and cartridge, the CDC and CDD, CD-DA, disc images (§5.8) | mcd-verificator (self-grading, no BIOS) | 6 |
| 9 | Sega CD, the rest: the graphics ASIC, the RF5C164, the fader and mix | Sega CD tests with the player's BIOS against Genesis Plus GX; mcd-verificator | 4 |
| 10 | The Sega CD BIOS replacement (§5.6) | The 240p Sega CD suite; mcd-verificator in mode 1; games against the same games on the player's BIOS | 6 |
| 11 | Beryl SH-2: the interpreter, delay slots, the cache, the SH7604's modules; the disassembler and assembler; its test programs | The manual's pseudocode as test programs; the SH-4 suite's shared instructions | 6 |
| 12 | The 32X: system registers, comm ports, the FIFO, SDRAM, the frame buffers and VDP modes, the palette and the overlay, PWM, interrupts, the boot ROM replacements; CD32X | 240p 32X and homebrew against PicoDrive; S32X_MiSTer in disputes | 6 |
| 13 | Speed (§5.5): the exact levers for the 32X, measured on the bench and the handheld | §5.5's budget | 3 |
| 14 | The gates of §7, the goldens, the clone check | §7 | 2 |

*Stage 0 was run on 2026-10-04 (`Nephrite_Native.md` §1). Stage 1's first step, the same day, passed every case of
SingleStepTests' 68000 suite with every transaction (`Beryl_M68k.md` §4); its second, on 2026-10-05, built interrupts,
reset, trace, the disassembler and the state, checked the decoder against TomHarte's instruction map, and measured the
processor at 0.147 ms a frame of 68000 work (§5 there). Its dispute step, the same day, refereed the two
suites' disagreements with fx68k and Nuked-MD (§6 there). Stage 2's first step, on 2026-10-05, built the Z80's
instruction set, its interrupts, the disassembler, the state and the CP/M shim: every case of SingleStepTests' Z80
suite, ZEXDOC and ZEXALL pass (`Beryl_Z80.md` §4-§6). Its second step, the same day, did the probe's work of §1.6
(`Nephrite_Native.md` §8), which ends stage 2. Stage 3's first step, the same day, built the buses, the cartridge, I/O,
the scheduler and the interrupts; the BCD verifier, the opcode sizes and the illegal-instruction test pass
(`Nephrite_Native.md` §9). Its second built the EEPROM boards the documents wired, Sonic & Knuckles' lock-on, the pads
chosen by setting, the external interrupt and the references' anchors over the game corpus (§10 there); its third
measured the Z80's window on Nuked-MD's board, stripped copier headers, traced Sonic 2's lead to the 68000's timing
over its boot checksum, opening the main RAM's refresh (D-2), and wired the EEPROM boards of Eke's document (§11
there). Stage 3 was closed the same day (§12 there): its oracle met but for the memory test's first `$A11100` row,
which goes to stage 4 with D-2, seven EEPROM boards no document wires and Barver Battle Saga's protection device.
Stage 4's first step, the same day, put the VDP's ports, FIFO and DMA on the slot schedule with the HV counter and
the interrupts placed from Nemesis's tables: VDPFIFOTesting passes 121 of its 122 tests (`Nephrite_Native.md` §13).
Its second drew the picture: planes, the window, sprites with their limits and masking, priority and
shadow/highlight; Nemesis's sprite masking test passes as on his console, VDPFIFOTesting 122 of 122, and 626 of the
corpus's 944 pictures at frame 600 match the reference's up to a colour map (§14 there). Its third added interlace,
PAL and V30, mode 4, the eight-colour mode, CRAM dots, sprites parsed on the line before and MacDonald's window bug,
each measured against Nuked-MD's board through its video pins where the documents are silent (D-4 to D-8); the
slot-stamped renderer is its next step (§15 there).*

About 65 steps, or 190–230 hours (P3). The order is the order of dependence; stage 6 makes the Genesis available to
players before either attachment is started, and the Sega CD (8–10) and the 32X (11–12) are independent of each other
and may swap.

---

## 7. The gates

Nephrite replaces nothing, so its gate is the one to **offer a system to players** (shown on a shelf, chosen by
default for its files), applied per system: the Genesis after stage 6, the Sega CD after stage 10, the 32X after stage
12.

- **G1, the CPU suites.** 100% of both 68000 suites and the Z80 suite, state and cycles, except cases named in a
  disputes entry (the 68000 suite's TAS and TRAPV caveats are the likely ones); ZEXDOC and ZEXALL pass.
- **G2, the self-grading programs.** VDPFIFOTesting, the BCD verifier, the opcode sizes and the illegal test pass,
  each failure a disputes entry; for the Sega CD, mcd-verificator passes.
- **G3, the pictures.** On the test programs whose pictures stand still, the picture equals the references' up to a
  colour map, or the difference is a disputes entry or a named loss of §5.2.
- **G4, the frontend.** The kit's C1–C15 pass on each system's games; battery files round-trip; the fit audit passes;
  the system's cheats decode.
- **G5, the goldens.** Commercial games per system (the tester's library, §9 Q13), each at three anchors, equal to the
  references in picture and RAM, or dispute-logged.
- **G6, speed.** §5.5's budget at 33% quota on the bench set, and full speed on the Legion Go S on battery.
- **G7, the debugger.** Breakpoints, stepping, watches, coverage and the call stack on each processor, each a test.
- **G8, the clean room.** `clonecheck.py` (`VenusRT_Native.md` §4) over Nephrite's and Beryl's sources against
  Genesis Plus GX, PicoDrive, BlastEm and MAME's Sega drivers finds nothing above its threshold, the reference side
  never printed; the disputes log read end to end by someone who did not write it.
- **G9, firmware.** Every game the replacement is claimed to run runs with no firmware folder, and its machine info
  names the replacement; with the player's image, the same game runs on it.

---

## 8. Reuse: the Beryl crates and the framework

### 8.1 The Beryl crates

`EmuSen/Cores/Sega/Beryl - Shared CPUs/`, one crate per processor so that each can be used alone, each with its own
oracle and record page:

| Crate | Users | Oracle |
|---|---|---|
| `beryl-m68k` | Nephrite's Genesis and Sega CD; later the Saturn's sound CPU (a 68EC000), and any machine with a 68000 | SingleStepTests 68000 and TomHarte's 680x0 tests, every transaction |
| `beryl-z80` | Nephrite's sound CPU; later Endou (Master System) and Jadeite (Game Gear) | SingleStepTests Z80, T-state by T-state; ZEXDOC/ZEXALL |
| `beryl-sh2` | Nephrite's 32X; later Zoisite (Saturn) | test programs from the manual; the SH-4 suite's shared instructions |

**The interface the cores use** (built at stage 0 as signatures; `Beryl - Shared CPUs/README.md`):

- **A bus trait per processor**, in the processor's own terms: the 68000's `Access` (address with A0, width, function
  code, locked for TAS) with read, write, idle, interrupt level, acknowledge, address error and the RESET pulse; the
  Z80's fetch with its refresh address, read, write, input, output, idle, INT, NMI and acknowledge; the SH-2's external
  read and write by width, idle, interrupt level and acknowledge.
- **The bus owns the clock.** A processor reports every access and every internal cycle, and the bus advances its own
  time, adding its wait states; the processor keeps no clock. One processor implementation therefore serves a flat test
  bus, the Genesis's bus with its waits and the Sega CD's sub bus alike.
- **The debugger through `emusen-native`'s `Observer`** (added at stage 0 to `debug.rs`): `step_observed` is generic
  over the observer, so the plain step (`Unobserved`) compiles with no hooks at all and the observed one feeds the
  shared `Hooks`: the instruction boundary (breakpoints, stepping, coverage on its processor), stores, calls and
  returns.
- **State** through `StateWriter`/`StateReader`, so each processor's registers are a block in its core's state.
- **A disassembler** per crate for `DEBUG_DISASSEMBLE`, and **an assembler** per crate as a development tool, for the
  test programs and for the firmware replacements of §5.6.

### 8.2 What else the cores could share (a proposal, §9 Q9)

The SN76489 is also the Master System's and the Game Gear's PSG, and the VDP's mode 4 is the Master System's video
mode. Both could become shared crates when Endou starts, as the CPUs are now.

### 8.3 What the framework should gain, found by planning this core

- **Multi-file images** for discs (§5.8), and a decision on streaming reads for large media.
- **A shelf for a console with no C# core.** Mistress's library lists the extensions of `CoreCatalog`'s C# cores only;
  a console reached only through discovery has no shelf until the catalogue reads the system packs (stage 6).
- **The probe's options and memory maps** (§1.6), needed by every libretro reference, not only Nephrite's.
- **Device kinds** (multitap, light gun, mouse) in a later minor (`EmuSen_CoreAPI.md` §6.8).

---

## 9. Questions, decided 2026-10-04

Each question as it was put, and its decision. Q1, Q2, Q4 and Q13 were decided by the tester in their own terms; every
other recommendation this page made was accepted as written.

1. **Q1, may Sega's developer manuals be used as sources?** They are Sega's, circulated without its permission, like
   the SNES Development Manual that VenusRT's Q1 excluded; unlike that case, no public document describes the Sega CD
   BIOS's call interface or the 32X's boot protocol. *Decided:* they may be used, each use cited by manual and section
   (§2.2). The exclusion rules stay: no firmware listing or disassembly is read by a replacement writer (§1.3).
2. **Q2, the references.** *Decided:* Genesis Plus GX, PicoDrive, BlastEm and ClownMDEmu run as local black boxes only,
   never distributed, their sources never read; every other emulator's source stays excluded (§1.3, §1.6).
3. **Q3, the CPU suites' generators.** *Decided as recommended:* the emulator-generated suites are primary oracles,
   with TomHarte's suite, the BCD verifier and ZEXALL as independent checks and every disagreement a disputes entry.
4. **Q4, the Sega CD BIOS replacement's form.** *Decided:* an open 68000 program run by the emulated CPUs (§5.6), the
   documented interface first and the undocumented routines added game by game.
5. **Q5, disc images.** *Decided as recommended:* the system pack parses `.cue` and the host passes the tracks as
   numbered files; whole-image copies at create are accepted; CHD later; no streaming minor until a measurement shows
   the copy matters.
6. **Q6, regions and names.** *Decided as recommended:* a cartridge allowing several markets runs as the Americas, then
   Japan, then Europe, with a Region setting to override; the console row and ES-DE's names stay "Genesis",
   `genesis`, `segacd`, `sega32x`.
7. **Q7, the default model.** *Decided as recommended:* model 1 (YM2612 with its ladder distortion, the model 1 audio
   filter, no TMSS), with model 2 (YM3438, TMSS) as a setting.
8. **Q8, `.md` and `.bin`.** *Decided as recommended:* both claimed, the system decided by the contents. The tester's
   library bears it out: 946 of its 948 files are `.bin`.
9. **Q9, shared sound and video crates.** *Decided as recommended:* the SN76489 and mode 4 become shared crates when
   Endou and Jadeite start; whether they join Beryl or take names of their own is settled then.
10. **Q10, the default pad.** *Decided as recommended:* the 3-button pad, the 6-button pad per port as a setting.
11. **Q11, the speed budget and its levers.** *Decided as recommended:* §5.5's budget per system; for the 32X, the
    exact levers before any accuracy setting.
12. **Q12, the special cartridges.** *Decided as recommended:* SRAM, EEPROM, SSF2, lock-on, the J-Cart and both
    multitaps for the Genesis gate; the SVP (with an open replacement for its internal ROM), Sega Channel, the light
    guns, the mouse and the Pico after it.
13. **Q13, commercial games.** *Decided:* the tester's Genesis library is `AppSettings.RomDirectory`'s `genesis/`
    (948 files), read only and copied to scratch for every use, with a copy at
    `~/.cache/emusen/probe/nephrite/games/genesis/`. No Sega CD or 32X games yet; their goldens wait for them.
14. **Q14, Sega's own test programs.** *Decided as recommended:* used if the tester supplies them locally, never
    fetched by the project.
15. **Q15, the border.** *Decided as recommended:* active display only, the border a later setting.

---

## 10. Negative results

- No AccuracyCoin-like suite for the Genesis; no homebrew 32X hardware test; no SH-2 single-step suite; no
  Genesis-hosted ZEXALL; no 68000 cycle-timing test ROM; no PSG test ROM (§3.1).
- No libretro reference runs the Sega CD without a BIOS except ClownMDEmu, whose replacement did not get past the 240p
  suite's loading screen (§1.6).
- No reference exposes VSRAM or the VDP's registers through the standard calls (§1.6).
- The Legion Go S and the laptop were not measured; §5.5's machine factors are VenusRT's.

## 11. Predictions to be retired

- **P1.** The Genesis's desktop mean is at most 1.5 ms a frame on the bench games at the end of stage 5.
- **P2.** The 32X misses §5.5's budget with the interpreter alone, and meets it with the exact levers of stage 13.
- **P3.** The whole is about 65 steps; the VDP, the Sega CD BIOS and the SH-2 are the stages most likely to overrun.
- **P4.** Stage 1 passes both 68000 suites' state on every case before any bus exists, with the transactions on all
  but TAS's and TRAPV's caveats.
- **P5.** A Z80 slice bound exists that passes the Z80-window tests and costs under 0.2 ms a frame.
- **P6.** At least half the disputes entries fall in §2.3's thin areas, and most are the VDP's.
- **P7.** The documented Sega CD BIOS interface takes about 6 steps, and at least one commercial disc needs an
  undocumented routine.
- **P8.** The skip-versus-draw state test passes on every bench game from stage 4 on.
