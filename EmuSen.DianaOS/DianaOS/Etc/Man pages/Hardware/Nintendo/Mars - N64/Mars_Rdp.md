# Mars — the display processor's interface, its command stream, and the fill cycle

*Landed 2026-09-17. Phase D's first slice: the registers through which the CPU and the RSP hand
the display processor a list of commands, the stream those registers deliver, and the one way
of drawing that the hardware corpus exercises — filling a rectangle with a single colour. No
triangle, texture, combiner, blender, depth test or coverage exists yet (§9). The code is
`Memory/DpInterface.cs`, `Rdp/Rdp.cs` and `Rdp/Rdp.Fill.cs`; the tests are `MarsRdpTests` and
the second case in `MarsMicrocodeTests`.*

*`Mars_Documentation.md` §4.1 asked that the RDP's page say which of its rules came from where.
§0 is that ledger, and every rule below carries its tier.*

*Updated the same day: the fill cycle's pixel rules are now graded against the reference
rasterizer (`Mars_RdpDifferential.md`), and one of them did not survive (§5.2).*

---

## 0. Where each rule came from

| tier | source | what it settles here |
| --- | --- | --- |
| **hardware** | the corpus's `tests/rdp/mod.rs`: six groups at its `RDPBasic` level, run by default and not behind the experimental flag | the start, end and current registers, the start-valid rule, freeze and xbus, the status word during a run (§2) |
| **community** | the n64brew command reference, as already cited in `Mars_Microcode.md` §4 | command numbers and lengths (§3) |
| **reference reading** | angrylion in the parallel-n64 checkout at `39819865`, read for mechanism under `Mars_References.md` §2 and never run | how a fill writes its colour, the width field, the address width, which rectangle edges a fill includes (§5) |
| **commercial program** | Wave Race 64's first display list and its interrupt handler, measured | supporting evidence for §5.1 and §6, and the observations in §8 |
| **reference, graded** | `MarsRdpDifferentialTests`: angrylion, cross-checked by parallel-rdp (`Mars_RdpDifferential.md`) | which pixels a fill covers, and how its colour lands at every pixel size (§5) |

A rule of the third tier is a reading of another implementation's source; a rule of the fifth has
been run against it. `Mars_TestOracle.md` §5 says what running it certifies — agreement with the
community's model, not with the hardware — and reading it certifies less.

> **Retired 2026-09-17: "Nothing on this page is graded against the reference yet."** True when
> written, and the differential that followed within the day changed a rule it had covered (§5.2).

## 1. What the slice is

The corpus's display processor tests are not rasterization tests. Two of the six check the
registers alone. The other four hand the processor a list, check its registers along the way,
and read one pixel of an 8×8 framebuffer the list filled in the fill cycle. So the slice is the
interface, a command stream correct in its lengths, and enough of the fill cycle to put a
colour where those tests look — and no further, because every further pixel rule is the kind
`Mars_Documentation.md` §4.1 says must be established by comparison rather than read.

**159 → 152 on the corpus's own tally**: the seven failure lines of the six groups, and nothing
else (§7).

## 2. The interface

Four registers at `0x0410_0000`: start, end, current and status. A program writes start, then
end; the processor takes every command between current and end.

### 2.1 Start, end and current

- **All three keep twenty-four bits, word-aligned**: a write is masked with `0xFF_FFF8`. The
  corpus writes `0xFFF`, `0xFF_FFFF`, `0x12FF_FFFF`, `0x1280_0000` and `0xFFFF_FFFF` and reads
  each back through all three. **[hardware]**
- **A start write is held until an end write takes it.** Writing start sets *start-valid*;
  while it is set, further start writes are ignored; writing end copies start into current and
  clears it. A start write does not move current. **[hardware]**
- **A frozen end write does not set end-valid.** That is the corpus's only statement about
  end-valid, which it qualifies itself — *"at least not while frozen"*. Mars never sets the bit,
  and what sets it on hardware is not established here.

The reading this gives the start-valid rule: a list cannot be moved under one that is still
being handed over. That is an argument for why the rule would exist, not evidence that it does;
the evidence is the corpus.

### 2.2 The status word during a run

The corpus hands over an eight-command list one end write at a time and waits, after each, for
an exact status value. Every value it waits for is met by this model:

- an end write while not frozen sets **start-gclk** (`0x08`) and **pipe-busy** (`0x20`);
- a full sync clears both;
- **buffer-ready** (`0x80`) is always set.

So the status is `0xA8` after an end write that stops before a full sync, and `0x80` after one
that reaches it — or `0x81` with the xbus bit set. **[hardware for the values, model for the
rule]**

What this does not establish:

- **Buffer-ready as anything but a constant.** In Mars every word handed over has already been
  taken (§3), so the buffer is never full. What hardware reports when it is full is not known
  here.
- **Whether a frozen end write starts the clock.** Mars says no; after one, the corpus reads
  only the two valid bits.
- **DMA-busy.** The corpus's comment says hardware sets it briefly as each command is copied and
  that it deliberately does not assert it. Mars never sets it.

### 2.3 Data memory as the source

With the status register's xbus bit set, commands come from the RSP's data memory instead of
RDRAM, and the addresses are offsets into it. **The end address is compared, not masked**: a
list from `0xFF0` to `0x1028` runs, reading past the end of data memory and wrapping to its
start. The corpus's own comment on that test infers *"an internal START < END check"*, and
that is what Mars does — it takes words while current is below end, and reads each byte at its
address modulo `0x1000`. **[hardware]**

### 2.4 The status write, and the registers after it

Four bits are modelled, the four the corpus uses: clear and set xbus (`0x1`, `0x2`), clear and
set freeze (`0x4`, `0x8`). **[hardware]** A write naming both halves of a pair does nothing,
**by analogy with the RSP's status register** (`Mars_Rsp.md` §5.1), where that rule is
measured. For the display processor it is not: the corpus lists *"set and clear bits at the same
time"* among its own untested cases. The other bits community documentation assigns — flush,
the busy-flag clears, the clock counter — are ignored.

