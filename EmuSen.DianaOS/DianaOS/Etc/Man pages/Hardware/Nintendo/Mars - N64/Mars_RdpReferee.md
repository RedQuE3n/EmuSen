# Mars — the referee: an FPGA implementation read against Phase D's recorded disputes

*Written 2026-09-17. Not a slice: no rule of Mars changed and no case was added. Phase D grades against
angrylion and cross-checks against parallel-rdp; where the two disagree the disagreement is recorded as a
**dispute** and Mars follows angrylion. This page asks a third implementation — the N64_MiSTer core's VHDL —
what it does for each of those twenty-one rules, and writes its answer beside theirs.*

***§0 is the part that decides what the rest is worth.** The referee is a reimplementation, not a netlist,
and parts of it were plainly written with angrylion open. Its agreement with angrylion is therefore weak
evidence and its disagreement is the informative direction — the opposite of how a second grader would read.*

---

## 0. What the referee is, and what its answers are worth

**The checkout.** `github.com/MiSTer-devel/N64_MiSTer`, branch `main`, head `adbf9b5` (2026-08-15), cloned to
`~/Projects/n64-mister-reference`. The display processor is about ten thousand lines of VHDL in `rtl/RDP*.vhd`.
It is synthesisable register-transfer logic that runs the real command stream on real hardware, which is why it
was worth asking; it is **not** a dump, a netlist or a decapping. No public dump of the RCP exists.

**It is a reimplementation, and not an independent one everywhere.** Three things bound how much weight to
give it:

- **Borrowed vocabulary.** The rasteriser carries angrylion's identifiers almost unchanged — `xlsc`, `xrsc`,
  `curover`, `curunder`, `allover`, `allunder`, `allinval`, `invaly`, `unscrx`, `sticky` (`rtl/RDP_raster.vhd`
  §466–505). So do the level-of-detail unit (`lodfrac`, `magnify`) and the blender's memory-alpha shifts
  (`blend_shift_a`/`blend_shift_b`, from angrylion's `blshifta`/`blshiftb`). Where the names are shared the
  algorithm is shared, and agreement says only that the transliteration was faithful.
- **Its own structure elsewhere.** The combiner, the framebuffer read and the write path are the author's own
  shape — staged pipelines with explicit registers, named nothing like the reference. Agreement there is worth
  much more, because the rule had to be arrived at rather than copied.
- **Unfinished corners, marked as such.** The core reports what it has not built: `error_texMode`,
  `error_drawMode`, `errorCombine`, `error_combineAlpha` are outputs wired up to the top level
  (`rtl/n64top.vhd` §708–711), and the source carries `-- todo:` where a mode is missing. A referee that says
  "not implemented" in its own voice is more useful than one that guesses, and ten of the twenty-one
  disputes get exactly that answer.

**So the reading rule for this page is asymmetric.** Where the referee sides with angrylion *and* the code is
the author's own, that is real support. Where it sides with angrylion in borrowed code, it is nearly no
support. Where it sides with parallel-rdp, that is worth attention whatever the provenance, because it means
somebody building the rule for hardware did not find angrylion's version necessary. And where it implements
neither, the dispute stands exactly where it stood.

**What this pass cannot do.** It cannot measure the hardware. Nothing here is a console result. Every answer
below is one more opinion about what the console does, from a source that has the advantage of having to run
games in real time on a real FPGA and the disadvantage of being written by reading the same sources Mars was.

## 1. Method

Each dispute recorded in `MarsRdpDifferentialTests` — the `dispute:` strings, twenty-one distinct rules — was
carried to the VHDL, the relevant path read, and the answer classified:

| Verdict | Meaning |
|---|---|
| **angrylion** | the referee implements angrylion's rule |
| **parallel-rdp** | the referee implements parallel-rdp's |
| **third** | it implements a third thing |
| **silent** | it implements neither: the mode is missing, errored or held |

No case was run, no simulation built and no Mars code touched. This is a reading, and every row names the file
and line so it can be checked or refuted.

## 2. The answers

| # | Dispute (slice) | Referee | Where |
|---|---|---|---|
| 1 | Empty scissor: parallel-rdp draws one column, angrylion nothing (coverage) | **angrylion** *(borrowed code)* | `RDP_raster.vhd` §481, §722 |
| 2 | The combined input is the previous pixel's result (coverage, two-cycle) | **angrylion** *(own code)* | `RDP_CombineColor.vhd` §181 |
| 3 | 8-bit image: a forced blend against memory colour differs in green bytes (coverage) | **angrylion** *(own code)* | `RDP.vhd` §1368 |
| 4 | Texture formats 5–7 read as intensity (textures) | silent | `RDP_TexSingle.vhd` §201 |
| 5 | A 32-bit tile's rows past half of texture memory fold back (textures) | leans **angrylion** | `RDP_TexFetch.vhd` §328, §352 |
| 6 | The one-cycle texel1 is filtered by the first cycle's bilerp bit (textures) | **angrylion**, with §5 | `RDP_TexFetch.vhd` §716 |
| 7 | A tile whose only row has no sub-row loads one step (textures) | silent | — |
| 8 | A block load's left column is signed when placing the pointer (textures) | leans **parallel-rdp** | `RDP_raster.vhd` §1199 |
| 9 | An image read past sixteen megabytes wraps to zero (textures) | silent | — |
| 10 | A YUV tile is indexed through a palette by its bytes (filtering) | leans **angrylion** | `RDP_TexFetch.vhd` §359 |
| 11 | The one-cycle level of detail is measured along the span (level of detail) | silent | `RDP_pipeline.vhd` §1310 |
| 12 | The first blend's memory-alpha shifts use the previous pixel's slope (two-cycle) | **parallel-rdp** | `RDP_Zbuffer.vhd` §361 |
| 13 | The second blend reads the next pixel's shade alpha (two-cycle) | **parallel-rdp** | `RDP_BlendColor.vhd` §142 |
| 14 | A right-major copy span is written torn (copy) | **parallel-rdp** | `RDP_pipeline.vhd` §429 |
| 15 | The copy mode's tile is picked by the level of detail (copy) | **angrylion** | `RDP_pipeline.vhd` §385 |
| 16 | An 8-bit intensity-alpha copied texel doubles its high nibble (copy) | silent | `RDP_TexFetch.vhd` §863 |
| 17 | Alpha compare tests a copied byte in an 8-bit image (copy) | silent | `RDP_pipeline.vhd` §421 |
| 18 | A copied 4-bit colour-indexed texel carries the tile's palette number (copy) | silent, but see §7 | `RDP_TexSingle.vhd` §150 |
| 19 | A copied 4-bit intensity-alpha texel folds into intensity and alpha (copy) | silent | — |
| 20 | A copied YUV tile's chroma is read a further step along (copy) | silent | — |
| 21 | Chroma keying — no cross-check at all (chroma key) | silent, but see §6 | `RDP_CombineAlpha.vhd` §167 |

**Five rules go to angrylion outright and two more lean that way; three go to parallel-rdp with one more
leaning; ten are silent.** No row gives a fourth answer of its own, though row 6 comes with a complication
that §5 sets out. Two of the five — rows 2 and 3 — are in code the author wrote his own way, which is where
the pass earns its keep.

## 3. The two that matter most, both for angrylion

### 3.1 The combined input really is the previous pixel's

`Mars_RdpCoverage.md` §6.2 and `Mars_RdpTwoCycle.md` §5.2 record the same rule twice: combiner selector 0 —
the *combined* input — reads the **previous pixel's** combiner result, because the hardware has one register
there and nothing clears it between pixels. parallel-rdp reads zero.

The referee settles it as plainly as a reading can:

```vhdl
if (trigger = '1' or step2 = '1') then
   for i in 0 to 2 loop
      combiner_save(i) <= combiner_cut(i);
   end loop;
end if;
```

`combiner_save` is a register, written at the end of every combiner cycle and read by all four selectors of
the next one (`RDP_CombineColor.vhd` §100, §115, §128, §149; the alpha unit does the same with
`combine_alpha_save`). In the one-cycle mode there is no second cycle, so what selector 0 reads at pixel *n*
is what the combiner produced at pixel *n−1*. **A hardware implementation has nowhere else for that value to
come from**, which is the argument angrylion's authors make and parallel-rdp's shader cannot make, because a
shader has no previous pixel. This is the strongest single result of the pass.

### 3.2 An 8-bit image really does take green on odd addresses

`Mars_RdpCoverage.md` §6.3 records the oddest of Phase D's disputes: in an 8-bit colour image, the byte a
pixel writes is **red at an even address and green at an odd one**. Mars writes
`int value = (at & 1) != 0 ? g : r;` (`Rdp/Rdp.OneCycle.cs`). The referee, in code that shares no vocabulary
with angrylion at all:

```vhdl
writePixelData8 <= writePixelColor(1) when (writePixelAddr(0) = '1') else writePixelColor(0);
```

Identical rule, arrived at separately, and it explains itself: the write port is sixteen bits wide, so an
8-bit image's pixel takes whichever half of the pixel pair its address lands in. The dispute was recorded from
a case whose modes were captured verbatim because nobody could say what made it fire; this is what made it
fire.

## 4. The three that go the other way

These are the informative ones, because the referee had to build a blender for a real pipeline and chose
*not* to carry a value across pixels.

- **The first blend's memory-alpha shifts (row 12).** angrylion shifts them by the *previous* pixel's stored
  depth slope; parallel-rdp by the pixel's own. The referee pipelines the stored slope forward three stages —
  `old_dz_mem` → `_1` → `_2` → `_3` — precisely so that the shift is computed at the combiner stage **from
  the same pixel's** value (`RDP_Zbuffer.vhd` §247, §283, §312, §361), and feeds one pair of shifts to both
  blender cycles. Its author went to trouble to keep the pixel's own slope; angrylion's skew is not
  reproduced.
- **The second blend's shade alpha (row 13).** The referee's blender takes shade alpha from `pipeInColor(3)`,
  the pixel currently in the pipeline, for both cycles (`RDP_BlendColor.vhd` §142). angrylion reads the
  *next* pixel's.
- **The torn right-major copy span (row 14).** angrylion writes a right-major span's eight-byte group
  backwards from the starting pixel's first byte, so the span's end pixels are written in halves —
  `Mars_RdpCopy.md` §5.5, demonstrated there against the reference's own framebuffer rows. The referee writes
  the group **forwards from the pixel's address**, with the byte enables shifted by the address's low bits and
  the overflow carried into the next word (`RDP_pipeline.vhd` §429–437). For a 16-bit image that address is
  always even, so every write lands on whole pixels and nothing can tear.

**None of the three makes Mars wrong.** Mars grades against angrylion by construction, and these three rules
are each held by a named case that would fail if Mars changed. What they do is move the burden: for these
three the claim "the hardware does what angrylion does" now has an implementation on the other side of it, and
`Mars_RdpTwoCycle.md` §5.2 and `Mars_RdpCopy.md` §5.5 should be read with that in mind.

A caveat against over-reading row 14: the referee's right-major copy path looks unfinished rather than
decided. It starts the walk at the span's right end and steps left in groups of four pixels
(`RDP_raster.vhd` §944, §1011), but writes each group *upward* in memory from the current pixel, so the first
group of a right-major span appears to cover pixels past the span's own end. That is not parallel-rdp's rule
either. What can be said is narrow and still useful: **the referee has no mechanism by which a copied pixel
could be written in halves.**

A fourth rule leans the same way without joining the argument. Row 8 asks whether a block load's left column
is read as signed when the image pointer is placed; the referee subtracts `'0' & signed(Tile_sl) & "000"`
(`RDP_raster.vhd` §1199), and that leading zero makes the value non-negative whatever the field holds, which
is parallel-rdp's reading. It is filed as a lean rather than an answer because the referee places the pointer
from the edge-walked span rather than from the column directly, so the question does not arise in quite the
same shape.

## 5. The one that needs its own paragraph

Row 6 asks which cycle's bilerp bit filters the one-cycle mode's texel1 — angrylion says the first cycle's,
parallel-rdp the second's. The referee's unfiltered override reads `biLerp0` (and `biLerp1` only in
combination with `convertOne`) whenever the cycle type is not copy or fill (`RDP_TexFetch.vhd` §716), which is
angrylion's answer.

**But the referee disagrees with both about which pixel that texel belongs to.** In the one-cycle mode it
assigns (`RDP_TexFetch.vhd` §797)

```vhdl
tex2_color_outnext(0) <= tex_color_outnext(0);
```

— texel1 takes the value texel0 held *before* this pixel's fetch, which makes it the **previous** pixel's
texel0. angrylion and parallel-rdp both make it the **next** pixel's. All three model a one-pixel skew
between the two texels in a mode that only fetches one; the referee's runs the other way. Since a skew's
direction is a matter of where a pipeline's phase is counted from, and since the referee saves a fetch by
reading backwards, this is recorded as a difference rather than a correction. It does answer a smaller
question decisively: **the skew is real**, and a one-cycle texel1 is nobody's idea of the pixel's own texel.

## 6. Chroma keying: still no cross-check, but not nothing

`Mars_RdpChromaKey.md` is Phase D's only slice with no second opinion, and this pass does not give it one.
The referee parses the key and then does not use it:

```vhdl
if (settings_otherModes.key = '0') then
   calc_alpha := combiner_result + to_integer(ditherAlpha);
else
   error_combineAlpha <= '1'; -- todo: key alpha mode
end if;
```

The key alpha is an acknowledged hole, and a keyed primitive raises the core's error line
(`RDP_CombineAlpha.vhd` §167, §199). **§2 of that page — the distance, the widths in sixteenths, the
low-nibble-eight rounding — remains graded against angrylion alone.**

Three structural facts do come back confirmed, from a source that had to decode the command stream for itself:

- **The mode bit is bit 40** (`RDP_command.vhd` §290, `settings_otherModes.key`).
- **The command fields are where `Mars_RdpChromaKey.md` §1 puts them** — Set Key GB (`0x2A`) carries blue
  scale 0–7, blue centre 8–15, green scale 16–23, green centre 24–31, blue width 32–43, green width 44–55;
  Set Key R (`0x2B`) carries red width 16–27 (`RDP_command.vhd` §233–244). Widths are twelve bits, centres
  and scales eight — exactly the table Mars reads.
- **The centre and the scale are combiner inputs**, colour B selector 6 and multiplier selector 6
  (`RDP_CombineColor.vhd` §121, §134).
- **Alpha from coverage wins over the key**, because the referee's key branch sits inside
  `if (alphaCvgSelect = '0')` — the precedence `Mars_RdpChromaKey.md` §1 states.

So the slice's *framing* is corroborated and its *arithmetic* is not. The page's closing sentence stands
unchanged: settling it needs a test ROM on a console.

## 7. What the pass confirmed outside the disputes

Reading ten thousand lines for twenty-one answers turned up corroboration for mechanisms that were never in
dispute but were read from one source only. These are worth recording because each was a place a
misreading would have been silent:

- **The copy mode moves eight bytes a group, four texels read at once** (`RDP_pipeline.vhd` §403–425), not one
  pixel at a time — `Mars_RdpCopy.md` §2's central claim, and the thing parallel-rdp does differently.
- **The copy mode's stride is twice the texel's width**: the referee forms the address as `index × 2` for an
  8-bit or YUV tile, `× 4` for 16- and 32-bit, `× 1` for 4-bit (`RDP_TexFetch.vhd` §253–270).
  `Mars_RdpCopy.md` §2.2.
- **An odd row swaps eight bytes**, as a single inversion of address bit 3 (`RDP_TexFetch.vhd` §308–316).
- **The bank is address bits 2 and 3 and the upper half is bit 12**:
  `select0next <= addr(12) & addr(3) & (not addr(2))`, with the word index taken from bits 11–4
  (`RDP_TexFetch.vhd` §323–332). This is the arrangement `Mars_RdpCopy.md` §2.1 argues makes a bank conflict
  unobservable.
- **A row's line is nine bits**, added shifted up four (`RDP_TexFetch.vhd` §272).
- **Alpha compare in a copied 16-bit image clears a pixel's byte enables in pairs**
  (`RDP_pipeline.vhd` §421–424) — the rule Mars's "each tested byte keeps a pair" breakage holds.
- **An 8-bit colour image reads back as its byte in all three channels with alpha 0xE0**
  (`RDP_FBread.vhd` §118) — and the referee's `-- todo: unclear` beside that alpha says its author had the
  same doubt Mars's page records.
- **A 4-bit colour-indexed texel carries the tile's palette number in its high nibble**
  (`RDP_TexSingle.vhd` §150). That is the sampler's path, not the copy mode's, so it does not settle row 18 —
  but it does mean angrylion's copy rule reuses wiring the hardware demonstrably has, while parallel-rdp's
  doubling of the nibble would need a second path built specially.
- **A 4-bit fill is not modelled anywhere**: the referee reports `"4 Bit Fill mode, RDP will crash"` with
  `severity failure` (`RDP_raster.vhd` §1155). parallel-rdp aborts the process on the same command. Two
  implementations treating a command as impossible is a reasonable basis for `Mars_RdpDifferential.md` §4.4's
  decision to mark those cases `crossChecked: false` rather than chase them.

## 8. What this pass did not establish

- **Nothing about the hardware.** Three implementations agreeing is three readings agreeing. The only
  instruments that would settle a dispute are a test ROM on a console (`lemmy-64/n64-systemtest`,
  `rasky/n64-systemcrash`) or a measurement of the die.
- **Nothing that changes Mars.** No rule moved, no case was added or retired, and the twenty-one disputes are
  still disputes. Mars follows angrylion in all of them, including the four where the referee does not.
- **Nothing about the ten silent rows.** Rows 4, 7, 9, 11, 16, 17, 18, 19, 20 and 21 are unmodelled in the
  referee, and row 5's answer is a lean read off an address width, not a rule anyone wrote down. A dispute
  with no third opinion is exactly as open as it was this morning.
- **Nothing that could be automated.** This was a reading, not a differential. Running the referee as a
  *grader* would mean simulating the VHDL against the same dumps — the core carries the scaffolding for it
  (`export_*` signals throughout `RDP_pipeline.vhd`, and an `export` module that writes traces), but it would
  be a harness of its own and is not proposed here.

## 9. Retired predictions

~~"An FPGA implementation would settle chroma keying."~~ Recorded in `Mars_RdpChromaKey.md` §0 on 2026-09-17
and retired the same day: the one open-source N64 FPGA core parses the key's commands and then reports the
key alpha as unimplemented (§6). The prediction was reasonable and wrong in a specific way worth keeping —
a core good enough to run commercial games can leave a feature out precisely because commercial games rarely
use it, which is the opposite of the property a reference needs.

~~"The referee will mostly repeat angrylion."~~ Half right. Of the eleven rules it implements it takes
angrylion's side in seven and parallel-rdp's in four. The four are not scattered: rows 12 and 13 are both
places where angrylion carries a value from one pixel to the next, and the third rule of that kind — row 2,
the combined input — is one the referee confirms instead. So **the cross-pixel rules are exactly where the
three implementations part company**, and they are what to put on a console first if anybody ever gets one on
the bench. Row 6 belongs to the same family and points the same way: all three implementations skew the
one-cycle texel1 by a pixel, and no two of them agree on the direction. Rows 8 and 14 belong to no pattern and
may be no more than unfinished paths.
