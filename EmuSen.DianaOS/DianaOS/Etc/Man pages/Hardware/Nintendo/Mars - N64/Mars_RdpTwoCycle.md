# Mars — the two-cycle mode: two combiner and blender cycles per pixel

*Landed 2026-09-17. Phase D's ninth slice: the mode in which every pixel runs the combiner twice and the
blender twice, with its texels, its level of detail and the order in which one pixel's work interleaves with
the next's. The code is `Rdp/Rdp.TwoCycle.cs` (the pixel loop and the first cycles), `Rdp/Rdp.OneCycle.cs`
(the combiner and blender, now shared between modes), `Rdp/Rdp.Textures.cs` and `Rdp/Rdp.Filter.cs` (the
second cycle's texel), `Rdp/Rdp.Lod.cs` (the two-cycle level of detail), `Rdp/Rdp.Depth.cs` (the blend
shifts) and `Rdp/Rdp.Modes.cs` (each cycle's selectors); the grading is `MarsRdpDifferentialTests`.*

*This page continues the one-cycle slices — `Mars_RdpCoverage.md` for the combiner, blender and coverage,
`Mars_RdpTextures.md`, `Mars_RdpFiltering.md` and `Mars_RdpLod.md` for texels — and describes only what
differs. §5 is the part to read before trusting a number here.*

---

## 0. Where each rule came from

**Every rule is the reference's, graded**, as in the slices before: read from angrylion for mechanism,
written in Mars's own structure, run against angrylion with parallel-rdp as the cross-check.

**angrylion models the mode as a pipeline, and parallel-rdp does not.** In angrylion a pixel's first cycles
run before its predecessor's last ones, and all three disputes this slice records (§5.2) are what that
ordering makes visible. Mars follows angrylion's order.

## 1. The order of work

For each pixel, angrylion's loop does this:

1. **The pixel's first cycles** — coverage, shade, dither, both texels, the level of detail, and the first
   combiner cycle — were already run, at the end of the previous pixel's turn, or at the row's start for its
   first pixel.
2. **Depth is corrected**, and **the second combiner cycle** runs, giving the pixel's colour and alpha as the
   one-cycle mode's single cycle does.
3. **Memory is read and depth tested.** If the pixel survives and is covered — its coverage count under
   anti-aliasing, its coverage bit otherwise — **the first blender cycle** runs.
4. **The next pixel's first cycles run** (step 1 for it). At a row's last pixel "the next pixel" is the one
   past the row's end, with no coverage.
5. **Alpha compare** tests the alpha the first combiner cycle produced at step 1, and if it passes, **the
   second blender cycle** runs and the pixel is written, with its depth if depth update is on.

Mars runs the same five steps in the same order. Most of the state the next pixel's first cycles overwrite at
step 4 is read by nothing after it; three things are, and they are the mode's peculiarities:

- **The combined input.** The first combiner cycle's combined input reads the combiner's last result — the
  previous pixel's second cycle, or at a row's start the first cycle of the pixel past the previous row's end.
  At a primitive's start it is zero. The second cycle's combined input is the same pixel's first cycle, which
  is the point of the mode.
- **Shade alpha in the second blend.** The first combiner cycle refreshes the shade alpha the blender reads, so
  the second blender cycle, running after step 4, reads **the next pixel's** shade alpha — or the pixel past the
  row's end's, with its dither.
- **Dither.** The second blend dithers by the pixel's own colour dither, kept from its step 1.

## 2. Texels

**The first texel is the one-cycle mode's**, filtered by the first cycle's bilinear bit. **The second is taken
at the same coordinates from the second tile** (§3), filtered by the **second** cycle's bilinear bit, and with
the convert-one bit set it converts the first texel instead of its own:

- **Point-sampled, not filtered**: the first texel, its colour's nine bits sign-extended, through the YUV
  conversion (`Mars_RdpTextures.md` §5.3).
- **Point-sampled, filtered**: the first texel's blue, as all four channels.
- **Four texels, not filtered**: as point-sampled — no texel is read.
- **Four texels, filtered**: the four corners are fetched, and each channel is the first texel's blue plus the
  corners' differences weighted by its red and green — all three sign-extended to nine bits — in 256ths: the
  lower triangle `B + (R·(t1 − t0) + G·(t2 − t0) + 128) >> 8`, the upper `B + (R·(t2 − t3) + G·(t1 − t3) +
  128) >> 8`, and at the mid-texel the upper form with `(~t3 + t0) << 6` added and 192 rounding in place of
  128. The triangles are the filter's (`Mars_RdpFiltering.md` §2.2).

**The second combiner cycle swaps the two**: its texel 0 input reads the second texel and its texel 1 input
the first. The first cycle reads them as named.

**What is fetched follows what is read**, as in the one-cycle mode but with four outcomes: both texels, and the
next row's rule of §3, when the second cycle reads texel 1 or alpha compare is on and the first cycle's alpha
reads a texel or the fraction; both texels when the first cycle reads texel 1 or the second reads texel 0;
the first texel alone when the first cycle reads it or either reads the fraction; and neither otherwise. Every
texel a cycle reads is fetched; the four outcomes differ only past a row's end.

## 3. The level of detail and the two tiles

**The two-cycle mode measures from the pixel itself**: the movement to the next pixel, and the movement down to
the next row — the coordinate plus its y step with the low fifteen bits cleared — the larger of the two, from
the rules of `Mars_RdpLod.md` §3 otherwise. This is the measurement parallel-rdp uses in both modes.

**Two tiles are chosen.** With the enable bit set and detail off, the first is the primitive's tile plus the
level, and the second one past it — or the same tile when distant, or when magnified without sharpen. With
detail on, the first is one past the level unless magnified, and the second one past that unless distant or
magnified. With the enable bit clear they are the primitive's tile and the one after.

**Past a row's end, in the first of §2's four outcomes**, when the next row is drawn and the span reaches three
pixels, the pixel past the end takes its second texel from **the next row's first pixel**,
with the first cycle's filter and no conversion, and its levels from that pixel: the fraction and first tile
measured down alone, the second tile down and across. Only the first combiner cycle of the pixel past the end
reads these, and only the next row's first pixel reads that cycle's result, through its combined input (§1).

## 4. The blender

**The first blender cycle always blends, and never divides**: the two weighted inputs are summed and shifted
down five, as a forced blend is — whatever the force-blend bit says. With memory alpha as its second weight it
shifts the weights by **the previous pixel's** stored depth slope against this pixel's — the slope the previous
depth test read, or fifteen if depth test was off — which the reference calls the inter-pixel shifters.

**The second blender cycle is the one-cycle blender without its tests**: blend, pass through or take the second
input by the colour-on-coverage and blend-enable rules, dividing unless forced, with the pixel's own shifts. Its
first input's colour, where the one-cycle blender reads the combiner's pixel, is **the first cycle's result**.
Alpha compare and the coverage test are made before it, at steps 3 and 5 of §1.

**Each cycle has its own selectors**: the combiner's first cycle reads the upper selector of each pair, the
blender's first cycle the upper of each pair of two bits; the one-cycle mode reads the combiner's second cycle
and the blender's first.

## 5. What the differential says

### 5.1 The first run

**The first 145 cases — twenty-five named and 120 random — matched angrylion on the first run, and no
earlier case changed.** As on every page before, that is recorded as a result about reading, not as evidence
that the cases bite; §5.4 is that.

The named cases pass shade through both cycles; read a texel times shade then times the primitive colour;
read the combined input in the first cycle; read two tiles as texel 1 and as the swapped pair; convert one
texel from another point-sampled, filtered, and over four texels, and convert a YUV second texel without the
bit; run both blend cycles forced and with the second dividing; weigh each blend cycle by memory alpha; read
shade alpha in the second blend; test alpha, alpha from coverage and coverage times alpha; draw depth-tested
anti-aliased triangles; filter mipmaps trilinearly, under detail and under sharpen, and through a texture
rectangle; take the second texel past a row's end; look up a palette; and fill a rectangle. Fourteen more were
added by §5.4. The random cases draw both combiner cycles' eight selectors and both blenders' eight, over
mipmap chains of one to four levels in a random format, through triangles and texture rectangles, with dither,
coverage, depth, perspective, either cycle's bilinear bit, convert-one, four texels, the level of detail,
sharpen and detail, into 16- and 32-bit colour images.

### 5.2 The three disputes, and the constraint they put on the random cases

**parallel-rdp disagreed with angrylion on four named cases and fifteen random ones.** Reading its shader
explains all four: it runs a pixel's two cycles together, so none of §1's pipelining exists in it. Three
disputes are recorded, and Mars follows angrylion:

- **The combined input.** angrylion's first cycle reads the previous pixel's result, and at a row's start
  the result for the pixel past the last row's end; parallel-rdp reads zero. Seven named cases carry it: the
  first cycle reading the combined input, the second texel past a row's end, and five more that §5.4 added
  for rules which can show through nothing else.
- **The first blend's memory-alpha shifts.** angrylion shifts by the previous pixel's stored depth slope
  (§4), parallel-rdp by the pixel's own. Two named cases carry it.
- **The second blend's shade alpha.** angrylion reads the next pixel's (§1), parallel-rdp the pixel's own.

**The random generator was then constrained** never to select those three inputs — the combined input in
the first cycle, memory alpha as the first blend's second weight, shade alpha as the second blend's first.
With the constraint, parallel-rdp agrees with all 120 random cases in the full dump. The fifteen that
disagreed were cases from the unconstrained generator, and since the constraint changes the whole draw they
were not traced one by one; what is recorded is that no random case disagrees now.

**Five random cases disagree when replayed alone and agree when every tile is set first** — the tile
start-state difference of `Mars_RdpFiltering.md` §5.2, not a rule. It is visible here because a random case
may name a tile no command in that isolated replay has set.

**Nothing here says which model the hardware follows.** That angrylion pipelines the cycles is an argument
about how the parts are arranged, not a measurement of them, and parallel-rdp's own source says of the texel
swap that following the pipelining further is somewhere it would rather not go.

### 5.3 What the cross-check still covers

Of the thirty-nine named cases, twenty-nine are cross-checked and agree, including every texel rule of §2 — both
tiles, the swap, all four convert-one forms, the mid-texel centre — the level of detail and the two tiles of
§3, the second blend's memory alpha, alpha compare in its three forms, and the fill rectangle. The disputes
are confined to the three inputs of §5.2.

### 5.4 Against Mars

**The first round: forty-one breakages, thirty-three caught, eight survived.** Each survivor is a rule no
case reached:

- **The point conversion's nine-bit sign.** Every convert-one case filtered its first texel, whose channels
  are eight bits; only a first texel that is itself a conversion carries a ninth.
- **Four texels converted without filtering, which reads no texel.** No case took four texels with the
  second cycle's bilinear bit clear, so nothing distinguished converting the first texel from converting
  the four fetched.
- **The filtered conversion's mid-texel term.** No case set the mid-texel bit on a coordinate at a texel's
  centre.
- **Compare alpha under coverage times alpha, and compare alpha dithered.** The alpha-compare cases neither
  multiplied by coverage nor dithered alpha; the cases that did neither tested alpha.
- **Alpha compare's part in the fetch rule** (§2's first outcome). It changes only what the pixel past a
  row's end fetches, which reaches the image only through the combined input, and no alpha-compare case read
  it.
