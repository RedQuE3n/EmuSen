# Mars — the anti-aliasing filter: coverage, six neighbours and two runners-up

*Landed 2026-09-17. Phase E's second slice, and the first thing in Mars to read a coverage back out of memory
rather than write one into it. The code is `Vi/Vi.Filter.cs` (the filter) and `Vi/Vi.Scanout.cs` (the fetch
that now carries a coverage, and the fetch bug); the grading is `MarsViDifferentialTests`.*

*The FPGA implementation matters more here than anywhere so far, and §5 says why: it implements this exact
slice in a vocabulary of its own, so for once its agreement is worth something — and it turns out to prove
two things about the software reference that the software reference's own shape hides.*

---

## 0. What grades this

**The instrument is the one `Mars_Video.md` §0 built, with one addition.** `RDPDUMP2` has always carried a
record that uploads *hidden* memory — the ninth bit of each RDRAM byte, which is where a sixteen-bit pixel
keeps two of its three coverage bits — and, as with the video interface's registers a slice ago, the C#
writer had never emitted it. `RdpDump.UploadHidden` does now. It is the only upload in the format that is
**not** byte-swapped and **not** addressed by byte: the hidden array holds one byte per sixteen-bit word and
is indexed by that word, on both sides, which is why `Mars_RdpDifferential.md` §2's comparison of hidden
pages has always been able to work page for page without a swizzle.

**parallel-rdp's `vi-conformance` still passes all twenty-four suites**, including its anti-alias matrix, so
the two software references agree about this device as a whole. As in the slice before, that makes a Mars
disagreement a Mars bug rather than a dispute, and it makes the cross-check weaker than it sounds: the
agreement is established over *their* random frames, not over Mars's cases.

## 1. The coverage a pixel carries

**Every pixel in a frame buffer carries three bits of coverage beside its colour, and where those bits live
depends on the format.**

- **A thirty-two-bit pixel** keeps them in the top three bits of its fourth byte: `(byte3 >> 5) & 7`. The
  rest of that byte is not read.
- **A sixteen-bit pixel** keeps the *high* bit in the word itself — bit 0, the one the 5-5-5-1 layout calls
  alpha — and the *two low* bits in RDRAM's hidden ninth bits, one byte per word, which no processor
  instruction can reach. The coverage is `((word & 1) << 2) | hidden`. Mars has carried that hidden array
  since the coverage slice (`Mars_RdpCoverage.md` §3.1); this is the first thing outside the display
  processor to read it.

**Anti-alias modes 2 and 3 read no coverage at all** and take every pixel as whole. Mars tests `AntiAlias >
Covered`, where `Covered` is 1; the FPGA tests bit 1 of the mode and forces the coverage to seven, which is
the same test said in hardware.

**The coverage reaches the screen.** The raster's fourth byte per pixel is the coverage the scan read, not a
constant — the slice before wrote seven there because modes 2 and 3 are all it could scan. Mixing moves the
colour and leaves the coverage alone, so a resampled pixel carries the coverage of the pixel the step landed
on and none of its neighbours'.

**A read past the end of memory is a whole zero**: black, and coverage zero. Coverage zero is not "no
coverage" but "as partly covered as a pixel can be", so such a pixel is filtered rather than skipped — and
since its neighbours are usually past the end too, the filter finds nothing to pull it towards and leaves it
black. The effect is invisible; the path is not.

**Mars does not mask the hidden byte to two bits**, although only two exist. Every writer of that array
inside Mars already masks — the fill, the copy, the one-cycle write and the depth write all store `& 3` or
`(x & 1) * 3` — so a third bit cannot be there to read, and a mask on the read would state a fact that the
writes already guarantee. The reference does not mask either.

## 2. The filter

**A pixel whose coverage is seven is left alone. Any other pixel is pulled towards the whole pixels around
it**, by an amount proportional to the coverage it lacks. That is the whole of anti-aliasing at this stage of
the machine: the display processor already blended the edge into the frame buffer and recorded how much of
the pixel the polygon covered, and the video interface finishes the job on the way out.

### 2.1 The six neighbours

**Six, not eight**, and they are not the eight around the pixel:

| | | | | |
|---|---|---|---|---|
| | ↖ | | ↗ | |
| ← | | **·** | | → |
| | ↙ | | ↘ | |

- the row above, one column either side;
- **this row, two columns either side** — not the pixels next door, the ones beyond them;
- the row below, one column either side.

The two-column reach along the row is the surprising one and it is not an accident of either software
reference: the FPGA gathers its six from three five-wide shift registers as positions 1 and 3 of the rows
above and below and positions 0 and 4 of this one (§5).

**Only the whole neighbours count.** A neighbour whose coverage is under seven is dropped, so the filter
weighs between one and seven values: the pixel itself, always first, and however many of the six survived.

