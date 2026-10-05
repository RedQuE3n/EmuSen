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

All 826 SNES files of the tester's library, each copied to scratch, read by header, and removed; nothing in the
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
key carries the firmware's hash. One image in the tester's `home/Firmware`, `dsp1.rom`, is 8,192 bytes but not the
DSP-1's program (its first opcodes are not "JRQM $", and it differs from the split pair); the harness builds the
DSP-1 from the pair.

The state carries the chip's registers and RAM in a `Coprocessor` group only a DSP cartridge has; a test round-trips
it. **Not built:** the ST01x's battery-backed RAM as a battery file (stage 6's file handling), and the debugger's view.

### 24.2 The games (measured 2026-10-02)

The games table over nine DSP titles from the tester's library, every tenth frame to 2,400 with no input, both
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
- **The games** (every tenth frame to 2,400, no input), all five SA-1 titles in the tester's library:

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
- **The clone check across `chips/`** (the four coprocessor files, 2,353 lines with mod.rs, against Mesen's 164 files): the
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

**Measured 2026-10-02: items 4, 2 and 1.** No core code was changed; two examples were added (`chip_activity`,
where the S-CPU runs while the board's processor does; `gsu_jobs`, the GSU's jobs per frame with WRAM per frame).

*Item 4, the first WRAM difference*, from both engines' WRAM after every one of the first 300 frames:

| Game | First byte differing | First difference outside the stack pages, persisting 10 frames | Frames with WRAM equal outside the stack |
|---|---|---|---|
| Super Mario World (plain, the control) | 68 | 77 | 80 of 300 |
| Super Mario Kart (DSP-1) | 85 | 87, growing to 81,786 bytes | 86 |
| Super Mario RPG (SA-1) | 1 | 1 (one byte) | 44 |
| Kirby Super Star (SA-1) | 11 | 11 | 51 |
| Yoshi's Island (GSU-2) | 73 | 279 | 274 |
| Star Fox (Mario Chip) | 1 (four bytes, frame 1 only) | 152 | 136 |
| Doom (GSU-2) | 1 | 14 | 135 |

**The control drifts too:** Super Mario World, with no coprocessor, has a few WRAM bytes apart from frame 77 on (2 at
frame 100, 10 at 300), and Super Mario Kart, whose pictures end on Mesen's, parts wholesale at 87. Small WRAM
differences are therefore not a coprocessor's signature, and a part of every game's drift is S-CPU or PPU side. The
SA-1 games' first bytes (Super Mario RPG's $1D3F from frame 1, Kirby's $95 and $A7 from 11) differ for tens of
frames and are equal again by frame 100: a transient that converges, the shape of a wait-loop count or a seed taken
while the two processors race, not a broken start. **Yoshi's Island's WRAM is equal to Mesen's outside the stack
until frame 279**, although its pictures differ from frame 80: its early picture difference is not GSU-driven.

*Item 2, the S-CPU while the coprocessor runs* (`chip_activity`, 600 frames; Doom 1,800, as it starts its GSU later):
in all six GSU games **no S-CPU instruction was fetched from the cartridge while the GSU ran**: the S-CPU waits in
WRAM (17-47% of its instructions in Vortex, Dirt Trax FX and Star Fox, 2-14% in Stunt Race FX, Yoshi's Island and
Doom). Bus contention therefore cannot be a GSU game's drift (its data accesses were not counted, but code running
in WRAM has only I/O reads to make). In the SA-1 games the S-CPU fetches from the cartridge for **38% (Super Mario
RPG), 84% (Kirby Super Star) and 100% (the two PGA titles, Power Rangers Zeo)** of its instructions while the SA-1 runs:
contention is a live candidate there, and only there.

*Item 1, the games' own frame counters* (WRAM bytes that step by +1 at the 3D frame rate, found in VenusRT's dumps and
read in Mesen's):

- **Star Fox, $15BB** (one count per 3D frame): the first 3D frame completes at SNES frame 147 in VenusRT and 148 in
  Mesen, the first parting of the two counters; later frames land one SNES frame earlier now and then. Over frames
  300-900, **4.38 SNES frames per 3D frame in VenusRT against 4.58 in Mesen**, 137 counts against 131: VenusRT's GSU
  finishes its 3D frames a little sooner, the game running about 4% ahead (the scenes are no longer the same after
  frame 152, so the long-window rate mixes speed with scene; the first-frame lead is the clean measurement).
- **Vortex, $198C**: the counters agree to frame 140; from 141 VenusRT steps one frame early, then Mesen one frame
  early, both ways; over 150-450 both 1.500 SNES frames per count, over 450-900 2.586 against 2.601.
- Doom and Stunt Race FX: no such counter in the windows searched (Doom's GSU runs six jobs every four frames in
  VenusRT; Stunt Race FX's attract runs in menus without the GSU at 700-740).

**Recommendation.** The measurements separate the two chips:

- **GSU: the cycle costs are the candidate; contention is ruled out.** Star Fox's GSU completes its first 3D frame
  one SNES frame earlier than Mesen's, and runs about 4% more 3D frames over 600 frames, so VenusRT's GSU costs are
  slightly low. The next step is the referee's GSU cycle rules read in a logged dispute step (fullsnes marks them
  unknown; `Venus_Referee.md` §0 rates the GSU mixed, its instruction core the author's own), then one cost change at
  a time measured against Star Fox's $15BB (first 3D frame at 148, the gaps' histogram) and Vortex's $198C, kept only
  while PeterLemon's 31 GSU tests and 27 drawing ROMs still pass. Yoshi's Island's early picture difference, with its
  WRAM equal to frame 279, belongs to the PPU side and to a separate look.
- **SA-1: contention first, costs second.** The S-CPU runs from the cartridge during most of the SA-1's time, the
  first differences are transients that converge, and no 3D counter exists to measure speed by. A contention model
  needs the rule (who waits, and for how many cycles, when both reach ROM or BW-RAM in the same cycle), which no
  fetched document gives; the referee's SA-1 (real support on structure) is the source to read in a dispute step,
  then measured against the transients' length (Super Mario RPG's $1D3F, Kirby's $95/$A7) and the pictures.
- **The S-CPU-side drift the control shows** (Super Mario World from frame 77) is smaller and separate; D-6's
  power-on phase is the open item it would start from.

## 28. The coprocessors' costs from the referee (2026-10-03)

§27.4's recommendation carried out: the referee's cost rules read in logged dispute steps (D-35 for the GSU), then one
cost changed at a time, measured against the games' own counters and kept only while the GSU's test ROMs, the drawing
ROMs and the skip-versus-draw check still pass. Mesen's counters are the comparison, not the authority: a change is
kept for its referee basis, and one that only moved a number toward Mesen's would not be.

### 28.1 The GSU, one cost at a time (measured 2026-10-03)

Each row is the build after that change and every change above it. "Star Fox" is $15BB: the SNES frame its first 3D
frame completes at, then over frames 300-900 the count, SNES frames per 3D frame, and the histogram of SNES frames
between counts (gap: occurrences). "Vortex" is $198C over 450-900, its count and SNES frames per count. Mesen's runs:
Star Fox 148, 131 counts at 4.58, gaps 3:31 4:37 5:25 6:30 7:7; Vortex 173 at 2.601. Every row kept passed the 31
GSUTest ROMs and the 27 drawing ROMs (each final picture equal to Mesen's, and the frames differing on the way
unchanged) and skip-versus-draw over the six GSU games for 600 frames.

| Build | D-35 basis | Star Fox first | Star Fox 300-900 | Gaps | Vortex 450-900 | Kept |
|---|---|---|---|---|---|---|
| Stage 5 as closed | fullsnes's guesses | 147 | 137 at 4.38 | 2:2 3:34 4:45 5:24 6:29 7:2 | 174 at 2.586 | - |
| Cache line fill at slow() a byte, the CPU waiting for the line | the line runs 16 bytes at 3/5 cycles each, plus a state to begin and one to end | 147 | 126 at 4.76 | 2:1 3:28 4:28 5:26 6:29 7:13 | 174 at 2.586 | yes |
| FMULT and LMULT hold the CPU 5 cycles at MS0=0, 1 at MS0=1 (were 7 and 3, LMULT one more) | the multiplier's start state plus a count of 4 or 0; MULT's 1 cycle at MS0=0 already matched | 147 | 127 at 4.72 | 2:1 3:29 4:28 5:28 6:26 7:14 | 174 at 2.586 | yes |
| The ROM buffer loads beside the instruction stream: GETB, GETBH, GETBL, GETBS and GETC wait only for a load still running (were slow() each) | the load ends ROM_CYCLES + 4 cycles after the opcode writing R14, and the CPU waits only on a pending load; the GSU's state gains the load's end, state version 15 | 147 | 129 at 4.65 | 2:1 3:30 4:31 5:26 6:30 7:10 | 174 at 2.586 | yes |
| FMULT and LMULT run three microcode cycles before the hold: 8 cycles at MS0=0, 4 at MS0=1, for both (correcting the multiply row, which read the hold alone; against stage 5 only LMULT's extra cycle is gone) | the microcode table's cycle count for microcode 24, which FMULT and LMULT share | 147 | 127 at 4.72 | 2:1 3:29 4:28 5:27 6:27 7:14 | 174 at 2.586 | yes |
| Loads cost their second microcode cycle, the RAM port's start state and slow() - 1 cycles a byte: LDB one cycle more, LDW, LM and LMS unchanged | the microcode table's 2-cycle LDB and LDW, and the RAM port counting RAM_CYCLES + 1 a byte from a start state | 147 | 127 at 4.72 | 2:2 3:27 4:28 5:28 6:28 7:13 | 174 at 2.586 | yes |
| Stores are posted to the RAM port: STB, STW, SBK, SM and SMS cost their second microcode cycle and the CPU carries on, while the port runs the store for a start state and slow() - 1 cycles a byte; a later RAM opcode, and RAMB, waits for it (were slow() a byte, SM and SMS 2 slow()) | RAM_SAVE_PEND posted at the store's last cycle, and RAMWAIT held while a save is pending or running; the GSU's state gains the port's end | 147 | 129 at 4.65 | 2:1 3:31 4:30 5:25 6:33 7:8 | 188 at 2.394 | yes |
| PLOT's pixel cache flushed on the RAM port: when the plot leaves the primary row's 8 pixels or fills them, the port writes the row out (slow() an access, a read and a write a bitplane unless all 8 were plotted) and the CPU waits only while an earlier flush still runs (PLOT was 1 cycle with no flush) | the flush trigger (`PC0_OFFS_HIT`, `PC0_FULL`), `RAM_PCF_WAIT` while a flush executes, and the PCF state's accesses; the GSU's state gains the row's position, its plotted pixels and the flush's end | 147 | 129 at 4.65 | 2:1 3:31 4:29 5:27 6:32 7:8 | 188 at 2.394 | yes |
| RPIX waits for a running flush or store, flushes the primary row and reads a byte a bitplane, 2 + 3 x bpp x slow() cycles past the wait (was 20) | the RPIX microcode's 2 cycles, `RAM_PCF_WAIT` held to the RPIX state's end | 147 | 129 at 4.65 | 2:1 3:29 4:32 5:26 6:32 7:8 | 188 at 2.394 | yes |

What the referee changed, and what it did not:

- **Star Fox's long-window rate moved to within 2% of Mesen's** (129 counts at 4.65 against 131 at 4.58, from 137 at
  4.38), with the gap histogram's 2-frame gaps nearly gone and its 4- to 6-frame gaps near Mesen's. The cache fill
  carried most of it, overshooting to 4.76; the ROM buffer and the posted stores took back part of the overshoot.
- **Star Fox's first 3D frame stayed at 147** through every change, one SNES frame ahead of Mesen's 148. No GSU cost
  moves it, so the lead is set before or outside the GSU's work: the S-CPU's side (D-6's power-on phase) or the start
  of the GSU's first job, not the GSU's speed.
- **Vortex moved away from Mesen** with the posted stores (188 counts at 2.394 against 173 at 2.601, from 174 at
  2.586): its GSU code stores heavily, and posting the stores beside the instruction stream is the referee's rule,
  so the change was kept on its basis. Vortex's first counter difference stayed at frame 141.
- The overlaps D-35 lists as not modelled are each a cycle or a few per occurrence; none was built, since none has a
  measurement asking for it.

Each kept change passed the GSUTest ROMs and the drawing ROMs with every picture as before, skip-versus-draw over
the six GSU games, and the crate's unit tests. The cost after all of them, measured 2026-10-03 with `frame_cost`
under the timing lock, best of three over 1,200 frames, load average 2.0: Yoshi's Island 1.76 ms a frame, Star Fox
2.14 ms. The clone check over `chips/` against Mesen's `Core/SNES` with fullsnes's vocabulary: largest pair 4 shared
fingerprints (`sa1.rs` against `Sa1Types.h`, unchanged from §27.2), `gsu.rs` at most 1.

Next, recorded and not started: the S-CPU-side drift from D-6's power-on phase, which Super Mario World shows from
frame 77 and which may hold Star Fox's one-frame lead; and Yoshi's Island's picture difference from frame 80 with its
WRAM equal to Mesen's until 279, a PPU-side look.

### 28.2 The SA-1's contention with the S-CPU (measured 2026-10-03)

D-36's rule as built: every S-CPU cycle, its memory cycles, internal cycles and DMA's A-bus cycles alike, holds the
SA-1 off the memory its address selects (ROM, BW-RAM or I-RAM, by the referee's decode of the address alone) from 3
master clocks into the cycle (4 for DMA) to its end. An SA-1 access to that memory waits for the hold's end, a BW-RAM
access needing its two cycles clear; an access to another memory, or an internal cycle, goes on. Because the SA-1 is
caught up by whole instructions, it can run a few clocks past the S-CPU's clock before the S-CPU's next cycles are
known; its accesses past the catch-up's target are kept (16 at most, in the state), and a hold arriving over one of
them moves that access, and the SA-1's clock with it, past the hold. The S-CPU never waits. State version 16.

How much the SA-1 is held, counted over the first 300 frames by a probe not kept: Super Mario RPG's SA-1 loses 5.0%
of the master clocks to holds, Kirby Super Star's 42.6%. Against the measures D-36 named:

| Game | Measure | Before | With contention | Mesen |
|---|---|---|---|---|
| Super Mario RPG | $1D3F differing, frames 1-300 | 14 frames, first 1, last 22 | the same | - |
| Super Mario RPG | WRAM outside the stack equal | 44 of 300 | 44 of 300 | - |
| Kirby Super Star | $95 and $A7 differing | 81 frames, first 11, last 122 | the same | - |
| Kirby Super Star | WRAM outside the stack equal | 51 of 300 | 51 of 300 | - |
| The five SA-1 games | pictures differing from Mesen's, every tenth frame to 600 | 4, 10, 8, 4, 18 of 60 | the same | - |

**The transients and the pictures do not move**, although Kirby Super Star's SA-1 spends two fifths of its time held.
The SA-1's speed is therefore not what sets them. The likely reading, not measured: both games' SA-1 waits on the
S-CPU for much of each frame, so holding it shortens its idle loops rather than delaying the work the S-CPU waits for. Super Mario RPG's $1D3F differs
from frame 1, in the frame the SA-1 is first released, which points at the start-up or the S-CPU's side (D-6) rather
than at the board. The change was kept on its referee basis: absindx's RAM-protection test still shows all 222 tests
passed; the version-code test, a table of what each register reads, shows the same result with the SA-1's H-counter
latch reading other values, as a timing change should; skip-versus-draw holds over the five SA-1 games for 600
frames; the crate's tests, with one for the holds, and the WiseMan VenusRT tests pass. The cost, best of three over
1,200 frames under the timing lock, run before and after in turn at load average 8: Super Mario RPG 2.78-2.83 ms a
frame before and 2.78-2.80 after, Kirby Super Star 2.56 and 2.57.

Not modelled: the SA-1's own DMA and character conversion against the S-CPU (they wait on the SA-1's priority bits,
which the referee reading did not cover), HDMA's table reads from the board, and the SA-1's clock phase restarting at
each S-CPU cycle (the cycles are even lengths, so the phase drifts at most a clock).

## 29. D-6 and the start-up drift, measured (2026-10-03)

§27.4 left a drift that every game shows: the plain control parts from Mesen from frame 77, Super Mario RPG's $1D3F
differs from frame 1, Star Fox's first 3D frame lands one early. Measured instead of WRAM's first byte, which shows
only the effect: the probe's CPU trace of Mesen against VenusRT's own (`cpu_trace`, which now takes a power-on
offset in master clocks as D-6's `power_phase` does), instruction by instruction from power-on, to where the two
first part.

| Game | First parting | Master clocks in | What the instruction does | Cause |
|---|---|---|---|---|
| Star Fox | instruction 149 | 3,990 | `LDA $213D` after `LDA $2137`: the V counter reads 2, Mesen 3 | the CPU's start against the PPU's counters (D-6) |
| Super Mario World | 2,039 | 51,498 | `CMP $2140` / `BNE` on the SPC700's boot ROM: one poll later than Mesen | the S-CPU's and SPC700's port timing |
| Super Mario RPG | 6,944 | 262,438 (frame 1) | the same wait on `$2140` | the same |
| Super Mario Kart | 8,729 | 1,297,542 | a `CMP` / `BNE` wait loop | the same kind |

- **Star Fox's trace equals Mesen's for its first ten frames (115,103 instructions) with the CPU's start moved 132
  master clocks into line 0**, and at no other offset tried. That is D-6's 132 a third time.
- **The SPC700's speed is not the cause** of the handshake partings: Super Mario World's two uploads take 525.04 and
  587.81 clocks a byte in both engines, equal to 0.003% and exactly.
- **VenusRT's port timing is granular**: before each port access the SPC700 is run a whole instruction past the
  S-CPU's clock, so its port writes in that instruction are seen early and the S-CPU's writes arrive after SPC700
  reads that already happened. An experiment (not kept) timestamped both directions to the SPC700 cycle. It moved
  the first parting to instruction 1,994, and no SPC700 lead from -60 to 180 clocks, with the CPU's start at 0 or
  132, carried Super Mario World past instruction 2,062. Mesen's port timing differs from both of VenusRT's in a
  way a phase does not absorb.

The referee was read for D-6's rule as logged: the SPC700 leaves reset with the reset line and the 65C816 is held
150 master clocks more, while the S-PPU's counters do not reset at all, so the referee has no rule for the phase
against the counters. D-6 stays open and nothing was changed: the counters' half has only Mesen behind it, and the
SPC700's half cannot be measured until the ports are right.

**Proposed next, not started:** D-37, the S-CPU's and SPC700's port timing: when an SPC700 write to $F4-$F7 becomes
visible to an S-CPU read of $2140-$2143, and the converse, at the bus cycle. Documents first (fullsnes's and anomie's
APU port notes), then the referee's `SMP.vhd` port latches in a logged step. A cycle-timed port in VenusRT needs the
SPC700's writes timestamped (cheap) and its reads of the S-CPU's writes made at the right cycle, which the
instruction-stepped SPC700 cannot do while it runs ahead of the S-CPU. Measured against the four traces above, which
show the effect within the first frame. D-6's 150-clock SPC700 lead is judged after it. Yoshi's Island's PPU-side
look stays recorded as next.

## 30. The S-CPU's and SPC700's ports by the cycle, and the SPC700's start (2026-10-03)

### 30.1 What was built (D-37, D-6)

- **The ports' latches as the referee places them.** Each side's writes wait in a queue stamped in quarters of a DSP
  cycle (1.024 MHz). The S-CPU's write lands at the end of its bus cycle. The SPC700 latches $F4-$F7 in the first
  quarter of its read cycle and its write lands in the fourth, as does CONTROL's clearing of the inputs. The S-CPU
  reads at its cycle's end what had landed by then. The queues travel in the state (state version 17).
- **The SPC700 kept behind the S-CPU.** It runs an instruction only when every cycle of it starts before the
  S-CPU's clock, so no S-CPU write still to come is one it should have seen. The instruction it stands at, which
  would cross the S-CPU's clock, is run on a copy that changes nothing (`Smp::peek`) when the S-CPU reads a port, for
  the port writes it makes before the read. The peek is cached while the SPC700 and the input queue stand still.
- **The SPC700's start (D-6, the referee's half).** The SPC700 leaves reset 150 master clocks before the 65C816 and
  runs its 8-cycle reset sequence (its BRK with the writes held off) before fetching at $FFC0, where before it
  fetched in its first cycle with the 65C816. The 65C816's start against the PPU's counters is unchanged.

### 30.2 The keep checks (measured 2026-10-03)

| Check | Before | Ports timed | With the SPC700's start |
|---|---|---|---|
| SPC700 single-step suite (`cargo test spc700`, the corpus set) | passes | passes | passes |
| Corpus, VenusRT's verdicts | 108 of Mesen's 117 passed, 172 visual | the same, verdict for verdict | the same |
| spc_dsp6, its 111 tests singly | 105 pass, 6 hang | the same | 106 pass, 5 hang: `4c000_Random_pitch_mod` (§23.8's regression) passes |
| Skip-versus-draw, 25 games for 600 frames | - | all same | all same |
| WiseMan VenusRT tests, the crate's tests | pass | pass (one new test of the ports' timing) | pass |

blargg's three SPC700 ROMs, spctest and the 2010 ROMs keep their verdicts. The 2010 ROMs and the two speed
measurements print timing numbers without a verdict; against Mesen's prints, VRAM bytes differing, before / ports /
start: `test_speed` 4 / 1 / 1, `test_timer_speed` and `_speed2` 7 / 4 / 6, `test_timer_speed3` 5 / 3 / 8,
`test_timer_speed_2` 2 / 3 / 2, lidnariq's `smpspeed` 9 / 3 / 9, undisbeliever's `ipl-speed-test` 6 / 17 / 21. The
numbers move by one count here and there, toward Mesen's and away; none has a console's value to be graded by.

### 30.3 The four traces against Mesen's

| Game | Before | Ports timed | With the SPC700's start |
|---|---|---|---|
| Star Fox | 149 | 149 | 149 |
| Super Mario World | 2,039 | 1,994 | 1,994 |
| Super Mario RPG | 6,944 | 6,920 | 6,920 |
| Super Mario Kart | 8,729 | 8,560 | 8,560 |

The exact ports agree with Mesen's less, not more: each handshake game now parts a few polls earlier. Star Fox's
parting is the PPU phase, D-6's other half, which neither change touches. A scan of SPC700 leads from -40 to 200
master clocks on the timed ports found Super Mario World's trace following Mesen's furthest, to instruction 6,952
with the CPU's start at 132, at a lead near 82; that is fitting Mesen, not a rule, and nothing was built from it.

### 30.4 The cost

`frame_cost`, best of three over 1,200 frames under the timing lock, before and after in turn: Super Mario World 2.09
against 2.10 ms a frame at load 0.4 (ports timed, after the peek's allocation was removed and its result cached), and
at load 6.7 with the SPC700's start, Super Mario World 2.19 against 2.24, Donkey Kong Country 2.14 against 2.18-2.21,
Super Mario Kart 1.60 against 1.65, Kirby Super Star 2.69 against 2.71-2.72: about 2-3% where the S-CPU polls the
ports often, under 1% where it does not.

### 30.5 What is left

- D-6's other half, the 65C816's start against the PPU's counters: open, Mesen's 132 the only number (§29).
- fullsnes's OR of a simultaneous SPC700 write and S-CPU read, not modelled.
- Yoshi's Island's PPU-side look, recorded as next.

## 31. VenusRT on the core ABI v1 (2026-10-03)

### 31.1 What was built

`src/v1.rs` implements `emusen_native::core::Core` for the machine and invokes `core_exports!(Machine;)`, as
`EmuSen_CoreAPI.md` §18.6 lays out; `ffi.rs`'s `native_exports!` stays beside it while the shim and WiseMan's SNES
fixtures still load the library through `emusen_native_*`. Where the two traits share a method, the v1 one forwards to
the pre-stable one by naming the trait (`NativeCore::advance(self, ...)`), so one body serves both.

- **No optional group is claimed yet** (capabilities 0): the pre-stable library had none either. Reset, snapshots,
  the battery's change tracking, ROM patches and cheat pokes are stage 6's.
- **`info()`**: the id `venusrt`, the SNES with the system pack's extensions (`.smc`, `.sfc`), both regions, the
  `snes.pad` controller on ports 0 and 1 with its twelve buttons at `PadButton`'s bits, and two firmware files: the
  SPC700's 64-byte boot ROM as file 1, required; a NEC DSP cartridge's firmware as file 2, its size left open, since
  the header does not say which of DSP-1 to DSP-4, ST010 or ST011 the game needs (`VenusRT_Plan.md` stage 6's title
  lookup is what will name it). `firmware_for` asks for file 2 only where the header names a NEC DSP.
- **`machine_info()`**: the region from the header; the frame rate exactly, NTSC 236,250,000/11 Hz over 357,366
  master clocks a frame and PAL 21,281,370 Hz over 425,568; 256x224 to 512x448 at 4:3; 32 kHz; the eight spaces in
  the shim's order, `CpuBus` (0) and `WRAM` (2) marked for cheats, as the SNES pack's Action Replay codec requires,
  and `IO` read-only; the battery as file 0, `.srm`; the state as `VNRT`, version 17; rendering skipped state-neutral,
  which skip-versus-draw proves.
- **`settings_schema()`** is empty: VenusRT has no settings yet, and create receives none.
- **`status_text()`** names VenusRT's two codes of the core band: -9, an image shorter than one 32 KiB bank, and -10,
  file 1 not given.
- **CpuBus writes** (`System::poke`): a write from outside the machine lands where the documented bus would take it in
  memory, WRAM through its mirrors, the cartridge's SRAM, the SA-1's I-RAM and BW-RAM under their write protection,
  the GSU's RAM when the GSU does not own it, the OBC1's SRAM below its registers, and is dropped at registers,
  the APU's ports, a coprocessor's I/O and ROM. Nothing else is touched: no register, the MDR or a coprocessor's
  clock. The test `cpu_bus_pokes_land_in_memory_and_nowhere_else` holds a poke's landing and the state's bytes
  unchanged by pokes at $2100, $2140, $4200, $420B, $4300, $8000 and $80FFFF.

### 31.2 Measured 2026-10-03

- **The exports**: 32 `emusen_core_` symbols beside the 24 `emusen_native_` ones; `export_check.py` reports ABI 1.0,
  capabilities 0, the exports as the baseline requires.
- **The conformance kit** (`emusen-core-conform`, built from the branch carrying C4's decided rule, that a malformed
  image need only not harm the core), on eleven images with the boot ROM as file 1: Super Mario World, Super Mario
  RPG, Yoshi's Island, Star Fox, Super Mario Kart (with the DSP-1B's firmware as file 2), A Link to the Past, the
  240p test suite, blargg's `spc_smp` and `spc_timer`, PeterLemon's `GSUADD` and absindx's `SA1RamProtectionTest`:
  **compliant on every one**. C4's empty and garbage images are refused with -9 and its words; the half-length image
  is accepted and runs 300 frames, half of a cartridge image being often a well-formed smaller one.
- **The sidecar**, `libvenusrt.so.core.json`, written by `--sidecar` beside the built library (a build artefact, not
  committed: it carries the library's SHA-256).
- **The machine is unchanged by the switch**: the state's FNV-1a hash every 60 frames to frame 600, by the new example
  `state_hash`, is identical before and after for Super Mario World, Super Mario RPG, Star Fox and Super Mario Kart.
- **Every oracle as before**: the corpus with VenusRT as its third engine gives every VenusRT column of all 295 ROMs
  identical to the run before the switch; the crate's 68 tests and WiseMan's VenusRT, core-ABI, adapter, discovery
  and SNES-pack tests pass.

### 31.3 Not done here

VenusRT is not selectable in Mistress (stage 6). The optional groups, the DSP firmware's name, and the pre-stable
exports' removal wait for the adapter to be VenusRT's only loader.

## 32. Stage 6, step 1: the optional groups and the firmware lookup (2026-10-03)

### 32.1 What was built

VenusRT claims `RESET`, `SNAPSHOT`, `BATTERY_DIRTY`, `ROM_PATCHES` and `CHEAT_POKES` (capabilities 0x4C05;
`core_exports!(Machine; reset, rom_patches, cheat_pokes)`, the other two having no exports).

- **Reset** (`Machine::reset`): the console's reset button. /RESET reaches the S-CPU, the PPUs, the S-SMP (fullsnes:
  "The SPC and DSP chips are started via same /RESET") and the cartridge's chips, which start again as at power-on,
  the counters at H=0, V=0 and D-6's SPC700 lead included. The memories keep what they held (WRAM, VRAM, CGRAM, OAM,
  the APU's RAM, the cartridge's RAM, the SA-1's I-RAM), a NEC DSP its firmware, and the frame count carries on.
- **The snapshot** is the full state under kind 1, which `state_load` reads for either kind; rewind takes it.
- **The battery's changes**: the battery RAM is compared at each frame's end with the copy the host last saved, and
  the first difference raises `BATTERY` and sets the changed flag until `battery_saved`. The copy is not in the state.
- **ROM patches** (`System::patched`): a patch answers a read the cartridge serves, with or without its compare value;
  WRAM and the I/O page are the console's, which a Game Genie between the console and the cartridge cannot reach.
  They apply to the S-CPU's bus, DMA included, and not to the SA-1's or GSU's own reads of the ROM, which the device
  cannot see. Machine info's patch range is the whole 24-bit bus. With no patch the read path is one branch more.
- **Cheat pokes**: applied at each frame's end, in CpuBus through `System::poke` and in WRAM, a compare value holding
  a poke until the byte matches it; another space is refused.
- **The firmware lookup** (`Cartridge::nec_firmware`), from the header's chipset and title against fullsnes's list of
  the 23 games: ST010 for F1 ROC II / Exhaust Heat II, ST011 for the other ST01x game; DSP-2 (Dungeon Master), DSP-3
  (SD Gundam GX), DSP-4 (Top Gear 3000); DSP-1 for Pilotwings, which fullsnes names as the DSP-1 game with its
  visible glitch, and DSP-1B, the corrected revision, for every other DSP-1 game. `firmware_for` names the file
  (`dsp1b.rom` and so on, 8,192 or 53,248 bytes) and its split form, the program and data as files 2 and 3, which
  `create` now accepts. Compared with C# Venus's public `FirmwareRequirements` on the nine DSP games of the bench
  folder, the names agree for every one. SD Gundam GX's title was not available to test; a Japanese title the match
  misses gets DSP-1B.
- **The WiseMan harness** asks VenusRT's library for the firmware (`CoreLibrary.FirmwareFor`) instead of C# Venus,
  for its own runs and for the Mesen runs it prepares. Mesen's cached runs were found under the same keys, so the
  firmware bytes are the ones it had before.

### 32.2 Measured 2026-10-03

- The crate's 75 tests, eight of them new (reset, battery tracking, ROM patches, cheat pokes, the snapshot, the
  firmware names, the DSP firmware in two parts, CpuBus pokes); WiseMan's VenusRt, Snes, CoreAbi, CoreAdapter,
  CoreDiscovery and NativeHost tests, 93 of 93.
- `export_check.py`: capabilities 0x4C05, 35 `emusen_core_` symbols, as the baseline requires.
- The corpus with VenusRT as its third engine: every VenusRT column of all 295 ROMs identical to §31's run.
- The conformance kit (C4's decided rule) on the same eleven images as §31.2: compliant on every one.
- The state's hash every 60 frames to frame 600: identical to before the switch to v1 for Super Mario World, Super
  Mario RPG, Star Fox and Super Mario Kart.
- The cost of the patch check with no patch, `frame_cost` before and after in turn under the timing lock at load 5 to
  11: Super Mario World 2.20-2.25 against 2.18-2.24 ms a frame, Donkey Kong Country 2.10-2.15 against 2.10-2.14,
  within the noise.

## 33. Stage 6, step 2: VenusRT in the SNES's engine row through discovery (2026-10-03)

### 33.1 What was done

- **The sidecar.** `RustCores.props` marks VenusRT `CoreAbi 1`, so `EmuSen.csproj`'s `WriteCoreSidecars` target, the
  one MoonRT's move added (`EmuSen_CoreAPI.md` §22), runs the kit's runner with `--sidecar` and copies
  `libvenusrt.so.core.json` beside the library in every build that carries it; no second mechanism was made.
- **No code names VenusRT.** It has no hand-registered engine (`CoreCatalog.IsRegisteredEngine` is false for
  `VenusRT (Rust)`), so discovery appends it to the SNES row, which it creates with Venus (C#) the default, and the
  factory's generic branch opens it as a `CoreEngine`. Where the library is missing, the row is not there and a stored
  choice of VenusRT runs Venus (C#); where the library is not the one its sidecar names, it is never loaded, and the
  engine notice says why.
- **An ST010 or ST011's battery is its on-chip RAM** (fullsnes: "680000h-6FFFFFh ST010/ST011 On-chip Battery-backed
  RAM"), 4,096 bytes as the S-CPU reads them, low byte of each word first: read from file 0 at create, reported by
  the battery export with its changes tracked, shown as the SRAM space, and reached by a CpuBus poke at $68:0000. Found
  by the round trip below, where F1 ROC II's save came out empty.
- **`firmware_for` scores the headers before it loads anything**: an image with no NEC DSP chipset among its header
  candidates is answered without building the cartridge, which had cost a copy of the ROM and D-4's reset evidence on
  every call (the kit's C13 counted 264 calls on Donkey Kong Country in its window against about 225,000 on MoonRT).
  An image with one is loaded whole as before, so the answer is the one the machine's own header choice gives.
- **Tests whose premise was a single SNES engine** now read the row: `CoreDiscoveryTests` points discovery at an empty
  folder before asserting there is no row; `MarsRtEngineTests`, `MarsRtFrontendTests`, `GameEngineAndPlayTimeTests`
  and `ThemedGameOptionsTests` expect the SNES's engines and its game editor's engine choice as discovery gives them.

### 33.2 The checks, measured 2026-10-03

| Check | Result |
|---|---|
| The engine row and the factory (`VenusRtCoreAbiTests`) | the build's sidecar lists VenusRT, capabilities 0x4C05; the row is Venus (C#), VenusRT (Rust), Venus the default; the factory gives `VenusCore` by default and `CoreEngine` when VenusRT is asked for, with no notice |
| The fallback | with no library, no row and Venus (C#) running; with a library one byte longer than its sidecar's, never loaded, Venus (C#) running and the notice naming the SHA-256 |
| The adapter against the shim, a synthetic game | 240 frames with the pad pressed in turn: the same picture and samples every frame and a byte-identical state; a WRAM cheat lands |
| Commercial games through `ICore` on the adapter (opt-in, `EMUSEN_VENUSRT_GAMES`) | 13 games, 600 frames each, against the shim with the same firmware: the picture and the samples equal every frame and the states byte-identical at the end, for A Link to the Past, Chrono Trigger, Donkey Kong Country, F1 ROC II (ST010), Kirby Super Star (SA-1), Metal Combat (OBC1), Pilotwings (DSP-1), Star Fox (GSU), Super Mario Kart (DSP-1B), Super Mario RPG (SA-1), Super Mario World, Super Metroid and Yoshi's Island (GSU-2) |
| `.srm` round trips with C# Venus (opt-in, `EMUSEN_VENUSRT_BATTERY_GAMES`) | ten games, one each of LoROM, HiROM, DSP-1B, DSP-2, ST010, SA-1 (two), GSU (two) and OBC1: the length VenusRT keeps equals the file Venus (C#) writes; a file Venus (C#) wrote is read whole by VenusRT; a file VenusRT wrote after a change is read whole by Venus (C#) |
| Mistress (`VenusRtEngineTests`) | the SNES tab's engine row offers VenusRT under Venus (C#) and stores the choice; a game runs on Venus until VenusRT is chosen, then on the adapter, with rewind filling from the snapshot |
| The fit audit | 92 of 92, the SNES tab's engine row at 1280x800 and 1920x1200 among them; `WindowFitScrapeAuditTests` failed once in two runs on the scrape window, which this change does not touch, and passed alone and in the full run |
| The kit (`emusen-core-conform`, the build's runner) | compliant on 13 images, Metal Combat and F1 ROC II (with its ST010 firmware) added to §31's eleven |
| The state's hash every 60 frames to frame 600 | identical to before the v1 switch for the four bench games |
| The full WiseMan suite | 9,088 passed, 53 skipped and two failed, both a single-SNES-engine premise, since corrected; the affected tests then 151 of 151 |

`dotnet publish` of Mistress alone stopped on NETSDK1152 (DianaOS's apphost and runtime files found twice), before
the sidecar step, so the publish half was not seen end to end here; the target copies the sidecar into the publish
folder by the same item as the library.

## 34. The adapter's split firmware, and the prompt asking the engine that runs (2026-10-03)

§33 left the generic adapter loading a firmware file whole by its name only, so the program and data pair VenusRT's
`firmware_for` also names was never tried. Built, in `EmuSen/Cores/Native/CoreEngine.cs` and `EmuSen/Common/Firmware/`:

- **`FirmwareRequest.Parts`** carries `firmware_for`'s split forms; **`FirmwareLibrary.TryLoadParts`** returns the first
  form whose files are all present and together the whole's size, part by part, and `IsInstalled` holds a request met
  either way.
- **`CoreEngine.LoadRom`** takes each file whole by its name and size, else its first split form found, part n as
  file `which` + n (`EmuSen_CoreAPI.md` §6.2). For VenusRT that is `dsp1b.program.rom` and `dsp1b.data.rom` as files 2
  and 3, which §32's `create` accepts.
- **The missing-firmware prompt asks the engine that will run the game.** `EmulatorSession.MissingFirmwareFor` and
  `CoreFactory.ForFirmwareProbe` take the engine, and Mistress passes the one it is about to start (the game's own
  choice, else the graphics window's row). Before, the reference engine was always asked, so a game about to run on
  VenusRT was never asked for the SPC700's boot ROM, which Venus (C#) does not need and VenusRT refuses to start
  without.

Tested with synthetic bytes, no dump (`VenusRtCoreAbiTests.The_adapter_takes_a_dsp_firmware_whole_or_as_its_split_pair`):
a DSP-1 cartridge of `SyntheticRom` asks VenusRT for `spc700.rom` and `dsp1.rom` and Venus (C#) for no boot ROM; with
the pair in the firmware folder nothing is missing and the DSP is fitted (the state grows by its group); a pair of the
wrong total size is not taken; the whole file gives the same machine. WiseMan's firmware, factory, adapter, discovery,
engine and Mistress window tests: 306 of 306.

## 35. VenusRT's own SPC700 boot program, and no firmware for an ordinary game (2026-10-04)

*Decided 2026-10-03* (`EmuSen_Firmware.md` §0): VenusRT neither requires nor uses a firmware file for the SPC700's boot
program. §21.2 had made the console's 64 bytes file 1 of `create`, required, with status -10 when absent. That is
withdrawn here. The NEC DSP firmware (files 2 and 3) is unchanged.

### 35.1 What was built

- **The program.** `src/apu/boot.rs` holds 64 bytes written from fullsnes's prose of the transfer protocol, with their
  provenance, the sections read and those deliberately not read, in `VenusRT_Disputes.md` D-38. The S-SMP maps them at
  $FFC0-$FFFF while CONTROL bit 7 is set, and the SPC700 always runs: the `Option` that let it stand still without a
  boot ROM is gone.
- **The interface.** `create` takes no file 1 and refuses one as `BAD_FILE`, since the number no longer names
  anything (*superseded 2026-10-04 by §38.4*: file 1 is again the player's optional image). Status -10 and its words
  are removed from the core, `status_text` and the shim. `info()` lists only the
  NEC DSP's file, and `firmware_for` answers an empty list for a cartridge without a NEC DSP. The core info's
  description states the accuracy cost below.
- **No dependence on the console's image.** The examples' loader, the WiseMan SNES harness
  (`VenusRtSnesEngine`), the adapter and Mistress tests and the kit runs no longer read `spc700.rom`, and the
  synthetic stand-ins (`idle_ipl`, `IdleIpl`, `Machine::with_ipl`, `EMUSEN_VENUSRT_IPL`) are deleted. The D-37 port
  test runs its program from RAM instead.
- **Mistress asks for nothing for an ordinary game.** `VenusRtEngineTests.An_ordinary_game_on_venusrt_prompts_for_no_firmware`
  chooses VenusRT with an empty firmware folder and runs Mistress's own prompt: an ordinary game changes nothing, and a
  DSP-1 cartridge of `SyntheticRom` still asks for its chip. `The_adapter_takes_a_dsp_firmware_whole_or_as_its_split_pair`
  now finds only `dsp1.rom` missing for that cartridge.
- **The console's image as a reference only.** The copy at `~/.cache/emusen/probe/venusrt/firmware/spc700.rom` is kept
  outside the repository, never committed, as the comparison `firmwarecheck.py` (`EmuSen_Debugging_Tools_Reference_v5.md`
  §3.61) runs against. Nothing else reads it: not the core, its tests, the harness, the examples or the kit.

### 35.2 The similarity (measured 2026-10-04)

`firmwarecheck.py`, against the cached image: 6 of 64 bytes equal at the same offset (9.4%), the longest aligned run 2
bytes; the longest common run at any offsets 4 bytes, twice. PASS against the thresholds decided beforehand (at most
25%, no run over 6). Two faster versions failed the check, a 9-byte opening and an 8-byte block loop, and were
discarded (D-38).

### 35.3 The accuracy cost

- **The bytes.** A program reading $FFC0-$FFFF with the ROM mapped sees VenusRT's bytes. Only $FFC0's $CD (blargg's
  SPC tests compare it before returning to the boot program) and the reset vector are placed to match what software is
  known to read. blargg's `spc_smp` has a test, "CPU/verify IPL ROM", that compares all 64 bytes; it fails, and
  cannot pass without the console's bytes.
- **The handshake's timing.** On spc_dsp6's uploader, which is bound by the SPC700's latency, one byte takes 602 master
  clocks (the median pass of the S-CPU's loop) against 510 in Mesen with the console's program, about four SPC700
  cycles; 40 clocks of it are the deliberately slower echo that keeps the loop from being the console's sequence.
  Super Mario World's uploader is bound by the S-CPU and does not move with the boot program (612.65 clocks a byte
  in every variant, against Mesen's 569.26, a difference outside it).
- **The state on the jump.** RAM $0000-$0001 hold the entry point and $0002-$00EF are zero, as fullsnes states. SP is
  $EF, which gilyon's `spctest` requires (D-38). A, X, Y and PSW are this program's, as no document gives the
  console's.

### 35.4 The oracles (measured 2026-10-04)

| Check | Result |
|---|---|
| The corpus, VenusRT as the third engine, against §34's run | one verdict changes: `spc_smp`, Passed to Failed ("CPU/verify IPL ROM"), its picture no longer Mesen's. 107 of Mesen's 117 pass (108); pictures equal 229 of 295 (230); VRAM equal 255 (256). C# Venus's and Mesen's columns identical. Cells move without a verdict change on spc_dsp6 (still incomplete), the 2010 timing ROMs `test_speed`, `test_timer_speed`, `_speed2`, `_speed3`, `_speed_2` (printed counts by one), lidnariq's `smpspeed` and undisbeliever's `ipl-speed-test`, which times the boot program itself |
| blargg's SPC ROMs | `spc_timer` and `spc_mem_access_times` pass; `spc_smp` fails as above |
| gilyon `spctest`, both copies | pass, to test $0557. With SP $F0 they failed at $01AE, which settled SP (D-38) |
| The 2010 ROMs | every verdict as before |
| spc_dsp6, its 111 tests singly, by §30.2's runner (600 frames, two rounds without "Failed") | 104 pass, 1 fails, 6 hang (106 and 5 at §30.2): "Misc/brr addr wrap-around" passed and now fails, "Order/voice 0 noise" passed and now hangs |
| The four bench games' first lit frame against Mesen | unchanged: Star Fox 210/200, Super Mario Kart 10/10, Super Mario RPG 40/40, Super Mario World 90/90 |
| The state's hash every 60 frames to 600 | the commit before, given these 64 bytes as its file 1, gives the same hash as the new code at every sample for the four bench games: the code change is neutral and the 64 bytes are the whole difference. Against runs with the console's program the hashes differ from the first sample, the boot program's run leaving other SPC700 registers, cycle position and port values; not measured, since the console's image is not run |
| The conformance kit, the eleven images of §31.2, no boot file | compliant on every one |
| The adapter against the shim (`EMUSEN_VENUSRT_GAMES`, the four bench games) | 600 frames each, the picture, the samples and the state equal |
| Tests | the crate's 91 (four new for the boot program), WiseMan's VenusRt, CoreAdapter, CoreDebug, CoreAbi, NativeHost, CoreDiscovery, Snes and Firmware filters, 199 of 199 |
| The cost, `frame_cost` best of three over 1,200 frames, in turn under the timing lock | Super Mario World 2.11-2.12 against 2.14 ms a frame, Donkey Kong Country 2.08-2.09 against 2.10-2.11: the checks of an absent boot ROM are gone, about 1% |

### 35.5 What is open

**The two spc_dsp6 tests are a sensitivity of VenusRT's that this program exposes, not a fault in its protocol**,
argued from these measurements. "Misc/brr addr wrap-around" never reads $FFC0-$FFFF outside the boot program; the
APU RAM after its upload equals Mesen's but for the test's two delay counters; and it fails with every one of 34 S-CPU
power-on offsets across a line and 13 jump delays (0 and 2 to 14 SPC700 cycles), in its first to seventh round
according to the phase. A test that passes on a console cannot depend on that phase, since there the S-CPU's kick
reaches the SPC700 at a random point of the S-DSP's period. Mesen passes it near frame 2,800; with the console's
program VenusRT showed four rounds without failure in §30.2's 600 frames, and no longer run was recorded. "Order/voice
0 noise" gives no verdict in Mesen by frame 6,000. Both are left for the S-DSP's next step.

## 36. Stage 7: the debugger (2026-10-04)

`VenusRT_Plan.md` §4.7 asked for a debug target from Rust exports, since no C# core can read VenusRT's state. Revised by
`EmuSen_CoreAPI.md` §13.2 to no per-core C# class, the target is the generic `CoreDebugTarget` over the v1 debug
exports, for any core that claims `DEBUG`. The stage was built in four commits: the debugger's frame, the disassemblers,
the C# bridge, and this step's completion and oracle.

### 36.1 What was built

- **The debugger's frame** (`src/debugger.rs`, `src/probe.rs`). An armed frame runs the machine's own loop with the
  shared hooks (`emusen_native::debug::Hooks`) asked before each S-CPU instruction, its calls, returns, interrupts
  and stores noted after it. The SPC700 and the cartridge's processor run inside catch-ups, so each carries a probe
  while a debugger's frame runs: its breakpoints, the steps it took for coverage, its stores, and where it stopped.
  After every S-CPU instruction the SPC700 and a NEC DSP are caught up (the SA-1 and the GSU already are) and their
  probes are read. A stop leaves the frame open; `debug_run_frame` reports the processor and its pc. A plain frame
  runs with no probe fitted, and `advance` disarms anything left. The skip-versus-draw and armed-versus-plain checks
  hold it to that.
- **Processors** 0 (the S-CPU), 1 (the SPC700) and 2 (the SA-1, the GSU or a NEC DSP), named as DianaOS's scope words
  name them: `CPU`, `SPC`, `SA1`, `GSU`, `DSP`. Each has its registers as machine info lists them (the 65C816's A, X, Y,
  S, D, PB, PC, DB, P, E for the S-CPU and the SA-1; the SPC700's A, X, Y, SP, PC, PSW; the GSU's sixteen and its
  control registers; the µPD77C25's), its breakpoints, coverage and live pc. Processor 0 has the call stack and
  stepping by depth.
- **The chips' spaces**, appended where the cartridge has the chip: `GSURAM`, `GSUBUS`, `SA1IRAM`, `BWRAM`, `SA1BUS`,
  `DSPRAM`, `DSPPRG`, with their ids kept.
- **The disassemblers** (`src/disasm/`), behind `DEBUG_DISASSEMBLE`, each from its document: the 65C816 from WDC's
  datasheet (Table 5-4's opcode matrix and Table 6-2's operand formats), the SPC700, the GSU (with ALT1, ALT2 and
  WITH carried from byte to byte) and the µPD77C25/µPD96050 from fullsnes's tables. Each gives the static reference
  a call, jump, read or write makes, which the target classifies. The instruction set follows the space where the
  space is a processor's own (APURAM the SPC700's, and so on), else the processor asked. The SPC700's listing of
  $FFC0-$FFFF reads the boot program while CONTROL maps it, as the SPC700 executes it, not the RAM beneath.
- **The bridge** (`EmuSen/Cores/Native/CoreDebugBridge.cs`). It pushes every processor's breakpoints, the call stack,
  the write watches, the data breakpoints and the coverage flags down as the core's tables. It runs an armed frame
  through `debug_run_frame`, judging each stop with the registry of the processor that stopped (conditions, hit
  counts, steps), and drains stores, calls, the profile and coverage back into the registries. `CoreEngine` halts in
  front of a breakpoint on any processor (`IsHaltedAtBreakpoint`, `HaltedProcessor`, `ICoprocessorHalt`) and resumes
  the open frame on the next `RunFrame`. A loaded state closes the frame.
- **Completed in this step:**
  - *A processor's code space from machine info.* `processors[]` gains an optional `code_space`, the space id its code is
    listed from (`EmuSen_CoreAPI.md` §6.4). VenusRT names CpuBus, APURAM, SA1BUS, GSUBUS and DSPPRG, so `disasm spc`
    lists APURAM as C# Venus's target does. Absent, the host keeps its rule by name, which MoonRT and MercuryRT use.
  - *Stepping over and out.* Processor 0's registry is given the bridge's call stack, without which `step over` was
    refused.
  - *Every processor's registers refreshed* with `RefreshProviders`, so `regs spc` and `eval spc a` show live values.

### 36.2 The oracle (measured 2026-10-04)

**§4.7's claims as tests** (`EmuSen.WiseMan/Cores/VenusRtDebugTests.cs`), on cartridges of `SyntheticRom` with
synthetic firmware for the DSP-1, through `CoreEngine`, `CoreDebugTarget` and DianaOS's own commands:

| Claim | Test |
|---|---|
| Processors named as the scope words, the chip as the coprocessor | `cpu`, `spc` on a plain cartridge; `sa1`, `gsu`, `dsp` added by the chip; `CoprocessorBreakpoints` and `CoprocessorCoverage` the chip's |
| Spaces | the console's eight, and the chip's two or three |
| Code spaces | CpuBus, APURAM, SA1BUS, GSUBUS, DSPPRG |
| Registers | named and live for the S-CPU and the SPC700; `regs spc` through the shell |
| Breakpoints, by scope word | `bp spc add` reaches the SPC700's registry only, `bp add` the S-CPU's |
| Halting | a breakpoint on the S-CPU halts in front of its instruction with the frame open, and resuming completes it; on the SPC700 it halts on processor 1; on the DSP-1 on processor 2 |
| Stepping | four single steps through a JSR into its routine and back; step over runs the routine whole; three steps on the SPC700 alone |
| Call stack | the JSR's frame in front of a breakpoint in the routine, and in `bt` |
| Watches | a WRAM write watch records the store with its value and the storing instruction |
| Coverage | kept per processor; the S-CPU's executed instructions and nothing else |
| Expressions | `eval a` and `eval spc sp` read each processor's registers |
| Disassembly | `disasm spc` equals `disasm APURAM`; a JSR classified as a Call to its target; the SPC700, GSU and µPD77C25 decoders on their spaces |
| States | a state loaded while halted starts a whole frame |

19 of 19 pass.

**Every table armed, on four games** (`Every_table_armed_with_nothing_to_hit_gives_the_plain_run`, opt-in through
`EMUSEN_VENUSRT_GAMES`). Two engines ran each game for 600 frames, the pad pressed in turn. One ran plain. The other
had coverage armed on every processor, a breakpoint no processor reaches on each, the profiler, and write watches over
CpuBus, WRAM, SRAM and APURAM. The picture and the samples were equal every frame and the states byte-identical at the
end, for Super Mario World, Super Mario RPG (SA-1), Star Fox (GSU) and Super Mario Kart (DSP-1B), with 2.5 to 5.0
million stores watched and 7.5 to 8.9 million instructions profiled. On a third engine, 120 frames in, a breakpoint over
the cartridge processor's whole space halted on it: the SA-1 at $C08171, the GSU at $01B301, the DSP-1B at $00000C.
Removing the breakpoint, the next frame completed.

A first version checked the coprocessor halt on the armed engine after its 600 frames, and Star Fox did not halt on
the GSU. The machine and the bridge were sound: the core alone halted at once, and so did the engine with every table
armed. After 600 frames of buttons pressed in turn, the game stands in a scene where the GSU does not run. The check
moved to its own engine, 120 frames in.

**The conformance kit**, from the branch's release libraries, on the eleven images of §31.2: all fifteen cases pass on
every one, C15 included (a `RING` stop continued, a breakpoint at processor 0's next instruction, eight steps under
`EACH`); C14 finds 3 processors and 11 spaces on Super Mario RPG.

**Tests**: the crate's 91; WiseMan's VenusRt, CoreAdapter, CoreDebug, CoreAbi, NativeHost, CoreDiscovery, MoonRt,
MercuryRt, CoprocessorDebug, NecDspDebugTarget, SetRegVectors, Snes, Firmware, MoonDebug and MercuryDebug filters, 755
of 755; `emusen-native` 50, the kit's own suites. Two MercuryRT crate tests fail (`a_state_that_is_not_mercurys_or_is_cut_short_changes_nothing`,
`version_6_writes_the_cartridge_once_...`). They fail identically on the merged tree before this step, which touches
MercuryRT only by the descriptor's new field, and are not this stage's.

### 36.3 What §4.7 lists that v1 does not carry

`EmuSen_CoreAPI.md` §6.14 bounds the generic debugger: memory, registers, disassembly, breakpoints, watches,
stepping, the call stack, coverage and the profile. Console-shaped views wait for extension exports or a system pack
(§15, Q10). On VenusRT, then:

- **Not on the generic target:** the video, APU and coprocessor register providers with history, sprites, palettes,
  audio channels, hardware load, interrupt vectors, DMA channels and the DMA log. A test holds the views empty.
- **Not built:** register flow, freezes and access counters. Freezes undo each write as it lands, and access counters
  count reads; v1 reports a frame's stores after it, and no reads.
- **Kept from before:** cheats, the frame log and labels, all host-side.

The plain frame is untouched by this step, so it has no cost to measure.

## 37. The NEC DSP replacements, step 1: the oracle over the low-level path (2026-10-04)

Step 1 of `VenusRT_DspHle.md` §8: the port driver, the command oracle, the trace recorder and the bench games' command
histograms, run against the low-level path (LLE) alone. No replacement exists yet. Everything below was measured on
2026-10-04 with the tools named, from the tester's dumps in `home/Firmware` (split pairs). No dump, no LLE output in
bulk and no game trace is in the repository; the reports are in `~/.cache/emusen/probe/venusrt/dsp-hle/`. Only counts,
shares and cycle numbers are recorded here, and individual values only where fullsnes documents them.

### 37.1 What was built

- **`chips/necdsp.rs`, one additive change**: a `Transfer` log, `NecDsp::transfers`, `None` unless the oracle fits it,
  and not part of the state. When fitted, it records the chip cycle of every DR transfer: the chip's read that raises
  RQM, the chip's write, the S-CPU's DR reads and writes, and its stores into an ST01x's RAM. The chip's two entries
  carry the program counter after the instruction. Unfitted, the cost is one branch per DR transfer.
- **`chips/dsporacle.rs`**, the driver and the oracle:
  - the `Chip` trait, which offers a clock, the ports and the edge each cycle made, the only interface a replacement
    will need in order to be graded. `Lle` implements it over `NecDsp`;
  - `Places`, which tells the chip's idle place from its command place by the program counter alone. The idle place is
    the first edge after power-on, and the command place is the first read edge after the S-CPU's first write at idle;
  - `Host`, how the S-CPU answers. It has a delay to the first byte, a gap between a word's two bytes, and limits;
    `FASTEST`, 4 and 3 chip cycles, is about one S-CPU bus cycle each;
  - `transact`, which drives one command from idle. It writes the next input wherever the chip asks for one and reads
    wherever it wrote. Each transfer is recorded as a `Step` with its direction, value, byte count, latency, answer
    time, SR at RQM's rise and at the first byte, and the edges made after the rise. The run ends `Idle`, `Ignored`,
    `Stalled` or `Capped`;
  - `compare`, `timing` with `predict` (§37.3), the ST010's `st_ready` and `mailbox`, `sweep`, `sweep_mailbox` and
    `mirror_classes`;
  - `firmware`, which reads a chip's image from `EMUSEN_VENUSRT_FIRMWARE` as a pair or whole, checks its size, and
    otherwise prints "not run";
  - `Pcg`, the seeded generator.
- **Eight tests.** The driver on a synthetic program; the timing model on its own. Through the real images: fullsnes's
  ROM versions and DR on completion, and SNESdev's Multiply of 4000h by 4000h; the ST010's 00h and its mirrors; the
  DSP-2's and DSP-4's documented mirrors; every command byte of all seven chips from two power-on phases; the model's
  prediction of a jittered S-CPU on the DSP-1B; and the DSP-1 against the DSP-1B. Expected values are fullsnes's or
  come from running the LLE at test time. The tests pass without an image, saying "not run".
- **`examples/dsp_oracle.rs`**: `sweep`, `versus`, `pair`, `latency` and `rate`.
- **`examples/dsp_trace.rs`**: the trace recorder, which records every command a game gives its chip with the frame,
  inputs, results and each transfer's latency, and the game's histogram. It runs no battery file, so runs reproduce.
  Its pad scripts use the harness's verbs (`frames`, `tap`, `hold`, `release`, `tapuntil ... wram`), plus
  `tapuntil BTN chip N`, which taps until N commands that compute have reached the chip, and `shot`. The five scripts
  are in `examples/dsp_pads/`.

### 37.2 The protocol as the ports show it

- **DSP-1, DSP-1B, DSP-3.** At idle the chip writes 80h in 8-bit mode. Its read of the command byte raises RQM, which
  is the request for the first input. DRC changes to 16-bit 3 cycles after that rise, so a driver that answered sooner
  would move one byte, not a word. Each input read raises RQM for the next, the last is read without raising it, and
  the results follow as writes. A command without inputs still raises RQM at its read, so one transfer in either
  direction (fullsnes's "oblivious" chip) comes before its first result: Multiply is `i2o`, the ROM version `io`.
- **Bytes passed over.** On the DSP-1 and DSP-1B, 40h-FFh are read and passed over: the chip takes the next byte as a
  command without returning to idle (`End::Ignored`). The bench games resynchronise this way: Pilotwings and Super
  Mario Kart write 128 bytes of 80h at boot, Suzuka 8 Hours 256, Super Bases Loaded 2 one FFh. The DSP-3's 40h-FFh form
  one class too.
- **Raster runs.** 0Ah gives one line of four results after another for as long as the S-CPU reads. Every bench game
  ends a run by writing over the results it no longer wants, after which the chip returns to idle: four words a run in
  Pilotwings, Super Mario Kart, Lock On and Suzuka 8 Hours, one word in Michael Andretti's Indy Car Challenge and Super
  Bases Loaded 2.
- **DSP-2.** 8-bit throughout. At idle the chip writes 00h after power-on and FFh after a command.
- **DSP-4.** 16-bit throughout, the command word included; idle is FFFFh. An S-CPU read at idle makes the chip write
  FFFFh again, and Top Gear 3000 polls that way (514 reads before its first command). The first transfer after a
  command is a write by the chip.
- **ST010.** At power-on the chip writes a word to DR. It serves its mailbox only after the S-CPU has read that word:
  without the read, no command completed within 5,000 cycles. F1 ROC II reads it once. The busy flag is polled every 3
  cycles, so the mailbox's latency moves with the start's phase modulo 3, by a constant for every command.
- **ST011.** It does not answer the mailbox. It speaks through DR in 8-bit mode and signals with USF1 and USF0.

### 37.3 The latency model, corrected

**`VenusRT_DspHle.md` §6.2's model does not hold.** That model takes one constant per phase, counted from the S-CPU's
completion of the previous transfer. The measured latency from completion depends on when the S-CPU answered, because
the chip goes on computing while RQM is high. For Triangle (04h), with the S-CPU answering d cycles after each rise, the
second input's latency is 14 - d down to a floor of 2, and the result's 28 - d down to a floor of 4: the edges
themselves come 14 and 28 cycles after the preceding ones, or 2 and 4 after the answer, whichever is later.

**Two numbers per phase** describe it: the *work*, cycles from the previous rise, and the *notice*, cycles from the
S-CPU's completion. RQM rises at the later of the two. `dsporacle::timing` measures them from two runs, one with the
S-CPU answering at `FASTEST` and one answering after all work is done, and `predict` gives the latencies of any other
run. Against runs whose answer time was drawn anew at every transfer, from 4 to 29 cycles to the first byte and 3 to
6 between bytes:

| Chip | Seeded cases | Predicted exactly | Mispredicted | Not modelled (the transfers change with the answer time, or capped) |
|---|---|---|---|---|
| DSP-1, DSP-1B | 1,024 each | 1,008 | 0 | 16 |
| DSP-2 | 1,024 | 832 | 0 | 192 |
| DSP-3 | 1,024 | 860 | 0 | 164 |
| DSP-4 | 1,024 | 939 | 0 | 85 |
| ST011 | 1,024 | 992 | 0 | 32 |

**The start's phase does not matter** on the DR chips. Over 1,024 cases each, a start 10,007 cycles later gave the
same values, transfers, SR and latencies, so every wait loop resolves to the cycle. On the ST010 only the poll period of
§37.2 moves the latency.

**Some transfers are not handshaken.** On these command bytes the values or the sequence of transfers change with the
S-CPU's answer time:

- **DSP-1, DSP-1B**: none.
- **DSP-2**: 01h, 0Fh and their mirrors, and the 0Eh class. On 01h, the command Dungeon Master gives most, the change
  comes once the S-CPU answers 16 cycles or more after a rise (12 still answers alike). This is fullsnes's remark that
  the game moves the data with block moves and no handshake, now seen at the ports.
- **DSP-3**: 22 bytes, among them 02h, 06h, 07h, 18h and 38h.
- **DSP-4**: 00h, 09h, 0Ah and 0Fh, and five of 11h's mirrors in single cases.
- **ST011**: 03h, 0Fh and F2h.

A replacement must reproduce these by the same rule, which §6.4 of the plan anticipated for the DSP-2 alone.

**The DSP-1B's phases** for the commands the bench games use, from `dsp_oracle latency dsp1b 256`, as work/notice in
chip cycles. "-" is work hidden behind the fastest answer, and "a-b (n)" a range of n values over the seeded cases. The
first phase runs from the command byte, and the second absorbs the command's decoding:

| Command | Transfers | Phases |
|---|---|---|
| 00h Multiply | `i2o` | -/2, 14/2, -/4, -/3 |
| 01h, 11h Attitude | `i4` | -/2, 15/2, -/2, -/2, 119-132 (7)/91-99 (3) |
| 02h Parameter | `i7o4` | -/2, 13/2, -/2 x5, 489-545 (47)/448-503 (45), -/2, 10/2, -/2, -/3 |
| 03h, 13h, 0Dh Subjective, Objective | `i3o3` | -/2, 15/2, -/2, -/5, -/2, -/2, -/3 |
| 04h Triangle | `i2o2` | -/2, 14/2, 28-33 (4)/4, -/2, -/3 |
| 06h Project | `i3o3` | -/2, 14/2, -/2, -/245-269 (21), -/2, -/2, -/3 |
| 0Ah Raster | `io4...` | with zero inputs, a line's first result 71 cycles after the read of the last line's fourth (64 on 1Ah-3Ah); per input not yet measured |
| 0Ch Rotate | `i3o2` | -/2, 14/2, -/2, 31-35 (2)/4, -/2, -/3 |
| 0Eh Target | `i2o2` | -/2, 13/2, -/72, -/2, -/3 |
| 10h Inverse | `i2o2` | -/2, 15/2, -/36-54 (10), -/2, -/3 |
| 14h Gyrate | `i6o3` | -/2, 16/2, -/2 x4, 201-242 (35)/3, -/2, -/2, -/3 |
| 1Ch Polar | `i6o3` | -/2, 15/2, -/2 x4, 89-102 (8)/33-38 (4), -/2, -/2, -/3 |
| 28h Distance | `i3o` | -/2, 17/2, -/2, -/59-74 (5), -/3; on the DSP-1, -/58-73 (5) |

Against SNESdev's split, the first input's work less its notice equals SNESdev's first input column for seven of the
eight commands it splits (Inverse is one cycle over). The later phases agree only for Multiply, Radius and Range; for
Triangle, only at the range's lower end. Inverse, Distance, Rotate and Polar compute in fewer cycles than SNESdev gives
(36-54 against 73; 59-74 against 127; 27-31 against 37; at most 64 against 107).

**The rate.** Multiply runs back to back on one chip at 2.49 million cases a second, about 152 million chip cycles a
second (`dsp_oracle rate dsp1b 00`, under the timing lock with the load average at 8). 2^32 pairs would take 29
minutes on one thread, and about 3.6 minutes on eight if the work scales.

### 37.4 The DSP-1 against the DSP-1B

From `dsp_oracle versus dsp1 dsp1b 2048`, 2,048 seeded cases per command byte, and `pair` for sequences:

- **28h Distance**: the values differ in 1,049 of 2,048 cases, and the latency in all of them. This is the fix
  fullsnes names.
- **2Fh, 27h**: 0100h against 0101h, as fullsnes says.
- **17h, 1Fh, 37h, 3Fh**, the data ROM dump and its mirrors: the two data ROMs differ.
- **40h-FFh**: both pass the byte over. The rest of each transaction is the following byte's own command (on the
  DSP-1B, 12,288 seeded cases agree but at the transfer cap), so it differs exactly where that command does.
- **Pairs**: of every command from 00h to 3Fh followed by every other, only the memory test (07h, 0Fh) changes a later
  command. After it, Raster (0Ah) differs in 4,096 of 4,096 cases and Target (0Eh) in 4,058; Project does not. The
  memory test leaves the RAM in a state Parameter never produces. After Parameter itself, Raster, Target and Project
  agree in 4,096 of 4,096 cases each.
- **Sequences**: 16,384 random commands from the bytes in neither list, without a reset, agree.

Everything else agrees bit for bit, latency included. **The two programs differ, for a game, only in Distance**, which
Pilotwings, Super Mario Kart, Michael Andretti's Indy Car Challenge, Lock On and Super Bases Loaded 2 use (§37.6).

### 37.5 The sweeps

The 256 command bytes per chip: zero inputs and four seeded sets of 64 words for the DR chips, and four seeded RAM
fills for the ST010 (`dsp_oracle sweep`).

| Chip | Distinct behaviours | What the classes show |
|---|---|---|
| DSP-1, DSP-1B | 27 | fullsnes's commands, and as alike: 04h/24h, 0Ch/2Ch, 0Eh/1Eh/2Eh/3Eh, 10h/30h, 1Ch/3Ch, 17h/1Fh/37h/3Fh, 07h/0Fh, 27h/2Fh. **20h is not 00h, and 38h is not 18h**, though fullsnes names each pair alike. 0Ah, 1Ah-3Ah are raster runs, 1Ah-3Ah with a shorter line. Gyrate (14h) takes six inputs and gives three. On a fresh chip the attitude matrices are zero, so commands that read them look alike here (03h with 0Dh; 01h with 05h); telling them apart needs a setter first, left to step 5 |
| DSP-2 | 18 | fullsnes's "10h..FFh mirrors of 00h..0Fh" holds for every command it names, and for all but 0Eh, whose 16 bytes form four classes by bits 4-5. 09h and 0Ah answer alike |
| DSP-3 | 21 | 2Fh gives 0300h; mirrors with periods from 4 to 20h within 00h-3Fh; 40h-FFh one class. 1Ch, 3Ch and 20h stall with zero inputs (no edge in 2 million cycles) |
| DSP-4 | 22 | 13h gives 1,024 words and 14h 0400h. 15h-1Eh do nothing, as fullsnes says; 1Fh also does nothing, a cycle sooner. 20h-FFh mirror 10h-1Fh in every case. 03h and 0Eh do nothing in 377 cycles |
| ST010 | 9 | exactly fullsnes's list: 00h-08h, 09h-0Fh mirroring 01h-07h, 10h-FFh mirroring 00h-0Fh |
| ST011 | 16 | 00h, 08h, 0Ah, 0Dh, 10h-EFh, F0h and F4h-FFh do nothing, F0h with F4h-FFh (18 cycles) and not with 10h-EFh (12) as fullsnes groups it; 01h takes 128 bytes; 0Bh, 0Ch and 0Eh compute for about 1,000-1,800 cycles; F1h-F3h are their own |

**ST010 00h.** fullsnes's "Set RAM[0010h]=0000h" is the chip's word 0010h, the mailbox that the S-CPU reaches as bytes
0020h-0021h. Every command clears it on completion. 00h clears it and changes nothing else (test
`the_st010_mailbox_answers_and_mirrors_as_documented`). The superfamicom wiki's parameter tables are in bytes: 01h
changes words 0-3 and 8, its X1, Y1, Quadrant, Y0 and Theta.

### 37.6 The bench games' command histograms

`dsp_trace <rom> 7200 [script]`: 7,200 frames from power-on, no battery file. Shares are of every command recorded,
the resynchronising bytes included (at most 256 a game). The third column is the first command below 40h other than a
test.

| Game (chip) | Input | First command that computes | Commands, as a share of the total |
|---|---|---|---|
| Pilotwings (DSP-1) | none | frame 1,798 | 122,649: 00h 30%, 13h 23%, 06h 19%, 0Ah, 0Eh, 02h, 0Dh 3% each, 01h, 03h, 14h, 28h, 11h 2.9% each, 0Ch 1.5% |
| Super Mario Kart (DSP-1B) | none | 95 | 54,580: 06h 56%, 04h 23%, 02h 12%, 00h 6%, 28h 2%, 0Ah 0.2% |
| Ballz 3D (DSP-1B) | none | 760 | 112,641: 06h 48%, 03h 47%, 01h 2%, 0Ch 2%, 02h 1%, 04h 0.6%; 0Fh and 2Fh once at boot |
| Michael Andretti's Indy Car Challenge (DSP-1B) | `andretti.txt`, Start | 3,309 | 22,502: 06h 46%, 28h 30%, 04h 13%, 02h 6%, 0Ah 6%; 0Fh and 2Fh once at boot |
| Lock On (DSP-1B) | `lockon.txt`, Start | 1,522 | 266,536: 28h 21.5%, 00h 13.7%, 10h 13.7%, 01h 13.3%, 03h 13.3%, 06h 9.5%, 1Ch 8.8%, 02h 4%, 0Ah 2% |
| Super Bases Loaded 2 (DSP-1B) | none | 3,028 | 14,434: 06h 94%, 28h 3%, 02h 1%, 04h 1%, 0Ah 0.5%, 0Ch 0.3% |
| Suzuka 8 Hours (DSP-1B) | `suzuka.txt`, Start | 1,473 | 25,724: 02h 47%, 06h 44%, 00h, 04h, 0Ch 2% each, 0Ah 0.5% |
| Dungeon Master (DSP-2) | `dm.txt`, Start | 1,035 | 334,894: 05h 66.5%, 01h 29%, 0Fh 3%, 03h 0.8%, 09h 0.8% |
| Top Gear 3000 (DSP-4) | `tg3000.txt`, through the menus and the shop | 3,006 | 223,899: 00h 82%, 0Ah 11%, 0Bh 1.3%, 01h, 03h, 05h, 06h, 07h, 08h, 09h 0.7% each, 11h 0.3% |
| F1 ROC II (ST010) | none | 1,005 | 183,207 mailbox commands: 05h 53.6%, 08h 17%, 03h 12.5%, 04h 12.5%, 02h 2%, 07h 2%, 06h 12 times |

**Every script reaches the chip**, and the five games without one reach it without input. Super Bases Loaded 2 and
F1 ROC II reach their chips in the attract mode, against §24.2 and plan §4.3. Michael Andretti's Indy Car Challenge and
Top Gear 3000, which §24.2 counted as using their chips by frame 2,400, give no command that computes without input.
There the chip's presence changed WRAM through the boot's tests (Andretti's 0Fh and 2Fh) and the idle reads (Top Gear
3000's 514), not through commands. Suzuka 8 Hours, not in §24.2, needs input too: without it no command arrives in
3,600 frames, and at frame 3,368 a DMA reads DR, which the chip takes as bytes passed over. Top Gear 3000's script
follows its menus exactly and is the one likely to need rewriting if the game's timing moves.

**The union of commands used**, which sets the scope of the later steps:

- **DSP-1 family**: 00h, 01h/11h, 02h, 03h/13h, 04h, 06h, 0Ah, 0Ch, 0Dh, 0Eh, 10h, 14h, 1Ch and 28h, with 0Fh and 2Fh at
  boot. No bench game uses Radius (08h), Range (18h, 38h), Scalar (0Bh-2Bh), the second Multiply (20h), the data ROM
  dump (1Fh), Raster's other forms or attitude C.
- **DSP-2**: 01h, 03h, 05h, 09h and 0Fh. 06h and 0Dh are unused.
- **DSP-4**: 00h, 01h, 03h, 05h-0Bh and 11h. 00h carries 82% of the commands.
- **ST010**: 02h-08h, all seven computing commands.

### 37.7 Against the plan's predictions and claims

- **P3, retired, false in its second half.** For only three of SNESdev's eight commands (Multiply, Radius, Range) are
  all phases within 2 cycles of its columns, and four if Triangle's lowest is taken. The first half, a constant or a
  function of one input feature per phase, is open: nine of the DSP-1B's commands have a phase that varies (§37.3), and
  which feature drives each is steps 3-7's question.
- **P4, holds by extrapolation**: about 3.6 minutes on eight threads against the 30 predicted (§37.3), not run.
- **P5, holds**: no bench game issues 1Fh or 13h in 7,200 frames.
- **P7, retired, false**: F1 ROC II issues seven distinct commands in its attract mode, against at most three.
- **§6.2's model** is replaced by work and notice (§37.3). **§6.4's unhandshaken transfers** are not the DSP-2's alone:
  the DSP-3, DSP-4 and ST011 have them too.
- **§2.1 and §3.5**: the ST010 needs its power-on DR word read before its mailbox runs; fullsnes's RAM[0010h] for 00h is
  a word address.
- **§3.1**: 20h and 38h are not the alike forms the table presumes; Gyrate takes six inputs and gives three; Raster is a
  run, ended by the S-CPU's writes.
- **§4.3**: of the four games said not to reach their chips without input, two do; three others need input.

## 38. The NEC DSP replacements, step 2: the replacement's frame (2026-10-04)

Step 2 of `VenusRT_DspHle.md` §8. A DSP game now runs with no firmware folder: on VenusRT's open replacement for the
DSP-1, DSP-1B, DSP-2 and ST010, or without its chip for the DSP-3, DSP-4 and ST011. The player's image, when present,
stays the exact path. The replacement is a frame. Its ports behave as the chip's, and of its commands only those
fullsnes fixes answer exactly. Steps 3 to 11 fill it in command by command against §37's oracle.

### 38.1 What was built

- **The slot.** `chips/dspengine.rs` holds `DspEngine`, either `Lle(NecDsp)` or `Hle(DspHle)`, behind the low-level
  path's port interface (`host_read`, `host_write`, `run_to`, `reset`, `pack`). The bus code is unchanged.
  - Under the replacement there is no µPD77C25 program, so the debugger lists neither processor 2 nor the `DSPPRG`
    space.
  - `DSPRAM` stays for the ST010, whose RAM is real on both engines and is the battery file as before.
- **The replacement.** `chips/dsphle.rs` is a state machine over the ports with one scheduled chip edge. Its DR and SR
  handshake is the low-level path's, byte for byte, including the "oblivious" chip (a read or a write completes a
  transfer alike). It follows §37.2:
  - **DSP-1 and DSP-1B.** At idle the chip writes 80h in 8-bit mode. The command's read raises RQM, and DRC turns to
    16-bit 3 cycles later. A command without inputs takes one transfer before its result. 40h-FFh are passed over, and
    a raster run repeats its line until the S-CPU writes over a result.
  - **DSP-2.** 8-bit throughout, with the idle word 00h after power-on and FFh after a command.
  - **ST010.** A word written to DR at power-on, and the mailbox served only after that word is read.
- **What it computes.**
  - Exactly: the DSP-1's 2Fh (0100h, and 0101h on the DSP-1B), the DSP-2's 0Fh, a no-op, and the ST010's 00h, which
    clears the mailbox and nothing else.
  - Every other DSP-1 command takes its documented transfers and gives zeros. The counts come from fullsnes's codes,
    SnesLab's parameter lists and SNESdev's two words for Radius. Gyrate's six inputs and three results are
    characterised (§37.5): a count, the one choice R2 leaves open there, since SnesLab's list of three inputs and one
    result would desynchronise every game that uses it.
  - A DSP-2 command other than 0Fh returns to idle, since nothing documents its lengths. An ST010 command clears its
    busy bit 16 cycles after it is set.
  - **Timing** is one notice of 2 cycles for every phase. The work of §37.3 is not yet modelled.
- **No table** is written in the source. A crate test fails on an array literal of more than 16 numbers in
  `dsphle.rs`: this is plan §5.4's guard.
- **Selection at create** (`ffi.rs`, shared by both export sets): a file 2 fits the low-level path. Without one, a
  cartridge whose chip has a replacement gets it, and the DSP-3, DSP-4 and ST011 run without their chip as before.
- **The state** is version 18. A DSP cartridge's `Coprocessor` group gains `DspEngine`, 0 for the low-level path and 1
  for the replacement, then `NecDsp` or `DspHle`. A version 17 state still loads, as the low-level path's. A state
  written under the other engine is refused, the machine unchanged, with status -11 and its words.
- **The descriptors**:
  - **emusen-native.** `Firmware` gains `replacement: Option<Replacement>`, which is `Exact`, `Accuracy { cost }` or
    `None { cost }`. `MachineInfo` gains `firmware: Vec<(which, FirmwareSource)>`, where the source is `File`,
    `Replacement` or `Absent`.
  - **Schemas and baseline.** The firmware, info and machine-info schemas carry the new fields, and the ABI baseline
    gains their nine lines at 1.0, since no minor has been released (`EmuSen_CoreAPI.md` §4.1). §6.2 and §6.4 there
    record them.
  - **VenusRT's answers.** Every firmware entry is `required: false`. The DSP-1, DSP-2 and ST010 entries carry
    `accuracy` with a cost naming what is exact; the DSP-3, DSP-4 and ST011 entries carry `none`. Machine info lists
    the path the cartridge runs on.
- **The C# side**:
  - `CoreFirmware.Replacement`, `CoreReplacement`, `CoreFirmwareSource` and `CoreMachineInfo.Firmware`.
  - `FirmwareRequest` carries `Required` as a field, true unless a core says otherwise, with the replacement's effect
    and cost.
  - `EmulatorSession.MissingFirmwareFor` returns required requests only, so Mistress's picker never opens for VenusRT.
    C# Venus's NEC DSP requests stay required.
  - `CoreEngine.FirmwareNotice`, read through `ICore` and the session, gives Mistress one status line when the game
    runs on a replacement short of exact or without its chip.
- **The conformance kit.** C4 checks each entry's `replacement.effect` word, and its cost unless the effect is `exact`.
  It also creates an image whose entries are all optional with no files.
- **The oracle.** `dsporacle::Hle` implements `Chip` over the replacement, so steps 3 onward grade it through the same
  driver as the image.

### 38.2 Measured 2026-10-04

| Check | Result |
|---|---|
| The crate's tests, with the tester's images | 108 of 108, nine new with §38.4's. `the_replacements_rom_version_agrees_with_the_image`: the replacement's 2Fh gives the same values and transfers as the DSP-1's and DSP-1B's images, and 41h is passed over on both. Latency is not compared until step 3 |
| emusen-native, the conformance kit's tests, the ABI check | 51, 9, and 382 facts agreeing with the header on four triples and the baseline |
| The kit against VenusRT with no files: Super Mario Kart, Pilotwings, Dungeon Master, F1 ROC II, Top Gear 3000, 300 frames, and after §38.4 Super Mario Kart and Super Mario World, and Super Mario World with the tester's boot image as file 1 | compliant on every one; C4 notes "every firmware entry optional (1): created with no files". The first Super Mario Kart run failed C15: a debug-armed run's state differed from a plain one, because `run_to` made only one due edge per catch-up. It now makes every due edge |
| The games without an image at frame 1,500, by eye | Super Mario Kart's title, Pilotwings' menu, F1 ROC II's race, Dungeon Master's opening text, all drawn. Their 3D results are zeros, so play is wrong where a game computes with its chip |
| WiseMan, each run under an 8 GB cap | the Mistress suite, 1,641 passed and 51 skipped, before §38.4; after it, the VenusRt, CoreAdapter, Firmware, NecDsp, CoreAbi, CoreDiscovery, NativeHost, CoreDebug, Conform, MainWindowLibrary and WindowFitAudit filters, 319 of 319. Among them are the new `A_dsp_cartridge_runs_on_the_replacement_with_no_firmware` (no request missing, the replacement's tag in the state, machine info's `replacement`, and the notice; a DSP-4 cartridge `absent`), `A_players_boot_image_runs_in_place_of_the_open_program` and the revised `An_ordinary_game_on_venusrt_prompts_for_no_firmware`, in which neither game asks and the DSP-1 cartridge loads with the notice |

### 38.3 What is open

- **The commands.** Steps 3 to 11 build the commands into the frame and grade each against the image. Until then the
  DSP-1, DSP-2 and ST010 games run with wrong results, which their costs state.
- **The timing.** Step 3 takes on the work and notice of §37.3. Until then every phase answers 2 cycles after the
  S-CPU, so a game's polling loops run fewer times than on the chip.
- **The DSP-2's lengths** are characterised at step 9. Until then Dungeon Master's walls are not drawn.
- **Plan §7.3's firmware window**, which lists each entry with its replacement's effect, is not built. The status line
  is the notice.

### 38.4 The player's SPC700 boot image, file 1 again

*Decided 2026-10-04* (`EmuSen_Firmware.md` §0, point 2: the player's own image is used in place of the replacement when
present). §35 withdrew file 1 outright. It returns the way the NEC DSPs' files do, and D-38's program is unchanged.

- **The interface.**
  - `firmware_for` lists `spc700.rom` for every image: 64 bytes, `required: false`, never prompted for, with
    `replacement: { effect: accuracy }` and §35.3's cost in a player's words.
  - Core info lists it too.
  - `create` takes a 64-byte file 1 and refuses any other length with `BAD_FILE`.
  - Machine info's `firmware` gives `{ which: 1, source: file }` or `replacement`.
  - A frontend finds the file in its firmware folder like any other; Mistress's status line names the open program
    when no file is there, and a DSP's line first when both apply.
- **The core.** `Smp::with_boot` maps the image at $FFC0-$FFFF in place of `apu/boot.rs`'s program and starts from its
  reset vector. The console's reset button keeps it, and the debugger's disassembly reads it while CONTROL maps it. The
  examples take it from `EMUSEN_VENUSRT_BOOT`.
- **The tests** use a synthetic 64-byte image, a loop at its own vector, for the path with a file. No image is
  committed.

**Measured 2026-10-04**, with the tester's own image from the probe cache as the input (the core reads it; its bytes
were not inspected):

| Check | VenusRT's own program | The tester's image |
|---|---|---|
| blargg `spc_smp`, the backdrop at frames 600, 1,800 and 3,600 | red from 1,800: Failed ("CPU/verify IPL ROM", §35.3) | blue at 3,600: **Passed** |
| blargg `spc_timer`, `spc_mem_access_times` | Passed | Passed |
| spc_dsp6's 111 tests singly, 600 frames, §30.2's grading | 104 pass, 1 fails, 6 hang | **106 pass, 5 hang**, §30.2's result before D-38 |
| "Misc/brr addr wrap-around" | fails | passes |
| "Order/voice 0 noise" | hangs | passes |

With the console's program, VenusRT returns to its §30.2 results. **§35.5's argument stands with this evidence**:
the two spc_dsp6 tests are a sensitivity of VenusRT's to the boot program's timing, not a fault in the open
program's protocol. The open program remains the default, and these two tests and `spc_smp` are its named cost.

## 39. The NEC DSP replacements, step 3: the DSP-1's arithmetic (2026-10-04)

Step 3 of `VenusRT_DspHle.md` §8 covers the DSP-1's Multiply (00h, 20h), Radius (08h), Range (18h, 38h), the ROM
version (2Fh) and the memory test (0Fh), with the latency model for each. Rules R1-R4 of the plan's §1.3 govern it.
§39.1 was written and committed before the first comparison with the image.

### 39.1 The families, declared before the first comparison

The documents give each command's formula. The SNESdev wiki gives the equations and "rounded to <= 15 bits" for
Multiply, and its data types: T, 15 bits after the point; L2 and H2, the halves of a 32-bit value counted in units of
2^-1. fullsnes gives the µPD77C25's multiplier, K·L·2 as a 32-bit product. What no document fixes is left as a family
of variants, and the oracle chooses one member. A command's chosen member must agree on every case graded, or the
command is a named loss under R4.

| Command | Formula | Variants (count, bits of choice) |
|---|---|---|
| 00h, 20h Multiply | P = K·I, both signed 16-bit; M = R(P / 2^15) | R: floor, half up (floor of x + 1/2), half to even, toward zero; overflow of M: wrap to 16 bits, or saturate to 7FFFh and 8000h (8, 3 bits; each code chooses on its own) |
| 08h Radius | V = 2^s·(x² + y² + z²) in 32 bits; L = V mod 2^16, H = V div 2^16 | s: -1 (floor), 0, 1; 32-bit overflow: wrap, or saturate to 7FFFFFFFh (6, under 3 bits) |
| 18h, 38h Range | V = 2^s·(x² + y² + z² - r²) in 32 bits; D = R(V / 2^16) | s: -1, 0, 1; overflow: wrap, saturate; R: floor, half up (12, under 4 bits; each code on its own) |
| 0Fh memory test | a constant on a working chip (plan §3.1, argued) | 0000h, 0001h, 00FFh, FFFFh (4, 2 bits); its effect on the chip's RAM is not modelled (§37.4) |
| 2Fh ROM version | 0100h (DSP-1), 0101h (DSP-1B), fullsnes | none |

Over the step that is at most 19 bits of choice, against the 2^32 Multiply cases and the 2^20 seeded cases of each
other command graded. **Timing** follows plan §6.3 as §37.3 corrected it. Each phase takes the work and notice that
`dsporacle::timing` measures over the seeded cases. A constant is the model; a phase that varies would need a rule
naming one feature of the inputs, and none is declared, so a varying phase in this step would be recorded as
inexact. A phase whose work the fastest answer hides takes work 0, which gives the same latency for any answer at or
above `FASTEST`.

### 39.2 Two families widened once, in writing (plan §5.2, rule 4)

The first grading, over 265,865 cases per code, had these results:

- **One member agreed on every case** for 00h (Floor, wrap), 08h (s = 1, wrap), 18h (s = 1, wrap, Floor) and 0Fh
  (0000h).
- **No member agreed for 20h or 38h.** The counts of how each differed from the nearest member, over 20,000 seeded
  cases, show the structure:
  - 20h is Multiply's floor or one more, and one more exactly when the floor is even.
  - 38h is Range's s = 1 floor plus one on every case counted.

Each family is widened once, by members that state those structures as rules, before they are graded:

- **20h**: Floor | 1 (the low bit set), or Floor + 1; wrap or saturate. That is 4 members, 2 bits.
- **38h**: s = 1, then Floor + 1, or the ceiling; wrap or saturate. That is 4 members, 2 bits.

Neither is a table: each is one operation more than the documented formula. If neither member agrees on every case,
the code is a named loss.

### 39.3 What was chosen and built, and the grade (measured 2026-10-04)

**Chosen**, each agreeing on every case `dsp_family` graded (265,865 per code on the DSP-1, and 1,052,297 on the
DSP-1B):

| Code | Member | In words |
|---|---|---|
| 00h | Floor, wrap | the high word of the multiplier's K·I·2, SNESdev's "rounded to <= 15 bits" being the floor |
| 20h | Floor \| 1, wrap | 00h with its lowest bit set |
| 08h | s = 1, wrap | x² + y² + z² in halves, as SNESdev's L2 and H2 say, in 32 bits |
| 18h | s = 1, wrap, Floor | the high word of (x² + y² + z² - r²) in halves |
| 38h | s = 1, wrap, Floor + 1 | 18h plus one |
| 0Fh | 0000h | the memory test passes |

None needs a table. Counting the widened families whole, the choices total 3 (00h) + 3 + 2 (20h) + 3 (08h) + 4 (18h)
+ 4 + 2 (38h) + 2 (0Fh) = 23 bits. The DSP-1 and DSP-1B choose alike, as §37.4's
map predicted for every code but 28h.

**Timing.** Each command's phases took one work and one notice over the seeded cases (§37.3's table). `dsphle.rs`
keeps them per code and phase, 16 pairs in all, and schedules each edge at the later of the last rise plus the work
and the S-CPU's completion plus the notice. A phase it does not list takes notice 2. The replacement's state carries
the phase and the last rise, so the state version is 19. A version 18 state of the low-level path still loads; one of
the replacement is refused with `VERSION`.

**The grade.** `dsp_grade` compares the image and the replacement through the ports on every case: values,
transfers, SR at every rise and access, and each latency. Every other case runs under a jittered S-CPU, as §37.3
measured it:

| Code | Cases | DSP-1B: differ | DSP-1: differ |
|---|---|---|---|
| 00h | every pair, 4,294,967,296 | 0 | 0 (1,190 s) |
| 20h, 08h, 18h, 38h, 0Fh, 2Fh | 1,048,576 seeded each | 0 | 0 |

The control: on 04h and 28h, which are not built, every case differs in values and latency, so the grader sees a
difference. The crate test `the_replacements_arithmetic_agrees_with_the_image` repeats the comparison over 4,752
edge and seeded cases per code on both images, steady and jittered. `the_documented_formulas_answer_through_the_ports`
checks SNESdev's equations on the replacement with no image.

**The cost.** The exhaustive pass took 1,081 s on the DSP-1B and 1,190 s on the DSP-1, eight threads each at a load of
2 to 12, running both engines and the comparison. **P4 holds**: the image alone is about five times faster than this
pass (§37.3), so well under 30 minutes.

**What this settles.** For the codes of this step, the replacement is the chip at its ports: no case of 2^32 for 00h,
nor of 2^20 for the others, differs in any value, transfer, SR bit or cycle. The DSP-1's cost in core info now names
these five codes and 2Fh as exact.

## 40. The NEC DSP replacements, step 4: the DSP-1's sine, Triangle, Rotate and Polar (2026-10-04)

Step 4 of `VenusRT_DspHle.md` §8. §40.1 was written and committed before the first comparison with the image.

### 40.1 The families, declared before the first comparison

**The sine.** The SNESdev wiki sets the units: an angle A is a full turn in 2^16 steps, and a T value carries 15 bits
after the point. Triangle's equations are S = r·sin θ and C = r·cos θ, and cos θ = sin(θ + 2^14). No document gives the
sine's resolution, rounding or interpolation. A member of the family is a table generated by a formula, never written
out (plan §5.2):

- **N**, the entries over a whole turn: 64, 128, 256, 512, 1024, 2048, 4096 or 65536 (3 bits).
- **Amplitude**: 2^15 saturated to 7FFFh, or 2^15 - 1 (1 bit).
- **Quantisation** of each entry: floor, or half up (1 bit).
- **Symmetry**: every entry from the formula directly, or a quarter wave whose magnitude is mirrored and negated, so
  that the rounding is symmetric about zero (1 bit).
- **Lookup**: the angle's top log2(N) bits, or the nearest entry (1 bit).
- **Interpolation**: none, or linear between neighbours, the step floored (1 bit).
- **The product** r·s scaled by 2^-15: floor, as the µPD77C25's multiplier gives it, or half up (1 bit).

That is 9 bits of choice: one more than P2's 8, which counted the symmetry inside quantisation. Triangle is graded over
every angle at radii 7FFFh, 4000h, 1, 8000h and FFFFh, and over 2^20 seeded pairs.

**Rotate (0Ch)**: x2 = x·cos θ - y·sin θ and y2 = x·sin θ + y·cos θ, counter-clockwise (SnesLab), with Triangle's chosen
sine. Its variants:

- the sense: counter-clockwise, or the transpose that the SNESdev matrix read as a row vector gives (1 bit);
- the rounding: each product scaled by the chosen product rule then summed, or the exact sum scaled once (1 bit);
- the overflow: wrap to 16 bits, or saturate (1 bit).

That is 3 bits.

**Polar (1Ch)**: the point (I4, I5, I6) times SNESdev's three matrices for the angles about Y (I3), X (I2) and Z (I1),
in that written order, with the chosen sine. Its variants:

- the order: as written, or reversed (1 bit);
- each matrix as written, or transposed (1 bit);
- the rounding: after each rotation, or once at the end in 32 bits (1 bit);
- the overflow: wrap, or saturate (1 bit).

That is 4 bits.

**Under R4**, a sine that no member reproduces on every angle is a stop: the DSP-1's sine is then the program's data,
and Triangle, Rotate and Polar are named losses. So is a member that agrees on all but a few entries. **Tables and
firmwarecheck**: the chosen table is generated in `dsphle.rs`, and an independent Python generator,
`EmuSen.WiseMan/Reference/analysis/dsp1_tables.py`, writes the same table from this record's formula. A crate test
asserts the two are equal word for word, and `firmwarecheck.py` is run plain and `--forced` against the data ROM.

### 40.2 The sine family rejected, and widened once, in writing (plan §5.2, rule 4)

`dsp_sine dsp1b 262144` graded the 352 members of §40.1 over every angle at five radii and 262,144 seeded pairs,
589,824 cases. **No member reproduced every case**: the best agreed on 221,228 (37.5%). The structure of the
disagreement was then measured. The error of the image's S against the rounded value of r·sin θ was taken at r =
7FFFh, as a histogram over every angle and as three runs of 96 consecutive angles. That is evidence of the kind §1.3's
black-box characterisation allows: errors, not values.

- The error lies between -10 and +10 and is symmetric about zero over the whole turn.
- It resets at every multiple of 256 angle steps (the run from 12000 jumps from +7 to -1 at 12032 = 2F00h).
- Within a segment it grows, by about δ² sin θ₀ / 2 near 66°, where δ is the offset in the segment.
- Near 0° it is a constant -2 to -3.

That is the signature of a first-order Taylor step from a table at 256 points: sin(θ₀ + δ) ≈ sin θ₀ + δ·cos θ₀,
the derivative from a second table, with a floor's bias. **The widened family**, declared before it is graded:

- S[i] = Q(A·sin(2πi/N)) and D[i] = Q'(A·(2π/N)·cos(2πi/N)), two tables generated by formula, with
  sin θ = S[i] + P(f·D[i] / 2^15). Here i is θ's top log2(N) bits and f the rest, scaled to 15 bits.
- N: 128, 256 or 512.
- A: 2^15 saturated to 7FFFh, or 2^15 - 1.
- Q and Q': floor or half up, each.
- P: floor or half up.
- Tables direct, or quarter-wave mirrored.
- The product r·s: floor or half up.

That is 3·2^6 = 192 members, under 8 bits (this paragraph first said 384, a count of seven binary choices where there
are six), with cos θ = sin(θ + 2^14). No further widening follows. If no member agrees on every case, the DSP-1's sine
is a named loss under R4, and Triangle, Rotate and Polar become approximate.

### 40.3 The sine closed, Rotate chosen, and Polar widened once, in writing

**The sine is a named loss under R4, as §40.2 declared.** The 192 widened members graded over the same 589,824 cases.
The closest agreed on 250,863 (42.5%) and differed by up to ±10 at full radius: N = 256, amplitude 2^15 saturated,
both tables floored and quarter-mirrored, the step half up, the product floored. It resets at the right segments but
not to the chip's values. The DSP-1's sine is therefore not reproduced from documents. Its error pattern, a table at
256 points with a first-order step, is recorded as a structure, and no further member is tried. Triangle, Rotate and
Polar become **approximate**. The replacement uses that closest member, generated in `dsphle.rs` from its formula, and
the tables are checked word for word against `dsp1_tables.py`.

**Rotate's variants (§40.1) were graded with that sine.** The criterion is the share of cases within Triangle's
bound, a difference of 12, since exact agreement is impossible with an inexact sine. Over 65,536 seeded cases, one
member agreed in 65,496 (99.94%): the SNESdev matrix read literally with the row vector on the left, so
x2 = x·cos + y·sin and y2 = -x·sin + y·cos; the exact sum scaled once; wrap. The 40 left differ where the sum sits at
16-bit overflow. The counter-clockwise sense of SnesLab's prose agreed in 13 to 14 cases. So SnesLab's word describes
the angle's other sign, or the S-CPU's view of the screen.

**Polar's 16 members all failed**, and they fail with structure, not noise. With coordinates under 2^11, so that no
member overflows, the best agreed within 12 on 341 of 16,384 cases, and errors ran into the thousands. The order or
the assignment of the angles is wrong, not the arithmetic. **Polar's family is widened once**, with the reason that
SNESdev's own labels conflict: "Angle In: XYZ (I1, I2, I3)" against an equation that puts I1 in the matrix about Z.

- **The widened family**: every order of the three axis rotations (6), every assignment of I1, I2 and I3 to the axes
  (6), and each matrix as written or transposed (2).
- **Fixed by Rotate's choice**: the row vector on the left, the exact sum scaled once, wrap.

That is 72 members, about 6 bits. If none agrees within 64 on nearly every small-coordinate case, Polar is approximate
by SNESdev's literal reading.

### 40.4 Polar chosen, what was built, and the grade (measured 2026-10-04)

**Polar's widened family.** One member of the 72 agreed with the image within 64 on all 16,384 small-coordinate
cases, its largest difference 4, which is the sine's own error. At full range it agreed on 16,381 of 16,384, the rest
at 16-bit overflow. The member is the row vector times SNESdev's matrices **about Z by I1, then Y by I2, then X by I3**,
each as written. SNESdev's equation names the matrices about Y by I3, X by I2 and Z by I1, and its labels say "XYZ".
The order and the angles of both are wrong for this chip.

**Built** in `dsphle.rs`:

- **The sine.** It is §40.3's closest member, held to 16 bits. That last step was added after the first grade showed
  the step carrying -32768 past the limit at 270°: three cases off by 65,531 out of 2^20.
- **Triangle.** r·sin and r·cos, floored.
- **Rotate.** x·cos + y·sin and y·cos - x·sin, the sum scaled once.
- **Polar** by the member above.
- **Timing.** Each command's varying phases take their medians over 4,096 seeded cases: Triangle 32/4, Rotate 31/4,
  Polar 93/33 (§37.3 gives the ranges).
- **The tables** are generated at first use. `quarter_sine` sums the Taylor series in plain f64 arithmetic, so every
  platform builds the same words. The crate test `the_tables_equal_the_independent_generator` runs `dsp1_tables.py`
  and finds the 1,024 bytes identical. The table-literal guard passes.

**The grade** (`dsp_grade`, 2^20 seeded cases each, steady and jittered; the DSP-1 and DSP-1B alike):

| Code | Values differ | Largest difference in a result | Cases off by 16 or more | Latency differs |
|---|---|---|---|---|
| 04h Triangle | 687,963 (65.6%) | 12 | 0 | 461,392 |
| 0Ch Rotate | 944,712 | wraps at overflow | 99 | 491,948 |
| 1Ch Polar | 1,040,444 | wraps at overflow | 4,932 | 664,629 |

Of Rotate's cases, 99.4% are within 12, and of Polar's 99.5% within 15. The crate test
`the_replacements_rotations_stay_within_their_bounds` holds Triangle and Rotate to 12, and Polar to 8 on small
coordinates, against both images.

**firmwarecheck** (`dsp_oracle tables`, then `dsp1_tables.py` for the formula image, byte-identical to it), against
each image's data half:

| Run | DSP-1B | DSP-1 |
|---|---|---|
| plain | 3 of 1,024 equal at the same offset (0.3%); two common runs of 256 bytes, the sine table's first 128 words at original offset 556 and its words 64-191 at 812; FAIL by the plain thresholds | the same at 560 and 816 |
| `--forced` | 2 formula runs; formula coverage 512 of 2,048 bytes (25.0%); residual 0; PASS | the same |

**What the counts say.** The plain FAIL is the expected outcome of success that plan §5.3 describes. The two 256-byte
runs are the generated sine table found in the program's data, entry for entry, and every byte of them is
formula-forced. The derivative table appears nowhere. **The DSP-1's sine table is therefore this formula, and the
step between its entries is what §40.3 did not find.** That locates the remaining difference in the step's arithmetic,
not the table. It is a finding for a later step's decision, not a further widening here.

**P11 holds for this table**: the residual is zero, and the coverage, 25%, is under 40%. **P2**: no member of either
declared family reproduces the sine. That is the prediction's "or by none", though the table itself is a formula's.

### 40.5 What is open

- **The sine's step.** §40.3 closed the family as declared. The table being the formula's, a second widening limited
  to the step's arithmetic (its scale, its rounding, the derivative's source) is the natural next measurement. It
  needs a decision first, since §5.2 allows one widening per family.
- **Rotate's and Polar's overflow.** The few cases at 16-bit overflow differ because the chip's arithmetic there is
  unknown. Once the sine is exact, their remaining variants can be graded exactly.
- **Latency.** The medians leave a third to two thirds of these commands' cases a few cycles off. A rule naming one
  input feature (§6.3), with the exact sine, would come next.

### 40.6 Amendment: a second widening for the sine's step, declared before it is graded

*Decided 2026-10-04.* **The rule change.** Plan §5.2 allows one widening per family. One extra widening is allowed
where firmwarecheck proves the table itself formula-exact, and only for the arithmetic left between its entries. §40.4
gives that proof for the DSP-1. The generated sine table appears in both programs' data, all of it formula-forced. It
appears twice: as the first half-turn, and from a quarter-turn on, which is the cosine at the same 128 points. So the
derivative the step needs is already one of the table's own entries, S[i + 64]. No new table is admitted and no entry
is patched. If no member below reproduces the chip on every case, the sine stays a named loss, with no third widening.

**Fixed, from §40.3's closest member and the firmwarecheck proof:**

- S[i] = floor(2^15·sin(2πi/256)), quarter-mirrored and saturated;
- i is θ's top 8 bits, and f = (θ mod 256)·2^7 is the rest as a 15-bit fraction;
- sin θ = S[i] + step, held to 16 bits, and cos θ = sin(θ + 2^14);
- Triangle's product r·s is floored.

**The step, step ≈ f·S[i + 64]·(2π/256), and its open arithmetic:**

- **The constant K**, 2π/256 in fixed point: floor(2^15·2π/256) = 804, or floor and half up of 2^16·2π/256 = 1608
  and 1609, with the shift p = 15 or 16 to match. That is 3 values, counted as 4 (2 bits).
- **The term order** (1 bit): (f·C » 15)·K » p, or f·(C·K » p) » 15, with C = S[i + 64].
- **The first shift's rounding** (1 bit): floor, or half up.
- **The second shift's rounding** (1 bit): floor, or half up.
- **The sign handling** (1 bit): signed arithmetic throughout, or the magnitude of C carried and its sign applied
  after.

That is 64 members, 6 bits. They are graded as §40.2's were, over every angle at radii 7FFFh, 4000h, 1, 8000h and
FFFFh, and 2^18 seeded pairs.

### 40.7 The second widening graded: the sine stays a named loss (measured 2026-10-04)

`DSP_SINE_STEP=1 dsp_sine` graded §40.6's 64 members over 589,824 cases on each image, with the same result on both.
**No member reproduced every case.**

- The closest is K = 804 at p = 15, the order (f·|C| » 15)·K » 15, both shifts floored, and the sign of C applied
  after. It agreed on 281,052 cases (47.7%) and came within 1 on 427,366 (72.5%).
- Members differing only in K's representation tie, since 804, 1608 and 1609 give the same steps here.

**Under §40.6's condition the sine remains a named loss, and no third widening follows.**

- **What is established.** The DSP-1's sine table is the documented formula, as is its cosine table, and the chip
  steps from S[i] using S[i + 64].
- **What is not.** The arithmetic of that step lies outside both declared families. It may be a second-order term,
  another constant, or a fixed-point sequence that no document describes.
- **Triangle, Rotate and Polar stay approximate**, as §40.4 graded them. With the closest step member in place of the
  derivative table, Triangle's exact share rose from 34.4% to 39.6% of 2^20 cases, with the same largest difference
  of 12, and Rotate and Polar moved as little. The change was measured and not kept: it would alter the generated
  tables and their check for no change of bound.
- **The re-grade of condition 4 does not arise.**

## 41. Stage 8: the parity gates (2026-10-04)

`VenusRT_Plan.md` §7 lists eight conditions, G1 to G8, that must all hold before VenusRT becomes Mistress's SNES
default in place of C# Venus. This section runs each in full on the branch's head and records a verdict: **met**,
**met with named exceptions**, or **not met**. No core behaviour was changed; the step's commits are a harness option
for G1, a test fix for G7 and the goldens test for G5. What was found is listed as blockers (§41.10), each with a
proposed fix and its effort, for a following step to take up after review.

The firmware policy (`EmuSen_Firmware.md` §0) changed what two of the gates mean after the plan was written. The plan
assumed the player's own images (§9, Q2), and the policy now makes VenusRT's open replacements the default path: its
SPC700 boot program (§35) and the NEC DSP replacement, whose frame exists (§38) and whose commands are being built on
another branch. Each gate below is therefore read twice where firmware can matter: with no firmware folder, and with
the tester's own images.

### 41.1 What was run, and what was added to run it

- **The corpus runner** (`VenusRtTestRomRunnerTests.The_corpus_reproduces_the_recorded_baseline`), with C# Venus,
  Mesen through the probe and VenusRT as its third engine, over the 295 ROMs of the manifest, twice:
  - *no firmware*: `EMUSEN_VENUSRT_NO_FIRMWARE=1`, which withholds the NEC DSP image from VenusRT, and no boot image;
  - *the tester's images*: the DSP images (the copies in the corpus's `firmware/` folder, byte-identical to
    `home/Firmware`'s split pairs, checked file by file) and `EMUSEN_VENUSRT_BOOT` naming the cached SPC700 image.

  Both options are new in `VenusRtSnesEngine`. VenusRT then receives the boot image as file 1, through
  `VenusMachine`'s new `boot` parameter. Mesen is always given the DSP image, because it is the oracle. The test
  `The_runner_starts_VenusRT_from_a_boot_image_when_given_one` checks the option on a synthetic image. Both runs
  reproduced the committed baseline for C# Venus and Mesen in every cell; each took about 21 minutes on six threads,
  with the desktop's load average between 4 and 19 from other work.
- **The single-step suites** through the crate's own harness (`cargo test --release`, the corpus set):
  `the_cpu_through_the_whole_suite` and `the_spc700_suite_with_cycles`.
- **The crate's tests** with the corpus and the tester's DSP images: 108 of 108.
- **WiseMan's VenusRT, adapter, Mistress-engine, debugger and fit-audit tests**, with the opt-in game sets: 104
  tests, 102 passing at first and the other two after the corrections of §41.7.
- **The goldens**, a new opt-in test (§41.6).
- **The speed bench** through `ICore` on the adapter, at a 33% CPU quota and unquota'd, under the timing lock (§41.8).
- **The clone check** of every VenusRT source against Mesen's SNES sources and against the DSP comparison set of
  `VenusRT_DspHle.md` §5.4; `firmwarecheck.py` on the boot program; a read of the disputes log end to end (§41.9).

### 41.2 The gates

| Gate | Verdict | Evidence | Exceptions, and what keeps it from being met |
|---|---|---|---|
| G1, the test ROMs | **not met** | With the tester's images, 108 of the 117 self-grading ROMs Mesen passes; with none, 107. All 81 C# Venus passes pass in both runs, and VenusRT passes none that Mesen fails (§41.3) | Nine Mesen passes fail in both runs. None has a ruling against Mesen: D-24, D-26 and D-14 are open, and four have no entry. Without images, `spc_smp` fails as well; it reads the console's boot ROM (§41.3) |
| G2, single-step | **met with named exceptions** | 65816: 5,079,735 of 5,120,000 with cycle lists; SPC700: 256,000 of 256,000 (§41.4) | 40,265 cases, every one named and pinned by the test that requires exactly these to fail: D-1 (265 emulation-mode (d,X) cases), D-2 (the 40,000 cases of `7c` and `fc` on two cycles' VDA/VPA) and D-3 (43 JSR (a,X) cases in `fc.e`, inside D-2's files) |
| G3, pictures | **not met** | 151 picture-graded ROMs stand still in Mesen. VenusRT's VRAM equals Mesen's on 141, and VRAM and picture both on 118 (§41.5) | Of the 33 that differ, 22 are named: plan §5.1's losses with D-12 (13), D-6 (5), D-27 (2), D-16 (1), and the Cx4, outside parity (1). Eleven have no entry |
| G4, the frontend | **not met** | Every plan §4 item is present, or written down as a loss, except two. `.srm` round trips on ten games, one or more of each kind. The fit audit passes 73 of 73. Mistress's engine, rewind and firmware tests pass (§41.7) | A C# Venus state is refused, but the message does not name the engine that made it. No output-rate setting exists, against plan §4.6. The debugger's console views are a recorded loss (§36.3) |
| G5, the goldens | **not met** | 29 of 36 anchors equal to a Mesen frame within ±30 frames (§41.6) | Three are D-7. Two are Super Mario RPG's attract mode running 40 frames ahead, equal outside the window (the drift of D-6 and D-37). Two have no entry: Yoshi's Island's stork, and NHL '94's puck |
| G6, speed | **not met** | At 33% quota, every bench game's mean is within 13.3 ms and every p99 but one within 16.6 ms (§41.8) | Super Mario RPG's p99 is 18.69 and 19.26 ms. The handheld was unreachable all day, so the battery run is owed |
| G7, the debugger | **met** | §36.2's 19 claims, 19 of 19. Every table armed, equal to plain on all nine bench games, with a coprocessor halt on each of the five chip games (§41.7) | None. One test fix was needed to include Yoshi's Island |
| G8, the clean room | **met with named exceptions** | The mechanical check finds one pair over the threshold and two shared tables. All three are the S-DSP's documented tables (fullsnes's Gaussian table and anomie's counter rates). Nothing is shared with any DSP comparison source. `firmwarecheck` passes. The log was read end to end (§41.9) | Five entries whose reasoning is weaker than stated (D-5, D-6's SPC700 lead, D-11, D-19, D-27). Each is to be amended by a dispute step, not by code |

**The default does not flip at this stage.** One gate is met, two are met with named exceptions, and five are not met.
The blockers are §41.10. Most are small, and they cluster: HDMA timing, alone or with other causes, accounts for
eight ROMs across G1 and G3.

### 41.3 G1, the test ROMs (measured 2026-10-04)

| Suite | ROMs | C# Venus passes | Mesen passes | VenusRT, the tester's images | VenusRT, no firmware |
|---|---|---|---|---|---|
| gilyon `cputest` | 2 | 0 | 2 | 2 | 2 |
| gilyon `spctest` | 2 | 2 | 2 | 2 | 2 |
| PeterLemon CPU, SPC700, GSU | 61 | 61 | 61 | 61 | 61 |
| blargg SPC (`spc_smp`, `spc_timer`, `spc_mem_access_times`) | 3 | 0 | 3 | 3 | 2 |
| blargg `spc_dsp6` | 1 | 0 | 1 | 0 | 0 |
| blargg 2010 | 15 | 3 | 10 | 9 | 9 |
| ADC/SBC | 6 | 6 | 6 | 6 | 6 |
| multiply/divide | 7 | 0 | 5 | 5 | 5 |
| absindx SA-1 | 2 | 0 | 1 | 1 | 1 |
| higan collection, other (byuu's backdrop-graded ROMs among them) | 56 | 9 | 26 | 19 | 19 |
| Cx4, undisbeliever, the 240p suite | 140 | 0 | 0 | 0 | 0 |
| **All** | **295** | **81** | **117** | **108** | **107** |

The record predicted both totals before the runs. §34's corpus gave 108 with the console's program and §35.4's gave
107 with VenusRT's own, and both held, cell for cell, where nothing has changed since: pictures equal to Mesen's on
230 and 229 rows, VRAM on 256 and 255. The two runs differ on nine rows, all of them SPC700-side:
- `spc_smp` passes only with the image;
- `spc_dsp6` fails in both, at different places;
- six timing printers shift by a count (`test_speed`, the four `test_timer_speed` ROMs, lidnariq's `smpspeed`);
- undisbeliever's `ipl-speed-test`, which times the boot program itself.

Withholding the DSP image changes no row, because no corpus ROM has a NEC DSP. **The DSP replacement is therefore
not graded by G1 at all.** Its grade is G5's, and §37's oracle.

**The nine Mesen passes VenusRT fails with the tester's images**, and what the log says of each:

| ROM | Where it stands | Logged |
|---|---|---|
| `spc_dsp6`, in order | the backdrop red. Run singly, 106 of 111 tests pass and 5 hang (§38.4) | §23.8, §35.5; no entry for the five |
| `test_timer_stop2` | prints 00 against Mesen's 04 | D-26, open, the referee read |
| `blobs/test_irqb` | case 5, WMDATA read once where twice is expected | D-24, open |
| `snestest_082506/test_hdma`, `blobs/test_hdma` | the init's OPHCT two dots late | D-14, open for the timing |
| `blobs/test_hdmasync`, `blobs/test_hdmatiming` | HDMA start | §20.5's named list; **no entry** |
| `blobs/irq`, `blobs/nmi` | tests `$2D` and `$1E` (§17.4) | **never read; no entry** |

**How G1 reads under the firmware policy (proposed).** G1 is to hold twice:
1. *With the player's images*, as the plan wrote it: every Mesen pass, less rulings against Mesen.
2. *With no images*, the same set less the ROMs that cannot pass without the original firmware's bytes or programs.

The second list is fixed by what a ROM reads, not by what fails, and today it holds one ROM: **blargg's `spc_smp`**,
whose "CPU/verify IPL ROM" test compares all 64 bytes at $FFC0-$FFFF (§35.3). Two other kinds were considered:
- *ROMs sensitive to the boot program's timing.* `spc_dsp6`'s "Misc/brr addr wrap-around" and "Order/voice 0 noise"
  pass singly only with the image (§38.4). They do not read the bytes. §35.5 argued that they expose a sensitivity of
  VenusRT's, not a fault in the open program's protocol. They are therefore not exempt; they stay `spc_dsp6`'s
  blockers on both runs.
- *DSP commands not yet replaced.* No corpus ROM reaches a NEC DSP, so the list is empty. When the replacement's
  commands land, a test ROM that issues a command the replacement leaves unexact would join this list for the
  no-images run only, and the replacement's own oracle (§37) grades it instead.

The timing printers (`test_speed` and the like) print without a verdict and are G3's, not G1's.

### 41.4 G2, the single-step suites (measured 2026-10-04)

Graded with cycle lists, as the plan requires, through `singlestep/cpu65816.rs` and `singlestep/spc700_cpu.rs`.

- **65816, both modes:** 5,079,735 of 5,120,000 cases pass registers, memory and every cycle. The 40,265 that fail
  are exactly those the test's `disputed` table names, file by file, and it asserts that each named case fails and
  every other case of its file passes:
  - D-1: 265 cases of `01`, `21`, `41`, `61`, `81`, `a1`, `c1` and `e1` in emulation mode, the (d,X) pointer at a
    page's last byte, where gilyon's console-verified ROM settles the wrap against the suite's model;
  - D-2: all 40,000 cases of `7c` and `fc`, which grade on VDA/VPA exchanged on the two pointer reads;
  - D-3: within `fc.e`, the 43 cases at S=$0100.

  MVN and MVP's 39,989 capped cases are graded on their 100 recorded cycles (§2.3).
- **SPC700:** 256,000 of 256,000, with cycles. The I/O-page exception the plan anticipated was never needed.

G2 is met with named exceptions. D-2's 40,000 cases differ in two pin values that the SNES's bus cannot observe
(argued, D-2). They are named exceptions by the gate's own wording, not by a ruling against the suite.

### 41.5 G3, the pictures (measured 2026-10-04)

**The set.** The plan counted 172 standing picture ROMs over its own grading; the runner counted 176 at stage 0. Since
§16.2, 25 of those grade themselves by the backdrop and are G1's. **The G3 set is therefore 151 ROMs**: Mesen gives no
self-graded verdict, and Mesen's VRAM is equal between frames 1800 and 3600. 149 are visual and 2 print "Done".
The two firmware runs agree on all 151 but the two blargg printers' counts.

| Result | ROMs |
|---|---|
| VRAM and picture equal to Mesen's at frame 3600 | 118 |
| VRAM equal, picture differs | 23 |
| VRAM differs | 10 |

**The 33 that differ, by cause** (each picture was drawn beside Mesen's and the difference mapped):

| Cause | ROMs | Status |
|---|---|---|
| Plan §5.1's named losses: the INIDISP early-read glitch and writes during active display (with D-12 for OAM): undisbeliever's `hdma-2100-glitch-2ch-0a`, `-81`, `hdma-21ff-2100-glitch`, `inidisp_brightness_delay`, `inidisp_enable_display_mid_frame` (each in both builds), `inidisp_forgot_to_force_blank` (both builds) and `_2` | 13 | named; §9's Q10 keeps them unmodelled until a golden needs one, and none does (§41.6) |
| D-6, counters printed: Sour's `timing_test` (2), `test_dmatiming/demo`, `reset-position-test`, `blip-autojoy-timing-test` | 5 | open entry; a console reading of `reset-position-test` settles it |
| D-27, blargg's ungraded printers `test_speed` and `test_timer_speed3`, within a count of Mesen's | 2 | settled entry; the counts are not graded |
| D-16, `ppubusact`'s mode 6 band, 86 half-pixels | 1 | open entry |
| `cx4test`, a Cx4 cartridge | 1 | outside parity (plan §4.2, §9 Q8) |
| **No entry:** `hdmaen_latch_test` and `_2` (each in both builds: Mesen shows red stripes HDMA writes, VenusRT none) | 4 | blocker |
| **No entry:** `hdma_midframe/demo` (red backdrop against Mesen's black with HDMA lines; CGRAM 3 bytes), `hdma-double-buffered-parallax` (its HDMA columns one step apart), `test_hello` (the HDMA shear of "HELLO" a pixel apart on some lines), Motive's `HblankEmuTest` (the H-blank text Mesen overwrites) | 4 | blocker; HDMA or mid-line timing, as G1's two HDMA ROMs |
| **No entry:** `test_noise` (one grey level a channel brighter over the screen), `hvdma` (one line, 36 pixels, HDMA to VRAM in H-blank), `wrmpyb-in-flight` (the multiplier's products read in flight: digits of 3 to 5 cycles differ) | 3 | blocker |

### 41.6 G5, the goldens (measured 2026-10-04)

**The games** were chosen before VenusRT was run on any of them:
- *One per coprocessor:* Super Mario RPG (SA-1), Yoshi's Island (GSU-2), Super Mario Kart (DSP-1B), F1 ROC II (ST010)
  and Metal Combat (OBC1).
- *The tricky-to-emulate list:* the SNESdev wiki's "Tricky-to-emulate games", fetched 2026-10-04 into the corpus's
  `docs/snesdev/`. "The list's first entries" was read as the first game of each of its rows in order, skipping those
  not in the library. That gives:
  - Captain America and the Avengers (open bus);
  - ActRaiser (BRK/COP);
  - Super Mario World (ORA [d]);
  - Hook (VRAM writes during display);
  - Breath of Fire (VRAM reads);
  - Axelay (offset-per-tile);
  - NHL '94 (mode 7 scroll latch).

  Kick Off, Super Famista 5 and The Atlas are not in the library.
- *The anchors* are three per game, each chosen from Mesen's pictures alone: contact sheets every 60 frames to 3600,
  and for three games every 30 frames with a pad script. They are a logo or text page, an attract or title scene, and
  a scene past a menu where one is reachable: Super Mario World's overworld map, ActRaiser's sky palace, Breath of
  Fire's name entry.
- *The window* is fixed at ±30 frames of Mesen's run, since the drift of §27.4 is up to ten frames.

The test is `VenusRtGoldenTests.The_goldens_against_Mesen`. It is opt-in: `EMUSEN_VENUSRT_GOLDENS` names the library,
or `library` for `AppSettings.RomDirectory`. It finds each game by MD5 and copies it to a scratch folder; the library
is only read. It runs VenusRT to the anchors, own boot program, DSP image from the corpus or `home/Firmware` and a DSP
game skipped without one, then Mesen over the window, and passes an anchor whose picture equals one of Mesen's frames
there. What is committed is each ROM's MD5, file name, anchor frames, scene descriptions, the recorded outcome and an
FNV-1a hash of VenusRT's picture. Without the probe, the test still checks every hash.

| Game | Anchor 1 | Anchor 2 | Anchor 3 |
|---|---|---|---|
| Super Mario RPG | 300 equal | 1200 differs: drift | 2400 differs: drift |
| Yoshi's Island | 600 equal (lag +1) | 1800 differs: the stork | 3000 equal (lag -10) |
| Super Mario Kart | 600 equal (-11) | 2400 equal (-7) | 3300 equal (-11) |
| F1 ROC II | 600 equal (-1) | 1500 equal (-1) | 3300 equal (-1) |
| Metal Combat | 600 equal (-6) | 1500 equal (-5) | 3000 equal (-5) |
| Captain America | 600 equal | 1500 equal | 2100 equal |
| ActRaiser | 600 differs: D-7 | 900 equal | 1400 equal (-1) |
| Super Mario World | 150 equal | 1200 equal | 2400 equal (the overworld map) |
| Hook | 300 equal | 600 equal | 1500 equal (+11) |
| Breath of Fire | 700 equal | 1800 equal | 2700 equal |
| Axelay | 600 equal | 1500 differs: D-7 | 3000 differs: D-7 |
| NHL '94 | 600 equal | 1500 equal | 2200 differs: the puck |

- **D-7, three anchors.** ActRaiser at 600 and Axelay at 1500 and 3000 run with master brightness 10 or 11. Every
  differing pixel is one level brighter in one or more channels in VenusRT and never darker. That is D-7's signature:
  fullsnes's c×(N+1)/16 rounded down against Mesen's one lower. The entry is open, argued from the only document with a
  formula.
- **The drift, two anchors.** Super Mario RPG's attract scenes at 1200 and 2400 are equal in every pixel to Mesen's
  frames 1240 and 2442, outside the window. Its attract mode runs 40 to 42 frames ahead of Mesen's. This is the
  start-up drift of §27.4 and §29, where Super Mario RPG first parts at a port poll in frame 1. It is a timing
  difference of the S-CPU's and SPC700's handshake (D-37, D-6), not a picture rule. The window was not widened after
  the measurement; the two are recorded as differing, with the lag measured.
- **No entry, two anchors.**
  - *Yoshi's Island at 1800.* The stork alone differs, 1,426 pixels at the best lag of -10. Over ±100 frames no
    frame does better, so it is not a lag. Candidates: the GSU's costs (D-35) desynchronising its animation from the
    scroll, or a sprite rule.
  - *NHL '94 at 2200.* The spinning puck in the credits differs by 11 pixels at lag -1, and no frame within ±100 is
    equal.

  Each needs a look before G5 can be met.
- **The DSP replacement** was not used here: Super Mario Kart and F1 ROC II ran on the tester's images, all six
  anchors equal. The replacement's commands are being built on another branch and are not yet graded by any golden.

### 41.7 G4, the frontend, and G7, the debugger (measured 2026-10-04)

**Plan §4, item by item**, on the adapter as Mistress runs it:

| §4 item | State |
|---|---|
| 4.1 `ICore`, two pads of twelve buttons, `SkipRendering` state-neutral, halts with the processor that stopped (`ICoprocessorHalt`), `IStateFormat` (`VNRT`, version 18), `IFrameProfiler` | present. `CoreName` is `VenusRT`, not `SNES`; nothing keys on it but the state record (below) |
| 4.1 `ICoprocessorLoad` (the SA-1's executed clocks), `ITraceFlushable` (Pharaoh's CPU trace) | not on the generic adapter: the first is the hardware-load view of §36.3's recorded loss, the second is stage 9's Pharaoh decision |
| 4.1 sound at the player's rate | **not built**: the core gives 32 kHz and the audio device is opened at the core's rate, so sound plays, but the player's sample-rate setting does not reach VenusRT (plan §4.6, §5.3's resampler) |
| 4.2 boards, coprocessors, firmware | present (§32, §34, §38); every firmware entry optional |
| 4.3 battery saves | present: ten games both ways, `A_battery_save_crosses_between_venus_and_venusrt_both_ways`: LoROM, HiROM, DSP-1B, DSP-2, ST010, two SA-1, two GSU (Yoshi's Island, Stunt Race FX), OBC1 |
| 4.4 cheats | present, from the SNES system pack (§32) |
| 4.5 states and rewind | **the refusal does not name the engine.** A C# Venus state offered to VenusRT through `ICore` is refused with "VenusRT (Rust) refused the state: not this core's state.", the machine unchanged. Mistress then starts the game afresh with that line, and adds the record's "saved by …, SNES state version 3" only where a `.resume.json` exists. The plan asked for the engine that made it, which the magic `SNES` identifies. Rewind: present (`VenusRtEngineTests`) |
| 4.6 settings | the Engine row present; the output-rate key not built (above) |
| 4.7 the debugger | present for what v1 carries (G7); the console views are a recorded loss (§36.3): video, APU and coprocessor register providers with history, sprites, palettes, audio channels with their mutes (VenusRT claims no `MUTES`), hardware load, interrupt vectors, DMA channels and the DMA log, register flow, freezes and access counters |
| 4.8 Mistress | the Engine row, resume, rewind and fast-forward, no firmware prompt, pacing from the exact frame rate, the fit audit 73 of 73 (the SNES engine row at 1280x800 and 1920x1200 among them). Pharaoh and Hotaru are stage 9's |

**G7.** `VenusRtDebugTests` passes all 19 of §36.2's claims. `Every_table_armed_with_nothing_to_hit_gives_the_plain_run`
ran over the nine bench games instead of §36.2's four. Eight passed, and Yoshi's Island failed one check: no halt on
the GSU, because its GSU does not run between frames 120 and 180. The machine was not at fault (armed equal to plain
for all 600 frames). The check now waits up to 600 frames for a chip that starts later, a test fix in its own commit.
With it, all nine pass: armed equal to plain over 600 frames each, and halts on the DSP-1 and DSP-1B at $00000C, the
SA-1 at $C08171 and the GSU at $01B301 (Star Fox) and $08BD16 (Yoshi's Island). The battery test's first game set
included Star Fox, which has no battery; it was replaced by Stunt Race FX. That is a correction of the input, not of
the test.

### 41.8 G6, speed (measured 2026-10-04)

A throwaway bench outside the repository, `venusrtbench` in the corpus folder, does what `venusbench` does for C#
Venus through `CoreFactory` and `ICore`, but names the engine. It boots 1,200 frames tapping Start and A, then times
3,000 frames with Right held, fetching the picture and draining the sound each frame. Each run was under
`flock ~/.cache/emusen/probe/timing.lock` and `systemd-run --scope -p CPUQuota=33% -p CPUQuotaPeriodSec=5ms`, the
plan's weak-laptop proxy. The load average was 12.3 for round 1 and 3.4 to 3.9 after.

| Game | VenusRT round 1, mean (p99) | Round 2, mean (p99) | C# Venus, mean (p99) | VenusRT unquota'd, mean (p99) |
|---|---|---|---|---|
| Super Mario World | 9.94 (14.88) | 7.42 (15.68) | 7.37 (14.56) | 2.02 (2.58) |
| Super Metroid | 9.21 (15.14) | 6.99 (13.83) | 7.44 (13.76) | 2.02 (2.62) |
| Donkey Kong Country | 7.74 (14.33) | 6.52 (10.89) | 7.54 (14.46) | 1.88 (2.63) |
| A Link to the Past | 7.94 (14.64) | 7.72 (14.37) | 7.72 (18.69) | 2.25 (2.48) |
| Yoshi's Island (GSU-2) | 8.55 (13.75) | 9.10 (14.37) | 10.85 (15.65) | 2.54 (2.90) |
| Star Fox (GSU) | 7.50 (13.76) | 8.11 (15.28) | 3.05 (8.14) | 2.19 (2.84) |
| Super Mario RPG (SA-1) | 11.96 (**18.69**) | 12.38 (**19.26**) | 15.23 (20.08) | 3.33 (3.88) |
| Pilotwings (DSP-1, image) | 8.90 (14.82) | 8.96 (14.84) | 9.45 (15.42) | 2.55 (3.41) |
| Super Mario Kart (DSP-1B, image) | 9.75 (14.94) | 9.82 (14.84) | 3.68 (8.88) | 2.86 (3.18) |
| Pilotwings, replacement | 7.17 (13.28) | | | |
| Super Mario Kart, replacement | 8.45 (13.56) | | | |

- **The budget**, a mean of at most 13.3 ms and a p99 of at most 16.6 ms: every mean is met, the largest 12.38 ms on
  Super Mario RPG. Every p99 is met except Super Mario RPG's, 18.69 and 19.26 ms in the two rounds. C# Venus misses
  both the mean and the p99 on that game, and A Link to the Past's p99. Star Fox and Super Mario Kart are 3 ms on C# Venus because
  its script does not reach 3D play there (§5.5's note); VenusRT's state hashes say it reaches other scenes, so those
  two rows do not compare engines.
- **Unquota'd**, the desktop means are 1.9 to 3.3 ms against plan P1's 2.8 and P2's 3.8, with the picture fetched and
  the sound drained through the adapter.
- **The handheld half is owed.** `deck@10.1.1.205` did not answer from 10:37 to 11:55: ping at 100% loss, ARP
  incomplete, port 22 unreachable twenty times, a minute apart. No session was opened and nothing was copied or
  started there. The bench is published self-contained in the corpus folder's `venusrtbench/pub/`, ready for a battery
  run.

### 41.9 G8, the clean room (measured 2026-10-04)

**The mechanical check.**
- *Command.* `clonecheck.py` (§4) of `src/` (40 files) and of `examples/` (25), with the vocabularies of fullsnes,
  anomie's six documents, the WDC datasheet, the DSP pages and the other Rust cores' `src/` folders.
- *Reference sets:*
  - Mesen's `Core/SNES`, at `b9fa69dd`;
  - the DSP comparison set of `VenusRT_DspHle.md` §5.4, fetched into the corpus folder's `clonecheck/refs/` with its
    provenance. It holds the HLE of bsnes 0.59 (dsp1–dsp4, st010, st011, as carried by beetle-bsnes); bsnes-plus's
    necdsp; snes9x's `dsp*.cpp` and `seta*.cpp`; MAME 0.140's `snesdsp1`–`4` and `snesst10`; current MAME's `upd`
    and `upd7725`; and the superfamicom wiki's ST010 code.
- *What could not be fetched as the plan names it.* bsnes-plus never carried the HLE modules; snes9x's ST010 is
  `seta010.cpp`; current MAME has no SNES DSP HLE. The substitutes are those listed.
- *Results.*
  - **Against Mesen**, one pair passes the threshold of 12 shared fingerprints: `apu/dsp.rs` with
    `DspInterpolation.h`, 115 shared. All of it is inside the `GAUSS` array, whose 512 values were checked one by
    one against fullsnes's Gauss table and are equal.
  - **The two shared tables** are that one and `RATES`, anomie's `counter_rates`, as §22.5 recorded.
  - **The largest pair under the threshold** is 7 fingerprints: `dsp.rs`'s register addresses, from fullsnes.
  - **The 28 shared names** are the hardware's plain names (`forcedblank`, `hdmainit`, `rombuffer`, `mode7extbg`
    and the like).
  - **Against every DSP comparison set**, no pair reaches 12, the largest being 5. The only table hit is `dsp.rs`'s
    Gaussian table again, on two arithmetic runs that the DSP sources also contain by coincidence. None of the DSP
    replacement's files (`dsphle.rs`, `dspengine.rs`, `dsporacle.rs`) shares a table or a pair.
  - **`examples/`** shares nothing with any set.
- *Not a finding.* §4.2's calibration says a restructured port escapes this tool, so "nothing found" bounds
  statement-order copying only.

**`firmwarecheck.py` on the boot program**, the 64 bytes extracted mechanically from `apu/boot.rs`: 6 of 64 equal at
the same offset (9.4%), the longest aligned run 2, the longest common run 4, twice. PASS, as §35.2.

**The disputes log, D-1 to D-38, read end to end** by a reader who wrote none of it. 33 entries hold as reasoned. Five
are weaker than they state, and none of the five calls for a code change before a dispute step:
- **D-5.** It is to be "settled when the goldens of stage 6 run it [Test Drive II] against Mesen". The goldens are
  this stage's, and Test Drive II is not among them. The check it promised never ran. It stays open, with its
  settling step owed: one run of Test Drive II against Mesen past its banks $40-$7D and $C0-$FF reads.
- **D-6, the SPC700's 150-clock lead.** It was adopted from `CPU.vhd`'s reset counter, `P65_RST_CNT`, as the
  console's rule. A counter in the FPGA's reset release is as likely to be that implementation's own sequencing as the
  console's. The referee's weight table rates the core's structure, not its reset glue. The lead should be recorded
  as the referee's choice, argued, not as a rule the referee states. D-6's counters half is unaffected and correctly
  open.
- **D-11.** fullsnes's narrower reload rule was kept "from its being the narrower statement and the one built first".
  Neither is evidence. anomie's timing document reports an observation of a console, the reload "on any 1->0"
  transition, which is the only hardware report either way. The reasoning should either rest on a test ROM or say
  that the built rule stands against the one console observation cited.
- **D-19.** The conclusion is "settled for the rule as built" because the referee agrees with anomie's sentence. But
  `Venus_Referee.md` §0 says the referee's PPU is written from anomie's and fullsnes's register documents, so its
  agreement with anomie is correlated, the caveat D-1 itself makes for the 65C816. The entry is better recorded as
  argued from one document and its correlated implementation, against Mesen's picture. The tricky-to-emulate list's
  "Jurassic Park: broken graphics during gameplay" under hi-res sub-screen math is a game that could arbitrate.
- **D-27.** The timer step is settled by blargg's graded ROMs and his `notes.txt`. The stretch of every SPC700 cycle
  under TEST bits 6-7, and bits 4-5 not slowing the CPU, rest on the counts Mesen prints for `test_speed`, an
  ungraded ROM, against fullsnes's text. That uses Mesen as the authority the order of recourse ranks last. The
  rule should be marked argued from Mesen's output, pending a console reading of `test_speed`.

P6's second half, that fewer than one entry in five needs Mesen's source, holds: no entry read Mesen's source.

### 41.10 What blocks the default flip

Effort is in the plan's steps of about three hours. A **dispute step** produces one entry and no code, and a writer's
step implements it.

| # | Gate | Blocker | Proposed fix | Effort |
|---|---|---|---|---|
| 1 | G1, G3 | HDMA timing: `test_hdmasync`, `test_hdmatiming`, both `test_hdma` (D-14's two dots), and on the G3 side `hdmaen_latch_test` and `_2` (4 ROMs), `hdma_midframe/demo`, `hdma-double-buffered-parallax`, `test_hello`, `HblankEmuTest` | One dispute step on HDMA's start, init and HDMAEN latch timing: the ROMs' own headers first (byuu's and undisbeliever's state console results), then the referee's DMA/HDMA machine (real support, `Venus_Referee.md` §0); then a writer's step implementing it, judged on all twelve ROMs at once | 2–3 |
| 2 | G1 | `blobs/irq` and `blobs/nmi`, never read (§17.4) | Read each failing test's disassembly (`blobs/disassembly/`) and its expectation, open an entry each, then implement | 1–2 |
| 3 | G1 | `test_irqb` case 5 (D-24) | The referee's 65C816 interrupt sequence in the logged step D-24 already names | 1 |
| 4 | G1 | `test_timer_stop2` (D-26) | The SPC700 program it uploads, disassembled with VenusRT's own disassembler (§1.2 allows a test's code), to see what it toggles; then the referee's TEST gating again | 1 |
| 5 | G1 | `spc_dsp6` in order: 5 tests hang singly with the image, 6 hang and 1 fails without | The S-DSP step §35.5 left: the five hangs singly first, then the in-order run to 18,000 frames | 2 |
| 6 | G3 | `test_noise`, `hvdma`, `wrmpyb-in-flight` | `test_noise`: one level brighter at full brightness, so not D-7; colour math and the fixed colour first. `hvdma`: the one line, with its README's console description. `wrmpyb-in-flight`: the multiplier's intermediate products, a plan §2.2 thin area, from the ROM's own console notes. A dispute step for the three | 1–2 |
| 7 | G4 | A C# Venus state's refusal does not name the engine | In the core's `last_error` for a foreign state with Venus's magic `SNES`: "a save state of Venus (C#), the C# SNES core; VenusRT cannot read it" (plan §4.5's sentence), with a WiseMan test that offers a real C# Venus state. Separately, Mistress's state record keeps `CoreName` for both engines (`SNES` against `VenusRT`) and should record the engine's display name | ≤1 |
| 8 | G4 | No output-rate setting; VenusRT always gives 32 kHz | Either the plan's create-time and run-time key with a resampler in the core (§5.3, exact at 32 kHz), or a decision that the host's device-rate path is parity and §4.6's item is withdrawn | 1, or a decision |
| 9 | G4 | The debugger's console views (§36.3) | Already a recorded loss on the generic target. The gate asks that the loss be written down, which §36.3 and this section do. Confirm it as accepted for the flip, or schedule the extension exports `EmuSen_CoreAPI.md` §15 Q10 anticipates | a decision |
| 10 | G5 | Yoshi's Island's stork at 1800, NHL '94's puck at 2200 | A look at each: OAM and the sprite's source buffer in WRAM against Mesen's dumps at the matched frame, then an entry or a fix | 1 |
| 11 | G5 | D-7, three anchors | A console capture of `RedSpace9BitHDMA` or any brightness-graded scene settles it. Without one, the gate's wording (dispute-logged) is met already, and the item is only a decision to accept D-7 as argued | a decision |
| 12 | G5 | Super Mario RPG's 40-frame attract lead | The S-CPU-side drift of §29 (D-37's ports exact, Mesen's handshake one poll apart, D-6's counters). Accept as dispute-logged timing, or measure it against a console recording of the attract mode | a decision |
| 13 | G6 | Super Mario RPG's p99 at 33% quota, 18.7–19.3 ms against 16.6 | Plan §9 Q3's first lever: the SA-1's catch-up bound (§5.4), made a setting before anything is loosened. Measure where the slow frames fall first (`ipsample.py`, not the safepoint-biased sampler) | 1–2 |
| 14 | G6 | The handheld on battery | The published bench, copied to the handheld's scratch folder and run under `systemd-run --user`, once it is reachable | ≤1 |
| 15 | G8 | D-5, D-6's lead, D-11, D-19, D-27 | One dispute-log amendment step: re-record each as argued where §41.9 says, struck through and followed by the new conclusion, as the log's rules require; run D-5's settling check on Test Drive II | 1 |

About 14 to 19 steps in all, with four decisions. Blockers 1, 5 and 13 are the large ones; 1 alone bears on twelve of
the twenty ROMs G1 and G3 leave open or unlogged.

### 41.11 Negative results

- The handheld could not be reached (§41.8).
- The DSP replacement is graded by no gate of §7. No test ROM has a NEC DSP, and the goldens ran on the tester's
  images, since the replacement's commands are still being built (§38.3). When they land, G5's two DSP games should
  be rerun without images.
- Three comparison sources named in `VenusRT_DspHle.md` §5.4 do not exist in the form named (§41.9).
- The window of ±30 frames, fixed before measuring, is too narrow for Super Mario RPG's attract mode. It was not
  widened after the fact.

### 41.12 Blocker 7: a C# Venus state refused by name (2026-10-04)

- **The core.** A state whose magic is C# Venus's `SNES` is still refused as foreign, with the machine unchanged. VenusRT
  now gives the refusal its own words through the v1 outbox's detail. Mistress shows "VenusRT (Rust) refused the
  state: it was saved by Venus (C#), the C# SNES engine, whose states VenusRT cannot read.", after its own "Could not
  resume, started from the beginning:". Any other foreign magic keeps the shared words. The words are produced only on
  the refused load, so the frame has no cost to measure.
- **The record.** Mistress's state record now stores the engine's catalog name instead of `ICore.CoreName`, which was
  `SNES` for C# Venus and `VenusRT` for VenusRT. Its version check applies only to a state the running engine wrote.
  The rule and the one legacy case it leaves are `EmuSen_Galaxia.md` §5.3b.
- **Tests.**
  - `VenusRtCoreAbiTests.A_venus_state_is_refused_naming_the_engine_that_made_it` takes a real C# Venus state of a
    `SyntheticRom` cartridge through `CoreEngine`: the message is as above, the machine unchanged, and another foreign
    magic gives the shared words.
  - `VenusRtEngineTests.A_states_record_names_its_engine_and_another_engines_version_is_not_compared` checks that
    the record says Venus (C#), that a VenusRT record at version 18 is offered to Venus rather than called a newer
    build's, and that the same engine's version 99 and a legacy record still are.
  - `IdentityAndCollectionsTests` expects the engine's name.
  - VenusRt, FileRecords, IdentityAndCollections, Resume and StateRecord filters: 137 passed, 1 skipped. The crate:
    112 of 112.

### 41.13 Blocker 15: the five entries amended, and D-5's check run (2026-10-04)

Each of §41.9's five entries gains a dated paragraph in `VenusRT_Disputes.md`, as the log's rules ask. The overturned
words are struck through and followed by the restated conclusion. No entry's implementation changes.

- **D-5.** Test Drive II's check, owed since stage 2, has now run. Both library dumps went through `ICore` and Mesen
  with one pad script into a race, and pictures at six frames from the opening to the race are equal at the same
  frame in every pixel. Settled as far as the scenes reach. Whether the game reads banks $40-$7D or $C0-$FF remains
  unobserved, since v1 reports no reads.
- **D-6.** The SPC700's 150-clock lead is restated as the referee's implementation choice, argued, not a console
  rule. The counters' half is unchanged and open.
- **D-11.** The built rule is fullsnes's, kept against the one reported console observation, which favours anomie's.
  Recorded as a choice that a cheap test ROM would settle.
- **D-19.** Restated as argued. The referee's PPU was written from the same documents, so its agreement with anomie
  is correlated. Jurassic Park's gameplay, on the tricky-to-emulate list under this cause, is the game that would
  arbitrate.
- **D-27.** The timer step stays settled. The per-cycle stretch and bits 4-5's effect are restated as argued from
  Mesen's printed counts, pending a console reading of `test_speed`.

The tool for D-5's comparison, `anchorshot` (VenusRT through `ICore` with a pad script) with `cmp.py` (Mesen over a
window of frames), is in the corpus folder beside `venusrtbench`, outside the repository.

### 41.14 Blocker 1: HDMA's counter and its run's timing (D-39, D-40), measured 2026-10-04

- **D-39, from the documents.** anomie's register document says a line decrements NTRLx as a whole byte, so $00
  becomes 127 lines with repeat and $80 127 lines without. VenusRT had kept bit 7. With the whole byte decremented,
  `hdma_midframe/demo` and `hdma-double-buffered-parallax` equal Mesen's picture in every pixel. The corpus run with
  D-39 alone changed no verdict.
- **D-40, from the referee.** `rtl/CPU.vhd`'s HDMA machine was read in a logged step, with `PPU.vhd`'s H-blank line.
  It was then measured against the two console tables in the corpus: `test_hdmasync`, 1,022 latched H positions
  recorded from a console, and `test_hdmatiming`, eight rows.

  | Change | `test_hdmasync`, latches off the console's | `test_hdmatiming` rows differing |
  |---|---|---|
  | before (18 clocks before the channels, per-channel interleave, run at the first cycle start past dot 278) | 1,020 of 1,022 (511 pairs at +1 to +3 dots) | 2 |
  | run start moved to clock 1,100, nothing else | 1,020 | 2 |
  | the ending step 8 clocks | **0** | 2 |
  | and transfers before counters, each byte at its step's end | 0 | 1 |
  | and the referee's start, sampling point swept 1,084-1,124 in twos | 0 at 1,098 and 1,100 only; elsewhere 42 to 1,020 | 1 at those two |

  The stage-count and sampling-point sweep is recorded in the corpus folder's `fit.txt`. It was a check of where the
  referee's structure agrees with the console, not a search for a structure. The single-stage and immediate variants
  also had sampling points that kept both tables, at 1,106 and 1,112, so the tables alone do not choose the referee's
  structure. `hdmaen_latch_test` does: with the referee's two stages at 1,100, both its ROMs equal Mesen's picture in
  every pixel. The single stage at 1,106 left 3,584 and 5,888 pixels differing, the immediate run at 1,112 6,144 and
  12,032.
- **The corpus, with the tester's images** (against §41.3's run):
  - **111 of Mesen's 117 self-grading passes** (108 before). Newly passing: `test_hdmasync`, `blobs/test_hdma`,
    `snestest_082506/test_hdma`. Every C# Venus pass still passes, and no verdict got worse.
  - **Pictures** equal to Mesen's on the two `hdmaen_latch_test` ROMs (each in two builds), `hdma_midframe/demo` and
    `hdma-double-buffered-parallax`. Several other pictures are closer to Mesen's: Sour's two timing tests, VitorVilela7's
    speed test, `dma-ends-hdma-start-1-ch`.
  - **One picture is further from Mesen's:** `hvdma`, 252 pixels against 36. Line 1, which Mesen shows in forced
    blank, is drawn in VenusRT. Its eight channels' burst now ends before the line begins. The ROM's console
    photograph cannot resolve one line, so this is recorded and not judged.
- **Skip versus draw**: the nine bench games, 600 frames each, the same.
- **The cost**, `frame_cost` best of three over 1,200 frames, before and after in turn under the timing lock at load
  1.3:

  | Game | Before | After |
  |---|---|---|
  | Super Mario World | 2.13-2.18 | 2.10 |
  | A Link to the Past | 1.91 | 1.88 |
  | Super Mario RPG | 2.80 | 2.78 |
  | Yoshi's Island | 1.75 | 1.82-1.83 |

  The 4% on Yoshi's Island is the slow path the bus now takes from clock 1,097 of every visible line, so that the
  run's two stages see each cycle.
- **The state** is version 20. The run's stage and its channels travel in it. Versions 17 to 19 still load, with no
  run pending.
- **Left open**: `test_hdmatiming`'s test 2, one dot (D-40). `blobs/test_hdmatiming` therefore still fails.

### 41.15 Blocker 2: `blobs/irq` and `blobs/nmi` were D-6, settled by their console logs (2026-10-04)

The two ROMs were never read before this step. Each logs 1,024 records of NMI or IRQ timing from power-on into SRAM,
and compares them with a log captured from a console, kept in its own image. `nmi.smc`'s source says so. The stage 2
reading of their failures as tests `$1E` and `$2D` was their first SRAM byte. A scratch tool moved the power-on
position through a whole line and graded each position against the console's log.
- **Both logs match at the offsets 28 + 52k and 29 + 52k, and nowhere else.** At 0 they do not match (961 of
  `nmi.smc`'s records differ).
- **`test_hdmasync`'s console table keeps 28 + 52k**, which contains D-6's Mesen-measured 132.
- **D-6's counters half is therefore settled** at 132 master clocks into line 0, the class from the console and the
  member from Mesen (`VenusRT_Disputes.md` D-6, 2026-10-04). Built as the master clock's zero at line clock 132, with
  the DMA clock's 8-clock steps counted from line 0 of the first frame, and the SPC700's 150-clock lead kept against
  the CPU.

| Check | Result |
|---|---|
| The corpus, the tester's images | **113 of Mesen's 117** (111). `blobs/irq` and `blobs/nmi` pass. No verdict worse. VRAM equal on 257 rows, pictures on 242 (238) |
| Pictures closer to Mesen's | Sour's two timing tests (448 and 419 pixels from 589 and 610), `reset-position-test` (140 from 304), `test_dmatiming/demo` (now equal), the 2010 timer printers, lidnariq's `smpspeed`, `ipl-speed-test`, `test_noise` (22,606 from 29,567) |
| Pictures further | `test_hello` (929 from 809), `enable-autojoy-late-test-2` (278 from 276) |
| The goldens | the same outcomes at all 36 anchors. Three hashes are re-recorded: Super Mario RPG at 2400 and Yoshi's Island at 1800, still differing, and Hook at 1500, still equal at lag -20 (+11). Super Mario RPG's attract mode still leads Mesen's by 40 frames (equal at 1240 for 1200) |
| Skip versus draw | the nine bench games, 600 frames, the same |
| Tests | the crate's 113. In WiseMan, `The_call_stack_holds_the_jsr_in_front_of_a_breakpoint_in_the_routine` armed its breakpoint after a first plain frame and assumed that frame ended outside the routine. It is now armed from power-on. With that, the VenusRt, CoreAbi, Snes, CoreDebug and Conform filters pass, 119 of 119 |

No cost: the change moves where the clock starts, not what a frame does.

### 41.16 Blockers 3 and 6: D-24 read against the referee, and three pictures with no console oracle (2026-10-04)

- **D-24, `test_irqb` case 5.** The referee step D-24 named was logged and read. It does not settle the entry.
  SNES_MiSTer strobes the S-CPU's reads only in cycles with VDA or VPA set, and CLC's second cycle and the interrupt's
  first microcode cycle are internal. So the referee reads $2180 once before the handler, as VenusRT does, where the
  ROM's console expectation is twice. The next rung, Mesen's source, is outside what this work may read. One
  hypothesis would be tested by a console ROM and is consistent with the other four cases: the 5A22 strobes /RD on
  internal cycles. It is recorded there and not built. G1 keeps `test_irqb` as an open entry.
- **`test_noise`** alternates INIDISP's brightness between 5 and 15 with two stores in a branch loop, so its picture
  is where each write lands in the line, to the dot. D-6's power-on position took it from 29,567 pixels differing to
  22,606. What is left is the brightness write's latency within a span, plan §5.1's INIDISP loss, which
  `inidisp_brightness_delay` already names. It joins that named loss.
- **`hvdma`** differs on two lines, 1 and 107, where its eight-channel burst of forced blank, VRAM data and unblank
  meets the start of a line. The ROM's console photograph shows the tile change but cannot resolve a line. Recorded
  as a difference with no oracle at its resolution.
- **`wrmpyb-in-flight`** prints what RDMPY holds when WRMPYB is written again 2 to 9 cycles into a product. Its source
  records no console result. fullsnes and anomie leave the in-flight product unknown (plan §2.2's thin areas), and
  the referee's multiplier carries an emulator's rule (`Venus_Referee.md` §0, nearly no support). Only Mesen's output
  is there to compare with, so it is recorded as a difference with no oracle.

### 41.17 Blockers 4, 5 and 10: what was found, none closed (2026-10-04)

- **D-26, `test_timer_stop2`.** One mechanism was tried in a trial build and not kept: the second stage counting on
  the rising edge of a gated first-stage clock. It prints 03 against the console's 04, and gating by bit 3 as well
  breaks `test_timer_stop`. Reaching 04 would mean choosing a duty to fit one printed number, and that was not done.
  Recorded in D-26.
- **`spc_dsp6`.** Run in order with the tester's boot image and the changes of §41.14-§41.15, the ROM now passes
  every test up to "Random/brr while playing", one of the checksum-only random tests, and fails there with checksum
  `B87AF7F6`. At §38.4 it stopped at "Misc/brr addr wrap-around". The single-test copies that §23.1 built in scratch
  were not kept, so the five tests that hang singly were not rerun. The remaining work is an S-DSP step of its own:
  rebuild the splitter, then the random tests' checksums one at a time.
- **Yoshi's Island's stork at 1800.** The stork's OAM is a copy of GSU RAM at $0AA0, which the GSU fills, and GSU RAM
  differs from Mesen's at 126 bytes. VRAM and CGRAM are equal. Over Mesen's frames 1760-1840 the OAM nearest
  VenusRT's is at frames 1799-1800, lag 0, while the background's nearest is at lag -10. The stork, which the GSU
  drives, and the scroll, which the S-CPU drives, keep a different phase from each other than in Mesen. That is the GSU
  phase drift of §27.4 and §28.1, not a sprite rule. It stays open with D-35.
- **NHL '94's puck at 2200.** At the matched frame, CGRAM is equal and VRAM differs by 1,250 bytes, all in the puck's
  tile data. OAM differs by 28 bytes, and WRAM by 2,340, the least at frame 2201. The game decompresses the puck's
  animation through WRAM, and its program state there differs from Mesen's, so the uploaded frame of the animation
  does too. BG mode is 1 there, so mode 7's open `$2134` products are not the cause. A program-state drift whose start
  was not found.

### 41.18 Blocker 13: Super Mario RPG's p99 at 33% quota, measured, and the lever proposed (2026-10-04)

Measured only. Nothing was built, since optimisation is on hold.

- **The slow frames are one scene.** Through the bench's script (`frametimes`, a scratch tool), timed frames 1,800 to
  2,100 are the heaviest on the desktop: p50 3.32 ms a frame, p90 and p99 3.82 and 3.88, max 4.02, the slow 1% all in
  that window. They are a scene, not scattered spikes.
- **Under the quota the scene costs more than the mean's factor.** At 33%, the mean is 12.4-12.7 ms, 3.8 times the
  desktop's, and the p99 18.2-18.9 ms, 4.7 times the desktop's p99. A finer quota period (1 ms against 5 ms) takes
  the p99 from 18.6-18.9 to 18.2. So the quota's own granularity adds about half a millisecond, and the rest is the
  scene's work, slowed more than proportionally under the quota (argued: the process loses its caches when throttled).
- **Where the scene's time goes**, from a ptrace sampler of instruction pointers over 6 s inside the scene (5,656
  samples, symbols from the binary):
  - the PPU 42%: compositing 19%, spans 8.5%, encoding 7.6%, backgrounds 3.4%, the rest smaller;
  - the SA-1 37%: its bus reads 11%, `Sa1::run_to`'s catch-up loop itself 10%, the 65C816 core on its bus 14%, the
    D-36 holds 0.8%;
  - the S-CPU and the APU about 15% together.
- **The lever proposed.** The plan's §9 Q3 gives the first lever for exactly this case: the SA-1's catch-up bound,
  made a setting before anything else is loosened. Today the SA-1 is caught up after every S-CPU instruction, and
  the catch-up's own loop is a tenth of the scene. A bound of several S-CPU instructions, kept exact wherever the
  S-CPU touches the SA-1's registers or shared memory, would remove most of that tenth. The p99 needs about 12% to
  reach 16.6 ms. The second lever, the SA-1 bus's address decode as a per-bank table (11%), changes no behaviour and
  could be built without a setting. Either needs a go: the first changes contention timing within the bound (D-36),
  and both are optimisation. **Predicted** (P8 of this record), before any measurement: the bound alone takes
  Super Mario RPG's 33%-quota p99 under 16.6 ms, and the decode table alone does not.

### 41.19 The gates after the blockers (measured 2026-10-04)

| Gate | Now | Changed by |
|---|---|---|
| G1 | **not met**: 113 of Mesen's 117 with the tester's images, 112 without (`spc_smp`, the one ROM excepted by §41.3's proposal); every C# Venus pass passes. Left: `spc_dsp6` (§41.17), `test_timer_stop2` (D-26), `blobs/test_hdmatiming` (D-40's test 2, one dot), `test_irqb` (D-24) | D-39, D-40, D-6 |
| G3 | **not met**: 125 of the 151 standing ROMs equal (118). Of the 26 left: plan §5.1's losses 14 (with `test_noise`), counter printers 4, D-27's printers 2, D-16 1, the Cx4 1, no console oracle 2 (`hvdma`, `wrmpyb-in-flight`), unlogged 2 (`test_hello`, `HblankEmuTest`) | D-39, D-40, D-6 |
| G4 | **not met only by decisions**: the refusal names Venus (C#) (§41.12). The output-rate setting and the debugger's console views are put to the tester | §41.12 |
| G5 | **not met**: 29 of 36, the same outcomes. D-7 (3) and Super Mario RPG's 40-frame lead (2) are put to the tester; the stork (GSU drift, D-35) and the puck (program-state drift) are open (§41.17) | no outcome changed |
| G6 | **not met**: the means within budget; Super Mario RPG's p99 18.2-18.9 ms with a lever proposed (§41.18); the handheld still unreachable at 10.1.1.205 through the day | measured only |
| G2, G7, G8 | as §41.2 (G8's five amended entries done, §41.13) | §41.13 |

### 41.20 The four decisions blockers 8, 9, 11 and 12 waited on (decided 2026-10-04)

- **Blocker 8, the output rate.** The core's fixed 32 kHz counts as parity. The host's audio path already resamples to
  the device's rate, so a resampler inside the core would change nothing a player hears. G4 no longer waits on it.
- **Blocker 9, the debugger's console views.** The default may flip without them. The loss stays recorded (§36.3), and
  the views return later as extension exports that any console's core can offer, not as a VenusRT-only surface.
- **Blocker 11, D-7 on three anchors.** Accepted as logged. The reading follows the documents, and Mesen is the
  comparison, not the authority. G5's three D-7 anchors count as dispute-logged.
- **Blocker 12, Super Mario RPG's attract lead.** Accepted as logged drift (D-6, D-37). The scenes are equal, shifted by
  under a second, and every documented timing rule is already built. G5's two drift anchors count as dispute-logged.

With these, G4 is met apart from the recorded debugger loss. G5 is met apart from Yoshi's Island's stork (the GSU phase
drift, D-35) and NHL '94's puck (a WRAM drift whose start is not found). G6 still waits on the handheld battery run and
Super Mario RPG's p99.

## 42. The NEC DSP replacements, steps 9 and 10: the DSP-2 (2026-10-04)

Steps 9 and 10 of `VenusRT_DspHle.md` §8, taken before the DSP-1's steps 5 to 8 so that a game runs exact soonest
(*decided 2026-10-04*). Step 9 covers every DSP-2 command with its transfer timing, the unhandshaken transfers of 01h
and 0Fh among them (§37.3); step 10 runs Dungeon Master in lockstep against the image. §42.1 was written and committed
before the first comparison of a replacement with the image.

### 42.1 The commands as characterised, and the families declared before the first comparison

**What the documents give.** fullsnes names nine commands and states that 10h-FFh mirror 00h-0Fh; nothing documents a
parameter, a length or a byte order. Everything below beyond the names is black-box characterisation under plan §1.3.
It was formed with two tools added for this step: `dsp_edges`, which prints the chip's DR edges against an S-CPU that
answers late or ignores RQM, and `dsp_oracle ask`, `phases` and `batch`, which run single transactions and print their
transfers, results and per-phase timing. Each hypothesis came from the structure of a handful of cases: single bits set
in an input, a run of distinct nibbles, a pair of signed values. No result file of the image enters the repository.

**Exposure, recorded.** One `ask` of 1Eh printed the first eighteen words of the DSP-2's data ROM to the terminal before
the byte was recognised as the data ROM transfer. Nothing of them is used; 1Eh is a named loss below. The value 2Eh
returns was also seen before its family was declared.

**The protocol, measured** (protocol facts and latencies, which plan §1.3 admits into the model):

- **Transfers.** 8-bit throughout, but for the four test commands 0Eh, 1Eh, 2Eh and 3Eh, which turn DR to 16-bit 11
  cycles after the command's read whatever the S-CPU does. 3Eh turns it back at 16 cycles and goes idle at 19; the
  other three turn it back 3 cycles before their idle edge.
- **Reads.** Each input is read by an edge that raises RQM, except, for most commands, the last, which is taken without
  one when the S-CPU completes it. 01h reads all 32 inputs by edges.
- **Waits.** Every phase waits for the S-CPU except three. 01h writes its first result 15 cycles after reading its
  last input. 0Fh goes idle 14 cycles after its command's read. 3Eh goes idle at 19. 05h with a count of 0 goes idle
  8 cycles after reading it. An S-CPU that answers later than these finds the chip already past them, which is the
  dependence on answer time of §37.3.
- **Counts.** 02h converts m tiles, m = n for n from 1 to 3 and 4 otherwise. 05h takes n = min(n, 80) byte pairs; 06h
  takes n bytes; 0Dh takes ceil(a/2) bytes and gives ceil(b/2).
- **The word left in DR at idle**, which the S-CPU sees only by reading at idle, is per command: 0000h at power-on and
  after 00h, 02h, 03h and 0Dh; 0003h after 01h; FFFFh after 06h and 0Eh; 00FFh after 0Fh, 2Eh and 3Eh; 0400h after 1Eh;
  after 04h and 05h the last result; after 07h and 08h the first two results as a word, the first high; after 09h the
  third and fourth so; after 0Bh and 0Ch the fifth and sixth so, or 0000h on a zero divisor.
- **Flags.** 05h with a count of 0 sets USF1, and 0Fh clears it.
- **Timing.** Each phase's work and notice, as `dsporacle::handshakes` measures them, are constants per command and
  position, with these rules: 04h's result comes 15 cycles after its input plus one per nibble of the overlay that is
  not the transparent colour; 05h's compute takes 10n + 9 plus one per such nibble, and when n mod 16 is 1, a further
  160 plus the last byte's count again; 0Bh's division takes 739 + 2·popcount(q) + 4·(q mod 2), and 0Ch's 735 +
  2·popcount(|q|) + 4·(|q| mod 2), plus a constant per sign class, plus one where a negated quotient or remainder is 0.
  0Dh's compute depends on a and b alone, by no rule found; it takes a fitted estimate and is recorded as inexact.

**The families.** Each is the formula in words and the choices it leaves open:

| Code | Formula | Variants (bits of choice) |
|---|---|---|
| 00h | one row of 8 pixels, 4 bytes of two 4-bit pixels each, to its four bitplanes, plane 0 first | the left pixel in the high or the low nibble; pixel p at bit 7 - p or bit p (2 bits) |
| 01h | an 8 by 8 tile of 32 such bytes to the SNES's 4-bit tile, fullsnes's planar layout: planes 0 and 1 row by row, then planes 2 and 3 | as 00h; and that layout or four planes per row in turn (3 bits) |
| 02h | m tiles as 01h, one after another | as 01h, chosen with it (0 further bits) |
| 03h | the transparent colour, the input's low nibble | the low nibble, or the whole byte compared with each nibble (1 bit) |
| 05h | n bytes A, then n bytes B; each nibble of B unless it equals the colour, then A's | B over A, or A over B (1 bit) |
| 04h | 05h for one byte, with the same colour | as 05h, chosen with it (0 bits) |
| 06h | n bytes reversed as a row of pixels: the byte order reversed and each byte's nibbles swapped | that, or the byte order alone (1 bit) |
| 07h, 08h | A + B and A - B of two 32-bit values, each sent low byte first | 32-bit, or two independent 16-bit halves (1 bit each) |
| 09h | M = K·L·2, the µPD77C25's product of two signed words (fullsnes); fullsnes calls the command bugged | M's halves each shifted right once, M shifted right as 32 bits logically or arithmetically, or K·L (2 bits) |
| 0Bh | an unsigned 32-bit division: quotient, then remainder | that order or the other (1 bit) |
| 0Ch | a signed 32-bit division | truncating or flooring; magnitudes of 32 or of 31 bits (2 bits) |
| 0Dh | a pixels scaled to b: for b < a, output pixel j is input pixel floor(j·a/(b + 1)); otherwise the first a pixels and zeros | the step a/(b + 1) in 8, 12 or 16 fractional bits or exact (2 bits) |
| 0Eh | the memory test's word on a working chip | 0000h, 0001h, 00FFh, FFFFh (2 bits) |
| 1Eh | the data ROM transfer: **not replaceable**, a named loss answered with zeros | none |
| 2Eh | the ROM version | 0100h, 0200h, 0201h, 0000h (2 bits) |
| 0Fh, 3Eh | do nothing (fullsnes's "dummy NOP") | none |
| 0Ah | 09h | none |

That is 21 bits of choice over the chip, against the 2^20 seeded cases per code that will grade it. **Named exceptions,
not modelled:** 06h with a count of 0 and 0Dh with a of 2 or less or b of 0, where the program leaves its normal
course (06h turns DR to 16-bit and sets SIC; 0Dh reads past any count or stalls). A member is chosen when it agrees on
every graded case; a command none agrees on is a named loss under R4.

### 42.2 Three families widened once, in writing (plan §5.2, rule 4)

The first grade with `dsp_grade` rejected every declared member of three families. For each, the structure of the
mismatches pointed to one further choice. That choice is recorded here as the widening. **This departs from the order
plan §1.3 asks for:** the widened member was identified from the first grade's mismatches before this paragraph was
written, and was then graded on fresh seeded cases. The bits of choice it adds are counted below.

- **09h.** The two halves' shifts are each logical or arithmetic: 4 members, 2 bits, in place of the declared "halves
  each shifted right once". The mismatches were bit 15 of the low half alone.
- **0Ch.** The magnitudes are 31 or 32 bits for each operand separately: 4 members, 2 bits, in place of one choice for
  both. The mismatches were divisors of ±2^31 and dividends of -2^31.
- **0Dh.** The step a/(b + 1) takes 7 to 16 fractional bits: 10 members, under 4 bits. In the copy branch the first
  ceil(a/2) bytes are copied whole, rather than the first a pixels: 1 bit. The mismatches were pixels at an exact
  multiple of the step, and the pad nibble of an odd a.

### 42.3 What was chosen and built, and the grade (measured 2026-10-04)

**Chosen**, each agreeing on every graded case:

| Code | Member |
|---|---|
| 00h, 01h, 02h | the left pixel in the high nibble, pixel x at bit 7 - x; 01h and 02h in fullsnes's tile layout |
| 03h | the low nibble |
| 04h, 05h | B over A |
| 06h | the byte order reversed and each byte's nibbles swapped |
| 07h, 08h | 32-bit |
| 09h | (M_hi >> 1, logically) and (M_lo >> 1, arithmetically), as 16-bit halves |
| 0Bh | quotient, then remainder |
| 0Ch | truncating; the dividend's magnitude in 31 bits, the divisor's in 32 |
| 0Dh | the step in 10 fractional bits; whole bytes copied |
| 0Eh | 0000h |
| 2Eh | 0100h |

Counting the widened families whole, the chip's choices total about 26 bits. **None needs a table.** The replacement is
`chips/dsp2.rs`. Its phases follow §42.1's protocol: each edge at the later of its work after the last edge and its
notice after the S-CPU's completion, or at its work alone where the chip does not wait.

**Found by the grade**, beyond §42.1, and modelled (protocol facts and latencies):

- **The flags.** The commands 07h, 08h, 0Bh, 0Ch, 0Fh and the 0Eh class write SR 10 cycles after their command's
  read, which clears USF1. 07h and 08h then set USF1 to the signed overflow of their result, 2 cycles after their
  second result. 0Bh, 0Ch and 05h set USF0 while they compute, 3 cycles after the last input. 05h clears it 1 cycle
  before its first result, and 0Bh and 0Ch 1 cycle after. A zero divisor sets USF1 instead and gives no result. 0Dh
  sets USF1 while it computes. DRS, which the program cannot write, survives a change of DR's width.
- **Two buffers in the chip's RAM that persist between commands.**
  - 05h's compute takes, when n mod 16 is 1, one cycle more per opaque nibble over the whole last 16-byte chunk of
    its overlay buffer. The bytes past n are left from earlier 05h commands, and each byte's count is the one made
    against the colour of its own time.
  - 0Dh's copy branch gives its row buffer whole, 104 bytes, and so returns pixels left there by earlier 0Dh commands.
- **Smaller timing rules.** 05h reads B0 with a work of 7 when n exceeds 80. 01h's second result comes 5 cycles after
  its first, which only an answer late enough to see the unhandshaken first result reveals. 0Ch's sign costs are 18
  for a negative dividend and 10 for a negative divisor, 9 for a negated quotient, and one cycle more for each
  negated value whose low word is 0.

**The grade.** `dsp_grade` ran 2^20 seeded cases per code, every other one under a jittered S-CPU; 04h and 05h ran
after a seeded colour; the counts were held to 1-255 for 02h and 06h, to 1-80 for 05h, and for 0Dh to a of 3-208 and b
of 1-208:

| Codes | Cases | Values, transfers, SR or latency differ |
|---|---|---|
| 00h, 01h, 02h, 03h, 04h, 05h, 06h, 07h, 08h, 09h, 0Ah, 0Bh, 0Ch, 0Eh, 0Fh, 2Eh, 3Eh | 1,048,576 each | **0** |
| 0Dh | 262,144 | values 0; latency in 261,179 |
| 1Eh, the data ROM transfer | 1,048,576 | values in all, by construction; transfers, SR and latency 0 |
| the 180 mirrors in 10h-FFh of the codes without a count | 4,096 each | 0, but for 1Eh's four mirrors, by construction |

The crate test `the_dsp2_replacement_agrees_with_the_image` repeats this over 256 cases per code on the image, each
after a colour, under steady and jittered answers. `the_dsp2_formulas_answer_through_the_ports` checks fullsnes's tile
layout and the arithmetic on the replacement with no image.

**Named losses and inexactness**, in the core's stated cost:

- **1Eh**, the data ROM transfer, answers zeros: a named loss, as the plan set out.
- **0Dh's compute time** is a least-squares estimate. The error is under 11% when scaling down and under 6% when
  scaling up, over 5,774 measured cases. No rule was found in its dependence on a and b.
- **Outside the chip's buffers, not reproduced.**
  - 05h with a count of 0 (which sets USF1) is modelled; above 80 its compute is off by a cycle and some results differ.
  - 06h with a count of 0 leaves the program's normal course: DR turns 16-bit and SIC is set, from then on.
  - 0Dh with a or b over 208 reads past its 104-byte buffer, and with a of 2 or less or b of 0 it does not end.
  - Commands that share RAM with 05h's and 0Dh's buffers (06h, 0Bh, 0Ch and the memory test 0Eh) leave there values
    the replacement does not track.

### 42.4 Step 10: Dungeon Master in lockstep (measured 2026-10-04)

`dsp_lockstep`, the whole-game runner of plan §4.3, is new: one ROM on two machines, the image on one and the
replacement on the other, the same pad on both. Each frame it compares the picture, WRAM, VRAM, CGRAM, OAM, the APU's
RAM, the master clock and the whole state with the chip's group left out. With `LOCKSTEP_AT` it finds the first
instruction after which the two S-CPUs differ.

| Run | Frames | Result |
|---|---|---|
| `dm.txt`, Start into the dungeon | 3,600 | **identical states throughout** |
| `dm.txt` | 10,800 | **identical throughout**; 353,703 05h, 155,219 01h, 14,344 0Fh, 4,164 03h and 4,164 09h at the image's chip |
| `dm_walk.txt`, the same, then 132 taps of the pad and buttons | 7,205 | **identical throughout** |

**What the lockstep found that the command grade could not.** The first lockstep parted at frame 1,036. Dungeon
Master polls SR during 05h's compute and waits for USF0 to clear. The command oracle samples SR only at each rise and
at the S-CPU's access, so it never saw the busy flag. A second parting, one frame earlier, placed USF0's clear 1 cycle
before 05h's first result. Both are now modelled, and the grade was rerun without change.

**What this settles.** Dungeon Master runs on the replacement as on the image, to the cycle and the byte, over 10,800
frames. The DSP-2's cost in core info now says so, and names 1Eh, 0Dh's timing and the out-of-range counts.

**Against the plan's predictions.** **P8**, every DSP-2 command exact by the end of step 9, is **retired, false in
part.** The values of every command are exact within the chip's buffers. 0Dh's timing is not, nor are the
out-of-range counts. 1Eh is a named loss, as planned.

## 43. The NEC DSP replacements, step 11: the ST010 (2026-10-04)

Step 11 of `VenusRT_DspHle.md` §8: the ST010's mailbox, and the seven computing commands F1 ROC II gives it (02h-08h,
§37.6), with the RAM compared whole and F1 ROC II run in lockstep. §43.1 was written and committed before the first
comparison of a replacement with the image.

**Provenance.** This step was written by someone who has not seen the superfamicom wiki's ST010 page (plan §1.4). Its
sources are fullsnes ("ST010 Commands"), plan §3.5's table of parameter addresses (taken from that page's address
tables by the plan's author, who recorded the exposure), and the oracle. No emulator's source, no listing and no byte
of the dumps were read.

### 43.1 The commands as characterised, and the families declared before the first comparison

**The tools.** `dsp_oracle mail` runs mailbox commands with chosen RAM words. `mailbatch` runs one case a line.
`dsp_trace` with `DSP_TRACE_MAIL` records each of a game's mailbox commands with its input words and the words it
changed, to the probe cache. The hypotheses below came from single words set against a zero RAM and against seeded
fills, and from F1 ROC II's own traffic. Words are 16-bit RAM addresses; plan §3.5's byte addresses are twice these.

**The protocol, measured.**

- **The mailbox.** The chip polls bit 15 of word 10h every 3 cycles; a command's busy bit clears a fixed number of
  cycles after the first poll that sees it, so the latency varies by 0 to 2 with the S-CPU's phase. The poll resumes
  after each command.
- **The results.** Each command writes its results and some working words, and nothing else. Its time from the poll
  is a constant per command, except for 02h and 04h.

**The families.**

| Code | Formula | Variants (bits of choice) |
|---|---|---|
| 06h | words 8-9 = the µPD96050's product K·L·2 of words 0 and 1 (fullsnes's multiplier), 32 bits | that or K·L (1 bit) |
| 03h | words 8-9 = word 0 times word 2, and words A-B = word 1 times word 2, each as 06h | chosen with 06h (0 bits) |
| 04h | |X| and |Y| of words 0 and 1, the larger stored in word 0 and the smaller in word 1, and word 8 = α·max + β·min: the alpha-max-plus-beta-min distance, α = 2cos(π/8)/(1 + cos(π/8)) and β = 2sin(π/8)/(1 + cos(π/8)) | α and β in 15 or 16 fractional bits, each floored or rounded; the sum floored or rounded; ties stored as swapped or not (4 bits) |
| 08h | words 8, 9 = X·cos θ + Y·sin θ, Y·cos θ - X·sin θ of words 0, 1 and angle 2, with S[i] = A·sin(2πi/256) at θ's top 8 bits | A 32,767 or 32,768 saturated; floor or half up; computed per quadrant from the magnitude or directly; each product shifted on its own or the sum once (4 bits) |
| 07h | four arrays of 176 words from word 78h, 128h, 1D8h, 288h: L(n)·cos θ, L(n)·sin θ, -L(n)·sin θ, L(n)·cos θ, with 08h's sine and product; words 0-2 = θ's top 8 bits, sin θ, cos θ. L(n) = round(K / (n + c)), the perspective scale of a raster line | the negation before or after the product (1 bit); K and c, found by a search over the image's 176 words at θ = 0, K = 7,885 and c = 8.8 (about 15 bits) |
| 02h | a stable sort of word 12h's count of words from 20h into descending order, the words from 40h moved with them; word 0 the loop's count, left 0 after a sort and the count when it is 0, 1 or over 32 | keys unsigned or signed (1 bit); timing by comparisons and exchanges, measured |
| 00h | word 10h cleared, nothing else (§37.5) | none |
| 09h-0Fh, 10h-FFh | 01h-07h and 00h-0Fh (fullsnes, §37.5) | none |

**07h's K and c are the one place this step takes numbers from the image's output.** They are two constants of a
documented shape, a perspective divisor, fitted to one angle's 176 words, and the other 65,535 angles grade them. By
plan §1.3's argument, about 15 bits are taken against 2,816 output bits per angle. This is recorded as a characterisation
with fitted constants, and the record does not claim more.

> *Amended 2026-10-04* (§48.2). Fitting K and c to the image's output takes values from the chip into the code, which
> plan §1.3's R1 forbids however few bits they carry. The fit is withdrawn, and 07h takes a documented constant in
> their place.

**Not characterised in this step.** 05h, the driver simulation, is 53.6% of F1 ROC II's commands. It reads at least
seventeen words from 60h, among them a flags word whose bits choose its paths, and computes an angle as 01h does.
Its paths were not resolved in the time this step had. 05h and 01h stay as the frame left them: they clear the
mailbox and change nothing else, a named loss for now.

### 43.2 One family widened once, a count corrected, and the sine a named loss (measured 2026-10-04)

`dsp_stgrade` is new. It grades a command through the mailbox on both engines from a seeded phase of 0 to 8 cycles,
comparing the cycles to the busy bit's clearing and the whole RAM. With `mailbatch` and `DSP_TRACE_MAIL` it graded the
families of §43.1.

- **04h.** No member agreed on every case: the best agreed on 42%, and a closer look found the alpha-max-plus-beta-min
  form right but α's constant half a unit off. **The family is widened once**: each constant floored, rounded or
  raised (1 bit more). As with §42.2, the member was found in the same pass that rejected the declared ones.
  - **Chosen**: |X| and |Y| as unsigned 16-bit magnitudes, so that -32,768 stays 8000h, swapped when |X| <= |Y|;
    D = (2·α·max + 2·β·min + 8000h) >> 16 with the magnitudes signed in the products.
  - α = ceil(2^15 · 2cos(π/8)/(1 + cos(π/8))) = 31,472, and β = round(2^15 · 2sin(π/8)/(1 + cos(π/8))) = 13,036.
  - It agreed on 20,000 of 20,000 cases.
- **02h's count**, a protocol fact, is 2 to 15, not 2 to 32. A count of 16 or more sorts nothing and leaves the
  count in word 0. The sort is a bubble sort of fixed passes, unsigned and stable. Its time is 41 + 10n + 6n(n - 1) +
  15 per exchange, which agreed on 800 of 800 traced cases.
- **08h's and 07h's sine is a named loss under R4.**
  - The family's 32 members agreed with the image on at most 18,200 of 20,000 08h cases (91%). The closest is
    round(32,767·sin) by quadrant, each product shifted on its own, the negation after.
  - firmwarecheck against the ST010's data half finds that member's table in the program's data in runs of up to 120
    words, broken at single entries. That is the formula correct except at a few entries, which plan §5.2 makes a stop.
  - No entry is patched, and the closest member is kept.
  - 07h's L(n) and its products agree wherever the sine does.

**firmwarecheck** (`dsp_oracle tables`, then `st010_tables.py` for the formula image, byte-identical to it), against the
data half:

| Run | Result |
|---|---|
| plain | 3 of 512 bytes equal at the same offset; common runs up to 240 bytes; FAIL by the plain thresholds, the expected outcome of a formula found in the data |
| `--forced` | 12 formula runs; coverage 608 of 4,096 bytes (14.8%); residual 0; **PASS** |

**The mailbox's phase.** The poll's base is set 2 cycles after the start word's read and 2 cycles after each
command's busy bit clears, from which polls fall every 3 cycles. Each command's time runs from the first poll at or
after the S-CPU's write. With those two constants every graded command's latency agreed, from every phase.

**The grade** (`dsp_stgrade`, seeded phases and inputs; 02h's count held to 0-20):

| Codes | Cases | Latency differs | RAM differs |
|---|---|---|---|
| 00h, 02h, 03h, 04h, 06h, and the mirrors 0Ah, 0Bh, 0Eh, 1Ch, F4h, FAh | 4,000 each | **0** | **0** |
| 08h | 4,000 | 0 | 50 (1.25%), by one in a result word |
| 07h | 4,000 | 0 | 126 (3.2%), by one, in the words that carry the sine |
| 07h, after §48.2 | 4,000 | 0 | 4,000, by up to 640 in the four arrays; words 0-2 by one |

The crate tests `the_st010_replacement_agrees_with_the_image` (256 cases per code on the image, from seeded phases) and
`the_st010_formulas_answer_through_the_mailbox` (fullsnes's multiplier and the sort, with no image) hold this.
`the_st010_sine_equals_the_independent_generator` holds the table to `st010_tables.py`, and
`the_st010_battery_file_reads_alike_on_either_engine` holds the battery file across engines: a file one engine
writes is, byte for byte, the RAM the other engine starts from.

### 43.3 F1 ROC II in lockstep (measured 2026-10-04)

`dsp_lockstep` now compares the ST01x's RAM as well. F1 ROC II, 3,600 frames from power-on with no input:
**identical to frame 1,019**, through its boot, its titles and every 06h, 03h, 04h, 02h and 08h before the first
05h. The first parting is the S-CPU's read of 05h's first result at frame 1,019 (word 62h, 0034h against 0000h).
Pictures were equal in 1,765 of 3,596 frames. 05h is a named loss for now (§43.1), so this parting is the expected
one.

**What is open.** 05h, the driver simulation, carries 53.6% of the game's commands. Its traffic, 98,150 cases in
7,200 frames, is in the probe cache. The first rule found there holds in every case: words 68h and 69h are words 60h
less 63h and 61h less 65h. A further characterisation step of its paths would let F1 ROC II run past frame 1,019. With
the sine a named loss, 05h could be exact only where its paths do not use the sine.

## 44. The NEC DSP replacements, steps 5 to 8: the DSP-1's remaining commands, begun (2026-10-04)

Steps 5 to 8 of `VenusRT_DspHle.md` §8. The sine is a named loss (§40.7), so every command built on it can only be
approximate. This section therefore records closest members and their measured error, not exact choices. The
commands that do not use the sine, Inverse and Distance, were searched for an exact member and none was found in the
time given. This step is not finished: §44.4 lists what remains.

### 44.1 What was characterised, from the oracle

- **Attitude (n1h), Objective (nDh), Subjective (n3h), Scalar (nBh).**
  - Each Attitude sets matrix n to (m/2)·Rx(I4)·Ry(I3)·Rz(I2).
  - Objective gives the matrix times (X, Y, Z), Subjective its transpose times (F, L, U), and Scalar the first row
    times the vector.
  - The order and the axes were found by unit vectors through Objective after Attitudes with random angles: the
    best of the 96 orders, axis assignments, signs and transpositions agreed to 4.5·10^-4 of full scale, and the next
    to 1.04.
  - As with Polar (§40.4), the angles are not SnesLab's "about Z, X, Y": the second rotates about Y and the third
    about X.
  - With m = 7FFFh and no rotation, the diagonal is 3FFEh, consistent with three floored products by cos 0 and a
    halving.
- **Inverse (10h).**
  - a is normalised to a mantissa in [4000h, 8000h) or [-8000h, -4000h) with s shifts, and the exponent is 1 + s - b.
  - The exponent agreed on all 65,536 values of a at b = 0. The mantissa lies within 2 of 2^29 over the normalised
    a, floored: equal in 48.7%, one away in 49.6%, two away in 1.6%. That is the signature of a Newton iteration's
    truncations.
  - The best of 96 simple Newton members (seed, iterations, product rounding) agreed in 14,701 of 16,384 positive
    mantissas (89.7%).
  - **The time is a rule of one feature, exact on all 65,536 values**: 36 + 2s cycles for a positive a and 40 + 2s for
    a negative one, plus 2 (positive) or 1 (negative) when the normalised value is a power of two; 14 for a = 0.
- **Distance (28h).**
  - On the DSP-1B the root is the floor of √(x² + y² + z²) or up to 4 below it, for sums under 2^30. Above that, the
    sum's overflow takes another course that was not resolved.
  - The time steps by 4 cycles per two bits of the sum's length, in two runs for the sum's low and high words.
  - The DSP-1's fullsnes bug multiplies the root by a factor between 0.97 and 0.99 that alternates with the parity of
    the sum's bit length. It was measured over 30,000 cases and not modelled.
- **Parameter (02h), first findings.**
  - Vva = -Les·cot(Azs), with Azs the angle from straight down. It agreed at every angle tried below a limit near
    79.8°, where Vva holds and Vof becomes Les·(cot θ_limit - cot Azs).
  - (Cx, Cy) is F moved along the view's forward direction (-sin Aas, -cos Aas) by Fz·tan(Azs).
  - Raster, Project and Target were not characterised.

### 44.2 What was built and the grade (measured 2026-10-04)

`dsphle.rs` gains the three matrices in its state, so the state version is 20. A replacement state of version 19 is
refused with `VERSION`, while the low-level path's states of 17-19 still load. *Amended 2026-10-04* (§48.1): the version is 21. Steps 9 to 11 also changed the
DSP-2's and the ST010's replacement states without raising the version; that defect is corrected here.

| Code | Built as | Grade (2^17 to 10^5 seeded cases, both images where alike) |
|---|---|---|
| 10h Inverse | normalised; 2^29 over the mantissa, floored; the exponent and the time by §44.1's rules | exponent and latency exact in every case; mantissa within 2, exact in 48.8% |
| 28h Distance | the floored root; the time by §44.1's steps | DSP-1B within 4 for sums under 2^30; the DSP-1's bug and sums of 2^30 and over not modelled |
| n1h Attitude | (m/2)·Rx·Ry·Rz in floating point from the replacement's sine, floored to Q15 | through the commands below |
| nDh, n3h, nBh | each product floored, summed | within 18 of the image in every case; within 3 in 92% (Objective, Subjective) and 96% (Scalar) |

The crate test `the_replacements_approximate_commands_stay_within_their_bounds` holds these bounds on both images.

### 44.3 The seven DSP-1 titles in lockstep

`dsp_lockstep`, 3,600 frames each, with §37.6's pad scripts:

| Game | Identical until | Pictures equal |
|---|---|---|
| Super Mario Kart | frame 96 | 258 of 3,600 |
| Ballz 3D | 762 | 1,007 |
| Pilotwings | 1,799 | 1,802 |
| Lock On | 1,523 | 1,525 |
| Suzuka 8 Hours | 1,474 | 1,491 |
| Super Bases Loaded 2 | 3,029 | 3,030 |
| Michael Andretti's Indy Car Challenge | 3,310 | 3,310 of 3,593 |

Each parts one frame after its first command that computes (§37.6). That is the expected outcome while the sine is a
named loss and the projection commands are unbuilt: **P6 is retired, false**, since no DSP-1 title can be identical
to the image while its sine is approximate.

### 44.4 What remains of steps 5 to 8

- **Parameter, Raster, Project, Target.** These are the most-used commands: Project alone is 56% of Super Mario Kart's
  traffic. §44.1's first findings fix the camera's frame. Raster's mode 7 matrix, Project's screen position and
  enlargement, Target's ground point and the limit-angle branch remain to be characterised.
- **Gyrate (14h)**, six inputs and three results, not characterised.
- **An exact Inverse and Distance**: a wider declared search over seeds and iteration sequences, if the plan's rules
  allow it.

## 45. The NEC DSP replacements, step 12: the DSP-4 characterised, and the decision put (2026-10-04)

Step 12 of `VenusRT_DspHle.md` §8. No document describes the DSP-4's commands (plan §3.4). This step reads Top Gear
3000's traffic (§37.6) and grades hypotheses against the image. It ends in a decision that is the tester's to take,
not this record's.

### 45.1 The protocol, measured

- **The first input is written over the chip's offer.** After a command the DSP-4 writes a word first (§37.2), and
  Top Gear 3000 writes its first input over it: the "bytes against the chip's direction" of §37.6, two per command
  for the short commands. The chip then takes DR as that input.
  - The oracle's driver gains `write_over`, which answers chosen transfers with a write, and `dsp_oracle` sets it with
    `ORACLE_WRITE_OVER`.
  - With it, the short commands' shapes become plain: 00h `i2o2`, 0Bh `i3o`, 11h `i4o`, 0Ah `i4o4`.
- **Unhandshaken results.** 00h's first result is written without waiting for the S-CPU, 8 or 9 cycles after its last
  input, as the DSP-2's 01h is (§42.1).
- **The long commands are conversations.** 01h, 07h, 08h and 09h alternate short runs of inputs and results for as
  long as the game supplies segments: 30 to 90 exchanges in Top Gear 3000. They are the road and its scenery projected
  segment by segment, and their traffic is 78.8% of the game's DR transfers.

### 45.2 What is characterised

| Code | Share of commands | Share of DR transfers | Status |
|---|---|---|---|
| 00h | 82.3% | 16.7% | **a rule, exact on 100,000 of 100,000 seeded pairs**: (K·L·2 as a signed 32-bit word) >> 1, the µPD77C25's product (fullsnes) halved arithmetically, low word first; so (-32,768)² gives C0000000h |
| 0Ah | 11.4% | 3.7% | four results from four inputs, linear in the second at least (a 10h there gives 30h); in the game its results are fixed pairs (FF40h, 00C0h) for any first input. Not resolved |
| 0Bh | 1.3% | 0.3% | in the game, three results; with the oracle's inputs, one result of 0 and a latency of 16 to 19 that depends on the inputs. Not resolved |
| 11h | 0.3% | 0.1% | one result, an angle in appearance (00E8h alone gives 4000h). Not resolved |
| 01h, 07h, 08h, 09h | 2.8% | 78.8% | conversational; not characterised |
| 03h, 05h, 06h | 2.0% | 0.4% | no transfers (03h, 05h) or sixteen results with no input (06h); not characterised |

### 45.3 Against P9, and the decision

**P9** asked for rules explaining at least 80% of Top Gear 3000's traced command volume after step 12's two steps.
The answer depends on what is counted:

- **By commands, P9 holds**: 00h alone is 82.3% of them, and it is exact.
- **By DR transfers, it fails**: 16.7%. The four conversational commands, with 78.8%, are where the picture is made, and
  none of them is characterised.

**The options**, each argued:

- **A, no-go.** The DSP-4 keeps `effect: none` and Top Gear 3000 runs only with the player's image. The cost is one game.
- **B, a limited go.** Build the replacement's frame, with the first-input protocol, 00h exact, and the short
  commands as far as one more characterisation step reaches. Then decide on the conversational commands against
  their own measurement. The game would run, but its road would not be drawn until those commands are built. That is
  `effect: accuracy`, with the cost naming the road.
- **C, the full go of plan step 12a,** 4 to 6 steps. Whether 01h-09h are formula-like is unknown. If they use a sine
  of their own, the DSP-1's and the ST010's histories (§40.7, §43.2) suggest they will be approximate at best.

**Recommended: B**, because it costs one step, makes the game start without the image, and replaces a guess about the
conversational commands with a measurement before more is spent. **The decision is the tester's**, and nothing past
this record is built until it is taken.

## 46. The NEC DSP replacements, step 13: the DSP-3's decoder question (2026-10-04)

Plan §3.3 asks whether 38h, the Shannon-Fano decoder, depends on a table the game sends or on the chip's own data. If
the game sends it, the decoder is a documented algorithm; if the chip holds it, R4 stops it. No DSP-3 game is in the
library, so only the command oracle can answer.

**What was measured.**

- **38h is driven by USF1.** fullsnes says "USF1 bit in SR register = direction". The driver gains `usf1_writes`,
  `ORACLE_USF1_WRITES` in `dsp_oracle`, which answers a rise with USF1 set by a write. With it the decoder runs as a
  conversation: after the command, one word in and one out, then a run of inputs, then symbols out, interleaved with
  further inputs, as variable-length codes would need.
- **The opening run's length depends on the stream's content.** Over four seeded streams it was 52, 49, 47 and 90
  words before the first symbol. Two streams that differed only after their hundredth word gave the same opening run
  and the same first twenty symbols. So did two streams that shared words 1 to 59 and differed in their first word
  and after their sixtieth: the first word is not part of the preamble.

**The answer, argued.** A decoder whose code table were fixed in its data ROM would have no reason to read a
preamble whose length depends on its content. A preamble read until its own structure ends is how a transmitted code
table looks. **This is consistent with the game sending the table**, and so with 38h being a documented algorithm. It
is not proof. That would need the preamble's format characterised, and then the same coded bits shown to decode
differently under two preambles of equal length. **The DSP-3 stays parked** (plan §10, Q3), with this measurement
recorded for the step that resumes it.

## 47. The NEC DSP replacements, step 14: the close-out, in part (2026-10-04)

The plan's close (§8, step 14) assumes every step done. Steps 5 to 8 (§44.4) and the DSP-4's decision (§45.3) are not
done, so this section closes what can be closed and lists the rest.

- **The clone check.** `clonecheck.py` ran with `dsp2.rs`, `st010.rs` and `dsphle.rs` audited against Mesen, the only
  reference tree on the desktop.
  - Structure: 0 pairs sharing 12 or more fingerprints. Tables: no shared run of 8 or more literals. Names: 37 shared
    identifiers, every one an ordinary word (`command`, `cycles`, `latency`, `mantissa`, `overflow`).
  - Plan §5.4's DSP comparison set (bsnes/higan's and snes9x's DSP and ST010 modules, MAME's, and the superfamicom wiki's
    ST010 code) is not on the desktop. That check is owed.
- **firmwarecheck**, every generated table: the DSP-1's sine (§40.4) and the ST010's sine (§43.2), both forced-PASS
  with residual 0, formula coverage 25.0% and 14.8%. The DSP-2 and the attitude matrices hold no table.
- **The core's stated costs** are set from the measurements: the DSP-2 near exact (§42.4), the ST010 with its named
  loss 05h (§43), and the DSP-1 as §39-§44 graded it.
- **Owed:** `EmuSen_Games_Tested.md`'s VenusRT entries for the DSP games without their images, once the DSP-1 and the
  ST010 run past their first partings. Plan §9's Q2 was already marked superseded at step 1.

## 48. The NEC DSP replacements: the merge with stage 8, ST010 07h's constants, and the DSP-4's limited step (2026-10-04)

### 48.1 The state version, after the merge with stage 8

Stage 8's HDMA work (§41) took state version 20 for two bytes of each line's HDMA run, and §44.2 had taken 20 for the
replacements' layout on this branch. The two meet here as **version 21**.

- **Version 20 means stage 8's.** The DSP branch's own 20 was never merged, so no state of it exists outside this
  branch's tests. It is withdrawn rather than supported.
- **Version 21 states** have both stage 8's HDMA bytes and this branch's replacement layout.
- **Loading.** States of versions 17 to 20 still load on the low-level path, and on cartridges without a NEC DSP.
- **The replacement.** Its state from before 21 is refused with `VERSION`, whichever branch's meaning its 20 has.
  Its layout before 21 is not the present one either way, and the version alone cannot tell the two apart.
- **Tests.** `a_replacement_state_before_version_21_is_refused` holds this. The registration golden was re-recorded,
  and its diff is the four state-version lines alone.
- **After the merge** Dungeon Master is still identical to the image for 3,600 frames. F1 ROC II still parts at
  its first 05h, at frame 1,020 now, since stage 8 moved the 65C816's start by 132 master clocks.

### 48.2 ST010 07h: the fitted constants withdrawn

§43.1 fitted 07h's perspective constants, K = 7,885 and c = 8.8, to the image's 176 words at one angle. However few
bits that takes, it is a value seen from the chip entering the code, and R1 makes it a grade only. No document gives
07h's scale: fullsnes names the command "Raster Data Calculation" and nothing more, and plan §3.5's addresses give the
arrays' places, not their contents. No declared formula was found either. A perspective divisor needs a camera height
and a distance to the screen, and every value tried for them would be a second fit.

**What replaces it.** Every line takes mode 7's unit scale, 100h, 1.0 in the 8.8 matrix format fullsnes documents for
the PPU, so that the four arrays are the rotation by θ at unit scale on every line. The sine and the products are
those of §43.2.

**The larger error, measured** (`dsp_stgrade 07`, 4,000 seeded cases):

- the busy bit's latency and words 0-2 as before, the latter within one;
- the four arrays differ in every case, by up to 640, against the image's scale of 896 on the first line and 43 on
  the last.

The crate test now bounds the arrays at 640. In play, F1 ROC II's road would lose its perspective wherever the game
draws from 07h's arrays. That cannot be seen yet: the game parts at its first 05h, before 07h's arrays reach a
picture. 07h is now an approximation with a documented constant, and its perspective is a named loss until a source
for it is found.

> *Amended 2026-10-04* (§57). The perspective is built from the one L sequence an exact search of a structure declared
> in advance found, under plan §1.3's rule for constants by exact match. The unit scale is withdrawn.

### 48.3 The DSP-4, the limited step (measured 2026-10-04)

*Decided 2026-10-04*: §45.3's option B. One step builds the DSP-4's protocol, 00h, and the short commands, so that
Top Gear 3000 starts without the image. The road commands then go back for a decision with the measurement below.

**Built** in `chips/dsp4.rs`, with `Program::Dsp4`. A Top Gear 3000 cartridge now runs on the replacement with no
file, and core info's cost says what it lacks.

- **The protocol.** 16-bit throughout, the idle word FFFFh, and 20h-FFh mirroring 10h-1Fh (fullsnes). At idle, DR's
  low byte is taken as the next command, a read included, so an S-CPU read at idle runs the mirror of 1Fh, which does
  nothing and writes FFFFh again: the idle polling of §37.2. After a command with inputs the chip offers 0000h, and
  the S-CPU's first input written over it is taken by the next read. 03h, 05h, 0Eh and 15h-1Fh do nothing in their
  measured times.
- **00h**, §45.2's rule: both inputs read by edges, and the low word first.
  - The first result is written 7 cycles after the second input's read whatever the S-CPU has done, 8 when its
    bit 15 is set. That is a timing rule of one feature, exact on 100,000 cases.
  - The second result is handshaken.
- **The short commands' transfers and timing**: 0Ah four inputs and four results, its first unhandshaken at 23; 0Bh
  and 0Ch three and one; 11h four and one, unhandshaken at 6. Their results are not characterised and are 0000h.
- **02h, 06h, 12h, 13h and 14h**: their transfers and timing. 14h gives 0400h (fullsnes), 12h, the memory test, gives
  0000h, 13h's data ROM a named loss in zeros, and 02h and 06h zeros.
- **The road commands 01h, 07h, 08h and 09h**, with 04h, 0Dh, 0Fh and 10h, are not built. Each goes idle 13 cycles
  after its command.

**The grade** (`dsp_grade` with `DSP_GRADE_WRITE_OVER=1`, 20,000 seeded cases a code, half jittered):

| Code | Result |
|---|---|
| 00h | **exact** in values, transfers, SR and latency |
| 03h, 05h, 06h (fresh chip), 0Eh, 12h, 14h, 15h, 1Fh and mirrors | exact |
| 0Ah | transfers exact; values not characterised; latency in 371 (1.9%) |
| 0Bh, 0Ch | transfers exact; latency off by one in 75%, by a rule of the inputs not found |
| 11h | transfers and latency exact; values not characterised |
| 02h, 13h | values 0000h against the chip's |

The crate test `the_dsp4_replacement_agrees_with_the_image` holds these. In WiseMan,
`A_dsp_cartridge_runs_on_the_replacement_with_no_firmware` now takes SD Gundam GX as the cartridge without a
replacement.

**Top Gear 3000 in lockstep** (`dsp_lockstep`, 7,200 frames, `tg3000.txt` with shots).

- **Identical to the image until frame 3,178**, through the boot, the title, the Championship and player menus, the
  track choice and the shop. This is the first parting.
- **The parting** comes at 11h, the first command whose result is not characterised. The first road command follows
  at 3,183.
- **The race.** From then on the picture is black on the replacement. Its S-CPU runs, 534 distinct addresses in a
  frame against the image's 2,321, but it draws nothing: neither road, scenery, cars nor the status bar. With the
  image the same frames show the grid, the countdown and the race.
- **Pictures equal**: 3,232 of 7,199 frames.

`dsp_lockstep` now counts commands as the S-CPU's first write after the chip's idle edge, which every DR chip makes.
The DSP-4 learns no command place from the program counter, so the old count missed its commands, and a script's
`tapuntil ... chip` tapped Start through the race.

**For the decision.** The menus run as with the image. The race needs the road commands. 01h, 07h, 08h and 09h are
78.8% of the game's DR transfers, they are exchanges whose lengths depend on the data, and none is characterised.
Short of them, 11h's and 0Ah's results would keep the run identical a few frames further, to frame 3,183, but would
not draw a race. Building the road commands is plan step 12a, estimated at 4 to 6 steps, with no measurement yet of
whether they are formula-like.

## 49. The NEC DSP replacements, step 7 begun: the DSP-1's Parameter, Raster and Project (2026-10-04)

Step 7 of `VenusRT_DspHle.md` §8, first part. No document read for this work gives an equation for any projection
command: SnesLab names their parameters, fullsnes their codes, and the SNESdev wiki stops at Polar (plan §3.1). The
commands are therefore characterised from the oracle under plan §1.3, then each is stated as a formula with a declared
family, as §44 did for the attitude commands. §49.1 and §49.2 were written and committed before the first comparison
of any member. The sine is a named loss (§40.7), so no member can be exact and every grade below is a closeness grade.

### 49.1 What was characterised, from the oracle

Measured 2026-10-04 with `dsp_oracle ask` and `dsp_oracle chains` (a new mode: one chain of transactions a line, each
from the same idle chip) on the DSP-1B, as real-valued errors of a stated geometry against the image's outputs. A
handful of cases are cited where a rule rests on them (R1).

- **The frame.** The eye E sits Lfe behind the point F along the view's forward vector f, and the screen Les in front
  of the eye. With a = Aas and z = Azs (z the angle from straight down):
  - f = (sin a·sin z, -cos a·sin z, -cos z); the screen's right r = (cos a, sin a, 0); the screen's down
    u = (-sin a·cos z, cos a·cos z, -sin z). §44.1's (-sin a, -cos a) had the horizontal sign of x wrong: at a = 45° the
    centre point lies at +x.
  - E = F - Lfe·f, so the eye's height is Fz + Lfe·cos z. Raster's scale is proportional to Fz + Lfe·cos z within 1 over
    every Fz, Lfe, Les and z tried.
- **Parameter (02h).** Vva = -Les·cot z, the horizon's raster line. (Cx, Cy) is where f meets the ground: E's
  horizontal position moved along (sin a, -cos a) by the eye's height times tan z. Vof is 0 below a limit angle. Over 800
  seeded cases with Azs from 11° to 79° the errors of this geometry are within 3.4 (Vva) and 6 (Cx, Cy).
- **The limit branch.** Above a limit near 79.9°, the same for every Les, Fz and Lfe tried, Vva holds its value at the
  limit, Vof grows as Les·tan(z - limit), and (Cx, Cy) moves on another course. §44.1's Les·(cot θ - cot z) fails at
  z = 112.5°. No document gives the limit (§49.3).
- **Raster (0Ah).** The run gives line Vs first, then Vs + 1, and so on; the values are a function of the line alone,
  whichever line the run starts from. For a line v with the denominator n = Les·cos z + v·sin z and the scale
  k = 256·(eye height)/n:
  - An = k·cos a, Cn = k·sin a, Bn = -k·sin a / cos z, Dn = k·cos a / cos z. Dn/An is 1/cos z on every line.
  - **The denominator is an integer.** Near the horizon An falls as 1, 1/2, 1/3 ... of its first value, and the same
    value repeats every 23 lines at 73°, where sin z is 0.957: the signature of n taken whole. The value at n = 1 is not
    256 times the eye height but one below, and the run's values near n = 1 to 7 follow a 15-bit reciprocal's mantissa
    as Inverse's (§44.1) does.
  - Where n is 0 the line saturates, and on the sky's side (n < 0) the values are negative.
- **Project (06h).** With d = (X, Y, Z) - E: H = Les·(d·r)/(d·f), V = Les·(d·u)/(d·f), and M = 256·Les/(d·f), the
  enlargement in 8.8. Over 800 seeded cases the errors are within 1.4 of this geometry where nothing overflows; past
  16 bits the results saturate.
- **Raster, Project and Target use Parameter's state**, and only that: §37.4's sequences found Raster and Target
  disturbed by the memory test alone.

### 49.2 The families, declared before the first comparison

Every product of two words is the µPD77C25's, floored at 2^-15, as Multiply's member (§39.3). The sine and cosine are
the replacement's (§40.4). Results saturate to 16 bits. The **reciprocal** of a word w is one of S2's two forms below;
the replacement's own Inverse routine (§44.2) normalises w to a mantissa in [4000h, 8000h) with s shifts and takes
2^29 over it, floored, so that 1/w = mantissa·2^(1 + s - 15 - 15).

**Shared choices**, one member for all three commands:

- **S1, the eye's height** Ez = Fz + Lfe·cos z: taken whole (floored), or carried with 15 fraction bits (2).
- **S2, the reciprocal**: the replacement's Inverse routine, or an exact division at full width (2).
- **S3, a quotient's last rounding**: floor, or half up (2).

**Parameter (02h)**:

- Vva = -(Les·cos z)/sin z; Eh = (Fx, Fy) - Lfe·sin z·(sin a, -cos a); (Cx, Cy) = Eh + Ez·(sin z/cos z)·(sin a, -cos a);
  Vof = 0.
- **P1, the horizontal offset**: Lfe·sin z floored, then times sin a and cos a, floored; or the triple product scaled
  once (2).

**Raster (nAh)**:

- k = 256·Ez/n, saturated; An = k·cos a, Cn = k·sin a; K = k/cos z, saturated; Bn = -K·sin a, Dn = K·cos a.
- **R1, the denominator n**: the sum of the two floored products Les·cos z and v·sin z; the floor of their exact sum;
  or unquantised, carried with 15 fraction bits (3).

**Project (06h)**:

- d = (X, Y, Z) - E; x = d·r, y = d·u, w = d·f; H = Les·x/w, V = Les·y/w, M = 256·Les/w.
- **J1, the view's elements** (sin a·sin z and the rest): floored Q15 products, or exact (2).
- **J2, each dot product**: its terms floored and summed, or the exact sum floored once (2).

That is 2·2·2·2·3·2·2 = 192 members, under 8 bits, chosen as one implementation.

**The grade.** A result is *close* when it is within 2 of the image's, or within 1/128 of the image's value where that
is larger: the sine's own error at full scale is 10 in 32,768, and Raster's values near the horizon reach 32,767. The
member chosen is the one with the most close cases over the three commands together, ties broken by the smaller
largest difference. Two sets:

- **Seeded**: 2^16 cases from `Pcg`, Fx and Fy in ±4,096, Fz in 0-1,000, Lfe in 0-1,024, Les in 64-1,024, any Aas, Azs in
  0800h-3800h (11° to 79°, below the limit); a point within 500 of (Cx, Cy) at a height of 0-200 for Project; a line
  between the horizon and +112 for Raster.
- **Traced**: every Parameter, Raster and Project the seven DSP-1 titles give in §37.6's 7,200 frames, their inputs
  replayed on both engines in order (plan §4.2's trace oracle). Inputs only are taken from the traces.

**Timing.** Parameter's, Project's and a raster line's phases that vary take their medians, as Triangle's did
(§40.4). A rule of one feature may replace a median: the declared feature is the reciprocal's normalisation shift, as
Inverse's time follows it (§44.1).

### 49.3 The limit branch, a named loss

No document gives the limit angle, nor the way Vof and (Cx, Cy) behave past it. It could be read from the image to
within one step, but that is a constant taken from the chip, which R1 forbids (§48.2 withdrew ST010 07h's for the same
reason). **The replacement therefore has no limit**: past 79.9° it keeps the geometry of §49.1, with Vof 0 and the
horizon where Les·cot z puts it. That is a named loss, measured in the grade and in the games. The traces say what it
costs: Lock On's and Ballz 3D's every Parameter is past the limit, and 190 of Pilotwings' 600; Super Mario Kart's,
Suzuka 8 Hours', Michael Andretti's Indy Car Challenge's and Super Bases Loaded 2's never are. Whether to admit the
one threshold, as §40.6 admitted a second widening, is the tester's decision, put in §49 after the games are measured.

### 49.4 Project's family widened once, in writing (plan §5.2, rule 4)

The first grading (`dsp_projection`, 2^16 seeded cases and the seven traces) left Parameter and Raster close
wherever the limit branch is not reached. Two corrections to the grader came first, neither a model change: a traced
Raster run's last line is where the game writes its terminator over the results, so it is not graded; and Pilotwings'
Parameter failures are all past the limit (§49.3).

**Project was not.** Its best member was close in 98% of Super Mario Kart's results but 67% of Michael Andretti's
Indy Car Challenge's, whose eye sits 24 units behind F at a height near 10. The errors have structure:

- The image's M is 256·Les over a whole number. In that game's cases (Les = 96) it is 24,576/w for w = 23, 11, 8, 4,
  2 and -1, so w = d·f is taken whole before the reciprocal. No member of §49.2 does that while carrying the eye's
  height with its fraction.
- F itself projects to V = 0 on the image, and to V = -4 with the eye's height taken whole (9.98 floored to 9). The
  image's H for F is 0 or ±3 as Aas changes by 40 steps, which a whole horizontal eye position gives.

The family for Project is widened once, by two choices, before they are graded:

- **J3, Project's eye**, independently of S1 and P1: its height whole or with 15 fraction bits, and its horizontal
  position whole (P1's) or with 15 fraction bits, from F - Lfe·f with f's elements as J1 gives them (4).
- **J4, w**: taken whole (floored) before the reciprocal, or not (2).

With J1 and J2, Project has 32 members of its own, and the whole family 48·32 = 1,536 members, about 10.6 bits, still
chosen as one implementation. The member chosen is the one with the most close results over both sets together. If no
member is close on nearly all of a game's traced Project results below the limit, the remainder is recorded as the
sine's and this structure's joint error, with no further widening.

### 49.5 Chosen, built, and the grade (measured 2026-10-04)

**The choice.** `dsp_projection dsp1b 65536` with the seven traces graded all 1,536 members. By §49.2's rule, the most
close results over both sets, the member is:

| Choice | Member |
|---|---|
| S1, the eye's height (Parameter, Raster) | whole: Fz + Lfe·cos z, floored |
| S2, the reciprocal | the Inverse routine's 15-bit mantissa |
| S3, the last rounding | half up |
| P1, the horizontal offset | the triple product Lfe·sin z·sin a scaled once |
| R1, Raster's denominator | the sum of the two floored products: n = ⌊Les·cos z⌋ + ⌊v·sin z⌋ |
| J1, J2 | the view's elements exact, each dot product's sum floored once |
| J3, Project's eye | height whole, horizontal position with its fraction |
| J4 | w not taken whole |

**Measured against it.** The member ranked 69th by the rule, 1,954 close results behind in 2.1 million, takes the
exact division floored, the elements floored, the eye with both fractions and w whole. It is exact far more often: in
Super Mario Kart's traced Project 97% against 52%, and in the seeded Project 83% against 51%; and the seeded set alone
on the DSP-1 ranks it first. It loses the count on Super Mario Kart's and Suzuka 8 Hours' Raster lines and Lock On's
Project. The rule was declared, so the first member is built. The other's exact share says the chip's own sequence
lies nearer to it, and §49.6 records it with §49.4's structure for a later decision.

**Built** in `dsphle.rs`, as `Projection`, from Parameter's seven inputs, which the replacement keeps:

- **Parameter** gives Vva, (Cx, Cy) and Vof = 0, by §49.2's formulas with the member above. It has no limit (§49.3).
- **Raster** gives line Vs and then Vs + 1, Vs + 2 and so on for as long as the S-CPU reads.
  - **The run ends when DR no longer holds a line's last result.** Writing another word over the fourth result ends it:
    measured on the image with the driver's `write_over`, a write over the first, second or third result goes on, and
    a write of the same value as the fourth is taken as a read. §38's frame ended on any write; that was wrong.
  - Bench games write a terminator over the fourth result, so the rule only matters for writes elsewhere.
- **Project** gives H, V and M.
- **Timing**, from `DSP_PROJ_TIMING` over 4,096 seeded cases after a Parameter:
  - Parameter: work 527 and notice 485 for its last input (ranges 508-555 and 464-515);
  - Project: notice 362 for its last input (328-404);
  - Raster: the first line's first result at work 128 and notice 116, each later line's at notice 115 (109-129), and
    idle 6 cycles after the terminator.
  - Each is the median. The declared feature, the reciprocal's normalisation shift, does not decide Project's time
    alone: over 1,500 cases one shift count spans 40 cycles. The medians stay.
- **State version 22.** The DSP-1 replacement's state gains Parameter's seven inputs. A DSP-1 or DSP-1B replacement's
  state of version 21 is refused with `VERSION`; the DSP-2's, DSP-4's and ST010's still load, since their layout did not
  change. The low-level path's states of 17-21 load as before. Test: `a_dsp1_replacement_state_before_version_22_is_refused`.
- **No new table.** The commands use the sine of §40.4 and the Inverse routine's normalisation, so firmwarecheck's counts
  are §40.4's.

**The grade through the ports** (`DSP_PROJ_PORTS=1 dsp_projection`, 65,536 seeded cases, every other one under a
jittered S-CPU; values, transfers, SR and latency compared; the DSP-1 and DSP-1B alike):

| Command | All results close | All exact | Transfers or SR differ | Latency differs | Largest difference |
|---|---|---|---|---|---|
| 02h Parameter | 65,301 (99.6%) | 3,997 (6.1%) | 0 | 56,948 | 6 |
| 0Ah Raster, two lines | 65,434 (99.8%) | 10,733 (16.4%) | 0 | 49,578 | 4,237 (a line beside the horizon) |
| 06h Project | 64,587 (98.6%) | 9,151 (14.0%) | 0 | 60,501 | wraps at 16 bits |

**The traced shares**, results close of results graded, from the chosen member's row:

| Game | Parameter | Raster | Project |
|---|---|---|---|
| Super Mario Kart | 27,124 of 27,124 | 49,600 of 49,600 | 90,064 of 91,200 (98.8%) |
| Suzuka 8 Hours | all | all | all |
| Super Bases Loaded 2 | all | all | 40,891 of 40,893 |
| Michael Andretti's Indy Car Challenge | all | all | 25,529 of 31,086 (82.1%) |
| Ballz 3D | 978 of 5,588 (past the limit) | none given | all |
| Pilotwings | 1,845 of 2,400 (past the limit) | 358,306 of 422,416 (84.8%) | 8,621 of 8,955 (96.3%) |
| Lock On | 5,497 of 43,872 (past the limit) | 57,699 of 2,647,288 (2.2%) | 74,331 of 75,588 (98.3%) |

Every Parameter result that is not close lies past the limit. The four games that never pass it are close in every
Parameter and Raster result. Michael Andretti's Indy Car Challenge's Project misses are the eye's own rounding (§49.4):
F itself is drawn at an enlargement of 1,024 against the image's 1,068.

> *Amended 2026-10-04* (measured, §52.2). The figure of 1,024 was another member's. The member built here draws F at
> 1,042 and 4 lines low, against the image's 1,068 on its line.

The crate test `the_replacements_projection_stays_close_to_the_image` holds the shares on both images, and
`a_raster_run_ends_on_a_write` the run's end.

### 49.6 What is open

- **The limit branch** (§49.3): Lock On's and Ballz 3D's views and a third of Pilotwings' are past it. The cost in play is
  measured in §51.
- **Project's eye and the choice.** §49.4's characterisation found the eye at F - ⌊Lfe·f⌋ coordinate by coordinate,
  with w floored, which is not in the family; §49.4 allowed no further widening. The 69th member (§49.5), which is in
  it, is exact twice as often as the one the rule chose. Whether to choose by exact share instead, or to admit the
  coordinate-wise eye, is a decision for the tester.
- **Latency.** The medians leave most cases a few cycles off. Exact timing needs the reciprocal's sequence, which the
  sine's loss already puts out of reach of exact values.

## 50. The NEC DSP replacements, step 7 continued: the DSP-1's Target and Gyrate (2026-10-04)

§50.1 and §50.2 were written and committed before the first comparison of any member.

### 50.1 What was characterised, from the oracle

Measured 2026-10-04 with `dsp_oracle chains` on the DSP-1B, as errors of a stated formula against the image.

- **Target (0Eh)** gives the ground point under a screen position (H, V), as SnesLab says.
  - At Aas = 0 it is §49.1's geometry: the ray from the eye through (H, V) on the screen, met with the ground.
  - **Its H axis turns the other way from Project's.** Moving H moves the ground point along (cos a, -sin a), where
    Project's right is (cos a, sin a): at a = 90° the two are opposite, and at 45° Target's H runs along the view.
    Measured at a = 0, 45°, 90°, 135° and 270°.
  - **Target is Raster's matrix applied to (H, V).** With Raster's line-V results An, Bn, Cn, Dn from the same Parameter,
    the image's (X, Y) is (Cx + (H·An + V·Bn)/256, Cy + (V·Dn - H·Cn)/256) within 2 in 594 of 600 seeded cases. The
    six others are where the line's values saturate. The geometry agrees: the ground's offset from C for line V is
    -V·Ez/(cos z·n), which is -V·K/256 in §49.2's terms, and its lateral scale is k/256.
- **Gyrate (14h)** takes three angles and three turns and gives three angles. With the inputs (Az, Ax, Ay, U, F, L):
  - Az' = Az + (U·cos Ay - F·sin Ay)/cos Ax;
  - Ax' = Ax + U·sin Ay + F·cos Ay;
  - Ay' = Ay + L - (U·cos Ay + F·sin Ay)·tan Ax.
  - Over 600 seeded cases each result is within 2 of this, its median error -0.5 to -1, except near Ax = ±90°, where
    1/cos Ax saturates. The sign of F in Ay' is not the one that Az' has. These are the Euler-angle rates for turns
    about the body's axes, as fullsnes's name, "3D Angle Rotation", suggests.

### 50.2 The families, declared before the first comparison

Products and the sine as §49.2. Target uses Parameter's state and §49.5's member for Cx, Cy and Raster's k and K.

**Target (0Eh)**: X = Cx + (H·k·cos a - V·K·sin a)/256, Y = Cy + (V·K·cos a - H·k·sin a)/256, with k and K Raster's for
line V.

- **T1, k and K**: saturated to 16 bits as Raster gives them, or carried at full width (2).
- **T2, the products**: through Raster's floored An-Dn, then times H or V; or each triple product scaled once (2).
- **T3, the division by 256**: floor, or half up (2).

That is 8 members, 3 bits.

**Gyrate (14h)**: §50.1's three equations, each increment saturated to 16 bits and added to its angle with wrapping.

- **G1, 1/cos Ax and tan Ax**: through the Inverse routine on cos Ax, tan Ax as sin Ax times it; or exact division (2).
- **G2, each sum**: its terms floored and summed, or the sum scaled once (2).
- **G3, Az's quotient**: the bracket formed and then divided, or each term divided (2).
- **G4, a quotient's rounding**: floor, or half up (2).

That is 16 members, 4 bits.

**The grade**, as §49.2's: a result is close within 2 or 1/128 of the image's value; the member with the most close
results wins, ties to the smaller largest difference. Target is graded over 2^16 seeded cases (§49.2's Parameter
ranges, H in ±128 and V from the horizon's line plus 2 to +112) and the traced Target commands of Pilotwings, the only
bench game that gives it (§37.6). Gyrate is graded over 2^16 seeded cases (any Az and Ay, Ax in ±75°, turns in ±1,024)
and Pilotwings' traced Gyrates. Timing as §49.2: medians of the phases that vary.

### 50.3 Chosen, built, and the grade (measured 2026-10-04)

`dsp_target_gyrate dsp1b 65536` with Pilotwings' trace graded both families. One grader defect was found and corrected
before the choice, not a model change: tan Ax had been held to a word, so past 45° it saturated at 1, and Gyrate's
Ay' missed by hundreds. It is now carried at full width, as §50.2's "tan Ax as sin Ax times it" requires.

| Command | Member chosen | Seeded results close | Traced results close |
|---|---|---|---|
| 0Eh Target | T1 at full width, T2 each triple product scaled once, T3 floor | 128,983 of 131,072 (98.4%) | 995 of 1,202 (82.8%); every miss past the limit |
| 14h Gyrate | G1 the Inverse routine, G2 each sum scaled once, G3 the bracket divided, G4 floor | 193,975 of 196,608 (98.7%), largest 6 | 1,257 of 1,257, largest 1 |

**Built** in `dsphle.rs`: Target as `Projection::target`, which applies Raster's line-V scale to (H, V) at full width
and adds (Cx, Cy); Gyrate as `gyrate`. 1Eh, 2Eh and 3Eh take Target's transfers, as §37.5 measured them alike.
**Timing**, the medians over 4,096 seeded cases (`DSP_PROJ_TIMING`): Target's last input at notice 116 (110-132),
Raster's own line time; Gyrate's last input at work 237 (205-273) and notice 3.

**The grade through the ports** (`DSP_TG_PORTS=1 dsp_target_gyrate`, 65,536 seeded cases, half under a jittered S-CPU;
the DSP-1 and DSP-1B alike):

| Command | All results close | All exact | Transfers or SR differ | Latency differs | Largest difference |
|---|---|---|---|---|---|
| 0Eh Target | 63,994 (97.6%) | 3,138 (4.8%) | 0 | 49,159 | saturates beside the horizon |
| 14h Gyrate | 62,928 (96.0%) | 4,596 (7.0%) | 0 | 57,329 | 6 |

Gyrate's misses are of 3 to 6, where Ax is past 60° and tan Ax magnifies the sine's error. The crate test
`the_replacements_target_and_gyrate_stay_close_to_the_image` holds these shares on both images.

**Step 7 is built.** Every command the bench games give the DSP-1 is now computed: exactly for 00h, 0Fh and 2Fh, and
approximately for the rest on the named-loss sine. What remains is the limit branch (§49.3) and the DSP-1's own 28h
bug (§44.1).

## 51. The NEC DSP replacements, step 8: the seven DSP-1 titles in lockstep (2026-10-04)

Step 8 of `VenusRT_DspHle.md` §8, with every command the games give now computed (§49, §50). P6 was retired at §44.3;
the question here is the one the sine's loss leaves: whether the games play acceptably, and what their pictures cost.

### 51.1 What was added to measure it

- **`dsp_lockstep`** reports the visual error as well as the first parting:
  - per frame, the share of pixels that differ;
  - for sprites alike in tile and attributes on both machines, in frames whose pictures differ, the larger of their x
    and y distances, which is where Project placed them;
  - `LOCKSTEP_SHOTS=f1,f2,...` saves the image's picture, the replacement's and their differing pixels side by side as a
    PNG in the probe cache's `shots/` folder, never in the repository;
  - `LOCKSTEP_OTHER=<stem>` runs a second image in the replacement's place, and `LOCKSTEP_AS=<stem>` runs both machines as
    another program of the same slot (`Machine::attach_replacement_as`); `LOCKSTEP_OTHER_CLOCK=<per mille>` runs that
    second image's clock faster or slower, so that only its timing differs.
- **`dsp_replay <chip> <trace>`**, plan §4.2's trace oracle for the whole DSP-1: every transaction of a game's trace
  replayed on the replacement in order and compared with the image's recorded results, per command.
- **`examples/dsp_pads/smk_race.txt`**: Super Mario Kart through its menus into a 50cc Mushroom Cup race as Mario, then
  accelerating and steering.

### 51.2 The seven titles (measured 2026-10-04)

`dsp_lockstep <rom> 7200` with §37.6's pad scripts. "Placed within 1" is the share of sprites alike on both machines
whose positions differ by at most one pixel. The pixel share counts any difference, so a mode 7 floor sampled one
texel over counts whole, though it looks the same.

| Game | Identical until | Pictures equal (§44.3, of 3,600) | Pixels differing, median and 95th percentile | Sprites placed within 1 (within 4) |
|---|---|---|---|---|
| Super Mario Kart | 96 | 3,893 of 7,200 (258) | 0.00%, 21.2% | 98.1% (99.1%) of 56,445 |
| Super Mario Kart, `smk_race.txt`, 3,014 frames | 96 | 1,289 | 8.65%, 28.1% | 99.2% (99.8%) of 41,034 |
| Michael Andretti's Indy Car Challenge | 3,310 | 3,310 of 7,193 (3,310) | 7.66%, 7.83% | 80.9% (99.96%) of 126,880 |
| Suzuka 8 Hours | 1,474 | 1,499 of 7,195 (1,491) | 5.59%, 7.47% | 100.0% of 58,728 |
| Super Bases Loaded 2 | 3,029 | 3,054 of 7,198 (3,030) | 56.3%, 98.7% | 93.8% (95.9%) of 136,414 |
| Ballz 3D | 762 | 2,528 of 7,200 (1,007) | 0.46%, 77.4% | 69.7% (77.5%) of 43,565 |
| Pilotwings | 1,799 | 1,802 of 7,200 (1,802) | 69.3%, 100% | 90.4% (92.8%) of 76,232 |
| Lock On | 1,523 | 1,525 of 7,198 (1,525) | 37.3%, 99.9% | 98.0% (98.5%) of 638,697 |

Each still parts in the frame after its first command that computes, as the sine's loss requires. **The trace oracle**
(`dsp_replay`) agrees with the ports' grades: in Super Mario Kart's trace every Parameter, Raster, Triangle and
Distance transaction is close, and 29,935 of 30,400 Projects; in Pilotwings', every Attitude, Subjective, Objective,
Rotate and Gyrate transaction is close, while Parameter, Raster and Target miss where the limit is passed and Distance
in 198 of 419, the DSP-1's own bug (§44.1).

### 51.3 Playable or not, game by game

The pictures named are in `~/.cache/emusen/probe/venusrt/dsp-hle/shots/`.

- **Super Mario Kart: playable.** In the attract race (`smk-f2400.png`) and in a race driven by `smk_race.txt`
  (`smk-race/`), the track, the karts on it, Lakitu, the map below and the race's course are as on the image. Sprites
  sit within a pixel of the image's in 98-99% of cases and within four in 99.1-99.8%. The race keeps its course: at
  every frame shown the karts stand where they stand on the image's map. What differs is the floor's texture sampling
  and the odd sprite a pixel off, which no player would see.
- **Suzuka 8 Hours, Michael Andretti's Indy Car Challenge, Super Bases Loaded 2: playable.** Their roads and field are
  drawn as on the image (`suzuka-f3000.png`, `andretti-f6500.png`, `sbl2-f3500.png`). Andretti's own car sits 4%
  larger and a few pixels off where Project's eye rounding (§49.4) matters, at the eye's height of 10.
- **Ballz 3D: plays, and its attract fight diverges.** The arena floor is the image's. The fighters are built of their
  balls correctly, but the fight takes another course after a few seconds (`ballz-f5000.png`), since Subjective's and
  Project's small differences feed the game's own animation.
- **Pilotwings: not acceptable as it stands.**
  - Below the limit angle the views are the image's (`pilotwings-f2000.png`, `-f2400.png`).
  - In level flight, past the limit, the ground is drawn in the wrong place. At frame 2,120 (`pilotwings-f2120.png`)
    the image shows the island below the glider and the replacement shows sea; 190 of the game's 600 traced Parameters
    are past the limit (§49.3).
  - The demo flights diverge from about frame 2,400: at frame 3,500 the image's glider is at 529 feet and climbing, the
    replacement's at 126 near the runway. This is not the DSP-1's Distance bug: with both machines run as the DSP-1B
    (`LOCKSTEP_AS=dsp1b`, `pilotwings-as-dsp1b/`) the demo diverges alike, while the DSP-1B's image against the
    DSP-1's stays within 3 feet for 53 seconds (`pilotwings-dsp1-vs-dsp1b/`). The demo replays its pad frame by frame,
    so the flight model integrates every small difference. **The divergence is the values', not the timing's:** with
    the image's own clock run 2% faster and 2% slower against itself (`LOCKSTEP_OTHER=dsp1 LOCKSTEP_OTHER_CLOCK=1020`
    and `980`), so that only the chip's timing moves, every picture is equal for 5,010 frames and the states meet
    again by frame 2,342. The flight model's commands (Attitude, Subjective, Objective, Gyrate, Rotate) are close to
    the image's in every traced transaction but exact in 57-99%, and their differences of 1 or 2 accumulate. A flight
    the player flies is therefore not the image's flight to the foot, though it looks and handles alike; a recorded
    demo does not replay.
- **Lock On: not acceptable as it stands.** Its view is always level, Azs 90°, past the limit. With no limit, Raster's
  K = k/cos z saturates there, and the ground under the aircraft is drawn as streaks (`lockon-f2000.png`). The chip's
  limit exists, in effect, to keep cos z away from zero.

### 51.4 Against the plan, and what is put for decision

- **P1**, at least eleven of the DSP-1B's seventeen command families bit-exact: retired, false. Seven codes are exact
  (00h, 20h, 08h, 18h, 38h, 0Fh and 2Fh), and every command built on the sine is approximate, as §40.7 made certain.
- **P6** stays retired (§44.3).
- **The limit branch is the one defect a player sees** in the DSP-1's replacement: Pilotwings' level flight, Lock On
  throughout. Fixing it needs one constant, the limit angle, which no document read gives, and the branch's behaviour
  past it, which §49.1 began to characterise: Vva held at the limit's and Vof = Les·tan(z - limit). Measured since
  (`dsp_oracle chains`, Les 256, z from 81.6° to 91.4°): past the limit Raster keeps the limit's horizon and
  denominator on every line, while its scale follows the eye's height at the true z (An at line 0 falls with
  Fz + Lfe·cos z, 1,115 to 1,002); the game scrolls the picture by Vof. (Cx, Cy), Project and Target there are still to
  be measured. Taking the constant from the image is what R1 forbids (§48.2).
  **The decision is the tester's:** admit the limit angle as one measured constant under a written amendment, as §40.6
  admitted the sine's second widening; or leave the branch a named loss, with Lock On and Pilotwings' level flight
  needing the player's image.
- **Project's member** (§49.6): choose by exact share, or admit the coordinate-wise eye. Either would bring Michael
  Andretti's Indy Car Challenge's car to the image's size.

## 52. The NEC DSP replacements: the limit angle admitted and Project chosen by exact share (2026-10-04)

### 52.1 The two decisions, and the amendments they make

*Decided 2026-10-04*, on §51.4's questions.

- **The limit angle is admitted.** R1 (plan §1.3) gains one exception: the DSP-1's limit angle on Azs, measured from
  the image as a latency is, with the method and the precision recorded (§52.3). It is the only constant admitted.
  The branch past it, Vof = Les·tan(z - limit), (Cx, Cy), Raster, Project and Target there, follows the ordinary rules:
  formulas characterised from the oracle, families declared before they are graded. The plan's §1.3 carries the
  amendment.
- **Project is chosen by exact share.** §49.2's rule chose by close results, and §49.5 found the member ranked 69th by
  it exact twice as often. The family is now ranked by exact results over both sets together. Since the family's
  shared choices S1-S3 make it one implementation, the choice is of the whole member, and it changes Parameter's and
  Raster's arithmetic too; §52.2 grades all three. Target's family, graded on §49.5's member, is graded again on the new
  one.
- **The coordinate-wise eye** of §49.4 (E = F - ⌊Lfe·f⌋ coordinate by coordinate, w floored) is not added: §49.4's one
  widening is used. It stays an open lead.

### 52.2 Project, Parameter and Raster by exact share (measured 2026-10-04)

`DSP_PROJ_BY=exact dsp_projection dsp1b 65536` with the seven traces ranked the 1,536 members by exact results over
both sets. **The first is §49.5's 69th**, with 1,586,842 exact results against the old member's 1,278,903 (ranked
196th): S1 the eye's height whole for Parameter and Raster, S2 exact division, S3 floor, P1 the triple product scaled
once, R1 the sum of floors, J1 the view's elements floored, J2 each sum floored once, J3 Project's eye with both its
fractions, J4 w taken whole. It is the member exact in 97% of Super Mario Kart's traced Project results.

Target's family was graded again on it (`dsp_target_gyrate`, the same sets): the same member wins by close and by exact
results, T1 at full width, T2 each triple product scaled once, T3 floor; 129,194 of 131,072 seeded results close,
32,021 exact. Gyrate does not use the projection and is unchanged.

**Through the ports** (`DSP_PROJ_PORTS=1`, `DSP_TG_PORTS=1`, 65,536 seeded cases, half jittered):

| Command | All close, before | after | All exact, before | after |
|---|---|---|---|---|
| 02h Parameter | 65,301 | 65,296 | 3,997 | 1,679 |
| 0Ah Raster, two lines | 65,434 | 65,397 | 10,733 | 13,310 |
| 06h Project | 64,587 | 64,724 | 9,151 (14.0%) | 41,042 (62.6%) |
| 0Eh Target | 63,994 | 64,202 | 3,138 | 2,202 |

**On the traces.** Super Mario Kart's Project is exact in 28,178 of 30,400 transactions and close in 30,330. Its Raster
lines lose closeness where the old member had it: 48,340 of 49,600 close (97.5%, from 100%) though 34,553 exact (from
30,706); the misses are Bn and Dn off by 3 in 300, the floored k divided again. Michael Andretti's Indy Car Challenge's
Project is close in 28,185 of 31,086 results (90.7%, from 82.1%).

**Andretti's car.** F itself, about 1,280 of the game's 10,362 Projects, is now drawn on the image's line (V = 0, from
-4) but at an enlargement of 1,024 against the image's 1,068, from 1,042: 4.1% small, from 2.4%. The difference is w,
24 here against the chip's 23, which is the coordinate-wise eye (§52.1), not this family.

**The seven titles in lockstep** (`dsp_lockstep`, 7,200 frames, as §51.2), before and after:

| Game | Pictures equal | Sprites placed alike | Within 1 | Within 2 |
|---|---|---|---|---|
| Super Mario Kart | 3,893, unchanged | 72.8% to 95.6% | 98.1% to 98.5% | 98.6% to 98.7% |
| Super Mario Kart, `smk_race.txt` | 1,289, unchanged | 78.0% to 97.1% | 99.2% to 99.3% | 99.5% to 99.5% |
| Michael Andretti's Indy Car Challenge | 3,310, unchanged | 79.7% to 75.3% | 80.9% to 94.4% | 86.6% to 99.7% |
| Suzuka 8 Hours | 1,499, unchanged | 95.5% to 100% | 100% | 100% |
| Super Bases Loaded 2 | 3,054 to 3,191 | 72.6% to 80.8% | 93.8% to 95.4% | 94.2% to 95.9% |
| Ballz 3D | 2,528 to 2,607 | 13.9% to 43.0% | 69.7% to 76.4% | 76.7% to 77.5% |
| Pilotwings | 1,802, unchanged | 86.2% to 87.4% | 90.4% to 90.5% | 91.9% to 92.0% |
| Lock On | 1,525, unchanged | 97.9% to 97.9% | 98.0% to 98.0% | 98.3% to 98.4% |

The percentages are of sprites alike on both machines in frames that differ (§51.1). The pixel shares fall in the
racing games: Suzuka 8 Hours' median from 5.6% to 3.8%, Andretti's from 7.7% to 7.2%, and the race's from 8.7% to 4.1%.

### 52.3 The limit angle, measured, and the branch characterised

**The constant** (measured 2026-10-04, `dsp_oracle chains` on the DSP-1B; the DSP-1 alike). Parameter was given Fx = Fy
= 0, Fz = 100, Lfe = 64, Aas = 0 and Les = 7FFFh, 4000h and 1000h, with Azs stepped by one from 38B0h to 38F0h.

- Vva changes at every step up to 38CDh, by about 3 a step at Les = 7FFFh, and from 38CEh on it holds: -5,847 at
  7FFFh, -2,924 at 4000h, -731 at 1000h. Vof is 0 up to 38CDh and grows from 38CEh.
- The held value is -Les·cot(38CEh) within one unit at Les = 7FFFh, where one step of Azs moves Vva by 3. So the
  angle at which Vva is held is 38CEh to within a third of a step.
- §49.1 found the same threshold at every Les, Fz and Lfe tried from 40h to 400h.

**The limit is 38CEh, 79.882°, to within one step of Azs (2^-16 of a turn, 0.0055°)**, the threshold at or above which
the branch is taken. It is the one constant §52.1 admits. A mirrored threshold sits at C732h, its negative: from 8000h
to C732h Vva holds +Les·cot(38CEh) and Vof is negative, and C800h is ordinary again.

**The branch**, characterised as real-valued errors of stated formulas against the image (300 to 400 seeded cases each,
Azs from 3900h to 4200h; Les from 64 to 512). With z_v the limit, Δ = Azs - z_v, and the eye placed by Azs itself:

- **Vof** = Les·tan Δ: within 1 to 3 up to 95°, 5,845 against 5,848 at 90° with Les = 7FFFh. Past 100° the chip's value
  departs (20,668 against 20,970 at 112.5°), and from 135° it follows another course.
- **Vva** is not held exactly. It is -Les·cot z_v / cos Δ: -5,938 at 90° against 5,847.6/cos 10.12° = 5,940, and
  -6,921 at 112.5° against 6,943. So the virtual screen stands at Les/cos Δ.
- **(Cx, Cy)** is where the limit's axis meets the ground from the eye at Azs: E as §49.1 places it for Azs, then E's
  height times tan z_v along (sin a, -cos a). Within about 1%.
- **Raster** keeps the limit's horizon on every line, and its scale follows the eye's height at Azs: 1.4 to 1.7% median
  relative error with the screen at Les or Les/cos Δ, no line offset. Shifting the lines by Vof either way is 28% or
  worse. Dn/An is cos z_v times about 1/cos Δ.
- **Project is untouched by the limit.** It is §49.1's projection at Azs itself, within 1 of the image's results; the
  virtual camera with V moved by Vof comes close only because the two nearly agree.
- **Target** is the virtual camera's: the ray from the eye at Azs through (H, V) on the screen at the limit, V not moved,
  within 1% median. The image's Target no longer equals C plus Raster's matrix (§50.1) here, which fits Raster's Dn/An
  departing from cos z_v while Target's ground offset keeps it.

### 52.4 The branch's family, declared before the first comparison

For lim ≤ Azs < 8000h, with z_v = 38CEh (and for 8000h ≤ Azs ≤ C732h, z_v = C732h), Δ = Azs - z_v, sine and arithmetic
as §52.2's member:

- **Vof** = Les·sin Δ/cos Δ. **B1, its rounding**: floor, or half up (2).
- **Vva** = -Les_v·cos z_v/sin z_v. **B2, the screen Les_v**: Les, or Les/cos Δ, also used by Raster and Target (2).
- **(Cx, Cy)**: §52.2's Parameter with the eye from Azs and t = Ez(Azs)·sin z_v/cos z_v.
- **Raster**: n = ⌊Les_v·cos z_v⌋ + ⌊v·sin z_v⌋, k = 256·Ez(Azs)/n. **B3, K**: k/cos z_v, or k·cos Δ/cos z_v (2).
- **Target**: §50.3's formula with the branch's (Cx, Cy), n and k, and K = k/cos z_v, the virtual camera's ground offset.
- **Project. B4**: §52.2's Project at Azs, with no branch; or the virtual camera at z_v with Vof added to V (2).

That is 16 members, 4 bits, beside the one constant. They are ranked as §52.1 ranks Project, by exact results over both
sets, ties to the most close. The sets: 2^16 seeded cases in §49.2's ranges with Azs from 38CEh to 4800h (101°), and the
traced Parameter, Raster, Project and Target commands of Lock On, Ballz 3D and Pilotwings past the limit. Past 4800h,
and past B800h on the negative side, the same formulas run ungraded, a named loss.

### 52.5 The branch's family widened once, in writing (plan §5.2, rule 4)

`dsp_limit dsp1b 65536` with the three traces graded §52.4's 16 members. By exact results the first is: Vof half up,
the screen at Les/cos Δ, K times cos Δ, and Project with no branch. Built that way, Pilotwings' frame 2,120 draws the
island under the glider as the image does. **Lock On draws no ground at all**: sea-blue from the horizon down.

The cause was found by one measured difference. Lock On's Parameter (Les = 256, Azs = 4000h) gives Vva = -46 on the
image and -47 on the member, -46.4 floored. Built for this measurement alone with Vva rounded toward zero, and not kept,
Lock On draws its ground at frame 1,600 as the image does. The game's own horizon test turns on Vva's last unit. The
image's Vva past the limit lies nearer zero than the floored formula elsewhere too: -5,938 against -5,940 at Les =
7FFFh and 90° (§52.3).

**The family is widened once**, by one choice, before it is graded:

- **B5, Vva's rounding past the limit**: floor, as §52.2's member rounds quotients; or toward zero (2).

That is 32 members, 5 bits, beside the constant. The rank is §52.4's. No further widening follows.

### 52.6 The branch chosen, built, and the grade (measured 2026-10-04)

**The choice.** `dsp_limit dsp1b 65536` with the traces of Lock On, Ballz 3D and Pilotwings ranked the 32 members by exact
results. The first keeps **B5 at floor**: Vof half up, the screen at Les/cos Δ, K times cos Δ, Project with no branch,
Vva floored. It has 1,781,381 exact results. The member with Vva toward zero has 1,766,964, though it has more close
results (3,649,852 against 3,649,824). It gains in Lock On's Parameter (30,652 exact against 19,684) and Ballz 3D's
(5,082 against 3,685), and loses more in the seeded set's (44,436 against 71,187). By the declared rank the floored
member is built.

**Built** in `dsphle.rs`. `Projection` takes the branch when lim ≤ Azs < 8000h or 8000h ≤ Azs ≤ C732h:

- the view is placed at the limit, or its negative, and the eye at Azs;
- Vof = Les·tan Δ, half up;
- Vva and Raster's denominator use the screen at Les/cos Δ;
- Raster's K is k·cos Δ/cos z_v;
- Target uses the virtual camera;
- Project does not change.

Below the limit every result is as §52.2's. **Timing** past the limit, the medians of 4,096 seeded cases
(`DSP_PROJ_AZS=38CE-4800 DSP_PROJ_TIMING`): Parameter's last input at work 541 and notice 499 (506-581, 464-539),
Project's at notice 354, Target's at 118, and a raster line's first result at 130/118 and later ones at 117. No table
was added; the constant is `LIMIT`. The state did not change, so its version stays 22.

**Through the ports** (`DSP_LIMIT_PORTS=1 dsp_limit`, 65,536 seeded cases with Azs from 38CEh to 4800h, half jittered;
the DSP-1 and DSP-1B alike):

| Command | All results close | All exact | Transfers or SR differ |
|---|---|---|---|
| 02h Parameter | 40,714 (62.1%) | 169 | 0 |
| 0Ah Raster, two lines | 32,362 (49.4%) | 304 | 0 |
| 06h Project | 64,468 (98.4%) | 47,574 (72.6%) | 0 |
| 0Eh Target | 40,505 (61.8%) | 611 | 0 |

Without the limit (§49.3), none of these but Project came near the image. The misses left are of 1 to 2%: Bn and Dn
about 1.4% large, and (Cx, Cy) 5 to 20 off at distances of thousands. Below the limit the ports' grades are unchanged
to the case. **On the traces**:

| Game | Parameter | Raster |
|---|---|---|
| Lock On | 43,752 of 43,872 close | 2,560,736 of 2,647,288 lines (96.7%, from 2.2%) |
| Ballz 3D | 5,588 of 5,588 | none given |
| Pilotwings, past the limit | 567 of 756 | 104,914 of 133,056 lines |

Pilotwings' Target past the limit is close in 195 of 378. The crate test
`the_replacements_branch_past_the_limit_stays_close_to_the_image` holds the ports' shares on both images.

**In lockstep** (7,200 frames, the pictures in `~/.cache/emusen/probe/venusrt/dsp-hle/shots/`; those before the limit
was built in `shots/before-limit/`):

- **Pilotwings' level flight is fixed.** At frames 2,080, 2,120 and 2,160 the replacement draws the island under the
  glider as the image does (`pilotwings-f2120.png`, against `before-limit/pilotwings-f2120.png`, which showed sea). The
  demo still diverges later, as §51.3 found, from the flight model's values.
- **Lock On draws no ground** (`lockon-f1600.png`, `lockon-f2000.png`): the floored Vva, -47 against the image's -46,
  fails the game's horizon test. With Vva toward zero the ground is drawn and matches the image's closely
  (`exp/lockon-trunc.png`). Pictures equal: 1,532 of 7,198. The median of differing pixels is 58.6%, from 37.3% when
  the ground was drawn as streaks.
- **Super Mario Kart, Suzuka 8 Hours and Michael Andretti's Indy Car Challenge** never pass the limit and are unchanged
  from §52.2.
- **Ballz 3D** is unchanged: it passes the limit, but nothing it draws comes from Parameter's results.
- **Super Bases Loaded 2's** camera never passes the limit on the image. Its attract sequence took another course on
  the replacement: pictures equal 3,031, from 3,191, and sprites within 1 pixel 94.8%, from 95.4%.

### 52.7 What is put for decision

**B5, Vva's rounding past the limit.** The declared rank, exact results over both sets, chose floor by 14,417 exact
results in 1.78 million. That choice leaves Lock On without its ground, which the other member draws. The tester may:

- keep the rank, and Lock On needs the player's image;
- or choose B5 toward zero for Lock On's sake. That is a re-ranking within the declared family and adds nothing from
  the image.

The difference is one line in `Projection::parameter`. The image's Vva past the limit is not quite either member's,
which the seeded set's exact counts show (71,187 and 44,436 of 262,144).

### 52.8 Amendment: a tie rule on the trace oracle, written before the regrade

*Decided 2026-10-04*, on §52.7. A member is not chosen by which game it draws. The rank of §52.4 (exact results over
both sets) gains a tie rule: **when two declared members' exact counts on the seeded cases differ by under 1%, they are
ranked by exact share on the games' own traced traffic**, plan §4.2's trace oracle. That is how Project's choice was
justified, on Super Mario Kart's traced Project (§52.2). For B5 the traffic is the traced Parameter and Raster commands
past the limit of every DSP-1 title that passes it. Whichever member the rule ranks first is built, and the result is
recorded either way.

### 52.9 B5 regraded under the tie rule (measured 2026-10-04)

`dsp_limit dsp1b 65536`, then each bench trace alone. Only Lock On, Ballz 3D and Pilotwings pass the limit; Super Mario
Kart, Suzuka 8 Hours, Michael Andretti's Indy Car Challenge and Super Bases Loaded 2 never do.

- **The seeded cases.** Floor has 320,147 exact results and toward zero 293,396, a difference of 26,751, which is 8.4%.
  The two differ only in Parameter's Vva: 71,187 against 44,436. **The tie rule's condition, under 1%, is not met.**
  §52.7's figure of 14,417 in 1.78 million (0.8%) was over both sets together, not the seeded cases.
- **The traced traffic past the limit**, measured since the rule asks for it as the tie's measure: Parameter and Raster
  exact results are 1,260,721 for floor and 1,273,055 toward zero. That is Lock On's Parameter, 19,684 against 30,652;
  Ballz 3D's, 3,685 against 5,082; and Pilotwings', 368 against 337. Raster is the same under both.

**By the rule as written, floor stays, and nothing is rebuilt.** Lock On keeps drawing no ground (§52.6). Had the
condition been read over both sets, the traced traffic would have ranked toward zero first.

## 53. The DSP replacements' clone check (2026-10-04)

`VenusRT_DspHle.md` §5.4 asks for the replacements' sources to be compared mechanically with the DSP code most likely to
have contaminated a writer. §41.9 ran that comparison once, as part of G8, when only `dsphle.rs` and `dspengine.rs`
existed. This section repeats it on the finished set of replacements. It was run by a reader who has written none of
the replacement code and writes none of it. The reference sources were read by the tool. The reader looked at them only
at the lines a flagged match pointed to, and this section describes them only by file name, count and offset, as §4.1
requires. Every number below was **measured** on 2026-10-04 at WiseMan `f4ec8031`.

### 53.1 What was audited

- **The replacements**, `src/chips/`: `dsphle.rs` (1,130 lines; 1,218 fingerprints at §4.1's k and window), `dsp2.rs`
  (645; 722), `st010.rs` (206; 226), `dsp4.rs` (288; 238) and `dspengine.rs` (112; 40).
- **The independent generators**, `EmuSen.WiseMan/Reference/analysis/dsp1_tables.py` and `st010_tables.py`.
- **The generated tables**, written by `dsp_oracle tables` (the DSP-1's sine S and derivative D, 256 words each; the
  ST010's sine, 256 words). Each image is byte-identical to its generator's output. The crate tests
  `no_replacement_source_holds_a_table_literal`, `the_tables_equal_the_independent_generator` and
  `the_st010_sine_equals_the_independent_generator` pass.
- **As a supplement**, `src/chips/dsporacle.rs` and the thirteen `examples/dsp_*.rs` files (4,271 lines). They hold the
  oracle and the declared formula families, which a copied formula would pass through first.

### 53.2 The comparison set

Everything was fetched into `~/.cache/emusen/probe/venusrt/clonecheck/dsp/`, outside the repository.
`PROVENANCE.txt` there gives each source's URL, commit and licence, and `scripts/` and `runs/` hold the commands and
their full output.

| Set | Source | Files, lines |
|---|---|---|
| `bsnes-hle` | bsnes `0c2fa0db`, `sfc/coprocessor/` dsp1, dsp2, dsp4, st0010 | 19, 5,152 |
| `bsnesmercury-hle` | bsnes-mercury `79d7f9de`, `sfc/chip/` dsp1-dsp4, st0010 | 21, 6,427 |
| `bsnes059-hle` | bsnes 0.59 as carried by beetle-bsnes `5f05e4c7`: dsp1-dsp4, st010, st011 | 23, 6,414 |
| `snes9x-dsp` | snes9x `1bcc369e`: `dsp.cpp`, `dsp.h`, `dsp1.cpp`-`dsp4.cpp`, `seta.cpp`, `seta.h`, `seta010.cpp`, `seta011.cpp` | 10, 6,544 |
| `mame0140-hle` | MAME tag `mame0140`: `snesdsp1.c`-`snesdsp4.c`, `snesdsp4.h`, `snesst10.c` | 6, 6,200 |
| `wiki-st010` | the superfamicom wiki's ST010 page, its nine code blocks (page of 2022-05-24) | 1, 249 |
| `bsnes-lle` | bsnes `0c2fa0db`: `sfc/coprocessor/necdsp`, `processor/upd96050` | 9, 837 |
| `bsnesplus-necdsp` | bsnes-plus `a9789fab`: `snes/chip/necdsp` | 6, 748 |
| `mame-upd` | MAME `c2334733`: `bus/snes/upd`, `cpu/upd7725` | 6, 1,908 |
| `mesen` | Mesen2 `b9fa69dd`, `Core/SNES` (its DSP files unmodified in the local checkout) | 164 |

**What this adds to §41.9's set.** §41.9 recorded that no bsnes or higan `sfc/coprocessor/` HLE could be found. Current
bsnes carries one (dsp1, dsp2, dsp4 and st0010), and so does bsnes-mercury, with dsp3 as well. Both are included. The
other sources are at §41.9's commits, except current MAME, which is at a newer head. snes9x has no `st010.cpp`, and its
ST010 is `seta010.cpp`. MAME 0.140 has no ST011 file.

**The sets against each other**, run as a check that the tool sees these sources' known lineage:

| Pair | Pairs over 12 | Highest |
|---|---|---|
| bsnes current HLE, bsnes-mercury | 10 | 1.000 (`dsp4emu.c`, 767 shared) |
| bsnes 0.59, bsnes current HLE | 10 | 1.000 |
| bsnes 0.59, snes9x | 9 | 0.777 (`dsp4emu.c` ~ `dsp4.cpp`, 591) |
| bsnes 0.59, MAME 0.140 | 10 | 1.000 (`st010_data.hpp` ~ `snesst10.c`, 361) |
| the wiki's ST010 code, bsnes 0.59 | 2 | 0.601 (217) |
| bsnes-plus necdsp, current MAME upd7725 | 3 | 0.553 (26) |

The HLE sources form one family. The bsnes lineages are one code base, and snes9x and MAME 0.140 share most of it. A
copy from any of them would be a copy from all.

### 53.3 Structure

**The runs.** `clonecheck.py <audited> <set>` for each of the two audited groups against each of the ten sets, with
§41.9's vocabularies (fullsnes, anomie's documents, the WDC datasheet, the pinned DSP pages, and MoonRT's, MercuryRT's
and MarsRT's `src/`). The threshold is §4.2's 12 shared fingerprints, at k = 16 and a window of 8. Each pair was run a
second time with `--min-shared 1`, so that the largest pair under the threshold is known as well. The tool reads no
`.py` file, so the generators were run through a wrapper that adds the extension and blanks docstrings. The tool itself
was not changed.

**No pair reaches the threshold.** The largest number of fingerprints any audited file shares with any reference file
is 1:

| | bsnes HLE | mercury | 0.59 | snes9x | MAME 0.140 | wiki | bsnes LLE | bsnes-plus | MAME upd | Mesen |
|---|---|---|---|---|---|---|---|---|---|---|
| `dsphle.rs` | 1 | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 1 |
| `dsp2.rs` | 0 | 0 | 0 | 1 | 1 | 0 | 1 | 0 | 0 | 1 |
| `st010.rs` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| `dsp4.rs`, `dspengine.rs` | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| the two generators | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |

§4.2's negative control, two unrelated cores, peaked at 4. One shared 16-token k-gram is below that noise floor, and
none of the single matches was examined further. The supplement peaks at 1 as well. In particular, `st010.rs` shares
nothing with the wiki's ST010 code, the one source that `VenusRT_DspHle.md` §1.4 records as seen by the author of the
plan.

### 53.4 Tables

**The tool's signal** (runs of eight or more literals, at least four distinct, matched by value) finds no shared table
in any of the twenty runs. It sees fifteen runs in `dsphle.rs`, all in `dsp1_timing`, the cycle counts §39.3 measured,
and one in a `st010.rs` test. None of them occurs in a reference.

**A supplementary scan**, because the tool's runs break at a minus sign and cannot see a generated table. Every
comma-separated run of eight or more literals in the references, signs kept and compared modulo 2^16 (229 runs), was
matched in 8-word windows against the three generated tables and against the literal runs of the audited sources. The
results:

- **T1. The DSP-1's sine S** coincides with a 256-entry sine table that five references print: bsnes's three lineages,
  snes9x and MAME 0.140, all identical to one another. 241 of S's 249 windows match, and 255 of 256 entries are equal.
  The references' table is exactly §40.4's formula, floor(2^15·sin(2πi/256)) quarter-mirrored, at all 256 entries. The
  one difference is at the formula's saturation point, where §40.3's member saturates after negating. S is produced by
  `tables()` and `quarter_sine()` in `dsphle.rs`, a Taylor series in f64, and independently by `dsp1_tables.py`.
  Neither holds a literal. **Verdict: (a)**, convergence forced by a documented formula. The replacement's copy is
  generated by code, which is allowed. The entry where the two differ is evidence against copying.
- **T2. The same S** also coincides with a 1,024-word literal in four references (bsnes's three lineages and snes9x).
  Compared byte for byte, without printing, that literal **is the DSP-1B's data image**: those references embed the
  program's data as a literal. S's words 0-127 sit in it at byte 556, and words 64-191 at byte 812. These are the two
  formula-forced runs that `firmwarecheck --forced` found against the image in §40.4. **Verdict: (a)**, the same forced
  run as T1, already proved formula-forced with zero residual. That these references embed the firmware's data is one
  more reason why `VenusRT_DspHle.md` §1.2 keeps them unread.
- **T3. The ST010's sine** coincides with a 256-entry table that six sources print: the wiki's code, bsnes's three
  lineages, snes9x and MAME 0.140, all identical. 180 of 249 windows match, and 240 of 256 entries are equal. The
  replacement is exactly round(32,767·sin(2πk/256)) by quadrant, §43.2's closest member, at all 256 entries. The
  references' table departs from that formula at 16 entries, and the replacement carries none of the 16. It is
  generated by `sine()` in `st010.rs` on `dsphle::quarter_sine`, and independently by `st010_tables.py`. **Verdict:
  (a)**, generated by code, which is allowed. That the replacement keeps the formula where the references depart from
  it is direct evidence that it was not taken from them. It also agrees with §43.2's finding that the image's table is
  the formula broken at single entries.
- **Cross-hits.** S matches 26 windows of the ST010 tables, and the ST010 sine matches 32 windows of the DSP-1's. These
  are two sines of amplitude 2^15 and 2^15 - 1 agreeing wherever their roundings meet. **(a).**
- **The derivative table D** matches nothing. Neither do the audited sources' own literal runs: the two in `dsp2.rs`
  are a test's inputs, with fewer than four distinct values.
- **T4 (supplement).** `dsporacle.rs:761`, the DSP-2 test's list of command numbers, which opens 00h, 01h, ... 07h,
  matches an ascending run in `dsp3emu.c`, `dsp3.cpp` and `upd7725.cpp`. **(b)**, a coincidence of trivial code.

### 53.5 Names

**The tool's signal**, identifiers of six or more characters outside the vocabularies, gives one name: `necdsp`, against
the bsnes LLE, bsnes-mercury, bsnes-plus and Mesen. The supplement adds `chipread` and `chipwrite` against current MAME.

**A wider pass**, because the DSP-1's documented parameter names are short (Vof, Vva, Les, Lfe, Azs). It took every
identifier of three or more characters. Of the audited code's 352, 89 occur in some reference. 85 of those are in the
documents or the other cores. Vof, Vva, Les, Lfe and the command names are among them, as SnesLab's pages print them.
The four that are not:

- **`necdsp`** (`NecDsp` in `dspengine.rs` and `dsphle.rs`): the chip family's name, NEC's DSP, and the name of
  VenusRT's own LLE module since §24. It escapes the vocabulary only because fullsnes writes it as two words. **(a).**
- **`usf0`** (`USF0`, `usf0` in `dsp2.rs`): the status register's user flag 0. fullsnes prints it as "USF1-0", which the
  folding splits. **(a).**
- **`ceil`** (`st010.rs`): Rust's `f64::ceil`. **(b).**
- **`tag`** (`dspengine.rs`, `dsphle.rs`): the state's engine tag, an ordinary word. **(b).**
- **`chipread`, `chipwrite`** (supplement): the variants `Transfer::ChipRead` and `ChipWrite` of VenusRT's LLE, named
  for what they are. **(b).**

### 53.6 Constants

The audited code holds 33 distinct non-trivial integer literals: at least 256, and neither a power of two, a power of two
plus or minus one, nor a byte mask. 11 also occur in some reference. To read that count, the base rate: the references
together contain 30.3% of all values in 256-1,023, 1.8% of those in 1,024-4,095 and 1.1% of those in 4,096-65,535,
almost all inside their large data tables.

- **C1. 38CEh**, `dsphle.rs`'s `LIMIT`, the DSP-1's limit angle on Azs. It is in no document. In five references
  (bsnes's three lineages, snes9x, MAME 0.140) it occurs as one entry of a longer numeric table, not as a code
  constant. The replacement holds the single value, which §52.1's amendment admits as measured. §52.3 records how: Azs
  stepped by one through 38B0h-38F0h at three values of Les, the change found at 38CEh, and the held Vva checked
  against -Les·cot(38CEh) to a third of a step. **The measurement was repeated here** with `dsp_oracle chains` on the
  DSP-1B image (Fx = Fy = 0, Fz = 100, Lfe = 64, Aas = 0, Azs from 38C8h to 38D4h by one). At Les = 7FFFh, Vof is 0
  through 38CDh and non-zero from 38CEh, and Vva stops changing at 38CEh. Les = 1000h gives the same threshold, and so
  does the DSP-1 image at 7FFFh. **Verdict: (a)**, in the extended sense that the convergence is forced by a property
  of the chip, measured and reproduced, rather than by a document. Any correct model must hold this value.
- **C2. 272, 311, 362, 499, 527, 541, 600, 735 and FFF2h.** The first eight are cycle counts measured from the image
  (§39.3, §42, §43, §52.6) or a test's tick, and FFF2h is -14, a test's expected word. Every occurrence in the
  references is inside a data table of eight or more literals, none in code. 9 of the audited code's 18 values in
  256-1,023 recur (these eight and C3's 768), against the 5.5 that the 30.3% base rate predicts, about 1.8 standard
  deviations over. That is
  within chance for a handful of values, and no recurrence sits where a cycle count could be read. **(b).**
- **C3. 768** (300h), an offset in `dsp2.rs`'s state layout (`d[base + 768..]`), occurs in code only in the references'
  DSP-3 files. VenusRT has no DSP-3 replacement. It is 3·256. **(b).**

### 53.7 Verdict

| Item | What | Verdict |
|---|---|---|
| Structure | no pair over 12; the largest share is 1 fingerprint, against a noise floor of 4 | nothing to judge |
| T1 | DSP-1 sine ~ the references' sine table, 255/256 | (a), generated |
| T2 | DSP-1 sine ~ the DSP-1B data image embedded in four references, at bytes 556 and 812 | (a), generated, §40.4's forced runs |
| T3 | ST010 sine ~ the references' ST010 table, 240/256; the replacement keeps the formula at the 16 where they depart | (a), generated |
| T4 | an ascending list of command numbers (supplement) | (b) |
| Names | `necdsp`, `usf0`; `ceil`, `tag`, `chipread`, `chipwrite` | (a); (b) |
| C1 | 38CEh, the limit angle | (a), a measured chip property, reproduced |
| C2, C3 | cycle counts, -14, 768 | (b) |

**No item is evidence of copying (c).** The replacements share no code structure with any DSP emulation in the set,
nor with Mesen. Every shared table is one that the replacement generates from a stated formula, and two of the three
differ from the references exactly where the formula says they should. The one undocumented shared constant is a
measured fact about the chip, and the measurement reproduces.

**What this does not show.** §4.2's calibration found that a port which restructures the code escapes the tool, so
"nothing found" bounds statement-order copying only. A rule taken from a reference and rewritten in new code would pass
this check. Against that, the record of each step stands: the declared families, the R1-R4 rules, and the one exposure
`VenusRT_DspHle.md` §1.4 records. Nor can the check tell what a writer has seen. It can only say that what was written does
not carry the references' text, their tables or their departures from the formulas.

**Two observations, not findings.** `no_replacement_source_holds_a_table_literal` covers `dsphle.rs`, `dsp2.rs`,
`st010.rs` and `dsp4.rs` but not `dspengine.rs`, which holds no array literal today. And `dsp1_timing`'s fifteen tuple
runs escape that guard's count by their parentheses. They are measured latencies, which R1 admits, and none recurs in a
reference.
## 54. The NEC DSP replacements: ST010 05h, the driver simulation, and 01h, the angle (2026-10-04)

§43.1 left two ST010 commands as named losses: 05h, the driver simulation, which is 53.6% of F1 ROC II's commands
(§37.6), and 01h, which no bench game gives. F1 ROC II parts from the image at its first 05h (§43.3, §48.1). This
section characterises both from the oracle and builds them. §54.1 and §54.2 were written and committed before the
replacement was built and graded through the ports.

**Provenance.** Written by someone who has **never opened the superfamicom wiki's ST010 page**
(<https://wiki.superfamicom.org/st010>, plan §1.4), nor any other page that embeds emulator source. The sources are
fullsnes's "ST010 Commands" (names only: "01h Unknown Command", "05h Simulated Driver Coordinate Calculation"), plan
§3.5's table of 05h's parameter addresses (bytes 00C0h-00E0h, the words 60h-70h, as "limits, a 32-bit position, an
angle, a radius, an increment, a maximum radius, a flags word, an unnamed word"), and the oracle. No emulator's
source, no listing or disassembly, and no byte of the dumps were read.

### 54.1 The commands as characterised from the oracle

**The method.** `dsp_oracle mailbatch 05` and `mailbatch 01` with single words varied against a zero RAM and against
seeded fills; F1 ROC II's 98,150 traced 05h cases (`mail-f1roc2.txt`, §43.3); and the hypotheses written as a model in
Python, kept in the probe cache with the reports. As in §43.1, the characterisation ran each hypothesis against the
image's outputs until it held; for 01h's angle that included the whole normalised domain of 32 by 32 pairs. What is
declared below fixes the formulas and their choices before the replacement is built; the grade (§54.3) is of the built
replacement through the ports, from seeded phases. Words are 16-bit RAM addresses; X is word 0 or a position's first
coordinate, Y the second.

**01h, the angle of a vector.** Inputs X = word 0, Y = word 1, signed.

- The quadrant Q: 0000h for X >= 0 and Y >= 0, C000h for X < 0 and Y >= 0, 8000h for X < 0 and Y < 0, 4000h for X >= 0
  and Y < 0.
- The vector turned into the first quadrant: (a, b) = (X, Y), (Y, -X), (-X, -Y) and (-Y, X) respectively.
- **Normalisation**: while a or b is 32 or more, both are halved (arithmetic shift); a nonzero value that a halving
  makes zero becomes 1.
- **The angle**: θ = Q + 8000h + 256·n, n = round(256·atan2(a, b)/2π), half up; (0, 0) gives n = 0. This is the
  angle of the vector measured as atan2(-X, -Y), 65,536 to a turn, to a whole 256th of a turn.
- **Results**: word 0 = a, word 1 = b, word 2 = Q, word 3 = Y as given, word 8 = θ.
- **X = 0 with Y < 0** is its own branch: words 0 and 1 are (0, |Y|) normalised, not turned, and θ = 0.
- **-32768 in either input** never completes: the busy bit stays set for at least the 2,000,000 cycles the oracle
  waits. The chip has by then written word 0 = |X| and word 1 = |Y| (16-bit, so 8000h stays 8000h), word 2 = Q and
  word 3 = Y, and nothing else.
- **Time** from the poll that sees the busy bit to its clearing: 71 cycles in quadrant 0000h, 78 in C000h and 4000h,
  75 in 8000h, and 64 on the X = 0 branch; and per halving 7 cycles, or 8 when the smaller value was already 0, or 10
  when the halving made it 0 and it was set to 1.

**05h, one driver's step.** The words, named from what they do (plan §3.5's names in brackets where they map):

| Word | Name here | Role |
|---|---|---|
| 60h, 61h | the waypoint (T_x, T_y) | [limits] where the driver is heading, whole units |
| 62h-63h, 64h-65h | the position (P_x, P_y) | [the 32-bit position] each a 16.16 value, low word first |
| 66h | the heading h | [the angle] 65,536 to a turn |
| 67h | the heading error | result only |
| 68h, 69h | the offset (D_x, D_y) | result only: T - P in whole units |
| 6Ah | the speed v | [the radius] unsigned, 1/8,192 unit a step |
| 6Bh | the acceleration | [the increment] |
| 6Ch | the wanted speed | [the maximum radius] |
| 6Dh | the gate's orientation | [the flags word] bit 15 |
| 6Eh | the flags | bit 3 set when a waypoint is passed |
| 6Fh, 70h | the next waypoint | [the unnamed word] 6Fh its x; 70h its y in bits 0-11 and the next gate's orientation in bit 15 |

The step, in the order the results show:

1. **The offset**: D_x = T_x - P_x and D_y = T_y - P_y from the position's whole words, 16-bit, stored in 68h and 69h.
2. **The bearing**: θ_t = 01h's θ of (D_x, D_y), by the same normalisation and table; word 8 = θ_t. Either offset
   -32768 never completes, as 01h does; words 0-3 are then 01h's and words 68h and 69h are written, nothing else.
3. **The heading error** e = θ_t - h (16-bit), stored in 67h. **The turn**: with e's high byte taken as signed, h
   moves by +0280h when it is positive, by -0280h when it is negative, and not at all when it is zero.
4. **The speed**: if |e| >= 1000h (|8000h| taken as 8000h), v' = v - (|e| >> 4), the shift arithmetic, and 0 if
   that borrows; otherwise v' = v + acceleration, and v' = the wanted speed when that sum carries out of 16 bits or
   is at least the wanted speed. Stored in 6Ah.
5. **The move**: with k = h' >> 8 and S the ST010 sine of §43.2 (round(32,767·sin(2πk/256)), the named loss), the
   step M_x = 2·⌊v'/256⌋·⌊S(k)/32⌋ and M_y = 2·⌊v'/256⌋·⌊S(k + 64)/32⌋, 32-bit, the floors arithmetic. Words 0-1 = M_x
   and words 2-3 = M_y; P_x' = P_x - M_x and P_y' = P_y - M_y as 16.16 values, the whole word kept to 13 bits (mod
   2000h). The step is along (-sin h', -cos h'), the direction 01h's angle measures.
6. **The gate**: with 6Dh's bit 15 clear, the waypoint is passed when |D_x| <= 127 and |D_y| <= 7; with it set, when
   |D_x| <= 7 and |D_y| <= 127 (the offsets of step 1). When passed: T_x = 6Fh, T_y = 70h AND 0FFFh, 6Dh = FFFFh if
   70h's bit 15 is set and 0000h if not, and 6Eh's bit 3 is set.

Nothing else is written, and no other word is read: words 67h, 68h, 69h and 6Eh are results only (6Eh's other bits are
kept). **Time** from the poll: 01h's time for (D_x, D_y), plus 128 when the sum of step 4 is below the wanted speed,
129 when it reaches it, 130 when it carries, 123 when the speed falls and 124 when that borrows; plus 6 for a turn up
and 9 for a turn down; plus the gate's test, in that order: 1 when 6Dh's bit 15 is set, 2 when D_x is negative, then,
if D_x passed its bound, 5 and 2 more when D_y is negative, then, if D_y passed too, 20 and 1 more when 70h's bit 15 is
set.

### 54.2 The families, declared before the replacement is graded

| Choice | Members | Bits |
|---|---|---|
| A1, the angle's rounding | n half up, floored, or raised | 1.6 |
| A2, a value halved to zero | set to 1; or kept 0, with the shifted-out bit carried into bit 0 | 1 |
| F1, the speed in the move | ⌊v'/256⌋; or v'/256 kept whole and the product shifted once | 1 |
| F2, the sine in the move | ⌊S/32⌋ floored; or truncated toward zero | 1 |
| F3, the heading in the move | h' >> 8; or h' rounded to the nearest 256th | 1 |
| F4, the sum at the wanted speed | at least; or above (time only) | 1 |

About 7 bits of choice beyond the sine's, which §43.2 fixed. The rank is §52.1's: exact results first over the seeded
cases and F1 ROC II's traced traffic together, ties to the most close.

**The constants, and R1.** The model carries integers read from the oracle's outputs, none fitted by error against them:
the waypoint gate's half-widths 127 and 7, the masks of 70h (0FFFh, bit 15) and of the position (13 bits), the turn
step 0280h, the speed rule's threshold 1000h and shift 4, the move's factor 2 and its shifts 8 and 5, and the cycle
counts. The cycle counts are latencies, which the conventions admit. The masks and the gate are read here as protocol
facts of the RAM layout, as §43.2 read 02h's count of 2 to 15. The turn step, the threshold and the shifts are
structural constants of a command no document describes. Each was fixed by single probes and holds exactly, which
is the black-box characterisation plan §1.3 allows. They are nonetheless values observed from the image, and the R1
amendment of §52.1 admits only the DSP-1's limit angle. Whether they stand is put for decision (§54.6). If they do
not, 05h returns to the named loss of §43.1.

**Predictions** (to be retired by §54.3-§54.5):

- **P54.1.** 05h is exact on every seeded case, value and latency alike, except where its bearing falls on an entry
  where the chip's angle differs from the formula. The characterisation found two such entries of the 1,024: (10, 9),
  where the chip gives n = 32 against the formula's 34, and (31, 8), 53 against 54. They are left as they fall, a named
  loss as §43.2 left the sine's; no entry is patched.
- **P54.2.** On F1 ROC II's traced traffic, 05h is exact in 98,038 of 98,150 cases (99.89%). The 112 misses all fall at
  those two entries, the first in frame 1,246.
- **P54.3.** F1 ROC II in lockstep still parts at frame 1,019 or 1,020. 07h's arrays carry no perspective since §48.2,
  and they reach the S-CPU in the frame of the first 05h. The race's cars nevertheless follow the image's for some
  hundreds of frames.
- **P54.4.** 01h is exact on every seeded case except at the same two entries.

### 54.3 Chosen, built, and the grade (measured 2026-10-04)

**The choice.** The family of §54.2 was ranked in the probe cache's Python model against the image's outputs: 40,000
seeded 05h cases (value and time) and the 98,150 traced ones (value only, the trace's times being the game's).

| Member | Seeded exact | Traced exact | Together |
|---|---|---|---|
| A1 half up, A2 set to 1, F1 ⌊v'/256⌋, F2 floor, F3 h' >> 8, F4 at least | 39,955 | 98,038 | **137,993** |
| the same, F4 above | 39,954 | 98,038 | 137,992 |
| A2 the shifted-out bit carried | 22,966 | 73,886 | 96,852 |
| A1 raised | 20,380 | 61,881 | 82,261 |
| A1 floored | 20,690 | 61,426 | 82,116 |

Every member with F1, F2 or F3 changed ranks below these. F4 is a choice of time alone, and its two members differ by
one seeded case, under §52.8's 1%; the traced traffic, which carries no time, cannot separate them. A set aimed at the
boundary does: 3,336 cases with the wanted speed within 3 of the sum, 333 of them on it, of which the "at least"
member gives 3,336 exactly and the "above" member 3,013. The first member is built. For 01h alone, over 26,611 seeded
cases (the whole square from -40 to 40, 20,000 drawn over every magnitude, 50 with -32768), half up with values set to
1 gives 26,596 exact, the carried bit 17,042, and the floored and raised roundings 14,649 and 15,055.

**Built** in `chips/st010.rs`:

- `bearing`, 01h's routine, with its words, its time and the case that never completes;
- `drive`, 05h's step, calling it;
- the angle table, generated at first use from the formula, never a literal (`atan_n`, `atan_image`).

01h, 05h and their mirrors take their time from these in place of the constants §43 gave the frame. A command that
never completes keeps the busy bit set and its due edge at the end of time, so the state needs nothing new and its
version stays 22. `st010_tables.py --atan` is the independent generator. The crate test
`the_st010_angle_equals_the_independent_generator` holds the table to it. `the_st010_angle_and_driver_follow_their_formulas`
holds the formulas with no image. `the_st010_driver_and_angle_agree_with_the_image` holds 2,304 seeded cases on the
image, value and time, but at the angle's entries, which it bounds at 1%. `dsp_stgrade` now gives 05h's words, counts
a command that never completes on both engines, starts both afresh after one, and carries the image's RAM into the
next case, so that a difference is not counted twice.

**The grade** (`dsp_stgrade`, seeded phases and inputs, 8 threads):

| Code | Cases | Never complete, on both | RAM differs | of which word 8 differs | Latency differs |
|---|---|---|---|---|---|
| 05h | 65,536 | 598 | 94 (0.14%) | 94 | 2, both among the 94 |
| 01h | 65,536 | 6,394 | 48 (0.07%) | 48 | 0 |
| 0Dh, 09h, 15h, F9h, the mirrors | 4,000 each | | 0 | | 0 |
| 03h, 06h, 07h, 08h, again | 65,536 or 4,000 | 0 | as §43.2 and §48.2 | | 0 |

**P54.1 holds**, and so does **P54.4**: every difference is a bearing one 256th of a turn from the image's, the two
entries of §54.1, and through 05h the turn, speed and step that follow from it. **P54.2 holds exactly**: on the traced
traffic, 98,038 of 98,150 are exact, and the 112 misses are all at those entries. In the first five misses (frames
1,246 to 1,337) only words 8 and 67h differ, and the first that changes a turn, and so a position, is in frame 1,366.

**firmwarecheck** (`dsp_oracle tables` writes `st010.atan.bin`, byte-identical to `st010_tables.py --atan`'s):

| Against | Plain | `--forced` |
|---|---|---|
| the data half (4,096 bytes) | 548 of 2,048 bytes equal at the same offset (26.8%); common runs of 659 and 1,341 bytes; FAIL, the expected outcome of a formula found in the data | residual 0; **PASS**; formula coverage 3,199 of 4,096 bytes (78.1%) |
| the program half (49,152) | 25.7% equal; longest run 65 bytes; FAIL | residual 0; **PASS**; coverage 96.9% |

The two long runs in the data half end at entries 329 and 1,000 of the formula image, which are (10, 9) and (31, 8):
the program's table is the formula broken at exactly the two entries §54.1 found, as §43.2 found for its sine. The
coverages are upper bounds. The table's words have zero low bytes and its first row is zero, so runs over zero bytes
in either half count.

### 54.4 F1 ROC II in lockstep (measured 2026-10-04)

`dsp_lockstep` now reports each space's first differing frame. `dsp_cars` is new. It runs the two machines apart from
power-on under one pad script, records the position each completed 05h leaves, and compares the n-th 05h of a frame on
one machine with the n-th on the other. 7,200 frames from power-on with no input, the attract mode's four demo races,
before (WiseMan f4ec8031, built as a throwaway copy and removed) and after:

| | Before | After |
|---|---|---|
| First differing frame: chip RAM, picture | 1,019, 1,025 | 1,019, 1,025 |
| WRAM, the APU's RAM, the clock | 1,020 | **1,367** |
| OAM, VRAM, CGRAM | 1,021, 1,046, 1,285 | 1,989, 3,623, 5,730 |
| Pictures equal | 3,284 of 7,194 | 3,284 of 7,194 |
| Pixels differing per frame, median, 95th percentile | 59.7%, 73.0% | 28.6%, 58.6% |
| Sprites alike on both in frames that differ: compared, placed alike, unmatched | 15,262; 248; 29,078 | 41,536; 41,236 (99.3%); 197 |
| Drivers placed alike, frames 1,200-1,499 | 0 of 6,832, median 1,552 units off | 7,190 of 7,493; largest 7.8 units |

**P54.3 is false in its first half.** The chip's RAM parts at frame 1,019, when 07h's arrays lose their perspective
(§48.2). Those arrays reach the picture from frame 1,025 but never WRAM, and the game's state stays identical to the
image's until frame 1,367. That frame follows the first 05h whose bearing falls on one of the two entries and changes a
turn (frame 1,366, above). **Its second half holds.** Selected windows of 300 frames over the demo races; the whole
table is `cars-f1roc2.txt`:

| Frames | Drivers compared | Alike | Within 16 units | 95th percentile | Largest |
|---|---|---|---|---|---|
| 1,200-1,499 | 7,493 | 7,190 | 7,493 | 0 | 7.8 |
| 1,500-1,799 | 7,500 | 6,762 | 7,241 | 2.7 | 42.2 |
| 1,800-2,099 | 7,506 | 6,498 | 7,206 | 2.8 | 38.5 |
| 2,100-2,399 | 2,992 | 2,211 | 2,801 | 30.2 | 2,423.5 |
| 3,600-3,899 | 7,491 | 6,849 | 7,454 | 1.2 | 274.1 |
| 5,700-5,999 | 7,505 | 5,507 | 6,971 | 36.8 | 2,555.1 |

Each demo race starts alike: the windows that open the second, third and fourth races (frames 2,700-3,299,
4,800-5,099 and 6,600-7,199) are alike throughout. Within a race most drivers stay within a few units of the image's.
The largest distances come in windows where a frame holds a different count of 05h on the two machines, so
that the n-th of one is another driver's. Before 05h was built, no driver was ever placed alike after frame 1,019.

**With a player** (a script of 12 Start presses through the menus to Moon City's qualifying, then B and A held and
the pad steered, 2,700 frames): WRAM, VRAM, CGRAM, OAM, the APU's RAM and the clock **never part**. Only the chip's RAM
(frame 1,468) and the picture (frame 1,474) differ, and the speed and lap times on the screen are the image's frame
for frame. Qualifying has no opponents. The pictures are in the cache's `shots/`.

**Does it play?** Yes, as a game: its menus, qualifying and races run with the game's state identical to the image's
until a driver's bearing falls on one of the two angle entries. After that, the opponents drive the image's course
within a few units for most of a race, and the player's car is the image's. **It does not look like the game.** The
road is drawn as a flat plane seen from above, not in perspective, because 07h's perspective is still a named loss
(§48.2). That loss, not 05h, is now what separates F1 ROC II on the replacement from F1 ROC II on the image. The
replacement's cost text says so.

### 54.5 What remains

- **The angle's two entries**, (10, 9) and (31, 8), are a named loss as the sine's are. The table is the formula
  everywhere else, and firmwarecheck shows the program's own table broken at those two places.
- **07h's perspective** (§48.2) is the largest loss left in F1 ROC II.
- **08h's sine** (§43.2) is unchanged.

### 54.6 What is put for decision

1. **R1 and 05h's constants.** §54.2 lists the integers 05h's model takes from the oracle: the gate's half-widths,
   the waypoint word's masks, the 13-bit position, the turn step 0280h, the speed rule's threshold 1000h and shift 4,
   and the move's shifts. Each holds exactly and none was fitted by error. The amended R1 admits only the DSP-1's
   limit angle, though, and the black-box characterisation of plan §1.3 and Q4 cannot work without such constants.
   The tester may:
   - admit them as the structure of a command no document describes, recorded as here;
   - or not, and 05h and 01h return to the named loss of §43.1, which costs F1 ROC II every opponent's line from
     frame 1,019.
2. **F4's tie.** It was broken by a set aimed at the boundary, which neither §52.4's rank nor §52.8's tie rule names.
   It changes 05h's time by one cycle when the sum equals the wanted speed, and no value.

## 55. The NEC DSP replacements: the R1 amendment widened, and the tie-break by boundary cases (2026-10-04)

### 55.1 The decision, and the amendment it makes

*Decided 2026-10-04*, on §54.6. Plan §1.3's R1 exception is widened from one constant to a class: **whole-number
behavioural parameters of the chip's program logic**, such as gate widths, masks, shifts, step sizes and thresholds,
found by black-box probing, each recorded with how it was measured. **Values fitted by minimising an error against
the image's outputs remain forbidden.** The plan's §1.3 carries the amendment. The distinction is one of method, not
of type: the DSP-1's limit angle (§52.3) was read off as the threshold at which a branch is taken, and 05h's
constants below were each fixed by single probes that hold exactly. 07h's K and c (§43.1) were chosen as the values
of a shape that came closest to 176 outputs, which stays a breach (§48.2).

### 55.2 ST010 05h's constants, admitted, and how each was measured

All with `dsp_oracle mailbatch 05` against a zero RAM unless said; each holds exactly on the 65,536 seeded cases and
98,150 traced cases of §54.3.

| Constant | Value | How it was measured |
|---|---|---|
| The gate's half-widths | 127 and 7 | the waypoint placed at every offset from -300 to 300 along each axis with the other held at 0, 3, 7 and 8: the next waypoint is taken for \|Dx\| <= 127 with \|Dy\| <= 7, and the axes exchange with 6Dh's bit 15 |
| The gate's orientation bit | 6Dh bit 15 | the traced values 0 and FFFFh, then seeded 6Dh: the box follows bit 15 alone |
| The next waypoint's masks | 70h AND 0FFFh; bit 15 to 6Dh | seeded 70h: T_y takes bits 0-11 and 6Dh = FFFFh exactly when bit 15 is set |
| The flag set on passing | 6Eh bit 3 | traced 0, 4 and 5 became 8, Ch and Dh; seeded 6Eh keeps its other bits |
| The position's width | 13 bits (mod 2000h) | positions of 0, 1, 1FFFh, 2000h, 2001h, 3FFFh, 4000h, 7FFFh, 8000h, C000h, FFFFh, with and without a move |
| The turn step | 0280h | heading 0 with bearings from 1 to 359 degrees: the heading moves by exactly +0280h, -0280h or 0; the 98,150 traced turns are all one of the three |
| The turn's dead band | e's high byte zero | the traced errors in steps of 80h: +80h no turn, -80h a turn down |
| The speed rule's threshold | \|e\| >= 1000h | the traced speeds by \|e\|'s high byte: the fall begins at 10h exactly |
| The speed rule's shift | \|e\| >> 4 | headings 2^14 from the bearing give a fall of 0418h, 0428h, 0438h for errors 4180h, 4280h, 4380h; an error of A000h a fall of 0600h |
| The move's factor and shifts | 2, v >> 8, S >> 5 | speed FF00h at every one of the 256 headings: each step equals 2·⌊v/256⌋·⌊S/32⌋ exactly, and speeds 1000h and 1010h move alike |

The cycle counts of §54.1 are latencies, admitted before this amendment. The angle's rounding (A1) and its zero rule
(A2) are choices within a declared family, not constants.

### 55.3 The tie-break by boundary cases

§52.8's tie rule ranks two members within 1% on the seeded cases by their exact share on the games' traced traffic.
**Accepted 2026-10-04**: when the traced traffic cannot separate the two, because the choice concerns time, which the
trace does not carry, or a case the trace never reaches, they are ranked by **cases aimed at the boundary between
them**, drawn by a seeded generator written before the ranking, the counts recorded either way. §54.3's F4 is the
first such tie: 3,336 cases with the wanted speed within 3 of the sum, 333 on it, gave "at least" 3,336 exact and
"above" 3,013, and "at least" is built.


## 56. The NEC DSP replacements: ST010 07h's perspective, probed, and stopped (2026-10-04)

§54.4 found 07h's flat road the largest loss left in F1 ROC II. Under §55.1's class, 07h's scale could be built if it
came from documented mode-7 quantities and whole-number parameters exposed by probing the chip's behaviour. This
section asks whether any input exposes them. It was written after the probes below, and no structure was graded,
since none could be declared without fitting.

**What the probes show** (measured 2026-10-04, `dsp_oracle mailbatch 07`):

- **The arrays depend on word 0's high byte alone.** With word 0 = 0, twenty seeded fills of the other 2,046 RAM words
  left all 176 values of the first array unchanged. Over 1,024 angles in steps of 40h, the four angles sharing a high
  byte gave identical RAM in 768 of 768 comparisons.
- **The time is a constant**, 4,988 cycles from the poll, at every one of the 1,024 angles and every fill.
- **The scale is the chip's alone.** By §43.1 each array is a sequence L(n) times a sine. L(n) falls with the line the
  way a perspective divisor K/(n + c) does, but no word the game writes enters it.

**Why this is a stop under §55.1.** A gate width or a threshold is found by varying an input until the behaviour
changes, and reading off where it changes. Neither K nor c, nor any whole-number form of them (a divisor m·n + d, a
numerator, a rounding), has an input that moves it. The only way to obtain them is to choose values that make the
formula's 176 outputs equal the image's. Requiring whole numbers and an exact match does not change that this is a
fit. It is §43.1's fit with a smaller search space, which §48.2 withdrew and §55.1 keeps forbidden. Whether such
whole numbers exist was therefore not tested: the test is the fit itself.

**No document supplies them.** fullsnes names 07h "Raster Data Calculation" and gives nothing more. Plan §3.5's
addresses give where the arrays are, not what is in them. fullsnes's mode-7 chapter documents the PPU's 8.8 matrix,
which fixes the units (§48.2's 100h), not a camera height or a distance to the screen. The array's length, 176 lines,
fixes n's range but not the scale.

**What stays.** 07h keeps §48.2's unit scale and its named loss. F1 ROC II's road is drawn flat on the replacement.
The open routes are for the tester to choose:

1. A document that states the ST010's perspective constants, or F1 ROC II's camera, if one is found.
2. A further amendment admitting a value found by an exact match of a declared structure against one command's
   outputs, as distinct from a least-squares fit. That would be a change of rule, not a reading of §55.1.
3. The player's image, the exact path, which the policy already provides.

## 57. The NEC DSP replacements: constants by exact match, and ST010 07h's perspective (2026-10-04)

### 57.1 The decision, and the amendment it makes

*Decided 2026-10-04*, on §56's second route, narrowly. Plan §1.3 gains a rule for **constants by exact match**:

- a command's constants may be found by exact match when no input moves them and no document gives them;
- the structure is declared and committed in advance;
- it has at most three whole-number parameters, each searched over a range declared in advance;
- a member is accepted only if it reproduces every output of the command exactly at every probed input; for 07h, all
  176 words of each array at all 256 high-byte angles;
- any shortfall rejects it, and there is no closest member;
- if more than one member matches exactly, that is reported, not chosen between;
- least-squares and other error-minimising fits stay forbidden.

The rule differs from §43.1's fit in what it can conclude. A fit always yields a member, the closest. This rule yields a
member only when the declared structure is the command's, and otherwise says that it is not.

### 57.2 07h's structure and ranges, declared before the search

**What is already fixed** (§43.1, §43.2, §48.2): word 0 = θ >> 8, words 1 and 2 = S(θ >> 8) and S((θ >> 8) + 64), and
the four arrays from words 78h, 128h, 1D8h and 288h are, line by line for n = 0 to 175,

    A(n) = H(L(n), C),  B(n) = H(L(n), S),  C(n) = H(L(n), -S),  D(n) = H(L(n), C),   H(k, l) = (k·l·2) >> 16 (arithmetic)

with S and C the sine and cosine of the angle and L(n) a 16-bit whole number per line. Only L(n) is open.

**The structure:**

    L(n) = R(K / (m·n + d))

- R, the rounding: floor, half up, or ceiling, three variants.
- K, the numerator: a whole number from 1 to 2^24 - 1.
- m, the lines' step in the denominator: a whole number from 1 to 16.
- d, the denominator's offset: a whole number from 0 to 32·m, so up to 32 lines.

That is three parameters, at the limit §57.1 sets. The structure is a perspective divisor: a line's scale is a height
over its distance below a horizon, m and d placing the horizon in units of a fraction of a line.

**The criterion**, as §57.1 sets it, with one reading made explicit. The sine is a named loss (§43.2): at some angles
the chip's words 1 and 2 differ from the replacement's by one, and the arrays' products carry that difference. No L
can mend that. So for the search each angle's arrays are computed from **the chip's own words 1 and 2 at that
angle**, and a member is accepted only if all 704 array words equal the image's at all 256 high-byte angles: 180,224
words. Those two words are outputs of the same command, and they are used only to set aside a loss already named.
Nothing of them enters the code, which keeps the replacement's sine. The grade with the replacement's own sine is
reported beside the search's.

**Counting members.** Two members count as one when they give the same L(n) at every n from 0 to 175, since no input
can tell them apart: (jK, jm, jd) is the same function as (K, m, d), and neighbouring K often give the same 176
values. If the members that match exactly all share one L sequence, the search has one answer, and the record lists
every triple that gives it. If members giving different L sequences match, the record reports them and nothing is
built.

**Prediction.** P57.1: no member matches exactly. §43.1's fitted c of 8.8 is close to no ratio with a small m, and a
chip with a data ROM may hold the scale as a table rather than divide.

### 57.3 The search, and what it found (measured 2026-10-04)

**The data.** `dsp_oracle mailbatch 07` at the 256 angles k·100h from a zero RAM, the arrays and words 0-2 of each,
held in the probe cache. Before the search, each line's L(n) was tested for consistency with the declared product:
for every n exactly one 16-bit L gives all four arrays' words at all 256 angles, with the chip's own sine words. The
product structure of §57.2 therefore holds, and each line has one value to reproduce.

**The search.** For each rounding, m from 1 to 16 and d from 1 to 32·m (d = 0 divides by zero at n = 0 and is no
member), the K that give each line's value form an interval, and the intervals were intersected over the 176 lines
within K's declared range. Every member left was then checked directly against all 180,224 array words.

| Rounding | m | d | K | Members |
|---|---|---|---|---|
| half up | 5 | 44 | 39,421 to 39,428 | 8 |
| half up | 10 | 88 | 78,841 to 78,857 | 17 |
| half up | 15 | 132 | 118,262 to 118,286 | 25 |

No floored or raised member, and no other (m, d), matches. **The 50 members give one L sequence**: the second and
third rows are the first with the fraction scaled, and neighbouring K differ by less than any line's rounding can
show. By §57.2's counting that is one answer, with no choice made. The code takes the first triple, K = 39,421, m = 5,
d = 44, which is L(n) = round(39,421/(5n + 44)) half up: 896 on the first line, 43 on the last. **P57.1 is false.**
The answer sits beside §43.1's fit, K = 7,885 and c = 8.8, which is the middle of the first row divided by 5. That fit
was withdrawn for its method, and its numbers were in fact exact; the rule of §57.1 is what shows it.

**The plain grade**, with the replacement's own sine at the 256 angles: 31 angles have a word differing, 56 words in
all, each by one, where the sine is the named loss of §43.2.

**Built.** `perspective(n)` in `chips/st010.rs` computes each line's scale from the formula, in integers, with no table.
`perspective_image()` writes the 176 words for the checks. `st010_tables.py --perspective` is the independent
generator, and `the_st010_perspective_equals_the_independent_generator` holds the two equal. The crate test of the
ST010 against the image now bounds 07h at one unit everywhere, from 640. The state is unchanged; its version stays 22.

**The grade** (`dsp_stgrade`, seeded phases and angles):

| Code | Cases | Latency differs | RAM differs | Largest difference |
|---|---|---|---|---|
| 07h | 65,536 | 0 | 1,967 (3.0%) | 1, at the sine's entries |
| 0Fh, its mirror | 4,000 | 0 | 109 | 1 |

**firmwarecheck** (`dsp_oracle tables` writes `st010.perspective.bin`, byte-identical to the generator's):

| Against | Plain | `--forced` |
|---|---|---|
| the data half | FAIL: the 352 bytes are the data ROM's first 352 bytes, a common run of the whole table at offset 0 | residual 0, **PASS**; formula coverage 679 of 4,096 bytes (16.6%) |
| the program half | 13.4% equal, longest run 3 bytes; PASS | residual 0, PASS |

So the chip holds its scale as a 176-word table in its data ROM, and the formula reproduces every word of it. Unlike
the sine (§43.2) and the angle (§54.3), the table has no entry that departs from the formula.

### 57.4 F1 ROC II in lockstep (measured 2026-10-04)

`dsp_lockstep`, before (the unit scale of §48.2) and after; the pictures are in the cache's `shots/`, the earlier ones
in `shots/before-perspective/`.

| Run | | Pictures equal | Pixels differing per frame, median, 95th percentile, largest | Chip RAM first parts |
|---|---|---|---|---|
| Attract mode, 7,200 frames, no input | before | 3,284 of 7,194 | 28.6%, 58.6%, 66.6% | 1,019 |
| | after | **6,918 of 7,194** | 0%, 0%, **0.83%** | 1,021, equal again at 1,022 |
| A player's qualifying, 2,700 frames (§54.4's script) | before | 1,468 of 2,708 | 0%, 61.3%, 67.3% | 1,468 |
| | after | **2,692 of 2,708** | 0%, 0%, **0.16%** | 2,097, equal again at 2,099 |

**The road is drawn in perspective, as on the image** (`f1roc2-f1800.png`, `f2300.png`, `f2600.png`, each the image,
the replacement and the differing pixels side by side). In the player's run the two pictures are equal in all but 16
frames, and those differ in at most 0.16% of their pixels, where a raster word at one of the sine's entries is off
by one. The chip's RAM now parts only for a frame at a time, at such an angle, and meets the image's again.

The attract mode's remaining differences start at frame 1,367, where §54.4's first angle miss changes an opponent's
turn. WRAM, OAM and the rest part from there as before, and the pictures differ in at most 0.83% of their pixels:
an opponent's car placed a few units off. The drivers (`dsp_cars`) are as §54.4 found, first placed differently at
frame 1,366.

**F1 ROC II now plays and looks as on the image without firmware.** It is held back by three named losses, each a
departure of the chip's own tables from their formulas at a few entries: the sine (§43.2), and through it 07h and 08h
by one unit; and the angle's two entries (§54.3), which move an opponent by a few units now and then.


### 57.5 How the exact-match rule was read for 07h (decided 2026-10-04)

The rule (§57.1) asks that every output of the command be reproduced exactly. For 07h the outputs the search could
decide are its 176 line scales, and each is reproduced exactly: the formula's table equals, word for word, the first
352 bytes of the chip's data, with no exception (§57.4). The array words are products of those scales with the sine and
cosine, whose two-entry departures from the formula are the named loss already recorded in §43.2. Reading "every
output" as the array words would reject a scale proven exact because of a loss the rule was not written to judge. The
reading recorded in §57.2 before the search, scales checked against the chip's own trigonometric words, is therefore
the one adopted; the built code uses the formula's sine throughout, and its one-unit array differences remain charged to
§43.2.

## 58. Super Mario RPG's p99: two neutral levers, and the SA-1 catch-up bound as a setting (2026-10-04)

§41.18 left G6's Super Mario RPG half open: a p99 of 18.2-18.9 ms at 33% quota against the budget's 16.6, from one
scene (timed frames 1,800-2,100), with optimisation on hold. **Decided 2026-10-04 by the tester: the fix goes ahead**,
for this scope only. The rule it was built under is the project's: the default stays exact, and a lever that changes
behaviour becomes an opt-in setting with its cost stated. The order was therefore fixed in advance: levers that change
nothing first, kept on by default; then plan §9 Q3's first lever, the SA-1 catch-up bound, made a setting unless it
too proved neutral; then the measurements.

### 58.1 The scene, profiled again (measured 2026-10-04)

A scratch replay tool runs the bench's script (Start and A tapped, Right held after frame 1,200) to frame 3,000, saves
the state, and replays frames 3,000-3,300 from it, so that the whole sample falls inside the heavy scene. Unquota'd
on the desktop the scene costs 3.79-3.85 ms a frame (best and worst of eight replays), against §41.18's p90 of 3.82.
The sampler was §41.18's ptrace sampler of instruction pointers, extended to name each sample by its innermost inlined
function and source line through `addr2line`, and samples inside libc by the caller's return address.

What that finer naming showed, against §41.18's coarser reading:

- **§41.18's "catch-up loop itself, 10%" was not the catch-up.** The lines it falls on are the two statements of
  `Sa1::run_until` that copy the SA-1's 65C816 out of the chip and back, around each instruction (4.4% and 6.3% of the
  scene). The copy exists because the CPU borrows the rest of the chip as its bus. It is paid once per SA-1
  *instruction*, not once per catch-up, so a bound on catch-ups could not remove it (argued here, measured in §58.4).
- **The compositor's priority walk is the largest single cost**: `Ppu::front` 23%, most of it on the loop over the
  mode's order and its per-entry tests.
- **3.7% is libc's `memcpy`**, called from `fill_row`'s variable-length copy of each decoded chunk into the line buffer.
- The SA-1's bus reads 5.1% and `rom_offset` 2.8%; the tile decode (`encode`, `decode_chunk`) 9.6%.

### 58.2 The neutral levers (built 2026-10-04)

| Lever | What it does | Scene, ms a frame (best of 8) |
|---|---|---|
| None (0c0335c1) | | 3.79 |
| The CPU taken out once per catch-up | `run_until` copies the 65C816 out of the chip once per run and back at its end; nothing else in the chip reads it during the run | 3.57 |
| The SA-1 bus's ROM decode as a per-bank table | each bank's linear window in ROM, (first offset, last offset, base), built from the MMC registers and rebuilt when one is written or a state loads; the vectors the SA-1 supplies at 00/80:FFEA-FFFD and every non-ROM address keep the decode as before; a bank is in the table only if its window is linear in the mirrored ROM | 3.41 |
| The compositor by rank | the mode's priority order turned once a span into a rank for each (layer, priority), and the front-most pixel found as the opaque layer of greatest rank, over at most five layers instead of up to twelve order entries; a layer absent from the order is never read, as before | |
| Tile rows eight pixels at once | each bitplane byte spread one bit a byte through a 256-entry table, so a chunk's eight colour numbers are assembled with one OR a plane, and byte-swapped for a horizontal flip | |
| A whole chunk copied at its fixed length | the line buffer takes an 8-entry chunk as a fixed-size copy, which compiles to moves; a partial chunk at a span's edge copies as before | |
| Brightness by table | the five-bit to eight-bit scaling at the span's brightness read from a 32-entry table made per span | 2.47 (the four PPU rows together) |

Together the scene's frame falls from 3.79 to 2.47 ms, by 35%. The first two rows are one commit (the SA-1's), the
last four another (the PPU's).

**The proof that they change nothing**, run on each commit:

- **State and picture hashes**: the state's FNV-1a hash every 60th frame and a running hash of every frame's picture,
  identical to the baseline's for the nine bench games and the five SA-1 games of §25.2 (Super Mario RPG, Kirby Super
  Star, PGA European Tour, PGA Tour 96, Power Rangers Zeo), each under the bench's script to frame 4,200 and with no
  input to frame 2,400.
- **The corpus**: the same hashes every 60th frame to frame 3,600 with no input, identical on all 295 ROMs of the
  manifest. Since every VenusRT column of the corpus runner is a function of the machine's state and pictures at frames
  1,800 and 3,600, those columns cannot move; the runner was also run on the final commit (§58.5).
- **Skip-versus-draw**: `skip_check` over the thirteen games for 1,200 frames, and the state hashes with the picture
  skipped equal to the drawn ones under the script to frame 4,200 for Super Mario RPG, Kirby Super Star, Yoshi's Island
  and Star Fox.
- **Unit tests holding the new paths to the old**: the rank against the order walked from the front over 20,000 random
  modes, enables, masks and pixels; the spread decode against the pixel-at-a-time decode at every depth and flip; the
  bank table against `rom_offset` for five ROM sizes (one not a multiple of 64 KiB, which the table refuses) and three
  MMC settings.
- **The conformance kit**, from the final commit's release library, on §31.2's eleven images with no boot file:
  compliant on every one.

### 58.3 The SA-1 catch-up bound (built 2026-10-04)

The SA-1 is caught up after every S-CPU instruction and before every S-CPU access to the board (§25.1). The bound
defers the first kind: under `sa1_catch_up` = 4 or 16 the SA-1 is caught up after that many S-CPU instructions, while
every S-CPU access to the board, its registers, I-RAM, BW-RAM and ROM alike, still catches it up first, as do a
frame's end and every instruction of a debugger's observed frame. A state is therefore never written with the SA-1
behind the S-CPU, and nothing new enters the state. Holds that ended before the SA-1's clock are now dropped as it
runs, since the deferred SA-1 meets more of them queued; that changes nothing.

How much it can defer was counted in the scene, by a counter not kept: per frame, 16,605 catch-ups that ran the SA-1
and 2,102 that found it already there; the S-CPU's accesses to the board were 6,162, of which only 1,223 were ROM. **The
S-CPU runs the scene from WRAM**, so most catch-ups are the per-instruction kind the bound defers.

**It is not neutral.** The default, 1, is today's behaviour, and its hashes equal the previous commit's everywhere.
At 2, 4 or 8, the pictures stay identical over every frame of all thirteen games' runs, but the state differs:

| sa1_catch_up | Super Mario RPG, frames whose state differs (every 60th, script / none) | Kirby Super Star | The other eleven games |
|---|---|---|---|
| 2 | 60-360 and 540 / 60-360 | none / none | none |
| 4 and 8 | the same | 14 frames between 240 and 4,020 / 25 between 300 and 2,400 | none |

The likely reading, not measured: the SA-1's position at a frame's end and the D-36 holds it met inside the span
differ, and the games' own loops absorb the difference, as §28.2 found for contention generally. The setting is
therefore declared with `effect: accuracy`:

- **key** `sa1_catch_up`, a choice of 1 (exact, the default), 4 and 16, in the Performance category, marked advanced,
  applied between frames;
- **cost**, in the words the frontend shows: the SA-1 can fall up to 4 or 16 instructions of the console's CPU
  behind it; its interrupt to that CPU can arrive that much later, and the two processors' contention for ROM and
  BW-RAM inside that span is timed differently from the console's; some games' internal state then differs from the
  exact setting's at some frames, though the pictures measured were the same; it saves about 3% of the time of Super
  Mario RPG's heaviest scene, and nothing in games without an SA-1.

The core declares `SETTINGS` among its capabilities; the registration golden gained only the setting's rows.

### 58.4 Super Mario RPG at 33% quota (measured 2026-10-04)

Every row is the bench through `ICore` (`venusrtbench`, rebuilt against this branch so that a setting can be passed),
with each build's library swapped in, under `flock ~/.cache/emusen/probe/timing.lock` and
`systemd-run --scope -p CPUQuota=33% -p CPUQuotaPeriodSec=5ms`, the variants interleaved within each round. The
load average was read before each run. The bench's own state hash after 4,200 frames, A13DD00F173D23E2, was the same
for every build and setting. p99 in ms:

| Round (load) | Baseline | + SA-1 levers | + PPU levers (the defaults) | Defaults, again | Defaults, catch-up 16 | Defaults, catch-up 4 | Baseline + bound only, 1 | Baseline + bound only, 16 |
|---|---|---|---|---|---|---|---|---|
| 1 (4.0-8.2) | 19.72 | 20.81 | 18.98 | 19.14 | 18.80 | 18.80 | 19.75 | 19.56 |
| 2 (3.2-5.3) | 18.95 | 15.60 | 15.08 | 14.90 | 14.27 | 13.88 | 18.86 | 18.80 |
| 3 (2.9-8.0) | 18.75 | 15.38 | 13.71 | 13.87 | 15.21 | 17.80 (load 6.7) | 19.30 | 19.13 |
| 4 (2.7-6.5) | 18.91 | | | 14.22 | 14.17 | 15.98 | | 18.59 (load 6.5) |
| 5 (2.1-3.9) | 15.34 | | | 13.89 | 13.42 | 16.01 | | 22.19 |
| 6 (2.2-2.6) | 20.08 | | | 13.90 | 13.56 | 13.72 | | 18.44 |
| All-games round 1 (2.0) | 18.63 | | | 13.75 | | | | |
| All-games round 2 (1.5) | 18.12 | | | 13.57 | | | | |

The means over the rounds at load under 5.5: baseline 11.1-12.0 ms; with the SA-1 levers 10.5; with the defaults
8.2-8.8; at catch-up 16, 8.2-8.7; at 4, 8.3-8.9; the bound alone on the baseline 11.7-12.8. Unquota'd, the defaults
run the bench at a mean of 2.51-2.61 ms and a p99 of 2.87-2.91, against §41.8's 3.33 (3.88). On the desktop, the
scene replays at 2.42-2.50 ms a frame with the bound at 4 to 64 against 2.52-2.58 at 1, and on the baseline at 3.86
with the bound at 16 against 3.93 at 1.

- **The defaults meet the budget.** In every round at load average under 5.5, Super Mario RPG's p99 is 13.6-15.1 ms
  with the defaults (the PPU commit's and the final commit's runs together; the final commit's alone 13.6-14.9),
  against the baseline's 15.3-20.1 (median 18.8). The SA-1 levers alone bring it to 15.4-15.7.
- **The bound adds nothing measurable to the p99.** At 16 its median is 14.2 ms against the defaults' 13.9; at 4 the
  spread is wider (13.7-16.0). It lowers the mean by about 3%, as on the desktop.
- **Under heavy load nothing meets it.** Round 1 ran while other test hosts loaded the desktop to 4-8, and every
  variant's p99 was 18.8 ms or more. The quota proxy is sensitive to the machine's load as well as to the scene's work,
  so the verdict below rests on the rounds at the load §41.8 ran at (3.4-3.9) or lower.

**The predictions.** §41.18 predicted (its P8), before any measurement, that the bound alone takes the p99 under 16.6
ms and the decode table alone does not. Written before this section's matrix, from the desktop measurements of
§58.1-§58.3 and one pair of quota runs on the first bench:

- **P8, retired, false in both halves.** The bound alone on the baseline leaves the p99 at 18.4-22.2 ms (P58.3 below).
  The SA-1 commit, the decode table with the CPU copy, brings it to 15.4-15.7, under the budget. P8's reading of the
  sampler took the copy for the catch-up.
- **P58.1, false by a hair**: with the defaults the p99 was to be at most 15.0 ms in every round. At load under 5.5 the
  final commit's largest is 14.90, but the PPU commit, the same machine one commit earlier, gave 15.08 in round 2; and
  round 1, at load 6.9, gave 19.14.
- **P58.2, holds**: the bound at 16 lowers the p99 by less than 1 ms against the defaults; it does not lower it
  measurably at all.
- **P58.3, holds**: the bound alone on the baseline leaves the p99 above 16.6 ms, in every round.
- **P58.4, holds**: no other bench game is worse (§58.5).

### 58.5 Every bench game, and the corpus runner (measured 2026-10-04)

The nine bench games at 33% quota, the baseline against the defaults, two interleaved rounds at load 1.4-3.2; mean
(p99) in ms, and the bench's state hash equal between the builds in every row:

| Game | Baseline, round 1 | Defaults, round 1 | Baseline, round 2 | Defaults, round 2 |
|---|---|---|---|---|
| Super Mario World | 7.10 (13.75) | 5.17 (9.56) | 6.85 (10.29) | 5.18 (9.91) |
| Super Metroid | 6.66 (10.38) | 5.42 (9.26) | 6.72 (10.33) | 5.49 (9.81) |
| Donkey Kong Country | 6.32 (10.39) | 4.87 (9.64) | 6.26 (10.59) | 4.77 (9.46) |
| A Link to the Past | 7.67 (10.08) | 5.89 (9.37) | 7.61 (13.31) | 5.61 (9.25) |
| Yoshi's Island (GSU-2) | 8.67 (14.28) | 6.94 (13.87) | 8.65 (14.20) | 6.72 (10.33) |
| Star Fox (GSU) | 7.24 (10.20) | 6.13 (10.35) | 7.61 (13.57) | 6.15 (10.17) |
| Super Mario RPG (SA-1) | 11.67 (18.63) | 8.30 (13.75) | 11.22 (18.12) | 8.41 (13.57) |
| Pilotwings (DSP-1, image) | 8.78 (14.45) | 6.94 (10.12) | 8.51 (14.33) | 6.94 (12.91) |
| Super Mario Kart (DSP-1B, image) | 9.59 (14.51) | 7.81 (13.18) | 9.47 (14.36) | 7.83 (13.42) |

Every mean falls, by 15-29%, the PPU levers acting on every game. Every p99 but one falls or stays within the rounds'
spread; Star Fox's round-1 p99 is 0.15 ms higher, inside it.

**The corpus runner** (`The_corpus_reproduces_the_recorded_baseline` with VenusRT as its third engine, no firmware,
six threads), on the final commit and, with the baseline's library swapped into the same test build, on the baseline:
**every VenusRT column of all 295 ROMs identical** between the two (verdict, detail, and VRAM, CGRAM, OAM and picture
against Mesen's), with 112 self-graded passes, §41.19's count without firmware. C# Venus's and Mesen's cells reproduced
the committed baseline in both runs.

### 58.6 G6, and what is left

**G6's Super Mario RPG half is met** at the defaults: its 33%-quota p99 is 13.6-14.9 ms in every round at the load
§41.8 measured under, and every bench game's mean and p99 is within the budget. No accuracy was traded for it: the
defaults' machine is the baseline's, frame for frame. G6 as a whole is still **not met**, because the handheld's battery
run (§41.8) is owed.

Not done, and why:

- **The SA-1's hold queue** (`Sa1::access`, 9.5% of the scene after the levers) and the compositor's per-pixel colour
  math are the largest costs left. Both could be made cheaper without changing behaviour; neither is needed for the
  budget, so neither was built.
- **The bound stays an opt-in setting**, though it does not help the p99. It is kept because it is plan Q3's named
  lever, it lowers the mean, and a weaker machine than the quota proxy may be bound by the mean; its cost is stated.

## 59. Blocker 3: `test_irqb` case 5, D-24 settled by elimination (2026-10-05)

§41.10's blocker 3 proposed the referee's 65C816 interrupt sequence, in the logged step D-24 named. That step was
taken at §41.16 and did not settle the entry: the referee strobes no read on an internal cycle, so it reads $2180 once
where the ROM's console expectation is twice. The next rung, Mesen's source, is outside what this work may read. The
entry was settled instead from the ROM's own five console expectations, with Mesen run as a black box to locate the
access (`VenusRT_Disputes.md` D-24, 2026-10-05).

### 59.1 The argument, and what was measured

- **Measured: the second read costs no time.** Mesen's CPU trace through the probe and VenusRT's `cpu_trace` are the
  same from power-on to the handler of case 5, instruction by instruction, with every instruction's master clocks
  equal. The handler's record holds the same latched counters in both (H $06, $7A and $CF); Mesen's starts one byte
  further on in WRAM. The extra WMDATA read is therefore made in a cycle VenusRT already times.
- **Argued: two cycles qualify.** Only two internal cycles address $2180 there: CLC's second cycle (Table 5-7, row 19a,
  PBR,PC+1) and the interrupt's second (row 22a, PBR,PC).
- **Measured: case 1 excludes the interrupt's.** Its interrupt's internal cycle addresses $2137, and a read there
  moves the latched H from 7 to 9 (D-24, 2026-10-02). §41.16's remark that a strobe on every internal cycle fits cases
  1 to 4 overlooked that row 22a addresses the cycle at PBR,PC; it is struck through in the entry.
- **Measured: how far the rule reaches.** Two trial builds, not kept: T1, the second cycle of row 19a's 25 opcodes and
  XBA's; T2, every internal cycle but the interrupt's. Both pass all five cases. Over the corpus's 304 images and 17
  games (the bench set and the goldens), each state hashed every 60 frames to frame 3,600 with no input:
  - T1 changes `test_irqb` alone;
  - T2 also changes VitorVilela7's `speed_test_v51` from frame 60, and Super Mario RPG at frame 3,600, where a read
    of the SA-1's addresses catches the SA-1 up.

  `speed_test_v51`'s VRAM at frame 3,600 is 42 bytes from Mesen's under either build, so no ROM here separates the two
  readings by an oracle.

### 59.2 What was built

The narrowest reading, T1, restricted to the B bus. The internal second cycle of a one-byte implied instruction strobes
a read of PBR,PC+1 when that address is a B-bus register ($2100-$21FF in banks $00-$3F and $80-$BF). The byte is
dropped and the MDR kept, as anomie's open-bus document says of internal cycles. In code it is `Bus::implied_cycle`,
whose default is an ordinary internal cycle, so only the S-CPU's bus answers it: the single-step harness and the
SA-1's 65C816 are unchanged. Open, and named in the entry:
- whether other internal cycles strobe too;
- whether the A bus and the S-CPU's registers at $4000-$43FF see the strobe;
- XBA's third cycle.

### 59.3 Checks (measured 2026-10-05)

| Check | Result |
|---|---|
| `test_irqb` | passes, all five cases, with VenusRT's boot program and with the tester's image |
| The corpus and 17 games, state hashes every 60 frames to 3,600 | only `test_irqb` differs from the previous commit's; every hash equals T1's |
| The 65816 single-step suite | `the_cpu_through_the_whole_suite` passes: D-1, D-2 and D-3's cases and no others |
| The crate | 137 of 137, among them `an_implied_instructions_internal_cycle_reads_a_b_bus_register`, shown failing with the read disabled |
| WiseMan, VenusRt, CoreAbi, Snes, CoreDebug and Conform filters, the goldens against Mesen included | 121 of 121; all 36 anchors keep their recorded outcomes and hashes |
| `frame_cost`, best of three over 1,200 frames, base and new interleaved three times under the timing lock (load 2.0-2.8) | Super Mario World 1.538-1.545 against 1.530-1.539 ms; A Link to the Past 1.401-1.406 against 1.398-1.404; Super Mario RPG 2.089-2.106 against 2.096-2.120; Yoshi's Island 1.380-1.385 against 1.383-1.390. Within the runs' spread: no cost |

The corpus runner itself was not rerun. Its VenusRT columns are functions of the machine's state and pictures at
frames 1,800 and 3,600, and those hashes are unchanged on every image but `test_irqb`, as §58.2 argued for its levers.

### 59.4 G1 after blocker 3

**114 of Mesen's 117** self-grading passes with the tester's images, and 113 without, against §41.19's 113 and 112.
`spc_smp` is still the one ROM excepted without images. Every C# Venus pass passes. Left:
- `spc_dsp6` (§41.17);
- `test_timer_stop2` (D-26);
- `blobs/test_hdmatiming` (D-40's test 2, one dot).

## 60. Blocker 4: `test_timer_stop2`, D-26 settled as argued with a measured band (2026-10-05)

§41.10's blocker 4 asked for two steps: the SPC700 program the ROM uploads, disassembled with VenusRT's own
disassembler, and then the referee's TEST gating. Both were done (`VenusRT_Disputes.md` D-26, 2026-10-05).

### 60.1 What the program does

The ROM's program was read with `disasm/spc700.rs`, each instruction decoded from APU RAM in the frame it first ran.
`test_timer_stop` was read the same way for comparison.

| | `test_timer_stop2` (console 04) | `test_timer_stop` (console 00) |
|---|---|---|
| Timer | 0, T0DIV 2, CONTROL $87 | 0, T0DIV 1, CONTROL $81 |
| Synchronised | T0OUT read until non-zero, just after a first-stage tick | the same |
| Stopped by | TEST $0B (bit 0 set) | TEST $02 (bit 3 clear) |
| Pattern | $0B and $0A in turns of 4 cycles, 14 of each, over 115 cycles | ten loops of 128 cycles, about 72 stopped and 56 running |

A cycle trace of VenusRT, from a scratch tool stepping the SPC700 alone from the sync, puts the 14 stops at cycles 236
to 340. One first-stage tick falls in the window, and VenusRT counted one stage step and printed 00. To print 04, the
second stage has to step eight or nine times in 115 cycles. A rule that only stops and starts the count gives at most
one, whatever its phase.

### 60.2 The referee

`SMP.vhd` was read as logged in the entry. Its second stages count on the first stage's single-cycle event, with
TEST's enable sampled at that moment, so stopping and starting the timers can drop a tick and never add one. It prints
00 here. The referee's timers are written as clock enables. On this rule its agreement is not evidence either way, and
here it disagrees with the console.

### 60.3 The model, and how far the two ROMs bound it

**Argued:** a gated clock. Each first stage drives a level that falls at its tick. TEST's enable gates that level, and a
second stage counts the gated level's falling edges. With TEST held constant this is exactly today's tick. Stopping the
timers while the level is high makes an edge, and so a count.

**Predicted before the trial was built:** with the level high over the period's second half (64 cycles at TEST's
default), 8 of the 14 stops fall while it is high, which gives 8 or 9 steps, so T0OUT is 4.

**Measured:** the trial printed 04. Its one parameter, the level's length, was then swept on both ROMs:

- `test_timer_stop2` is 04 for thresholds 176 to 216 of the first stage's 384;
- `test_timer_stop` is 00 from 184 upward.

Together the two hold the level to 168-200 of 384, 44% to 52% of the period. Half the period lies inside. The 2026-10-04
trial counted the rising edge of the same gated level, and no length keeps `test_timer_stop` at 00 with that edge.
CONTROL's per-timer bit is left as D-25 has it, an enable read at the edge, since no ROM here toggles it against a high
level.

### 60.4 What was built, and the checks (measured 2026-10-05)

Built as stated, with the level high from 192 of 384 for timers 0 and 1 and from 24 of 48 for timer 2. A cycle in which
TEST is unchanged keeps the previous rule, a wrap with the gate open. A cycle in which TEST changed compares the
levels under the old and new values. TEST as the last cycle saw it equals TEST at every instruction boundary, so the
state format is unchanged.

| Check | Result |
|---|---|
| `test_timer_stop2`, `test_timer_stop` | 04 and 00, both passing, with VenusRT's boot program and with the tester's image |
| The corpus with and without the image, and the 17 games, state hashes every 60 frames to 3,600 | only `test_timer_stop2` differs from §59's build |
| The crate | 138 of 138, among them `stopping_the_timers_while_the_first_stage_is_high_counts_once`, shown failing on the previous rule |
| WiseMan, the VenusRt, CoreAbi, Snes, CoreDebug and Conform filters, the goldens against Mesen included | 121 of 121 |
| `frame_cost`, best of three over 1,200 frames, the two builds interleaved three times or more under the timing lock (load 2-7) | Super Mario RPG 2.090-2.107 against 2.105-2.132 ms, Yoshi's Island 1.383-1.395 against 1.396-1.400, Super Mario World 1.534-1.547 against 1.553-1.563. About 1% |

The cost is the comparison of TEST in each SPC700 cycle. Two other formulations measured the same, one keeping the
gated levels as booleans and one with the changed-TEST path inline instead of cold. Optimisation is on hold, so it
stands as measured.

### 60.5 G1 after blocker 4

**115 of Mesen's 117** with the tester's images, and 114 without (§59.4's 114 and 113). Every C# Venus pass still
passes. Left:
- `spc_dsp6` (§41.17);
- `blobs/test_hdmatiming` (D-40's test 2, one dot).

D-26's structure is argued and its band measured. A console ROM sweeping the toggle's phase across the first stage's
period would measure the level directly.

## 61. Blocker 1's remainder: `blobs/test_hdmatiming`'s test 2 was WDM, not HDMA (D-41, 2026-10-05)

§41.14 left one row of `test_hdmatiming` open, test 2's second latch: $34 against the console's $35. D-40 recorded
that no sampling point and no resume rule fixed it. The cause is outside HDMA.

### 61.1 What was found

- **The CPU's position, measured.** The first latch is equal ($006). The code between the two latches is the same as
  test 1's, whose row is equal. A trial that delayed the CPU's resume after test 2's HDMA run by 2 or 3 clocks gave
  the console's row, and 4 or more moved the first latch. VenusRT's CPU therefore stands 1 to 3 clocks early within
  a dot after that run.
- **Why, from the ROM's source.** Test 2 differs from test 1 by three `WDM #$00` in front of its NOPs, with the comment
  "CPU sync cycle = 8" against test 1's "= 6". In VenusRT a WDM cost a NOP's 14 clocks, an opcode fetch and an
  internal cycle, so the three moved nothing. VenusRT's cycles around H-blank on line 0 were the same in both tests.
  `seek_frame` returns at line 0, clock 0 in both, as its header promises.
- **Measured: WDM's second cycle as a program fetch.** Then it costs 8 clocks in this slow ROM, and all eight graded
  rows equal the console's. The HDMA rules of D-39 and D-40 are unchanged.
- **The referee** (logged in D-41) drives VPA in WDM's second cycle, the pair its immediate-operand rows drive. The
  datasheet has no Table 5-7 row for WDM, so this is the implementer's choice, not a transcription. It agrees with
  fullsnes's "WDM #nn" and with anomie's rule that internal cycles take 6. SingleStepTests records the cycle as
  internal. The reading built is the fetch, against the suite.

### 61.2 Checks (measured 2026-10-05)

| Check | Result |
|---|---|
| `blobs/test_hdmatiming` (the same image as `test_hdma/` and `test_mdrhdma/`) | passes, the eight graded rows equal to the console's, with and without the tester's boot image. The four ungraded rows (HDMA during DMA) are as before, 1 to 2 dots from the ROM's table |
| State hashes every 60 frames to 3,600, the 304 corpus images and 17 games | changed: `test_hdmatiming`, gilyon's `cputest-basic` and `-full`, PeterLemon's `CPUMSC`, `test_mul`, `test_hello`, the two `snes_mul_div_timing` printers. None of the games |
| The corpus runner with VenusRT (C# Venus beside it, without Mesen, graded against the baseline's Mesen verdicts) | **116 of Mesen's 117** with the tester's images, 115 without; every C# Venus pass passes. gilyon's two builds and `CPUMSC` still pass |
| Pictures | `test_mul`'s picture and VRAM unchanged; `test_hello`'s VRAM equal to Mesen's, and its picture 809 pixels from Mesen's against 929 before |
| The 65816 suite | 5,059,735 of 5,120,000: the 60,265 failures exactly D-1's 265, D-2's 40,000 (D-3 within them) and D-41's 20,000 |
| The crate | 139 of 139, `wdm_fetches_its_second_byte` among them, shown failing on the old rule |
| WiseMan, the VenusRt, CoreAbi, Snes, CoreDebug and Conform filters, the goldens against Mesen included | 121 of 121 |

No cost was measured: none of the 17 games' states changes, so none executes WDM in its first 3,600 frames.

### 61.3 The gates after it

- **G1: 116 of Mesen's 117** with the tester's images and 115 without. The two missing without images are `spc_smp`,
  which §41.3's proposal excepts, and `spc_dsp6`. Left: `spc_dsp6` alone (§41.17).
- **G2: met with named exceptions**, now four entries' worth: D-1, D-2 and D-3 as before, and D-41's 20,000 WDM
  cases. D-41's exceptions differ from D-2's in kind. D-2's pins cannot be observed on the SNES's bus. WDM's second
  cycle can: its timing is what `test_hdmatiming`'s console table measures.
- **G3:** `test_hello` is closer to Mesen's picture, still one of §41.19's two unlogged rows.

## 62. Blocker 5: `spc_dsp6`, singly and in order (D-42, D-43, 2026-10-05)

§41.17 left `spc_dsp6` failing in order at "Random/brr while playing", with five tests hanging singly. The single-test
copies of §23.1 were found in scratch, so the splitter was not rebuilt. Each copy was graded as §23.1 grades it,
after 600 frames and again after 4,000 for those still running.

### 62.1 The singles, before

| | VenusRT's program | The tester's image | Mesen |
|---|---|---|---|
| 600 frames | 105 pass, 6 running | 106 pass, 5 running | 3 of the 6 also still running: "Echo/edl lengths", "Random/echo data", "Random/echo fir" |
| 4,000 frames | "Random/brr while playing", "Random/echo data" and "Random/envelope" fail with a checksum; "Order/voice 0 noise" hangs | the same three fail | all pass |

Two of §41.17's "hangs" were only slow, as in Mesen. What was left was three checksum failures and one hang.

### 62.2 How the checksums were located

The random tests print nothing but a checksum, so each was disassembled with VenusRT's own disassembler and copied in
scratch with its own SPC700 program patched. Each copy runs a shorter test and always prints its checksum, so that
VenusRT and Mesen could be compared at any length. This is a test ROM's own code, changed and run on both engines as
black boxes.

- **"Random/envelope"**, cut to one round of K iterations:
  - Equal checksums for every K up to 99, and the whole APU RAM equal at K = 99.
  - At K = 100, the round's first KON, the echo buffer shows VenusRT keying voice 0 on one sample later than Mesen.
  - VenusRT's KON poll fell 30 and 94 cycles after the timers' T0 and T1 tick. anomie's S-DSP document gives two
    possible phases and calls 62 and 126 the more frequent at power-on. VenusRT's poll was at the other one.
  - Built as D-42.
- **"Random/echo data"**, cut to n rounds:
  - The first round is equal and the second is not.
  - The difference was traced to the 4-byte echo loop that EDL 0 leaves between the rounds. There, with every FIR
    coefficient at -128, Mesen's right sample holds at $8000 and VenusRT's cannot, because VenusRT summed the taps'
    products in 32 bits.
  - The products wrapped to 16 bits, the arithmetic both documents describe, give the console's checksum. Built as
    D-43.

### 62.3 The singles and the in-order run, after (measured 2026-10-05)

| | VenusRT's program | The tester's image |
|---|---|---|
| Singly, by frame 4,000 | **111 of 111 pass** | **110 pass**; "Order/voice 0 noise" hangs |
| In order, at frame 18,000 | "PASSED TESTS" | "PASSED TESTS" |
| The corpus runner's verdict (backdrop) | **Passed** | **Passed** |

**"Order/voice 0 noise" singly depends on the program's phase.** Swept over 35 S-CPU power-on positions (0 to 1,360
clocks in steps of 40), it passed at 31 (image) and 25 (VenusRT's program) before D-42, and at 4 and 10 after. The
counts are complementary, so the test does not fix its own phase against the KON poll. The other order of D-30's
noise step fails it, and "noise rate flg.1F", at every position. A third trial, the KON poll after voice 0's step in
cycle 30, failed 39 and hung 9 of the 111 singles, and was rejected. The single test is recorded as open. The whole ROM
passes it in order, and that is what G1 grades.

### 62.4 Checks (measured 2026-10-05)

| Check | Result |
|---|---|
| Pictures, every frame to 3,600, the 304 corpus images and 17 games | only `spc_dsp6` changes |
| State hashes | change everywhere, as the phase is part of the DSP's state from power-on; the pictures above are the comparison |
| The corpus runner (C# Venus beside it, without Mesen, graded against the baseline's Mesen verdicts) | **117 of Mesen's 117** with the tester's images, **116** without, the one missing `spc_smp`; `spc_dsp6` is the one verdict that changed in either run; every C# Venus pass passes |
| The crate | 141 of 141, the two new tests (`the_kon_poll_falls_62_and_126_cycles_after_the_timers_tick`, `a_fir_tap_at_minus_128_wraps_in_16_bits`) shown failing on the previous rules |
| WiseMan, the VenusRt, CoreAbi, Snes, CoreDebug and Conform filters, the goldens against Mesen included | 121 of 121 |
| `frame_cost`, best of three over 1,200 frames, before and after interleaved three times under the timing lock (load 1.1-2.1) | Super Mario World 1.554-1.567 against 1.556-1.567 ms, Super Mario RPG 2.113-2.119 against 2.104-2.110, Yoshi's Island 1.398-1.404 against 1.397-1.409, Donkey Kong Country 1.537-1.545 against 1.525-1.539: no cost |

### 62.5 The gates after blocker 5

- **G1 is met** under §41.3's reading for the firmware policy:
  - with the tester's images, VenusRT passes all 117 of Mesen's self-grading passes and every C# Venus pass;
  - without images it passes 116, and the one missing is `spc_smp`, the ROM that list names because it reads the
    console's boot ROM.

  No exception rests on a ruling against Mesen.
- **Still open, outside G1:** "Order/voice 0 noise" run singly with the tester's image (§62.3). The single copies are a
  diagnostic, not a gate ROM.
- **G2, G3, G4, G6, G7, G8:** as after §61. G2's named exceptions are unchanged (D-42 and D-43 touch the S-DSP, which the
  single-step suites do not reach).
- **G5:** the goldens keep their 36 outcomes and hashes. Yoshi's Island's stork and NHL '94's puck are the remaining
  blocker, §41.10's 10.

