# Mars — the construction plan for the Nintendo 64 core

*Pinned 2026-09-15. **Nothing is built.** This is the plan written before the first
line of code, which makes it a different document from `Mercury_Gameplan.md` — that
one was pinned mid-construction and reached its "no phases left" state by being
edited as each phase landed. This one starts at the other end and is expected to be
wrong in places; §8 says where it is most likely to be wrong, and every phase names
what would prove it finished rather than asserting a date.*

*Two decisions were taken before writing it, in answer to direct questions, and §2
records both with the reasoning and the condition that would reopen them.*

---

## 1. What exists today

The name and two stub folders. `EmuSen/Cores/Nintendo/Mars - N64/README.md` says
"reserved for the future N64 core", this folder's `README.md` says the same, and
`EmuSen_Core_Naming_Scheme.md`'s table lists Mars as "Not started — reserved".

`CoreCatalog` does not mention Mars, no extension reaches it, and
`CoreFactory.IsSupported` refuses `.z64` today. That is the correct state: a
descriptor in the catalogue is a promise to the ROM browser, the file picker and
the cheat database, and Mercury's own plan records that its registration waited on
a real condition rather than being done early for tidiness (`Mercury_Gameplan.md`
§2). Mars registers in Phase F, and §4.6 says why that is late rather than first.

## 2. The two decisions taken before any code

### 2.1 The whole RCP is LLE

The RSP runs the game's real microcode, and the RDP consumes the real command
stream it emits. No microcode is recognised by signature, no display list is
reinterpreted as calls to a modern rasterizer, and no audio list is turned into a
mixer written in C#.

The argument is not that HLE is wrong — it is what mupen64plus and Project64 do,
it is why they were playable a decade before the accurate emulators, and for a
frontend whose goal is a playable library it remains the right engineering. It is
that **HLE forecloses the oracle**, and this project is organised around having one.
A recognised microcode is a per-game contract: an unrecognised one is a gap, a
subtly different one is a bug that reproduces in exactly one title, and the
question "is this frame correct?" degrades into a human judging a screenshot.
Under LLE the same question is a diff against a pixel-exact reference, and the
existing test corpus (§3) is written against silicon rather than against a
convention.

This is the same reasoning `Mercury_Gameplan.md` §3.2 used to reject "diff against
gambatte" — *agreement with another emulator is agreement, not correctness* — one
level further down: agreement with a microcode's intent is not agreement with the
RSP that ran it.

**What it costs**, stated plainly rather than discovered later: the RSP and the RDP
are each a subsystem on the scale of the entire Mercury core, and the RDP in
particular has to be pixel-exact to be worth the choice at all. An LLE N64 that is
approximately right is strictly worse than a good HLE one, because it is slower
*and* wrong.

**What would reopen this:** nothing about speed — that is §2.2's question, and
Phase G's. The condition is evidential: if the RDP's coverage and anti-aliasing
rules turn out not to be recoverable to pixel-exactness from the available
references, then an LLE RDP cannot be graded, and an ungradeable LLE RDP has lost
its only advantage over an HLE one.

> **Tested 2026-09-15 against a documentation survey. The condition does not fire,
> and the sentence it is written in was badly posed** — it conflated *derivable from
> documents* with *gradeable*, and only the second decides anything. Coverage's
> meaning, storage and consumption are documented; the video filters are documented
> at algorithm level in the designers' own patents and corroborated by an oral
> history; but **how the rasterizer generates the subpixel mask is documented
> nowhere at all**, and that is where anti-aliasing lives. Grading against a
> reference implementation does reach pixel-exactness, so LLE stands — at the price
> that one tier of the RDP is established empirically rather than read, and must be
> labelled as such wherever it is written down. `Mars_Documentation.md` §4.

**Supporting evidence arrived after this was decided**, and is recorded because it
was not what the decision rested on: the two mature non-LLE cores each carry a
per-title database of 1,587 and 3,292 entries, including per-game multipliers on
how fast the CPU's counter advances. An emulator that models the machine does not
need to be told about the game. `Mars_References.md` §5.1 has the counts, and the
standing test is that a per-game timing knob appearing in Mars means a piece of the
machine was never modelled.

### 2.2 Correctness first; speed is Phase G

Mars will not run at full speed on the low-end laptop that
`EmuSen_Core_Gameplan.md`'s optimization effort targets, and will not for a long
time. The decision is to build it headless and measured, accept that early Mars is
a fraction of real time and not playable, and give performance its own phase once
there is a correct thing to make fast.

Two reasons, neither of them "optimization is premature":

- **Every fast choice for an N64 costs a measurement.** Recompilation, cached
  interpretation, a GPU rasterizer and microcode HLE all make some class of
  divergence unobservable — which is acceptable against a core already proven
  correct and not against one being proven.
- **The existing optimization effort has a target and Mars is not it.** That effort
  is about `mainComposite` and a specific SNES decode path. Adding a core that
  cannot hit the frame budget does not slow that work down; it only invalidates any
  claim that "EmuSen runs well on weak hardware" if Mars is quoted as part of it,
  which §7 makes explicit.

