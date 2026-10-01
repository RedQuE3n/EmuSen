# VenusRT_Native — the build record

*Started 2026-09-30, at stage 0.* `VenusRT_Plan.md` says what VenusRT is to be and how it is graded; this page records
what was built, stage by stage, what was measured, and where the plan turned out to be wrong. The plan's three
registers are kept: **measured** is a number taken here, on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10, Rust
1.98.1), with the tool named beside it; **argued** is reasoning with no measurement behind it; **predicted** is one of the
plan's P1–P7, retired where a measurement reaches it.

Nothing on this page describes C# Venus's internals. Where it describes the old core, it describes what can be seen
from outside it, as the plan does (§1.2).

---

## 1. Stage 0 (2026-09-30)

Stage 0 is the plan's §6 row 0, with three items brought forward into it: the crate skeleton (the plan put it in
stage 1), the fix for the probe's pacing (the plan only noted the defect), and the clone check (the plan's G8, at
stage 8). What it built:

- **The crate**, `EmuSen/Cores/Nintendo/VenusRT - SNES/`, on the common native interface from its first commit, with
  its own state format (§2.1–§2.2). No part of the console is emulated.
- **The single-step harness** over SingleStepTests' 65816 and SPC700 suites, with every bus cycle compared (§2.3).
- **The test-ROM runner** in EmuSen.WiseMan, generic over engines: C# Venus through `ICore` as a black box, VenusRT
  through the common interface, and Mesen through the probe; with each suite's own grading, a frame-by-frame
  differential of spaces, picture and sound, and the recorded baseline (§3).
- **The clone check** for G8, calibrated with positive and negative controls (§4).
- **The probe's pacing fixed** (§5).
- **anomie's documents**, six of eight, fetched and pinned outside the repository (§6).
- **`VenusRT_Disputes.md`**, created with no entries: nothing was implemented that could disagree with an oracle.

It wrote no emulation rule. The clean-room protocol of the plan's §1.2 applied throughout: no source of C# Venus
beyond its public surface, no algorithm section of its pages, and no Mesen or other emulator source was read. The
probe's fix read only the probe's own glue and Mesen's settings header (§5).

---

## 2. The crate

### 2.1 On the common interface

- **`NativeCore` and `native_exports!(Machine;)`**, with no optional group. `CAPABILITIES` is 0. The plan's §8 lists
  `RESET`, `MUTES`, `ROM_PATCHES`, `SETTINGS`, `PHASES` and later `DEBUG` as VenusRT's; each is claimed in the stage
  that implements it, because a library that claims an export it lacks is refused, and one that exports what it does
  not claim fails `VenusRTs_library_claims_nothing_it_does_not_export_and_its_state_is_its_own`.
- **The version** is `0x0001_0001`: common interface 1, core 1. The engine's name in the crash log is `VenusRT`.
- **Create.** The image is the file; a copier's 512-byte header is dropped by length modulo 1 KiB. An image shorter
  than one 32 KiB bank is status −9. No setting key is taken yet (`UNKNOWN_SETTING`). File 0, the battery save, is
  copied into the cartridge's RAM clipped to its length, which is 0 until the cartridge exists; a file numbered 1 or
  more is `BAD_FILE` until stage 5 makes firmware the console's files.
- **Spaces** are numbered in the order of the plan's §4.7: 0 `CpuBus`, 1 `IO`, 2 `WRAM`, 3 `VRAM`, 4 `CGRAM`, 5 `OAM`,
  6 `SRAM`, 7 `APURAM`. `CpuBus` and `IO` answer `NO_SUCH_SPACE` until stage 2 builds the bus; past a space's end
  reads zero and writes are dropped.
- **The picture** is 256×224 RGBA, black. **Sound** reports the S-DSP's nominal 32,000 Hz and has no samples.
- **The pads** take twelve bits in `PadButton` order and are in no state. Whether the auto-joypad's latched words
  belong in the state is stage 2's question, answered with its oracle.
- **What it is not:** no `ICore` shim and no row in `CoreCatalog`, which are stage 6's. `VenusMachine`, a
  `NativeMachine` over the library, exists so that the runners can drive the engine before then.
- **The build.** `RustCores.props` names the crate, so `EmuSen.csproj`'s generic cargo build and
  `RustCoresPublish.targets` pick it up with no edit of their own; `rust-cores.yml` builds and tests it on the four
  platforms, counts its `emusen_native_` exports, and runs the runner's tests in the WiseMan job.

### 2.2 The state, "VNRT"

Version 1, written with `emusen-native`'s `StateWriter`, as the plan's §5.6 decided: the magic `VNRT` (bytes
56 4E 52 54), the version, the frame count, then WRAM, VRAM, CGRAM, OAM and APU RAM, 263,216 bytes in all. The layout
listing is pinned by `the_version_1_layout_is_pinned`, because without a C# oracle the listing is the only record of
what a version means. A state with any other magic is refused as foreign (−3), and that includes C# Venus's, whose
magic is `SNES`; the message naming the engine that made it is stage 6's C# pre-check (plan §4.5). A refused or
truncated load changes nothing, which a test pins.

### 2.3 The single-step harness

**Where it lives.** In the crate, as unit tests over two traits (`singlestep::w65816::Cpu` and
`singlestep::spc700::Cpu`), so that stage 1's CPU implements the trait and runs where it is compiled. The suites'
files are read by `emusen_native::json`, a reader of about 300 lines with no dependency. None of the Rust cores takes
a dependency beyond `emusen-native`, and the offline cargo cache had no JSON crate; a reader written for the purpose
kept both true (measured: the whole of both suites, 2.8 GB of JSON, is read and graded in 16 s of wall time on four
threads). The corpus is found through `EMUSEN_VENUSRT_CORPUS`; without it the corpus tests pass unrun, as
`NecDspRealFirmwareTests` does without firmware.

**The 65816 protocol.** A flat 16 MiB bus that records every cycle as (address, data, pins). The pins are the suite's
eight, in its order: VDA, VPA, VPB, RWB, E, M, X, MLB. RAM answers only a cycle with VDA, VPA or VPB, as the suite's
own environment did, so an unselected read returns 0 and records no data. Each case is graded three ways, counted
apart: the registers, the memory the case lists, and the cycle list, compared whole.

**The SPC700 protocol.** A flat 64 KiB bus recording reads, writes and waits; the registers PC, A, X, Y, SP and PSW.

**Three readings of the suites, found by the positive control before any CPU existed.** Each is a statement about the
file format, settled by the suites' own data and README, not a statement about the hardware, so none is a dispute.

1. **WAI and STP halt the bus.** Every case of `cb` and `db`, in both modes, 40,000 in all, ends with cycles whose
   address and data are null and whose pins are eight dashes, a string outside the README's alphabet. The harness
   reads such a cycle as one in which the processor holds the bus idle, and the bus trait has a `halted` call for it.
2. **MVN and MVP stop at 100 cycles, mid-move.** 39,989 of their 40,000 cases record exactly 100 cycles, and a move
   repeats every seven, so the list ends two cycles into a repetition, with the program counter pointing into the
   instruction. The other 11 finish their move first. A CPU that stops only at instruction boundaries cannot reach
   the recorded final state, so for a capped case the harness steps whole repetitions until 100 cycles are recorded,
   compares those 100, and does not compare registers or memory; the case is counted as capped. *Argued:* this is why
   Pharaoh's state-only runner scored 0 of 10,000 on MVN and MVP (plan §3.5): its final state can only match a
   machine stopped two cycles into a repetition. A CPU that can stop on a cycle budget could be graded on state too;
   that is a choice for stage 1, and it would not change the cycle comparison.
3. **The SPC700 suite records dummy reads of memory a case does not list without their data.** In 60 of the 256
   opcode files, 60,000 cases, a read cycle has an address and no value: the reference environment answered reads of
   unlisted memory with nothing it recorded. Such a read constrains its address and kind only.

**The controls.**

- *Positive:* a replay processor that performs the suite's own cycles against the harness's bus and ends in the
  suite's final registers. Because its reads take their data from the harness's memory, a replay pass is a check that
  the harness loads, answers and records as the suite's environment did. **Measured:** it passes 5,120,000 of
  5,120,000 65816 cases (39,989 capped) and 256,000 of 256,000 SPC700 cases.
