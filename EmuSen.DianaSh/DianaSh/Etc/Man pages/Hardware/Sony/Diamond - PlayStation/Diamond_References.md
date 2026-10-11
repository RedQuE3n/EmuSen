# Diamond_References — the oracles, documents and reference cores, pinned

*Written 2026-10-11, at stage 1.* The record of what `Diamond_Plan.md` §1.5, §1.6, §2.1 and §3.1 asked stage 1 to fetch,
pin and measure, and of what that found. The plan's sections carry a dated revision pointing here. Everything named
lives outside the repository under `~/.cache/emusen/probe/diamond/` (and the reference cores under the shared
`~/.cache/emusen/probe/libretro/cores/`), listed with source, version and MD5 in `PROVENANCE.txt` there, the documents
with SHA-256 in `docs/SHA256SUMS` and with their status in `docs/DOCS.txt`. Nothing fetched is committed.

Marks as in the plan: **[M]** measured on 2026-10-11 with the tool named; **[R]** read from the source named, on
2026-10-11; **[A]** argued.

---

## 1. The test programs

| Oracle | Fetched | Size | What it holds | Licence, as read |
|---|---|---|---|---|
| **ps1-tests** | Source at `f727802` (git) [M]; binaries from the newest release, `build-158` (2021-04-22, built at `5f73d61`), `tests.zip`, MD5 `dce947f6…` [M] | 29 MB source, 6.6 MB zip, 17 MB unpacked | **49 PS-EXEs, 17 VRAM references (`vram*.png`), 21 console logs (`psx.log`)** and `diffvram` for Linux, macOS and Windows [M]. The release also has programs the README does not list: `cdrom/timing`, `dma/dpcr`, `timer-dump` (which needs a modified board), `input/pad` [M] | MIT [R GitHub API] |
| **amidog's tests** | `psxtest_cpu` 1.3 (2014-02-07), `psxtest_cpx` 1.1 (2013-12-31), `psxtest_gte` 1.4 (2013-12-31), `psxtest_gpu` 1.7 (2014-12-28), each a zip of a PS-EXE and a readme, from `psx.amidog.se` [M, versions R their readmes] | 62, 32, 61 and 100 KB | The four programs; the GPU test's readme lists its v1/v2 GPU support and its cache, FIFO, IRQ and MOVE tests [R] | **No licence on the download pages themselves**, the CPU, GTE and GPU pages read; the wiki's footer licenses its content under **CC BY-NC-SA 3.0** "except where otherwise noted"; the readmes state none [R]. Local use only (Q7) |
| **Peter Lemon's PSX** | `6d20c13` (git) [M] | 32 MB | **72 prebuilt PS-EXEs** and 78 PNG pictures beside their armips sources [M]: CPU per-instruction tests, GTE `AVSZ` and `NCLIP`, 16- and 24-bit GPU programs, MDEC, SPU, decompression [R directory names] | None stated [R GitHub API]; its README says he tests "with a real PSX" only and names no source of the pictures [R] |
| **PSn00bSDK examples** | `5d9aa2d`, `examples/` only (sparse checkout) [M] | 16 MB | Source; no SDK installed [M] | Not asserted by GitHub [R] |

**What changed from the plan's §3.1** [M]: the newest prebuilt ps1-tests are older than `f727802` and lack only
`gpu/gpustat`, which `ac27df2` (2023-02-20) added; `f727802` itself changes nothing but CI. `gpustat` would be built with
the repository's own recipe (`docker run … jaczekanski/psn00bsdk make`); it was not built and no toolchain was
installed. amidog's CPU and GPU tests are newer than the plan's single date of 2013-12-31. The plan said ps1-tests'
pictures sit beside the GPU and MDEC programs; the release adds console logs for most of the text-reporting programs,
which makes them graded by comparison with a console's output, not only by their own verdicts [A].