**What would reopen this:** a measurement, not a feeling. Phase G opens with a
profile, and the first question it asks is whether the interpreter or the RDP is
the wall — the answer determines whether a recompiler is even the relevant lever.

## 3. The oracle, settled before Phase A rather than after

`Mercury_Gameplan.md` §3.2 is the precedent: the question *diff against what?* was
answered for the Game Boy by its own hardware test corpus reporting through the
serial port, which made the work reference-agnostic — the oracle names no emulator.
The N64 has a richer answer and a harder one.

**Three candidate oracles, in the order of how much they are worth:**

1. **A hardware test-ROM corpus.** The N64 has one of the best in emulation —
   `n64-systemtest` — covering integer edge cases, COP0, the TLB, COP1, exceptions
   in delay slots, and the RSP's vector unit, written against real silicon and
   reporting per-assertion failures rather than a screenshot. This is the direct
   analogue of blargg and mooneye, and it is what Phases A through C are graded
   against.
2. **A pixel-exact software RDP.** For Phase D the corpus stops being enough,
   because the RDP's output is a framebuffer rather than a verdict. The reference
   the emulation community treats as ground truth is angrylion's software
   rasterizer. A per-primitive pixel diff against it is what "the RDP is correct"
   has to mean. **The mechanism is now specific**: angrylion exposes a
   command-stream dump — commands, main and hidden DRAM, VI registers, frame
   boundaries — so the differential is the same stream into two rasterizers rather
   than two emulators run side by side. `Mars_References.md` §4.
3. **A full-system reference through the existing probe.** `Reference/probe-rs/`
   already has a libretro backend that drives any core, and its
   `system_from_rom` is a small extension table — adding the N64's three extensions
   and a memory-space name is the whole change on our side. This is the weakest of
   the three and stays third: it grades a running machine against another
   emulator's running machine, which §2.1's own argument says is agreement rather
   than correctness. It is still worth having for bisecting a divergence to a frame.

> **Resolved 2026-09-15, and the answer is yes.** The paragraph below was the
> plan's largest unknown; it has been checked against the corpus's own source and
> `Mars_TestOracle.md` replaces it. The text channel exists, the phase order stands,
> and the cost of admission turned out to be four specific behaviours Mars must have
> before the ROM will say anything — two of which fail *silently* if got wrong
> (`Mars_TestOracle.md` §2.2, §2.3). The corpus also moved repository, and it grades
> Phases A–C only: D and E get nothing from it, and §5 there records that the RDP has
> no hardware-grounded oracle anywhere.

**The prerequisite this rested on, and it was unverified.** The Game Boy corpus was
readable headlessly because the hardware has a serial port and the ROMs write their
verdict to it. The N64 has no equivalent *by default*: its test ROMs render results
to the framebuffer, and the channel they also log through is the ISViewer debug
port that development hardware exposed in the cartridge address space. Whether
`n64-systemtest` in particular emits its per-assertion output there, in what format,
and whether it needs anything else present to do so, **is the first task in Phase 0
and is not assumed by this plan.** If it turns out not to, the fallback is reading
the framebuffer — which is a far worse oracle, because it requires the VI and a
font rasterization to be right before the CPU can be graded, and that inverts the
dependency order the phases are built on.

That is the single most load-bearing unknown in this document. Everything in §4 is
ordered as if a text verdict channel exists.

## 4. The phases

Ordered by dependency, and each one states what would prove it done. Phases A–D are
sequential because each is the next one's prerequisite; E is partly parallel; F and
G are terminal.

### 4.0 Phase 0 — the harness before the hardware

No emulation at all. What this phase produces is the ability to grade the next one.

- ~~Verify the §3 output-channel claim~~ — **done 2026-09-15**, before any other
  Phase 0 work, and the answer is in `Mars_TestOracle.md`. What it leaves for this
  phase is the ROM's discovery path and the four admission requirements in its §3,
  which belong to Phase A's memory map rather than here.
- ~~ROM image handling~~ — **done 2026-09-15.** The three containers, decided by the
  magic word rather than the extension, normalised to big-endian, with the header
  read and truncated images refused rather than half-converted. `Mars_Rom.md`.
- ~~A `SyntheticN64Rom` fixture~~ — **done**, alongside `N64TestRomLibrary` for the
  corpus discovery path, which skips when the corpus is absent. The repo rule that no
  commercial image is ever committed applies unchanged; the corpus ROM is MIT
  homebrew built from source and lives in the gitignored `TestRoms/n64`.

**Done when** a WiseMan test can load a synthetic image and assert its header, with
the CPU still entirely unimplemented. **That is now true**: 24 tests, including one
against the real corpus ROM, verified by mutation rather than by a green run.

**What is left in this phase: nothing.** The remaining Phase 0 item as originally
written — running a corpus ROM to a captured verdict — belongs to Phase A, because
capturing a verdict requires executing instructions. What Phase 0 owed that work was
the oracle's protocol and its four admission requirements, and `Mars_TestOracle.md`
carries both.