The four registers after status — the clock counter and three busy counters — read zero. Before
this slice they read back whatever was last written, from the dictionary `Mars_Memory.md` §2.2
describes; the display processor's block no longer goes through it.

### 2.5 Freeze

A frozen processor takes nothing; thawing it takes whatever is pending. **The corpus's authors
say hardware does not quite do this**: a TODO in its source records that current should advance
up to start + 240 while frozen, as commands are transferred without being run. That is not
modelled, and for that reason no Mars test asserts current while frozen — only that nothing is
drawn until the thaw.

### 2.6 The list carried out on another thread

*2026-09-19, Phase G.* In play the display processor is a quarter to a third of the emulation thread's frame
(`Mars_Performance.md` §26), and everything above describes a processor that runs its list *inside* the end write:
`Take` reads every word up to end and runs it before the write returns, so the full sync's interrupt, the status
word and every byte the list draws are in place at the cycle of the write. That instantaneous model is what the
corpus graded and what the probe's baseline holds, and it is kept exactly. What moves is the work. With
`MarsCore.ThreadedRdp` set (`DpInterface.Threaded`), the interface still reads the words at the write — from data
memory or RDRAM, as §2.3 says — and still decides the full sync at the word that completes it, by gathering the
stream as the processor's `Accept` does (§3), so the interrupt and the status are raised at the same instant as
before; but the words go into a ring, and a thread from the pool runs them through the processor while the machine
goes on.

**The argument that nothing can tell.** The processor reads and writes RDRAM and the hidden bits (the colour
image, the depth image, the texture image it loads from), and everyone else on the machine reads and writes RDRAM
too. The two sides agree with the instantaneous model if, and only if, every access by anyone else to a byte the
list will touch happens after the words that touch it have run. So the interface keeps a **mark per 4 KB page of
RDRAM**: the count of words handed over up to and including the command that reaches the page. Every other reader
and writer of RDRAM tests the mark of the page it is about to touch — one load and one compare on the fast paths —
and where the mark is set, waits until the thread's count of words run has reached it, then clears the mark,
since nothing later has marked the page. The sites are the bus's word read and word write, through which every
byte, halfword and doubleword access and every device's DMA passes; the processor's own direct load, store and
fetch (`Mars_Performance.md` §17) and the entry of a block (`Mars_Recompiler.md` §2.3), whose words are compared
against memory; the signal processor's DMA, a range at a time; the audio interface's sample fetch; the serial
interface's PIF transfer; the scan-out's capture (`Mars_Video.md` §2.7); a cheat's read and write; the debugger's
memory spaces; and the interface itself, reading a list from RDRAM. A state written or read joins the thread first,
so a state holds the finished drawing and a loaded one is not drawn over.

**Which pages a batch marks.** The interface shadows what the processor's registers say a draw reaches: the
colour image's address, width and pixel size, the depth image's address, the scissor's top and bottom rows, and
the texture image's address, width and texel size, from the set-image and set-scissor words as they pass. At the
first draw of a batch — a triangle, a textured rectangle, a fill — every row the scissor admits of the colour image
and of the depth image is marked; a load command marks the bytes it reads, a block's one linear run or a tile's
rows from its first column to its last, each widened by the sixteen-byte window `ImageWord` reads through. A draw's
mark says *everything handed over so far*, since the batch's later draws are not yet known, and when the batch ends
the interface writes over those pages the batch's own count, so a page drawn last frame is waited for only until
last frame's words have run — the scan-out of a finished buffer, and a display list beside a depth buffer, would
otherwise wait for a frame still being drawn. A batch that changes images more times than the interface remembers
leaves them at *everything so far*, which is slower and still right.

**The processor checks the argument as it draws.** Every byte of RDRAM the processor reads or writes passes
`Touch`, which — in Debug builds and when `EMUSEN_MARS_VERIFY_RDP` is set, since it costs a compare a pixel — asks
whether the page's mark reaches the word being run, and records the first byte for which it does not as a fault the
next wait or join rethrows. So a run that ends without a fault has *proved*, byte by byte, that the marks reached
everything the list touched. The probe's 1,800 frames and 600 frames from each of `Mars_Performance.md` §26's
gameplay states have run this way without a fault; §7 says what the tests pin.

**Ordering.** A word is marked for before it is published: the shadow runs, the marks are written, then the ring's
count moves, so a waiter that sees a mark can always be satisfied by words the thread can see, and the thread never
runs a word whose marks are not yet down. A batch that finds the ring full waits for the thread, and a batch larger
than the ring gets a larger one. The thread is a pool item that drains what is published, lingers a moment for
more, and hands its thread back; a waiter that finds no drainer starts one. A fault on the thread — the processor's
own or the verifier's — stops the drain, lets every waiter through, and is thrown to whoever waits or joins next.

**What it does not do.** The instantaneous model's consequence remains: a game told by the full sync that its
frame is drawn swaps the display to that buffer at once, and the scan-out then waits for the thread to finish it,
so the tail of a frame's list is on the emulation thread's path whenever the list is handed over late in the frame;
what the thread buys is the part of the list issued while the machine still had work to do. The processor's own
speed is unchanged, and on its own thread it is the frame's floor: in Wave Race its list takes longer than a
sixtieth of a second. The debugger's memory windows hand out the RDRAM array itself, and a window read from another
thread while the list runs may see a partly drawn frame, as it may already see a partly run frame.

### 2.6.1 Readers, writers, and the bytes behind a mark

*2026-09-19, later the same day.* The first version of the marks conflated two things, and the interface's own
counters, printed from the gameplay states of `Mars_Performance.md` §26, showed what each cost. **A mark did not
say what the processor would do to the page.** A page a load reads from carried the same mark as one an image is
drawn into, so a *reader* of that page — the signal processor's DMA fetching its display list, a block's fetch of
code, the audio interface's samples — waited for a load it had no order with: two reads of the same bytes can
happen in either order and agree. **And a mark named a page, not the bytes.** A page is 4 KB, an image's rows or a
load's bytes seldom fill one, and a bystander sharing the page waited for the whole of it: Ocarina of Time keeps its
display list in the page after its depth buffer's last rows, so the interface's own reading of the list, and the
signal processor's, waited for the batch that cleared the depth buffer; Super Mario 64's and Wave Race's signal
processors DMA from pages their textures are loaded from. Between them these waits were 4 to 7 ms of every frame.

