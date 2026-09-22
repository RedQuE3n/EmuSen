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

### 2.8 A scan that would repeat the last one

*2026-09-19, Phase G.* The games this phase measures change their frame buffer on every second or third field and
the interface scans on every field (`Mars_Performance.md` §26), so a half to two thirds of scans walk bytes the
previous walk already walked. §2.7 made the walk run on another thread over a capture of the bytes it can reach,
and that capture is what makes a redundant scan cheap to recognise.

**The argument.** The walk reads two things and nothing else: the registers and derived geometry the job carries —
the picture, the origin, the width, the pixel size, the resample, divot, anti-alias, dither-filter and gamma modes,
and the byte range §2.7 computes — and the bytes of that range, with their hidden bits. It keeps no frame counter,
reseeds no noise and reads nothing from the machine while it runs, which is what separates Mars from every reference
read for this: angrylion reseeds the gamma dither's noise from a field counter (`vi.c`, `reseed_noise(…,
vi_frame_count)`) and parallel-rdp from a frame count pushed to the shader, so in both a scan over identical bytes
produces a different picture, and neither can skip one exactly. In Mars the walk is a pure function of those two
inputs, so a scan whose geometry and bytes both equal the last walk's writes the raster the last walk left, byte for
byte — and the picture already shows it.

**The mechanism.** The job keeps the geometry of the last scan it captured, and its capture is written only when a
walk follows, so the capture still holds the last walk's bytes. `Capture` compares the geometry first; if it
matches, it compares the live range of RDRAM and its hidden bits against the capture, after the display processor's
marks for that range have been waited on as §2.7 already does. If both match, no bytes are copied and no walk is
queued, and the presentation keeps the picture it holds rather than swapping in the pending one — which is right
because a deferred picture is one frame behind, and the frame it would have shown is the same picture. If either
differs, the bytes are copied and walked as before. The borders and the two frames of grace of §2.4 are advanced in
`Prepare`, which always runs, so a skipped walk leaves them as a walk would have.

**A loaded state forgets the capture**, since the raster it presents at once is not the one the last walk wrote;
the scan after a load always walks.

**Measured.** Interleaved, order rotated, three rounds of 600 frames from each of `Mars_Performance.md` §26's
gameplay states on a quiet machine, second halves, with the scan-out and the display processor's list already on
their own threads:

| from the state, second 300 frames | walking every scan | skipping the repeats | scans skipped of 600 |
| --- | --- | --- | --- |
| Ocarina of Time (PAL, 50) | 49.9 fps | **52.2** | 399 |
| Wave Race 64 (NTSC, 60) | 59.6 | **60.4** | 400 |
| Super Mario 64, in the castle (PAL, 50) | 73.5 | 73.0 | 300 |
| Super Mario 64, outside it (PAL, 50) | 55.2 | 55.6 | 299 |

Medians of three. The first two rows' rounds do not overlap; the last two lie inside their own spread, and the
change is recorded as worth nothing to them. The counts are the games' own draw rates: two scans in three repeat in
the first two, one in two in the other two, exactly as §26 said they would. **The gain follows the scan-out's share
of a frame rather than the count of scans skipped** — §26 put it at 24 per cent of Ocarina of Time's frame and 11
and 17 of the others', and the two Super Mario 64 states are bound by the display processor's thread, where work
taken off the emulation thread buys nothing.

**A first version cost Wave Race what it saved.** It copied the bytes out and then compared the copy with the
previous one, so a skipped scan still paid for a full copy of the range; Wave Race's median fell from 60.7 to 59.8
while Ocarina of Time's rose. Since the previous capture is overwritten only when a walk follows it, the comparison
can be made against live memory instead and the copy taken only when the bytes have changed — which deletes the
copy on the half to two thirds of scans that skip, and a whole second buffer with it. That is the version measured
above, and it turned Wave Race's loss into a gain.

**What no reference does.** The study behind this section (`Mars_References.md`) found every scan-out read for it —
angrylion, its parallel-n64 fork, parallel-rdp, the MiSTer register-transfer code and mupen64plus-core — decides
not to scan only on register state: a zero origin, two blank fields in a row, an invalid width, and in parallel-rdp
a self-described "dirty hack" that reuses the previous image when the registers are invalid. None looks at whether
the bytes changed. The fork's own measured campaign attacks the cost of a scan — it found 97 to 100 per cent of
pixels fully covered and doing no filtering, and skipped the vertical resample at a zero fraction — and Mars already
had both of those (§2.6 and `Mars_Performance.md` §5 and §21).

### 2.9 The walk at a multiple

*2026-09-20.* When the display processor draws at a multiple (`Mars_Rdp.md` §11), the scan-out walks the multiple's
memory into a raster N times as wide and as tall, and that is what the frontend shows. `Prepare` sets the job's
`Scale` to the processor's multiple once it has drawn there (`ScaledDrawn`) and one otherwise, and builds a second
`Picture` from the first with its left, top, columns, rows, stride, active lines, start column and row and first and
last column multiplied by N and its two steps unchanged: the source is N times wider and the output N times wider,
so the fractional step an output pixel takes is the same, and the multiple's detail is in the raster being N times
denser. The job takes the multiple's lines as it takes the console's (`ReachScaled` and the capture), so a walk on
another thread (§2.7) reads one frame, and the repeat test of §2.8 includes the multiple, so a change of it is never
a repeat. `Walk` chooses the multiple's source, picture and raster and records which it walked (`OutputScale`);
`Raster` returns that raster and `OutputWidth` its width; `Darken` writes the darkened lines into both rasters, and a
blank clears both. `MarsCore.Compose` returns the width it composed and multiplies the rows by the multiple, and
`ScreenWidth` follows it, so the frontend receives a 640×480 frame at two and 1,280×960 at four and scales it as it
would any other. Nothing about the walk's rules changes: the same registers, the same fetch, the same filters, on a
denser source.

### 2.10 Antialiasing: the multiple averaged down

*2026-09-20.* `MarsCore.Antialiasing` (Off, 2x, 3x, 4x) is supersampling, built from §2.9 and nothing else: the
display processor draws the picture that many times finer each way than it is to be shown, the walk of §2.9 scans
that drawing out at its full density, and `Raster` averages each square of the dense raster into one pixel of the
picture (`BoxAverage`: the rounded mean of the square, channel by channel). The drawn multiple is the internal
resolution times the averaging, `DpInterface.Scale = RenderScale × Antialiasing`, and the picture the frontend
receives is the internal resolution's: 2x resolution with 2x antialiasing draws at four and shows the picture at two. The product is held to four, the limit §11.1 of `Mars_Rdp.md` measured as already a quarter of full
speed, and the averaging gives way first: at 2x resolution 4x antialiasing is 2x, and at 3x or 4x there is none
(`EffectiveAntialiasing`, `Antialiasing_multiplies_the_drawing_and_is_held_to_four_with_the_resolution`).

**Why the average is taken in `Raster` and not in the walk.** `Darken` and the blank path write the dense raster
after the walk has filled it, and a line nothing rewrites is held there between frames (§2.4); an average taken at
the end of the walk would miss them, and one taken where the raster is read sees whatever the scan left. It also
runs on whichever thread composes the frame, which with §2.7 is not the machine's. It is taken only when the dense
raster's multiple is one the averaging divides, so the frame between a change of either setting and the next
drawing shows the console's raster or the dense one whole, never a torn one.

**What it is and is not.** The console's own antialiasing — coverage blended at silhouette edges by the display
processor, and the video interface's filter over it — is the game's choice and is untouched; it runs on the dense
drawing as on any other. This averaging is over the top of it and smooths what the console's cannot: interior
edges, texture shimmer, and the stair steps the console's 320 columns leave. The mean is of the stored bytes, not of
linear light, which darkens a high-contrast edge slightly; a gamma-correct mean was not tried. Seen on Majora's
Mask's Clock Town introduction, frame 2,200, with the `framedump` harness at Off, 2x and 4x: the rooftop diagonals
lose their steps at 2x and the textures resolve further at 4x. The cost is the drawn multiple's, exactly §11.1's
table, plus the average, which is a few milliseconds at four on the composing thread and was not separately timed.
`The_average_of_a_square_is_its_rounded_mean_channel_by_channel` holds the arithmetic.

### 2.11 Fixed: the walk at a multiple lost the origin's high bits

*2026-09-21. Found while measuring the GPU path at four (`Mars_Gpu.md` §11.5); the defect is older than that work and
has nothing to do with it.*

**What happened.** At three or four, a frame buffer above one megabyte scanned out as the wrong part of memory on the
immediate path and crashed the deferred one — *"the scan reached frame buffer address DA9400 outside the lines
captured for it"*. Deferred presentation is on by default, so a player choosing an internal resolution of four in a
game that keeps its frame buffer high in memory, which Super Mario 64 does, saw the emulator stop.

**Why.** The VI's origin register holds twenty-four bits, and the console never needs more: eight megabytes is
twenty-three. The memory at a multiple is bigger by the multiple squared, and at four an eight-megabyte machine's is
128 megabytes, which is twenty-seven bits. `Walk` computed the origin at the multiple as `job.Origin × n²` and passed
it to `Fetch`, and `Fetch` then aligned it with `& 0xFF_FFFE`: a mask that aligns *and* keeps twenty-four bits. On
the console the second part is invisible. At a multiple it cut 0x3DA9400 to 0xDA9400. `ReachScaled`, which decides
what a deferred scan captures, aligns the register's value first and multiplies after, so it held the right lines,
and the walk then reached for lines it had not been given.

**The two paths failed differently, and one of them silently.** The deferred path checks every read against what it
captured, so it threw. The immediate path reads the whole shadow, so every address was in bounds and nothing was
checked: it drew whatever lay at the truncated address, which is usually memory nothing had drawn into. That is the
worse of the two failures, because it looks like a picture.

**The fix** aligns the register's value first and multiplies after, in `Walk` as `ReachScaled` and `Reach` already
did, and `Fetch` stops masking what it is given. For the console's own scan the address is what it was, bit for bit,
since the aligned twenty-four-bit value is exactly what `Fetch` used to compute; the golden probe at one is unchanged.

**Coverage.** `A_frame_buffer_high_in_memory_scans_out_at_a_multiple_as_a_low_one_does` draws eight bands of colour
into a frame buffer at 512 kilobytes and again at three megabytes, and requires the two scans to be the same picture,
at two, three and four, on the immediate path and on the deferred one. Against the unfixed code the two cases at two
pass, since three megabytes times four is twelve and fits; the four at three and four fail, the immediate ones on the
picture and the deferred ones on the capture. All six pass with the fix. At two the defect is still reachable in
principle, by a frame buffer above four megabytes on an eight-megabyte machine, which this test does not construct.

**What this retires.** `Mars_Rdp.md` §11.1's speed numbers at four, and `Mars_Gpu.md` §11 and §12's claim that the
device and the CPU path agree at four, were all taken on this defect. The timings stand, since a walk over the wrong
lines is the same amount of work as a walk over the right ones. The agreement at four stood only in the sense that
both paths were wrong in the same way for any game whose frame buffer is above a megabyte, and it was taken again
with the fix (`Mars_Gpu.md` §11.5).

### 2.12 The walk in bands, side by side (2026-09-21)

At one the walk runs on the processor, on the deferred thread (§2.7), and the next frame's end joins it. Timed in
place, the walk took 4.9 ms a walk in Super Mario 64, 8.4 in Ocarina of Time and 3.9 in Wave Race, starting 0.015 ms
after it was queued, and the join waited 1.44, 1.84 and 0.62 ms a frame for it: nineteen, seventeen and eight per
cent of the emulation thread, which the sampler showed as its largest native wait.

**Rows are independent, so the walk is split into bands.** Two things looked like carries from row to row, and
`Mars_Gpu.md` §13.1 showed neither is: the slot cache and the line window are caches of a pure function of memory
and the registers, so any order of asking gives the same answers, and the fetch bug's counter has a closed form, two
where a row reads its line again and one where the row before did. Each band therefore has caches of its own (the
walk's state moved into a nested `Walker`), starts its counter from the closed form at its first row, and writes rows
of the raster no other band writes. The bands are contiguous, as many as a quarter of the processors and at most
four, with no band under thirty-two rows; a picture smaller than that is one band, as before.

**Measured** (`pacebench`, flat out, 7d68f4d against this, interleaved and rotated, three rounds, medians, every state
hash the same): at one, Mario ran at 324 per cent of full speed against 261, Ocarina at 217 against 169, Wave Race at
240 against 221; at two on the processor, Mario 142 against 105, Ocarina 86 against 66. The golden probe's pictures
were identical for all six hundred frames of both its games, and a mutant starting every band's counter at zero
failed fifteen of the video tests.

**What this does not cover.** On a machine of four processors the walk is still one band, and a weak machine is
where the join costs most; there the device's walk (`Mars_Gpu.md` §13) or a walk started earlier would be the
lever. The bands share the processors with the emulation thread and the display processor's workers, so their
count is a guess at a fair share, not a measured optimum.

### 2.13 Fixed: the walker's stamps wrapped into its own memo (2026-09-22)

*Found while porting the walk to MarsRT (`Mars_Native.md` §5.4.2); the defect is as old as the pre-divot memo of
`Mars_Performance.md` §21.*

**The mechanism.** A walker keeps two memos of a slot's samples, `_samples` after divot and `_plain` before it, each
entry tagged with the stamp of the line its slot held when it was made (`Mars_Performance.md` §3 and §21). A stamp is
a counter that only rises, so a tag equal to the slot's stamp means the entry is this line's. The counter wraps at
`int.MaxValue` and starts again at 1, and at the wrap `NextStamp` cleared `_sampledRow` and not `_plainRow`. Any
plain entry still tagged with a small stamp from before the wrap was then taken for the new line that reached the same
stamp after it.

**What it takes to see it.** An entry is re-tagged whenever a row touches it, so the stale one is found only if the
first touch after the wrap falls exactly on the stamp it was left with. That needs an offset no scan reached for the
whole of the counter's span, which is a picture mode used once and then not again: a wide picture, a narrow one for
two thousand million lines, and the wide one back at the right stamp. At one or two stamps a row, the span is about ten
hours of continuous play in one process at the most (a 480-row interlaced picture reading two new lines a row, in one
band) and days at the usual rate. No run has been shown to meet it; the case below constructs it.

**The case.** `A_scan_after_the_walkers_stamps_wrap_reads_no_sample_from_before_it` scans a forty-row picture 640
columns wide, stepping at half a pixel, then the same picture 256 columns wide and 37 rows tall, over one frame buffer.
It sets every walker's counter to one below the wrap by reflection, which stands in for the narrow picture's two
thousand million lines, paints the frame buffer anew, and scans the narrow picture and the wide one again. The first
wide picture left the outer offsets of its two slots tagged 39 and 40, from its last two rows; after the wrap the
narrow picture takes stamps 1 to 38 and never reaches those offsets, so the wide picture's first two rows are stamped
39 and 40 again. The raster is compared with an interface that ran the same four scans without the jump.

**Measured.** Against the code as it was, 2,886 bytes differed, all in rows 0 and 1 and columns 257 to 632: the outer
part of the first two rows showed lines 38 and 39 of the old frame buffer. With `_plainRow` cleared beside
`_sampledRow` the rasters are identical. Each clear alone is needed: without `_plainRow`'s, 2,886 bytes differ; without
`_sampledRow`'s, 2,894. The golden probe was not run, since the counter cannot wrap in its 600 frames, and
`MarsViTests`, `MarsViDifferentialTests`, `MarsDeferredPresentationTests` and `MarsRTViTests` pass unchanged.

**The claim this retires.** `Mars_Performance.md` §21 says the pre-divot memo "is stamped by the slot's line as
`Remembered` is", which was true of the tags and not of the wrap: until this fix the two memos were exact only below
the counter's first wrap. MarsRT's walker cleared both from the start (`Mars_Native.md` §5.4.2), so the two
implementations now agree at the wrap as well.

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
