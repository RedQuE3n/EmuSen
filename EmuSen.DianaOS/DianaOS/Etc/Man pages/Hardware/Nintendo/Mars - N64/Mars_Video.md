# Mars — the video interface: registers, geometry and the scan into the raster

*Landed 2026-09-17. Phase E's first slice, and the first thing in Mars that produces a picture rather than a
memory. The code is `Vi/Vi.cs` (the fourteen registers and the raster) and `Vi/Vi.Scanout.cs` (the geometry,
the borders and the walk); the grading is `MarsViDifferentialTests`.*

*Two things are different here from every Phase D page, and §0 says why: the references **agree** about this
device, and the harness had to be extended before a single rule could be graded.*

---

## 0. What grades this, and why the instrument changed

**The display processor's output is memory; the video interface's output is a picture.** `rdp-reference`
compared RDRAM at each sync, and nothing it wrote could show what the interface scanned out. Three changes
made this slice gradable, and they are the first half of the work:

- **The reference's frames are captured.** angrylion hands each scanned frame to a `vdac_write` callback
  that the probe discarded with the comment *"Nothing is displayed: the RDRAM is the output."* It now copies
  the frame row by row — the pointer is into a buffer the reference keeps between frames — and writes it as
  record tag 6 of `RDPREF01`, with the frames since the last sync attached to that sync.
  `Mars_RdpDifferential.md` §2's format grew one record; nothing else about it changed.
- **The dump can drive the interface.** `RDPDUMP2` already carried a *set VI register* record and an *update
  screen* record, and `rdp-reference` already applied both — the C# writer simply never emitted them.
  `RdpDump.ViRegister` and `RdpDump.UpdateScreen` do now.
- **The cross-check is a different shape.** `rdp-validate-dump` compares memory, not frames, so it cannot
  check a scanned picture. parallel-rdp's own repository carries `vi-conformance`, which runs **its video
  interface against angrylion's** over randomised memory across a matrix of modes, and `build-probe.sh rdp`
  now builds it beside the other two tools.

**What that conformance suite says is the headline of this page.** All twenty-four of its suites pass at
sixty-four random frames each: every anti-alias mode, both pixel formats, the blank and reserved types,
divot, the dither filter, gamma, gamma dither, randomised scale and bias, randomised horizontal and vertical
start and end, PAL, serrate, and the per-scanline registers. **The two references agree about the whole
video interface.** Phase D recorded twenty-one rules where they did not; here there are none.

**So the grading is stronger and the cross-check is weaker, both at once.** Stronger, because a Mars
disagreement cannot be blamed on a reference disagreement — it is a Mars bug until proven otherwise, which
was never true of the display processor. Weaker, because the agreement is established by *their* suite over
*their* random frames, not case by case over Mars's: nothing checks that the two references agree about the
particular pictures below. That is a real gap and §3.1 keeps it in view.

## 1. The registers

**Fourteen words at `0x0440_0000`**, held as written and read back unchanged; this slice gives none of them
an effect beyond the scan.

| Word | Name | What the scan reads |
|---|---|---|
| `0x00` | control | the pixel type, the anti-alias mode, serrate, and the filters §4 leaves out |
| `0x04` | origin | the frame buffer's address, twenty-four bits |
| `0x08` | width | the frame buffer's line in pixels, twelve bits |
| `0x0C` | interrupt | — (§4) |
| `0x10` | current line | its lowest bit, which picks an interlaced field |
| `0x14` | burst | — |
| `0x18` | vertical sync | the field's half lines, which also decide PAL from NTSC |
| `0x1C` | horizontal sync | — |
| `0x20` | leap | — |
| `0x24` | horizontal start | the picture's first and last column of the line, ten bits each |
| `0x28` | vertical start | its first and last half line, ten bits each |
| `0x30` | x scale | the step across, twelve bits, and the bias it starts from |
| `0x34` | y scale | the step down, and its bias |

**Three fields are packed the same way**: a start in bits 16–25 and an end in bits 0–9 for the two start
registers, and a step in bits 0–11 with a bias in bits 16–27 for the two scales.

### 1.1 The type and the mode

**The control word's low two bits are the pixel type**: 0 blank, 1 reserved, 2 sixteen-bit, 3 thirty-two-bit.
Only bit 1 decides whether there is a picture at all, and only bit 0 decides how wide a pixel is — so the
reserved type reads memory as thirty-two-bit pixels and blanks the screen, which is what both references do
and what a case holds.