**Two marks a page.** The interface now keeps `Marks`, the count a *writer* of the page must wait for, set by
every range the processor reaches, and `WriteMarks`, the count a *reader* must wait for, set only by the images the
processor writes. The sites of §2.6 are sorted by what they do: the bus's word read, the processor's load, its
fetch and a block's entry, the audio interface's samples, the scan-out's capture, the signal processor's DMA into
its own memory, the serial interface's transfer into PIF RAM, a cheat's read and the interface's reading of a list
test the reader's mark; the bus's word write, the processor's store, the DMAs into RDRAM, a cheat's write and the
debugger's window test the writer's. The verifier splits the same way: a byte the processor *writes* passes
`Wrote`, which asks the reader's mark — since the readers are the ones a write can mislead — and a byte it
*reads* passes `Touched`, which asks the writer's.

**The bytes behind a mark.** Every range that marks a page is also kept as a range: a ring of the last 1,024,
each its first and last byte, the count of the word that first reaches it, the count that must run before it is
free, and whether the processor writes it; and, for the batch still being taken, the image ranges it marked idle,
whose count is not yet known. A waiter that finds its page marked, and the mark not yet run, consults them before
waiting: a writer for any range that overlaps its bytes, a reader for one the processor writes; finding none, it is
a **bystander** — it neither waits nor clears the mark, which still speaks for the range's bytes, and the
interface counts it. The scan runs from the newest range back and stops at the first whose count has run, which is
sound because the ring is appended in the order of the counts: a load's range as its word is taken, and a batch's
image ranges when the batch ends, with the batch's own count, which no word of the batch exceeds. Two rules keep
the page marks consistent with the ranges. A later mark never lowers an earlier one — a load on a page whose image
is still idle leaves the idle mark, since the batch's later draws are not yet known — except at the batch's end,
which writes its count over the idle marks it made. And a wait clears a page's mark only when the mark waited for
is at least that mark, so an idle mark, or a later load's, stands.

**The verifier is now exact to the byte.** A word `w` that touches byte `b` must find a range that contains `b`,
whose first word is at most `w` and whose count is at least `w`, and a range the processor writes if the touch is
a write; the page check of §2.6 is made first, as before. The ring is read newest first and stops where the counts
fall below `w`; the open batch's ranges are read under a sequence the taking side increments before and after it
changes them, and read again if it changed meanwhile. Two limits, both taken as covered so that neither can fault:
a range older than the ring's 1,024 entries, which the thread detects by the count of appends having moved past its
slot while it read; and a batch with more image ranges than the sixteen the interface remembers, which §2.6 already
leaves idle. Neither has been reached by any run in §7 or `Mars_Performance.md` §29.

**An image's extent is not the scissor's rectangle, and the verifier said so.** The first version of the ranges
took the colour image's extent to be the scissor's rows times the image's width, and the verifier rejected it
within forty frames of every gameplay state: the processor read and wrote bytes inside the marked pages and outside
those ranges. The reason is that **a span's right edge is clipped by the scissor and not by the image's width**
(§2 of `Mars_RdpTriangles.md`; `ClipEdge` clamps to `_scissorRight`), while an address is formed as
`base + (y × width + x) × bytes`. A scissor wider than the image therefore carries an address past the end of its
row — and on the last row, past the end of the image, which is where Ocarina of Time keeps the buffer after it. The
extent a batch marks is now the least and greatest address a span can form: from the top row's first pixel, less two
for a read beside a span, to the greater of the bottom row's first pixel and the bottom row's scissor-right pixel
plus three. The rows themselves were also one too many, `bottom − top + 1` where the rasteriser draws `top` to the
row the bottom edge ends in, exclusive; the bottom is now the row its quarter-pixel edge ends in. With both
corrected the verifier accepts every run in §7 and `Mars_Performance.md` §29.

**Two limits the first version simply lost.** A batch that named more images than the interface remembered left its
pages idle and recorded no ranges at all, so a touch from one of its words found nothing and faulted; such a batch
now appends a single range of the whole of RDRAM, which is the behaviour of §2.6 before this section and costs only
what that cost. And the ring of ranges was a thousand entries, while the thread legitimately lags the machine by
ten thousand words and more, so the ranges that spoke for the word being run had been written over; the ring is now
eight thousand, and a scan that reaches its oldest entry, or finds its slot rewritten underneath it, reports that
it has nothing to say rather than a miss. Both were found by the verifier and neither by reasoning, which is the
argument for having it.

**What a fault now says.** The first form of the message named the byte and the word and nothing else, and neither
defect could be told from it. It now carries the shadow's images and scissor, every range the open batch holds,
the last twelve of the ring, and — the line that settled both — the nearest range in the ring that holds the byte
at all, with whichever of its two counts disagreed spelled out. The two defects above were each diagnosed from one
run of forty frames after that was added. The strings are built only where a fault is being constructed.

**What remains.** A reader of bytes inside an image still being drawn waits, as it must: the scan-out of a buffer
the game displayed at the full sync, and Ocarina of Time's processor reading its depth buffer's page for the rows the
list does not share. Those are the list's own speed on its thread, §2.6's last paragraph.

### 2.7 The thread held between two words

*2026-09-19.* A state must hold what the thread drew, so writing one joins it (§2.6), and at a frame boundary the thread
is a list behind: the rewind buffer's capture every fourth frame cost Wave Race 64 a 7 ms join each time
(`Mars_Performance.md` §34). A snapshot for that buffer holds the thread instead. `Pause` raises a request the drain
reads before every word and in its linger; the thread answers by standing still between two words, or by leaving,
and only the machine's thread can start it again, so once `Pause` returns nothing writes RDRAM until `Resume`. The
state's body is then written as of the words run so far, and the words handed over but not run follow it
(`WritePending`); a load runs them at once on the loading thread (`ReadPending`), with the byte-level verifier off
for the replay, because the join that precedes a load has cleared the ranges it would check against, and with the
full sync's interrupt not raised again, since the interface answered it when the word was handed over. The cost to the
thread is the standstill, under a millisecond a capture. `A_snapshot_taken_while_the_thread_stands_loads_to_what_the_finished_list_leaves`
pauses before a list is handed over, so every word of it is pending, and proves the loaded machine equal to the list
run at once.

