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

*Retired 2026-09-21 by §2.6.2: most of what this paragraph called unavoidable was not a reader of bytes being drawn
but a mark wider than the draw. Wave Race's scan-out wait went to nothing and Ocarina's fell, once each draw marked
only its own rows.*

### 2.6.2 Each draw marks its own rows (2026-09-21)

§2.6.1 marked, at a batch's first draw, the whole scissor of both the colour and the depth image, and kept that mark
for the batch. A census of the waits by call site, with the last word to touch each eight bytes of memory tracked
beside them, found that most waits were for bytes **no pending word would touch**. In Ocarina of Time the RSP's
memory transfer into the page after the depth buffer waited for about twelve thousand words when forty to six
hundred reached its bytes, and the processor's loads of its depth buffer waited on words that reached none of them.
In Wave Race the scan-out's capture reached a line past the end of the buffer on show, into the first page of the one
being drawn next, which the new batch had claimed whole before drawing anything there. Super Mario 64's waits were
genuine: the last word to write the captured bytes was ninety-four per cent of the way to the mark.

**The mark is now each draw's own rows.** A triangle or rectangle marks the rows the walker can shade, from its own
limits (`Rdp.Walker.cs`'s `UpperLimit` and `LowerLimit`, the machine's processor only, since the interface never
drives the multiple's) to the scissor, with §2.6.1's reach past a row's end and two pixels of slack. Fill and copy
modes draw a rectangle's bottom row whole, and the mark does. **The depth image is marked only in the one- and
two-cycle modes with depth compared or updated**, since fill and copy never reach it. Within a batch, one extent per
image grows in place as draws reach further, under the sequence the verifier reads the open batch by, so the batch
still ends with one range an image; the extent keeps its first word, which the verifier's "the range started no
later than the word" test accepts for every draw it covers. After a state is read, the shadow takes the raw scissor
and the other modes from the processor (`Rdp.Bounds`), as it takes the images.

**Measured** (`pacebench`, flat out, 7d68f4d against this with and without the walk of `Mars_Video.md` §2.12,
interleaved, three rounds, medians, every state hash the same): the interface's waits went from 0.41 to 0.00 ms a
frame in Wave Race, and from 2.03 to 1.54 in Ocarina; Wave Race at one ran at 263 per cent of full speed against
240 with the walk alone, Ocarina at 223 against 217, and Mario unchanged. The verifier, in every Debug test and
forced on for 600 frames of each of the three games in Release, found no touch outside a mark.

**What this does not cover.** Ocarina's remaining waits are its processor's small reads of the depth buffer, which a
row mark still covers because the rows are the draw's; a test of those reads against the pending draws' edges at the
read's own row would take most of them, and was not built. Marking columns as well as rows was measured and bought
nothing over rows alone. Two tests changed meaning: the depth image is no longer marked by a scene drawn in fill mode,
which `Pages_no_command_reaches_are_not_marked_and_the_images_and_texture_source_are` now holds both ways, and a list
beside the depth image is found unmarked until the draw that reaches its page is shadowed. One unrelated test,
`A_page_only_a_load_reads_stops_writers_within_the_load_and_nobody_else`, failed once in twenty-one runs of its class:
it counts a bystander only if the thread has not yet finished the batch, a race of the test's own that this change
does not touch.

### 2.6.3 A small read waits only for the draws that write it (2026-09-21)

After §2.6.2, Ocarina of Time still waited 1.07 ms a frame for its processor's loads of the depth buffer, a seventh of
its frame, and 0.46 for the RSP's transfer into the page after it. The marks are ranges of rows; a load of four bytes
at one pixel waited for every pending draw whose rows reached the pixel's row.

**Each draw now also records a box**: the command's words as they came, the images it writes, and the word it ends
on, in a ring of sixteen thousand. A read of eight bytes or fewer that reaches a pending range scans the ring newest
first and waits only for the last pending draw whose box holds one of those bytes, or not at all. Only draws write
memory, so this is enough for a read. A write, and any transfer of a range, keeps §2.6.1's wait, since a write must
also wait for readers.

**What a box holds, and how that was arrived at, since each step was measured.**

- *The bounding box of the draw* (its rows, and its columns from its edges' ends, with §2.6.1's reach past a row's
  end, a span running on into the next row included) took the loads' wait from 1.07 to 0.95 ms a frame: 1,340 of
  1,938 narrowed reads in 900 frames were freed outright, the rest still found a box.
- *A triangle's own reach at the read's row*, from its edges over that row's four sub-scanlines, took it to 0.92.
  So the geometry was not what held them.
- A log of the first waiting reads showed what was: every one was a read of the depth buffer at a pixel a small
  triangle late in the frame really does cover, and **those triangles compare depth without updating it**. They read
  the depth buffer and never write it, and a read never waits for another reader. A box now names the depth image only
  for a draw that updates depth. The loads' wait fell to 0.51 ms a frame, and the interface's waits from 1.40 to 0.95.

**Sound by test, not by argument.** The verifier of §2.6.1 now also checks, for every byte the processor writes, that
it lies inside the box of the draw that wrote it; a draw with no rows records an empty box, so a write it made would
fault. Shrinking every box by four columns, or by one row, or narrowing the row reach to the edges themselves, each
fails eight to eighteen of the threaded tests with that fault. **It found one real defect before anything shipped**: a
command being gathered when a snapshot was read has its first words in the processor, not the ring, and the column
bound read stale words from the ring for it; such a command now takes whole rows. With the verifier forced on, 600
frames of each of the three games found no write outside a box, and every state hash was the same.

**The first version cost the other games.** Working out every draw's columns as it was shadowed made Mario and Wave
Race 1.4 and 1.6 per cent slower, Wave Race having no waits to save. A box now keeps only the command's words, and
its columns are worked out when a read or the verifier asks, which is rarely.

**Measured** (`pacebench`, flat out, 9536644 against this, five rounds alternated, medians, every state hash the
same): Ocarina at one 274 per cent of full speed against 258, Mario 356 against 350, Wave Race level at 280.

**What this does not cover, and one suspicion.** The RSP's transfer into the page after the depth buffer (0.44 ms a
frame) is a write of a range and keeps the coarse wait; narrowing it needs boxes that also name what a draw reads. And
the single-address wait that decides whether a read is a bystander at all checks the first byte of the access only,
so a four-byte read whose second half entered a pending range would not wait. Nothing has shown that happening, and it
is recorded here rather than changed.