### 4.1 Phase A — the VR4300 integer core, the memory map, and boot

The 64-bit MIPS III integer instruction set, the delay-slot rules, the exception
model, COP0 (`Count`/`Compare`, `Status`, `Cause`, `EPC`), and the 32-entry TLB.
Alongside it the physical memory map: RDRAM, the RSP's DMEM and IMEM as plain
memory, the MMIO register blocks stubbed to sane reads, and PI DMA from the
cartridge.

**Started 2026-09-15 with the memory map** — segments, the physical map, the debug
port and the machine clock, with `Count` derived rather than incremented so that no
opcode can forget it (`Mars_Memory.md` §3.1) — and then the two transfer engines the
corpus's bootstrap drives before any instruction of ours runs (§6, §7). **All three
of the bootstrap's admission requirements now hold and are tested.** The integer
instruction set landed next (`Mars_Cpu.md`), with the fourth admission requirement —
the emulator-extension opcodes the corpus calls unconditionally — ignored rather than
refused. The merging loads and stores followed
(`Mars_Cpu.md` §7). Exception vectoring and interrupts followed
(`Mars_Cpu.md` §9, §10). The TLB followed (`Mars_Tlb.md`), and then the boot
handoff (`Mars_Boot.md`).

**Phase A is done, in the sense its own paragraph asked for.** The corpus boots
through its own bootcode and reaches its entry point, and stops there on a
coprocessor-1 control move — which is Phase B's opening rather than a defect in this
one. What A cannot yet claim is the rest of its "done when": the corpus's CPU, COP0,
TLB and exception sections have not been *run*, because reaching them needs the
floating-point unit the corpus configures in its first instruction.

**Four behaviours are the cost of admission to being graded at all**, and belong to
this phase's memory map: a *readable* ISViewer region, COP0 CO `funct` `0x20`–`0x3F`
as no-ops, and three RDRAM/SP DMA edge cases the corpus's bootstrap depends on. Two
of them fail silently. `Mars_TestOracle.md` §3.

**Boot is HLE, deliberately.** The real sequence runs the PIF ROM, which performs a
CIC challenge-response with the cartridge's security chip before IPL3 — copied from
the cartridge's first bytes into DMEM — copies the game into RDRAM and jumps to the
header's entry point. Mars instead starts with the register state IPL3 leaves
behind and performs that copy itself. The reasons are that the PIF ROM is
copyrighted and undumped on most users' machines, that the CIC handshake proves
nothing about the emulated machine's correctness, and that `EmuSen_Firmware.md`'s
contract (§1) already says a missing image must never be fatal — so a real PIF
boot, if it is ever wanted, is a `FirmwareRequest` and an alternative path rather
than a rewrite.

**What HLE boot does not cover**, recorded now so it is not rediscovered as a bug: a
game that inspects the boot state beyond what IPL3 leaves, anything that depends on
CIC-seed-derived values, and the handful of titles whose protection notices an
emulated boot. Each of those is a condition that argues for the firmware path, not
a defect in this one.

**Done when** the corpus's CPU, COP0, TLB and exception sections pass, each failure
being a numbered assertion rather than a suspicion.

> **Met, 2026-09-16** — `Mars_Cop0.md` §1. Every CPU, COP0, TLB, exception, LL/SC and
> address-error test the corpus runs passes; what still fails is caches, which are
> deliberately unmodelled, and cartridge DMA, which is Phase E's.
>
> **It was met by a route this plan did not anticipate, and the order is the lesson.**
> Phase A could not be finished as Phase A. The condition above is stated in terms of
> the corpus's verdict, and the corpus could not deliver a verdict until enough of
> *Phase B* existed for its first instruction to run (`Mars_Fpu.md` §9). Three slices
> of Phase B and Phase A work then cleared 202 → 138 failures, almost all of them in
> Phase A code that had been written, reviewed and unit-tested months of work earlier
> and was wrong in ways no test written here would have suspected.
>
> The general form: **a phase whose completion is defined by an instrument cannot be
> completed before the instrument runs.** §4's ordering treated the phases as
> independently finishable and they are not. Nothing about the phase *contents* is
> retired by this — the boundaries held — but "done when the corpus says so" should be
> read as a claim about evidence, not about sequence.

### 4.2 Phase B — COP1, the floating-point unit

MIPS III floating point in both precisions, all four rounding modes, the FCSR flag
and enable bits, and the VR4300's specific answers for the operations the
architecture leaves to the implementation — denormal handling and the
unimplemented-operation exception among them.

**Started as a software implementation with explicit rounding, not as C# `double`
with care taken.** Project64 began with host floating point and had to convert its
interpreter to a soft-float library afterwards, having chased the divergences in
between — `Mars_References.md` §5.2. A second, independent reason arrived with the
documentation survey: **the VR4300's quiet/signalling NaN bit patterns are the
reverse of the modern IEEE convention**, so a host FPU gets NaN handling backwards
silently. The exception scope is also split across two parts of the vendor manual
that do not agree, and the rounding of underflow is disputed between sources —
`Mars_Documentation.md` §2.1. All three are settled by the corpus, not by reading.

