# Mars: the GPU under the picture at a multiple

*Begun 2026-09-21. A record, where `Mars_GpuPlan.md` is the plan: each phase of that page is retired into this one as
it is carried out, with what was measured and what was not. Read the plan's §0 first: this is for the picture at a
multiple and nothing else, and the software rasteriser stays the machine's.*

## 1. The dependency, and how little of it there is

The core takes one package, `Silk.NET.Vulkan` 2.23.0, which is a managed binding and nothing more: 3.3 MB of
assembly in a publish of 117 MB, and no native code of the project's or the package's. The Vulkan loader and the
driver are the system's. Their absence is not an error. `GpuDevice.TryCreate` returns null with a sentence saying
why, and the caller's answer to null is the CPU path of `Mars_Rdp.md` §11, which remains complete.

`EmuSen.csproj` gained `AllowUnsafeBlocks`, because Vulkan's binding is pointers throughout. Everything unsafe is in
`Rdp/Gpu/GpuDevice.cs`. The rest of the core is as it was and should stay so; this is a permission one file uses,
not a change of style.

**What the layer is.** `GpuDevice` is one compute queue, one command buffer and one fence. `GpuBuffer` is a storage
buffer in one of two kinds of memory: the host's, mapped for its whole life and read through `Span<T>`, or the
device's own, reached only by a copy. The host kind prefers cached memory, because the readback is the path that
matters and reading write-combined memory from the CPU crawls. `GpuProgram` is a compute shader with storage
buffers at bindings 0 to n−1 of set 0 and one block of push constants. `Submit` records copies and dispatches,
submits them, and **waits**. Nothing is in flight when it returns. That is a deliberate simplification: it is what
lets a program's one descriptor set be rebound before every dispatch, and it puts a barrier after every command
rather than reasoning about which need one. A submission here holds a handful of commands, not thousands. If phase 5
finds the wait on the emulation thread, the answer is the deferred presentation of `Mars_Video.md` §2.7, which
already moves the scan-out off it, and not an asynchronous layer built before anything needs it. *(§14 records the
measurement that did need it, and the one submission left pending that it added.)*

**Choosing a device.** Discrete before integrated before virtual before software, by the loader's own type field.
`EMUSEN_MARS_GPU_DEVICE=<part of a name>` overrides, which is how a test names llvmpipe on a machine that has
something better, and how a player with two adapters picks.

## 2. Shaders: GLSL in the tree, SPIR-V in the product, a test between them

Each shader is a `.comp` file in `Rdp/Gpu/Shaders/` with its compiled `.spv` beside it, and the `.spv` is what the
assembly embeds. The compiler, shaderc through `Silk.NET.Shaderc`, is a dependency of `EmuSen.WiseMan` alone, so
that no player's machine carries a compiler and no build needs one installed. This machine has none installed and
no root to install one, which is how the arrangement was arrived at; it would have been the right one anyway.

`MarsGpuDeviceTests.Every_compiled_shader_is_the_compilation_of_its_source` recompiles every source and requires
three byte sequences to be equal: the fresh compilation, the file on disk, and the resource embedded in the core. A
source edited without recompiling fails the second; a recompilation without a rebuild fails the third. Running the
test once with `EMUSEN_GPU_SHADERS_RECORD=1` writes the files.

**What this leans on** is that one version of shaderc compiles one source to the same bytes every time, which held
across every run here. A change of the package's version will change the bytes and fail the test for every shader at
once; that is the test working, and the remedy is to record again.

## 3. Phase 0: the spike (2026-09-21)

**The exit was** a shader that adds two buffers, read back in a WiseMan test, on the software device and a real
one, with the packaging and start-up costs written down.

`add.comp` computes `c[i] = a[i] + b[i] + bias` over 100,003 signed words of random values, so that most sums
overflow: the pixel pipeline is 32-bit integer arithmetic that wraps, and the first thing worth knowing of a device
is that its integers wrap as C#'s do. The count is not a multiple of the group size of 64, so the bound check in
the shader is exercised. The result is written to **device** memory and copied to a host buffer, which is the route
a frame image will take. Every element is compared against `unchecked` C#.

| Device | Create | Pipeline | Dispatch, copy and readback of 400 kB |
|---|---|---|---|
| AMD Radeon RX 6800 (RADV), discrete | 28–29 ms | 0.1 ms warm, 4.6 ms cold | 0.30–0.34 ms |
| AMD Raphael (RADV), integrated | 28–30 ms | 0.1–0.6 ms | 0.54 ms |
| llvmpipe, software | 48–50 ms | 1.6–1.7 ms | 2.8–3.0 ms |

Three runs each; the first run of the session is in the cold figures. All three devices agree with the CPU on every
element. A machine with no device reports that and passes, since that machine is the CPU path's.

**Which device the tests use.** The table above was taken with every test run as a theory over all three devices.
Since the same day they run on the machine's first choice only, here the RX 6800, because the suite's time is the
user's and llvmpipe at four is slow; `EMUSEN_MARS_GPU_TEST_DEVICES=all` brings the other two back for an occasional
cross-check, which is worth doing when a phase of the shader port lands and is recorded where it was done. This
retires the plan's §4 sentence that tests run on lavapipe: they can, and by default they do not. The integrated
adapter has a second use. With two RDNA2 compute units against a Steam Deck's eight it is the machine's stand-in for
a low-end device, so speed at a multiple is to be measured on it as well as on the discrete card
(`EMUSEN_MARS_GPU_DEVICE=RAPHAEL`), and a multiple that holds there is a conservative answer for the Deck.

**What these numbers are and are not.** Thirty milliseconds to create a device is paid once, at the moment the
setting is turned on, and is nothing. The third column bounds the fixed cost of a flush from above: a submission
and a wait, with a trivial kernel, costs a third of a millisecond on the discrete card. The plan's §2 guessed a
readback of four megabytes at "under a millisecond or two"; ten times this test's size on the same route is
consistent with that and is not yet a measurement of it. None of this says anything about whether shading is fast.
That is phase 2's question and the plan's stop-or-go point.

**Also held by tests:** a name no device has is refused with a reason and no exception; a dispatch given the wrong
number of buffers or the wrong size of push constants is refused before the device sees it, and the device is
usable afterwards.

## 4. Metal, by way of MoltenVK

*Asked for on 2026-09-21, while phase 0 was being written.* A Mac has no Vulkan driver. It has Metal, and MoltenVK,
which is a Vulkan implementation on top of Metal that translates SPIR-V to Metal's shading language when a pipeline
is created. The plan's §2 already named it as the route, and it remains the route: the same `GpuDevice`, the same
shaders, one back end.

**Why not a Metal back end of our own.** The shaders are going to be a port of some three thousand lines of pixel
pipeline (the plan's §6), held exact by a bit-for-bit oracle. A native Metal back end is a second port of those
lines in a second shading language, held exact by the same oracle on a machine this project does not have. The
argument that rejected Direct3D 12 and Metal in the plan's table, "two or three back ends for a research project",
has not changed; what MoltenVK costs instead is a translation at pipeline creation and whatever it does not
support.

**What was built for it.** A loader that follows the portability rules hides layered implementations unless the
instance asks for them, and on a Mac every device is layered. `GpuDevice` therefore enables
`VK_KHR_portability_enumeration`, with its instance flag, when the loader offers it, and enables
`VK_KHR_portability_subset` on a device that offers it, which the specification requires of any application that
uses such a device. On this machine the loader offers the first, so the instance branch runs in every phase 0 test and
is known to be harmless with drivers that are not layered; no device here offers the second, so the device branch
has never executed anywhere.

**What was not done, and is not claimed.** None of it has run on a Mac. The shaders so far use nothing outside the
portability subset, since integer storage buffers and push constants are its core, and the rule for the port is to
keep it so: no 64-bit integers in shader code, no subgroup operations, no features beyond Vulkan 1.1. Shipping it
means publishing `Silk.NET.MoltenVK.Native` with the macOS builds, which is a publish decision for whoever has the
machine to check it on. Until someone does, "Metal support" means that nothing in the design stands in its way, and
no more than that.

## 5. Phase 1: memory, rows, tiles, and the fill cycle (2026-09-21)

**The exit was** fill scenes identical to the CPU path at the multiple. It is met at two, three and four, byte for
byte over the whole of the memory and its hidden bits, on all three devices.

### 5.1 The memory is a mirror, not a set of images

The plan's §3 kept "frame images on the device per colour-image address". What was built instead is one device
buffer mirroring the **whole** memory at the multiple, in the console's sixteen-bit words: one `uint` a word, the
word in the low half and its two hidden bits above. The reasons are the console's habits. Every game clears its
depth image by making it the colour image and filling it; images overlap and alias; a pixel's address is arithmetic
on a base, not an index into an object. A mirror gives all of that the CPU path's meaning with no bookkeeping, and
makes the oracle's comparison a comparison of two memories. The cost is size: 64 MB of device memory at two and 256
MB at four for an 8 MB machine. `GpuRasteriser.TryCreate` measures that against the device's
`maxStorageBufferRange` and returns null with the reason when it does not fit, which is the CPU path's case like any
other.

**Why a word and not a byte.** GLSL has no byte arrays, and two invocations writing different bytes of one `uint`
is a race. A sixteen-bit pixel is one word and a thirty-two-bit pixel two, so an invocation per pixel owns what it
writes. An **eight-bit image** puts two pixels in a word. Those rows are not shaded and are counted
(`RowsOfUnsupportedImages`); the fix is an invocation per word for such images, and it waits for a game that needs
it at a multiple.

### 5.2 Host: rows, then tiles

`Rdp.ShadeOn` turns a processor at the multiple into a recorder. `Draw` reaches one new line, and instead of
shading, the walker's spans go to the rasteriser: a primitive record of eight words (so far the kind and the fill
colour) and a row record of four (primitive, row, left, right). A change of colour image ends the batch.

