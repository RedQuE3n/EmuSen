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
