# Mars — shade, depth, and the depth buffer

*Landed 2026-09-17. Phase D's fifth slice: the attributes a triangle carries past its edges — shade
and depth — interpolated across each span and corrected at partly covered pixels, and the depth
buffer that depth is compared against and stored in. With it, Gouraud-shaded, depth-tested
triangles draw in the one-cycle mode. The code is `Rdp/Rdp.Walker.cs` (the attributes),
`Rdp/Rdp.OneCycle.cs` (the per-pixel steps) and `Rdp/Rdp.Depth.cs`; the grading is
`MarsRdpDifferentialTests` (`Mars_RdpDifferential.md`). Textures are still not here (§5).*

---

## 0. Where each rule came from

**Every rule is the reference's, graded**, as in `Mars_RdpCoverage.md`: read from angrylion for
mechanism, written in Mars's own structure, run against angrylion with parallel-rdp as the
cross-check. `Mars_Documentation.md` §4 lists the Z and delta-Z memory format as documented at
Tier 1; the interpolation's fixed-point details, the correction, and the compare's margins are
not, and are established by this comparison.

Two rules the reference contains are **deliberately not in Mars**, because no input can tell them
apart from what Mars does (§1.3, §3.2). Both are recorded where they would have been.

## 1. The attributes

### 1.1 Where a command carries them

After a triangle's four edge words: eight words of shade if bit 2 of the command is set, eight of
texture if bit 1 is, and two of depth if bit 0 is. Each shade word packs one sixteen-bit half for
each of red, green, blue and alpha. A shade value is its whole half from one word and its fraction
half from another: the value itself, its step across x, its step along the major edge and its step
down y each take a pair. Depth is four whole 16.16 values: the value, its step across x, along the
edge and down y.

The test's triangle setup builds these from three coloured, depth-tagged vertices as planes through
them. That convention is the test's; the RDP reads only the numbers.

### 1.2 One value per row, taken where the major edge crosses it

The walker carries each attribute down the major edge, adding its along-edge step once per row. On
one sub-scanline of each row it records the row's starting values, the major edge's unclipped
column, and the edge's fraction of a pixel:

- **Which sub-scanline** depends on which way the major edge leans relative to the side it is on: the
  last sub-scanline of the row when the lean and side agree, the first when they do not.
- **When they agree, an offset is added** built from the along-edge and down-y steps, each with its
  low nine bits cleared, as three-quarters of the first less three-quarters of the second.
- **The edge's fraction of a pixel is subtracted** times half the x step, with the x step's low bit
  cleared; in the copy cycle this term is zero.
- The result has its low ten bits cleared.

The choice of sub-scanline, the offset and the fraction term are each caught by at least 39 cases when
broken, and so is stepping along the edge; clearing the low ten bits was not broken separately.

### 1.3 The steps, and the depth slope

Per pixel, shade steps by its x step with the low five bits cleared, and depth by its x step
unchanged. Correction at partial pixels (§2) uses coarser versions: shade steps shifted down by 14
and taken as thirteen-bit signed, depth steps shifted down by 10 and taken as twenty-two-bit signed.

The **depth slope** that the compare and the store use is the sum of the whole parts' magnitudes of
the depth's x and y steps, rounded up to the power of two above its highest bit; a sum reaching
`0x4000` is `0x8000`, and zero is one.

**Not in Mars: the reference's value for a unit sum.** The reference gives a sum of exactly one the
slope 3 instead of 2. Both encode to the same four bits and have the same highest bit, which is all
the compare and the store read, so no pixel can differ; a breakage replacing it survived every case,
and Mars has the plain rule.

## 2. The pixel

Each row starts from its recorded values and steps them from the major edge's unclipped column to
the first pixel the span actually draws — a column difference taken modulo 4,096 — and then once per
pixel, in the direction the span is drawn.

**Shade and depth are corrected by coverage.** A fully covered pixel's shade is its value scaled back
from its extra precision, and its depth likewise. A partly covered pixel's shade and depth are moved
to its **first covered sample**: the first row of the coverage mask with a covered sample, from the
top, and that row's first covered column, from the left, each counted in sub-samples and multiplied
by the coarse steps of §1.3. Shade is then clamped to eight bits as the combiner clamps. Depth is kept
to eighteen bits, and a value that overflowed that range becomes the maximum or zero by its top two
bits.

The combiner's shade inputs and the blender's shade alpha now read these values.

## 3. The depth buffer

A command sets the depth buffer's address, twenty-four bits. Each pixel has one sixteen-bit word there
and its two hidden bits.

### 3.1 How depth is stored

**Eighteen bits of depth are kept in fourteen**: a three-bit exponent — the number of leading ones in
the top seven bits, at most seven — and an eleven-bit mantissa, shifted to fit by the exponent. Reading
it back fills the bits the exponent stood for. The word's low two bits and the hidden bits together
hold the depth slope's four-bit encoding.

### 3.2 The compare

The stored depth and stored slope are read back. **At low precision the stored slope widens**: when the
stored exponent is below three, the slope is doubled and raised to at least sixteen, eight or four, or,
if it was already the largest, becomes `0xFFFF`. The compare's **margin** is eight times the highest bit
of the pixel's slope combined with the stored one.

