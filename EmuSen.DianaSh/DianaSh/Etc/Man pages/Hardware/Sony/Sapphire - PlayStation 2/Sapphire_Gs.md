# Sapphire_Gs — the GS drawn on the device

*Started 2026-10-31, funded by the tester the same day.* Sapphire's GS rendering on a Vulkan device, beside the
software rasteriser (`Sapphire_Native.md` §24, §27, §31, §50), which stays the exact reference and the oracle. Code:
`src/gs/device/` (the plan and the device's side), `shaders/walk.comp` (the walk), `src/gs/stream.rs` and
`examples/gsstream.rs` (GS streams). The marks are the project's: **[M]** measured on the date given, **[R]** read,
**[A]** argued, **[C]** a choice.

## 1. The form, and why it is compute

**Decided 2026-10-31 by the tester:** Play!'s form. The GS's 4 MiB of local memory stay on the device in the GS's own
page, block and column layout. Every PSM's addressing, the pixel pipeline and texturing are emulated in shaders,
reading and writing that memory. There is no texture cache and no render-target tracking, so there are no per-game
fixes. It is not PCSX2's hardware renderer, whose texture cache needs 3,928 per-game `gsHWFixes` lines [R, PCSX2's
`GameIndex.yaml`, counted 2026-10-31].

**Departure, accepted by the tester the same day: the rasterisation is a compute walk, not Play!'s fixed-function
rasteriser with a fragment shader interlock** (`GSH_VulkanDrawDesktop.cpp`; Play!'s code was not read for this stage).
One workgroup owns a tile of 16 by 8 pixels and applies its records in drawing order (§4). The reasons [A]:

1. **Read-backs must be the same on every device.** A local-to-host transfer, a CLUT loaded from drawn pixels, and a
   texture read from a render target all return drawn bytes to the machine. The walk uses the software rasteriser's own
   integer arithmetic: edge functions, top-left ties, interpolation floored in its fixed point, and the exact texture
   quotient. Its memory is therefore the reference's, and every device gives the same bytes. Hardware coverage plus
   floating-point attribute interpolation gives neither.
2. **No interlock is needed.** The walk runs on RADV, MoltenVK, lavapipe and any device with compute, storage buffers
   and 64-bit integers. RADV on the RX 6800 does report `fragmentShaderPixelInterlock` [M, `vulkaninfo`, 2026-10-31].
3. **It fits `emusen-gpu` as it stands.** No graphics pipeline, image or render pass is added to a shared surface. It
   is also the form `Sapphire_Plan.md` §5.4 chose for the device path, and the one Diamond's renderer took
   (`Diamond_Enhancements.md` §14 on).

**What it costs against Play!'s form** [A, measured in §6]: binning on the processor (records into tiles, stretches);
a dispatch and a barrier for each run of records that depend on one another across tiles; and the exact arithmetic in
every textured pixel (Play! interpolates in floating point). The rasteriser's own work is done in shader instructions
rather than by fixed-function hardware.

**Departures from the plan's text.** `Sapphire_Plan.md` §5.4 and `Sapphire_Redesign.md` §4.6 have the GS use the
presenter's lent device. Here the GS takes a lent device when one is given (`Gs::set_device_on`). No caller passes one
yet, so the bench and the replay make their own; wiring the core ABI's lend is the presenter's step (§8). §4.6's
unified memory first is not built: the memory is device-local, with host staging for copies in and read-backs, which is
§4.6's discrete case. §7.3's streams are built (§3).

## 2. The memory, and what the processor reads

The processor keeps a mirror of the 4 MiB for its own reads: a CLUT's load, a local-to-host or local-to-local transfer,
the PCRTC's lines, a state and a debugger. Each 8 KiB page is in one of three states:

- **clean**;
- **newer on the processor**: written by a transfer or a primitive drawn on the processor, and copied in, whole pages
  from host staging, at that point of the order before the next record;
- **newer on the device**: written by a record (by lane, the low 24 bits and the high 8, as the workers count them,
  `Sapphire_Native.md` §50), and read back, whole pages, before the processor reads it.

A page is never both. Before the processor writes a page with either lane newer on the device, everything recorded is
submitted and the drawn pages are read back (a *sync*). A host-to-local transfer **claims** its destination's pages at
TRXDIR: one check, not a check per pixel. The claim is kept, and the pages marked again after every copy-in, until the
transfer ends, so that pixels arriving after a V-sync or a drawing still reach the device. The bug this guards against
was found on the arena stream (§7). A CLUT's load settles the rectangle it reads once, then reads the mirror through
the page table (`Clut::load_local`). What is recorded is submitted without waiting at each V-sync, so the device draws
while the machine runs; the first PCRTC line of a field then waits for it.

## 3. GS streams (`Sapphire_Redesign.md` §7.3)

`gs::stream::Recorder` keeps the GS's state at a start (`emusen_native::State`). After it come the general registers'
writes, the host interface's reads with the values the machine got, the privileged writes and the V-syncs, one tag
byte each, under the magic `GSS1`. The bench writes one with `--gs-stream FILE` from `--from` to the end.
`examples/gsstream.rs` replays one into a GS made fresh, in software, on workers, on the device (`--device`) or
planning only (`--plan`). For each field it prints the digest of the picture the display circuit shows, read as the
PCRTC reads it, and with `--memory` the digest of the whole memory. It checks every recorded read. The corpus is kept
with the discs, outside the repository: the arena window, frames 9060 to 9360, is
`~/.cache/emusen/gs-streams/arena.gss`, 1,192,119,944 bytes [M, 2026-10-31].

## 4. The plan and the walk

**Records** (`device/plan.rs`): 32 words for each primitive: kind, environment index, CLUT slot, snapshot flag, then
each vertex's window coordinates, Z, RGBA, Q, S, T, UV and F. **Environments** are 32 words, recorded again only when
they differ from the last: frame, Z, scissor, TEST, the attributes and flags, ALPHA, DIMX, FOGCOL, and the raw texture
registers. A record with an indexed texture gets a copy of the CLUT buffer when it changed. A primitive covering
nothing is passed before its environment is read.

**Stretches.** Records are binned into 16-by-8 tiles. A tile's records keep their drawing order, and a 16-bit word's
two pixels (x and x + 8) fall in one tile. One stretch is one dispatch of the walk. A stretch ends before a record
that would race an earlier one of the stretch across tiles:

- **texels** that meet lanes the stretch has drawn;
- **surfaces**: stores meeting another surface's pixels (a surface is a buffer's format, base and width, under which
  pixel (x, y) is one word);