Separate from A because it is separately gradeable and because its bugs are silent:
an FPU that is wrong in the last bit produces a game that looks right and drifts,
which is precisely the failure mode a test corpus exists to catch early.

**Done when** the corpus's COP1 sections pass, including the exception cases.

> **Substantially met, 2026-09-16** — `Mars_FpuMath.md` §9. Every COP1 test the corpus
> run reaches passes. It is not *fully* met: the run is truncated inside the 64-bit
> conversion tests, where an edge case still storms (§9.1), so the COP1 sections have
> not all been *reached*, let alone passed.
>
> **The phase's founding decision was right, and for a stronger reason than was
> argued.** The case for starting soft rested on Project64's late conversion and on a
> reported NaN quirk. The quirk is real and measured (§1) — but the decisive finding
> was not anticipated at all: **this part refuses to compute with denormals**, which
> removes gradual underflow from the problem entirely and makes a software float
> *smaller* than the careful host-float wrapper the plan was arguing against.

> **Progress, 2026-09-16.** The first slice landed — the register files and every path
> into them, with no arithmetic (`Mars_Fpu.md`). The consequence was larger than the
> slice: the corpus now runs 4.65 million instructions and prints verdicts, where
> before it printed nothing. **521 tests started, 202 failed**, and that pair is now
> asserted as a ratchet. The phase order's bet in §3 — that the oracle would be
> available from Phase A onward — is confirmed in the strongest form available, by
> the oracle actually reporting. Two defects it found in *Phase A* code are recorded
> in `Mars_Fpu.md` §9.2 and are the next slice's subject.

> **Phase B complete, 2026-09-16.** The software float landed (`Mars_FpuMath.md`), and
> then a slice whose job was to let the corpus run to the end of what Mars can attempt
> (`Mars_Corpus.md`). It now starts **720 tests and fails 171**, stopping inside the RSP
> tests — which is to say Phase B has no remaining corpus failure of its own, and the
> instrument is now asking for Phase C.
>
> **What the plan did not anticipate, in its own terms.** §8 predicts the ways this plan
> is likely to be wrong and none of them is this one: the thing that had been silently
> capping every measurement since the oracle started speaking was **twelve conditional
> trap instructions that no phase description mentions** (`Mars_Cpu.md` §15). They are
> not exotic and not late-added; they are ordinary MIPS III, and the plan's Phase A
> description of "the integer core" simply did not enumerate them. Implementing them
> cleared no test directly and made 159 further tests reachable, three of which turned
> out to be defects in subsystems this plan had already recorded as finished.
>
> The transferable form: **a phase is finished when the instrument says so, and the
> instrument cannot say so about tests it never reaches.** §3 bet correctly that the
> oracle would be available early; what it did not say is that an oracle's silence
> about a subsystem is not evidence about that subsystem.

### 4.3 Phase C — the RSP

The scalar subset, the 8-element vector unit with its 48-bit accumulator, the
clamping and saturation rules that differ per instruction family, the reciprocal
and reciprocal-square-root table instructions, DMA between DMEM/IMEM and RDRAM, and
the SP control registers including halt, break and the semaphore.

This is the phase where real microcode first runs, and the first point at which a
commercial ROM does anything visible. It is also, with D, where the estimate in §7
is least trustworthy.

**Done when** the corpus's RSP sections pass and a commercial game's boot microcode
runs to the point of emitting a display list — which nothing can yet consume.

> **Progress, 2026-09-17: the scalar half.** The RSP runs, halts and breaks, with every
> scalar instruction, the interface registers from both sides, and the program counter
> (`Mars_Rsp.md`). Eighty-eight of the corpus's 248 RSP and SP groups pass; all but six of
> the rest are the vector unit, which is this phase's next slice.
>
> **The phase's larger effect was on the instrument.** The RSP learning to halt let the
> corpus run past the wait it had sat in since Phase A, and the run that followed was the
> first to reach the corpus's own summary line — after a detour through a harness that had
> been running the corpus on a 4MB machine when it assumes 8MB (`Mars_Corpus.md` §8). So
> the "done when" condition above is now checkable in the form it was written, against a
> complete run, for the first time.
>
> The complete run also moved work *backwards* into phases this plan had recorded as
> finished: seventeen CPU test groups, in TLB register masking, 64-bit TLB matching, the
> extended refill vector, and `Config` (`Mars_Corpus.md` §3). Phase A's completion was
> claimed against a truncated run; that claim is now known to have been premature, and it
> is recorded where it was made (`Mars_Cop0.md` §1) rather than here.

