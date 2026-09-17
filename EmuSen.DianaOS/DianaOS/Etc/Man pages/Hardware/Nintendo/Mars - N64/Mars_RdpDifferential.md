# Mars — the display processor's differential, and what it grades

*Landed 2026-09-17. The instrument `Mars_Gameplan.md` §4.4 says Phase D is graded with: the same
command stream put through Mars's display processor and through a reference rasterizer, and the
memory each leaves behind compared byte for byte. The tests are `MarsRdpDifferentialTests`; the
tools are built by `./build-probe.sh rdp`. §4 is the reason to read this page: the instrument's
first run refuted a rule `Mars_Rdp.md` had labelled ungraded, found a workaround inside the
reference that is not a claim about hardware, and found a case the two references disagree on.*

---

## 1. Two references, and what each is for

**angrylion-rdp-plus grades Mars. parallel-rdp checks angrylion.** Decided by the project owner
on 2026-09-17, when asked which of the two to grade against; `Mars_Documentation.md` §7 had
recorded a preference for parallel-rdp on licence grounds, and that preference is retired there.

- **angrylion** is the software rasterizer the emulation community treats as ground truth, and
  the one parallel-rdp's own conformance suite is measured against. Grading against it removes a
  layer of agreement. It runs on the CPU, deterministically, and has a plain C interface. It is
  under the MAME licence, which permits this use: it is run locally as an instrument, and none of
  it enters the repository.
- **parallel-rdp** (MIT) is a Vulkan compute implementation of the same chip, written by studying
  angrylion. Its value here is independence of a limited kind: where it and angrylion agree on a
  case, the answer is less likely to be one implementation's accident. Where they disagree, the
  case is named (§4.5).

**What a pass certifies**, stated to the standard `Mars_TestOracle.md` §5 set: agreement with
angrylion, on a case where parallel-rdp also agrees with angrylion. **It is not agreement with the
hardware**, which neither reference is.

## 2. The shape

**One dump, two replays, one comparison.** The test writes every case into a single `RDPDUMP2`
stream, the interchange format parallel-rdp's tools read. Each case is:

1. a flush of RDRAM and hidden RDRAM from an upload cache, which returns both memories to zero
   apart from anything a case has uploaded — since textures, a texture image, which stays in the
   cache for every later case;
2. the case's commands, each as its 32-bit halves;
3. a `SignalComplete` record, which is the comparison point for both tools.

The display processor's own state is not reset between cases, in either reference or in Mars,
so every case sets everything it depends on.

**`rdp-reference`** replays the dump through angrylion exactly as parallel-rdp's angrylion driver
does — each command's words to `rdp_cmd` — and at each sync writes every 4 KiB page of RDRAM and
hidden RDRAM that differs from the last flush, all four kilobytes of texture memory, and anything
angrylion printed. Its output format, `RDPREF01`, is described at the top of its source.

**`rdp-validate-dump --sync-only`** replays the same stream through both references at once and
compares RDRAM, hidden RDRAM and texture memory at every sync, stopping at the first difference.
It is run unmodified.

**Mars** replays the cases through one display processor, feeding its command words directly —
the register interface in front of it is graded by the corpus instead (`Mars_Rdp.md` §2) — and
restores RDRAM from the same upload cache where the references flush. The test then compares, page
by page, every page either side touched, and reports a difference as pixels of the case's colour
image. A page the reference did not write is compared against the upload cache, not against zero —
§4.8 is why that sentence exists. Texture memory is compared whole.

**The references' memory is in host byte order, not the console's.** angrylion keeps RDRAM as
32-bit words in the host's order, so a 16-bit pixel `0x1111` beside `0x2222` reads `22 22 11 11` on
this machine. The fixture reverses each word before comparing. That was found by decoding the
first smoke test by hand, and it is the reason the tools are declared for little-endian Linux only
(§3).

## 3. Building it

```
git clone https://github.com/Themaister/parallel-rdp.git ~/Projects/parallel-rdp-reference
git -C ~/Projects/parallel-rdp-reference submodule update --init --recursive
./build-probe.sh rdp ~/Projects/parallel-rdp-reference
```

Both tools land in `~/.cache/emusen/probe/rdp/`, where `MarsRdpDifferentialTests` looks for them.
The first build takes several minutes, almost all of it Granite, the framework parallel-rdp's
tools are built on.

