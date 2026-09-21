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
already moves the scan-out off it, and not an asynchronous layer built before anything needs it.

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

At a flush the rows are binned into tiles of 8×8 pixels over the batch's bounding box, by counting, summing and
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
