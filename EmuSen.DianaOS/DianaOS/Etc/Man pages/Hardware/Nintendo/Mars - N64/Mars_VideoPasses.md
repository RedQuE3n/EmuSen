# Mars — the three passes past the filter: undoing dither, divot, and gamma

*Landed 2026-09-17. Phase E's third slice. The code is `Vi/Vi.Dither.cs`, `Vi/Vi.Divot.cs` and
`Vi/Vi.Gamma.cs`, one file a pass as the reference has one file a pass; the grading is
`MarsViDifferentialTests`.*

*One of the four control bits in this slice is **not graded and not implemented**, and §3.1 is the argument
for that rather than an admission. It is the first time in Phase E that the right answer was to build less.*

---

## 0. What grades this

**No new instrument.** The dump already carries video registers, hidden memory and the frames angrylion
scans out; this slice only sets three more bits in the control word. parallel-rdp's `vi-conformance` covers
divot, the dither filter, gamma and gamma dither in its matrix and passes all twenty-four suites, so the two
software references still agree about the whole device.

**Where the three passes sit.** A pixel is fetched, then either anti-aliased or dither-filtered — never both,
because the two are the arms of one test on its coverage — then divoted against its neighbours across the
row, then interpolated with the pixel below and to the right, and only then gamma'd on its way into the
raster. Divot is therefore a pass over *source* pixels and gamma a pass over *output* pixels, and §5 is the
referee agreeing about that order from a pipeline that makes it structural.

## 1. The dither filter

**The display processor dithers when it writes a pixel; the video interface undoes it when it reads one
back.** A whole pixel — coverage seven, which is the arm of §0's test that the filter slice left empty —
moves one step towards each of its eight neighbours that is brighter and one step away from each that is
darker, per channel, when control bit 16 is set.

**Only the top five bits are compared**, which is the depth the processor dithered at, and **every neighbour
is weighed against the pixel as it arrived** rather than against the running total. So the result is the
pixel plus a number between −8 and +8.

**It needs no clamp, and cannot be given one that fires.** A pixel whose top five bits are already 31 has no
neighbour that can be brighter, so it can only lose; a pixel that can gain all eight steps has top five bits
of at most 30, so its value is at most 247 and its result at most 255. The same holds at the bottom. The
reference clamps nothing and the FPGA clamps at 255 (§5), and neither clamp can ever be reached.

### 1.1 The eight neighbours

**The full ring this time**, not the six of the anti-aliasing filter: the row above at −1, 0 and +1, the row
below at −1, 0 and +1, and this row at −1 and +1. The pixel itself is not among them.

**The fetch artefact of `Mars_VideoFilter.md` §3 reaches this pass too**, and reaches it differently: where
the anti-aliasing filter's two lower neighbours become this row's two horizontal ones, the dither filter's
*three* lower neighbours become this row's −1, 0 and +1 — and the middle of those three is the pixel itself,
which contributes nothing by construction. A pass that weighs eight neighbours therefore weighs seven under
the artefact, one of them twice.

## 2. Divot

**Three pixels across the row, of which the middle takes their median**, when control bit 4 is set. It is
there to fill in the single-pixel notches that anti-aliasing leaves where two edges meet, which is what the
name means.

**A run of whole pixels is left alone.** The pass returns early when all three coverages are seven, so divot
only reaches the places the display processor left partly covered — which is also why a picture with no
partial coverage anywhere is unchanged by it, and why a case that proves the early return needs a frame
buffer built for it.

**The median is spelled as two questions**, not as a sort: is the left pixel between the other two, and
failing that, is the right one? If neither, the middle keeps its own value. The comparisons are written with
`>=` at both ends, so ties fall to the left pixel first.

**Divot runs on source pixels, before the interpolation**, so each of the four pixels a resampled output
mixes is divoted against *its own* neighbours in the frame buffer. That is four medians and twelve fetched
pixels for one output pixel, on top of what the filter already reads.

## 3. Gamma

**The channel is replaced by twice the integer square root of itself with six more bits below it**, when
control bit 3 is set: `sqrt(channel << 6) << 1`. It is the last thing that happens to a pixel, after the
interpolation, and it happens only in the columns that carry signal — a dark guard column is written as zero
without passing through it.

**One table serves the whole pass.** The reference builds two — one of 256 entries indexed by the channel
and one of 16,384 indexed by the channel with the dither below it — but the first is the second sampled
every sixty-fourth entry, by construction rather than by coincidence, so Mars builds the larger one and
indexes it at `channel << 6`.

### 3.1 The dither that is not graded

