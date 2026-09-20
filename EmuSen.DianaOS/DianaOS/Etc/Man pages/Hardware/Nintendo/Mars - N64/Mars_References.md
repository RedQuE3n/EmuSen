# Mars — the reference emulators, what they are for, and the line around them

*Written 2026-09-15, when four existing N64-adjacent codebases were checked out for
study. They are read, never copied, and §2 is the rule that keeps that true rather
than aspirational. §5 is the half that turned out to be worth the most: not what
these projects got right, but what they got wrong and had to paper over, which is
evidence about the hardware that no correct emulator would ever have recorded.*

---

## 1. What is checked out, and where

Outside the repository, beside `mesen-reference/` which set the precedent, as forks
under the project owner's account so an upstream rewrite cannot move the ground
under a citation. Full history, because §5 needs the commit log.

| Checkout | Upstream | License | At clone |
|---|---|---|---|
| `~/Projects/project64-reference` | `project64/project64` | GPL v2 | `7720a9462`, 2026-08-27 |
| `~/Projects/retroarch-reference` | `libretro/RetroArch` | GPL v3 | `ce5544fdb0`, 2026-07-24 |
| `~/Projects/mupen64plus-core-reference` | `mupen64plus/mupen64plus-core` | GPL v2 | `6dca4c15`, 2026-07-06 |
| `~/Projects/parallel-n64-reference` | `libretro/parallel-n64` | GPL v2 | `39819865`, 2026-07-24 |
| `~/Projects/parallel-rdp-reference` | `Themaister/parallel-rdp` | MIT; its `angrylion-rdp-plus` submodule MAME | `1cecd042`, 2024-11-09; angrylion-rdp-plus `31bdb1f`; Granite `cf71dee7` |

> **Added 2026-09-17, and held differently.** The fifth checkout is the display processor's
> differential (`Mars_RdpDifferential.md`), cloned with its submodules on the day the instrument was
> built. **It is a local clone, not a fork** — the project owner's decision when asked — so the
> commits above are what pins it, and an upstream rewrite would move a fresh clone but not this one.
> It is the only checkout here that is compiled and run rather than read, and nothing of it is in
> this repository either.

> **Provenance, read 2026-09-19.** Two of the checkouts are not upstream. `parallel-n64-reference` is a locally
> modified fork whose commit log heads with its own angrylion work, so its comments describe the fork's reasoning
> and not upstream's. The `angrylion-rdp-plus` submodule under `parallel-rdp-reference` is Themaister's fork,
> r8-26-g31bdb1f, carrying determinism patches upstream lacks (`Mars_Rdp.md` §10.1). The differential grades against
> that fork, and the fork's rules — the combined colour cleared at each primitive, noise a function of position — are
> the ones Mars follows. A citation of angrylion's behaviour should say which angrylion.

Nothing about them is in this repository — no submodule, no vendored file, no
gitignore entry, because there is nothing here to ignore.

## 2. The line, stated once

> **Correction, 2026-09-15.** A fifth codebase matters to Phase D and is not in §1's
> table: **angrylion, the RDP pixel reference, is under the MAME licence** — not GPL,
> not permissive, and displayed as no licence at all by GitHub, which is how projects
> come to treat it as permissive. It is non-commercial, and it asks in terms for
> exactly the discipline this section already imposes: read it to understand the
> hardware, then write your own. The practical consequence is a preference —
> parallel-rdp is MIT and is itself validated bit-exact against angrylion, so grading
> against parallel-rdp reaches the same standard through a permissively licensed
> instrument. `Mars_Documentation.md` §7.
>
> **Update 2026-09-17: both are now run, and angrylion is the grader** (`Mars_RdpDifferential.md`
> §1). The preference is retired in `Mars_Documentation.md` §7. Running an implementation is a
> different use from reading it: nothing is copied, and the §2 rule still governs the source.
> Mars's fill rule was read from angrylion's edge walker for mechanism and written in its own terms,
> and then graded by running angrylion — the rule's authority is the grading, not the reading.

**Every one of these is GPL. EmuSen is not.** Copying any of it — a function, a
table, a struct layout transcribed field for field — would put this project under a
licence it has not chosen, and the fact that a reader could not tell is exactly why
the rule has to be procedural rather than a matter of good intentions.

