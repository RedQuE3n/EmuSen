# Mars — the level of detail: mipmap tiles and the fraction

*Landed 2026-09-17. Phase D's eighth slice: how far a pixel's texture coordinates move, the mipmap tile
that movement picks, and the level-of-detail fraction the combiner reads, in the one-cycle mode. The code
is `Rdp/Rdp.Lod.cs` (the measurement, the level, the tile and the fraction), `Rdp/Rdp.OneCycle.cs` (where
each pixel takes them), `Rdp/Rdp.Textures.cs` (coordinates before their clamp), `Rdp/Rdp.Fill.cs` (each
primitive's maximum level) and `Rdp/Rdp.Modes.cs` (the mode bits and the minimum level); the grading is
`MarsRdpDifferentialTests` (`Mars_RdpDifferential.md`).*

*This page continues `Mars_RdpTextures.md` and `Mars_RdpFiltering.md`. §4 is the part to read before
trusting a number here: the two references model this slice differently, so the cross-check covers less
of it than of any slice before.*

---

## 0. Where each rule came from

**Every rule is the reference's, graded**, as in `Mars_RdpTextures.md` §0: read from angrylion for
mechanism, written in Mars's own structure, run against angrylion with parallel-rdp as the cross-check.

**The scope is the one-cycle mode.** angrylion measures the level of detail differently in the two-cycle
mode (§5), and parallel-rdp measures it the two-cycle way in both (§4.2). Where they disagree, Mars follows
angrylion.

## 1. The inputs

- **Three mode bits**: the level-of-detail enable (bit 48), sharpen (49) and detail (50).
- **A maximum level per primitive**: bits 51–53 of a triangle's first word. Texture rectangles and fill
  rectangles carry zero. Every primitive sets it, so it is a property of the draw, not state; a load setting
  it to zero, which the reference also does, is not observable and not carried.
- **A minimum level**: bits 40–44 of the primitive colour command, beside the primitive's own fraction.
- **The combiner reads the fraction** as colour multiplier 13 and alpha multiplier 0, where earlier slices
  read zero.

**The level is measured only when it can matter**: when the enable bit is set or the combiner reads the
fraction, and the combiner reads a texel or the fraction. Otherwise the tile is the primitive's, and the
fraction keeps whatever it last was — which nothing then reads.

## 2. What is measured

**Not the pixel: the movement from the next pixel's texture coordinates to the one after**, along the span,
each divided by w when perspective is on. That is the one-cycle mode's arrangement, in which a pixel's texel
is fetched a pixel early (`Mars_RdpTextures.md` §6), and the level travels with it.

**Near a span's end the second coordinate moves**, when the next row is one the primitive draws:

- **A long span's last pixel** — last column less first, plus the columns clipped from its start, over seven —
  measures from the next row's first pixel to one step past it.
- **A long span's next-to-last pixel, and a seven-pixel span's last**, measure from the next pixel *back* to
  the previous one: two steps, not one, so the movement measured there is about twice a step's — exactly
  twice less one for a step forward, through the complement of §3.1.

With the next row not drawn, every pixel measures from the next pixel to the one after.

**Texel 1 takes the next pixel's level.** For every pixel but a span's last, that is exactly the level the
next pixel's texel 0 then uses — the reference keeps each pixel's texel 0 as its predecessor's texel 1, and
the two are one fetch. Mars measures once and uses it for both. `Mars_RdpTextures.md` §6 said this slice
must revisit whether fetching texel 0 afresh stays equivalent; it does.

**A span's last texel 1** is measured from the pixel after the span, by the span's length: a long span from
one and two steps into the next row, a seven-pixel span from the next row's first pixel and one step past
it, a six-pixel span from the next pixel back to the previous one, and anything else from the next pixel to
the one after.

## 3. The level, the tile and the fraction

### 3.1 The movement

**s and t each give a difference of their seventeen-bit coordinates**, and the larger is the movement, in
thirty-seconds of a texel. **A negative difference is taken as its complement**, one less than its magnitude.
Movement past fourteen bits saturates. **If either coordinate at either end overflowed the divider**
(`Mars_RdpTextures.md` §4.2), there is no movement, and the level saturates too.

### 3.2 Three cases

- **Saturated**: level 0, *distant*, fraction `0xFF`.
- **Under a texel per step** (movement below 32): *magnified*, level 0, distant only if the maximum level is
  zero. The fraction, with neither sharpen nor detail on, is `0xFF` if distant and zero otherwise; with
  either, it is eight times the movement or the minimum level, whichever is larger, and **sharpen sets its
  ninth bit**, which the combiner reads as a negative multiplier.
- **Otherwise**: the level is the base-two logarithm of the whole texels moved, and the pixel is distant when
  the maximum level is zero, when the movement's bit 13 is set, or when the level reaches the maximum. The
  fraction is `0xFF` when distant with neither sharpen nor detail on, and otherwise the low eight bits of
  eight times the movement shifted down by the level.

The fraction is computed whether or not the enable bit is set.

### 3.3 The tile

**With the enable bit set**, a distant pixel's level is the maximum level, and the tile is the primitive's
tile plus the level — **plus one more under detail, unless magnified** — wrapped to eight. With the bit clear
the tile is the primitive's.

## 4. What the differential says

### 4.1 The first run

**The first 140 cases — twenty named and 120 random — matched angrylion on the first run.** As with
filtering, that is a result about reading, not evidence of bite (§4.4 is that).

The named cases draw a four-level mipmap chain — RGBA16 tiles of 32, 16, 8 and 4 texels, each shifted one
further — through perspective and affine triangles, from tile 6 so the tiles wrap, with the maximum level
below the chain and at zero, with the fraction as the multiplier with and without the enable bit, below the
minimum level, under detail and sharpen, magnified under detail, reading texel 1, past the divider's range,
moving past a quarter of the coordinate range, as a texture rectangle, at the ends of spans of seven to ten
pixels, and across a triangle's rows. The random cases draw a chain of one to four levels in a random format
from a random first tile, with random maximum and minimum levels, mode bits and combiner inputs weighted
towards the fraction.

### 4.2 The dispute, and why it covers so much

**parallel-rdp disagreed on ten of the twenty named cases and 23 of the 120 random ones.** Its shader measures
the level of detail with one function for both modes: from the pixel's own coordinates to its right-hand
neighbour's and to the row below. That is angrylion's *two-cycle* measurement (§5). angrylion's one-cycle
measurement is §2's — along the span only, from the next pixel, with the span-end rules.

**The explanation was tested before it was recorded.** Four cases isolate it, each drawing a mipmap chain
under an affine triangle whose texture moves along one axis only:

| Movement | Level | Result |
|---|---|---|
| along x, nine texels a pixel | saturated at the maximum | agree |
| along x, a texel and a half a pixel, spans of at most seven | unsaturated | agree |
| along x, a texel and a half a pixel, long spans | unsaturated | **disagree** |
| down the rows, nine texels a row | | **disagree** |

So the references agree exactly where the two measurements coincide: when nothing moves down the rows, and
away from a long span's last two pixels, or when the level saturates either way. One probe did not fit at
first: a texture rectangle with slow movement along x, sharpened so its fraction is unsaturated, agreed while
its combiner multiplied a texel by the fraction; showing the fraction directly as the pixel's colour, the same
rectangle disagrees (§4.4). That fits a difference
the first combiner rounded away — by §3.2's arithmetic, 124 against 128 at the next-to-last pixel — but it was
not checked pixel by pixel.

**It is recorded as one dispute**, carried by twenty named cases: the ten that disagreed, the two one-axis
cases that disagree, four of §4.4's, and the four span-end rectangles once they could show it (§4.4). Mars
follows angrylion. Nothing here says which measurement the hardware makes in the one-cycle mode; that angrylion keeps two, and makes the one-cycle one along the span
where a pixel's texel is fetched a pixel early, is an argument about the pipeline's arrangement, not a
measurement.

**The random cases are graded against angrylion alone.** Nearly any case that reads an unsaturated level meets
the dispute somewhere, and a generator constrained to avoid it — no movement down the rows, no long spans —
would no longer exercise §2's span-end rules, which are the part angrylion alone describes.

### 4.3 What agrees

Thirteen named cases are cross-checked and agree: the maximum level below the chain and at zero, the divider's
range, movement past a quarter of the coordinate range, a texture rectangle with the enable bit set, a
triangle's rows, the two agreeing one-axis cases, two magnified rectangles, a divider that under- and
overflows at some pixels, s held under the divider's range, and movement with bit 13 set.

### 4.4 Against Mars

**The first round: forty-two breakages, thirty-six caught.** Six survived:

- **Measuring when only the fraction is read.** Every case reading the fraction also read a texel.
- **A long span's last texel 1 measured a step into the next row.**
- **The divider's under-flag**, when only its over-flag was checked.
- **A magnified pixel counted distant when there are no levels, and the plain fraction that follows** — two
  breakages, both visible only through a magnified fraction with the maximum level at zero, which no case read.
- **Sharpen's ninth bit**, which no magnified, sharpened fraction reached.

**Five more were caught only by a random case**: the fraction as the alpha multiplier, the span-end rule's
condition that the next row be drawn, a seven-pixel span's last texel 1, the divider's flags saturating the
level, and the movement's bit 13. Eight named cases were added — most showing the fraction directly as the
pixel's colour, so that a rounding in the combiner cannot hide a wrong one.

**The second round: fourteen breakages** — the eleven rules above, and three that named cases had caught
only alongside random ones — **twelve caught.** Two still survived, each for a reason the first
attempt did not anticipate:

- **The texel-1 case moved linearly.** Measuring from the next row's first pixel or from the pixel after the
  span gives the same movement when coordinates change at a constant rate, so no affine case can tell the
  span-end rules for texel 1 apart. It was made perspective.
- **The divider case's flagged coordinates moved far enough to saturate anyway**, so dropping the flag check
  changed nothing. A case whose s stays under the divider's range without moving was added.

With those, both are caught.

**Five cases measured less than their names said.** The four span-end rectangles and the rectangle with the
enable bit were drawn with perspective on, and a texture rectangle carries no w: every texel coordinate was
flagged and clamped to one value, so the span-end rules could not show. They still caught the rectangles'
maximum level of zero, through the tile the clamped coordinate was read from. The baseline below exposed it —
all five passed with the slice removed. They were made affine, the span-end rectangles sharpened so their
fraction is unsaturated; the four then met the dispute, and the rules they catch were run again and are caught.

**Every breakage is now caught by a named case.**

### 4.5 The baseline, and a test without the tools

**153 cases**: thirty-three named and 120 random. With the slice removed, **sixty-five tests fail — thirty
named cases, thirty-four of the random, and the tool-free test below — and no case from an earlier slice
does.** The three named cases that pass without it are the triangle with no maximum level, whose level always
saturates at the primitive's tile and reads no fraction; the magnified, sharpened rectangle, whose negative
fraction clamps the pixel to the same black as a fraction of zero; and movement down the rows alone, which
angrylion's one-cycle measurement does not see at all — so the case that passes without the slice is itself
evidence for §2.

**A tool-free test** in `MarsRdpTests` loads a four-level mipmap chain from RDRAM and draws a perspective
triangle over it with the fraction as the multiplier, and checks twelve of angrylion's pixels, several at span
ends. It fails with the slice removed. Of fourteen breakages run against it alone it catches nine; it does not
see the enable bit on its own, since it reads the fraction; a seven-pixel span's end, which it has none of;
texel 1's tile, which it does not read; the complement; or a distant pixel taking the maximum level.

## 5. What is not here

- ~~**The two-cycle mode's level of detail**, which angrylion measures from the pixel itself, across x and down
  y, and which also chooses the second cycle's tile.~~ Built in the next slice (`Mars_RdpTwoCycle.md` §3),
  where both of those turned out to be as predicted here.
- **The copy mode**, whose level of detail is another path.
- **The copy mode** generally, and **chroma key** and **noise**, as earlier pages left them.
