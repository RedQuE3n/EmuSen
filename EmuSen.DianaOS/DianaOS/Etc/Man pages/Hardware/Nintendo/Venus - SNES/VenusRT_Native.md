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

---

## 14. Stage 2, step 3: the traces, the scheduler, and what step 2 left open (2026-09-30)

### 14.1 The CPU trace, and the stack byte

The probe records every instruction Mesen's CPU executes in a 24-byte record: the address, the opcode, the
registers and the instruction's master clocks (`EmuSen_Debugging_Tools_Reference_v5.md` §3.40). The machine can now
write the same record (`Machine::trace`, `examples/cpu_trace.rs`), so the two engines are compared instruction by
instruction, cost included. That is Mesen's output, read as any dump is.

- **The stack byte of §13.5 was the stack pointer at power-on.** The first record differed: S was $01FF in Mesen and
  $01FD in VenusRT. The datasheet's §2.25 leaves SL uninitialised at reset, and its reset sequence makes three stack
  cycles; no SNES document gives S after reset. Both values are therefore within the documents, and the choice is a
  power-on value like the RAM fill, not a rule. *Decided 2026-09-30:* power-on SL is $02, so S is $01FF after the
  three decrements, and the engines are comparable from the first instruction. A reset later decrements from
  wherever S stands.
- **With that, the first 1,994 instructions of Super Mario World agree** in every register, and the clocks' running
  total is identical at record 1,990: 50,222. 74 instructions' own costs differ before that, each time by the
  refresh's 40 clocks landing in a different instruction, about 175 clocks earlier in Mesen. Super Metroid's 44th
  instruction reads $4212 with the H-blank flag set in Mesen and clear here. One cause explains both, the CPU's
  phase in line 0 at power-on, and it is D-6: open until the PPU's H-counter can separate a later start from an
  earlier refresh.
- **Where each game first parts from Mesen now**, by trace: Super Mario World at instruction 1,446 and A Link to
  the Past at 32, where the sound CPU's boot signature is ready here at once and in Mesen only after its boot;
  Donkey Kong Country at 494, on the acknowledge's timing; Super Metroid at 44, on D-6. Each is stage 4's or D-6's.
  Donkey Kong Country's two bytes at $7E01FE are the same matter: pushes made at a different time.

| Game | First WRAM difference (frame, bytes) | Bytes at frames 10, 30, 60, 120 |
|---|---|---|
| Super Mario World | 4, 1 | 2, 1, 1, 2,255 |
| A Link to the Past | 30, 1 | 0, 1, 254, 3,038 |
| Super Metroid | 5, 159 | 95, 4, 4, 4 |
| Donkey Kong Country | 3, 2 | 16, 38, 0, 216 |

### 14.2 The ROMs whose VRAM differed (measured 2026-09-30)

VenusRT's and Mesen's VRAM words were set side by side for each of the nine.

| ROM | What differs | Cause |
|---|---|---|
| `memtest` | the lower halves of banks $40-$6F read as ROM | **fixed**: step 1 mapped a ROM mirror there on no document's authority; fullsnes gives a LoROM cartridge only SRAM below $8000. VRAM now equals Mesen's |
| Sour's `timing_test` (two copies) | every printed value | the PPU's latched H and V counters ($2137, $213C, $213D), read as open bus; stage 3 |
| Sour's `dma_irq_test` | the picture it draws from those counters | stage 3 |
| `test_dmatiming/demo` | two printed positions | the same counters; stage 3 |
| `blip-autojoy-timing-test` | its plotted positions | the same counters; stage 3 |
| `hvdma`, `hvdma_max` | words written by DMA in H-blank | VRAM's access rules during display; stage 3. Venus differs by the same bytes |
| `test_dma` | one word | a VRAM read, which the port's stub lacks; stage 3 (argued). Venus differs by the same byte |

None is DMA's, IRQ's or the joypad's own.

### 14.3 The interrupt check, mid-frame HDMA, the IRQ's exclusions

- **The check.** The bus keeps the NMI edge and the IRQ flag as each cycle found them, so that when an instruction
  ends the machine has them as its final cycle found them, which is where anomie's document places the check. An
  edge that arrives in the final cycle waits for the next instruction. After WAI the line that ended it is taken
  without a further instruction.
- **HDMA started mid-frame**, fullsnes's two cases: a channel enabled after V=0 is active at once, and transfers on
  its first line only if an init ran this frame, that is, if any channel was enabled at V=0.
- **The exclusions:** dot 153 raises no IRQ on the short line or on a frame's last line.

*Measured after them:* the single-step suite is unchanged, 5,079,735 of 5,120,000 with the same 40,265 named
exceptions; gilyon's two ROMs pass on the machine; the nine speed variants are within one pass of Mesen; the IRQ and
NMI picture ROMs are as in §14.6. No ROM in the corpus was found that passes or fails on any of the three alone,
which is a negative result: they are implemented from the documents and not yet pinned by an oracle.

### 14.4 What the two silent ROMs wait for

Not the sound CPU. `speed_test_v51` polls $3000, cartridge coprocessor space: it is an SA-1 test (stage 5).
`test_mdrhdma` polls $213F bit 7, the PPU's field flag (stage 3). The games do wait on the sound CPU, so its ports
have a stand-in for the boot ROM (fullsnes, "Uploader"): $BBAA until the CPU's $CC on port 0, then each port as last
written. It is marked as stage 4's in the code and it answers at once, which no real sound CPU does (§14.1).

### 14.5 D-4, the header's reset handler (measured 2026-09-30)

Every candidate header of all 826 library files (scratch copies, removed after) was run from its reset vector for
4,000 instructions under its own map. Of the candidates the fields reject, 819 crash (BRK, COP, STP or unmapped
memory); of those they choose, 4. Counting I/O writes separates little by itself: 337 chosen handlers write fewer than
8 registers in that time. The rule taken is the narrow one the data supports: when the fields' choice neither
crashes nor writes more than one register, and another candidate does not crash and writes at least 4, the other is
taken. **It changes one file of 826, Batman, to LoROM with map mode $00, which is Mesen's choice; 823 of 826 now
agree with Mesen and none of the 822 moved.** The other three are D-5. The constants are calibrated on this library
and on nothing else, and the entry says so.

### 14.6 The oracle tables again

| | Step 2 | Step 3 |
|---|---|---|
| Self-grading ROMs VenusRT passes, of Mesen's 92 | 37 | 37 |
| Standing picture ROMs with VRAM equal to Mesen's, of 176 (Venus: 157) | 145 | 148 |

