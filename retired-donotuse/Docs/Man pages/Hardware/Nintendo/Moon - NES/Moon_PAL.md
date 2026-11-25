# Moon (NES) — the PAL NES and the Dendy

*MoonRT, 2026-10-17. Decided 2026-10-17 by the tester: MoonRT gains a PAL timing mode as work of its own, after which it
reports `"pal"` through machine info's existing `region` field (`EmuSen_CRT.md` §27.6). This page records that work. It
describes MoonRT, not the retired C# core, and is current.*

Covers `src/model.rs` (the one owner of every fact in which the consoles differ), the parts of the CPU, PPU, APU, bus
and machine that read it, the region setting in `src/ffi/v1.rs`, and `src/pal_tests.rs`.

Sources, cited by the short names used below. **[nesdev]** pages read on 2026-10-17: *Cycle reference chart*, *PPU
rendering*, *PPU OAM*, *PAL video*, *APU Frame Counter*, *APU Noise*, *APU DMC*, *iNES* (bytes 9 and 10), *NES 2.0*
(byte 12). **[blargg]** Shay Green's
`pal_apu_tests`, "tested on a PAL NES and all give a passing result" (its readme), from the test-ROM clone of
`Moon_TestRoms.md` §3.1. **[MiSTer]** NES_MiSTer at `~/Projects/nes-mister-reference` (`rtl/nes.v`, `rtl/ppu.sv`,
`rtl/apu.sv`), a referee under `Moon_Native.md` §3.7's reading rule: its PPU and APU are its own and count as evidence,
its T65 CPU is borrowed and does not, and its PAL frame counter is marked "Speculative" by its own authors.
**[Mesen]** the Mesen checkout, read only where the two sources above leave a placement open, and named where it was.

---

## 1. What differs, and where each fact comes from

| Fact | NTSC NES (2A03, 2C02) | PAL NES (2A07, 2C07) | Dendy (6527P, 6538) | Source |
|---|---|---|---|---|
| Master clock | 21,477,272 Hz | 26,601,712.5 Hz | as PAL | [nesdev] *Cycle reference chart*; PAL's is six times 4.43361875 MHz |
| Master clocks a CPU cycle | 12 | 16 | 15 | [nesdev] |
| Master clocks a dot | 4 | 5 | 5 | [nesdev] |
| Dots a CPU cycle | 3 | 3.2 | 3 | [nesdev] |
| Lines a frame | 262 | 312 | 312 | [nesdev] *PPU rendering*, *PAL video* |
| Vblank flag and NMI | line 241 | line 241 (70 lines of vblank) | line 291 (51 post-render lines) | [nesdev]; [MiSTer] `ppu.sv:215–236` |
| Odd frame's dropped dot | yes, while rendering | no | no | [nesdev] gives it only for NTSC; [MiSTer] `skip_en` |
| CPU cycles a frame | 29,780⅔ | 33,247.5 | 35,464 | [nesdev] |
| Frame rate | 60.0985 Hz | 50.0070 Hz | 50.0070 Hz | [nesdev] |
| Black border | no | top line, two pixels each side | as PAL | [nesdev] *PPU rendering*, *PAL video*; [MiSTer] `ppu.sv:1856` |
| Emphasis bits 5 and 6 | red, green | green, red | green, red | [nesdev] *PPU registers*, *NTSC video* |
| Noise periods, DMC rates | 2A03's | 2A07's | 2A03's | [nesdev] *APU Noise*, *APU DMC* ("all Dendy-timing clone CPUs use the NTSC period table") |
| Frame counter | 29,830-cycle sequence | 33,254-cycle sequence | 2A03's | [nesdev] *APU Frame Counter*; [blargg] |
| DMA halts | on any read | on an instruction's first cycle only | on any read (assumed, §10) | [nesdev] *APU DMC* |

`Model` (`src/model.rs`) holds each row as a method, and nothing else in the crate names a console: the bus, the PPU
and the APU are configured from it once, at load (`MemoryBus::set_model`, `Ppu::set_model`, `Apu::set_model`), and
copy what they read per cycle into small fields of their own.

---

## 2. The clocks

The PAL master clock is held as the exact ratio 53,203,425 / 2 Hz. The NTSC NES keeps the 21,477,272 Hz and the CPU
clock of 1,789,773.0 Hz that its goldens were recorded with (the true figures are 21,477,272.7 and 1,789,772.7); a
PAL and a Dendy CPU clock are derived from the ratio, 1,662,607.03 and 1,773,447.5 Hz.

