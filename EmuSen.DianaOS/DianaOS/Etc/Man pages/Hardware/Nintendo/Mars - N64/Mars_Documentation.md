# Mars — what the Nintendo 64 is documented to do, and what it is not

*Written 2026-09-15 from a literature survey. `Mars_Gameplan.md` §8 said that its
hardware claims came from general knowledge rather than citation and should be
checked against primary documentation as each phase opens. This is that
documentation, assessed rather than listed: which sources are authoritative, where
they contradict each other, where they are wrong, and — the section that matters
most — what an implementer needs that **no document anywhere answers**.*

*Provenance is marked per claim: **[primary]** a vendor, patent or archival source
read directly; **[community]** a wiki or emulator author's notes; **[reported]** the
survey's finding, not independently re-read here. The survey marked its own
confidence and this page preserves it.*

---

## 1. The tiers, in order of what a citation is worth

**Vendor manuals — authoritative, and legally awkward.** The NEC *VR4300 User's
Manual* (655 pages) is close to complete for the CPU, and the SGI *RSP Programmer's
Guide* (331 pages) and *RDP Command Summary* (48 pages) are far better than the
field's folklore suggests. The Nintendo *Programming Manual* (534 pages, every page
stamped DRAFT) carries the conceptual model for coverage, blending and the video
filters. **All four circulate without any licence grant.** They are read as
documentation; nothing is reproduced from them, and §6 says what gets cited instead
where a choice exists. **[reported, with extensive direct quotation in the survey]**

**Patents and the designer's own testimony — the citable record.** This is the
survey's most useful structural finding for a project that wants an academic
register. The N64's antialiasing filter, its dedither filter and its VI block chain
are described in granted patents by the engineers who designed them, and Phil
Gossett — who designed the RDP — described the same mechanisms independently in a
Computer History Museum oral history. Patents are dated, attributed, permanently
archived and freely citable; the relevant ones here have long expired. Where a wiki
and a patent say the same thing, **cite the patent**. **[primary]**

**n64brew wiki — the best working reference, CC BY-SA 4.0.** Community reverse
engineering, and for the RDP command set it is now better than SGI's own summary
because it supplies the semantics SGI omitted. It marks its own uncertainty with
TODO and TOVERIFY markers, which is why it can be trusted about what it does not
know. **[community]**

**The test corpus — the executable specification.** Where prose and hardware
disagree, `nemu64-test` is the arbiter (`Mars_TestOracle.md`). For the RSP's clamping
rules in particular, its per-opcode tests are more trustworthy than any written
source, because no written source states those rules completely (§3.2).

## 2. The CPU is a solved problem

The NEC manual covers the instruction set, the pipeline, the TLB, exceptions,
COP0 hazards, the caches, and the FPU, with per-instruction detail pages for the
awkward cases — `LWL`/`LWR`/`LDL`/`LDR` included — and explicit
`NullifyCurrentInstruction` semantics for branch-likely. Integer multiply and divide
stall counts are tabulated (`MULT` 5, `DIV` 37, `DMULT` 8, `DDIV` 69 PCycles).
**[reported, quoting tables read directly]**

**One question the plan asked is now answered exactly.** `Count` increments at half
PClock — 46.875 MHz against a 93.75 MHz CPU clock — stated in the manual as a
constant rate independent of instruction execution. `Mars_Gameplan.md` §5 listed
`Count`'s rate among the hazards; for the rate itself the hazard is closed, and what
remains is the bookkeeping discipline `Mars_References.md` §5.3 argued for.

### 2.1 Three CPU traps that will not announce themselves

> **Settled, 2026-09-16.** All three of the traps below were checked against the
> corpus when Phase B's arithmetic landed. The first is **confirmed** and its
> provenance upgraded from [community] to [read]; the second is **confirmed**; the
> third, which this page called unresolved and said should be settled by test, **is
> now settled by test** — the directed rounding modes are taken literally on underflow
> and a tiny value can round to the minimum normal. `Mars_FpuMath.md` §1, §7 and §4.1.
>
> A fourth trap, which no source here predicted, turned out to matter more than any of
> them: **the part refuses to compute with denormal operands at all.**

- **NaN encoding is reversed from modern IEEE.** The VR4300 follows the pre-2008
  convention, so the quiet/signalling bit patterns are the opposite of what a
  present-day host FPU produces. **[community]** An implementation that leans on C#'s
  `double` inherits the wrong convention silently — which is a second, independent
  argument for Phase B starting soft (`Mars_References.md` §5.2).
