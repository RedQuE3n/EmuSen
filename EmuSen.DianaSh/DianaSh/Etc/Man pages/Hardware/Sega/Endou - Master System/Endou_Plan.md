# Endou_Plan — a clean-room Master System core in Rust

*Written 2026-10-19, at stage 0.* This page plans Endou, EmuSen's Sega Master System core: the Mark III and the
Master System in their Japanese and export forms, NTSC and PAL, with the SG-1000's cartridges, written in Rust from
hardware documentation, graded by test programs and loaded through the core ABI v1 (`EmuSen_CoreAPI.md`,
`EmuSen_NativeCores.md`). Like Nephrite and Diamond it has no predecessor: there is no C# Master System core to port or
to reach parity with, so the page follows `Nephrite_Plan.md`'s shape, with one section added for the Game Gear (§2),
which shares most of the machine. No code is written at stage 0.

**Decided by the tester, 2026-10-19:**

- The Master System is the next new console, ahead of the PlayStation 2. The plan is written now, beside the backend's
  port to Rust; building starts only on the tester's go.
- The core is named **Endou**. Its folders, `EmuSen/Cores/Sega/Endou - Master System/` and this one, exist with a
  README each. Endymion, the name first reserved, is now the SDL crate's (`EmuSen_Core_Naming_Scheme.md` §3, §10).

**Decided by the tester, 2026-10-19, after the plan was read:** every question of §10 as recommended, with one change:
**no real hardware is used.** The tester's consoles are out of use for their age and fragility, so nothing is run on a
Master System, a Game Gear, an SG-1000 or a flash cartridge; disputes such as D-1 (the colour levels) and the VDP's
access slots are settled from documents, test programs and black-box referees only (§10, Q11). The go is for stage 1
alone; each later stage waits for its own.

**Decided by the tester, 2026-10-19, on stage 1's questions** (`Endou_Native.md` §8), each as recommended: the core
ABI minor that lets a host set `system` beside an image is accepted, to be built as its own piece of work before
stage 4 (§3.7); MAME is not installed before stage 3; a VHDL simulator for SMS_MiSTer is installed only when the
first dispute needs it, and only after asking the tester; the copier's 512-byte header is confirmed against a
document at stage 2. The go for stage 2 came with them.

**Decided by the tester, 2026-10-19, on stage 2's questions** (`Endou_Native.md` §9.15), each as recommended: at
stage 7 a public per-game listing may be fetched to complete the battery table; D-3 (the Mark III's TH) and D-4 (port
`$3E`) stay as argued from the documents until a game shows otherwise, TH returning at the Light Phaser's stage; CI's
testing of the Beryl crates with all their features is confirmed. The go for stage 3 came with them.

**Decided by the tester, 2026-10-19, on stage 3's questions** (`Endou_Native.md` §10.12), each as recommended: D-1's
boosted blue is taken for the 315-5124, marked as read from a document; a VHDL simulator is approved for SMS_MiSTer as
a black-box referee, installed by the tester, its RTL read only in a logged dispute step; an (E) image runs in PAL only
by the region setting, with no per-game table; MAME is not used, as a reference or a referee; and under core ABI 1.15
MercuryRT and Nephrite keep their own system rules, routing being left to a separate piece of work. The go for stage 4
came with them.

**Decided by the tester, 2026-10-19, on stage 4's questions** (`Endou_Native.md` §11.15), each as recommended: a
Nephrite dispute step asks the board about the 315-5313's right-column lock the next time Nephrite is worked on; a
logged dispute step may write stand-ins for SMS_MiSTer's `altera_mf` memory blocks so that it runs under GHDL as a
black-box referee for D-6, D-7, D-8 and Space Harrier's opening, reading only the entity interfaces the stand-ins need,
optional within stage 5; the SG-1000 stays the default model for SG-1000 games, the Mark III offered; and the SG-1000
gets its own shelf (stage 7). The go for stage 5 came with them.

**Decided by the tester, 2026-10-19, on stage 5's questions** (`Endou_Native.md` §12.14), all four: a published
recording of a Master System's audio output may be searched for and fetched, and the analogue filter fitted only from
it, else left a named loss; SMS_MiSTer's `vdp` entity may be run alone under GHDL with stand-ins for its `altera_mf`
memories in a logged dispute step for D-8, reading only that entity's port list and those components' interfaces; the
go for stage 6, opened by `Endou_OpenBoot.md`; and the player's own BIOS is an opt-in accuracy upgrade per console
model, the open boot program the default (§5.8, amended).

**Decided by the tester, 2026-10-19, on stage 6's questions** (`Endou_Native.md` §13.14), all as recommended: the
documented export check stays, the four library images it refuses playing on the Mark III, the refusal's words saying
so; a public BIOS checksum list (No-Intro's DAT or SMS Power!'s `BIOS` page) may be fetched to cite each model's BIOS
MD5 and SHA-1, only its checksum table read and its provenance recorded, the clean room holding; BIOS images with a
built-in game stay refused, for a later stage; `firmwarecheck.py` is measured with a control at stage 7; D-8's referee
(nvc) is set up apart, and D-8 is not worked on at stage 7; and the go for stage 7.

**Decided by the tester, 2026-10-19, on stage 7's questions** (`Endou_Native.md` §14.14), all as recommended: the
Controller Bindings window's player selector stays below the drawing; LunaP gains the Master System's pad and the
SG-1000's joystick as drawings of their own, a separate piece of work; the cover aspects stay at 1.4 until measured, the
Genesis's 0.7 checked in a Nephrite step; the export check refusing a Game Genie-patched game stays; and the go for
stage 8.

Every claim carries one of six marks:

- **[M]** *measured* on 2026-10-19, with the tool named. Nothing was run to write this page beyond listings, the local
  cheat database and OpenVGDB's table; the core's measurements begin at stage 1.
- **[R]** *read* from a source, named, on 2026-10-19.
- **[D]** *stated as the public documents of §6 describe it*, from a reading made before this page and not repeated
  for it. The stage that builds the fact cites the document's section, and a fact found to read otherwise is corrected
  in place, struck through, with the date.
- **[A]** *argued*: reasoning from the documents or from other cores' measurements, with no measurement behind it here.
- **[C]** *a choice*: a rule set for this core, said so, or a recommendation, each of §10's decided 2026-10-19.
- **P1–P10** *predicted*, numbered in §12 so that a later stage can retire each.

---

## 1. Purpose, scope and the clean-room protocol

### 1.1 Which machines, and why one core