The frame loop is unchanged in shape. It earns a scanline's CPU cycles from the master clock and runs the CPU until
the PPU wraps (`Moon_Core.md` §2.1, §3.1); a scanline is 341 dots of the model's master clocks and a CPU cycle the
model's, so on PAL a line earns 106.5625 cycles and the remainder carries as on NTSC. The frontend paces by machine
info's `frame_rate`, which is the master clock over a frame of whole lines: 21,477,272 / 357,368 on NTSC, as before,
and 53,203,425 / 1,063,920 = 50.00698 Hz on PAL and the Dendy.

---

## 3. Which console a game runs as

**The header.** NES 2.0's byte 12, bits 0–1: 0 the NTSC NES, 1 the PAL NES, 2 "multiple-region", a game that "detects
the console's timing and self-adjusts" (run as the NTSC NES, as Mesen also runs it), 3 the Dendy [nesdev] *NES 2.0*. `Cartridge::header_model` reads it, beside the rest of the header's facts.

**An iNES 1.0 header** has a TV-system bit in byte 9, which "very few emulators honor ... as virtually no ROM images in
circulation make use of it", and an unofficial field in byte 10 that "relatively few emulators honor" ([nesdev]
*iNES*). Neither can be trusted to mark a PAL game. MoonRT ignores both and runs such an image as the NTSC NES, which is
what it did for every image before. A database of images known to be PAL, keyed by hash, would choose better for the
common case of an old header. It is not built: it would be a second job for the core (knowing games rather than
consoles) and a data set to maintain, and the form it fits is a component of its own beside the library, which already
keys games by hash, feeding the region setting below. It is proposed to the tester in §11, not built.

