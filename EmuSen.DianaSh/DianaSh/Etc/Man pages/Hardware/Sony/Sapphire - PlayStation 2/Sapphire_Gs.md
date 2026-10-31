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

## 10. Second step (2026-10-31): single-precision coordinates, walks by class, ordered records

*Decided by the tester 2026-10-31:* on the device, STQ and the LOD are deterministic by specification rather than
bit-equal to the software GS. Coverage, Z, colour and every byte read back stay exact; exact mode is kept for proof.
Self-reading records are applied record by record on one workgroup, option (b) of §8.

### 10.1 Lane utilisation [M, the arena stream, measurement switch 32]

| Measure | A field |
|---|---|
| lane-record evaluations | 11.54 M |
| pixels covered | 1.98 M (17.2 %) |
| pixels textured | 1.79 M |

This is above the 10 % at which a change of form was to be reported. The cost is divergence: the textured path runs
with about one lane in six live.

### 10.2 Single-precision coordinates (§5.4)

The walk sets round-towards-zero and flush-to-zero for 32-bit floats (`SPV_KHR_float_controls` execution modes) and
marks every product and sum `precise`, so no fused multiply-add is formed. S, T and Q are read as `exact::Float` reads
them. The sums ΣS·w and ΣQ·w are formed in the record's order; the quotient takes 1/ΣQ·w from an integer seed and three
Newton steps; log2 is the exponent plus a polynomial of degree 6 (error below 2.5e-6). There is no division and no
transcendental, so every conformant device gives the same bits. The UV path stays in integers. `emusen-gpu` 1.4
enables `VK_KHR_shader_float_controls` when it offers both modes (`GpuDevice::float_controls`); without them the exact
walk draws.

**The picture** [M, pictures every 50 fields against the software GS, with self-reading records in the exact order]:
fields 200 and 250 differ in 35 and 27 of 229,376 pixels, largest channel difference 2. That is invisible. Exact mode
(`SAPPHIRE_GS_EXACT`, or `SAPPHIRE_GS_EXACT_TEXTURE` for the coordinates alone) remains bit-equal on all 300 fields.

### 10.3 Walks by class (§5.5)

One source, specialised by constants (`emusen-gpu` 1.4, `create_program_specialised`). The constants are exact
coordinates, triangles, sprites, lines or points, texture, STQ, a LOD a pixel, fog and antialiasing. A stretch's class
is the union of its records' (`plan::class`). Each class's walk is made the first time a stretch of it is submitted;
the walk with every feature is made at the start. Compilation is synchronous for now. Building the general walk as the
fallback while a class compiles, and caching compiled forms, are owed.

[M] Nine classes on the arena. Eight still use 256 VGPRs and 92 to 102 KB of code. The pressure lies in what every
textured class shares, not in the features taken out.

### 10.4 Ordered records, option (b)

A run of self-reading records is one stretch dispatched as one workgroup. Each record carries its bounds; the workgroup
covers them 128 pixels at a time in raster order. Every lane reads, then all store, so a pixel reads the memory as the
earlier records and the earlier batches of its own record left it. The snapshot is gone.

[M] It does not reach the software picture: fields 100 and 150 still differ in 3.9 % and 5.4 % of pixels, largest
channel difference 122. That is the same order as the snapshot's. Jak's refraction primitives read pixels they have
drawn themselves earlier in raster order, which only option (c) keeps. It is also slow: the RX 6800's field went from
10.3 ms to about 108 ms. Each of about 1,657 records a field waits on its own chain of memory and barriers in one
workgroup. Reported to the tester before going on.

### 10.5 Numbers after this step [M, 2026-10-31]

| Measure | Value |
|---|---|
| processor's part (plan) | 220 M instructions and 63.6 M cycles a field, the replay's parse taken off |
| GPU, RX 6800, single-precision coordinates, walks by class | 10.3 ms a field |
| GPU, RX 6800, with ordered records | about 108 ms a field |
| exact mode | all 300 arena pictures equal the software GS's |

Seeded faults for this step:

- A stretch's class missing the texture bit was caught by the generated streams, the state test and the transfer test.
- The reciprocal with one Newton step first survived, because every test compared in exact mode. It was caught after
  `single_precision_coordinates_leave_nearly_every_pixel_as_the_reference` was added.
- An ordered record's lanes storing before every lane had read **survived**. It is a race between the waves of one
  workgroup, which no test here provokes reliably.
