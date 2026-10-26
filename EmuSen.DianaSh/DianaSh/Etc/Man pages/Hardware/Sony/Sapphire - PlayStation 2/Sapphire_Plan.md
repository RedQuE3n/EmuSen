# Sapphire_Plan — a clean-room PlayStation 2 core in Rust

*Written 2026-10-23, at stage 0.* This page plans Sapphire, EmuSen's Sony PlayStation 2 core, written in Rust from
hardware documentation, graded by test programs whose expected results were recorded on consoles, refereed by
emulators run as black boxes, and loaded through the core ABI v1 (`EmuSen_CoreAPI.md`, `EmuSen_NativeCores.md`). It has
no predecessor to port. It follows `Diamond_Plan.md`'s shape, since Diamond began the same way twelve days earlier, and
it borrows Diamond's rules wherever the PlayStation 2 gives no reason to change them; where it changes one, it says so.

Every claim carries one of five marks:

- **[M]** *measured* on 2026-10-23, with the tool named beside it. Nothing was run to write this page beyond listings,
  the hosts' public APIs (GitHub's, the libretro buildbot's listing) and a query of the OpenVGDB file the project
  already holds; the measurements of the core begin at stage S1.
- **[R]** *read* from a source, named, on 2026-10-23 unless another date is given. Online sources are listed with
  their addresses in §14.
- **[A]** *argued*: reasoning from the documents or from other cores' measurements, with no measurement here.
- **[C]** *a choice*: either a rule set for this core, said so, or a recommendation awaiting the tester (§11).
- **P1–P16** *predicted*, numbered in §13 so that a later stage can retire each; P17–P19 were added on 2026-10-24
  with the speed decision below.

**The tester's decisions this page records:**

- *Decided 2026-10-19 by the tester:* the PlayStation 2 is the project's next large target, for its game library.
  Firmware: the player's own BIOS dump first, "to get it going"; the open replacement follows later, under the
  standing firmware policy (`EmuSen_Firmware.md` §0: every core ships an open replacement, and the player's dump is
  optional and never fetched, shipped or prompted for).
- *Decided 2026-10-23 by the tester:* Sapphire starts now, before the last step of the backend's port to Rust (B10,
  paused since 2026-10-20) closes.
- *Standing, restated for this core:* no real hardware, ever. The tester's consoles are too old and fragile to be
  measured, so there are no console measurements and no homebrew run on a console. Disputes are settled by reference
  emulators run as black boxes, and by the logs and pictures recorded from consoles that public test suites ship.
- *Decided 2026-10-23 (routine), as recommended:* Q2 (ps2tek pinned at `1c91660`), Q5 (Play!'s outputs for games
  only), Q9 (three market entries with No-Intro names), Q13 (the clean-room review at G8), Q15 (the recompiler on by
  default once its oracles pass), Q16 (the formats' stages) and Q17 (plain RAM writes as the only cheats at the player
  gate). §11 records each.
- *Decided 2026-10-24 by the tester, as recommended:* Q1 (Sony's Linux-kit manuals may be read, cited by document and
  section), Q3 (SDK and loader sources never read; ps2toolchain's compiler alone allowed), Q4 (PCSX2 standalone and
  Play! as the referees), Q6 (the open BIOS in form A, begun on the tester's go, its EE kernel not brought ahead of
  S10), Q7 (a research step for the card's authentication, then cards on the open BIOS path only if it finds
  nothing), Q8 (the tester's own dumps, under `AppSettings.RomDirectory`'s `ps2/`), Q10 (the shared crates), Q12 (the
  scope at the player gate) and Q14 (the EE's timing at instruction granularity). **The go for S1** came with them; it
  covers S1 alone.
- *Changed by the tester on 2026-10-24, Q11:* **the full-speed target is the handheld** (the Legion Go S), not the
  desktop alone. §5.12 carries the revised targets.
- *A standing decision for this core, 2026-10-24.* The tester: "I am ok with deviating from accuracy now that we are
  past the nintendo 64 era. I understand its going to be difficult and compromises will need to be made. We can focus
  on trying to be as accurate as possible while also balancing performance." For Sapphire, therefore, **accuracy is
  balanced against speed, and the default build aims for full speed on the handheld**. Every compromise is chosen on
  purpose, measured (the accuracy it loses against the speed it gains) and named in the plan's losses (§5.13); none is
  taken because it is convenient. The exact interpreter and the software GS stay, where they are built, as the
  references every compromise is graded against and every dispute is settled on. What this changes is revised in
  place with notes dated 2026-10-24: §1.1, §5.2, §5.3, §5.4, §5.12, §6.1, §8 and §13; §5.13 lists the candidate
  compromises, none decided.
- *Target games named by the tester on 2026-10-24:* **Jak and Daxter** (a test copy of the USA disc is kept read-only in
  the probe's cache, `~/.cache/emusen/probe/sapphire/discs/`, never committed or copied into the repository),
  **Kingdom Hearts** and **Dark Cloud 2** (not yet dumped). Q8's bench stays beside them, its three titles first.
- *Two player-facing gates added on 2026-10-24 by the tester's request.* The tester: "the goal with PS2 is similar to
  PSX with the ability to boot it without a disc and use the memory card functions." The plan built the parts (S24,
  the player's BIOS to OSDSYS; S25, the card and its crate; O12, the menu without a disc on the open BIOS) and promised
  neither to the player. **G10, start without a disc**, and **G11, the Memory Cards window for the PlayStation 2**,
  are therefore gates of §8, placed at S25 and offered with the system at S29 (§7); neither is built before then.
- *Decided 2026-10-24 by the tester, both as S3 recommended (`Sapphire_Referees.md` §6, §7):* **the probe gets no
  hardware render context now**; the referees' pictures come from Play!, taken as RAM, the project's GS programs
  reading the GS's memory back to the EE, and a headless EGL context stays in reserve for the first GS dispute that
  needs it (§1.6, §10). **DobieStation is not built now**; it is built user-locally only if a dispute needs a third
  lineage that ps2tek does not decide (§1.6).
- *Decided 2026-10-24 by the tester, as S4 recommended (`Sapphire_Native.md` §14.8 Q1):* **how Sapphire keeps what the
  IOP prints.** On the open BIOS, the TTY device writes its text to a port of the project's own, as Diamond's open
  kernel prints through the DUART. On the player's BIOS, the text is caught at the IOP's module interface: the output
  export is resolved at run time through the export tables ps2tek lists (cleared for every writer,
  `Sapphire_References.md` §3.4), never at an address inside the BIOS image, and the text is kept in a `tty` space.
  The player's BIOS capture is built at S10, whose gate needs it, and the open BIOS's port with its I/O at O8 (§7).
- *Decided 2026-10-25 by the tester: S11 is built before S10.* S10's gate runs on the player's BIOS by fast boot, which
  §7 places after S24 under Q6 as decided, so S10 is taken once S24's BIOS boots; S11 follows S9 directly. The EE's
  peripherals (S11 to S15) are graded by the project's own bare programs until then, as the EE's stages were, and the
  kernel tests among their gates run on the referees now and on Sapphire with S10 (§7).

---

## 1. Purpose, scope and the clean-room protocol

### 1.1 The machine, and how accurate each part must be

The whole console is in scope but for the parts §1.2 leaves out. "Exact" means bit-identical results and, where a
column says so, the same cycle at which a result is visible to software. The clocks are ps2tek's [R ps2tek "EE
Architecture", "EE Timers", "CDVD Reads and Seeks"]; that they hold on every model is the first dispute (§2.3, D-0).

*Amended 2026-10-24, by the speed decision above.* The table's targets are now the **reference's** targets: what the
exact interpreter and the software GS are built to, graded by the oracles of §3 and used to settle disputes. The
**default path**, what a player runs, is the reference with the compromises §5.13 has chosen, each measured and
named; where none is chosen for a part, the default is the reference. Before the decision this page held the default
to the table; that reading is retired, and the purpose of the core is restated as *as accurate as the handheld's full
speed allows, every departure deliberate and recorded*.

| Part | What is in scope | Target |
|---|---|---|
| **EE core** (R5900, 294.912 MHz) | The MIPS III subset the R5900 implements with its own additions: 128-bit general registers, LQ/SQ, the three-operand MULT, MADD, the second multiplier pipeline (MULT1, DIV1, HI1/LO1), the branch-likely forms, SYNC.L/SYNC.P; the **MMI** instructions (the packed 128-bit integer set: parallel add, subtract, compare, shift, multiply-add, funnel and shuffle) [R ps2tek "EE Instruction Decoding"; Toshiba TX79 manual, §14 sources]; the two-way superscalar issue and the documented latencies; the short-loop hardware bug that GCC works around [R GCC manual, `-mfix-r5900`] | Instructions exact; timing as the documents give it, at instruction granularity (§5.2, Q14) |
| **COP0, TLB and memory** | Status, Cause, EPC, ErrorEPC, Count/Compare, the 48-entry TLB with its scratchpad-mapping S bit, the performance counters, the debug registers, EI/DI, ERET; the memory map: 32 MiB RDRAM, the 16 KiB scratchpad at `70000000h` reached only through virtual addresses, the 4 MiB boot ROM at `1FC00000h` [R ps2tek "Memory Map"]; the uncached and uncached-accelerated regions | Exact |
| **Caches** | 16 KiB instruction cache, 8 KiB data cache, both two-way [R ps2tek "EE Architecture"]; the CACHE instruction; write-back and its coherency with DMA, which games must manage themselves [R ps2tek, cache section] | Functional exactness first (a game that forgets to write back sees stale data as on a console); hit and miss timing as the timing model needs (§5.2) |
| **COP1, the FPU** | Single precision only; no NaN, no infinity, no denormals; rounding forced towards zero [R ps2tek "EE COP1"]; the accumulator instructions (MADD.S, MSUB.S, ADDA.S); the last bit after rounding, which ps2tek calls "not well understood" [R ibid.] | **Exact**, by software floats in the interpreter (§5.3); the thin area most likely to need disputes after the CDVD (P6) |
| **VU0 and VU1** | VU0 in **macro mode** (COP2: QMFC2, QMTC2, CTC2, CFC2, VCALLMS and the upper and lower instructions issued by the EE) and in **micro mode**; VU1 in micro mode only; 4/16 KiB of micro memory and 4/16 KiB of data memory [R ps2tek "VU"]; the upper (FMAC) and lower (integer, load/store, branch, FDIV, EFU on VU1) pipelines, the MAC, status and clipping flags with their pipeline delays, Q and P, XGKICK, the E bit and the D/T bits; VU0's view of VU1's registers | **Exact** results and the cycle at which a flag or Q is visible, since microprograms are scheduled by hand around those delays |
| **VIF0 and VIF1** | Every VIF command (NOP, STCYCL, OFFSET, BASE, ITOP, STMOD, MSKPATH3, MARK, FLUSHE, FLUSH, FLUSHA, MSCAL, MSCALF, MSCNT, STMASK, STROW, STCOL, MPG, DIRECT, DIRECTHL, UNPACK in every format and mask mode); VIF1's double buffering; stalls and interrupts | Exact data; the stall behaviour as ps2autotests' `dma/vif/*` records it |
| **GIF** | PATH1 (VU1's XGKICK), PATH2 (VIF1's DIRECT), PATH3 (GIF DMA); PACKED, REGLIST and IMAGE modes; arbitration between the paths and PATH3 masking [R ps2tek "GIF"] | Exact packets in exact order; arbitration as `dma/vif/mskpath3` records it |
| **GS** | 4 MiB of embedded DRAM [R ps2tek "GS"] in its page and block layout for every pixel, texture and depth format (PSMCT32/24/16/16S, PSMT8/4/8H/4HL/4HH, PSMZ32/24/16/16S); every primitive (point, line, line strip, triangle, strip, fan, sprite); flat and Gouraud shading; texturing with CLUTs, mipmaps, LOD, filtering and wrap modes; fog; scissor, alpha test, destination alpha test, depth test; alpha blending, PABE, FBA, colour clamp, dithering, frame and Z masks; host-to-local, local-to-host and local-to-local transfers; the privileged registers (PMODE, SMODE1/2, SYNCH, DISPFB1/2, DISPLAY1/2, EXTBUF, BGCOLOR, CSR, IMR, SIGLBLID); SIGNAL, FINISH and LABEL; the **PCRTC** with its two read circuits and merge, interlaced field and frame modes, NTSC, PAL and the progressive modes | **A software rasteriser first, exact in eDRAM after every primitive**; the Vulkan path and every enhancement later, the accurate path untouched (§5.4, §6) |
| **IPU** | Its commands (BCLR, IDEC, BDEC, VDEC, FDEC, SETIQ, SETVQ, CSC, PACK, SETTH); MPEG-2 variable-length decoding, inverse quantisation, the IDCT, colour conversion, dithering [R ps2tek "IPU"]; its FIFOs and DMA | Exact output against the MPEG-2 standard's arithmetic where the IPU follows it; the IDCT's rounding a likely dispute |
| **DMAC, INTC, timers** | Ten DMA channels (VIF0, VIF1, GIF, IPU from and to, SIF0, SIF1, SIF2, scratchpad from and to) with normal, chain and interleave modes, source and destination tags, the stall control, priorities; INTC's sources and masks; four EE timers on the bus clock, H-blank and their gates [R ps2tek "DMAC", "INTC", "EE Timers"] | Exact; transfer timing where `dma/*` measures it |
| **SIF** | The mailbox registers (MSCOM, SMCOM, MSFLG, SMFLG, CTRL, BD6) and the three SIF DMA channels between EE and IOP [R ps2tek "SIF Registers", "SIF DMA"] | Exact |
| **IOP** | An R3000A-family processor at about 36.864 MHz [R ps2tek "CDVD Reads and Seeks": "IOP_CLOCK is ~36,864,000 Hz"], 2 MiB of RAM, its caches and scratchpad (to be read at S2; the PlayStation's were 4 KiB and 1 KiB, `Diamond_Plan.md` §1.1; *read 2026-10-24 at S2: no public document gives the sizes; Sony said in 1999 the core is the PlayStation's "except for some minor enhancements to the cache" [R Microprocessor Report, 1999-04-19], so S21 starts from 4 KiB and 1 KiB [A], the enhancement a dispute candidate, `Sapphire_References.md` §5.1*), its interrupt controller, its DMA (channels 0–12, among them SPU2 on 7, SIO2 in and out on 11 and 12), six timers, three of 16 bits and three of 32 [R ps2tek "IOP DMA", "IOP Timers"] | Instructions exact through `blackmoon-r3000` (§9.2); timing to the bus access as Diamond's |
| **SPU2** | Two cores of 24 voices each [R Wikipedia, PS2 technical specifications, uncited there; to be confirmed at S2; *confirmed 2026-10-24 at S2 by ps2tek's register map, "Core0 Voice 0-23", and Microprocessor Report's "48-channel", `Sapphire_References.md` §5.2*], 2 MiB of sound RAM [R ps2tek], 48,000 Hz output, ADPCM voices with the PlayStation's interpolation and envelopes, AutoDMA streaming of PCM to the memory-in area [R ps2tek "SPU2 AutoDMA"], the effects (reverb) area per core, the IRQ addresses, the core mixing and the external input | **Exact** to the sample, every voice and the reverb |
| **CDVD** | The drive's N and S command sets, reads and seeks with their timing ("block_timing = (IOP_CLOCK * block_size) / read_speed", R ps2tek "CDVD Reads and Seeks"), CD-ROM modes and CD-DA, DVD-ROM single and dual layer with the layer break, the drive's status, the real-time clock, the console's EEPROM through the drive (§5.9), the tray; the drive controller (the "Mechacon") is a microcontroller running Sony's firmware [R ps2tek "CDVD Drive"], so it is modelled at its command interface as Diamond's HC05 is (§5.6) | Results exact; timing to the documented formula and to distributions measured on referees, the open item where documents are thinnest |
| **SIO2** | The port and its registers, "having no documentation whatsoever" by ps2tek's own words [R ps2tek "SIO2"]; the DualShock 2 with its configuration mode and **pressure-sensitive buttons** (00h–FFh per button, R ps2tek "SIO2 Standard Controllers"); the 8 MiB memory card (512-byte pages with 16 spare bytes each for error correction, 16 pages an erase block, the superblock "Sony PS2 Memory Card Format", R ps2tek "SIO2 PS2 Memcards", "SIO2 PS2 Memcard Filesystem"); the multitap; the card's authentication commands, whose purpose ps2tek calls unknown (§2.3, Q7) | Protocol exact; byte timing where a game measures it |
| **Regions and models** | NTSC and PAL timing; the console's region is the BIOS's; models as the player's BIOS versions distinguish them | Exact clocks, after D-0 |

### 1.2 What is left out, and why

| Left out | Why [C, §11 Q12] |
|---|---|
| **DEV9: the hard disk drive and the network adapter** | The network's servers are gone and online play is outside an accuracy-first emulator's first job; the hard disk serves a small number of titles (installs, and the few that require it) and is a medium of its own with its own file system. Both are a later device kind, proposed when the player gate is met |
| **USB** | Its devices (EyeToy, SingStar's microphones, keyboards, Buzz! buzzers, the later drum kits) are a family each, with host-side capture to design; a later device kind |
| **FireWire / i.LINK** | Present on early models only, used by a handful of games for multi-console play; there is nothing for it to connect to |
| **DVD-Video playback** | The DVD player is a separate firmware (`rom1`, the encrypted `erom`) and a separate product; Sapphire runs games |
| **The PlayStation (PS1) mode** | PS1 discs go to Diamond (§4.1), which is exact for them; Sapphire models the IOP in its PlayStation 2 mode only |
| **Arcade boards built on the PlayStation 2** (Namco System 246 and its family) | Different boards; a later core's question |
| **TOOL and development consoles** (128 MiB of RAM [R ps2tek "Memory Map"]) | Retail consoles first; a development console's extra memory is a later setting |

### 1.3 Disc images and other inputs

| Format | What it holds | Recommended [C, §11 Q16] |
|---|---|---|
| **ELF** (`.elf`) | A program side-loaded into RDRAM | **S1 on**: the project's own test programs and ps2autotests' EE tests arrive this way |
| **IRX** (`.irx`) | An IOP module, ps2autotests' IOP tests | Loaded through an IOP kernel: the player's BIOS (S24) or the open IOP kernel (O-track) |
| **`.iso`** | A DVD's 2,048-byte sectors, or a CD's | **S23** |
| **`.cue` with `.bin`** | The CD-ROM titles (a minority of the library, on blue-bottomed discs) | S23, through `emusen-disc` as Diamond reads them |
| **`.chd`** | MAME's compressed hunks; a DVD made with chdman's `createdvd` [R a libretro guide; a blog dating `createdvd` to MAME 0.255, single source] | S23, after `emusen-disc` gains the DVD layout (§9.4) |
| **`.m3u`** | A multi-disc set | S29 |
| **`.cso`/`.ciso`, `.zso`** | Block-compressed ISO forms | Later, each format from its published description |
| **`.gz`, `.mdf`, `.nrg`, `.img`, `.isz`, `.dump`** | Other dump and archive forms ES-DE lists for the system [R ES-DE `es_systems.xml`, `ps2`] | Later, or refused with words |

ES-DE's `ps2` entry lists `.bin .chd .ciso .cso .dump .elf .gz .m3u .mdf .img .iso .isz .nrg .zso` (and `.desktop`,
a launcher file) [R]; LRPS2's core information lists `elf iso ciso cue bin gz chd cso zso mdf nrg dump img m3u` [R
`pcsx2_libretro.info`]. The two agree but for `.cue` and `.isz`.

### 1.4 Who may read what

*Set 2026-10-23, at stage 0:* `Diamond_Plan.md` §1.3's rules, with the PlayStation 2's sources placed in them. A
**writer** writes Sapphire's or a shared crate's code, tests or design pages; a **replacement writer** writes the open
BIOS (§5.9); a **dispute step** is a supervised step whose only product is a disputes entry.

| Source | A writer | A replacement writer | A dispute step | Notes |
|---|---|---|---|---|
| Hardware documents (§2.1): ps2tek, the Toshiba TX79 manuals, GCC's notes on the R5900, ECMA-119 and ECMA-267, the MPEG-2 standard, Sony's published papers and talks, Wikipedia's specifications page | yes | yes, but ps2tek's BIOS chapter only as screened (§2.2) | yes | The primary sources |
| Sony's manuals distributed with the Linux for PlayStation 2 kit (§2.2) | per §11 Q1 | per §11 Q1, with the same screening | per §11 Q1 | Cited by document and section |
| Test programs: ps2autotests' binaries, their `.expected` files and READMEs (§3) | yes | yes, the expected texts and the tests' own sources; **not** ps2sdk, which the binaries link (§2.4) | yes | An expected file is a console's output |
| Reference emulators **run** as black boxes (§1.6) | yes, outputs only | outputs of games only; **never a log, picture or trace of a BIOS running** | yes | Pictures, sound, RAM |
| ps2sdk, Open-PS2-Loader, ps2link and other SDK or loader **source** (§2.4) | **no** | **no** | **no** | Run only as the binaries a test suite ships, per §11 Q3 |
| Any emulator's **source**: PCSX2 with its GS and SPU2 code and plugins (GSdx among them), LRPS2, PCEE2, Play!, DobieStation, hpsx64, and every other | **no** | **no** | **no** | §6.1 repeats it for the enhancements |
| Any FPGA RTL | **no** | **no** | **no** | No PlayStation 2 FPGA core is known [A]; the rule stands if one appears |
| The SIF link's traffic while the player's BIOS runs: the SIF DMA tags and the SIF RPC packets, read in ps2tek's documented layouts (`SifCmdHeader`, the `SifRpc*` packets, the IOP's chain tags) | a writer of a player's-BIOS stage only, never committed, each time recorded as exposure | **no** | as a writer | *Decided 2026-10-26 by the tester* (`Sapphire_Native.md` §36.14 Q4); never BIOS code, a program counter inside the BIOS or memory holding BIOS code or data |
| Firmware: the boot ROM, the DVD player's ROMs, the drive controller's program, the DualShock 2's and the memory card's; any listing, disassembly, dump analysis or trace of them | **no** | **no** | **no** | §5.9; the player's images are executed by Sapphire and by referees, never read |