**Bits 8 and 9 are the anti-alias mode.** This slice implements the two that need no coverage: **resample
only** (2), which interpolates between neighbouring pixels, and **replicate** (3), which does not. Modes 0
and 1 read coverage out of the hidden bits and filter by it, and landed in the slice after this one —
`Mars_VideoFilter.md`.

**Bit 6 is serrate**, the interlaced signal, which this slice carries through the geometry and the raster
even though nothing here interlaces the output.

## 2. The scan

### 2.1 The geometry

**The registers are edges and steps; the scan needs lengths and offsets.** In order:

1. **The picture's size** is the difference between each register's end and start, and the vertical one is in
   half lines, so it is halved. Both are taken from ten-bit fields, so an end that runs past 1023 wraps — a
   case holds that, because it is easy to write a test that means to ask for six hundred rows and asks for
   eighty-eight.
2. **The picture is pulled back by a fixed offset** — 108 columns and 34 half lines for NTSC, 128 and 44 for
   PAL — because the registers count from the start of the signal and the raster counts from the start of
   the picture. PAL is decided by the vertical sync being longer than 550 half lines.
3. **A picture that now starts before the raster is stepped into instead.** The bias gains the step times the
   overshoot and the picture loses those columns, which leaves the same pixels under the same columns of the
   raster. The same holds vertically.
4. **What is left is cut to the raster's width**, 640 columns. Both references also cut the height, twice
   over; §3.3 says why Mars does not.
5. **The active lines** are the vertical sync less the same offset, halved unless interlaced. They are the
   height of the borders, not of the picture.

### 2.2 When there is no frame

**Four conditions produce no picture at all**, and each is a case:

- **The frame buffer's address is zero.** Nothing is read and nothing is scanned.
- **The vertical sync is shorter than the offset**, so there are no active lines.
- **A blank type twice in a row.** The first blank clears the raster and scans; the second does nothing, so
  the screen it left stays.
- **A picture no columns wide**, or one starting past the raster's right. The borders are still drawn — the
  order matters, and §2.4's fade is how it shows — but no pixel is scanned and no frame is produced.

**A blank picture still scans.** The type's bit 1 clears the raster and the held lines, and then the walk
runs anyway, reading memory as sixteen-bit pixels and writing them over what was just cleared. It reads
oddly and both references do it, so Mars does it.

### 2.3 Where the picture goes

**The raster is 640 by 625** — the widest and tallest signal the interface can raise — and a frame is its
first rows: 240 for NTSC, 288 for PAL, and twice that when interlaced, because an interlaced field writes
every other line of a frame that holds both.

**An interlaced picture writes at twice the stride**, starting one line down when the current line register's
lowest bit is clear. That bit is the field number, and it is the only register the scan reads that a game
does not write.

### 2.4 The borders, and the two frames of grace

**The raster is kept between frames**, because a television keeps showing a line nothing has rewritten. That
makes the interface stateful in the same way the display processor's tiles are, and the same way it bites:
a rule about a line that is *not* written can only be seen a frame or two later.

Each line carries a count, and a frame spends it:

- **A line the picture covers is held**, its count set to two.
- **A line outside the picture counts down**, and the frame it reaches zero is the frame it goes dark. Above
  the picture the count is watched — a line is cleared only in the frame its count runs out. Below it the
  count is spent and then tested, so a line that is already out is darkened again every frame. The asymmetry
  is in both references and a breakage finds it.
- **The columns either side of the picture are cleared every frame**, on every active line, not on the
  picture's lines alone.
- **An interlaced field holds its own lines and fades the other field's.** This is the rule that cost the
  slice its only real debugging: Mars held both fields' lines, which is what "draw every other line" suggests,
  and nothing showed for two cases — the counts were wrong, not the pixels. It surfaced a case later as one
  row of stale picture where the reference had gone dark. §3.2 has the shape of the hunt.

### 2.5 The walk

**One pass over the picture.** Each row takes its own line of the frame buffer, at `bias + row × step`
shifted down ten, times the frame buffer's width in pixels; the five bits below that shift are the row's
fraction. Each column steps across the same way.

**The columns at either end are dark.** Eight at the left and seven at the right of every picture carry no
signal — unless the picture was pulled in at that edge, in which case that end's guard is not applied. §5
is the referee's word on this, because it looks exactly like a software convenience and is not.

