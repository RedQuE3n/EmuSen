# Nephrite_LfoTables — the YM2612's vibrato: the offset the LFO adds to a frequency number

*Written 2026-10-05, at stage 5, before the LFO is built.* This page records the rule in `src/lfo_tables.rs`: for a
channel's frequency number, its PMS and the LFO's position, the amount the chip adds to or takes from the frequency
number. It gives the documents' account, the rule, the measurement that fixes every value of it on the board's chip,
which chip that is, what is left open, and every source opened. It was written under the clean-room protocol of
`Nephrite_Plan.md` §1.3: Yamaha's documents, the project's own pages and sources, and the board model of
`Nephrite_Disputes.md` D-16 run as a black box. No emulator source and no RTL file was opened.

The page keeps the plan's registers: **documented** is a statement a cited source makes; **measured** is a result of a
run on the board, with the tool named; **argued** is reasoning with neither behind it.

---

## 1. What the documents say

- **[S1]** Yamaha, *YM2608 (OPNA) Application Manual*, §2-6 "LFO: Low Frequency Oscillator", pages 33-34, read in the
  Japanese original and its English translation. *Documented*: the LFO is a sine wave; register `$22` bit 3 turns it on
  and bits 2-0 choose its frequency, 3.98, 5.56, 6.02, 6.37, 6.88, 9.63, 48.1 and 72.2 Hz; PMS (`$B4`-`$B6` bits 2-0)
  "adds the LFO to the frequency (phase) information set by F-Number/Block", per channel; its depths are 0, 3.4, 6.7,
  10, 14, 20, 40 and 80 cents. Figure 2-5, the LFO's block diagram, takes one LFO through the on/off switch to every
  channel's PMS block, whose output reaches all four of the channel's operators.
- **[S2]** Yamaha, *YM2608 data sheet*, the register map of the FM part (page 18 as printed): `$22` LFO (FREQ
  CONTROL), `$B4`-`$B6` L, R, AMS and PMS.
- **[S3]** Yamaha, *YM3438 (OPN2C) data sheet*, pages 2-48 and 2-49: the LFO is listed among the chip's blocks
  ("modulates the operators by the output of a low-frequency oscillator, giving the sound a periodic change"), and the
  register map has `$22` and `$B4`-`$B6` as the YM2608's.

What the documents leave open, and §2 to §4 settle on the board: the depth as a function of the frequency number (the
cents are a ratio, and a ratio of a whole number is not a whole number); the number of positions in the LFO's cycle and
the shape of the wave over them; where the result is rounded; whether the block shift comes before or after; and what
happens when the sum leaves the frequency number's eleven bits. No document read describes the PM path's internals.

## 2. The rule (measured on the board)

Write `top` for the frequency number's top seven bits (`fnum >> 4`), `q` for the LFO's position folded into a quarter
wave, and offsets in **half frequency-number steps**.

1. **The cycle.** The vibrato has 32 positions a cycle, `lfo_step` 0 to 31. Steps 0-7 rise through a quarter wave,
   steps 8-15 fall back through it (`q = 15 − step`), and steps 16-31 repeat steps 0-15 with the offset negated. At
   every PMS, `q` 0 and 1 give no offset: the wave stands at zero for four steps around each crossing.
2. **The magnitude.** For each PMS and `q`, the chip sums some of `top`, `top >> 1` and `top >> 2`, and shifts the sum
   right by a fixed amount (`PM_TERMS` and `PM_SHIFT`):

   | PMS | q 2 | q 3 | q 4, 5 | q 6 | q 7 |
   |---|---|---|---|---|---|
   | 0 | 0 | 0 | 0 | 0 | 0 |
   | 1 | 0 | 0 | `top >> 4` | `top >> 4` | `top >> 4` |
   | 2 | 0 | `top >> 4` | `top >> 4` | `top >> 3` | `top >> 3` |
   | 3 | `top >> 4` | `top >> 4` | `top >> 3` | `((top >> 1) + (top >> 2)) >> 2` | as q 6 |
   | 4 | `top >> 4` | `top >> 3` | `top >> 3` | `((top >> 1) + (top >> 2)) >> 2` | `top >> 2` |
   | 5 | `top >> 3` | `((top >> 1) + (top >> 2)) >> 2` | `top >> 2` | `(top + (top >> 2)) >> 2` | `(top + (top >> 1)) >> 2` |
   | 6 | `top >> 2` | `((top >> 1) + (top >> 2)) >> 1` | `top >> 1` | `(top + (top >> 2)) >> 1` | `(top + (top >> 1)) >> 1` |
   | 7 | `top >> 1` | `(top >> 1) + (top >> 2)` | `top` | `top + (top >> 2)` | `top + (top >> 1)` |

   For the frequency number's top bit alone (`top` = 64) this is, in half steps, PMS 7's 32, 48, 64, 64, 80, 96 over `q`
   2-7, PMS 6's half and PMS 5's quarter of that, PMS 4's 4, 8, 8, 8, 12, 16, PMS 3's 4, 4, 8, 8, 12, 12, PMS 2's 0, 4,
   4, 4, 8, 8 and PMS 1's 0, 0, 4, 4, 4, 4. The low four bits of the frequency number never count.