- **The level measured when only the first cycle reads the fraction.** Every case reading the fraction had
  the enable bit set.
- **The previous pixel's slope after a pixel drawn without a depth test**, which is fifteen. No case
  followed an untested primitive with a tested one weighing its first blend by memory alpha.

**Two more were caught only by a random case**: the first blend never dividing, and the second tile when the
enable bit is clear. Every named case's first blend weighed by a pixel alpha and its complement, and for
those the divisor is thirty-two whatever the alpha, so dividing and shifting agree; and the tile pair for a
level that is measured only because the fraction is read had no named case at all.

**Twelve named cases were added**, and a second round ran the ten rules above, with the alpha-compare fetch
rule split into its three clauses — a texel 0, a texel 1 or the fraction read by the first cycle's alpha.
All thirteen are caught, each by the case written for it.

**Two more breakages came from re-reading §3 against the reference**, not from the first list: the two tiles
the pixel past a row's end takes when the enable bit is clear, which are both the primitive's, in Mars as in
the reference. Neither was covered, and two more named cases were written for them. **All forty-six breakages
are now caught by a named case.**

### 5.5 The baseline, and a test without the tools

**159 cases**: thirty-nine named and 120 random. With the slice removed, **139 tests fail — all
thirty-nine named cases, 99 of the random, and the tool-free test below — and no case from an earlier slice
does.** That every named case fails without the slice is expected here in a way it was not for the level of
detail: without it the dispatch has no branch for this cycle type, so a two-cycle primitive draws nothing at
all. The twenty-one random cases that pass without it are ones whose primitives draw nothing in the reference
either — clipped away, or killed by depth or alpha — and they were not examined one by one.

**A tool-free test** in `MarsRdpTests` loads the level-of-detail slice's four tiles from RDRAM and draws one
perspective triangle over them in the two-cycle mode, with both cycles filtering, the first mixing the two
tiles' texels, the second mixing its own combined input with the swapped texel, and both blend cycles forced
— the first against memory, the second against the blend colour. A second pass over the same triangle turns
on the level of detail, convert-one and alpha compare and blends over the first, and eighteen of angrylion's
pixels are checked. It fails with the slice removed. **Of the forty-six breakages run against it alone it
catches eight**: the texel swap, the second tile, the conversion of the second texel, the first cycle's
colour selector, the second blender's
second inputs, the second blend reading the first's result, the level measured down a row, and the mode
drawing at all. A first version with one pass caught six; the rules it still cannot see are the ones its two
mode settings do not reach, and the differential cases carry those.

## 6. What is not here

- ~~**The copy mode.**~~ Built in the next slice (`Mars_RdpCopy.md`), which is Phase D's last cycle type.
- **Chroma key** and **noise**, in either cycle, as `Mars_RdpDepth.md` §7 left them. The random cases never
  select noise or turn the key on.
- **A dithered alpha-compare threshold**, which reads noise.
