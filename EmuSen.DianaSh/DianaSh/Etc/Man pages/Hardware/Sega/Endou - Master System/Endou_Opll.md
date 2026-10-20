# Endou_Opll — the YM2413 (stage 9)

*Written 2026-10-20, at stage 9.* This page is the provenance of Endou's YM2413, `src/opll.rs` and `src/opll/`: which
document or measurement fixes each of its rules, how its instrument ROM was measured, what is argued, and what is
left for the referee. The build record of the stage is `Endou_Native.md` §16; the disputes it opened are
`Endou_Disputes.md` D-2 and D-17 to D-20.

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

**The referee the plan named** — Nuked-OPLL built from its tarball and driven through its header, sample for sample
(`Endou_Plan.md` §6.1, §10 Q4) — **was not run at this stage.** Its bench is written and committed
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
envelope, then every slot's phase, then the LFOs. It returns the melody's sum and the rhythm's sum in the DAC's nine-bit
units, the rhythm's twice [D (III-4): "the same percussive sounds are output twice"]. The caller (`sound.rs`) runs it
to each write's clock and to the frame's end, steps the sum into `emusen-native`'s synthesiser beside the PSG, and lets
port `$F2` choose what sounds (`Endou_Native.md` §16.3).

## 2. The log-sine and exponent ROMs (D-2)