**The exposure rule carries over from Diamond** [C]: anyone exposed to BIOS code, to BIOS boot banners or BIOS output in
a referee's log, to a test suite's BIOS-derived code, or to ps2tek's excluded sections (§2.2), never writes the open
replacement. Running the player's BIOS on Sapphire to compare its pictures with a referee's (S24) is such an exposure:
the writers of the player's-BIOS stages and the replacement writers are therefore different people, chosen before
S24 [C]. The three rules of `Nephrite_Plan.md` §1.3 hold unchanged: the order of recourse is documents, then a test
program written for the purpose, then the black-box referees, never an emulator's source; nothing read in a dispute
step is typed into code in that step; the code carries no emulator's identifiers, which G8's clone check tests.

*Decided 2026-10-26 by the tester, as `Sapphire_Native.md` §36.14 Q4 recommended:* a writer of a player's-BIOS stage
may trace the SIF link while the player's BIOS runs, narrowly: the SIF DMA tags crossing SIF0 and SIF1 and the SIF RPC
packets they carry, read in ps2tek's documented layouts and no further. The trace is an interface's traffic, but the
packets are data the BIOS's modules made, so it is an exposure of the same kind as running the BIOS: never BIOS code,
never a program counter inside the boot ROM or inside BIOS code in RAM, never memory holding BIOS code or data beyond
the packets themselves; the tracing code and its output are never committed, and each stage that traces records it in
its exposure section with what was looked at. A replacement writer never traces the link while a BIOS runs. Anything
beyond this, such as what the BIOS executes where it stops, remains the tester's decision, put as a question.

### 1.5 The disputes log, and why there is no hardware rung

`Sapphire_Disputes.md`, created with this page because a first disagreement between sources is already known: D-0, the
system clocks (§2.3). Disputes about the IOP's processor belong to `blackmoon-r3000`'s record (`BlackMoon_R3000.md`
§8), as Diamond's did; disputes about the R5900 go to its crate's record once it exists (§9.1).

There is no console rung and no FPGA rung. The tester's consoles are not used (the standing decision above) and no
public PlayStation 2 FPGA core exists [A, from the MiSTer project's published core list as commonly reported; to be
confirmed at S2]. The order of recourse is therefore the documents, then a test program, then the referees of §1.6,
weighed by lineage and by how each scores against the console logs ps2autotests ships (`Diamond_Referees.md` §3 and §6,
whose rule this core adopts: a ruling needs agreement between lineages, or names the strongest single witness as
such; a majority alone rules nothing).

### 1.6 The black-box references