At a flush the rows are binned into tiles of 8×8 pixels (64×1 since §14.1, which says why) over the batch's bounding box, by counting, summing and
placing, so that each tile's list is in the order the rows were drawn. The four arrays are copied to device memory in
the same submission that dispatches, because a discrete card reading its inputs across the bus for every pixel is
the slow way to be correct.

### 5.3 Device: one invocation a pixel, in order

`shade.comp` runs one invocation per pixel, a work group per tile. It loads its one or two words, walks its tile's
list, applies every row that is its row and covers its column, and stores once at the end. The fill is
`Rdp.FillPixel` restated on words: a word takes the half of the fill colour its address holds, and its hidden bits
are three times its low bit. Words past the end of memory are dropped, as the CPU path drops bytes.

### 5.4 Where it departs from the CPU path, on purpose

**A span past the image's width is clipped and counted** (`ColumnsPastTheWidth`). On the CPU path, and on the
console, such a span runs on into the next row's first bytes, because a pixel is an address. A per-pixel invocation
cannot own a pixel that two coordinates reach. The exact answer is an invocation per address that considers both
coordinates and merges them in drawing order; it is not built, and the counter is there so that a game which leans
on the wrap is seen rather than suspected. The oracle's claim is therefore conditional, and the test asserts the
condition: identical **where the counter is zero**.

**Copy mode moved to phase 3.** The plan put "the copy mode's rectangles" here. Copy mode fetches texels, and texels
are phase 3. Until then every primitive that is not a fill is counted (`PrimitivesNotShaded`) and not drawn.

### 5.5 Coverage

`MarsGpuRasteriserTests`: per list, an edge-to-edge fill and then sixty random overlapping rectangles into each of
three image bindings, sixteen-bit and thirty-two-bit, at quarter-pixel positions under scissors that fall inside
pixels, three seeds, three multiples. Fill colours have differing halves and low bits, so the word lanes and hidden
bits are exercised. Three mutants:

| Mutant | Result |
|---|---|
| Tile lists placed in reverse order | caught, all nine |
| Hidden bits taken from the wrong bit, in the shader | caught, all nine |
| Host clips one column short of the width | **survived at first**: no scene reached the image's last column, since every scissor stopped short of it. The edge-to-edge fill was added for this, and it is now caught |

The surviving mutant is the useful result of the three: a scene generator that draws "randomly" had a systematic
hole at exactly the place an off-by-one lives.

**Not covered:** speed, of which nothing here is a measurement; the batch is flushed and waited for at every image
change, which is phase 5's to reconsider. Rows above the image or left of it. Eight-bit images. A memory larger than
the device binds, which is refused by a path no test reaches on this machine's devices at these sizes.

## 6. Phase 2: the one-cycle mode, untextured, and the first number (2026-09-21)

**The exit was** shaded scenes identical to the CPU path at the multiple, and a first speed number on a shade-heavy
scene, which the plan named as the point to stop if the device was not several times faster than four CPU workers.
The scenes are identical. The number says go on the discrete card and is an honest maybe on the integrated one.

### 6.1 What was ported

`shade.comp` now restates, stage for stage and under the C# method's name in a comment, `RowCoverage` for one
column, the coverage offsets, `CorrectShade` and `CorrectDepth`, the dither patterns, `CombineSecondCycle` with its
selectors and the chroma key, `ReadMemory`, `CompareDepth` with its four modes and the blend shifts, `Blend` and
`BlendEquation`, `FinalCoverage`, `WriteMemory` for sixteen and thirty-two bits, and `StoreDepth` with the depth
encoding. Two tables the C# builds once are computed where they are used instead: the coverage offsets, and the
blender's quotients, whose bit-serial divider is eight iterations a channel and costs less than a binding.

On the host, `Rdp.RecordOneCycle` is `DrawOneCycle`'s setup written down instead of run. A primitive's record is 64
words: flags, the packed modes, combiner and blender selectors, seven colours, the constants, the eight steps already
signed by the direction of the span, and the corrections. A row's is 24: its ends, its eight values **at its first
pixel**, stepped there from the major edge on the host exactly as the loop does, and the four sub-scanline edges
coverage reads. An invocation therefore computes a pixel's values as the row's plus the step times its distance from
the first pixel, which is the loop's repeated addition in closed form and equal to it because both wrap.

The depth word is carried through the tile's list in a local, like the colour words, and a change of depth image
ends the batch for the same reason a change of colour image does.

### 6.2 What is kept back from the device, and why it is a carry

Reading the C# for the port found something the plan's §3 had attributed only to the two-cycle mode: **in the
one-cycle mode the combiner's COMBINED input is the previous pixel's result** (`_combined`, `Rdp.OneCycle.cs`). It
is a carry between neighbouring pixels, which an invocation per pixel cannot see. So is a texel, until phase 3, and
the level-of-detail fraction, which is measured from texture coordinates. `TheDeviceShadesThisCombiner` keeps back
any primitive whose selectors read one of them, and it is counted (`PrimitivesNotShaded`), not drawn. Zero constants
are selectors 7, 15 and 31 and not 0, so a plain shaded primitive is not caught by this. The carries are phase 4's,
by one of the two routes the plan's §3 names.

### 6.3 The counter of §5.4 was wrong, and the oracle said so

The first run of the new test failed, and not on a byte: every byte matched at two, three and four. It failed on
the guard, thousands of `ColumnsPastTheWidth` under a scissor exactly as wide as the image. The walker's spans
include the scissor's own column, where the clipped edge leaves no sample covered; the CPU path reads that pixel and
never writes it, which is the "read by the next row's owner" of `Mars_Rdp.md` §2.8. The counter now counts what it
was meant to: columns past the width **that the CPU path would have written**, found by running the CPU's own
`RowCoverage` over them on the host. For a fill that is every column, as before.

### 6.4 Coverage, and two mutants that taught something

