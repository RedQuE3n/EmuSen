# Mars — textures: texture memory, loads and point sampling

*Landed 2026-09-17. Phase D's sixth slice: the display processor's own texture memory, the two
commands that fill it from RDRAM, and a point-sampled texel read back from it in the one-cycle mode,
for textured triangles and texture rectangles. The code is `Rdp/Rdp.TextureMemory.cs` (the tiles and
the loads), `Rdp/Rdp.Textures.cs` (coordinates and the sample) and `Rdp/Rdp.OneCycle.cs` (the pixel);
the grading is `MarsRdpDifferentialTests` (`Mars_RdpDifferential.md`).*

*Filtering, palette lookup, the level of detail and the two-cycle and copy modes are later slices
(§8). §7 is the part to read before trusting a number here: of the first 177 texture cases only 86 could
be used as generated, the two references disagree on six behaviours, and two defects in the test's own
machinery produced failures that had nothing to do with Mars.*

---

## 0. Where each rule came from

**Every rule is the reference's, graded**, as in `Mars_RdpCoverage.md` §0: read from angrylion for
mechanism, written in Mars's own structure, run against angrylion with parallel-rdp as the cross-check.
`Mars_Documentation.md` §5 lists texture memory's edge cases among the things documented nowhere; the
formats and the command fields are Tier 1, and everything below the level of "a texel is fetched from
texture memory" is established by this comparison.

**The scope is point sampling in the one-cycle mode.** Both mode bits that turn on filtering and
palette lookup stay clear in every case, as do the level-of-detail bits.

## 1. The texture image and the tiles

Five commands set texture state.

- **The texture image** names an RDRAM address, a width in texels and a pixel size. Its format field
  is carried by the command and read by nothing here.
- **A tile** — one of eight — is a format, a size, a *line* (how many 64-bit groups of texture memory
  one texel row takes), a starting place in texture memory in those groups, a palette number, and for
  each of s and t a clamp bit, a mirror bit, a four-bit mask and a four-bit shift.
- **The tile size** is the tile's corners in quarter texels, `SL, TL` and `SH, TH`. The two load
  commands set the same four fields, so a load also resizes the tile it loads.

Derived per axis: a tile **clamps** if its clamp bit is set *or its mask is zero*; its **mirror bit**
is its mask, at most ten; its **clamp limit** is the difference of its corners' whole parts, modulo
1,024.

## 2. Texture memory

**Four kilobytes inside the display processor**, addressed as 4,096 bytes or 2,048 sixteen-bit words.
Mars keeps it in the console's byte order. The reference keeps it as host-order words, so on a
little-endian host the console's byte *i* is the reference's byte *i* with its low two bits flipped;
that is what made it possible to grade texture memory byte for byte (§7.1).

### 2.1 Groups, banks and odd rows

A tile's texel row *t* starts at group `line × t + start`, and every index into texture memory wraps
at its size. Fetching takes `t`'s low eight bits before multiplying, which differs from loading only for
rows past 255, and no case has one. Each group is eight bytes, or four words.

**Not in Mars: the reference's nine-bit wrap of `line × t` when loading.** A load's word index is kept to
eleven bits, which already wraps the product at 512 groups; a breakage removing the separate wrap
survived every case, and Mars does not carry it.

**Odd texel rows store each group with its halves swapped.** Reading, that is a byte index with bit 2
flipped or a word index with bit 1 flipped; loading, it is the same swap applied to the four words'
banks (§3.2).

### 2.2 Formats that use both halves

**32-bit RGBA and every YUV size use both halves of texture memory**: a texel's red and green, or its
chroma, are in the lower half, and its blue and alpha, or its luma, at the same place in the upper
half. Every other format uses the whole four kilobytes as one space.

## 3. Loads

### 3.1 Rows and columns

**A tile load** walks texel rows `TL` to `TH` and columns `SL` to `SH`, whole texels in both. The image
pointer for a row is the image address plus the bytes of `width × row + SL` pixels. s starts at `SL`
and t at `TL`, t stepping by one texel per row.