- *Negative:* four spoiled replays per suite, each run over the cases it applies to (every eighth 65816 file, the
  first 500 cases of each; the first 200 of every SPC700 file), and each must pass none. **Measured:**

| Spoiling | 65816 cases it applies to, passed | SPC700 cases, passed | Check that fails |
|---|---|---|---|
| The last cycle dropped | 32,000, 0 | 51,200, 0 | cycles (and memory, when it was a write) |
| One written byte's bit 0 flipped | 9,500, 0 | 19,400, 0 | cycles and memory |
| The carry flipped in the final state | 31,000, 0 | 51,200, 0 | registers |
| The first cycle's M pin flipped (65816) / a read replaced by a wait (SPC700) | 32,000, 0 | 51,200, 0 | cycles |

**What it does not cover.**

- **The S-CPU's bus.** The suite's environment is a bare 65816 on flat RAM. Open bus, the SNES's memory map and each
  cycle's length in master clocks (6, 8 or 12) are not in it; they are stage 2's, graded by test ROMs.
- **The SPC700's I/O page.** The suite's memory is flat, with no registers at `$F0`–`$FF`, so an instruction that
  touches them is graded as if they were RAM. Gate G2 already names this as a likely dispute.
- **Speed.** The harness says nothing about how fast a CPU runs.

**The baseline.** C# Venus's CPU cannot be driven through `ICore`, so the harness cannot grade it. The plan's
single-step column came from Pharaoh's existing `--singlestep`, which compares state only; it was rerun unchanged on
2026-09-30 and reproduced (§3.5). Mesen cannot be run over the suites through the probe, which runs ROMs.

---

## 3. The test-ROM runner

### 3.1 Engines

`EmuSen.WiseMan/Fixtures/Snes/`. An `ISnesEngine` runs a ROM from power-on to a list of frames, with an optional
schedule of presses on port 0 in the probe's `--press` form, and returns a snapshot at each: VRAM, CGRAM, OAM, WRAM
and APU RAM under the probe's lower-case names, and the picture; with the sound from power-on when asked.

- **`ICoreSnesEngine.Venus()`**: `CoreFactory.Load` and nothing else, the memories read through the bundle's
  `IDebugTarget`. This is the plan's black box (§1.6): no type of Venus's is named.
- **`VenusRtSnesEngine`**: VenusRT through `VenusMachine`.
- **`MesenProbeSnesEngine`**: the probe as a process, from the Mesen checkout, at a fixed stride. Each dump set is
  cached under the corpus's `runs/mesen/`, keyed by the ROM's MD5, the frames, the presses, whether sound was asked
  for and the probe binary's size and time, so that a rebuilt probe is rerun. The picture's size is taken from the
  probe's own report line for each frame (`PPU <w>x<h>`), because the screen file is always 512×478 and a normal
  frame occupies its start.

### 3.2 Pictures

Both sides become 15-bit BGR words: an RGBA picture by dropping each channel's low three bits, Mesen's as it is. The
top bit of a word is not compared. **The rows:** Mesen's picture is 239 lines and Venus's 224, and a 224-line
picture's row *r* is Mesen's row *r* + 7. *Measured by `The_picture_offset_is_the_one_steady_pictures_agree_at`* on
three steady pictures (the 240p suite's menu, gilyon's `spctest` and PeterLemon's `CPUADC`, at frame 3600): at offset
7 not one of the 57,344 compared pixels differs on any of them, and at offset 0 between 1,442 and 21,631 do. The
first value written for the constant, 1, was a guess, and the test is what replaced it. That C# Venus and Mesen draw
these three pictures identically says nothing about VenusRT; it says the comparison is aligned.

### 3.3 Grading

`SnesTestRomGrader` implements the last column of the plan's §3.1, as `grade.py` implemented it for the plan:

| Protocol | Reads | Verdicts |
|---|---|---|
| gilyon | the word at `$0032` begins `Success` or `Failed`; the word at `$006E` is the test number | Passed, Failed, Incomplete |
| blargg's SPC tests | the backdrop, CGRAM entry 0: blue passed, red failed | Passed, Failed, Incomplete |
| PeterLemon | `PASS`/`FAIL` in VRAM's low bytes, for a ROM whose source counts passes with `PrintText(Pass`; a pass needs VRAM unchanged since half the run | Passed, Failed, Incomplete |
| text | the last of `Passed`, `PASSED`, `Failed`, `FAILED`, `FAIL`, `OK`, `Done`, `Success` in VRAM's printable runs | Passed, Failed, Done |
| none | no verdict word | Visual: graded by picture or values against Mesen |

Frames: 1800 and 3600 for every ROM, and 9000 and 18,000 for blargg's `spc_dsp6`, which Mesen finishes near 10,800.
The ROMs are the corpus's own manifest, `unique-roms.txt`, one path per MD5: its choice among duplicates was made in
fetch order and cannot be recomputed by sorting (measured: sorting picks another path for 43 of the 78 duplicated
MD5s), and the grader reads the path.

The grader's protocols are pinned by tests over synthetic dumps, a pass, a failure and a silence for each.

### 3.4 The differential

`SnesDifferential.Compare(a, b)` compares two runs frame by frame: bytes differing in each space, and pixels
differing over the rectangle both pictures cover. `SnesDifferential.Audio` compares sound by the loudness envelope in
sixtieths of a second, the best Pearson correlation within half a second of lag, and whether either side is silent.

**What each comparison can say.** Memory and pictures across two emulators are comparable only where the plan's §1.3
allows: a self-grading ROM's result, a picture that has stopped changing, or an anchor. Two engines do not reach
frame *n* in the same state. The sound comparison is coarser still: Mesen's WAV is its mixer's output at its own rate
and VenusRT's will be the DSP's 32 kHz, so no sample comparison is meaningful; an envelope that correlates says the
two played the same notes at the same times, and nothing about the waveform. The DSP's own oracle is blargg's
`spc_dsp6` (plan §3.2). The controls: a run compared with itself is identical at every frame, one byte and one pixel
changed are counted as one each, and a tone delayed by five windows is found at lag 5 with a correlation above 0.95.

### 3.5 The baseline

Run on 2026-09-30 by `The_corpus_reproduces_the_recorded_baseline` (opt-in through `EMUSEN_VENUSRT_CORPUS`), over the
295 ROMs of the manifest, C# Venus and Mesen each, six at a time. The table it writes is committed as
`EmuSen.WiseMan/Cores/VenusRtBaseline.tsv`, one row per ROM: its MD5 and path, its suite, each engine's verdict and
detail, whether the two engines' VRAM agrees at the last frame, and whether Mesen's VRAM stood still between the two
frames. The test fails when a later run's verdicts differ from it. **These are the old core's results for comparison,
not expected values for VenusRT** (plan §3.5).

**It reproduces the plan's baseline.** Venus's verdict equals the plan's `grade.py` verdict on all 295 ROMs.
Mesen's equals it on 294; the one that differs is `spc_dsp6`, which the plan found passing at frame 10,800 by a
separate run and the runner now runs to 18,000. A second run gave a table identical in every cell, with Venus rerun and
Mesen rerun on the fixed probe (§5).

| Suite (by path) | ROMs | Venus passed | Venus failed | Mesen passed | Mesen failed | Mesen no verdict |
|---|---|---|---|---|---|---|
| gilyon `cputest` | 2 | 0 | 2 | 2 | 0 | 0 |
| gilyon `spctest` | 2 | 2 | 0 | 2 | 0 | 0 |
| PeterLemon CPU | 23 | 23 | 0 | 23 | 0 | 0 |
| PeterLemon SPC700 | 7 | 7 | 0 | 7 | 0 | 0 |
| PeterLemon GSU | 31 | 31 | 0 | 31 | 0 | 0 |
| blargg SPC (`spc_smp`, `spc_timer`, `spc_mem_access_times`) | 3 | 0 | 3 | 3 | 0 | 0 |
| blargg `spc_dsp6` (to frame 18,000) | 1 | 0 | 1 | 1 | 0 | 0 |
| blargg 2010 | 15 | 3 | 10 | 10 | 1 | 4 |
| ADC/SBC | 6 | 6 | 0 | 6 | 0 | 0 |
| multiply/divide | 7 | 0 | 6 | 5 | 1 | 1 |
| absindx SA-1 | 2 | 0 | 2 | 1 | 1 | 0 |
| Cx4 | 1 | 0 | 1 | 0 | 0 | 1 |
| undisbeliever | 137 | 0 | 0 | 0 | 0 | 137 |
| 240p suite | 2 | 0 | 0 | 0 | 0 | 2 |
| higan collection, other | 56 | 1 | 0 | 1 | 0 | 55 |
| **All** | **295** | **73** | **25** | **92** | **3** | **200** |

"Failed" counts a failure and an unfinished run together. The suite labels are the path rules of
`SnesTestRomCorpus.Suite`; the plan's §3.5 grouped some ROMs by hand, so its row counts differ (its ADC/SBC row has 7,
its multiply/divide row 5), while the totals agree.