> **Progress, 2026-09-17: the vector unit.** Every vector instruction, load and store, the
> flags, the 48-bit accumulator and the reciprocals with their hidden registers
> (`Mars_RspVector.md`). All 155 vector groups pass and nothing else moved; the corpus's
> tally went from 319 failed assertions to 163 (`Mars_Corpus.md` §10). **243 of the 248 RSP
> and SP groups now pass.**
>
> **The first half of this phase's "done when" is therefore nearly met, and the half that is
> left is not RSP work.** Of the five RSP and SP groups that fail, four are the main CPU's
> sub-word access to RSP memory, which is a bus question, and one is an RDP freeze that
> belongs to Phase D. The phase description's DMA and SP-register items were built in Phase A
> and the scalar slice. What remains of Phase C as written is the second condition — a
> commercial game's boot microcode running to a display list — which no corpus test measures
> and which has not been attempted.
>
> **Progress, 2026-09-17: the corpus half is met.** The four `spmem` groups were the main
> CPU's sub-word stores into RSP memory, which latches whole words (`Mars_Memory.md` §2.4).
> **247 of the 248 RSP and SP groups pass, and the one left is an RDP test** that shares the
> RSP's register naming. What remains of this phase is the microcode condition.
>
> **Progress, 2026-09-17: the microcode condition is met, with two stand-ins named.** Wave
> Race 64's first graphics task runs to a seventeen-command display list handed to the display
> processor (`Mars_Microcode.md`). **It only does so when the test harness raises the video
> and serial interrupts**, because a commercial game submits no microcode until those two
> interfaces have spoken — and they are Phase E's. As built, neither game starts the RSP in
> fifty million instructions.
>
> **So Phase C's "done when" is met in the only form this plan's order allows**, and the
> dependency it hid is recorded as a finding rather than worked around: this phase's condition
> named a commercial program, and a commercial program waits on every device it uses
> (`Mars_Microcode.md` §5). The test's stand-ins are to be deleted when Phase E builds the two
> devices, and the test is expected to pass without them.

### 4.4 Phase D — the RDP

The command stream, the primitive types, TMEM and texture loading, the combiner,
the blender, z-buffering, coverage, anti-aliasing and dithering, writing into a
framebuffer in RDRAM.

Graded by §3's second oracle: a per-primitive pixel diff against a reference
software rasterizer, starting from single primitives with known parameters and
working up. Starting at "run a game and look at it" is the trap — a whole-scene
diff reports a wall of disagreements that teaches nothing, the same reason
`Mercury_Gameplan.md` §3.3 insisted an audio differential be a feature stream
rather than raw samples.

**Done when** a set of primitives covering each mode diffs to zero, in the sense
`EmuSen_Core_Gameplan.md` means when it says the GSU diffs to zero against Mesen.

