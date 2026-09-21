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
element. The test is a theory over whatever devices the machine offers, so a machine with only llvmpipe still runs
it and a machine with none reports that and passes, since that machine is the CPU path's.

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