**Of the 92 ROMs Mesen passes, Venus passes 73, and Venus passes none that Mesen does not.** The 19: gilyon's two
CPU tests (at `0024`, `adc ($EF,x)`, and `02BD`), blargg's four SPC tests, seven of the 2010 tests, five
multiply/divide tests and the SA-1 RAM-protection test. The plan counted 93
because it graded the Cx4 memory test by hand as a pass on Mesen; to the runner it prints no verdict word.

**The ROMs with no verdict** (200 on Mesen's side) are graded by picture or values. Mesen's VRAM stood still between
frames 1800 and 3600 on 176 of them, and on 157 of those Venus's VRAM at the last frame is byte-identical to Mesen's.
The plan counted 172 standing and the same 157 identical; it counted over its 196 rows graded `TEXT`, the runner over
every row without a Mesen verdict, which adds `Done` and the ROMs Venus fails and Mesen does not grade.

**The single-step suites**, C# Venus through Pharaoh's `--singlestep`, rerun unchanged: SPC700 255,105 of 256,000 (1.9
s), 65816 2,815,936 of 5,120,000 (724 s): native 2,539,998 and emulation 275,938, with MVN and MVP 0 of 10,000 each
and `C0` and `C4` 9,999 in native mode, all as the plan recorded. State only; §2.3 explains the MVN and MVP zeros.

---

## 4. The clone check

### 4.1 Method

`EmuSen.WiseMan/Reference/analysis/clonecheck.py <audited> <reference>`. Three signals, reported apart:

- **Structure.** Both trees are reduced to one token vocabulary. Identifiers become `ID`; types, casts, parentheses,
  statement ends, `self`/`this` and their member access disappear; a member chain `a.b.c` is one `ID`; the control
  words of C++, C# and Rust map onto one set; spellings that differ only by language are folded (`~`/`!`, `++`/`+= 1`,
  C's truthiness `!= 0`, `case`/`break` against match arms, implicit returns, type ascriptions); numbers keep their
  value. Fingerprints are winnowed 16-token k-grams in windows of 8 (Schleimer, Wilkerson and Aiken's winnowing, the
  method of MOSS). A file pair is reported when it shares at least 12 fingerprints; its score, the share of the
  smaller file's fingerprints it holds, is printed beside it.
- **Tables.** Runs of eight or more numeric literals with at least four distinct values, matched by value. A table a
  document also prints (fullsnes prints several) is expected, and the reader checks each hit against the document.
- **Names.** The reference's identifiers of six or more characters, with case and underscores folded, that the
  audited code also uses, less the words of the vocabulary files given (the hardware documents, the other cores).

Only the audited side is ever printed. The reference side appears as file names and scores, so that a writer bound
by the protocol can run the tool against sources they may not read, and did so here.

### 4.2 Calibration (measured 2026-09-30)

**Why the positive control is not a translation of Mesen.** A hand translation of Mesen's code would have meant a
writer reading Mesen's source, which the plan's §1.2 forbids, and the protocol is the thing G8 checks. The controls
were therefore built from code that may be read: a small C++ interpreter of an invented 8-bit machine, written for
the purpose (108 lines), and its hand translation into Rust (98 lines) with every identifier renamed, the switch
turned into a match, the member accesses restructured and the casts moved. Both are in scratch, outside the
repository. The C++ file was planted in a copy of Mesen's `Core/SNES/` (165 files, copied without being opened), and
the Rust one in a copy of MercuryRT's sources (20 files), so that the tool had to find the pair among 3,300.

| Control | What it is | Result |
|---|---|---|
| Positive, whole | the translation among MercuryRT's files, against Mesen's SNES tree with the original among it | the only pair reported: score 0.721, 44 fingerprints shared; its 32-value timing table also found |
| Positive, partial | only the translated `execute`, 46 lines, appended to MercuryRT's largest file | reported: 0.295, 18 shared |
| Negative | MercuryRT's sources against Mesen's SNES tree, nothing planted | no pair; the most any pair shares is 4 fingerprints; no table |
| Natural port | MoonRT (Rust) against C# Moon, of which it is a line-for-line port | 4 of MoonRT's 16 files reported (77, 24, 22 and 20 shared) |
| Same project | MoonRT against MercuryRT, two independent cores on one framework | 6 files reported, all the framework's idioms (the interface's `ffi`, the state walk) |

**The threshold** of 12 shared fingerprints is three times the negative's maximum and below the partial plant's 18.
k was chosen by the translated pair's score: 0.80 at k=8, 0.75 at 10, 0.79 at 12, 0.72 at 16; 16 is the longest k
that keeps the score above 0.7, and a longer k is less likely to match by chance.

**What the calibration shows the tool cannot do.**

- **A port that restructures escapes it.** C# Moon to MoonRT is a real port, and only a quarter of its files are
  found: where the Rust splits or merges functions, or turns a C# method into a macro or a table, the token order
  changes. The tool finds a translation that keeps statement order, which is what copying from a reference usually
  looks like, and not a rewrite that happens to share its rules. That second thing is what the disputes log and the
  protocol are for, not this tool.
- **Two cores on one framework look alike.** Within the project the tool reports shared idioms, so it is for a core
  against another project's sources only.
- **The names signal is a reading list, not a verdict.** Against Mesen, MercuryRT uses 13 of Mesen's identifiers
  beyond the documents' and the other cores' words: `rombank`, `readram`, `divcounter`, `supergameboy` and the like,
  the ordinary names of the things. A name only an emulator uses is the finding the plan's §1.2 means; telling it
  from an ordinary name takes a reader.

**Not run against VenusRT**, which has no code yet. At the gates it is run as
`clonecheck.py 'EmuSen/Cores/Nintendo/VenusRT - SNES/src' ~/Projects/mesen-reference/Core/SNES --vocabulary <fullsnes> --vocabulary <anomie> --vocabulary <the other RT cores>`.

---

## 5. The probe's pacing

Recorded in full in `EmuSen_Debugging_Tools_Reference_v5.md` §3.59. In short: the probe set Mesen's maximum-speed
flag, and Mesen's loop is paced by a separate setting, the configured emulation speed, which defaults to 100 per cent.
Setting it to 0, unlimited, through `EmuSettings::SetEmulationConfig` took 600 SNES frames from 10.11 s of wall time to
0.96 s, with every space and the screen byte-identical. The fix touched only the probe's own C ABI file; what was read
of Mesen was `SettingTypes.h`'s `EmulationConfig` and `EmuSettings.h`'s accessors, both configuration, and a context
line of the probe's own frame-stop patch. `build-probe.sh` gained a sentinel so an already-patched checkout is
refreshed.

