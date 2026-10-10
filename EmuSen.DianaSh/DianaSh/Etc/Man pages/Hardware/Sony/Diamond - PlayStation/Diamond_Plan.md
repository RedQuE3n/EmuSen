# Diamond_Plan — a clean-room PlayStation core in Rust

*Written 2026-10-10, at stage 0.* This page plans Diamond, EmuSen's Sony PlayStation (PS1, PSX) core, written in Rust
from hardware documentation, graded by test programs, and loaded through the core ABI v1 (`EmuSen_CoreAPI.md`,
`EmuSen_NativeCores.md`). Like Nephrite it has no predecessor: there is no C# PlayStation core to port or to reach
parity with, so the page follows `Nephrite_Plan.md`'s shape, with one section added for the enhancements the tester
decided on the day it was written (§6).

Every claim carries one of five marks:

- **[M]** *measured* on 2026-10-10, with the tool named beside it. Nothing was run to write this page beyond listings
  and the hosts' public APIs; the measurements of the core begin at stage 1.
- **[R]** *read* from a source, named, on 2026-10-10 unless another date is given.
- **[A]** *argued*: reasoning from the documents or from other cores' measurements, with no measurement here.
- **[C]** *a choice*: either a rule set for this core, said so, or a recommendation awaiting the tester (§11).
- **P1–P12** *predicted*, numbered in §13 so that a later stage can retire each.

The tester's decisions are dated where they are recorded. Section 6 records one, of 2026-10-10.

---

## 1. Purpose, scope and the clean-room protocol

### 1.1 The machine, and how accurate each part must be

The whole console is in scope. The target for each part is stated in hardware terms; "exact" means bit-identical
results and, where a column says so, the same cycle at which a result is visible to software.

| Part | What is in scope | Target |
|---|---|---|
| **CPU** (R3000A-compatible core in the CXD8530/CXD8606, 33.8688 MHz [R psx-spx "SPU": 768 × 44.1 kHz]) | MIPS I in full with the load and branch delay slots; COP0 without a TLB: SR's interrupt and mode stack, RFE, Cause, EPC, BadVaddr, the cache-isolation and swap bits, the debug registers (DCIC and the breakpoint registers), PRId; exceptions (interrupt, address error, bus error, syscall, break, reserved instruction, coprocessor unusable, overflow); MULT/DIV with their HI/LO latency and interlock | Instructions exact; timing to the cycle at every bus access (§5.2) |
| **I-cache and scratchpad** | 4 KiB I-cache, 256 lines of four words [R psx-spx "Memory Map"]; the BIU/cache control register at `FFFE0130h`; isolated-cache writes that the kernel's cache flush uses; 1 KiB scratchpad at `1F800000h` with no data cache [R ibid.] | Hits and misses exact, so fetch timing is the hardware's; the write buffer modelled (§5.2) |
| **Memory and buses** | 2 MiB main RAM and its mirrors, the RAM size register, the memory control registers (each region's delay and width), EXP1/EXP2/EXP3, the 512 KiB BIOS ROM at `1FC00000h` [R ibid.], open bus | Access widths and wait states exact; open-bus values where test programs pin them |
| **GTE** (COP2) | All 22 commands with `sf` and `lm`, MVMVA's matrix and vector selections including its documented faulty combination, the FLAG register's every bit, saturation, the division by the documented reciprocal table, the SXY/SZ FIFOs, IRGB/ORGB, LZCS/LZCR, the registers' read quirks; each command's cycle count and the interlock on MFC2/CFC2 while busy | **Exact** in results and in when they can be read |
| **DMA** | Seven channels (MDECin, MDECout, GPU, CD-ROM, SPU, PIO, OTC); sync modes 0 (with chopping), 1 and 2 (linked list); DPCR priorities and enables; DICR's interrupt logic; the CPU held during transfer | Words and order exact; transfer timing to the cycle where test programs measure it (P4) |
| **Timers** | Three root counters, their clock sources (system clock, dot clock, horizontal blank, system clock / 8), sync modes against the blanks, targets, one-shot and repeat interrupts, pulse and toggle | Exact to the cycle, the dot clock by the GPU's current mode |
| **Interrupts** | `I_STAT`/`I_MASK` with eleven sources (vblank, GPU, CD-ROM, DMA, three timers, the controller and card port, SIO, SPU, the lightpen line) | Exact edges and acknowledge |
| **GPU** | GP0 and GP1 in full; the command FIFO; VRAM (1024×512, 16-bit) and its transfers CPU-to-VRAM, VRAM-to-CPU and VRAM-to-VRAM with the mask bit; polygons flat, Gouraud and textured, quads as two triangles; rectangles; lines and polylines; fills; texture pages, CLUTs, the texture window, the texture cache and CLUT cache; four semi-transparency modes; dithering; drawing area and offset; the display area, 15- and 24-bit display, interlace; video timing and the blanks the timers see; the GPU versions [R psx-spx "GPU Versions": the 160-pin CXD8514Q and the 208-pin CXD8561Q family]; GPUSTAT's busy bits and drawing time | **A cycle-faithful software rasteriser first**: VRAM exact after every command; drawing time modelled so that busy bits and DMA pacing are the hardware's. GPU rendering and upscaling come later as opt-in settings (§6) |
| **SPU** | 24 voices of ADPCM in 512 KiB at 44,100 Hz [R psx-spx "SPU"]; the four-point Gaussian interpolation from its 512-entry table [R ibid.]; ADSR; sweep; noise; pitch modulation; key on and off timing; loop flags and the end flags; the reverb unit at 22,050 Hz [R ibid.] with its input and output filters; the capture buffers in the first 4 KiB [R ibid.]; the IRQ address; the transfer FIFO and its access-width behaviour; CD audio input with its volumes | **Exact** to the sample, every voice and the reverb |
| **CD-ROM** | The controller's register interface, its command set and responses with their timing; seeks; reads at 75 and 150 sectors a second [R psx-spx "CDROM Drive"]; whole-sector reads; filters by file and channel; **XA-ADPCM** at 37,800 or 18,900 Hz, 4- or 8-bit, mono or stereo [R ibid.] with its resampling to 44.1 kHz; **CD-DA** playback and reports; the volume matrix to the SPU; subchannel Q (GetlocP, LibCrypt); the lid and disc swap; SCEx region strings [R ibid.] | Results exact; timing to the measured distributions (§5.6), the open item where documents are thinnest |
| **MDEC** | Its commands (quantisation tables, the IDCT's scale table, decode); run-length decode, the IDCT as documented, YUV-to-RGB; 4-, 8-, 15- and 24-bit output; DMA in and out with its pacing | **Exact** output; timing as measured |
| **SIO0: pads and cards** | The port's registers, baud rate, /ACK timing and interrupt; the digital pad (SCPH-1080); the analog controllers and the DualShock (SCPH-1200) with its configuration commands, analog mode and rumble; memory cards (read, write, ID, the flag byte, timing); the multitap (SCPH-1070) | Protocol exact; byte timing exact where the kernel or a game measures it |
| **SIO1** | The serial port and link cable | Later (§11, Q12) |
| **Regions** | NTSC and PAL video timing; the console's region against the disc's SCEx string | Exact clocks, with one open discrepancy in the documents (§2.3, D-0) |

**Not in scope at first** (§11, Q12): the PocketStation, the light guns (GunCon, Justifier), the mouse, the NeGcon and
steering wheels, the link cable, the cheat cartridges' hardware, the arcade boards built on the PlayStation (Namco
System 11 and 12, ZN-1 and ZN-2), the Net Yaroze's serial cable, and the PS2's PlayStation mode.

### 1.2 Disc images and other inputs

| Format | What it holds | Recommended [C, §11 Q9] |
|---|---|---|
| **PS-EXE** (`.exe`, `.psexe`) | A program with its header, side-loaded into RAM | **Stage 1**: every test program arrives this way |
| **`.cue` with `.bin` tracks** | The common dump: 2,352-byte sectors, several tracks, pregaps | **Stage 7** |
| **`.iso`** | 2,048-byte data sectors only | Stage 7, raw sectors synthesised with their headers, EDC and ECC from ECMA-130 [A] |
| **`.chd`** (CD, version 5) | MAME's compressed hunks with metadata for tracks | Stage 7, through the shared disc crate (§9.2) |
| **`.m3u`** | A list of discs for one game | Stage 11, with the framework's disc change (§10) |
| **`.sbi`** | Subchannel Q patches for LibCrypt-protected PAL discs | Stage 7 |
| **`.ccd`/`.img`/`.sub`** | CloneCD's dump with full subchannel | Later |
| **`.ecm`** | Sectors with EDC/ECC removed | Later |
| **`.pbp`** | A PSP "EBOOT", the PlayStation's discs compressed for the PSP; the PSN's are encrypted | **Later, unencrypted only**; an encrypted PBP is refused with words, never decrypted |

ES-DE lists every one of these for the system but `.sbi`, which is read beside a disc and never opened alone [R ES-DE `es_systems.xml`, `psx`].

### 1.3 Who may read what

*Set 2026-10-10, when the core was started:* the clean-room rules of `Nephrite_Plan.md` §1.3, made stricter in one
place: **the FPGA referee is run only as a black box and its logic is never read, in a dispute step either** (§1.5).
A **writer** is anyone who writes Diamond's or a shared crate's code, tests or design pages; a **replacement writer**
is a writer of the open BIOS (§5.9); a **dispute step** is a supervised step whose only product is a disputes entry.

| Source | A writer | A replacement writer | A dispute step | Notes |
|---|---|---|---|---|
| Hardware documents (§2.1): psx-spx, the MIPS manuals, datasheets, ECMA-130/ECMA-119, the public research write-ups | yes | yes, but psx-spx's "BIOS Patches" section and any code fragment taken from the BIOS are excluded (§2.2) | yes | The primary sources |
| Sony's developer documents (§2.2) | per §11 Q1 | per §11 Q1, the same exclusion | per §11 Q1 | Cited by document and section |
| Test programs: binaries, their sources, READMEs and reference pictures (§3) | yes | yes, except a test program whose source is a BIOS listing | yes | A test's source says what it expects |
| Reference emulators **run** through the probe (§1.6) | yes, outputs only | yes, outputs of games, never a trace of the BIOS's own code | yes | Memory, picture, sound |
| PSX_MiSTer, **run** as a black box (§1.5) | outputs only | outputs only | outputs only | **Its RTL is never read**, by anyone, at any step |
| The SingleStepTests r3000 suite's **data** (§3.1) | yes | yes | yes | Its generator, a fork of an emulator, is never read |
| Any emulator's **source**: DuckStation, SwanStation, Beetle PSX and Mednafen, PCSX-ReARMed, PCSX-Redux (its emulator and its OpenBIOS), Avocado, ePSXe, no$psx, Rustation, MAME's PlayStation driver, Ares and the rest; any PGXP implementation or other enhancement code | **no** | **no** | **no** | §6.1 repeats it for the enhancements |
| Firmware images (the BIOS, the CD controller's microcontroller ROM, the DualShock's), and any listing, disassembly or trace of them | **no** | **no** | **no** | §5.9 |
| OpenBIOS's **source** | no | **no** | no | Its own documentation says it was written from a disassembly of a retail BIOS (§5.9) |

The three rules of Nephrite's §1.3 hold unchanged: the order of recourse is documents, then a test program written for
the purpose, then the black-box referee, and never an emulator's source; nothing read in a dispute step is typed into
code in that step; the code carries no emulator's identifiers, which the clone check of §8's G8 tests.

### 1.4 The disputes log

`Diamond_Disputes.md`, created with its first entry, in the shape `VenusRT_Plan.md` §1.4 gives. Its first entry is
already known, D-0, the NTSC frame rate (§2.3). A dispute in the R3000A crate is kept in that crate's own record (§9.1),
since it belongs to every core that uses the crate.

### 1.5 The FPGA referee, run only as a black box

| Core | Where | Commit | Licence | What its README says |
|---|---|---|---|---|
| PSX_MiSTer | `github.com/MiSTer-devel/PSX_MiSTer`, to be cloned to `~/Projects/psx-mister-reference` when stage 4 first needs it | `cd17b5a`, pushed 2026-08-21 [R GitHub API] | GPL-2.0 [R ibid.] | "Many games working"; it **requires a BIOS** (`boot.rom`, `boot1.rom`, `boot2.rom` by region) and takes CUE+BIN and CHD [R README] |

**How it is run, without reading it** [C, §11 Q4]. Two ways, in order of preference:

1. **On a MiSTer device**, if the tester has one: a test program or a disc image is loaded through its menu, and the
   picture is captured from its video output and the sound from its audio output. Nothing of the core is opened. A
   BIOS is still needed: the tester's own dump (§11 Q6), or Diamond's open BIOS once it exists (stage 10), which is
   itself a test of the open BIOS.
2. **In simulation**, with an open VHDL simulator (GHDL or nvc), if no device is available. Only the README, the
   licence, the directory names and the simulation folder's build scripts and its top level's port list are read, to
   learn how to feed it a file and where its outputs appear; nothing under `rtl/` is opened, and the clone's reading
   log names every file opened. Bare-metal test programs are the only input that needs no BIOS.

**What its agreement is worth** [A]. Unlike Nephrite's Nuked-MD parts, no PlayStation chip has been transcribed from
die images in public, so PSX_MiSTer is its author's reading of the documents and of hardware tests. Its agreement is
moderate evidence, and its disagreement starts a dispute rather than settling one. It is a referee, never a grader.

### 1.6 The black-box references

The probe's libretro backend (`EmuSen.WiseMan/Reference/probe-rs`) already passes a path to cores that need one
(`need_fullpath`) and takes a `--sysdir` of a person's own dumps [R `backends/libretro.rs`]. Every PlayStation core
below needs the path.

| Core | Buildbot (Linux x86-64) | Licence | Without a BIOS | Notes |
|---|---|---|---|---|
| **Beetle PSX** (Mednafen's PlayStation) | `mednafen_psx_libretro.so.zip`, 2026-10-10 [R buildbot listing] | GPLv2 [R its `.info`] | **refuses**: the three regional BIOS entries are not optional [R ibid.] | Deterministic states, memory descriptors [R ibid.]; `mednafen_psx_hw` is its hardware renderer |
| **PCSX-ReARMed** | `pcsx_rearmed_libretro.so.zip`, 2026-10-10 [R] | GPLv2 [R] | runs, every BIOS entry optional [R] (its own high-level kernel, of unknown origin; outputs only) | Deterministic states, memory descriptors [R] |
| **SwanStation** | `swanstation_libretro.so.zip`, 2026-10-10 [R] | GPLv3 [R] | every BIOS entry optional [R]; whether it runs without one is measured at stage 1 | The libretro descendant of an earlier, GPL DuckStation [A, from its name and licence; to be confirmed from its README] |
| **DuckStation** | **none**: no `duckstation` core is on the buildbot [R] | CC-BY-NC-ND, which permits redistributing unmodified releases [R its README] | — | A desktop application only; usable only if the probe gained a backend for it (§11 Q5) |

The cores are run as local instruments, never distributed, their sources never read. Their costs and what each exposes
through the standard calls (RAM, VRAM, SPU RAM, scratchpad) are measured at stage 1, as Nephrite measured its four
[A]. Every reference is compared, not trusted: they will disagree with each other where the hardware is thinly
documented, and the disagreement is the evidence that a dispute is needed.

### 1.7 What was read to write this page

No emulator source of any kind and no firmware or firmware listing. Read: EmuSen's own pages (`Nephrite_Plan.md`,
`Nephrite_Native.md` §1–§7 and §30–§35, `Beryl-HW/README.md`, `EmuSen_CoreAPI.md` §6.2, §6.7–§6.13, §12, §27 and §28,
`EmuSen_Firmware.md`, `EmuSen_Core_Naming_Scheme.md`, `EmuSen_LunaP_Kit.md` §0 and §5, `EmuSen_CRT.md` §3.2,
`EmuSen_Settings_Reference.md` §4.81); EmuSen's own code where this page cites it (MarsRT's `cpu/` module list and
`interp.rs`'s function list, `CrtFilter.ConsoleTiming`, `PadButton`, the probe's libretro backend); psx-spx's pages on
the memory map, the GPU's timing, the SPU, the CD-ROM drive and the kernel's contents list (§2.1); the READMEs of
ps1-tests, SingleStepTests' r3000, PSX_MiSTer and DuckStation; amidog's download pages; PCSX-Redux's OpenBIOS
documentation page (prose, not code); rustc's `mipsel-sony-psx` page; the libretro buildbot listing and the four cores'
`.info` files; GitHub's API for licences and commits; crates.io for the `chd` crate; OpenVGDB's `SYSTEMS` table; ES-DE's
`es_systems.xml`; the libretro database's cheat folder list.