**A dark column clears the colour and keeps the coverage** the raster already held, because the reference
writes three bytes there and not the fourth. A scanned column writes a coverage with its colour: seven in
the modes this slice implements, and ~~always seven~~ the pixel's own from `Mars_VideoFilter.md` §1 on.

### 2.6 The fetch

**A sixteen-bit pixel is 5-5-5-1 and becomes eight bits a channel by moving up, not by filling in**: the low
three bits of each channel are left at zero. They are not lost — the anti-aliasing of `Mars_VideoFilter.md`
is what fills them — so a picture scanned in this slice is measurably darker than the same picture through a
console's filter, and that is correct rather than a defect.

**A thirty-two-bit pixel takes its first three bytes** and ignores the fourth, which is coverage — until
`Mars_VideoFilter.md` §1, where three of its bits are read.

**A read past the end of memory gives zero** rather than wrapping.

Since Phase G a scan fetches each source line once, into a window contiguous in the index the walk and the
filters share, and the filters read their neighbours from it (`Mars_Performance.md` §21); an index outside the
window is fetched as described here.

**Interpolation, when the mode asks for it and either fraction is nonzero**, mixes down the column first and
across the row second: the pixel and the one below it, the pixel to the right and the one below that, then
those two. Each mix is `near + ((far − near) × fraction + 16) >> 5`, with the fraction five bits.

### 2.7 The walk deferred to another thread

*2026-09-19, Phase G.* The scan-out was a quarter of Ocarina of Time's frame and a sixth of Super Mario 64's in
play (`Mars_Performance.md` §26), and it reads the machine without changing it: a scan is a function of the
registers and of the frame buffer's lines, and what it writes is the raster. So the walk — everything from §2.5
and §2.6 — now runs on a thread of its own while the next frame runs, and the picture a frontend is handed is the
one the walk before last produced.

**The scan is split at the walk.** `Prepare` does everything §2.1 to §2.4 describe — the geometry, the blank
rule, the borders and the held lines — and says whether a walk is due; it runs on the emulation thread, because the
held-line count and the blank flag are state (`Mars_SaveStates.md`), and a state written between frames must hold
what a scan at once would have left. `Capture` copies out of RDRAM the lines the walk can reach, and the hidden
bits beside them. `Walk` runs over the capture. `Scan()` is the three in a row over live memory, and is what it
was.

**What a walk can reach, and the argument that the capture holds it.** A row's window fetches whole lines from two
above its own to three below (`Mars_Performance.md` §21); a step lands up to the row's span past a line's start,
2,563 pixels at the widest picture and the largest step; a sample reaches one line and two pixels either side of
where it landed, through the filter's and the dither's neighbours and divot's; the fetch-bug fold reads the row's
own line where the one below would be. The capture therefore begins one line and a row's span before the window's
first line and ends two lines and two spans after its last, in pixels of the frame buffer's width, clamped to
memory. An address the walk asks for inside memory and outside the capture is not read from live memory — that
would be a torn picture, silently — but thrown as a defect in this argument, and the join rethrows it.
A negative index wraps, as §2.6's arithmetic always has, to an address just below the origin or past the end of
memory; the former is inside the capture, the latter reads zero as before.

**The picture is one frame behind the machine.** After `RunFrame` *n* the walk of frame *n* is in flight; it is
joined at the start of the next frame's presentation, before the raster is touched again, and its picture becomes
the shown one. So `GetFrameBufferRgba()` after frame *n* + 1 is the picture of frame *n*, exactly, and the state
is the state of frame *n* + 1. A loaded state is presented at once, on the loading thread, so a rewind or a slot
shows what it loaded. Switching the mode off joins the walk in flight and shows it. `SkipRendering` skips both
halves, as it skipped the scan.

**What proves it.** `MarsDeferredPresentationTests`: every register set the reference test of §3.4 scans, walked
from a capture after the live frame buffer and hidden bits were overwritten, against the same set walked over
memory — twenty-seven geometries, both pixel formats, interlaced and progressive, a 320-wide picture at every
pass, and a PAL one stepping down at 0x355; a synthetic program painting a changing frame buffer, run as an
immediate core and a deferred one for eight frames, states identical every frame and the deferred picture the
immediate one of the frame before; and a state loaded into a deferred core presented at once. From the three
gameplay states of `Mars_Performance.md` §26, 600 frames each: the two machines identical every frame and the
picture one behind, through 200 distinct pictures in Ocarina of Time and Wave Race and 300 in Super Mario 64.
The probe's 1,800 frames are unchanged, since the probe presents at once.