**The corpus found a second defect the pacing had hidden.** Dumped again unpaced, all 2,270 memory dumps of the 295
ROMs equalled the paced dumps, and 42 of 590 screens did not, on 24 ROMs whose pictures move every frame. The first
screen of an unpaced run is a stale picture while memory is current; a frame run on its own after a park is right.
`probe_run_until` now runs to the frame before its target, parks and runs the last frame alone. *Measured after it:*
every memory dump and every screen equals the paced dump except the two screens of one ROM, `inidisp_extend_vblank`.
That ROM's screen differs between two runs of the same command on rows 90 to 174, the band it keeps in forced blank,
while all five memories are identical: the rows Mesen does not draw keep what its buffer last held. Its picture is
therefore not a differential oracle, and its memory is. The whole second corpus run, C# Venus and Mesen together to
frame 3600, took 9 min 57 s on six threads in a Release build; the plan's paced Mesen run alone took about 25
minutes on fourteen, by its dumps' times.

---

## 6. anomie's documents

Fetched on 2026-09-30 from the Wayback Machine's captures of 2017-07-31, because romhacking.net answers a script
with a challenge page. Into `~/.cache/emusen/probe/venusrt/docs/anomie/`, with `SHA256SUMS`, never committed:

| Document | Subject | Revision | SHA-256 (first 16) |
|---|---|---|---|
| 191 | S-DSP | 1212 | `21b793b10a70ac87` |
| 193 | memory mapping | 1160 | `955ad18c3336fb4c` |
| 194 | open bus and wrapping | 1126 | `ce11ce8fc18179ab` |
| 196 | registers | 1157 | `19dfdf3f1c206655` |
| 198 | SPC700 cycles | 1126 | `6c6825c467a5fb1b` |
| 199 | timing | 1160 | `a26b6a8899146601` |

**Not obtained:** 195 (ports) and 197 (SPC700, with the boot ROM). Every capture of either replays as a page rather
than the file, including the two whose recorded type is not HTML. Both subjects are covered in fullsnes; the boot ROM
itself is the player's dump or the SNESdev wiki's description, a question for stage 4.

---

## 7. What the plan got wrong, found at stage 0

- **The single-step suites' formats** (§2.3): the plan's §3.1 described both suites as "JSON before/after state and a
  per-cycle list", which is right for most cases and wrong for three groups: WAI and STP's halted cycles, MVN and
  MVP's 100-cycle cap, and the SPC700's valueless dummy reads. The cap is the likely explanation of Venus's 0 of
  10,000 on MVN and MVP in §3.5.
- **The probe's pacing** was a missing setting, not an unknown pacer (§5).
- **The count of self-grading ROMs.** 92 by the grader's rules, not 93: the plan's 93 includes a hand grading of the
  Cx4 test (§3.5). Its 172 standing pictures are 176 when every ROM without a Mesen verdict is counted.
- **The probe's screens**, which the plan used for its pictures: the paced probe's were right, and the first unpaced
  fix alone would have made 42 of them stale (§5). One ROM's screen is not reproducible on Mesen at all.
- **Stage 0's contents.** The plan put the crate skeleton in stage 1; it was built here, because the runner needs an
  engine to drive and the build's wiring is cheapest to test before there is code in it.

## 8. Negative results

- No JSON crate was available offline, and none was added (§2.3).
- anomie's documents 195 and 197 could not be obtained (§6).
- The clone check finds a restructured port only in part (§4.2).
- The probe's pacing was measured only on the SNES before the fix.
- One ROM's picture, `inidisp_extend_vblank`'s, is not reproducible on Mesen between runs (§5).

## 9. What stage 0 does not cover

- No emulation. Every VenusRT verdict in the runner is Visual or a failure, which is correct for a blank machine and
  says nothing yet.
- The runner has no engine for Pharaoh's traces or Hotaru's dumps (plan §4.8), which stage 9 decides.
- The speed budget of the plan's §5.5 is not re-measured: no timing was taken at stage 0.

---

## 10. Stage 1, step 1: the 65816's data instructions (2026-09-30)

### 10.1 What it built

`src/cpu/`: the 65C816's registers, flags, the M and X widths, emulation mode and the rules it imposes (M and X
forced, the stack in page 1, the indexes' high bytes cleared with X set), the decode, every data addressing mode, and
these instruction groups: loads, stores (STA, STX, STY, STZ), transfers (with XBA and XCE), the flag instructions
(with REP and SEP), add and subtract in binary and decimal, the compares, the logic group with BIT, the
read-modify-write group (shifts, rotates, INC, DEC, TSB, TRB, on the accumulator and on memory) and the index
increments. Written from WDC's W65C816S datasheet (March 13, 2024; fetched into the corpus's `docs/wdc/` and pinned),
whose Table 5-7 lists every cycle of every mode with its address and pins, and from anomie's wrapping document (194)
for the bank and page carries; decimal mode from the arithmetic as Bruce Clark's tutorial states it; fullsnes for its
notes on wrapping and the dummy write; the suite's own README for its pins. Nothing else was read. The remaining 44 opcodes (§10.7) set `unimplemented` and do nothing.

### 10.2 The interface the stage-2 bus implements

`cpu::Bus` has three calls, one per cycle: `read(address, pins)`, `write(address, value, pins)` and
`idle(address, pins)`. The pins are VDA, VPA, VPB, MLB, E, M and X as one byte; RWB is which call it is. The CPU knows
nothing of time: the bus decides what each cycle costs (6, 8 or 12 master clocks, or 6 for an internal cycle) and
advances the master clock by it, which is §5.2's design. The CPU is generic over the bus, so each call is a direct,
inlinable call with no boxing and no table (§10.5 prices it).