---

## 2. Sources

### 2.1 The public documents

To be fetched at stage 1 into `~/.cache/emusen/probe/diamond/docs/`, each file with its SHA-256 in `SHA256SUMS` and its
source, date and status in `DOCS.txt`, as Nephrite's were.

| Source | Where, revision | Status | What it covers | Thin on |
|---|---|---|---|---|
| **psx-spx**, Martin Korth's (nocash) PlayStation specifications | The maintained conversion at `psx-spx/psx-spx.github.io`, branch `stable`, **pinned at `6d7d1bc`** (pushed 2026-10-10) [R GitHub API]; nocash's original at `problemkaputt.de`, last released by him on 2022-08-20 [R the conversion's front page] | Public, **no licence obtained** from its author, and by its maintainers' own statement **not clean-room**: it incorporates material from confidential Sony documents [R ibid.] (§2.2) | Every chip, the kernel's functions, cheat devices, controllers, memory cards, the CD formats | Cycle timing of the CPU's bus, the GPU's drawing time, the CD controller's response times, the reverb's arithmetic in places |
| The conversion's **development** version | The same repository, other branch | Its front page says it holds machine-written changes [R] | — | **Not used** [C, §11 Q2] |
| **MIPS R3000 references**: IDT's *R30xx Family Software Reference Manual*; Kane and Heinrich, *MIPS RISC Architecture*; LSI Logic's manuals for its R3000-compatible cores | Public; to be located and pinned at stage 1 | Public | MIPS I, COP0, the delay slots, exceptions, cache isolation | The PlayStation's own wait states, the write buffer's depth and timing |
| **ECMA-130** (CD-ROM data format) and **ECMA-119** (ISO 9660) | ECMA's site, free | Public standards | Sectors, modes, EDC/ECC, the subchannel's Q and its CRC; the filesystem the kernel reads | CD-ROM XA's mode 2 forms (the Green Book is not public) |
| **Sony's chip datasheets**: CXD2510Q/CXD2545Q (CD signal processor), CXD1199/CXD1815 (CD-ROM decoder) | Datasheet archives; to be located | Manufacturers' datasheets where public | The drive's DSP and the decoder's registers | The controller's microcontroller and its command protocol, which no datasheet describes |
| **Public research**: Joshua Walker's *Everything You Have Always Wanted to Know about the Playstation*, the GPU and GTE write-ups on psxdev.net and the PSXDEV forum, ps1-tests' and amidog's published descriptions | Public | Public write-ups | Supplements | Mixed reliability; each use cited |
| **Sony's patents** on the GTE, the GPU and the controller | Public patent offices; to be searched at stage 1 | Public | Intended designs | May differ from the shipped silicon |

### 2.2 Sony's confidential documents, and psx-spx's provenance

Sony's developer documents (the run-time library references and overviews, the hardware notes and the file-format
documents that circulated with the Psy-Q kits) are hosted on community sites without Sony's release. psx-spx itself
incorporates material from them, by its maintainers' statement [R]. Using psx-spx at all therefore means relying on
them indirectly; whether they may be read directly is the tester's choice (§11 Q1).

