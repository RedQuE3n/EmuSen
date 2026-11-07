# Diamond_Referees — the disputes refereed by emulators run as black boxes

*Written 2026-10-11, after stage 4.* Decided 2026-10-11: no PlayStation console is used for any dispute, now or later.
The evidence is the console logs and pictures that ship with the test suites (`Diamond_References.md` §1) and, where
those do not reach, reference emulators run as black boxes, their outputs only. This page records which referees were
fetched and how each is driven headless (§2), how their agreement is weighed (§3), what each dispute's program showed
on every one of them and the ruling it supports (§4), what changed in Diamond (§5), how the referees score against
the console's own logs (§6), and what was read (§7). The entries themselves stay in `Diamond_Disputes.md` (and D-2 in
`../BlackMoon-HW/BlackMoon_R3000.md` §8), each pointing here.

Marks as in the plan: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from the source named; **[A]**
argued; **[C]** a choice.

---

## 1. Why emulators, and what they are worth

The order of recourse in `Diamond_Disputes.md` was the documents, a test program, then PSX_MiSTer run as a black box.
No console is used and no VHDL simulator is installed (`Diamond_References.md` §5), so the third rung is now the
emulators of this page. **An emulator's output is its authors' reading of the hardware, not the hardware** [A]. Two
things raise it above a guess: where an emulator reproduces a console log that a suite ships, it has been checked
against the console on that point (§6), and where emulators of separate authorship agree exactly on a detail no
document gives, the detail is unlikely to be any one author's invention. Neither makes an emulator a grader; the
console logs and pictures remain the only measurements of hardware the project holds.

## 2. The referees

| Referee | Version | Binary | Licence [R] | Lineage [R its README] | Driven by |
|---|---|---|---|---|---|
| **DuckStation** | `v0.1-11894` (release of 2026-09-12, commit `c66b2694d`) | `DuckStation-x64.AppImage`, 92,725,536 bytes, MD5 `16086adc…`, SHA-256 `c2fd2625…` | CC BY-NC-ND 4.0, unmodified redistribution permitted | its own | §2.1 |
| **PCSX-Redux** | `9272f440` (dev channel build 396, 2026-10-11) | `PCSX-Redux-9272f440-linux-x86_64.zip`, 91,016,315 bytes, MD5 `fe0ec971…`, SHA-256 `18c1b39e…`; inside it the AppImage, MD5 `494d00d3…` | GPL-2.0 [R GitHub API] | the PCSX family | §2.2 |
| **Beetle PSX** | 0.9.44.1 `10e4b0c` | the libretro core of `Diamond_References.md` §4 | GPLv2 | Mednafen | §2.3 |
| **SwanStation** | 1.0.0 `dd48056` | the same | GPLv3 | "a hard fork … of DuckStation" | §2.3 |
| **PCSX-ReARMed** | r26 `c881679` | the same | GPLv2 | "based on the PCSX-Reloaded project"; on x86-64 its GPU is "NEON GPU by Exophase … x86 SSE2+" | §2.3 |
| no$psx | — | not fetched | — | — | wine is not installed, so it was not considered further [M] |

The two standalone releases are pinned by SHA-256 in `EmuSen.WiseMan/Reference/psx-referees.sha256` with their URLs,
as the libretro cores are in `libretro-cores.sha256`, and `EmuSen.WiseMan/Reference/analysis/psxreferee.py fetch`
fetches, verifies, unpacks and prepares them in `~/.cache/emusen/probe/diamond/referees/` [M]. DuckStation's release
is GitHub's tagged `v0.1-11894`, not the rolling `latest`, so that the pin names a fixed file. PCSX-Redux publishes
no GitHub release: its README links its Linux build to `distrib.app`, whose public manifest
(`/storage/manifests/pcsx-redux/dev-linux-x64/manifest-396.json`) gives the zip's path, its MD5 and SHA-1, and the
changeset [R]. Everything fetched, unpacked and captured stays in the cache; nothing is committed.

**The firmware** [M]. Every run was given the player's own BIOS: DuckStation and the libretro cores by their BIOS
folder set to the probe's firmware folder, PCSX-Redux by its `-bios` flag naming the file there. Nothing read it.
PCSX-Redux falls back to its own OpenBIOS only for a BIOS set in its configuration, never for one given by `-bios`,
which is an error if missing [R its "Command Line Flags" page]; the driver refuses any PCSX-Redux run whose log lacks
"Loaded BIOS", and any DuckStation run whose log lacks "Using BIOS" (DuckStation needs a BIOS to start [R its
README]); the probe's refusal of Beetle PSX and SwanStation without dumps (`Diamond_References.md` §3) stands.
PCSX-ReARMed ran on its own high-level kernel, as before.

### 2.1 DuckStation, headless

DuckStation's Linux release is a Qt application. Run headless it needs three things [M]:

- **A display that is not the person's.** The release carries no `offscreen` Qt platform, and its GPU device cannot be
  created on that platform anyway ("Unknown PNI platform offscreen"). It is run instead under a **private KWin
  compositor on a virtual framebuffer** (`kwin_wayland --virtual`, with its own socket, its own message bus from
  `dbus-run-session`, its own configuration folder, and `DISPLAY` and `WAYLAND_DISPLAY` removed from the environment),
  which exits when DuckStation does; DuckStation draws to that, never to the desktop. No Xvfb is installed.
- **Qt from the system.** The bundled Qt 6.11.2 libraries are replaced in the unpacked copy by links to Fedora's Qt
  6.11.2, whose platform plugins they then use; `LD_LIBRARY_PATH` cannot be used, since DuckStation stops on a dialog
  warning about it. Its own `libpng16` is preloaded, since the system's is otherwise loaded first through the system's
  Qt and its images fail. These are changes to a local unpacked copy, never redistributed.
- **Settings**, in a portable `settings.ini` next to the binary (`portable.txt`): the interpreter, the software
  renderer, unlimited speed, no audio, no update check, no on-screen messages, the BIOS folder, TTY logging with every
  log channel but TTY and System off, and the GDB server on port 2345. The region is `Auto` (an EXE boots as NTSC-U) or
  `PAL`. Its command line is `duckstation-qt -batch -nogui -- <exe>`; `-exe` alone stops on a dialog.

Outputs: the **TTY** from its log's `I/TTY:` lines, and **RAM** through its GDB server, a few dozen lines of the
GDB remote protocol in the driver (a status query halts it, `m` packets read RAM). It runs unthrottled, so a run is
a number of wall seconds, after which RAM is read and the process ended.

### 2.2 PCSX-Redux, headless

PCSX-Redux has a text mode: `-cli` runs it "without the graphical user interface, using a text interface instead",
with its log and the TTY on standard output [R "Command Line Flags"]. It is run as `pcsx-redux -portable . -cli -run
-interpreter -bios <file> -exe <exe> -exec <lua>`, with `DISPLAY` and `WAYLAND_DISPLAY` removed [M]. The Lua is a
`GPU::Vsync` listener that counts frames, prints the RAM words asked for from `PCSX.getMemPtr()` at the frame given
and calls `PCSX.quit()` [R "Events", "Memory and registers", "Redux basic API"]. Frames count from power-on, so a
side-loaded program starts about 300 frames in, after the BIOS's boot. Its TTY lines are kept from the program's
heading on; its own complaints about unmapped reads are dropped.

PCSX-Redux keeps its own clock coarser than the console's: a 1,000-cycle delay loop of `timers` reads 2,023 cycles
and a NOP costs two [M, §4.2, §4.5], so its cycle counts are not compared with the console's except as patterns.

### 2.3 The libretro cores