`Shaded_depth_tested_triangles_on_the_device_are_the_cpus_byte_for_byte`: the depth image cleared by filling it as a
colour image, the picture cleared, then 72 triangles over and past the image, each under its own random modes (both
dithers, the key, the first blend's four selectors and the low fifteen mode bits whole), its own combiner from the
inputs the device has, and its own seven colours and constants. Five lists, sixteen-bit and thirty-two-bit, at two,
three and four.

Sixteen mutants, one a ported stage, each recompiled to SPIR-V and run. Fourteen were caught at once: the dither
compare, the depth margin, the blend divisor, the coverage offset's column, the shade correction's shift, the final
coverage, the depth encoding, the stored coverage's hidden bits, coverage's right edge, the blend shifts swapped,
full alpha's promotion to 0x100, the depth mode's scaled coverage, and the thirty-two-bit hidden bits.

**Two in the chroma key survived, twice.** The reason is arithmetic and is worth keeping. The key's alpha varies
only while a channel is within 256 of the key's width times sixteen, out of 131,072 values, and the alpha's low
bits reach the picture through one thing only, the blender's test for an opaque pixel under one pair of selectors.
Random modes do not line those up: making the widths narrow did nothing, and keying a third of the triangles did
nothing. What caught both was two lists aimed at it, where shade times a scalar sweeps the window and a forced
blend weighted by the pixel's alpha carries the key's alpha into the colour. The lesson is §5.5's again in another
place: a generator that is random over the *inputs* can be nearly empty over the *behaviours*, and a mutant is how
one finds out.

### 6.5 The number

Release build, medians of nine, interleaved, one list drawn at the multiple and the picture read back. "A quarter"
is the single CPU worker's time divided by four: the bound four workers cannot beat, and better than they do.

| Scene | Multiple | CPU, one worker | A quarter of it | RX 6800 | of which host | Raphael (integrated) | of which host |
|---|---|---|---|---|---|---|---|
| 72 large triangles, heavy overdraw | 2 | 152 ms | 38 | **5.4** | 2.1 | 29.1 | 3.8 |
| 2,000 small triangles | 2 | 73 | 18 | **9.2** | 5.7 | 24.4 | 7.4 |
| 72 large | 4 | 579 | 145 | **14.3** | 3.3 | 96.5 | 5.9 |
| 2,000 small | 4 | 247 | 62 | **20.6** | 12.3 | 61.0 | 13.5 |

**On the discrete card the plan's condition is met**: seven and ten times the ideal four workers where shading
dominates, and two and three times where it does not. **Where it does not, the device is not the cost.** With many
small triangles, more than half of the device path's time is the host walking at the multiple and writing 24 words a
row, and that half is the same on either adapter. That is the next thing to make cheaper, and it is ordinary C#.

**On the integrated adapter it is parity, not a win**: from 1.5 times the ideal four workers to 0.7. Two things
keep it from being a no. The comparison flatters the CPU, which does not scale by four. And the device's time is not
the emulation thread's, nor four cores': `Mars_Performance.md` §37 found the machine bound by that thread, and cores
given back to it are worth something a table of rasteriser times does not show. Whether that is enough on a
handheld is phase 6's question and needs games, which need textures. **Decision: go on to phase 3.**

**What this is not.** Not a game. Synthetic lists with no textures, one flush a list, the picture read back once.
Real lists change images and modes more often, and every change of image is a flush and a wait here.

**Not covered:** the carries of §6.2; eight-bit images; a depth image that aliases another pixel's colour word,
which the invocation's two locals would get wrong and which no scene draws; the noise dithers, which the CPU path
does not build either.

## 7. Phase 3, part one: point-sampled textures (2026-09-21)

The plan's phase 3 is textures, and it is large enough to land in parts. This part is the texel a coordinate names:
the perspective division, the tile's shift, clamp, mask and mirror, and the fetch for every format and size. The four
texels of a filtered sample and the palette are `Rdp.Filter.cs`, and the level of detail is `Rdp.Lod.cs`; both wait
for the next part. Primitives that ask for them are counted and not drawn, as §6.2's are.

### 7.1 What a primitive carries, and what a ring carries

A texel needs the processor's four kilobytes of texture memory and its eight tile descriptors. Both change far less
often than a primitive is drawn, so neither belongs in a primitive's record. They go into two rings of their own, and
a primitive holds an index into each. The host pushes a snapshot only when the thing has changed: `Rdp` sets a flag
in `WriteTextureWord` and in the two tile commands, and clears it when the rasteriser has taken a copy. A batch that
loads no texture therefore uploads one snapshot, not one per primitive.

Texture memory is uploaded as its 2,048 sixteen-bit words, which is how `Rdp.TextureWord` already addresses it; a
byte index is halved in the shader as the C# halves it. A tile is four words: format, size, line, memory and palette
in the first, the clamp, mirror, mask and shift fields in the second, and its four quarter-texel bounds in the last
two. The primitive record grew from 64 words to 72 to hold the texture flags, the two ring indices, the base tile,
the two level bounds and the four conversion constants. A row's grew from 24 to 32, for the next row's first three
texture coordinates and three span flags, which is what the last pixel of a long span needs (§7.3).

**The loads themselves stay on the host**, unported. `Rdp.Load` walks rows of the machine's memory into texture
memory in the console's byte order, with its own swizzling; it runs once per load and not once per pixel, and it
reads the machine's own memory, which the device does not have. Both processors run it, as they run every command.

### 7.2 The reciprocal table is the host's

`Rdp.BuildDivideTable` rounds in floating point (`Math.Round(0x100000 / (64.0 + segment))`). Putting that in a
shader would be a second implementation of a rounding rule, held equal only by luck. The table is built once on the
host, exactly as the C# builds it, and uploaded to a device buffer at creation: 32,768 words, 128 KB, written once
and read by every dispatch. It is the only input that is not per-batch.

### 7.3 The level of detail is not a carry, and neither is the next pixel's texel

`Rdp.DrawOneCycle` carries a `levelReady` flag: the level measured for a pixel's texel 1 is reused by the next pixel
instead of being measured again. Read as a carry that would have put the whole of the level of detail out of an
invocation's reach. It is not one. The arguments the next pixel would pass are the arguments the previous pixel
passed for its texel 1, term for term, so the flag is an optimisation and the level at a pixel is a function of that
pixel alone. The same holds for texel 1's coordinates, which are the next pixel's, except at the last pixel of a long
span, where they are the next drawn row's first — and the host knows that row and now writes its three coordinates
into the record. So the only true carry in the one-cycle mode remains the combiner's COMBINED input (§6.2).

### 7.4 Coverage, and four scene gaps the mutants found

`Point_sampled_textures_on_the_device_are_the_cpus_byte_for_byte` draws, for each of the twenty format and size
pairs three times over, a tile loaded from a sixteen-bit image and then read under that format, with random line,
memory offset, palette, mask, shift, clamp and mirror, and a triangle over it whose combiner reads texel 0 and
texel 1. Four lists: one without perspective, two with the divider in its ordinary range, one with it at its edges.

The first run of that test passed at every multiple, and **eleven of twelve mutants of the new code survived it**.
The test was very nearly worthless, and four separate faults of the scene had to be found and fixed, each by
measurement rather than by reading:

1. **The texture source address was 0x400000**, which is the first byte past a four megabyte machine. Every load
   read zero, so texture memory was empty and every texel was the same. Found by writing different bytes at the
   source and counting how many bytes of the picture changed: none.
2. **Every triangle used tile zero**, because the tile index lives in bits 48 to 50 of a triangle's first word and
   nothing put it there, while the loads used a tile chosen at random.
3. **No tile had a memory offset**, so a mutant that dropped the offset from the fetch's address changed nothing.
4. **The perspective divider was never exercised**, and this took three attempts. A census of the shifts and the
   overflow flags, taken by instrumenting the CPU, showed every coordinate saturating. The cause was arithmetic the
   scene generator had backwards: the divider answers s over w, so a coordinate inside its range needs **s below w**,
   and the generator made s a multiple of w. Correcting that gave clean results at every shift.

### 7.5 A test that shows the divider, because a scene will not

Even with all four fixed, three mutants of the divider survived. The reason is a property of the hardware worth
recording: **at its coarsest shifts the divider can answer almost nothing**. The result is s times two to the
fifteen over w, so with w of two the only coordinates it can produce are zero and sixteen thousand; with w of one,
only zero. A scene of random triangles reaches those states constantly and can distinguish nothing in them, because
neighbouring inputs give the same answer.

`Every_shift_of_the_perspective_divider_reaches_the_picture` is the answer: 356 triangles, each carrying one
constant coordinate and one constant w, over a tile that neither clamps nor mirrors, with a combiner that is the
texel itself and no depth or alpha test. Each triangle therefore paints the coordinate the divider gave it. The
cases sweep every shift, each with a power of two and a nearby non-power of two, so that the table interpolates
between two reciprocals rather than landing on one, and with coordinates below w, at w and past it, which is where
the divider decides a result is out of range. A w of zero and two negative ones close the sign test. The test also
asserts that the picture holds more than sixteen colours, so that a future change which collapses every coordinate
to one texel fails rather than passes quietly.

With it, three of the four survivors are caught: the shift's special case at fourteen, the sign test's boundary at
zero, and a reciprocal off by one.

### 7.6 One equivalent mutant, and why it is equivalent

Widening the divider's range test by one bit — `(1 << 29) >> shift` to `(1 << 28) >> shift` — changes nothing, and
this is provable rather than merely unobserved. The range's lowest bit is the bit that becomes bit 16 of the
seventeen-bit result, so the original declares overflow when bits 16 and above are not all equal. The wider range
declares it when bits 15 and above are not, which is exactly the case `ClampCoordinate` saturates on without any
flag: bit 15 set and bit 16 clear gives 0x7FFF, the reverse gives 0x8000. The mutant's flags give the same two
values, choosing between them by bit 29 of the product, and bit 29 is the product's sign extension here because the
reciprocal never exceeds 2^14 and the coordinate never exceeds 2^15, so the product never reaches 2^29. The two
paths therefore agree for every input the machine can present. Recorded rather than chased.

### 7.7 What this part does not cover

- **Filtering and the palette.** `SampleFour` and `PaletteEnabled` primitives are kept back and counted. That is
  most textured drawing in a real game, so the counter will be large until the next part.
- **The level of detail.** Argued above to be per-pixel, but not written; `LodEnabled` primitives are kept back, and
  so is a combiner reading the level fraction.
- **Copy mode**, still, for the reason §5.4 gave.
- **The loads**, which run on the host and are not the device's business.
- **A texture image of four bits**, whose loads the C# already declines.
- **Speed.** Nothing here was measured. The texture memory ring is eight kilobytes a snapshot, and a batch of a
  real game's frame may hold many; whether that upload matters is phase 5's question, not this one's.

## 8. Phase 3, part two: filtering, the palette and the level of detail (2026-09-21)

With this the one-cycle mode is whole. `Rdp.Filter.cs` and `Rdp.Lod.cs` are ported, and the only primitive the device
still declines is one whose combiner reads its own previous result (§6.2). `TheDeviceShadesThisPrimitive` is now that
single test.

### 8.1 What was ported

**Four texels.** `FourTexels` with `Texels`, `PaletteTexels`, `NearestPaletteTexels`, `Wrapped`, `Interpolated`,
`PaletteIndex` and `PaletteColor`. The second cycle's conversion path is left out, since the one-cycle mode never
asks for it; it belongs with phase 4. The luma crossing between the two chroma triangles is kept, as is the
mid-texel's average and YUV's half-width chroma.

**The level of detail.** `LevelOfDetail`, `LevelSignals`, `PixelLevelOfDetail`, `AfterSpanTile`, `Movement` and
`Saturated`, with `Log2` computed by `findMSB` where the C# tabulates it. §7.3 argued this stage was per-pixel; it
is, and the row record's three next-row coordinates are what the argument needed. The fraction the combiner reads
is now a value an invocation computes, so `CombineColorC == 13` and `CombineAlphaC == 0` no longer keep a primitive
back.

### 8.2 Coverage, and three more corners a scene does not reach

The scene of §7.4 gained the filter and palette mode bits, a palette loaded through a tile of its own, mask values
up to ten, and the level-of-detail bits with a maximum level per triangle. Fourteen mutants of the filter were run
against it and eleven died. The three survivors were the same kind of finding as before, and each needed something
the scene could not produce by chance:

- **The mid texel** wants both fractions at exactly half a texel, which a moving coordinate hits only by accident.
- **The wrap** returns a different step only on the mask's last texel, and only for masks of nine or ten, which the
  scene did not generate.
- **YUV's chroma step** needs the four-texel path on a YUV tile without a palette.

`The_filters_corners_reach_the_picture` is the answer: 180 flat triangles, one per format, size, mask and fraction,
each holding one coordinate over all its pixels, with filtering and the mid texel on and the palette on alternate
cases. It caught all three — but only after one more measured correction. **A tile line of eight hides the wrap**:
the wrap's two answers differ by 768 rows, and at eight words a row that is 49,152 bytes, an exact multiple of the
four-kilobyte address mask, so both land on the same byte. The test now uses lines of eight, nine and ten, and an
odd line makes the difference visible.

Twelve mutants of the level of detail followed, of which ten died at once. `The_level_of_detail_reaches_the_picture`
was built for the other two: 112 flat triangles sweeping fourteen measured movements against all eight maximum
levels, with a combiner whose colour *is* the level's fraction, so that the fraction is the picture. Two more scene
faults had to be corrected in it, both found by looking at what the numbers could be rather than by reading the
code:

- The first version stepped the coordinate by exact powers of two. The fraction is `(lod << 3) >> level`, and when
  the measurement is a power of two that is 0x100 for every level, so the fraction was zero in all 64 cases.
- The measurements only reached 0x1034, and two of the bits that declare a pixel distant sit at 0x2000 and 0x4000.
  The sweep now runs to 0xC600, which also covers the multiples, since a step is divided by the multiple.

### 8.3 One more equivalent mutant, proven

`Saturated` is `(m & 0x7FFF) | ((m & 0x1C000) != 0 ? 0x4000 : 0)`. Narrowing the test to `0x18000` changes nothing:
the bit the test would drop is 0x4000, and `m & 0x7FFF` has already kept it, so the only case the two tests decide
differently is one where the OR contributes a bit that is present either way. Recorded beside §7.6.

### 8.4 What the one-cycle mode still does not cover

- **The combiner's previous result**, the one true carry (§6.2), and the reason a primitive is still counted rather
  than drawn. Phase 4 decides between recomputing the neighbour and drawing those primitives on the CPU.
- **Copy mode** and **two-cycle**, both phase 4's.
- **Eight-bit colour images** (§5.1) and **spans past the image's width** (§5.4), unchanged.
- **Speed.** Still unmeasured for textures. The texture memory ring is the thing to watch: eight kilobytes a
  snapshot, and a game that loads a texture between every primitive would upload one per primitive.

## 9. Phase 3's real exit: three recorded frames (2026-09-21)

The plan's exit for phase 3 is "recorded display lists of the three probe games identical for a frame each". Those
recordings already exist, from `rdprecord` in the speed tooling: one frame each of Super Mario 64, Ocarina of Time
and Wave Race 64, with the machine state from before it. They need a commercial ROM, so the harness that replays
them lives in the probe (`~/.cache/emusen/probe/mars-speed/rdpgpu/`) and not in the suite, as `rdpbench` does.
`rdpgpu <rom> <state> <dplist> [scale]` builds two processors at the multiple exactly as `DpInterface.NewScaled`
does, runs the frame's words through both, and compares the whole of the scaled memory and its hidden bits.

| Frame | Words | 2× | 4× | Declined |
|---|---|---|---|---|
| Super Mario 64, frame 400 | 3,741 | **identical** | **identical** | none |
| Wave Race 64, frame 301 | 15,013 | **identical** | **identical** | 6, all two-cycle |
| Ocarina of Time, frame 302 | 5,186 | differs | differs | 144, all two-cycle |

**The exit is met for two of the three games outright**, and the third fails for one reason only. Wave Race is
identical *despite* six declined primitives, which means those six drew nothing the picture kept.

### 9.1 The carry does not happen, and that redirects phase 4

The reason to break the counter down by cause was to price the plan's §3 choice between recomputing a neighbouring
pixel and drawing the affected primitives on the CPU. The answer is that **the choice does not need making yet**:
across three frames of three games, the number of primitives declined for the combiner's carry is **zero**. Every
declined primitive is in the **two-cycle mode**, which is a stage not yet ported rather than a carry that cannot be.

This retires the assumption under §6.2's closing sentence. The carry is real and still unhandled, but it is not what
stands between the device and these games; the two-cycle mode is. Phase 4 should therefore be read as "port the
two-cycle mode", with the carry as a smaller question inside it, and the plan's two options priced when a game is
found that needs them.

### 9.2 What the frames cost, and where the time actually goes

Medians of one run each, Release, at the two multiples, against a single CPU worker drawing the same frame at the
same multiple:

| Frame | Multiple | CPU, one worker | Host walk and record | Device shading | Readback of the whole memory |
|---|---|---|---|---|---|
| Super Mario 64 | 2× | 183 ms | 12.3 | **1.5** | 21.1 |
| Ocarina of Time | 2× | 259 | 10.6 | **1.5** | 21.2 |
| Wave Race 64 | 2× | 250 | 14.1 | **1.8** | 20.6 |
| Super Mario 64 | 4× | 317 | 12.2 | **3.3** | 77.7 |
| Ocarina of Time | 4× | 642 | 11.5 | **2.0** | 78.9 |
| Wave Race 64 | 4× | 351 | 14.2 | **3.9** | 80.3 |

**The shading is no longer the cost of anything.** One to four milliseconds a frame, against 183 to 642 on a single
CPU worker: two orders of magnitude, and far past the plan's condition. What remains is two things that are not the
rasteriser.

**The host's walk is now the larger half.** Ten to fourteen milliseconds, and it does not change with the multiple,
because the walker's step is divided by the multiple and the span count is what grows. It is the same `Walk` the CPU
path runs, plus writing 72 words a primitive and 32 a row. That is ordinary C# and is where the next work is.

**The readback here is an artefact of the measurement, not of the design.** The harness reads the entire scaled
memory — sixteen megabytes at two, sixty-four at four — because it compares every byte. A frontend reads the
picture, which at two is six hundred kilobytes. The figure is still worth keeping as an upper bound on what a
readback costs: about 1.3 gigabytes a second on this card, so the picture alone would be under half a millisecond.

### 9.3 The integrated adapter, and a prediction retired

§6.5 measured the synthetic scenes on the integrated Radeon and called the result "parity, not a win": from 1.5
times four ideal CPU workers down to 0.7. **Real frames say otherwise, and by a wide margin.**

| Frame | Multiple | CPU, one worker | A quarter of it | Integrated shading | Discrete shading |
|---|---|---|---|---|---|
| Super Mario 64 | 2× | 236 ms | 59 | **9.9** | 1.5 |
| Wave Race 64 | 2× | 212 | 53 | **10.5** | 1.8 |
| Super Mario 64 | 4× | 318 | 79 | **34.9** | 3.3 |
| Wave Race 64 | 4× | 354 | 88 | **35.2** | 3.9 |

Six times the ideal four workers at two, and two and a half at four, on two compute units. Both frames are
byte-identical there as well.

**Why the synthetic scenes were wrong about this.** They were seventy-two large triangles with heavy overdraw,
chosen to exercise the blender, which makes them shader-bound in a way a game's frame is not. A real frame has many
small primitives and far less overdraw per pixel, so the device is not saturated and the weak adapter keeps up. The
lesson is the one §5.5 and §6.4 keep teaching in another form: **a scene built to exercise a stage is not a scene
that predicts its cost**, and the speed question needed the recorded frames as much as the exactness question did.

This retires §6.5's "parity" sentence. What survives of it is the caution that a handheld's answer needs games, and
that is still true: this is one frame, not a frame rate.

### 9.4 What this does not measure

One frame each, one run each, no threading on the CPU side, and the state was
restored before each. The CPU column is a single worker, not the four the CPU path would use; divide by four for a
fair bound and the device is still thirty times faster at shading on the discrete card and six times on the
integrated one.

**Nothing here is a frame rate**, and the three things between this and one are all phase 5's: every batch is
flushed and waited for synchronously, a frame changes its colour image more often than these replays did, and the
picture has to be read back once a frame rather than once a measurement. Nor does any of it touch the machine's own
thread, which `Mars_Performance.md` §37 found to be the bound at one — the device is aimed at the multiple and at
nothing else, exactly as `Mars_GpuPlan.md` §0 said.

### 9.5 Is the host's walk a problem?

It is the next thing to make faster and it is not a reason to doubt the design, because **the CPU path pays it
too**. Both paths run the same `Walk`; the device path then writes 72 words a primitive and 32 a row where the CPU
path shades every pixel. So the device path's own CPU cost, ten to fourteen milliseconds, is measured against the
CPU path's whole one hundred and eighty to six hundred, and the walk is common to both. Removing the shading
removes about ninety-five per cent of what drawing at a multiple costs the processor, and what is left is a walk
that was always there.

## 10. Phase 4: the two-cycle mode (2026-09-21)

With this the device draws every primitive of all three recorded frames, at two and at four, byte for byte, and
declines none of them. §9's table is now three identical rows.

### 10.1 A pipeline, and which of its edges are real carries

`Rdp.DrawTwoCycle` is written as a pipeline: a pixel's first combiner cycle runs **before its predecessor's last
blender cycle**. Read as a list of carries that looks disqualifying, and three of the four are not:

- **The second cycle's COMBINED** is this pixel's own first cycle, not the pixel before. Fine, and now the device's
  `ReadsCombined` test applies to the *first* cycle only in this mode.
- **The first cycle's COMBINED** is the pixel before. Declined, as in the one-cycle mode (§6.2).
- **The last blend's shade alpha is the NEXT pixel's**, because the successor's first cycles have already run by
  then. That is a forward carry, and a forward carry over an affine value is not a carry at all: the invocation
  computes the next pixel's shade alpha and its dither directly. It is the only thing the successor's first cycles
  change that this pixel's last blend reads, which is what makes the whole pipeline collapse into one invocation.
- **The first blend's memory-alpha weighting** uses `_pastShiftA`/`_pastShiftB`, which come from the depth slope
  stored at the **previous** pixel, as it was before that pixel's own store. That is a backward carry through memory
  and it is genuinely serial, so a primitive whose first blend cycle weighs by memory alpha is declined and counted.
  Across the three frames, no primitive does.

Also declined: `ConvertOne`, whose conversion path through the filter is not ported.

### 10.2 Two defects the port had, and how each was found

**The blend shifts came from the wrong cycle.** `CompareDepth` decides whether to compute its shifts from
`twoCycle ? SecondBlendCycle.SecondAlpha : BlendSecondAlpha` — the *last* blend cycle in this mode, the *only* one
in the other. The shader read the first cycle's selector in both. Ocarina's frame differed by 110 bytes of
16,777,216, all of them one step in red and green. Found by bisecting the display list with `WORDS=n` in `rdpgpu`
down to a single triangle at word 1086, then printing the modes in force for it. The 110 bytes are what a shift of
zero instead of a shift of one does to a blend, which is a good illustration of how small a real defect can be and
still be a defect.

**COMBINED is assigned before the key reads it back.** `CombineSecondCycle` writes its own result into `_combined`
and *then*, when keying is on, passes `ColorA(last.ColorA)` through in place of that result. So a key whose first
input is COMBINED passes through **this** cycle's colour, not the previous one's. The shader assigned `combined`
only in the first cycle, so the key read a value a cycle too old. Found by the synthetic scene, not by the games:
none of the three frames pairs a chroma key with a second cycle reading COMBINED.

### 10.3 Coverage

The `Shaded` and `Textured` scenes gained two-cycle lists: the cycle type, both blend cycles' four selectors each,
and `TwoCycleCombine`, which gives the two cycles **separate** selectors — the first drawn from a set that excludes
COMBINED, the second from one that includes it. Eleven mutants of the new code, then two more.

Twelve of the thirteen died at once. The survivor was the handover between the cycles itself — dropping the shift
when the first cycle's result becomes COMBINED — and it survived for the reason §7.4 and §8.2 keep finding: the
scene could not express it. `Combine` gave both cycles the *same* selectors, drawn from a set with no COMBINED in
it, so nothing the first cycle produced was ever read by the second. `TwoCycleCombine` is the fix, and it caught
the mutant and the second defect of §10.2 in the same run.

**Not covered:** the two declined cases above, copy mode, and everything §5.4 and §8.4 already list. And the games
still only vouch for one frame each.

## 11. Phase 5: the device behind the interface, and a game on it (2026-09-21)

**The plan's exit was** Majora's Mask and the three probe games playing at two and four, and the probe at one still
identical. Both are met. Four games — Super Mario 64, Ocarina of Time, Majora's Mask and Wave Race 64 — were run
from boot through the whole machine with the device on and off, and every frame compared was byte for byte the same
at two and at four with nothing declined. The probe at one passed unchanged.

### 11.1 Where the device sits

`DpInterface` owns the device. `Gpu` is a new property beside `Scale`; setting either rebuilds a `GpuDevice` and a
`GpuRasteriser` for the multiple, or neither, with the reason in `GpuReport`. The machine's own processor is never
the device's (the plan's §0): only the processor drawing at the multiple is given one, through `Rdp.ShadeOn`.