*2026-09-22, from MarsRT's port (`Mars_Native.md` §5.6.2).* MarsRT tests every byte of an access, and no comparison
changed. A second finding corrects this section. The statement above, that "any transfer of a range keeps §2.6.1's
wait", does not hold for a read. `WaitForReadRange` narrows by the boxes that hold the range's first eight bytes. A
capture whose first bytes no pending draw holds is therefore freed, even while draws still hold its later rows.
`The_csharp_interface_lets_a_range_read_pass_the_draws_that_hold_all_but_its_first_bytes` shows it with the thread
paused. The C# is unchanged. *Retired 2026-09-23 (7fbf4b1): only a read inside one aligned doubleword is narrowed now,
and a range read keeps the page wait; §2.9.2.*

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

*2026-09-22, on item 3 (`Mars_Native.md` §5.6.6).* The reach is counted from the moment the image was set. A load
made before the image's first draw is therefore run apart, and the draws after it can write its source before a
slower processor has read it. ThreadSanitizer found this race in MarsRT's port of the rule, in Ocarina of Time, where
the frames matched by timing. It has not been shown in C#.

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

*2026-09-22: one field breaks that fact* (`Mars_Native.md` §5.6.6). A one-cycle primitive that computes no level of
detail writes each processor's own last `_lodFraction` at every pixel, with its row's stamp. So a primitive drawn
alone then assembles a stale value.
`The_csharp_split_assembles_a_stale_level_of_detail_fraction_from_rows_that_computed_none` finds the state wrong in
that field alone for 8 of 16 seeds. It never reaches a picture. *Fixed 2026-09-23 (51f3377): a one-cycle row stamps
the fraction only when it measures one; §2.9.3, which also records the same shape in the two-cycle path, not fixed.*

**A snapshot with several processors** (§2.7) needs a point every processor stands at. The pause point is the furthest
word any processor has reached when it sees the request: each raises the point to its own position if that is
further, runs on to it if it is short of it, and stands there; the request is answered when all stand at the same
word. A processor short of the point can never be waited for at a barrier by one past it, because a barrier is taken
at a command's last word and a processor stops only before gathering a word. The cost is that the slowest processor
runs to the fastest's position, which is about a primitive. *The sentence "a processor short of the point can never
be waited for at a barrier by one past it" is true and was not enough: the waiters that deadlock are at the point, not
past it (below, and §2.9.1).*

*2026-09-22: the argument has a gap* (`Mars_Native.md` §5.6.6). A processor waiting at a barrier for word *k* + 1 is
not past the point but at it. If the others were short of the barrier when the request came, they stand at *k*, and
the waiter never gets through. MarsRT's port deadlocked this way on four workers.
`The_csharp_workers_deadlock_when_a_pause_finds_some_at_a_barrier_and_the_rest_short_of_it`, run behind
`EMUSEN_MARS_DEADLOCK_PROBE=1`, finds the C# pause unanswered too. This is a candidate cause of the Super Mario 64
freeze described below, and it has not been shown to be that cause. *Fixed 2026-09-23 (a72533c): a waiter at a
barrier raises the pause point to its own word; §2.9.1. The freeze remains unexplained: the fix removes a cause that
could produce it, and no run has shown that it was the one.*

*The first version stood only at a command's boundary, and hung.* The play harness takes no snapshots, so the case
was met in the frontend, in play: a game hands its list over in pieces, and a piece can end inside a command — the
signal processor writes the end register as it fills the buffer, not at commands' ends — so every processor sat
waiting for the command's next word at the end of what was issued, none could reach a boundary, and the request
spun forever. `A_snapshot_while_every_processor_waits_inside_a_command_is_written_and_loads` hands over the first
words of a triangle, asks for the snapshot, and hangs on the first version; the fix is the paragraph above, and the
state a mid-command snapshot writes is exact because every processor gathers the same words, which the state
carries. Whether this was the freeze the first play test met in Super Mario 64 is inferred and not proven — the
process was gone before it could be looked at — and the frontend's rewind is off for the N64 until the snapshot
has been proven in play (`EmuSen_Settings_Reference.md` §4.21b). Two lessons from the test's own writing are worth
keeping: a hand-over in pieces rewrites the list's bytes in memory and the interface's registers, so a machine fed one
piece and one fed three differ in memory and state for reasons that are the test's, not the core's; both machines
must be fed the same pieces at the same places.

**What is exact, and how it is known.** `MarsThreadedRdpTests` hand over fills, shaded triangles clipped at the
scissor's right edge, a two-cycle scene with memory alpha in its first blend, a load from the drawn image, an image
set one row into the last, and a snapshot, to two, three and four processors, and compare RDRAM, the hidden bits and
the state byte for byte with the list run at once. Each of the four mechanisms above was removed in turn and its
test failed — every run for the serialisation, the load and the image change, three of five for the aliased read.
The rasteriser bench replays the three recorded frames of §7 with the same RDRAM hashes at one, two and four
processors; the probe grades 1,800 frames with the byte-level verifier on. None of it is a proof of the design;
all of it is what the design predicts and nothing yet contradicts.

