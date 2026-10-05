# VenusRT_Plan — a clean-room SNES core in Rust

*Written 2026-09-30, before any VenusRT code.* This page plans VenusRT, a new SNES core written in Rust from hardware
documentation and graded by test ROMs, to replace Venus. It is not a port. MarsRT, MercuryRT and MoonRT were ports:
each was graded byte for byte against its C# core and read that core's state format. VenusRT is graded against the
hardware's own test programs and against Mesen run as a black box, and nobody writing it reads C# Venus.

The page keeps three things apart:

- **Measured** means a number taken on 2026-09-30 on the desktop (Ryzen 7 7700X, Fedora 44, .NET 10, Rust 1.98.1),
  with the tools named beside it. The raw results are outside the repository, in `~/.cache/emusen/probe/venusrt/`
  (§3.6).
- **Argued** means reasoning from the documents, or from the other ports' measurements, with no measurement behind it
  here.
- **Predicted** means a claim numbered P1–P7 (§11) so that a later stage can retire it.

Where this page describes the old core, it describes only what can be seen from outside it: its `ICore` members,
the interfaces it declares, its save files, its debugger surface, and what it does when run over test ROMs (§4, §3.5).
It says nothing about how Venus works inside, and the failures in §3.5 are recorded without explanation on purpose.

---

## 1. Purpose and the clean-room protocol

### 1.1 Why a rewrite and not a port

Venus was EmuSen's first core, and parts of it were taken from Mesen during early development. The project's research
value rests on its cores being independent implementations: a finding that "EmuSen agrees with Mesen" means nothing if
EmuSen's rule was Mesen's to begin with. Venus cannot be made independent by editing it, because nobody can say which
of its rules survived from the copied code. *Decided 2026-09-30:* Venus is replaced by a clean rewrite, VenusRT, and
C# Venus keeps running SNES games in Mistress until VenusRT reaches parity (§7). Then it moves to a legacy branch
(§9, Q9).

### 1.2 Who may read what

A **writer** is anyone who writes VenusRT's code, its tests or its design pages. A **dispute step** is a separate,
supervised step whose only product is one entry in the disputes log (§1.4). It changes no code.

| Source | A writer | A dispute step | Notes |
|---|---|---|---|
| Hardware documents (§2): fullsnes, the SNESdev wiki, anomie's documents, datasheets | yes | yes | The primary source |
| Test ROMs and their own sources and READMEs (§3) | yes | yes | A test's source says what the test expects; that is its purpose |
| Mesen, **run** through the reference probe | yes | yes | Outputs only: memory, picture, sound, traces (§1.3) |
| Mesen's **source** (`~/Projects/mesen-reference`) | **no** | only the files that bear on the one disputed rule, each logged | Never transcribed; the log records the files and the conclusion in prose |
| SNES_MiSTer's RTL (`~/Projects/snes-mister-reference`) | **no** | yes, before Mesen's source, each file logged | The referee (§1.5); never transcribed |
| C# Venus, **run** as a black box through `ICore` | yes, as a baseline | yes, as a baseline | Never an expected value (§1.6) |
| C# Venus's **source**, and the algorithm sections of its pages (`Venus_CPU.md`, `Venus_PPU.md`, `Venus_APU.md`, `Venus_Memory.md`, `Venus_SA1.md`, `Venus_SuperFX.md`, `Venus_NecDSP.md`, `Venus_OBC1.md`, `Venus_Referee.md` §1–§9) | **no** | **no** | Not even in a dispute step |
| C# Venus's **public surface**: its `ICore` members, declared interfaces, save files, debugger surface, cheat codecs' input formats | yes | yes | What parity means (§4) |
| Other emulators' source (bsnes/higan/ares, snes9x, blargg's `snes_spc`, and the rest) | **no** | **no** | §9, Q4 |
| The SNES Development Manual (Nintendo's, circulated without permission) | **no** | **no** | §9, Q1; public documents that cite it are read as any document is |

Three rules follow from the table:

- **The order of recourse is fixed.** A dispute is settled first by the documents, then by a test ROM written for the
  purpose, then by the RTL, and only then by Mesen's source. An entry that cites Mesen's source says why the three
  before it were not enough.
- **Nothing read in a dispute step is typed into code in that step.** The step ends with a rule stated in hardware terms
  ("the write lands two dots later"), and a later writer's step implements that sentence and cites the entry.
- **The core's code carries no reference to an emulator's identifiers.** A name, constant or table layout that appears
  only in an emulator's source is a finding against the protocol. §7's gate G8 checks it mechanically.

### 1.3 How Mesen is used

Mesen is a black-box differential. The reference probe (`EmuSen.WiseMan/Reference/build-probe.sh mesen`, §3.4) runs a
ROM in it headlessly, applies a scripted pad, and writes VRAM, CGRAM, OAM, WRAM, APU RAM and the screen at chosen
frames, with CPU and GSU traces and the audio when asked.

- **What agreement means.** Mesen is widely regarded as accurate, so agreement with it is evidence that a rule is
  right. It is not proof: Mesen is not hardware, and §3.5 has one self-grading ROM that Mesen fails too.
- **What disagreement means.** A disagreement opens a dispute (§1.4). It never licenses copying Mesen's behaviour into
  the core.
- **What cannot be compared directly.** Two emulators do not reach frame *n* in the same machine state, because power-on
  timing differs by cycles. Comparisons are made on a self-grading ROM's result, on a picture that has stopped changing
  (§3.5 checks frames 1800 and 3600), or at an anchor the probe waits for (`--pressuntil`). The comparability gate of
  `EmuSen_Debugging_Tools_Reference_v5.md` §3.48 applies unchanged.

### 1.4 The disputes log

The log is a section of `VenusRT_Disputes.md`, created with the first entry. Each entry has this shape:

```
### D-<n>. <component>: <the rule, one sentence, in hardware terms>
- Opened: <date>, by <what showed the disagreement: ROM, frame, space, the two values>
- Documents read: <title, section> for each
- Test ROM: <the ROM written or run to settle it, its result on VenusRT and on Mesen>
- Referee: <RTL files and lines read, and the weight of that subsystem per Venus_Referee.md §0>
- Mesen's source: <none> | <commit, each file path read, and why the three above did not settle it>
- Conclusion: <the rule as implemented, in prose>; argued | measured
- Pinned by: <the test that fails if the rule is changed>
- Implemented in: <the later commit that cites this entry>
```

An entry is never deleted. A conclusion that is later overturned is struck through and followed by the new one, with
the evidence that overturned it.

### 1.5 How the MiSTer core referees

`Venus_Referee.md` §0 established, before any finding, what the SNES_MiSTer RTL's agreement is worth subsystem by
subsystem: real support for its 65C816 (transcribed from the WDC datasheet), its SPC700, the S-CPU's DMA and HDMA
machines, its PPU, SA-1 and NEC DSP; weak support for the S-DSP's arithmetic, which follows blargg's; nearly none for
the multiply and divide unit and the cartridge header scorer, which follow emulators'. That table is reused as the
weighting here. Only §0 of that page is cited: its later sections describe Venus's internals and are outside the
protocol.

The checkout is `github.com/MiSTer-devel/SNES_MiSTer` at `c61bfd4` (2026-09-17). It includes, beside what Venus
supports, `rtl/chip/` folders for the Cx4, S-DD1, SPC7110, S-RTC, BS-X and Sufami Turbo, which matter only after retirement,
when §9's Q8 takes them one at a time.

### 1.6 Why C# Venus is excluded as an oracle

- **It would import Mesen's rules by another road.** Grading against an oracle partly derived from Mesen makes
  VenusRT's later agreement with Mesen uninformative, and informative agreement is what the rewrite exists to restore.
- **It is measurably less accurate than the oracles that are available.** On the self-grading ROMs Venus passes 73
  of the 93 on which Mesen passes, and 9 of the 20 it fails are about timing (§3.5). Grading against it would pin its
  failures.
- **A byte-exact oracle shapes the design.** The three earlier ports were exact against their C# cores and so kept
  their C# cores' structure (`Moon_Native.md` §5, Q4). VenusRT's structure is to come from the hardware (§5), and an
  exactness target would pull it back.

What Venus is used for is a **baseline**: its results on the same ROMs, so that §7 can require VenusRT to do no worse
anywhere, and its speed, so that §5.5 can price the new design against the frame the player gets today.

### 1.7 What was read to write this page

So that the protocol can be audited from its first day, this is everything of C# Venus that was read for this page:
the declarations and the load, frame-buffer, sound, state and SRAM members of `VenusCore.cs`; the public members of
`Cartridge.cs` and its `FirmwareRequirements`; the chip names and memory sizes in `NecDspVariant.cs` and the file
names in `NecDspFirmware.cs`; the public members and the space list of `SnesDebugTarget.cs`; the codec entry points of
`VenusCheatCodecs.cs`; the SNES rows of `CoreFactory.cs` and `CoreCatalog.cs`; and `Venus_Referee.md` §0. A text
search also showed single lines of `Venus_CPU.md` and `Venus_APU.md` that mention the single-step harness's pass
counts, and a search of `Cartridge.cs` for its save behaviour showed single lines of its constructor and its read
path. No CPU, PPU, APU, DMA or coprocessor implementation was opened, and no Mesen source.

---

## 2. Sources

### 2.1 The documents the core is written from

| Source | Where | Version | What it covers |
|---|---|---|---|
| **fullsnes** (Martin Korth, nocash) | <https://problemkaputt.de/fullsnes.htm> | `Last-Modified: 2021-01-28`; 1,524,441 bytes; SHA-256 `3853b65a7331bf12…` (copy in `~/.cache/emusen/probe/venusrt/docs/`) | The whole machine: memory map, every register, CPU and DMA timing, PPU, APU, controllers, and the cartridge chips including SA-1, GSU, DSP-n, ST01x, OBC1, Cx4, S-DD1, SPC7110 |
| **SNESdev wiki** | <https://snes.nesdev.org/wiki/> | Living; main page last edited 2025-09-09 (read 2026-09-30); CC0 | Pages checked to exist: `Timing`, `MMIO_registers`, `DMA_registers`, `PPU_registers`, `Sprites`, `Windows`, `Color_math`, `Mode_7`, `Offset-per-tile`, `Tilemaps`, `S-SMP`, `S-DSP_registers`, `DSP_envelopes`, `BRR_samples`, `SPC-700_instruction_set`, `Open_bus`, `Errata`, `Tricky-to-emulate_games`, `Memory_map`, `ROM_header`, `Controller_reading`, `Super_FX`, `DSP-1` |
| **anomie's documents** | romhacking.net documents 191 (S-DSP), 193 (memory map), 194 (open bus and wrapping), 195 (ports), 196 (registers), 197 (SPC700 with the boot ROM), 198 (SPC700 cycles), 199 (timing, 2008-12-21); list at <https://sneslab.net/wiki/List_of_Anomie's_Docs> | As posted | Register behaviour, cycle-level CPU and SPC700 timing, open bus |
| **WDC W65C816S datasheet** | <https://www.westerndesigncenter.com/wdc/documentation/w65c816s.pdf> | Current WDC edition | Instruction set, the cycle-by-cycle bus table with `VDA`/`VPA`, emulation mode, interrupts |
| **Programming the 65816** (Eyes and Lichty) | WDC's documentation page | 1986 text as WDC hosts it | Addressing-mode semantics and edge cases |
| **Decimal mode** (Bruce Clark) | <http://www.6502.org/tutorials/decimal_mode.html>, Appendix A | As posted | ADC/SBC in decimal mode, including invalid BCD; gilyon's CPU test cites it |
| **Test-ROM authors' notes** | the READMEs and source comments of §3's ROMs | Per ROM | undisbeliever's hardware findings, blargg's and anomie's test notes, absindx's SA-1 notes |
| **NEC µPD7725 / µPD96050 datasheets** | to be pinned in stage 5 | — | The DSP-n and ST01x instruction sets and timing; the firmware itself is the player's dump (§9, Q2) |

anomie's documents could not be fetched mechanically on 2026-09-30 (romhacking.net answered each download with a page
rather than the file). They are fetched by hand in stage 0, and their dates recorded then. *Stage 0, 2026-09-30:* six
of the eight were fetched from the Wayback Machine's captures of 2017-07-31: 191 (S-DSP, `$Revision: 1212`), 193
(memory mapping, 1160), 194 (open bus and wrapping, 1126), 196 (registers, 1157), 198 (SPC700 cycles, 1126) and 199
(timing, 1160), into `docs/anomie/` with their SHA-256 sums. 195 (ports) and 197 (SPC700 with the boot ROM) were not
obtained: every capture of them replays as a page, not the file (`VenusRT_Native.md` §6).