**One processor at the multiple, not one a worker.** The CPU path gives every drawing thread a processor at the
multiple, each shading its own rows. The device path has exactly one, because the device is the parallelism. It
rides the **leading worker's** thread, which already sees every word in order, so nothing about the interface's
threading, marks or barriers changed. `Rdp.Follow`, which hands a scaled processor its native one's split, is now a
no-op for a processor on the device, since that one must always draw alone.

**No device is the CPU path.** With no Vulkan, no matching device or a memory too large to bind, `_gpu` stays null
and every line above falls through to what was there: a processor a worker, and `ScaledDrawn` as before. That
fallback is tested only structurally — through the same interface test with the device off — because selecting
"no device" in a test means an environment variable, and GPU tests run in parallel.

**The setting** is `Gpu` in `MarsCore.Settings`, a switch, off by default. The graphics window builds its controls
from the catalogue, so it appears there with no frontend change.

### 11.2 The readback, and the path that skipped it

The device's memory has to reach `_scaledRdram` before the scan-out walks it. The scan-out already knows exactly
which bytes it will read (`ReachScaled`), so `DpInterface.ReadBackScaled(from, count)` joins the drawing and reads
back just that range.

It went first into `Vi.Capture`, the deferred path, and a real game then differed on two frames of three. The
cause: **the scan-out has a second path.** `Vi.Scan` walks the live shadow directly, without a capture, and so
never reached the readback; on that path the shadow held whatever the last deferred frame had left. Found by
running Super Mario 64 through `framedump` with the device on and off and comparing the PPMs, then reading which
branch of `Walk` chose its source. The readback is now in both.