> **Progress, 2026-09-17: the interface, the command stream, and the fill cycle.** The registers
> that hand the display processor its list, a stream that gathers each command's words before
> running it, and fills in whole pixels (`Mars_Rdp.md`). The corpus's six RDP groups pass —
> 159 → 152 — and Wave Race's first list now clears its depth buffer.
>
> **This phase does have a hardware oracle, for its interface.** `Mars_TestOracle.md` §5 said
> Phases D and E get nothing from the corpus. For the rasterizer that held; for the registers
> and the stream it did not, and those six groups are graded against silicon, which the
> reference this section names is not.
>
> **The slice was also where this section's grading instrument first became necessary.** Every
> pixel rule past "the fill goes where the corpus looks" had to be taken from a reading of the
> reference and labelled ungraded (`Mars_Rdp.md` §0, §5.2), and the edge rule is already one
> column and one row visible in a commercial depth buffer. The differential is the next piece
> of work this phase needs, before any further drawing is built on readings.
>
> **Progress, 2026-09-17: the differential exists, and this section's "done when" can now be
> measured** (`Mars_RdpDifferential.md`). Mars's display processor and angrylion replay one
> command stream and their memory is compared; parallel-rdp replays it too, as a check on angrylion.
> Its first run of fill cases refuted the scissor edge rule the previous note describes as
> ungraded, and the rebuilt rule now matches angrylion on every fill case the differential carries —
> so the fill cycle is the first mode for which the "diffs to zero" condition holds on the cases
> written so far, apart from one where the reference carries a validation workaround rather than a
> hardware claim.
>
> **Two findings about the instrument change how this section's condition should be read.** The
> reference contains at least one deliberate departure from what it models (§4.3 there), so "diffs
> to zero" has to mean *to zero apart from named artefacts*. And the two references disagree on at
> least one case (§4.5 there), so for some primitives there is no agreed answer to diff against;
> those are graded against angrylion and recorded as disputed, not as correct.
>
> **Progress, 2026-09-17: triangles, in the fill cycle** (`Mars_RdpTriangles.md`). The edge walker —
> the part that decides which pixels any primitive covers — built from the reference's and graded:
> eighty-two triangle cases, sixty of them random, match angrylion, parallel-rdp agrees on every one,
> and fill rectangles now go through the same walker without losing a case. What it does not yet do
> is the rest of this section's list — shading, texturing, depth, the combiner and blender, and
> coverage — so the fill cycle remains the only mode that diffs to zero. Coverage is the next piece
> that §2.1's bet depends on, and the walker is now in place to carry it.
>
> **Progress, 2026-09-17: coverage, and the one-cycle mode for flat primitives**
> (`Mars_RdpCoverage.md`). Coverage from the walker's sub-scanlines, the combiner, the blender with its
> hardware divider, dither, and the framebuffer with hidden RDRAM — 177 one-cycle cases match angrylion.
> **§2.1's condition was about whether coverage could be graded, and it can**: for the primitives built,
> it diffs to zero. What the slice adds to the reading of "done when" is that the oracle is now plural in
> practice — the references disagree on two one-cycle behaviours, and the reference's noise is a
> validation construct rather than a model — so "diffs to zero" is against angrylion, with those named.
>
> **Progress, 2026-09-17: shade, depth and the depth buffer** (`Mars_RdpDepth.md`). Gouraud shading
> and depth-tested triangles in the one-cycle mode: 171 cases match angrylion, and parallel-rdp agrees on
> all of them. **The remaining list for this phase is textures** — texture memory, sampling and filtering,
> the level of detail and texture rectangles — **and the two-cycle and copy modes.** The slice's larger
> lesson was about the instrument: two cases named for decal mode had never drawn a pixel, which only a
> breakage that survived exposed, so a case's name is not evidence of what it exercises.
>
> **Progress, 2026-09-17: texture memory, loads and point sampling** (`Mars_RdpTextures.md`). Tile and block
> loads into the display processor's texture memory, texture rectangles, perspective division and a
> point-sampled texel in every format, in the one-cycle mode: 203 cases match angrylion, texture memory
> byte for byte. **The oracle's plurality grew**: the references disagree on six texture behaviours — five at
> formats or boundaries ordinary content is unlikely to reach, and one, how the one-cycle mode's next-pixel
> texel is filtered, that content could — and parallel-rdp refuses several uploads outright, so five cases
> are graded against angrylion alone. The remaining list is filtering, palette lookup, the level
> of detail, and the two-cycle and copy modes.
>
> **Progress, 2026-09-17: filtering and palettes** (`Mars_RdpFiltering.md`). Four texels where filtering or
> a palette asks for them, the three-point filter and its mid-texel average, palette lookup and the palette
> load, in the one-cycle mode: 176 cases match angrylion, and parallel-rdp agrees with every one it accepts
> but a YUV tile read through a palette. **The disagreement the textures slice left open was not a rule**:
> the two references start their tiles at different sizes, which only the first command to name a tile can
> see. The remaining list is the level of detail, and the two-cycle and copy modes.
>
> **Progress, 2026-09-17: the level of detail** (`Mars_RdpLod.md`). The mipmap tile and the fraction, in the
> one-cycle mode: 153 cases match angrylion. **For the first time the cross-check cannot follow a slice's
> core**: parallel-rdp measures this mode's level of detail the two-cycle way, from the pixel across x and
> down y, where angrylion measures along the span from the next pixel. The two agree only where those
> coincide, so twenty named cases carry one dispute and the random cases are graded against angrylion alone.
> What "diffs to zero" means for this slice is therefore narrower than for any before it. The remaining
> list is the two-cycle and copy modes.
>
> **Progress, 2026-09-17: the two-cycle mode** (`Mars_RdpTwoCycle.md`). Two combiner and two blender cycles
> per pixel, the second cycle's texel and the four forms of converting one texel from another, the two tiles
> its level of detail picks, and angrylion's pipelining of one pixel's work into the next's: 159 cases match
> angrylion. **The cross-check narrows again, but for a reason the last slice did not have**: parallel-rdp
> runs a pixel's two cycles together, so the three inputs angrylion feeds from a neighbouring pixel are
> disputes, and the random cases are constrained never to select them rather than being dropped from the
> cross-check. Twenty-nine of the thirty-nine named cases still agree with both references. The remaining list
> is the copy mode.
>
> **Progress, 2026-09-17: the copy mode** (`Mars_RdpCopy.md`). Four texels fetched a group from texture
> memory's four banks and written to the colour image as bytes, with no combiner, blender or depth: 176 cases
> match angrylion, and **Phase D now has every cycle type**. The cross-check is at its narrowest here — seven
> disputes, because angrylion copies eight bytes at a time where parallel-rdp computes each pixel — and the
> lesson is about method rather than the mode: the random cases disagreed in numbers no rule explained, and
> fifteen single-axis probes all *agreed* before one disagreement appeared, from a triangle whose major edge
> the test helper had never put on the right. A case that cannot be built by the helper is a rule that cannot
> be graded. What is left in Phase D is chroma key and noise, neither of which either reference models in a
> way a case can grade.
>
> **Progress, 2026-09-17: chroma keying** (`Mars_RdpChromaKey.md`). The key a primitive measures its colour
> against, which becomes the pixel's alpha while the cycle's first input becomes its colour: 78 cases match
> angrylion. **This is the first slice with no cross-check at all** — parallel-rdp lists chroma keying among
> its missing features — so what the differential can say here is narrower than anywhere else in the phase,
> and the page says so before it says anything else. The evidence is the reference's arithmetic read
> carefully, seventeen breakages each caught by a named case, and a tool-free test; it is not a second
> opinion, and only hardware or an FPGA implementation could be one. Phase D's last named gap is noise,
> which neither reference models in a way a case can grade.
>
> **Progress, 2026-09-17: the referee pass** (`Mars_RdpReferee.md`). Not a slice — no rule changed and no
> case was added. Every dispute the differential has recorded, twenty-one of them, was carried to a third
> implementation, the N64_MiSTer core's VHDL, and its answer written beside angrylion's and parallel-rdp's.
> Five rules go to angrylion outright, two more lean that way, three go to parallel-rdp with one more
> leaning, and ten are unmodelled there. Two results are worth the phase's attention. The first is that the
> strongest confirmations — the combined input being the previous pixel's, and an 8-bit image taking green
> on odd addresses — come from code that core's author wrote his own way, while the rules it contradicts are
> the two blender skews of the two-cycle slice; **cross-pixel carries are where the three implementations
> part company**, and they are what a console test should settle first. The second is that the FPGA option
> raised for chroma keying does not pay out: that core decodes the key's commands into exactly the fields
> Mars reads and then reports the key alpha as unimplemented, so the keying arithmetic is still graded
> against angrylion alone.

