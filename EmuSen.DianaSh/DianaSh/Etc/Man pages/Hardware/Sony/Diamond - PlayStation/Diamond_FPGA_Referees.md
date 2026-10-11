# Diamond_FPGA_Referees — the FPGA cores as black-box referees

*Written 2026-10-11.* The record of the open PlayStation FPGA implementations prepared as black-box referees for
Diamond's open disputes: the rule that governs what of them may be read, the cores and their pins, the simulator, how
each is driven, the calibration against ps1-tests' console data, the outputs dispute by dispute, and every file opened.
`Diamond_Plan.md` §1.5 is the plan this follows; `Diamond_Disputes.md`, `../BlackMoon-HW/BlackMoon_R3000.md` §8 and
`../BlackMoon-HW/BlackMoon_GTE.md` §12 hold the disputes.

Marks as in the plan: **[M]** measured on 2026-10-11 on the desktop (Ryzen 7 7700X, Fedora 44) with the tool named;
**[R]** read from the source named; **[A]** argued; **[C]** a choice.

**Status, 2026-10-11:** the simulator is installed, the three cores are cloned and pinned, their interface files read
and the PSX_MiSTer harness written. **Nothing has yet been compiled or simulated**: the step that builds the clones and
runs them was held for the tester's leave on the same day. §5 to §7 record what each run is to be and what it can and
cannot decide; they carry no FPGA output.

---

## 1. The access rule