**The setting.** The core's settings schema has one entry, `region`, of create scope: **From the cartridge**
(`auto`, the default), **NTSC NES** (`nes.ntsc`), **PAL NES** (`nes.pal`) and **Dendy** (`nes.dendy`), in the category
Console. Its effect is `none`: which console a game runs on is not an accuracy trade, and each value is a console
exactly. The host passes it at create from the values a frontend stored for the console (`CoreEngine`'s settings), so
a player can force PAL for every NES game, or NTSC for a PAL-headed one; a per-game value waits on SR10's file per game
(`EmuSen_CRT.md` §26.2). MoonRT does not claim `SETTINGS`, as Diamond does not: it has no run-scope key, and
`set_settings` is never sent one.

The pre-stable interface, which still serves the step goldens (`MoonMachine`), takes no settings, so a machine made
through it follows the header alone.

---

## 4. The PPU

### 4.1 3.2 dots a cycle

Sixteen master clocks a CPU cycle over five a dot is sixteen dots in five cycles: four cycles of three dots and one of
four. MoonRT's bus runs a cycle's first two dots before its access and the third after it (`Moon_Core.md` §3,
`MemoryBus::tick` and `end_cycle`). On the 2A07 the fifth cycle of every five runs a third dot before its access
(`MemoryBus::long_cycle_dot`), counted by `dot_phase`.

Where the fourth dot falls is NES_MiSTer's placement: in PAL mode `cpu_tick_count` counts CPU cycles to five, and on
the fifth `skip_ppu_cycle` holds the CPU's divider for one PPU tick at the start of the cycle (`nes.v:265`, `295`,
`309`), so that cycle has four PPU ticks with its access on the last, as every other cycle has its access on its last. Mesen splits
each 2A07 cycle into eight master clocks before the access and eight after (`NesCpu.cpp:609`), which puts the extra dot
on whichever cycle the master clock's phase gives; the two agree on the count and differ in which dot of five lands
before the access. No source settles the phase of the pattern against power-on, which is not fixed on hardware either
(nmi_sync's readme: on PAL "the left edge's general position will change randomly each time you press reset"); MoonRT
starts the count at zero at load and keeps it across RESET.

### 4.2 The lines

The vblank flag rises on dot 1 of line 241 on the 2C07, as on the 2C02, and of line 291 on the 6538; the pre-render
line is 311 for both, and neither drops a dot from it on an odd frame. `Ppu::timing` holds the three numbers and the
border's switch, chosen by `Ppu::set_model`; the dot loop reads them where it read constants.

### 4.3 The border

"The border is always black and extends 1 pixel into the top and 2 pixels into each of the left and right edges of the
picture region, whether rendering is enabled or not. Emphasis and greyscale effects do not apply to this border. Note
that this border does not change the behavior of sprite 0 hit compared to NTSC." [nesdev] *PPU rendering*. NES_MiSTer
draws it as colour $0E, after greyscale (`ppu.sv:1856`, `1894`; `edge_mask_color`, `1260`, outside the Vs. System).

MoonRT draws it after each line is composed (`Ppu::draw_border`, out of line and reached only on a model with a
border): colour $0E in the raw picture with no emphasis bits, and that colour's black from the palette in the RGB
frame. The line is still composed whole first, so sprite 0 and every flag are as on NTSC. A frame whose rendering is
skipped draws no border, as it draws nothing.

### 4.4 Emphasis, and the raw picture

The 2C07 swaps the meaning of PPUMASK's bits 5 and 6. MoonRT's RGB frame applies no emphasis on either console
(`Moon_PPU.md` §3.5), so it has nothing to swap. The raw picture, `nes.ppu` (`EmuSen_CoreAPI.md` §6.30), carries
PPUMASK's bits 5–7 as written on both, and the CRT filter's 2C07 path swaps them before it chooses their oscillators
(`EmuSen_CRT.md` §27.2; `CrtFilter.cs`, the PAL `ppuAt`). So the encoding is the same on both chips, its name stays
`nes.ppu`, and machine info's `video.raw` is unchanged; which wave the filter makes of it follows the standard, which
follows machine info's `region` (`EmuSen_CRT.md` §26.4). Pre-swapping the bits in the core would have made the filter
swap them back, and two owners of one fact.

### 4.5 Not modelled

- **The PAL OAM refresh.** The 2C07 refreshes OAM "during scanlines 265 through 310", 24 lines after the NMI line,
  rendering or not, so that a PAL game's OAM does not decay in its long vblank; with rendering off, OAM can still decay
  on lines 311 and 0–264 ([nesdev] *PPU OAM*). Mesen runs sprite evaluation on those lines (`NesPpu.cpp:198`, `1345`,
  quoting an older wording of the page, "roughly 21 scanlines after NMI"); NES_MiSTer evaluates only while rendering.
  MoonRT models neither OAM decay nor any side effect of sprite evaluation on OAMADDR beyond its reset during dots
  257–320 of a rendered line (`Moon_PPU.md` §4), so the refresh has nothing to act on yet: a game that writes OAM by
  `$2003`/`$2004` or DMA late in a PAL vblank, where the refresh would disturb it, is drawn as if it did not.
- **The pixel's shape.** A 2C07 pixel is 1.386 times as wide as tall against the 2C02's 8:7. Machine info keeps the
  4:3 aspect, as VenusRT does for a PAL SNES; the display's shape is the frontend's (`EmuSen_CRT.md` §14–§15), and a
  question for the tester (§11).
- **nmi_sync's PAL demo** cannot be read on MoonRT: it draws by greyscale writes in the middle of lines, which the
  line renderer applies to the whole line (`Moon_PPU.md` §1).

---

## 5. The APU

### 5.1 The tables

The noise periods and DMC rates are [nesdev]'s 2A07 tables in CPU cycles (`NOISE_PERIOD_PAL`, `DMC_RATE_PAL`, beside
the 2A03's in `apu/channels.rs`). They were checked against NES_MiSTer, whose tables are LFSR seeds rather than counts
(`apu.sv:433`, `539`): run until each LFSR reaches its terminal value ($400 for the noise, $100 for the DMC),
the NTSC seeds take table/2 − 1 steps for every entry, and the PAL seeds take [nesdev]'s PAL table/2 − 1 for every
entry of both tables, 32 of 32.

The frame counter's sequence on the 2A07, in CPU cycles, is [nesdev]'s APU-cycle figures doubled as the 2A03's are
(`Moon_APU.md` §2.1): 8,313, 16,627, 24,939, then 33,252, 33,253 and 33,254 in four-step mode, and 33,253, 41,565 and
41,566 in five-step mode. NES_MiSTer's PAL counter is "educated guesses" (`apu.sv:786`) and places the fourth step one
APU cycle earlier, at 16,625. [blargg] decides between them: `07.irq_flag_timing` requires the flag "set three times in
a row 33255 clocks after writing $4017 with $00", which [nesdev]'s figures give (3 cycles of the write's delay and
33,252) and NES_MiSTer's do not.

`Apu::tables` holds the four tables and the clock for the model; the per-cycle step reads them where it read statics.

### 5.2 The sound's rate

