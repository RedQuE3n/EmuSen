# Mars: a plan for drawing the multiple on the GPU

*Drafted 2026-09-20. A plan, not a record: nothing here is built. It follows `Mars_Rdp.md` §11 (the picture at a
multiple), `Mars_Video.md` §2.10 (antialiasing) and `Mars_Performance.md` §37 (what frame pacing measured), and it
should be retired into those pages, phase by phase, as it is carried out or abandoned.*

## 0. What this is for, and what it is not for

**It is for the multiple, and only the multiple.** `Mars_Rdp.md` §11.1 measured the internal resolution at two at a
third to a half of the frame rate and at four at a quarter, because the display processor's threads shade 1 + N²
pictures on the CPU. That work is per pixel, independent between pixels once order is respected, and integer: the
shape a GPU is for.

**It is not for the machine, and it would not make the machine faster.** `Mars_Performance.md` §37 found that at
one the frames that overrun their slot are the game's own drawing frames, and that in those frames the display
processor's waits are two to six milliseconds of twenty-two to thirty-two: the bound is the emulation thread, the
CPU and the signal processor, as §35 and §36 said it had become. A GPU rasteriser takes nothing off that thread.
The speed of the game at one is a question for the signal processor (`Mars_Rsp.md` §10.1) and is not this plan's.

**The exactness contract does not move.** The software rasteriser stays the machine's: what games read back, what a
state holds and what the probe grades are its bytes, as now. The GPU replaces exactly one thing, the processors at
the multiple and their shadow memory (`DpInterface.Scale`, `Worker.Scaled`), whose picture is already an
approximation of the console's by design. With the GPU absent, unsupported or switched off, the CPU path of §11
remains, unchanged, and is also the GPU path's oracle (§4).

## 1. Relation to parallel-rdp

parallel-rdp (`Mars_References.md`) is the existing proof that the display processor can be run in compute shaders,
bit-exact against angrylion, and `Mars_Rdp.md` §10.1 already records what was read of its architecture: primitives
binned into screen tiles, and each pixel's invocation walking its tile's primitives in submission order so that
blending and depth see what the console's pipeline would. That ordering scheme is forced by the problem, not by
their code: the display processor's pixel depends on every earlier primitive at that pixel and on nothing else, so
any correct parallel formulation is "per pixel, in order". This plan takes that idea and nothing else.

**The rule for the work: no shader is written from their source.** Every stage is ported from Mars's own C#
(`Rdp.OneCycle.cs`, `Rdp.TwoCycle.cs`, `Rdp.Depth.cs`, `Rdp.Coverage.cs`, `Rdp.Textures.cs`, `Rdp.Lod.cs`), which
was itself written against angrylion's behaviour and the hardware corpus and carries its own provenance ledger
(`Mars_Rdp.md` §0). parallel-rdp stays where it is now, a differential reference run as a black box
(`build-probe.sh rdp`). Its licence would permit more; the project's purpose does not want it.

**Where this design departs from theirs, on purpose.** They rasterise on the GPU: edge setup, span generation and
binning are shader work, and their host side is thin. Mars's walker is under two per cent of the rasteriser's time
(§10.2) and is exact, tested and already runs at a multiple (§11), so here the **CPU walks and the GPU shades**:
the host produces spans and per-row attributes exactly as `Walk` does now, and uploads them. That keeps the
hardest-won exact code in one language, removes the edge walker from the shader port entirely, and makes the unit
of GPU work the same unit the CPU split already uses, the row.

## 2. The graphics API

| Option | For | Against |
|---|---|---|
| **Vulkan compute through Silk.NET** | C# bindings, MIT, no native code of ours; compute needs no window, so WiseMan can run it headless; lavapipe gives a software device for tests on a machine with no GPU; integer storage buffers and images are first class | Verbose; macOS only through MoltenVK; a loader and a driver become a run-time dependency |
| OpenGL 4.3 compute in Avalonia's context | Already have a context in the window | No headless path, so it breaks the harness rule; deprecated on macOS at 4.1, which has no compute at all |
| wgpu-native | Portable, modern | A native Rust library to build and ship per platform, and WGSL's integer support is thinner |
| Direct3D 12 / Metal natively | Best on their platforms | Two or three back ends for a research project |