**One write is not a memory write.** In emulation mode the modify cycle of a read-modify-write instruction is a write
with VDA low (Table 5-7's cycle with VDA 0 and VPA 0, and note 17: RWB low). The single-step suite records it so, with
the old value on the data bus and RAM unselected. fullsnes says the 65C816 does not reproduce the 6502's dummy write
(its "CPU Glitches"). The two agree once VDA is read: the cycle drives RWB low but selects no memory. The stage-2 bus
therefore treats a write without VDA as an internal cycle (argued; stage 2 checks it against a ROM if one exists).

### 10.3 Decimal mode

ADC adds digit by digit, adding 6 to a digit above 9 and carrying; V is taken from the sum before the top digit's
adjustment. SBC subtracts digit by digit, subtracting 6 from a digit that borrowed; V and C are the binary
subtraction's and the last borrow's. *Measured:* these passed every decimal case of the suite on the first run, in
both widths and both modes, including the cases with digits above 9, which the suite draws at random.

### 10.4 The grade, by group (measured 2026-09-30)

`singlestep::cpu65816::tests::the_cpu_through_the_whole_suite`, every case of all 512 files, 3.3 s on six threads. A
case passes when its registers, its listed memory and its whole cycle list (address, data and all eight pins) match.

| Group | Native mode | Emulation mode |
|---|---|---|
| load (LDA, LDX, LDY) | 250,000 / 250,000 | 250,000 / 250,000 |
| store (STA, STX, STY, STZ) | 240,000 / 240,000 | 240,000 / 240,000 |
| transfer (with XBA, XCE) | 140,000 / 140,000 | 140,000 / 140,000 |
| flag (with REP, SEP, NOP) | 100,000 / 100,000 | 100,000 / 100,000 |
| add/subtract (ADC, SBC) | 300,000 / 300,000 | 300,000 / 300,000 |
| compare (CMP, CPX, CPY) | 210,000 / 210,000 | 210,000 / 210,000 |
| logic (ORA, AND, EOR, BIT) | 500,000 / 500,000 | 500,000 / 500,000 |
| read-modify-write, index increments | 380,000 / 380,000 | 380,000 / 380,000 |
| **implemented, all** | **2,120,000** | **2,120,000** |
| later steps (44 opcodes) | 0 / 440,000 | 0 / 440,000 |

The test fails if any implemented group falls below all of its cases. None of the suite's three known quirks
(§2.3) touches these groups: WAI and STP, MVN and MVP are later steps, and the SPC700's valueless reads are the other
suite's. A fourth reading was needed, and like the other three it is about the file format:

4. **Emulation-mode cases start with a stack high byte no 65816 can hold.** Every `.e` case's initial S has a random
   high byte, and every final S has `$01` there. On the first run every emulation-mode file failed on that alone. The
   harness now loads S as an emulation-mode 65816 holds it, `$01` and the low byte. *Measured:* the first failure of
   `ff.e.json` on the first run was `s: got F638 want 138`, which is word for word the failure §3.5 of the plan quotes
   from Pharaoh's run of C# Venus. So the 238 emulation-mode files Venus failed there are, at least in the first
   failure each shows, the harness adapter's loading rather than Venus's CPU (argued from the identical value; Venus
   was not rerun with the loading changed, which would need its internals).

**The two failures that remained after the first runs**, and what fixed them:

- The emulation-mode modify cycle was first written with VDA high, so it wrote RAM; all 380,000 read-modify-write
  emulation cases failed on the cycle's pins (§10.2).
- One case, `e1 e 8669`, read a direct-page pointer's second byte from the wrong page. That is dispute D-1 in
  `VenusRT_Disputes.md`, settled for (d,X) at the documents and left open for (d) and (d),Y, which the suite never
  exercises.

### 10.5 The CPU's own cost (measured 2026-09-30)

`examples/cpu_cost.rs`: the CPU over a flat 64 KiB bus whose only work is to add 8 master clocks an access and 6 an
internal cycle, through a straight-line program of implemented opcodes with random operands that wraps within its
bank (REP, SEP and XCE left out, so the widths stay as drawn). Three runs of 3 s each under the timing lock, load
average 0.9: **3.85–3.87 ns a bus cycle, 14.7–14.8 ns an instruction**. At the program's 7.66 master clocks a cycle,
an NTSC frame's 357,368 master clocks are about 46,700 cycles, or **0.18 ms**: the first number against §5.5's 0.8 ms
budget for the CPU, the bus and DMA together. *Not covered:* the real bus's decode and its events, which stage 2 adds
to every call; branches and calls, which change the instruction mix; and any game's mix. No optimisation was done.

### 10.6 The clone check (measured 2026-09-30)

`clonecheck.py` over VenusRT's sources (11 files) against Mesen's `Core/SNES` (164 files), with the vocabulary of
fullsnes, anomie's documents, the datasheet's text and the other Rust cores: **no pair shares 12 fingerprints, no
table of eight literals is shared, and no identifier of Mesen's beyond the vocabulary's is used.** The largest pair
below the threshold shares 4 fingerprints (`cpu/mod.rs`), the same as the calibration's unplanted negative (§4.2).
Convergence on documented behaviour was expected, and at this threshold none showed; the tool's limits of §4.2 apply.

### 10.7 What is left of stage 1

The 44 opcodes: the branches (nine relative, BRL), the jumps and calls (JMP in four modes, JML, JSR in two, JSL, RTS,
RTL), the stack (PHA, PHX, PHY, PHB, PHD, PHK, PHP and their pulls, PEA, PEI, PER), BRK, COP and RTI with the hardware
interrupt sequence, MVN and MVP under the suite's cycle cap, WAI and STP with their halted cycles, and WDM; then
gilyon's `cputest-full` on a minimal bus with the VRAM port only, the plan's other stage-1 oracle. *Predicted:* the
first step took about a third of its budget, so these fit in one more step, and stage 1 takes two steps against the
plan's four; P3's estimate for stage 1 is retired at its end.

---

## 11. Stage 1, step 2: the rest of the 65816 (2026-09-30)

### 11.1 What it built

`src/cpu/control.rs`: the 44 remaining opcodes, from Table 5-7 and the datasheet's §7: the branches and BRL; JMP in
its four modes, JML, JSR in two, JSL, RTS, RTL; the pushes and pulls, PEA, PEI and PER, with the emulation-mode stack
of the 6502 for the old opcodes and the 16-bit addressing of S for the 65816's own (§7.1's list, with PLB added by
D-3), S put back in page 1 after each; BRK, COP and RTI, and RTI's pulled P applied only after its last pull, which is
when the suite's pins show it; MVN and MVP, one repetition a step with the opcode fetched again; WAI and STP; WDM, which
skips its second byte without reading it. `Cpu::interrupt` is the hardware sequence of row 22a for RESET, ABORT, NMI
and IRQ, which the machine calls between instructions when it takes a line: the dropped opcode fetch, an internal
cycle, PC (not advanced) pushed with the bank in native mode and P with B clear in emulation mode, the vector read
with VPB; a reset reads the stack instead of writing it and conditions the registers as §2.25 lists, and any
interrupt ends WAI. Four unit tests pin those sequences, which the single-step suite does not exercise.

**The bus gained `halted`**, a cycle in which WAI or STP holds the processor. A halted CPU's `step` is one such cycle.
The suite records exactly one after the instruction's three; the harness steps WAI and STP, like the block moves,
until the case's cycle count is reached.

### 11.2 Three readings of the datasheet, and a fourth source