### 2.8 The list shared by several processors, each shading the rows that are its by count

*2026-09-20, Phase G. Built from §10.2's pricing; the measurements are `Mars_Performance.md` §35.* The interface can
hold more than one processor (`Workers`, `MarsCore.RdpWorkers`), each a complete `Rdp` on a thread of its own, each
reading every word of the ring in order and running every command, and each shading only the rows whose number
modulo the count is its index. That is angrylion-plus's shape (§10.1), and it fits Mars better than it fits angrylion
because the processor was already one object with no thread in it: the walk, the loads and the register writes are
simply done N times, which costs about a fifth of one processor's work, and the shading, the other four fifths, is
divided. The first processor is `Processor`, the one the state serialises and the immediate path runs; the others
are made when the count is set, copied from it (`CopyStateFrom`, the serializer's own bytes and the decoded modes),
and discarded when it is set back.

**A word is done when every processor has run it.** `Completed()` is the least of their counts, and everything the
machine's thread reads — a mark's wait, the ring's room, a join, a snapshot's tail — reads that. Each processor
publishes its own count after every word; nothing else is shared between them but RDRAM, its hidden bits and the
ring, and the ring's slots are reused only past the least count.

**Four things a shared list cannot let happen, and the step each command is given for them.** A command's step is
decided by every processor alike at the word that completes it (`Classify`), from the processor's own registers, so
no processor tells another anything: the decision is deterministic because every register was set by a command they
all ran in order. The only decision that once depended on the interface's timing — whether a load's bytes are being
drawn — was made deterministic by tracking the furthest pixel any draw could have reached in each image since it was
set (`Reach`, the interface's own extent for an image), which the barrier at every image change makes sufficient.

1. *A carry that crosses rows* (§10's two): the previous pixel's stored depth slope, read by a two-cycle primitive
   whose first blend selects memory alpha, and the combiner's previous result, selected as COMBINED by the second
   cycle in one-cycle mode or the first in two-cycle mode. Such a primitive is `Step.Leader`: every processor arrives
   at a barrier, the first draws every row alone, and every processor arrives again before any goes on. Before it
   draws, the first takes the scratch each field's last writer in raster order left (below), so the slope it starts
   from is the raster order's.

2. *A span that writes past its row.* A span is clipped by the scissor and not by the image's width, so a scissor
   past the width lets a pixel of row *y* write row *y* + 1's bytes, which another processor owns. In one and
   two-cycle modes the pixel at exactly the width has no coverage and writes nothing, so only a scissor, or a
   rectangle's own right edge, past the width in any fraction serialises the primitive; in fill and copy modes the
   scissor's own column is drawn (§5.2), so reaching the width at all does. A primitive whose colour and depth images
   overlap is serialised for the same reason: one row's depth word is another row's colour bytes.

3. *A load from bytes a draw may have written*, since the primitives before it may still be drawing on another
   processor, and the primitives after it may write its source before another processor has read it. Such a load is
   `Step.AllJoined`: a barrier, every processor's own load into its own texture memory, and a barrier.

4. *An image change*, which maps the rows onto other bytes, so that a row of the new image is a row of the old one
   by another count. `Step.All`: a barrier, then every processor's own register write. The barrier is a counting
   spin barrier with a generation, three to nine passes a frame in the recorded frames, and it lets everyone through
   when a processor has faulted so that the fault, not a hang, is what the machine's thread sees.

**The pixel at the width, which reads and does not write.** With a scissor equal to the image's width — the usual
case — a span clipped by it visits the column at the width, whose bytes are the next row's first, and the pixel reads
them (the memory colour, and the stored depth slope) with zero coverage and writes nothing. Nothing in the picture
depends on the read, but the scratch it leaves does — `_memory`, the blend shifts and the stored slope are in the
state, and the slope is read by the next live primitive — and its value depends on when the read happens relative to
the next row's owner. So, for a shared primitive whose last shaded row's last pixel is at or past the width, the
processor that *owns the next row* makes those reads itself, at its own end of the primitive (`RecordAliasedRead`):
at that moment it has finished every row of its own of every primitive up to this one and started nothing after, and
no other processor writes its rows, so what it reads is what raster order reads. The processor that shaded the row
still makes the read at the pixel, and what it reads is timing; the stamps below discard it. This is the one place
the split needed something angrylion does not have, and the test that proves the need
(`The_read_past_a_rows_end_is_made_by_the_next_rows_owner_before_it_runs_on`) is the day's best instrument: without the
record it fails three runs in five, which is what a frontend would have seen.

**What a join assembles.** The state serialises every field of the first processor, and the scratch fields hold
their last write in raster order — a different pixel for each field, as §10.2 found. Every processor stamps each
scratch field it writes with the primitive's count over the row (`Stamp`), the fields that every pixel writes once at
the row's start and the conditional ones at the write; at a join, at a snapshot's pause and at a live primitive's
barrier, the first processor takes each field from whichever processor's stamp is the latest (`TakeScratchFrom`),
the coverage buffer entry by entry. The tables the walk writes need no stamps: every processor walks every primitive
it does not draw alone, and the ones it skips are rewritten by the next walk before anything reads them. The
processors' own stale scratch is never read for output — every field but the two carries is written by a pixel
before that pixel reads it — which is the fact §10.2's inventory established and this design rests on.

**A snapshot with several processors** (§2.7) needs a point every processor stands at. The pause point is the furthest
command boundary any processor has reached when it sees the request: each raises the point to its own boundary if
that is further, runs on to it if it is short of it, and stands there; the request is answered when all stand at the
same point. A processor short of the point can never be waited for at a barrier by one past it, because a barrier
is inside a command and the point is a boundary. The cost is that the slowest processor runs to the fastest's
boundary, which is about a primitive.

**What is exact, and how it is known.** `MarsThreadedRdpTests` hand over fills, shaded triangles clipped at the
scissor's right edge, a two-cycle scene with memory alpha in its first blend, a load from the drawn image, an image
set one row into the last, and a snapshot, to two, three and four processors, and compare RDRAM, the hidden bits and
the state byte for byte with the list run at once. Each of the four mechanisms above was removed in turn and its
test failed — every run for the serialisation, the load and the image change, three of five for the aliased read.
The rasteriser bench replays the three recorded frames of §7 with the same RDRAM hashes at one, two and four
processors; the probe grades 1,800 frames with the byte-level verifier on. None of it is a proof of the design;
all of it is what the design predicts and nothing yet contradicts.

## 3. The command stream

Words are taken into a buffer one at a time, and a command runs once all of its words have
arrived. The length comes from the first word's command number **[community]**:

- a triangle (`0x08`–`0x0F`) is four words, plus eight for shade, eight for texture and two for
  depth;
- a texture rectangle (`0x24`, `0x25`) is two words;
- everything else is one.

So an end write that stops inside a triangle leaves the triangle waiting, and none of its later
words is read as a command — which matters, because those words are coefficients and any of
them can carry a byte that reads as a full sync. The tests put exactly that byte there.

**Current reports words taken, not commands run.** That is the reading consistent with the
corpus's freeze TODO in §2.5, which describes current as a transfer pointer. With nothing
pending the two are the same.

**Everything is instantaneous.** A list runs inside the end write that hands it over — including
an end write made by the RSP's microcode, halfway through an RSP instruction. There is no
transfer time and no per-command cost, and `Mars_Documentation.md` §5 records that no document
supplies either.

## 4. The commands that act

Six do something: full sync (`0x29`), set scissor (`0x2D`), set other modes (`0x2F`, of which
only the cycle type is read), fill rectangle (`0x36`), set fill colour (`0x37`) and set colour
image (`0x3F`).

Every other command is taken whole and does nothing. For the three other syncs — load, pipe and
tile — doing nothing is arguably correct already: they wait for work in progress, and a
processor that finishes each command inside the write that delivered it has none. That is a
reading about a model with no pipeline, not a claim about hardware. For the rest — triangles,
texture rectangles, colours, the combiner, textures, the depth image and the tile commands —
doing nothing is simply unbuilt.

> **Update 2026-09-17: the eight triangle commands act too**, in the fill cycle, through the edge
> walker that fill rectangles now share (`Mars_RdpTriangles.md`).

## 5. The fill cycle

### 5.1 The colour is written as whole words

**Every byte a pixel occupies takes the byte of the fill colour at that address's position in a
big-endian word.** So a 32-bit image gets the colour; a 16-bit image gets its upper half on
pixels at even 16-bit indices and its lower half on odd ones; an 8-bit image cycles through the
colour's four bytes by address. **[reference reading]** The reference's per-size fill functions
compose to this, and its vectorised 16-bit path says so in a comment: a fill writes the colour
*"as a 32-bit word to each physical framebuffer word"*.

Two pieces of evidence support it, and neither proves it:

- **Wave Race's depth clear sets the fill colour to `0xFFFC_FFFC`** — the depth value packed into
  both halves. A program would need that under this rule. It would also be harmless under a rule
  that uses only the lower half, so the packing is consistent with this rule without excluding
  the other.
- **The corpus's list assembler takes two colours for a 16-bit fill**, one per half, and its
  tests pass the same colour twice.

The resemblance to `Mars_Memory.md` §2.4, where the RSP's memories latch whole words from the
CPU, is noted and not used as an argument; the two are different parts on different buses.

**Hidden bits are not written.** The reference also sets RDRAM's hidden per-pixel bits from the
colour's low bit. Mars has no hidden RDRAM yet.

> **Update 2026-09-17: they are now**, and the differential compares them (`Mars_RdpCoverage.md` §3.1).

> **Update 2026-09-17: graded.** The differential's 32-bit, 8-bit-at-an-odd-address,
> 16-bit-at-an-odd-address and odd-width cases match angrylion, and parallel-rdp agrees with it on
> each (`Mars_RdpDifferential.md` §4). The rule is now **[reference, graded]**; the evidence above
> is kept as what it rested on before.

The colour image's **width is stored one short**: the reference adds one to a ten-bit field, the
corpus writes eight-wide buffers as `7`, and Wave Race's 320-wide image is `0x13F`. Its
**address keeps twenty-four bits**, per the reference, which matches the register mask the
corpus measures in §2.1.

### 5.2 Which pixels a fill covers

**Graded, and the rule Mars now implements is `Mars_RdpDifferential.md` §4.2's**: the rectangle's
edges clipped into the scissor in eighths of a pixel, rows chosen by quarter-pixel sub-scanlines,
and the scissor's interlace bits honoured. It matches angrylion on every fill case the differential
grades, apart from one recorded artefact of the reference itself. Of the
ungraded rule below, **the scissor's right edge was wrong** — its column is drawn — and so was the
statement that fractional coordinates are wrong here: for a fill rectangle the reference's sub-pixel
walk lands on the same pixels truncation does. Its bottom-row and scissor-bottom claims held for
whole pixels.

> **Retired 2026-09-17, by the differential's first run: the ungraded rule and its evidence.** Kept
> because it stated which parts were readings and which an assumption, and the assumption is the
> part that failed.
>
> **The rule in Mars:** coordinates are taken as whole pixels (their two fraction bits dropped); a
> fill covers its rectangle's left through right columns and top through bottom rows inclusively;
> and the scissor clips with its left and top edges inside and its right and bottom edges
> outside.
>
> **This rule is not graded, and the corpus cannot grade it**: its fills and scissors coincide, on
> 8×8 buffers, and it reads pixel 0. What there is:
>
> - **The bottom row is inside for a reason the reference shows.** In the fill and copy cycles it
>   forces the rectangle's lowest sub-scanlines in before walking it. **[reference reading]**
> - **The scissor's bottom edge is outside, on a reading of the reference's row limit.**
>   **[reference reading]**
> - **The right edges were not traced.** The reference decides them through sub-pixel spans with a
>   sticky bit, and reading that code closely enough to be sure is precisely the work §4.1 of
>   `Mars_Documentation.md` says must be done by comparison instead. The column rule mirrors the
>   row rule by assumption.
> - **Wave Race cannot discriminate.** Its scissor and its fill rectangle have the same corners,
>   (8,20)–(311,219). Under Mars's rule the clear stops one column and one row short of the
>   rectangle; under a scissor that includes its far edges it would not. §8 records the pixels.
>
> **Fractional coordinates are simply wrong here.** The reference places sub-pixel edges; Mars
> truncates. Every list this slice has run — the corpus's and Wave Race's — uses whole pixels. The
> scissor's interlace bits, which restrict drawing to odd or even rows, are ignored.

### 5.3 What the fill cycle does not do

- **A 4-bit colour image draws nothing.** The reference records the pipeline as crashed. What
  hardware does is not established. parallel-rdp aborts on it, so that one case is graded against
  angrylion alone (`Mars_RdpDifferential.md` §4.4).
- **Past the colour image's right side**, Mars writes on into the next row by address, and the
  reference stops at the last column through a validation workaround that is not a hardware claim
  (`Mars_RdpDifferential.md` §4.3). Neither is measured against hardware.
- **A fill under some combinations of image read and depth settings** also crashes the pipeline in
  the reference. Mars draws it.
- **A fill rectangle outside the fill cycle draws nothing, and that is known to be wrong**: in every
  other cycle the reference draws it through the same span renderers as a triangle, with the
  combiner and blender, neither of which exists here. **[reference reading]** §7 records that no
  test pins this, deliberately.

## 6. The full sync and its interrupt

A full sync stops the clock (§2.2) and raises the display processor's interrupt, bit 5 of the
interrupt aggregator. Writing bit 11 (`0x800`) of the aggregator's mode register clears it; the
display processor has no clear of its own. The bit's meaning is the one community documentation
gives, which this slice did not re-read.

**The evidence it rests on here is a commercial handler.** In Wave Race the interrupt is raised at
instruction 63,710,383, during the first graphics task, and cleared at 63,710,618 — 235
instructions later. The mode register's bit 11 is the only thing in Mars that clears it, so the
game's handler wrote that bit.

## 7. What the corpus and the tests say

**159 → 152, and 83 failing groups → 77.** The prediction stated before the run was seven failure
lines in six groups and a tally of 152, and it held: the six RDP groups — including the one named
*RSP STATUS* — are gone, and the 77 that remain fall into the census's other three rows with the
same counts as before (`Mars_Corpus.md` §12).

**The nineteen cases in `MarsRdpTests` were written first, and all nineteen failed against the
dictionary stub.**

**Twelve mutations were run against them, and eleven were caught**: removing the start refusal,
the copy of start into current, the address mask, the freeze check, the full sync's clock stop,
the xbus source, the command buffer, the whole-word lanes, the exclusive scissor, the width's
extra one or the mode register's clear. **The survivor is deliberate.** Letting a fill rectangle
draw outside the fill cycle fails nothing, because the only behaviour a test could assert there
is today's placeholder, and §5.3 says it is wrong.

The depth-clear case in `MarsMicrocodeTests` was checked the same way: with the fill writing
nothing, it fails at the pixel loop, expecting `0xFFFC` and finding zero.

> **Update 2026-09-17.** One of those caught mutations, "the exclusive scissor", guarded a rule the
> differential then refuted. A mutation check shows that a test pins a behaviour, not that the
> behaviour is right. The unit test now pins the graded rule — the scissor's right column drawn, its
> bottom row not — and the fill rule's own eleven mutations are recorded in
> `Mars_RdpDifferential.md` §4.6.

**The threaded list, 2026-09-19** (`MarsThreadedRdpTests`, verification on). A scene of fill rectangles over a
whole frame buffer with a depth image set, handed over to a bus that runs its list at once and to one that runs it
on the thread: the interrupt register and the status word agree at the end write, and after the join RDRAM, the
hidden bits and the whole bus state agree. Six such scenes with a state written after each, while the thread still
runs, all identical. A read of the image being drawn returns the drawn pixel every time over twenty scenes, and
finds the page marked before and unmarked after. Pages no command reaches are unmarked, the colour and depth images
and a texture's source are marked, and a join clears them. ~~What no test here forces is a fault from the verifier;
that it fires is shown only by inspection of `Touched`.~~

**Readers, writers and bystanders, 2026-09-19** (§2.6.1, the same class). A texture's page is marked for writers
and not for readers; a word written to that page beside the load's bytes is counted a bystander and leaves the
mark; one written inside them waits and clears it. A list placed in the page after the depth image's last row is
read by the interface without a wait — every word a bystander — and the drawing agrees with the list at once, in
RDRAM and in the whole state. And the verifier's two faults are forced: told the processor wrote a byte in a page
nothing marked, and then one in the colour image's last page beyond its rows, it records each, and the join that
follows throws it, with the message naming which; a third scene after both joins clean.

## 8. Wave Race's first list, carried out

The commands `Mars_Microcode.md` §4 tabled can now be read as what they do. **The list's one fill
is a depth clear.** It points the colour image at the depth image's address, `0x2A_0000`, 320
wide and 16-bit; sets the fill colour to `0xFFFC_FFFC`; fills (8,20)–(311,219); and then points
the colour image at `0x3B_5000` before the full sync.

After the task, measured:

| depth pixel | value |
| --- | --- |
| (0,0), (7,20), (8,19) | `0000` |
| (8,20), (100,100), (310,218) | `FFFC` |
| (311,218), (310,219), (311,219) | `0000` |

The last row is §5.2's ungraded rule, visible.

> **Update 2026-09-17, under the graded rule:** (311,218) now holds `FFFC`; (310,219), (311,219)
> and (312,218) still hold `0000`. The scissor's right column is cleared and its bottom row is not
> (`Mars_RdpDifferential.md` §4.7).

The test asserts only what the old rule could not reach, and still does:
every pixel at least one pixel inside the rectangle's edges holds the colour, with the rectangle,
colour and image read from the list rather than written into the test, and pixel (0,0) is
untouched.

**Carrying the list out did not change what the game does next.** With the same stand-in pulses
(`Mars_Microcode.md` §3), 200 million instructions on the unmodified bus and on this one both give
**one graphics task and 122 audio tasks**. The interrupt is raised and acknowledged, and no second
graphics task follows. That retires the reading `Mars_Microcode.md` §6 gave for why the game
submits only one. The first task's break also arrives eighteen instructions later than on the
unmodified bus; that difference was not investigated.

## 9. What is not here

- **Every other way of drawing**: triangles and the edge walker that turns their coefficients into
  spans — which is where the undocumented sub-pixel rule lives — texture rectangles and the copy
  cycle, the combiner, the blender, depth testing and update, coverage and anti-aliasing, dither,
  textures and texture memory.
- ~~**Hidden RDRAM**~~ — built with coverage (`Mars_RdpCoverage.md` §3.1).
- **The span registers** at `0x0420_0000`.
- **Any timing** (§3).
- ~~**The reference differential**~~ — built the same day (`Mars_RdpDifferential.md`). It grades
  the fill cycle; every other drawing path above has nothing to grade yet.

## 10. Whether the rasteriser could be split across scanlines, and what blocks it

*2026-09-19, Phase G. A feasibility study, not an implementation.* §2.6 put the list on one thread and
`Mars_Performance.md` §29 to §31 leave three of the four gameplay states bound by that thread's own speed, 1.2 to
1.9 µs a word. angrylion-rdp-plus renders a display list on several threads by giving each one the rows where
`row % workers == worker`, so the obvious question is whether Mars can do the same while keeping §2.6's rule that
output is bit-identical. It is asked here because the answer is mostly yes, and because the two things that block
it are worth recording whether or not it is ever built.

**Mars is better placed for this than angrylion is.** In angrylion the worker's share of the rows is folded into
the same `validline` flag that the level-of-detail path reads when it peeks at the next scanline, so with more than
one worker that peek is never taken and the output changes with the worker count. Mars's edge walk (§2's walker)
writes `_spanDrawn`, `_spanLeft`, `_spanRight`, `_spanAttributes` and the sub-scanline edges **for every row of the
primitive before any shading starts, with no thread term in them** — `_spanDrawn` folds in only the scissor and the
interlace field. So Mars's own next-row peek would stay exact under a split, and only the shading loop would need
partitioning. That is the architecture angrylion lacks and that parallel-rdp had to build a second dispatch to get.

**Two dependencies genuinely block a bit-identical split, and both are narrow.**

*The previous pixel's stored depth slope.* `Rdp.Depth.cs` keeps `_pastStoredEncoded` from one pixel to the next and
derives the first blend cycle's shifts from it, with no reset at a row or a primitive, so a row's first pixel uses
the previous row's last. It cannot be recovered by recomputing one pixel redundantly: it is the slope that pixel
*read out of RDRAM*, and whether the pixel before it wrote there depends on its own depth test. It is live only in
two-cycle mode with the first blend selecting memory alpha, which is a mode bit the processor already computes.

*The combiner's previous result.* `_combined` is read as the combined input and then overwritten, so within a
primitive it chains from the first pixel to the last across row boundaries. Recomputing it for a row would mean
replaying the whole primitive. It is live only when a combine mode selects COMBINED, which is decidable when the
combine word lands.

**Both are real hardware, not Mars's invention — and the study is the reason to believe that.** The MiSTer
register-transfer code carries `combiner_save` and `combine_alpha_save` as clocked registers with no reset, in its
own identifiers rather than angrylion's, which makes the one-cycle combiner feedback a genuine agreement rather
than a borrowed one. The depth-slope carry is the opposite: MiSTer's nearest equivalent is a same-pixel pipeline
alignment delay, and parallel-rdp has no such value at all — so there the references **disagree with angrylion**,
and Mars follows angrylion. That disagreement is a question for `Mars_RdpReferee.md`, not for threading.

**What a split would have to do**, if it is ever built: render primitives in those two modes on one thread, decided
by predicates computed where the mode words land; give every thread its own copy of the per-pixel state and the
coverage buffer, which are today fields on the one processor; join before the next command, since primitives read
each other's output; and either clamp a span to the colour image's width or fall back to one thread when the
scissor exceeds it, because Mars does not clamp today and two rows can otherwise address the same bytes. That last
one is a latent defect of its own, independent of threading, and is recorded in §9.

**Not attempted.** Three of Mars's four measured states are bound by this thread, so the prize is real; the work is
not small, and nothing here is built.

### 10.1 What the references and the field do, read on 2026-09-19

Read after §10, from the local checkouts and from what is published; mechanism only, as `Mars_References.md` §2
requires.

**angrylion-rdp-plus.** Every worker replays every buffered command against its own complete copy of the processor's
state — the span table, the combined colour, the previous pixel's stored depth slope, the coverage buffer, a copy of
texture memory — and rasterises the rows where `row % workers == worker`; state commands, loads and edge setup are
run N times and only the span shading is divided. Commands buffer until a full sync, a full buffer, or an image change
in the medium and high compatibility profiles, and the flush blocks the emulator thread until every worker is done,
with worker 0 on the calling thread. That is what §10 said, with three additions. First, the checkout is Themaister's
fork, and it carries determinism patches upstream lacks: the combined colour cleared at every primitive, the noise a
function of the pixel and the primitive count rather than a per-worker seed, the memory colour readable in the
blender's first cycle. Mars clears `_combined` at the start of every walk too, which is the fork's rule, and is graded
against the fork. Second, the next-scanline peek — the level-of-detail and texel paths that read the row below's
span — is gated on that row's `validline`, which under more than one worker is never the same worker's, so the peek
is dead and the fallback runs: the output's dependence on the worker count is silent, and no comment, option or
readme names it. Against that, one published test found Super Mario 64's RDRAM byte-identical over 300 frames across
one and four workers, so the peek does not bite in every scene. Third, the conformance oracle runs angrylion
single-threaded, bypassing the command buffer, which is the configuration Mars is graded against. Published speed is
thin: one report of Super Mario 64 doubling on a four-core part, several of no scaling at all, and one that an odd
worker count beats an even one on a six-core, which is worker 0 contending with the emulation thread.

**parallel-rdp** renders on the GPU and is not a rasteriser to split, but its synchronisation is the design Mars's
marks approximate from the other side: a full sync is a flush, not a wait; the CPU blocks only on a real read or
write of RDRAM, through a timeline whose values name a submission of at most 256 primitives, eight passes or a
millisecond, rather than a batch to the next full sync. Its integrator ares waits fully at every full sync regardless.
gopher64's integration keeps a dirty bitmap of RDRAM at eight-byte granularity, marked by each draw's colour and depth
images, and waits only when an access intersects a dirty run — the nearest published analogue to §2.6.1's ranges,
armed at the full sync rather than at the word. Dolphin bounds its GPU thread by a distance window in emulated cycles
and blocks the CPU only at a read of the GPU's output. mupen64plus, Project64 and parallel-n64's angrylion all run
the list synchronously on the CPU thread, so they have nothing to say about the wait.

**What this changes for §10.** Nothing in its two blockers; both are confirmed to be what a bit-exact split must
handle and what the references chose not to. The precomputed span tables remain Mars's advantage: the peek angrylion
loses under a split is a read of Mars's own tables, which every thread can see. What the survey adds is a warning
about the emulation thread: angrylion's worker 0 shares the calling thread, and Mars's split must not — the drain's
thread is the natural owner of a worker, and the machine's thread is not.

### 10.2 The split, priced from an inventory of the rasteriser's state

*2026-09-19.* An inventory of every field the shading loops write — per-pixel carry, same-pixel scratch, per-row
scratch, per-primitive read-only state, and memory — was taken to price §10's design. It confirms the design in
most of its parts and changes it in three.

**Confirmed.** The two carries of §10 are the only ones that chain a pixel to a later pixel across a row boundary:
`_combined` (in one-cycle mode pixel n reads n−1's result; in two-cycle mode the next pixel's first cycle reads this
pixel's second, and the row's synthetic pixel past its end is what the next row's first pixel reads) and
`_pastStoredEncoded` (written by every pixel that reaches the depth compare in either mode, read only in two-cycle
mode with the first blend's second alpha selecting memory alpha). `_blenderShadeAlpha` and `_lodFraction` look like
carries and are not observable across rows: the row's first pixel recomputes the former, and the latter is recomputed
whenever it is read. The coverage buffer is rebuilt per row within `[left, right]`, and nothing reads a previous row's
entries. Dither is a function of the pixel's position and the mode word; noise is not built at all. Every read and
write of RDRAM a pixel makes is at the pixel's own address, colour and depth alike, and the two-cycle "next pixel"
reads are the coverage of `x + direction` and a synthetic first cycle that touches no memory. Texture memory is
written only by loads, never by shading.

**Three things the inventory adds.**

1. *The stored depth slope crosses primitives.* `_pastStoredEncoded` has no reset anywhere: a two-cycle primitive's
   first pixel reads what the previous primitive's last pixel left, including a one-cycle primitive's, while fill and
   copy leave it alone. A primitive drawn on one thread in the live mode must therefore start from the value the
   raster-order last pixel of the previous primitive left, which under a split lives on whichever thread drew that
   primitive's last row.

2. *Two aliasings of the registers make rows share bytes.* A span is clipped to the scissor and never to the image's
   width, so a scissor wider than the image sends x ≥ width into the next row's bytes (§9's latent defect, which the
   marks accommodate); and the colour and depth images are independent addresses, so one pixel's depth word can be
   another row's colour word. Neither is a carry, but both make two rows' bytes overlap, and a split must draw such a
   primitive on one thread. Both predicates are decidable from the registers when the primitive lands.

3. *The state format holds the scratch.* The serializer walks every field of the processor not marked skipped, so
   `_combined`, `_pixel`, `_memory`, `_shade`, the texels, the blend shifts, `_blended`, `_pastStoredEncoded`,
   `_lodFraction` and the whole coverage buffer are in a state, each holding its last write in raster order — and that
   is a different pixel for each: the last pixel that passed depth and coverage for `_blended`, the last pixel to reach
   the compare for the slope, the last row's synthetic pixel for the two-cycle texels. A split that keeps the state
   identical must, at a join, assemble each of these from the thread that holds the raster-order last write: for the
   fields every pixel writes that is the thread of the last drawn row, and for the conditional ones the row of the last
   write has to be remembered beside the value.

**The shape this prices.** The angrylion shape — each worker a complete processor of its own, replaying every word,
owning the rows where `row % workers == worker`, sharing only RDRAM and its hidden bits — fits Mars unusually well
because the processor is already one object: N workers are N instances of `Rdp` fed the same words, each walking
every primitive's spans (the walk is under two per cent of the thread) and shading a share of the rows. Shading is 75
to 85 per cent of the thread's time, so N workers cost about 0.2 + 0.8/N of one: 1.7× at two, 2.5× at four. What
must be serialised is exactly the list above: a primitive in the live mode or either aliasing is drawn by one worker
after the others have finished the words before it and before they take the words after; a load whose source
overlaps bytes the batch has written waits for every worker, which §2.6.1's ranges can already decide; an image
change re-partitions the rows' bytes and waits likewise, which is angrylion's medium profile; and a join, for a state
or a reader, waits for every worker's count. The verifier runs per worker unchanged. The exactness proof is the one
§2.6 used — the list at once against the list split, byte for byte and state for state, on scenes built to exercise
the live mode, both aliasings and a load from a drawn image — and then the probe and the differential. §10.1's warning
stands: no worker may be the machine's thread.

**Built the next day, as §2.8**, to this shape; what the inventory did not foresee — the read at the width, made by the next row's owner — is that section's fourth paragraph.