- **overwrite**: stores meeting the stretch's texels;
- or a copy-in meeting it, a full buffer, or a sync.

A triangle's tiles that one edge excludes entirely are not listed.

**Records that read where they draw** (texels sharing 64-word blocks with their stores, checked by blocks after a page
check) are the named departure. They read their texels from a **snapshot** of the texture's pages, copied on the
device before their stretch. A stretch of them keeps one environment, so a run of one GS draw is one dispatch. The
software rasteriser instead reads in raster order the pixels the same primitive has just drawn. The snapshot reads them
as they stood before the run, deterministically on every device. Jak's arena has about 1,657 such records a field, in
about 2.2 snapshot stretches [M]. **The departure is visible** [M, the arena stream, pictures every 50 fields against the
software rasteriser's]:

- 174 of 300 field pictures differ;
- in the sampled fields, 0 to 6.6 % of pixels differ, 2.47 % over all sampled pixels;
- the largest channel difference is 138.

The differences are in the refraction effect drawn from the frame. This is not within a small tolerance. Whether a GPU
form keeps the reference's order there is the question of §8. `SAPPHIRE_GS_FEEDBACK=exact` (`Gs::set_exact_feedback`) draws them on the processor in
the reference's order instead. That mode is for proof only. It is a processor path and not the player's.

**On the processor only**: a record whose frame and Z are different surfaces sharing pages, and one whose pixels alias
between tiles, x past the buffer's width or a span of more pages than the memory has. Neither occurs on the arena [M].

**The walk** (`shaders/walk.comp`, compiled by `SapphireShaderTests`'s route, checked in as `walk.spv`). Each chunk of
16 records is staged in shared memory with its environment, and a triangle's reciprocal of its area is found once. Each
pixel meets the chunk's records in order: coverage, interpolation, texturing, fog, antialiasing, the alpha, destination
alpha and depth tests, blending, dithering, clamp, the masks. Each step is in `raster.rs`, `pixel.rs` and `texture.rs`'s
integers. The frame's and Z's words stay in a two-entry register cache across records; a 16-bit word's half is stored
by two atomics. **`emusen-gpu` 1.3** enables `shaderInt64` when the device offers it (`GpuDevice::int64`), an addition
to its surface (`EmuSen_Gpu.md` §3).

## 5. The arithmetic, and its stated tolerance

Everything is in integers, so every device computes the same bytes. Divisions by a triangle's area use a reciprocal
found once for each record, corrected exactly. The exact quotient's and the LOD's divisions start from a
single-precision estimate and are corrected exactly by 128-bit integer arithmetic; where the estimate does not settle
in eight corrections, the bit-serial division answers. **Two places differ from the reference, both deterministic:**

1. A sum ΣS·w whose terms would pass 125 bits keeps its lowest bits dropped. The reference keeps 768 bits. This
   happens only for S, T or Q whose exponents differ by more than about 65.
2. The LOD's log2 takes the ratio's significands to 64 bits and squares a 62-bit fraction. The reference uses 256 bits
   or a double estimate checked against 2^-24. The floor can differ only where 2^(L+4)·log2(w/|ΣQw|) lies within about
   2^-50 of a whole number.

Neither changed a byte on the generated streams or on the arena (§7).

## 6. Measurements (2026-10-31, the arena stream, 300 fields, the RX 6800)

**The processor's part** [M, user instructions counted with a perf counter, the replay's own parse (7 M a field) taken
off]:

| Step | Instructions a field |
|---|---|
| first build, primitives drawn back on the processor | 401 M |
| snapshot stretches, a texture's whole reach cached | 281 M |
| CLUT loads through the page table, transfers claimed by page | **219 M** |
| the software rasteriser on one thread, for scale | 4,660 M |

The PCRTC's picture read, paid by every way of drawing, is another 22 M. The gate is 150 M. What is left is spread
thinly: the register writes, the drawing kick and `Environment::read` together about a third; the plan about a tenth;
transfers about a seventh; the CLUT reads; and units.

**The device** [M, the PCRTC's wait a field, in snapshot mode, by measurement switch]:

| Walk | ms a field |
|---|---|
| first build | 35.7 |
| records staged in shared memory, frame and Z cached (no gain alone) | 35.5 |
| divisions by reciprocal and corrected estimate | **27.5** |
| of which: no record applied (staging, read-back, dispatches) | 1.3 |
| of which: coverage, interpolation and the pixel pipeline | 4.9 |
| of which: texturing with the coordinates' arithmetic left out | 3.4 |
| of which: the exact quotients and sums | 11 |
| of which: the LOD a pixel | 7 |

RADV's statistics for the walk [M, `RADV_DEBUG=shaderstats`]: 256 VGPRs, so one wave a SIMD; 1,164 SGPRs spilled; 217 KB
of code. With the exact coordinates compiled out it is still 168 VGPRs and 118 KB. The walk is bound by its register
pressure and code size, not by its form. 44 stretches a field; 90,124 tile entries; 6,024 tiles; 1,153 environments;
1,726 CLUT slots; 179 pages read back a field.

**The gates**, as set on 2026-10-31:

| Gate | Now | Target |
|---|---|---|
| GS CPU work a field | 219 M instructions | 150 M |
| GPU a field, RX 6800 | 27.5 ms | about 0.9 ms (the 780M taken as 13 times slower) |
| GPU a field, handheld | about 360 ms, **estimated** from the RX 6800 by 13, not measured | 11 ms |

The emulation thread's own GS milliseconds are not separated yet: the replay's 13.3 ms a field in plan mode includes
the replay itself.

## 7. Tests and seeded faults

`tests/gs_device.rs`, on the RX 6800 [M]:

- generated streams, 16 seeds of 700 steps, every primitive, environment, texture (from the frame's own pages too), CLUT
  load and transfer, with exact feedback: the memory and every read-back equal to the software rasteriser's at every
  check;
- a state saved mid-stream, loaded into a device GS and continued, equal;
- with snapshots, two runs equal;
- a transfer across a V-sync and a drawing reaching the device whole.

The generator moved from `threads.rs` into `tests/gsgen/`, shared by both.

**The whole machine** [M, the bench's arena window, frames 9060 to 9360, `--defaults --gs-device`]. With exact feedback,
the picture, sound and state digests (F0279E81EE5EB699, BCA8F00F4743A955, 2EF5C6B11B77E8DC) equal the software
workers' run, so everything the machine read back was the same. With snapshots, the sound and the final state are
equal, and the pictures differ as §4 gives. Arena stream [M]: with exact feedback all
300 pictures equal the software rasteriser's (digest 5F046C5478DA64A2), and two runs are equal. Two defects were found
this way and fixed before the first commit. One was a page with its other lane drawn, copied in over the device's
pixels, caught by the generated streams' seed 5. The other was a transfer's pixels after a V-sync never copied in,
caught by the arena stream at field 85.

Seeded faults, each reverted:

| Fault | Caught by |
|---|---|
| a transfer's claimed pages not marked again after a copy-in | the transfer across a V-sync (first survived a version where a drawing in between, and then a later iteration, masked it; the test was sharpened) |
| a claim reading back only the lanes it writes | the generated streams and the state test |
| no stretch ended where a record reads what the stretch drew | a record reading what an earlier one drew in other tiles (first survived the generated streams, whose readers ran after their sources by luck of scheduling; the test gives the source tiles 64 records and the reader one) |
| a 16-bit word stored whole from the register cache | the generated streams and the state test |
| the reciprocal division left uncorrected | the generated streams and the state test |

## 8. What is left, in order

0. **An exact GPU form for records that read where they draw** (§4). The options, for the tester:
   - (a) the snapshot of each run, as built (2.5 % of the arena's sampled pixels differ);
   - (b) each such record after the one before, on one workgroup, its pixels reading the memory as it stood before
     that record. This is cheap on the arena's records of one to four pixels, though a barrier for each record costs
     milliseconds a field, and it differs from the reference only where a record reads a pixel it drew itself;
   - (c) the reference's raster order within each record too, which is exact and dearer still.
1. **The walk's register pressure** (§6): the per-record setup decoded once for each workgroup into shared memory (edge
   equations, the attributes' planes, the sampler's fields), and walks for each class of record (untextured, UV, STQ
   with and without a LOD a pixel) **generated from one source**, not copied by hand. The targets are the STQ quotients'
   11 ms and the LOD's 7 ms.
2. **The processor's part to 150 M**: the drawing kick's environment and the plan's units.
3. **The lend and the picture on the device**: the core ABI's lent device passed in by v1's create, and the picture
   handed through `PictureOut` rather than read back for the PCRTC.
4. **A setting for the player**, once the gates are met; until then the device path is reached only by
   `SAPPHIRE_GS_DEVICE`, the bench's `--gs-device` and the replay.
5. **The handheld's measurement**, once access is back.

## 9. What was read for this stage

The software GS (`src/gs/`), `EmuSen_Gpu.md` §1 to §6, Diamond's renderer's layout and the start of its walk (the
project's own code), `Sapphire_Plan.md` §5.4, `Sapphire_Redesign.md` §4.6 and §7.3, and the 2026-10-31 hardware-counter
data. No other emulator's source was read for this stage.