**A search for an AccuracyCoin-like suite** (one image, many hardware areas, a score) found none [M, 2026-10-11: two
web searches; GitHub's repository search for "psx test accuracy" and "ps1 hardware tests"; the Emulation General Wiki's
"PS1 Tests" page]. That page is a table of emulators' scores on amidog's four tests taken together, marked outdated,
not a suite of its own [R]. Also found and not fetched: `filipalac/240pTestSuite-PS1` (GPL-2.0, release 2020-12-19, a
video test suite whose pictures are judged by eye, 0.2 and 0.5 MB) [R GitHub API]; `mrodriguezme/ps1_tests` is an empty
repository [R].

## 2. The documents

| Document | Pinned | Notes |
|---|---|---|
| psx-spx, the maintained conversion | branch `stable` at `6d7d1bc` (committed 2026-10-08) [M] | 47 files, 22 MB without its history; no licence file [M] |
| nocash's single-file `psx-spx.htm` | as served on 2026-10-11, MD5 `958d11ea…` [M] | The server's Last-Modified is 2026-06-05, later than the 2022-08-20 the conversion gives as his last release; not compared [M] |
| IDT, *R30xx Family Software Reference Manual*, revision 1.0 | 354 pages [M] | A university course's copy |
| IDT, *MIPS Microprocessor Family Software Reference Manual*, version 2.0, October 1996 | 548 pages [M] | From a community archive; IDT's own manual |
| LSI Logic, *CW33000 MIPS Embedded Processor User's Manual*, 1992 (CI4002A) | 134 pages [M] | bitsavers |
| ECMA-130, 2nd edition (1996); ECMA-119, 2nd (1987) and 5th (2024) editions | [M] | Ecma's own site, free |
| Sony CXD2510Q (CD DSP), CXD1199AQ (CD-ROM decoder), CXD2545Q (DSP with servo) | 48, 43 and 102 pages [M] | bitsavers for the first two, the CXD1199AQ's MD5 equal to the Internet Archive's copy; the CXD2545Q is a scan with no text layer, from a datasheet site |

**Not fetched** [M]: the CXD1815Q's datasheet, found nowhere (psx-spx documents its registers and relates it to the
CXD1199AQ's); Kane and Heinrich's book, which is not free; LSI's LR33000 manuals; the write-ups, patents and Sony
documents of the plan's §2.1 and §2.2, left to the stage that first cites each.

**Exclusions** (`docs/DOCS.txt` §X): for the open BIOS's writers, psx-spx's "BIOS Patches" (`kernelbios.md` from line
2741 to the end, and the same chapter in nocash's edition), with three more kernel chapters to be screened ("BIOS
Internal Boot Functions", "BIOS More Internal Functions", "BIOS Versions"). For **every** writer, psx-spx's chapters on
the CD controller's HC05 microcontroller and its "Summary of Used CX(xx) Commands", to be screened before anyone reads
them for design: the controller is firmware, modelled at its register interface (the plan's §5.6), and Diamond needs
none of its internals [A]. The lists were made from headings only.

**Added for D-0** (2026-10-11): Sony's service manuals for the SCPH-5500, SCPH-7500 and SCPH-9000 series and the
SCPH-100 (PU-18, PU-22, PU-23, PM-41), under `docs/sony-service-manuals/` with their hashes (`DOCS.txt` §C); what was read
in them, and the psx-spx `master` commit `cafc0eb` read beside the pinned `stable`, are recorded in `Diamond_Disputes.md`
D-0's second step.

## 3. The probe

The libretro backend already passed a path to cores that ask for one; all three set `need_fullpath` [R their system
info] and needed nothing new to load a PS-EXE. Two small changes, both in `src/backends/libretro.rs` [M]:

- **`.exe` and `.psexe` name the PlayStation** (`psx`), and a PlayStation run a core refuses without the player's own
  dumps is reported `[SKIP]` and exits 4, as a Sega CD run is.
- **A core that would boot OpenBIOS is not run without the player's own dumps.** Beetle PSX (and its hardware-rendered
  build) and SwanStation boot OpenBIOS when no BIOS is given (§4). OpenBIOS is not used at all, by the plan's §5.9,
  decided with Q3 on 2026-10-11. The probe therefore refuses them, by their library names, **before the core is
  initialised**, whatever the image, unless `--sysdir` holds a file; such a run is `[SKIP]`, exit 4, which WiseMan's
  `LibretroProbeEngine` already turns into a skipped test. With the tester's own dumps (Q6, open) they run as any
  reference does.
- The run's log now names the timing the core reports (frames a second, the audio rate, the base geometry), a
  witness for D-0.

Pinned by `probe-rs`'s unit tests (`system_comes_from_the_rom_extension`,
`firmware_is_looked_for_only_in_the_persons_own_folder`, `a_core_that_would_boot_openbios_is_not_run_without_dumps`;
50 pass) and by runs of the three cores: Beetle PSX and SwanStation skipped with no `--sysdir` and with an empty one,
PCSX-ReARMed run [M]; `NephriteTests`' two probe tests pass with this build (`EMUSEN_LIBRETRO_PROBE`) [M]. No other
system's behaviour changes: the extensions were unclaimed, and the refusal names only
PlayStation cores [A]. The three cores are pinned by hash in `EmuSen.WiseMan/Reference/libretro-cores.sha256`, and
`build-probe.sh libretro-core <core>` finds each pinned copy [M].