### 2.2 The two runners-up

**The filter does not use the extremes of that list but the runners-up: the second smallest and the second
largest.** The reference finds them with a single pass that tracks the leader at each end and remembers the
value the leader displaced, then rescans whatever came after the leader. Mars does the same, and the shape
matters, because it is not quite what "second smallest" means:

> **A leader that is never displaced stands as its own runner-up.** The pixel itself is first in the list, so
> when the pixel is darker than every whole neighbour, the low runner-up is the pixel's own value — not the
> darkest neighbour. The filter can then only pull it brighter.

This is the rule to keep in mind when reading a filtered picture: the pull is always towards a pair of values
that **bracket the pixel**, so the filter never moves a pixel outside the range its neighbourhood already
spans, and never overshoots.

**Two parts of the reference's construction were proved to decide nothing and are not in Mars.**

- The reference's pass writes the low runner-up in an `else` of the high one, so a value that takes the lead
  at the top is never offered to the bottom. It cannot matter: the low leader's value never rises above the
  high leader's, so no single value can be a new leader at both ends. Mars writes two independent tests.
- The reference guards each rescan with "only if the runner-up is not already the leader's own value". It
  cannot matter either: if the runner-up already equals the leader, nothing in the rescan can exceed it.

**Both were checked rather than argued.** The filter weighs between one and seven values, and the pass
compares values without arithmetic, so its result depends only on the *order pattern* of the list — and every
order pattern of *n* values is realised by values drawn from `0..n-1`. Enumerating all 873,612 of them for
`n` from one to seven, the reference's construction and Mars's agree on every one. `Mars_Video.md` §3.3
retired an unobservability argument that did not name the range it held over; this one names it, and the
range is the whole domain.

### 2.3 The pull

With `low` and `high` the two runners-up and `missing = 7 − coverage`:

```
pixel + (((low + high − 2 × pixel) × missing + 4) >> 3)
```

**The bracket in the middle is twice the distance from the pixel to the midpoint of its two runners-up**, and
the multiply and shift take `missing/8` of it, rounding by the half step that the `+ 4` provides. A pixel of
coverage 6 moves an eighth of the way to that midpoint; a pixel of coverage 0 moves seven eighths. Coverage 7
never arrives here.

**The result cannot leave a byte**, and this is worth stating because both references mask it to one anyway.
Since the runners-up bracket the pixel (§2.2), the bracketed term lies in `[−pixel, 255 − pixel]`, and
seven eighths of that keeps the sum inside `[0, 255]` for every input. Mars had the mask, the breakage round
found no case that could see it, and it came out — the mask is not a rule about the hardware but an artefact
of writing the arithmetic in a wider type than it needs.

## 3. The fetch bug

**When a row of the picture reads the same line of the frame buffer as the row before it, the row *after*
that one filters its lower neighbours out of its own line instead of the line below.** The reference names
the state `fetchbugstate` and carries it from row to row: set to 2 when this row's successor reads the same
line, halved otherwise, and the artefact fires on the value 1 — one row after the repeat, not on the repeat
itself.

**It reaches only the second of the two lines a resampled pixel mixes.** The line the pixel sits on is always
fetched clean; the line below it — the vertical mixing partner — is the one whose filter neighbourhood goes
stale. A picture with a vertical step below a whole line therefore has one row in every repeat cycle whose
lower mixing partner was filtered against the wrong neighbours.

**This is the one rule in the slice the FPGA contradicts** (§5), and the only one with a plausible route to
being wrong.

## 4. What the differential says

### 4.1 The cases

**One hundred and thirty cases: fifty named and eighty random, of which every one matched angrylion on the
first run.** That is not the usual result and it is worth saying what it does and does not mean. The
mechanism here is small and entirely arithmetic — no state carries between frames, unlike the raster of the
slice before, which is where the previous slice's one defect lived. A first-run pass says the arithmetic was
read correctly; it says nothing about the parts of this device that are still not built.

**Twenty of the named cases are new**, and they are chosen to separate things the arithmetic cannot
separate on its own:

- Both anti-alias modes, in both pixel formats, over a frame buffer whose hidden bits are random, so that
  neighbourhoods of every size from one to seven occur.
- **Every pixel whole**, which must come out identical to the resample-only mode, and **no pixel whole**,
  where the filter has only the pixel itself to weigh and must leave it exactly alone. These two bracket the
  filter: one proves it does not fire when it should not, the other proves its degenerate case is the
  identity and not something that merely looks like it.
- **The hidden bits held at 3 with the word's own bit left random**, so a pixel is whole exactly where the
  colour word says so — which is the case that separates the two halves of a sixteen-bit coverage.
