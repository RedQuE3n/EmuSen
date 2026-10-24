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
- **P1–P16** *predicted*, numbered in §13 so that a later stage can retire each.

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

---

## 1. Purpose, scope and the clean-room protocol

### 1.1 The machine, and how accurate each part must be

The whole console is in scope but for the parts §1.2 leaves out. "Exact" means bit-identical results and, where a
column says so, the same cycle at which a result is visible to software. The clocks are ps2tek's [R ps2tek "EE
Architecture", "EE Timers", "CDVD Reads and Seeks"]; that they hold on every model is the first dispute (§2.3, D-0).

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
| **IOP** | An R3000A-family processor at about 36.864 MHz [R ps2tek "CDVD Reads and Seeks": "IOP_CLOCK is ~36,864,000 Hz"], 2 MiB of RAM, its caches and scratchpad (to be read at S2; the PlayStation's were 4 KiB and 1 KiB, `Diamond_Plan.md` §1.1), its interrupt controller, its DMA (channels 0–12, among them SPU2 on 7, SIO2 in and out on 11 and 12), six timers, three of 16 bits and three of 32 [R ps2tek "IOP DMA", "IOP Timers"] | Instructions exact through `blackmoon-r3000` (§9.2); timing to the bus access as Diamond's |
| **SPU2** | Two cores of 24 voices each [R Wikipedia, PS2 technical specifications, uncited there; to be confirmed at S2], 2 MiB of sound RAM [R ps2tek], 48,000 Hz output, ADPCM voices with the PlayStation's interpolation and envelopes, AutoDMA streaming of PCM to the memory-in area [R ps2tek "SPU2 AutoDMA"], the effects (reverb) area per core, the IRQ addresses, the core mixing and the external input | **Exact** to the sample, every voice and the reverb |
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
| Firmware: the boot ROM, the DVD player's ROMs, the drive controller's program, the DualShock 2's and the memory card's; any listing, disassembly, dump analysis or trace of them | **no** | **no** | **no** | §5.9; the player's images are executed by Sapphire and by referees, never read |

**The exposure rule carries over from Diamond** [C]: anyone exposed to BIOS code, to BIOS boot banners or BIOS output in
a referee's log, to a test suite's BIOS-derived code, or to ps2tek's excluded sections (§2.2), never writes the open
replacement. Running the player's BIOS on Sapphire to compare its pictures with a referee's (S24) is such an exposure:
the writers of the player's-BIOS stages and the replacement writers are therefore different people, chosen before
S24 [C]. The three rules of `Nephrite_Plan.md` §1.3 hold unchanged: the order of recourse is documents, then a test
program written for the purpose, then the black-box referees, never an emulator's source; nothing read in a dispute
step is typed into code in that step; the code carries no emulator's identifiers, which G8's clone check tests.

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
as it stands; and the probe has no README (the task of writing this page named one), its documentation being
`EmuSen_Debugging_Tools_Reference_v5.md` §3.50. A hardware context for the probe (headless EGL with OpenGL, or the
libretro Vulkan interface) is a feature of its own, evaluated at S3 against PCSX2's standalone form with PINE, which
needs no probe change (§11 Q4).

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
source, date and status in `DOCS.txt`, as Diamond's were.

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
downloads are small (ps2autotests is 1,850 KB [R GitHub API]); nothing large is fetched at this stage.

| Oracle | Source, version | Licence | Covers | How it reports |
|---|---|---|---|---|
| **ps2autotests** (unknownbrackets) | `unknownbrackets/ps2autotests` @`97469ff` (2019-11-16, the last push) [R GitHub API] | ISC [R] | **78 programs with expected results** [M, the repository's tree]: EE `cpu/ee/{alu, branch, branchdelay, lsu, lsudelay, muldiv}`; `cpu/ee_cop0/performance`; nine `cpu/ee_fpu/*`; six `cpu/ee_simd/*` (the MMI); seven IOP `cpu/iop/*` including `hilodelay`; `cpu/vu0_macro/{integer, transfer}`; `dma/dmac/{cpcond, tagintr}`; `dma/spr/{chain, interleave, normal}`; `dma/vif/{basic, intstall, mskpath3, stcycl, stmod, unpack}`; `gs/{label, signal}`; eight EE kernel thread tests and two interrupt-order tests; sixteen IOP kernel tests (event flags, interrupts, libc, message boxes, threads, variable pools); `memory/ee/default`; `vu/lower/{branch, efu, fdivdelay, integer, random}`, `vu/upper/clip`, `vu/games/triace` | Text on standard output, compared whole with the `.expected` file [R README]; `.elf` tests run on the EE, `.irx` on the IOP [R ibid.]. The README describes running each on a console through ps2link and saving its output as the expected file [R], so **the expected texts are consoles' outputs**. The binaries are in the repository [M]. Built with ps2sdk, so they call the BIOS's kernel and, for output, whatever channel ps2link gave them; which channel reaches Sapphire is found at S4 by running them (P2) |
| **The kernel tests** of ps2autotests | the same | ISC | The EE and IOP kernels' thread, interrupt, event flag, message box and pool calls | Oracles of the **player's BIOS path** and, later, of the open BIOS's behaviour; their sources test an interface and are not BIOS-derived [A], so a replacement writer may read them |
| **The project's own test programs** | Written at each stage, assembled with `emusen-mips` (§9.1) or built for the EE's and IOP's targets (§5.9) | GPL-3.0 | Whatever a dispute needs; bare metal, needing no kernel | Results written to RAM at a fixed address, as Diamond's `programs/*.s` do (`Diamond_Referees.md` §2.4), so every referee can be read through PINE or the probe |
| **MPEG-2 streams** of the project's own making | Encoded with a public encoder at S27 | GPL-3.0 | The IPU, against the standard's decoder arithmetic | Decoded pictures compared exactly |
| **Referees** | §1.6 | — | Everything, run whole | RAM, pictures, sound |
| **Commercial discs** | the tester's own, **none in the library** [M: `~/Documents/Roms/` holds no PlayStation 2 folder, the memory note of 2026-10-19]; one on the handheld's card, not touched (§11 Q8) | not redistributable | Integration and the goldens | References at anchors |

**Negative results of the survey** [M, GitHub's organisation listing and search, 2026-10-23]: SingleStepTests publishes
no R5900 suite (its repositories include `r3000`, which the IOP's crate already passes, and none for the EE); no GS,
SPU2, IPU or CDVD suite with console results was found; ps2autotests' GS coverage is two programs, both about
signals. **Nothing like AccuracyCoin is known for the PlayStation 2**, and the picture side of the GS has no console
oracle at all: its exactness can be graded only by documents, by the project's own programs and by referees (P5).

### 3.2 By component

| Component | Primary oracle | Secondary | Thin (§2.3) |
|---|---|---|---|
| R5900 integer, branches, delays | ps2autotests `cpu/ee/*` | the project's programs; referees' RAM | dual issue |
| MMI | `cpu/ee_simd/*` | the project's programs over every encoding against the TX79 manual's definitions | — |
| COP0, TLB, caches | `cpu/ee_cop0/performance`, `memory/ee/default` | the project's programs; referees | timing |
| FPU | `cpu/ee_fpu/*` | referees | the last bit |
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
form with Diamond's.

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
  FastForward.md`) is measured on it at S30 before rewind is offered (P14).
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

### 5.3 Floating point, exact

The FPU and the VUs follow the PlayStation 2's own format: no NaN, no infinity, no denormals, rounding towards zero,
results clamped where the documents say [R ps2tek]. The interpreter computes them in **software floats on integers**,
as MarsRT's COP1 does (`Mars_FpuMath.md`), so the result is the same on every host and in every tier. The last bit after
rounding is the first open rule (§2.3), graded by `ee_fpu/*` and `vu/*`. A recompiled tier may use the host's floating
point only where a test over every class of input shows it equal to the software path, with the host's rounding mode
set to truncation and denormals flushed; any input class where it is not equal stays on the software path [C]. This is
the largest single cost of exactness in the VU recompiler and is priced in §5.12.

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
  per the database's description, never by reading them.
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
oldest a player has had, as Nephrite's `OLDEST_STATE_VERSION` does.

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

**Targets** [C, §11 Q11]:

- **The desktop**: every bench game at full speed with the exact levers on, leaving a fifth of the frame: a mean of
  13.4 ms of wall time a frame for the emulation's threads together, and no frame over 16.7 ms at the 95th percentile.
- **The handheld**: full speed on battery for the bench games the desktop runs within 9.5 ms (13.4 / 1.4); the others
  measured and named. Some VU1-bound titles are expected to miss at first (P12).
- **The laptop**: **not a full-speed target**. Nephrite's rule (full speed at a 33 % CPU quota) cannot hold for a
  machine nine times the PlayStation's clock, and claiming otherwise would be dishonest; the laptop is measured and
  reported.
- **How measured**: Diamond's bench method (`Diamond_Speed.md` §2): `examples/bench.rs` timing each frame's `advance`,
  digests outside the timed span, builds interleaved on pinned cores, the aligned link as the instrument, fastest and
  median of rounds; threaded units measured as wall time with every thread's share sampled by `ipsample.py`; the
  handheld under `systemd-run --user` with no frontend running, on charger and on battery.

**The speed settings that are not exact**, offered only after all of the above (`effect: accuracy`, each with its
cost): the EE's timing reduced to a fixed cost an instruction; the VUs' floats on the host's arithmetic where it is not
proven equal; the CDVD's delays shortened. And, apart from accuracy, the enhancements of §6.

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

**Each stage is one supervised step of about three hours** with a check-in after it, as the brief for this plan set
[C]. A stage's oracle must be met, or its misses recorded as disputes or named losses, before the next that depends on
it begins. The effort is predicted (P3). Phases B to F are partly independent once A is done; the open BIOS (O) and
the enhancements (E) are separate tracks.

**Phase A — the frame**

| Stage | What it covers | Its gate |
|---|---|---|
| **0** | This plan | The tester's go (2026-10-23) |
| **S1** | The crate `sapphire` on the core ABI v1 as a stub with the `InDevelopment` mark: an ELF recognised and refused with words; the system pack `ps2` (§4.1) with the recogniser's `BOOT2` rule and Diamond's `BOOT` rule beside it; the three firmware entries (§5.9); the `SPH2` header | The kit's C1–C15 on a synthetic ELF and a synthetic ISO; a synthetic PS1 disc routed to Diamond, a PS2 CD and DVD to Sapphire, by test |
| **S2** | References: the documents of §2.1 fetched and pinned with `DOCS.txt` and `SHA256SUMS`; ps2tek's BIOS chapter screened by a non-replacement writer and the exclusions written; ps2autotests at `97469ff` with `PROVENANCE.txt`; the MD5 lists of §5.9 cited from a public database; the IOP's cache facts and the SPU2's voice count read | Each file pinned; the exclusions listed by section and line; every MD5 with its citation |
| **S3** | Referees: PCSX2 standalone headless (a private compositor, the software renderer, PINE for RAM), Play! standalone and its libretro form, the probe on LRPS2 and PCEE2 to measure the hardware-render refusal; their costs and the RAM each exposes | One project program's RAM record read from each referee that runs it; the referees' table of `Sapphire_Referees.md` written |
| **S4** | The runners (§3.3): `SyntheticPs2Elf`, the synthetic disc builder for ISO 9660 CDs and single- and dual-layer DVDs, the expected-text grader, the RAM-record reader; **one ps2autotests program run on PCSX2 with the player's BIOS** to learn its output channel by observation | A positive and a negative control for each runner; the channel ps2autotests prints on recorded (P2) |

**Phase B — the EE**

| Stage | What it covers | Its gate |
|---|---|---|
| **S5** | `emusen-mips`: the R5900's decode and disassembly as a table of its own (§9.1), MMI, COP0, COP1 and COP2's macro forms, the VU micro tables; the assembler for both | Every encoding of the TX79 manual's opcode tables decoded, disassembled and reassembled to the same word; the R3000 and VR4300 goldens unchanged |
| **S6** | The R5900 interpreter, integer: 128-bit registers, the MIPS III subset, the R5900's multiply forms and pipelines, branch-likely, LQ/SQ, exceptions; the timing model's issue and latency table | The project's programs over every integer encoding against the manual's definitions; the timing model's table pinned |
| **S7** | COP0, the TLB with the scratchpad mapping, the caches and CACHE, the memory map, Count/Compare and the performance counters | The project's programs on TLB refills, cache write-back and the counters |
| **S8** | MMI | The project's programs; `ee_simd/*` waits on S10 |
| **S9** | COP1 in software floats (§5.3) | The project's programs on every input class |
| **S10** | **The EE's ps2autotests**: the ELF loader and whatever the tests need to run — on the player's BIOS by fast boot once S24 exists, or earlier on the open EE kernel's first phase if §11 Q6 says so | `cpu/ee/*`, `ee_cop0/performance`, `ee_fpu/*`, `ee_simd/*`, `memory/ee/default` equal their expected texts, each miss a dispute |

**Phase C — the EE's peripherals**

| Stage | What it covers | Its gate |
|---|---|---|
| **S11** | DMAC (ten channels, chain and interleave modes, tags, the scratchpad channels, stall control), INTC, EE timers | `dma/dmac/*`, `dma/spr/*`, `kernel/ee/interrupt/*` |
| **S12** | VU0 macro mode, the VU float | `cpu/vu0_macro/*` |
| **S13** | VU micro mode, both units: the pipelines, flags, Q and P, EFU, branches, XGKICK, E/D/T | `vu/lower/*`, `vu/upper/clip`, `vu/games/triace` |
| **S14** | VIF0 and VIF1 | `dma/vif/basic`, `intstall`, `stcycl`, `stmod`, `unpack` |
| **S15** | GIF, its three paths and their arbitration | `dma/vif/mskpath3`; the project's programs on path interleaving against referees |

**Phase D — the GS, the software reference**

| Stage | What it covers | Its gate |
|---|---|---|
| **S16** | Registers, privileged registers, CSR and IMR, SIGNAL, FINISH, LABEL; the eDRAM's page and block layout of every format; the three transfer directions | `gs/label`, `gs/signal`; transfers round-trip through every format as the documents lay them out; referees' pictures of a transfer program |
| **S17** | Points, lines, triangles, strips, fans, sprites; flat and Gouraud; scissor; depth formats and tests | The project's primitive programs, each picture equal to the referees' (or a dispute) |
| **S18** | Texturing: formats, CLUTs (both storage modes), TEX0/1/2, LOD and mipmaps, filtering, wrap and clamp, fog | The same, per feature |
| **S19** | Alpha test, destination alpha, blending, PABE, FBA, colour clamp, dithering, masks, local-to-local overlap | The same |
| **S20** | The PCRTC: both circuits, merge, interlace in field and frame modes, NTSC, PAL and progressive, the picture handed to the frontend, the video timing | The referees' pictures at anchors of the project's display programs; the timing against D-0's ruling |

**Phase E — the IOP side, and the player's BIOS**

| Stage | What it covers | Its gate |
|---|---|---|
| **S21** | The IOP on `blackmoon-r3000`: its memory map, RAM, caches and scratchpad in Sapphire's IOP bus, its INTC, DMA and six timers, its PRId | The r3000 suite still passes; the project's bare-metal IOP programs |
| **S22** | The SIF: registers and its three DMA channels | The project's programs passing messages both ways, EE and IOP each checking the other's record |
| **S23** | The CDVD model (§5.6) and `emusen-disc`'s DVD layout (§9.4): reads, seeks, layers, CD modes and CD-DA, the clock, the EEPROM, the tray | Synthetic CDs and single- and dual-layer DVDs read sector for sector, the layer break placed; DVD images opened mapped on the desktop and the handheld, load time recorded (P13) |
| **S24** | **The player's BIOS to its menu**: what is missing for it, SIO2's pad, the EEPROM synthesised; the writers of this stage are thereafter not replacement writers | OSDSYS's picture at an anchor equal to PCSX2's on the same BIOS within a stated count of pixels, for the Americas and Europe versions of §11 Q9; then **all 78 ps2autotests on the player's BIOS** equal their expected texts, each miss a dispute |
| **S25** | SIO2: the DualShock 2 with pressure and rumble, the card (pages, erase, timing, the medium of §4.4, its crate §9.5), the multitap | The pad drives OSDSYS; a card formatted by the player's BIOS on Sapphire is read by the crate and by PCSX2, and the reverse, or the refusal of §5.8 recorded as P8 says |
| **S26** | The SPU2 on the shared voice crate (§9.3) | The project's voice programs against the referees' sound; Diamond's SPU goldens unchanged by the crate's extraction |
| **S27** | The IPU | The project's MPEG-2 streams decoded equal to the standard's arithmetic, each difference a dispute |
| **S28** | Games on the player's BIOS to their title screens and into play (needs §11 Q8) | The bench discs at three anchors each against the referees in picture and RAM, or dispute-logged |

**Phase F — players**

| Stage | What it covers | Its gate |
|---|---|---|
| **S29** | The PlayStation 2 to players: the pack's serial, region, shelf and media, the settings, the firmware page's entries, the DualShock 2 drawing, the CRT profile (§4.6), `.m3u` and the disc change | The kit's C1–C15 on the tester's discs in the corpus form; the fit audit with the PlayStation 2's tab, drawing and shelf at both sizes; `ConsoleScreenRuleTests` |
| **S30** | States: `SPH2`'s layout listing, loading back, the rewind codec measured on a 41 MiB state | States round-trip at every anchor of the bench; rewind's cost recorded (P14) |
| **S31** | The debugger: four processors, 128-bit registers, the VUs' disassembly, breakpoints, stepping, coverage | Each claim a test; every table armed gives the plain run's digests |

**Phase G — speed** (§5.12)

| Stage | What it covers | Its gate |
|---|---|---|
| **S32** | The bench and the profile, before any lever | The per-unit costs of §5.12's table replaced by measurements on the desktop and the handheld |
| **S33** | Idle skipping on the EE, the IOP and the VU waits | Every golden equal; the bench before and after |
| **S34** | The decoded caches (EE and IOP) | The same |
| **S35** | `emusen-jit`, the shared JIT crate extracted from MarsRT (§9.6) | MarsRT's block tests, corpus and game runs identical at every tier, and its speed unchanged within its noise; the crate's own tests |
| **S36–S39** | The EE recompiler: integer and branches (S36), loads and stores with the RDRAM and scratchpad fast paths (S37), MMI as vector code (S38), COP1 and COP2's macro forms (S39) | At each, Diamond's and MarsRT's six oracles: the interpreter stepped beside it and compared after every instruction, random programs, the mechanisms one at a time, ps2autotests, the bench games frame by frame, and a state loaded into an empty cache |
| **S40–S41** | The VU1 recompiler, micro mode, whole microprograms; then VU0 | The same, with every float class tested against the software path (§5.3) |
| **S42** | The units on threads (§5.12 lever 3) | Every golden equal threaded and unthreaded, at every anchor (P11) |
| **S43** | The GS's software rasteriser on worker threads | eDRAM equal after every frame; the bench |
| **S44–S46** | The GS on the device: the compute rasteriser's eDRAM mirror and transfers (S44), primitives and texturing (S45), the pixel pipeline and the PCRTC on the device with the picture handed through the slot (S46) | eDRAM equal to the software reference after every frame of every program and bench game; the validation layer with the synchronisation heuristic clean; RX 6800 only |
| **S47** | The targets (§5.12) | G6 |

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
| **O5–O9** | The IOP kernel: memory and the module loader with its linker (O5), interrupts and threads (O6), the synchronisation objects and pools (O7), I/O, the file server and SIF RPC (O8), the reboot into a disc's module image (O9) | ps2autotests' `kernel/iop/*` and `cpu/iop/*`; a game's own IOP modules loaded and linked |
| **O10–O11** | CDVD's file system and server (O10); SIO2, pads, the card's file system and server on §9.5's crate (O11) | Games boot with no firmware folder; their cards are read under the player's BIOS and the reverse |
| **O12** | The menu without a disc, and `firmwarecheck.py` against the tester's images with a control (`EmuSen_Firmware.md` §0) | G9 |

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
- **G6, speed.** §5.12's desktop target on the bench; the handheld's measured and its misses named.
- **G7, the debugger.** Each claim a test.
- **G8, the clean room.** `clonecheck.py` over Sapphire's and the shared crates' sources against the reference
  emulators' sources, the reference side never printed, finds nothing above its threshold; the disputes log read end to
  end by someone who did not write it (§11 Q13).
- **G9, firmware.** On the player's BIOS, every claimed game runs; with the open track done, every game the open BIOS is
  claimed to run runs with no firmware folder, machine info naming the replacement, and `firmwarecheck.py` passes
  against a control.
- **GE, the enhancements.** Each offered only with its stage's gate met and E1's exactness test passing in CI on every
  change to the core.
