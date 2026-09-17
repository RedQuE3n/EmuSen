# Mars — triangles, and the edge walker that turns their words into spans

*Landed 2026-09-17. Phase D's third slice: the part of the display processor that decides which
pixels a primitive covers — its edge walker — built for triangles and drawn in the fill cycle,
the one mode that needs no combiner or blender. Fill rectangles now go through the same walker.
The code is `Rdp/Rdp.Walker.cs`; the grading is `MarsRdpDifferentialTests`
(`Mars_RdpDifferential.md`). Nothing here shades, textures or depth-tests a pixel (§5).*

---

## 0. Where each rule came from

| tier | source | what it settles here |
| --- | --- | --- |
| **corpus layout** | the corpus's triangle command assembler (`rdp/rdp_assembler.rs`) | where each field of a triangle's first four words sits (§1) |
| **reference, graded** | angrylion's edge walker, read for mechanism and rewritten, then graded by the differential with parallel-rdp agreeing | every rule in §2 and §3 |

The corpus has triangle tests of its own, behind a feature flag the corpus ROM here is not built
with. They are not usable as a grader: they draw in the one-cycle mode, which needs the blender,
and the CPU model they compare against marks itself unfinished — its coverage table ends in *"placeholder
until all are filled out"*.

No commercial list has exercised a triangle yet. Wave Race's only graphics task so far is fills
(`Mars_Rdp.md` §8), and Mario submits none (`Mars_Microcode.md` §6).

## 1. The command

Eight command numbers, `0x08`–`0x0F`, are triangles. Every one begins with the same four words,
and adds shade, texture and depth coefficients after them by its low three bits
(`Mars_Rdp.md` §3). Only the four are read here.

- **Word 0:** bit 55 chooses which edge is the major one; three fourteen-bit signed y values in
  quarter pixels — the bottom (`yl`, bits 45–32), the middle (`ym`, 29–16) and the top (`yh`,
  13–0).
- **Words 1–3:** an x in the high half, signed with sixteen fraction bits, and its slope per pixel
  of y in the low half, signed with sixteen fraction bits: the lower minor edge (`xl`), the major
  edge (`xh`), and the upper minor edge (`xm`), in that order.

**Bit 55 set means the major edge is on the left.** The corpus's assembler names the bit *right
major*, and its own model of the part treats a set bit as putting the major edge on the left too;
the reference and the differential agree with the behaviour, not the name.

**In the fill cycle every triangle command draws alike.** The shaded, textured and depth-tested
forms carry coefficients the fill cycle never reads; cases for three of them match.

## 2. The walk

### 2.1 Edges are stepped from the top of a row, in quarter pixels

The two edges are evaluated once per **sub-scanline** — a quarter of a pixel row — starting at the
top of the row that contains `yh`, **not at `yh` itself**. So `xh` and `xm` are the edges' x at that
row's top, and each step adds the slope divided by four with its lowest bit cleared. The cleared
bit is one part in 65,536 of a pixel per sub-scanline; a case built so that the difference crosses
a pixel boundary part-way down an edge shows it (§4.2).

The major edge runs the whole height. The minor edge starts as the upper one and becomes the lower
one — its x replaced by `xl`, its slope by the lower slope — **at the sub-scanline exactly equal to
`ym`**. A middle y above the row the walk starts in is never met, and the upper edge runs on to the
bottom; a case with the top and middle swapped draws exactly that.

### 2.2 Which rows

The walk's top is the larger of `yh` and the scissor's top, and its bottom the smaller of `yl` and
the scissor's bottom, with two exceptions taken from the fields' high bits: a negative `yh` starts
at the scissor's top and one of 1,024 rows or more stands; a negative `yl` stands and one of 1,024
or more stops at the scissor's bottom. Rows from the top's row to the bottom's row are candidates.

A sub-scanline is **valid** if it is at or below the top, above the bottom, and its right edge is
not left of its left edge at quarter-pixel precision.

### 2.3 Where each edge lands

Each edge's x is taken to **eighths of a pixel**, with a sticky bit recording any smaller fraction.
It is **under** the scissor if it is negative or left of the scissor's left edge, and is then moved
onto that edge; the result is **over** if it is at or past the scissor's right edge or past 1,024
pixels, and is then moved onto the right edge. The pixel is the eighths divided by eight.

**The sticky bit changes nothing a fill can see.** It can only raise an even count of eighths by one,
which never changes the pixel and never changes a comparison against the scissor's even edges. It
is kept because coverage, which this page does not build, reads the eighths themselves, and a
mutation that removes it is recorded in §4.3 as surviving for that reason.

### 2.4 Each row's span