The mixer still makes 44,100 stereo samples a second, now of the console's own time: its count of CPU cycles a sample
is the model's CPU clock over 44,100, 37.70 on PAL. A PAL frame is 33,247.5 cycles, so 881.88 sample pairs a frame,
which at 50.007 frames a second is 44,100 a second. Machine info's `audio.rate` is unchanged.

---

## 6. The 2A07's DMA

"This problem is fixed on the 2A07 series of PAL CPUs by forcing DMA to halt on the first cycle of an instruction,
which fetches an instruction opcode and thus isn't normally reading from registers." [nesdev] *APU DMC*, of the DMC
fetch that clocks a pad an extra bit when it lands on a read of $4016.

MoonRT's CPU runs any pending DMA from its read of a cycle (`Moon_Native.md` §3.9). On the 2A07 `run_dma` returns
unless the instruction has spent no cycle yet, which is true at an opcode fetch, at the first read of an interrupt's
entry and at a jammed CPU's read, and the DMA waits for the next. The test costs nothing on the NTSC NES's path, since
`run_dma` is reached only while a DMA is pending. Mesen does the same, interrupts included (`NesCpu.cpp:239`, `381`).
OAM DMA is not changed in practice: its write is an instruction's last cycle, so the next read was already an opcode
fetch.

**Measured** (`only_the_2a03s_dmc_fetches_corrupt_a_pad_read`): a loop reads the pad, A held, while the DMC fetches at
its fastest rate. The NTSC NES read 147 wrong bytes in 10,857 reads in 60 frames; the PAL NES 0 in 12,133.

**A test that passed for the wrong reason, recorded.** The loop's first form took 160 cycles a pass. A fetch every 432
cycles then lands on only ten of its cycles, the multiples of gcd(160, 432) = 16, and none was a read of $4016: the
NTSC NES read no wrong byte, and the check passed only while the wrong-byte count was eight bits wide and happened to be
non-zero. A three-cycle instruction makes the pass 163 cycles, a prime, so the fetches visit every cycle of it in turn
on either console.

---

## 7. Save states

**Version 7** adds two fields at the end of the state: `Model`, NES 2.0's number for the console (0, 1 or 3), and
`Bus.DotPhase`, the long cycle's count. **Only a machine that is not the NTSC NES writes them**; the NTSC NES still
writes version 6, byte for byte as before. That is deliberate: `MoonGoldenTests` hash the full state after every frame
of every golden run, so a byte added to every state would have changed every NTSC golden for a field that, on NTSC,
always says NTSC. A state of version 3 to 6 is therefore the NTSC NES's, and version 7 names its console.

A state loads only into a machine of its own console. One of another is refused with MoonRT's status −12, "a state
saved on another console, the NTSC NES, the PAL NES or the Dendy, than the one this game runs on", and changes nothing
(the load reads into a copy, `Machine::load_state`). The host throws an `InvalidDataException` carrying those words,
from the core's status text on the core ABI and from the shim's `StatusWords` on the pre-stable interface
(`MoonRtPalTests`). Converting a state between consoles is not offered: the PPU's line, the frame counter's step and the DMA's
timing have no counterpart on the other.

Machine info's `state.version` is the machine's: 6 for the NTSC NES, 7 for the others, with `loads_from` 3 to 7 for
both. `MoonRtCore.StateVersion` and `FrameRateHz` now read machine info once a game is loaded, and the shim's check
reads up to version 7.

---

## 8. Tests and mutants

`src/pal_tests.rs`, synthetic programs only, each console beside the NTSC NES as the control:

| Test | Measured |
|---|---|
| `the_ppu_runs_its_dots_per_cpu_cycle` | over 200,000 cycles the PAL PPU stays within 3/5 of a dot of 16/5 a cycle; NTSC and the Dendy exactly 3 |
| `a_frame_is_the_consoles_lines_and_only_the_2c02_drops_a_dot` | 20 rendered frames: 312 × 341 dots each on PAL and the Dendy, none dropped; NTSC drops 10 |
| `vblank_starts_on_the_consoles_line_and_lasts_its_lines` | the flag rises on line 241 (291 on the Dendy) and falls on line 311 (261): 7,458–7,461 cycles on PAL against 7,459.4; 2,271–2,274 on the Dendy and NTSC against 2,273.3 |
| `the_nmi_comes_once_a_frame_on_the_vblank_line` | 20 PAL frames in 664,949 cycles against 664,950 (33,247.5 each); the Dendy 709,280 exactly; NTSC, rendering off, 595,613 |
| `the_frame_irq_comes_at_the_consoles_period` | 20 frame IRQs in 665,080 cycles on PAL (33,254 each), 596,599 on NTSC and the Dendy (29,830) |
| `the_frame_counter_runs_the_consoles_sequence` | the flag first rises 33,255 cycles after `$4017 = $00` on PAL, then every 33,254; 29,831 and 29,830 on NTSC |
| `the_noise_and_dmc_count_the_consoles_tables` | noise periods 4 and 13, DMC rates 0 and 15: each the console's table entry, every period |
| `the_audio_rate_follows_the_consoles_clock` | 100 PAL frames make 88,188 pairs in 1.999721 s of console time, 881.88 a frame |
| `the_header_names_the_console_only_in_nes_2_0` | byte 12's four values; an iNES 1.0 header with byte 9's TV bit set runs as NTSC |
| `the_2c07_draws_its_black_border` | every pixel of a frame with all three emphasis bits set: the border $0E without emphasis and black in RGB, the rest the backdrop with emphasis; none on NTSC |
| `only_the_2a03s_dmc_fetches_corrupt_a_pad_read` | §6 |
| `a_state_records_its_console_and_loads_only_into_it` | NTSC version 6 with no `Model` field, the others 7; each round-trips and runs on equal to the saving machine; each of the six cross loads refused with −12, the machine unchanged |
| `ffi::v1`'s `the_region_setting_chooses_the_console_and_machine_info_reports_it` | the schema checks; `auto`, `nes.pal`, `nes.dendy`, and a PAL header with `auto` and `nes.ntsc`, each the console, region, rate and state version expected; NTSC's rate is the old ratio exactly |
| `model`'s `each_consoles_clocks_are_the_documented_ones` | 50.0070 Hz, 1,662,607 and 1,773,448 Hz, 33,247.5 and 35,464 cycles a frame |

Fifteen mutants were seeded, one at a time, the crate's suite run on each; all were caught, one only after its test was
corrected:

| Mutant | Caught by |
|---|---|
| the 2A07 at 3 dots a cycle | dots, vblank, NMI, audio |
| the long cycle's dot removed | dots, vblank, NMI, audio |
| PAL on the 2A03's APU tables | noise and DMC, frame counter, frame IRQ |
| PAL's four-step sequence one cycle short at its end | frame counter, frame IRQ |
| DMA on every read on the 2A07 | the pad's corruption |
| no border | the border, once the test stopped taking the border's switch from `Model` (it survived the first run) |
| the border with emphasis bits | the border |
| PAL dropping the odd frame's dot | the frame's lines |
| the Dendy's vblank on line 241 | vblank |
| PAL's mixer on the NTSC clock | audio, the model's clocks |
| no console check on a state's load | the state |
| PAL states written as version 6 | the state, the v1 test |
| the long cycle's phase not restored by a load | the state |
| iNES 1.0's byte 9 trusted | the header (and the mutant's own read past a three-byte image, in an existing test) |
| the setting ignored at create | the v1 test |

**[blargg] `pal_apu_tests`**, run read-only from the clone through a scratch program that loads each ROM as each
console and reads the result code at $F8 after 1,500 frames (the suite predates the $6000 protocol, so `--testroms`
cannot read it; the text each printed agrees with the code):

| Console | Passed | Failed |
|---|---|---|
| PAL NES | **10 of 10** | — |
| NTSC NES | 3 of 10 | `04`–`11`, each #2 or #3: too soon, as an NTSC sequence is on a PAL test |
| Dendy | 3 of 10 | the same seven, as its CPU's sequence is the 2A03's |

No other ROM in the clone reports PAL results by a protocol: `nmi_sync/demo_pal`, `window5/colorwin_pal`,
`other/window2_pal` and `nes15-PAL` are screen demonstrations.

**Through the host** (`EmuSen.WiseMan/Cores/MoonRtPalTests.cs`, 6 cases): a NES 2.0 header naming the PAL NES or the
Dendy, the region setting over an iNES header, and the setting's NTSC over a PAL header, each with the television
standard, frame rate and state version the host reads from machine info; a PAL state loaded back into a PAL session;
and each cross load refused with an `InvalidDataException` in the core's words, the session unchanged.

**The NTSC NES unchanged**, measured 2026-10-17 with this build:

- the crate's suite: 22 of 22, the 8 that were there before among them;
- every WiseMan class naming Moon, with `RustStateHeaderTests`, `NesRunToFrameTests` and
  `CoreRegistrationEquivalenceTests`: 277 cases, all passing but one, the registration golden, which recorded the new
  setting's 16 lines (one per NES factory row and discovery world) and was recorded again for them alone;