A row counts only if one of its four quarter-texel sub-rows *k* has `TL ≤ k < (TH | 3)`. The only row
that can fail is `TL`'s, when `TL` equals `TH` and both fractions are three. It is then walked with a right
column of zero, so its length is `1 − SL` wrapped to twelve bits: one step from column zero, none from
column one, and about four thousand texels from any column further right. parallel-rdp loads the row as
if it were valid (§7.3).

**A block load** is one row, `TL` taken to ten bits, whose fields are whole texels rather than quarters.
Its `TH` field is not a corner but a slope: t steps by `TH / 2^11` texels per image step, the way a
block of texels at an angle would be read. Its left column is a **signed** twelve-bit number when it moves
the image pointer, but not when it becomes s; parallel-rdp takes it as unsigned in both (§7.3).

### 3.2 Eight bytes to four words

**Each step reads the eight image bytes at the pointer** — through the two 32-bit RDRAM words holding
them and the following pair — and stores them as four words. The words' indices wrap at twenty-two bits,
so a read past sixteen megabytes continues from address zero, which parallel-rdp does not do (§7.3); a
word past the end of RDRAM but inside sixteen megabytes reads zero, which both do.
The pointer then advances eight bytes, and s by as many texels as eight bytes hold at the image's size:
eight, four or two. **A 4-bit image loads nothing**, and in angrylion marks the pipeline crashed
(`Mars_RdpDifferential.md` §5).

Where the four words go:

- The texel's word index is s halved for 8-bit tiles and every YUV tile, s itself for 16- and 32-bit
  tiles, and s quartered for 4-bit tiles; it is added to the row's group times four, and **bit 1 of the
  result is cleared**.
- The four consecutive words from there are **sorted into banks by their low two bits**, after odd rows
  flip bit 1 of each.
- **Formats using one space** (§2.2) put the image's first word in bank 0 and its last in bank 3, or on
  odd rows the halves the other way round; in the upper two kilobytes if the first index was there.
- **YUV** takes the image's even bytes as one 32-bit value and its odd bytes as another; **32-bit RGBA**
  takes its first and third words, and its second and fourth. The first value goes to two banks in the
  lower half and the second to the same banks in the upper half — banks 0 and 1, or 2 and 3 when the
  texel's bit 1 and the row's oddness differ.

### 3.3 Palette loads

~~**The palette load is taken whole and does nothing.** It shares the pipeline, and the two references
disagreed about one — in texture memory, on a palette loaded into a tile no command had set — which was
not characterised. It is left to the slice that builds palette lookup, where what it loads can be seen.~~
Built in the next slice (`Mars_RdpFiltering.md` §4.2), where the disagreement is taken up again (§5.2 there).

## 4. Texture coordinates

### 4.1 Where a primitive carries them

**Triangles** carry s, t and w in the eight words between shade and depth, packed as shade is
(`Mars_RdpDepth.md` §1.1), and the walker records and steps them as it does shade — the same row
offset, the same edge fraction, the same low bits cleared. Per pixel they step by their x step with the
low five bits cleared.

**Texture rectangles** are rectangles (`Mars_RdpTriangles.md` §3) with a second word: s and t in 10.5
fixed point and their steps in 5.10. s and t become attribute values sixteen bits up; the steps become
steps eleven bits up, **across x for s and down the edge and y for t — or, in the flipped command, the
other way round.** w is zero. A texture rectangle in the fill cycle is a fill.

### 4.2 Division by w, and the clamp to sixteen bits

The walker's values are 16.16; the sixteen whole bits are what is divided.

**Without perspective**, s and t are their sixteen bits sign-extended to seventeen.

**With perspective**, s and t are multiplied by a reciprocal of w and scaled back:

- **A w at or below zero** (as sixteen signed bits) flags both results as overflowing.
- **The reciprocal** comes from w's low fifteen bits. w is normalised by a shift of up to fourteen; the
  normalised value's top six bits choose one of 64 points and its next eight interpolate linearly towards
  the next point.
- **The product** is shifted down by `13 − shift`, or doubled when the shift is fourteen, and seventeen
  bits are kept. If the bits of the product above that range are neither all clear nor all set, the result
  is flagged: over if bit 29 of the product is clear, under if it is set — bit 29 of the product **as
  shifted**, except at the largest shift.