The datasheet's §7, which step 1 had not read, says which direct modes leave the page in emulation mode (§7.2) and
which opcodes leave the stack's page (§7.1). Both disagreed with the single-step suite in places, which opened D-1
again and opened D-2 and D-3. gilyon's `cputest-full`, a test ROM written for exactly these behaviours (its README
names two of them as undocumented: PLB at S=$01FF and (d,X) with DL nonzero), then settled D-1 and D-3 at the
protocol's second rung. It also answers the question whether a new opcode could settle D-1's (d) and (d),Y cases:
**PEI could not**, because §7.2 names PEI as an exception (the suite's one crossing PEI case carries, as §7.2 says);
**the ROM did**, with 26 tests of (d) and (d),Y in emulation mode with DL zero, all wrapping within the page.

| Entry | Rule | Settled by | Single-step cases left as named exceptions |
|---|---|---|---|
| D-1 | (d,X)'s pointer wraps within its page in emulation mode whatever DL is; (d) and (d),Y wrap with DL zero; [d], [d],Y and PEI carry | gilyon `cputest-full`, 64 tests | 265: every emulation-mode (d,X) case whose pointer starts at a page's last byte |
| D-2 | JMP (a,X) and JSR (a,X) read their pointer with VPA, as Table 5-7 gives | open; unobservable on the SNES bus (argued) | 40,000: the two pointer reads' VDA and VPA |
| D-3 | JSR (a,X) and PLB address S in 16 bits in emulation mode | gilyon `cputest-full`, tests 0277 and 03D9 | 43 JSR (a,X) cases at S=$0100, inside D-2's |

### 11.3 The grade, by group (measured 2026-09-30)

All 512 files, 5,120,000 cases, registers, memory and every cycle; the test takes about 18 s on six threads with the
exceptions' files graded a second time:

| Group | Native | Emulation |
|---|---|---|
| load | 250,000 / 250,000 | 249,972 / 250,000 (D-1) |
| store | 240,000 / 240,000 | 239,968 / 240,000 (D-1) |
| transfer | 140,000 / 140,000 | 140,000 / 140,000 |
| flag (with REP, SEP, NOP, WDM) | 110,000 / 110,000 | 110,000 / 110,000 |
| add/subtract | 300,000 / 300,000 | 299,922 / 300,000 (D-1) |
| compare | 210,000 / 210,000 | 209,970 / 210,000 (D-1) |
| logic | 500,000 / 500,000 | 499,903 / 500,000 (D-1) |
| read-modify-write, index increments | 380,000 / 380,000 | 380,000 / 380,000 |
| branch | 100,000 / 100,000 | 100,000 / 100,000 |
| jump, call, return | 80,000 / 100,000 (D-2) | 80,000 / 100,000 (D-2, D-3) |
| stack (with PEA, PEI, PER) | 160,000 / 160,000 | 160,000 / 160,000 |
| BRK, COP, RTI | 30,000 / 30,000 | 30,000 / 30,000 |
| block move (MVN, MVP) | 20,000 / 20,000 | 20,000 / 20,000 |
| WAI, STP | 20,000 / 20,000 | 20,000 / 20,000 |
| **All** | **2,540,000 / 2,560,000** | **2,539,735 / 2,560,000** |

**5,079,735 pass and 40,265 fail, and the 40,265 are exactly the named exceptions.**
`the_cpu_through_the_whole_suite` checks that case by case: each exception must fail, every other case of its file
must pass, and D-2's files must pass in full once the two pointer reads' VDA and VPA are exchanged. So no failure is
unexplained, and none is hidden in a group's total. The suite's quirks, as the harness treats them:
- WAI and STP: stepped until the case's cycle count, the halted cycle compared (all 40,000 pass);
- MVN and MVP: stepped whole repetitions; the 39,989 capped cases compared on their 100 cycles only, the 11 that
  finish compared on everything (all 40,000 pass);
- unlisted dummy reads: an SPC700 matter, not this suite's;
- the emulation-mode stack high byte: loaded as the chip holds it (§10.4).

### 11.4 C# Venus's emulation-mode result, read again (argued)

Plan §3.5 records C# Venus passing 275,938 of 2,560,000 emulation-mode cases through Pharaoh's runner, with 238 of 256
files failing, the first failures showing "the stack's high byte kept rather than forced (s: got F638 want 138)", and
the cause not examined. *Argued:* the failures are most likely the adapter's loading of the suite's initial S, not
Venus's CPU. The evidence:
- every emulation-mode case of the suite starts with a random stack high byte, which no 65816 in emulation mode can
  hold, and ends with $01 there (§10.4, reading 4);
- VenusRT's own harness, before it loaded S as the chip holds it, failed `ff.e.json` first with `s: got F638 want 138`,
  the identical values;
- once S was loaded so, every emulation-mode case of the groups then implemented passed.

Not covered: Venus was not rerun with the loading changed, because that needs its internals (plan §1.2), so the share of
its 238 files that would still fail is unknown. Its gilyon failure at test 0024, `adc ($EF,x)`, is a separate matter:
that is a D-1 case, decided by the ROM, and says Venus's (d,X) rule differs from the hardware's.

### 11.5 gilyon's CPU tests (measured 2026-09-30)

`cputest::gilyons_cpu_tests_on_the_minimal_bus`: the CPU over a bus with only what the ROM uses. That is LoROM, WRAM
and the VRAM port ($2115 to $2119) the ROM writes its text through; $4210's vblank flag on a frame of 357,368 master
clocks, which `wait_for_vblank` polls; and $4212 and $4218 for the press it waits for after a failure. Its DMA only
clears and fills VRAM, so it is left out, and every other register reads 0 and drops writes. The grader reads the
text the plan's §3.1 describes (word $0032, `Success` or `Failed`; word $006E, the test number). When the ROM asks
for a key after a failure, the runner records the number and presses A, so that one run lists every failing test.

| ROM | Tests | Before the rulings | Now |
|---|---|---|---|
| `cputest-basic` | 452 | all pass, 112 frames | all pass, 112 frames |
| `cputest-full` | 649 | 13 fail: 0027, 0074, 015F, 0194, 0216, 02CC, 0397, 046F, 04E0, 0518, 05D2, 0621, all `(d,X)` with DL nonzero, and 03D9, `plb` | all pass, 162 frames |

C# Venus fails both (at 0024 and 02BD) and Mesen passes both (plan §3.5).

### 11.6 The clone check, over the whole CPU (measured 2026-09-30)

13 files of VenusRT against Mesen's 164 in `Core/SNES`, with gilyon's sources added to the vocabulary: **no pair
shares 12 fingerprints and no table is shared.** The largest pair below the threshold shares 4 (`cpu/mod.rs`), the
calibration's negative level. Two of Mesen's identifiers that are in neither the documents nor the other cores are
also VenusRT's, `nmiflag` (the minimal bus's field in `cputest.rs`) and `softwareinterrupt` (a method in
`cpu/control.rs`). Both are ordinary names for what they name: the datasheet calls BRK and COP software interrupts,
and $4210's bit is the NMI flag in fullsnes (whose spelling, "NMI Flag", the vocabulary does not fold into one word).
They are recorded here as the reading list §4.2 says the names signal is, not as findings.

### 11.7 The cost, with branches and calls in the mix (measured 2026-09-30)

`examples/cpu_cost.rs` now runs every opcode but REP, SEP, XCE, WAI and STP, and the block moves unless asked for
(a move counts as one instruction a repetition). A random program with branches falls into loops that the host's
predictors learn, so the program restarts at a pseudo-random address every 32 instructions; a store that writes WAI
or STP into the program is counted and cleared (about 0.18 per cent of instructions). Three runs of 3 s under the
timing lock, load average 0.9: **1.33–1.36 ns a bus cycle, 8.0–8.2 ns an instruction, 6.0 cycles an instruction**, so an
NTSC frame's cycles cost **0.062–0.063 ms** against §5.5's 0.8 ms for the CPU, the bus and DMA.

**The number moved, and the cause was not found.** Step 1 recorded 3.85 ns a cycle (§10.5). A build of step 1's
commit, timed interleaved with this one under the lock, reproduces 3.83–3.86 ns; this build runs step 1's own mix
(its opcodes only, straight-line) at 1.47 ns. So the same kind of program costs 2.6 times less on this build. The
two builds differ in what the opcode dispatch's last arm does: an assignment in step 1, a call into the control
instructions now. That this changed how LLVM compiled the dispatch is argued, not measured. Optimisation is on hold,
so it was not pursued. What it does show: at this stage a single number for the CPU's cost is a statement about one
build's code generation as much as about the design. The budget is checked again at stage 2 with the real bus, and
that check, not this one, is what §5.5 needs.

### 11.8 Stage 1 is complete

The 65816 is complete: every opcode, every addressing mode and the interrupt sequence. It passes all 5,120,000
single-step cases but the named exceptions, and both of gilyon's CPU ROMs in full. Stage 1 took two steps against the
plan's four. P3 is not retired, because the plan predicted stages 3 and 5, not this one, as the likely overruns.

