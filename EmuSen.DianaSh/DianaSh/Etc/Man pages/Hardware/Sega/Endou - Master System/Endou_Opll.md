# Endou_Opll — the YM2413 (stage 9)

*Written 2026-10-20, at stage 9, and revised the same day once the referee ran.* This page is the provenance of
Endou's YM2413, `src/opll.rs` and `src/opll/`. It records:

- which document or measurement fixes each of its rules;
- how its instrument ROM was measured;
- what is argued, and what is still open.

The build record of the stage is `Endou_Native.md` §16. The disputes it opened are `Endou_Disputes.md` D-2, D-17 to D-20
and D-24.

Marks as in the plan: **[D]** stated by a document, cited; **[R]** read on the date given from the source named;
**[M]** measured on 2026-10-20 with the tool named; **[A]** argued; **[C]** a choice.

---

## 0. The clean room, and what stood in for the referee

**Read** [R 2026-10-20]: Yamaha's *YM2413 Application Manual* in Maxim's HTML (`yamaha/YM2413ApplicationManual.html`,
SHA-256 `fdc12e0e…96d1`), the scans beside it consulted for Table III-4's layout; SMS Power!'s "YM2413", "Audio Control
Port" and "FM Chip Detection"; MacDonald's `smstech` §1 and §7; *OPLx decapsulated* (Nephrite's cache, `56c3d6fa…b56b`);
a forum thread SMS Power!'s "Audio Control Port" cites, "FM/PSG mix: testing the audio control port", for its posters'
tests of port `$F2` on a Japanese Master System and a Mark III with the FM unit, through a summarising fetch; and of
Nuked-OPLL **its header `opll.h` alone** (SHA-256 `4af73da6…82de`), extracted by itself from the tarball pinned in
`Endou_References.md` §3: its chip-type and patch enumerations, its state structure's field list and its three
functions, `OPLL_Reset`, `OPLL_Clock` and `OPLL_Write`.

**Not read**: `opll.c` or any other emulator source; SMS_MiSTer's `VM2413`; and andete's *YM2413 Reverse Engineering
Notes*, which SMS Power!'s "YM2413" page links. The notes carry program code beside their prose, and Nephrite's
provenance list records one of them abandoned on that ground (`Nephrite_OperatorTables.md` §6, row 10); none was
opened. No identifier of the header's structure is used in the code. The header's patch pointer was not followed,
which would have been reading the emulator's data.

**The referee, run on 2026-10-20** [M]. The tester approved building `opll.c` on 2026-10-20. `build-opll.sh` extracted it
into the probe's cache beside the header and compiled it with `tb_opll.c`; it was never opened. Every rule in §3 is now
held to that referee sample for sample, and so is the instrument table in §4. The paragraphs below on Genesis Plus GX
record how the stage's first pass was measured. Several of its rules did not survive the referee. §3 keeps each such
rule beside the one that replaced it, so that the earlier measurement can be judged against its outcome.
Two things were read of the running chip beyond its output pins:

- **The bench** reads the header's cycle counter, `cycles`, to align its samples.
- **One diagnostic probe** (`p6`) printed three of the header's per-slot fields, the envelope's level, the envelope's
  state and the phase, for one voice, while the attack was being investigated. They were observations of the black
  box's state, as a logic analyser would give. Every rule in §3 rests on the output pins, compared sample for sample.

*As first written:* **the referee the plan named** — Nuked-OPLL built from its tarball and driven through its header,
sample for sample (`Endou_Plan.md` §6.1, §10 Q4) — **was not run at that point.** Its bench is written and committed
(`EmuSen.WiseMan/Reference/opll/tb_opll.c` and `build-opll.sh`, §7) and compiles against the header alone; building
`opll.c` from the fetched tarball on the desktop waits on the tester's approval of that compile, which the stage did not
have. The sample-for-sample gate therefore stands open (§7, `Endou_Native.md` §16.10).