**The 64 points are `2^20 / (64 + i)` rounded — except the seventh**, which is `0x3A83` where rounding
gives `0x3A84`. The slopes are the differences between successive points, with `2^20 / 128` after the
last. Mars generates both tables from that rule instead of copying the reference's 128 constants, and the
rule was checked against those constants entry by entry. It is the position `Mars_References.md` §2 took
on the RSP's reciprocal ROM, for the reason `Mars_Documentation.md` §3.2 gives: derive rather than lift.
Here the derivation has an exception, and nothing says whether it is the hardware's or a slip carried
forward.

**Then both coordinates are clamped to sixteen bits**: an overflow flag gives `0x7FFF`, an underflow flag
`0x8000`; otherwise bit 15 set with bit 16 clear gives `0x7FFF`, bit 16 set with bit 15 clear gives
`0x8000`, and anything else keeps the low sixteen bits.

## 5. A point sample

### 5.1 The fetch, by format

The row's group is `line × (t & 0xFF) + start`. Within it, a 4-bit texel is at byte `group × 8 + s/2` —
the high nibble for an even s, the low for an odd one — an 8-bit texel at byte `group × 8 + s`, and a
16-bit texel at word `group × 4 + s`. 4-bit YUV is the exception: it is read a whole byte at the 8-bit
index. Odd rows flip bits as §2.1 says.

| Format | 4-bit | 8-bit | 16-bit | 32-bit |
|---|---|---|---|---|
| RGBA | nibble repeated, all four | byte, all four | 5-5-5-1, each five widened by its top three bits; alpha all or nothing | red-green word in the lower half, blue-alpha in the upper |
| YUV | the byte's top nibble repeated; luma and alpha it, both chroma it less 128 | the byte the same way | chroma word at half the byte index, lower half, each byte less 128; luma and alpha the byte at the byte index, upper half | chroma as 16-bit; for odd s luma as 16-bit, for even s luma from the upper-half word's high byte and alpha from its middle two nibbles, the upper moved below the lower |
| CI | palette number, then the nibble | byte, all four | word: high byte red and blue, low green and alpha | as 16-bit |
| IA | nibble's top three bits widened to eight; alpha its low bit, all or nothing | high nibble intensity, low nibble alpha, each repeated | high byte intensity, low alpha | as CI |
| I | as RGBA | as RGBA | as CI | as CI |

**Formats 5 to 7 read as intensity.** That is angrylion's choice and parallel-rdp's is different (§7.3).

### 5.2 Shift, clamp, mask and mirror

In order, for each of s and t:

1. **Shift.** Below eleven, the coordinate's sixteen bits are sign-extended and shifted right; from eleven,
   shifted left by `16 − shift` and then sign-extended.
2. **Beyond** is decided here: whether the shifted coordinate, in quarter texels, is at or past the tile's
   `SH` — the absolute corner, not the relative one.
3. **Relative.** `SL` is subtracted.
4. **Clamp**, if the tile clamps: beyond gives the clamp limit, a negative coordinate gives zero, and
   anything else its whole texels. A tile that does not clamp just takes whole texels.
5. **Mask**, if the mask is not zero: when mirroring and the mirror bit is set, the coordinate is
   complemented; then it keeps its low `mask` bits, and never more than ten.

### 5.3 The conversion when not filtering

**With the first cycle's bilinear bit clear, every texel is put through the YUV conversion**, whatever its
format:

- red is `B + (k0·G + 128) >> 8`,
- green is `B + (k1·R + k2·G + 128) >> 8`,
- blue is `B + (k3·R + 128) >> 8`,
- and alpha is `B`, each kept to nine bits.

Each constant is the convert command's nine-bit signed field, doubled, plus one. With the bit set there is
no conversion.

**Not in Mars: the reference's nine-bit cut of red and green with the bit set.** It matters only to YUV,
whose chroma can be negative, and the combiner reads every input through a nine-bit sign extension that
gives a negative value and its nine-bit cut the same result. A breakage removing the cut survived every
case, and Mars does not make it.

## 6. The one-cycle pixel

**Texel 0** is sampled at the pixel's own s, t and w. **Texel 1**, in this mode, is the *next* pixel's
texel 0: one step further along the span. The exception is a span's last pixel, when the span's last
column less its first, plus the columns clipped from its start, exceeds seven and the next row is one this
primitive draws; then texel 1 is sampled at the next row's recorded starting values.