3. **Where it goes.** The offset is added to twice the frequency number, a twelve-bit number in half steps, and the sum
   wraps in twelve bits; the phase increment's base is that number shifted left by the block and right by two, where
   without the LFO it is the frequency number shifted left by the block and right by one (`pm_fnum`). At PMS 7 a
   frequency number of 2,047 therefore wraps to a very low pitch on the wave's upper half.

How each part was fixed, and how well, is §3. The table's entries are not each frequency-number bit's own offset added
up: on the board 2,047 at PMS 4 and `q` 6 is 23 half steps where its bits' offsets, each measured alone, sum to 22 (§3.4).

**Against the documents.** At the peak (`q` 7) the offset for a frequency number of 1,024 is 4, 8, 12, 16, 24, 48 and 96
half steps for PMS 1 to 7: 3.4, 6.7, 10.1, 13.5, 20.2, 40.1 and 79.3 cents, against [S1]'s 3.4, 6.7, 10, 14, 20, 40 and
80. For 2,047 the truncations take PMS 4 to 13.1 cents and PMS 7 to 78.5.

## 3. The measurement

### 3.1 The program

`mdboard.py`'s `lfo_sweep` writes six channels, each with S4 alone at TL 0 (algorithm 7, S1-S3 at TL `$7F`, attack
rate 31, no decay), block 7 and multiple 15, turns the LFO on at rate 7, and then, for each configuration, writes a
frequency number and a PMS to every channel, keys every channel off and S4 on again, and waits about 690 samples, a
cycle of the LFO and more. Its writes come from a table the 68000 walks (`ym_table`): the busy flag waited out before
each write and four NOPs after it, as D-15 requires. The board's pins (`TB_AUDIO`) carry each channel's nine-bit level
once a sample; channel 1's turn is 1,223 cycles into the sample from power-on (D-19), the others' at steps of 336 in the
order 1, 5, 3, 2, 6, 4, which these runs confirmed; the Z80 bus log (`TB_ZBUS`) gives each write's cycle.

Multiple 15 and block 7 are chosen so that one half step of offset moves the phase by 480 of the accumulator's 2^20
units a sample, nearly half of one of the ten phase bits the operator reads: a step's offset changes the phase by
several of those bits within the step, at every frequency number.

### 3.2 Reading the frequency number back

The operator's level is a known function of its ten-bit phase (the log-sine and exponent ROMs of
`Nephrite_OperatorTables.md`, which D-16 found on the board), and the phase is cleared at key-on. The phase increment
is constant within an LFO step, so the record is a sequence of whole-number increments, one a step:

1. **Without the rule.** The first runs (`lfo_sweep` with frequency numbers 512 to 2,047 at PMS 7, multiple 15, rates
   5 and 7) searched every whole-number increment within 130 frequency-number steps of the nominal one, the steps'
   boundaries left free. At rate 5, where a step is 176 samples, exactly one increment explained each step in all six
   channels, and every increment found was a multiple of 32 at block 7: the offset has a resolution of half a
   frequency-number step and no finer. A frequency number of 2,047 at PMS 7 gave increments of 1,952 to 6,016 on the
   wave's upper half, which is the twelve-bit sum wrapped (§2). The steps were 20 samples long at rate 7 and 176 at
   rate 5, on one grid for all six channels. (At multiple 1 and rate 7 the samples did not separate neighbouring
   increments, which is why every later run is at multiple 15.)
2. **On the lattice.** `lfo_fit` then tries, at each step on that grid, every offset from −400 to +400 half steps
   (more than twice the largest seen), starting from the key-on's phase of 0, and carries every phase that explains the
   step's samples into the next. After the last step it works back, keeping only the moves that belong to some
   explanation of the whole record. **A step's offset is taken as determined only when every explanation of the whole
   record gives it the same value**, and only from steps the record covers whole. The sample at which the key-on reset
   the phase is not assumed: every candidate within a few samples of the write is tried, and the explanations from all
   of them are pooled.