A pixel is **farther** if it is no more than the margin in front of the stored depth, **nearer** if it is
no more than the margin behind, and **in front** if strictly in front.

- **Opaque:** passes if the stored depth is the maximum, or — when the pixel's coverage and memory's
  overflow — if in front, and otherwise if nearer.
- **Interpenetrating:** as opaque, except when the pixel is in front, farther and overflowing; then it
  passes and its coverage is scaled by how far in front it is, in units of the slope.
- **Transparent:** passes if in front or if the stored depth is the maximum.
- **Decal:** passes if farther and nearer and the stored depth is not the maximum. Over a cleared buffer
  that is nothing.

**A blend now also needs the pixel to be farther.** With depth compare on, the blend shifts against memory
alpha come from the difference between the stored slope's encoding and the pixel's, clamped to four.

**Not in Mars: the reference's coplanar flag.** The reference marks the widened maximum as coplanar and
forces farther and nearer true. A stored slope of `0xFFFF` already makes the margin exceed the whole depth
range, so both are true anyway; a breakage removing the flag survived every case, and Mars does not carry
it.

### 3.3 The store

After a pixel is written, with depth update on, its depth is compressed into the word, the slope's top two
bits into the word's bottom two, and the slope's bottom two into the hidden bits. A depth update with the
compare off stores without comparing.

## 4. Primitive depth

With the *primitive depth* mode set, every pixel uses the depth and slope from the primitive-depth command
instead of the interpolated ones, its depth does not step, and its correction steps are zero. This is how a
fill rectangle, which carries no depth, is depth-tested.

## 5. What is drawn, and what is not

- **In the one-cycle mode every untextured triangle now draws**: flat, depth-only, shaded, and shaded with
  depth, and fill rectangles. The four textured forms still draw only in the fill cycle.
- **Textures, the level-of-detail fraction, texture rectangles, the two-cycle and copy modes, chroma key and
  noise** remain as `Mars_RdpCoverage.md` §5 and §8 left them.

## 6. What the differential says

**171 shade and depth cases**: twenty-one named — Gouraud shading with and without anti-aliasing, shade alpha
as a blend weight, one and two triangles over a cleared depth buffer with and without anti-aliasing, all four
depth modes, depth-only triangles, depth written without comparing, primitive depth on rectangles, blend shifts
from stored slopes, a low-precision buffer, a buffer holding the widest slope, a 32-bit image, a steep slope,
depth beyond its range in two modes, a decal behind its surface, and a slope widened at low precision — and 150
random, with one to three triangles of the four untextured forms, every depth mode, random depth clears, random
primitive depths, and the one-cycle modes `Mars_RdpCoverage.md` grades. **parallel-rdp agrees with angrylion on
every one**; this slice added no dispute.

Against Mars before the slice, 132 of the first 167 failed. **With the slice built, all of them matched on the
first run**, and so do the 177 one-cycle and 131 earlier cases.

**The cases were checked for bite, and the checking found more than the implementation did.** Twenty-five
breakages were run and nineteen caught. Of the six survivors, two were rules no input can see, now removed
(§1.3, §3.2). The other four were cases that could not see what they were named for:

- **Depth out of range** had no case whose interpolated depth left eighteen bits; two were added.
- **Transparent mode against a maximum** only matters when the new depth is also the maximum, which the same
  out-of-range case reaches.
- **The low-precision widening** was being tested in the opaque mode with image reads off, where memory
  coverage reads as full, every pixel overflows, and the compare uses *in front*, which ignores the margin;
  the case now uses decal mode.
- **Decal mode** exposed a flaw in two cases: over a cleared buffer, whose stored depth is the maximum, decal
  passes nothing, so both decal cases — including the one named for decal mode — had never drawn at all. Each
  now draws its surface opaque first and switches to decal for the second triangle.

With those, every breakage is caught.

**A tool-free test** in `MarsRdpTests` replays two shaded triangles crossing in depth and checks nine of
angrylion's colours and stored depths along one row. It fails when pixels do not step and when stored depth
decodes wrongly; it does not see a changed margin — its pixels overflow, so *in front* decides — nor
compression of exponents above four, which its depths never reach.

## 7. What is not here

- ~~**Textures**, and everything that follows from them: texture memory, the level-of-detail fraction and texel
  inputs, texture rectangles.~~ Built in the next slice (`Mars_RdpTextures.md`), except the level-of-detail
  fraction, filtering and palette lookup, which followed (`Mars_RdpFiltering.md`, `Mars_RdpLod.md`).
- ~~**The two-cycle mode.**~~ Built four slices later (`Mars_RdpTwoCycle.md`), which is also where this
  page's depth encoding gained a second reader: the previous pixel's stored slope, which shifts that mode's
  first blend. **The copy mode** is still not here.
- **Chroma key** and **noise** (`Mars_RdpCoverage.md` §6.3).
- **The reference's validation clamp**, still not copied (`Mars_RdpDifferential.md` §4.3).