## 4. The reference cores

From the libretro buildbot's nightlies of 2026-10-10, unpacked binaries only, with their `.info` files; sources never
read; READMEs read where named.

| Core | Version | `.so` | Licence | Without a BIOS | What it was run on | Cost a frame (desktop) | Spaces through the standard calls |
|---|---|---|---|---|---|---|---|
| **Beetle PSX** | 0.9.44.1 `10e4b0c` | 12.6 MB | GPLv2 [R `.info`] | **Boots OpenBIOS** [R its README: "OpenBIOS, allowing the emulator to be used without a BIOS file"; M, below]. Its `.info` still marks the three regional BIOS files required, which the core no longer enforces [R, M] | Not run, but once (below) | not measured | from the one run: `ram` 2 MiB, `sram` 128 KiB, `map_000000` 2 MiB (RAM), `map_1f800000` 1 KiB (the scratchpad); no VRAM, no SPU RAM [M] |
| **PCSX-ReARMed** | r26 `c881679` | 2.2 MB | GPLv2 [R] | **Runs**, on its own high-level kernel ("BIOS HLE emulation" [R its README]; options `pcsx_rearmed_bios=auto\|HLE` [M]) | Peter Lemon's bare-metal `RenderPolygon16BPP.exe`, its text found loaded at `0x10000` and its picture drawn; ps1-tests' `triangle`, `transparency`, `timers`, `cop` and `benchmark`, which call the kernel, all drawing or running [M] | **Lightrec recompiler** (default): 0.030 ms (Lemon), 0.108 ms (`triangle`), 0.098 ms (`benchmark`); **interpreter** (`pcsx_rearmed_drc=disabled`): 1.96, 1.44 and 1.54 ms [M] | the same four: `ram` 2 MiB, `sram` 128 KiB (a memory card's size [A]), `map_000000` 2 MiB, `map_1f800000` 1 KiB; no VRAM, no SPU RAM; memory ids 4–8, 256 and 257 answer nothing [M] |
| **SwanStation** | 1.0.0 `dd48056` | 4.9 MB | GPLv3 [R] | **Boots OpenBIOS** [R its README: "If no ROM image is provided, OpenBIOS will be used instead"] | Not run | not measured | not measured; its `.info` declares no memory descriptors [R] |

**SwanStation's lineage** [R its README]: "a hard fork and open-source Libretro core implementation of DuckStation",
GPLv3, which confirms the plan's guess. The README also claims it "passes amidog's CPU and GTE tests", a claim not
checked here. Its `.info` says a BIOS "is required", and its README says it is not; the README describes the core.

**The cost**, measured 2026-10-11 on the desktop (Ryzen 7 7700X), under the shared test lock, as the difference of
wall times between runs of 600 and 3,600 frames divided by 3,000, three runs of each, the median and its range: Lemon's
program 0.030 (0.029–0.031) and 1.961 (1.958–1.984) ms; `triangle` 0.108 (0.105–0.108) and 1.435 (1.424–1.448) ms;
`benchmark`, 100 flat triangles a frame, 0.098 (0.097–0.101) and 1.535 (1.388–1.599) ms; the on-screen information pinned
off, other options at their declared defaults (54 options, GPU thread rendering `auto`). These are a black box's wall
times, recompiler and GPU thread included, not a budget for Diamond [A]: they say only that PCSX-ReARMed is cheap
enough to run long reference sequences.

**Reported timing** [M]: PCSX-ReARMed reports 60.000000 frames a second by default and 59.817333 with
`pcsx_rearmed_fractional_framerate=enabled`, 44,100 Hz audio and a 256×240 base. 59.817333 is 53.693175 MHz / (263 ×
3,413): a witness to one reading of D-0 (`Diamond_Disputes.md`).

**The one run of Beetle PSX**, before its README was read. Beetle PSX was run once with no BIOS on Peter Lemon's
`RenderPolygon16BPP.exe`, for 60 frames, expecting the refusal its `.info` predicts. It ran. At frame 60 the program's
text was not in RAM; the screen, 640×472, showed a rotating square on a dark field, and RAM held a resident program
whose printable strings included `PSX.EXE` [M]. That is consistent with a replacement kernel's shell waiting, not with
the test program [A]; its README then named OpenBIOS. What was looked at: the picture, the map of RAM's non-zero
256-byte blocks and the first twenty printable strings of RAM; all of it was deleted the same hour and nothing of it is
recorded beyond this paragraph. Whoever wrote this page saw it, and so should not be a writer of the open BIOS (the
plan's §1.3) [C, recommended; the tester's to confirm]; the reading log (§6) records it. The probe's refusal (§3) was
added so that it cannot happen again by accident.

**What this leaves** [A]: until the tester supplies a BIOS of their own (Q6), **PCSX-ReARMed is the only black-box
reference that may run**, and its kernel is high-level and of unknown origin, which the plan's §1.3 allows (outputs
only; a replacement writer sees outputs of games, never a trace of the kernel's code). The plan's comparison of three
references is not possible without a BIOS; disagreements that need a second witness wait for one. None exposes VRAM or
SPU RAM through the standard calls, so ps1-tests' VRAM references are graded against Diamond alone, and the references
are compared by their pictures and RAM.

## 5. PSX_MiSTer, the referee

Not cloned, as the plan says (stage 4 first needs it) [M]. **No VHDL simulator is installed**: neither GHDL nor nvc, as
a package or on any path searched [M]. Verilator 5.046 sits unpacked, not installed, in
`~/.cache/emusen/toolchains/verilator/` from Nephrite's bench [M], but it reads Verilog and SystemVerilog only, and
GitHub's language count for PSX_MiSTer is 1.90 MB of VHDL against 0.33 MB of Verilog and 0.23 MB of SystemVerilog
[R GitHub API], so simulating the core needs GHDL or nvc, which stage 4 asks the tester to allow installing, or a
MiSTer device (Q4, Q16).

## 6. What was read for this page

No emulator source and no firmware or firmware listing. Read: the READMEs of `libretro/beetle-psx-libretro`,
`libretro/pcsx_rearmed` and `libretro/swanstation` (the parts about BIOS files, OpenBIOS, HLE, lineage and tests); the
three cores' `.info` files and the system info they report; ps1-tests' README and `timers/psx.log`; amidog's download
pages, start page and the four readmes; Peter Lemon's README; psx-spx's "GPU Timings", "Pinouts - CLK Pinouts", "PSX
Dev-Board Chipsets", and the chapter headings of the kernel and CD controller pages; GitHub's API for sizes, licences,
commits, releases and languages; the search results named in §1. Seen as a run's output: Beetle PSX's single run
(§4), by the person who wrote this page.

## 7. Negative results

- No AccuracyCoin-like suite for the PlayStation (§1).
- No reference core runs without a BIOS but PCSX-ReARMed on its own kernel; Beetle PSX and SwanStation run only on
  OpenBIOS without one (§4).
- No reference exposes VRAM or SPU RAM through the standard calls (§4).
- No CXD1815Q datasheet (§2); no prebuilt `gpustat` (§1); no VHDL simulator (§5).