`src/opll/tables.rs` holds the OPL2's two ROMs as *OPLx decapsulated* prints them, the same 512 numbers
`Nephrite - Genesis/src/genesis/fm_tables.rs` holds and `Nephrite_OperatorTables.md` documents. That they are the
OPLL's is **argued**: the manual describes the operator as that family's (a sine table and an exponent, no multiplier,
[D (I-3), (III-4)]), and the black box's levels agree with them to the resolution it gives (a full-level sine is
4,111 against Endou's 510 nine-bit units scaled to the same, every 3 dB volume step to 0.1 dB down to 24 dB [M §5.1]),
which cannot tell one entry from its neighbour. **D-2 is not settled**, so the tables do not move to a Beryl crate
(`Endou_Plan.md` §3.2's condition); Endou holds its own copy, its provenance this section. The referee settles it as
Nephrite's D-16 settled the OPN2's, each of the 512 entries moved by one in a build made for the purpose (§7).

## 3. The rules

Each rule as built, with the evidence for it. "The box" is the black box of §0; a rate is RATE = 4R + Rks as the
manual defines it.

### 3.1 Ports and registers

| Rule | Evidence |
|---|---|
| Port 0 takes the address, port 1 the data for the address last written; the address persists | [D (II-4), (III-1)] |
| Every register is 0 after reset | [D (III-1)] |
| `$00`-`$07` the original instrument, `$0E` rhythm, `$0F` test, `$10`-`$18`, `$20`-`$28`, `$30`-`$38` the channels; a write to an address the map does not name, or past `$3F`, changes nothing | [D (II-7)]; the second half [A] |
| The writes take effect at the sample after the chip has been run to the write's clock; the manual's waits (12 clocks after an address, 84 after data) are the program's to keep, and a write made sooner is taken all the same | [D Table II-2] for the waits; the rest [A]: the box cannot show a write's latency |
| `$0F`, the test register, is kept and does nothing | [D (III-1-1): "Normally all addresses are 0"] [A] |

### 3.2 The phase generator

| Rule | Evidence |
|---|---|
| A phase accumulator of 19 bits a cycle; its top ten bits address the sine | [M]: frequencies from zero crossings over seconds, `m7.py`: F-number `$101` in block 0 with MUL 0 is 128 steps of the 19-bit cycle a sample (12.1377 Hz), not 128.5 (12.19); `$103` 129 (12.2325 Hz); with MUL 1, 257 |
| The step is ((2F + vibrato) × 2^block × 2·MUL) / 4, truncated | [D (III-1-6): ΔP = 2^B × F′ × MUL]; the truncation and the vibrato's half-units [M, the rows above and §3.6] |
| MUL's codes: ½, 1-10, 10, 12, 12, 15, 15 | [D Table III-1] |
| The phase restarts at 0 when the attack begins, after the damp (§3.5), and not at the key-on | [M `phasereset.py`, `phasereset2.py`: two programs whose first notes start 2.5 samples apart give equal recordings after a retrigger from silence; a retrigger from full level, whose damp is longer, does not] |

### 3.3 The operator and the output

| Rule | Evidence |
|---|---|
| A ten-bit phase: bits 0-7 address the quarter sine, mirrored when bit 8 is set; bit 9 is the sign | ["OPLx decapsulated"], as for the OPN2 (`Nephrite_OperatorTables.md` §3.1) |
| The attenuation, in 0.375 dB, is added to the log-sine as `att << 4` and the exponent returns ((EXP[255 − f] + 1024) × 2) >> int: 4,084 at full level | the tables [D]; the 0.375 dB unit [D (III-3)]; every volume step of 3 dB measured to ±0.1 dB down to 24 dB [M `volsweep.py`] |
| The half-wave sine (DM, DC) is silent on its negative half | [D (III-1-3): "rectified to half wave"]; its level, 6 dB under the sine's, to 0.1 dB at 2 kHz and 4.8 kHz [M `cmpv.py`] |
| The modulator's output is added to the carrier's ten-bit phase as it stands: full level, ±4,084, is ±4 cycles | [M `m8.py`: indices fitted to Bessel spectra, TL 16 → 2π, TL 24 → π, TL 32 → π/2] |
| Feedback adds the modulator's last two outputs, shifted right by 9 − FB, to its own phase | [D Table III-6: FB 1 = π/16 … 7 = 4π, which this shift gives exactly]; the two outputs' sum [A, as the OPL2] |
| A channel's output is its carrier's twelve bits shifted right by three, nine bits as the DAC takes them | [D (II-1): "a built-in 9-Bit DA Converter"] [A] for the shift. The box's small signals are larger than this gives: D-19 |

### 3.4 Levels

| Rule | Evidence |
|---|---|
| TL adds 2 steps of 0.375 dB per unit to the modulator; VOL 8 per unit to the carrier | [D Table III-3, (III-1-8)] |
| The key scale at 3 dB an octave is Table III-5's: in block 7, by the F-number's top four bits, 0, 24, 32, 37, 40, 43, 45, 47, 48, 50, 51, 52, 53, 54, 55, 56 steps, each lower block 8 fewer, never below 0 | [D Table III-5], whose 2.525 and 14.825 are misprints for 2.625 and 14.625 (every other entry is a multiple of 0.375 dB) |
| **KSL 1, 2 and 3 are 1.5, 3 and 6 dB an octave** | [M `cmpshow.py`: at F-number `$1A0`, block 7, KSL 1 lowers the sine 10 dB, KSL 2 20 dB, KSL 3 more; in block 5, 7, 14 and 28.5 dB]. The manual's Table III-4 gives bits 7 and 6 the other way; its text lists the three in this order. D-18 |
| An attenuation past 127 is 127 | [A]: the envelope's seven bits [D (III-3)] |

### 3.5 The envelope

| Rule | Evidence |
|---|---|
| Seven bits, 0.375 dB each | [D (III-3)] |
| Rates 4-51 step 4, 5, 6 or 7 times in eight ticks of 2^(13 − RATE/4) samples (by RATE's low bits): one step every 8,192 samples at rate 4 | [M `m3.py`, `dbg2.py`: decay slopes at rates 4-43 double every four and grow as 4:5:6:7 within them; at rate 4, 0.375 dB steps 8,201 samples apart]. The manual's Table III-7 has the same ratios, its absolute times being those of a 96 dB range |
| Rates 52-55 step 4-7 of every eight samples, 56-59 one more, 60-63 two a sample; an attack at 60-63 is at once | [A] from the doubling and Table III-7's 0.00 ms; the box resolves rates past 43 only roughly |
| The attack falls as level −= (level >> 2) + 1 at each step | [M `m12.py`]: at rate 4 the box's plateaus are 95, 71, 54, 40, 29, 21, 14, 10, 6, 2, 0, each a tick apart; this rule gives 95, 71, 53, 39, 29, 21, 15, 11, 8, 5, 3, 2, 1, 0. The two agree to the eighth step and the box's tail is shorter: an open point (§8) |
| A decay that has reached its sustain level does not step: the check comes before the step | [M]: a carrier with DR 15 and SL 0 sounded 0.75 dB under the box until the order was changed, and equal after, to 0.02 dB [`harmcmp.py`] |
| SL is 3 dB a unit | [D (III-1-5)] |
| A sustained tone (EG-TYP 1) holds at its sustain level; a percussive one goes on at RR | [D (III-1-2), Fig. III-1] |
| Key-off: a sustained tone releases at RR, a percussive one at 7, either at 5 when SUS is set | [M `m10.py`]: step periods after key-off of 2,048, 256, 32 samples for RR 3, 6, 9 on a sustained tone; 128 for any RR on a percussive one; 512 for any with SUS. The manual says SUS gives 5 [D (III-1-6)] |
| A key-on takes the slot to silence at rate 48, the damp, before the attack | [M `m11.py`]: a retrigger from full level falls over about 470 samples before it rises |
| From level 124 a slot sounds nothing | [M `m13.py`]: a release at rate 16 goes silent 127,806 samples after key-off, 124.8 steps of 1,024 |

### 3.6 The LFOs

| Rule | Evidence |
|---|---|
| The tremolo: a triangle of 0-13 steps of 0.375 dB (4.875 dB), a counter of 0-104 stepped every 64 samples, each end held twice, the level its value over 8: 13,440 samples a cycle, 3.70 Hz | [M `m5.py`]: plateaus of 512 samples, 1,024 at 0 and 128 at 13, a cycle of 186 windows of 64 output samples. The manual's 3.7 Hz [D (III-1-2)] |
| The vibrato: eight steps of 1,024 samples, the F-number moved by 0, half, full, half, 0 and the same below, full being its top three bits in half-units and half being full's half, truncated | [M `m6.py`]: F-numbers `$100`, `$180`, `$1FF` move by ±1, ±1.5 and ±3.5 at the full steps and ±1, ±1.5 and ±1.5 at the half. 6.07 Hz; the manual's 6.4 Hz at 3.6 MHz [D (III-1-2)] is 6.25 Hz scaled, near |

### 3.7 The order within a sample

| Rule | Evidence |
|---|---|
| The carrier takes the modulator's output of the sample before | [M `harmcmp.py`]: with MUL 1 on both slots the folded fundamental depends on the two's alignment; at TL 30 the box gives 143 against Endou's 48 with the newest output and 137 with the one before, every other harmonic within 1% either way |
| The envelopes and the phases step after the outputs | [A] |

### 3.8 The rhythm

| Rule | Evidence |
|---|---|
| Rhythm mode (`$0E` bit 5) gives channels 7-9 to the drums: the bass drum both slots of channel 7, the high hat and snare drum channel 8's, the tom-tom and top cymbal channel 9's; `$0E`'s bits key them, beside the channels' own KEY | [D (III-1-7), Table III-9, (II-5)] |
| The high hat's and tom-tom's slots take their volumes from `$37` and `$38` as a carrier does | [D (II-7)'s rhythm map] |
| The rhythm sounds twice a sample | [D (III-4)]; the bass drum and tom-tom against melody voices on the box fit better at twice than at once, by 11 and 18 dB [M `relres_r.py`] |
| The bass drum is a two-slot voice; the tom-tom a plain sine on its slot | [D (V-4): "the bass drum ... by the same procedure"]; the tom-tom [A] |
| The high hat, snare drum and top cymbal play phases made of a 23-bit noise register and of bits of the high hat's and top cymbal's slots' phases | [D (V-4): "a noise oscillator obtained by composing a white noise generator with several frequencies"]; the register and the bits [A], as the OPL2 is described. Their spectra on the box and on Endou, a 516 Hz band at a time, peak in the same bands (§4.3) |

## 4. The instrument ROM, measured

The manual names the fifteen instruments and the five drums and gives none of their parameters [D (II-7)], and no
document read states them. The plan's rule — the table taken from a document that states it, each value pinned by
the referee (`Endou_Plan.md` §5.4) — could not be met; the tester's choice of a document is a question (`Endou_Native.md`
§16.15). The table in `src/opll/tables.rs` is instead **measured on the box** [M], each instrument as the original
instrument whose recording comes nearest the instrument's own:

1. A program plays four notes (437 Hz for 25 frames; 4.9 kHz; 86 Hz; 485 Hz with SUS) on instrument N, and the same
   program with the tone registers set to a candidate and instrument 0; the two differ only in data bytes, so their
   timing is the same.
2. A start: the carrier's envelope from the recording's RMS (phase modulation leaves a sine's amplitude alone), with
   the candidates' modulators silent; the multiples, modulation and feedback from the model's steady tones fitted to
   the box's harmonics.
3. Descent on the box's recordings, field by field and in pairs, first on the distance of their spectrograms, then on
   the raw difference, the sum of squares of the two recordings' difference, which is 0 exactly when the chip's outputs
   were equal.

The residual is that raw difference over the instrument's own energy. 0 (−∞ dB) is the instrument itself. The descent
was run three times over, the later passes from the earlier results and from the next timbres the model proposed, and
each instrument keeps its lowest residual [M 2026-10-20].

| # | Instrument | `$00`-`$07` | Residual |
|---|---|---|---|
| 1 | Violin | `71 41 1E 17 D0 78 00 10` | **0, exact** |
| 2 | Guitar | `33 41 22 0D A3 F7 50 13` | −23.9 dB |
| 3 | Piano | `33 01 99 00 C2 74 20 89` | −6.3 dB |
| 4 | Flute | `31 41 2A 07 90 64 00 20` | −27.6 dB |
| 5 | Clarinet | `32 21 1E 06 E0 76 00 28` | **0, exact** |
| 6 | Oboe | `31 22 16 05 E0 71 30 B8` | **0, exact** |
| 7 | Trumpet | `31 41 1E 07 70 80 00 11` | −13.0 dB |
| 8 | Organ | `23 21 2D 14 A0 70 00 37` | **0, exact** |
| 9 | Horn | `41 61 1C 06 62 65 16 17` | −12.6 dB |
| 10 | Synthesizer | `41 E1 0D 18 84 F0 26 07` | −6.6 dB |
| 11 | Harpsichord | `33 01 87 11 F0 E0 00 04` | −24.5 dB |
| 12 | Vibraphone | `37 C1 2C 07 F2 F8 40 12` | −24.8 dB |
| 13 | Synthesizer Bass | `61 50 0C 05 C2 F5 10 42` | **0, exact** |
| 14 | Acoustic Bass | `01 21 56 03 D2 92 16 17` | −18.0 dB |
| 15 | Electric Guitar | `61 41 89 03 F1 74 10 45` | −10.0 dB |

**Five instruments are exact** — 1, 5, 6, 8 and 13: the original instrument loaded with the bytes above gives recordings
equal to the instrument's, sample for sample, in every note of the four, and `opll::tests::every_instrument_and_drum_sounds_and_the_exact_ones_are_their_registers`
holds Endou to the same identity. The other ten are the nearest the descents found, from −27.6 dB (the flute) to
−6.3 dB (the piano); a residual that is not 0 means at least one field differs, the box saying no more. The referee
would finish them (§7).

### 4.3 The rhythm's patches

| Channel | Patch | How |
|---|---|---|
| 7 (bass drum) | `DE 11 1E 08 C2 D7 0B 6C` | against a original-instrument voice on channel 7 in melody mode, scaled by two: −17.3 dB |
| 8 (high hat, snare drum) | `11 4A C0 88 66 78 A9 EC` | the slots' envelopes fitted on Endou's model to the box's RMS, their multiples to its spectra (§3.8) |
| 9 (tom-tom, top cymbal) | `05 C1 00 00 F8 7F 59 45` | the tom-tom against a carrier's sine on channel 9 in melody mode, scaled by two: −24.1 dB; the top cymbal as the high hat |

## 5. How it was measured

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

`src/opll/tests.rs`, each rule of §3 a test: the tables against their formulas; the operator's full scale and half
wave; a tone's frequency; the phase step's truncation; the key scale's codes; the decay that does not step at its
sustain level; the carrier taking the modulator's output of the sample before; silence from level 124; the LFOs'
shapes and periods; the release rates; the damp and the phase's restart; the rhythm's keys; an unnamed address; a state
that continues the chip exactly and one no chip could be in refused.

## 7. The referee bench, for the tester

`EmuSen.WiseMan/Reference/opll/build-opll.sh` checks the tarball's SHA-256, extracts `opll.c`, `opll.h` and `LICENSE`
into the probe's cache without opening them, and compiles `tb_opll.c` against the header with `opll.c` beside it.
`tb_opll.c` finds how many calls of `OPLL_Clock` make a sample from a sine of known frequency, then reads the voices
`examples/opllref.rs voices` prints (sines, every multiple, modulation, feedback, half waves, attacks, decays, releases,
the key scales, the key-scaled rate, the tremolo and the vibrato, the fifteen instruments and the five drums), drives
the chip through `OPLL_Write` and `OPLL_Clock`, and prints each voice's sums a sample. `opllref.rs compare` runs Endou's
chip on the same writes and prints, a voice a line, the lag and scale that fit best and how many samples are then
equal. Built and run by the tester's leave, it gives the sample-for-sample gate its numbers, D-2 its 512 entries
moved by one, the instrument ROM its exact check, and the DAC's small signals (D-19) their rule.

## 8. Open

- The sample-for-sample gate and D-2, for want of the referee (§0, §7).
- The instruments whose residual is not 0 (§4), and the rhythm's patches and noise (§4.3), best fits.
- The attack's last steps (§3.5), the rates past 51, a write's latency, the DAC's small signals (D-19).
- Port `$F2`'s counter and decoding (D-17), argued.
