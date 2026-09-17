# Mars — coverage, and the one-cycle mode for flat primitives

*Landed 2026-09-17. Phase D's fourth slice: drawing in the one-cycle mode — the mode ordinary
content uses — for primitives that carry no shade, texture or depth. That is where coverage first
becomes visible: it is computed per pixel from the edge walker's sub-scanlines, fed through the
combiner and blender, and stored beside the colour, partly in RDRAM's hidden bits. The code is
`Rdp/Rdp.Modes.cs`, `Rdp/Rdp.Coverage.cs` and `Rdp/Rdp.OneCycle.cs`; the grading is
`MarsRdpDifferentialTests` (`Mars_RdpDifferential.md`).*

*`Mars_Gameplan.md` §2.1 named anti-aliasing as the part of the RDP that could lose the project's
bet on low-level emulation. This page is where coverage is graded, and §6 is where the two
references turned out not to be one oracle.*

---

## 0. Where each rule came from

**Every rule below is the reference's, graded.** Read from angrylion-rdp-plus for mechanism,
written in Mars's own structure, and then run against angrylion with parallel-rdp as the cross-check.
Nothing here is documented at bit level anywhere `Mars_Documentation.md` §4 found: coverage's meaning
and storage are Tier 1, its generation is Tier 2, and the blender's divider and the dither patterns
are established only by this comparison.

**The scope is flat primitives in the one-cycle mode**: the plain triangle command and fill
rectangles, with every other mode and colour register that needs neither texture, depth nor chroma
key. §5 says exactly what that excludes.

## 1. The modes and the registers

The other-modes command now decodes in full, and five commands that set colours and constants act:
the combine mode, the primitive, environment, blend and fog colours, the convert constants (only
`k4` and `k5` are read), the two key commands (only the key centre and scale), and the primitive
depth (only its delta).

**In the one-cycle mode the combiner reads the second cycle's selectors and the blender the first
cycle's.** Mars decodes only those. That is the reference's arrangement, and every case sets both
cycles alike, so a mismatch between the two choices would not have shown; it is recorded as a rule
the cases do not isolate.

The mode flags the reference derives — whether a blend is a partial reject, whether blend shifts
are needed, which dither level applies — are derived at draw time from the current modes. The
reference derives them lazily at the next primitive after a mode changes, which gives the same
values.

## 2. Coverage

### 2.1 Eight samples of sixteen

A pixel is sampled on a four-by-four grid and **eight** samples are kept, two per sub-scanline,
alternating columns between sub-scanlines so that the kept samples form a checkerboard. Each
sub-scanline's two samples are set or cleared by comparing its clipped edges, in eighths of a pixel,
with the pixel: samples wholly inside the span are set, those outside it cleared, and at the pixel
containing an edge the edge's fraction chooses how many of the pair are on the covered side. A
sub-scanline that is invalid clears its samples across the row.

The result is one byte per pixel. Its **count** is the pixel's coverage, zero to eight; its **top
bit** alone is the *coverage bit*, which is what decides whether a pixel is drawn when
anti-aliasing is off.

**The sticky bit is now visible.** `Mars_RdpTriangles.md` §2.3 recorded that the walker's sticky bit
changed nothing a fill could see and was kept for this. It is: an edge's fraction of a pixel rounds
through it, and removing the sticky bit now fails 142 of the RDP tests, almost all of them one-cycle
cases whose edges carry fractions below an eighth of a pixel.

### 2.2 The walker keeps every sub-scanline

The walker now stores each sub-scanline's two clipped edge positions and its validity, which is what
§2.1 reads. The spans themselves are unchanged.

## 3. The framebuffer

Reads and writes now take coverage with them, by colour size.

- **16-bit:** RGBA 5551 read back to 8-bit channels; with image reads on, the stored coverage is the
  pixel's low bit and the two hidden bits beside it. A write packs the blended colour and puts the
  final coverage's top bit in the pixel and its other two in the hidden bits. An intensity image —
  any format but RGBA — uses the pixel's upper byte as colour and bits 7–5 as coverage.
- **32-bit:** the low byte's top three bits are the coverage. The hidden bits take the green
  channel's low bit, for reasons the reference does not state.
- **8-bit:** one byte stands for all three channels on read, and a write takes red at even addresses
  and green at odd ones, with hidden bits from odd-address bytes only.
- **4-bit:** reads are black with full coverage, and a write stores a zero byte.

With image reads off, memory coverage reads as full and memory alpha as `0xE0`.

**What a write stores as coverage** depends on the coverage destination: *clamp* adds the pixel's
coverage to memory's when blending (or takes the pixel's less one when not) and saturates at seven;
*wrap* adds and wraps; *zap* stores seven; *save* keeps memory's.

