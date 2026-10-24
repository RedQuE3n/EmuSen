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