### 4.5 Phase E — the peripherals, and the first frame anyone can see

> **Progress, 2026-09-17: the video interface's registers and scan** (`Mars_Video.md`). The fourteen
> registers, the geometry they imply, the raster a frame is placed in with its borders and its two frames of
> grace, and the walk for the two anti-alias modes that need no coverage, in both pixel formats: ninety cases
> match angrylion. The harness had to grow first — the probe now captures the frames angrylion scans out,
> the dump writer can set video registers and ask for a screen update, and `build-probe.sh` builds
> parallel-rdp's `vi-conformance`. That last tool reports **all twenty-four of its suites passing**, so the
> two references agree about this entire device; Phase D had twenty-one rules where they did not. What is
> left here is the anti-aliasing that reads coverage, the dither filter, divot and gamma, and then the
> interrupt and the timing that would let a game drive any of it.

> **Progress, 2026-09-17: the anti-aliasing filter** (`Mars_VideoFilter.md`). The two modes a game actually
> uses: a pixel's coverage read back out of RDRAM's hidden ninth bits, the six neighbours it is weighed
> against, and the pull towards the two of them that bracket it — one hundred and thirty cases match
> angrylion, on the first run. The FPGA core implements the same filter from an independent construction and
> confirms every rule in it but one; the one it contradicts is a fetch artefact both software references
> model, and it is now the first thing in Phase E to test on a console. What is left here is the dither
> filter, divot and gamma, and then the interrupt and the timing.

> **Progress, 2026-09-17: the dither filter, divot and gamma** (`Mars_VideoPasses.md`). One hundred and
> fifty-one cases match angrylion on the first run, and the picture side of the video interface is complete
> apart from one bit that is deliberately empty. The gamma dither reads a noise angrylion hashes from the
> pixel's position and the FPGA core takes off a free-running shift register; the two cannot be reconciled
> and neither is a measurement, so Mars reads the dither as zero and says what that costs. What is left
> here is the interrupt and the timing, and then AI, SI/PIF and PI.

> **Progress, 2026-09-17: the half line and the video interrupt** (`Mars_VideoTiming.md`). The interface now
> keeps time off the bus's own counter, reports the half line and the field in the current-line register, and
> raises the interrupt when the half line reaches the one a game asked for. Nothing that graded the picture
> slices can grade this one — the dump has no clock and the corpus tests no video group — so it rests on a
> prediction `Mars_Microcode.md` §3 registered before the device existed: its stand-in video interrupt was
> deleted and Wave Race still reaches its display list. Thirteen unit tests hold the semantics, and all
> seventeen applicable breakages are caught. What is left
> here is AI, SI/PIF and PI.

VI (including the filters the console genuinely applies — anti-aliasing, divot and
gamma — because a framebuffer read out raw is not what the machine displayed), AI
streaming to `DequeueAudioSamples`, SI and the PIF's joybus for controllers, MI's
interrupt aggregation, and PI for save hardware. Save types — EEPROM in two sizes,
SRAM, FlashRAM and the Controller Pak — detected per title, which is a heuristic
problem rather than a hardware one and should be treated as such.

This is the phase that makes `ICore` satisfiable: `GetFrameBufferRgba`,
`AudioSampleRate`, `SetButton`, `SaveSram`.

**Done when** a commercial title boots to its title screen headlessly, with audio
sample production non-zero and a controller able to reach the game.

### 4.6 Phase F — the project's own surfaces

`MarsDebugTarget` with named memory spaces and both disassemblers (VR4300 and RSP),
save states, cheats, coverage, breakpoints, and only now the `CoreCatalog`
descriptor claiming `.z64`, `.n64` and `.v64` plus the libretro cheat-database
folder name.