- A vertical step of half a line and of three quarters of a line, for the fetch bug, and a step of none,
  which repeats every row and so never fires it.
- A narrow and a wide frame buffer, because the frame buffer's width is the filter's vertical reach.
- A frame buffer at the very start of memory, where the row above underflows.

### 4.2 Against Mars

**Eighty-two breakages, of which seventy-eight apply to the code as it stands, and every one of the
seventy-eight is caught by a named case**, none only by a random one. Three of the four that do not apply
are the rules `Mars_Video.md` §3.3 removed a slice ago; the fourth is §2.3's mask, removed by this round and
described below.

**The list is deliberately unkind to the filter.** Each of the six neighbour positions is moved
independently, each half of a sixteen-bit coverage is swapped with the other, the runner-up pass is turned
into a plain minimum and maximum, each rescan is started from the wrong place, and each of the four
constants in the pull is changed on its own. The two rules of §2.2 that look most like accidents of how
the reference is written — *a runner-up is only written when its leader moves* — are caught by fifteen
named cases each. The quirk is not decoration.

**Two survived, and both are dead rather than unwatched.** The distinction matters: a survivor is normally
a gap in the cases, and these are not.

- ~~*The pull is masked to a byte.*~~ **Removed.** Since the runners-up bracket the pixel (§2.2), the
  bracketed term lies in `[−pixel, 255 − pixel]` and seven eighths of it can never carry the sum out of a
  byte. No case can see the mask because no input exists that the mask would change. This is the argument
  `Mars_Video.md` §3.3 asks for and did not get last time: it names its range, and the range is every input.
- *A pixel of coverage seven is not filtered.* **Kept, and not graded here.** The filter is the identity at
  coverage seven — `missing` is zero, so the pull is `(0 + 4) >> 3`, which is zero — so in this slice the
  test decides nothing about any pixel's value. What it decides is that six neighbours are not read, which
  no case can observe and every frame pays for. It stops being free next slice: coverage seven is exactly
  the condition that selects the dither filter, and the same test will then choose between two different
  answers rather than between one answer and itself.

### 4.3 The tool-free test

**`MarsViTests` gained six scans and twenty-three of angrylion's pixels, and now catches sixty-five of the
seventy-eight breakages** — where a slice ago it caught thirty-three of forty-five. The six are both
anti-alias modes, both pixel formats, a fractional step in each direction, a vertical step of half a line for
the fetch bug, and the same picture scanned twice more with every hidden byte held at 3 and then at 0, so
that one frame is mostly whole pixels and the next has none. It passed on its first run.

**One of the twelve it used to miss is now caught, and by accident.** *The mix rounds by half a step* — the
`+ 16` in the interpolation of `Mars_Video.md` §2.6 — needed a fractional step, and the anti-aliased scan
that has one was added for an unrelated reason. Rounding is caught by a test that was not aiming at it,
which is the ordinary way a rounding rule is caught.

**Thirteen it still misses, and they fall into five classes:**

- **Five are geometry** the test's register sets do not separate: the vertical offset and its halving,
  stepping into a picture that starts above the field, the cut at the raster's edge, and the two
  active-line rules. Unchanged from the slice before, and each is held by a named differential case.
- **Four are border and fade rules** that need a picture narrower or shorter in one particular way. Also
  unchanged.
- **One is replication itself.** Removing the test that skips interpolation in mode 3 changes nothing here,
  because every replicated scan in this test uses whole steps — and interpolating by a fraction of zero
  returns the nearer pixel. The rule needs a replicated scan with a fractional step, which this test does not
  have and the differential does.
- **One is the coverage a dark column keeps** in the guard band.
- **Two are new, and both are rules about the filter that no test can see.** *The pull rounds by half a step*
  survives here and is caught by only three differential cases, all thirty-two-bit: the `+ 4` changes a
  channel only when the eighth it rounds lands exactly on a half, which a five-bit colour reaches too rarely
  to show. *A pixel of coverage seven is not filtered* survives everywhere, for the reason §4.2 gives.

**What the number means and does not.** Sixty-five of seventy-eight is a statement about what survives on a
machine where the reference tools were never built — it is not a second grading. Every one of the thirteen is
held by a named case in `MarsViDifferentialTests`, so nothing here is ungraded; what is missing is only the
guarantee that outlives the instrument.

## 5. The referee

**This is the first slice where the FPGA implementation is a peer rather than a witness.** The reading rule
of `Mars_RdpReferee.md` §0 is that the N64_MiSTer core's agreement is weak wherever its RTL carries
angrylion's identifiers, because then the two are one implementation in two languages. Nothing here carries
them. `VI_filter.vhd` and `VI_filter_pen.vhd` name their signals `AA_sort_in1`, `penmin_sort`, `inv_c`,
`proc_pixels_AA`; the arithmetic is built from VHDL `signed` vectors and bit slices; the structure is a
pipeline with named stages. **Its agreement is therefore evidence, and it agrees about everything in this
slice but one.**

