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
- ROM image handling: the three container formats (`.z64` big-endian, `.n64`
  byte-swapped words, `.v64` byte-swapped halfwords) normalised to one internal
  order at load, and the header parsed — entry point, the two CRCs, the region and
  cartridge id.
- A `SyntheticN64Rom` fixture in `EmuSen.WiseMan/Fixtures/`, following
  `SyntheticGbRom` and `SyntheticNesRom`: a builder that produces a valid, minimal,
  committed-to-nothing image with a caller-supplied payload, so the suite passes on
  a machine with no ROMs at all. The repo rule that no commercial image is ever
  committed applies unchanged.
- The `TestRoms/` discovery path for the corpus, mirroring
  `HardwareTestRomLibrary`, so an absent corpus skips rather than fails.

**Done when** a WiseMan test can load a synthetic image, assert its header, and — if
§3's channel exists — run a corpus ROM to a captured text verdict, with the CPU
still entirely unimplemented.

### 4.1 Phase A — the VR4300 integer core, the memory map, and boot

The 64-bit MIPS III integer instruction set, the delay-slot rules, the exception
model, COP0 (`Count`/`Compare`, `Status`, `Cause`, `EPC`), and the 32-entry TLB.
Alongside it the physical memory map: RDRAM, the RSP's DMEM and IMEM as plain
memory, the MMIO register blocks stubbed to sane reads, and PI DMA from the
cartridge.

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

### 4.5 Phase E — the peripherals, and the first frame anyone can see

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