Registration is last because a descriptor is a promise to five consumers at once
(`EmuSen_Multicore.md` §3), and the cheat wiring fixed on 2026-09-15 is the
cautionary case: a core reachable from the frontends before its seams are tested is
a core whose seams get tested by the user.

**Done when** `.z64` opens in both frontends, `emusen` and `disasm cpu` work against
Mars in DianaOS, and the cheat-wiring tests cover Mars the way they now cover the
other three cores.

### 4.7 Phase G — performance

Opens with a profile, not a plan. The question it must answer first is which of the
interpreter, the RSP and the RDP is the wall, because the levers are different and
the literature's default answer — "write a recompiler" — is an answer to only one
of them.

## 5. Where this will actually hurt

Listed because a plan that only lists phases implies uniform difficulty, and this
one is not uniform.

- **The RSP's vector semantics.** Per-instruction accumulator behaviour and
  clamping rules that resist being factored into one shared helper. Traditionally
  the single largest source of "one game is broken" in N64 emulation.
- **RDP coverage and anti-aliasing.** The part of §2.1's bet that can lose it.
- **Cache coherency.** The VR4300 has caches the software manages explicitly, and
  DMA that does not respect them. Ignoring the caches is viable until it is
  suddenly not, and that transition is usually a corrupted texture.
- **Unaligned access.** `LWL`/`LWR`/`LDL`/`LDR` and their store forms are a classic
  source of subtly wrong values that survive casual review.
- **Exceptions in delay slots**, and the branch-likely instructions' nullification
  rules.
- **Timing that is not the CPU's, and is documented nowhere.** `Count`'s rate is
  settled — half the CPU clock, stated by the vendor. Everything else in this entry
  is not: RDRAM latency, DMA durations, bus arbitration between the six masters, and
  cache miss cost in system cycles are absent from every source, and the community's
  own task list says so in writing. Games busy-wait on these. It also reframes the
  per-game timing knobs in §2.1's evidence — those emulators were working around
  timing **nobody has written down**, which is not an excuse but is a location.
  `Mars_Documentation.md` §5.
- **The RDP's subpixel-mask rule, which no document states.** The one piece of the
  rasterizer that has to be inferred from a reference rather than implemented from a
  specification — `Mars_Documentation.md` §4.
- **Save-type detection**, which has no header field and is conventionally a
  per-title database — an honesty problem as much as a technical one. Confirmed
  from primary sources: two independent emulators each ship one, and Project64's
  own documentation states outright that 4 kbit and 16 kbit EEPROM cannot be told
  apart by observation. `Mars_References.md` §5.1.
- **`Count` bookkeeping spread across the opcodes.** A leak with twenty years of
  evidence behind it in another project's commit log; whatever advances the counter
  in Mars has to be something an opcode or an exception path cannot forget to call
  — `Mars_References.md` §5.3.

## 6. Deferred, each with the condition that reopens it

- **A recompiler** — Phase G, and only if the profile names the interpreter.
- **HLE graphics or audio as an option** — reopens only if §2.1's condition fires.
- **A GPU-backed RDP** — same.
- **64DD, the Expansion Pak as a default, rumble, the Transfer Pak, netplay** —
  each reopens when something concrete needs it, not before.

## 7. Scale, stated without false precision

Mercury is 226 tests and took four phases; it is a 4 MHz 8-bit machine with one
display mode family. Mars has two processors before the RCP is counted, and the
RSP and RDP are each plausibly a Mercury-sized effort on their own. The honest
statement is that this is the largest single piece of work the project has taken
on, that the estimate for Phases C and D is the least reliable part of this
document, and that no completion date appears anywhere in it on purpose.

One consequence worth stating outright: while Mars is in progress, "EmuSen runs
well on low-end hardware" remains a claim about the other three cores. Quoting it
with Mars included is wrong until Phase G has measurements.

## 8. How this plan is most likely to be wrong

- **§3's verdict channel may not exist in the form assumed.** First task of Phase 0,
  and the fallback inverts the phase order.
- ~~**The hardware claims here are from general knowledge, not from citation.**~~
  **Partly closed 2026-09-15.** `Mars_Documentation.md` is the assessed source list,
  and it found the vendor manuals more complete than assumed for the CPU, adequate
  for the RSP's structure, and silent on one tier of the RDP and on system timing
  entirely. It also found the most authoritative document for one RSP instruction to
  be **wrong** (§3.1), which is the standing argument against implementing from a
  reading without a test. The individual claims in this plan still each want checking
  as their phase opens; what has changed is that there is now a map of what to check
  them against.
- **The phase boundaries assume the subsystems separate cleanly.** A/B/C/D are drawn
  where the test corpus's own sections are drawn, which is a good sign but not a
  guarantee — the RSP's DMA touching RDRAM during a CPU write is exactly the kind of
  interaction that respects no phase boundary.

## 9. Resuming

```
dotnet test EmuSen.WiseMan/EmuSen.WiseMan.csproj --filter "FullyQualifiedName~Mars"
```

That filter matches nothing today. Phase 0 is what makes it match something, and
until §3's first task is answered, no phase after it is safe to start.