### 11.3 Copy mode

Copy mode was put off in phase 1 because it fetches texels, and by phase 4 it was the last cycle type left. It
turned out to be the first thing a game in play needed: Mario's frames were identical until frame 600, where 245
copy-mode primitives were declined.

At a multiple it reduces neatly. `Rdp.DrawCopyScaled` fetches a pixel's four texels exactly as the machine's copy
does, and writes only the top one or two bytes. For a sixteen-bit image that is **the top half of the fetched
sixty-four bits**, kept whole or not at all by the first texel's low bit. The port carries `CopyTexels`, with its
bank rule, `Replicated`, `CopyPaletteIndex` and the YUV chroma step, in two 32-bit halves, since the shaders keep to
32-bit integers for MoltenVK's sake (§4).

One detail had to be matched rather than reasoned about: **the copy mode at a multiple starts each row from the
major edge's values, not stepped to the first drawn pixel** as the one-cycle mode does. That is how the CPU path is
written, and the device must equal the CPU path, so the host writes the row's values unstepped for this mode.

**Coverage.** `Copy_mode_rectangles_on_the_device_are_the_cpus_byte_for_byte`: texture rectangles, flipped and not,
over every format and size, with lines up to the field's nine bits and coordinates over their whole range for a
third of them. Ten mutants; eight died.

- **The line mask is provably redundant.** `CopyTexels` masks `line × t` to nine bits and then shifts left four and
  masks to thirteen, which keeps nine anyway, and addition distributes over the mask. Removing it changes nothing
  for any input.
- **The bank rule for the second texel is believed equivalent at a multiple, not proven.** It redirects a texel to
  an earlier one's word when both land in the same bank. At a multiple only the first two bytes are written, which
  come from the first two texels, and in every case analysed — sizes of four and eight bits with and without a
  wrapping mask — two adjacent texels either fall in different banks or in the same word. Mirrored wraps and YUV's
  chroma step were not worked through. The machine's own copy path writes all four texels and is where the rule
  does its work.

### 11.4 A crash the save state found

Adding `GpuReport` as a public auto-property crashed the test host with an access violation — in
`MarsSaveStateTests`, far from anything to do with a device. The state serializer walks `DpInterface`'s fields, and
an auto-property's backing field is a field it had never been told to skip. The fix is a `[SkipInState]` field, and
the lesson is worth stating plainly because the failure pointed nowhere near its cause: **on a class the state walks,
a new auto-property is new machine state unless it is said otherwise.**

### 11.5 A crash found on the way, not caused by it

At four, with deferred presentation, `pacebench` fails in the scan-out: *"The scan reached frame buffer address
DA9400 outside the lines captured for it."* It fails with the device off as well, and it fails identically, at the
same address, on `db4b74d`, a build from before any of this work. So it is a defect in `Vi.ReachScaled`'s capture
of the memory at a multiple, present since that capture was written, and it is **reachable by a player**: deferred
presentation is on by default, so an internal resolution of four crashes the emulator. Reading `Walk` and
`ReachScaled` shows a reach that covers more lines than the walk fetches, so the two must disagree in a number
reading did not find.

**Fixed the same day; `Mars_Video.md` §2.11.** Putting the captured range into the exception's message found it at
once: the capture started at 0x3DA2FE8 and the walk reached 0xDA9400, which is the same address with its top digit
gone. `Walk` multiplied the register's origin by the multiple squared and `Fetch` then aligned it with a mask that
also keeps only twenty-four bits. The immediate path had the same fault silently, drawing whatever lay at the
truncated address. **This means the agreement at four claimed in §11 was, for any game whose frame buffer is above a
megabyte, two paths reading the same wrong memory.** It was taken again after the fix: Super Mario 64, Ocarina of
Time, Majora's Mask and Wave Race 64 all agree at four, now on a correct picture, and the deferred path runs.

## 12. Phase 6: the measurement, and a criterion that was wrong about where the cost lived (2026-09-21)

The plan set its decision in advance: *2× within five per cent of 1×'s frame rate and 4× at full speed on this
machine, or the plan is recorded as not worth its dependency and retired.* By that sentence the device fails.
The measurements say something more useful than that sentence does.

### 12.1 What was measured

`pacebench`, flat out, from three in-game states, the device off and on interleaved and the order alternated, three
rounds each. The state hash was identical with the device on and off in every run, at every multiple: the machine
never sees the device, which is the plan's first promise kept.

| Game | Multiple | Mean, CPU | Mean, device | 90th pct, CPU | 90th pct, device | Full speed, CPU | Full speed, device |
|---|---|---|---|---|---|---|---|
| Super Mario 64 | 1× | 7.6 ms | 7.6 | 11.3 | 11.3 | 260% | 261% |
| Super Mario 64 | 2× | 19.1 | **16.6** | 25.9 | **18.3** | 104% | **121%** |
| Ocarina of Time | 2× | 27.9 | **20.2** | 48.5 | **31.9** | 71.7% | **98.6%** |
| Wave Race 64 | 2× | 20.8 | **14.0** | 56.4 | **15.5** | 79.9% | **119%** |
| Super Mario 64 | 4× | 77.6 | 66.4 | 118.0 | 76.3 | 25.8% | 30.1% |
| Ocarina of Time | 4× | 153.2 | 123.9 | 237.0 | 141.7 | 13.1% | 16.1% |
| Wave Race 64 | 4× | 67.5 | 55.2 | 137.8 | 68.6 | 24.7% | 30.2% |

