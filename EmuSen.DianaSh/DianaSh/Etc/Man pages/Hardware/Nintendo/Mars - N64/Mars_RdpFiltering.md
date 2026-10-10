# Mars — filtering and palettes: four texels, the bilinear filter and palette lookup

*Landed 2026-09-17. Phase D's seventh slice: the texel the one-cycle mode reads when filtering or a
palette asks for more than one texel, the three-point filter and its mid-texel exception, palette lookup,
and the palette load. The code is `Rdp/Rdp.Filter.cs` (the four texels, the filter and the palette),
`Rdp/Rdp.Textures.cs` (the sample's front end, shared with point sampling) and `Rdp/Rdp.TextureMemory.cs`
(the palette load); the grading is `MarsRdpDifferentialTests` (`Mars_RdpDifferential.md`).*

*This page continues `Mars_RdpTextures.md`, whose §2 (texture memory), §5.1 (the fetch by format) and
§5.2 (shift, clamp, mask) it assumes. §5 is the part to read before trusting a number here.*

---

## 0. Where each rule came from

**Every rule is the reference's, graded**, as in `Mars_RdpTextures.md` §0: read from angrylion for
mechanism, written in Mars's own structure, run against angrylion with parallel-rdp as the cross-check.
Where the two references disagree, Mars follows angrylion and the case is recorded as a dispute (§5.3).

**The scope is the one-cycle mode.** Four mode bits are new here: the palette enable (bit 47), the
palette's type (bit 46), the sample type (bit 45) and the mid-texel bit (bit 44), all alongside the
first cycle's bilinear bit (43) that point sampling already read. The convert-one bit (41) is not among
them: the reference passes the one-cycle mode's texels through the first cycle's pipeline, which never
converts the previous cycle's result, so the bit is the two-cycle mode's (§6).

## 1. One texel or four

**With both the sample-type and palette-enable bits clear, a texel is a point sample**, exactly as
`Mars_RdpTextures.md` §5 describes. **With either set, four texels are read** — the texel the coordinate
falls in, the one to its right, the one below and the one diagonal — and the result is filtered from them
or converted from one of them. The palette-enable bit forces four even when the sample type asks for one;
what the four then are is §4.1's.

The front end is shared: both paths shift the coordinate, decide whether it is beyond the tile's far
corner, and make it relative to the near one, before diverging at the clamp.

## 2. The four texels

### 2.1 The fraction and the step to the neighbour

**The fraction is the relative coordinate's low five bits**, taken before the clamp. When the tile clamps
and the clamp acts — the coordinate beyond the far corner, or negative — the fraction is zero, so a clamped
edge is flat rather than blended.

**The clamp, mask and mirror are the point sample's** (`Mars_RdpTextures.md` §5.2), and each axis also
yields a step to the neighbouring texel:

- **No mask**: one.
- **A mask without mirroring**: one, except at the mask's last texel, where the step returns to zero —
  minus the whole coordinate for s, **minus only its low eight bits for t**.
- **A mirror**: plus one on the forward half, minus one on the reflected half, and **zero at either turn**,
  where the texel and its neighbour are the same texel.

**The second row is the first row's low eight bits plus t's step, and is not cut again**, so it can be
row 256 or row minus one. The first row is cut to eight bits as the point sample's is.

### 2.2 The square's two triangles

**The fractions choose one of the texel square's two triangles**: the upper one when their sum carries
into bit 5 — when it reaches 32 — and the lower one otherwise. The filter uses three corners, and which
three is this choice.

**YUV halves its chroma horizontally**, so for a YUV tile red and green use a second fraction: the s
fraction halved, with the texel's own low bit as its top bit. That fraction chooses a *chroma triangle*,
which can differ from the luma triangle, and the four texels are read accordingly:

- **At 16 and 32 bits**, the right-hand texels take their luma from the neighbouring texel and their
  chroma from one step further along, since one chroma word serves two texels.
- **At 4 and 8 bits**, where chroma and luma are one byte, the right-hand texels are read a step further
  along; and **when the two triangles differ, blue and alpha swap between opposite corners** — the first
  with the fourth, the second with the third.

For every other format both triangles are the same and nothing moves.

## 3. The filter

### 3.1 Three corners

**With the first cycle's bilinear bit set**, each channel is interpolated across three corners, in 1/32
steps rounded to nearest:

- **Lower triangle**: `t0 + (sf·(t1 − t0) + tf·(t2 − t0) + 16) >> 5`.
- **Upper triangle**: `t3 + ((32 − sf)·(t2 − t3) + (32 − tf)·(t1 − t3) + 16) >> 5`.

where `t0` is the texel, `t1` its right-hand neighbour, `t2` the one below and `t3` the diagonal. **Red and
green use the chroma fraction and triangle; blue and alpha use the texel's own.** For formats other than
YUV the two are the same.

The shifts are arithmetic, so a negative difference rounds towards minus infinity after the bias; YUV's
chroma, stored less 128, is the channel that is negative in practice.

### 3.2 The mid-texel

**With the mid-texel bit set and both fractions exactly 16**, the channel is instead a four-corner term:

`t3 + (64·(t1 + t2) − 128·t3 + 64·(~t3 + t0) + 192) >> 8`

**That is the four corners' average rounded to nearest**, `(t0 + t1 + t2 + t3 + 2) >> 2`, for every
integer input: `~t3` is `−t3 − 1`, the numerator is `64·(t0 + t1 + t2 − 3·t3 + 2)`, and a floor of a
quarter of that, plus `t3`, is the floor of the rounded average. Mars computes the average.

Red and green take it when the *chroma* fraction and t's fraction are 16. For a YUV tile that is not the
luma condition: a chroma fraction of 16 means an s fraction of 0 or 1 at an odd texel, so chroma and luma
reach the mid-texel term at different coordinates, and never at the same one.

### 3.3 The conversion instead

**With the bilinear bit clear**, the four texels are not filtered but one is converted
(`Mars_RdpTextures.md` §5.3): the chroma from the chroma triangle's base corner — the diagonal in the
upper triangle, the texel itself in the lower — and the luma from the luma triangle's.

### 3.4 A cut left out

**Not in Mars: the reference's nine-bit cut of all four filtered channels.** The argument of
`Mars_RdpTextures.md` §5.3 applies unchanged — the combiner reads every input through a nine-bit sign
extension — and the point sample's rule was already left out on the strength of a surviving breakage.
This rule was not separately broken; it was not built.

## 4. Palettes

### 4.1 Lookup

**A palette-enabled texel is an index into a palette in the upper half of texture memory.** The tile's
format and size — not the palette's — decide how the index is read:

| Tile | Index |
|---|---|
| 4-bit, not YUV | the palette number, then the nibble at the 4-bit index |
| 8-bit, not YUV | the byte at the 8-bit index |
| 16- or 32-bit, not YUV | the high byte of the word at the 16-bit index, the word index cut to ten bits |
| YUV at any size | the byte at the 8-bit index — its top nibble with the palette number at 4 bits |

Odd rows flip bits as for any fetch. **Byte indices are cut to eleven bits and word indices to ten**, not
the twelve and eleven a texel fetch keeps, so indices are always read from the lower half of texture
memory and the palette from the upper. A YUV tile steps two texels to its right-hand neighbour, where
every other tile steps one.

**Palette entry n is four words, one per bank, at word `0x400 + 4n`**, and the four corners read banks 0,
1, 2 and 3 in order — **reversed, bank `3 − k`, in the upper chroma triangle.** An entry is RGBA16 or IA16
by the palette-type bit, decoded as the texel formats of the same name are. When the luma and chroma
triangles differ, blue and alpha swap between opposite corners, as for 4- and 8-bit YUV (§2.2).

**With the sample-type bit clear, the four texels are one index read four times**: the texel's own
index, through all four banks of its entry. In the ordinary case every bank holds the same colour and
the filter returns it; the banks exist to be read separately only when something has written them
separately.

**Whether a palette lookup cuts the row to eight bits cannot be seen.** The reference cuts it for four
texels and not for one, and a breakage cutting it for one survived (§5.5). It could not have failed: a row
256 further on is 2,048 bytes or 1,024 words further on, exactly the width the index cuts keep, so both read
the same place. Mars keeps the reference's shape — the four-texel path shares the texel fetch's cut and the
single-texel path has none — and neither choice is graded.

### 4.2 The palette load

**The palette load is a tile load with three differences** (`Mars_RdpTextures.md` §3):

- **More than one row loads nothing.** angrylion marks the pipeline crashed, and parallel-rdp reports a
  crash; neither writes texture memory.
- **Its coordinates are kept in quarter texels**, not whole ones, so a step a tile load counts as four
  texels counts as sixteen — which the tile's size then divides into words: **a 4-bit tile by four, so
  one entry fills one group of four words**. An 8-bit or YUV tile spaces entries eight words apart, and
  16- and 32-bit tiles sixteen.
- **From a 16-bit image it takes one entry per step**, two image bytes, rather than four texels. At an
  even image address the entry's sixteen bits fill all four words; at an odd one the eight-byte window is
  read as a tile load reads it. From 8- and 32-bit images it steps as a tile load does, and an even
  address still repeats the window's first sixteen bits four times.

**Where the palette lands is the tile's memory field.** A 4-bit tile starting at group 256 puts entry n at
word `0x400 + 4n`, where lookup reads it; any other placement loads texture memory that lookup does not
read as a palette, which is not an error. How content chooses the tile is not this page's question.

## 5. What the differential says

### 5.1 The first run

**The first 166 cases — 46 named and 120 random — matched angrylion on the first run with the slice
built, and no earlier case changed.** That is recorded as a result and not as evidence: a clean first
run says the rules were read carefully, not that the cases would notice a rule read wrongly. §5.5 is the
evidence for that.

The named cases are every format and size both references load, filtered through a rectangle whose
steps are fractions of a texel; the three YUV sizes parallel-rdp refuses; masked, mirrored and clamped
tiles; the mid-texel term for an RGBA and a YUV tile, and its mode away from the mid-texel; the conversion
of four texels; texel 1 filtered; a perspective triangle and one with coordinates out of range; eight
palette loads (256 and 16 entries, the lower half, an 8-bit tile, 8- and 32-bit images, an odd address,
two rows); and nine lookups through palettes of both types, with and without the sample-type and bilinear
bits, through 4-, 8- and 16-bit indices, a YUV tile and a mirrored tile. The random cases draw formats,
tiles, masks, loads, palettes and modes as the point-sampled random cases do (`Mars_RdpTextures.md` §7.2),
and about a third of them read through a palette loaded after their tile.

### 5.2 The palette load the textures slice left uncharacterised

`Mars_RdpTextures.md` §3.3 recorded that the references disagreed about one palette load, *"on a palette
loaded into a tile no command had set"*, and left it to this slice. **It is characterised, and it is not
about loading.** The case was sixteen entries from a 16-bit image through tile 5, the first command ever
to name that tile. Replayed alone through the validator, angrylion writes entry 1 at word 4 and parallel-rdp
leaves word 4 zero. The same load through a tile first set to all zeros, format, size, line and memory
alike, agrees.

The difference is the tiles' starting state. **angrylion's tiles start at zero, a 4-bit size; parallel-rdp's
start at 16 bits**, which spaces entries sixteen words apart instead of four (§4.2). Mars's tiles start at
zero, as angrylion's do, and what the console's tiles hold before a command sets them is not known from
either.

**It is not a case**, because it cannot be one here: the display processor is not reset between cases
(`Mars_RdpDifferential.md` §2), so only the dump's first case ever sees a tile no command has set, and a
case that depends on its position in the dump is a measurement of the dump. The named case *palette load
through an all-zero tile* keeps the rest of the original — the tile number, the image, the sixteen entries
— and agrees.

### 5.3 One dispute

**The referee leans towards angrylion here** (`Mars_RdpReferee.md` row 10): the N64_MiSTer core builds its
palette index out of the fetched bytes with no test of the tile's format, so a YUV tile is indexed like any
other — angrylion's answer, not parallel-rdp's. It is filed as a lean because that core's YUV conversion
takes precedence over the palette under some mode combinations, so the two paths are not cleanly separable
by reading alone.

**A YUV tile read through a palette.** angrylion reads each index as a byte and steps two texels to the
neighbour (§4.1); parallel-rdp's palette lookup has no YUV branch and samples nothing, and the pixels it draws
are black. Mars follows angrylion. A palette-enabled YUV tile is an odd thing for content to ask for, and nothing
here says what the hardware does with it.

### 5.4 Six cases graded against angrylion alone

parallel-rdp refuses the uploads of **YUV tiles at 4, 8 and 32 bits** (three filtered rectangles, and one
lookup through a 4-bit YUV tile), and **a palette load through a 32-bit tile**, logging the refusal and loading
nothing; and it reports **a palette load of two rows** as a crash, which angrylion also loads nothing for. None
is a disagreement about an answer both compute. **parallel-rdp agrees with angrylion on every other case.**

### 5.5 Against Mars

**176 cases**: fifty-six named and 120 random. With the slice removed, **156 fail — all fifty-six named and
100 of the random — and no case from an earlier slice does.** Twenty random cases pass without it; they
were not examined one by one. Some named failures are inherited rather than earned: the case *palette load
of two rows* loads nothing, and fails without the slice only because an earlier palette load went missing
from texture memory, which no case clears (`Mars_RdpDifferential.md` §4.9). The breakages below are the
per-rule evidence.

**The first round: forty-nine breakages, forty-four caught.** Five survived:

- **The mid-texel term without its mode bit, and the mode bit read one place lower.** No case put both
  fractions at exactly 16 with the bit clear; the bit below it is the bilinear bit, set whenever filtering
  is.
- **The second row cut to eight bits** (§2.1). No tile had more than 256 rows.
- **A 4-bit YUV tile's palette index** (§4.1). No case read one through a palette.
- **The nearest lookup's row cut to eight bits**, which cannot be seen (§4.1) and is not carried.

**Seven more were caught only by a random case** — t's wrap by its low byte, each corner's own bank, the
bank reversed in the upper triangle, the nearest lookup's four banks, the eleven- and ten-bit index cuts,
and a palette entry's alpha bit. A rule one random draw happens to reach is a rule the next change to the
generator can stop reaching, so each was given a named case.

Eight named cases were added: the mid-texel with its bit clear; a 512-row tile filtered across rows 256
and 511; a lookup through a 4-bit YUV tile; filtered and single-texel lookups through a palette loaded from
an odd address, whose banks differ; 4- and 16-bit index tiles placed in the upper half; and a palette's
alpha as the combiner's multiplier. Two more came from §5.2: the all-zero tile, and a 32-bit tile, which
parallel-rdp refuses.

**One new case did not catch its breakage at first.** The 512-row tile drew its second rectangle, the one
reaching row 511, at rows 40 to 60 — outside the cases' scissor, whose corner of 124 is in quarter pixels
and so 31 pixels. It drew nothing, and t's wrap by its low byte survived again. A probe of the texels the
case actually sampled showed it; with the rectangle moved inside, both of the tile's rules are caught.

**The mid-texel term was then rewritten as the average it is** (§3.2). The reference's form had been
broken twice in the first round, and both were caught; the average was broken twice in the second, and
both were caught.

**The second round: the twelve rules above and the two average breakages.** All are caught by named cases
except the nearest lookup's row cut.

### 5.6 A test without the tools

**A tool-free test** in `MarsRdpTests` loads a sixteen-entry RGBA16 palette and an eight-by-eight 8-bit
colour-indexed tile from RDRAM, and draws the tile filtered through a texture rectangle with fractional
steps. It checks twelve of angrylion's pixels, the first two palette entries in all four banks, and the
tile's odd second row. With the slice removed it is the only `MarsRdpTests` test that fails. Of twelve
breakages run against it alone it catches nine; it does not see a fraction sum of exactly 32, the bank
reversal — its palette's banks all hold the same entry — or a palette entry's alpha, which nothing in its
combine mode or colour image carries.

## 6. What is not here

- ~~**The two-cycle mode**, including its conversion of the first cycle's result (the convert-one bit) and
  the second cycle's own bilinear bit.~~ Built two slices later (`Mars_RdpTwoCycle.md` §2), where the
  conversion turned out to have four forms, one of which reads no texel at all.
- ~~**The level of detail**: the fraction the combiner reads, mipmaps, detail and sharpen, and the tile
  choice they make — and with it the reference's texel 0, which `Mars_RdpTextures.md` §6 notes differs from
  the previous pixel's texel 1 only then.~~ Built in the next slice (`Mars_RdpLod.md`), where the texel 0
  concern turned out unfounded.
- ~~**The copy mode**, whose palette lookup is a different path.~~ Built in `Mars_RdpCopy.md` §2.4; the path
  is indeed a different one, reading the entry at four times the index plus the texel's own place.
- **Chroma key** and **noise**, as `Mars_RdpDepth.md` §7 left them.