**Control bit 2 is the gamma dither: six bits of noise sitting below the channel in that square root, or,
with gamma off, a bit added to each channel. Mars does not implement it, and no case sets the bit.**

The reason is the one `Mars_RdpCoverage.md` §6.3 gives for the display processor's noise, and this slice is
where it turns out to extend to Phase E. angrylion's noise is a hash of the pixel's column, its row and a
count of frames, carrying a comment saying it is from a shader example *"to make noise deterministic in
validation"*. It is not a claim about the console.

**What is new here is that the referee makes the point twice as strongly.** The FPGA core does not use a
hash: it uses a free-running 23-bit shift register with taps at 22 and 18, clocked every cycle, and takes its
six dither bits off the bottom of it. That is a plausible model of a hardware noise source and angrylion's is
not — but the two cannot be made to agree, because one is a function of where the pixel is and the other is a
function of how many cycles have passed. **Two implementations with two incompatible noise sources, neither
of them a measurement, is a stronger argument for not grading the values than one implementation admitting
its source is a stand-in.**

**So Mars reads the dither as zero**, which is what `Mars_RdpCoverage.md` §6.1 already does for every unbuilt
combiner input, and which the FPGA itself does when the bit is clear — its six dither bits are hard-wired to
zero there. With gamma on, Mars's picture is then systematically at the low end of each entry the dither
would have spread over: never wrong by more than one step of the table, always wrong in the same direction.
With gamma off it is unchanged, where a console would have dithered.

**What this costs, stated plainly:** a game that sets bit 2 gets a picture from Mars that is smoother than
the console's by exactly the dither. Nothing in the suite can see it, and nothing could — which is the
point. The frame counter Mars briefly grew to seed the hash was deleted with it, because a counter nothing
reads is not state.

**The two references also disagree about the dither with gamma off**, which is worth recording even though
nothing grades it: angrylion adds the bit and guards against passing 255, while the FPGA replaces the
channel's low bit outright. Those differ for every channel whose low bit is already set.

## 4. What the differential says

### 4.1 The cases

**One hundred and fifty-one cases: seventy-one named and eighty random, every one matching angrylion on the
first run.** Twenty-one of the named cases are new and half the random ones now turn on a random set of the
three implemented bits. One of the twenty-one was added by the breakage round rather than before it, and
another rewritten by it, for the reason §4.2 gives.

**The named cases are built around what each pass must *not* do**, because each is easy to get right in the
common case and wrong at its boundary:

- The dither filter with **no whole pixel to filter**, which must leave a picture exactly as the
  anti-aliasing filter left it, and with **every pixel whole**, where it must touch all of them.
- Divot where **every pixel is whole**, which must be identical to divot switched off, and where **none
  is**, where it runs everywhere.
- Both filters together, to show the coverage test really does choose between them rather than running both.
- Gamma in the replicate, resample and anti-aliased modes, because gamma is past the interpolation and must
  not care which produced the pixel.
- A fractional step with divot on, which is the case that separates a divot before the interpolation from
  one after it.
- Every implemented pass at once, in both pixel formats.

### 4.2 Against Mars

**One hundred and six breakages, of which one hundred and one apply to the code as it stands, and every one
of the hundred that can be seen at all is caught by a named case.** Five describe rules that are not in
Mars: three the review of `Mars_Video.md` §3.3 removed, the byte mask `Mars_VideoFilter.md` §4.2 removed,
and one this round removed, below.

**A prediction from the slice before came true.** `Mars_VideoFilter.md` §4.2 recorded a survivor — *a pixel
of coverage seven is not filtered* — and argued it would stop being free when the dither filter arrived,
because coverage seven is the condition that selects between the two filters rather than between one filter
and itself. It is now caught by **twenty-four named cases**. A survivor documented with a reason and a date
is worth more than one quietly removed.

**The round found a case that could not reach the rule it was written for.** *The dither filter after a row
read twice* was written for the fetch artefact of §1.1 and set the replicate mode — where nothing samples
the row below at all, so the artefact cannot arise. Only random cases caught the rule. The case now
resamples, in both pixel formats, and catches it. The lesson is narrower than it looks: a case aimed at a
rule that only fires in one mode has to be written in that mode, and nothing but a breakage round will say
whether it was.

**One rule was removed and one survives.**

- ~~*Gamma runs only in the columns that carry signal.*~~ **Removed.** A dark column is written as zero
  regardless of what the pixel holds, so passing it through gamma first cannot change the raster. The
  reference has the same guard, where it sits inside the branch that also writes the pixel; in Mars the
  write is a ternary, which makes the guard decide nothing.