The 4× rows are through the immediate scan-out, because the deferred one crashed at four on every build (§11.5);
they were taken before that was fixed, over the wrong lines of memory, and the timings stand because the walk does
the same work either way. After the fix, Mario at four through the **deferred** path, which is the default, runs at
58.6 milliseconds and 34 per cent of full speed on the CPU and **45.9 milliseconds and 43.5 per cent with the device**:
better than either immediate figure, because the scan-out's walk has a thread of its own there.

**At two the device is plainly worth having.** Ocarina goes from visibly slow to full speed, and the worst frames
improve most: Wave Race's ninetieth percentile falls from 56 milliseconds to 15.5.

**At four it is worth twenty per cent,** and four stays nowhere near real time either way.

### 12.2 Where four's time goes, measured rather than reasoned

Phase 3 measured the device shading a frame at four in two to four milliseconds, so the device was not the
bound. The same Mario state was run with the scan-out skipped (`SKIPRENDER=1`), which removes both the VI's walk and
the readback that feeds it:

| 4×, Super Mario 64 | CPU | Device |
|---|---|---|
| Scan-out on | 77.3 ms | 65.9 |
| Scan-out skipped | 29.7 | **6.5** |

**With the scan-out skipped, drawing at four costs the device path 6.5 milliseconds, the same as drawing at one.**
The device has made drawing at a multiple free, which is exactly and only what the plan built it for. What four
costs now is presentation: the CPU path spends about 47 milliseconds on its scan-out, of which the display
processor wait is 18 and the VI's walk over sixteen times the pixels is the rest; the device path spends about 59,
the same walk plus the readback of the picture and the join before it.

### 12.3 The decision

**The criterion conflated drawing with presentation.** It was written when the drawing was the cost, and it
assumed that removing the drawing's cost would bring the frame to one's rate. The drawing's cost has been removed —
6.5 milliseconds at four against 7.6 at one. What the criterion measures now is the VI scan-out and the readback,
neither of which the plan's phases 0 to 5 touched.

So the decision is not retirement. It is that **the device stays, and the next phase is the scan-out on the
device**, which the plan already names as its phase 7 and which removes both remaining costs at once: a compute
pass that walks the device's own memory into the finished picture needs no readback of that memory, only of the
picture, and does the VI's per-pixel filtering where the pixels already are. The criterion is carried over to it
unchanged, as the thing that phase must meet.

**What this does not establish.** One state per game and three rounds, on one machine with a discrete card. The
integrated adapter was not measured in a running game.
The deferred crash at four is fixed (§11.5). And the join inside the readback, which §11 suspected of putting the device path's single-threaded walk on the
critical path, is not separated from the readback's own cost by these runs; the scan-out on the device would remove
both, so it was not worth separating first.

## 13. Phase 7: the VI's walk on the device (2026-09-21)

§12 found that at four the device had made drawing cost what it costs at one, and that what remained was
presentation: the VI walking sixteen times the pixels on the processor, and the readback of the memory it walks.
This phase moves that walk onto the device, which removes both at once. The device walks its own memory into the
finished picture, so the memory never comes back to the host; only the picture does, and the frontend wants the
picture anyway.

### 13.1 The walk is per pixel, once two things are seen through

`Vi.Walk` reads as a loop full of state, and two pieces of it look like carries.

**The slot cache.** `Remembered` and `Sampled` keep each line's samples in two slots, keyed by the line and by whether
the row is "folded". It is a cache and not a carry — "a sample is a function of RDRAM and the registers alone", as
its own comment says — but its key holds only whether the fetch bug is *one*, while the value it stores is computed
with a bug that may be zero, one or two. If a value differed between zero and two, the cache would make a pixel
depend on which row asked first. It does not: every use of the bug in `Filter` and `Dither` compares it with one.
So the cache returns what a direct computation would, and an invocation can compute directly.

**The fetch bug.** `bug = repeats ? 2 : bug >> 1`, row after row. Since it only ever holds two, one or zero, it has a
closed form: two where this row reads its line again, one where the row before did, zero otherwise. The shader
computes that from the row's neighbours' lines.

Everything else — the fetch, the anti-aliasing filter with its runners-up, the de-dither, divot's median, the
resampling mix and gamma — is a function of the memory and the pixel's own coordinates. The gamma table is computed
in the shader from `Vi.Root`, the reference's integer square root, rather than uploaded, since it is integers
throughout and exact.

### 13.2 When the device walks, and why then

A deferred scan is captured on the machine's thread and walked on another. The walk cannot submit to the device
from that other thread: the rasteriser submits from the leading worker while the next frame draws, and one command
buffer does not take two threads. So the device walks **at capture time**, on the machine's thread, after the join
that makes the drawing complete and the device idle, and writes its picture into the job. The deferred walk then
only copies that picture into the raster: one block copy a row for the shown columns, and the few dark columns
either side cleared without touching their coverage, which is what `Walk` does to them.

The immediate path does the same inline. Neither path reads back the memory at the multiple any more when the device
holds it; `ReadBackScaled` is now only the fallback's.

### 13.3 Coverage, and the two survivors

`The_scan_out_on_the_device_is_the_cpus_byte_for_byte` draws a shaded scene into the memory at the multiple through
the whole interface, once with the device and once without, and scans it both ways under thirteen VI
configurations: every anti-aliasing mode, the de-dither, divot and gamma on and off, resampling steps that give every
fraction, a 625-line mode, and four thirty-two-bit modes over a thirty-two-bit scene — at two, three and four, on the
immediate path and the deferred one. The whole frame is compared byte for byte.

Fourteen mutants of `scan.comp`. Twelve were caught. The first run had nine VI configurations, all sixteen-bit, and
three survived; the reasons are the part worth keeping.

- **The filter's rounding cannot be seen through a sixteen-bit frame buffer, and this is arithmetic.** `Pull` adds
  four before shifting right by three. A sixteen-bit pixel's channels are five bits moved up three, so every value
  the filter sees is a multiple of eight, the difference it scales is a multiple of eight, and adding four can never
  carry into the shift. Only a thirty-two-bit pixel, with eight-bit channels, distinguishes the two. So did
  **the wide coverage** survive, for the plainer reason that no mode read a thirty-two-bit pixel at all. Four
  thirty-two-bit modes were added and both mutants, and a third on the wide fetch's blue byte, died.
- **The runner-up's tie-break is equivalent, proven by exhaustion.** `Runners` displaces its leader on `>`; with `>=`
  a tie displaces as well. The function only ever compares values, so every array of up to seven values — the
  filter's centre and its six neighbours — over seven distinct values covers every ordering there is, ties included.
  That is 873,612 arrays, and the two agree on all of them.

### 13.4 The number

`pacebench`, flat out, deferred presentation, the device off and on interleaved and the order alternated, three
rounds each; medians. Each game's state hash was the same across all eighteen of its runs, at one, two and four,
with and without the device.

| Game | 1× | 2×, CPU | 2×, device | 4×, CPU | 4×, device |
|---|---|---|---|---|---|
| Super Mario 64 | 7.57 ms, 263% | 19.55, 102% | **8.39, 237%** | 58.07, 34.4% | **12.53, 159%** |
| Ocarina of Time | 10.55, 189% | 27.96, 71.4% | **10.61, 188%** | 85.93, 23.3% | **14.06, 142%** |
| Wave Race 64 | 7.60, 219% | 20.82, 80.0% | **9.94, 167%** | 60.65, 27.5% | **13.92, 120%** |

**The plan's criterion, carried over from §12 unchanged, is met for four and in part for two.** *4× at full speed on
this machine*: all three games, the slowest at 120 per cent. *2× within five per cent of 1×'s frame rate*: Ocarina,
at 0.6 per cent; Mario misses at 11 per cent and Wave Race at 31. That part of the criterion is stricter than it
reads, since one runs flat out at two to two and a half times full speed here, so "within five per cent of it" at two
means well over twice real time. In play, which is paced, every game at two and at four now runs above full speed
with room to spare; the worst frames improve most, Wave Race's ninetieth percentile at four falling from 203
milliseconds to 24.

The phase-6 table (§12.1) measured the device path before this phase at two and four; against it the device's own
frame at four fell from 45.9 milliseconds to 12.5 for Mario.

### 13.5 What is left, and what this does not cover

**What four still costs** over one is 5 milliseconds for Mario and Wave Race and 3.5 for Ocarina. *(The attribution
that follows was reasoned, not measured, and §14 retires it: the largest piece was the last batch's shading, waited
for at the scan.)* It is no longer drawing or walking. It is the picture's readback — nineteen megabytes at four — two host copies of it, one into the
job and one into the raster, and the join before the scan. The copies can be halved by writing the raster directly
on the immediate path and double-buffering the device's output for the deferred one; the readback goes away only if
the frontend presents the device's image itself, which is the remaining half of the plan's phase 7 and is not
started.

**Not covered:** the integrated adapter in a running game; `Vi.Average`, the antialiasing setting's downsampling,
which still runs on the host after the walk; the borders and the two frames of grace, which the host writes as
before; and more than one state a game.

## 14. After phase 7: where four's remaining cost was, and a submission left pending (2026-09-21)

§13.5 said what four still cost over one, about five milliseconds, and named where: the picture's readback, two host
copies of it, and the join. **That attribution was a reasoned one, and it was mostly wrong.** Timed with temporary
stopwatches around each piece on the machine's thread (Super Mario 64, four, the device on, deferred presentation,
400 frames, of which 200 draw), a drawn frame spent:

| Piece | ms |
|---|---|
| the join before the scan | 1.3 |
| the last batch: binning on the host | 1.1 |
| the last batch: staging | 0.3 |
| the last batch: submitted and waited for | 3.0 |
| the walk and its transfer | 1.5 |
| the copy into the job | 0.7 |

The largest piece was not presentation at all but **the frame's shading**, which reaches the device only when the
scan flushes the batch, and which the machine's thread therefore waited through. The copies §13.5 proposed to halve
were the smallest two.

### 14.1 The tile was the wrong shape

`shade.comp` gave each 8×8 tile of the image one workgroup, one invocation a pixel, and walked the tile's rows in
drawing order. A row is one line, so of the sixty-four invocations that read each entry at most eight could use it;
the rest compared its line with their own and moved on, and a wave waits for its slowest lane. The host's binning
paid the same factor: a row of span L was listed about L/8 + 1 times.

Tiles of 64×1 remove both. Every invocation of a workgroup shares its line, so an entry is useful to all of them
that fall in its span, and a row is listed about L/64 + 1 times. The rows of Mario at four average some 115 pixels,
which predicts about 5.5 times fewer entries; the count measured was 286,931 against 50,856 a frame, 5.6. The
machine's flush halved (2.22 to 1.10 ms a frame) and RunFrame at four fell from 12.53 to 11.17 ms, with the state
hash unchanged. Nothing in the shader depended on the square: a pixel's writes and reads are its own, and each pixel
still meets its rows in drawing order, which is the one thing the binning must keep.

128×1 measured the same as 64 and 256×1 slightly worse, so the tile stays at 64, the width every device offers,
MoltenVK's included.

### 14.2 The last batch and the walk go out together, and nobody waits for them

`GpuDevice` gained one submission that is not waited for (`SubmitPending`), beside the waiting one. §1 had put that
off until something needed it; this is the measurement that did. The scan now stages the frame's last batch and
submits it with the walk and the walk's transfer, then returns. The deferred walk's thread waits for the device
(`ScannedPicture`) and copies the picture into the raster **from the mapped buffer itself**, so the copy into the
job is gone as well.

Three things make that sound, each for a different hazard:

- **Barriers that reach past their submission.** Every command is followed by a barrier over compute and transfer,
  and a barrier's second scope is everything later in submission order, other submissions included. So what is
  submitted after a pending walk, including what stages no batch (a clear on a state read, a readback), waits for it
  on the device. An opening barrier on every submission was written first, in the belief that a submission's
  barriers stop at its end; §14.4 shows it equivalent, and it was removed.
- **Staging waits for the pending submission.** The batch's host staging buffers and the shading program's one
  descriptor set belong to the pending submission until it finishes, so `Stage` waits for it before rewriting
  either. In a game this wait falls on the leading drawing worker in the next frame, by when the device has long
  finished.
- **A presentation join before any rebuild.** The walk's thread reads memory the device owns, so the device must
  outlive it: `MarsCore.ApplyMultiple` joins the deferred walk before changing the multiple or the device. The
  hazard this closes (a settings change disposing the buffer under a walk still reading it) is closed by
  construction and **was not demonstrated**: it needs a walk slowed to the length of a setting change, and there is
  no deterministic way to arrange that without instrumenting the thread.

Every submission also ends with a barrier to the host. A fence makes the device's own accesses complete, not
visible to a host read of mapped memory; the specification asks for the barrier, the driver here did not need it,
and no test can tell the two apart on this machine (§14.4).

### 14.3 Two defects found on the way, both older than this work

**A state read did not empty the device's memory.** `DpInterface.ResetScaled` zeroes the shadow at the multiple when
a state is read, and the device's copy was never told: `GpuRasteriser.Clear` was called only by its constructor, at
HEAD and since phase 5. After a load the device kept the old scene wherever the new one did not reach, and any rows
recorded before the load were still queued and would be drawn after it.
`After_a_state_is_read_the_device_holds_what_the_cpu_path_holds` failed on both counts before the fix (byte 0x800102,
0x84 on the device and 0 on the CPU) and passes after it. Each half of `Clear` has its own variant: without the
dropped batch the recorded case fails, without the fill both do.

**A repeated capture walked again on the device showed nothing it should.** A capture with the device copies no
scaled bytes, since the device walks its own memory; a capture that repeats the last returned before walking at all.
With `SkipRepeatedScans` off, the walk that followed read a scaled capture that had never been taken. Reachable in
Mistress by turning that setting off with the device on; since 4ddc4a8. The repeat now reuses the device's last
picture when the interface's count of scans (`ScanOuts`, which a rebuild also advances) says nothing has replaced it,
and walks on the device again otherwise. `A_repeated_capture_walked_again_on_the_device_is_the_cpus` failed before
(from byte 64 at two, 128 at four); its variant with another scan between catches a mutant that always reuses the
picture. Whether the device holds the multiple is now part of a repeat's shape, so turning the device on or off
between two scans makes the second a fresh capture.

### 14.4 The validation layer as a witness, and how it was made one

The Khronos validation layer (`vulkan-validation-layers` 1.4.341) was installed for this work, to have a witness
for ordering that does not depend on timing. **Its default configuration is blind to exactly the hazards this phase
has.** With synchronization validation turned on by the variable most documentation gives
(`VK_KHRONOS_VALIDATION_VALIDATE_SYNC`), and then by the layer's own current one (`VK_LAYER_VALIDATE_SYNC=1`), a
mutant that removed the barrier between the staging copies and the shading dispatch corrupted 331 bytes of a
recorded Mario frame and the layer said nothing. Core validation was working (an oversized copy was reported at
once), so the silence was specific: synchronization validation counts a shader's storage-buffer accesses only when
`syncval_shader_accesses_heuristic` is on, and every access this rasteriser makes is one. With it on, the same mutant
is reported as a read-after-write at the dispatch, naming the binding. The working configuration is therefore

    VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation VK_LAYER_VALIDATE_SYNC=1 VK_LAYER_SYNCVAL_SHADER_ACCESSES_HEURISTIC=1

with `VK_KHRONOS_VALIDATION_LOG_FILENAME` when the process is a test host, whose native output does not reach the
test log. That the same mutant also once produced a byte-identical frame is the reason to prefer the layer to the
oracle for ordering: a race that loses is invisible to a comparison.

Under that configuration the tree with the new tiles alone was silent over the three recorded frames at two and four
and over every GPU test, and the tree with the pending submission as well was silent over every GPU,
deferred-presentation and settings test.

**The mutants**, each run through the scan-out oracle, the repeated capture and
`A_walk_left_pending_shows_the_frame_it_captured` (a capture, then either the next frame drawn and pushed to the
device or a state read, then the walk), under the layer:

| Mutant | Tests | Layer | Reading |
|---|---|---|---|
| the walk's thread does not wait for the device | 8 of 14 fail | silent | caught |
| staging does not wait for the pending submission | all pass | silent | not demonstrable here |
| no opening barrier on a submission | all pass | silent | equivalent |
| no barrier to the host | all pass | silent | not demonstrable here |

*The opening barrier is equivalent, and was removed.* Every command `GpuCommands` records is followed by a barrier
over compute and transfer, and a barrier's second scope is everything later in submission order, other command
buffers included. The last command of a pending walk is such a command, so whatever is submitted next already
waits for it. The layer, which reasons about exactly this, agrees. The opening barrier was written in the belief that
a submission's barriers stop at its end; they do not.

*The staging wait is required by the specification and cannot be shown to matter on this machine.* Without it the
next batch rewrites staging a pending submission copies from, and a descriptor set it has bound. Neither happened in
time to be seen: the device finishes the pending batch in well under a millisecond, and recording the next scene
takes the host longer. The layer does not track host writes to mapped memory and did not report the descriptor
update. The wait is kept on the specification's authority, and this is recorded as an argument, not a result.

*The barrier to the host* is the same case: the memory here is coherent and cached, so the driver makes the device's
writes visible without it, and nothing the layer checks can tell.

The earlier interrupted run of these mutants is worth one sentence, because it went wrong in the way
`reference_mutation_runner_trap` warns of: a command that was interrupted ran twice, its second copy saved the
"clean" backup while the first had a mutant applied, and every restore afterwards put the mutant back. It was found
because a later mutant would not apply, and the runner now refuses to start unless its backups contain every piece a
mutant removes.

### 14.5 `Compose`'s alpha

The deferred walk's thread, once it no longer copied through the job, spent most of its time in `MarsCore.Compose`,
which copies the raster into the frame a line at a time, twice for a progressive field, and sets each pixel's fourth
byte to opaque one byte at a time. It now copies each line once with the alpha ORed in a vector at a time and copies
the finished line for the doubled row. Measured in isolation on the same raster (four 400-iteration trials, the
minimum, the machine otherwise loaded): 0.079 to 0.018 ms at one, 0.34 to 0.068 at two, 1.63 to 0.45 at four, and
byte-identical output at every multiple. The first measurements showed the new loop slower at one and two; that was
the JIT's first tier, and disappeared with warming. This is the host's path at every multiple and at one, device or
not, so the whole Mars suite (3,517 tests) and the golden probe (Super Mario 64 and Ocarina, identical for all 600
frames) were run over it.

### 14.6 The number

`pacebench`, flat out, deferred presentation, HEAD (4ddc4a8) and this work interleaved and the order alternated,
three rounds; medians. The device is on at two and four and off at one. Each game's state hash was the same across
all eighteen of its runs.