**The black box used instead** [C]: Genesis Plus GX v1.7.4 (`58c3414`, pinned in
`EmuSen.WiseMan/Reference/libretro-cores.sha256`), whose core option `genesis_plus_gx_ym2413_core` offers `mame` and
`nuked` [M `--list-options`]; with `nuked`, a Master System program's YM2413 is that build's Nuked-OPLL, run as a
black box through the probe like every other reference. What reaches the probe is not the chip's samples: Genesis Plus
GX resamples its 49,716 samples a second to 44,100 with a band-limited step synthesiser, then holds a DC-blocking
high-pass of 2⁻⁹ a sample (the decay of its start-up offset [M]), and its output response falls as a one-pole low-pass
of coefficient 0.75 at 44.1 kHz would, −0.19 dB at 2 kHz, −2.2 dB at 10 kHz and −3.9 dB at 16 kHz against Endou's flat
synthesiser [M §5.2]. So the black box answers two kinds of question exactly and no other:

- **levels, rates, frequencies, spectra and envelopes**, which survive a resampler; and
- **whether two recordings are equal**: two programs that differ only in data bytes run with the same timing, and the
  resampler is linear and deterministic, so their recordings are equal sample for sample exactly when the chip's
  outputs were. This is how instrument 8 was measured exactly (§4).

A one-bit difference in the chip's output a sample at a time is below what the first kind resolves; that is the
referee's to settle (§7).

## 1. The chip as built