- **The unimplemented-operation exception has two documented scopes that do not
  match.** The NEC main text excludes compare instructions from the denormal/QNaN
  rule; a later-added appendix adds integer-conversion overflow and infinite/NaN
  sources for the `CEIL`/`FLOOR`/`ROUND`/`TRUNC`/`CVT` family. Both must be read;
  reading one is how you implement half of it. **[reported]**
- **Rounding on underflow is not the host's.** `FLOOR`/`CEIL` are reported to be
  taken literally even on underflow, so a value below the smallest normal may round
  to the minimum normal rather than to zero. The vendor table on denormal flushing is
  consistent with this but is stated only for one control-bit setting, so the general
  claim is **unresolved and should be settled by test** rather than by reading.

## 3. The RSP is documented in structure and not in semantics

The SGI guide gives the 48-bit-per-lane accumulator, which accumulator bits each
instruction extracts, the round values per format, the control registers, the
element and broadcast encodings, and full per-instruction pseudocode. **[reported]**

### 3.1 The official pseudocode contains a bug

The reciprocal instruction's leading-bit search, as printed, assigns zero where it
must assign the loop index — so an implementation transcribed faithfully from the
vendor document produces wrong reciprocals for every non-trivial input. The survey
rendered the page as an image to rule out a text-extraction artefact; the error is in
the document. **[reported, verified against the page image]**

This is the single best argument in the whole survey for the project's standing rule
that a measurement beats a reading: the most authoritative available source for this
instruction is wrong, and only a test catches it.

### 3.2 What the vendor never defines, and what fills the hole

`Clamp_Signed` is used throughout the pseudocode and **defined nowhere** beyond "x is
clamped to prevent overflow". The unsigned variant is never defined at all. The
asymmetric unsigned rule everyone implements — saturating threshold at 15 bits but
saturated value at 16 — appears only in community sources, and the two that state it
use near-identical wording, so they are **probably not independent confirmations**.
**[reported]**

The reciprocal ROM tables are published in full in a community document with **no
derivation, no formula and no account of how they were obtained**. `Mars_References.md`
§2 already commits to deriving them rather than lifting them; that commitment now has
a second justification, which is that nobody can say where the published numbers came
from.

**The corpus is the real authority here**: per-opcode tests exist for the whole
multiply-accumulate family, the rounding instructions, the select family, the
reciprocal family and the load/store forms.

## 4. The RDP, and a precise answer to the reopening condition

`Mars_Gameplan.md` §2.1 committed to LLE with one condition that would reopen it: *if
the RDP's coverage and anti-aliasing rules turn out not to be recoverable to
pixel-exactness from the available references, an LLE RDP cannot be graded.* The
survey splits that into three questions with three different answers.

**Tier 1 — what coverage means, and how it is stored and consumed: documented.**
The 4×4 subpixel mask, the checkerboard dither mask that halves it to a 0–8 range
with worked examples, `memcvg = coverage − 1` with automatic restoration on read, the
hidden-bit layout per colour depth, the Z and dZ memory format, coverage wrap for
transparency, and the pixel-rejection rules. Implementable from documents.
**[primary/community, high confidence]**

**Tier 2 — how the subpixel mask is *generated*: documented nowhere.** No source
states the rule that turns a triangle's edge coefficients into which subpixels are
covered on each sub-scanline. The vendor summary encodes the coefficients but not the
scan conversion; the programming manual says only that a mask "is computed"; the wiki's
pipeline page says only "computes a coverage value" and carries an explicit TODO; two
independent emulator authors list it among their unresolved problems. **Edge pixels
are exactly where anti-aliasing lives, so this gap is the whole question.**
**[reported, high confidence that no document covers it]**

**Tier 3 — the video interface's filters: documented at algorithm level.** This is
the survey's pleasant surprise. The antialiasing filter's neighbourhood — seven
pixels in a checkerboarded hexagonal arrangement, partially-covered neighbours
rejected, the *penultimate* maximum and minimum taken rather than the extremes, and
the background recovered by subtracting the foreground from their sum — is in a
granted patent, restated independently in the designer's oral history, and
corroborated by a second patent's block diagram. The dedither filter has its own
patent with its increment/decrement rule. What is absent is bit-level detail: widths,
rounding, clamping, evaluation order, and the gamma transfer function beyond "square
root". **[primary]**

### 4.1 The verdict: the condition does not fire, and the sentence it was written in was wrong

Pixel-exactness is **not** reachable from documentation alone, because of Tier 2. It
**is** reachable against a reference implementation, which is what the field actually
does — and `Mars_TestOracle.md` §5 establishes that the infrastructure for grading
that way exists and is good.

So the plan's condition, read literally, asked the wrong question. It conflated *"can
be derived from documents"* with *"can be graded"*, and only the second is load-bearing.
The decision stands, on the second reading, with the cost stated plainly:

> **Mars's RDP will be developed against a reference implementation, not against
> documentation, for the rasterization tier specifically.** Tiers 1 and 3 are written
> from documents and patents. Tier 2 is established empirically — by generating
> primitives, comparing output, and inferring the rule — which is a different kind of
> work with a different kind of confidence, and the man page for the RDP will have to
> say which of its rules came from which.

That distinction is the thing to carry forward. It is not a licence to guess; it is a
requirement to label.

## 5. The gaps that must be established rather than read

Beyond the subpixel mask, the survey's gap list clusters in one place: **system
timing is undocumented, and the community says so in writing** — the wiki's own task
list names CPU pipeline cycles, every interface's DMA and I/O timings, and DMA
priority arbitration as needed and unwritten. **[community]** Specifically absent:

- RDRAM latency and bandwidth under real access patterns.
- DMA arbitration and priority between the CPU, SP, DP, VI, AI and PI.
- PI DMA duration as a function of length, alignment and domain settings.
- Serial interface transaction duration.
- Cache miss cost in system cycles, as opposed to pipeline-relative interlocks.
- RDP per-command cost, and what the clock counter counts under load.

**What this means for the plan.** `Mars_Gameplan.md` §2.2 deferred performance to
Phase G; this is a different and more serious deferral, because a game that
busy-waits on a DMA cares about durations that no document supplies. Two of the three
per-game timing knobs `Mars_References.md` §5.1 found in other emulators' databases
live precisely here — which reframes those knobs. They are not only workarounds for
timing that was never modelled; they are workarounds for timing **nobody has written
down**. That does not excuse them, but it does say where the difficulty is real.

Smaller documented-nowhere items: the RDP's span registers, chroma key, TMEM edge
cases, line primitives from degenerate edges, the exact divot median, the gamma
function's precision, the IPL3 checksum algorithm, and device behaviour after a
joybus reset.

## 6. Errata, contradictions, and the citation policy that follows

Recorded because each one costs a day if met without warning:

1. **The vendor reciprocal pseudocode is wrong** (§3.1).
2. **One RDP sync command's opcode is stated wrongly in every document** and the
   corrected value is known from community disassembly. **[reported]**
3. **`.n64` byte order is disputed between sources.** Detect the container from the
   magic word, never from the extension — a rule the survey found stated by the more
   careful source and contradicted by several others, and which Phase 0's ROM handling
   should implement directly.
4. **The wiki's dither matrices are uncited** and one is not the textbook ordering
   its name implies. Plausible, unverified, and worth a test.
5. **Hidden-bit semantics differ per colour depth** — ignored for 32-bit framebuffers
   per the programming manual, yet compared as a first-class output by the modern
   reference's conformance suite. Model "unused" and "undefined" as different things.
6. **Two long-cited community sources are effectively dead** — one wiki is returning
   permission errors to everyone and one file host has reorganised away a
   frequently-linked page. Cite the archive, not the live URL, for anything from them.

**The citation policy this argues for**, and which the RDP and VI pages should follow
when they are written: prefer the patent and the oral history over the wiki where both
cover a mechanism, prefer the corpus over all prose where behaviour is contested, and
state in the page which tier a rule came from. A rule inferred from a reference
implementation is labelled as such.

## 7. A licensing correction that changes which reference we grade against

`Mars_References.md` §2 set the rule for the four checked-out emulators. The survey
adds a fact that page did not have: **angrylion — the pixel reference for the RDP —
is under the MAME licence**, which is non-commercial and which states in terms that
the proper use of the source is to read it, understand the hardware, and then write
your own implementation. GitHub does not display a licence for it, so projects
routinely treat it as permissive. It is not.

The practical consequence is a preference rather than a prohibition: **parallel-rdp
is MIT and is itself validated bit-exact against angrylion**, so grading Mars against
parallel-rdp reaches the same standard through a permissively licensed instrument.
Where angrylion is read, it is read under exactly the terms §2 already imposes on
everything else — which is, as it happens, precisely what the MAME licence asks for.

## 8. What this does not settle

- Everything marked **[reported]** is the survey's reading, not a second reading by
  this project. The load-bearing items — the reciprocal typo, the NaN convention, the
  appendix's extra exception cases — are cheap to confirm once Phase A runs, and
  §2.1's discipline applies: confirm, do not trust.
- **No peer-reviewed literature on this hardware exists.** The citable record is
  vendor manuals, the patent family, one oral history and one conference talk. A
  project that wants an academic register on the N64 is building on primary sources
  and its own measurements, with no secondary literature to stand on.
- The survey found an N64 emulator written in C#. Whether it is worth reading as
  prior art — for what the language costs on this hardware, not for its code — is
  unexamined and is a separate question from anything here.