**Stage 2's first step**, the bus and the memory map:
- the cartridge header, its scoring, and LoROM and HiROM with their mirrors and SRAM;
- WRAM and its port ($2180 to $2183);
- the access speeds (6, 8 or 12 master clocks by region and $420D's FastROM bit);
- the open bus, the MDR;
- the I/O registers' decode;
- the scheduler's master clock that every `Bus` call advances;
- the write without VDA of §10.2 treated as an internal cycle;
- `halted` cycles while WAI waits.

Its oracles are gilyon's `cputest-full` on the real bus in place of the minimal one, the single-step suite unchanged
through the real bus where it can be (flat memory is the suite's model, so only the CPU tests carry over), and the
memory-mapping tests of the higan collection (`memtest`). DMA, HDMA, NMI, IRQ, the multiply and divide unit and the
auto-joypad follow in stage 2's later steps.

---

## 12. Stage 2, step 1: the bus and the memory map (2026-09-30)

### 12.1 What it built

Written from fullsnes ("SNES Memory Map", "SNES Memory Control", "SNES Memory Work RAM Access", "SNES Timings",
"SNES Cartridge ROM Header") and anomie's document 194 (open bus); nothing else was read, except Mesen's message-log
API for the probe (§12.3).

- **`cart.rs`**: the header looked for at the LoROM, HiROM and ExHiROM places (the last only in an image over
  4 MiB), each scored (§12.2); LoROM, HiROM and ExHiROM decoded with fullsnes's mirroring of odd sizes (the larger
  power of two, then the remainder's mirrors); SRAM by the header's size when the chipset says RAM, LoROM's at banks
  $70-$7D and $F0-$FF below $8000, HiROM's at $6000-$7FFF of banks $20-$3F and $A0-$BF. ExHiROM is not in Venus's
  parity list (plan §4.2); it is there because it costs two lines of the same decode.
- **`bus.rs`**: `System`, which implements the CPU's `Bus`:
  - the access speeds by region (8 for WRAM, $6000-$7FFF and slow ROM; 6 for $2000-$3FFF, $4200-$5FFF, internal
    cycles and, with $420D bit 0 set, the ROM of banks $80-$FF; 12 for $4000-$41FF);
  - the master clock: 1364-clock lines, 262 or 312 a frame by the header's country, line 240 of field 1 four
    clocks short at 60 Hz, and the 40-clock refresh taken between two CPU cycles at the first boundary past H=133.5
    (master clock 534 of the line);
  - the vblank and NMI flags at line 225, cleared at line 0 (and the NMI flag on a read of $4210);
  - open bus as the MDR, which every read and write sets and internal cycles leave alone;
  - WRAM, its mirror and its port ($2180-$2183);
  - the I/O decode, with what later steps fill in kept as written ($4000-$43FF), and two stubs so that test ROMs can
    report: the VRAM port ($2115-$2119) and $4210/$4212's flags;
  - a write with neither VDA nor VPA, the emulation-mode modify cycle, taken as an internal cycle (§10.2);
  - `halted` as a 6-clock cycle while WAI or STP holds the CPU.
- **`machine.rs`**: the machine resets through `Cpu::interrupt(Reset)` and runs a frame to the master clock's next
  frame. Its state is version 2: the CPU, the clock, the bus, WRAM, VRAM, SRAM and the stage-0 memories, 264,283 bytes
  for a cartridge without SRAM, with the layout pinned.

### 12.2 The header's score

Each candidate scores on its own fields: +4 when the map mode fits the place (bits 5-7 $20, and mode 0, 2 or 3 for
LoROM, 1 or $A for HiROM, 5 for ExHiROM); +4 when the complement and checksum agree, +2 more when the checksum is
neither $0000 nor $FFFF; +2 for a reset vector at $8000 or above, +2 more when the byte there is a plausible first
instruction, −4 for one below $8000; +2 for a printable title; +1 for a ROM size byte that covers half the image or
more; +1 for an SRAM size byte of 8 or less. The highest wins, ties going to LoROM, then HiROM. A file with no header
that scores is mapped LoROM. The weights are this core's own; the library is their test (§12.3).

### 12.3 The header check on the library (measured 2026-09-30)

All 826 SNES files of the player's library, each copied to scratch, read by header, and removed; nothing in the
library was written. Mesen's choice was read from its own loader's message log, which the probe now prints after a
load when `EMUSEN_PROBE_LOG` is set (`EmuSen_Debugging_Tools_Reference_v5.md` §3.60). It names the type, the map mode
and the title it settled on. **822 of 826 agree** on the map family and the map mode, which says the same header was
chosen. Mesen named 663 LoROM, 160 HiROM and 3 ExLoROM. The four that differ:

- **Batman: Revenge of the Joker (U)**, D-4. VenusRT took the HiROM-place header, whose checksum even equals the
  file's sum; the LoROM place is empty; Mesen took LoROM. The game decides it: under LoROM VenusRT's WRAM at frame 30
  equals Mesen's in 131,069 of 131,072 bytes, and under HiROM it never leaves its first loop. VenusRT's scorer is
  wrong on this file, and the fix is owed (D-4).
- **Street Fighter Alpha 2 (U) and two dumps of Test Drive II: The Duel (U)**, D-5: the same header, map mode $32,
  which Mesen maps as ExLoROM and VenusRT as LoROM. The S-DD1 is outside parity; Test Drive II waits for the goldens.

### 12.4 The access speeds, three ways (measured 2026-09-30)

`speedtest.rs` builds nine LoROM images whose loop counts in WRAM: the loop's code in slow ROM, fast ROM (MEMSEL set,
banks $80+) or WRAM, and one long read a pass from WRAM, ROM, $2000, $4100, $4300 or $6000. The documents' table and
the datasheet's cycles give each pass 82 to 104 master clocks.

- **Against the documents:** over 60 frames every variant's count is within 0.001 per cent of the frame (less 40
  clocks of refresh a line) over its documented pass, with the carry every 256th pass spread in.
- **Against Mesen**, the same images through the probe, the counter's growth from frame 300 to frame 600:

| Variant | Clocks a pass | VenusRT | Mesen | Difference |
|---|---|---|---|---|
| slow ROM code, ROM data | 100 | 1,038,459 | 1,038,458 | +1 |
| slow ROM code, WRAM data | 100 | 1,038,459 | 1,038,458 | +1 |
| slow ROM code, $2000 data | 98 | 1,059,607 | 1,059,607 | 0 |
| slow ROM code, $4100 data | 104 | 998,599 | 998,599 | 0 |
| slow ROM code, $4300 data | 98 | 1,059,607 | 1,059,607 | 0 |
| slow ROM code, $6000 data | 100 | 1,038,459 | 1,038,458 | +1 |
| fast ROM code, fast ROM data | 82 | 1,266,309 | 1,266,310 | −1 |
| fast ROM code, slow ROM data | 84 | 1,236,224 | 1,236,224 | 0 |
| WRAM code, WRAM data | 100 | 1,038,459 | 1,038,458 | +1 |

One pass in a million is the two engines' different phase in the frame, not a rate. The refresh's 40 clocks, the short
line and every region's speed agree with Mesen's. What this does not cover: where in a line the refresh falls (a
counter over whole frames cannot see it; H-counter latching in stage 3 can), fullsnes's 50:50 "stuttering" refresh,
which is not modelled, and DMA's speed, which is the next step's.

### 12.5 The other oracles

- **gilyon's `cputest-full` and `cputest-basic` on the machine's own bus**: both pass, every test, at frames 163 and
  113, against 162 and 112 on the minimal bus (the real bus is slower by the refresh).
- **The memory and timing ROMs of the corpus**, `memtest`, VitorVilela7's `speed_test_v51` and `test_mdrhdma`, write
  no text by frame 600: each needs DMA for its font or NMI for its waits, which are the next step's. Run again then.
- **The probe's WRAM and SRAM differential** on commercial games needs NMI too (a game waits for its vblank handler
  within its first frames); the one game run here is D-4's.

### 12.6 The cost with the real bus (measured 2026-09-30)

`examples/frame_cost.rs`, best of three under the timing lock, load average 0.6: **0.27 ms a frame** for gilyon's
`cputest-full` (160 frames), **0.25** for the slow-ROM loop and **0.30** for the fast-ROM one (600 frames each),
against §5.5's 0.8 ms for the CPU, the bus and DMA together. Over the flat bus of §11.7 the CPU alone cost 0.06 ms;
the decode, the clock and the refresh check on every cycle are the rest. No tuning was done.

### 12.7 The clone check (measured 2026-09-30)

16 files against Mesen's 164: **no pair shares 12 fingerprints, no table is shared.** The largest pair below the
threshold is `bus.rs` with 6, above the calibration negative's 4 and half the threshold; the names it shares are the
registers' own (`nmiflag`, `vramaddress`), and fullsnes spells them so. Five Mesen identifiers outside the vocabulary
are also VenusRT's: `nmiflag`, `softwareinterrupt`, `sramsize`, `vramaddress` and `writevalue`, each the plain name
of what it holds or does, recorded as §4.2's reading list.

### 12.8 Stage 2's next step