**What it confirms, in its own vocabulary:**

- **Where a coverage comes from.** `VI_lineProcess.vhd` §102 builds a sixteen-bit pixel's coverage as
  `fetchdata16(i)(0) & fetchdata9(i)` — the word's own bit as the high one, the *ninth-bit* memory as the two
  below — and §97 builds a thirty-two-bit pixel's as `fetchdata(i)(31 downto 29)`. §1's two rules, including which end
  each half goes.
- **That modes 2 and 3 read none.** §105–106: `if (VI_CTRL_AA_MODE(1) = '1') then fetchArray(i).c <= "111"`
  — the same test on the same bit.
- **The six neighbours, and the two-column reach.** §114–119 takes them from three five-wide shift registers as `fetchshift0(1)`, `fetchshift0(3)`, `fetchshift1(0)`, `fetchshift1(4)`, `fetchshift2(1)`,
  `fetchshift2(3)`, with the pixel itself at `fetchshift1(2)` (§112). The rows above and below are sampled one
  column either side and this row two columns either side, which is §2.1 arrived at by someone laying out
  registers rather than someone writing array indices.
- **The arithmetic, constant for constant.** `VI_filter.vhd` §157–163: `pendiff = penmin + penmax − (mid << 1)`,
  `diff_mul = pendiff × inv_c` where `inv_c` is `7 − mid.c` (§194), `diff_add4 = diff_mul + 4`, and the result is
  `mid + diff_add4(10 downto 3)`. That is §2.3 including the `+ 4` and the shift by three.
- **That the filter runs only below coverage seven.** §206: `elsif (stage0_Mid.c < 7 and VI_CTRL_AA_MODE(1) = '0')`.

**What it proves, which is more than agreement.** The FPGA finds its two runners-up a completely different
way: it replaces every neighbour that is not whole with the pixel's own colour, sorts the six as two triples
through a decision tree, takes the second smallest and second largest of the six, and then **clamps each
against the pixel** — `penmin = min(centre, penmin)`, `penmax = max(centre, penmax)`, `VI_filter_pen.vhd`
§171–183. There is no running
leader, no rescan, and no order dependence at all.

The two constructions are the same function. Enumerating every configuration of a pixel and six neighbours
that are each either absent or one of seven values — 1,835,008 of them, which covers every order pattern the
seven-value list can take — the reference's pass and the FPGA's sort agree on all of them.

**That equivalence is what makes §2.2's rule statable.** The reference's loop leaves "a leader that is never
displaced is its own runner-up" as a fact about the order values arrive in; the FPGA says the same thing as a
clamp, in one line, without reference to order. It is also what proves the mask in §2.3 dead: the clamped
form makes `low ≤ pixel ≤ high` obvious, and everything else follows.

**What it contradicts is the fetch bug of §3.** The FPGA holds three line buffers and advances them by the
difference between successive source lines, so when a line repeats the window simply does not move and the
rows above and below stay right. There is no second cache to go stale, and nothing in `VI_linefetch.vhd`
reproduces the artefact under any name. So both software references model a hardware defect that the FPGA
either did not encounter or did not judge real.

**Mars follows angrylion, because angrylion grades it.** The direction of the disagreement is the informative
one, though, and this rule is now the first thing in Phase E to test on a console: scan a picture with a
vertical scale under `0x400` in anti-alias mode 1 and read back the row after a repeat. The name angrylion
gave the state is itself evidence — one does not call a variable `fetchbugstate` without having seen
something — but it is not evidence that the model of it is right.

**What none of the three settles is the difference between anti-alias modes 0 and 1.** The mode's name in
every document is "always fetch the extra lines" against "fetch them only if needed", which is a statement
about memory bandwidth; all three implementations test only bit 1 of the mode and treat 0 and 1 alike. Three
implementations agreeing that the pixels are the same is **not** evidence that the modes are the same on
hardware. It is evidence that none of the three models whatever the difference is, and if it is a timing
difference, nothing here could see it.

## 6. What is not here

- **The dither filter**, which is what a *whole* pixel gets instead when control bit 16 is set: eight
  neighbours, each nudging the pixel by one step up or down per channel. The FPGA calls it `dedither` and
  builds it from the same shift registers, so the neighbourhood is already in front of us.
- **Divot**, a median of three consecutive output pixels applied when any of the three is not whole, and
  **gamma** and its dithering, which sit past everything else on the way out.
- **The per-scanline registers**, the interrupt, and the timing, as `Mars_Video.md` §4 has them. Nothing
  advances a half line yet, so no game drives this device.
