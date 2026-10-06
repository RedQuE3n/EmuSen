# Nephrite_OperatorTables — the YM2612 operator's log-sine and exponent ROMs

*Written 2026-10-05, at stage 5, before the FM operators are built.* This page records where the two constant tables in
`src/fm_tables.rs` come from, what fixes each bit of them, and what the documents leave open. It was written under the
clean-room protocol of `Nephrite_Plan.md` §1.3: hardware documents and prose write-ups only, no emulator or RTL source.
The provenance list of §6 names every page opened, including those abandoned.

The page keeps the plan's registers: **documented** is a statement a cited source makes; **measured** is a check run
here, with the tool named; **argued** is reasoning with no document or measurement behind it.

---

## 1. What the tables are

A Yamaha FM operator of this generation computes no products. It holds the sine in the logarithmic domain, adds the
envelope's and the total level's attenuation to it there, and returns to the linear domain through an exponent table
whose output is a floating-point number: a significand from the table and an exponent from the attenuation's integer
part. Two ROMs make this possible:

- **LOGSIN**, 256 entries: one quarter of a sine wave, each entry an attenuation, −log₂ of the sine, in 4.8 fixed point:
  4 integer and 8 fractional bits, 12 in all (the largest entry, 2137, is 8.35). One unit is 1/256 of a factor of two,
  20·log₁₀2 / 256 ≈ 0.0235 dB.
- **EXP**, 256 entries: the fractional part of a power of two, 10 bits per entry, with the leading one not stored.

The source of both tables, and of the statement that they are the whole of the operator's arithmetic, is the die
analysis of the YM3812 (OPL2) in [S1]. Whether the YM2612 (OPN2) carries the same ROMs is the subject of §4, and is
**not settled by any clean document found**.

## 2. Sources cited

- **[S1]** Matthew Gambrell and Olli Niemitalo, *OPLx decapsulated*, dated 2008/04/20 in its header (the year is 2008,
  not 2018 as it is sometimes cited). Google Docs,
  <https://docs.google.com/document/d/18IGx18NQY_Q1PJVZ-bHywao9bhsDoAqoIn1rIm42nwo/edit>, read through its plain-text
  export (`…/export?format=txt`). A YM3812 and a YMF262 were decapsulated and the YM3812's two ROMs read bit by bit.
  Sections used: the opening account of the ROMs, *Exponential table*, *Log-sin table*, **Table I** (all 256 exponent
  values) and **Table II** (all 256 log-sin values). Archived at `~/.cache/emusen/probe/nephrite/docs/oplx/` with its
  SHA-256 in `SHA256SUMS` and an entry in `DOCS.txt` §H.
- **[S2]** Olli Niemitalo, *Adlib / OPL2 / YM3812*, yehar.com, <https://yehar.com/blog/?p=665> and its second comment
  page <https://yehar.com/blog/?cpage=1&p=665>. The post that announces [S1]; its reader comments discuss how the
  exponent table is indexed (a commenter signing "Jani", 2016: zero from the log-sine is the largest output, which the
  exponent table gives at index 255).
- **[S3]** Ken Shirriff, *Yamaha DX7 reverse-engineering, part III: Inside the log-sine ROM*, righto.com, December 2021,
  <http://www.righto.com/2021/12/yamaha-dx7-reverse-engineering-part-iii.html>. The DX7's own log-sine ROM, read from
  its die; used only for its account of *why* a quarter-wave table is sampled half a step in.
- **[S4]** Yamaha, *YM2608 (OPNA) Application Manual*, English translation, §2-5-3 "Output control circuit" and Table 2-9
  "Weight putting of TL each bit". Local copy `~/.cache/emusen/probe/nephrite/docs/yamaha/YM2608J_Translated.PDF`
  (SHA-256 `570b6210…49ee81`); the Japanese original is beside it. The nearest datasheet to the OPN2 (`Nephrite_Plan.md`
  §2.1).
- **[S5]** Wikipedia, *Yamaha OPL*, section "Internal operation", <https://en.wikipedia.org/wiki/Yamaha_OPL>. A secondary
  restatement of [S1]'s two formulas; cited only as such.

## 3. The formulas and the two offsets

### 3.1 LOGSIN: the sine is sampled at the middle of each step

*Documented* in [S1], *Log-sin table*:

    LOGSIN[x] = round( −log₂( sin( (x + ½) · π / 512 ) ) · 256 ),   x = 0 … 255

[S1] states this formula "re-created the contents of the ROM tables exactly", and prints all 256 values as Table II.
Entry x samples the quarter wave at (x + ½)/256 of its length, so entry 0 is 2137, not infinite, and entry 255 is 0.

**What fixes the half step.** The document fixes it twice over: by the formula, and by Table II as data. *Measured*
2026-10-05 (Python 3, f64): the formula reproduces Table II in all 256 entries; with no offset (x · π/512) 186 of the
255 defined entries differ, and with a whole step (x + 1) 196 of 256 differ. No other placement is compatible with the
printed table.