### 3.1 Hidden RDRAM

**Mars now has hidden RDRAM**: one byte beside each sixteen-bit word, which the CPU cannot see. The
reference compares it and the differential now compares it too. Adding that comparison made **108
existing cases fail at once**, because Mars's fills had never written hidden bits. A fill now sets each word's hidden bits from the low bit of the byte it wrote at
the word's odd address, which is what the reference's three fill routines come to, and all 108
passed again before any one-cycle drawing existed.

The reference initialises its hidden RDRAM to all threes; the dumps flush it to zero before every
case, so what hidden RDRAM holds at power-on is untested here, and Mars starts it at zero.

## 4. The pixel

Per pixel, in order: coverage, dither values, the combiner, a memory read, the blend decision, the
blender, and the write.

### 4.1 The combiner

Each channel is **(A − B) × C + D** in nine-bit signed arithmetic, rounded and clamped to eight bits.
A nine-bit input is negative only when both of its top two bits are set; the multiplier is negative
on its ninth bit alone. An alpha of exactly `0xFF` is carried as `0x100` until dither is added. With
*coverage times alpha* the pixel's coverage is scaled by its alpha; with *alpha from coverage* the
alpha is the coverage instead.

**The combined input is the previous pixel's result**, cleared at the start of each primitive. The
references disagree about this (§6.1).

### 4.2 Alpha compare and the blender

A pixel is dropped if alpha compare is on and its alpha is below the blend colour's, and then if its
coverage (with anti-aliasing) or coverage bit (without) is zero. **A blend happens** when it is forced,
or when anti-aliasing is on and the pixel's and memory's coverage do not together overflow seven.
Without a blend the first input passes through, and with *colour on coverage* and no overflow the
second does instead; an opaque pixel with alpha-weighted inputs is never blended.

The blend weighs the two colour inputs by the first alpha and by the second alpha plus one, in
eighths. Against memory alpha the two weights are shifted by amounts that come from the depth
buffer's comparison; with depth compare off, the first shift is zero and the second is four unless
the primitive's depth slope is steep, which the primitive-depth delta sets.

### 4.3 The divider

Unless the blend is forced, the weighted sum is divided by the sum of the two weights through a
**table of 32,768 quotients produced by a bit-serial divider**: four divisor bits and eleven dividend
bits in, one quotient bit per step, each step adding the divisor or its complement according to the
previous quotient bit. It is not an ordinary division: replacing it with one, in the red channel alone,
fails nine cases. A forced blend skips it and shifts instead.

### 4.4 Dither

Two four-by-four patterns — a *magic square* and a *Bayer* pattern, as the reference carries them —
and a noise source. Colour dither raises a channel to the next multiple of eight when the pattern value
is below its low three bits. Alpha dither is added to the alpha before blending. Under an interlaced
scissor the pattern's row counts fields, not lines.

**Which pattern the alpha uses is not the colour's choice alone.** With colour dither off, a patterned
alpha dither uses the Bayer pattern; with noise colour dither, the magic square, which no graded case
exercises. A case built at the alpha-compare threshold is what showed Mars got the first of these right,
after a breakage that swapped the patterns survived every random case.

`Mars_Documentation.md` §6 recorded that the wiki's dither matrices are uncited and one is not the
textbook ordering its name implies. The patterns here are the reference's, and they are graded, which
is a different claim from either the wiki's or the textbook's.

## 5. What "flat" leaves out

- **Only the plain triangle and fill rectangles draw in the one-cycle mode.** The seven triangle forms
  carrying shade, texture or depth coefficients draw in the fill cycle and **not** here, because their
  coefficients are not yet interpolated; drawing them with zero shade would be drawing them wrong.
- **Shade reads zero**, which is exactly right for these primitives: every coefficient past their edges
  is zero, so the reference's shade and its coverage-based correction are zero too.
- **Unbuilt inputs read zero:** texels, the level-of-detail fraction and noise. Chroma key and depth
  compare and update are ignored. No graded case selects any of them, so none of this is claimed.
- **A fill rectangle's bottom takes its whole last row only in the fill and copy cycles.** In the
  one-cycle mode it does not, which fifteen cases showed when the old unconditional rule was restored.
- ~~**The two-cycle and copy modes** draw nothing.~~ Both draw now: `Mars_RdpTwoCycle.md` and
  `Mars_RdpCopy.md`.

## 6. What the references disagree on, and what the reference is not

**A third implementation was read against §6.1 and §6.2 on 2026-09-17 and confirmed both** — see
`Mars_RdpReferee.md` §3. Both confirmations come from code the N64_MiSTer core's author wrote his own way,
which is the strongest support either rule has: the combined input is a register there too, and the 8-bit
image's byte is chosen by address parity there too. The empty scissor of `Mars_RdpDifferential.md` §4.5 is
confirmed as well, in code transliterated from angrylion, which is worth much less.