The working rule, which is narrower than "don't paste":

- **Read for mechanism, write from hardware documentation.** The legitimate use is
  *knowing what to go and read about* — that a subsystem exists, that a behaviour
  has an edge case, that a value has a name. The implementation is then written
  against the hardware reference, and the hardware reference is what gets cited in
  the man page, not the emulator that pointed at it.
- **Constants and tables come from hardware or from our own generation.** The RSP's
  reciprocal tables are the case that will actually come up in Phase C. They are
  derivable, and derived is what they must be.
- **A file is never open in an editor next to the file being written.** The
  practical form of the rule; transcription happens by proximity long before anyone
  decides to transcribe.
- **Their per-game databases are read as evidence, never as data** (§5.1). What the
  entries prove is interesting. Importing them would be importing the thing this
  core exists not to need.

None of this is about their authors deserving less credit. It is that the moment
Mars's provenance is arguable, the project's claim to be independent research is
gone, and no amount of it having been quicker would buy that back.

## 3. Which of the four is which

**`mupen64plus-core`** — the structural reference, and the most useful of the four
for orientation. Its `src/device/` decomposes the machine into exactly the units
the plan's phases assume: `r4300/`, `rdram/`, `pif/`, `cart/`, `controllers/`, and
`rcp/` split into `ai/ mi/ pi/ rdp/ ri/ rsp/ si/ vi/`. That an independent project
arrived at the same decomposition is weak evidence the phase boundaries in
`Mars_Gameplan.md` §4 are cut along real seams rather than convenient ones.

**`parallel-n64`** — the one that matters for the oracle, and the reason §4 below
exists. It carries three RSP implementations (`-rsp-hle`, `-rsp-cxd4` as an LLE
interpreter, `-rsp-paraLLEl`) and three video plugins including
`mupen64plus-video-angrylion`, the pixel-exact software RDP that
`Mars_Gameplan.md` §3 names as the Phase D reference. Having the reference's source
rather than only its binary is what makes §4 possible.

**`project64`** — the mistake record (§5), and a study of what HLE costs over
twenty years. Also the only one of the four with a serious Windows-first history,
which is not relevant to us.

**`retroarch`** — **contains no N64 emulation whatsoever.** Worth stating plainly
because it was pulled in expecting otherwise: RetroArch is the frontend, the cores
are separate projects, and a search for N64 anything in its tree returns nothing.
It stays checked out for a different reason: it is the reference implementation of
the API our probe already speaks (`build-probe.sh` pins `v1.22.2`), and its
frontend-side subsystems — `state_manager.c`, `runahead.c`, the audio driver's rate
control, `cheat_manager.c` — are the closest thing to peers for parts of EmuSen
that are not cores at all. Those comparisons belong in `EmuSen_Audio_Sync.md` and
the rewind documentation, not here.

## 4. What this changes in the plan, today

`Mars_Gameplan.md` §3 ranked a pixel-exact RDP reference second among the oracles
and left its mechanism unspecified. It is now specific, and cheaper than assumed.

`angrylion/rdp_dump.h` exposes a command-stream trace: `rdp_dump_emit_command`,
plus `rdp_dump_flush_dram`, `rdp_dump_flush_hidden_dram`, the VI register writes and
a frame boundary. That is exactly the shape a Phase D differential needs — the same
command stream into two rasterizers, the framebuffer diffed after. The presence of
a *hidden* DRAM flush alongside the main one is itself the confirmation that the
RDRAM's extra bits per byte are observable state the reference tracks, which
`Mars_Gameplan.md` §5 listed as a hazard without knowing the reference modelled it.

`rsp_dump.cpp` in the cxd4 plugin is the same idea one processor up, and may do for
Phase C what the corpus cannot — though the corpus remains the primary grader
there, for §2.1's reason in the gameplan.

**This does not make the reference an authority.** A dump is a measurement
instrument: it says what angrylion did, and angrylion is believed pixel-exact
because of how much it has been compared against hardware, not because it is
definitionally right. `Mercury_Gameplan.md` §3.1's lesson holds unchanged — our own
tooling's output is evidence about our tooling.

## 5. What they got wrong, which is the valuable half