**Why the half step is the right design** (argued from [S1], stated outright for the DX7 in [S3]). [S1] says the rest of
the wave "can be constructed by flipping all the bits of x and/or by changing the sign of the samples". Complementing an
8-bit x gives 255 − x, and 255 − x + ½ = 256 − (x + ½): with the half step, the complemented index samples exactly the
mirror image of the original point about the quarter's end, so the descending quarter is the ascending one reversed with
no sample repeated. Without the half step the mirror would be off by one and the peak and the zero crossing would each be
sampled twice per cycle. [S3] gives this reason for the DX7's ROM ("the input to the ROM is incremented by half a bit.
This avoids duplication of the 0 value of the waveform when the quarter-wave is mirrored").

**Folding a 10-bit phase** (argued from [S1]'s sentence above). The low 8 bits index the table, complemented when the
phase is in the second or fourth quarter (bit 8 set); bit 9 gives the sample's sign. [S1] names no bit numbers; this is
the only assignment of a 10-bit phase's top two bits that yields the four quarters in order.

### 3.2 EXP: no offset in the ROM, one whole step when read as 2^−x

*Documented* in [S1], *Exponential table*:

    EXP[x] = round( (2^(x / 256) − 1) · 1024 ),   x = 0 … 255

with all 256 values printed as Table I (0, 3, 6, 8, … 1013, 1018). The values fit in 10 bits. [S1] states how the table
is used: it "is read at the position given by the 8 LSB's of the input. The value + 1024 (the hidden bit) is then the
significand of the floating point output and the yet unused MSB's of the input are the exponent". **The leading one is
therefore implicit: the ROM stores only the fraction, and 1024 is added on reading.** `EXP` in `fm_tables.rs` holds the
ROM's stored values, without the hidden bit.

**What fixes the offset.** The ROM's own index has no offset: entry x is 2^(x/256), sampled at the step's start. Again
the document fixes it twice. *Measured* 2026-10-05: the formula reproduces Table I in all 256 entries; a half-step
variant, 2^((x + ½)/256), differs in all 256.

**The offset when the table is read as 2^−x.** The ROM ascends, but an attenuation must produce a descending output.
[S2]'s comment records the reading: an attenuation of zero is the largest output, found at index 255, so the index is the
bitwise complement of the attenuation's fractional byte f. Then

    EXP[255 − f] + 1024 = round( 1024 · 2^((255 − f)/256) ) = round( 2048 · 2^(−(f + 1)/256) )

so, read through the complement, the table is 2^−x with the index offset by **one whole step**, (f + 1), and scaled to
an 11-bit significand. The integer part of the attenuation then shifts that significand right. A test in
`fm_tables.rs` holds this identity for every f. How many bits the OPN2's operator keeps after the shift, and where it
rounds, belong to the operator's own page when it is built, not to the ROM.

### 3.3 How the constants are held

The tables are written out as literal constants so every machine has the same bits. A unit test regenerates both from
the formulas above in f64 and requires equality; another checks entries against Tables I and II as printed; another
checks their widths (12 and 10 bits) and monotonicity. *Measured* 2026-10-05: no entry of either formula lies within
0.0003 (LOGSIN) or 0.0007 (EXP) of a rounding tie, so the regeneration does not depend on the last bits of a libm's
`sin`, `log2` or `exp2`.

First and last entries, for reference:

| Table | 0–7 | 252–255 |
|---|---|---|
| LOGSIN | 2137, 1731, 1543, 1419, 1326, 1252, 1190, 1137 | 0, 0, 0, 0 |
| EXP | 0, 3, 6, 8, 11, 14, 17, 20 | 1002, 1007, 1013, 1018 |

## 4. Does the OPN2 share the OPL's tables?

**No clean source found states that it does, and none states that it does not.** The tables in `fm_tables.rs` are the
YM3812's; their use in the YM2612 is an assumption, recorded here as open rather than presented as settled.

What the clean sources do say:

- [S1] reads the YM3812's ROMs and remarks that the YMF262 has no extra ROMs. It says nothing of the OPN family.
- [S5] presents the tables under the OPL only.
- [S3] compares the DX7's log-sine ROM with the OPL3's and not with the OPN's. (It also describes the OPL3's entries as
  8-bit; [S1]'s Table II needs 12 bits, and [S1], being the primary reading, governs.)
- [S4] gives the OPNA's total level as seven bits weighted 48, 24, 12, 6, 3, 1.5 and 0.75 dB. A step of 0.75 dB is 31.9
  units of 0.0235 dB, so one TL step is 32 units of a 4.8 log₂ attenuation, the same step the OPL's total level is
  described with in [S2]'s comments. This shows the OPNA's attenuation arithmetic is *compatible* with a 4.8 log₂ scale.
  It does not show that the sine ROM's contents, its sampling, or the exponent ROM's width are the same.

Three statements bearing on the question reached this work only as text inside search-engine result summaries, from
pages the protocol excludes; none was opened and none is relied on: a summary of SpritesMind topic t=386, page 21,
saying that the YM2612's "sin and pow tables have 256 entries"; a summary of a source-file header of an OPN2 emulator
crediting [S1] for "OPL2 ROMs"; and a summary of the SMS Power YM2413 notes saying the ROMs are identical between the
OPLL, the OPL2 and the OPL3 (the OPLL is not an OPN either). They are listed in §6 so the exposure can be audited.

**How it is to be settled.** By the order of recourse in `Nephrite_Plan.md` §1.3: a test program written for the purpose
(a single carrier at fixed attenuations, its samples read at the DAC; whether nine bits resolve the question is itself
to be shown), then the MiSTer RTL as referee in a separate dispute
step, logged in `Nephrite_Disputes.md`. Until then the OPL tables stand, and any disagreement found is to be recorded
here rather than absorbed into the constants.

## 5. What this page does not cover

The operator's phase generator, the envelope's arithmetic, the feedback path and the accumulator's width. Each is a
later step of stage 5 with its own sources.

## 6. Provenance

Every page opened for this work, in the order opened, with what was taken. "Opened" includes a fetch whose content was
only summarised by a reading tool.

| # | URL or file | Kind | Code on it? | Taken |
|---|---|---|---|---|
| 1 | <https://docs.google.com/document/d/18IGx18NQY_Q1PJVZ-bHywao9bhsDoAqoIn1rIm42nwo/export?format=txt> | [S1], plain-text export | No | Both formulas, Tables I and II, the hidden bit |
| 2 | <https://yehar.com/blog/?p=665> | [S2] | No program code; one reader comment writes the gain as a one-line shift expression | The OPL's TL step of 32 units (Jani, 2010) |
| 3 | <https://en.wikipedia.org/wiki/Yamaha_YM2612> | Encyclopaedia | No | Nothing relevant |
| 4 | <https://en.wikipedia.org/wiki/Yamaha_OPL> | [S5] | No | The formulas, as a restatement |
| 5 | <https://www.vgmpf.com/Wiki/index.php?title=YM2612> | Wiki | No | Nothing relevant |
| 6 | <https://gendev.spritesmind.net/forum/viewtopic.php?t=2191> | Forum, "Decapping more Genesis chips" | No | Nothing relevant |
| 7 | <https://nukeykt.retrohost.net/> | Index of projects | No (links only) | Nothing; the linked repositories were not opened |
| 8 | <https://news.ycombinator.com/item?id=43473195> | Discussion thread | No | Nothing relevant |
| 9 | <http://www.righto.com/2021/12/yamaha-dx7-reverse-engineering-part-iii.html> | [S3] | No | The half-step rationale |
| 10 | <https://www.smspower.org/Development/YM2413ReverseEngineeringNotes2015-04-09> | Reverse-engineering notes | **Yes** (C++ blocks) | **Abandoned** on finding code; nothing read beyond the check, nothing used |
| 11 | <https://gist.github.com/bryc/e85315f758ff3eced19d2d4fdeef01c5> | Notes on Yamaha chips | No | Nothing relevant |
| 12 | <https://siliconpr0n.org/archive/doku.php?id=vendor%3Ayamaha%3Aopl2> | Die-photo archive | — | Refused (HTTP 403) |
| 13 | <https://en.wikipedia.org/wiki/Yamaha_YM2151> | Encyclopaedia | No | Nothing relevant |
| 14 | <https://en.wikipedia.org/wiki/Yamaha_YM2203> | Encyclopaedia | No | Nothing relevant |
| 15 | <http://www.larwe.com/technical/chip_ym2203.html> | Translated datasheet | — | Not found (HTTP 404) |
| 16 | `~/.cache/emusen/probe/nephrite/docs/yamaha/` (YM2608 manual in Japanese and English, YM2608 data sheet, YM3438 data sheet), via `pdftotext` | [S4] and Yamaha datasheets | No | TL weights (§2-5-3, Table 2-9); no ROM description in any of them |
| 17 | `~/.cache/emusen/probe/nephrite/docs/plutiedev/ym2612-*`, `railgun/YM2612.wiki` (searched for the tables' keywords only) | Saved documents | No | No mention of the ROMs |
| 18 | <https://www.sega-16forums.com/forum/general-discussion/insert-coin/11989-yamaha-chips-ym2612-vs-opl2/page3> | Forum | — | Refused (HTTP 403) |
| 19 | <https://www.copetti.org/writings/consoles/mega-drive-genesis/> | Architecture write-up | No emulator code (citation and example snippets only) | Nothing relevant |
| 20 | <https://yehar.com/blog/?cpage=1&p=665> | [S2], comment page | No | The exponent table's complemented reading (Jani, 2016) |
| 21 | <https://gendev.spritesmind.net/forum/viewtopic.php?t=2678> | Forum, "Maybe some new technical information about the Yamaha YM2612" | No | Nothing relevant |

Seen only as search-result text, never opened (§4): SpritesMind topic t=386 (pages 1, 11 and 21 appeared as results;
the topic is excluded for this work in its entirety); a GitHub source header of an OPN2 emulator; the SMS Power YM2413
notes' summary; a blog series on writing a YM2612 emulator (jsgroth.dev, "Emulating the YM2612"), which carries an
emulator's code and was therefore not opened. Nothing from any of them is used. The local SpritesMind archive's t=386
pages were not opened either.