**The checkout is a local clone**, not a fork under the project owner's account as
`Mars_References.md` §1's four are — decided at the same time as §1. Its commits are recorded
there.

Five things about the build are not obvious:

- **CMake is borrowed, not installed**, from the distribution's packages into the work directory,
  as `build-probe.sh` borrows every other dependency. The download is pinned to the machine's
  architecture: without that, `dnf` also fetched the i686 package and the unpack order decided which
  binary survived.
- **Two compiler settings are required on this toolchain.** CMake 4 refuses Granite's third-party
  trees without `CMAKE_POLICY_VERSION_MINIMUM=3.5` — confirmed by configuring without it, which
  fails on meshoptimizer. GCC 16 no longer reaches `<cstdint>` transitively and Granite's headers
  relied on that, so the build force-includes it rather than editing the checkout.
- **`rdp-reference` links the angrylion library the validator's build produced**, so the grader and
  the cross-check run one angrylion rather than two builds of it that could differ by flags.
- **That library is not position-independent**, so `rdp-reference` alone links as a fixed-address
  executable.
- **angrylion writes through pointers it keeps into RDRAM and the video registers.** The first
  version of `rdp-reference` also took ordinary mutable borrows of that memory, which Rust's
  aliasing rules make undefined behaviour whether or not it misbehaves in practice. The memory is
  now reached only through the same raw pointers angrylion holds, with each borrow ended before
  angrylion runs again. angrylion's message functions are declared variadic, which stable Rust
  cannot define; the tool reads only the format argument, so a recorded message keeps its
  placeholders.

**Why the tests run the tools themselves.** The existing reference tests read dumps produced
offline, because a reference emulator takes minutes per run (`EmuSen_Debugging_Tools_Reference_v5.md`
§3.55). A display processor case replays in milliseconds, so the test writes the dump, runs both
tools and compares in one pass, and there is no stale output to be out of step with the cases. On
a machine where the tools were never built the tests return early and pass; that silence is the
cost of the arrangement, as it is for every reference test here.

## 4. What the first runs found

### 4.1 Twenty-six of thirty-eight

The first run graded thirty-eight fill cases. Twenty-six matched angrylion. The twelve that did not
fell into four groups, and the most important result is one that passed:

- **The scissor's right column is drawn** (five cases). `Mars_Rdp.md` §5.2 had made the right edge
  exclusive by assuming it mirrored the bottom edge, and said so. It does not mirror it.
- **A scissor bottom with a fraction includes that row** (three cases).
- **The scissor's interlace bits are honoured** (two cases), which Mars had ignored.
- **A rectangle running past the image's right side** (one case) — §4.3 — and one case combining
  fractional edges on all sides, which cleared with the rest.
- **Every fractional rectangle edge matched** (twelve cases). §5.2 of the RDP page had said
  fractional coordinates were "simply wrong here", because Mars truncates them and the reference
  places sub-pixel edges. For a fill rectangle the reference's sub-pixel walk truncates to the same
  pixels. The prediction was wrong in the direction of Mars being right.

### 4.2 The rule, as rebuilt and graded

Read from angrylion's rectangle path through its edge walker, written in Mars's own terms, and
then graded by this instrument rather than trusted:

- The rectangle's left and right edges are taken in **eighths of a pixel**. Each is moved onto the
  scissor's left edge if it is under it, and **then** onto the scissor's right edge if it is at or
  past it — in that order, which matters only for an empty scissor.
- A row is dropped if **both** edges were under the scissor, or **both** at or past its right edge,
  or if the right edge is left of the left edge at **quarter-pixel** precision.
- The span runs from the left edge's pixel to the right edge's pixel, inclusive.
- In the fill cycle the rectangle's bottom takes its whole last row. A row is drawn if **any of its
  four quarter-pixel sub-scanlines** lies at or below the larger of the rectangle's and scissor's
  tops and strictly above the smaller of the rectangle's bottom and the scissor's bottom.
- With the scissor's field bit set, only rows whose parity matches its keep-odd bit are drawn.

**Tier: reference, graded.** Of the forty-nine cases now in the dump, Mars matches angrylion on
forty-eight; the forty-ninth is §4.3's artefact, which differs by design.