### 2.9 The port's findings, fixed in the C# core (2026-09-23)

MarsRT's port of §2.6 to §2.8 (`Mars_Native.md` §5.6) found four places where this core was not exact, each recorded
with a WiseMan test that showed the C# behaviour, and left; its stage D (§6.4.4 there) found a race in the scan, and
its ThreadSanitizer run a race in §2.8's third rule that was argued for the C# and not shown. This section records
each one's cause, what was changed, the evidence before and after, and the mutant that reintroduces it. The deferred
repeat is `Mars_Video.md` §2.8's and is recorded there; the rest are the drain's. Every test named below was run on the
unfixed code first and failed there, and each mutant is the unfixed line put back alone into an otherwise fixed copy
of the source: its named test fails, and the rest of `MarsThreadedRdpTests` (32 tests then) or
`MarsDeferredPresentationTests` (19) passes, so no other test in the class depends on the defect.

#### 2.9.1 The pause barrier's deadlock (a72533c)

*The cause.* §2.8's pause point is the furthest word any processor has reached when it sees the request. A processor
that has gathered word *k* + 1, a command whose step is a barrier, and is waiting there for the rest, is inside
*k* + 1 and never returns to the top of its loop, where a processor stands; the rest, arriving at *k*, set the point to
*k* and stand. Nothing then moves: the waiter needs them at the barrier, and they wait for the point to change. With
two processors this needs only one to be a command ahead of the other at a barrier, which an image change makes every
few commands.

*The fix.* While it spins, a barrier's waiter raises an existing pause point that is short of its own word to that
word (`RaisePauseTo`). The processors standing at *k* see the point move, run word *k* + 1 through the barrier, and
stand at *k* + 1, where the waiter, now through, also stands. The raise is a compare-and-exchange from the value read,
and only of a point already set (at least zero): a waiter that read the request before a `Resume` cannot then write a
stale point over `Resume`'s −1, since its exchange expects the old value. The raise is counted (`PausesRaised`) so that
a test can say the case was met. The rest of §2.8's argument holds as written: nobody is ever past a barrier's word
while one of its waiters is still waiting, so raising the point to that word never asks a processor to go back.

*The evidence.* `A_pause_that_finds_some_processors_at_a_barrier_and_the_rest_short_of_it_is_answered` hands 1,500
fills alternating over four images to four processors and asks for a pause after a spin that differs per attempt, a
hundred attempts, each under a five-second watchdog; after each, the list is run out and the state compared with the
list run at once. `Pauses_and_snapshots_while_the_processors_run_barriers_are_all_answered_and_exact`, at two, three and
four processors, hands a list of 3,000 such fills over in two pieces and, from another thread, pauses, resumes and
takes snapshots forty times per round across eight rounds, under a thirty-second watchdog per round; each snapshot
is loaded into an unthreaded machine, its tail run, and compared with the finished list. Both assert that at least one
pause was raised, and that at least one snapshot carried pending words. On the unfixed code every case hung at its
first round (the pause test at attempt 0, the stress at round 0 for two, three and four processors); fixed, all pass,
and the probe `The_csharp_workers_deadlock_…` of `MarsRtThreadsTests`, which it replaces, is gone. The mutant that
never raises the point hangs both tests again. The deadlock is a hang and not a race on a byte, so it is shown by a
watchdog and not by the verifier: no byte is written wrongly, the processors simply never answer.

*What it reaches in play, and what it does not settle.* A pause is asked for only by a snapshot (`MemoryBus.WriteState`
with `snapshot`, which `Hold`s), and the only caller of a snapshot in Mistress is the rewind buffer, off for the N64
since the Super Mario 64 freeze of §2.8 (`EmuSen_Settings_Reference.md` §4.21b). So the deadlock could not have met a
player since then, and it is a candidate for that freeze, whose cause was never established. Whether rewind is turned
back on is a decision for play; this section says only that the hang the headless tests could reach is gone.

#### 2.9.2 A range read narrowed by its first bytes (7fbf4b1)

*The cause.* §2.6.3 narrows a read by the boxes of the pending draws that hold its eight bytes, and `Wait` applied
that to whatever range it was given, testing only the aligned eight bytes at the range's start. `WaitRange` calls it
page by page, so a page of a capture, an SP DMA or an SI transfer whose first eight bytes no pending draw held was
freed at once, although pending draws held its later bytes.

*The fix.* Only a read that lies inside one aligned doubleword (`(from ^ (to − 1)) & ~7 == 0`) is narrowed; a range
keeps §2.6.1's page and range wait. This is MarsRT's rule (`Mars_Native.md` §5.6.2).

*The evidence.* `A_range_read_whose_first_bytes_no_draw_holds_still_waits_for_the_draws_that_hold_the_rest` pauses
the thread with a fill of rows 8 and 9 pending and reads the page from row 6.4 as a range: unfixed, the read returns
at once and the bytes it lets through at row 8 are undrawn; fixed, it is still waiting after 500 ms, returns on
`Resume`, and reads the drawn bytes. The mutant that narrows every read fails it.