Sega's 8-bit consoles are one lineage. The SG-1000 (1983) carries a Z80, a Texas Instruments TMS9918A and a discrete
SN76489AN [R SMS Power's "SN76489", "SN76489 sightings"]; the Mark III (1985) replaces the video chip with Sega's
315-5124, which keeps the TMS9918's modes and adds mode 4, and moves the PSG into it [R MacDonald, `msvdp.txt` §1, §6];
the Master System is the Mark III re-cased for export, then given the 315-5246 in the Master System II; the Japanese
Master System adds the YM2413 FM unit that the Mark III took as an add-on [D]. A Mark III or Japanese Master System runs
SG-1000 cartridges through the VDP's TMS9918 modes [D].

One core therefore runs three systems: **`sms`** (the Master System and the Mark III, one system since their
cartridges are the same images), **`sg1000`** (the SG-1000's cartridges, on a modelled SG-1000 by default, §1.3) and,
as a follow-on, **`gg`** (§2). The models within a system are a setting (§5.8), each a real console.

### 1.2 Hardware scope

The target for each part is stated in hardware terms; "exact" means bit-identical results at the cycle at which a
result is visible to software.

| Part | In scope | Target |
|---|---|---|
| **Z80** at the master clock over 15 (3.579545 MHz NTSC, 3.546895 MHz PAL) [D] | Beryl's `beryl-z80` unchanged: every opcode with WZ, Q and P, NMI, IM 1 [R `Beryl_Z80.md` §4, §5] | Exact, every T-state on the bus |
| **Memory and I/O decoding** | 8 KiB RAM at `$C000` mirrored to `$FFFF`; the memory control port `$3E` (BIOS, cartridge, card, expansion, RAM and I/O enables); the I/O control port `$3F` (TH and TR directions and levels, the export consoles' read-back that tells a region); the controller ports `$DC`/`$DD`; the partial decoding and its mirrors, which differ between the 315-5124 and 315-5246 boards [D] | Exact, mirrors included where a test program pins them |
| **VDP, mode 4** (315-5124 and 315-5246) | The ports, the address and code registers, the read buffer; VRAM (16 KiB) and CRAM (32 bytes); registers 0–10; the name table with the scroll locks and the left column's mask; sprites (64, eight a line, 8×8 or 8×16, zoom, the early clock), the overflow and collision flags; the 224- and 240-line modes of the 315-5246; the frame and line interrupts with the line counter; the V and H counters and the H counter's latch by TH; NTSC and PAL timing; the CPU's access slots, where too-fast writes are lost [R `msvdp.txt` §15] | Results exact; timing to the dot where test programs measure it (§5.2) |
| **VDP, the TMS9918 modes** | Graphic I and II, Text, Multicolor and the undocumented combinations [R `msvdp.txt` §6], with the 315-5246's hardwired palette and the 315-5124's behaviour in them; and, for the SG-1000, the **TMS9918A** itself (its palette, 32 sprites with four a line and the fifth-sprite flag) [D] | Exact in results; the TMS9918A's timing where its data manual gives it |
| **PSG** | Beryl's `beryl-sn76489`: Sega's variant on the Mark III and Master System, the discrete chip on the SG-1000 [R `Beryl_SN76489.md` §2; SMS Power's "SN76489"] | Exact to the clock; the analogue stage as measured (§5.4) |
| **YM2413 (OPLL)** | Nine two-operator channels, the fifteen fixed instruments and the user instrument, the rhythm mode, the ports `$F0`–`$F2` and the Japanese console's audio control; on the Japanese Master System and on the Mark III with the FM Sound Unit [D] | Exact to the sample (§5.4); **stage 9**, after the gate (§10 Q2) |
| **The Pause button** | The console's button, an NMI edge to the Z80 [D] | Exact: the edge at the instruction boundary `beryl-z80` takes it [R `Beryl_Z80.md` §5] |
| **The Reset button** | The Master System's button, a bit software reads at `$DD` (no Master System II has one) [D] | Exact |
| **Cartridges** | Plain ROM to 48 KiB; Sega's mapper (315-5235: `$FFFC`–`$FFFF`, the first kilobyte fixed, cartridge RAM in two banks); Codemasters' mapper; the SG-1000's RAM boards; then the Korean and multicart boards (stage 12) [D] | Exact; each board as a document describes it |
| **The card slot** | Mapped by `$3E` and empty; a card's image runs from the cartridge slot (§1.3) [C] | — |
| **The BIOS** | The player's image when present; else Endou's open boot program (§5.5) | The player's image exact; the open program's cost stated |
| **Regions** | Japan (Mark III, Japanese Master System), the Americas and Brazil (NTSC), Europe (PAL); the I/O control port's region behaviour; the header's region code [D] | Exact per model |
| **Peripherals** | The control pad at the gate; the Paddle, the Sports Pad and the 3-D glasses at stage 10; the Light Phaser at stage 11 (§1.3) | Protocol exact; the light gun's latch to the dot |

### 1.3 What is in and out, and why

| Item | Recommended [C] | Why |
|---|---|---|
| Master System and Mark III, Japanese and export, NTSC and PAL | **In**, from stage 2 | The console's core; the models differ by a variant of a shared crate or a decode table, not by code paths of their own [A] |
| The TMS9918 modes | **In**, stage 4 | The VDP has them on every model but the Genesis [R `msvdp.txt` §6, §13]; a commercial Master System game uses one (MacDonald names it "F-15 Fighting Falcon", §13) [R]; the SG-1000's cartridges need them |
| The SG-1000 as a system, on a modelled SG-1000 | **In**, stage 4 (§10 Q8) | Its 76 cartridges and 29 My Cards [R Art Book Next's `sg-1000.xml`] were made for a TMS9918A and a discrete PSG; running them on a Mark III is a second real console, offered as a model. Its cost is a TMS9918A variant of the VDP crate and the discrete PSG variant, which exists [R `Beryl_SN76489.md` §2] |
| The SC-3000 and its keyboard, the Othello Multivision | **Out** | A computer with a keyboard device kind the ABI lacks [R `EmuSen_CoreAPI_Extensions.md` §10.1]; a clone |
| The Pause button's NMI and the Reset button | **In**, stage 2 | Most games read one or both [A] |
| The YM2413 | **A stage directly after the gate** (stage 9; §10 Q2) | Until it lands, a Japanese cartridge runs on a Mark III without the FM Sound Unit, which is a real console, so the default stays accurate; the stage then adds the Japanese Master System and the Mark III with the unit |
| The control pad | **In**, stage 2 | — |
| The Paddle and the Sports Pad | **Stage 10** | Both fit today's ABI as axes behind a port setting [R `EmuSen_CoreAPI.md` §6.8; `EmuSen_CoreAPI_Extensions.md` §10.1]; a handful of games each [A] |
| The 3-D glasses | **Stage 10** | Not input: the game writes the shutter's state, and the picture alternates eyes; a display setting chooses how a player sees it (§5.8) |
| The Light Phaser | **Stage 11** | It needs a screen position, which v1 has no call for: `set_pointer` behind a bit and a controller kind, the form `EmuSen_CoreAPI_Extensions.md` §10.1 recorded and did not propose. Endou would be its first user, so the minor is proposed with the stage, for the tester's approval (§3.6) |
| The Handle Controller, the Graphic Board, the keyboard | **Out** | Japan-only devices with one or two titles each, or a device kind the ABI lacks [A] |
| Sega's mapper, Codemasters', plain ROM, the SG-1000's RAM boards, cartridge RAM | **In**, stage 2 | Most of the library [A] |
| The Korean boards and multicarts | **Stage 12** | Each a small mapper; identified by a table, since no header names them (§5.6) |
| The card slot | **Mapped, empty; cards run from the cartridge slot** | A Sega Card's dump is a ROM image like a cartridge's, and only a BIOS's media selection tells the slots apart [A]; a separate slot would need a second image, which v1 passes only as numbered files |
| The BIOS's built-in games (Hang-On, Alex Kidd in Miracle World, the hidden maze and the rest) | **With the player's image only** | They are the firmware's own code; the open boot program carries no game. A console started empty (`starts_empty`, `EmuSen_CoreAPI.md` §6.31) runs a player's BIOS into its built-in game [R §6.31: "a console starting into its firmware with an empty cartridge slot"] |
| The border | **Active display only**, the border a later setting | As Nephrite's Q15 decided [R `Nephrite_Plan.md` §9] |
| The Game Gear | **A follow-on**, §2 | Planned now, built after the Master System's gate as a system of Endou's library (§10 Q1, decided 2026-10-19) |

### 1.4 Who may read what

*Set 2026-10-19 [C]:* the clean-room rules of `Nephrite_Plan.md` §1.3, which bind the Sega cores, unchanged in their
order of recourse and widened only by naming this console's sources. A **writer** writes Endou's or a shared crate's
code, tests or design pages; a **replacement writer** writes or reviews the open boot program (§5.5); a **dispute step**
is a separate, supervised step whose only product is an entry in `Endou_Disputes.md`.

| Source | A writer | A replacement writer | A dispute step | Notes |
|---|---|---|---|---|
| Hardware documents (§6): MacDonald's, SMS Power's development pages, Zilog's and Texas Instruments' manuals, Young's TMS9918 and Z80 documents, Yamaha's YM2413 manual, the public research write-ups | yes | yes, but no page that lists or disassembles a BIOS | yes | The primary sources |
| Sega's developer documents (the Mark III and Game Gear manuals, if they are found) | yes, each use cited by manual and section, as Nephrite's Q1 decided for Sega's documents [R `Nephrite_Plan.md` §9 Q1] | the same | yes | |
| A listing or disassembly of any firmware (a Master System or Game Gear BIOS), wherever published | per the general rules | **no** | no | Marked "excluded for replacement writers" in the corpus's `DOCS.txt` |
| Test programs, their sources and READMEs | yes | yes, unless the source is a BIOS listing | yes | A test's source says what it expects |
| Reference emulators **run** through the probe (§1.7) | outputs only | outputs of games, never a trace of a BIOS | yes | Memory, picture, sound |
| Nuked-MD's board **run** as a black box (§1.6) | outputs only, each finding a disputes entry | no | yes, as Nephrite's dispute steps ran it | Evidence for the 315-5313's mode 4 only |
| SMS_MiSTer's RTL (§1.6) | **no** | **no** | after the documents and a test program, each file logged; never transcribed | By Nephrite's rule for the Sega cores (§10 Q6, decided 2026-10-19) |
| Any emulator's **source**: Genesis Plus GX, SMS Plus and SMS Plus GX, PicoDrive, BlastEm, Gearsystem, MEKA, Emulicious, Kega Fusion, MAME's Sega 8-bit and TMS9918 drivers, Mesen 2, Ares and higan, openMSX, blueMSX, emu2413, Nuked-OPLL's code, ymfm and the rest | **no** | **no** | **no** | Not in a dispute step either |
| Firmware images (the BIOS, the Game Gear's boot ROM) | **no**: not read, disassembled or traced | **no** | **no** | §5.5 |

**The replacement writers' rule** is Nephrite's: whoever writes or reviews the open boot program reads no listing,
disassembly or trace of a Master System or Game Gear BIOS in any form. The program is written from the documents'
account of what the BIOS leaves behind (§5.5), and the player's image is run beside it only as a black box, comparing
games' behaviour, never the firmware's bytes [R `Nephrite_Plan.md` §1.3, §5.6].

The three rules follow as Nephrite's do: the order of recourse is the documents, then a test program written for the
purpose, then the referee, and never an emulator's source; nothing read in a dispute step is typed into code in that
step; the code carries no emulator's identifiers, which §8's G8 checks.

### 1.5 The disputes log

`Endou_Disputes.md`, opened with this page because reading for it found three entries, D-0 to D-2, in the shape
`Nephrite_Disputes.md` keeps. A dispute in a Beryl crate goes to that crate's record page, since it belongs to every
core that uses the crate.

### 1.6 The referees, run before they are read

| Referee | Where | What it is | What its agreement is worth [A] |
|---|---|---|---|
| **SMS_MiSTer** | `github.com/MiSTer-devel/SMS_MiSTer`, not cloned; to `~/Projects/sms-mister-reference` when a dispute first needs it | An FPGA Master System and Game Gear; what its folders hold is read from its README and directory names at the clone | Its VDP is its author's reading, since no transcription of the 315-5124 from die images is known to this page; its Z80 is likely T80, the general core that `Mercury_Referee.md` §0 weighed as weak in the Game Boy's referee, as `Moon_Native.md` weighed T65 in the NES's. Moderate for the VDP, weak for the Z80. It is a referee, never a grader |
| **Nuked-MD's board** | `~/.cache/emusen/probe/nukedmd-board`, built for Nephrite with Verilator [R `Nephrite_Native.md` §11's account of `md_board` and `mdboard.py`] | The Genesis transcribed from die images, its VDP (`ym7101.v`) the 315-5313 [R `Nephrite_Plan.md` §1.5] | **Strong** for the 315-5313's mode 4, **weak** for the 315-5124 and 315-5246 wherever MacDonald lists a difference (§3.3) [R `msvdp.txt` §13] |

**How a referee is run.** First as a black box: SMS_MiSTer under Verilator or GHDL, its simulation's scripts and top
level's port list read to learn how to feed it a program, as Diamond's FPGA referees were run [R `Diamond_Plan.md`
§1.5]; the board as Nephrite's dispute steps ran it. Only then, and only in a dispute step under Nephrite's rule, a file
of SMS_MiSTer's RTL, each file and its lines logged in the entry. Which rule binds was §10's Q6: Nephrite's plan is clear
for the Sega cores, but Diamond's decision of 2026-10-11 says "of each FPGA core", which could be read as general.
*Decided 2026-10-19:* Nephrite's rule binds Endou, the core run as a black box first.

### 1.7 The black-box references

Three libretro cores in the probe's cache already run the Master System, the Game Gear and the SG-1000, each pinned by
hash in `EmuSen.WiseMan/Reference/libretro-cores.sha256` for the Genesis [R]:

| Core | Pinned | Licence [R its `.info`] | Extensions it declares [R] |
|---|---|---|---|
| Genesis Plus GX | v1.7.4 `58c3414`, 2026-10-04 | non-commercial | `sms`, `bms`, `gg`, `sg` among the Genesis's |
| PicoDrive | 2.05 `1890c29`, 2026-10-04 | MAME (non-commercial) | `sms`, `gg`, `sg`, `sc` |
| BlastEm | 1.0.1-pre, 2026-10-04 | GPLv3 | `sms`, `gg`, `sg`, `sg1`, `sc`, `sc3`, `sf7` |

Genesis Plus GX lists the Master System's BIOSes (`bios_E.sms`, `bios_U.sms`, `bios_J.sms`) and the Game Gear's
(`bios.gg`) as optional, with no checksums [R its `.info`]. Whether each runs every model, which memory it exposes and
what it costs are measured at stage 1, as Nephrite measured its four [A]. Further references, fetched at stage 1 on
the tester's go (§10 Q4, decided 2026-10-19) and listed in `Endou_References.md` §3: **Gearsystem** and **SMS Plus GX** (libretro cores), ~~**MAME**
(its Sega 8-bit drivers, run headless with a Lua script reading memory)~~ (*decided 2026-10-19: MAME is not used*), **Mesen 2** (through the probe's Mesen
backend, whose patch would gain the console) and **Emulicious** (a desktop program with no headless mode, so a
reference only by the tester's own runs). Every reference is compared, not trusted; their disagreement is the evidence
that a dispute is needed.

### 1.8 What was read to write this page

No emulator source of any kind, and no firmware or firmware listing. Read: `Nephrite_Plan.md`, `Nephrite_OpenBios.md`'s
contents and opening, `Nephrite_Disputes.md`'s rules and D-7, `Nephrite_Native.md` §11 (the board), §15 (mode 4) and §18 (the PSG),
`Nephrite_OperatorTables.md` §4; `Diamond_Plan.md` §1 and §7–§8, `Diamond_References.md` §1–§2; `Mercury_Referee.md`
§0 and `Moon_Native.md`'s referee table, for the borrowed CPU cores; `Beryl-HW/README.md`, `Beryl_Z80.md`,
`Beryl_SN76489.md`; Nephrite's mode 4 code (`render.rs`'s `render_mode4`, `vdp.rs`'s mode 4 paths); `EmuSen_CoreAPI.md`
§6.2, §6.8, §6.31, §6.32, §12.1 and §27; `EmuSen_CoreAPI_Extensions.md` §10.1; `EmuSen_Firmware.md` §0;
`EmuSen_Core_Naming_Scheme.md` §3 and §10; `EmuSen_Backend.md` §5, §12.1, §13.3, §13.5, §27.2 and §27.11; the systems
crate's `lib.rs`, `probe.rs`'s `route` and the data files; `RustCores.props`; WiseMan's `SyntheticRom.cs` and
`NephriteTests.cs`'s use of `SyntheticMdRom`. From the corpus Nephrite fetched: MacDonald's `msvdp.txt` §1, §4–§6 and
§13–§15, and SMS Power's "SN76489" (its "sightings" and LFSR sections); `DOCS.txt`'s entries for both. The libretro
cores' `.info` files; OpenVGDB's `SYSTEMS` table; the libretro cheat database's folder list and its codes' shapes;
Art Book Next's system metadata files.

---

## 2. The Game Gear (Jadeite)

### 2.1 What the Game Gear adds

The Game Gear is a Master System II with a different video chip and I/O, run on batteries [A from `msvdp.txt` §14: "The
GameGear I have behaves like a NTSC SMS2"]:

| | Master System II | Game Gear | Where it lands |
|---|---|---|---|
| CPU, RAM, mapper | Z80, 8 KiB, Sega's | the same [D] | unchanged |
| VDP | 315-5246 | 315-5378 [R `msvdp.txt` §1, its note 1 doubting the part] | a variant of the VDP crate |
| Colours | 32 bytes, `--BBGGRR` | in GG mode 32 words, `----BBBBGGGGRRRR`, written through a latch at even addresses [R §5] | the variant |
| Picture | 256 × 192 (224, 240) | in GG mode a fixed 160 × 144 window of the picture; in SMS mode the picture scaled onto the LCD with "some rows skipped" [R §14] | the variant, and machine info's size |
| TMS9918 modes | hardwired palette | colours 16–31 of CRAM [R §14] | the variant |
| Pause | NMI from the console | the Start button at port `$00`, read by software in GG mode [D] | Endou's I/O, a model |
| PSG | mono | stereo through port `$06` [R `msvdp.txt` §1 note 3; `Beryl_SN76489.md` §2] | already in `beryl-sn76489` |
| Link | — | Gear-to-Gear, ports `$01`–`$05` [D] | out at first: two machines (§2.4) |
| Cartridges | — | the same mappers; a few boards with a serial EEPROM [D] | Endou's boards, one more |
| Firmware | BIOS | an optional boot ROM on some units (`bios.gg` [R Genesis Plus GX's `.info`]) | an open replacement (§5.5) |
| FM, light gun, 3-D | — | none [D] | — |

### 2.2 The two forms, weighed

- **(a) A system of Endou's library**, as `gbc` is of MercuryRT's and `mcd` and `32x` of Nephrite's [R `RustCores.props`;
  `EmuSen_Backend.md` §12.1]. One crate `endou` reports `sms`, `sg1000` and `gg`; the Game Gear's I/O, its LCD window
  and its scaling are a module of it, its VDP a variant of the shared crate. **Cost** [A]: about 900–1,700 lines (§9),
  nothing duplicated. The kit's rule that a system in development shares no extension with an offered one holds,
  `.gg` being the Game Gear's alone [R `EmuSen_CoreAPI.md` §27.4], so the Game Gear can be built behind its own
  development mark after the Master System is offered.
- **(b) A thin core of its own**, `jadeite`, over shared crates. Everything Endou has above the chips (the bus and its
  decoding, the mappers and their table, the scheduler, the state, the v1 glue, the settings, the battery files, the
  kit sample) would then be needed twice, or moved into further Beryl crates whose only users are these two cores.
  **Cost** [A]: 2,500–4,000 lines more than (a), or the same split into crates that buy no separability anyone uses,
  which the pruning test refuses [R `EmuSen_Backend.md` §5.2, citing `EmuSen_Multicore.md` §9.2].

**Recommended [C]: (a).** The Game Gear differs from the Master System II as the Game Boy Color differs from the Game
Boy, by variants of the same chips and a few registers; the precedent of one library per lineage is MercuryRT's. The
name Jadeite then names the Game Gear's module and its record pages, which stay in `Jadeite - Game Gear/` as the
naming scheme placed them; the engine a player sees is Endou. *Decided 2026-10-19 (§10 Q1):* (a), the name Jadeite
kept for the module and its pages.

### 2.3 What is built now for it, and what waits

The shared crates are built with the Game Gear's variants named from the start (the VDP crate's `Variant::GG` with its
CRAM and window; `beryl-sn76489` already has the stereo register), so nothing is rebuilt when the Game Gear starts.
Nothing of the Game Gear's own is built before the Master System's gate (§8, G4).

### 2.4 Its stages, planned as a follow-on

J1 to J3 (§7): the VDP's GG mode, the window and SMS mode's scaling; the I/O, the EEPROM boards and the open boot
replacement; the Game Gear to players. **Gear-to-Gear is out** at first: it joins two machines, which the ABI has no
form for; a later netplay-like design would decide it, and few games need it to be played [A]. *Added 2026-10-20:* JL,
the Game Gear's own LCD model (§2.5), before J3. *J1 built 2026-10-20:* `Hardware/Sega/Jadeite - Game Gear/Jadeite_Native.md`.

### 2.5 The LCD model (JL), planned 2026-10-20

*Set by the tester's rule of 2026-10-20:* a handheld needs an accurate model of its own LCD, its panel's response, its
grid at the display's size and its colours, from documents and measurements, not a borrowed or fitted filter; the rule
of `EmuSen_CRT.md` §31, every console an accurate display of its own, which `ConsoleScreens` and
`ConsoleScreenRuleTests` enforce, and which the Game Boy's and Game Boy Color's filters, fits to measurements of their
panels, were judged on 2026-10-20 not to meet: each carries a dated gap (`EmuSen_CRT.md` §31.7, Q2). The Game Gear is
not given such a gap; it gets the model before it is offered, so JL comes before J3, and `ConsoleScreenRuleTests` will
demand a row for `gg` once J3 offers it.

**What it is.** A stage of research and design first, as the CRT's was (`EmuSen_CRT.md`): a page, `Jadeite_Lcd.md`,
that gathers what is published about the panel (Citizen's UC-320, a 3.2-inch colour STN panel of 160 by 146 dots with a
cold-cathode backlight [R SMS Power!'s "Game Gear LCD"]), its colour elements' order and shape, its response in time (the
smear of a moving picture), its gamut, gamma and contrast under the backlight, and the published photographs and
captures of units; then the model, written from that page alone, as a profile of the screen filters with a row in
`ConsoleScreens`. What Endou draws stays the console's output to the panel (`Jadeite_Native.md` §3); the panel's
behaviour is the model's.

**Its gate.** Each property of the model traced to a document or a published measurement, or named as argued; the model
against published photographs and captures of a Game Gear showing known pictures, GG mode and a Master System
cartridge's mode (the photographs of SMS Power!'s forum topic 9562 among them), by a measure written down before the
comparison; its row in `ConsoleScreens` with no gap; the fit audit at both sizes; its cost on the handheld within the
frame; seeded faults.

**Questions it raises** (for the tester before JL starts):

1. **Measurements.** No console is run or captured for the project (§10 Q11). May published photographs and videos of
   units be used as measurements, and which, since none was made for the purpose? Recommended: yes, each cited with its
   source and date as a document is, and weighed as evidence of one unit. **[decision]**
2. **Which panel.** Game Gears were made over years and are often repaired with replacement screens. Recommended: the
   original panel only, as Sega shipped it, a replacement panel being another display. **[decision]**
3. **One model for the three handhelds.** The Game Boy's and Game Boy Color's dated gaps ask for the same kind of model.
   Recommended: JL builds the model as one component with a profile per panel, the Game Gear's first, and the two Game
   Boys follow as a separate step on the same component, so the three are not three filters. **[decision]**
4. **Tiers.** The CRT has Performance, Balanced and Accurate. Recommended: Accurate first, then cheaper tiers only if the
   handheld's frame budget asks for them, as for the CRT. **[routine]**
5. **A Master System cartridge's frame of 146 rows** (`Jadeite_Native.md` §8, Q1): the model draws the panel's 146 rows
   in both modes, which settles it. **[routine]**

---

## 3. Reuse

### 3.1 From Beryl, unchanged

- **`beryl-z80`**: the processor whole, its `Bus` trait (fetch with refresh, read, write, input, output, idle, INT, NMI,
  acknowledge), its state block, its disassembler and its observer step [R `Beryl_Z80.md` §2, §5, §6]. Its interrupt
  modes cover the Master System's: mode 1 and the NMI [R §5: "the Genesis, the Master System and the Game Gear all run
  mode 1"]. Its measured cost, 0.038 ms for a frame of Z80 work at 3.58 MHz, is the same work Endou's CPU does
  [R §6.4].
- **`beryl-sn76489`**: `Variant::SEGA` for the Mark III onward, `Variant::DISCRETE` for the SG-1000, the stereo register
  for the Game Gear [R `Beryl_SN76489.md` §2]. The crate keeps the contract of a chip that is not a processor: the
  caller owns the clock, writes are the port, output is levels [R `Beryl-HW/README.md`].

### 3.2 What Beryl gains

- **`beryl-z80`'s assembler**, behind an `asm` feature as `beryl-m68k`'s is [R Nephrite's `Cargo.toml`], which every
  Beryl crate was to carry as a development tool [R `Beryl-HW/README.md`] and the Z80 crate does not yet [R
  `Beryl_Z80.md`, no assembler recorded]. It assembles Endou's test programs and the open boot program at build time.
  Stage 2. *Built 2026-10-19 (`Beryl_Z80.md` §8), its front half shared with the 68000's in `beryl-asm`.*
- **`beryl-sms-vdp`**, the VDP family as one chip crate (§3.3). Stages 3 and 4. *Mode 4 built 2026-10-19
  (`Beryl_SmsVdp.md`, `Endou_Native.md` §10); the TMS9918's modes, the TMS9918A and the Genesis's variant the same day
  at stage 4 (`Endou_Native.md` §11).*
- **Conditionally, the operator ROMs.** If D-2 finds that the YM2413's log-sine and exponent ROMs are the OPL2's, as
  they are for the OPN2 on Nuked-MD's board [R `Nephrite_OperatorTables.md` §4], the two tables Nephrite holds in
  `fm_tables.rs` [R] would be needed by two Sega cores, and move to a Beryl crate of their own with their provenance
  page, so that one owner holds them. If they differ, nothing moves. Stage 9.

### 3.3 The VDP crate, and Nephrite's mode 4

*Nephrite's Q9, decided 2026-10-04:* mode 4 becomes a shared crate when Endou starts, its place settled then [R
`Nephrite_Plan.md` §9 Q9]; *decided 2026-10-05:* crates shared between Sega cores live in Beryl [R `Beryl-HW/README.md`].
This section settles what is shared.

**Is Nephrite's mode 4 the same logic?** In part. Nephrite draws mode 4 a line at a time, about a hundred lines of
`render_mode4`: the sprite pass (the end marker `$D0`, eight a line, overflow, collision, tall sprites, the early clock),
the background (the name table entry's flips, palette and priority, the top rows' and right columns' scroll locks), and
the left column's mask [R `render.rs`]. These are the rules of MacDonald's §7–§10 for every VDP of the family. Where
the 315-5313 differs, MacDonald lists it [R `msvdp.txt` §13]: no TMS9918 modes (black), no extended heights, no sprite
zoom, no name-table mirroring by register 2's bit 0, CRAM written when the code register's bit 1 is set, the address
register's low byte latched until the second byte, and a data write that does not refill the read buffer. And mode 4's
16 KiB lie in the Genesis's 64 by a map the board measured, its colours shown at the board's levels (Nephrite's D-7)
[R]. The ports, the timing and the interrupts of the two consoles differ wholly: Nephrite's mode 4 runs inside a VDP of
FIFO and slots [R `Nephrite_Native.md` §13].

So the shared part is the **composition of a line**, not the chip. Recommended [C]:

- **One crate, `beryl-sms-vdp`**, holding the 315-5124/5246/5378 family's chip (ports, registers, counters, interrupts,
  access timing, the renderer) and, as a public module of it, `compose`: the line composition with a `Variant`
  (`SMS1`, `SMS2`, `GG`, `TMS9918A`, `GENESIS`) carrying the differences above. Splitting the module into a crate of its
  own would buy Nephrite nothing it uses apart, by the pruning test [A].
- **Its contract** is the chip contract of `Beryl-HW/README.md`, with one addition: the caller owns the clock and the
  write times; VRAM is read through a trait, so the Genesis keeps its D-7 map and its 64 KiB; the crate writes **CRAM
  indices**, never colours, into a caller's buffer, so the console's DAC levels stay the console's (the Genesis's
  measured, the Master System's D-1, the Game Gear's 12 bits); it allocates nothing per line.
- **Nephrite moves onto it at stage 4** with `Variant::GENESIS`, its `render_mode4` deleted. The move's gate is
  Nephrite's: D-7's two tests unchanged (`mode_4_writes_land_where_the_board_put_them`,
  `mode_4_shows_the_master_system_colours`) and its corpus digests unchanged (P4). Its two known mode 4 losses, the
  left column's garbage under a fine scroll and the H40 mode [R `Nephrite_Native.md` §15.2], stay losses unless the
  crate's tests on the board settle them, which would be a Nephrite dispute entry. *Moved 2026-10-19 at stage 4, its
  D-7 tests, its corpus reports and the open Sega CD BIOS unchanged (`Endou_Native.md` §11.6, `Nephrite_Native.md`
  §67); the variant also keeps Nephrite's right-column lock by the pixel, a question for the board.*

### 3.4 What stays in Endou

The bus and its decoding, the mappers and their identification, I/O and the peripherals, the scheduler, the YM2413,
the open boot program, the state, settings and the v1 glue. **The YM2413 stays a module** [C]: only the Master System
carries one among the Sega cores [A], so by the rule of 2026-10-05 it is not a Beryl crate; it keeps the chip
contract all the same, so that a second user (the NES's VRC7 is an OPLL derivative [D], which MoonRT does not emulate
[M: no `vrc7` in `EmuSen/Cores`]) would move it into a crate rather than copy it.

### 3.5 From the framework

`emusen-native` (the ABI's types and `core_exports!`, `StateWriter`/`StateReader`, the band-limited step synthesiser
Nephrite mixes through, the `Observer` and `Hooks`) [R `Beryl-HW/README.md`, `Beryl_SN76489.md` §2]; `emusen-systems`
for the systems' data and the pack's code (§4); `emusen-core-conform` for the kit; `emusen-core-host`, which discovers
and checks the library [R `EmuSen_Backend.md` §27.2]; WiseMan's generic test-ROM runner [R `Nephrite_Plan.md` §3.3].

### 3.6 What the framework should gain, found by planning this core

Each is an extension of an existing surface, proposed to the tester with the stage that needs it and not built ahead
of it [C]:

- **A pointer, for the Light Phaser** (stage 11): an optional `set_pointer` behind a capability bit and a controller
  `kind` in the descriptor, the shape `EmuSen_CoreAPI_Extensions.md` §10.1 recorded; a minor of the core ABI with its
  kit case, the hosts' input latch, and a frontend's mapping of a mouse or a touch onto it. Endou would be its first
  user; the NES's Zapper and the SNES's Super Scope would be later ones [A].
- **A battery that appears after create** (stage 2), only if the host does not already save a battery file that machine
  info lists after a `MACHINE_INFO` event; if it does not, §4.4's table form is used and nothing is proposed. *Settled
  at stage 2:* the host takes its battery files at create only, so the table form is used and nothing is proposed
  (`Endou_Native.md` §9.2).
- **The probe's options for the Master System** (stage 1): each reference's model, region and FM options pinned through
  `GET_VARIABLE`, as Nephrite's §1.6 did for the Genesis. *Built 2026-10-19:* nothing was needed beyond `--option`; the
  keys are listed in `Endou_Native.md` §5.2, and `.sg` names its system in the probe's log.
- **The routed system beside an image** (found at stage 1, `Endou_Native.md` §2.3): a host sets `system` in the create
  parameters only with an image of no bytes, so a core of two systems that share no telling header must guess. An
  extension of the existing field, a minor letting a host set it beside an image too. *Decided 2026-10-19:* accepted,
  designed in §3.7 and built as its own piece of work before stage 4. *Built 2026-10-19 as core ABI 1.15*
  (`EmuSen_CoreAPI.md` §6.40).
- **An `sms` module of `emusen-systems`** (stages 7 and 8) for the codecs and the views: a module beside the others,
  which the crate's design provides for [R `EmuSen_Backend.md` §5.2], so no new surface.

### 3.7 The routed system beside an image: the core ABI minor's design

*Accepted by the tester 2026-10-19; built before stage 4 as its own piece of work, across `emusen-native`'s header and
Rust side, `emusen-core-host`, both Crystal implementations and the kit. Not built at stage 2.* It is an extension of
an existing surface (`EmuSen_CoreAPI.md` §4.4): no export, struct, status or capability bit is added.

- **The field.** `emusen_create_params.system`, the `const char *` at offset 104 that 1.11 added (§6.31), unchanged in
  type and place. Its meaning widens: beside an image it names the system the host routed the file to, a system id as
  core info lists it.
- **The version.** The next minor of the core ABI, 1.15 at the time of writing (1.14 being the latest, §6.39). The
  header's comment on the field gains "@since 1.15: beside an image too, the system the host routed the image to", and
  `EmuSen_CoreAPI.md` gains a section that is the normative text.
- **How a host fills it.** A host of 1.15 or later sets `system` with every image it routes to a system, from the
  routing it already does (`route` by extension, then a recogniser where an extension is shared): the id of the system
  chosen, the same string machine info will report. A host that launches a file without routing it (a direct launch
  naming a core) sets the system it chose, or null if it chose none.
- **What a core does with it.** Non-null beside an image, the core makes a machine of that system and does not guess
  between its systems; a system the core's info does not list is refused with `EMUSEN_BAD_IMAGE` and words, as an
  unknown system with an empty image is (§6.31). Null, the core chooses as before. A core of one system may ignore the
  field. `emusen-native` reads it as `Create::system: Option<&str>`, `None` when the struct is shorter than 112 bytes
  or the pointer is null.
- **An older host's absence.** A host of 1.11 to 1.14 sets `system` only with an image of no bytes, and a host older
  than 1.11 passes a struct too short to hold it; beside an image either reads as `None`, and the core keeps the rule
  it has today (`Endou_Native.md` §2.3), so nothing an older host could observe changes. The minor gates nothing in a
  host or a core (§4.1): the field's presence is the whole signal, so no version check is made.
- **An older core.** A core built before 1.15 ignores the field beside an image, as §6.31 told it to, so a newer
  host's value changes nothing for it.
- **The kit.** A case for a library of 1.15 or later claiming two or more systems: for each system it lists, an image
  created with `system` naming it reports that system in machine info; one created with a system it does not list is
  refused. A library of one system passes the case unrun.
- **Endou's use.** At stage 4, when the SG-1000 becomes a modelled console, Endou takes the routed system and keeps
  the stage-1 rule only for a host that passes none; the three images that rule gets wrong in the tester's library
  (§10 Q10, `Endou_Native.md` §2.3) then run on the system their extension names.

*Built 2026-10-19 as core ABI 1.15* (`EmuSen_CoreAPI.md` §6.40, `EmuSen_Backend.md` §31), with two departures from
this design, weighed in `EmuSen_CoreAPI_Extensions.md` §17.2: `emusen-native` gives the system to a core as
`Core::create_as(system, request)`, defaulted to `create`, rather than as a field of `Create`; and a core whose own
rule tells its systems apart without a guess may keep it (MercuryRT's Model setting, Nephrite's contents), the kit
holding such a machine to the unnamed one. Endou takes the routed system from 1.15 on, ahead of stage 4.

---

## 4. What a frontend gets

Nothing here needs per-core code in a frontend (`EmuSen_CoreAPI.md` §2.4).

### 4.1 Systems, as data files

Three files in `Platform/systems/data/`, the fields being the table's seventeen [R `lib.rs`'s `KEYS`]:

| Field | `sms` | `sg1000` | `gg` (J3) |
|---|---|---|---|
| `name` | Sega Master System / Mark III | Sega SG-1000 | Sega Game Gear |
| `release_year` | 1985, the Mark III's [R Art Book Next's `mark3.xml`] [C] | 1983 [R `sg-1000.xml`] | 1990 [R `gamegear.xml`] |
| `extensions` | `.sms` | `.sg` | `.gg` |
| `cheat_systems` | `Sega - Master System - Mark III` [M: the cheat database's folder] | `Sega - SG-1000` [M] | `Sega - Game Gear` [M] |
| `openvgdb_systems` | `SMS` [M: OpenVGDB's `SYSTEMS`, 513 ROMs] | `SG1000` [M, 187] | `GG` [M, 490] |
| `esde_system` | `mastersystem` [R the theme's file names] | `sg-1000` [R] | `gamegear` [R] |
| `screenscraper_system` | 2 ~~[A, to be confirmed against ScreenScraper's list at stage 1]~~ [R ScreenScraper's system page, 2026-10-19] | 109 [R] | 21 [R] |
| `self_delimiting_images` | false | false | false |
| `byte_order` | little | little | little |
| `media` | none | none | none |

`esde_full_name`, `cover_aspect` and `cheat_space` are set at stage 7 from ES-DE's `es_systems.xml` and the boxes, as
Nephrite's were [R `Nephrite_Plan.md` §1.7]. ES-DE also lists a Mark III system: Art Book Next names its file `mark3`
[R]; its id in ES-DE is read then. *Read at stage 1 (2026-10-19):* ES-DE's `es_systems.xml` names it `mark3`,
"Sega Mark III", with the platform `mastersystem` [R `Endou_Native.md` §3]. No system of its own is recommended for it [C], since `.sms` is the Master System's
alone and routing by extension never consults the folder (below).

**Routing.** `.sms`, `.sg` and `.gg` are each one system's, so `route` chooses without a recogniser [R `probe.rs`'s
`route`: one candidate is chosen at once]. `.bin`, which the Genesis claims [R `md.json`], is **not** claimed at first
[C]; if the tester's library holds Master System images named `.bin`, `genesis::recognise`'s neighbour `sms::recognise`
(the `TMR SEGA` header at `$7FF0`, `$3FF0` or `$1FF0` [D]) and the folder's name decide, at stage 7.

### 4.2 Controllers

`sms.pad`: Up, Down, Left, Right, 1, 2, and the console's Pause and Reset as bits of port 0 [C]. Canonical controls
[C]: 1→B and 2→A (the bottom row's two, as Nephrite put the Genesis's B and C [R `Nephrite_Plan.md` §4.2]), Pause→Start,
Reset→Select. The Paddle, the Sports Pad and the Light Phaser are controller choices per port by setting (stages 10–11),
raising `MACHINE_INFO` when a port changes, as the Genesis's multitaps do [R `EmuSen_CoreAPI_Extensions.md` §10.1].

### 4.3 Spaces

`RAM` (8 KiB), `VRAM` (16 KiB), `CRAM` (32 bytes; 64 on the Game Gear), `CARTRAM` (the cartridge's RAM when it has
one), `ROM` (read-only), `BIOS` (read-only, the image or the open program); then `Z80BUS`, numbered 0, the processor
named `Z80` so the host's `<name>BUS` rule finds its code [R `Nephrite_Plan.md` §4.3]. The cheat space is decided with
the codecs (§4.5).

### 4.4 Battery files

File 0, the cartridge's RAM or the Game Gear board's EEPROM, as `.srm`, raw bytes in the bus's order. **Which games
have a battery** is open: the header cannot say, and Sega's mapper enables RAM by a write [D]. Two forms [C, settled at
stage 2]: a machine info that gains its battery entry with `MACHINE_INFO` the first time a game enables cartridge RAM,
if the host saves a battery that appears after create; or a table of battery boards keyed by the image's checksum,
from a public per-game listing, as Nephrite identifies its EEPROM boards by serial [R `Nephrite_Plan.md` §1.2]. The
first is preferred if the host already honours it, since it needs no list. *Settled at stage 2 (`Endou_Native.md`
§9.2):* the host does not, so the table, keyed by the image's CRC-32, names the battery boards; a battery game it does
not name keeps its RAM only while the game runs.

### 4.5 Cheats

The library's Master System, Game Gear and SG-1000 codes come in two shapes [M: the local cheat database,
2026-10-19]: `XXXX-XXXX` (3,135 lines for the Master System, 2,945 for the Game Gear, 40 for the SG-1000) and
`XXX-XXX-XXX` (1,037, 3,582 and 28), with combinations joined by `+`. The first is the Pro Action Replay's RAM write, the
second the Game Genie's ROM patch [D]. Two codecs in `emusen-systems`' `sms` module, shared by the three systems,
written from public descriptions of the formats at stage 7 [C]; the cheat space follows from where the Action Replay's
addresses point.

### 4.6 States, settings, the debugger

The state is Endou's own (§5.7). Settings arrive with the stages that make them meaningful (§5.8), and Mistress shows
them from core info's schema, the console's Graphics Settings tab appearing when a discovered engine runs it
[R `EmuSen_CoreAPI.md` §27.4]. The debugger is the generic one over `DEBUG`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`
with `beryl-z80`'s observer and disassembler; the VDP's, the PSG's and the OPLL's registers as units (core ABI 1.13)
[R `EmuSen_CoreAPI.md` §6.33]; and the console's views as an `sms` module of `emusen-systems` beside the NES's and
the Game Boy's [R `EmuSen_Backend.md` §13.5]: palettes at the console's levels, the 512 tiles in their planar format,
the name table, the 64 sprites, and the Z80's three vectors (`$0000`, `$0038`, `$0066`). *Built at stage 8
(`Endou_Native.md` §15) without the vectors, which the views take as pointers while the three are entry points (§15.5).*

---

## 5. Design decisions

Each states the trade-off, the recommendation and what it does not cover.

### 5.1 The master clock and the scheduler

- **One master clock**, 53.693175 MHz NTSC and 53.203424 MHz PAL [D], the Genesis's NTSC clock as an exact rational
  [R `Nephrite_Plan.md` §5.1], from which the Z80 and the PSG take ÷15 and the VDP's dot ÷10: a line is 342 dots and
  228 Z80 cycles, a frame 262 lines NTSC and 313 PAL [D]. The YM2413 runs at the Z80's clock and makes a sample every 72
  of its clocks [D].
- **Every bus access advances the clock**, as in every Beryl core: `beryl-z80` reports each machine cycle through the
  bus, which owns the time [R `Beryl-HW/README.md`]. The Z80 is the only processor, so there is no slice to bound.
- **Catch-up.** The VDP is brought to the present at each port access and at each line's end; the PSG and the OPLL at
  their writes and at the frame's end. The VDP's interrupt line is computed at its exact dot, so the Z80 sees INT at the
  instruction boundary at which the hardware would [A]. The Master System has no wait states for the Z80 [D].

### 5.2 The VDP's granularity

- **Recommended [C]: a dot-stamped span renderer**, Nephrite's slot-stamped design at the Master System's scale: every
  register, VRAM and CRAM write is stamped with the dot at which the VDP takes it, the line is drawn up to a stamped
  write before it is applied, and a line with no mid-line write costs one composition. The counters, the interrupts,
  the sprite pass and the access slots run on the dot schedule whether or not a picture is drawn, so skipping it leaves
  the state as a drawn frame would (`skip_rendering_state_neutral`, P9).
- **The access slots** are the thin part: MacDonald saw writes lost on a 315-5246 when written too fast [R `msvdp.txt`
  §15], and which dots of a line serve the CPU is not stated in the documents read for this page. A test program
  written for the purpose measures it against the references, and SMS_MiSTer referees it (P3).
- **Not covered at first:** the analogue output (no composite artefacts; the screen filters are the frontend's) and
  the border (§1.3).

### 5.3 The Z80, interrupts and the Pause button

The VDP's INT is level, held until the status port is read [R `msvdp.txt` §4]; the Pause button is an edge on NMI,
which `beryl-z80` takes at the boundary before INT [R `Beryl_Z80.md` §5]. A player's press reaches the latch at the
frame's start, as every input does; a press is therefore one edge a frame at most [A]. The Light Phaser's latch (stage
11) needs the beam's position at its dot: the core compares the pointer with the dot being drawn when TH is sampled
[A].

### 5.4 Sound

- **The PSG** through `beryl-sn76489`, written at `$7F` (and its mirrors) at the Z80's master clock [D].
- **The YM2413** (stage 9), a module of Endou keeping the chip contract (§3.4). Its operator, envelope and rhythm rules
  come from Yamaha's YM2413 application manual and the public research; its log-sine and exponent ROMs are D-2's
  question; its fifteen fixed instruments are a table whose provenance is itself a thin area, since the manual names
  the instruments without their parameters [D]: the table is taken only from a public document that states it, and
  each value is pinned by the referee (§6.1).
- **The mix and the output.** Both chips' outputs are levels, mixed and resampled by `emusen-native`'s step synthesiser
  to the host's rate [R `Beryl_SN76489.md` §2]. The Japanese console's audio control at `$F2` selects what reaches the
  output [D]; the analogue stage (the consoles' filters and levels) is measured from recordings where they exist, as
  Nephrite's was from MDFourier's [R `Nephrite_Plan.md` §6, stage 5], and is otherwise a named loss.

### 5.5 Firmware: the open boot program

`EmuSen_Firmware.md` §0 (decided 2026-10-03): every game runs with no firmware folder; the player's own image is the
exact path when present, never prompted for, shipped or fetched; each replacement states its cost [R]. Endou declares
every image `required: false` with its `replacement` words [R `EmuSen_CoreAPI.md` §6.2], and may list the images'
MD5s from a public description of them [R §6.32; `EmuSen_Firmware.md` §0 item 4].

**Which consoles have firmware** [D]: the Mark III and the SG-1000 none; the Japanese Master System, the Master System
and the Master System II a BIOS each, in several versions and sizes, some with a game built in; some Game Gears a boot
ROM. On a console without firmware Endou runs the cartridge from reset, which is exact.

**What the BIOS does that a game can see** [D, cited at stage 6 in `Endou_OpenBoot.md` §1]: it initialises the VDP and RAM and shows Sega's
screen; it chooses the medium (the card, the cartridge, the expansion slot) by enabling each through `$3E` and looking
for a header; an export BIOS checks the `TMR SEGA` header's checksum and region and refuses a cartridge that fails;
it hands over by writing `$3E` to map the chosen medium and jumping to `$0000`, leaving the value it wrote in RAM at
`$C000`, which a few games read.

**Recommended [C, §10 Q5]: an open Z80 program**, written in assembly with `beryl-z80`'s assembler, its source
committed and its binary built at compile time, occupying the BIOS's slot and run by the emulated CPU, as the open
Sega CD BIOS is [R `Nephrite_Plan.md` §5.6, Q4]. It initialises the VDP and RAM to the documented state; shows a short
screen of its own, with no maker's logo; chooses the medium as the BIOS does; on an export model performs the
documented header check, refusing with a screen in words a cartridge that fails it; and hands over through `$3E`
with the value at `$C000`. With no cartridge (`starts_empty`) it shows its screen and waits. Running real code keeps
the timing, the interrupts and the RAM a game finds; answering the hand-over in Rust would be simpler and wrong for
any game that reads what the BIOS left [A].

**Written first as a page of its own.** Nephrite's open Sega CD BIOS began with a page, `Nephrite_OpenBios.md`, whose
first finding stopped its stage at the plan: the manuals named every call, but no document the writer could read gave
the numbers the code needed, and the assembler the plan named did not yet exist [R its opening]. Endou takes the
lesson: the assembler comes at stage 2, and stage 6 opens with `Endou_OpenBoot.md`, written by a replacement writer
before any of its code, giving what each document says the BIOS leaves behind, cited by section, what none says, the
steps with their oracles, the cost's words and the questions only the tester can answer.

**Its accuracy cost**, as the firmware page shows it [R `EmuSen_CoreAPI.md` §6.2, amended 2026-10-05]: `accuracy`,
"Games start without Sega's logo screen. The open boot program sets up the console as the documents say the BIOS does
and hands the game over the same way; a game that reads the BIOS's code or relies on what the documents leave unsaid
may behave differently, the start-up takes a different time, and the BIOS's built-in games are absent." *Risk:* low;
the contract is narrow and P6 predicts no bench game differs. **G9** holds it (§8), and `firmwarecheck.py` measures it
against the player's images if the tester supplies them to the probe cache [R `EmuSen_Firmware.md` §0]. The Game Gear's
boot ROM, a licence screen and a header check [D], gets the same treatment at J2.

### 5.6 Cartridge boards, and how a board is chosen

The header names no mapper [D]. The rule [C]: a table of boards keyed by the image's checksum, from public per-game
listings, for every board but the common ones; otherwise Codemasters' when its own header checksum at `$7FE0` agrees
[D], the SG-1000's plain board for `.sg`, and Sega's mapper for anything larger than 48 KiB. A board the table cannot
place runs on Sega's mapper and is named in the log. This is the rule Nephrite follows for its EEPROM boards [R
`Nephrite_Plan.md` §1.2], and it keeps the choice out of guessing by write patterns.

### 5.7 The state format

Endou's own, `ENDO`, written with `StateWriter`: the magic, a version, the system and model, the frame count, then each
block in a fixed order (the Z80's 35 bytes [R `Beryl_Z80.md` §6.3], RAM, the VDP with VRAM and CRAM, the PSG, the
OPLL, the mapper, the cartridge's RAM, the I/O and the peripherals), its layout pinned by a test. About 26 KiB, 58 KiB
with 32 KiB of cartridge RAM [A]; rewind's snapshots are that size. Loading older versions follows Nephrite's rule from
the first version a player has had [R `Nephrite_Plan.md` §5.7].

### 5.8 Settings

All `effect: none` unless said, each a real console, the defaults the accurate configuration [R `EmuSen_CoreAPI.md` §12.1
C3]:

- **Model** per system. `sms`: Mark III, Mark III with the FM Sound Unit (stage 9), Japanese Master System (stage 9),
  Master System, Master System II. `sg1000`: SG-1000, Mark III. Default by the header's region code [D]: export → Master
  System II; Japanese → Mark III until stage 9, then the Japanese Master System (§10 Q9).
- **Region**: auto, Japan, the Americas, Europe, Brazil; NTSC or PAL follows the region, auto taking the Americas for an
  export cartridge (§10 Q9).
- **Controller per port** (stages 10–11).
- **3-D glasses' picture** (stage 10), an `enhancement` setting: its `hardware` value, the default, shows the picture
  as the console outputs it, both eyes alternating; its other value shows one eye, with its cost ("The 3-D picture is
  shown flat, as the left eye sees it").
- ~~**BIOS**: the player's image when present, else the open program; no setting, by the firmware policy.~~
  *Decided 2026-10-19:* **BIOS, per model with one**: the open boot program by default; the player's own image of that
  console's BIOS as an opt-in accuracy upgrade, used when chosen and present, else the open program.
- The output sample rate.

### 5.9 The speed budget

**What the work costs, argued.** The Z80 is 0.038 ms a frame on the desktop [R `Beryl_Z80.md` §6.4]; the VDP composes
at most 262 lines of 342 dots; the PSG is a few hundred changes a frame; the OPLL makes 830 samples a frame of nine
channels [A from §5.1's clocks]. **The target** is Nephrite's form [R `Nephrite_Plan.md` §5.5]: every bench game at full
speed at a 33% CPU quota with a fifth of the frame left for the frontend, a desktop mean of about 3.8 ms. Endou's
**budget** is far inside it: a desktop mean of at most 0.6 ms a frame without the OPLL and 0.9 ms with it (P1). No
speed lever is planned; if one is ever needed it is exact or an opt-in setting, never a default (§7, stage 13).

---

## 6. References, oracles and referees

The sources, the test programs and the references are recorded with their provenance and licence in
**`Endou_References.md`**, which says what is already in the probe's cache from Nephrite's fetch and what stage 1
fetched on the tester's go (§10 Q4, decided 2026-10-19). Nothing was fetched at stage 0.

### 6.1 Oracles by component

| Component | Primary oracle | Secondary | Thin (§6.2) |
|---|---|---|---|
| Z80 | already met in `beryl-z80`: SingleStepTests' Z80 suite and ZEXDOC/ZEXALL [R `Beryl_Z80.md` §4, §6.1] | an SMS-hosted ZEXALL on the whole machine | — |
| Bus, mappers, I/O | synthetic programs written for each rule, assembled with `beryl-z80`'s assembler | references' RAM at anchors; SMS Power's mapper pages | the decoding's mirrors by board |
| VDP, mode 4 | the homebrew VDP test programs (`Endou_References.md` §2), self-reporting where they report | references' pictures up to a colour map; the board for the 315-5313's rules; SMS_MiSTer in disputes | access slots; the line interrupt's dot; the 315-5124's zoom; colour levels (D-1) |
| VDP, TMS9918 modes | SG-1000 programs and cartridges against the references | Young's document; TI's data manual | the undocumented modes' details |
| PSG | already met in `beryl-sn76489` by SMS Power's document [R] | references' audio | the discrete chip's half-period 0 [R `Beryl_SN76489.md` §4] |
| YM2413 | a referee run sample for sample (Nuked-OPLL as a black box, allowed by §10 Q4) | recordings; references' audio | ROM tables (D-2), the instrument table, the rhythm channels |
| Open boot program | the bench games with no firmware folder, against the same games on the player's BIOS | references with the player's BIOS | what the documents leave unsaid of the hand-over |
| Peripherals | test programs and games at anchors against the references | — | the Paddle's Japanese and export protocols |
| Frontend | the kit's cases on synthetic images, then on games (corpus form) | the fit audit | — |

### 6.2 Where the documents are thin

Each is a prediction of where the disputes log will fill (P3): the CPU's access slots and when a write lands; the line
interrupt's and the frame interrupt's exact dots and the counters' values around them; the 315-5124's sprite zoom and
its other differences from the 315-5246; the Master System's colour levels (D-1); the decoding's mirrors on each
board; the YM2413's ROMs (D-2) and instrument table; the Japanese console's audio control and the FM unit's
detection; the Paddle's protocol by region; the region read-back of `$3F` by model.

---

## 7. Stages

Each stage is a run of supervised steps of about three hours with a check-in after each, as Nephrite's and Diamond's
ran. **The effort is predicted** (P2). A stage's gate must be met, or its misses recorded as disputes or named losses,
before the next begins. The core carries the `InDevelopment` mark from stage 1 until the tester offers it
[R `EmuSen_CoreAPI.md` §27.2].

| Stage | What it covers | Its gate: what passes, measured how | Steps |
|---|---|---|---|
| **0** | This plan, its references page and disputes log | The tester's go | 1 (2026-10-19) |
| **1** | *Built 2026-10-19, `Endou_Native.md` §1–§6.* The crate `endou` on the core ABI v1 as a stub: core info for `sms` and `sg1000`, the header read, a black picture; its row in `RustCores.props` with `<InDevelopment>true</InDevelopment>`, so the build writes its sidecar marked and lists it in `cores.manifest` [R `EmuSen_Backend.md` §27.11]; CI's matrix row; the data files `sms.json` and `sg1000.json`; WiseMan's `SyntheticSmsRom` (a header with its checksum and region, a boot stub, a pad reader, as `SyntheticMdRom` serves Nephrite [R `NephriteTests.cs`]); on the tester's go, the documents, test programs and references fetched and pinned (`Endou_References.md`), the three local references and any fetched ones run through the probe with their costs and spaces | The kit's C1–C23 on synthetic images; the systems crate's tests; a published tree's `sha256sum -c cores.manifest`; the references' costs recorded | 2 |
| **2** | *Built 2026-10-19, `Endou_Native.md` §9.* The machine without its picture: the bus, RAM, `$3E`, `$3F`, the controller ports, the region read-back, the Pause NMI and the Reset bit; cartridges (plain, Sega's mapper with its RAM, Codemasters', the SG-1000's RAM boards) and the board table's form; the battery form (§4.4); the scheduler on the master clock; `beryl-z80`'s assembler | Synthetic programs for each port, mapper and interrupt rule; an SMS-hosted ZEXALL on the whole machine; RAM against the references at anchors on test programs | 3 |
| **3** | *Built 2026-10-19, `Endou_Native.md` §10.* `beryl-sms-vdp`, mode 4: ports, registers, VRAM and CRAM, the counters and the TH latch, the interrupts, the status flags, the composition with sprites (limits, zoom, tall, collision, overflow), the scroll locks, the mask, the 315-5246's heights, the access slots, the dot-stamped renderer, PAL | The VDP test programs (self-reporting ones by their verdicts); pictures equal to the references up to a colour map, or a dispute; the skip-versus-draw state test | 5 |
| **4** | *Built 2026-10-19, `Endou_Native.md` §11.* The VDP's TMS9918 modes and the TMS9918A variant; the SG-1000 as a system; **Nephrite moved onto the crate's composition** | SG-1000 programs and cartridges' pictures against the references; **Nephrite's D-7 tests and its corpus digests unchanged** (P4) | 3 |
| **5** | *Built 2026-10-19, `Endou_Native.md` §12.* Sound: the PSG's wiring in both variants, the mix and the resampler, the analogue stage as far as recordings allow (none: a named loss), `$F2` with no FM unit | Synthetic voices against the references' audio; recordings if any | 2 |
| **6** | *Built 2026-10-19, `Endou_OpenBoot.md` and `Endou_Native.md` §13.* Firmware: `Endou_OpenBoot.md` written first (§5.5); the player's BIOS by region and size, with `md5` from a public description; the open boot program; `starts_empty` for `sms` | **G9**: every bench game with no firmware folder, machine info naming the replacement; the same games on the player's BIOS when given; `firmwarecheck.py` if the tester's images are in the probe cache | 3 |
| **7** | *Built 2026-10-19, `Endou_Native.md` §14; offered to players by the tester's go.* The Master System to players: the data files completed (`esde_full_name`, `cover_aspect`, `cheat_space`), routing (and `.bin` if the library needs it), the shelf from the pack, scraper ids checked, the codecs (`sms.par`, `sms.gg`), the settings, battery files, states with rewind, the kit in corpus form (`kit/sms-sample.txt`, as Nephrite's `kit/genesis-sample.txt` [R]), the core host's listing, Mistress's settings and its Graphics tab; the SG-1000 on a shelf of its own, its console `"SG-1000"` apart from `"Master System"` (decided 2026-10-19) | **G4**: the kit's cases on the tester's games; a test that `sms` and `sg1000` land on two shelves; `.srm` round trips; the fit audit at both sizes; the cheats decode; the sidecar and manifest verified over a publish before any install. The mark is removed only by the tester's decision | 3 |
| **8** | *Built 2026-10-19, `Endou_Native.md` §15.* The debugger: `Z80` through the observer, registers, disassembly, breakpoints, stepping, watches, coverage, the call stack; the VDP's and PSG's units; the `sms` views module in `emusen-systems` | **G7**: each claim a test; every table armed gives the plain run's digests (C15) | 2 |
| **9** | The YM2413 and its models (the Japanese Master System, the Mark III with the FM Sound Unit), `$F0`–`$F2`, detection; D-2; the operator ROMs' move if D-2 allows | The referee sample for sample on synthetic voices; FM games against the references' audio; Nephrite's FM tests unchanged if the ROMs move | 4 |
| **10** | The Paddle, the Sports Pad (axes behind port settings) and the 3-D glasses with their display setting | Test programs and the named games at anchors against the references | 2 |
| **11** | The Light Phaser, with the core ABI's pointer minor proposed and, on the tester's approval, built across `emusen-native`, the header, the hosts and the kit | A kit case for the minor; the phaser's latch on a test program; the named games' hits against the references | 3 |
| **12** | The Korean and multicart boards, each from a document; the table extended | Each board's synthetic program; each named game to its title against a reference | 2 |
| **13** | Speed, only if §5.9's budget is missed: exact levers, measured on the bench, under the quota and on the handheld | **G6** | 0–2 |
| **14** | The gates of §8, the goldens, the clone check as the tester decides it | §8 | 1 |
| **J1** | *Built 2026-10-20, `Jadeite_Native.md`.* The Game Gear's VDP: GG mode's CRAM and window, SMS mode's scaling, TMS9918 colours | GG test programs and pictures against the references | 2 |
| **J2** | The Game Gear's I/O: Start, the region, stereo, the EEPROM boards; its open boot replacement | Synthetic programs; G9 for the Game Gear | 2 |
| **JL** | *Added 2026-10-20 (§2.5).* The Game Gear's own LCD model: `Jadeite_Lcd.md`'s research and design first, then the model as a profile of the screen filters, the panel's response in time, its grid at the display's size and its colours, from documents and published measurements | Each property traced to a source or named as argued; the model against published photographs and captures by a measure written beforehand; a `ConsoleScreens` row with no gap; the fit audit; the handheld's cost; seeded faults | 3 |
| **J3** | The Game Gear to players: `gg.json`, shelf, scraper, codecs, settings, kit | G4 for the Game Gear, and `ConsoleScreenRuleTests` with `gg`'s row from JL | 1 |

About 37 steps for the Master System and the SG-1000 and 5 for the Game Gear, or 120–150 hours (P2); JL, added
2026-10-20, about 3 more. The order is the
order of dependence; stage 7 offers the Master System before the FM unit and the peripherals, stages 9 to 12 are
independent of each other, and J1–J3 need only stage 7.

---

## 8. The gates

Endou replaces nothing, so its gate is the one to **offer a system to players**, per system, by the tester's decision,
as the Genesis, the Sega CD and the PlayStation were offered [R `Nephrite_Plan.md` §7; `Diamond_Plan.md` §8].

- **G1, the processor.** Met in `beryl-z80` [R]; on the machine, an SMS-hosted ZEXALL passes.
- **G2, the self-reporting programs.** The VDP and machine test programs that report a verdict pass, each failure a
  disputes entry.
- **G3, the pictures.** On test programs whose pictures stand still, the picture equals the references' up to a colour
  map, or the difference is a dispute or a named loss of §5.2.
- **G4, the frontend.** The kit's cases on each system's games; battery files round-trip; the fit audit passes; the
  codecs decode; the published tree's manifest verifies.
- **G5, the goldens.** Commercial games per system (§10 Q10), each at three anchors, equal to the references in picture
  and RAM, or dispute-logged. Candidates, by what each exercises [A]: Alex Kidd in Miracle World and Sonic the Hedgehog
  (Sega's mapper); Phantasy Star and Golden Axe Warrior (cartridge RAM with a battery); Wonder Boy III: The Dragon's Trap
  and Ys (FM on the Japanese console); Space Harrier, Out Run and Fantasy Zone (line interrupts and raster effects);
  Micro Machines and Cosmic Spacehead (Codemasters' mapper and the 224-line mode, which MacDonald names [R `msvdp.txt`
  §13]); The Simpsons: Bart vs. the Space Mutants and the Japanese Ys (the quirks §13 names [R]); Safari Hunt and
  Shooting Gallery (the Light Phaser); Missile Defense 3-D and Space Harrier 3-D (the glasses); Alex Kidd BMX Trial
  (the Paddle); Great Ice Hockey (the Sports Pad); Flicky and Girl's Garden (the SG-1000); for the Game Gear, Sonic
  Chaos and Shinobi.
- **G6, speed.** §5.9's budget at 33% quota on the bench set, and full speed on the Legion Go S on battery.
- **G7, the debugger.** Breakpoints, stepping, watches, coverage and the call stack, each a test.
- **G8, the clean room.** `clonecheck.py` over Endou's and the new Beryl crates' sources against the references whose
  sources exist finds nothing above its threshold, the reference side never printed; the disputes log read end to end
  by someone who did not write it. For the Genesis this review was skipped by decision [R `Nephrite_Plan.md` §7]; for
  Endou it is the tester's again.
- **G9, firmware.** Every game the open boot program is claimed to run runs with no firmware folder, its machine info
  naming the replacement; with the player's image, the same game runs on it.

---

## 9. Sizes

### 9.1 Per stage

Lines of Rust (non-test code) and of tests, each a range [A]. Calibrated against what exists [M: `wc -l`, 2026-10-19]:
MercuryRT, the Game Boy with its CPU, 6,041 lines in all; MoonRT, the NES with its CPU and mappers, 7,398;
`beryl-z80` 2,267; `beryl-sn76489` 343; Nephrite's mode 4 about 100 of `render.rs`'s 754. Endou carries no CPU or
PSG of its own.

| Stage | Rust | Tests | Notes |
|---|---:|---:|---|
| 1 | 600–1,000 | 200–400 | the crate's v1 glue, info, header; two data files; `SyntheticSmsRom` (100–200 lines of C#) |
| 2 | 1,800–3,000 | 600–1,100 | the bus, I/O, mappers, scheduler 1,200–2,000; `beryl-z80`'s assembler 600–1,000 |
| 3 | 1,800–3,000 | 500–1,000 | `beryl-sms-vdp`: ports and timing 900–1,500, composition and renderer 900–1,500 |
| 4 | 700–1,200 | 300–600 | the TMS9918 modes and the TMS9918A; Nephrite about 100 lines lighter |
| 5 | 300–600 | 150–300 | wiring, mix, the analogue stage |
| 6 | 400–800 | 200–400 | 300–700 lines of Z80 assembly beside the Rust |
| 7 | 600–1,100 | 300–600 | the codecs and pack code 200–400 of it |
| 8 | 500–900 | 200–400 | the views module 300–500 of it |
| 9 | 1,200–2,200 | 400–900 | the OPLL 900–1,600; models and ports; the referee's harness |
| 10 | 300–600 | 150–300 | |
| 11 | 600–1,200 | 300–600 | the ABI minor across the stack 400–800 of it |
| 12 | 400–900 | 200–400 | |
| 13 | 0–600 | 0–200 | only if needed |
| 14 | 100–300 | — | tools |
| J1–J3 | 900–1,700 | 400–800 | |

### 9.2 In total

**The Master System and the SG-1000: 9,300–17,400 lines of Rust and 3,500–7,200 of tests; with the Game Gear,
10,200–19,100 and 3,900–8,000** [A]. The wide range is the VDP's access timing and the YM2413, the two places the
documents are thinnest; P2 records the prediction.

---

## 10. Questions, decided 2026-10-19

Each question as it was put, and its decision. The tester answered all of them as recommended, changing one: Q11, the
console for disputes, is decided in the tester's own terms. "Please continue" was not taken as an answer to any of them.

1. **Q1, the Game Gear: together or after, and in what form?** *Recommended:* planned now as §2 has it, built after the
   Master System's gate (J1–J3), as a system of Endou's library like `gbc` in MercuryRT's; the shared crates carry its
   variants from the start. The name Jadeite stays on the Game Gear's module and pages; the alternative is to retire it
   to the pool. *Decided as recommended.*
2. **Q2, the YM2413 in v1?** *Recommended:* not before the gate, but the stage directly after it (stage 9). Until then a
   Japanese cartridge runs on a Mark III without the FM unit, a real console, so the default stays accurate; the stage
   adds the Japanese Master System as the Japanese default. *Decided as recommended.*
3. **Q3, which peripherals?** *Recommended:* the pad, Pause and Reset at the gate; the Paddle, the Sports Pad and the
   3-D glasses at stage 10; the Light Phaser at stage 11 with the core ABI's pointer minor (`set_pointer` behind a bit,
   a controller kind), proposed then for approval; the Handle Controller, the Graphic Board and the keyboard out.
   *Decided as recommended.*
4. **Q4, may the named sources be fetched?** *Recommended:* yes, at stage 1, into `~/.cache/emusen/probe/endou/`, never
   committed, each file with its source, date, hash and licence in `PROVENANCE.txt` and `DOCS.txt`, the search for
   test programs recorded with its negative results as Nephrite's was; the libretro cores Gearsystem and SMS Plus GX
   pinned beside the three already in the cache; MAME and Mesen 2 built as black boxes; Nuked-OPLL built from its
   public release and driven through its header alone, as a black box for stage 9. *Decided as recommended*, for
   stage 1's fetch; as with every reference, no emulator's source is read, Nuked-OPLL's beyond its public header
   included, and prebuilt binaries are preferred to builds from source.
5. **Q5, the BIOS replacement's scope?** *Recommended:* an open Z80 program run by the emulated CPU (§5.5): the
   documented set-up, the medium's choice, the export models' header check with a refusal in words, the hand-over
   through `$3E` with the value at `$C000`, and a screen of its own with no logo; no built-in game. The Game Gear's
   boot ROM in the same form at J2. *Decided as recommended.*
6. **Q6, may SMS_MiSTer's RTL be read in a dispute step?** Nephrite's rule for the Sega cores allows it after the
   documents and a test program, each file logged and nothing transcribed; Diamond's decision of 2026-10-11 allows
   only the README, licence, file names, top-level ports and simulation scripts "of each FPGA core". *Recommended:*
   Nephrite's rule, the Sega precedent, the core run as a black box first. *Decided as recommended:* black box
   first; its RTL read only in a logged dispute step, each file listed.
7. **Q7, the shared VDP crate.** *Recommended:* `beryl-sms-vdp`, the 315-5124/5246/5378 family with the TMS9918A as a
   variant, its line composition shared with Nephrite, which moves onto it at stage 4 under its own digests. *Decided
   as recommended.*
8. **Q8, the SG-1000.** *Recommended:* a system of its own, `sg1000`, on a modelled SG-1000 by default (the TMS9918A,
   the discrete PSG), the Mark III as a model setting; the SC-3000 and the Othello Multivision out. *Decided as
   recommended.*
9. **Q9, the default models and region.** *Recommended:* an export cartridge on a Master System II, NTSC, the Americas;
   Europe and PAL by the Region setting; a Japanese cartridge on a Mark III until stage 9, then on the Japanese Master
   System. A table of PAL-only games is added only when one is shown to need it. *Decided as recommended.*
10. **Q10, the tester's library.** The desktop library has no Master System folder [M: `~/Documents/Roms`'s listing];
    the handheld's card holds `ROMs/mastersystem` (371 files) and `ROMs/gamegear` (1). *Recommended:* copied into
    `AppSettings.RomDirectory` when stage 1 needs a corpus, read only and copied to scratch for every use, as the
    Genesis library was [R `Nephrite_Plan.md` §9 Q13]. *Decided as recommended:* the library was copied on
    2026-10-19 into `AppSettings.RomDirectory`'s `mastersystem/`, 370 files (316 `.sms` and 54 `.sg`), each md5 checked
    against the handheld's; ~~there are no Game Gear games.~~ *The same day the tester supplied six Game Gear games
    in `gamegear/` (zips holding one `.gg` each: Ecco the Dolphin, Chuck Rock, Chuck Rock II, Mega Man, Marble Madness
    and Sonic The Hedgehog 2), candidates for the Game Gear's gates at J1–J3.* It is read in place and never written,
    and nothing from it is committed; tests build their own images with `SyntheticSmsRom`.
11. **Q11, a console for disputes?** If the tester owns a Master System, a Game Gear or a flash cartridge for either,
    a test program run on it settles D-1 and the access slots outright. *Recommended:* not required; references and
    referees as for Diamond, a console's run welcome where one exists. *Decided otherwise:* **no real hardware is
    used.** The tester's consoles are out of use for their age and fragility, so no program is run on a console, a Game
    Gear or a flash cartridge, and no capture of a console's output is made for the project. D-1, the access slots and
    every other dispute are settled from documents, test programs and black-box referees, a published measurement made
    by others counting as a document. A rule that only a console could settle is implemented as the evidence argues and
    marked argued, not measured.

---

## 11. Negative results

- No Master System or Game Gear test program, SMS-specific document beyond `msvdp.txt` and SMS Power's "SN76489", or
  YM2413 document outside an unextracted archive is in the probe's cache [M: `ls`, `DOCS.txt`, 2026-10-19].
- No measurement of a Master System's colour output exists in the corpus; Nephrite's mode 4 levels are the board's DAC
  model's [R `Nephrite_Disputes.md` D-7].
- No SMS_MiSTer clone is on the desktop [M: `~/Projects`]. *Since stage 1 a shallow clone is in the probe's cache,
  read only as far as its README and directory names (`Endou_References.md` §4).*
- The desktop library had no Master System or Game Gear folder when this page was written [M]. ~~Measured
  2026-10-19, at stage 0.~~ *Since 2026-10-19 it has `mastersystem/` and `gamegear/` (§10 Q10).*
- No transcription of the 315-5124 from die images is known to this page; the network was not searched for one, by
  this stage's rule, and stage 1's fetch searches. *Stage 1's fetch found none (`Endou_References.md` §8).*

## 12. Predictions to be retired

- **P1.** At the gate the desktop mean is at most 0.6 ms a frame on the bench games, and at most 0.9 ms with the OPLL
  after stage 9. *At stage 3 (2026-10-19), with the VDP and without sound: 0.22-0.26 ms on five games, the desktop
  busy [M `Endou_Native.md` §10.7]. At stage 4: 0.19-0.22 ms on three Master System games, 0.18 on two SG-1000 games
  [M §11.9]. At stage 5, with the PSG mixed: 0.20-0.23 ms on three Master System games, 0.18 on an SG-1000 game, the
  sound 0.005-0.014 ms of it [M §12.8]. At stage 6, after the hand-over: 0.206-0.236 ms on three games, within 0.005 ms of stage 5's
  build measured alongside [M §13.7]. At stage 7, with the Game Genie's check on the cartridge's reads: 0.204-0.239 ms on
  three games, 0.005-0.006 ms of it the check [M §14.11]. At stage 8, with the debugger built and unarmed: 0.210-0.247 ms on three games, within 0.006 ms of stage 7's build measured alongside [M §15.10].*
- **P2.** The Master System and the SG-1000 take 32–42 steps and the Game Gear 4–6; the VDP's stage overruns first.
- **P3.** At least half the disputes entries fall in §6.2's thin areas, most of them the VDP's access slots and
  interrupt dots. *At stage 3: of D-0 to D-9, D-1, D-2, D-3, D-4, D-6, D-8 and D-9 are in them (the colour levels, the
  OPLL's ROMs, the region read-back, the decoding, the 315-5124's zoom, the access slots, the interrupt dots).* *At stage 5: D-13, the audio control port with no FM unit, is in them too.* *At stage 6: D-14 to D-16 are the
  firmware's, a thin area §6.2 did not name; D-16 was settled by measuring the games.*
- **P4.** Nephrite's move onto the shared composition leaves its D-7 tests and every corpus digest unchanged. *Retired
  at stage 4 (2026-10-19) as predicted: both D-7 tests pass, the kit's 64 image reports are identical, and 200,000
  random lines drew the same before the old renderer was deleted [M `Endou_Native.md` §11.6].*
- **P5.** The YM2413's log-sine and exponent ROMs are the OPL2's, entry for entry (D-2).
- **P6.** No bench game behaves differently on the open boot program than on the player's BIOS after the hand-over,
  start-up time apart. *At stage 6 (2026-10-19): not measurable by the project, which runs no real BIOS; against the
  start with no BIOS, 29 of 31 export games reach the same pictures, the other two by the hand-over's phase and the
  Z80's R register, which any BIOS also sets its own way (`Endou_Native.md` §13.8). The tester's runs on their own
  images would retire it.*
- **P7.** The Game Gear as a system of Endou costs at most 1,700 lines of Rust.
- **P8.** The Light Phaser's minor needs one export, one capability bit and a controller kind, nothing more.
- **P9.** The skip-versus-draw state test passes on every bench game from stage 3 on. *At stage 3: it passes on the
  eight games and sixteen programs the kit ran with rendering skipped on alternate frames (`Endou_Native.md` §10.7).*
- **P10.** No commercial game needs the 315-5124 rather than the 315-5246, so the Master System II is a safe export
  default; a game shown otherwise is added to the defaults' table.