Through the reference probe as before (`Diamond_References.md` §3), RAM read at a frame, with three options pinned
where a default would hide timing: SwanStation's `CPU_ExecutionMode=Interpreter` and `GPU_Renderer=Software`,
PCSX-ReARMed's `drc=disabled`; Beetle PSX's recompiler is off by default. For a PAL console, each core's region
option (`beetle_psx_region=pal`, `swanstation_Console_Region=PAL`, `pcsx_rearmed_region=PAL`) [M]. No libretro
channel carries the TTY (`Diamond_Native.md` §11.15), so these cores are witnesses of RAM-recording programs only.

### 2.4 The programs

Each dispute's program is one of Diamond's `programs/*.s`, assembled with `emusen-mips` and run with no kernel on
Diamond and side-loaded on the referees; each writes its results from 80100000h and D1A0D00Eh to 800FFFF0h when done,
and every referee was checked to have written that word. The GPU programs read VRAM back through GP0(C0h) into RAM, so
that no referee need expose VRAM. The ps1-tests programs were run for their TTY on DuckStation and PCSX-Redux, and on
Diamond on its open kernel. Diamond's figures below are its own runs (`examples/run`), pinned by `tests/programs.rs`.
*Since 2026-10-11* five of ps1-tests' TTY programs (`timers`, `cpu/access-time`, `dma/chopping`, `gpu/bandwidth`,
`dma/chain-looping`) also exist rewritten to record in RAM, `programs/timers.s` and `programs/records/`, so that every
referee is graded on them (§4.10; `Diamond_Records.md`); *and since a second step of the same day* three more,
`dma/otc-test`, `dma/dpcr` and `cpu/io-access-bitwidth`, as records of their printf calls (§4.11).

### 2.5 Each run confined, and DuckStation's runs one at a time