Endou's OPLL runs a sample at a time, 72 master clocks each [D (III-1-6): fsam = fM / 72], clocked at the Z80's clock
(both are the console's 3.58 MHz, 53.693175 MHz over 15). Each sample: the nine channels' outputs, then every slot's
envelope, then every slot's phase, then the LFOs. It returns the melody's sum and the rhythm's sum in the DAC's units
(§3.3), the rhythm's twice [D (III-4): "the same percussive sounds are output twice"]. The caller (`sound.rs`) runs it
to each write's clock and to the frame's end, steps the sum into `emusen-native`'s synthesiser beside the PSG, and lets
port `$F2` choose what sounds (`Endou_Native.md` §16.3).

## 2. The log-sine and exponent ROMs (D-2)

`src/opll/tables.rs` holds the OPL2's two ROMs as *OPLx decapsulated* prints them, the same 512 numbers
`Nephrite - Genesis/src/genesis/fm_tables.rs` holds and `Nephrite_OperatorTables.md` documents.

**Measured on the referee, 2026-10-20** [M `d2.py`, `d2b.py`, `d2c.py`]. The test used a build made for the purpose:
one entry was moved up or down by one through an environment variable, and that build was never committed. The two
tables allow 1,015 such changes that stay inside a ROM's width (12 bits for the log-sine, 10 for the exponent). Each
was compared with the referee over the synthetic voices of §7, which are exact, plus three more sets:

- 40 modulated voices, every multiple, TL 0-8 and every feedback;
- a sweep of TL 0-4 by the four KSL codes across F-numbers and blocks;
- the retriggers of §3.5.

**1,003 of the 1,015 changes alter at least one sample.** Twelve did not, all of them log-sine entries next to the zero
crossing:

- **0 and 1, each moved down.** These cannot alter any output. At every attenuation they change only bits that the
  exponent's shift of 8 discards.
- **2, 4, 6, 8 and 10, the rest of the twelve.** These alter the twelve-bit output only at a few odd attenuations,
  which none of the voices held.

The referee therefore computes with tables equal to the OPL2's, entry for entry, wherever an entry can be seen. The
same caveat as Nephrite's D-16 applies: the referee is a model, and its agreement is evidence about a console only so
far as its ROMs came from a die. D-2 is settled on that footing. `Endou_Plan.md` §3.2 now has the tables move to a
Beryl crate that Nephrite and Endou both read. That move touches Nephrite, so it is a question for the tester
(`Endou_Native.md` §16.15) and is not made here.

## 3. The rules

Each rule as built, with its evidence. "The referee" is Nuked-OPLL run sample for sample (§7). Unless a row says
otherwise, "exact" means every sample of the voices named is equal, measured 2026-10-20. "The box" is the stage's
earlier black box (§0). A rate is RATE = 4R + Rks, as the manual defines it.

### 3.1 Ports and registers

| Rule | Evidence |
|---|---|
| Port 0 takes the address, port 1 the data for the address last written; the address persists | [D (II-4), (III-1)] |
| Every register is 0 after reset | [D (III-1)] |
| `$00`-`$07` the original instrument, `$0E` rhythm, `$0F` test, `$10`-`$18`, `$20`-`$28`, `$30`-`$38` the channels; a write to an address the map does not name, or past `$3F`, changes nothing | [D (II-7)]; the second half [A] |
| A write takes effect for every slot at the next sample | [M referee]: a data write placed late in a sample's eighteen cycles reaches every channel one sample later; placed early, it reaches channel 6's carrier and channels 7-9 in the same sample but the others in the next (§7). Endou applies writes between samples, which is the first case |
| `$0F`, the test register, is kept and does nothing | [D (III-1-1): "Normally all addresses are 0"] [A] |

### 3.2 The phase generator

| Rule | Evidence |
|---|---|
| A phase accumulator of 19 bits a cycle; its top ten bits address the sine | [M] the box's frequencies; [M referee] every multiple, exact |
| The step is (((2F + vibrato) × 2^block) / 2, truncated) × 2·MUL / 2 | [M referee] `rnd.py`: with block 0, F-number 230 and MUL 7, the step taken as ((2F + vibrato) × 2^block × 2·MUL) / 4 differed in 3,309 of 4,500 samples, and this rule makes them equal. The first rule was the stage's own, and its seeded fault F5 (truncation before the multiple) was in fact the chip's rule; the box's measurements, at even multiples, could not tell the two apart |
| MUL's codes: ½, 1-10, 10, 12, 12, 15, 15 | [D Table III-1]; [M referee] `mul-0` to `mul-15` exact |
| The phase restarts at 0 when the carrier's damp ends (§3.5), for both of the channel's slots | [M referee] `retrig*.py`, `shrink.py`; the modulator's own damp end does not restart its phase |

### 3.3 The operator and the output

| Rule | Evidence |
|---|---|
| A ten-bit phase: bits 0-7 address the quarter sine, mirrored when bit 8 is set; bit 9 is the sign | *OPLx decapsulated*; [M referee] the sines exact |
| The attenuation, in 0.375 dB, is added to the log-sine as `att << 4`, and the exponent returns ((EXP[255 − f] + 1024) × 2) >> int: 4,084 at full level | the tables [D]; [M referee] |
| A negative half is the ones' complement of the magnitude, and a magnitude past the exponent's range is a negative zero, −1, on the negative half | [M referee] `modprobe`: modulated voices went from 1,800 to 3,450 of 3,501 samples equal with the complement, and `decay-5` was exact only with the negative zero |
| The half-wave sine (DM, DC) gives a negative zero on its negative half | [M referee] `half-both`: the referee's −4 there is a channel's negative zero (below) |
| A slot whose level is 127 gives a positive zero | [M referee]: a silent chip sums to exactly its idle level |
| The carrier's phase takes the modulator's output with its lowest bit cleared | [M referee] `mod-tl*`, exact |
| Feedback is the modulator's last two outputs, each shifted right by one, summed, and shifted right by 8 − FB | [M referee] `fb-1` to `fb-7`: from 483-1,952 of 3,501 samples equal to 3,499-3,500, the rest at the voice's end. It replaces the stage's earlier (sum) >> (9 − FB), which differs only in the lowest bit |
| The DAC takes the output's sign and its top eight bits as magnitude. On the chip's pins, a channel's turn shows the magnitude plus one on one pin and the sign as ±1 on the other; an idle turn is +1 on both. Endou gives each channel's turn less the idle level: `o >> 4` for an output o ≥ 0, and −((!o >> 4) + 4) for o < 0 | [M referee] the pins of `p5`, every voice; this replaces the nine-bit `o >> 3`. **It settles D-19**: a quiet note's negative half never comes nearer zero than −4 units, so it sounds louder than a truncation gives, and at VOL 15 the waveform shows four levels and no zero, as the box did |

### 3.4 Levels

| Rule | Evidence |
|---|---|
| TL adds 2 steps of 0.375 dB per unit to the modulator; VOL 8 per unit to the carrier | [D Table III-3, (III-1-8)]; [M referee] `mod-tl*` |
| The key scale at 3 dB an octave is Table III-5's: in block 7, by the F-number's top four bits, 0, 24, 32, 37, 40, 43, 45, 47, 48, 50, 51, 52, 53, 54, 55, 56 steps, each lower block 8 fewer, never below 0 | [D Table III-5]; [M referee] `ksl-1` to `ksl-3` exact |
| **KSL 1, 2 and 3 are 1.5, 3 and 6 dB an octave** | [M] the box (D-18); [M referee] exact |
| An attenuation past 127 is 127 | [A] |

### 3.5 The envelope

Endou counts samples from reset. A carrier's envelope reads that count plus 1 as its tick count; a modulator's reads it
plus 2. The modulator also reads the LFOs one sample ahead (§3.6). [M referee: `lead.py` found, slot by slot, the
offsets that make each slot's attack, decay, tremolo and vibrato exact, and they are these on every channel.]

| Rule | Evidence |
|---|---|
| Seven bits, 0.375 dB each | [D (III-3)] |
| Rates 4-47: a decay, sustain or release steps 4-7 times in eight ticks of 2^(13 − RATE/4) samples, on the fourth sample of a tick | [M referee] `decay-*`, `release-*`, `atk.py` |
| Rates 48-59 step on some samples of each group of four, by RATE's low bits. Rates 48-51 step on the fourth sample of each pair of pairs, plus the second in the groups `GROUPS` marks. Rates 52-55 step on every odd sample, plus the even ones in the marked groups. Rates 56-59 step one a sample, two in the marked groups. Rates 60-63 step two a sample | [M referee] `pattern.py`: the steps by the count modulo 16 at every rate 48-63, and `rr12`-`rr14` exact after |
| An attack step lowers the level by (level × w) / 16 + 1, where w is the step's weight. Rates 4-47 weigh `STEPS` on each of the first four samples of a tick. Rates 48-51 weigh 1 every sample, 52-55 weigh 2 and 56-59 weigh 4, each doubled in the groups `GROUPS` marks. Rates 60-63 go to 0 at once | [M referee] `atk.py`: at rate 16 the levels after the tick are 119, 111, 104 and 97 on four running samples. The stage's earlier rule, level −= (level >> 2) + 1 once a tick, gave 95 and a tail longer than the box's (the open point of the stage's first pass), and is retired |
| An attack at rate 60 or more sets level 0 on its first update and starts the decay on the next | [M referee] `decay-13` |
| A decay that has reached its sustain level does not step: the check comes before the step | [M] the box; [M referee] exact |
| A sustained tone (EG-TYP 1) holds at its sustain level; a percussive one goes on at RR | [D (III-1-2), Fig. III-1] |
| Key-off releases the carrier: a sustained tone at RR, a percussive one at 7, either at 5 when SUS is set. Only the percussive tone without SUS steps on the key-off's own update | [M] the box's rates; [M referee] `modrel.py`, 348 key-offs placed on and beside their ticks, all exact |
| Key-off holds the modulator at its level | [M referee] `modrel.py`: of rate 0, the modulator's RR, and 1-15, only 0 is exact, with and without SUS and for either tone |
| A key-on starts the damp, at rate 12 with the key scale. The damp ends at level 124 or above, and the attack starts from the level it reached | [M referee] `retrig.py`, `retrig2.py`: a retrigger from full level steps three times in eight samples at RATE 50, not once in four; 72 retriggers at eight attack rates, three pitches and three gaps, exact. This replaces the stage's earlier rate 48 to level 127 |
| A falling envelope (sustain or release) at level 124 or above is cut to 127 on its next update | [M referee] `vib`'s release, whose last sound is at level 124 |
| A slot at level 127 sounds nothing | [M referee]. The stage's earlier rule, silence from 124, came from the box's 124.8 steps (`m13.py`), which this rule also gives |
| A modulator's damp ends at its own 124. Its phase restarts when its carrier's damp ends | [M referee] `shrink.py`: three retriggers (a held modulator with a silent carrier, the reverse, and one with both sounding) exact only so |

### 3.6 The LFOs

| Rule | Evidence |
|---|---|
| The tremolo: a counter from 0 to 105 and back, stepped at each 64th sample, with no hold at either end; the level is its value over 8: 13,440 samples a cycle, 3.70 Hz. A modulator reads the value the next step will give | [M referee] `am`, `modam` exact. The stage's earlier 0-104 with each end held twice gave the box's plateau lengths to within its resolution, and is retired |
| The vibrato: eight steps of 1,024 samples, the F-number moved by 0, half, full, half, 0 and the same below. A carrier reads the sample count, a modulator the count plus one | [M] the box; [M referee] `vib`, `modvib` exact |

### 3.7 The order within a sample

| Rule | Evidence |
|---|---|
| The carrier takes the modulator's output of the sample before | [M] the box; [M referee] exact |
| The envelopes and the phases step after the outputs, a carrier's envelope before its modulator's | [M referee] |

### 3.8 The rhythm

| Rule | Evidence |
|---|---|
| Rhythm mode (`$0E` bit 5) gives channels 7-9 to the drums: the bass drum both slots of channel 7, the high hat and snare drum channel 8's, the tom-tom and top cymbal channel 9's; `$0E`'s bits key them, beside the channels' own KEY | [D (III-1-7), Table III-9, (II-5)] |
| The high hat's and tom-tom's slots take their volumes from `$37` and `$38` as a carrier does | [D (II-7)'s rhythm map] |
| The rhythm sounds twice a sample | [D (III-4)]; [M referee] the pins of `p5` |
| The bass drum is a two-slot voice; the tom-tom a plain sine on its slot | [D (V-4)]; the tom-tom [A] |
| The high hat, snare drum and top cymbal play phases made of a 23-bit noise register and of bits of the high hat's and top cymbal's slots' phases | [A], as the OPL2 is described |

**Not yet held to the referee.** All five drums start 1-3 samples before Endou's on the referee: a write to `$0E` reaches
the drums sooner than a key bit reaches a melody channel. The bass drum's patch can only be fitted with that settled
first. The rhythm is open (§8).

### 3.9 What differs on channels 4-9

On channels 1-3 every synthetic voice is exact. On channels 4-9, two things still differ [M referee `lead.py`,
`eglead.py`]:

- A modulator's attack at RATE 44 or more differs in its first 30-60 samples: 62, 31 and 19 samples of 8,000 at RATE 45,
  49 and 53. No offset of the modulator's tick count, its attack's groups or its key-on makes it equal.
- On channel 9, a carrier's decay steps one sample early at some of its ticks: 6-11 samples of 8,000.

Both are open (§8). They are a few samples in a note's first 60, or single samples. The instrument table was measured
on channel 1 (§4).

## 4. The instrument ROM, measured

The manual names the fifteen instruments and the five drums and gives none of their parameters [D (II-7)], and no
document read states them. The table in `src/opll/tables.rs` is measured: for each instrument, the eight bytes of the
tone registers that sound the same as the instrument. Its provenance is black-box measurement (the tester's answer
of 2026-10-20), first on the box and **since 2026-10-20 on the referee**.

**On the referee** [M `fit.py`, `examples/opllref.rs fit`]. Six voices of instrument N on channel 1 are played on the
referee:

- five F-numbers and blocks;
- VOL 0, 3 and 8;
- one voice with SUS;
- notes of 1,500-3,000 samples, each with its release.

Endou's chip then plays the same writes with the tone registers set to a candidate and instrument 0. The descent:

1. Changes one field at a time, then pairs of fields.
2. Kicks out of a local best at random, up to 400 times.
3. Counts equal samples first over each voice's opening 300-900 samples, then over the whole.

Endou's model is exact for the original instrument (§3). So a candidate equal in every sample is the instrument, up to
fields these notes cannot reach (an RR the release never uses, for instance).

| # | Instrument | `$00`-`$07` | On the referee |
|---|---|---|---|
| 1 | Violin | `71 41 1E 17 D0 78 00 10` | **exact**, 22,494 of 22,494 samples |
| 2 | Guitar | `13 41 1A 0D D8 F7 23 13` | **exact** |
| 3 | Piano | `33 01 99 00 F2 C4 20 89` | best fit: exact on the six short notes, 11,757 of 30,000 samples on a note of 20,000 |
| 4 | Flute | `31 41 2A 07 90 64 00 20` | best fit: 29,433 of 30,000 samples. Every difference is in an attack, and is one or two units at levels near silence |
| 5 | Clarinet | `32 21 1E 06 E0 76 00 28` | **exact** |
| 6 | Oboe | `31 22 16 05 E0 71 30 B8` | **exact** |
| 7 | Trumpet | `01 41 1D 07 8E 80 02 17` | **exact** |
| 8 | Organ | `23 21 2D 14 A0 70 00 37` | **exact** |
| 9 | Horn | `41 61 1B 06 64 65 16 17` | **exact** |
| 10 | Synthesizer | `41 61 0B 18 85 F0 26 07` | **exact** |
| 11 | Harpsichord | `33 01 87 11 F0 E0 00 04` | best fit: 29,988 of 30,000. Every difference is in a note's first 20 samples |
| 12 | Vibraphone | `17 C1 24 07 F8 F8 22 12` | **exact** |
| 13 | Synthesizer Bass | `61 50 0C 05 C2 F5 10 42` | **exact** |
| 14 | Acoustic Bass | `01 01 55 03 C3 92 96 1F` | **exact** |
| 15 | Electric Guitar | `61 41 89 03 F1 E4 50 13` | **exact** |

The five that the box found exact (1, 5, 6, 8 and 13) are exact on the referee too. The others' bytes changed. The
box's best fits differed in modulator TL, attack and decay rates and multiples. Its residuals of −6 to −28 dB were
those fields; the referee shows them one sample at a time.

A local best can fit short notes and not long ones. Instruments 9, 10 and 15 were exact on the six short notes,
and then differed on a note of 20,000 samples (the reference voice `instrument-N`). Two such notes were added to the
set: that one, and one at VOL 2 with a retrigger. Refitted on all eight, they are exact. Instrument 3 is still the
first kind.

Instruments 4 and 11 resisted 400 kicks each. On the referee itself, each differs from the original instrument
loaded with its best bytes only in its attacks [M `romvar.py`]. For 11, no single field of the original instrument
reproduces those attacks either, so its ROM entry may hold what the tone registers cannot. This is open (§8).

The descent's local bests are deceptive. Instrument 2 sat at 20,090 of 22,494 samples from the box's bytes, with no
field and no pair of fields giving more. It was exact only from a kick that changed three fields at once.

### 4.3 The rhythm's patches

| Channel | Patch | How |
|---|---|---|
| 7 (bass drum) | `DE 11 1E 08 C2 D7 0B 6C` | on the box, against an original-instrument voice on channel 7 in melody mode, scaled by two: −17.3 dB |
| 8 (high hat, snare drum) | `11 4A C0 88 66 78 A9 EC` | the slots' envelopes fitted on Endou's model to the box's RMS, their multiples to its spectra (§3.8) |
| 9 (tom-tom, top cymbal) | `05 C1 00 00 F8 7F 59 45` | the tom-tom against a carrier's sine on channel 9 in melody mode, scaled by two: −24.1 dB; the top cymbal as the high hat |

They are the box's best fits still. The referee's fitter takes them (`Opll::rhythm_patches`, `rfit.py`). Its first
run, on the bass drum, found no patch near the referee's: the drums' earlier start (§3.8) must be modelled first.

## 5. How the box was measured

### 5.1 The tools

Python programs in the stage's scratch (not committed): `opllrom.py` writes a Master System image that runs a table of
YM2413 writes with timed waits, `bb.py` runs it on the box through the probe (and on Endou through `examples/screen.rs`),
`meas.py` and the scripts named in §3 measure; `spec` (Rust) gives spectrogram, RMS-envelope and band distances; `fit`
(Rust, on Endou's crate) renders scripts with Endou's chip through the same synthesiser and the box's measured output
response, for the fits of §4 and §4.3.

### 5.2 The box's output response

A carrier's sine at 200 Hz to 20 kHz on the box and on Endou's chip resampled flat [M `hresp.py`]: the box lower by
0.01 dB at 200 Hz, 0.19 at 2 kHz, 0.66 at 4 kHz, 1.19 at 6 kHz, 2.23 at 10 kHz, 3.26 at 14 kHz and 3.93 at 16 kHz,
which a one-pole low-pass `y += 0.75 (x − y)` at 44.1 kHz gives to 0.2 dB. The comparisons of §3 and §4 put Endou's
output through it; Endou itself has no such filter, the console's analogue stage being a named loss
(`Endou_Native.md` §12.3).

## 6. The tests

`src/opll/tests.rs` holds each rule of §3 in a test:

- the tables against their formulas;
- the operator's full scale, ones' complement and negative zeros, and the DAC's levels;
- a tone's frequency, and the phase step's truncation, including block 0's lost half;
- the key scale's codes;
- the decay that does not step at its sustain level;
- the carrier taking the modulator's previous output with its lowest bit dropped, and feedback's halved outputs;
- the cut to 127 from level 124;
- the LFOs' shapes and periods, and the modulator's sample of lead;
- the release rates, the key-off that holds the modulator, and the percussive tone's step on its key-off;
- the slow attack's four running steps, the fast attack's weights, the slow decay's tick sample, and the fast decay's
  groups;
- the damp's end at 124, the attack from there, and the instant attack's next-update decay;
- the modulator's phase restarting with its carrier, and running on through its own damp's end;
- the rhythm's keys, an unnamed address, and the state.

`every_instrument_and_drum_sounds_and_the_exact_ones_are_their_registers` holds every instrument exact on the referee
to its bytes on Endou's own chip.

The referee itself is not in the tests. Its source may not be read, and its build needs the tarball. §7 says how to
rerun it.

## 7. The referee bench

`EmuSen.WiseMan/Reference/opll/build-opll.sh` does three things:

1. checks the tarball's SHA-256;
2. extracts `opll.c`, `opll.h` and `LICENSE` into the probe's cache without opening them;
3. compiles `tb_opll.c` against the header, with `opll.c` beside it.

`tb_opll.c` resets the chip for each voice and runs eighteen calls of `OPLL_Clock` per sample. A sample begins at the
header's cycle counter 5 (`OPLL_FIRST_CYCLE`). A write puts the address at the sample's call 2 and the data at call 6
(`OPLL_DATA_CALL`). The bench prints each sample's sum of the two output pins over its eighteen calls.

The window and the write's place were chosen by measurement on 2026-10-20 [M `dc` runs]:

- **The window starting at cycle 0.** Channels 7-9's turns fall at cycles 2-4, at the end of the chip's sample, so they
  showed a sample later than channels 1-6.
- **The data at call 4 from cycle 0.** Channel 6's carrier and channels 7-9 saw a write in its own sample, the others in
  the next.
- **As chosen now.** Every channel shows the same lag of one sample.

`examples/opllref.rs` holds the tools:

- **`voices`** prints the synthetic set: sines, every multiple, modulation by TL, feedback, half waves, attacks, decays,
  releases, the key scales, the key-scaled rate, the tremolo and vibrato, the fifteen instruments and the five drums.
- **`compare`** runs Endou's chip on the same writes and prints, a voice a line, how many samples equal the referee's
  sample one later, less the idle level of 36.
- **`diff`, `egprobe`, `trace` and `whatif`** show where a voice differs, the levels that fit the referee's samples,
  Endou's slots, and Endou with a slot overridden.
- **`fit`** is §4's descent.

The scripts named in §3 and §4 drive them from the stage's scratch and are not committed.

To rerun: `bash build-opll.sh`, then in Endou's crate
`cargo run --release --example opllref -- voices | <bench> > referee.txt` and
`cargo run --release --example opllref -- compare referee.txt`. On 2026-10-20 the 54 synthetic voices are exact, the
last sample excepted (the lag leaves it no partner). Of the fifteen instruments, twelve are exact. The drums are
not (§3.8).

## 8. Open

- **The rhythm** (§3.8, §4.3). Model the drums' earlier start, then fit the three patches and check the noise and the
  phase bits on the referee.
- **Channels 4-9** (§3.9). The modulator's fast attack in a note's first 60 samples, and channel 9's carrier at some
  decay ticks.
- **Instruments 3, 4 and 11** (§4): a long note for 3; an attack, which no original instrument seems to give, for 4
  and 11.
- **D-2's last twelve entries** (§2). Ten could be seen at attenuations no voice held; two cannot be seen at all.
- **Port `$F2`'s counter and decoding** (D-17), argued.