**Recommendation: Vulkan compute through Silk.NET**, shaders in GLSL compiled to SPIR-V at build time and embedded
as resources. It is the only option that satisfies the harness rule (tests headless through WiseMan) and the stack
rule's spirit (no new language for host code; GLSL is a new artefact and the stack page should say so when it
lands). The GPU path is optional at run time: no Vulkan device means the CPU path, silently, with a line in the log.

**Presentation is by readback at first.** The shaded multiple is copied to host memory once per scan and handed to
`Vi` where `ScaledRdram` is now; 1,280×960 sixteen-bit pixels plus hidden bits is about four megabytes a frame,
well inside what a discrete or integrated GPU returns in under a millisecond or two. Sharing the image with
Avalonia's renderer would remove the copy and is deliberately left to the end (§5, phase 7), because it couples the
core to a frontend's renderer and buys little until everything else is fast.

## 3. The design

**Host side, per primitive** (in `Rdp` at a multiple, replacing `DrawOneCycle` and its siblings for the GPU
back end): run `Walk` as now; append to a frame-long **primitive buffer** one record — the mode words as already
decoded (`_otherModes`, the combiner selections, blend selections, the constant colours, primitive depth, fog,
key, convert), the attribute steps, the tile descriptors in use, and an index into a **texture memory ring** —
and to a **row buffer** one record per drawn row: left, right, the row's eight attributes at the major edge, the
major column, and the four sub-scanline edges coverage needs. A snapshot of the four kilobytes of texture memory
is pushed to the ring only when a load has changed it since the last primitive that sampled it.

**Binning.** The host bins rows, not triangles: the screen is cut into tiles of 8×8 pixels of the multiple, and
each row record is appended to the list of every tile its span touches, in submission order. Lists are built on the
host in the same pass that writes the rows (a span touches `(right − left) / 8 + 1` tiles of one tile row), then
uploaded as one offsets array and one indices array.

**Device side.** One dispatch per flush, one invocation per pixel of the multiple. The invocation loads its pixel's
colour, coverage and depth from the frame images, then walks its tile's list in order; for each row record that is
its own row and covers its column it runs the pipeline ported from the C# — coverage from the sub-scanline edges,
attribute at the column from the row's start and the steps, texel fetch and filter from the ring's snapshot, level
of detail, combiner, blender with the memory colour held in a local, depth compare and update — and carries the
result to the next record. At the end it stores colour, coverage and depth. All arithmetic is 32-bit integer, as in
the C#; no floating point enters the pixel path, so drivers cannot disagree about the result.