**The replacement writers' exclusions** [C, set 2026-10-10]: psx-spx's "BIOS Patches" section, which describes the
patches games apply to the kernel and the kernel code they replace [R the kernel chapter's contents]; any other page or
document that quotes BIOS code; and any program listing of the BIOS. The kernel chapter's function descriptions, which
read as interface specifications [R its contents], are allowed, as the Mega-CD BIOS manual's call interface was for
Nephrite's replacement. A page that mixes the two is screened first by someone who does not write the replacement and
either cleared or added to the exclusions in `DOCS.txt`.

### 2.3 Where the documents are thin

Each is a prediction of where the disputes log will fill (P6):

- **D-0, the NTSC frame rate.** psx-spx gives the nominal NTSC video clock as 53.693175 MHz and the PAL one as
  53.203425 MHz, 3,413 and 3,406 video cycles a line, 263 and 314 lines a field, and then computes the frame rates
  from 53.222400 MHz for both: about 59.29 Hz NTSC and 49.76 Hz PAL [R psx-spx "GPU Timings"]. From the nominal clocks
  the same counts give 59.817 Hz and 49.747 Hz [A, arithmetic]. The dot clocks it lists (5.32224 MHz for 256 pixels and
  so on) follow the 53.2224 MHz figure [R ibid.]. Which clock an NTSC console's GPU runs on decides the frame rate, the
  dot clock timer 0 sees and the CRT filter's phases (§4.6); a timer test on hardware, or the tester's console, settles
  it.
- **The CPU's bus timing**: the stall of an uncached fetch and of a data access by region, the write buffer's depth and
  drain, the I-cache refill. ps1-tests' `access-time` measures them on a console [R its README].
- **The GPU's drawing time** per primitive and per pixel, which sets GPUSTAT's busy bits and the pacing of GPU DMA.
- **The CD-ROM controller**: response delays of each command, seek times by distance, the spin-up, the read's first
  sector, what the controller does mid-command. The controller is a microcontroller running Sony's firmware (§5.6), so
  its timing is that program's, and no document describes the program.
- **The SPU's reverb and its filters** to the bit, the ADSR's exact steps at every rate, key-on latency, the SPU RAM
  transfer's quirks (ps1-tests reports data lost on 32-bit access [R its README]).
- **The MDEC's timing** and the IDCT's rounding.
- **The load delay slot's corner cases**: a load into a register a pending load is writing; LWL/LWR merging with a
  pending load; MTC0/MFC0 hazards.
- **The GTE's undocumented corners**: MVMVA's faulty selection, the FLAG bits' order of evaluation, cycle counts.
- **The two GPU versions' differences**, and the models' differences generally (§11 Q11).

---

## 3. The oracles, by component

### 3.1 What exists, and how each reports

To be fetched at stage 1 into `~/.cache/emusen/probe/diamond/`, never committed, each file listed in `PROVENANCE.txt`
with its source, version and MD5.

| Oracle | Source, version | Licence | Covers | How it reports |
|---|---|---|---|---|
| **SingleStepTests r3000** | `SingleStepTests/r3000` @`51e2274` (pushed 2026-09-04) [R GitHub API] | MIT [R] | Every valid MIPS encoding, 1,000 cases each: the 32 registers, HI/LO, PC, EPC, the jump target and Cause, **the branch and load delay states**, and each cycle's fetch, read or write [R README] | JSON, from its binary form by its own `transcode_json.py`. **Generated from an emulator's interpreter** (a fork of Ares) [R README]. Caveats it states: no coprocessor tests, so **no GTE**; exceptions disabled, so no address errors; memory flat, with no MMIO [R ibid.] |
| **ps1-tests** (Jakub Czekański) | `JaCzekanski/ps1-tests` @`f727802` (pushed 2023-03-05) [R] | MIT [R] | CPU (`access-time`, `code-in-io`, `cop`, `io-access-bitwidth`), DMA (`chain-looping`, `chopping`, `otc-test`), GPU (18 programs: fill rules, transparency, lines, mask bit, texture flips and overflow, the CLUT cache, clipping, GPUSTAT, UV interpolation, VRAM copies, version detection), GTE (`gte-fuzz`, `test-all`), MDEC (`4bit`, `8bit`, `frame`, `movie`, `step-by-step-log`), SPU (`memory-transfer`, `stereo`, `test`), timers, CD-ROM (`getloc`, `disc-swap`, a terminal) [R README] | Text on the TTY for most; **VRAM pictures** (`vram.png`) beside the GPU and MDEC programs, compared exactly by its `diffvram` tool [R]; `gte-fuzz` is said to verify against a real console [R]. Built with PSn00bSDK, so the programs call the kernel (§5.9) [A] |
| **amidog's tests**: `psxtest_cpu`, `psxtest_cpx`, `psxtest_gte`, `psxtest_gpu` | `psx.amidog.se` and an Internet Archive copy, 2013-12-31 [R] | **CC BY-NC-SA 3.0** for the CPU and CPX tests [R their download page]; the GTE and GPU tests' pages read at stage 1 | Basic and complex CPU behaviour; the GTE's registers and every command with `sf`, `lm` and MVMVA's bits against a software model, hours long on a console [R a search engine's quotation of the GTE page, to be read at stage 1] | On screen; the GTE test stops after a number of errors [R ibid.] |
| **Peter Lemon's PSX programs** | `PeterLemon/PSX` @`6d20c13` (pushed 2026-01-20) [R] | **none stated** [R GitHub API] | Bare-metal MIPS programs: CPU tests, GPU, SPU, MDEC block decode, image loading, decompression [R README] | Visual; its author tests on a console only [R ibid.] |
| **PSn00bSDK** examples (Lameguy64) | `Lameguy64/PSn00bSDK` @`5d9aa2d` [R] | not asserted by GitHub; per file [R] | Programs that exercise the kernel's interface as homebrew uses it | Visual; and a source of test programs the project writes itself |
| **The project's own test programs** | Written at each stage with the shared MIPS assembler (§9.1), or in Rust for `mipsel-sony-psx` [R rustc: tier 3, `core` and `alloc` only, nightly with `build-std`] | GPL-3.0 | Whatever a dispute needs | TTY text and RAM records, designed to be machine-read |
| **The probe** | §1.6 | — | Everything, frame by frame | RAM and the other spaces each core exposes, picture, sound |
| **PSX_MiSTer** | §1.5 | — | Disputes only | Picture and sound |
| **Commercial discs** | the tester's own, **none at present** [M: `/home/red/Documents/Roms/` holds no PlayStation folder, listed 2026-10-10] | not redistributable | Integration; the goldens | References at anchors |

**Nothing like AccuracyCoin is known for the PlayStation** [A, from the survey above; a search at stage 1 records the
negative result or finds one]. The single-step suite has no GTE cases yet; the GTE's oracles are `gte-fuzz`'s
`test-all` and amidog's test.

### 3.2 By component

| Component | Primary oracle | Secondary | Thin (§2.3) |
|---|---|---|---|
| R3000A instructions | SingleStepTests r3000, state, delay slots and every cycle | amidog's CPU and CPX tests; ps1-tests `cop` | load delay corners |
| CPU timing, cache, bus | ps1-tests `access-time`, `code-in-io`, `io-access-bitwidth` | programs written for each dispute | write buffer |
| GTE | ps1-tests `test-all` (hardware-generated, by its README's account of `gte-fuzz`) | amidog's GTE test; the r3000 suite if its GTE set is published | MVMVA, FLAG order |
| DMA, timers, IRQ | ps1-tests `dma/*`, `timers` | references' RAM at anchors | DMA timing |
| GPU | ps1-tests `gpu/*` VRAM exactly against their `vram.png` | amidog's GPU test, Peter Lemon's GPU programs; references' VRAM | drawing time |
| SPU | ps1-tests `spu/*`; references' audio | recordings of a console, if the tester can make them (§11 Q16) | reverb |
| CD-ROM | ps1-tests `getloc`, `disc-swap`; synthetic discs | games against references; PSX_MiSTer | response timing |
| MDEC | ps1-tests `mdec/*` VRAM exactly | Peter Lemon's DCT decode | timing |
| Pads, cards | the open kernel's card and pad functions against games; references | the tester's console and cards | byte timing |
| The open BIOS | games booting with no firmware folder; their cards readable under the player's BIOS | the player's BIOS run in Diamond as a black box | undocumented calls |

### 3.3 The runners

The generic test-ROM runner of `Nephrite_Native.md` §3 (`EmuSen.WiseMan/Fixtures/RomRunner/`) serves Diamond
unchanged: an engine over any v1 library, the probe as an engine, protocols as data. Diamond adds, each at the stage
that needs it:

- **A TTY reader**: the kernel's character output (the open kernel's `putchar`, and the EXP2 debug port some programs
  write) collected as text in a space, so the runner can grade "pass"/"fail" lines.
- **A VRAM grader**: the `VRAM` space against a reference `vram.png`, exactly, as ps1-tests' `diffvram` defines it.
- **A PS-EXE builder** in WiseMan, the counterpart of `SyntheticRom`, and a **synthetic disc builder** (ISO 9660 with
  `SYSTEM.CNF`, mode 1 and mode 2 tracks, a CD-DA track, XA interleave), so the project's CI never needs a commercial
  disc.
- **The r3000 harness** in the R3000A crate, with positive and negative controls, as Beryl's harnesses were built.

### 3.4 The conformance kit's corpus form

Diamond runs `emusen-core-conform` as every v1 core does: the single-image form on the synthetic images in CI, and the
corpus form (`EmuSen_CoreAPI.md` §28) over a list file, `kit/psx-sample.txt`, naming the tester's discs and the test
programs. One assumption of the corpus form does not hold for discs [A]: it sizes its jobs at 256 MiB an image
(§28.4 there), and a disc is copied whole at create (§5.11), up to about 700 MiB. Before the kit runs Diamond's discs,
it estimates an image's memory from its size (a framework change, §10).

---

## 4. What a frontend gets

Nothing here needs per-core code in a frontend (`EmuSen_CoreAPI.md` §2.4).

### 4.1 The system

| id | Name | Extensions | ES-DE | libretro cheats | OpenVGDB |
|---|---|---|---|---|---|
| `psx` | Sony PlayStation | `.exe`, `.psexe`, `.cue`, `.chd`, `.m3u`; `.iso` and `.pbp` as §11 Q9 decides (`.sbi` is read beside a disc) | `psx`, "Sony PlayStation" [R] | `Sony - PlayStation` [R the database's `cht/` listing] | `PSX`, matched by serial [R `SYSTEMS`: `systemSerial` 1] |

**Two extensions collide with Nephrite** [A]: Nephrite claims `.bin` for the Genesis and `.iso` for the Sega CD
(`Nephrite_Plan.md` §4.1), and both cores will claim `.cue` and `.chd`. The host routes by extension, so a second
engine claiming one is never reached. Diamond therefore claims no bare `.bin` (its tracks are reached through a cue
sheet), and the shared extensions need the system pack to decide the system from the contents, as Nephrite's stub
already does inside one core: a PlayStation disc by the licence text in its system area and by `SYSTEM.CNF`, a Sega
CD by `SEGADISCSYSTEM`. That is a framework change (§10, §11 Q9).

The game's serial for OpenVGDB and the cheat database is the `BOOT` line of `SYSTEM.CNF` (`cdrom:\SLUS_007.70;1` is
SLUS-00770) [A, from the format psx-spx describes]; the region is read from the system area's licence text.

### 4.2 Controllers, and the drawings

Two descriptors on ports 0 and 1, more with the multitap:

- **`psx.pad`**, the digital pad (SCPH-1080): Select, Start, the cross, △ ○ × □, L1 R1 L2 R2.
- **`psx.dualshock`**, the DualShock (SCPH-1200): the same with L3 and R3, two sticks, the Analog button and its LED.

**Canonical controls** [C, §11 Q10]: × → B, ○ → A, □ → Y, △ → X (the RetroPad's diamond, position for position), L1 →
L, R1 → R, L2 → L2, R2 → R2, L3 → L3, R3 → R3, Select → Select, Start → Start, the sticks to the two sticks. **The
Analog button has no RetroPad control** [R `PadButton`: B through R3, sixteen, no guide button], so it is a key binding
(`PadControl`) and a pad combination, until a later `PadButton` is appended (the enum is append-only).

**Rumble** has nowhere to go in v1.0: outputs are "pulled through events", and none is defined
(`EmuSen_CoreAPI.md` §6.8). The DualShock's motors are modelled in the core from stage 6 and reach a pad when a minor
adds the event (§10).

**The drawings in Controller Bindings.** Every console's tab draws its controller with LunaP's `ControllerDiagram`
(`EmuSen_Settings_Reference.md` §4.81), the Genesis's added at Nephrite's stage 6 (`Nephrite_Native.md` §35.1). Diamond's
stage 11 adds two drawings on LunaP's branch, shown by the pad the player's port is set to: the digital pad (the cross,
the four face buttons with their symbols in Sony's colours, Select and Start, the shoulder pairs drawn above the body
since a front view hides them) and the DualShock (the same with both sticks, their clicks as L3/R3 labels, and the
Analog button with its LED). The fit audit runs with both at 1280×800 and 1920×1200.

### 4.3 Spaces

`RAM` (2 MiB, the cheats' space), `SCRATCH` (1 KiB), `VRAM` (1 MiB), `SPURAM` (512 KiB), `BIOS` (read-only, 512 KiB,
whichever kernel is running), `CDBUFFER` (the controller's sector buffer), `CARD0`/`CARD1` (the cards); the processor
`R3000` with its bus `R3000BUS` as id 0, and the GTE's registers through `DEBUG_REGISTERS`.

### 4.4 Memory cards

Each card is a battery file: file 0 the card in slot 1, file 1 the card in slot 2, and further files for a multitap's
cards. **The format is the raw card**, 131,072 bytes in the card's own frame order, which is what `.mcr` and `.mcd`
files hold, so a card moves between Diamond and other emulators that store raw cards [A; checked against the references
at stage 6]. DexDrive's `.gme` and the PSP's `.vmp` add headers; importing and exporting them is a later tool, not the
battery format. **One card per game in slot 1, slot 2 empty**, is the recommended default (§11 Q12): it fits the
runtime's per-image battery files with no new framework, while a shared card, like a console's, needs a runtime option.

### 4.5 Cheats

The codes are console knowledge and go in the system pack's codecs (`EmuSen_CoreAPI.md` §6.12), from psx-spx's "Cheat
Devices" chapter [R its contents list]: **GameShark / Action Replay** (`8AAAAAAA VVVV` a 16-bit write, `3AAAAAAA 00VV` an
8-bit write, the `D0`–`D3` and `E0`–`E3` conditionals, `10`/`11`/`20`/`21` increments and decrements, the `50` slide
codes, the `C0`/`C1` delays), **Xploder** and **Game Buster** forms, which the libretro database also carries [R its
file names]. All are RAM pokes in `RAM`. **A gap** [A]: a conditional code tests one address and writes another, which
v1's poke quad (space, address, value, compare on the same address) cannot express; either the codec gains a
conditional form applied by the runtime between frames, or the core claims `CHEAT_POKES` with a richer list. Settled at
stage 11 (§10).

### 4.6 The CRT filter's timing

`CrtFilter.ConsoleTiming` takes a console's active picture in microseconds, the subcarrier cycles across it, and the
phase left over per line and per frame (`EmuSen_CRT.md` §3.2). For the PlayStation, from psx-spx's **nominal** clocks
[A, arithmetic on the [R] figures of §2.3]:

| | NTSC | PAL |
|---|---|---|
| Video clock | 53.693175 MHz = 15 × 3.579545 (fsc) | 53.203425 MHz = 12 × 4.43361875 |
| Active picture | 2,560 video clocks (256 at ÷10, 320 at ÷8, 512 at ÷5, 640 at ÷4) = 47.68 µs; 368 at ÷7 is 2,576 = 47.98 µs | 2,560 clocks = 48.12 µs |
| Cycles across | 170⅔ (as the NES, SNES and Genesis) | 213⅓ |
| Cycles per line | 3,413 / 15 = 227 8⁄15 | 3,406 / 12 = 283 5⁄6 |
| Per line | +8⁄15 | +5⁄6 |
| Per frame (263 or 314 lines) | +4⁄15 | +2⁄3 |

So the PlayStation's pattern crawls, unlike the Genesis's fixed bands. The entry `["PlayStation"] = Timing(2560 /
53.693175, 2560 / 15.0, 8.0 / 15, 4.0 / 15)` is added at stage 11 and **waits on D-0**: if an NTSC console's GPU runs at
53.2224 MHz instead, every number in the NTSC column changes. The console's video encoder by model, and its filters, are
to be found as `EmuSen_CRT.md` found the others' [A].

### 4.7 States, settings, the debugger

- **The state** is Diamond's own, magic `DMND`, written with `emusen-native`'s `StateWriter`: the magic, a version, the
  frame count, the disc's identity (its hash and the disc number in a set), then each block in a fixed order. About 3.6
  MiB [A: RAM 2 MiB, VRAM 1 MiB, SPU RAM 512 KiB, the scratchpad, the I-cache with its tags, the CD controller's buffer
  and state, the MDEC's, registers]. The disc is never in the state. The enhancements' blocks are optional (§6.2).
  Loading back to the oldest version a player has had, as Nephrite does since 2026-10-07 (`Nephrite_Plan.md` §5.7).
- **Settings** with the stages that make them meaningful: region (auto from the disc, Japan, Americas, Europe;
  `effect: none`), console model and GPU version (`none`, each a real console), the pad per port (`none`), the memory
  card mode, then the enhancements (`enhancement`, §6). Defaults are a real console's.
- **The debugger** is the generic one over `DEBUG`, `DEBUG_REGISTERS` and `DEBUG_DISASSEMBLE`, the R3000A through the
  `Observer` trait (§9.1); console views (VRAM, the SPU's voices) are extensions outside v1.

---

## 5. Design decisions

### 5.1 The clocks and the scheduler

- **One system clock**, the CPU's 33.8688 MHz, which is the SPU's 768 × 44,100 [R psx-spx "SPU"]. The GPU's video
  clock is a second domain, related to it by an exact rational (11/7 if D-0 finds 53.2224 MHz; otherwise the nominal
  crystal's ratio), so nothing drifts [A].
- **Every bus access advances the clock**, as `VenusRT_Plan.md` §5.2 and the Beryl crates arrange: the R3000A reports
  each fetch, read, write and internal cycle to a bus that owns time and adds the region's wait states. A scheduler holds
  the fixed events (the GPU's lines and blanks, the SPU's samples, the CD's sector times, the timers' targets) as times.
- **Catch-up**: the GPU is brought to the present at a GP0/GP1 access, a DMA block and a line's end; the SPU at a
  register access and at each sample time; the CD controller at its register accesses and its own events; the timers
  computed lazily from their sources. Exact where the access that triggers catch-up is the only coupling [A]; the test
  programs check it.

### 5.2 The CPU's timing model

**Recommended [C]: the R3000A's every access timed.** A fetch from an I-cache hit costs one cycle; a miss refills a line
from RAM or ROM at the region's cost; uncached fetches (KSEG1) pay every time. A data access goes to the scratchpad in
one cycle, to RAM or I/O at the memory control register's delay; a store enters the write buffer and stalls only when
it is full; MULT/DIV run in parallel and stall MFHI/MFLO until done; COP2 commands run in parallel and stall the next
GTE access until done. The load delay slot is architectural, not a stall. The exact counts are ps1-tests'
`access-time` measurements and the disputes they raise (§2.3). **Cost** [A]: a tag compare per fetch and a counter per
store, a few nanoseconds an instruction (§5.12).

### 5.3 The GTE, exact

Implemented from psx-spx's GTE chapter as arithmetic on fixed-width integers, every intermediate at the width the
documents give, each saturation setting its FLAG bit in the documented order; the division by its reciprocal table and
Newton step as documented; the commands' cycle counts in the scheduler. Its oracles are hardware-derived (§3.2), so
exactness is measurable: **every case of `test-all` and of amidog's test, bit for bit** (G1). Its own crate (§9.1).

### 5.4 The GPU: a software rasteriser, faithful in results and time

- **Command-granular, exact VRAM.** Each GP0 command is drawn whole, at the time the GPU would finish it, into the one
  VRAM that the CPU, DMA and the display read. The rasteriser follows the documented edge rules, the interpolation of
  colours and texture coordinates in the hardware's fixed point (affine, which is what makes PlayStation textures warp),
  the dither matrix, the four blend modes, the mask bit, the texture window, and the texture and CLUT caches, whose
  staleness a program can observe (ps1-tests' `clut-cache`).
- **Drawing time.** A command's cost (setup, per line, per pixel, textured or not, blended or not) is charged to the
  GPU's clock, so GPUSTAT's ready bits and the DMA's pacing are the hardware's (P5). The costs come from ps1-tests'
  `bandwidth`, `benchmark` and `gpustat` and from programs written for the disputes.
- **The display** is read from VRAM at its own time each line, so a mid-frame change to the display registers shows
  where the hardware shows it; interlace fields and 24-bit display (the MDEC's movies) are read as the hardware reads
  them.
- **Known losses at first**: the analogue output (the screen filters are the frontend's), and any effect below a
  command's granularity that a test finds, each listed as it is found.
- **`skip_rendering_state_neutral` is not claimed** [A]: VRAM is machine state that the CPU reads back, so skipping
  drawing would change what a game sees. A frame can skip *presenting*, never drawing.

### 5.5 The SPU

Every voice runs each sample: ADPCM decoded block by block, the pitch counter, the Gaussian interpolation from the
documented table, ADSR from its documented rates, then volume and sweep, the mix with CD audio, the reverb at half rate,
the output. **The output is 44,100 Hz exactly, reported as a fixed rate** as MoonRT's and MercuryRT's are
(`EmuSen_CoreAPI.md` §6.7); nothing is resampled in the core. XA-ADPCM's own resampling to 44.1 kHz is the hardware's
and is modelled as documented.

### 5.6 The CD-ROM controller

The controller is a Motorola HC05 microcontroller running Sony's firmware, beside the decoder and the drive's DSP
[R psx-spx "CDROM Drive"], and its firmware changed across console revisions [R ibid.]. **That firmware is a program its
maker shipped inside the machine, so the firmware policy covers it** (`EmuSen_Firmware.md` §0): Diamond's model of the
controller's command protocol, at its register interface, written from psx-spx's description of the commands and their
responses, **is the open replacement for it**. No image of it is asked for, and none is accepted [C]: a low-level model
of the HC05 running a dump would need a firmware listing to verify, which the clean room forbids.

- **Timing.** Each command's first and second response delays, seeks by distance, the read's sector period (75 or 150 a
  second), and what the controller does when commanded mid-operation are modelled as distributions measured by
  ps1-tests' `getloc` and by programs written for the disputes, with the references and PSX_MiSTer as witnesses. This is
  the part most likely to need disputes (P6) and the one where a game's loading most depends on accuracy.
- **SCEx.** A disc image does not hold the lead-in's region string [A: the string is outside the sectors a dump
  records]. Diamond reports a disc as licensed for the console's region, the region taken from the disc's system area,
  and states it in the machine info's words. A region mismatch is then a setting the player chooses, not a fault.
- **The firmware version** the controller reports (its date) is the chosen model's, from psx-spx's table [A].

### 5.7 The MDEC

The documented decode: the run-length codes, the quantisation, the IDCT with the scale table the program uploads, the
colour conversion, the output depths. Exact output is graded against ps1-tests' VRAM; the timing (how fast it consumes
input and produces output under DMA) is measured, since FMV playback's smoothness depends on it.

### 5.8 SIO0: pads, cards and the multitap

A byte exchanged on the port takes the baud rate's time; the device's /ACK follows after its own delay, which the
kernel's pad and card functions wait for. Each device is a state machine on its protocol: the digital pad's ID `41h`,
the DualShock's analog and configuration modes and its rumble mapping, the card's read, write and ID commands with its
flag byte, the multitap's four-port frame. A card's write takes its real time, since games show "saving" for it.

### 5.9 Firmware: the BIOS

The BIOS (512 KiB) holds the kernel (exceptions, interrupts, events, threads, the A0/B0/C0 function tables of about
three hundred calls: memory and string functions, files on the CD and the cards, the CD's reads, pads, the GPU's
helpers), the boot (reading `SYSTEM.CNF` from the ISO 9660 filesystem, loading the executable, setting the stack and
jumping to it), the memory card functions with their on-card format, and the shell (the boot logo, the CD player and the
card manager) [R psx-spx's kernel chapter contents]. Games call the kernel throughout, patch it in RAM, and hook its
interrupt chain. **Every disc and almost every test program needs it** [A: ps1-tests are built with PSn00bSDK, which
calls the kernel].

The policy (`EmuSen_Firmware.md` §0): never shipped, fetched or prompted for; an open replacement in the core; the
player's own image optional, used as the exact path when present; the replacement written from public documentation,
its writers never reading a BIOS disassembly or trace; and `firmwarecheck.py` showing how little of the original it
repeats.

**Option A, PCSX-Redux's OpenBIOS** (MIT). Its own documentation says: "this code has been written using disassembly
and reverse engineering of a retail bios", naming the US SCPH-7001 BIOS it was disassembled from, and argues the legal
precedents for doing so [R pcsx-redux.consoledev.net/openbios]. Its shell is its own, not reversed [R ibid.]. **That
settles the clean-room question by its own account**: it is a work derived from reading a disassembly, which the policy
forbids any replacement to be, whatever its licence. MIT licenses its authors' code; it does not make the work
clean-room. Its origin can be checked only by its authors' statement, which here is unambiguous, and by
`firmwarecheck.py` against a retail image, which measures repetition, not derivation. **Not compatible with the policy
or the clean-room rules** [A]. Running it as a black box (to boot a reference without a Sony BIOS) would make its output
a derivative of the BIOS's behaviour in a replacement writer's view, so it is not used at all [C].

**Option B, Diamond's own** [C, recommended, §11 Q3]. A MIPS program occupying the BIOS's address range, run by the
emulated CPU, so that games that hook, patch or call into the kernel find code and tables where the documents say they
are, and the CPU's timing and the interrupts are a game's as on hardware (the reasoning of Nephrite's Q4).

- **What it must provide**, written from psx-spx's function descriptions and the public homebrew documentation, never
  from code: (1) the reset path, the exception vector and the interrupt dispatch with the documented hook chain; (2) the
  A0/B0/C0 tables with every documented call, in documented order and calling convention, and the data structures games
  read (the process and thread control blocks, the event table, the device tables) at the documented places; (3) the CD
  boot: the ISO 9660 reader, `SYSTEM.CNF`'s `BOOT`, `TCB`, `EVENT` and `STACK`, the PS-EXE loader and the jump; (4) the
  memory card functions **in the real on-card format** (the directory frames, the block chain, the checksums), so that a
  card written under either kernel is read by the other; (5) pads through the kernel's pad functions; (6) a screen with
  no disc: a plain open screen and a minimal card manager, not Sony's shell.
- **Language** (§11 Q3): Rust, `no_std`, for the `mipsel-sony-psx` target, with a small assembly entry for the vectors,
  is recommended over assembly alone for a kernel of three hundred calls and two filesystems; the target is tier 3 and
  needs nightly with `build-std` [R rustc's page], so the binary is built by a pinned toolchain, committed with its
  source and its hash, and rebuilt by CI to the same bytes. The alternative is assembly with the shared MIPS assembler
  (§9.1), as Nephrite's replacements are written.
- **Two phases.** The kernel's core (vectors, interrupts, events, the tables' memory, string and TTY calls, the pad
  hooks) at **stage 3**, because the test programs need it from the start; the rest (CD boot, files, cards, the screen)
  at **stage 10**, once the CD-ROM and the cards exist.
- **Accuracy cost** (`effect: accuracy`), in a player's words first: "Most games start and save as on a console. A game
  that relies on an undocumented kernel call, reads the kernel's code, or depends on its exact timing may behave
  differently, and the start-up screen is EmuSen's own." **Risk: high** (P7): the documented surface is large, games
  patch the kernel in ways psx-spx's excluded section describes (so a replacement writer learns of them only by a game
  failing), and the card format must match bit for bit.
- **Oracles**: the test programs from stage 3, then games booting with no firmware folder, then the same games on the
  player's own BIOS in Diamond as a black box, comparing the games' behaviour, never the firmware's bytes; their cards
  exchanged between the two kernels.

**Option C, a kernel answered in Rust** (calls trapped at the table addresses): simpler for the documented calls and
breaks every game that hooks or patches the kernel. Not recommended.

**Option D, the player's BIOS only**, until option B's stage 10: during development the CD path runs on the tester's
own dump if one exists (§11 Q6), and machine info says `absent` otherwise. The policy requires option B before the
system is offered to players (G9).

**Declared files** [C]: per region, `scph5500.bin` (Japan), `scph5501.bin` (Americas), `scph5502.bin` (Europe), each
524,288 bytes, `required: false`, with the replacement's `effect: accuracy` and its cost; the names are the ones
players' own dumps commonly carry, as Nephrite chose. The CD controller's and the DualShock's microcontrollers declare
nothing (§5.6).

### 5.10 The state format

`DMND` (§4.7): the layout listing pinned by a test from version 1; an older version loaded in its own layout back to the
oldest a player has had, as Nephrite's `OLDEST_STATE_VERSION` does.

### 5.11 Disc images and the ABI

Nephrite's Q5 decided that the system pack parses `.cue` and the host passes the tracks as numbered files, a whole disc
copied at create, CHD later, no streaming minor until a measurement shows the copy matters (`Nephrite_Plan.md` §9 Q5).
Diamond takes that decision with one refinement for the shared disc crate [C, §11 Q9]: **the image passed to `create`
is the sheet itself** (`.cue`, `.m3u`), the system pack reads only its `FILE` lines to know which files to pass, and the
core's disc crate parses the sheet in full (tracks, modes, indexes, pregaps), so one parser of the format exists, in the
crate both cores use. A `.chd` is one file and needs none of this. **Size** [A]: a disc of up to about 700 MiB copied at
create is workable on 8 GiB machines and costs load time; whether a streaming read is needed is measured at stage 7 on
the handheld.

### 5.12 The speed budget

**The machines.** The Legion Go S is the low-end target and the i5-10310U laptop the floor. VenusRT's proxy for the
laptop is the desktop under a 33% CPU quota, a factor of 3.3–3.5; the handheld's single thread is argued at 1.3–1.4
times slower than the desktop's (`Nephrite_Plan.md` §5.5). A PlayStation frame is 16.7 ms NTSC or 20.1 ms PAL.

**The target** [C, §11 Q13]: Nephrite's rule, every bench game at full speed at 33% quota with a fifth of the frame
left for the frontend, which is a desktop mean of about **3.8 ms a frame** on the emulation thread.

**What accuracy costs, priced** [A]:

| Part | Work a frame (NTSC) | Desktop estimate |
|---|---|---|
| R3000A with the timing model | 33.8688 MHz / 59.8 Hz ≈ 566,000 cycles, ≈ 380,000 instructions at an assumed CPI of 1.5 | 1.1–2.3 ms at 3–6 ns an instruction, the range Nephrite's plan argued for an interpreter (its §5.5) |
| GPU, software, cycle-costed | 100,000–500,000 pixels in a 3D game, textured and blended | 0.4–2.0 ms at 3–4 ns a pixel |
| SPU, 24 voices and reverb | ≈ 17,700 voice-samples and 370 reverb steps | 0.2–0.4 ms |
| GTE | a few thousand commands | 0.05–0.25 ms |
| MDEC (during movies) | ≈ 1,200 blocks a frame at 320×240 | 0.3–1.0 ms |
| CD, timers, DMA, SIO | events | < 0.2 ms |
| **Total** | | **2.0–6.0 ms**; a heavy 3D scene likely above 3.8 ms with the interpreter alone (P2) |

**The exact levers, in order** (each bit-identical, checked against the plain path; `effect: exact`):

1. **The rasteriser on worker threads**, commands drawn in order, as MarsRT's RDP threads are (`Mars_Native.md`): the
   GPU leaves the emulation thread. The display and readbacks wait for the commands before them.
2. **A decoded-instruction cache** keyed by I-cache line and invalidated by stores to code, with the timing model kept
   exact (a block's fetch costs are its lines' hit or miss, computed as the hardware would).
3. **Idle skipping**: a polling loop on `I_STAT`, a timer or GPUSTAT skipped to its next possible change, exact where its
   exit depends only on what the skip watches (MarsRT's `RunIdle`).
4. **A recompiler** on MarsRT's Cranelift path, bit-identical to the interpreter by test.

**The speed settings that are not exact**, offered only after those (`effect: accuracy`, each with its cost): the CPU's
timing model reduced to a fixed cost per instruction; the GPU's drawing time made instant; the CD controller's delays
shortened. And, apart from accuracy, the enhancements of §6.

---

## 6. Enhancements

### 6.1 Decided 2026-10-10

*Decided by the tester on 2026-10-10:* **PGXP-style precision is a feature of Diamond, written from scratch.** No
existing PGXP code is used, and no emulator's PGXP or enhancement code is read: the clean room of §1.3 applies to the
enhancements as to the rest of the core. What such precision does may be learned only from published descriptions and
from the hardware's own behaviour (the GTE's fixed-point outputs and the GPU's integer vertices). It is designed as
EmuSen's own: sub-pixel and depth precision carried alongside the GTE's results to the GPU, so that polygons stop
wobbling and textures stop warping.

Beside it, **other game-agnostic corrections of the PlayStation's flaws**, each an opt-in feature of the same family.
The aim is for EmuSen to stand apart from existing emulators on three things at once: accuracy as an option, features,
and architecture.

### 6.2 The architecture

The rules of the decision, and how the design keeps them:

- **The accurate path is the default, and stays bit-exact when every enhancement is off.** A test holds it: the
  accurate path's own golden outputs (every test program's verdict, VRAM and TTY; the bench discs' frame, audio and
  state digests at their anchors) are recorded at each accurate stage, and the build with every enhancement compiled in
  and off must reproduce them exactly (E1's gate). The kit's C9 already requires `exact` settings to change nothing;
  this goes further and pins the off position of every enhancement to the accurate path.
- **A separable layer, not edits through the accurate code.** Two parts, each its own crate:
  1. **The precision side-channel.** The R3000A and the GTE report through an observer trait, generic and compiled out
     when unobserved, as Beryl's `Observer` is (`Beryl-HW/README.md`): the GTE's untruncated results at each
     projection, and the CPU's moves of those values (MFC2, SWC2, loads and stores). The layer keeps a shadow of RAM
     and the scratchpad: for each word holding a projected vertex, the precise value and the integer it was made from.
     When the GPU receives a vertex word (by the CPU or by DMA), the layer looks the word up by address and integer; a
     match attaches the precise vertex to the command as side data, a mismatch (the game changed the vertex) leaves the
     integer one. **The accurate GPU never reads the side data.**
  2. **The enhanced renderer**, a second renderer that consumes the same GP0 command stream with its side data and
     draws the picture the player sees. **The software rasteriser keeps running and keeps VRAM**, which is machine
     state: the CPU's readbacks, VRAM-to-VRAM copies and the display's own reads see the accurate VRAM, so an
     enhancement cannot change what a game computes. This is MarsRT's arrangement for its internal resolution, "the
     picture drawn at a multiple of the console's, beside the exact drawing games read back" (`EmuSen_CoreAPI.md`
     §6.13's example).
- **Where an enhancement needs a GPU renderer** (internal resolution, filtering, depth), the enhanced renderer is a
  **Vulkan renderer on the shared device of `EmuSen_LunaP_Kit.md` §5.2**, beside the software rasteriser, so that the
  two never mix: the software rasteriser owns VRAM, the Vulkan renderer owns only the presented image. Until the kit's
  device exists, it runs on the device MarsRT's GPU path uses, with ash at the version wgpu-hal uses (0.38 today,
  `EmuSen_LunaP_Kit.md` §5.2) so the move is a change of who creates the device. On a machine without a usable GPU,
  lavapipe runs the same code (ibid. §5.3), slowly; the enhancements are then honestly costed as such.
- **Each enhancement is its own setting**, in the console's tab of Graphics Settings, with `effect: enhancement`, its
  `hardware` value the default (the kit's C3 checks it), and a cost in a player's words.
- **Save states stay compatible.** An enhancement's state (the shadow memory, the renderer's caches) is an optional
  block after the accurate blocks, tagged and skippable. A state made with enhancements loads without them, the block
  ignored; a state made without them loads with them, the shadow starting empty and filling within a frame [A]. No
  enhancement block is ever needed to reproduce the machine.
- **Two enhancements change the emulation itself**: the CPU overclock and the faster CD. They are not picture layers;
  they are parameters of the accurate model (a clock ratio, a timing table) whose hardware values are the defaults, and
  they are tagged in the state so that a state is loaded with the timing it was made with.

### 6.3 The enhancements, one by one

| Enhancement | Game-agnostic? | Needs | Cost | What it can break |
|---|---|---|---|---|
| **Sub-pixel vertex precision** | Largely [A]: it applies wherever a vertex reached the GPU unchanged from a GTE projection | The side-channel; the enhanced renderer | The shadow memory (a few MiB) and an observer call on GTE results and on stores; a lookup per vertex | Vertices a game adjusts on the CPU after projection (2D overlays built from polygons, snapped UI) fall back to integers, and a polygon mixing precise and integer vertices can show a seam where neighbours disagree; games that never use the GTE gain nothing |
| **Perspective-correct texturing** | Largely, on the same vertices [A] | Each precise vertex's depth from the side-channel; the enhanced renderer | A division per pixel, or per span, in the renderer | Polygons whose vertices lack depth (sprites, 2D quads) stay affine, which is correct for them; a polygon with some vertices precise and some not would interpolate inconsistently, so such a polygon is drawn wholly affine |
| **Depth from the precision path** | **Partly** [A]: PlayStation games order by their ordering tables and draw overlays, shadows and decals coplanar or out of depth order on purpose | Precise depth per vertex; a depth buffer in the Vulkan renderer | A depth buffer at the internal resolution | A plain depth test hides HUDs, decals and transparent layers; the useful form uses depth only to break ties the ordering table leaves, and remains experimental, off, with that cost stated (§11 Q14) |
| **Higher internal resolution** | Largely, through the Vulkan renderer at N× [A] | The Vulkan renderer; VRAM's picture kept at N× beside the accurate VRAM | N² the drawing and the memory; a GPU | Effects that read the framebuffer back as a texture must be handled by the renderer's own N× copy (a game's copy of its display into a texture page); 2D sprites made of several polygons can show half-pixel gaps; games that draw at 480 lines gain little |
| **True colour without dithering** | Yes [A] | The Vulkan renderer drawing at 24 bits per pixel; or, as the cheapest form, the dither left out of the enhanced picture | Little | Games whose art was made for the dither (gradients, fog) can band; the accurate VRAM keeps 15-bit values, so readbacks are unaffected |
| **Texture filtering** | Partly [A] | The Vulkan renderer | Little on a GPU | Filtering after the CLUT lookup bleeds across the edges of packed texture pages and around colour-0 transparency, and inside texture windows; it is offered with those exceptions handled by clamping to the primitive's texture rectangle |
| **Widescreen** | **Not in general** [A]: a game decides its own field of view, culls what lies outside 4:3, and draws its HUD without the GTE | A scale of the GTE's horizontal projection, and an N× renderer wider than the display area | Little | Geometry missing at the widened edges where the game culled it, the HUD stretched, 2D games unchanged. The only agnostic form is that projection scale, offered as experimental; a true widescreen is per game, by patch codes, which are outside this family |
| **CPU overclock** | Yes, as a setting [A] | The clock ratio in the scheduler | The host's time, in proportion | Games that time with CPU loops run fast; movie and audio sync can drift where a game paces by CPU work; slowdowns become smooth where the CPU was the bound, and not where the GPU or the CD was |
| **Faster CD access** | Mostly [A] | Shorter seeks and a higher read rate in the controller's timing table, kept at the real speed while XA or CD-DA streams | Nothing | Games that depend on a read taking its time (loading screens that race, copy protection that times the drive, streaming not marked as XA); a list of exceptions is learned game by game, so the setting states that cost |
| **Seams and gaps between polygons** | Partly [A] | The enhanced renderer: precise shared vertices so neighbours agree; optionally a conservative expansion of edges by a fraction of a pixel | Little | Expanded edges draw twice where semi-transparent polygons meet (double blending at the seam), and soften sprite edges; offered off, with that cost |

---

## 7. Stages

Each stage is a run of supervised steps of about three hours with a check-in after each, as Nephrite's ran. **The
effort is predicted** (P3). A stage's oracle must be met, or its misses recorded as disputes or named losses, before the
next begins. The order puts a drawn picture at stage 4 and a booting disc at stage 7 (with the player's BIOS) or stage
10 (with none), as early as the dependencies allow.

| Stage | What it covers | Its gate: what passes, measured how | Steps |
|---|---|---|---|
| **0** | This plan | The tester's go | 1 (2026-10-10) |
| **1** | The crate `diamond` on the core ABI v1 as a stub (the image recognised: PS-EXE, a disc's system area); the system pack `psx`; the declared firmware; the corpus, the documents, the referee and the references pinned outside the repository; the probe on the three libretro cores with their costs measured; the runners' first pieces (§3.3); the R3000A and GTE crates as skeletons with the r3000 harness and its controls | The kit's C1–C15 on a synthetic PS-EXE and a synthetic disc; the r3000 replay control passes every case and the spoiled replay none; the references' costs and spaces recorded | 2 |
| **2** | The R3000A: MIPS I with both delay slots, COP0, exceptions and interrupts, MULT/DIV latency; the disassembler and the assembler (the shared MIPS crate, §9.1); its state | **SingleStepTests r3000, every case: registers, delay states and every cycle**, except cases named in a disputes entry | 3 |
| **3** | The bus and the kernel's core: the memory map, RAM, scratchpad, I-cache and the timing model, memory control, interrupts, timers, all seven DMA channels, PS-EXE loading; **the open kernel's first phase** (§5.9); the TTY reader | ps1-tests' `cpu/*`, `dma/otc-test`, `dma/chain-looping`, `timers` pass by their TTY, on the open kernel with no firmware folder; `access-time`'s table equals the console's within the test's tolerance, each difference a dispute | 4 |
| **4** | **The GPU**: GP0 and GP1, the FIFO, VRAM transfers, every primitive, texturing, CLUT and texture caches, blending, mask, dithering, drawing area, the display with 15- and 24-bit and interlace, the video timing, the drawing time and GPUSTAT | ps1-tests' 18 GPU programs, **VRAM exactly equal to their `vram.png`** (or a dispute); amidog's GPU test; Peter Lemon's GPU programs against the references' pictures; GPUSTAT's busy bits as `gpustat` reports on a console | 6 |
| **5** | **The GTE**, exact, with its timing | **ps1-tests' `test-all`, every case bit for bit; amidog's GTE test runs to its end with no error**; ps1-tests' `gte-fuzz` cases against the references | 3 |
| **6** | SIO0: the digital pad, the DualShock with its modes and rumble state, memory cards with their timing, the multitap | The open kernel's pad functions read the pad on a test program; cards read and written through the protocol, their files the raw 128 KiB form, round-tripped with a reference's card; the multitap's four ports on a test program | 3 |
| **7** | **The CD-ROM** and the shared disc crate (§9.2): the controller's commands, responses and timing, reads, filters, XA-ADPCM, CD-DA, subchannel Q and `.sbi`, the lid; `.cue`/`.bin`, `.iso`, `.chd`; the content-based routing (§10) | ps1-tests' `getloc` and `disc-swap`; synthetic discs read sector for sector; XA and CD-DA streams on synthetic discs equal to their sources; games boot **on the player's BIOS** (if any, §11 Q6) to their title screens, against the references | 5 |
| **8** | **The SPU**: voices, ADPCM, Gaussian interpolation, ADSR, sweep, noise, pitch modulation, reverb, capture, IRQ, transfers, CD audio in | ps1-tests' `spu/memory-transfer` and `spu/test`; synthetic voice programs against the references' audio, and against the tester's recordings if any (§11 Q16); the reverb against the references | 5 |
| **9** | **The MDEC** | ps1-tests' `mdec/4bit`, `8bit`, `frame` and `movie`, VRAM exactly; Peter Lemon's DCT decode | 2 |
| **10** | **The open BIOS, the rest** (§5.9): CD boot, files, the card functions in the real format, the screen without a disc | **Games boot with no firmware folder**; the cards they write are read by the same games on the player's BIOS and the reverse; `firmwarecheck.py` within the policy's limits; G9 | 6 |
| **11** | The PlayStation to players: the system pack's codecs (GameShark, Xploder), settings (region, model, pads, card mode), battery files, the two pad drawings, the CRT timing, `.m3u` and the disc change, the shelf, the kit on games | The kit's C1–C15 on the tester's discs (corpus form); cards round-trip; the fit audit with the PlayStation's tab, drawings and shelf at both sizes; the cheats decode (G4) | 4 |
| **12** | The debugger: the R3000A through the observer; registers including the GTE's; disassembly; breakpoints, stepping, coverage, the call stack | Each claim a test; every table armed gives the plain run's digests (C15, G7) | 2 |
| **13** | Speed (§5.12): the exact levers, in order, measured on the bench, under the 33% quota and on the handheld | §5.12's budget (G6) | 4 |
| **14** | The gates of §8, the goldens, the clone check | §8 | 2 |
| **E1** | The enhancement layer's frame: the observer trait on the R3000A and the GTE, the optional state block, the settings' category, the exactness harness | **Every accurate golden reproduced bit for bit with every enhancement compiled in and off**; states made with and without the layer load both ways; C3 and C9 pass | 2 |
| **E2** | The emulation enhancements: the CPU overclock and the faster CD | Each changes only what it states: off, E1's goldens; on, a bench of slowdown and loading measured; states tagged with their timing | 2 |
| **E3** | The Vulkan renderer at 1×, consuming the command stream, beside the software rasteriser | With no enhancement, its picture equals the software rasteriser's display on ps1-tests' GPU programs and the bench discs, exactly or within a stated count of pixels, each difference explained; E1's goldens unchanged (the renderer never writes machine state) | 4 |
| **E4** | The precision side-channel; sub-pixel vertices, perspective-correct texturing and the seam fixes in the renderer | A synthetic program that rotates a textured mesh slowly: vertex jitter and texture warp measured with and without; on the bench discs, the fraction of vertices resolved precisely, recorded per game; E1's goldens unchanged | 4 |
| **E5** | Internal resolution, true colour, texture filtering | Each on the bench discs at 2× and 4×, its cost measured on the desktop, the laptop and the handheld; framebuffer-as-texture effects checked on the bench | 3 |
| **E6** | The experimental two: depth from the precision path and the widescreen projection | Built only if the tester keeps them after E5 (§11 Q14); each with its breakage recorded on the bench | 2 |

About 69 steps, 52 for the accurate core and 17 for the enhancements, or 200–250 hours (P3). Stages 4 to 9 are independent of each other in part, beyond stage 3; stage 10
needs 6 and 7; the enhancements start after stage 11 and may run beside stages 12 to 14.

---

## 8. The gates

Diamond replaces nothing, so its gate is the one to **offer the system to players**, after stage 11, by the tester's
decision, as the Genesis was offered (`Nephrite_Plan.md` §7). The core carries the `InDevelopment` mark of
`EmuSen_CoreAPI.md` §27 from stage 1 until then.

- **G1, the processors.** 100% of the r3000 suite, state, delay slots and cycles, but cases named in a disputes entry;
  the GTE's `test-all` and amidog's GTE test bit for bit.
- **G2, the self-reporting programs.** ps1-tests' CPU, DMA, timer, GPU, MDEC and SPU programs; amidog's CPU, CPX and GPU
  tests; each failure a disputes entry.
- **G3, the pictures.** ps1-tests' VRAM exactly; on programs whose pictures stand still, the picture equal to the
  references' up to a colour map or a dispute.
- **G4, the frontend.** The kit's C1–C15 on the tester's discs; cards round-trip; the fit audit; the cheats decode.
- **G5, the goldens.** The bench discs (§11 Q16), each at three anchors, equal to the references in picture and RAM, or
  dispute-logged.
- **G6, speed.** §5.12's budget at 33% quota on the bench set, and full speed on the Legion Go S on battery.
- **G7, the debugger.** Breakpoints, stepping, watches, coverage and the call stack, each a test.
- **G8, the clean room.** `clonecheck.py` over Diamond's and the shared crates' sources against the reference
  emulators finds nothing above its threshold, the reference side never printed; the disputes log read end to end by
  someone who did not write it. (For the Genesis this review was skipped by decision, `Nephrite_Plan.md` §7; for
  Diamond it is the tester's again, §11 Q15.)
- **G9, firmware.** Every game the open BIOS is claimed to run runs with no firmware folder, its machine info naming
  the replacement; with the player's image, the same game runs on it.
- **GE, the enhancements.** Each enhancement offered only with its stage's gate met and E1's exactness test passing in
  CI on every change to the core.

---

## 9. Shared code

### 9.1 The MIPS processors

**What MarsRT's R4300i is, read from its tree.** The VR4300 is part of the `marsrt` crate (`src/cpu/`: the interpreter,
COP0, COP1 with soft floats, the TLB, segments, the idle loop, a block cache with a Cranelift recompiler), its `Cpu`
stepping against MarsRT's concrete `MemoryBus` rather than a trait [R `interp.rs`'s signatures], ported from C# Mars and
graded bit-identical against it (`Mars_Native.md`). It has no Rust disassembler [M: no `disasm` in MarsRT's source,
searched 2026-10-10], and MarsRT is not on the core ABI v1 (`EmuSen_CoreAPI.md` §28.6).

**What the two processors share and do not** [A, from the MIPS manuals' descriptions]:

| | R3000A (PlayStation) | VR4300 (N64) | Shared? |
|---|---|---|---|
| Encoding | MIPS I | MIPS III, a superset | **Yes**: the fields and the MIPS I opcode map |
| Width | 32-bit | 64-bit, 32-bit results sign-extended | No: every arithmetic rule differs in width |
| Load delay | Architectural: the loaded register is not yet written for the next instruction | None visible: the pipeline interlocks | No |
| COP0 | R3000-style: the three-level mode stack, RFE, no TLB on the PlayStation, cache isolation | R4000-style: EXL/ERL, ERET, a 32-entry TLB, Count/Compare | No |
| Coprocessors | COP2 is the GTE; no FPU | COP1 is the FPU; COP2 a latch | No |
| Caches | 4 KiB I-cache, scratchpad, no D-cache | 16 KiB I and 8 KiB D, write-back | No |
| Exceptions | R3000's vector and Cause codes | R4000's vectors, TLB refill | No |
| Disassembly text | MIPS I mnemonics, COP0 and GTE names | MIPS III, COP0, COP1 names | **Yes**, with a table per extension |
| An assembler (a development tool) | for test programs and the open BIOS's entry | for test programs | **Yes** |

**Recommended** [C, §11 Q10]:

- **`emusen-mips`**, a shared crate in `EmuSen/Cores/Shared/`: instruction fields, the decode of MIPS I to MIPS III into
  an instruction form with an extension point per coprocessor, the disassembler with extension tables (the GTE's,
  COP1's, and later the RSP's vector unit's), and the assembler. It contains no execution. Diamond builds it at stage 2;
  MarsRT may adopt the disassembler when it gains `DEBUG_DISASSEMBLE`, a change MarsRT's own owner decides and measures,
  not one this plan makes.
- **The R3000A as its own crate**, with Beryl's interface (a bus trait in the processor's terms, the bus owning the
  clock, the `Observer`, `StateWriter`, the disassembler through `emusen-mips`), because a second user is known: the
  PlayStation 2's I/O processor is an R3000A-family core, so Sapphire can use it [A]. **The GTE as a crate beside it**,
  since it has its own oracle and its own record. Where they live follows Beryl's rule: crates shared between the Sony
  cores in a Sony library folder, whose name is the tester's (§11 Q10).
- **Not shared**: the execution engines, the recompilers' front ends (each lowers its own semantics), the caches and the
  exception models. A shared interpreter "with flags for the differences" would be two interpreters in one file and
  would put MarsRT's measured bit-identity at risk for no gain.

### 9.2 Disc images

**Recommended** [C, §11 Q9]: **`emusen-disc`** in `EmuSen/Cores/Shared/`, one job: a disc as sectors.

- **In it**: the geometry (MSF and LBA, pregaps and indexes, track modes); raw 2,352-byte sectors, synthesised for a
  `.iso` with their sync, header, EDC and ECC from ECMA-130; mode 1 and mode 2 form 1 and form 2 decoding; the
  subchannel, synthesised (Q with its positions and CRC) or read (`.sub`, `.sbi` patches); the TOC; CD-DA as samples; the
  cue sheet and `.m3u` parsed (§5.11); CHD v5 read. Its interface is a trait: read sector *n* with its subchannel.
- **Not in it**: any console's drive controller (the PlayStation's HC05 protocol, the Sega CD's LC8951 and CDD, the
  Saturn's CD block), seek and read timing, XA-ADPCM decoding (the PlayStation's own), filesystems (the open BIOS reads
  ISO 9660 itself, as the kernel does).
- **Users**: Diamond at stage 7; Nephrite's paused stage 8, whose plan names the same needs (`.cue` with its tracks,
  `.chd` later, `Nephrite_Plan.md` §5.8); later Zoisite (Saturn), Kunzite (Dreamcast, for `.gdi` and `.chd`), Tiger's
  Eye (PC Engine CD) and Sapphire (CD and DVD).
- **CHD's decoders** (zlib, LZMA, FLAC, zstd): Nephrite's plan said "written without dependencies; later". The `chd`
  crate (0.3.4, BSD-3-Clause, compatible with GPL-3.0) [R crates.io] reads CHD in Rust; it is a port of a library that
  descends from MAME's code [A, from its repository's description, to be confirmed]. Used as a dependency, its code is
  not read, as Cranelift's is not; a container format is not emulation logic. §11 Q9 asks which.

### 9.3 The framework's shared pieces

- **`emusen-core-conform`**: unchanged, both forms, with the per-image memory estimate of §10.
- **`emusen-native`**: `core_exports!`, the descriptors, `StateWriter`/`StateReader`, `SampleQueue` for the 44.1 kHz
  output (no step synthesiser is needed: the SPU makes samples, not held levels), `Observer` and `Hooks` for the
  debugger, the settings schema.
- **The test-ROM runner** (`Nephrite_Native.md` §3), with Diamond's protocols added as data.
- **Vulkan**: the device of `EmuSen_LunaP_Kit.md` §5.2 for the enhanced renderer (§6.2); MarsRT's GPU path is the
  precedent for running the emulation's own Vulkan code beside the frontend's.

---

## 10. What the framework should gain, found by planning this core

*Each item is evaluated for its form, with the surface change it would make, in `EmuSen_CoreAPI_Extensions.md` (2026-10-10), which awaits the tester's decisions.*

- **Content-based routing** of extensions that several systems claim (`.cue`, `.chd`, `.iso`, `.bin`): a system pack
  sniffs the image (a disc's system area, a cartridge's header) and the host routes to the engine whose system matches.
  Needed before Diamond and Nephrite's Sega CD both claim `.cue` (stage 7).
- **A disc change**: an export to open the lid and insert another disc of a set (`.m3u`), in a later minor; the core
  models the lid already.
- **Rumble**: the output event v1.0 left open (`EmuSen_CoreAPI.md` §6.8).
- **An Analog button**: a `PadButton` appended, or a documented key-only control.
- **Conditional cheats**: a codec form for "if address A holds X, write Y at B", applied between frames by the runtime.
- **The kit's memory estimate per image**, from the image's size, for discs.
- **A shared memory card option** (one card for every game) in the runtime, if the tester wants one.

---

## 11. Questions for the tester

Each with a recommendation.

1. **Q1, Sony's confidential documents.** psx-spx already rests on them in part, by its maintainers' statement. May a
   writer read Sony's developer documents directly? *Recommended:* yes, each use cited by document and section, as
   Nephrite's Q1 allowed Sega's; the replacement writers' exclusions of §2.2 stand beside it.
2. **Q2, psx-spx's revision.** *Recommended:* the maintained conversion's `stable` branch pinned at `6d7d1bc`
   (2026-10-10), re-pinned only by a step that records why; nocash's 2022-08-20 original kept beside it for checks; the
   development branch, with its machine-written changes, not used.
3. **Q3, the BIOS.** *Recommended:* option B, Diamond's own open BIOS (§5.9), in two phases (stages 3 and 10), written
   in Rust for `mipsel-sony-psx` with an assembly entry, the binary committed with its source and rebuilt by CI to the
   same hash. OpenBIOS (option A) is not used, by its own account of its origin. Until stage 10, discs run on the
   player's BIOS only. *The choices for the tester:* B as recommended; B in assembly instead of Rust; C, a kernel
   answered in Rust (not recommended); or a different policy for the PlayStation.
4. **Q4, PSX_MiSTer.** Is a MiSTer device available? *Recommended:* the device if there is one, its picture and sound
   captured; otherwise GHDL simulation of bare-metal programs only, reading only its README, licence, directory names
   and the simulation folder's scripts and ports, never `rtl/`. Cloned at stage 4, not before.
5. **Q5, the reference emulators.** *Recommended:* Beetle PSX, PCSX-ReARMed and SwanStation through the probe, local
   black boxes only, never distributed, their sources never read. DuckStation, having no libretro core, is not used
   unless a measurement shows the three insufficient, in which case a probe backend for its released binary is a
   separate decision.
6. **Q6, a BIOS of the tester's own.** Beetle PSX refuses to run without one, the CD path before stage 10 needs one,
   and the open BIOS is best compared against one. Does the tester have a dump from a console of their own?
   *Recommended:* if so, kept in `~/.cache/emusen/probe/diamond/firmware/` outside the repository, passed to the
   references by `--sysdir` and to Diamond as the player's image, never read, disassembled or traced by anyone, and
   used by `firmwarecheck.py` only. If not, none is obtained: the project never fetches firmware.
7. **Q7, the test programs' licences.** amidog's are CC BY-NC-SA 3.0 and Peter Lemon's state none. *Recommended:* all
   used as local instruments under `~/.cache/emusen/probe/diamond/`, never committed, as Nephrite's corpus is.
8. **Q8, the CPU suite's generator.** The r3000 suite comes from an emulator's interpreter. *Recommended:* a primary
   oracle all the same, with amidog's CPU tests and ps1-tests as independent checks, every disagreement a dispute (as
   Nephrite's Q3).
9. **Q9, discs.** *Recommended:* PS-EXE at stage 1; `.cue`/`.bin`, `.iso`, `.chd` and `.sbi` at stage 7; `.m3u` at
   stage 11; `.ccd`/`.sub` and `.ecm` later; `.pbp` later and unencrypted only. The shared `emusen-disc` crate (§9.2),
   which parses the cue sheet itself, refining Nephrite's Q5 so that one parser exists; content-based routing in the
   framework (§10); and CHD through the `chd` crate as an unread dependency rather than four decoders written anew.
10. **Q10, the shared crates.** *Recommended:* `emusen-mips` in `Cores/Shared/` (decode, disassembly, assembly; no
    execution); the R3000A and the GTE as crates of a Sony library folder named in the Black Moon Clan's manner, as
    Beryl is for the Sega cores. A name to choose: *proposed* `BlackMoon-HW`, crates `blackmoon-r3000` and
    `blackmoon-gte`. (Crystal is the name of a retired assembly and of the planned runtime; the Ayakashi Sisters are kept
    for the PS4 and PS5, `EmuSen_Core_Naming_Scheme.md` §4.)
11. **Q11, the default console.** *Recommended:* a model with the 208-pin GPU (the later revisions), the region from
    the disc, a DualShock in digital mode at power-on in port 1 (it answers as a digital pad until switched, so no game
    is refused), with the 160-pin GPU, the region and the pads as settings.
12. **Q12, peripherals in scope for the player gate.** *Recommended:* the digital pad, the DualShock, the memory cards
    and the multitap; the mouse, NeGcon, light guns, link cable and PocketStation after it, as device kinds arrive.
13. **Q13, the speed budget.** *Recommended:* Nephrite's rule (§5.12), the exact levers first, the accuracy settings
    only after them.
14. **Q14, the enhancements.** (a) *A name:* "PGXP" names an existing implementation; EmuSen's feature should carry its
    own. *Proposed:* "Precise geometry" in the settings, `precision` as the layer's crate. (b) *Which to build:*
    *recommended* all of §6.3 but depth and widescreen as planned, those two kept experimental (stage E6) and built only
    if E5's results justify them. (c) *Where the renderer runs before the kit's device exists:* *recommended* on the
    device MarsRT's GPU path creates, moved to the kit's when it exists.
15. **Q15, the clean-room review.** *Recommended:* the clone check and the disputes reading at G8, since Diamond's
    sources include an open BIOS where derivation matters most. The tester skipped them for the Genesis; it is asked
    again here rather than assumed.
16. **Q16, what the plan needs from the tester.**
    - *Discs:* the library has no PlayStation games [M]. Dumps of discs the tester owns, as `.cue`/`.bin` (or `.chd`),
      under `AppSettings.RomDirectory`'s `psx/`, read only and copied to scratch for every use. *Recommended bench*,
      chosen for what each loads: Crash Bandicoot (CPU, scratchpad), Ridge Racer (early 3D, CD-DA), Final Fantasy VII
      (MDEC movies, mixed 2D and 3D, three discs), Metal Gear Solid (XA streaming, two discs), Gran Turismo 2 (GPU load,
      PAL and NTSC differ), Castlevania: Symphony of the Night (2D, hi-res menus), Tekken 3 (interlaced hi-res), Ape
      Escape (DualShock-only), Silent Hill (dithering, fog), Wipeout 2097 (CD-DA), and **one LibCrypt-protected PAL
      disc** with its subchannel, and **one Japanese disc**. Any subset the tester owns serves.
    - *Recordings, if possible:* the console's audio output on a few synthetic SPU programs, which would make the SPU's
      oracle a console rather than the references.
    - *A BIOS dump* of the tester's own console (Q6), if one exists.
    - *Sony's own diagnostic programs*, if the tester has any locally; never fetched (as Nephrite's Q14).

---

## 12. Negative results

- No GTE cases in the r3000 single-step suite (its caveats, §3.1); no AccuracyCoin-like suite known.
- No DuckStation libretro core on the buildbot (§1.6).
- No public transcription of any PlayStation chip, so the FPGA referee is no stronger than its author's reading (§1.5).
- No public document of the CD controller's microcontroller program; its behaviour is known only by its commands'
  descriptions and by measurement (§5.6).
- OpenBIOS is not clean-room by its own account (§5.9).
- Nothing was measured on the handheld or the laptop; the factors are VenusRT's and Nephrite's.

## 13. Predictions to be retired

- **P1.** The R3000A passes every case of the r3000 suite's registers before any bus exists, and its cycle lists on
  all but a handful of cases that become disputes about fetch ordering.
- **P2.** A heavy 3D scene misses §5.12's budget with the interpreter alone and meets it with the threaded rasteriser
  and the decoded-instruction cache, before the recompiler.
- **P3.** The whole is about 69 steps (52 and 17); the GPU's timing, the CD-ROM controller and the open BIOS are the stages most
  likely to overrun.
- **P4.** DMA's timing needs at least one dispute, about chopping or the GPU channel's pacing.
- **P5.** At least one bench game behaves differently when the GPU's drawing time is made instant.
- **P6.** At least half the disputes entries fall in §2.3's thin areas, and most are the CD-ROM controller's and the
  CPU's bus timing.
- **P7.** The open BIOS's documented surface takes about 6 steps after the kernel's core, and at least one bench game
  needs a kernel behaviour no document describes.
- **P8.** The GTE passes `test-all` and amidog's test bit for bit within its stage, with no more than two disputes.
- **P9.** With every enhancement off, the build reproduces every accurate golden from E1 on, with no exception.
- **P10.** The precision path resolves most projected vertices on 3D bench games and fewer than half on games that
  build geometry on the CPU.
- **P11.** D-0 is settled in favour of one clock for every NTSC model the tester can measure.
- **P12.** The handheld runs the accurate core at full speed on battery after stage 13, and 2× internal resolution
  with the enhanced renderer beside it.