The reference computes each pixel's texel 0 by keeping the previous pixel's texel 1. With the level of
detail off those are the same fetch, so Mars fetches directly; ~~**the level-of-detail slice must revisit
this**, because there the two differ by the tile each is taken from.~~ Revisited (`Mars_RdpLod.md` §2): the
prediction was wrong, because texel 1's tile is measured for the next pixel and is the tile that pixel's
texel 0 uses, so the fetches stay the same.

**Texels are fetched only when the second cycle's selectors read them**: texel 0 or 1 as a colour input to
any of the four, texel 0 or 1 alpha as the multiplier, or either as an alpha input. A primitive whose
combiner reads neither fetches nothing, which is invisible. The primitive's tile is its command's: bits
48–50 of a triangle, the tile field of a rectangle.

An untextured triangle whose combiner reads a texel samples at zero coordinates from its command's tile.
That follows from the code; it is not graded (§8).

## 7. What the differential says

### 7.1 Texture memory is graded

The plan for this slice said texture memory would be left to the reference-to-reference check and only
what reaches RDRAM graded, because angrylion's texture memory is not in the console's byte order. That was
retired once the order turned out to be a fixed permutation (§2). `rdp-reference` now writes texture memory
at every sync, and every case compares it byte for byte along with RDRAM and hidden RDRAM.

### 7.2 The first cross-check

**Of the first 177 texture cases generated, 86 could be used.** Run one at a time through the validator:

- **77 were refused by parallel-rdp**, which declines some uploads outright: a 4-bit texture image (38),
  a 32-bit tile in any format but RGBA (26), and a YUV tile at any size but 16 bits (13). None of those is a
  disagreement about the answer. The 4-bit image is refused by angrylion too.
- **14 drew differently.** Seven used formats 5 to 7, four read texel 1 with the two cycles' bilinear bits
  different, and one loaded a 32-bit tile past the first half of texture memory — the first three disputes
  of §7.3. One was the palette load (§3.3). **One was the test's own fault**: its combine mode asked for an
  add input of 8 in a three-bit field, which spilled into the next field and turned the case into one
  reading the one-cycle combined input — already a recorded dispute (`Mars_RdpCoverage.md` §6.1). The
  combine builder now refuses a selector wider than its field; no earlier case had one.

The random cases now keep to what both references model — formats 0 to 4, YUV only at 16 bits, 32 bits
only as RGBA and within half of texture memory, both cycles' bilinear bits equal, no palette loads — and
each excluded behaviour but the palette load has a named case. A YUV tile running into the second half of
texture memory agrees, so that restriction is on 32-bit tiles only. **parallel-rdp agrees with angrylion on
every cross-checked case.**

### 7.3 Six disputes

Mars follows angrylion in each, as the grader; none is known to be the hardware's answer. The first three
were found in random cases; the last three in cases written to catch breakages that had survived (§7.5).

- **Formats 5 to 7.** angrylion reads them as intensity; parallel-rdp samples nothing, and draws black with
  zero alpha.
- **A 32-bit tile reaching the second half of texture memory.** angrylion folds its rows back into the first
  half, so later rows overwrite earlier ones; parallel-rdp does not.
- **Texel 1 in the one-cycle mode.** Both take the next pixel's texel. angrylion filters it by the first
  cycle's settings and parallel-rdp by the second's, so they differ whenever the two bilinear bits do.
- **A tile load whose only row has no sub-row inside it** (§3.1). angrylion loads one step; parallel-rdp the
  whole row.
- **A block load from a column past 2,047** (§3.1). angrylion moves the image pointer back by the column taken
  as signed; parallel-rdp moves it forward.
- **An image read past sixteen megabytes** (§3.2). angrylion continues from address zero; parallel-rdp reads
  zero.

The last three are each the reference's arithmetic at a boundary — a length wrapped to twelve bits, a sign
extension, an index cut to twenty-two bits — and each is plausible hardware and plausible accident. Nothing
here decides between them.

### 7.4 Five cases graded against angrylion alone

A load from a **4-bit image**, which both references refuse, ends the case at the load, since what a crashed
pipeline does next is not graded. **YUV tiles at 4, 8 and 32 bits** and a **32-bit intensity-alpha tile** are
drawn, and parallel-rdp skips their loads.