- the goldens: every synthetic and step run (328 traces), the corpus by blargg's protocol (`EMUSEN_MOONRT_CORPUS`, 263
  traces) and the seven games read from the library in place (`EMUSEN_MOONRT_ROMS=library`), all as recorded, and no
  corpus verdict worse than the C# core's baseline;
- the settings window's layout, pad menus and navigation, window retention, the game's television standard, the screen
  filter's settings and NES drawings, the display aspect, the fit audit and DianaSh's golden corpus: 722 cases, all
  passing;
- the conformance kit on a copy of Super Mario Bros. as the NTSC NES, with a PAL header, forced to the PAL NES and to
  the Dendy by `--settings region=…`, and with a PAL header forced back to NTSC: every case that applies, 21 of 21, in
  each of the five. On the synthetic loop image C15 fails ("step under EACH did not move the machine"), on the library
  before this work as after it, since a jump to itself does not move the program counter; it is the image's and not
  PAL's.

---

## 9. What it costs

MoonRT's bench (`~/.cache/emusen/probe/ci-speed/bench/`, `nesbench` with `bench.sh`'s four games: 900 frames with Start
and A in turn, then 3,000 timed), the dist library before and after, seven rounds interleaved by game and library under
the timing lock, the load average 2.4–5.7 as other work ran; medians in ms a frame:

| Game | Before | After | Change |
|---|---|---|---|
| Super Mario Bros. | 0.905 | 0.899 | −0.6% |
| Zelda | 0.893 | 0.878 | −1.7% |
| Super Mario Bros. 3 | 1.118 | 1.106 | −1.1% |
| Punch-Out!! | 1.085 | 1.076 | −0.9% |

The geometric mean is −1.1%, the after build at or below the before in 22 of 28 paired runs: no cost on the NTSC NES's
path. By `Moon_Native.md` §9.7's caution, a change of a per cent or two either way on this library is placement, not
work, and is not claimed as a gain.

**Same output.** `digest.sh` over the 25 ROMs of `Moon_Native.md` §8.6.4 (the four bench games, three more games, the
two synthetic ROMs and the sixteen board ROMs): the picture, sound and state digests of every frame, and the final
state, are identical before and after for all 25.

---

## 10. The Dendy

It fits cheaply, so it is built. With the facts held by `Model`, the Dendy is a row: PAL's master clock and lines, a CPU
divider of 15 that gives exactly three dots a cycle (so no long cycle), the vblank on line 291, the 2A03's APU tables
and sequence (§1), the border and the swapped emphasis bits. It reports `"pal"`, since its video is PAL and its PPU is
the 6538 whose capture the CRT filter's 2C07 model was read against (`EmuSen_CRT.md` §27.3).

Two of its facts rest on less than the PAL NES's. NES_MiSTer notes at its Dendy row "FIXME vblank doesn't ACTUALLY start
here, just the nmi" (`ppu.sv:232`); MoonRT raises the flag and the NMI together on line 291, as [nesdev] and Mesen do.
And no source was found on whether the Dendy's CPU halts for DMA as the 2A03 or the 2A07 does; MoonRT keeps the 2A03's
rule, as Mesen does. [blargg]'s PAL suite fails on it exactly as on the NTSC NES, which is what the 2A03's sequence
predicts.

---

## 11. Not done, and questions for the tester

- **A database of PAL games**, by hash, for iNES 1.0 images (§3). *Recommendation: a component of its own beside the
  library, which already keys games by hash, writing the per-game region setting once SR10's file per game exists;
  not in the core.*
- **The pixel's shape on PAL** (§4.5). *Recommendation: leave 4:3, as the SNES and the Genesis do on PAL, unless the
  display work of `EmuSen_CRT.md` §14 decides a PAL tube's area otherwise for every console at once.*
- **Mistress's message for a state of the other console.** Mistress compares a state's recorded version with the
  running core's (`MainWindow.Identity.cs:151`) before it loads, so a PAL state (version 7) offered to an NTSC session
  (version 6) is refused there as "saved by a newer build" rather than by the core's −12 with its own words. The
  refusal is right and the reason wrong. *Recommendation: compare against machine info's `loads_from`; it is the
  frontend's to change, and not changed here.*
- The PAL OAM refresh (§4.5), the 2C07's levels (`EmuSen_CRT.md` §27.6), and the phase of the long cycle against
  power-on (§4.1).
