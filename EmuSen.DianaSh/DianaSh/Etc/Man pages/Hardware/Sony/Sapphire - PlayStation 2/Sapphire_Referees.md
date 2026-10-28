# Sapphire_Referees — the PlayStation 2 referees, run as black boxes

*Written 2026-10-24, at stage S3* (`Sapphire_Plan.md` §7, on the tester's go of the same day). No PlayStation 2 console
is used (`Sapphire_Plan.md` §1.5); where the documents and ps2autotests' console-recorded texts do not reach, the
evidence is the outputs of emulators run as black boxes, weighed as `Diamond_Referees.md` §1 and §3 weigh Diamond's.
This page records which referees were installed and how each is driven headless (§2), how their agreement is weighed
(§3), the project program every referee that runs one was graded on (§4), what this stage's writer was exposed to
(§5), the evaluation of a hardware context for the probe (§6) and of DobieStation (§7), the plan's predictions this
stage tested (§8), and what was read (§9). The driver is `EmuSen.WiseMan/Reference/analysis/ps2referee.py`.

Marks as in the plan: **[M]** measured on 2026-10-24 on the desktop (Ryzen 7 7700X, RX 6800, Fedora 44) with the tool
named; **[R]** read from the source named; **[A]** argued; **[C]** a choice.

---

## 1. What the referees are for

An emulator's output is its authors' reading of the hardware, not the hardware [A, as `Diamond_Referees.md` §1]. For
the PlayStation 2 there is no FPGA core (`Sapphire_Native.md` §12.4) and no console in the project, so the referees and
ps2autotests' `.expected` texts are the only outputs to compare with. A referee is graded where it can be: against
the expected texts (from S4 on, P2) and against the project's own programs, whose results the architecture fixes
(§4). Neither makes one a grader.

## 2. The referees

| Referee | Form and version | Binary, pinned | Licence [R] | BIOS | Driven by | Cost [M] | RAM it exposes | Lineage |
|---|---|---|---|---|---|---|---|---|
| **PCSX2** | Qt application `pcsx2-qt`, **v2.8.2** (release of 2026-09-04) | `pcsx2-v2.8.2-linux-appimage-x64-Qt.AppImage`, 60,119,544 bytes, SHA-256 `0c46bb6a…` (equal to GitHub's own digest) | GPL-3.0 | the player's, by folder and name | §2.1: a private KWin, the software renderer, PINE | done word 0.70–0.85 s after launch, a run 1.11–1.27 s; **2.56 ms a frame** unlimited (2.65 ms CPU), 4.4 ms CPU a frame at the console's speed | **PINE**: reads and writes of 8–64 bits at the EE's addresses; the title, serial, CRC and status | PCSX2 |
| **LRPS2** (`pcsx2_libretro`) | libretro core; reports itself as **`parallel-ps2 v2.0.0-38d3a83`** | buildbot nightly of 2026-10-24, the `.so` 13,584,968 bytes, SHA-256 `05d64500…` | GPLv3 | the player's: refuses without one ("Could not find any valid PS2 BIOS File") | §2.3: the probe, `Software (SW)` | 0.44 s for 60 frames; **0.28 ms a frame** | the probe's `ram`, 33,570,816 bytes from address 0 | PCSX2 (a hard fork) |
| **PCEE2** (`pcee2_libretro`) | libretro core; reports **`pcee2 v2.9.138`** (its `.info` says v2.7.x) | the same nightly, 42,205,472 bytes, SHA-256 `00275dbf…` | GPLv3 | the player's: without one it runs but exposes no RAM | §2.3: the probe, `software` | 0.86–0.93 s for 60 frames; **2.31 ms a frame**, drawing 640×448 | `ram` 33,554,432 bytes; memory maps at 00000000h (32 MiB) and 70000000h (16 KiB) | PCSX2 (a port of upstream) |
| **Play!** | Qt application `Play`, **0.72** (commit `8de4a71f`, 2026-01-23) | `Play!-8de4a71f-x86_64.AppImage`, 52,587,000 bytes, SHA-256 `5523586c…` | MIT [R its `.info`] | none: its own high-level BIOS [R README] | §2.2: a private KWin, `--elf`, its process's memory scanned | done word 0.68–0.70 s, a run 1.03–1.10 s; at the console's speed only, **0.4 ms CPU a frame** | none of its own; §2.2's scan | Play! |
| **`play_libretro`** | libretro core; reports **`play! 0.77-16-g505dac7`** | the same nightly, 7,592,664 bytes, SHA-256 `45fc9028…` | MIT | none | §2.3 | refused: asks for OpenGL core 3.2, runs nothing, and crashes on unload | `ram` 67,108,864 bytes, empty | Play! |
| DobieStation | — | not installed | GPL-3.0 | the player's | — | — | — | its own, tied to ps2tek (§7) |

The standalone releases are pinned in `EmuSen.WiseMan/Reference/ps2-referees.sha256` with their URLs (PCSX2's tagged
GitHub release, not a rolling build; Play!'s from purei.org's `downloads/play/stable/0.72/`), and the cores in
`libretro-cores.sha256` beside Diamond's. `ps2referee.py fetch` downloads, verifies and unpacks the standalone ones into
`~/.cache/emusen/probe/sapphire/referees/`; `build-probe.sh libretro-core <core>` fetches each core; the probe is
built from `probe-rs` into `~/.cache/emusen/probe/sapphire/probe/`. Nothing system-wide, no sudo; nothing fetched or
captured is committed. `pidof EmuSen EmuSen.Mistress` answered nothing before the installs [M].

**The firmware** [M]. PCSX2, LRPS2 and PCEE2 are given the cache's folder `system/pcsx2/bios/`, which holds only three
links to the tester's images in `~/Documents/BIOS/PS2/`: `ps2-0230a-20080220.bin` (Americas),
`ps2-0230e-20080220.bin` (Europe) and `ps2-0200j-20040614.bin` (Japan, `Sapphire_Native.md` §12.7). No tool of the
project opens, hashes or copies them; the driver refuses a run if any `.bin` in the folder is not a link. The
referees write their own console data beside the links: PCSX2 a 4-byte `.mec` and a 1,024-byte `.nvm` per image,
"creating default" when absent, and the libretro cores their settings tree under `system/pcsx2/`; nothing was written
under `~/Documents/BIOS` (`find -newer` after the runs, empty). PCSX2's run is refused unless its log says it loaded the
market's image (`BIOS Found: USA     v02.30`, `Europe  v02.30`, `Japan   v02.00`); LRPS2 and PCEE2 are given the image
by their option `pcsx2_bios`, which lists the folder's three names, and a probe run is refused unless the done word is
in RAM. **Negative controls**: with an empty BIOS folder LRPS2 refuses the program and PCEE2 runs 60 frames exposing no
RAM, so neither can produce a record without the firmware.

### 2.1 PCSX2, headless

- **A display that is not the person's.** The AppImage's Qt carries the `wayland` and `xcb` platforms and no
  `offscreen` [M: Qt's own list when started without a display]. It runs under a private KWin as DuckStation does
  (`Diamond_Referees.md` §2.1): `kwin_wayland --virtual` with its own socket and message bus, here with `--xwayland`,
  and PCSX2 on the `xcb` platform; `DISPLAY` and `WAYLAND_DISPLAY` are removed from the environment first. The whole
  runs in a transient user scope stopped when the result is written, since KWin's own exit takes about ten seconds
  more [M: 11.2 s a run before, 1.2 s after]; no portal or secret service was left running by any run [M: every such
  process found was the desktop's own, started 2026-10-23].
- **Settings**, in its own data folder (`-datapath`): no setup wizard, no shutdown prompt, `Renderer = 13`, which is
  the software renderer [M: a failed start's message names "the chosen renderer (Software)"; the software renderer
  presents through Vulkan on the RX 6800; under lavapipe (`VK_DRIVER_FILES` set to `lvp_icd`, `LIBGL_ALWAYS_SOFTWARE=1`)
  it never reports a swap chain, so the driver never connects, even for D-780's program that passed before (M 2026-10-28,
  D-1103); a referee run therefore takes the RX 6800], a `Null` audio stream, fast boot (an ELF is entered without the menu),
  patches off, PINE on at a free slot. Its command line is `pcsx2-qt -datapath D -batch -nogui [-unlimited] -logfile L --
  <elf>` [R pcsx2.net "Command Line Options"].
- **PINE** [R the protocol's draft standard, `pine/standard/draft.dtd`]: a Unix socket named `pcsx2.sock.<slot>` in
  `XDG_RUNTIME_DIR`; each message a little-endian 32-bit size that counts itself, an opcode and its arguments; each
  answer the size, a result byte (0 OK, FFh FAIL) and the value. Read32 is opcode 2, the version 8. **The draft's
  numbering past opcode 10 is off by one** [M]: PCSX2 answers the title at 11, the serial at 12, the CRC at 13, the
  game's version at 14 and the status at 15 (0, running). Addresses are the EE's: 00200000h reads RAM. PINE answers
  FAIL until the machine runs; the driver treats that as "not yet". **PINE does not reach the EE's I/O registers**
  [M 2026-10-26, `Sapphire_Native.md` §30.53]: Read32 at timer 0's, timer 1's and timer 3's registers and at I_STAT and
  I_MASK (10000000h to 1000F010h) answered OK with 0 in each of 2,358 reads over 55 s of the BIOS's menu, I_MASK among
  them, which the kernel has set by then; so PINE is no witness for a register, and a register is read on the referee
  by a program of the project's that reads it itself.
- **A defect of the run, not of the machine** [M]: a PINE client that connects before PCSX2's GS has opened makes it
  fail to acquire its render window ("Failed to acquire render window", then "Failed to initialize GS"), with or
  without `-batch` and `-nogui`. The driver therefore waits until the log reports the swap chain before it connects;
  with that, `-batch -nogui` runs cleanly.
- **Outputs**: RAM through PINE, read when the done word appears (`--until`); the log is read only for the BIOS line
  and the swap chain, never printed or kept (§5). `--again S` reads the words again S seconds later with the CPU time
  PCSX2 spent meanwhile, from `/proc/<pid>/stat`.
- **A printing program's text** (*added 2026-10-24 at S4*, `Sapphire_Native.md` §14.5): `ps2referee.py tty` switches
  one of PCSX2's log channels on in its settings, `EnableEEConsole` or `EnableIOPConsole` under `[Logging]` (the names
  PCSX2 writes into its own settings file), with timestamps off, waits until the expected text's last line reaches the
  log, and returns only the lines from the first that ends with the expected text's first line to the last that ends
  with its last. The rest of the log, which holds the BIOS's output, is read by the driver and never printed or kept, so
  the run exposes nobody to it. A project program writing to KPUTCHAR shows that the EE's console carries KPUTCHAR.
- **A disc in the drive** (*added 2026-10-25 at S23*, `Sapphire_Native.md` §25.3): `ps2referee.py ram pcsx2 … --disc DISC`
  gives PCSX2 `-elf PROGRAM -- DISC`, so the program runs in the disc's place with the disc in the drive. A raw `.bin`
  works; a `.cue` beside it never ran the program. PCSX2's lineage maps the CDVD's registers at BF402000h for the EE, so
  an EE program can drive the drive's N and S commands; Play! answers CCh there.
- **Cost** [M, `programs/frames.s`, §4.2]: unlimited, 3,910 frames in 10.0 s (391 a second, 2.56 ms each) for 10.36 s of
  CPU; at the console's speed, 600 frames in 10 s for 2.66 s of CPU. Start-up: the GS draws 0.50–0.60 s after launch and
  the record is in RAM 0.20–0.25 s later. These are an idle program's costs; a game's are §5's.

- **A game through the BIOS** (*added 2026-10-26 at S28*, `Sapphire_Native.md` §39.2): `ps2referee.py game pcsx2 OUTDIR
  SCRIPT --disc DISC` boots the disc with fast boot off at the console's speed, a card file or none in slot 1 and an
  EEPROM file (`--nvm`, `--keep-nvm` to keep the one a run through the initial setup leaves), plays a script of key
  presses, pictures from states and the game's RAM from 00100000h (`SECONDS ram NAME`), read through PINE in batch
  messages, one message of 4,096 Read64 commands answered by one reply (31 MiB in under a second; separate messages sent
  back to back are not all answered). The kernel's memory below 00100000h is never read.

### 2.2 Play!, standalone

- **Run** in the same private KWin, `Play --elf <elf>` [R README], on `xcb` (its only bundled platform), with `HOME` and
  the XDG folders pointed at the cache's `play-home/`, where it keeps `Play Data Files/`. Play! takes no BIOS. It has
  no documented command-line option for its speed, so it runs at the console's speed [M: 600 frames counted in 10 s].
- **RAM has no interface**, so it is found in Play!'s own process, which the driver started (so `/proc/<pid>/mem` is
  readable as a debugger's is): the done word is searched for in the process's writable mappings, and a place counts as
  the EE's RAM only where **the program's own code sits where it would in RAM** relative to it (the ELF's segment at
  00100000h, 1 MiB before the done word). Every address is then read relative to that place.
- **Why the anchor** [M]: the done word was found in two to six places a run, all but one in a mapping of about 4.5 MiB
  that also holds the word as a constant [A: a recompiler folds `lui` and `ori` into it]; a first version of the driver
  that took the first place read garbage on one run of four. The RAM's place lay 0xFEFF0 bytes into a mapping that
  begins 1 MiB and one page past RAM's start, a read-only page between [A: the program's code page write-protected, as a
  recompiler watching for self-modifying code would; Play!'s source is not read to confirm it]. The driver reads across
  such pages and refuses a read of anything unmapped.
- **Cost** [M]: at the console's speed, 600 frames for 0.24–0.25 s of CPU; the record is in RAM 0.68–0.70 s after
  launch.
- **Its kernel** (`Sapphire_Plan.md` §11 Q5): Play!'s high-level BIOS only loads the program here; the record is
  computed by the EE with no kernel call (§4), so it is a program's output, never BIOS evidence.

### 2.3 The libretro cores, through the probe

- **Run** as Diamond's cores are (`Diamond_Referees.md` §2.3): `probe <elf> <out> N N --core <core> --system ps2`, with
  `--sysdir` (the cache's `system/`, whose `pcsx2/bios/` is §2's folder) and `pcsx2_bios=<image>` for LRPS2 and PCEE2,
  in a user scope of their own. RAM is the probe's dump of `RETRO_MEMORY_SYSTEM_RAM` at frame N; the words asked for are
  read from it and every dump is removed unread, since on the player's BIOS low RAM holds the kernel.
- **The hardware-render refusal, measured.** The probe answers `SET_HW_RENDER` false, as before; since this stage it
  prints one line naming the context and version asked for (`probe-rs/src/backends/libretro.rs`, `hw_refusal`):
  - **LRPS2** with its default renderer `Auto` asks in turn for OpenGL core 3.3, OpenGL 3.0, OpenGL ES 3.1 and Vulkan
    (its version field holding 4198400, Vulkan's packed 1.1.0), then prints "No usable hardware context; using the
    software renderer." and runs. With `Software (SW)` pinned it asks for nothing. **It hands the frontend no picture**
    by frame 600; RAM is complete.
  - **PCEE2** with its default `vulkan` asks for Vulkan 1.1, is refused, and runs; with `software` pinned it asks for
    nothing and **hands 640×448 XRGB8888 pictures**, as its `.info` says the software renderer does.
  - **`play_libretro`** asks for OpenGL core 3.2, is refused, and **runs nothing**: its 64 MiB `ram` is all but empty at
    frames 60 and 600, and it crashes with SIGSEGV when unloaded. It needs `libbz2.so.1.0`, which Fedora names
    `libbz2.so.1`, so the driver gives it a link in the cache's `lib/` through `LD_LIBRARY_PATH`.
- **PCEE2's resources folder** is not needed for an ELF [M: every PCEE2 run here had none]; the `.info` says the core
  will not boot without it.
- **Cost** [M, `frames.s`]: LRPS2 0.44 s for 60 frames and 0.59 s for 600, so 0.28 ms a frame and about 0.4 s to start;
  PCEE2 0.93 s and 2.18 s, so 2.31 ms a frame, drawing in software, and about 0.8 s to start.

## 3. Independence

| Lineage | Referees | Counted as |
|---|---|---|
| PCSX2 | PCSX2 2.8.2, LRPS2 (`parallel-ps2` 2.0.0, a hard fork of an older PCSX2), PCEE2 (2.9.138, a port of upstream) | **one witness** where they agree [`Sapphire_Plan.md` §1.6]; where they differ, the difference is an older or newer state of the same code |
| Play! | Play! 0.72 standalone; `play_libretro` 0.77 (not running here) | one witness, independent of PCSX2's |
| DobieStation | not run (§7) | would be a third, tied to ps2tek |

A ruling needs agreement between lineages, or else names the strongest single witness as such, as `Diamond_Referees.md`
§3 rules [C, carried over].

*Added 2026-10-25 at S12*, the weights found per unit since S6 (`Sapphire_Native.md` §19.6, §20.8, §21.7): **neither
lineage is a witness for the FPU's or the VUs' arithmetic** (both read exponent 255 as IEEE 754's and clamp it, raise no U
or O, and compute the VU on IEEE 754's rounding towards zero, PCSX2's FPU alone modelling the console's division), nor
for the exceptions the referees never raise; PCSX2 is a witness for the integer, multimedia and DMAC units the console
texts exercise, for COP2's transfers, the VU's integer and random units, CLIP, MAX and MINI. The console texts are
weighed first for every FPU and VU dispute. Play!'s kernel output is excluded from every ruling about the BIOS (`Sapphire_Plan.md` §11
Q5).

*Added 2026-10-25 at S16* (`Sapphire_Native.md` §24.7): **PCSX2 is a witness for the GS's local memory** (every format's
layout at a page's start, the transfers' offsets, restart and end, local to local without overlap) **and its registers**
(CSR, SIGNAL's stall, LABEL's and SIGNAL's masks, the reset), departing at a depth format based inside a page (D-105),
X's wrap (D-106), an overlapping copy (D-107) and the interrupt's edges (D-103). Play! is one for the colour and texture
formats' layouts and their local-to-host transfer, which carries §6's form C, and for the wrap; it moves no depth format,
performs no local-to-local transfer, and of the registers keeps CSR's events and SIGLBLID but neither IDMSK, the stall nor
the reset.

*Added 2026-10-25 at S17* (`Sapphire_Native.md` §27.7): **PCSX2 is a witness for the GS's coverage rules** (points,
sprites, triangles' sides, lines' interiors and the corners their columns cross), the vertex queue, the contexts,
scissoring and SCANMSK, every frame and Z format with FBMSK, every depth test with ZMSK and ZTE 0, and Z's clamp; not
for interpolation's last step (Gouraud colour and Z depart by a step either way near whole values, D-160, D-164) nor
for line ends on a diamond's edge (D-161). **Play! is no witness for drawing**: its GS draws through a GPU renderer, and
its local-to-host transfer, which reads transferred data exactly, reads every drawn picture back as it was before the
drawing; form C (§6) therefore carries transfers and not drawing, and a second lineage for S17 to S19 needs form A, the
reserve kept at S3.

*Added 2026-10-25 at S18* (`Sapphire_Native.md` §31.7): **PCSX2 is a witness for texturing's addressing and tables**:
every texture format, the CLUT buffer under every CLD, CSA, CSM and CPSM case, TEXA, the texture functions with TCC
(D-240), the wrap modes and their regions at levels (D-245), MTBA (D-246), level choice by a fixed LOD, and fog's formula
(D-248); **not for the filters' and the LOD's arithmetic**: it rounds bilinear filtering after each stage (D-249), holds
the LOD to eight fractional bits and ignores Q with UV (D-242), takes MMAG at a LOD of 1/16 (D-241), leaves blue and
alpha unblended between mip levels in two pixel columns of every four (D-250), and crosses texel boundaries within
single precision's reach (D-247).

*Added 2026-10-25 at S27* (`Sapphire_Native.md` §34.11): **PCSX2 is a witness for the IPU's register behaviour and its
handling of the bit stream** (the FIFO's depth and filling, BCLR, FB, VDEC's and FDEC's values but D-308's lengths, the
pointer and the flags at every slice's end, CBP, INT_IPU), **for non-intra and MPEG-1 decoding** within its IDCT's one
step, and **for every PACK word**; not for intra output's range (D-301), the alternate scan's weights (D-303), IDEC's
scale (D-304), CSC's last step (D-319) or IDP's width (D-316), each of which a stated rule reproduces word for word.
**Play! is a witness for VDEC's output and IPU_TOP** (D-308, D-311) and leaves the pointer short at a slice's end
(D-305), after which its replays are out of step. A third lineage stands behind the corpus itself: **ffmpeg's decoding,
run as a black box, matches the project's model of H.262 within one step**, which the IPU's stage takes as the witness
for the model's tables and arithmetic.

## 4. The project's programs

### 4.1 The gate: `programs/record.s`

A bare EE program in `EmuSen/Cores/Sony/Sapphire - PlayStation 2/programs/`, assembled by `emusen-mips` (MIPS I, as
Diamond's programs are) at 00100000h and written as an ELF by `examples/elf.rs` through the core's `media::program`, the
writer the crate's tests use. It calls no kernel, computes twelve words whose values the architecture fixes, writes
them from 00200000h and then **5A99D00Eh to 001FFFF0h**. Loaded values are used only after a `nop`, so no load delay is
relied on. `tests/programs.rs` checks that every program assembles into an ELF the core reads and that the record ends
with its done word.

| Word | What | Expected |
|---|---|---|
| 0 | `addu` of 7FFFFFFFh and 1, which wraps without a trap | 80000000 |
| 1, 2 | `sra` and `srl` of it by 4 | F8000000, 08000000 |
| 3, 4 | `sltu` and `slt` of 1 against it | 00000001, 00000000 |
| 5, 6 | `mult` of 7FFFFFFFh by itself: LO, HI | 00000001, 3FFFFFFF |
| 7, 8 | `div` of −7 by 2: quotient, remainder | FFFFFFFD, FFFFFFFF |
| 9 | a loop summing 100 down to 1 | 000013BA |
| 10 | an `addiu` in a taken branch's delay slot (runs) and one after it (skipped) | 00000001 |
| 11 | `lb`, `lbu` and `lhu` of a word with one byte stored over it, combined | EEDDBB80 |

**The record, read from every referee that runs it** [M, `ps2referee.py ram <referee> record.elf --until
1FFFF0=5A99D00E 1FFFF0:1 200000:12`]:

| Referee | Firmware | Done word | The twelve words |
|---|---|---|---|
| PCSX2 2.8.2 | Americas, Europe, Japan (three runs) | 5A99D00E | 80000000 F8000000 08000000 00000001 00000000 00000001 3FFFFFFF FFFFFFFD FFFFFFFF 000013BA 00000001 EEDDBB80 |
| LRPS2 | Americas | 5A99D00E | the same |
| PCEE2 | Americas | 5A99D00E | the same |
| Play! 0.72 | its own | 5A99D00E | the same (three runs) |
| `play_libretro` | — | absent | refused (§2.3) |

Both lineages agree with the expected values on every word. The program tests the record path, not the EE: these
results are ones no emulator that boots anything gets wrong. **Reservation** [A]: the record is written through the
EE's data cache, and every referee here reads RAM as if writes went straight through it; a referee that modelled the
cache's write-back could show stale RAM until a line is evicted, and Sapphire's own reader will read through the core's
view of memory.

### 4.2 The frame counter: `programs/frames.s`

Writes the done word at once, then counts the GS's vertical syncs at 00200000h by polling CSR's VSINT bit (bit 3 of
12001000h) and writing it back to clear it [ps2tek names CSR at 12001000h and no bit of it; the bit's role is
general knowledge of the GS, A, confirmed by the measurement below]. With `--again` it measures a
referee's frames per wall second and CPU per frame (§2). It counted 600 frames in 10 s on every referee held to the
console's speed, and 589–591 by frame 600 on the probe (the BIOS's boot takes the first nine to eleven) [M].

### 4.3 IOP programs, through an EE program (2026-10-25, stage S21)

`ps2referee.py` runs EE programs only, and needs nothing more for the IOP: an EE program places the IOP program and
reads its record (`Sapphire_Native.md` §22.9). It copies the IOP program into IOP RAM through the EE's window at
BC000000h, writes a jump to it at the IOP's general exception vector (IOP RAM 80h), so that the IOP's next interrupt
under the BIOS's kernel enters it, and copies its done word and record back into EE RAM, where `ram` reads them as for
any program. Nothing of the BIOS is read, traced or patched; the vector is the processor's. **PCSX2** runs them (PRId
1Fh, the OTC list, the flags, the prescalers equal to the documents; no IOP cache, wait states, bus errors or undefined
bits modelled; a fetch from the IOP's scratchpad stops it). **Play!** cannot: its EE reads CCCCCCCCh at BC000000h and
drops stores there, and its IOP is its own high-level BIOS. LRPS2 and PCEE2 are PCSX2's lineage. The IOP therefore has
one referee lineage.

### 4.4 Paired programs on the SIF (2026-10-25, stage S22)

S22's programs are two at once, an EE program and the IOP program it places by §4.3's route, stepping together
through a mailbox of two words in IOP RAM that the EE reaches through its window, so that neither the handshake nor
the record depends on the SIF they test (`Sapphire_Native.md` §26.7). `examples/s22.rs` writes each as one ELF and
`ps2referee.py ram` reads the whole record from EE RAM as for any program, the EE having copied the IOP's record after
its own. **PCSX2** runs them; **Play!** cannot, for §4.3's reason. PCSX2 is a witness for the mailbox registers, the
two-word EE tag of SIF0 and the quadword IOP tag of SIF1, and for nothing on the FIFOs' depth, which it does not hold,
or on SIF2, which it does not move (`Sapphire_Disputes.md` D-140 to D-145). A disagreement that leaves words in a FIFO
spoils every later part of a program on the referee, so each program puts the part PCSX2 is known to stop on last.

### 4.5 SIO2 programs, with the referee's cards and multitap (2026-10-25, stage S25)

S25's programs are IOP programs placed by §4.3's route that talk to SIO2's devices (`Sapphire_Native.md` §30.8).
`ps2referee.py ram pcsx2 … --cards A,B` puts the cards named A and B, files or folders in PCSX2's memory card folder, in
its two slots for one run, and `--multitap` plugs a multitap into its first port; both are put back in its settings
afterwards, so that no other run meets them. The project's cards there are its own erased files
(`sapphire-s25.ps2`, `sapphire-s25b.ps2`) and an empty folder card (`sapphire-s25-folder.ps2`); PCSX2's own
`Mcd001.ps2` and `Mcd002.ps2`, which every other run keeps in its slots, are never written. **PCSX2** answers through
its pads, cards and multitap with no input given, so a pressed button cannot be graded there; **Play!** cannot run an
IOP program. PCSX2 makes a new card with every byte FFh and formats none without a BIOS, so a referee's formatted card
waits for a BIOS (part B). *Part B, 2026-10-25* (`Sapphire_Native.md` §30.18): `ps2referee.py pads pcsx2 OUTDIR SCRIPT
--card CARD` boots the BIOS with no disc at the console's speed with a copy of CARD in slot 1 (`--card none` leaves the
slot empty) and plays SCRIPT, keys of PCSX2's keyboard pad at set seconds after it begins to draw and pictures from
states saved through PINE, each kept as a PNG and the state removed; the card file as PCSX2 left it is copied out, the
black box's output, and the console's EEPROM, which a script through the initial setup changes, is put back as it was
(the first runs, before that, left it set up; it was removed on 2026-10-25 so that PCSX2 made its default again, and a
run then showed the language screen as before). XTEST's events sent to the private compositor's Xwayland never reached PCSX2 (measured 2026-10-25);
key events sent with `XSendEvent` to its largest window, given the input focus first, do. Only pictures and the card
were kept; the log, which holds the BIOS's output, is read only for the BIOS line. With it the player's BIOS formatted
a card in its browser and read the crate's edits back. *2026-10-26, D-383* (`Sapphire_Native.md` §30.25): `--cards`
takes `none` for a slot left empty, its `Slot<n>_Enable` set false for the run and put back afterwards; with it PCSX2
showed that every queue ends 25 IOP cycles after its start, so it is no witness for SIO2's time. *2026-10-26, D-540*
(`Sapphire_Native.md` §30.33): PCSX2 ends SIO2out after every queue whatever its blocks, and is a witness for that end,
its flag and the reply's place, not for the bytes past the replies, since it clocks the whole of FIFOIN; it raises
SIO2's interrupt again only once the flag at 1F808280h was written back (D-541), so the project's probes acknowledge it.
*2026-10-26, D-620* (`Sapphire_Native.md` §30.40): with the player's BIOS stepped to its browser, PCSX2's EE RAM read
through PINE at the EE's SIF receive address showed the SIF RPC calls the BIOS makes there, each END packet's documented
fields; the buffer is overwritten by each packet, so PCSX2 is a witness for which servers are called and roughly how
often, not for every call. The reading code was a scratch copy of the tool, never committed, as `Sapphire_Plan.md` §1.4
asks of a trace.
*2026-10-26, D-640* (`Sapphire_Native.md` §30.46): PCSX2's pad trace log (`EmuCore/TraceLog`, `IOP.pad`), allowed by the
tester for that step, prints in the 2.8.2 release build no command or reply byte, only one line as each pad's
configuration finishes (the LED, the Analog button, the motors, the reply bytes' set), so PCSX2 is a witness for the
state a driver leaves the pad in and when, not for the exchange. With trace logging on, its `-logfile` copy stops after
the start-up lines and the rest goes to the default log under `logs/`, which also carries the IOP modules'
registrations and recompiler messages: a reader keeps the `Pad: ` lines only. The referee's settings have no pad in
port 2; a run with one there drew the browser as before.
*2026-10-26, D-760* (`Sapphire_Native.md` §30.62): PCSX2's two EE tiers part on an MFC0 straight after EI or DI, the
recompiler reading the Status from before it, the interpreter the Status after it; `d700-exec`'s ExecPS2 follows the
same split. A program whose record touches COP0 or the pipeline is run on both tiers (`--interpreter`, which `pads`
also takes since this step), and a split is a dispute, not a grade. The interpreter runs OSDSYS below the console's
speed, so a `pads` script for it allows the setup more time.
*2026-10-26, D-800* (`Sapphire_Native.md` §30.73): the two tiers part again, on when a pending interrupt is taken. A
tight loop polling I_STAT sees V-blank's start before the interrupt at every arrival on the interpreter and at none on
the recompiler. On the interpreter, a pending interrupt is not taken within sixteen instructions after EI, which
reads as PCSX2's grain of testing interrupts rather than a console's delay. Neither tier is a witness for the delay's
size.

### 4.6 The SPU2's sound (2026-10-25, stage S26)

PCSX2's SPU2 can be heard two ways (`Sapphire_Native.md` §32.8). **The output areas**: PCSX2 writes each sample into
sound RAM's first 2000h halfwords (each core's voices 1 and 3, core 0's output, every voice sum), which an IOP program
reads through its DMA channel into its record; these are exact samples, compared under rotation with `examples/s26.rs`'s
`grade` and `diff` (`Sapphire_Disputes.md` D-261). **The captured sound**: `ps2referee.py ram … --throttled --wave DIR`
gives PCSX2 its SDL backend with SDL 3's disk driver, which writes the output as 32-bit float stereo at 48 kHz to
`DIR/pcsx2.raw`; that output has passed PCSX2's own output stage (a high-pass, about half the level, D-270), so it shows
routing and level and not samples. The settings return to the null backend on the next run without `--wave`. Play!
reaches no IOP program (§4.3), so the SPU2 has one lineage.

### 4.7 Replay programs on the IPU (2026-10-25, stage S27)

An IPU program carries a command log and its stream image behind its code, and replays the log through the IPU's
registers with toIPU and fromIPU moving the data, recording IPU_CMD, IPU_CTRL without its FIFO counts (which depend on
the referee's DMA pace), IPU_BP's pointer, the command's EE cycles by Count and its output (`Sapphire_Native.md`
§34.10); `examples/s27.rs explain` reads a referee's record by command and counts the output's lanes by their
difference, so that a hypothesis about a departure can be built into a scratch copy of the IPU and compared word for
word.

## 5. Exposure, and a retail disc

- **This stage's writer is exposed** (`Sapphire_Plan.md` §1.4): while finding why PCSX2 failed to draw, the writer read
  PCSX2's log of a run, which holds the BIOS's own output: the IOP's modules registering by name and version, and
  traces of the BIOS's register reads and writes during boot. **S3's writer never writes the open replacement.** The
  driver keeps no log: it reads the log for the BIOS line and the swap chain inside the run's temporary folder, which is
  removed, and `boot` prints only PINE's answers.
- **Jak and Daxter, once** [M, `ps2referee.py boot pcsx2 <the test copy> --for 60`, Americas BIOS, at the console's
  speed]: PINE answered the title "Jak and Daxter - The Precursor Legacy", the serial SCUS-97124 and the status running
  after 58 s; PCSX2 spent 73.3 s of CPU in 57.4 s, about **21 ms of CPU a frame** if it held 59.94 frames a second,
  which was not measured. Nothing of the disc was read by the project, and no picture or RAM was kept.

## 6. A hardware context for the probe: an evaluation, not built (`Sapphire_Plan.md` §11 Q4, §10)

**What S3 found** [M, §2.3]: the PCSX2 family's two cores do not need one. Both fall back to their software renderers
without it, both give a complete RAM record, and PCEE2 hands the frontend software-rendered pictures. Only
`play_libretro` needs one, and Play!'s standalone form already gives that lineage's RAM. What a context would add is
**Play!'s pictures** (G3 and G5 need a second lineage's pictures) and LRPS2's, which PCEE2's already stand for.

| Form | What the probe gains | Size [A] | Evidence it would give | Cost and risk [A] |
|---|---|---|---|---|
| **A. Headless EGL with OpenGL core** | an EGL display on the RX 6800's render node (surfaceless), an OpenGL core context and a framebuffer; `SET_HW_RENDER` answered for OpenGL, `get_current_framebuffer` and `get_proc_address` given, `context_reset` called after load, the framebuffer read back on each valid frame | 300–400 lines of Rust; EGL and GL loaded at run time, so the build stays header-only | `play_libretro`'s RAM and pictures; LRPS2's OpenGL pictures | a GPU test in the probe's harness (one at a time, RX 6800 only); a hardware renderer's pictures differ between drivers, so they are weaker evidence than a software renderer's |
| **B. libretro's Vulkan interface** | instance and device creation, the negotiation interface, queue sharing, the image handed over with its synchronisation | 800–1,200 lines | PCEE2's Vulkan and LRPS2's paraLLEl-GS pictures | the largest, and the validation layer's synchronisation checks with it; nothing it adds is not given by A or C |
| **C. None** (recommended) | nothing | 0 | Play!'s pictures as RAM: the project's GS programs transfer the drawn rectangle from the GS's local memory back to the EE (`TRXDIR` 1, "VRAM->GIF", with `BITBLTBUF` and `BUSDIR`, R ps2tek's GS registers) and record it, as Diamond's GPU programs read VRAM back through GP0(C0h) so that no referee need expose VRAM (`Diamond_Referees.md` §2.4) | depends on each referee modelling the local-to-host transfer, which is itself a thing to grade; if Play! does not, A is the fallback |

**Recommended: C** [C], with A held in reserve for the first GS dispute whose ruling needs Play!'s picture and finds its
readback wanting. PCSX2's standalone form with PINE suffices for that family, and the probe already runs its cores, so
the plan's condition for building a context (S3 showing the standalone forms insufficient) is not met.

## 7. DobieStation

Evaluated for a headless form [R its README, GitHub's API, 2026-10-24]: a Qt 5 application, last pushed 2021-07-25, with
**no releases** on GitHub and no CI artifacts still held, so it exists only as source; its command line is `-b <bios>`,
`-f <elf or iso>` and `-s` (skip the BIOS animation), with no documented headless mode, no RAM interface (no PINE, no GDB
server) and no frame or speed option. Building it needs Qt 5's development files and CMake, neither installed [M,
`rpm -q`]; compiling reads nothing, but the build would be the project's own, not a pinned release. It could be driven
as Play! is (the private compositor and §2.2's scan). It is written by ps2tek's author, so where it agrees with ps2tek
it is not independent of it. **Recommended** [C]: not installed now; built user-locally only if a dispute needs a third
lineage that ps2tek does not decide.

## 8. The plan's statements tested

- **§1.6's first finding** ("none of the three PlayStation 2 libretro cores can draw through it as it stands") is
  **half right**: Play!'s core cannot run at all without a context; the PCSX2 family's two run on their software
  renderers, and PCEE2 draws.
- **§1.6's table**: PCEE2 is not "experimental" in its behaviour here and is 2.9.138, not 2.7.x; its `resources` folder is
  not needed for an ELF; Play!'s standalone form expects CHD for discs but takes an ELF directly.
- **§11 Q4**: PCSX2 standalone with PINE works headless as recommended; Play! standalone works with §2.2's scan in place
  of an interface.
- **P1** (D-0's clocks) is not tested here; `frames.s` confirms only that every referee's vertical sync is at the console's
  59.94 Hz [M].

## 9. What was read for this page

`Sapphire_Plan.md` §1.6, §3.3, §5.9, §7, §10 and §11 Q3–Q5; `Sapphire_References.md`; `Sapphire_Native.md`;
`Diamond_Referees.md` §1–§3; `psxreferee.py`, `build-probe.sh` and the probe's libretro backend (the project's own). Of
the referees, only: pcsx2.net's "Command Line Options" (read 2026-10-24); the PINE repository's README and its draft
standard `standard/draft.dtd` (2026-10-24); Play!'s README and purei.org's downloads page; DobieStation's README; ps2tek's GS register list and `TRXDIR`; the three
cores' `.info` files from the buildbot's `info.zip`; each program's own log and Qt's own messages. No source of PCSX2,
LRPS2, PCEE2, Play!, DobieStation or PINE's client library, and no ps2sdk or loader source, was read. The BIOS images
were linked, never opened.

---

**CHD and chdman, credited (2026-10-25).** CHD (Compressed Hunks of Data) is MAME's image format, and chdman is MAME's
tool for making, checking and extracting it. chdman's source file is under the BSD-3-Clause licence, copyright Aaron
Giles; MAME as a whole is distributed under the GNU General Public License, version 2 or later ("The MAME License",
MAME Documentation 0.289, <https://docs.mamedev.org/license.html>). chdman's commands, codecs and defaults are
documented in "chdman – CHD (Compressed Hunks of Data) File Manager", MAME Documentation 0.289,
<https://docs.mamedev.org/tools/chdman.html>, read 2026-10-25; every chdman run the project records was of release
0.289. EmuSen reads CHD through the `chd` crate (BSD-3-Clause, Ronny Chan; `THIRD_PARTY_NOTICES.md`), whose source is
not read, and MAME's source is never read (`EmuSen_Disc.md` §1, §8).