### 7.5 Against Mars

**203 cases**: fifty-three named and 150 random. The first thirty-five named cases are every format and size
both references accept, plus format 5; a 4-bit image, an 8-bit YUV tile and a 32-bit intensity-alpha tile; a
flipped rectangle; the conversion without filtering; tile shift, mask and mirror; a tile clamped against a
smaller size; a block load; a tile larger than texture memory; the two tiles reaching the second half; a second
tile at an offset; texel 1 across a rectangle, across a triangle's rows, and with the cycles' bits different; a
32-bit colour image; and affine, perspective, perspective-shaded-and-depth-tested and out-of-range triangles.

With the slice removed, **all 203 failed, and so did the tool-free test below; nothing else did.**

**The first run with the slice built failed every texture case, and none of it was Mars.** Every case reported
one page: the uploaded texture image. The reference writes only the pages drawing changed, and the test took
a page it did not write to be zero — sound until this slice added uploads (`Mars_RdpDifferential.md` §4.8).
With unwritten pages compared against the upload cache, **Mars matched angrylion on all 185 cases then
written.**

**The cases were checked for bite, and more than a third of the breakages survived at first.** Fifty-eight
breakages of the slice were run, and thirty-six were caught. Of the twenty-two that survived:

- **Two were rules no input can observe**, and they are no longer in Mars (§2.1, §5.3).
- **Twenty needed a case.** Eighteen cases were added, and the format 5 case moved from 16 bits to 4: at 16
  bits, reading formats 5 to 7 as intensity fetches the same word as the fallback an unknown format gets, so
  the rule could not show. The new cases cover the alpha bit of RGBA16, a palette number, 4- and 32-bit YUV,
  a clamp exactly at a tile's corner, a mirror bit past ten, the divider's flags, its largest shift and its
  seventh point, texel 1 read only through its alpha, the low five bits of a texture step, a rectangle's
  fractional steps, and loads from an odd column, from a row with no sub-row, from a block column past 2,047,
  and from past the end of RDRAM and of addressable memory. Three of them found the last three disputes of
  §7.3.
- **Three of the new cases did not catch their breakage at first.** The 32-bit YUV case never let texel alpha
  reach a pixel. The mirror case mirrored s, and the breakage was of t's rule; it now mirrors both, and also
  loads a tile that fills texture memory, a precaution not tested on its own. The seventh-point case chose a
  w whose fraction made the exception cancel — the lower point with the shallower slope and the higher point
  with the steeper slope rounded to the same reciprocal — and it now uses a w with no fraction.

With those, every breakage but the two unobservable rules is caught.

**A tool-free test** in `MarsRdpTests` loads an eight-by-eight RGBA16 tile from RDRAM and draws it through a
texture rectangle, and checks twelve of angrylion's pixels and the texture memory of the tile's first two
rows, the second with its halves swapped. It fails with the slice removed, and with either odd-row swap, the
five-bit widening or the flipped rectangle's steps broken. It does not see perspective, texel 1 or an image
pointer that is not a multiple of eight, none of which it has.

## 8. What is not here

- ~~**Filtering**: bilinear, the median, and the mid-texel rule.~~
- ~~**Palette lookup**, and the palette load (§3.3).~~ Both built in the next slice (`Mars_RdpFiltering.md`).
  The median was a wrong prediction: neither reference filters texels by a median — the only median in
  either is the video interface's divot filter — and the four-texel rule is the mid-texel average.
- ~~**The level of detail**: the fraction, mipmaps, detail and sharpen, and the tile choice they make.~~ Built
  two slices later (`Mars_RdpLod.md`).
- ~~**The two-cycle mode**, in which textured primitives still draw nothing.~~ Built four slices later
  (`Mars_RdpTwoCycle.md`), where the second cycle's texel, the swap and the conversion of one texel from
  another are §2. **The copy mode** is still not here.
- **Untextured primitives that read a texel** (§6), and texture rectangles in the fill cycle — both follow
  from rules graded elsewhere, and neither has a case.
- **Chroma key** and **noise**, as `Mars_RdpDepth.md` §7 left them.