*What the fix costs,* since this is the busiest wait in the core: §2.9.7. *What it reached in play:* with the display
processor threaded and four processors, the fixed core skips 150 of Ocarina of Time's 300 scans as repeats and 149 of
Super Mario 64's, the counts it skips unthreaded; the code before the fixes skipped 82 to 87 and 131 to 132. Its
capture was freed while draws into the page past the buffer on show (§2.6.2's Wave Race case) were still being made,
so the bytes it compared, and copied, were changing under it. No picture was wrong, since those bytes are the capture's
slack and the walk does not reach them, but the capture was reading bytes still being drawn at every scan, which is
the defect's reach in play. The attribution to this fix alone is checked in §2.9.6.

#### 2.9.3 The level-of-detail fraction stamped by rows that measured none (51f3377)

*The cause.* `Rdp.OneCycle.cs` stamped `_lodFraction` with every row's stamp, and a row whose primitive measures no
level writes the processor's own last fraction at every pixel. With the list shared, each processor's own value is the
fraction of its own last measuring row, so the latest stamp could carry a value that raster order had overwritten.

*The fix.* A one-cycle row stamps the fraction only when it measures one (`measures`, the condition under which its
pixels compute a level). A row that does not measure leaves the fraction and its stamp alone, so the assembly takes
the value of the last row in raster order that did.

*The evidence.* `A_primitive_drawn_alone_after_rows_that_measured_no_level_assembles_the_raster_orders_fraction`, the
case of `MarsRtThreadsTests`' record moved here and turned round: sixteen seeds of a measuring scene, a split one that
measures none, and one drawn alone. Unfixed, the state differs in `_lodFraction` alone for 8 of 16 seeds, the memory for
none; fixed, all sixteen are exact. The mutant that stamps every row fails it.

*The same shape, left in the two-cycle path.* `Rdp.TwoCycle.cs` stamps the fraction at every textured pixel and writes
the processor's own value when level of detail is off, which by the argument above has the same defect for a
two-cycle textured primitive without level of detail, shared, followed by one drawn alone. MarsRT's `two_cycle.rs`
does the same. No test has shown it, since the synthetic scenes of `MarsThreadedRdpTests` draw no textured two-cycle
primitive, and the rule of this project is that a defect is shown before it is fixed; it is recorded here as the next
case to write, and a fix would have to be made in both cores at once to keep their comparison (`Mars_Native.md`
§5.6.7) meaningful. Like the one-cycle case, it cannot reach a picture: a combiner that reads the fraction turns
level of detail on.

#### 2.9.4 The scan deciding from the drain's progress whether the multiple drew (05d41e4)

*The cause.* `Vi.Prepare` chose the multiple's picture when `DpInterface.ScaledDrawn`, and on the drain that flag is
set by the processors as they draw, before `Capture`'s wait; the first scan after a load or a change of the multiple
therefore showed the console's picture or the multiple's by timing. `Mars_Native.md` §6.4.4 found it in MarsRT's port
and kept the C# comparison's oracle unthreaded because of it.

*The fix.* `ScaledDrawnHandedOver` joins the drain first when the flag is false and words are pending, so the answer is
that of every word handed over. Once true the flag stays true until a load empties the multiple's memory (§11), so the
join is paid only while nothing has drawn there, once per load or change, and never at one.

*The evidence.* `A_scan_decides_whether_the_multiple_drew_from_the_words_handed_over`, with one processor and with
two: the multiple at two, the drain paused with a scene pending and released from another thread after 300 ms.
Unfixed, `Prepare` returns at once with the picture at one where the list run at once gives two; fixed, it waits for
the release and gives two. The mutant that reads `ScaledDrawn` fails both.

#### 2.9.5 A load before the image's first draw, run apart (4291e9e)

*The cause.* §2.8's third rule joins a load when a draw since the image was set may have reached its bytes
(`LoadReachesDrawn`, over the drawn-to extents). A load made before the image's first draw was therefore run by each
processor apart, and the draws after it, which a fast processor runs while a slow one has not yet loaded, can write
the load's source first. Raster order has the load read first. MarsRT's port found the race with ThreadSanitizer in
Ocarina of Time (`Mars_Native.md` §5.6.6) and changed its rule; the C# was left, since a race that timing hides cannot be
shown by an equality test, and nothing had shown it.

*It was shown in play here.* The game comparison of §2.9.6, run while other agents' builds and tests kept the
machine's load at 10 to 20, parted the C# core with four processors from itself unthreaded, from Ocarina of Time's
state, in 10 of 16 runs across three modes: the state compared every frame (1 of 4), every sixtieth frame (3 of 4)
and a snapshot every frame (6 of 8). The first difference was in the CPU's registers (and once RDRAM) at frames 9, 78,
209 and 282, or in the picture's top rows at frames 84 and 288. The code before this branch, where it did not hang
(§2.9.1), parted in 1 of 8 of the same runs, at frame 84 as the fixed code did; so the race is older than the fixes,
and the fixes changed how often timing met it. The game reads its depth buffer from the CPU (§2.6.3), which is how a
drawing race reaches the processor's registers.

*The fix.* MarsRT's rule: a load is joined when its bytes meet the first 1,024 rows of the current colour image or depth
image at the colour image's width, the span a draw under any scissor can reach before the next image change, which is
itself a barrier. The drawn-to extents are no longer read, and are gone. More loads are joined, which costs barriers;
§2.9.7 measures it.

*The evidence.* With the rule, the same three modes ran 18 of 18 exact under the same load. The synthetic case,
`A_load_from_the_current_image_before_its_first_draw_is_run_by_every_processor_together` (MarsRT's, ported), fills a
second image, sets the first, loads from it before any draw and then fills over the loaded rows, forty times at two
to four processors. On the unfixed rule all forty ran the load apart and none left a wrong state, which is the point
MarsRT's page made: the test holds the rule, and only the games, by timing, or a race detector, show the race. The
mutant is the unfixed rule, which fails the test as the unfixed code did.

*What is not covered.* The rule is a superset: it joins loads that no draw of the image would reach. A load from an
image other than the current two, drawn earlier, is ordered by the image change's barrier and was never at issue.

#### 2.9.6 The core against itself in play

`MarsThreadedGamesTests` runs the core, with compiled blocks in both processors as it ships, from the three gameplay
states (`EMUSEN_MARSRT_STATES`), against itself unthreaded and presented at once, with the same input, 300 frames, and
compares the state and the picture after every frame: threaded on one processor and on four, at once; deferred alone,
threaded on one and on four, a picture a frame late; four processors deferred with the state compared every sixtieth
frame, so the drain runs across frames; and four processors deferred with a snapshot taken every frame and loaded into
a scratch machine. Every run reports the words its drain ran, so a run in which nothing reached the drain cannot pass,
and a deferred threaded run must skip exactly as many repeats as a third machine deferred and unthreaded.

| from the state, 300 frames | Super Mario 64 | Ocarina of Time | GoldenEye, the Dam |
| --- | --- | --- | --- |
| threaded, one processor, at once | exact | exact | exact |
| threaded, four, at once | exact | exact | exact |
| deferred, unthreaded | exact, 149 repeats | exact, 150 | exact, 54 |
| deferred, threaded on one | exact, 149 | exact, 150 | exact, 54 |
| deferred, four | exact, 149 | exact, 150 | exact, 54 |
| deferred, four, state every sixtieth frame | exact | exact | exact |
| deferred, four, a snapshot every frame | exact | exact | exact |

The drain ran 2.5, 1.8 and 5.6 million words on one processor, 10.1, 7.4 and 22.2 million counted over four (each
processor runs every word). The table is the final code's, run while the machine's load was about 12.

*What the snapshot row found before the fixes.* The same run of the code before this branch hung in Ocarina of Time in
3 of 3 runs, at frames 22, 9 and 34, a snapshot never answered within thirty seconds; hung in Super Mario 64 in 1 of 3,
at frame 166; and parted in GoldenEye at frame 6 in `_lodFraction` alone in 2 of 2, which is §2.9.3. So the pause
barrier's deadlock is met in play, by a snapshot every frame from Ocarina of Time's state within a second of play, and
the fraction in the Dam, as MarsRT's comparison had said. A snapshot every frame is heavier than the rewind buffer's
one in four, so this measures the rate an hour of rewind would meet, not the rate a player saw; §2.9.1's statement
about the Super Mario 64 freeze is unchanged.

*The mutants against the games.* The range read's (§2.9.2) passes every state and picture comparison and fails the
repeat count in every deferred threaded mode: Ocarina of Time skips 78 to 82 repeats threaded against 150, Super Mario
64 131 against 149. The deadlock's is the snapshot row's hang above; the fraction's part in the Dam was seen on the code before all the
fixes, and the fraction's mutant alone was not run against the games. The repeat's and the scan race's are not reached by these
rows: the first needs a held line's expiry followed by a repeat, and Super Mario 64 deferred and unthreaded skipped the
same 149 repeats before the fix and after, so these frames hold none; the second needs a multiple, which the rows do
not set. The unit tests of §2.9.4 and `Mars_Video.md` §2.8 hold those two.

*MarsRT against the fixed core.* `MarsRtThreadsTests` was rerun against the fixed C# core: MarsRT threaded against the
C# threaded, at once and deferred, 300 frames of each of the six games, exact; MarsRT's four processors against the C#
core's four, exact in all six, where before the fraction's fix the Dam parted at frame 6; and MarsRT at a multiple
against the C# core at the multiple, 27 cases of 150 frames with and without the device, exact (run before §2.9.5's
rule, which that comparison's unthreaded oracle does not use).

#### 2.9.7 What the fixes cost

*The method.* `pacebench` flat out from the three gameplay states, 900 frames after 120 of warm-up, at one with
production's settings: compiled blocks, the display processor threaded, deferred presentation, repeats skipped, and
four processors, the default on this sixteen-core desktop (one per three cores). Two harnesses were built apart, one
over the code before this branch (f55bfc0) and one over the fixed code, and run in three rounds with the order
reversed each round; each run took the bench lock and began only with the one-minute load below 3, since other
agents were using the machine. The mean `RunFrame` is given, then the emulation thread's waits for the drain.

| ms a frame, four processors | before | after |
| --- | --- | --- |
| Super Mario 64 | 6.44, 5.74, 5.66 (waits 1.31, 1.24, 1.15) | 6.37, 6.19, 6.15 (waits 1.68, 1.55, 1.52) |
| Ocarina of Time | 7.92, 7.29, 7.20 | 7.45, 7.23, 7.38 |
| GoldenEye, the Dam | 16.96, 16.75, 16.86 (waits 0.01, 0.00, 0.01) | 16.95, 17.10, 16.96 (waits 0.11, 0.11, 0.12) |

| ms a frame, one processor | before | after |
| --- | --- | --- |
| Super Mario 64 | 15.50, 14.98, 15.04 | 15.96, 16.47, 15.93 |
| Ocarina of Time | 13.75, 13.68, 14.06 | 13.70, 14.15, 13.69 |
| GoldenEye, the Dam | 23.53, 23.00, 23.26 (waits 7.13, 6.59, 6.99) | 25.29, 25.88, 25.35 (waits 7.81, 9.13, 9.09) |

The final state hashes of the two builds are the same in every run but one: GoldenEye's with four processors, where
the code before this branch ends in a state that differs from its own at one processor, the fraction of §2.9.3, and the
fixed code's with four equals both builds' at one.

*What the fixes cost.* Ocarina of Time's rounds overlap at both counts, and the Dam's at four; those are recorded as
costing nothing measurable. Super Mario 64 at four is 0.3 to 0.5 ms a frame slower in every round but the first, about
7 per cent, and at one 0.5 to 1.5 ms; the Dam at one is 1.8 to 2.9 ms slower, 8 to 12 per cent. That the fixes cost
nothing is therefore **not** true of the range read, and the claim is withdrawn for it.

*Which fix.* A third harness, the fixed code with §2.9.2's line alone put back, ran Super Mario 64 at four at 5.64,
5.78 and 5.85 ms against the fixed code's 6.10, 6.11 and 6.09, with the waits 1.12 to 1.20 ms against 1.47 to 1.61: the
whole cost is the range read's. At one processor the pause, the fraction, the hazard load and the split do not run, and
the repeat rule walks nothing more in these frames (the counts of §2.9.6), so the range read is the only change left
there, by elimination rather than by a separate run.

*Why, and what was tried.* A capture reaches past the lines the walk reads (`Mars_Video.md` §2.7's slack), and in these
games past the buffer on show into the next one, which the list is drawing; the capture now waits for those draws, as
a range read must, where before it copied the bytes as they were being drawn. Whether the waits were for draws that
really hold the bytes, or a page wait wider than the draws, was tested: the range read was narrowed by the boxes over
its whole length, exactly (`Within`'s test as one interval of pixels a box row), and measured against the build with
§2.9.2's line put back. Super Mario 64's waits stayed at 1.52 to 2.08 ms against 1.14 to 1.25, and the frame at 5.98 to
7.49 against 5.59 to 6.22: the waits are for draws that do hold the bytes, the narrowing bought nothing, and it was
reverted. The lever that is left is the capture's reach. `Mars_Native.md` §5.6.9 records that a capture one line
short, three lines short, or without its span slack survives every mutant round in MarsRT as equivalent, while one
ending at the last line or one past it is caught; so the walk needs less than the capture takes, and a capture cut to
the walk's own reach would wait only for bytes the walk reads. It was not built: the reach is §2.7's argument, which
throws on a walk outside the capture, and changing it is a change to the scan-out's design to be made in both cores
together, not a fix of this section.

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

## 11. The picture drawn at a multiple of the console's resolution

*2026-09-20.* `MarsCore.RenderScale` (`Mars_Core.md` §10) asks for the picture at two, three or four times the console's
resolution. The machine is not touched: what games read back, what a state holds and what the probe grades are the
console's drawing, exactly as at one. The multiple is a second drawing, made beside the first from the same words,
into a memory nothing in the machine reads, which the scan-out shows in the first's place (`Mars_Video.md` §2.9).

**The shape.** `DpInterface.Scale` allocates a shadow of RDRAM and its hidden bits at the multiple squared
(`ScaledRdram`, `ScaledHidden`: 32, 72 and 128 megabytes at two, three and four on the eight-megabyte machine a frontend builds, half that on a stock one, and half as much again for the hidden bits), and beside every native processor
a processor at the multiple: one for the direct paths (`_scaledProcessor`, fed by `Take` and the pool's drain after
the native processor takes each word) and one per worker of §2.8 (`Worker.Scaled`, fed by the worker's loop). A
processor at the multiple is an ordinary `Rdp` told `DrawAt(scale, frame, hidden)`: it takes the same words, walks
the same primitives and runs the same pipeline, with every coordinate multiplied and every image address taken to
the multiple's square — the colour and depth images' addresses ×N², their width ×N, the scissor ×N, a rectangle's
edges ×N, a triangle's six y and x values ×N with the slopes unchanged, the attribute steps across, along and down a
pixel ÷N (`DecodeAttributes`), and the texture rectangle's ds/dx and dt/dy likewise. A fill or copy rectangle draws
its right and bottom edge pixel, so those two edges are extended by N−1 whole pixels; without that a fill of the
image at two leaves the multiple's last column and row bare. Textures are loaded by the native processor from the
machine's RDRAM as ever; the processor at the multiple reads and writes only its own frame, through `_frame` and
`_frameHidden` in place of the bus's arrays at every colour and depth access. It is made by `NewScaled`:
constructed, told its frame, given the native processor's state by `CopyStateFrom`, then `Rescale`d — the images
and scissor the state carries taken to the multiple once. It skips the verifier's marks, §2.8's aliased-read record
and the hazard accounting, all of which are the native machine's to decide; it `Follow`s the native processor's
decision to draw alone, and the leader's takes the others' scratch at `Assemble` as the native leader does. A state
read empties the shadow (`ResetScaled`), and the scan-out shows the multiple only once something has drawn there
(`ScaledDrawn`).

**What is proved.** `Drawing_at_a_multiple_leaves_the_machines_memory_as_at_one` runs the fill scene and both
shaded scenes at two and three, with one and three workers, and finds the machine's RDRAM, hidden bits and
serialised state byte-identical to the machine at one: the multiple touches nothing the machine owns.
`A_fill_at_a_multiple_is_the_fill_at_one_at_every_pixel_of_the_multiple` reads every pixel of the multiple's fill
against the console's. `A_shaded_scene_at_a_multiple_averages_back_close_to_the_scene_at_one` draws twenty-four
shaded, coverage-blended triangles with quarter-pixel corners at two, three and four, averages each block of the
multiple back to one pixel and compares the channels: the mean difference is 0.10, 0.12 and 0.13 of a level over
the 76,800 pixels, with 9, 19 and 23 beyond eight levels, every one on a triangle's edge, where the multiple's
coverage resolves a slope the console's could not. The test asks for a mean under a quarter of a level and fewer
than one pixel in two thousand beyond eight.

**The walker at a multiple is not the console's walker.** Three of its widths are the hardware's, and a multiple
exceeds them. (1) Its scratch — the spans, the sub-scanline edges, the row attributes and a row's coverage — is
sized for 1,024 rows and columns; `Widen` sizes a processor at the multiple by N, since a 640-pixel image at four is
2,560 columns. (2) `ClipEdge` keeps ten bits of column and a sticky bit, reads the eleventh bit as past the right
and the twelfth as under the left (the console's wrap of an edge more than 2,048 pixels off the image); the row's
major column is kept as twelve signed bits, the clipped count is masked to twelve, the crossing test is made on
fourteen bits of quarter pixel, and the row limits read a fourteen-bit y with bit 12 meaning past the last row. At
the multiple each is done at full width: an edge is under the scissor when its column is less than the left edge
and over when at or past the right, the crossing test compares the quarter-pixel columns as signed integers, the row
limits are the plain comparisons, and the clipped count is not masked. The wrap at 2,048 columns is therefore not
reproduced at a multiple; a game that relied on it would draw differently there, and none is known to. (3) The
edges are given at the console's row top: `xh` and `xm` are the columns where the two edges cross the top of the
row holding `yh`, and the walk begins there. At the multiple the walk begins at the multiple's row top,
`(N·yh) & ~3`, which lies up to N−1 of the console's sub-scanlines below `N·(yh & ~3)`, where the multiplied
columns are true. Without moving both edges along by that many sub-scanline steps, and the row attributes by as many
rows of their edge step, every triangle whose top is off a row boundary is drawn shifted: the closeness test with
quarter-pixel corners measured that shift at 79 pixels beyond eight levels against 9 with the offset, which is why
its corners are on quarter pixels.

**What the first test measured was the test.** The closeness test's first form reported the drawing at two touching
13,500 more of the console's pixels than the console's drawing, mostly darker, and this was recorded as a rasteriser
defect and pursued for a session. A harness drawing each triangle alone at both resolutions found every triangle
grown, and its dump found two defects in the scene. The corners were computed in unsigned arithmetic (`uint % int`
with a constant divisor is unsigned), so "twenty pixels off the left" was a column of four billion, which the
saturating conversion turned into a degenerate edge at column zero with the steepest slope — which the console's
twelve-bit column wrap then shaded from wrapped counts, differently at each multiple. And the "gentle" shade put its
random values into the integer halves of the green and alpha steps, so the shade wrapped every few pixels and the
alpha reached zero, where a blend over the clear colour reproduces the clear colour and a drawn pixel counts as
untouched. With signed corners and steps within a quarter of a level a pixel, the well-formed triangles agreed to
their edges before any change to the walker; only the width limits and the start offset above were then real. The
lesson is §0's: a scene that is meant to be gentle has to be checked to be, and the first thing to do with a
surprising count is to draw one primitive and look.

**The scratch is in the state.** §10.2's third item noted the coverage buffer is serialised; so are the walker's
per-row arrays, some 70 kilobytes of every state that no command reads across a word, since a primitive is walked
and drawn in one `Execute`. `CopyStateFrom` into a processor at a multiple therefore reads them at the console's
width and widens again after. Dropping them from the format would shrink every state and the rewind's deltas; it is
not done here because the states in use are format 1 and the gain is not this section's.

**What the multiple is not.** It is an approximation of the console's picture, not another exact one: the level of
detail and the texel a pixel samples are chosen at the multiple's pixel positions with steps ÷N, so texture
filtering resolves finer where the console's would blur, which is the behaviour a high-resolution plugin has and the
reason to want the multiple. A frame the CPU writes rather than draws — a copy the processor makes, an effect it
paints, a buffer the game fills without the display processor — is not in the shadow, which holds only what the
display processor drew, so once anything has drawn at the multiple such a buffer scans out from the shadow as black
or stale. The shadow costs its memory at the multiple squared, and the processors at the multiple cost their
shading, N² pixels for every one of the console's, on the same threads as §2.8's; what that comes to in play is
§11.1.

**A defect found the same day: after a state was read the multiple never showed again.** `ResetScaled` replaces
every processor at the multiple, but a worker's loop held the one it started with in a local, so the threads went
on drawing with processors nothing looked at while `ScaledDrawn` asked the new ones, which never drew. The loop now
takes its processor again whenever it has stood or slept, which is the only time a state can have been read.
`After_a_state_is_read_the_multiple_is_drawn_and_shown_again` fails without that and passes with it. It was found by
a run from a state that reported the console's width at a multiple of two, not by a test, because every test of the
multiple until then had drawn without reading a state.

### 11.1 What the multiple costs in play

*2026-09-20.* `playbench` took a `RENDERSCALE=n` switch, and `ab-modes2.sh` ran one build in three modes from the
four in-game states, three rounds of 600 frames, order rotated, four workers, the display processor on its thread.
Medians of the second half, frames a second:

| State | 1× | 2× | 4× |
|---|---|---|---|
| Ocarina of Time, the field | 53.6 | 35.5 | 17.8 |
| Wave Race, racing | 71.5 | 45.9 | 18.6 |
| Super Mario 64, the castle | 72.8 | 55.6 | 27.0 |
| Super Mario 64, outside | 77.8 | 41.4 | 15.7 |

The multiple is drawn beside the console's, not instead of it, so the display processor's threads shade 1 + N²
pictures: five at two and seventeen at four. Two costs a third to a half of the frame rate and stays above the
console's sixty in the castle and near it elsewhere, which is the setting's hint; four is a quarter of one's and not
playable on this machine at four workers. The states are bound by the display processor's thread at four workers
before the multiple is added (§35 of `Mars_Performance.md`), so more workers would carry the multiple further, and
a run at eight was not made. The 1× column is below §36's figures for the same states because the rounds ran on a
loaded machine; only the interleaved difference is the measurement, as the harness's README says.


### 11.2 Fixed: a copy at a multiple stepped a group's texels at every pixel

*2026-09-21.* At an internal resolution above one, the HUD of Super Mario 64 and the status bar of Kirby 64 were
drawn wrong, on the processor's threads and on the device alike. In Mario's HUD each glyph — the head, the cross, the
digits, the star, the camera — appeared as four copies side by side, each a quarter of the glyph's width, at two, three
and four; Kirby's status bar was squeezed into the left quarter of its width, and the remaining three quarters showed
the texture memory beyond the image the game had loaded. From the user's two states (made in Mistress with
`RenderScale` 2, the device on), each run ten frames and the multiple sampled at the centre of every console pixel of
the HUD, the mean largest-channel difference from the picture at one was 35.9 (Mario, 21,150 pixels) and 96.1 (Kirby,
65,400 pixels) at two; after the fix it is 8.1 and 6.2, what is left being the scene behind and around the HUD, which
the multiple resolves more finely. Device and processor pictures were identical before the fix and after it, at every
multiple.

**It was not the recorded limit of §11.** A frame the CPU writes scans out stale at a multiple, and a HUD would be a
plausible such frame; but both HUDs are drawn by the display processor, as copy-mode texture rectangles, and the
multiple held them — at the wrong texels.

**The cause.** The copy mode advances its texture coordinates once per group of eight bytes, which is four pixels of
a sixteen-bit image and eight of a narrower one (`Mars_RdpCopy.md` §1), so content gives a copy rectangle a step of
four texels, not one. `DrawCopyScaled` (`Rdp.Copy.cs`) walks the multiple one pixel at a time, each pixel taking the
texel its own coordinate names, and it stepped that coordinate by `_textureStep` — the group's step, divided by the
multiple — at every pixel. At two, each pixel of the multiple therefore moved two texels where it should move half of
one: four times too fast, whatever the multiple, which is exactly the fourfold repetition in Mario's glyphs and the
fourfold squeeze in Kirby's bar. `RecordShaded` (`Rdp.Gpu.cs`) handed the device the same steps. The fix is
`CopyPixelStep`: at a multiple, the copy's per-pixel step is the group's step shared among the group's pixels,
`_textureStep / 4` for a sixteen-bit image and `/ 8` for a narrower one, used by both `DrawCopyScaled` and the
device's record. The machine's own copy (`DrawCopy`) is untouched, and so is everything at one.

**Why the device's tests did not see it.** `Copy_mode_rectangles_on_the_device_are_the_cpus_byte_for_byte` holds the
device to the CPU path at the multiple, and the two were wrong the same way. No test held the multiple's copy to the
console's. `A_copy_at_a_multiple_is_the_copy_at_one_at_every_pixel_of_the_multiple` (`MarsThreadedRdpTests`) does:
three rectangles of a 32×32 sixteen-bit texture with a step of four texels — from the tile's corner, from an odd
column and an odd texel, and a third elsewhere — and every pixel of the multiple compared with the console's pixel
whose square holds it. Before the fix it failed at every multiple ("11456 pixels of the multiple differ; the first:
pixel 81,40 of the multiple is 0005 where the console's 40,20 is 0001", at two: two texels along after one pixel);
after it, none differ at two and four.

**Mutants.** The per-pixel step left undivided in `DrawCopyScaled`: the new test fails at 2, 3 and 4, and so does the
device test, the device now differing from the processor. Left undivided in the device's record only: the device test
fails at 2, 3 and 4. A group of two pixels for a sixteen-bit image: the new test fails at 2, 3 and 4 (the device test
does not, since both paths share `CopyPixelStep`). A group of four for every image size: **survives**; no test draws a
copy into an eight- or four-bit image at a multiple.

**What the fix does not cover.**

- **Three is not exact.** A step divided by three is truncated, so a texel the console reaches exactly — every one,
  with the usual step — begins one pixel of the multiple late, in both directions. The test accepts at three a pixel
  equal to the console's pixel to its left, above, or above-left, and nothing else. The one-cycle texture rectangle
  divides its steps by the multiple the same way and is presumed to have the same one-pixel lag at three; that was not
  measured. Stepping in the console's units with a remainder would make three exact and is not done here.
- **A flipped copy rectangle differs from the console's.** The copy mode fetches four consecutive columns of s for a
  group whichever way the rectangle is flipped, while the multiple samples each pixel on its own and so walks t along
  the row. With the third rectangle flipped, 3,072 of the 4,096 pixels of the multiple it covers at two differ from the
  console's: all but the first pixel of every group. Reproducing the group's fetch at the multiple would fix it, and it
  would also make a copy whose step is not four texels match the console's rather than sample each pixel; neither case
  has been seen in a game.
- **Eight- and four-bit images** take `/ 8` on reading §1 of `Mars_RdpCopy.md`, not on a measurement; the surviving
  mutant above is the record of that. The byte order of a right-to-left copy (§5.2 of that page) is still written as
  whole pixels at the multiple.