### 4.3 A workaround inside the reference, which Mars does not copy

For a rectangle running past the colour image's right side, angrylion stops each span at the
image's last column. Mars instead writes on by address, into the start of the next row. **The
clamp is not a claim about hardware.** It sits under a comment reading *"Workaround game bugs in
validation which mistakenly render past their scanline"*, added in commit `9a1965c` of the fork
parallel-rdp vendors — *"Clamp scanline span lengths for sanity with validation"*, 2020 — and it is
absent from the angrylion in the parallel-n64 checkout.

The case is kept as an **artefact**: it is expected to differ, and fails if it stops differing.
What Mars does instead is not established either. Writing on by address is the obvious reading of
how a span's address is formed, and nothing here measures it.

### 4.4 A case one reference refuses

A fill into a 4-bit colour image draws nothing in angrylion, which records the pipeline as crashed
without printing anything. parallel-rdp **aborts the process** — *"RDP crash: Attempted to use Fill
mode on 4bpp surface"* — which on the first run cut the cross-check short and left the last three cases of
the dump unchecked. The case is graded against angrylion alone and left out of the
cross-check's dump.

### 4.5 A case the two references disagree on

**With an empty scissor** — left and right edges equal — angrylion draws nothing, and parallel-rdp
draws the scissor's one column. Mars follows angrylion, **because angrylion is the grader**, not
because it is known to be right; neither reference is hardware.

The case is left out of the agreement dump and given one of its own, and a test asserts that the
validator still rejects it. If either reference changes its answer, that test fails and this
section is out of date.

### 4.6 The instrument was checked for bite

Eleven deliberate breakages of the new rule were run against it. Nine were caught at once. The two
that survived — dropping the crossed-edges check, and dropping the guard for an empty range of
rows — needed geometry no case had: crossed edges within a single pixel, and a rectangle starting
below a fractional scissor bottom in the same row. With those two cases added, all eleven are
caught.

### 4.7 Wave Race's depth clear, re-measured

Its scissor and rectangle share the corners (8,20)–(311,219). Under the rule `Mars_Rdp.md` §8
measured, column 311 and row 219 were left uncleared. Under the graded rule, pixel (311,218) now
holds `0xFFFC`; (310,219), (311,219) and (312,218) still hold zero.

### 4.8 Pages the reference does not write

Until textures, no case uploaded anything, so a page the reference left out of its output was a page
of zeros, and the test treated it as one. Uploads, added for texture images, broke that assumption: an
image's page is unchanged by drawing, so the reference did not write it, and the test compared Mars's
copy of the image against zeros. **On the first run with textures built, every texture case failed on
that page alone** — and once the comparison took an unwritten page from the upload cache instead, every
one matched (`Mars_RdpTextures.md` §7.5). The assumption was sound when it was made; the slice that
invalidated it is the one that had to find it.

## 5. What it does not grade

- ~~**Hidden RDRAM.**~~ Compared since coverage landed, when Mars gained it (`Mars_RdpCoverage.md` §3.1).
- ~~**Texture memory**, which only the cross-check compares, and which no fill case touches.~~ Compared
  since textures landed (`Mars_RdpTextures.md` §7.1).
- **Anything but the fill cycle.** The cases are fill rectangles and, since the walker, fill-cycle
  triangles (`Mars_RdpTriangles.md` §4); every other mode is unbuilt in Mars (`Mars_Rdp.md` §9).
  **Update:** flat primitives in the one-cycle mode are now graded too (`Mars_RdpCoverage.md` §7), and
  the differential carries two further disputes and a second reference artefact (§6 there).
- **The register interface**, which the cases bypass, and **timing**.
- **Pipeline crashes.** angrylion's crash flag stops `n64video_process_list` and nothing else, and
  the replay never calls that function; what a crashed pipeline does next is not graded.

## 6. Keeping it honest

The claims here are about the checkout's pinned commits — parallel-rdp `1cecd042`, angrylion-rdp-plus
`31bdb1f`, Granite `cf71dee7` — and about no other version. §4.3's clamp is itself evidence that
the reference changes. If the checkout is updated, the tools must be rebuilt and the artefact and
dispute tests re-read before the tally is quoted.