| Referee | Form | Licence [R] | Without a BIOS | How it can be driven |
|---|---|---|---|---|
| **PCSX2** | Standalone Qt application (`pcsx2-qt`), GPL-3.0 [R GitHub API] | GPL-3.0 | needs the player's BIOS [A, from the libretro form's notes] | Documented flags `-batch`, `-nogui`, `-portable`, `-elf <file>`, `-fastboot`, `-slowboot`, `-bios`, `-logfile`, `-statefile`, `-unlimited` [R pcsx2.net "Command Line Arguments"]; RAM through **PINE**, its documented inter-process protocol (reads and writes of 8 to 64 bits and byte ranges, on a local socket, R the PCSX2 wiki's "IPC Protocol" page as a search engine quotes it: the page refused a direct read with 403); run under a private compositor as DuckStation is (`Diamond_Referees.md` §2.1) |
| **LRPS2** (`pcsx2_libretro`) | libretro core, a hard fork of PCSX2; on the buildbot, 4,630 KB, 2026-10-23 [R buildbot listing] | GPLv3 [R `.info`] | refuses: the `pcsx2/bios` folder is not optional [R ibid.] | `needs_fullpath`; **`hw_render`**, Direct3D 11, OpenGL Core 3.3 or OpenGL 3.0 [R ibid.] |
| **PCEE2** (`pcee2_libretro`) | libretro port of upstream PCSX2, "experimental"; 15,267 KB, 2026-10-23 [R] | GPLv3 [R] | refuses: BIOS folder and a `resources` folder from a PCSX2 installation, "the core will not boot without it" [R ibid.] | `needs_fullpath`, `hw_render` [R] |
| **Play!** (`play_libretro`) | libretro core and standalone; 2,432 KB, 2026-10-23 [R] | MIT [R `.info`] | runs: "a built-in high-level emulation BIOS. Using an external BIOS file is not necessary or possible" [R its README] | standalone `--disc`, `--elf`, `--state` [R ibid.]; expects CD and DVD images as CHD [R ibid.] |
| **DobieStation** | Standalone; last pushed 2021-07-25 [R GitHub API] | GPL-3.0 | — | Evaluated at S3 for a headless form; written by ps2tek's author (§2.1), so not independent of that document |

**Two findings that shape S3** [M]: the probe's libretro backend (`EmuSen.WiseMan/Reference/probe-rs/src/backends/
libretro.rs`) answers no `SET_HW_RENDER` request, so none of the three PlayStation 2 libretro cores can draw through it
as it stands; and the probe has no README of its own, its documentation being
`EmuSen_Debugging_Tools_Reference_v5.md` §3.50. A hardware context for the probe (headless EGL with OpenGL, or the
libretro Vulkan interface) is a feature of its own, evaluated at S3 against PCSX2's standalone form with PINE, which
needs no probe change (§11 Q4). *Measured 2026-10-24 at S3 (`Sapphire_Referees.md` §2.3, §6): the first finding holds
for `play_libretro` only, which asks for OpenGL core 3.2 and runs nothing without it; LRPS2 and PCEE2 fall back to their
software renderers, give the whole RAM, and PCEE2 hands the frontend pictures. PCEE2 reports version 2.9.138 and runs an
ELF with no `resources` folder. The hardware context is evaluated there and not built.* *Decided 2026-10-24 by the
tester: no hardware context now; Play!'s pictures taken as RAM through the project's GS programs, a headless EGL context
in reserve for the first GS dispute that needs it. DobieStation is not built now, and is built user-locally only if a
dispute needs a third lineage that ps2tek does not decide.*

**Lineage** [A]: PCSX2, LRPS2 and PCEE2 are one family and count as one witness where they agree; Play! is an
independent lineage; DobieStation is a third but tied to ps2tek. **Firmware for the referees**: the player's BIOS, as
Diamond's referees had theirs, through the probe's `--sysdir` or each application's BIOS folder set to a folder holding
only the tester's images, never read by any tool of the project (§5.9). Play! runs its own high-level BIOS whatever is
given; its outputs are those of a kernel of unknown provenance, so, like PCSX-ReARMed's for Diamond, they are used as
outputs of games only, and a replacement writer never sees its kernel's own output (§11 Q5).

### 1.7 What was read to write this page

No emulator source of any kind, no SDK or loader source, and no firmware. Read: EmuSen's own pages (`Diamond_Plan.md`
in full; `Diamond_Speed.md` §1, §2, §6, §7.2, §14; `Diamond_Referees.md` §1–§3; `Diamond_Disputes.md`'s rules and D-0;
`Mars_Native.md` §5.8; `Mars_Recompiler.md` §0–§1; `EmuSen_Firmware.md` §0–§2, §7.1–§7.4, §7.8–§7.9; `EmuSen_GpuPass.md`
§0, L4, §7–§9; `EmuSen_Gpu.md` §1; `EmuSen_Mips.md` §1–§2.4, §6–§7; `EmuSen_Disc.md` §1, §4, §5, §9.1–§9.2;
`EmuSen_MediumFormats.md` §1; `BlackMoon_R3000.md` §2; `EmuSen_Core_Naming_Scheme.md` §4); EmuSen's own code where cited
(the probe's libretro backend, `psx.json` and `psx/disc.rs` of the systems pack, MarsRT's `cpu/blocks/jit/mod.rs`'s
declarations, the line counts of Diamond's kernel); ps2tek in full as published at `1c91660`; the READMEs and the file
listings (names only) of ps2autotests, ps2sdk and the referees; libretro's core information files for LRPS2, PCEE2 and
Play!; the buildbot's listing; PCSX2's command-line page; GCC's MIPS options; the bibliographic records of Sony's EE
papers; Wikipedia's specifications page; ES-DE's `es_systems.xml`; the Batocera wiki's PlayStation 2 page; a forum
post of 2002 on the Linux kit's manuals. Listed, never opened: the names and sizes of the tester's 53 BIOS images
(§5.9).

---

## 2. Sources

### 2.1 The public documents

To be fetched at S2 into `~/.cache/emusen/probe/sapphire/docs/`, each file with its SHA-256 in `SHA256SUMS` and its
source, date and status in `DOCS.txt`, as Diamond's were. *Fetched 2026-10-24 at S2, `Sapphire_References.md` §1: every
row below but Sony's papers, of which the Hot Chips slides were fetched and the IEEE papers are behind paywalls;
Redump's BIOS datfile was added as the database of §5.9.*

| Source | Where, revision | Status | What it covers | Thin on |
|---|---|---|---|---|
| **ps2tek**, "PS2 documentation, themed after no$cash's documentation" | `PSI-Rockin/ps2tek`, **pinned at `1c91660`** (2022-01-27, the last push) [R GitHub API]; rendered at `psi-rockin.github.io/ps2tek` | Public, **no licence** [R GitHub API: none]; **no statement of its sources** [R the page]; written by DobieStation's author (214 of 224 commits by the same account as DobieStation's [R GitHub API]) and **in places quotes an emulator's choices** ("PCSX2 uses 00020603h as the version", in the Mechacon's version) [R "CDVD S Commands"] | Every unit: EE, COP0, COP1, VU, VIF, GIF, GS, IPU, DMAC, INTC, timers, IOP and its peripherals, CDVD, SPU2, SIO2, SIF and its RPC, and a long BIOS chapter (file structure, boot process, threading, syscalls, patches, the IOP modules' exports) | SIO2 ("no documentation whatsoever"), the FPU's last bit, the card's authentication, the Mechacon's version bytes, timing generally |
| **Toshiba TX79 Core Architecture manual** | `TMPR79xx-um-r2.0_200104.pdf` on bitsavers' mirror [R search; HTTP 200, 2026-10-23], and the TX7901 user's manual, rev. 6.30T (2001) | Public manufacturer's manual | The TX79, Toshiba's two-way superscalar 64-bit core: its instruction set including the multimedia instructions, pipeline, latencies, caches, COP0 and the FPU | **It describes Toshiba's commercial derivative, not the EE**: every difference from the EE is a dispute candidate (P6) |
| **GCC's MIPS options** | gcc.gnu.org, "MIPS Options" | Public | `-march=r5900`; `-mfix-r5900`, the short-loop bug: "loops to execute only once or twice, due to a hardware bug in the R5900 chip" [R] | The bug's exact conditions |
| **Sony's papers and talks** | Kunimatsu et al., "Vector Unit Architecture for Emotion Synthesis", IEEE Micro 20(2), 2000, 40–47; Suzuoki et al., "A microprocessor with a 128-bit CPU, ten floating-point MAC's, four floating-point dividers, and an MPEG-2 decoder", IEEE JSSC 34(11), 1999, 1608–1618; Kunimatsu's Hot Chips 11 slides (1999) [R dblp, a search engine's abstracts] | Published | The VUs' design, the EE's organisation; "5.5 Gflops at an operation frequency of 300 MHz" [R the abstract] | Designs, not the shipped silicon's every rule (D-0) |
| **ECMA-119** (ISO 9660) and **ECMA-267** (DVD-ROM, 120 mm) | ECMA's site, free | Public standards | The file system both disc kinds carry; DVD sectors with their ID, IED, CPR_MAI and EDC, the layers | The PlayStation 2's own disc authentication, which a dump does not hold |
| **ISO/IEC 13818-2** (MPEG-2 video) | ITU-T H.262, its twin, public | Public standard | The IPU's decoding arithmetic and its VLC tables | The IPU's own rounding and its non-standard commands |
| **Wikipedia, "PlayStation 2 technical specifications"** | en.wikipedia.org, read 2026-10-23 | Public, secondary, cites Sony's papers and Hennessy and Patterson in places | Clock, memory and fill-rate figures | Its figures conflict with one another (D-0) |
| **Batocera's wiki, PlayStation 2 page** | wiki.batocera.org | Public | Image names in the No-Intro form and their MD5s | — |

**CHD and chdman, credited (2026-10-25).** CHD (Compressed Hunks of Data) is MAME's image format, and chdman is MAME's
tool for making, checking and extracting it. chdman's source file is under the BSD-3-Clause licence, copyright Aaron
Giles; MAME as a whole is distributed under the GNU General Public License, version 2 or later ("The MAME License",
MAME Documentation 0.289, <https://docs.mamedev.org/license.html>). chdman's commands, codecs and defaults are
documented in "chdman – CHD (Compressed Hunks of Data) File Manager", MAME Documentation 0.289,
<https://docs.mamedev.org/tools/chdman.html>, read 2026-10-25; every chdman run the project records was of release
0.289. EmuSen reads CHD through the `chd` crate (BSD-3-Clause, Ronny Chan; `THIRD_PARTY_NOTICES.md`), whose source is
not read, and MAME's source is never read (`EmuSen_Disc.md` §1, §8).

### 2.2 Sony's Linux-kit manuals, and ps2tek's BIOS chapter

**The manuals.** Sony distributed its hardware manuals with the Linux for PlayStation 2 kit (beta 2001, release 1.0 in
2002): an Italian review of the beta lists the EE overview, the EE user's manual, the EE Core user's manual, the EE
Core instruction set manual, the VU user's manual and the GS user's manual among them [R everyeye.it], and a 2002 post
says they come as PDF files "a few hundred pages" long and that "the only documentation that Sony admit not to
supplying is for IO processor" [R empegbbs, tman, 2002-11-29]. They were sold with a consumer product rather than kept
under a developer's agreement, but they are Sony's copyright and their online copies are not Sony's own hosting.
**What they would give**: the EE, the VUs, the DMAC, the IPU and the GS from their maker, which is everything ps2tek
covers on the EE side. **What they would not**: the IOP, the SPU2, the CDVD and the SIO2. Whether writers may read them
is the tester's choice (§11 Q1); Wikipedia already cites the GS user's manual for its fill rate [R].

*Decided 2026-10-24 by the tester, for Q1: archived online copies of the manuals Sony shipped with the kit may be
read*, Q1's "lawful source" being taken to include them. Only the kit's own edition is read; a licensee SDK edition, or
any copy marked confidential for licensees, is refused at its title page. *Fetched 2026-10-24 at S6*
(`Sapphire_References.md` §1.1): the EE Core User's Manual, the EE Core Instruction Set Manual, the EE User's Manual,
the VU User's Manual and the GS User's Manual, with the EE Overview and the kit's list of restrictions beside them, each
"Version 5.0", "Publication date: October 2001", none marked confidential, taken file by file from the kit's own
Release 1.0 Disc 1 as the Internet Archive keeps it and checked against a second archived copy of the same disc. They
are kept in the probe's cache and never committed. Like ps2tek's BIOS chapter, they are screened for replacement
writers before one reads them (§1.4).

**ps2tek's BIOS chapter, screened for replacement writers** [C, set 2026-10-23]. The chapter's sections are "BIOS File
Structure", "BIOS Boot Process", "BIOS EE Threading", "BIOS EE Syscalls", "BIOS EE Patches", "BIOS PlayStation
Compatibility", "BIOS IOP Module Linking", "BIOS List of IOP Modules" and one section per IOP module [R the table of
contents]. **Excluded** for replacement writers: "BIOS EE Patches" ("The EE kernel is full of bugs, so games have to
work around them", R its first line), which by its subject describes the kernel's code that games replace;
"BIOS Boot Process"; and any passage with code or pseudo-code of the BIOS. **Screened first**, by someone who does not
write the replacement, and either cleared or added to the exclusions in `DOCS.txt`: the threading, syscall, linking and
per-module sections, which read as interface descriptions (names, numbers, arguments, export lists) but may mix in
behaviour learned from disassembly. ps2tek states no sources at all (§2.1), so its interface sections carry the same
caution psx-spx's did for Diamond, more strongly: its author also wrote an emulator.

*Screened 2026-10-24 at S2* by S2's writer, who is therefore a non-replacement writer. The exclusions by section and
line are `Sapphire_References.md` §3 (and `DOCS.txt` §X): beyond the three set above, the kernel's internal structures,
scheduler and call graph inside the threading and syscall sections, IOPBOOT whole, the PlayStation compatibility but
two lines, SYSMEM's and LOADCORE's internals, the boot banner, the reboot's internal sequence, and three traces of the
BIOS's I/O outside the chapter. The interface sections are otherwise cleared. **Found outside the chapter: two passages
of emulators' code** (the RDRAM initialisation's register model and the "IOP Console" hook), excluded for every writer,
and four lines quoting an emulator's choice.

### 2.3 Where the documents are thin

Each is a prediction of where the disputes log will fill (P6):

- **D-0, the system clocks.** ps2tek gives the EE at 294.912 MHz, the bus at "~147 MHz" and the IOP at "~36,864,000
  Hz" [R]. Wikipedia gives the EE at 294.912 MHz "(299 MHz on newer versions)", tagged there as failing verification,
  and the GS both at 147.456 MHz (uncited) and its bus at 150 MHz (citing Hennessy and Patterson), with a fill rate of
  2,400 Mpixel/s cited to the GS user's manual [R]. Sony's VU paper reports 300 MHz [R the abstract]. Arithmetic [A]:
  36.864 MHz is 768 × 48,000 Hz, the SPU2's rate, as the PlayStation's 33.8688 MHz is 768 × 44,100 Hz; 294.912 MHz is
  8 × 36.864 and 147.456 is 4 × 36.864, so ps2tek's family is one crystal with an audio-exact ratio; 2,400 Mpixel/s at 16
  pixels a clock is 150 MHz, while 147.456 MHz gives 2,359.3; and 299 MHz is not an integer multiple of 36.864 MHz
  (8.11). The readings therefore disagree on the clocks of later models and on whether the published figures are
  design targets. Entered as `Sapphire_Disputes.md` D-0.
- **The FPU's and the VUs' arithmetic** to the last bit: round towards zero is documented, the last bit "not well
  understood" [R ps2tek]; the accumulators, the clamping of large values, the flags' order. ps2autotests' `ee_fpu/*`,
  `vu0_macro/*` and `vu/*` are the oracles (§3).
- **The VU pipelines**: the flags' and Q's delays, the hazards of the integer unit after a load, XGKICK's timing,
  branch delay corners, VU0 macro mode's interlocks with the EE.
- **The EE's timing**: dual issue, the latencies, cache refill, RDRAM latency, the scratchpad's speed. No public suite
  measures it but `ee_cop0/performance` (§3).
- **GIF arbitration and PATH3 masking**, VIF stalls, DMAC interleave timing.
- **The GS's rasterisation rules**: edges, the top-left rule, the fixed point of texture coordinates and LOD, the
  filtering's weights, the order of the pixel pipeline's tests, the page and block layouts of the 4- and 8-bit
  formats, local-to-local transfers that overlap.
- **The CDVD**: every command's response time, seek times by distance and layer, the drive's status bits mid-command.
  The controller is firmware (§5.6) and no document describes its program.
- **SIO2**: undocumented by ps2tek's own words; the card's authentication (F0h, F3h, F7h: "Sent by SECRMAN. Unknown
  purpose", R ps2tek "SIO2 PS2 Memcards"), which the player's BIOS performs before it uses a card (§11 Q7).
- **The SPU2**: its differences from the PlayStation's SPU, AutoDMA's timing, the second core's routing, the reverb's
  addressing per core.
- **The IPU's IDCT rounding** and its colour conversion's dither.

### 2.4 SDKs and homebrew projects

These are open-source projects that may contain reverse-engineered BIOS code. **Their code is not read by anyone, at
any step** [C, §11 Q3]; this page read only their READMEs, licences and directory names.

| Project | Licence [R] | What its directory names show [R GitHub API, names only] | Classification |
|---|---|---|---|
| **ps2sdk** | Academic Free License 2.0 [R its LICENSE's heading] | `iop/system/` holds directories named after the BIOS's IOP modules: `sysmem`, `loadcore`, `threadman`, `intrman`, `modload`, `ioman`, `sifman`, `sifcmd`, `udnl`, `reboot` and others | **Probably contains reimplementations of BIOS modules**, of unknown method [A]. Never read. ps2autotests' binaries link it; running them is not reading it |
| **ps2toolchain** | BSD-2-Clause | The GCC and binutils build for the EE and IOP | A toolchain; its compilers could build the project's own test programs (§3.3), a question for the tester |
| **Open-PS2-Loader** | AFL-3.0 | A game loader with its own IOP modules | Never read; contains replacements of BIOS drive modules by its purpose [A] |
| **ps2link** | not asserted | The loader ps2autotests' results were taken under | Never read; its role in how the tests print is learned by running them (§3.1) |

---

## 3. The oracles, by component

### 3.1 What exists, and how each reports

To be fetched at S2 into `~/.cache/emusen/probe/sapphire/`, never committed, each file listed in `PROVENANCE.txt`. The
downloads are small (ps2autotests is 1,850 KB [R GitHub API]); nothing large is fetched at this stage. *Fetched
2026-10-24 at S2: the 78 programs counted below are there, 55 `.elf` and 23 `.irx`, each with its expected text; the
1,850 KB was the compressed history, the files being 10.5 MB [M]. Five files of its `common/` are ps2dev's or ps2sdk's
and excluded for every writer (`Sapphire_References.md` §2).*

| Oracle | Source, version | Licence | Covers | How it reports |
|---|---|---|---|---|
| **ps2autotests** (unknownbrackets) | `unknownbrackets/ps2autotests` @`97469ff` (2019-11-16, the last push) [R GitHub API] | ISC [R] | **78 programs with expected results** [M, the repository's tree]: EE `cpu/ee/{alu, branch, branchdelay, lsu, lsudelay, muldiv}`; `cpu/ee_cop0/performance`; nine `cpu/ee_fpu/*`; six `cpu/ee_simd/*` (the MMI); seven IOP `cpu/iop/*` including `hilodelay`; `cpu/vu0_macro/{integer, transfer}`; `dma/dmac/{cpcond, tagintr}`; `dma/spr/{chain, interleave, normal}`; `dma/vif/{basic, intstall, mskpath3, stcycl, stmod, unpack}`; `gs/{label, signal}`; eight EE kernel thread tests and two interrupt-order tests; sixteen IOP kernel tests (event flags, interrupts, libc, message boxes, threads, variable pools); `memory/ee/default`; `vu/lower/{branch, efu, fdivdelay, integer, random}`, `vu/upper/clip`, `vu/games/triace` | Text on standard output, compared whole with the `.expected` file [R README]; `.elf` tests run on the EE, `.irx` on the IOP [R ibid.]. The README describes running each on a console through ps2link and saving its output as the expected file [R], so **the expected texts are consoles' outputs**. The binaries are in the repository [M]. Built with ps2sdk, so they call the BIOS's kernel and, for output, whatever channel ps2link gave them; which channel reaches Sapphire is found at S4 by running them (P2) |
| **The kernel tests** of ps2autotests | the same | ISC | The EE and IOP kernels' thread, interrupt, event flag, message box and pool calls | Oracles of the **player's BIOS path** and, later, of the open BIOS's behaviour; their sources test an interface and are not BIOS-derived [A], so a replacement writer may read them |
| **The project's own test programs** | Written at each stage, assembled with `emusen-mips` (§9.1) or built for the EE's and IOP's targets (§5.9) | GPL-3.0 | Whatever a dispute needs; bare metal, needing no kernel | Results written to RAM at a fixed address, as Diamond's `programs/*.s` do (`Diamond_Referees.md` §2.4), so every referee can be read through PINE or the probe |
| **MPEG-2 streams** of the project's own making | Encoded with a public encoder at S27 | GPL-3.0 | The IPU, against the standard's decoder arithmetic | Decoded pictures compared exactly |
| **Referees** | §1.6 | — | Everything, run whole | RAM, pictures, sound |
| **Commercial discs** | the tester's own, **none in the library** [M: `~/Documents/Roms/` holds no PlayStation 2 folder, the memory note of 2026-10-19]; one on the handheld's card, not touched (§11 Q8) | not redistributable | Integration and the goldens | References at anchors |

*Found 2026-10-24 at S4 (`Sapphire_Native.md` §14.5): `cpu/ee/alu` and `cpu/ee/lsudelay`, run on PCSX2 with the
Americas and Europe BIOS, print their whole expected texts, `-- TEST BEGIN` to `-- TEST END`, on the **IOP's console**;
nothing of them reaches the EE's console, which a project program shows carries KPUTCHAR (1000F180h). An EE program's
text therefore crosses to the IOP before it is printed, and the IOP's console is the channel Sapphire must keep.*

**Negative results of the survey** [M, GitHub's organisation listing and search, 2026-10-23]: SingleStepTests publishes
no R5900 suite (its repositories include `r3000`, which the IOP's crate already passes, and none for the EE); no GS,
SPU2, IPU or CDVD suite with console results was found; ps2autotests' GS coverage is two programs, both about
signals. **Nothing like AccuracyCoin is known for the PlayStation 2**, and the picture side of the GS has no console
oracle at all: its exactness can be graded only by documents, by the project's own programs and by referees (P5).

### 3.2 By component

| Component | Primary oracle | Secondary | Thin (§2.3) |
|---|---|---|---|
| R5900 integer, branches, delays | ps2autotests `cpu/ee/*` | the project's programs; referees' RAM | dual issue |
| MMI | `cpu/ee_simd/*` | the project's programs over every encoding against the TX79 manual's definitions *(built at S8 against the EE Core Instruction Set Manual's chapter 3 instead, `Sapphire_Native.md` §18.5)* | — |
| COP0, TLB, caches | `cpu/ee_cop0/performance`, `memory/ee/default` | the project's programs; referees | timing |
| FPU | `cpu/ee_fpu/*` | referees *(neither lineage models exponent 255 or the multiplier; the texts carried S9, `Sapphire_Native.md` §19.6)* | the last bit |
| VU0 macro, VU micro | `cpu/vu0_macro/*`, `vu/lower/*`, `vu/upper/clip`, `vu/games/triace` | the project's microprograms; referees | flag delays |
| VIF, GIF, DMAC, INTC, timers | `dma/*`, `kernel/ee/interrupt/*` | referees | arbitration |
| GS | `gs/label`, `gs/signal` | **referees' pictures** of the project's programs; the documents | every rasterisation rule |
| IOP processor | SingleStepTests r3000 through `blackmoon-r3000`, already passing; `cpu/iop/*` | — | — |
| SIF, IOP DMA and timers | the project's programs; the kernel tests on the player's BIOS | referees | — |
| CDVD | synthetic discs read sector for sector; the documented read formula | referees' load timing | response times |
| SPU2 | the project's voice programs against referees' sound | — | reverb |
| IPU | the project's MPEG-2 streams against the standard | referees | IDCT rounding |
| SIO2, pads, cards | the protocol as documented; cards round-tripped with referees' card files | games | authentication |
| The player's BIOS path | the BIOS reaching its menu and booting games as on referees | the kernel tests | — |
| The open BIOS | games booting with no firmware folder; their cards readable under the player's BIOS; the kernel tests | the player's BIOS run as a black box | undocumented calls |

### 3.3 The runners

The generic test runner (`EmuSen.WiseMan/Fixtures/RomRunner/`, `Nephrite_Native.md` §3) serves Sapphire with
protocols added as data, each at the stage that needs it: **a TTY reader** over the output channels ps2autotests is
found to use (P2); **an expected-text grader** that compares a run's whole output with its `.expected` file after
normalising line ends, as the README asks [R]; **a synthetic ELF builder** in WiseMan (`SyntheticPs2Elf`, the counterpart
of `SyntheticRom`) and **a synthetic disc builder** (ISO 9660 with `SYSTEM.CNF`, single- and dual-layer DVD images,
CD images), so CI never needs a commercial disc; and **a RAM-record reader** for the project's own programs, shared in
form with Diamond's. *Built 2026-10-24 at S4 (`Sapphire_Native.md` §14): the expected-text grader (`ExpectedText`), the
RAM-record reader (`RamRecordForm`, Diamond's and Sapphire's forms), `SyntheticPs2Elf` with its recording program, and
`SyntheticPs2Disc` with CD sets and dual-layer DVDs on one ISO 9660 writer shared with the PlayStation's builder; the TTY
reader takes the channel's space as data, the IOP's console once Sapphire keeps it.*

### 3.4 The conformance kit's corpus form

Sapphire runs `emusen-core-conform` as every v1 core does. Two of the corpus form's assumptions break on DVDs [A]:
it sizes a job at 256 MiB an image (`EmuSen_CoreAPI.md` §28.4), and a DVD image is up to about 8.5 GB, so it cannot be
copied at create as Diamond's discs once were. The host's mapped backing (`EmuSen_Disc.md` §9.1, the kit's `Kept`) is
therefore required for DVDs, and the kit's memory estimate must count a mapped image as mapped (§10).

---

## 4. What a frontend gets

Nothing here needs per-core code in a frontend (`EmuSen_CoreAPI.md` §2.4).

### 4.1 The system, and how PlayStation discs reach Diamond

| id | Name | Extensions | ES-DE | libretro cheats | OpenVGDB | ScreenScraper |
|---|---|---|---|---|---|---|
| `ps2` | Sony PlayStation 2 | `.elf`, `.iso`, `.chd`, `.cue`, `.bin`, `.m3u`; the compressed forms later (§1.3) | `ps2`, "Sony PlayStation 2" [R] | **none**: the database's `cht/` folder has no PlayStation 2 entry [M, GitHub API] | **none**: OpenVGDB's `SYSTEMS` table has no PlayStation 2 row [M, a read-only query of the project's copy] | 58 [A, to be confirmed against ScreenScraper's list at S29] |

**Routing.** `.iso`, `.chd`, `.cue`, `.bin` and `.m3u` are claimed by Diamond, Nephrite's Sega CD and Sapphire alike, and
the host routes them by content through the system pack's recogniser, as it has since the core ABI's routed system
(`EmuSen_Backend.md` §31, minor 1.15). The PlayStation 2's recogniser [C, A]:

1. a disc whose ISO 9660 root holds `SYSTEM.CNF` with a **`BOOT2`** line (`BOOT2 = cdrom0:\SLUS_xxx.yy;1`) is a
   PlayStation 2 disc, CD or DVD; one with a **`BOOT`** line and no `BOOT2`, or with no `SYSTEM.CNF` but a `PSX.EXE`,
   is a PlayStation disc and goes to **Diamond**, whose recogniser (`Platform/systems/src/psx/disc.rs`) already reads
   the licence text and `BOOT`; a Sega CD is told by `SEGADISCSYSTEM` as now;
2. a DVD-sized image (more than a CD can hold) with `BOOT2` is the PlayStation 2's without further question;
3. an `.elf` is the PlayStation 2's (no other system claims it; a MIPS ELF of the IOP is an `.irx`).

The `BOOT2` key is the format's common knowledge [A]: no document read here states it, and ps2tek's "BIOS Boot Process"
is excluded for replacement writers (§2.2), so the recogniser's writer, who is not one, confirms it from that section
at S1 and cites it. **Why Diamond takes PS1 discs** [A]: Diamond is exact for them, has its own open BIOS and
enhancements, and a PlayStation 2's PS1 mode would be a second, lesser PlayStation emulator inside Sapphire, against
the reuse rule. A player who opens a PS1 disc through the PlayStation 2's shelf is routed to Diamond with no setting.

**The serial and the region.** The serial is `BOOT2`'s file name (`SLUS_203.12` is SLUS-20312), as Diamond takes
`BOOT`'s; `SYSTEM.CNF`'s `VMODE` gives NTSC or PAL [A, from the format as referee logs print it; to be confirmed at
S29]. The console's region is the BIOS's, chosen per market (§5.9).

### 4.2 Controllers, and the drawing

Two ports, more with the multitap:

- **`ps2.dualshock2`**: Select, Start, the cross, △ ○ × □, L1 R1 L2 R2, L3 R3, two sticks, the Analog button and its
  LED, and **pressure** on the cross, the four face buttons and the four shoulders, 00h to FFh each [R ps2tek "SIO2
  Standard Controllers"].
- **`ps2.digital`** answers as a digital pad, the DualShock 2 in digital mode, for games that ask for it.

**Canonical controls** [C]: Diamond's mapping (× → B, ○ → A, □ → Y, △ → X, the shoulders by name, the sticks to the two
sticks, `Diamond_Plan.md` §4.2), so a player's bindings carry from one Sony system to the other. **Pressure has no
place in v1** [A]: `PadButton` is digital and append-only, and `PadAxis` holds sticks. Until a minor adds per-button
analogue values (§10), a pressed button reads FFh and a released one 00h, which is what a digital mapping can give and
what games accept from a pad held fully [A]; the pressure model in the core is built at S25 so that the minor needs no
core change. **Rumble** goes through the output event Diamond's plan asked for (`Diamond_Plan.md` §10). **The Analog
button** is Diamond's key binding. **The drawing** in Controller Bindings: a DualShock 2 drawn with LunaP's
`ControllerDiagram`, as Diamond's two were (`Diamond_Plan.md` §4.2), fit-audited at 1280×800 and 1920×1200.

### 4.3 Spaces

`RAM` (32 MiB, the cheats' space), `SCRATCH` (16 KiB), `GSMEM` (4 MiB eDRAM), `VU0MICRO`, `VU0DATA` (4 KiB each),
`VU1MICRO`, `VU1DATA` (16 KiB each), `IOPRAM` (2 MiB), `SPU2RAM` (2 MiB), `BIOS` (read-only, 4 MiB, whichever kernel
runs), `CARD0`/`CARD1`; the processors `R5900` (id 0), `IOP` (id 1, `blackmoon-r3000`'s), `VU0` and `VU1`, each with
its bus, and their registers through `DEBUG_REGISTERS`. The R5900's general registers are 128 bits wide; whether
`DEBUG_REGISTERS` carries a 128-bit value is read in `EmuSen_CoreAPI.md` at S1, and a minor proposed if it does not (§10).

### 4.4 Memory cards and the console's own memory

- **Cards** are battery files, file 0 the card in slot 1 and file 1 in slot 2, further files for a multitap's. **The
  format is the raw card with its spare bytes**: 512-byte pages each followed by 16 bytes of error correction [R ps2tek
  "SIO2 PS2 Memcards"], 16,384 pages for 8 MiB of data, 8,650,752 bytes in all [A, arithmetic], which is what other
  emulators' `.ps2` card files hold [A; checked against the referees' cards at S25]. The medium is `ps2.card` under
  `EmuSen_SaveMedia.md`'s model, *Per game* by default as Diamond's, and its file system is a new crate (§9.5).
- **The console's EEPROM**, which the drive controller reads and writes for the BIOS (its settings, its model and
  region parameters) [R ps2tek "CDVD S Commands", in outline], is a `console` medium, `ps2.nvm` (§5.9). It is data, not
  firmware: Sapphire writes a blank-but-valid one on first use from the documented layout, and keeps it per console
  model.

### 4.5 Cheats

The libretro database holds no PlayStation 2 cheats [M]. The cheat devices of the era (Action Replay MAX, CodeBreaker,
GameShark 2 and later) publish their codes encrypted, and decrypting them means following algorithms learned from those
devices' software [A]; PCSX2's patch files are an emulator's own format. **Recommended** [C, §11 Q17]: no codec at the
player gate but plain RAM writes in `RAM`, typed or imported as address and value; the devices' formats later, each
from a published description of its encryption or not at all.

### 4.6 The CRT filter

The PlayStation 2 is a television console, so `EmuSen.Serenity/Shaders/ConsoleScreens.cs` gains `new("PlayStation 2",
Kind.Television, "PlayStation 2")` with **its own profile** when the system reaches players (S29), and
`ConsoleScreenRuleTests` holds it. `CrtFilter.ConsoleTiming` needs the PCRTC's active picture in microseconds and the
subcarrier phase per line and frame (`EmuSen_CRT.md` §3.2). The PCRTC's video clock and its dividers are in ps2tek's GS
chapter and in Sony's GS manual (§2.2); the entry **waits on D-0** and on S20's reading, as Diamond's waited on its own
D-0. The progressive and 1080i modes some games offer need the profile's answer for a 31 kHz picture, which is
`EmuSen_CRT.md`'s question, not this core's.

### 4.7 States, settings, the debugger

- **The state** is Sapphire's own, magic `SPH2`, written with `emusen-native`'s `StateWriter`: the magic, a version,
  the frame count, the disc's identity, then each block in a fixed order. About 41 MiB [A: RDRAM 32, eDRAM 4, IOP RAM
  2, SPU2 RAM 2, the VUs' 40 KiB, the scratchpad, the caches with their tags, the FIFOs, registers]. The disc is never
  in it. At 41 MiB a state is twelve times Diamond's, so the rewind buffer's delta codec (`EmuSen_Rewind_And_
  FastForward.md`) is measured on it at S30 before rewind is offered (P14). *Built 2026-10-25 at S30 (`Sapphire_Native.md`
  §38.2): the header also names the market, the BIOS entry by its file number and the EE's stop; the picture follows it,
  the one block whose length varies; 41.2 MiB with a 640 × 448 picture. The cards' images and the EEPROM's bytes are the
  player's media, kept apart (§38.3).*
- **Settings**, each with the stage that makes it meaningful: the BIOS per market (§5.9), the pad per port, the
  multitap, the card mode, then the speed levers (`effect: exact`) and the enhancements (`effect: enhancement`, §6).
  Defaults are a real console's.
- **The debugger** is the generic one over `DEBUG`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`, with four processors:
  the R5900 and the IOP through their observers, VU0 and VU1 through theirs, the VUs' disassembly from `emusen-mips`
  (§9.1).

---

## 5. Design decisions

### 5.1 The clocks and the scheduler

- **One master clock** [C, pending D-0]: 36.864 MHz × 8, the EE's 294.912 MHz, of which the bus and GS run at half and
  the IOP at an eighth; the SPU2's 48 kHz is 6,144 EE cycles [A, arithmetic]. Every unit's time is an exact integer
  of EE cycles, so nothing drifts.
- **Every bus access advances the clock** on the IOP as Diamond's does; the EE is timed at instruction granularity
  (§5.2). A scheduler holds the fixed events (the PCRTC's lines and fields, the SPU2's samples, the CDVD's sector times,
  the timers' targets) as times.
- **Catch-up and the units that run beside the EE.** VU1 runs a microprogram in parallel with the EE once kicked; the
  GS draws while the EE computes; the IOP is a second computer. Each is brought to the present only where another can
  observe it: VU1 at a VIF1 or EE access to its memories or registers, at XGKICK's packet and at the E bit; the GS at a
  privileged register read, a local-to-host transfer, SIGNAL, FINISH and LABEL; the IOP at SIF registers and SIF DMA.
  Exact where the observation is the only coupling [A], which the project's programs test at each stage, and the
  basis for running those units on threads later without losing exactness (§5.12).
- *Built 2026-10-25 at S11* (`Sapphire_Native.md` §20.2, §20.3): the scheduler did not exist before S11; each unit now
  reports its next event (a DMA unit's end, a timer's flag, a blanking edge) and the board takes the earliest, as
  Diamond's bus does, nothing polled cycle by cycle; the clock's figures are `clock.rs`'s, D-0's one place.

### 5.2 The EE's timing model

**Recommended [C, §11 Q14]: the documented timing at instruction granularity, with the caches modelled.** Each
instruction is charged the issue and latency the TX79 manual gives (§2.1) with the two-way pairing rules it states, a
stall where a result is used before it is ready, the multiplier and divider pipelines, a cache miss its refill from
RDRAM at the documented cost, uncached accesses their bus cost, and the scratchpad a single cycle. **Not modelled at
first**: the pipeline's every interlock below what the manual tabulates, and RDRAM's page behaviour, each listed as a
named loss and a dispute candidate. **Why not cycle-exact like Diamond's R3000A**: no suite measures the EE's timing
but one performance-counter test, the manual describes a commercial derivative, and every EE cycle costs host time at a
clock nine times the PlayStation's (§5.12); the model is made exact to what can be graded, and its losses are named
rather than guessed. **The interpreter is the reference**: every recompiled tier reproduces its registers, memory,
events and cycle counts bit for bit (§5.12).

*Amended 2026-10-24.* The model above stays the reference's. Whether the default path keeps it, or charges a fixed
cost an instruction or a block in its place, is compromise C1 of §5.13, decided by measurement at S32 and S36. The
recompiler's bit-for-bit claim is made against the interpreter with every compromise off; each compromise is then
measured separately against that pair.

*Built 2026-10-24 at S6* (`Sapphire_Native.md` §16.4): the tables are the *EE Core User's Manual*'s own (§1.3.5, §1.3.6,
§2.12), which equal the TX79's on every integer row and differ on the floating-point ones; the caches wait for S7.
*Built 2026-10-25 at S7* (§17.5 there): the caches modelled with the documented refill burst as their cost, RDRAM's
latency still the named loss. *Built 2026-10-25 at S8* (§18.4 there): the multimedia instructions in Wide Operate, the
multimedia multiply and divide on both multipliers; PLZCW's and QFSRV's untabulated latencies named losses. *Built
2026-10-25 at S9* (§19.5 there): the floating-point rows with the FPRs, ACC and FCR31 scoreboarded and one divider for
DIV.S, SQRT.S and RSQRT.S, the rows §2.12 omits named losses.

### 5.3 Floating point, exact

The FPU and the VUs follow the PlayStation 2's own format: no NaN, no infinity, no denormals, rounding towards zero,
results clamped where the documents say [R ps2tek]. The interpreter computes them in **software floats on integers**,
as MarsRT's COP1 does (`Mars_FpuMath.md`), so the result is the same on every host and in every tier. The last bit after
rounding is the first open rule (§2.3), graded by `ee_fpu/*` and `vu/*`. A recompiled tier may use the host's floating
point only where a test over every class of input shows it equal to the software path, with the host's rounding mode
set to truncation and denormals flushed; any input class where it is not equal stays on the software path [C]. This is
the largest single cost of exactness in the VU recompiler and is priced in §5.12.

*Amended 2026-10-24.* The software floats stay the reference and the rule above stays the exact tier's. Host floats
with clamping on input classes **not** proven equal become a candidate for the default path, compromise C3 of §5.13,
decided on the classes' measured differences in `ee_fpu/*`, `vu/*` and the bench's anchors against the VU1
recompiler's measured gain.

*Built 2026-10-25 at S9* (`Sapphire_Native.md` §19): `sapphire::float`, used by the FPU and owed to the VUs. "Rounding
towards zero" holds of an inexact intermediate, not of the exact result: the adder aligns without a sticky bit, the
multiplier loses its lowest Booth column, and DIV.S and SQRT.S round to the nearest, as the console texts show
(`Sapphire_Disputes.md` D-25 to D-28); C3's classes now include every product, every sum that loses bits to
alignment, and every quotient and root (§19.8 there).

*Built 2026-10-25 at S12* (`Sapphire_Native.md` §21.3): the VUs compute on the same module, every rule of the arithmetic
shared and only the flags their own; that the VU's adder, multiplier and divider are the FPU's console-decided ones is
argued from the manuals (`Sapphire_Disputes.md` D-43), and both referees compute the VU on IEEE 754's rounding towards
zero, so S13's console texts are to decide it.

### 5.4 The GS: an exact software reference, then the device

- **The software rasteriser is the reference** and owns the machine's eDRAM: every primitive drawn whole into the 4 MiB
  in its page and block layout, so that a texture read from a reinterpreted render target, a local-to-host transfer or
  a CLUT loaded from eDRAM sees the bytes a console would. The pixel pipeline in the documented order: scissor, the
  tests, texture lookup with its LOD and filter in the documented fixed point, fog, blending, dithering, colour clamp,
  the masks. The PCRTC reads the eDRAM at its own time each line, so a mid-frame change shows where the hardware shows
  it.
- **`skip_rendering_state_neutral` is not claimed** [A], as Diamond's is not: eDRAM is machine state that games read
  back and reinterpret routinely, so a frame may skip presenting, never drawing.
- **The Vulkan path is a second implementation of the same rules, held equal to the first** [C]: a compute rasteriser
  over a mirror of the eDRAM on `emusen-gpu`, the form Diamond's renderer and MarsRT's RDP took (`EmuSen_Gpu.md` §6,
  `Diamond_Enhancements.md` §14), chosen over a graphics pipeline because a graphics pipeline's rasterisation and
  blending are the host GPU's rules, not the GS's, and could not be held bit-exact. It is an **exact speed lever**
  (`effect: exact`), offered only when its eDRAM equals the software rasteriser's after every frame of every program and
  bench game (S44–S46). It follows `EmuSen_GpuPass.md`'s decisions of 2026-10-20: the device is one and lendable and made
  by whoever presents (its §7), so the GS uses the lent device through `emusen-gpu`'s seam (`EmuSen_Gpu.md` §5) and never
  assumes it owns one; its picture is handed on the device through the core ABI's picture slot, as L4 plans for MarsRT
  (`EmuSen_GpuPass.md` L4, assigned to MarsRT's pass by its Q4); the chain's output format stays a parameter (its §7);
  and its tests run on the RX 6800 only, one GPU-heavy run at a time.
- **Enhancements** (internal resolution and the rest) are a third renderer that never writes eDRAM (§6).
- *Amended 2026-10-24: the GS's default.* The software rasteriser stays the reference, the grader of every GS stage
  and the witness in every GS dispute, and a setting a player may choose. **The device path becomes the intended
  default**, since the handheld's full speed is the target and the software path's own estimate (3–30 ms a frame on
  one desktop thread, §5.12) leaves no room for the rest of the machine on the handheld. It is offered as the default
  when it equals the reference, or, if it does not, when its departures are measured and named. Whether it stays a
  compute rasteriser held bit-exact, or relaxes rules where the bit-exact form costs most (the host's rasterisation and
  blending in a graphics pipeline, eDRAM synchronised lazily), is compromises C5 and C6 of §5.13, decided at S44–S46
  on the eDRAM and picture differences per bench game against the reference and the GS's frame time on the handheld's
  GPU. The rule that the enhancements never write eDRAM is unchanged.

### 5.5 The SPU2

Every voice of both cores runs each sample: ADPCM, the interpolation, ADSR, sweep, noise and pitch modulation as the
PlayStation's SPU does them, from the shared voice crate (§9.3); then each core's mix, its effects area, AutoDMA's
PCM, the routing of core 0 into core 1 and the output, **48,000 Hz exactly, a fixed rate** reported as Diamond's 44,100
is. What differs from the PlayStation's SPU is learned from ps2tek's SPU2 sections and the referees' sound, each
difference a dispute where they disagree.

### 5.6 The CDVD drive controller

The Mechacon runs Sony's firmware [R ps2tek "CDVD Drive"], so **the firmware policy covers it** (`EmuSen_Firmware.md` §0):
Sapphire's model of its N and S commands at the register interface, written from ps2tek's descriptions, is its open
replacement, as Diamond's model of the HC05 is (`Diamond_Plan.md` §5.6). No image of it is asked for or accepted.

- **Reads and seeks** use ps2tek's formula for a block's time and measured seek distributions; the dual layer's second
  layer and its layer break are taken from the image (§9.4); DVD reads report what ECMA-267's sector header gives.
- **Disc authentication** happens between the drive and the disc's physical marks, which no dump records [A]. The
  model reports every disc as authenticated for the BIOS's region, states so in the machine info's words, and leaves a
  region mismatch to the player as Diamond does with SCEx.
- **The version and the EEPROM**: the drive's version bytes are ones ps2tek does not explain and quotes from PCSX2
  ("Unknown what the bytes mean", "PCSX2 uses 00020603h") [R], so the value is chosen at S23 by a dispute step from the
  referees' behaviour, never copied from the quotation; the EEPROM is §4.4's medium.
- **MagicGate** keys live in the drive [R ps2tek "SECRMAN"] and are Sony's secrets: never sought, never used (§11 Q7).

### 5.7 The IPU

MPEG-2 decoding as ISO/IEC 13818-2 defines it, with the IPU's commands, FIFOs and colour conversion from ps2tek and
(per Q1) Sony's manual. Exact output graded against the standard's arithmetic on the project's own streams; the IDCT's
rounding where the IPU departs from the standard's reference is a dispute (P7). FMV smoothness depends on its timing
under DMA, which is measured.

### 5.8 SIO2: pads, cards and the multitap

Each device is a state machine on its documented protocol: the DualShock 2's identification, configuration mode,
analogue and pressure modes and rumble; the card's probe, page reads and writes, erase and termination with the
card's real write and erase times, since games show their saving screens for them; the multitap's four-port frame. The
card's authentication commands are answered as §11 Q7 decides; until then the open BIOS path never sends them and the
player's BIOS path is expected to refuse the card (P8).

### 5.9 Firmware

**The player's BIOS first** (decided 2026-10-19). The boot ROM is 4 MiB at `1FC00000h` [R ps2tek], holding the EE
kernel, the IOP kernel and its modules, OSDSYS (the menu) and the PlayStation compatibility driver, as "dozens of
separate files" indexed by a ROMDIR [R ps2tek "BIOS"].

- **What the path needs**: the EE, the IOP, the SIF, the GS with its PCRTC, the CDVD with its S commands and the
  EEPROM, SIO2 with pads, and the INTC and timers on both sides; the BIOS reaches OSDSYS with all of them (S24). The
  BIOS's own region decides the console's region.
- **How it is declared** [C, §11 Q9]: three entries, `ps2.bios.japan`, `ps2.bios.americas`, `ps2.bios.europe`, each
  4,194,304 bytes, `required: false`, labelled for the firmware page as "PlayStation 2 BIOS (Japan)", "(Americas)" and
  "(Europe)", with the open replacement's `effect: accuracy` once it exists. **Canonical names** in the No-Intro form the
  tester's images carry and Batocera documents [R]: `ps2-0230a-20080220.bin` and `ps2-0230e-20080220.bin` for the
  Americas and Europe; for Japan the newest Japanese retail version listed by the public database chosen at S2. **Each
  entry lists the MD5s of every retail version of its market**, cited from a public database (`EmuSen_Firmware.md` §0
  item 4 and §7.8), so a player's file is found by its content whatever its name; Batocera's page already gives one,
  `21038400dc633070a78ad53090c53017` for `ps2-0230a-20080220.bin` [R], and the rest are cited at S2. Asia (`h`), China
  (`c`) and Russia (`r`) versions in the tester's set are recognised as alternates of the market whose games they boot,
  per the database's description, never by reading them. *Done 2026-10-24 at S2 (`Sapphire_Native.md` §12.3): the
  database is Redump's PS2 BIOS datfile (115, 2022-12-12); the newest Japanese retail version is
  **`ps2-0230j-20080220.bin`**, so the Japanese entry carries that name; the lists hold 27, 15 and 16 retail MD5s,
  development consoles left out, Korea (`k`) joining Asia and China as Japan's.*
- **The tester's images** [M, a listing of names and sizes only]: 42 in `~/Documents/BIOS/PS2/` and 11 in its `Misc BIOS/`,
  every one 4,194,304 bytes, named `ps2-VVVVr-YYYYMMDD.bin` from version 0100 (2000-01-17) to 0230 (2008-02-20), with
  region letters `j`, `a`, `e`, `h`, `c` and `r`; the folder `Misc BIOS/` adds names with a second letter `d` and one
  `0250j` of 2010. Later stages use them in place, read-only: the firmware folder setting (`EmuSen_Firmware.md` §2.3)
  for Sapphire, the probe's `--sysdir` and each standalone referee's BIOS folder pointed at a folder that holds links to
  them; no tool of the project opens, hashes or diffs them, except `firmwarecheck.py` against the open replacement
  (§5.9's later track, as the policy says). Which versions the gates use is §11 Q9.
- **What else a console holds that is not taken**: the DVD player's ROMs (`rom1`, `erom`) and the Chinese font ROM
  (`rom2`), since DVD playback is out of scope (§1.2); the drive controller's program (§5.6); the EEPROM, which is data
  and is synthesised (§4.4).

**The open replacement, a separate later track** (§7, O-stages). Its parts: the **EE kernel** (the reset path, the
exception and interrupt dispatch, about 130 syscalls for threads, semaphores, interrupt and DMAC handlers, alarms, SIF
DMA, the GS's CRT setup, program loading by `LoadExecPS2`, and the TLB's setup [R ps2tek "BIOS EE Syscalls", by its
table's extent, screened per §2.2]); the **IOP kernel and modules** (memory, the module loader and linker, interrupts,
threads with semaphores, event flags, message boxes and pools, the I/O manager and file server, the SIF command and RPC
servers, the CDVD file system, SIO2, pads, the card's file system and its server, the sound library, the IOP's reboot
with a replacement module image); and, optional, a **menu** of EmuSen's own in OSDSYS's place.

| Form | What it is | Games that work | Verdict [A] |
|---|---|---|---|
| **A. Native kernel images** (Diamond's form) | The EE kernel as R5900 code and the IOP kernel and modules as IRX modules for the R3000A, in a 4 MiB image at the ROM's address, run by the emulated processors | Every game that hooks, patches, or loads its own IOP modules against the kernel's exports finds code, tables and module exports where the interface says; the IOP's timing is a program's, as on a console | **Recommended** |
| **B. High-level IOP** | The IOP's services answered in Rust at the SIF's RPC level, with no IOP code | Games that use only the BIOS's modules | **Fails most of the library** [A]: games ship their own IOP modules on disc and reboot the IOP into module images of their own, all of which need a real IOP kernel's exports and threads |
| **C. EE syscalls trapped** | Syscalls answered in Rust | Games that only call syscalls | Breaks games that install kernel patches or handlers into kernel memory (ps2tek's excluded "BIOS EE Patches" exists because they do) |
| **D. The player's BIOS only** | — | All, exactly | The path until the open track's gate; the policy requires A before the system is offered with no firmware folder (G9) |

**Recommended: A** [C, §11 Q6]. Language: the IOP modules in Rust, `no_std`, for `mipsel-sony-psx`, the R3000A's target
Diamond's kernel already builds with its pinned nightly; the EE kernel in Rust for a MIPS target restricted to the
instructions the R5900 implements, since **rustc has no R5900 target** [R a search of rustc's platform pages], checked by
a CI scan of the built image with `emusen-mips`'s R5900 decode that refuses any instruction the R5900 lacks, with a small
assembly entry for the vectors. Both images committed with their sources and hashes and rebuilt by CI to the same
bytes, as Diamond's is.

**Its size, honestly** [A]. Diamond's open kernel is 6,285 lines with a 1,106-line shell [M, `wc -l` of `kernel/` and
`kernel-shell/`]. The PlayStation 2's EE kernel is of the same order or up to twice it (about 130 syscalls against
Diamond's three hundred smaller calls, but with a scheduler, the TLB and program loading): **6,000–12,000 lines**. The
IOP side is a multithreaded operating system with a dynamic module linker and some 25 modules, each with an export
table games link against by name and index: **20,000–35,000 lines**. With a minimal menu and the build: **30,000–50,000
lines, five to eight times Diamond's kernel**, about **25–40 steps** (P10). The card's file system is shared with the
host's card tools through §9.5's crate, which an open firmware may build on, as `EmuSen_MediumFormats.md` §1 allows.

**Accuracy cost**, in a player's words first: "Most games start and save as on a console. A game that relies on an
undocumented BIOS behaviour, reads the BIOS's code, or depends on its exact timing may behave differently, the start-up
screen is EmuSen's own, and there is no DVD player." **Risk: higher than Diamond's** (P10), because the IOP's module
interface is wide, documented only as export lists, and exercised by Sony's own modules on game discs.

**Exposure** (§1.4): anyone exposed to BIOS code, to BIOS boot banners or output in referee logs, to ps2tek's excluded
sections, or to a test suite's BIOS-derived code never writes the replacement. ps2autotests' kernel tests are interface
tests and may be read by replacement writers (§3.1); their expected texts were produced on a console's BIOS and are
outputs of calls, not banners, and are allowed [C].

### 5.10 The state format

`SPH2` (§4.7): the layout listing pinned by a test from version 1; an older version loaded in its own layout back to the
oldest a player has had, as Nephrite's `OLDEST_STATE_VERSION` does. *Amended 2026-10-25 at S30 (`Sapphire_Native.md` §38.4):
version 1 (S24, the memories alone) was never a player's; version 2 holds every unit, its listing pinned against its file,
and is the oldest read.*

### 5.11 Disc images and the ABI

Diamond's arrangement holds (`Diamond_Plan.md` §5.11): the sheet or the image is passed to `create`, the system pack
reads only a sheet's `FILE` lines, `emusen-disc` parses and reads the rest. **One change** [C, §11 Q16]: a DVD image is
always opened on the host's mapped backing (`Kept`), never copied; a host that cannot map is refused with words for
images over 2 GiB rather than made to copy 4–8.5 GB. Measured on the handheld at S23 (P13).

### 5.12 The speed budget, honestly

**The machines** (`Diamond_Speed.md` §1): the desktop, a Ryzen 7 7700X with an RX 6800; the handheld, the Legion Go S
(Ryzen Z1 Extreme, 16 threads), argued 1.3–1.4 times slower a thread than the desktop (`Nephrite_Plan.md` §5.5); the
laptop, an i5-10310U with Intel UHD graphics, the project's floor for older cores. An NTSC frame is 16.68 ms, PAL 20.0.

**Why an interpreter alone cannot run games at full speed.** An NTSC frame holds 294,912,000 / 59.94 ≈ 4.92 million EE
cycles and 615,000 IOP cycles [A, arithmetic]. Diamond's interpreter costs about 10 ns a step with its timing model on
the desktop (`Diamond_Speed.md` §7.2) and MarsRT's, at 93.75 MHz, keeps up only with its levers (`Mars_Native.md`
§5.8.8). Priced [A]:

| Unit | Work a frame (NTSC) | Interpreted, desktop | With its levers, desktop |
|---|---|---|---|
| EE (R5900, timing at instruction granularity) | 1.6–3.3 million instructions, at an assumed 1.5–3 cycles each including stalls | 6–33 ms at 4–10 ns | 0.8–5 ms recompiled at 0.5–1.5 ns, less where idle skipping removes waits |
| VU1 micro mode | 1–3 million instruction pairs at 20–60 % occupancy | 8–60 ms at 8–20 ns a pair with software floats on four lanes | 1.5–12 ms recompiled at 1.5–4 ns, the floats' exactness the main cost (§5.3) |
| VU0 (macro and micro) | 0–1 million | 0–15 ms | 0–3 ms |
| IOP (R3000A) | 615,000 cycles, mostly waiting | 1–2.5 ms | 0.3–1 ms with idle skipping and Diamond's decoded cache |
| GS, software | 1–6 million pixels with overdraw and multipass effects | 3–30 ms at 3–5 ns, one thread | 0.5–5 ms of waiting with workers, or 1–3 ms of GPU time on the device path |
| VIF unpacks, GIF, DMAC | 1–4 MB moved | 1–4 ms | 0.5–2 ms |
| SPU2 | 38,400 voice-samples and two reverbs | 0.4–0.9 ms | the same |
| IPU (FMV only) | about 1,100 macroblocks | 1–3 ms | the same |
| **Total** | | **20–150 ms: 1.2 to 9 times too slow** | **4–25 ms** |

**The levers, in order** (each bit-identical to the plain path, `effect: exact`):

1. **Idle skipping** on both processors and on VU waits, as Diamond's lever 3 (`Diamond_Speed.md` §4), which there
   bought the most.
2. **Decoded caches** for the IOP (`blackmoon-r3000`'s `fetch_decoded` already exists) and the EE.
3. **The units on threads**: the GS's software rasteriser on workers drawing in order (MarsRT's RDP threads), VU1 and the
   IOP on their own threads, synchronised at the observation points of §5.1, so that each waits only where another
   could see it. Exact by construction if every observation waits for its producer up to the same cycle; the test is
   every golden equal threaded and unthreaded (P11).
4. **The recompilers**, on Cranelift through the shared JIT crate (§9.6): the EE first, then VU1's micro mode (whole
   microprograms compiled, which suits them: they are short, run often and are entered at known addresses), then VU0.
5. **The GS on the device**, the exact compute rasteriser of §5.4.

**What MarsRT's recompiler offers, and what it does not** [R `Mars_Native.md` §5.8; A]. Its design is "blocks between
the interpreter's checks": machine code for simple arithmetic, branches and aligned loads, **everything else a call to
the interpreter's handler**; it bought 3–4 % on two games and 16 % on a third (from the decode kept, not the code), and
holding registers in host registers bought nothing because every handler call writes them back (its §5.8.8). That
design is right for MarsRT, whose frame is bound by its signal and display processors, and **insufficient for the EE**,
where the processor is the bound: an EE recompiler must inline nearly every instruction, the MMI's 128-bit operations as
Cranelift vector operations, loads and stores through a fast path for RDRAM and the scratchpad, and keep registers in
host registers across a block, which only pays when calls are rare. **What carries over unchanged**: the claim (a block
leaves the machine as the interpreter would have at every observable point), the compiler's thread and queue, code
published by one atomic pointer, W^X batches, code never freed a block at a time, the block cache by physical address
validated against memory, the verifier that steps the interpreter beside the blocks, and the six-oracle method of its
§5.8.5. Those are the parts to share (§9.6); the emitter is each processor's own.

*Amended 2026-10-24.* The five levers above stay exact and are taken first, since each buys speed at no cost in
accuracy. They are no longer the whole of the default path: where they leave the handheld short of full speed, the
compromises of §5.13 are measured next, cheapest in accuracy first, and the default takes those the tester accepts.
**The recompilers' role changes accordingly**: still graded bit for bit against the interpreter with every compromise
off (the six oracles), they are also where most compromises live (C1, C3, C4), each a switch in the emitter whose
effect is measured against the exact output.

**Targets** [C, §11 Q11; *superseded 2026-10-24*, below]:

- **The desktop**: every bench game at full speed with the exact levers on, leaving a fifth of the frame: a mean of
  13.4 ms of wall time a frame for the emulation's threads together, and no frame over 16.7 ms at the 95th percentile.
- **The handheld**: full speed on battery for the bench games the desktop runs within 9.5 ms (13.4 / 1.4); the others
  measured and named. Some VU1-bound titles are expected to miss at first (P12).

**Targets, decided 2026-10-24 by the tester** (Q11 changed):

- **The handheld is the full-speed target**: every target game and every bench game at full speed on the Legion Go S,
  on battery, in the default build: a mean of 13.4 ms of wall time a frame for the emulation's threads together,
  leaving a fifth of the NTSC frame, and no frame over 16.7 ms at the 95th percentile (the desktop's former target,
  moved). The exact levers come first and the compromises of §5.13 after them; a game that still misses is named with
  its measured shortfall and the compromise that would close it, for the tester to accept or refuse.
- **The desktop** follows from it: whatever runs at full speed on the handheld runs on the desktop with about 1.3–1.4
  times the margin [A, `Nephrite_Plan.md` §5.5's ratio, to be measured at S32]. The desktop additionally runs the
  reference path (no compromises) as fast as it can, for grading; that speed is reported, not targeted.
- **The laptop**: **not a full-speed target**. Nephrite's rule (full speed at a 33 % CPU quota) cannot hold for a
  machine nine times the PlayStation's clock, and claiming otherwise would be dishonest; the laptop is measured and
  reported.
- **How measured**: Diamond's bench method (`Diamond_Speed.md` §2): `examples/bench.rs` timing each frame's `advance`,
  digests outside the timed span, builds interleaved on pinned cores, the aligned link as the instrument, fastest and
  median of rounds; threaded units measured as wall time with every thread's share sampled by `ipsample.py`; the
  handheld under `systemd-run --user` with no frontend running, on charger and on battery.

**The speed settings that are not exact**, offered only after all of the above (`effect: accuracy`, each with its
cost): the EE's timing reduced to a fixed cost an instruction; the VUs' floats on the host's arithmetic where it is not
proven equal; the CDVD's delays shortened. And, apart from accuracy, the enhancements of §6. *Amended 2026-10-24:*
these three are now candidates for the default path, C1, C3 and C9 of §5.13; one chosen for the default is still a
setting, `effect: accuracy`, whose other value is the reference's behaviour.

### 5.13 The compromises: candidates, each with the measurement that decides it

*Added 2026-10-24, by the speed decision. None is decided.* Each candidate is a departure of the default path from the
reference. It is taken only when a measurement shows the speed it buys on the handheld and the accuracy it costs, the
tester accepts the trade, and it is then named in machine info's words and in this list's losses. The reference
behaviour stays one setting away for every candidate taken. The order is the one in which each is expected to be
weighed, cheapest in accuracy first [A].

| # | Candidate: the default path would | What it gives up | What decides it, and when |
|---|---|---|---|
| **C7** | Synchronise VU1, the IOP and the GS with the EE at a bounded slack (a quantum of cycles or a scanline) instead of at every observation point (§5.1) | The exact cycle at which one unit sees another's writes, inside the quantum | The goldens and ps2autotests that change at each quantum, against the threads' wait time saved; S42 |
| **C11** | Skip waits recognised by pattern (a VSync or interrupt wait loop) without the exact cycle accounting of lever 1 | Timer and counter values read inside a skipped wait | The bench's anchors that differ, against the frame time saved; S33 |
| **C8** | Time the IOP by a fixed cost an instruction instead of Diamond's every-bus-access model | IOP-side timing that a module measures | `kernel/iop/*` and `cpu/iop/*`, the bench's anchors, against the IOP's share of the frame; S32 |
| **C1** | Charge the EE a fixed cost an instruction or a block instead of §5.2's documented issue, latency and cache model | Timing-sensitive games; the performance-counter test | `ee_cop0/performance`, the bench's anchors in picture and RAM, against the EE's share of the frame with and without the model; S32, S36 |
| **C2** | Model the caches' coherency effects (stale data after a missing write-back) but not their contents' timing, or neither | Games that read stale lines as a console does, or that time cache misses | The project's cache programs and the bench, against the cost of the tags on every access; S7, S37 |
| **C3** | Compute the FPU's and the VUs' floats on the host's arithmetic with clamping on the input classes not proven equal (§5.3) | The last bit and the edge classes (the PlayStation 2's own overflow and denormal rules) on those classes | `ee_fpu/*`, `vu/*`, each class's differing results and the bench's anchors, against the VU1 recompiler's speed with and without software floats; S39–S41 |
| **C4** | Drop the cycle-exact visibility of the VUs' MAC, status and clipping flags and of Q and P where the analysis of a microprogram cannot prove a read harmless | Microprograms that read a flag or Q inside its delay | `vu/lower/*`, `vu/upper/*`, `vu/games/triace` and the bench's pictures, against VU1's frame time; S40 |
| **C5** | Draw on the device with the host's rasterisation and blending (a graphics pipeline) instead of a compute rasteriser held bit-exact (§5.4) | Pixel coverage at edges, blending's rounding, the GS's exact tie rules | eDRAM and picture differences per bench game against the software reference (pixels differing, largest error), against the GS's frame time on the handheld's GPU; S44–S46 |
| **C6** | Keep render targets on the device and bring the eDRAM mirror up to date only at local-to-host transfers and detected reinterpretations | Readbacks the detection misses | The bench's readback effects (§6.2's list) against the synchronisation's measured cost; S44 |
| **C9** | Shorten the CDVD's seek and read times | Loading screens' length and any game that times its reads | The bench's load times and anchors, against the time a player waits (not frame time, which it does not change); S23 |

**Not candidates** [A]: the SPU2 and the IPU, which §5.12 prices at under 1 ms and about 1–3 ms of the frame and
which a player hears and sees directly; and the instruction results of every processor, which no compromise may
change but C3's named float classes.

*Decided 2026-10-26 by the tester: for the PlayStation 2 and later consoles, cycle accuracy where it is realistic; where
it is not, byte accuracy is the floor.* Byte accuracy means that every value a game can read back, and every pixel and
sample the player receives, equals the console's; timing is modelled only as finely as that needs. The candidates above
are therefore of two kinds:

- **Timing-only** (C1, C7, C8, C11, and C2's timing part, the caches' miss times without their coherency): each changes
  when something happens and none of the bytes a game reads or the player receives where the game does not depend on
  that timing. Each is taken where `Sapphire_Native.md` §43's measurements show its gain, named and switchable, without
  waiting for a separate acceptance.
- **Byte-changing** (C3, C4, C5, C6, C2's coherency part, and any other candidate that changes a value a game reads or a
  pixel or sample the player receives): each needs a proof that it keeps the bytes on the classes it is used for, or the
  tester's acceptance of the bytes it changes, measured as the table above says. C9 changes no byte of a frame but a
  loading screen's length, and stays with the tester as before.

Three conditions hold for every candidate taken:

1. **Results stay deterministic**: the same input gives the same bytes, run after run and on every host, and units on
   threads synchronise at fixed points of the emulated time, never at points chosen by the host's scheduling.
2. **A game whose results depend on timing gets finer timing for the part involved**: where a timing-only candidate
   changes a bench game's bytes, that candidate is narrowed or turned off for the unit or game involved, and the case
   recorded, rather than the bytes accepted.
3. **The exact path stays the reference**: every candidate's other value is the reference's behaviour, the recompilers
   are graded against the interpreter with every candidate off, and the goldens are taken on that path.

---

## 6. Enhancements

### 6.1 The rule

The clean room of §1.4 applies to the enhancements as to the rest: no emulator's enhancement code is read, and what an
enhancement does is learned from published descriptions and from the hardware's own behaviour. Diamond's architecture
(`Diamond_Plan.md` §6.2) is taken whole [C]: **the accurate path is the default and stays bit-exact with every
enhancement off**, pinned by the accurate goldens recorded before the layer exists (E1); **a separable layer**, never
edits through the accurate code; each enhancement **its own setting** with `effect: enhancement` and its `hardware`
value the default; optional, skippable **state blocks**; and the two that change the emulation itself (the clocks and
the CDVD's speed) parameters of the accurate model, tagged in the state.

*Amended 2026-10-24.* "The accurate path is the default" no longer holds for this core: the **reference path** stays
bit-exact with every enhancement off and is what E1's goldens pin, and the **default path** is the reference with the
compromises of §5.13 the tester has accepted. The enhancements remain a separable layer over either; a compromise is
not an enhancement (it is `effect: accuracy`, not `effect: enhancement`) and is never mixed into the layer.

### 6.2 The enhancements

| Enhancement | Game-agnostic? | Needs | What it can break |
|---|---|---|---|
| **Higher internal resolution** | Largely [A], through a renderer of its own at N× beside the exact eDRAM | The enhanced renderer consuming the GIF's packets, with its own N× copies of render targets | The GS's games reinterpret eDRAM constantly: a render target read back as a texture at another format, post-processing that samples the frame at half-pixel offsets, depth read as colour, and effects built from many one-pixel sprites. Each is a known breakage at N×, handled by the renderer's own rules and listed per game; the exact eDRAM is never affected, so what a game computes cannot change |
| **De-interlacing** | Yes | The PCRTC's two fields | Field-rendered games combed or blurred by the chosen method; none in the accurate path |
| **Texture filtering and mipmap overrides** | Partly | The enhanced renderer | Packed textures bleeding at their edges, CLUT-based effects |
| **True colour without dithering** | Yes | The enhanced renderer | Banding where art relied on the dither |
| **Widescreen** | **Not in general** [A] | Per game, by patches outside this family | — |
| **EE and VU clock ratio** | Yes, as a setting | The clocks of §5.1 | Games that time by loops; slowdowns smoothed only where the EE or VU was the bound |
| **Faster CDVD** | Mostly | The drive's timing table, real speed kept while streaming | Games that race their loading or time the drive |

---

## 7. Stages

**Each stage is one supervised step of about three hours** with a check-in after it [C]. A stage's oracle must be met, or its misses recorded as disputes or named losses, before the next that depends on
it begins. The effort is predicted (P3). Phases B to F are partly independent once A is done; the open BIOS (O) and
the enhancements (E) are separate tracks.

**Phase A — the frame**

| Stage | What it covers | Its gate |
|---|---|---|
| **0** | This plan (2026-10-23) | The tester's go for S1 |
| **S1** | The crate `sapphire` on the core ABI v1 as a stub with the `InDevelopment` mark: an ELF recognised and refused with words; the system pack `ps2` (§4.1) with the recogniser's `BOOT2` rule and Diamond's `BOOT` rule beside it; the three firmware entries (§5.9); the `SPH2` header | The kit's C1–C15 on a synthetic ELF and a synthetic ISO; a synthetic PS1 disc routed to Diamond, a PS2 CD and DVD to Sapphire, by test *Built 2026-10-24, `Sapphire_Native.md`: met; the CRT row recorded as owed (its §6).* |
| **S2** | References: the documents of §2.1 fetched and pinned with `DOCS.txt` and `SHA256SUMS`; ps2tek's BIOS chapter screened by a non-replacement writer and the exclusions written; ps2autotests at `97469ff` with `PROVENANCE.txt`; the MD5 lists of §5.9 cited from a public database; the IOP's cache facts and the SPU2's voice count read | Each file pinned; the exclusions listed by section and line; every MD5 with its citation *Built 2026-10-24, `Sapphire_Native.md` §12, `Sapphire_References.md`: met; the Japanese entry renamed to `0230j`.* |
| **S3** | Referees: PCSX2 standalone headless (a private compositor, the software renderer, PINE for RAM), Play! standalone and its libretro form, the probe on LRPS2 and PCEE2 to measure the hardware-render refusal; their costs and the RAM each exposes | One project program's RAM record read from each referee that runs it; the referees' table of `Sapphire_Referees.md` written *Built 2026-10-24, `Sapphire_Native.md` §13, `Sapphire_Referees.md`: met; `record.s`'s twelve words equal on PCSX2, LRPS2, PCEE2 and Play!, `play_libretro` refused for want of an OpenGL context.* |
| **S4** | The runners (§3.3): `SyntheticPs2Elf`, the synthetic disc builder for ISO 9660 CDs and single- and dual-layer DVDs, the expected-text grader, the RAM-record reader; **one ps2autotests program run on PCSX2 with the player's BIOS** to learn its output channel by observation | A positive and a negative control for each runner; the channel ps2autotests prints on recorded (P2) *Built 2026-10-24, `Sapphire_Native.md` §14: met; `cpu/ee/alu`'s whole text reached PCSX2's IOP console and not its EE console, which carries KPUTCHAR (P2 held).* |

**Phase B — the EE**

| Stage | What it covers | Its gate |
|---|---|---|
| **S5** | `emusen-mips`: the R5900's decode and disassembly as a table of its own (§9.1), MMI, COP0, COP1 and COP2's macro forms, the VU micro tables; the assembler for both | Every encoding of the TX79 manual's opcode tables decoded, disassembled and reassembled to the same word; the R3000 and VR4300 goldens unchanged *Built 2026-10-24, `Sapphire_Native.md` §15: met for the 723 cells where the EE agrees with the TX79, the other 83 decoded as ps2tek reads them under disputes D-1 to D-3; ps2tek's two VU tables read back in both modes; the VU micro-only lower instructions in no allowed document (§15.6 Q1); the other processors' text unchanged.* |
| **S6** | The R5900 interpreter, integer: 128-bit registers, the MIPS III subset, the R5900's multiply forms and pipelines, branch-likely, LQ/SQ, exceptions; the timing model's issue and latency table | The project's programs over every integer encoding against the manual's definitions; the timing model's table pinned *Built 2026-10-24, `Sapphire_Native.md` §16: met; 128 mnemonics through `r5900.rs`, seventeen programs equal to the EE Core manual's record, the results equal on PCSX2 and Play! but where D-6 to D-11 record otherwise; neither referee raises address-error, overflow, reserved or unusable exceptions, so those rest on the manual until S10; the EE Core User's Manual's §2.12 and Tables 1-1 to 1-3 pinned.* |
| **S7** | COP0, the TLB with the scratchpad mapping, the caches and CACHE, the memory map, Count/Compare and the performance counters | The project's programs on TLB refills, cache write-back and the counters *Built 2026-10-25, `Sapphire_Native.md` §17: met on the interpreter, seven programs and 400 graded words equal to the manuals' records, the breakpoints built beside the counters; neither referee maps a program's own TLB entries, models the caches, counts events or takes the TLB, counter, debug or bus error exceptions, so those rules rest on the manuals; D-12 to D-19.* |
| **S8** | MMI | The project's programs; `ee_simd/*` waits on S10 *Built 2026-10-25, `Sapphire_Native.md` §18: met; 91 instructions through `r5900.rs`, nine programs and 20,712 record words equal to the EE Core Instruction Set Manual's chapter 3 on the interpreter and on PCSX2; Play! does not execute eight of them; the six `ee_simd` programs equal their console texts on PCSX2, which decided D-21 and D-22; D-20 to D-23, D-9 closed.* |
| **S9** | COP1 in software floats (§5.3) | The project's programs on every input class *Built 2026-10-25, `Sapphire_Native.md` §19: met; 34 instructions through `r5900.rs` on `sapphire::float`, the format's one owner, six programs and 48,648 record words equal to chapter 5 on the interpreter; the last bit decided where the console texts of `ee_fpu/*` decide it (no sticky bit in the adder, a Booth multiplier on ft that is not commutative, DIV.S and SQRT.S to the nearest), 900 of their 901 result lines reproduced; neither referee models exponent 255 or the multiplier, PCSX2 reproducing 721 of the texts' 1,068 lines; D-24 to D-33.* |
| **S10** | **The EE's ps2autotests**: the ELF loader and whatever the tests need to run — on the player's BIOS by fast boot, so taken after S24 if §11 Q6 is answered as recommended, or earlier on the open EE kernel's first phase if it is not *(Q6 decided as recommended; decided 2026-10-25 by the tester: S10 is taken once S24's BIOS boots, after S11)*; **the IOP's console kept in a `tty` space**, caught at the IOP's module interface with the output export resolved at run time through the export tables, never at an address in the image *(decided 2026-10-24)* | `cpu/ee/*`, `ee_cop0/performance`, `ee_fpu/*`, `ee_simd/*`, `memory/ee/default` equal their expected texts, each miss a dispute *Built 2026-10-25, `Sapphire_Native.md` §36: met on both 0230 images, 22 of the 23 programs equal and the 23rd but the one line D-28 names; the program entered by fast boot through the kernel's ExecPS2 once the IOP has finished booting and the EE runs in user mode, no BIOS code read; the IOP's console caught at IOMAN's `write` found through the export tables, kept in the `tty` space and answering as an attached TTY for a fast-booted program (D-364); the IOP's DMAC's flag rule and start time corrected (D-360, D-361), which had deadlocked the SIF's RPC; D-360 to D-364.* |

**Phase C — the EE's peripherals**

| Stage | What it covers | Its gate |
|---|---|---|
| **S11** | DMAC (ten channels, chain and interleave modes, tags, the scratchpad channels, stall control), INTC, EE timers *(decided 2026-10-25 by the tester: built before S10, directly after S9)* | `dma/dmac/*`, `dma/spr/*`, `kernel/ee/interrupt/*`, which need the BIOS's kernel: on the referees against their console texts at S11, on Sapphire with S10; the project's bare programs on each mechanism meanwhile *Built 2026-10-25, `Sapphire_Native.md` §20: met on the interpreter, eight programs and 1,230 graded words equal to the EE User's Manual's chapters 3 to 5, the interleave arithmetic also to the console text; the seven console texts reproduced by PCSX2; neither referee models stall control or the priority control, and PCSX2 takes an interrupt only as its request rises; the channels whose peripheral comes later end at `dmac::Peripherals`, the H-blank and V-blank a stand-in until S20; D-34 to D-42, D-15 closed.* |
| **S12** | VU0 macro mode, the VU float | `cpu/vu0_macro/*` *Built 2026-10-25, `Sapphire_Native.md` §21: met on the interpreter, seven programs and 26,102 graded words equal to the VU User's Manual's pages; the VU shares `sapphire::float`'s arithmetic and has its own flags; micro mode's start an interface S13 fills; `cpu/vu0_macro/integer`'s console text reproduced by PCSX2 and `transfer`'s but eight lines, whose values the project's model gives; neither referee models the VU's arithmetic (IEEE 754 towards zero), its U and O flags or Q's delay; D-43 to D-54.* |
| **S13** | VU micro mode, both units: the pipelines, flags, Q and P, EFU, branches, XGKICK, E/D/T | `vu/lower/*`, `vu/upper/clip`, `vu/games/triace` *Built 2026-10-25, `Sapphire_Native.md` §23: met on the interpreter, seven programs (1,948 record words) over every micro instruction but XGKICK, XTOP and XITOP, and 24 transcriptions of the manual's pipeline figures; the console texts need the BIOS's kernel (S10): on PCSX2 `branch`, `fdivdelay`, `random` and `clip` equal, two of them reproduced on the interpreter; the EFU from §7.2's nano code, 193 of `efu`'s 208 console values exact and the rest within two in the last place; PCSX2 a witness for the pipeline's visibility; D-43 stands, `triace` deciding it at S10; D-80 to D-85.* |
| **S14** | VIF0 and VIF1 | `dma/vif/basic`, `intstall`, `stcycl`, `stmod`, `unpack` *Built 2026-10-25, `Sapphire_Native.md` §28: met on the interpreter, seven programs and 4,190 graded record words equal to the EE User's Manual's chapter 6; the texts need the BIOS's kernel (S10): on PCSX2 `basic` and `stmod` equal, `unpack` but one line, `stcycl` but 26 of 508, `intstall` differing, and on the VIF directly `stcycl`, `stmod`, `unpack` and `intstall` reproduced in the shape their lines show, deciding UNPACK's indeterminate fields, MODE 3, the address wrap and NUM's width; PCSX2 a witness for the data path, not for the stalls or errors; VIF1's reverse direction is the GS's local-to-host data; D-180 to D-194.* |
| **S15** | GIF, its three paths and their arbitration | `dma/vif/mskpath3`; the project's programs on path interleaving against referees *Built 2026-10-25, `Sapphire_Native.md` §33: met on the interpreter, six programs and 6,728 graded record words equal to the EE User's Manual's chapter 7 over S17's model of the GS, and S16's, S17's and S18's programs run whole through PATH3 and VIF1 (473,156 words); PCSX2 equal on the formats but descriptors 0Ch and 0Dh, the masks with the FIFO and the held DMA, TTE, PATH1's wrap and DIRECT overtaking IMAGE in the intermittent mode, departing on a packet holding the GIF, where Play! holds as the manual does; `dma/vif/mskpath3`'s text reproduced by PCSX2 and on the GIF, Sapphire running it with S10; S14's second DIRECT a PCSX2 defect after fast boot without a GIF reset; D-280 to D-291.* |

**Phase D — the GS, the software reference**

| Stage | What it covers | Its gate |
|---|---|---|
| **S16** | Registers, privileged registers, CSR and IMR, SIGNAL, FINISH, LABEL; the eDRAM's page and block layout of every format; the three transfer directions | `gs/label`, `gs/signal`; transfers round-trip through every format as the documents lay them out; referees' pictures of a transfer program *Built 2026-10-25, `Sapphire_Native.md` §24: met on Sapphire, five programs and 51,268 graded record words equal to the GS User's Manual's chapters 4, 6, 7 and 8, the GS driven through the interface the GIF (S15) will use; PCSX2 equal on every format's layout at a page's start and on `gs/label`'s and `gs/signal`'s console texts, which Sapphire runs with S10; Play!'s local-to-host transfer reads the colour formats back, so the referees' pictures come as RAM as S3 decided; D-100 to D-107.* |
| **S17** | Points, lines, triangles, strips, fans, sprites; flat and Gouraud; scissor; depth formats and tests | The project's primitive programs, each picture equal to the referees' (or a dispute) *Built 2026-10-25, `Sapphire_Native.md` §27: met on Sapphire, eleven programs and 225,280 graded record words equal to a model of the GS User's Manual's chapter 3, which also draws 2,400 generated primitives as the source does; PCSX2 equal on every pixel's coverage but four line ends on a diamond's edge, on every format, mask and depth decision, on XYOFFSET at the drawing kick and Z's clamp, departing by a step in Gouraud colour and in Z's low bits, where the documents give no precision; Play! draws through a GPU renderer its local memory never sees, so no second lineage until the S3 reserve is built; D-160 to D-165.* |
| **S18** | Texturing: formats, CLUTs (both storage modes), TEX0/1/2, LOD and mipmaps, filtering, wrap and clamp, fog | The same, per feature *Built 2026-10-25, `Sapphire_Native.md` §31: met on Sapphire, nine programs and 196,608 graded record words equal to a model of the GS User's Manual's §3.4 and §3.5 beneath S17's, which also draws 720 generated textured primitives as the source does; texture coordinates and the LOD exact, taken to sixteenths; PCSX2 equal on every texture format, every CLUT case, TEXA, the texture functions, the wrap modes, MTBA and level choice, and on fog, measured against §3.5's formula; departing in bilinear rounding, the LOD's precision and Q with UV, MMAG at a LOD of 1/16, a defect of its blend between levels, and texel boundaries within single precision; Play! no witness (§27.7); D-240 to D-250.* |
| **S19** | Alpha test, destination alpha, blending, PABE, FBA, colour clamp, dithering, masks, local-to-local overlap | The same *Built 2026-10-25, `Sapphire_Native.md` §37: met on Sapphire, eleven programs and 98,304 graded record words equal to S17's model with the manual's §3.6 to §3.9 as tables, which also draws 2,400 generated primitives under generated pixel-operation registers as the source does; PCSX2 equal on every ATST and AFAIL, all 81 blend selections with FIX under both COLCLAMPs, PABE, FBA and the masks, and showing dithering into the 16-bit formats alone; departing on DATE in PSMCT24, COLCLAMP 0 after an unblended dither, antialiasing's coverage and overlapping copies, the last decided in TRXPOS's order (D-107 closed); the BIOS intro's white bands and square light trails both alpha blending missing, by ablation, isolated by two programs equal on PCSX2, and the intro's frame within 261 pixels of PCSX2's picture; D-400 to D-406.* |
| **S20** | The PCRTC: both circuits, merge, interlace in field and frame modes, NTSC, PAL and progressive, the picture handed to the frontend, the video timing | The referees' pictures at anchors of the project's display programs; the timing against D-0's ruling *Built 2026-10-25, `Sapphire_Native.md` §29: met; seven display programs equal to chapter 5's model on the board, their memory equal on both referees and PCEE2's own pictures equal pixel for pixel where it shows a circuit at its own size; the timing the GS manual's table read at 13.5 MHz (59.94 Hz), where both referees run 60 Hz; SMODE1 and the synchronisation registers have no fields in any permitted source, so the mode is the board's; the feedback write a named loss; D-200 to D-206, D-42 closed.* |

**Phase E — the IOP side, and the player's BIOS**

| Stage | What it covers | Its gate |
|---|---|---|
| **S21** | The IOP on `blackmoon-r3000`: its memory map, RAM, caches and scratchpad in Sapphire's IOP bus, its INTC, DMA and six timers, its PRId | The r3000 suite still passes; the project's bare-metal IOP programs *Built 2026-10-25, `Sapphire_Native.md` §22: met; the crate unchanged and its suite passing, seven programs and 174 graded words equal to ps2tek's and psx-spx's records; an IOP program placed on PCSX2 through an EE program and the IOP's exception vector, Play! unable; PCSX2 differing on 41 of 153 words where it models no IOP cache, wait state, bus error or register width; PRId 1Fh; D-60 to D-68; `cpu/iop/*` and `kernel/iop/*` wait for S24's BIOS.* |
| **S22** | The SIF: registers and its three DMA channels | The project's programs passing messages both ways, EE and IOP each checking the other's record *Built 2026-10-25, `Sapphire_Native.md` §26: met; the SIF in `sif`, a FIFO of 128 bytes for each of SIF0, SIF1 and SIF2 between both DMACs' channels, the stand-ins replaced; five paired programs and 194 graded words equal on Sapphire to ps2tek's and the EE manuals' records; on PCSX2, the one lineage that runs them, 57 differ, where it holds no FIFO, ends an IOP chain at an IRQ tag, sends SIF0's tag words without bit 8, takes the EE's SMCOM write and moves nothing on SIF2; the EE's CTRL bit 18 raising the IOP's interrupt 1 its single witness's reading; D-140 to D-145, D-68 amended; the SIF RPC protocol S24's, above `sif`.* |
| **S23** | The CDVD model (§5.6) and `emusen-disc`'s DVD layout (§9.4): reads, seeks, layers, CD modes and CD-DA, the clock, the EEPROM, the tray | Synthetic CDs and single- and dual-layer DVDs read sector for sector, the layer break placed; DVD images opened mapped on the desktop and the handheld, load time recorded (P13) *Built 2026-10-25, `Sapphire_Native.md` §25: met but for P13's handheld half (the handheld did not answer); the drive's answers where ps2tek is silent measured on PCSX2's lineage, the only one that reaches it from the EE, disputes D-120 to D-128.* |
| **S24** | **The player's BIOS to its menu**: what is missing for it, SIO2's pad, the EEPROM synthesised; the writers of this stage are thereafter not replacement writers | OSDSYS's picture at an anchor equal to PCSX2's on the same BIOS within a stated count of pixels, for the Americas and Europe versions of §11 Q9; then **all 78 ps2autotests on the player's BIOS** equal their expected texts, each miss a dispute *First half built 2026-10-25, `Sapphire_Native.md` §35: met; the machine on the board with the player's BIOS for both processors, the IOP's reset line on the SIF's CTRL bit 19, the drive on the IOP's bus, the EEPROM the console medium `ps2.nvm` (blank all zeros, no layout in any permitted source), the PCRTC's picture and the SPU2's sound through `v1`; both 0230 images reach OSDSYS's language screen at 12.0 s, equal to PCSX2's woven picture but for the clock OSDSYS draws on Sapphire (269 and 274 pixels elsewhere differing by more than 32), and the pad steps the initial setup on; fourteen behaviours of the board built or corrected, D-340 to D-351. The second half is S10's.* *Second half at S10, 2026-10-25, `Sapphire_Native.md` §36: 52 of the 55 EE programs equal, the misses D-28's and S13's named losses and `intstall`'s FQC (D-363, open); `vu/games/triace` settles D-43 as argued; BC1T's delay slot (D-362), JR and JALR (D-80) and WL 0 (D-181) taken from the console texts; the 23 IOP programs not run, their loading the tester's decision (§36.14 Q1); S24's field order settled by `s10-fieldmark` on PCSX2.* *The 23 IOP programs, 2026-10-26 (`Sapphire_Native.md` §40, §36.14 Q1 decided as recommended): loaded by the project's EE loader through LOADFILE's SIF RPC, its layout learned on PCSX2 (D-480), from a synthetic CD put in through G10's `MEDIA`; **all 23 equal their console texts on both 0230 images**, so all 78 programs run and 75 are equal; D-481 (the drive's interrupt) and D-482 (the IOP's region past 8 MiB) found by them and fixed, D-483 (a DVD's file system) open.* *D-483 closed 2026-10-26 (`Sapphire_Native.md` §40.14): the BIOS reads a DVD's path table at sector 257; the synthetic DVDs place theirs there, and the 23 equal their texts from DVDs as from CDs on both images.* |
| **S25** | SIO2: the DualShock 2 with pressure and rumble, the card (pages, erase, timing, the medium of §4.4, its crate §9.5), the multitap; **the start without a disc** (G10) and **the PlayStation 2's tab of the Memory Cards window** (G11) built on them, behind the `InDevelopment` mark | The pad drives OSDSYS; a card formatted by the player's BIOS on Sapphire is read by the crate and by PCSX2, and the reverse, or the refusal of §5.8 recorded as P8 says; G10's and G11's tests (§8) *Amended 2026-10-24: G10 and G11 added here.* *Part A built 2026-10-25, `Sapphire_Native.md` §30: Q7's research step (Q7(b) holds); SIO2 in `sio2` with channels 11 and 12, the DualShock 2, the card with its flash's times, the multitap's frame; `emusen-ps2-card`; the medium `ps2.card` declared by the core; five programs, 1,007 graded words, equal on Sapphire to the documents' records; on PCSX2, the one lineage with an IOP window, the pad's identification, modes and rumble map equal but for eight words, the card's commands equal but for the reply's first bytes, its terminator commands and its erase's alignment (D-220 to D-230). Part B, after S24 and on the tester's go: the OSDSYS checks, a BIOS-formatted card both ways, G10 and G11.* *Part B built
2026-10-25, `Sapphire_Native.md` §30.15-§30.24: G10, `starts_empty` on `ps2`, the console created with its tray closed
and empty, `MEDIA` with one DVD drive, a synthetic disc put in reaching the recogniser and the drive, the start refused
by the host in the firmware entry's words with no BIOS, the frontends' entries from core info, all behind the
development mark; G11, `ps2.card` read by `emusen-ps2-card` behind the card manager's format surface, its unit the
kilobyte the browser shows, the Memory Cards tab in the fit audit at both sizes, held and not; on PCSX2 the player's
BIOS formatted a card that the crate's copy, delete and import edited and the BIOS read back, and a delete made by the
BIOS was read by the crate (D-380 to D-382); on Sapphire OSDSYS with an empty tray stepped through its setup to the main
menu at the pad's presses, and its browser, entered with a card in slot 1, never drew (P8 as observed; D-383 open).* *D-383 resolved 2026-10-26 (`Sapphire_Native.md` §30.25): a command no device acknowledges ends at its peripheral byte; the browser stalls with slot 1 empty as with a card.* *The SIO2out step, 2026-10-26 (`Sapphire_Native.md` §30.31): a queue's end requests channel 12 until SIO2's FIFOs are reset, so the BIOS's slices end (D-540); the browser still stalls alike, card or none.* *The SIF link step, 2026-10-26 (`Sapphire_Native.md` §30.38): every SIF RPC call answered; the browser's card and multitap polling never begins on Sapphire (D-620).* |
| **S26** | The SPU2 on the shared voice crate (§9.3) | The project's voice programs against the referees' sound; Diamond's SPU goldens unchanged by the crate's extraction *Built 2026-10-25, `Sapphire_Native.md` §32: met; the voice moved into `blackmoon-spu` with Diamond's goldens and tests unchanged first; two cores on the IOP's bus and channels 4 and 7, 48,000 Hz exactly; eight programs graded on Sapphire, their sound read from the output areas PCSX2 also writes, the mix 28 of 30 copies equal on PCSX2, AutoDMA's stream, the registers and the IRQ scenes equal but where PCSX2's voice reads psx-spx otherwise than Diamond's referees, which keep the shared voice; key on at 1A0h, not ps2tek's 190h; D-260 to D-270.* |
| **S27** | The IPU | The project's MPEG-2 streams decoded equal to the standard's arithmetic, each difference a dispute *Built 2026-10-25, `Sapphire_Native.md` §34: met; the ten commands, the registers, both FIFOs and channels 3 and 4, MPEG-2 and MPEG-1 decoding from H.262 with Annex A's reference IDCT decided exactly near every half, §8.6's colour conversion; twelve ffmpeg streams and two written symbol by symbol decoded by Sapphire's IPU equal to a model of the standard written apart, which ffmpeg's own decoding matches within one step; 32 replay programs and a register program, 163,964 graded words, equal on the board; PCSX2 a witness for the register behaviour, the stream's handling, non-intra decoding and PACK, its IDCT one step off (P7), its intra range, alternate-scan weighting, IDEC scale and CSC departing by rules reproduced exactly; Play! a witness for VDEC and IPU_TOP; the decoding untimed, the referees disagreeing by a factor of four; D-300 to D-326.* |
| **S28** | Games on the player's BIOS to their title screens and into play (needs §11 Q8) | The bench discs at three anchors each against the referees in picture and RAM, or dispute-logged |

**Phase F — players**

| Stage | What it covers | Its gate |
|---|---|---|
| **S29** | The PlayStation 2 to players: the pack's serial, region, shelf and media, the settings, the firmware page's entries, the DualShock 2 drawing, the CRT profile (§4.6), `.m3u` and the disc change; the start without a disc and the Memory Cards tab (G10, G11) offered with the system | The kit's C1–C15 on the tester's discs in the corpus form; the fit audit with the PlayStation 2's tab, drawing and shelf at both sizes; `ConsoleScreenRuleTests`; G10 and G11 met on the player's build *Amended 2026-10-24.* |
| **S30** | States: `SPH2`'s layout listing, loading back, the rewind codec measured on a 41 MiB state | States round-trip at every anchor of the bench; rewind's cost recorded (P14) *Built 2026-10-25, `Sapphire_Native.md` §38: met; `SPH2` version 2 with every unit's state in §4.7's order, the picture first so that a load knows its length, 43,231,819 bytes for a 640 × 448 picture, the queues in fixed slots, the listing pinned against `tests/sph2/version-2.layout`; S24's refusal of loads on a running console removed; states saved at five frames of the boot to OSDSYS on both 0230 images and at four instants of 30 of the project's programs continuing equal in picture, sound and state; the kit's C1 to C15; P14 measured, a capture 6.5 ms and a restore 4.3 ms on the desktop, the default budget holding seconds; no dispute.* |
| **S31** | The debugger: four processors, 128-bit registers, the VUs' disassembly, breakpoints, stepping, coverage | Each claim a test; every table armed gives the plain run's digests *Built 2026-10-25, `Sapphire_Native.md` §41: met; `DEBUG`, `DEBUG_STACK`, `DEBUG_REGISTERS`, `DEBUG_DISASSEMBLE` and `DEBUG_READS` with the R5900, the IOP, VU0 and VU1, every register at its width and a 128-bit register as its two 64-bit halves, the ABI's value being 64 bits (§41.10 Q1); listings from `emusen-mips`'s tables, the VUs' upper and lower; the EE observed around its untouched step, the IOP and the VUs by probes that stop them only where the board brings them to time, so that every stop resumes exactly; every table armed, and every processor stopped at every step, equal to the plain run in picture, sound and state on fourteen of the project's programs, and on the Americas OSDSYS boot over 760 frames, digests only; attached and unarmed 16 per cent over the plain frame on the boot, the plain frame unchanged; no dispute.* |

**Phase G — speed** (§5.12)

| Stage | What it covers | Its gate |
|---|---|---|
| **S32** | The bench and the profile, before any lever | The per-unit costs of §5.12's table replaced by measurements on the desktop and the handheld |
| **S33** | Idle skipping on the EE, the IOP and the VU waits | Every golden equal; the bench before and after |
| **S34** | The decoded caches (EE and IOP) | The same |
| **S35** | `emusen-jit`, the shared JIT crate extracted from MarsRT (§9.6) | MarsRT's block tests, corpus and game runs identical at every tier, and its speed unchanged within its noise; the crate's own tests *Built 2026-10-25, `Sapphire_Native.md` §42: met; `EmuSen/Cores/Shared/emusen-jit` (`EmuSen_Jit.md`) holds the thread and its queue, the slots, the module with its W^X batches and budget, the counters and the verifier's shape, generic over a `Lowering` trait; MarsRT keeps its emitter, shapes and dispatcher (`marsrt -> emusen-jit`); the 21 block tests, the corpus's 1,722 lines and the games (Super Mario 64 and Ocarina of Time, 300 frames) identical at all three tiers; its speed unchanged within its noise (Super Mario 64 3.633 against 3.581 ms a frame, Ocarina of Time 4.726 against 4.573, medians of five interleaved rounds whose ranges overlap); the crate's ten tests on a toy processor, ThreadSanitizer finding the publication made Relaxed; no dispute.* |
| **S36–S39** | The EE recompiler: integer and branches (S36), loads and stores with the RDRAM and scratchpad fast paths (S37), MMI as vector code (S38), COP1 and COP2's macro forms (S39) | At each, Diamond's and MarsRT's six oracles: the interpreter stepped beside it and compared after every instruction, random programs, the mechanisms one at a time, ps2autotests, the bench games frame by frame, and a state loaded into an empty cache *S36 built 2026-10-26, `Sapphire_Native.md` §44: met; the EE's lowering on `emusen-jit` (`sapphire -> emusen-jit`, the crate unchanged) for 99 integer, multiplier, SA, trap and branch mnemonics, the issue model computed in the block, the board brought through each instruction's cycles at the block's exit with the interpreter's interrupt check, the instructions before an interrupt run again by the interpreter; the verifier after every instruction and every block, 240 random programs, fourteen mechanisms, ps2autotests 52 of 55 on both tiers with every text identical, the OSDSYS boot (760 frames), Jak and Daxter's program (600) and the project's loop equal frame by frame and from a state in an empty cache, the verifier holding over 30 frames of each of the first two; faster on every workload (medians of three rounds: 2.63 times on the loop, 1.49 on Jak and Daxter, 1.20 on the GS-bound boot); the recompiler the default tier, decided 2026-10-26 by the tester, the interpreter a switch away; C1 built off, measured (12.6 % less time a frame on the loop) and not taken; no dispute.* |
| **S40–S41** | The VU1 recompiler, micro mode, whole microprograms; then VU0 | The same, with every float class tested against the software path (§5.3) |
| **S42** | The units on threads (§5.12 lever 3) | Every golden equal threaded and unthreaded, at every anchor (P11) |
| **S43** | The GS's software rasteriser on worker threads | eDRAM equal after every frame; the bench |
| **S44–S46** | The GS on the device: the compute rasteriser's eDRAM mirror and transfers (S44), primitives and texturing (S45), the pixel pipeline and the PCRTC on the device with the picture handed through the slot (S46) | eDRAM equal to the software reference after every frame of every program and bench game; the validation layer with the synchronisation heuristic clean; RX 6800 only |
| **S47** | The targets (§5.12) | G6 |

*Amended 2026-10-24, the speed decision.* Phase G's gates that ask for "every golden equal" (S33, S34, S36–S43) hold
for the levers, which stay exact; each compromise of §5.13 is a step of its own inside the phase, gated not by equality
but by its measurement written into §5.13's row (accuracy lost, speed gained on the handheld) and the tester's
acceptance. S44–S46's gate becomes: eDRAM equal to the software reference after every frame, **or** each difference
measured and attributed to an accepted C5 or C6. S47's target is the handheld's (§5.12). S28's bench begins with the
tester's three target games, Jak and Daxter, Kingdom Hearts and Dark Cloud 2.

**Phase H — the gates**: **S48**, the goldens and the clone check (§8).

**The enhancements** (after S29, beside phase G): **E1**, the layer's frame and the accurate goldens recorded first;
**E2**, the enhanced renderer at 1×, equal to the exact picture; **E3**, internal resolution at 2× to 4×; **E4**,
de-interlacing, true colour and filtering; **E5**, the clock ratio and the faster CDVD. Each one step, each gated as
Diamond's E-stages were (`Diamond_Plan.md` §7).

**The open BIOS** (a separate track with its own writers, §5.9; begun only on the tester's go):

| Stage | What it covers | Its gate |
|---|---|---|
| **O1** | The image's frame: the build for both processors, the R5900 instruction scan, the reproducible hashes, the reset path | The image boots to a defined loop on Sapphire; CI rebuilds the same bytes |
| **O2–O4** | The EE kernel: exceptions and interrupts (O2), threads, semaphores, alarms and handlers (O3), SIF DMA, the CRT, `LoadExecPS2` and the ELF loader (O4) | ps2autotests' `kernel/ee/*` equal their expected texts on the open kernel |
| **O5–O9** | The IOP kernel: memory and the module loader with its linker (O5), interrupts and threads (O6), the synchronisation objects and pools (O7), I/O, the file server and SIF RPC, the TTY device writing to the project's own port (O8, decided 2026-10-24), the reboot into a disc's module image (O9) | ps2autotests' `kernel/iop/*` and `cpu/iop/*`; a game's own IOP modules loaded and linked |
| **O10–O11** | CDVD's file system and server (O10); SIO2, pads, the card's file system and server on §9.5's crate (O11) | Games boot with no firmware folder; their cards are read under the player's BIOS and the reverse |
| **O12** | The menu without a disc, and `firmwarecheck.py` against the tester's images with a control (`EmuSen_Firmware.md` §0) | G9; G10's start reaching the open BIOS's menu with no firmware folder *Amended 2026-10-24.* |

About 48 accurate stages, 5 enhancement stages and 12 open-BIOS stages: **65 steps, 200–260 hours** if each fits its
step (P3). The GS's rules, the VU pipelines, the CDVD and the IOP's module interface are the most likely to overrun.

---

## 8. The gates

Sapphire replaces nothing, so its gate is the one to **offer the system to players** after S29, by the tester's
decision. The core carries the `InDevelopment` mark of `EmuSen_CoreAPI.md` §27 from S1 until then. With the player's
BIOS first (decided 2026-10-19), the system may be offered on the player's BIOS before the open track is done; G9
then states that the open replacement is owed, as the policy requires before the system runs with no firmware folder.

- **G1, the processors.** ps2autotests' `cpu/*` all equal their expected texts, but entries named in a disputes entry;
  the IOP's crate at 100 % of the r3000 suite.
- **G2, the self-reporting programs.** All 78 ps2autotests on the player's BIOS; each failure a disputes entry.
- **G3, the pictures.** The project's GS programs equal to the referees' pictures or dispute-logged, with the
  referees' lineages agreeing (§1.5).
- **G4, the frontend.** The kit's C1–C15 on the tester's discs; cards round-trip; the fit audit; the CRT profile.
- **G5, the goldens.** The bench discs, each at three anchors, equal to the referees in picture and RAM, or
  dispute-logged.
- **G6, speed.** §5.12's desktop target on the bench; the handheld's measured and its misses named. *Amended
  2026-10-24:* §5.12's handheld target on the target games and the bench, in the default build, with the compromises
  taken listed in machine info and in §5.13; the reference path's speed on the desktop reported.
- *Amended 2026-10-24, for G3, G5 and GE.* The pictures and goldens are recorded on the **reference path**, which is
  what the referees are compared with and what disputes are settled on; the default path is graded against the
  reference, each difference attributed to an accepted compromise or treated as a defect. GE's exactness test (E1)
  pins the reference path.
- **G7, the debugger.** Each claim a test.
- **G8, the clean room.** `clonecheck.py` over Sapphire's and the shared crates' sources against the reference
  emulators' sources, the reference side never printed, finds nothing above its threshold; the disputes log read end to
  end by someone who did not write it (§11 Q13).
- **G9, firmware.** On the player's BIOS, every claimed game runs; with the open track done, every game the open BIOS is
  claimed to run runs with no firmware folder, machine info naming the replacement, and `firmwarecheck.py` passes
  against a control.
- **GE, the enhancements.** Each offered only with its stage's gate met and E1's exactness test passing in CI on every
  change to the core.
- **G10, start without a disc** (*added 2026-10-24 at the tester's request*). A player-facing way to boot the console
  to its menu with an empty tray, as the PlayStation's is (`Diamond_Shell.md` §1.1, §1.2 and §1.5, read 2026-10-24).
  The form is Diamond's and needs no new surface: the `ps2` system's core-info entry gains `starts_empty: true`
  (the core ABI's minor 1.11); `create` takes a null image with the system named and makes the console with the tray
  closed and empty, its firmware chosen from the files the host passes from the system's `firmware` list; the drive
  then takes a disc through `MEDIA`, which the kit's C22 requires of a system with a drive. Mistress offers it from core
  info alone, never from a list of consoles (`MainWindow.ConsoleStart.cs`, read 2026-10-24: `RuntimeCatalog`'s
  `ConsolesThatStartEmpty` feeds *Start a Console*, the library's *Start PlayStation 2* button, the pad menu's row and
  big picture's *Start PlayStation 2 With No Disc*), with *Insert Disc…* and *Eject Disc* in a running session;
  DianaSh's `core <corename> --console ps2` the same. **It needs the player's BIOS** until O12: with no BIOS in the
  firmware folder the start is refused with the core's words, as a game is, never with a black screen. Its tests: the
  kit's C22 on `ps2`; OSDSYS's picture at an anchor with an empty tray, not black and changed by a press, on the
  tester's machine only and never in CI (as `Diamond_Shell.md` §4.5); a synthetic disc inserted afterwards reaching
  the recogniser and the drive. Built at S25, when the BIOS reaches OSDSYS (S24) and the pad drives it; offered with
  the system at S29; on the open BIOS at O12.
- **G11, the Memory Cards window for the PlayStation 2** (*added 2026-10-24 at the tester's request*). Mistress's
  Memory Cards window gains the PlayStation 2's tab with the PlayStation's actions (`EmuSen_SaveMedia.md` §6.5, read
  2026-10-24; `MemoryCardsWindow.cs`, whose *Copy Save*, *Move Save*, *Delete Save*, *New Card* and *Import Every
  Game's Saves* reach the runtime's card manager as `CardsCopy`, `CardsMove`, `CardsDelete`, `CardsNew` and
  `CardsImport`; the fit audit's `MemoryCardsPlayStation` and `MemoryCardsPlayStationHeld` cases): a card's saves
  listed with title, size and game and the card's free space, copied or moved to another card, deleted, and imported
  from each game's own card, all or none. The window gains nothing of the PlayStation 2's own: the medium `ps2.card`
  (§4.4) is declared by the core as `removable`, and its file system is §9.5's crate `emusen-ps2-card` behind the card
  manager's format surface (`EmuSen_SaveMedia.md` §6.2: read, copy, delete and format, pure, bytes in and bytes out),
  so DianaSh's `memcard` gets the same actions. The PlayStation 2's saves are directories of files, not blocks, so the
  medium's unit is the crate's (clusters, or kilobytes as OSDSYS shows them), written down when the crate is. Its
  tests: the PlayStation 2's tab in WiseMan's fit audit at 1280 × 800 and 1920 × 1200 beside the PlayStation's cases,
  a held card's writes refused; the crate's copy, delete and import round-tripped on a card formatted by PCSX2 (a black
  box) and read back by PCSX2, and on one formatted by the player's BIOS on Sapphire where §11 Q7 lets the BIOS use a
  card. The window works on card files whether or not the player's BIOS accepts the card in play, so P8's refusal
  does not block it. Built at S25 with the crate; offered with the system at S29.

---

## 9. Shared code

The reuse rule: when two places need the same thing, it goes in one shared crate, with one owner for each fact. Each
item says what moves where, and why.

### 9.1 `emusen-mips`: the R5900, the MMI and the VUs

`emusen-mips` owns the instruction set of the project's MIPS processors (`EmuSen_Mips.md` §1), with no execution. The
R3000's decode is MIPS I exactly and the VR4300's is a decode of its own (`EmuSen_Mips.md` §2.4, §6). **The R5900 gets a
decode of its own too** [C]: `src/r5900.rs`, since its map differs from MIPS III in both directions (it adds the MMI
opcode space, the second multiplier's instructions, LQ and SQ, and lacks some MIPS III instructions [R the TX79 manual,
to be read at S5]), so sharing the VR4300's decode would make one opcode two rules, the argument `rsp.rs` was made by.
With it: the R5900's COP0 table (its registers' names), COP1's names, and COP2's macro-mode names in VU0's table. **The
VU micro mode** is a table of its own as well: a micro instruction is a 64-bit pair of an upper and a lower instruction
with flag bits, not a MIPS word, so it is `src/vu.rs`, with its own spelling, as `rsp.rs` is beside `vr4300.rs` [C]. The
assembler gains both, for the project's test programs and the open BIOS's entry code. **Nothing of Diamond's or MarsRT's
text changes**: their goldens are S5's gate. *Built 2026-10-24 at S5 (`EmuSen_Mips.md` §8, §9): the instructions
the R5900 lacks are DMULT, DMULTU, DDIV, DDIVU, LL, LLD, SC and SCD [R the TX79 manual, §3.3 and A.5], and on the EE the
FPU's double and long formats as well (`Sapphire_Disputes.md` D-1).*

### 9.2 `blackmoon-r3000` for the IOP

The IOP is an R3000A-family core, and `blackmoon-r3000` was built for it from the start: its README names Sapphire as
its second user, PRId is a constant a core sets for its own chip, and the caches and scratchpad were placed in the
core's bus, not the crate, so that a second core with different caches could use it (`BlackMoon_R3000.md` §2). **What
moves**: nothing. **What Sapphire adds in its own IOP bus**: the IOP's memory map, its caches and scratchpad, its wait
states, and the PRId of its chip; ps2tek notes that the BIOS chooses the PlayStation 2's boot path when "PRid >= 59h"
[R], so the PRId Sapphire sets is at least that, the exact value a dispute step's. *Amended 2026-10-25 at S21
(`Sapphire_Disputes.md` D-60): the sentence reads the other way, 59h and above being the EE's path, so the IOP's PRId is
below 59h, and at least 0Fh not to take TBIN's; Sapphire sets 1Fh, PCSX2's value.* **Not used**: `blackmoon-gte`, since the
IOP has no GTE in PlayStation 2 mode [A], and the PlayStation mode is Diamond's (§4.1). A dispute about the processor
itself goes to the crate's record and is checked against Diamond's goldens, since a change reaches both cores.

### 9.3 Diamond's SPU as the base of the SPU2

Diamond's SPU is inside the `diamond` crate (`src/spu`, `Diamond_SPU.md`). The SPU2's voices are the PlayStation's
voices [A, from ps2tek's description of the SPU2 as the SPU doubled with additions; read in full at S2], so **the voice
moves into a crate of its own**, `blackmoon-spu`, in BlackMoon-HW [C, §11 Q10]: ADPCM decoding, the interpolation table,
ADSR, sweep, noise and pitch modulation, each voice's state, and the reverb's algorithm with its addressing as a
parameter. **What stays in each core**: the register maps, the mixing and routing (one core in Diamond, two in
Sapphire), the transfer paths (the SPU's FIFO, the SPU2's AutoDMA), CD and external input, the output rate (44,100 or
48,000 Hz). **Gate**: Diamond's SPU goldens, every sample, unchanged by the extraction, run before Sapphire uses the
crate (S26). *Built 2026-10-25 (`BlackMoon_SPU.md`, `Sapphire_Native.md` §32): the gate met, the reverb's registers' layout and
the IRQ address's comparison staying with each core as well; the SPU2's further differences are D-260 to D-270.*

### 9.4 `emusen-disc` for DVDs

`emusen-disc` is a compact disc as sectors (`EmuSen_Disc.md` §1). A DVD is not a compact disc: 2,048-byte user sectors
with a 16-byte header of its own (sector ID with the layer bit, IED, CPR_MAI) and an EDC, no subchannel and no MSF
[R ECMA-267's contents, to be read at S23]. **What it gains** [C]: a medium kind, CD or DVD, chosen when an image is
opened; the DVD's geometry with one or two layers and the layer break (taken from where the image's second layer
begins, by the rule ECMA-267 and the PlayStation 2's discs give, settled at S23 and a dispute candidate); DVD sectors
read whole or synthesised with their header and EDC from ECMA-267, as CD sectors are from ECMA-130; CHD's DVD form
(`createdvd`); and later the block-compressed ISO forms, each from its published description. **What it does not gain**:
the CDVD's commands or timing, which are Sapphire's, as the HC05's are Diamond's. **Users**: Sapphire; later any core whose discs are
DVD-ROM. *Built 2026-10-25 at S23 (`EmuSen_Disc.md` §14): the break is the first volume's Volume Space Size where a second
volume descriptor lies 16 sectors past it (D-127, open); CHD's DVD form measured from chdman 0.289.*

### 9.5 Cards: a new `emusen-ps2-card`, not `emusen-psx-card`

`emusen-psx-card` is the PlayStation card's file system, 128 KiB of fifteen blocks (`EmuSen_MediumFormats.md`). The
PlayStation 2's card is another medium: 8 MiB, pages with spare bytes for error correction, erase blocks, a superblock,
a cluster-allocated file system with directories [R ps2tek "SIO2 PS2 Memcard Filesystem"]. Nothing of the layout is
shared, so **a new crate**, `emusen-ps2-card`, in `EmuSen/Cores/Shared/`, `no_std`, allocating nothing, as the three
medium crates are, with the same two users: the host's card tools and the open BIOS's card module, which may build on it
(`EmuSen_MediumFormats.md` §1) [C]. The PlayStation card in a PlayStation 2 is a PS1 game's concern and goes to Diamond
with the disc.

### 9.6 The recompiler: `emusen-jit`

**What moves** [C, §11 Q10]: from MarsRT's `cpu/blocks/jit/mod.rs` (239 lines [M]) and the parts of `cpu/blocks/` that
are not the VR4300's: the compiler's thread and its queue, the slots code is published in (one pointer, stored with
Release and read with Acquire), the Cranelift module with its W^X batches and its code budget, the counters, and the
shape of the verifier that steps an interpreter beside the blocks (`Mars_Native.md` §5.8.4, §5.8.5). They become
`emusen-jit` in `EmuSen/Cores/Shared/`, generic over a processor's lowering: a trait that turns a block's words into
Cranelift IR and names its exits. **What stays**: MarsRT's emitter (`jit/emit.rs`, 1,076 lines), its block shapes and
dispatcher; Sapphire's EE and VU emitters are its own. **Why now, and why not further**: two processors need the same
thread, publication and verifier, which are subtle (W^X, atomics that ThreadSanitizer sees, the code budget) and were
proven once; the emitters lower different semantics and share nothing worth a generic layer, which is
`Diamond_Plan.md` §9.1's argument against a shared interpreter. **Whose change**: the extraction touches MarsRT, so it
is made at S35 with MarsRT's own tests as its gate, its speed measured unchanged, and the tester's leave (§11 Q10).
Diamond may adopt the crate when its own recompiler is wanted (`Diamond_Speed.md` §7.2).

### 9.7 The framework's shared pieces

`emusen-native` (the exports, `StateWriter`, `SampleQueue` for 48 kHz, the observers, the settings schema),
`emusen-core-conform` (both forms, with the mapped-image estimate of §10), the test runner, `emusen-gpu` for the device
path and the enhanced renderer, and Galaxia for every path. Nothing new is needed beyond §10's items.

---

## 10. What the framework should gain, found by planning this core

Each item is to be weighed for its form (extension, module or component) in `EmuSen_CoreAPI_Extensions.md` before it
is built, as Diamond's were.

- **Per-button analogue values** ("pressure") for pads: twelve buttons of the DualShock 2 with 0–255 each. An optional
  capability in a minor, the digital reading staying the default for every other core.
- **128-bit registers** in `DEBUG_REGISTERS`, if S1's reading finds the ABI's register value narrower.
- **A mapped-only rule for large images**: a host that cannot lend a mapping refuses an image over a stated size
  rather than copying it; the kit's memory estimate counts a mapped image as mapped.
- **The routed system's recogniser** gains the PlayStation 2's `BOOT2` rule beside Diamond's `BOOT` (a system pack
  change, no ABI change).
- **A console medium** for the drive's EEPROM, under `EmuSen_SaveMedia.md`'s `console` kind, which exists.
- **The probe's hardware render**, if S3 finds PCSX2's standalone form with PINE insufficient: a headless OpenGL or
  Vulkan context for libretro cores that ask for one. *Evaluated 2026-10-24 at S3 (`Sapphire_Referees.md` §6): not needed by
  the PCSX2 family's cores; recommended not built, Play!'s pictures to be taken as RAM by the project's GS programs.
  Decided 2026-10-24 by the tester as recommended: not built now, headless EGL kept in reserve for the first GS dispute
  that needs it.*

---

## 11. Questions for the tester

*Answered: Q2, Q5, Q9, Q13, Q15, Q16 and Q17 decided 2026-10-23 (routine) as recommended; Q1, Q3, Q4, Q6, Q7, Q8, Q10,
Q12 and Q14 decided 2026-10-24 by the tester as recommended; Q11 changed by the tester on 2026-10-24, the handheld
being the full-speed target (§5.12). The questions are kept as asked, below, with each decision beside it.*

Each with a recommendation. **[decision]** changes scope, an interface, cost or what the player sees; **[routine]**
does not. **What S1 needs answered before it starts**: Q9 (the names and markets it declares) and the go. S2 needs Q1,
Q2 and Q3; S3 needs Q4 and Q5; S10 needs Q6; S6 needs Q14; S24 needs Q7; S28 needs Q8; S35 needs Q10's last part; S47
needs Q11.

1. **Q1 [decision], Sony's Linux-kit manuals** (§2.2). *Decided 2026-10-24 by the tester: as recommended.* May writers read the EE, VU, DMAC, IPU and GS manuals Sony
   shipped with the Linux for PlayStation 2 kit? *Recommended:* yes, each use cited by document and section, as
   Diamond's Q1 allowed Sony's documents, with ps2tek's screening (§2.2) applied to them for replacement writers; copies
   only from the tester's own or a lawful source, kept in the probe's cache, never committed. *Decided 2026-10-24 by the
   tester: archived online copies of the manuals Sony shipped with the kit count as that source*, the kit's own edition
   only, a licensee edition or a copy marked confidential refused at its title page. Fetched at S6 (§2.2,
   `Sapphire_References.md` §1.1).
2. **Q2 [routine], ps2tek's revision and status.** *Decided 2026-10-23 (routine): as recommended.* *Recommended:* pinned at `1c91660` (2022-01-27, its last push), cited
   as a document of unstated sources written by an emulator's author, its BIOS chapter screened and partly excluded for
   replacement writers (§2.2).
3. **Q3 [decision], SDKs and homebrew.** *Decided 2026-10-24 by the tester: as recommended.* ps2sdk's directories are named after the BIOS's IOP modules (§2.4), so it
   probably holds reimplementations of them; Open-PS2-Loader holds replacement drive modules. *Recommended:* their code
   is never read by anyone; running binaries built with them (ps2autotests) is allowed for every role; anyone who reads
   them is excluded from the open BIOS and from the stages that model the IOP's kernel interface. And may the project's
   own test programs be compiled with ps2toolchain's GCC (a compiler, not a library)? *Recommended:* yes for the
   compiler alone, linking none of ps2sdk; otherwise `emusen-mips`'s assembler and Rust only.
4. **Q4 [decision], the referees.** *Decided 2026-10-24 by the tester: as recommended.* *Recommended:* PCSX2's standalone release, pinned by SHA-256 like DuckStation's,
   run headless under a private compositor with its software renderer and read through PINE; Play! standalone as the
   second lineage; the libretro forms only if the probe gains a hardware-render context, which is built only if S3
   shows the standalone forms insufficient. All local black boxes, never distributed, sources never read.
5. **Q5 [routine], Play!'s own BIOS.** *Decided 2026-10-23 (routine): as recommended.* It runs only its high-level BIOS. *Recommended:* used for games' outputs only,
   as PCSX-ReARMed's kernel was for Diamond; a replacement writer never sees its kernel's own output.
6. **Q6 [decision], the open BIOS: form, and when.** *Decided 2026-10-24 by the tester: as recommended.* *Recommended:* form A, native kernel images for both processors
   (§5.9), in Rust (the IOP's on `mipsel-sony-psx`, the EE's on a restricted MIPS target with a CI scan), started only on
   the tester's go and written by people never exposed (§1.4); estimated at 30,000–50,000 lines and 25–40 steps. *And
   whether its EE kernel's first phase comes early* (O2–O3 before S10), so that the EE's ps2autotests run with no
   firmware folder as Diamond's tests did: *recommended* no, given the decision of 2026-10-19 — S10 runs on the player's
   BIOS once S24 exists, and the EE stages are graded by the project's own programs until then.
7. **Q7 [decision], the card's authentication under the player's BIOS.** *Decided 2026-10-24 by the tester: as recommended.* The BIOS authenticates a card before using it
   (ps2tek: SECRMAN sends F0h, F3h, F7h, "unknown purpose"); no public document describing the exchange was found (§12),
   and a trace of the BIOS's traffic is forbidden by §1.4. *Recommended:* (a) one research step for public documents
   of the protocol, including the published documentation (not the code) of open card-emulating devices; (b) if none
   suffices, cards on the player's BIOS path wait, and saving is offered on the open BIOS path only, which never
   authenticates. MagicGate's keys are never sought.
   *The research step, done 2026-10-25 at S25 (part A), `Sapphire_Native.md` §30.2:* the exchange's *sequence* is now
   documented in public, its *cryptography* is not, and answering it needs keys. A later edition of ps2tek (Matías
   Israelson's, revision `295bc61`, "SIO2 PS2 Memcards") names F0h's subcommands (the card sending CardIV,
   CardMaterial, CardNonce and three CardResponses, the console sending three MechaChallenges), F3h as "reset
   cryptography" and F7h as the selection of one of four MagicGate key sets (developer, retail, one unknown, arcade);
   the pinned ps2tek says the drive controller "contains the actual MagicGate keys" ("BIOS IOP SECRMAN"); the published
   README of SD2PSX, an open card-emulating device, says the console variants "use different magicgate keysets" which
   the device "must actively support". So a card answers the challenges with values computed under keys it holds, and
   the drive controller checks them under its own: no document gives a way to answer without the keys. **Q7(b)
   holds**: on the player's BIOS path the card's authentication commands get no answer (built, `sio2::card`), P8 to be
   recorded at part B, and the open BIOS path never sends them. Neither a key nor any device's firmware was sought or
   read; the later ps2tek edition credits ps2sdk's homebrew SECRMAN driver as its source, which was not read.
8. **Q8 [decision], games.** *Decided 2026-10-24 by the tester: as recommended, with three target games named: Jak and Daxter (a test copy in the probe's cache), Kingdom Hearts and Dark Cloud 2, the bench beside them.* The library holds no PlayStation 2 game; the handheld's card holds Jak and Daxter (USA,
   Rev 1) as an `.iso`, which is not touched. *Recommended:* dumps of discs the tester owns, under
   `AppSettings.RomDirectory`'s `ps2/`, DVDs as `.iso` and CD titles as `.cue`/`.bin`, read only. A bench chosen for what
   each loads: Jak and Daxter (VU1, streaming), Ratchet & Clank (VU1, effects), Gran Turismo 4 (GS and VU1 load, the
   progressive modes), Shadow of the Colossus (the heaviest load commonly cited), Final Fantasy X (IPU movies),
   Metal Gear Solid 2 (post-processing from eDRAM), Silent Hill 2 (render-to-texture, noise filter), Kingdom Hearts,
   God of War (a dual-layer DVD), one CD-ROM title, one PAL disc and one Japanese disc. Any subset serves.
9. **Q9 [routine], the BIOS entries and versions.** *Decided 2026-10-23 (routine): as recommended.* *Recommended:* three market entries with No-Intro canonical names,
   `ps2-0230a-20080220.bin` and `ps2-0230e-20080220.bin` for the Americas and Europe and the newest Japanese retail
   version for Japan, each listing every retail version's MD5 cited from a public database at S2; the gates run the
   Americas and Europe images the tester holds of version 0230 and the newest Japanese one the tester holds
   (`ps2-0210j-20040917.bin` by its name; *found at S2 to be the DESR recorders' image, so a console's takes its place,
   `Sapphire_Native.md` §12.7*); the images stay in `~/Documents/BIOS/PS2/`, read only, reached by the
   firmware folder setting and `--sysdir`, never read by the project's tools.
10. **Q10 [decision], the shared crates.** *Decided 2026-10-24 by the tester: as recommended.* *Recommended:* `emusen-mips` gains `r5900.rs` and `vu.rs` (§9.1);
    `blackmoon-r3000` serves the IOP unchanged (§9.2); Diamond's SPU voice moves into `blackmoon-spu` (§9.3), Diamond's
    goldens the gate; `emusen-disc` gains the DVD medium (§9.4); a new `emusen-ps2-card` (§9.5); and `emusen-jit`
    extracted from MarsRT at S35 (§9.6), MarsRT's tests and speed the gate.
11. **Q11 [decision], the speed targets.** *Changed by the tester on 2026-10-24: the handheld is the full-speed target, §5.12.* *Recommended:* the desktop target of §5.12 with the exact levers; the handheld
    best effort, measured and its misses named; the laptop not a full-speed target; Nephrite's 33 % quota rule not
    applied to this core.
12. **Q12 [decision], scope at the player gate.** *Decided 2026-10-24 by the tester: as recommended.* *Recommended:* the DualShock 2 with pressure, cards and the multitap;
    left out as §1.2 says: DEV9's disk and network, USB, i.LINK, DVD-Video, the PS1 mode (Diamond's), arcade boards and
    development consoles.
13. **Q13 [routine], the clean-room review.** *Decided 2026-10-23 (routine): as recommended.* *Recommended:* the clone check and the disputes reading at G8, as for
    Diamond.
14. **Q14 [decision], the EE's timing model.** *Decided 2026-10-24 by the tester: as recommended, for the reference; whether the default path keeps it is C1 of §5.13.* *Recommended:* the documented timing at instruction granularity with the
    caches modelled (§5.2), losses named, rather than a cycle-exact pipeline no oracle can grade.
15. **Q15 [routine], the recompiler's default.** *Decided 2026-10-23 (routine): as recommended.* *Recommended:* on once its six oracles pass, as MarsRT's was on
    2026-09-22, since the interpreter cannot run games at full speed; the interpreter a switch away and still exact.
16. **Q16 [routine], formats.** *Decided 2026-10-23 (routine): as recommended.* *Recommended:* `.elf` from S1; `.iso`, `.cue`/`.bin` and `.chd` at S23 with DVDs always
    mapped; `.m3u` at S29; the compressed and other dump forms later, each from a published description.
17. **Q17 [routine], cheats.** *Decided 2026-10-23 (routine): as recommended.* *Recommended:* plain RAM writes at the player gate; the devices' encrypted formats later,
    and only from published descriptions of their encryption.

---

## 12. Negative results

- No public PlayStation 2 FPGA core [A, to be confirmed at S2]; no console measurements by decision. *Confirmed
  2026-10-24 at S2 [M, GitHub's API]: MiSTer-devel's 415 repositories hold `PSX_MiSTer` and nothing for the PlayStation
  2; a repository search finds none.*
- *Added 2026-10-24 at S2:* no public document of the IOP's cache sizes [R, `Sapphire_References.md` §5.1]; ps2autotests'
  `ee_cop0/performance` records no count against a time base, so it gives D-0 nothing [R].
- No R5900 single-step suite (SingleStepTests' repositories, listed 2026-10-23) [M]; no AccuracyCoin-like suite; no GS
  picture oracle from consoles; ps2autotests' GS coverage is two signal programs [M].
- The probe answers no hardware-render request, so no PlayStation 2 libretro core draws through it as it stands [M];
  the probe has no README [M].
- No PlayStation 2 folder in libretro's cheat database and no PlayStation 2 system in OpenVGDB [M].
- No rustc target for the R5900 [R a search of rustc's platform documentation].
- No public document of the card's authentication exchange [R two searches]; ps2tek calls its commands' purpose
  unknown. *Revised 2026-10-25 at S25 (Q7's research step, §11):* its sequence is documented (a later ps2tek edition's
  F0h subcommands, F3h, F7h's key sets), its cryptography is not, and every source says the card and the drive
  controller hold MagicGate keys; so no document lets a card answer it without them.
- *Added 2026-10-25 at S25:* no public document of the PlayStation 2 multitap's commands (a search of the documents and
  psdevwiki; psx-spx calls its protocol unknown and gives a different peripheral byte from ps2tek's), of SIO2's port
  rates or SEND1 and SEND2, or of the card's own write and erase times (its flash's data sheet stands in,
  `Sapphire_Native.md` §30.5); PCSX2 makes a new card erased and formats none without a BIOS, so no referee card
  formatted by a referee alone exists (§30.9).
- Sony's Linux-kit manuals omit the IOP [R a 2002 forum post].
- ps2tek states no sources and quotes an emulator's values in places (§2.1).
- No PlayStation 2 game in the library and no measurement of any kind on the handheld or the laptop.

## 13. Predictions to be retired

- **P1.** D-0 is settled for 294.912 / 147.456 / 36.864 MHz on every model the referees and the test suite's logs can
  distinguish, with no model at 299 MHz.
- **P2.** ps2autotests' programs print through a channel that is not a plain EE serial port (they were run under
  ps2link), so S4 finds that at least some need the IOP and the SIF running before they print anything. *Held
  2026-10-24 at S4: an EE test's text reaches PCSX2's IOP console and not KPUTCHAR (`Sapphire_Native.md` §14.5).*
- **P3.** The whole is about 65 steps (48 accurate, 5 enhancement, 12 open BIOS); the GS's rules, the VU pipelines, the
  CDVD and the IOP's module interface overrun their single steps.
- **P4.** The R5900 differs from the TX79 manual in at least one instruction's result that ps2autotests records.
- **P5.** The GS stages raise more disputes than any other unit, and at least one is decided by a single witness for
  want of agreeing lineages.
- **P6.** At least half the disputes fall in §2.3's thin areas, most of them the CDVD's timing, the floats' last bit and
  the VU pipelines.
- **P7.** The IPU's IDCT departs from the standard's reference arithmetic in rounding, and one dispute settles it.
  *Held for the referees 2026-10-25 at S27 (`Sapphire_Native.md` §34.11, D-300): PCSX2's IDCT is one step off the
  reference in 1.4 to 6.6 % of the samples and Play!'s two; the IPU's own is not observable, and each lineage's larger
  departures lie elsewhere (D-301, D-303, D-304).*
- **P8.** The player's BIOS refuses an emulated card until Q7's answer is built. *Observed 2026-10-25 at S25 part B
  (`Sapphire_Native.md` §30.19): with a formatted card in slot 1 the browser never draws and no longer answers the pad,
  the card never written; whether the unanswered authentication is the cause waits for D-383's control.* *D-383 resolved
  2026-10-26 (`Sapphire_Native.md` §30.25 to §30.28): with the empty port's rule fixed, the browser stalls the same with
  slot 1 empty as with the card, and no command reaches the card on entering it, so the stall is not the card's refusal;
  P8 stands as observed, its cause put to the tester (§30.29 Q1).* *Decided 2026-10-26 by the tester: P8 recorded as
  observed, Q7(b) kept, and the SIO2out step taken (`Sapphire_Native.md` §30.31 to §30.37): with channel 12's slices now
  ending (D-540) the browser still stalls on entry, card or none, the card never written; P8 stands as observed.* *The SIF
  link step, 2026-10-26 (`Sapphire_Native.md` §30.38 to §30.43, D-620): every documented call is answered while the
  browser is entered; the EE never begins the browser's polling of the card server that PCSX2 begins two seconds after
  Cross, so the card is not reached and P8 stands as observed.*
- **P9.** All 78 ps2autotests pass on the player's BIOS within three steps of S24, but for entries named in disputes.
- **P10.** The open BIOS takes 25–40 steps and at least one bench game needs an IOP kernel behaviour no document
  describes.
- **P11.** Running VU1, the GS and the IOP on their own threads changes no golden.
- **P12.** With every exact lever, the desktop meets §5.12's target on the bench, and at least one VU1-bound bench game
  misses full speed on the handheld at first.
- **P13.** A dual-layer DVD image opens mapped in under a second on the desktop and the handheld; a copy would take
  tens of seconds and is never made. *Half held 2026-10-25 at S23 (`Sapphire_Native.md` §25.7): 1.15 ms cold on the desktop;
  a copy there took 0.52 s, not tens of seconds; the handheld not measured.*
- **P14.** Rewind on a 41 MiB state costs more than a frame's budget to snapshot at the default interval and needs its
  own setting. *Half held 2026-10-25 at S30 (`Sapphire_Native.md` §38.7): a capture costs the emulation thread 6.5 ms on the
  desktop, under a frame's 16.7 ms (8.5 to 9.1 ms estimated on the handheld), and a restore 4.3 ms; it needs its own setting
  for its reach, not its cost: the 96 MiB default holds about 1.5 s if a tenth of the state changes between captures.*
- **P15.** The EE recompiler built on `emusen-jit` with inlined instructions and held registers is at least five times
  faster than the interpreter on the bench, unlike MarsRT's few per cent, because the EE is the bound.
- **P16.** `blackmoon-spu`'s extraction leaves Diamond's SPU goldens unchanged at the first attempt.

*Amended 2026-10-24, by the speed decision; the predictions above are left as written.* P11 stands as a prediction
about the threads under the exact synchronisation of §5.1; with C7 taken it says nothing of the default path. **P12
stands, and is now a prediction about the reference path alone**: it predicts that the exact levers will not reach the
handheld's full speed, which is the case §5.13 exists for; it is retired at S47 by the reference path's handheld
measurement, whichever way that falls. P15 stands, measured with every compromise off. The rest are untouched. Three
are added:

- **P17.** The exact levers alone leave at least one of the three target games short of the handheld's full speed, so
  that at least one compromise of §5.13 is taken for the default path.
- **P18.** C5, the GS drawn by the host's graphics pipeline, changes pixels in every bench game against the software
  reference but changes no golden in RAM; and the bit-exact compute form fits the handheld's frame for at least one
  target game, so that C5 is weighed game by game rather than for the core.
- **P19.** No compromise is needed on the desktop for any bench game: the desktop meets full speed on the reference path
  with the exact levers alone.

## 14. Sources, read 2026-10-23

- ps2tek: <https://psi-rockin.github.io/ps2tek/> and <https://github.com/PSI-Rockin/ps2tek> (`1c91660`).
- ps2autotests: <https://github.com/unknownbrackets/ps2autotests> (`97469ff`, ISC).
- Toshiba, TX79 Core Architecture manual, rev. 2.0 (2001-04):
  <https://ftp.mirrorservice.org/sites/www.bitsavers.org/components/toshiba/_dataSheet/TMPR79xx-um-r2.0_200104.pdf>;
  TX7901 user's manual: <https://bitsavers.mirrorservice.org/components/toshiba/_dataSheet/TMPR79xx-um-r1.0_200111.pdf>.
- GCC manual, MIPS Options: <https://gcc.gnu.org/onlinedocs/gcc/MIPS-Options.html>.
- Kunimatsu et al., IEEE Micro 20(2), 2000; Suzuoki et al., IEEE JSSC 34(11), 1999: records at <https://dblp.org>;
  Hot Chips 11 slides: <https://old.hotchips.org/wp-content/uploads/hc_archives/hc11/2_Mon/hc99.s3.1.Kunimatsu.pdf>.
- Wikipedia, PlayStation 2 technical specifications:
  <https://en.wikipedia.org/wiki/PlayStation_2_technical_specifications>.
- The Linux kit's manuals: <https://www.everyeye.it/articoli/speciale-linux-ps2-1034.html>;
  <https://empegbbs.com/ubbthreads.php/ubb/printthread/Board/14/main/11921/type/thread>;
  <https://en.wikipedia.org/wiki/Linux_for_PlayStation_2>.
- PCSX2 command-line arguments: <https://pcsx2.net/docs/advanced/cli/>; IPC protocol (PINE):
  <https://wiki.pcsx2.net/IPC_Protocol> (refused a direct read, quoted by a search engine).
- Play!: <https://github.com/jpd002/Play->.
- libretro core information: <https://github.com/libretro/libretro-core-info> (`pcsx2_libretro.info`,
  `pcee2_libretro.info`, `play_libretro.info`); buildbot: <https://buildbot.libretro.com/nightly/linux/x86_64/latest/>.
- ES-DE systems: <https://gitlab.com/es-de/emulationstation-de/-/raw/master/resources/systems/linux/es_systems.xml>.
- Batocera, PlayStation 2: <https://wiki.batocera.org/systems:ps2>.
- chdman's DVD form: <https://docs.libretro.com/guides/disc-images/>; <https://ravener.is-a.dev/posts/compressing-games-to-chd/>.
- GitHub's API for licences, commits and directory names: ps2sdk, ps2toolchain, Open-PS2-Loader, ps2link, PCSX2,
  DobieStation, Play!, SingleStepTests.
- *Read 2026-10-24, at S2:* Redump, PlayStation 2 BIOS datfile (115, 2022-12-12): <http://redump.org/datfile/ps2-bios/>;
  ECMA-267, 3rd edition: <https://ecma-international.org/wp-content/uploads/ECMA-267_3rd_edition_april_2001.pdf>;
  ITU-T H.262 (02/2000): <https://www.itu.int/rec/T-REC-H.262-200002-S/en>; K. Diefendorff, "Sony's Emotionally
  Charged Chip", Microprocessor Report 13(5), 1999-04-19, a course's copy at
  <https://docencia.ac.upc.edu/ETSETB/SEGPAR/microprocessors/emotionengine%20(mpr).pdf>; copetti.org, "PlayStation 2
  Architecture": <https://www.copetti.org/writings/consoles/playstation-2/>; GitHub's API for MiSTer-devel's
  repositories. The rest are in `Sapphire_References.md` §1.