### 3.3 Results (measured 2026-10-05)

- **Every frequency number at every PMS** (`mdboard.py lfo`): 2,048 × 8 = 16,384 items in 69 runs of up to 40
  configurations, each over a whole LFO cycle. The runs left 3 of the 16,384 items with a step that two neighbouring
  offsets both explain (1,638 at PMS 0, step 24; 1,096 at PMS 4, steps 20 and 21; 547 at PMS 3, steps 28 and 29; the
  value §2 gives was one of the two each time). Run again four times each, in other channels and configurations, every
  step of the three was decided, no run disagreeing with another. **Every one of the 524,288 offsets is then
  determined by the board**, and every one is §2's: the FNV-1a hash of the board's table, 0x8f2ddbc5, is the one
  `lfo_tables.rs`'s functions give, which the crate's tests hold. In all 1,024 groups of sixteen frequency numbers
  that share their top seven bits (128 groups at each PMS), the sixteen agree at every step: the low four bits never
  count.
- **Single bits and checks** (`lfo_sweep` with each power of two at PMS 1-7, and PMS 0): 83 items, all 32 steps
  determined; they give the table of §2 bit by bit.
- **The blocks.** The same six frequency numbers at PMS 7 and PMS 4, at each block from 0 to 6: wherever the record
  decides a step's increment (all 384 steps at blocks 4-6; 362, 319, 301 and 264 of 384 at blocks 3 to 0, where an
  increment that small moves the phase too slowly over 20 samples to separate neighbouring offsets, or block 0 and 1
  drop the low bits), it is `(pm_fnum << block) >> 2`, and in no step does the record exclude it.

### 3.4 The rule as a sum of bits fails

A reading of the chip as one offset per frequency-number bit, summed, fits every single bit and every frequency number
at PMS 0, 1, 2 and 7, and fails at PMS 3 to 6: in 3,712 of their 8,192 items, at 24,576 steps, always by one half step
too few. The reason is in §2's table. Where a cell is one shifted copy of `top`, or a sum of copies with no shift
after it (all of PMS 7), dropping the low bits copy by copy is the same as summing per bit. The nine cells that sum two
copies and shift the sum after (PMS 3 at `q` 6 and 7, PMS 4 at 6, PMS 5 and 6 at 3, 6 and 7) keep a carry between the
copies that a per-bit sum loses.

The form of §2 is a description fitted to the measurements, not a reading of the die: for each PMS and `q`, every sum
of up to three of `top`, `top >> 1`, …, `top >> 7` shifted right by 0 to 7 was tried against the board's value at all
128 values of `top`, and the table gives one that fits all of them (where several fit, they are the same function of
`top`, and the one with the latest shift is written). That a fit exists for all 64 cells is itself the result; the
tests hold the function, not the form, to the board.

## 4. The LFO's steps and timing (measured; for the rate counter, which is not built here)

