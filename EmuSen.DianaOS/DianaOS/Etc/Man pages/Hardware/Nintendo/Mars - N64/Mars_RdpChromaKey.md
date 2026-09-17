# Mars — chroma key: the last combiner cycle's key alpha and bypass

*Landed 2026-09-17. Phase D's eleventh slice: the key a primitive can measure its colour against, which
replaces the pixel's alpha with how near that colour is to the key and passes the cycle's first input
through in place of its result. The code is `Rdp/Rdp.ChromaKey.cs` (the distance and the key alpha),
`Rdp/Rdp.OneCycle.cs` (the bypass, and where the key alpha goes) and `Rdp/Rdp.Modes.cs` (the mode bit and
the two commands' widths); the grading is `MarsRdpDifferentialTests`.*

***§3 is the part to read before trusting anything here: parallel-rdp does not implement chroma keying at
all, so this slice has no cross-check.** Every case is graded against angrylion alone — the first slice of
which that is true.*

---

## 0. Where each rule came from

**Every rule is angrylion's, graded against angrylion.** The reading is the same as the slices before —
mechanism from the reference, written in Mars's own structure — but the instrument is halved:
`Mars_RdpDifferential.md` §1 built the cross-check precisely so that a rule read wrongly from one reference
would show up against the other, and here it cannot. parallel-rdp's README lists *"color combiner chroma
keying"* under its missing features, and its shaders have no key path.

**What that leaves.** The evidence for this slice is: the cases match angrylion; the breakages against Mars
(§3.2), which say the cases can see each rule; and the rules' shape, which is the reference's arithmetic
read carefully. What it cannot say is whether angrylion's arithmetic is the hardware's. The other slices
could point at a second opinion, and this one cannot — an FPGA implementation or a test ROM on a console
would be the way to settle it.

## 1. What keying changes

**One mode bit** — bit 40 — and **two commands**, which carry three channels of key in three parts:

| Command | Width | Centre | Scale |
|---|---|---|---|
| Set Key R (`0x2B`) | bits 16–27 | 8–15 | 0–7 |
| Set Key GB (`0x2A`) | green 44–55, blue 32–43 | green 24–31, blue 8–15 | green 16–23, blue 0–7 |

**The centre and the scale are combiner inputs** — colour B selector 6 and colour multiplier selector 6 —
so a keyed primitive combines `(colour − key centre) × key scale`, and the cycle's result is the *distance*
from the key, scaled. Mars carried both inputs from the one-cycle slice; the width is this slice's, and
nothing but the key reads it.

**With the bit set, the last combiner cycle changes in two ways**, and only the last: in the two-cycle mode
the first cycle combines as it always does.

- **The pixel's colour is the cycle's own first input** — the colour A selector's value, clamped to eight
  bits — instead of the cycle's result. The key's arithmetic is a measurement, not a colour, so the colour
  the pixel keeps is the one the cycle started from.
- **The pixel's alpha is the key alpha** (§2), in place of the combined alpha and its dither. Alpha from
  coverage still wins over both, and coverage-times-alpha still scales the coverage by the combined alpha
  before any of this, as `Mars_RdpCoverage.md` §4.1 describes.

**The combined colour is unchanged**: the next cycle, and the next pixel's first cycle in the two-cycle
mode, still read the cycle's result, not the bypass.

## 2. The key alpha

**Each channel gives a distance, and the narrowest wins.** For a channel's seventeen-bit combined value:

1. **Take it as signed.** The value is the combiner's, before the shift down eight that makes a colour.
2. **A positive value counts back**: it becomes its own negation — except that a value whose low nibble is
   exactly eight becomes `0x10 − value`, which is the negation rounded the other way. A negative value is
   left as it is.
3. **Add the channel's width, in sixteenths** — the twelve-bit width shifted up four.

**The key alpha is the smallest of the three, clamped to 0…255.** A colour at the key's centre gives a
distance of zero in every channel, so the alpha is the narrowest width; a colour far from the key gives a
large negative distance, so the alpha is zero.

## 3. What the differential says

### 3.1 One reference, and what a first run is worth here

**Seventy-seven cases — seventeen named and sixty random — matched angrylion on the first run, and every
one of them is graded against angrylion alone.** On the pages before, a clean first run was worth little on its
own and the cross-check carried the weight; here there is no cross-check, so the first run is worth less
still. §3.2 is the only evidence that the cases can see these rules at all.

The named cases key on a texel's distance, on shade's, and on the primitive colour's; pass the first input
through where the key's arithmetic would otherwise be the colour; set every width to zero and to its widest;
key on one channel while the others pass everything; drop the scale so every distance is the centre's; hit
the low-nibble rounding of §2 head on; take alpha from coverage and coverage times alpha, where the key alpha
is not used; dither the alpha, which the key replaces; test the key alpha with alpha compare; key in the
two-cycle mode, and in a two-cycle primitive whose *first* cycle reads the key inputs, which the key does not
touch; and key in the fill and copy cycles, which have no combiner. The random cases draw one or two shaded,
textured or depth-tested triangles through random keys, random combiner selectors weighted towards the key
centre and scale, in one and two cycles, with random dither, coverage and alpha modes.

### 3.2 Against Mars

**Seventeen breakages, sixteen caught, one survived.** The survivor was §2's rounding — a distance whose low
nibble is exactly eight, which negates to `0x10 − value` rather than `−value`. No case reached it, because
reaching it needs a positive distance with that nibble, and the nibble follows the key scale: at a scale of
eight, every channel an odd distance from the centre lands on it. A named case with that scale, a low centre
so most texels sit above it, and a width that keeps the alpha in range now catches it. **All seventeen are
caught by a named case.**

Nothing survived as unobservable, and nothing was caught only by a random case.

### 3.3 The baseline, and a test without the tools

**With the slice removed, 69 tests fail — fifteen named cases, fifty-three of the random ones and the
tool-free test below — and no case from an earlier slice does.** The two named cases that pass are the
fill-cycle and copy-mode ones, which is what they are for: they show the key changes nothing where there is
no combiner, and they would fail if Mars keyed there. The seven random cases that pass were not examined one
by one; a random key can leave every pixel outside it, where a keyed primitive draws what an unkeyed one
would.

**A tool-free test** in `MarsRdpTests` draws one keyed triangle over a texture, with the key's three widths,
centres and scales set so that the alpha runs the whole way from nothing to full across the shape, and checks
eighteen of angrylion's pixels. It fails with the slice removed. **Of the seventeen breakages run against it
alone it catches twelve.** The five it does not see are instructive: alpha from coverage and the two-cycle
bypass, which its one mode setting never reaches; the rounding of §2, which needs a key scale it does not
use; the green width's field, because a wrong field there still leaves green wider than the channel that
decides the minimum; and the mode bit itself, because the bit below it is one of the dither bits, which this
test sets — so moving the read one place down still reads a one.

## 4. What is not here

- **Noise**, and with it the dithered alpha threshold, as every page since `Mars_RdpCoverage.md` §6.3 has
  left them. Phase D's remaining gap is that one.
- **The key in the copy and fill cycles**, which have no combiner and so no key. Two cases show it: they
  draw the same with the bit set as without.