| Game | | 1× | 2× | 4× | 4×, 90th percentile |
|---|---|---|---|---|---|
| Super Mario 64 | HEAD | 265% | 237% (8.41 ms) | 159% (12.57) | 22.4 ms |
| | now | 265% | **272% (7.31)** | **192% (10.38)** | **15.1** |
| Ocarina of Time | HEAD | 187% | 188% (10.59) | 141% (14.17) | 34.2 |
| | now | 189% | **213% (9.35)** | **168% (11.85)** | **23.6** |
| Wave Race 64 | HEAD | 216% | 167% (9.96) | 120% (13.93) | 24.2 |
| | now | 218% | **191% (8.70)** | **150% (11.11)** | **14.2** |

One is unchanged, as it should be: nothing here runs at one except `Compose`, whose saving there is a
twentieth of a millisecond. **Two is now faster than one for Mario and Ocarina**, which is not a paradox: at a
multiple the device walks the picture, while at one the processor walks it on the deferred thread, and the next
frame's join waited on that walk for 1.4 milliseconds a frame in Mario, measured with the same stopwatches. The
plan's criterion for two, *within five per cent of one*, is now met by Mario and Ocarina and missed by Wave Race at 12
per cent. Four is now at 150 per cent or more for all three, and its ninetieth percentile fell by a third.

### 14.7 What is left, and what this does not cover

**At four**, what remains over two is the deferred walk's own length: the wait for the device's shading of the last
batch and its walk, then the copy into the raster and `Compose`, a little over seven milliseconds of a drawn frame at
the last stopwatch reading (before `Compose` was vectorised), which the next frame's join partly waits for. The
readback of the picture itself is unchanged; only presenting the device's image in the frontend would remove it.

**At one**, the processor's walk on the deferred thread is now the visible cost of the path, as the paragraph above
the table says; nothing in this phase touches it.

**Not covered:** the integrated adapter in a running game; `Vi.Average`, still on the host; the frontend's own
per-frame cost, which none of these tools measures, since `pacebench` has no render thread; the hazard the
presentation join closes (§14.2), argued and not demonstrated; and the staging wait and the host barrier (§14.4),
required by the specification and not observable on this machine.

## 15. The antialiasing average on the device, over a raster the device keeps (2026-09-21)

The antialiasing setting draws at the multiple and shows the picture averaged back down: `Vi.Raster` box-averages
the raster at the multiple on the host, after the walk, on the deferred thread. §14 left it there. Measured in Super
Mario 64 (`pacebench`, flat out, the device on), drawing at four and showing at one cost 12.36 ms a frame against
10.25 for drawing and showing at four; at two shown at one, 8.06 against 7.09; at four shown at two, 13.51 against
10.25. `Vi.BoxAverage` alone, timed over a raster of the same size, is 1.85 ms at two averaged by two, 7.4 at four
averaged by two and 4.9 at four by four, and the whole picture at the multiple was being read back to be averaged.

### 15.1 Why the picture could not simply be averaged where it is walked

The average is over the **raster**, not the picture, and the raster is more than this scan's walk. In an interlaced
mode it holds this field's rows interleaved with the last field's; the borders are darkened into it every scan; lines
the picture no longer covers keep their old contents for two frames of grace before they are cleared (Mars_Video.md
§2.4); a blank clears it. A square of the average can straddle both fields, so the device's picture of one field is
not enough to compute it. An exact port therefore keeps **the raster at the multiple on the device** whenever the
device averages, and sends it every change the host would have made to its own.

### 15.2 How the host's raster becomes the device's

Four things change the raster at the multiple, and each has its device counterpart:

| On the host | On the device |
|---|---|
| a blank clears it | the next submission fills it with zero |
| `Darken(line, from, count)` clears the same lines and columns at the multiple | recorded as spans of the device's raster, cleared by `clear.comp` before the walk |
| the walk writes the picture, a dark column's colour cleared and its coverage kept | `scan.comp` writes into the raster at `Walk`'s own placement, by the same rule |
| a new multiple starts a zeroed raster | a raster of a new size starts zeroed |

The average is `average.comp`, `BoxAverage` restated in integers, and only its result comes back: at four averaged
by four, 1.6 MB instead of the 9.8 MB picture. The clears, the walk and the average go out as one submission left
pending, as §14's walk does, and `Raster` reads the result through the same wait.

**When the submission goes.** Clears accumulate until something needs the raster: a walk (the capture, or the
immediate scan), or a scan with no signal, which darkens without walking and is still shown. A scan that repeats the
last is the one subtle case. The processor's path walks it again after this scan's clears, so a blank between the
two, which a repeat's shape does not see, is undone by the repeat; the device must do the same, so a repeat with
clears pending is walked again on the device. Nearly every scan darkens its borders, so that rule alone would walk
every repeated frame the frontend then throws away; `Capture` therefore takes `walkRepeats`, which `MarsCore`
passes as the inverse of `SkipRepeatedScans`.

**Entering and leaving.** When the device comes on over a raster the processor made, that raster seeds the device's,
so the other field and the held lines survive the change. When it goes off, the host's raster has been stale since
the device took it, and is cleared, as a blank would be: **the first frames after turning the device off show the
current field alone where the processor's path would still show the last one.** That is a deviation, it lasts at most
the two frames of grace, and it was chosen over reading back the whole raster at every scan to keep the host's
current.

### 15.3 Coverage

`Averaging_on_the_device_is_the_cpus_scan_after_scan` runs one sequence of twenty-three scans on two machines in step,
at two averaged by two, four by two and four by four, immediate and deferred, comparing the averaged frame after each:
a bordered picture, a shorter one whose lower lines expire, interlaced fields alternating, a new scene drawn between
fields, a blank and a second blank, a picture with no signal, a thirty-two-bit mode with gamma, and scans repeated so
that the repeat's path runs with clears pending. `Averaging_moved_onto_the_device_keeps_the_raster_the_processor_made`
turns the device on between interlaced fields over a raster the processor made.

Nine mutants, all killed in the end; three of them only after the sequence was changed, and how is the useful part:

| Mutant | Killed by |
|---|---|
| the average does not round | every case |
| a dark column's coverage cleared, not kept | every case |
| the clear shader clears nothing | every case |
| no spans sent for `Darken` | every case |
| the walk writes the host's raster, not the device's | every case |
| the lower field's offset dropped | every case |
| no seed on entering | the entering test only, as designed |
| a blank's clear not sent | only once the sequence had a blank over a **smaller** picture |
| a repeat with clears pending not walked again | only once the sequence had a repeat after a blank **followed by a scan that shows without walking** |

The first sequence had a blank over a picture covering the whole frame, whose walk rewrote everything the blank had
cleared, so the clear was invisible. The repeat case needed two things at once: a blank with the geometry of the
scan before it (a repeat's shape does not include blanking), so that the repeat runs with a clear pending; and then a
scan whose clears reach the device without a walk rewriting them. The first attempt at that third scan was a picture
with no columns, and it failed to kill the mutant because `Borders` darkens every line for a picture of no width,
on both paths alike; a picture starting past the raster's right edge also has no signal, but darkens nothing, and the
held lines still show, which is where the two paths differ. The repeat mutant dies in the deferred cases only, since the immediate scan has
no repeat path.

In play, `framedump` over Super Mario 64 and Ocarina of Time from their play states, deferred, at one averaged by
two, one by four and two by two: all thirty-six frames compared were byte for byte the processor's, with the device
in use on every device run.

### 15.4 The number

`pacebench`, flat out, deferred presentation, the device on; 55c5e05 and this work interleaved and the order
alternated, three rounds, medians. "Drawn at 4, shown at 1" is the antialiasing setting at four with the resolution
at one. Each game's state hash was the same across every run.

| Game | Drawn at, shown at | 55c5e05 | now |
|---|---|---|---|
| Super Mario 64 | 2, 1 | 245% (8.12 ms) | **282% (7.05)** |
| | 4, 1 | 164% (12.15) | **248% (8.04)** |
| | 4, 2 | 147% (13.54) | **239% (8.35)** |
| | 4, 4 | 190% (10.51) | 195% (10.22) |
| Ocarina of Time | 2, 1 | 201% (9.92) | **218% (9.16)** |
| | 4, 1 | 155% (12.91) | **201% (9.90)** |
| | 4, 2 | 144% (13.86) | **193% (10.34)** |
| | 4, 4 | 169% (11.78) | 170% (11.75) |
| Wave Race 64 | 2, 1 | 182% (9.12) | **195% (8.54)** |
| | 4, 1 | 139% (12.00) | **174% (9.56)** |
| | 4, 2 | 131% (12.72) | **171% (9.74)** |
| | 4, 4 | 152% (10.98) | 152% (10.94) |

The last row of each game, drawn and shown at four, does not average and is unchanged, as it should be. Averaging
now costs less than showing at the multiple it averages from: Mario drawn at four and shown at one runs at 248 per
cent against 195 for the same drawing shown at four, because what comes back and what the host composes are a
sixteenth of the size. Drawn at two and shown at one, Mario and Ocarina are faster than at one on the processor
alone (§14.6), for §14's reason.

### 15.5 What this does not cover

**The transition off the device** (§15.2) deviates from the processor's path for up to two frames, and no test holds
it; the entering transition is tested. **The immediate path's repeat** has no repeat logic to test. **The average of
the raster lines past the frame's height** is computed and sent back although `Raster` reads only the frame's first
lines, which at four averaged by two is 6.4 MB where a progressive frame needs 2.5; trimming it needs the frame height at
submission, which the VI has, and was left for simplicity. The other half of the plan's phase 7, the frontend
presenting the device's image, is still not started; with the average on the device, what it would now remove is a
transfer of at most the frame itself, at one averaged by four 1.6 MB.
