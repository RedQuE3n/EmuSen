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