- *The dither filter keeps the coverage it was given.* **Kept, and unreachable by construction.** Its only
  caller reaches it when the coverage is exactly seven, so returning the pixel's own coverage and returning
  a literal seven are the same thing. Unlike the survivors above this is not a rule that might become
  observable later — it is a statement the call site already guarantees, and the breakage is testing the
  arithmetic of a constant.

### 4.3 The tool-free test

**`MarsViTests` gained five scans and seventeen of angrylion's pixels, and catches eighty-six of the
hundred and one breakages.** The five are the dither filter in both pixel formats, divot over a picture with
mixed coverage, gamma, and all three at once with a fractional step in both directions. It passed on its
first run.

**Fifteen it misses, of which eleven are the classes `Mars_VideoFilter.md` §4.3 already named** — five
geometry, four border and fade, replication where the test has no fractional step, and the coverage a dark
column keeps — plus the rounding in the anti-aliasing pull. **Four are new, and three of them say something
about what a fixed list of pixels can hold:**

- *Only the five bits the processor dithered at are compared.* Comparing all eight bits instead differs only
  where two pixels agree in their top five bits and differ below, which cannot happen at all in a
  sixteen-bit picture — the low three bits are zero there — and happens for about one neighbour in
  thirty-two of a thirty-two-bit one. Three checked pixels in one thirty-two-bit scan do not reach it. The
  differential does, over a hundred and fifty thousand.
- *Three whole pixels are left alone.* Divot's early return needs a checked pixel whose whole
  neighbourhood is whole, which the test's mixed-coverage scan does not happen to provide.
- *The dither filter keeps the coverage it was given*, which nothing can catch (§4.2).
- The one removed rule, which no longer exists to catch.

**The ratio is the same story as the slice before.** Eighty-six of a hundred and one is what survives on a
machine where the reference tools were never built; every one of the fifteen is held by a named case in the
differential, so nothing is ungraded.

## 5. The referee

**The FPGA core implements all three passes, in the same file that gave `Mars_VideoFilter.md` §5 its
independent construction, and it agrees about all three.** Its vocabulary is again its own — `dedither_add`,
`dedither_clamp`, `gamma_in`, `gamma_sqrt` — so the agreement carries weight.

- **The dither filter is `dedither`.** `VI_filter.vhd` §119–128 counts `+1` for each of eight neighbours
  whose bits 7 downto 3 exceed the pixel's and `−1` for each below, adds the count, and clamps at 255 — the
  clamp §1 proves unreachable. The eight neighbours are `proc_pixels_DD(0..7)` in `VI_lineProcess.vhd`
  §121–128, taken from the same three shift registers as the anti-aliasing filter's six: the full ring
  around the pixel.
- **The two filters are arms of one test.** `VI_filter.vhd` §202–210 chooses the dedither when the coverage
  is seven and its control bit is set, and the anti-aliasing result when the coverage is under seven — an
  `if`/`elsif` on the same signal, which is §0's structure in hardware.
- **Divot is a median of three, before the interpolation.** `VI_filter.vhd` §230–265 takes three consecutive
  pixels out of the filter pipeline and picks the middle by the same pair of questions, gated on any of the
  three not being whole. The pipeline order settles the rest: `VI_videoout.vhd` §426–520 wires
  `VI_lineProcess` into `VI_filter`, the filter's output into a line RAM, and the RAM into `VI_outProcess`,
  which is where the interpolation and gamma live. Divot is on the source side of that RAM and gamma on the
  output side, exactly as §0 has it.
- **Gamma is the same square root, built as a circuit.** `VI_sqrt.vhd` is a seven-stage restoring square
  root over a **fourteen-bit** argument whose output is `stage1_result & '0'` — the root, doubled. That the
  argument is fourteen bits and not eight is itself the confirmation of §3's shape: the six bits below the
  channel are part of the operand, and `VI_outProcess.vhd` §346 fills them with the shift register or with
  zeros.

**What it does not settle is §3.1's noise**, and it is the reason that section is written the way it is.

**What none of the three can settle** is whether the console's dither filter really compares five bits rather
than six or four. All three implementations agree, and all three could have inherited the choice from the
same place; the filter's effect is at most one step of eight in the low bits, which makes it a poor candidate
for a console test and a good candidate for staying as it is.

## 6. What is not here

- **The gamma dither** (§3.1), deliberately.
- **The per-scanline registers**, the interrupt, and the timing, as `Mars_Video.md` §4 has them. Nothing
  advances a half line yet, so no game drives this device — and with this slice the picture side of the
  interface is otherwise complete.
- **`GetFrameBufferRgba`**, which is still not wired to the raster.