A row's span runs from the **leftmost** left-edge pixel to the **rightmost** right-edge pixel over its
valid sub-scanlines — the union of its sub-scanlines, not any one of them. The row is dropped if none
of its sub-scanlines is valid, if both edges were over on every sub-scanline, or if both were under
on every one, and, with the scissor's field bit set, if its parity is not the one kept.

**Two things the reference does that Mars does not, because no fill can observe them.** The reference
walks one row further when the primitive continues below a scissor, and records that row as not
drawn; that bookkeeping matters to texture level of detail, which reads the row after a span, and is
to be restored when textures are. And Mars clears the drawn flags of the rows it is about to walk
before walking them: a guard, since every row returned is written, which a mutation confirms by
changing nothing.

## 3. Rectangles are walked

A fill rectangle is now given to the same walker, rewritten as the reference rewrites it: the major
edge on the left at the rectangle's left, both minor edges at its right, no slope, and in the fill
and copy cycles the bottom taken to the end of its row. The rule `Mars_RdpDifferential.md` §4.2
states for rectangles is what this walk does with those inputs, and the forty-nine rectangle cases
still match.

Whether the hardware has a separate path for rectangles is not known here; the reference's structure
is followed because it is graded, not because it is the silicon's.

## 4. What the differential says

### 4.1 The cases, and the first run

Twenty-two named triangles — both major sides, a flat top and a flat bottom, fractional vertices, a
triangle thinner than a pixel, shallow and steep edges, triangles starting above and left of the
image, scissors cutting every edge and fractional scissors, an interlaced scissor, 32- and 8-bit
images, three attribute-carrying forms, two malformed ones, one reaching past x 1,024, and one built so
that a step's cleared low bit crosses a pixel — and sixty random triangles from a
fixed seed, a third of them under a random scissor.

Every image is 32 wide under a scissor ending at column 31, so no span reaches the reference's width
clamp (`Mars_RdpDifferential.md` §4.3). **parallel-rdp agrees with angrylion on every one.**

The first eighty cases were run against Mars as it stood, drawing no triangles, and seventy-five
failed: every one the reference draws. The five random triangles the reference leaves empty passed.
With the walker built, and the two cases §4.3 added, **all eighty-two match**, and the forty-nine
rectangle cases still do.

### 4.2 What the unusual cases draw

Recorded because each exercises a rule an ordinary triangle does not:

| case | angrylion draws |
| --- | --- |
| major-edge flag inverted | one pixel, (4,2), where the edges meet at the top; every other sub-scanline has them crossed |
| middle vertex's y above the top | the upper edge running to the bottom, widening to the scissor (§2.1) |
| a vertex at x 1,100 | spans running to the scissor's right edge on every row |
| a step with its low bit set | the right edge reaching column 9 at row 10, as predicted before the run |

### 4.3 The instrument was checked for bite

Fifteen breakages of the walker were run. Ten were caught at once, one of them by failing every case
in the dump, rectangles included. Of the five survivors:

- **The sticky bit** and **the extra row below a cut scissor** are invisible to a fill (§2.3, §2.4).
  The first is kept for coverage; the second was removed. **Update:** with coverage built, removing the
  sticky bit fails 142 RDP tests (`Mars_RdpCoverage.md` §2.1).
- **Clearing stale spans** is a guard the walk never needs (§2.4), and is kept.
- **The over test for x past 1,024 pixels** and **the cleared low bit of a step** had no case that
  could see them. One case each was added, and both breakages are now caught.

### 4.4 A test that needs no reference

The differential returns early on a machine without the tools. `MarsRdpTests` therefore carries one
triangle — the words and the twenty-four spans angrylion drew for the left-major case, copied from a
recorded run — so the walker is pinned everywhere. It fails with triangle commands ignored.

## 5. What is not here

- **Shading, texturing and depth.** The attribute coefficients are skipped, and the one- and
  two-cycle modes that would use them draw nothing (`Mars_Rdp.md` §5.3).
- **Coverage and anti-aliasing.** The walker keeps no per-sub-scanline edge positions; the reference
  stores four per row for coverage, and that is where the undocumented sub-pixel mask of
  `Mars_Documentation.md` §4 lives.
- **The copy cycle**, and texture rectangles.
- **Pipeline crashes** on fills with image reads or depth settings (`Mars_Rdp.md` §5.3).
- **Commercial evidence.** No list has reached a triangle yet (§0).

> **Update 2026-09-17:** coverage and the one-cycle mode now exist for the plain triangle command, and
> the walker keeps its per-sub-scanline edges for them (`Mars_RdpCoverage.md`). The two bullets above on
> shading and coverage still hold for every triangle form that carries shade, texture or depth.