**What it does not do.** It does not skip a scan whose inputs have not changed, though the lockstep runs show the
games draw a new picture every third field (Ocarina of Time, Wave Race) or every second (Super Mario 64), so two
scans in three or one in two are of a frame buffer the last scan already walked; off the emulation thread that
costs a core's time and not the frame's, so it is left. Hotaru presents at once. The display processor still
draws on the emulation thread, which §26 names as the next thing to move.

## 3. What the differential says

### 3.1 The first run, and the bug it found

**Ninety cases — thirty named and sixty random — of which eighty-four of the first eighty-five matched
angrylion on the first run.** The one that did not was a real defect in Mars, and it is the rule of §2.4's
last bullet: an interlaced field holds its own lines and **fades the other field's**, where Mars held both.

That failure is worth its shape. It did not appear in the interlaced case, which passed; it appeared two
cases later, as one row of stale picture where the reference had gone dark, in a case about something else
entirely. Nothing was wrong with any pixel the interlaced case drew — what was wrong was a *count*, and a
count is only ever visible through a line that should have gone dark and did not. Finding it needed a trace
of which case last wrote that row, not a closer look at the case that failed. The sequence that exposed it is
now a named case of its own.

**The random cases scan one frame each** through random horizontal and vertical starts and ends, random
steps and biases in both directions, random frame buffer widths, and both pixel types in both implemented
anti-alias modes. They found nothing the named cases did not, which is the usual result and still worth
recording.

### 3.2 Against Mars

**Forty-eight breakages, forty-five of which apply to the code as it stands; every one of the forty-five is
caught by a named case.** The other three describe rules that the review removed, and §3.3 says why.

**Seven survived the first round**, and the resolutions fell into three kinds:

- **Three were cases nobody had written.** A blank signal clears the raster *and* every line's count before
  it scans, and both clears are invisible until a narrower or shorter picture follows; a line below the
  picture is darkened again every frame once its count is spent, which needs a frame that writes it and two
  that do not. Three named cases now do exactly that.
- **One was a rule that could not be seen from where the cases stood**, and could be seen from somewhere
  else: the clear a line gets when its count runs out covers only the picture's own columns, not the whole
  line. Everywhere the cases looked, the border clears of §2.4 had already darkened the rest. They do not
  reach a line *above* the picture and *past* the active lines, which is possible because the frame is 240
  rows whatever the vertical sync says. A case with a short sync and a high picture reaches it.
- **Three were rules Mars did not need.** One was genuinely redundant — the test that skips interpolation
  when both fractions are zero, which the interpolation itself already handles by returning the nearer pixel.
  The other two are §3.3's clamps.

### 3.3 Three rules the review took out

**Both references clamp the picture's height twice** — once against the raster and once, more tightly, for an
interlaced field. Both are overflow fixups for a buffer indexed without bounds, and Mars's raster is a C#
array whose writes are range-checked, so neither clamp can change a frame. They are left out. This follows
`Mars_RdpDepth.md` §1.3: a reference rule that exists to keep the reference from misbehaving is not a rule
about the hardware.

**A third went for a plainer reason.** Both references test whether either fraction is nonzero before
interpolating; the interpolation returns the nearer pixel when a fraction is zero, so the test decides
nothing. With those three rules gone, three of the forty-eight breakages describe code that no longer
exists.

~~"The narrower clear a spent line gets cannot be observed, so Mars can darken the whole line."~~ **Retired
the same afternoon it was written.** The argument was that the border clears always cover the complement of
the picture's columns, which is true for every line the borders reach — and the borders stop at the active
lines while the frame does not. Mars was simplified on that argument, two cases went red within the minute,
and the faithful version went back in. The lesson is narrow and worth keeping: *an argument that a rule is
unobservable has to name the range over which it holds*, and this one quietly assumed the active lines
covered the frame.

### 3.4 The baseline, and a test without the tools

**There is no baseline to measure.** With the slice removed there is no video interface at all, so the
comparison does not compile; unlike a Phase D slice, this one cannot be subtracted from a working subsystem.
The breakages of §3.2 carry the whole weight of "the cases can see these rules", which is why there are
forty-eight of them.