### 2.2 Where the public documents are thin

These are the places where the core will depend on test ROMs and disputes more than on text. Each is a prediction of
where the disputes log will fill (P6).

- **The SPC700's bus cycle by cycle, and the timers.** Instruction cycle counts are documented; the order of the
  reads and dummy reads inside an instruction, and the exact tick on which a timer's stage increments, are documented
  mostly by blargg's test ROMs (`spc_mem_access_times`, `spc_timer`) rather than by text.
- **The S-DSP's step schedule.** The 32-step sample schedule, the KON/KOFF latency and the echo buffer's timing were
  reverse-engineered by blargg and published mostly as code, which §1.2 excludes. `spc_dsp6` is the oracle; the RTL's
  DSP formulas are weak evidence (§1.5).
- **The PPU mid-scanline.** When a register write takes effect relative to the dot being drawn, which fetches are
  latched per tile or per line, and what the renderer sees during an HDMA write are covered by undisbeliever's tests
  and scattered wiki notes, not by a register description.
- **DMA and HDMA timing at the edges.** The documents give the per-byte cost and the HDMA start point; the overlap of a
  general DMA with HDMA start, and the S-CPU revision differences (`scpu-a-dma-bug-*`), are known from test ROMs.
- **The multiply and divide unit's intermediate values.** Reading `$4214`–`$4217` before the result is ready shows
  partial values; `mul_timing`/`div_timing` test it. Documented thinly, and the RTL copies an emulator here (§1.5).
- **Open bus.** anomie's document 194 and the wiki's `Open_bus` cover it; the MDR's behaviour around DMA is
  covered by `test_mdrhdma`.