*Fixed 2026-10-11.* A DuckStation run's private message bus activates the portal and secret services
(`xdg-desktop-portal-kde`, `ksecretd`) as children of its `dbus-daemon`; when the bus ends they are re-parented to
systemd's per-user manager and stay, one pair a run [M: the driver as it was, run once, left `xdg-desktop-portal-kde`
and `ksecretd` with that manager as their parent; two dozen such pairs from earlier runs were found running]. The driver now
starts every referee, the libretro probe and PCSX-Redux included, in a transient user scope of its own
(`systemd-run --user --scope`, unit `emusen-referee-<pid>-<referee>`) and stops the scope when the run returns, which
ends whatever the run started wherever it was re-parented; after each of the rewrites' runs no process was left in any
such scope and none of those services was new [M]. Two other faults of a shared machine were closed with it: DuckStation's
runs take a lock on the cache's `duckstation.lock`, since they share one `settings.ini`, and each picks a free port for
its GDB server, since with the fixed 2345 a run could read the RAM of another DuckStation started meanwhile [C, M: other
runs were seen to start during this step's]. The pairs left by earlier runs were not stopped here; they are the
tester's to stop.

## 3. Independence

| Lineage | Referees | Counted as |
|---|---|---|
| DuckStation | DuckStation, SwanStation | one witness: SwanStation is a fork, and it agrees with DuckStation wherever both model a thing [M]; where it does not (the multiplier, §4.2), it is an older state of the same code |
| PCSX | PCSX-Redux, PCSX-ReARMed | one family, but two witnesses where their outputs differ in kind; ReARMed's x86-64 GPU is Exophase's, whose authorship its README separates from the rest of PCSX |
| Mednafen | Beetle PSX | one witness |

A ruling needs **agreement between lineages**, or else it names **the strongest single witness** as such, the
strength measured by how the witness scores against the console logs on the same subsystem (§6). Where the referees
disagree, the disagreement is recorded, and a majority is never taken for a ruling by itself [C, the rule of this
page]. None of the READMEs read says whether any renderer was written from another's; agreement between the GPU
renderers of DuckStation, Beetle PSX and PCSX-ReARMed is counted as three witnesses with that reservation [A].

## 4. The disputes

### 4.1 D-0: the line length, and the interlaced fields — `video-timing.s`

Interrupts off, I_STAT's vblank polled, each edge stamped with timer 2 on the system clock (unwrapped by the poll) and
timer 1 counting hblanks; 4 settling edges, then 41 stamps in each of six GP1(08h) modes. Mean CPU cycles of the 40
fields, and the hblanks a field [M]:

| Referee, NTSC console | NTSC 320 | PAL 320 | NTSC interlaced | PAL interlaced | Line, video clocks |
|---|---|---|---|---|---|
| Diamond before §5 | 566,120.7, 263 | 674,415.2, 314 | 565,044.6, 262/263 | **673,341.6, 313/314** | 3,412.50, 3,405.00 |
| Diamond after §5 | 566,120.9, 263 | 674,415.2, 314 | 565,044.6, 262/263 | 671,193.9, 312/313 | 3,412.50, 3,405.00 |
| Beetle PSX | 566,121.6, 263 | 674,416.2, 314 | 565,045.0, 262/263 | 671,194.1, 312/313 | 3,412.50, 3,405.00 |
| DuckStation | 566,203.8, 263 | 674,613.5, 314 | 566,204.1, 263 | 674,613.1, 314 | 3,413.00, 3,406.00 |
| SwanStation | 566,212.3, 263 | 674,623.8, 314 | 566,212.3, 263 | 674,623.8, 314 | 3,413.05, 3,406.05 |
| PCSX-Redux | 564,406.8, 263 | 673,854.0, 314 | 564,406.8, 263 | 673,854.6, 314 | 60.008 Hz flat |
| PCSX-ReARMed | 564,479.9, 263 | 564,480.2, 263 | the same | the same | 60.000 Hz, PAL mode ignored |

The 480-line modes (bits 5 and 2) read as the 240-line interlaced ones on every referee [M]. As a **PAL console**, in
NTSC mode: Diamond 571,332.5, Beetle 571,334.4, DuckStation 571,415.7, SwanStation 571,424.7; in PAL mode 680,623.6,
680,626.0, 680,823.3 and 680,834.1; PAL interlaced, Beetle 677,375.0 (50.0001 Hz) and Diamond after §5 677,372.2
(50.0003 Hz); PCSX-Redux and PCSX-ReARMed do not change with the region [M]. ps1-tests' `timers` on DuckStation as a
PAL console reads its system-clock frame rows at 112,641 against the console log's 112,499–112,557, its ÷8 row at
71,423 against 71,407–71,412, and its dot-clock frame row at 112,197 against 112,025–112,034, the count of an NTSC
video clock [M].

**What it shows** [A]. The referees divide exactly as the documents do (`Diamond_Disputes.md` D-0): Beetle PSX takes
3,412.5 and 3,405, the lengths psx-spx's maintained rates imply; DuckStation and its fork take nocash's 3,413 and
3,406; the PCSX family models no line length at all. The majority is not the ruling; the console log is. On the one
log that times a frame, a PAL-clocked console in NTSC mode, the second step of D-0 unwrapped 571,324 ± 6 cycles a
frame. Beetle PSX's figure is 10 cycles (18 ppm) from it, DuckStation's 92 (161 ppm), and DuckStation's dot clock on
a PAL console runs at the NTSC rate, 167 dots a frame more than the log's [M]. **The strongest witness for the line
is Beetle PSX**, the one referee that reproduces the console's log, and it agrees with Diamond on both line lengths
and on NTSC's interlaced fields. Its PAL ratio, 3,405 / 3,412.5 = 0.997802, is also the crystal-free ratio of the
public report D-0 read. The line stays at **3,412.5 and 3,405 video clocks**, now witnessed rather than only argued;
how the half clock is spread over the lines no referee output shows.

**The interlaced PAL field changes.** Diamond had alternated 314 and 313 lines, 313.5 on average and 49.84 Hz on a
PAL console, against psx-spx's 312.5 lines and 50.000 Hz, against D-0's own conclusion (50.0003 Hz), and against
Beetle PSX's 313 and 312 in the same phase; DuckStation does not lengthen or shorten an interlaced field at all, so
it is no witness here [M]. Defect proven, fixed (§5).

### 4.2 D-2: the multiplier — `multiply.s`

Cycles over the MFLO-alone baseline, MFLO straight after the operation: MULTU and MULT by rs class (below 800h, below
100000h, above), DIV and DIVU; then MULTU of a small rs with N NOPs before MFLO, N = 0, 1, 2 … [M]:

| Referee | Classes, division | N = 0, 1, 2 … |
|---|---|---|
| Diamond | 6, 9, 13; 36 | 6 5 4 3 2 1, then 1 from N = 5 |
| DuckStation | 5, 8, 12; 35 | 5 4 3 2 1, then 1 from N = 4 |
| Beetle PSX | 8, 11, 15; 38 | 8 7 6 5 4 3, then 1 from N = 6 |
| PCSX-Redux | 8, 11, 15; 38 | 8 6 4 2, then 2 from N = 3 (a NOP costs it two) |
| SwanStation | 1 everywhere, recompiler or interpreter | 1 |
| PCSX-ReARMed | recompiler 0; interpreter 7 to 16 with no rs classes, 36 to 38 | 7 5 4 2 2 … |

**What it shows** [A]. Three lineages (DuckStation, Mednafen, PCSX-Redux) agree on the **shape** of psx-spx's table:
the class steps of +3 and +4 and division 30 above the smallest class, the same for MULT by rs's magnitude, rt
irrelevant. That part is confirmed by independent agreement. They disagree on the **offset**, where the latency is
counted from: one cycle less than Diamond (DuckStation), two more (Beetle PSX, and PCSX-Redux in its own clock), and
on the boundary, MFLO free after 4, 5 or 6 instructions. Beetle PSX's step from 3 to 1 between N = 5 and 6 says a
stall costs it two cycles more than the wait. No console log times the multiplier. **Still open** for the offset and
the boundary; the referees differ by up to three cycles and no output ranks them.

### 4.3 D-4: the access times — ps1-tests' `cpu/access-time`

The console's table against DuckStation's and PCSX-Redux's TTY [M] (cycles a load, 8/16/32 bits, as each prints them):

| Region | Console | Diamond | DuckStation | PCSX-Redux |
|---|---|---|---|---|
| RAM | 5.21, 5.3, 5.14 | within 0.20 | 7.1, 7.1, 7.5 | 3.2, 3.2, 3.2 |
| BIOS | 7.6, 12.94, 24.94 | within 0.20 | 7.9, 13.5, 25.1 | 3.2 each |
| EXPANSION2 | 10.99, 25.99, 55.98 | within 0.20 | 1.4, 1.0, 1.0 | 3.0 each |
| CDROM_STAT | 8.0, 14.0, 25.93 | within 0.20 | 7.0, 13.0, 25.0 | 3.0 each |
| SPUCNT | 17.99, 17.99, **38.94** | within 0.20, **36.07** | 21.4, 21.0, 82.4 | 3.0, 3.0, 6.0 |

**What it shows** [A]. Neither referee models memory control as the console times it: of the 51 cells, DuckStation
meets ~~11 within 0.2 cycles (25 within 0.5) and PCSX-Redux 11~~ *(recounted 2026-10-11, the hundredths read as
printed, §4.10)* 36 within 0.2 cycles and PCSX-Redux 24, mostly the internal registers' three cycles, which PCSX-Redux
charges for every access [M]. The open SPUCNT 32-bit
cell (an address error's round trip through the kernel) reads 82.4 and 6.0 on them, 38.94 on the console. **Still
open** for that cell and for register values other than the kernel's; the rule stays measured on the console log, and
no referee bears on it.

### 4.4 D-5: the DMA's gaps and a store's drain — `dma/chopping`, `dma/chain-looping`, `stores.s`

`chopping`'s 130 rows within 10% of the console: Diamond 129, DuckStation 17, PCSX-Redux 0. `chain-looping`, the
console 16,040 / 16,032 / 25,632 / 25,640 cycles with the two never-ending chains unfinished; Diamond 16,032 / 16,016 /
24,024 / 24,024; DuckStation 13,016 / 13,024 / 143,024 / 143,024; PCSX-Redux 16,016 four times with every chain
reported finished [M]. `stores.s` times N stores to RAM (A), N stores and a RAM load (B), N stores and eight NOPs (C),
N = 1 to 8, the closing timer read itself a load [M]:

| Referee | A | B | C |
|---|---|---|---|
| Diamond | 8N | 8N + 5 | 9, then 8N |
| DuckStation, SwanStation | N | N + 7 | N + 8 |
| Beetle PSX | N + 1 | N + 7 | N + 9 |
| PCSX-Redux | 3N | 3N + 3 | 3N + 16 |
| PCSX-ReARMed | 0 (recompiler); 1, 3, 5, 7, 8, 10, 12, 14 (interpreter) | | |

**What it shows** [A]. No referee has a write queue that a load waits for, and none reproduces the console's
`chain-looping` or `chopping` cycles. **Still open**: Diamond's 10-cycle gaps and 8-cycle drain remain fits to the two
console logs, and the referees do not divide them.

### 4.5 D-6: timer sync mode 2 — ps1-tests' `timers`

The console reads mode 2 as zero outside the blank and counting within it (36, 159, 282, 405, 528, then 0 at 320
pixels); Diamond reproduces the rows. DuckStation counts freely in mode 2 (242, 369, 496 … or 258, 385 …, never
zero); PCSX-Redux reads zero always, within the blank too [M]. Its frame rows also show PCSX-Redux at 262 hblanks a
frame where the console reads 263 [M]. **Confirmed on the console log**, which stays the evidence: neither referee
models mode 2 as the console behaves, so neither bears on the rule.

*2026-10-11, `timer-sync.s`* (`Diamond_Disputes.md` D-6's second part): a mode write with the sync on, read a few
cycles later, k × 8,192 loops after a vblank, for timers 0 and 1 in each sync mode [M]. Every referee's two reads are
at most 15 in all 32 cases, the first DuckStation's 0 or 1, SwanStation's 0, 1 or 5, PCSX-ReARMed's 2, PCSX-Redux's 0
or 3 and Beetle PSX's 0 or 4, where Diamond's timer 0 had read up to 46,876; after the fix Diamond's first reads are 0
or 3. The lineages
agree that nothing before the write reaches the count after it; they differ, as above, on which modes pause or wait.

### 4.6 D-7: where the interpolation starts — `raster-order.s`

One shaded triangle, A (2,1), B (40,9), C (13,30), in its six vertex orders; a shaded quad in the four orders that
keep its diagonal from the second vertex to the third; and a triangle with two vertices in its leftmost column, A
(2,1), B (40,9), C (2,30), in its six orders; dithering off, VRAM read back [M]. Pixels differing from DuckStation's
picture of the same shape:

| Referee | Triangle, orders ABC ACB BAC BCA CAB CBA | Quad, four orders | Tied triangle, six orders |
|---|---|---|---|
| Diamond, from vertex 0 | 0 0 2 2 0 0 | 2 0 3 4 | 0 2 0 2 2 0 |
| Diamond, from the leftmost vertex (§5) | 0 0 0 0 0 0 | 0 0 0 0 | 0 0 0 0 0 0 |
| Beetle PSX, SwanStation, PCSX-ReARMed | 0 everywhere: the four pictures identical | | |
| PCSX-Redux | 1 1 1 1 1 1 | 3 3 2 1 | 2 2 0 0 0 0 |

DuckStation, Beetle PSX, SwanStation and PCSX-ReARMed draw all sixteen shapes word for word alike (FNV-1a
`DD743D1026ABB444` over the read-back), and each draws the first triangle and the quad the same in every order. On the
tied triangle their pictures fall into two groups, ABC/BCA/CAB and ACB/BAC/CBA: the winding, which in the command's
order means **the tied vertex that follows the other**, 0 following 2 [M]. Of the bases tried on Diamond (vertex 0;
the leftmost with ties to the lower y, to the higher y, or to the earlier vertex; the topmost), only "the leftmost
vertex, of two the follower" gives every shape exactly; the topmost is off on the quads [M].

**What it shows** [A]. Three renderers of separate authorship by their READMEs agree exactly with each other, and
none starts from vertex 0. Diamond's vertex 0 had been chosen because it met ps1-tests' pictures exactly, and those
pictures cannot tell the start apart where the leftmost and vertex 0 coincide (D-7). The leftmost rule meets the same
pictures: **ps1-tests' GPU gate stays exact on every picture with it** [M, `Ps1_tests_gpu_programs_draw_their_vram_
pictures_exactly`], while the topmost start, tried in the same gate, leaves `triangle` 3,227 pixels off, as D-7
recorded. **Changed**, on independent agreement (§5). PCSX-Redux's renderer differs from all four by a pixel or
three a shape and depends on no order; it is not counted.

### 4.7 D-8: the x-flipped rectangle's start — `xflip.s`

A texture whose texel holds 8000h + V·100h + U, drawn raw as rectangles two lines high: x-flipped at even and odd
columns with U = 0, 1, 10, 11; y-flipped at V = 0 and 1; both; one pixel wide [M]. First texels (U of row 0):

| Case | Diamond before §5 | Beetle PSX | DuckStation, SwanStation, PCSX-Redux, PCSX-ReARMed |
|---|---|---|---|
| x-flip, U 0, even and odd column | 1, 0, 255 … | 1, 0, 255 … | 0, 1, 2 …: no flip |
| x-flip, U 1 | **2**, 1, 0 … | **1**, 0, 255 … | 1, 2, 3 … |
| x-flip, U 10 | 11, 10, 9 … | 11, 10, 9 … | 10, 11 … |
| x-flip, U 11 | **12**, 11, 10 … | **11**, 10, 9 … | 11, 12 … |
| y-flip, V 0 / V 1 | rows V, V−1 | rows V, V−1 | no flip |

**What it shows** [A]. Only Beetle PSX flips at all; the other four lineages ignore both flip bits. Beetle PSX
agrees with ps1-tests' console picture `texture-flip` (U 0 at even columns, U+1 first) and with Diamond there; with
an odd U it starts at U itself, **U with its low bit set**, at even and odd columns alike, where Diamond had started
at U+1. The start does not depend on the column [M]. psx-spx's remark that a rectangle's "texture UV, as well as the
texture width must be even" [R, D-8] is consistent with a texel pair being the unit, which U | 1 implies and U+1 does
not. **Changed, on the strongest single witness**, Beetle PSX, the only referee that flips and one that matches the
console's picture where the picture reaches (§5).

### 4.8 D-10: GPUSTAT bit 28 while polygons and lines arrive — `gpustat-shapes.s`

GPUSTAT's bits 26 and 28 (as 26/28) idle, after a polygon's command word, after each vertex, 300 cycles into the
drawing and after; the same for a line and a rectangle; then a triangle with a second queued behind it [M]:

| Referee | Triangle | Line | Rectangle | Queued pair |
|---|---|---|---|---|
| Diamond, DuckStation, SwanStation | 11 01 01 01 01 01 11 | 01 01 01 01 11 | 01 01 01 01 11 | 00 00 11 |
| Beetle PSX | 11 **00 00 00** 01 01 11 | **00 00** 01 01 11 | 01 01 01 01 11 | 00 00 11 |
| PCSX-Redux | 11 11 11 11 11 11 11 | 11 … | 11 … | 11 11 11 |
| PCSX-ReARMed | 11 11 11 11 01 11 11 | 11 11 01 11 11 | 11 11 11 01 11 | 01 11 11 |

Against `gpu/gpustat`'s console log, the twelve readings of `gpustat.s`, the referees score: Diamond 12, DuckStation
and SwanStation 9, Beetle PSX 6, PCSX-Redux 4, PCSX-ReARMed 3 [M].

**What it shows** [A]. The lineages divide: DuckStation keeps bit 28 set while a polygon's or a line's vertices
arrive, as Diamond does; Beetle PSX clears it from the command word to the last vertex, which is psx-spx's sentence
for those commands. Both keep it set while a rectangle draws, as the console log does, against psx-spx's first clause.
DuckStation is the stronger witness on GPUSTAT by the console log (9 of 12 against 6). **Kept, on the strongest single
witness, and still open**: the referees disagree where the log does not reach, and psx-spx's sentence sides with the
weaker one.

### 4.9 D-11: the GPU busy after a long transfer — `gpu-busy.s`, `dma/chopping`

2,048 words to VRAM by DMA in blocks of 1 to 128 and in one burst; the cycle the channel is idle, and the first polls
that find bits 28 and 26 set [M]. Every referee reads the GPU ready at the first poll after every case: DuckStation
13–17 cycles after the channel ends, SwanStation 13, Beetle PSX 17–31, PCSX-Redux 19, Diamond 25–32; PCSX-ReARMed
alone, after the burst, sets bit 26 22,031 cycles late (its blocks are not timed at all: 523 cycles each). The bursts
take 2,195 (Diamond), 2,190 (DuckStation), 2,186 (SwanStation), 2,077 (Beetle PSX), 36 (PCSX-ReARMed) and 19
(PCSX-Redux) cycles; the console's `chopping` log, 2,196 [M]. DuckStation's `chopping` busy marks equal Diamond's on
all 130 rows, so they share the same 13 disagreements with the console, the console busy after the 128-word blocks,
the burst and the chopped windows of 16 to 128 words [M]. `gpu/bandwidth` printed only its heading on DuckStation in 60
seconds, and on PCSX-Redux every row in 58 to 7,257 hblanks against the console's 1,216 to 39,938 [M].

**What it shows** [A]. No referee keeps the GPU busy after a transfer, and one (DuckStation) independently arrives at
Diamond's busy marks, which supports the 117 rows they share and says nothing of the 13. **Still open**.

### 4.10 ps1-tests' TTY programs as RAM records — `timers.s` and `records/`'s `access-time.s`, `chopping.s`, `bandwidth.s`, `chain-looping.s`

*Added 2026-10-11.* The five rewrites of `Diamond_Records.md` hold, on Diamond, what the release programs print (that
page's §2–§6), and on DuckStation and PCSX-Redux what those print there (its §7.1: `access-time` 51 of 51 cells on both,
`chopping` 130 of 130 rows on PCSX-Redux and 109 of 130 on DuckStation, the rest ±4 cycles). Run on every referee, they
grade for the first time Beetle PSX, SwanStation and PCSX-ReARMed on these programs [M; the tables are `Diamond_Records.md`
§7.2]. Against the console's logs:

| Log | Diamond | DuckStation | SwanStation | Beetle PSX | PCSX-Redux | PCSX-ReARMed |
|---|---|---|---|---|---|---|
| `timers`, the gate's 18 rows | 18 | 13 | 13 | 9 | 1 (stops in the target check, as its release does) | 2 |
| `cpu/access-time`, 51 cells within 0.2 | 49 | 36 | 33 | 27 | 24 | 0 |
| `dma/chopping`, 130 rows within 10% | 129 | 17 | 11 | 5 | 0 | 0 |
| `dma/chopping`, busy marks | 117 | 117 | 117 | 117 | 60 | 19 |
| `gpu/bandwidth`, 14 rows within 2% | 14 | not finished | 0 | 0 | 0 | 0 |
| `dma/chain-looping`, 4 lines within 10% (words equal) | 4 (4) | 0 (4) | 0 (4) | 0 (4) | 2 (2) | 0 (2) |

**What it shows** [A]. The new data bear on four entries, none of whose rulings change:

- **D-4.** §4.3's counts read the release's `"%2-d"` hundredths as tenths ("7.9" is 7.09 cycles, not 7.9) [M: so read,
  the same DuckStation TTY gives §4.3's 11 and 25]; read as the program prints them, as the TTY gate reads them,
  DuckStation meets ~~11~~ **36** of the 51 cells within 0.2 cycles and PCSX-Redux ~~11~~ **24**,
  the regions they miss being those memory control times (RAM, EXPANSION2 and 3, CD-ROM, SPU). The named SPUCNT 32-bit
  cell now has referee readings on the same exception handler (the rewrite's own): Beetle PSX 38.09, Diamond 36.07,
  DuckStation and SwanStation 82.04, against the console's 38.94 with Sony's. Still no referee models memory control as
  the console times it, and the rule stays on the console's log.
- **D-5.** Beetle PSX and SwanStation reproduce neither `chopping`'s cycles (5 and 11 rows within 10%) nor
  `chain-looping`'s (Beetle no slower under a never-ending chain, SwanStation 5.6 times slower, as DuckStation). Still
  open.
- **D-6.** Of timer 0's ten sync-mode-2 rows, Diamond's alone read zero outside the blank and count within it, as the
  console's do; Beetle PSX and PCSX-ReARMed count in every row, SwanStation and PCSX-Redux read zero in every row,
  DuckStation zero in seven. Confirmed on the log, as before; no referee bears on it.
- **D-11.** Beetle PSX and SwanStation mark `chopping`'s GPU busy on the same 57 rows as Diamond and DuckStation, those
  whose blocks leave words unsent, so three lineages now agree on the 117 rows and none shows the 13. The release reads
  the mark after `measure`'s `ExitCriticalSection`, on Diamond's open kernel 2,141 cycles or more after the channel ends
  (`Diamond_Records.md` §8.2), so the console's 13 marks say its GPU is busy at least that long after, on a retail
  BIOS's timing. Still open.

D-0 gains one reading: Beetle PSX's `timers` record on a PAL console counts a frame on the system clock at 112,560
against the log's 112,531 (Diamond 112,557, DuckStation 112,633), but a frame on the dot clock at 112,184 against
112,031, as DuckStation's 112,196 does: its dot clock on a PAL console is not the console's. The line's ruling rests on
the frame's cycles, which it meets; its standing as the strongest witness is for the line, not the dot clock.

### 4.11 ps1-tests' value programs as printf records — `records/`'s `otc-test.s`, `dpcr.s`, `io-access-bitwidth.s`

*Added 2026-10-11.* The three record every printf call, its format's address and arguments, and are rendered with the
release's own formats (`Diamond_Records.md` §11). On Diamond each renders as the release prints on the open kernel but
for one line, `io-access-bitwidth`'s GPUSTAT row, whose cause is Diamond's (that page's §14, §16.1); on DuckStation as
the release prints there but for two rows read at other moments, and on PCSX-Redux but for the stack's address in 15
fail lines (its §15). Against the console's logs [M]:

| Log | Diamond | DuckStation | SwanStation | Beetle PSX | PCSX-Redux | PCSX-ReARMed |
|---|---|---|---|---|---|---|
| `dma/otc-test`, the log's 13 tests passed (of the release's 15) | 13 (15) | 13 (15) | 8 (9) | 12 (14) | 4 (4) | 12 (12) |
| `dma/dpcr`, the log's 14 lines | 14 | 14 | 14 | 12 | 14 | 14 |
| `cpu/io-access-bitwidth`, 108 cells as the console's | 85 | 58 | 64 | 70 | stops at the first misaligned store, as its release | 45 |

**What it shows** [A]. Two questions with no entry yet, reported for a dispute step rather than ruled here: GPUSTAT
after GP1(00h) during a fill, which psx-spx gives as `14802000h` and DuckStation, SwanStation and Beetle PSX read ready at
once, where Diamond stays busy until the fill's end (`Diamond_Records.md` §16.1); and DICR's bit 6, writable by psx-spx
and on Diamond and DuckStation, zero in the console's log and on Beetle PSX and SwanStation, and on PCSX-ReARMed after
a halfword or word store (§16.2). Beetle
PSX runs a DMA channel whose DPCR enable is clear, in `otc-test` and `dpcr` alike, where the logs and every other
referee do not: on the DMA's enable bits it is not a witness. None of D-0 to D-23 is moved by these data.
*DICR's bit 6 was ruled by §4.12.*

### 4.12 DICR's bit 6, SWCz's store and the garbage registers — `dicr.s`, `swcz.s`, `cop0-garbage.s`

*Added 2026-10-11.* Three programs of the project's own, side-loaded on all five as in §2.4, interrupts off; the
second and third install their own exception vector with the I-cache off, as `exceptions.s` does, and record what it
takes. Each finished on every referee (D1A0D00Eh written) but `swcz.s` on PCSX-Redux, which stopped producing frames
before its done word, as it does at an address error (§8) [M].

**`dicr.s`** (`Diamond_Records.md` §16.2), DICR read back after each write:

| Case | Console log | Diamond before | DuckStation | SwanStation | Beetle PSX | PCSX-ReARMed | PCSX-Redux |
|---|---|---|---|---|---|---|---|
| word, halfword, byte store of 12345678h | `340038h` ×3 | `340078h` ×3 | `340078h` ×3 | `340038h` ×3 | `340038h` ×3 | `C340038h`, `C340038h`, `C340078h` | `340038h`, `12345678h`, `12345678h` |
| 7Fh; 40h; 7FFFh | — | 7Fh, 40h, 7Fh | 7Fh, 40h, 7Fh | 3Fh, 0, 3Fh | 3Fh, 0, 3Fh | `C00003Fh`, `C000000h`, `C00003Fh` | `1200003Fh`, `12000000h`, `1200003Fh` |
| 00FF007Fh | — | `FF007Fh` | `FF007Fh` | `FF003Fh` | `FF003Fh` | `8CFF003Fh` | `92FF003Fh` |
| halfword, byte read after 7Fh; then 0 | — | 7Fh, 7Fh, 0 | 7Fh, 7Fh, 0 | 3Fh, 3Fh, 0 | 3Fh, 3Fh, 0 | 3Fh, 3Fh, `C000000h` | 3Fh, 3Fh, `12000000h` |

Ruled with the console's log and three lineages against DuckStation: bit 6 reads zero. Diamond after the change reads
as SwanStation and Beetle PSX in every case. PCSX-ReARMed and PCSX-Redux carry flag bits among 24-30 from the first
store that the others acknowledge, and PCSX-Redux keeps a narrow store's register whole, so they bear on bit 6 only.

**`swcz.s`** (`../BlackMoon-HW/BlackMoon_R3000.md` D-21): DuckStation stores 0 for SWC0, SWC1 and SWC3 to RAM and takes
DBE on SWC3, SW and SWC0 to C0000000h; Beetle PSX, SwanStation and PCSX-ReARMed leave RAM unchanged, and Beetle PSX
takes no bus error even on SW there. SWC2, the control, stores 12345678h on all four. Ruled: the store is made (a
console's expectations, through amidog's model), its value DuckStation's 0, a single witness.

**`cop0-garbage.s`** (`../BlackMoon-HW/BlackMoon_R3000.md` §10), 28 reads of registers 16-31 and the exceptions taken:
DuckStation 28 reserved-instruction exceptions; Beetle PSX none, every register left as it was (08004088h, 0, 80000000h,
C1h, 304h, 1F800000h, 8, 4 are what the program's start left in t1 to t8); SwanStation, PCSX-ReARMed and PCSX-Redux
0 and none; none reads BPC's, BDA's, BPCM's or BDAM's known value straight after it, as psx-spx describes. Ruled: no
exception (three lineages), the value 0 a choice left standing.

### 4.12 D-23: the texture cache, interlaced drawing and GPUREAD after a transfer — `texture-cache.s`, `interlace.s`, `gpuread-repeat.s`

*Added 2026-10-11, at the cache step* (`Diamond_Disputes.md` D-23). Each program side-loaded on the five referees by
`psxreferee.py ram`, the player's BIOS given by path, NTSC, and read from RAM [M]:

| | Diamond | DuckStation | SwanStation | Beetle PSX | PCSX-ReARMed | PCSX-Redux |
|---|---|---|---|---|---|---|
| `texture-cache.s`: lines 10 to 16, the texel each row reads (fresh: 1, 2, 2, 2, 3, 4, 5) | 1, 1, 2, 2, 2, 4, 5 | fresh | fresh | 1, 1, 2, 2, 2, 4, 5 | fresh | fresh |
| `interlace.s`, display from line 0: lines 0 to 3 left (fill, rectangle) | 0, 2 | none | none | 1, 3 | 0, 2 | none |
| `interlace.s`, display from line 1: lines 8 to 11 left | 9, 11 | none | none | 9, 11 | none | none |
| `interlace.s`, bit 10 set | none left | none | none | none | none | none |
| `gpuread-repeat.s`: after three words | the first, `02020101h` | the last | the last | 0 | the first | `00000400h` |
| `gpuread-repeat.s`: the one-word read, then after it | `03030202h`, then `02020101h` | `00000202h`, then the same | the same as DuckStation | `03030202h`, then 0 | `03030202h`, then the same | `00000202h`, then `00000400h` |

**What it shows** [A]. Beetle PSX alone keeps texels in a cache at all, and keeps exactly Diamond's new rule's: a row of
64 8-bit texels whole, line 32 taking line 0's entries, the cache kept through a 15-bit page and cleared by a 4-bit one
and by another X; the others read VRAM afresh for every texel. Two lineages leave lines undrawn in vblank of 480-line
interlace, fills included (Beetle PSX, PCSX-ReARMed); PCSX-ReARMed the same lines as amidog's model with the display at
line 0, Beetle PSX the odd lines whatever the start, so the start's part in the parity stands on the model alone.
PCSX-ReARMed repeats a longer transfer's first word, as the model; nothing repeats an earlier word after a one-word
transfer but the model. Beetle PSX and PCSX-ReARMed read a one-pixel transfer's second halfword as the pixel after,
as D-23's odd read has it, where DuckStation, SwanStation and PCSX-Redux pad it with zero.

### 4.13 D-41 and D-42: the order of a triangle's rows, and a fill cut by GP1(01h) — `triangle-rows.s`, `fill-reset.s`

*Added 2026-10-12, at the GPU's rows step* (`Diamond_Disputes.md` D-41, D-42). Each program side-loaded on the five
referees by `psxreferee.py ram`, the player's BIOS given by path, NTSC, and read from RAM; each finished on every
referee (D1A0D00Eh written) [M].

**`triangle-rows.s`**: a 4-bit page at 256,0 filled with texels 1 and the cache cleared, a raw textured shape drawn
with U = 40h + x and V = 40h + y, the page filled with texels 2, and a 64x64 rectangle from 40h,40h read back at its
columns 0, 16, 32 and 48, one per cache entry; the lines on which the rectangle found the entry kept (texel 1):

| Shape | Beetle PSX, columns 0 / 16 / 32 / 48 | Diamond as built | Diamond, D-41's rule |
|---|---|---|---|
| 1, flat at the bottom: (64,0) (0,72) (64,72) | 56-63 / 38-63 / 20-63 / 2-63 | the same but 8-63 in the last | as Beetle PSX |
| 2, flat at the top: (0,0) (64,0) (0,72) | 8-63 / 0-53 / 0-35 / 0-17 | as Beetle PSX | as Beetle PSX |
| 3, the middle vertex low on the left: (32,0) (0,72) (64,80) | 39-63 / 3-63 / 1-63 / 41-63 | 12-63 and 14-63 in the middle two | as Beetle PSX |
| 4, low on the right: (32,0) (64,72) (0,80) | 43-63 / 3-63 / 1-63 / 37-63 | 14-63 and 12-63 in the middle two | as Beetle PSX |
| 5, high on the left: (32,0) (0,8) (64,80) | 5-24 / 1-42 / 1-60 / 41-63 | as Beetle PSX | as Beetle PSX |
| 6, shape 3 in another vertex order | as shape 3 | as shape 3 | as Beetle PSX |
| 7, a quad: (0,0) (64,0) (0,72) (64,72) | 8-63 / 8-63 / 8-63 / 2-63 | the same but 8-63 in the last | as Beetle PSX |

DuckStation, SwanStation, PCSX-ReARMed and PCSX-Redux read texel 2 on every line of every shape: no cache. Beetle PSX
reads as a triangle drawn from its middle vertex's row down and then from above it upward, the lower part first; the
upper part first gives shapes 3, 4 and 6 otherwise (lines 8 to 11 or 13 of columns 16 and 32 fresh), and a reversal of only
the triangles flat at the bottom gives them as built [M, the orders tried on Diamond].

**`fill-reset.s`**: four white fills of 240x511 at x 0, 256, 512 and 768, each written by the CPU and followed after
a loop of 0, 100, 400 or 1,600 turns by GP1(01h), column 0 of each read back: all 511 lines white in all four on all
five referees, as on Diamond. None stops a fill at GP1(01h), where amidog's overflow test expects one stopped after
some five lines.

### 4.14 D-29 and D-43: DICR's edge and a channel beside a list — `dicr-edge.s`, `dicr-storm.s`, `dma-beside.s`, and the discs' pictures

*Added 2026-10-12.* Three programs of the project's own, side-loaded on all five as in §2.4, run with the player's BIOS
given as each referee's firmware [M]:

| Program | Diamond before | DuckStation | SwanStation | Beetle PSX | PCSX-ReARMed | PCSX-Redux |
|---|---|---|---|---|---|---|
| `dicr-edge.s`, I_STAT's bit 3 in cases 0–9 | 8 8 0 8 0 0 8 8 0 8 | 8 0 0 8 0 0 8 0 0 8 | 8 0 0 0 8 0 0 0 0 8 | as DuckStation | as DuckStation | as DuckStation |
| `dicr-storm.s`, the handler's runs | 1,000 | 1 | 1 | 1 | 1 | 1 |
| `dma-beside.s`, channel 6 done straight after its start (four cases) | no, no, no, no | yes ×4 | yes ×4 | yes ×4 | its table written ×4, CHCR still started | its table written ×4, CHCR still started, the list already ended |

DICR itself reads alike on all six in `dicr-edge.s` but for SwanStation, which never raises bit 31 by a write (cases
3, 6 and 7). Ruled in `Diamond_Disputes.md` D-29 and D-43; Diamond after both changes reads as DuckStation on
`dicr-edge.s` and `dicr-storm.s`, and as DuckStation, SwanStation and Beetle PSX on `dma-beside.s`'s channel 6.

**The discs' pictures.** `psxreferee.py pictures` runs a libretro referee on a disc and keeps its screen every K frames
in the run tool's picture form, the probe's other spaces removed unread, since on the player's BIOS they hold its RAM.
SwanStation and Beetle PSX ran the seven test discs to frame 6,000 on the player's American BIOS; PCSX-ReARMed ran them on its own
kernel. The libretro channel carries no pad setting the driver pins, and SwanStation's port 1 is not a DualShock:
Ape Escape shows it its "DUALSHOCK … has not been plugged into port 1" screen, Beetle PSX and Diamond go on to the
game. `Diamond_CDROM.md` §9.4 records each disc against them.

### 4.15 D-45: a narrow store to a 32-bit register — `narrow-stores.s`, and the discs' pictures

*Added 2026-10-12.* The program side-loaded on all five as in §2.4, the player's BIOS given as each referee's
firmware, RAM read at its done word [M]: DuckStation and SwanStation read the seven cases alike, and Beetle PSX the
same; PCSX-Redux keeps DICR's bits 0–5 under a halfword store to its upper half (`0044003Fh`), and PCSX-ReARMed keeps
those and DICR's flags and DPCR's other bytes (`0C880000h`, `0044003Fh`, `00000B99h`). Every referee shifts the store
to its byte. `Diamond_Disputes.md` D-45 has the table and the ruling; Diamond after the change reads as DuckStation,
SwanStation and Beetle PSX in every case.

SwanStation ran the seven test discs again to frame 6,000 on the player's American BIOS, pictures every 150 frames, and Silent
Hill, Castlevania: SotN and Tekken 3 to frame 4,500 every 10 frames, and Tekken 3 to frame 12,000 every 300, through
`psxreferee.py pictures`; `Diamond_CDROM.md` §9.8 records the discs against them.

### 4.16 D-46: a triangle's drawing time — `draw-time.s`, and the games driven into play

*Added 2026-10-12.* D-11's `programs/draw-time.s`, unchanged, side-loaded on Beetle PSX and SwanStation through the
probe and on DuckStation and PCSX-Redux through their own drivers, the player's BIOS given as each one's firmware, RAM
read at its done word [M]; PCSX-ReARMed did not run: the probe built for this step refused it with its no-firmware exit (4), `psxreferee.py`
passing that core no firmware folder, and the cause was not pursued. The four triangle records, cycles from the last word to
GPUSTAT bit 26: Beetle PSX 10, 10, 262 and 4,006; DuckStation and SwanStation alike, 25, 43, 286 and 4,129; PCSX-Redux 8
for every record of the program, as it charges no drawing time. `Diamond_Disputes.md` D-46 has the table and the ruling.

For the games, `psxreferee.py pictures` gained what a player does: `--press` and `--presson` (the probe's existing
schedule, through the libretro input callback), `--stick` (the probe's new analog schedule, answered through the same
callback for `RETRO_DEVICE_ANALOG`), `--device`, `--option`, `--card-in` (the probe's `--sram`), `--card-out` (the
card's save RAM as the last picture leaves it, the one space of the probe's that is kept) and `--wav`. `psxplay.py`
beside it drives a disc by a press script on Diamond's run tool or on SwanStation; `Diamond_Players.md` §13 records the
seven games so driven. The probe's input reaches only the referee's own pad; nothing of the BIOS is read.

### 4.17 D-47: a primitive carried past 11 bits by the offset — `offset-fold.s`

*Added 2026-10-12.* `programs/offset-fold.s`, side-loaded on SwanStation and Beetle PSX through the probe and on
DuckStation and PCSX-Redux through their own drivers, the player's BIOS given as each one's firmware, RAM read at its
done word [M]. The area is VRAM's right half and the offset 768; the program draws a flat triangle from (286,16), the
same triangle from a first vertex on the screen, a Gouraud triangle from 300 and a flat line from 300 leftward, and
reads back x 896 to 1023, lines 0 to 191. DuckStation, SwanStation and Beetle PSX read back word for word alike: 783,
783, 1,326 and 56 pixels drawn, from x 992 for the shapes that start at 286 or 300; Diamond before the change drew only
the triangle from the screen, after it all four the same as those three. PCSX-Redux fills columns 896 to 991 for every
shape, the control among them, and is not counted. `Diamond_Disputes.md` D-47 has the table and the ruling.

### 4.18 D-49: a display range moved onto or across the beam — `range-cross.s`

*Added 2026-10-18.* `programs/range-cross.s`, side-loaded on the five referees by `psxreferee.py ram`, the player's
BIOS given as each one's firmware, RAM read at its done word [M]. Nine cases of a GP1(07h) written mid-line: the vblank
interrupts that follow, GPUSTAT bit 31 over the next lines, and timer 1 in sync modes 0 and 3 written before or after
the range. PCSX-Redux and PCSX-ReARMed raise IRQ0 at the same lines whatever the range and gate nothing, and are not
counted. Beetle PSX and DuckStation agree on timer 1's gate in every case written before the range, and disagree on
the interrupt in every case that moves the line across the range. `Diamond_Native.md` §29.4 has the tables,
`Diamond_Disputes.md` D-49 the ruling.

### 4.19 D-50: a Y2 past the field — `y2-past-field.s`

*Added 2026-10-18.* `programs/y2-past-field.s`, side-loaded on the five referees by `psxreferee.py ram` as in §4.18.
Beetle PSX gives no vblank interrupt for a Y2 of 263 or 300 and shows the top lines as the picture; DuckStation and
SwanStation give one each field and hold timer 1 in the control as well; the PCSX family ignores the range.
`Diamond_Native.md` §29.10 has the table, `Diamond_Disputes.md` D-50 the ruling.

## 5. What changed in Diamond

Each a writer's step citing its entry, after the rulings above:

- **D-0, the interlaced PAL field**: 313 and 312 lines alternately, 312.5 on average (`video.rs`); 50.0003 Hz
  interlaced on a PAL console, 50.4605 Hz on an NTSC one [M]. Pinned by
  `interlaced_fields_alternate_about_262_5_and_312_5_lines` and `the_video_timing_is_the_referees_line_on_both_clocks`.
- **D-7, the base vertex**: a triangle's attributes start from its leftmost vertex, of two in the leftmost column the
  one that follows the other in the command's order (`raster.rs`). Pinned by
  `the_base_is_the_leftmost_vertex_and_of_two_the_follower`, `a_shaded_triangle_reads_the_same_in_every_vertex_order`,
  `shaded_triangles_draw_in_every_vertex_order_as_the_referees_do`, and ps1-tests' pictures, all exact.
- **D-8, the x-flip's start**: U with its low bit set (`raster.rs`). Pinned by
  `an_x_flipped_rectangle_starts_from_u_with_its_low_bit_set`, `flipped_rectangles_start_where_the_referee_starts_them`,
  and `texture-flip`, exact.

- *The cache step (D-23)*: the 8-bit cache block of 64 by 32 texels and the page changes that clear the cache, on
  amidog's model and Beetle PSX; the interlace rule; GPUREAD's repeat (`texture.rs`, `draw.rs`, `mod.rs`). Pinned by
  `the_texture_cache_keeps_a_64_texel_row_and_clears_on_a_page_change`,
  `interlaced_drawing_in_vblank_skips_the_lines_from_the_display_start`,
  `gpuread_repeats_the_first_word_of_the_last_longer_transfer`.

- *The stream's step (D-45)*: a byte or halfword store to a 32-bit register at an address not word-aligned writes the
  register whole, the word shifted to the store's byte (`bus.rs`). Pinned by
  `a_narrow_store_writes_its_register_shifted_to_its_byte`.

- *The second play step (D-47)*: a triangle or line reaching left of -1,024 (or above it) is drawn 2,048 further on
  (`raster.rs`), on three referees of two lineages. Pinned by `a_primitive_past_the_offsets_eleven_bits_is_drawn_modulo_2048`.

- *D-49*: a new Y1 or Y2 on the beam's line is met in the next field (`video.rs`), on two lineages for the timer and
  the strongest witness for the field. Pinned by `a_range_moved_onto_the_beams_line_waits_for_the_next_field`.

- *D-50*: a Y2 at or past the field's line count gives no vblank (`video.rs`), psx-spx and Beetle PSX against
  DuckStation, the tester's choice. Pinned by `y2_past_the_field_stops_the_vblank_interrupt`.

The other programs are pinned on Diamond as it reads them: `gpustat_while_each_shape_arrives_and_draws`,
`the_gpu_is_ready_at_once_after_each_transfer`, `stores_drain_as_the_fit_has_them`.

## 6. The referees against the console's logs

| Console log or picture | Diamond | DuckStation | PCSX-Redux | Beetle PSX | SwanStation | PCSX-ReARMed |
|---|---|---|---|---|---|---|
| `timers`, a PAL-clocked frame (571,324 ± 6) | 571,332 | 571,416 | no region clock | 571,334 | 571,425 | no region clock |
| `timers`, mode 2 rows | exact | free-running; zero in 7 of 10 by the record | zero | counting, by the record | zero, by the record | counting, by the record |
| `timers`, the gate's 18 rows (the records, §4.10) | 18 | 13 | 1, stopped | 9 | 13 | 2 |
| `timers`, the target check's booleans | as the log | not | not reached | not | not | not |
| `timers`, sync tables in the log's shape (of 14, *2026-10-11*, `Diamond_Native.md` §11.12) | 14 (6 before D-6's second part) | 10, by the record | 3, by the record | 13, by the record | 9, by the record | 1, by the record |
| `cpu/access-time`, 51 cells within 0.2 | 50 | ~~11~~ 36 | ~~11~~ 24 | 27, by the record | 33, by the record | 0, by the record |
| `dma/chopping`, 130 rows within 10% | 129 | 17 | 0 | 5, by the record | 11, by the record | 0, by the record |
| `dma/chopping`, busy marks | 117 | 117 (the same) | 60 | 117 (the same), by the record | 117 (the same), by the record | 19, by the record |
| `gpu/bandwidth`, 14 rows within 2% | 14 | not finished | 0 | 0, by the record | 0, by the record | 0, by the record |
| `dma/chain-looping`, 4 lines within 10% | 4 | 0 | 2 | 0, by the record | 0, by the record | 0, by the record |
| `gpu/gpustat`, 12 readings | 12 | 9 | 4 | 6 | 9 | 3 |
| `dma/otc-test`, the log's 13 tests (§4.11) | 13 | 13 | 4, by the record | 12, by the record | 8, by the record | 12, by the record |
| `dma/dpcr`, the log's 14 lines (§4.11) | 14 | 14 | 14, by the record | 12, by the record | 14, by the record | 14, by the record |
| `cpu/io-access-bitwidth`, 108 cells (§4.11) | 85 (91 since §4.12) | 58, by the record | stops, the release too | 70, by the record | 64, by the record | 45, by the record |
| ps1-tests' pictures | exact | not read | not read | not read | not read | not read |

DuckStation and Beetle PSX are the strongest referees, each on its own subsystem: Beetle PSX on video timing,
DuckStation on GPUSTAT; PCSX-Redux models almost none of the timing the logs measure [A]. Diamond's own fits score
best on every log, which is expected of rules fitted to them and says nothing more [A]. *The records of 2026-10-11*
(§4.10) leave the ranking as it was: Beetle PSX's frame cycles on a PAL console are the nearest the log's but its dot
clock is not, and on the access times, the DMA's cycles and the drawing times no referee comes near the logs.

## 7. What was read for this page

No emulator source, no firmware, no firmware listing, no PSX_MiSTer file. Read: the READMEs of DuckStation,
PCSX-Redux and PCSX-ReARMed (licences, lineage, renderers, downloads); PCSX-Redux's user documentation pages
"Command Line Flags", "Redux basic API", "Events", "Memory and registers", "Introduction" (Lua), "GDB server", "Misc
Features" and "Mips API" (the last quotes its SDK's header for the debug port, which was not used); GitHub's API for
releases and licences; the `distrib.app` web page's script, for its manifest's address, and the manifest; the
referees' settings files as they wrote them, their command-line help and their logs' System and TTY channels; the
probe's option lists; ps1-tests' console logs. Seen as runs' outputs, kept in the cache and not recorded here beyond
the figures of §4: the referees' TTY from each program's heading on, and RAM.

Exposures, recorded as the rules ask [M]: the first runs' outputs, before the driver cut them at the program's
heading, held the player's BIOS's own boot messages and PCSX-Redux's report of its POST codes; one hung DuckStation
was examined with a debugger's backtrace of its threads, which names functions of its Qt frontend from its symbols
(it was waiting in a message box); and an early PCSX-Redux `-h` ran for 20 seconds with the desktop's display in its
environment, printing nothing, before every run removed it. The person who wrote this page also read
`Diamond_References.md` §4's account of Beetle PSX's OpenBIOS run, and should not be a writer of the open BIOS (the
plan's §1.3) [C, the tester's to confirm].

*The records' step, 2026-10-11* (§2.5, §4.10): the referees' RAM through the driver; for DuckStation and PCSX-Redux the
release programs' TTY from each program's heading on, which held PCSX-Redux's own report of an address error in
`timers` and DuckStation's `VSync: timeout` lines from the release's library, and nothing of the BIOS's; the process
table and the processes' cgroups, to find what runs left. What else was read is `Diamond_Records.md` §9.

*The value programs' step, 2026-10-11* (§4.11): the referees' RAM through the driver, and for DuckStation and
PCSX-Redux the three releases' TTY from each heading on, `io-access-bitwidth`'s through `psxrecords.py mask` so that no
BIOS word of the BIOS row was stored or read; it held PCSX-Redux's own reports of unmapped writes and of the address
error. What else was read is `Diamond_Records.md` §17.

*The step of §4.12, 2026-10-11*: the referees' RAM through the driver, and nothing else of theirs; amidog's
`psxtest_cpx` disassembled from Diamond's RAM while it ran (its BREAK case runner and the cases' code), and its TTY on
Diamond. psx-spx's "DMA Channels" DICR section and "Unpredictable Things", "COP0 - Debug Registers" and
"cop0r16-r31"; LSI's *CW33000 User's Manual* for its debug registers (it defers their detail to the LR33000 manual,
which was searched for and not found). No emulator source, firmware or listing; no BIOS output was read.

## 8. Negative results

- no$psx was not run: wine is not installed, and installing system packages was out of scope.
- No referee exposes a console-faithful model of memory control, the write queue, the DMA's gaps, timer sync mode 2,
  or the GPU's busy time after a transfer (§4.3–§4.5, §4.9); those disputes stay on the console logs alone.
- No referee times the multiplier the same way (§4.2).
- DuckStation's `gpu/bandwidth` did not finish in 60 seconds; PCSX-Redux models no drawing time. *2026-10-11:* neither
  the release nor its record finished the first row (vramToCpu) on DuckStation in 300 seconds.
- PCSX-Redux stops in `timers`' target check, the release and its record alike: its timer 2 passes the target, and the
  test's table index with it, until a halfword store faults.
- DuckStation cannot run on Qt's `offscreen` platform; it needs a compositor (§2.1).
- PCSX-Redux stops at `cpu/io-access-bitwidth`'s first misaligned store, the release and its record alike: its debugger
  takes the address error as a "first chance exception" and the frames stop, so the record's RAM is never read (§4.11).
- PCSX-Redux stops producing frames in `swcz.s`, before its done word, so its RAM was not read (§4.12).
- PCSX-Redux stops producing frames partway through ps1-tests' `cpu/cop` (after its reserved-opcode case), so the
  driver also stops it after a wall-clock limit and keeps what it printed.