The instruction was to learn from the mistakes too. These are not criticisms of
projects that solved a harder problem earlier with less; they are the hardware
telling us where it bites, recorded in the only form it survives in.

### 5.1 Both independent projects needed a per-title database, and the knobs name the gaps

Counted at the commits in §1:

| | Project64 `Project64.rdb` | mupen64plus `mupen64plus.ini` |
|---|---|---|
| Titles with entries | 1,587 | 3,292 |
| Explicit save-type overrides | 86 | 886 |
| Timing fudge factor | `Counter Factor`, 550 | `CountPerOp`, 213 |
| DMA behaviour override | `Unaligned DMA`, 301 | `SiDmaDuration`, 16 |
| Memory size override | `RDRAM Size`, 1,181 | — |

Every one of those columns is a behaviour the emulator could not derive from the
machine and had to be told per game. The timing factors are the loudest: two
projects, independently, ended up with a per-title multiplier on how fast the CPU's
counter advances, which is a statement that the real timing was never modelled and
that games notice.

Project64's own documentation is admirably direct about the worst of it. On save
types: *"it is not possible for the emulator to detect the difference between a game
asking for 16kbit and a game asking for 4kbit"*. On memory size, it records that
some entries exist because the video plugin faults without the Expansion Pak *"for
reasons unknown to me"* — a per-game workaround for a bug in a different component
entirely, preserved in a shipped database.

**What Mars takes from this.** The save-type problem is real and is not solved by
being more careful: `Mars_Gameplan.md` §5 already calls it a heuristic problem, and
this is the confirmation. What LLE buys is that the *other* columns should not
exist. If Mars ever acquires a per-game timing factor, that is the signal that a
piece of the machine was never modelled — not a feature, and the gameplan's §2.1
condition is the thing to re-read.

### 5.2 Host floating point was not good enough, and the fix came late

Project64's interpreter used the host's floating point, and a commit named *"Core:
Convert interpreter FPU ops to use softfloat"* moved it onto Berkeley SoftFloat,
now vendored in its `3rdParty/`. The lesson for Phase B is the ordering, not the
outcome: a whole class of divergence had to be chased down and re-fixed once the
FPU was known to be wrong in the last bit.

Mars's Phase B should therefore not begin with C#'s `double` and an intention to be
careful. A software implementation with explicit rounding modes is the starting
position, and the corpus's COP1 sections are what say it worked.

### 5.3 The counter is a recurring bug, not a one-time one

`git log` in the Project64 checkout shows a long tail of fixes clustered on the same
thing years apart: updating counters when writing `Count` from `MTC0`, updating them
before reading it back, updating around exceptions in `ADD`/`SUB` and
`DADD`/`DSUB`, and a double counter update in `JAL`. Each is small; together they
say that "advance the cycle counter" being the caller's responsibility, spread
across every opcode and every exception path, is a design that leaks forever.

**What Mars takes from this.** Whatever advances `Count` should be one mechanism the
opcodes cannot forget to call, and the exception path is part of it rather than a
place that has to remember. Mercury's cycle-granular bus is the local precedent —
the CPU cannot execute without the machine advancing, because advancing is what
access *is*.

### 5.4 Three RSPs and three rasterizers in one distribution

parallel-n64 ships an HLE RSP, an LLE interpreter and a Vulkan-backed one, plus
three video plugins. That is not indecision; it is what happens when accuracy and
speed cannot be had together and the answer is left to the user. It is worth seeing
as the alternative to the gameplan's §2.1 — the honest version of "support both" is
maintaining several backends forever, and the reason to refuse it here is that this
project has one person and an accuracy claim to defend, not that the approach is
unreasonable.

## 6. What is not here

- **ares** — the current accuracy leader, and the reference most worth having for
  Phase 0's verdict-channel question. Not checked out.
- **CEN64** — the cycle-accuracy attempt. Not checked out.
- **`n64-systemtest`** — the hardware corpus, which is the actual oracle for Phases
  A–C and is not an emulator at all. Not checked out, and it is the most important
  missing piece.

## 7. Keeping this honest over time

The table in §1 is pinned to a commit each. Nothing here fetches automatically, and
a claim in this document is about that commit and no other. If a checkout is
updated, the counts in §5.1 must be re-run rather than assumed to have held —
they are measurements of a moving upstream, taken once.