**The carries the CPU split serialises** (`Mars_Rdp.md` §2.8: the two-cycle mode's first blend from the previous
pixel's memory alpha, and `COMBINED` selected across primitives) are carries between *neighbouring pixels*, which a
per-pixel invocation cannot see. Phase 4 decides between two honest options: compute the predecessor's value inside
the invocation (one extra partial pipeline evaluation for the pixel to the left, only in those modes), or draw the
primitives that use them on the CPU path into the same images after a flush. The second is simpler and those
primitives are rare (§35's counters give their share per game); measure before choosing.

**Flushes.** The batch is dispatched when the scan-out needs the frame, when the colour or depth image changes,
when a load reads bytes the batch has drawn (the hazard §2.8 already detects — and at a multiple the load reads the
*native* memory, so this is a no-op for the GPU path and stays the CPU's business), and when a buffer fills. Frame
images live on the device per colour-image address, keyed as the shadow's addresses are now, so a game that
alternates two or three frame buffers keeps two or three device images.

**What stays as it is:** `DpInterface`'s threading, marks, barriers and workers for the native drawing; the state
format (the GPU holds no state a save needs: a state read clears the device images as `ResetScaled` clears the
shadow); the settings (`RenderScale`, `Antialiasing`), to which one is added, the multiple's renderer: CPU or GPU.

## 4. How it is proved

**The CPU path at the multiple is the oracle, and the comparison is bit for bit.** This is the project's unusual
advantage: `Rdp` already draws at a multiple on the CPU (§11), in integers, and the shaders are a port of that same
code, so the GPU's frame images must equal `ScaledRdram` and `ScaledHidden` byte for byte on every scene — not
"close", as the multiple is to the console, but identical, as two implementations of one function. The scenes are
the ones `MarsThreadedRdpTests` and `MarsRdpDifferentialTests` already build, plus recorded display lists
(`rdp-frames/`) replayed through both. A mismatch is a porting defect by definition, and the first differing pixel
names the stage. Tests run headless on lavapipe; a machine with no Vulkan device skips them and says so.

**Speed is measured the way §11.1 was**: `playbench` with `RENDERSCALE` and a new `GPU=1`, modes interleaved from
the in-game states. The plan's reason to exist is a number, so each phase that can be measured is.

## 5. Phases, each with its exit

| Phase | What | Exit |
|---|---|---|
| 0 | Spike: Silk.NET Vulkan instance, compute pipeline, storage buffers, readback, in a WiseMan test on lavapipe and on the real device | A shader adds two buffers and the test reads the sum; packaging cost and start-up time written down |
| 1 | Frame images, primitive, row and tile buffers; the fill cycle and the copy mode's rectangles | Fill and copy scenes identical to the CPU multiple |
| 2 | One-cycle, untextured: coverage, shade, combiner, blender, depth, dither | The shaded scenes identical; first `playbench` number at 2× and 4× on a shade-heavy state |
| 3 | Textures: the ring, tile descriptors, all formats and TLUT, filtering, level of detail, perspective | Recorded display lists of the three probe games identical for a frame each |
| 4 | Two-cycle, and the neighbour carries by whichever of §3's options measures better | Every scene in the threaded suite identical; the serialised share recorded |
| 5 | Integration: a GPU back end behind `DpInterface.Scale`, flush rules, the scan-out reading the readback, the setting, fallback when no device | Majora's Mask and the three probe games play at 2× and 4×; the probe at scale one still identical |
| 6 | Measurement and the decision | 2× within five per cent of 1×'s frame rate and 4× at full speed on this machine, or the plan is recorded as not worth its dependency and retired |
| 7 | Optional: supersampling's average on the device; sharing the image with the frontend instead of readback | Only if phase 6's profile shows the readback or the average as the remaining cost |

Phases 0 to 2 are the cheap part and answer the expensive question early: if shading a shaded, depth-tested scene
at 4× is not several times faster than four CPU workers by the end of phase 2, stop there.

## 6. Risks, stated in advance

- **The dependency.** A Vulkan loader, a driver and, on macOS, MoltenVK. Mitigated by the path being optional and
  the CPU path remaining the default until phase 6 says otherwise.
- **Frames the CPU writes** are as invisible to the GPU path as to the shadow (`Mars_Rdp.md` §11). This plan does
  not fix that; a fix (seeding the multiple from native memory when the machine has written it) would serve both.
- **Latency of the flush.** The scan-out must wait for the dispatch that holds its frame. With §2.7's deferred
  presentation that wait is off the machine's thread already; without it, it is a stall to measure in phase 5.
- **Tile list size.** A full-screen rectangle at 4× touches every tile row by row; the lists are bounded by rows ×
  tiles across and must be sized from the recorded lists before phase 1 fixes a format.
- **Driver differences.** Integer-only shaders remove the usual cause; lavapipe and one real device in the tests
  catch the rest.
- **Scope.** The pixel pipeline is some three thousand lines of C#. The port is the bulk of the work, and §4's
  oracle is what keeps it honest.