*Decided 2026-10-11 by the tester; absolute.* For each FPGA core, only these may be read: its README and licence, its
directory and file names, the port declarations of its top-level entity or module (names, directions and widths; the
header and nothing below it), and the scripts and project files of its simulation folder (build scripts, file lists,
run scripts, and the testbench's top-level ports and its documented file-loading mechanism). No architecture body,
process, always block, state machine, constant table or comment explaining behaviour is read, the RTL is not searched
for behaviour, and no file is opened that driving the core does not need. Where driving a core needs more, the step
stops and says what would be needed. Every file opened is logged (§8). The cores' logic is never described in this
documentation set; what is recorded of them is their outputs and what their READMEs state.

The rule refines the plan's §1.5 (only the README, the licence, the directory names, the simulation folder's scripts
and the top level's port list) by naming the testbench's loading mechanism, which a simulation cannot do without.

## 2. The cores

| Core | Where | Commit | Licence | Language, simulator | What its README says |
|---|---|---|---|---|---|
| PSX_MiSTer | `github.com/MiSTer-devel/PSX_MiSTer`, cloned to `~/Projects/psx-mister-reference` | `cd17b5a0ad26` (2026-08-21), the head on 2026-10-11 and the commit §1.5 recorded; 564 commits | GPL-2.0 (`LICENSE`) | VHDL with one SystemVerilog file in the simulation's list; GHDL | requires a BIOS by region; "Fast Boot (Skips BIOS)"; status by unit, e.g. "Memctrl: register stubs only", "Timer: 90% - accuracy for dotclock and gates timer not tested", "GTE: 90% - CPU <-> GTE Transfer pipeline delay not fully correct", "GPU: 90%"; error overlays including "E8 - DMA chopping enabled" [R] |
| Laxer3a's PS-GPU | `github.com/Laxer3a/PS-GPU`, `~/Projects/ps-gpu-reference` | `c16b2b0640cd` (2021-05-08); 161 commits | GPLv2 or commercial, by the README (no licence file) | SystemVerilog and Verilog; Verilator | "All commands are supported"; GP1(00h) and GP1(01h) do not interrupt a primitive in flight; "UV coordinate sampling and color rounding are not perfectly identical to the original" [R] |
| Laxer3a's PS-GTE | `github.com/Laxer3a/PS-GTE`, `~/Projects/ps-gte-reference` | `289bcf316966` (2021-05-08); 30 commits | GPLv2 or commercial, by the README | SystemVerilog and Verilog; Verilator | "All commands are supported"; "Unsupported feature - None known" [R] |

Neither of Laxer3a's repositories has a simulation folder [R the file names]. Provenance, with full hashes, is in the
probe cache's `PROVENANCE.txt`; the clones, the simulator and the harnesses are never committed.

**What their agreement is worth** [A]. PSX_MiSTer and Laxer3a's units are separate lineages, and none of the three
READMEs documents testing against a console. One FPGA alone is weak evidence; two agreeing, moderate. Where a README
states a unit is incomplete, its output on that unit is a witness of nothing: by its own statement PSX_MiSTer's memory
control is register stubs (D-4), its timers' dot-clock and gate modes are untested (D-0, D-6), its GTE transfer delay
is "not fully correct" (D-13, D-15), and DMA chopping is a condition it reports as an error (D-5). Laxer3a's GPU is not
a witness for D-7, by its README's sentence on UV sampling and colour rounding.

## 3. The simulators

- **GHDL 6.0.0**, LLVM 22.1.8 back end, unpacked user-local in `~/.cache/emusen/toolchains/ghdl/` from Fedora 44's
  `ghdl-llvm`, `ghdl-llvm-grt` and `ghdl-grt` packages (the last carries the precompiled standard libraries) and
  `libgnat` 16.2.1, the one dependency the system lacked; `env.sh` there sets the paths. No system install. Its smoke
  test, a VHDL-2008 counter, ran [M]. The LLVM back end was chosen over mcode because it compiles to native code, which
  a full system's simulation needs. nvc was not needed.
- **Verilator 5.046**, already unpacked in `~/.cache/emusen/toolchains/verilator/` (Nephrite's).

## 4. How each is driven

**PSX_MiSTer** [C]. The harness is a testbench of the project's own around the core's top level, built from the
simulation folder's file list in its order. It loads a boot image and a PS-EXE into the simulation folder's own SDRAM
model by that model's documented script-loading mechanism (the model unmodified), asks for the PS-EXE to be loaded,
and releases reset, as the folder's own `bootexe` run script does. The boot image is the tester's own BIOS, passed by
path only and never read, copied or named. Results are read without reaching into the core: the harness watches the
core's RAM write port and keeps a shadow of the window the test programs write (`80100000h` up), and stops when the
program's done marker (`D1A0D00Eh` at `800FFFF0h`) is written or after a set number of vertical blanks; VRAM is read
from a memory model of the harness's own on the core's video-memory port, written from the bus convention that port
follows, and dumped at the end. The core's reports and the frame count go to the log. Runs are to be niced and
confined (`systemd-run --user --scope -p MemoryMax=8G`).

**What the harness cannot see** [A]. No TTY reaches the core's top level: its ports are the memories, video, sound,
pads, cards, the disc and the configuration. A program that reports only by printf, which is most of ps1-tests, cannot
be graded through it; ps1-tests' GPU programs can, by their VRAM pictures, and so can every program of Diamond's
`programs/`, which report in RAM. §7 names the questions this leaves.

**Laxer3a's GPU and GTE** [C]. Each is a unit with no system around it and no simulation folder, so the only interface
is the top-level module's port declarations, which are to be read when the harness is built (not yet opened). The GPU
is to be driven by GP0 and GP1 word streams and VRAM read back; the GTE by register writes and command words, with the
registers and the cycle at which the command reports done read back. Neither runs a PS-EXE: each program of §6 is
carried over as the word stream or the register sequence its CPU code issues.

## 5. Calibration against ps1-tests' console data

*Not run.* The planned calibration, on PSX_MiSTer before any dispute: ps1-tests build-158's GPU programs whose VRAM
pictures are a console's (`triangle`, `uv-interpolation`, `lines`, `quad`, `clipping`, `texture-flip`, `transparency`
and the rest of the 17), graded as Diamond's gate grades them (`Diamond_GPU.md` §9), with the two exclusions of D-12.
The TTY-graded programs (`timers`, `cpu/access-time`, `dma/chopping`, `gpu/bandwidth`) cannot be graded by the harness
(§4). The simulation's speed, and so which of these finish, is unmeasured.

## 6. The disputes

Diamond's outputs are those its entries already record; the console's are named where a ps1-tests log or picture holds
them. No FPGA output exists yet, so no entry gains a verdict here and `Diamond_Disputes.md` is not annotated.

| Dispute | Program and referee | Diamond's output | Console's data | FPGA output, verdict |
|---|---|---|---|---|
| D-0, the line length | a RAM-recording program counting CPU cycles, hblanks and dots across vblanks (none exists; ps1-tests' `timers` reports by TTY); PSX_MiSTer only | 571,303 cycles a frame on the PAL clock in NTSC mode, 566,121 NTSC [M, D-0] | `timers/psx.log`, a PAL-clocked console in NTSC mode | not run. The simulation folder's testbench supplies the video clock at the crystals' nominal ratio to the CPU's [R `tb.vhd`], so a run measures the core's line given that clock; its author read the same documents, so it carries no weight on a document dispute (D-0's own words) |
| D-2, multiply and divide | `programs/multiply.s`; PSX_MiSTer | psx-spx's 6, 9, 13 and 36, NOP table 6, 5, 4, 3, 2, 1, 1 [M] | none | not run |
| D-4, memory control | `cpu/access-time` (TTY) or a RAM-recording rewrite; PSX_MiSTer | 50 of 51 cells within 0.20 cycles [M] | `cpu/access-time/psx.log` | not run; README: register stubs only, so at most a witness for the kernel's values |
| D-5, DMA gaps and the drain | `dma/chopping`, `dma/chain-looping` (TTY); PSX_MiSTer | within 10.5% and 6.2% [M] | both logs | not run; chopping is an error condition by the README |
| D-6, timer sync mode 2 | `timers` (TTY) or a RAM-recording rewrite; PSX_MiSTer | the log's rows [M] | `timers/psx.log` | not run; gate modes untested by the README |
| D-7, the rasteriser | the five GPU pictures, then a program of shaded triangles at every vertex order; PSX_MiSTer only | exact on all five [M] | the five pictures | not run (the calibration of §5) |
| D-8, x-flip at U+1 | `texture-flip`, then x-flipped rectangles at odd and even columns and U; PSX_MiSTer and Laxer3a's GPU | U+1, U, U−1 [M] | `texture-flip`'s picture | not run |
| D-10, GPUSTAT bit 28 | `programs/gpustat.s` on PSX_MiSTer; the same word streams on Laxer3a's GPU with GPUSTAT read back | the log's twelve readings [M] | `gpu/gpustat/psx.log` | not run |
| D-11, busy after a burst | GPUSTAT bit 26 against cycles after bursts of N words; PSX_MiSTer and Laxer3a's GPU | idle where the console reads busy on 13 of 130 rows [M] | `dma/chopping`'s busy marks | not run |
| D-13, the GTE interlock | `programs/gte-timing.s` tables A, B, C, E on PSX_MiSTer; each command's cycles to done on Laxer3a's GTE | psx-spx's counts less one; every access waits [M] | none | not run; the README's sentence on the transfer delay weakens PSX_MiSTer here |
| D-15, GTE write delays | `gte-timing.s` table D on PSX_MiSTer; the same sequences on Laxer3a's GTE | the new value at once [M] | none | not run; as D-13 |
| D-16, unlisted numbers | `programs/gte-fuzz.s`'s 24 N/A cases and table E; PSX_MiSTer and Laxer3a's GTE | 00h as RTPS, the rest clear FLAG [M] | none | not run |

## 7. Open, and the questions it leaves

- **The build and the runs**, held for the tester's leave (Status). Once given: build the harness, run one Diamond
  program to fix the RAM port's addressing against the done marker, measure the simulation's speed, then §5 and §6 in
  that order, shortest programs first.
- **The TTY.** ps1-tests' timing programs, whose logs are the console's only timing data the project holds, report by
  printf. A black box shows no TTY, so either the project writes RAM-recording versions of them (as `gpustat.s` is of
  `gpu/gpustat`), or a harness-side program installs a TTY device of its own through the kernel's documented device
  calls before starting the test. The first keeps the BIOS untouched and is preferred [A]; it is a question for the
  tester which, if either. *2026-10-11:* the first was done for `timers`, `cpu/access-time`, `dma/chopping`,
  `gpu/bandwidth` and `dma/chain-looping` (`Diamond_Records.md`); §5's TTY-graded programs can be given to the harness
  as those records.
- **Laxer3a's units** need their top-level port declarations read when their harnesses are built (§4); nothing else.
- **The simulation's cost** is unmeasured; a full system in GHDL is expected to be slow, so the BIOS's own boot before
  the PS-EXE starts may bound what runs.

## 8. Reading log

Every file opened for this page, and which part.

- **PSX_MiSTer** (`cd17b5a`): `README.md` (whole); `LICENSE` (its first fifteen lines, the version); the directory and
  file names of the repository, of `rtl/` and of `sim/`; `rtl/psx_mister.vhd` lines 5–286, the entity's generic and
  port declarations, and the line numbers of `entity`, `end entity` and `architecture` in it and in `rtl/psx_top.vhd`
  (no line of either architecture); `sim/system/vcom_all.bat`, `vmap_all.bat` and `vsim_start.bat` (whole);
  `sim/system/tests/vsim_comm.lua`, `lib.lua`, `bootexe.lua`, `bootrom.lua` and `luareg.lua` (whole);
  `sim/system/src/tb/tb.vhd` (whole: the testbench, its signal declarations, the core's instantiation, the memory
  models' instantiations, the disc and memory-card stimulus processes and the command interpreter's instantiation);
  `sim/system/src/tb/globals.vhd` (whole: the loading signals); `sim/system/src/tb/sdram_model3x.vhd` lines 1–64 (the
  entity and the model's first declarations), 170–200 (a process's declarations) and 370–450 (the script-loading
  branch and the initial-file branch), and a search of it for its entity, generics, ports and file handling, lines
  only; the line counts of the testbench folder's files. Not opened: any other file under `rtl/` or `sim/`, the
  `.qsf`, `.qip`, `.sv` and `sys/` files, `sim/system/tests/boot_load_state.lua` and `savestate_test.lua`, and the
  simulation folder's `ddrram_model.vhd` (the harness's own model replaces it). The testbench's whole was read where
  the rule names its ports and its loading mechanism; nothing in it is the core's logic.
- **PS-GPU** (`c16b2b0`) and **PS-GTE** (`289bcf3`): `README.md` (whole) and the file names; GitHub's API for the
  licence field and the last push. No source file opened.
- **The project's own pages**: `Diamond_Disputes.md` (whole); `../BlackMoon-HW/BlackMoon_GTE.md` §6, §8 and §12;
  `../BlackMoon-HW/BlackMoon_R3000.md` §8; `Diamond_Plan.md` §1.5; `Diamond_Native.md`'s lines on the TTY and the
  development tools (§11.10–§11.11, §11.15); `Diamond_GPU.md`'s opening; `Diamond_References.md` §5, and with it on the
  same screen the opening paragraph of §6, a section outside what this step was to read, of no use here; the headers of
  the programs in `programs/`, and the first sixty lines of `gte-timing.s`.
- **Not opened**: any emulator's source, any BIOS or firmware (the tester's BIOS was located by its folder's listing
  only), psx-spx.