DMA and HDMA (their channels, the per-byte cost and alignment, HDMA's line timing and its overlap with DMA), NMI and
the H/V IRQ with their timers ($4200, $4207-$420A, $4211), the multiply and divide unit with its partial results, and
the auto-joypad read with its busy flag. Its oracles are the plan's §3.2 rows for those: jonasquinn's IRQ, NMI, DMA
and HDMA tests and `muldiv_tests`, undisbeliever's DMA, HDMA and auto-joypad tests, Sour's tests, this step's
memory ROMs, and the probe's WRAM differential on the commercial games once they run past their first vblank.

---

## 13. Stage 2, step 2: DMA, interrupts, the math unit and the joypads (2026-09-30)

### 13.1 What it built

`src/scpu.rs`, with the clock's line events in `bus.rs` and the interrupt dispatch in `machine.rs`. Written from
fullsnes ("SNES DMA Transfers", the $42xx register pages, "SNES Maths Multiply/Divide", "SNES Controllers I/O
Ports"), anomie's timing document (199) for every timing below, and the notes and sources of jonasquinn's
`muldiv_tests` for the math unit's partial values.

- **DMA.** The pause comes one CPU cycle after the write to $420B. It waits to a multiple of 8 master clocks, takes
  8 for the transfer, 8 a channel and 8 a byte, and ends on a whole cycle of the speed the CPU resumes at, counted
  from the pause's start. The eight unit patterns, the A-bus step, the registers left as the transfer leaves them
  (address advanced, count zero), and DMA's own decode: the A-bus never reaches $2100-$21FF, $4300-$437F or
  $420B/$420C. The refresh and a due HDMA are taken between a DMA's bytes.
- **HDMA.** At V=0, H=6 every enabled channel reloads its table and first entry (18 clocks, 8 a direct channel, 24
  an indirect one). At H=278 of lines 0 to 224 each active channel transfers its unit when its entry says so, counts
  its line and loads the next entry at zero (18 clocks a line, 8 a channel, 16 for a new indirect pointer, 8 a
  byte). A zero entry ends the channel for the frame; vblank ends them all; a channel HDMA takes is dropped from a
  pending DMA.
- **NMI and IRQ.** $4210's flag is set at line 225 and cleared at line 0 or by a read; the CPU's NMI is the edge of
  that flag with $4200 bit 7. The H/V comparator sets $4211 at 14 clocks past HTIME's dot, the two long dots
  counted, and at 10 clocks for HTIME 0 or the V mode; a read or a disable clears it.
- **The dispatch**, between instructions: NMI's latched edge before IRQ's level; IRQ only with I clear as the
  check saw it, which after CLI, SEI, PLP, REP and SEP is the old I; WAI ends on either line with two internal
  cycles and then takes the interrupt or carries on.
- **The multiply and divide unit**, one step a CPU cycle, between a read's sampling and a write's latch. A product
  adds the WRMPYB shifter to RDMPY on RDDIV's low bit and shifts RDDIV right, 8 steps; a quotient shifts the divisor
  right, takes it from RDMPY when it fits and shifts the bit into RDDIV, 16 steps. Writes to the operands during a
  run change nothing; WRMPYB clears RDMPY and WRDIVB reloads it without restarting.
- **The joypads.** With $4200 bit 0 set, $4218-$421B are read at line 225, H=74.5, and $4212 bit 0 is set for 4224
  clocks. $4016's strobe reloads two 16-bit shift registers, which $4016 and $4017 shift out, B first, ones after.
- **The state** is version 3: version 2 and the devices, 264,337 bytes without SRAM, the layout pinned.

### 13.2 The self-grading ROMs (measured 2026-09-30)

The corpus runner with `EMUSEN_VENUSRT_ENGINE=1` adds VenusRT as a third engine, 295 ROMs in 7 min 41 s.

| Suite | ROMs | Venus passes | Mesen passes | VenusRT passes |
|---|---|---|---|---|
| gilyon `cputest` | 2 | 0 | 2 | 2 |
| PeterLemon CPU | 23 | 23 | 23 | 23 |
| ADC/SBC | 6 | 6 | 6 | 6 |
| multiply/divide | 7 | 0 | 5 | 5 |
| higan collection, other (one self-grading) | 56 | 1 | 1 | 1 |
| those that need the APU, the GSU or the SA-1 | 62 | 43 | 55 | 0 |

**The old core's five multiply/divide failures all pass**: `mul_behavior`, `div_behavior`, and the three timing
ROMs whose printed partial values equal Mesen's (`muldiv_tests/mul_timing` and `div_timing`,
`snes_mul_div_timing/div_timing`). The sixth ROM both Mesen and Venus fail, and VenusRT fails it too. VenusRT passes
37 of the 92 ROMs Mesen passes, and every ROM it can run that Venus passes; the other 55 wait for the APU (stage 4)
and the coprocessors (stage 5). Nothing threw.

### 13.3 The picture ROMs, by VRAM (measured 2026-09-30)

Without a PPU the measure is VRAM against Mesen's at the last frame, on the 176 ROMs with no verdict whose VRAM stands
still in Mesen. It is weak evidence, as the plan's §3.5 says: a ROM that draws nothing agrees trivially.

| Family (by name) | ROMs | Venus equals Mesen | VenusRT equals Mesen |
|---|---|---|---|
| HDMA | 36 | 36 | 36 |
| DMA | 30 | 26 | 26 |
| IRQ | 14 | 12 | 13 |
| NMI | 4 | 4 | 4 |
| joypad | 3 | 2 | 2 |
| Sour's | 2 | 0 | 0 |
| `memtest` | 1 | 0 | 0 |
| the rest | 86 | 77 | 64 |
| **All** | **176** | **157** | **145** |

VenusRT equals Mesen on `test_irqb`, where Venus does not. The 13 where Venus equals Mesen and VenusRT does not are
the PPU's: eight of undisbeliever's VMAIN remapping ROMs, `test_vram`, the two 240p suites, `intro` and
`inidisp_extend_vblank`; the VRAM port here is a stub with no remapping and no reads.

### 13.4 Step 1's memory ROMs, again

`memtest` now runs to its table. `speed_test_v51` and both `test_mdrhdma` still write nothing by frame 600 and sit in
a wait; what they wait for was not traced (the APU's ports are the likely answer, argued).

### 13.5 A first look at four games (measured 2026-09-30)

Scratch copies of four library games, WRAM after each of 120 frames against the probe's. Not a gate: with no PPU and
no APU each game stalls where it first needs one.

| Game | First difference | By frame 120 |
|---|---|---|
| Super Mario World | frame 1, one byte: $7E01FA is 0, Mesen's $CC | 9 bytes to frame 60, 2,255 from frame 70 |
| A Link to the Past | frame 1, one byte: $7E01FA is 0, Mesen's $73 | 2 bytes at frame 60, 832 |
| Super Metroid | frame 5, 159 bytes | 4 bytes |
| Donkey Kong Country | frame 3, two bytes at $7E01FE | 238 bytes |

Super Metroid's WRAM converges on Mesen's to four bytes and stays there, with NMI enabled and its handler running.
**Unexplained:** the single stack byte at $7E01FA that differs from the first frame in two games. It is in the
reset's stack region, and the datasheet's reset reads the stack without writing it, which is what VenusRT does. The
probe's CPU trace would show which instruction writes it in Mesen; that is the next step's first task, and a dispute
if it turns on the reset sequence. The probe hard-links dumps that did not change and leaves some frames out; the
comparison skips those.

### 13.6 The cost of the CPU, the bus and DMA (measured 2026-09-30)

`frame_cost`, best of three under the lock, twice, load average 0.6 and 1.9 with the same results to 1 per cent:

| ROM | ms a frame |
|---|---|
| gilyon `cputest-full` | 0.66 |
| the slow-ROM loop | 0.56 |
| Super Mario World | 0.61 |
| A Link to the Past | 0.60 |
| Super Metroid | 0.70 |
| Donkey Kong Country | 0.73 |

**Against §5.5's 0.8 ms for the CPU, the bus and DMA on a plain cartridge: inside it, with a tenth to a third left.**
Step 1 measured 0.25 to 0.30 ms on the same two test ROMs, so this step's per-cycle work (the HDMA and DMA checks
before each cycle, the edges after each advance, the math step, the dispatch's look at the next opcode) more than
doubled the cost. No tuning was done, as decided. This is the first place the budget is close, and P1 is the
prediction at risk: the games here are not yet running their real loads, so the number will move.

### 13.7 The clone check (measured 2026-09-30)

17 files against Mesen's 164, with jonasquinn's `muldiv_tests` added to the vocabulary: **no pair shares 12
fingerprints, no table is shared.** The largest below the threshold is still `bus.rs` with 6. Seven of Mesen's
identifiers outside the vocabulary are VenusRT's too: `hdmainit`, `irqflag`, `nmiflag`, `softwareinterrupt`,
`sramsize`, `vramaddress` and `writevalue`, the plain names of what they are.

### 13.8 What is left of stage 2

D-4, the header scorer's rule, was not attempted: a rule that weighs where a reset handler leads needs designing and
testing over the whole library, which did not fit. **Stage 2 needs a third step**, to close what this one measured
and did not explain:

- the stack byte of §13.5, with the probe's CPU trace;
- the four DMA ROMs, the IRQ ROM, the joypad ROM, Sour's two and `memtest` whose VRAM differs from Mesen's, each
  either explained by the missing PPU or fixed;
- the interrupt check placed before an instruction's final cycle, as anomie describes it, in place of the check
  between instructions with the old I;
- HDMA started mid-frame (fullsnes's two cases) and the IRQ's exclusions on the short and last lines;
- D-4;
- what `speed_test_v51` and `test_mdrhdma` wait for.