By family: HDMA 36 of 36, DMA 26 of 30, IRQ 13 of 14, NMI 4 of 4, joypad 2 of 3, Sour's 0 of 2, `memtest` 1 of 1
(after §14.2's fix; the corpus run before it counted 147), the rest 66 of 86. Every one that differs in the first
seven families is in §14.2's table.

### 14.7 The cost, and where it went

**A profile first.** A ptrace sampler over Super Metroid's frames (5,816 samples, each attributed to its innermost
inlined function through the binary's line tables) put 41 per cent of the time in the clock's `advance`, 17 in the
HDMA check and 8 in the other checks made before each cycle: two thirds in bookkeeping repeated on every CPU cycle,
against 7 in the CPU's own step and 11 in the memory decode.

**That was a drift from the plan, and it is corrected.** §5.2 of the plan says "a scheduler holds the fixed events of
a line (the DRAM refresh, HDMA initialisation and transfer points, the H/V IRQ comparators, auto-joypad reads, the
NMI line) as master-clock times". Steps 1 and 2 instead asked on every cycle whether each event was due. The clock
now keeps the line clock of its next event, the earliest of the line's pending events and its end; a cycle that ends
before it only adds to the clock, and the checks are the slow path taken at an event. Following the plan's design is
not tuning, and nothing else was changed for speed. *Measured:* 35 CPU traces (four games over 60 frames,
`cputest-full`, thirty test ROMs) are byte-identical before and after, registers and each instruction's clocks.

| ROM | Step 2, ms a frame | Now |
|---|---|---|
| the slow-ROM loop | 0.56 | 0.36 |
| Super Mario World | 0.61 | 0.38 |
| A Link to the Past | 0.60 | 0.40 |
| gilyon `cputest-full` | 0.66 | 0.42 |
| Super Metroid | 0.70 | 0.49 |
| Donkey Kong Country | 0.73 | 0.55 |

Best of three under the lock, load average 1.5. Against §5.5's 0.8 ms for the CPU, the bus and DMA, a third to a
half is left. The profile after: the clock a quarter, the memory decode about 30 per cent, the CPU's step 11. The
games still stall where they need the sound CPU, so these are not their real loads.

### 14.8 The clone check

17 files against Mesen's 164: no pair shares 12 fingerprints, no table is shared, and the largest pair below the
threshold is back to 4 (`cpu/mod.rs`); `bus.rs`'s 6 went with the per-cycle checks. The same seven plain names.

### 14.9 Stage 2 is complete, with what it leaves open

Built and graded: the cartridge and its header, the bus, the master clock and its scheduled events, DMA and HDMA,
NMI and IRQ, the multiply and divide unit, the joypads. Open, and named: D-2 (a pin), D-5 (map mode $32 without an
S-DD1), D-6 (the power-on phase), and the three behaviours of §14.3, implemented from the documents with no oracle
yet. The state is version 4.

**Stage 3's first step**, the PPU: the registers and their ports with their real behaviour in place of the stubs
(VRAM with VMAIN's remapping and its reads, CGRAM, OAM with its address reload, the H/V counter latch and $213F,
which D-6, Sour's tests and `test_mdrhdma` wait for); then the picture's backbone, backgrounds in mode 0 and 1 and
the backdrop, as a scanline renderer drawn in spans (plan §5.1), with the frame handed to the interface. Its oracle
is Mesen's picture at standing frames on the 240p suite and PeterLemon's PPU ROMs, and the VRAM, CGRAM and OAM
spaces of every ROM in §14.2's table.


---

## 15. Stage 3, step 1: the PPU's ports and the picture's backbone (2026-09-30)

### 15.1 What it built

`src/ppu.rs`, from fullsnes's PPU sections and anomie's register document, in place of stage 2's VRAM-port stub:

- **VRAM** through `$2115`-`$2119` and `$2139`/`$213A`: VMAIN's step (1, 32 or 128 words, on the low or the high
  byte) and its three translations (the low 8, 9 or 10 bits of the address rotated left by three); the read
  prefetch, refilled from the address as it stood before the step, and taken again on a write to `$2116`/`$2117`.
  Writes land only in V-Blank or forced blank (fullsnes: "All video memory can be accessed only during V-Blank, or
  Forced Blank"); the address steps whether or not the write landed.
- **CGRAM** through `$2121`, `$2122` and `$2138`/`$213B`, with the low byte held until the second write and bit 7 of
  the second read byte from PPU2's open bus. **OAM** through `$2102`-`$2104` and `$2138`: the 9-bit reload value and
  the 10-bit address, the low table written in pairs, the high table byte by byte, and the address reloaded at the
  start of line 225 when not in forced blank and when forced blank ends during that line. CGRAM and OAM take writes at
  any time; what the hardware does with OAM writes during display is not modelled (stage 3's sprite step).
- **The counter latch**: `$2137` latches when WRIO (`$4201`) bit 7 is set, as a 1-to-0 write of that bit does; the
  counters read low byte then high through `$213C`/`$213D`, with PPU2's open bus in the high byte's upper bits;
  `$213F` returns the field, the latch flag, the region and version 3, and resets the high/low selectors, and the
  latch flag only while WRIO bit 7 is set. The dot counts the line's two long dots (323 and 327, six clocks each). WRIO
  is `$FF` at power-on.
- **The two open-bus latches** of PPU1 and PPU2 (anomie's open-bus document), shown by the write-only addresses
  that read as PPU1's and by the unused bits of `$213B`-`$213F`.
- **The picture**, as plan §5.1's scanline renderer in spans. Each register write first draws the current line up to
  the pixel of the write's dot (H=22 is the first visible pixel), from the registers as they stood; each line's end
  draws the rest. A pixel is the frontmost opaque pixel of the enabled backgrounds in modes 0 and 1, in fullsnes's
  priority chart (mode 1 with and without BG3's priority bit), or the backdrop: tile maps of 32 or 64 tiles a side,
  8×8 and 16×16 tiles, both flips, 2 and 4 bits a pixel, mode 0's per-background palette offset, and the
  two-write scroll registers. Forced blank is black; brightness N scales each 5-bit component to c×(N+1)/16 (D-7).
  The frame, 256×224, is the interface's picture. **What it does not do**: a register write takes effect at its own
  dot, with no per-register latency (the plan's "documented or disputed rule per register"; no document gives one for
  these registers yet); sprites, windows, colour math, mosaic and modes 2 to 7 draw nothing.
- **Drawing is apart from the machine.** The renderer reads the PPU's state and writes only the frame, which is not
  part of the state; the line counter that tells the PPU of each line's end, the OAM reload and the latches run
  whether or not the picture is drawn. `set_options` bit 0 skips the pixel writes and nothing else.
- **The state is version 5**: a `Ppu` group of every port, latch and counter, then VRAM, CGRAM as 256 words, and
  OAM; 264,449 bytes in 88 layout lines, pinned by `the_version_5_layout_is_pinned`.

### 15.2 Skipping the picture never changes the machine (measured 2026-09-30)

`a_skipped_picture_leaves_the_machine_as_a_drawn_one_does` runs a program that draws and feeds every readable PPU
port back into VRAM, CGRAM, OAM and the scroll and brightness registers, twice, once skipping the picture, and
compares the whole saved state after each of six frames. `examples/skip_check.rs` does the same for any ROM: over
every image of the corpus and the nine bench games (304 images, 300 frames each) the two runs never part, 268 of the
304 with a picture lit; over 3,600 frames of `inidisp_extend_vblank` likewise. This is P7 for stage 3 so far.

### 15.3 D-6, the power-on phase (measured 2026-09-30)

The H counter settles the measurement, not the dispute. Sour's `timing_test` prints four positions before it
starts HDMA; every one is 32 dots, 128 master clocks, later in Mesen than in VenusRT, and `examples/power_phase.rs`,
which moves VenusRT's power-on position, reproduces all four at 128 clocks and at no other offset tried.
undisbeliever's `reset-position-test`, whose reset handler latches the counters first, prints OPHCT `$0015` on
VenusRT and `$0035` in Mesen, the same 32 dots. No document gives the position and no ROM in the corpus records a
console's result, so the rule stays fullsnes's start at H=0, V=0 and Mesen's offset an observation. The full
argument, and the two console readings that would settle it, are in `VenusRT_Disputes.md` D-6. Over 1,200 frames of
all 304 images, moving the phase changes the text left in VRAM in eight images and the verdict of no
self-grading test.

### 15.4 The pictures against Mesen (measured 2026-09-30)

Mesen through the probe and VenusRT at frame 300, compared over 256×224 with Mesen's rows offset by 7 (§3); a
ROM's features are read from VenusRT's registers at that frame.

| Set | ROMs | Using only this step's features | Of those, equal to Mesen |
|---|---|---|---|
| PeterLemon's PPU ROMs | 47 | 9 | 8 |
| the 240p suite (NTSC and PAL) | 2 | 0 (sprites) | — |

The eight equal ones: the four `8x8BGnMap2BPP32x328PAL`, `GreenSpace`, `RedSpaceHDMA`, `RedSpaceIndirectHDMA` and
`Rings`, every pixel. The ninth, `RedSpace9BitHDMA`, pairs each line's backdrop colour with a brightness by HDMA;
VenusRT follows fullsnes's formula on every line, and Mesen is one lower on 59 of 224 (D-7). The other 38 use
modes 3, 5 or 7, sprites, windows, colour math, mosaic, pseudo-hi-res or interlace, the remaining PPU steps. **The 240p suite's
menu differs from Mesen's in one 54×101 region**, x 177 to 230 and y 80 to 180, which is its sprite character;
the rest of the screen, three backgrounds in mode 1, is equal.

### 15.5 The corpus again (measured 2026-09-30)

The runner gained VenusRT columns for CGRAM, OAM and the picture at the last frame. The VRAM table of §13.3 and
§14.6, by family, on the 176 ROMs with no verdict whose VRAM stands still in Mesen:

| Family | ROMs | VRAM equal, step 3 of stage 2 | Now | CGRAM equal | OAM equal | Picture equal |
|---|---|---|---|---|---|---|
| HDMA | 36 | 36 | 36 | 30 | 36 | 12 |
| DMA | 30 | 26 | 27 | 28 | 30 | 26 |
| IRQ | 14 | 13 | 14 | 7 | 14 | 7 |
| NMI | 4 | 4 | 4 | 1 | 4 | 1 |
| joypad | 3 | 2 | 2 | 3 | 3 | 1 |
| Sour's | 2 | 0 | 0 | 2 | 2 | 0 |
| `memtest` | 1 | 1 | 1 | 1 | 1 | 1 |
| the rest | 86 | 66 | 81 | 82 | 82 | 34 |
| **All** | **176** | **148** | **165** | **154** | **172** | **82** |

C# Venus's VRAM equals Mesen's on 157. The eleven that still differ in VRAM: Sour's two, `test_dmatiming/demo`,
`blip-autojoy-timing-test` and `reset-position-test`, which print latched counters (moving the power-on phase by D-6's 128
clocks changes four of the five, and makes none of them equal); `test_speed`, `test_timer_speed3` and `wrmpyb-in-flight`, which print timings; `cx4test`, a
Cx4 cartridge (stage 5); `hvdma`, ten words written by HDMA during H-blank with forced blank set and cleared around
them, the ROM's own README describing what a console shows; and `test_dma`, below. The self-grading verdicts are as
at stage 2: 37 of Mesen's 92. The corpus was run again after §15.7's `$213F` fix, and every VenusRT cell of all 295 rows
was the same.

Of the 176, **116 use only this step's features at their last frame and 79 of those are equal to Mesen** in every
pixel. The 37 that are not are mostly not the PPU's:

- **28 ROMs grade themselves through the backdrop** (*corrected 2026-10-01: 25; three of the 28 write only blue, when
  they finish, and grade nothing, §16.2*), colour 0 blue (`$7C00`) for a pass and red (`$001F`) for a
  failure: byuu's `snestest_082506`, and the `blobs`, `nmi_irq`, `test_dmavalid`, `test_mdrhdma` and other folders of
  the same collection. The runner's protocols do not read it, so they count as visual, and VRAM equal to Mesen hid
  their verdicts. **Mesen passes all 28; VenusRT passes 11 and fails 17**, IRQ, NMI, HDMA and DMA tests, at either
  power-on phase. The first of them read, `test_nmi`, stops at its test 1 on an NMI taken one instruction early;
  its header records the console's timing of the NMI line, which neither document gives, and that is D-8. These are
  stage 2's behaviours, seen for the first time because there is now a picture. Teaching the runner this protocol
  would move the recorded baseline of all three engines, and was left for the step that re-records it.
- `hdma_midframe/demo` shows a red backdrop where Mesen's is black, `test_noise` a grey one level brighter on a
  third of the screen, `inidisp_brightness_delay` 30 pixels, the INIDISP early-read effect plan §5.1 accepts as a
  span's loss (argued from its name); the two `hdmaen_latch_test` copies, `hdma-double-buffered-parallax`,
  `test_hello`, Sour's and blargg's timing printers and `cx4test` make up the rest. Not read further.
- `window-precalculated-single` and `-symmetrical` clip the main screen to black with the colour window, a feature
  the tagging missed; windows are a later step.

### 15.6 The leftover ROMs of stage 2 (measured 2026-09-30)

| ROM | VRAM | CGRAM | OAM | Picture | Why |
|---|---|---|---|---|---|
| Sour's `timing_test`, two copies | 30 B, 28 B | equal | equal | 598, 526 px | the printed counters (D-6); sprites |
| `test_dmatiming/demo` | 4 B | equal | equal | 77 px | two printed positions (D-6) |
| `blip-autojoy-timing-test` | 37 B | equal | equal | 677 px | its plotted positions |
| `hvdma` | 16 B | equal | equal | sprites | §15.5 |
| `hvdma_max` | equal | equal | equal | equal | |
| `test_dma` | 1 B | 128 B | equal | every pixel | its own verdict, a failure: DMA overlapping HDMA on the same channel |
| `test_mdrhdma` (both) | equal | equal | equal | equal | runs now that `$213F` reads |

### 15.7 Two defects of this step, and one of the runner's

- **VRAM opened on line 0.** The first version let writes land on line 0 as well, on no document's authority.
  `vram-mid-scanline-test`, whose writes begin as V-Blank ends, wrote 672 words that Mesen does not; with the
  documents' rule its VRAM and picture equal Mesen's. Nothing else in the corpus moved. Plan §5.1 had listed this
  ROM as a known loss of spans; it is not one.
- **`$213F` cleared its latch flag on every read**; anomie's document clears it only while WRIO bit 7 is set.
- **The runner counted calls, not frames.** A DMA longer than a frame (a 64 KiB fill is about 1.5 frames of bus time)
  is one CPU step, so one `Advance` can end two frames on, and after 3,600 calls `inidisp_extend_vblank` was at
  frame 3,602, compared with Mesen's 3,600. The runner now stops on the machine's own frame count. The examples that
  run to a frame number by `total_frames` were never affected; stage 2's per-frame WRAM comparison of four games
  (§13.5) counted calls and was not measured again.

### 15.8 The four games (measured 2026-09-30)

`The_games_pictures_and_spaces_against_Mesen` (opt-in through `EMUSEN_VENUSRT_GAMES`), every tenth frame to 600:

| Game | Mesen's first picture | VenusRT's | First frame VRAM, CGRAM, OAM, picture differ | Why the picture parts |
|---|---|---|---|---|
| Super Mario World | 90 | none | 70, 90, 70, 90 | forced blank throughout: stalled on the sound CPU's stand-in |
| A Link to the Past | 90 | 580 | 60, 60, 60, 90 | sprites only (TM `$10`) until 580; then colour math |
| Super Metroid | 20 | none | 280, 250, 230, 20 | sprites only, then mode 7 |
| Donkey Kong Country | 80 | none | 70, 80, never, 80 | forced blank throughout, as Super Mario World |

No game's first picture uses only this step's features, so each parts from Mesen at the first frame it draws. The
two that stall are stage 4's.

### 15.9 The cost, and the PPU's share (measured 2026-09-30)

`frame_cost`, best of three under the timing lock, load average 1.5, with the picture drawn and skipped:

| ROM | Drawn, ms a frame | Skipped | The picture |
|---|---|---|---|
| gilyon `cputest-full` | 1.10 | 0.44 | 0.66 |
| the slow-ROM loop (forced blank) | 0.40 | 0.38 | 0.03 |
| the 240p suite's menu (three backgrounds) | 1.27 | 0.31 | 0.95 |
| `Rings` | 1.42 | 0.42 | 0.99 |
| `8x8BG1Map2BPP32x328PAL` (one background) | 1.05 | 0.44 | 0.61 |
| Super Mario World (blank) | 0.43 | 0.40 | 0.03 |
| A Link to the Past | 0.69 | 0.43 | 0.26 |
| Super Metroid | 0.65 | 0.51 | 0.15 |
| Donkey Kong Country (blank) | 0.60 | 0.58 | 0.02 |

Against §5.5's split, 1.5 ms for the PPU on a plain cartridge: **backgrounds alone in modes 0 and 1 take 0.6 to 1.0
ms, two thirds of it**, before sprites, windows, colour math and the other modes. The renderer works a pixel at a
time, fetching each background's map entry and tile row again for every pixel; spans make a line with no mid-line
write cost what a scanline costs, as the plan says, and the scanline itself is what is slow. Nothing was tuned, as
decided. This is P1's first likely overrun, and it is recorded now so that the remaining steps are priced against
it: at this rate the sprite and colour-math steps alone would take the PPU past its 1.5 ms. The skipped column is
stage 2's cost (§14.7) to within a few hundredths: the span bookkeeping on each register write costs little.

### 15.10 The clone check (measured 2026-09-30)

18 files against Mesen's 164: no pair shares 12 fingerprints, no run of 8 literals is shared, `ppu.rs` shares no
fingerprint with any Mesen file, and the largest pair below the threshold is still 4 (`cpu/mod.rs`). The names are
the seven of §14.8 and two more, `cgramaddress` and `forcedblank`, fullsnes's own two-word terms run together.

### 15.11 What is left of stage 3, and the next step

Open: D-6 (the power-on phase), D-7 (brightness's rounding) and D-8 (the NMI line's time, stage 2's). The plan gives
stage 3 eight steps; this was the first. The next, in the plan's order: **sprites and their evaluation** (OBSEL's
sizes and base, the 32-sprite and 34-tile limits and `$213E`'s flags, which run when the picture is skipped,
priority rotation, OAM writes during display), then windows and colour math with the sub screen, then modes 2 to 6
with offset-per-tile and direct colour, mosaic, hi-res, pseudo-hi-res and interlace, then mode 7, then the PPU's
read-side timing. Before the sprite step: D-8, implemented from `test_nmi`'s header and measured on the 17
backdrop-graded ROMs and the CPU trace, and the backdrop protocol taught to the runner with the baseline re-recorded.

---

## 16. Stage 3, step 2: the interrupt lines, the backdrop protocol, and sprites (2026-10-01)

### 16.1 The interrupt lines and the counter latch (D-8, D-9, D-10)

byuu's `test_nmi` and `test_irq` state their console results in their headers, and the ROMs grade themselves
against them; they were taken as the documents for three rules (the entries give the argument):

- **NMI** (D-8): line 225's flag and /NMI take effect at HC=6 in the bus's end-of-cycle frame (the header's HC=2 for
  the flag, read two clocks into a six-clock cycle, and HC=6 for the line); a `$4210` read ending before HC=10 does
  not clear the flag; an NMI edge a `$4200` write makes is not seen by the check at the very next cycle's start.
- **The latch** (D-9): `$2137` latches four clocks before its cycle ends, which with the WRIO latch at the end of
  its write is anomie's "1 dot later". `test_nmi` reaches its test points through a seek that reads the counter, so
  D-8 could not be measured without it.
- **IRQ** (D-10): the flag and the CPU's line at anomie's point plus four; a `$4211` read within four clocks of it
  leaves the flag; the IRQ the check saw is taken though the final cycle disables it.

**Measured:** the three copies of `test_nmi` and `blobs/test_irq` pass every test; they stopped at test 1. Of the
25 backdrop-graded ROMs (§16.2) VenusRT passes 12, against 8 before. gilyon's two CPU ROMs still pass on the
machine. **D-6 moved:** with the latch a dot earlier no power-on offset reproduces all four of Sour's rows (132
clocks gives three, 130 the other three), and the 128 of §15.3 is retired. The state is version 6 for the NMI hold,
then 7 for the sprite flags.

### 16.2 The backdrop protocol, and the baseline re-recorded (measured 2026-10-01)

The runner reads byuu's verdict where no text verdict is found: colour 0 `$7C00` is a pass and `$001F` a failure,
for ROMs of the jonasquinn collection whose folder's source, or `blobs`' disassembly, writes both colours. **25 ROMs**
grade this way, not §15.5's 28: `test_math`, `test_mul` and `test_mdrhdma2` write only blue, when they finish, with
their results in SRAM for another engine to compare, and the protocol rightly leaves them visual.

| On the 25 | Passed | Failed | Other colour |
|---|---|---|---|
| Mesen | 25 | 0 | 0 |
| C# Venus | 8 | 13 | 4 |
| VenusRT | 12 | 13 | 0 |

The old core's 13 are IRQ, NMI, HDMA and DMA tests: eight of VenusRT's 13, its four new passes (the three
`test_nmi` and `test_irq`), and `demo_nmi`, which VenusRT passed already; its other four are on a colour that is
neither verdict. It is black-box information for the plan's baseline, which §3.1 and §3.5 of the plan now carry. Across the
corpus as the runner counts: Mesen passes 117 self-grading ROMs (92 before), Venus 81 of those (73), VenusRT 49
(37). `VenusRtBaseline.tsv` is re-recorded with all three engines' columns (VenusRT's for the record; the test
compares Venus's and Mesen's), and every cell that changed was a backdrop verdict or its detail. The test passes against the new table
(a fresh Venus run, Mesen from its cache).

### 16.3 Sprites (`ppu.rs`)

From anomie's register document ("SPRITES") and fullsnes (OBSEL, STAT77, the priority chart):

- **Evaluation, a line ahead.** As each line ends the PPU chooses the next line's sprites: from the first sprite
  (sprite 0, or with priority rotation the internal word address's sprite), the first 32 whose rows cover the line
  and whose X is within range, setting `$213E` bit 6 at a 33rd; then, from the last of those back, up to 34 tiles
  whose X is on the screen, setting bit 7 at a 35th, so it is the first sprites that lose tiles. An OBJ at X=256
  counts as at 0 for both limits and draws off the screen. A sprite's row is the line less one less its Y, in eight
  bits, so tall sprites wrap from the bottom to the top. The flags clear at the end of V-Blank outside forced blank;
  nothing is evaluated in forced blank. **The evaluation runs whether or not the picture is skipped**; only the tile
  decoding below is skipped.
- **Tiles.** OBSEL's eight size pairs (two undocumented), its base and its name gap; the 16×16 tile table wrapping
  in each direction; horizontal flip, and vertical flip with a rectangular sprite flipped as two squares. **Each
  tile's row is decoded once** into a 256-entry line of sprite pixels (CGRAM index and OAM priority), a tile loaded
  later covering what is there, so the first sprite is on top and only its priority meets the backgrounds. Plan
  §5.1 does not say how a span is decoded; for sprites this is the simplest correct way, since the time limit is
  counted in tiles.
- **Priority.** Sprites take their four places in modes 0 and 1's chart, and draw over the backdrop alone in the
  modes whose backgrounds are later steps'.
- **Not modelled:** OAM writes during display land where the address points (D-12); the overflow flags are set as
  the line is chosen, not at the dots fullsnes gives (H=index×2 and H=0 of the next line); the "write 4n+2(A&1)+1
  bytes" rotation oddity anomie describes; sprite interlace; the reload on any 1-to-0 of `$2100` bit 7 (D-11).

### 16.4 The sprite oracles (measured 2026-10-01)

- **PeterLemon's PPU ROMs have no sprite test**: the sprites in that folder are in its mode 3, pseudo-hi-res and
  interlace demos, later steps'.
- **The 240p suite's menu** (NTSC and PAL), whose one difference at §15.4 was its sprite character: **equal to
  Mesen in every pixel.**
- **The standing ROMs** (§15.5's 176): pictures equal on 115 (82 at §15.5). Of the 40 whose only feature beyond
  step 1 is sprites, **29 are equal**, among them undisbeliever's `object-dropout-test` (the range and time limits,
  flipped sprites, the X=256 case), byuu's `rto` (range and time overflow) and `test_oam`, and
  `setini-early-read-obj`. The 11 that are not: INIDISP and HDMA-to-INIDISP glitch tests (plan §5.1's accepted
  losses), `hvdma` (§15.6), Sour's two (D-6) and `HblankEmuTest`. Over all 295 rows, pictures equal Mesen's on 157
  (124).
- **The four games**, every tenth frame to 600. **Super Metroid** draws sprites only, then sprites over mode 7's
  disabled background: equal to Mesen at every twentieth frame from 20 to 400 but the fades at 20 and 160, which
  are D-7's brightness, until its state parts at 420. **A Link to the Past**: its sprite-only frames 100, 120 and 240
  to 280 are equal; at 60 to 80 and 140 to 220 its "Nintendo" logo comes and goes on different frames in the two
  engines, VenusRT ahead with the sound CPU's stand-in answering at once, which is timing and not the PPU. Super
  Mario World and Donkey Kong Country stay in forced blank, as before (stage 4).
- **Skip versus draw:** the 304 images of §15.2 never part over 300 frames, with evaluation running in both.

### 16.5 The cost (measured 2026-10-01)

`frame_cost`, best of three under the timing lock, load average 1.75 (1.46 at §15.9):

| ROM | Drawn | Skipped | The picture | §15.9's picture |
|---|---|---|---|---|
| gilyon `cputest-full` | 1.19 | 0.53 | 0.66 | 0.66 |
| the 240p menu (three backgrounds, sprites) | 1.35 | 0.39 | 0.96 | 0.95 (no sprites) |
| `Rings` | 1.53 | 0.52 | 1.01 | 0.99 |
| `object-dropout-test` (sprites, one background) | 0.77 | 0.37 | 0.40 | — |
| `rto` | 0.85 | 0.48 | 0.37 | — |
| A Link to the Past | 0.85 | 0.50 | 0.35 | 0.26 |
| Super Metroid | 0.82 | 0.56 | 0.26 | 0.15 |
| Super Mario World (forced blank) | 0.45 | 0.42 | 0.03 | 0.03 |

**The sprites' decoding is small beside the backgrounds**: a sprite-heavy test costs 0.4 ms of picture against
1.0 for three backgrounds. The skipped column rose by 0.06 to 0.09 ms on every ROM that is not in forced blank,
the sprite evaluation that runs in both modes (128 entries a line for 224 lines), argued from its absence on the
blank Super Mario World; part of it is the higher load. Against §5.5's 1.5 ms for the PPU, the backgrounds' 0.6 to
1.0 ms of §15.9 stand, the evaluation adds about 0.07 always, and a sprite-heavy line about 0.1 to 0.3 more. Nothing
was tuned, as decided; the backgrounds' per-pixel fetch is the open question.

### 16.6 The clone check (measured 2026-10-01)

18 files against Mesen's 164: no pair shares 12 fingerprints, no table is shared (`OBJ_SIZES` included), the largest
pair below the threshold is still 4 (`cpu/mod.rs`), and `ppu.rs` shares none. The names are §15.10's nine and two
more, `rangeover` and `timeover`, fullsnes's "Range overflow" and "Time overflow".

### 16.7 What is left, and the next step

Open: D-6, D-7, D-11, D-12; D-10 for five IRQ tests; the HDMA and DMA tests among the 13 backdrop failures. The next
PPU step in the plan's order is **windows and colour math with the sub screen**: the two windows and their logic, the
colour window, the sub screen and fixed colour, add and subtract with halving, OBJ palettes 4 to 7 alone taking
part, and the clip to black; its oracles are undisbeliever's window ROMs, PeterLemon's `Window` and `Blend` folders
where their mode allows, and the colour-math frames of A Link to the Past. The 13 backdrop failures can be read test
by test beside it, as D-8 and D-10 were.

---

## 17. Stage 3, step 3: windows and colour math, the backgrounds decoded per tile, and the backdrop failures (2026-10-01)

### 17.1 What it built

- **The compositor, on line buffers.** Each span first fills the line buffers of the backgrounds either screen
  shows, beside the sprites' buffer of §16.3; the compositor reads only the buffers, so how a layer fills its
  buffer can change without touching it. Per pixel: the two windows, each inside or outside, combined by
  WBGLOG/WOBJLOG for each layer and for the colour window (fullsnes, "SNES PPU Window"); the main screen's front-most
  pixel of the layers TM shows and TMW does not hide; the colour window's clip to black and prevent-math (CGWSEL,
  both documents agreeing); math on the front-most sub-screen pixel of TS less TSW, or on COLDATA's fixed colour
  when CGWSEL bit 1 is clear or the sub screen is transparent; add or subtract per component, halved except on a
  clipped pixel or the sub backdrop, saturated; sprites taking part only with palettes 4 to 7. Which sub pixel is
  used regardless of priority is D-13. Brightness is applied after. The state is version 8, with COLDATA's colour.
- **The backgrounds decoded per tile** (§17.3).
- **D-14**, an indirect HDMA terminator's pointer load (§17.4).

### 17.2 The window and colour-math oracles (measured 2026-10-01)

- **undisbeliever's window ROMs**: `window-mask-logic`, `window-shapes-single`, `window-precalculated-single` and
  `window-precalculated-symmetrical`, **every pixel equal to Mesen at frame 3600**; at §15.5 they differed on 9,664
  to 56,607 pixels. So is `color_halve_proof/demo`, which differed on every pixel; `test_math` and
  `inidisp_fadein_fadeout` stay equal.
- **PeterLemon's `Window` and `Blend` folders** are in mode 3 or pseudo-hi-res, later steps': nothing there is in
  this step's modes. His eight ROMs in modes 0 and 1 stay equal, and `RedSpace9BitHDMA` stays D-7's.
- **A Link to the Past's colour-math frames** (580 to 600, CGADSUB `$31`, CGWSEL `$02`) differ from Mesen's over
  the title's centre, but its VRAM already differs from frame 60, where the sound CPU's stand-in answers at once
  (§15.8); the frames show different scenes, so they are no oracle for the math until stage 4. A negative result.
- **The corpus**: §17.6.

### 17.3 The backgrounds decoded once per tile (decided 2026-10-01, measured)

*Decided 2026-10-01*: the background renderer's per-pixel fetch is reworked now, ahead of the general hold on
optimisation, because every later PPU step builds on it. Each span walks the 8-pixel chunks of background space it
covers; for each, the map entry is read and the tile row's words fetched once, decoded to eight pixels in screen
order, and the part of the chunk inside the span copied to the line buffer. Spans split where they did: the chunk is
re-decoded at a span's start from the registers as they stand then.

**Built for what follows:** the scroll is read per chunk through one function, where offset-per-tile's per-column
offsets (modes 2, 4, 6) go; mosaic can repeat pixels over the filled buffer, which persists across a line's spans,
so a block that began in an earlier span is still there; the buffers are 256 wide, and hi-res needs 512, a
widening of the buffers and of the compositor's loop, not of the decode; mode 7 is its own fill into the same
buffer.

**Proof (measured 2026-10-01).** `examples/frame_hashes.rs` hashes the picture after every frame. Built from the
commit before the rework and from the rework, over all 304 images (the corpus's 295 and the nine bench games) to
frame 3600: **1,094,067 frames, every hash equal**. The hashes are 64-bit SipHash, so this is equality to within a
collision probability of order 10⁻¹³ over the set, not a byte comparison. The skip check of §15.2 passes on all
304; the crate's 39 tests pass.

**The cost**, `frame_cost` before and after in turn, best of three under the timing lock, load 1.4:

| ROM | Before: drawn, skipped, picture | After | §16.5's picture (no windows or math) |
|---|---|---|---|
| gilyon `cputest-full` | 1.65, 0.52, 1.12 | 1.48, 0.50, 0.99 | 0.66 |
| the 240p menu | 1.61, 0.39, 1.22 | 1.31, 0.38, 0.92 | 0.96 |
| `Rings` | 1.82, 0.52, 1.31 | 1.46, 0.50, 0.96 | 1.01 |
| `object-dropout-test` | 1.22, 0.38, 0.84 | 1.18, 0.37, 0.80 | 0.40 |
| A Link to the Past | 1.38, 0.50, 0.88 | 1.29, 0.48, 0.81 | 0.35 |
| Super Metroid | 1.11, 0.59, 0.52 | 1.07, 0.55, 0.51 | 0.26 |

(`Rings`' skipped run after the rework read 0.64 once and 0.50 once; 0.50 is taken.) **The rework takes 0.1 to 0.4
ms off the picture where backgrounds dominate, and the compositor of §17.1 added more than that**: it evaluates six
window masks and walks the priority chart up to twice for every pixel, so a sprite-and-one-background test costs
twice §16.5's figure. Against §5.5's 1.5 ms for the PPU, modes 0 and 1 with windows and colour math now take 0.5 to
1.0 ms of picture, before modes 2 to 7, mosaic and hi-res. **P1's likely overrun of §15.9 still stands**, now with
its cause moved: the per-pixel compositor, not the fetch. The masks are constant between the window edges, so they
could be computed per run of pixels within a span; that is a structural question for the same decision, recorded
here and not built.

### 17.4 The 13 backdrop failures, read (2026-10-01)

| ROM | Where it stops | What it needs |
|---|---|---|
| `snestest_082506/test_hdma`, `blobs/test_hdma` | test 1 | **D-14**: an indirect terminator loads the pointer's high byte. With it the five registers are right; the test then stops on OPHCT `$3A` where it wants `$38`, the init two dots late; its own comment says the init's timing differs between console revisions. Open |
| `blobs/test_irqb` | test 7 | its disassembly's note: it uploads code to the sound CPU so the ports return `$18`; stage 4 |
| `blobs/test_irq4200` | (SRAM `$FF`) | which `$4200` enable sequences raise an IRQ at H=0 and H=338, against a stored table: the comparator on enable (anomie: "the IRQ output will go low even if the enable write occurs at the exact cycle"). Not read further |
| `nmi_irq/demo_irq`, `blobs/demo_irqtest` | test 6 | IRQ positions the counters cannot reach: H=339 on the short line, H=340, lines 262 and 263 with interlace; VenusRT has no interlace line count (stage 3's interlace step), and its comparator carries a point past the short line's end into the next line, which the test calls unlatchable. Not changed: the documents were not read for it in this step |
| `blobs/irq` | test `$2D` | not read |
| `blobs/nmi` | test `$1E` | not read |
| `blobs/test_hdmasync`, `blobs/test_hdmatiming` | 0 and `$51` | HDMA timing; not read |
| `snestest_082506/test_dma` | 0 | DMA overlapping HDMA on one channel (§15.6); not read |
| the two `test_dmavalid` | (no number) | not read |

None of the 13 passes yet; D-14 moved two of them to their next check.

### 17.5 Disputes

D-13 (opened, argued: anomie's sub pixel at any priority), D-14 (opened and implemented: the registers settled by
the test ROM, the init's timing open). The others as at §16.

### 17.6 The corpus again (measured 2026-10-01)

The runner over all 295 rows on the step's head, against §16's run: every verdict of all three engines is the
same (VenusRT 49 of Mesen's 117; the baseline test passes); VRAM, CGRAM and OAM equal Mesen's on 206, 272 and 290
rows, as before; **pictures equal on 162, from 157**, the five gained being the four window ROMs and
`color_halve_proof/demo`, and none lost. D-14 changed no verdict and no space.

### 17.7 The clone check (measured 2026-10-01)

18 files against Mesen's 164: no pair at 12 fingerprints, no shared run of literals, the largest pair below the
threshold still 4 (`cpu/mod.rs`), `ppu.rs` sharing none; the eleven names of §16.6, no new one.

### 17.8 The next step

Stage 3's remaining PPU work, in the plan's order: **modes 2 to 6** (4bpp and 8bpp backgrounds, mode 2's and 4's
offset-per-tile and 6's, direct colour), **mosaic**, **hi-res and pseudo-hi-res** (512-wide buffers, the sub
screen's half-pixels and their colour math, anomie's previous-main-pixel rule), **interlace** (the 263-line field,
which `demo_irq`'s test 6 also needs), then **mode 7**. Beside it: the compositor's per-pixel window masks, if the
decision of §17.3 extends to them, and the backdrop failures not yet read.

---

## 18. Stage 3, step 4: modes 2 to 4, direct colour, offset-per-tile, mosaic, and three DMA rules (2026-10-01)

### 18.1 What it built

- **Modes 2, 3 and 4** through §17.3's per-chunk decode: 16- and 256-colour tiles (two and four words a row), the
  palette bases anomie gives for each mode, and the two-background priority chart of modes 2 to 5.
- **Direct colour**: a 256-colour pixel under CGWSEL bit 0 is a colour itself, BBGGGRRR with the map entry's
  palette bits as each component's next bit; the line buffer carries it with a flag and the compositor converts it
  where it reads CGRAM for any other pixel.
- **Offset-per-tile** in modes 2 and 4, in the scroll function §17.3 left for it: visible tile T of BG1 or BG2 takes
  its scroll from visible tile T-1 of BG3's map, mode 4 reading one entry where mode 2 reads two (D-16). **It has no
  oracle**: no ROM in the corpus draws with it, and the bench games that use it do not reach it yet.
- **Mosaic**: a block's first pixel repeated over the filled line buffer, and its first line through a row counter
  that starts at the first picture line and takes a new size when a block ends (D-15). The counter is machine state
  and runs whether or not the picture is drawn.
- **Three DMA rules** from byuu's tests (§18.4): D-17 and D-18, and D-14's continuation.
- The state is version 9, with mosaic's row and height.
- **Not built: modes 5 and 6 and pseudo-hi-res.** They need the picture 512 wide: both screens' half-pixels, the
  compositor's loop and its hi-res colour math, the frame's width reported per frame, and the runner's comparison
  against a 512-wide Mesen picture. That is a step of its own, the next; mode 6's offset-per-tile comes with it.

### 18.2 The picture oracles (measured 2026-10-01)

Frame 300, VenusRT against Mesen, as §15.4:

| Set | ROMs | Equal to Mesen, §17 | Now |
|---|---|---|---|
| PeterLemon's PPU ROMs | 47 | 8 | 25 |
| the 240p suite | 2 | 2 | 2 |

The 17 gained: `8x8BGMap4BPP`, three of the four `8x8BGMap8BPP` sizes, `8x8BGMapTileFlip`, the five
`HiColor64PerTileRow` ROMs and `WaveHDMA` (mode 3, with HDMA writing CGRAM each line); `MosaicMode3`; **`WindowHDMA`
and `WindowMultiHDMA`** (mode 3 with windows); and the three `Blend/HiColor` ROMs (mode 3, the sub screen added),
which are the colour-math oracle §17.2 lacked. The fourth 8bpp size, `32x32`, scrolls by a counter it advances
whenever `$4210` reads set, which happens twice in some frames in both engines (a read in D-8's window leaves the
flag): its scroll advances 15 in ten frames in Mesen and in VenusRT alike, and VenusRT's frames 305 and 315 equal
Mesen's 300 and 310 in every pixel; the two engines are a constant five frames apart on it, not different in the
picture. Still different: the five `HiColor128PerTileRow` ROMs, whose CGRAM differs from Mesen's by 223 bytes at
the frame's end (HDMA writing 128 colours a tile row; not read), `RedSpace9BitHDMA` (D-7), and the modes 5 to 7,
pseudo-hi-res and interlace ROMs.

**Mosaic over time.** The 240p suite's intro fades by mosaic, a size a frame. Mesen runs it ten frames after
VenusRT, and its frames 60, 64 and 68 equal VenusRT's 50, 54 and 58 in every pixel, at sizes 5, 9 and 13.

**The corpus**, all 295 rows against §17.6: pictures equal on **169, from 162**, none lost; gained `demo_mode3`,
the two `vmain-8bpp` ROMs, `test_opt`, and the three ROMs of §18.4. CGRAM equal on 275 (272); VRAM and OAM as before.

**Modes 0 and 1 untouched.** `frame_hashes` over the 304 images to frame 3600 against §17.3's hashes: 292 images
identical in every frame; the 12 that differ are the ROMs that use this step's features (the 240p suites from their
mosaic intro, `demo_mode3`, the `vmain-8bpp` pair, `test_opt`, `SplitScreen`, `ppubusact`,
`inidisp_forgot_to_force_blank` and `hdma-2100-glitch-2ch-0a`, each listed twice where the corpus holds two copies).
The skip check passes on all 304.

### 18.3 Disputes

D-15 (mosaic's vertical start: fullsnes's counter built, the agreed case measured, mid-frame changes open); D-16
(offset-per-tile's columns: anomie's prose over his formula, argued, no oracle); D-17 and D-18 (settled by their
test ROMs, each with a named remainder).

### 18.4 The backdrop failures, continued (measured 2026-10-01)

| ROM | Before | Now | What |
|---|---|---|---|
| `snestest_082506/test_dma` | failed, test 1 | **passes** | D-17: an HDMA run or init ends a general DMA on the channels it takes; the DMA's address and count are live in the registers. Channel 1's registers after test 4 equal the console's; channel 0 stopped four bytes earlier than the console's |
| the two `test_dmavalid` | failed | **pass** | D-18: DMA between WRAM and `$2180` reaches nothing through the port, and a read of it writes `$00`; fullsnes has the rule, the test its details |
| `blobs/test_hdmasync` | 0 | stops at `$85` | 512 HDMA runs, one for each channel mask twice, each followed by two latches compared with a console's cached table; its header: "hdma begins at H=1100+DMA_counter". VenusRT now agrees with the table for the first `$85` masks; not read further |
| `blobs/test_hdmatiming` | `$51` | `$51` | not read |
| `blobs/test_irq4200` | — | — | §17.4; not read further |
| `blobs/irq`, `blobs/nmi` | `$2D`, `$1E` | same | binaries without source or disassembly; not read |

Of the 25 backdrop-graded ROMs VenusRT now passes **15** (12 at §17), Mesen 25, C# Venus 8; across the corpus
VenusRT passes 52 of Mesen's 117 (49). A unit test of stage 2 had itself used a WRAM-to-`$2180` DMA, which D-18
made move nothing; it now moves its bytes to OAM's port and pins D-18's case beside it.

### 18.5 The cost (measured 2026-10-01)

`frame_cost`, §17.3's binary and this step's in turn, best of three under the lock, load 2.3 falling to 1.75; the
picture's share, drawn less skipped:

| ROM | §17's | Now |
|---|---|---|
| gilyon `cputest-full` | 0.98 | 0.97 |
| the 240p menu | 0.91 | 0.99 |
| `Rings` | 0.96 | 0.98 |
| `object-dropout-test` | 0.79 | 0.81 |
| A Link to the Past | 0.80 | 0.84 |
| Super Metroid | 0.50 | 0.53 |
| `8x8BGMap8BPP64x64` (mode 3, one 256-colour background) | (not drawn) | 0.90 |
| `MosaicMode3` | (not drawn) | 0.89 |

The decode by depth and the mosaic pass cost 0.01 to 0.08 ms on modes 0 and 1. A 256-colour background costs what
the others do, about 0.9 ms with the compositor, against §5.5's 1.5 ms for the PPU. §17.3's finding stands: the
compositor's per-pixel windows and priority walks are the larger part, and were left as they are in this step.

### 18.6 The clone check (measured 2026-10-01)

18 files against Mesen's 164: no pair at 12 fingerprints, no shared run of literals (`DEPTHS` and the charts
included), the largest pair below the threshold still 4, `ppu.rs` sharing none. One new name, `mosaicsize`,
fullsnes's "Mosaic Size".

### 18.7 VenusRT against `EmuSen_CoreAPI.md` §13.2, read for stage 6

Nothing was changed for it; these are what would meet it:

- **The picture's size.** VenusRT's frame is one fixed 256×224 buffer and its frame info a constant. §6.6 reports
  the size every frame, 256 or 512 wide for VenusRT, with the host allocating from machine info's maximum. The
  hi-res step has to make the width vary within the present interface's frame info, and stage 6 inherits that.
- **`advance` and the frame's end.** §6.5 has `advance` take "the machine to the frame's end". VenusRT's ends at
  the first instruction boundary past it, and a DMA longer than a frame is one instruction, so one call can end two
  frames on (§15.7). A host that counts calls drifts, as the runner did. Either the specification says the frame
  count is the authority, or VenusRT must be able to stop inside a DMA.
- **No per-core C# class.** `Shim/VenusNative.cs` and `Shim/VenusMachine.cs` exist for the runner; stage 6 replaces
  them with the generic engine, and the runner's VenusRT engine and its space names (a C# array today) move to the
  core's own descriptors.
- **Smaller things:** VenusRT's own status, -9 for an image shorter than a bank, needs its place in §6.15's
  status space; it uses only bit 0 of the options and refuses every setting, which §6.5 and §6.13 already allow;
  it has no `present`, `phases`, axes, events or log queue yet.

### 18.8 The next step

**Hi-res**: modes 5 and 6 (16-pixel-wide tiles, even pixels to the sub screen and odd to the main), pseudo-hi-res
by SETINI, 512-wide line buffers and frame, the hi-res colour math anomie describes (the previous main pixel's
choice), mode 6's offset-per-tile, and the runner's comparison of a 512-wide picture. Then **interlace** with the
263-line field (which `demo_irq`'s test 6 needs), then **mode 7**. Beside it: `test_hdmasync` and `test_hdmatiming`
(the HDMA start, from the first's header and table), `test_irq4200`, and the compositor's masks if that decision
comes.

---

## 19. Stage 3, step 5: hi-res, interlace, and four interrupt and DMA rules (2026-10-01)

### 19.1 What it built

- **Hi-res.** Modes 5 and 6 take tiles 16 half-pixels wide (two tiles side by side whatever the size bit), the even
  half-pixels to the sub screen's line buffer and the odd to the main's; SETINI bit 3's pseudo-hi-res shows the sub
  screen's pixel left of the main's in any mode; both screens show colour 0 behind them (fullsnes, "Hires Notes");
  the sub half-pixel is clipped and mathed as the main pixel before it was (D-19); mode 6 has offset-per-tile
  through the same scroll function as modes 2 and 4; in true hi-res a mosaic block's first half-pixel fills both
  screens'. Mode 5 and 6's priority chart is mode 2's and mode 6's own.
- **The picture is 256 or 512 wide, and 224 or 448 high, by the frame.** Lines are drawn 512 wide into a canvas,
  a low-res pixel written twice, and the frame is presented at the start of V-Blank: 512 wide if a line of it was
  hi-res, 448 high with the two fields woven if a line was interlaced hi-res, else the 256-wide picture exactly.
  The interface's frame info reports the size; no export changed. A frame a ROM turns hi-res for some lines is 512
  wide for all of them, its low-res lines doubled, which is how the reference shows such a frame too.
- **Interlace.** SETINI bit 0 gives frames with the field flag clear a 263rd line and takes away the short line
  (anomie's timing document); modes 5 and 6 then draw the even or odd half-lines by the field, a mosaic block
  covering both; SETINI bit 1 gives the sprites every other row at half height. The line count is derived from the
  register, so the state stays version 9.
- **Four rules from byuu's tests** (§19.4): D-20 and D-21, and the runner's reading of a hi-res reference frame.

### 19.2 The runner and the reference's hi-res frames

The probe's screen buffer is always 512×478 (its glue reports 256×239 in the log whatever the frame). A low-res
frame fills the first 256×239 of it; a hi-res frame fills all of it, every line twice, and an interlaced one with
the two fields. The runner now tells them apart by whether anything lies past the first 256×239, and compares a
512-wide picture with it column for column on every other row, a 256-wide one pixel for pair, and a 448-high one
row for row with the offset doubled. A test pins the three.

### 19.3 The picture oracles (measured 2026-10-01)

- **PeterLemon's 47 PPU ROMs at frame 300: 32 equal to Mesen** (25 at §18), and the 240p suite's two as before.
  Gained: the seven interlaced mode 5 ROMs (`InterlaceFont`, `MosaicMode5`, `InterlaceMoogle`, `InterlaceScroll`,
  `InterlaceMystHDMA`, `InterlaceSimpsonsHDMA` and `InterlaceRPG`), every pixel of 512×448. The four pseudo-hi-res
  ROMs differ on 2,691 to 27,099 half-pixels, all D-19's (with the variant
  it records they equal Mesen in every half-pixel). Still different besides: mode 7's four, the five
  `HiColor128PerTileRow` ROMs (§18.2), `RedSpace9BitHDMA` (D-7) and `8x8BGMap8BPP32x32` (five frames apart, §18.2).
- **An offset-per-tile oracle exists after all**: lidnariq's `ppubusact` switches modes every 32 lines and makes
  offset-per-tile visible by HDMA on BG3VOFS. Its mode 2 and mode 4 bands equal Mesen's in every pixel, and differ
  on 3,294 and 3,300 without offset-per-tile; its mode 6 band differs on 86 half-pixels (2,295 without). D-16 is
  measured for modes 2 and 4. No commercial game in the bench reaches an offset-per-tile scene yet.
- **The corpus**, 295 rows: pictures equal on **173**, from 169, none lost: `SplitScreen` (interlaced mode 5) and
  the three ROMs of §19.4. CGRAM equal on 278 (275). VenusRT passes **55** of Mesen's 117 self-grading ROMs (52).
- **Modes 0 to 4 untouched by the canvas**: the frame hashes of all 304 images to frame 3600, against §18's, are
  identical on 299; the five that differ are §18.4's three DMA ROMs (whose rules came after §18's hashes),
  `ppubusact` and `SplitScreen`. The skip check passes on all 304.

### 19.4 The backdrop failures, continued (measured 2026-10-01)

| ROM | Now | Rule |
|---|---|---|
| `nmi_irq/demo_irq`, `blobs/demo_irqtest` | **pass** (stopped at test 6) | D-20: an IRQ point past its line's end is lost over the short line's end and a frame's, with interlace's line counts |
| `blobs/test_irq4200` | **passes** | D-21: V-IRQ selected on its own line, past its point, is raised by the `$4200` write; its record equals the stored table |
| `blobs/test_hdmasync` | stops at `$85` | the HDMA start at H=1100 plus the DMA counter's phase, from its header; not built |
| `blobs/test_hdmatiming` | stops at `$51` | not read |
| both `test_hdma` | test 1 | D-14's open init timing |
| `blobs/test_irqb` | test 7 | the sound CPU (stage 4) |
| `blobs/irq`, `blobs/nmi` | `$2D`, `$1E` | no source or disassembly |

Of the 25 backdrop-graded ROMs VenusRT passes **18** (15 at §18; 21 of the wider 28-ROM list of §16).

### 19.5 The cost (measured 2026-10-01)

`frame_cost`, §18's binary and this step's in turn, best of three under the lock, load 2.7 falling to 2.3 (above
§18's 1.75, so the absolute figures are a little high); the picture's share, drawn less skipped:

| ROM | §18 | Now |
|---|---|---|
| gilyon `cputest-full` | 0.98 | 1.12 |
| the 240p menu | 1.01 | 1.18 |
| `Rings` | 0.98 | 1.21 |
| `object-dropout-test` | 0.82 | 1.00 |
| A Link to the Past | 0.86 | 1.00 |
| Super Metroid | 0.54 | 0.69 |
| `8x8BGMap8BPP64x64` | 0.92 | 1.14 |
| `MosaicMode3` | 0.92 | 1.13 |
| `HiColor64PerTileRowPseudoHiRes` (pseudo-hi-res, sub screen) | 1.53 (drawn low-res) | 2.45 |
| `InterlaceRPG` (mode 5, interlace) | 0.65 (backdrop only) | 2.07 |

**The canvas costs every ROM 0.14 to 0.23 ms**: each pixel is written twice into a 512-wide line and the frame is
copied out at V-Blank. **A hi-res line costs about twice a low-res one**, its sub half-pixel being a second
priority walk and a second math. Against §5.5's 1.5 ms for the PPU: low-res modes 0 to 4 now take 1.0 to 1.2 ms of
picture, hi-res 2.0 to 2.5. The overrun §15.9 predicted is here for hi-res; it is recorded, not tuned, as decided.
The compositor's per-pixel structure, whose rework awaits a decision, is most of both figures. Writing low-res
frames straight to a 256-wide picture, and doubling only when a hi-res line appears, would take back most of the
canvas's share; that is a structural choice of the same kind, recorded here and not built.

### 19.6 Disputes

D-19 (hi-res colour math: anomie's rule built, Mesen's picture equal to a variant; open for the MiSTer referee);
D-20 and D-21 (settled by their test ROMs); D-16 measured for modes 2 and 4.

### 19.7 The clone check (measured 2026-10-01)

18 files against Mesen's 164: no pair at 12 fingerprints, no shared run of literals, the largest pair below the
threshold still 4 (`cpu/mod.rs`), `ppu.rs` sharing none. Two new names, `objinterlace` (anomie's "OBJ Interlace")
and `tilerow`, a plain one.

### 19.8 The interface, again

`advance`'s stopping rule (§18.7) is unchanged and still recorded: a DMA longer than a frame ends one call two
frames on. The picture's size now varies by frame within the present frame info, as the stable API will want.

### 19.9 The next step

**Mode 7**: the matrix and its centre, the screen flips, the playing field's outside (wrap, transparent, tile 0),
EXTBG's BG2 with its priority bit, direct colour, mosaic's odd case, and mode 7's write-twice registers and the
multiplier at `$2134`; its oracles PeterLemon's four mode 7 ROMs, `StarWars` and the games' mode 7 frames (Super
Metroid's title). Beside it: D-19 as a referee dispute step (the SNES_MiSTer RTL), mode 6's 86 half-pixels in
`ppubusact`, `test_hdmasync`'s HDMA start, and the canvas and compositor costs if those decisions come.

---

## 20. Stage 3, step 6: mode 7, D-19 refereed, and the compositor's two structural changes (2026-10-02)

### 20.1 Mode 7

Built from fullsnes ("Rotation/Scaling") and anomie (`$211A`-`$2120`, "Mode 7"), which agree: the matrix with the
origin's and the line's products rounded to a quarter pixel and the pixel's product not; the 13-bit centre and
scroll, the scroll less the centre kept to ten bits and a sign; the shared write-twice byte M7_old, through which
`$210D` and `$210E` write mode 7's scroll beside BG1's; M7SEL's screen flips and the field's outside (wrapped,
transparent, or tile 0's pixels); direct colour; EXTBG's BG2 from the same pixels with bit 7 its priority, and its
mosaic taking vertical blocks from BG1's enable bit and horizontal ones from its own; mode 7's two priority charts;
and `$2134`-`$2136`, M7A times M7B's high byte, signed. **Not modelled:** the products `$2134` returns during mode 7's
drawing (fullsnes lists eight per pixel), and a mode 7 picture in hi-res. Mode 7 fills the same line buffers by its
own path. The state is version 10, with the mode 7 registers.

**Oracles (measured 2026-10-02):**

- PeterLemon's `RotZoom`, `Perspective` and `Mode7HDMA` equal Mesen's picture in every pixel at frame 300.
  `StarWars` differs on about 2,500 pixels scattered over the screen, with VRAM, CGRAM and OAM equal to Mesen's at
  the frame and no pairing of neighbouring frames closer; not read further.
- undisbeliever's six VMAIN mode 7 ROMs (`vmain-mode7-tilemap-columns` and `-rows`, `vmain-mode7-image-*`) and
  `setini-early-read-mode7ex` now equal Mesen in the corpus run.
- Super Metroid's title equals Mesen at every twentieth frame from 240 to 400, but its mode 7 frames show the sprites
  alone (TM `$10`), so they do not test the mode 7 background; its state parts after 420, as before.
- PeterLemon's 47 PPU ROMs: **35 equal** (32 at §19). The corpus: pictures equal on **180** of 295 (173), none lost.

### 20.2 D-19, refereed

A dispute step of its own, which changed no code: `Venus_Referee.md` §0, then SNES_MiSTer's `rtl/PPU.vhd`, its
colour-math process only. The referee maths the sub half-pixel with the previous main pixel's colour before math,
under that pixel's enable, halving and clip, which is anomie's sentence read literally and the rule built. D-19 is
**settled** for the built rule; Mesen's pictures of the four pseudo-hi-res ROMs are recorded as the ones that differ,
and the variant they agree with is not adopted. The left edge stays open between "unmathed" (built) and the
referee's "mathed with black". The entry lists what was read.

### 20.3 The two structural changes (decided 2026-10-02)

*Decided 2026-10-02*, as an extension of the rework decision of 2026-10-01 (§17.3): (1) the compositor's six window
masks are found once for each run of pixels between the windows' edges, instead of for every pixel; (2) a low-res
frame is drawn straight into a 256-wide picture, swapped in at V-Blank with no copy, and the 512-wide canvas is used
only from a frame's first hi-res line, the lines drawn before it doubled into it. A blank line does not make a frame
hi-res, as before.

**Proof (measured 2026-10-02).** `frame_hashes` over all 304 images to frame 3600, built before the two changes,
after the first, and after the second: **1,094,067 frames, every hash equal at each change**, hi-res and interlaced
ROMs included. A unit test checks the run masks against the per-pixel ones over 200 random window settings, and one
pins a frame that turns hi-res at line 100 (the earlier lines doubled, presented 512 wide) and the low-res frame
after it (256 wide again). The skip check passes on all 304; the crate's 46 tests pass.

**The cost**, `frame_cost`, the build before the changes, after the first, and after both, best of three under the
lock, load 1.3 to 1.45; the picture's share, drawn less skipped, in ms:

| ROM | Before | Masks per run | And the 256-wide picture |
|---|---|---|---|
| gilyon `cputest-full` | 1.29 | 1.06 | 0.95 |
| the 240p menu | 1.38 | 1.12 | 1.05 |
| `Rings` | 1.39 | 1.17 | 1.08 |
| `object-dropout-test` | 1.05 | 0.84 | 0.75 |
| A Link to the Past | 1.01 | 0.78 | 0.68 |
| Super Metroid | 0.78 | 0.58 | 0.50 |
| `8x8BGMap8BPP64x64` (mode 3) | 1.17 | 0.95 | 0.90 |
| `MosaicMode3` | 1.18 | 0.95 | 0.89 |
| `HiColor64PerTileRowPseudoHiRes` | 2.72 | 2.20 | 2.12 |
| `InterlaceRPG` (mode 5, interlaced) | 2.06 | 1.60 | 1.58 |

**The masks per run take 0.2 to 0.5 ms off the picture, and the 256-wide picture another 0.05 to 0.11** on low-res
frames; on the two hi-res ROMs, which still use the canvas, 0.02 and 0.08, within the runs' spread. The "before"
column is 0.15 to 0.2 ms above §19.5's "now"; mode 7's added branches are the likely part of it (argued, not
measured apart from the spread between runs). Against §5.5's
1.5 ms for the PPU: **low-res modes 0 to 4 now take 0.5 to 1.1 ms of picture, inside the budget; hi-res and
interlaced frames 1.6 to 2.1 ms, still over it.** P1's overrun of §15.9 and §19.5 is retired for low-res frames and
stands for hi-res ones, where the sub half-pixel's second priority walk and math is the cost.

### 20.4 The clone check (measured 2026-10-02)

18 files against Mesen's 164: no pair at 12 fingerprints, the largest below it still 4. One run of nine literals was
reported, `ppu.rs`'s window-mask test listing the register numbers `0x23` to `0x2B`, nine consecutive integers that
a table in Mesen's DSP code also holds; the list is now written as a range, and the check reports none. One new
name, `mode7extbg`, anomie's "Mode 7 EXTBG".

### 20.5 Is the PPU stage complete?

**Every feature of the plan's stage 3 row is built**: the ports and their timing, backgrounds in modes 0 to 7,
sprites and their evaluation, windows, colour math, mosaic, offset-per-tile, direct colour, hi-res and
pseudo-hi-res, interlace and mode 7, in spans, with the skip-versus-draw test passing on every image. **What it
leaves open, named:** D-6 (power-on phase), D-7 (brightness rounding), D-11 and D-12 (OAM reload and writes during
display), D-15 (mosaic mid-frame), D-16 for mode 6's 86 half-pixels in `ppubusact`, D-19's left edge; `StarWars`'s
scattered pixels; the five `HiColor128PerTileRow` ROMs' CGRAM (HDMA writing colours a tile row); the products of
`$2134` during mode 7; and from the S-CPU side, `test_hdmasync`'s and `test_hdmatiming`'s HDMA start and both
`test_hdma`'s init timing. None of these blocks stage 4. **Stage 3 is complete in the plan's sense**, with those
recorded as named exceptions for §7's gate.

### 20.6 What stage 4 needs first

The plan's §5.3 and stage 4 row. In order:

1. **The SPC700 against SingleStepTests' SPC700 suite with cycle lists**, through the harness stage 0 built for it
   (`singlestep/spc700.rs`, its bus trait and the three readings of §2.3): the CPU and its own bus, nothing else.
2. **The APU's side of the machine**: the IPL ROM, the four ports both ways, the three timers, and the clock domain
   of §5.3 (24.576 MHz, an exact rational to the master clock, catch-up at a port access and the frame's end). That
   replaces the APU stand-in of §14.4, which is what Super Mario World and Donkey Kong Country stall on, A Link to the
   Past and the 240p suite run ten frames early against, and `test_irqb` needs.
3. **gilyon's `spctest`** on the machine, then blargg's `spc_smp`, `spc_timer` and `spc_mem_access_times` and the 2010
   tests.
4. **The S-DSP**: BRR, envelopes, echo, noise, pitch modulation, one of its 32 steps per SPC700 cycle, into the 32 kHz
   queue; graded by `spc_dsp6` and by audio against Mesen.

The first two are what most of the remaining picture differences wait for, because the games' states part from
Mesen's where the stand-in answers at once.


---

## 21. Stage 4, step 1: the SPC700 and the S-SMP around it (2026-10-02)

### 21.1 The SPC700

`apu/spc700.rs`, written from fullsnes ("SPC700 CPU" and its opcode tables) and anomie's SPC700 cycle document
(romhacking.net document 198), against a bus of three calls: a read, a write, and an internal cycle with no address.
Every opcode makes its bus cycles in the order anomie gives where he marks it verified, and in the order the suite
records where he leaves it open (D-22); an internal cycle is either a dummy read of the next program byte or a wait,
as the suite records it. SLEEP and STOP halt the CPU, and the harness steps the halted CPU on to the case's cycle
count. DIV divides bit-serially, as the referee's divider does (D-23); the documents give only the case whose
quotient fits in a byte.

**SingleStepTests' SPC700 suite with cycle lists (measured 2026-10-02):** 256,000 of 256,000 cases pass on
registers, memory and every cycle's address, value and kind. The suite's one known quirk is kept as §2.3 reads it: a
dummy read of memory the case does not list has no recorded value, and is checked on its address and kind only. The
old core's baseline (§3) was 255,105 on state alone, without cycles.

| Group | Opcodes | Cases | Passing, with cycles |
|---|---|---|---|
| ALU (ADC, SBC, AND, OR, EOR, CMP in every mode) | 78 | 78,000 | 78,000 |
| branches (relative, BBS/BBC, CBNE, DBNZ) | 29 | 29,000 | 29,000 |
| shift, rotate, increment, decrement | 28 | 28,000 | 28,000 |
| one-bit (SET1, CLR1, AND1, OR1, EOR1, NOT1, MOV1) | 24 | 24,000 | 24,000 |
| jump, call, return (JMP, CALL, PCALL, TCALL, BRK, RET, RETI) | 23 | 23,000 | 23,000 |
| load | 18 | 18,000 | 18,000 |
| store | 17 | 17,000 | 17,000 |
| stack and transfer | 14 | 14,000 | 14,000 |
| flags and control (incl. SLEEP, STOP) | 11 | 11,000 | 11,000 |
| 16-bit (MOVW, INCW, DECW, ADDW, SUBW, CMPW) | 7 | 7,000 | 7,000 |
| decimal, XCN, test-and-set (DAA, DAS, XCN, TSET1, TCLR1) | 5 | 5,000 | 5,000 |
| multiply and divide | 2 | 2,000 | 2,000 (1,240 before D-23) |
| **All** | 256 | 256,000 | 256,000 |

### 21.2 The S-SMP

`apu/smp.rs` replaces §14.4's stand-in. From fullsnes's "SNES APU" chapters:

- **64 KiB of RAM and the IPL ROM** over its last 64 bytes while CONTROL bit 7 is set; writes go to RAM beneath the
  ROM and the I/O page, while TEST bit 1 allows them. TEST is $0A and CONTROL $B0 at power-on, and the PC starts at
  the ROM's reset vector.
- **The four ports both ways:** $2140-$2143 (and their mirrors to $217F) write the SPC700's inputs and read its
  outputs; CONTROL bits 4 and 5 clear the input pairs. The OR of old and new values that fullsnes describes for a read
  racing a write is not modelled, since the two sides do not run in the same cycle (§21.3).
- **The three timers:** a first stage of 128 SPC700 cycles for timers 0 and 1 (8 kHz) and of 16 for timer 2 (64 kHz),
  counting toward TnDIV (0 meaning 256), then TnOUT's four bits, cleared by a read. TEST bits 0 and 3 gate the count.
  A timer starts again from zero when its CONTROL bit goes from 0 to 1, and a cleared bit only stops it (D-25,
  measured on `spc_smp`; the referee's read under D-26 agrees). **Not built:** TEST bits 4 to 7, the waitstates and
  the timers' step they change (D-27).
- **The S-DSP is a stand-in:** 128 registers, readable and writable through $F2/$F3, and silence. It is step 2.
- **Reads of TEST, CONTROL and TnDIV return 0.** fullsnes marks them write-only and does not say what a read returns;
  no oracle in this step distinguished another value.

**The boot ROM is firmware the frontend supplies.** The plan's §4.2 covers firmware for the NEC DSPs only, asked for
from the header and loaded from `home/Firmware/` through `FirmwareLibrary`, where a missing image is never fatal.
The IPL follows its path and name convention, `spc700.rom` in that folder, and departs from its fallback, since
without the boot ROM no game passes its first handshake with the sound CPU: it is file 1 of the native interface's
`create`, exactly 64 bytes (another size is `BAD_FILE`), and without it `create` returns VenusRT's own status -10,
which the shim raises as `FileNotFoundException`. An image too short is still -9, checked first. Nothing in the
repository holds the ROM: the test fixtures use a synthetic two-instruction stand-in, and the corpus runner reads
`firmware/spc700.rom` beside the corpus, then `FirmwareLibrary`'s folder. The interface's exports are unchanged.

### 21.3 The clock domain

The plan's §5.3: the SPC700 at 1.024 MHz, carried against the master clock as an exact rational (5,632/118,125 for
NTSC, 102,400/2,128,137 for PAL), run behind the S-CPU and caught up at every access to $2140-$217F and at the frame's
end. The catch-up runs whole instructions, so the SPC700 may finish up to an instruction's cycles (at most 12, about
250 master clocks) past the S-CPU's time; a port written by the S-CPU then reaches it a few cycles late. That is
argued to be below anything the corpus measures, and is recorded as the granularity, not a rule. The state is
version 11, with the S-SMP's registers, timers and cycle count in an `Apu` group and its RAM in the RAM block.

### 21.4 The oracles (measured 2026-10-02)

The corpus with VenusRT as the third engine, after D-25:

| ROM | Mesen | VenusRT before (§20) | VenusRT now |
|---|---|---|---|
| gilyon `spctest` (both copies) | Passed | Incomplete | **Passed** (tests 0557 and 0527, "Success") |
| blargg `spc_smp` | Passed | Incomplete | **Passed** (after D-25; before it, failed at "Timers/random timer0 enable") |
| blargg `spc_timer` | Passed | Incomplete | **Passed** |
| blargg `spc_mem_access_times` | Passed | Incomplete | **Passed** |
| PeterLemon's seven SPC700 ROMs | Passed | Incomplete | **Passed** |
| blargg 2010 `exec_io_tests` 1-4 | Passed | not graded or Failed | **Passed** |
| blargg 2010 `test_ram_disable_ipl` | Passed | Failed | **Passed** |
| blargg 2010 `test_timer_stop`, `test_timer_speed_2` | Passed | not graded | **Passed** |
| blargg 2010 `test_timer_stop2` | Passed | not graded | Failed, 00 against 04 (D-26) |
| blargg 2010 `test_timer_speed`, `test_timer_speed2` | Passed | not graded | Failed, 2731 for every TEST setting (D-27) |
| blargg 2010 `test_speed`, `test_timer_speed3` | Done | — | Done; print constant counts where Mesen's vary with TEST (D-27) |
| blargg 2010 `speed_2_freezes2` | Failed | — | Failed, as Mesen |
| KungFuFurby `test_irqb` | Passed | Failed | Failed: cases 1 to 4 pass, case 5 leaves the record one byte early (D-24) |
| blargg `spc_dsp6` | Passed | Incomplete | Incomplete (step 2) |

Over the corpus: **74 of the 117 ROMs Mesen passes now pass** (55 at §20), and none that passed before fails. Of the
43 left, 32 are the GSU and SA-1 ROMs of stage 5, one is `spc_dsp6`, three are D-26 and D-27, and seven are byuu's
backdrop-graded IRQ, NMI and HDMA ROMs (`irq`, `nmi`, `test_hdma` twice, `test_hdmasync`, `test_hdmatiming`,
`test_irqb`), all failing before. Pictures equal to Mesen's at the last frame: **199** of 295 (180); VRAM equal: 225
(206). The skip-versus-draw check, whose state now includes the S-SMP's, holds on all 304 images over 600 frames.

### 21.5 The games (measured 2026-10-02)

`The_games_pictures_and_spaces_against_Mesen`, every tenth frame to 600; first frame each part differs, and how many
of the 60 sampled pictures differ (a column added to the table in this step):

| Game | First lit (Mesen / VenusRT) | VRAM | CGRAM | OAM | Picture | Pictures differing | At frame 600 |
|---|---|---|---|---|---|---|---|
| the 240p suite | 40 / 40 | 80 | 80 | never | never | 0 of 60 | all equal |
| A Link to the Past | 90 / 90 | 90 | never | 380 | 170 | 2 of 60 | all equal |
| Donkey Kong Country | 80 / 80 | 70 | 80 | never | never | 0 of 60 | all equal |
| Super Metroid | 20 / 20 | never | 310 | 430 | 20 | 3 of 60 | CGRAM 4 bytes; picture equal |
| Super Mario World | 90 / 90 | 90 | 410 | 420 | 200 | 7 of 60 | VRAM 230 bytes; picture equal |

At §20 Super Mario World and Donkey Kong Country never lit (forced blank, waiting on the stand-in), and A Link to the
Past lit at the sampled frame 60 against Mesen's 90; §14.1 and §20.6 had it and the 240p suite running about ten
frames ahead, by the stand-in's instant answers. **All four of the plan's cases are answered:** the two stalled games
pass their forced blank and light on Mesen's frame, and A Link to the Past and the 240p suite no longer lead. Where a
space differs early and is equal later, the difference is a transient of a frame or two (an upload landing one
sampled frame apart), not a divergence; Super Mario World's VRAM at 600 and Super Metroid's four CGRAM bytes are the
standing differences, not read further. The games run with silence: no game in the set waits on a DSP register.

### 21.6 Disputes

D-22 (the SPC700's cycle order where anomie is open) and D-23 (DIV, settled by the referee and measured) before the
SPC700's code; D-24 to D-27 after the corpus run, before the timer change. D-25 is settled by measurement. D-26 read
the referee's timer process (`rtl/SMP.vhd` lines 210-360), which agrees with VenusRT's timers in every rule but one,
the step TEST's speed bits set, and argues `test_timer_stop2` waits on D-27. D-24 is a CPU-side question and stays
open with one reading rejected by measurement. The entries list what was read.

### 21.7 The clone check (measured 2026-10-02)

22 files against Mesen's 164, with fullsnes, anomie's documents, the datasheet, jonasquinn's `muldiv_tests` and the
other Rust cores as vocabulary: **no pair shares 12 fingerprints, no table of eight literals is shared, and no new
identifier of Mesen's appears.** `apu/smp.rs` and `singlestep/spc700_cpu.rs` share no fingerprint with any Mesen
file; `apu/spc700.rs` shares 4 with `SpcTypes.h` and 3 with `SnesCpuTypes.h`, the processor-status flag names, the
same counts as `cpu/mod.rs` has with those two files.

### 21.8 The cost (measured 2026-10-02)

Under the timing lock, load 1.2 falling to 1.1, best of three over 600 frames. `apu_cost` copies the S-SMP out of a
machine at frame 600 and runs it alone for 600 more frames' worth of master clock with the ports held; the frame
costs are `frame_cost` with the picture drawn and skipped, §20's binary (the stand-in) against this step's:

| ROM | SPC700 alone | Skipped, §20 | Skipped, now | Drawn, §20 | Drawn, now |
|---|---|---|---|---|---|
| the 240p menu | 0.049 | 0.309 | 0.368 | 1.359 | 1.405 |
| A Link to the Past | 0.059 | 0.483 | 0.549 | 1.152 | 1.186 |
| Super Metroid | 0.055 | 0.528 | 0.617 | 1.024 | 1.145 |
| gilyon `spctest` | 0.051 | 0.466 | 0.559 | 1.408 | 1.503 |
| Super Mario World | 0.060 | (stalled) 0.386 | 0.560 | (blank) 0.397 | 1.607 |
| Donkey Kong Country | 0.053 | (stalled) 0.547 | 0.546 | (blank) 0.561 | 1.660 |

**The SPC700 costs 0.05 to 0.06 ms a frame alone** (17,038 cycles, about 3.3 ns a cycle), and 0.06 to 0.09 ms in the
machine, the difference being the catch-up at each port access of a polling loop. Against §5.5's 0.5 ms for the
SPC700 and the DSP together, that leaves about 0.4 ms for the DSP. The two games that stalled before now draw
their pictures, at 1.6 to 1.7 ms a frame, inside P1's 2.8 ms with the DSP still to come. No tuning was done.

### 21.9 What step 2 is

**The S-DSP**, from anomie's S-DSP document (romhacking.net 191, whose 32-step sample loop places every register
access and the timers' first-stage ticks) and fullsnes's "SNES APU DSP" chapters: BRR decoding, the ADSR and GAIN
envelopes, the Gaussian interpolation, noise, pitch modulation, echo with its FIR filter and its buffer in APU RAM,
KON and KOFF with their every-other-sample poll, ENDX, ENVX and OUTX, and FLG's reset and mute; one of its 32 steps
per SPC700 cycle (§5.3), so the timers' first stage moves into the DSP's loop where anomie places it; the 32 kHz
stereo into the native interface's audio, which the exports already carry. **Oracles:** blargg `spc_dsp6`, and the
audio against Mesen's through the probe's sample dump on the four games and the 240p suite's sound tests. **Also
owed:** D-27's waitstates and timer step (then `test_timer_stop2` again under D-26), and the cost against §5.5's
0.4 ms left. D-24 belongs to a CPU step.

---

## 22. Stage 4, step 2: the S-DSP, D-27, and the sound against Mesen's (2026-10-02)

### 22.1 What it built

`apu/dsp.rs`, written from anomie's S-DSP document (romhacking.net 191, revision 1212) and fullsnes's "SNES APU
DSP" chapters, in place of §21's stand-in. The DSP takes one of its 32 steps in each 1.024 MHz cycle, as the plan's
§5.3 recommends, and each step does what anomie's sample loop places there: the voices' nine steps interleaved,
the echo's reads at 22 and 23, the FIR coefficients at 22 to 25, the outputs at 26 and 27, NON, EON and DIR at 28,
the counter, EDL, ESA and the left echo write at 29, the right echo write, the noise and the KON/KOFF poll at 30.

- **BRR:** 9-byte blocks decoded four samples at a time into a 12-sample ring of three groups, the four filters as
  both documents give them, clamped to 16 bits and clipped to 15; shifts 13 to 15 as fullsnes and anomie agree. The
  end and loop flags set ENDX, take the loop address at the next sample's S2, and an end without loop releases the
  voice with its envelope at zero.
- **Interpolation:** the 512-entry table both documents list (identical, checked by script), and fullsnes's
  rounding, since the two documents' formulas differ (D-28, open: no oracle in this step separated them).
- **Envelopes:** ADSR, the four GAIN modes and direct gain, run every sample on the global counter with anomie's
  rates and offsets, the new value computed every sample and applied when the counter fires; the phase changes and
  the bent increase's memory follow the new value either way (anomie's four-point list).
- **Key-on:** the five start-up samples of anomie's "BRR DECODING", the three groups decoded at the second to fourth;
  KON and KOFF taken on the poll's samples only, every other one, and ENVX showing the envelope applied to the
  sample before its update (D-29).
- **Noise, pitch modulation, volumes, echo:** the 15-bit noise generator, PMON from the previous voice's output, the
  voice volumes summed into the main and echo pairs with 16-bit clamps after each addition, MVOL and EVOL, the FIR
  over eight samples with only its last addition saturating, EFB, the echo ring by ESA and EDL with EDL applied at
  offset 0, and FLG's reset, mute and echo-write bits. FLG acts as $E0 at power-on.
- **Output:** the stereo pair at cycles 26 and 27 into the machine's sample queue, 32 kHz, before fullsnes's
  "final phase inversion", which it attributes to the post-amplifier outside the chip.

The DSP's state beyond its registers is packed into a fixed 568-byte block beside them; the state is version 13
with D-27's prescalers.

### 22.2 D-27, settled: TEST's speed bits

The 2010 ROMs' counts are explained exactly by one rule, the referee's (read under D-26) and blargg's own notes
beside the ROMs: the timers' first stage advances by 2^(bits 7-6) + 2·2^(bits 5-4) each SPC700 cycle and ticks at
384 (timers 0 and 1) and 48 (timer 2). And `test_speed`, as Mesen prints it, shows bits 6-7 stretching every SPC700
cycle to 1, 2, 5 or 10 of the 1.024 MHz cycles and bits 4-5 slowing nothing, which is fullsnes's 0/1/4/9 applied to
every cycle, not its RAM-versus-I/O split. Both are built; the timers' part of a cycle now follows the SPC700's
access in it. `test_timer_speed` and `test_timer_speed2` pass; `test_timer_speed_2`, `test_timer_speed3` and
`test_speed` print Mesen's counts to within one. **D-26 is not explained by it:** `test_timer_stop2` still prints 00
against 04, and the log of its accesses (TEST alternating $0B and $0A every four cycles for about 110 cycles)
shows that 04 cannot come from the 8 kHz first stage at all. The prediction of §21.6 and D-26, that D-27 would
settle it, is retired.

### 22.3 The oracles (measured 2026-10-02)

**blargg's `spc_dsp6`:** 21 of its 104 tests pass in order, the eight echo tests, the seven envelope tests and six
of the key-on tests, and it stops at "KON/kon decoding when another kon" (each round prints 0000 five times, FFFE
twice and 0000 three times, whichever round). Two rules came from it on the way, D-29's. The rest of the list
(more key-on cases, the "Misc", "Order", "Random" and "Timing" groups) is not reached, so this is a lower bound, not a
grade.

**The sound against Mesen's**, by the differential's loudness envelope (sixtieth-of-a-second windows, Pearson's r at
the best lag within half a second), over 600 frames of each game, with a column added to the games table:

| Game | r | Lag | Pictures equal at 600 |
|---|---|---|---|
| A Link to the Past | 0.992 | 0 | yes |
| Donkey Kong Country | 0.997 | 0 | yes |
| Super Metroid | 0.997 | 0 | yes |
| Super Mario World | 0.986 | 0 | yes |
| the 240p suite's menu | — | — | silent in both |

The four games' loudness follows Mesen's frame for frame with no lag. The envelope measures loudness, not the
waveform: a difference in a sample's low bits, or a phase inversion, is invisible to it.

**The corpus**, after both: **76 of the 117 ROMs Mesen passes now pass** (74 at §21), the two timer-speed ROMs the
gain; `spc_dsp6` moves from Incomplete to Failed; nothing that passed fails. The pictures, the games' tables and
the skip-versus-draw check (all 304 images, 600 frames, the DSP's state included) are as at §21.

### 22.4 Disputes

D-28 (interpolation's rounding) logged before the code, open. D-29 (key-on and the hidden envelope) opened by
`spc_dsp6`, the referee's DSP read for that rule (`rtl/DSP.vhd` lines 925-1000, 1075-1090, 1108-1120 and
1185-1270), and settled for its two rules by measurement. D-27 settled by measurement; D-26 measured again and left
open. Each entry lists what was read.

### 22.5 The clone check (measured 2026-10-02)

23 files against Mesen's 164. **One pair passes the threshold and two tables are shared, and all three are the
documented constants:** `apu/dsp.rs` shares 115 fingerprints with Mesen's `DspInterpolation.h`, and the check's table
runs are the 512-entry Gaussian table (in both fullsnes and anomie) and the 32 counter rates (anomie's
`counter_rates`). Run again on `dsp.rs` with those two tables blanked, it reports no table and a largest pair of 8
fingerprints, `DspTypes.h`, the register names fullsnes gives. Three new names are Mesen's too: `brroffset`,
`echooffset` and `echolength`, anomie's "echo offset" and the plain names of the BRR pointer's offset and the
ring's length.

### 22.6 The cost (measured 2026-10-02)

Under the timing lock, load 3.6 falling to 2.3 (higher than §21's 1.1, so the absolute figures are a little high),
best of three over 600 frames; §21's binary against this step's, and `apu_cost`, the sound unit alone:

| ROM | Sound unit alone, §21 / now | Skipped, §21 / now | Drawn, §21 / now |
|---|---|---|---|
| the 240p menu | 0.050 / 0.173 | 0.367 / 0.507 | 1.405 / 1.543 |
| A Link to the Past | 0.059 / 0.234 | 0.550 / 0.690 | 1.186 / 1.320 |
| Super Metroid | 0.055 / 0.199 | 0.623 / 0.738 | 1.140 / 1.250 |
| Super Mario World | 0.060 / 0.230 | 0.561 / 0.694 | 1.596 / 1.729 |
| Donkey Kong Country | 0.053 / 0.182 | 0.548 / 0.701 | 1.651 / 1.782 |

**The SPC700 and the DSP together cost 0.17 to 0.23 ms a frame alone, and 0.11 to 0.15 ms more than §21 in the
machine**, against §5.5's 0.5 ms for the two. Every bench game draws at 1.25 to 1.78 ms a frame, inside P1's 2.8 ms
with the sound unit complete. No tuning was done.

### 22.7 What is left of stage 4

- `spc_dsp6` beyond its 22nd test, one case at a time, starting with "kon decoding when another kon".
- D-28, which a later `spc_dsp6` case or an audio sample comparison against Mesen's (not its envelope) can decide.
- D-26 and D-24, the one SPC700-side and one CPU-side case left from step 1.
- The sound's waveform against Mesen's at the sample level, which the envelope does not measure; the probe's WAV is
  48 kHz, so it needs the resampler §5.3 describes, or a 32 kHz dump.

---

## 23. Stage 4, step 3, and the stage's close (2026-10-02)

### 23.1 `spc_dsp6`, test by test

The whole ROM stops at its first failure, so the step began by making the rest of it reachable. **Each of its 111
tests was run on its own**, from a copy of the ROM (built in scratch, never committed) whose every test block holds
that one test; a test passes when its first run completes without "Failed". Mesen passes every such copy that
VenusRT fails, which checks that the tests do not depend on the ones before them. The rules found (each a dispute
entry or a document's sentence, as marked):

| Rule | From | Tests it brought |
|---|---|---|
| The key-on sample keeps its own BRR decode; ring, position and block offset reset on the first start-up sample | anomie ("#0 ... the final pre-KON BRR decode also occurs here"), D-29's third rule | "kon decoding when another kon" and the eleven key-on tests after it |
| A voice's ENVX or OUTX written between the DSP preparing it and writing it keeps the SMP's value; an ENDX write also clears the prepared ENDX | anomie, VxENVX/VxOUTX/ENDX ("a write by the SMP to this register up to 2 cycles earlier will overwrite the DSP's updated value") | "endx write clears immediately", "endx after final brr decode", "V7 endx set", "V8 outx", "V9 envx" |
| The loop address is the one the voice's S2 read in the sample whose block ended | D-30 (anomie's S4 sentence read literally fails) | "loop addr read in prev sample", "V1 srcn.loop", "V2 dir.loop.lsb/msb", "28 dir" |
| The noise moves before voice 0's step in cycle 30 | D-30 (no document gives the order) | "noise rate flg.1F", "voice 0 noise" |
| EFB is read once, at cycle 26, for both sides | anomie ("EFB is accessed during cycle 26") | "26 efb" |

**Measured 2026-10-02: 106 of the 111 tests pass on their own** (21 at §22). The whole ROM, run in order, now
passes 33 and stops at its 34th, "Misc/$F0-$FF are not ram". Left: that test; "Random/brr while playing",
"Random/echo data" and "Random/envelope", which print only a checksum; and "Random/pitch mod", whose first run
passed before the ENVX/OUTX window rule and no longer completes within 5,000 frames with it, a regression
measured and not explained. Its later random rounds failed before that too.

### 23.2 D-28, D-26 and D-24

- **D-28 is settled by the single-test runs:** fullsnes's interpolation passes "Random/brr before playing",
  "Random/kon pitch" and "Random/pitch mod"'s first run, and anomie's formula fails all three. The rule built at
  §22 stands.
- **D-26 and D-24 stay open, with the reason recorded in their entries:** a purpose-written ROM could separate the
  remaining readings only on a console, and run on Mesen it would measure Mesen. D-24's next recourse is the
  referee's 65C816 interrupt sequence, at a CPU-side step.

### 23.3 The sound, sample by sample

The harness now resamples (linear interpolation, both runs to mono at 32 kHz; Mesen's WAV is 48 kHz) and compares
each second from frame 60 on at that second's best lag within 1,600 samples, silent seconds left out:
Pearson's r, the least-squares gain of Mesen's onto VenusRT's, and the residual's RMS over VenusRT's. A chirp test
pins the resampler, the lag and the gain.

| Game | Median r (least) | Lag, samples | Gain | Residual | Seconds |
|---|---|---|---|---|---|
| A Link to the Past | 0.988 (0.967) | 6 to 20 | 0.985 | 19% | 4 |
| Donkey Kong Country | 0.992 (0.984) | 8 to 18 | 0.993 | 12% | 7 |
| Super Metroid | 0.998 (0.997) | 32 to 36 | 1.002 | 6% | 3 |
| Super Mario World | 0.964 (0.941) | 12 to 22 | 0.967 | 31% | 6 |

**The waveforms agree at the sample level, at the same loudness** (gain within 3.5% of 1) and in phase (every r
positive). The lag drifts by up to 14 samples within a game, and a single lag over the whole run gives r of only
0.71 to 0.97 (measured): the two engines' sound runs at slightly different rates over a run, which Mesen's 48 kHz
resampling and its own audio clock would explain (argued, not measured). The residual is the comparison's
resolution and is not attributed: linear interpolation, Mesen's resampler and the drift within a second all
contribute. A bit-exact sound comparison needs a 32 kHz dump from the probe.

### 23.4 The corpus, the pictures and the skip check (measured 2026-10-02)

The corpus is as at §22: 76 of the 117 ROMs Mesen passes pass, 199 of 295 pictures equal Mesen's at the last frame,
and nothing that passed fails (C# Venus passes 81). The games' pictures and spaces are as at §21. The skip-versus-
draw check holds on all 304 images over 600 frames.

### 23.5 The clone check (measured 2026-10-02)

As at §22.5: the one pair over the threshold and the two shared tables are the DSP's documented Gaussian table and
counter rates; no new identifier.

### 23.6 The cost (measured 2026-10-02)

Under the timing lock (load 2.7 falling to 1.8), §22's binary against this step's, best of three over 600 frames:
every figure within 0.01 ms of §22's (drawn 1.26 to 1.80 ms, skipped 0.50 to 0.74, the sound unit alone 0.18 to
0.24). This step changed rules, not cost.

### 23.7 Stage 4 closed

**Built:** the SPC700 (256,000 of 256,000 single-step cases with cycles), the S-SMP with its ports, timers, boot ROM
and clock domain, and the S-DSP. **Graded:** gilyon's `spctest`, blargg's `spc_smp`, `spc_timer` and
`spc_mem_access_times`, PeterLemon's seven SPC700 ROMs, nine of the 2010 SPC ROMs, 106 of `spc_dsp6`'s 111 tests on
their own, and the four games' sound against Mesen's at r 0.96 to 0.998 sample by sample.

**Predictions retired.** P1 (a desktop mean of at most 2.8 ms a frame on the plain benched games at the end of stage
4): **met** on the four measured, at 1.26 to 1.80 ms drawn with the sound unit complete; Pilotwings, the fifth in
the plan's table, waits for its DSP-1 at stage 5.

**Left open, named:** D-6, D-7, D-11, D-12 and the other PPU exceptions of §20.5; D-24 (CPU side); D-26
(`test_timer_stop2`); and `spc_dsp6`'s five tests of §23.1. None is a game-facing difference this stage measured.

### 23.8 Addendum: the in-order stop and the pitch-mod regression (2026-10-02)

**"Misc/$F0-$FF are not ram" passes (D-31).** VenusRT read AUXIO4 and AUXIO5 from the RAM under the I/O page,
which the DSP's echo writes reach; fullsnes describes output latches read back as written. As latches (power-on
$FF), the test passes alone and in order. The state is version 14. `spc_smp`, `spc_timer`,
`spc_mem_access_times`, `spctest` and the 2010 SPC ROMs still pass (measured).

**`spc_dsp6` now (measured 2026-10-02):** 107 of 111 tests pass on their own. In order, the ROM passes 34 and stops
at the 35th, "Misc/brr addr wrap-around", which passes on its own: run after "$F0-$FF are not ram" (a copy of the ROM
alternating the two) it prints its table with two wrong entries (514E and BE44 where 8008 and 9008 belong) and the
suite ends with "Passed 01", the SPC700 back in the boot ROM's wait loop and the S-CPU in the suite's idle loop at
00:808F. The preceding test leaves the bytes 0A FF 7C 01 FE FF ... at $8000 in VenusRT and in Mesen alike, so the
leftover is not the difference; what is, is not found. Recorded, not settled.

**The "Random/pitch mod" regression is the ENVX half of the window rule** (measured: with only the OUTX half the
test's first run passes; with only the ENVX half it fails as with both). Moving the ENVX window: excluding the cycle
in which S7 prepares the value fails "Timing/Voice/V9 envx", and any window that includes that cycle stops pitch mod
the same way. The stop is not inside the test: its last DSP accesses are the cleanup (FLG $E0, EDL 0), after which
the SPC700 is back in the boot ROM's wait loop and the screen reads "Passed 01", the same end as the in-order stop
above. The test synchronises by writing $88 to ENVX and reading it back until the DSP overwrites it, so the window
moves the point it synchronises to by up to a sample, and every write it times after that moves with it; argued to
be the mechanism, not measured. **Neither is the rule shown wrong:** the window is anomie's sentence, and "V9 envx"
needs it. Kept, with the regression recorded; the two "Passed 01" ends are the next thing to trace, from the SPC700's
last instructions before it re-enters the boot ROM.

---

## 24. Stage 5, step 1: the NEC DSP-n and ST01x (2026-10-02)

### 24.1 What it built

`chips/necdsp.rs`, the NEC µPD77C25 of the DSP-1 to DSP-4 and the µPD96050 of the ST010 and ST011, from fullsnes's
"SNES Cart DSP-n/ST010/ST011" chapters: the 24-bit ALU, LD and JP instructions, both accumulators with their six
flags (S1 and OV1 as fullsnes describes them, including S1 following the sign on the logical operations), the K*L*2
multiplier, the data RAM and ROM pointers with their adjust fields, the stack (four levels, eight on the µPD96050),
the 8- and 16-bit DR handshake with RQM and DRS, and the boards' maps of fullsnes's "SNES I/O Ports" table (LoROM at
30-3F or 20-3F, LoROM of 2 MiB at 60-6F, HiROM at 00-1F:6000-7FFF, the ST01x at 60-67 with its RAM at 68-6F). The
serial port is not wired on the SNES, so its acknowledges read as set. The ST01x map is chosen when the firmware is
the µPD96050's, since a dump of F1 ROC in the library carries chipset 02h, not fullsnes's F6h.

**Clock (D-32):** the referee's rate, 7.60 MHz for the DSP-n and 10 MHz for the ST01x as exact fractions of the
master clock, one instruction per cycle as fullsnes states, caught up at each S-CPU access and at the frame's end.

**Firmware** is create file 2, 8,192 bytes (µPD77C25) or 53,248 (µPD96050), in fullsnes's newer little-endian
layout or the older big-endian one, told apart by the "JRQM $" every image opens with; the interface's exports are
unchanged. The harness asks C# Venus's public `Cartridge.FirmwareRequirements` which image a ROM needs and reads it
whole or as a program and data pair from the corpus's firmware folder. **Mesen reads it as
`<home>/Firmware/<name>.rom`, program and data in one file** (measured: Pilotwings' attract mode parts from the run
without firmware at frame 2400 only with that file), so the probe's runs for DSP games now get it, and their cache
key carries the firmware's hash. One image in the player's `home/Firmware`, `dsp1.rom`, is 8,192 bytes but not the
DSP-1's program (its first opcodes are not "JRQM $", and it differs from the split pair); the harness builds the
DSP-1 from the pair.

The state carries the chip's registers and RAM in a `Coprocessor` group only a DSP cartridge has; a test round-trips
it. **Not built:** the ST01x's battery-backed RAM as a battery file (stage 6's file handling), and the debugger's view.

### 24.2 The games (measured 2026-10-02)

The games table over nine DSP titles from the player's library, every tenth frame to 2,400 with no input, both
engines given the firmware C# Venus names:

| Game (chip) | The DSP used by frame 2,400 (VenusRT with and without it) | Picture at 2,400 | First picture difference |
|---|---|---|---|
| Super Mario Kart (DSP-1) | yes, 99,456 WRAM bytes | equal | 140, 23 of 240 frames differ |
| Pilotwings (DSP-1) | yes, 1,690 | equal | 150, 6 of 240 |
| Top Gear 3000 (DSP-4) | yes, 27,747 | 19 pixels | 70 |
| Michael Andretti's Indy Car Challenge (DSP-1) | yes, 9,468 at 600 | differs | 440; equal again at 600, WRAM within 3 bytes of Mesen's at 100, 300 and 600 |
| Ballz 3D (DSP-1B) | yes, 25,973 | 4,117 pixels | 760 (OAM from 60) |
| Dungeon Master (DSP-2), Lock On (DSP-1), Super Bases Loaded 2 (DSP-1) | no | equal | — |
| F1 ROC (ST010) | no (Mesen's runs with and without the firmware also agree to 600) | differs from frame 40 | not the chip's |

**Where the DSP runs, VenusRT follows Mesen**: Super Mario Kart's and Pilotwings' attract modes end on Mesen's
picture, and Andretti's WRAM stays within three bytes of Mesen's through frame 600. The later partings of Andretti,
Ballz 3D and Top Gear 3000 are not attributed: a DSP result, the clock's rate (argued, D-32) or the S-CPU or PPU are
all candidates. The three DSP-2 and ST01x games, and Lock On and Super Bases Loaded 2, do not reach their chips
without input, so the DSP-2, DSP-3, ST010 and ST011 are built and untested on games.

### 24.3 The clone check (measured 2026-10-02)

25 files: `chips/necdsp.rs` shares at most 2 fingerprints with any Mesen file, and no table. Two names are Mesen's
too: `necdsp`, the chip's name (C# Venus's public `NecDspVariant` spells it so), and `datarom`, fullsnes's "Data
ROM". The rest is as at §22.5.

### 24.4 The cost (measured 2026-10-02)

Under the lock (load 2.4 falling to 1.4), best of three over 2,400 frames, the same binary with and without the
firmware: Pilotwings 1.28 → 2.11 ms a frame drawn, Super Mario Kart 1.44 → 2.45, Top Gear 3000 0.67 → 2.60 (it
stalls without its chip). **The DSP costs about 0.6 to 1.0 ms a frame**: 127,000 instructions, executed whether the
chip waits on RQM or computes. Against §5.5, where Pilotwings costs C# Venus 2.60 ms, VenusRT is inside it. No
tuning was done; skipping the chip's waiting loop is a lever recorded, not taken.

### 24.5 What is left of the NEC DSPs

Games driven with input to the DSP-2, DSP-3 and ST01x titles' chip use; the ST01x RAM as a battery file at stage 6;
the three late partings above.

---

## 25. Stage 5, step 2: the SA-1 (2026-10-02)

### 25.1 What it built

`chips/sa1.rs`, from fullsnes's "SNES Cart SA-1" chapters: a second 65816, the stage 1 CPU unchanged, over its own
bus with its own map (I-RAM at 0000-07FF and 3000-37FF, the I/O at 2200-23FF, the BW-RAM window at 6000-7FFF,
BW-RAM itself and its 2- and 4-bit bitmap view at 60-6F, ROM through the MMC's four 1 MiB regions in LoROM and
HiROM form). Both sides' control and interrupt registers with the S-CPU's NMI and IRQ vectors replaceable and the
SA-1's always replaced; the H/V timer, with HCNT and VCNT read as compare values (fullsnes's argument); BW-RAM and
I-RAM write protection; normal DMA, charged to the SA-1's clock at fullsnes's rates; character conversion 2 (pixels
written to the register file, bitplaned into I-RAM) and 1 (a bitmap in BW-RAM read out as tiles through the S-CPU's
reads of banks 40-4F); the arithmetic unit; the variable-length bit reader. Registers take only their side's writes,
by fullsnes's "Side" column. The SA-1 board is chosen by a chipset of 3xh on map mode 23h, and decodes the whole
cartridge space; the S-CPU sees nothing of it outside the SA-1's map.

**Clocking (P5's slice):** the SA-1 runs at 2 master clocks a bus cycle (10.74 MHz) and 4 on BW-RAM, behind the
S-CPU, caught up **after every S-CPU instruction** and before every S-CPU access to the board, its IRQ line joining
the S-CPU's at the next cycle. The slice is therefore one S-CPU instruction. Contention between the two CPUs for ROM
and BW-RAM, which fullsnes leaves undocumented ("XXX pg 62..66 timings"), is not modelled. The SA-1's state and the
line travel in the `Coprocessor` group; a 128 KiB-board image skip-checks equal (§25.3).

### 25.2 The oracles (measured 2026-10-02)

- **absindx's `SA1RamProtectionTest`: passes all 222 tests**, with three rules from it beyond fullsnes (D-34): BW-RAM
  at the S-CPU's banks 40-4F only, nothing of the board in the LoROM SRAM banks, the SA-1 seeing BW-RAM in 40-5F,
  and the SA-1's reset clearing CIWP. Before them it stopped at tests 155 and 221. Its photograph of a console's
  run shows stack pointers ($01FD, $01FC) that differ from VenusRT's and Mesen's, which are power-on values.
- **`SA1VersionCodeTest`**: "FAILED" on VenusRT and on Mesen alike; $230E's value is VenusRT's choice (D-33).
- **The games** (every tenth frame to 2,400, no input), all five SA-1 titles in the player's library:

| Game | First lit (Mesen / VenusRT) | Picture at 2,400 | First picture difference | Pictures differing |
|---|---|---|---|---|
| Kirby Super Star | 110 / 110 | 10,961 pixels; spaces equal | 110 | 82 of 240 |
| Super Mario RPG | 40 / 40 | 2,686 pixels; VRAM 1,139 bytes | 40 | 179 of 240 |
| PGA European Tour | 50 / 50 | equal | 50 | 14 of 240 |
| PGA Tour 96 | 50 / 50 | equal | 50 | 15 of 240 |
| Power Rangers Zeo: Battle Racers | 80 / 80 | equal | 400 | 4 of 240 |

Every title boots, lights on Mesen's frame and plays its attract sequence. Super Mario RPG's and Kirby Super Star's
frames at 2,400 show the same scene as Mesen's with the actors a little apart in time (Kirby a few pixels along his
path; Mario, the Goomba and the Shy Guy at other points of the same walk), a phase difference rather than a broken
picture: the slice and the absent contention are the first suspects (argued).

- **The corpus**: 77 of the 117 ROMs Mesen passes now pass (76 at §23), `SA1RamProtectionTest` the gain; nothing
  that passed fails.

### 25.3 The skip check, the clone check, the cost (measured 2026-10-02)

**Skip-versus-draw:** 309 images (the 304 and the five SA-1 games) equal over 600 frames, the SA-1's state included.

**Clone check:** 26 files. The first run found `chips/sa1.rs` sharing 18 fingerprints with Mesen's `Sa1.cpp`, over
§4.2's threshold of 12: the byte-at-a-time register writes for fullsnes's registers (CRV, CNV, CIV, SNV, SIV, SDA,
DTC) written as the same masking expression in the same order. **Those fingerprints are convergence forced by the
documented register layout:** the shared lines are, all of them, single-byte writes to fullsnes's registers in
fullsnes's address order (its "SA-1 I/O Map" table lists CRV, CNV, CIV at 2203h-2208h, SNV, SIV at 220Ch-220Fh,
SDA at 2232h-2234h and DTC at 2238h-2239h, each "Lsb"/"Msb"/"Mid"), and the masking expression is the one way to
write a byte of a wider register; no logic beyond the layout is in them, and the module was written from fullsnes
alone (no Mesen file was read). The writes were then put through two small helpers, which drops the pair below 2
fingerprints; that rewrite changes the count, not the provenance, and is recorded as such: the helpers are kept
because they say what the code does, not as evidence. The largest pair left is 4 fingerprints with `Sa1Types.h`,
the fields named after fullsnes's registers. No table, and no new identifier, is Mesen's.

**Cost** (the lock, load 0.8 rising to 1.1, best of three over 2,400 frames):

| Game | Drawn | Picture skipped |
|---|---|---|
| Super Mario RPG | 2.72 ms | 1.64 |
| Kirby Super Star | 2.70 | 1.77 |
| Power Rangers Zeo | 2.59 | 1.50 |
| PGA European Tour | 2.17 | 1.39 |
| PGA Tour 96 | 2.18 | 1.40 |

**P2 (Super Mario RPG at most 3.8 ms at the end of stage 5): met so far**, at 2.72 against C# Venus's 4.16; Yoshi's
Island waits for the GSU. **P5's slice** (one S-CPU instruction) passes absindx's RAM-protection test and keeps Super
Mario RPG inside P2; whether it explains the two games' phase differences is the next measurement. No tuning was done.

### 25.4 What is left of the SA-1

ROM and BW-RAM contention and the SA-1's real access timings (fullsnes's missing pages); character conversion 1 as
the hardware streams it (the two golf games that use it are not in the library); the variable-length reader's
unknowns (Jumpin' Derby, not in the library); $230E (D-33); the two games' phase differences.

---

## 26. Stage 5, step 3: the GSU (2026-10-02)

### 26.1 What it built

`chips/gsu.rs`, the Mario Chip and the Super FX 1 and 2, from fullsnes's "SNES Cart GSU-n" chapters: R0-R15 with
R15 pipelined, so that the byte after a jump or branch runs before its target, as fullsnes's "Jump Notes" describe
(the opcode byte executes while the next is fetched; a write to R15 sends the next fetch to the target); the
ALT1/ALT2/ALT3, TO, WITH and FROM prefixes with the B flag, the branches keeping the prefixes, ALT3 falling back to
ALT1 and both to the base opcode where a variant does not exist; the ALU, shift, byte and multiply opcodes with
fullsnes's flags (MERGE's four masks, CMP as SUB under ALT3); GETB/GETBH/GETBL/GETBS and GETC from the one-byte ROM
buffer, refilled when R14 is written; LDB/LDW/STB/STW, LM/LMS/SM/SMS and SBK with the swapped bytes of an odd word;
PLOT and RPIX on the 128, 160 and 192 heights and OBJ mode, with transparency, dither and POR's nibble and freeze
bits; COLOR, CMODE, RAMB and ROMB; CACHE, LJMP and STOP; the 512-byte code cache with its SNES window at
3100h-32FFh, a line marked loaded when its last byte is written; and the S-CPU's view: the I/O page with GSU2's
mirrors, R0-R15 through the latch, SFR and its IRQ bit cleared by a read, ROM in LoROM and HiROM form and Game Pak
RAM at 70-71 and 6000-7FFF, and the fixed vector values the S-CPU reads from ROM while the GSU owns it. A chipset of
13h-1Ah on a LoROM map selects the board; the RAM's size comes from the extended header's FFBDh, 32 KiB where there
is none (Star Fox); a ROM over 1 MiB gives the GSU-2's version code.

**Clocking:** 10.74 or 21.48 MHz by CLSR, caught up as the SA-1 is (§25.1). Costs follow fullsnes's "CPU Misc",
where it is itself uncertain: 1 cycle for a cached opcode byte, 3 or 5 for an uncached one or a ROM or RAM byte,
twice that for a word, and the multiply table's counts. **Not modelled:** the pixel caches (PLOT writes RAM at once,
which is the same RAM by the time anything can read it, since RPIX and the S-CPU's access both flush or wait), the
RAM write buffer and RON/RAN's wait states.

### 26.2 The oracles (measured 2026-10-02)

- **PeterLemon's `GSUTest`: all 31 pass**, each table of results equal to Mesen's picture (they stood Incomplete).
- **His 27 drawing ROMs** (PlotPixel, PlotLine and FillPoly at 2, 4 and 8 bits and the three heights): every picture
  compared equals Mesen's at frame 600 (three compare no picture in either engine).
- **The games**, every tenth frame to 2,400 with no input:

| Game | First lit (Mesen / VenusRT) | Picture at 2,400 | First picture difference | Pictures differing |
|---|---|---|---|---|
| Super Mario World 2: Yoshi's Island (GSU-2) | 80 / 90 | 6,867 pixels | 80 | 174 of 240 |
| Star Fox (Mario Chip) | 210 / 200 | 19,461 pixels | 200 | 213 of 240 |
| Doom (GSU-2) | 10 / 10 | 106,657 of 114,688 | 260 | 190 of 240 |
| Stunt Race FX (GSU-1) | 80 / 80 | 26,733 | 250 | 99 of 240 |
| Vortex (GSU-1) | 150 / 150 | 22,661 | 150 | 182 of 240 |
| Dirt Trax FX (GSU-1) | never / 220 | — | — | Mesen's run shows no picture |

Every GSU game boots and draws its 3D or scaled scenes as Mesen's pictures show them; looked at side by side, Star
Fox's space attract scene at frame 600 and Doom's first room at 1,500 are the same pictures, and Yoshi's Island's
storybook intro the same pages, **a little apart in time**: the actors stand at other points of the same movement,
and Star Fox lights ten frames early. That phase is what the counts measure. The GSU's cycle costs, fullsnes's
uncertain ones, are the first suspect (argued); they decide how long each frame's drawing takes and so when the game
moves on.

- **The corpus**: 108 of the 117 ROMs Mesen passes now pass (77 at §25), the 31 `GSUTest` ROMs the gain; pictures equal
  to Mesen's at the last frame on 230 of 295 (199); nothing that passed fails. The skip-versus-draw check holds on
  the six GSU games over 600 frames.

### 26.3 The clone check (measured 2026-10-02)

27 files. `chips/gsu.rs` shares fewer than 2 fingerprints with any Mesen file, and no table. Seven names are Mesen's
too, each the plain name of what fullsnes calls it or of a common operation: `rombuffer` ("ROM Read Buffer"),
`ramaddress` ("RAM Address"), `readbyte`, `readword`, `writebyte`, `writeword`, and `gsuram` in `cart.rs`.

### 26.4 The cost (measured 2026-10-02)

The lock, load 1.0 rising to 1.5, best of three over 2,400 frames:

| Game | Drawn | Picture skipped |
|---|---|---|
| Yoshi's Island | 1.95 ms | 0.92 |
| Star Fox | 2.14 | 1.15 |
| Doom | 2.76 | 1.50 |
| Stunt Race FX | 2.04 | 1.01 |
| Vortex | 2.38 | 1.18 |
| Dirt Trax FX | 1.97 | 1.32 |

**P2 (Yoshi's Island at most 3.8 ms at the end of stage 5): met**, at 1.95 against C# Venus's 3.13, with the caveat
that the GSU's cycle costs are approximate and a slower model would cost more host time only as far as the GSU runs
longer between the same frames. No tuning was done.

### 26.5 What is left of the GSU

The pixel caches, the RAM write buffer and RON/RAN waits; the cycle costs fullsnes marks unknown, which the games'
phase measures; and Dirt Trax FX, whose Mesen run shows no picture.

---

## 27. Stage 5, step 4: the OBC1, and stage 5 closed (2026-10-02)

### 27.1 The OBC1

`chips/obc1.rs`, from fullsnes's "SNES Cart OBC1": the board's 8 KiB of SRAM at 6000h-7FFFh in banks 00-3F and 80-BF,
and the eight registers at 7FF0h-7FF7h: 7FF0h-7FF3h read and write the four bytes of OBJ Index (7FF6h) in a
220h-byte workspace based at 7C00h or 7800h (7FF5h bit 0), and 7FF4h writes that OBJ's two bits into the
workspace's high table, read back as the whole byte as fullsnes reports. A LoROM chipset of 2xh selects it (Metal
Combat's is 25h). Not built, as fullsnes leaves them unknown: 7FF7h's meaning and the "Index bits 7+5" SRAM-mapping
report. **Metal Combat** (the only OBC1 game, in the library): lit on Mesen's frame (10), the picture equal to
Mesen's at frame 2,400 with 18 of 240 sampled frames differing; whether its attract mode reaches the chip's registers
was not measured. 2.10 ms a frame drawn.

### 27.2 Stage 5 at its close (measured 2026-10-02)

- **The corpus: 108 of the 117 ROMs Mesen passes pass** (76 at stage 4's close), and **every ROM C# Venus passes,
  VenusRT passes**, 27 more besides (C# Venus passes 81). Pictures equal to Mesen's at the last frame: 230 of 295
  (199). The nine Mesen passes left: `spc_dsp6` (§23.8), `test_timer_stop2` (D-26), `test_irqb` (D-24), and byuu's
  six backdrop-graded IRQ, NMI and HDMA ROMs, all failing since stage 2.
- **Skip-versus-draw over every coprocessor game: 21 of 21 equal over 600 frames** (nine NEC DSP games with their
  firmware, five SA-1, six GSU, the OBC1), with the chips' state in the comparison.
- **P2 (Super Mario RPG and Yoshi's Island at most 3.8 ms a frame at the end of stage 5): met.** Under the lock (load
  2.6 falling to 1.9): Super Mario RPG 2.77 ms drawn (1.67 skipped), Yoshi's Island 1.97 (0.94), against C# Venus's
  4.16 and 3.13; Pilotwings 2.23. P2 is retired as met.
- **The clone check across `chips/`** (the four coprocessor files, 2,800 lines, against Mesen's 164 files): the
  largest pair is 4 fingerprints (`sa1.rs` with `Sa1Types.h`, fullsnes's register names, §25.3), `gsu.rs`'s largest
  3 and `necdsp.rs`'s 2; `obc1.rs` shares none; no table. Over the whole crate (28 files) the one pair over the
  threshold and the two shared tables are still the S-DSP's documented Gaussian table and counter rates (§22.5).

### 27.3 Every coprocessor's open items

| Chip | Open item | Where recorded |
|---|---|---|
| NEC DSP-n, ST01x | The DSP-2, DSP-3, ST010 and ST011 unexercised on games (their titles reach the chip only with input) | §24.2 |
| | The ST01x's battery RAM as a battery file | §24.1, stage 6 |
| | Andretti, Ballz 3D and Top Gear 3000 parting from Mesen late | §24.2 |
| | The instruction rate is the referee's, one instruction a cycle argued (D-32) | D-32 |
| | VenusRT's own firmware lookup, replacing the harness's call into C# Venus | plan §6, stage 6 |
| SA-1 | ROM and BW-RAM contention and the access timings fullsnes lacks | §25.1 |
| | The phase drift in Super Mario RPG and Kirby Super Star | §25.2, §27.4 |
| | Character conversion 1 and the variable-length reader untested on games | §25.4 |
| | $230E's value (D-33) | D-33 |
| GSU | Pixel caches, the RAM write buffer and RON/RAN waits not modelled | §26.1 |
| | Cycle costs fullsnes marks unknown; the phase drift in every GSU game | §26.2, §27.4 |
| | Dirt Trax FX, whose Mesen run shows no picture | §26.2 |
| OBC1 | 7FF7h and the SRAM-mapping bits; whether the attract mode reaches the chip | §27.1 |

### 27.4 The phase drift: a proposal, not started

Super Mario RPG, Kirby Super Star and every GSU game show Mesen's scenes with the actors a little apart in time; Star
Fox lights ten frames early and Yoshi's Island ten late. The plain games and the NEC DSP games do not drift this way
over the same frames (Super Mario Kart's and Pilotwings' attract modes end on Mesen's picture), which points at the
two processors with their own instruction streams rather than at the S-CPU, the PPU or the APU. Candidates, each
with the measurement that would tell it from the others:

1. **The coprocessor's own cycle costs** (the GSU's opcode, cache and memory costs from fullsnes's uncertain table;
   the SA-1's 2 and 4 master clocks per access). A coprocessor too fast finishes each frame's work early and the game
   runs ahead (Star Fox), too slow and it falls behind (Yoshi's Island), so the sign can differ per game.
   *Telling it apart:* read the games' own frame-rate counters from WRAM in both engines (Star Fox and Doom draw below
   60 Hz and count the SNES frames each 3D frame takes; the counter's address is found by diffing WRAM over a steady
   scene). If VenusRT's 3D frames take a different number of SNES frames than Mesen's, the costs are the cause, and
   the counter measures by how much. Then a sweep of the cost model (one parameter at a time) against that counter,
   kept only where PeterLemon's GSU tests and absindx's test still pass.
2. **Contention between the S-CPU and the coprocessor** (unmodelled for both). *Telling it apart:* count, in VenusRT,
   the S-CPU's accesses to the board's ROM and RAM while the coprocessor runs. Near zero for the GSU games (the S-CPU
   waits in WRAM while the GSU owns the bus) would rule contention out there, leaving it a candidate for the SA-1
   only; a large count is the case to model first.
3. **The slice** (the coprocessor caught up after each S-CPU instruction, its IRQ seen up to one instruction late).
   *Telling it apart:* rerun with the catch-up moved to every S-CPU bus cycle (a slower build kept for the test). If
   the drift does not move, the slice is not the cause; P5's bound would then stand as built.
4. **A start-up difference** (the coprocessor's state at power-on or when the S-CPU first starts it, or the boot ROM
   handshake's timing). *Telling it apart:* the first frame at which WRAM differs from Mesen's, from the probe's
   per-frame dumps over the first 300 frames, and which bytes: a drift that is there from the first coprocessor run
   is a start-up or cost cause; one that grows from a later point is an event's.
5. **The referee as a second opinion on the costs:** SNES_MiSTer's GSU and SA-1 (rated mixed and real support in
   `Venus_Referee.md` §0) carry their own cycle counts; read in a logged dispute step for the cost rules alone, after
   (1) has measured how far off VenusRT is, so the reading answers a measured question.

The order proposed: (4) and (2) first, being measurements without code changes; then (1)'s counter, which decides
whether the sweep is worth making; (3) only if the others leave the drift unexplained; (5) when (1) has a number.