- **SA-1 bus contention and its timers, the GSU's cache and pixel caches' timing, and the DSP-n's clocking.**
  fullsnes describes the registers; cycle cost and contention come from test ROMs (absindx's) and the referee.
- **The clock crystals.** The APU's ceramic resonator varies between consoles (~32.0–32.1 kHz sample rates are reported);
  a nominal value is a choice, not a fact (§5.3).

---

## 3. The oracles, by component

### 3.1 What exists, and how each reports

All of it was fetched on 2026-09-30 into `~/.cache/emusen/probe/venusrt/` and is never committed. Provenance, commits
and licences are in that folder's `PROVENANCE.txt`. 384 ROM files, 295 distinct by MD5.

| Oracle | Source, version | Licence | Covers | How it reports |
|---|---|---|---|---|
| **SingleStepTests 65816** | `github.com/SingleStepTests/65816` @`db6b104` (2024-09-21), `v1/`, 512 files × 10,000 | none stated | Every opcode in native and emulation mode: registers, memory and the bus cycle by cycle | JSON before/after state and a per-cycle `(address, value, signals)` list; the runner compares |
| **SingleStepTests SPC700** | `github.com/SingleStepTests/spc700` @`67d15f4` (2024-05-10), `v1/`, 256 × 1,000 | MIT | Every opcode, with cycles | Same shape |
| **gilyon's snes-tests** | `github.com/gilyon/snes-tests` release v1.4 (2025-06-10) | MIT | `cputest-full`/`-basic`: every 65C816 opcode but STP/WAI in every mode, emulation-mode wrapping, decimal mode (649 and 452 tests); `spctest`: every SPC700 opcode (557 tests) | Text in VRAM: word `$0032` reads `Success` or `Failed`, word `$006E` the test number in hex; `tests-*.txt` describes each test |
| **PeterLemon (krom)** | `github.com/PeterLemon/SNES` @`350b394` (2022-09-20) | none stated | `CPUTest/CPU` (23 ROMs by instruction group), `CPUTest/SPC700` (7), `CHIP/GSU/GSUTest` (31, in the higan collection); plus PPU, DMA and chip demos | `PASS`/`FAIL` written into the BG tilemap's low bytes; a failing test loops on its `FAIL`, so the final page shows it |
| **blargg's SPC tests** | higan collection `blargg-spc-6/` | unstated | `spc_smp`, `spc_timer`, `spc_mem_access_times`, `spc_dsp6` | Backdrop colour: blue passed, red failed (CGRAM entry 0), with a list of test names |
| **blargg 2010 tests** | higan collection `jonasquinn-test-roms/blargg_2010-03-14/` | unstated | SPC700 execution from I/O, IPL RAM disable, timer speed and stop, controller strobe | Text `Passed`/`Failed` in VRAM |
| **anomie/byuu era tests** | higan collection `jonasquinn-test-roms/` (82 ROMs) | unstated | ADC/SBC, multiply/divide behaviour and timing, IRQ/NMI, DMA and HDMA timing, VRAM/OAM timing, a Cx4 test | Text in VRAM; some print values only; ~~that is all~~ byuu's (`snestest_082506`, `blobs`, `nmi_irq`, `test_dmavalid`, `test_hdmadisable`, `test_mdrhdma`) grade themselves by the backdrop, colour 0 blue (`$7C00`) for a pass and red (`$001F`) for a failure, with the failing test's number in SRAM at `$700000` (corrected 2026-10-01: missed when the table was written; `VenusRT_Native.md` §16.2) |
| **Sour's tests** | higan collection `Sour/SnesTests` (mirror of `github.com/SourMesen/SnesTests`) | unstated | DMA/IRQ interaction, per-opcode timing | Values on screen, graded against a recording of a real console (the `.mp4` in the folder) |
| **absindx's SA-1 tests** | higan collection `absindx/` | per its `LICENSE` | SA-1 RAM protection, version code | Text `PASSED`/`FAILED` |
| **undisbeliever's tests** | `github.com/undisbeliever/snes-test-roms` @`ac6ef80` (2026-05-01), built here (108 ROMs); release v20210217 (29) | zlib (MIT for the INIDISP set) | Auto-joypad timing, HDMA/DMA glitches, INIDISP early-read glitch, VMAIN remapping, windows, mode 7 tilemaps, `wrmpyb-in-flight`, `vram-mid-scanline-test` | Mostly visual, with hardware photographs described in each source's header |
| **240p Test Suite** | SourceForge `OldFiles/SNES_SFC/`, 1.03 NTSC and PAL (2016); 1.10 is on itch.io only | GPLv2 | Picture: grids, colour bars, scroll, overscan, interlace, audio sync | Visual; graded by picture against Mesen |
| **higan's collection, the rest** | `gitlab.com/higan/snes-test-roms` @`26e8aa9` (2024-09-23) | per author | 93143's H-blank DMA to VRAM, lidnariq's PPU bus activity and SMP clock measurement, Motive's H-blank tests, tepples's `tellinglys`, VitorVilela7's speed test, KungFuFurby's IRQ/NMI tests | Mixed |
| **The Mesen probe** | §3.4 | — | Everything, frame by frame | Memory spaces, screen, traces, audio |
| **Commercial games** | the ROM library, copied to scratch | not redistributable | Integration; the goldens of §7 | Mesen differential at anchors |

**Nothing like AccuracyCoin exists for the SNES** (searched 2026-09-30, a negative result): no single ROM grades a
hundred-odd behaviours with one pass/fail table. The nearest are gilyon's exhaustive CPU tests and the higan
collection taken together. None is written during the rewrite (§9, Q6).

### 3.2 By component

| Component | Primary oracle | Secondary | Thin (§2.2) |
|---|---|---|---|
| 65816 | SingleStepTests 65816 with cycles; gilyon `cputest-full` | PeterLemon CPU; jonasquinn ADC/SBC | — |
| Bus, map, open bus | SingleStepTests' cycle lists (addresses); gilyon | jonasquinn `memtest`, `test_mdrhdma` | open bus around DMA |
| DMA, HDMA | undisbeliever's DMA/HDMA glitch and latch tests; jonasquinn `test_dma*`, `test_hdma*` | Sour's `dma_irq_test`; 93143 | HDMA/DMA overlap |
| IRQ, NMI, H/V timers | jonasquinn and KungFuFurby `test_irq*`, `test_nmi*` | Sour's `timing_test` | — |
| Multiply/divide | jonasquinn `muldiv_tests`, `snes_mul_div_timing` | undisbeliever `wrmpyb-in-flight` | intermediate values |
| Auto-joypad | undisbeliever `auto-joypad/` (15 ROMs) | — | — |
| PPU | Mesen picture at stable frames on the 240p suite and undisbeliever's effects | PeterLemon PPU; commercial goldens | mid-scanline |
| SPC700 | SingleStepTests SPC700 with cycles; gilyon `spctest`; blargg `spc_smp`, `spc_mem_access_times` | PeterLemon SPC700; blargg 2010 | bus order |
| Timers, ports, IPL | blargg `spc_timer`, 2010 `test_timer_*`, `test_ram_disable_ipl` | lidnariq `smpspeed` | — |
| S-DSP | blargg `spc_dsp6`; audio against Mesen | — | step schedule |
| SA-1 | absindx's two tests; Mesen on Super Mario RPG, Kirby Super Star | the RTL | contention |
| GSU | PeterLemon `GSUTest` (31); Mesen on Yoshi's Island, Star Fox | — | cache timing |
| NEC DSP-n, ST01x | Mesen on Pilotwings, Super Mario Kart, and the ST010/ST011 titles, with the player's firmware | the RTL (the only second opinion, `Venus_Referee.md` §0) | clocking |
| OBC1 | Mesen on Metal Combat | fullsnes | — |

### 3.3 The runners

Two runners are needed and neither exists in the repository yet (stage 0):

- **A WiseMan test-ROM runner** over `ICore` and `IDebugTarget`, generic over engines so that it grades C# Venus and
  VenusRT the same way. It reads the corpus from an environment variable and skips when the corpus is absent, as
  `NecDspRealFirmwareTests` skips without firmware (`EmuSen_Firmware.md` §6). Its grading rules are the ones of
  §3.1's last column, as `grade.py` implements them today (§3.6).
- **A crate test over SingleStepTests** that reads the JSON directly and checks the cycle lists, which the existing
  Pharaoh runner does not (its 65816 comparison is state only).

### 3.4 The Mesen probe: status (measured 2026-09-30)

- **It builds.** `bash build-probe.sh mesen ~/Projects/mesen-reference` from `EmuSen.WiseMan/Reference/` found the
  checkout already patched, built nothing in Mesen (`make: Nothing to be done for 'core'`) and relinked the Rust probe
  in 0.74 s, to `~/.cache/emusen/probe/mesen/probe`.
- **It runs one SNES frame.** `cd ~/Projects/mesen-reference && ~/.cache/emusen/probe/mesen/probe cputest-full.sfc
  <out> 1 1` exits 0, reports `backend mesen, system snes`, five spaces (VRAM 65,536, CGRAM 512, OAM 544, WRAM 131,072,
  APU RAM 65,536 bytes) and a screen of 512×478 `Bgr555` (a 256×239 picture occupies its start when the frame is not
  hi-res).
- **Defect: it runs at the console's speed, not flat out.** 600 frames took 10.11 s of wall time and 1.22 s of CPU;
  1,200 took 20.07 s and 2.44 s. The probe's C ABI sets Mesen's maximum-speed flag, so something else paces it. Not
  investigated further in this step. The workaround used here is parallelism: fourteen probes at once over the corpus.
  *Fixed at stage 0, 2026-09-30:* the pacing follows Mesen's configured emulation speed, a setting separate from the
  flag, and the probe now sets it to unlimited through the public settings API. The same 600 frames take 0.96 s,
  with every space and the screen byte-identical (`EmuSen_Debugging_Tools_Reference_v5.md` §3.59,
  `VenusRT_Native.md` §5).
- Usage is `EmuSen_Debugging_Tools_Reference_v5.md` §3.45–§3.54; positional arguments come before flags.

### 3.5 The old core's baseline (measured 2026-09-30)

C# Venus was run through `CoreFactory.Load` and `ICore` by a throwaway console app (`venusbench`, §3.6), with the
battery disabled, no input, and its memory read through the bundle's `IDebugTarget`. Every one of the 295 ROMs ran
to frame 3600 without an exception. Mesen was run through the probe to the same frames. The rules are §3.1's. **These
numbers describe the old core's accuracy for comparison; they are not expected values for VenusRT.**

**The single-step suites** (Pharaoh `--singlestep`, state only, no cycle checks):

| Suite | Venus |
|---|---|
| SPC700 v1 | 255,105 / 256,000 |
| 65816 v1, native mode | 2,539,998 / 2,560,000; `MVN` and `MVP` 0 of 10,000 each; `C0` and `C4` 9,999 |
| 65816 v1, emulation mode | 275,938 / 2,560,000; 238 of 256 files fail. The failures shown have the stack's high byte kept rather than forced (`s: got F638 want 138`); whether the core or the harness adapter is responsible was not examined. *Read again at stage 1 (2026-09-30), argued:* most likely the adapter. The suite's emulation-mode cases start with a stack high byte no 65816 can hold, and VenusRT's harness, before loading S as the chip holds it, failed with these identical values (`VenusRT_Native.md` §11.4). The numbers stand as measured |

**The self-grading ROMs:**

| Suite | ROMs | Venus passes | Mesen passes |
|---|---|---|---|
| gilyon `cputest-full` / `-basic` | 2 | 0 (fails at test `0024`, `adc ($EF,x)`; and at `02BD`) | 2 (649 and 452 tests) |
| gilyon `spctest` (release and collection copies) | 2 | 2 | 2 |
| PeterLemon CPU | 23 | 23 | 23 |
| PeterLemon SPC700 | 7 | 7 | 7 |
| PeterLemon GSU | 31 | 31 | 31 |
| blargg `spc_smp`, `spc_timer`, `spc_mem_access_times` | 3 | 0 (red) | 3 (blue) |
| blargg `spc_dsp6` | 1 | 0 (still on "Echo/basics" at frame 18,000) | 1 (blue by frame 10,800) |
| blargg 2010 SPC and timer tests that Mesen passes | 10 | 3 (`test_exec_from_io`, `test_ram_disable`, `test_timer_speed_2`) | 10 |
| ADC/SBC (jonasquinn, three sets) | 7 | 7 | 7 |
| Multiply/divide behaviour and timing | 5 | 0 | 5 |
| absindx SA-1 RAM protection | 1 | 0 | 1 |
| absindx SA-1 version code | 1 | 0 | 0 (it reads open bus where the test expects values) |
| Cx4 memory test | 1 | not supported | pass |

Of the 93 self-grading ROMs Mesen passes, Venus passes 73. *Corrected 2026-10-01:* the count missed 25 ROMs of the
jonasquinn collection that grade themselves by the backdrop (§3.1's row). Read by that protocol, Mesen passes all 25
and Venus 8, failing 13 (IRQ, NMI, HDMA and DMA tests) and leaving 4 on another colour; so Mesen passes 117 of the
corpus's self-grading ROMs as the runner counts them, and Venus 81 of those 117 (`VenusRT_Native.md` §16.2). The
sentences below keep the count of the 93. Of the 20 it fails, 9 are about timing (the SPC700's
timers and memory access times, the multiply/divide unit's timing), one is a chip Venus does not support, and the rest
are instruction, IPL, DSP, multiply/divide and SA-1 behaviour. Excluded as ungradable here: `timer_at_power_reset`
(it asks for the reset button), `test_speed` and `test_timer_speed3` (they print values and "Done"), `speed_2_freezes2`
and one `mul_timing` (both engines report failure, which may be the expected output).

**The rest, 196 ROMs that grade by picture or by values on screen** (171 once the 25 backdrop-graded ROMs are counted as self-grading, 2026-10-01). Mesen's VRAM was identical at frames 1800 and
3600 on 172 of them. On 157 of those 172, Venus's VRAM at frame 3600 is byte-identical to Mesen's; the 15 that differ
include the H-blank DMA to VRAM tests, Sour's timing and DMA/IRQ tests, `vram-mid-scanline-test`, `wrmpyb-in-flight`,
`blip-autojoy-timing-test` and `test_dma`. VRAM agreement is weak evidence (a test that prints nothing agrees
trivially), and the list is where VenusRT's first picture disputes are expected.

### 3.6 Where the results are

`~/.cache/emusen/probe/venusrt/`: `PROVENANCE.txt`; `unique-roms.txt` (MD5, size, path); `venusbench/` (the runner,
built against this branch's `EmuSen.csproj`); `run-all.sh venus|mesen`; `grade.py`; `summarise.py`, which writes
`baseline.tsv`; `topng.py`, which draws the two engines' frames side by side; `bench.sh`; `logs/`; and `out/<engine>/<md5>/`.

---

## 4. What parity with C# Venus means

Read from its public surface only (§1.2).

### 4.1 `ICore` and the declared interfaces

- **`VenusCore : ICore, IFrameProfiler, ICoprocessorHalt, ICoprocessorLoad, ITraceFlushable, IStateFormat`.** It
  implements neither `ICoreSettings` nor `ISnapshotCore`, and has no `IFrameBufferPool`.
- `CoreName` is `"SNES"`. `ScreenWidth` is 256, or 512 on a frame with hi-res; `ScreenHeight` is always 224 (the probe's Mesen
  picture is 239 lines; overscan is outside parity, §9 Q8). `FrameRateHz` follows the header's region, NTSC or PAL.
- **Input:** two pads, twelve buttons in `PadButton` order B, Y, Select, Start, Up, Down, Left, Right, A, X, L, R; no
  axes. No mouse, Super Scope or multitap.
- **Sound:** `AudioSampleRate` follows the player's `AudioSettings.SampleRate`. 532.5 stereo frames a frame were
  drained at the default, i.e. 32 kHz.
- **`SkipRendering`**, honoured. *Measured:* with it set, Venus's state after 4,200 frames differs from the rendered
  run's on four of the seven games benched (Donkey Kong Country, A Link to the Past, Super Mario RPG, Pilotwings) and
  is identical on the other three. So in Venus the picture pass feeds the machine on some games; §5.1 makes VenusRT's
  skip state-neutral by design.
- **Halts:** `IsHaltedAtBreakpoint`, `HaltedAddress`; `ICoprocessorHalt.HaltedProcessorName` is `S-CPU` or `SA-1`,
  and a halt may land on the SA-1, the GSU or the DSP.
- **`ICoprocessorLoad`** reports the SA-1's executed and offered master clocks against a frame's.
- **`IFrameProfiler`** names seven phases: `cpu+spc700`, `ppu`, `hdma`, `ppu/obj-eval`, `ppu/blend`,
  `ppu/main-composite`, `ppu/sub-composite`. VenusRT's phases will be its own; parity is that the interface exists.
- **`IStateFormat`:** magic `0x53454E53` ("SNES"), version 3.

### 4.2 Cartridges, coprocessors and firmware

- **Boards** (by the mapper names it reports): LoROM, HiROM, SA-1, SuperFX, and a coprocessor overlay for the NEC DSPs
  and OBC1. Copier headers are stripped.
- **Coprocessors:** SA-1; the GSU (SuperFX 1 and 2); the NEC DSPs DSP-1, DSP-1B, DSP-2, DSP-3, DSP-4, ST010 and
  ST011; OBC1. **Not supported, and so not parity:** Cx4, S-DD1, SPC7110, ST018, S-RTC, BS-X, Sufami Turbo, MSU-1.
- **Firmware** is asked for before loading through `ICore.GetFirmwareRequirements`, from the header alone, and loaded
  from `home/Firmware/` through `FirmwareLibrary` (`EmuSen_Firmware.md`). The names are `dsp1.rom`, `dsp1b.rom`,
  `dsp2.rom`, `dsp3.rom`, `dsp4.rom` (8,192 bytes: program 6,144, data 2,048), `st010.rom` and `st011.rom` (53,248:
  49,152 and 4,096); each may also be a split `<name>.program.rom` + `<name>.data.rom` pair. A ROM that carries its
  firmware appended asks for nothing. A missing image is never fatal: the game loads without the chip.

### 4.3 Battery saves

- `Saves/SNES/<rom name>.srm` (`SaveLibrary.SramPathFor`), raw bytes; a save at the pre-2026-09-28 flat path is read
  once and copied into the console folder. Written through `ICore.SaveSram`; `CoreOptions.BatteryRamDisabled`
  (`--nobattery`) disables reading and writing.
- On a SuperFX cartridge the file is the GSU's Game Pak RAM; on an ST010/ST011 it is the DSP's data RAM.
- **These carry over.** They are raw SRAM, so VenusRT reads and writes the same files with the same length and
  meaning. A test pins it for each kind (§7, G4).

### 4.4 Cheats

`CoreFactory` gives the SNES two codecs: Pro Action Replay (auto-detect) and Game Genie (explicit). Game Genie patches
ROM reads by CPU address through `IRomReadPatcher`; the pokes are applied once a frame through the debug target.
VenusRT takes both through the common interface's `set_rom_patches` triples and `space_write`
(`EmuSen_NativeCores.md` §3.12). The codec classes are rewritten, not kept, in the SNES's system pack rather than in a
shim, so that any SNES engine has them (§9, Q5; `EmuSen_CoreAPI.md` §15 Q14).

### 4.5 Save states and rewind

**VenusRT has its own state format. Save states made by C# Venus will not load in VenusRT, and VenusRT's will not
load in Venus.** That includes the `.resume.state` files Mistress keeps beside library ROMs. VenusRT refuses a Venus
state with a message naming the engine that made it (the magic says so), and Mistress's resume falls back to a cold
boot with a notice. Rewind captures through `SaveState(Stream)`, as for Venus; `SNAPSHOT` is not needed.

### 4.6 Settings

The SNES has no rows in `CoreCatalog.SettingsFor` today. VenusRT adds the Engine row (`EngineByConsole["SNES"]`),
*Venus (C#)* first and the default until the gate of §7, and takes the output sample rate as a create-time and
run-time key (`EmuSen_NativeCores.md` §3.7 anticipated exactly this).

### 4.7 The debugger

`SnesDebugTarget` is built over Venus's internals, so it cannot serve VenusRT, and a mirror (MoonRT's and MercuryRT's
method) is impossible because no C# core can load VenusRT's state. VenusRT needs a new debug target whose data come
from Rust exports. Its surface, to match:

- **Spaces:** `CpuBus`, `IO`, `WRAM`, `VRAM`, `CGRAM`, `OAM`, `SRAM`, `APURAM`; with the chip, `GSURAM`, `GSUBUS`,
  `SA1IRAM`, `BWRAM`, `SA1BUS`, `DSPRAM`, `DSPPRG`.
- **Registers:** the 65816's A, X, Y, S, D, PB, PC, DB, P, E; video, APU and coprocessor register providers, with
  history for the coprocessor.
- **Views:** sprites, palettes, audio channels, hardware load, the interrupt vectors, DMA channels and the DMA log.
- **Disassembly** of 65816, SPC700, GSU and NEC DSP code, with static-reference classification.
- **Registries:** watches, frame log, cheats, breakpoints and coprocessor breakpoints, coverage and coprocessor
  coverage, call stack, labels, access counters, freezes, expressions, register flow.

### 4.8 The frontends

- **Mistress:** the Engine row; resume states (§4.5); rewind and fast-forward; the firmware prompt; the controller
  diagram (`ControllerLayout.Snes`); pacing from `FrameRateHz`; the fit audit gains the SNES tab's Engine row; the
  library's copier-header rule for OpenVGDB is catalogue data and unchanged.
- **Pharaoh and Hotaru** name Venus's types: Pharaoh's frame runner, command scripts, the trace differ against the
  probe's CPU and GSU traces, and `--singlestep`'s `65816` and `spc700` targets; Hotaru's backdrop and OAM dumps. Each
  is either given a VenusRT equivalent or retired with Venus (stage 9 decides each, and says which).

---

## 5. Design decisions

Each decision states the trade-off, the recommendation, and what the recommendation does not cover.

### 5.1 PPU granularity

- **Per scanline** is cheapest and wrong for any game that writes a PPU register during active display outside HDMA,
  and for HDMA writes whose effect depends on the dot at which they land.
- **Per dot** (a pipeline stepped every four master clocks through fetch, evaluation and composition) is the most
  faithful and the costliest: roughly 89,000 dot steps a frame, each doing layer fetch and priority work.
- **Recommended: a scanline renderer in spans.** Every PPU register write is stamped with the dot at which it takes
  effect (the latency is a documented or disputed rule per register). Before a stamped write is applied, the renderer
  draws the current line up to that dot; at the line's end it draws the rest. A line with no mid-line write is one span
  and costs what a scanline renderer costs. Writes during H-blank, where HDMA's land, cost nothing extra.
- **Evaluation is separate from drawing.** Sprite range and time evaluation (the `$213E` flags), the OAM address and
  priority rotation, and every other bit of machine state the PPU produces run whether or not the picture is skipped.
  `SkipRendering` then skips only the pixel writes, and a test compares the state of a skipped and a drawn run every
  frame (Venus fails that comparison on four of seven games, §4.1).
- **What spans do not cover:** effects below the granularity of "a register changes at dot *x*": the INIDISP early-read
  glitch (undisbeliever's `inidisp_*`), VRAM reads and writes during active display (`vram-mid-scanline-test`), and
  per-dot fetch-pipeline artefacts (lidnariq's `ppubusact`). These are accepted as known losses at first and listed
  in §7's gate as named exceptions, and none is modelled before the default flips (§9, Q10).

### 5.2 CPU and bus timing

- **Per instruction** timing (add an instruction's cycles, then run the rest of the machine) cannot place an IRQ,
  a DMA start or a register read inside an instruction, which Sour's `timing_test`, the IRQ tests and the auto-joypad
  tests all observe.
- **Recommended: every bus access advances the master clock.** The 65816 performs its accesses in the datasheet's
  cycle order; each access costs its region's speed (6, 8 or 12 master clocks; internal cycles 6), and the clock
  advances by that much before the next. A scheduler holds the fixed events of a line (the DRAM refresh, HDMA
  initialisation and transfer points, the H/V IRQ comparators, auto-joypad reads, the NMI line) as master-clock times.
  General DMA and HDMA are machine states that suspend the CPU at the next access boundary, with the documented
  alignment costs, so the CPU resumes mid-instruction exactly where it stopped.
- **The rest of the machine catches up lazily.** The PPU is brought up to the current dot when a PPU register is
  touched and at the end of each line; the APU when a `$2140`–`$2143` port is touched and at the end of the frame; each
  coprocessor when its registers are touched (§5.4). Catching up is exact where the only coupling between two parts is
  the access that triggers it (argued); the tests of §3.2 check it.
- **The cycle lists of SingleStepTests 65816 are the oracle for the order of accesses,** including the dummy reads,
  which Pharaoh's state-only runner does not check today.
- **Cost:** one call through the bus per access, a clock add and an event check. At about 2.7–3.6 million CPU
  accesses a second (argued from 21.47 MHz over 6 or 8 clocks), this is the core's hottest path and it is the one that
  must stay monomorphic and inlined.

### 5.3 The SPC700 and the S-DSP, a clock domain of their own

- The APU runs from its own resonator, nominally 24.576 MHz: the SPC700 at 1.024 MHz and a sample every 32 of its
  cycles (32 kHz). The ratio to the master clock is carried as an exact rational, so neither side drifts.
- **Recommended:** the SPC700 steps by bus cycle, as the 65816 does, and the DSP advances one of its 32 steps per
  SPC700 cycle, so that a DSP register write lands on the step at which it is documented (or disputed) to land; the
  timers tick on their documented divisions of the same clock. Cost is small (argued): 1.024 million SPC700 cycles and
  as many DSP steps a second, against the CPU's accesses.
- **Synchronisation** is catch-up at a port access from either side and at the frame end, as §5.2 describes. The ports
  are the only coupling (argued from the documents), so this is exact without lock-step.
- **The resonator's variance** is a choice: nominal by default. A setting for a measured console's rate would affect
  only games that race the two domains; none is known to matter, and it is left out of parity.
- **Output:** the DSP's 32 kHz stereo into a `SampleQueue`, and a resampler to the player's rate in the core if the
  setting asks for another rate (§4.1); the resampler is a design choice with no oracle beyond "32 kHz in, the same
  sound out", and it is tested for exactness at 32 kHz, where it is the identity.

### 5.4 Coprocessor clocking

- **SA-1:** a second 65816 at 10.74 MHz sharing ROM and BW-RAM with the S-CPU, with contention. **Recommended:** the
  same CPU implementation over an SA-1 bus, run in bounded slices interleaved with the S-CPU (a slice ends at the next
  S-CPU access to a shared region or register, or after a fixed number of master clocks). The slice bound trades
  accuracy of contention against speed and is a measured parameter, not a constant chosen in advance (P5).
- **GSU:** runs at 10.74 or 21.47 MHz while it owns the Game Pak bus, and the S-CPU is locked out of ROM and RAM while
  it does. **Recommended:** run to its stop, or to the S-CPU's next access to its registers or to ROM/RAM it owns,
  whichever is first; the caches and the pixel buffer are modelled with their documented timing.
- **NEC DSP-n and ST01x:** slaves polled through a status register, on crystals of their own. **Recommended:** catch up
  on each access, with the chip's clock carried as a rational against the master clock like the APU's.
- **OBC1:** no clock; a register view of SRAM.

### 5.5 The speed budget

**What C# Venus costs today** (measured 2026-09-30 through `ICore`: `venusbench bench`, 1,200 frames of boot with Start
and A tapped, then 3,000 timed frames with Right held; means of two interleaved rounds under the bench lock; scenes not
verified by screenshot):

| Game (chip) | Desktop, ms/frame (p99) | Desktop, picture skipped | 33% CPU quota, ms/frame (p99) |
|---|---|---|---|
| Super Mario World | 2.07 (2.90) | 0.67 | 6.88 (14.04) |
| Super Metroid | 2.18 (3.15) | 0.79 | 7.55 (14.34) |
| Donkey Kong Country | 2.08 (2.94) | 0.59 | 7.26 (14.16) |
| A Link to the Past | 2.10 (4.36) | 0.74 | 7.42 (21.60) |
| Yoshi's Island (GSU-2) | 3.13 (3.81) | 1.16 | 10.99 (17.43) |
| Super Mario RPG (SA-1) | 4.16 (4.83) | 2.14 | 14.36 (19.85) |
| Pilotwings (DSP-1) | 2.60 (3.31) | 1.33 | 8.58 (14.55) |

Star Fox and Super Mario Kart were benched too (0.84 and 1.03 ms) and are left out: their PPU phases near zero say the
script did not reach 3D play. The quota runs use `systemd-run --scope -p CPUQuota=33%`, the weak-laptop proxy of the
earlier optimisation work (a third of the desktop's single thread). The Legion Go S could not be reached on 2026-09-30;
Mars's measurements there (`Mars_Native.md` §6.1) put its single thread within about 1.3–1.4× of the desktop's
(argued from their MarsRT figures), so **the weak laptop, not the handheld, is the binding case**.

**The budget.** A frame is 16.64 ms (NTSC) or 20.0 ms (PAL). *Decided 2026-09-30:* VenusRT must run every bench game
at full speed at 33% quota with a fifth of the frame left for the frontend: a mean of at most 13.3 ms and a p99 of at
most 16.6 ms there, which is a desktop mean of about 3.8 ms (the measured desktop-to-quota factor is 3.3–3.5×).
Venus meets that mean everywhere but on Super Mario RPG (14.36 ms), and misses the p99 on Super Mario RPG, Yoshi's
Island and A Link to the Past.

**What the design adds, priced (argued):**

| Part | Venus today (desktop, from its phases) | VenusRT budget, plain / SA-1 or GSU cart | What drives it |
|---|---|---|---|
| CPU, bus, DMA | together with the APU, `cpu+spc700` is 0.5–0.7 on a plain cartridge | 0.8 / 1.0 | Per-access clocking (§5.2) |
| SPC700 and DSP | (inside `cpu+spc700`) | 0.5 / 0.5 | Per-cycle DSP steps (§5.3) |
| PPU | `ppu` is 1.3–2.0 | 1.5 / 1.4 | Spans; evaluation always on (§5.1) |
| Coprocessor | `cpu+spc700` grows to 1.1 on Yoshi's Island and 2.1 on Super Mario RPG | — / 0.9 | Slice interleave (§5.4) |
| **Total** | 2.1 / 3.1–4.2 | **≤ 2.8 / ≤ 3.8** | |

MoonRT measured Rust at 1.28–1.40× the speed of line-for-line C# (`Moon_Native.md` §8.3.3), which is what makes a
more exact design affordable within the same budget. No tuning is done while the rewrite runs (*decided 2026-09-30*: the
optimisation work is on hold); the budget is checked at each stage's end with `bench.sh`, and a stage that breaks it
records the overrun rather than tuning (P1, P2).

### 5.6 The state format

Its own, written with `emusen-native`'s `StateWriter`/`StateReader`: a magic distinct from Venus's (`"VNRT"`), a
version, and the machine's fields in a fixed order, with each chip's block present only for a cartridge that has it.
The layout listing (`state_layout`) is kept and a test pins it, because without a C# oracle the listing is the only
record of what a version means. Rewind's size matters: Venus's states are 597–728 KB (measured), mostly memory; VenusRT's
will be similar (argued: WRAM 128 KB, VRAM 64 KB, APU RAM 64 KB and the cartridge RAMs dominate).

---

## 6. Stages

Each stage is a run of supervised steps of a few hours, with a check-in after each, as MoonRT's port ran from
2026-09-28. The effort is in steps; a step is about three hours. The estimates are predictions (P3).

| Stage | What it covers | Its oracle | Steps |
|---|---|---|---|
| 0 | The WiseMan test-ROM runner generic over engines (§3.3), with §3.5's rules and baseline; the crate test over SingleStepTests with cycle lists; anomie's documents fetched and pinned; `VenusRT_Disputes.md` created; the probe's pacing defect noted in the tools reference | The runner reproduces §3.5's Venus and Mesen columns | 2 |
| 1 | The crate on the common interface from its first commit (`NativeCore`, `native_exports!`), with a stub machine; the 65816 over a flat test bus | SingleStepTests 65816, native and emulation, state and cycles; gilyon `cputest-full` on a minimal bus with the VRAM port only | 4 |
| 2 | The S-CPU's system side: the memory map and cartridge boards, access speeds, WRAM and its ports, open bus, the scheduler and its line events, DMA and HDMA, NMI, H/V IRQ, the multiply/divide unit, auto-joypad, the region | gilyon `cputest-full` on the real bus; jonasquinn IRQ/NMI/DMA/HDMA/muldiv; undisbeliever DMA/HDMA/auto-joypad; Sour's tests | 5 |
| 3 | The PPU: VRAM/CGRAM/OAM ports and their timing, backgrounds in modes 0–7, sprites and their evaluation, windows, colour math, mosaic, offset-per-tile, direct colour, hi-res and pseudo-hi-res, interlace, mode 7; spans (§5.1) | Mesen picture at stable frames: the 240p suite, undisbeliever's effects and VMAIN tests, PeterLemon PPU; the skip-versus-draw state test | 8 |
| 4 | The SPC700, IPL, timers and ports; the S-DSP, BRR, envelopes, echo, noise, pitch modulation; the 32 kHz queue and the output rate | SingleStepTests SPC700 with cycles; gilyon `spctest`; blargg `spc_smp`, `spc_timer`, `spc_mem_access_times`, `spc_dsp6`; the 2010 tests; audio against Mesen | 5 |
| 5 | Coprocessors: NEC DSP-n and ST01x with the player's firmware; SA-1 (the stage 1 CPU on its own bus, BW-RAM, I-RAM, DMA, character conversion, arithmetic, timers); GSU-1/2 with caches and plotting; OBC1 | absindx's SA-1 tests; PeterLemon `GSUTest`; Mesen on the chip games | 9 |
| 6 | Frontends on the core ABI v1, with no per-core C# class (revised 2026-10-03 by `EmuSen_CoreAPI.md` §13.2): VenusRT runs under the v1 adapter, the one generic engine class over any `emusen_core` library, and presents itself from its own descriptors: core info with the SNES's controller, machine info with the spaces, the battery files, the state format and the patch limit, the settings schema, and `status_text` for its band. The Engine row and `CoreFactory` come from discovery, with no code naming VenusRT; cheats from the SNES's system pack (§9, Q5); battery saves; rewind on the generated snapshot; firmware requests from the header through `firmware_for` (new code: VenusRT's own lookup of which NEC DSP image a cartridge needs, from its header and title, which also replaces the WiseMan harness's interim call to C# Venus's public `Cartridge.FirmwareRequirements` before parity, since that call cannot outlive Venus; added 2026-10-02, `VenusRT_Native.md` §24.1); state refusals as the core's own statuses with `last_error`, and no exception mapping, a clean-room core having no C# exceptions to reproduce; the crash log | Commercial ROMs through `ICore` on the adapter; `.srm` round trips from C# Venus for every chip; the Engine row and fallback tests; the conformance kit's core suite (C1–C8, C10–C11); the fit audit | 3 |
| 7 | The debugger: the shared hooks with processors 0 (S-CPU), 1 (SPC700) and 2 (the cartridge's processor); a new `VenusRtDebugTarget` over Rust exports; new disassemblers written from the datasheets | The debug-target claims of §4.7 as tests on VenusRT; every table armed on three games identical to plain runs | 4 |
| 8 | Parity gates: §7's list run in full, the goldens recorded, the clone check | §7 | 3 |
| 9 | Retirement: the default flipped; after the waiting period, Venus removed or moved; Pharaoh's and Hotaru's Venus uses given equivalents or retired | §7 | 1 |

*Stage 0 was run on 2026-09-30, with the crate skeleton, the probe's pacing fix and the clone check brought forward
into it; `VenusRT_Native.md` §1 is its record. Stage 1 took two steps, not four, the same day (§10 and §11 there), and stage 2 three, not five (§12 to §14). Stage 7 closed on 2026-10-04 in four steps
(§36): on the generic debug target, which `EmuSen_CoreAPI.md` §13.2 put in place of a `VenusRtDebugTarget`; §4.7's
console views stay outside v1 there. Stage 8 was run on 2026-10-04 in one step (§41 there): one gate met, two met with named
exceptions, five not met, so the default does not flip; its §41.10 lists what blocks it.* *Stage 9's first half, the default
flipped, was done on 2026-10-05, decided by the tester that day ("You may make venus the default"), with G6's run on
the handheld on battery still owed and recorded as owed (`VenusRT_Native.md` §65). C# Venus stays a choice and the
fallback. It is removed or moved only after §7's four weeks of play as the default with no open regression against
it.*

About 44 steps, or 130–160 hours (P3). Stage 6 can run after stage 4, before the coprocessors, so that the player gets
the engine for ordinary cartridges early; games with a chip then fall back to Venus with a notice until stage 5 lands.
That order is recommended, and the table keeps the order in which the stages depend on each other.

---

## 7. The retirement gate

Two gates: one to make VenusRT Mistress's SNES default, and one to remove C# Venus.

**To become the default, every one of these holds:**

- **G1, the test ROMs.** VenusRT passes every self-grading ROM of §3.5 that Mesen passes, except those named in the
  disputes log with a ruling against Mesen; and it passes every ROM that C# Venus passes, with no exception.
- **G2, the single-step suites.** 100% of SingleStepTests 65816 (both modes) and SPC700, state and cycle lists,
  except cases named in the disputes log (for example, SPC700 cases that address the I/O page, if the suite's flat
  memory is ruled to disagree with the hardware there).
- **G3, the pictures.** On the 172 picture ROMs that are stable on Mesen, VRAM and the picture equal Mesen's, or the
  difference is a disputes-log entry or one of §5.1's named losses.
- **G4, the frontend.** Every item of §4 is present or its loss written down; `.srm` files from C# Venus load and are
  rewritten unchanged for a plain SRAM cartridge, SA-1, SuperFX and ST010/ST011; Venus states are refused with the
  message of §4.5; the fit audit passes.
- **G5, the goldens.** Twelve commercial games (at least one per coprocessor, plus the tricky-to-emulate list's first
  entries), each at three anchors, equal to Mesen in picture, or dispute-logged.
- **G6, speed.** §5.5's budget at 33% quota on the bench set, and full speed on the Legion Go S on battery.
- **G7, the debugger.** Breakpoints, stepping, watches, coverage, the call stack and coprocessor breakpoints work on
  VenusRT, each claim a test.
- **G8, the clean room.** A clone-detection pass of VenusRT's sources against Mesen's SNES sources (the audit owed for
  every core) finds nothing above its threshold, and the disputes log has been read end to end by someone who did not
  write the entries.

**To remove C# Venus:** VenusRT has been the default for four weeks of play with no open regression against Venus
(each report either fixed or shown to be Venus's behaviour, not hardware's). It then moves to a legacy branch (§9, Q9).

---

## 8. Reuse, and what the framework should gain

**Taken from `emusen-native` as it is:**

- `abi`: the `NativeCore` trait and `native_exports!`, the fixed names, the version and capabilities. VenusRT claims
  `RESET`, `MUTES`, `ROM_PATCHES`, `SETTINGS`, `PHASES`, and from stage 7 `DEBUG` and `DEBUG_STACK`.
- `StateWriter`/`StateReader` and `State`, for VenusRT's own format.
- `SampleQueue`, with its drop-oldest limit.
- `debug::Hooks`, with coverage widths of 24 bits (S-CPU), 16 (SPC700) and the cartridge processor's (24 for the SA-1;
  the GSU's and DSP's own).

**From the C# host:** `NativeCoreLibrary`, `NativeMachine`, `NativeRtCore` and `NativeDebugBridge`, with the SNES's
console part as §4.5 of `EmuSen_NativeCores.md` lays out for the other three.

**What the framework should gain, found by planning this core:**

- **Create-time firmware files.** `NativeFile.which` ≥ 1 is the console's; the SNES's firmware is its first real use,
  including the split program/data pair.
- **A state with no C# oracle.** The naming rule (`naming.rs`) ties Rust fields to C# names; VenusRT has none. A
  layout-stability test takes its place, and `EmuSen_RustState.md` should say which rules are for ports only.
- **The output rate as a setting key** (`EmuSen_NativeCores.md` §3.7).
- **Halts on processors other than the first.** `ICoprocessorHalt` needs the processor that stopped; `debug_pc` takes a
  processor already, and the run's stop should report which.
- **The registration record** (`EmuSen_NativeCores.md` §5.1), still owed: VenusRT is the natural first engine to
  register through it, since its Engine row is new.
- **A test-ROM runner generic over engines** (§3.3), which the other cores could share.

---

## 9. Questions, decided 2026-09-30

Each question as it was put, and its decision. Every recommendation this page made was accepted as written.

1. **Q1, may the SNES Development Manual be used as a source?** It is Nintendo's, circulated without permission, the
   only first-party register description, and parts of the public documents already rest on it. Decided: it is not a
   source for writers. Public documents that cite it are used as any document is (§1.2, §2).
2. **Q2, how is coprocessor firmware obtained?** Decided: as today, from the player's own dumps, found or picked
   through `FirmwareLibrary` (`EmuSen_Firmware.md` §5 is why nothing ships). There is no high-level emulation of the
   DSPs, and VenusRT reads the same files, combined or split (§4.2). *Superseded 2026-10-04* by `EmuSen_Firmware.md`
   §0: VenusRT carries open replacements for the NEC DSP programs, planned in `VenusRT_DspHle.md`, whose first step is
   recorded in `VenusRT_Native.md` §37; the player's own image stays the exact path when present.
3. **Q3, how much accuracy is traded for speed on the handheld and the weak laptop?** Decided: none by default, and
   no second "fast" core. If §5.5's budget is missed, the first lever is the SA-1 slice bound (§5.4), which changes only
   contention timing, and it becomes a setting before anything in the PPU or CPU is loosened.
4. **Q4, are other emulators' sources excluded as Mesen's is?** Decided: yes, all of them. The MiSTer RTL is the only
   implementation read, and only in dispute steps (§1.2, §1.5).
5. **Q5, can Venus's cheat codecs be kept?** They encode two public cheat formats and sit in Venus's folder. Decided:
   they are rewritten, about 140 lines, because provenance is judged by folder under this protocol and the cost is
   small (§4.4). The rewrite was first placed in VenusRT's shim. Decided 2026-10-03 (`EmuSen_CoreAPI.md` §15 Q14): it
   goes in the SNES's system pack in DianaOS instead, keyed by system id, so that any SNES engine has the codecs and
   VenusRT needs no shim for them.
6. **Q6, should the project write an SNES accuracy ROM of its own?** Decided: not during the rewrite. Disputes produce
   small test ROMs anyway (§1.4), and collecting them into one suite afterwards is the cheaper route (§3.1).
7. **Q7, what happens to Venus's `.resume.state` files in the library?** They cannot load (§4.5). Decided: a cold boot
   with a one-line notice, and no converter.
8. **Q8, does the scope grow beyond parity?** Cx4, S-DD1, SPC7110, ST018, S-RTC, BS-X, Sufami Turbo, MSU-1, overscan,
   the mouse, the Super Scope and the multitap. Decided: only after retirement, one at a time, each with its oracle.
9. **Q9, is C# Venus deleted or moved to a legacy branch?** Decided: moved to a legacy branch, as was decided for the
   other C# cores, so that §3.5's baseline stays reproducible (§7).
10. **Q10, which of §5.1's named losses must be modelled?** Decided: none before the default flips, unless a game in
    the goldens needs one. Each is then its own stage, with its undisbeliever ROM as the oracle (§5.1, §7's G3).

---

## 10. Negative results

- No SNES counterpart of AccuracyCoin was found (§3.1).
- anomie's documents could not be downloaded mechanically (§2.1).
- ~~The Mesen probe does not run flat out (§3.4).~~ Fixed at stage 0 (§3.4).
- The Legion Go S was unreachable, so the handheld column of §5.5 is argued, not measured.
- Two benched games did not reach gameplay, and no bench scene was verified by screenshot.

## 11. Predictions to be retired

- **P1.** VenusRT's desktop mean on the five plain benched games is at most 2.8 ms a frame at the end of stage 4,
  with spans, per-access clocking and per-cycle DSP steps all on.
- **P2.** On Super Mario RPG and Yoshi's Island VenusRT's desktop mean is at most 3.8 ms at the end of stage 5.
- **P3.** The whole is about 44 steps; the PPU and the coprocessors are the stages most likely to overrun.
- **P4.** Stage 2 passes gilyon `cputest-full` and every jonasquinn IRQ/NMI test before any PPU code exists.
- **P5.** An SA-1 slice bound exists that passes absindx's RAM-protection test and keeps Super Mario RPG within P2.
- **P6.** At least half the disputes-log entries fall in the thin areas of §2.2, and fewer than one in five need
  Mesen's source.
- **P7.** The skip-versus-draw state test (§5.1) passes on every bench game from stage 3 on.
