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