### 6.1 The combined input

**parallel-rdp does not reproduce angrylion's one-cycle combined input.** On the first full run, every
disagreeing random case used that input and no case without it disagreed, so it was taken out of the
random cases and kept as one named case, recorded as a dispute. angrylion's value — the previous
pixel's result — follows from its combiner writing into shared state; a GPU implementation evaluating
pixels independently has no previous pixel. Which of the two hardware does is not established. Mars
follows angrylion, as the grader.

### 6.2 An 8-bit forced blend against memory colour

A second, narrower disagreement, found in two random cases and then isolated by changing one setting
at a time in one of them. **It needs an 8-bit image, a forced blend, and memory colour as exactly one of
the two blend inputs**: not forcing the blend, using 16- or 32-bit images, or selecting memory colour as
both inputs all agree, while changing the other input or anti-aliasing does not matter. It also depends
on pixel values: a hand-built case with those settings agreed, so the named case replays the recorded
commands of the one that did not. The differences are in the low bits of green-lane bytes.

### 6.3 Noise is the reference's, not the hardware's

The reference's noise generator carries a comment saying it is taken from a shader example **"to make
noise deterministic in validation"**. It is not a model of the console's noise. Every mode that reads
noise — the combiner's noise input, the noise dithers and a dithered alpha threshold — would grade Mars
against that choice, so none is graded and Mars does not copy it. This is the same kind of finding as the
width clamp in `Mars_RdpDifferential.md` §4.3.

### 6.4 The first disagreements were the test's

The first run showed forty-nine disagreements, and most were not between the references. The test's
own mode builder shifted blender selectors as signed integers, so a selector of 2 or 3 in the top field
sign-extended into the command byte and turned the other-modes command into a different command
altogether. Both references were given malformed streams and disagreed about them. After the fix,
nineteen remained, and every one used the combined input (§6.1). The two cases in §6.2 appeared only
in the random cases generated after that input was taken out.

## 7. What the differential says

**One hundred and seventy-seven one-cycle cases**: twenty-seven named — coverage kept, wrapped,
clamped, zapped and saved; blends over memory colour and memory alpha; colour on coverage; forced
blends; alpha compare; coverage times alpha and alpha from coverage; both dither patterns; the combined
input; the key and convert constants; every image size and an intensity image — and one hundred and
fifty random, drawn from every mode §5 does not exclude, with one or two random triangles and sometimes
a rectangle, over a random background. **Mars matches angrylion on every one**, and parallel-rdp agrees
with angrylion on every one except the two §6 disputes.

**The cases were checked for bite.** With the one-cycle renderer switched off, 142 of the first 173
failed; the rest draw nothing either way. Twenty-one breakages of the pipeline were then run, and
eighteen were caught. Of the three that survived: nine-bit sign extension had no case with a negative
constant, and the alpha pattern with colour dither off had no case sitting at the alpha threshold — one
case each was added and both are now caught. The third, the first blend shift, is always zero without
depth compare, so that breakage changed nothing; the live one is the second shift, and its breakage
survived twice before the case that sees it was right — first because full memory coverage stopped any
blend, then because empty memory coverage made every shift of zero equal — and is caught by a forced
blend over covered memory.

**A tool-free test** in `MarsRdpTests` replays one anti-aliased case and checks seven of angrylion's
recorded pixels and their hidden bits, across a row with a fully covered edge, the interior, a partly
covered edge and untouched background. It fails with the renderer off, and with the edge rounding of
§2.1 broken.

## 8. What is not here

- ~~**Shade and depth interpolation**, and with them the shaded triangle forms in this mode, the
  coverage-based shade correction, and the depth buffer's compare and update.~~ Built in the next slice
  (`Mars_RdpDepth.md`), which also lets the shaded and depth-tested triangle forms draw in this mode, so
  §5's first bullet no longer holds for them.
- ~~**Textures**, the level-of-detail fraction and texture memory.~~ Texture memory, loads and point-sampled
  textures built (`Mars_RdpTextures.md`); the level-of-detail fraction since (`Mars_RdpLod.md`).
- ~~**The two-cycle and copy modes**~~ (built in `Mars_RdpTwoCycle.md` and `Mars_RdpCopy.md`), and
  ~~texture rectangles~~ (built with textures).
- **Chroma key** — which parallel-rdp lists among its own missing features, so a dispute is expected.
- **Noise**, deliberately (§6.3).
- **The video interface**, which is where coverage becomes anti-aliasing on screen.