**A tool-free test** in `MarsViTests` scans sixteen configurations of one frame buffer — replicated one to
one, resampled with a bias in both directions, thirty-two bit, both interlaced fields, a picture that shrinks
and is left to fade over three frames, a blank one, one pulled in at the left, a PAL one, and three that
scan no frame at all — and checks forty-one of angrylion's pixels, colour and coverage byte alike. It passed
on its first run. **Of the forty-five breakages it catches thirty-three.**

It took three versions to get there. The first scanned three frames of a single configuration and
caught nineteen; the eleven-scan version caught twenty-nine; the three no-frame scans and two pixels deep
inside an interlaced frame brought it to thirty-three. Each step closed a *class* rather than a rule: a test
where every scan produces a frame cannot hold a rule about **not** producing one, and a test that reads only
the first 240 rows cannot hold the rule that an interlaced frame is twice as tall.

**The twelve it still misses are worth naming, because they say what a tool-free test of this device costs.**
Five are geometry that its register sets do not separate — the vertical offset and its halving, stepping into
a picture that starts above the field, the cut at the raster's edge, and the two active-line rules. Four are
border and fade rules that need a picture narrower or shorter in one particular way: the left border's clear,
the count a blank signal clears, the foot's darkening, and the width of a spent line's clear. Two need a
fractional step where this test uses whole ones. One is the coverage a dark column keeps. Every one of the
twelve is held by a named case in `MarsViDifferentialTests`, so nothing is ungraded — what is missing is only
the guarantee that survives without the reference tools built.

## 4. What is not here

- ~~**Anti-alias modes 0 and 1**, which read a pixel's coverage out of the hidden bits and filter by it.~~
  **Landed the same day, in `Mars_VideoFilter.md`.** They are the modes a game actually uses.
- ~~**The dither filter, divot and gamma**, each a further pass over a fetched pixel.~~ **Landed in
  `Mars_VideoPasses.md`**, where the gamma dither turned out not to be gradable at all.
- **The per-scanline registers** parallel-rdp models and angrylion does not reach through this dump format.
- ~~**The interrupt and the timing.** The interface's interrupt fires when the current half line reaches the
  interrupt register, and nothing here advances a half line.~~ **Landed in `Mars_VideoTiming.md`**, which
  deleted `Mars_Microcode.md` §3's stand-in for the video interrupt and kept its test passing.
- ~~**The frame anyone can see.** `GetFrameBufferRgba` is not wired to this raster yet; `ICore` is still
  unsatisfied, as `Mars_Gameplan.md` §4.5 has it.~~ **Wired 2026-09-18 in `Mars_Core.md` §2**, which copies
  this raster, line-doubles a progressive field, and replaces the coverage byte with an opaque alpha.

## 5. The referee

**There are no disputes here to referee**, so the FPGA implementation was asked about mechanism instead —
the standing practice since 2026-09-17 (`Mars_RdpReferee.md` §0 is the reading rule, and this is the first
slice where the referee's own code is the more natural source). Three rules that look like software
conveniences are in the N64_MiSTer core's video interface in its own vocabulary:

- **The eight- and seven-column guard is hardware.** `VI_outProcess.vhd` §154–162 computes
  `H_GUARD_START <= VI_H_VIDEO_START + 8` *only when* the start is at or past the standard offset (108 NTSC,
  128 PAL), and `H_GUARD_STOP <= VI_H_VIDEO_END - 7` only when the end is within 640 of it. Pixels outside
  are forced dark through a `bi_guard` flag. That is §2.5's rule, including the exception for a picture that
  starts early, arrived at by someone making a real signal.
- **The interpolation rounds the same way.** `bi_add <= bi_mul + 16; bi_shift <= bi_add(14 downto 5)` —
  multiply the difference by the five-bit fraction, add sixteen, shift down five (`VI_outProcess.vhd`
  §332–336). §2.6's arithmetic, to the constant.
- **The register fields are where §1 puts them**, named and commented in `VI.vhd` §113–131: origin
  twenty-four bits, width twelve, horizontal video start at bits 25–16 and end at 9–0, the same for vertical,
  the vertical sync ten bits of half lines, and the current-line register documented as the field number.

**What it does not confirm** is the raster's two frames of grace (§2.4). The FPGA core scans out to a real
display and has no prescale buffer to keep, so the question does not arise there in the same form. That rule
rests on both software references agreeing, which — given §0 — is less independent than it sounds, and it is
the first thing to test on a console.