- **Steps.** 32 a cycle for the vibrato. The amplitude modulation (AMS 3, S4's AM on, PMS 0, `lfo_sweep(…, ams=3,
  amon=True)`) steps every 5 samples at rate 7, four times in each vibrato step, its steps aligned with the vibrato's:
  so the LFO counts 128 a cycle and the vibrato takes its top five bits. That the vibrato's step is the counter's top
  five bits is argued from the alignment; the AM's own arithmetic is not examined here.
- **Durations.** Samples per vibrato step, rates 0-7 (fnum 1,024, PMS 7, block 7, multiple 15, with free boundaries):

  | Rate | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
  |---|---|---|---|---|---|---|---|---|
  | Samples a step | 432 | 308 | 284 | 268 | 248 | 176 | 32 | 20 |
  | Samples a counter step (÷ 4) | 108 | 77 | 71 | 67 | 62 | 44 | 8 | 5 |
  | Hz at 53,267 samples a second | 3.85 | 5.40 | 5.86 | 6.21 | 6.71 | 9.46 | 52.0 | 83.2 |
  | [S1]'s Hz | 3.98 | 5.56 | 6.02 | 6.37 | 6.88 | 9.63 | 48.1 | 72.2 |

  [S1]'s figures are the YM2608's, at 8 MHz and 55,556 samples a second. At that rate a counter that steps every
  109, 78, 72, 68, 63, 45, 9 and 6 samples gives 3.98, 5.56, 6.03, 6.38, 6.89, 9.65, 48.2 and 72.3 Hz, [S1]'s eight
  figures to within 0.3%: one sample more a step than the board takes at every rate. Whether the YM2612 counts like the
  board or like [S1]'s arithmetic, or [S1] computed its table with a divider one longer, the board cannot say. It is
  left open for the step that builds the counter, which should weigh it with a recording of a console.
- **On and off.** Clearing `$22` bit 3 brings the offset to 0 within two samples, mid-step (`lfo_sweep(…,
  extra=…)`, the LFO turned off at step 13 and on again 104 samples later); turning it on again starts the cycle from
  step 0, whose zero offset lasts until step 2 begins, 39 to 41 samples after the write. In eight runs of identical
  history, one at each rate, the first step after the LFO was turned on was 9 samples shorter than the others at rates
  0 to 5, 6 at rate 6 and 1 at rate 7: the counter is cleared, and some divider ahead of it seemingly is not.

## 5. Which chip this speaks for

The bench's FM chip is the YM3438 inside the model 2 ASIC (D-16). Every value here is that model's. For model 1's
discrete YM2612 they hold as far as the two chips share the PM path, which is argued and not measured: [S3] gives the
YM3438 the YM2608's PMS register and the same register map, no document read gives the YM2612's and the YM3438's PM
paths as different, and the YM3438 is the YM2612's redesign rather than a different family. Nothing measured here
depends on the output stage in which the two differ (the ladder effect, D-13's status ports). The open point of §4, the
dividers, is the one where a console's evidence is most wanted.

## 6. What this page does not cover

The rate counter and its divider, the amplitude modulation's arithmetic, AMS, the wiring into `fm.rs`, channel 3's
special mode (whether S1-S3's own frequency numbers are modulated by the channel's PMS is not measured), and whether the
key code, and so detune and key scaling, are taken from the modulated frequency number (not measured: every run here
used detune 0 and an envelope that does not move).

## 7. Provenance

Every file opened for this work. No URL was opened.

| # | File | Kind | Code on it? | Taken |
|---|---|---|---|---|
| 1 | `~/.cache/emusen/probe/nephrite/docs/yamaha/YM2608J_Translated.PDF`, via `pdftotext` | [S1], English translation | No | §2-6's text, §2-2-1 on `$22` |
| 2 | `~/.cache/emusen/probe/nephrite/docs/yamaha/archive-yamaha-chip-jpn/YM2608-application-manual.pdf`, via `pdftotext` and pages 33-34 as images | [S1], Japanese original | No | The PMS and rate tables, Figure 2-5 |
| 3 | `…/archive-yamaha-chip-jpn/YM2608-data-sheet.pdf`, via `pdftotext` and page 19 as an image | [S2] | No | The register map |
| 4 | `…/archive-yamaha-chip-jpn/YM3438.pdf`, via `pdftotext` and pages 4-5 as images | [S3] | No | The block list and register map |
| 5 | `EmuSen.DianaOS/DianaOS/Etc/Man pages/Hardware/Sega/Nephrite - Genesis/`: `Nephrite_Native.md` §19-21, `Nephrite_Disputes.md` (its rules, D-15 to D-19), `Nephrite_OperatorTables.md`, `README.md` | Project pages | No | How the bench is driven and read |
| 6 | Nephrite's `src/fm.rs`, `src/fm_tables.rs`, `src/lib.rs`, `examples/fmtrace.rs`, `Cargo.toml` | Project source | Yes, the project's own | The phase path the offset enters |
| 7 | `EmuSen.WiseMan/Reference/rtl68k/mdboard.py`, `tb_md.cpp`, `build-referees.sh` | Project tools | Yes, the project's own | The bench's programs and logs |

The directory `~/.cache/emusen/probe/nephrite/docs/` was listed to find the Yamaha documents; nothing else in it was
opened. The bench's binary, built at stage 5's bench step, was copied into work directories of its own and run; no
RTL file and no Verilator-generated file was opened. ~~D-16 records~~ `Nephrite_Native.md` §21.5 records, and since
2026-10-07 `Nephrite_Disputes.md` D-20 (*corrected then: D-16 never held it*), that rows looking like vibrato tables
were displayed from topic 386 at stage 5's writer's step, to a writer other than this page's: nothing here is taken from them, from any other table or from any emulator, and
every value above is the board's.
