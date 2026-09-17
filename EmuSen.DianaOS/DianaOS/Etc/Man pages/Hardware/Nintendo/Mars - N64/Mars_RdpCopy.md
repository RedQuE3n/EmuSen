# Mars — the copy mode: four texels a step, written as bytes

*Landed 2026-09-17. Phase D's tenth slice, and the last of its cycle types: the mode that reads four texels
at a time and writes them to the colour image as bytes, with no combiner, no blender and no depth. The code
is `Rdp/Rdp.Copy.cs` (the walk, the texel read, the alpha mask and the write), `Rdp/Rdp.Lod.cs` (the tile it
picks) and `Rdp/Rdp.Walker.cs` with `Rdp/Rdp.Fill.cs` (the two rules the mode changes before a span exists);
the grading is `MarsRdpDifferentialTests`.*

*This page continues `Mars_RdpTextures.md` for texture memory and `Mars_RdpLod.md` for the level of detail,
and describes only what differs. §5 is the part to read before trusting a number here: the references model
this mode differently enough that seven of its rules are disputes.*

---

## 0. Where each rule came from

**Every rule is the reference's, graded**, as in the slices before: read from angrylion for mechanism,
written in Mars's own structure, run against angrylion with parallel-rdp as the cross-check.

**The two references model the mode differently.** angrylion copies eight bytes at a time, as a hardware
fetch would; parallel-rdp computes each pixel on its own and writes whole pixels. Where the two coincide —
which is most of what content does — they agree. Where they do not, §5.2 records the difference and Mars
follows angrylion.

## 1. The walk

**A 32-bit colour image draws nothing.** The reference marks its pipeline crashed and stops the primitive;
Mars returns, which is the same in every way a case can see.

For each row the primitive draws, the walk starts at the span's near end — its left pixel when the major
edge is on the left, its right pixel otherwise — and moves a **group of eight bytes** at a time:

- **A group covers eight pixels in a 4- or 8-bit image and four in a 16-bit one.** Its bytes are written
  from the group's first address in the walk's direction, which for a right-to-left span means the bytes
  run backwards through memory (§5.2's dispute).
- **The group is cut at the span's end**: at most eight bytes, and no further than the last pixel's last
  byte.
- **The texture coordinates advance once a group**, not once a pixel, by the primitive's per-pixel steps.
  A texture rectangle therefore carries a step four times what the same rectangle would in any other mode,
  which is how content uses this mode.

**Two rules reach back before the span.** The edge walker skips this mode's sub-pixel attribute correction,
and a rectangle's bottom takes its whole last row as it does in the fill cycle (`Mars_RdpCoverage.md` §5).
Both were already in Mars for the fill cycle and are now shared.

## 2. The four texels

**Four texels are read a group**, at the coordinate and the three columns after it. The coordinate is the
tile's shift applied to s and t, the tile's origin subtracted, and the low five bits dropped — **there is no
clamp and no fraction in this mode**, only the tile's mask and mirror, applied to each of the four columns
and to the row.

**Each column's address strides twice its texel's width**: one byte for a 4-bit texel, two for an 8-bit one
or any YUV tile, four for 16 bits or wider. The row's base is the tile's line times the row, kept to nine
bits, plus the tile's place in texture memory, and an odd row swaps eight bytes as it does everywhere else.

### 2.1 Four banks, one access

**Texture memory answers the four addresses at once from four banks**, a bank being bits 2 and 3 of an
address. A texel whose bank an earlier address has already claimed **reads that address's word**, not its
own. That is what makes the mode's narrower texels repeat: at a stride of two bytes the second and fourth
texels land in the same banks as the first and third, and at one byte all four land in the first's.

**The word is sixteen bits**, indexed by the address's bits 2 to 11; bit 12 chooses between texture memory's
halves, and a texel whose address has it set takes its word from the upper half's read instead.

### 2.2 YUV

**A YUV tile reads its chroma a further step along.** The lower half's read takes the second address two
strides past the first instead of one, and the fourth three strides past that second — the upper half's read
keeps the plain addresses. A YUV or 32-bit RGBA tile then always takes the lower half's words, whatever bit
12 says.

### 2.3 The byte a narrow texel becomes

**A texel narrower than sixteen bits is cut to one byte**, by the tile's format and size:

- **Four bits**: the nibble the address picks, doubled — or the tile's palette number above it for a
  colour-indexed tile, or, for intensity-alpha, the doubled nibble folded as `(x & 0xE0) | ((x & 0xE0) >> 3)
  | ((x & 0xC0) >> 6)`.
- **Eight bits**: the two nibbles of the texel's own byte — or, for intensity-alpha, its high nibble doubled.
- **Sixteen bits or wider**: the word's high byte.

**The eight bytes of a group** are the third and fourth texels' whole words in the low half, and in the high
half either the first and second texels' whole words, when the texel is sixteen bits, or the four texels'
bytes. A 4-bit colour image copies zeros instead.

### 2.4 Palettes

**With a palette the texel is sixteen bits whatever the tile says**, so neither §2.3's byte rule nor §2.2's
wide rule applies, and the palette's type — which decides how an entry decodes in every other mode — does not
matter here, because the entry's sixteen bits are copied as they stand. Each texel's word gives an index — a
nibble with the tile's palette number above it for a 4-bit tile, or both nibbles of its own byte otherwise —
and the entry is read from the upper half at four times the index **plus the texel's own place**, so the four
texels read four different banks.

## 3. Alpha compare, and the write

**Without alpha compare every byte is written.** With it:

- **A 16-bit image** keeps each pixel by **its own alpha bit**, the low bit of its sixteen bits.
- **An 8-bit image** keeps each pair of bytes by comparing **one of the low four bytes** against the alpha
  threshold — the blend colour's alpha, or zero where the reference would dither it from noise
  (`Mars_RdpCoverage.md` §6.3).
- **Any other image writes nothing at all.**

**Each byte is written on its own**, with the hidden bits its pair shares taken from the byte's low bit and
written only by an odd address, as an 8-bit image's writes do elsewhere (`Mars_RdpTextures.md` §3).

## 4. The level of detail

**One tile, no fraction.** The level is measured as the one-cycle mode's is — from the next group's
coordinates to the one after, with none of the span-end rules — and picks one tile: the primitive's plus the
level, plus one more under detail unless the pixel is magnified. Without the enable bit the tile is the
primitive's. There is no second tile and nothing reads a fraction, since there is no combiner.

## 5. What the differential says

### 5.1 The first run

**Every copy case matched angrylion on the first run** — the twenty-nine named ones written from §1–§4, the
120 random ones added next, the nineteen written while characterising §5.2's disputes, and the ten that §5.4
added. As on the pages before, that is a result about reading, not evidence that the cases bite; §5.4 is that.

The named cases copy a 16-bit texture rectangle, one whose span is not four pixels wide, a flipped one, one
stepping four texels a pixel and one stepping backwards, and one whose step is not whole texels; copy into
16-, 8-, 4- and 32-bit images, and every texture format and size into each; copy through both kinds of
palette, from a 4-bit and an 8-bit index; copy from a tile in texture memory's upper half, a masked and
mirrored tile, a shifted tile, a tile with a size and a load offset, and a tile masked past its size; copy
triangles — inside the scissor, clipped on either side, with negative coordinates, under perspective, over
a mipmap chain, under detail, and with the major edge on the right; and test alpha in a 16- and an 8-bit
image. The random cases draw one or two shapes, rectangles and triangles, over mipmap chains of one to four
levels in a random format and size, with random tiles, perspective, palettes and alpha compare, into 16- and
8-bit images.

### 5.2 Seven disputes, because the references model the mode differently

**angrylion copies eight bytes a group; parallel-rdp computes each pixel by itself** and writes whole pixels,
sampling the texel its own x asks for. The two coincide for what content does with this mode, which is why
most cases agree, and part ways in seven places. Each is carried by a named case, and Mars follows angrylion:

- **A right-to-left span is written torn.** angrylion starts at the span's first pixel's *first byte* and
  writes the group's eight bytes backwards through memory, so in a 16-bit image the bytes land half a pixel
  off and the span's end pixels are written in halves — a background byte beside a copied one. parallel-rdp
  writes whole pixels. Three named cases carry it, one of them with the texture coordinate held still so that
  nothing but the write order can differ. Only a triangle can reach it: a texture rectangle is always
  left-major.
- **The level of detail picks the tile.** angrylion measures it (§4); parallel-rdp copies from the
  primitive's tile always. The detail case carries it; a mipmapped triangle whose level stays zero agrees.
- **A YUV tile's chroma step** (§2.2), which parallel-rdp's copy path has no case for.
- **An eight-bit intensity-alpha texel**, which angrylion reduces to its high nibble doubled and parallel-rdp
  takes whole.
- **A four-bit colour-indexed texel**, which angrylion gives the tile's palette number as its high nibble and
  parallel-rdp doubles like an intensity nibble.
- **A four-bit intensity-alpha texel**, which angrylion folds into intensity and alpha bits (§2.3).
- **Alpha compare in an eight-bit image**, which angrylion tests byte by byte and parallel-rdp does not test
  at all, testing only 16-bit images.

**Four cases are graded against angrylion alone.** parallel-rdp reports a crash for a copy into a 32-bit
image — as angrylion does, but the validator cannot compare a crashed run — and it aborts on the *fill* that
each 4-bit case needs for a background, which `Mars_RdpDifferential.md` §4.4 already recorded.

**The random generator is constrained to stay clear of four of those**: no intensity-alpha tile narrower
than sixteen bits, no four-bit colour-indexed tile unless a palette reads it, alpha compare only in a 16-bit
image, and a triangle that would be right-major is mirrored. Before those constraints 67 of 120 random cases
disagreed; after them, **none disagrees in the full dump**. Ten disagree when replayed alone and agree when
every tile is set first — the tile start-state difference of `Mars_RdpFiltering.md` §5.2, not a rule.

### 5.3 How the disputes were found

The first named cases were written from the reference's rules and agreed; the random cases then disagreed in
numbers no single rule explained. **The cause was found by probing, not by reading**: fifteen cases were
written one axis at a time — a span clipped on the left, on the right, a rectangle reaching past the scissor,
a step that is not whole texels, two shapes in one list, a backwards step, a flipped rectangle, a third
mipmap tile, a chain without the level of detail, a widely varying w, a 32-bit texture in an 8-bit image, a
4-bit intensity texture, negative coordinates, a tile masked past its size — and **every one of them agreed**.
Only when a triangle was built with its major edge on the right, which the earlier probes never produced
because the helper's geometry always put it on the left, did the disagreement appear. Those fifteen probes
are kept as named cases: they are evidence about what is *not* disputed, which is most of the mode.

### 5.4 Against Mars

**The first round: fifty-one breakages, thirty-seven caught, seven survived, and seven were never applied** —
a defect in the runner, not in Mars. The list was built through a generator that wrote each pattern's
newlines as two characters, so seven multi-line rules matched nothing and the runner reported it. They were
rerun in the second round. **Three more were caught only by a random case**: the coordinate clamp, the
ten-bit word index, and a YUV or 32-bit texel reading the lower half.

**Ten named cases were added**, four of them for rules no case reached:

- **A 4-bit image's group width, and its alpha compare.** The only 4-bit case wrote zeros over a page nothing
  had written, so neither a missing write nor a wrong mask could show. Two cases now fill the image as a
  16-bit one first and copy over it.
- **A palette entry's bank.** A palette loaded the usual way holds each entry in all four banks
  (`Mars_RdpFiltering.md` §4.2), so reading the wrong one reads the same sixteen bits — but a palette loaded
  from an *odd* address does not, and a case that loads one catches the rule.
- **The tile's format under a palette**, which no palette case reached because none used a YUV tile.

**Three survivors are unobservable, and stay that way:**

- **A group covering fewer pixels than it should.** The group width is only a loop bound: each turn writes
  eight bytes at the pointer and advances it eight, and a turn past the span's end writes nothing because the
  byte count is cut (§1). Making the width smaller therefore only adds turns that write nothing. Making it
  *larger* stops the walk early, and that is caught.
- **The row's nine-bit line.** `((line × t) & 0x1FF) + memory`, shifted up four and cut to thirteen bits, is
  equal to `line × t + memory` shifted and cut the same way: what the inner mask removes is a multiple of
  512, and 512 × 16 is exactly the outer mask's modulus. The mask is the reference's, and arithmetically
  redundant here.
- **A texel reading its bank's first address** (§2.1). A bank is bits 2 and 3 of an address and the word
  index is bits 2 to 11, so **the bank is part of the index**: two addresses in one bank differ by a multiple
  of sixteen bytes, which four texels at most twelve bytes apart — or equal, once masking has folded them —
  never are. The conflict rule therefore always returns the texel's own word. It is kept because it is the
  mechanism, and a slice that changes the addressing would need it.

**The format survivor was answered by making the code say what the survivor proved**: under a palette the
texel is sixteen bits, so neither §2.3's byte rule nor §2.2's wide rule reads the tile's format, and the code
no longer pretends otherwise (§2.4).

**With the second round's reruns and the ten new cases, every breakage but those three is caught by a named
case.**

### 5.5 The baseline, and a test without the tools

**178 cases**: fifty-eight named and 120 random. With the slice removed, **176 fail — fifty-five named cases,
all 120 random ones, and the tool-free test below — and no case from an earlier slice does.** The three named
cases that pass without it are the 32-bit image, which draws nothing either way, and the two 4-bit cases that
write nothing visible: one writes zeros onto a page the fill had left untouched, the other is stopped by
alpha compare, which is the rule it exists to check. Both still catch a breakage that *writes* where the mode
should not.

**A tool-free test** in `MarsRdpTests` copies two rectangles — one plain, one alpha-tested — and an 8-bit
tile into a 16-bit image, then draws a right-major triangle whose spans the mode writes torn (§5.2), and
checks eighteen of angrylion's pixels, four of them half-written. It fails with the slice removed. **Of the
fifty breakages run against it alone it catches twenty**, including the walk's direction and order, the
group's cut, the odd row's swap, the four texels' stride and consecutiveness, bit twelve's half, and the
eight-bit byte rule. What it cannot see are the rules its two mode settings never reach: the palettes, the
4-bit and 32-bit images, the level of detail and the narrow formats.

## 6. What is not here

- **Chroma key** and **noise**, as every page since `Mars_RdpCoverage.md` §6.3 has left them.
- **A dithered alpha threshold**, which reads noise. The reference rotates that threshold for each of the
  four bytes an 8-bit image tests; with the threshold at zero, as Mars takes it, every byte passes.
